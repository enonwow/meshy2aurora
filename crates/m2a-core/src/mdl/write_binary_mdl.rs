use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use sha2::{Digest, Sha256};

use crate::{
    model_ir::AuroraModelIrV1,
    profile_a::{AuroraCreatureIrV1, AuroraCreatureSegmentV1, RigSegmentDeformationV1},
    walkmesh::{AabbTreeV1, TileNavigationIrV1, validate_tile_navigation_v1},
};

use super::semantic_readback::{
    ExpectedAabbEntry, ExpectedAabbTree, ExpectedAnimation, ExpectedAnimationController,
    ExpectedAnimationNode, ExpectedFace, ExpectedMesh, ExpectedNode, ExpectedReadback,
    ExpectedSkin, semantic_diff,
};
use super::types::{ArrayReport, ParserLimits};
use super::writer_types::{
    BinaryMdlArtifactV1, M4_WRITER_SCHEMA_VERSION, M4SemanticProjectionV1, MdlAabbNodeLayoutV1,
    MdlAnimationClipLayoutV1, MdlAnimationEventV1, MdlAnimationInterpolationV1,
    MdlAnimationNodeLayoutV1, MdlAnimationSetV1, MdlAnimationTrackLayoutV1,
    MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlAnimationWriterReportV1, MdlFormatProfileV1,
    MdlLayoutReportV1, MdlMeshNodeLayoutV1, MdlRigNodeLayoutV1, MdlStateProjectionProfileV1,
    MdlStateProjectionProvenanceV1, MdlWriteError, MdlWriterDeviationV1, MdlWriterOptionsV1,
    MdlWriterReportV1, NWN_EE_BINARY_MDL_EPSILON_V1, NWN_EE_MAX_MESH_INDEX_COUNT_V1,
    is_well_formed_state_projection_provenance_v1,
};
use super::{inspect_binary_mdl, inspect_binary_mdl_with_limits};

const FILE_HEADER_SIZE: usize = 0x0c;
const MODEL_HEADER_SIZE: usize = 0xe8;
const NODE_HEADER_SIZE: usize = 0x70;
const MESH_HEADER_SIZE: usize = 0x270;
const AABB_HEADER_SIZE: usize = 0x274;
const AABB_ENTRY_SIZE: usize = 0x28;
const SKIN_HEADER_SIZE: usize = 0x330;
const SKIN_INLINE_COUNT: usize = 64;
const FACE_SIZE: usize = 0x20;
const CONTROLLER_KEY_SIZE: usize = 0x0c;
const CONTROLLER_KEY_COUNT: usize = 2;
const CONTROLLER_DATA_COUNT: usize = 9;
const ANIMATION_HEADER_SIZE: usize = 0xc4;
const ANIMATION_EVENT_SIZE: usize = 0x24;
const EPSILON: f32 = NWN_EE_BINARY_MDL_EPSILON_V1;

// Versioned product policy for the historical direct-creature culling
// envelope. Mesh-level bounds remain derived from caller-owned source geometry
// below; witness identities and payloads are not embedded in production code.
const DIRECT_CREATURE_MODEL_BOUNDS_MIN: [f32; 3] = [-5.0, -5.0, -1.0];
const DIRECT_CREATURE_MODEL_BOUNDS_MAX: [f32; 3] = [5.0, 5.0, 10.0];
const DIRECT_CREATURE_MODEL_RADIUS: f32 = 7.0;

struct RigPlan {
    source_index: usize,
    id: u32,
    part: u32,
    offset: usize,
    parent_part: Option<u32>,
    emit_bind_controllers: bool,
    children_offsets: Vec<u32>,
    children_array: Option<usize>,
    keys: usize,
    data: usize,
    quaternion: [f32; 4],
    bind_scale: Option<f32>,
}

struct MeshPlan {
    segment_index: usize,
    part: u32,
    offset: usize,
    parent_part: u32,
    bind_controller_keys: Option<usize>,
    bind_controller_data: Option<usize>,
    faces: usize,
    index_count: usize,
    index_offset: usize,
    raw_positions: usize,
    raw_uv0: usize,
    raw_normals: usize,
    raw_colors: Option<usize>,
    raw_indices: usize,
    skin: Option<SkinPlan>,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    radius: f32,
    average: [f32; 3],
}

struct SkinPlan {
    forward_offset: usize,
    q_offset: usize,
    t_offset: usize,
    constants_offset: usize,
    raw_weights: usize,
    raw_refs: usize,
    forward: Vec<i16>,
    inline_reverse: [i16; SKIN_INLINE_COUNT],
    inverse_rotations_wxyz: Vec<[f32; 4]>,
    inverse_translations: Vec<[f32; 3]>,
    vertex_weights: Vec<[f32; 4]>,
    vertex_refs: Vec<[u16; 4]>,
    resolved_ir_ids: Vec<[Option<u32>; 4]>,
}

struct AabbMeshPlan {
    part: u32,
    offset: usize,
    parent_part: u32,
    faces: usize,
    index_count: usize,
    index_offset: usize,
    entry_offsets: Vec<usize>,
    raw_positions: usize,
    raw_uv0: usize,
    raw_normals: usize,
    raw_colors: Option<usize>,
    raw_indices: usize,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    radius: f32,
    average: [f32; 3],
}

#[derive(Clone, Copy)]
enum FacePlaneDegeneracyPolicyV1 {
    LegacyAbsoluteEpsilon,
    ExactFiniteNonCollinear,
}

struct Plan {
    core_length: usize,
    raw_length: usize,
    file_length: usize,
    root_offset: usize,
    rig: Vec<RigPlan>,
    source_to_part: Vec<u32>,
    mesh: Vec<MeshPlan>,
    aabb: Option<AabbMeshPlan>,
    model_bounds_min: [f32; 3],
    model_bounds_max: [f32; 3],
    model_radius: f32,
    mesh_type: u32,
    emit_vertex_colors: bool,
    textures: HashMap<u32, String>,
    deviations: Vec<MdlWriterDeviationV1>,
    animation_pointer_array: Option<usize>,
    animations: Vec<AnimationPlan>,
    face_plane_degeneracy_policy: FacePlaneDegeneracyPolicyV1,
}

struct AnimationPlan {
    clip_index: usize,
    header: usize,
    events: Option<usize>,
    sorted_events: Vec<MdlAnimationEventV1>,
    root: usize,
    nodes: Vec<AnimationNodePlan>,
    track_count: usize,
}

struct AnimationNodePlan {
    kind: AnimationNodeKind,
    offset: usize,
    children_array: Option<usize>,
    children_offsets: Vec<u32>,
    keys: Option<usize>,
    data: Option<usize>,
    tracks: Vec<AnimationTrackPlan>,
    data_values: Vec<f32>,
}

#[derive(Clone, Copy)]
enum AnimationNodeKind {
    Rig(usize),
    RigidMeshPlaceholder(usize),
    MeshDummy(usize),
}

impl AnimationNodeKind {
    fn mesh_index(self) -> Option<usize> {
        match self {
            Self::Rig(_) => None,
            Self::RigidMeshPlaceholder(index) | Self::MeshDummy(index) => Some(index),
        }
    }

    fn uses_zero_geometry_mesh_placeholder(self) -> bool {
        matches!(self, Self::RigidMeshPlaceholder(_))
    }
}

struct AnimationTrackPlan {
    controller_type: i32,
    packed_byte: u8,
    rows: u16,
    time_index: u16,
    data_index: u16,
}

struct PreparedAnimationTrack {
    input_path: String,
    controller_type: i32,
    packed_byte: u8,
    times: Vec<f32>,
    flat_values: Vec<f32>,
}

struct MeshMetrics {
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    radius: f32,
    average: [f32; 3],
}

/// Emits one deterministic structural Profile-A binary MDL with appended MDX
/// and immediately validates it with the project's own reader.
pub fn write_binary_mdl(
    creature: &AuroraModelIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_with_animations(creature, &MdlAnimationSetV1::empty(), options)
}

/// Emits static geometry while preserving every finite, non-collinear face,
/// including valid microtriangles below the legacy absolute epsilon.
pub(crate) fn write_binary_mdl_exact_face_planes_v1(
    model: &AuroraModelIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_with_animations_exact_face_planes_v1(
        model,
        &MdlAnimationSetV1::empty(),
        options,
    )
}

/// Emits a model that inherits animation state from an existing compatible
/// supermodel. The caller remains responsible for providing an independently
/// owned/user-provided rig whose ordered node topology matches that
/// supermodel; this function never opens or copies the reference model.
pub fn write_binary_mdl_with_supermodel(
    creature: &AuroraModelIrV1,
    supermodel_resref: &str,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_internal(
        creature,
        &MdlAnimationSetV1::empty(),
        supermodel_resref,
        options,
        None,
        FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
        None,
    )
}

/// Emits a static model against an existing supermodel while preserving every
/// finite, exactly non-collinear face plane, including valid microtriangles.
pub fn write_binary_mdl_with_supermodel_exact_face_planes_v1(
    creature: &AuroraModelIrV1,
    supermodel_resref: &str,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_internal(
        creature,
        &MdlAnimationSetV1::empty(),
        supermodel_resref,
        options,
        None,
        FacePlaneDegeneracyPolicyV1::ExactFiniteNonCollinear,
        None,
    )
}

pub fn write_binary_mdl_with_animations(
    creature: &AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_internal(
        creature,
        animations,
        "NULL",
        options,
        None,
        FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
        None,
    )
}

pub(crate) fn write_binary_mdl_with_animations_exact_face_planes_v1(
    creature: &AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_internal(
        creature,
        animations,
        "NULL",
        options,
        None,
        FacePlaneDegeneracyPolicyV1::ExactFiniteNonCollinear,
        None,
    )
}

pub fn write_binary_mdl_with_animations_and_supermodel(
    creature: &AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    supermodel_resref: &str,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_internal(
        creature,
        animations,
        supermodel_resref,
        options,
        None,
        FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
        None,
    )
}

/// Emits caller-owned local animation clips while retaining an exact
/// supermodel reference and the strict finite/non-collinear face policy used
/// by dense Meshy render meshes. Local clips may override inherited states;
/// their provenance and admission remain the caller's responsibility.
pub fn write_binary_mdl_with_animations_and_supermodel_exact_face_planes_v1(
    creature: &AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    supermodel_resref: &str,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    write_binary_mdl_internal(
        creature,
        animations,
        supermodel_resref,
        options,
        None,
        FacePlaneDegeneracyPolicyV1::ExactFiniteNonCollinear,
        None,
    )
}

/// Emits the tile profile through the same binary MDL writer used by creature
/// and placeable routes. The extra navigation IR supplies only the AABB mesh;
/// render meshes still come from `AuroraModelIrV1`.
pub fn write_binary_tile_mdl_v1(
    model: &AuroraModelIrV1,
    navigation: &TileNavigationIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    if options.format_profile != MdlFormatProfileV1::TileStaticV1 {
        return Err(error(
            "TILE-MDL-PROFILE-REQUIRED",
            "options.formatProfile",
            "tile entry point requires TileStaticV1",
        ));
    }
    write_binary_mdl_internal(
        model,
        &MdlAnimationSetV1::empty(),
        "NULL",
        options,
        Some(navigation),
        FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
        None,
    )
}

fn write_binary_mdl_internal(
    creature: &AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    supermodel_resref: &str,
    options: &MdlWriterOptionsV1,
    navigation: Option<&TileNavigationIrV1>,
    face_plane_degeneracy_policy: FacePlaneDegeneracyPolicyV1,
    readback_limits: Option<&ParserLimits>,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    if supermodel_resref != "NULL" {
        validate_resref(supermodel_resref, "options.supermodelResref")?;
    }
    let plan = plan_with_face_plane_policy(
        creature,
        animations,
        options,
        navigation,
        face_plane_degeneracy_policy,
    )?;
    let mut core = zeroed(plan.core_length, "layout.coreLength")?;
    let mut raw = zeroed(plan.raw_length, "layout.rawLength")?;
    emit_model(&mut core, creature, options, supermodel_resref, &plan)?;
    emit_nodes(&mut core, creature, &plan)?;
    emit_meshes(&mut core, &mut raw, creature, &plan, options)?;
    if let (Some(navigation), Some(aabb)) = (navigation, plan.aabb.as_ref()) {
        emit_aabb_mesh(&mut core, &mut raw, navigation, aabb, plan.mesh_type)?;
    }
    emit_animations(
        &mut core,
        creature,
        animations,
        &plan,
        options.state_projection_profile,
    )?;

    let mut payload = Vec::new();
    payload.try_reserve_exact(plan.file_length).map_err(|_| {
        error(
            "M4-LAYOUT-OVERFLOW",
            "layout.fileLength",
            "payload allocation failed",
        )
    })?;
    payload.resize(FILE_HEADER_SIZE, 0);
    write_u32(&mut payload, 0, 0)?;
    write_u32(
        &mut payload,
        4,
        as_u32(plan.core_length, "layout.coreLength")?,
    )?;
    write_u32(
        &mut payload,
        8,
        as_u32(plan.raw_length, "layout.rawLength")?,
    )?;
    payload.extend_from_slice(&core);
    payload.extend_from_slice(&raw);
    if payload.len() != plan.file_length {
        return Err(error(
            "M4-LAYOUT-OVERFLOW",
            "layout.fileLength",
            "emitted length differs from the checked plan",
        ));
    }

    let inspection = match readback_limits {
        Some(limits) => inspect_binary_mdl_with_limits(&payload, limits),
        None => inspect_binary_mdl(&payload),
    }
    .map_err(|source| {
        error(
            if animations.clips.is_empty() {
                "M4-READBACK-FAILED"
            } else {
                "M4A-READBACK-FAILED"
            },
            "payload",
            format!("own reader rejected emitted payload: {source}"),
        )
    })?;
    let expected = expected_readback(
        creature,
        animations,
        options,
        supermodel_resref,
        &plan,
        navigation,
    )?;
    let differences = semantic_diff(&expected, &inspection);
    if !differences.is_empty() {
        return Err(error(
            if animations.clips.is_empty() {
                "M4-SEMANTIC-DIFF"
            } else {
                "M4A-SEMANTIC-DIFF"
            },
            "payload",
            format!("own readback differs at {}", differences.join(", ")),
        ));
    }
    let payload_sha256 = hex_sha256(&payload);
    let triangle_count = creature
        .segments
        .iter()
        .try_fold(0usize, |sum, segment| {
            sum.checked_add(segment.indices.len() / 3)
        })
        .ok_or_else(|| {
            error(
                "M4-LAYOUT-OVERFLOW",
                "creature.segments",
                "triangle count overflow",
            )
        })?;
    let rig_nodes = plan
        .rig
        .iter()
        .map(|node| {
            Ok(MdlRigNodeLayoutV1 {
                ir_node_id: node.id,
                part_number: node.part,
                core_offset: as_u32(node.offset, "report.layout.rigNodes.coreOffset")?,
            })
        })
        .collect::<Result<Vec<_>, MdlWriteError>>()?;
    let mesh_nodes = plan
        .mesh
        .iter()
        .map(|node| {
            Ok(MdlMeshNodeLayoutV1 {
                segment_id: creature.segments[node.segment_index].segment_id,
                part_number: node.part,
                core_offset: as_u32(node.offset, "report.layout.meshNodes.coreOffset")?,
            })
        })
        .collect::<Result<Vec<_>, MdlWriteError>>()?;
    let animation_report = animation_writer_report(creature, animations, &plan)?;
    let aabb_node = plan
        .aabb
        .as_ref()
        .map(|node| {
            Ok(MdlAabbNodeLayoutV1 {
                part_number: node.part,
                core_offset: as_u32(node.offset, "report.layout.aabbNode.coreOffset")?,
                root_entry_core_offset: navigation
                    .and_then(|navigation| {
                        node.entry_offsets
                            .get(navigation.aabb_tree.root_index as usize)
                            .copied()
                    })
                    .map(|offset| as_u32(offset, "report.layout.aabbNode.rootEntryCoreOffset"))
                    .transpose()?
                    .unwrap_or(0),
                entry_count: node.entry_offsets.len(),
            })
        })
        .transpose()?;
    let report = MdlWriterReportV1 {
        schema_version: M4_WRITER_SCHEMA_VERSION,
        format_profile: options.format_profile,
        state_projection_profile: options.state_projection_profile,
        state_projection_provenance: options.state_projection_provenance.clone(),
        payload_sha256,
        layout: MdlLayoutReportV1 {
            core_length: plan.core_length,
            raw_length: plan.raw_length,
            file_length: plan.file_length,
            rig_nodes,
            mesh_nodes,
            aabb_node,
        },
        projection: M4SemanticProjectionV1 {
            model_resource_resref: options.model_resource_resref.clone(),
            animation_count: plan.animations.len(),
            rig_node_count: plan.rig.len(),
            mesh_node_count: plan.mesh.len(),
            triangle_count,
        },
        semantic_diff: differences,
        deviations: plan.deviations,
        animation: animation_report,
    };
    Ok(BinaryMdlArtifactV1 {
        payload,
        inspection,
        report,
    })
}

#[cfg(test)]
fn plan(
    creature: &AuroraCreatureIrV1,
    animations: &MdlAnimationSetV1,
    options: &MdlWriterOptionsV1,
    navigation: Option<&TileNavigationIrV1>,
) -> Result<Plan, MdlWriteError> {
    plan_with_face_plane_policy(
        creature,
        animations,
        options,
        navigation,
        FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
    )
}

fn plan_with_face_plane_policy(
    creature: &AuroraCreatureIrV1,
    animations: &MdlAnimationSetV1,
    options: &MdlWriterOptionsV1,
    navigation: Option<&TileNavigationIrV1>,
    face_plane_degeneracy_policy: FacePlaneDegeneracyPolicyV1,
) -> Result<Plan, MdlWriteError> {
    validate_public_contract(creature, options)?;
    match (options.format_profile, navigation) {
        (MdlFormatProfileV1::TileStaticV1, Some(navigation)) => {
            validate_tile_navigation_v1(navigation).map_err(|source| {
                error(
                    &source.code,
                    &source.path,
                    format!("tile navigation validation failed: {}", source.message),
                )
            })?;
            if navigation.model_resref != options.model_resource_resref {
                return Err(error(
                    "TILE-MDL-WOK-RESREF-MISMATCH",
                    "navigation.modelResref",
                    "tile navigation/WOK resref must equal the binary MDL resref",
                ));
            }
            if !animations.clips.is_empty() {
                return Err(error(
                    "TILE-MDL-ANIMATION-UNSUPPORTED",
                    "animations",
                    "TileStaticV1 cannot emit local animations",
                ));
            }
        }
        (MdlFormatProfileV1::TileStaticV1, None) => {
            return Err(error(
                "TILE-MDL-NAVIGATION-MISSING",
                "navigation",
                "TileStaticV1 requires TileNavigationIrV1 for its AABB node",
            ));
        }
        (_, Some(_)) => {
            return Err(error(
                "TILE-MDL-PROFILE-REQUIRED",
                "options.formatProfile",
                "navigation IR is accepted only by TileStaticV1",
            ));
        }
        (_, None) => {}
    }
    if options.format_profile == MdlFormatProfileV1::ItemPartStaticRigidNativeV1
        && !animations.clips.is_empty()
    {
        return Err(error(
            "ITEM-MDL-ANIMATION-UNSUPPORTED",
            "animations",
            "ItemPartStaticRigidNativeV1 cannot emit local animations",
        ));
    }
    let mesh_type = match options.format_profile {
        MdlFormatProfileV1::M4DirectCreatureExtended64V1
        | MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2
        | MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3 => 3,
        MdlFormatProfileV1::M0StaticRigidNativeV1 => 3,
        MdlFormatProfileV1::PlaceableStaticRigidNativeV1 => 3,
        MdlFormatProfileV1::ItemPartStaticRigidNativeV1 => 3,
        MdlFormatProfileV1::TileStaticV1 => 3,
        MdlFormatProfileV1::SourceTopologyPreservingRigidExperimentV1 => 3,
        MdlFormatProfileV1::SourceTopologyPreservingRigidCandidateV1 => 3,
        MdlFormatProfileV1::Legacy17V1 => unreachable!("validated unsupported profile"),
    };
    let emit_vertex_colors = matches!(
        options.format_profile,
        MdlFormatProfileV1::M0StaticRigidNativeV1
            | MdlFormatProfileV1::PlaceableStaticRigidNativeV1
            | MdlFormatProfileV1::ItemPartStaticRigidNativeV1
            | MdlFormatProfileV1::TileStaticV1
            | MdlFormatProfileV1::SourceTopologyPreservingRigidExperimentV1
            | MdlFormatProfileV1::SourceTopologyPreservingRigidCandidateV1
    );
    let textures = validate_materials(creature, options)?;
    let node_count = creature.nodes.len();
    let total_nodes = add(
        add(node_count, creature.segments.len(), "creature.nodes")?,
        usize::from(navigation.is_some()),
        "creature.nodes",
    )?;
    let _ = as_u32(total_nodes, "creature.nodes")?;

    let mut id_to_index = HashMap::with_capacity(node_count);
    let mut output_node_names = HashSet::with_capacity(total_nodes);
    for (index, node) in creature.nodes.iter().enumerate() {
        if id_to_index.insert(node.id, index).is_some() {
            return Err(error(
                "M4-HIERARCHY-INVALID",
                &format!("creature.nodes[{index}].id"),
                "duplicate node id",
            ));
        }
        validate_node_name(&node.name, &format!("creature.nodes[{index}].name"))?;
        if !output_node_names.insert(node.name.to_ascii_lowercase()) {
            return Err(error(
                "M4-NODE-NAME-DUPLICATE",
                &format!("creature.nodes[{index}].name"),
                "output node names must be globally unique after ASCII case-fold",
            ));
        }
    }
    for (index, segment) in creature.segments.iter().enumerate() {
        let generated_name = format!("m2a_seg_{}", segment.segment_id);
        if !output_node_names.insert(generated_name.to_ascii_lowercase()) {
            return Err(error(
                "M4-NODE-NAME-DUPLICATE",
                &format!("creature.segments[{index}].segmentId"),
                "generated mesh node name collides after ASCII case-fold",
            ));
        }
    }
    if let Some(navigation) = navigation
        && !output_node_names.insert(navigation.node_name.to_ascii_lowercase())
    {
        return Err(error(
            "M4-NODE-NAME-DUPLICATE",
            "navigation.nodeName",
            "AABB node name collides after ASCII case-fold",
        ));
    }
    let roots = creature
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.parent_id.is_none())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(error(
            "M4-HIERARCHY-INVALID",
            "creature.nodes",
            "rig must contain exactly one root",
        ));
    }
    let controllerless_root_index = if matches!(
        options.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
            | MdlFormatProfileV1::ItemPartStaticRigidNativeV1
    ) {
        validate_controllerless_identity_root(creature, options, roots[0])?;
        Some(roots[0])
    } else {
        None
    };
    let mut parent_indices = Vec::with_capacity(node_count);
    for (index, node) in creature.nodes.iter().enumerate() {
        let parent = match node.parent_id {
            Some(parent_id) => Some(*id_to_index.get(&parent_id).ok_or_else(|| {
                error(
                    "M4-HIERARCHY-INVALID",
                    &format!("creature.nodes[{index}].parentId"),
                    "parent id does not exist",
                )
            })?),
            None => None,
        };
        if parent == Some(index) {
            return Err(error(
                "M4-HIERARCHY-INVALID",
                &format!("creature.nodes[{index}].parentId"),
                "node cannot parent itself",
            ));
        }
        parent_indices.push(parent);
    }
    validate_acyclic(&parent_indices)?;

    let mut quaternions = Vec::with_capacity(node_count);
    let mut bind_scales = Vec::with_capacity(node_count);
    for (index, node) in creature.nodes.iter().enumerate() {
        let path = format!("creature.nodes[{index}].bindLocalMatrix");
        if options.format_profile == MdlFormatProfileV1::ItemPartStaticRigidNativeV1 {
            let (quaternion, scale) =
                matrix_quaternion_with_uniform_scale(node.bind_local_matrix, &path)?;
            quaternions.push(quaternion);
            bind_scales.push(if (scale - 1.0).abs() > EPSILON {
                Some(scale)
            } else {
                None
            });
        } else {
            quaternions.push(matrix_quaternion(node.bind_local_matrix, &path)?);
            bind_scales.push(None);
        }
    }

    let mut cursor = MODEL_HEADER_SIZE;
    let mut rig_children = vec![Vec::new(); node_count];
    for (child, parent) in parent_indices.iter().enumerate() {
        if let Some(parent) = parent {
            rig_children[*parent].push(child);
        }
    }
    let mut rig_order = Vec::with_capacity(node_count);
    let mut pending = vec![roots[0]];
    while let Some(source_index) = pending.pop() {
        rig_order.push(source_index);
        pending.extend(rig_children[source_index].iter().rev().copied());
    }
    if rig_order.len() != node_count {
        return Err(error(
            "M4-HIERARCHY-INVALID",
            "creature.nodes",
            "rig hierarchy is not fully reachable from its single root",
        ));
    }
    let mut source_to_part = vec![0; node_count];
    for (part, &source_index) in rig_order.iter().enumerate() {
        source_to_part[source_index] = as_u32(part, "layout.partNumber")?;
    }
    let (animation_pointer_array, animation_plans) = plan_animations(
        creature,
        animations,
        roots[0],
        &id_to_index,
        &rig_children,
        &creature.segments,
        options,
        &mut cursor,
    )?;
    let mut rig = Vec::with_capacity(node_count);
    for &source_index in &rig_order {
        let node = &creature.nodes[source_index];
        let offset = take(&mut cursor, NODE_HEADER_SIZE, "layout.rigNodes")?;
        rig.push(RigPlan {
            source_index,
            id: node.id,
            part: source_to_part[source_index],
            offset,
            parent_part: parent_indices[source_index].map(|parent| source_to_part[parent]),
            emit_bind_controllers: controllerless_root_index != Some(source_index),
            children_offsets: Vec::new(),
            children_array: None,
            keys: 0,
            data: 0,
            quaternion: quaternions[source_index],
            bind_scale: bind_scales[source_index],
        });
    }

    let mut segment_ids = HashSet::new();
    let mut mesh = Vec::with_capacity(creature.segments.len());
    let unused_inline_value = if matches!(
        options.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2
            | MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
    ) {
        0
    } else {
        -1
    };
    for (index, segment) in creature.segments.iter().enumerate() {
        validate_segment(segment, index, &id_to_index, face_plane_degeneracy_policy)?;
        if !segment_ids.insert(segment.segment_id) {
            return Err(error(
                "M4-MESH-INVALID",
                &format!("creature.segments[{index}].segmentId"),
                "duplicate segment id",
            ));
        }
        let is_skin = segment.deformation == RigSegmentDeformationV1::Skin;
        let offset = take(
            &mut cursor,
            if is_skin {
                SKIN_HEADER_SIZE
            } else {
                MESH_HEADER_SIZE
            },
            "layout.meshNodes",
        )?;
        let skin = if is_skin {
            let forward_offset = take(
                &mut cursor,
                mul(total_nodes, 2, "layout.skin.forwardMap")?,
                "layout.skin.forwardMap",
            )?;
            cursor = align4(cursor, "layout.skin.forwardMapAlignment")?;
            Some(SkinPlan {
                forward_offset,
                q_offset: 0,
                t_offset: 0,
                constants_offset: 0,
                raw_weights: 0,
                raw_refs: 0,
                forward: Vec::new(),
                inline_reverse: [unused_inline_value; SKIN_INLINE_COUNT],
                inverse_rotations_wxyz: Vec::new(),
                inverse_translations: Vec::new(),
                vertex_weights: Vec::new(),
                vertex_refs: Vec::new(),
                resolved_ir_ids: Vec::new(),
            })
        } else {
            None
        };
        let parent_index = id_to_index[&segment.parent_node_id];
        let metrics = mesh_metrics(&segment.positions)?;
        mesh.push(MeshPlan {
            segment_index: index,
            part: as_u32(
                add(node_count, index, "layout.partNumber")?,
                "layout.partNumber",
            )?,
            offset,
            parent_part: source_to_part[parent_index],
            bind_controller_keys: None,
            bind_controller_data: None,
            faces: 0,
            index_count: 0,
            index_offset: 0,
            raw_positions: 0,
            raw_uv0: 0,
            raw_normals: 0,
            raw_colors: None,
            raw_indices: 0,
            skin,
            bounds_min: metrics.bounds_min,
            bounds_max: metrics.bounds_max,
            radius: metrics.radius,
            average: metrics.average,
        });
    }
    let mut aabb = if let Some(navigation) = navigation {
        let metrics = mesh_metrics(&navigation.vertices)?;
        Some(AabbMeshPlan {
            part: as_u32(
                add(node_count, creature.segments.len(), "layout.partNumber")?,
                "layout.partNumber",
            )?,
            offset: take(&mut cursor, AABB_HEADER_SIZE, "layout.aabbNode")?,
            parent_part: source_to_part[roots[0]],
            faces: 0,
            index_count: 0,
            index_offset: 0,
            entry_offsets: Vec::new(),
            raw_positions: 0,
            raw_uv0: 0,
            raw_normals: 0,
            raw_colors: None,
            raw_indices: 0,
            bounds_min: metrics.bounds_min,
            bounds_max: metrics.bounds_max,
            radius: metrics.radius,
            average: metrics.average,
        })
    } else {
        None
    };

    for child_part in 0..rig.len() {
        if let Some(parent_part) = rig[child_part].parent_part {
            let child_offset = as_u32(rig[child_part].offset, "layout.childOffset")?;
            rig[parent_part as usize]
                .children_offsets
                .push(child_offset);
        }
    }
    for item in &mesh {
        let child_offset = as_u32(item.offset, "layout.childOffset")?;
        rig[item.parent_part as usize]
            .children_offsets
            .push(child_offset);
    }
    if let Some(aabb) = &aabb {
        rig[aabb.parent_part as usize]
            .children_offsets
            .push(as_u32(aabb.offset, "layout.childOffset")?);
    }
    for node in &mut rig {
        if !node.children_offsets.is_empty() {
            cursor = align4(cursor, "layout.children")?;
            node.children_array = Some(take(
                &mut cursor,
                mul(node.children_offsets.len(), 4, "layout.children")?,
                "layout.children",
            )?);
        }
    }
    cursor = align4(cursor, "layout.controllerKeys")?;
    for node in &mut rig {
        if node.emit_bind_controllers {
            let key_count = CONTROLLER_KEY_COUNT + usize::from(node.bind_scale.is_some());
            node.keys = take(
                &mut cursor,
                mul(key_count, CONTROLLER_KEY_SIZE, "layout.controllerKeys")?,
                "layout.controllerKeys",
            )?;
        }
    }
    for item in &mut mesh {
        if item.skin.is_some() {
            item.bind_controller_keys = Some(take(
                &mut cursor,
                mul(
                    CONTROLLER_KEY_COUNT,
                    CONTROLLER_KEY_SIZE,
                    "layout.skinBindControllerKeys",
                )?,
                "layout.skinBindControllerKeys",
            )?);
        }
    }
    for node in &mut rig {
        if node.emit_bind_controllers {
            let data_count = CONTROLLER_DATA_COUNT + 2 * usize::from(node.bind_scale.is_some());
            node.data = take(
                &mut cursor,
                mul(data_count, 4, "layout.controllerData")?,
                "layout.controllerData",
            )?;
        }
    }
    for item in &mut mesh {
        if item.skin.is_some() {
            item.bind_controller_data = Some(take(
                &mut cursor,
                mul(CONTROLLER_DATA_COUNT, 4, "layout.skinBindControllerData")?,
                "layout.skinBindControllerData",
            )?);
        }
    }
    for item in &mut mesh {
        let segment = &creature.segments[item.segment_index];
        item.faces = take(
            &mut cursor,
            mul(segment.indices.len() / 3, FACE_SIZE, "layout.faces")?,
            "layout.faces",
        )?;
    }
    if let (Some(aabb), Some(navigation)) = (aabb.as_mut(), navigation) {
        aabb.faces = take(
            &mut cursor,
            mul(navigation.faces.len(), FACE_SIZE, "layout.aabb.faces")?,
            "layout.aabb.faces",
        )?;
    }
    for item in &mut mesh {
        item.index_count = take(&mut cursor, 4, "layout.indexCounts")?;
    }
    if let Some(aabb) = aabb.as_mut() {
        aabb.index_count = take(&mut cursor, 4, "layout.aabb.indexCount")?;
    }
    for item in &mut mesh {
        item.index_offset = take(&mut cursor, 4, "layout.indexOffsets")?;
    }
    if let Some(aabb) = aabb.as_mut() {
        aabb.index_offset = take(&mut cursor, 4, "layout.aabb.indexOffset")?;
    }
    for item in &mut mesh {
        if let Some(skin) = &mut item.skin {
            skin.q_offset = take(
                &mut cursor,
                mul(total_nodes, 16, "layout.skin.qInverse")?,
                "layout.skin.qInverse",
            )?;
        }
    }
    for item in &mut mesh {
        if let Some(skin) = &mut item.skin {
            skin.t_offset = take(
                &mut cursor,
                mul(total_nodes, 12, "layout.skin.tInverse")?,
                "layout.skin.tInverse",
            )?;
        }
    }
    for item in &mut mesh {
        if let Some(skin) = &mut item.skin {
            skin.constants_offset = take(
                &mut cursor,
                mul(total_nodes, 4, "layout.skin.constants")?,
                "layout.skin.constants",
            )?;
        }
    }
    if let (Some(aabb), Some(navigation)) = (aabb.as_mut(), navigation) {
        aabb.entry_offsets = Vec::with_capacity(navigation.aabb_tree.entries.len());
        for _ in &navigation.aabb_tree.entries {
            aabb.entry_offsets
                .push(take(&mut cursor, AABB_ENTRY_SIZE, "layout.aabb.entries")?);
        }
    }
    let core_length = align4(cursor, "layout.coreLength")?;
    let _ = as_u32(core_length, "layout.coreLength")?;
    validate_skin_signed_fields(&mesh, total_nodes)?;

    let mut raw_cursor = 0usize;
    for item in &mut mesh {
        let segment = &creature.segments[item.segment_index];
        item.raw_positions = take(
            &mut raw_cursor,
            mul(segment.positions.len(), 12, "layout.rawPositions")?,
            "layout.rawPositions",
        )?;
        item.raw_uv0 = take(
            &mut raw_cursor,
            mul(segment.uv0.len(), 8, "layout.rawUv0")?,
            "layout.rawUv0",
        )?;
        item.raw_normals = take(
            &mut raw_cursor,
            mul(segment.normals.len(), 12, "layout.rawNormals")?,
            "layout.rawNormals",
        )?;
        if emit_vertex_colors {
            item.raw_colors = Some(take(
                &mut raw_cursor,
                mul(segment.positions.len(), 4, "layout.rawColors")?,
                "layout.rawColors",
            )?);
        }
        item.raw_indices = take(
            &mut raw_cursor,
            mul(segment.indices.len(), 2, "layout.rawIndices")?,
            "layout.rawIndices",
        )?;
        if let Some(skin) = &mut item.skin {
            raw_cursor = align4(raw_cursor, "layout.skin.rawWeightsAlignment")?;
            skin.raw_weights = take(
                &mut raw_cursor,
                mul(segment.positions.len(), 16, "layout.skin.rawWeights")?,
                "layout.skin.rawWeights",
            )?;
            skin.raw_refs = take(
                &mut raw_cursor,
                mul(segment.positions.len(), 8, "layout.skin.rawRefs")?,
                "layout.skin.rawRefs",
            )?;
        }
        raw_cursor = align4(raw_cursor, "layout.rawAlignment")?;
    }
    if let (Some(aabb), Some(navigation)) = (aabb.as_mut(), navigation) {
        aabb.raw_positions = take(
            &mut raw_cursor,
            mul(navigation.vertices.len(), 12, "layout.aabb.rawPositions")?,
            "layout.aabb.rawPositions",
        )?;
        aabb.raw_uv0 = take(
            &mut raw_cursor,
            mul(navigation.vertices.len(), 8, "layout.aabb.rawUv0")?,
            "layout.aabb.rawUv0",
        )?;
        aabb.raw_normals = take(
            &mut raw_cursor,
            mul(navigation.vertices.len(), 12, "layout.aabb.rawNormals")?,
            "layout.aabb.rawNormals",
        )?;
        if emit_vertex_colors {
            aabb.raw_colors = Some(take(
                &mut raw_cursor,
                mul(navigation.vertices.len(), 4, "layout.aabb.rawColors")?,
                "layout.aabb.rawColors",
            )?);
        }
        aabb.raw_indices = take(
            &mut raw_cursor,
            mul(navigation.faces.len(), 6, "layout.aabb.rawIndices")?,
            "layout.aabb.rawIndices",
        )?;
        raw_cursor = align4(raw_cursor, "layout.aabb.rawAlignment")?;
    }
    let raw_length = raw_cursor;
    let _ = as_u32(raw_length, "layout.rawLength")?;
    for item in &mesh {
        let _ = as_i32(item.raw_positions, "layout.rawPositions")?;
        let _ = as_i32(item.raw_uv0, "layout.rawUv0")?;
        let _ = as_i32(item.raw_normals, "layout.rawNormals")?;
        if let Some(raw_colors) = item.raw_colors {
            let _ = as_i32(raw_colors, "layout.rawColors")?;
        }
        let _ = as_i32(item.raw_indices, "layout.rawIndices")?;
        if let Some(skin) = &item.skin {
            let _ = as_i32(skin.raw_weights, "layout.skin.rawWeights")?;
            let _ = as_i32(skin.raw_refs, "layout.skin.rawRefs")?;
        }
    }
    if let Some(aabb) = &aabb {
        for (value, path) in [
            (aabb.raw_positions, "layout.aabb.rawPositions"),
            (aabb.raw_uv0, "layout.aabb.rawUv0"),
            (aabb.raw_normals, "layout.aabb.rawNormals"),
            (aabb.raw_indices, "layout.aabb.rawIndices"),
        ] {
            let _ = as_i32(value, path)?;
        }
        if let Some(colors) = aabb.raw_colors {
            let _ = as_i32(colors, "layout.aabb.rawColors")?;
        }
    }
    let file_length = add(
        add(FILE_HEADER_SIZE, core_length, "layout.fileLength")?,
        raw_length,
        "layout.fileLength",
    )?;

    let worlds = world_matrices(creature, &parent_indices)?;
    let (part_to_tree_ordinal, ordinal_parts) = tree_ordinals(
        source_to_part[roots[0]] as usize,
        &rig,
        &mesh,
        aabb.as_ref(),
    )?;
    if mesh.iter().any(|item| item.skin.is_some()) {
        let rig_skin_worlds = world_matrices_f64(creature, &parent_indices, &quaternions)?;
        let mut binary_skin_worlds = Vec::with_capacity(total_nodes);
        binary_skin_worlds.extend(rig.iter().map(|node| rig_skin_worlds[node.source_index]));
        binary_skin_worlds.extend(
            mesh.iter()
                .map(|item| rig_skin_worlds[rig[item.parent_part as usize].source_index]),
        );
        let id_to_part = rig
            .iter()
            .map(|node| (node.id, node.part as usize))
            .collect::<HashMap<_, _>>();
        for item in &mut mesh {
            let Some(skin) = &mut item.skin else {
                continue;
            };
            build_skin_plan(
                skin,
                &creature.segments[item.segment_index],
                item.segment_index,
                item.parent_part as usize,
                &id_to_part,
                &part_to_tree_ordinal,
                &ordinal_parts,
                &binary_skin_worlds,
            )?;
        }
    }
    validate_skin_layout(&mesh, total_nodes)?;
    let mut model_min = [f32::INFINITY; 3];
    let mut model_max = [f32::NEG_INFINITY; 3];
    let mut model_radius = 0.0_f32;
    for item in &mesh {
        let segment = &creature.segments[item.segment_index];
        let world = worlds[rig[item.parent_part as usize].source_index];
        for &position in &segment.positions {
            let point = transform_point(world, position);
            if !finite3(point) {
                return Err(error(
                    "M4-MESH-INVALID",
                    &format!("creature.segments[{}].positions", item.segment_index),
                    "world-space geometry overflowed to a non-finite value",
                ));
            }
            for axis in 0..3 {
                model_min[axis] = model_min[axis].min(point[axis]);
                model_max[axis] = model_max[axis].max(point[axis]);
            }
            model_radius = model_radius.max(checked_length3(point).ok_or_else(|| {
                error(
                    "M4-MESH-INVALID",
                    &format!("creature.segments[{}].positions", item.segment_index),
                    "model radius overflowed to a non-finite value",
                )
            })?);
        }
    }
    if let Some(navigation) = navigation {
        let world = worlds[roots[0]];
        for &position in &navigation.vertices {
            let point = transform_point(world, position);
            if !finite3(point) {
                return Err(error(
                    "TILE-MDL-NAVIGATION-INVALID",
                    "navigation.vertices",
                    "world-space navigation geometry overflowed to a non-finite value",
                ));
            }
            for axis in 0..3 {
                model_min[axis] = model_min[axis].min(point[axis]);
                model_max[axis] = model_max[axis].max(point[axis]);
            }
            model_radius = model_radius.max(checked_length3(point).ok_or_else(|| {
                error(
                    "TILE-MDL-NAVIGATION-INVALID",
                    "navigation.vertices",
                    "navigation radius overflowed to a non-finite value",
                )
            })?);
        }
    }

    let mut deviations = vec![
        deviation("M4-RUNTIME-MODEL-FIELDS-OPEN-M6", "model.runtimeFields"),
        deviation("M4-RUNTIME-NODE-FIELDS-OPEN-M6", "nodes.runtimeFields"),
        deviation("M4-RUNTIME-MESH-FIELDS-OPEN-M6", "meshes.runtimeFields"),
        deviation("M4-FACE-TOPOLOGY-DEFAULT-OPEN-M6", "meshes.faces"),
        deviation("M4-MESH-TAIL-DEFAULT-OPEN-M6", "meshes.tailFields"),
    ];
    for (index, segment) in creature.segments.iter().enumerate() {
        if segment.tangents.is_some() {
            deviations.push(MdlWriterDeviationV1 {
                code: "M4-TANGENTS-NOT-EMITTED".to_owned(),
                path: format!("creature.segments[{index}].tangents"),
                message: "tangents have no confirmed NWN1 common-mesh output field; OPEN_M6"
                    .to_owned(),
            });
        }
    }
    if mesh.iter().any(|item| item.skin.is_some()) {
        if !matches!(
            options.format_profile,
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2
                | MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
        ) {
            deviations.push(deviation(
                "M4-SKIN-INLINE-UNUSED-OPEN-M6",
                "skin.inlineReverse.unused",
            ));
        }
        deviations.extend([
            deviation("M4-SKIN-SLOT-BOUNDARY-OPEN-M6", "skin.activeSlots"),
            deviation("M4-SKIN-CONSTANTS-MEANING-OPEN-M6", "skin.boneConstants"),
            deviation("M4-SKIN-WXYZ-DEFORMATION-OPEN-M6", "skin.qInverse"),
            deviation(
                "M4-SKIN-VISUAL-DEFORMATION-OPEN-M6",
                "skin.runtimeDeformation",
            ),
        ]);
    }
    if !animation_plans.is_empty() {
        deviations.extend([
            deviation(
                "M4A-RUNTIME-FIELD-68-OPAQUE-ZERO-OPEN-M6",
                "animations.runtimeField68",
            ),
            deviation(
                "M4A-RUNTIME-ANIM-TREE-PROFILE-OPEN-M6",
                "animations.nodeTrees",
            ),
            deviation(
                "M4A-DECOMP-ANIMROOT-CONSUMER-OPEN-M6",
                "animations.animroot",
            ),
            deviation(
                "M4A-DECOMP-EVENT-NAME-SEMANTICS-OPEN-M6",
                "animations.events.names",
            ),
            deviation(
                "M4A-RUNTIME-STATE-ROUTING-OPEN-M6",
                "animations.stateRouting",
            ),
        ]);
    }

    let (model_bounds_min, model_bounds_max, model_radius) = if matches!(
        options.format_profile,
        MdlFormatProfileV1::PlaceableStaticRigidNativeV1 | MdlFormatProfileV1::TileStaticV1
    ) {
        (model_min, model_max, model_radius)
    } else {
        (
            DIRECT_CREATURE_MODEL_BOUNDS_MIN,
            DIRECT_CREATURE_MODEL_BOUNDS_MAX,
            DIRECT_CREATURE_MODEL_RADIUS,
        )
    };

    Ok(Plan {
        core_length,
        raw_length,
        file_length,
        root_offset: rig[source_to_part[roots[0]] as usize].offset,
        rig,
        source_to_part,
        mesh,
        aabb,
        model_bounds_min,
        model_bounds_max,
        model_radius,
        mesh_type,
        emit_vertex_colors,
        textures,
        deviations,
        animation_pointer_array,
        animations: animation_plans,
        face_plane_degeneracy_policy,
    })
}

// Keep the frozen binary-writer inputs explicit at this boundary: grouping them
// would obscure which independently validated source controls each layout plan.
#[allow(clippy::too_many_arguments)]
fn plan_animations(
    creature: &AuroraCreatureIrV1,
    animation_set: &MdlAnimationSetV1,
    root_index: usize,
    id_to_index: &HashMap<u32, usize>,
    rig_children: &[Vec<usize>],
    segments: &[AuroraCreatureSegmentV1],
    options: &MdlWriterOptionsV1,
    cursor: &mut usize,
) -> Result<(Option<usize>, Vec<AnimationPlan>), MdlWriteError> {
    if animation_set.schema_version != 1 {
        return Err(error(
            "M4A-ANIMATION-SET-SCHEMA-INVALID",
            "animationSet.schemaVersion",
            "MdlAnimationSetV1 must use schema version 1",
        ));
    }
    if animation_set.clips.is_empty() {
        return Ok((None, Vec::new()));
    }

    let parser_limits = ParserLimits::default();
    if creature.nodes.len() > parser_limits.max_nodes {
        return Err(error(
            "M4A-LAYOUT-OVERFLOW",
            "creature.nodes",
            "animation rig node count exceeds the own-reader product guardrail",
        ));
    }
    let mut pending_depths = vec![(root_index, 0usize)];
    while let Some((node, depth)) = pending_depths.pop() {
        if depth > parser_limits.max_depth {
            return Err(error(
                "M4A-LAYOUT-OVERFLOW",
                &format!("creature.nodes[{node}]"),
                "animation rig depth exceeds the own-reader product guardrail",
            ));
        }
        let child_depth = depth.checked_add(1).ok_or_else(|| {
            error(
                "M4A-LAYOUT-OVERFLOW",
                "creature.nodes",
                "animation rig depth overflow",
            )
        })?;
        pending_depths.extend(
            rig_children[node]
                .iter()
                .rev()
                .map(|child| (*child, child_depth)),
        );
    }

    let mut mesh_children = vec![Vec::new(); creature.nodes.len()];
    for (segment_index, segment) in segments.iter().enumerate() {
        let parent_index = *id_to_index.get(&segment.parent_node_id).ok_or_else(|| {
            error(
                "M4-HIERARCHY-INVALID",
                &format!("creature.segments[{segment_index}].parentNodeId"),
                "segment parent node id is absent from the output rig",
            )
        })?;
        match options.state_projection_profile {
            // This family mirrors every base part as a plain dummy. Geometry
            // remains exclusively in the base tree/MDX.
            MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1 => {
                mesh_children[parent_index].push(AnimationNodeKind::MeshDummy(segment_index));
            }
            // This family projects only the ordered rig. Renderable leaves are
            // present in the base tree and deliberately absent from states.
            MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
            | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 => {}
            MdlStateProjectionProfileV1::CepRigidPlaceholderV1 => {
                debug_assert_eq!(segment.deformation, RigSegmentDeformationV1::Rigid);
                mesh_children[parent_index]
                    .push(AnimationNodeKind::RigidMeshPlaceholder(segment_index));
            }
        }
    }

    let pointer_array = animation_take(
        cursor,
        animation_mul(
            animation_set.clips.len(),
            4,
            "layout.animations.pointerArray",
        )?,
        "layout.animations.pointerArray",
    )?;
    let mut names = HashSet::new();
    let mut plans = Vec::with_capacity(animation_set.clips.len());
    for (clip_index, clip) in animation_set.clips.iter().enumerate() {
        let clip_path = format!("animationSet.clips[{clip_index}]");
        validate_animation_string(
            &clip.name,
            63,
            "M4A-ANIMATION-NAME-INVALID",
            &format!("{clip_path}.name"),
        )?;
        let folded = clip.name.to_ascii_lowercase();
        if !names.insert(folded.clone()) {
            return Err(error(
                "M4A-ANIMATION-NAME-INVALID",
                &format!("{clip_path}.name"),
                "clip names must be unique after ASCII case-fold",
            ));
        }
        validate_animation_string(
            &clip.animation_root,
            63,
            "M4A-ANIMROOT-INVALID",
            &format!("{clip_path}.animationRoot"),
        )?;
        if !clip.length_seconds.is_finite() || clip.length_seconds <= 0.0 {
            return Err(error(
                "M4A-CLIP-LENGTH-INVALID",
                &format!("{clip_path}.lengthSeconds"),
                "clip length must be positive and finite",
            ));
        }
        if !clip.transition_seconds.is_finite() || clip.transition_seconds < 0.0 {
            return Err(error(
                "M4A-TRANSITION-INVALID",
                &format!("{clip_path}.transitionSeconds"),
                "transition must be non-negative and finite",
            ));
        }

        let mut sorted_events = clip.events.clone();
        for (event_index, event) in sorted_events.iter().enumerate() {
            let event_path = format!("{clip_path}.events[{event_index}]");
            validate_animation_string(
                &event.name,
                31,
                "M4A-EVENT-NAME-INVALID",
                &format!("{event_path}.name"),
            )?;
            if !event.time_seconds.is_finite()
                || event.time_seconds < 0.0
                || event.time_seconds > clip.length_seconds
            {
                return Err(error(
                    "M4A-EVENT-TIME-INVALID",
                    &format!("{event_path}.timeSeconds"),
                    "event time must be finite and inside the clip",
                ));
            }
        }
        sorted_events.sort_by(|left, right| {
            if left.time_seconds == right.time_seconds {
                Ordering::Equal
            } else {
                left.time_seconds.total_cmp(&right.time_seconds)
            }
        });

        let mut tracks_by_node = (0..creature.nodes.len())
            .map(|_| Vec::<PreparedAnimationTrack>::new())
            .collect::<Vec<_>>();
        let mut seen_tracks = HashSet::new();
        for (track_index, track) in clip.tracks.iter().enumerate() {
            let track_path = format!("{clip_path}.tracks[{track_index}]");
            let node_index = *id_to_index.get(&track.target_node_id).ok_or_else(|| {
                error(
                    "M4A-TRACK-TARGET-MISSING",
                    &format!("{track_path}.targetNodeId"),
                    "animation target node id is absent from the output rig",
                )
            })?;
            if !seen_tracks.insert((track.target_node_id, track.path)) {
                return Err(error(
                    "M4A-TRACK-DUPLICATE",
                    &format!("{track_path}.path"),
                    "a clip may contain only one track per target node and path",
                ));
            }
            let prepared = prepare_animation_track(track, clip.length_seconds, &track_path)?;
            tracks_by_node[node_index].push(prepared);
        }
        for tracks in &mut tracks_by_node {
            tracks.sort_by_key(|track| track.controller_type);
        }
        let header = animation_take(cursor, ANIMATION_HEADER_SIZE, "layout.animations.header")?;
        let events = if sorted_events.is_empty() {
            None
        } else {
            Some(animation_take(
                cursor,
                animation_mul(
                    sorted_events.len(),
                    ANIMATION_EVENT_SIZE,
                    "layout.animations.events",
                )?,
                "layout.animations.events",
            )?)
        };
        let mut nodes = Vec::with_capacity(creature.nodes.len() + segments.len());
        let root = plan_animation_node(
            AnimationNodeKind::Rig(root_index),
            rig_children,
            &mesh_children,
            &mut tracks_by_node,
            &mut nodes,
            cursor,
        )?;
        plans.push(AnimationPlan {
            clip_index,
            header,
            events,
            sorted_events,
            root,
            nodes,
            track_count: clip.tracks.len(),
        });
    }
    Ok((Some(pointer_array), plans))
}

fn prepare_animation_track(
    track: &MdlAnimationTrackV1,
    clip_length: f32,
    path: &str,
) -> Result<PreparedAnimationTrack, MdlWriteError> {
    if track.interpolation != MdlAnimationInterpolationV1::Linear {
        return Err(error(
            "M4A-INTERPOLATION-UNSUPPORTED",
            &format!("{path}.interpolation"),
            "M4A1 supports LINEAR interpolation only",
        ));
    }
    let (controller_type, packed_byte, arity) = match track.path {
        MdlAnimationTrackPathV1::Translation => (8, 3, 3),
        MdlAnimationTrackPathV1::Rotation => (20, 4, 4),
        MdlAnimationTrackPathV1::Scale => (36, 1, 1),
        MdlAnimationTrackPathV1::Weights => {
            return Err(error(
                "M4A-TRACK-PATH-UNSUPPORTED",
                &format!("{path}.path"),
                "M4A1 supports TRANSLATION, ROTATION and uniform SCALE only",
            ));
        }
    };
    if track.times_seconds.is_empty() || track.times_seconds.len() > usize::from(u16::MAX) {
        return Err(error(
            "M4A-CONTROLLER-U16-OVERFLOW",
            &format!("{path}.timesSeconds"),
            "track row count must fit nonzero u16",
        ));
    }
    if track.values.len() != track.times_seconds.len()
        || track.values.iter().any(|row| row.len() != arity)
    {
        return Err(error(
            "M4A-TRACK-ARITY-INVALID",
            &format!("{path}.values"),
            "track values must match time rows and path arity",
        ));
    }
    for (time_index, &time) in track.times_seconds.iter().enumerate() {
        if !time.is_finite() || (time_index > 0 && time <= track.times_seconds[time_index - 1]) {
            return Err(error(
                "M4A-TRACK-TIME-NOT-STRICT",
                &format!("{path}.timesSeconds[{time_index}]"),
                "track times must be finite and strictly increasing",
            ));
        }
        if time < 0.0 || time > clip_length {
            return Err(error(
                "M4A-TRACK-TIME-OOB",
                &format!("{path}.timesSeconds[{time_index}]"),
                "track time must be inside the clip",
            ));
        }
    }
    let mut flat_values = Vec::with_capacity(track.values.len() * arity);
    for (row_index, row) in track.values.iter().enumerate() {
        if row.iter().any(|value| !value.is_finite()) {
            return Err(error(
                "M4A-TRACK-VALUE-NONFINITE",
                &format!("{path}.values[{row_index}]"),
                "track values must be finite",
            ));
        }
        if track.path == MdlAnimationTrackPathV1::Rotation {
            let q = canonical_animation_quaternion(
                [row[0], row[1], row[2], row[3]],
                &format!("{path}.values[{row_index}]"),
            )?;
            flat_values.extend_from_slice(&q);
        } else {
            flat_values.extend_from_slice(row);
        }
    }
    Ok(PreparedAnimationTrack {
        input_path: path.to_owned(),
        controller_type,
        packed_byte,
        times: track.times_seconds.clone(),
        flat_values,
    })
}

fn plan_animation_node(
    kind: AnimationNodeKind,
    rig_children: &[Vec<usize>],
    mesh_children: &[Vec<AnimationNodeKind>],
    tracks_by_node: &mut [Vec<PreparedAnimationTrack>],
    nodes: &mut Vec<AnimationNodePlan>,
    cursor: &mut usize,
) -> Result<usize, MdlWriteError> {
    let node_size = match kind {
        AnimationNodeKind::Rig(_) => NODE_HEADER_SIZE,
        // Recorded direct-creature animation trees retain a zero-geometry
        // trimesh-shaped marker for rigid base meshes, not a generic dummy.
        AnimationNodeKind::RigidMeshPlaceholder(_) => MESH_HEADER_SIZE,
        AnimationNodeKind::MeshDummy(_) => NODE_HEADER_SIZE,
    };
    let offset = animation_take(cursor, node_size, "layout.animations.nodes")?;
    let plan_index = nodes.len();
    nodes.push(AnimationNodePlan {
        kind,
        offset,
        children_array: None,
        children_offsets: Vec::new(),
        keys: None,
        data: None,
        tracks: Vec::new(),
        data_values: Vec::new(),
    });
    let children = match kind {
        AnimationNodeKind::Rig(rig_index) => rig_children[rig_index]
            .iter()
            .copied()
            .map(AnimationNodeKind::Rig)
            .chain(mesh_children[rig_index].iter().copied())
            .collect::<Vec<_>>(),
        AnimationNodeKind::RigidMeshPlaceholder(_) | AnimationNodeKind::MeshDummy(_) => Vec::new(),
    };
    let children_array = if children.is_empty() {
        None
    } else {
        Some(animation_take(
            cursor,
            animation_mul(children.len(), 4, "layout.animations.children")?,
            "layout.animations.children",
        )?)
    };
    let mut children_offsets = Vec::with_capacity(children.len());
    for child in children {
        let child_offset = plan_animation_node(
            child,
            rig_children,
            mesh_children,
            tracks_by_node,
            nodes,
            cursor,
        )?;
        children_offsets.push(animation_as_u32(
            child_offset,
            "layout.animations.childOffset",
        )?);
    }

    let prepared_tracks = match kind {
        AnimationNodeKind::Rig(rig_index) => std::mem::take(&mut tracks_by_node[rig_index]),
        AnimationNodeKind::RigidMeshPlaceholder(_) | AnimationNodeKind::MeshDummy(_) => Vec::new(),
    };
    let mut data_values = Vec::new();
    let mut tracks = Vec::with_capacity(prepared_tracks.len());
    for track in prepared_tracks {
        let rows = track.times.len();
        let time_index = data_values.len();
        let data_index = animation_add(time_index, rows, "layout.animations.controllerData")?;
        let columns = usize::from(track.packed_byte & 0x0f);
        let time_end = animation_add(time_index, rows, "layout.animations.controllerData")?;
        let value_count = animation_mul(rows, columns, "layout.animations.controllerData")?;
        let data_end = animation_add(data_index, value_count, "layout.animations.controllerData")?;
        if rows > usize::from(u16::MAX)
            || time_index > usize::from(u16::MAX)
            || data_index > usize::from(u16::MAX)
            || time_end > usize::from(u16::MAX) + 1
            || data_end > usize::from(u16::MAX) + 1
        {
            return Err(error(
                "M4A-CONTROLLER-U16-OVERFLOW",
                &format!("{}.timesSeconds", track.input_path),
                "controller rows and every evaluated time/data index must fit u16",
            ));
        }
        data_values.extend_from_slice(&track.times);
        data_values.extend_from_slice(&track.flat_values);
        tracks.push(AnimationTrackPlan {
            controller_type: track.controller_type,
            packed_byte: track.packed_byte,
            rows: rows as u16,
            time_index: time_index as u16,
            data_index: data_index as u16,
        });
    }
    let keys = if tracks.is_empty() {
        None
    } else {
        Some(animation_take(
            cursor,
            animation_mul(
                tracks.len(),
                CONTROLLER_KEY_SIZE,
                "layout.animations.controllerKeys",
            )?,
            "layout.animations.controllerKeys",
        )?)
    };
    let data = if data_values.is_empty() {
        None
    } else {
        Some(animation_take(
            cursor,
            animation_mul(data_values.len(), 4, "layout.animations.controllerData")?,
            "layout.animations.controllerData",
        )?)
    };
    nodes[plan_index] = AnimationNodePlan {
        kind,
        offset,
        children_array,
        children_offsets,
        keys,
        data,
        tracks,
        data_values,
    };
    Ok(offset)
}

fn canonical_animation_quaternion(mut q: [f32; 4], path: &str) -> Result<[f32; 4], MdlWriteError> {
    let norm = q
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || (norm - 1.0).abs() > f64::from(EPSILON) {
        return Err(error(
            "M4A-QUATERNION-INVALID",
            path,
            "rotation key must be a unit XYZW quaternion within 1e-5",
        ));
    }
    for value in &mut q {
        *value = (f64::from(*value) / norm) as f32;
    }
    let negate = q[3] < 0.0
        || (q[3] == 0.0
            && q[..3]
                .iter()
                .copied()
                .find(|value| *value != 0.0)
                .is_some_and(|value| value < 0.0));
    if negate {
        for value in &mut q {
            *value = -*value;
        }
    }
    Ok(q)
}

fn validate_animation_string(
    value: &str,
    max_bytes: usize,
    code: &str,
    path: &str,
) -> Result<(), MdlWriteError> {
    if value.is_empty()
        || value.len() > max_bytes
        || !value.is_ascii()
        || value.bytes().any(|byte| byte == 0)
    {
        return Err(error(
            code,
            path,
            format!("value must be a non-empty ASCII C string of at most {max_bytes} bytes"),
        ));
    }
    Ok(())
}

fn animation_add(left: usize, right: usize, path: &str) -> Result<usize, MdlWriteError> {
    left.checked_add(right).ok_or_else(|| {
        error(
            "M4A-LAYOUT-OVERFLOW",
            path,
            "animation layout addition overflow",
        )
    })
}

fn animation_mul(left: usize, right: usize, path: &str) -> Result<usize, MdlWriteError> {
    left.checked_mul(right).ok_or_else(|| {
        error(
            "M4A-LAYOUT-OVERFLOW",
            path,
            "animation layout multiplication overflow",
        )
    })
}

fn animation_take(cursor: &mut usize, length: usize, path: &str) -> Result<usize, MdlWriteError> {
    let start = *cursor;
    *cursor = animation_add(start, length, path)?;
    let _ = animation_as_u32(*cursor, path)?;
    Ok(start)
}

fn animation_as_u32(value: usize, path: &str) -> Result<u32, MdlWriteError> {
    u32::try_from(value).map_err(|_| {
        error(
            "M4A-LAYOUT-OVERFLOW",
            path,
            "animation layout value exceeds u32",
        )
    })
}

fn validate_public_contract(
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<(), MdlWriteError> {
    if options.schema_version != M4_WRITER_SCHEMA_VERSION || creature.schema_version != 1 {
        return Err(error(
            "M4-INVALID-SCHEMA",
            "schemaVersion",
            "writer options and AuroraCreatureIrV1 must use schema version 1",
        ));
    }
    if !matches!(
        options.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64V1
            | MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2
            | MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
            | MdlFormatProfileV1::M0StaticRigidNativeV1
            | MdlFormatProfileV1::PlaceableStaticRigidNativeV1
            | MdlFormatProfileV1::ItemPartStaticRigidNativeV1
            | MdlFormatProfileV1::TileStaticV1
            | MdlFormatProfileV1::SourceTopologyPreservingRigidExperimentV1
            | MdlFormatProfileV1::SourceTopologyPreservingRigidCandidateV1
    ) {
        return Err(error(
            "M4-UNSUPPORTED-PROFILE",
            "options.formatProfile",
            "only the M4 direct-creature and M0 static-rigid profiles are emitted",
        ));
    }
    validate_state_projection_profile(creature, options)?;
    if matches!(
        options.format_profile,
        MdlFormatProfileV1::M0StaticRigidNativeV1
            | MdlFormatProfileV1::PlaceableStaticRigidNativeV1
            | MdlFormatProfileV1::ItemPartStaticRigidNativeV1
            | MdlFormatProfileV1::TileStaticV1
            | MdlFormatProfileV1::SourceTopologyPreservingRigidExperimentV1
            | MdlFormatProfileV1::SourceTopologyPreservingRigidCandidateV1
    ) && creature
        .segments
        .iter()
        .any(|segment| segment.deformation != RigSegmentDeformationV1::Rigid)
    {
        return Err(error(
            "M0-STATIC-RIGID-SKIN-INVALID",
            "creature.segments",
            "M0 static-rigid profile accepts only RIGID segments",
        ));
    }
    validate_resref(
        &options.model_resource_resref,
        "options.modelResourceResref",
    )?;
    if creature.nodes.is_empty() || creature.segments.is_empty() {
        return Err(error(
            "M4-HIERARCHY-INVALID",
            "creature",
            "rig nodes and rigid segments must be non-empty",
        ));
    }
    Ok(())
}

fn validate_controllerless_identity_root(
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
    root_index: usize,
) -> Result<(), MdlWriteError> {
    let root = &creature.nodes[root_index];
    let path = format!("creature.nodes[{root_index}]");
    if root.name != options.model_resource_resref {
        return Err(error(
            "M4-CONTROLLERLESS-ROOT-INVALID",
            &format!("{path}.name"),
            "controllerless root must be named exactly like the model resource resref",
        ));
    }
    let identity = [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ];
    if root
        .bind_local_matrix
        .iter()
        .zip(identity)
        .any(|(actual, expected)| !actual.is_finite() || (*actual - expected).abs() > EPSILON)
    {
        return Err(error(
            "M4-CONTROLLERLESS-ROOT-INVALID",
            &format!("{path}.bindLocalMatrix"),
            "controllerless root must have an exact identity bind transform within writer tolerance",
        ));
    }
    if let Some((segment_index, _)) = creature.segments.iter().enumerate().find(|(_, segment)| {
        segment
            .weights
            .iter()
            .flat_map(|row| row.bone_node_ids)
            .any(|bone| bone == Some(root.id))
    }) {
        return Err(error(
            "M4-CONTROLLERLESS-ROOT-INVALID",
            &format!("creature.segments[{segment_index}].weights"),
            "controllerless root must not be referenced by skin weights",
        ));
    }
    Ok(())
}

fn validate_state_projection_profile(
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<(), MdlWriteError> {
    match options.state_projection_profile {
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
        | MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
        | MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 => {
            if options.state_projection_provenance.is_some() {
                return Err(error(
                    "M4A-STATE-PROJECTION-PROFILE-MIXED",
                    "options.stateProjectionProvenance",
                    "retail state-projection families must not carry CEP placeholder provenance",
                ));
            }
        }
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1 => {
            if creature
                .segments
                .iter()
                .any(|segment| segment.deformation != RigSegmentDeformationV1::Rigid)
            {
                return Err(error(
                    "M4A-STATE-PROJECTION-CEP-SKIN-UNPROVEN",
                    "creature.segments",
                    "CEP_RIGID_PLACEHOLDER_V1 is evidenced only for rigid base meshes",
                ));
            }
            let provenance = options
                .state_projection_provenance
                .as_ref()
                .ok_or_else(|| {
                    error(
                        "M4A-STATE-PROJECTION-PROVENANCE-MISSING",
                        "options.stateProjectionProvenance",
                        "CEP_RIGID_PLACEHOLDER_V1 requires an exact audited R3 witness binding",
                    )
                })?;
            require_cep_r3_projection_provenance(provenance)?;
        }
    }
    Ok(())
}

fn require_cep_r3_projection_provenance(
    provenance: &MdlStateProjectionProvenanceV1,
) -> Result<(), MdlWriteError> {
    if !is_well_formed_state_projection_provenance_v1(provenance) {
        return Err(error(
            "M4A-STATE-PROJECTION-PROVENANCE-MISMATCH",
            "options.stateProjectionProvenance",
            "CEP placeholder output requires explicit well-formed caller-owned provenance; runtime admission must compare it with an independent expected binding",
        ));
    }
    Ok(())
}

fn validate_materials(
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<HashMap<u32, String>, MdlWriteError> {
    let used = creature
        .segments
        .iter()
        .map(|segment| segment.material_slot)
        .collect::<HashSet<_>>();
    let mut textures = HashMap::new();
    for (index, binding) in options
        .diffuse_texture_resref_by_material_slot
        .iter()
        .enumerate()
    {
        validate_resref(
            &binding.resref,
            &format!("options.diffuseTextureResrefByMaterialSlot[{index}].resref"),
        )?;
        if !used.contains(&binding.material_slot)
            || textures
                .insert(binding.material_slot, binding.resref.clone())
                .is_some()
        {
            return Err(error(
                "M4-MATERIAL-BINDING-INVALID",
                "options.diffuseTextureResrefByMaterialSlot",
                "bindings must be unique and exactly cover used material slots",
            ));
        }
    }
    if used.iter().any(|slot| !textures.contains_key(slot)) {
        return Err(error(
            "M4-MATERIAL-BINDING-MISSING",
            "options.diffuseTextureResrefByMaterialSlot",
            "a used material slot has no output texture resref",
        ));
    }
    Ok(textures)
}

fn validate_segment(
    segment: &AuroraCreatureSegmentV1,
    index: usize,
    id_to_index: &HashMap<u32, usize>,
    face_plane_degeneracy_policy: FacePlaneDegeneracyPolicyV1,
) -> Result<(), MdlWriteError> {
    let path = format!("creature.segments[{index}]");
    if !id_to_index.contains_key(&segment.parent_node_id) {
        return Err(error(
            "M4-HIERARCHY-INVALID",
            &format!("{path}.parentNodeId"),
            "segment parent does not exist",
        ));
    }
    if segment.positions.is_empty()
        || segment.positions.len() > usize::from(u16::MAX)
        || segment.indices.len() > NWN_EE_MAX_MESH_INDEX_COUNT_V1
    {
        return Err(error(
            "M4-MESH-LIMIT",
            &path,
            "each mesh is limited to non-empty u16 vertices and at most 65535 index entries (21845 triangles)",
        ));
    }
    if segment.normals.len() != segment.positions.len()
        || segment.uv0.len() != segment.positions.len()
        || segment.indices.is_empty()
        || !segment.indices.len().is_multiple_of(3)
        || (!segment.face_surface_ids.is_empty()
            && segment.face_surface_ids.len() != segment.indices.len() / 3)
        || segment
            .indices
            .iter()
            .any(|&value| value as usize >= segment.positions.len() || value > u32::from(u16::MAX))
        || segment
            .positions
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        || segment
            .normals
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        || segment.uv0.iter().flatten().any(|value| !value.is_finite())
    {
        return Err(error(
            "M4-MESH-INVALID",
            &path,
            "geometry counts, finite values or indices are invalid",
        ));
    }
    if let Some(tangents) = &segment.tangents
        && (tangents.len() != segment.positions.len()
            || tangents.iter().flatten().any(|value| !value.is_finite()))
    {
        return Err(error(
            "M4-MESH-INVALID",
            &format!("{path}.tangents"),
            "tangent deviation still requires finite one-per-vertex input",
        ));
    }
    for triangle in segment.indices.chunks_exact(3) {
        let _ = checked_face_plane(
            segment.positions[triangle[0] as usize],
            segment.positions[triangle[1] as usize],
            segment.positions[triangle[2] as usize],
            &format!("{path}.indices"),
            face_plane_degeneracy_policy,
        )?;
    }
    match segment.deformation {
        RigSegmentDeformationV1::Rigid => {
            if !segment.weights.is_empty() {
                return Err(error(
                    "M4-SKIN-LANE-INVALID",
                    &path,
                    "RIGID segment must not contain skin weights",
                ));
            }
        }
        RigSegmentDeformationV1::Skin => {
            validate_skin_weights(segment, index, id_to_index)?;
        }
    }
    Ok(())
}

fn validate_skin_weights(
    segment: &AuroraCreatureSegmentV1,
    segment_index: usize,
    id_to_index: &HashMap<u32, usize>,
) -> Result<(), MdlWriteError> {
    let weights_path = format!("creature.segments[{segment_index}].weights");
    if segment.weights.len() != segment.positions.len() {
        return Err(error(
            "M4-SKIN-LANE-INVALID",
            &weights_path,
            "SKIN weight row count must equal vertex count",
        ));
    }
    let mut active_bones = HashSet::new();
    for (vertex, row) in segment.weights.iter().enumerate() {
        let row_path = format!("{weights_path}[{vertex}]");
        if !(1..=4).contains(&row.influence_count) {
            return Err(error(
                "M4-SKIN-LANE-INVALID",
                &row_path,
                "influenceCount must be in 1..=4",
            ));
        }
        let active_count = usize::from(row.influence_count);
        let mut sum = 0.0_f64;
        for lane in 0..4 {
            let value = row.values[lane];
            let bone = row.bone_node_ids[lane];
            if lane < active_count {
                if !value.is_finite() || value <= 0.0 || bone.is_none() {
                    return Err(error(
                        "M4-SKIN-LANE-INVALID",
                        &row_path,
                        "active lanes require Some bone and positive finite weight",
                    ));
                }
                let bone = bone.unwrap_or_default();
                if !id_to_index.contains_key(&bone) {
                    return Err(error(
                        "M4-SKIN-BONE-MISSING",
                        &format!("{row_path}.boneNodeIds[{lane}]"),
                        "active bone id is absent from rig hierarchy",
                    ));
                }
                active_bones.insert(bone);
                sum += f64::from(value);
            } else if bone.is_some() || value.to_bits() != 0.0_f32.to_bits() {
                return Err(error(
                    "M4-SKIN-LANE-INVALID",
                    &row_path,
                    "inactive lanes require None bone and exact zero weight",
                ));
            }
        }
        if (sum - 1.0).abs() > f64::from(EPSILON) {
            return Err(error(
                "M4-SKIN-LANE-INVALID",
                &row_path,
                "active weights must sum to one within absolute tolerance 1e-5",
            ));
        }
    }
    if active_bones.len() > SKIN_INLINE_COUNT {
        return Err(error(
            "M4-SKIN-SLOT-LIMIT",
            &weights_path,
            "distinct active influencing bones exceed product guardrail 64",
        ));
    }
    Ok(())
}

fn emit_model(
    core: &mut [u8],
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
    supermodel_resref: &str,
    plan: &Plan,
) -> Result<(), MdlWriteError> {
    write_c_string(core, 0x08, 64, &options.model_resource_resref)?;
    write_u32(core, 0x48, as_u32(plan.root_offset, "model.root")?)?;
    write_u32(
        core,
        0x4c,
        as_u32(
            add(
                add(plan.rig.len(), plan.mesh.len(), "model.nodeCount")?,
                usize::from(plan.aabb.is_some()),
                "model.nodeCount",
            )?,
            "model.nodeCount",
        )?,
    )?;
    write_u32(core, 0x6c, 2)?;
    if let Some(pointer_array) = plan.animation_pointer_array {
        write_array(core, 0x78, pointer_array, plan.animations.len())?;
        for (index, animation) in plan.animations.iter().enumerate() {
            write_u32(
                core,
                pointer_array + index * 4,
                as_u32(animation.header, "model.animationPointers")?,
            )?;
        }
    }
    core[0x72] = match options.format_profile {
        MdlFormatProfileV1::ItemPartStaticRigidNativeV1 => 4,
        MdlFormatProfileV1::TileStaticV1 => 2,
        _ => 4,
    };
    core[0x73] = 1;
    write_vec3(core, 0x88, plan.model_bounds_min)?;
    write_vec3(core, 0x94, plan.model_bounds_max)?;
    write_f32(core, 0xa0, plan.model_radius)?;
    write_f32(core, 0xa4, 1.0)?;
    // Self-contained profiles pass the uppercase `NULL` sentinel. The
    // reference-supermodel route passes an exact validated resref while
    // retaining the same native binary field.
    write_c_string(core, 0xa8, 64, supermodel_resref)?;
    let _ = creature;
    Ok(())
}

fn emit_animations(
    core: &mut [u8],
    creature: &AuroraCreatureIrV1,
    animation_set: &MdlAnimationSetV1,
    plan: &Plan,
    state_projection_profile: MdlStateProjectionProfileV1,
) -> Result<(), MdlWriteError> {
    for animation in &plan.animations {
        let clip = &animation_set.clips[animation.clip_index];
        write_c_string(core, animation.header + 0x08, 64, &clip.name)?;
        write_u32(
            core,
            animation.header + 0x48,
            animation_as_u32(animation.root, "animations.root")?,
        )?;
        write_u32(
            core,
            animation.header + 0x4c,
            animation_as_u32(animation.nodes.len(), "animations.nodeCount")?,
        )?;
        // Binary local-animation headers use a one-byte type followed by
        // three padding bytes. The owned H1 v20 corrupt-draw witness uses
        // historical type 0; audited retail profiles use type 5.
        let animation_type = match state_projection_profile {
            MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 => 0,
            MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
            | MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
            | MdlStateProjectionProfileV1::CepRigidPlaceholderV1 => 5,
        };
        write_u32(core, animation.header + 0x6c, animation_type)?;
        write_f32(core, animation.header + 0x70, clip.length_seconds)?;
        write_f32(core, animation.header + 0x74, clip.transition_seconds)?;
        write_c_string(core, animation.header + 0x78, 64, &clip.animation_root)?;
        if let Some(events) = animation.events {
            write_array(
                core,
                animation.header + 0xb8,
                events,
                animation.sorted_events.len(),
            )?;
            for (event_index, event) in animation.sorted_events.iter().enumerate() {
                let offset = events + event_index * ANIMATION_EVENT_SIZE;
                write_f32(core, offset, event.time_seconds)?;
                write_c_string(core, offset + 4, 32, &event.name)?;
            }
        }
        for node in &animation.nodes {
            let (part_number, node_name) = if let AnimationNodeKind::Rig(rig_index) = node.kind {
                (
                    plan.source_to_part[rig_index],
                    creature.nodes[rig_index].name.clone(),
                )
            } else {
                let mesh_index = node
                    .kind
                    .mesh_index()
                    .expect("non-rig animation kind must identify a mesh segment");
                (
                    animation_as_u32(
                        animation_add(
                            creature.nodes.len(),
                            mesh_index,
                            "animations.nodes.partNumber",
                        )?,
                        "animations.nodes.partNumber",
                    )?,
                    format!("m2a_seg_{}", creature.segments[mesh_index].segment_id),
                )
            };
            write_u32(core, node.offset + 0x1c, part_number)?;
            write_c_string(core, node.offset + 0x20, 32, &node_name)?;
            if let Some(children) = node.children_array {
                write_array(
                    core,
                    node.offset + 0x48,
                    children,
                    node.children_offsets.len(),
                )?;
                for (child_index, &child) in node.children_offsets.iter().enumerate() {
                    write_u32(core, children + child_index * 4, child)?;
                }
            }
            if let Some(keys) = node.keys {
                write_array(core, node.offset + 0x54, keys, node.tracks.len())?;
                for (track_index, track) in node.tracks.iter().enumerate() {
                    write_animation_controller_key(
                        core,
                        keys + track_index * CONTROLLER_KEY_SIZE,
                        track,
                    )?;
                }
            }
            if let Some(data) = node.data {
                write_array(core, node.offset + 0x60, data, node.data_values.len())?;
                for (value_index, &value) in node.data_values.iter().enumerate() {
                    write_f32(core, data + value_index * 4, value)?;
                }
            }
            write_u32(
                core,
                node.offset + 0x6c,
                if node.kind.uses_zero_geometry_mesh_placeholder() {
                    0x21
                } else {
                    0x01
                },
            )?;
        }
    }
    Ok(())
}

fn write_animation_controller_key(
    bytes: &mut [u8],
    offset: usize,
    track: &AnimationTrackPlan,
) -> Result<(), MdlWriteError> {
    write_i32(bytes, offset, track.controller_type)?;
    write_u16(bytes, offset + 4, track.rows)?;
    write_u16(bytes, offset + 6, track.time_index)?;
    write_u16(bytes, offset + 8, track.data_index)?;
    let packed = bytes.get_mut(offset + 10).ok_or_else(|| {
        error(
            "M4A-LAYOUT-OVERFLOW",
            "payload",
            "animation controller key escapes buffer",
        )
    })?;
    *packed = track.packed_byte;
    Ok(())
}

fn emit_nodes(
    core: &mut [u8],
    creature: &AuroraCreatureIrV1,
    plan: &Plan,
) -> Result<(), MdlWriteError> {
    for item in &plan.rig {
        let node = &creature.nodes[item.source_index];
        write_u32(core, item.offset + 0x1c, item.part)?;
        write_c_string(core, item.offset + 0x20, 32, &node.name)?;
        if let Some(children) = item.children_array {
            write_array(
                core,
                item.offset + 0x48,
                children,
                item.children_offsets.len(),
            )?;
            for (child_index, &child) in item.children_offsets.iter().enumerate() {
                write_u32(core, children + child_index * 4, child)?;
            }
        }
        write_u32(core, item.offset + 0x6c, 0x01)?;
        if item.emit_bind_controllers {
            let key_count = CONTROLLER_KEY_COUNT + usize::from(item.bind_scale.is_some());
            let data_count = CONTROLLER_DATA_COUNT + 2 * usize::from(item.bind_scale.is_some());
            write_array(core, item.offset + 0x54, item.keys, key_count)?;
            write_array(core, item.offset + 0x60, item.data, data_count)?;
            write_controller_key(core, item.keys, 8, 0, 1, 3)?;
            write_controller_key(core, item.keys + CONTROLLER_KEY_SIZE, 20, 4, 5, 4)?;
            let matrix = node.bind_local_matrix;
            for (data_index, value) in [
                0.0,
                matrix[12],
                matrix[13],
                matrix[14],
                0.0,
                item.quaternion[0],
                item.quaternion[1],
                item.quaternion[2],
                item.quaternion[3],
            ]
            .into_iter()
            .enumerate()
            {
                write_f32(core, item.data + data_index * 4, value)?;
            }
            if let Some(scale) = item.bind_scale {
                write_controller_key(
                    core,
                    item.keys + CONTROLLER_KEY_COUNT * CONTROLLER_KEY_SIZE,
                    36,
                    9,
                    10,
                    1,
                )?;
                write_f32(core, item.data + CONTROLLER_DATA_COUNT * 4, 0.0)?;
                write_f32(core, item.data + (CONTROLLER_DATA_COUNT + 1) * 4, scale)?;
            }
        }
    }
    Ok(())
}

fn emit_meshes(
    core: &mut [u8],
    raw: &mut [u8],
    creature: &AuroraCreatureIrV1,
    plan: &Plan,
    options: &MdlWriterOptionsV1,
) -> Result<(), MdlWriteError> {
    for item in &plan.mesh {
        let segment = &creature.segments[item.segment_index];
        let base = item.offset;
        write_u32(core, base + 0x1c, item.part)?;
        write_c_string(
            core,
            base + 0x20,
            32,
            &format!("m2a_seg_{}", segment.segment_id),
        )?;
        write_u32(
            core,
            base + 0x6c,
            if item.skin.is_some() { 0x61 } else { 0x21 },
        )?;
        if let (Some(keys), Some(data)) = (item.bind_controller_keys, item.bind_controller_data) {
            write_array(core, base + 0x54, keys, CONTROLLER_KEY_COUNT)?;
            write_array(core, base + 0x60, data, CONTROLLER_DATA_COUNT)?;
            write_controller_key(core, keys, 8, 0, 1, 3)?;
            write_controller_key(core, keys + CONTROLLER_KEY_SIZE, 20, 4, 5, 4)?;
            for (data_index, value) in [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
                .into_iter()
                .enumerate()
            {
                write_f32(core, data + data_index * 4, value)?;
            }
        }
        write_array(core, base + 0x78, item.faces, segment.indices.len() / 3)?;
        write_vec3(core, base + 0x84, item.bounds_min)?;
        write_vec3(core, base + 0x90, item.bounds_max)?;
        write_f32(core, base + 0x9c, item.radius)?;
        write_vec3(core, base + 0xa0, item.average)?;
        write_vec3(core, base + 0xac, [1.0, 1.0, 1.0])?;
        write_vec3(core, base + 0xb8, [1.0, 1.0, 1.0])?;
        write_vec3(core, base + 0xc4, [0.0, 0.0, 0.0])?;
        write_f32(core, base + 0xd0, 1.0)?;
        write_u32(core, base + 0xd4, u32::from(segment.cast_shadow))?;
        write_u32(core, base + 0xdc, 1)?;
        write_c_string(
            core,
            base + 0xe8,
            64,
            &plan.textures[&segment.material_slot],
        )?;
        write_array(core, base + 0x204, item.index_count, 1)?;
        write_array(core, base + 0x210, item.index_offset, 1)?;
        write_i32(core, base + 0x21c, -1)?;
        // Aurora passes this four-byte field to the renderer. Every non-empty
        // triangle mesh in the bounded native corpus uses value 3, for both
        // M4 extended64 skin and M0 static-rigid direct-creature profiles.
        write_u32(core, base + 0x224, plan.mesh_type)?;
        write_i32(core, base + 0x228, 0)?;
        write_i32(
            core,
            base + 0x22c,
            as_i32(item.raw_positions, "mesh.vertices")?,
        )?;
        write_u16(core, base + 0x230, segment.positions.len() as u16)?;
        write_u16(core, base + 0x232, 1)?;
        write_i32(core, base + 0x234, as_i32(item.raw_uv0, "mesh.uv0")?)?;
        for offset in [0x238, 0x23c, 0x240] {
            write_i32(core, base + offset, -1)?;
        }
        write_i32(
            core,
            base + 0x244,
            as_i32(item.raw_normals, "mesh.normals")?,
        )?;
        write_i32(
            core,
            base + 0x248,
            item.raw_colors
                .map(|offset| as_i32(offset, "mesh.colors"))
                .transpose()?
                .unwrap_or(-1),
        )?;
        for offset in [0x24c, 0x250, 0x254, 0x258, 0x25c, 0x260] {
            write_i32(core, base + offset, -1)?;
        }
        write_u32(
            core,
            item.index_count,
            as_u32(segment.indices.len(), "mesh.indexCount")?,
        )?;
        write_i32(
            core,
            item.index_offset,
            as_i32(item.raw_indices, "mesh.indexOffset")?,
        )?;

        for (vertex, &value) in segment.positions.iter().enumerate() {
            write_vec3(raw, item.raw_positions + vertex * 12, value)?;
        }
        for (vertex, &value) in segment.uv0.iter().enumerate() {
            write_f32(raw, item.raw_uv0 + vertex * 8, value[0])?;
            write_f32(raw, item.raw_uv0 + vertex * 8 + 4, value[1])?;
        }
        for (vertex, &value) in segment.normals.iter().enumerate() {
            write_vec3(raw, item.raw_normals + vertex * 12, value)?;
        }
        if let Some(raw_colors) = item.raw_colors {
            for vertex in 0..segment.positions.len() {
                raw[raw_colors + vertex * 4..raw_colors + vertex * 4 + 4]
                    .copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        for (index, &value) in segment.indices.iter().enumerate() {
            write_u16(raw, item.raw_indices + index * 2, value as u16)?;
        }
        if let Some(skin) = &item.skin {
            write_i32(
                core,
                base + 0x27c,
                as_i32(skin.raw_weights, "skin.rawWeights")?,
            )?;
            write_i32(core, base + 0x280, as_i32(skin.raw_refs, "skin.rawRefs")?)?;
            write_i32(
                core,
                base + 0x284,
                as_i32(skin.forward_offset, "skin.forwardMap")?,
            )?;
            write_i32(
                core,
                base + 0x288,
                i32::try_from(skin.forward.len()).map_err(|_| {
                    error(
                        "M4-LAYOUT-OVERFLOW",
                        "skin.mapCount",
                        "skin map count cannot fit i32",
                    )
                })?,
            )?;
            write_array(core, base + 0x28c, skin.q_offset, skin.forward.len())?;
            write_array(core, base + 0x298, skin.t_offset, skin.forward.len())?;
            write_array(
                core,
                base + 0x2a4,
                skin.constants_offset,
                skin.forward.len(),
            )?;
            for (index, &value) in skin.inline_reverse.iter().enumerate() {
                write_i16(core, base + 0x2b0 + index * 2, value)?;
            }
            for (index, &value) in skin.forward.iter().enumerate() {
                write_i16(core, skin.forward_offset + index * 2, value)?;
            }
            for (index, &q) in skin.inverse_rotations_wxyz.iter().enumerate() {
                for (lane, value) in q.into_iter().enumerate() {
                    write_f32(core, skin.q_offset + index * 16 + lane * 4, value)?;
                }
            }
            for (index, &translation) in skin.inverse_translations.iter().enumerate() {
                write_vec3(core, skin.t_offset + index * 12, translation)?;
            }
            for index in 0..skin.forward.len() {
                write_u32(core, skin.constants_offset + index * 4, 0)?;
            }
            for (vertex, &weights) in skin.vertex_weights.iter().enumerate() {
                for (lane, value) in weights.into_iter().enumerate() {
                    write_f32(raw, skin.raw_weights + vertex * 16 + lane * 4, value)?;
                }
            }
            for (vertex, refs) in skin.vertex_refs.iter().enumerate() {
                for (lane, &value) in refs.iter().enumerate() {
                    write_u16(raw, skin.raw_refs + vertex * 8 + lane * 2, value)?;
                }
            }
        }
        let face_adjacency = face_adjacency_for_profile(
            &segment.positions,
            &segment.indices,
            options.format_profile,
            &format!("creature.segments[{}].indices", item.segment_index),
        )?;
        for (face_index, triangle) in segment.indices.chunks_exact(3).enumerate() {
            let a = segment.positions[triangle[0] as usize];
            let (normal, distance) = checked_face_plane(
                a,
                segment.positions[triangle[1] as usize],
                segment.positions[triangle[2] as usize],
                &format!("creature.segments[{}].indices", item.segment_index),
                plan.face_plane_degeneracy_policy,
            )?;
            let face = item.faces + face_index * FACE_SIZE;
            write_vec3(core, face, normal)?;
            write_f32(core, face + 0x0c, distance)?;
            write_i32(
                core,
                face + 0x10,
                segment
                    .face_surface_ids
                    .get(face_index)
                    .copied()
                    .unwrap_or(0),
            )?;
            for (edge_index, offset) in [0x14, 0x16, 0x18].into_iter().enumerate() {
                write_i16(core, face + offset, face_adjacency[face_index][edge_index])?;
            }
            write_u16(core, face + 0x1a, triangle[0] as u16)?;
            write_u16(core, face + 0x1c, triangle[1] as u16)?;
            write_u16(core, face + 0x1e, triangle[2] as u16)?;
        }
    }
    Ok(())
}

fn emit_aabb_mesh(
    core: &mut [u8],
    raw: &mut [u8],
    navigation: &TileNavigationIrV1,
    item: &AabbMeshPlan,
    mesh_type: u32,
) -> Result<(), MdlWriteError> {
    let base = item.offset;
    write_u32(core, base + 0x1c, item.part)?;
    write_c_string(core, base + 0x20, 32, &navigation.node_name)?;
    write_u32(core, base + 0x6c, 0x221)?;
    write_array(core, base + 0x78, item.faces, navigation.faces.len())?;
    write_vec3(core, base + 0x84, item.bounds_min)?;
    write_vec3(core, base + 0x90, item.bounds_max)?;
    write_f32(core, base + 0x9c, item.radius)?;
    write_vec3(core, base + 0xa0, item.average)?;
    write_vec3(core, base + 0xac, [1.0, 1.0, 1.0])?;
    write_vec3(core, base + 0xb8, [1.0, 1.0, 1.0])?;
    write_vec3(core, base + 0xc4, [0.0, 0.0, 0.0])?;
    write_f32(core, base + 0xd0, 1.0)?;
    write_u32(core, base + 0xd4, 1)?;
    write_u32(core, base + 0xdc, 1)?;
    write_array(core, base + 0x204, item.index_count, 1)?;
    write_array(core, base + 0x210, item.index_offset, 1)?;
    write_i32(core, base + 0x21c, -1)?;
    write_u32(core, base + 0x224, mesh_type)?;
    write_i32(core, base + 0x228, 0)?;
    write_i32(
        core,
        base + 0x22c,
        as_i32(item.raw_positions, "aabb.vertices")?,
    )?;
    write_u16(
        core,
        base + 0x230,
        u16::try_from(navigation.vertices.len()).map_err(|_| {
            error(
                "TILE-MDL-NAVIGATION-INVALID",
                "navigation.vertices",
                "AABB vertex count exceeds u16",
            )
        })?,
    )?;
    write_u16(core, base + 0x232, 1)?;
    write_i32(core, base + 0x234, as_i32(item.raw_uv0, "aabb.uv0")?)?;
    for offset in [0x238, 0x23c, 0x240] {
        write_i32(core, base + offset, -1)?;
    }
    write_i32(
        core,
        base + 0x244,
        as_i32(item.raw_normals, "aabb.normals")?,
    )?;
    write_i32(
        core,
        base + 0x248,
        item.raw_colors
            .map(|offset| as_i32(offset, "aabb.colors"))
            .transpose()?
            .unwrap_or(-1),
    )?;
    for offset in [0x24c, 0x250, 0x254, 0x258, 0x25c, 0x260] {
        write_i32(core, base + offset, -1)?;
    }
    let index_length = navigation.faces.len().checked_mul(3).ok_or_else(|| {
        error(
            "TILE-MDL-LAYOUT-OVERFLOW",
            "navigation.faces",
            "AABB index count overflow",
        )
    })?;
    write_u32(
        core,
        item.index_count,
        as_u32(index_length, "aabb.indexCount")?,
    )?;
    write_i32(
        core,
        item.index_offset,
        as_i32(item.raw_indices, "aabb.indexOffset")?,
    )?;
    for (vertex, &position) in navigation.vertices.iter().enumerate() {
        write_vec3(raw, item.raw_positions + vertex * 12, position)?;
        write_f32(raw, item.raw_uv0 + vertex * 8, (position[0] + 5.0) / 10.0)?;
        write_f32(
            raw,
            item.raw_uv0 + vertex * 8 + 4,
            (position[1] + 5.0) / 10.0,
        )?;
        write_vec3(raw, item.raw_normals + vertex * 12, [0.0, 0.0, 1.0])?;
        if let Some(colors) = item.raw_colors {
            raw[colors + vertex * 4..colors + vertex * 4 + 4]
                .copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    for (face_index, face) in navigation.faces.iter().enumerate() {
        for (lane, index) in face.vertex_indices.into_iter().enumerate() {
            write_u16(
                raw,
                item.raw_indices + (face_index * 3 + lane) * 2,
                u16::try_from(index).map_err(|_| {
                    error(
                        "TILE-MDL-NAVIGATION-INVALID",
                        &format!("navigation.faces[{face_index}].vertexIndices[{lane}]"),
                        "AABB vertex index exceeds u16",
                    )
                })?,
            )?;
        }
        let a = navigation.vertices[face.vertex_indices[0] as usize];
        let b = navigation.vertices[face.vertex_indices[1] as usize];
        let c = navigation.vertices[face.vertex_indices[2] as usize];
        let (normal, distance) = checked_face_plane(
            a,
            b,
            c,
            &format!("navigation.faces[{face_index}]"),
            FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
        )?;
        let offset = item.faces + face_index * FACE_SIZE;
        write_vec3(core, offset, normal)?;
        write_f32(core, offset + 0x0c, distance)?;
        write_i32(core, offset + 0x10, face.surface_id)?;
        for (edge, field) in [0x14, 0x16, 0x18].into_iter().enumerate() {
            write_i16(
                core,
                offset + field,
                i16::try_from(face.adjacent_faces[edge]).map_err(|_| {
                    error(
                        "TILE-MDL-NAVIGATION-INVALID",
                        &format!("navigation.faces[{face_index}].adjacentFaces[{edge}]"),
                        "AABB adjacency exceeds i16",
                    )
                })?,
            )?;
        }
        write_u16(core, offset + 0x1a, face.vertex_indices[0] as u16)?;
        write_u16(core, offset + 0x1c, face.vertex_indices[1] as u16)?;
        write_u16(core, offset + 0x1e, face.vertex_indices[2] as u16)?;
    }
    let root_offset = *item
        .entry_offsets
        .get(navigation.aabb_tree.root_index as usize)
        .ok_or_else(|| {
            error(
                "TILE-AABB-ROOT-OOB",
                "navigation.aabbTree.rootIndex",
                "AABB root index exceeds planned entry offsets",
            )
        })?;
    write_u32(core, base + 0x270, as_u32(root_offset, "aabb.root")?)?;
    emit_aabb_entries(
        core,
        &navigation.aabb_tree,
        &item.entry_offsets,
        "navigation.aabbTree",
    )
}

fn emit_aabb_entries(
    core: &mut [u8],
    tree: &AabbTreeV1,
    offsets: &[usize],
    path: &str,
) -> Result<(), MdlWriteError> {
    if offsets.len() != tree.entries.len() {
        return Err(error(
            "TILE-AABB-LAYOUT-OVERFLOW",
            path,
            "planned AABB entry offsets differ from tree entry count",
        ));
    }
    for (index, entry) in tree.entries.iter().enumerate() {
        let offset = offsets[index];
        write_vec3(core, offset, entry.bounds_min)?;
        write_vec3(core, offset + 0x0c, entry.bounds_max)?;
        write_u32(
            core,
            offset + 0x18,
            entry
                .left
                .map(|child| {
                    offsets
                        .get(child as usize)
                        .copied()
                        .ok_or_else(|| {
                            error(
                                "TILE-AABB-CHILD-OOB",
                                &format!("{path}.entries[{index}].left"),
                                "left child exceeds entry array",
                            )
                        })
                        .and_then(|value| as_u32(value, "aabb.left"))
                })
                .transpose()?
                .unwrap_or(0),
        )?;
        write_u32(
            core,
            offset + 0x1c,
            entry
                .right
                .map(|child| {
                    offsets
                        .get(child as usize)
                        .copied()
                        .ok_or_else(|| {
                            error(
                                "TILE-AABB-CHILD-OOB",
                                &format!("{path}.entries[{index}].right"),
                                "right child exceeds entry array",
                            )
                        })
                        .and_then(|value| as_u32(value, "aabb.right"))
                })
                .transpose()?
                .unwrap_or(0),
        )?;
        write_i32(
            core,
            offset + 0x20,
            entry
                .leaf_face
                .map(|face| {
                    i32::try_from(face).map_err(|_| {
                        error(
                            "TILE-AABB-LEAF-FACE-OOB",
                            &format!("{path}.entries[{index}].leafFace"),
                            "leaf face exceeds i32",
                        )
                    })
                })
                .transpose()?
                .unwrap_or(-1),
        )?;
        write_u32(core, offset + 0x24, entry.plane)?;
    }
    Ok(())
}

fn expected_readback(
    creature: &AuroraCreatureIrV1,
    animation_set: &MdlAnimationSetV1,
    options: &MdlWriterOptionsV1,
    supermodel_resref: &str,
    plan: &Plan,
    navigation: Option<&TileNavigationIrV1>,
) -> Result<ExpectedReadback, MdlWriteError> {
    let mut nodes =
        Vec::with_capacity(plan.rig.len() + plan.mesh.len() + usize::from(plan.aabb.is_some()));
    for item in &plan.rig {
        let index = item.source_index;
        nodes.push(ExpectedNode {
            ir_node_id: Some(item.id),
            part_number: item.part,
            name: creature.nodes[index].name.clone(),
            parent_part_number: item.parent_part,
            bind_matrix: Some(creature.nodes[index].bind_local_matrix),
            controllerless_identity: !item.emit_bind_controllers,
            mesh_bind_matrix: None,
            content_flags: 0x01,
            mesh: None,
            skin: None,
            aabb: None,
        });
    }
    for item in &plan.mesh {
        let segment = &creature.segments[item.segment_index];
        let expected_skin = item
            .skin
            .as_ref()
            .map(|skin| {
                Ok(ExpectedSkin {
                    raw_weights_pointer: as_i32(
                        skin.raw_weights,
                        "semantic.skin.rawWeightsPointer",
                    )?,
                    raw_refs_pointer: as_i32(skin.raw_refs, "semantic.skin.rawRefsPointer")?,
                    q_pointer: as_u32(skin.q_offset, "semantic.skin.qPointer")?,
                    t_pointer: as_u32(skin.t_offset, "semantic.skin.tPointer")?,
                    constants_pointer: as_u32(
                        skin.constants_offset,
                        "semantic.skin.constantsPointer",
                    )?,
                    forward: skin.forward.clone(),
                    inline_reverse: skin.inline_reverse.to_vec(),
                    inverse_rotations_wxyz: skin.inverse_rotations_wxyz.clone(),
                    inverse_translations: skin.inverse_translations.clone(),
                    bone_constants: vec![0; skin.forward.len()],
                    vertex_weights: skin.vertex_weights.clone(),
                    vertex_refs: skin.vertex_refs.clone(),
                    resolved_ir_ids: skin.resolved_ir_ids.clone(),
                })
            })
            .transpose()?;
        let face_adjacency = face_adjacency_for_profile(
            &segment.positions,
            &segment.indices,
            options.format_profile,
            "semantic.faces.adjacentFaces",
        )?;
        nodes.push(ExpectedNode {
            ir_node_id: None,
            part_number: item.part,
            name: format!("m2a_seg_{}", segment.segment_id),
            parent_part_number: Some(item.parent_part),
            bind_matrix: None,
            controllerless_identity: false,
            mesh_bind_matrix: item.skin.as_ref().map(|_| {
                [
                    1.0, 0.0, 0.0, 0.0, //
                    0.0, 1.0, 0.0, 0.0, //
                    0.0, 0.0, 1.0, 0.0, //
                    0.0, 0.0, 0.0, 1.0,
                ]
            }),
            content_flags: if item.skin.is_some() { 0x61 } else { 0x21 },
            mesh: Some(ExpectedMesh {
                texture_resref: plan.textures[&segment.material_slot].clone(),
                mesh_type: plan.mesh_type,
                positions: segment.positions.clone(),
                normals: segment.normals.clone(),
                uv0: segment.uv0.clone(),
                vertex_colors: if plan.emit_vertex_colors {
                    vec![[255, 255, 255, 255]; segment.positions.len()]
                } else {
                    Vec::new()
                },
                indices: segment.indices.iter().map(|value| *value as u16).collect(),
                bounds_min: item.bounds_min,
                bounds_max: item.bounds_max,
                radius: item.radius,
                average: item.average,
                shadow: u32::from(segment.cast_shadow),
                raw_index_offset: as_i32(item.raw_indices, "mesh.indexOffset")?,
                faces: segment
                    .indices
                    .chunks_exact(3)
                    .enumerate()
                    .map(|(face_index, triangle)| {
                        let (normal, distance) = checked_face_plane(
                            segment.positions[triangle[0] as usize],
                            segment.positions[triangle[1] as usize],
                            segment.positions[triangle[2] as usize],
                            "semantic.faces",
                            plan.face_plane_degeneracy_policy,
                        )?;
                        Ok(ExpectedFace {
                            normal,
                            distance,
                            surface_id: segment
                                .face_surface_ids
                                .get(face_index)
                                .copied()
                                .unwrap_or(0),
                            adjacent_faces: face_adjacency[face_index],
                            vertex_indices: [
                                triangle[0] as u16,
                                triangle[1] as u16,
                                triangle[2] as u16,
                            ],
                        })
                    })
                    .collect::<Result<Vec<_>, MdlWriteError>>()?,
            }),
            skin: expected_skin,
            aabb: None,
        });
    }
    if let (Some(aabb), Some(navigation)) = (plan.aabb.as_ref(), navigation) {
        let indices = navigation
            .faces
            .iter()
            .flat_map(|face| face.vertex_indices)
            .map(|index| index as u16)
            .collect::<Vec<_>>();
        let faces = navigation
            .faces
            .iter()
            .enumerate()
            .map(|(face_index, face)| {
                let (normal, distance) = checked_face_plane(
                    navigation.vertices[face.vertex_indices[0] as usize],
                    navigation.vertices[face.vertex_indices[1] as usize],
                    navigation.vertices[face.vertex_indices[2] as usize],
                    &format!("semantic.aabb.faces[{face_index}]"),
                    FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
                )?;
                Ok(ExpectedFace {
                    normal,
                    distance,
                    surface_id: face.surface_id,
                    adjacent_faces: [
                        face.adjacent_faces[0] as i16,
                        face.adjacent_faces[1] as i16,
                        face.adjacent_faces[2] as i16,
                    ],
                    vertex_indices: [
                        face.vertex_indices[0] as u16,
                        face.vertex_indices[1] as u16,
                        face.vertex_indices[2] as u16,
                    ],
                })
            })
            .collect::<Result<Vec<_>, MdlWriteError>>()?;
        let expected_aabb = ExpectedAabbTree {
            root_pointer: as_u32(
                aabb.entry_offsets[navigation.aabb_tree.root_index as usize],
                "semantic.aabb.root",
            )?,
            entries: navigation
                .aabb_tree
                .entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    Ok(ExpectedAabbEntry {
                        offset: as_u32(aabb.entry_offsets[index], "semantic.aabb.entry")?,
                        bounds_min: entry.bounds_min,
                        bounds_max: entry.bounds_max,
                        left_pointer: entry
                            .left
                            .map(|child| {
                                as_u32(aabb.entry_offsets[child as usize], "semantic.aabb.left")
                            })
                            .transpose()?,
                        right_pointer: entry
                            .right
                            .map(|child| {
                                as_u32(aabb.entry_offsets[child as usize], "semantic.aabb.right")
                            })
                            .transpose()?,
                        leaf_face: entry.leaf_face,
                        plane: entry.plane,
                    })
                })
                .collect::<Result<Vec<_>, MdlWriteError>>()?,
        };
        nodes.push(ExpectedNode {
            ir_node_id: None,
            part_number: aabb.part,
            name: navigation.node_name.clone(),
            parent_part_number: Some(aabb.parent_part),
            bind_matrix: None,
            controllerless_identity: false,
            mesh_bind_matrix: None,
            content_flags: 0x221,
            mesh: Some(ExpectedMesh {
                texture_resref: String::new(),
                mesh_type: plan.mesh_type,
                positions: navigation.vertices.clone(),
                normals: vec![[0.0, 0.0, 1.0]; navigation.vertices.len()],
                uv0: navigation
                    .vertices
                    .iter()
                    .map(|position| [(position[0] + 5.0) / 10.0, (position[1] + 5.0) / 10.0])
                    .collect(),
                vertex_colors: if plan.emit_vertex_colors {
                    vec![[255, 255, 255, 255]; navigation.vertices.len()]
                } else {
                    Vec::new()
                },
                indices,
                bounds_min: aabb.bounds_min,
                bounds_max: aabb.bounds_max,
                radius: aabb.radius,
                average: aabb.average,
                shadow: 1,
                raw_index_offset: as_i32(aabb.raw_indices, "aabb.indexOffset")?,
                faces,
            }),
            skin: None,
            aabb: Some(expected_aabb),
        });
    }
    let id_to_part = plan
        .rig
        .iter()
        .map(|node| (node.id, node.part))
        .collect::<HashMap<_, _>>();
    let mut animations = Vec::with_capacity(plan.animations.len());
    for animation in &plan.animations {
        let clip = &animation_set.clips[animation.clip_index];
        let mut animation_nodes = Vec::with_capacity(animation.nodes.len());
        for node in &animation.nodes {
            let (part_number, name, parent_part_number) =
                if let AnimationNodeKind::Rig(rig_index) = node.kind {
                    let source = &creature.nodes[rig_index];
                    (
                        plan.source_to_part[rig_index],
                        source.name.clone(),
                        source.parent_id.map(|parent| id_to_part[&parent]),
                    )
                } else {
                    let mesh_index = node
                        .kind
                        .mesh_index()
                        .expect("non-rig animation kind must identify a mesh segment");
                    let segment = &creature.segments[mesh_index];
                    let parent_part_number = *id_to_part
                        .get(&segment.parent_node_id)
                        .expect("validated mesh parent exists in output rig");
                    (
                        animation_as_u32(
                            animation_add(
                                creature.nodes.len(),
                                mesh_index,
                                "semantic.animations.partNumber",
                            )?,
                            "semantic.animations.partNumber",
                        )?,
                        format!("m2a_seg_{}", segment.segment_id),
                        Some(parent_part_number),
                    )
                };
            let mut controllers = Vec::with_capacity(node.tracks.len());
            for (track_index, track) in node.tracks.iter().enumerate() {
                let columns = usize::from(track.packed_byte & 0x0f);
                let time_start = usize::from(track.time_index);
                let data_start = usize::from(track.data_index);
                let rows = usize::from(track.rows);
                let times = node.data_values[time_start..time_start + rows].to_vec();
                let values = (0..rows)
                    .map(|row| {
                        let start = data_start + row * columns;
                        node.data_values[start..start + columns].to_vec()
                    })
                    .collect();
                controllers.push(ExpectedAnimationController {
                    key_offset: animation_as_u32(
                        node.keys.unwrap_or_default() + track_index * CONTROLLER_KEY_SIZE,
                        "semantic.animations.controllerKey",
                    )?,
                    controller_type: track.controller_type,
                    packed_byte: track.packed_byte,
                    rows,
                    time_index: time_start,
                    data_index: data_start,
                    times,
                    values,
                });
            }
            animation_nodes.push(ExpectedAnimationNode {
                offset: animation_as_u32(node.offset, "semantic.animations.node")?,
                part_number,
                name,
                parent_part_number,
                children_header: array_report(
                    node.children_array,
                    node.children_offsets.len(),
                    "semantic.animations.children",
                )?,
                controller_keys_header: array_report(
                    node.keys,
                    node.tracks.len(),
                    "semantic.animations.controllerKeys",
                )?,
                controller_data_header: array_report(
                    node.data,
                    node.data_values.len(),
                    "semantic.animations.controllerData",
                )?,
                controllers,
                mesh_placeholder: node.kind.uses_zero_geometry_mesh_placeholder(),
            });
        }
        animations.push(ExpectedAnimation {
            offset: animation_as_u32(animation.header, "semantic.animations.header")?,
            name: clip.name.clone(),
            animation_type: match options.state_projection_profile {
                MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1 => 0,
                MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
                | MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
                | MdlStateProjectionProfileV1::CepRigidPlaceholderV1 => 5,
            },
            length: clip.length_seconds,
            transition: clip.transition_seconds,
            animation_root: clip.animation_root.clone(),
            events_header: array_report(
                animation.events,
                animation.sorted_events.len(),
                "semantic.animations.events",
            )?,
            events: animation
                .sorted_events
                .iter()
                .map(|event| (event.time_seconds, event.name.clone()))
                .collect(),
            root_part_number: match animation.nodes[0].kind {
                AnimationNodeKind::Rig(rig_index) => plan.source_to_part[rig_index],
                AnimationNodeKind::RigidMeshPlaceholder(_) | AnimationNodeKind::MeshDummy(_) => {
                    return Err(error(
                        "M4A-LAYOUT-OVERFLOW",
                        "semantic.animations.rootPartNumber",
                        "an animation root must be a rig node",
                    ));
                }
            },
            nodes: animation_nodes,
        });
    }
    Ok(ExpectedReadback {
        model_name: options.model_resource_resref.clone(),
        supermodel_name: supermodel_resref.to_owned(),
        model_bounds_min: plan.model_bounds_min,
        model_bounds_max: plan.model_bounds_max,
        model_radius: plan.model_radius,
        classification: match options.format_profile {
            MdlFormatProfileV1::ItemPartStaticRigidNativeV1 => 4,
            MdlFormatProfileV1::TileStaticV1 => 2,
            _ => 4,
        },
        root_part_number: plan
            .rig
            .iter()
            .find(|item| item.offset == plan.root_offset)
            .unwrap()
            .part,
        nodes,
        animation_pointers_header: array_report(
            plan.animation_pointer_array,
            plan.animations.len(),
            "semantic.animations.pointerArray",
        )?,
        animations,
    })
}

fn animation_writer_report(
    creature: &AuroraCreatureIrV1,
    animation_set: &MdlAnimationSetV1,
    plan: &Plan,
) -> Result<Option<MdlAnimationWriterReportV1>, MdlWriteError> {
    if plan.animations.is_empty() {
        return Ok(None);
    }
    let event_count = plan.animations.iter().try_fold(0usize, |sum, animation| {
        animation_add(
            sum,
            animation.sorted_events.len(),
            "report.animation.eventCount",
        )
    })?;
    let track_count = plan.animations.iter().try_fold(0usize, |sum, animation| {
        animation_add(sum, animation.track_count, "report.animation.trackCount")
    })?;
    let clips = plan
        .animations
        .iter()
        .map(|animation| {
            let nodes = animation
                .nodes
                .iter()
                .map(|node| {
                    let (ir_node_id, part_number) =
                        if let AnimationNodeKind::Rig(rig_index) = node.kind {
                            let part = plan.source_to_part[rig_index] as usize;
                            (Some(plan.rig[part].id), plan.source_to_part[rig_index])
                        } else {
                            let mesh_index = node
                                .kind
                                .mesh_index()
                                .expect("non-rig animation kind must identify a mesh segment");
                            (
                                None,
                                animation_as_u32(
                                    animation_add(
                                        creature.nodes.len(),
                                        mesh_index,
                                        "report.animation.node.partNumber",
                                    )?,
                                    "report.animation.node.partNumber",
                                )?,
                            )
                        };
                    let key_base = if node.tracks.is_empty() {
                        0
                    } else {
                        node.keys.ok_or_else(|| {
                            error(
                                "M4A-LAYOUT-OVERFLOW",
                                "report.animation.node.controllerKeysCoreOffset",
                                "planned animation tracks are missing their key array",
                            )
                        })?
                    };
                    let tracks = node
                        .tracks
                        .iter()
                        .enumerate()
                        .map(|(track_index, track)| {
                            let target_node_id = match node.kind {
                                AnimationNodeKind::Rig(rig_index) => {
                                    plan.rig[plan.source_to_part[rig_index] as usize].id
                                }
                                AnimationNodeKind::RigidMeshPlaceholder(_)
                                | AnimationNodeKind::MeshDummy(_) => {
                                    return Err(error(
                                        "M4A-LAYOUT-OVERFLOW",
                                        "report.animation.node.tracks",
                                        "a virtual mesh animation dummy cannot own tracks",
                                    ));
                                }
                            };
                            Ok(MdlAnimationTrackLayoutV1 {
                                target_node_id,
                                path: match track.controller_type {
                                    8 => MdlAnimationTrackPathV1::Translation,
                                    20 => MdlAnimationTrackPathV1::Rotation,
                                    36 => MdlAnimationTrackPathV1::Scale,
                                    _ => unreachable!(
                                        "writer emits only known animation controllers"
                                    ),
                                },
                                controller_type: track.controller_type,
                                packed_byte: track.packed_byte,
                                key_core_offset: animation_as_u32(
                                    key_base + track_index * CONTROLLER_KEY_SIZE,
                                    "report.animation.track.keyCoreOffset",
                                )?,
                                row_count: usize::from(track.rows),
                                time_index: usize::from(track.time_index),
                                data_index: usize::from(track.data_index),
                            })
                        })
                        .collect::<Result<Vec<_>, MdlWriteError>>()?;
                    Ok(MdlAnimationNodeLayoutV1 {
                        ir_node_id,
                        part_number,
                        core_offset: animation_as_u32(
                            node.offset,
                            "report.animation.node.coreOffset",
                        )?,
                        children_array_core_offset: report_animation_offset(
                            node.children_array,
                            "report.animation.node.childrenArrayCoreOffset",
                        )?,
                        controller_keys_core_offset: report_animation_offset(
                            node.keys,
                            "report.animation.node.controllerKeysCoreOffset",
                        )?,
                        controller_data_core_offset: report_animation_offset(
                            node.data,
                            "report.animation.node.controllerDataCoreOffset",
                        )?,
                        tracks,
                    })
                })
                .collect::<Result<Vec<_>, MdlWriteError>>()?;
            Ok(MdlAnimationClipLayoutV1 {
                name: animation_set.clips[animation.clip_index].name.clone(),
                header_core_offset: animation_as_u32(
                    animation.header,
                    "report.animation.headerCoreOffset",
                )?,
                root_core_offset: animation_as_u32(
                    animation.root,
                    "report.animation.rootCoreOffset",
                )?,
                event_array_core_offset: report_animation_offset(
                    animation.events,
                    "report.animation.eventArrayCoreOffset",
                )?,
                event_count: animation.sorted_events.len(),
                track_count: animation.track_count,
                node_count: animation.nodes.len(),
                nodes,
            })
        })
        .collect::<Result<Vec<_>, MdlWriteError>>()?;
    Ok(Some(MdlAnimationWriterReportV1 {
        pointer_array_core_offset: animation_as_u32(
            plan.animation_pointer_array.ok_or_else(|| {
                error(
                    "M4A-LAYOUT-OVERFLOW",
                    "report.animation.pointerArrayCoreOffset",
                    "non-empty animation plan is missing its model pointer array",
                )
            })?,
            "report.animation.pointerArrayCoreOffset",
        )?,
        clip_count: plan.animations.len(),
        event_count,
        track_count,
        clips,
    }))
}

fn report_animation_offset(
    offset: Option<usize>,
    path: &str,
) -> Result<Option<u32>, MdlWriteError> {
    offset
        .map(|value| animation_as_u32(value, path))
        .transpose()
}

fn array_report(
    pointer: Option<usize>,
    count: usize,
    path: &str,
) -> Result<ArrayReport, MdlWriteError> {
    Ok(ArrayReport {
        pointer: pointer
            .map(|value| animation_as_u32(value, path))
            .transpose()?
            .unwrap_or(0),
        used: count,
        allocated: count,
    })
}

fn tree_ordinals(
    root_part: usize,
    rig: &[RigPlan],
    mesh: &[MeshPlan],
    aabb: Option<&AabbMeshPlan>,
) -> Result<(Vec<usize>, Vec<usize>), MdlWriteError> {
    let total = add(
        add(rig.len(), mesh.len(), "layout.treeOrdinals")?,
        usize::from(aabb.is_some()),
        "layout.treeOrdinals",
    )?;
    let mut children = vec![Vec::<usize>::new(); rig.len()];
    for (child, node) in rig.iter().enumerate() {
        if let Some(parent) = node.parent_part {
            children[parent as usize].push(child);
        }
    }
    for (mesh_index, item) in mesh.iter().enumerate() {
        children[item.parent_part as usize].push(rig.len() + mesh_index);
    }
    if let Some(aabb) = aabb {
        children[aabb.parent_part as usize].push(aabb.part as usize);
    }
    let mut ordinal_parts = Vec::with_capacity(total);
    let mut pending = vec![root_part];
    while let Some(part) = pending.pop() {
        ordinal_parts.push(part);
        if part < rig.len() {
            for &child in children[part].iter().rev() {
                pending.push(child);
            }
        }
    }
    if ordinal_parts.len() != total {
        return Err(error(
            "M4-HIERARCHY-INVALID",
            "creature",
            "binary node tree is not fully reachable",
        ));
    }
    let mut part_to_ordinal = vec![usize::MAX; total];
    for (ordinal, &part) in ordinal_parts.iter().enumerate() {
        part_to_ordinal[part] = ordinal;
    }
    Ok((part_to_ordinal, ordinal_parts))
}

#[allow(clippy::too_many_arguments)]
fn build_skin_plan(
    skin: &mut SkinPlan,
    segment: &AuroraCreatureSegmentV1,
    segment_index: usize,
    parent_part: usize,
    id_to_part: &HashMap<u32, usize>,
    part_to_tree_ordinal: &[usize],
    ordinal_parts: &[usize],
    binary_worlds: &[[f64; 16]],
) -> Result<(), MdlWriteError> {
    let mut active_ordinals = segment
        .weights
        .iter()
        .flat_map(|row| row.bone_node_ids.iter().flatten().copied())
        .map(|id| part_to_tree_ordinal[id_to_part[&id]])
        .collect::<Vec<_>>();
    active_ordinals.sort_unstable();
    active_ordinals.dedup();
    if active_ordinals.len() > SKIN_INLINE_COUNT {
        return Err(error(
            "M4-SKIN-SLOT-LIMIT",
            &format!("creature.segments[{segment_index}].weights"),
            "distinct active influencing bones exceed product guardrail 64",
        ));
    }
    let mut ordinal_to_slot = HashMap::with_capacity(active_ordinals.len());
    skin.forward = vec![-1; ordinal_parts.len()];
    for (slot, &ordinal) in active_ordinals.iter().enumerate() {
        let slot_i16 = i16::try_from(slot).map_err(|_| {
            error(
                "M4-SKIN-SLOT-LIMIT",
                &format!("creature.segments[{segment_index}].weights"),
                "skin slot cannot fit i16",
            )
        })?;
        let ordinal_i16 = i16::try_from(ordinal).map_err(|_| {
            error(
                "M4-LAYOUT-OVERFLOW",
                "layout.skin.treeOrdinal",
                "tree ordinal cannot fit i16 inline mapping",
            )
        })?;
        skin.forward[ordinal] = slot_i16;
        skin.inline_reverse[slot] = ordinal_i16;
        ordinal_to_slot.insert(ordinal, slot as u16);
    }

    let skin_world = binary_worlds[parent_part];
    skin.inverse_rotations_wxyz = Vec::with_capacity(ordinal_parts.len());
    skin.inverse_translations = Vec::with_capacity(ordinal_parts.len());
    for &part in ordinal_parts {
        let relative_f64 = mul_mat4_f64(inverse_rigid_f64(binary_worlds[part]), skin_world);
        let mut relative = [0.0_f32; 16];
        for (output, value) in relative.iter_mut().zip(relative_f64) {
            if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
                return Err(error(
                    "M4-SKIN-INVERSE-BIND-UNSUPPORTED",
                    &format!("creature.segments[{segment_index}].parentNodeId"),
                    "relative inverse-bind matrix exceeds finite f32 representation",
                ));
            }
            *output = value as f32;
        }
        let q = matrix_quaternion(relative, "layout.skin.inverseBind").map_err(|_| {
            error(
                "M4-SKIN-INVERSE-BIND-UNSUPPORTED",
                &format!("creature.segments[{segment_index}].parentNodeId"),
                "relative inverse-bind matrix is not finite proper rigid",
            )
        })?;
        skin.inverse_rotations_wxyz.push([q[3], q[0], q[1], q[2]]);
        skin.inverse_translations
            .push([relative[12], relative[13], relative[14]]);
    }

    skin.vertex_weights = segment.weights.iter().map(|row| row.values).collect();
    skin.vertex_refs = Vec::with_capacity(segment.weights.len());
    skin.resolved_ir_ids = Vec::with_capacity(segment.weights.len());
    for row in &segment.weights {
        let mut refs = [u16::MAX; 4];
        let mut resolved = [None; 4];
        for lane in 0..usize::from(row.influence_count) {
            let id = row.bone_node_ids[lane].unwrap_or_default();
            let ordinal = part_to_tree_ordinal[id_to_part[&id]];
            refs[lane] = ordinal_to_slot[&ordinal];
            resolved[lane] = Some(id);
        }
        skin.vertex_refs.push(refs);
        skin.resolved_ir_ids.push(resolved);
    }

    if skin
        .forward
        .iter()
        .enumerate()
        .filter(|(_, slot)| **slot >= 0)
        .any(|(ordinal, slot)| {
            skin.inline_reverse[*slot as usize] != i16::try_from(ordinal).unwrap_or(-1)
        })
    {
        return Err(error(
            "M4-SKIN-MAPPING-INVALID",
            &format!("creature.segments[{segment_index}]"),
            "forward and inline skin maps are not inverse",
        ));
    }
    Ok(())
}

fn validate_skin_layout(mesh: &[MeshPlan], map_count: usize) -> Result<(), MdlWriteError> {
    let skins = mesh
        .iter()
        .enumerate()
        .filter_map(|(index, item)| item.skin.as_ref().map(|skin| (index, item, skin)))
        .collect::<Vec<_>>();
    for &(index, item, skin) in &skins {
        let expected_forward = add(item.offset, SKIN_HEADER_SIZE, "layout.skin.forwardMap")?;
        if skin.forward_offset != expected_forward
            || skin.forward.len() != map_count
            || skin.inverse_rotations_wxyz.len() != map_count
            || skin.inverse_translations.len() != map_count
            || skin.vertex_weights.len() != skin.vertex_refs.len()
            || [
                skin.forward_offset,
                skin.q_offset,
                skin.t_offset,
                skin.constants_offset,
                skin.raw_weights,
                skin.raw_refs,
            ]
            .into_iter()
            .any(|offset| !offset.is_multiple_of(4))
        {
            return Err(error(
                "M4-SKIN-LAYOUT-INVALID",
                &format!("layout.skinNodes[{index}]"),
                "planned skin boundary, count or alignment differs from locked extended64 layout",
            ));
        }
    }
    let q_bytes = mul(map_count, 16, "layout.skin.qInverse")?;
    let t_bytes = mul(map_count, 12, "layout.skin.tInverse")?;
    let constants_bytes = mul(map_count, 4, "layout.skin.constants")?;
    for pair in skins.windows(2) {
        let (left_index, _, left) = pair[0];
        let (_, _, right) = pair[1];
        if right.q_offset != add(left.q_offset, q_bytes, "layout.skin.qInverse")?
            || right.t_offset != add(left.t_offset, t_bytes, "layout.skin.tInverse")?
            || right.constants_offset
                != add(
                    left.constants_offset,
                    constants_bytes,
                    "layout.skin.constants",
                )?
        {
            return Err(error(
                "M4-SKIN-LAYOUT-INVALID",
                &format!("layout.skinNodes[{left_index}]"),
                "skin category arrays are not contiguous in segment order",
            ));
        }
    }
    if let (Some((_, _, first)), Some((last_index, _, last))) = (skins.first(), skins.last())
        && (first.t_offset != add(last.q_offset, q_bytes, "layout.skin.qToTBoundary")?
            || first.constants_offset
                != add(last.t_offset, t_bytes, "layout.skin.tToConstantsBoundary")?)
    {
        return Err(error(
            "M4-SKIN-LAYOUT-INVALID",
            &format!("layout.skinNodes[{last_index}]"),
            "q, t and constants category boundaries differ from locked order",
        ));
    }
    Ok(())
}

fn validate_skin_signed_fields(mesh: &[MeshPlan], map_count: usize) -> Result<(), MdlWriteError> {
    if mesh.iter().any(|item| item.skin.is_some()) {
        let _ = as_i32(map_count, "layout.skin.mapCount")?;
        for item in mesh {
            if let Some(skin) = &item.skin {
                let _ = as_i32(skin.forward_offset, "layout.skin.forwardMap")?;
            }
        }
    }
    Ok(())
}

fn inverse_rigid_f64(matrix: [f64; 16]) -> [f64; 16] {
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
    [
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
    ]
}

fn validate_acyclic(parents: &[Option<usize>]) -> Result<(), MdlWriteError> {
    for start in 0..parents.len() {
        let mut seen = HashSet::new();
        let mut current = Some(start);
        while let Some(index) = current {
            if !seen.insert(index) {
                return Err(error(
                    "M4-HIERARCHY-INVALID",
                    "creature.nodes",
                    "rig hierarchy contains a cycle",
                ));
            }
            current = parents[index];
        }
    }
    Ok(())
}

fn world_matrices(
    creature: &AuroraCreatureIrV1,
    parents: &[Option<usize>],
) -> Result<Vec<[f32; 16]>, MdlWriteError> {
    let mut worlds = vec![None; creature.nodes.len()];
    for start in 0..creature.nodes.len() {
        let mut chain = Vec::new();
        let mut current = start;
        while worlds[current].is_none() {
            chain.push(current);
            let Some(parent) = parents[current] else {
                break;
            };
            current = parent;
        }
        while let Some(index) = chain.pop() {
            let world = match parents[index] {
                Some(parent) => mul_mat4(
                    worlds[parent].ok_or_else(|| {
                        error(
                            "M4-HIERARCHY-INVALID",
                            "creature.nodes",
                            "parent world missing",
                        )
                    })?,
                    creature.nodes[index].bind_local_matrix,
                ),
                None => creature.nodes[index].bind_local_matrix,
            };
            if world.iter().any(|value| !value.is_finite()) {
                return Err(error(
                    "M4-MESH-INVALID",
                    "creature.nodes.bindLocalMatrix",
                    "composed bind transform overflowed to a non-finite value",
                ));
            }
            worlds[index] = Some(world);
        }
    }
    worlds
        .into_iter()
        .map(|value| {
            value.ok_or_else(|| {
                error(
                    "M4-HIERARCHY-INVALID",
                    "creature.nodes",
                    "world transform missing",
                )
            })
        })
        .collect()
}

fn world_matrices_f64(
    creature: &AuroraCreatureIrV1,
    parents: &[Option<usize>],
    quaternions_xyzw: &[[f32; 4]],
) -> Result<Vec<[f64; 16]>, MdlWriteError> {
    let mut worlds = vec![None; creature.nodes.len()];
    for start in 0..creature.nodes.len() {
        let mut chain = Vec::new();
        let mut current = start;
        while worlds[current].is_none() {
            chain.push(current);
            let Some(parent) = parents[current] else {
                break;
            };
            current = parent;
        }
        while let Some(index) = chain.pop() {
            let local = rigid_f64(
                quaternions_xyzw[index],
                [
                    creature.nodes[index].bind_local_matrix[12],
                    creature.nodes[index].bind_local_matrix[13],
                    creature.nodes[index].bind_local_matrix[14],
                ],
            );
            let world = match parents[index] {
                Some(parent) => mul_mat4_f64(
                    worlds[parent].ok_or_else(|| {
                        error(
                            "M4-HIERARCHY-INVALID",
                            "creature.nodes",
                            "parent f64 skin world missing",
                        )
                    })?,
                    local,
                ),
                None => local,
            };
            if world.iter().any(|value| !value.is_finite()) {
                return Err(error(
                    "M4-SKIN-INVERSE-BIND-UNSUPPORTED",
                    "creature.nodes.bindLocalMatrix",
                    "composed f64 skin bind transform is not finite",
                ));
            }
            worlds[index] = Some(world);
        }
    }
    worlds
        .into_iter()
        .map(|value| {
            value.ok_or_else(|| {
                error(
                    "M4-HIERARCHY-INVALID",
                    "creature.nodes",
                    "f64 skin world transform missing",
                )
            })
        })
        .collect()
}

fn rigid_f64(q: [f32; 4], translation: [f32; 3]) -> [f64; 16] {
    let [x, y, z, w] = q.map(f64::from);
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
        f64::from(translation[0]),
        f64::from(translation[1]),
        f64::from(translation[2]),
        1.0,
    ]
}

fn matrix_quaternion(matrix: [f32; 16], path: &str) -> Result<[f32; 4], MdlWriteError> {
    if matrix.iter().any(|value| !value.is_finite())
        || matrix[3].abs() > EPSILON
        || matrix[7].abs() > EPSILON
        || matrix[11].abs() > EPSILON
        || (matrix[15] - 1.0).abs() > EPSILON
    {
        return Err(error(
            "M4-BIND-TRANSFORM-UNSUPPORTED",
            path,
            "matrix is not finite affine",
        ));
    }
    let columns = [
        [matrix[0], matrix[1], matrix[2]],
        [matrix[4], matrix[5], matrix[6]],
        [matrix[8], matrix[9], matrix[10]],
    ];
    if columns
        .iter()
        .any(|column| (length3(*column) - 1.0).abs() > EPSILON)
        || dot(columns[0], columns[1]).abs() > EPSILON
        || dot(columns[0], columns[2]).abs() > EPSILON
        || dot(columns[1], columns[2]).abs() > EPSILON
        || (dot(columns[0], cross(columns[1], columns[2])) - 1.0).abs() > EPSILON
    {
        return Err(error(
            "M4-BIND-TRANSFORM-UNSUPPORTED",
            path,
            "matrix is not a proper rigid transform",
        ));
    }
    let m00 = matrix[0];
    let m01 = matrix[4];
    let m02 = matrix[8];
    let m10 = matrix[1];
    let m11 = matrix[5];
    let m12 = matrix[9];
    let m20 = matrix[2];
    let m21 = matrix[6];
    let m22 = matrix[10];
    let trace = m00 + m11 + m22;
    let (x, y, z, w) = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        ((m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s)
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        (0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s)
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        ((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s)
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        ((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s)
    };
    let inverse = 1.0 / (x * x + y * y + z * z + w * w).sqrt();
    let mut q = [x * inverse, y * inverse, z * inverse, w * inverse];
    let flip = q[3] < 0.0
        || (q[3] == 0.0
            && q[..3]
                .iter()
                .find(|value| **value != 0.0)
                .is_some_and(|value| *value < 0.0));
    if flip {
        for value in &mut q {
            *value = -*value;
        }
    }
    Ok(q)
}

fn matrix_quaternion_with_uniform_scale(
    matrix: [f32; 16],
    path: &str,
) -> Result<([f32; 4], f32), MdlWriteError> {
    if matrix.iter().any(|value| !value.is_finite())
        || matrix[3].abs() > EPSILON
        || matrix[7].abs() > EPSILON
        || matrix[11].abs() > EPSILON
        || (matrix[15] - 1.0).abs() > EPSILON
    {
        return Err(error(
            "ITEM-MDL-BIND-TRANSFORM-UNSUPPORTED",
            path,
            "item-part bind matrix must be finite affine",
        ));
    }
    let columns = [
        [matrix[0], matrix[1], matrix[2]],
        [matrix[4], matrix[5], matrix[6]],
        [matrix[8], matrix[9], matrix[10]],
    ];
    let lengths = columns.map(length3);
    let scale = (lengths[0] + lengths[1] + lengths[2]) / 3.0;
    let tolerance = EPSILON * scale.max(1.0);
    if !scale.is_finite()
        || scale <= EPSILON
        || lengths
            .iter()
            .any(|length| (*length - scale).abs() > tolerance)
    {
        return Err(error(
            "ITEM-MDL-BIND-TRANSFORM-UNSUPPORTED",
            path,
            "item-part bind matrix must have a positive uniform scale",
        ));
    }
    let mut rigid = matrix;
    for index in [0, 1, 2, 4, 5, 6, 8, 9, 10] {
        rigid[index] /= scale;
    }
    matrix_quaternion(rigid, path).map(|quaternion| (quaternion, scale))
}

fn mesh_metrics(positions: &[[f32; 3]]) -> Result<MeshMetrics, MdlWriteError> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    let mut sum = [0.0_f64; 3];
    let mut radius = 0.0_f32;
    for &position in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
            sum[axis] += f64::from(position[axis]);
        }
        radius = radius.max(checked_length3(position).ok_or_else(|| {
            error(
                "M4-MESH-INVALID",
                "creature.segments.positions",
                "mesh radius overflowed to a non-finite value",
            )
        })?);
    }
    let divisor = positions.len() as f64;
    let average = [
        (sum[0] / divisor) as f32,
        (sum[1] / divisor) as f32,
        (sum[2] / divisor) as f32,
    ];
    if !finite3(average) {
        return Err(error(
            "M4-MESH-INVALID",
            "creature.segments.positions",
            "mesh average overflowed to a non-finite value",
        ));
    }
    Ok(MeshMetrics {
        bounds_min: min,
        bounds_max: max,
        radius,
        average,
    })
}

fn validate_resref(value: &str, path: &str) -> Result<(), MdlWriteError> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "M4-INVALID-NAME",
            path,
            "generated resref must match [a-z0-9_]{1,16}",
        ));
    }
    Ok(())
}

fn validate_node_name(value: &str, path: &str) -> Result<(), MdlWriteError> {
    if value.is_empty()
        || value.len() > 31
        || !value.is_ascii()
        || value.bytes().any(|byte| byte == 0)
    {
        return Err(error(
            "M4-INVALID-NAME",
            path,
            "node name must be a non-empty ASCII C string of at most 31 bytes",
        ));
    }
    Ok(())
}

fn deviation(code: &str, path: &str) -> MdlWriterDeviationV1 {
    MdlWriterDeviationV1 {
        code: code.to_owned(),
        path: path.to_owned(),
        message: "structural writer value is explicit but runtime acceptance remains OPEN_M6"
            .to_owned(),
    }
}

fn error(code: &str, path: &str, message: impl Into<String>) -> MdlWriteError {
    MdlWriteError::fatal(code, path, message)
}

fn add(left: usize, right: usize, path: &str) -> Result<usize, MdlWriteError> {
    left.checked_add(right)
        .ok_or_else(|| error("M4-LAYOUT-OVERFLOW", path, "addition overflow"))
}

fn mul(left: usize, right: usize, path: &str) -> Result<usize, MdlWriteError> {
    left.checked_mul(right)
        .ok_or_else(|| error("M4-LAYOUT-OVERFLOW", path, "multiplication overflow"))
}

fn take(cursor: &mut usize, length: usize, path: &str) -> Result<usize, MdlWriteError> {
    let start = *cursor;
    *cursor = add(*cursor, length, path)?;
    Ok(start)
}

fn align4(value: usize, path: &str) -> Result<usize, MdlWriteError> {
    add(value, 3, path).map(|value| value & !3)
}

fn as_u32(value: usize, path: &str) -> Result<u32, MdlWriteError> {
    u32::try_from(value).map_err(|_| error("M4-LAYOUT-OVERFLOW", path, "value exceeds u32"))
}

fn as_i32(value: usize, path: &str) -> Result<i32, MdlWriteError> {
    i32::try_from(value).map_err(|_| error("M4-LAYOUT-OVERFLOW", path, "raw pointer exceeds i32"))
}

fn write_array(
    bytes: &mut [u8],
    offset: usize,
    pointer: usize,
    count: usize,
) -> Result<(), MdlWriteError> {
    write_u32(bytes, offset, as_u32(pointer, "layout.arrayPointer")?)?;
    write_u32(bytes, offset + 4, as_u32(count, "layout.arrayCount")?)?;
    write_u32(bytes, offset + 8, as_u32(count, "layout.arrayCount")?)
}

fn write_controller_key(
    bytes: &mut [u8],
    offset: usize,
    kind: i32,
    time: i16,
    data: i16,
    columns: i8,
) -> Result<(), MdlWriteError> {
    write_i32(bytes, offset, kind)?;
    write_i16(bytes, offset + 4, 1)?;
    write_i16(bytes, offset + 6, time)?;
    write_i16(bytes, offset + 8, data)?;
    let target = bytes.get_mut(offset + 10).ok_or_else(|| {
        error(
            "M4-LAYOUT-OVERFLOW",
            "payload",
            "controller key escapes buffer",
        )
    })?;
    *target = columns as u8;
    Ok(())
}

fn write_c_string(
    bytes: &mut [u8],
    offset: usize,
    capacity: usize,
    value: &str,
) -> Result<(), MdlWriteError> {
    if value.len() >= capacity {
        return Err(error(
            "M4-INVALID-NAME",
            "payload.string",
            "C string exceeds field capacity",
        ));
    }
    let target = bytes
        .get_mut(offset..offset + value.len())
        .ok_or_else(|| error("M4-LAYOUT-OVERFLOW", "payload", "string escapes buffer"))?;
    target.copy_from_slice(value.as_bytes());
    Ok(())
}

fn write_vec3(bytes: &mut [u8], offset: usize, value: [f32; 3]) -> Result<(), MdlWriteError> {
    write_f32(bytes, offset, value[0])?;
    write_f32(bytes, offset + 4, value[1])?;
    write_f32(bytes, offset + 8, value[2])
}

fn write_f32(bytes: &mut [u8], offset: usize, value: f32) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_i32(bytes: &mut [u8], offset: usize, value: i32) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_i16(bytes: &mut [u8], offset: usize, value: i16) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_fixed<const N: usize>(
    bytes: &mut [u8],
    offset: usize,
    value: [u8; N],
) -> Result<(), MdlWriteError> {
    let target = bytes.get_mut(offset..offset + N).ok_or_else(|| {
        error(
            "M4-LAYOUT-OVERFLOW",
            "payload",
            "write escapes planned buffer",
        )
    })?;
    target.copy_from_slice(&value);
    Ok(())
}

fn zeroed(length: usize, path: &str) -> Result<Vec<u8>, MdlWriteError> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| {
        error(
            "M4-LAYOUT-OVERFLOW",
            path,
            "planned buffer allocation failed",
        )
    })?;
    bytes.resize(length, 0);
    Ok(bytes)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn length3(value: [f32; 3]) -> f32 {
    dot(value, value).sqrt()
}

fn checked_length3(value: [f32; 3]) -> Option<f32> {
    if !finite3(value) {
        return None;
    }
    let squared =
        f64::from(value[0]).powi(2) + f64::from(value[1]).powi(2) + f64::from(value[2]).powi(2);
    let length = squared.sqrt();
    (length.is_finite() && length <= f64::from(f32::MAX)).then_some(length as f32)
}

fn checked_face_plane(
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    path: &str,
    degeneracy_policy: FacePlaneDegeneracyPolicyV1,
) -> Result<([f32; 3], f32), MdlWriteError> {
    let edge_ab = [
        f64::from(b[0]) - f64::from(a[0]),
        f64::from(b[1]) - f64::from(a[1]),
        f64::from(b[2]) - f64::from(a[2]),
    ];
    let edge_ac = [
        f64::from(c[0]) - f64::from(a[0]),
        f64::from(c[1]) - f64::from(a[1]),
        f64::from(c[2]) - f64::from(a[2]),
    ];
    let cross = [
        edge_ab[1] * edge_ac[2] - edge_ab[2] * edge_ac[1],
        edge_ab[2] * edge_ac[0] - edge_ab[0] * edge_ac[2],
        edge_ab[0] * edge_ac[1] - edge_ab[1] * edge_ac[0],
    ];
    let length = (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt();
    let accepted = match degeneracy_policy {
        FacePlaneDegeneracyPolicyV1::LegacyAbsoluteEpsilon => {
            length.is_finite() && length > f64::from(EPSILON)
        }
        FacePlaneDegeneracyPolicyV1::ExactFiniteNonCollinear => length.is_finite() && length > 0.0,
    };
    if !accepted {
        return Err(error(
            "M4-MESH-INVALID",
            path,
            "triangle cannot produce a finite non-degenerate face plane",
        ));
    }
    let normal = [
        (cross[0] / length) as f32,
        (cross[1] / length) as f32,
        (cross[2] / length) as f32,
    ];
    let distance = -(f64::from(normal[0]) * f64::from(a[0])
        + f64::from(normal[1]) * f64::from(a[1])
        + f64::from(normal[2]) * f64::from(a[2]));
    if !finite3(normal) || !distance.is_finite() || distance.abs() > f64::from(f32::MAX) {
        return Err(error(
            "M4-MESH-INVALID",
            path,
            "face plane overflowed to a non-finite value",
        ));
    }
    Ok((normal, distance as f32))
}

type FaceAdjacencyPositionKey = [u32; 3];
type FaceAdjacencyEdgeKey = (FaceAdjacencyPositionKey, FaceAdjacencyPositionKey);

#[derive(Clone, Copy)]
enum FaceAdjacencyEdgeUses {
    One((usize, usize)),
    Two((usize, usize), (usize, usize)),
    NonManifold,
}

fn face_adjacency_position_key(position: [f32; 3]) -> FaceAdjacencyPositionKey {
    position.map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}

fn checked_face_adjacency(
    positions: &[[f32; 3]],
    indices: &[u32],
    path: &str,
) -> Result<Vec<[i16; 3]>, MdlWriteError> {
    if !indices.len().is_multiple_of(3) {
        return Err(error(
            "M4-MESH-INVALID",
            path,
            "triangle index count must be divisible by three",
        ));
    }
    let face_count = indices.len() / 3;
    if face_count > i16::MAX as usize {
        return Err(error(
            "M4-MESH-INVALID",
            path,
            "face adjacency indices exceed the signed 16-bit face range",
        ));
    }

    let mut adjacency = vec![[-1_i16; 3]; face_count];
    let mut edge_uses = HashMap::<FaceAdjacencyEdgeKey, FaceAdjacencyEdgeUses>::new();
    for (face_index, triangle) in indices.chunks_exact(3).enumerate() {
        let edges = [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ];
        for (edge_index, (start, end)) in edges.into_iter().enumerate() {
            let start_position = positions.get(start as usize).copied().ok_or_else(|| {
                error(
                    "M4-MESH-INVALID",
                    path,
                    format!(
                        "face {face_index} edge {edge_index} start index {start} is out of bounds"
                    ),
                )
            })?;
            let end_position = positions.get(end as usize).copied().ok_or_else(|| {
                error(
                    "M4-MESH-INVALID",
                    path,
                    format!("face {face_index} edge {edge_index} end index {end} is out of bounds"),
                )
            })?;
            let start_key = face_adjacency_position_key(start_position);
            let end_key = face_adjacency_position_key(end_position);
            if start_key == end_key {
                return Err(error(
                    "M4-MESH-INVALID",
                    path,
                    format!(
                        "face {face_index} edge {edge_index} collapses to one geometric position"
                    ),
                ));
            }
            let key = if start_key < end_key {
                (start_key, end_key)
            } else {
                (end_key, start_key)
            };
            use std::collections::hash_map::Entry;
            match edge_uses.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(FaceAdjacencyEdgeUses::One((face_index, edge_index)));
                }
                Entry::Occupied(mut entry) => {
                    let next = match *entry.get() {
                        FaceAdjacencyEdgeUses::One(first) => {
                            FaceAdjacencyEdgeUses::Two(first, (face_index, edge_index))
                        }
                        FaceAdjacencyEdgeUses::Two(_, _) | FaceAdjacencyEdgeUses::NonManifold => {
                            FaceAdjacencyEdgeUses::NonManifold
                        }
                    };
                    entry.insert(next);
                }
            }
        }
    }
    for uses in edge_uses.into_values() {
        if let FaceAdjacencyEdgeUses::Two((first_face, first_edge), (second_face, second_edge)) =
            uses
        {
            adjacency[first_face][first_edge] = second_face as i16;
            adjacency[second_face][second_edge] = first_face as i16;
        }
    }
    Ok(adjacency)
}

fn face_adjacency_for_profile(
    positions: &[[f32; 3]],
    indices: &[u32],
    profile: MdlFormatProfileV1,
    path: &str,
) -> Result<Vec<[i16; 3]>, MdlWriteError> {
    if matches!(
        profile,
        MdlFormatProfileV1::PlaceableStaticRigidNativeV1
            | MdlFormatProfileV1::ItemPartStaticRigidNativeV1
            | MdlFormatProfileV1::TileStaticV1
    ) {
        checked_face_adjacency(positions, indices, path)
    } else {
        Ok(vec![[-1_i16; 3]; indices.len() / 3])
    }
}

fn finite3(value: [f32; 3]) -> bool {
    value.iter().all(|item| item.is_finite())
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

fn mul_mat4_f64(a: [f64; 16], b: [f64; 16]) -> [f64; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    output
}

fn transform_point(matrix: [f32; 16], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use crate::profile_a::{
        AuroraCreatureIrV1, AuroraCreatureNodeV1, AuroraCreatureSegmentV1, AuroraVertexWeightsV1,
        MaterialSourceBindingV1, RigSegmentDeformationV1,
    };

    use super::super::writer_types::{
        MdlAnimationClipV1, MdlAnimationEventV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
        MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlFormatProfileV1,
        MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1, MdlWriterOptionsV1,
    };
    use super::{
        ANIMATION_EVENT_SIZE, CONTROLLER_KEY_SIZE, add, as_i32, expected_readback,
        inspect_binary_mdl, mul, plan, semantic_diff, validate_skin_layout,
        validate_skin_signed_fields, write_binary_mdl, write_binary_mdl_with_animations,
        write_binary_mdl_with_animations_exact_face_planes_v1,
    };

    #[test]
    fn checked_layout_helpers_return_the_stable_overflow_code() {
        for error in [
            add(usize::MAX, 1, "test").unwrap_err(),
            mul(usize::MAX, 2, "test").unwrap_err(),
            as_i32(usize::MAX, "test").unwrap_err(),
        ] {
            assert_eq!(error.code, "M4-LAYOUT-OVERFLOW");
            assert_eq!(error.path, "test");
        }
    }

    #[test]
    fn semantic_skin_mutations_reject_equivalent_q_sign_and_reader_permitted_drift() {
        let input = skin_input();
        let options = skin_options();
        let artifact = write_binary_mdl(&input, &options).unwrap();
        let animations = MdlAnimationSetV1::empty();
        let plan = plan(&input, &animations, &options, None).unwrap();
        let expected =
            expected_readback(&input, &animations, &options, "NULL", &plan, None).unwrap();
        let skin_node = artifact.inspection.node_tree.roots[0]
            .children
            .last()
            .unwrap();
        let skin = skin_node.skin.as_ref().unwrap();
        let node_absolute = 12 + skin.node_offset as usize;

        let mut q_sign = artifact.payload.clone();
        for lane in 0..4 {
            let absolute = 12 + skin.q_header.pointer as usize + lane * 4;
            let value = f32::from_le_bytes(q_sign[absolute..absolute + 4].try_into().unwrap());
            q_sign[absolute..absolute + 4].copy_from_slice(&(-value).to_le_bytes());
        }
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&q_sign).unwrap()),
            ["nodes[3].skin.qInverse"]
        );

        let mut mesh_type = artifact.payload.clone();
        mesh_type[node_absolute + 0x224..node_absolute + 0x228]
            .copy_from_slice(&0_u32.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&mesh_type).unwrap()),
            ["nodes[3].profileDefaults"]
        );

        let mut constant = artifact.payload.clone();
        let constant_absolute = 12 + skin.constants_header.pointer as usize;
        constant[constant_absolute + 2..constant_absolute + 4]
            .copy_from_slice(&1_u16.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&constant).unwrap()),
            ["nodes[3].skin.constants"]
        );

        let mut translation = artifact.payload.clone();
        let translation_absolute = 12 + skin.t_header.pointer as usize;
        translation[translation_absolute..translation_absolute + 4]
            .copy_from_slice(&0.001_f32.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&translation).unwrap()),
            ["nodes[3].skin.tInverse"]
        );

        let mut unused_inline = artifact.payload.clone();
        unused_inline[node_absolute + 0x2b0 + 4..node_absolute + 0x2b0 + 6]
            .copy_from_slice(&0_i16.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&unused_inline).unwrap()),
            ["nodes[3].skin.inlineReverse"]
        );

        let mut metadata = artifact.payload.clone();
        metadata[node_absolute + 0x270..node_absolute + 0x274]
            .copy_from_slice(&1_u32.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&metadata).unwrap()),
            ["nodes[3].skin.weightsMetadata"]
        );

        let raw_start = artifact.inspection.file_header.raw_range.start;
        let weights_pointer = u32::from_le_bytes(
            artifact.payload[node_absolute + 0x27c..node_absolute + 0x280]
                .try_into()
                .unwrap(),
        ) as usize;
        let mut weight = artifact.payload.clone();
        let weight_absolute = raw_start + weights_pointer;
        let value = f32::from_le_bytes(
            weight[weight_absolute..weight_absolute + 4]
                .try_into()
                .unwrap(),
        );
        weight[weight_absolute..weight_absolute + 4]
            .copy_from_slice(&value.next_up().to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&weight).unwrap()),
            ["nodes[3].skin.vertexWeights"]
        );

        let mut negative_zero = artifact.payload.clone();
        negative_zero[weight_absolute + 8..weight_absolute + 12]
            .copy_from_slice(&(-0.0_f32).to_bits().to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&negative_zero).unwrap()),
            ["nodes[3].skin.vertexWeights"]
        );

        let refs_pointer = u32::from_le_bytes(
            artifact.payload[node_absolute + 0x280..node_absolute + 0x284]
                .try_into()
                .unwrap(),
        ) as usize;
        let mut inactive_ref = artifact.payload.clone();
        let inactive_absolute = raw_start + refs_pointer + 4;
        inactive_ref[inactive_absolute..inactive_absolute + 2]
            .copy_from_slice(&0_u16.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&inactive_ref).unwrap()),
            ["nodes[3].skin.vertexRefs", "nodes[3].skin.slotResolution"]
        );

        let mut forward = artifact.payload.clone();
        let forward_absolute = 12 + skin.node_to_bone_pointer as usize + 2;
        forward[forward_absolute..forward_absolute + 2].copy_from_slice(&1_i16.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&forward).unwrap()),
            ["nodes[3].skin.forwardMap", "nodes[3].skin.slotResolution"]
        );

        let mut flags = artifact.payload.clone();
        flags[node_absolute + 0x6c..node_absolute + 0x70].copy_from_slice(&0x21_u32.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&flags).unwrap()),
            ["nodes[3].nodeDefaults", "nodes[3].skin"]
        );
    }

    #[test]
    fn semantic_animation_mutations_name_opaque_budget_and_padding() {
        let input = skin_input();
        let options = skin_options();
        let animations = MdlAnimationSetV1 {
            schema_version: 1,
            clips: vec![MdlAnimationClipV1 {
                name: "cpause1".to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: vec![MdlAnimationEventV1 {
                    time_seconds: 0.5,
                    name: "owned_event".to_owned(),
                }],
                tracks: vec![
                    MdlAnimationTrackV1 {
                        target_node_id: 10,
                        path: MdlAnimationTrackPathV1::Translation,
                        interpolation: MdlAnimationInterpolationV1::Linear,
                        times_seconds: vec![0.0, 1.0],
                        values: vec![vec![0.0, 0.0, 0.0], vec![0.25, 0.0, 0.0]],
                    },
                    MdlAnimationTrackV1 {
                        target_node_id: 10,
                        path: MdlAnimationTrackPathV1::Rotation,
                        interpolation: MdlAnimationInterpolationV1::Linear,
                        times_seconds: vec![0.0, 1.0],
                        values: vec![vec![0.0, 0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0, 1.0]],
                    },
                ],
            }],
        };
        let artifact = write_binary_mdl_with_animations(&input, &animations, &options).unwrap();
        assert_eq!(
            artifact
                .report
                .deviations
                .iter()
                .filter(|deviation| deviation.code.starts_with("M4A-"))
                .map(|deviation| (deviation.code.as_str(), deviation.path.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "M4A-RUNTIME-FIELD-68-OPAQUE-ZERO-OPEN-M6",
                    "animations.runtimeField68"
                ),
                (
                    "M4A-RUNTIME-ANIM-TREE-PROFILE-OPEN-M6",
                    "animations.nodeTrees"
                ),
                (
                    "M4A-DECOMP-ANIMROOT-CONSUMER-OPEN-M6",
                    "animations.animroot"
                ),
                (
                    "M4A-DECOMP-EVENT-NAME-SEMANTICS-OPEN-M6",
                    "animations.events.names"
                ),
                (
                    "M4A-RUNTIME-STATE-ROUTING-OPEN-M6",
                    "animations.stateRouting"
                ),
            ]
        );
        let layout = plan(&input, &animations, &options, None).unwrap();
        let expected =
            expected_readback(&input, &animations, &options, "NULL", &layout, None).unwrap();
        let clip = &artifact.inspection.animations[0];
        let root = &clip.node_tree.roots[0];
        let absolute = |offset: u32| 12 + offset as usize;
        fn relocate_core_array(
            payload: &[u8],
            source_core_offset: u32,
            byte_length: usize,
            pointer_absolute: usize,
        ) -> Vec<u8> {
            let mut relocated = payload.to_vec();
            let old_core_length = u32::from_le_bytes(relocated[4..8].try_into().unwrap()) as usize;
            let source = 12 + source_core_offset as usize;
            let copy = relocated[source..source + byte_length].to_vec();
            relocated.splice(12 + old_core_length..12 + old_core_length, copy);
            relocated[4..8]
                .copy_from_slice(&(old_core_length as u32 + byte_length as u32).to_le_bytes());
            relocated[pointer_absolute..pointer_absolute + 4]
                .copy_from_slice(&(old_core_length as u32).to_le_bytes());
            relocated
        }

        let relocated_events = relocate_core_array(
            &artifact.payload,
            clip.events_header.pointer,
            clip.events_header.used * ANIMATION_EVENT_SIZE,
            absolute(clip.offset) + 0xb8,
        );
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&relocated_events).unwrap()),
            ["animations[0].eventsHeader"]
        );

        let relocated_children = relocate_core_array(
            &artifact.payload,
            root.children_header.pointer,
            root.children_header.used * 4,
            absolute(root.offset) + 0x48,
        );
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&relocated_children).unwrap()),
            ["animations[0].nodes[0].children"]
        );

        let relocated_keys = relocate_core_array(
            &artifact.payload,
            root.controller_keys_header.pointer,
            root.controller_keys_header.used * CONTROLLER_KEY_SIZE,
            absolute(root.offset) + 0x54,
        );
        let key_diff = semantic_diff(&expected, &inspect_binary_mdl(&relocated_keys).unwrap());
        assert!(key_diff.contains(&"animations[0].nodes[0].controllerKeys".to_owned()));
        assert!(key_diff.contains(&"animations[0].nodes[0].controllers[0]".to_owned()));

        let relocated_data = relocate_core_array(
            &artifact.payload,
            root.controller_data_header.pointer,
            root.controller_data_header.used * 4,
            absolute(root.offset) + 0x60,
        );
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&relocated_data).unwrap()),
            ["animations[0].nodes[0].controllerData"]
        );

        let mut opaque = artifact.payload.clone();
        opaque[absolute(clip.offset) + 0x68..absolute(clip.offset) + 0x6c]
            .copy_from_slice(&1_u32.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&opaque).unwrap()),
            ["animations[0].header"]
        );

        let mut animation_type = artifact.payload.clone();
        animation_type[absolute(clip.offset) + 0x6c..absolute(clip.offset) + 0x70]
            .copy_from_slice(&0_u32.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&animation_type).unwrap()),
            ["animations[0].header"]
        );

        let mut animroot = artifact.payload.clone();
        animroot[absolute(clip.offset) + 0x78] = b'x';
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&animroot).unwrap()),
            ["animations[0].header"]
        );

        let mut budget = artifact.payload.clone();
        budget[absolute(clip.offset) + 0x4c..absolute(clip.offset) + 0x50]
            .copy_from_slice(&(clip.node_tree.declared_node_count as u32 + 1).to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&budget).unwrap()),
            ["animations[0].nodes.countOrRoot"]
        );

        let mut name = artifact.payload.clone();
        name[absolute(root.offset) + 0x20] = b'x';
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&name).unwrap()),
            ["animations[0].nodes[0].header"]
        );

        let mut padding = artifact.payload.clone();
        let key = absolute(root.controllers[0].key_offset);
        padding[key + 11] = 1;
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&padding).unwrap()),
            ["animations[0].nodes[0].controllers[0]"]
        );

        let translation = &root.controllers[0];
        let first_time = absolute(root.controller_data_header.pointer) + translation.time_index * 4;
        let second_time = first_time + 4;
        let mut nonfinite_time = artifact.payload.clone();
        nonfinite_time[first_time..first_time + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&nonfinite_time).unwrap()),
            ["animations[0].nodes[0].controllers[0]"]
        );
        let mut unordered_time = artifact.payload.clone();
        let first_time_bits = unordered_time[first_time..first_time + 4].to_vec();
        unordered_time[second_time..second_time + 4].copy_from_slice(&first_time_bits);
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&unordered_time).unwrap()),
            ["animations[0].nodes[0].controllers[0]"]
        );
        let mut changed_value = artifact.payload.clone();
        let first_value =
            absolute(root.controller_data_header.pointer) + translation.data_index * 4;
        let value = f32::from_le_bytes(
            changed_value[first_value..first_value + 4]
                .try_into()
                .unwrap(),
        );
        changed_value[first_value..first_value + 4].copy_from_slice(&value.next_up().to_le_bytes());
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&changed_value).unwrap()),
            ["animations[0].nodes[0].controllers[0]"]
        );

        let mut quaternion_sign = artifact.payload.clone();
        let rotation = &root.controllers[1];
        let first_value = absolute(root.controller_data_header.pointer) + rotation.data_index * 4;
        for row in 0..rotation.row_count {
            for lane in 0..4 {
                let offset = first_value + (row * 4 + lane) * 4;
                let value =
                    f32::from_le_bytes(quaternion_sign[offset..offset + 4].try_into().unwrap());
                quaternion_sign[offset..offset + 4].copy_from_slice(&(-value).to_le_bytes());
            }
        }
        assert_eq!(
            semantic_diff(&expected, &inspect_binary_mdl(&quaternion_sign).unwrap()),
            ["animations[0].nodes[0].controllers[1]"]
        );
    }

    #[test]
    fn internal_skin_layout_drift_uses_the_stable_layout_error() {
        let input = skin_input();
        let mut layout = plan(&input, &MdlAnimationSetV1::empty(), &skin_options(), None).unwrap();
        layout.mesh[0].skin.as_mut().unwrap().forward_offset += 4;
        let error = validate_skin_layout(&layout.mesh, input.nodes.len() + input.segments.len())
            .unwrap_err();
        assert_eq!(error.code, "M4-SKIN-LAYOUT-INVALID");
        assert_eq!(error.path, "layout.skinNodes[0]");
    }

    #[test]
    fn signed_skin_fields_are_rejected_by_preallocation_validation() {
        let input = skin_input();
        let mut layout = plan(&input, &MdlAnimationSetV1::empty(), &skin_options(), None).unwrap();
        let over = i32::MAX as usize + 1;
        let error = validate_skin_signed_fields(&layout.mesh, over).unwrap_err();
        assert_eq!(error.code, "M4-LAYOUT-OVERFLOW");
        assert_eq!(error.path, "layout.skin.mapCount");

        layout.mesh[0].skin.as_mut().unwrap().forward_offset = over;
        let error =
            validate_skin_signed_fields(&layout.mesh, input.nodes.len() + input.segments.len())
                .unwrap_err();
        assert_eq!(error.code, "M4-LAYOUT-OVERFLOW");
        assert_eq!(error.path, "layout.skin.forwardMap");
    }

    #[test]
    fn exact_face_plane_writer_accepts_a_finite_micro_triangle_rejected_by_legacy_epsilon() {
        let mut input = skin_input();
        input.segments[0].positions = vec![[2.0, 0.0, 0.0], [2.01, 0.0, 0.0], [2.0, 0.001, 0.0]];
        let options = skin_options();

        let legacy = write_binary_mdl(&input, &options).unwrap_err();
        assert_eq!(legacy.code, "M4-MESH-INVALID");

        let artifact = write_binary_mdl_with_animations_exact_face_planes_v1(
            &input,
            &MdlAnimationSetV1::empty(),
            &options,
        )
        .unwrap();

        assert_eq!(artifact.report.projection.triangle_count, 1);
        assert!(artifact.report.semantic_diff.is_empty());
        assert_eq!(
            artifact.inspection.node_tree.roots[0].children[2]
                .mesh
                .as_ref()
                .unwrap()
                .faces
                .len(),
            1
        );
    }

    fn skin_input() -> AuroraCreatureIrV1 {
        let identity = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let nodes = vec![
            AuroraCreatureNodeV1 {
                id: 10,
                name: "root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity,
            },
            AuroraCreatureNodeV1 {
                id: 20,
                name: "bone_a".to_owned(),
                parent_id: Some(10),
                bind_local_matrix: identity,
            },
            AuroraCreatureNodeV1 {
                id: 30,
                name: "bone_b".to_owned(),
                parent_id: Some(10),
                bind_local_matrix: identity,
            },
        ];
        let rows = vec![
            AuroraVertexWeightsV1 {
                bone_node_ids: [Some(20), Some(30), None, None],
                values: [0.5, 0.5, 0.0, 0.0],
                influence_count: 2,
            };
            3
        ];
        AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "skin-unit".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
            engine_facing_proof: "OPEN_M6".to_owned(),
            uv_runtime_proof: "OPEN_M6".to_owned(),
            nodes,
            material_source_bindings: vec![MaterialSourceBindingV1 {
                slot: 0,
                source_material_id: None,
                source_material_name: None,
            }],
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Skin,
                parent_node_id: 10,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: rows,
            }],
        }
    }

    fn skin_options() -> MdlWriterOptionsV1 {
        MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "skin_unit".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "skin_tex".to_owned(),
            }],
        }
    }
}
