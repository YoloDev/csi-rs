use super::Secrets;
use crate::proto;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug)]
pub struct DeleteVolumeRequest {
  volume_id: String,
  secrets: Secrets,
}

impl TryFrom<proto::DeleteVolumeRequest> for DeleteVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::DeleteVolumeRequest) -> Result<Self, Self::Error> {
    if value.volume_id.is_empty() {
      return Err(tonic::Status::invalid_argument(
        "DeleteVolumeRequest.volume_id is empty",
      ));
    }

    Ok(DeleteVolumeRequest {
      volume_id: value.volume_id,
      secrets: value.secrets.into(),
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeleteVolumeError {
  /// Indicates that the volume corresponding to the specified `volume_id` could not be
  /// deleted because it is in use by another resource or has snapshots and the plugin
  /// doesn't treat them as independent entities.
  #[error("Volume in use: {0}")]
  VolumeInUse(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<DeleteVolumeError> for tonic::Status {
  fn from(value: DeleteVolumeError) -> tonic::Status {
    use tonic::{Code, Status};

    match value {
      DeleteVolumeError::VolumeInUse(v) => Status::new(Code::FailedPrecondition, v),
      DeleteVolumeError::Other(v) => v,
    }
  }
}
