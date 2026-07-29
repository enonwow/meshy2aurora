use std::collections::HashMap;

use m2a_core::direct_creature_animation::evaluate_direct_creature_animation_behavior_v2;
use m2a_core::mdl::{
    MdlAnimationClipV1, MdlAnimationEventV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
    MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
    MdlStateProjectionProfileV1, MdlStateProjectionProvenanceV1, MdlWriterOptionsV1,
    NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, evaluate_skin_deformation_v1, inspect_binary_mdl,
    verify_direct_creature_state_projection_v1,
    verify_direct_creature_state_projection_with_expected_provenance_v1, write_binary_mdl,
    write_binary_mdl_with_animations,
};
use m2a_core::model_pipeline::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1;
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
        cast_shadow: true,
        positions: vec![[0.0, 0.0, z], [1.0, 0.0, z], [0.0, 1.0, z]],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        tangents: None,
        uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        indices: vec![0, 1, 2],
        face_surface_ids: Vec::new(),
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
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_test".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_tex".to_owned(),
        }],
    }
}

fn placeable_options() -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        format_profile: MdlFormatProfileV1::PlaceableStaticRigidNativeV1,
        ..options()
    }
}

fn cep_r3_provenance() -> MdlStateProjectionProvenanceV1 {
    MdlStateProjectionProvenanceV1 {
        schema_version: 1,
        source_family: "CEP3_CORE1_R3_RIGID_PLACEHOLDER".to_owned(),
        container_sha256: "6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a"
            .to_owned(),
        resource_resref: "c_phod_horror_b".to_owned(),
        resource_sha256: "62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a"
            .to_owned(),
    }
}

fn translation_track(target_node_id: u32) -> MdlAnimationTrackV1 {
    MdlAnimationTrackV1 {
        target_node_id,
        path: MdlAnimationTrackPathV1::Translation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        times_seconds: vec![0.0, 1.0],
        values: vec![vec![0.0, 0.0, 0.0], vec![0.25, -0.5, 1.0]],
    }
}

fn rotation_track(target_node_id: u32) -> MdlAnimationTrackV1 {
    let half = std::f32::consts::FRAC_PI_4;
    MdlAnimationTrackV1 {
        target_node_id,
        path: MdlAnimationTrackPathV1::Rotation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        times_seconds: vec![0.0, 1.0],
        values: vec![
            vec![0.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, -half.sin(), -half.cos()],
        ],
    }
}

fn cpause1_set(target_node_id: u32) -> MdlAnimationSetV1 {
    MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "cpause1".to_owned(),
            animation_root: "owned_root".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.25,
            events: vec![
                MdlAnimationEventV1 {
                    time_seconds: 0.75,
                    name: "owned_late".to_owned(),
                },
                MdlAnimationEventV1 {
                    time_seconds: 0.5,
                    name: "owned_equal_a".to_owned(),
                },
                MdlAnimationEventV1 {
                    time_seconds: 0.5,
                    name: "owned_equal_b".to_owned(),
                },
            ],
            tracks: vec![
                rotation_track(target_node_id),
                translation_track(target_node_id),
            ],
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
    assert_eq!(artifact.inspection.model.supermodel_name, "NULL");
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
        "257f30d79926e38922f1a8af375ac2c1869aedd41e8783420cca56ab647c25c1"
    );

    let mut trailing = artifact.payload.clone();
    trailing.push(0);
    let error = inspect_binary_mdl(&trailing).expect_err("trailing byte must fail exact EOF");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");
}

#[test]
fn mesh_shadow_participation_roundtrips_without_changing_render_participation() {
    let mut input = creature();
    input.segments[0].cast_shadow = false;
    let artifact = write_binary_mdl(&input, &options()).expect("shadowless rigid mesh");
    assert!(artifact.report.semantic_diff.is_empty());
    let mesh = artifact.inspection.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("mesh readback");
    assert_eq!(mesh.shadow, 0);
    assert_eq!(mesh.render, 1);
}

#[test]
fn extended64_skin_roundtrips_1_2_4_lanes_tree_ordinals_and_exact_layout() {
    let input = skin_creature();
    let artifact = write_binary_mdl(&input, &options()).expect("extended64 skin writer");
    assert!(artifact.report.semantic_diff.is_empty());
    assert_eq!(
        artifact
            .report
            .layout
            .rig_nodes
            .iter()
            .map(|node| (node.ir_node_id, node.part_number))
            .collect::<Vec<_>>(),
        [(10, 0), (40, 1), (20, 2), (70, 3), (90, 4)],
        "binary part numbers must follow root-first hierarchy order, not source-array order"
    );
    assert_eq!(
        (
            artifact.inspection.node_tree.roots[0].name.as_str(),
            artifact.inspection.node_tree.roots[0].number,
        ),
        ("root", 0),
        "the exact model root must be native-compatible part 0"
    );
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
    assert!(skin.bone_constants.iter().all(|value| *value == 0));

    assert_eq!(
        preorder
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>(),
        ["root", "bone_a", "bone_c", "bone_b", "bone_d", "m2a_seg_5"]
    );
    assert_eq!(
        preorder[1].number, 1,
        "root-first numbering keeps native part numbers aligned with tree ordinals"
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
fn native_zero_terminated_extended64_skin_ends_the_runtime_palette_after_active_slots() {
    let input = skin_creature();
    let mut writer_options = options();
    writer_options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;

    let artifact =
        write_binary_mdl(&input, &writer_options).expect("zero-terminated extended64 skin writer");
    assert!(artifact.report.semantic_diff.is_empty());

    let mut stack = artifact
        .inspection
        .node_tree
        .roots
        .iter()
        .collect::<Vec<_>>();
    let mut skin_node = None;
    while let Some(node) = stack.pop() {
        if node.skin.is_some() {
            skin_node = Some(node);
            break;
        }
        stack.extend(&node.children);
    }
    let skin_node = skin_node.expect("skin node readback");
    assert_eq!(
        skin_node
            .controllers
            .iter()
            .map(|controller| controller.controller_type)
            .collect::<Vec<_>>(),
        [8, 20],
    );
    assert_eq!(skin_node.controllers[0].values, [[0.0, 0.0, 0.0]]);
    assert_eq!(skin_node.controllers[1].values, [[0.0, 0.0, 0.0, 1.0]],);
    let skin = skin_node.skin.as_ref().expect("skin readback");
    assert_eq!(&skin.inline_mapping[..4], &[1, 2, 3, 4]);
    assert!(
        skin.inline_mapping[4..].iter().all(|value| *value == 0),
        "the native runtime palette loop requires a zero-terminated unused tail"
    );
    assert!(
        artifact
            .report
            .deviations
            .iter()
            .all(|item| item.code != "M4-SKIN-INLINE-UNUSED-OPEN-M6")
    );
}

#[test]
fn controllerless_identity_model_root_profile_removes_only_redundant_base_controllers() {
    let mut input = skin_creature();
    input
        .nodes
        .iter_mut()
        .find(|node| node.parent_id.is_none())
        .expect("single model root")
        .name = "m2a_test".to_owned();
    let animation_set = cpause1_set(40);
    let mut legacy_options = options();
    legacy_options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;
    legacy_options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;
    let mut controllerless_options = legacy_options.clone();
    controllerless_options.format_profile =
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3;

    let legacy = write_binary_mdl_with_animations(&input, &animation_set, &legacy_options).unwrap();
    let artifact =
        write_binary_mdl_with_animations(&input, &animation_set, &controllerless_options).unwrap();
    let repeated =
        write_binary_mdl_with_animations(&input, &animation_set, &controllerless_options).unwrap();
    let legacy_root = &legacy.inspection.node_tree.roots[0];
    let root = &artifact.inspection.node_tree.roots[0];

    assert_eq!(root.name, "m2a_test");
    assert_eq!(legacy_root.controllers.len(), 2);
    assert!(root.controllers.is_empty());
    assert_eq!(root.controller_keys_header.used, 0);
    assert_eq!(root.controller_keys_header.allocated, 0);
    assert_eq!(root.controller_data_header.used, 0);
    assert_eq!(root.controller_data_header.allocated, 0);
    assert_eq!(
        artifact.report.layout.core_length + 60,
        legacy.report.layout.core_length
    );
    assert_eq!(
        artifact.report.layout.raw_length,
        legacy.report.layout.raw_length
    );
    assert_eq!(artifact.payload, repeated.payload);
    assert_eq!(artifact.report, repeated.report);
    assert!(artifact.report.semantic_diff.is_empty());
    assert!(
        artifact
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.roots[0].controllers.is_empty())
    );

    let legacy_skin = legacy_root
        .children
        .iter()
        .find_map(|node| node.skin.as_ref())
        .expect("legacy skin");
    let skin = root
        .children
        .iter()
        .find_map(|node| node.skin.as_ref())
        .expect("controllerless-root skin");
    assert_eq!(skin.node_to_bone_map, legacy_skin.node_to_bone_map);
    assert_eq!(skin.inline_mapping, legacy_skin.inline_mapping);
    assert_eq!(
        skin.inverse_bone_rotations_raw,
        legacy_skin.inverse_bone_rotations_raw
    );
    assert_eq!(
        skin.inverse_bone_translations,
        legacy_skin.inverse_bone_translations
    );
    assert_eq!(skin.vertex_weights, legacy_skin.vertex_weights);
    assert_eq!(skin.bone_references, legacy_skin.bone_references);

    let mut non_identity = input.clone();
    non_identity
        .nodes
        .iter_mut()
        .find(|node| node.parent_id.is_none())
        .unwrap()
        .bind_local_matrix = translated(1.0, 0.0, 0.0);
    let error = write_binary_mdl(&non_identity, &controllerless_options).unwrap_err();
    assert_eq!(error.code, "M4-CONTROLLERLESS-ROOT-INVALID");
    assert!(error.path.ends_with("bindLocalMatrix"));

    let mut wrong_name = input.clone();
    wrong_name
        .nodes
        .iter_mut()
        .find(|node| node.parent_id.is_none())
        .unwrap()
        .name = "other_root".to_owned();
    let error = write_binary_mdl(&wrong_name, &controllerless_options).unwrap_err();
    assert_eq!(error.code, "M4-CONTROLLERLESS-ROOT-INVALID");
    assert!(error.path.ends_with(".name"));

    let mut weighted_root = input;
    weighted_root.segments[0].weights[0] = weight_row(&[(10, 1.0)]);
    let error = write_binary_mdl(&weighted_root, &controllerless_options).unwrap_err();
    assert_eq!(error.code, "M4-CONTROLLERLESS-ROOT-INVALID");
    assert!(error.path.contains("weights"));
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
fn emitted_skin_motion_has_end_to_end_cpu_deformation_conformance() {
    let half = std::f32::consts::FRAC_PI_4;
    let animations = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "cpause1".to_owned(),
            animation_root: "root".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.25,
            events: Vec::new(),
            tracks: vec![
                MdlAnimationTrackV1 {
                    target_node_id: 40,
                    path: MdlAnimationTrackPathV1::Translation,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: vec![0.0, 1.0],
                    values: vec![vec![2.0, 3.0, 4.0], vec![3.0, 3.0, 4.0]],
                },
                MdlAnimationTrackV1 {
                    target_node_id: 40,
                    path: MdlAnimationTrackPathV1::Rotation,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: vec![0.0, 1.0],
                    values: vec![
                        vec![0.0, 0.0, half.sin(), half.cos()],
                        vec![0.0, 0.0, half.sin(), half.cos()],
                    ],
                },
            ],
        }],
    };
    let artifact = write_binary_mdl_with_animations(&skin_creature(), &animations, &options())
        .expect("animated skin artifact");

    let bind =
        evaluate_skin_deformation_v1(&artifact.inspection, "cpause1", 0.0).expect("bind sample");
    assert_eq!((bind.skin_count, bind.vertex_count), (1, 3));
    assert_eq!(bind.moved_vertex_count, 0);
    assert!(bind.max_displacement <= 1.0e-5);

    let moved =
        evaluate_skin_deformation_v1(&artifact.inspection, "cpause1", 1.0).expect("motion sample");
    assert_eq!(moved.moved_vertex_count, 3);
    assert!((moved.max_displacement - 1.0).abs() <= 1.0e-5);
    let vertices = &moved.skins[0].vertices;
    for (actual, expected) in vertices.iter().map(|vertex| vertex.sampled_world).zip([
        [1.0, 0.0, 0.0],
        [1.75, 0.0, 0.0],
        [0.6, 1.0, 0.0],
    ]) {
        for lane in 0..3 {
            assert!(
                (actual[lane] - expected[lane]).abs() <= 1.0e-5,
                "actual {actual:?}, expected {expected:?}"
            );
        }
    }
}

#[test]
fn skin_deformation_oracle_slerps_wxyz_rotation_and_fails_closed_without_skin() {
    let mut input = creature();
    input.segments[0].deformation = RigSegmentDeformationV1::Skin;
    input.segments[0].weights = vec![weight_row(&[(70, 1.0)]); 3];
    let half = std::f32::consts::FRAC_PI_4;
    let animations = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "cpause1".to_owned(),
            animation_root: "root".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.25,
            events: Vec::new(),
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 70,
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![
                    vec![0.0, 0.0, 0.0, 1.0],
                    vec![0.0, 0.0, half.sin(), half.cos()],
                ],
            }],
        }],
    };
    let artifact = write_binary_mdl_with_animations(&input, &animations, &options())
        .expect("rotating skin artifact");
    let sample =
        evaluate_skin_deformation_v1(&artifact.inspection, "cpause1", 0.5).expect("mid sample");
    let diagonal = std::f32::consts::FRAC_1_SQRT_2;
    for (actual, expected) in sample.skins[0]
        .vertices
        .iter()
        .map(|vertex| vertex.sampled_world)
        .zip([
            [0.0, 0.0, 0.0],
            [diagonal, diagonal, 0.0],
            [-diagonal, diagonal, 0.0],
        ])
    {
        for lane in 0..3 {
            assert!(
                (actual[lane] - expected[lane]).abs() <= 1.0e-5,
                "actual {actual:?}, expected {expected:?}"
            );
        }
    }

    let rigid = write_binary_mdl_with_animations(&creature(), &cpause1_set(70), &options())
        .expect("rigid animated control");
    let error = evaluate_skin_deformation_v1(&rigid.inspection, "cpause1", 0.5)
        .expect_err("rigid model must not produce a skin conformance sample");
    assert_eq!(error.code, "M2A-MDL-SKIN-DEFORMATION-NO-SKIN");

    let mut corrupt = artifact.inspection.clone();
    corrupt.node_tree.roots[0].children[0]
        .skin
        .as_mut()
        .expect("skin report")
        .inverse_bone_rotations_raw[0] = [0.0; 4];
    let error = evaluate_skin_deformation_v1(&corrupt, "cpause1", 0.5)
        .expect_err("zero inverse-bind quaternion must fail closed");
    assert_eq!(error.code, "M2A-MDL-SKIN-DEFORMATION-INVERSE-BIND");
}

#[test]
fn multi_node_multi_segment_numbers_the_actual_root_zero_and_preserves_hierarchy() {
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
        [(70, 0), (9, 1)]
    );
    assert_eq!(artifact.inspection.node_tree.roots[0].name, "root");
    assert_eq!(artifact.inspection.node_tree.roots[0].number, 0);
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
fn empty_animation_api_is_the_frozen_m4_wrapper_byte_for_byte() {
    let input = creature();
    let legacy = write_binary_mdl(&input, &options()).expect("frozen M4 wrapper");
    let explicit =
        write_binary_mdl_with_animations(&input, &MdlAnimationSetV1::empty(), &options())
            .expect("explicit empty animation set");

    assert_eq!(legacy.payload, explicit.payload);
    assert_eq!(legacy.report, explicit.report);
    assert_eq!(legacy.inspection, explicit.inspection);
    assert_eq!(legacy.payload.len(), 1188);
    assert_eq!(legacy.report.layout.core_length, 1072);
    assert_eq!(legacy.report.layout.raw_length, 104);
    assert_eq!(
        legacy.report.payload_sha256,
        "257f30d79926e38922f1a8af375ac2c1869aedd41e8783420cca56ab647c25c1"
    );
    assert_eq!(legacy.inspection.model.animation_pointers_header.used, 0);
    assert!(legacy.report.animation.is_none());
}

#[test]
fn owned_cpause1_roundtrips_exact_animation_layout_events_and_linear_keys() {
    let animation_set = cpause1_set(70);
    let artifact = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("owned cpause1 writer");

    assert!(artifact.report.semantic_diff.is_empty());
    assert_eq!(artifact.inspection.byte_length, artifact.payload.len());
    assert_eq!(
        (
            artifact.report.layout.core_length,
            artifact.report.layout.raw_length,
            artifact.report.layout.file_length,
            artifact.payload.len(),
        ),
        (1704, 104, 1820, 1820)
    );
    assert_eq!(
        (
            artifact.inspection.model.animation_pointers_header.pointer,
            artifact.inspection.model.animation_pointers_header.used,
            artifact
                .inspection
                .model
                .animation_pointers_header
                .allocated,
        ),
        (232, 1, 1)
    );
    let report = artifact
        .report
        .animation
        .as_ref()
        .expect("animation report");
    assert_eq!(
        (report.clip_count, report.event_count, report.track_count),
        (1, 3, 2)
    );
    assert_eq!(
        (
            report.pointer_array_core_offset,
            report.clips[0].header_core_offset,
            report.clips[0].root_core_offset,
            report.clips[0].event_array_core_offset,
        ),
        (232, 236, 540, Some(432))
    );
    assert_eq!(
        report.clips[0].event_array_core_offset,
        Some(report.clips[0].header_core_offset + 0xc4)
    );
    assert_eq!(report.clips[0].nodes.len(), 2);
    assert_eq!(
        report.clips[0].nodes[0]
            .tracks
            .iter()
            .map(|track| (track.controller_type, track.packed_byte))
            .collect::<Vec<_>>(),
        vec![(8, 3), (20, 4)]
    );
    assert!(
        report.clips[0].nodes[1].ir_node_id.is_none() && report.clips[0].nodes[1].tracks.is_empty(),
        "the second animation node is the retail mesh-identity dummy"
    );
    let clip = &artifact.inspection.animations[0];
    assert_eq!(clip.name, "cpause1");
    assert_eq!(clip.length.to_bits(), 1.0_f32.to_bits());
    assert_eq!(clip.transition.to_bits(), 0.25_f32.to_bits());
    assert_eq!(clip.animation_root, "owned_root");
    assert_eq!(
        (
            clip.events_header.pointer,
            clip.events_header.used,
            clip.events_header.allocated,
        ),
        (432, 3, 3)
    );
    assert_eq!(
        clip.events
            .iter()
            .map(|event| event.name.as_str())
            .collect::<Vec<_>>(),
        vec!["owned_equal_a", "owned_equal_b", "owned_late"]
    );
    assert_eq!(
        clip.events
            .iter()
            .map(|event| event.time.to_bits())
            .collect::<Vec<_>>(),
        vec![0.5_f32.to_bits(), 0.5_f32.to_bits(), 0.75_f32.to_bits()]
    );
    assert_eq!(
        (
            clip.geometry_array_50.pointer,
            clip.geometry_array_50.used,
            clip.geometry_array_50.allocated,
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (
            clip.geometry_array_5c.pointer,
            clip.geometry_array_5c.used,
            clip.geometry_array_5c.allocated,
        ),
        (0, 0, 0)
    );
    assert_eq!((clip.runtime_68, clip.animation_type), (0, 5));
    assert_eq!(clip.animation_type_padding, [0, 0, 0]);
    let clip_absolute = 12 + clip.offset as usize;
    assert_eq!(
        &artifact.payload[clip_absolute + 0x6d..clip_absolute + 0x70],
        &[0, 0, 0]
    );
    assert_eq!(clip.node_tree.node_count, 2);
    let root = &clip.node_tree.roots[0];
    assert_eq!(
        (root.number, root.name.as_str(), root.content_flags),
        (0, "root", 1)
    );
    assert!(root.mesh.is_none() && root.skin.is_none());
    assert_eq!(
        (
            root.children_header.pointer,
            root.children_header.used,
            root.children_header.allocated,
        ),
        (
            report.clips[0].nodes[0]
                .children_array_core_offset
                .expect("animation root must link its mesh-identity dummy") as u32,
            1,
            1,
        )
    );
    let mesh_dummy = &root.children[0];
    assert_eq!(
        (
            mesh_dummy.number,
            mesh_dummy.name.as_str(),
            mesh_dummy.content_flags,
        ),
        (1, "m2a_seg_5", 0x01)
    );
    assert!(mesh_dummy.mesh.is_none() && mesh_dummy.skin.is_none());
    assert_eq!(
        (
            root.controller_keys_header.pointer,
            root.controller_keys_header.used,
            root.controller_keys_header.allocated,
        ),
        (768, 2, 2)
    );
    assert_eq!(
        (
            root.controller_data_header.pointer,
            root.controller_data_header.used,
            root.controller_data_header.allocated,
        ),
        (792, 18, 18)
    );
    assert_eq!(
        (
            report.clips[0].nodes[0].core_offset,
            report.clips[0].nodes[0].children_array_core_offset,
            report.clips[0].nodes[0].controller_keys_core_offset,
            report.clips[0].nodes[0].controller_data_core_offset,
        ),
        (540, Some(652), Some(768), Some(792))
    );
    assert_eq!(
        root.controllers
            .iter()
            .map(|controller| controller.controller_type)
            .collect::<Vec<_>>(),
        vec![8, 20]
    );
    let translation = &root.controllers[0];
    assert_eq!(
        (
            translation.packed_byte,
            translation.interpolation_flags,
            translation.padding_byte,
            translation.row_count,
            translation.time_index,
            translation.data_index
        ),
        (3, 0, 0, 2, 0, 2)
    );
    assert_eq!(translation.times, vec![0.0, 1.0]);
    assert_eq!(
        translation.values,
        vec![vec![0.0, 0.0, 0.0], vec![0.25, -0.5, 1.0]]
    );
    let rotation = &root.controllers[1];
    assert_eq!(
        (
            rotation.packed_byte,
            rotation.interpolation_flags,
            rotation.padding_byte,
            rotation.row_count,
            rotation.time_index,
            rotation.data_index
        ),
        (4, 0, 0, 2, 8, 10)
    );
    assert!(rotation.values[1][2] > 0.0 && rotation.values[1][3] > 0.0);
}

#[test]
fn retail_rig_only_projection_keeps_skin_in_base_and_omits_it_from_type5_states() {
    type FlattenedNode = (u64, String, Option<u64>, u64, bool, bool, Vec<i64>);

    fn flatten(
        node: &serde_json::Value,
        parent_number: Option<u64>,
        output: &mut Vec<FlattenedNode>,
    ) {
        let number = node["number"].as_u64().expect("node number");
        output.push((
            number,
            node["name"].as_str().expect("node name").to_owned(),
            parent_number,
            node["contentFlags"].as_u64().expect("content flags"),
            !node["mesh"].is_null(),
            !node["skin"].is_null(),
            node["controllers"]
                .as_array()
                .expect("node controllers")
                .iter()
                .map(|controller| {
                    controller["controllerType"]
                        .as_i64()
                        .expect("controller type")
                })
                .collect(),
        ));
        for child in node["children"].as_array().expect("node children") {
            flatten(child, Some(number), output);
        }
    }

    let input = skin_creature();
    let animations = cpause1_set(20);
    let mut rig_only_options = options();
    rig_only_options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;

    let artifact = write_binary_mdl_with_animations(&input, &animations, &rig_only_options)
        .expect("retail rig-only state projection");
    assert_eq!(artifact.inspection.node_tree.node_count, 6);
    assert_eq!(artifact.inspection.animations.len(), 1);
    assert_eq!(artifact.inspection.animations[0].animation_type, 5);
    assert_eq!(artifact.inspection.animations[0].node_tree.node_count, 5);

    let mut base = Vec::new();
    flatten(
        &serde_json::to_value(&artifact.inspection.node_tree.roots[0]).unwrap(),
        None,
        &mut base,
    );
    let renderable = base
        .iter()
        .filter(|(_, _, _, _, has_mesh, has_skin, _)| *has_mesh || *has_skin)
        .collect::<Vec<_>>();
    assert_eq!(renderable.len(), 1);
    assert_eq!(renderable[0].1, "m2a_seg_5");
    assert!(renderable[0].4 && renderable[0].5);

    let mut state = Vec::new();
    flatten(
        &serde_json::to_value(&artifact.inspection.animations[0].node_tree.roots[0]).unwrap(),
        None,
        &mut state,
    );
    let expected_rig_identity = base
        .iter()
        .filter(|(_, _, _, _, has_mesh, has_skin, _)| !*has_mesh && !*has_skin)
        .map(|(number, name, parent, _, _, _, _)| (*number, name.clone(), *parent))
        .collect::<Vec<_>>();
    let state_identity = state
        .iter()
        .map(|(number, name, parent, _, _, _, _)| (*number, name.clone(), *parent))
        .collect::<Vec<_>>();
    assert_eq!(state_identity, expected_rig_identity);
    assert!(
        state
            .iter()
            .all(|(_, name, _, flags, has_mesh, has_skin, _)| {
                name != "m2a_seg_5" && *flags == 0x01 && !*has_mesh && !*has_skin
            })
    );

    let animated_bone = state
        .iter()
        .find(|(_, name, _, _, _, _, _)| name == "bone_c")
        .expect("animated bone_c");
    assert_eq!(animated_bone.6, vec![8, 20]);

    verify_direct_creature_state_projection_v1(
        &artifact.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .expect("rig-only output must pass only its own conformance family");
    let as_full_dummy = verify_direct_creature_state_projection_v1(
        &artifact.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect_err("rig-only output must fail the full-base dummy family");
    assert_eq!(as_full_dummy.code, "M2A-MDL-CONFORMANCE-STATE-NODE-MISSING");

    let full_dummy = write_binary_mdl_with_animations(&input, &animations, &options()).unwrap();
    let as_rig_only = verify_direct_creature_state_projection_v1(
        &full_dummy.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .expect_err("a state SkinMesh dummy must fail the rig-only family");
    assert_eq!(
        as_rig_only.code,
        "M2A-MDL-CONFORMANCE-RIG-ONLY-STATE-NODE-COUNT"
    );
}

#[test]
fn cep_rigid_placeholder_is_an_explicit_provenance_bound_family() {
    let animations = cpause1_set(70);
    let mut cep_options = options();
    let exact_provenance = cep_r3_provenance();
    cep_options.state_projection_profile = MdlStateProjectionProfileV1::CepRigidPlaceholderV1;
    cep_options.state_projection_provenance = Some(exact_provenance.clone());
    let artifact = write_binary_mdl_with_animations(&creature(), &animations, &cep_options)
        .expect("exact CEP R3 provenance admits the separate rigid-placeholder family");
    assert_eq!(
        artifact.report.state_projection_profile,
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1
    );
    let child = &artifact.inspection.animations[0].node_tree.roots[0].children[0];
    assert_eq!(child.content_flags, 0x21);
    let mesh = child.mesh.as_ref().expect("CEP state mesh placeholder");
    assert_eq!(mesh.vertex_count, 0);
    assert!(mesh.faces.is_empty() && mesh.raw_indices.is_empty() && child.skin.is_none());
    let independently_declared_expected_provenance = cep_r3_provenance();
    verify_direct_creature_state_projection_with_expected_provenance_v1(
        &artifact.inspection,
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1,
        Some(&exact_provenance),
        Some(&independently_declared_expected_provenance),
    )
    .expect("CEP output must pass only its own conformance family");
    let missing_runtime_provenance =
        verify_direct_creature_state_projection_with_expected_provenance_v1(
            &artifact.inspection,
            MdlStateProjectionProfileV1::CepRigidPlaceholderV1,
            None,
            Some(&independently_declared_expected_provenance),
        )
        .expect_err("CEP-shaped readback without provenance must fail conformance");
    assert_eq!(
        missing_runtime_provenance.code,
        "M2A-MDL-CONFORMANCE-PROVENANCE-MISSING"
    );
    let mut wrong_runtime_provenance = exact_provenance.clone();
    wrong_runtime_provenance.resource_sha256 = "0".repeat(64);
    let wrong_runtime_provenance_error =
        verify_direct_creature_state_projection_with_expected_provenance_v1(
            &artifact.inspection,
            MdlStateProjectionProfileV1::CepRigidPlaceholderV1,
            Some(&wrong_runtime_provenance),
            Some(&independently_declared_expected_provenance),
        )
        .expect_err("CEP-shaped readback with wrong provenance must fail conformance");
    assert_eq!(
        wrong_runtime_provenance_error.code,
        "M2A-MDL-CONFORMANCE-PROVENANCE-MISMATCH"
    );
    let missing_expected = verify_direct_creature_state_projection_v1(
        &artifact.inspection,
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1,
        Some(&exact_provenance),
    )
    .expect_err("CEP-shaped readback cannot declare its own expected provenance");
    assert_eq!(
        missing_expected.code,
        "M2A-MDL-CONFORMANCE-EXPECTED-PROVENANCE-MISSING"
    );
    let mixed = verify_direct_creature_state_projection_v1(
        &artifact.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect_err("a CEP placeholder must fail the retail dummy profile");
    assert_eq!(mixed.code, "M2A-MDL-CONFORMANCE-PROFILE-MIXED");

    let mut missing = cep_options.clone();
    missing.state_projection_provenance = None;
    assert_code(
        write_binary_mdl_with_animations(&creature(), &animations, &missing),
        "M4A-STATE-PROJECTION-PROVENANCE-MISSING",
    );

    let mut wrong_hash = cep_options.clone();
    wrong_hash
        .state_projection_provenance
        .as_mut()
        .unwrap()
        .resource_sha256 = "0".repeat(63);
    assert_code(
        write_binary_mdl_with_animations(&creature(), &animations, &wrong_hash),
        "M4A-STATE-PROJECTION-PROVENANCE-MISMATCH",
    );

    let mut mixed_retail = options();
    mixed_retail.state_projection_provenance = Some(cep_r3_provenance());
    assert_code(
        write_binary_mdl_with_animations(&creature(), &animations, &mixed_retail),
        "M4A-STATE-PROJECTION-PROFILE-MIXED",
    );

    let error = write_binary_mdl_with_animations(&skin_creature(), &animations, &cep_options)
        .expect_err("CEP rigid placeholder profile is not evidenced for skin");
    assert_eq!(error.code, "M4A-STATE-PROJECTION-CEP-SKIN-UNPROVEN");
}

#[test]
fn multiple_owned_clips_are_deterministic_and_do_not_mutate_inputs() {
    let input = skin_creature();
    let mut animation_set = cpause1_set(20);
    animation_set.clips.push(MdlAnimationClipV1 {
        name: "cwalk".to_owned(),
        animation_root: "owned_root".to_owned(),
        length_seconds: 0.5,
        transition_seconds: 0.0,
        events: Vec::new(),
        tracks: vec![MdlAnimationTrackV1 {
            target_node_id: 70,
            path: MdlAnimationTrackPathV1::Rotation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            times_seconds: vec![0.0, 0.5],
            values: vec![vec![0.0, 0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0, 1.0]],
        }],
    });
    let input_before = serde_json::to_vec(&input).unwrap();
    let animations_before = serde_json::to_vec(&animation_set).unwrap();

    let first = write_binary_mdl_with_animations(&input, &animation_set, &options()).unwrap();
    let second = write_binary_mdl_with_animations(&input, &animation_set, &options()).unwrap();
    assert_eq!(first.payload, second.payload);
    assert_eq!(first.report, second.report);
    assert_eq!(serde_json::to_vec(&input).unwrap(), input_before);
    assert_eq!(
        serde_json::to_vec(&animation_set).unwrap(),
        animations_before
    );
    assert_eq!(first.inspection.animations.len(), 2);
    assert_eq!(first.inspection.animations[1].events_header.used, 0);
    assert_eq!(first.inspection.animations[1].events_header.pointer, 0);
    for clip in &first.inspection.animations {
        assert_eq!(
            clip.node_tree.node_count,
            input.nodes.len() + input.segments.len()
        );
        fn assert_native_animation_tree(node: &serde_json::Value) -> usize {
            if node["contentFlags"] == 0x21 {
                assert_eq!(node["mesh"]["vertexCount"], 0);
                assert!(node["mesh"]["faces"].as_array().unwrap().is_empty());
                assert!(node["mesh"]["indexCounts"].as_array().unwrap().is_empty());
                assert!(node["mesh"]["rawIndices"].as_array().unwrap().is_empty());
                assert!(node["skin"].is_null());
                1 + node["children"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(assert_native_animation_tree)
                    .sum::<usize>()
            } else {
                assert_eq!(node["contentFlags"], 1);
                assert!(node["mesh"].is_null() && node["skin"].is_null());
                node["children"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(assert_native_animation_tree)
                    .sum()
            }
        }
        assert_eq!(
            assert_native_animation_tree(&serde_json::to_value(&clip.node_tree.roots[0]).unwrap()),
            0,
            "skin animation trees must remain generic until their placeholder profile is evidenced"
        );
    }
}

#[test]
fn full_creature_behavior_oracle_distinguishes_movement_and_terminal_death_offline() {
    let stable_hold =
        |name: &str| matches!(name, "ccastoutlp" | "cgetmidlp" | "ckdbckdie" | "cdead");
    let mut full = MdlAnimationSetV1 {
        schema_version: 1,
        clips: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .enumerate()
            .map(|(index, name)| MdlAnimationClipV1 {
                name: (*name).to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: vec![MdlAnimationTrackV1 {
                    target_node_id: 70,
                    path: MdlAnimationTrackPathV1::Translation,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: vec![0.0, 1.0],
                    values: vec![
                        vec![0.0, 0.0, 0.0],
                        vec![
                            if stable_hold(name) {
                                0.0
                            } else {
                                index as f32 + 1.0
                            },
                            0.0,
                            0.0,
                        ],
                    ],
                }],
            })
            .collect(),
    };
    let terminal_death_pose = full
        .clips
        .iter()
        .find(|clip| clip.name == "ckdbck")
        .expect("ckdbck")
        .tracks[0]
        .values[1]
        .clone();
    for hold_name in ["ckdbckdie", "cdead"] {
        full.clips
            .iter_mut()
            .find(|clip| clip.name == hold_name)
            .expect("death-family hold")
            .tracks[0]
            .values = vec![terminal_death_pose.clone(), terminal_death_pose.clone()];
    }
    let artifact = write_binary_mdl_with_animations(&creature(), &full, &options())
        .expect("full behavior fixture");
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert_eq!(report.schema_version, 2);
    assert!(report.full_namespace_complete);
    assert!(report.all_required_content_present);
    assert!(report.active_motion_complete);
    assert!(report.walk_run_distinct);
    assert!(report.essential_states_distinct);
    assert!(report.death_transition_terminal_pose);
    assert!(report.death_family_boundary_continuous);
    assert!(report.behavior_candidate_eligible);
    assert!(report.violations.is_empty());

    let mut indistinguishable = full.clone();
    let walk_values = indistinguishable
        .clips
        .iter()
        .find(|clip| clip.name == "cwalk")
        .expect("cwalk")
        .tracks[0]
        .values
        .clone();
    indistinguishable
        .clips
        .iter_mut()
        .find(|clip| clip.name == "crun")
        .expect("crun")
        .tracks[0]
        .values = walk_values;
    let artifact =
        write_binary_mdl_with_animations(&creature(), &indistinguishable, &options()).unwrap();
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert!(!report.walk_run_distinct);
    assert!(!report.behavior_candidate_eligible);
    assert!(
        report
            .violations
            .contains(&"WALK_RUN_NOT_DISTINCT".to_owned())
    );

    let mut aliased_damage = full.clone();
    let attack_values = aliased_damage
        .clips
        .iter()
        .find(|clip| clip.name == "ca1slashl")
        .expect("ca1slashl")
        .tracks[0]
        .values
        .clone();
    let damage = aliased_damage
        .clips
        .iter_mut()
        .find(|clip| clip.name == "cdamagel")
        .expect("cdamagel");
    damage.tracks[0].values = attack_values;
    damage.transition_seconds = 0.75;
    damage.events.push(MdlAnimationEventV1 {
        time_seconds: 0.5,
        name: "owned_damage_event".to_owned(),
    });
    let artifact =
        write_binary_mdl_with_animations(&creature(), &aliased_damage, &options()).unwrap();
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert!(!report.essential_states_distinct);
    assert!(!report.behavior_candidate_eligible);
    assert!(
        report
            .violations
            .contains(&"ESSENTIAL_STATES_NOT_DISTINCT".to_owned())
    );

    let mut discontinuous_death_transition = full.clone();
    discontinuous_death_transition
        .clips
        .iter_mut()
        .find(|clip| clip.name == "ckdbckdie")
        .expect("ckdbckdie")
        .tracks[0]
        .values = vec![vec![99.0, 0.0, 0.0], vec![99.0, 0.0, 0.0]];
    let artifact =
        write_binary_mdl_with_animations(&creature(), &discontinuous_death_transition, &options())
            .unwrap();
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert!(report.death_transition_terminal_pose);
    assert!(report.active_motion_complete);
    assert!(!report.death_family_boundary_continuous);
    assert!(!report.behavior_candidate_eligible);
    assert!(
        report
            .violations
            .contains(&"DEATH_FAMILY_BOUNDARY_DISCONTINUOUS:ckdbck->ckdbckdie".to_owned())
    );
    assert!(
        report
            .violations
            .contains(&"DEATH_FAMILY_BOUNDARY_DISCONTINUOUS:ckdbckdie->cdead".to_owned())
    );

    let mut moving_cdead_with_discontinuous_transition = full.clone();
    moving_cdead_with_discontinuous_transition
        .clips
        .iter_mut()
        .find(|clip| clip.name == "ckdbckdie")
        .expect("ckdbckdie")
        .tracks[0]
        .values = vec![vec![99.0, 0.0, 0.0], vec![99.0, 0.0, 0.0]];
    moving_cdead_with_discontinuous_transition
        .clips
        .iter_mut()
        .find(|clip| clip.name == "cdead")
        .expect("cdead")
        .tracks[0]
        .values[1] = vec![99.0, 0.0, 0.0];
    let artifact = write_binary_mdl_with_animations(
        &creature(),
        &moving_cdead_with_discontinuous_transition,
        &options(),
    )
    .unwrap();
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert!(report.active_motion_complete);
    assert!(report.death_transition_terminal_pose);
    assert!(!report.death_family_boundary_continuous);
    assert!(!report.behavior_candidate_eligible);
    assert!(
        report
            .violations
            .contains(&"DEATH_FAMILY_BOUNDARY_DISCONTINUOUS:ckdbck->ckdbckdie".to_owned())
    );

    let mut moving_dead_hold = full;
    moving_dead_hold
        .clips
        .iter_mut()
        .find(|clip| clip.name == "cdead")
        .expect("cdead")
        .tracks[0]
        .values[1] = vec![99.0, 0.0, 0.0];
    let artifact =
        write_binary_mdl_with_animations(&creature(), &moving_dead_hold, &options()).unwrap();
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert!(
        report.behavior_candidate_eligible,
        "native families prove that cdead may be either a stable or moving populated state: {:?}",
        report.violations
    );

    let mut missing_content = moving_dead_hold;
    missing_content
        .clips
        .iter_mut()
        .find(|clip| clip.name == "ctaunt")
        .expect("ctaunt")
        .tracks
        .clear();
    let artifact =
        write_binary_mdl_with_animations(&creature(), &missing_content, &options()).unwrap();
    let report = evaluate_direct_creature_animation_behavior_v2(&artifact.inspection);
    assert!(!report.all_required_content_present);
    assert!(!report.active_motion_complete);
    assert!(!report.behavior_candidate_eligible);
    assert!(
        report
            .violations
            .contains(&"CLIP_CONTENT_MISSING:ctaunt".to_owned())
    );
    assert!(
        report
            .violations
            .contains(&"ACTIVE_MOTION_MISSING:ctaunt".to_owned())
    );
}

#[test]
fn equal_positive_and_negative_zero_event_times_keep_input_order_and_bits() {
    let mut animation_set = cpause1_set(70);
    animation_set.clips[0].events = vec![
        MdlAnimationEventV1 {
            time_seconds: 0.0,
            name: "positive_zero_first".to_owned(),
        },
        MdlAnimationEventV1 {
            time_seconds: -0.0,
            name: "negative_zero_second".to_owned(),
        },
    ];
    let artifact = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("equal numeric event times are stably sorted");
    let events = &artifact.inspection.animations[0].events;
    assert_eq!(events[0].name, "positive_zero_first");
    assert_eq!(events[1].name, "negative_zero_second");
    assert_eq!(events[0].time.to_bits(), 0.0_f32.to_bits());
    assert_eq!(events[1].time.to_bits(), (-0.0_f32).to_bits());
}

#[test]
fn owned_cpause_fixture_has_measurable_translation_motion() {
    let animation_set = cpause1_set(70);
    let translation = animation_set.clips[0]
        .tracks
        .iter()
        .find(|track| track.path == MdlAnimationTrackPathV1::Translation)
        .unwrap();
    let first = &translation.values[0];
    let second = &translation.values[1];
    let distance = ((second[0] - first[0]).powi(2)
        + (second[1] - first[1]).powi(2)
        + (second[2] - first[2]).powi(2))
    .sqrt();
    assert!(distance >= 0.01);
    write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("owned loader-smoke fixture remains a legal writer input");
}

#[test]
fn arbitrary_zero_track_and_one_row_nonendpoint_clips_are_legal() {
    let animation_set = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![
            MdlAnimationClipV1 {
                name: "owned_idle".to_owned(),
                animation_root: "unmatched_owned_animroot".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.0,
                events: Vec::new(),
                tracks: Vec::new(),
            },
            MdlAnimationClipV1 {
                name: "owned_pose".to_owned(),
                animation_root: "also_not_a_node".to_owned(),
                length_seconds: 2.0,
                transition_seconds: 0.1,
                events: Vec::new(),
                tracks: vec![MdlAnimationTrackV1 {
                    target_node_id: 70,
                    path: MdlAnimationTrackPathV1::Translation,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: vec![0.75],
                    values: vec![vec![0.1, 0.2, 0.3]],
                }],
            },
        ],
    };
    let artifact = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("general writer does not impose loader-smoke policy");
    assert_eq!(artifact.inspection.animations.len(), 2);
    assert_eq!(artifact.inspection.animations[0].events_header.pointer, 0);
    assert!(
        artifact.inspection.animations[0].node_tree.roots[0]
            .controllers
            .is_empty()
    );
    let pose = &artifact.inspection.animations[1].node_tree.roots[0].controllers[0];
    assert_eq!(pose.row_count, 1);
    assert_eq!(pose.times, vec![0.75]);
}

#[test]
fn duplicate_output_node_names_are_rejected_globally_after_ascii_case_fold() {
    let mut branched = creature();
    branched.nodes.extend([
        AuroraCreatureNodeV1 {
            id: 71,
            name: "left".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 72,
            name: "right".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 73,
            name: "shared".to_owned(),
            parent_id: Some(71),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 74,
            name: "shared".to_owned(),
            parent_id: Some(72),
            bind_local_matrix: identity(),
        },
    ]);
    let error = write_binary_mdl(&branched, &options())
        .expect_err("global duplicate names remain ambiguous across branches");
    assert_eq!(error.code, "M4-NODE-NAME-DUPLICATE");
    assert_eq!(error.path, "creature.nodes[4].name");

    let mut case_folded = creature();
    case_folded.nodes.extend([
        AuroraCreatureNodeV1 {
            id: 71,
            name: "left".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 72,
            name: "right".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 73,
            name: "Bone".to_owned(),
            parent_id: Some(71),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 74,
            name: "bone".to_owned(),
            parent_id: Some(72),
            bind_local_matrix: identity(),
        },
    ]);
    let error = write_binary_mdl(&case_folded, &options())
        .expect_err("global node identity is ASCII case-insensitive");
    assert_eq!(error.code, "M4-NODE-NAME-DUPLICATE");
    assert_eq!(error.path, "creature.nodes[4].name");

    let mut generated_mesh_collision = creature();
    generated_mesh_collision.nodes.push(AuroraCreatureNodeV1 {
        id: 71,
        name: "M2A_SEG_5".to_owned(),
        parent_id: Some(70),
        bind_local_matrix: identity(),
    });
    let error = write_binary_mdl(&generated_mesh_collision, &options())
        .expect_err("rig names must not collide with generated mesh names");
    assert_eq!(error.code, "M4-NODE-NAME-DUPLICATE");
    assert_eq!(error.path, "creature.segments[0].segmentId");
}

#[test]
fn quaternion_sign_equivalents_produce_identical_animation_payloads() {
    let positive = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "owned_rotation".to_owned(),
            animation_root: "owned".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.0,
            events: Vec::new(),
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 70,
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.2, 0.8],
                values: vec![
                    vec![0.0, 0.0, 0.0, 1.0],
                    vec![
                        0.0,
                        0.0,
                        std::f32::consts::FRAC_1_SQRT_2,
                        std::f32::consts::FRAC_1_SQRT_2,
                    ],
                ],
            }],
        }],
    };
    let mut negative = positive.clone();
    for row in &mut negative.clips[0].tracks[0].values {
        for value in row {
            *value = -*value;
        }
    }
    let positive_artifact =
        write_binary_mdl_with_animations(&creature(), &positive, &options()).unwrap();
    let negative_artifact =
        write_binary_mdl_with_animations(&creature(), &negative, &options()).unwrap();
    assert_eq!(positive_artifact.payload, negative_artifact.payload);
}

#[test]
fn negative_consecutive_quaternion_dot_is_preserved_for_runtime_slerp() {
    fn globally_canonicalized_unit(mut q: [f32; 4]) -> [f32; 4] {
        let norm = q
            .iter()
            .map(|value| f64::from(*value).powi(2))
            .sum::<f64>()
            .sqrt();
        for value in &mut q {
            *value = (f64::from(*value) / norm) as f32;
        }
        if q[3] < 0.0 {
            for value in &mut q {
                *value = -*value;
            }
        }
        q
    }

    let half_angle = std::f32::consts::FRAC_PI_3;
    let scale = 1.000_005_f32;
    let first = [
        0.0,
        0.0,
        -half_angle.sin() * scale,
        -half_angle.cos() * scale,
    ];
    let second = [
        0.0,
        0.0,
        half_angle.sin() * scale,
        -half_angle.cos() * scale,
    ];
    let input_dot = first
        .iter()
        .zip(second)
        .map(|(left, right)| left * right)
        .sum::<f32>();
    assert!(input_dot < 0.0);

    let mut animation_set = cpause1_set(70);
    animation_set.clips[0].events.clear();
    animation_set.clips[0].tracks = vec![MdlAnimationTrackV1 {
        target_node_id: 70,
        path: MdlAnimationTrackPathV1::Rotation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        times_seconds: vec![0.0, 1.0],
        values: vec![first.to_vec(), second.to_vec()],
    }];
    let artifact = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("runtime slerp owns consecutive-key shortest-path handling");
    let emitted = &artifact.inspection.animations[0].node_tree.roots[0].controllers[0].values;
    let expected = [
        globally_canonicalized_unit(first),
        globally_canonicalized_unit(second),
    ];
    for (actual, expected) in emitted.iter().zip(expected) {
        assert_eq!(
            actual
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert!(actual[3] > 0.0);
        let norm = actual.iter().map(|value| value * value).sum::<f32>();
        assert!((norm - 1.0).abs() <= f32::EPSILON);
    }
    let emitted_dot = emitted[0]
        .iter()
        .zip(&emitted[1])
        .map(|(left, right)| left * right)
        .sum::<f32>();
    assert!(
        emitted_dot < 0.0,
        "serialized keys stay globally canonical; runtime slerp handles dot < 0"
    );
    let shortest_angular_distance = 2.0 * emitted_dot.abs().acos();
    assert!((shortest_angular_distance - 2.0 * std::f32::consts::FRAC_PI_3).abs() <= 1.0e-5);
}

#[test]
fn quaternion_is_normalized_once_in_the_planner_before_emission() {
    let scale = 1.000_005_f32;
    let half_angle = 0.4_f32;
    let raw = [0.0, 0.0, half_angle.sin() * scale, half_angle.cos() * scale];
    let norm = raw
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    let expected = raw.map(|value| (f64::from(value) / norm) as f32);
    let animation_set = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "owned_normalized".to_owned(),
            animation_root: "owned".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.0,
            events: Vec::new(),
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 70,
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.5],
                values: vec![raw.to_vec()],
            }],
        }],
    };
    let artifact = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("unit-tolerance quaternion is normalized in the plan");
    let actual = &artifact.inspection.animations[0].node_tree.roots[0].controllers[0].values[0];
    assert_eq!(
        actual
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
}

#[test]
fn animation_writer_negative_contract_has_stable_codes_and_paths() {
    fn rejected(
        mut animation_set: MdlAnimationSetV1,
        edit: impl FnOnce(&mut MdlAnimationSetV1),
    ) -> m2a_core::mdl::MdlWriteError {
        edit(&mut animation_set);
        write_binary_mdl_with_animations(&creature(), &animation_set, &options())
            .expect_err("animation contract must reject the mutation")
    }

    let error = rejected(cpause1_set(70), |set| set.schema_version = 2);
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        (
            "M4A-ANIMATION-SET-SCHEMA-INVALID",
            "animationSet.schemaVersion"
        )
    );
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].name = "BAD-NONASCII-é".to_owned()
    });
    assert_eq!(error.code, "M4A-ANIMATION-NAME-INVALID");
    let error = rejected(cpause1_set(70), |set| set.clips[0].animation_root.clear());
    assert_eq!(error.code, "M4A-ANIMROOT-INVALID");
    let error = rejected(cpause1_set(70), |set| set.clips[0].length_seconds = 0.0);
    assert_eq!(error.code, "M4A-CLIP-LENGTH-INVALID");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].transition_seconds = -0.1
    });
    assert_eq!(error.code, "M4A-TRANSITION-INVALID");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].events[0].time_seconds = 1.1
    });
    assert_eq!(error.code, "M4A-EVENT-TIME-INVALID");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].target_node_id = 999
    });
    assert_eq!(error.code, "M4A-TRACK-TARGET-MISSING");
    let error = rejected(cpause1_set(70), |set| {
        let duplicate = set.clips[0].tracks[0].clone();
        set.clips[0].tracks.push(duplicate);
    });
    assert_eq!(error.code, "M4A-TRACK-DUPLICATE");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].interpolation = MdlAnimationInterpolationV1::Step
    });
    assert_eq!(error.code, "M4A-INTERPOLATION-UNSUPPORTED");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].path = MdlAnimationTrackPathV1::Weights
    });
    assert_eq!(error.code, "M4A-TRACK-PATH-UNSUPPORTED");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].times_seconds[1] = 0.0
    });
    assert_eq!(error.code, "M4A-TRACK-TIME-NOT-STRICT");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].values[0].pop();
    });
    assert_eq!(error.code, "M4A-TRACK-ARITY-INVALID");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].values.pop();
    });
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        (
            "M4A-TRACK-ARITY-INVALID",
            "animationSet.clips[0].tracks[1].values"
        )
    );
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].values[0] = vec![0.0, 0.0, 0.0, 0.0]
    });
    assert_eq!(error.code, "M4A-QUATERNION-INVALID");
    let error = rejected(cpause1_set(70), |set| {
        set.clips.push(set.clips[0].clone());
        set.clips[1].name = "CPAUSE1".to_owned();
    });
    assert_eq!(error.code, "M4A-ANIMATION-NAME-INVALID");

    for value in [String::new(), "x".repeat(64), "bad\0name".to_owned()] {
        let error = rejected(cpause1_set(70), |set| set.clips[0].name = value);
        assert_eq!(error.code, "M4A-ANIMATION-NAME-INVALID");
    }
    for value in [
        "x".repeat(64),
        "bad-nonascii-é".to_owned(),
        "bad\0root".to_owned(),
    ] {
        let error = rejected(cpause1_set(70), |set| set.clips[0].animation_root = value);
        assert_eq!(error.code, "M4A-ANIMROOT-INVALID");
    }
    for value in [
        String::new(),
        "x".repeat(32),
        "bad-nonascii-é".to_owned(),
        "bad\0event".to_owned(),
    ] {
        let error = rejected(cpause1_set(70), |set| set.clips[0].events[0].name = value);
        assert_eq!(error.code, "M4A-EVENT-NAME-INVALID");
    }
    for value in [-1.0, f32::NAN, f32::INFINITY] {
        let error = rejected(cpause1_set(70), |set| set.clips[0].length_seconds = value);
        assert_eq!(error.code, "M4A-CLIP-LENGTH-INVALID");
    }
    for value in [f32::NAN, f32::INFINITY] {
        let error = rejected(cpause1_set(70), |set| {
            set.clips[0].transition_seconds = value
        });
        assert_eq!(error.code, "M4A-TRANSITION-INVALID");
        let error = rejected(cpause1_set(70), |set| {
            set.clips[0].events[0].time_seconds = value
        });
        assert_eq!(error.code, "M4A-EVENT-TIME-INVALID");
    }
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].times_seconds[1] = -0.25
    });
    assert_eq!(error.code, "M4A-TRACK-TIME-NOT-STRICT");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].times_seconds[0] = f32::NAN
    });
    assert_eq!(error.code, "M4A-TRACK-TIME-NOT-STRICT");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].times_seconds[1] = 1.25
    });
    assert_eq!(error.code, "M4A-TRACK-TIME-OOB");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[1].values[0][0] = f32::NAN
    });
    assert_eq!(error.code, "M4A-TRACK-VALUE-NONFINITE");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].values[0] = vec![0.0, 0.0, 0.0, 1.001]
    });
    assert_eq!(error.code, "M4A-QUATERNION-INVALID");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].interpolation = MdlAnimationInterpolationV1::CubicSpline
    });
    assert_eq!(error.code, "M4A-INTERPOLATION-UNSUPPORTED");
    let error = rejected(cpause1_set(70), |set| {
        set.clips[0].tracks[0].path = MdlAnimationTrackPathV1::Weights
    });
    assert_eq!(error.code, "M4A-TRACK-PATH-UNSUPPORTED");
    for value in [
        String::new(),
        "x".repeat(32),
        "bad-nonascii-é".to_owned(),
        "bad\0node".to_owned(),
    ] {
        let mut bad_creature = creature();
        bad_creature.nodes[0].name = value;
        let error = write_binary_mdl_with_animations(&bad_creature, &cpause1_set(70), &options())
            .expect_err("invalid node name");
        assert_eq!(error.code, "M4-INVALID-NAME");
    }
}

#[test]
fn animation_controller_evaluated_u16_boundary_is_preflighted() {
    fn large_translation(row_count: usize) -> MdlAnimationTrackV1 {
        let times = (0..row_count)
            .map(|index| index as f32 / (row_count - 1) as f32)
            .collect::<Vec<_>>();
        let values = (0..row_count)
            .map(|index| vec![index as f32 / (row_count - 1) as f32, 0.0, 0.0])
            .collect::<Vec<_>>();
        MdlAnimationTrackV1 {
            target_node_id: 70,
            path: MdlAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            times_seconds: times,
            values,
        }
    }

    let row_count = 16_384;
    let mut animation_set = cpause1_set(70);
    animation_set.clips[0].events.clear();
    animation_set.clips[0].tracks = vec![large_translation(row_count)];
    let artifact = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect("last evaluated translation data index u16::MAX is legal");
    let controller = &artifact.inspection.animations[0].node_tree.roots[0].controllers[0];
    assert_eq!(controller.row_count, row_count);
    assert_eq!(controller.data_index, row_count);
    assert_eq!(
        artifact.inspection.animations[0].node_tree.roots[0]
            .controller_data_header
            .used,
        usize::from(u16::MAX) + 1
    );

    animation_set.clips[0].tracks = vec![large_translation(row_count + 1)];
    let error = write_binary_mdl_with_animations(&creature(), &animation_set, &options())
        .expect_err("last evaluated translation data index exceeds u16");
    assert_eq!(error.code, "M4A-CONTROLLER-U16-OVERFLOW");
    assert_eq!(error.path, "animationSet.clips[0].tracks[0].timesSeconds");
}

#[test]
fn animation_rig_depth_is_preflighted_before_recursive_layout() {
    let mut input = creature();
    let mut parent_id = 70;
    for index in 1..258_u32 {
        let id = 1_000 + index;
        input.nodes.push(AuroraCreatureNodeV1 {
            id,
            name: format!("n{index}"),
            parent_id: Some(parent_id),
            bind_local_matrix: identity(),
        });
        parent_id = id;
    }
    let error = write_binary_mdl_with_animations(&input, &cpause1_set(70), &options())
        .expect_err("depth beyond own-reader guardrail must fail before recursive planning");
    assert_eq!(error.code, "M4A-LAYOUT-OVERFLOW");
    assert_eq!(error.path, "creature.nodes[257]");
}

#[test]
fn animation_reader_rejects_named_pointer_array_and_controller_mutations() {
    let artifact = write_binary_mdl_with_animations(&creature(), &cpause1_set(70), &options())
        .expect("mutation baseline");
    let clip = &artifact.inspection.animations[0];
    let root = &clip.node_tree.roots[0];
    let controller = &root.controllers[0];
    let core_absolute = |offset: u32| 12 + offset as usize;

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, 12 + 0x78 + 4, 2);
    let error = inspect_binary_mdl(&mutated).expect_err("model used > allocated");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");

    let mut mutated = artifact.payload.clone();
    let pointer_array = artifact.inspection.model.animation_pointers_header.pointer;
    write_u32_test(&mut mutated, core_absolute(pointer_array), u32::MAX);
    let error = inspect_binary_mdl(&mutated).expect_err("animation header pointer OOB");
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(clip.offset) + 0x48, u32::MAX);
    let error = inspect_binary_mdl(&mutated).expect_err("animation root pointer OOB");
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(clip.offset) + 0x4c, 0);
    let error = inspect_binary_mdl(&mutated).expect_err("declared animation budget mismatch");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");

    let mut mutated = artifact.payload.clone();
    write_u32_test(
        &mut mutated,
        core_absolute(clip.offset) + 0xbc,
        (clip.events_header.allocated + 1) as u32,
    );
    let error = inspect_binary_mdl(&mutated).expect_err("events used > allocated");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(clip.offset) + 0xb8, u32::MAX);
    let error = inspect_binary_mdl(&mutated).expect_err("event pointer OOB");
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(clip.offset) + 0xbc, 4);
    write_u32_test(&mut mutated, core_absolute(clip.offset) + 0xc0, 4);
    let error = inspect_binary_mdl(&mutated).expect_err("event stride must not overlap root tree");
    assert_eq!(error.code, "M2A-MDL-OFFSET-TYPE-CONFLICT");

    let mut mutated = artifact.payload.clone();
    write_u32_test(
        &mut mutated,
        core_absolute(root.offset) + 0x58,
        (root.controller_keys_header.allocated + 1) as u32,
    );
    let error = inspect_binary_mdl(&mutated).expect_err("keys used > allocated");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(root.offset) + 0x54, u32::MAX);
    let error = inspect_binary_mdl(&mutated).expect_err("key pointer OOB");
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");

    let mut mutated = artifact.payload.clone();
    write_u32_test(
        &mut mutated,
        core_absolute(root.offset) + 0x64,
        (root.controller_data_header.allocated + 1) as u32,
    );
    let error = inspect_binary_mdl(&mutated).expect_err("data used > allocated");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(root.offset) + 0x60, u32::MAX);
    let error = inspect_binary_mdl(&mutated).expect_err("controller data pointer OOB");
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");

    let mut mutated = artifact.payload.clone();
    let key = core_absolute(controller.key_offset);
    mutated[key + 8..key + 10].copy_from_slice(&u16::MAX.to_le_bytes());
    let error = inspect_binary_mdl(&mutated).expect_err("controller data index OOB");
    assert_eq!(error.code, "M2A-MDL-CONTROLLER-INDEX-OOB");

    let mut mutated = artifact.payload.clone();
    mutated[key + 6..key + 8].copy_from_slice(&u16::MAX.to_le_bytes());
    let error = inspect_binary_mdl(&mutated).expect_err("controller time index OOB");
    assert_eq!(error.code, "M2A-MDL-CONTROLLER-INDEX-OOB");

    let mut mutated = artifact.payload.clone();
    mutated[key + 10] = 0;
    let error = inspect_binary_mdl(&mutated).expect_err("zero packed columns");
    assert_eq!(error.code, "M2A-MDL-CONTROLLER-LAYOUT-INVALID");

    for packed in [4_u8, 0x23] {
        let mut mutated = artifact.payload.clone();
        mutated[key + 10] = packed;
        let error = inspect_binary_mdl(&mutated).expect_err("invalid type8 packed byte");
        assert_eq!(error.code, "M2A-MDL-CONTROLLER-LAYOUT-INVALID");
    }
    let rotation_key = core_absolute(root.controllers[1].key_offset);
    let mut mutated = artifact.payload.clone();
    mutated[rotation_key + 10] = 3;
    let error = inspect_binary_mdl(&mutated).expect_err("invalid type20 low nibble");
    assert_eq!(error.code, "M2A-MDL-CONTROLLER-LAYOUT-INVALID");

    let child_artifact =
        write_binary_mdl_with_animations(&skin_creature(), &cpause1_set(20), &options())
            .expect("child pointer mutation baseline");
    let child_root = &child_artifact.inspection.animations[0].node_tree.roots[0];
    let mut mutated = child_artifact.payload.clone();
    write_u32_test(
        &mut mutated,
        core_absolute(child_root.offset) + 0x48,
        u32::MAX,
    );
    let error = inspect_binary_mdl(&mutated).expect_err("child pointer array OOB");
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");

    let mut mutated = artifact.payload.clone();
    write_u32_test(&mut mutated, core_absolute(clip.offset) + 0x68, 7);
    let inspected = inspect_binary_mdl(&mutated).expect("opaque runtime mutation remains visible");
    assert_eq!(inspected.animations[0].runtime_68, 7);

    let mut mutated = artifact.payload.clone();
    mutated[core_absolute(clip.offset) + 0x6c] = 7;
    let inspected = inspect_binary_mdl(&mutated).expect("animation type mutation remains visible");
    assert_eq!(inspected.animations[0].animation_type, 7);

    let mut mutated = artifact.payload.clone();
    mutated[core_absolute(clip.offset) + 0x6d] = 1;
    let inspected =
        inspect_binary_mdl(&mutated).expect("animation padding mutation remains visible");
    assert_eq!(inspected.animations[0].animation_type_padding, [1, 0, 0]);

    let mut mutated = artifact.payload.clone();
    let name = core_absolute(root.offset) + 0x20;
    mutated[name..name + 32].fill(0);
    mutated[name..name + 7].copy_from_slice(b"changed");
    let inspected = inspect_binary_mdl(&mutated).expect("node name mutation remains visible");
    assert_eq!(inspected.animations[0].node_tree.roots[0].name, "changed");
}

#[test]
fn every_animation_payload_truncated_prefix_returns_without_panicking() {
    let complete = write_binary_mdl_with_animations(&creature(), &cpause1_set(70), &options())
        .expect("animation truncation baseline")
        .payload;
    for length in 0..complete.len() {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            inspect_binary_mdl(&complete[..length])
        }));
        assert!(
            outcome.is_ok(),
            "animation payload prefix {length} panicked"
        );
        assert!(
            outcome.unwrap().is_err(),
            "truncated animation payload prefix {length} parsed"
        );
    }
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

    let mut cycle = creature();
    cycle.nodes.extend([
        AuroraCreatureNodeV1 {
            id: 71,
            name: "cycle_a".to_owned(),
            parent_id: Some(72),
            bind_local_matrix: identity(),
        },
        AuroraCreatureNodeV1 {
            id: 72,
            name: "cycle_b".to_owned(),
            parent_id: Some(71),
            bind_local_matrix: identity(),
        },
    ]);
    let error = write_binary_mdl(&cycle, &options()).expect_err("disconnected cycle must fail");
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        ("M4-HIERARCHY-INVALID", "creature.nodes")
    );
    assert_eq!(error.message, "rig hierarchy contains a cycle");

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
fn face_surface_ids_roundtrip_and_require_one_value_per_triangle() {
    let mut input = creature();
    input.segments[0].face_surface_ids = vec![7];

    let artifact = write_binary_mdl(&input, &options()).expect("surface-aware mesh must write");
    let mesh = artifact.inspection.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("mesh readback");
    assert_eq!(mesh.faces.len(), 1);
    assert_eq!(mesh.faces[0].surface_id, 7);

    input.segments[0].indices.extend_from_slice(&[0, 1, 2]);
    assert_code(write_binary_mdl(&input, &options()), "M4-MESH-INVALID");
}

#[test]
fn nwn_ee_per_mesh_index_boundary_accepts_20k_and_rejects_21846_triangles() {
    let mut stress = creature();
    stress.profile_id = "synthetic-rigid-20k-triangle-stress".to_owned();
    stress.segments[0].indices = [0_u32, 1, 2].repeat(20_000);

    let artifact =
        write_binary_mdl(&stress, &options()).expect("20,000 triangles must fit one EE mesh");
    assert_eq!(artifact.report.projection.triangle_count, 20_000);
    let mesh = artifact.inspection.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("read back 20k mesh");
    assert_eq!(mesh.faces.len(), 20_000);
    assert_eq!(mesh.index_counts, [60_000]);
    assert_eq!(mesh.raw_indices[0].len(), 60_000);

    stress.segments[0].indices = [0_u32, 1, 2].repeat(NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1 + 1);
    assert_code(write_binary_mdl(&stress, &options()), "M4-MESH-LIMIT");
}

#[test]
fn placeable_shadow_adjacency_crosses_render_vertex_splits_at_uv_seams() {
    let mut input = creature();
    input.segments[0].positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    input.segments[0].normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    input.segments[0].uv0 = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];
    input.segments[0].indices = vec![0, 1, 2, 3, 4, 5];

    let artifact = write_binary_mdl(&input, &placeable_options()).expect("placeable seam fixture");
    let mesh = artifact.inspection.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("mesh readback");

    assert_eq!(mesh.faces[0].adjacent_faces, [-1, 1, -1]);
    assert_eq!(mesh.faces[1].adjacent_faces, [-1, -1, 0]);
    assert_eq!(mesh.raw_indices, [vec![0, 1, 2, 3, 4, 5]]);
}

#[test]
fn placeable_shadow_adjacency_leaves_non_manifold_edges_open() {
    let mut input = creature();
    input.segments[0].positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    input.segments[0].normals = vec![[0.0, 0.0, 1.0]; 9];
    input.segments[0].uv0 = vec![[0.0, 0.0]; 9];
    input.segments[0].indices = (0_u32..9).collect();

    let artifact =
        write_binary_mdl(&input, &placeable_options()).expect("non-manifold seam fixture");
    let mesh = artifact.inspection.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("mesh readback");

    assert_eq!(mesh.faces[0].adjacent_faces[0], -1);
    assert_eq!(mesh.faces[1].adjacent_faces[0], -1);
    assert_eq!(mesh.faces[2].adjacent_faces[0], -1);
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
