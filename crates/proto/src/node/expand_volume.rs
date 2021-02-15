use super::{CapacityRange, VolumeCapability};
use crate::proto;
use std::{
  convert::{TryFrom, TryInto},
  num::NonZeroU64,
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct NodeExpandVolumeRequest {
  volume_id: String,
  volume_path: PathBuf,
  capacity_range: Option<CapacityRange>,
  staging_target_path: Option<PathBuf>,
  volume_capability: Option<VolumeCapability>,
}

impl NodeExpandVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// The path on which volume is available. This field is REQUIRED.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn volume_path(&self) -> &Path {
    &self.volume_path
  }

  /// This allows CO to specify the capacity requirements of the volume
  /// after expansion. If capacity_range is omitted then a plugin MAY
  /// inspect the file system of the volume to determine the maximum
  /// capacity to which the volume can be expanded. In such cases a
  /// plugin MAY expand the volume to its maximum capacity.
  /// This field is OPTIONAL.
  #[inline]
  pub fn capacity_range(&self) -> Option<CapacityRange> {
    self.capacity_range
  }

  /// The path where the volume is staged, if the plugin has the
  /// STAGE_UNSTAGE_VOLUME capability, otherwise empty.
  /// If not empty, it MUST be an absolute path in the root
  /// filesystem of the process serving this request.
  /// This field is OPTIONAL.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn staging_target_path(&self) -> Option<&Path> {
    self.staging_target_path.as_deref()
  }

  /// Volume capability describing how the CO intends to use this volume.
  /// This allows SP to determine if volume is being used as a block
  /// device or mounted file system. For example - if volume is being
  /// used as a block device the SP MAY choose to skip expanding the
  /// filesystem in NodeExpandVolume implementation but still perform
  /// rest of the housekeeping needed for expanding the volume. If
  /// volume_capability is omitted the SP MAY determine
  /// access_type from given volume_path for the volume and perform
  /// node expansion. This is an OPTIONAL field.
  #[inline]
  pub fn volume_capability(&self) -> Option<&VolumeCapability> {
    self.volume_capability.as_ref()
  }
}

impl TryFrom<proto::NodeExpandVolumeRequest> for NodeExpandVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::NodeExpandVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeExpandVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let volume_path = match value.volume_path {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeExpandVolumeRequest.volume_path is empty",
        ))
      }
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => {
          return Err(tonic::Status::invalid_argument(
            "NodeExpandVolumeRequest.volume_path is not absolute",
          ))
        }
        v => v,
      },
    };

    let staging_target_path = match value.staging_target_path {
      v if v.is_empty() => None,
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => {
          return Err(tonic::Status::invalid_argument(
            "NodeExpandVolumeRequest.staging_target_path is not absolute",
          ))
        }
        v => Some(v),
      },
    };

    let capacity_range = value.capacity_range.map(TryInto::try_into).transpose()?;
    let volume_capability = value.volume_capability.map(TryInto::try_into).transpose()?;

    Ok(NodeExpandVolumeRequest {
      volume_id,
      volume_path,
      capacity_range,
      staging_target_path,
      volume_capability,
    })
  }
}

#[derive(Debug)]
pub struct NodeExpandVolumeResponse {
  /// The capacity of the volume in bytes. This field is OPTIONAL.
  capacity_bytes: Option<NonZeroU64>,
}

impl TryFrom<NodeExpandVolumeResponse> for proto::NodeExpandVolumeResponse {
  type Error = tonic::Status;

  fn try_from(value: NodeExpandVolumeResponse) -> Result<Self, Self::Error> {
    let capacity_bytes = value
      .capacity_bytes
      .map(|v| v.get() as i64)
      .unwrap_or_default();

    Ok(proto::NodeExpandVolumeResponse { capacity_bytes })
  }
}

#[derive(Debug, Error)]
pub enum NodeExpandVolumeError {
  /// Indicates that CO has specified capabilities not supported by the volume.
  #[error("Exceeds capabilities: {0}")]
  ExceedsCapabilities(String),

  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  /// Indicates that the volume corresponding to the specified `volume_id` could not be
  /// expanded because it is node-published or node-staged and the underlying filesystem
  /// does not support expansion of published or staged volumes.
  #[error("Volume in use: {0}")]
  VolumeInUse(String),

  /// Indicates that the capacity range is not allowed by the Plugin. More human-readable
  /// information MAY be provided in the gRPC `status.message` field.
  #[error("Unsupported capacity_range: {0}")]
  UnsupportedCapacityRange(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<NodeExpandVolumeError> for tonic::Status {
  fn from(value: NodeExpandVolumeError) -> Self {
    match value {
      NodeExpandVolumeError::Other(v) => v,
      value => {
        let code = match &value {
          NodeExpandVolumeError::ExceedsCapabilities(_) => Code::InvalidArgument,
          NodeExpandVolumeError::VolumeNotFound(_) => Code::NotFound,
          NodeExpandVolumeError::VolumeInUse(_) => Code::FailedPrecondition,
          NodeExpandVolumeError::UnsupportedCapacityRange(_) => Code::OutOfRange,
          NodeExpandVolumeError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
