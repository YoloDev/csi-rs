macro_rules! unsupported {
  ($name:expr) => {{
    ::tracing::error!("Unsupported method {} called", $name);
    return Err(
      ::tonic::Status::new(
        ::tonic::Code::Unimplemented,
        format!("Unsupported method {} called", $name),
      )
      .into(),
    );
  }};
}

pub mod controller;
pub mod node;
pub mod volume;

mod plugin;
mod proto;
mod secrets;
mod utils;

use std::collections::HashMap;

use lazy_static::lazy_static;

pub use controller::ControllerService;
pub use node::NodeService;

#[derive(Eq, Clone, Copy, PartialEq, Debug, Hash)]
pub enum VolumeExpansionSupport {
  None,
  Offline,
  Online,
}

pub trait IdentityService: Send + Sync + 'static {
  /// The name MUST follow domain name notation format
  /// (<https://tools.ietf.org/html/rfc1035#section-2.3.1>). It SHOULD
  /// include the plugin's host company name and the plugin name,
  /// to minimize the possibility of collisions. It MUST be 63
  /// characters or less, beginning and ending with an alphanumeric
  /// character ([a-z0-9A-Z]) with dashes (-), dots (.), and
  /// alphanumerics between.
  fn name(&self) -> &str;

  /// Plugin version. Value of this field is opaque to the CO.
  fn version(&self) -> &str;

  /// Whether or not this plugin supports volume accessibility constraints.
  #[inline]
  fn volume_accessibility_constraints_support(&self) -> bool {
    false
  }

  /// Whether or not this plugin supports volume accessibility constraints.
  #[inline]
  fn volume_expansion_support(&self) -> VolumeExpansionSupport {
    VolumeExpansionSupport::None
  }

  #[inline]
  fn ready(&self) -> bool {
    true
  }

  #[inline]
  fn manifest(&self) -> &HashMap<String, String> {
    lazy_static! {
      static ref EMPTY_MANIFEST: HashMap<String, String> = HashMap::new();
    }

    &EMPTY_MANIFEST
  }
}
