use super::{Secrets, Snapshot};
use crate::proto;
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
  num::NonZeroU32,
};
use thiserror::Error;

#[derive(Debug)]
pub struct ListSnapshotsRequest {
  max_entries: Option<NonZeroU32>,
  starting_token: Option<String>,
  source_volume_id: Option<String>,
  snapshot_id: Option<String>,
  secrets: Secrets,
}

impl ListSnapshotsRequest {
  /// If specified (non-zero value), the Plugin MUST NOT return more
  /// entries than this number in the response. If the actual number of
  /// entries is more than this number, the Plugin MUST set `next_token`
  /// in the response which can be used to get the next page of entries
  /// in the subsequent `ListSnapshots` call. This field is OPTIONAL. If
  /// not specified (zero value), it means there is no restriction on the
  /// number of entries that can be returned.
  /// The value of this field MUST NOT be negative.
  #[inline]
  pub fn max_entries(&self) -> Option<NonZeroU32> {
    self.max_entries
  }

  /// A token to specify where to start paginating. Set this field to
  /// `next_token` returned by a previous `ListSnapshots` call to get the
  /// next page of entries. This field is OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  #[inline]
  pub fn starting_token(&self) -> Option<&str> {
    self.starting_token.as_deref()
  }

  /// Identity information for the source volume. This field is OPTIONAL.
  /// It can be used to list snapshots by volume.
  #[inline]
  pub fn source_volume_id(&self) -> Option<&str> {
    self.source_volume_id.as_deref()
  }

  /// Identity information for a specific snapshot. This field is
  /// OPTIONAL. It can be used to list only a specific snapshot.
  /// ListSnapshots will return with current snapshot information
  /// and will not block if the snapshot is being processed after
  /// it is cut.
  #[inline]
  pub fn snapshot_id(&self) -> Option<&str> {
    self.snapshot_id.as_deref()
  }

  /// Secrets required by plugin to complete ListSnapshot request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[inline]
  pub fn secrets(&self) -> &HashMap<String, String> {
    self.secrets.as_ref()
  }
}

impl TryFrom<proto::ListSnapshotsRequest> for ListSnapshotsRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ListSnapshotsRequest) -> Result<Self, Self::Error> {
    let max_entries = match value.max_entries {
      v if v < 0 => {
        return Err(tonic::Status::invalid_argument(
          "ListSnapshotsRequest.max_entries was less than 0",
        ))
      }
      v => NonZeroU32::new(v as u32),
    };

    let starting_token = match value.starting_token {
      v if v.is_empty() => None,
      v => Some(v),
    };

    let source_volume_id = match value.source_volume_id {
      v if v.is_empty() => None,
      v => Some(v),
    };

    let snapshot_id = match value.snapshot_id {
      v if v.is_empty() => None,
      v => Some(v),
    };

    let secrets = value.secrets.into();

    Ok(ListSnapshotsRequest {
      max_entries,
      starting_token,
      source_volume_id,
      snapshot_id,
      secrets,
    })
  }
}

impl TryFrom<Snapshot> for proto::list_snapshots_response::Entry {
  type Error = tonic::Status;

  fn try_from(value: Snapshot) -> Result<Self, Self::Error> {
    let snapshot = Some(value.try_into()?);

    Ok(proto::list_snapshots_response::Entry { snapshot })
  }
}

#[derive(Debug)]
pub struct ListSnapshotsResponse {
  entries: Vec<Snapshot>,
  next_token: Option<String>,
}

impl TryFrom<ListSnapshotsResponse> for proto::ListSnapshotsResponse {
  type Error = tonic::Status;

  fn try_from(value: ListSnapshotsResponse) -> Result<Self, Self::Error> {
    let entries = value
      .entries
      .into_iter()
      .map(TryInto::try_into)
      .collect::<Result<_, _>>()?;
    let next_token = value.next_token.unwrap_or_default();

    Ok(proto::ListSnapshotsResponse {
      entries,
      next_token,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ListSnapshotsError {
  /// Indicates that `starting_token` is not valid.
  #[error("Invalid `starting_token`: {0}")]
  InvalidStartingToken(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<ListSnapshotsError> for tonic::Status {
  fn from(value: ListSnapshotsError) -> Self {
    match value {
      ListSnapshotsError::Other(v) => v,
      value => {
        let code = match &value {
          ListSnapshotsError::InvalidStartingToken(_) => Code::Aborted,
          ListSnapshotsError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
