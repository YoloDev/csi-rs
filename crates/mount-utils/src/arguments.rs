mod cowarc;

use cowarc::CowArc;
use std::{
  ffi::{OsStr, OsString},
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  path::{Path, PathBuf},
};

pub struct Arg<T: ?Sized + 'static = OsStr>(CowArc<T>);

impl<T: ?Sized + 'static> Clone for Arg<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }

  fn clone_from(&mut self, source: &Self) {
    self.0.clone_from(&source.0)
  }
}

impl<T: ?Sized + fmt::Debug + 'static> fmt::Debug for Arg<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <T as fmt::Debug>::fmt(&*self.0, f)
  }
}

impl<T: ?Sized + fmt::Display + 'static> fmt::Display for Arg<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <T as fmt::Display>::fmt(&*self.0, f)
  }
}

impl<T: ?Sized + 'static> AsRef<T> for Arg<T> {
  #[inline]
  fn as_ref(&self) -> &T {
    &self.0
  }
}

impl<T: ?Sized + 'static> Deref for Arg<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<&'static OsStr> for Arg {
  fn from(value: &'static OsStr) -> Self {
    Arg(CowArc::BorrowedStatic(value))
  }
}

impl From<OsString> for Arg {
  fn from(value: OsString) -> Self {
    Arg(CowArc::OwnedArc(value.into()))
  }
}

impl From<&'static str> for Arg {
  fn from(value: &'static str) -> Self {
    Arg(CowArc::BorrowedStatic(value.as_ref()))
  }
}

impl From<String> for Arg {
  fn from(value: String) -> Self {
    Arg(CowArc::OwnedArc(OsString::from(value).into()))
  }
}

impl From<&'static Path> for Arg {
  fn from(value: &'static Path) -> Self {
    Arg(CowArc::BorrowedStatic(value.as_ref()))
  }
}

impl From<PathBuf> for Arg {
  fn from(value: PathBuf) -> Self {
    Arg(CowArc::OwnedArc(OsString::from(value).into()))
  }
}

impl From<&'static OsStr> for Arg<Path> {
  fn from(value: &'static OsStr) -> Self {
    Arg(CowArc::BorrowedStatic(value.as_ref()))
  }
}

impl From<OsString> for Arg<Path> {
  fn from(value: OsString) -> Self {
    Arg(CowArc::OwnedArc(PathBuf::from(value).into()))
  }
}

impl From<&'static str> for Arg<Path> {
  fn from(value: &'static str) -> Self {
    Arg(CowArc::BorrowedStatic(value.as_ref()))
  }
}

impl From<String> for Arg<Path> {
  fn from(value: String) -> Self {
    Arg(CowArc::OwnedArc(PathBuf::from(value).into()))
  }
}

impl From<&'static Path> for Arg<Path> {
  fn from(value: &'static Path) -> Self {
    Arg(CowArc::BorrowedStatic(value))
  }
}

impl From<PathBuf> for Arg<Path> {
  fn from(value: PathBuf) -> Self {
    Arg(CowArc::OwnedArc(value.into()))
  }
}

impl From<Arg> for OsString {
  fn from(value: Arg) -> Self {
    value.0.into_owned()
  }
}

impl From<Arg<Path>> for OsString {
  fn from(value: Arg<Path>) -> Self {
    OsString::from(&*value.0)
  }
}

impl From<Arg<Path>> for Arg {
  fn from(value: Arg<Path>) -> Self {
    match value.0 {
      CowArc::BorrowedStatic(r) => Arg::from(r),
      CowArc::OwnedArc(v) => Arg::from(PathBuf::from(&*v)),
    }
  }
}

impl duct::IntoExecutablePath for Arg<Path> {
  fn to_executable(self) -> OsString {
    self.as_os_str().to_owned()
  }
}

impl<B: ?Sized + 'static> Eq for Arg<B> where B: Eq {}

impl<B: ?Sized + 'static, C: ?Sized + 'static> PartialEq<Arg<C>> for Arg<B>
where
  B: PartialEq<C>,
{
  #[inline]
  fn eq(&self, other: &Arg<C>) -> bool {
    PartialEq::eq(&**self, &**other)
  }
}

impl<B: ?Sized + 'static> Hash for Arg<B>
where
  B: Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    Hash::hash(&**self, state)
  }
}
