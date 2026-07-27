use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::types::{MeshReport, NodeReport, NodeTreeReport};
use super::writer_types::is_well_formed_state_projection_provenance_v1;
use super::{InspectionReport, MdlStateProjectionProfileV1, MdlStateProjectionProvenanceV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureEngineEnvelopeVerdictV1 {
    StructuralPassRuntimeNotWitnessed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineByteRangeV1 {
    pub start: u64,
    pub length: u64,
    pub end: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineControllerEnvelopeV1 {
    pub controller_type: i32,
    pub controller_name: Option<String>,
    pub packed_byte: u8,
    pub interpolation_flags: u8,
    pub decoded: bool,
    pub padding_byte: u8,
    pub row_count: u32,
    pub time_index: u32,
    pub data_index: u32,
    pub column_count: u32,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineRawStreamEnvelopeV1 {
    pub field: String,
    pub pointer: Option<i32>,
    pub byte_length: u64,
    pub stride: Option<u32>,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineMeshEnvelopeV1 {
    pub node_part: u32,
    pub node_name: String,
    pub parent_part: Option<u32>,
    pub textures: Vec<String>,
    pub texture_count: u32,
    pub vertex_count: u32,
    pub vertex_colors_present: bool,
    pub vertex_color_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub radius: f32,
    pub average: Option<[f32; 3]>,
    pub diffuse: [f32; 3],
    pub ambient: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub shadow: u32,
    pub beaming: u32,
    pub render: u32,
    pub transparency: u32,
    pub render_hint: u32,
    pub tile_fade: u32,
    pub mesh_type: u32,
    pub start_mdx: i32,
    pub face_count: u32,
    pub face_index_count: u32,
    pub index_counts: Vec<u32>,
    pub raw_index_offsets: Vec<i32>,
    pub max_vertex_index: Option<u16>,
    pub face_indices_match_raw_indices: bool,
    pub geometry_sha256: String,
    pub raw_streams: Vec<EngineRawStreamEnvelopeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineNodeEnvelopeV1 {
    pub order: u32,
    pub part: u32,
    pub name: String,
    pub parent_part: Option<u32>,
    pub content_flags: u32,
    pub inherit_color: u32,
    pub kind: String,
    pub children_used: u32,
    pub children_allocated: u32,
    pub controller_keys_used: u32,
    pub controller_keys_allocated: u32,
    pub controller_data_used: u32,
    pub controller_data_allocated: u32,
    pub controllers: Vec<EngineControllerEnvelopeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineAnimationEnvelopeV1 {
    pub order: u32,
    pub name: String,
    pub animation_type: u8,
    pub animation_type_padding: [u8; 3],
    pub animation_root: String,
    pub runtime_68: u32,
    pub geometry_array_50_used: u32,
    pub geometry_array_50_allocated: u32,
    pub geometry_array_5c_used: u32,
    pub geometry_array_5c_allocated: u32,
    pub event_count: u32,
    pub node_count: u32,
    pub max_depth: u32,
    pub topology_matches_base: bool,
    pub nodes: Vec<EngineNodeEnvelopeV1>,
}

/// A complete, payload-free structural account of one owned direct-creature
/// binary MDL/MDX.  The only positive verdict is deliberately named
/// `STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED`; it cannot promote visual evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureEngineEnvelopeV1 {
    pub schema_version: u32,
    pub profile: String,
    pub verdict: DirectCreatureEngineEnvelopeVerdictV1,
    pub binary_mdl_byte_length: u64,
    pub binary_mdl_sha256: String,
    pub binary_mdl_id: u32,
    pub core: EngineByteRangeV1,
    pub raw_mdx: EngineByteRangeV1,
    pub geometry_type: u32,
    pub classification: u8,
    pub fog: u8,
    pub supermodel_name: String,
    pub child_model_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub radius: f32,
    pub animation_scale: f32,
    pub animation_pointer_used: u32,
    pub animation_pointer_allocated: u32,
    pub base_node_count: u32,
    pub base_max_depth: u32,
    pub base_nodes: Vec<EngineNodeEnvelopeV1>,
    pub meshes: Vec<EngineMeshEnvelopeV1>,
    pub animations: Vec<EngineAnimationEnvelopeV1>,
    pub parser_diagnostics: Vec<String>,
    pub unsupported: Vec<String>,
    pub unassessed_fields: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlRuntimeConformanceErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for MdlRuntimeConformanceErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for MdlRuntimeConformanceErrorV1 {}

/// Payload-free structural projection used as a differential oracle for local
/// retail, owned H1, and CEP witnesses.  It intentionally records counts and
/// state-family facts, never model bytes, vertex streams, or animation data.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureStructuralSummaryV1 {
    pub schema_version: u32,
    pub model_name: String,
    pub byte_length: u64,
    pub base_node_count: u32,
    pub base_mesh_node_count: u32,
    pub base_skin_node_count: u32,
    pub base_max_depth: u32,
    pub animation_count: u32,
    pub animation_type_histogram: BTreeMap<u8, u32>,
    pub full_base_topology_projection_count: u32,
    pub all_generic_dummy_projection_count: u32,
    pub cep_rigid_placeholder_projection_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureStateSkinProfileSummaryV1 {
    pub schema_version: u32,
    pub base_skin_count: u32,
    pub animation_count: u32,
    pub matching_generic_node_count: u32,
    pub matching_generic_leaf_no_controller_count: u32,
    pub matching_state_skin_node_count: u32,
}

/// Tolerant, payload-free renderer-facing projection used only for differential
/// diagnosis. Unlike `DirectCreatureEngineEnvelopeV1`, this is not an M0
/// admission type and may represent foreign/read-only witnesses with explicitly
/// unavailable or unassessed fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureEnvelopeProjectionV1 {
    pub schema_version: u32,
    pub profile: String,
    pub binary_mdl_byte_length: u64,
    pub binary_mdl_sha256: String,
    pub model: DirectCreatureModelProjectionV1,
    pub base_nodes: Vec<DirectCreatureNodeProjectionV1>,
    pub meshes: Vec<DirectCreatureMeshProjectionV1>,
    pub animations: Vec<DirectCreatureAnimationProjectionV1>,
    pub field_assessments: Vec<DirectCreatureEnvelopeFieldAssessmentV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureModelProjectionV1 {
    pub name: String,
    pub geometry_type: u32,
    pub classification: u8,
    pub fog: u8,
    pub child_model_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub radius: f32,
    pub animation_scale: f32,
    pub supermodel_name: String,
    pub animation_count: u32,
    pub routine_words: [u32; 2],
    pub geometry_array_50: [u32; 3],
    pub geometry_array_5c: [u32; 3],
    pub geometry_ref_count: u32,
    pub geometry_type_padding: [u8; 3],
    pub runtime_unknown_bytes: [u8; 2],
    pub supermodel_pointer: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureNodeProjectionV1 {
    pub order: u32,
    pub part: u32,
    pub name: String,
    pub parent_part: Option<u32>,
    pub content_flags: u32,
    pub inherit_color: u32,
    pub kind: String,
    pub routine_words: [u32; 6],
    pub geometry_pointer: u32,
    pub parent_pointer: u32,
    pub children_used: u32,
    pub children_allocated: u32,
    pub controller_signature_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureMeshProjectionV1 {
    pub node_part: u32,
    pub node_name: String,
    pub parent_part: Option<u32>,
    pub routine_words: [u32; 2],
    pub vertex_count: u32,
    pub face_count: u32,
    pub texture_count: u32,
    pub textures: Vec<String>,
    pub vertex_colors_present: bool,
    pub vertex_color_count: u32,
    pub diffuse: [f32; 3],
    pub ambient: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub shadow: u32,
    pub beaming: u32,
    pub render: u32,
    pub transparency: u32,
    pub render_hint: u32,
    pub tile_fade: u32,
    pub mesh_type: u32,
    pub start_mdx: i32,
    pub light_mapped: u8,
    pub rotate_texture: u8,
    pub tail_padding: u16,
    pub vertex_normal_sum_bits: u32,
    pub tail_unknown: u32,
    pub face_surface_ids_sha256: String,
    pub face_adjacency_sha256: String,
    pub geometry_sha256: String,
    pub raw_pointer_signature_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureAnimationProjectionV1 {
    pub order: u32,
    pub name: String,
    pub animation_type: u8,
    pub animation_type_padding: [u8; 3],
    pub animation_root: String,
    pub runtime_68: u32,
    pub node_count: u32,
    pub max_depth: u32,
    pub topology_sha256: String,
    pub controller_signature_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureEnvelopeFieldStateV1 {
    Available,
    Unavailable,
    Unassessed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureEnvelopeFieldAssessmentV1 {
    pub path: String,
    pub state: DirectCreatureEnvelopeFieldStateV1,
    pub reason_code: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureEnvelopeDifferenceClassV1 {
    ArtifactIdentity,
    ModelHeader,
    ModelRuntimeDefaults,
    ModelBounds,
    BaseTopology,
    NodeRuntimeDefaults,
    MeshAttachment,
    MeshMaterial,
    MeshRuntimeDefaults,
    VertexColorPresence,
    FaceRuntimeDefaults,
    GeometryShape,
    RawMdxLayout,
    AnimationHeader,
    StateTopology,
    FieldAvailability,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureEnvelopeFieldDifferenceV1 {
    pub path: String,
    pub classification: DirectCreatureEnvelopeDifferenceClassV1,
    pub left: Option<serde_json::Value>,
    pub right: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureEnvelopeDifferentialV1 {
    pub schema_version: u32,
    pub profile: String,
    pub left_projection_sha256: String,
    pub right_projection_sha256: String,
    pub identical: bool,
    pub classification_counts: BTreeMap<DirectCreatureEnvelopeDifferenceClassV1, u32>,
    pub differences: Vec<DirectCreatureEnvelopeFieldDifferenceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NodeIdentity {
    number: u32,
    name: String,
    parent_number: Option<u32>,
    content_flags: u32,
    has_mesh: bool,
    has_skin: bool,
    zero_geometry_mesh: bool,
    child_count: usize,
    controller_count: usize,
}

/// Summarizes a parsed binary MDL without assuming one native family.  This is
/// safe for general reader output and lets tests compare witnesses without
/// copying protected payloads into the repository.
pub fn summarize_direct_creature_structure_v1(
    report: &InspectionReport,
) -> DirectCreatureStructuralSummaryV1 {
    let base = flatten_tree(&report.node_tree);
    let base_identity = identity_only(&base);
    let mut animation_type_histogram = BTreeMap::new();
    let mut full_base_topology_projection_count = 0u32;
    let mut all_generic_dummy_projection_count = 0u32;
    let mut cep_rigid_placeholder_projection_count = 0u32;

    for animation in &report.animations {
        *animation_type_histogram
            .entry(animation.animation_type)
            .or_insert(0) += 1;
        let state = flatten_tree(&animation.node_tree);
        if identity_only(&state) == base_identity {
            full_base_topology_projection_count += 1;
        }
        if state.iter().all(is_generic_dummy) {
            all_generic_dummy_projection_count += 1;
        }
        if state.len() == base.len()
            && state.iter().zip(&base).all(|(state, base)| {
                if base.content_flags & 0x20 != 0 {
                    state.content_flags == 0x21 && state.zero_geometry_mesh && !state.has_skin
                } else {
                    is_generic_dummy(state)
                }
            })
        {
            cep_rigid_placeholder_projection_count += 1;
        }
    }

    DirectCreatureStructuralSummaryV1 {
        schema_version: 1,
        model_name: report.model.name.clone(),
        byte_length: report.byte_length as u64,
        base_node_count: base.len() as u32,
        base_mesh_node_count: base.iter().filter(|node| node.has_mesh).count() as u32,
        base_skin_node_count: base.iter().filter(|node| node.has_skin).count() as u32,
        base_max_depth: report.node_tree.max_depth as u32,
        animation_count: report.animations.len() as u32,
        animation_type_histogram,
        full_base_topology_projection_count,
        all_generic_dummy_projection_count,
        cep_rigid_placeholder_projection_count,
    }
}

/// Counts the three observed native state-skin shapes without choosing one as
/// a global writer policy. Names use the same ASCII case-fold identity domain
/// enforced for caller-owned model nodes.
pub fn summarize_direct_creature_state_skin_profile_v1(
    report: &InspectionReport,
) -> DirectCreatureStateSkinProfileSummaryV1 {
    let base = flatten_tree(&report.node_tree);
    let base_skin_names = base
        .iter()
        .filter(|node| node.has_skin)
        .map(|node| node.name.to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    let mut matching_generic_node_count = 0u32;
    let mut matching_generic_leaf_no_controller_count = 0u32;
    let mut matching_state_skin_node_count = 0u32;
    for animation in &report.animations {
        for node in flatten_tree(&animation.node_tree) {
            if !base_skin_names.contains(&node.name.to_ascii_lowercase()) {
                continue;
            }
            if node.content_flags == 0x01 && !node.has_mesh && !node.has_skin {
                matching_generic_node_count += 1;
                if node.child_count == 0 && node.controller_count == 0 {
                    matching_generic_leaf_no_controller_count += 1;
                }
            }
            if node.content_flags == 0x61 && node.has_skin {
                matching_state_skin_node_count += 1;
            }
        }
    }
    DirectCreatureStateSkinProfileSummaryV1 {
        schema_version: 1,
        base_skin_count: base.iter().filter(|node| node.has_skin).count() as u32,
        animation_count: report.animations.len() as u32,
        matching_generic_node_count,
        matching_generic_leaf_no_controller_count,
        matching_state_skin_node_count,
    }
}

/// Hashes the canonical JSON form of the payload-free structural summary.
/// `BTreeMap` keeps the animation-type histogram ordered, so this digest is
/// stable across processes and can be persisted in runtime identity packets.
pub fn direct_creature_structural_summary_digest_v1(
    summary: &DirectCreatureStructuralSummaryV1,
) -> Result<String, MdlRuntimeConformanceErrorV1> {
    let bytes = serde_json::to_vec(summary).map_err(|error| {
        conformance_error(
            "M2A-MDL-CONFORMANCE-SUMMARY-SERIALIZATION",
            "stateProjectionSummary",
            &format!("structural summary cannot be canonically serialized: {error}"),
        )
    })?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Enforces the selected direct-creature state family.  The binary reader is
/// intentionally more tolerant; only caller-owned writer/runtime profiles use
/// this stricter oracle.
pub fn verify_direct_creature_state_projection_v1(
    report: &InspectionReport,
    profile: MdlStateProjectionProfileV1,
    provenance: Option<&MdlStateProjectionProvenanceV1>,
) -> Result<DirectCreatureStructuralSummaryV1, MdlRuntimeConformanceErrorV1> {
    verify_direct_creature_state_projection_with_expected_provenance_v1(
        report, profile, provenance, None,
    )
}

/// Enforces a profile with an expected provenance supplied independently from
/// the report/writer binding. CEP-shaped input is inadmissible through the
/// compatibility API above and must use this explicit trust boundary.
pub fn verify_direct_creature_state_projection_with_expected_provenance_v1(
    report: &InspectionReport,
    profile: MdlStateProjectionProfileV1,
    provenance: Option<&MdlStateProjectionProvenanceV1>,
    expected_provenance: Option<&MdlStateProjectionProvenanceV1>,
) -> Result<DirectCreatureStructuralSummaryV1, MdlRuntimeConformanceErrorV1> {
    require_state_projection_provenance_v1(profile, provenance, expected_provenance)?;
    if report.model.geometry_type != 2 || report.model.classification != 4 {
        return Err(conformance_error(
            "M2A-MDL-CONFORMANCE-DIRECT-CREATURE-HEADER",
            "model",
            "state projection profiles require a direct Character geometry",
        ));
    }
    let base = flatten_tree(&report.node_tree);
    if report.node_tree.roots.len() != 1 || base.is_empty() {
        return Err(conformance_error(
            "M2A-MDL-CONFORMANCE-BASE-ROOT",
            "nodeTree.roots",
            "direct creature must have exactly one non-empty base tree",
        ));
    }
    if profile == MdlStateProjectionProfileV1::CepRigidPlaceholderV1
        && base.iter().any(|node| node.has_skin)
    {
        return Err(conformance_error(
            "M2A-MDL-CONFORMANCE-CEP-SKIN-UNPROVEN",
            "nodeTree",
            "CEP_RIGID_PLACEHOLDER_V1 is not evidenced for skin base nodes",
        ));
    }
    if matches!(
        profile,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
            | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1
    ) && !base.iter().any(|node| node.has_mesh || node.has_skin)
    {
        return Err(conformance_error(
            "M2A-MDL-CONFORMANCE-RIG-ONLY-BASE-MESH-MISSING",
            "nodeTree",
            "rig-only state projection requires at least one renderable base mesh/skin leaf to omit",
        ));
    }

    let projected_base = base
        .iter()
        .filter(|node| {
            !matches!(
                profile,
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
                    | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1
            ) || (!node.has_mesh && !node.has_skin)
        })
        .collect::<Vec<_>>();
    let base_identity = projected_base
        .iter()
        .map(|node| identity_only_one(node))
        .collect::<Vec<_>>();
    for (animation_index, animation) in report.animations.iter().enumerate() {
        let animation_path = format!("animations[{animation_index}]");
        let expected_animation_type =
            if profile == MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 {
                0
            } else {
                5
            };
        if animation.animation_type != expected_animation_type {
            return Err(conformance_error(
                "M2A-MDL-CONFORMANCE-ANIMATION-TYPE",
                format!("{animation_path}.animationType"),
                "animation type differs from the exact selected runtime family",
            ));
        }
        let state = flatten_tree(&animation.node_tree);
        if state.len() != projected_base.len() {
            if matches!(
                profile,
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
                    | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1
            ) {
                return Err(conformance_error(
                    "M2A-MDL-CONFORMANCE-RIG-ONLY-STATE-NODE-COUNT",
                    format!("{animation_path}.nodeTree"),
                    "rig-only state tree must project every non-renderable base rig identity exactly once and omit every mesh/skin leaf",
                ));
            }
            return Err(conformance_error(
                "M2A-MDL-CONFORMANCE-STATE-NODE-MISSING",
                format!("{animation_path}.nodeTree"),
                "state tree must project every base name/part/parent identity exactly once",
            ));
        }
        for (node_index, (state_node, base_node)) in state.iter().zip(&projected_base).enumerate() {
            let node_path = format!("{animation_path}.nodeTree.nodes[{node_index}]");
            if identity_only_one(state_node) != base_identity[node_index] {
                return Err(conformance_error(
                    "M2A-MDL-CONFORMANCE-STATE-IDENTITY",
                    node_path,
                    "state name, part number, or parent identity differs from the base tree",
                ));
            }
            match profile {
                MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
                | MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
                | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 => {
                    require_generic_dummy(state_node, &node_path)?;
                }
                MdlStateProjectionProfileV1::CepRigidPlaceholderV1 => {
                    if base_node.content_flags & 0x20 != 0 {
                        if state_node.content_flags != 0x21 {
                            return Err(conformance_error(
                                "M2A-MDL-CONFORMANCE-PROFILE-MIXED",
                                format!("{node_path}.contentFlags"),
                                "CEP rigid mesh identity requires a 0x21 placeholder",
                            ));
                        }
                        if !state_node.zero_geometry_mesh || state_node.has_skin {
                            return Err(conformance_error(
                                "M2A-MDL-CONFORMANCE-CEP-PLACEHOLDER-PAYLOAD",
                                node_path,
                                "CEP placeholder must have an empty mesh header and no skin/raw payload",
                            ));
                        }
                    } else {
                        require_generic_dummy(state_node, &node_path)?;
                    }
                }
            }
        }
    }
    Ok(summarize_direct_creature_structure_v1(report))
}

fn require_state_projection_provenance_v1(
    profile: MdlStateProjectionProfileV1,
    provenance: Option<&MdlStateProjectionProvenanceV1>,
    expected_provenance: Option<&MdlStateProjectionProvenanceV1>,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    match profile {
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
        | MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
        | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 => {
            if provenance.is_some() || expected_provenance.is_some() {
                return Err(conformance_error(
                    "M2A-MDL-CONFORMANCE-PROFILE-MIXED",
                    "stateProjectionProvenance",
                    "retail state-projection families must not carry CEP placeholder provenance",
                ));
            }
        }
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1 => {
            let provenance = provenance.ok_or_else(|| {
                conformance_error(
                    "M2A-MDL-CONFORMANCE-PROVENANCE-MISSING",
                    "stateProjectionProvenance",
                    "CEP_RIGID_PLACEHOLDER_V1 requires explicit provenance",
                )
            })?;
            let expected = expected_provenance.ok_or_else(|| {
                conformance_error(
                    "M2A-MDL-CONFORMANCE-EXPECTED-PROVENANCE-MISSING",
                    "expectedStateProjectionProvenance",
                    "CEP conformance requires an independently supplied expected provenance binding",
                )
            })?;
            if !is_well_formed_state_projection_provenance_v1(provenance)
                || !is_well_formed_state_projection_provenance_v1(expected)
                || provenance != expected
            {
                return Err(conformance_error(
                    "M2A-MDL-CONFORMANCE-PROVENANCE-MISMATCH",
                    "stateProjectionProvenance",
                    "CEP placeholder provenance differs from the independent expected binding",
                ));
            }
        }
    }
    Ok(())
}

fn require_generic_dummy(
    node: &NodeIdentity,
    path: &str,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    if node.content_flags != 0x01 {
        return Err(conformance_error(
            "M2A-MDL-CONFORMANCE-PROFILE-MIXED",
            format!("{path}.contentFlags"),
            "retail projection requires a generic 0x01 dummy for every base identity",
        ));
    }
    if node.has_mesh || node.has_skin {
        return Err(conformance_error(
            "M2A-MDL-CONFORMANCE-RETAIL-DUMMY-PAYLOAD",
            path,
            "retail state dummy must not contain mesh, skin, or raw-MDX payload",
        ));
    }
    Ok(())
}

fn is_generic_dummy(node: &NodeIdentity) -> bool {
    node.content_flags == 0x01 && !node.has_mesh && !node.has_skin
}

fn flatten_tree(tree: &NodeTreeReport) -> Vec<NodeIdentity> {
    let mut output = Vec::with_capacity(tree.node_count);
    for root in &tree.roots {
        flatten_node(root, None, &mut output);
    }
    output
}

fn flatten_node(node: &NodeReport, parent_number: Option<u32>, output: &mut Vec<NodeIdentity>) {
    output.push(NodeIdentity {
        number: node.number,
        name: node.name.clone(),
        parent_number,
        content_flags: node.content_flags,
        has_mesh: node.mesh.is_some(),
        has_skin: node.skin.is_some(),
        zero_geometry_mesh: node.mesh.as_ref().is_some_and(is_zero_geometry_mesh),
        child_count: node.children.len(),
        controller_count: node.controllers.len(),
    });
    for child in &node.children {
        flatten_node(child, Some(node.number), output);
    }
}

fn identity_only(nodes: &[NodeIdentity]) -> Vec<(u32, String, Option<u32>)> {
    nodes.iter().map(identity_only_one).collect()
}

fn identity_only_one(node: &NodeIdentity) -> (u32, String, Option<u32>) {
    (node.number, node.name.clone(), node.parent_number)
}

fn is_zero_geometry_mesh(mesh: &MeshReport) -> bool {
    mesh.vertex_count == 0
        && mesh.vertices.is_empty()
        && mesh.faces.is_empty()
        && mesh.index_counts.is_empty()
        && mesh.raw_index_offsets.is_empty()
        && mesh.raw_indices.is_empty()
        && mesh.textures.iter().all(String::is_empty)
        && mesh.start_mdx == 0
}

/// Parses exact binary MDL bytes through the tolerant general reader and then
/// projects only renderer-facing schema facts and hashes. It never invokes the
/// strict M0 envelope verifier, opens a file, identifies a witness, or supplies
/// a runtime verdict.
pub fn inspect_direct_creature_envelope_projection_v1(
    mdl: &[u8],
) -> Result<(DirectCreatureEnvelopeProjectionV1, String), MdlRuntimeConformanceErrorV1> {
    let report = super::inspect_binary_mdl(mdl).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-PARSE",
            format!("model@{}", error.offset),
            &error.context,
        )
    })?;
    let model = DirectCreatureModelProjectionV1 {
        name: report.model.name.clone(),
        geometry_type: report.model.geometry_type,
        classification: report.model.classification,
        fog: report.model.fog,
        child_model_count: report.model.child_model_count,
        bounds_min: vec3_array(report.model.bounds_min),
        bounds_max: vec3_array(report.model.bounds_max),
        radius: report.model.radius,
        animation_scale: report.model.animation_scale,
        supermodel_name: report.model.supermodel_name.clone(),
        animation_count: u32_len(report.animations.len(), "projection.model.animationCount")?,
        routine_words: [
            projection_core_u32(mdl, 0x00, "projection.model.routineWords[0]")?,
            projection_core_u32(mdl, 0x04, "projection.model.routineWords[1]")?,
        ],
        geometry_array_50: projection_core_u32x3(mdl, 0x50, "projection.model.geometryArray50")?,
        geometry_array_5c: projection_core_u32x3(mdl, 0x5c, "projection.model.geometryArray5c")?,
        geometry_ref_count: projection_core_u32(mdl, 0x68, "projection.model.geometryRefCount")?,
        geometry_type_padding: [
            projection_core_u8(mdl, 0x6d, "projection.model.geometryTypePadding[0]")?,
            projection_core_u8(mdl, 0x6e, "projection.model.geometryTypePadding[1]")?,
            projection_core_u8(mdl, 0x6f, "projection.model.geometryTypePadding[2]")?,
        ],
        runtime_unknown_bytes: [
            projection_core_u8(mdl, 0x70, "projection.model.runtimeUnknownBytes[0]")?,
            projection_core_u8(mdl, 0x71, "projection.model.runtimeUnknownBytes[1]")?,
        ],
        supermodel_pointer: projection_core_u32(mdl, 0x84, "projection.model.supermodelPointer")?,
    };
    let mut base_nodes = Vec::new();
    let mut meshes = Vec::new();
    for root in &report.node_tree.roots {
        project_tolerant_node(mdl, root, None, &mut base_nodes, &mut meshes)?;
    }
    let mut animations = Vec::with_capacity(report.animations.len());
    for (order, animation) in report.animations.iter().enumerate() {
        let mut nodes = Vec::new();
        let mut ignored_meshes = Vec::new();
        for root in &animation.node_tree.roots {
            project_tolerant_node(mdl, root, None, &mut nodes, &mut ignored_meshes)?;
        }
        let topology = nodes
            .iter()
            .map(|node| {
                (
                    node.order,
                    node.part,
                    &node.name,
                    node.parent_part,
                    node.content_flags,
                    &node.kind,
                )
            })
            .collect::<Vec<_>>();
        let controllers = nodes
            .iter()
            .map(|node| (&node.name, &node.controller_signature_sha256))
            .collect::<Vec<_>>();
        animations.push(DirectCreatureAnimationProjectionV1 {
            order: u32_len(order, "projection.animations.order")?,
            name: animation.name.clone(),
            animation_type: animation.animation_type,
            animation_type_padding: animation.animation_type_padding,
            animation_root: animation.animation_root.clone(),
            runtime_68: animation.runtime_68,
            node_count: u32_len(
                animation.node_tree.node_count,
                "projection.animations.nodeCount",
            )?,
            max_depth: u32_len(
                animation.node_tree.max_depth,
                "projection.animations.maxDepth",
            )?,
            topology_sha256: projection_hash(&topology, "projection.animations.topology")?,
            controller_signature_sha256: projection_hash(
                &controllers,
                "projection.animations.controllers",
            )?,
        });
    }
    let mut field_assessments = vec![
        projection_assessment(
            "reader.diagnostics",
            report.diagnostics.is_empty(),
            "PARSER_DIAGNOSTICS_PRESENT",
        ),
        projection_assessment(
            "reader.unsupported",
            report.unsupported.is_empty(),
            "UNSUPPORTED_FAMILIES_PRESENT",
        ),
        DirectCreatureEnvelopeFieldAssessmentV1 {
            path: "meshes".to_owned(),
            state: if meshes.is_empty() {
                DirectCreatureEnvelopeFieldStateV1::Unavailable
            } else {
                DirectCreatureEnvelopeFieldStateV1::Available
            },
            reason_code: if meshes.is_empty() {
                "NO_PARSED_MESH".to_owned()
            } else {
                "NONE".to_owned()
            },
        },
    ];
    for node in &base_nodes {
        if node.kind == "UNSUPPORTED" {
            field_assessments.push(DirectCreatureEnvelopeFieldAssessmentV1 {
                path: format!("baseNodes[{}]", node.order),
                state: DirectCreatureEnvelopeFieldStateV1::Unassessed,
                reason_code: "UNSUPPORTED_NODE_FAMILY".to_owned(),
            });
        }
    }
    field_assessments.sort_by(|left, right| left.path.cmp(&right.path));
    let projection = DirectCreatureEnvelopeProjectionV1 {
        schema_version: 1,
        profile: "DIRECT_CREATURE_ENVELOPE_PROJECTION_V1".to_owned(),
        binary_mdl_byte_length: mdl.len() as u64,
        binary_mdl_sha256: sha256_bytes(mdl),
        model,
        base_nodes,
        meshes,
        animations,
        field_assessments,
    };
    let digest = direct_creature_envelope_projection_digest_v1(&projection)?;
    Ok((projection, digest))
}

fn projection_assessment(
    path: &str,
    available: bool,
    unavailable_reason: &str,
) -> DirectCreatureEnvelopeFieldAssessmentV1 {
    DirectCreatureEnvelopeFieldAssessmentV1 {
        path: path.to_owned(),
        state: if available {
            DirectCreatureEnvelopeFieldStateV1::Available
        } else {
            DirectCreatureEnvelopeFieldStateV1::Unassessed
        },
        reason_code: if available {
            "NONE"
        } else {
            unavailable_reason
        }
        .to_owned(),
    }
}

fn project_tolerant_node(
    mdl: &[u8],
    node: &NodeReport,
    parent_part: Option<u32>,
    nodes: &mut Vec<DirectCreatureNodeProjectionV1>,
    meshes: &mut Vec<DirectCreatureMeshProjectionV1>,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    let order = u32_len(nodes.len(), "projection.nodes.order")?;
    let base = usize::try_from(node.offset).map_err(|_| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-RANGE",
            format!("projection.nodes[{order}].offset"),
            "node offset does not fit this platform",
        )
    })?;
    let kind = match (
        node.mesh.is_some(),
        node.skin.is_some(),
        node.unsupported_families.is_empty(),
    ) {
        (_, _, false) => "UNSUPPORTED",
        (false, false, true) => "DUMMY",
        (true, false, true) => "MESH",
        (true, true, true) => "SKIN_MESH",
        (false, true, true) => "SKIN_WITHOUT_MESH",
    };
    let controller_signature = node
        .controllers
        .iter()
        .map(|controller| {
            (
                controller.controller_type,
                controller.packed_byte,
                controller.interpolation_flags,
                controller.padding_byte,
                controller.row_count,
                controller.time_index,
                controller.data_index,
                controller.column_count,
            )
        })
        .collect::<Vec<_>>();
    nodes.push(DirectCreatureNodeProjectionV1 {
        order,
        part: node.number,
        name: node.name.clone(),
        parent_part,
        content_flags: node.content_flags,
        inherit_color: node.inherit_color,
        kind: kind.to_owned(),
        routine_words: [
            projection_core_u32(mdl, base, "projection.nodes.routineWords[0]")?,
            projection_core_u32(mdl, base + 4, "projection.nodes.routineWords[1]")?,
            projection_core_u32(mdl, base + 8, "projection.nodes.routineWords[2]")?,
            projection_core_u32(mdl, base + 12, "projection.nodes.routineWords[3]")?,
            projection_core_u32(mdl, base + 16, "projection.nodes.routineWords[4]")?,
            projection_core_u32(mdl, base + 20, "projection.nodes.routineWords[5]")?,
        ],
        geometry_pointer: projection_core_u32(
            mdl,
            base + 0x40,
            "projection.nodes.geometryPointer",
        )?,
        parent_pointer: projection_core_u32(mdl, base + 0x44, "projection.nodes.parentPointer")?,
        children_used: u32_len(node.children_header.used, "projection.nodes.childrenUsed")?,
        children_allocated: u32_len(
            node.children_header.allocated,
            "projection.nodes.childrenAllocated",
        )?,
        controller_signature_sha256: projection_hash(
            &controller_signature,
            "projection.nodes.controllers",
        )?,
    });
    if let Some(mesh) = node.mesh.as_ref() {
        let face_surface_ids = mesh
            .faces
            .iter()
            .map(|face| face.surface_id)
            .collect::<Vec<_>>();
        let face_adjacency = mesh
            .faces
            .iter()
            .map(|face| face.adjacent_faces)
            .collect::<Vec<_>>();
        let face_indices = mesh
            .faces
            .iter()
            .map(|face| face.vertex_indices)
            .collect::<Vec<_>>();
        let geometry = (
            &mesh.vertices,
            &mesh.normals,
            &mesh.uv0,
            &mesh.vertex_colors,
            &face_indices,
            &mesh.raw_indices,
        );
        let raw_pointer_signature = mesh
            .validated_raw_pointers
            .iter()
            .map(|pointer| (&pointer.field, pointer.pointer, pointer.validated_length))
            .collect::<Vec<_>>();
        meshes.push(DirectCreatureMeshProjectionV1 {
            node_part: node.number,
            node_name: node.name.clone(),
            parent_part,
            routine_words: [
                projection_core_u32(mdl, base + 0x70, "projection.meshes.routineWords[0]")?,
                projection_core_u32(mdl, base + 0x74, "projection.meshes.routineWords[1]")?,
            ],
            vertex_count: u32_len(mesh.vertex_count, "projection.meshes.vertexCount")?,
            face_count: u32_len(mesh.faces.len(), "projection.meshes.faceCount")?,
            texture_count: u32_len(mesh.texture_count, "projection.meshes.textureCount")?,
            textures: mesh.textures.clone(),
            vertex_colors_present: !mesh.vertex_colors.is_empty(),
            vertex_color_count: u32_len(
                mesh.vertex_colors.len(),
                "projection.meshes.vertexColorCount",
            )?,
            diffuse: mesh.diffuse,
            ambient: mesh.ambient,
            specular: mesh.specular,
            shininess: mesh.shininess,
            shadow: mesh.shadow,
            beaming: mesh.beaming,
            render: mesh.render,
            transparency: mesh.transparency,
            render_hint: mesh.render_hint,
            tile_fade: mesh.tile_fade,
            mesh_type: mesh.mesh_type,
            start_mdx: mesh.start_mdx,
            light_mapped: projection_core_u8(mdl, base + 0x264, "projection.meshes.lightMapped")?,
            rotate_texture: projection_core_u8(
                mdl,
                base + 0x265,
                "projection.meshes.rotateTexture",
            )?,
            tail_padding: projection_core_u16(mdl, base + 0x266, "projection.meshes.tailPadding")?,
            vertex_normal_sum_bits: projection_core_u32(
                mdl,
                base + 0x268,
                "projection.meshes.vertexNormalSumBits",
            )?,
            tail_unknown: projection_core_u32(mdl, base + 0x26c, "projection.meshes.tailUnknown")?,
            face_surface_ids_sha256: projection_hash(
                &face_surface_ids,
                "projection.meshes.faceSurfaceIds",
            )?,
            face_adjacency_sha256: projection_hash(
                &face_adjacency,
                "projection.meshes.faceAdjacency",
            )?,
            geometry_sha256: projection_hash(&geometry, "projection.meshes.geometry")?,
            raw_pointer_signature_sha256: projection_hash(
                &raw_pointer_signature,
                "projection.meshes.rawPointers",
            )?,
        });
    }
    for child in &node.children {
        project_tolerant_node(mdl, child, Some(node.number), nodes, meshes)?;
    }
    Ok(())
}

fn projection_hash<T: Serialize>(
    value: &T,
    path: &str,
) -> Result<String, MdlRuntimeConformanceErrorV1> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-SERIALIZATION",
            path,
            &format!("payload-free projection field cannot be serialized: {error}"),
        )
    })?;
    Ok(sha256_bytes(&bytes))
}

fn projection_core_u8(
    mdl: &[u8],
    core_offset: usize,
    path: &str,
) -> Result<u8, MdlRuntimeConformanceErrorV1> {
    projection_core_slice(mdl, core_offset, 1, path).map(|bytes| bytes[0])
}

fn projection_core_u16(
    mdl: &[u8],
    core_offset: usize,
    path: &str,
) -> Result<u16, MdlRuntimeConformanceErrorV1> {
    let bytes = projection_core_slice(mdl, core_offset, 2, path)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn projection_core_u32(
    mdl: &[u8],
    core_offset: usize,
    path: &str,
) -> Result<u32, MdlRuntimeConformanceErrorV1> {
    let bytes = projection_core_slice(mdl, core_offset, 4, path)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn projection_core_u32x3(
    mdl: &[u8],
    core_offset: usize,
    path: &str,
) -> Result<[u32; 3], MdlRuntimeConformanceErrorV1> {
    Ok([
        projection_core_u32(mdl, core_offset, path)?,
        projection_core_u32(mdl, core_offset + 4, path)?,
        projection_core_u32(mdl, core_offset + 8, path)?,
    ])
}

fn projection_core_slice<'a>(
    mdl: &'a [u8],
    core_offset: usize,
    length: usize,
    path: &str,
) -> Result<&'a [u8], MdlRuntimeConformanceErrorV1> {
    let start = 12usize.checked_add(core_offset).ok_or_else(|| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-RANGE",
            path,
            "core offset overflow",
        )
    })?;
    let end = start.checked_add(length).ok_or_else(|| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-RANGE",
            path,
            "renderer field end overflow",
        )
    })?;
    mdl.get(start..end).ok_or_else(|| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-RANGE",
            path,
            "renderer field escapes exact MDL bytes",
        )
    })
}

/// Re-inspects exact owned MDL bytes and emits the versioned engine envelope.
/// General MDL parsing remains tolerant; this profile-bound admission is
/// fail-closed on diagnostics, trailing bytes, incomplete raw ranges, aliases,
/// or any field the envelope cannot assess.
pub fn inspect_direct_creature_engine_envelope_v1(
    mdl: &[u8],
) -> Result<(DirectCreatureEngineEnvelopeV1, String), MdlRuntimeConformanceErrorV1> {
    let report = super::inspect_binary_mdl(mdl).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-PARSE",
            format!("model@{}", error.offset),
            &error.context,
        )
    })?;
    if report.file_header.raw_range.end != mdl.len()
        || report.file_header.core_range.end != report.file_header.raw_range.start
        || !report.file_header.mdx_range_in_bounds
    {
        return Err(conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-RANGE-INCOMPLETE",
            "fileHeader",
            "core and raw MDX ranges must be contiguous, in bounds, and consume exact EOF",
        ));
    }
    if !report.diagnostics.is_empty() || !report.unsupported.is_empty() {
        return Err(conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-UNASSESSED",
            "diagnostics",
            "profile-bound engine admission rejects parser diagnostics and unsupported families",
        ));
    }

    let core = engine_range(
        mdl,
        report.file_header.core_range.start,
        report.file_header.core_range.length,
    )?;
    let raw_mdx = engine_range(
        mdl,
        report.file_header.raw_range.start,
        report.file_header.raw_range.length,
    )?;
    let raw_mdx_bytes = mdl
        .get(report.file_header.raw_range.start..report.file_header.raw_range.end)
        .ok_or_else(|| {
            conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                "fileHeader.rawRange",
                "exact raw MDX bytes are unavailable after validated range readback",
            )
        })?;
    let mut base_nodes = Vec::new();
    let mut meshes = Vec::new();
    flatten_engine_nodes(
        mdl,
        report.file_header.raw_range.start,
        &report.node_tree,
        &mut base_nodes,
        &mut meshes,
    )?;
    validate_raw_marker_semantics(&meshes)?;
    validate_raw_stream_partition(&meshes, raw_mdx_bytes)?;
    let base_identity = base_nodes
        .iter()
        .map(|node| (node.part, node.name.clone(), node.parent_part))
        .collect::<Vec<_>>();
    let animations = report
        .animations
        .iter()
        .enumerate()
        .map(|(order, animation)| {
            let mut nodes = Vec::new();
            flatten_engine_node_tree_without_mesh_payload(&animation.node_tree, &mut nodes)?;
            let identity = nodes
                .iter()
                .map(|node| (node.part, node.name.clone(), node.parent_part))
                .collect::<Vec<_>>();
            Ok(EngineAnimationEnvelopeV1 {
                order: u32_len(order, "animations.order")?,
                name: animation.name.clone(),
                animation_type: animation.animation_type,
                animation_type_padding: animation.animation_type_padding,
                animation_root: animation.animation_root.clone(),
                runtime_68: animation.runtime_68,
                geometry_array_50_used: u32_len(
                    animation.geometry_array_50.used,
                    "animations.geometryArray50.used",
                )?,
                geometry_array_50_allocated: u32_len(
                    animation.geometry_array_50.allocated,
                    "animations.geometryArray50.allocated",
                )?,
                geometry_array_5c_used: u32_len(
                    animation.geometry_array_5c.used,
                    "animations.geometryArray5c.used",
                )?,
                geometry_array_5c_allocated: u32_len(
                    animation.geometry_array_5c.allocated,
                    "animations.geometryArray5c.allocated",
                )?,
                event_count: u32_len(animation.events.len(), "animations.events")?,
                node_count: u32_len(animation.node_tree.node_count, "animations.nodeCount")?,
                max_depth: u32_len(animation.node_tree.max_depth, "animations.maxDepth")?,
                topology_matches_base: identity == base_identity,
                nodes,
            })
        })
        .collect::<Result<Vec<_>, MdlRuntimeConformanceErrorV1>>()?;

    let envelope = DirectCreatureEngineEnvelopeV1 {
        schema_version: 1,
        profile: "DIRECT_CREATURE_ENGINE_ENVELOPE_V1".to_owned(),
        verdict: DirectCreatureEngineEnvelopeVerdictV1::StructuralPassRuntimeNotWitnessed,
        binary_mdl_byte_length: mdl.len() as u64,
        binary_mdl_sha256: sha256_bytes(mdl),
        binary_mdl_id: report.file_header.binary_mdl_id,
        core,
        raw_mdx,
        geometry_type: report.model.geometry_type,
        classification: report.model.classification,
        fog: report.model.fog,
        supermodel_name: report.model.supermodel_name.clone(),
        child_model_count: report.model.child_model_count,
        bounds_min: vec3_array(report.model.bounds_min),
        bounds_max: vec3_array(report.model.bounds_max),
        radius: report.model.radius,
        animation_scale: report.model.animation_scale,
        animation_pointer_used: u32_len(
            report.model.animation_pointers_header.used,
            "model.animationPointers.used",
        )?,
        animation_pointer_allocated: u32_len(
            report.model.animation_pointers_header.allocated,
            "model.animationPointers.allocated",
        )?,
        base_node_count: u32_len(report.node_tree.node_count, "nodeTree.nodeCount")?,
        base_max_depth: u32_len(report.node_tree.max_depth, "nodeTree.maxDepth")?,
        base_nodes,
        meshes,
        animations,
        parser_diagnostics: Vec::new(),
        unsupported: Vec::new(),
        unassessed_fields: Vec::new(),
    };
    if envelope.geometry_type != 2
        || envelope.classification != 4
        || envelope.base_nodes.is_empty()
        || envelope.meshes.is_empty()
        || !envelope.unassessed_fields.is_empty()
    {
        return Err(conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-STRUCTURAL-REJECTED",
            "engineEnvelope",
            "direct-creature envelope requires an assessed Character model with a base tree and mesh",
        ));
    }
    let digest = direct_creature_engine_envelope_digest_v1(&envelope)?;
    Ok((envelope, digest))
}

pub fn direct_creature_engine_envelope_digest_v1(
    envelope: &DirectCreatureEngineEnvelopeV1,
) -> Result<String, MdlRuntimeConformanceErrorV1> {
    if envelope.schema_version != 1
        || envelope.profile != "DIRECT_CREATURE_ENGINE_ENVELOPE_V1"
        || !envelope.unassessed_fields.is_empty()
    {
        return Err(conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-VERSION",
            "engineEnvelope",
            "canonical envelope digest requires the exact assessed V1 profile",
        ));
    }
    let bytes = serde_json::to_vec(envelope).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-SERIALIZATION",
            "engineEnvelope",
            &format!("engine envelope cannot be canonically serialized: {error}"),
        )
    })?;
    Ok(sha256_bytes(&bytes))
}

pub fn direct_creature_envelope_projection_digest_v1(
    projection: &DirectCreatureEnvelopeProjectionV1,
) -> Result<String, MdlRuntimeConformanceErrorV1> {
    if projection.schema_version != 1
        || projection.profile != "DIRECT_CREATURE_ENVELOPE_PROJECTION_V1"
    {
        return Err(conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-VERSION",
            "projection",
            "canonical projection digest requires the exact tolerant V1 profile",
        ));
    }
    let bytes = serde_json::to_vec(projection).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENVELOPE-PROJECTION-SERIALIZATION",
            "projection",
            &format!("renderer projection cannot be canonically serialized: {error}"),
        )
    })?;
    Ok(sha256_bytes(&bytes))
}

/// Compares two caller-supplied, payload-free tolerant projections. This is a
/// diagnostic classifier only: it neither invokes nor substitutes strict M0
/// admission and it carries no runtime-visibility verdict.
pub fn direct_creature_envelope_differential_v1(
    left: &DirectCreatureEnvelopeProjectionV1,
    right: &DirectCreatureEnvelopeProjectionV1,
) -> Result<(DirectCreatureEnvelopeDifferentialV1, String), MdlRuntimeConformanceErrorV1> {
    let left_projection_sha256 = direct_creature_envelope_projection_digest_v1(left)?;
    let right_projection_sha256 = direct_creature_envelope_projection_digest_v1(right)?;
    let left_json = serde_json::to_value(left).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENVELOPE-DIFFERENTIAL-SERIALIZATION",
            "left",
            &format!("left projection cannot be compared: {error}"),
        )
    })?;
    let right_json = serde_json::to_value(right).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENVELOPE-DIFFERENTIAL-SERIALIZATION",
            "right",
            &format!("right projection cannot be compared: {error}"),
        )
    })?;
    let mut differences = Vec::new();
    collect_projection_differences("$", Some(&left_json), Some(&right_json), &mut differences);
    differences.sort_by(|left, right| left.path.cmp(&right.path));
    let mut classification_counts = BTreeMap::new();
    for difference in &differences {
        *classification_counts
            .entry(difference.classification)
            .or_insert(0) += 1;
    }
    let report = DirectCreatureEnvelopeDifferentialV1 {
        schema_version: 1,
        profile: "DIRECT_CREATURE_ENVELOPE_DIFFERENTIAL_V1".to_owned(),
        left_projection_sha256,
        right_projection_sha256,
        identical: differences.is_empty(),
        classification_counts,
        differences,
    };
    let bytes = serde_json::to_vec(&report).map_err(|error| {
        conformance_error(
            "M2A-MDL-ENVELOPE-DIFFERENTIAL-SERIALIZATION",
            "differential",
            &format!("differential cannot be canonically serialized: {error}"),
        )
    })?;
    Ok((report, sha256_bytes(&bytes)))
}

fn collect_projection_differences(
    path: &str,
    left: Option<&serde_json::Value>,
    right: Option<&serde_json::Value>,
    output: &mut Vec<DirectCreatureEnvelopeFieldDifferenceV1>,
) {
    match (left, right) {
        (Some(serde_json::Value::Object(left)), Some(serde_json::Value::Object(right))) => {
            let keys = left
                .keys()
                .chain(right.keys())
                .collect::<std::collections::BTreeSet<_>>();
            for key in keys {
                collect_projection_differences(
                    &format!("{path}.{key}"),
                    left.get(key),
                    right.get(key),
                    output,
                );
            }
        }
        (Some(serde_json::Value::Array(left)), Some(serde_json::Value::Array(right))) => {
            for index in 0..left.len().max(right.len()) {
                collect_projection_differences(
                    &format!("{path}[{index}]"),
                    left.get(index),
                    right.get(index),
                    output,
                );
            }
        }
        (left, right) if left == right => {}
        (left, right) => output.push(DirectCreatureEnvelopeFieldDifferenceV1 {
            path: path.to_owned(),
            classification: classify_projection_difference(path),
            left: left.cloned(),
            right: right.cloned(),
        }),
    }
}

fn classify_projection_difference(path: &str) -> DirectCreatureEnvelopeDifferenceClassV1 {
    if path.starts_with("$.fieldAssessments") {
        return DirectCreatureEnvelopeDifferenceClassV1::FieldAvailability;
    }
    if path == "$.binaryMdlByteLength" || path == "$.binaryMdlSha256" {
        return DirectCreatureEnvelopeDifferenceClassV1::ArtifactIdentity;
    }
    if path.starts_with("$.model.bounds") || path == "$.model.radius" {
        return DirectCreatureEnvelopeDifferenceClassV1::ModelBounds;
    }
    if path.starts_with("$.model.routineWords")
        || path.starts_with("$.model.geometryArray")
        || path == "$.model.geometryRefCount"
        || path.starts_with("$.model.geometryTypePadding")
        || path.starts_with("$.model.runtimeUnknownBytes")
        || path == "$.model.supermodelPointer"
        || path == "$.model.fog"
    {
        return DirectCreatureEnvelopeDifferenceClassV1::ModelRuntimeDefaults;
    }
    if path.starts_with("$.model") {
        return DirectCreatureEnvelopeDifferenceClassV1::ModelHeader;
    }
    if path.starts_with("$.baseNodes") {
        if path.contains(".routineWords")
            || path.ends_with(".geometryPointer")
            || path.ends_with(".parentPointer")
            || path.ends_with(".inheritColor")
            || path.ends_with(".controllerSignatureSha256")
        {
            return DirectCreatureEnvelopeDifferenceClassV1::NodeRuntimeDefaults;
        }
        return DirectCreatureEnvelopeDifferenceClassV1::BaseTopology;
    }
    if path.starts_with("$.meshes") {
        if is_direct_array_element_path(path, "$.meshes") {
            return DirectCreatureEnvelopeDifferenceClassV1::GeometryShape;
        }
        if path.contains("vertexColor") {
            return DirectCreatureEnvelopeDifferenceClassV1::VertexColorPresence;
        }
        if path.contains("faceSurface") || path.contains("faceAdjacency") {
            return DirectCreatureEnvelopeDifferenceClassV1::FaceRuntimeDefaults;
        }
        if path.contains("geometrySha256")
            || path.ends_with("vertexCount")
            || path.ends_with("faceCount")
        {
            return DirectCreatureEnvelopeDifferenceClassV1::GeometryShape;
        }
        if path.contains("rawPointer") || path.ends_with("startMdx") {
            return DirectCreatureEnvelopeDifferenceClassV1::RawMdxLayout;
        }
        if path.contains("textures") || path.ends_with("textureCount") {
            return DirectCreatureEnvelopeDifferenceClassV1::MeshMaterial;
        }
        if path.ends_with("nodePart") || path.ends_with("nodeName") || path.ends_with("parentPart")
        {
            return DirectCreatureEnvelopeDifferenceClassV1::MeshAttachment;
        }
        return DirectCreatureEnvelopeDifferenceClassV1::MeshRuntimeDefaults;
    }
    if path.starts_with("$.animations") {
        if is_direct_array_element_path(path, "$.animations") {
            return DirectCreatureEnvelopeDifferenceClassV1::StateTopology;
        }
        if path.ends_with("topologySha256")
            || path.ends_with("nodeCount")
            || path.ends_with("maxDepth")
        {
            return DirectCreatureEnvelopeDifferenceClassV1::StateTopology;
        }
        return DirectCreatureEnvelopeDifferenceClassV1::AnimationHeader;
    }
    DirectCreatureEnvelopeDifferenceClassV1::ArtifactIdentity
}

fn is_direct_array_element_path(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix)
        .and_then(|suffix| suffix.strip_prefix('['))
        .and_then(|suffix| suffix.strip_suffix(']'))
        .is_some_and(|index| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()))
}

fn flatten_engine_nodes(
    mdl: &[u8],
    raw_start: usize,
    tree: &NodeTreeReport,
    nodes: &mut Vec<EngineNodeEnvelopeV1>,
    meshes: &mut Vec<EngineMeshEnvelopeV1>,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    for root in &tree.roots {
        flatten_engine_node(mdl, raw_start, root, None, nodes, meshes)?;
    }
    Ok(())
}

fn flatten_engine_node(
    mdl: &[u8],
    raw_start: usize,
    node: &NodeReport,
    parent_part: Option<u32>,
    nodes: &mut Vec<EngineNodeEnvelopeV1>,
    meshes: &mut Vec<EngineMeshEnvelopeV1>,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    let order = u32_len(nodes.len(), "nodeTree.order")?;
    nodes.push(engine_node_envelope(node, parent_part, order)?);
    if let Some(mesh) = node.mesh.as_ref() {
        meshes.push(engine_mesh_envelope(
            mdl,
            raw_start,
            node,
            parent_part,
            mesh,
        )?);
    }
    for child in &node.children {
        flatten_engine_node(mdl, raw_start, child, Some(node.number), nodes, meshes)?;
    }
    Ok(())
}

fn flatten_engine_node_tree_without_mesh_payload(
    tree: &NodeTreeReport,
    nodes: &mut Vec<EngineNodeEnvelopeV1>,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    fn walk(
        node: &NodeReport,
        parent_part: Option<u32>,
        output: &mut Vec<EngineNodeEnvelopeV1>,
    ) -> Result<(), MdlRuntimeConformanceErrorV1> {
        let order = u32_len(output.len(), "animations.nodeTree.order")?;
        output.push(engine_node_envelope(node, parent_part, order)?);
        for child in &node.children {
            walk(child, Some(node.number), output)?;
        }
        Ok(())
    }
    for root in &tree.roots {
        walk(root, None, nodes)?;
    }
    Ok(())
}

fn engine_node_envelope(
    node: &NodeReport,
    parent_part: Option<u32>,
    order: u32,
) -> Result<EngineNodeEnvelopeV1, MdlRuntimeConformanceErrorV1> {
    let kind = match (node.mesh.is_some(), node.skin.is_some()) {
        (false, false) => "DUMMY",
        (true, false) => "MESH",
        (true, true) => "SKIN_MESH",
        (false, true) => "SKIN_WITHOUT_MESH",
    };
    let controllers = node
        .controllers
        .iter()
        .map(|controller| {
            let payload =
                serde_json::to_vec(&(&controller.times, &controller.values, controller.key_offset))
                    .map_err(|error| {
                        conformance_error(
                            "M2A-MDL-ENGINE-ENVELOPE-SERIALIZATION",
                            "controllers",
                            &format!("controller payload cannot be hashed: {error}"),
                        )
                    })?;
            Ok(EngineControllerEnvelopeV1 {
                controller_type: controller.controller_type,
                controller_name: controller.controller_name.clone(),
                packed_byte: controller.packed_byte,
                interpolation_flags: controller.interpolation_flags,
                decoded: controller.decoded,
                padding_byte: controller.padding_byte,
                row_count: u32_len(controller.row_count, "controllers.rowCount")?,
                time_index: u32_len(controller.time_index, "controllers.timeIndex")?,
                data_index: u32_len(controller.data_index, "controllers.dataIndex")?,
                column_count: u32_len(controller.column_count, "controllers.columnCount")?,
                payload_sha256: sha256_bytes(&payload),
            })
        })
        .collect::<Result<Vec<_>, MdlRuntimeConformanceErrorV1>>()?;
    Ok(EngineNodeEnvelopeV1 {
        order,
        part: node.number,
        name: node.name.clone(),
        parent_part,
        content_flags: node.content_flags,
        inherit_color: node.inherit_color,
        kind: kind.to_owned(),
        children_used: u32_len(node.children_header.used, "nodes.children.used")?,
        children_allocated: u32_len(node.children_header.allocated, "nodes.children.allocated")?,
        controller_keys_used: u32_len(
            node.controller_keys_header.used,
            "nodes.controllerKeys.used",
        )?,
        controller_keys_allocated: u32_len(
            node.controller_keys_header.allocated,
            "nodes.controllerKeys.allocated",
        )?,
        controller_data_used: u32_len(
            node.controller_data_header.used,
            "nodes.controllerData.used",
        )?,
        controller_data_allocated: u32_len(
            node.controller_data_header.allocated,
            "nodes.controllerData.allocated",
        )?,
        controllers,
    })
}

fn engine_mesh_envelope(
    mdl: &[u8],
    raw_start: usize,
    node: &NodeReport,
    parent_part: Option<u32>,
    mesh: &MeshReport,
) -> Result<EngineMeshEnvelopeV1, MdlRuntimeConformanceErrorV1> {
    let mut raw_streams = mesh
        .validated_raw_pointers
        .iter()
        .map(|pointer| {
            let sha256 = match pointer.pointer {
                Some(value) if pointer.validated_length > 0 => {
                    let relative = usize::try_from(value).map_err(|_| {
                        conformance_error(
                            "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                            format!("mesh.{}", pointer.field),
                            "negative raw stream pointer cannot be assessed",
                        )
                    })?;
                    let start = raw_start.checked_add(relative).ok_or_else(|| {
                        conformance_error(
                            "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                            format!("mesh.{}", pointer.field),
                            "raw stream absolute offset overflow",
                        )
                    })?;
                    let end = start.checked_add(pointer.validated_length).ok_or_else(|| {
                        conformance_error(
                            "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                            format!("mesh.{}", pointer.field),
                            "raw stream end overflow",
                        )
                    })?;
                    let bytes = mdl.get(start..end).ok_or_else(|| {
                        conformance_error(
                            "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                            format!("mesh.{}", pointer.field),
                            "raw stream escapes exact MDX range",
                        )
                    })?;
                    Some(sha256_bytes(bytes))
                }
                _ => None,
            };
            Ok(EngineRawStreamEnvelopeV1 {
                field: pointer.field.clone(),
                pointer: pointer.pointer,
                byte_length: pointer.validated_length as u64,
                stride: raw_stride(&pointer.field),
                sha256,
            })
        })
        .collect::<Result<Vec<_>, MdlRuntimeConformanceErrorV1>>()?;
    for (index, (&offset, &count)) in mesh
        .raw_index_offsets
        .iter()
        .zip(&mesh.index_counts)
        .enumerate()
    {
        let length = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(2))
            .ok_or_else(|| {
                conformance_error(
                    "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                    format!("mesh.rawIndices[{index}]"),
                    "raw index stream length overflow",
                )
            })?;
        let relative = usize::try_from(offset).map_err(|_| {
            conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                format!("mesh.rawIndices[{index}]"),
                "negative raw index pointer cannot be assessed",
            )
        })?;
        let start = raw_start.checked_add(relative).ok_or_else(|| {
            conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                format!("mesh.rawIndices[{index}]"),
                "raw index absolute offset overflow",
            )
        })?;
        let end = start.checked_add(length).ok_or_else(|| {
            conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                format!("mesh.rawIndices[{index}]"),
                "raw index end overflow",
            )
        })?;
        let bytes = mdl.get(start..end).ok_or_else(|| {
            conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                format!("mesh.rawIndices[{index}]"),
                "raw index stream escapes exact MDX range",
            )
        })?;
        raw_streams.push(EngineRawStreamEnvelopeV1 {
            field: format!("rawIndices[{index}]"),
            pointer: Some(offset),
            byte_length: length as u64,
            stride: Some(2),
            sha256: Some(sha256_bytes(bytes)),
        });
    }
    let face_indices = mesh
        .faces
        .iter()
        .flat_map(|face| face.vertex_indices)
        .collect::<Vec<_>>();
    let raw_flat = mesh
        .raw_indices
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let geometry_payload = serde_json::to_vec(&(
        &mesh.vertices,
        &mesh.normals,
        &mesh.uv0,
        &mesh.vertex_colors,
        &face_indices,
        &mesh.raw_indices,
    ))
    .map_err(|error| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-SERIALIZATION",
            "mesh.geometry",
            &format!("geometry mapping cannot be hashed: {error}"),
        )
    })?;
    Ok(EngineMeshEnvelopeV1 {
        node_part: node.number,
        node_name: node.name.clone(),
        parent_part,
        textures: mesh.textures.clone(),
        texture_count: u32_len(mesh.texture_count, "mesh.textureCount")?,
        vertex_count: u32_len(mesh.vertex_count, "mesh.vertexCount")?,
        vertex_colors_present: !mesh.vertex_colors.is_empty(),
        vertex_color_count: u32_len(mesh.vertex_colors.len(), "mesh.vertexColorCount")?,
        bounds_min: vec3_array(mesh.bounds_min),
        bounds_max: vec3_array(mesh.bounds_max),
        radius: mesh.radius,
        average: mesh.average.map(vec3_array),
        diffuse: mesh.diffuse,
        ambient: mesh.ambient,
        specular: mesh.specular,
        shininess: mesh.shininess,
        shadow: mesh.shadow,
        beaming: mesh.beaming,
        render: mesh.render,
        transparency: mesh.transparency,
        render_hint: mesh.render_hint,
        tile_fade: mesh.tile_fade,
        mesh_type: mesh.mesh_type,
        start_mdx: mesh.start_mdx,
        face_count: u32_len(mesh.faces.len(), "mesh.faceCount")?,
        face_index_count: u32_len(face_indices.len(), "mesh.faceIndexCount")?,
        index_counts: mesh.index_counts.clone(),
        raw_index_offsets: mesh.raw_index_offsets.clone(),
        max_vertex_index: face_indices.iter().copied().max(),
        face_indices_match_raw_indices: face_indices == raw_flat,
        geometry_sha256: sha256_bytes(&geometry_payload),
        raw_streams,
    })
}

fn validate_raw_stream_partition(
    meshes: &[EngineMeshEnvelopeV1],
    raw_mdx: &[u8],
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    let mut ranges = meshes
        .iter()
        .flat_map(|mesh| &mesh.raw_streams)
        // `startMdx` is the explicitly assessed zero-origin marker and is
        // therefore allowed to alias the first real stream. `unknown0` is
        // required to be the absent -1 sentinel by
        // `validate_raw_marker_semantics` and contributes no range.
        .filter(|stream| stream.field != "startMdx" && stream.byte_length > 0)
        .map(|stream| {
            let start = stream
                .pointer
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| {
                    conformance_error(
                        "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                        format!("mesh.{}", stream.field),
                        "raw stream has no non-negative pointer",
                    )
                })?;
            let length = usize::try_from(stream.byte_length).map_err(|_| {
                conformance_error(
                    "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                    format!("mesh.{}", stream.field),
                    "raw stream length does not fit this platform",
                )
            })?;
            let end = start.checked_add(length).ok_or_else(|| {
                conformance_error(
                    "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
                    format!("mesh.{}", stream.field),
                    "raw stream range overflow",
                )
            })?;
            Ok((stream.field.clone(), start, end))
        })
        .collect::<Result<Vec<_>, MdlRuntimeConformanceErrorV1>>()?;
    ranges.sort_by_key(|(_, start, _)| *start);
    let mut assessed_end = 0usize;
    for (field, start, end) in &ranges {
        if *start < assessed_end || *end > raw_mdx.len() {
            return Err(conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-ALIAS",
                format!("mesh.{field}"),
                "raw MDX streams must be non-overlapping and remain within the exact MDX range",
            ));
        }
        assessed_end = *end;
    }
    let mut previous_end = 0usize;
    for (field, start, end) in ranges {
        if start > previous_end {
            let message = format!(
                "raw MDX stream starts at {start}, leaving unassessed bytes after {previous_end}"
            );
            return Err(conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-GAP",
                format!("mesh.{field}"),
                &message,
            ));
        }
        previous_end = end;
    }
    let tail = raw_mdx.get(previous_end..).ok_or_else(|| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-RAW-RANGE",
            "rawMdx.tail",
            "assessed stream end escapes exact raw MDX bytes",
        )
    })?;
    if tail.len() > 3 {
        let message = format!(
            "canonical raw MDX alignment tail is at most 3 bytes, got {}",
            tail.len()
        );
        return Err(conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-RAW-TAIL-LENGTH",
            "rawMdx.tail",
            &message,
        ));
    }
    if tail.iter().any(|byte| *byte != 0) {
        return Err(conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-RAW-TAIL-NONZERO",
            "rawMdx.tail",
            "canonical raw MDX alignment tail must contain only zero bytes",
        ));
    }
    Ok(())
}

fn validate_raw_marker_semantics(
    meshes: &[EngineMeshEnvelopeV1],
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    for (index, mesh) in meshes.iter().enumerate() {
        let unknown = mesh
            .raw_streams
            .iter()
            .filter(|stream| stream.field == "unknown0")
            .collect::<Vec<_>>();
        if unknown.len() != 1
            || unknown[0].pointer.is_some()
            || unknown[0].byte_length != 0
            || unknown[0].sha256.is_some()
        {
            return Err(conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-MARKER",
                format!("meshes[{index}].rawStreams.unknown0"),
                "the direct-creature profile requires the unknown0 raw slot to be the absent -1 sentinel",
            ));
        }

        let start = mesh
            .raw_streams
            .iter()
            .filter(|stream| stream.field == "startMdx")
            .collect::<Vec<_>>();
        let expected_length = u64::from(mesh.vertex_count > 0);
        let expected_pointer = (mesh.vertex_count > 0).then_some(0);
        if start.len() != 1
            || start[0].pointer != expected_pointer
            || start[0].byte_length != expected_length
            || mesh.start_mdx != 0
        {
            return Err(conformance_error(
                "M2A-MDL-ENGINE-ENVELOPE-RAW-MARKER",
                format!("meshes[{index}].rawStreams.startMdx"),
                "the direct-creature profile requires startMdx=0 as the assessed raw-origin marker for a non-empty mesh",
            ));
        }
    }
    Ok(())
}

fn engine_range(
    bytes: &[u8],
    start: usize,
    length: usize,
) -> Result<EngineByteRangeV1, MdlRuntimeConformanceErrorV1> {
    let end = start.checked_add(length).ok_or_else(|| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-RANGE-INCOMPLETE",
            "fileHeader",
            "binary range end overflow",
        )
    })?;
    let slice = bytes.get(start..end).ok_or_else(|| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-RANGE-INCOMPLETE",
            "fileHeader",
            "binary range escapes exact MDL bytes",
        )
    })?;
    Ok(EngineByteRangeV1 {
        start: start as u64,
        length: length as u64,
        end: end as u64,
        sha256: sha256_bytes(slice),
    })
}

fn raw_stride(field: &str) -> Option<u32> {
    match field {
        "vertices" | "normals" | "textureAnimation0" | "textureAnimation1"
        | "textureAnimation2" | "textureAnimation3" | "textureAnimation4" => Some(12),
        "uv0" | "uv1" | "uv2" | "uv3" => Some(8),
        "colors" | "textureAnimation5" => Some(4),
        _ => None,
    }
}

fn vec3_array(value: super::types::Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
}

fn u32_len(value: usize, path: &str) -> Result<u32, MdlRuntimeConformanceErrorV1> {
    u32::try_from(value).map_err(|_| {
        conformance_error(
            "M2A-MDL-ENGINE-ENVELOPE-COUNT-OVERFLOW",
            path,
            "structural count exceeds u32",
        )
    })
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn conformance_error(
    code: &str,
    path: impl Into<String>,
    message: &str,
) -> MdlRuntimeConformanceErrorV1 {
    MdlRuntimeConformanceErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.into(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod differential_tests {
    use super::*;

    #[test]
    fn differential_v1_is_versioned_deterministic_and_classifies_renderer_fields() {
        let left = minimal_differential_projection_v1();
        let mut right = left.clone();
        right.model.fog = 0;
        right.base_nodes[0].inherit_color = 1;
        right.meshes[0].render = 0;
        right.meshes[0].vertex_colors_present = true;
        right.meshes[0].vertex_color_count = 3;
        right.meshes[0].face_surface_ids_sha256 = "1".repeat(64);

        let (first, first_sha) = direct_creature_envelope_differential_v1(&left, &right)
            .expect("pure projection differential");
        let (second, second_sha) =
            direct_creature_envelope_differential_v1(&left, &right).expect("deterministic repeat");
        assert_eq!(first, second);
        assert_eq!(first_sha, second_sha);
        assert_eq!(first.schema_version, 1);
        assert_eq!(first.profile, "DIRECT_CREATURE_ENVELOPE_DIFFERENTIAL_V1");
        assert_eq!(
            first
                .differences
                .iter()
                .map(|difference| (difference.path.as_str(), difference.classification))
                .collect::<Vec<_>>(),
            vec![
                (
                    "$.baseNodes[0].inheritColor",
                    DirectCreatureEnvelopeDifferenceClassV1::NodeRuntimeDefaults
                ),
                (
                    "$.meshes[0].faceSurfaceIdsSha256",
                    DirectCreatureEnvelopeDifferenceClassV1::FaceRuntimeDefaults
                ),
                (
                    "$.meshes[0].render",
                    DirectCreatureEnvelopeDifferenceClassV1::MeshRuntimeDefaults
                ),
                (
                    "$.meshes[0].vertexColorCount",
                    DirectCreatureEnvelopeDifferenceClassV1::VertexColorPresence
                ),
                (
                    "$.meshes[0].vertexColorsPresent",
                    DirectCreatureEnvelopeDifferenceClassV1::VertexColorPresence
                ),
                (
                    "$.model.fog",
                    DirectCreatureEnvelopeDifferenceClassV1::ModelRuntimeDefaults
                ),
            ]
        );
    }

    #[test]
    fn differential_v1_keeps_unavailable_and_unassessed_explicit() {
        let left = minimal_differential_projection_v1();
        let mut right = left.clone();
        right.field_assessments[0].state = DirectCreatureEnvelopeFieldStateV1::Unassessed;
        right.field_assessments[0].reason_code = "PARSER_DIAGNOSTIC".to_owned();

        let (report, _) = direct_creature_envelope_differential_v1(&left, &right)
            .expect("assessment differential");
        assert!(report.differences.iter().all(|difference| {
            difference.classification == DirectCreatureEnvelopeDifferenceClassV1::FieldAvailability
        }));
        assert!(
            report
                .differences
                .iter()
                .any(|difference| difference.path == "$.fieldAssessments[0].state")
        );
    }

    fn minimal_differential_projection_v1() -> DirectCreatureEnvelopeProjectionV1 {
        DirectCreatureEnvelopeProjectionV1 {
            schema_version: 1,
            profile: "DIRECT_CREATURE_ENVELOPE_PROJECTION_V1".to_owned(),
            binary_mdl_byte_length: 1,
            binary_mdl_sha256: "0".repeat(64),
            model: DirectCreatureModelProjectionV1 {
                name: "owned_fixture".to_owned(),
                geometry_type: 2,
                classification: 4,
                fog: 1,
                child_model_count: 0,
                bounds_min: [-1.0, -1.0, -1.0],
                bounds_max: [1.0, 1.0, 1.0],
                radius: 1.0,
                animation_scale: 1.0,
                supermodel_name: "NULL".to_owned(),
                animation_count: 1,
                routine_words: [0, 0],
                geometry_array_50: [0, 0, 0],
                geometry_array_5c: [0, 0, 0],
                geometry_ref_count: 0,
                geometry_type_padding: [0, 0, 0],
                runtime_unknown_bytes: [0, 0],
                supermodel_pointer: 0,
            },
            base_nodes: vec![DirectCreatureNodeProjectionV1 {
                order: 0,
                part: 0,
                name: "owned_fixture".to_owned(),
                parent_part: None,
                content_flags: 1,
                inherit_color: 0,
                kind: "DUMMY".to_owned(),
                routine_words: [0; 6],
                geometry_pointer: 0,
                parent_pointer: 0,
                children_used: 1,
                children_allocated: 1,
                controller_signature_sha256: "0".repeat(64),
            }],
            meshes: vec![DirectCreatureMeshProjectionV1 {
                node_part: 1,
                node_name: "owned_mesh".to_owned(),
                parent_part: Some(0),
                routine_words: [0, 0],
                vertex_count: 3,
                face_count: 1,
                texture_count: 1,
                textures: vec!["owned_texture".to_owned()],
                vertex_colors_present: false,
                vertex_color_count: 0,
                diffuse: [1.0, 1.0, 1.0],
                ambient: [1.0, 1.0, 1.0],
                specular: [0.0, 0.0, 0.0],
                shininess: 1.0,
                shadow: 1,
                beaming: 0,
                render: 1,
                transparency: 0,
                render_hint: 0,
                tile_fade: 0,
                mesh_type: 3,
                start_mdx: 0,
                light_mapped: 0,
                rotate_texture: 0,
                tail_padding: 0,
                vertex_normal_sum_bits: 0,
                tail_unknown: 0,
                face_surface_ids_sha256: "0".repeat(64),
                face_adjacency_sha256: "0".repeat(64),
                geometry_sha256: "0".repeat(64),
                raw_pointer_signature_sha256: "0".repeat(64),
            }],
            animations: vec![DirectCreatureAnimationProjectionV1 {
                order: 0,
                name: "owned_idle".to_owned(),
                animation_type: 5,
                animation_type_padding: [0, 0, 0],
                animation_root: "owned_fixture".to_owned(),
                runtime_68: 0,
                node_count: 2,
                max_depth: 1,
                topology_sha256: "0".repeat(64),
                controller_signature_sha256: "0".repeat(64),
            }],
            field_assessments: vec![DirectCreatureEnvelopeFieldAssessmentV1 {
                path: "reader.diagnostics".to_owned(),
                state: DirectCreatureEnvelopeFieldStateV1::Available,
                reason_code: "NONE".to_owned(),
            }],
        }
    }
}
