use crate::proto;
use std::{
  convert::TryFrom,
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct NodeUnpublishVolumeRequest {
  volume_id: String,
  target_path: PathBuf,
}

impl NodeUnpublishVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// The path at which the volume was published. It MUST be an absolute
  /// path in the root filesystem of the process serving this request.
  /// The SP MUST delete the file or directory it created at this path.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn target_path(&self) -> &Path {
    &self.target_path
  }
}

impl TryFrom<proto::NodeUnpublishVolumeRequest> for NodeUnpublishVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::NodeUnpublishVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeUnpublishVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let target_path = match value.target_path {
      v if v.is_empty() => return Err(tonic::Status::invalid_argument(
        "NodeUnpublishVolumeRequest.target_path is empty",
      )),
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => return Err(tonic::Status::invalid_argument(
          "NodeUnpublishVolumeRequest.target_path is not absolute",
        )),
        v if !v.parent().map(|p| p.is_dir()).unwrap_or(false) => return Err(tonic::Status::invalid_argument(
          "NodeUnpublishVolumeRequest.staging_target_path is does not have a parent, or it's parent is not a directory",
        )),
        v => v,
      },
    };

    Ok(NodeUnpublishVolumeRequest {
      volume_id,
      target_path,
    })
  }
}

#[derive(Debug, Error)]
pub enum NodeUnpublishVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<NodeUnpublishVolumeError> for tonic::Status {
  fn from(value: NodeUnpublishVolumeError) -> Self {
    match value {
      NodeUnpublishVolumeError::Other(v) => v,
      value => {
        let code = match &value {
          NodeUnpublishVolumeError::VolumeNotFound(_) => Code::NotFound,
          NodeUnpublishVolumeError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
