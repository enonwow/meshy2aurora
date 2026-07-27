//! Retargets one static Meshy mesh onto a separate user-provided animated rig.
//!
//! The donor supplies its own hierarchy, bind transforms, reference skin
//! surface, weights and animation channels. The static source contributes only
//! geometry and material bindings. No Aurora reference asset is opened or
//! copied by this route.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::Serialize;

use crate::{
    glb::{GlbLimits, ingest_glb},
    mdl::{
        BinaryMdlArtifactV1, MdlAnimationSetV1, MdlAnimationTrackPathV1, MdlWriterOptionsV1,
        write_binary_mdl_with_animations,
    },
    model_pipeline::materialize_direct_creature_runtime_clips,
    profile_a::{
        AuroraCreatureIrV1, AuroraCreatureSegmentV1, ProfileAConversionOutcomeV1,
        ProfileAOptionsV1, RigSegmentDeformationV1, convert_profile_a,
        convert_profile_a_with_animations_v1, derive_meshy_h1_profile_and_mapping_v1,
    },
};

pub const ANIMATED_DONOR_RETARGET_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimatedDonorRetargetReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub donor_sha256: String,
    pub rig_profile_sha256: String,
    pub model_resource_resref: String,
    pub uniform_scale: f32,
    pub rig_node_count: usize,
    pub skin_segment_count: usize,
    pub active_bone_count: usize,
    pub local_animation_count: usize,
    pub animation_clip_names: Vec<String>,
    pub model_sha256: String,
}

#[derive(Debug)]
pub struct AnimatedDonorRetargetArtifactV1 {
    pub conversion: ProfileAConversionOutcomeV1,
    pub animations: MdlAnimationSetV1,
    pub model: BinaryMdlArtifactV1,
    pub report: AnimatedDonorRetargetReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimatedDonorRetargetErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for AnimatedDonorRetargetErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for AnimatedDonorRetargetErrorV1 {}

/// Converts a static source mesh to the target space and skin weights derived
/// from a separate Meshy H1-style animated GLB, then emits the donor's mapped
/// local animations with the new model's root name.
pub fn retarget_static_mesh_to_animated_donor_v1(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options: &MdlWriterOptionsV1,
) -> Result<AnimatedDonorRetargetArtifactV1, AnimatedDonorRetargetErrorV1> {
    retarget_static_mesh_to_animated_donor(
        source_glb,
        animated_donor_glb,
        writer_options,
        AuroraRootPolicy::RenameWeightedSkeletonRootV1,
    )
}

/// Converts the same inputs as V1 but preserves the donor skeleton root as a
/// weighted joint below a dedicated, identity Aurora Root named after the
/// model resource. This keeps tree ordinal zero out of the active SkinMesh
/// palette without changing donor joint ids, bind transforms or tracks.
pub fn retarget_static_mesh_to_animated_donor_v2(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options: &MdlWriterOptionsV1,
) -> Result<AnimatedDonorRetargetArtifactV1, AnimatedDonorRetargetErrorV1> {
    retarget_static_mesh_to_animated_donor(
        source_glb,
        animated_donor_glb,
        writer_options,
        AuroraRootPolicy::InsertIdentityAuroraRootV2,
    )
}

/// Extends V2 by attaching every SkinMesh directly to the dedicated Aurora
/// Root, matching the observed native creature topology. The old skeleton-root
/// transform is baked into positions, normals and tangents so bind-pose world
/// geometry stays unchanged while the skin node's parent changes.
pub fn retarget_static_mesh_to_animated_donor_v3(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options: &MdlWriterOptionsV1,
) -> Result<AnimatedDonorRetargetArtifactV1, AnimatedDonorRetargetErrorV1> {
    retarget_static_mesh_to_animated_donor(
        source_glb,
        animated_donor_glb,
        writer_options,
        AuroraRootPolicy::InsertIdentityAuroraRootAndReparentSkinV3,
    )
}

/// Extends V3 by removing the donor's constant, uniform animation-scale
/// controller from the preserved weighted skeleton root. Aurora's SkinMesh
/// palette is translation/rotation based, so this route fails closed unless
/// every materialized runtime clip contains the same removable root scale and
/// no other scale controller.
pub fn retarget_static_mesh_to_animated_donor_v4(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options: &MdlWriterOptionsV1,
) -> Result<AnimatedDonorRetargetArtifactV1, AnimatedDonorRetargetErrorV1> {
    retarget_static_mesh_to_animated_donor(
        source_glb,
        animated_donor_glb,
        writer_options,
        AuroraRootPolicy::InsertIdentityAuroraRootReparentSkinAndNormalizeScaleV4,
    )
}

/// Extends V4 by replacing the runtime-unproven SkinMesh with deterministic
/// rigid triangle groups parented to the donor bones. Each complete triangle
/// is assigned to its strongest aggregate bone, transformed into that bone's
/// local bind space and emitted through the proven rigid TriMesh path.
pub fn retarget_static_mesh_to_animated_donor_v5(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options: &MdlWriterOptionsV1,
) -> Result<AnimatedDonorRetargetArtifactV1, AnimatedDonorRetargetErrorV1> {
    retarget_static_mesh_to_animated_donor(
        source_glb,
        animated_donor_glb,
        writer_options,
        AuroraRootPolicy::InsertIdentityAuroraRootRigidTriangleGroupsV5,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuroraRootPolicy {
    RenameWeightedSkeletonRootV1,
    InsertIdentityAuroraRootV2,
    InsertIdentityAuroraRootAndReparentSkinV3,
    InsertIdentityAuroraRootReparentSkinAndNormalizeScaleV4,
    InsertIdentityAuroraRootRigidTriangleGroupsV5,
}

fn retarget_static_mesh_to_animated_donor(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options: &MdlWriterOptionsV1,
    aurora_root_policy: AuroraRootPolicy,
) -> Result<AnimatedDonorRetargetArtifactV1, AnimatedDonorRetargetErrorV1> {
    let source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        retarget_error(
            "M7-DONOR-STATIC-SOURCE-INVALID",
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            format!("{}: {}", error.code, error.message),
        )
    })?;
    let donor = ingest_glb(animated_donor_glb, &GlbLimits::default()).map_err(|error| {
        retarget_error(
            "M7-DONOR-GLB-INVALID",
            error
                .json_path
                .unwrap_or_else(|| "animatedDonorGlb".to_owned()),
            format!("{}: {}", error.code, error.message),
        )
    })?;
    let (rig, mapping) = derive_meshy_h1_profile_and_mapping_v1(&donor).map_err(|error| {
        retarget_error(
            "M7-DONOR-RIG-INVALID",
            format!("animatedDonor.{}", error.path),
            format!("{}: {}", error.code, error.message),
        )
    })?;

    let donor_conversion =
        convert_profile_a_with_animations_v1(&donor, &rig, &ProfileAOptionsV1::default(), &mapping)
            .map_err(|error| {
                retarget_error(
                    "M7-DONOR-ANIMATION-INVALID",
                    format!("animatedDonor.{}", error.path),
                    format!("{}: {}", error.code, error.message),
                )
            })?;
    if !donor_conversion.base.report.conversion_eligible {
        return Err(blocking_conversion_error(
            "M7-DONOR-ANIMATION-INELIGIBLE",
            "animatedDonor.conversion.report.gates",
            &donor_conversion.base,
        ));
    }
    let source_animations = donor_conversion.animations.ok_or_else(|| {
        retarget_error(
            "M7-DONOR-ANIMATION-INELIGIBLE",
            "animatedDonor.animations",
            "eligible animated donor did not produce an animation set",
        )
    })?;
    if source_animations.clips.is_empty() {
        return Err(retarget_error(
            "M7-DONOR-ANIMATION-INELIGIBLE",
            "animatedDonor.animations.clips",
            "animated donor must produce at least one mapped animation clip",
        ));
    }
    let mut animations =
        materialize_direct_creature_runtime_clips(&source_animations).map_err(|error| {
            retarget_error(
                error.code,
                format!("animatedDonor.{}", error.path),
                error.message,
            )
        })?;

    let mut conversion = convert_profile_a(&source, &rig, &ProfileAOptionsV1::default())
        .map_err(|error| retarget_error(error.code, error.path, error.message))?;
    if !conversion.report.conversion_eligible {
        return Err(blocking_conversion_error(
            "M7-DONOR-STATIC-SOURCE-INELIGIBLE",
            "source.conversion.report.gates",
            &conversion,
        ));
    }
    let creature = conversion.creature.as_mut().ok_or_else(|| {
        retarget_error(
            "M7-DONOR-STATIC-SOURCE-INELIGIBLE",
            "source.conversion.creature",
            "eligible static-source conversion did not produce creature IR",
        )
    })?;

    let root_indexes = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    let [root_index] = root_indexes.as_slice() else {
        return Err(retarget_error(
            "M7-DONOR-RIG-INVALID",
            "conversion.creature.nodes",
            "animated donor rig must produce exactly one root",
        ));
    };
    match aurora_root_policy {
        AuroraRootPolicy::RenameWeightedSkeletonRootV1 => {
            creature.nodes[*root_index].name = writer_options.model_resource_resref.clone();
        }
        AuroraRootPolicy::InsertIdentityAuroraRootV2
        | AuroraRootPolicy::InsertIdentityAuroraRootAndReparentSkinV3
        | AuroraRootPolicy::InsertIdentityAuroraRootReparentSkinAndNormalizeScaleV4
        | AuroraRootPolicy::InsertIdentityAuroraRootRigidTriangleGroupsV5 => {
            if creature.nodes.iter().any(|node| {
                node.name
                    .eq_ignore_ascii_case(&writer_options.model_resource_resref)
            }) {
                return Err(retarget_error(
                    "M7-DONOR-AURORA-ROOT-NAME-COLLISION",
                    "writerOptions.modelResourceResref",
                    "dedicated Aurora Root name collides with an existing donor rig node",
                ));
            }
            let aurora_root_id = creature
                .nodes
                .iter()
                .map(|node| node.id)
                .max()
                .unwrap_or_default()
                .checked_add(1)
                .ok_or_else(|| {
                    retarget_error(
                        "M7-DONOR-AURORA-ROOT-ID-OVERFLOW",
                        "conversion.creature.nodes",
                        "cannot allocate a distinct node id for the dedicated Aurora Root",
                    )
                })?;
            let skeleton_root_id = creature.nodes[*root_index].id;
            let skeleton_root_bind = creature.nodes[*root_index].bind_local_matrix;
            creature.nodes[*root_index].parent_id = Some(aurora_root_id);
            creature.nodes.insert(
                0,
                crate::profile_a::AuroraCreatureNodeV1 {
                    id: aurora_root_id,
                    name: writer_options.model_resource_resref.clone(),
                    parent_id: None,
                    bind_local_matrix: [
                        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
                        1.0,
                    ],
                },
            );
            debug_assert_eq!(creature.nodes[1].id, skeleton_root_id);
            if matches!(
                aurora_root_policy,
                AuroraRootPolicy::InsertIdentityAuroraRootAndReparentSkinV3
                    | AuroraRootPolicy::InsertIdentityAuroraRootReparentSkinAndNormalizeScaleV4
                    | AuroraRootPolicy::InsertIdentityAuroraRootRigidTriangleGroupsV5
            ) {
                reparent_skin_segments_to_aurora_root(
                    creature,
                    skeleton_root_id,
                    skeleton_root_bind,
                    aurora_root_id,
                )?;
            }
            if matches!(
                aurora_root_policy,
                AuroraRootPolicy::InsertIdentityAuroraRootReparentSkinAndNormalizeScaleV4
                    | AuroraRootPolicy::InsertIdentityAuroraRootRigidTriangleGroupsV5
            ) {
                remove_constant_skeleton_root_scale_tracks(&mut animations, skeleton_root_id)?;
            }
            if aurora_root_policy == AuroraRootPolicy::InsertIdentityAuroraRootRigidTriangleGroupsV5
            {
                rigidize_skin_triangles_by_dominant_bone(creature, aurora_root_id)?;
            }
        }
    }
    for clip in &mut animations.clips {
        clip.animation_root = writer_options.model_resource_resref.clone();
    }

    let skin_segment_count = creature
        .segments
        .iter()
        .filter(|segment| segment.deformation == RigSegmentDeformationV1::Skin)
        .count();
    let rigid_group_profile =
        aurora_root_policy == AuroraRootPolicy::InsertIdentityAuroraRootRigidTriangleGroupsV5;
    let active_bones = if rigid_group_profile {
        creature
            .segments
            .iter()
            .filter(|segment| segment.deformation == RigSegmentDeformationV1::Rigid)
            .map(|segment| segment.parent_node_id)
            .collect::<BTreeSet<_>>()
    } else {
        creature
            .segments
            .iter()
            .filter(|segment| segment.deformation == RigSegmentDeformationV1::Skin)
            .flat_map(|segment| &segment.weights)
            .flat_map(|weights| weights.bone_node_ids)
            .flatten()
            .collect::<BTreeSet<_>>()
    };
    if (!rigid_group_profile && skin_segment_count == 0)
        || (rigid_group_profile
            && (skin_segment_count != 0
                || creature.segments.is_empty()
                || creature
                    .segments
                    .iter()
                    .any(|segment| segment.deformation != RigSegmentDeformationV1::Rigid)))
        || active_bones.is_empty()
    {
        return Err(retarget_error(
            "M7-DONOR-SKIN-REQUIRED",
            "conversion.creature.segments",
            if rigid_group_profile {
                "V5 requires nonempty rigid triangle groups and zero remaining SkinMesh segments"
            } else {
                "animated-donor output requires weighted Skin geometry"
            },
        ));
    }

    let model = write_binary_mdl_with_animations(creature, &animations, writer_options)
        .map_err(|error| retarget_error(error.code, error.path, error.message))?;
    let uniform_scale = conversion.report.transform.scale.ok_or_else(|| {
        retarget_error(
            "M7-DONOR-STATIC-SOURCE-INELIGIBLE",
            "conversion.report.transform.scale",
            "eligible static-source conversion did not report its uniform scale",
        )
    })?;
    let animation_clip_names = animations
        .clips
        .iter()
        .map(|clip| clip.name.clone())
        .collect::<Vec<_>>();
    let report = AnimatedDonorRetargetReportV1 {
        schema_version: ANIMATED_DONOR_RETARGET_SCHEMA_VERSION,
        source_sha256: conversion.source_sha256.clone(),
        donor_sha256: donor_conversion.base.source_sha256,
        rig_profile_sha256: rig.content_sha256,
        model_resource_resref: writer_options.model_resource_resref.clone(),
        uniform_scale,
        rig_node_count: creature.nodes.len(),
        skin_segment_count,
        active_bone_count: active_bones.len(),
        local_animation_count: animations.clips.len(),
        animation_clip_names,
        model_sha256: model.report.payload_sha256.clone(),
    };

    Ok(AnimatedDonorRetargetArtifactV1 {
        conversion,
        animations,
        model,
        report,
    })
}

fn remove_constant_skeleton_root_scale_tracks(
    animations: &mut MdlAnimationSetV1,
    skeleton_root_id: u32,
) -> Result<(), AnimatedDonorRetargetErrorV1> {
    const SCALE_TOLERANCE: f32 = 0.00001;

    let mut expected_scale = None;
    for (clip_index, clip) in animations.clips.iter_mut().enumerate() {
        let scale_track_indexes = clip
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(track_index, track)| {
                (track.path == MdlAnimationTrackPathV1::Scale).then_some(track_index)
            })
            .collect::<Vec<_>>();
        let [scale_track_index] = scale_track_indexes.as_slice() else {
            let (code, message) = if scale_track_indexes.is_empty() {
                (
                    "M7-DONOR-SKIN-SCALE-MISSING",
                    "V4 requires exactly one removable skeleton-root scale track in every runtime clip",
                )
            } else {
                (
                    "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                    "V4 permits exactly one scale track in every runtime clip",
                )
            };
            return Err(retarget_error(
                code,
                format!("animatedDonor.animations.clips[{clip_index}].tracks"),
                message,
            ));
        };
        let track = &clip.tracks[*scale_track_index];
        if track.target_node_id != skeleton_root_id {
            return Err(retarget_error(
                "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                format!(
                    "animatedDonor.animations.clips[{clip_index}].tracks[{scale_track_index}].targetNodeId"
                ),
                "V4 can remove a scale track only from the preserved weighted skeleton root",
            ));
        }
        if track.values.is_empty() || track.values.len() != track.times_seconds.len() {
            return Err(retarget_error(
                "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                format!(
                    "animatedDonor.animations.clips[{clip_index}].tracks[{scale_track_index}].values"
                ),
                "the removable root-scale track must have one scalar row for every time",
            ));
        }
        let first_scale = track.values[0].first().copied().ok_or_else(|| {
            retarget_error(
                "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                format!(
                    "animatedDonor.animations.clips[{clip_index}].tracks[{scale_track_index}].values[0]"
                ),
                "the removable root-scale track must use one scalar per row",
            )
        })?;
        if !first_scale.is_finite() || first_scale <= 0.0 {
            return Err(retarget_error(
                "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                format!(
                    "animatedDonor.animations.clips[{clip_index}].tracks[{scale_track_index}].values[0]"
                ),
                "the removable root-scale value must be positive and finite",
            ));
        }
        for (row_index, row) in track.values.iter().enumerate() {
            if row.len() != 1
                || !row[0].is_finite()
                || row[0] <= 0.0
                || (row[0] - first_scale).abs() > SCALE_TOLERANCE
            {
                return Err(retarget_error(
                    "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                    format!(
                        "animatedDonor.animations.clips[{clip_index}].tracks[{scale_track_index}].values[{row_index}]"
                    ),
                    "V4 requires one positive, finite, constant scalar in every root-scale row",
                ));
            }
        }
        if expected_scale
            .is_some_and(|expected: f32| (expected - first_scale).abs() > SCALE_TOLERANCE)
        {
            return Err(retarget_error(
                "M7-DONOR-SKIN-SCALE-UNSUPPORTED",
                format!(
                    "animatedDonor.animations.clips[{clip_index}].tracks[{scale_track_index}].values"
                ),
                "all materialized runtime clips must carry the same removable root scale",
            ));
        }
        expected_scale.get_or_insert(first_scale);
        clip.tracks.remove(*scale_track_index);
    }
    Ok(())
}

fn reparent_skin_segments_to_aurora_root(
    creature: &mut AuroraCreatureIrV1,
    skeleton_root_id: u32,
    skeleton_root_bind: [f32; 16],
    aurora_root_id: u32,
) -> Result<(), AnimatedDonorRetargetErrorV1> {
    for (segment_index, segment) in creature.segments.iter_mut().enumerate() {
        if segment.deformation != RigSegmentDeformationV1::Skin {
            continue;
        }
        if segment.parent_node_id != skeleton_root_id {
            return Err(retarget_error(
                "M7-DONOR-SKIN-PARENT-UNSUPPORTED",
                format!("conversion.creature.segments[{segment_index}].parentNodeId"),
                "V3 can preserve bind-pose geometry only when SkinMesh is attached to the single donor skeleton root",
            ));
        }
        for position in &mut segment.positions {
            *position = transform_point(skeleton_root_bind, *position).ok_or_else(|| {
                retarget_error(
                    "M7-DONOR-SKIN-REPARENT-NONFINITE",
                    format!("conversion.creature.segments[{segment_index}].positions"),
                    "baking the old SkinMesh parent transform produced a non-finite position",
                )
            })?;
        }
        for normal in &mut segment.normals {
            *normal = transform_direction(skeleton_root_bind, *normal).ok_or_else(|| {
                retarget_error(
                    "M7-DONOR-SKIN-REPARENT-NONFINITE",
                    format!("conversion.creature.segments[{segment_index}].normals"),
                    "baking the old SkinMesh parent transform produced an invalid normal",
                )
            })?;
        }
        if let Some(tangents) = &mut segment.tangents {
            for tangent in tangents {
                let direction =
                    transform_direction(skeleton_root_bind, [tangent[0], tangent[1], tangent[2]])
                        .ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-SKIN-REPARENT-NONFINITE",
                            format!("conversion.creature.segments[{segment_index}].tangents"),
                            "baking the old SkinMesh parent transform produced an invalid tangent",
                        )
                    })?;
                *tangent = [direction[0], direction[1], direction[2], tangent[3]];
            }
        }
        segment.parent_node_id = aurora_root_id;
    }
    Ok(())
}

#[derive(Debug)]
struct RigidTriangleGroupV5 {
    material_slot: u32,
    parent_node_id: u32,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tangents: Option<Vec<[f32; 4]>>,
    uv0: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

fn rigidize_skin_triangles_by_dominant_bone(
    creature: &mut AuroraCreatureIrV1,
    aurora_root_id: u32,
) -> Result<(), AnimatedDonorRetargetErrorV1> {
    let tree_ordinals = rig_tree_ordinals(creature)?;
    let worlds = rig_world_matrices(creature)?;
    let root_world = *worlds.get(&aurora_root_id).ok_or_else(|| {
        retarget_error(
            "M7-DONOR-RIGID-GROUP-HIERARCHY",
            "conversion.creature.nodes",
            "dedicated Aurora Root has no bind-world transform",
        )
    })?;
    let mut inverse_worlds = BTreeMap::new();
    for node in &creature.nodes {
        inverse_worlds.insert(
            node.id,
            inverse_rigid_transform(
                worlds[&node.id],
                &format!("conversion.creature.nodes[id={}].bindLocalMatrix", node.id),
            )?,
        );
    }

    let original_segments = std::mem::take(&mut creature.segments);
    if original_segments.is_empty()
        || original_segments
            .iter()
            .any(|segment| segment.deformation != RigSegmentDeformationV1::Skin)
    {
        return Err(retarget_error(
            "M7-DONOR-RIGID-GROUP-SOURCE",
            "conversion.creature.segments",
            "V5 accepts only a nonempty weighted SkinMesh source before rigid grouping",
        ));
    }

    let mut groups = BTreeMap::<(usize, usize), RigidTriangleGroupV5>::new();
    for (segment_index, segment) in original_segments.iter().enumerate() {
        if segment.parent_node_id != aurora_root_id {
            return Err(retarget_error(
                "M7-DONOR-RIGID-GROUP-SOURCE",
                format!("conversion.creature.segments[{segment_index}].parentNodeId"),
                "V5 requires the V4 direct-Aurora-Root SkinMesh input",
            ));
        }
        if segment.indices.len() % 3 != 0
            || segment.positions.len() != segment.normals.len()
            || segment.positions.len() != segment.uv0.len()
            || segment.positions.len() != segment.weights.len()
            || segment
                .tangents
                .as_ref()
                .is_some_and(|tangents| tangents.len() != segment.positions.len())
        {
            return Err(retarget_error(
                "M7-DONOR-RIGID-GROUP-SOURCE",
                format!("conversion.creature.segments[{segment_index}]"),
                "V5 source arrays must form complete weighted indexed triangles",
            ));
        }

        for (triangle_index, triangle) in segment.indices.chunks_exact(3).enumerate() {
            let mut scores = BTreeMap::<usize, (u32, f64)>::new();
            for &source_vertex in triangle {
                let vertex_index = usize::try_from(source_vertex).map_err(|_| {
                    retarget_error(
                        "M7-DONOR-RIGID-GROUP-INDEX",
                        format!(
                            "conversion.creature.segments[{segment_index}].indices[{triangle_index}]"
                        ),
                        "triangle vertex index does not fit this platform",
                    )
                })?;
                let weights = segment.weights.get(vertex_index).ok_or_else(|| {
                    retarget_error(
                        "M7-DONOR-RIGID-GROUP-INDEX",
                        format!(
                            "conversion.creature.segments[{segment_index}].indices[{triangle_index}]"
                        ),
                        "triangle vertex index escapes the weighted source arrays",
                    )
                })?;
                for lane in 0..usize::from(weights.influence_count) {
                    let bone_id = weights.bone_node_ids[lane].ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-RIGID-GROUP-WEIGHT",
                            format!(
                                "conversion.creature.segments[{segment_index}].weights[{vertex_index}]"
                            ),
                            "active weight lane has no donor bone id",
                        )
                    })?;
                    let value = weights.values[lane];
                    if !value.is_finite() || value <= 0.0 {
                        return Err(retarget_error(
                            "M7-DONOR-RIGID-GROUP-WEIGHT",
                            format!(
                                "conversion.creature.segments[{segment_index}].weights[{vertex_index}]"
                            ),
                            "active rigid-group weight must be positive and finite",
                        ));
                    }
                    let ordinal = *tree_ordinals.get(&bone_id).ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-RIGID-GROUP-WEIGHT",
                            format!(
                                "conversion.creature.segments[{segment_index}].weights[{vertex_index}]"
                            ),
                            "active rigid-group weight references a bone outside the output rig",
                        )
                    })?;
                    let entry = scores.entry(ordinal).or_insert((bone_id, 0.0));
                    entry.1 += f64::from(value);
                }
            }
            let (selected_ordinal, (selected_bone_id, _)) = scores
                .into_iter()
                .fold(None, |selected, candidate| match selected {
                    None => Some(candidate),
                    Some(current) if candidate.1.1 > current.1.1 => Some(candidate),
                    Some(current) => Some(current),
                })
                .ok_or_else(|| {
                    retarget_error(
                        "M7-DONOR-RIGID-GROUP-WEIGHT",
                        format!(
                            "conversion.creature.segments[{segment_index}].triangles[{triangle_index}]"
                        ),
                        "triangle has no positive donor-bone influence",
                    )
                })?;
            let inverse_bone_world = inverse_worlds[&selected_bone_id];
            let group = groups
                .entry((segment_index, selected_ordinal))
                .or_insert_with(|| RigidTriangleGroupV5 {
                    material_slot: segment.material_slot,
                    parent_node_id: selected_bone_id,
                    positions: Vec::new(),
                    normals: Vec::new(),
                    tangents: segment.tangents.as_ref().map(|_| Vec::new()),
                    uv0: Vec::new(),
                    indices: Vec::new(),
                });
            if group.positions.len() > usize::from(u16::MAX) - 3 {
                return Err(retarget_error(
                    "M7-DONOR-RIGID-GROUP-VERTEX-LIMIT",
                    format!(
                        "conversion.creature.segments[{segment_index}].triangles[{triangle_index}]"
                    ),
                    "one rigid bone group would exceed the binary u16 vertex-count profile",
                ));
            }
            for &source_vertex in triangle {
                let vertex_index = source_vertex as usize;
                let world_position =
                    transform_point(root_world, segment.positions[vertex_index]).ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-RIGID-GROUP-NONFINITE",
                            format!(
                                "conversion.creature.segments[{segment_index}].positions[{vertex_index}]"
                            ),
                            "source bind-world position is non-finite",
                        )
                    })?;
                let local_position = transform_point(inverse_bone_world, world_position)
                    .ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-RIGID-GROUP-NONFINITE",
                            format!(
                                "conversion.creature.segments[{segment_index}].positions[{vertex_index}]"
                            ),
                            "bone-local rigid position is non-finite",
                        )
                    })?;
                let world_normal = transform_direction(root_world, segment.normals[vertex_index])
                    .ok_or_else(|| {
                    retarget_error(
                        "M7-DONOR-RIGID-GROUP-NONFINITE",
                        format!(
                            "conversion.creature.segments[{segment_index}].normals[{vertex_index}]"
                        ),
                        "source bind-world normal is invalid",
                    )
                })?;
                let local_normal = transform_direction(inverse_bone_world, world_normal)
                    .ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-RIGID-GROUP-NONFINITE",
                            format!(
                                "conversion.creature.segments[{segment_index}].normals[{vertex_index}]"
                            ),
                            "bone-local rigid normal is invalid",
                        )
                    })?;
                let output_index = u32::try_from(group.positions.len()).map_err(|_| {
                    retarget_error(
                        "M7-DONOR-RIGID-GROUP-VERTEX-LIMIT",
                        format!(
                            "conversion.creature.segments[{segment_index}].triangles[{triangle_index}]"
                        ),
                        "rigid group vertex index exceeds u32",
                    )
                })?;
                group.positions.push(local_position);
                group.normals.push(local_normal);
                group.uv0.push(segment.uv0[vertex_index]);
                group.indices.push(output_index);
                if let (Some(source_tangents), Some(output_tangents)) =
                    (&segment.tangents, &mut group.tangents)
                {
                    let source_tangent = source_tangents[vertex_index];
                    let world_tangent = transform_direction(
                        root_world,
                        [source_tangent[0], source_tangent[1], source_tangent[2]],
                    )
                    .ok_or_else(|| {
                        retarget_error(
                            "M7-DONOR-RIGID-GROUP-NONFINITE",
                            format!(
                                "conversion.creature.segments[{segment_index}].tangents[{vertex_index}]"
                            ),
                            "source bind-world tangent is invalid",
                        )
                    })?;
                    let local_tangent = transform_direction(inverse_bone_world, world_tangent)
                        .ok_or_else(|| {
                            retarget_error(
                                "M7-DONOR-RIGID-GROUP-NONFINITE",
                                format!(
                                    "conversion.creature.segments[{segment_index}].tangents[{vertex_index}]"
                                ),
                                "bone-local rigid tangent is invalid",
                            )
                        })?;
                    output_tangents.push([
                        local_tangent[0],
                        local_tangent[1],
                        local_tangent[2],
                        source_tangent[3],
                    ]);
                }
            }
        }
    }

    creature.segments = groups
        .into_values()
        .enumerate()
        .map(|(index, group)| {
            let segment_id = u32::try_from(index + 1).map_err(|_| {
                retarget_error(
                    "M7-DONOR-RIGID-GROUP-ID-LIMIT",
                    "conversion.creature.segments",
                    "rigid group count exceeds u32 segment ids",
                )
            })?;
            Ok(AuroraCreatureSegmentV1 {
                segment_id,
                material_slot: group.material_slot,
                deformation: RigSegmentDeformationV1::Rigid,
                parent_node_id: group.parent_node_id,
                cast_shadow: true,
                positions: group.positions,
                normals: group.normals,
                tangents: group.tangents,
                uv0: group.uv0,
                indices: group.indices,
                face_surface_ids: Vec::new(),
                weights: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, AnimatedDonorRetargetErrorV1>>()?;
    if creature.segments.is_empty() {
        return Err(retarget_error(
            "M7-DONOR-RIGID-GROUP-EMPTY",
            "conversion.creature.segments",
            "V5 produced no rigid triangle groups",
        ));
    }
    Ok(())
}

fn rig_tree_ordinals(
    creature: &AuroraCreatureIrV1,
) -> Result<BTreeMap<u32, usize>, AnimatedDonorRetargetErrorV1> {
    let roots = creature
        .nodes
        .iter()
        .filter(|node| node.parent_id.is_none())
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let [root] = roots.as_slice() else {
        return Err(retarget_error(
            "M7-DONOR-RIGID-GROUP-HIERARCHY",
            "conversion.creature.nodes",
            "rigid grouping requires exactly one rig root",
        ));
    };
    let mut children = BTreeMap::<u32, Vec<u32>>::new();
    for node in &creature.nodes {
        if let Some(parent_id) = node.parent_id {
            children.entry(parent_id).or_default().push(node.id);
        }
    }
    let mut ordinals = BTreeMap::new();
    let mut pending = vec![*root];
    while let Some(node_id) = pending.pop() {
        let ordinal = ordinals.len();
        if ordinals.insert(node_id, ordinal).is_some() {
            return Err(retarget_error(
                "M7-DONOR-RIGID-GROUP-HIERARCHY",
                "conversion.creature.nodes",
                "rig hierarchy contains a cycle or duplicate node id",
            ));
        }
        if let Some(node_children) = children.get(&node_id) {
            pending.extend(node_children.iter().rev().copied());
        }
    }
    if ordinals.len() != creature.nodes.len() {
        return Err(retarget_error(
            "M7-DONOR-RIGID-GROUP-HIERARCHY",
            "conversion.creature.nodes",
            "rig hierarchy is not fully reachable from the dedicated Aurora Root",
        ));
    }
    Ok(ordinals)
}

fn rig_world_matrices(
    creature: &AuroraCreatureIrV1,
) -> Result<BTreeMap<u32, [f32; 16]>, AnimatedDonorRetargetErrorV1> {
    let mut worlds = BTreeMap::new();
    let mut remaining = creature.nodes.iter().collect::<Vec<_>>();
    while !remaining.is_empty() {
        let before = remaining.len();
        remaining.retain(|node| {
            let world = match node.parent_id {
                None => Some(node.bind_local_matrix),
                Some(parent_id) => worlds
                    .get(&parent_id)
                    .copied()
                    .map(|parent| mul_mat4(parent, node.bind_local_matrix)),
            };
            if let Some(world) = world {
                worlds.insert(node.id, world);
                false
            } else {
                true
            }
        });
        if remaining.len() == before {
            return Err(retarget_error(
                "M7-DONOR-RIGID-GROUP-HIERARCHY",
                "conversion.creature.nodes",
                "rig hierarchy contains an absent parent or cycle",
            ));
        }
    }
    Ok(worlds)
}

fn inverse_rigid_transform(
    matrix: [f32; 16],
    path: &str,
) -> Result<[f32; 16], AnimatedDonorRetargetErrorV1> {
    const TOLERANCE: f32 = 0.0001;
    if matrix.iter().any(|value| !value.is_finite())
        || matrix[3].abs() > TOLERANCE
        || matrix[7].abs() > TOLERANCE
        || matrix[11].abs() > TOLERANCE
        || (matrix[15] - 1.0).abs() > TOLERANCE
    {
        return Err(retarget_error(
            "M7-DONOR-RIGID-GROUP-BIND-UNSUPPORTED",
            path,
            "rigid grouping requires a finite affine bind matrix",
        ));
    }
    let columns = [
        [matrix[0], matrix[1], matrix[2]],
        [matrix[4], matrix[5], matrix[6]],
        [matrix[8], matrix[9], matrix[10]],
    ];
    for axis in 0..3 {
        let length = columns[axis].iter().map(|value| value * value).sum::<f32>();
        if (length - 1.0).abs() > TOLERANCE {
            return Err(retarget_error(
                "M7-DONOR-RIGID-GROUP-BIND-UNSUPPORTED",
                path,
                "rigid grouping does not accept scale or shear in donor bind nodes",
            ));
        }
        for other in axis + 1..3 {
            let dot = (0..3)
                .map(|component| columns[axis][component] * columns[other][component])
                .sum::<f32>();
            if dot.abs() > TOLERANCE {
                return Err(retarget_error(
                    "M7-DONOR-RIGID-GROUP-BIND-UNSUPPORTED",
                    path,
                    "rigid grouping does not accept non-orthogonal donor bind nodes",
                ));
            }
        }
    }
    let r00 = matrix[0];
    let r01 = matrix[4];
    let r02 = matrix[8];
    let r10 = matrix[1];
    let r11 = matrix[5];
    let r12 = matrix[9];
    let r20 = matrix[2];
    let r21 = matrix[6];
    let r22 = matrix[10];
    let t = [matrix[12], matrix[13], matrix[14]];
    Ok([
        r00,
        r01,
        r02,
        0.0,
        r10,
        r11,
        r12,
        0.0,
        r20,
        r21,
        r22,
        0.0,
        -(r00 * t[0] + r10 * t[1] + r20 * t[2]),
        -(r01 * t[0] + r11 * t[1] + r21 * t[2]),
        -(r02 * t[0] + r12 * t[1] + r22 * t[2]),
        1.0,
    ])
}

fn mul_mat4(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    output
}

fn transform_point(matrix: [f32; 16], point: [f32; 3]) -> Option<[f32; 3]> {
    let transformed = [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ];
    transformed
        .iter()
        .all(|value| value.is_finite())
        .then_some(transformed)
}

fn transform_direction(matrix: [f32; 16], direction: [f32; 3]) -> Option<[f32; 3]> {
    let transformed = [
        matrix[0] * direction[0] + matrix[4] * direction[1] + matrix[8] * direction[2],
        matrix[1] * direction[0] + matrix[5] * direction[1] + matrix[9] * direction[2],
        matrix[2] * direction[0] + matrix[6] * direction[1] + matrix[10] * direction[2],
    ];
    let length_squared = transformed.iter().map(|value| value * value).sum::<f32>();
    if !length_squared.is_finite() || length_squared <= f32::EPSILON {
        return None;
    }
    let inverse_length = length_squared.sqrt().recip();
    let normalized = transformed.map(|value| value * inverse_length);
    normalized
        .iter()
        .all(|value| value.is_finite())
        .then_some(normalized)
}

fn blocking_conversion_error(
    code: &str,
    path: &str,
    conversion: &ProfileAConversionOutcomeV1,
) -> AnimatedDonorRetargetErrorV1 {
    let blocking = conversion
        .report
        .gates
        .iter()
        .filter(|gate| gate.severity == "BLOCKING")
        .map(|gate| gate.code.as_str())
        .collect::<Vec<_>>();
    retarget_error(
        code,
        path,
        if blocking.is_empty() {
            "Profile A rejected conversion without a blocking gate".to_owned()
        } else {
            format!("Profile A blocking gates: {}", blocking.join(", "))
        },
    )
}

fn retarget_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> AnimatedDonorRetargetErrorV1 {
    AnimatedDonorRetargetErrorV1 {
        schema_version: ANIMATED_DONOR_RETARGET_SCHEMA_VERSION,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
