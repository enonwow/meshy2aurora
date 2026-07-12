use std::collections::HashMap;

use super::types::{InspectionReport, MeshReport, NodeReport, SkinReport, SkinVariant, Vec2, Vec3};

pub(crate) struct ExpectedReadback {
    pub model_name: String,
    pub model_bounds_min: [f32; 3],
    pub model_bounds_max: [f32; 3],
    pub model_radius: f32,
    pub root_part_number: u32,
    pub nodes: Vec<ExpectedNode>,
}

pub(crate) struct ExpectedNode {
    pub ir_node_id: Option<u32>,
    pub part_number: u32,
    pub name: String,
    pub parent_part_number: Option<u32>,
    pub bind_matrix: Option<[f32; 16]>,
    pub content_flags: u32,
    pub mesh: Option<ExpectedMesh>,
    pub skin: Option<ExpectedSkin>,
}

pub(crate) struct ExpectedSkin {
    pub raw_weights_pointer: i32,
    pub raw_refs_pointer: i32,
    pub q_pointer: u32,
    pub t_pointer: u32,
    pub constants_pointer: u32,
    pub forward: Vec<i16>,
    pub inline_reverse: Vec<i16>,
    pub inverse_rotations_wxyz: Vec<[f32; 4]>,
    pub inverse_translations: Vec<[f32; 3]>,
    pub bone_constants: Vec<[i16; 2]>,
    pub vertex_weights: Vec<[f32; 4]>,
    pub vertex_refs: Vec<[u16; 4]>,
    pub resolved_ir_ids: Vec<[Option<u32>; 4]>,
}

pub(crate) struct ExpectedMesh {
    pub texture_resref: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uv0: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub radius: f32,
    pub average: [f32; 3],
    pub raw_index_offset: i32,
    pub faces: Vec<ExpectedFace>,
}

pub(crate) struct ExpectedFace {
    pub normal: [f32; 3],
    pub distance: f32,
    pub vertex_indices: [u16; 3],
}

pub(crate) fn semantic_diff(expected: &ExpectedReadback, actual: &InspectionReport) -> Vec<String> {
    let mut diff = Vec::new();
    if actual.model.name != expected.model_name || actual.model.geometry_type != 2 {
        diff.push("model.name".to_owned());
    }
    if !actual.animations.is_empty() {
        diff.push("model.animations".to_owned());
    }
    if actual.byte_length != actual.file_header.raw_range.end {
        diff.push("file.exactEof".to_owned());
    }
    if !actual.diagnostics.is_empty() {
        diff.push("diagnostics".to_owned());
    }
    if !actual.unsupported.is_empty() {
        diff.push("unsupported".to_owned());
    }
    compare_vec3(
        "model.boundsMin",
        expected.model_bounds_min,
        actual.model.bounds_min,
        &mut diff,
    );
    compare_vec3(
        "model.boundsMax",
        expected.model_bounds_max,
        actual.model.bounds_max,
        &mut diff,
    );
    compare_f32(
        "model.radius",
        expected.model_radius,
        actual.model.radius,
        &mut diff,
    );
    if actual.model.classification != 4
        || actual.model.fog != 1
        || actual.model.child_model_count != 0
        || actual.model.animation_scale != 1.0
        || actual.model.supermodel_name != "null"
    {
        diff.push("model.profileDefaults".to_owned());
    }

    let mut flat = Vec::new();
    for root in &actual.node_tree.roots {
        flatten(root, &mut flat);
    }
    if flat.len() != expected.nodes.len() || actual.node_tree.node_count != expected.nodes.len() {
        diff.push("nodes.count".to_owned());
        return diff;
    }
    let by_number = flat
        .iter()
        .map(|node| (node.number, *node))
        .collect::<HashMap<_, _>>();
    let offset_to_number = flat
        .iter()
        .map(|node| (node.offset, node.number))
        .collect::<HashMap<_, _>>();
    let part_to_ir = expected
        .nodes
        .iter()
        .filter_map(|node| node.ir_node_id.map(|id| (node.part_number, id)))
        .collect::<HashMap<_, _>>();
    if actual.node_tree.roots.len() != 1
        || actual.node_tree.roots[0].number != expected.root_part_number
    {
        diff.push("nodes.root".to_owned());
    }

    for expected_node in &expected.nodes {
        let path = format!("nodes[{}]", expected_node.part_number);
        let Some(actual_node) = by_number.get(&expected_node.part_number) else {
            diff.push(format!("{path}.missing"));
            continue;
        };
        if actual_node.name != expected_node.name {
            diff.push(format!("{path}.name"));
        }
        if actual_node.inherit_color != 0
            || actual_node.content_flags != expected_node.content_flags
        {
            diff.push(format!("{path}.nodeDefaults"));
        }
        let actual_parent = actual_node
            .parent_offset
            .and_then(|offset| offset_to_number.get(&offset).copied());
        if actual_parent != expected_node.parent_part_number {
            diff.push(format!("{path}.parent"));
        }
        match (&expected_node.bind_matrix, &expected_node.mesh) {
            (Some(matrix), None) => {
                if expected_node.skin.is_some() {
                    diff.push(format!("{path}.kind"));
                }
                compare_bind_matrix(&path, *matrix, actual_node, &mut diff)
            }
            (None, Some(mesh)) => {
                if !actual_node.controllers.is_empty() {
                    diff.push(format!("{path}.controllers"));
                }
                let Some(actual_mesh) = actual_node.mesh.as_ref() else {
                    diff.push(format!("{path}.mesh"));
                    continue;
                };
                compare_mesh(&path, mesh, actual_mesh, &mut diff);
                match (&expected_node.skin, &actual_node.skin) {
                    (Some(expected_skin), Some(actual_skin)) => compare_skin(
                        &path,
                        expected_skin,
                        actual_skin,
                        &flat,
                        &part_to_ir,
                        &mut diff,
                    ),
                    (None, None) => {}
                    _ => diff.push(format!("{path}.skin")),
                }
            }
            _ => diff.push(format!("{path}.kind")),
        }
    }
    diff
}

fn compare_skin(
    path: &str,
    expected: &ExpectedSkin,
    actual: &SkinReport,
    preorder: &[&NodeReport],
    part_to_ir: &HashMap<u32, u32>,
    diff: &mut Vec<String>,
) {
    let skin_path = format!("{path}.skin");
    if actual.variant != SkinVariant::Extended64
        || actual.header_size != 0x330
        || actual.node_to_bone_pointer != actual.node_offset + 0x330
    {
        diff.push(format!("{skin_path}.layout"));
    }
    if actual.raw_weights_pointer != expected.raw_weights_pointer
        || actual.raw_refs_pointer != expected.raw_refs_pointer
    {
        diff.push(format!("{skin_path}.rawPointers"));
    }
    if actual.weights_header.pointer != 0
        || actual.weights_header.used != 0
        || actual.weights_header.allocated != 0
    {
        diff.push(format!("{skin_path}.weightsMetadata"));
    }
    if actual.q_header.used != expected.forward.len()
        || actual.q_header.allocated != expected.forward.len()
        || actual.t_header.used != expected.forward.len()
        || actual.t_header.allocated != expected.forward.len()
        || actual.constants_header.used != expected.forward.len()
        || actual.constants_header.allocated != expected.forward.len()
        || actual.q_header.pointer != expected.q_pointer
        || actual.t_header.pointer != expected.t_pointer
        || actual.constants_header.pointer != expected.constants_pointer
    {
        diff.push(format!("{skin_path}.arrayCounts"));
    }
    if actual.node_to_bone_map != expected.forward {
        diff.push(format!("{skin_path}.forwardMap"));
    }
    if actual.inline_mapping != expected.inline_reverse {
        diff.push(format!("{skin_path}.inlineReverse"));
    }
    if actual.bone_constants != expected.bone_constants {
        diff.push(format!("{skin_path}.constants"));
    }
    if actual.inverse_bone_rotations_raw.len() != expected.inverse_rotations_wxyz.len()
        || actual
            .inverse_bone_rotations_raw
            .iter()
            .zip(&expected.inverse_rotations_wxyz)
            .any(|(actual, expected)| {
                actual
                    .iter()
                    .zip(expected)
                    .any(|(a, e)| !finite_approx(*e, *a))
            })
    {
        diff.push(format!("{skin_path}.qInverse"));
    }
    if actual.inverse_bone_translations.len() != expected.inverse_translations.len()
        || actual
            .inverse_bone_translations
            .iter()
            .zip(&expected.inverse_translations)
            .any(|(actual, expected)| !vec3_approx(*expected, *actual))
    {
        diff.push(format!("{skin_path}.tInverse"));
    }
    if actual.vertex_weights.len() != expected.vertex_weights.len()
        || actual
            .vertex_weights
            .iter()
            .zip(&expected.vertex_weights)
            .any(|(actual, expected)| {
                actual
                    .iter()
                    .zip(expected)
                    .any(|(actual, expected)| actual.to_bits() != expected.to_bits())
            })
    {
        diff.push(format!("{skin_path}.vertexWeights"));
    }
    if actual.bone_references != expected.vertex_refs {
        diff.push(format!("{skin_path}.vertexRefs"));
    }

    let mut resolved = Vec::with_capacity(actual.bone_references.len());
    let mut resolution_valid = true;
    for (weights, refs) in actual.vertex_weights.iter().zip(&actual.bone_references) {
        let mut row = [None; 4];
        for lane in 0..4 {
            if weights[lane] == 0.0 {
                if refs[lane] != u16::MAX {
                    resolution_valid = false;
                }
                continue;
            }
            let slot = usize::from(refs[lane]);
            let Some(&ordinal_signed) = actual.inline_mapping.get(slot) else {
                resolution_valid = false;
                continue;
            };
            let Ok(ordinal) = usize::try_from(ordinal_signed) else {
                resolution_valid = false;
                continue;
            };
            let Some(node) = preorder.get(ordinal) else {
                resolution_valid = false;
                continue;
            };
            let Some(&ir_id) = part_to_ir.get(&node.number) else {
                resolution_valid = false;
                continue;
            };
            if actual.node_to_bone_map.get(ordinal).copied() != Some(refs[lane] as i16) {
                resolution_valid = false;
            }
            row[lane] = Some(ir_id);
        }
        resolved.push(row);
    }
    if !resolution_valid || resolved != expected.resolved_ir_ids {
        diff.push(format!("{skin_path}.slotResolution"));
    }
}

fn flatten<'a>(node: &'a NodeReport, output: &mut Vec<&'a NodeReport>) {
    output.push(node);
    for child in &node.children {
        flatten(child, output);
    }
}

fn compare_bind_matrix(path: &str, expected: [f32; 16], node: &NodeReport, diff: &mut Vec<String>) {
    if node.mesh.is_some() || node.controllers.len() != 2 {
        diff.push(format!("{path}.bindControllers"));
        return;
    }
    let position = node
        .controllers
        .iter()
        .find(|controller| controller.controller_type == 8);
    let orientation = node
        .controllers
        .iter()
        .find(|controller| controller.controller_type == 20);
    let (Some(position), Some(orientation)) = (position, orientation) else {
        diff.push(format!("{path}.bindControllers"));
        return;
    };
    if position.times != [0.0]
        || orientation.times != [0.0]
        || position.row_count != 1
        || position.time_index != 0
        || position.data_index != 1
        || position.column_count != 3
        || orientation.row_count != 1
        || orientation.time_index != 4
        || orientation.data_index != 5
        || orientation.column_count != 4
        || position.values.len() != 1
        || position.values[0].len() != 3
        || orientation.values.len() != 1
        || orientation.values[0].len() != 4
    {
        diff.push(format!("{path}.bindControllerLayout"));
        return;
    }
    let p = &position.values[0];
    let q = &orientation.values[0];
    let actual = matrix_from_quaternion_translation([q[0], q[1], q[2], q[3]], [p[0], p[1], p[2]]);
    if actual
        .iter()
        .zip(expected)
        .any(|(actual, expected)| (*actual - expected).abs() > 1.0e-5)
    {
        diff.push(format!("{path}.bindMatrix"));
    }
}

fn compare_mesh(path: &str, expected: &ExpectedMesh, actual: &MeshReport, diff: &mut Vec<String>) {
    if actual.textures.first() != Some(&expected.texture_resref) || actual.texture_count != 1 {
        diff.push(format!("{path}.texture"));
    }
    if actual.vertices.len() != expected.positions.len()
        || actual
            .vertices
            .iter()
            .zip(&expected.positions)
            .any(|(a, e)| !vec3_approx(*e, *a))
    {
        diff.push(format!("{path}.positions"));
    }
    if actual.normals.len() != expected.normals.len()
        || actual
            .normals
            .iter()
            .zip(&expected.normals)
            .any(|(a, e)| !vec3_approx(*e, *a))
    {
        diff.push(format!("{path}.normals"));
    }
    if actual.uv0.len() != expected.uv0.len()
        || actual
            .uv0
            .iter()
            .zip(&expected.uv0)
            .any(|(a, e)| !vec2_approx(*e, *a))
    {
        diff.push(format!("{path}.uv0"));
    }
    let face_indices = actual
        .faces
        .iter()
        .flat_map(|face| face.vertex_indices)
        .collect::<Vec<_>>();
    let raw_indices = actual
        .raw_indices
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    if face_indices != expected.indices
        || raw_indices != expected.indices
        || face_indices != raw_indices
    {
        diff.push(format!("{path}.indices"));
    }
    if actual.faces.len() != expected.faces.len() {
        diff.push(format!("{path}.faces"));
    } else {
        for (index, (actual_face, expected_face)) in
            actual.faces.iter().zip(&expected.faces).enumerate()
        {
            if !vec3_approx(expected_face.normal, actual_face.normal)
                || !finite_approx(expected_face.distance, actual_face.distance)
                || actual_face.surface_id != 0
                || actual_face.adjacent_faces != [-1, -1, -1]
                || actual_face.vertex_indices != expected_face.vertex_indices
            {
                diff.push(format!("{path}.faces[{index}]"));
            }
        }
    }
    if actual.index_counts != [expected.indices.len() as u32]
        || actual.raw_index_offsets != [expected.raw_index_offset]
        || actual.raw_indices.len() != 1
    {
        diff.push(format!("{path}.indexMetadata"));
    }
    compare_vec3(
        &format!("{path}.boundsMin"),
        expected.bounds_min,
        actual.bounds_min,
        diff,
    );
    compare_vec3(
        &format!("{path}.boundsMax"),
        expected.bounds_max,
        actual.bounds_max,
        diff,
    );
    if let Some(average) = actual.average {
        compare_vec3(&format!("{path}.average"), expected.average, average, diff);
    } else {
        diff.push(format!("{path}.average"));
    }
    compare_f32(
        &format!("{path}.radius"),
        expected.radius,
        actual.radius,
        diff,
    );
    if actual.diffuse != [1.0, 1.0, 1.0]
        || actual.ambient != [1.0, 1.0, 1.0]
        || actual.specular != [0.0, 0.0, 0.0]
        || actual.shininess != 1.0
        || actual.shadow != 1
        || actual.beaming != 0
        || actual.render != 1
        || actual.transparency != 0
        || actual.render_hint != 0
        || actual.tile_fade != 0
        || actual.mesh_type != 3
        || actual.start_mdx != 0
    {
        diff.push(format!("{path}.profileDefaults"));
    }
}

fn matrix_from_quaternion_translation(q: [f32; 4], p: [f32; 3]) -> [f32; 16] {
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

fn compare_vec3(path: &str, expected: [f32; 3], actual: Vec3, diff: &mut Vec<String>) {
    if !vec3_approx(expected, actual) {
        diff.push(path.to_owned());
    }
}

fn compare_f32(path: &str, expected: f32, actual: f32, diff: &mut Vec<String>) {
    if !finite_approx(expected, actual) {
        diff.push(path.to_owned());
    }
}

fn vec3_approx(expected: [f32; 3], actual: Vec3) -> bool {
    finite_approx(expected[0], actual.x)
        && finite_approx(expected[1], actual.y)
        && finite_approx(expected[2], actual.z)
}

fn vec2_approx(expected: [f32; 2], actual: Vec2) -> bool {
    finite_approx(expected[0], actual.x) && finite_approx(expected[1], actual.y)
}

fn finite_approx(expected: f32, actual: f32) -> bool {
    expected.is_finite() && actual.is_finite() && (expected - actual).abs() <= 1.0e-5
}
