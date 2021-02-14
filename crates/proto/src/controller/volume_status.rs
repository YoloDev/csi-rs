use crate::proto;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct VolumeCondition {
  /// Normal volumes are available for use and operating optimally.
  /// An abnormal volume does not meet these criteria.
  abnormal: bool,
  /// The message describing the condition of the volume.
  /// This field is REQUIRED.
  message: String,
}

impl TryFrom<VolumeCondition> for proto::VolumeCondition {
  type Error = tonic::Status;

  fn try_from(value: VolumeCondition) -> Result<Self, Self::Error> {
    let abnormal = value.abnormal;
    let message = value.message;

    Ok(proto::VolumeCondition { abnormal, message })
  }
}

#[derive(Debug)]
pub struct VolumeStatus {
  /// A list of all `node_id` of nodes that the volume in this entry
  /// is controller published on.
  /// This field is OPTIONAL. If it is not specified and the SP has
  /// the LIST_VOLUMES_PUBLISHED_NODES controller capability, the CO
  /// MAY assume the volume is not controller published to any nodes.
  /// If the field is not specified and the SP does not have the
  /// LIST_VOLUMES_PUBLISHED_NODES controller capability, the CO MUST
  /// not interpret this field.
  /// published_node_ids MAY include nodes not published to or
  /// reported by the SP. The CO MUST be resilient to that.
  published_node_ids: Vec<String>,

  /// Information about the current condition of the volume.
  /// This field is OPTIONAL.
  /// This field MUST be specified if the
  /// VOLUME_CONDITION controller capability is supported.
  volume_condition: Option<VolumeCondition>,
}

impl TryFrom<VolumeStatus> for proto::list_volumes_response::VolumeStatus {
  type Error = tonic::Status;

  fn try_from(value: VolumeStatus) -> Result<Self, Self::Error> {
    let published_node_ids = value.published_node_ids;
    let volume_condition = value.volume_condition.map(TryInto::try_into).transpose()?;

    Ok(proto::list_volumes_response::VolumeStatus {
      published_node_ids,
      volume_condition,
    })
  }
}

impl TryFrom<VolumeStatus> for proto::controller_get_volume_response::VolumeStatus {
  type Error = tonic::Status;

  fn try_from(value: VolumeStatus) -> Result<Self, Self::Error> {
    let published_node_ids = value.published_node_ids;
    let volume_condition = value.volume_condition.map(TryInto::try_into).transpose()?;

    Ok(proto::controller_get_volume_response::VolumeStatus {
      published_node_ids,
      volume_condition,
    })
  }
}
