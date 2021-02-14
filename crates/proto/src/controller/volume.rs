use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
  fmt,
  num::NonZeroU64,
};

use crate::proto;

pub type Topology = HashMap<String, String>;

#[derive(Debug)]
pub enum VolumeContentSource {
  Snapshot(String),
  Volume(String),
}

impl TryFrom<proto::VolumeContentSource> for Option<VolumeContentSource> {
  type Error = tonic::Status;

  fn try_from(value: proto::VolumeContentSource) -> Result<Self, Self::Error> {
    #[inline]
    fn fail_if_empty(v: String, error: &'static str) -> Result<String, tonic::Status> {
      if v.is_empty() {
        Err(tonic::Status::invalid_argument(error))
      } else {
        Ok(v)
      }
    }

    Ok(match value.r#type {
      None => None,
      Some(proto::volume_content_source::Type::Volume(v)) => Some(VolumeContentSource::Volume(
        fail_if_empty(v.volume_id, "VolumeContentSource volume_id cannot be empty")?,
      )),
      Some(proto::volume_content_source::Type::Snapshot(v)) => {
        Some(VolumeContentSource::Snapshot(fail_if_empty(
          v.snapshot_id,
          "VolumeContentSource snapshot_id cannot be empty",
        )?))
      }
    })
  }
}

impl TryFrom<VolumeContentSource> for proto::VolumeContentSource {
  type Error = tonic::Status;

  fn try_from(value: VolumeContentSource) -> Result<Self, Self::Error> {
    Ok(proto::VolumeContentSource {
      r#type: Some(match value {
        VolumeContentSource::Snapshot(snapshot_id) => proto::volume_content_source::Type::Snapshot(
          proto::volume_content_source::SnapshotSource { snapshot_id },
        ),
        VolumeContentSource::Volume(volume_id) => {
          proto::volume_content_source::Type::Volume(proto::volume_content_source::VolumeSource {
            volume_id,
          })
        }
      }),
    })
  }
}

#[derive(Debug)]
pub struct Volume {
  capacity_bytes: Option<NonZeroU64>,
  volume_id: String,
  volume_context: HashMap<String, String>,
  content_source: Option<VolumeContentSource>,
  accessible_topology: Vec<Topology>,
}

impl TryFrom<Volume> for proto::Volume {
  type Error = tonic::Status;

  fn try_from(value: Volume) -> Result<Self, Self::Error> {
    let capacity_bytes = match value.capacity_bytes {
      None => 0,
      Some(v) => v.get() as i64,
    };
    let volume_id = value.volume_id;
    let volume_context = value.volume_context;
    let content_source = value.content_source.map(TryInto::try_into).transpose()?;
    let accessible_topology = value
      .accessible_topology
      .into_iter()
      .map(|segments| proto::Topology { segments })
      .collect();

    Ok(proto::Volume {
      capacity_bytes,
      volume_id,
      volume_context,
      content_source,
      accessible_topology,
    })
  }
}

impl TryFrom<Volume> for proto::CreateVolumeResponse {
  type Error = tonic::Status;

  fn try_from(value: Volume) -> Result<Self, Self::Error> {
    let volume = Some(value.try_into()?);

    Ok(proto::CreateVolumeResponse { volume })
  }
}

#[derive(Debug)]
pub struct VolumeCapability {
  access_mode: AccessMode,
  access_type: AccessType,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AccessMode {
  Unknown,
  /// Can only be published once as read/write on a single node, at
  /// any given time.
  SingleNodeWriter,
  /// Can only be published once as readonly on a single node, at
  /// any given time.
  SingleNodeReaderOnly,
  /// Can be published as readonly at multiple nodes simultaneously.
  MultiNodeReaderOnly,
  /// Can be published at multiple nodes simultaneously. Only one of
  /// the node can be used as read/write. The rest will be readonly.
  MultiNodeSingleWriter,
  /// Can be published as read/write at multiple nodes
  /// simultaneously.
  MultiNodeMultiWriter,
}

impl TryFrom<proto::volume_capability::AccessMode> for AccessMode {
  type Error = tonic::Status;

  fn try_from(value: proto::volume_capability::AccessMode) -> Result<Self, Self::Error> {
    Ok(
      match proto::volume_capability::access_mode::Mode::from_i32(value.mode) {
        Some(proto::volume_capability::access_mode::Mode::SingleNodeWriter) => {
          AccessMode::SingleNodeWriter
        }
        Some(proto::volume_capability::access_mode::Mode::SingleNodeReaderOnly) => {
          AccessMode::SingleNodeReaderOnly
        }
        Some(proto::volume_capability::access_mode::Mode::MultiNodeReaderOnly) => {
          AccessMode::MultiNodeReaderOnly
        }
        Some(proto::volume_capability::access_mode::Mode::MultiNodeSingleWriter) => {
          AccessMode::MultiNodeSingleWriter
        }
        Some(proto::volume_capability::access_mode::Mode::MultiNodeMultiWriter) => {
          AccessMode::MultiNodeMultiWriter
        }
        _ => AccessMode::Unknown,
      },
    )
  }
}

impl TryFrom<AccessMode> for proto::volume_capability::AccessMode {
  type Error = tonic::Status;

  fn try_from(value: AccessMode) -> Result<Self, Self::Error> {
    let mode = match value {
      AccessMode::Unknown => proto::volume_capability::access_mode::Mode::Unknown,
      AccessMode::SingleNodeWriter => proto::volume_capability::access_mode::Mode::SingleNodeWriter,
      AccessMode::SingleNodeReaderOnly => {
        proto::volume_capability::access_mode::Mode::SingleNodeReaderOnly
      }
      AccessMode::MultiNodeReaderOnly => {
        proto::volume_capability::access_mode::Mode::MultiNodeReaderOnly
      }
      AccessMode::MultiNodeSingleWriter => {
        proto::volume_capability::access_mode::Mode::MultiNodeSingleWriter
      }
      AccessMode::MultiNodeMultiWriter => {
        proto::volume_capability::access_mode::Mode::MultiNodeMultiWriter
      }
    } as i32;

    Ok(proto::volume_capability::AccessMode { mode })
  }
}

#[derive(Debug)]
pub enum AccessType {
  /// Indicate that the volume will be accessed via the block device API.
  Block,

  /// Indicate that the volume will be accessed via the filesystem API.
  Mount(MountVolume),
}

impl TryFrom<proto::volume_capability::AccessType> for AccessType {
  type Error = tonic::Status;

  fn try_from(value: proto::volume_capability::AccessType) -> Result<Self, Self::Error> {
    Ok(match value {
      proto::volume_capability::AccessType::Block(_) => AccessType::Block,
      proto::volume_capability::AccessType::Mount(v) => AccessType::Mount(v.try_into()?),
    })
  }
}

impl TryFrom<AccessType> for proto::volume_capability::AccessType {
  type Error = tonic::Status;

  fn try_from(value: AccessType) -> Result<Self, Self::Error> {
    Ok(match value {
      AccessType::Block => {
        proto::volume_capability::AccessType::Block(proto::volume_capability::BlockVolume {})
      }
      AccessType::Mount(v) => proto::volume_capability::AccessType::Mount(v.try_into()?),
    })
  }
}

pub struct MountVolume {
  fs_type: Option<String>,
  mount_flags: Vec<String>,
}

impl MountVolume {
  /// The filesystem type.
  #[inline]
  pub fn fs_type(&self) -> Option<&str> {
    self.fs_type.as_deref()
  }

  /// The mount options that can be used for the volume. This field is
  /// OPTIONAL. `mount_flags` MAY contain sensitive information.
  /// Therefore, the CO and the Plugin MUST NOT leak this information
  /// to untrusted entities. The total size of this repeated field
  /// SHALL NOT exceed 4 KiB.
  pub fn mount_flags(&self) -> impl Iterator<Item = &str> + ExactSizeIterator {
    self.mount_flags.iter().map(|v| &**v)
  }
}

impl TryFrom<proto::volume_capability::MountVolume> for MountVolume {
  type Error = tonic::Status;

  fn try_from(value: proto::volume_capability::MountVolume) -> Result<Self, Self::Error> {
    let fs_type = match value.fs_type {
      v if v.is_empty() => None,
      v => Some(v),
    };

    let mount_flags = value.mount_flags;

    Ok(MountVolume {
      fs_type,
      mount_flags,
    })
  }
}

impl TryFrom<MountVolume> for proto::volume_capability::MountVolume {
  type Error = tonic::Status;

  fn try_from(value: MountVolume) -> Result<Self, Self::Error> {
    let fs_type = value.fs_type.unwrap_or_default();
    let mount_flags = value.mount_flags;

    Ok(proto::volume_capability::MountVolume {
      fs_type,
      mount_flags,
    })
  }
}

impl fmt::Debug for MountVolume {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MountVolume")
      .field("fs_type", &self.fs_type)
      .field(
        "mount_flags",
        &format!("REDACTED ({} items)", self.mount_flags.len()),
      )
      .finish()
  }
}

impl TryFrom<proto::VolumeCapability> for VolumeCapability {
  type Error = tonic::Status;

  fn try_from(value: proto::VolumeCapability) -> Result<Self, Self::Error> {
    let access_mode = value
      .access_mode
      .ok_or_else(|| tonic::Status::invalid_argument("Missing access_mode for VolumeCapability"))
      .and_then(TryInto::try_into)?;

    let access_type = value
      .access_type
      .ok_or_else(|| tonic::Status::invalid_argument("Missing access_type for VolumeCapability"))
      .and_then(TryInto::try_into)?;

    Ok(VolumeCapability {
      access_mode,
      access_type,
    })
  }
}

impl TryFrom<VolumeCapability> for proto::VolumeCapability {
  type Error = tonic::Status;

  fn try_from(value: VolumeCapability) -> Result<Self, Self::Error> {
    let access_mode = Some(value.access_mode.try_into()?);
    let access_type = Some(value.access_type.try_into()?);

    Ok(proto::VolumeCapability {
      access_mode,
      access_type,
    })
  }
}
