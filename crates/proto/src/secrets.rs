use std::{collections::HashMap, fmt};

pub(crate) struct Secrets(HashMap<String, String>);

impl AsRef<HashMap<String, String>> for Secrets {
  #[inline]
  fn as_ref(&self) -> &HashMap<String, String> {
    &self.0
  }
}

impl From<HashMap<String, String>> for Secrets {
  #[inline]
  fn from(v: HashMap<String, String>) -> Self {
    Secrets(v)
  }
}

impl From<Secrets> for HashMap<String, String> {
  #[inline]
  fn from(v: Secrets) -> Self {
    v.0
  }
}

impl fmt::Debug for Secrets {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut m = f.debug_map();
    for k in self.0.keys() {
      m.key(k).value(&"SECRET");
    }

    m.finish()
  }
}
