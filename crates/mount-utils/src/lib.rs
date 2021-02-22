#![allow(clippy::too_many_arguments)]

cfg_if::cfg_if! {
  if #[cfg(unix)] {
    mod unix;
    use unix::*;
  } else {
    compile_error!("Only cfg(unix) is supported at this time")
  }
}

mod arguments;
mod fake;
mod runner;
mod utils;

pub use arguments::*;

use async_trait::async_trait;
use futures::future::BoxFuture;
use runner::{run, run_inst, MounterImpl, MounterWrapper};
use static_assertions::assert_impl_all;
use std::{fmt, io, path::Path, result, sync::Arc, time::Duration};
use thiserror::Error;
use tracing::Instrument;

pub type Result<T> = result::Result<T, MountError>;
pub type FutureResult<T> = BoxFuture<'static, Result<T>>;

const DEFAULT_MOUNT_COMMAND: &str = "mount";

/// Interface defines the set of methods to allow for mount operations on a system.
pub trait Mounter: Sized {
  /// Create a new mounter from a given mount path.
  fn new<P>(mount_path: P) -> FutureResult<Self>
  where
    P: Into<Arg<Path>>;

  /// Mounts source to target as fstype with given options.
  /// options MUST not contain sensitive material (like passwords).
  fn mount<I, P1, P2, A>(
    &self,
    source: Option<P1>,
    target: P2,
    fstype: A,
    options: I,
  ) -> FutureResult<()>
  where
    I: IntoIterator,
    <I as IntoIterator>::Item: Into<Arg>,
    P1: Into<Arg<Path>>,
    P2: Into<Arg<Path>>,
    A: Into<Arg>;

  /// mount_sensitive is the same as [mount] but this method allows
  /// sensitive_options to be passed in a separate parameter from the normal
  /// mount options and ensures the sensitiveOptions are never logged. This
  /// method should be used by callers that pass sensitive material (like
  /// passwords) as mount options.
  fn mount_sensitive<I1, I2, P1, P2, A>(
    &self,
    source: Option<P1>,
    target: P2,
    fstype: A,
    options: I1,
    sensitive_options: I2,
  ) -> FutureResult<()>
  where
    I1: IntoIterator,
    <I1 as IntoIterator>::Item: Into<Arg>,
    I2: IntoIterator,
    <I2 as IntoIterator>::Item: Into<Arg>,
    P1: Into<Arg<Path>>,
    P2: Into<Arg<Path>>,
    A: Into<Arg>;

  /// mount_sensitive_without_systemd is the same as [mount_sensitive] but this method disable using systemd mount.
  fn mount_sensitive_without_systemd<I1, I2, P1, P2, A>(
    &self,
    source: Option<P1>,
    target: P2,
    fstype: A,
    options: I1,
    sensitive_options: I2,
  ) -> FutureResult<()>
  where
    I1: IntoIterator,
    <I1 as IntoIterator>::Item: Into<Arg>,
    I2: IntoIterator,
    <I2 as IntoIterator>::Item: Into<Arg>,
    P1: Into<Arg<Path>>,
    P2: Into<Arg<Path>>,
    A: Into<Arg>;

  /// Unmount unmounts given target.
  fn unmount<P>(&self, target: P, force_after: Option<Duration>) -> FutureResult<()>
  where
    P: Into<Arg<Path>>;

  /// List returns a list of all mounted filesystems.  This can be large.
  /// On some platforms, reading mounts directly from the OS is not guaranteed
  /// consistent (i.e. it could change between chunked reads). This is guaranteed
  /// to be consistent.
  fn list(&self) -> FutureResult<Vec<MountPoint>>;

  /// IsLikelyNotMountPoint uses heuristics to determine if a directory
  /// is not a mountpoint. It should return ErrNotExist when the directory
  /// does not exist. IsLikelyNotMountPoint does NOT properly detect all
  /// mountpoint types most notably linux bind mounts and symbolic link.
  /// For callers that do not care about such situations, this is a faster
  /// alternative to calling List() and scanning that output.
  fn is_likely_not_mount_point<P>(&self, file: P) -> FutureResult<bool>
  where
    P: Into<Arg<Path>>;

  /// GetMountRefs finds all mount references to pathname, returning a slice of
  /// paths. Pathname can be a mountpoint path or a normal directory
  /// (for bind mount). On Linux, pathname is excluded from the slice.
  /// For example, if /dev/sdc was mounted at /path/a and /path/b,
  /// GetMountRefs("/path/a") would return ["/path/b"]
  /// GetMountRefs("/path/b") would return ["/path/a"]
  /// On Windows there is no way to query all mount points; as long as pathname is
  /// a valid mount, it will be returned.
  fn get_mount_refs<P>(&self, path: P) -> FutureResult<Vec<Arg<Path>>>
  where
    P: Into<Arg<Path>>;

  /// getMountRefsByDev finds all references to the device provided
  /// by mountPath; returns a list of paths.
  /// Note that mountPath should be path after the evaluation of any symblolic links.
  fn get_mount_refs_by_dev<P>(&self, mount_path: P) -> FutureResult<Vec<Arg<Path>>>
  where
    P: Into<Arg<Path>>;

  fn get_device_name_from_mount<P>(
    &self,
    mount_path: P,
  ) -> FutureResult<Option<(Arg<Path>, usize)>>
  where
    P: Into<Arg<Path>>;

  fn is_not_mount_point<P>(&self, file: P) -> FutureResult<bool>
  where
    P: Into<Arg<Path>>;
}

/// MountPoint represents a single line in /proc/mounts or /etc/fstab.
#[derive(Clone, PartialEq)]
pub struct MountPoint {
  device: Arg<Path>,
  path: Arg<Path>,
  ty: Arg,
  opts: Vec<Arg>,
  freq: isize,
  pass: isize,
}

impl fmt::Debug for MountPoint {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MountPoint")
      .field("device", &self.device)
      .field("path", &self.path)
      .field("type", &self.ty)
      .field("opts", &format!("length={}", self.opts.len()))
      .field("freq", &self.freq)
      .field("pass", &self.pass)
      .finish()
  }
}

impl MountPoint {
  #[inline]
  pub fn device(&self) -> &Arg<Path> {
    &self.device
  }

  #[inline]
  pub fn path(&self) -> &Arg<Path> {
    &self.path
  }

  #[inline]
  pub fn mount_type(&self) -> &Arg {
    &self.ty
  }

  #[inline]
  pub fn opts(&self) -> &[Arg] {
    &self.opts
  }

  #[inline]
  pub fn freq(&self) -> isize {
    self.freq
  }

  #[inline]
  pub fn pass(&self) -> isize {
    self.pass
  }

  pub fn matches(&self, dir: Arg<Path>) -> bool {
    // is_mount_point_match(&self, dir)
    todo!()
  }
}

#[derive(Debug, Error)]
pub enum MountError {
  #[error("File system mismatch: {0}")]
  FilesystemMismatch(String, #[source] io::Error),
  #[error("Has filesystem errors: {0}")]
  HasFilesystemErrors(String, #[source] io::Error),
  #[error("Unformatted read only: {0}")]
  UnformattedReadOnly(String, #[source] io::Error),
  #[error("Format failed: {0}")]
  FormatFailed(String, #[source] io::Error),
  #[error("Get disk format failed: {0}")]
  GetDiskFormatFailed(String, #[source] io::Error),
  #[error("Unknown mount error: {0}")]
  UnknownMountError(
    #[from]
    #[source]
    io::Error,
  ),
}

impl MountError {
  fn io(&self) -> &io::Error {
    match self {
      MountError::FilesystemMismatch(_, e) => e,
      MountError::HasFilesystemErrors(_, e) => e,
      MountError::UnformattedReadOnly(_, e) => e,
      MountError::FormatFailed(_, e) => e,
      MountError::GetDiskFormatFailed(_, e) => e,
      MountError::UnknownMountError(e) => e,
    }
  }

  fn is_permission_error(&self) -> bool {
    matches!(self.io().kind(), io::ErrorKind::PermissionDenied)
  }

  fn new(msg: impl Into<String>) -> Self {
    MountError::UnknownMountError(io::Error::new(io::ErrorKind::Other, msg.into()))
  }
}

#[async_trait]
impl<T> Mounter for T
where
  T: MounterWrapper,
{
  fn new<P>(mount_path: P) -> FutureResult<Self>
  where
    P: Into<Arg<Path>>,
  {
    let mount_path = mount_path.into();

    Box::pin(
      async move {
        run(move || <T as MounterWrapper>::Mounter::new(mount_path))
          .await
          .map(|inner| T::new(Arc::new(inner)))
      }
      .in_current_span(),
    )
  }

  fn mount<I, P1, P2, A>(
    &self,
    source: Option<P1>,
    target: P2,
    fstype: A,
    options: I,
  ) -> FutureResult<()>
  where
    I: IntoIterator,
    <I as IntoIterator>::Item: Into<Arg>,
    P1: Into<Arg<Path>>,
    P2: Into<Arg<Path>>,
    A: Into<Arg>,
  {
    let source = source.map(Into::into);
    let target = target.into();
    let fstype = fstype.into();
    let options = options.into_iter().map(Into::into).collect();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.mount(source, target, fstype, options)
    }))
  }

  fn mount_sensitive<I1, I2, P1, P2, A>(
    &self,
    source: Option<P1>,
    target: P2,
    fstype: A,
    options: I1,
    sensitive_options: I2,
  ) -> FutureResult<()>
  where
    I1: IntoIterator,
    <I1 as IntoIterator>::Item: Into<Arg>,
    I2: IntoIterator,
    <I2 as IntoIterator>::Item: Into<Arg>,
    P1: Into<Arg<Path>>,
    P2: Into<Arg<Path>>,
    A: Into<Arg>,
  {
    let source = source.map(Into::into);
    let target = target.into();
    let fstype = fstype.into();
    let options = options.into_iter().map(Into::into).collect();
    let options_sensitive = sensitive_options.into_iter().map(Into::into).collect();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.mount_sensitive(source, target, fstype, options, options_sensitive)
    }))
  }

  fn mount_sensitive_without_systemd<I1, I2, P1, P2, A>(
    &self,
    source: Option<P1>,
    target: P2,
    fstype: A,
    options: I1,
    sensitive_options: I2,
  ) -> FutureResult<()>
  where
    I1: IntoIterator,
    <I1 as IntoIterator>::Item: Into<Arg>,
    I2: IntoIterator,
    <I2 as IntoIterator>::Item: Into<Arg>,
    P1: Into<Arg<Path>>,
    P2: Into<Arg<Path>>,
    A: Into<Arg>,
  {
    let source = source.map(Into::into);
    let target = target.into();
    let fstype = fstype.into();
    let options = options.into_iter().map(Into::into).collect();
    let options_sensitive = sensitive_options.into_iter().map(Into::into).collect();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.mount_sensitive_without_systemd(source, target, fstype, options, options_sensitive)
    }))
  }

  fn unmount<P>(&self, target: P, force_after: Option<Duration>) -> FutureResult<()>
  where
    P: Into<Arg<Path>>,
  {
    let target = target.into();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.unmount(target, force_after)
    }))
  }

  fn list(&self) -> FutureResult<Vec<MountPoint>> {
    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.list()
    }))
  }

  fn is_likely_not_mount_point<P>(&self, file: P) -> FutureResult<bool>
  where
    P: Into<Arg<Path>>,
  {
    let file = file.into();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.is_likely_not_mount_point(file)
    }))
  }

  fn get_mount_refs<P>(&self, path: P) -> FutureResult<Vec<Arg<Path>>>
  where
    P: Into<Arg<Path>>,
  {
    let path = path.into();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.get_mount_refs(path)
    }))
  }

  fn get_mount_refs_by_dev<P>(&self, mount_path: P) -> FutureResult<Vec<Arg<Path>>>
  where
    P: Into<Arg<Path>>,
  {
    let mount_path = mount_path.into();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.get_mount_refs_by_dev(mount_path)
    }))
  }

  fn get_device_name_from_mount<P>(&self, mount_path: P) -> FutureResult<Option<(Arg<Path>, usize)>>
  where
    P: Into<Arg<Path>>,
  {
    let mount_path = mount_path.into();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.get_device_name_from_mount(mount_path)
    }))
  }

  fn is_not_mount_point<P>(&self, file: P) -> FutureResult<bool>
  where
    P: Into<Arg<Path>>,
  {
    let file = file.into();

    Box::pin(run_inst(self.mounter().clone(), move |mounter| {
      mounter.is_not_mount_point(file)
    }))
  }
}

assert_impl_all!(OsMounter: MounterImpl);
assert_impl_all!(fake::FakeMounter: MounterImpl);

pub struct DefaultMounter(Arc<OsMounter>);

impl MounterWrapper for DefaultMounter {
  type Mounter = OsMounter;

  fn new(inner: Arc<Self::Mounter>) -> Self {
    DefaultMounter(inner)
  }

  #[inline]
  fn mounter(&self) -> &Arc<Self::Mounter> {
    &self.0
  }
}

pub struct FakeMounter(Arc<fake::FakeMounter>);

impl MounterWrapper for FakeMounter {
  type Mounter = fake::FakeMounter;

  fn new(inner: Arc<Self::Mounter>) -> Self {
    FakeMounter(inner)
  }

  #[inline]
  fn mounter(&self) -> &Arc<Self::Mounter> {
    &self.0
  }
}
