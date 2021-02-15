mod capabilities;
mod create_snapshot;
mod create_volume;
mod delete_snapshot;
mod delete_volume;
mod expand_volume;
mod get_capacity;
mod get_volume;
mod list_snapshots;
mod list_volumes;
mod publish_volume;
mod snapshot;
mod unpublish_volume;
mod validate_volume_capabilities;

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
pub use create_snapshot::*;
pub use create_volume::*;
pub use delete_snapshot::*;
pub use delete_volume::*;
pub use expand_volume::*;
pub use get_capacity::*;
pub use get_volume::*;
pub use list_snapshots::*;
pub use list_volumes::*;
pub use publish_volume::*;
pub use snapshot::*;
pub use unpublish_volume::*;
pub use validate_volume_capabilities::*;

#[async_trait]
pub trait ControllerService: IdentityService {
  /// Get the set of services provided by this controller.
  #[inline]
  fn capabilities(&self) -> ControllerCapabilities {
    ControllerCapabilities::empty()
  }

  /// A Controller Plugin MUST implement this RPC call if it has `CREATE_DELETE_VOLUME`
  /// controller capability.
  ///
  /// This RPC will be called by the CO to provision a new volume on behalf of a user
  /// (to be consumed as either a block device or a mounted filesystem).
  ///
  /// This operation MUST be idempotent.
  ///
  /// If a volume corresponding to the specified volume `name` already exists, is
  /// accessible from `accessibility_requirements`, and is compatible with the specified
  /// `capacity_range`, `volume_capabilities` and `parameters` in the `CreateVolumeRequest`,
  /// the Plugin MUST reply `0 OK` with the corresponding `CreateVolumeResponse`.
  ///
  /// Plugins MAY create 3 types of volumes:
  ///
  /// - Empty volumes. When plugin supports `CREATE_DELETE_VOLUME` OPTIONAL capability.
  /// - From an existing snapshot. When plugin supports `CREATE_DELETE_VOLUME` and
  ///  `CREATE_DELETE_SNAPSHOT` OPTIONAL capabilities.
  /// - From an existing volume. When plugin supports cloning, and reports the OPTIONAL
  ///   capabilities `CREATE_DELETE_VOLUME` and `CLONE_VOLUME`.
  #[allow(unused_variables)]
  async fn create_volume(&self, request: CreateVolumeRequest) -> Result<Volume, CreateVolumeError> {
    unsupported!("CreateVolume")
  }

  /// A Controller Plugin MUST implement this RPC call if it has CREATE_DELETE_VOLUME capability.
  /// This RPC will be called by the CO to deprovision a volume.
  ///
  /// This operation MUST be idempotent. If a volume corresponding to the specified volume_id
  /// does not exist or the artifacts associated with the volume do not exist anymore, the
  /// Plugin MUST reply 0 OK.
  ///
  /// CSI plugins SHOULD treat volumes independent from their snapshots.
  ///
  /// If the Controller Plugin supports deleting a volume without affecting its existing snapshots,
  /// then these snapshots MUST still be fully operational and acceptable as sources for new
  /// volumes as well as appear on ListSnapshot calls once the volume has been deleted.
  ///
  /// When a Controller Plugin does not support deleting a volume without affecting its
  /// existing snapshots, then the volume MUST NOT be altered in any way by the request and
  /// the operation must return the FAILED_PRECONDITION error code and MAY include meaningful
  /// human-readable information in the status.message field.
  #[allow(unused_variables)]
  async fn delete_volume(&self, request: DeleteVolumeRequest) -> Result<(), DeleteVolumeError> {
    unsupported!("DeleteVolume")
  }

  /// A Controller Plugin MUST implement this RPC call if it has PUBLISH_UNPUBLISH_VOLUME
  /// controller capability. This RPC will be called by the CO when it wants to place a workload
  /// that uses the volume onto a node. The Plugin SHOULD perform the work that is necessary for
  /// making the volume available on the given node. The Plugin MUST NOT assume that this RPC
  /// will be executed on the node where the volume will be used.
  ///
  /// This operation MUST be idempotent. If the volume corresponding to the volume_id has
  /// already been published at the node corresponding to the node_id, and is compatible with
  /// the specified volume_capability and readonly flag, the Plugin MUST reply 0 OK.
  ///
  /// If the operation failed or the CO does not know if the operation has failed or not, it
  /// MAY choose to call ControllerPublishVolume again or choose to call ControllerUnpublishVolume.
  ///
  /// The CO MAY call this RPC for publishing a volume to multiple nodes if the volume has
  /// MULTI_NODE capability (i.e., MULTI_NODE_READER_ONLY, MULTI_NODE_SINGLE_WRITER or
  /// MULTI_NODE_MULTI_WRITER).
  #[allow(unused_variables)]
  async fn controller_publish_volume(
    &self,
    request: ControllerPublishVolumeRequest,
  ) -> Result<ControllerPublishVolumeResponse, ControllerPublishVolumeError> {
    unsupported!("ControllerPublishVolume")
  }

  /// Controller Plugin MUST implement this RPC call if it has PUBLISH_UNPUBLISH_VOLUME
  /// controller capability. This RPC is a reverse operation of ControllerPublishVolume.
  /// It MUST be called after all NodeUnstageVolume and NodeUnpublishVolume on the volume
  /// are called and succeed. The Plugin SHOULD perform the work that is necessary for making
  /// the volume ready to be consumed by a different node. The Plugin MUST NOT assume that
  /// this RPC will be executed on the node where the volume was previously used.
  ///
  /// This RPC is typically called by the CO when the workload using the volume is being moved
  /// to a different node, or all the workload using the volume on a node has finished.
  ///
  /// This operation MUST be idempotent. If the volume corresponding to the volume_id is
  /// not attached to the node corresponding to the node_id, the Plugin MUST reply 0 OK.
  /// If the volume corresponding to the volume_id or the node corresponding to node_id cannot
  /// be found by the Plugin and the volume can be safely regarded as ControllerUnpublished
  /// from the node, the plugin SHOULD return 0 OK. If this operation failed, or the CO does
  /// not know if the operation failed or not, it can choose to call ControllerUnpublishVolume
  /// again.
  #[allow(unused_variables)]
  async fn controller_unpublish_volume(
    &self,
    request: ControllerUnpublishVolumeRequest,
  ) -> Result<(), ControllerUnpublishVolumeError> {
    unsupported!("ControllerUnpublishVolume")
  }

  /// A Controller Plugin MUST implement this RPC call. This RPC will be called by the
  /// CO to check if a pre-provisioned volume has all the capabilities that the CO wants.
  /// This RPC call SHALL return confirmed only if all the volume capabilities specified
  /// in the request are supported (see caveat below). This operation MUST be idempotent.
  ///
  /// NOTE: Older plugins will parse but likely not "process" newer fields that MAY be
  /// present in capability-validation messages (and sub-messages) sent by a CO that is
  /// communicating using a newer, backwards-compatible version of the CSI protobufs.
  /// Therefore, the CO SHALL reconcile successful capability-validation responses by
  /// comparing the validated capabilities with those that it had originally requested.
  async fn validate_volume_capabilities(
    &self,
    request: ValidateVolumeCapabilitiesRequest,
  ) -> Result<ValidateVolumeCapabilitiesResponse, ValidateVolumeCapabilitiesError>;

  /// A Controller Plugin MUST implement this RPC call if it has LIST_VOLUMES capability.
  /// The Plugin SHALL return the information about all the volumes that it knows about.
  /// If volumes are created and/or deleted while the CO is concurrently paging through
  /// ListVolumes results then it is possible that the CO MAY either witness duplicate
  /// volumes in the list, not witness existing volumes, or both. The CO SHALL NOT expect
  /// a consistent "view" of all volumes when paging through the volume list via multiple
  /// calls to ListVolumes.
  #[allow(unused_variables)]
  async fn list_volumes(
    &self,
    request: ListVolumesRequest,
  ) -> Result<ListVolumesResponse, ListVolumesError> {
    unsupported!("ListVolumes")
  }

  /// A Controller Plugin MUST implement this RPC call if it has
  /// GET_CAPACITY controller capability. The RPC allows the CO to
  /// query the capacity of the storage pool from which the controller
  /// provisions volumes.
  #[allow(unused_variables)]
  async fn get_capacity(
    &self,
    request: GetCapacityRequest,
  ) -> Result<GetCapacityResponse, GetCapacityError> {
    unsupported!("GetCapacity")
  }

  /// A Controller Plugin MUST implement this RPC call if it has `CREATE_DELETE_SNAPSHOT` controller capability.
  /// This RPC will be called by the CO to create a new snapshot from a source volume on behalf of a user.
  ///
  /// This operation MUST be idempotent.
  /// If a snapshot corresponding to the specified snapshot `name` is successfully cut and ready to use (meaning it MAY be specified as a `volume_content_source` in a `CreateVolumeRequest`), the Plugin MUST reply `0 OK` with the corresponding `CreateSnapshotResponse`.
  ///
  /// If an error occurs before a snapshot is cut, `CreateSnapshot` SHOULD return a corresponding gRPC error code that reflects the error condition.
  ///
  /// For plugins that supports snapshot post processing such as uploading, `CreateSnapshot` SHOULD return `0 OK` and `ready_to_use` SHOULD be set to `false` after the snapshot is cut but still being processed.
  /// CO SHOULD then reissue the same `CreateSnapshotRequest` periodically until boolean `ready_to_use` flips to `true` indicating the snapshot has been "processed" and is ready to use to create new volumes.
  /// If an error occurs during the process, `CreateSnapshot` SHOULD return a corresponding gRPC error code that reflects the error condition.
  ///
  /// A snapshot MAY be used as the source to provision a new volume.
  /// A CreateVolumeRequest message MAY specify an OPTIONAL source snapshot parameter.
  /// Reverting a snapshot, where data in the original volume is erased and replaced with data in the snapshot, is an advanced functionality not every storage system can support and therefore is currently out of scope.
  ///
  /// ### The ready_to_use Parameter
  ///
  /// Some SPs MAY "process" the snapshot after the snapshot is cut, for example, maybe uploading the snapshot somewhere after the snapshot is cut.
  /// The post-cut process MAY be a long process that could take hours.
  /// The CO MAY freeze the application using the source volume before taking the snapshot.
  /// The purpose of `freeze` is to ensure the application data is in consistent state.
  /// When `freeze` is performed, the container is paused and the application is also paused.
  /// When `thaw` is performed, the container and the application start running again.
  /// During the snapshot processing phase, since the snapshot is already cut, a `thaw` operation can be performed so application can start running without waiting for the process to complete.
  /// The `ready_to_use` parameter of the snapshot will become `true` after the process is complete.
  ///
  /// For SPs that do not do additional processing after cut, the `ready_to_use` parameter SHOULD be `true` after the snapshot is cut.
  /// `thaw` can be done when the `ready_to_use` parameter is `true` in this case.
  ///
  /// The `ready_to_use` parameter provides guidance to the CO on when it can "thaw" the application in the process of snapshotting.
  /// If the cloud provider or storage system needs to process the snapshot after the snapshot is cut, the `ready_to_use` parameter returned by CreateSnapshot SHALL be `false`.
  /// CO MAY continue to call CreateSnapshot while waiting for the process to complete until `ready_to_use` becomes `true`.
  /// Note that CreateSnapshot no longer blocks after the snapshot is cut.
  ///
  /// A gRPC error code SHALL be returned if an error occurs during any stage of the snapshotting process.
  /// A CO SHOULD explicitly delete snapshots when an error occurs.
  ///
  /// Based on this information, CO can issue repeated (idemponent) calls to CreateSnapshot, monitor the response, and make decisions.
  /// Note that CreateSnapshot is a synchronous call and it MUST block until the snapshot is cut.
  #[allow(unused_variables)]
  async fn create_snapshot(
    &self,
    request: CreateSnapshotRequest,
  ) -> Result<Snapshot, CreateSnapshotError> {
    unsupported!("CreateSnapshot")
  }

  /// A Controller Plugin MUST implement this RPC call if it has `CREATE_DELETE_SNAPSHOT` capability.
  /// This RPC will be called by the CO to delete a snapshot.
  ///
  /// This operation MUST be idempotent.
  /// If a snapshot corresponding to the specified `snapshot_id` does not exist or the artifacts associated with the snapshot do not exist anymore, the Plugin MUST reply `0 OK`.
  #[allow(unused_variables)]
  async fn delete_snapshot(
    &self,
    request: DeleteSnapshotRequest,
  ) -> Result<(), DeleteSnapshotError> {
    unsupported!("DeleteSnapshot")
  }

  /// A Controller Plugin MUST implement this RPC call if it has `LIST_SNAPSHOTS` capability.
  ///
  /// The Plugin SHALL return the information about all snapshots on the storage system within
  /// the given parameters regardless of how they were created.
  ///
  /// `ListSnapshots` SHALL NOT list a snapshot that is being created but has not been cut
  /// successfully yet.
  ///
  /// If snapshots are created and/or deleted while the CO is concurrently paging through
  /// `ListSnapshots` results then it is possible that the CO MAY either witness duplicate
  /// snapshots in the list, not witness existing snapshots, or both.
  ///
  /// The CO SHALL NOT expect a consistent "view" of all snapshots when paging through
  /// the snapshot list via multiple calls to `ListSnapshots`.
  #[allow(unused_variables)]
  async fn list_snapshots(
    &self,
    request: ListSnapshotsRequest,
  ) -> Result<ListSnapshotsResponse, ListSnapshotsError> {
    unsupported!("ListSnapshots")
  }

  /// A Controller plugin MUST implement this RPC call if plugin has `EXPAND_VOLUME`
  /// controller capability. This RPC allows the CO to expand the size of a volume.
  ///
  /// This operation MUST be idempotent. If a volume corresponding to the specified
  /// volume ID is already larger than or equal to the target capacity of the expansion
  /// request, the plugin SHOULD reply 0 OK.
  ///
  /// This call MAY be made by the CO during any time in the lifecycle of the volume
  /// after creation if plugin has `VolumeExpansion.ONLINE` capability. If plugin has
  /// `EXPAND_VOLUME` node capability, then `NodeExpandVolume` MUST be called after
  /// successful `ControllerExpandVolume` and `node_expansion_required` in
  /// `ControllerExpandVolumeResponse` is `true`.
  ///
  /// If specified, the `volume_capability` in `ControllerExpandVolumeRequest` should
  /// be same as what CO would pass in `ControllerPublishVolumeRequest`.
  ///
  /// If the plugin has only `VolumeExpansion.OFFLINE` expansion capability and volume
  /// is currently published or available on a node then `ControllerExpandVolume` MUST
  /// be called ONLY after either:
  /// - The plugin has controller `PUBLISH_UNPUBLISH_VOLUME` capability and
  ///   `ControllerUnpublishVolume` has been invoked successfully. OR ELSE
  /// - The plugin does NOT have controller `PUBLISH_UNPUBLISH_VOLUME` capability,
  ///   the plugin has node `STAGE_UNSTAGE_VOLUME` capability, and `NodeUnstageVolume`
  ///   has been completed successfully. OR ELSE
  /// - The plugin does NOT have controller `PUBLISH_UNPUBLISH_VOLUME` capability, nor
  ///   node `STAGE_UNSTAGE_VOLUME` capability, and `NodeUnpublishVolume` has completed
  ///   successfully.
  #[allow(unused_variables)]
  async fn controller_expand_volume(
    &self,
    request: ControllerExpandVolumeRequest,
  ) -> Result<ControllerExpandVolumeResponse, ControllerExpandVolumeError> {
    unsupported!("ControllerExpandVolume")
  }

  /// **ALPHA FEATURE**
  ///
  /// This optional RPC MAY be called by the CO to fetch current information about
  /// a volume.
  ///
  /// A Controller Plugin MUST implement this `ControllerGetVolume` RPC call if it
  /// has `GET_VOLUME` capability.
  ///
  /// A Controller Plugin MUST provide a non-empty `volume_condition` field in
  /// `ControllerGetVolumeResponse` if it has `VOLUME_CONDITION` capability.
  ///
  /// `ControllerGetVolumeResponse` should contain current information of a volume
  /// if it exists. If the volume does not exist any more, `ControllerGetVolume`
  /// should return gRPC error code `NOT_FOUND`.
  #[allow(unused_variables)]
  async fn controller_get_volume(
    &self,
    request: ControllerGetVolumeRequest,
  ) -> Result<ControllerGetVolumeResponse, ControllerGetVolumeError> {
    unsupported!("ControllerGetVolume")
  }
}

struct Controller<T: ControllerService>(Arc<T>);

#[async_trait]
impl<T: ControllerService> proto::identity_server::Identity for Controller<T> {
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
      name: self.0.name().record_field("name").into(),
      vendor_version: self.0.version().record_field("vendor_version").into(),
      manifest: self.0.manifest().record_field("manifest").clone(),
    };

    Ok(tonic::Response::new(response))
  }

  // TODO: Instrument response
  #[instrument(name = "identity.get_plugin_capabilities", skip(self, _request))]
  async fn get_plugin_capabilities(
    &self,
    _request: tonic::Request<proto::GetPluginCapabilitiesRequest>,
  ) -> Result<tonic::Response<proto::GetPluginCapabilitiesResponse>, tonic::Status> {
    let mut response = plugin::get_capabilities(&*self.0);
    response.capabilities.push(proto::PluginCapability {
      r#type: Some(proto::plugin_capability::Type::Service(
        proto::plugin_capability::Service {
          r#type: proto::plugin_capability::service::Type::ControllerService.into(),
        },
      )),
    });

    Ok(tonic::Response::new(response))
  }

  #[instrument(name = "identity.probe", skip(self, _request), fields(ready))]
  async fn probe(
    &self,
    _request: tonic::Request<proto::ProbeRequest>,
  ) -> Result<tonic::Response<proto::ProbeResponse>, tonic::Status> {
    let response = proto::ProbeResponse {
      ready: Some(self.0.ready().record_field("ready")),
    };

    Ok(tonic::Response::new(response))
  }
}

#[async_trait]
impl<T: ControllerService> proto::controller_server::Controller for Controller<T> {
  #[instrument(
    name = "controller.create_volume",
    skip(self, request),
    fields(request, response)
  )]
  async fn create_volume(
    &self,
    request: tonic::Request<proto::CreateVolumeRequest>,
  ) -> Result<tonic::Response<proto::CreateVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .create_volume(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.delete_volume",
    skip(self, request),
    fields(request)
  )]
  async fn delete_volume(
    &self,
    request: tonic::Request<proto::DeleteVolumeRequest>,
  ) -> Result<tonic::Response<proto::DeleteVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.delete_volume(request).await?;
    let response = proto::DeleteVolumeResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.controller_publish_volume",
    skip(self, request),
    fields(request, response)
  )]
  async fn controller_publish_volume(
    &self,
    request: tonic::Request<proto::ControllerPublishVolumeRequest>,
  ) -> Result<tonic::Response<proto::ControllerPublishVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .controller_publish_volume(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.controller_unpublish_volume",
    skip(self, request),
    fields(request)
  )]
  async fn controller_unpublish_volume(
    &self,
    request: tonic::Request<proto::ControllerUnpublishVolumeRequest>,
  ) -> Result<tonic::Response<proto::ControllerUnpublishVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.controller_unpublish_volume(request).await?;
    let response = proto::ControllerUnpublishVolumeResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.validate_volume_capabilities",
    skip(self, request),
    fields(request, response)
  )]
  async fn validate_volume_capabilities(
    &self,
    request: tonic::Request<proto::ValidateVolumeCapabilitiesRequest>,
  ) -> Result<tonic::Response<proto::ValidateVolumeCapabilitiesResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .validate_volume_capabilities(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.list_volumes",
    skip(self, request),
    fields(request, response)
  )]
  async fn list_volumes(
    &self,
    request: tonic::Request<proto::ListVolumesRequest>,
  ) -> Result<tonic::Response<proto::ListVolumesResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .list_volumes(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.get_capacity",
    skip(self, request),
    fields(request, response)
  )]
  async fn get_capacity(
    &self,
    request: tonic::Request<proto::GetCapacityRequest>,
  ) -> Result<tonic::Response<proto::GetCapacityResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .get_capacity(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.controller_get_capabilities",
    skip(self),
    fields(response)
  )]
  async fn controller_get_capabilities(
    &self,
    _: tonic::Request<proto::ControllerGetCapabilitiesRequest>,
  ) -> Result<tonic::Response<proto::ControllerGetCapabilitiesResponse>, tonic::Status> {
    let response = self.0.capabilities().record_response().try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.create_snapshot",
    skip(self, request),
    fields(request, response)
  )]
  async fn create_snapshot(
    &self,
    request: tonic::Request<proto::CreateSnapshotRequest>,
  ) -> Result<tonic::Response<proto::CreateSnapshotResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .create_snapshot(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.delete_snapshot",
    skip(self, request),
    fields(request)
  )]
  async fn delete_snapshot(
    &self,
    request: tonic::Request<proto::DeleteSnapshotRequest>,
  ) -> Result<tonic::Response<proto::DeleteSnapshotResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    self.0.delete_snapshot(request).await?;
    let response = proto::DeleteSnapshotResponse {};
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.list_snapshots",
    skip(self, request),
    fields(request, response)
  )]
  async fn list_snapshots(
    &self,
    request: tonic::Request<proto::ListSnapshotsRequest>,
  ) -> Result<tonic::Response<proto::ListSnapshotsResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .list_snapshots(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.controller_expand_volume",
    skip(self, request),
    fields(request, response)
  )]
  async fn controller_expand_volume(
    &self,
    request: tonic::Request<proto::ControllerExpandVolumeRequest>,
  ) -> Result<tonic::Response<proto::ControllerExpandVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .controller_expand_volume(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }

  #[instrument(
    name = "controller.controller_get_volume",
    skip(self, request),
    fields(request, response)
  )]
  async fn controller_get_volume(
    &self,
    request: tonic::Request<proto::ControllerGetVolumeRequest>,
  ) -> Result<tonic::Response<proto::ControllerGetVolumeResponse>, tonic::Status> {
    let request = record_request(request.into_inner().try_into()?);
    let response = self
      .0
      .controller_get_volume(request)
      .await?
      .record_response()
      .try_into()?;
    Ok(tonic::Response::new(response))
  }
}
