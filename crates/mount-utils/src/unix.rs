use crate::{
  runner::{make_bind_opts_sensitive, MounterImpl},
  utils::{join, PathExt},
  Arg, MountError, MountPoint, Result, DEFAULT_MOUNT_COMMAND,
};
use crossbeam::{scope, select};
use duct::cmd;
use smallvec::SmallVec;
use std::{
  ffi::OsString,
  fmt::Write,
  fs, io,
  num::NonZeroUsize,
  os::unix::fs::MetadataExt,
  path::{Path, PathBuf},
  time::Duration,
};
use tracing::{debug, error, info, warn};
use which::which;

const PROC_MOUNTS_PATH: &str = "/proc/mounts";
const PROC_MOUNT_INFO_PATH: &str = "/proc/self/mountinfo";
const MAX_LIST_TRIES: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(3) };
const EXPECTED_FIELDS_PER_MOUNTS_LINE: usize = 6;
const EXPECTED_AT_LEAST_FIELDS_PER_MOUNT_INFO: usize = 10;
const COMMON_MAX_FIELDS_PER_MOUNT_INFO: usize = 20;

pub struct OsMounter {
  mounter_path: Arg<Path>,
  systemd: bool,
}

impl MounterImpl for OsMounter {
  fn new(mounter_path: Arg<Path>) -> Result<Self> {
    Ok(OsMounter {
      mounter_path,
      systemd: Self::detect_systemd()?,
    })
  }

  fn mount_sensitive(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: smallvec::SmallVec<[Arg; 4]>,
    sensitive_options: smallvec::SmallVec<[Arg; 4]>,
  ) -> Result<()> {
    self.mount_sensitive_impl(
      source,
      target,
      fstype,
      &options.into_iter().map(Into::into).collect::<Vec<_>>(),
      &sensitive_options
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>(),
      true,
    )
  }

  fn mount_sensitive_without_systemd(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: smallvec::SmallVec<[Arg; 4]>,
    sensitive_options: smallvec::SmallVec<[Arg; 4]>,
  ) -> Result<()> {
    self.mount_sensitive_impl(
      source,
      target,
      fstype,
      &options.into_iter().map(Into::into).collect::<Vec<_>>(),
      &sensitive_options
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>(),
      false,
    )
  }

  fn unmount(&self, target: Arg<Path>, force_after: Option<Duration>) -> Result<()> {
    info!("Unmounting {}", target.display());
    let cmd = cmd!("umount", target.clone()).stderr_to_stdout();

    let run_forced = match force_after {
      None => {
        cmd.run()?;
        false
      }
      Some(duration) => {
        let handle = cmd.start()?;
        let handle = &handle;
        scope::<'_, _, Result<bool>>(|s| {
          let (sender, receiver) = crossbeam::channel::bounded(1);

          s.spawn(move |_| {
            let result = handle.wait();
            let _ = sender.send(result);
          });

          let succeeded = select! {
            recv(receiver) -> _ => true,
            default(duration) => false,
          };

          drop(receiver);
          if !succeeded {
            let _ = handle.kill();
          }

          Ok(!succeeded)
        })
        .map_err(|e| MountError::new(format!("Failed to spawn threads: {:?}", e)))??
      }
    };

    if run_forced {
      cmd!("umount", "-f", target).run()?;
    }

    Ok(())
  }

  fn list(&self) -> Result<Vec<MountPoint>> {
    list_proc_mounts(PROC_MOUNTS_PATH.into())
  }

  fn is_likely_not_mount_point(&self, file: Arg<Path>) -> Result<bool> {
    let stat = fs::metadata(&*file)?;
    let parent = file
      .parent()
      .ok_or_else(|| MountError::new("file does not have parent"))?;
    let parent_stat = fs::metadata(parent)?;
    // If the directory has a different device as parent, then it is a mountpoint.
    if stat.dev() != parent_stat.dev() {
      return Ok(false);
    }

    Ok(true)
  }

  fn get_mount_refs(&self, path: Arg<Path>) -> Result<Vec<Arg<Path>>> {
    match mount_info(&path)? {
      MountPathInfo::Exists => (),
      MountPathInfo::NotExists => return Ok(Vec::new()),
      MountPathInfo::Corrupted => {
        warn!(
          "get_mount_refs found corrupted mount at {}, treating as unmounted path",
          path.display()
        );
        return Ok(Vec::new());
      }
    }

    let realpath = path.canonicalize()?;
    search_mount_points(&realpath, PROC_MOUNT_INFO_PATH.as_ref())
  }
}

impl OsMounter {
  // detectSystemd returns true if OS runs with systemd as init. When not sure
  // (permission errors, ...), it returns false.
  // There may be different ways how to detect systemd, this one makes sure that
  // systemd-runs (needed by Mount()) works.
  fn detect_systemd() -> Result<bool> {
    let systemd_run_path = match which("systemd-run") {
      Ok(p) => p,
      Err(_) => {
        info!("Detected OS without systemd");
        return Ok(false);
      }
    };

    // Try to run systemd-run --scope /bin/true, that should be enough
    // to make sure that systemd is really running and not just installed,
    // which happens when running in a container with a systemd-based image
    // but with different pid 1.
    let result = cmd!(
      systemd_run_path,
      "--description=Kubernetes systemd probe",
      "--scope",
      "true"
    )
    .stderr_to_stdout()
    .stdout_capture()
    .unchecked()
    .run()?;

    if result.status.success() {
      info!("Detected OS with systemd");
      Ok(true)
    } else {
      let output = std::str::from_utf8(&*result.stdout).unwrap_or("INVALID_UTF8_DATA");
      info!("Cannot run systemd-run, assuming non-systemd OS");
      debug!(
        "systemd-run output: {}, failed with: {:?}",
        output, result.status
      );
      Ok(false)
    }
  }

  fn mount_sensitive_impl(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: &[Arg],
    sensitive_options: &[Arg],
    systemd_mount_required: bool,
  ) -> Result<()> {
    let mut mounter_path = None;

    // Path to mounter binary if containerized mounter is needed. Otherwise, it is set to empty.
    // All Linux distros are expected to be shipped with a mount utility that a support bind mounts.
    let (bind, bind_opts, bind_remount_opts, bind_remount_opts_sensitive) =
      make_bind_opts_sensitive(options, sensitive_options);
    if bind {
      self.do_mount(
        mounter_path.clone(),
        DEFAULT_MOUNT_COMMAND.into(),
        source.clone(),
        target.clone(),
        fstype.clone(),
        &bind_opts,
        &bind_remount_opts_sensitive,
        systemd_mount_required,
      )?;
      self.do_mount(
        mounter_path,
        DEFAULT_MOUNT_COMMAND.into(),
        source,
        target,
        fstype,
        &bind_remount_opts,
        &bind_remount_opts_sensitive,
        systemd_mount_required,
      )?;
    } else {
      if matches!(
        fstype.to_str(),
        Some("nfs") | Some("glusterfs") | Some("ceph") | Some("cifs")
      ) {
        mounter_path = Some(self.mounter_path.clone());
      }

      self.do_mount(
        mounter_path,
        DEFAULT_MOUNT_COMMAND.into(),
        source,
        target,
        fstype,
        options,
        sensitive_options,
        systemd_mount_required,
      )?;
    }

    Ok(())
  }

  /// doMount runs the mount command. mounterPath is the path to mounter binary if containerized mounter is used.
  /// sensitiveOptions is an extension of options except they will not be logged (because they may contain sensitive material)
  /// systemdMountRequired is an extension of option to decide whether uses systemd mount.
  fn do_mount(
    &self,
    mounter_path: Option<Arg<Path>>,
    mount_cmd: Arg<Path>,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: &[Arg],
    options_sensitive: &[Arg],
    systemd_mount_required: bool,
  ) -> Result<()> {
    let mut mount_cmd = mount_cmd;
    let (mut mount_args, mut log_str) =
      make_mount_args_sensitive(source, target.clone(), fstype, options, options_sensitive);

    if let Some(mounter_path) = mounter_path {
      // prefix with mounter_path
      mount_args = Some(Arg::from(mounter_path.clone()))
        .into_iter()
        .chain(mount_args.into_iter())
        .collect();
      log_str = {
        let suffix = log_str;
        let mut new = String::from(mounter_path.to_string_lossy());
        new.reserve(suffix.len() + 1);
        new.push(' ');
        new.push_str(&suffix);
        new
      };
      mount_cmd = mounter_path;
    }

    if self.systemd && systemd_mount_required {
      // Try to run mount via systemd-run --scope. This will escape the
      // service where kubelet runs and any fuse daemons will be started in a
      // specific scope. kubelet service than can be restarted without killing
      // these fuse daemons.
      //
      // Complete command line (when mounterPath is not used):
      // systemd-run --description=... --scope -- mount -t <type> <what> <where>
      //
      // Expected flow:
      // * systemd-run creates a transient scope (=~ cgroup) and executes its
      //   argument (/bin/mount) there.
      // * mount does its job, forks a fuse daemon if necessary and finishes.
      //   (systemd-run --scope finishes at this point, returning mount's exit
      //   code and stdout/stderr - thats one of --scope benefits).
      // * systemd keeps the fuse daemon running in the scope (i.e. in its own
      //   cgroup) until the fuse daemon dies (another --scope benefit).
      //   Kubelet service can be restarted and the fuse daemon survives.
      // * When the fuse daemon dies (e.g. during unmount) systemd removes the
      //   scope automatically.
      //
      // systemd-mount is not used because it's too new for older distros
      // (CentOS 7, Debian Jessie).
      let t = self::add_systemd_scope_sensitive(
        "systemd-run".into(),
        target,
        mount_cmd,
        mount_args,
        log_str,
      );

      mount_cmd = t.0;
      mount_args = t.1;
      log_str = t.2;
    }

    info!(
      "Mounting cmd {} with arguments ({})",
      mount_cmd.display(),
      log_str
    );
    match cmd(mount_cmd, mount_args).stderr_to_stdout().run() {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("Mount failed: {:?}", e);
        Err(e.into())
      }
    }
  }
}

pub(crate) fn is_mount_point_match(mp: &MountPoint, dir: Arg<Path>) -> bool {
  let deleted_dir = format!("{}\\040(deleted)", dir.display());
  let deleted_dir: &Path = deleted_dir.as_ref();
  *mp.path() == dir || **mp.path() == *deleted_dir
}

// MakeMountArgsSensitive makes the arguments to the mount(8) command.
// sensitiveOptions is an extension of options except they will not be logged (because they may contain sensitive material)
fn make_mount_args_sensitive(
  source: Option<Arg<Path>>,
  target: Arg<Path>,
  fstype: Arg,
  options: &[Arg],
  options_sensitive: &[Arg],
) -> (Vec<Arg>, String) {
  // Build mount command as follows:
  //   mount [-t $fstype] [-o $options] [$source] $target
  let mut mount_args: Vec<Arg> = Vec::new();
  let mut log_str = String::new();

  if !fstype.is_empty() {
    mount_args.push("-t".into());
    mount_args.push(fstype.clone());
    write!(log_str, "-t {}", fstype.to_string_lossy()).unwrap();
  }

  if !options.is_empty() || !options_sensitive.is_empty() {
    let arg = join(options.iter().chain(options_sensitive.iter()), ",");
    let log_arg = options
      .iter()
      .map(|s| s.to_string_lossy())
      .collect::<Vec<_>>()
      .join(",");

    mount_args.push("-o".into());
    mount_args.push(arg);
    write!(log_str, " -o {}", log_arg).unwrap();
  }

  if let Some(source) = source {
    mount_args.push(source.clone().into());
    write!(log_str, " {}", source.display()).unwrap();
  }

  mount_args.push(target.clone().into());
  write!(log_str, " {}", target.display()).unwrap();

  (mount_args, log_str)
}

// AddSystemdScopeSensitive adds "system-run --scope" to given command line
// It also accepts takes a sanitized string containing mount arguments, mountArgsLogStr,
// and returns the string appended to the systemd command for logging.
fn add_systemd_scope_sensitive(
  systemd_run_path: Arg<Path>,
  mount_name: Arg<Path>,
  command: Arg<Path>,
  args: Vec<Arg>,
  mut log_str: String,
) -> (Arg<Path>, Vec<Arg>, String) {
  let description_arg = {
    let mut arg = OsString::with_capacity(
      "--description=Kubernetes transient mount for ".len() + mount_name.as_os_str().len(),
    );
    arg.push("--description=Kubernetes transient mount for ");
    arg.push(mount_name.as_os_str());
    Arg::from(arg)
  };
  let mut systemd_run_args: Vec<Arg> = vec![
    description_arg,
    "--scope".into(),
    "--".into(),
    command.into(),
  ];
  systemd_run_args.extend(args);
  log_str = {
    let mut new = systemd_run_args
      .iter()
      .map(|s| s.to_string_lossy())
      .collect::<Vec<_>>()
      .join(" ");
    new.reserve(log_str.len() + 1);
    new.push(' ');
    new.push_str(&*log_str);
    new
  };
  (systemd_run_path, systemd_run_args, log_str)
}

fn list_proc_mounts(mount_file_path: Arg<Path>) -> Result<Vec<MountPoint>> {
  let content = consistent_read(&*mount_file_path, MAX_LIST_TRIES)?;

  parse_proc_mounts(&content)
}

/// ConsistentRead repeatedly reads a file until it gets the same content twice. This is useful when reading files
/// in /proc that are larger than page size and kernel may modify them between individual read() syscalls.
fn consistent_read(path: &Path, attempts: NonZeroUsize) -> io::Result<Vec<u8>> {
  let mut old_content = fs::read(path)?;

  for _ in 0..attempts.get() {
    let new_content = fs::read(path)?;
    if new_content == old_content {
      return Ok(new_content);
    }

    // Files are different, continue reading
    old_content = new_content;
  }

  Err(io::Error::new(
    io::ErrorKind::Other,
    format!(
      "could not get consistent content of '{}' after {} attempts",
      path.display(),
      attempts.get()
    ),
  ))
}

fn parse_proc_mounts(content: &[u8]) -> Result<Vec<MountPoint>> {
  let mut out = Vec::new();
  let s = std::str::from_utf8(content)
    .map_err(|_| MountError::new("proc mounts contain invalid UTF8"))?;
  for line in s.lines() {
    if line.is_empty() {
      continue;
    }

    let fields = line
      .split_whitespace()
      .collect::<SmallVec<[&str; EXPECTED_FIELDS_PER_MOUNTS_LINE]>>();
    if fields.len() != EXPECTED_FIELDS_PER_MOUNTS_LINE {
      return Err(MountError::new(format!(
        "wrong number of fields (expected {}, got {})",
        EXPECTED_FIELDS_PER_MOUNTS_LINE,
        fields.len()
      )));
    }

    let device = Arg::from(fields[0].to_owned());
    let path = Arg::from(fields[1].to_owned());
    let ty = Arg::from(fields[2].to_owned());
    let opts = fields[3]
      .split(',')
      .map(|v| Arg::from(v.to_owned()))
      .collect();
    let freq = fields[4]
      .parse()
      .map_err(|e| MountError::new(format!("invalid freq: {:?}", e)))?;
    let pass = fields[5]
      .parse()
      .map_err(|e| MountError::new(format!("invalid pass: {:?}", e)))?;

    out.push(MountPoint {
      device,
      path,
      ty,
      opts,
      freq,
      pass,
    })
  }

  Ok(out)
}

enum MountPathInfo {
  Exists,
  NotExists,
  Corrupted,
}

fn mount_info(path: &Path) -> io::Result<MountPathInfo> {
  match path.metadata() {
    Ok(_) => Ok(MountPathInfo::Exists),
    Err(e) => match e.raw_os_error() {
      None => Err(e),
      Some(code) => match code {
        libc::ENOENT => Ok(MountPathInfo::NotExists),
        libc::ENOTCONN | libc::ESTALE | libc::EIO | libc::EACCES => Ok(MountPathInfo::Corrupted),
        _ => Err(e),
      },
    },
  }
}

fn search_mount_points(host_source: &Path, mount_info_path: &Path) -> Result<Vec<Arg<Path>>> {
  let mis = parse_mount_into(mount_info_path)?;

  // Finding the underlying root path and major:minor if possible.
  // We need search in backward order because it's possible for later mounts
  // to overlap earlier mounts.
  let (mount_id, root_path, major, minor) = mis
    .iter()
    .rev()
    .find_map(|mi| {
      host_source
        .strip_resolved_prefix(&mi.mount_point)
        .ok()
        .map(|p| (p, mi))
    })
    .map(|(path_extra, mi)| {
      let mount_id = mi.id;
      let root_path = mi.root.join(path_extra);
      let major = mi.major;
      let minor = mi.minor;
      (mount_id, root_path, major, minor)
    })
    .ok_or_else(|| {
      MountError::new(format!(
        "failed to get root path and major:minor for '{}'",
        host_source.display()
      ))
    })?;

  // let mut refs = Vec::new();
  // for mi in mis.into_iter() {
  //   if mi.id == mount_id {
  //     continue;
  //   }

  // }
  let refs = mis
    .into_iter()
    .filter(|mi| {
      mi.id != mount_id && mi.root == root_path && mi.major == major && mi.minor == minor
    })
    .map(|mi| mi.mount_point.into())
    .collect();
  Ok(refs)
}

struct MountInfo {
  /// Unique ID for the mount (maybe reused after umount).
  id: isize,
  /// The ID of the parent mount (or of self for the root of this mount namespace's mount tree).
  parent_id: isize,
  /// Major indicates one half of the device ID which identifies the device class
  /// (parsed from `st_dev` for files on this filesystem).
  major: isize,
  /// Minor indicates one half of the device ID which identifies a specific
  /// instance of device (parsed from `st_dev` for files on this filesystem).
  minor: isize,
  /// The pathname of the directory in the filesystem which forms the root of this mount.
  root: PathBuf,
  /// Mount source, filesystem-specific information. e.g. device, tmpfs name.
  source: String,
  /// Mount point, the pathname of the mount point.
  mount_point: PathBuf,
  /// Optional fieds, zero or more fields of the form "tag[:value]".
  optional_fields: Vec<String>,
  /// The filesystem type in the form "type[.subtype]".
  fs_type: String,
  /// Per-mount options.
  mount_options: Vec<String>,
  /// Per-superblock options.
  super_options: Vec<String>,
}

fn parse_error(name: &'static str, value: &str, line: &str) -> io::Error {
  io::Error::new(
    io::ErrorKind::Other,
    format!(
      "Failed to parse field {} from value '{}' in line: {}",
      name, value, line
    ),
  )
}

fn parse_mount_into(path: &Path) -> io::Result<Vec<MountInfo>> {
  let content = consistent_read(path, MAX_LIST_TRIES)?;
  let content = std::str::from_utf8(&content).map_err(|_| {
    io::Error::new(
      io::ErrorKind::Other,
      format!("invalid utf8 in file {}", path.display()),
    )
  })?;

  let mut infos = Vec::new();
  for line in content.lines() {
    if line.is_empty() {
      continue;
    }

    // See `man proc` for authoritative description of format of the file.
    let fields = line
      .split_whitespace()
      .collect::<SmallVec<[&str; COMMON_MAX_FIELDS_PER_MOUNT_INFO]>>();

    if fields.len() < EXPECTED_AT_LEAST_FIELDS_PER_MOUNT_INFO {
      return Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
          "wrong number of fields in (expected at least {}, got {}): {}",
          EXPECTED_AT_LEAST_FIELDS_PER_MOUNT_INFO,
          fields.len(),
          line
        ),
      ));
    }

    let id = fields[0]
      .parse()
      .map_err(|_| parse_error("id", fields[0], line))?;
    let parent_id = fields[1]
      .parse()
      .map_err(|_| parse_error("parent_id", fields[0], line))?;
    let (major, minor) = {
      let mm = fields[2].split(':').collect::<SmallVec<[&str; 2]>>();
      if mm.len() != 2 {
        return Err(io::Error::new(
          io::ErrorKind::Other,
          format!(
            "parsing '{}' failed: unexpected minor:major pair {:?}",
            line,
            mm.as_ref()
          ),
        ));
      }
      let major = mm[0]
        .parse()
        .map_err(|_| parse_error("major", mm[0], line))?;
      let minor = mm[1]
        .parse()
        .map_err(|_| parse_error("minor", mm[1], line))?;

      (major, minor)
    };
    let root = PathBuf::from(fields[3]);
    let mount_point = PathBuf::from(fields[4]);
    let mount_options = fields[5].split(',').map(ToOwned::to_owned).collect();

    // All fields until "-" are "optional fields".
    let mut optional_fields = Vec::new();
    let mut iter = fields.into_iter().skip(6);
    while let Some(field) = iter.next() {
      if field == "-" {
        break;
      }
      optional_fields.push(field.to_owned());
    }

    let fs_type = iter
      .next()
      .ok_or_else(|| {
        io::Error::new(
          io::ErrorKind::Other,
          format!("missing fs_type after - in line: {}", line,),
        )
      })?
      .to_owned();
    let source = iter
      .next()
      .ok_or_else(|| {
        io::Error::new(
          io::ErrorKind::Other,
          format!("missing source after - in line: {}", line,),
        )
      })?
      .to_owned();
    let super_options = iter
      .next()
      .ok_or_else(|| {
        io::Error::new(
          io::ErrorKind::Other,
          format!("missing super_options after - in line: {}", line,),
        )
      })?
      .split(',')
      .map(ToOwned::to_owned)
      .collect();

    infos.push(MountInfo {
      id,
      parent_id,
      major,
      minor,
      root,
      source,
      mount_point,
      optional_fields,
      fs_type,
      mount_options,
      super_options,
    })
  }

  Ok(infos)
}

#[cfg(test)]
mod tests {
  use super::{make_mount_args_sensitive, parse_proc_mounts};
  use crate::{fake::*, runner::MounterImpl, Arg, MountPoint};
  use std::{collections::HashSet, ffi::OsStr, path::Path};
  use test_case::test_case;

  #[test]
  fn read_proc_mounts() {
    let success_case = "
/dev/0 /path/to/0 type0 flags 0 0
/dev/1    /path/to/1   type1	flags 1 1
/dev/2 /path/to/2 type2 flags,1,2=3 2 2
";

    let mounts = parse_proc_mounts(success_case.as_ref()).expect("parse succeeded");
    assert_eq!(mounts.len(), 3);
    assert_eq!(
      mounts[0],
      MountPoint {
        device: "/dev/0".into(),
        path: "/path/to/0".into(),
        ty: "type0".into(),
        opts: vec!["flags".into()],
        freq: 0,
        pass: 0,
      }
    );
    assert_eq!(
      mounts[1],
      MountPoint {
        device: "/dev/1".into(),
        path: "/path/to/1".into(),
        ty: "type1".into(),
        opts: vec!["flags".into()],
        freq: 1,
        pass: 1,
      }
    );
    assert_eq!(
      mounts[2],
      MountPoint {
        device: "/dev/2".into(),
        path: "/path/to/2".into(),
        ty: "type2".into(),
        opts: vec!["flags".into(), "1".into(), "2=3".into()],
        freq: 2,
        pass: 2,
      }
    );

    let error_cases = &[
      "/dev/0 /path/to/mount\n",
      "/dev/1 /path/to/mount type flags a 0\n",
      "/dev/2 /path/to/mount type flags 0 b\n",
    ];
    for ec in error_cases {
      parse_proc_mounts(ec.as_ref()).expect_err(&format!("Error case '{}' should fail", ec.trim()));
    }
  }

  fn mp_simple(device: &'static str, path: &'static str) -> MountPoint {
    MountPoint {
      device: device.into(),
      path: path.into(),
      ty: "".into(),
      opts: Vec::new(),
      freq: 0,
      pass: 0,
    }
  }

  #[test]
  fn get_mount_refs() {
    let fm = FakeMounter::new(vec![
      mp_simple(
        "/dev/sdb",
        "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd",
      ),
      mp_simple(
        "/dev/sdb",
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd-in-pod",
      ),
      mp_simple(
        "/dev/sdc",
        "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd2",
      ),
      mp_simple(
        "/dev/sdc",
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod1",
      ),
      mp_simple(
        "/dev/sdc",
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod2",
      ),
    ]);

    let cases = &[
      (
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd-in-pod",
        &["/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd"] as &[&'static str],
      ),
      (
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod1",
        &[
          "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod2",
          "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd2",
        ],
      ),
      ("/var/fake/directory/that/doesnt/exist", &[]),
    ];

    for (mount_path, expected_refs) in cases {
      let refs = fm
        .get_mount_refs((*mount_path).into())
        .unwrap_or_else(|e| panic!("get_mount_refs failed for '{}': {:?}", mount_path, e));

      let actual = refs.into_iter().collect::<HashSet<_>>();
      let expected = (*expected_refs)
        .iter()
        .map(|p| Arg::<Path>::from(*p))
        .collect::<HashSet<_>>();

      assert_eq!(
        actual, expected,
        "get_mount_refs got wrong values for '{}'",
        mount_path
      );
    }
  }

  #[test]
  fn get_device_name_from_mount() {
    let fm = FakeMounter::new(vec![
      mp_simple("/dev/disk/by-path/prefix-lun-1", "/mnt/111"),
      mp_simple("/dev/disk/by-path/prefix-lun-1", "/mnt/222"),
    ]);

    struct Test {
      mount_path: Arg<Path>,
      expected_device: Arg<Path>,
      expected_refs: usize,
    }

    let tests = vec![Test {
      mount_path: "/mnt/222".into(),
      expected_device: "/dev/disk/by-path/prefix-lun-1".into(),
      expected_refs: 2,
    }];

    for test in tests {
      let (device, refs) = fm
        .get_device_name_from_mount(test.mount_path.clone())
        .unwrap_or_else(|e| {
          panic!(
            "get_device_name_from_mount({}): {:?}",
            test.mount_path.display(),
            e
          )
        })
        .unwrap_or_else(|| {
          panic!(
            "get_device_name_from_mount({}) returned None",
            test.mount_path.display()
          )
        });

      assert_eq!(device, test.expected_device);
      assert_eq!(refs, test.expected_refs);
    }
  }

  #[test]
  fn get_mount_refs_by_dev() {
    let fm = FakeMounter::new(vec![
      mp_simple(
        "/dev/sdb",
        "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd",
      ),
      mp_simple(
        "/dev/sdb",
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd-in-pod",
      ),
      mp_simple(
        "/dev/sdc",
        "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd2",
      ),
      mp_simple(
        "/dev/sdc",
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod1",
      ),
      mp_simple(
        "/dev/sdc",
        "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod2",
      ),
    ]);

    let cases = &[
      (
        "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd",
        &["/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd-in-pod"]
          as &[&'static str],
      ),
      (
        "/var/lib/kubelet/plugins/kubernetes.io/gce-pd/mounts/gce-pd2",
        &[
          "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod1",
          "/var/lib/kubelet/pods/some-pod/volumes/kubernetes.io~gce-pd/gce-pd2-in-pod2",
        ],
      ),
    ];

    for (mount_path, expected_refs) in cases {
      let refs = fm
        .get_mount_refs_by_dev((*mount_path).into())
        .unwrap_or_else(|e| panic!("get_mount_refs_by_dev({}): {:?}", mount_path, e));

      let actual = refs.into_iter().collect::<HashSet<_>>();
      let expected = (*expected_refs)
        .iter()
        .map(|p| Arg::<Path>::from(*p))
        .collect::<HashSet<_>>();

      assert_eq!(
        actual, expected,
        "get_mount_refs_by_dev got wrong values for '{}'",
        mount_path
      );
    }
  }

  mod search_mount_points {
    use super::*;
    use tempfile::tempdir;

    const BASE: &str = "
19 25 0:18 / /sys rw,nosuid,nodev,noexec,relatime shared:7 - sysfs sysfs rw
20 25 0:4 / /proc rw,nosuid,nodev,noexec,relatime shared:12 - proc proc rw
21 25 0:6 / /dev rw,nosuid,relatime shared:2 - devtmpfs udev rw,size=4058156k,nr_inodes=1014539,mode=755
22 21 0:14 / /dev/pts rw,nosuid,noexec,relatime shared:3 - devpts devpts rw,gid=5,mode=620,ptmxmode=000
23 25 0:19 / /run rw,nosuid,noexec,relatime shared:5 - tmpfs tmpfs rw,size=815692k,mode=755
25 0 252:0 / / rw,relatime shared:1 - ext4 /dev/mapper/ubuntu--vg-root rw,errors=remount-ro,data=ordered
26 19 0:12 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime shared:8 - securityfs securityfs rw
27 21 0:21 / /dev/shm rw,nosuid,nodev shared:4 - tmpfs tmpfs rw
28 23 0:22 / /run/lock rw,nosuid,nodev,noexec,relatime shared:6 - tmpfs tmpfs rw,size=5120k
29 19 0:23 / /sys/fs/cgroup ro,nosuid,nodev,noexec shared:9 - tmpfs tmpfs ro,mode=755
30 29 0:24 / /sys/fs/cgroup/systemd rw,nosuid,nodev,noexec,relatime shared:10 - cgroup cgroup rw,xattr,release_agent=/lib/systemd/systemd-cgroups-agent,name=systemd
31 19 0:25 / /sys/fs/pstore rw,nosuid,nodev,noexec,relatime shared:11 - pstore pstore rw
32 29 0:26 / /sys/fs/cgroup/devices rw,nosuid,nodev,noexec,relatime shared:13 - cgroup cgroup rw,devices
33 29 0:27 / /sys/fs/cgroup/freezer rw,nosuid,nodev,noexec,relatime shared:14 - cgroup cgroup rw,freezer
34 29 0:28 / /sys/fs/cgroup/pids rw,nosuid,nodev,noexec,relatime shared:15 - cgroup cgroup rw,pids
35 29 0:29 / /sys/fs/cgroup/blkio rw,nosuid,nodev,noexec,relatime shared:16 - cgroup cgroup rw,blkio
36 29 0:30 / /sys/fs/cgroup/memory rw,nosuid,nodev,noexec,relatime shared:17 - cgroup cgroup rw,memory
37 29 0:31 / /sys/fs/cgroup/perf_event rw,nosuid,nodev,noexec,relatime shared:18 - cgroup cgroup rw,perf_event
38 29 0:32 / /sys/fs/cgroup/hugetlb rw,nosuid,nodev,noexec,relatime shared:19 - cgroup cgroup rw,hugetlb
39 29 0:33 / /sys/fs/cgroup/cpu,cpuacct rw,nosuid,nodev,noexec,relatime shared:20 - cgroup cgroup rw,cpu,cpuacct
40 29 0:34 / /sys/fs/cgroup/cpuset rw,nosuid,nodev,noexec,relatime shared:21 - cgroup cgroup rw,cpuset
41 29 0:35 / /sys/fs/cgroup/net_cls,net_prio rw,nosuid,nodev,noexec,relatime shared:22 - cgroup cgroup rw,net_cls,net_prio
58 25 7:1 / /mnt/disks/blkvol1 rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordere
";

    fn search_mount_points(source: &Path, mount_infos: &str, expected_refs: &[&'static str]) {
      let dir = tempdir().expect("failed to get tempfile");
      let file = dir.path().join(source.file_name().unwrap());
      std::fs::write(&file, mount_infos).expect("failed to write source to temp file");

      let refs = super::super::search_mount_points(source, &file)
        .unwrap_or_else(|e| panic!("failed to search mount points: {:?}", e));

      let actual = refs.into_iter().collect::<HashSet<_>>();
      let expected = (*expected_refs)
        .iter()
        .map(|p| Arg::<Path>::from(*p))
        .collect::<HashSet<_>>();

      assert_eq!(actual, expected, "search_mount_points got wrong values",);
    }

    #[test]
    fn dir() {
      let source = "/mnt/disks/vol1";
      let mount_infos = String::from(BASE);
      let expected_refs = &[];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn dir_used() {
      let source = "/mnt/disks/vol1";
      let mount_infos = String::from(BASE) + "
56 25 252:0 /mnt/disks/vol1 /var/lib/kubelet/pods/1890aef5-5a60-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test rw,relatime shared:1 - ext4 /dev/mapper/ubuntu--vg-root rw,errors=remount-ro,data=ordered
57 25 0:45 / /mnt/disks/vol rw,relatime shared:36 - tmpfs tmpfs rw
";
      let expected_refs = &["/var/lib/kubelet/pods/1890aef5-5a60-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test"];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn tmpfs_vol() {
      let source = "/mnt/disks/vol1";
      let mount_infos = String::from(BASE)
        + "120 25 0:76 / /mnt/disks/vol1 rw,relatime shared:41 - tmpfs vol1 rw,size=10000k
";
      let expected_refs = &[];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn tmpfs_vol_used_by_two_pods() {
      let source = "/mnt/disks/vol1";
      let mount_infos = String::from(BASE) +  "120 25 0:76 / /mnt/disks/vol1 rw,relatime shared:41 - tmpfs vol1 rw,size=10000k
196 25 0:76 / /var/lib/kubelet/pods/ade3ac21-5a5b-11e8-8559-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-8f263585 rw,relatime shared:41 - tmpfs vol1 rw,size=10000k
228 25 0:76 / /var/lib/kubelet/pods/ac60532d-5a5b-11e8-8559-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-8f263585 rw,relatime shared:41 - tmpfs vol1 rw,size=10000k
";
      let expected_refs = &[
				"/var/lib/kubelet/pods/ade3ac21-5a5b-11e8-8559-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-8f263585",
				"/var/lib/kubelet/pods/ac60532d-5a5b-11e8-8559-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-8f263585",
			];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn tmpfs_subdir_used_indirectly_via_bindmount_dir_by_one_pod() {
      let source = "/mnt/vol1/foo";
      let mount_infos = String::from(BASE) + "177 25 0:46 / /mnt/data rw,relatime shared:37 - tmpfs data rw
190 25 0:46 /vol1 /mnt/vol1 rw,relatime shared:37 - tmpfs data rw
191 25 0:46 /vol2 /mnt/vol2 rw,relatime shared:37 - tmpfs data rw
62 25 0:46 /vol1/foo /var/lib/kubelet/pods/e25f2f01-5b06-11e8-8694-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test rw,relatime shared:37 - tmpfs data rw
";
      let expected_refs = &["/var/lib/kubelet/pods/e25f2f01-5b06-11e8-8694-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test"];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn dir_bindmounted() {
      let source = "/mnt/disks/vol2";
      let mount_infos = String::from(BASE) +  "342 25 252:0 /mnt/disks/vol2 /mnt/disks/vol2 rw,relatime shared:1 - ext4 /dev/mapper/ubuntu--vg-root rw,errors=remount-ro,data=ordered
";
      let expected_refs = &[];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn dir_bindmounted_used_by_one_pod() {
      let source = "/mnt/disks/vol2";
      let mount_infos = String::from(BASE) +  "342 25 252:0 /mnt/disks/vol2 /mnt/disks/vol2 rw,relatime shared:1 - ext4 /dev/mapper/ubuntu--vg-root rw,errors=remount-ro,data=ordered
77 25 252:0 /mnt/disks/vol2 /var/lib/kubelet/pods/f30dc360-5a5d-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-1fb30a1c rw,relatime shared:1 - ext4 /dev/mapper/ubuntu--vg-root rw,errors=remount-ro,data=ordered
";
      let expected_refs = &["/var/lib/kubelet/pods/f30dc360-5a5d-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-1fb30a1c"];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn blockfs() {
      let source = "/mnt/disks/blkvol1";
      let mount_infos = String::from(BASE)
        + "58 25 7:1 / /mnt/disks/blkvol1 rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordered
";
      let expected_refs = &[];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn blockfs_used_by_one_pod() {
      let source = "/mnt/disks/blkvol1";
      let mount_infos = String::from(BASE) +  "58 25 7:1 / /mnt/disks/blkvol1 rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordered
62 25 7:1 / /var/lib/kubelet/pods/f19fe4e2-5a63-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordered
";
      let expected_refs = &["/var/lib/kubelet/pods/f19fe4e2-5a63-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test"];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }

    #[test]
    fn blockfs_used_by_two_pods() {
      let source = "/mnt/disks/blkvol1";
      let mount_infos = String::from(BASE) +  "58 25 7:1 / /mnt/disks/blkvol1 rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordered
62 25 7:1 / /var/lib/kubelet/pods/f19fe4e2-5a63-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordered
95 25 7:1 / /var/lib/kubelet/pods/4854a48b-5a64-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test rw,relatime shared:38 - ext4 /dev/loop1 rw,data=ordered
";
      let expected_refs = &["/var/lib/kubelet/pods/f19fe4e2-5a63-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test",
				"/var/lib/kubelet/pods/4854a48b-5a64-11e8-962f-000c29bb0377/volumes/kubernetes.io~local-volume/local-pv-test"];

      search_mount_points(source.as_ref(), &mount_infos, &*expected_refs)
    }
  }

  #[test_case("mySrc", "myTarget", "myFS", &["o1", "o2"], &["s1", "s2"] ; "options and sensitive")]
  #[test_case("mySrc", "myTarget", "myFS", &[], &["s1", "s2"] ; "sensitive only")]
  #[test_case("mySrc", "myTarget", "myFS", &["o1", "o2"], &[] ; "options only")]
  fn sensitive_mount_options(
    source: &'static str,
    target: &'static str,
    fstype: &'static str,
    options: &[&'static str],
    options_sensitive: &[&'static str],
  ) {
    let options = options.iter().map(|v| Arg::from(*v)).collect::<Vec<_>>();
    let options_sensitive = options_sensitive
      .iter()
      .map(|v| Arg::from(*v))
      .collect::<Vec<_>>();

    let (mount_args, log_str) = make_mount_args_sensitive(
      Some(source.into()),
      target.into(),
      fstype.into(),
      &options,
      &options_sensitive,
    );

    for option in &options {
      assert!(
        opts_contains(&mount_args, option),
        "expected option ({}) to exist in returned mount_args, but it does not: {:?}",
        option.to_string_lossy(),
        mount_args
      );
      assert!(
        log_str.contains(&*option.to_string_lossy()),
        "expected option ({}) to exist in returned log_str, but it does not: {:?}",
        option.to_string_lossy(),
        log_str
      );
    }

    for option in &options_sensitive {
      assert!(
        opts_contains(&mount_args, option),
        "expected sensitive option ({}) to exist in returned mount_args, but it does not: {:?}",
        option.to_string_lossy(),
        mount_args
      );
      assert!(
        !log_str.contains(&*option.to_string_lossy()),
        "expected sensitive option ({}) to not exist in returned log_str, but it does: {:?}",
        option.to_string_lossy(),
        log_str
      );
    }

    fn opts_contains(args: &[Arg], opt: &Arg) -> bool {
      if let Some(opt_arg) = args
        .iter()
        .skip_while(|a| *a != &Arg::<OsStr>::from("-o"))
        .nth(1)
      {
        opt_arg.to_string_lossy().contains(&*opt.to_string_lossy())
      } else {
        false
      }
    }
  }
}
