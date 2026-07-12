use std::collections::{HashMap, HashSet};

use sha2::{Digest, Sha256};

use crate::profile_a::{AuroraCreatureIrV1, AuroraCreatureSegmentV1, RigSegmentDeformationV1};

use super::inspect_binary_mdl;
use super::semantic_readback::{
    ExpectedFace, ExpectedMesh, ExpectedNode, ExpectedReadback, ExpectedSkin, semantic_diff,
};
use super::writer_types::{
    BinaryMdlArtifactV1, M4_WRITER_SCHEMA_VERSION, M4SemanticProjectionV1, MdlFormatProfileV1,
    MdlLayoutReportV1, MdlMeshNodeLayoutV1, MdlRigNodeLayoutV1, MdlWriteError,
    MdlWriterDeviationV1, MdlWriterOptionsV1, MdlWriterReportV1,
};

const FILE_HEADER_SIZE: usize = 0x0c;
const MODEL_HEADER_SIZE: usize = 0xe8;
const NODE_HEADER_SIZE: usize = 0x70;
const MESH_HEADER_SIZE: usize = 0x270;
const SKIN_HEADER_SIZE: usize = 0x330;
const SKIN_INLINE_COUNT: usize = 64;
const FACE_SIZE: usize = 0x20;
const CONTROLLER_KEY_SIZE: usize = 0x0c;
const CONTROLLER_KEY_COUNT: usize = 2;
const CONTROLLER_DATA_COUNT: usize = 9;
const EPSILON: f32 = 1.0e-5;

struct RigPlan {
    id: u32,
    part: u32,
    offset: usize,
    parent_part: Option<u32>,
    children_offsets: Vec<u32>,
    children_array: Option<usize>,
    keys: usize,
    data: usize,
    quaternion: [f32; 4],
}

struct MeshPlan {
    segment_index: usize,
    part: u32,
    offset: usize,
    parent_part: u32,
    faces: usize,
    index_count: usize,
    index_offset: usize,
    raw_positions: usize,
    raw_uv0: usize,
    raw_normals: usize,
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

struct Plan {
    core_length: usize,
    raw_length: usize,
    file_length: usize,
    root_offset: usize,
    rig: Vec<RigPlan>,
    mesh: Vec<MeshPlan>,
    model_bounds_min: [f32; 3],
    model_bounds_max: [f32; 3],
    model_radius: f32,
    textures: HashMap<u32, String>,
    deviations: Vec<MdlWriterDeviationV1>,
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
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<BinaryMdlArtifactV1, MdlWriteError> {
    let plan = plan(creature, options)?;
    let mut core = zeroed(plan.core_length, "layout.coreLength")?;
    let mut raw = zeroed(plan.raw_length, "layout.rawLength")?;
    emit_model(&mut core, creature, options, &plan)?;
    emit_nodes(&mut core, creature, &plan)?;
    emit_meshes(&mut core, &mut raw, creature, &plan)?;

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

    let inspection = inspect_binary_mdl(&payload).map_err(|source| {
        error(
            "M4-READBACK-FAILED",
            "payload",
            format!("own reader rejected emitted payload: {source}"),
        )
    })?;
    let expected = expected_readback(creature, options, &plan)?;
    let differences = semantic_diff(&expected, &inspection);
    if !differences.is_empty() {
        return Err(error(
            "M4-SEMANTIC-DIFF",
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
    let report = MdlWriterReportV1 {
        schema_version: M4_WRITER_SCHEMA_VERSION,
        format_profile: options.format_profile,
        payload_sha256,
        layout: MdlLayoutReportV1 {
            core_length: plan.core_length,
            raw_length: plan.raw_length,
            file_length: plan.file_length,
            rig_nodes,
            mesh_nodes,
        },
        projection: M4SemanticProjectionV1 {
            model_resource_resref: options.model_resource_resref.clone(),
            animation_count: 0,
            rig_node_count: plan.rig.len(),
            mesh_node_count: plan.mesh.len(),
            triangle_count,
        },
        semantic_diff: differences,
        deviations: plan.deviations,
    };
    Ok(BinaryMdlArtifactV1 {
        payload,
        inspection,
        report,
    })
}

fn plan(
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
) -> Result<Plan, MdlWriteError> {
    validate_public_contract(creature, options)?;
    let textures = validate_materials(creature, options)?;
    let node_count = creature.nodes.len();
    let total_nodes = add(node_count, creature.segments.len(), "creature.nodes")?;
    let _ = as_u32(total_nodes, "creature.nodes")?;

    let mut id_to_index = HashMap::with_capacity(node_count);
    for (index, node) in creature.nodes.iter().enumerate() {
        if id_to_index.insert(node.id, index).is_some() {
            return Err(error(
                "M4-HIERARCHY-INVALID",
                &format!("creature.nodes[{index}].id"),
                "duplicate node id",
            ));
        }
        validate_node_name(&node.name, &format!("creature.nodes[{index}].name"))?;
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
    for (index, node) in creature.nodes.iter().enumerate() {
        quaternions.push(matrix_quaternion(
            node.bind_local_matrix,
            &format!("creature.nodes[{index}].bindLocalMatrix"),
        )?);
    }

    let mut cursor = MODEL_HEADER_SIZE;
    let mut rig = Vec::with_capacity(node_count);
    for (index, node) in creature.nodes.iter().enumerate() {
        let offset = take(&mut cursor, NODE_HEADER_SIZE, "layout.rigNodes")?;
        rig.push(RigPlan {
            id: node.id,
            part: as_u32(index, "layout.partNumber")?,
            offset,
            parent_part: parent_indices[index]
                .map(|value| as_u32(value, "layout.parentPartNumber"))
                .transpose()?,
            children_offsets: Vec::new(),
            children_array: None,
            keys: 0,
            data: 0,
            quaternion: quaternions[index],
        });
    }

    let mut segment_ids = HashSet::new();
    let mut mesh = Vec::with_capacity(creature.segments.len());
    for (index, segment) in creature.segments.iter().enumerate() {
        validate_segment(segment, index, &id_to_index)?;
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
                inline_reverse: [-1; SKIN_INLINE_COUNT],
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
            parent_part: as_u32(parent_index, "layout.parentPartNumber")?,
            faces: 0,
            index_count: 0,
            index_offset: 0,
            raw_positions: 0,
            raw_uv0: 0,
            raw_normals: 0,
            raw_indices: 0,
            skin,
            bounds_min: metrics.bounds_min,
            bounds_max: metrics.bounds_max,
            radius: metrics.radius,
            average: metrics.average,
        });
    }

    for child_index in 0..rig.len() {
        if let Some(parent_index) = parent_indices[child_index] {
            let child_offset = as_u32(rig[child_index].offset, "layout.childOffset")?;
            rig[parent_index].children_offsets.push(child_offset);
        }
    }
    for item in &mesh {
        let child_offset = as_u32(item.offset, "layout.childOffset")?;
        rig[item.parent_part as usize]
            .children_offsets
            .push(child_offset);
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
        node.keys = take(
            &mut cursor,
            mul(
                CONTROLLER_KEY_COUNT,
                CONTROLLER_KEY_SIZE,
                "layout.controllerKeys",
            )?,
            "layout.controllerKeys",
        )?;
    }
    for node in &mut rig {
        node.data = take(
            &mut cursor,
            mul(CONTROLLER_DATA_COUNT, 4, "layout.controllerData")?,
            "layout.controllerData",
        )?;
    }
    for item in &mut mesh {
        let segment = &creature.segments[item.segment_index];
        item.faces = take(
            &mut cursor,
            mul(segment.indices.len() / 3, FACE_SIZE, "layout.faces")?,
            "layout.faces",
        )?;
    }
    for item in &mut mesh {
        item.index_count = take(&mut cursor, 4, "layout.indexCounts")?;
    }
    for item in &mut mesh {
        item.index_offset = take(&mut cursor, 4, "layout.indexOffsets")?;
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
    let raw_length = raw_cursor;
    let _ = as_u32(raw_length, "layout.rawLength")?;
    for item in &mesh {
        let _ = as_i32(item.raw_positions, "layout.rawPositions")?;
        let _ = as_i32(item.raw_uv0, "layout.rawUv0")?;
        let _ = as_i32(item.raw_normals, "layout.rawNormals")?;
        let _ = as_i32(item.raw_indices, "layout.rawIndices")?;
        if let Some(skin) = &item.skin {
            let _ = as_i32(skin.raw_weights, "layout.skin.rawWeights")?;
            let _ = as_i32(skin.raw_refs, "layout.skin.rawRefs")?;
        }
    }
    let file_length = add(
        add(FILE_HEADER_SIZE, core_length, "layout.fileLength")?,
        raw_length,
        "layout.fileLength",
    )?;

    let worlds = world_matrices(creature, &parent_indices)?;
    let (part_to_tree_ordinal, ordinal_parts) =
        tree_ordinals(rig[roots[0]].part as usize, &rig, &mesh)?;
    if mesh.iter().any(|item| item.skin.is_some()) {
        let rig_skin_worlds = world_matrices_f64(creature, &parent_indices, &quaternions)?;
        let mut binary_skin_worlds = Vec::with_capacity(total_nodes);
        binary_skin_worlds.extend_from_slice(&rig_skin_worlds);
        binary_skin_worlds.extend(
            mesh.iter()
                .map(|item| rig_skin_worlds[item.parent_part as usize]),
        );
        for item in &mut mesh {
            let Some(skin) = &mut item.skin else {
                continue;
            };
            build_skin_plan(
                skin,
                &creature.segments[item.segment_index],
                item.segment_index,
                item.parent_part as usize,
                &id_to_index,
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
        let world = worlds[item.parent_part as usize];
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
        deviations.extend([
            deviation("M4-SKIN-INLINE-UNUSED-OPEN-M6", "skin.inlineReverse.unused"),
            deviation("M4-SKIN-SLOT-BOUNDARY-OPEN-M6", "skin.activeSlots"),
            deviation("M4-SKIN-CONSTANTS-MEANING-OPEN-M6", "skin.boneConstants"),
            deviation("M4-SKIN-WXYZ-DEFORMATION-OPEN-M6", "skin.qInverse"),
            deviation(
                "M4-SKIN-VISUAL-DEFORMATION-OPEN-M6",
                "skin.runtimeDeformation",
            ),
        ]);
    }

    Ok(Plan {
        core_length,
        raw_length,
        file_length,
        root_offset: rig[roots[0]].offset,
        rig,
        mesh,
        model_bounds_min: model_min,
        model_bounds_max: model_max,
        model_radius,
        textures,
        deviations,
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
    if options.format_profile != MdlFormatProfileV1::M4DirectCreatureExtended64V1 {
        return Err(error(
            "M4-UNSUPPORTED-PROFILE",
            "options.formatProfile",
            "only M4_DIRECT_CREATURE_EXTENDED64_V1 is emitted",
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
        || segment.indices.len() > u32::MAX as usize
    {
        return Err(error(
            "M4-MESH-LIMIT",
            &path,
            "each rigid mesh is limited to non-empty u16 vertices and indices",
        ));
    }
    if segment.normals.len() != segment.positions.len()
        || segment.uv0.len() != segment.positions.len()
        || segment.indices.is_empty()
        || !segment.indices.len().is_multiple_of(3)
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
    plan: &Plan,
) -> Result<(), MdlWriteError> {
    write_c_string(core, 0x08, 64, &options.model_resource_resref)?;
    write_u32(core, 0x48, as_u32(plan.root_offset, "model.root")?)?;
    write_u32(
        core,
        0x4c,
        as_u32(
            add(plan.rig.len(), plan.mesh.len(), "model.nodeCount")?,
            "model.nodeCount",
        )?,
    )?;
    write_u32(core, 0x6c, 2)?;
    core[0x72] = 4;
    core[0x73] = 1;
    write_vec3(core, 0x88, plan.model_bounds_min)?;
    write_vec3(core, 0x94, plan.model_bounds_max)?;
    write_f32(core, 0xa0, plan.model_radius)?;
    write_f32(core, 0xa4, 1.0)?;
    write_c_string(core, 0xa8, 64, "null")?;
    let _ = creature;
    Ok(())
}

fn emit_nodes(
    core: &mut [u8],
    creature: &AuroraCreatureIrV1,
    plan: &Plan,
) -> Result<(), MdlWriteError> {
    for (index, item) in plan.rig.iter().enumerate() {
        let node = &creature.nodes[index];
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
        write_array(core, item.offset + 0x54, item.keys, CONTROLLER_KEY_COUNT)?;
        write_array(core, item.offset + 0x60, item.data, CONTROLLER_DATA_COUNT)?;
        write_u32(core, item.offset + 0x6c, 0x01)?;
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
    }
    Ok(())
}

fn emit_meshes(
    core: &mut [u8],
    raw: &mut [u8],
    creature: &AuroraCreatureIrV1,
    plan: &Plan,
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
        write_array(core, base + 0x78, item.faces, segment.indices.len() / 3)?;
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
        write_c_string(
            core,
            base + 0xe8,
            64,
            &plan.textures[&segment.material_slot],
        )?;
        write_array(core, base + 0x204, item.index_count, 1)?;
        write_array(core, base + 0x210, item.index_offset, 1)?;
        write_i32(core, base + 0x21c, -1)?;
        core[base + 0x224] = 3;
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
        for offset in [0x248, 0x24c, 0x250, 0x254, 0x258, 0x25c, 0x260] {
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
                write_i16(core, skin.constants_offset + index * 4, 0)?;
                write_i16(core, skin.constants_offset + index * 4 + 2, 0)?;
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
        for (face_index, triangle) in segment.indices.chunks_exact(3).enumerate() {
            let a = segment.positions[triangle[0] as usize];
            let (normal, distance) = checked_face_plane(
                a,
                segment.positions[triangle[1] as usize],
                segment.positions[triangle[2] as usize],
                &format!("creature.segments[{}].indices", item.segment_index),
            )?;
            let face = item.faces + face_index * FACE_SIZE;
            write_vec3(core, face, normal)?;
            write_f32(core, face + 0x0c, distance)?;
            write_i32(core, face + 0x10, 0)?;
            for offset in [0x14, 0x16, 0x18] {
                write_i16(core, face + offset, -1)?;
            }
            write_u16(core, face + 0x1a, triangle[0] as u16)?;
            write_u16(core, face + 0x1c, triangle[1] as u16)?;
            write_u16(core, face + 0x1e, triangle[2] as u16)?;
        }
    }
    Ok(())
}

fn expected_readback(
    creature: &AuroraCreatureIrV1,
    options: &MdlWriterOptionsV1,
    plan: &Plan,
) -> Result<ExpectedReadback, MdlWriteError> {
    let mut nodes = Vec::with_capacity(plan.rig.len() + plan.mesh.len());
    for (index, item) in plan.rig.iter().enumerate() {
        nodes.push(ExpectedNode {
            ir_node_id: Some(item.id),
            part_number: item.part,
            name: creature.nodes[index].name.clone(),
            parent_part_number: item.parent_part,
            bind_matrix: Some(creature.nodes[index].bind_local_matrix),
            content_flags: 0x01,
            mesh: None,
            skin: None,
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
                    bone_constants: vec![[0, 0]; skin.forward.len()],
                    vertex_weights: skin.vertex_weights.clone(),
                    vertex_refs: skin.vertex_refs.clone(),
                    resolved_ir_ids: skin.resolved_ir_ids.clone(),
                })
            })
            .transpose()?;
        nodes.push(ExpectedNode {
            ir_node_id: None,
            part_number: item.part,
            name: format!("m2a_seg_{}", segment.segment_id),
            parent_part_number: Some(item.parent_part),
            bind_matrix: None,
            content_flags: if item.skin.is_some() { 0x61 } else { 0x21 },
            mesh: Some(ExpectedMesh {
                texture_resref: plan.textures[&segment.material_slot].clone(),
                positions: segment.positions.clone(),
                normals: segment.normals.clone(),
                uv0: segment.uv0.clone(),
                indices: segment.indices.iter().map(|value| *value as u16).collect(),
                bounds_min: item.bounds_min,
                bounds_max: item.bounds_max,
                radius: item.radius,
                average: item.average,
                raw_index_offset: as_i32(item.raw_indices, "mesh.indexOffset")?,
                faces: segment
                    .indices
                    .chunks_exact(3)
                    .map(|triangle| {
                        let (normal, distance) = checked_face_plane(
                            segment.positions[triangle[0] as usize],
                            segment.positions[triangle[1] as usize],
                            segment.positions[triangle[2] as usize],
                            "semantic.faces",
                        )?;
                        Ok(ExpectedFace {
                            normal,
                            distance,
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
        });
    }
    Ok(ExpectedReadback {
        model_name: options.model_resource_resref.clone(),
        model_bounds_min: plan.model_bounds_min,
        model_bounds_max: plan.model_bounds_max,
        model_radius: plan.model_radius,
        root_part_number: plan
            .rig
            .iter()
            .find(|item| item.offset == plan.root_offset)
            .unwrap()
            .part,
        nodes,
    })
}

fn tree_ordinals(
    root_part: usize,
    rig: &[RigPlan],
    mesh: &[MeshPlan],
) -> Result<(Vec<usize>, Vec<usize>), MdlWriteError> {
    let total = add(rig.len(), mesh.len(), "layout.treeOrdinals")?;
    let mut children = vec![Vec::<usize>::new(); rig.len()];
    for (child, node) in rig.iter().enumerate() {
        if let Some(parent) = node.parent_part {
            children[parent as usize].push(child);
        }
    }
    for (mesh_index, item) in mesh.iter().enumerate() {
        children[item.parent_part as usize].push(rig.len() + mesh_index);
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
    id_to_index: &HashMap<u32, usize>,
    part_to_tree_ordinal: &[usize],
    ordinal_parts: &[usize],
    binary_worlds: &[[f64; 16]],
) -> Result<(), MdlWriteError> {
    let mut active_ordinals = segment
        .weights
        .iter()
        .flat_map(|row| row.bone_node_ids.iter().flatten().copied())
        .map(|id| part_to_tree_ordinal[id_to_index[&id]])
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
            let ordinal = part_to_tree_ordinal[id_to_index[&id]];
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
    if !length.is_finite() || length <= f64::from(EPSILON) {
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
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlWriterOptionsV1,
    };
    use super::{
        add, as_i32, expected_readback, inspect_binary_mdl, mul, plan, semantic_diff,
        validate_skin_layout, validate_skin_signed_fields, write_binary_mdl,
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
        let plan = plan(&input, &options).unwrap();
        let expected = expected_readback(&input, &options, &plan).unwrap();
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

        let mut constant = artifact.payload.clone();
        let constant_absolute = 12 + skin.constants_header.pointer as usize;
        constant[constant_absolute..constant_absolute + 2].copy_from_slice(&1_i16.to_le_bytes());
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
    fn internal_skin_layout_drift_uses_the_stable_layout_error() {
        let input = skin_input();
        let mut layout = plan(&input, &skin_options()).unwrap();
        layout.mesh[0].skin.as_mut().unwrap().forward_offset += 4;
        let error = validate_skin_layout(&layout.mesh, input.nodes.len() + input.segments.len())
            .unwrap_err();
        assert_eq!(error.code, "M4-SKIN-LAYOUT-INVALID");
        assert_eq!(error.path, "layout.skinNodes[0]");
    }

    #[test]
    fn signed_skin_fields_are_rejected_by_preallocation_validation() {
        let input = skin_input();
        let mut layout = plan(&input, &skin_options()).unwrap();
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
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                indices: vec![0, 1, 2],
                weights: rows,
            }],
        }
    }

    fn skin_options() -> MdlWriterOptionsV1 {
        MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            model_resource_resref: "skin_unit".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "skin_tex".to_owned(),
            }],
        }
    }
}
