use super::{Topology, VolumeCapability};
use crate::proto;
use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
  num::NonZeroU64,
};
use thiserror::Error;

#[derive(Debug)]
pub struct GetCapacityRequest {
  volume_capabilities: Vec<VolumeCapability>,
  parameters: HashMap<String, String>,
  accessible_topology: Option<Topology>,
}

impl GetCapacityRequest {
  /// If specified, the Plugin SHALL report the capacity of the storage
  /// that can be used to provision volumes that satisfy ALL of the
  /// specified `volume_capabilities`. These are the same
  /// `volume_capabilities` the CO will use in `CreateVolumeRequest`.
  /// This field is OPTIONAL.
  #[inline]
  pub fn volume_capabilities(&self) -> &[VolumeCapability] {
    &self.volume_capabilities
  }

  /// If specified, the Plugin SHALL report the capacity of the storage
  /// that can be used to provision volumes with the given Plugin
  /// specific `parameters`. These are the same `parameters` the CO will
  /// use in `CreateVolumeRequest`. This field is OPTIONAL.
  #[inline]
  pub fn parameters(&self) -> &HashMap<String, String> {
    &self.parameters
  }

  /// If specified, the Plugin SHALL report the capacity of the storage
  /// that can be used to provision volumes that in the specified
  /// `accessible_topology`. This is the same as the
  /// `accessible_topology` the CO returns in a `CreateVolumeResponse`.
  /// This field is OPTIONAL. This field SHALL NOT be set unless the
  /// plugin advertises the VOLUME_ACCESSIBILITY_CONSTRAINTS capability.
  #[inline]
  pub fn accessible_topology(&self) -> Option<&Topology> {
    self.accessible_topology.as_ref()
  }
}

impl TryFrom<proto::GetCapacityRequest> for GetCapacityRequest {
  type Error = tonic::Status;

  fn try_from(value: proto::GetCapacityRequest) -> Result<Self, Self::Error> {
    let volume_capabilities = value
      .volume_capabilities
      .into_iter()
      .map(TryInto::try_into)
      .collect::<Result<_, _>>()?;
    let parameters = value.parameters;
    let accessible_topology = value.accessible_topology.map(|t| t.segments);

    Ok(GetCapacityRequest {
      volume_capabilities,
      parameters,
      accessible_topology,
    })
  }
}

#[derive(Debug)]
pub struct GetCapacityResponse {
  /// The available capacity, in bytes, of the storage that can be used
  /// to provision volumes. If `volume_capabilities` or `parameters` is
  /// specified in the request, the Plugin SHALL take those into
  /// consideration when calculating the available capacity of the
  /// storage. This field is REQUIRED.
  available_capacity: NonZeroU64,
}

impl TryFrom<GetCapacityResponse> for proto::GetCapacityResponse {
  type Error = tonic::Status;

  fn try_from(value: GetCapacityResponse) -> Result<Self, Self::Error> {
    let available_capacity = value.available_capacity.get() as i64;
    Ok(proto::GetCapacityResponse { available_capacity })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum GetCapacityError {
  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<GetCapacityError> for tonic::Status {
  fn from(value: GetCapacityError) -> Self {
    match value {
      GetCapacityError::Other(v) => v,
    }
  }
}
