/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPluginInfoRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPluginInfoResponse {
  /// The name MUST follow domain name notation format
  /// (https://tools.ietf.org/html/rfc1035#section-2.3.1). It SHOULD
  /// include the plugin's host company name and the plugin name,
  /// to minimize the possibility of collisions. It MUST be 63
  /// characters or less, beginning and ending with an alphanumeric
  /// character ([a-z0-9A-Z]) with dashes (-), dots (.), and
  /// alphanumerics between. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  /// This field is REQUIRED. Value of this field is opaque to the CO.
  #[prost(string, tag = "2")]
  pub vendor_version: ::prost::alloc::string::String,
  /// This field is OPTIONAL. Values are opaque to the CO.
  #[prost(map = "string, string", tag = "3")]
  pub manifest:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPluginCapabilitiesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPluginCapabilitiesResponse {
  /// All the capabilities that the controller service supports. This
  /// field is OPTIONAL.
  #[prost(message, repeated, tag = "1")]
  pub capabilities: ::prost::alloc::vec::Vec<PluginCapability>,
}
/// Specifies a capability of the plugin.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PluginCapability {
  #[prost(oneof = "plugin_capability::Type", tags = "1, 2")]
  pub r#type: ::core::option::Option<plugin_capability::Type>,
}
/// Nested message and enum types in `PluginCapability`.
pub mod plugin_capability {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Service {
    #[prost(enumeration = "service::Type", tag = "1")]
    pub r#type: i32,
  }
  /// Nested message and enum types in `Service`.
  pub mod service {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
      Unknown = 0,
      /// CONTROLLER_SERVICE indicates that the Plugin provides RPCs for
      /// the ControllerService. Plugins SHOULD provide this capability.
      /// In rare cases certain plugins MAY wish to omit the
      /// ControllerService entirely from their implementation, but such
      /// SHOULD NOT be the common case.
      /// The presence of this capability determines whether the CO will
      /// attempt to invoke the REQUIRED ControllerService RPCs, as well
      /// as specific RPCs as indicated by ControllerGetCapabilities.
      ControllerService = 1,
      /// VOLUME_ACCESSIBILITY_CONSTRAINTS indicates that the volumes for
      /// this plugin MAY NOT be equally accessible by all nodes in the
      /// cluster. The CO MUST use the topology information returned by
      /// CreateVolumeRequest along with the topology information
      /// returned by NodeGetInfo to ensure that a given volume is
      /// accessible from a given node when scheduling workloads.
      VolumeAccessibilityConstraints = 2,
    }
  }
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct VolumeExpansion {
    #[prost(enumeration = "volume_expansion::Type", tag = "1")]
    pub r#type: i32,
  }
  /// Nested message and enum types in `VolumeExpansion`.
  pub mod volume_expansion {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
      Unknown = 0,
      /// ONLINE indicates that volumes may be expanded when published to
      /// a node. When a Plugin implements this capability it MUST
      /// implement either the EXPAND_VOLUME controller capability or the
      /// EXPAND_VOLUME node capability or both. When a plugin supports
      /// ONLINE volume expansion and also has the EXPAND_VOLUME
      /// controller capability then the plugin MUST support expansion of
      /// volumes currently published and available on a node. When a
      /// plugin supports ONLINE volume expansion and also has the
      /// EXPAND_VOLUME node capability then the plugin MAY support
      /// expansion of node-published volume via NodeExpandVolume.
      ///
      /// Example 1: Given a shared filesystem volume (e.g. GlusterFs),
      ///   the Plugin may set the ONLINE volume expansion capability and
      ///   implement ControllerExpandVolume but not NodeExpandVolume.
      ///
      /// Example 2: Given a block storage volume type (e.g. EBS), the
      ///   Plugin may set the ONLINE volume expansion capability and
      ///   implement both ControllerExpandVolume and NodeExpandVolume.
      ///
      /// Example 3: Given a Plugin that supports volume expansion only
      ///   upon a node, the Plugin may set the ONLINE volume
      ///   expansion capability and implement NodeExpandVolume but not
      ///   ControllerExpandVolume.
      Online = 1,
      /// OFFLINE indicates that volumes currently published and
      /// available on a node SHALL NOT be expanded via
      /// ControllerExpandVolume. When a plugin supports OFFLINE volume
      /// expansion it MUST implement either the EXPAND_VOLUME controller
      /// capability or both the EXPAND_VOLUME controller capability and
      /// the EXPAND_VOLUME node capability.
      ///
      /// Example 1: Given a block storage volume type (e.g. Azure Disk)
      ///   that does not support expansion of "node-attached" (i.e.
      ///   controller-published) volumes, the Plugin may indicate
      ///   OFFLINE volume expansion support and implement both
      ///   ControllerExpandVolume and NodeExpandVolume.
      Offline = 2,
    }
  }
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Type {
    /// Service that the plugin supports.
    #[prost(message, tag = "1")]
    Service(Service),
    #[prost(message, tag = "2")]
    VolumeExpansion(VolumeExpansion),
  }
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProbeRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProbeResponse {
  /// Readiness allows a plugin to report its initialization status back
  /// to the CO. Initialization for some plugins MAY be time consuming
  /// and it is important for a CO to distinguish between the following
  /// cases:
  ///
  /// 1) The plugin is in an unhealthy state and MAY need restarting. In
  ///    this case a gRPC error code SHALL be returned.
  /// 2) The plugin is still initializing, but is otherwise perfectly
  ///    healthy. In this case a successful response SHALL be returned
  ///    with a readiness value of `false`. Calls to the plugin's
  ///    Controller and/or Node services MAY fail due to an incomplete
  ///    initialization state.
  /// 3) The plugin has finished initializing and is ready to service
  ///    calls to its Controller and/or Node services. A successful
  ///    response is returned with a readiness value of `true`.
  ///
  /// This field is OPTIONAL. If not present, the caller SHALL assume
  /// that the plugin is in a ready state and is accepting calls to its
  /// Controller and/or Node services (according to the plugin's reported
  /// capabilities).
  #[prost(message, optional, tag = "1")]
  pub ready: ::core::option::Option<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateVolumeRequest {
  /// The suggested name for the storage space. This field is REQUIRED.
  /// It serves two purposes:
  /// 1) Idempotency - This name is generated by the CO to achieve
  ///    idempotency.  The Plugin SHOULD ensure that multiple
  ///    `CreateVolume` calls for the same name do not result in more
  ///    than one piece of storage provisioned corresponding to that
  ///    name. If a Plugin is unable to enforce idempotency, the CO's
  ///    error recovery logic could result in multiple (unused) volumes
  ///    being provisioned.
  ///    In the case of error, the CO MUST handle the gRPC error codes
  ///    per the recovery behavior defined in the "CreateVolume Errors"
  ///    section below.
  ///    The CO is responsible for cleaning up volumes it provisioned
  ///    that it no longer needs. If the CO is uncertain whether a volume
  ///    was provisioned or not when a `CreateVolume` call fails, the CO
  ///    MAY call `CreateVolume` again, with the same name, to ensure the
  ///    volume exists and to retrieve the volume's `volume_id` (unless
  ///    otherwise prohibited by "CreateVolume Errors").
  /// 2) Suggested name - Some storage systems allow callers to specify
  ///    an identifier by which to refer to the newly provisioned
  ///    storage. If a storage system supports this, it can optionally
  ///    use this name as the identifier for the new volume.
  /// Any Unicode string that conforms to the length limit is allowed
  /// except those containing the following banned characters:
  /// U+0000-U+0008, U+000B, U+000C, U+000E-U+001F, U+007F-U+009F.
  /// (These are control characters other than commonly used whitespace.)
  #[prost(string, tag = "1")]
  pub name: ::prost::alloc::string::String,
  /// This field is OPTIONAL. This allows the CO to specify the capacity
  /// requirement of the volume to be provisioned. If not specified, the
  /// Plugin MAY choose an implementation-defined capacity range. If
  /// specified it MUST always be honored, even when creating volumes
  /// from a source; which MAY force some backends to internally extend
  /// the volume after creating it.
  #[prost(message, optional, tag = "2")]
  pub capacity_range: ::core::option::Option<CapacityRange>,
  /// The capabilities that the provisioned volume MUST have. SP MUST
  /// provision a volume that will satisfy ALL of the capabilities
  /// specified in this list. Otherwise SP MUST return the appropriate
  /// gRPC error code.
  /// The Plugin MUST assume that the CO MAY use the provisioned volume
  /// with ANY of the capabilities specified in this list.
  /// For example, a CO MAY specify two volume capabilities: one with
  /// access mode SINGLE_NODE_WRITER and another with access mode
  /// MULTI_NODE_READER_ONLY. In this case, the SP MUST verify that the
  /// provisioned volume can be used in either mode.
  /// This also enables the CO to do early validation: If ANY of the
  /// specified volume capabilities are not supported by the SP, the call
  /// MUST return the appropriate gRPC error code.
  /// This field is REQUIRED.
  #[prost(message, repeated, tag = "3")]
  pub volume_capabilities: ::prost::alloc::vec::Vec<VolumeCapability>,
  /// Plugin specific parameters passed in as opaque key-value pairs.
  /// This field is OPTIONAL. The Plugin is responsible for parsing and
  /// validating these parameters. COs will treat these as opaque.
  #[prost(map = "string, string", tag = "4")]
  pub parameters:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// Secrets required by plugin to complete volume creation request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "5")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// If specified, the new volume will be pre-populated with data from
  /// this source. This field is OPTIONAL.
  #[prost(message, optional, tag = "6")]
  pub volume_content_source: ::core::option::Option<VolumeContentSource>,
  /// Specifies where (regions, zones, racks, etc.) the provisioned
  /// volume MUST be accessible from.
  /// An SP SHALL advertise the requirements for topological
  /// accessibility information in documentation. COs SHALL only specify
  /// topological accessibility information supported by the SP.
  /// This field is OPTIONAL.
  /// This field SHALL NOT be specified unless the SP has the
  /// VOLUME_ACCESSIBILITY_CONSTRAINTS plugin capability.
  /// If this field is not specified and the SP has the
  /// VOLUME_ACCESSIBILITY_CONSTRAINTS plugin capability, the SP MAY
  /// choose where the provisioned volume is accessible from.
  #[prost(message, optional, tag = "7")]
  pub accessibility_requirements: ::core::option::Option<TopologyRequirement>,
}
/// Specifies what source the volume will be created from. One of the
/// type fields MUST be specified.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VolumeContentSource {
  #[prost(oneof = "volume_content_source::Type", tags = "1, 2")]
  pub r#type: ::core::option::Option<volume_content_source::Type>,
}
/// Nested message and enum types in `VolumeContentSource`.
pub mod volume_content_source {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct SnapshotSource {
    /// Contains identity information for the existing source snapshot.
    /// This field is REQUIRED. Plugin is REQUIRED to support creating
    /// volume from snapshot if it supports the capability
    /// CREATE_DELETE_SNAPSHOT.
    #[prost(string, tag = "1")]
    pub snapshot_id: ::prost::alloc::string::String,
  }
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct VolumeSource {
    /// Contains identity information for the existing source volume.
    /// This field is REQUIRED. Plugins reporting CLONE_VOLUME
    /// capability MUST support creating a volume from another volume.
    #[prost(string, tag = "1")]
    pub volume_id: ::prost::alloc::string::String,
  }
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Type {
    #[prost(message, tag = "1")]
    Snapshot(SnapshotSource),
    #[prost(message, tag = "2")]
    Volume(VolumeSource),
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateVolumeResponse {
  /// Contains all attributes of the newly created volume that are
  /// relevant to the CO along with information required by the Plugin
  /// to uniquely identify the volume. This field is REQUIRED.
  #[prost(message, optional, tag = "1")]
  pub volume: ::core::option::Option<Volume>,
}
/// Specify a capability of a volume.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VolumeCapability {
  /// This is a REQUIRED field.
  #[prost(message, optional, tag = "3")]
  pub access_mode: ::core::option::Option<volume_capability::AccessMode>,
  /// Specifies what API the volume will be accessed using. One of the
  /// following fields MUST be specified.
  #[prost(oneof = "volume_capability::AccessType", tags = "1, 2")]
  pub access_type: ::core::option::Option<volume_capability::AccessType>,
}
/// Nested message and enum types in `VolumeCapability`.
pub mod volume_capability {
  /// Indicate that the volume will be accessed via the block device API.
  ///
  /// Intentionally empty, for now.
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct BlockVolume {}
  /// Indicate that the volume will be accessed via the filesystem API.
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct MountVolume {
    /// The filesystem type. This field is OPTIONAL.
    /// An empty string is equal to an unspecified field value.
    #[prost(string, tag = "1")]
    pub fs_type: ::prost::alloc::string::String,
    /// The mount options that can be used for the volume. This field is
    /// OPTIONAL. `mount_flags` MAY contain sensitive information.
    /// Therefore, the CO and the Plugin MUST NOT leak this information
    /// to untrusted entities. The total size of this repeated field
    /// SHALL NOT exceed 4 KiB.
    #[prost(string, repeated, tag = "2")]
    pub mount_flags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
  }
  /// Specify how a volume can be accessed.
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct AccessMode {
    /// This field is REQUIRED.
    #[prost(enumeration = "access_mode::Mode", tag = "1")]
    pub mode: i32,
  }
  /// Nested message and enum types in `AccessMode`.
  pub mod access_mode {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Mode {
      Unknown = 0,
      /// Can only be published once as read/write on a single node, at
      /// any given time.
      SingleNodeWriter = 1,
      /// Can only be published once as readonly on a single node, at
      /// any given time.
      SingleNodeReaderOnly = 2,
      /// Can be published as readonly at multiple nodes simultaneously.
      MultiNodeReaderOnly = 3,
      /// Can be published at multiple nodes simultaneously. Only one of
      /// the node can be used as read/write. The rest will be readonly.
      MultiNodeSingleWriter = 4,
      /// Can be published as read/write at multiple nodes
      /// simultaneously.
      MultiNodeMultiWriter = 5,
    }
  }
  /// Specifies what API the volume will be accessed using. One of the
  /// following fields MUST be specified.
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum AccessType {
    #[prost(message, tag = "1")]
    Block(BlockVolume),
    #[prost(message, tag = "2")]
    Mount(MountVolume),
  }
}
/// The capacity of the storage space in bytes. To specify an exact size,
/// `required_bytes` and `limit_bytes` SHALL be set to the same value. At
/// least one of the these fields MUST be specified.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CapacityRange {
  /// Volume MUST be at least this big. This field is OPTIONAL.
  /// A value of 0 is equal to an unspecified field value.
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "1")]
  pub required_bytes: i64,
  /// Volume MUST not be bigger than this. This field is OPTIONAL.
  /// A value of 0 is equal to an unspecified field value.
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "2")]
  pub limit_bytes: i64,
}
/// Information about a specific volume.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Volume {
  /// The capacity of the volume in bytes. This field is OPTIONAL. If not
  /// set (value of 0), it indicates that the capacity of the volume is
  /// unknown (e.g., NFS share).
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "1")]
  pub capacity_bytes: i64,
  /// The identifier for this volume, generated by the plugin.
  /// This field is REQUIRED.
  /// This field MUST contain enough information to uniquely identify
  /// this specific volume vs all other volumes supported by this plugin.
  /// This field SHALL be used by the CO in subsequent calls to refer to
  /// this volume.
  /// The SP is NOT responsible for global uniqueness of volume_id across
  /// multiple SPs.
  #[prost(string, tag = "2")]
  pub volume_id: ::prost::alloc::string::String,
  /// Opaque static properties of the volume. SP MAY use this field to
  /// ensure subsequent volume validation and publishing calls have
  /// contextual information.
  /// The contents of this field SHALL be opaque to a CO.
  /// The contents of this field SHALL NOT be mutable.
  /// The contents of this field SHALL be safe for the CO to cache.
  /// The contents of this field SHOULD NOT contain sensitive
  /// information.
  /// The contents of this field SHOULD NOT be used for uniquely
  /// identifying a volume. The `volume_id` alone SHOULD be sufficient to
  /// identify the volume.
  /// A volume uniquely identified by `volume_id` SHALL always report the
  /// same volume_context.
  /// This field is OPTIONAL and when present MUST be passed to volume
  /// validation and publishing calls.
  #[prost(map = "string, string", tag = "3")]
  pub volume_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// If specified, indicates that the volume is not empty and is
  /// pre-populated with data from the specified source.
  /// This field is OPTIONAL.
  #[prost(message, optional, tag = "4")]
  pub content_source: ::core::option::Option<VolumeContentSource>,
  /// Specifies where (regions, zones, racks, etc.) the provisioned
  /// volume is accessible from.
  /// A plugin that returns this field MUST also set the
  /// VOLUME_ACCESSIBILITY_CONSTRAINTS plugin capability.
  /// An SP MAY specify multiple topologies to indicate the volume is
  /// accessible from multiple locations.
  /// COs MAY use this information along with the topology information
  /// returned by NodeGetInfo to ensure that a given volume is accessible
  /// from a given node when scheduling workloads.
  /// This field is OPTIONAL. If it is not specified, the CO MAY assume
  /// the volume is equally accessible from all nodes in the cluster and
  /// MAY schedule workloads referencing the volume on any available
  /// node.
  ///
  /// Example 1:
  ///   accessible_topology = {"region": "R1", "zone": "Z2"}
  /// Indicates a volume accessible only from the "region" "R1" and the
  /// "zone" "Z2".
  ///
  /// Example 2:
  ///   accessible_topology =
  ///     {"region": "R1", "zone": "Z2"},
  ///     {"region": "R1", "zone": "Z3"}
  /// Indicates a volume accessible from both "zone" "Z2" and "zone" "Z3"
  /// in the "region" "R1".
  #[prost(message, repeated, tag = "5")]
  pub accessible_topology: ::prost::alloc::vec::Vec<Topology>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopologyRequirement {
  /// Specifies the list of topologies the provisioned volume MUST be
  /// accessible from.
  /// This field is OPTIONAL. If TopologyRequirement is specified either
  /// requisite or preferred or both MUST be specified.
  ///
  /// If requisite is specified, the provisioned volume MUST be
  /// accessible from at least one of the requisite topologies.
  ///
  /// Given
  ///   x = number of topologies provisioned volume is accessible from
  ///   n = number of requisite topologies
  /// The CO MUST ensure n >= 1. The SP MUST ensure x >= 1
  /// If x==n, then the SP MUST make the provisioned volume available to
  /// all topologies from the list of requisite topologies. If it is
  /// unable to do so, the SP MUST fail the CreateVolume call.
  /// For example, if a volume should be accessible from a single zone,
  /// and requisite =
  ///   {"region": "R1", "zone": "Z2"}
  /// then the provisioned volume MUST be accessible from the "region"
  /// "R1" and the "zone" "Z2".
  /// Similarly, if a volume should be accessible from two zones, and
  /// requisite =
  ///   {"region": "R1", "zone": "Z2"},
  ///   {"region": "R1", "zone": "Z3"}
  /// then the provisioned volume MUST be accessible from the "region"
  /// "R1" and both "zone" "Z2" and "zone" "Z3".
  ///
  /// If x<n, then the SP SHALL choose x unique topologies from the list
  /// of requisite topologies. If it is unable to do so, the SP MUST fail
  /// the CreateVolume call.
  /// For example, if a volume should be accessible from a single zone,
  /// and requisite =
  ///   {"region": "R1", "zone": "Z2"},
  ///   {"region": "R1", "zone": "Z3"}
  /// then the SP may choose to make the provisioned volume available in
  /// either the "zone" "Z2" or the "zone" "Z3" in the "region" "R1".
  /// Similarly, if a volume should be accessible from two zones, and
  /// requisite =
  ///   {"region": "R1", "zone": "Z2"},
  ///   {"region": "R1", "zone": "Z3"},
  ///   {"region": "R1", "zone": "Z4"}
  /// then the provisioned volume MUST be accessible from any combination
  /// of two unique topologies: e.g. "R1/Z2" and "R1/Z3", or "R1/Z2" and
  ///  "R1/Z4", or "R1/Z3" and "R1/Z4".
  ///
  /// If x>n, then the SP MUST make the provisioned volume available from
  /// all topologies from the list of requisite topologies and MAY choose
  /// the remaining x-n unique topologies from the list of all possible
  /// topologies. If it is unable to do so, the SP MUST fail the
  /// CreateVolume call.
  /// For example, if a volume should be accessible from two zones, and
  /// requisite =
  ///   {"region": "R1", "zone": "Z2"}
  /// then the provisioned volume MUST be accessible from the "region"
  /// "R1" and the "zone" "Z2" and the SP may select the second zone
  /// independently, e.g. "R1/Z4".
  #[prost(message, repeated, tag = "1")]
  pub requisite: ::prost::alloc::vec::Vec<Topology>,
  /// Specifies the list of topologies the CO would prefer the volume to
  /// be provisioned in.
  ///
  /// This field is OPTIONAL. If TopologyRequirement is specified either
  /// requisite or preferred or both MUST be specified.
  ///
  /// An SP MUST attempt to make the provisioned volume available using
  /// the preferred topologies in order from first to last.
  ///
  /// If requisite is specified, all topologies in preferred list MUST
  /// also be present in the list of requisite topologies.
  ///
  /// If the SP is unable to to make the provisioned volume available
  /// from any of the preferred topologies, the SP MAY choose a topology
  /// from the list of requisite topologies.
  /// If the list of requisite topologies is not specified, then the SP
  /// MAY choose from the list of all possible topologies.
  /// If the list of requisite topologies is specified and the SP is
  /// unable to to make the provisioned volume available from any of the
  /// requisite topologies it MUST fail the CreateVolume call.
  ///
  /// Example 1:
  /// Given a volume should be accessible from a single zone, and
  /// requisite =
  ///   {"region": "R1", "zone": "Z2"},
  ///   {"region": "R1", "zone": "Z3"}
  /// preferred =
  ///   {"region": "R1", "zone": "Z3"}
  /// then the the SP SHOULD first attempt to make the provisioned volume
  /// available from "zone" "Z3" in the "region" "R1" and fall back to
  /// "zone" "Z2" in the "region" "R1" if that is not possible.
  ///
  /// Example 2:
  /// Given a volume should be accessible from a single zone, and
  /// requisite =
  ///   {"region": "R1", "zone": "Z2"},
  ///   {"region": "R1", "zone": "Z3"},
  ///   {"region": "R1", "zone": "Z4"},
  ///   {"region": "R1", "zone": "Z5"}
  /// preferred =
  ///   {"region": "R1", "zone": "Z4"},
  ///   {"region": "R1", "zone": "Z2"}
  /// then the the SP SHOULD first attempt to make the provisioned volume
  /// accessible from "zone" "Z4" in the "region" "R1" and fall back to
  /// "zone" "Z2" in the "region" "R1" if that is not possible. If that
  /// is not possible, the SP may choose between either the "zone"
  /// "Z3" or "Z5" in the "region" "R1".
  ///
  /// Example 3:
  /// Given a volume should be accessible from TWO zones (because an
  /// opaque parameter in CreateVolumeRequest, for example, specifies
  /// the volume is accessible from two zones, aka synchronously
  /// replicated), and
  /// requisite =
  ///   {"region": "R1", "zone": "Z2"},
  ///   {"region": "R1", "zone": "Z3"},
  ///   {"region": "R1", "zone": "Z4"},
  ///   {"region": "R1", "zone": "Z5"}
  /// preferred =
  ///   {"region": "R1", "zone": "Z5"},
  ///   {"region": "R1", "zone": "Z3"}
  /// then the the SP SHOULD first attempt to make the provisioned volume
  /// accessible from the combination of the two "zones" "Z5" and "Z3" in
  /// the "region" "R1". If that's not possible, it should fall back to
  /// a combination of "Z5" and other possibilities from the list of
  /// requisite. If that's not possible, it should fall back  to a
  /// combination of "Z3" and other possibilities from the list of
  /// requisite. If that's not possible, it should fall back  to a
  /// combination of other possibilities from the list of requisite.
  #[prost(message, repeated, tag = "2")]
  pub preferred: ::prost::alloc::vec::Vec<Topology>,
}
/// Topology is a map of topological domains to topological segments.
/// A topological domain is a sub-division of a cluster, like "region",
/// "zone", "rack", etc.
/// A topological segment is a specific instance of a topological domain,
/// like "zone3", "rack3", etc.
/// For example {"com.company/zone": "Z1", "com.company/rack": "R3"}
/// Valid keys have two segments: an OPTIONAL prefix and name, separated
/// by a slash (/), for example: "com.company.example/zone".
/// The key name segment is REQUIRED. The prefix is OPTIONAL.
/// The key name MUST be 63 characters or less, begin and end with an
/// alphanumeric character ([a-z0-9A-Z]), and contain only dashes (-),
/// underscores (_), dots (.), or alphanumerics in between, for example
/// "zone".
/// The key prefix MUST be 63 characters or less, begin and end with a
/// lower-case alphanumeric character ([a-z0-9]), contain only
/// dashes (-), dots (.), or lower-case alphanumerics in between, and
/// follow domain name notation format
/// (https://tools.ietf.org/html/rfc1035#section-2.3.1).
/// The key prefix SHOULD include the plugin's host company name and/or
/// the plugin name, to minimize the possibility of collisions with keys
/// from other plugins.
/// If a key prefix is specified, it MUST be identical across all
/// topology keys returned by the SP (across all RPCs).
/// Keys MUST be case-insensitive. Meaning the keys "Zone" and "zone"
/// MUST not both exist.
/// Each value (topological segment) MUST contain 1 or more strings.
/// Each string MUST be 63 characters or less and begin and end with an
/// alphanumeric character with '-', '_', '.', or alphanumerics in
/// between.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Topology {
  #[prost(map = "string, string", tag = "1")]
  pub segments:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteVolumeRequest {
  /// The ID of the volume to be deprovisioned.
  /// This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// Secrets required by plugin to complete volume deletion request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "2")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteVolumeResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerPublishVolumeRequest {
  /// The ID of the volume to be used on a node.
  /// This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The ID of the node. This field is REQUIRED. The CO SHALL set this
  /// field to match the node ID returned by `NodeGetInfo`.
  #[prost(string, tag = "2")]
  pub node_id: ::prost::alloc::string::String,
  /// Volume capability describing how the CO intends to use this volume.
  /// SP MUST ensure the CO can use the published volume as described.
  /// Otherwise SP MUST return the appropriate gRPC error code.
  /// This is a REQUIRED field.
  #[prost(message, optional, tag = "3")]
  pub volume_capability: ::core::option::Option<VolumeCapability>,
  /// Indicates SP MUST publish the volume in readonly mode.
  /// CO MUST set this field to false if SP does not have the
  /// PUBLISH_READONLY controller capability.
  /// This is a REQUIRED field.
  #[prost(bool, tag = "4")]
  pub readonly: bool,
  /// Secrets required by plugin to complete controller publish volume
  /// request. This field is OPTIONAL. Refer to the
  /// `Secrets Requirements` section on how to use this field.
  #[prost(map = "string, string", tag = "5")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  #[prost(map = "string, string", tag = "6")]
  pub volume_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerPublishVolumeResponse {
  /// Opaque static publish properties of the volume. SP MAY use this
  /// field to ensure subsequent `NodeStageVolume` or `NodePublishVolume`
  /// calls calls have contextual information.
  /// The contents of this field SHALL be opaque to a CO.
  /// The contents of this field SHALL NOT be mutable.
  /// The contents of this field SHALL be safe for the CO to cache.
  /// The contents of this field SHOULD NOT contain sensitive
  /// information.
  /// The contents of this field SHOULD NOT be used for uniquely
  /// identifying a volume. The `volume_id` alone SHOULD be sufficient to
  /// identify the volume.
  /// This field is OPTIONAL and when present MUST be passed to
  /// subsequent `NodeStageVolume` or `NodePublishVolume` calls
  #[prost(map = "string, string", tag = "1")]
  pub publish_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerUnpublishVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The ID of the node. This field is OPTIONAL. The CO SHOULD set this
  /// field to match the node ID returned by `NodeGetInfo` or leave it
  /// unset. If the value is set, the SP MUST unpublish the volume from
  /// the specified node. If the value is unset, the SP MUST unpublish
  /// the volume from all nodes it is published to.
  #[prost(string, tag = "2")]
  pub node_id: ::prost::alloc::string::String,
  /// Secrets required by plugin to complete controller unpublish volume
  /// request. This SHOULD be the same secrets passed to the
  /// ControllerPublishVolume call for the specified volume.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "3")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerUnpublishVolumeResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidateVolumeCapabilitiesRequest {
  /// The ID of the volume to check. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  #[prost(map = "string, string", tag = "2")]
  pub volume_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// The capabilities that the CO wants to check for the volume. This
  /// call SHALL return "confirmed" only if all the volume capabilities
  /// specified below are supported. This field is REQUIRED.
  #[prost(message, repeated, tag = "3")]
  pub volume_capabilities: ::prost::alloc::vec::Vec<VolumeCapability>,
  /// See CreateVolumeRequest.parameters.
  /// This field is OPTIONAL.
  #[prost(map = "string, string", tag = "4")]
  pub parameters:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// Secrets required by plugin to complete volume validation request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "5")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidateVolumeCapabilitiesResponse {
  /// Confirmed indicates to the CO the set of capabilities that the
  /// plugin has validated. This field SHALL only be set to a non-empty
  /// value for successful validation responses.
  /// For successful validation responses, the CO SHALL compare the
  /// fields of this message to the originally requested capabilities in
  /// order to guard against an older plugin reporting "valid" for newer
  /// capability fields that it does not yet understand.
  /// This field is OPTIONAL.
  #[prost(message, optional, tag = "1")]
  pub confirmed: ::core::option::Option<validate_volume_capabilities_response::Confirmed>,
  /// Message to the CO if `confirmed` above is empty. This field is
  /// OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  #[prost(string, tag = "2")]
  pub message: ::prost::alloc::string::String,
}
/// Nested message and enum types in `ValidateVolumeCapabilitiesResponse`.
pub mod validate_volume_capabilities_response {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Confirmed {
    /// Volume context validated by the plugin.
    /// This field is OPTIONAL.
    #[prost(map = "string, string", tag = "1")]
    pub volume_context:
      ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Volume capabilities supported by the plugin.
    /// This field is REQUIRED.
    #[prost(message, repeated, tag = "2")]
    pub volume_capabilities: ::prost::alloc::vec::Vec<super::VolumeCapability>,
    /// The volume creation parameters validated by the plugin.
    /// This field is OPTIONAL.
    #[prost(map = "string, string", tag = "3")]
    pub parameters:
      ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListVolumesRequest {
  /// If specified (non-zero value), the Plugin MUST NOT return more
  /// entries than this number in the response. If the actual number of
  /// entries is more than this number, the Plugin MUST set `next_token`
  /// in the response which can be used to get the next page of entries
  /// in the subsequent `ListVolumes` call. This field is OPTIONAL. If
  /// not specified (zero value), it means there is no restriction on the
  /// number of entries that can be returned.
  /// The value of this field MUST NOT be negative.
  #[prost(int32, tag = "1")]
  pub max_entries: i32,
  /// A token to specify where to start paginating. Set this field to
  /// `next_token` returned by a previous `ListVolumes` call to get the
  /// next page of entries. This field is OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  #[prost(string, tag = "2")]
  pub starting_token: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListVolumesResponse {
  #[prost(message, repeated, tag = "1")]
  pub entries: ::prost::alloc::vec::Vec<list_volumes_response::Entry>,
  /// This token allows you to get the next page of entries for
  /// `ListVolumes` request. If the number of entries is larger than
  /// `max_entries`, use the `next_token` as a value for the
  /// `starting_token` field in the next `ListVolumes` request. This
  /// field is OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  #[prost(string, tag = "2")]
  pub next_token: ::prost::alloc::string::String,
}
/// Nested message and enum types in `ListVolumesResponse`.
pub mod list_volumes_response {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct VolumeStatus {
    /// A list of all `node_id` of nodes that the volume in this entry
    /// is controller published on.
    /// This field is OPTIONAL. If it is not specified and the SP has
    /// the LIST_VOLUMES_PUBLISHED_NODES controller capability, the CO
    /// MAY assume the volume is not controller published to any nodes.
    /// If the field is not specified and the SP does not have the
    /// LIST_VOLUMES_PUBLISHED_NODES controller capability, the CO MUST
    /// not interpret this field.
    /// published_node_ids MAY include nodes not published to or
    /// reported by the SP. The CO MUST be resilient to that.
    #[prost(string, repeated, tag = "1")]
    pub published_node_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Information about the current condition of the volume.
    /// This field is OPTIONAL.
    /// This field MUST be specified if the
    /// VOLUME_CONDITION controller capability is supported.
    #[prost(message, optional, tag = "2")]
    pub volume_condition: ::core::option::Option<super::VolumeCondition>,
  }
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Entry {
    /// This field is REQUIRED
    #[prost(message, optional, tag = "1")]
    pub volume: ::core::option::Option<super::Volume>,
    /// This field is OPTIONAL. This field MUST be specified if the
    /// LIST_VOLUMES_PUBLISHED_NODES controller capability is
    /// supported.
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<VolumeStatus>,
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerGetVolumeRequest {
  /// The ID of the volume to fetch current volume information for.
  /// This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerGetVolumeResponse {
  /// This field is REQUIRED
  #[prost(message, optional, tag = "1")]
  pub volume: ::core::option::Option<Volume>,
  /// This field is REQUIRED.
  #[prost(message, optional, tag = "2")]
  pub status: ::core::option::Option<controller_get_volume_response::VolumeStatus>,
}
/// Nested message and enum types in `ControllerGetVolumeResponse`.
pub mod controller_get_volume_response {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct VolumeStatus {
    /// A list of all the `node_id` of nodes that this volume is
    /// controller published on.
    /// This field is OPTIONAL.
    /// This field MUST be specified if the PUBLISH_UNPUBLISH_VOLUME
    /// controller capability is supported.
    /// published_node_ids MAY include nodes not published to or
    /// reported by the SP. The CO MUST be resilient to that.
    #[prost(string, repeated, tag = "1")]
    pub published_node_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Information about the current condition of the volume.
    /// This field is OPTIONAL.
    /// This field MUST be specified if the
    /// VOLUME_CONDITION controller capability is supported.
    #[prost(message, optional, tag = "2")]
    pub volume_condition: ::core::option::Option<super::VolumeCondition>,
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCapacityRequest {
  /// If specified, the Plugin SHALL report the capacity of the storage
  /// that can be used to provision volumes that satisfy ALL of the
  /// specified `volume_capabilities`. These are the same
  /// `volume_capabilities` the CO will use in `CreateVolumeRequest`.
  /// This field is OPTIONAL.
  #[prost(message, repeated, tag = "1")]
  pub volume_capabilities: ::prost::alloc::vec::Vec<VolumeCapability>,
  /// If specified, the Plugin SHALL report the capacity of the storage
  /// that can be used to provision volumes with the given Plugin
  /// specific `parameters`. These are the same `parameters` the CO will
  /// use in `CreateVolumeRequest`. This field is OPTIONAL.
  #[prost(map = "string, string", tag = "2")]
  pub parameters:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// If specified, the Plugin SHALL report the capacity of the storage
  /// that can be used to provision volumes that in the specified
  /// `accessible_topology`. This is the same as the
  /// `accessible_topology` the CO returns in a `CreateVolumeResponse`.
  /// This field is OPTIONAL. This field SHALL NOT be set unless the
  /// plugin advertises the VOLUME_ACCESSIBILITY_CONSTRAINTS capability.
  #[prost(message, optional, tag = "3")]
  pub accessible_topology: ::core::option::Option<Topology>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCapacityResponse {
  /// The available capacity, in bytes, of the storage that can be used
  /// to provision volumes. If `volume_capabilities` or `parameters` is
  /// specified in the request, the Plugin SHALL take those into
  /// consideration when calculating the available capacity of the
  /// storage. This field is REQUIRED.
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "1")]
  pub available_capacity: i64,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerGetCapabilitiesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerGetCapabilitiesResponse {
  /// All the capabilities that the controller service supports. This
  /// field is OPTIONAL.
  #[prost(message, repeated, tag = "1")]
  pub capabilities: ::prost::alloc::vec::Vec<ControllerServiceCapability>,
}
/// Specifies a capability of the controller service.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerServiceCapability {
  #[prost(oneof = "controller_service_capability::Type", tags = "1")]
  pub r#type: ::core::option::Option<controller_service_capability::Type>,
}
/// Nested message and enum types in `ControllerServiceCapability`.
pub mod controller_service_capability {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Rpc {
    #[prost(enumeration = "rpc::Type", tag = "1")]
    pub r#type: i32,
  }
  /// Nested message and enum types in `RPC`.
  pub mod rpc {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
      Unknown = 0,
      CreateDeleteVolume = 1,
      PublishUnpublishVolume = 2,
      ListVolumes = 3,
      GetCapacity = 4,
      /// Currently the only way to consume a snapshot is to create
      /// a volume from it. Therefore plugins supporting
      /// CREATE_DELETE_SNAPSHOT MUST support creating volume from
      /// snapshot.
      CreateDeleteSnapshot = 5,
      ListSnapshots = 6,
      /// Plugins supporting volume cloning at the storage level MAY
      /// report this capability. The source volume MUST be managed by
      /// the same plugin. Not all volume sources and parameters
      /// combinations MAY work.
      CloneVolume = 7,
      /// Indicates the SP supports ControllerPublishVolume.readonly
      /// field.
      PublishReadonly = 8,
      /// See VolumeExpansion for details.
      ExpandVolume = 9,
      /// Indicates the SP supports the
      /// ListVolumesResponse.entry.published_nodes field
      ListVolumesPublishedNodes = 10,
      /// Indicates that the Controller service can report volume
      /// conditions.
      /// An SP MAY implement `VolumeCondition` in only the Controller
      /// Plugin, only the Node Plugin, or both.
      /// If `VolumeCondition` is implemented in both the Controller and
      /// Node Plugins, it SHALL report from different perspectives.
      /// If for some reason Controller and Node Plugins report
      /// misaligned volume conditions, CO SHALL assume the worst case
      /// is the truth.
      /// Note that, for alpha, `VolumeCondition` is intended be
      /// informative for humans only, not for automation.
      VolumeCondition = 11,
      /// Indicates the SP supports the ControllerGetVolume RPC.
      /// This enables COs to, for example, fetch per volume
      /// condition after a volume is provisioned.
      GetVolume = 12,
    }
  }
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Type {
    /// RPC that the controller supports.
    #[prost(message, tag = "1")]
    Rpc(Rpc),
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSnapshotRequest {
  /// The ID of the source volume to be snapshotted.
  /// This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub source_volume_id: ::prost::alloc::string::String,
  /// The suggested name for the snapshot. This field is REQUIRED for
  /// idempotency.
  /// Any Unicode string that conforms to the length limit is allowed
  /// except those containing the following banned characters:
  /// U+0000-U+0008, U+000B, U+000C, U+000E-U+001F, U+007F-U+009F.
  /// (These are control characters other than commonly used whitespace.)
  #[prost(string, tag = "2")]
  pub name: ::prost::alloc::string::String,
  /// Secrets required by plugin to complete snapshot creation request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "3")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
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
  #[prost(map = "string, string", tag = "4")]
  pub parameters:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSnapshotResponse {
  /// Contains all attributes of the newly created snapshot that are
  /// relevant to the CO along with information required by the Plugin
  /// to uniquely identify the snapshot. This field is REQUIRED.
  #[prost(message, optional, tag = "1")]
  pub snapshot: ::core::option::Option<Snapshot>,
}
/// Information about a specific snapshot.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Snapshot {
  /// This is the complete size of the snapshot in bytes. The purpose of
  /// this field is to give CO guidance on how much space is needed to
  /// create a volume from this snapshot. The size of the volume MUST NOT
  /// be less than the size of the source snapshot. This field is
  /// OPTIONAL. If this field is not set, it indicates that this size is
  /// unknown. The value of this field MUST NOT be negative and a size of
  /// zero means it is unspecified.
  #[prost(int64, tag = "1")]
  pub size_bytes: i64,
  /// The identifier for this snapshot, generated by the plugin.
  /// This field is REQUIRED.
  /// This field MUST contain enough information to uniquely identify
  /// this specific snapshot vs all other snapshots supported by this
  /// plugin.
  /// This field SHALL be used by the CO in subsequent calls to refer to
  /// this snapshot.
  /// The SP is NOT responsible for global uniqueness of snapshot_id
  /// across multiple SPs.
  #[prost(string, tag = "2")]
  pub snapshot_id: ::prost::alloc::string::String,
  /// Identity information for the source volume. Note that creating a
  /// snapshot from a snapshot is not supported here so the source has to
  /// be a volume. This field is REQUIRED.
  #[prost(string, tag = "3")]
  pub source_volume_id: ::prost::alloc::string::String,
  /// Timestamp when the point-in-time snapshot is taken on the storage
  /// system. This field is REQUIRED.
  #[prost(message, optional, tag = "4")]
  pub creation_time: ::core::option::Option<::prost_types::Timestamp>,
  /// Indicates if a snapshot is ready to use as a
  /// `volume_content_source` in a `CreateVolumeRequest`. The default
  /// value is false. This field is REQUIRED.
  #[prost(bool, tag = "5")]
  pub ready_to_use: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSnapshotRequest {
  /// The ID of the snapshot to be deleted.
  /// This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub snapshot_id: ::prost::alloc::string::String,
  /// Secrets required by plugin to complete snapshot deletion request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "2")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSnapshotResponse {}
/// List all snapshots on the storage system regardless of how they were
/// created.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSnapshotsRequest {
  /// If specified (non-zero value), the Plugin MUST NOT return more
  /// entries than this number in the response. If the actual number of
  /// entries is more than this number, the Plugin MUST set `next_token`
  /// in the response which can be used to get the next page of entries
  /// in the subsequent `ListSnapshots` call. This field is OPTIONAL. If
  /// not specified (zero value), it means there is no restriction on the
  /// number of entries that can be returned.
  /// The value of this field MUST NOT be negative.
  #[prost(int32, tag = "1")]
  pub max_entries: i32,
  /// A token to specify where to start paginating. Set this field to
  /// `next_token` returned by a previous `ListSnapshots` call to get the
  /// next page of entries. This field is OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  #[prost(string, tag = "2")]
  pub starting_token: ::prost::alloc::string::String,
  /// Identity information for the source volume. This field is OPTIONAL.
  /// It can be used to list snapshots by volume.
  #[prost(string, tag = "3")]
  pub source_volume_id: ::prost::alloc::string::String,
  /// Identity information for a specific snapshot. This field is
  /// OPTIONAL. It can be used to list only a specific snapshot.
  /// ListSnapshots will return with current snapshot information
  /// and will not block if the snapshot is being processed after
  /// it is cut.
  #[prost(string, tag = "4")]
  pub snapshot_id: ::prost::alloc::string::String,
  /// Secrets required by plugin to complete ListSnapshot request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "5")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSnapshotsResponse {
  #[prost(message, repeated, tag = "1")]
  pub entries: ::prost::alloc::vec::Vec<list_snapshots_response::Entry>,
  /// This token allows you to get the next page of entries for
  /// `ListSnapshots` request. If the number of entries is larger than
  /// `max_entries`, use the `next_token` as a value for the
  /// `starting_token` field in the next `ListSnapshots` request. This
  /// field is OPTIONAL.
  /// An empty string is equal to an unspecified field value.
  #[prost(string, tag = "2")]
  pub next_token: ::prost::alloc::string::String,
}
/// Nested message and enum types in `ListSnapshotsResponse`.
pub mod list_snapshots_response {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Entry {
    #[prost(message, optional, tag = "1")]
    pub snapshot: ::core::option::Option<super::Snapshot>,
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerExpandVolumeRequest {
  /// The ID of the volume to expand. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// This allows CO to specify the capacity requirements of the volume
  /// after expansion. This field is REQUIRED.
  #[prost(message, optional, tag = "2")]
  pub capacity_range: ::core::option::Option<CapacityRange>,
  /// Secrets required by the plugin for expanding the volume.
  /// This field is OPTIONAL.
  #[prost(map = "string, string", tag = "3")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// Volume capability describing how the CO intends to use this volume.
  /// This allows SP to determine if volume is being used as a block
  /// device or mounted file system. For example - if volume is
  /// being used as a block device - the SP MAY set
  /// node_expansion_required to false in ControllerExpandVolumeResponse
  /// to skip invocation of NodeExpandVolume on the node by the CO.
  /// This is an OPTIONAL field.
  #[prost(message, optional, tag = "4")]
  pub volume_capability: ::core::option::Option<VolumeCapability>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerExpandVolumeResponse {
  /// Capacity of volume after expansion. This field is REQUIRED.
  #[prost(int64, tag = "1")]
  pub capacity_bytes: i64,
  /// Whether node expansion is required for the volume. When true
  /// the CO MUST make NodeExpandVolume RPC call on the node. This field
  /// is REQUIRED.
  #[prost(bool, tag = "2")]
  pub node_expansion_required: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeStageVolumeRequest {
  /// The ID of the volume to publish. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The CO SHALL set this field to the value returned by
  /// `ControllerPublishVolume` if the corresponding Controller Plugin
  /// has `PUBLISH_UNPUBLISH_VOLUME` controller capability, and SHALL be
  /// left unset if the corresponding Controller Plugin does not have
  /// this capability. This is an OPTIONAL field.
  #[prost(map = "string, string", tag = "2")]
  pub publish_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// The path to which the volume MAY be staged. It MUST be an
  /// absolute path in the root filesystem of the process serving this
  /// request, and MUST be a directory. The CO SHALL ensure that there
  /// is only one `staging_target_path` per volume. The CO SHALL ensure
  /// that the path is directory and that the process serving the
  /// request has `read` and `write` permission to that directory. The
  /// CO SHALL be responsible for creating the directory if it does not
  /// exist.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "3")]
  pub staging_target_path: ::prost::alloc::string::String,
  /// Volume capability describing how the CO intends to use this volume.
  /// SP MUST ensure the CO can use the staged volume as described.
  /// Otherwise SP MUST return the appropriate gRPC error code.
  /// This is a REQUIRED field.
  #[prost(message, optional, tag = "4")]
  pub volume_capability: ::core::option::Option<VolumeCapability>,
  /// Secrets required by plugin to complete node stage volume request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "5")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  #[prost(map = "string, string", tag = "6")]
  pub volume_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeStageVolumeResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeUnstageVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The path at which the volume was staged. It MUST be an absolute
  /// path in the root filesystem of the process serving this request.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "2")]
  pub staging_target_path: ::prost::alloc::string::String,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeUnstageVolumeResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodePublishVolumeRequest {
  /// The ID of the volume to publish. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The CO SHALL set this field to the value returned by
  /// `ControllerPublishVolume` if the corresponding Controller Plugin
  /// has `PUBLISH_UNPUBLISH_VOLUME` controller capability, and SHALL be
  /// left unset if the corresponding Controller Plugin does not have
  /// this capability. This is an OPTIONAL field.
  #[prost(map = "string, string", tag = "2")]
  pub publish_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// The path to which the volume was staged by `NodeStageVolume`.
  /// It MUST be an absolute path in the root filesystem of the process
  /// serving this request.
  /// It MUST be set if the Node Plugin implements the
  /// `STAGE_UNSTAGE_VOLUME` node capability.
  /// This is an OPTIONAL field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "3")]
  pub staging_target_path: ::prost::alloc::string::String,
  /// The path to which the volume will be published. It MUST be an
  /// absolute path in the root filesystem of the process serving this
  /// request. The CO SHALL ensure uniqueness of target_path per volume.
  /// The CO SHALL ensure that the parent directory of this path exists
  /// and that the process serving the request has `read` and `write`
  /// permissions to that parent directory.
  /// For volumes with an access type of block, the SP SHALL place the
  /// block device at target_path.
  /// For volumes with an access type of mount, the SP SHALL place the
  /// mounted directory at target_path.
  /// Creation of target_path is the responsibility of the SP.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "4")]
  pub target_path: ::prost::alloc::string::String,
  /// Volume capability describing how the CO intends to use this volume.
  /// SP MUST ensure the CO can use the published volume as described.
  /// Otherwise SP MUST return the appropriate gRPC error code.
  /// This is a REQUIRED field.
  #[prost(message, optional, tag = "5")]
  pub volume_capability: ::core::option::Option<VolumeCapability>,
  /// Indicates SP MUST publish the volume in readonly mode.
  /// This field is REQUIRED.
  #[prost(bool, tag = "6")]
  pub readonly: bool,
  /// Secrets required by plugin to complete node publish volume request.
  /// This field is OPTIONAL. Refer to the `Secrets Requirements`
  /// section on how to use this field.
  #[prost(map = "string, string", tag = "7")]
  pub secrets:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
  /// Volume context as returned by SP in
  /// CreateVolumeResponse.Volume.volume_context.
  /// This field is OPTIONAL and MUST match the volume_context of the
  /// volume identified by `volume_id`.
  #[prost(map = "string, string", tag = "8")]
  pub volume_context:
    ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodePublishVolumeResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeUnpublishVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The path at which the volume was published. It MUST be an absolute
  /// path in the root filesystem of the process serving this request.
  /// The SP MUST delete the file or directory it created at this path.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "2")]
  pub target_path: ::prost::alloc::string::String,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeUnpublishVolumeResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeGetVolumeStatsRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// It can be any valid path where volume was previously
  /// staged or published.
  /// It MUST be an absolute path in the root filesystem of
  /// the process serving this request.
  /// This is a REQUIRED field.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "2")]
  pub volume_path: ::prost::alloc::string::String,
  /// The path where the volume is staged, if the plugin has the
  /// STAGE_UNSTAGE_VOLUME capability, otherwise empty.
  /// If not empty, it MUST be an absolute path in the root
  /// filesystem of the process serving this request.
  /// This field is OPTIONAL.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "3")]
  pub staging_target_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeGetVolumeStatsResponse {
  /// This field is OPTIONAL.
  #[prost(message, repeated, tag = "1")]
  pub usage: ::prost::alloc::vec::Vec<VolumeUsage>,
  /// Information about the current condition of the volume.
  /// This field is OPTIONAL.
  /// This field MUST be specified if the VOLUME_CONDITION node
  /// capability is supported.
  #[prost(message, optional, tag = "2")]
  pub volume_condition: ::core::option::Option<VolumeCondition>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VolumeUsage {
  /// The available capacity in specified Unit. This field is OPTIONAL.
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "1")]
  pub available: i64,
  /// The total capacity in specified Unit. This field is REQUIRED.
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "2")]
  pub total: i64,
  /// The used capacity in specified Unit. This field is OPTIONAL.
  /// The value of this field MUST NOT be negative.
  #[prost(int64, tag = "3")]
  pub used: i64,
  /// Units by which values are measured. This field is REQUIRED.
  #[prost(enumeration = "volume_usage::Unit", tag = "4")]
  pub unit: i32,
}
/// Nested message and enum types in `VolumeUsage`.
pub mod volume_usage {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
  #[repr(i32)]
  pub enum Unit {
    Unknown = 0,
    Bytes = 1,
    Inodes = 2,
  }
}
/// VolumeCondition represents the current condition of a volume.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VolumeCondition {
  /// Normal volumes are available for use and operating optimally.
  /// An abnormal volume does not meet these criteria.
  /// This field is REQUIRED.
  #[prost(bool, tag = "1")]
  pub abnormal: bool,
  /// The message describing the condition of the volume.
  /// This field is REQUIRED.
  #[prost(string, tag = "2")]
  pub message: ::prost::alloc::string::String,
}
/// Intentionally empty.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeGetCapabilitiesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeGetCapabilitiesResponse {
  /// All the capabilities that the node service supports. This field
  /// is OPTIONAL.
  #[prost(message, repeated, tag = "1")]
  pub capabilities: ::prost::alloc::vec::Vec<NodeServiceCapability>,
}
/// Specifies a capability of the node service.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeServiceCapability {
  #[prost(oneof = "node_service_capability::Type", tags = "1")]
  pub r#type: ::core::option::Option<node_service_capability::Type>,
}
/// Nested message and enum types in `NodeServiceCapability`.
pub mod node_service_capability {
  #[derive(Clone, PartialEq, ::prost::Message)]
  pub struct Rpc {
    #[prost(enumeration = "rpc::Type", tag = "1")]
    pub r#type: i32,
  }
  /// Nested message and enum types in `RPC`.
  pub mod rpc {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
      Unknown = 0,
      StageUnstageVolume = 1,
      /// If Plugin implements GET_VOLUME_STATS capability
      /// then it MUST implement NodeGetVolumeStats RPC
      /// call for fetching volume statistics.
      GetVolumeStats = 2,
      /// See VolumeExpansion for details.
      ExpandVolume = 3,
      /// Indicates that the Node service can report volume conditions.
      /// An SP MAY implement `VolumeCondition` in only the Node
      /// Plugin, only the Controller Plugin, or both.
      /// If `VolumeCondition` is implemented in both the Node and
      /// Controller Plugins, it SHALL report from different
      /// perspectives.
      /// If for some reason Node and Controller Plugins report
      /// misaligned volume conditions, CO SHALL assume the worst case
      /// is the truth.
      /// Note that, for alpha, `VolumeCondition` is intended to be
      /// informative for humans only, not for automation.
      VolumeCondition = 4,
    }
  }
  #[derive(Clone, PartialEq, ::prost::Oneof)]
  pub enum Type {
    /// RPC that the controller supports.
    #[prost(message, tag = "1")]
    Rpc(Rpc),
  }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeGetInfoRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
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
  #[prost(string, tag = "1")]
  pub node_id: ::prost::alloc::string::String,
  /// Maximum number of volumes that controller can publish to the node.
  /// If value is not set or zero CO SHALL decide how many volumes of
  /// this type can be published by the controller to the node. The
  /// plugin MUST NOT set negative values here.
  /// This field is OPTIONAL.
  #[prost(int64, tag = "2")]
  pub max_volumes_per_node: i64,
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
  ///   accessible_topology =
  ///     {"region": "R1", "zone": "Z2"}
  /// Indicates the node exists within the "region" "R1" and the "zone"
  /// "Z2".
  #[prost(message, optional, tag = "3")]
  pub accessible_topology: ::core::option::Option<Topology>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeExpandVolumeRequest {
  /// The ID of the volume. This field is REQUIRED.
  #[prost(string, tag = "1")]
  pub volume_id: ::prost::alloc::string::String,
  /// The path on which volume is available. This field is REQUIRED.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "2")]
  pub volume_path: ::prost::alloc::string::String,
  /// This allows CO to specify the capacity requirements of the volume
  /// after expansion. If capacity_range is omitted then a plugin MAY
  /// inspect the file system of the volume to determine the maximum
  /// capacity to which the volume can be expanded. In such cases a
  /// plugin MAY expand the volume to its maximum capacity.
  /// This field is OPTIONAL.
  #[prost(message, optional, tag = "3")]
  pub capacity_range: ::core::option::Option<CapacityRange>,
  /// The path where the volume is staged, if the plugin has the
  /// STAGE_UNSTAGE_VOLUME capability, otherwise empty.
  /// If not empty, it MUST be an absolute path in the root
  /// filesystem of the process serving this request.
  /// This field is OPTIONAL.
  /// This field overrides the general CSI size limit.
  /// SP SHOULD support the maximum path length allowed by the operating
  /// system/filesystem, but, at a minimum, SP MUST accept a max path
  /// length of at least 128 bytes.
  #[prost(string, tag = "4")]
  pub staging_target_path: ::prost::alloc::string::String,
  /// Volume capability describing how the CO intends to use this volume.
  /// This allows SP to determine if volume is being used as a block
  /// device or mounted file system. For example - if volume is being
  /// used as a block device the SP MAY choose to skip expanding the
  /// filesystem in NodeExpandVolume implementation but still perform
  /// rest of the housekeeping needed for expanding the volume. If
  /// volume_capability is omitted the SP MAY determine
  /// access_type from given volume_path for the volume and perform
  /// node expansion. This is an OPTIONAL field.
  #[prost(message, optional, tag = "5")]
  pub volume_capability: ::core::option::Option<VolumeCapability>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeExpandVolumeResponse {
  /// The capacity of the volume in bytes. This field is OPTIONAL.
  #[prost(int64, tag = "1")]
  pub capacity_bytes: i64,
}
#[doc = r" Generated server implementations."]
pub mod identity_server {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  #[doc = "Generated trait containing gRPC methods that should be implemented for use with IdentityServer."]
  #[async_trait]
  pub trait Identity: Send + Sync + 'static {
    async fn get_plugin_info(
      &self,
      request: tonic::Request<super::GetPluginInfoRequest>,
    ) -> Result<tonic::Response<super::GetPluginInfoResponse>, tonic::Status>;
    async fn get_plugin_capabilities(
      &self,
      request: tonic::Request<super::GetPluginCapabilitiesRequest>,
    ) -> Result<tonic::Response<super::GetPluginCapabilitiesResponse>, tonic::Status>;
    async fn probe(
      &self,
      request: tonic::Request<super::ProbeRequest>,
    ) -> Result<tonic::Response<super::ProbeResponse>, tonic::Status>;
  }
  #[derive(Debug)]
  pub struct IdentityServer<T: Identity> {
    inner: _Inner<T>,
  }
  struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
  impl<T: Identity> IdentityServer<T> {
    pub fn new(inner: T) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, None);
      Self { inner }
    }
    pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, Some(interceptor.into()));
      Self { inner }
    }
  }
  impl<T, B> Service<http::Request<B>> for IdentityServer<T>
  where
    T: Identity,
    B: HttpBody + Send + Sync + 'static,
    B::Error: Into<StdError> + Send + 'static,
  {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = Never;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
      let inner = self.inner.clone();
      match req.uri().path() {
        "/csi.v1.Identity/GetPluginInfo" => {
          #[allow(non_camel_case_types)]
          struct GetPluginInfoSvc<T: Identity>(pub Arc<T>);
          impl<T: Identity> tonic::server::UnaryService<super::GetPluginInfoRequest> for GetPluginInfoSvc<T> {
            type Response = super::GetPluginInfoResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::GetPluginInfoRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).get_plugin_info(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = GetPluginInfoSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Identity/GetPluginCapabilities" => {
          #[allow(non_camel_case_types)]
          struct GetPluginCapabilitiesSvc<T: Identity>(pub Arc<T>);
          impl<T: Identity> tonic::server::UnaryService<super::GetPluginCapabilitiesRequest>
            for GetPluginCapabilitiesSvc<T>
          {
            type Response = super::GetPluginCapabilitiesResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::GetPluginCapabilitiesRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).get_plugin_capabilities(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = GetPluginCapabilitiesSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Identity/Probe" => {
          #[allow(non_camel_case_types)]
          struct ProbeSvc<T: Identity>(pub Arc<T>);
          impl<T: Identity> tonic::server::UnaryService<super::ProbeRequest> for ProbeSvc<T> {
            type Response = super::ProbeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::ProbeRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).probe(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ProbeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        _ => Box::pin(async move {
          Ok(
            http::Response::builder()
              .status(200)
              .header("grpc-status", "12")
              .header("content-type", "application/grpc")
              .body(tonic::body::BoxBody::empty())
              .unwrap(),
          )
        }),
      }
    }
  }
  impl<T: Identity> Clone for IdentityServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self { inner }
    }
  }
  impl<T: Identity> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone(), self.1.clone())
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: Identity> tonic::transport::NamedService for IdentityServer<T> {
    const NAME: &'static str = "csi.v1.Identity";
  }
}
#[doc = r" Generated server implementations."]
pub mod controller_server {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  #[doc = "Generated trait containing gRPC methods that should be implemented for use with ControllerServer."]
  #[async_trait]
  pub trait Controller: Send + Sync + 'static {
    async fn create_volume(
      &self,
      request: tonic::Request<super::CreateVolumeRequest>,
    ) -> Result<tonic::Response<super::CreateVolumeResponse>, tonic::Status>;
    async fn delete_volume(
      &self,
      request: tonic::Request<super::DeleteVolumeRequest>,
    ) -> Result<tonic::Response<super::DeleteVolumeResponse>, tonic::Status>;
    async fn controller_publish_volume(
      &self,
      request: tonic::Request<super::ControllerPublishVolumeRequest>,
    ) -> Result<tonic::Response<super::ControllerPublishVolumeResponse>, tonic::Status>;
    async fn controller_unpublish_volume(
      &self,
      request: tonic::Request<super::ControllerUnpublishVolumeRequest>,
    ) -> Result<tonic::Response<super::ControllerUnpublishVolumeResponse>, tonic::Status>;
    async fn validate_volume_capabilities(
      &self,
      request: tonic::Request<super::ValidateVolumeCapabilitiesRequest>,
    ) -> Result<tonic::Response<super::ValidateVolumeCapabilitiesResponse>, tonic::Status>;
    async fn list_volumes(
      &self,
      request: tonic::Request<super::ListVolumesRequest>,
    ) -> Result<tonic::Response<super::ListVolumesResponse>, tonic::Status>;
    async fn get_capacity(
      &self,
      request: tonic::Request<super::GetCapacityRequest>,
    ) -> Result<tonic::Response<super::GetCapacityResponse>, tonic::Status>;
    async fn controller_get_capabilities(
      &self,
      request: tonic::Request<super::ControllerGetCapabilitiesRequest>,
    ) -> Result<tonic::Response<super::ControllerGetCapabilitiesResponse>, tonic::Status>;
    async fn create_snapshot(
      &self,
      request: tonic::Request<super::CreateSnapshotRequest>,
    ) -> Result<tonic::Response<super::CreateSnapshotResponse>, tonic::Status>;
    async fn delete_snapshot(
      &self,
      request: tonic::Request<super::DeleteSnapshotRequest>,
    ) -> Result<tonic::Response<super::DeleteSnapshotResponse>, tonic::Status>;
    async fn list_snapshots(
      &self,
      request: tonic::Request<super::ListSnapshotsRequest>,
    ) -> Result<tonic::Response<super::ListSnapshotsResponse>, tonic::Status>;
    async fn controller_expand_volume(
      &self,
      request: tonic::Request<super::ControllerExpandVolumeRequest>,
    ) -> Result<tonic::Response<super::ControllerExpandVolumeResponse>, tonic::Status>;
    async fn controller_get_volume(
      &self,
      request: tonic::Request<super::ControllerGetVolumeRequest>,
    ) -> Result<tonic::Response<super::ControllerGetVolumeResponse>, tonic::Status>;
  }
  #[derive(Debug)]
  pub struct ControllerServer<T: Controller> {
    inner: _Inner<T>,
  }
  struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
  impl<T: Controller> ControllerServer<T> {
    pub fn new(inner: T) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, None);
      Self { inner }
    }
    pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, Some(interceptor.into()));
      Self { inner }
    }
  }
  impl<T, B> Service<http::Request<B>> for ControllerServer<T>
  where
    T: Controller,
    B: HttpBody + Send + Sync + 'static,
    B::Error: Into<StdError> + Send + 'static,
  {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = Never;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
      let inner = self.inner.clone();
      match req.uri().path() {
        "/csi.v1.Controller/CreateVolume" => {
          #[allow(non_camel_case_types)]
          struct CreateVolumeSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::CreateVolumeRequest> for CreateVolumeSvc<T> {
            type Response = super::CreateVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::CreateVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).create_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = CreateVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/DeleteVolume" => {
          #[allow(non_camel_case_types)]
          struct DeleteVolumeSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::DeleteVolumeRequest> for DeleteVolumeSvc<T> {
            type Response = super::DeleteVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::DeleteVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).delete_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = DeleteVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ControllerPublishVolume" => {
          #[allow(non_camel_case_types)]
          struct ControllerPublishVolumeSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ControllerPublishVolumeRequest>
            for ControllerPublishVolumeSvc<T>
          {
            type Response = super::ControllerPublishVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ControllerPublishVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).controller_publish_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ControllerPublishVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ControllerUnpublishVolume" => {
          #[allow(non_camel_case_types)]
          struct ControllerUnpublishVolumeSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ControllerUnpublishVolumeRequest>
            for ControllerUnpublishVolumeSvc<T>
          {
            type Response = super::ControllerUnpublishVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ControllerUnpublishVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).controller_unpublish_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ControllerUnpublishVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ValidateVolumeCapabilities" => {
          #[allow(non_camel_case_types)]
          struct ValidateVolumeCapabilitiesSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ValidateVolumeCapabilitiesRequest>
            for ValidateVolumeCapabilitiesSvc<T>
          {
            type Response = super::ValidateVolumeCapabilitiesResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ValidateVolumeCapabilitiesRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).validate_volume_capabilities(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ValidateVolumeCapabilitiesSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ListVolumes" => {
          #[allow(non_camel_case_types)]
          struct ListVolumesSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ListVolumesRequest> for ListVolumesSvc<T> {
            type Response = super::ListVolumesResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::ListVolumesRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).list_volumes(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ListVolumesSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/GetCapacity" => {
          #[allow(non_camel_case_types)]
          struct GetCapacitySvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::GetCapacityRequest> for GetCapacitySvc<T> {
            type Response = super::GetCapacityResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::GetCapacityRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).get_capacity(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = GetCapacitySvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ControllerGetCapabilities" => {
          #[allow(non_camel_case_types)]
          struct ControllerGetCapabilitiesSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ControllerGetCapabilitiesRequest>
            for ControllerGetCapabilitiesSvc<T>
          {
            type Response = super::ControllerGetCapabilitiesResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ControllerGetCapabilitiesRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).controller_get_capabilities(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ControllerGetCapabilitiesSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/CreateSnapshot" => {
          #[allow(non_camel_case_types)]
          struct CreateSnapshotSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::CreateSnapshotRequest>
            for CreateSnapshotSvc<T>
          {
            type Response = super::CreateSnapshotResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::CreateSnapshotRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).create_snapshot(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = CreateSnapshotSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/DeleteSnapshot" => {
          #[allow(non_camel_case_types)]
          struct DeleteSnapshotSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::DeleteSnapshotRequest>
            for DeleteSnapshotSvc<T>
          {
            type Response = super::DeleteSnapshotResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::DeleteSnapshotRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).delete_snapshot(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = DeleteSnapshotSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ListSnapshots" => {
          #[allow(non_camel_case_types)]
          struct ListSnapshotsSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ListSnapshotsRequest>
            for ListSnapshotsSvc<T>
          {
            type Response = super::ListSnapshotsResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ListSnapshotsRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).list_snapshots(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ListSnapshotsSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ControllerExpandVolume" => {
          #[allow(non_camel_case_types)]
          struct ControllerExpandVolumeSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ControllerExpandVolumeRequest>
            for ControllerExpandVolumeSvc<T>
          {
            type Response = super::ControllerExpandVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ControllerExpandVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).controller_expand_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ControllerExpandVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Controller/ControllerGetVolume" => {
          #[allow(non_camel_case_types)]
          struct ControllerGetVolumeSvc<T: Controller>(pub Arc<T>);
          impl<T: Controller> tonic::server::UnaryService<super::ControllerGetVolumeRequest>
            for ControllerGetVolumeSvc<T>
          {
            type Response = super::ControllerGetVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::ControllerGetVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).controller_get_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = ControllerGetVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        _ => Box::pin(async move {
          Ok(
            http::Response::builder()
              .status(200)
              .header("grpc-status", "12")
              .header("content-type", "application/grpc")
              .body(tonic::body::BoxBody::empty())
              .unwrap(),
          )
        }),
      }
    }
  }
  impl<T: Controller> Clone for ControllerServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self { inner }
    }
  }
  impl<T: Controller> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone(), self.1.clone())
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: Controller> tonic::transport::NamedService for ControllerServer<T> {
    const NAME: &'static str = "csi.v1.Controller";
  }
}
#[doc = r" Generated server implementations."]
pub mod node_server {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  #[doc = "Generated trait containing gRPC methods that should be implemented for use with NodeServer."]
  #[async_trait]
  pub trait Node: Send + Sync + 'static {
    async fn node_stage_volume(
      &self,
      request: tonic::Request<super::NodeStageVolumeRequest>,
    ) -> Result<tonic::Response<super::NodeStageVolumeResponse>, tonic::Status>;
    async fn node_unstage_volume(
      &self,
      request: tonic::Request<super::NodeUnstageVolumeRequest>,
    ) -> Result<tonic::Response<super::NodeUnstageVolumeResponse>, tonic::Status>;
    async fn node_publish_volume(
      &self,
      request: tonic::Request<super::NodePublishVolumeRequest>,
    ) -> Result<tonic::Response<super::NodePublishVolumeResponse>, tonic::Status>;
    async fn node_unpublish_volume(
      &self,
      request: tonic::Request<super::NodeUnpublishVolumeRequest>,
    ) -> Result<tonic::Response<super::NodeUnpublishVolumeResponse>, tonic::Status>;
    async fn node_get_volume_stats(
      &self,
      request: tonic::Request<super::NodeGetVolumeStatsRequest>,
    ) -> Result<tonic::Response<super::NodeGetVolumeStatsResponse>, tonic::Status>;
    async fn node_expand_volume(
      &self,
      request: tonic::Request<super::NodeExpandVolumeRequest>,
    ) -> Result<tonic::Response<super::NodeExpandVolumeResponse>, tonic::Status>;
    async fn node_get_capabilities(
      &self,
      request: tonic::Request<super::NodeGetCapabilitiesRequest>,
    ) -> Result<tonic::Response<super::NodeGetCapabilitiesResponse>, tonic::Status>;
    async fn node_get_info(
      &self,
      request: tonic::Request<super::NodeGetInfoRequest>,
    ) -> Result<tonic::Response<super::NodeGetInfoResponse>, tonic::Status>;
  }
  #[derive(Debug)]
  pub struct NodeServer<T: Node> {
    inner: _Inner<T>,
  }
  struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
  impl<T: Node> NodeServer<T> {
    pub fn new(inner: T) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, None);
      Self { inner }
    }
    pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, Some(interceptor.into()));
      Self { inner }
    }
  }
  impl<T, B> Service<http::Request<B>> for NodeServer<T>
  where
    T: Node,
    B: HttpBody + Send + Sync + 'static,
    B::Error: Into<StdError> + Send + 'static,
  {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = Never;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
      let inner = self.inner.clone();
      match req.uri().path() {
        "/csi.v1.Node/NodeStageVolume" => {
          #[allow(non_camel_case_types)]
          struct NodeStageVolumeSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeStageVolumeRequest> for NodeStageVolumeSvc<T> {
            type Response = super::NodeStageVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodeStageVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_stage_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeStageVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodeUnstageVolume" => {
          #[allow(non_camel_case_types)]
          struct NodeUnstageVolumeSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeUnstageVolumeRequest>
            for NodeUnstageVolumeSvc<T>
          {
            type Response = super::NodeUnstageVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodeUnstageVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_unstage_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeUnstageVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodePublishVolume" => {
          #[allow(non_camel_case_types)]
          struct NodePublishVolumeSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodePublishVolumeRequest>
            for NodePublishVolumeSvc<T>
          {
            type Response = super::NodePublishVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodePublishVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_publish_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodePublishVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodeUnpublishVolume" => {
          #[allow(non_camel_case_types)]
          struct NodeUnpublishVolumeSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeUnpublishVolumeRequest>
            for NodeUnpublishVolumeSvc<T>
          {
            type Response = super::NodeUnpublishVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodeUnpublishVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_unpublish_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeUnpublishVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodeGetVolumeStats" => {
          #[allow(non_camel_case_types)]
          struct NodeGetVolumeStatsSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeGetVolumeStatsRequest>
            for NodeGetVolumeStatsSvc<T>
          {
            type Response = super::NodeGetVolumeStatsResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodeGetVolumeStatsRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_get_volume_stats(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeGetVolumeStatsSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodeExpandVolume" => {
          #[allow(non_camel_case_types)]
          struct NodeExpandVolumeSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeExpandVolumeRequest>
            for NodeExpandVolumeSvc<T>
          {
            type Response = super::NodeExpandVolumeResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodeExpandVolumeRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_expand_volume(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeExpandVolumeSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodeGetCapabilities" => {
          #[allow(non_camel_case_types)]
          struct NodeGetCapabilitiesSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeGetCapabilitiesRequest>
            for NodeGetCapabilitiesSvc<T>
          {
            type Response = super::NodeGetCapabilitiesResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(
              &mut self,
              request: tonic::Request<super::NodeGetCapabilitiesRequest>,
            ) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_get_capabilities(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeGetCapabilitiesSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/csi.v1.Node/NodeGetInfo" => {
          #[allow(non_camel_case_types)]
          struct NodeGetInfoSvc<T: Node>(pub Arc<T>);
          impl<T: Node> tonic::server::UnaryService<super::NodeGetInfoRequest> for NodeGetInfoSvc<T> {
            type Response = super::NodeGetInfoResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::NodeGetInfoRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).node_get_info(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = NodeGetInfoSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        _ => Box::pin(async move {
          Ok(
            http::Response::builder()
              .status(200)
              .header("grpc-status", "12")
              .header("content-type", "application/grpc")
              .body(tonic::body::BoxBody::empty())
              .unwrap(),
          )
        }),
      }
    }
  }
  impl<T: Node> Clone for NodeServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self { inner }
    }
  }
  impl<T: Node> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone(), self.1.clone())
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: Node> tonic::transport::NamedService for NodeServer<T> {
    const NAME: &'static str = "csi.v1.Node";
  }
}
