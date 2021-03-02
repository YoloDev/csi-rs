use crate::Arg;
use std::{
  ffi::{OsStr, OsString},
  path::{Component, Path, PathBuf},
};
use thiserror::Error;

pub(crate) fn join<'a>(iter: impl IntoIterator<Item = &'a Arg>, separator: impl Into<Arg>) -> Arg {
  let mut iter = iter.into_iter();
  let first = match iter.next() {
    None => return Arg::from(""),
    Some(v) => v,
  };

  match iter.next() {
    None => (*first).clone(),
    Some(second) => do_join(&**first, &**second, &*separator.into(), iter),
  }
}

fn do_join<'a>(
  first: &OsStr,
  second: &OsStr,
  sep: &OsStr,
  iter: impl Iterator<Item = &'a Arg>,
) -> Arg {
  let mut ret = OsString::with_capacity(first.len() + second.len() + sep.len());
  ret.push(first);
  ret.push(sep);
  ret.push(second);

  for next in iter {
    ret.push(sep);
    ret.push(next);
  }

  ret.into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[error("Failed to find matching prefix")]
pub struct StripPrefixError;

pub(crate) trait PathExt {
  fn strip_resolved_prefix(&self, p: impl AsRef<Path>) -> Result<PathBuf, StripPrefixError>;
}

impl PathExt for Path {
  fn strip_resolved_prefix(&self, p: impl AsRef<Path>) -> Result<PathBuf, StripPrefixError> {
    let p = p.as_ref();

    iter_after(
      fixed_path_comp(self).components(),
      fixed_path_comp(p).components(),
    )
    .map(|c| c.as_path().to_owned())
    .ok_or(StripPrefixError)
  }
}

fn iter_after<'a, 'b, I, J>(mut iter: I, mut prefix: J) -> Option<I>
where
  I: Iterator<Item = Component<'a>> + Clone,
  J: Iterator<Item = Component<'b>>,
{
  loop {
    let mut iter_next = iter.clone();
    match (iter_next.next(), prefix.next()) {
      (Some(ref x), Some(ref y)) if x == y => (),
      (Some(_), Some(_)) => return None,
      (Some(_), None) => return Some(iter),
      (None, None) => return Some(iter),
      (None, Some(_)) => return None,
    }
    iter = iter_next;
  }
}

fn fixed_path_comp(p: &Path) -> PathBuf {
  let mut skip = 0usize;
  p.components()
    .rev()
    .filter(move |c| {
      if skip > 0 {
        skip -= 1;
        false
      } else {
        match c {
          Component::Prefix(_) => true,
          Component::RootDir => true,
          Component::CurDir => true,
          Component::ParentDir => {
            skip += 1;
            false
          }
          Component::Normal(_) => true,
        }
      }
    })
    .collect::<Vec<_>>() // ensure side effects happens on the rev
    .into_iter()
    .rev()
    .collect::<PathBuf>()
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;

  #[test_case("/a/b/c", "/a" => Some(String::from("b/c")) ; "good subpath")]
  #[test_case("/a/b/c", "/a/b" => Some(String::from("c")) ; "good subpath 2")]
  #[test_case("/a/b/c/", "/a/b" => Some(String::from("c")) ; "good subpath end slash")]
  #[test_case("/a/b/../c", "/a" => Some(String::from("c")) ; "good subpath backticks")]
  #[test_case("/a/b/c", "/a/b/c" => Some(String::from("")) ; "good subpath equal")]
  #[test_case("/a/b/c/", "/a/b/c" => Some(String::from("")) ; "good subpath equal 2")]
  #[test_case("/a", "/" => Some(String::from("a")) ; "good subpath root")]
  #[test_case("/a/b/c", "/a/b/c/d" => None ; "bad subpath parent")]
  #[test_case("/b/c", "/a/b/c" => None ; "bad subpath outside")]
  #[test_case("/a/b/cd", "/a/b/c" => None ; "bad subpath prefix")]
  #[test_case("/a/../b", "/a" => None ; "bad subpath backticks")]
  #[test_case(
    "/var/lib/kubelet/pods/uuid/volumes/kubernetes.io~configmap/config/..timestamp/file.txt",
    "/var/lib/kubelet/pods/uuid/volumes/kubernetes.io~configmap/config" => Some(String::from("..timestamp/file.txt"))
    ; "configmap subpath")]
  fn strip_resolved_prefix(full_path: &str, base_path: &str) -> Option<String> {
    let full_path: &Path = full_path.as_ref();
    let base_path: &Path = base_path.as_ref();

    full_path
      .strip_resolved_prefix(base_path)
      .ok()
      .map(|p| (*p.to_string_lossy()).to_owned())
  }
}
