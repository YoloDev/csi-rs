use super::secrets::Secrets;
use crate::proto;
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

#[derive(Debug)]
pub struct DeleteSnapshotRequest {
  snapshot_id: String,
  secrets: Secrets,
}

impl DeleteSnapshotRequest {
  /// The ID of the snapshot to be deleted.
  #[inline]
  pub fn snapshot_id(&self) -> &str {
    &self.snapshot_id
  }

  /// Secrets required by plugin to complete snapshot deletion request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }
}

impl TryFrom<proto::DeleteSnapshotRequest> for DeleteSnapshotRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::DeleteSnapshotRequest) -> Result<Self, Self::Error> {
    let snapshot_id = match value.snapshot_id {
      v if v.is_empty() => {
        return Err(tonic::Status::invalid_argument(
          "DeleteSnapshotRequest.snapshot_id is empty",
        ))
      }
      v => v,
    };

    let secrets = value.secrets.into();

    Ok(DeleteSnapshotRequest {
      snapshot_id,
      secrets,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeleteSnapshotError {
  /// Indicates that the snapshot corresponding to the specified `snapshot_id`
  /// could not be deleted because it is in use by another resource.
  #[error("Snapshot in use: {0}")]
  SnapshotInUse(String),

  /// Indicates that there is already an operation pending for the specified snapshot.
  /// In general the Cluster Orchestrator (CO) is responsible for ensuring that there
  /// is no more than one call "in-flight" per snapshot at a given time. However, in some
  /// circumstances, the CO MAY lose state (for example when the CO crashes and restarts),
  /// and MAY issue multiple calls simultaneously for the same snapshot. The Plugin, SHOULD
  /// handle this as gracefully as possible, and MAY return this error code to reject
  /// secondary calls.
  #[error("Operation pending for snapshot: {0}")]
  Pending(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<DeleteSnapshotError> for tonic::Status {
  fn from(value: DeleteSnapshotError) -> Self {
    use tonic::{Code, Status};

    match value {
      DeleteSnapshotError::SnapshotInUse(v) => Status::new(Code::FailedPrecondition, v),
      DeleteSnapshotError::Pending(v) => Status::new(Code::Aborted, v),
      DeleteSnapshotError::Other(v) => v,
    }
  }
}
