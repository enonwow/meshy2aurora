use m2a_core::{
    profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        CreatureSourceForwardV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_profile_sha256,
    },
    reference_supermodel_authoring::{
        ReferenceSupermodelComponentBindingV2, ReferenceSupermodelJointOverrideV1,
        ReferenceSupermodelLandmarkOverrideV2, ReferenceSupermodelRegionWeightConstraintV2,
        ReferenceSupermodelVertexWeightOverrideV1, apply_reference_supermodel_rig_authoring_v1,
        apply_reference_supermodel_rig_authoring_v2,
        migrate_reference_supermodel_rig_authoring_v1_to_v2,
        new_reference_supermodel_rig_authoring_v1, new_reference_supermodel_rig_authoring_v2,
        seal_reference_supermodel_rig_authoring_v1, seal_reference_supermodel_rig_authoring_v2,
    },
};

const SOURCE_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CHAIN_SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const CONTRACT_SHA: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const STRUCTURE_SHA: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const ANATOMY_SHA: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

fn rig() -> CreatureRigProfileV1 {
    let mut rig = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: "authoring-test-rig".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds: Bounds3V1 {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        },
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes: vec![
            CreatureRigNodeV1 {
                id: 0,
                name: "root".to_owned(),
                parent_id: None,
                bind_local_matrix: translation(0.0, 0.0, 0.0),
            },
            CreatureRigNodeV1 {
                id: 1,
                name: "tail".to_owned(),
                parent_id: Some(0),
                bind_local_matrix: translation(0.0, -0.5, 0.2),
            },
        ],
        segments: vec![CreatureRigSegmentV1 {
            id: 7,
            name: "surface".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 0,
            surface_positions: vec![[0.0, 0.0, 0.0], [0.0, -0.5, 0.2], [0.1, -0.5, 0.2]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: vec![0, 1],
            reference_weights: vec![
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 0,
                    value: 1.0,
                }],
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 1.0,
                }],
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 1.0,
                }],
            ],
        }],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    rig
}

#[test]
fn joint_and_weight_edits_are_applied_without_mutating_carrier_topology() {
    let base = rig();
    let mut authoring = new_reference_supermodel_rig_authoring_v1(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap();
    authoring
        .joint_overrides
        .push(ReferenceSupermodelJointOverrideV1 {
            carrier_part_number: 1,
            bind_local_matrix: translation(0.0, -0.8, 0.35),
            semantic_role: Some("tail_base".to_owned()),
            joint_axis: Some([0.0, 1.0, 0.0]),
            locked: false,
        });
    authoring
        .weight_overrides
        .push(ReferenceSupermodelVertexWeightOverrideV1 {
            segment_id: 7,
            vertex_index: 2,
            influences: vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 0,
                    value: 0.25,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.75,
                },
            ],
        });
    let authoring = seal_reference_supermodel_rig_authoring_v1(authoring).unwrap();

    let applied = apply_reference_supermodel_rig_authoring_v1(
        &authoring,
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap();

    assert_eq!(applied.rig.nodes[0].name, "root");
    assert_eq!(applied.rig.nodes[1].name, "tail");
    assert_eq!(applied.rig.nodes[1].parent_id, Some(0));
    assert_eq!(
        applied.rig.nodes[1].bind_local_matrix,
        translation(0.0, -0.8, 0.35)
    );
    assert_eq!(applied.rig.segments[0].reference_weights[2].len(), 2);
    assert_ne!(applied.rig.content_sha256, base.content_sha256);
    assert_eq!(applied.report.joint_override_count, 1);
    assert_eq!(applied.report.weight_override_count, 1);
    assert_eq!(applied.report.authoring_sha256, authoring.content_sha256);
}

#[test]
fn stale_or_tampered_authoring_is_rejected() {
    let base = rig();
    let mut authoring = new_reference_supermodel_rig_authoring_v1(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap();
    authoring
        .joint_overrides
        .push(ReferenceSupermodelJointOverrideV1 {
            carrier_part_number: 1,
            bind_local_matrix: translation(0.0, -0.8, 0.35),
            semantic_role: None,
            joint_axis: None,
            locked: false,
        });
    let sealed = seal_reference_supermodel_rig_authoring_v1(authoring).unwrap();

    let stale = apply_reference_supermodel_rig_authoring_v1(
        &sealed,
        &"d".repeat(64),
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap_err();
    assert_eq!(
        stale.code,
        "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-MISMATCH"
    );

    let mut tampered = sealed;
    tampered.joint_overrides[0].bind_local_matrix[12] += 1.0;
    let hash = apply_reference_supermodel_rig_authoring_v1(
        &tampered,
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap_err();
    assert_eq!(
        hash.code,
        "M2A-REFERENCE-SUPERMODEL-AUTHORING-HASH-MISMATCH"
    );
}

#[test]
fn invalid_joint_matrix_and_weight_rows_fail_closed() {
    let base = rig();
    let mut authoring = new_reference_supermodel_rig_authoring_v1(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap();
    let mut singular = translation(0.0, 0.0, 0.0);
    singular[0] = 0.0;
    authoring
        .joint_overrides
        .push(ReferenceSupermodelJointOverrideV1 {
            carrier_part_number: 1,
            bind_local_matrix: singular,
            semantic_role: None,
            joint_axis: None,
            locked: false,
        });
    let error = seal_reference_supermodel_rig_authoring_v1(authoring).unwrap_err();
    assert_eq!(
        error.code,
        "M2A-REFERENCE-SUPERMODEL-AUTHORING-JOINT-MATRIX-INVALID"
    );

    let mut authoring = new_reference_supermodel_rig_authoring_v1(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap();
    authoring
        .weight_overrides
        .push(ReferenceSupermodelVertexWeightOverrideV1 {
            segment_id: 7,
            vertex_index: 0,
            influences: vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 0,
                    value: 0.25,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.25,
                },
            ],
        });
    let error = seal_reference_supermodel_rig_authoring_v1(authoring).unwrap_err();
    assert_eq!(
        error.code,
        "M2A-REFERENCE-SUPERMODEL-AUTHORING-WEIGHT-INVALID"
    );
}

#[test]
fn v1_migration_adds_context_binding_without_inventing_new_corrections() {
    let base = rig();
    let v1 = new_reference_supermodel_rig_authoring_v1(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        &base,
    )
    .unwrap();
    let v2 = migrate_reference_supermodel_rig_authoring_v1_to_v2(
        &v1,
        STRUCTURE_SHA,
        ANATOMY_SHA,
        "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
    )
    .unwrap();
    assert_eq!(v2.schema_version, 2);
    assert!(v2.landmark_overrides.is_empty());
    assert!(v2.component_bindings.is_empty());
    assert!(v2.region_weight_constraints.is_empty());
    assert_eq!(v2.joint_overrides, v1.joint_overrides);
    assert_eq!(v2.weight_overrides, v1.weight_overrides);
}

#[test]
fn v2_joint_and_landmark_mutation_is_rejected_for_immutable_supermodel_bind() {
    let base = rig();
    let mut document = new_reference_supermodel_rig_authoring_v2(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        STRUCTURE_SHA,
        ANATOMY_SHA,
        "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
        &base,
    )
    .unwrap();
    document
        .landmark_overrides
        .push(ReferenceSupermodelLandmarkOverrideV2 {
            landmark_id: "appendage_tip_0".to_owned(),
            carrier_part_number: 1,
            target_world_position: [0.0, -0.9, 0.35],
            locked: true,
        });
    let document = seal_reference_supermodel_rig_authoring_v2(document).unwrap();
    let error = apply_reference_supermodel_rig_authoring_v2(
        &document,
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        STRUCTURE_SHA,
        ANATOMY_SHA,
        "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
        &base,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        "M2A-REFERENCE-SUPERMODEL-IMMUTABLE-BIND-OVERRIDE-FORBIDDEN"
    );
}

#[test]
fn v2_component_and_region_weight_authoring_preserves_immutable_bind() {
    let base = rig();
    let mut document = new_reference_supermodel_rig_authoring_v2(
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        STRUCTURE_SHA,
        ANATOMY_SHA,
        "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
        &base,
    )
    .unwrap();
    document
        .component_bindings
        .push(ReferenceSupermodelComponentBindingV2 {
            segment_id: 7,
            component_index: 0,
            region_id: "owned_tail_surface".to_owned(),
            allowed_bone_node_ids: vec![0, 1],
            locked: true,
        });
    document
        .region_weight_constraints
        .push(ReferenceSupermodelRegionWeightConstraintV2 {
            segment_id: 7,
            region_id: "tail_tip".to_owned(),
            vertex_indices: vec![2],
            allowed_bone_node_ids: vec![1],
            forbidden_bone_node_ids: vec![0],
            maximum_influence_count: 1,
            locked: true,
        });
    let document = seal_reference_supermodel_rig_authoring_v2(document).unwrap();
    let applied = apply_reference_supermodel_rig_authoring_v2(
        &document,
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        STRUCTURE_SHA,
        ANATOMY_SHA,
        "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
        &base,
    )
    .unwrap();
    assert_eq!(applied.rig.nodes, base.nodes);
    assert_eq!(applied.rig.segments[0].reference_weights[2].len(), 1);
    assert_eq!(
        applied.rig.segments[0].reference_weights[2][0].bone_node_id,
        1
    );
    assert_eq!(applied.report.landmark_override_count, 0);
    assert_eq!(applied.report.component_binding_count, 1);
    assert_eq!(applied.report.region_weight_constraint_count, 1);

    let stale = apply_reference_supermodel_rig_authoring_v2(
        &document,
        SOURCE_SHA,
        CreatureSourceForwardV1::PositiveZ,
        "c_wolf",
        CHAIN_SHA,
        CONTRACT_SHA,
        STRUCTURE_SHA,
        &"f".repeat(64),
        "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
        &base,
    )
    .unwrap_err();
    assert_eq!(
        stale.code,
        "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-MISMATCH"
    );
}
