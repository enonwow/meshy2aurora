use std::collections::HashMap;

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

fn translated(x: f32, y: f32, z: f32) -> [f32; 16] {
    let mut matrix = identity();
    matrix[12] = x;
    matrix[13] = y;
    matrix[14] = z;
    matrix
}

fn half_turn_x() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, -1.0, 0.0, 0.0, //
        0.0, 0.0, -1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn rotated_z(degrees: f32) -> [f32; 16] {
    let angle = degrees.to_radians();
    let (sin, cos) = angle.sin_cos();
    [
        cos, sin, 0.0, 0.0, //
        -sin, cos, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn weight_row(influences: &[(u32, f32)]) -> AuroraVertexWeightsV1 {
    let mut row = AuroraVertexWeightsV1 {
        bone_node_ids: [None; 4],
        values: [0.0; 4],
        influence_count: influences.len() as u8,
    };
    for (lane, &(bone, value)) in influences.iter().enumerate() {
        row.bone_node_ids[lane] = Some(bone);
        row.values[lane] = value;
    }
    row
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

fn skin_creature() -> AuroraCreatureIrV1 {
    let mut input = creature();
    input.profile_id = "synthetic-skin".to_owned();
    input.nodes = vec![
        AuroraCreatureNodeV1 {
            id: 40,
            name: "bone_a".to_owned(),
            parent_id: Some(10),
            bind_local_matrix: translated_rotated_z(),
        },
        AuroraCreatureNodeV1 {
            id: 10,
            name: "root".to_owned(),
            parent_id: None,
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 70,
            name: "bone_b".to_owned(),
            parent_id: Some(10),
            bind_local_matrix: translated(0.0, 2.0, 0.0),
        },
        AuroraCreatureNodeV1 {
            id: 20,
            name: "bone_c".to_owned(),
            parent_id: Some(40),
            bind_local_matrix: translated(1.0, 0.0, 0.0),
        },
        AuroraCreatureNodeV1 {
            id: 90,
            name: "bone_d".to_owned(),
            parent_id: Some(10),
            bind_local_matrix: half_turn_x(),
        },
    ];
    input.segments = vec![segment(5, 10, 0, 0.0)];
    input.segments[0].deformation = RigSegmentDeformationV1::Skin;
    input.segments[0].weights = vec![
        weight_row(&[(40, 1.0)]),
        weight_row(&[(70, 0.25), (40, 0.75)]),
        weight_row(&[(90, 0.1), (20, 0.2), (70, 0.3), (40, 0.4)]),
    ];
    input
}

fn skin_with_distinct_bones(bone_count: usize) -> AuroraCreatureIrV1 {
    let mut input = creature();
    input.profile_id = format!("synthetic-skin-{bone_count}");
    input.nodes = vec![AuroraCreatureNodeV1 {
        id: 1,
        name: "root".to_owned(),
        parent_id: None,
        bind_local_matrix: identity(),
    }];
    for index in 0..bone_count {
        input.nodes.push(AuroraCreatureNodeV1 {
            id: 100 + index as u32,
            name: format!("bone_{index}"),
            parent_id: Some(1),
            bind_local_matrix: translated(index as f32 * 0.01, 0.0, 0.0),
        });
    }
    let row_count = bone_count.div_ceil(4).max(3);
    let mut skin = segment(5, 1, 0, 0.0);
    skin.deformation = RigSegmentDeformationV1::Skin;
    while skin.positions.len() < row_count {
        let index = skin.positions.len() as f32;
        skin.positions.push([index, 0.5, 0.0]);
        skin.normals.push([0.0, 0.0, 1.0]);
        skin.uv0.push([0.0, 0.0]);
    }
    skin.weights = (0..row_count)
        .map(|row| {
            let start = row * 4;
            if start >= bone_count {
                return weight_row(&[(100, 1.0)]);
            }
            let count = (bone_count - start).min(4);
            let value = 1.0 / count as f32;
            let influences = (0..count)
                .map(|lane| (100 + (start + lane) as u32, value))
                .collect::<Vec<_>>();
            weight_row(&influences)
        })
        .collect();
    input.segments = vec![skin];
    input
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
    assert!(
        artifact
            .report
            .deviations
            .iter()
            .all(|item| !item.code.starts_with("M4-SKIN-"))
    );
    assert!(artifact.inspection.animations.is_empty());
    assert!(artifact.inspection.diagnostics.is_empty());
    assert!(artifact.inspection.unsupported.is_empty());
    let position = &artifact.inspection.node_tree.roots[0].controllers[0];
    let orientation = &artifact.inspection.node_tree.roots[0].controllers[1];
    assert_eq!((position.packed_byte, position.interpolation_flags), (3, 0));
    assert_eq!(
        (orientation.packed_byte, orientation.interpolation_flags),
        (4, 0)
    );
    assert!(position.decoded && orientation.decoded);
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
fn extended64_skin_roundtrips_1_2_4_lanes_tree_ordinals_and_exact_layout() {
    let input = skin_creature();
    let artifact = write_binary_mdl(&input, &options()).expect("extended64 skin writer");
    assert!(artifact.report.semantic_diff.is_empty());
    assert_eq!(
        artifact
            .report
            .deviations
            .iter()
            .filter(|item| item.code.starts_with("M4-SKIN-"))
            .map(|item| item.code.as_str())
            .collect::<Vec<_>>(),
        [
            "M4-SKIN-INLINE-UNUSED-OPEN-M6",
            "M4-SKIN-SLOT-BOUNDARY-OPEN-M6",
            "M4-SKIN-CONSTANTS-MEANING-OPEN-M6",
            "M4-SKIN-WXYZ-DEFORMATION-OPEN-M6",
            "M4-SKIN-VISUAL-DEFORMATION-OPEN-M6",
        ]
    );
    let skin_layout = &artifact.report.layout.mesh_nodes[0];
    let mut stack = artifact
        .inspection
        .node_tree
        .roots
        .iter()
        .rev()
        .collect::<Vec<_>>();
    let mut preorder = Vec::new();
    while let Some(node) = stack.pop() {
        preorder.push(node);
        stack.extend(node.children.iter().rev());
    }
    let skin_node = preorder
        .iter()
        .copied()
        .find(|node| node.number == skin_layout.part_number)
        .expect("skin node");
    assert_eq!(skin_node.content_flags, 0x61);
    let skin = skin_node.skin.as_ref().expect("skin readback");
    assert_eq!(skin.header_size, 0x330);
    assert_eq!(skin.node_to_bone_pointer, skin_layout.core_offset + 0x330);
    assert_eq!(skin.weights_header.pointer, 0);
    assert_eq!(skin.weights_header.used, 0);
    assert_eq!(skin.weights_header.allocated, 0);
    assert_eq!(skin.node_to_bone_map, [-1, 0, 1, 2, 3, -1]);
    assert_eq!(&skin.inline_mapping[..4], &[1, 2, 3, 4]);
    assert!(skin.inline_mapping[4..].iter().all(|value| *value == -1));
    for header in [&skin.q_header, &skin.t_header, &skin.constants_header] {
        assert_eq!(header.used, 6);
        assert_eq!(header.allocated, 6);
    }
    assert_eq!(
        skin.vertex_weights,
        input.segments[0]
            .weights
            .iter()
            .map(|row| row.values)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        skin.bone_references,
        [
            [0, u16::MAX, u16::MAX, u16::MAX],
            [2, 0, u16::MAX, u16::MAX],
            [3, 1, 2, 0]
        ]
    );
    assert!(skin.bone_constants.iter().all(|value| *value == [0, 0]));

    assert_eq!(
        preorder
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>(),
        ["root", "bone_a", "bone_c", "bone_b", "bone_d", "m2a_seg_5"]
    );
    assert_eq!(
        preorder[1].number, 0,
        "tree ordinal differs from part number"
    );
    let q = skin.inverse_bone_rotations_raw[1];
    let t = skin.inverse_bone_translations[1];
    assert!(q[0] > 0.0, "WXYZ product sign must keep W positive");
    assert!((q[0] - std::f32::consts::FRAC_1_SQRT_2).abs() <= 1.0e-5);
    assert!((q[3] + std::f32::consts::FRAC_1_SQRT_2).abs() <= 1.0e-5);
    assert!((t.x + 3.0).abs() <= 1.0e-5);
    assert!((t.y - 2.0).abs() <= 1.0e-5);
    assert!((t.z + 4.0).abs() <= 1.0e-5);
    let half_turn = skin.inverse_bone_rotations_raw[4];
    assert_eq!(half_turn[0], 0.0, "exact half turn has W=0");
    assert!(
        half_turn[1] > 0.0,
        "first nonzero XYZ component is positive"
    );
}

#[test]
fn extended64_active_slot_boundaries_1_4_64_accept_and_65_rejects() {
    for count in [1, 4, 64] {
        let input = skin_with_distinct_bones(count);
        let artifact = write_binary_mdl(&input, &options()).expect("slot boundary accepted");
        let skin = artifact.inspection.node_tree.roots[0]
            .children
            .last()
            .and_then(|node| node.skin.as_ref())
            .expect("skin readback");
        assert_eq!(
            skin.node_to_bone_map
                .iter()
                .filter(|slot| **slot >= 0)
                .count(),
            count
        );
        assert_eq!(skin.node_to_bone_map.len(), input.nodes.len() + 1);
        assert!(artifact.report.semantic_diff.is_empty());
    }

    let error = write_binary_mdl(&skin_with_distinct_bones(65), &options())
        .expect_err("65 active slots must fail");
    assert_eq!(error.code, "M4-SKIN-SLOT-LIMIT");
    assert_eq!(error.path, "creature.segments[0].weights");
}

#[test]
fn skin_negative_matrix_has_stable_codes_and_exact_paths() {
    let mut cases = Vec::new();

    let mut bad = skin_creature();
    bad.segments[0].weights.pop();
    cases.push((bad, "M4-SKIN-LANE-INVALID", "creature.segments[0].weights"));

    let mut bad = skin_creature();
    bad.segments[0].deformation = RigSegmentDeformationV1::Rigid;
    cases.push((bad, "M4-SKIN-LANE-INVALID", "creature.segments[0]"));

    for count in [0, 5] {
        let mut bad = skin_creature();
        bad.segments[0].weights[0].influence_count = count;
        cases.push((
            bad,
            "M4-SKIN-LANE-INVALID",
            "creature.segments[0].weights[0]",
        ));
    }

    let mut mismatch = skin_creature();
    mismatch.segments[0].weights[0].influence_count = 2;
    cases.push((
        mismatch,
        "M4-SKIN-LANE-INVALID",
        "creature.segments[0].weights[0]",
    ));

    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 0.0] {
        let mut bad = skin_creature();
        bad.segments[0].weights[0].values[0] = value;
        cases.push((
            bad,
            "M4-SKIN-LANE-INVALID",
            "creature.segments[0].weights[0]",
        ));
    }

    let mut inactive_some = skin_creature();
    inactive_some.segments[0].weights[0].bone_node_ids[1] = Some(70);
    cases.push((
        inactive_some,
        "M4-SKIN-LANE-INVALID",
        "creature.segments[0].weights[0]",
    ));

    let mut inactive_nonzero = skin_creature();
    inactive_nonzero.segments[0].weights[0].values[1] = 0.1;
    cases.push((
        inactive_nonzero,
        "M4-SKIN-LANE-INVALID",
        "creature.segments[0].weights[0]",
    ));

    let mut inactive_negative_zero = skin_creature();
    inactive_negative_zero.segments[0].weights[0].values[1] = -0.0;
    cases.push((
        inactive_negative_zero,
        "M4-SKIN-LANE-INVALID",
        "creature.segments[0].weights[0]",
    ));

    let mut bad_sum = skin_creature();
    bad_sum.segments[0].weights[1].values = [0.25002, 0.75, 0.0, 0.0];
    cases.push((
        bad_sum,
        "M4-SKIN-LANE-INVALID",
        "creature.segments[0].weights[1]",
    ));

    let mut missing = skin_creature();
    missing.segments[0].weights[0].bone_node_ids[0] = Some(9999);
    cases.push((
        missing,
        "M4-SKIN-BONE-MISSING",
        "creature.segments[0].weights[0].boneNodeIds[0]",
    ));

    for (input, code, path) in cases {
        let error = write_binary_mdl(&input, &options()).expect_err("negative skin case");
        assert_eq!(error.code, code, "unexpected error: {error:?}");
        assert_eq!(error.path, path, "unexpected path: {error:?}");
    }

    let mut inside = skin_creature();
    inside.segments[0].weights[1].values = [0.250004, 0.75, 0.0, 0.0];
    write_binary_mdl(&inside, &options()).expect("sum inside tolerance accepted");
}

#[test]
fn mixed_rigid_two_skin_segments_keep_locked_block_order_and_are_deterministic() {
    let mut input = skin_creature();
    let first_skin = input.segments[0].clone();
    let mut rigid = segment(6, 40, 0, 1.0);
    rigid.deformation = RigSegmentDeformationV1::Rigid;
    let mut second_skin = first_skin.clone();
    second_skin.segment_id = 7;
    second_skin.parent_node_id = 40;
    second_skin.positions.iter_mut().for_each(|p| p[2] = 2.0);
    input.segments = vec![first_skin, rigid, second_skin];
    let before = serde_json::to_vec(&input).unwrap();

    let first = write_binary_mdl(&input, &options()).expect("mixed skin writer");
    let second = write_binary_mdl(&input, &options()).expect("repeat mixed skin writer");
    assert_eq!(first.payload, second.payload);
    assert_eq!(first.report, second.report);
    assert_eq!(serde_json::to_vec(&input).unwrap(), before);
    assert!(first.report.semantic_diff.is_empty());

    let layouts = &first.report.layout.mesh_nodes;
    let map_count = input.nodes.len() + input.segments.len();
    assert_eq!(
        layouts[1].core_offset as usize,
        align4_test(layouts[0].core_offset as usize + 0x330 + map_count * 2)
    );
    assert_eq!(layouts[2].core_offset, layouts[1].core_offset + 0x270);

    let mut stack = first
        .inspection
        .node_tree
        .roots
        .iter()
        .rev()
        .collect::<Vec<_>>();
    let mut nodes = Vec::new();
    while let Some(node) = stack.pop() {
        nodes.push(node);
        stack.extend(node.children.iter().rev());
    }
    let skins = [0usize, 2].map(|index| {
        nodes
            .iter()
            .copied()
            .find(|node| node.number == layouts[index].part_number)
            .and_then(|node| node.skin.as_ref())
            .expect("skin report")
    });
    let q_bytes = map_count * 16;
    let t_bytes = map_count * 12;
    assert_eq!(
        skins[1].q_header.pointer as usize,
        skins[0].q_header.pointer as usize + q_bytes
    );
    assert_eq!(
        skins[0].t_header.pointer as usize,
        skins[1].q_header.pointer as usize + q_bytes
    );
    assert_eq!(
        skins[1].t_header.pointer as usize,
        skins[0].t_header.pointer as usize + t_bytes
    );
    assert_eq!(
        skins[0].constants_header.pointer as usize,
        skins[1].t_header.pointer as usize + t_bytes
    );
    assert_eq!(
        skins[1].constants_header.pointer as usize,
        skins[0].constants_header.pointer as usize + map_count * 4
    );
}

#[test]
fn deep_raw_controller_drift_has_stable_inverse_bind_rejection() {
    let mut input = creature();
    input.nodes = (0..256_u32)
        .map(|index| AuroraCreatureNodeV1 {
            id: 1_000 + index,
            name: format!("n{index}"),
            parent_id: (index > 0).then_some(1_000 + index - 1),
            bind_local_matrix: rotated_z(117.72),
        })
        .collect();
    let deepest = 1_000 + 255;
    let mut skin = segment(5, 1_000, 0, 0.0);
    skin.deformation = RigSegmentDeformationV1::Skin;
    skin.weights = vec![weight_row(&[(deepest, 1.0)]); 3];
    input.segments = vec![skin];

    let error = write_binary_mdl(&input, &options())
        .expect_err("RAW emitted controller composition beyond rigid tolerance must be rejected");
    assert_eq!(error.code, "M4-SKIN-INVERSE-BIND-UNSUPPORTED");
    assert_eq!(error.path, "creature.segments[0].parentNodeId");
}

#[test]
fn emitted_skin_inverse_bind_matches_worlds_rebuilt_only_from_inspection_controllers() {
    let artifact = write_binary_mdl(&skin_creature(), &options()).expect("skin artifact");
    assert_inspected_skin_inverse_bind(&artifact, 1.0e-5);
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

#[test]
fn reader_rejects_mutated_skin_boundary_counts_and_active_refs() {
    let artifact = write_binary_mdl(&skin_creature(), &options()).unwrap();
    let node = artifact.report.layout.mesh_nodes[0].core_offset as usize;
    let map_count = read_u32_test(&artifact.payload, 12 + node + 0x288);

    let mut boundary = artifact.payload.clone();
    write_u32_test(
        &mut boundary,
        12 + node + 0x284,
        artifact.report.layout.mesh_nodes[0].core_offset + 0x332,
    );
    assert_eq!(
        inspect_binary_mdl(&boundary).unwrap_err().code,
        "M2A-MDL-SKIN-VARIANT-AMBIGUOUS"
    );

    let mut q_count = artifact.payload.clone();
    write_u32_test(&mut q_count, 12 + node + 0x28c + 4, map_count - 1);
    assert_eq!(
        inspect_binary_mdl(&q_count).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );

    let refs_pointer = read_u32_test(&artifact.payload, 12 + node + 0x280) as usize;
    let raw_start = artifact.inspection.file_header.raw_range.start;
    let mut active_sentinel = artifact.payload.clone();
    active_sentinel[raw_start + refs_pointer..raw_start + refs_pointer + 2]
        .copy_from_slice(&u16::MAX.to_le_bytes());
    assert_eq!(
        inspect_binary_mdl(&active_sentinel).unwrap_err().code,
        "M2A-MDL-BONE-REF-OOB"
    );

    let mut ref_oob = artifact.payload.clone();
    ref_oob[raw_start + refs_pointer..raw_start + refs_pointer + 2]
        .copy_from_slice(&(map_count as u16).to_le_bytes());
    assert_eq!(
        inspect_binary_mdl(&ref_oob).unwrap_err().code,
        "M2A-MDL-BONE-REF-OOB"
    );
}

fn assert_inspected_skin_inverse_bind(
    artifact: &m2a_core::mdl::BinaryMdlArtifactV1,
    tolerance: f64,
) {
    let mut pending = artifact
        .inspection
        .node_tree
        .roots
        .iter()
        .rev()
        .collect::<Vec<_>>();
    let mut preorder = Vec::new();
    while let Some(node) = pending.pop() {
        preorder.push(node);
        pending.extend(node.children.iter().rev());
    }

    let mut worlds_by_offset = HashMap::new();
    let mut worlds = Vec::with_capacity(preorder.len());
    for node in &preorder {
        let local = if node.controllers.is_empty() {
            identity_f64_test()
        } else {
            let position = node
                .controllers
                .iter()
                .find(|controller| controller.controller_type == 8)
                .expect("inspection position controller");
            let orientation = node
                .controllers
                .iter()
                .find(|controller| controller.controller_type == 20)
                .expect("inspection orientation controller");
            let p = &position.values[0];
            let q = &orientation.values[0];
            matrix_f64_from_xyzw_test([q[0], q[1], q[2], q[3]], [p[0], p[1], p[2]])
        };
        let world = match node.parent_offset {
            Some(parent) => mul_mat4_f64_test(worlds_by_offset[&parent], local),
            None => local,
        };
        worlds_by_offset.insert(node.offset, world);
        worlds.push(world);
    }

    for (skin_ordinal, node) in preorder.iter().enumerate() {
        let Some(skin) = node.skin.as_ref() else {
            continue;
        };
        assert_eq!(skin.inverse_bone_rotations_raw.len(), worlds.len());
        assert_eq!(skin.inverse_bone_translations.len(), worlds.len());
        let skin_world = worlds[skin_ordinal];
        for (ordinal, node_world) in worlds.iter().enumerate() {
            let expected = mul_mat4_f64_test(inverse_rigid_f64_test(*node_world), skin_world);
            let raw_q = skin.inverse_bone_rotations_raw[ordinal];
            let raw_t = skin.inverse_bone_translations[ordinal];
            let actual = matrix_f64_from_xyzw_test(
                [raw_q[1], raw_q[2], raw_q[3], raw_q[0]],
                [raw_t.x, raw_t.y, raw_t.z],
            );
            let max_error = expected
                .iter()
                .zip(actual)
                .map(|(expected, actual)| (expected - actual).abs())
                .fold(0.0_f64, f64::max);
            assert!(
                max_error <= tolerance,
                "skin ordinal {skin_ordinal}, row {ordinal}: inspection-only inverse bind error {max_error} exceeds {tolerance}"
            );
        }
    }
}

fn identity_f64_test() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn matrix_f64_from_xyzw_test(q: [f32; 4], p: [f32; 3]) -> [f64; 16] {
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
        f64::from(p[0]),
        f64::from(p[1]),
        f64::from(p[2]),
        1.0,
    ]
}

fn mul_mat4_f64_test(a: [f64; 16], b: [f64; 16]) -> [f64; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    output
}

fn inverse_rigid_f64_test(matrix: [f64; 16]) -> [f64; 16] {
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

fn write_u32_test(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32_test(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn align4_test(value: usize) -> usize {
    (value + 3) & !3
}

fn assert_code(
    result: Result<m2a_core::mdl::BinaryMdlArtifactV1, m2a_core::mdl::MdlWriteError>,
    expected: &str,
) {
    let error = result.expect_err("writer should reject invalid input");
    assert_eq!(error.code, expected, "unexpected error: {error:?}");
}
