use crate::proto;

use super::{Secrets, VolumeCapability};
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct NodeStageVolumeRequest {
  volume_id: String,
  publish_context: HashMap<String, String>,
  staging_target_path: PathBuf,
  volume_capability: VolumeCapability,
  secrets: Secrets,
  volume_context: HashMap<String, String>,
}

impl NodeStageVolumeRequest {
  /// The ID of the volume to publish. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// The CO SHALL set this field to the value returned by
  /// `ControllerPublishVolume` if the corresponding Controller Plugin
  /// has `PUBLISH_UNPUBLISH_VOLUME` controller capability, and SHALL be
  /// left unset if the corresponding Controller Plugin does not have
  /// this capability. This is an OPTIONAL field.
  #[inline]
  pub fn publish_context(&self) -> &HashMap<String, String> {
    &self.publish_context
  }

  /// The path to which the volume MAY be staged. It MUST be an
  /// absolute path in the root filesystem of the process serving this
  /// request, and MUST be a directory. The CO SHALL ensure that there
  /// is only one `staging_target_path` per volume. The CO SHALL ensure
  /// that the path is directory and that the process serving the
  /// request has `read` and `write` permission to that directory. The
  /// CO SHALL be responsible for creating the directory if it does not
  /// exist.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn staging_target_path(&self) -> &Path {
    &self.staging_target_path
  }

  /// Volume capability describing how the CO intends to use this volume.
  /// SP MUST ensure the CO can use the staged volume as described.
  /// Otherwise SP MUST return the appropriate gRPC error code.
  /// This is a REQUIRED field.
  #[inline]
  pub fn volume_capability(&self) -> &VolumeCapability {
    &self.volume_capability
  }

  /// Secrets required by plugin to complete node stage volume request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }

  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  #[inline]
  pub fn volume_context(&self) -> &HashMap<String, String> {
    &self.volume_context
  }
}

impl TryFrom<proto::NodeStageVolumeRequest> for NodeStageVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::NodeStageVolumeRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeStageVolumeRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let publish_context = value.publish_context;
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
        v if !v.is_dir() => {
          return Err(tonic::Status::invalid_argument(
            "NodeStageVolumeRequest.staging_target_path is not a directory",
          ))
        }
        v => v,
      },
    };

    let volume_capability = match value.volume_capability {
      None => {
        return Err(tonic::Status::invalid_argument(
          "NodeStageVolumeRequest.volume_capability is missing",
        ))
      }
      Some(v) => v.try_into()?,
    };

    let secrets = value.secrets.into();
    let volume_context = value.volume_context;

    Ok(NodeStageVolumeRequest {
      volume_id,
      publish_context,
      staging_target_path,
      volume_capability,
      secrets,
      volume_context,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NodeStageVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  /// Indicates that a volume corresponding to the specified `volume_id` has already been
  /// published at the specified `staging_target_path` but is incompatible with the specified
  /// `volume_capability` flag.
  #[error("Volume published but is incompatible: {0}")]
  IncompatibleVolumePublished(String),

  /// Indicates that the CO has exceeded the volume's capabilities because the volume does
  /// not have MULTI_NODE capability.
  #[error("Exceeds capabilities: {0}")]
  ExceedsCapabilities(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<NodeStageVolumeError> for tonic::Status {
  fn from(value: NodeStageVolumeError) -> Self {
    match value {
      NodeStageVolumeError::Other(v) => v,
      value => {
        let code = match &value {
          NodeStageVolumeError::VolumeNotFound(_) => Code::NotFound,
          NodeStageVolumeError::IncompatibleVolumePublished(_) => Code::AlreadyExists,
          NodeStageVolumeError::ExceedsCapabilities(_) => Code::FailedPrecondition,
          NodeStageVolumeError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
