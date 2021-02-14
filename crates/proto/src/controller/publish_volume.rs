use super::{Secrets, VolumeCapability};
use crate::proto;
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
};
use thiserror::Error;

#[derive(Debug)]
pub struct ControllerPublishVolumeRequest {
  volume_id: String,
  node_id: String,
  volume_capability: VolumeCapability,
  readonly: bool,
  secrets: Secrets,
  volume_context: HashMap<String, String>,
}

impl ControllerPublishVolumeRequest {
  /// The ID of the volume to be used on a node.
  /// This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// The ID of the node. This field is REQUIRED. The CO SHALL set this
  /// field to match the node ID returned by `NodeGetInfo`.
  #[inline]
  pub fn node_id(&self) -> &str {
    &self.node_id
  }

  /// Volume capability describing how the CO intends to use this volume.
  /// SP MUST ensure the CO can use the published volume as described.
  /// Otherwise SP MUST return the appropriate gRPC error code.
  #[inline]
  pub fn volume_capability(&self) -> &VolumeCapability {
    &self.volume_capability
  }

  // Indicates SP MUST publish the volume in readonly mode.
  /// CO MUST set this field to false if SP does not have the
  /// PUBLISH_READONLY controller capability.
  #[inline]
  pub fn readonly(&self) -> bool {
    self.readonly
  }

  /// Secrets required by plugin to complete controller publish volume
  /// request. This field is OPTIONAL. Refer to the
  /// `Secrets Requirements` section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }

  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  pub fn volume_context(&self) -> &HashMap<String, String> {
    &self.volume_context
  }
}

impl TryFrom<proto::ControllerPublishVolumeRequest> for ControllerPublishVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ControllerPublishVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "ControllerPublishVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let node_id = match value.node_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "ControllerPublishVolumeRequest.node_id is empty",
        ))
      }
      v => v,
    };

    let volume_capability = match value.volume_capability {
      None => {
        return Err(tonic::Status::invalid_argument(
          "ControllerPublishVolumeRequest.volume_capability missing",
        ))
      }
      Some(v) => v.try_into()?,
    };

    let readonly = value.readonly;
    let secrets = value.secrets.into();
    let volume_context = value.volume_context;

    Ok(ControllerPublishVolumeRequest {
      volume_id,
      node_id,
      volume_capability,
      readonly,
      secrets,
      volume_context,
    })
  }
}

#[derive(Debug)]
pub struct ControllerPublishVolumeResponse {
  /// Opaque static publish properties of the volume. SP MAY use this
  /// field to ensure subsequent `NodeStageVolume` or `NodePublishVolume`
  /// calls calls have contextual information.
  /// The contents of this field SHALL be opaque to a CO.
  /// The contents of this field SHALL NOT be mutable.
  /// The contents of this field SHALL be safe for the CO to cache.
  /// The contents of this field SHOULD NOT contain sensitive
  /// information.
  /// The contents of this field SHOULD NOT be used for uniquely
  /// identifying a volume. The `volume_id` alone SHOULD be sufficient to
  /// identify the volume.
  /// This field is OPTIONAL and when present MUST be passed to
  /// subsequent `NodeStageVolume` or `NodePublishVolume` calls
  publish_context: HashMap<String, String>,
}

impl TryFrom<ControllerPublishVolumeResponse> for proto::ControllerPublishVolumeResponse {
  type Error = tonic::Status;

  fn try_from(value: ControllerPublishVolumeResponse) -> Result<Self, Self::Error> {
    let publish_context = value.publish_context;

    Ok(proto::ControllerPublishVolumeResponse { publish_context })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ControllerPublishVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeDoesNotExist(String),

  /// Indicates that a node corresponding to the specified `node_id` does not exist.
  #[error("Node does not exist: {0}")]
  NodeDoesNotExist(String),

  /// Indicates that a volume corresponding to the specified `volume_id` has already been
  /// published at the node corresponding to the specified `node_id` but is incompatible
  /// with the specified `volume_capability` or `readonly` flag.
  #[error("Volume published but is incompatible: {0}")]
  AlreadyExists(String),

  /// Indicates that a volume corresponding to the specified `volume_id` has already been
  /// published at another node and does not have MULTI_NODE volume capability. If this
  /// error code is returned, the Plugin SHOULD specify the `node_id` of the node at which
  /// the volume is published as part of the gRPC `status.message`.
  #[error("Volume published to another node: {0}")]
  PublishedToAnotherNode(String),

  /// Indicates that the maximum supported number of volumes that can be attached to the
  /// specified node are already attached. Therefore, this operation will fail until at
  /// least one of the existing attached volumes is detached from the node.
  #[error("Max volumes attached: {0}")]
  MaxVolumesAttached(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<ControllerPublishVolumeError> for tonic::Status {
  fn from(value: ControllerPublishVolumeError) -> tonic::Status {
    use tonic::{Code, Status};

    match value {
      ControllerPublishVolumeError::VolumeDoesNotExist(v) => Status::new(Code::NotFound, v),
      ControllerPublishVolumeError::NodeDoesNotExist(v) => Status::new(Code::NotFound, v),
      ControllerPublishVolumeError::AlreadyExists(v) => Status::new(Code::AlreadyExists, v),
      ControllerPublishVolumeError::PublishedToAnotherNode(v) => {
        Status::new(Code::FailedPrecondition, v)
      }
      ControllerPublishVolumeError::MaxVolumesAttached(v) => {
        Status::new(Code::ResourceExhausted, v)
      }
      ControllerPublishVolumeError::Other(v) => v,
    }
  }
}
