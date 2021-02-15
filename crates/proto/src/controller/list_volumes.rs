use super::{Volume, VolumeStatus};
use crate::proto;
use std::{
  convert::{TryFrom, TryInto},
  num::NonZeroU32,
};
use thiserror::Error;

#[derive(Debug)]
pub struct ListVolumesRequest {
  max_entries: Option<NonZeroU32>,
  starting_token: Option<String>,
}

impl ListVolumesRequest {
  /// If specified (non-zero value), the Plugin MUST NOT return more
  /// entries than this number in the response. If the actual number of
  /// entries is more than this number, the Plugin MUST set `next_token`
  /// in the response which can be used to get the next page of entries
  /// in the subsequent `ListVolumes` call. This field is OPTIONAL. If
  /// not specified (zero value), it means there is no restriction on the
  /// number of entries that can be returned.
  #[inline]
  pub fn max_entries(&self) -> Option<NonZeroU32> {
    self.max_entries
  }

  /// A token to specify where to start paginating. Set this field to
  /// `next_token` returned by a previous `ListVolumes` call to get the
  /// next page of entries. This field is OPTIONAL.
  #[inline]
  pub fn starting_token(&self) -> Option<&str> {
    self.starting_token.as_deref()
  }
}

impl TryFrom<proto::ListVolumesRequest> for ListVolumesRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::ListVolumesRequest) -> Result<Self, Self::Error> {
    let max_entries = match value.max_entries {
      v if v < 0 => {
        return Err(tonic::Status::invalid_argument(
          "ListVolumesRequest.max_entries was less than 0",
        ))
      }
      v => NonZeroU32::new(v as u32),
    };

    let starting_token = match value.starting_token {
      v if v.is_empty() => None,
      v => Some(v),
    };

    Ok(ListVolumesRequest {
      max_entries,
      starting_token,
    })
  }
}

#[derive(Debug)]
pub struct VolumeListEntry {
  /// The volume
  volume: Volume,
  /// This field is OPTIONAL. This field MUST be specified if the
  /// LIST_VOLUMES_PUBLISHED_NODES controller capability is
  /// supported.
  status: Option<VolumeStatus>,
}

impl TryFrom<VolumeListEntry> for proto::list_volumes_response::Entry {
  type Error = tonic::Status;

  fn try_from(value: VolumeListEntry) -> Result<Self, Self::Error> {
    let volume = Some(value.volume.try_into()?);
    let status = value.status.map(TryInto::try_into).transpose()?;

    Ok(proto::list_volumes_response::Entry { volume, status })
  }
}

#[derive(Debug)]
pub struct ListVolumesResponse {
  /// The volume entires.
  entries: Vec<VolumeListEntry>,
  /// This token allows you to get the next page of entries for
  /// `ListVolumes` request. If the number of entries is larger than
  /// `max_entries`, use the `next_token` as a value for the
  /// `starting_token` field in the next `ListVolumes` request. This
  /// field is OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  next_token: Option<String>,
}

impl TryFrom<ListVolumesResponse> for proto::ListVolumesResponse {
  type Error = tonic::Status;

  fn try_from(value: ListVolumesResponse) -> Result<Self, Self::Error> {
    let entries = value
      .entries
      .into_iter()
      .map(TryInto::try_into)
      .collect::<Result<_, _>>()?;
    let next_token = value.next_token.unwrap_or_default();

    Ok(proto::ListVolumesResponse {
      entries,
      next_token,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ListVolumesError {
  /// Indicates that `starting_token` is not valid.
  #[error("Invalid `starting_token`: {0}")]
  InvalidStartingToken(String),

  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

use tonic::{Code, Status};
impl From<ListVolumesError> for tonic::Status {
  fn from(value: ListVolumesError) -> Self {
    match value {
      ListVolumesError::Other(v) => v,
      value => {
        let code = match &value {
          ListVolumesError::InvalidStartingToken(_) => Code::Aborted,
          ListVolumesError::Other(_) => unreachable!(),
        };

        Status::new(code, value.to_string())
      }
    }
  }
}
