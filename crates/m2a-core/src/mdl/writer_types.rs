use std::fmt;

use serde::{Deserialize, Serialize};

use super::InspectionReport;

pub const M4_WRITER_SCHEMA_VERSION: u32 = 1;
pub const M4A_ANIMATION_SCHEMA_VERSION: u32 = 1;
/// Shared finite face-plane threshold used by the binary MDL writer and by
/// the final runtime-geometry sanitation pass.
pub const NWN_EE_BINARY_MDL_EPSILON_V1: f32 = 1.0e-5;
/// Current NWN:EE product boundary for one triangle-list mesh stream.
///
/// Raw vertex references are emitted as `u16`; constraining the stream to the
/// same 65,535-entry boundary keeps one mesh at no more than 21,845 triangles.
/// Larger models must be split into multiple model-IR segments before writing.
pub const NWN_EE_MAX_MESH_INDEX_COUNT_V1: usize = u16::MAX as usize;
pub const NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1: usize = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdlFormatProfileV1 {
    M4DirectCreatureExtended64V1,
    /// Extended64 SkinMesh with the unused inline palette tail filled with
    /// native-style zero terminators. The frozen V1 profile retains its
    /// historical `-1` tail for exact lineage replay.
    M4DirectCreatureExtended64ZeroTerminatedV2,
    /// Extends V2 for a native-style dedicated creature model root. The
    /// single parentless, model-named identity root has no redundant bind
    /// controllers; every descendant retains the V2 controller contract.
    M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
    /// Static, unskinned direct creature using the selected runtime family.
    M0StaticRigidNativeV1,
    /// Static, unskinned placeable. It uses the common binary MDL writer but
    /// derives model bounds from caller-owned geometry instead of the
    /// historical direct-creature culling envelope.
    PlaceableStaticRigidNativeV1,
    /// Static, unskinned item part. Item assembly remains UTI/baseitems.2da
    /// driven while each resolved part is an independent binary MDL.
    ItemPartStaticRigidNativeV1,
    /// Non-default, offline-only controlled experiment. It preserves the M0
    /// native mesh policy while allowing caller-bound source transform dummies.
    SourceTopologyPreservingRigidExperimentV1,
    /// Explicit production candidate family for a caller-bound source
    /// topology. Legacy M0/V2 construction never selects this variant.
    SourceTopologyPreservingRigidCandidateV1,
    /// Static tile profile using the common binary writer, caller-owned model
    /// bounds, classification 2 and one semantic AABB mesh sourced from the
    /// same `TileNavigationIrV1` as the ASCII WOK.
    TileStaticV1,
    Legacy17V1,
}

/// Selects one audited binary family for projecting the base creature tree
/// into every local-animation state.  This is deliberately independent from
/// `MdlFormatProfileV1`: native corpora contain more than one legal state-tree
/// family, and mixing their node semantics produces an ungrounded hybrid.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdlStateProjectionProfileV1 {
    /// The retail family mirrors base name/part/parent topology with generic
    /// `0x01` nodes. Animation trees contain no mesh, skin, or raw-MDX payload.
    RetailDirectCreatureType5DummyV1,
    /// The direct-creature family projects only the ordered rig into type-5
    /// local-animation states. Renderable mesh/skin leaves remain exclusively
    /// in the base tree and are deliberately absent from every state tree.
    RetailDirectCreatureType5RigOnlyV1,
    /// The historical project-owned type-0 rig-only family. The public variant
    /// name is retained for exact lineage/API compatibility; its documented
    /// runtime result belongs to the immutable candidate record and is not a
    /// correctness baseline for TileStaticV1.
    OwnedRuntimePositiveType0RigOnlyV1,
    /// The separately audited CEP R3 family keeps rigid base-mesh identities
    /// as zero-geometry `0x21` placeholders.  It is never an implicit default
    /// and requires an exact provenance binding below.
    CepRigidPlaceholderV1,
}

/// Metadata-only provenance supplied by the caller. Production code validates
/// its shape but never embeds or opens a reference witness.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlStateProjectionProvenanceV1 {
    pub schema_version: u32,
    pub source_family: String,
    pub container_sha256: String,
    pub resource_resref: String,
    pub resource_sha256: String,
}

pub(crate) fn is_well_formed_state_projection_provenance_v1(
    provenance: &MdlStateProjectionProvenanceV1,
) -> bool {
    provenance.schema_version == 1
        && !provenance.source_family.is_empty()
        && provenance.source_family.len() <= 96
        && is_lower_sha256(&provenance.container_sha256)
        && !provenance.resource_resref.is_empty()
        && provenance.resource_resref.len() <= 16
        && provenance
            .resource_resref
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && is_lower_sha256(&provenance.resource_sha256)
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlMaterialTextureBindingV1 {
    pub material_slot: u32,
    pub resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlWriterOptionsV1 {
    pub schema_version: u32,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_projection_provenance: Option<MdlStateProjectionProvenanceV1>,
    pub model_resource_resref: String,
    pub diffuse_texture_resref_by_material_slot: Vec<MdlMaterialTextureBindingV1>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdlAnimationTrackPathV1 {
    Translation,
    Rotation,
    Scale,
    Weights,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdlAnimationInterpolationV1 {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlAnimationTrackV1 {
    pub target_node_id: u32,
    pub path: MdlAnimationTrackPathV1,
    pub interpolation: MdlAnimationInterpolationV1,
    pub times_seconds: Vec<f32>,
    pub values: Vec<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlAnimationEventV1 {
    pub time_seconds: f32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlAnimationClipV1 {
    pub name: String,
    pub animation_root: String,
    pub length_seconds: f32,
    pub transition_seconds: f32,
    pub events: Vec<MdlAnimationEventV1>,
    pub tracks: Vec<MdlAnimationTrackV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlAnimationSetV1 {
    pub schema_version: u32,
    pub clips: Vec<MdlAnimationClipV1>,
}

impl MdlAnimationSetV1 {
    pub fn empty() -> Self {
        Self {
            schema_version: M4A_ANIMATION_SCHEMA_VERSION,
            clips: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlWriteError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl MdlWriteError {
    pub(crate) fn fatal(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: M4_WRITER_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for MdlWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for MdlWriteError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlRigNodeLayoutV1 {
    pub ir_node_id: u32,
    pub part_number: u32,
    pub core_offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlMeshNodeLayoutV1 {
    pub segment_id: u32,
    pub part_number: u32,
    pub core_offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlLayoutReportV1 {
    pub core_length: usize,
    pub raw_length: usize,
    pub file_length: usize,
    pub rig_nodes: Vec<MdlRigNodeLayoutV1>,
    pub mesh_nodes: Vec<MdlMeshNodeLayoutV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aabb_node: Option<MdlAabbNodeLayoutV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlAabbNodeLayoutV1 {
    pub part_number: u32,
    pub core_offset: u32,
    pub root_entry_core_offset: u32,
    pub entry_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlWriterDeviationV1 {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M4SemanticProjectionV1 {
    pub model_resource_resref: String,
    pub animation_count: usize,
    pub rig_node_count: usize,
    pub mesh_node_count: usize,
    pub triangle_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlWriterReportV1 {
    pub schema_version: u32,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_projection_provenance: Option<MdlStateProjectionProvenanceV1>,
    pub payload_sha256: String,
    pub layout: MdlLayoutReportV1,
    pub projection: M4SemanticProjectionV1,
    pub semantic_diff: Vec<String>,
    pub deviations: Vec<MdlWriterDeviationV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<MdlAnimationWriterReportV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlAnimationClipLayoutV1 {
    pub name: String,
    pub header_core_offset: u32,
    pub root_core_offset: u32,
    pub event_array_core_offset: Option<u32>,
    pub event_count: usize,
    pub track_count: usize,
    pub node_count: usize,
    pub nodes: Vec<MdlAnimationNodeLayoutV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlAnimationNodeLayoutV1 {
    /// `None` marks a derived animation node that mirrors a base mesh part.
    /// Its binary shape is selected by the report-level state projection
    /// profile; callers must not infer a universal `0x01` or `0x21` family.
    pub ir_node_id: Option<u32>,
    pub part_number: u32,
    pub core_offset: u32,
    pub children_array_core_offset: Option<u32>,
    pub controller_keys_core_offset: Option<u32>,
    pub controller_data_core_offset: Option<u32>,
    pub tracks: Vec<MdlAnimationTrackLayoutV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlAnimationTrackLayoutV1 {
    pub target_node_id: u32,
    pub path: MdlAnimationTrackPathV1,
    pub controller_type: i32,
    pub packed_byte: u8,
    pub key_core_offset: u32,
    pub row_count: usize,
    pub time_index: usize,
    pub data_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlAnimationWriterReportV1 {
    pub pointer_array_core_offset: u32,
    pub clip_count: usize,
    pub event_count: usize,
    pub track_count: usize,
    pub clips: Vec<MdlAnimationClipLayoutV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryMdlArtifactV1 {
    pub payload: Vec<u8>,
    pub inspection: InspectionReport,
    pub report: MdlWriterReportV1,
}
