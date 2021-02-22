use std::{
  borrow::Borrow,
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  sync::Arc,
};

pub(crate) enum CowArc<B: ?Sized + 'static> {
  BorrowedStatic(&'static B),
  OwnedArc(Arc<B>),
}

impl<B: ?Sized + 'static> Clone for CowArc<B> {
  fn clone(&self) -> Self {
    match self {
      Self::BorrowedStatic(v) => Self::BorrowedStatic(*v),
      Self::OwnedArc(v) => Self::OwnedArc(v.clone()),
    }
  }

  fn clone_from(&mut self, source: &Self) {
    match (self, source) {
      (&mut Self::OwnedArc(ref mut dest), &Self::OwnedArc(ref o)) => dest.clone_from(&*o),
      (t, s) => *t = s.clone(),
    }
  }
}

impl<B: ?Sized + 'static> Borrow<B> for CowArc<B> {
  fn borrow(&self) -> &B {
    &**self
  }
}

impl<B: ?Sized + 'static> CowArc<B> {
  pub const fn is_borrowed(&self) -> bool {
    matches!(self, CowArc::BorrowedStatic(_))
  }

  pub const fn is_owned(&self) -> bool {
    matches!(self, CowArc::OwnedArc(_))
  }

  pub fn into_owned(self) -> <B as ToOwned>::Owned
  where
    B: ToOwned,
  {
    (*self).to_owned()
  }
}

impl<B: ?Sized + 'static> Deref for CowArc<B> {
  type Target = B;

  fn deref(&self) -> &Self::Target {
    match self {
      CowArc::BorrowedStatic(borrowed) => *borrowed,
      CowArc::OwnedArc(arc) => &**arc,
    }
  }
}

impl<B: ?Sized + 'static> Eq for CowArc<B> where B: Eq {}

impl<B: ?Sized + 'static> Ord for CowArc<B>
where
  B: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    Ord::cmp(&**self, &**other)
  }
}

impl<'a, B: ?Sized + 'static, C: ?Sized + 'static> PartialEq<CowArc<C>> for CowArc<B>
where
  B: PartialEq<C>,
{
  #[inline]
  fn eq(&self, other: &CowArc<C>) -> bool {
    PartialEq::eq(&**self, &**other)
  }
}

impl<'a, B: ?Sized + 'static, C: ?Sized + 'static> PartialOrd<CowArc<C>> for CowArc<B>
where
  B: PartialOrd<C>,
{
  #[inline]
  fn partial_cmp(&self, other: &CowArc<C>) -> Option<std::cmp::Ordering> {
    PartialOrd::partial_cmp(&**self, &**other)
  }
}

impl<B: ?Sized + 'static> fmt::Debug for CowArc<B>
where
  B: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <B as fmt::Debug>::fmt(&**self, f)
  }
}

impl<B: ?Sized + 'static> fmt::Display for CowArc<B>
where
  B: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <B as fmt::Display>::fmt(&**self, f)
  }
}

impl<B: ?Sized + 'static> Default for CowArc<B>
where
  B: ToOwned,
  <B as ToOwned>::Owned: Default + Into<Arc<B>>,
{
  fn default() -> Self {
    Self::OwnedArc(<<B as ToOwned>::Owned>::default().into())
  }
}

impl<B: ?Sized + 'static> Hash for CowArc<B>
where
  B: Hash,
{
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    Hash::hash(&**self, state)
  }
}

impl<B: ?Sized + 'static> AsRef<B> for CowArc<B> {
  fn as_ref(&self) -> &B {
    self
  }
}
