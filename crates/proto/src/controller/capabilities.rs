use std::convert::TryFrom;

use bitflags::bitflags;

use crate::proto;

#[rustfmt::skip]
bitflags! {
  pub struct ControllerCapabilities: u32 {
    const CREATE_DELETE_VOLUME         = 0b_0000_0000_0000_0001;
    const PUBLISH_UNPUBLISH_VOLUME     = 0b_0000_0000_0000_0010;
    const LIST_VOLUMES                 = 0b_0000_0000_0000_0100;
    const GET_CAPACITY                 = 0b_0000_0000_0000_1000;

    /// Currently the only way to consume a snapshot is to create
    /// a volume from it. Therefore plugins supporting
    /// `CREATE_DELETE_SNAPSHOT` MUST support creating volume from
    /// snapshot.
    const CREATE_DELETE_SNAPSHOT       = 0b_0000_0000_0001_0000;
    const LIST_SNAPSHOTS               = 0b_0000_0000_0010_0000;

    /// Plugins supporting volume cloning at the storage level MAY
    /// report this capability. The source volume MUST be managed by
    /// the same plugin. Not all volume sources and parameters
    /// combinations MAY work.
    const CLONE_VOLUME                 = 0b_0000_0000_0100_0000;

    /// Indicates the SP supports ControllerPublishVolume.readonly
    /// field.
    const PUBLISH_READONLY             = 0b_0000_0000_1000_0000;

    /// See VolumeExpansion for details.
    const EXPAND_VOLUME                = 0b_0000_0001_0000_0000;

    /// Indicates the SP supports the
    /// ListVolumesResponse.entry.published_nodes field
    const LIST_VOLUMES_PUBLISHED_NODES = 0b_0000_0010_0000_0000;

    /// Indicates that the Controller service can report volume
    /// conditions.
    /// An SP MAY implement `VolumeCondition` in only the Controller
    /// Plugin, only the Node Plugin, or both.
    /// If `VolumeCondition` is implemented in both the Controller and
    /// Node Plugins, it SHALL report from different perspectives.
    /// If for some reason Controller and Node Plugins report
    /// misaligned volume conditions, CO SHALL assume the worst case
    /// is the truth.
    /// Note that, for alpha, `VolumeCondition` is intended be
    /// informative for humans only, not for automation.
    const VOLUME_CONDITION             = 0b_0000_0100_0000_0000;

    /// Indicates the SP supports the ControllerGetVolume RPC.
    /// This enables COs to, for example, fetch per volume
    /// condition after a volume is provisioned.
    const GET_VOLUME                   = 0b_0000_1000_0000_0000;
  }
}

use proto::controller_service_capability::rpc::Type;
impl TryFrom<ControllerCapabilities> for proto::ControllerGetCapabilitiesResponse {
  type Error = tonic::Status;

  fn try_from(value: ControllerCapabilities) -> Result<Self, Self::Error> {
    #[inline]
    fn push_cap(
      vec: &mut Vec<proto::ControllerServiceCapability>,
      value: ControllerCapabilities,
      test: ControllerCapabilities,
      proto: Type,
    ) {
      if value.contains(test) {
        vec.push(proto::ControllerServiceCapability {
          r#type: Some(proto::controller_service_capability::Type::Rpc(
            proto::controller_service_capability::Rpc {
              r#type: proto as i32,
            },
          )),
        })
      }
    }

    let mut capabilities = Vec::with_capacity(12 /* number of different capabilities */);
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::CREATE_DELETE_VOLUME,
      Type::CreateDeleteVolume,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::PUBLISH_UNPUBLISH_VOLUME,
      Type::PublishUnpublishVolume,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::LIST_VOLUMES,
      Type::ListVolumes,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::GET_CAPACITY,
      Type::GetCapacity,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::CREATE_DELETE_SNAPSHOT,
      Type::CreateDeleteSnapshot,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::LIST_SNAPSHOTS,
      Type::ListSnapshots,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::CLONE_VOLUME,
      Type::CloneVolume,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::PUBLISH_READONLY,
      Type::PublishReadonly,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::EXPAND_VOLUME,
      Type::ExpandVolume,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::LIST_VOLUMES_PUBLISHED_NODES,
      Type::ListVolumesPublishedNodes,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::VOLUME_CONDITION,
      Type::VolumeCondition,
    );
    push_cap(
      &mut capabilities,
      value,
      ControllerCapabilities::GET_VOLUME,
      Type::GetVolume,
    );

    Ok(proto::ControllerGetCapabilitiesResponse { capabilities })
  }
}
