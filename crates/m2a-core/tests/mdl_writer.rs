use m2a_core::mdl::{
    MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlWriterOptionsV1, inspect_binary_mdl,
    write_binary_mdl,
};
use m2a_core::profile_a::{
    AuroraCreatureIrV1, AuroraCreatureNodeV1, AuroraCreatureSegmentV1, AuroraVertexWeightsV1,
    MaterialSourceBindingV1, RigSegmentDeformationV1,
};

fn identity() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn translated_rotated_z() -> [f32; 16] {
    [
        0.0, 1.0, 0.0, 0.0, //
        -1.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        2.0, 3.0, 4.0, 1.0,
    ]
}

fn segment(id: u32, parent: u32, material_slot: u32, z: f32) -> AuroraCreatureSegmentV1 {
    AuroraCreatureSegmentV1 {
        segment_id: id,
        material_slot,
        deformation: RigSegmentDeformationV1::Rigid,
        parent_node_id: parent,
        positions: vec![[0.0, 0.0, z], [1.0, 0.0, z], [0.0, 1.0, z]],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        tangents: None,
        uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        indices: vec![0, 1, 2],
        weights: Vec::<AuroraVertexWeightsV1>::new(),
    }
}

fn creature() -> AuroraCreatureIrV1 {
    AuroraCreatureIrV1 {
        schema_version: 1,
        profile_id: "synthetic-rigid".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
        engine_facing_proof: "OPEN_M6".to_owned(),
        uv_runtime_proof: "OPEN_M6".to_owned(),
        nodes: vec![AuroraCreatureNodeV1 {
            id: 70,
            name: "root".to_owned(),
            parent_id: None,
            bind_local_matrix: identity(),
        }],
        material_source_bindings: vec![MaterialSourceBindingV1 {
            slot: 0,
            source_material_id: None,
            source_material_name: None,
        }],
        segments: vec![segment(5, 70, 0, 0.0)],
    }
}

fn options() -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
        model_resource_resref: "m2a_test".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_tex".to_owned(),
        }],
    }
}

#[test]
fn minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof() {
    let artifact = write_binary_mdl(&creature(), &options()).expect("rigid writer");
    assert!(artifact.report.semantic_diff.is_empty());
    assert_eq!(artifact.report.layout.rig_nodes[0].ir_node_id, 70);
    assert_eq!(artifact.report.layout.rig_nodes[0].part_number, 0);
    assert_eq!(artifact.report.layout.mesh_nodes[0].segment_id, 5);
    assert_eq!(artifact.report.layout.mesh_nodes[0].part_number, 1);
    assert_eq!(
        artifact.report.layout.core_length as u32,
        artifact.inspection.file_header.mdx_start
    );
    assert_eq!(
        artifact.report.layout.raw_length as u32,
        artifact.inspection.file_header.mdx_size
    );
    assert_eq!(artifact.report.layout.file_length, artifact.payload.len());
    assert_eq!(artifact.inspection.model.name, "m2a_test");
    assert_eq!(artifact.inspection.model.classification, 4);
    assert_eq!(artifact.inspection.model.fog, 1);
    assert_eq!(artifact.inspection.model.child_model_count, 0);
    assert_eq!(artifact.inspection.model.animation_scale, 1.0);
    assert_eq!(artifact.inspection.model.supermodel_name, "null");
    assert!(artifact.inspection.animations.is_empty());
    assert!(artifact.inspection.diagnostics.is_empty());
    assert!(artifact.inspection.unsupported.is_empty());
    let position = &artifact.inspection.node_tree.roots[0].controllers[0];
    let orientation = &artifact.inspection.node_tree.roots[0].controllers[1];
    assert_eq!(
        (
            position.controller_type,
            position.row_count,
            position.time_index,
            position.data_index,
            position.column_count
        ),
        (8, 1, 0, 1, 3)
    );
    assert_eq!(
        (
            orientation.controller_type,
            orientation.row_count,
            orientation.time_index,
            orientation.data_index,
            orientation.column_count
        ),
        (20, 1, 4, 5, 4)
    );

    let mesh_node = &artifact.inspection.node_tree.roots[0].children[0];
    let mesh = mesh_node.mesh.as_ref().expect("mesh readback");
    assert_eq!(mesh.textures[0], "m2a_tex");
    assert_eq!(mesh.faces[0].vertex_indices, [0, 1, 2]);
    assert_eq!(mesh.faces[0].normal.x, 0.0);
    assert_eq!(mesh.faces[0].normal.y, 0.0);
    assert_eq!(mesh.faces[0].normal.z, 1.0);
    assert_eq!(mesh.faces[0].distance, 0.0);
    assert_eq!(mesh.faces[0].surface_id, 0);
    assert_eq!(mesh.faces[0].adjacent_faces, [-1, -1, -1]);
    assert_eq!(mesh.index_counts, [3]);
    assert_eq!(mesh.raw_indices, [vec![0, 1, 2]]);
    assert_eq!(mesh.mesh_type, 3);
    assert_eq!(mesh.start_mdx, 0);
    assert_eq!(mesh.diffuse, [1.0, 1.0, 1.0]);
    assert_eq!(mesh.ambient, [1.0, 1.0, 1.0]);
    assert_eq!(mesh.specular, [0.0, 0.0, 0.0]);
    assert_eq!(mesh.shininess, 1.0);
    assert_eq!(mesh.shadow, 1);
    assert_eq!(mesh.beaming, 0);
    assert_eq!(mesh.render, 1);
    assert_eq!(artifact.payload.len(), 1188);
    assert_eq!(
        artifact.report.payload_sha256,
        "e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2"
    );

    let mut trailing = artifact.payload.clone();
    trailing.push(0);
    let error = inspect_binary_mdl(&trailing).expect_err("trailing byte must fail exact EOF");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");
}

#[test]
fn multi_node_multi_segment_preserves_ir_order_but_points_at_the_actual_root() {
    let mut input = creature();
    input.nodes = vec![
        AuroraCreatureNodeV1 {
            id: 9,
            name: "child".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: translated_rotated_z(),
        },
        AuroraCreatureNodeV1 {
            id: 70,
            name: "root".to_owned(),
            parent_id: None,
            bind_local_matrix: identity(),
        },
    ];
    input.segments = vec![segment(20, 9, 0, 0.0), segment(10, 70, 1, 1.0)];
    input
        .material_source_bindings
        .push(MaterialSourceBindingV1 {
            slot: 1,
            source_material_id: Some(1),
            source_material_name: Some("second".to_owned()),
        });
    let mut write_options = options();
    write_options
        .diffuse_texture_resref_by_material_slot
        .push(MdlMaterialTextureBindingV1 {
            material_slot: 1,
            resref: "m2a_tex2".to_owned(),
        });

    let artifact = write_binary_mdl(&input, &write_options).expect("multi rigid writer");
    assert!(artifact.report.semantic_diff.is_empty());
    assert_eq!(
        artifact
            .report
            .layout
            .rig_nodes
            .iter()
            .map(|node| (node.ir_node_id, node.part_number))
            .collect::<Vec<_>>(),
        [(9, 0), (70, 1)]
    );
    assert_eq!(artifact.inspection.node_tree.roots[0].name, "root");
    assert_eq!(artifact.inspection.node_tree.roots[0].number, 1);
    assert_eq!(artifact.inspection.node_tree.node_count, 4);
    assert_eq!(
        artifact.inspection.node_tree.roots[0].children[0].name,
        "child"
    );
    assert_eq!(
        artifact.inspection.node_tree.roots[0].children[1].name,
        "m2a_seg_10"
    );
    assert_eq!(
        artifact.inspection.node_tree.roots[0].children[0].children[0].name,
        "m2a_seg_20"
    );
}

#[test]
fn identical_input_is_byte_identical_report_identical_and_input_is_not_mutated() {
    let input = creature();
    let before = serde_json::to_vec(&input).unwrap();
    let first = write_binary_mdl(&input, &options()).unwrap();
    let second = write_binary_mdl(&input, &options()).unwrap();
    assert_eq!(first.payload, second.payload);
    assert_eq!(first.report, second.report);
    assert_eq!(serde_json::to_vec(&input).unwrap(), before);
    assert_eq!(first.report.payload_sha256.len(), 64);
}

#[test]
fn tangents_are_a_stable_nonfatal_deviation() {
    let mut input = creature();
    input.segments[0].tangents = Some(vec![[1.0, 0.0, 0.0, 1.0]; 3]);
    let artifact = write_binary_mdl(&input, &options()).unwrap();
    assert_eq!(artifact.report.deviations.len(), 6);
    assert!(artifact.report.deviations.iter().any(|item| {
        item.code == "M4-TANGENTS-NOT-EMITTED" && item.path == "creature.segments[0].tangents"
    }));
    assert!(artifact.report.semantic_diff.is_empty());
}

#[test]
fn schema_profile_names_and_material_bindings_have_stable_errors() {
    let mut bad_options = options();
    bad_options.schema_version = 2;
    assert_code(
        write_binary_mdl(&creature(), &bad_options),
        "M4-INVALID-SCHEMA",
    );

    let mut bad_options = options();
    bad_options.format_profile = MdlFormatProfileV1::Legacy17V1;
    assert_code(
        write_binary_mdl(&creature(), &bad_options),
        "M4-UNSUPPORTED-PROFILE",
    );

    let mut bad_options = options();
    bad_options.model_resource_resref = "Bad-Name".to_owned();
    assert_code(
        write_binary_mdl(&creature(), &bad_options),
        "M4-INVALID-NAME",
    );

    let mut bad_input = creature();
    bad_input.nodes[0].name = "x".repeat(32);
    assert_code(write_binary_mdl(&bad_input, &options()), "M4-INVALID-NAME");

    let mut bad_options = options();
    bad_options.diffuse_texture_resref_by_material_slot.clear();
    assert_code(
        write_binary_mdl(&creature(), &bad_options),
        "M4-MATERIAL-BINDING-MISSING",
    );

    let mut bad_options = options();
    bad_options
        .diffuse_texture_resref_by_material_slot
        .push(bad_options.diffuse_texture_resref_by_material_slot[0].clone());
    assert_code(
        write_binary_mdl(&creature(), &bad_options),
        "M4-MATERIAL-BINDING-INVALID",
    );

    let mut bad_options = options();
    bad_options
        .diffuse_texture_resref_by_material_slot
        .push(MdlMaterialTextureBindingV1 {
            material_slot: 99,
            resref: "unused".to_owned(),
        });
    assert_code(
        write_binary_mdl(&creature(), &bad_options),
        "M4-MATERIAL-BINDING-INVALID",
    );
}

#[test]
fn hierarchy_transform_mesh_and_limit_failures_are_stable() {
    let mut bad = creature();
    bad.nodes.push(AuroraCreatureNodeV1 {
        id: 71,
        name: "second_root".to_owned(),
        parent_id: None,
        bind_local_matrix: identity(),
    });
    assert_code(write_binary_mdl(&bad, &options()), "M4-HIERARCHY-INVALID");

    let mut bad = creature();
    bad.nodes[0].bind_local_matrix[0] = 2.0;
    assert_code(
        write_binary_mdl(&bad, &options()),
        "M4-BIND-TRANSFORM-UNSUPPORTED",
    );

    let mut bad = creature();
    bad.segments[0].normals.pop();
    assert_code(write_binary_mdl(&bad, &options()), "M4-MESH-INVALID");

    let mut bad = creature();
    bad.segments[0].indices[2] = 3;
    assert_code(write_binary_mdl(&bad, &options()), "M4-MESH-INVALID");

    let mut bad = creature();
    bad.segments[0].positions = vec![[0.0, 0.0, 0.0]; usize::from(u16::MAX) + 1];
    bad.segments[0].normals = vec![[0.0, 0.0, 1.0]; bad.segments[0].positions.len()];
    bad.segments[0].uv0 = vec![[0.0, 0.0]; bad.segments[0].positions.len()];
    assert_code(write_binary_mdl(&bad, &options()), "M4-MESH-LIMIT");

    let mut bad = creature();
    bad.segments[0].positions = vec![
        [f32::MAX, f32::MAX, 0.0],
        [f32::MAX, 0.0, 0.0],
        [0.0, f32::MAX, 0.0],
    ];
    assert_code(write_binary_mdl(&bad, &options()), "M4-MESH-INVALID");
}

#[test]
fn near_half_turn_keeps_positive_w_and_uses_xyz_tie_break_only_for_exact_zero() {
    let w = 5.0e-6_f32;
    let x = -(1.0 - w * w).sqrt();
    let mut input = creature();
    input.nodes[0].bind_local_matrix = [
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 - 2.0 * x * x,
        2.0 * x * w,
        0.0,
        0.0,
        -2.0 * x * w,
        1.0 - 2.0 * x * x,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    let artifact = write_binary_mdl(&input, &options()).unwrap();
    let orientation = artifact.inspection.node_tree.roots[0]
        .controllers
        .iter()
        .find(|controller| controller.controller_type == 20)
        .unwrap();
    assert!(orientation.values[0][3] > 0.0);
    assert!(orientation.values[0][0] < 0.0);
}

#[test]
fn exact_half_turn_uses_the_first_exactly_nonzero_xyz_component() {
    let x = -1.0e-6_f32;
    let y = (1.0 - x * x).sqrt();
    let mut input = creature();
    input.nodes[0].bind_local_matrix = [
        1.0 - 2.0 * y * y,
        2.0 * x * y,
        0.0,
        0.0,
        2.0 * x * y,
        1.0 - 2.0 * x * x,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    let artifact = write_binary_mdl(&input, &options()).unwrap();
    let q = &artifact.inspection.node_tree.roots[0].controllers[1].values[0];
    assert_eq!(q[3], 0.0);
    assert!(q[0] > 0.0, "first nonzero xyz component must be positive");
    assert!(q[1] < 0.0, "the whole quaternion sign must flip");
}

#[test]
fn reader_rejects_mutated_index_metadata_pointer_and_value() {
    let artifact = write_binary_mdl(&creature(), &options()).unwrap();
    let node = artifact.report.layout.mesh_nodes[0].core_offset as usize;

    let mut used_mismatch = artifact.payload.clone();
    write_u32_test(&mut used_mismatch, 12 + node + 0x204 + 4, 0);
    assert_eq!(
        inspect_binary_mdl(&used_mismatch).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );

    let mut core_oob = artifact.payload.clone();
    write_u32_test(&mut core_oob, 12 + node + 0x204, u32::MAX);
    assert_eq!(
        inspect_binary_mdl(&core_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    let offset_array = read_u32_test(&artifact.payload, 12 + node + 0x210) as usize;
    let mut raw_oob = artifact.payload.clone();
    write_u32_test(&mut raw_oob, 12 + offset_array, i32::MAX as u32);
    assert_eq!(
        inspect_binary_mdl(&raw_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    let raw_index = artifact.inspection.file_header.raw_range.start
        + artifact.inspection.node_tree.roots[0].children[0]
            .mesh
            .as_ref()
            .unwrap()
            .raw_index_offsets[0] as usize;
    let mut value_oob = artifact.payload.clone();
    value_oob[raw_index..raw_index + 2].copy_from_slice(&3_u16.to_le_bytes());
    assert_eq!(
        inspect_binary_mdl(&value_oob).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );

    let count_array = read_u32_test(&artifact.payload, 12 + node + 0x204) as usize;
    let mut empty_invalid_negative = artifact.payload.clone();
    write_u32_test(&mut empty_invalid_negative, 12 + count_array, 0);
    write_u32_test(
        &mut empty_invalid_negative,
        12 + offset_array,
        (-2_i32) as u32,
    );
    assert_eq!(
        inspect_binary_mdl(&empty_invalid_negative)
            .unwrap_err()
            .code,
        "M2A-MDL-POINTER-OOB"
    );

    let mut empty_raw_oob = artifact.payload.clone();
    write_u32_test(&mut empty_raw_oob, 12 + count_array, 0);
    write_u32_test(
        &mut empty_raw_oob,
        12 + offset_array,
        artifact.inspection.file_header.mdx_size + 1,
    );
    assert_eq!(
        inspect_binary_mdl(&empty_raw_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );
}

fn write_u32_test(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32_test(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn assert_code(
    result: Result<m2a_core::mdl::BinaryMdlArtifactV1, m2a_core::mdl::MdlWriteError>,
    expected: &str,
) {
    let error = result.expect_err("writer should reject invalid input");
    assert_eq!(error.code, expected, "unexpected error: {error:?}");
}
