use crate::{runner::MounterImpl, Arg, MountError, MountPoint, Result};
use std::{collections::HashMap, path::Path, sync::Mutex};
use tracing::info;

#[derive(Clone)]
pub enum FakeAction {
  Mount {
    target: Arg<Path>,
    source: Arg<Path>,
    fs_type: Arg,
  },

  Unmount {
    target: Arg<Path>,
  },
}

type UnmountFunc = Box<dyn Fn(Arg<Path>) -> Option<MountError> + Send>;
type ErrorFactory = Box<dyn Fn() -> MountError + Send>;

struct FakeMounterInner {
  mount_points: Vec<MountPoint>,
  log: Vec<FakeAction>,
  mount_check_errors: HashMap<Arg<Path>, ErrorFactory>,
  unmount_func: Option<UnmountFunc>,
}

pub struct FakeMounter(Mutex<FakeMounterInner>);

impl FakeMounter {
  pub fn new(mps: impl IntoIterator<Item = MountPoint>) -> Self {
    let mount_points = mps.into_iter().collect();
    let inner = FakeMounterInner {
      mount_points,
      log: Vec::new(),
      mount_check_errors: HashMap::new(),
      unmount_func: None,
    };

    Self(Mutex::new(inner))
  }

  pub fn reset_log(&self) {
    self.0.lock().unwrap().log.clear();
  }

  pub fn get_log(&self) -> Vec<FakeAction> {
    self.0.lock().unwrap().log.clone()
  }
}

impl MounterImpl for FakeMounter {
  fn new(_: Arg<Path>) -> Result<Self> {
    Ok(FakeMounter::new(None))
  }

  fn mount_sensitive(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: smallvec::SmallVec<[Arg; 4]>,
    sensitive_options: smallvec::SmallVec<[Arg; 4]>,
  ) -> Result<()> {
    let mut source =
      source.ok_or_else(|| MountError::new("missing required source in fake mounter"))?;
    let mut inner = self.0.lock().unwrap();
    let mut opts = Vec::new();
    for opt in options {
      // find 'bind' option
      if opt.to_str() == Some("bind") {
        // This is a bind-mount. In order to mimic linux behaviour, we must
        // use the original device of the bind-mount as the real source.
        // E.g. when mounted /dev/sda like this:
        //      $ mount /dev/sda /mnt/test
        //      $ mount -o bind /mnt/test /mnt/bound
        // then /proc/mount contains:
        // /dev/sda /mnt/test
        // /dev/sda /mnt/bound
        // (and not /mnt/test /mnt/bound)
        // I.e. we must use /dev/sda as source instead of /mnt/test in the
        // bind mount.
        for mnt in &inner.mount_points {
          if source == mnt.path {
            source = mnt.path.clone();
            break;
          }
        }
      }

      opts.push(opt);
    }

    // If target is a symlink, get its absolute path
    let target = target
      .canonicalize()
      .map(Arg::from)
      .unwrap_or_else(|_| target);

    opts.extend(sensitive_options);
    inner.mount_points.push(MountPoint {
      device: source.clone(),
      path: target.clone(),
      ty: fstype.clone(),
      opts,
      freq: 0,
      pass: 0,
    });
    info!(
      "Fake mounter: mounted {} to {}",
      source.display(),
      target.display()
    );
    inner.log.push(FakeAction::Mount {
      target,
      source,
      fs_type: fstype,
    });

    Ok(())
  }

  fn mount_sensitive_without_systemd(
    &self,
    source: Option<Arg<Path>>,
    target: Arg<Path>,
    fstype: Arg,
    options: smallvec::SmallVec<[Arg; 4]>,
    sensitive_options: smallvec::SmallVec<[Arg; 4]>,
  ) -> Result<()> {
    self.mount_sensitive(source, target, fstype, options, sensitive_options)
  }

  fn unmount(&self, target: Arg<Path>, _: Option<std::time::Duration>) -> Result<()> {
    let mut inner = self.0.lock().unwrap();

    // If target is a symlink, get its absolute path
    let orig_target = target.clone();
    let target = target
      .canonicalize()
      .map(Arg::from)
      .unwrap_or_else(|_| target);

    if let Some((i, mp)) = inner
      .mount_points
      .iter()
      .enumerate()
      .find(|(_, mp)| mp.path == target)
    {
      match inner.unmount_func.as_ref().and_then(|f| f(target.clone())) {
        None => (),
        Some(e) => return Err(e),
      };

      info!(
        "Fake mounter: unmounted {} from {}",
        mp.device.display(),
        target.display()
      );

      inner.mount_points.remove(i);
    }

    inner.log.push(FakeAction::Unmount { target });
    inner.mount_check_errors.remove(&orig_target);
    Ok(())
  }

  fn list(&self) -> Result<Vec<MountPoint>> {
    Ok(self.0.lock().unwrap().mount_points.clone())
  }

  fn is_likely_not_mount_point(&self, file: Arg<Path>) -> Result<bool> {
    let inner = self.0.lock().unwrap();

    if let Some(err_factory) = inner.mount_check_errors.get(&file) {
      return Err(err_factory());
    }

    let _ = file.metadata()?;

    // If file is a symlink, get its absolute path
    let file = file.canonicalize().map(Arg::from).unwrap_or_else(|_| file);

    for mp in &inner.mount_points {
      if mp.path == file {
        info!(
          "isLikelyNotMountPoint for {}: mounted {}, false",
          file.display(),
          mp.path.display()
        );
        return Ok(false);
      }
    }

    info!("isLikelyNotMountPoint for {}: true", file.display());
    Ok(true)
  }

  fn get_mount_refs(&self, path: Arg<Path>) -> Result<Vec<Arg<Path>>> {
    let realpath = path.canonicalize().map(Arg::from).unwrap_or_else(
      |_| /* Ignore error in FakeMounter, because we actually didn't create files. */ path,
    );

    self.get_mount_refs_by_dev(realpath)
  }
}
