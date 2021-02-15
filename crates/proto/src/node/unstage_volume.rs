use crate::proto;

use std::{
  convert::TryFrom,
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct NodeUnstageVolumeRequest {
  volume_id: String,
  staging_target_path: PathBuf,
}

impl NodeUnstageVolumeRequest {
  /// The ID of the volume to publish. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// The path at which the volume was staged. It MUST be an absolute
  /// path in the root filesystem of the process serving this request.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn staging_target_path(&self) -> &Path {
    &self.staging_target_path
  }
}

impl TryFrom<proto::NodeUnstageVolumeRequest> for NodeUnstageVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::NodeUnstageVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeStageVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let staging_target_path = match value.staging_target_path {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeStageVolumeRequest.staging_target_path is empty",
        ))
      }
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => {
          return Err(tonic::Status::invalid_argument(
            "NodeStageVolumeRequest.staging_target_path is not absolute",
          ))
        }
        v => v,
      },
    };

    Ok(NodeUnstageVolumeRequest {
      volume_id,
      staging_target_path,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NodeUnstageVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<NodeUnstageVolumeError> for tonic::Status {
  fn from(value: NodeUnstageVolumeError) -> Self {
    match value {
      NodeUnstageVolumeError::Other(v) => v,
      value => {
        let code = match &value {
          NodeUnstageVolumeError::VolumeNotFound(_) => Code::NotFound,
          NodeUnstageVolumeError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
