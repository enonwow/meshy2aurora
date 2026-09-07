use std::collections::BTreeSet;

use m2a_core::{
    profile_a::{Bounds3V1, RigWeightInfluenceV1},
    reference_supermodel_generic::{
        ReferenceSupermodelChainPayloadV2, ReferenceSupermodelChainResourceV2,
        ReferenceSupermodelFormatV2, analyze_reference_supermodel_chain_v2,
        audit_generic_reference_skin_influences_v3, derive_generic_reference_rig_from_surface_v2,
        derive_immutable_reference_supermodel_rig_from_surface_v3,
        inspect_reference_supermodel_mdl_v2, validate_authored_generic_reference_rig_v3,
    },
    reference_supermodel_motion::{
        REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2, ReferenceSupermodelCarrierClassV3,
        ReferenceSupermodelMotionContractV2, ReferenceSupermodelMotionNodeV2,
        canonical_reference_supermodel_motion_contract_sha256_v2,
        default_reference_supermodel_motion_tolerances_v2,
    },
    reference_supermodel_product::reference_supermodel_resource_sha256_v1,
    reference_supermodel_structure::fit_reference_supermodel_joints_v1,
};

fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

#[test]
fn immutable_supermodel_application_preserves_exact_bind_and_registers_only_the_mesh() {
    let family = contract(
        "c_immutable",
        &[
            ("root", None, [0.0, 0.0, 0.0]),
            ("body", Some(0), [0.0, 0.0, 0.4]),
        ],
    );
    let positions = vec![
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 2.0],
        [1.0, -1.0, 2.0],
        [1.0, 1.0, 2.0],
        [-1.0, 1.0, 2.0],
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3,
        3, 7, 4, 3, 4, 0,
    ];
    let reference_render_bounds = Bounds3V1 {
        min: [-0.5, -1.0, 0.0],
        max: [0.5, 1.0, 1.0],
    };

    let artifact = derive_immutable_reference_supermodel_rig_from_surface_v3(
        &family,
        &positions,
        &indices,
        reference_render_bounds,
    )
    .expect("register owned geometry to an immutable selected-supermodel bind");

    assert_eq!(
        artifact.report.algorithm,
        "IMMUTABLE_REFERENCE_BIND_MESH_REGISTRATION_V1"
    );
    assert_eq!(artifact.rig.nodes.len(), family.nodes.len());
    for (actual, expected) in artifact.rig.nodes.iter().zip(&family.nodes) {
        assert_eq!(actual.id, expected.part_number);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.parent_id, expected.parent_part_number);
        assert_eq!(actual.bind_local_matrix, expected.carrier_bind_local_matrix);
    }
    assert_eq!(
        artifact.report.joint_fit.algorithm,
        "IMMUTABLE_REFERENCE_BIND_V1"
    );
    assert!(
        artifact
            .report
            .joint_fit
            .joints
            .iter()
            .all(|joint| joint.reference_world_position == joint.target_world_position)
    );
    assert!(
        (artifact.rig.target_bounds.max[2] - artifact.rig.target_bounds.min[2] - 1.0).abs()
            <= 1.0e-6
    );
    assert_eq!(artifact.rig.alignment_anchor, [0.0, 0.0, 0.0]);
}

fn multiply_matrix(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0_f32; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

fn contract(
    family: &str,
    nodes: &[(&str, Option<u32>, [f32; 3])],
) -> ReferenceSupermodelMotionContractV2 {
    let mut contract = ReferenceSupermodelMotionContractV2 {
        schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
        contract_id: format!("synthetic-{family}-structural-v2"),
        content_sha256: String::new(),
        supermodel_resref: family.to_owned(),
        source_model_sha256: "1".repeat(64),
        inspected_read_only: true,
        no_payload_copied: true,
        classification: 4,
        animation_scale: 1.0,
        clean_room_profile_id: "exact-inspected-supermodel-bind-v3".to_owned(),
        clean_room_profile_sha256: "2".repeat(64),
        nodes: nodes
            .iter()
            .enumerate()
            .map(
                |(part, (name, parent, point))| ReferenceSupermodelMotionNodeV2 {
                    part_number: part as u32,
                    name: (*name).to_owned(),
                    parent_part_number: *parent,
                    carrier_bind_local_matrix: translation(point[0], point[1], point[2]),
                    position_controller_required: part == 0,
                    orientation_controller_required: part > 0,
                    scale_controller_required: false,
                    anchor_role: None,
                    joint_axis: None,
                    carrier_class: if part == 0 {
                        ReferenceSupermodelCarrierClassV3::PassiveStructural
                    } else {
                        ReferenceSupermodelCarrierClassV3::SkinRelevant
                    },
                    structural_role: if part == 0 { "ROOT" } else { "CHAIN" }.to_owned(),
                    controlling_clips: if part == 0 {
                        Vec::new()
                    } else {
                        vec!["move".to_owned()]
                    },
                    dynamic_clips: if part == 0 {
                        Vec::new()
                    } else {
                        vec!["move".to_owned()]
                    },
                },
            )
            .collect(),
        carrier_exclusions: Vec::new(),
        required_clips: vec!["move".to_owned()],
        required_events: Vec::new(),
        tolerances: default_reference_supermodel_motion_tolerances_v2(),
    };
    contract.content_sha256 =
        canonical_reference_supermodel_motion_contract_sha256_v2(&contract).unwrap();
    contract
}

#[test]
fn structurally_different_families_derive_without_name_or_species_whitelists() {
    let families = [
        contract(
            "c_stag",
            &[
                ("root", None, [0.0, 0.0, 0.0]),
                ("spine", Some(0), [0.0, 0.4, 0.3]),
                ("antler_l", Some(1), [-0.4, 0.2, 0.5]),
                ("antler_r", Some(1), [0.4, 0.2, 0.5]),
            ],
        ),
        contract(
            "c_serpent",
            &[
                ("root", None, [0.0, 0.0, 0.0]),
                ("s1", Some(0), [0.0, 0.3, 0.0]),
                ("s2", Some(1), [0.0, 0.3, 0.0]),
                ("s3", Some(2), [0.0, 0.3, 0.0]),
                ("s4", Some(3), [0.0, 0.3, 0.0]),
            ],
        ),
        contract(
            "c_flyer",
            &[
                ("root", None, [0.0, 0.0, 0.0]),
                ("chest", Some(0), [0.0, 0.2, 0.2]),
                ("wing_l", Some(1), [-0.8, 0.0, 0.0]),
                ("wing_r", Some(1), [0.8, 0.0, 0.0]),
                ("tail", Some(0), [0.0, -0.8, 0.0]),
            ],
        ),
    ];
    let positions = vec![
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
        // Deliberate seam duplicate: equal positions must receive equal weights.
        [-1.0, -1.0, 0.0],
    ];
    for family in families {
        let fit = fit_reference_supermodel_joints_v1(&family, &positions)
            .unwrap_or_else(|error| panic!("{}: {error}", family.supermodel_resref));
        assert_eq!(fit.fitted_world_positions.len(), family.nodes.len());
        assert_eq!(fit.report.used_aabb_seed_only_count, 0);
        assert_eq!(fit.report.carrier_node_count, family.nodes.len());
    }
}

#[test]
fn ground_contact_fitting_uses_all_four_surface_contacts_when_front_terminals_start_at_midbody() {
    let mut family = contract(
        "c_quadruped",
        &[
            ("root", None, [0.0, 0.0, 0.0]),
            ("ribcage", Some(0), [0.0, 0.0, 1.0]),
            ("front_l", Some(1), [-0.5, 0.0, -1.0]),
            ("front_r", Some(1), [0.5, 0.0, -1.0]),
            ("pelvis", Some(0), [0.0, -1.0, 1.0]),
            ("back_l", Some(4), [-0.5, 0.0, -1.0]),
            ("back_r", Some(4), [0.5, 0.0, -1.0]),
            ("head", Some(1), [0.0, 1.0, 0.0]),
        ],
    );
    for part in [2_usize, 3, 5, 6] {
        family.nodes[part].structural_role = "LIMB_GROUND_CONTACT_TERMINAL".to_owned();
    }

    let mut positions = Vec::new();
    for (center_x, center_y) in [(-1.0_f32, 2.0_f32), (1.0, 2.0), (-1.0, -2.0), (1.0, -2.0)] {
        for row in 0..3 {
            for column in 0..3 {
                positions.push([
                    center_x + (column as f32 - 1.0) * 0.08,
                    center_y + (row as f32 - 1.0) * 0.08,
                    0.0,
                ]);
            }
        }
    }
    positions.extend_from_slice(&[
        [-1.2, -2.2, 1.0],
        [1.2, -2.2, 1.0],
        [1.2, 2.2, 2.0],
        [-1.2, 2.2, 2.0],
    ]);

    let fit = fit_reference_supermodel_joints_v1(&family, &positions)
        .expect("fit a quadruped whose initial front terminal pivots are far from the paws");
    assert_eq!(fit.report.fitted_ground_contact_chain_count, 4);
    for part in [2_usize, 3, 5, 6] {
        assert!(fit.fitted_world_positions[part][2] <= 0.02);
    }
}

#[test]
fn full_inherited_clip_union_adds_late_only_joint_and_reports_passive_end_node() {
    let ascii = b"newmodel c_full\nsetsupermodel c_full NULL\nclassification CHARACTER\nsetanimationscale 1\nbeginmodelgeom c_full\nnode dummy c_full\nparent NULL\nposition 0 0 0\nendnode\nnode dummy torso\nparent c_full\nposition 0 0 1\nendnode\nnode dummy late_joint\nparent torso\nposition 0 1 0\nendnode\nnode dummy attachment_end\nparent late_joint\nposition 0 0.5 0\nendnode\nendmodelgeom c_full\nnewanim cpause1 c_full\nlength 1\nanimroot c_full\nnode dummy torso\nparent c_full\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.1\nendnode\ndoneanim cpause1 c_full\nnewanim cwalk c_full\nlength 1\nanimroot c_full\nnode dummy torso\nparent c_full\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.2\nendnode\ndoneanim cwalk c_full\nnewanim crun c_full\nlength 1\nanimroot c_full\nnode dummy torso\nparent c_full\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.3\nendnode\ndoneanim crun c_full\nnewanim cspecial c_full\nlength 1\nanimroot c_full\nnode dummy late_joint\nparent torso\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.8\nendnode\ndoneanim cspecial c_full\ndonemodel c_full\n".to_vec();
    let payloads = vec![ReferenceSupermodelChainPayloadV2 {
        resource: ReferenceSupermodelChainResourceV2 {
            resref: "c_full".to_owned(),
            supermodel_resref: "NULL".to_owned(),
            format: ReferenceSupermodelFormatV2::Ascii,
            sha256: reference_supermodel_resource_sha256_v1(&ascii),
            byte_length: ascii.len(),
        },
        payload: ascii,
    }];

    let analysis = analyze_reference_supermodel_chain_v2("c_full", &payloads).unwrap();
    assert_eq!(
        analysis.motion_contract.required_clips,
        vec!["cpause1", "cwalk", "crun", "cspecial"]
    );
    let late = analysis
        .motion_contract
        .nodes
        .iter()
        .find(|node| node.name == "late_joint")
        .expect("joint controlled only by a non-locomotion clip must be emitted");
    assert_eq!(
        late.carrier_class,
        ReferenceSupermodelCarrierClassV3::SkinRelevant
    );
    assert_eq!(late.dynamic_clips, vec!["cspecial"]);
    let passive = analysis
        .motion_contract
        .nodes
        .iter()
        .find(|node| node.name == "attachment_end")
        .expect("passive attachment/end node must remain in the complete hierarchy");
    assert_eq!(
        passive.carrier_class,
        ReferenceSupermodelCarrierClassV3::PassiveAttachmentOrEnd
    );
    assert!(passive.dynamic_clips.is_empty());
}

#[test]
fn influence_audit_counts_real_positive_references_and_blocks_unweighted_required_joint() {
    let family = contract(
        "c_audit",
        &[
            ("root", None, [0.0, 0.0, 0.0]),
            ("left", Some(0), [-1.0, 0.0, 0.0]),
            ("right", Some(0), [1.0, 0.0, 0.0]),
            ("socket", Some(0), [0.0, 0.0, 1.0]),
        ],
    );
    let mut family = family;
    family.nodes[3].carrier_class = ReferenceSupermodelCarrierClassV3::PassiveAttachmentOrEnd;
    family.nodes[3].controlling_clips.clear();
    family.nodes[3].dynamic_clips.clear();
    let weights = vec![
        vec![RigWeightInfluenceV1 {
            bone_node_id: 1,
            value: 1.0,
        }],
        vec![
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.5,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.5,
            },
        ],
    ];

    let report = audit_generic_reference_skin_influences_v3(&family, &[1, 2], &weights)
        .expect("audit well-formed normalized weight rows");
    assert_eq!(report.allowed_bone_count, 2);
    assert_eq!(report.active_weighted_bone_count, 1);
    assert_eq!(report.weighted_bone_count, 1);
    assert_eq!(report.unweighted_required_joint_names, vec!["right"]);
    assert_eq!(
        report.passive_unweighted_joint_names,
        vec!["root", "socket"]
    );
    assert!(!report.skin_influence_coverage);
}

#[test]
fn disconnected_surface_components_receive_component_stable_weights() {
    let family = contract(
        "c_componental",
        &[
            ("root", None, [0.0, 0.0, 0.0]),
            ("left", Some(0), [-1.0, 0.0, 0.0]),
            ("right", Some(0), [1.0, 0.0, 0.0]),
        ],
    );
    let positions = vec![
        [-1.0, -0.1, 0.0],
        [-0.9, -0.1, 0.0],
        [-0.9, 0.1, 0.0],
        [-1.0, 0.1, 0.0],
        [0.9, -0.1, 0.0],
        [1.0, -0.1, 0.0],
        [1.0, 0.1, 0.0],
        [0.9, 0.1, 0.0],
    ];
    let indices = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
    let rig = derive_generic_reference_rig_from_surface_v2(&family, &positions, &indices)
        .expect("derive disconnected structural surface");
    let weights = &rig.rig.segments[0].reference_weights;
    assert!(weights.iter().all(|row| {
        !row.is_empty()
            && row.len() <= 4
            && (row.iter().map(|influence| influence.value).sum::<f32>() - 1.0).abs() <= 1.0e-5
    }));
    assert!(
        weights[0..4]
            .iter()
            .all(|row| row[0].bone_node_id == weights[0][0].bone_node_id)
    );
    assert!(
        weights[4..8]
            .iter()
            .all(|row| row[0].bone_node_id == weights[4][0].bone_node_id)
    );
    assert_ne!(weights[0][0].bone_node_id, weights[4][0].bone_node_id);
    assert!(weights[0..4].windows(2).all(|pair| pair[0] == pair[1]));
    assert!(weights[4..8].windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(rig.report.stabilized_small_component_count, 0);
    assert_eq!(rig.report.stabilized_small_component_vertex_count, 0);
    assert!(rig.report.skin_influence_coverage);
}

#[test]
fn authored_joint_override_is_revalidated_and_cannot_collapse_a_carrier_chain() {
    let family = contract(
        "c_authored_validation",
        &[
            ("root", None, [0.0, 0.0, 0.0]),
            ("body", Some(0), [0.0, 0.0, 0.5]),
        ],
    );
    let positions = vec![
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3,
        3, 7, 4, 3, 4, 0,
    ];
    let base = derive_generic_reference_rig_from_surface_v2(&family, &positions, &indices)
        .expect("derive a base rig for authored validation");
    let mut authored = base.rig.clone();
    authored.nodes[1].bind_local_matrix[12] = 0.0;
    authored.nodes[1].bind_local_matrix[13] = 0.0;
    authored.nodes[1].bind_local_matrix[14] = 0.0;

    let validated = validate_authored_generic_reference_rig_v3(
        &family,
        &base.report,
        &authored,
        &BTreeSet::from([1]),
    )
    .expect("a geometrically valid authoring document must return a blocked validation report");

    assert_eq!(validated.joint_fit.status, "NEEDS_AUTHORING");
    assert_eq!(validated.bind_pose.status, "BLOCKED");
    assert_eq!(validated.joint_fit.joints[1].constraint_verdict, "BLOCKED");
    assert!(
        validated
            .joint_fit
            .constraint_violations
            .iter()
            .any(|constraint| constraint.code == "POSITIVE_BOUNDED_SEGMENT_LENGTHS")
    );
}

#[test]
fn fitted_bind_preserves_reference_local_frames_and_reconstructs_world_pivots() {
    let mut family = contract(
        "c_rotated",
        &[
            ("root", None, [0.0, 0.0, 0.0]),
            ("rotated_joint", Some(0), [0.0, 0.5, 0.5]),
            ("child", Some(1), [0.0, 0.5, 0.0]),
        ],
    );
    // This regression exercises carrier-frame reconstruction, not visible
    // skin coverage.  Keep the terminal as a structural carrier so the test
    // does not manufacture a semantic surface cluster for it on a tiny cube.
    family.nodes[2].carrier_class = ReferenceSupermodelCarrierClassV3::PassiveStructural;
    family.nodes[1].carrier_bind_local_matrix = [
        0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.5, 1.0,
    ];
    let positions = vec![
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3,
        3, 7, 4, 3, 4, 0,
    ];
    let artifact = derive_generic_reference_rig_from_surface_v2(&family, &positions, &indices)
        .expect("derive a frame-preserving target rig");
    assert_eq!(artifact.report.joint_fit.status, "NEEDS_AUTHORING");
    assert_eq!(artifact.report.bind_pose.status, "BLOCKED");

    for (target, reference) in artifact.rig.nodes.iter().zip(&family.nodes) {
        for index in [0_usize, 1, 2, 4, 5, 6, 8, 9, 10] {
            assert_eq!(
                target.bind_local_matrix[index], reference.carrier_bind_local_matrix[index],
                "local bind basis drifted for {} at matrix element {index}",
                target.name
            );
        }
    }

    let mut worlds = Vec::new();
    for node in &artifact.rig.nodes {
        let world = node
            .parent_id
            .map(|parent| multiply_matrix(worlds[parent as usize], node.bind_local_matrix))
            .unwrap_or(node.bind_local_matrix);
        worlds.push(world);
    }
    for (world, fitted) in worlds.iter().zip(&artifact.report.joint_fit.joints) {
        for axis in 0..3 {
            assert!(
                (world[12 + axis] - fitted.target_world_position[axis]).abs() <= 1.0e-4,
                "world pivot drifted for {} on axis {axis}",
                fitted.joint_name
            );
        }
    }
}

#[test]
fn ascii_mdl_parser_exposes_full_hierarchy_controllers_and_animation() {
    let ascii = br#"
newmodel c_ascii
setsupermodel c_ascii NULL
classification CHARACTER
setanimationscale 1.25
beginmodelgeom c_ascii
node dummy c_ascii
  parent NULL
  position 0 0 0
endnode
node dummy torso
  parent c_ascii
  position 0 0 1
  orientation 0 0 1 0
endnode
node dummy wing_l
  parent torso
  position -1 0 0
endnode
node dummy wing_r
  parent torso
  position 1 0 0
endnode
endmodelgeom c_ascii
newanim fly c_ascii
  length 1.0
  transtime 0.25
  animroot c_ascii
  node dummy c_ascii
    parent NULL
    positionkey 2
      0.0 0 0 0
      1.0 0 0.5 0
  endnode
  node dummy wing_l
    parent torso
    orientationkey 2
      0.0 0 1 0 0
      1.0 0 1 0 1.0
  endnode
doneanim fly c_ascii
donemodel c_ascii
"#;

    let report = inspect_reference_supermodel_mdl_v2(ascii, ReferenceSupermodelFormatV2::Ascii)
        .expect("parse structural ASCII MDL");
    assert_eq!(report.format, "NWN_ASCII_MDL");
    assert_eq!(report.model.name, "c_ascii");
    assert_eq!(report.model.classification, 4);
    assert_eq!(report.model.animation_scale, 1.25);
    assert_eq!(report.node_tree.node_count, 4);
    assert_eq!(report.node_tree.max_depth, 3);
    assert_eq!(report.animations.len(), 1);
    assert_eq!(report.animations[0].name, "fly");
    assert_eq!(report.animations[0].node_tree.node_count, 3);
    assert!(
        report.animations[0].node_tree.roots[0]
            .controllers
            .iter()
            .any(|controller| controller.controller_name.as_deref() == Some("position"))
    );
}

#[test]
fn format_is_detected_from_exact_bytes_not_from_a_family_name() {
    let ascii = b"newmodel c_any\nsetsupermodel c_any NULL\nclassification CHARACTER\nbeginmodelgeom c_any\nnode dummy c_any\nparent NULL\nposition 0 0 0\nendnode\nendmodelgeom c_any\ndonemodel c_any\n";
    let report = inspect_reference_supermodel_mdl_v2(ascii, ReferenceSupermodelFormatV2::Auto)
        .expect("auto-detect ASCII");
    assert_eq!(report.model.name, "c_any");
    assert_eq!(report.format, "NWN_ASCII_MDL");
}

#[test]
fn full_ascii_chain_is_sha_bound_and_inherited_motion_is_analyzed_from_parent() {
    let parent = b"newmodel c_parent\nsetsupermodel c_parent NULL\nclassification CHARACTER\nsetanimationscale 1\nbeginmodelgeom c_parent\nnode dummy c_parent\nparent NULL\nposition 0 0 0\nendnode\nnode dummy joint\nparent c_parent\nposition 0 0 1\nendnode\nendmodelgeom c_parent\nnewanim move c_parent\nlength 1\nanimroot c_parent\nnode dummy c_parent\nparent NULL\npositionkey 2\n0 0 0 0\n1 0.25 0 0\nendnode\nnode dummy joint\nparent c_parent\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.5\nendnode\ndoneanim move c_parent\ndonemodel c_parent\n".to_vec();
    let selected = b"newmodel c_selected\nsetsupermodel c_selected c_parent\nclassification CHARACTER\nsetanimationscale 1\nbeginmodelgeom c_selected\nnode dummy c_selected\nparent NULL\nposition 0 0 0\nendnode\nnode dummy joint\nparent c_selected\nposition 0 0 1\nendnode\nendmodelgeom c_selected\ndonemodel c_selected\n".to_vec();
    let payloads = vec![
        ReferenceSupermodelChainPayloadV2 {
            resource: ReferenceSupermodelChainResourceV2 {
                resref: "c_selected".to_owned(),
                supermodel_resref: "c_parent".to_owned(),
                format: ReferenceSupermodelFormatV2::Ascii,
                sha256: reference_supermodel_resource_sha256_v1(&selected),
                byte_length: selected.len(),
            },
            payload: selected,
        },
        ReferenceSupermodelChainPayloadV2 {
            resource: ReferenceSupermodelChainResourceV2 {
                resref: "c_parent".to_owned(),
                supermodel_resref: "NULL".to_owned(),
                format: ReferenceSupermodelFormatV2::Ascii,
                sha256: reference_supermodel_resource_sha256_v1(&parent),
                byte_length: parent.len(),
            },
            payload: parent,
        },
    ];

    let analysis = analyze_reference_supermodel_chain_v2("c_selected", &payloads)
        .expect("analyze the exact selected-to-root chain");
    assert_eq!(
        analysis.report.status,
        "REFERENCE_SUPERMODEL_STRUCTURALLY_READY"
    );
    assert_eq!(analysis.report.exact_chain.len(), 2);
    assert_eq!(
        analysis.report.selected_format,
        ReferenceSupermodelFormatV2::Ascii
    );
    assert_eq!(analysis.report.inherited_animation_names, vec!["move"]);
    assert_eq!(analysis.motion_contract.supermodel_resref, "c_selected");
    assert_eq!(analysis.motion_contract.required_clips, vec!["move"]);
    assert_eq!(analysis.combined_reference.animations.len(), 1);
    assert!(analysis.report.structural_errors.is_empty());
    assert!(!analysis.report.retail_payload_copied);
}

#[test]
fn semantic_tail_anchors_are_discovered_from_the_selected_structure_not_family_name() {
    let ascii = b"newmodel c_tailed\nsetsupermodel c_tailed NULL\nclassification CHARACTER\nsetanimationscale 1\nbeginmodelgeom c_tailed\nnode dummy c_tailed\nparent NULL\nposition 0 0 0\nendnode\nnode dummy appendage_tail\nparent c_tailed\nposition 0 -1 0\nendnode\nnode dummy appendage_tailend\nparent appendage_tail\nposition 0 -1 0\nendnode\nendmodelgeom c_tailed\nnewanim move c_tailed\nlength 1\nanimroot c_tailed\nnode dummy c_tailed\nparent NULL\npositionkey 2\n0 0 0 0\n1 0 0.1 0\nendnode\nnode dummy appendage_tail\nparent c_tailed\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.4\nendnode\nnode dummy appendage_tailend\nparent appendage_tail\norientationkey 2\n0 0 0 1 0\n1 0 0 1 0.2\nendnode\ndoneanim move c_tailed\ndonemodel c_tailed\n".to_vec();
    let payloads = vec![ReferenceSupermodelChainPayloadV2 {
        resource: ReferenceSupermodelChainResourceV2 {
            resref: "c_tailed".to_owned(),
            supermodel_resref: "NULL".to_owned(),
            format: ReferenceSupermodelFormatV2::Ascii,
            sha256: reference_supermodel_resource_sha256_v1(&ascii),
            byte_length: ascii.len(),
        },
        payload: ascii,
    }];
    let analysis = analyze_reference_supermodel_chain_v2("c_tailed", &payloads).unwrap();
    let tail_roles = analysis
        .motion_contract
        .nodes
        .iter()
        .filter_map(|node| node.anchor_role.as_deref())
        .filter(|role| role.starts_with("tail"))
        .collect::<Vec<_>>();
    assert_eq!(tail_roles, vec!["tail_0", "tail_1"]);
    let surface = [
        [-0.2, -2.0, 0.0],
        [0.2, -2.0, 0.0],
        [0.2, -1.0, 0.0],
        [-0.2, -1.0, 0.0],
    ];
    let fit = fit_reference_supermodel_joints_v1(&analysis.motion_contract, &surface).unwrap();
    assert!(fit.report.fitted_appendage_chain_count >= 1);
    assert!(
        fit.report
            .joints
            .iter()
            .filter(|joint| joint.joint_name.starts_with("appendage_tail"))
            .all(|joint| joint.provenance == "surface_appendage_centerline")
    );
}
