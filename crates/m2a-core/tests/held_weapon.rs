use m2a_core::{
    animation_studio::{
        AnimationStudioRigNodeV1, AnimationStudioRigV1, AuthoredAnimationClipKindV1,
        AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1, AuthoredAnimationSourceKindV1,
        AuthoredAnimationSourceV1,
    },
    held_weapon::{
        HeldWeaponHandV1, HeldWeaponLocalTransformV1, HeldWeaponPivotPolicyV1, HeldWeaponSourceV1,
        bake_held_weapon_attachment_v1, compose_held_weapon_attachment_v1,
        measure_held_weapon_grip_error_v1, resolve_hand_attachment_target_v1,
    },
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
        AuroraSegmentDeformationV1,
    },
};

fn rig() -> AnimationStudioRigV1 {
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        animation_root: "Root".into(),
        nodes: vec![
            AnimationStudioRigNodeV1 {
                node_id: 1,
                name: "Root".into(),
                parent_id: None,
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: 2,
                name: "mixamorig:RightHand".into(),
                parent_id: Some(1),
                translation: [1.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: 3,
                name: "LeftHand".into(),
                parent_id: Some(1),
                translation: [-1.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    }
}

fn clip() -> AuthoredAnimationClipV1 {
    AuthoredAnimationClipV1 {
        id: "attack".into(),
        name: "attack".into(),
        kind: AuthoredAnimationClipKindV1::StaticPose,
        status: AuthoredAnimationClipStatusV1::Valid,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::BlankPose,
            source_revision: "a".repeat(64),
            source_clip_name: None,
            source_clip_fingerprint: None,
            procedural_template: None,
            library_preset: None,
            retarget: None,
        },
        length_seconds: 1.0,
        transition_seconds: 0.1,
        animation_root: "Root".into(),
        tracks: vec![],
        events: vec![],
        revision: 1,
    }
}

fn source() -> HeldWeaponSourceV1 {
    HeldWeaponSourceV1 {
        schema_version: 1,
        filename: "sword.glb".into(),
        byte_size: 100,
        sha256: "b".repeat(64),
        provenance: "LOCAL_FILE".into(),
        triangle_count: 1,
        material_count: 1,
        texture_count: 1,
        bounds_min: Some([-0.1, 0.0, -1.0]),
        bounds_max: Some([0.1, 0.2, 1.0]),
    }
}

fn model(source_sha256: &str, root_id: u32, root_name: &str) -> AuroraModelIrV1 {
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "TEST".into(),
        source_sha256: source_sha256.into(),
        basis_status: "TEST".into(),
        engine_facing_proof: "TEST".into(),
        uv_runtime_proof: "TEST".into(),
        nodes: vec![AuroraModelNodeV1 {
            id: root_id,
            name: root_name.into(),
            parent_id: None,
            bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: Some(0),
            source_material_name: Some(format!("{root_name}_material")),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 0,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: root_id,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            face_surface_ids: vec![],
            weights: vec![],
        }],
    }
}

#[test]
fn hand_alias_resolution_is_versioned_by_side_and_can_be_explicit() {
    assert_eq!(
        resolve_hand_attachment_target_v1(&rig(), HeldWeaponHandV1::Right, None)
            .unwrap()
            .node_id,
        2
    );
    assert_eq!(
        resolve_hand_attachment_target_v1(&rig(), HeldWeaponHandV1::Left, Some(3))
            .unwrap()
            .name,
        "LeftHand"
    );
}

#[test]
fn attachment_fingerprint_and_grip_metrics_are_deterministic() {
    let first = compose_held_weapon_attachment_v1(
        source(),
        &rig(),
        HeldWeaponHandV1::Right,
        2,
        HeldWeaponLocalTransformV1::default(),
        HeldWeaponPivotPolicyV1::SourceOrigin,
        Some(3),
        Some([-2.0, 0.0, 0.0]),
        1,
    )
    .unwrap();
    let second = compose_held_weapon_attachment_v1(
        source(),
        &rig(),
        HeldWeaponHandV1::Right,
        2,
        HeldWeaponLocalTransformV1::default(),
        HeldWeaponPivotPolicyV1::SourceOrigin,
        Some(3),
        Some([-2.0, 0.0, 0.0]),
        1,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.canonical_fingerprint_sha256.len(), 64);
    let report = measure_held_weapon_grip_error_v1(&clip(), &rig(), &first, 30).unwrap();
    assert_eq!(report.primary_hand_max_error, 0.0);
    assert!(report.secondary_hand_max_error.unwrap() < 1.0e-6);
}

#[test]
fn rigid_weapon_ir_is_baked_below_the_exact_hand_without_triangle_loss() {
    let attachment = compose_held_weapon_attachment_v1(
        source(),
        &rig(),
        HeldWeaponHandV1::Right,
        2,
        HeldWeaponLocalTransformV1 {
            translation: [0.1, 0.2, 0.3],
            ..HeldWeaponLocalTransformV1::default()
        },
        HeldWeaponPivotPolicyV1::BoundsCenter,
        None,
        None,
        2,
    )
    .unwrap();
    let mut target = model(&"a".repeat(64), 1, "Root");
    target.nodes.push(AuroraModelNodeV1 {
        id: 2,
        name: "mixamorig:RightHand".into(),
        parent_id: Some(1),
        bind_local_matrix: target.nodes[0].bind_local_matrix,
    });
    let weapon = model(&"b".repeat(64), 10, "SwordRoot");

    let report = bake_held_weapon_attachment_v1(&target, &weapon, &attachment).unwrap();

    assert_eq!(report.target_triangle_count_before, 1);
    assert_eq!(report.weapon_triangle_count, 1);
    assert_eq!(report.target_triangle_count_after, 2);
    assert_eq!(report.segment_count_after, 2);
    assert!(
        report
            .model
            .nodes
            .iter()
            .any(|node| { node.id == report.attachment_node_id && node.parent_id == Some(2) })
    );
    assert_eq!(report.fingerprint_sha256.len(), 64);
}
