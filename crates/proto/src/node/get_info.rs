use super::Topology;
use crate::proto;
use std::{convert::TryFrom, num::NonZeroU64};
use thiserror::Error;

#[derive(Debug)]
pub struct NodeGetInfoResponse {
  /// The identifier of the node as understood by the SP.
  /// This field is REQUIRED.
  /// This field MUST contain enough information to uniquely identify
  /// this specific node vs all other nodes supported by this plugin.
  /// This field SHALL be used by the CO in subsequent calls, including
  /// `ControllerPublishVolume`, to refer to this node.
  /// The SP is NOT responsible for global uniqueness of node_id across
  /// multiple SPs.
  /// This field overrides the general CSI size limit.
  /// The size of this field SHALL NOT exceed 192 bytes. The general
  /// CSI size limit, 128 byte, is RECOMMENDED for best backwards
  /// compatibility.
  node_id: String,

  /// Maximum number of volumes that controller can publish to the node.
  /// If value is not set or zero CO SHALL decide how many volumes of
  /// this type can be published by the controller to the node. The
  /// plugin MUST NOT set negative values here.
  /// This field is OPTIONAL.
  max_volumes_per_node: Option<NonZeroU64>,

  /// Specifies where (regions, zones, racks, etc.) the node is
  /// accessible from.
  /// A plugin that returns this field MUST also set the
  /// VOLUME_ACCESSIBILITY_CONSTRAINTS plugin capability.
  /// COs MAY use this information along with the topology information
  /// returned in CreateVolumeResponse to ensure that a given volume is
  /// accessible from a given node when scheduling workloads.
  /// This field is OPTIONAL. If it is not specified, the CO MAY assume
  /// the node is not subject to any topological constraint, and MAY
  /// schedule workloads that reference any volume V, such that there are
  /// no topological constraints declared for V.
  ///
  /// Example 1:
  /// ```text
  ///   accessible_topology =
  ///     {"region": "R1", "zone": "Z2"}
  /// ```
  /// Indicates the node exists within the "region" "R1" and the "zone"
  /// "Z2".
  accessible_topology: Option<Topology>,
}

impl TryFrom<NodeGetInfoResponse> for proto::NodeGetInfoResponse {
  type Error = tonic::Status;

  fn try_from(value: NodeGetInfoResponse) -> Result<Self, Self::Error> {
    let node_id = value.node_id;
    let max_volumes_per_node = value
      .max_volumes_per_node
      .map(|v| v.get() as i64)
      .unwrap_or_default();
    let accessible_topology = value
      .accessible_topology
      .map(|segments| proto::Topology { segments });

    Ok(proto::NodeGetInfoResponse {
      node_id,
      max_volumes_per_node,
      accessible_topology,
    })
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NodeGetInfoError {
  #[error(transparent)]
  #[doc(hidden)]
  Other(#[from] tonic::Status),
}

impl From<NodeGetInfoError> for tonic::Status {
  fn from(value: NodeGetInfoError) -> Self {
    match value {
      NodeGetInfoError::Other(v) => v,
    }
  }
}
