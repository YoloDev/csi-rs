use crate::{Arg, MountError, MountPoint, Result};
use futures::channel::oneshot::{channel as oneshot, Receiver, Sender};
use once_cell::sync::OnceCell;
use smallvec::SmallVec;
use std::{
  fs,
  future::Future,
  panic::{catch_unwind, RefUnwindSafe, UnwindSafe},
  path::Path,
  sync::Arc,
  time::Duration,
};
use tracing::error;
use tracing::Span;

struct MounterMessage {
  span: Span,
  run: Box<dyn FnOnce(Span) + Send + UnwindSafe + RefUnwindSafe>,
}

impl MounterMessage {
  fn new(span: Span, f: impl FnOnce(Span) + Send + UnwindSafe + RefUnwindSafe + 'static) -> Self {
    Self {
      span,
      run: Box::new(f),
    }
  }
}

// Span isn't unwindsafe due to dyn content - but it should be unwind safe in practice (missing some bounds)
impl UnwindSafe for MounterMessage {}

type MounterDispatcher = crossbeam::channel::Sender<MounterMessage>;

// Note: All these methods are to be run in a separate thread, ensuring
// that only one runs at once, and that it does not block async processing.
pub trait MounterImpl: Sized + Send + Sync + UnwindSafe + RefUnwindSafe + 'static {
  /// Create a new mounter from a given mount path.
  fn new(mount_path: Arg<Path>) -> Result<Self>;

  /// Mounts source to target as fstype with given options.
  /// options MUST not contain sensitive material (like passwords).
  fn mount(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: SmallVec<[Arg; 4]>,
  ) -> Result<()> {
    self.mount_sensitive(source, target, fstype, options, SmallVec::new())
  }

  /// mount_sensitive is the same as [mount] but this method allows
  /// sensitive_options to be passed in a separate parameter from the normal
  /// mount options and ensures the sensitiveOptions are never logged. This
  /// method should be used by callers that pass sensitive material (like
  /// passwords) as mount options.
  fn mount_sensitive(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: SmallVec<[Arg; 4]>,
    sensitive_options: SmallVec<[Arg; 4]>,
  ) -> Result<()>;

  /// mount_sensitive_without_systemd is the same as [mount_sensitive] but this method disable using systemd mount.
  fn mount_sensitive_without_systemd(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: SmallVec<[Arg; 4]>,
    sensitive_options: SmallVec<[Arg; 4]>,
  ) -> Result<()>;

  /// Unmount unmounts given target. If a force_after is provided, will retry
  /// after that amount of time with force option.
  fn unmount(&self, target: Arg<Path>, force_after: Option<Duration>) -> Result<()>;

  /// List returns a list of all mounted filesystems.  This can be large.
  /// On some platforms, reading mounts directly from the OS is not guaranteed
  /// consistent (i.e. it could change between chunked reads). This is guaranteed
  /// to be consistent.
  fn list(&self) -> Result<Vec<MountPoint>>;

  /// IsLikelyNotMountPoint uses heuristics to determine if a directory
  /// is not a mountpoint. It should return ErrNotExist when the directory
  /// does not exist. IsLikelyNotMountPoint does NOT properly detect all
  /// mountpoint types most notably linux bind mounts and symbolic link.
  /// For callers that do not care about such situations, this is a faster
  /// alternative to calling List() and scanning that output.
  fn is_likely_not_mount_point(&self, file: Arg<Path>) -> Result<bool>;

  /// GetMountRefs finds all mount references to pathname, returning a slice of
  /// paths. Pathname can be a mountpoint path or a normal directory
  /// (for bind mount). On Linux, pathname is excluded from the slice.
  /// For example, if /dev/sdc was mounted at /path/a and /path/b,
  /// GetMountRefs("/path/a") would return ["/path/b"]
  /// GetMountRefs("/path/b") would return ["/path/a"]
  /// On Windows there is no way to query all mount points; as long as pathname is
  /// a valid mount, it will be returned.
  fn get_mount_refs(&self, path: Arg<Path>) -> Result<Vec<Arg<Path>>>;

  /// getMountRefsByDev finds all references to the device provided
  /// by mountPath; returns a list of paths.
  /// Note that mountPath should be path after the evaluation of any symblolic links.
  fn get_mount_refs_by_dev(&self, mount_path: Arg<Path>) -> Result<Vec<Arg<Path>>> {
    let mps = self.list()?;
    let disk_dev = mps
      .iter()
      .find(|m| *m.path() == mount_path)
      .map(|m| m.device().clone());

    let refs = mps
      .into_iter()
      .filter_map(|m| {
        if (Some(m.device()) == disk_dev.as_ref() || *m.device() == mount_path)
          && *m.path() != mount_path
        {
          Some(m.path)
        } else {
          None
        }
      })
      .collect();

    Ok(refs)
  }

  fn get_device_name_from_mount(
    &self,
    mount_path: Arg<Path>,
  ) -> Result<Option<(Arg<Path>, usize)>> {
    let mps = self.list()?;

    // If mountPath is symlink, need get its target path.
    let mount_path = fs::canonicalize(&mount_path)
      .map(Arg::from)
      .unwrap_or(mount_path);

    // Find the device name.
    // FIXME if multiple devices mounted on the same mount path, only the first one is returned.
    Ok(
      mps
        .iter()
        .find(|m| *m.path() == mount_path)
        .map(|m| m.device().clone())
        .map(|device| {
          let ref_count = mps.into_iter().filter(|m| *m.device() == device).count();
          (device, ref_count)
        }),
    )
  }

  fn is_not_mount_point(&self, file: Arg<Path>) -> Result<bool> {
    // IsLikelyNotMountPoint provides a quick check
    // to determine whether file IS A mountpoint.
    let not_mnt = match self.is_likely_not_mount_point(file.clone()) {
      Ok(v) => v,
      Err(e) if e.is_permission_error() => {
        // We were not allowed to do the simple stat() check, e.g. on NFS with
        // root_squash. Fall back to /proc/mounts check below.
        true
      }
      Err(e) => return Err(e),
    };

    // identified as mountpoint, so return this fact.
    if not_mnt {
      return Ok(not_mnt);
    }

    // Resolve any symlinks in file, kernel would do the same and use the resolved path in /proc/mounts.
    let resolved_file = match fs::canonicalize(file) {
      Ok(v) => Arg::from(v),
      Err(_) => return Ok(true),
    };

    // check all mountpoints since IsLikelyNotMountPoint
    // is not reliable for some mountpoint types.
    Ok(
      self
        .list()?
        .iter()
        .any(|mp| mp.matches(<Arg<Path> as Clone>::clone(&resolved_file))),
    )
  }
}

static DISPATCHER: OnceCell<MounterDispatcher> = OnceCell::new();

fn dispatcher() -> Result<&'static MounterDispatcher> {
  DISPATCHER
    .get_or_try_init(|| {
      let (sender, receiver) = crossbeam::channel::unbounded::<MounterMessage>();

      std::thread::Builder::new()
        .name("mount-utils:dispatch".into())
        .spawn(move || {
          let mut msg_count = 0usize;

          while let Ok(msg) = receiver.recv() {
            msg_count = msg_count + 1;
            if let Err(e) = catch_unwind(move || {
              let MounterMessage { span, run } = msg;
              run(span)
            }) {
              error!("Failed to run mount function in dispatcher: {:?}", e);
            }
          }
        })
        .map(|_| sender)
    })
    .map_err(|e| MountError::new(format!("failed to spawn dispatcher: {:?}", e)))
}

pub(crate) fn run<R: 'static, F: 'static>(f: F) -> impl Future<Output = Result<R>>
where
  F: FnOnce() -> Result<R> + Send + UnwindSafe + RefUnwindSafe,
  R: UnwindSafe + RefUnwindSafe + Send,
{
  struct OuterMsg<R, F>
  where
    F: FnOnce() -> Result<R> + Send + UnwindSafe + RefUnwindSafe,
    R: UnwindSafe + RefUnwindSafe,
  {
    run: F,
    sender: Sender<Result<R>>,
  }

  impl<R, F> UnwindSafe for OuterMsg<R, F>
  where
    F: FnOnce() -> Result<R> + Send + UnwindSafe + RefUnwindSafe,
    R: UnwindSafe + RefUnwindSafe,
  {
  }

  impl<R, F> RefUnwindSafe for OuterMsg<R, F>
  where
    F: FnOnce() -> Result<R> + Send + UnwindSafe + RefUnwindSafe,
    R: UnwindSafe + RefUnwindSafe,
  {
  }

  let span = Span::current();
  let (sender, receiver) = oneshot();

  match dispatcher() {
    Ok(dispatch) => {
      let msg = OuterMsg { run: f, sender };

      dispatch
        .send(MounterMessage::new(span, move |span| {
          let OuterMsg { run, sender } = msg;
          let result = {
            let _enter = span.enter();
            run()
          };

          let _ = sender.send(result);
        }))
        .unwrap()
    }
    Err(e) => {
      let _ = sender.send(Err(e));
    }
  };

  read(receiver)
}

pub(crate) fn run_inst<T: MounterImpl, R: 'static, F: 'static>(
  mounter: Arc<T>,
  f: F,
) -> impl Future<Output = Result<R>>
where
  F: FnOnce(&T) -> Result<R> + Send + UnwindSafe + RefUnwindSafe,
  R: UnwindSafe + RefUnwindSafe + Send,
{
  run(move || f(&*mounter))
}

async fn read<R>(receiver: Receiver<Result<R>>) -> Result<R> {
  match receiver.await {
    Ok(r) => r,
    Err(cancelled) => Err(MountError::new("request was cancelled (thread paniced?)")),
  }
}

// MakeBindOpts detects whether a bind mount is being requested and makes the remount options to
// use in case of bind mount, due to the fact that bind mount doesn't respect mount options.
// The list equals:
//   options - 'bind' + 'remount' (no duplicate)
pub(crate) fn make_bind_opts(opts: &[Arg]) -> (bool, Vec<Arg>, Vec<Arg>) {
  let (bind, bind_opts, bind_remount_opts, _) = make_bind_opts_sensitive(opts, &[]);
  (bind, bind_opts, bind_remount_opts)
}

// MakeBindOptsSensitive is the same as MakeBindOpts but this method allows
// sensitiveOptions to be passed in a separate parameter from the normal mount
// options and ensures the sensitiveOptions are never logged. This method should
// be used by callers that pass sensitive material (like passwords) as mount
// options.
pub(crate) fn make_bind_opts_sensitive(
  opts: &[Arg],
  opts_sensitive: &[Arg],
) -> (bool, Vec<Arg>, Vec<Arg>, Vec<Arg>) {
  // Because we have an FD opened on the subpath bind mount, the "bind" option
  // needs to be included, otherwise the mount target will error as busy if you
  // remount as readonly.
  //
  // As a consequence, all read only bind mounts will no longer change the underlying
  // volume mount to be read only.
  let mut bind_remount_opts: Vec<Arg> = vec!["bind".into(), "remount".into()];
  let mut bind_remount_sensitive_opts: Vec<Arg> = Vec::new();
  let mut bind = false;
  let mut bind_opts = vec!["bind".into()];

  for opt in opts {
    match opt.to_str() {
      Some("bind") => {
        bind = true;
      }
      Some("_netdev") => {
        // _netdev is a userspace mount option and does not automatically get added when
        // bind mount is created and hence we must carry it over.
        bind_opts.push("_netdev".into());
        bind_remount_opts.push("_netdev".into());
      }
      _ => bind_remount_opts.push(opt.clone()),
    }
  }

  for opt in opts_sensitive {
    match opt.to_str() {
      Some("bind") => {
        bind = true;
      }
      Some("_netdev") => {
        // _netdev is a userspace mount option and does not automatically get added when
        // bind mount is created and hence we must carry it over.
        bind_opts.push("_netdev".into());
        bind_remount_opts.push("_netdev".into());
      }
      _ => bind_remount_sensitive_opts.push(opt.clone()),
    }
  }

  (
    bind,
    bind_opts,
    bind_remount_opts,
    bind_remount_sensitive_opts,
  )
}

pub trait MounterWrapper {
  type Mounter: MounterImpl;

  fn new(inner: Arc<Self::Mounter>) -> Self;

  fn mounter(&self) -> &Arc<Self::Mounter>;
}

#[cfg(test)]
mod tests {
  use std::ffi::OsStr;

  use super::*;
  use test_case::test_case;

  #[test_case(&["vers=2", "ro", "_netdev"], false, &[], &[])]
  #[test_case(&["bind", "vers=2", "ro", "_netdev"], true, &["bind", "_netdev"], &["bind", "remount", "vers=2", "ro", "_netdev"])]
  fn make_bind_opts(
    mount_option: &[&'static str],
    is_bind: bool,
    expected_bind_opts: &[&'static str],
    expected_remount_opts: &[&'static str],
  ) {
    let expected_bind_opts = expected_bind_opts
      .iter()
      .map(|s| Arg::<OsStr>::from(*s))
      .collect::<Vec<_>>();
    let expected_remount_opts = expected_remount_opts
      .iter()
      .map(|s| Arg::<OsStr>::from(*s))
      .collect::<Vec<_>>();

    let (bind, bind_opts, bind_remount_opts) = super::make_bind_opts(
      &*mount_option
        .iter()
        .map(|s| Arg::from(*s))
        .collect::<Vec<_>>(),
    );

    assert_eq!(bind, is_bind);
    if is_bind {
      assert_eq!(expected_bind_opts, bind_opts);
      assert_eq!(expected_remount_opts, bind_remount_opts);
    }
  }
}
