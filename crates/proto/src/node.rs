use crate::{plugin, proto, IdentityService};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::instrument;

#[async_trait]
pub trait NodeService: IdentityService {}

struct Node<T: NodeService>(Arc<T>);

#[async_trait]
impl<T: NodeService> proto::identity_server::Identity for Node<T> {
  #[instrument(
    name = "Identity.get_plugin_info",
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
  #[instrument(name = "Identity.get_plugin_capabilities", skip(self, _request))]
  async fn get_plugin_capabilities(
    &self,
    _request: tonic::Request<proto::GetPluginCapabilitiesRequest>,
  ) -> Result<tonic::Response<proto::GetPluginCapabilitiesResponse>, tonic::Status> {
    let response = plugin::get_capabilities(&*self.0);

    Ok(tonic::Response::new(response))
  }

  #[instrument(name = "Identity.probe", skip(self, _request), fields(ready))]
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
