use super::{Volume, VolumeStatus};
use crate::proto;
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

#[derive(Debug)]
pub struct ControllerGetVolumeRequest {
  volume_id: String,
}

impl ControllerGetVolumeRequest {
  /// The ID of the volume to fetch current volume information for.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }
}

impl TryFrom<proto::ControllerGetVolumeRequest> for ControllerGetVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ControllerGetVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "ControllerGetVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    Ok(ControllerGetVolumeRequest { volume_id })
  }
}

#[derive(Debug)]
pub struct ControllerGetVolumeResponse {
  volume: Volume,
  status: VolumeStatus,
}

impl TryFrom<ControllerGetVolumeResponse> for proto::ControllerGetVolumeResponse {
  type Error = tonic::Status;

  fn try_from(value: ControllerGetVolumeResponse) -> Result<Self, Self::Error> {
    let volume = Some(value.volume.try_into()?);
    let status = Some(value.status.try_into()?);

    Ok(proto::ControllerGetVolumeResponse { volume, status })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ControllerGetVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<ControllerGetVolumeError> for tonic::Status {
  fn from(value: ControllerGetVolumeError) -> Self {
    use tonic::{Code, Status};

    match value {
      ControllerGetVolumeError::VolumeNotFound(v) => Status::new(Code::NotFound, v),
      ControllerGetVolumeError::Other(v) => v,
    }
  }
}
