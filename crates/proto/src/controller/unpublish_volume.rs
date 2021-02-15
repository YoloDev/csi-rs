use super::Secrets;
use crate::proto;
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

#[derive(Debug)]
pub struct ControllerUnpublishVolumeRequest {
  volume_id: String,
  node_id: String,
  secrets: Secrets,
}

impl ControllerUnpublishVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// The ID of the node. This field is OPTIONAL. The CO SHOULD set this
  /// field to match the node ID returned by `NodeGetInfo` or leave it
  /// unset. If the value is set, the SP MUST unpublish the volume from
  /// the specified node. If the value is unset, the SP MUST unpublish
  /// the volume from all nodes it is published to.
  #[inline]
  pub fn node_id(&self) -> &str {
    &self.node_id
  }

  /// Secrets required by plugin to complete controller unpublish volume
  /// request. This SHOULD be the same secrets passed to the
  /// ControllerPublishVolume call for the specified volume.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }
}

impl TryFrom<proto::ControllerUnpublishVolumeRequest> for ControllerUnpublishVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ControllerUnpublishVolumeRequest) -> Result<Self, Self::Error> {
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

    let secrets = value.secrets.into();

    Ok(ControllerUnpublishVolumeRequest {
      volume_id,
      node_id,
      secrets,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ControllerUnpublishVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id`
  /// does not exist and is not assumed to be ControllerUnpublished from
  /// node corresponding to the specified `node_id`.
  #[error("Volume does not exist and volume not assumed ControllerUnpublished from node: {0}")]
  VolumeNotFound(String),

  /// Indicates that a node corresponding to the specified `node_id` does
  /// not exist and the volume corresponding to the specified `volume_id`
  /// is not assumed to be ControllerUnpublished from node.
  #[error("Node does not exist and volume not assumed ControllerUnpublished from node: {0}")]
  NodeNotFound(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<ControllerUnpublishVolumeError> for tonic::Status {
  fn from(value: ControllerUnpublishVolumeError) -> Self {
    match value {
      ControllerUnpublishVolumeError::Other(v) => v,
      value => {
        let code = match &value {
          ControllerUnpublishVolumeError::VolumeNotFound(_) => Code::NotFound,
          ControllerUnpublishVolumeError::NodeNotFound(_) => Code::NotFound,
          ControllerUnpublishVolumeError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
