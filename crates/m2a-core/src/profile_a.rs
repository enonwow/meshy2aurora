//! Deterministic Profile A geometry conversion into Aurora target space.
//!
//! M3A/M3B cover exhaustive surface-based multi-segment assignment plus
//! deterministic `RIGID` attachment and `SKIN` reference-weight transfer.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Profile A work accounting is a serialized contract and therefore cannot use
// host `usize`/pointer layout. These fixed 64-bit-layout charges are also
// conservative on wasm32 and keep native/WASM reports byte-identical.
const WORK_BOOL: u64 = 1;
const WORK_U32: u64 = 4;
const WORK_USIZE: u64 = 8;
const WORK_OPTION_U32: u64 = 8;
const WORK_REFERENCE: u64 = 8;
const WORK_VEC_HEADER: u64 = 24;
const WORK_PROFILE: u64 = 144;
const WORK_RIG_NODE: u64 = 104;
const WORK_RIG_INFLUENCE: u64 = 8;
const WORK_RIG_SEGMENT: u64 = 136;
const WORK_GATE: u64 = 152;
const WORK_GEOMETRY_INSTANCE: u64 = 72;
const WORK_MATERIAL_BINDING: u64 = 40;
const WORK_DIAGNOSTIC: u64 = 104;
const WORK_OPTION_MAT4: u64 = 68;
const WORK_SCENE_STACK_ENTRY: u64 = 76;
const WORK_BUCKET_PLAN_ENTRY: u64 = 40;
const WORK_CREATURE_SEGMENT: u64 = 160;
const WORK_VERTEX_WEIGHTS: u64 = 52;
const WORK_U32_F64_PAIR: u64 = 16;

use crate::{
    glb::{
        GlbIngestResult, IrAnimation, IrAnimationChannel, IrAnimationSampler, IrNode, IrPrimitive,
        IrTransform,
    },
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
        MdlAnimationTrackPathV1, MdlAnimationTrackV1,
    },
};

pub const PROFILE_A_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RigProvenanceKindV1 {
    Synthetic,
    Owned,
    UserProvided,
    ReferenceOnly,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigProvenanceV1 {
    pub kind: RigProvenanceKindV1,
    pub export_allowed: bool,
    pub attestations: RigProvenanceAttestationsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigProvenanceAttestationsV1 {
    pub controlled_construction: bool,
    pub no_reference_payload_copied: bool,
    pub rights_confirmed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureRigNodeV1 {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    /// Column-major affine bind matrix.
    pub bind_local_matrix: [f32; 16],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RigSegmentDeformationV1 {
    Skin,
    Rigid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigWeightInfluenceV1 {
    pub bone_node_id: u32,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureRigSegmentV1 {
    pub id: u32,
    pub name: String,
    pub deformation: RigSegmentDeformationV1,
    pub parent_node_id: u32,
    pub surface_positions: Vec<[f32; 3]>,
    pub surface_indices: Vec<u32>,
    pub allowed_bone_node_ids: Vec<u32>,
    pub reference_weights: Vec<Vec<RigWeightInfluenceV1>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureRigProfileV1 {
    pub schema_version: u32,
    pub profile_id: String,
    pub content_sha256: String,
    pub provenance: RigProvenanceV1,
    pub target_bounds: Bounds3V1,
    pub alignment_anchor: [f32; 3],
    pub nodes: Vec<CreatureRigNodeV1>,
    pub segments: Vec<CreatureRigSegmentV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bounds3V1 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileAAlignmentV1 {
    BottomCenterToProfileAnchor,
}

macro_rules! locked_policy {
    ($name:ident, $variant:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum $name {
            $variant,
        }
    };
}
locked_policy!(ProfileASourceScenePolicyV1, DefaultSceneOnly);
locked_policy!(ProfileASourceRigPolicyV1, RejectPresent);
locked_policy!(ProfileASourceAnimationPolicyV1, RejectPresent);
locked_policy!(ProfileANormalPolicyV1, RequireSource);
locked_policy!(ProfileABasisPolicyV1, GltfToAuroraXzy);
locked_policy!(ProfileAUvPolicyV1, FlipVOnce);
locked_policy!(ProfileAWindingPolicyV1, ReverseOnce);
locked_policy!(ProfileAMaterialPolicyV1, SingleSourceSlot);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileALimitsV1 {
    pub max_rig_nodes: u64,
    pub max_segments: u64,
    pub max_reference_vertices: u64,
    pub max_reference_triangles: u64,
    pub max_output_vertices: u64,
    pub max_output_indices: u64,
    pub max_distance_evaluations: u64,
    pub max_work_bytes: u64,
    pub max_diagnostics: u64,
    pub max_unique_materials: u64,
    pub triangle_warning_above: u64,
    pub triangle_blocking_above: u64,
}

impl Default for ProfileALimitsV1 {
    fn default() -> Self {
        Self {
            max_rig_nodes: 4_096,
            max_segments: 256,
            max_reference_vertices: 1_000_000,
            max_reference_triangles: 1_000_000,
            max_output_vertices: 1_000_000,
            max_output_indices: 3_000_000,
            max_distance_evaluations: 3_000_000,
            max_work_bytes: 256 * 1024 * 1024,
            max_diagnostics: 2_048,
            max_unique_materials: 1,
            triangle_warning_above: 5_000,
            triangle_blocking_above: 10_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileAOptionsV1 {
    pub schema_version: u32,
    pub source_scene_policy: ProfileASourceScenePolicyV1,
    pub source_rig_policy: ProfileASourceRigPolicyV1,
    pub source_animation_policy: ProfileASourceAnimationPolicyV1,
    pub normal_policy: ProfileANormalPolicyV1,
    pub basis_policy: ProfileABasisPolicyV1,
    pub uv_policy: ProfileAUvPolicyV1,
    pub winding_policy: ProfileAWindingPolicyV1,
    pub alignment_policy: ProfileAAlignmentV1,
    pub material_policy: ProfileAMaterialPolicyV1,
    pub weight_merge_epsilon: f32,
    pub weight_sum_tolerance: f32,
    pub bounds_tolerance_factor: f32,
    pub limits: ProfileALimitsV1,
}

impl Default for ProfileAOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: PROFILE_A_SCHEMA_VERSION,
            source_scene_policy: ProfileASourceScenePolicyV1::DefaultSceneOnly,
            source_rig_policy: ProfileASourceRigPolicyV1::RejectPresent,
            source_animation_policy: ProfileASourceAnimationPolicyV1::RejectPresent,
            normal_policy: ProfileANormalPolicyV1::RequireSource,
            basis_policy: ProfileABasisPolicyV1::GltfToAuroraXzy,
            uv_policy: ProfileAUvPolicyV1::FlipVOnce,
            winding_policy: ProfileAWindingPolicyV1::ReverseOnce,
            alignment_policy: ProfileAAlignmentV1::BottomCenterToProfileAnchor,
            material_policy: ProfileAMaterialPolicyV1::SingleSourceSlot,
            weight_merge_epsilon: 0.0,
            weight_sum_tolerance: 0.00001,
            bounds_tolerance_factor: 0.00001,
            limits: ProfileALimitsV1::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileAAnimationNodeMappingV1 {
    pub source_node_id: u32,
    pub output_rig_node_id: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileAAnimationClipMappingV1 {
    pub source_animation_id: u32,
    pub output_clip_name: String,
    pub transition_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileAAnimationMappingV1 {
    pub schema_version: u32,
    pub source_skin_id: u32,
    pub provenance: RigProvenanceV1,
    pub node_mappings: Vec<ProfileAAnimationNodeMappingV1>,
    pub clip_mappings: Vec<ProfileAAnimationClipMappingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAConversionFatalError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl ProfileAConversionFatalError {
    fn new(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: PROFILE_A_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ProfileAConversionFatalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ProfileAConversionFatalError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAAnimationFatalError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl ProfileAAnimationFatalError {
    fn new(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: PROFILE_A_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl From<ProfileAConversionFatalError> for ProfileAAnimationFatalError {
    fn from(error: ProfileAConversionFatalError) -> Self {
        Self {
            schema_version: error.schema_version,
            code: error.code,
            severity: error.severity,
            path: error.path,
            message: error.message,
        }
    }
}

impl fmt::Display for ProfileAAnimationFatalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ProfileAAnimationFatalError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAGateV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileADiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAReportSourceV1 {
    pub sha256: String,
    pub byte_length: u64,
    pub default_scene_id: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAReportRigV1 {
    pub profile_id: String,
    pub content_sha256: String,
    pub provenance_kind: RigProvenanceKindV1,
    pub export_allowed: bool,
    pub attestations: RigProvenanceAttestationsV1,
    pub all_attestations_satisfied: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAReportPoliciesV1 {
    pub basis_status: String,
    pub basis_evidence: String,
    pub asset_forward_mapping: String,
    pub orientation_parity: String,
    pub uv_evidence: String,
    pub uv_mapping: String,
    pub engine_facing_proof: String,
    pub uv_runtime_proof: String,
    pub source_scene_policy: String,
    pub alignment_policy: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileASourceSelectionReportV1 {
    pub reachable_node_count: u64,
    pub reachable_mesh_instance_count: u64,
    pub ignored_node_count: u64,
    pub ignored_mesh_count: u64,
    pub duplicated_mesh_instance_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileATransformReportV1 {
    pub basis_matrix: [f32; 16],
    pub determinant: f32,
    pub source_world_bounds: Option<Bounds3V1>,
    pub after_basis_bounds: Option<Bounds3V1>,
    pub target_bounds: Option<Bounds3V1>,
    pub scale: Option<f32>,
    pub source_bottom_center: Option<[f32; 3]>,
    pub alignment_anchor: [f32; 3],
    pub translation: Option<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialSourceBindingV1 {
    pub slot: u32,
    pub source_material_id: Option<u32>,
    pub source_material_name: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAMaterialsReportV1 {
    pub unique_used_count: u64,
    pub max_unique_materials: u64,
    pub bindings: Vec<MaterialSourceBindingV1>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAGeometryReportV1 {
    pub source_triangle_count: u64,
    pub output_triangle_count: u64,
    pub source_vertex_instance_count: u64,
    pub output_vertex_count: u64,
    pub duplicated_vertex_count: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileASegmentReportV1 {
    pub segment_id: u32,
    pub material_slot: u32,
    pub deformation: RigSegmentDeformationV1,
    pub triangle_count: u64,
    pub vertex_count: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAWeightsReportV1 {
    pub skinned_vertex_count: u64,
    pub rigid_vertex_count: u64,
    pub merged_duplicate_influence_count: u64,
    pub dropped_zero_influence_count: u64,
    pub dropped_after_top_four_count: u64,
    pub normalized_vertex_count: u64,
    pub max_influences_before: u64,
    pub max_influences_after: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAWorkReportV1 {
    pub distance_evaluations: u64,
    pub max_distance_evaluations: u64,
    pub work_bytes_peak: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAConversionReportV1 {
    pub schema_version: u32,
    pub source: ProfileAReportSourceV1,
    pub rig: ProfileAReportRigV1,
    pub policies: ProfileAReportPoliciesV1,
    pub source_selection: ProfileASourceSelectionReportV1,
    pub transform: ProfileATransformReportV1,
    pub materials: ProfileAMaterialsReportV1,
    pub geometry: ProfileAGeometryReportV1,
    pub segments: Vec<ProfileASegmentReportV1>,
    pub weights: ProfileAWeightsReportV1,
    pub work: ProfileAWorkReportV1,
    pub gates: Vec<ProfileAGateV1>,
    pub diagnostics: Vec<ProfileADiagnosticV1>,
    pub conversion_eligible: bool,
    pub creature_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAConversionOutcomeV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub report: ProfileAConversionReportV1,
    pub creature: Option<AuroraCreatureIrV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAAnimatedOutcomeV1 {
    pub base: ProfileAConversionOutcomeV1,
    pub animations: Option<MdlAnimationSetV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraCreatureIrV1 {
    pub schema_version: u32,
    pub profile_id: String,
    pub source_sha256: String,
    pub basis_status: String,
    pub engine_facing_proof: String,
    pub uv_runtime_proof: String,
    pub nodes: Vec<AuroraCreatureNodeV1>,
    pub material_source_bindings: Vec<MaterialSourceBindingV1>,
    pub segments: Vec<AuroraCreatureSegmentV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraCreatureNodeV1 {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub bind_local_matrix: [f32; 16],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraCreatureSegmentV1 {
    pub segment_id: u32,
    pub material_slot: u32,
    pub deformation: RigSegmentDeformationV1,
    pub parent_node_id: u32,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Option<Vec<[f32; 4]>>,
    pub uv0: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub weights: Vec<AuroraVertexWeightsV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraVertexWeightsV1 {
    pub bone_node_ids: [Option<u32>; 4],
    pub values: [f32; 4],
    pub influence_count: u8,
}

#[derive(Clone, Copy)]
struct Mat4([f32; 16]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceInventoryPolicy {
    RejectPresent,
    AllowMappedForM4A2,
}

#[derive(Clone, Copy)]
struct AnimationRestPose {
    translation: [f32; 3],
    rotation: [[f32; 3]; 3],
}

struct ValidatedAnimationMapping {
    animation_root: String,
    source_to_output: BTreeMap<u32, u32>,
    source_rest: BTreeMap<u32, AnimationRestPose>,
    target_rest: BTreeMap<u32, AnimationRestPose>,
    clip_mapping_index_by_source: BTreeMap<u32, usize>,
}

#[derive(Default)]
struct Counters {
    winding: u64,
    normals: u64,
    tangents: u64,
    uv: u64,
    source_vertices: u64,
    output_vertices: u64,
    source_triangles: u64,
    output_triangles: u64,
    duplicated_vertices: u64,
    distance_evaluations: u64,
    skinned_vertices: u64,
    rigid_vertices: u64,
    merged_duplicate_influences: u64,
    dropped_zero_influences: u64,
    dropped_top_four_influences: u64,
    normalized_vertices: u64,
    max_influences_before: u64,
    max_influences_after: u64,
    segment_reports: Vec<ProfileASegmentReportV1>,
    work_bytes_peak: u64,
}

/// SHA-256 of deterministic JSON serialization with `contentSha256` omitted.
pub fn canonical_profile_sha256(
    profile: &CreatureRigProfileV1,
) -> Result<String, ProfileAConversionFatalError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashView<'a> {
        schema_version: u32,
        profile_id: &'a str,
        provenance: &'a RigProvenanceV1,
        target_bounds: Bounds3V1,
        alignment_anchor: [f32; 3],
        nodes: &'a [CreatureRigNodeV1],
        segments: &'a [CreatureRigSegmentV1],
    }
    let mut normalized = profile.clone();
    normalize_profile_zeroes(&mut normalized)?;
    let bytes = serde_json::to_vec(&HashView {
        schema_version: normalized.schema_version,
        profile_id: &normalized.profile_id,
        provenance: &normalized.provenance,
        target_bounds: normalized.target_bounds,
        alignment_anchor: normalized.alignment_anchor,
        nodes: &normalized.nodes,
        segments: &normalized.segments,
    })
    .map_err(|_| {
        fatal(
            "M3A-INTERNAL-CONTRACT",
            "rig",
            "profile serialization failed",
        )
    })?;
    Ok(hex_sha256(&bytes))
}

fn normalize_profile_zeroes(
    profile: &mut CreatureRigProfileV1,
) -> Result<(), ProfileAConversionFatalError> {
    fn value(value: &mut f32) -> Result<(), ProfileAConversionFatalError> {
        if !value.is_finite() {
            return Err(fatal(
                "M3A-NONFINITE-FLOAT",
                "rig",
                "profile hash input contains non-finite float",
            ));
        }
        if *value == 0.0 {
            *value = 0.0;
        }
        Ok(())
    }
    for item in profile
        .target_bounds
        .min
        .iter_mut()
        .chain(profile.target_bounds.max.iter_mut())
        .chain(profile.alignment_anchor.iter_mut())
    {
        value(item)?;
    }
    for node in &mut profile.nodes {
        for item in &mut node.bind_local_matrix {
            value(item)?;
        }
    }
    for segment in &mut profile.segments {
        for item in segment.surface_positions.iter_mut().flatten() {
            value(item)?;
        }
        for influence in segment.reference_weights.iter_mut().flatten() {
            value(&mut influence.value)?;
        }
    }
    Ok(())
}

fn normalize_creature_zeroes(creature: &mut AuroraCreatureIrV1) {
    fn value(value: &mut f32) {
        if *value == 0.0 {
            *value = 0.0;
        }
    }
    for node in &mut creature.nodes {
        for item in &mut node.bind_local_matrix {
            value(item);
        }
    }
    for segment in &mut creature.segments {
        for item in segment
            .positions
            .iter_mut()
            .flatten()
            .chain(segment.normals.iter_mut().flatten())
            .chain(segment.uv0.iter_mut().flatten())
        {
            value(item);
        }
        if let Some(tangents) = &mut segment.tangents {
            for item in tangents.iter_mut().flatten() {
                value(item);
            }
        }
        for weights in &mut segment.weights {
            for item in &mut weights.values {
                value(item);
            }
        }
    }
}

pub fn canonical_creature_sha256(
    creature: &AuroraCreatureIrV1,
) -> Result<String, ProfileAConversionFatalError> {
    let nonfinite_nodes = creature
        .nodes
        .iter()
        .flat_map(|node| node.bind_local_matrix)
        .any(|value| !value.is_finite());
    let nonfinite_geometry = creature.segments.iter().any(|segment| {
        segment
            .positions
            .iter()
            .flatten()
            .chain(segment.normals.iter().flatten())
            .chain(segment.uv0.iter().flatten())
            .chain(segment.tangents.iter().flatten().flatten())
            .chain(
                segment
                    .weights
                    .iter()
                    .flat_map(|weights| weights.values.iter()),
            )
            .any(|value| !value.is_finite())
    });
    if nonfinite_nodes || nonfinite_geometry {
        return Err(fatal(
            "M3A-NONFINITE-FLOAT",
            "creature",
            "creature hash input contains non-finite float",
        ));
    }
    let mut normalized = creature.clone();
    normalize_creature_zeroes(&mut normalized);
    let bytes = serde_json::to_vec(&normalized).map_err(|_| {
        fatal(
            "M3A-INTERNAL-CONTRACT",
            "creature",
            "creature serialization failed",
        )
    })?;
    Ok(hex_sha256(&bytes))
}

pub fn convert_profile_a(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
) -> Result<ProfileAConversionOutcomeV1, ProfileAConversionFatalError> {
    convert_profile_a_impl(source, rig, options, SourceInventoryPolicy::RejectPresent)
}

pub fn convert_profile_a_with_animations_v1(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
) -> Result<ProfileAAnimatedOutcomeV1, ProfileAAnimationFatalError> {
    let validated = validate_animation_mapping_v1(source, rig, mapping)?;
    let base = convert_profile_a_impl(
        source,
        rig,
        profile_options,
        SourceInventoryPolicy::AllowMappedForM4A2,
    )
    .map_err(ProfileAAnimationFatalError::from)?;
    if base.creature.is_none() {
        return Ok(ProfileAAnimatedOutcomeV1 {
            base,
            animations: None,
        });
    }
    let scale = base.report.transform.scale.ok_or_else(|| {
        animation_fatal(
            "M4A-MAPPER-BASIS-INVALID",
            "base.report.transform.scale",
            "successful Profile A conversion did not expose its uniform scale",
        )
    })?;
    let animations = emit_animation_set_v1(source, mapping, &validated, scale)?;
    Ok(ProfileAAnimatedOutcomeV1 {
        base,
        animations: Some(animations),
    })
}

fn convert_profile_a_impl(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
    source_inventory_policy: SourceInventoryPolicy,
) -> Result<ProfileAConversionOutcomeV1, ProfileAConversionFatalError> {
    let base_work_bytes = validate_api(source, rig, options)?;
    let mut gates = collect_preflight_gates(source, rig, options, source_inventory_policy)?;
    let mut transform_report = empty_transform_report(rig.alignment_anchor);
    let mut counters = Counters {
        work_bytes_peak: base_work_bytes,
        ..Default::default()
    };
    let mut selection_report = ProfileASourceSelectionReportV1::default();

    if gates.iter().any(|item| item.code == "M3A-SOURCE-BLOCKED") {
        finalize_gates(&mut gates);
        ensure_diagnostic_limit(gates.len(), 0, &options.limits)?;
        return Ok(blocked_outcome(
            source,
            rig,
            options,
            gates,
            transform_report,
            counters,
            selection_report,
            Vec::new(),
            Vec::new(),
        ));
    }
    let selection = match select_default_scene(source, &options.limits, base_work_bytes) {
        Ok(selection) => {
            selection_report = selection.report.clone();
            Some(selection)
        }
        Err(error) => match *error {
            SourceSelectionError::Gate(scene_gate) => {
                push_gate_checked(&mut gates, scene_gate, &options.limits)?;
                None
            }
            SourceSelectionError::Fatal(error) => return Err(error),
        },
    };
    let selection_peak = selection
        .as_ref()
        .map_or(0, |value| value.traversal_peak_bytes);
    let (instances, instance_work_bytes, construction_peak) = if let Some(selection) = &selection {
        geometry_instances(
            source,
            &selection.worlds,
            &selection.ordered_nodes,
            &options.limits,
            selection.persistent_work_bytes,
            base_work_bytes,
        )?
    } else {
        (Vec::new(), base_work_bytes, base_work_bytes)
    };
    drop(selection);
    counters.work_bytes_peak = selection_peak.max(construction_peak);
    let primitive_seen_bytes = usize_u64(source.ir.primitives.len())
        .checked_mul(WORK_BOOL)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "primitive dedupe work byte product overflow",
            )
        })?;
    let mut primitive_seen_peak = instance_work_bytes;
    reserve_work_bytes(
        &mut primitive_seen_peak,
        primitive_seen_bytes,
        &options.limits,
    )?;
    counters.work_bytes_peak = counters.work_bytes_peak.max(primitive_seen_peak);
    let mut seen_primitive_instances = Vec::new();
    seen_primitive_instances
        .try_reserve(source.ir.primitives.len())
        .map_err(|_| {
            fatal(
                "M3A-LIMIT-EXCEEDED",
                "sourceSelection.meshInstances",
                "primitive dedupe allocation failed",
            )
        })?;
    seen_primitive_instances.resize(source.ir.primitives.len(), false);
    for instance in &instances {
        let primitive_index = usize::try_from(instance.primitive.id).map_err(|_| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.primitives",
                "primitive id does not fit host index",
            )
        })?;
        let seen = seen_primitive_instances
            .get_mut(primitive_index)
            .ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "source.ir.primitives",
                    "primitive dedupe index is missing",
                )
            })?;
        if *seen {
            counters.duplicated_vertices = checked_add(
                counters.duplicated_vertices,
                instance.primitive.positions.len(),
                "duplicated vertices",
            )?;
        }
        *seen = true;
    }
    for instance in &instances {
        if instance.primitive.normals.is_empty()
            || instance
                .primitive
                .normals
                .iter()
                .any(|normal| length_sq(*normal) <= f32::EPSILON)
            || instance
                .primitive
                .tangents
                .iter()
                .any(|tangent| length_sq([tangent[0], tangent[1], tangent[2]]) <= f32::EPSILON)
        {
            push_gate_checked(
                &mut gates,
                gate(
                    "M3A-NORMALS-REQUIRED",
                    &format!("source.ir.primitives[{}].normals", instance.primitive.id),
                    "Profile A requires source normals",
                ),
                &options.limits,
            )?;
        }
    }
    let mut first_material_key = None::<Option<u32>>;
    let mut multiple_material_keys = false;
    for instance in &instances {
        let key = instance.primitive.material_id;
        if let Some(first) = first_material_key {
            if first != key {
                multiple_material_keys = true;
            }
        } else {
            first_material_key = Some(key);
        }
    }
    if multiple_material_keys {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-MATERIAL-LIMIT",
                "sourceSelection.meshInstances",
                "unique material count exceeds Profile A guardrail",
            ),
            &options.limits,
        )?;
    }
    let selected_triangles = instances
        .iter()
        .try_fold(0usize, |sum, item| {
            sum.checked_add(item.primitive.indices.len() / 3)
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "sourceSelection.meshInstances",
                "triangle count overflow",
            )
        })?;
    if usize_u64(selected_triangles) > options.limits.triangle_blocking_above {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-TRIANGLE-BUDGET-BLOCKING",
                "sourceSelection.meshInstances",
                "triangle count exceeds Profile A blocking threshold",
            ),
            &options.limits,
        )?;
    } else if usize_u64(selected_triangles) > options.limits.triangle_warning_above {
        push_gate_checked(
            &mut gates,
            warning(
                "M3A-TRIANGLE-BUDGET-WARNING",
                "sourceSelection.meshInstances",
                "triangle count exceeds Profile A warning threshold",
            ),
            &options.limits,
        )?;
    }
    finalize_gates(&mut gates);
    drop(seen_primitive_instances);
    let material_summary = material_summary(
        source,
        &instances,
        gates.len(),
        instance_work_bytes,
        &options.limits,
    )?;
    counters.work_bytes_peak = counters
        .work_bytes_peak
        .max(material_summary.peak_work_bytes);
    let material_work_bytes = material_summary.retained_work_bytes;
    let material_bindings = material_summary.bindings;
    let diagnostics = material_summary.diagnostics;

    if gates.iter().any(|item| item.severity == "BLOCKING") {
        return Ok(blocked_outcome(
            source,
            rig,
            options,
            gates,
            transform_report,
            counters,
            selection_report,
            material_bindings,
            diagnostics,
        ));
    }

    let source_bounds = world_bounds(&instances)?;
    let basis_bounds = transform_bounds(source_bounds, basis_matrix());
    let source_height = basis_bounds.max[2] - basis_bounds.min[2];
    let target_height = rig.target_bounds.max[2] - rig.target_bounds.min[2];
    if !source_height.is_finite() || source_height <= 0.0 {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-ZERO-HEIGHT",
                "source.ir.primitives",
                "source height after basis transform must be positive",
            ),
            &options.limits,
        )?;
        finalize_gates(&mut gates);
        ensure_diagnostic_limit(gates.len(), diagnostics.len(), &options.limits)?;
        return Ok(blocked_outcome(
            source,
            rig,
            options,
            gates,
            transform_report,
            counters,
            selection_report,
            material_bindings,
            diagnostics,
        ));
    }
    let scale = target_height / source_height;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(fatal(
            "M3A-NONFINITE-FLOAT",
            "rig.targetBounds",
            "scale is not positive finite",
        ));
    }
    let scaled_basis_bounds = scale_bounds(basis_bounds, scale);
    let bottom_center = bottom_center(scaled_basis_bounds);
    let translation = sub3(rig.alignment_anchor, bottom_center);
    let conversion = Mat4::from_scale_basis_translation(scale, translation);
    let target_bounds_expected = translate_bounds(scaled_basis_bounds, translation);

    transform_report.source_world_bounds = Some(source_bounds);
    transform_report.after_basis_bounds = Some(basis_bounds);
    transform_report.scale = Some(scale);
    transform_report.source_bottom_center = Some(bottom_center);
    transform_report.translation = Some(translation);

    let rig_worlds = rig_bind_worlds(rig)?;
    let assignment_plan = plan_triangle_assignments(
        &instances,
        conversion,
        rig,
        &rig_worlds,
        &material_bindings,
        material_work_bytes,
        &mut counters,
        &options.limits,
    )?;
    counters.work_bytes_peak = counters
        .work_bytes_peak
        .max(assignment_plan.work_bytes_peak);
    for key in &assignment_plan.mixed_tangent_buckets {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-TANGENT-COVERAGE-MIXED",
                &format!("creature.segments[{},{}].tangents", key.0, key.1),
                "one output segment/material bucket cannot mix tangent presence",
            ),
            &options.limits,
        )?;
    }
    if assignment_plan.has_unreferenced_vertices {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-SEGMENT-ASSIGNMENT-FAILED",
                "sourceSelection.meshInstances",
                "source contains vertices not referenced by any assigned triangle",
            ),
            &options.limits,
        )?;
    }
    finalize_gates(&mut gates);
    ensure_diagnostic_limit(gates.len(), diagnostics.len(), &options.limits)?;
    if gates.iter().any(|item| item.severity == "BLOCKING") {
        return Ok(blocked_outcome(
            source,
            rig,
            options,
            gates,
            transform_report,
            counters,
            selection_report,
            material_bindings,
            diagnostics,
        ));
    }
    let buckets = emit_assigned_geometry(
        &instances,
        &assignment_plan,
        conversion,
        rig,
        &rig_worlds,
        &mut gates,
        &mut counters,
        &options.limits,
    )?;
    let mut segments = Vec::new();
    segments.try_reserve(buckets.len()).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "creature.segments",
            "output segment allocation failed",
        )
    })?;
    segments.extend(buckets.into_values());
    let target_bounds = bounds_from_segments_world(&segments, &rig_worlds)?;
    counters.segment_reports = segments
        .iter()
        .map(|segment| ProfileASegmentReportV1 {
            segment_id: segment.segment_id,
            material_slot: segment.material_slot,
            deformation: segment.deformation.clone(),
            triangle_count: usize_u64(segment.indices.len() / 3),
            vertex_count: usize_u64(segment.positions.len()),
        })
        .collect();
    transform_report.target_bounds = Some(target_bounds);

    let tolerance = 1.0e-5_f32 * target_height.max(1.0);
    if (height(target_bounds) - target_height).abs() > tolerance
        || !bounds_approx(target_bounds, target_bounds_expected, tolerance)
    {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-OUTPUT-BOUNDS-MISMATCH",
                "creature.bounds",
                "output bounds do not match the checked scale/alignment transform",
            ),
            &options.limits,
        )?;
    }
    finalize_gates(&mut gates);
    ensure_diagnostic_limit(gates.len(), diagnostics.len(), &options.limits)?;
    if gates.iter().any(|item| item.severity == "BLOCKING") {
        return Ok(blocked_outcome(
            source,
            rig,
            options,
            gates,
            transform_report,
            counters,
            selection_report,
            material_bindings,
            diagnostics,
        ));
    }

    let nodes = rig
        .nodes
        .iter()
        .map(|node| AuroraCreatureNodeV1 {
            id: node.id,
            name: node.name.clone(),
            parent_id: node.parent_id,
            bind_local_matrix: node.bind_local_matrix,
        })
        .collect();
    let mut creature = AuroraCreatureIrV1 {
        schema_version: PROFILE_A_SCHEMA_VERSION,
        profile_id: rig.profile_id.clone(),
        source_sha256: source.ir.source.sha256.clone(),
        basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
        engine_facing_proof: "OPEN_M6".to_owned(),
        uv_runtime_proof: "OPEN_M6".to_owned(),
        nodes,
        material_source_bindings: material_bindings.clone(),
        segments,
    };
    normalize_creature_zeroes(&mut creature);
    let creature_sha256 = canonical_creature_sha256(&creature)?;
    Ok(ProfileAConversionOutcomeV1 {
        schema_version: PROFILE_A_SCHEMA_VERSION,
        source_sha256: source.ir.source.sha256.clone(),
        report: report(
            true,
            source,
            rig,
            options,
            gates,
            transform_report,
            counters,
            selection_report,
            material_bindings,
            diagnostics,
            Some(creature_sha256),
        ),
        creature: Some(creature),
    })
}

fn animation_fatal(
    code: &str,
    path: &str,
    message: impl Into<String>,
) -> ProfileAAnimationFatalError {
    ProfileAAnimationFatalError::new(code, path, message)
}

fn validate_animation_mapping_v1(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    mapping: &ProfileAAnimationMappingV1,
) -> Result<ValidatedAnimationMapping, ProfileAAnimationFatalError> {
    if mapping.schema_version != PROFILE_A_SCHEMA_VERSION {
        return Err(animation_fatal(
            "M4A-MAPPING-SCHEMA-INVALID",
            "mapping.schemaVersion",
            "only ProfileAAnimationMappingV1 schema version 1 is supported",
        ));
    }
    if !animation_provenance_allowed(&mapping.provenance) {
        return Err(animation_fatal(
            "M4A-MAPPER-PROVENANCE-FORBIDDEN",
            "mapping.provenance",
            "mapping provenance policy forbids export",
        ));
    }
    if source.ir.skins.len() != 1 || source.ir.skins[0].id != mapping.source_skin_id {
        return Err(animation_fatal(
            "M4A-MAPPER-SKIN-INVALID",
            "mapping.sourceSkinId",
            "M4A2 v1 requires exactly one source skin matching sourceSkinId",
        ));
    }
    let skin = &source.ir.skins[0];
    let skeleton_root = skin.skeleton_root_node_id.ok_or_else(|| {
        animation_fatal(
            "M4A-MAPPER-SKIN-INVALID",
            "source.ir.skins[0].skeletonRootNodeId",
            "the selected source skin must declare a skeleton root",
        )
    })?;

    let mut source_nodes = BTreeMap::<u32, &IrNode>::new();
    for (index, node) in source.ir.nodes.iter().enumerate() {
        if source_nodes.insert(node.id, node).is_some() {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("source.ir.nodes[{index}].id"),
                "source node ids must be unique",
            ));
        }
        if node.parent_ids.len() > 1 {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("source.ir.nodes[{index}].parentIds"),
                "M4A2 v1 requires a single-parent source hierarchy",
            ));
        }
    }
    let mut rig_nodes = BTreeMap::new();
    for (index, node) in rig.nodes.iter().enumerate() {
        if rig_nodes.insert(node.id, node).is_some() {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("rig.nodes[{index}].id"),
                "output rig node ids must be unique",
            ));
        }
    }
    let roots = rig
        .nodes
        .iter()
        .filter(|node| node.parent_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(animation_fatal(
            "M4A-ANIMROOT-INVALID",
            "rig.nodes",
            "M4A2 requires exactly one output rig root",
        ));
    }
    validate_animation_name(
        &roots[0].name,
        63,
        "M4A-ANIMROOT-INVALID",
        "rig.nodes.root.name",
    )?;

    let mut clip_mapping_index_by_source = BTreeMap::new();
    let mut folded_clip_names = BTreeSet::new();
    for (index, clip) in mapping.clip_mappings.iter().enumerate() {
        if clip_mapping_index_by_source
            .insert(clip.source_animation_id, index)
            .is_some()
        {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("mapping.clipMappings[{index}].sourceAnimationId"),
                "each source animation must have exactly one clip mapping",
            ));
        }
        if !source
            .ir
            .animations
            .iter()
            .any(|animation| animation.id == clip.source_animation_id)
        {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                &format!("mapping.clipMappings[{index}].sourceAnimationId"),
                "clip mapping references a missing source animation",
            ));
        }
        validate_animation_name(
            &clip.output_clip_name,
            63,
            "M4A-ANIMATION-NAME-INVALID",
            &format!("mapping.clipMappings[{index}].outputClipName"),
        )?;
        if !folded_clip_names.insert(clip.output_clip_name.to_ascii_lowercase()) {
            return Err(animation_fatal(
                "M4A-ANIMATION-NAME-INVALID",
                &format!("mapping.clipMappings[{index}].outputClipName"),
                "output clip names must be unique after ASCII case-fold",
            ));
        }
        if !clip.transition_seconds.is_finite() || clip.transition_seconds < 0.0 {
            return Err(animation_fatal(
                "M4A-TRANSITION-INVALID",
                &format!("mapping.clipMappings[{index}].transitionSeconds"),
                "transition must be non-negative and finite",
            ));
        }
    }
    let mut animation_ids = BTreeSet::new();
    for (index, animation) in source.ir.animations.iter().enumerate() {
        if !animation_ids.insert(animation.id) {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("source.ir.animations[{index}].id"),
                "source animation ids must be unique",
            ));
        }
        if !clip_mapping_index_by_source.contains_key(&animation.id) {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                &format!("source.ir.animations[{index}].id"),
                "every source animation requires exactly one clip mapping",
            ));
        }
        validate_source_animation_v1(animation, index)?;
    }

    let mut source_to_output = BTreeMap::new();
    let mut used_outputs = BTreeSet::new();
    let mut source_rest = BTreeMap::new();
    let mut target_rest = BTreeMap::new();
    for (index, entry) in mapping.node_mappings.iter().enumerate() {
        if source_to_output
            .insert(entry.source_node_id, entry.output_rig_node_id)
            .is_some()
        {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("mapping.nodeMappings[{index}].sourceNodeId"),
                "source nodes may be mapped only once",
            ));
        }
        if !used_outputs.insert(entry.output_rig_node_id) {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("mapping.nodeMappings[{index}].outputRigNodeId"),
                "node mapping must be injective",
            ));
        }
        let source_node = source_nodes.get(&entry.source_node_id).ok_or_else(|| {
            animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                &format!("mapping.nodeMappings[{index}].sourceNodeId"),
                "mapped source node does not exist",
            )
        })?;
        let target_node = rig_nodes.get(&entry.output_rig_node_id).ok_or_else(|| {
            animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                &format!("mapping.nodeMappings[{index}].outputRigNodeId"),
                "mapped output rig node does not exist",
            )
        })?;
        source_rest.insert(entry.source_node_id, source_rest_pose(source_node, index)?);
        target_rest.insert(
            entry.output_rig_node_id,
            target_rest_pose(target_node.bind_local_matrix, index)?,
        );
    }

    let mut required = BTreeSet::new();
    required.insert(skeleton_root);
    for &joint in &skin.joint_node_ids {
        required.insert(joint);
    }
    for animation in &source.ir.animations {
        for channel in &animation.channels {
            required.insert(channel.target_node_id);
        }
    }
    required.extend(source_to_output.keys().copied());
    let mut pending = required.iter().copied().collect::<Vec<_>>();
    let mut expanded = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !expanded.insert(node_id) {
            continue;
        }
        let node = source_nodes.get(&node_id).ok_or_else(|| {
            animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                "mapping.nodeMappings",
                format!("required source node {node_id} does not exist"),
            )
        })?;
        if let Some(&parent_id) = node.parent_ids.first()
            && required.insert(parent_id)
        {
            pending.push(parent_id);
        }
    }
    for node_id in required {
        if !source_to_output.contains_key(&node_id) {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                "mapping.nodeMappings",
                format!("required source node {node_id} has no explicit output mapping"),
            ));
        }
    }
    for (&source_id, &output_id) in &source_to_output {
        let source_node = source_nodes[&source_id];
        let target_node = rig_nodes[&output_id];
        let expected_parent = source_node
            .parent_ids
            .first()
            .map(|parent| source_to_output.get(parent).copied())
            .transpose_option()
            .ok_or_else(|| {
                animation_fatal(
                    "M4A-MAPPER-TARGET-MISSING",
                    "mapping.nodeMappings",
                    "mapped source ancestor has no explicit output mapping",
                )
            })?;
        if target_node.parent_id != expected_parent {
            return Err(animation_fatal(
                "M4A-MAPPER-BASIS-INVALID",
                "mapping.nodeMappings",
                "node mapping must preserve the complete parent hierarchy",
            ));
        }
    }

    Ok(ValidatedAnimationMapping {
        animation_root: roots[0].name.clone(),
        source_to_output,
        source_rest,
        target_rest,
        clip_mapping_index_by_source,
    })
}

trait TransposeOption<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}

impl<T> TransposeOption<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            None => Some(None),
            Some(Some(value)) => Some(Some(value)),
            Some(None) => None,
        }
    }
}

fn animation_provenance_allowed(provenance: &RigProvenanceV1) -> bool {
    matches!(
        provenance.kind,
        RigProvenanceKindV1::Synthetic
            | RigProvenanceKindV1::Owned
            | RigProvenanceKindV1::UserProvided
    ) && provenance.export_allowed
        && provenance.attestations.controlled_construction
        && provenance.attestations.no_reference_payload_copied
        && provenance.attestations.rights_confirmed
}

fn validate_animation_name(
    value: &str,
    max_bytes: usize,
    code: &str,
    path: &str,
) -> Result<(), ProfileAAnimationFatalError> {
    if value.is_empty()
        || value.len() > max_bytes
        || !value.is_ascii()
        || value.bytes().any(|byte| byte == 0)
    {
        return Err(animation_fatal(
            code,
            path,
            format!("value must be a non-empty ASCII C string of at most {max_bytes} bytes"),
        ));
    }
    Ok(())
}

fn validate_source_animation_v1(
    animation: &IrAnimation,
    animation_index: usize,
) -> Result<(), ProfileAAnimationFatalError> {
    let clip_path = format!("source.ir.animations[{animation_index}]");
    if !animation.duration_seconds.is_finite() || animation.duration_seconds <= 0.0 {
        return Err(animation_fatal(
            "M4A-CLIP-LENGTH-INVALID",
            &format!("{clip_path}.durationSeconds"),
            "source animation duration must be positive and finite",
        ));
    }
    let mut sampler_ids = BTreeSet::new();
    for (index, sampler) in animation.samplers.iter().enumerate() {
        if !sampler_ids.insert(sampler.id) {
            return Err(animation_fatal(
                "M4A-MAPPER-TARGET-AMBIGUOUS",
                &format!("{clip_path}.samplers[{index}].id"),
                "source animation sampler ids must be unique",
            ));
        }
    }
    let mut seen_tracks = BTreeSet::new();
    for (channel_index, channel) in animation.channels.iter().enumerate() {
        let channel_path = format!("{clip_path}.channels[{channel_index}]");
        let arity = match channel.target_path.as_str() {
            "TRANSLATION" | "translation" => 3,
            "ROTATION" | "rotation" => 4,
            _ => {
                return Err(animation_fatal(
                    "M4A-TRACK-PATH-UNSUPPORTED",
                    &format!("{channel_path}.targetPath"),
                    "M4A2 v1 supports translation and rotation channels only",
                ));
            }
        };
        let folded_path = channel.target_path.to_ascii_lowercase();
        if !seen_tracks.insert((channel.target_node_id, folded_path)) {
            return Err(animation_fatal(
                "M4A-TRACK-DUPLICATE",
                &channel_path,
                "a clip may contain only one channel per target node and path",
            ));
        }
        let sampler = source_animation_sampler(animation, channel).ok_or_else(|| {
            animation_fatal(
                "M4A-MAPPER-TARGET-MISSING",
                &format!("{channel_path}.samplerId"),
                "animation channel references a missing sampler",
            )
        })?;
        if sampler.interpolation != "LINEAR" {
            return Err(animation_fatal(
                "M4A-INTERPOLATION-UNSUPPORTED",
                &format!("{channel_path}.interpolation"),
                "M4A2 v1 supports LINEAR interpolation only",
            ));
        }
        if sampler.input_times_seconds.is_empty()
            || sampler.input_times_seconds.len() > usize::from(u16::MAX)
        {
            return Err(animation_fatal(
                "M4A-CONTROLLER-U16-OVERFLOW",
                &format!("{channel_path}.timesSeconds"),
                "track row count must fit nonzero u16",
            ));
        }
        let expected_type = if arity == 3 { "VEC3" } else { "VEC4" };
        if sampler.output_accessor_type != expected_type
            || sampler.output_values.len()
                != sampler.input_times_seconds.len().saturating_mul(arity)
        {
            return Err(animation_fatal(
                "M4A-TRACK-ARITY-INVALID",
                &format!("{channel_path}.values"),
                "track values must match time rows and path arity",
            ));
        }
        for (time_index, &time) in sampler.input_times_seconds.iter().enumerate() {
            if !time.is_finite()
                || (time_index > 0 && time <= sampler.input_times_seconds[time_index - 1])
            {
                return Err(animation_fatal(
                    "M4A-TRACK-TIME-NOT-STRICT",
                    &format!("{channel_path}.timesSeconds[{time_index}]"),
                    "track times must be finite and strictly increasing",
                ));
            }
            if time < 0.0 || time > animation.duration_seconds {
                return Err(animation_fatal(
                    "M4A-TRACK-TIME-OOB",
                    &format!("{channel_path}.timesSeconds[{time_index}]"),
                    "track time must be inside the source animation duration",
                ));
            }
        }
        if sampler.output_values.iter().any(|value| !value.is_finite()) {
            return Err(animation_fatal(
                "M4A-TRACK-VALUE-NONFINITE",
                &format!("{channel_path}.values"),
                "track values must be finite",
            ));
        }
        if arity == 4 {
            for (row, values) in sampler.output_values.chunks_exact(4).enumerate() {
                canonical_animation_quaternion(
                    [values[0], values[1], values[2], values[3]],
                    &format!("{channel_path}.values[{row}]"),
                )?;
            }
        }
    }
    Ok(())
}

fn source_animation_sampler<'a>(
    animation: &'a IrAnimation,
    channel: &IrAnimationChannel,
) -> Option<&'a IrAnimationSampler> {
    animation
        .samplers
        .iter()
        .find(|sampler| sampler.id == channel.sampler_id)
}

fn source_rest_pose(
    node: &IrNode,
    mapping_index: usize,
) -> Result<AnimationRestPose, ProfileAAnimationFatalError> {
    let path = format!("mapping.nodeMappings[{mapping_index}].sourceNodeId");
    if node.transform.kind != "TRS" || node.transform.matrix.is_some() {
        return Err(animation_fatal(
            "M4A-MAPPER-BASIS-INVALID",
            &path,
            "mapped source nodes and ancestors must use TRS transforms",
        ));
    }
    let translation = node.transform.translation.unwrap_or([0.0; 3]);
    let scale = node.transform.scale.unwrap_or([1.0; 3]);
    if translation.iter().any(|value| !value.is_finite())
        || scale
            .iter()
            .any(|value| !value.is_finite() || (*value - 1.0).abs() > 1.0e-5)
    {
        return Err(animation_fatal(
            "M4A-MAPPER-BASIS-INVALID",
            &path,
            "mapped source TRS must be finite and have static unit scale",
        ));
    }
    let quaternion = canonical_animation_quaternion(
        node.transform.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]),
        &path,
    )?;
    Ok(AnimationRestPose {
        translation,
        rotation: quaternion_to_matrix3(quaternion),
    })
}

fn target_rest_pose(
    matrix: [f32; 16],
    mapping_index: usize,
) -> Result<AnimationRestPose, ProfileAAnimationFatalError> {
    let path = format!("mapping.nodeMappings[{mapping_index}].outputRigNodeId");
    let matrix = Mat4(matrix);
    if !matrix.is_finite()
        || matrix.get(3, 0).abs() > 1.0e-5
        || matrix.get(3, 1).abs() > 1.0e-5
        || matrix.get(3, 2).abs() > 1.0e-5
        || (matrix.get(3, 3) - 1.0).abs() > 1.0e-5
    {
        return Err(animation_fatal(
            "M4A-MAPPER-BASIS-INVALID",
            &path,
            "mapped target bind must be a finite affine rigid transform",
        ));
    }
    let rotation = matrix.linear();
    let gram = animation_mul3(transpose3(rotation), rotation);
    for (row, values) in gram.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            let expected = if row == column { 1.0 } else { 0.0 };
            if (*value - expected).abs() > 1.0e-4 {
                return Err(animation_fatal(
                    "M4A-MAPPER-BASIS-INVALID",
                    &path,
                    "mapped target bind may not contain scale or shear",
                ));
            }
        }
    }
    if (determinant3(rotation) - 1.0).abs() > 1.0e-4 {
        return Err(animation_fatal(
            "M4A-MAPPER-BASIS-INVALID",
            &path,
            "mapped target bind rotation must be proper and rigid",
        ));
    }
    Ok(AnimationRestPose {
        translation: [matrix.get(0, 3), matrix.get(1, 3), matrix.get(2, 3)],
        rotation,
    })
}

fn emit_animation_set_v1(
    source: &GlbIngestResult,
    mapping: &ProfileAAnimationMappingV1,
    validated: &ValidatedAnimationMapping,
    scale: f32,
) -> Result<MdlAnimationSetV1, ProfileAAnimationFatalError> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(animation_fatal(
            "M4A-MAPPER-BASIS-INVALID",
            "base.report.transform.scale",
            "Profile A animation scale must be positive and finite",
        ));
    }
    let mut clips = Vec::new();
    clips.try_reserve(source.ir.animations.len()).map_err(|_| {
        animation_fatal(
            "M4A-LAYOUT-OVERFLOW",
            "animationSet.clips",
            "animation clip allocation failed",
        )
    })?;
    for (animation_index, animation) in source.ir.animations.iter().enumerate() {
        let clip_index = validated.clip_mapping_index_by_source[&animation.id];
        let clip_mapping = &mapping.clip_mappings[clip_index];
        let mut tracks = Vec::new();
        tracks.try_reserve(animation.channels.len()).map_err(|_| {
            animation_fatal(
                "M4A-LAYOUT-OVERFLOW",
                "animationSet.clips.tracks",
                "animation track allocation failed",
            )
        })?;
        for (channel_index, channel) in animation.channels.iter().enumerate() {
            let sampler = source_animation_sampler(animation, channel).ok_or_else(|| {
                animation_fatal(
                    "M4A-MAPPER-TARGET-MISSING",
                    "source.ir.animations.channels.samplerId",
                    "validated animation sampler is missing",
                )
            })?;
            let output_id = validated.source_to_output[&channel.target_node_id];
            let source_rest = validated.source_rest[&channel.target_node_id];
            let target_rest = validated.target_rest[&output_id];
            let source_rest_basis = animation_basis_conjugate(source_rest.rotation);
            let correction = animation_mul3(target_rest.rotation, transpose3(source_rest_basis));
            let (path, values) = match channel.target_path.to_ascii_lowercase().as_str() {
                "translation" => {
                    let rows = sampler
                        .output_values
                        .chunks_exact(3)
                        .enumerate()
                        .map(|(row_index, row)| {
                            let delta = [
                                row[0] - source_rest.translation[0],
                                row[1] - source_rest.translation[1],
                                row[2] - source_rest.translation[2],
                            ];
                            let basis_delta = animation_basis_vector(delta);
                            let mapped = mul3(
                                correction,
                                [
                                    basis_delta[0] * scale,
                                    basis_delta[1] * scale,
                                    basis_delta[2] * scale,
                                ],
                            );
                            let output = [
                                canonical_animation_zero(target_rest.translation[0] + mapped[0]),
                                canonical_animation_zero(target_rest.translation[1] + mapped[1]),
                                canonical_animation_zero(target_rest.translation[2] + mapped[2]),
                            ];
                            if output.iter().any(|value| !value.is_finite()) {
                                return Err(animation_fatal(
                                    "M4A-TRACK-VALUE-NONFINITE",
                                    &format!(
                                        "animationSet.clips[{animation_index}].tracks[{channel_index}].values[{row_index}]"
                                    ),
                                    "retargeted translation row must contain only finite values",
                                ));
                            }
                            Ok(output.to_vec())
                        })
                        .collect::<Result<Vec<_>, ProfileAAnimationFatalError>>()?;
                    (MdlAnimationTrackPathV1::Translation, rows)
                }
                "rotation" => {
                    let rows = sampler
                        .output_values
                        .chunks_exact(4)
                        .enumerate()
                        .map(|(row_index, row)| {
                            let source_q = canonical_animation_quaternion(
                                [row[0], row[1], row[2], row[3]],
                                &format!("source.ir.animations.channels.values[{row_index}]"),
                            )?;
                            let source_key_basis =
                                animation_basis_conjugate(quaternion_to_matrix3(source_q));
                            let output_rotation = animation_mul3(correction, source_key_basis);
                            let q = matrix3_to_canonical_quaternion(
                                output_rotation,
                                &format!("animationSet.tracks.values[{row_index}]"),
                            )?;
                            Ok(vec![q[0], q[1], q[2], q[3]])
                        })
                        .collect::<Result<Vec<_>, ProfileAAnimationFatalError>>()?;
                    (MdlAnimationTrackPathV1::Rotation, rows)
                }
                _ => unreachable!("validated M4A2 channel path"),
            };
            tracks.push(MdlAnimationTrackV1 {
                target_node_id: output_id,
                path,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: sampler.input_times_seconds.clone(),
                values,
            });
        }
        clips.push(MdlAnimationClipV1 {
            name: clip_mapping.output_clip_name.clone(),
            animation_root: validated.animation_root.clone(),
            length_seconds: animation.duration_seconds,
            transition_seconds: clip_mapping.transition_seconds,
            events: Vec::new(),
            tracks,
        });
    }
    let mut set = MdlAnimationSetV1::empty();
    set.clips = clips;
    Ok(set)
}

fn canonical_animation_quaternion(
    mut quaternion: [f32; 4],
    path: &str,
) -> Result<[f32; 4], ProfileAAnimationFatalError> {
    let norm = quaternion
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm <= f64::from(f32::EPSILON) {
        return Err(animation_fatal(
            "M4A-QUATERNION-INVALID",
            path,
            "rotation key must be a finite nonzero XYZW quaternion",
        ));
    }
    for value in &mut quaternion {
        *value = canonical_animation_zero((f64::from(*value) / norm) as f32);
    }
    let negate = quaternion[3] < 0.0
        || (quaternion[3] == 0.0
            && quaternion[..3]
                .iter()
                .copied()
                .find(|value| *value != 0.0)
                .is_some_and(|value| value < 0.0));
    if negate {
        for value in &mut quaternion {
            *value = canonical_animation_zero(-*value);
        }
    }
    Ok(quaternion)
}

fn canonical_animation_zero(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

fn quaternion_to_matrix3(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

fn matrix3_to_canonical_quaternion(
    matrix: [[f32; 3]; 3],
    path: &str,
) -> Result<[f32; 4], ProfileAAnimationFatalError> {
    let trace = matrix[0][0] + matrix[1][1] + matrix[2][2];
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [
            (matrix[2][1] - matrix[1][2]) / s,
            (matrix[0][2] - matrix[2][0]) / s,
            (matrix[1][0] - matrix[0][1]) / s,
            0.25 * s,
        ]
    } else if matrix[0][0] > matrix[1][1] && matrix[0][0] > matrix[2][2] {
        let s = (1.0 + matrix[0][0] - matrix[1][1] - matrix[2][2]).sqrt() * 2.0;
        [
            0.25 * s,
            (matrix[0][1] + matrix[1][0]) / s,
            (matrix[0][2] + matrix[2][0]) / s,
            (matrix[2][1] - matrix[1][2]) / s,
        ]
    } else if matrix[1][1] > matrix[2][2] {
        let s = (1.0 + matrix[1][1] - matrix[0][0] - matrix[2][2]).sqrt() * 2.0;
        [
            (matrix[0][1] + matrix[1][0]) / s,
            0.25 * s,
            (matrix[1][2] + matrix[2][1]) / s,
            (matrix[0][2] - matrix[2][0]) / s,
        ]
    } else {
        let s = (1.0 + matrix[2][2] - matrix[0][0] - matrix[1][1]).sqrt() * 2.0;
        [
            (matrix[0][2] + matrix[2][0]) / s,
            (matrix[1][2] + matrix[2][1]) / s,
            0.25 * s,
            (matrix[1][0] - matrix[0][1]) / s,
        ]
    };
    canonical_animation_quaternion(q, path)
}

fn animation_basis_vector(value: [f32; 3]) -> [f32; 3] {
    [value[0], value[2], value[1]]
}

fn animation_basis_conjugate(matrix: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let basis = [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]];
    animation_mul3(animation_mul3(basis, matrix), basis)
}

fn animation_mul3(left: [[f32; 3]; 3], right: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut result = [[0.0; 3]; 3];
    for (row, result_row) in result.iter_mut().enumerate() {
        for (column, value) in result_row.iter_mut().enumerate() {
            *value = (0..3).map(|k| left[row][k] * right[k][column]).sum();
        }
    }
    result
}

fn validate_api(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
) -> Result<u64, ProfileAConversionFatalError> {
    validate_source_contract(source)?;
    if rig.schema_version != PROFILE_A_SCHEMA_VERSION {
        return Err(fatal(
            "M3A-PROFILE-SCHEMA-UNSUPPORTED",
            "rig.schemaVersion",
            "only schema version 1 is supported",
        ));
    }
    if options.schema_version != PROFILE_A_SCHEMA_VERSION {
        return Err(fatal(
            "M3A-OPTIONS-INVALID",
            "options.schemaVersion",
            "only options schema version 1 is supported",
        ));
    }
    if !logical_label(&rig.profile_id) {
        return Err(fatal(
            "M3A-PROFILE-JSON-INVALID",
            "rig.profileId",
            "profile id must be a non-path logical name",
        ));
    }
    validate_options(options)?;
    let base_work_bytes = estimate_auxiliary_work_bytes(source, rig)?;
    if base_work_bytes > options.limits.max_work_bytes {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "workBytes",
            "M3 auxiliary/hash work exceeds work byte limit",
        ));
    }
    if !is_lower_hex_sha256(&rig.content_sha256)
        || canonical_profile_sha256(rig)? != rig.content_sha256
    {
        return Err(fatal(
            "M3A-PROFILE-HASH-MISMATCH",
            "rig.contentSha256",
            "rig profile hash does not match canonical profile content",
        ));
    }
    validate_bounds(rig.target_bounds, "rig.targetBounds")?;
    finite3(rig.alignment_anchor, "rig.alignmentAnchor")?;
    if (0..3).any(|axis| {
        rig.alignment_anchor[axis] < rig.target_bounds.min[axis]
            || rig.alignment_anchor[axis] > rig.target_bounds.max[axis]
    }) {
        return Err(fatal(
            "M3A-PROFILE-BOUNDS-INVALID",
            "rig.alignmentAnchor",
            "alignment anchor must lie inside target bounds",
        ));
    }
    validate_rig(rig, &options.limits, options.bounds_tolerance_factor)?;
    Ok(base_work_bytes)
}

fn estimate_auxiliary_work_bytes(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
) -> Result<u64, ProfileAConversionFatalError> {
    let mut profile_bytes = WORK_PROFILE;
    profile_bytes = profile_bytes
        .checked_add(usize_u64(rig.profile_id.len()))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "profile bytes overflow",
            )
        })?;
    for node in &rig.nodes {
        profile_bytes = profile_bytes
            .checked_add(WORK_RIG_NODE)
            .and_then(|value| value.checked_add(usize_u64(node.name.len())))
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "rig node bytes overflow",
                )
            })?;
    }
    for segment in &rig.segments {
        let surface = usize_u64(segment.surface_positions.len())
            .checked_mul(12)
            .and_then(|value| {
                value.checked_add(usize_u64(segment.surface_indices.len()).checked_mul(4)?)
            })
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "rig surface bytes overflow",
                )
            })?;
        let weights = segment
            .reference_weights
            .iter()
            .try_fold(0u64, |sum, row| {
                sum.checked_add(usize_u64(row.len()).checked_mul(WORK_RIG_INFLUENCE)?)
            })
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "rig weight bytes overflow",
                )
            })?;
        profile_bytes = profile_bytes
            .checked_add(WORK_RIG_SEGMENT)
            .and_then(|value| value.checked_add(usize_u64(segment.name.len())))
            .and_then(|value| value.checked_add(surface))
            .and_then(|value| value.checked_add(weights))
            .and_then(|value| {
                value.checked_add(usize_u64(segment.allowed_bone_node_ids.len()).checked_mul(4)?)
            })
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "rig segment bytes overflow",
                )
            })?;
    }
    let profile_hash_peak = profile_bytes.checked_mul(7).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "canonical profile hash estimate overflow",
        )
    })?;
    let total_allowed = rig
        .segments
        .iter()
        .try_fold(0u64, |sum, segment| {
            sum.checked_add(usize_u64(segment.allowed_bone_node_ids.len()))
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "allowed bone count overflow",
            )
        })?;
    let rig_aux = usize_u64(rig.nodes.len())
        .checked_mul(512)
        .and_then(|value| value.checked_add(usize_u64(rig.segments.len()).checked_mul(128)?))
        .and_then(|value| value.checked_add(total_allowed.checked_mul(64)?))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "rig auxiliary estimate overflow",
            )
        })?;
    let gate_bytes = source
        .report
        .gates
        .iter()
        .try_fold(0u64, |sum, item| {
            let strings = item
                .code
                .len()
                .checked_add(item.path.len())?
                .checked_add(item.expected.len())?
                .checked_add(item.actual.len())?
                .checked_add(item.message.len())?;
            sum.checked_add(WORK_GATE.checked_add(usize_u64(strings))?)
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "gate estimate overflow",
            )
        })?;
    let source_dense = usize_u64(source.ir.primitives.len())
        .checked_add(
            usize_u64(source.ir.materials.len())
                .checked_mul(2)
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTEGER-OVERFLOW",
                        "workBytes",
                        "source dense estimate overflow",
                    )
                })?,
        )
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "source auxiliary estimate overflow",
            )
        })?;
    let fixed_report_allowance = 12u64.checked_mul(512).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "report allowance overflow",
        )
    })?;
    profile_hash_peak
        .checked_add(rig_aux)
        .and_then(|value| value.checked_add(gate_bytes))
        .and_then(|value| value.checked_add(source_dense))
        .and_then(|value| value.checked_add(fixed_report_allowance))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "M3 auxiliary estimate overflow",
            )
        })
}

fn validate_source_contract(source: &GlbIngestResult) -> Result<(), ProfileAConversionFatalError> {
    if source.schema_version != 1
        || source.ir.schema_version != 1
        || source.report.schema_version != 1
        || source.ir.source.format != "GLB_2_0"
        || source.report.format != "GLB_2_0"
        || source.ir.source.sha256 != source.report.input.sha256
        || source.ir.source.byte_length != source.report.input.byte_length
        || !is_lower_hex_sha256(&source.ir.source.sha256)
        || source.report.conversion_eligible
            == source
                .report
                .gates
                .iter()
                .any(|gate| gate.severity == "BLOCKING")
        || source.ir.coordinate_space != source.report.coordinate_policy
        || source.ir.coordinate_space != crate::glb::CoordinatePolicy::default()
    {
        return Err(source_mismatch(
            "source",
            "M2 schema, format, identity, gate summary, or coordinate policy is inconsistent",
        ));
    }
    let vertices = source
        .ir
        .primitives
        .iter()
        .try_fold(0usize, |sum, primitive| {
            sum.checked_add(primitive.positions.len())
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.primitives",
                "source vertex count overflow",
            )
        })?;
    let indices = source
        .ir
        .primitives
        .iter()
        .try_fold(0usize, |sum, primitive| {
            sum.checked_add(primitive.indices.len())
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.primitives",
                "source index count overflow",
            )
        })?;
    let triangles = source
        .ir
        .primitives
        .iter()
        .try_fold(0usize, |sum, primitive| {
            sum.checked_add(if primitive.topology == "TRIANGLES" {
                primitive.indices.len() / 3
            } else {
                0
            })
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.primitives",
                "source triangle count overflow",
            )
        })?;
    let inventory = &source.report.inventory;
    if source.report.statistics.vertex_count != vertices
        || source.report.statistics.index_count != indices
        || source.report.statistics.triangle_count != triangles
        || inventory.scene_count != source.ir.scenes.len()
        || inventory.node_count != source.ir.nodes.len()
        || inventory.mesh_count != source.ir.meshes.len()
        || inventory.primitive_count != source.ir.primitives.len()
        || inventory.material_count != source.ir.materials.len()
        || inventory.skin_count != source.ir.skins.len()
        || inventory.animation_count != source.ir.animations.len()
    {
        return Err(source_mismatch(
            "source",
            "M2 report inventory/statistics differ from IR",
        ));
    }
    let indexed = |index: usize, id: u32| u32::try_from(index).ok() == Some(id);
    let contiguous = source
        .ir
        .scenes
        .iter()
        .enumerate()
        .all(|(index, item)| indexed(index, item.id))
        && source
            .ir
            .nodes
            .iter()
            .enumerate()
            .all(|(index, item)| indexed(index, item.id))
        && source
            .ir
            .meshes
            .iter()
            .enumerate()
            .all(|(index, item)| indexed(index, item.id))
        && source
            .ir
            .primitives
            .iter()
            .enumerate()
            .all(|(index, item)| indexed(index, item.id))
        && source
            .ir
            .materials
            .iter()
            .enumerate()
            .all(|(index, item)| indexed(index, item.id));
    if !contiguous {
        return Err(source_mismatch(
            "source.ir",
            "M2 IDs must equal their source array indices",
        ));
    }
    let scene_ref = |id: u32| {
        usize::try_from(id)
            .ok()
            .and_then(|index| source.ir.scenes.get(index))
            .filter(|item| item.id == id)
    };
    let node_ref = |id: u32| {
        usize::try_from(id)
            .ok()
            .and_then(|index| source.ir.nodes.get(index))
            .filter(|item| item.id == id)
    };
    let mesh_ref = |id: u32| {
        usize::try_from(id)
            .ok()
            .and_then(|index| source.ir.meshes.get(index))
            .filter(|item| item.id == id)
    };
    let primitive_ref = |id: u32| {
        usize::try_from(id)
            .ok()
            .and_then(|index| source.ir.primitives.get(index))
            .filter(|item| item.id == id)
    };
    let material_ref = |id: u32| {
        usize::try_from(id)
            .ok()
            .and_then(|index| source.ir.materials.get(index))
            .filter(|item| item.id == id)
    };
    if source
        .ir
        .default_scene_id
        .is_some_and(|id| scene_ref(id).is_none())
        || source
            .ir
            .scenes
            .iter()
            .flat_map(|scene| &scene.root_node_ids)
            .any(|id| node_ref(*id).is_none())
    {
        return Err(source_mismatch(
            "source.ir.scenes",
            "scene references are invalid",
        ));
    }
    for node in &source.ir.nodes {
        if node.child_ids.iter().any(|id| node_ref(*id).is_none())
            || node.parent_ids.iter().any(|id| node_ref(*id).is_none())
            || node.mesh_id.is_some_and(|id| mesh_ref(id).is_none())
            || matrix_from_transform(&node.transform).is_err()
        {
            return Err(source_mismatch(
                "source.ir.nodes",
                "node references or transforms are invalid",
            ));
        }
        for child in &node.child_ids {
            let child_node = node_ref(*child)
                .ok_or_else(|| source_mismatch("source.ir.nodes.childIds", "child is missing"))?;
            if !child_node.parent_ids.contains(&node.id) {
                return Err(source_mismatch(
                    "source.ir.nodes",
                    "child/parent relations disagree",
                ));
            }
        }
        for parent in &node.parent_ids {
            let parent_node = node_ref(*parent)
                .ok_or_else(|| source_mismatch("source.ir.nodes.parentIds", "parent is missing"))?;
            if !parent_node.child_ids.contains(&node.id) {
                return Err(source_mismatch(
                    "source.ir.nodes",
                    "parent/child relations disagree",
                ));
            }
        }
    }
    for mesh in &source.ir.meshes {
        if mesh
            .primitive_ids
            .iter()
            .any(|id| primitive_ref(*id).is_none())
        {
            return Err(source_mismatch(
                "source.ir.meshes.primitiveIds",
                "primitive reference is missing or duplicated",
            ));
        }
        if mesh.primitive_ids.iter().any(|id| {
            primitive_ref(*id).is_none_or(|primitive| primitive.source_mesh_id != mesh.id)
        }) {
            return Err(source_mismatch(
                "source.ir.primitives.sourceMeshId",
                "primitive containing mesh disagrees with sourceMeshId",
            ));
        }
        if mesh.primitive_ids.iter().enumerate().any(|(index, id)| {
            primitive_ref(*id).is_none_or(|primitive| {
                u32::try_from(index).ok() != Some(primitive.source_primitive_index)
            })
        }) {
            return Err(source_mismatch(
                "source.ir.primitives.sourcePrimitiveIndex",
                "primitive source index disagrees with containing mesh order",
            ));
        }
    }
    for primitive in &source.ir.primitives {
        let any_nonfinite = primitive
            .positions
            .iter()
            .flatten()
            .chain(primitive.normals.iter().flatten())
            .chain(primitive.tangents.iter().flatten())
            .chain(primitive.uv0.iter().flatten())
            .chain(primitive.weights0.iter().flatten())
            .any(|value| !value.is_finite());
        if mesh_ref(primitive.source_mesh_id).is_none()
            || any_nonfinite
            || primitive
                .material_id
                .is_some_and(|id| material_ref(id).is_none())
        {
            return Err(source_mismatch(
                "source.ir.primitives",
                "primitive references or finite values are invalid",
            ));
        }
        let owner = mesh_ref(primitive.source_mesh_id).ok_or_else(|| {
            source_mismatch(
                "source.ir.primitives.sourceMeshId",
                "source mesh is missing",
            )
        })?;
        let source_index = usize::try_from(primitive.source_primitive_index).map_err(|_| {
            source_mismatch(
                "source.ir.primitives.sourcePrimitiveIndex",
                "source primitive index does not fit host",
            )
        })?;
        if owner.primitive_ids.get(source_index).copied() != Some(primitive.id) {
            return Err(source_mismatch(
                "source.ir.meshes.primitiveIds",
                "primitive is not present exactly at its declared source owner/index",
            ));
        }
        let allow = |code| source_has_primitive_blocking_gate(source, primitive, code);
        let has_positions = !primitive.positions.is_empty();
        let attribute_mismatch = has_positions
            && [
                primitive.normals.len(),
                primitive.tangents.len(),
                primitive.uv0.len(),
                primitive.joints0.len(),
                primitive.weights0.len(),
            ]
            .into_iter()
            .any(|count| count != 0 && count != primitive.positions.len());
        let indices_in_range = !has_positions
            || primitive
                .indices
                .iter()
                .all(|index| (*index as usize) < primitive.positions.len());
        let defects = [
            (primitive.positions.is_empty(), "M2A-GLB-POSITION-MISSING"),
            (primitive.uv0.is_empty(), "M2A-GLB-UV0-MISSING"),
            (
                primitive.topology != "TRIANGLES",
                "M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED",
            ),
            (
                primitive.topology == "TRIANGLES" && !primitive.indices.len().is_multiple_of(3),
                "M2A-GLB-INCOMPLETE-TRIANGLES",
            ),
            (has_positions && !indices_in_range, "M2A-GLB-INDEX-OOB"),
            (attribute_mismatch, "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH"),
            (
                has_positions
                    && primitive.topology == "TRIANGLES"
                    && primitive.indices.len().is_multiple_of(3)
                    && indices_in_range
                    && has_degenerate_positions(&primitive.positions, &primitive.indices),
                "M2A-GLB-DEGENERATE-TRIANGLES",
            ),
        ];
        if defects
            .into_iter()
            .any(|(present, code)| present != allow(code))
        {
            return Err(source_mismatch(
                "source.ir.primitives",
                "primitive defect and exact M2 blocking gate disagree",
            ));
        }
    }
    Ok(())
}

fn source_has_primitive_blocking_gate(
    source: &GlbIngestResult,
    primitive: &IrPrimitive,
    code: &str,
) -> bool {
    let path = format!(
        "meshes[{}].primitives[{}]",
        primitive.source_mesh_id, primitive.source_primitive_index
    );
    source
        .report
        .gates
        .iter()
        .any(|gate| gate.code == code && gate.severity == "BLOCKING" && gate.path == path)
}

fn has_degenerate_positions(positions: &[[f32; 3]], indices: &[u32]) -> bool {
    indices.chunks_exact(3).any(|triangle| {
        if triangle[0] == triangle[1] || triangle[1] == triangle[2] || triangle[0] == triangle[2] {
            return true;
        }
        let a = positions[triangle[0] as usize];
        let b = positions[triangle[1] as usize];
        let c = positions[triangle[2] as usize];
        length_sq(cross(sub3(b, a), sub3(c, a))) == 0.0
    })
}

fn validate_options(options: &ProfileAOptionsV1) -> Result<(), ProfileAConversionFatalError> {
    if options.weight_merge_epsilon != 0.0
        || options.weight_sum_tolerance != 0.00001
        || options.bounds_tolerance_factor != 0.00001
    {
        return Err(fatal(
            "M3A-OPTIONS-INVALID",
            "options",
            "Profile A numeric policies differ from schema version 1",
        ));
    }
    let limits = &options.limits;
    let hard = ProfileALimitsV1::default();
    let pairs = [
        (limits.max_rig_nodes, hard.max_rig_nodes),
        (limits.max_segments, hard.max_segments),
        (limits.max_reference_vertices, hard.max_reference_vertices),
        (limits.max_reference_triangles, hard.max_reference_triangles),
        (limits.max_output_vertices, hard.max_output_vertices),
        (limits.max_output_indices, hard.max_output_indices),
        (
            limits.max_distance_evaluations,
            hard.max_distance_evaluations,
        ),
        (limits.max_work_bytes, hard.max_work_bytes),
        (limits.max_diagnostics, hard.max_diagnostics),
        (limits.max_unique_materials, hard.max_unique_materials),
        (limits.triangle_warning_above, hard.triangle_warning_above),
        (limits.triangle_blocking_above, hard.triangle_blocking_above),
    ];
    if pairs
        .iter()
        .any(|(value, maximum)| *value == 0 || value > maximum)
        || limits.triangle_warning_above != hard.triangle_warning_above
        || limits.triangle_blocking_above != hard.triangle_blocking_above
        || limits.max_unique_materials != 1
    {
        return Err(fatal(
            "M3A-OPTIONS-INVALID",
            "options.limits",
            "limits must be positive, ordered, and no greater than compiled hard maxima",
        ));
    }
    Ok(())
}

fn rig_bind_worlds(
    rig: &CreatureRigProfileV1,
) -> Result<BTreeMap<u32, Mat4>, ProfileAConversionFatalError> {
    let by_id = rig
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<BTreeMap<_, _>>();
    let mut worlds = BTreeMap::<u32, Mat4>::new();
    let mut visiting = BTreeSet::new();
    let mut chain = Vec::new();
    chain.try_reserve(rig.nodes.len()).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "rig.nodes",
            "rig traversal allocation failed",
        )
    })?;
    for node in &rig.nodes {
        if worlds.contains_key(&node.id) {
            continue;
        }
        let mut cursor = Some(node.id);
        while let Some(id) = cursor {
            if worlds.contains_key(&id) {
                break;
            }
            if !visiting.insert(id) {
                return Err(fatal(
                    "M3A-PROFILE-HIERARCHY-INVALID",
                    "rig.nodes",
                    "rig hierarchy contains a cycle",
                ));
            }
            chain.push(id);
            cursor = by_id
                .get(&id)
                .ok_or_else(|| {
                    fatal(
                        "M3A-PROFILE-HIERARCHY-INVALID",
                        "rig.nodes.parentId",
                        "rig node is missing",
                    )
                })?
                .parent_id;
        }
        while let Some(id) = chain.pop() {
            let current = by_id[&id];
            let local = Mat4(current.bind_local_matrix);
            let world = if let Some(parent) = current.parent_id {
                worlds
                    .get(&parent)
                    .copied()
                    .ok_or_else(|| {
                        fatal(
                            "M3A-PROFILE-HIERARCHY-INVALID",
                            "rig.nodes.parentId",
                            "rig parent world is missing",
                        )
                    })?
                    .mul(local)
            } else {
                local
            };
            if !world.is_finite() || world.inverse_affine().is_none() {
                return Err(fatal(
                    "M3A-PROFILE-HIERARCHY-INVALID",
                    "rig.nodes.bindLocalMatrix",
                    "rig bind world is non-finite or singular",
                ));
            }
            worlds.insert(id, world);
            visiting.remove(&id);
        }
    }
    Ok(worlds)
}

fn rig_hierarchy_intervals(
    rig: &CreatureRigProfileV1,
) -> Result<BTreeMap<u32, (u64, u64)>, ProfileAConversionFatalError> {
    let root = rig
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .ok_or_else(|| {
            fatal(
                "M3A-PROFILE-HIERARCHY-INVALID",
                "rig.nodes",
                "rig root is missing",
            )
        })?
        .id;
    let mut children = BTreeMap::<u32, Vec<u32>>::new();
    for node in &rig.nodes {
        children.entry(node.id).or_default();
    }
    for node in &rig.nodes {
        if let Some(parent) = node.parent_id {
            children
                .get_mut(&parent)
                .ok_or_else(|| {
                    fatal(
                        "M3A-PROFILE-HIERARCHY-INVALID",
                        "rig.nodes.parentId",
                        "rig parent is missing",
                    )
                })?
                .push(node.id);
        }
    }
    let mut stack = Vec::new();
    let capacity = rig.nodes.len().checked_mul(2).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "rig.nodes",
            "rig traversal capacity overflow",
        )
    })?;
    stack.try_reserve(capacity).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "rig.nodes",
            "rig interval traversal allocation failed",
        )
    })?;
    stack.push((root, false));
    let mut clock = 0u64;
    let mut intervals = BTreeMap::<u32, (u64, u64)>::new();
    while let Some((id, exit)) = stack.pop() {
        if exit {
            let entry = intervals.get_mut(&id).ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "rig.nodes",
                    "rig interval entry is missing",
                )
            })?;
            entry.1 = clock;
            clock = clock.checked_add(1).ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "rig.nodes",
                    "rig interval clock overflow",
                )
            })?;
            continue;
        }
        if intervals.insert(id, (clock, 0)).is_some() {
            return Err(fatal(
                "M3A-PROFILE-HIERARCHY-INVALID",
                "rig.nodes",
                "rig hierarchy contains a cycle",
            ));
        }
        clock = clock.checked_add(1).ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "rig.nodes",
                "rig interval clock overflow",
            )
        })?;
        stack.push((id, true));
        for child in children.get(&id).into_iter().flatten().rev() {
            stack.push((*child, false));
        }
    }
    if intervals.len() != rig.nodes.len() {
        return Err(fatal(
            "M3A-PROFILE-HIERARCHY-INVALID",
            "rig.nodes",
            "rig hierarchy is disconnected",
        ));
    }
    Ok(intervals)
}

fn validate_rig(
    rig: &CreatureRigProfileV1,
    limits: &ProfileALimitsV1,
    tolerance_factor: f32,
) -> Result<(), ProfileAConversionFatalError> {
    if rig.segments.is_empty() {
        return Err(fatal(
            "M3A-PROFILE-SEGMENT-INVALID",
            "rig.segments",
            "rig must contain at least one segment",
        ));
    }
    if usize_u64(rig.nodes.len()) > limits.max_rig_nodes
        || usize_u64(rig.segments.len()) > limits.max_segments
    {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "rig",
            "rig node or segment limit exceeded",
        ));
    }
    let node_ids = rig
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<BTreeMap<_, _>>();
    if node_ids.len() != rig.nodes.len()
        || rig
            .nodes
            .iter()
            .filter(|node| node.parent_id.is_none())
            .count()
            != 1
    {
        return Err(fatal(
            "M3A-PROFILE-HIERARCHY-INVALID",
            "rig.nodes",
            "node ids must be unique and hierarchy must have exactly one root",
        ));
    }
    for node in &rig.nodes {
        if !logical_label(&node.name)
            || node
                .parent_id
                .is_some_and(|id| !node_ids.contains_key(&id) || id == node.id)
            || !Mat4(node.bind_local_matrix).is_finite()
            || Mat4(node.bind_local_matrix).inverse_affine().is_none()
        {
            return Err(fatal(
                "M3A-PROFILE-HIERARCHY-INVALID",
                "rig.nodes",
                "invalid node name, parent, or bind matrix",
            ));
        }
    }
    let worlds = rig_bind_worlds(rig)?;
    let intervals = rig_hierarchy_intervals(rig)?;
    let segment_ids = rig
        .segments
        .iter()
        .map(|segment| segment.id)
        .collect::<BTreeSet<_>>();
    if segment_ids.len() != rig.segments.len() {
        return Err(fatal(
            "M3A-PROFILE-SEGMENT-INVALID",
            "rig.segments",
            "segment ids must be unique",
        ));
    }
    let mut ref_vertices = 0usize;
    let mut ref_triangles = 0usize;
    for segment in &rig.segments {
        let allowed = segment
            .allowed_bone_node_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if !logical_label(&segment.name)
            || !node_ids.contains_key(&segment.parent_node_id)
            || segment.surface_positions.is_empty()
            || segment.surface_indices.is_empty()
            || segment.surface_indices.len() % 3 != 0
            || segment
                .surface_indices
                .iter()
                .any(|&index| index as usize >= segment.surface_positions.len())
            || segment
                .surface_positions
                .iter()
                .flatten()
                .any(|value| !value.is_finite())
        {
            return Err(fatal(
                "M3A-PROFILE-SEGMENT-INVALID",
                "rig.segments",
                "invalid segment surface or parent",
            ));
        }
        if allowed.len() != segment.allowed_bone_node_ids.len()
            || allowed.iter().any(|bone| {
                if !node_ids.contains_key(bone) {
                    return true;
                }
                let Some(parent_interval) = intervals.get(&segment.parent_node_id) else {
                    return true;
                };
                let Some(bone_interval) = intervals.get(bone) else {
                    return true;
                };
                !(parent_interval.0 <= bone_interval.0 && bone_interval.1 <= parent_interval.1)
            })
        {
            return Err(fatal(
                "M3A-PROFILE-SEGMENT-INVALID",
                "rig.segments.allowedBoneNodeIds",
                "allowed bones must be unique descendants of the segment parent",
            ));
        }
        ref_vertices = ref_vertices
            .checked_add(segment.surface_positions.len())
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "rig.segments",
                    "reference vertex count overflow",
                )
            })?;
        ref_triangles = ref_triangles
            .checked_add(segment.surface_indices.len() / 3)
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "rig.segments",
                    "reference triangle count overflow",
                )
            })?;
        for triangle in segment.surface_indices.chunks_exact(3) {
            let a = segment.surface_positions[triangle[0] as usize];
            let b = segment.surface_positions[triangle[1] as usize];
            let c = segment.surface_positions[triangle[2] as usize];
            if length_sq(cross(sub3(b, a), sub3(c, a))) <= f32::EPSILON {
                return Err(fatal(
                    "M3A-PROFILE-SEGMENT-INVALID",
                    "rig.segments.surfaceIndices",
                    "reference surface contains a degenerate triangle",
                ));
            }
        }
        match segment.deformation {
            RigSegmentDeformationV1::Rigid if !segment.reference_weights.is_empty() => {
                return Err(fatal(
                    "M3A-PROFILE-SEGMENT-INVALID",
                    "rig.segments.referenceWeights",
                    "RIGID segment must not carry reference weights",
                ));
            }
            RigSegmentDeformationV1::Skin
                if segment.reference_weights.len() != segment.surface_positions.len() =>
            {
                return Err(fatal(
                    "M3A-PROFILE-SEGMENT-INVALID",
                    "rig.segments.referenceWeights",
                    "SKIN weights must match surface vertices",
                ));
            }
            RigSegmentDeformationV1::Skin
                if allowed.is_empty()
                    || segment.reference_weights.iter().flatten().any(|influence| {
                        !influence.value.is_finite()
                            || influence.value < 0.0
                            || !allowed.contains(&influence.bone_node_id)
                    }) =>
            {
                return Err(fatal(
                    "M3A-PROFILE-SEGMENT-INVALID",
                    "rig.segments.referenceWeights",
                    "SKIN weights must be finite, nonnegative, and reference allowed bones",
                ));
            }
            _ => {}
        }
    }
    if usize_u64(ref_vertices) > limits.max_reference_vertices
        || usize_u64(ref_triangles) > limits.max_reference_triangles
    {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "rig.segments",
            "cumulative reference surface limit exceeded",
        ));
    }
    for segment in &rig.segments {
        let world = worlds[&segment.parent_node_id];
        for &position in &segment.surface_positions {
            let position = world.transform_point(position)?;
            for (axis, coordinate) in position.iter().enumerate() {
                let tolerance = tolerance_factor
                    * (rig.target_bounds.max[axis] - rig.target_bounds.min[axis]).max(1.0);
                if *coordinate < rig.target_bounds.min[axis] - tolerance
                    || *coordinate > rig.target_bounds.max[axis] + tolerance
                {
                    return Err(fatal(
                        "M3A-PROFILE-BOUNDS-INVALID",
                        "rig.segments.surfacePositions",
                        "rig surface lies outside target bounds",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn collect_preflight_gates(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
    source_inventory_policy: SourceInventoryPolicy,
) -> Result<Vec<ProfileAGateV1>, ProfileAConversionFatalError> {
    let mut gates = Vec::new();
    for source_gate in source
        .report
        .gates
        .iter()
        .filter(|item| item.severity == "BLOCKING")
    {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-SOURCE-BLOCKED",
                &source_gate.path,
                &format!("source gate {}: {}", source_gate.code, source_gate.message),
            ),
            &options.limits,
        )?;
    }
    if source_inventory_policy == SourceInventoryPolicy::RejectPresent
        && (!source.ir.skins.is_empty()
            || source.ir.nodes.iter().any(|node| node.skin_id.is_some()))
    {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-SOURCE-RIG-DEFERRED",
                "source.ir.skins",
                "source rig mapping is deferred for nonhumanoid Profile A",
            ),
            &options.limits,
        )?;
    }
    if source_inventory_policy == SourceInventoryPolicy::RejectPresent
        && !source.ir.animations.is_empty()
    {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-SOURCE-ANIMATION-DEFERRED",
                "source.ir.animations",
                "source animation mapping is deferred for nonhumanoid Profile A",
            ),
            &options.limits,
        )?;
    }
    let attestations = &rig.provenance.attestations;
    let provenance_allowed = matches!(
        rig.provenance.kind,
        RigProvenanceKindV1::Synthetic
            | RigProvenanceKindV1::Owned
            | RigProvenanceKindV1::UserProvided
    ) && rig.provenance.export_allowed
        && attestations.controlled_construction
        && attestations.no_reference_payload_copied
        && attestations.rights_confirmed;
    if !provenance_allowed {
        push_gate_checked(
            &mut gates,
            gate(
                "M3A-PROFILE-PROVENANCE-FORBIDDEN",
                "rig.provenance",
                "profile provenance policy forbids export",
            ),
            &options.limits,
        )?;
    }
    if usize_u64(gates.len()) > options.limits.max_diagnostics {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "report.gates",
            "diagnostic limit exceeded",
        ));
    }
    Ok(gates)
}

struct GeometryInstance<'a> {
    primitive: &'a IrPrimitive,
    source_world: Mat4,
}

fn geometry_buffer_bytes(primitive: &IrPrimitive) -> Result<u64, ProfileAConversionFatalError> {
    let stride = 32u64
        .checked_add(if primitive.tangents.is_empty() { 0 } else { 16 })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "vertex stride overflow",
            )
        })?;
    let vertices = usize_u64(primitive.positions.len())
        .checked_mul(stride)
        .ok_or_else(|| fatal("M3A-INTEGER-OVERFLOW", "workBytes", "vertex bytes overflow"))?;
    let indices = usize_u64(primitive.indices.len())
        .checked_mul(4)
        .ok_or_else(|| fatal("M3A-INTEGER-OVERFLOW", "workBytes", "index bytes overflow"))?;
    vertices.checked_add(indices).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "geometry bytes overflow",
        )
    })
}

fn geometry_instances<'a>(
    source: &'a GlbIngestResult,
    worlds: &[Option<Mat4>],
    nodes: &[&'a IrNode],
    limits: &ProfileALimitsV1,
    selection_persistent_bytes: u64,
    base_work_bytes: u64,
) -> Result<(Vec<GeometryInstance<'a>>, u64, u64), ProfileAConversionFatalError> {
    let mut entry_count = 0u64;
    let mut output_vertices = 0u64;
    let mut output_indices = 0u64;
    let mut output_bytes = 0u64;
    for node in nodes {
        let Some(mesh_id) = node.mesh_id else {
            continue;
        };
        let mesh_index = usize::try_from(mesh_id).map_err(|_| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.nodes.meshId",
                "mesh id does not fit host index",
            )
        })?;
        let mesh = source
            .ir
            .meshes
            .get(mesh_index)
            .filter(|mesh| mesh.id == mesh_id)
            .ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "source.ir.nodes.meshId",
                    "node references missing mesh",
                )
            })?;
        for primitive_id in &mesh.primitive_ids {
            let primitive_index = usize::try_from(*primitive_id).map_err(|_| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "source.ir.meshes.primitiveIds",
                    "primitive id does not fit host index",
                )
            })?;
            let primitive = source
                .ir
                .primitives
                .get(primitive_index)
                .filter(|primitive| primitive.id == *primitive_id)
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTERNAL-CONTRACT",
                        "source.ir.meshes.primitiveIds",
                        "mesh references missing primitive",
                    )
                })?;
            entry_count = entry_count.checked_add(1).ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "sourceSelection.meshInstances",
                    "geometry instance count overflow",
                )
            })?;
            output_vertices = output_vertices
                .checked_add(usize_u64(primitive.positions.len()))
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTEGER-OVERFLOW",
                        "creature.segments.positions",
                        "planned vertex count overflow",
                    )
                })?;
            output_indices = output_indices
                .checked_add(usize_u64(primitive.indices.len()))
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTEGER-OVERFLOW",
                        "creature.segments.indices",
                        "planned index count overflow",
                    )
                })?;
            output_bytes = output_bytes
                .checked_add(geometry_buffer_bytes(primitive)?)
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTEGER-OVERFLOW",
                        "workBytes",
                        "planned geometry bytes overflow",
                    )
                })?;
            if output_vertices > limits.max_output_vertices
                || output_indices > limits.max_output_indices
            {
                return Err(fatal(
                    "M3A-LIMIT-EXCEEDED",
                    "creature.segments",
                    "planned instanced geometry exceeds output limits",
                ));
            }
        }
    }
    let instance_bytes = entry_count
        .checked_mul(WORK_GEOMETRY_INSTANCE)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "instance bytes overflow",
            )
        })?;
    let construction_peak = selection_persistent_bytes
        .checked_add(instance_bytes)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "instance construction peak overflow",
            )
        })?;
    let retained_instance_bytes = base_work_bytes.checked_add(instance_bytes).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "retained instance bytes overflow",
        )
    })?;
    let output_peak = retained_instance_bytes
        .checked_add(output_bytes)
        .and_then(|value| value.checked_add(output_bytes.checked_mul(8)?))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "planned output peak overflow",
            )
        })?;
    if construction_peak.max(output_peak) > limits.max_work_bytes {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "workBytes",
            "planned instanced geometry exceeds work byte limit",
        ));
    }
    let reserve = usize::try_from(entry_count).map_err(|_| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "sourceSelection.meshInstances",
            "instance count does not fit host usize",
        )
    })?;
    let mut result = Vec::new();
    result.try_reserve(reserve).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "sourceSelection.meshInstances",
            "instance allocation failed",
        )
    })?;
    for node in nodes {
        let Some(mesh_id) = node.mesh_id else {
            continue;
        };
        let mesh_index = usize::try_from(mesh_id).map_err(|_| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.nodes.meshId",
                "mesh id does not fit host index",
            )
        })?;
        let mesh = source
            .ir
            .meshes
            .get(mesh_index)
            .filter(|mesh| mesh.id == mesh_id)
            .ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "source.ir.nodes.meshId",
                    "node references missing mesh",
                )
            })?;
        let node_index = usize::try_from(node.id).map_err(|_| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.nodes",
                "node id does not fit host index",
            )
        })?;
        let world = worlds.get(node_index).copied().flatten().ok_or_else(|| {
            fatal(
                "M3A-INTERNAL-CONTRACT",
                "source.ir.nodes",
                "node world transform is missing",
            )
        })?;
        for primitive_id in &mesh.primitive_ids {
            let primitive_index = usize::try_from(*primitive_id).map_err(|_| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "source.ir.meshes.primitiveIds",
                    "primitive id does not fit host index",
                )
            })?;
            result.push(GeometryInstance {
                primitive: source
                    .ir
                    .primitives
                    .get(primitive_index)
                    .filter(|primitive| primitive.id == *primitive_id)
                    .ok_or_else(|| {
                        fatal(
                            "M3A-INTERNAL-CONTRACT",
                            "source.ir.meshes.primitiveIds",
                            "mesh references missing primitive",
                        )
                    })?,
                source_world: world,
            });
        }
    }
    if result.is_empty() {
        return Err(fatal(
            "M3A-INTERNAL-CONTRACT",
            "source.ir.nodes",
            "no instanced geometry was found",
        ));
    }
    Ok((
        result,
        retained_instance_bytes,
        construction_peak.max(output_peak),
    ))
}

struct MaterialSummary {
    bindings: Vec<MaterialSourceBindingV1>,
    diagnostics: Vec<ProfileADiagnosticV1>,
    retained_work_bytes: u64,
    peak_work_bytes: u64,
}

fn material_summary(
    source: &GlbIngestResult,
    instances: &[GeometryInstance<'_>],
    gate_count: usize,
    base_work_bytes: u64,
    limits: &ProfileALimitsV1,
) -> Result<MaterialSummary, ProfileAConversionFatalError> {
    if instances.is_empty() {
        return Ok(MaterialSummary {
            bindings: Vec::new(),
            diagnostics: Vec::new(),
            retained_work_bytes: base_work_bytes,
            peak_work_bytes: base_work_bytes,
        });
    }

    let used_bytes = usize_u64(source.ir.materials.len())
        .checked_mul(WORK_BOOL)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "material usage work byte product overflow",
            )
        })?;
    let mut scratch_peak = base_work_bytes;
    reserve_work_bytes(&mut scratch_peak, used_bytes, limits)?;
    let mut used = Vec::new();
    used.try_reserve(source.ir.materials.len()).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "materials",
            "material usage allocation failed",
        )
    })?;
    used.resize(source.ir.materials.len(), false);
    let mut uses_null = false;
    for instance in instances {
        match instance.primitive.material_id {
            None => uses_null = true,
            Some(id) => {
                let index = usize::try_from(id).map_err(|_| {
                    fatal(
                        "M3A-INTEGER-OVERFLOW",
                        "source.ir.primitives.materialId",
                        "material id does not fit host index",
                    )
                })?;
                let entry = used.get_mut(index).ok_or_else(|| {
                    fatal(
                        "M3A-INTERNAL-CONTRACT",
                        "source.ir.primitives.materialId",
                        "validated material id is missing",
                    )
                })?;
                *entry = true;
            }
        }
    }

    let binding_count = usize::from(uses_null)
        .checked_add(used.iter().filter(|entry| **entry).count())
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "materials.bindings",
                "material binding count overflow",
            )
        })?;
    let diagnostic_count = source
        .ir
        .materials
        .iter()
        .enumerate()
        .filter(|(index, material)| {
            used[*index]
                && material
                    .name
                    .as_deref()
                    .is_some_and(|name| !logical_material_name(name))
        })
        .count();
    ensure_diagnostic_limit(gate_count, diagnostic_count, limits)?;

    const DIAGNOSTIC_CODE: &str = "M3A-SOURCE-MATERIAL-NAME-OMITTED";
    const DIAGNOSTIC_SEVERITY: &str = "INFO";
    const DIAGNOSTIC_PATH_PREFIX: &str = "source.ir.materials[";
    const DIAGNOSTIC_MESSAGE: &str = "non-logical source material name was omitted";
    let binding_struct_bytes = usize_u64(binding_count)
        .checked_mul(WORK_MATERIAL_BINDING)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "binding byte product overflow",
            )
        })?;
    let binding_name_bytes = source
        .ir
        .materials
        .iter()
        .enumerate()
        .filter(|(index, _)| used[*index])
        .filter_map(|(_, material)| material.name.as_deref())
        .filter(|name| logical_material_name(name))
        .try_fold(0_u64, |sum, name| sum.checked_add(usize_u64(name.len())))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "binding name byte sum overflow",
            )
        })?;
    let diagnostic_struct_bytes = usize_u64(diagnostic_count)
        .checked_mul(WORK_DIAGNOSTIC)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "diagnostic byte product overflow",
            )
        })?;
    let diagnostic_string_bytes = source
        .ir
        .materials
        .iter()
        .enumerate()
        .filter(|(index, material)| {
            used[*index]
                && material
                    .name
                    .as_deref()
                    .is_some_and(|name| !logical_material_name(name))
        })
        .try_fold(0_u64, |sum, (_, material)| {
            let id_digits = material.id.to_string().len();
            let bytes = DIAGNOSTIC_CODE
                .len()
                .checked_add(DIAGNOSTIC_SEVERITY.len())
                .and_then(|value| value.checked_add(DIAGNOSTIC_PATH_PREFIX.len()))
                .and_then(|value| value.checked_add(id_digits))
                .and_then(|value| value.checked_add(2))
                .and_then(|value| value.checked_add(DIAGNOSTIC_MESSAGE.len()))?;
            sum.checked_add(usize_u64(bytes))
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "diagnostic string byte sum overflow",
            )
        })?;
    let retained_material_bytes = binding_struct_bytes
        .checked_add(binding_name_bytes)
        .and_then(|value| value.checked_add(diagnostic_struct_bytes))
        .and_then(|value| value.checked_add(diagnostic_string_bytes))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "material summary byte sum overflow",
            )
        })?;
    reserve_work_bytes(&mut scratch_peak, retained_material_bytes, limits)?;
    let retained_work_bytes = base_work_bytes
        .checked_add(retained_material_bytes)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "retained material byte sum overflow",
            )
        })?;

    let mut bindings = Vec::new();
    bindings.try_reserve(binding_count).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "materials.bindings",
            "material binding allocation failed",
        )
    })?;
    if uses_null {
        bindings.push(MaterialSourceBindingV1 {
            slot: 0,
            source_material_id: None,
            source_material_name: None,
        });
    }
    for (index, material) in source.ir.materials.iter().enumerate() {
        if !used[index] {
            continue;
        }
        let slot = u32::try_from(bindings.len()).map_err(|_| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "materials.bindings.slot",
                "material binding slot does not fit u32",
            )
        })?;
        bindings.push(MaterialSourceBindingV1 {
            slot,
            source_material_id: Some(material.id),
            source_material_name: material
                .name
                .as_deref()
                .filter(|name| logical_material_name(name))
                .map(str::to_owned),
        });
    }

    let mut diagnostics = Vec::new();
    diagnostics.try_reserve(diagnostic_count).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "report.diagnostics",
            "diagnostic allocation failed",
        )
    })?;
    for (index, material) in source.ir.materials.iter().enumerate() {
        if !used[index] || material.name.as_deref().is_none_or(logical_material_name) {
            continue;
        }
        diagnostics.push(ProfileADiagnosticV1 {
            schema_version: 1,
            code: DIAGNOSTIC_CODE.to_owned(),
            severity: DIAGNOSTIC_SEVERITY.to_owned(),
            path: format!("{DIAGNOSTIC_PATH_PREFIX}{}].name", material.id),
            message: DIAGNOSTIC_MESSAGE.to_owned(),
        });
    }

    Ok(MaterialSummary {
        bindings,
        diagnostics,
        retained_work_bytes,
        peak_work_bytes: scratch_peak,
    })
}

fn logical_label(name: &str) -> bool {
    let bytes = name.as_bytes();
    !name.trim().is_empty()
        && bytes.len() <= 128
        && !name.contains(['/', '\\'])
        && !name.contains("://")
        && !name.contains(':')
        && !name.chars().any(char::is_control)
}
fn logical_material_name(name: &str) -> bool {
    logical_label(name)
}

struct SourceSelection<'a> {
    worlds: Vec<Option<Mat4>>,
    ordered_nodes: Vec<&'a IrNode>,
    report: ProfileASourceSelectionReportV1,
    persistent_work_bytes: u64,
    traversal_peak_bytes: u64,
}

enum SourceSelectionError {
    Gate(ProfileAGateV1),
    Fatal(ProfileAConversionFatalError),
}

fn select_default_scene<'a>(
    source: &'a GlbIngestResult,
    limits: &ProfileALimitsV1,
    base_work_bytes: u64,
) -> Result<SourceSelection<'a>, Box<SourceSelectionError>> {
    let scene_id = source.ir.default_scene_id.ok_or_else(|| {
        SourceSelectionError::Gate(gate(
            "M3A-DEFAULT-SCENE-REQUIRED",
            "source.ir.defaultSceneId",
            "default scene is required",
        ))
    })?;
    let scene_index = usize::try_from(scene_id).map_err(|_| {
        SourceSelectionError::Gate(gate(
            "M3A-DEFAULT-SCENE-REQUIRED",
            "source.ir.defaultSceneId",
            "default scene id does not fit host index",
        ))
    })?;
    let scene = source
        .ir
        .scenes
        .get(scene_index)
        .filter(|scene| scene.id == scene_id)
        .ok_or_else(|| {
            SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-REQUIRED",
                "source.ir.defaultSceneId",
                "default scene id does not exist",
            ))
        })?;
    if scene.root_node_ids.is_empty() {
        return Err(Box::new(SourceSelectionError::Gate(gate(
            "M3A-DEFAULT-SCENE-REQUIRED",
            "source.ir.scenes.rootNodeIds",
            "default scene has no roots",
        ))));
    }
    let node_count = usize_u64(source.ir.nodes.len());
    let edge_count = source
        .ir
        .nodes
        .iter()
        .try_fold(usize_u64(scene.root_node_ids.len()), |total, node| {
            total.checked_add(usize_u64(node.child_ids.len()))
        })
        .ok_or_else(|| {
            SourceSelectionError::Fatal(fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.nodes.childIds",
                "scene traversal edge count overflow",
            ))
        })?;
    let worlds_bytes = node_count.checked_mul(WORK_OPTION_MAT4).ok_or_else(|| {
        SourceSelectionError::Fatal(fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "world transform buffer overflow",
        ))
    })?;
    let ordered_bytes = node_count.checked_mul(WORK_REFERENCE).ok_or_else(|| {
        SourceSelectionError::Fatal(fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "ordered node buffer overflow",
        ))
    })?;
    let seen_bytes = node_count.checked_mul(WORK_BOOL).ok_or_else(|| {
        SourceSelectionError::Fatal(fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "visited buffer overflow",
        ))
    })?;
    let mesh_seen_bytes = usize_u64(source.ir.meshes.len())
        .checked_mul(WORK_BOOL)
        .ok_or_else(|| {
            SourceSelectionError::Fatal(fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "reachable mesh buffer overflow",
            ))
        })?;
    let stack_bytes = edge_count
        .checked_mul(WORK_SCENE_STACK_ENTRY)
        .ok_or_else(|| {
            SourceSelectionError::Fatal(fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "traversal stack buffer overflow",
            ))
        })?;
    let persistent_work_bytes = base_work_bytes
        .checked_add(worlds_bytes)
        .and_then(|value| value.checked_add(ordered_bytes))
        .ok_or_else(|| {
            SourceSelectionError::Fatal(fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "persistent scene buffer overflow",
            ))
        })?;
    let traversal_peak_bytes = persistent_work_bytes
        .checked_add(seen_bytes)
        .and_then(|value| value.checked_add(stack_bytes))
        .and_then(|value| value.checked_add(mesh_seen_bytes))
        .ok_or_else(|| {
            SourceSelectionError::Fatal(fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "scene traversal peak overflow",
            ))
        })?;
    if traversal_peak_bytes > limits.max_work_bytes {
        return Err(Box::new(SourceSelectionError::Fatal(fatal(
            "M3A-LIMIT-EXCEEDED",
            "workBytes",
            "default scene traversal exceeds work byte limit",
        ))));
    }
    let mut worlds = Vec::new();
    worlds.try_reserve(source.ir.nodes.len()).map_err(|_| {
        SourceSelectionError::Fatal(fatal(
            "M3A-LIMIT-EXCEEDED",
            "workBytes",
            "world transform allocation failed",
        ))
    })?;
    worlds.resize(source.ir.nodes.len(), None);
    let mut ordered_nodes = Vec::new();
    ordered_nodes
        .try_reserve(source.ir.nodes.len())
        .map_err(|_| {
            SourceSelectionError::Fatal(fatal(
                "M3A-LIMIT-EXCEEDED",
                "workBytes",
                "ordered node allocation failed",
            ))
        })?;
    let mut parent_seen = Vec::new();
    parent_seen
        .try_reserve(source.ir.nodes.len())
        .map_err(|_| {
            SourceSelectionError::Fatal(fatal(
                "M3A-LIMIT-EXCEEDED",
                "workBytes",
                "visited allocation failed",
            ))
        })?;
    parent_seen.resize(source.ir.nodes.len(), false);
    let mut stack = Vec::new();
    let stack_capacity = usize::try_from(edge_count).map_err(|_| {
        SourceSelectionError::Fatal(fatal(
            "M3A-INTEGER-OVERFLOW",
            "source.ir.nodes.childIds",
            "edge count does not fit host index",
        ))
    })?;
    stack.try_reserve(stack_capacity).map_err(|_| {
        SourceSelectionError::Fatal(fatal(
            "M3A-LIMIT-EXCEEDED",
            "workBytes",
            "scene stack allocation failed",
        ))
    })?;
    for &root in scene.root_node_ids.iter().rev() {
        stack.push((root, None, Mat4::identity()));
    }
    while let Some((id, parent, parent_world)) = stack.pop() {
        let index = usize::try_from(id).map_err(|_| {
            SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                "source.ir.nodes",
                "reachable node id does not fit host index",
            ))
        })?;
        if parent_seen.get(index).copied().unwrap_or(true) {
            return Err(Box::new(SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                "source.ir.nodes",
                "reachable node has multiple paths or a cycle",
            ))));
        }
        parent_seen[index] = true;
        let node = source
            .ir
            .nodes
            .get(index)
            .filter(|node| node.id == id)
            .ok_or_else(|| {
                SourceSelectionError::Gate(gate(
                    "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                    "source.ir.scenes.rootNodeIds",
                    "scene references missing node",
                ))
            })?;
        if node.parent_ids.len() > 1 || node.parent_ids.first().copied() != parent {
            return Err(Box::new(SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                "source.ir.nodes.parentIds",
                "reachable parent relation is ambiguous",
            ))));
        }
        let local = matrix_from_transform(&node.transform).map_err(|_| {
            SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                "source.ir.nodes.transform",
                "reachable node transform is invalid",
            ))
        })?;
        let world = parent_world.mul(local);
        worlds[index] = Some(world);
        ordered_nodes.push(node);
        for &child in node.child_ids.iter().rev() {
            stack.push((child, Some(id), world));
        }
    }
    let reachable_mesh_instances = ordered_nodes
        .iter()
        .filter(|node| node.mesh_id.is_some())
        .count();
    if reachable_mesh_instances == 0 {
        return Err(Box::new(SourceSelectionError::Gate(gate(
            "M3A-DEFAULT-SCENE-REQUIRED",
            "source.ir.scenes",
            "default scene contains no reachable mesh instance",
        ))));
    }
    let mut reachable_mesh_ids = Vec::new();
    reachable_mesh_ids
        .try_reserve(source.ir.meshes.len())
        .map_err(|_| {
            SourceSelectionError::Fatal(fatal(
                "M3A-LIMIT-EXCEEDED",
                "workBytes",
                "reachable mesh allocation failed",
            ))
        })?;
    reachable_mesh_ids.resize(source.ir.meshes.len(), false);
    let mut unique_meshes = 0usize;
    for mesh_id in ordered_nodes.iter().filter_map(|node| node.mesh_id) {
        let index = usize::try_from(mesh_id).map_err(|_| {
            SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                "source.ir.nodes.meshId",
                "mesh id does not fit host index",
            ))
        })?;
        let seen = reachable_mesh_ids.get_mut(index).ok_or_else(|| {
            SourceSelectionError::Gate(gate(
                "M3A-DEFAULT-SCENE-HIERARCHY-INVALID",
                "source.ir.nodes.meshId",
                "mesh id is missing",
            ))
        })?;
        if !*seen {
            *seen = true;
            unique_meshes = unique_meshes.checked_add(1).ok_or_else(|| {
                SourceSelectionError::Fatal(fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "sourceSelection",
                    "reachable mesh count overflow",
                ))
            })?;
        }
    }
    let report = ProfileASourceSelectionReportV1 {
        reachable_node_count: usize_u64(ordered_nodes.len()),
        reachable_mesh_instance_count: usize_u64(reachable_mesh_instances),
        ignored_node_count: usize_u64(source.ir.nodes.len().saturating_sub(ordered_nodes.len())),
        ignored_mesh_count: usize_u64(
            source
                .ir
                .meshes
                .iter()
                .filter(|mesh| {
                    usize::try_from(mesh.id)
                        .ok()
                        .is_none_or(|index| reachable_mesh_ids.get(index).copied() != Some(true))
                })
                .count(),
        ),
        duplicated_mesh_instance_count: usize_u64(
            reachable_mesh_instances.saturating_sub(unique_meshes),
        ),
    };
    Ok(SourceSelection {
        worlds,
        ordered_nodes,
        report,
        persistent_work_bytes,
        traversal_peak_bytes,
    })
}

fn world_bounds(
    instances: &[GeometryInstance<'_>],
) -> Result<Bounds3V1, ProfileAConversionFatalError> {
    let mut bounds = Bounds3V1::empty();
    for instance in instances {
        for &position in &instance.primitive.positions {
            bounds.include(instance.source_world.transform_point(position)?);
        }
    }
    bounds.ensure_nonempty("source.ir.primitives")
}

#[derive(Clone)]
struct BucketPlan {
    segment_index: usize,
    vertex_count: usize,
    index_count: usize,
    tangent_present: bool,
    tangent_absent: bool,
}

struct AssignmentPlan {
    triangle_segments: Vec<Vec<usize>>,
    buckets: BTreeMap<(u32, u32), BucketPlan>,
    segment_order: Vec<usize>,
    mixed_tangent_buckets: Vec<(u32, u32)>,
    has_unreferenced_vertices: bool,
    expected_distance_evaluations: u64,
    work_bytes_peak: u64,
}

#[allow(clippy::too_many_arguments)]
fn plan_triangle_assignments(
    instances: &[GeometryInstance<'_>],
    conversion: Mat4,
    rig: &CreatureRigProfileV1,
    rig_worlds: &BTreeMap<u32, Mat4>,
    bindings: &[MaterialSourceBindingV1],
    base_work_bytes: u64,
    counters: &mut Counters,
    limits: &ProfileALimitsV1,
) -> Result<AssignmentPlan, ProfileAConversionFatalError> {
    let triangle_count = instances
        .iter()
        .try_fold(0usize, |sum, instance| {
            sum.checked_add(instance.primitive.indices.len() / 3)
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "source.ir.primitives.indices",
                "triangle count overflow",
            )
        })?;
    let assignment_bytes = usize_u64(triangle_count)
        .checked_mul(WORK_USIZE)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "assignment byte product overflow",
            )
        })?;
    let assignment_outer_bytes = usize_u64(instances.len())
        .checked_mul(WORK_VEC_HEADER)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "assignment outer byte product overflow",
            )
        })?;
    let segment_order_bytes = usize_u64(rig.segments.len())
        .checked_mul(WORK_USIZE)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "segment order byte product overflow",
            )
        })?;
    let bucket_budget_bytes = usize_u64(rig.segments.len())
        .checked_mul(WORK_BUCKET_PLAN_ENTRY)
        .and_then(|value| value.checked_mul(3))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "bucket plan byte product overflow",
            )
        })?;
    let mut retained_work_bytes = base_work_bytes;
    reserve_work_bytes(&mut retained_work_bytes, assignment_bytes, limits)?;
    reserve_work_bytes(&mut retained_work_bytes, assignment_outer_bytes, limits)?;
    reserve_work_bytes(&mut retained_work_bytes, segment_order_bytes, limits)?;
    reserve_work_bytes(&mut retained_work_bytes, bucket_budget_bytes, limits)?;
    let surface_triangles = rig
        .segments
        .iter()
        .try_fold(0_u64, |sum, segment| {
            sum.checked_add(usize_u64(segment.surface_indices.len() / 3))
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "distanceEvaluations",
                "surface triangle count overflow",
            )
        })?;
    let assignment_evaluations = usize_u64(triangle_count)
        .checked_mul(3)
        .and_then(|value| value.checked_mul(surface_triangles))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "distanceEvaluations",
                "assignment evaluation product overflow",
            )
        })?;
    if assignment_evaluations > limits.max_distance_evaluations {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "distanceEvaluations",
            "assignment evaluation limit exceeded before allocation",
        ));
    }
    let mut work_bytes_peak = retained_work_bytes;
    let mut triangle_segments = Vec::new();
    triangle_segments
        .try_reserve(instances.len())
        .map_err(|_| {
            fatal(
                "M3A-LIMIT-EXCEEDED",
                "segmentAssignment",
                "assignment outer allocation failed",
            )
        })?;
    let mut segment_order = Vec::new();
    segment_order.try_reserve(rig.segments.len()).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "segmentAssignment",
            "segment order allocation failed",
        )
    })?;
    segment_order.extend(0..rig.segments.len());
    segment_order.sort_by_key(|index| rig.segments[*index].id);
    let mut buckets = BTreeMap::<(u32, u32), BucketPlan>::new();
    let mut has_unreferenced_vertices = false;

    for instance in instances {
        let primitive = instance.primitive;
        let material_slot = material_slot_for(primitive.material_id, bindings)?;
        let target_matrix = conversion.mul(instance.source_world);
        let mut assignments = Vec::new();
        assignments
            .try_reserve(primitive.indices.len() / 3)
            .map_err(|_| {
                fatal(
                    "M3A-LIMIT-EXCEEDED",
                    "segmentAssignment",
                    "assignment allocation failed",
                )
            })?;
        for triangle in primitive.indices.chunks_exact(3) {
            let target = [
                target_matrix.transform_point(primitive.positions[triangle[0] as usize])?,
                target_matrix.transform_point(primitive.positions[triangle[1] as usize])?,
                target_matrix.transform_point(primitive.positions[triangle[2] as usize])?,
            ];
            let mut best: Option<(f64, u32, usize)> = None;
            for (segment_index, segment) in rig.segments.iter().enumerate() {
                let surface_world = *rig_worlds.get(&segment.parent_node_id).ok_or_else(|| {
                    fatal(
                        "M3A-INTERNAL-CONTRACT",
                        "rig.segments.parentNodeId",
                        "validated rig parent is missing",
                    )
                })?;
                let mut score = 0.0_f64;
                for point in target {
                    let mut nearest = f64::INFINITY;
                    for surface_triangle in segment.surface_indices.chunks_exact(3) {
                        let a = surface_world.transform_point(
                            segment.surface_positions[surface_triangle[0] as usize],
                        )?;
                        let b = surface_world.transform_point(
                            segment.surface_positions[surface_triangle[1] as usize],
                        )?;
                        let c = surface_world.transform_point(
                            segment.surface_positions[surface_triangle[2] as usize],
                        )?;
                        let (distance, _) = evaluated_point_triangle(
                            point,
                            a,
                            b,
                            c,
                            &mut counters.distance_evaluations,
                            limits,
                        )?;
                        if distance < nearest {
                            nearest = distance;
                        }
                    }
                    score += nearest;
                }
                let candidate = (score, segment.id, segment_index);
                if best.is_none_or(|current| {
                    candidate.0 < current.0 || (candidate.0 == current.0 && candidate.1 < current.1)
                }) {
                    best = Some(candidate);
                }
            }
            let (_, _, segment_index) = best.ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "rig.segments",
                    "validated rig has no assignment candidate",
                )
            })?;
            assignments.push(segment_index);
            let segment = &rig.segments[segment_index];
            let plan = buckets
                .entry((segment.id, material_slot))
                .or_insert(BucketPlan {
                    segment_index,
                    vertex_count: 0,
                    index_count: 0,
                    tangent_present: false,
                    tangent_absent: false,
                });
            plan.index_count = plan.index_count.checked_add(3).ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "creature.segments.indices",
                    "bucket index count overflow",
                )
            })?;
            if primitive.tangents.is_empty() {
                plan.tangent_absent = true;
            } else {
                plan.tangent_present = true;
            }
        }

        let marks_bytes = usize_u64(primitive.positions.len())
            .checked_mul(WORK_U32)
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "assignment mark byte product overflow",
                )
            })?;
        let active_bytes = usize_u64(rig.segments.len())
            .checked_mul(WORK_BOOL)
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "active segment byte product overflow",
                )
            })?;
        let scratch = marks_bytes.checked_add(active_bytes).ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "assignment scratch byte sum overflow",
            )
        })?;
        let mut scratch_peak = retained_work_bytes;
        reserve_work_bytes(&mut scratch_peak, scratch, limits)?;
        work_bytes_peak = work_bytes_peak.max(scratch_peak);
        let mut marks = Vec::new();
        marks.try_reserve(primitive.positions.len()).map_err(|_| {
            fatal(
                "M3A-LIMIT-EXCEEDED",
                "segmentAssignment",
                "assignment mark allocation failed",
            )
        })?;
        marks.resize(primitive.positions.len(), 0_u32);
        let mut active = Vec::new();
        active.try_reserve(rig.segments.len()).map_err(|_| {
            fatal(
                "M3A-LIMIT-EXCEEDED",
                "segmentAssignment",
                "active segment allocation failed",
            )
        })?;
        active.resize(rig.segments.len(), false);
        let mut source_used = 0usize;
        for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
            active[assignments[triangle_index]] = true;
            for &source_index in triangle {
                let mark = &mut marks[source_index as usize];
                if *mark == 0 {
                    *mark = 1;
                    source_used += 1;
                }
            }
        }
        let mut stamp = 1_u32;
        has_unreferenced_vertices |= source_used != primitive.positions.len();
        let mut emitted_for_instance = 0usize;
        for &segment_index in segment_order.iter().filter(|index| active[**index]) {
            stamp = stamp.checked_add(1).ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "segmentAssignment",
                    "assignment stamp overflow",
                )
            })?;
            let mut count = 0usize;
            for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
                if assignments[triangle_index] != segment_index {
                    continue;
                }
                for &source_index in triangle {
                    let mark = &mut marks[source_index as usize];
                    if *mark != stamp {
                        *mark = stamp;
                        count += 1;
                    }
                }
            }
            let key = (rig.segments[segment_index].id, material_slot);
            let plan = buckets.get_mut(&key).ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "segmentAssignment",
                    "bucket plan is missing",
                )
            })?;
            plan.vertex_count = plan.vertex_count.checked_add(count).ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "creature.segments.positions",
                    "bucket vertex count overflow",
                )
            })?;
            emitted_for_instance = emitted_for_instance.checked_add(count).ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "report.geometry",
                    "emitted vertex count overflow",
                )
            })?;
        }
        counters.source_vertices = checked_add(
            counters.source_vertices,
            primitive.positions.len(),
            "source vertices",
        )?;
        counters.source_triangles = checked_add(
            counters.source_triangles,
            assignments.len(),
            "source triangles",
        )?;
        counters.output_vertices = checked_add(
            counters.output_vertices,
            emitted_for_instance,
            "output vertices",
        )?;
        counters.output_triangles = checked_add(
            counters.output_triangles,
            assignments.len(),
            "output triangles",
        )?;
        counters.duplicated_vertices = checked_add(
            counters.duplicated_vertices,
            emitted_for_instance.saturating_sub(source_used),
            "duplicated vertices",
        )?;
        triangle_segments.push(assignments);
    }

    if counters.output_vertices > limits.max_output_vertices
        || counters.output_triangles.checked_mul(3).ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "creature.segments.indices",
                "output index count overflow",
            )
        })? > limits.max_output_indices
    {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "creature.segments",
            "cumulative output geometry limit exceeded",
        ));
    }
    let mut output_bytes = usize_u64(buckets.len())
        .checked_mul(WORK_CREATURE_SEGMENT)
        .and_then(|value| value.checked_mul(2))
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "output segment container byte product overflow",
            )
        })?;
    let mut max_weight_temp_bytes = 0_u64;
    let mut weight_evaluations = 0_u64;
    for plan in buckets.values() {
        let segment = &rig.segments[plan.segment_index];
        let per_vertex = 12_u64
            .checked_add(12)
            .and_then(|value| value.checked_add(8))
            .and_then(|value| value.checked_add(if plan.tangent_present { 16 } else { 0 }))
            .and_then(|value| {
                value.checked_add(if segment.deformation == RigSegmentDeformationV1::Skin {
                    WORK_VERTEX_WEIGHTS
                } else {
                    0
                })
            })
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "output vertex stride overflow",
                )
            })?;
        let bytes = usize_u64(plan.vertex_count)
            .checked_mul(per_vertex)
            .and_then(|value| value.checked_add(usize_u64(plan.index_count).checked_mul(4)?))
            .ok_or_else(|| {
                fatal(
                    "M3A-INTEGER-OVERFLOW",
                    "workBytes",
                    "output geometry byte product overflow",
                )
            })?;
        output_bytes = output_bytes.checked_add(bytes).ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "output geometry byte sum overflow",
            )
        })?;
        if segment.deformation == RigSegmentDeformationV1::Skin {
            weight_evaluations = weight_evaluations
                .checked_add(
                    usize_u64(plan.vertex_count)
                        .checked_mul(usize_u64(segment.surface_indices.len() / 3))
                        .ok_or_else(|| {
                            fatal(
                                "M3A-INTEGER-OVERFLOW",
                                "distanceEvaluations",
                                "weight evaluation product overflow",
                            )
                        })?,
                )
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTEGER-OVERFLOW",
                        "distanceEvaluations",
                        "weight evaluation sum overflow",
                    )
                })?;
            for triangle in segment.surface_indices.chunks_exact(3) {
                let influences = triangle
                    .iter()
                    .try_fold(0usize, |sum, index| {
                        sum.checked_add(segment.reference_weights[*index as usize].len())
                    })
                    .ok_or_else(|| {
                        fatal(
                            "M3A-INTEGER-OVERFLOW",
                            "workBytes",
                            "weight influence count overflow",
                        )
                    })?;
                max_weight_temp_bytes = max_weight_temp_bytes.max(
                    usize_u64(influences)
                        .checked_mul(WORK_U32_F64_PAIR)
                        .ok_or_else(|| {
                            fatal(
                                "M3A-INTEGER-OVERFLOW",
                                "workBytes",
                                "weight temporary byte product overflow",
                            )
                        })?,
                );
            }
        }
    }
    let expected_distance_evaluations = assignment_evaluations
        .checked_add(weight_evaluations)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "distanceEvaluations",
                "total evaluation sum overflow",
            )
        })?;
    if expected_distance_evaluations > limits.max_distance_evaluations {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "distanceEvaluations",
            "total evaluation limit exceeded before output allocation",
        ));
    }
    let mapping_scratch = instances
        .iter()
        .map(|instance| instance.primitive.positions.len())
        .max()
        .unwrap_or(0);
    let mapping_scratch = usize_u64(mapping_scratch)
        .checked_mul(WORK_OPTION_U32)
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "mapping scratch byte product overflow",
            )
        })?;
    let mut output_peak = retained_work_bytes;
    reserve_work_bytes(&mut output_peak, output_bytes, limits)?;
    reserve_work_bytes(&mut output_peak, mapping_scratch, limits)?;
    reserve_work_bytes(
        &mut output_peak,
        max_weight_temp_bytes.checked_mul(2).ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "workBytes",
                "weight scratch byte product overflow",
            )
        })?,
        limits,
    )?;
    work_bytes_peak = work_bytes_peak.max(output_peak);
    let mut mixed_tangent_buckets = Vec::new();
    mixed_tangent_buckets
        .try_reserve(buckets.len())
        .map_err(|_| {
            fatal(
                "M3A-LIMIT-EXCEEDED",
                "creature.segments.tangents",
                "mixed tangent bucket allocation failed",
            )
        })?;
    mixed_tangent_buckets.extend(
        buckets
            .iter()
            .filter(|(_, plan)| plan.tangent_present && plan.tangent_absent)
            .map(|(key, _)| *key),
    );
    Ok(AssignmentPlan {
        triangle_segments,
        buckets,
        segment_order,
        mixed_tangent_buckets,
        has_unreferenced_vertices,
        expected_distance_evaluations,
        work_bytes_peak,
    })
}

fn material_slot_for(
    material_id: Option<u32>,
    bindings: &[MaterialSourceBindingV1],
) -> Result<u32, ProfileAConversionFatalError> {
    bindings
        .iter()
        .find(|binding| binding.source_material_id == material_id)
        .map(|binding| binding.slot)
        .ok_or_else(|| {
            fatal(
                "M3A-INTERNAL-CONTRACT",
                "materials.bindings",
                "material binding is missing",
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn emit_assigned_geometry(
    instances: &[GeometryInstance<'_>],
    plan: &AssignmentPlan,
    conversion: Mat4,
    rig: &CreatureRigProfileV1,
    rig_worlds: &BTreeMap<u32, Mat4>,
    gates: &mut Vec<ProfileAGateV1>,
    counters: &mut Counters,
    limits: &ProfileALimitsV1,
) -> Result<BTreeMap<(u32, u32), AuroraCreatureSegmentV1>, ProfileAConversionFatalError> {
    counters.work_bytes_peak = counters.work_bytes_peak.max(plan.work_bytes_peak);
    let mut buckets = BTreeMap::new();
    for (&key, bucket_plan) in &plan.buckets {
        let segment = &rig.segments[bucket_plan.segment_index];
        let mut bucket = AuroraCreatureSegmentV1 {
            segment_id: segment.id,
            material_slot: key.1,
            deformation: segment.deformation.clone(),
            parent_node_id: segment.parent_node_id,
            positions: Vec::new(),
            normals: Vec::new(),
            tangents: bucket_plan.tangent_present.then(Vec::new),
            uv0: Vec::new(),
            indices: Vec::new(),
            weights: Vec::new(),
        };
        bucket
            .positions
            .try_reserve(bucket_plan.vertex_count)
            .map_err(|_| {
                fatal(
                    "M3A-LIMIT-EXCEEDED",
                    "creature.segments.positions",
                    "output position allocation failed",
                )
            })?;
        bucket
            .normals
            .try_reserve(bucket_plan.vertex_count)
            .map_err(|_| {
                fatal(
                    "M3A-LIMIT-EXCEEDED",
                    "creature.segments.normals",
                    "output normal allocation failed",
                )
            })?;
        bucket
            .uv0
            .try_reserve(bucket_plan.vertex_count)
            .map_err(|_| {
                fatal(
                    "M3A-LIMIT-EXCEEDED",
                    "creature.segments.uv0",
                    "output UV allocation failed",
                )
            })?;
        bucket
            .indices
            .try_reserve(bucket_plan.index_count)
            .map_err(|_| {
                fatal(
                    "M3A-LIMIT-EXCEEDED",
                    "creature.segments.indices",
                    "output index allocation failed",
                )
            })?;
        if let Some(tangents) = bucket.tangents.as_mut() {
            tangents
                .try_reserve(bucket_plan.vertex_count)
                .map_err(|_| {
                    fatal(
                        "M3A-LIMIT-EXCEEDED",
                        "creature.segments.tangents",
                        "output tangent allocation failed",
                    )
                })?;
        }
        if segment.deformation == RigSegmentDeformationV1::Skin {
            bucket
                .weights
                .try_reserve(bucket_plan.vertex_count)
                .map_err(|_| {
                    fatal(
                        "M3A-LIMIT-EXCEEDED",
                        "creature.segments.weights",
                        "output weight allocation failed",
                    )
                })?;
        }
        buckets.insert(key, bucket);
    }

    for (instance_index, instance) in instances.iter().enumerate() {
        let primitive = instance.primitive;
        let assignments = &plan.triangle_segments[instance_index];
        for &segment_index in plan
            .segment_order
            .iter()
            .filter(|index| assignments.contains(index))
        {
            let segment = &rig.segments[segment_index];
            let material_slot = plan
                .buckets
                .keys()
                .find(|key| key.0 == segment.id)
                .map(|key| key.1)
                .ok_or_else(|| {
                    fatal(
                        "M3A-INTERNAL-CONTRACT",
                        "materials.bindings",
                        "planned material slot is missing",
                    )
                })?;
            let key = (segment.id, material_slot);
            let bucket = buckets.get_mut(&key).ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "creature.segments",
                    "planned output bucket is missing",
                )
            })?;
            let parent_world = *rig_worlds.get(&segment.parent_node_id).ok_or_else(|| {
                fatal(
                    "M3A-INTERNAL-CONTRACT",
                    "rig.segments.parentNodeId",
                    "validated rig parent is missing",
                )
            })?;
            let parent_inverse = parent_world.inverse_affine().ok_or_else(|| {
                fatal(
                    "M3A-PROFILE-HIERARCHY-INVALID",
                    "rig.nodes.bindLocalMatrix",
                    "rig parent bind world is singular",
                )
            })?;
            let total_matrix = parent_inverse.mul(conversion).mul(instance.source_world);
            let target_matrix = conversion.mul(instance.source_world);
            let normal_matrix = total_matrix.inverse_transpose_linear().ok_or_else(|| {
                fatal(
                    "M3A-NONFINITE-FLOAT",
                    "source.ir.nodes.transform",
                    "geometry transform is singular",
                )
            })?;
            let linear = total_matrix.linear();
            let parity = determinant3(linear);
            if !parity.is_finite() || parity.abs() <= 1.0e-12 {
                return Err(fatal(
                    "M3A-NONFINITE-FLOAT",
                    "source.ir.nodes.transform",
                    "composite geometry transform is singular",
                ));
            }
            let mut mapping = Vec::new();
            mapping
                .try_reserve(primitive.positions.len())
                .map_err(|_| {
                    fatal(
                        "M3A-LIMIT-EXCEEDED",
                        "creature.segments",
                        "output vertex mapping allocation failed",
                    )
                })?;
            mapping.resize(primitive.positions.len(), None);
            for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
                if assignments[triangle_index] != segment_index {
                    continue;
                }
                let emitted = if parity < 0.0 {
                    [triangle[0], triangle[2], triangle[1]]
                } else {
                    [triangle[0], triangle[1], triangle[2]]
                };
                for &source_index in triangle {
                    let source_index_usize = source_index as usize;
                    if mapping[source_index_usize].is_none() {
                        let index = u32::try_from(bucket.positions.len()).map_err(|_| {
                            fatal(
                                "M3A-INTEGER-OVERFLOW",
                                "creature.segments.positions",
                                "output vertex index does not fit u32",
                            )
                        })?;
                        bucket.positions.push(
                            total_matrix
                                .transform_point(primitive.positions[source_index_usize])?,
                        );
                        bucket.normals.push(normalize(
                            mul3(normal_matrix, primitive.normals[source_index_usize]),
                            "source.ir.primitives.normals",
                        )?);
                        if let Some(tangents) = bucket.tangents.as_mut() {
                            let source = primitive.tangents[source_index_usize];
                            let xyz = normalize(
                                mul3(linear, [source[0], source[1], source[2]]),
                                "source.ir.primitives.tangents",
                            )?;
                            tangents.push([xyz[0], xyz[1], xyz[2], source[3] * parity.signum()]);
                        }
                        bucket.uv0.push([
                            primitive.uv0[source_index_usize][0],
                            1.0 - primitive.uv0[source_index_usize][1],
                        ]);
                        match segment.deformation {
                            RigSegmentDeformationV1::Rigid => {
                                counters.rigid_vertices =
                                    checked_add(counters.rigid_vertices, 1, "rigid vertices")?
                            }
                            RigSegmentDeformationV1::Skin => {
                                let target_position = target_matrix
                                    .transform_point(primitive.positions[source_index_usize])?;
                                bucket.weights.push(transfer_skin_weights(
                                    target_position,
                                    segment,
                                    parent_world,
                                    gates,
                                    counters,
                                    limits,
                                )?);
                                counters.skinned_vertices =
                                    checked_add(counters.skinned_vertices, 1, "skinned vertices")?;
                            }
                        }
                        counters.normals = checked_add(counters.normals, 1, "normal transforms")?;
                        counters.tangents = checked_add(
                            counters.tangents,
                            usize::from(!primitive.tangents.is_empty()),
                            "tangent transforms",
                        )?;
                        counters.uv = checked_add(counters.uv, 1, "UV transforms")?;
                        mapping[source_index_usize] = Some(index);
                    }
                }
                for &source_index in &emitted {
                    let output_index = mapping[source_index as usize].ok_or_else(|| {
                        fatal(
                            "M3A-INTERNAL-CONTRACT",
                            "creature.segments.indices",
                            "output vertex mapping is missing",
                        )
                    })?;
                    bucket.indices.push(output_index);
                }
                if parity < 0.0 {
                    counters.winding = checked_add(counters.winding, 1, "winding reversals")?;
                }
            }
        }
    }
    if counters.distance_evaluations != plan.expected_distance_evaluations {
        return Err(fatal(
            "M3A-INTERNAL-CONTRACT",
            "distanceEvaluations",
            "distance evaluation accounting does not match the exact plan",
        ));
    }
    Ok(buckets)
}

fn transfer_skin_weights(
    point: [f32; 3],
    segment: &CreatureRigSegmentV1,
    parent_world: Mat4,
    gates: &mut Vec<ProfileAGateV1>,
    counters: &mut Counters,
    limits: &ProfileALimitsV1,
) -> Result<AuroraVertexWeightsV1, ProfileAConversionFatalError> {
    let mut nearest: Option<(f64, usize, [f64; 3])> = None;
    for (triangle_index, triangle) in segment.surface_indices.chunks_exact(3).enumerate() {
        let a = parent_world.transform_point(segment.surface_positions[triangle[0] as usize])?;
        let b = parent_world.transform_point(segment.surface_positions[triangle[1] as usize])?;
        let c = parent_world.transform_point(segment.surface_positions[triangle[2] as usize])?;
        let (distance, barycentric) =
            evaluated_point_triangle(point, a, b, c, &mut counters.distance_evaluations, limits)?;
        if nearest.is_none_or(|current| distance < current.0) {
            nearest = Some((distance, triangle_index, barycentric));
        }
    }
    let (_, triangle_index, barycentric) = nearest.ok_or_else(|| {
        fatal(
            "M3A-INTERNAL-CONTRACT",
            "rig.segments.surfaceIndices",
            "validated skin surface has no triangle",
        )
    })?;
    let triangle = &segment.surface_indices[triangle_index * 3..triangle_index * 3 + 3];
    let capacity = triangle
        .iter()
        .try_fold(0usize, |sum, index| {
            sum.checked_add(segment.reference_weights[*index as usize].len())
        })
        .ok_or_else(|| {
            fatal(
                "M3A-INTEGER-OVERFLOW",
                "creature.segments.weights",
                "interpolated influence count overflow",
            )
        })?;
    let mut influences = Vec::<(u32, f64)>::new();
    influences.try_reserve(capacity).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "creature.segments.weights",
            "interpolated influence allocation failed",
        )
    })?;
    for corner in 0..3 {
        for influence in &segment.reference_weights[triangle[corner] as usize] {
            influences.push((
                influence.bone_node_id,
                f64::from(influence.value) * barycentric[corner],
            ));
        }
    }
    influences.sort_by_key(|item| item.0);
    let mut merged = Vec::<(u32, f64)>::new();
    merged.try_reserve(influences.len()).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "creature.segments.weights",
            "merged influence allocation failed",
        )
    })?;
    for (bone, value) in influences {
        if let Some(last) = merged.last_mut().filter(|last| last.0 == bone) {
            last.1 += value;
            counters.merged_duplicate_influences = checked_add(
                counters.merged_duplicate_influences,
                1,
                "merged duplicate influences",
            )?;
        } else {
            merged.push((bone, value));
        }
    }
    let mut forbidden = false;
    merged.retain(|(bone, value)| {
        if *value > 0.0 && !segment.allowed_bone_node_ids.contains(bone) {
            forbidden = true;
            false
        } else {
            true
        }
    });
    if forbidden {
        push_gate_checked(
            gates,
            gate(
                "M3A-WEIGHT-BONE-FORBIDDEN",
                &format!("rig.segments[{}].referenceWeights", segment.id),
                "interpolated influence references a bone outside allowedBoneNodeIds",
            ),
            limits,
        )?;
    }
    let before_zero = merged.len();
    merged.retain(|(_, value)| *value > 0.0);
    counters.dropped_zero_influences = checked_add(
        counters.dropped_zero_influences,
        before_zero - merged.len(),
        "dropped zero influences",
    )?;
    merged.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    counters.max_influences_before = counters.max_influences_before.max(usize_u64(merged.len()));
    if merged.len() > 4 {
        counters.dropped_top_four_influences = checked_add(
            counters.dropped_top_four_influences,
            merged.len() - 4,
            "dropped top-four influences",
        )?;
        merged.truncate(4);
    }
    counters.max_influences_after = counters.max_influences_after.max(usize_u64(merged.len()));
    let sum = merged.iter().fold(0.0_f64, |sum, item| sum + item.1);
    let invalid_sum = !sum.is_finite() || sum <= 0.0;
    if invalid_sum {
        push_gate_checked(
            gates,
            gate(
                "M3A-WEIGHT-SUM-INVALID",
                &format!("creature.segments[{}].weights", segment.id),
                "interpolated weight sum must be positive and finite",
            ),
            limits,
        )?;
    }
    let mut result = AuroraVertexWeightsV1 {
        bone_node_ids: [None; 4],
        values: [0.0; 4],
        influence_count: u8::try_from(merged.len()).unwrap_or(4),
    };
    if !invalid_sum {
        for (index, (bone, value)) in merged.iter().enumerate() {
            result.bone_node_ids[index] = Some(*bone);
            result.values[index] = (*value / sum) as f32;
        }
        let normalized_sum = result.values.iter().sum::<f32>();
        if !normalized_sum.is_finite() || (normalized_sum - 1.0).abs() > 1.0e-5 {
            push_gate_checked(
                gates,
                gate(
                    "M3A-WEIGHT-SUM-INVALID",
                    &format!("creature.segments[{}].weights", segment.id),
                    "normalized weight sum is outside tolerance",
                ),
                limits,
            )?;
        } else {
            counters.normalized_vertices =
                checked_add(counters.normalized_vertices, 1, "normalized vertices")?;
        }
    }
    Ok(result)
}

fn evaluated_point_triangle(
    point: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    counter: &mut u64,
    limits: &ProfileALimitsV1,
) -> Result<(f64, [f64; 3]), ProfileAConversionFatalError> {
    let next = counter.checked_add(1).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "distanceEvaluations",
            "distance evaluation count overflow",
        )
    })?;
    if next > limits.max_distance_evaluations {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "distanceEvaluations",
            "distance evaluation limit exceeded before evaluation",
        ));
    }
    *counter = next;
    closest_point_triangle(point, a, b, c)
}

fn closest_point_triangle(
    point: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> Result<(f64, [f64; 3]), ProfileAConversionFatalError> {
    let p = point.map(f64::from);
    let a = a.map(f64::from);
    let b = b.map(f64::from);
    let c = c.map(f64::from);
    let sub = |x: [f64; 3], y: [f64; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
    let dot = |x: [f64; 3], y: [f64; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
    let finish = |bary: [f64; 3]| {
        let closest = [
            bary[0] * a[0] + bary[1] * b[0] + bary[2] * c[0],
            bary[0] * a[1] + bary[1] * b[1] + bary[2] * c[1],
            bary[0] * a[2] + bary[1] * b[2] + bary[2] * c[2],
        ];
        let delta = sub(p, closest);
        (dot(delta, delta), bary)
    };
    let ab = sub(b, a);
    let ac = sub(c, a);
    let area = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let area_sq = dot(area, area);
    if !area_sq.is_finite() || area_sq == 0.0 {
        return Err(fatal(
            "M3A-PROFILE-SEGMENT-INVALID",
            "rig.segments.surfacePositions",
            "reference surface triangle degenerates in target world",
        ));
    }
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return Ok(finish([1.0, 0.0, 0.0]));
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return Ok(finish([0.0, 1.0, 0.0]));
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return Ok(finish([1.0 - v, v, 0.0]));
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return Ok(finish([0.0, 0.0, 1.0]));
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return Ok(finish([1.0 - w, 0.0, w]));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return Ok(finish([0.0, 1.0 - w, w]));
    }
    let denominator = 1.0 / (va + vb + vc);
    let v = vb * denominator;
    let w = vc * denominator;
    let result = finish([1.0 - v - w, v, w]);
    if !result.0.is_finite() || result.1.iter().any(|value| !value.is_finite()) {
        return Err(fatal(
            "M3A-NONFINITE-FLOAT",
            "segmentAssignment",
            "closest-point result is non-finite",
        ));
    }
    Ok(result)
}

fn matrix_from_transform(transform: &IrTransform) -> Result<Mat4, ProfileAConversionFatalError> {
    let matrix = match transform.kind.as_str() {
        "MATRIX" => Mat4(transform.matrix.ok_or_else(|| {
            fatal(
                "M3A-INTERNAL-CONTRACT",
                "source.ir.nodes.transform.matrix",
                "MATRIX transform has no matrix",
            )
        })?),
        "TRS" => Mat4::from_trs(
            transform.translation.unwrap_or([0.0, 0.0, 0.0]),
            transform.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]),
            transform.scale.unwrap_or([1.0, 1.0, 1.0]),
        )?,
        _ => {
            return Err(fatal(
                "M3A-INTERNAL-CONTRACT",
                "source.ir.nodes.transform.kind",
                "unknown source transform kind",
            ));
        }
    };
    if !matrix.is_finite() {
        return Err(fatal(
            "M3A-NONFINITE-FLOAT",
            "source.ir.nodes.transform",
            "source transform contains non-finite values",
        ));
    }
    Ok(matrix)
}

impl Mat4 {
    fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }
    fn is_finite(self) -> bool {
        self.0.iter().all(|value| value.is_finite())
    }
    fn get(self, row: usize, col: usize) -> f32 {
        self.0[col * 4 + row]
    }
    fn set(&mut self, row: usize, col: usize, value: f32) {
        self.0[col * 4 + row] = value;
    }
    fn mul(self, rhs: Self) -> Self {
        let mut result = Self([0.0; 16]);
        for row in 0..4 {
            for col in 0..4 {
                result.set(
                    row,
                    col,
                    (0..4).map(|k| self.get(row, k) * rhs.get(k, col)).sum(),
                );
            }
        }
        result
    }
    fn transform_point(self, point: [f32; 3]) -> Result<[f32; 3], ProfileAConversionFatalError> {
        let result = [
            self.get(0, 0) * point[0]
                + self.get(0, 1) * point[1]
                + self.get(0, 2) * point[2]
                + self.get(0, 3),
            self.get(1, 0) * point[0]
                + self.get(1, 1) * point[1]
                + self.get(1, 2) * point[2]
                + self.get(1, 3),
            self.get(2, 0) * point[0]
                + self.get(2, 1) * point[1]
                + self.get(2, 2) * point[2]
                + self.get(2, 3),
        ];
        finite3(result, "transform.position")?;
        Ok(result)
    }
    fn linear(self) -> [[f32; 3]; 3] {
        [
            [self.get(0, 0), self.get(0, 1), self.get(0, 2)],
            [self.get(1, 0), self.get(1, 1), self.get(1, 2)],
            [self.get(2, 0), self.get(2, 1), self.get(2, 2)],
        ]
    }
    fn inverse_transpose_linear(self) -> Option<[[f32; 3]; 3]> {
        inverse3(self.linear()).map(transpose3)
    }
    fn inverse_affine(self) -> Option<Self> {
        if (self.get(3, 0)).abs() > 1e-6
            || (self.get(3, 1)).abs() > 1e-6
            || (self.get(3, 2)).abs() > 1e-6
            || (self.get(3, 3) - 1.0).abs() > 1e-6
        {
            return None;
        }
        let inv = inverse3(self.linear())?;
        let t = [self.get(0, 3), self.get(1, 3), self.get(2, 3)];
        let it = mul3(inv, [-t[0], -t[1], -t[2]]);
        Some(Self([
            inv[0][0], inv[1][0], inv[2][0], 0.0, inv[0][1], inv[1][1], inv[2][1], 0.0, inv[0][2],
            inv[1][2], inv[2][2], 0.0, it[0], it[1], it[2], 1.0,
        ]))
    }
    fn from_scale_basis_translation(scale: f32, translation: [f32; 3]) -> Self {
        Self([
            scale,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            scale,
            0.0,
            0.0,
            scale,
            0.0,
            0.0,
            translation[0],
            translation[1],
            translation[2],
            1.0,
        ])
    }
    fn from_trs(
        t: [f32; 3],
        q: [f32; 4],
        s: [f32; 3],
    ) -> Result<Self, ProfileAConversionFatalError> {
        finite3(t, "source.ir.nodes.transform.translation")?;
        finite3(s, "source.ir.nodes.transform.scale")?;
        if q.iter().any(|value| !value.is_finite()) {
            return Err(fatal(
                "M3A-NONFINITE-FLOAT",
                "source.ir.nodes.transform.rotation",
                "rotation is non-finite",
            ));
        }
        let len = (q.iter().map(|v| v * v).sum::<f32>()).sqrt();
        if !len.is_finite() || len <= f32::EPSILON {
            return Err(fatal(
                "M3A-NONFINITE-FLOAT",
                "source.ir.nodes.transform.rotation",
                "rotation quaternion has zero length",
            ));
        }
        let [x, y, z, w] = [q[0] / len, q[1] / len, q[2] / len, q[3] / len];
        let r = [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - z * w),
                2.0 * (x * z + y * w),
            ],
            [
                2.0 * (x * y + z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - x * w),
            ],
            [
                2.0 * (x * z - y * w),
                2.0 * (y * z + x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ];
        Ok(Self([
            r[0][0] * s[0],
            r[1][0] * s[0],
            r[2][0] * s[0],
            0.0,
            r[0][1] * s[1],
            r[1][1] * s[1],
            r[2][1] * s[1],
            0.0,
            r[0][2] * s[2],
            r[1][2] * s[2],
            r[2][2] * s[2],
            0.0,
            t[0],
            t[1],
            t[2],
            1.0,
        ]))
    }
}

impl Bounds3V1 {
    fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }
    fn include(&mut self, p: [f32; 3]) {
        for (axis, coordinate) in p.iter().enumerate() {
            self.min[axis] = self.min[axis].min(*coordinate);
            self.max[axis] = self.max[axis].max(*coordinate);
        }
    }
    fn ensure_nonempty(self, path: &str) -> Result<Self, ProfileAConversionFatalError> {
        if self.min.iter().all(|v| v.is_finite()) && self.max.iter().all(|v| v.is_finite()) {
            Ok(self)
        } else {
            Err(fatal(
                "M3A-INTERNAL-CONTRACT",
                path,
                "bounds are empty or non-finite",
            ))
        }
    }
}

fn basis_matrix() -> Mat4 {
    Mat4([
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}
fn transform_bounds(bounds: Bounds3V1, matrix: Mat4) -> Bounds3V1 {
    let mut out = Bounds3V1::empty();
    for x in [bounds.min[0], bounds.max[0]] {
        for y in [bounds.min[1], bounds.max[1]] {
            for z in [bounds.min[2], bounds.max[2]] {
                out.include(
                    matrix
                        .transform_point([x, y, z])
                        .expect("finite checked bounds"),
                );
            }
        }
    }
    out
}
fn scale_bounds(bounds: Bounds3V1, scale: f32) -> Bounds3V1 {
    Bounds3V1 {
        min: [
            bounds.min[0] * scale,
            bounds.min[1] * scale,
            bounds.min[2] * scale,
        ],
        max: [
            bounds.max[0] * scale,
            bounds.max[1] * scale,
            bounds.max[2] * scale,
        ],
    }
}
fn translate_bounds(bounds: Bounds3V1, t: [f32; 3]) -> Bounds3V1 {
    Bounds3V1 {
        min: add3(bounds.min, t),
        max: add3(bounds.max, t),
    }
}
fn bottom_center(bounds: Bounds3V1) -> [f32; 3] {
    [
        (bounds.min[0] + bounds.max[0]) * 0.5,
        (bounds.min[1] + bounds.max[1]) * 0.5,
        bounds.min[2],
    ]
}
fn height(bounds: Bounds3V1) -> f32 {
    bounds.max[2] - bounds.min[2]
}
fn bounds_approx(a: Bounds3V1, b: Bounds3V1, t: f32) -> bool {
    (0..3).all(|i| (a.min[i] - b.min[i]).abs() <= t && (a.max[i] - b.max[i]).abs() <= t)
}
fn bounds_from_segments_world(
    segments: &[AuroraCreatureSegmentV1],
    worlds: &BTreeMap<u32, Mat4>,
) -> Result<Bounds3V1, ProfileAConversionFatalError> {
    let mut b = Bounds3V1::empty();
    for s in segments {
        let world = *worlds.get(&s.parent_node_id).ok_or_else(|| {
            fatal(
                "M3A-INTERNAL-CONTRACT",
                "creature.segments.parentNodeId",
                "rig parent world is missing",
            )
        })?;
        for &p in &s.positions {
            b.include(world.transform_point(p)?);
        }
    }
    b.ensure_nonempty("creature.segments.positions")
}
fn validate_bounds(bounds: Bounds3V1, path: &str) -> Result<(), ProfileAConversionFatalError> {
    finite3(bounds.min, path)?;
    finite3(bounds.max, path)?;
    if (0..3).any(|i| bounds.max[i] <= bounds.min[i]) {
        return Err(fatal(
            "M3A-PROFILE-BOUNDS-INVALID",
            path,
            "target bounds must have positive extent on every axis",
        ));
    }
    Ok(())
}
fn finite3(v: [f32; 3], path: &str) -> Result<(), ProfileAConversionFatalError> {
    if v.iter().all(|x| x.is_finite()) {
        Ok(())
    } else {
        Err(fatal("M3A-NONFINITE-FLOAT", path, "non-finite vector"))
    }
}
fn normalize(v: [f32; 3], path: &str) -> Result<[f32; 3], ProfileAConversionFatalError> {
    let l = length_sq(v).sqrt();
    if !l.is_finite() || l <= f32::EPSILON {
        return Err(fatal(
            "M3A-NONFINITE-FLOAT",
            path,
            "zero or non-finite direction",
        ));
    }
    Ok([v[0] / l, v[1] / l, v[2] / l])
}
fn inverse3(m: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    let d = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if !d.is_finite() || d.abs() <= 1e-12 {
        return None;
    }
    let i = 1.0 / d;
    Some([
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * i,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * i,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * i,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * i,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * i,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * i,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * i,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * i,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * i,
        ],
    ])
}
fn determinant3(m: [[f32; 3]; 3]) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}
fn transpose3(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}
fn mul3(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn length_sq(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}
fn reserve_work_bytes(
    total: &mut u64,
    bytes: u64,
    limits: &ProfileALimitsV1,
) -> Result<(), ProfileAConversionFatalError> {
    *total = total.checked_add(bytes).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "workBytes",
            "work byte sum overflow",
        )
    })?;
    if *total > limits.max_work_bytes {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "workBytes",
            "work byte limit exceeded",
        ));
    }
    Ok(())
}
fn checked_add(a: u64, b: usize, label: &str) -> Result<u64, ProfileAConversionFatalError> {
    a.checked_add(usize_u64(b)).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "report",
            format!("{label} overflow"),
        )
    })
}
fn usize_u64(value: usize) -> u64 {
    u64::try_from(value).expect("usize fits u64 on supported wasm32 and 64-bit native targets")
}
fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        write!(&mut out, "{b:02x}").expect("String write");
    }
    out
}
fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn fatal(code: &str, path: &str, message: impl Into<String>) -> ProfileAConversionFatalError {
    ProfileAConversionFatalError::new(code, path, message)
}
fn source_mismatch(path: &str, message: &str) -> ProfileAConversionFatalError {
    fatal("M3A-SOURCE-CONTRACT-MISMATCH", path, message)
}
fn gate(code: &str, path: &str, message: &str) -> ProfileAGateV1 {
    ProfileAGateV1 {
        schema_version: 1,
        code: code.to_owned(),
        severity: "BLOCKING".to_owned(),
        path: path.to_owned(),
        expected: "policy satisfied".to_owned(),
        actual: "policy violated".to_owned(),
        message: message.to_owned(),
    }
}
fn push_gate_checked(
    gates: &mut Vec<ProfileAGateV1>,
    value: ProfileAGateV1,
    limits: &ProfileALimitsV1,
) -> Result<(), ProfileAConversionFatalError> {
    if gates
        .iter()
        .any(|existing| existing.code == value.code && existing.path == value.path)
    {
        return Ok(());
    }
    let next = usize_u64(gates.len()).checked_add(1).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "report.gates",
            "gate count overflow",
        )
    })?;
    if next > limits.max_diagnostics {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "report.gates",
            "diagnostic limit exceeded",
        ));
    }
    gates.try_reserve(1).map_err(|_| {
        fatal(
            "M3A-LIMIT-EXCEEDED",
            "report.gates",
            "gate allocation failed",
        )
    })?;
    gates.push(value);
    Ok(())
}
fn warning(code: &str, path: &str, message: &str) -> ProfileAGateV1 {
    ProfileAGateV1 {
        schema_version: 1,
        code: code.to_owned(),
        severity: "WARNING".to_owned(),
        path: path.to_owned(),
        expected: "within warning threshold".to_owned(),
        actual: "above warning threshold".to_owned(),
        message: message.to_owned(),
    }
}
fn finalize_gates(gates: &mut Vec<ProfileAGateV1>) {
    fn phase(code: &str) -> u8 {
        match code {
            "M3A-PROFILE-PROVENANCE-FORBIDDEN" => 0,
            "M3A-SOURCE-BLOCKED" | "M3A-SOURCE-RIG-DEFERRED" | "M3A-SOURCE-ANIMATION-DEFERRED" => 1,
            "M3A-DEFAULT-SCENE-REQUIRED" | "M3A-DEFAULT-SCENE-HIERARCHY-INVALID" => 2,
            "M3A-NORMALS-REQUIRED"
            | "M3A-TANGENT-COVERAGE-MIXED"
            | "M3A-MATERIAL-LIMIT"
            | "M3A-TRIANGLE-BUDGET-WARNING"
            | "M3A-TRIANGLE-BUDGET-BLOCKING"
            | "M3A-ZERO-HEIGHT" => 3,
            "M3A-SEGMENT-ASSIGNMENT-FAILED"
            | "M3A-WEIGHT-BONE-FORBIDDEN"
            | "M3A-WEIGHT-SUM-INVALID" => 4,
            _ => 5,
        }
    }
    fn severity(value: &str) -> u8 {
        if value == "WARNING" { 0 } else { 1 }
    }
    gates.sort_by(|a, b| {
        phase(&a.code)
            .cmp(&phase(&b.code))
            .then(a.path.as_bytes().cmp(b.path.as_bytes()))
            .then(a.code.as_bytes().cmp(b.code.as_bytes()))
            .then(severity(&a.severity).cmp(&severity(&b.severity)))
    });
    gates.dedup_by(|a, b| a.code == b.code && a.path == b.path);
}
fn ensure_diagnostic_limit(
    gate_count: usize,
    diagnostic_count: usize,
    limits: &ProfileALimitsV1,
) -> Result<(), ProfileAConversionFatalError> {
    let total = gate_count.checked_add(diagnostic_count).ok_or_else(|| {
        fatal(
            "M3A-INTEGER-OVERFLOW",
            "report",
            "diagnostic count overflow",
        )
    })?;
    if usize_u64(total) > limits.max_diagnostics {
        return Err(fatal(
            "M3A-LIMIT-EXCEEDED",
            "report.gates",
            "diagnostic limit exceeded",
        ));
    }
    Ok(())
}
fn empty_transform_report(anchor: [f32; 3]) -> ProfileATransformReportV1 {
    ProfileATransformReportV1 {
        basis_matrix: basis_matrix().0,
        determinant: -1.0,
        source_world_bounds: None,
        after_basis_bounds: None,
        target_bounds: None,
        scale: None,
        source_bottom_center: None,
        alignment_anchor: anchor,
        translation: None,
    }
}
#[allow(clippy::too_many_arguments)]
fn report(
    eligible: bool,
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
    gates: Vec<ProfileAGateV1>,
    transform: ProfileATransformReportV1,
    c: Counters,
    source_selection: ProfileASourceSelectionReportV1,
    bindings: Vec<MaterialSourceBindingV1>,
    diagnostics: Vec<ProfileADiagnosticV1>,
    creature_sha256: Option<String>,
) -> ProfileAConversionReportV1 {
    let segments = c.segment_reports.clone();
    ProfileAConversionReportV1 {
        schema_version: 1,
        source: ProfileAReportSourceV1 {
            sha256: source.ir.source.sha256.clone(),
            byte_length: usize_u64(source.ir.source.byte_length),
            default_scene_id: source.ir.default_scene_id,
        },
        rig: ProfileAReportRigV1 {
            profile_id: rig.profile_id.clone(),
            content_sha256: rig.content_sha256.clone(),
            provenance_kind: rig.provenance.kind.clone(),
            export_allowed: rig.provenance.export_allowed,
            attestations: rig.provenance.attestations.clone(),
            all_attestations_satisfied: rig.provenance.attestations.controlled_construction
                && rig.provenance.attestations.no_reference_payload_copied
                && rig.provenance.attestations.rights_confirmed,
        },
        policies: ProfileAReportPoliciesV1 {
            basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
            basis_evidence: "REFERENCE_ONLY_IMPLEMENTATION_INFERENCE".to_owned(),
            asset_forward_mapping: "GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y".to_owned(),
            orientation_parity: "NEGATIVE_FOR_POSITIVE_SOURCE_AND_RIG_PARITY".to_owned(),
            uv_evidence: "REFERENCE_ONLY_IMPLEMENTATION_INFERENCE".to_owned(),
            uv_mapping: "GLTF_V_TO_ONE_MINUS_V".to_owned(),
            engine_facing_proof: "OPEN_M6".to_owned(),
            uv_runtime_proof: "OPEN_M6".to_owned(),
            source_scene_policy: "DEFAULT_SCENE_ONLY".to_owned(),
            alignment_policy: "BOTTOM_CENTER_TO_PROFILE_ANCHOR".to_owned(),
        },
        source_selection,
        transform,
        materials: ProfileAMaterialsReportV1 {
            unique_used_count: usize_u64(bindings.len()),
            max_unique_materials: options.limits.max_unique_materials,
            bindings,
        },
        geometry: ProfileAGeometryReportV1 {
            source_triangle_count: c.source_triangles,
            output_triangle_count: c.output_triangles,
            source_vertex_instance_count: c.source_vertices,
            output_vertex_count: c.output_vertices,
            duplicated_vertex_count: c.duplicated_vertices,
        },
        segments,
        weights: ProfileAWeightsReportV1 {
            skinned_vertex_count: c.skinned_vertices,
            rigid_vertex_count: c.rigid_vertices,
            merged_duplicate_influence_count: c.merged_duplicate_influences,
            dropped_zero_influence_count: c.dropped_zero_influences,
            dropped_after_top_four_count: c.dropped_top_four_influences,
            normalized_vertex_count: c.normalized_vertices,
            max_influences_before: c.max_influences_before,
            max_influences_after: c.max_influences_after,
        },
        work: ProfileAWorkReportV1 {
            distance_evaluations: c.distance_evaluations,
            max_distance_evaluations: options.limits.max_distance_evaluations,
            work_bytes_peak: c.work_bytes_peak,
        },
        gates,
        diagnostics,
        conversion_eligible: eligible,
        creature_sha256,
    }
}
#[allow(clippy::too_many_arguments)]
fn blocked_outcome(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
    gates: Vec<ProfileAGateV1>,
    transform: ProfileATransformReportV1,
    c: Counters,
    selection: ProfileASourceSelectionReportV1,
    bindings: Vec<MaterialSourceBindingV1>,
    diagnostics: Vec<ProfileADiagnosticV1>,
) -> ProfileAConversionOutcomeV1 {
    ProfileAConversionOutcomeV1 {
        schema_version: 1,
        source_sha256: source.ir.source.sha256.clone(),
        report: report(
            false,
            source,
            rig,
            options,
            gates,
            transform,
            c,
            selection,
            bindings,
            diagnostics,
            None,
        ),
        creature: None,
    }
}
