use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    glb::{GlbIngestResult, GlbLimits, ingest_glb},
    mdl::inspect_direct_creature_engine_envelope_v1,
};

pub const DIRECT_CREATURE_RUNTIME_PROFILE_V2: &str =
    "M0_DIRECT_CREATURE_SOURCE_TOPOLOGY_ENGINE_ENVELOPE_V2";
pub const SOURCE_TOPOLOGY_PROFILE_V1: &str = "SOURCE_TOPOLOGY_BINDING_V1";
pub const M0_CONVERSION_PROFILE_V1: &str = "meshy-m0-static-rigid-user-profile-v1";
pub const M0_WRITER_PROFILE_V1: &str = "M0_STATIC_RIGID_NATIVE_V1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureContractErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for DirectCreatureContractErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for DirectCreatureContractErrorV1 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceTopologyOriginV1 {
    UserDerived,
    UserDeclared,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureRuntimeProfileV2 {
    pub schema_version: u32,
    pub profile: String,
    /// Informational only. It is never a trust root and may be a content URI.
    pub source_informational_path: String,
    pub source_canonical_identity: String,
    pub source_byte_length: u64,
    pub source_sha256: String,
    pub source_origin: SourceTopologyOriginV1,
    pub conversion_profile: String,
    pub conversion_profile_version: u32,
    pub writer_profile: String,
    pub writer_profile_version: u32,
    pub topology_profile: String,
    pub topology_profile_version: u32,
    pub engine_envelope_profile: String,
    pub engine_envelope_profile_version: u32,
    pub model_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSceneTopologyV1 {
    pub scene_id: u32,
    pub name: Option<String>,
    pub ordered_root_node_ids: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceNodeTopologyV1 {
    pub source_order: u32,
    pub node_id: u32,
    pub name: Option<String>,
    pub ordered_parent_ids: Vec<u32>,
    pub ordered_child_ids: Vec<u32>,
    pub mesh_id: Option<u32>,
    pub skin_id: Option<u32>,
    pub reachable_from_default_scene: bool,
    pub default_scene_traversal_order: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMeshTopologyV1 {
    pub source_order: u32,
    pub mesh_id: u32,
    pub name: Option<String>,
    pub ordered_primitive_ids: Vec<u32>,
    pub reachable_from_default_scene: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePrimitiveTopologyV1 {
    pub source_order: u32,
    pub primitive_id: u32,
    pub source_mesh_id: u32,
    pub source_primitive_index: u32,
    pub topology: String,
    pub material_id: Option<u32>,
    pub position_count: u32,
    pub normal_count: u32,
    pub tangent_count: u32,
    pub uv0_count: u32,
    pub joint_lane_count: u32,
    pub weight_lane_count: u32,
    pub index_count: u32,
    pub source_was_indexed: bool,
    pub attributes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSkinTopologyV1 {
    pub source_order: u32,
    pub skin_id: u32,
    pub name: Option<String>,
    pub skeleton_root_node_id: Option<u32>,
    pub ordered_joint_node_ids: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTopologySummaryV1 {
    pub schema_version: u32,
    pub profile: String,
    pub default_scene_id: u32,
    pub scenes: Vec<SourceSceneTopologyV1>,
    pub nodes: Vec<SourceNodeTopologyV1>,
    pub meshes: Vec<SourceMeshTopologyV1>,
    pub primitives: Vec<SourcePrimitiveTopologyV1>,
    pub skins: Vec<SourceSkinTopologyV1>,
    pub ordered_default_scene_traversal: Vec<u32>,
    pub ignored_node_count: u32,
    pub ignored_mesh_count: u32,
    pub duplicate_non_null_node_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePrimitiveSelectionV1 {
    pub scene_id: u32,
    pub node_id: u32,
    pub mesh_id: u32,
    pub primitive_id: u32,
    pub source_primitive_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceToOutputNodeV1 {
    pub source_node_id: u32,
    pub source_order: u32,
    pub source_name: Option<String>,
    pub source_parent_node_id: Option<u32>,
    pub output_order: u32,
    pub output_part: u32,
    pub output_name: String,
    pub output_parent_part: Option<u32>,
    pub root_resref_normalized: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMeshAttachmentV1 {
    pub source_node_id: u32,
    pub source_mesh_id: u32,
    pub source_primitive_id: u32,
    pub output_mesh_order: u32,
    pub output_mesh_part: u32,
    pub output_mesh_name: String,
    pub output_parent_part: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceGeometryMappingV1 {
    pub schema_version: u32,
    pub mapping_profile: String,
    pub source_geometry_sha256: String,
    pub final_mdl_geometry_sha256: String,
    pub mapping_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTopologyBindingV1 {
    pub schema_version: u32,
    pub profile: String,
    pub runtime_profile_sha256: String,
    pub source_informational_path: String,
    pub source_canonical_identity: String,
    pub source_byte_length: u64,
    pub source_sha256: String,
    pub origin: SourceTopologyOriginV1,
    pub conversion_profile: String,
    pub conversion_profile_version: u32,
    pub final_mdl_resref: String,
    pub final_mdl_byte_length: u64,
    pub final_mdl_sha256: String,
    pub selected: SourcePrimitiveSelectionV1,
    pub topology_summary: SourceTopologySummaryV1,
    pub topology_summary_sha256: String,
    pub source_to_output_nodes: Vec<SourceToOutputNodeV1>,
    pub mesh_attachment: SourceMeshAttachmentV1,
    pub geometry_mapping: SourceGeometryMappingV1,
}

pub fn declare_m0_direct_creature_runtime_profile_v2(
    source_glb: &[u8],
    source_informational_path: impl Into<String>,
    origin: SourceTopologyOriginV1,
    model_resref: impl Into<String>,
) -> Result<DirectCreatureRuntimeProfileV2, DirectCreatureContractErrorV1> {
    let path = source_informational_path.into();
    let model_resref = model_resref.into();
    if path.trim().is_empty() || model_resref.trim().is_empty() {
        return Err(contract_error(
            "M2A-DIRECT-CREATURE-PROFILE-INVALID",
            "profile",
            "informational source path and model resref must be explicit",
        ));
    }
    let sha256 = sha256_bytes(source_glb);
    Ok(DirectCreatureRuntimeProfileV2 {
        schema_version: 2,
        profile: DIRECT_CREATURE_RUNTIME_PROFILE_V2.to_owned(),
        source_informational_path: path,
        source_canonical_identity: format!("sha256:{sha256}"),
        source_byte_length: source_glb.len() as u64,
        source_sha256: sha256,
        source_origin: origin,
        conversion_profile: M0_CONVERSION_PROFILE_V1.to_owned(),
        conversion_profile_version: 1,
        writer_profile: M0_WRITER_PROFILE_V1.to_owned(),
        writer_profile_version: 1,
        topology_profile: SOURCE_TOPOLOGY_PROFILE_V1.to_owned(),
        topology_profile_version: 1,
        engine_envelope_profile: "DIRECT_CREATURE_ENGINE_ENVELOPE_V1".to_owned(),
        engine_envelope_profile_version: 1,
        model_resref,
    })
}

pub fn direct_creature_runtime_profile_digest_v2(
    profile: &DirectCreatureRuntimeProfileV2,
) -> Result<String, DirectCreatureContractErrorV1> {
    require_profile_v2(profile)?;
    canonical_digest(profile, "runtimeProfile")
}

pub fn source_topology_summary_digest_v1(
    summary: &SourceTopologySummaryV1,
) -> Result<String, DirectCreatureContractErrorV1> {
    if summary.schema_version != 1 || summary.profile != SOURCE_TOPOLOGY_PROFILE_V1 {
        return Err(contract_error(
            "M2A-SOURCE-TOPOLOGY-VERSION",
            "sourceTopology.summary",
            "canonical topology digest requires exact SOURCE_TOPOLOGY_BINDING_V1",
        ));
    }
    canonical_digest(summary, "sourceTopology.summary")
}

pub fn inspect_m0_source_topology_binding_v1(
    source_glb: &[u8],
    final_mdl: &[u8],
    profile: &DirectCreatureRuntimeProfileV2,
) -> Result<SourceTopologyBindingV1, DirectCreatureContractErrorV1> {
    require_profile_source(profile, source_glb)?;
    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        contract_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let (summary, selected) = summarize_and_require_exact_m0_source(&ingest)?;
    let topology_summary_sha256 = source_topology_summary_digest_v1(&summary)?;
    let (envelope, _) = inspect_direct_creature_engine_envelope_v1(final_mdl)
        .map_err(|error| contract_error(error.code, error.path, error.message))?;
    let root = envelope.base_nodes.first().ok_or_else(|| {
        contract_error(
            "M2A-SOURCE-TOPOLOGY-OUTPUT-ROOT",
            "engineEnvelope.baseNodes",
            "M0 output must contain a root node",
        )
    })?;
    if envelope.base_nodes.len() != 2
        || envelope.meshes.len() != 1
        || root.output_kind() != "DUMMY"
        || root.parent_part.is_some()
        || root.name != profile.model_resref
        || envelope.base_nodes[1].kind != "MESH"
        || envelope.base_nodes[1].parent_part != Some(root.part)
        || envelope
            .base_nodes
            .iter()
            .any(|node| node.kind.contains("SKIN"))
    {
        return Err(contract_error(
            "M2A-SOURCE-TOPOLOGY-OUTPUT-MISMATCH",
            "engineEnvelope.baseNodes",
            "exact M0 output requires resref-normalized root plus one attached rigid mesh and no invented joints",
        ));
    }
    let source_node = summary.nodes[0].clone();
    let source_mesh_id = summary.meshes[0].mesh_id;
    let source_primitive_id = summary.primitives[0].primitive_id;
    let mesh = &envelope.meshes[0];
    if mesh.parent_part != Some(root.part) || mesh.node_part != envelope.base_nodes[1].part {
        return Err(contract_error(
            "M2A-SOURCE-TOPOLOGY-ATTACHMENT-MISMATCH",
            "engineEnvelope.meshes[0]",
            "final mesh attachment does not target the source-derived output root",
        ));
    }
    let source_geometry_sha256 = source_geometry_digest(&ingest, &selected)?;
    let mapping_profile = "M0_SOURCE_WORLD_AURORA_BASIS_GROUND_TO_RIGID_MDL_V1".to_owned();
    let mapping_sha256 = canonical_digest(
        &(
            1u32,
            &mapping_profile,
            &selected,
            &source_geometry_sha256,
            &mesh.geometry_sha256,
        ),
        "sourceTopology.geometryMapping",
    )?;
    Ok(SourceTopologyBindingV1 {
        schema_version: 1,
        profile: SOURCE_TOPOLOGY_PROFILE_V1.to_owned(),
        runtime_profile_sha256: direct_creature_runtime_profile_digest_v2(profile)?,
        source_informational_path: profile.source_informational_path.clone(),
        source_canonical_identity: profile.source_canonical_identity.clone(),
        source_byte_length: profile.source_byte_length,
        source_sha256: profile.source_sha256.clone(),
        origin: profile.source_origin,
        conversion_profile: profile.conversion_profile.clone(),
        conversion_profile_version: profile.conversion_profile_version,
        final_mdl_resref: profile.model_resref.clone(),
        final_mdl_byte_length: final_mdl.len() as u64,
        final_mdl_sha256: sha256_bytes(final_mdl),
        selected,
        topology_summary: summary,
        topology_summary_sha256,
        source_to_output_nodes: vec![SourceToOutputNodeV1 {
            source_node_id: source_node.node_id,
            source_order: source_node.source_order,
            source_name: source_node.name.clone(),
            source_parent_node_id: None,
            output_order: root.order,
            output_part: root.part,
            output_name: root.name.clone(),
            output_parent_part: None,
            root_resref_normalized: source_node.name.as_deref() != Some(root.name.as_str()),
        }],
        mesh_attachment: SourceMeshAttachmentV1 {
            source_node_id: source_node.node_id,
            source_mesh_id,
            source_primitive_id,
            output_mesh_order: 0,
            output_mesh_part: mesh.node_part,
            output_mesh_name: mesh.node_name.clone(),
            output_parent_part: root.part,
        },
        geometry_mapping: SourceGeometryMappingV1 {
            schema_version: 1,
            mapping_profile,
            source_geometry_sha256,
            final_mdl_geometry_sha256: mesh.geometry_sha256.clone(),
            mapping_sha256,
        },
    })
}

pub fn verify_m0_source_topology_binding_v1(
    binding: &SourceTopologyBindingV1,
    profile: &DirectCreatureRuntimeProfileV2,
    source_glb: &[u8],
    final_mdl: &[u8],
) -> Result<(), DirectCreatureContractErrorV1> {
    let recomputed = inspect_m0_source_topology_binding_v1(source_glb, final_mdl, profile)?;
    if binding != &recomputed {
        return Err(contract_error(
            "M2A-SOURCE-TOPOLOGY-BINDING-MISMATCH",
            "sourceTopologyBinding",
            "source identity, topology, selected mapping, output attachment, or geometry mapping differs from independent readback",
        ));
    }
    Ok(())
}

fn summarize_and_require_exact_m0_source(
    ingest: &GlbIngestResult,
) -> Result<(SourceTopologySummaryV1, SourcePrimitiveSelectionV1), DirectCreatureContractErrorV1> {
    let default_scene_id = ingest.ir.default_scene_id.ok_or_else(|| {
        contract_error(
            "M2A-SOURCE-TOPOLOGY-DEFAULT-SCENE",
            "source.defaultSceneId",
            "M0 source requires an explicit default scene",
        )
    })?;
    let default_scene = ingest
        .ir
        .scenes
        .iter()
        .find(|scene| scene.id == default_scene_id)
        .ok_or_else(|| {
            contract_error(
                "M2A-SOURCE-TOPOLOGY-DEFAULT-SCENE",
                "source.defaultSceneId",
                "default scene id does not resolve",
            )
        })?;
    let nodes_by_id = ingest
        .ir
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<BTreeMap<_, _>>();
    let mut traversal = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    fn visit(
        id: u32,
        nodes: &BTreeMap<u32, &crate::glb::IrNode>,
        visiting: &mut BTreeSet<u32>,
        visited: &mut BTreeSet<u32>,
        traversal: &mut Vec<u32>,
    ) -> Result<(), DirectCreatureContractErrorV1> {
        if !visiting.insert(id) {
            return Err(contract_error(
                "M2A-SOURCE-TOPOLOGY-CYCLE",
                "source.nodes",
                "default-scene child order contains a cycle",
            ));
        }
        if visited.insert(id) {
            traversal.push(id);
            let node = nodes.get(&id).ok_or_else(|| {
                contract_error(
                    "M2A-SOURCE-TOPOLOGY-NODE-MISSING",
                    "source.nodes",
                    "scene traversal references a missing node",
                )
            })?;
            for child in &node.child_ids {
                visit(*child, nodes, visiting, visited, traversal)?;
            }
        }
        visiting.remove(&id);
        Ok(())
    }
    for root in &default_scene.root_node_ids {
        visit(
            *root,
            &nodes_by_id,
            &mut visiting,
            &mut visited,
            &mut traversal,
        )?;
    }
    let reachable_meshes = traversal
        .iter()
        .filter_map(|id| nodes_by_id.get(id).and_then(|node| node.mesh_id))
        .collect::<BTreeSet<_>>();
    let traversal_order = traversal
        .iter()
        .enumerate()
        .map(|(order, id)| (*id, order as u32))
        .collect::<BTreeMap<_, _>>();
    let mut name_counts = BTreeMap::<String, u32>::new();
    for name in ingest.ir.nodes.iter().filter_map(|node| node.name.clone()) {
        *name_counts.entry(name).or_default() += 1;
    }
    let summary = SourceTopologySummaryV1 {
        schema_version: 1,
        profile: SOURCE_TOPOLOGY_PROFILE_V1.to_owned(),
        default_scene_id,
        scenes: ingest
            .ir
            .scenes
            .iter()
            .map(|scene| SourceSceneTopologyV1 {
                scene_id: scene.id,
                name: scene.name.clone(),
                ordered_root_node_ids: scene.root_node_ids.clone(),
            })
            .collect(),
        nodes: ingest
            .ir
            .nodes
            .iter()
            .enumerate()
            .map(|(order, node)| SourceNodeTopologyV1 {
                source_order: order as u32,
                node_id: node.id,
                name: node.name.clone(),
                ordered_parent_ids: node.parent_ids.clone(),
                ordered_child_ids: node.child_ids.clone(),
                mesh_id: node.mesh_id,
                skin_id: node.skin_id,
                reachable_from_default_scene: visited.contains(&node.id),
                default_scene_traversal_order: traversal_order.get(&node.id).copied(),
            })
            .collect(),
        meshes: ingest
            .ir
            .meshes
            .iter()
            .enumerate()
            .map(|(order, mesh)| SourceMeshTopologyV1 {
                source_order: order as u32,
                mesh_id: mesh.id,
                name: mesh.name.clone(),
                ordered_primitive_ids: mesh.primitive_ids.clone(),
                reachable_from_default_scene: reachable_meshes.contains(&mesh.id),
            })
            .collect(),
        primitives: ingest
            .ir
            .primitives
            .iter()
            .enumerate()
            .map(|(order, primitive)| {
                let mut attributes = Vec::new();
                if !primitive.positions.is_empty() {
                    attributes.push("POSITION".to_owned());
                }
                if !primitive.normals.is_empty() {
                    attributes.push("NORMAL".to_owned());
                }
                if !primitive.tangents.is_empty() {
                    attributes.push("TANGENT".to_owned());
                }
                if !primitive.uv0.is_empty() {
                    attributes.push("TEXCOORD_0".to_owned());
                }
                if !primitive.joints0.is_empty() {
                    attributes.push("JOINTS_0".to_owned());
                }
                if !primitive.weights0.is_empty() {
                    attributes.push("WEIGHTS_0".to_owned());
                }
                SourcePrimitiveTopologyV1 {
                    source_order: order as u32,
                    primitive_id: primitive.id,
                    source_mesh_id: primitive.source_mesh_id,
                    source_primitive_index: primitive.source_primitive_index,
                    topology: primitive.topology.clone(),
                    material_id: primitive.material_id,
                    position_count: primitive.positions.len() as u32,
                    normal_count: primitive.normals.len() as u32,
                    tangent_count: primitive.tangents.len() as u32,
                    uv0_count: primitive.uv0.len() as u32,
                    joint_lane_count: primitive.joints0.len() as u32,
                    weight_lane_count: primitive.weights0.len() as u32,
                    index_count: primitive.indices.len() as u32,
                    source_was_indexed: primitive.source_was_indexed,
                    attributes,
                }
            })
            .collect(),
        skins: ingest
            .ir
            .skins
            .iter()
            .enumerate()
            .map(|(order, skin)| SourceSkinTopologyV1 {
                source_order: order as u32,
                skin_id: skin.id,
                name: skin.name.clone(),
                skeleton_root_node_id: skin.skeleton_root_node_id,
                ordered_joint_node_ids: skin.joint_node_ids.clone(),
            })
            .collect(),
        ordered_default_scene_traversal: traversal.clone(),
        ignored_node_count: (ingest.ir.nodes.len().saturating_sub(visited.len())) as u32,
        ignored_mesh_count: (ingest
            .ir
            .meshes
            .len()
            .saturating_sub(reachable_meshes.len())) as u32,
        duplicate_non_null_node_names: name_counts
            .into_iter()
            .filter_map(|(name, count)| (count > 1).then_some(name))
            .collect(),
    };

    if summary.scenes.len() != 1
        || default_scene.root_node_ids.len() != 1
        || summary.nodes.len() != 1
        || summary.meshes.len() != 1
        || summary.primitives.len() != 1
        || !summary.skins.is_empty()
        || !ingest.ir.animations.is_empty()
        || summary.ignored_node_count != 0
        || summary.ignored_mesh_count != 0
        || !summary.duplicate_non_null_node_names.is_empty()
    {
        return Err(contract_error(
            "M2A-SOURCE-TOPOLOGY-M0-SHAPE",
            "sourceTopology.summary",
            "M0 requires one scene/root/unskinned mesh/primitive with zero ignored nodes or meshes",
        ));
    }
    let node = summary.nodes[0].clone();
    let mesh = summary.meshes[0].clone();
    let primitive = summary.primitives[0].clone();
    if !node.ordered_parent_ids.is_empty()
        || !node.ordered_child_ids.is_empty()
        || node.mesh_id != Some(mesh.mesh_id)
        || node.skin_id.is_some()
        || mesh.ordered_primitive_ids.as_slice() != [primitive.primitive_id]
        || primitive.source_mesh_id != mesh.mesh_id
        || primitive.joint_lane_count != 0
        || primitive.weight_lane_count != 0
        || primitive.position_count == 0
        || primitive.normal_count != primitive.position_count
        || primitive.uv0_count != primitive.position_count
        || primitive.index_count == 0
        || !primitive.index_count.is_multiple_of(3)
    {
        return Err(contract_error(
            "M2A-SOURCE-TOPOLOGY-M0-ATTRIBUTES",
            "sourceTopology.summary",
            "M0 source mapping forbids invented joints and requires exact indexed POSITION/NORMAL/TEXCOORD_0 geometry",
        ));
    }
    Ok((
        summary,
        SourcePrimitiveSelectionV1 {
            scene_id: default_scene_id,
            node_id: node.node_id,
            mesh_id: mesh.mesh_id,
            primitive_id: primitive.primitive_id,
            source_primitive_index: primitive.source_primitive_index,
        },
    ))
}

fn source_geometry_digest(
    ingest: &GlbIngestResult,
    selected: &SourcePrimitiveSelectionV1,
) -> Result<String, DirectCreatureContractErrorV1> {
    let node = ingest
        .ir
        .nodes
        .iter()
        .find(|node| node.id == selected.node_id)
        .ok_or_else(|| {
            contract_error(
                "M2A-SOURCE-TOPOLOGY-NODE-MISSING",
                "source.nodes",
                "selected node is missing",
            )
        })?;
    let primitive = ingest
        .ir
        .primitives
        .iter()
        .find(|item| item.id == selected.primitive_id)
        .ok_or_else(|| {
            contract_error(
                "M2A-SOURCE-TOPOLOGY-PRIMITIVE-MISSING",
                "source.primitives",
                "selected primitive is missing",
            )
        })?;
    canonical_digest(
        &(
            1u32,
            selected,
            &node.transform,
            &primitive.positions,
            &primitive.normals,
            &primitive.uv0,
            &primitive.indices,
        ),
        "sourceTopology.geometry",
    )
}

fn require_profile_source(
    profile: &DirectCreatureRuntimeProfileV2,
    source_glb: &[u8],
) -> Result<(), DirectCreatureContractErrorV1> {
    require_profile_v2(profile)?;
    let sha256 = sha256_bytes(source_glb);
    if profile.source_byte_length != source_glb.len() as u64
        || profile.source_sha256 != sha256
        || profile.source_canonical_identity != format!("sha256:{sha256}")
    {
        return Err(contract_error(
            "M2A-DIRECT-CREATURE-SOURCE-TRUST-ROOT-MISMATCH",
            "runtimeProfile.source",
            "exact source SHA/length/content identity differs from the caller-owned trust root",
        ));
    }
    Ok(())
}

fn require_profile_v2(
    profile: &DirectCreatureRuntimeProfileV2,
) -> Result<(), DirectCreatureContractErrorV1> {
    if profile.schema_version != 2
        || profile.profile != DIRECT_CREATURE_RUNTIME_PROFILE_V2
        || profile.conversion_profile != M0_CONVERSION_PROFILE_V1
        || profile.conversion_profile_version != 1
        || profile.writer_profile != M0_WRITER_PROFILE_V1
        || profile.writer_profile_version != 1
        || profile.topology_profile != SOURCE_TOPOLOGY_PROFILE_V1
        || profile.topology_profile_version != 1
        || profile.engine_envelope_profile != "DIRECT_CREATURE_ENGINE_ENVELOPE_V1"
        || profile.engine_envelope_profile_version != 1
        || profile.source_informational_path.trim().is_empty()
        || profile.model_resref.trim().is_empty()
        || profile.source_sha256.len() != 64
    {
        return Err(contract_error(
            "M2A-DIRECT-CREATURE-PROFILE-VERSION",
            "runtimeProfile",
            "unknown, partial, or mixed direct-creature runtime profiles are inadmissible",
        ));
    }
    Ok(())
}

fn canonical_digest<T: Serialize>(
    value: &T,
    path: &str,
) -> Result<String, DirectCreatureContractErrorV1> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        contract_error(
            "M2A-DIRECT-CREATURE-CANONICAL-SERIALIZATION",
            path,
            format!("canonical V1/V2 JSON serialization failed: {error}"),
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

fn contract_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> DirectCreatureContractErrorV1 {
    DirectCreatureContractErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

trait EngineNodeKind {
    fn output_kind(&self) -> &str;
}

impl EngineNodeKind for crate::mdl::EngineNodeEnvelopeV1 {
    fn output_kind(&self) -> &str {
        &self.kind
    }
}
