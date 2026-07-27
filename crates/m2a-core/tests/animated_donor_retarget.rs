#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod fixtures;

use m2a_core::{
    animated_donor::{
        retarget_static_mesh_to_animated_donor_v1, retarget_static_mesh_to_animated_donor_v2,
        retarget_static_mesh_to_animated_donor_v3, retarget_static_mesh_to_animated_donor_v4,
        retarget_static_mesh_to_animated_donor_v5,
    },
    mdl::{
        MdlAnimationTrackPathV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlStateProjectionProfileV1, MdlWriterOptionsV1,
    },
    profile_a::RigSegmentDeformationV1,
};

fn linear_animated_donor() -> Vec<u8> {
    let donor = fixtures::mutate_json(
        fixtures::skin_animation_with_inverse_bind_matrices(),
        |root| {
            root["animations"][0]["samplers"][1]["interpolation"] = serde_json::json!("LINEAR");
            root["animations"][0]["samplers"]
                .as_array_mut()
                .expect("synthetic animation samplers")
                .truncate(2);
            root["animations"][0]["channels"]
                .as_array_mut()
                .expect("synthetic animation channels")
                .truncate(2);
        },
    );
    // Give the synthetic donor's target surface positive extent on every axis.
    fixtures::mutate_accessor_f32(donor, 0, 8, 0.5)
}

fn constant_root_scale_animated_donor(scale: f32, target_node: usize) -> Vec<u8> {
    let donor = fixtures::mutate_json(
        fixtures::skin_animation_with_inverse_bind_matrices(),
        |root| {
            root["animations"][0]["samplers"][1]["interpolation"] = serde_json::json!("LINEAR");
            root["animations"][0]["samplers"][2]["interpolation"] = serde_json::json!("LINEAR");
            root["animations"][0]["channels"][2]["target"]["node"] = serde_json::json!(target_node);
            let scale_accessor = root["animations"][0]["samplers"][2]["output"]
                .as_u64()
                .expect("synthetic scale accessor") as usize;
            root["accessors"][scale_accessor]["count"] = serde_json::json!(3);
        },
    );
    let donor = (0..9).fold(donor, |donor, scalar_index| {
        fixtures::mutate_accessor_f32(donor, 10, scalar_index, scale)
    });
    fixtures::mutate_accessor_f32(donor, 0, 8, 0.5)
}

fn writer_options() -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_donor01".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_donortex".to_owned(),
        }],
    }
}

#[test]
fn static_mesh_is_scaled_weighted_and_animated_on_user_provided_donor_rig() {
    let static_source = fixtures::minimal_indexed_triangle();
    let donor = linear_animated_donor();
    let source_before = static_source.clone();
    let donor_before = donor.clone();
    let options = writer_options();

    let artifact =
        retarget_static_mesh_to_animated_donor_v1(&static_source, &donor, &options).unwrap();
    let repeated =
        retarget_static_mesh_to_animated_donor_v1(&static_source, &donor, &options).unwrap();

    assert_eq!(static_source, source_before);
    assert_eq!(donor, donor_before);
    assert_eq!(artifact.model.payload, repeated.model.payload);
    assert_eq!(artifact.report, repeated.report);
    assert_eq!(artifact.animations, repeated.animations);
    assert_eq!(artifact.conversion, repeated.conversion);
    assert_ne!(artifact.report.source_sha256, artifact.report.donor_sha256);
    assert_eq!(artifact.report.model_resource_resref, "m2a_donor01");
    assert_eq!(
        artifact.report.animation_clip_names,
        [
            "cpause1",
            "cappear",
            "cwalk",
            "crun",
            "ca1slashl",
            "cdamagel",
            "cdead"
        ]
    );
    assert_eq!(artifact.report.local_animation_count, 7);
    assert!(artifact.report.active_bone_count > 0);
    assert!(artifact.report.uniform_scale.is_finite());
    assert!(artifact.report.uniform_scale > 0.0);

    let creature = artifact.conversion.creature.as_ref().unwrap();
    assert_eq!(
        creature
            .nodes
            .iter()
            .filter(|node| node.parent_id.is_none())
            .count(),
        1
    );
    assert_eq!(
        creature
            .nodes
            .iter()
            .find(|node| node.parent_id.is_none())
            .unwrap()
            .name,
        "m2a_donor01"
    );
    assert!(
        creature
            .segments
            .iter()
            .all(|segment| segment.deformation == RigSegmentDeformationV1::Skin)
    );
    assert!(
        creature
            .segments
            .iter()
            .flat_map(|segment| &segment.weights)
            .all(|weights| weights.influence_count > 0)
    );

    assert_eq!(artifact.animations.clips.len(), 7);
    assert!(
        artifact
            .animations
            .clips
            .iter()
            .all(|clip| clip.animation_root == "m2a_donor01")
    );
    assert!(artifact.animations.clips[0].tracks.iter().any(|track| {
        track.path == MdlAnimationTrackPathV1::Translation
            && track.values.windows(2).any(|rows| rows[0] != rows[1])
    }));

    assert_eq!(artifact.model.inspection.model.supermodel_name, "NULL");
    assert_eq!(
        artifact
            .model
            .inspection
            .model
            .animation_pointers_header
            .used,
        7
    );
    assert!(artifact.model.report.semantic_diff.is_empty());
}

#[test]
fn v2_inserts_an_unweighted_identity_aurora_root_without_retargeting_the_donor_joints() {
    let source = fixtures::minimal_indexed_triangle();
    let donor = linear_animated_donor();
    let mut options = writer_options();
    options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;
    options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;

    let v1 = retarget_static_mesh_to_animated_donor_v1(&source, &donor, &options).unwrap();
    let v2 = retarget_static_mesh_to_animated_donor_v2(&source, &donor, &options).unwrap();
    let v1_creature = v1.conversion.creature.as_ref().unwrap();
    let v2_creature = v2.conversion.creature.as_ref().unwrap();

    assert_eq!(v2_creature.segments, v1_creature.segments);
    assert_eq!(v2.report.active_bone_count, v1.report.active_bone_count);
    assert_eq!(v2.report.skin_segment_count, v1.report.skin_segment_count);
    assert_eq!(
        v2.report.local_animation_count,
        v1.report.local_animation_count
    );
    assert_eq!(v2.report.rig_node_count, v1.report.rig_node_count + 1);

    let aurora_root = v2_creature
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .expect("dedicated Aurora Root");
    assert_eq!(aurora_root.name, "m2a_donor01");
    assert_eq!(
        aurora_root.bind_local_matrix,
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0
        ]
    );
    let skeleton_root = v2_creature
        .nodes
        .iter()
        .find(|node| node.parent_id == Some(aurora_root.id))
        .expect("preserved donor skeleton root");
    assert_eq!(skeleton_root.name, "rig-root");
    assert_eq!(
        skeleton_root.bind_local_matrix,
        v1_creature
            .nodes
            .iter()
            .find(|node| node.parent_id.is_none())
            .expect("V1 skeleton root")
            .bind_local_matrix
    );
    assert!(
        v2_creature
            .segments
            .iter()
            .flat_map(|segment| &segment.weights)
            .flat_map(|weights| weights.bone_node_ids)
            .flatten()
            .all(|bone_id| bone_id != aurora_root.id)
    );
    assert!(v2.animations.clips.iter().all(|clip| {
        clip.animation_root == "m2a_donor01"
            && clip
                .tracks
                .iter()
                .all(|track| track.target_node_id != aurora_root.id)
    }));
    assert!(
        v2.model
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.node_count == v2.report.rig_node_count)
    );

    let mut pending = v2
        .model
        .inspection
        .node_tree
        .roots
        .iter()
        .collect::<Vec<_>>();
    let mut skin = None;
    while let Some(node) = pending.pop() {
        if node.skin.is_some() {
            skin = node.skin.as_ref();
            break;
        }
        pending.extend(&node.children);
    }
    let skin = skin.expect("V2 SkinMesh readback");
    assert_eq!(skin.node_to_bone_map[0], -1);
    assert_eq!(&skin.inline_mapping[..2], &[1, 2]);
    assert!(skin.inline_mapping[2..].iter().all(|ordinal| *ordinal == 0));
    assert_eq!(skin.q_header.used, 4);
    assert_eq!(skin.t_header.used, 4);
    assert_eq!(skin.constants_header.used, 4);
    assert!(v2.model.report.semantic_diff.is_empty());
}

#[test]
fn v3_reparents_skin_to_the_aurora_root_and_bakes_the_old_parent_transform() {
    let source = fixtures::minimal_indexed_triangle();
    let donor = fixtures::mutate_json(linear_animated_donor(), |root| {
        root["nodes"][0]["translation"] = serde_json::json!([1.0, 2.0, 3.0]);
        root["nodes"][0]["rotation"] = serde_json::json!([0.0, 0.0, 0.70710677, 0.70710677]);
    });
    let mut options = writer_options();
    options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;
    options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;

    let v2 = retarget_static_mesh_to_animated_donor_v2(&source, &donor, &options).unwrap();
    let v3 = retarget_static_mesh_to_animated_donor_v3(&source, &donor, &options).unwrap();
    let v2_creature = v2.conversion.creature.as_ref().unwrap();
    let v3_creature = v3.conversion.creature.as_ref().unwrap();
    let v2_root = v2_creature
        .nodes
        .iter()
        .find(|node| node.name == "m2a_donor01")
        .unwrap();
    let v2_skeleton_root = v2_creature
        .nodes
        .iter()
        .find(|node| node.name == "rig-root")
        .unwrap();
    let v3_root = v3_creature
        .nodes
        .iter()
        .find(|node| node.name == "m2a_donor01")
        .unwrap();
    let v2_segment = &v2_creature.segments[0];
    let v3_segment = &v3_creature.segments[0];

    assert_eq!(v2_segment.parent_node_id, v2_skeleton_root.id);
    assert_eq!(v3_segment.parent_node_id, v3_root.id);
    assert_eq!(v3_segment.weights, v2_segment.weights);
    assert_eq!(v3_segment.indices, v2_segment.indices);
    assert_eq!(v3_segment.uv0, v2_segment.uv0);
    assert_eq!(v3.animations, v2.animations);
    assert_eq!(v3_root.bind_local_matrix, v2_root.bind_local_matrix);
    for (v2_position, v3_position) in v2_segment.positions.iter().zip(&v3_segment.positions) {
        let expected = transform_point(v2_skeleton_root.bind_local_matrix, *v2_position);
        for axis in 0..3 {
            assert!((v3_position[axis] - expected[axis]).abs() <= 0.000001);
        }
    }
    for (v2_normal, v3_normal) in v2_segment.normals.iter().zip(&v3_segment.normals) {
        let expected = transform_direction(v2_skeleton_root.bind_local_matrix, *v2_normal);
        for axis in 0..3 {
            assert!((v3_normal[axis] - expected[axis]).abs() <= 0.000001);
        }
    }

    let model_root = &v3.model.inspection.node_tree.roots[0];
    assert_eq!(model_root.name, "m2a_donor01");
    let skin = model_root
        .children
        .iter()
        .find(|node| node.skin.is_some())
        .expect("SkinMesh directly below Aurora Root");
    assert_eq!(skin.parent_offset, Some(model_root.offset));
    assert_eq!(v3.model.inspection.node_tree.node_count, 4);
    assert!(
        v3.model
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.node_count == 3)
    );
    assert!(v3.model.report.semantic_diff.is_empty());
}

#[test]
fn v4_removes_only_a_constant_scale_track_from_the_weighted_skeleton_root() {
    let source = fixtures::minimal_indexed_triangle();
    let donor = constant_root_scale_animated_donor(1.1764704, 0);
    let mut options = writer_options();
    options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;
    options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;

    let v3 = retarget_static_mesh_to_animated_donor_v3(&source, &donor, &options).unwrap();
    let v4 = retarget_static_mesh_to_animated_donor_v4(&source, &donor, &options).unwrap();
    let skeleton_root_id = v4
        .conversion
        .creature
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.name == "rig-root")
        .unwrap()
        .id;

    assert_eq!(v4.conversion, v3.conversion);
    assert_eq!(v4.report.rig_node_count, v3.report.rig_node_count);
    assert_eq!(v4.report.skin_segment_count, v3.report.skin_segment_count);
    assert_eq!(v4.report.active_bone_count, v3.report.active_bone_count);
    assert_eq!(v4.animations.clips.len(), v3.animations.clips.len());
    for (v3_clip, v4_clip) in v3.animations.clips.iter().zip(&v4.animations.clips) {
        let v3_scale = v3_clip
            .tracks
            .iter()
            .filter(|track| track.path == MdlAnimationTrackPathV1::Scale)
            .collect::<Vec<_>>();
        assert_eq!(v3_scale.len(), 1);
        assert_eq!(v3_scale[0].target_node_id, skeleton_root_id);
        assert!(
            v3_scale[0]
                .values
                .iter()
                .all(|row| { row.len() == 1 && (row[0] - 1.1764704).abs() <= 0.000001 })
        );
        assert!(
            v4_clip
                .tracks
                .iter()
                .all(|track| track.path != MdlAnimationTrackPathV1::Scale)
        );
        assert_eq!(
            v4_clip.tracks,
            v3_clip
                .tracks
                .iter()
                .filter(|track| track.path != MdlAnimationTrackPathV1::Scale)
                .cloned()
                .collect::<Vec<_>>()
        );
    }
    assert!(v4.model.report.semantic_diff.is_empty());
    assert_ne!(v4.report.model_sha256, v3.report.model_sha256);
}

#[test]
fn v4_rejects_varying_or_non_root_animation_scale_tracks() {
    let source = fixtures::minimal_indexed_triangle();
    let mut options = writer_options();
    options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;
    options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;

    let varying = constant_root_scale_animated_donor(1.1764704, 0);
    let varying = (3..6).fold(varying, |donor, scalar_index| {
        fixtures::mutate_accessor_f32(donor, 10, scalar_index, 1.25)
    });
    let error = retarget_static_mesh_to_animated_donor_v4(&source, &varying, &options).unwrap_err();
    assert_eq!(error.code, "M7-DONOR-SKIN-SCALE-UNSUPPORTED");
    assert!(error.path.contains("values"));

    let non_root = constant_root_scale_animated_donor(1.1764704, 1);
    let error =
        retarget_static_mesh_to_animated_donor_v4(&source, &non_root, &options).unwrap_err();
    assert_eq!(error.code, "M7-DONOR-SKIN-SCALE-UNSUPPORTED");
    assert!(error.path.contains("targetNodeId"));

    let missing = linear_animated_donor();
    let error = retarget_static_mesh_to_animated_donor_v4(&source, &missing, &options).unwrap_err();
    assert_eq!(error.code, "M7-DONOR-SKIN-SCALE-MISSING");
}

#[test]
fn v5_replaces_skin_with_deterministic_bone_local_rigid_triangle_groups() {
    let source = fixtures::minimal_indexed_triangle();
    let donor = constant_root_scale_animated_donor(1.1764704, 0);
    let mut options = writer_options();
    options.format_profile =
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3;
    options.state_projection_profile =
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1;

    let v4 = retarget_static_mesh_to_animated_donor_v4(&source, &donor, &options).unwrap();
    let v5 = retarget_static_mesh_to_animated_donor_v5(&source, &donor, &options).unwrap();
    let repeated = retarget_static_mesh_to_animated_donor_v5(&source, &donor, &options).unwrap();
    let v4_creature = v4.conversion.creature.as_ref().expect("V4 creature");
    let v5_creature = v5.conversion.creature.as_ref().expect("V5 creature");

    assert_eq!(v5.model.payload, repeated.model.payload);
    assert_eq!(v5.report, repeated.report);
    assert_eq!(v5.animations, repeated.animations);
    assert_eq!(v5.animations, v4.animations);
    assert_eq!(v5.report.rig_node_count, v4.report.rig_node_count);
    assert_eq!(v5.report.skin_segment_count, 0);
    assert!(v5.report.active_bone_count > 0);
    assert!(!v5_creature.segments.is_empty());
    assert!(v5_creature.segments.iter().all(|segment| {
        segment.deformation == RigSegmentDeformationV1::Rigid
            && segment.weights.is_empty()
            && segment.indices.len().is_multiple_of(3)
            && segment.indices.len() == segment.positions.len()
            && segment.positions.len() == segment.normals.len()
            && segment.positions.len() == segment.uv0.len()
    }));
    assert_eq!(
        v5_creature
            .segments
            .iter()
            .map(|segment| segment.indices.len() / 3)
            .sum::<usize>(),
        v4_creature.segments[0].indices.len() / 3
    );
    assert_eq!(
        canonical_world_triangle_keys(v5_creature),
        canonical_world_triangle_keys(v4_creature),
        "rigid grouping must preserve the complete bind-pose world surface"
    );
    assert_eq!(
        canonical_uv_triangle_keys(v5_creature),
        canonical_uv_triangle_keys(v4_creature),
        "rigid grouping must preserve every source UV triangle"
    );

    let mut mesh_count = 0;
    let mut skin_count = 0;
    let mut pending = v5
        .model
        .inspection
        .node_tree
        .roots
        .iter()
        .collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        mesh_count += usize::from(node.mesh.is_some());
        skin_count += usize::from(node.skin.is_some());
        pending.extend(&node.children);
    }
    assert_eq!(mesh_count, v5_creature.segments.len());
    assert_eq!(skin_count, 0);
    assert_eq!(
        v5.model.inspection.node_tree.node_count,
        v5_creature.nodes.len() + v5_creature.segments.len()
    );
    assert!(v5.model.report.semantic_diff.is_empty());
}

#[test]
fn static_source_must_remain_unskinned_and_donor_must_supply_animation() {
    let donor = linear_animated_donor();
    let error =
        retarget_static_mesh_to_animated_donor_v1(&donor, &donor, &writer_options()).unwrap_err();
    assert_eq!(error.code, "M7-DONOR-STATIC-SOURCE-INELIGIBLE");

    let error = retarget_static_mesh_to_animated_donor_v1(
        &fixtures::minimal_indexed_triangle(),
        &fixtures::minimal_indexed_triangle(),
        &writer_options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M7-DONOR-RIG-INVALID");
}

fn transform_point(matrix: [f32; 16], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn transform_direction(matrix: [f32; 16], direction: [f32; 3]) -> [f32; 3] {
    let transformed = [
        matrix[0] * direction[0] + matrix[4] * direction[1] + matrix[8] * direction[2],
        matrix[1] * direction[0] + matrix[5] * direction[1] + matrix[9] * direction[2],
        matrix[2] * direction[0] + matrix[6] * direction[1] + matrix[10] * direction[2],
    ];
    let length = transformed
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    transformed.map(|value| value / length)
}

fn canonical_world_triangle_keys(
    creature: &m2a_core::profile_a::AuroraCreatureIrV1,
) -> Vec<[i64; 9]> {
    let worlds = creature_node_worlds(creature);
    let mut keys = creature
        .segments
        .iter()
        .flat_map(|segment| {
            let parent_world = worlds[&segment.parent_node_id];
            segment.indices.chunks_exact(3).map(move |triangle| {
                let mut key = [0_i64; 9];
                for (corner, &index) in triangle.iter().enumerate() {
                    let world = transform_point(parent_world, segment.positions[index as usize]);
                    for axis in 0..3 {
                        key[corner * 3 + axis] =
                            (f64::from(world[axis]) * 100_000.0).round() as i64;
                    }
                }
                key
            })
        })
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

fn canonical_uv_triangle_keys(creature: &m2a_core::profile_a::AuroraCreatureIrV1) -> Vec<[u32; 6]> {
    let mut keys = creature
        .segments
        .iter()
        .flat_map(|segment| {
            segment.indices.chunks_exact(3).map(|triangle| {
                let mut key = [0_u32; 6];
                for (corner, &index) in triangle.iter().enumerate() {
                    let uv = segment.uv0[index as usize];
                    key[corner * 2] = uv[0].to_bits();
                    key[corner * 2 + 1] = uv[1].to_bits();
                }
                key
            })
        })
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

fn assert_world_triangle_surfaces_close(
    actual: &m2a_core::profile_a::AuroraCreatureIrV1,
    expected: &m2a_core::profile_a::AuroraCreatureIrV1,
    tolerance: f32,
) {
    fn samples(creature: &m2a_core::profile_a::AuroraCreatureIrV1) -> Vec<([u32; 6], [f32; 9])> {
        let worlds = creature_node_worlds(creature);
        let mut samples = creature
            .segments
            .iter()
            .flat_map(|segment| {
                let parent_world = worlds[&segment.parent_node_id];
                segment.indices.chunks_exact(3).map(move |triangle| {
                    let mut uv_key = [0_u32; 6];
                    let mut world_key = [0_f32; 9];
                    for (corner, &index) in triangle.iter().enumerate() {
                        let index = index as usize;
                        let uv = segment.uv0[index];
                        uv_key[corner * 2] = uv[0].to_bits();
                        uv_key[corner * 2 + 1] = uv[1].to_bits();
                        let world = transform_point(parent_world, segment.positions[index]);
                        world_key[corner * 3..corner * 3 + 3].copy_from_slice(&world);
                    }
                    (uv_key, world_key)
                })
            })
            .collect::<Vec<_>>();
        samples.sort_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| {
                left.1
                    .iter()
                    .zip(&right.1)
                    .find_map(|(left, right)| {
                        let ordering = left.total_cmp(right);
                        ordering.ne(&std::cmp::Ordering::Equal).then_some(ordering)
                    })
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        samples
    }

    let actual = samples(actual);
    let expected = samples(expected);
    assert_eq!(actual.len(), expected.len());
    for (triangle_index, ((actual_uv, actual_world), (expected_uv, expected_world))) in
        actual.iter().zip(&expected).enumerate()
    {
        assert_eq!(
            actual_uv, expected_uv,
            "triangle {triangle_index} must preserve the source UV identity"
        );
        for (coordinate_index, (actual, expected)) in
            actual_world.iter().zip(expected_world).enumerate()
        {
            assert!(
                (actual - expected).abs() <= tolerance,
                "triangle {triangle_index} world coordinate {coordinate_index} differs: \
                 actual={actual}, expected={expected}, tolerance={tolerance}"
            );
        }
    }
}

fn creature_node_worlds(
    creature: &m2a_core::profile_a::AuroraCreatureIrV1,
) -> std::collections::BTreeMap<u32, [f32; 16]> {
    let mut worlds = std::collections::BTreeMap::new();
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
        assert!(
            remaining.len() < before,
            "synthetic creature hierarchy must be reachable"
        );
    }
    worlds
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

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_m0_retargets_to_the_real_h1_animated_donor_with_rig_only_states_for_r34() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p34".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };

    let artifact = retarget_static_mesh_to_animated_donor_v1(&source, &donor, &options).unwrap();

    assert_eq!(
        artifact.report.source_sha256,
        "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1"
    );
    assert_eq!(
        artifact.report.donor_sha256,
        "3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f"
    );
    assert_eq!(artifact.report.model_resource_resref, "m2a_m0p34");
    assert!(artifact.report.skin_segment_count > 0);
    assert!(artifact.report.active_bone_count > 0);
    assert!(artifact.report.local_animation_count > 0);
    assert!(artifact.report.uniform_scale.is_finite());
    assert!(artifact.report.uniform_scale > 0.0);
    assert!(artifact.model.report.semantic_diff.is_empty());
    assert_eq!(
        artifact
            .model
            .inspection
            .model
            .animation_pointers_header
            .used as usize,
        artifact.report.local_animation_count
    );
    assert!(
        artifact
            .conversion
            .creature
            .as_ref()
            .expect("retargeted creature")
            .segments
            .iter()
            .all(|segment| segment.deformation == RigSegmentDeformationV1::Skin)
    );
    assert!(
        artifact
            .model
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.node_count == artifact.report.rig_node_count)
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&artifact.report).expect("retarget report JSON")
    );
}

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_r34_to_native_zero_terminated_skin_changes_only_the_unused_inline_palette_tail() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let mut legacy_options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p34".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };
    let legacy =
        retarget_static_mesh_to_animated_donor_v1(&source, &donor, &legacy_options).unwrap();
    assert_eq!(
        legacy.report.model_sha256,
        "2fe4ad1ae4354335008916cbff3e0f724fedf0119f30f07aa8d5341c3d5b4af5"
    );

    legacy_options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2;
    let native =
        retarget_static_mesh_to_animated_donor_v1(&source, &donor, &legacy_options).unwrap();
    assert_eq!(legacy.model.payload.len(), native.model.payload.len());

    let mut pending = legacy
        .model
        .inspection
        .node_tree
        .roots
        .iter()
        .collect::<Vec<_>>();
    let mut skin = None;
    while let Some(node) = pending.pop() {
        if node.skin.is_some() {
            skin = node.skin.as_ref();
            break;
        }
        pending.extend(&node.children);
    }
    let skin = skin.expect("legacy r34 SkinMesh");
    let active_slots = skin
        .node_to_bone_map
        .iter()
        .filter(|slot| **slot >= 0)
        .count();
    assert_eq!(active_slots, 22);
    let tail_start = 12 + skin.node_offset as usize + 0x2b0 + active_slots * 2;
    let tail_end = 12 + skin.node_offset as usize + 0x330;
    let differences = legacy
        .model
        .payload
        .iter()
        .zip(&native.model.payload)
        .enumerate()
        .filter_map(|(offset, (left, right))| (left != right).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(differences, (tail_start..tail_end).collect::<Vec<_>>());
    assert!(
        legacy.model.payload[tail_start..tail_end]
            .iter()
            .all(|byte| *byte == 0xff)
    );
    assert!(
        native.model.payload[tail_start..tail_end]
            .iter()
            .all(|byte| *byte == 0)
    );

    legacy_options.model_resource_resref = "m2a_m0p35".to_owned();
    let r35 = retarget_static_mesh_to_animated_donor_v1(&source, &donor, &legacy_options).unwrap();
    assert_eq!(
        r35.report.model_sha256,
        "779d93fa762980ef17448762ba97d8f8775b03335b1d9561b8c2b6483772b3e0"
    );
}

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_m0_h1_v2_adds_only_a_dedicated_unweighted_aurora_root() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p36".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };

    let v1 = retarget_static_mesh_to_animated_donor_v1(&source, &donor, &options).unwrap();
    let v2 = retarget_static_mesh_to_animated_donor_v2(&source, &donor, &options).unwrap();
    let v1_creature = v1.conversion.creature.as_ref().expect("V1 creature");
    let v2_creature = v2.conversion.creature.as_ref().expect("V2 creature");

    assert_eq!(
        v2.report.source_sha256,
        "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1"
    );
    assert_eq!(
        v2.report.donor_sha256,
        "3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f"
    );
    assert_eq!(v1.report.rig_node_count, 24);
    assert_eq!(v2.report.rig_node_count, 25);
    assert_eq!(v2.report.active_bone_count, 22);
    assert_eq!(v2.report.skin_segment_count, 1);
    assert_eq!(v2.report.local_animation_count, 7);
    assert_eq!(v2_creature.segments, v1_creature.segments);
    assert_eq!(v2.animations, v1.animations);

    let aurora_root = &v2_creature.nodes[0];
    assert_eq!(aurora_root.name, "m2a_m0p36");
    assert_eq!(aurora_root.parent_id, None);
    let hips = &v2_creature.nodes[1];
    assert_eq!(hips.name, "Hips");
    assert_eq!(hips.parent_id, Some(aurora_root.id));
    assert_eq!(hips.id, v1_creature.nodes[0].id);
    assert_eq!(
        hips.bind_local_matrix,
        v1_creature.nodes[0].bind_local_matrix
    );
    assert!(
        v2_creature
            .segments
            .iter()
            .flat_map(|segment| &segment.weights)
            .flat_map(|weights| weights.bone_node_ids)
            .flatten()
            .all(|bone_id| bone_id != aurora_root.id)
    );
    assert!(
        v2.animations
            .clips
            .iter()
            .flat_map(|clip| &clip.tracks)
            .all(|track| track.target_node_id != aurora_root.id)
    );

    assert_eq!(v2.model.inspection.node_tree.node_count, 26);
    assert!(
        v2.model
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.node_count == 25)
    );
    let mut pending = v2
        .model
        .inspection
        .node_tree
        .roots
        .iter()
        .collect::<Vec<_>>();
    let mut skin = None;
    while let Some(node) = pending.pop() {
        if node.skin.is_some() {
            skin = node.skin.as_ref();
            break;
        }
        pending.extend(&node.children);
    }
    let skin = skin.expect("exact V2 SkinMesh");
    assert_eq!(skin.node_to_bone_map.len(), 26);
    assert_eq!(skin.node_to_bone_map[0], -1);
    assert_eq!(
        skin.node_to_bone_map
            .iter()
            .filter(|slot| **slot >= 0)
            .count(),
        22
    );
    assert_eq!(
        &skin.inline_mapping[..22],
        &(1_i16..=22).collect::<Vec<_>>()
    );
    assert!(
        skin.inline_mapping[22..]
            .iter()
            .all(|ordinal| *ordinal == 0)
    );
    assert_eq!(skin.q_header.used, 26);
    assert_eq!(skin.t_header.used, 26);
    assert_eq!(skin.constants_header.used, 26);
    assert!(v2.model.report.semantic_diff.is_empty());
    assert_eq!(
        v2.report.model_sha256,
        "459b9954d377c1daab9b12c73a2bf9a64507b5f3cf6d2a6a2ea7d751f680963a"
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&v2.report).expect("exact V2 report JSON")
    );
}

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_m0_h1_v3_reparents_skin_without_moving_bind_pose_world_geometry() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p37".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };

    let v2 = retarget_static_mesh_to_animated_donor_v2(&source, &donor, &options).unwrap();
    let v3 = retarget_static_mesh_to_animated_donor_v3(&source, &donor, &options).unwrap();
    let v2_creature = v2.conversion.creature.as_ref().expect("V2 creature");
    let v3_creature = v3.conversion.creature.as_ref().expect("V3 creature");
    let v2_hips = v2_creature
        .nodes
        .iter()
        .find(|node| node.name == "Hips")
        .expect("V2 Hips");
    let v3_root = v3_creature
        .nodes
        .iter()
        .find(|node| node.name == "m2a_m0p37")
        .expect("V3 Aurora Root");
    let v2_segment = &v2_creature.segments[0];
    let v3_segment = &v3_creature.segments[0];

    assert_eq!(v2.report.rig_node_count, 25);
    assert_eq!(v3.report.rig_node_count, 25);
    assert_eq!(v3.report.active_bone_count, 22);
    assert_eq!(v3.report.skin_segment_count, 1);
    assert_eq!(v3.report.local_animation_count, 7);
    assert_eq!(v2_segment.parent_node_id, v2_hips.id);
    assert_eq!(v3_segment.parent_node_id, v3_root.id);
    assert_eq!(v3_segment.weights, v2_segment.weights);
    assert_eq!(v3_segment.indices, v2_segment.indices);
    assert_eq!(v3_segment.uv0, v2_segment.uv0);
    assert_eq!(v3.animations, v2.animations);
    for (v2_position, v3_position) in v2_segment.positions.iter().zip(&v3_segment.positions) {
        let expected = transform_point(v2_hips.bind_local_matrix, *v2_position);
        for axis in 0..3 {
            assert!((v3_position[axis] - expected[axis]).abs() <= 0.000001);
        }
    }
    for (v2_normal, v3_normal) in v2_segment.normals.iter().zip(&v3_segment.normals) {
        let expected = transform_direction(v2_hips.bind_local_matrix, *v2_normal);
        for axis in 0..3 {
            assert!((v3_normal[axis] - expected[axis]).abs() <= 0.000001);
        }
    }

    let model_root = &v3.model.inspection.node_tree.roots[0];
    assert_eq!(model_root.name, "m2a_m0p37");
    let skin_node = model_root
        .children
        .iter()
        .find(|node| node.skin.is_some())
        .expect("exact SkinMesh directly below Aurora Root");
    assert_eq!(skin_node.parent_offset, Some(model_root.offset));
    let skin = skin_node.skin.as_ref().unwrap();
    assert_eq!(v3.model.inspection.node_tree.node_count, 26);
    assert_eq!(skin.node_to_bone_map.len(), 26);
    assert_eq!(skin.node_to_bone_map[0], -1);
    assert_eq!(
        skin.node_to_bone_map
            .iter()
            .filter(|slot| **slot >= 0)
            .count(),
        22
    );
    assert_eq!(
        &skin.inline_mapping[..22],
        &(1_i16..=22).collect::<Vec<_>>()
    );
    assert!(
        skin.inline_mapping[22..]
            .iter()
            .all(|ordinal| *ordinal == 0)
    );
    assert_eq!(skin.vertex_weights.len(), 2_380);
    assert!(
        v3.model
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.node_count == 25)
    );
    assert!(v3.model.report.semantic_diff.is_empty());
    assert_ne!(v3.report.model_sha256, v2.report.model_sha256);
    assert_eq!(
        v3.report.model_sha256,
        "48746e6e0b19bedbdcc8a364ff96cd583848dfa38e06971706bfb69b0341f676"
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&v3.report).expect("exact V3 report JSON")
    );
}

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_m0_h1_v4_removes_only_the_unrepresentable_root_scale_controllers() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p38".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };

    let v3 = retarget_static_mesh_to_animated_donor_v3(&source, &donor, &options).unwrap();
    let v4 = retarget_static_mesh_to_animated_donor_v4(&source, &donor, &options).unwrap();
    let repeated = retarget_static_mesh_to_animated_donor_v4(&source, &donor, &options).unwrap();
    let skeleton_root_id = v4
        .conversion
        .creature
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.name == "Hips")
        .expect("preserved Hips")
        .id;

    assert_eq!(v4.conversion, v3.conversion);
    assert_eq!(v4.model.payload, repeated.model.payload);
    assert_eq!(v4.report, repeated.report);
    assert_eq!(v4.report.rig_node_count, 25);
    assert_eq!(v4.report.skin_segment_count, 1);
    assert_eq!(v4.report.active_bone_count, 22);
    assert_eq!(v4.report.local_animation_count, 7);
    assert_eq!(v4.animations.clips.len(), v3.animations.clips.len());
    for (v3_clip, v4_clip) in v3.animations.clips.iter().zip(&v4.animations.clips) {
        let v3_scale = v3_clip
            .tracks
            .iter()
            .filter(|track| track.path == MdlAnimationTrackPathV1::Scale)
            .collect::<Vec<_>>();
        assert_eq!(v3_scale.len(), 1);
        assert_eq!(v3_scale[0].target_node_id, skeleton_root_id);
        assert!(
            v3_scale[0]
                .values
                .iter()
                .all(|row| { row.len() == 1 && (row[0] - 1.1764704).abs() <= 0.000001 })
        );
        assert_eq!(
            v4_clip.tracks,
            v3_clip
                .tracks
                .iter()
                .filter(|track| track.path != MdlAnimationTrackPathV1::Scale)
                .cloned()
                .collect::<Vec<_>>()
        );
    }

    let mut v3_scale_controller_count = 0;
    let mut v4_scale_controller_count = 0;
    for (inspection, count) in [
        (&v3.model.inspection, &mut v3_scale_controller_count),
        (&v4.model.inspection, &mut v4_scale_controller_count),
    ] {
        for animation in &inspection.animations {
            let mut pending = animation.node_tree.roots.iter().collect::<Vec<_>>();
            while let Some(node) = pending.pop() {
                *count += node
                    .controllers
                    .iter()
                    .filter(|controller| controller.controller_type == 36)
                    .count();
                pending.extend(&node.children);
            }
        }
    }
    assert_eq!(v3_scale_controller_count, 7);
    assert_eq!(v4_scale_controller_count, 0);
    assert!(v4.model.report.semantic_diff.is_empty());
    assert_ne!(v4.report.model_sha256, v3.report.model_sha256);
    assert_eq!(
        v4.report.model_sha256,
        "039d07cd937430d83006c7d0176aa7659440265417fb7fe53bf73b405563c248"
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&v4.report).expect("exact V4 report JSON")
    );
}

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_m0_h1_v5_preserves_the_surface_as_animated_rigid_triangle_groups() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile:
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p40".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };

    let v4 = retarget_static_mesh_to_animated_donor_v4(&source, &donor, &options).unwrap();
    let v5 = retarget_static_mesh_to_animated_donor_v5(&source, &donor, &options).unwrap();
    let repeated = retarget_static_mesh_to_animated_donor_v5(&source, &donor, &options).unwrap();
    let v4_creature = v4.conversion.creature.as_ref().expect("exact V4 creature");
    let v5_creature = v5.conversion.creature.as_ref().expect("exact V5 creature");

    assert_eq!(v5.model.payload, repeated.model.payload);
    assert_eq!(v5.report, repeated.report);
    assert_eq!(v5.animations, repeated.animations);
    assert_eq!(v5.animations, v4.animations);
    assert_eq!(v5.report.rig_node_count, 25);
    assert_eq!(v5.report.skin_segment_count, 0);
    assert_eq!(v5.report.local_animation_count, 7);
    assert_eq!(v5.report.active_bone_count, 20);
    assert_eq!(
        v5.report.model_sha256,
        "1809b05370e77f2c2559ec3e6fb354518bb5695ed48600954033175377fc0a40"
    );
    assert_eq!(v5_creature.segments.len(), 20);
    assert!(v5_creature.segments.iter().all(|segment| {
        segment.deformation == RigSegmentDeformationV1::Rigid && segment.weights.is_empty()
    }));
    assert_eq!(
        v5_creature
            .segments
            .iter()
            .map(|segment| segment.indices.len() / 3)
            .sum::<usize>(),
        1_569
    );
    assert_eq!(
        v5_creature
            .segments
            .iter()
            .map(|segment| segment.positions.len())
            .sum::<usize>(),
        4_707
    );
    assert_world_triangle_surfaces_close(v5_creature, v4_creature, 0.000_1);
    assert_eq!(
        canonical_uv_triangle_keys(v5_creature),
        canonical_uv_triangle_keys(v4_creature)
    );

    let root = &v5.model.inspection.node_tree.roots[0];
    assert_eq!(root.name, "m2a_m0p40");
    assert!(root.controllers.is_empty());
    let mut mesh_count = 0;
    let mut skin_count = 0;
    let mut pending = v5
        .model
        .inspection
        .node_tree
        .roots
        .iter()
        .collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        mesh_count += usize::from(node.mesh.is_some());
        skin_count += usize::from(node.skin.is_some());
        pending.extend(&node.children);
    }
    assert_eq!(mesh_count, v5_creature.segments.len());
    assert_eq!(skin_count, 0);
    assert_eq!(
        v5.model.inspection.node_tree.node_count,
        25 + v5_creature.segments.len()
    );
    assert_eq!(v5.model.inspection.node_tree.node_count, 45);
    assert!(v5.model.inspection.animations.iter().all(|animation| {
        animation.node_tree.node_count == 25
            && animation.animation_type == 5
            && animation
                .node_tree
                .roots
                .iter()
                .all(|root| root.controllers.is_empty())
    }));
    assert!(v5.model.report.semantic_diff.is_empty());

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "modelSha256": v5.report.model_sha256,
            "rigidSegmentCount": v5_creature.segments.len(),
            "activeBoneCount": v5.report.active_bone_count,
            "baseNodeCount": v5.model.inspection.node_tree.node_count,
            "triangleCount": 1569,
            "duplicatedVertexCount": 4707,
        }))
        .expect("exact V5 report JSON")
    );
}

#[test]
#[ignore = "requires the exact local M0 and Meshy H1 runtime-witness GLBs"]
fn exact_m0_h1_controllerless_root_profile_changes_only_the_base_root_controller_payload() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = std::fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source GLB");
    let donor = std::fs::read(repo.join("test-assets/meshy/incoming/h1-humanoid-1500.glb"))
        .expect("exact Meshy H1 donor GLB");
    let mut legacy_options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_m0p39".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_m0t01".to_owned(),
        }],
    };
    let legacy =
        retarget_static_mesh_to_animated_donor_v4(&source, &donor, &legacy_options).unwrap();
    legacy_options.format_profile =
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3;
    let candidate =
        retarget_static_mesh_to_animated_donor_v4(&source, &donor, &legacy_options).unwrap();
    let repeated =
        retarget_static_mesh_to_animated_donor_v4(&source, &donor, &legacy_options).unwrap();

    assert_eq!(candidate.conversion, legacy.conversion);
    assert_eq!(candidate.animations, legacy.animations);
    assert_eq!(candidate.model.payload, repeated.model.payload);
    assert_eq!(candidate.report, repeated.report);
    assert_eq!(
        candidate.model.report.layout.core_length + 60,
        legacy.model.report.layout.core_length
    );
    assert_eq!(
        candidate.model.report.layout.raw_length,
        legacy.model.report.layout.raw_length
    );
    let legacy_raw_start = 12 + legacy.model.report.layout.core_length;
    let candidate_raw_start = 12 + candidate.model.report.layout.core_length;
    assert_eq!(
        &candidate.model.payload[candidate_raw_start..],
        &legacy.model.payload[legacy_raw_start..]
    );
    let legacy_root = &legacy.model.inspection.node_tree.roots[0];
    let candidate_root = &candidate.model.inspection.node_tree.roots[0];
    assert_eq!(legacy_root.name, candidate_root.name);
    assert_eq!(legacy_root.controllers.len(), 2);
    assert!(candidate_root.controllers.is_empty());
    assert_eq!(candidate_root.controller_keys_header.used, 0);
    assert_eq!(candidate_root.controller_data_header.used, 0);
    assert!(candidate.model.report.semantic_diff.is_empty());
    assert!(
        candidate
            .model
            .inspection
            .animations
            .iter()
            .all(|animation| animation.node_tree.roots[0].controllers.is_empty())
    );
    assert_ne!(candidate.report.model_sha256, legacy.report.model_sha256);
    assert_eq!(
        candidate.report.model_sha256,
        "fab5ab98e9225c1553947f17994441273ae4c9bbd5a1d14034721ee3be2d86db"
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&candidate.report)
            .expect("exact controllerless-root report JSON")
    );
}
