use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    MdlRuntimeConformanceErrorV1,
    types::{ControllerReport, InspectionReport, NodeReport, Vec3},
};

const DEFORMATION_TOLERANCE: f64 = 1.0e-5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinDeformationVertexSampleV1 {
    pub vertex_index: u32,
    pub bind_world: [f32; 3],
    pub sampled_world: [f32; 3],
    pub displacement: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinDeformationNodeSampleV1 {
    pub node_part: u32,
    pub node_name: String,
    pub vertices: Vec<SkinDeformationVertexSampleV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinDeformationSampleV1 {
    pub schema_version: u32,
    pub profile: String,
    pub clip_name: String,
    pub time_seconds: f32,
    pub skin_count: u32,
    pub vertex_count: u32,
    pub moved_vertex_count: u32,
    pub max_displacement: f32,
    pub skins: Vec<SkinDeformationNodeSampleV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelNodeWorldSampleV1 {
    pub node_name: String,
    pub world_position: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelNodeWorldMatrixSampleV2 {
    pub node_part: u32,
    pub node_name: String,
    pub bind_world_matrix: [f32; 16],
    pub sampled_world_matrix: [f32; 16],
}

/// Evaluates the decoded binary SkinMesh at one animation time.
///
/// This is an offline conformance oracle, not a renderer proof. It consumes
/// only the output of this crate's binary reader and composes the same
/// renderer-facing ingredients emitted by the writer: base/state local
/// controllers, node hierarchy, WXYZ inverse-bind transforms, bone slots and
/// four-lane vertex weights.
pub fn evaluate_skin_deformation_v1(
    report: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
) -> Result<SkinDeformationSampleV1, MdlRuntimeConformanceErrorV1> {
    let clip = report
        .animations
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-CLIP-MISSING",
                "clipName",
                "requested animation clip is absent",
            )
        })?;
    evaluate_deformation_with_clip_v2(report, clip, time_seconds, false)
}

/// Evaluates a target SkinMesh with an animation clip resolved from a separate
/// supermodel inspection. No controller or animation payload is copied into
/// the target model; the read-only clip tree is composed directly with the
/// target's base hierarchy and inverse-bind data.
pub fn evaluate_reference_supermodel_skin_deformation_v1(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
) -> Result<SkinDeformationSampleV1, MdlRuntimeConformanceErrorV1> {
    let clip = supermodel
        .animations
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-CLIP-MISSING",
                "supermodel.animations",
                "requested inherited animation clip is absent from the supermodel",
            )
        })?;
    evaluate_deformation_with_clip_v2(target, clip, time_seconds, false)
}

/// Evaluates every renderer-visible geometry node against a separately
/// inspected supermodel clip. SkinMesh nodes use their inverse binds and
/// weights; rigid Trimesh nodes follow their owning node transform. This makes
/// the offline oracle applicable to retail rigid-part animation consumers such
/// as `c_dog` without copying any retail payload into generated output.
pub fn evaluate_reference_supermodel_render_deformation_v2(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
) -> Result<SkinDeformationSampleV1, MdlRuntimeConformanceErrorV1> {
    let clip = supermodel
        .animations
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            deformation_error(
                "M2A-MDL-RENDER-DEFORMATION-CLIP-MISSING",
                "supermodel.animations",
                "requested inherited animation clip is absent from the supermodel",
            )
        })?;
    evaluate_deformation_with_clip_v2(target, clip, time_seconds, true)
}

/// Evaluates a deterministic set of times while caching the target bind pose,
/// renderer-visible node layout and clip-to-base controller resolution once.
/// The returned samples are byte-for-byte equivalent to repeated V2 calls.
pub fn evaluate_reference_supermodel_render_deformation_samples_v3(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    times_seconds: &[f32],
) -> Result<Vec<SkinDeformationSampleV1>, MdlRuntimeConformanceErrorV1> {
    let clip = supermodel
        .animations
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            deformation_error(
                "M2A-MDL-RENDER-DEFORMATION-CLIP-MISSING",
                "supermodel.animations",
                "requested inherited animation clip is absent from the supermodel",
            )
        })?;
    evaluate_deformation_samples_with_clip_v3(target, clip, times_seconds, true)
}

/// Samples exact named carrier positions while resolving controllers from a
/// separate read-only supermodel. This is paired with visible weighted-surface
/// centroids by the reference-supermodel quality oracle.
pub fn evaluate_reference_supermodel_node_world_positions_v1(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
    node_names: &[String],
) -> Result<Vec<ReferenceSupermodelNodeWorldSampleV1>, MdlRuntimeConformanceErrorV1> {
    let clip = supermodel
        .animations
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            deformation_error(
                "M2A-MDL-NODE-WORLD-CLIP-MISSING",
                "supermodel.animations",
                "requested inherited clip is absent from the exact motion provider",
            )
        })?;
    let base = flatten_tree(&target.node_tree.roots);
    let state = flatten_tree(&clip.node_tree.roots);
    let state_by_base_ordinal = resolve_animation_states_by_base_ordinal(&base, &state)?;
    let worlds = build_worlds(&base, &state_by_base_ordinal, time_seconds)?;
    node_names
        .iter()
        .map(|requested| {
            let matches = base
                .iter()
                .enumerate()
                .filter(|(_, node)| node.name.eq_ignore_ascii_case(requested))
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(deformation_error(
                    "M2A-MDL-NODE-WORLD-NAME-UNRESOLVED",
                    "nodeNames",
                    format!(
                        "requested carrier {requested:?} must resolve exactly once in the target"
                    ),
                ));
            }
            let (ordinal, node) = matches[0];
            let position = transform_point(worlds[ordinal], [0.0, 0.0, 0.0]);
            Ok(ReferenceSupermodelNodeWorldSampleV1 {
                node_name: node.name.clone(),
                world_position: position.map(|value| value as f32),
            })
        })
        .collect()
}

/// Returns exact bind and inherited sampled world matrices for requested
/// output carrier parts. This lets the motion oracle transform a visible
/// cluster probe, so orientation-only controllers are not incorrectly treated
/// as static merely because the joint origin does not translate.
pub fn evaluate_reference_supermodel_node_world_matrices_v2(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
    node_parts: &[u32],
) -> Result<Vec<ReferenceSupermodelNodeWorldMatrixSampleV2>, MdlRuntimeConformanceErrorV1> {
    let clip = supermodel
        .animations
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            deformation_error(
                "M2A-MDL-NODE-WORLD-CLIP-MISSING",
                "supermodel.animations",
                "requested inherited clip is absent from the exact motion provider",
            )
        })?;
    let base = flatten_tree(&target.node_tree.roots);
    let state = flatten_tree(&clip.node_tree.roots);
    let state_by_base_ordinal = resolve_animation_states_by_base_ordinal(&base, &state)?;
    let bind_states = vec![None; base.len()];
    let bind_worlds = build_worlds(&base, &bind_states, 0.0)?;
    let sampled_worlds = build_worlds(&base, &state_by_base_ordinal, time_seconds)?;
    node_parts
        .iter()
        .map(|requested| {
            let matches = base
                .iter()
                .enumerate()
                .filter(|(_, node)| node.number == *requested)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(deformation_error(
                    "M2A-MDL-NODE-WORLD-PART-UNRESOLVED",
                    "nodeParts",
                    format!("requested carrier part {requested} must resolve exactly once"),
                ));
            }
            let (ordinal, node) = matches[0];
            Ok(ReferenceSupermodelNodeWorldMatrixSampleV2 {
                node_part: node.number,
                node_name: node.name.clone(),
                bind_world_matrix: bind_worlds[ordinal].map(|value| value as f32),
                sampled_world_matrix: sampled_worlds[ordinal].map(|value| value as f32),
            })
        })
        .collect()
}

fn evaluate_deformation_with_clip_v2(
    report: &InspectionReport,
    clip: &super::types::AnimationReport,
    time_seconds: f32,
    include_rigid_render_meshes: bool,
) -> Result<SkinDeformationSampleV1, MdlRuntimeConformanceErrorV1> {
    if !time_seconds.is_finite() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-TIME",
            "timeSeconds",
            "sample time must be finite",
        ));
    }
    if time_seconds < 0.0 || time_seconds > clip.length {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-TIME",
            "timeSeconds",
            "sample time must be inside the selected clip",
        ));
    }

    let base = flatten_tree(&report.node_tree.roots);
    if base.is_empty() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-BASE",
            "nodeTree",
            "base model tree is empty",
        ));
    }
    let state = flatten_tree(&clip.node_tree.roots);
    // Retail Creature rigs can legitimately repeat a node name in different
    // branches. Resolve animation state against the already-inspected parent
    // hierarchy instead of imposing a global name-uniqueness rule.
    let state_by_base_ordinal = resolve_animation_states_by_base_ordinal(&base, &state)?;
    let bind_states = vec![None; base.len()];
    let base_worlds = build_worlds(&base, &bind_states, 0.0)?;
    let sampled_worlds = build_worlds(&base, &state_by_base_ordinal, time_seconds)?;

    let mut skins = Vec::new();
    let mut vertex_count = 0u32;
    let mut moved_vertex_count = 0u32;
    let mut max_displacement = 0.0f32;
    for (node_ordinal, node) in base.iter().enumerate() {
        let Some(mesh) = &node.mesh else {
            continue;
        };
        let sample_rigid = node.skin.is_none()
            && include_rigid_render_meshes
            && node.aabb.is_none()
            && mesh.render != 0
            && !mesh.vertices.is_empty();
        if node.skin.is_none() && !sample_rigid {
            continue;
        }
        if node.skin.is_some() {
            require_skin_shape(node, base.len(), mesh.vertex_count)?;
        }
        if node_ordinal >= base_worlds.len() {
            return Err(deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-LAYOUT",
                format!("nodeTree.nodes[{}]", node.number),
                "geometry ordinal is outside the rebuilt base hierarchy",
            ));
        }

        let mut vertices = Vec::with_capacity(mesh.vertex_count);
        for vertex_index in 0..mesh.vertex_count {
            let local = mesh.vertices[vertex_index];
            let (bind_world, sampled_world) = if let Some(skin) = &node.skin {
                let weights = skin.vertex_weights[vertex_index];
                let references = skin.bone_references[vertex_index];
                (
                    deform_vertex(local, weights, references, skin, &base_worlds, node.number)?,
                    deform_vertex(
                        local,
                        weights,
                        references,
                        skin,
                        &sampled_worlds,
                        node.number,
                    )?,
                )
            } else {
                let local = [f64::from(local.x), f64::from(local.y), f64::from(local.z)];
                (
                    transform_point(base_worlds[node_ordinal], local),
                    transform_point(sampled_worlds[node_ordinal], local),
                )
            };
            let displacement = distance(bind_world, sampled_world) as f32;
            if displacement > DEFORMATION_TOLERANCE as f32 {
                moved_vertex_count = moved_vertex_count.checked_add(1).ok_or_else(|| {
                    deformation_error(
                        "M2A-MDL-SKIN-DEFORMATION-OVERFLOW",
                        "movedVertexCount",
                        "moved vertex count overflow",
                    )
                })?;
            }
            max_displacement = max_displacement.max(displacement);
            vertex_count = vertex_count.checked_add(1).ok_or_else(|| {
                deformation_error(
                    "M2A-MDL-SKIN-DEFORMATION-OVERFLOW",
                    "vertexCount",
                    "vertex count overflow",
                )
            })?;
            vertices.push(SkinDeformationVertexSampleV1 {
                vertex_index: vertex_index as u32,
                bind_world: bind_world.map(|value| value as f32),
                sampled_world: sampled_world.map(|value| value as f32),
                displacement,
            });
        }
        skins.push(SkinDeformationNodeSampleV1 {
            node_part: node.number,
            node_name: node.name.clone(),
            vertices,
        });
    }
    if skins.is_empty() {
        return Err(deformation_error(
            if include_rigid_render_meshes {
                "M2A-MDL-RENDER-DEFORMATION-NO-GEOMETRY"
            } else {
                "M2A-MDL-SKIN-DEFORMATION-NO-SKIN"
            },
            "nodeTree",
            if include_rigid_render_meshes {
                "base model contains no renderer-visible SkinMesh or rigid Trimesh node"
            } else {
                "base model contains no SkinMesh node"
            },
        ));
    }

    Ok(SkinDeformationSampleV1 {
        schema_version: 1,
        profile: if include_rigid_render_meshes {
            "M2A_MDL_RENDER_DEFORMATION_SAMPLE_V2"
        } else {
            "M2A_MDL_SKIN_DEFORMATION_SAMPLE_V1"
        }
        .to_owned(),
        clip_name: clip.name.clone(),
        time_seconds,
        skin_count: skins.len() as u32,
        vertex_count,
        moved_vertex_count,
        max_displacement,
        skins,
    })
}

fn evaluate_deformation_samples_with_clip_v3(
    report: &InspectionReport,
    clip: &super::types::AnimationReport,
    times_seconds: &[f32],
    include_rigid_render_meshes: bool,
) -> Result<Vec<SkinDeformationSampleV1>, MdlRuntimeConformanceErrorV1> {
    if times_seconds.is_empty() {
        return Err(deformation_error(
            "M2A-MDL-RENDER-DEFORMATION-TIMES-EMPTY",
            "timesSeconds",
            "at least one deformation sample time is required",
        ));
    }
    for &time_seconds in times_seconds {
        if !time_seconds.is_finite() || time_seconds < 0.0 || time_seconds > clip.length {
            return Err(deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-TIME",
                "timesSeconds",
                "every sample time must be finite and inside the selected clip",
            ));
        }
    }
    let base = flatten_tree(&report.node_tree.roots);
    if base.is_empty() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-BASE",
            "nodeTree",
            "base model tree is empty",
        ));
    }
    let state = flatten_tree(&clip.node_tree.roots);
    let state_by_base_ordinal = resolve_animation_states_by_base_ordinal(&base, &state)?;
    let bind_states = vec![None; base.len()];
    let base_worlds = build_worlds(&base, &bind_states, 0.0)?;

    struct PreparedNodeV3<'a> {
        ordinal: usize,
        node: &'a NodeReport,
        bind_vertices: Vec<[f64; 3]>,
    }
    let mut prepared_nodes = Vec::new();
    for (node_ordinal, node) in base.iter().enumerate() {
        let Some(mesh) = &node.mesh else {
            continue;
        };
        let sample_rigid = node.skin.is_none()
            && include_rigid_render_meshes
            && node.aabb.is_none()
            && mesh.render != 0
            && !mesh.vertices.is_empty();
        if node.skin.is_none() && !sample_rigid {
            continue;
        }
        if node.skin.is_some() {
            require_skin_shape(node, base.len(), mesh.vertex_count)?;
        }
        if node_ordinal >= base_worlds.len() {
            return Err(deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-LAYOUT",
                format!("nodeTree.nodes[{}]", node.number),
                "geometry ordinal is outside the rebuilt base hierarchy",
            ));
        }
        let mut bind_vertices = Vec::with_capacity(mesh.vertex_count);
        for vertex_index in 0..mesh.vertex_count {
            let local = mesh.vertices[vertex_index];
            let bind_world = if let Some(skin) = &node.skin {
                deform_vertex(
                    local,
                    skin.vertex_weights[vertex_index],
                    skin.bone_references[vertex_index],
                    skin,
                    &base_worlds,
                    node.number,
                )?
            } else {
                transform_point(
                    base_worlds[node_ordinal],
                    [f64::from(local.x), f64::from(local.y), f64::from(local.z)],
                )
            };
            bind_vertices.push(bind_world);
        }
        prepared_nodes.push(PreparedNodeV3 {
            ordinal: node_ordinal,
            node,
            bind_vertices,
        });
    }
    if prepared_nodes.is_empty() {
        return Err(deformation_error(
            if include_rigid_render_meshes {
                "M2A-MDL-RENDER-DEFORMATION-NO-GEOMETRY"
            } else {
                "M2A-MDL-SKIN-DEFORMATION-NO-SKIN"
            },
            "nodeTree",
            if include_rigid_render_meshes {
                "base model contains no renderer-visible SkinMesh or rigid Trimesh node"
            } else {
                "base model contains no SkinMesh node"
            },
        ));
    }

    times_seconds
        .iter()
        .copied()
        .map(|time_seconds| {
            let sampled_worlds = build_worlds(&base, &state_by_base_ordinal, time_seconds)?;
            let mut skins = Vec::with_capacity(prepared_nodes.len());
            let mut vertex_count = 0u32;
            let mut moved_vertex_count = 0u32;
            let mut max_displacement = 0.0f32;
            for prepared in &prepared_nodes {
                let node = prepared.node;
                let mesh = node.mesh.as_ref().expect("prepared render node has a mesh");
                let mut vertices = Vec::with_capacity(mesh.vertex_count);
                for (vertex_index, &bind_world) in prepared.bind_vertices.iter().enumerate() {
                    let local = mesh.vertices[vertex_index];
                    let sampled_world = if let Some(skin) = &node.skin {
                        deform_vertex(
                            local,
                            skin.vertex_weights[vertex_index],
                            skin.bone_references[vertex_index],
                            skin,
                            &sampled_worlds,
                            node.number,
                        )?
                    } else {
                        transform_point(
                            sampled_worlds[prepared.ordinal],
                            [f64::from(local.x), f64::from(local.y), f64::from(local.z)],
                        )
                    };
                    let displacement = distance(bind_world, sampled_world) as f32;
                    if displacement > DEFORMATION_TOLERANCE as f32 {
                        moved_vertex_count =
                            moved_vertex_count.checked_add(1).ok_or_else(|| {
                                deformation_error(
                                    "M2A-MDL-SKIN-DEFORMATION-OVERFLOW",
                                    "movedVertexCount",
                                    "moved vertex count overflow",
                                )
                            })?;
                    }
                    max_displacement = max_displacement.max(displacement);
                    vertex_count = vertex_count.checked_add(1).ok_or_else(|| {
                        deformation_error(
                            "M2A-MDL-SKIN-DEFORMATION-OVERFLOW",
                            "vertexCount",
                            "vertex count overflow",
                        )
                    })?;
                    vertices.push(SkinDeformationVertexSampleV1 {
                        vertex_index: vertex_index as u32,
                        bind_world: bind_world.map(|value| value as f32),
                        sampled_world: sampled_world.map(|value| value as f32),
                        displacement,
                    });
                }
                skins.push(SkinDeformationNodeSampleV1 {
                    node_part: node.number,
                    node_name: node.name.clone(),
                    vertices,
                });
            }
            Ok(SkinDeformationSampleV1 {
                schema_version: 1,
                profile: if include_rigid_render_meshes {
                    "M2A_MDL_RENDER_DEFORMATION_SAMPLE_V2"
                } else {
                    "M2A_MDL_SKIN_DEFORMATION_SAMPLE_V1"
                }
                .to_owned(),
                clip_name: clip.name.clone(),
                time_seconds,
                skin_count: skins.len() as u32,
                vertex_count,
                moved_vertex_count,
                max_displacement,
                skins,
            })
        })
        .collect()
}

fn flatten_tree(roots: &[NodeReport]) -> Vec<&NodeReport> {
    let mut pending = roots.iter().rev().collect::<Vec<_>>();
    let mut output = Vec::new();
    while let Some(node) = pending.pop() {
        output.push(node);
        pending.extend(node.children.iter().rev());
    }
    output
}

fn resolve_animation_states_by_base_ordinal<'state>(
    base: &[&NodeReport],
    state: &[&'state NodeReport],
) -> Result<Vec<Option<&'state NodeReport>>, MdlRuntimeConformanceErrorV1> {
    let base_ordinal_by_offset = base
        .iter()
        .enumerate()
        .map(|(ordinal, node)| (node.offset, ordinal))
        .collect::<BTreeMap<_, _>>();
    let mut state_ordinals_by_name = BTreeMap::<String, Vec<usize>>::new();
    for (ordinal, node) in state.iter().enumerate() {
        state_ordinals_by_name
            .entry(node.name.to_ascii_lowercase())
            .or_default()
            .push(ordinal);
    }

    let mut used_state = vec![false; state.len()];
    let mut resolved = Vec::<Option<&NodeReport>>::with_capacity(base.len());
    for (base_ordinal, base_node) in base.iter().enumerate() {
        let mut candidates = state_ordinals_by_name
            .get(&base_node.name.to_ascii_lowercase())
            .into_iter()
            .flatten()
            .copied()
            .filter(|ordinal| !used_state[*ordinal])
            .collect::<Vec<_>>();

        // Generated models may rename their model root while preserving the
        // selected supermodel's carrier hierarchy. A unique animation root is
        // the structural root state, regardless of the product resref.
        if candidates.is_empty() && base_node.parent_offset.is_none() {
            candidates = state
                .iter()
                .enumerate()
                .filter(|(ordinal, node)| node.parent_offset.is_none() && !used_state[*ordinal])
                .map(|(ordinal, _)| ordinal)
                .collect();
        }

        let selected = if candidates.len() <= 1 {
            candidates.first().copied()
        } else {
            let expected_parent_state_offset = match base_node.parent_offset {
                Some(parent_offset) => {
                    let parent_ordinal = base_ordinal_by_offset.get(&parent_offset).copied().ok_or_else(
                        || {
                            deformation_error(
                                "M2A-MDL-RENDER-DEFORMATION-BASE-PARENT-MISSING",
                                format!("nodeTree.nodes[{base_ordinal}].parent"),
                                "base node parent offset does not resolve inside the inspected hierarchy",
                            )
                        },
                    )?;
                    resolved[parent_ordinal].map(|node| node.offset)
                }
                None => None,
            };
            let hierarchy_matches = candidates
                .iter()
                .copied()
                .filter(|ordinal| state[*ordinal].parent_offset == expected_parent_state_offset)
                .collect::<Vec<_>>();
            if hierarchy_matches.len() != 1 {
                return Err(deformation_error(
                    "M2A-MDL-RENDER-DEFORMATION-STATE-HIERARCHY-AMBIGUOUS",
                    format!("animation.nodeTree.nodes[{base_ordinal}]"),
                    format!(
                        "animation state {:?} has {} unresolved case-insensitive candidates; parent hierarchy must select exactly one",
                        base_node.name,
                        hierarchy_matches.len()
                    ),
                ));
            }
            hierarchy_matches.first().copied()
        };

        if let Some(ordinal) = selected {
            used_state[ordinal] = true;
            resolved.push(Some(state[ordinal]));
        } else {
            // Owned correction/render children do not require an inherited
            // controller track; their local bind transform remains active.
            resolved.push(None);
        }
    }
    Ok(resolved)
}

fn build_worlds(
    base: &[&NodeReport],
    state_by_base_ordinal: &[Option<&NodeReport>],
    time_seconds: f32,
) -> Result<Vec<Matrix4>, MdlRuntimeConformanceErrorV1> {
    if state_by_base_ordinal.len() != base.len() {
        return Err(deformation_error(
            "M2A-MDL-RENDER-DEFORMATION-STATE-LAYOUT",
            "animation.nodeTree",
            "resolved animation state table must cover the complete base hierarchy",
        ));
    }
    let mut worlds_by_offset = BTreeMap::new();
    let mut worlds = Vec::with_capacity(base.len());
    for (ordinal, node) in base.iter().enumerate() {
        let state = state_by_base_ordinal[ordinal];
        let local = local_transform(node, state, time_seconds)?;
        let world = match node.parent_offset {
            Some(parent) => {
                let parent_world = worlds_by_offset.get(&parent).copied().ok_or_else(|| {
                    deformation_error(
                        "M2A-MDL-SKIN-DEFORMATION-HIERARCHY",
                        format!("nodeTree.nodes[{}].parent", node.number),
                        "parent world transform is unavailable before its child",
                    )
                })?;
                multiply(parent_world, local)
            }
            None => local,
        };
        worlds_by_offset.insert(node.offset, world);
        worlds.push(world);
    }
    Ok(worlds)
}

fn local_transform(
    base: &NodeReport,
    state: Option<&NodeReport>,
    time_seconds: f32,
) -> Result<Matrix4, MdlRuntimeConformanceErrorV1> {
    let mut position = controller_vec3(&base.controllers, 8, 0.0)?.unwrap_or([0.0, 0.0, 0.0]);
    let mut orientation =
        controller_quaternion(&base.controllers, 20, 0.0)?.unwrap_or([0.0, 0.0, 0.0, 1.0]);
    require_unit_scale(&base.controllers, base.number)?;
    if let Some(state) = state {
        if let Some(value) = controller_vec3(&state.controllers, 8, time_seconds)? {
            position = value;
        }
        if let Some(value) = controller_quaternion(&state.controllers, 20, time_seconds)? {
            orientation = value;
        }
        require_unit_scale(&state.controllers, state.number)?;
    }
    Ok(matrix_from_xyzw(
        orientation.map(f64::from),
        position.map(f64::from),
    ))
}

fn controller_vec3(
    controllers: &[ControllerReport],
    controller_type: i32,
    time_seconds: f32,
) -> Result<Option<[f32; 3]>, MdlRuntimeConformanceErrorV1> {
    let Some(controller) = unique_controller(controllers, controller_type)? else {
        return Ok(None);
    };
    let value = sample_controller(controller, time_seconds)?;
    if value.len() != 3 {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers.position",
            "position controller must contain exactly three values",
        ));
    }
    Ok(Some([value[0], value[1], value[2]]))
}

fn controller_quaternion(
    controllers: &[ControllerReport],
    controller_type: i32,
    time_seconds: f32,
) -> Result<Option<[f32; 4]>, MdlRuntimeConformanceErrorV1> {
    let Some(controller) = unique_controller(controllers, controller_type)? else {
        return Ok(None);
    };
    if controller.values.is_empty() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers.orientation",
            "orientation controller has no values",
        ));
    }
    if controller.values.iter().any(|value| value.len() != 4) {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers.orientation",
            "orientation controller must contain exactly four values per row",
        ));
    }
    let value = sample_quaternion(controller, time_seconds)?;
    Ok(Some(value))
}

fn unique_controller(
    controllers: &[ControllerReport],
    controller_type: i32,
) -> Result<Option<&ControllerReport>, MdlRuntimeConformanceErrorV1> {
    let mut matches = controllers
        .iter()
        .filter(|controller| controller.controller_type == controller_type);
    let first = matches.next();
    if matches.next().is_some() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers",
            "node has duplicate deformation controllers of the same type",
        ));
    }
    if first.is_some_and(|controller| !controller.decoded) {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers",
            "required deformation controller is not decoded",
        ));
    }
    Ok(first)
}

fn require_unit_scale(
    controllers: &[ControllerReport],
    node_part: u32,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    let Some(controller) = unique_controller(controllers, 36)? else {
        return Ok(());
    };
    if controller
        .values
        .iter()
        .flatten()
        .any(|value| !value.is_finite() || (*value - 1.0).abs() > 1.0e-5)
    {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-SCALE-UNSUPPORTED",
            format!("nodeTree.nodes[{node_part}].controllers.scale"),
            "offline deformation oracle supports only the writer's unit-scale profile",
        ));
    }
    Ok(())
}

fn sample_controller(
    controller: &ControllerReport,
    time_seconds: f32,
) -> Result<Vec<f32>, MdlRuntimeConformanceErrorV1> {
    if controller.values.is_empty() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers.values",
            "controller has no decoded values",
        ));
    }
    if controller.values.len() == 1 || controller.times.is_empty() {
        return Ok(controller.values[0].clone());
    }
    let (left, right, alpha) = sample_interval(controller, time_seconds)?;
    Ok(controller.values[left]
        .iter()
        .zip(&controller.values[right])
        .map(|(left, right)| left + (right - left) * alpha)
        .collect())
}

fn sample_quaternion(
    controller: &ControllerReport,
    time_seconds: f32,
) -> Result<[f32; 4], MdlRuntimeConformanceErrorV1> {
    if controller.values.len() == 1 || controller.times.is_empty() {
        return normalize_quaternion([
            controller.values[0][0],
            controller.values[0][1],
            controller.values[0][2],
            controller.values[0][3],
        ]);
    }
    let (left, right, alpha) = sample_interval(controller, time_seconds)?;
    let left = normalize_quaternion([
        controller.values[left][0],
        controller.values[left][1],
        controller.values[left][2],
        controller.values[left][3],
    ])?;
    let mut right = normalize_quaternion([
        controller.values[right][0],
        controller.values[right][1],
        controller.values[right][2],
        controller.values[right][3],
    ])?;
    let mut dot = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f32>();
    if dot < 0.0 {
        right = right.map(|value| -value);
        dot = -dot;
    }
    if dot > 0.9995 {
        return normalize_quaternion(std::array::from_fn(|lane| {
            left[lane] + (right[lane] - left[lane]) * alpha
        }));
    }
    let theta = dot.clamp(-1.0, 1.0).acos();
    let sin_theta = theta.sin();
    if sin_theta.abs() <= f32::EPSILON {
        return Ok(left);
    }
    let left_scale = ((1.0 - alpha) * theta).sin() / sin_theta;
    let right_scale = (alpha * theta).sin() / sin_theta;
    normalize_quaternion(std::array::from_fn(|lane| {
        left[lane] * left_scale + right[lane] * right_scale
    }))
}

fn sample_interval(
    controller: &ControllerReport,
    time_seconds: f32,
) -> Result<(usize, usize, f32), MdlRuntimeConformanceErrorV1> {
    if controller.times.len() != controller.values.len() {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers.times",
            "controller time/value row counts differ",
        ));
    }
    if time_seconds <= controller.times[0] {
        return Ok((0, 0, 0.0));
    }
    let last = controller.times.len() - 1;
    if time_seconds >= controller.times[last] {
        return Ok((last, last, 0.0));
    }
    for right in 1..controller.times.len() {
        if time_seconds <= controller.times[right] {
            let left = right - 1;
            let width = controller.times[right] - controller.times[left];
            if !width.is_finite() || width <= 0.0 {
                return Err(deformation_error(
                    "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
                    "controllers.times",
                    "controller times must be finite and strictly increasing",
                ));
            }
            return Ok((left, right, (time_seconds - controller.times[left]) / width));
        }
    }
    Ok((last, last, 0.0))
}

fn normalize_quaternion(value: [f32; 4]) -> Result<[f32; 4], MdlRuntimeConformanceErrorV1> {
    let length = value.iter().map(|value| value * value).sum::<f32>().sqrt();
    if !length.is_finite() || length <= f32::EPSILON {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-CONTROLLER",
            "controllers.orientation",
            "orientation quaternion must be finite and nonzero",
        ));
    }
    Ok(value.map(|value| value / length))
}

fn require_skin_shape(
    node: &NodeReport,
    base_node_count: usize,
    vertex_count: usize,
) -> Result<(), MdlRuntimeConformanceErrorV1> {
    let skin = node.skin.as_ref().expect("caller selected a skin node");
    if skin.node_to_bone_map.len() != base_node_count
        || skin.inverse_bone_rotations_raw.len() != base_node_count
        || skin.inverse_bone_translations.len() != base_node_count
    {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-LAYOUT",
            format!("nodeTree.nodes[{}].skin", node.number),
            "skin bone maps and inverse-bind arrays must cover the complete base tree",
        ));
    }
    if skin.vertex_weights.len() != vertex_count || skin.bone_references.len() != vertex_count {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-LAYOUT",
            format!("nodeTree.nodes[{}].skin.vertices", node.number),
            "skin weights/references must match the mesh vertex count",
        ));
    }
    Ok(())
}

fn deform_vertex(
    local: Vec3,
    weights: [f32; 4],
    references: [u16; 4],
    skin: &super::types::SkinReport,
    worlds: &[Matrix4],
    node_part: u32,
) -> Result<[f64; 3], MdlRuntimeConformanceErrorV1> {
    let mut result = [0.0f64; 3];
    let mut weight_sum = 0.0f64;
    for lane in 0..4 {
        let weight = weights[lane];
        if weight == 0.0 {
            continue;
        }
        if !weight.is_finite() || weight < 0.0 || references[lane] == u16::MAX {
            return Err(deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-WEIGHT",
                format!("nodeTree.nodes[{node_part}].skin.weights"),
                "active skin lane must have a finite nonnegative weight and valid reference",
            ));
        }
        let reference = references[lane] as i16;
        let mut ordinals = skin
            .node_to_bone_map
            .iter()
            .enumerate()
            .filter(|(_, slot)| **slot == reference)
            .map(|(ordinal, _)| ordinal);
        let ordinal = ordinals.next().ok_or_else(|| {
            deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-BONE-SLOT",
                format!("nodeTree.nodes[{node_part}].skin.boneReferences"),
                "active vertex reference has no node-to-bone mapping",
            )
        })?;
        if ordinals.next().is_some() {
            return Err(deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-BONE-SLOT",
                format!("nodeTree.nodes[{node_part}].skin.nodeToBoneMap"),
                "bone slot resolves to more than one base node",
            ));
        }
        let inverse_q = skin.inverse_bone_rotations_raw[ordinal];
        let inverse_t = skin.inverse_bone_translations[ordinal];
        let inverse_q_length = inverse_q
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>()
            .sqrt();
        if !inverse_q_length.is_finite()
            || (inverse_q_length - 1.0).abs() > DEFORMATION_TOLERANCE
            || ![inverse_t.x, inverse_t.y, inverse_t.z]
                .into_iter()
                .all(f32::is_finite)
        {
            return Err(deformation_error(
                "M2A-MDL-SKIN-DEFORMATION-INVERSE-BIND",
                format!("nodeTree.nodes[{node_part}].skin.inverseBind[{ordinal}]"),
                "inverse-bind rotation must be finite/unit and translation must be finite",
            ));
        }
        let inverse = matrix_from_xyzw(
            [
                f64::from(inverse_q[1]),
                f64::from(inverse_q[2]),
                f64::from(inverse_q[3]),
                f64::from(inverse_q[0]),
            ],
            [
                f64::from(inverse_t.x),
                f64::from(inverse_t.y),
                f64::from(inverse_t.z),
            ],
        );
        let point = transform_point(
            multiply(worlds[ordinal], inverse),
            [f64::from(local.x), f64::from(local.y), f64::from(local.z)],
        );
        for axis in 0..3 {
            result[axis] += point[axis] * f64::from(weight);
        }
        weight_sum += f64::from(weight);
    }
    if !weight_sum.is_finite() || (weight_sum - 1.0).abs() > DEFORMATION_TOLERANCE {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-WEIGHT",
            format!("nodeTree.nodes[{node_part}].skin.weights"),
            "active skin weights must sum to one",
        ));
    }
    if !result.into_iter().all(f64::is_finite) {
        return Err(deformation_error(
            "M2A-MDL-SKIN-DEFORMATION-NONFINITE",
            format!("nodeTree.nodes[{node_part}].skin.vertices"),
            "deformed vertex position must be finite",
        ));
    }
    Ok(result)
}

type Matrix4 = [f64; 16];

fn matrix_from_xyzw(q: [f64; 4], p: [f64; 3]) -> Matrix4 {
    let [x, y, z, w] = q;
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;
    [
        1.0 - 2.0 * (yy + zz),
        2.0 * (xy + wz),
        2.0 * (xz - wy),
        0.0,
        2.0 * (xy - wz),
        1.0 - 2.0 * (xx + zz),
        2.0 * (yz + wx),
        0.0,
        2.0 * (xz + wy),
        2.0 * (yz - wx),
        1.0 - 2.0 * (xx + yy),
        0.0,
        p[0],
        p[1],
        p[2],
        1.0,
    ]
}

fn multiply(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|lane| left[lane * 4 + row] * right[column * 4 + lane])
                .sum();
        }
    }
    output
}

fn transform_point(matrix: Matrix4, point: [f64; 3]) -> [f64; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn distance(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right) * (left - right))
        .sum::<f64>()
        .sqrt()
}

fn deformation_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> MdlRuntimeConformanceErrorV1 {
    MdlRuntimeConformanceErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mdl::types::ArrayReport;

    fn node(
        offset: u32,
        number: u32,
        name: &str,
        parent_offset: Option<u32>,
        children: Vec<NodeReport>,
    ) -> NodeReport {
        NodeReport {
            offset,
            number,
            name: name.to_owned(),
            parent_offset,
            inherit_color: 0,
            content_flags: 0,
            unsupported_families: Vec::new(),
            children_header: ArrayReport {
                pointer: 0,
                used: children.len(),
                allocated: children.len(),
            },
            controller_keys_header: ArrayReport {
                pointer: 0,
                used: 0,
                allocated: 0,
            },
            controller_data_header: ArrayReport {
                pointer: 0,
                used: 0,
                allocated: 0,
            },
            controllers: Vec::new(),
            mesh: None,
            skin: None,
            aabb: None,
            children,
        }
    }

    #[test]
    fn animation_state_resolution_disambiguates_duplicate_names_by_parent_hierarchy() {
        let base = vec![node(
            10,
            0,
            "target_root",
            None,
            vec![
                node(
                    20,
                    1,
                    "left",
                    Some(10),
                    vec![node(40, 3, "joint", Some(20), Vec::new())],
                ),
                node(
                    30,
                    2,
                    "right",
                    Some(10),
                    vec![node(50, 4, "joint", Some(30), Vec::new())],
                ),
            ],
        )];
        let state = vec![node(
            110,
            0,
            "source_root",
            None,
            vec![
                node(
                    120,
                    1,
                    "left",
                    Some(110),
                    vec![node(140, 3, "joint", Some(120), Vec::new())],
                ),
                node(
                    130,
                    2,
                    "right",
                    Some(110),
                    vec![node(150, 4, "joint", Some(130), Vec::new())],
                ),
            ],
        )];
        let base = flatten_tree(&base);
        let state = flatten_tree(&state);

        let resolved = resolve_animation_states_by_base_ordinal(&base, &state)
            .expect("hierarchy disambiguates repeated joint names");
        assert_eq!(
            resolved
                .iter()
                .map(|candidate| candidate.map(|node| node.offset))
                .collect::<Vec<_>>(),
            vec![Some(110), Some(120), Some(140), Some(130), Some(150)]
        );
    }
}
