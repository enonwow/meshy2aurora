//! Offline-only controlled experiment for a source-topology-preserving rigid
//! direct-creature wrapper.
//!
//! This module is intentionally not wired into MOD/HAK/package/proof writers.
//! It requires an explicit caller-owned V3 profile and cannot produce runtime
//! admission. Historical M0/V2 remains the flat, one-source-node contract.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    direct_creature_contract::{
        SourceMeshAttachmentV1, SourceToOutputNodeV1, SourceTopologyOriginV1,
    },
    glb::{GlbIngestResult, GlbLimits, IrNode, ingest_glb},
    mdl::{
        BinaryMdlArtifactV1, DirectCreatureEngineEnvelopeV1, DirectCreatureEnvelopeProjectionV1,
        MdlAnimationClipV1, MdlAnimationSetV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlStateProjectionProfileV1, MdlWriterOptionsV1,
        direct_creature_structural_summary_digest_v1, inspect_direct_creature_engine_envelope_v1,
        inspect_direct_creature_envelope_projection_v1, verify_direct_creature_state_projection_v1,
        write_binary_mdl_with_animations,
    },
    profile_a::{
        CreatureRigNodeV1, CreatureRigProfileV1, ProfileAOptionsV1, canonical_profile_sha256,
        convert_profile_a, derive_meshy_m0_static_rigid_profile_v1,
    },
};

pub const SOURCE_TOPOLOGY_RIGID_EXPERIMENT_PROFILE_V3: &str =
    "SOURCE_TOPOLOGY_PRESERVING_UNSKINNED_RIGID_DIRECT_CREATURE_V3";
pub const SOURCE_TOPOLOGY_RIGID_EXPERIMENT_CONTRACT_V3: &str =
    "SOURCE_TOPOLOGY_RIGID_OFFLINE_EXPERIMENT_CONTRACT_V3";
pub const MESHY_HIERARCHY_CANDIDATE_PROFILE_V4: &str =
    "MESHY_SOURCE_DERIVED_HIERARCHY_DIRECT_CREATURE_CANDIDATE_V4";
pub const MESHY_HIERARCHY_CANDIDATE_CONTRACT_V4: &str =
    "MESHY_SOURCE_DERIVED_HIERARCHY_MODEL_CONTRACT_V4";
pub const MESHY_HIERARCHY_DERIVATION_V1: &str = "MESHY_M0_NEUTRAL_IDENTITY_HIERARCHY_DERIVATION_V1";
const SOURCE_TOPOLOGY_RIGID_CONVERSION_V1: &str = "SOURCE_TOPOLOGY_PRESERVING_UNSKINNED_RIGID_V1";
const SOURCE_TOPOLOGY_RIGID_WRITER_V1: &str = "SOURCE_TOPOLOGY_PRESERVING_RIGID_EXPERIMENT_V1";
const NULLABLE_NAME_RULE_V1: &str = "ROOT_RESREF_ELSE_SOURCE_NAME_ELSE_M2A_SRC_ORDER_V1";
const TEXTURE_RESREF: &str = "m2a_hexpt01";
const M0_TEXTURE_RESREF: &str = "m2a_m0t01";
const FROZEN_M0_SOURCE_BYTE_LENGTH: u64 = 8_581_684;
const FROZEN_M0_SOURCE_SHA256: &str =
    "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1";
const FROZEN_R30_MODEL_BYTE_LENGTH: u64 = 150_024;
const FROZEN_R30_MODEL_SHA256: &str =
    "43a5cbfa1a20146ec0990ce7ee980d70a54d7f72689781a58c7d9614bc35b8c7";
const DERIVED_NODE_NAMES_V1: [&str; 3] = ["m2a_m0p01", "m2a_hier_1", "m2a_mesh_anchor"];
const REQUIRED_CLIPS: [&str; 7] = [
    "cwalk",
    "crun",
    "cdamage",
    "cconjure1",
    "castout1",
    "cclosew",
    "cpause1",
];
const M0_RUNTIME_CLIPS: [&str; 7] = [
    "cappear",
    "cpause1",
    "cwalk",
    "crun",
    "ca1slashl",
    "cdamagel",
    "cdead",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceTopologyRigidExperimentProfileV3 {
    pub schema_version: u32,
    pub profile: String,
    pub source_informational_path: String,
    pub source_canonical_identity: String,
    pub source_byte_length: u64,
    pub source_sha256: String,
    pub source_origin: SourceTopologyOriginV1,
    pub conversion_profile: String,
    pub conversion_profile_version: u32,
    pub writer_profile: String,
    pub writer_profile_version: u32,
    pub state_projection_profile: String,
    pub nullable_name_rule: String,
    pub model_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceTopologyRigidSourceNodeV3 {
    pub source_order: u32,
    pub node_id: u32,
    pub name: Option<String>,
    pub parent_node_id: Option<u32>,
    pub ordered_child_ids: Vec<u32>,
    pub mesh_id: Option<u32>,
    pub transform_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceTopologyRigidSourceSummaryV3 {
    pub schema_version: u32,
    pub profile: String,
    pub default_scene_id: u32,
    pub ordered_root_node_ids: Vec<u32>,
    pub ordered_traversal_node_ids: Vec<u32>,
    pub nodes: Vec<SourceTopologyRigidSourceNodeV3>,
    pub selected_node_id: u32,
    pub selected_mesh_id: u32,
    pub selected_primitive_id: u32,
    pub source_mesh_count: u32,
    pub source_primitive_count: u32,
    pub source_skin_count: u32,
    pub source_animation_count: u32,
    pub ignored_node_count: u32,
    pub ignored_mesh_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceTopologyRigidExperimentContractV3 {
    pub schema_version: u32,
    pub profile: String,
    pub runtime_admissible: bool,
    pub structural_verdict: String,
    pub experiment_profile_sha256: String,
    pub source_byte_length: u64,
    pub source_sha256: String,
    pub source_geometry_sha256: String,
    pub source_summary: SourceTopologyRigidSourceSummaryV3,
    pub source_summary_sha256: String,
    pub source_to_output_nodes: Vec<SourceToOutputNodeV1>,
    pub mesh_attachment: SourceMeshAttachmentV1,
    pub output_topology_sha256: String,
    pub hierarchy_mdl_byte_length: u64,
    pub hierarchy_mdl_sha256: String,
    pub hierarchy_engine_envelope_sha256: String,
    pub hierarchy_state_projection_sha256: String,
    pub hierarchy_raw_mdx_sha256: String,
    pub flat_control_mdl_sha256: String,
    pub flat_control_raw_mdx_sha256: String,
    pub hierarchy_protected_writer_fields_sha256: String,
    pub flat_control_protected_writer_fields_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceTopologyRigidExperimentArtifactV3 {
    pub hierarchy_mdl: BinaryMdlArtifactV1,
    pub flat_control_mdl: BinaryMdlArtifactV1,
    pub contract: SourceTopologyRigidExperimentContractV3,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeshyHierarchyDerivedSourceBindingV4 {
    pub schema_version: u32,
    pub profile: String,
    pub derivation_version: u32,
    pub original_informational_path: String,
    pub derived_informational_path: String,
    pub origin: SourceTopologyOriginV1,
    pub original_source_byte_length: u64,
    pub original_source_sha256: String,
    pub derived_source_byte_length: u64,
    pub derived_source_sha256: String,
    pub original_bin_byte_length: u64,
    pub original_bin_sha256: String,
    pub derived_bin_byte_length: u64,
    pub derived_bin_sha256: String,
    pub original_geometry_sha256: String,
    pub derived_geometry_sha256: String,
    pub topology_summary: SourceTopologyRigidSourceSummaryV3,
    pub topology_summary_sha256: String,
    pub node_identity_policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshyHierarchyDerivedSourceArtifactV4 {
    pub bytes: Vec<u8>,
    pub binding: MeshyHierarchyDerivedSourceBindingV4,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeshyHierarchyCandidateProfileV4 {
    pub schema_version: u32,
    pub profile: String,
    pub source_origin: SourceTopologyOriginV1,
    pub original_informational_path: String,
    pub derived_informational_path: String,
    pub derivation_profile: String,
    pub derivation_profile_version: u32,
    pub derived_source_binding_sha256: String,
    pub original_source_byte_length: u64,
    pub original_source_sha256: String,
    pub derived_source_byte_length: u64,
    pub derived_source_sha256: String,
    pub frozen_r30_model_byte_length: u64,
    pub frozen_r30_model_sha256: String,
    pub writer_profile: String,
    pub writer_profile_version: u32,
    pub state_projection_profile: String,
    pub model_resref: String,
    pub texture_resref: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeshyHierarchyCandidateModelContractV4 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub structural_verdict: String,
    pub runtime_model_visibility: String,
    pub runtime_proof_completeness: String,
    pub candidate_profile_sha256: String,
    pub derived_source_binding: MeshyHierarchyDerivedSourceBindingV4,
    pub derived_source_binding_sha256: String,
    pub source_to_output_nodes: Vec<SourceToOutputNodeV1>,
    pub mesh_attachment: SourceMeshAttachmentV1,
    pub output_topology_sha256: String,
    pub candidate_model_byte_length: u64,
    pub candidate_model_sha256: String,
    pub candidate_engine_envelope: DirectCreatureEngineEnvelopeV1,
    pub candidate_engine_envelope_sha256: String,
    pub candidate_state_projection_sha256: String,
    pub candidate_raw_mdx_sha256: String,
    pub candidate_protected_writer_fields_sha256: String,
    pub r30_model_byte_length: u64,
    pub r30_model_sha256: String,
    pub r30_raw_mdx_sha256: String,
    pub r30_protected_writer_fields_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshyHierarchyCandidateModelArtifactV4 {
    pub model: BinaryMdlArtifactV1,
    pub contract: MeshyHierarchyCandidateModelContractV4,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTopologyRigidExperimentErrorV3 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for SourceTopologyRigidExperimentErrorV3 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for SourceTopologyRigidExperimentErrorV3 {}

struct SourceShape<'a> {
    summary: SourceTopologyRigidSourceSummaryV3,
    ordered_nodes: Vec<&'a IrNode>,
    selected_node_id: u32,
    selected_mesh_id: u32,
    selected_primitive_id: u32,
}

struct OutputInspection {
    envelope: DirectCreatureEngineEnvelopeV1,
    envelope_sha256: String,
    state_projection_sha256: String,
    output_topology_sha256: String,
    source_to_output_nodes: Vec<SourceToOutputNodeV1>,
    mesh_attachment: SourceMeshAttachmentV1,
}

pub fn declare_source_topology_rigid_experiment_profile_v3(
    source_glb: &[u8],
    source_informational_path: impl Into<String>,
    source_origin: SourceTopologyOriginV1,
    model_resref: impl Into<String>,
) -> Result<SourceTopologyRigidExperimentProfileV3, SourceTopologyRigidExperimentErrorV3> {
    let source_informational_path = source_informational_path.into();
    let model_resref = model_resref.into();
    if source_informational_path.trim().is_empty() || !valid_resref(&model_resref) {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-PROFILE",
            "profile",
            "informational source path and lower-case model resref must be explicit",
        ));
    }
    Ok(SourceTopologyRigidExperimentProfileV3 {
        schema_version: 3,
        profile: SOURCE_TOPOLOGY_RIGID_EXPERIMENT_PROFILE_V3.to_owned(),
        source_informational_path,
        source_canonical_identity: format!("sha256:{}", sha256_bytes(source_glb)),
        source_byte_length: source_glb.len() as u64,
        source_sha256: sha256_bytes(source_glb),
        source_origin,
        conversion_profile: SOURCE_TOPOLOGY_RIGID_CONVERSION_V1.to_owned(),
        conversion_profile_version: 1,
        writer_profile: SOURCE_TOPOLOGY_RIGID_WRITER_V1.to_owned(),
        writer_profile_version: 1,
        state_projection_profile: "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1".to_owned(),
        nullable_name_rule: NULLABLE_NAME_RULE_V1.to_owned(),
        model_resref,
    })
}

pub fn build_source_topology_rigid_experiment_v3(
    source_glb: &[u8],
    profile: &SourceTopologyRigidExperimentProfileV3,
) -> Result<SourceTopologyRigidExperimentArtifactV3, SourceTopologyRigidExperimentErrorV3> {
    require_profile_source(profile, source_glb)?;
    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        experiment_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let shape = inspect_source_shape(&ingest)?;
    let mut flat_rig = derive_meshy_m0_static_rigid_profile_v1(&ingest)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    flat_rig.profile_id = "source-topology-rigid-flat-control-v1".to_owned();
    flat_rig.content_sha256 = canonical_profile_sha256(&flat_rig)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let flat_control_mdl = emit_experiment_model(&ingest, &flat_rig, profile)?;

    let mut hierarchy_rig = flat_rig.clone();
    hierarchy_rig.profile_id = SOURCE_TOPOLOGY_RIGID_CONVERSION_V1.to_owned();
    hierarchy_rig.nodes = shape
        .ordered_nodes
        .iter()
        .enumerate()
        .map(|(order, node)| CreatureRigNodeV1 {
            id: node.id,
            name: output_name(node, order, profile),
            parent_id: node.parent_ids.first().copied(),
            bind_local_matrix: identity_matrix(),
        })
        .collect();
    hierarchy_rig.segments[0].parent_node_id = shape.selected_node_id;
    hierarchy_rig.content_sha256 = canonical_profile_sha256(&hierarchy_rig)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let hierarchy_mdl = emit_experiment_model(&ingest, &hierarchy_rig, profile)?;

    let hierarchy_inspection = inspect_experiment_output(&hierarchy_mdl.payload, &shape, profile)?;
    let (flat_envelope, _) = inspect_direct_creature_engine_envelope_v1(&flat_control_mdl.payload)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let (hierarchy_projection, _) =
        inspect_direct_creature_envelope_projection_v1(&hierarchy_mdl.payload)
            .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let (flat_projection, _) =
        inspect_direct_creature_envelope_projection_v1(&flat_control_mdl.payload)
            .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let hierarchy_protected = protected_writer_fields_digest(
        &hierarchy_projection,
        &hierarchy_inspection.envelope.raw_mdx.sha256,
    )?;
    let flat_protected =
        protected_writer_fields_digest(&flat_projection, &flat_envelope.raw_mdx.sha256)?;
    if hierarchy_inspection.envelope.raw_mdx.sha256 != flat_envelope.raw_mdx.sha256 {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-RAW-MDX-DRIFT",
            "hierarchyMdl.rawMdx",
            "the topology-only experiment changed exact raw MDX bytes",
        ));
    }
    if hierarchy_protected != flat_protected {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-PROTECTED-FIELD-DRIFT",
            "hierarchyMdl",
            "the hierarchy delta changed a protected non-topology writer field",
        ));
    }
    let source_summary_sha256 = canonical_digest(&shape.summary, "sourceSummary")?;
    let source_geometry_sha256 = source_geometry_digest(&ingest, &shape)?;
    let experiment_profile_sha256 = canonical_digest(profile, "experimentProfile")?;
    let contract = SourceTopologyRigidExperimentContractV3 {
        schema_version: 3,
        profile: SOURCE_TOPOLOGY_RIGID_EXPERIMENT_CONTRACT_V3.to_owned(),
        runtime_admissible: false,
        structural_verdict: "OFFLINE_STRUCTURAL_EXPERIMENT_ONLY".to_owned(),
        experiment_profile_sha256,
        source_byte_length: source_glb.len() as u64,
        source_sha256: sha256_bytes(source_glb),
        source_geometry_sha256,
        source_summary: shape.summary,
        source_summary_sha256,
        source_to_output_nodes: hierarchy_inspection.source_to_output_nodes,
        mesh_attachment: hierarchy_inspection.mesh_attachment,
        output_topology_sha256: hierarchy_inspection.output_topology_sha256,
        hierarchy_mdl_byte_length: hierarchy_mdl.payload.len() as u64,
        hierarchy_mdl_sha256: sha256_bytes(&hierarchy_mdl.payload),
        hierarchy_engine_envelope_sha256: hierarchy_inspection.envelope_sha256,
        hierarchy_state_projection_sha256: hierarchy_inspection.state_projection_sha256,
        hierarchy_raw_mdx_sha256: hierarchy_inspection.envelope.raw_mdx.sha256,
        flat_control_mdl_sha256: sha256_bytes(&flat_control_mdl.payload),
        flat_control_raw_mdx_sha256: flat_envelope.raw_mdx.sha256,
        hierarchy_protected_writer_fields_sha256: hierarchy_protected,
        flat_control_protected_writer_fields_sha256: flat_protected,
    };
    Ok(SourceTopologyRigidExperimentArtifactV3 {
        hierarchy_mdl,
        flat_control_mdl,
        contract,
    })
}

pub fn verify_source_topology_rigid_experiment_v3(
    contract: &SourceTopologyRigidExperimentContractV3,
    expected_profile: &SourceTopologyRigidExperimentProfileV3,
    source_glb: &[u8],
    candidate_mdl: &[u8],
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    require_contract_version(contract)?;
    require_profile_source(expected_profile, source_glb)?;
    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        experiment_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let shape = inspect_source_shape(&ingest)?;
    inspect_experiment_output(candidate_mdl, &shape, expected_profile)?;
    let replay = build_source_topology_rigid_experiment_v3(source_glb, expected_profile)?;
    if candidate_mdl != replay.hierarchy_mdl.payload || contract != &replay.contract {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-REPLAY-MISMATCH",
            "contract",
            "candidate bytes or contract differ from independent exact-source deterministic replay",
        ));
    }
    Ok(())
}

fn inspect_source_shape(
    ingest: &GlbIngestResult,
) -> Result<SourceShape<'_>, SourceTopologyRigidExperimentErrorV3> {
    let default_scene_id = ingest.ir.default_scene_id.ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-SCENE",
            "source.defaultSceneId",
            "one explicit default scene is required",
        )
    })?;
    let scene = ingest
        .ir
        .scenes
        .iter()
        .find(|scene| scene.id == default_scene_id)
        .ok_or_else(|| {
            experiment_error(
                "M2A-HIERARCHY-EXPERIMENT-SCENE",
                "source.defaultSceneId",
                "default scene does not resolve",
            )
        })?;
    if ingest.ir.scenes.len() != 1 || scene.root_node_ids.len() != 1 {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-SCENE",
            "source.scenes",
            "the controlled profile requires exactly one scene and one root",
        ));
    }
    let mut ordered_nodes = Vec::new();
    let mut visiting = BTreeSet::new();
    fn visit<'a>(
        id: u32,
        ingest: &'a GlbIngestResult,
        visiting: &mut BTreeSet<u32>,
        output: &mut Vec<&'a IrNode>,
    ) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
        if !visiting.insert(id) {
            return Err(experiment_error(
                "M2A-HIERARCHY-EXPERIMENT-CYCLE",
                "source.nodes",
                "source child order contains a cycle",
            ));
        }
        let node = ingest
            .ir
            .nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or_else(|| {
                experiment_error(
                    "M2A-HIERARCHY-EXPERIMENT-NODE-MISSING",
                    "source.nodes",
                    "scene traversal references a missing node",
                )
            })?;
        output.push(node);
        for child in &node.child_ids {
            visit(*child, ingest, visiting, output)?;
        }
        visiting.remove(&id);
        Ok(())
    }
    visit(
        scene.root_node_ids[0],
        ingest,
        &mut visiting,
        &mut ordered_nodes,
    )?;
    let source_order = ingest
        .ir
        .nodes
        .iter()
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let traversal = ordered_nodes.iter().map(|node| node.id).collect::<Vec<_>>();
    if ordered_nodes.len() != ingest.ir.nodes.len() || source_order != traversal {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-ORDER",
            "source.nodes",
            "every source node must be reachable exactly once in source/traversal order",
        ));
    }
    if ordered_nodes.len() < 3 {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-DEPTH",
            "source.nodes",
            "the controlled witness requires at least two nested transform dummies plus the mesh transform",
        ));
    }
    let mut folded_names = BTreeSet::new();
    for (order, node) in ordered_nodes.iter().enumerate() {
        if node.parent_ids.len() > 1 || !is_identity_transform(node) {
            return Err(experiment_error(
                "M2A-HIERARCHY-EXPERIMENT-TRANSFORM",
                format!("source.nodes[{order}]"),
                "V3 isolates hierarchy with single-parent identity transforms only",
            ));
        }
        if let Some(name) = &node.name
            && (!valid_node_name(name) || !folded_names.insert(name.to_ascii_lowercase()))
        {
            return Err(experiment_error(
                "M2A-HIERARCHY-EXPERIMENT-NAME",
                format!("source.nodes[{order}].name"),
                "non-null source names must be valid and unique after ASCII case-fold",
            ));
        }
        for child in &node.child_ids {
            let child_node = ingest
                .ir
                .nodes
                .iter()
                .find(|candidate| candidate.id == *child)
                .ok_or_else(|| {
                    experiment_error(
                        "M2A-HIERARCHY-EXPERIMENT-NODE-MISSING",
                        format!("source.nodes[{order}].children"),
                        "child does not resolve",
                    )
                })?;
            if child_node.parent_ids.as_slice() != [node.id] {
                return Err(experiment_error(
                    "M2A-HIERARCHY-EXPERIMENT-PARENT",
                    format!("source.nodes[{order}].children"),
                    "ordered parent/child identity must be reciprocal and singular",
                ));
            }
        }
    }
    let mesh_nodes = ordered_nodes
        .iter()
        .filter(|node| node.mesh_id.is_some())
        .copied()
        .collect::<Vec<_>>();
    if mesh_nodes.len() != 1
        || ingest.ir.meshes.len() != 1
        || ingest.ir.primitives.len() != 1
        || !ingest.ir.skins.is_empty()
        || !ingest.ir.animations.is_empty()
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-INVENTORY",
            "source",
            "exactly one unskinned mesh/primitive and zero source animations are required",
        ));
    }
    let mesh_node = mesh_nodes[0];
    let mesh_id = mesh_node.mesh_id.expect("filtered mesh node");
    let mesh = &ingest.ir.meshes[0];
    let primitive = &ingest.ir.primitives[0];
    if mesh.id != mesh_id
        || mesh.primitive_ids.as_slice() != [primitive.id]
        || primitive.source_mesh_id != mesh_id
        || !primitive.joints0.is_empty()
        || !primitive.weights0.is_empty()
        || primitive.positions.is_empty()
        || primitive.normals.len() != primitive.positions.len()
        || primitive.uv0.len() != primitive.positions.len()
        || primitive.indices.is_empty()
        || !primitive.indices.len().is_multiple_of(3)
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-GEOMETRY",
            "source.meshes",
            "selected attachment must be one indexed unskinned POSITION/NORMAL/TEXCOORD_0 primitive",
        ));
    }
    let nodes = ordered_nodes
        .iter()
        .enumerate()
        .map(|(order, node)| {
            Ok(SourceTopologyRigidSourceNodeV3 {
                source_order: order as u32,
                node_id: node.id,
                name: node.name.clone(),
                parent_node_id: node.parent_ids.first().copied(),
                ordered_child_ids: node.child_ids.clone(),
                mesh_id: node.mesh_id,
                transform_sha256: canonical_digest(&node.transform, "source.nodes.transform")?,
            })
        })
        .collect::<Result<Vec<_>, SourceTopologyRigidExperimentErrorV3>>()?;
    let summary = SourceTopologyRigidSourceSummaryV3 {
        schema_version: 3,
        profile: SOURCE_TOPOLOGY_RIGID_EXPERIMENT_PROFILE_V3.to_owned(),
        default_scene_id,
        ordered_root_node_ids: scene.root_node_ids.clone(),
        ordered_traversal_node_ids: traversal,
        nodes,
        selected_node_id: mesh_node.id,
        selected_mesh_id: mesh_id,
        selected_primitive_id: primitive.id,
        source_mesh_count: 1,
        source_primitive_count: 1,
        source_skin_count: 0,
        source_animation_count: 0,
        ignored_node_count: 0,
        ignored_mesh_count: 0,
    };
    Ok(SourceShape {
        summary,
        ordered_nodes,
        selected_node_id: mesh_node.id,
        selected_mesh_id: mesh_id,
        selected_primitive_id: primitive.id,
    })
}

fn emit_experiment_model(
    ingest: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile: &SourceTopologyRigidExperimentProfileV3,
) -> Result<BinaryMdlArtifactV1, SourceTopologyRigidExperimentErrorV3> {
    emit_source_topology_model(
        ingest,
        rig,
        &profile.model_resref,
        TEXTURE_RESREF,
        &REQUIRED_CLIPS,
        MdlFormatProfileV1::SourceTopologyPreservingRigidExperimentV1,
    )
}

fn emit_source_topology_model(
    ingest: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    model_resref: &str,
    texture_resref: &str,
    clip_names: &[&str; 7],
    format_profile: MdlFormatProfileV1,
) -> Result<BinaryMdlArtifactV1, SourceTopologyRigidExperimentErrorV3> {
    let conversion = convert_profile_a(ingest, rig, &ProfileAOptionsV1::default())
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    if !conversion.report.conversion_eligible {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-CONVERSION",
            "conversion.report",
            "owned hierarchy conversion is not eligible",
        ));
    }
    let mut creature = conversion.creature.ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-CONVERSION",
            "conversion.creature",
            "eligible conversion omitted the creature",
        )
    })?;
    let roots = creature
        .nodes
        .iter_mut()
        .filter(|node| node.parent_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-OUTPUT-ROOT",
            "conversion.creature.nodes",
            "output must have exactly one root",
        ));
    }
    roots.into_iter().next().expect("one root").name = model_resref.to_owned();
    let animations = MdlAnimationSetV1 {
        schema_version: 1,
        clips: clip_names
            .iter()
            .map(|name| MdlAnimationClipV1 {
                name: (*name).to_owned(),
                animation_root: model_resref.to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: Vec::new(),
            })
            .collect(),
    };
    let material_slot = creature
        .segments
        .first()
        .map(|segment| segment.material_slot)
        .ok_or_else(|| {
            experiment_error(
                "M2A-HIERARCHY-EXPERIMENT-GEOMETRY",
                "conversion.creature.segments",
                "converted experiment has no segment",
            )
        })?;
    write_binary_mdl_with_animations(
        &creature,
        &animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: model_resref.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot,
                resref: texture_resref.to_owned(),
            }],
        },
    )
    .map_err(|error| experiment_error(error.code, error.path, error.message))
}

fn inspect_experiment_output(
    mdl: &[u8],
    shape: &SourceShape<'_>,
    profile: &SourceTopologyRigidExperimentProfileV3,
) -> Result<OutputInspection, SourceTopologyRigidExperimentErrorV3> {
    inspect_source_topology_output(mdl, shape, &profile.model_resref, &REQUIRED_CLIPS)
}

fn inspect_source_topology_output(
    mdl: &[u8],
    shape: &SourceShape<'_>,
    model_resref: &str,
    clip_names: &[&str; 7],
) -> Result<OutputInspection, SourceTopologyRigidExperimentErrorV3> {
    let (envelope, envelope_sha256) = inspect_direct_creature_engine_envelope_v1(mdl)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let parsed = crate::mdl::inspect_binary_mdl(mdl).map_err(|error| {
        experiment_error(
            error.code,
            format!("candidateMdl@{}", error.offset),
            error.context,
        )
    })?;
    let state_summary = verify_direct_creature_state_projection_v1(
        &parsed,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let state_projection_sha256 = direct_creature_structural_summary_digest_v1(&state_summary)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let source_count = shape.ordered_nodes.len();
    if envelope.base_nodes.len() != source_count + 1 || envelope.meshes.len() != 1 {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-OUTPUT-COUNT",
            "engineEnvelope.baseNodes",
            "output must contain one dummy per source transform plus one rigid mesh",
        ));
    }
    let mut source_to_output_nodes = Vec::with_capacity(source_count);
    for (order, source_node) in shape.ordered_nodes.iter().enumerate() {
        let output = &envelope.base_nodes[order];
        let expected_parent = source_node
            .parent_ids
            .first()
            .and_then(|parent| {
                shape
                    .ordered_nodes
                    .iter()
                    .position(|node| node.id == *parent)
            })
            .map(|index| index as u32);
        let expected_name = output_name_for_resref(source_node, order, model_resref);
        if output.order != order as u32
            || output.part != order as u32
            || output.kind != "DUMMY"
            || output.content_flags != 0x01
            || output.name != expected_name
            || output.parent_part != expected_parent
        {
            return Err(experiment_error(
                "M2A-HIERARCHY-EXPERIMENT-OUTPUT-TOPOLOGY",
                format!("engineEnvelope.baseNodes[{order}]"),
                "source order/name/part/parent mapping differs from the explicit hierarchy profile",
            ));
        }
        source_to_output_nodes.push(SourceToOutputNodeV1 {
            source_node_id: source_node.id,
            source_order: order as u32,
            source_name: source_node.name.clone(),
            source_parent_node_id: source_node.parent_ids.first().copied(),
            output_order: output.order,
            output_part: output.part,
            output_name: output.name.clone(),
            output_parent_part: output.parent_part,
            root_resref_normalized: order == 0 && source_node.name.as_deref() != Some(model_resref),
        });
    }
    let output_mesh_node = &envelope.base_nodes[source_count];
    let mesh = &envelope.meshes[0];
    let selected_order = shape
        .ordered_nodes
        .iter()
        .position(|node| node.id == shape.selected_node_id)
        .expect("validated selected node") as u32;
    if output_mesh_node.kind != "MESH"
        || output_mesh_node.parent_part != Some(selected_order)
        || mesh.parent_part != Some(selected_order)
        || mesh.node_part != output_mesh_node.part
        || envelope
            .base_nodes
            .iter()
            .any(|node| node.kind.contains("SKIN"))
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-ATTACHMENT",
            "engineEnvelope.meshes[0]",
            "the sole 0x21 rigid mesh must attach to its exact source-derived parent with no invented joint",
        ));
    }
    if envelope.animations.len() != REQUIRED_CLIPS.len()
        || envelope
            .animations
            .iter()
            .enumerate()
            .any(|(index, animation)| {
                animation.name != clip_names[index]
                    || animation.animation_type != 5
                    || animation.animation_root != model_resref
                    || !animation.topology_matches_base
                    || animation.nodes.iter().any(|node| {
                        node.content_flags != 0x01
                            || node.kind != "DUMMY"
                            || !node.controllers.is_empty()
                    })
            })
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-STATE-PROJECTION",
            "engineEnvelope.animations",
            "all seven type-5 states must mirror the full base identity as controller-free 0x01 dummies",
        ));
    }
    let output_topology_sha256 = canonical_digest(
        &(
            3u32,
            envelope
                .base_nodes
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
                .collect::<Vec<_>>(),
            envelope
                .animations
                .iter()
                .map(|animation| {
                    (
                        &animation.name,
                        animation.animation_type,
                        &animation.animation_root,
                        animation
                            .nodes
                            .iter()
                            .map(|node| {
                                (
                                    node.order,
                                    node.part,
                                    &node.name,
                                    node.parent_part,
                                    node.content_flags,
                                    &node.kind,
                                    node.controllers.len(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        "outputTopology",
    )?;
    let mesh_attachment = SourceMeshAttachmentV1 {
        source_node_id: shape.selected_node_id,
        source_mesh_id: shape.selected_mesh_id,
        source_primitive_id: shape.selected_primitive_id,
        output_mesh_order: 0,
        output_mesh_part: mesh.node_part,
        output_mesh_name: mesh.node_name.clone(),
        output_parent_part: selected_order,
    };
    Ok(OutputInspection {
        envelope,
        envelope_sha256,
        state_projection_sha256,
        output_topology_sha256,
        source_to_output_nodes,
        mesh_attachment,
    })
}

fn protected_writer_fields_digest(
    projection: &DirectCreatureEnvelopeProjectionV1,
    raw_mdx_sha256: &str,
) -> Result<String, SourceTopologyRigidExperimentErrorV3> {
    let mesh = projection.meshes.first().ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-PROTECTED-FIELD",
            "projection.meshes",
            "protected-field projection requires one mesh",
        )
    })?;
    canonical_digest(
        &serde_json::json!({
            "schemaVersion": 1,
            "model": {
                "name": projection.model.name,
                "geometryType": projection.model.geometry_type,
                "classification": projection.model.classification,
                "fog": projection.model.fog,
                "childModelCount": projection.model.child_model_count,
                "boundsMin": projection.model.bounds_min,
                "boundsMax": projection.model.bounds_max,
                "radius": projection.model.radius,
                "animationScale": projection.model.animation_scale,
                "supermodelName": projection.model.supermodel_name,
                "routineWords": projection.model.routine_words,
                "geometryTypePadding": projection.model.geometry_type_padding,
                "runtimeUnknownBytes": projection.model.runtime_unknown_bytes,
                "supermodelPointer": projection.model.supermodel_pointer,
            },
            "mesh": {
                "routineWords": mesh.routine_words,
                "vertexCount": mesh.vertex_count,
                "faceCount": mesh.face_count,
                "textureCount": mesh.texture_count,
                "textures": mesh.textures,
                "vertexColorsPresent": mesh.vertex_colors_present,
                "vertexColorCount": mesh.vertex_color_count,
                "diffuse": mesh.diffuse,
                "ambient": mesh.ambient,
                "specular": mesh.specular,
                "shininess": mesh.shininess,
                "shadow": mesh.shadow,
                "beaming": mesh.beaming,
                "render": mesh.render,
                "transparency": mesh.transparency,
                "renderHint": mesh.render_hint,
                "tileFade": mesh.tile_fade,
                "meshType": mesh.mesh_type,
                "startMdx": mesh.start_mdx,
                "lightMapped": mesh.light_mapped,
                "rotateTexture": mesh.rotate_texture,
                "tailPadding": mesh.tail_padding,
                "vertexNormalSumBits": mesh.vertex_normal_sum_bits,
                "tailUnknown": mesh.tail_unknown,
                "faceSurfaceIdsSha256": mesh.face_surface_ids_sha256,
                "faceAdjacencySha256": mesh.face_adjacency_sha256,
                "geometrySha256": mesh.geometry_sha256,
                "rawPointerSignatureSha256": mesh.raw_pointer_signature_sha256,
            },
            "rawMdxSha256": raw_mdx_sha256,
            "animationHeaders": projection.animations.iter().map(|animation| serde_json::json!({
                "order": animation.order,
                "name": animation.name,
                "animationType": animation.animation_type,
                "animationTypePadding": animation.animation_type_padding,
                "animationRoot": animation.animation_root,
                "runtime68": animation.runtime_68,
            })).collect::<Vec<_>>(),
        }),
        "protectedWriterFields",
    )
}

fn output_name(
    node: &IrNode,
    order: usize,
    profile: &SourceTopologyRigidExperimentProfileV3,
) -> String {
    output_name_for_resref(node, order, &profile.model_resref)
}

fn output_name_for_resref(node: &IrNode, order: usize, model_resref: &str) -> String {
    if order == 0 {
        model_resref.to_owned()
    } else {
        node.name
            .clone()
            .unwrap_or_else(|| format!("m2a_src_{order}"))
    }
}

fn source_geometry_digest(
    ingest: &GlbIngestResult,
    shape: &SourceShape<'_>,
) -> Result<String, SourceTopologyRigidExperimentErrorV3> {
    let node = shape
        .ordered_nodes
        .iter()
        .find(|node| node.id == shape.selected_node_id)
        .expect("validated selected source node");
    let primitive = ingest
        .ir
        .primitives
        .iter()
        .find(|primitive| primitive.id == shape.selected_primitive_id)
        .expect("validated selected source primitive");
    canonical_digest(
        &(
            3u32,
            shape.selected_node_id,
            shape.selected_mesh_id,
            shape.selected_primitive_id,
            &node.transform,
            &primitive.topology,
            primitive.material_id,
            primitive.source_was_indexed,
            &primitive.positions,
            &primitive.normals,
            &primitive.uv0,
            &primitive.indices,
        ),
        "sourceGeometry",
    )
}

pub fn derive_meshy_m0_hierarchy_source_v4(
    original_glb: &[u8],
    original_informational_path: impl Into<String>,
    derived_informational_path: impl Into<String>,
) -> Result<MeshyHierarchyDerivedSourceArtifactV4, SourceTopologyRigidExperimentErrorV3> {
    require_frozen_m0_source(original_glb)?;
    let original_informational_path = original_informational_path.into();
    let derived_informational_path = derived_informational_path.into();
    if original_informational_path.trim().is_empty() || derived_informational_path.trim().is_empty()
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-DERIVATION-PATH",
            "derivedSourceBinding",
            "original and derived informational paths must be explicit",
        ));
    }
    let (json_bytes, original_bin) = split_glb_v4(original_glb)?;
    let mut root: serde_json::Value = serde_json::from_slice(json_bytes).map_err(|error| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-JSON",
            "originalSource.json",
            error.to_string(),
        )
    })?;
    let original_ingest = ingest_glb(original_glb, &GlbLimits::default()).map_err(|error| {
        experiment_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "originalSource".to_owned()),
            error.message,
        )
    })?;
    require_flat_frozen_source_shape(&original_ingest)?;
    root["scene"] = serde_json::json!(0);
    root["scenes"] = serde_json::json!([{ "nodes": [0] }]);
    root["nodes"] = serde_json::json!([
        {
            "name": DERIVED_NODE_NAMES_V1[0],
            "children": [1],
            "matrix": identity_matrix(),
        },
        {
            "name": DERIVED_NODE_NAMES_V1[1],
            "children": [2],
            "matrix": identity_matrix(),
        },
        {
            "name": DERIVED_NODE_NAMES_V1[2],
            "mesh": 0,
            "matrix": identity_matrix(),
        }
    ]);
    let mut derived_json = serde_json::to_vec(&root).map_err(|error| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-JSON",
            "derivedSource.json",
            error.to_string(),
        )
    })?;
    while !derived_json.len().is_multiple_of(4) {
        derived_json.push(b' ');
    }
    let total_length = 12usize
        .checked_add(8)
        .and_then(|value| value.checked_add(derived_json.len()))
        .and_then(|value| value.checked_add(8))
        .and_then(|value| value.checked_add(original_bin.len()))
        .ok_or_else(|| {
            experiment_error(
                "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
                "derivedSource",
                "derived GLB length overflow",
            )
        })?;
    let total_length_u32 = u32::try_from(total_length).map_err(|_| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
            "derivedSource",
            "derived GLB exceeds u32 length",
        )
    })?;
    let json_length_u32 = u32::try_from(derived_json.len()).map_err(|_| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
            "derivedSource.json",
            "derived JSON exceeds u32 length",
        )
    })?;
    let bin_length_u32 = u32::try_from(original_bin.len()).map_err(|_| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
            "derivedSource.bin",
            "derived BIN exceeds u32 length",
        )
    })?;
    let mut bytes = Vec::with_capacity(total_length);
    bytes.extend_from_slice(b"glTF");
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&total_length_u32.to_le_bytes());
    bytes.extend_from_slice(&json_length_u32.to_le_bytes());
    bytes.extend_from_slice(b"JSON");
    bytes.extend_from_slice(&derived_json);
    bytes.extend_from_slice(&bin_length_u32.to_le_bytes());
    bytes.extend_from_slice(b"BIN\0");
    bytes.extend_from_slice(original_bin);

    let derived_ingest = ingest_glb(&bytes, &GlbLimits::default()).map_err(|error| {
        experiment_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "derivedSource".to_owned()),
            error.message,
        )
    })?;
    let mut shape = inspect_source_shape(&derived_ingest)?;
    shape.summary.profile = MESHY_HIERARCHY_CANDIDATE_PROFILE_V4.to_owned();
    let original_geometry_sha256 = single_mesh_geometry_digest_v4(&original_ingest)?;
    let derived_geometry_sha256 = single_mesh_geometry_digest_v4(&derived_ingest)?;
    if original_geometry_sha256 != derived_geometry_sha256 {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GEOMETRY-DRIFT",
            "derivedSource.geometry",
            "topology derivation changed selected positions/normals/UV/indices or transform semantics",
        ));
    }
    let (_, derived_bin) = split_glb_v4(&bytes)?;
    if original_bin != derived_bin {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-BIN-DRIFT",
            "derivedSource.bin",
            "topology derivation must preserve the exact GLB BIN payload",
        ));
    }
    let topology_summary_sha256 = canonical_digest(&shape.summary, "derivedTopologySummary")?;
    let binding = MeshyHierarchyDerivedSourceBindingV4 {
        schema_version: 4,
        profile: MESHY_HIERARCHY_DERIVATION_V1.to_owned(),
        derivation_version: 1,
        original_informational_path,
        derived_informational_path,
        origin: SourceTopologyOriginV1::UserDerived,
        original_source_byte_length: original_glb.len() as u64,
        original_source_sha256: sha256_bytes(original_glb),
        derived_source_byte_length: bytes.len() as u64,
        derived_source_sha256: sha256_bytes(&bytes),
        original_bin_byte_length: original_bin.len() as u64,
        original_bin_sha256: sha256_bytes(original_bin),
        derived_bin_byte_length: derived_bin.len() as u64,
        derived_bin_sha256: sha256_bytes(derived_bin),
        original_geometry_sha256,
        derived_geometry_sha256,
        topology_summary: shape.summary,
        topology_summary_sha256,
        node_identity_policy: "PROJECT_OWNED_NEUTRAL_ROOT_HIER_1_MESH_ANCHOR_V1".to_owned(),
    };
    Ok(MeshyHierarchyDerivedSourceArtifactV4 { bytes, binding })
}

pub fn declare_meshy_hierarchy_candidate_profile_v4(
    original_glb: &[u8],
    derived: &MeshyHierarchyDerivedSourceArtifactV4,
    frozen_r30_model: &[u8],
    source_origin: SourceTopologyOriginV1,
    model_resref: impl Into<String>,
    texture_resref: impl Into<String>,
) -> Result<MeshyHierarchyCandidateProfileV4, SourceTopologyRigidExperimentErrorV3> {
    require_frozen_m0_source(original_glb)?;
    require_frozen_r30_model(frozen_r30_model)?;
    if source_origin != SourceTopologyOriginV1::UserDerived {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-ORIGIN",
            "profile.sourceOrigin",
            "candidate topology must be explicitly USER_DERIVED",
        ));
    }
    let replay = derive_meshy_m0_hierarchy_source_v4(
        original_glb,
        derived.binding.original_informational_path.clone(),
        derived.binding.derived_informational_path.clone(),
    )?;
    if &replay != derived {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-DERIVATION-MISMATCH",
            "derivedSource",
            "derived bytes and binding differ from exact deterministic replay",
        ));
    }
    let model_resref = model_resref.into();
    let texture_resref = texture_resref.into();
    if model_resref != "m2a_m0p01" || texture_resref != M0_TEXTURE_RESREF {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-RESOURCE-IDENTITY",
            "profile",
            "the hierarchy-only candidate preserves exact M0 model and texture resrefs",
        ));
    }
    Ok(MeshyHierarchyCandidateProfileV4 {
        schema_version: 4,
        profile: MESHY_HIERARCHY_CANDIDATE_PROFILE_V4.to_owned(),
        source_origin,
        original_informational_path: derived.binding.original_informational_path.clone(),
        derived_informational_path: derived.binding.derived_informational_path.clone(),
        derivation_profile: MESHY_HIERARCHY_DERIVATION_V1.to_owned(),
        derivation_profile_version: 1,
        derived_source_binding_sha256: canonical_digest(&derived.binding, "derivedSourceBinding")?,
        original_source_byte_length: original_glb.len() as u64,
        original_source_sha256: sha256_bytes(original_glb),
        derived_source_byte_length: derived.bytes.len() as u64,
        derived_source_sha256: sha256_bytes(&derived.bytes),
        frozen_r30_model_byte_length: frozen_r30_model.len() as u64,
        frozen_r30_model_sha256: sha256_bytes(frozen_r30_model),
        writer_profile: "SOURCE_TOPOLOGY_PRESERVING_RIGID_CANDIDATE_V1".to_owned(),
        writer_profile_version: 1,
        state_projection_profile: "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1".to_owned(),
        model_resref,
        texture_resref,
    })
}

pub fn build_meshy_hierarchy_candidate_model_v4(
    original_glb: &[u8],
    derived_glb: &[u8],
    frozen_r30_model: &[u8],
    profile: &MeshyHierarchyCandidateProfileV4,
) -> Result<MeshyHierarchyCandidateModelArtifactV4, SourceTopologyRigidExperimentErrorV3> {
    let derived = require_meshy_hierarchy_candidate_profile_v4(
        original_glb,
        derived_glb,
        frozen_r30_model,
        profile,
    )?;
    let source_ingest = ingest_glb(derived_glb, &GlbLimits::default()).map_err(|error| {
        experiment_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "derivedSource".to_owned()),
            error.message,
        )
    })?;
    let shape = inspect_source_shape(&source_ingest)?;
    let mut emission_ingest = source_ingest.clone();
    sanitize_degenerate_triangles_v4(&mut emission_ingest)?;
    let mut rig = derive_meshy_m0_static_rigid_profile_v1(&emission_ingest)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    rig.profile_id = "MESHY_SOURCE_DERIVED_HIERARCHY_CANDIDATE_V4".to_owned();
    rig.nodes = shape
        .ordered_nodes
        .iter()
        .enumerate()
        .map(|(order, node)| CreatureRigNodeV1 {
            id: node.id,
            name: output_name_for_resref(node, order, &profile.model_resref),
            parent_id: node.parent_ids.first().copied(),
            bind_local_matrix: identity_matrix(),
        })
        .collect();
    rig.segments[0].parent_node_id = shape.selected_node_id;
    rig.content_sha256 = canonical_profile_sha256(&rig)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let model = emit_source_topology_model(
        &emission_ingest,
        &rig,
        &profile.model_resref,
        &profile.texture_resref,
        &M0_RUNTIME_CLIPS,
        MdlFormatProfileV1::SourceTopologyPreservingRigidCandidateV1,
    )?;
    let inspection = inspect_source_topology_output(
        &model.payload,
        &shape,
        &profile.model_resref,
        &M0_RUNTIME_CLIPS,
    )?;
    let (candidate_projection, _) = inspect_direct_creature_envelope_projection_v1(&model.payload)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let (r30_envelope, _) = inspect_direct_creature_engine_envelope_v1(frozen_r30_model)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let (r30_projection, _) = inspect_direct_creature_envelope_projection_v1(frozen_r30_model)
        .map_err(|error| experiment_error(error.code, error.path, error.message))?;
    let candidate_protected =
        protected_writer_fields_digest(&candidate_projection, &inspection.envelope.raw_mdx.sha256)?;
    let r30_protected =
        protected_writer_fields_digest(&r30_projection, &r30_envelope.raw_mdx.sha256)?;
    if inspection.envelope.raw_mdx.sha256 != r30_envelope.raw_mdx.sha256 {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-RAW-MDX-DRIFT",
            "candidateModel.rawMdx",
            "candidate raw MDX must remain byte-identical to frozen exact r30",
        ));
    }
    if candidate_protected != r30_protected {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-PROTECTED-FIELD-DRIFT",
            "candidateModel",
            "hierarchy candidate changed a protected non-topology writer field relative to r30",
        ));
    }
    let contract = MeshyHierarchyCandidateModelContractV4 {
        schema_version: 4,
        profile: MESHY_HIERARCHY_CANDIDATE_CONTRACT_V4.to_owned(),
        candidate_admissible: true,
        structural_verdict: "STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED".to_owned(),
        runtime_model_visibility: "not_tested".to_owned(),
        runtime_proof_completeness: "missing".to_owned(),
        candidate_profile_sha256: canonical_digest(profile, "candidateProfile")?,
        derived_source_binding: derived.binding.clone(),
        derived_source_binding_sha256: canonical_digest(&derived.binding, "derivedSourceBinding")?,
        source_to_output_nodes: inspection.source_to_output_nodes,
        mesh_attachment: inspection.mesh_attachment,
        output_topology_sha256: inspection.output_topology_sha256,
        candidate_model_byte_length: model.payload.len() as u64,
        candidate_model_sha256: sha256_bytes(&model.payload),
        candidate_engine_envelope: inspection.envelope,
        candidate_engine_envelope_sha256: inspection.envelope_sha256,
        candidate_state_projection_sha256: inspection.state_projection_sha256,
        candidate_raw_mdx_sha256: r30_envelope.raw_mdx.sha256.clone(),
        candidate_protected_writer_fields_sha256: candidate_protected,
        r30_model_byte_length: frozen_r30_model.len() as u64,
        r30_model_sha256: sha256_bytes(frozen_r30_model),
        r30_raw_mdx_sha256: r30_envelope.raw_mdx.sha256,
        r30_protected_writer_fields_sha256: r30_protected,
    };
    Ok(MeshyHierarchyCandidateModelArtifactV4 { model, contract })
}

pub fn verify_meshy_hierarchy_candidate_model_v4(
    contract: &MeshyHierarchyCandidateModelContractV4,
    expected_profile: &MeshyHierarchyCandidateProfileV4,
    original_glb: &[u8],
    derived_glb: &[u8],
    frozen_r30_model: &[u8],
    candidate_model: &[u8],
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    if contract.schema_version != 4
        || contract.profile != MESHY_HIERARCHY_CANDIDATE_CONTRACT_V4
        || !contract.candidate_admissible
        || contract.structural_verdict != "STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED"
        || contract.runtime_model_visibility != "not_tested"
        || contract.runtime_proof_completeness != "missing"
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-CONTRACT-VERSION",
            "contract",
            "only the explicit V4 candidate contract with an untested runtime axis is admissible",
        ));
    }
    let replay = build_meshy_hierarchy_candidate_model_v4(
        original_glb,
        derived_glb,
        frozen_r30_model,
        expected_profile,
    )?;
    if contract != &replay.contract || candidate_model != replay.model.payload {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-REPLAY-MISMATCH",
            "contract",
            "candidate model or contract differs from independent exact-source replay",
        ));
    }
    Ok(())
}

fn require_meshy_hierarchy_candidate_profile_v4(
    original_glb: &[u8],
    derived_glb: &[u8],
    frozen_r30_model: &[u8],
    profile: &MeshyHierarchyCandidateProfileV4,
) -> Result<MeshyHierarchyDerivedSourceArtifactV4, SourceTopologyRigidExperimentErrorV3> {
    require_frozen_m0_source(original_glb)?;
    require_frozen_r30_model(frozen_r30_model)?;
    if profile.schema_version != 4
        || profile.profile != MESHY_HIERARCHY_CANDIDATE_PROFILE_V4
        || profile.source_origin != SourceTopologyOriginV1::UserDerived
        || profile.original_informational_path.trim().is_empty()
        || profile.derived_informational_path.trim().is_empty()
        || profile.derivation_profile != MESHY_HIERARCHY_DERIVATION_V1
        || profile.derivation_profile_version != 1
        || profile.writer_profile != "SOURCE_TOPOLOGY_PRESERVING_RIGID_CANDIDATE_V1"
        || profile.writer_profile_version != 1
        || profile.state_projection_profile != "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1"
        || profile.model_resref != "m2a_m0p01"
        || profile.texture_resref != M0_TEXTURE_RESREF
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-PROFILE-VERSION",
            "profile",
            "unknown, legacy, experiment-only, partial, or mixed candidate profile is inadmissible",
        ));
    }
    let derived = derive_meshy_m0_hierarchy_source_v4(
        original_glb,
        profile.original_informational_path.clone(),
        profile.derived_informational_path.clone(),
    )?;
    if derived.bytes != derived_glb {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-DERIVED-SOURCE-MISMATCH",
            "derivedSource",
            "exact derived GLB differs from deterministic project-owned derivation",
        ));
    }
    let binding_sha = canonical_digest(&derived.binding, "derivedSourceBinding")?;
    if profile.derived_source_binding_sha256 != binding_sha
        || profile.original_source_byte_length != original_glb.len() as u64
        || profile.original_source_sha256 != sha256_bytes(original_glb)
        || profile.derived_source_byte_length != derived_glb.len() as u64
        || profile.derived_source_sha256 != sha256_bytes(derived_glb)
        || profile.frozen_r30_model_byte_length != frozen_r30_model.len() as u64
        || profile.frozen_r30_model_sha256 != sha256_bytes(frozen_r30_model)
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-TRUST-ROOT",
            "profile",
            "original, derived, r30, or derivation binding identity differs from the caller-owned V4 trust root",
        ));
    }
    Ok(derived)
}

fn require_frozen_m0_source(
    original_glb: &[u8],
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    if original_glb.len() as u64 != FROZEN_M0_SOURCE_BYTE_LENGTH
        || sha256_bytes(original_glb) != FROZEN_M0_SOURCE_SHA256
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-FROZEN-SOURCE",
            "originalSource",
            "candidate lineage requires the exact frozen M0 Meshy source identity",
        ));
    }
    Ok(())
}

fn require_frozen_r30_model(model: &[u8]) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    if model.len() as u64 != FROZEN_R30_MODEL_BYTE_LENGTH
        || sha256_bytes(model) != FROZEN_R30_MODEL_SHA256
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-FROZEN-R30-MODEL",
            "r30Model",
            "candidate lineage requires the exact frozen runtime-negative r30 model",
        ));
    }
    Ok(())
}

fn require_flat_frozen_source_shape(
    ingest: &GlbIngestResult,
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    let scene_ok = ingest.ir.scenes.len() == 1
        && ingest.ir.default_scene_id == Some(0)
        && ingest.ir.scenes[0].root_node_ids.as_slice() == [0];
    let node_ok = ingest.ir.nodes.len() == 1
        && ingest.ir.nodes[0].id == 0
        && ingest.ir.nodes[0].mesh_id == Some(0)
        && ingest.ir.nodes[0].parent_ids.is_empty()
        && ingest.ir.nodes[0].child_ids.is_empty()
        && is_identity_transform(&ingest.ir.nodes[0]);
    if !scene_ok
        || !node_ok
        || ingest.ir.meshes.len() != 1
        || ingest.ir.primitives.len() != 1
        || !ingest.ir.skins.is_empty()
        || !ingest.ir.animations.is_empty()
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-FROZEN-SOURCE-SHAPE",
            "originalSource",
            "frozen M0 must remain one scene/root/unskinned mesh/primitive with no ignored source content",
        ));
    }
    Ok(())
}

fn single_mesh_geometry_digest_v4(
    ingest: &GlbIngestResult,
) -> Result<String, SourceTopologyRigidExperimentErrorV3> {
    let node = ingest
        .ir
        .nodes
        .iter()
        .find(|node| node.mesh_id.is_some())
        .ok_or_else(|| {
            experiment_error(
                "M2A-HIERARCHY-CANDIDATE-GEOMETRY",
                "source.nodes",
                "selected mesh node is absent",
            )
        })?;
    let primitive = ingest.ir.primitives.first().ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GEOMETRY",
            "source.primitives",
            "selected primitive is absent",
        )
    })?;
    canonical_digest(
        &(
            4u32,
            &node.transform,
            &primitive.topology,
            primitive.material_id,
            primitive.source_was_indexed,
            &primitive.positions,
            &primitive.normals,
            &primitive.uv0,
            &primitive.indices,
            &primitive.joints0,
            &primitive.weights0,
        ),
        "sourceGeometryV4",
    )
}

fn sanitize_degenerate_triangles_v4(
    ingest: &mut GlbIngestResult,
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    if ingest.ir.primitives.len() != 1 {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GEOMETRY",
            "source.primitives",
            "candidate requires exactly one primitive before deterministic sanitation",
        ));
    }
    let primitive = &mut ingest.ir.primitives[0];
    let mut retained = Vec::with_capacity(primitive.indices.len());
    for triangle in primitive.indices.chunks_exact(3) {
        let a = primitive.positions[triangle[0] as usize];
        let b = primitive.positions[triangle[1] as usize];
        let c = primitive.positions[triangle[2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let length_squared = cross.iter().map(|value| value * value).sum::<f32>();
        if length_squared.is_finite() && length_squared > 1.0e-10 {
            retained.extend_from_slice(triangle);
        }
    }
    if retained.is_empty() {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GEOMETRY",
            "source.primitives[0].indices",
            "candidate source contains no Aurora-safe non-degenerate triangles",
        ));
    }
    primitive.indices = retained;
    Ok(())
}

fn split_glb_v4(glb: &[u8]) -> Result<(&[u8], &[u8]), SourceTopologyRigidExperimentErrorV3> {
    let read_u32 = |offset: usize| -> Result<u32, SourceTopologyRigidExperimentErrorV3> {
        glb.get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or_else(|| {
                experiment_error(
                    "M2A-HIERARCHY-CANDIDATE-GLB-LAYOUT",
                    "sourceGlb",
                    "GLB header or chunk field is truncated",
                )
            })
    };
    if glb.get(..4) != Some(b"glTF") || read_u32(4)? != 2 || read_u32(8)? as usize != glb.len() {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LAYOUT",
            "sourceGlb",
            "expected exact GLB 2.0 identity and declared file length",
        ));
    }
    let json_length = read_u32(12)? as usize;
    if glb.get(16..20) != Some(b"JSON") || !json_length.is_multiple_of(4) {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LAYOUT",
            "sourceGlb.json",
            "first GLB chunk must be aligned JSON",
        ));
    }
    let json_end = 20usize.checked_add(json_length).ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
            "sourceGlb.json",
            "JSON range overflow",
        )
    })?;
    let bin_length = read_u32(json_end)? as usize;
    let bin_start = json_end.checked_add(8).ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
            "sourceGlb.bin",
            "BIN range overflow",
        )
    })?;
    let bin_end = bin_start.checked_add(bin_length).ok_or_else(|| {
        experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LENGTH",
            "sourceGlb.bin",
            "BIN range overflow",
        )
    })?;
    if glb.get(json_end + 4..json_end + 8) != Some(b"BIN\0")
        || !bin_length.is_multiple_of(4)
        || bin_end != glb.len()
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-CANDIDATE-GLB-LAYOUT",
            "sourceGlb.bin",
            "second GLB chunk must be aligned BIN consuming exact EOF",
        ));
    }
    Ok((&glb[20..json_end], &glb[bin_start..bin_end]))
}

fn require_profile_source(
    profile: &SourceTopologyRigidExperimentProfileV3,
    source_glb: &[u8],
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    if profile.schema_version != 3
        || profile.profile != SOURCE_TOPOLOGY_RIGID_EXPERIMENT_PROFILE_V3
        || profile.conversion_profile != SOURCE_TOPOLOGY_RIGID_CONVERSION_V1
        || profile.conversion_profile_version != 1
        || profile.writer_profile != SOURCE_TOPOLOGY_RIGID_WRITER_V1
        || profile.writer_profile_version != 1
        || profile.state_projection_profile != "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1"
        || profile.nullable_name_rule != NULLABLE_NAME_RULE_V1
        || profile.source_informational_path.trim().is_empty()
        || !valid_resref(&profile.model_resref)
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-PROFILE-VERSION",
            "profile",
            "unknown, partial, defaulted, or mixed experiment profile is inadmissible",
        ));
    }
    let sha256 = sha256_bytes(source_glb);
    if profile.source_byte_length != source_glb.len() as u64
        || profile.source_sha256 != sha256
        || profile.source_canonical_identity != format!("sha256:{sha256}")
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-SOURCE-TRUST-ROOT",
            "profile.source",
            "exact caller-owned source SHA and byte length differ",
        ));
    }
    Ok(())
}

fn require_contract_version(
    contract: &SourceTopologyRigidExperimentContractV3,
) -> Result<(), SourceTopologyRigidExperimentErrorV3> {
    if contract.schema_version != 3
        || contract.profile != SOURCE_TOPOLOGY_RIGID_EXPERIMENT_CONTRACT_V3
        || contract.runtime_admissible
        || contract.structural_verdict != "OFFLINE_STRUCTURAL_EXPERIMENT_ONLY"
    {
        return Err(experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-CONTRACT-VERSION",
            "contract",
            "only the explicit non-admitting V3 offline contract is accepted",
        ));
    }
    Ok(())
}

fn is_identity_transform(node: &IrNode) -> bool {
    let matrix_identity = node
        .transform
        .matrix
        .is_none_or(|matrix| matrix == identity_matrix());
    let translation_identity = node
        .transform
        .translation
        .is_none_or(|value| value == [0.0, 0.0, 0.0]);
    let rotation_identity = node
        .transform
        .rotation
        .is_none_or(|value| value == [0.0, 0.0, 0.0, 1.0]);
    let scale_identity = node
        .transform
        .scale
        .is_none_or(|value| value == [1.0, 1.0, 1.0]);
    matrix_identity && translation_identity && rotation_identity && scale_identity
}

fn identity_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn valid_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_node_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 31
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn canonical_digest<T: Serialize>(
    value: &T,
    path: &str,
) -> Result<String, SourceTopologyRigidExperimentErrorV3> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        experiment_error(
            "M2A-HIERARCHY-EXPERIMENT-SERIALIZATION",
            path,
            format!("canonical versioned serialization failed: {error}"),
        )
    })?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn experiment_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> SourceTopologyRigidExperimentErrorV3 {
    SourceTopologyRigidExperimentErrorV3 {
        schema_version: 3,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
