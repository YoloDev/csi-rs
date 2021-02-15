use super::VolumeCapability;
use crate::{proto, secrets::Secrets};
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct NodePublishVolumeRequest {
  volume_id: String,
  publish_context: HashMap<String, String>,
  staging_target_path: Option<PathBuf>,
  target_path: PathBuf,
  volume_capability: VolumeCapability,
  readonly: bool,
  secrets: Secrets,
  volume_context: HashMap<String, String>,
}

impl NodePublishVolumeRequest {
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

  /// The path to which the volume was staged by `NodeStageVolume`.
  /// It MUST be an absolute path in the root filesystem of the process
  /// serving this request.
  /// It MUST be set if the Node Plugin implements the
  /// `STAGE_UNSTAGE_VOLUME` node capability.
  /// This is an OPTIONAL field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn staging_target_path(&self) -> Option<&Path> {
    self.staging_target_path.as_deref()
  }

  /// The path to which the volume will be published. It MUST be an
  /// absolute path in the root filesystem of the process serving this
  /// request. The CO SHALL ensure uniqueness of target_path per volume.
  /// The CO SHALL ensure that the parent directory of this path exists
  /// and that the process serving the request has `read` and `write`
  /// permissions to that parent directory.
  /// For volumes with an access type of block, the SP SHALL place the
  /// block device at target_path.
  /// For volumes with an access type of mount, the SP SHALL place the
  /// mounted directory at target_path.
  /// Creation of target_path is the responsibility of the SP.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn target_path(&self) -> &Path {
    &self.target_path
  }

  /// Volume capability describing how the CO intends to use this volume.
  /// SP MUST ensure the CO can use the published volume as described.
  /// Otherwise SP MUST return the appropriate gRPC error code.
  /// This is a REQUIRED field.
  #[inline]
  pub fn volume_capability(&self) -> &VolumeCapability {
    &self.volume_capability
  }

  /// Indicates SP MUST publish the volume in readonly mode.
  /// This field is REQUIRED.
  #[inline]
  pub fn readonly(&self) -> bool {
    self.readonly
  }

  /// Secrets required by plugin to complete node publish volume request.
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

impl TryFrom<proto::NodePublishVolumeRequest> for NodePublishVolumeRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::NodePublishVolumeRequest) -> Result<Self, Self::Error> {
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
      v if v.is_empty() => None,
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => {
          return Err(tonic::Status::invalid_argument(
            "NodePublishVolumeRequest.staging_target_path is not absolute",
          ))
        }
        v if !v.is_dir() => {
          return Err(tonic::Status::invalid_argument(
            "NodePublishVolumeRequest.staging_target_path is not a directory",
          ))
        }
        v => Some(v),
      },
    };

    let target_path = match value.target_path {
      v if v.is_empty() => return Err(tonic::Status::invalid_argument(
        "NodePublishVolumeRequest.target_path is empty",
      )),
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => return Err(tonic::Status::invalid_argument(
          "NodePublishVolumeRequest.target_path is not absolute",
        )),
        v if !v.parent().map(|p| p.is_dir()).unwrap_or(false) => return Err(tonic::Status::invalid_argument(
          "NodePublishVolumeRequest.staging_target_path is does not have a parent, or it's parent is not a directory",
        )),
        v => v,
      },
    };

    let volume_capability = match value.volume_capability {
      None => {
        return Err(tonic::Status::invalid_argument(
          "NodePublishVolumeRequest.volume_capability missing",
        ))
      }
      Some(v) => v.try_into()?,
    };

    let readonly = value.readonly;
    let secrets = value.secrets.into();
    let volume_context = value.volume_context;

    Ok(NodePublishVolumeRequest {
      volume_id,
      publish_context,
      staging_target_path,
      target_path,
      volume_capability,
      readonly,
      secrets,
      volume_context,
    })
  }
}

#[derive(Debug, Error)]
pub enum NodePublishVolumeError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  /// Indicates that a volume corresponding to the specified `volume_id` has already
  /// been published at the specified `target_path` but is incompatible with the specified
  /// `volume_capability` or `readonly` flag.
  #[error("Volume published but is incompatible: {0}")]
  IncompatibleVolumePublished(String),

  /// Indicates that the CO has exceeded the volume's capabilities because the volume does
  /// not have MULTI_NODE capability.
  #[error("Exceeds capabilities: {0}")]
  ExceedsCapabilities(String),

  /// Indicates that `STAGE_UNSTAGE_VOLUME` capability is set but no `staging_target_path`
  /// was set.
  #[error("Staging target path not set: {0}")]
  StagingTargetPathNotSet(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<NodePublishVolumeError> for tonic::Status {
  fn from(value: NodePublishVolumeError) -> Self {
    match value {
      NodePublishVolumeError::Other(v) => v,
      value => {
        let code = match &value {
          NodePublishVolumeError::VolumeNotFound(_) => Code::NotFound,
          NodePublishVolumeError::IncompatibleVolumePublished(_) => Code::AlreadyExists,
          NodePublishVolumeError::ExceedsCapabilities(_) => Code::FailedPrecondition,
          NodePublishVolumeError::StagingTargetPathNotSet(_) => Code::FailedPrecondition,
          NodePublishVolumeError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
