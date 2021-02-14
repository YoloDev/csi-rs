use super::{Secrets, VolumeCapability};
use crate::proto;
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
};
use thiserror::Error;

#[derive(Debug)]
pub struct ValidateVolumeCapabilitiesRequest {
  volume_id: String,
  volume_context: HashMap<String, String>,
  volume_capabilities: Vec<VolumeCapability>,
  parameters: HashMap<String, String>,
  secrets: Secrets,
}

impl ValidateVolumeCapabilitiesRequest {
  /// The ID of the volume to check. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  #[inline]
  pub fn volume_context(&self) -> &HashMap<String, String> {
    &self.volume_context
  }

  /// The capabilities that the CO wants to check for the volume. This
  /// call SHALL return "confirmed" only if all the volume capabilities
  /// specified below are supported. This field is REQUIRED.
  #[inline]
  pub fn volume_capabilities(&self) -> &[VolumeCapability] {
    &self.volume_capabilities
  }

  /// See CreateVolumeRequest.parameters.
  /// This field is OPTIONAL.
  #[inline]
  pub fn parameters(&self) -> &HashMap<String, String> {
    &self.parameters
  }

  /// Secrets required by plugin to complete volume validation request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }
}

impl TryFrom<proto::ValidateVolumeCapabilitiesRequest> for ValidateVolumeCapabilitiesRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ValidateVolumeCapabilitiesRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "ControllerPublishVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let volume_context = value.volume_context;
    let volume_capabilities = match value.volume_capabilities {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "Missing ValidateVolumeCapabilitiesRequest.volume_capabilities",
        ))
      }
      v => v
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()?,
    };
    let parameters = value.parameters;
    let secrets = value.secrets.into();

    Ok(ValidateVolumeCapabilitiesRequest {
      volume_id,
      volume_context,
      volume_capabilities,
      parameters,
      secrets,
    })
  }
}

#[derive(Debug)]
pub struct Confirmed {
  /// Volume context validated by the plugin.
  volume_context: Option<HashMap<String, String>>,
  /// Volume capabilities supported by the plugin.
  volume_capabilities: Vec<VolumeCapability>,
  /// The volume creation parameters validated by the plugin.
  parameters: Option<HashMap<String, String>>,
}

impl TryFrom<Confirmed> for proto::validate_volume_capabilities_response::Confirmed {
  type Error = tonic::Status;

  fn try_from(value: Confirmed) -> Result<Self, Self::Error> {
    let volume_context = value.volume_context.unwrap_or_default();
    let volume_capabilities = value
      .volume_capabilities
      .into_iter()
      .map(TryInto::try_into)
      .collect::<Result<_, _>>()?;
    let parameters = value.parameters.unwrap_or_default();

    Ok(proto::validate_volume_capabilities_response::Confirmed {
      volume_context,
      volume_capabilities,
      parameters,
    })
  }
}

#[derive(Debug)]
pub enum ValidateVolumeCapabilitiesResponse {
  Confirmed(Confirmed),
  Message(String),
}

impl TryFrom<ValidateVolumeCapabilitiesResponse> for proto::ValidateVolumeCapabilitiesResponse {
  type Error = tonic::Status;

  fn try_from(value: ValidateVolumeCapabilitiesResponse) -> Result<Self, Self::Error> {
    Ok(match value {
      ValidateVolumeCapabilitiesResponse::Confirmed(confirmed) => {
        proto::ValidateVolumeCapabilitiesResponse {
          confirmed: Some(confirmed.try_into()?),
          message: Default::default(),
        }
      }

      ValidateVolumeCapabilitiesResponse::Message(message) => {
        proto::ValidateVolumeCapabilitiesResponse {
          confirmed: None,
          message,
        }
      }
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValidateVolumeCapabilitiesError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist and volume not assumed ControllerUnpublished from node: {0}")]
  VolumeNotFound(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<ValidateVolumeCapabilitiesError> for tonic::Status {
  fn from(value: ValidateVolumeCapabilitiesError) -> Self {
    use tonic::{Code, Status};

    match value {
      ValidateVolumeCapabilitiesError::VolumeNotFound(v) => Status::new(Code::NotFound, v),
      ValidateVolumeCapabilitiesError::Other(v) => v,
    }
  }
}
