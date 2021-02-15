mod capabilities;
mod expand_volume;
mod get_info;
mod get_volume_stats;
mod publish_volume;
mod stage_volume;
mod unpublish_volume;
mod unstage_volume;

use crate::{
  plugin, proto,
  secrets::*,
  utils::{record_request, Record},
  IdentityService,
};
use async_trait::async_trait;
use std::{convert::TryInto, sync::Arc};
use tracing::instrument;

pub use crate::volume::*;
pub use capabilities::*;
pub use expand_volume::*;
pub use get_info::*;
pub use get_volume_stats::*;
pub use publish_volume::*;
pub use stage_volume::*;
pub use unpublish_volume::*;
pub use unstage_volume::*;

#[async_trait]
pub trait NodeService: IdentityService {
  /// Get the set of services provided by this controller.
  #[inline]
  fn capabilities(&self) -> NodeCapabilities {
    NodeCapabilities::empty()
  }

  /// A Node Plugin MUST implement this RPC call if it has `STAGE_UNSTAGE_VOLUME`
  /// node capability.
  ///
  /// This RPC is called by the CO prior to the volume being consumed by any
  /// workloads on the node by `NodePublishVolume`. The Plugin SHALL assume
  /// that this RPC will be executed on the node where the volume will be used.
  /// This RPC SHOULD be called by the CO when a workload that wants to use the
  /// specified volume is placed (scheduled) on the specified node for the first
  /// time or for the first time since a `NodeUnstageVolume` call for the specified
  /// volume was called and returned success on that node.
  ///
  /// If the corresponding Controller Plugin has `PUBLISH_UNPUBLISH_VOLUME`
  /// controller capability and the Node Plugin has `STAGE_UNSTAGE_VOLUME`
  /// capability, then the CO MUST guarantee that this RPC is called after
  /// `ControllerPublishVolume` is called for the given volume on the given node
  /// and returns a success. The CO MUST guarantee that this RPC is called and
  /// returns a success before any `NodePublishVolume` is called for the given
  /// volume on the given node.
  ///
  /// This operation MUST be idempotent. If the volume corresponding to the `volume_id`
  /// is already staged to the `staging_target_path`, and is identical to the specified
  /// `volume_capability` the Plugin MUST reply `0 OK`.
  ///
  /// If this RPC failed, or the CO does not know if it failed or not, it MAY choose to
  /// call `NodeStageVolume` again, or choose to call `NodeUnstageVolume`.
  #[allow(unused_variables)]
  async fn node_stage_volume(
    &self,
    request: NodeStageVolumeRequest,
  ) -> Result<(), NodeStageVolumeError> {
    unsupported!("NodeStageVolume")
  }

  /// A Node Plugin MUST implement this RPC call if it has `STAGE_UNSTAGE_VOLUME`
  /// node capability.
  ///
  /// This RPC is a reverse operation of `NodeStageVolume`. This RPC MUST undo the
  /// work by the corresponding `NodeStageVolume`. This RPC SHALL be called by the
  /// CO once for each `staging_target_path` that was successfully setup via
  /// `NodeStageVolume`.
  ///
  /// If the corresponding Controller Plugin has `PUBLISH_UNPUBLISH_VOLUME` controller
  /// capability and the Node Plugin has `STAGE_UNSTAGE_VOLUME` capability, the CO MUST
  /// guarantee that this RPC is called and returns success before calling
  /// `ControllerUnpublishVolume` for the given node and the given volume. The CO MUST
  /// guarantee that this RPC is called after all `NodeUnpublishVolume` have been called
  /// and returned success for the given volume on the given node.
  ///
  /// The Plugin SHALL assume that this RPC will be executed on the node where the volume
  /// is being used. This RPC MAY be called by the CO when the workload using the volume
  /// is being moved to a different node, or all the workloads using the volume on a node
  /// have finished.
  ///
  /// This operation MUST be idempotent. If the volume corresponding to the `volume_id` is
  /// not staged to the `staging_target_path`,  the Plugin MUST reply `0 OK`.
  ///
  /// If this RPC failed, or the CO does not know if it failed or not, it MAY choose to call
  /// `NodeUnstageVolume` again.
  #[allow(unused_variables)]
  async fn node_unstage_volume(
    &self,
    request: NodeUnstageVolumeRequest,
  ) -> Result<(), NodeUnstageVolumeError> {
    unsupported!("NodeUnstageVolume")
  }

  /// This RPC is called by the CO when a workload that wants to use the specified volume
  /// is placed (scheduled) on a node. The Plugin SHALL assume that this RPC will be executed
  /// on the node where the volume will be used.
  ///
  /// If the corresponding Controller Plugin has `PUBLISH_UNPUBLISH_VOLUME` controller
  /// capability, the CO MUST guarantee that this RPC is called after `ControllerPublishVolume`
  /// is called for the given volume on the given node and returns a success.
  ///
  /// This operation MUST be idempotent. If the volume corresponding to the `volume_id` has
  /// already been published at the specified `target_path`, and is compatible with the specified
  /// `volume_capability` and `readonly` flag, the Plugin MUST reply `0 OK`.
  ///
  /// If this RPC failed, or the CO does not know if it failed or not, it MAY choose to call
  /// `NodePublishVolume` again, or choose to call `NodeUnpublishVolume`.
  ///
  /// This RPC MAY be called by the CO multiple times on the same node for the same volume with
  /// possibly different `target_path` and/or other arguments if the volume has MULTI_NODE
  /// capability (i.e., `access_mode` is either `MULTI_NODE_READER_ONLY`, `MULTI_NODE_SINGLE_WRITER`
  /// or `MULTI_NODE_MULTI_WRITER`).
  ///
  /// The following table shows what the Plugin SHOULD return when receiving a second
  /// `NodePublishVolume` on the same volume on the same node:
  ///
  /// |                | T1=T2, P1=P2    | T1=T2, P1!=P2  | T1!=T2, P1=P2       | T1!=T2, P1!=P2     |
  /// |----------------|-----------------|----------------|---------------------|--------------------|
  /// | MULTI_NODE     | OK (idempotent) | ALREADY_EXISTS | OK                  | OK                 |
  /// | Non MULTI_NODE | OK (idempotent) | ALREADY_EXISTS | FAILED_PRECONDITION | FAILED_PRECONDITION|
  ///
  /// (`Tn`: target path of the n-th `NodePublishVolume`, `Pn`: other arguments of the n-th
  /// `NodePublishVolume` except `secrets`)
  async fn node_publish_volume(
    &self,
    request: NodePublishVolumeRequest,
  ) -> Result<(), NodePublishVolumeError>;

  /// A Node Plugin MUST implement this RPC call. This RPC is a reverse operation of `NodePublishVolume`.
  /// This RPC MUST undo the work by the corresponding `NodePublishVolume`. This RPC SHALL be called by
  /// the CO at least once for each `target_path` that was successfully setup via `NodePublishVolume`.
  /// If the corresponding Controller Plugin has `PUBLISH_UNPUBLISH_VOLUME` controller capability, the
  /// CO SHOULD issue all `NodeUnpublishVolume` (as specified above) before calling
  /// `ControllerUnpublishVolume` for the given node and the given volume. The Plugin SHALL assume that
  /// this RPC will be executed on the node where the volume is being used.
  ///
  /// This RPC is typically called by the CO when the workload using the volume is being moved to a
  /// different node, or all the workload using the volume on a node has finished.
  ///
  /// This operation MUST be idempotent. If this RPC failed, or the CO does not know if it failed or not,
  /// it can choose to call `NodeUnpublishVolume` again.
  async fn node_unpublish_volume(
    &self,
    request: NodeUnpublishVolumeRequest,
  ) -> Result<(), NodeUnpublishVolumeError>;

  /// A Node plugin MUST implement this RPC call if it has GET_VOLUME_STATS node capability or
  /// VOLUME_CONDITION node capability. `NodeGetVolumeStats` RPC call returns the volume capacity
  /// statistics available for the volume.
  ///
  /// If the volume is being used in `BlockVolume` mode then `used` and `available` MAY be omitted
  /// from `usage` field of `NodeGetVolumeStatsResponse`. Similarly, inode information MAY be omitted
  /// from `NodeGetVolumeStatsResponse` when unavailable.
  ///
  /// The `staging_target_path` field is not required, for backwards compatibility, but the CO SHOULD
  /// supply it. Plugins can use this field to determine if `volume_path` is where the volume is
  /// published or staged, and setting this field to non-empty allows plugins to function with less
  /// stored state on the node.
  #[allow(unused_variables)]
  async fn node_get_volume_stats(
    &self,
    request: NodeGetVolumeStatsRequest,
  ) -> Result<NodeGetVolumeStatsResponse, NodeGetVolumeStatsError> {
    unsupported!("NodeGetVolumeStats")
  }

  /// A Node Plugin MUST implement this RPC call if it has `EXPAND_VOLUME` node capability.
  /// This RPC call allows CO to expand volume on a node.
  ///
  /// This operation MUST be idempotent. If a volume corresponding to the specified volume ID
  /// is already larger than or equal to the target capacity of the expansion request, the plugin
  /// SHOULD reply 0 OK.
  ///
  /// `NodeExpandVolume` ONLY supports expansion of already node-published or node-staged volumes
  /// on the given `volume_path`.
  ///
  /// If plugin has `STAGE_UNSTAGE_VOLUME` node capability then:
  /// * `NodeExpandVolume` MUST be called after successful `NodeStageVolume`.
  /// * `NodeExpandVolume` MAY be called before or after `NodePublishVolume`.
  ///
  /// Otherwise `NodeExpandVolume` MUST be called after successful `NodePublishVolume`.
  ///
  /// If a plugin only supports expansion via the `VolumeExpansion.OFFLINE` capability, then the
  /// volume MUST first be taken offline and expanded via `ControllerExpandVolume` (see
  /// `ControllerExpandVolume` for more details), and then node-staged or node-published before it
  /// can be expanded on the node via `NodeExpandVolume`.
  ///
  /// The `staging_target_path` field is not required, for backwards compatibility, but the CO SHOULD
  /// supply it. Plugins can use this field to determine if `volume_path` is where the volume is
  /// published or staged, and setting this field to non-empty allows plugins to function with less
  /// stored state on the node.
  #[allow(unused_variables)]
  async fn node_expand_volume(
    &self,
    request: NodeExpandVolumeRequest,
  ) -> Result<NodeExpandVolumeResponse, NodeExpandVolumeError> {
    unsupported!("NodeExpandVolume")
  }

  /// A Node Plugin MUST implement this RPC call if the plugin has `PUBLISH_UNPUBLISH_VOLUME`
  /// controller capability. The Plugin SHALL assume that this RPC will be executed on the
  /// node where the volume will be used. The CO SHOULD call this RPC for the node at which
  /// it wants to place the workload. The CO MAY call this RPC more than once for a given node.
  /// The SP SHALL NOT expect the CO to call this RPC more than once. The result of this call
  /// will be used by CO in `ControllerPublishVolume`.
  #[allow(unused_variables)]
  async fn node_get_info(&self) -> Result<NodeGetInfoResponse, NodeGetInfoError> {
    unsupported!("NodeGetInfo")
  }
}

struct Node<T: NodeService>(Arc<T>);

#[async_trait]
impl<T: NodeService> proto::identity_server::Identity for Node<T> {
  #[instrument(
    name = "identity.get_plugin_info",
    skip(self, _request),
    fields(name, vendor_version, manifest)
  )]
  async fn get_plugin_info(
    &self,
    _request: tonic::Request<proto::GetPluginInfoRequest>,
  ) -> Result<tonic::Response<proto::GetPluginInfoResponse>, tonic::Status> {
    let response = proto::GetPluginInfoResponse {
      name: self.0.name().into(),
      vendor_version: self.0.version().into(),
      manifest: self.0.manifest().clone(),
    };

    Ok(tonic::Response::new(response))
  }

  // TODO: Instrument response
  #[instrument(name = "identity.get_plugin_capabilities", skip(self, _request))]
  async fn get_plugin_capabilities(
    &self,
    _request: tonic::Request<proto::GetPluginCapabilitiesRequest>,
  ) -> Result<tonic::Response<proto::GetPluginCapabilitiesResponse>, tonic::Status> {
    let response = plugin::get_capabilities(&*self.0);

    Ok(tonic::Response::new(response))
  }

  #[instrument(name = "identity.probe", skip(self, _request), fields(ready))]
  async fn probe(
    &self,
    _request: tonic::Request<proto::ProbeRequest>,
  ) -> Result<tonic::Response<proto::ProbeResponse>, tonic::Status> {
    let response = proto::ProbeResponse {
      ready: Some(self.0.ready()),
    };

    Ok(tonic::Response::new(response))
  }
}

#[async_trait]
impl<T: NodeService> proto::node_server::Node for Node<T> {
  #[instrument(name = "node.node_stage_volume", skip(self, request), fields(request))]
  async fn node_stage_volume(
    &self,
    request: tonic::Request<proto::NodeStageVolumeRequest>,
  ) -> Result<tonic::Response<proto::NodeStageVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.node_stage_volume(request).await?;
    let response = proto::NodeStageVolumeResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "node.node_unstage_volume",
    skip(self, request),
    fields(request)
  )]
  async fn node_unstage_volume(
    &self,
    request: tonic::Request<proto::NodeUnstageVolumeRequest>,
  ) -> Result<tonic::Response<proto::NodeUnstageVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.node_unstage_volume(request).await?;
    let response = proto::NodeUnstageVolumeResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "node.node_publish_volume",
    skip(self, request),
    fields(request)
  )]
  async fn node_publish_volume(
    &self,
    request: tonic::Request<proto::NodePublishVolumeRequest>,
  ) -> Result<tonic::Response<proto::NodePublishVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.node_publish_volume(request).await?;
    let response = proto::NodePublishVolumeResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "node.node_unpublish_volume",
    skip(self, request),
    fields(request)
  )]
  async fn node_unpublish_volume(
    &self,
    request: tonic::Request<proto::NodeUnpublishVolumeRequest>,
  ) -> Result<tonic::Response<proto::NodeUnpublishVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.node_unpublish_volume(request).await?;
    let response = proto::NodeUnpublishVolumeResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "node.node_get_volume_stats",
    skip(self, request),
    fields(request, response)
  )]
  async fn node_get_volume_stats(
    &self,
    request: tonic::Request<proto::NodeGetVolumeStatsRequest>,
  ) -> Result<tonic::Response<proto::NodeGetVolumeStatsResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .node_get_volume_stats(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "node.node_expand_volume",
    skip(self, request),
    fields(request, response)
  )]
  async fn node_expand_volume(
    &self,
    request: tonic::Request<proto::NodeExpandVolumeRequest>,
  ) -> Result<tonic::Response<proto::NodeExpandVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .node_expand_volume(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(name = "node.node_get_capabilities", skip(self), fields(response))]
  async fn node_get_capabilities(
    &self,
    _: tonic::Request<proto::NodeGetCapabilitiesRequest>,
  ) -> Result<tonic::Response<proto::NodeGetCapabilitiesResponse>, tonic::Status> {
    let response = self.0.capabilities().record_response().try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(name = "node.node_get_info", skip(self), fields(response))]
  async fn node_get_info(
    &self,
    _: tonic::Request<proto::NodeGetInfoRequest>,
  ) -> Result<tonic::Response<proto::NodeGetInfoResponse>, tonic::Status> {
    let response = self.0.node_get_info().await?.record_response().try_into()?;
    Ok(tonic::Response::new(response))
  }
}
