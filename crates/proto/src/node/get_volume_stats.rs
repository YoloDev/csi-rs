use super::{VolumeCondition, VolumeUsage};
use crate::proto;
use std::{
  convert::{TryFrom, TryInto},
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct NodeGetVolumeStatsRequest {
  volume_id: String,
  volume_path: PathBuf,
  staging_target_path: Option<PathBuf>,
}

impl NodeGetVolumeStatsRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[inline]
  pub fn volume_id(&self) -> &str {
    &self.volume_id
  }

  /// It can be any valid path where volume was previously
  /// staged or published.
  /// It MUST be an absolute path in the root filesystem of
  /// the process serving this request.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[inline]
  pub fn volume_path(&self) -> &Path {
    &self.volume_path
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
}

impl TryFrom<proto::NodeGetVolumeStatsRequest> for NodeGetVolumeStatsRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::NodeGetVolumeStatsRequest) -> Result<Self, Self::Error> {
    let volume_id = match value.volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeGetVolumeStatsRequest.volume_id is empty",
        ))
      }
      v => v,
    };

    let volume_path = match value.volume_path {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "NodeGetVolumeStatsRequest.volume_path is empty",
        ))
      }
      v => match PathBuf::from(v) {
        v if !v.is_absolute() => {
          return Err(tonic::Status::invalid_argument(
            "NodeGetVolumeStatsRequest.volume_path is not absolute",
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
            "NodeGetVolumeStatsRequest.staging_target_path is not absolute",
          ))
        }
        v => Some(v),
      },
    };

    Ok(NodeGetVolumeStatsRequest {
      volume_id,
      volume_path,
      staging_target_path,
    })
  }
}

#[derive(Debug)]
pub struct NodeGetVolumeStatsResponse {
  /// This field is OPTIONAL.
  usage: Vec<VolumeUsage>,

  /// Information about the current condition of the volume.
  /// This field is OPTIONAL.
  /// This field MUST be specified if the VOLUME_CONDITION node
  /// capability is supported.
  volume_condition: Option<VolumeCondition>,
}

impl TryFrom<NodeGetVolumeStatsResponse> for proto::NodeGetVolumeStatsResponse {
  type Error = tonic::Status;

  fn try_from(value: NodeGetVolumeStatsResponse) -> Result<Self, Self::Error> {
    let usage = value
      .usage
      .into_iter()
      .map(TryInto::try_into)
      .collect::<Result<_, _>>()?;
    let volume_condition = value.volume_condition.map(TryInto::try_into).transpose()?;

    Ok(proto::NodeGetVolumeStatsResponse {
      usage,
      volume_condition,
    })
  }
}

#[derive(Debug, Error)]
pub enum NodeGetVolumeStatsError {
  /// Indicates that a volume corresponding to the specified `volume_id` does not exist.
  #[error("Volume does not exist: {0}")]
  VolumeNotFound(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<NodeGetVolumeStatsError> for tonic::Status {
  fn from(value: NodeGetVolumeStatsError) -> Self {
    match value {
      NodeGetVolumeStatsError::Other(v) => v,
      value => {
        let code = match &value {
          NodeGetVolumeStatsError::VolumeNotFound(_) => Code::NotFound,
          NodeGetVolumeStatsError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
