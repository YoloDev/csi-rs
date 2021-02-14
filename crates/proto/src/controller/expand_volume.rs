use super::{secrets::Secrets, CapacityRange, VolumeCapability};
use crate::proto;
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
  num::NonZeroU64,
};
use thiserror::Error;

#[derive(Debug)]
pub struct ControllerExpandVolumeRequest {
  volume_id: String,
  capacity_range: CapacityRange,
  secrets: Secrets,
  volume_capability: Option<VolumeCapability>,
}

impl ControllerExpandVolumeRequest {
  /// The ID of the volume to expand. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// This allows CO to specify the capacity requirements of the volume
  /// after expansion. This field is REQUIRED.
  #[inline]
  pub fn capacity_range(&self) -> &CapacityRange {
    &self.capacity_range
  }

  /// Secrets required by the plugin for expanding the volume.
  /// This field is OPTIONAL.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }

  /// Volume capability describing how the CO intends to use this volume.
  /// This allows SP to determine if volume is being used as a block
  /// device or mounted file system. For example - if volume is
  /// being used as a block device - the SP MAY set
  /// node_expansion_required to false in ControllerExpandVolumeResponse
  /// to skip invocation of NodeExpandVolume on the node by the CO.
  /// This is an OPTIONAL field.
  #[inline]
  pub fn volume_capability(&self) -> Option<&VolumeCapability> {
    self.volume_capability.as_ref()
  }
}

impl TryFrom<proto::ControllerExpandVolumeRequest> for ControllerExpandVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ControllerExpandVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "ControllerExpandVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let capacity_range = match value.capacity_range {
      None => {
        return Err(tonic::Status::invalid_argument(
          "ControllerExpandVolumeRequest.capacity_range missing",
        ))
      }
      Some(v) => v.try_into()?,
    };

    let secrets = value.secrets.into();
    let volume_capability = value.volume_capability.map(TryInto::try_into).transpose()?;

    Ok(ControllerExpandVolumeRequest {
      volume_id,
      capacity_range,
      secrets,
      volume_capability,
    })
  }
}

#[derive(Debug)]
pub struct ControllerExpandVolumeResponse {
  /// Capacity of volume after expansion.
  capacity_bytes: NonZeroU64,
  /// Whether node expansion is required for the volume. When true
  /// the CO MUST make NodeExpandVolume RPC call on the node.
  node_expansion_required: bool,
}

impl TryFrom<ControllerExpandVolumeResponse> for proto::ControllerExpandVolumeResponse {
  type Error = tonic::Status;

  fn try_from(value: ControllerExpandVolumeResponse) -> Result<Self, Self::Error> {
    let capacity_bytes = value.capacity_bytes.get() as i64;
    let node_expansion_required = value.node_expansion_required;

    Ok(proto::ControllerExpandVolumeResponse {
      capacity_bytes,
      node_expansion_required,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ControllerExpandVolumeError {
  /// Indicates that CO has specified capabilities not supported by the volume.
  #[error("Exceeds capabilities: {0}")]
  ExceedsCapabilities(String),

  /// Indicates that a volume corresponding to the specified volume_id does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  /// Indicates that the volume corresponding to the specified `volume_id` could not
  /// be expanded because it is currently published on a node but the plugin does not
  /// have ONLINE expansion capability.
  #[error("Volume in use: {0}")]
  VolumeInUse(String),

  /// Indicates that the capacity range is not allowed by the Plugin. More human-readable
  /// information MAY be provided in the gRPC `status.message` field.
  #[error("Unsupported 'capacity_range': {0}")]
  UnsupportedCapacityRange(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<ControllerExpandVolumeError> for tonic::Status {
  fn from(value: ControllerExpandVolumeError) -> Self {
    use tonic::{Code, Status};

    match value {
      ControllerExpandVolumeError::ExceedsCapabilities(v) => Status::new(Code::InvalidArgument, v),
      ControllerExpandVolumeError::VolumeNotFound(v) => Status::new(Code::NotFound, v),
      ControllerExpandVolumeError::VolumeInUse(v) => Status::new(Code::FailedPrecondition, v),
      ControllerExpandVolumeError::UnsupportedCapacityRange(v) => Status::new(Code::OutOfRange, v),
      ControllerExpandVolumeError::Other(v) => v,
    }
  }
}
