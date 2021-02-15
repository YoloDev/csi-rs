use super::Secrets;
use crate::proto;
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

#[derive(Debug)]
pub struct CreateSnapshotRequest {
  source_volume_id: String,
  name: String,
  secrets: Secrets,
  parameters: HashMap<String, String>,
}

impl CreateSnapshotRequest {
  /// The ID of the source volume to be snapshotted.
  #[inline]
  pub fn source_volume_id(&self) -> &str {
    &self.source_volume_id
  }

  /// The suggested name for the snapshot. This field is REQUIRED for
  /// idempotency.
  /// Any Unicode string that conforms to the length limit is allowed
  /// except those containing the following banned characters:
  /// U+0000-U+0008, U+000B, U+000C, U+000E-U+001F, U+007F-U+009F.
  /// (These are control characters other than commonly used whitespace.)
  #[inline]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Secrets required by plugin to complete snapshot creation request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }

  /// Plugin specific parameters passed in as opaque key-value pairs.
  /// This field is OPTIONAL. The Plugin is responsible for parsing and
  /// validating these parameters. COs will treat these as opaque.
  /// Use cases for opaque parameters:
  /// - Specify a policy to automatically clean up the snapshot.
  /// - Specify an expiration date for the snapshot.
  /// - Specify whether the snapshot is readonly or read/write.
  /// - Specify if the snapshot should be replicated to some place.
  /// - Specify primary or secondary for replication systems that
  ///   support snapshotting only on primary.
  #[inline]
  pub fn parameters(&self) -> &HashMap<String, String> {
    &self.parameters
  }
}

impl TryFrom<proto::CreateSnapshotRequest> for CreateSnapshotRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::CreateSnapshotRequest) -> Result<Self, Self::Error> {
    let source_volume_id = match value.source_volume_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "CreateSnapshotRequest.source_volume_id is empty",
        ))
      }
      v => v,
    };

    let name = match value.name {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "CreateSnapshotRequest.name is empty",
        ))
      }
      v => v,
    };

    let secrets = value.secrets.into();
    let parameters = value.parameters;

    Ok(CreateSnapshotRequest {
      source_volume_id,
      name,
      secrets,
      parameters,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CreateSnapshotError {
  /// Indicates that a snapshot corresponding to the specified snapshot `name`
  ///already exists but is incompatible with the specified `volume_id`.
  #[error("Snapshot already exists but is incompatible: {0}")]
  AlreadyExists(String),

  /// Indicates that there is already an operation pending for the specified snapshot.
  /// In general the Cluster Orchestrator (CO) is responsible for ensuring that there
  /// is no more than one call "in-flight" per snapshot at a given time. However, in some
  /// circumstances, the CO MAY lose state (for example when the CO crashes and restarts),
  /// and MAY issue multiple calls simultaneously for the same snapshot. The Plugin, SHOULD
  /// handle this as gracefully as possible, and MAY return this error code to reject
  /// secondary calls.
  #[error("Operation pending for snapshot: {0}")]
  Pending(String),

  /// There is not enough space on the storage system to handle the create snapshot request.
  #[error("Not enough space to create snapshot: {0}")]
  NotEnoughSpace(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<CreateSnapshotError> for tonic::Status {
  fn from(value: CreateSnapshotError) -> Self {
    match value {
      CreateSnapshotError::Other(v) => v,
      value => {
        let code = match &value {
          CreateSnapshotError::AlreadyExists(_) => Code::AlreadyExists,
          CreateSnapshotError::Pending(_) => Code::Aborted,
          CreateSnapshotError::NotEnoughSpace(_) => Code::ResourceExhausted,
          CreateSnapshotError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
