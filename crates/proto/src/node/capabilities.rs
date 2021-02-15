use std::convert::TryFrom;

use bitflags::bitflags;

use crate::proto;

#[rustfmt::skip]
bitflags! {
  pub struct NodeCapabilities: u32 {
    const STAGE_UNSTAGE_VOLUME         = 0b_0000_0000_0000_0001;
    const GET_VOLUME_STATS             = 0b_0000_0000_0000_0010;
    const EXPAND_VOLUME                = 0b_0000_0000_0000_0100;
    const VOLUME_CONDITION             = 0b_0000_0000_0000_1000;
  }
}

use proto::node_service_capability::rpc::Type;
impl TryFrom<NodeCapabilities> for proto::NodeGetCapabilitiesResponse {
  type Error = tonic::Status;

  fn try_from(value: NodeCapabilities) -> Result<Self, Self::Error> {
    #[inline]
    fn push_cap(
      vec: &mut Vec<proto::NodeServiceCapability>,
      value: NodeCapabilities,
      test: NodeCapabilities,
      proto: Type,
    ) {
      if value.contains(test) {
        vec.push(proto::NodeServiceCapability {
          r#type: Some(proto::node_service_capability::Type::Rpc(
            proto::node_service_capability::Rpc {
              r#type: proto as i32,
            },
          )),
        })
      }
    }

    let mut capabilities = Vec::with_capacity(4 /* number of different capabilities */);
    push_cap(
      &mut capabilities,
      value,
      NodeCapabilities::STAGE_UNSTAGE_VOLUME,
      Type::StageUnstageVolume,
    );
    push_cap(
      &mut capabilities,
      value,
      NodeCapabilities::GET_VOLUME_STATS,
      Type::GetVolumeStats,
    );
    push_cap(
      &mut capabilities,
      value,
      NodeCapabilities::EXPAND_VOLUME,
      Type::ExpandVolume,
    );
    push_cap(
      &mut capabilities,
      value,
      NodeCapabilities::VOLUME_CONDITION,
      Type::VolumeCondition,
    );

    Ok(proto::NodeGetCapabilitiesResponse { capabilities })
  }
}
