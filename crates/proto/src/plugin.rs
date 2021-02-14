use crate::{proto, IdentityService, VolumeExpansionSupport};
use tracing::debug;

pub(crate) fn get_capabilities(s: &impl IdentityService) -> proto::GetPluginCapabilitiesResponse {
  let mut response = proto::GetPluginCapabilitiesResponse::default();

  let volume_accessibility_constraints_support = s.volume_accessibility_constraints_support();
  if volume_accessibility_constraints_support {
    response.capabilities.push(proto::PluginCapability {
      r#type: Some(proto::plugin_capability::Type::Service(
        proto::plugin_capability::Service {
          r#type: proto::plugin_capability::service::Type::VolumeAccessibilityConstraints.into(),
        },
      )),
    });
  }

  let volume_expansion_support = s.volume_expansion_support();
  match volume_expansion_support {
    VolumeExpansionSupport::None => (),
    VolumeExpansionSupport::Offline => {
      response.capabilities.push(proto::PluginCapability {
        r#type: Some(proto::plugin_capability::Type::VolumeExpansion(
          proto::plugin_capability::VolumeExpansion {
            r#type: proto::plugin_capability::volume_expansion::Type::Offline.into(),
          },
        )),
      });
    }
    VolumeExpansionSupport::Online => {
      response.capabilities.push(proto::PluginCapability {
        r#type: Some(proto::plugin_capability::Type::VolumeExpansion(
          proto::plugin_capability::VolumeExpansion {
            r#type: proto::plugin_capability::volume_expansion::Type::Online.into(),
          },
        )),
      });
    }
  }

  debug!(
    ?volume_accessibility_constraints_support,
    ?volume_expansion_support
  );
  response
}
