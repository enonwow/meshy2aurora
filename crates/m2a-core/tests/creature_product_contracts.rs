use m2a_core::creature_product::{
    CreatureBlueprintAuthoringV1, CreatureEquipmentItemV2, CreatureEquipmentLoadoutV2,
    CreatureEquipmentPlacementV2, CreatureItemGripFamilyV1, CreatureMaterialProfileV2,
    CreatureModelBoundsV1, CreatureMotionClipV1, CreatureMotionPackV1,
    CreatureMotionSkeletonProfileV1, CreaturePerformanceInputV1, CreaturePerformancePresetV1,
    CreatureRuntimeEnvelopePolicyV1, creature_contract_digest_v1, creature_item_grip_profile_v1,
    derive_creature_runtime_envelope_v1, evaluate_creature_performance_v1,
    validate_creature_blueprint_authoring_v1, validate_creature_equipment_loadout_v2,
    validate_creature_material_profile_v2, validate_creature_motion_pack_source_v1,
    validate_creature_motion_pack_v1,
};
use m2a_core::model_limits::AURORA_MODEL_TRIANGLE_BUDGET_V1;

#[test]
fn v10_loadout_is_a_complete_embedded_item_contract_and_participates_in_identity() {
    let loadout = CreatureEquipmentLoadoutV2::v10_bastard_sword_right_hand();
    let report = validate_creature_equipment_loadout_v2(&loadout).expect("valid V10 loadout");

    assert_eq!(report.embedded_item_count, 1);
    assert_eq!(report.native_slots, vec![16]);
    assert_eq!(report.required_proficiency_feats, vec![44]);
    assert!(report.no_equipped_res_shortcuts);

    let mut changed = loadout.clone();
    changed.items[0].model_parts[0] += 1;
    assert_ne!(
        creature_contract_digest_v1(&loadout).unwrap(),
        creature_contract_digest_v1(&changed).unwrap()
    );
}

#[test]
fn loadout_rejects_slot_overlap_and_two_handed_offhand_collisions() {
    let mut loadout = CreatureEquipmentLoadoutV2::v10_bastard_sword_right_hand();
    let mut shield = CreatureEquipmentItemV2::owned(
        "m2a_shield",
        "Meshy shield",
        14,
        [1, 1, 1],
        CreatureEquipmentPlacementV2::LeftHand,
        CreatureItemGripFamilyV1::Shield,
    );
    shield.required_proficiency_feats = vec![32];
    loadout.items.push(shield.clone());
    validate_creature_equipment_loadout_v2(&loadout).expect("one item per native hand");

    let duplicate_slot = CreatureEquipmentLoadoutV2 {
        schema_version: 2,
        items: vec![
            shield.clone(),
            CreatureEquipmentItemV2 {
                resref: "m2a_shield2".into(),
                ..shield
            },
        ],
    };
    let error = validate_creature_equipment_loadout_v2(&duplicate_slot).unwrap_err();
    assert_eq!(error.code, "CREATURE-EQUIPMENT-SLOT-COLLISION");

    let mut two_handed = CreatureEquipmentLoadoutV2::v10_bastard_sword_right_hand();
    two_handed.items[0].placement = CreatureEquipmentPlacementV2::BothHands;
    two_handed.items.push(CreatureEquipmentItemV2::owned(
        "m2a_offhand",
        "Offhand item",
        1,
        [1, 1, 1],
        CreatureEquipmentPlacementV2::LeftHand,
        CreatureItemGripFamilyV1::Sword,
    ));
    assert_eq!(
        validate_creature_equipment_loadout_v2(&two_handed)
            .unwrap_err()
            .code,
        "CREATURE-EQUIPMENT-TWO-HANDED-COLLISION"
    );
}

#[test]
fn every_item_family_has_a_distinct_proper_rigid_grip_basis() {
    let families = [
        CreatureItemGripFamilyV1::Sword,
        CreatureItemGripFamilyV1::AxeMace,
        CreatureItemGripFamilyV1::SpearPolearm,
        CreatureItemGripFamilyV1::BowCrossbow,
        CreatureItemGripFamilyV1::Shield,
    ];
    let profiles = families.map(creature_item_grip_profile_v1);
    for profile in &profiles {
        let m = profile.primary_item_basis_rotation;
        let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        assert!((determinant - 1.0).abs() <= 1.0e-6);
    }
    for left in 0..profiles.len() {
        for right in left + 1..profiles.len() {
            assert_ne!(
                profiles[left].primary_item_basis_rotation,
                profiles[right].primary_item_basis_rotation
            );
        }
    }
}

#[test]
fn runtime_envelope_can_be_derived_from_bounds_without_touching_model_identity() {
    let bounds = CreatureModelBoundsV1 {
        min: [-0.45, -0.30, 0.0],
        max: [0.45, 0.30, 2.4],
    };
    let envelope = derive_creature_runtime_envelope_v1(
        &CreatureRuntimeEnvelopePolicyV1::BoundsDerived,
        bounds,
    )
    .expect("finite bounds");

    assert_eq!(envelope.height, 2.4);
    assert!(envelope.hit_distance >= 0.45);
    assert!(envelope.personal_space >= envelope.hit_distance);
    assert!(envelope.preferred_attack_distance > envelope.hit_distance);

    let blueprint = CreatureBlueprintAuthoringV1::active_monster_default();
    validate_creature_blueprint_authoring_v1(&blueprint).expect("valid authored UTC");
    let mut changed = blueprint.clone();
    changed.max_hit_points += 10;
    assert_ne!(
        creature_contract_digest_v1(&blueprint).unwrap(),
        creature_contract_digest_v1(&changed).unwrap()
    );
}

#[test]
fn motion_pack_requires_semantic_skeleton_and_weapon_aware_attack_roles() {
    let humanoid = CreatureMotionPackV1::humanoid_weapon_pack(
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        vec![
            CreatureMotionClipV1::source("idle", "cpause1", None),
            CreatureMotionClipV1::source(
                "sword_slash",
                "ca1slashl",
                Some(CreatureItemGripFamilyV1::Sword),
            ),
            CreatureMotionClipV1::source(
                "polearm_thrust",
                "ca1stab",
                Some(CreatureItemGripFamilyV1::SpearPolearm),
            ),
            CreatureMotionClipV1::source(
                "bow_attack",
                "ca1bow",
                Some(CreatureItemGripFamilyV1::BowCrossbow),
            ),
        ],
    );
    let report = validate_creature_motion_pack_v1(&humanoid).expect("weapon-aware pack");
    assert_eq!(report.weapon_attack_families.len(), 3);
    assert_eq!(
        report.skeleton_profile,
        CreatureMotionSkeletonProfileV1::Humanoid
    );

    let mut quadruped = humanoid.clone();
    quadruped.skeleton_profile = CreatureMotionSkeletonProfileV1::Quadruped;
    quadruped.semantic_joints = vec!["Root".into(), "Spine".into(), "Head".into()];
    let error = validate_creature_motion_pack_v1(&quadruped).unwrap_err();
    assert_eq!(error.code, "CREATURE-MOTION-QUADRUPED-CONTACTS-MISSING");
}

#[test]
fn quadruped_motion_intake_is_bound_to_real_four_contact_joints_and_source_clips() {
    let pack = CreatureMotionPackV1 {
        schema_version: 1,
        source_identity: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .to_owned(),
        skeleton_profile: CreatureMotionSkeletonProfileV1::Quadruped,
        semantic_joints: vec![
            "Root".into(),
            "Spine".into(),
            "Head".into(),
            "FrontLeftFoot".into(),
            "FrontRightFoot".into(),
            "HindLeftFoot".into(),
            "HindRightFoot".into(),
        ],
        clips: vec![
            CreatureMotionClipV1::source("idle4", "cpause1", None),
            CreatureMotionClipV1::source("run4", "crun", None),
        ],
    };
    let report = validate_creature_motion_pack_source_v1(
        &pack,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        &pack.semantic_joints,
        &["idle4".into(), "run4".into()],
        None,
    )
    .expect("source-bound quadruped intake");
    assert!(report.four_point_contact_ready);
    assert_eq!(report.resolved_clip_count, 2);

    let error = validate_creature_motion_pack_source_v1(
        &pack,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        &pack.semantic_joints[..6],
        &["idle4".into(), "run4".into()],
        None,
    )
    .unwrap_err();
    assert_eq!(error.code, "CREATURE-MOTION-SOURCE-JOINT-MISSING");
}

#[test]
fn materials_and_performance_profiles_preserve_fail_closed_gates_and_shared_budget() {
    let classic = CreatureMaterialProfileV2::classic_diffuse();
    validate_creature_material_profile_v2(&classic).expect("classic compatibility");

    let mut invalid_normal = CreatureMaterialProfileV2::nwn_ee_mtr();
    invalid_normal.normal_maps = true;
    invalid_normal.tangent_space_ready = false;
    assert_eq!(
        validate_creature_material_profile_v2(&invalid_normal)
            .unwrap_err()
            .code,
        "CREATURE-MATERIAL-TANGENTS-REQUIRED"
    );

    let report = evaluate_creature_performance_v1(&CreaturePerformanceInputV1 {
        preset: CreaturePerformancePresetV1::Compact,
        triangle_count: AURORA_MODEL_TRIANGLE_BUDGET_V1 as u32,
        segment_count: 14,
        material_count: 6,
        draw_call_count: 14,
        joint_count: 64,
        controller_count: 800,
        model_bytes: 24_000_000,
        texture_bytes: 12_000_000,
        hak_bytes: 40_000_000,
        instance_count: 8,
    })
    .expect("exact shared hard maximum remains accepted");
    assert_eq!(report.hard_triangle_limit, 300_000);
    assert_eq!(report.triangle_count, 300_000);
    assert!(!report.suggested_target_met);

    let above = CreaturePerformanceInputV1 {
        triangle_count: AURORA_MODEL_TRIANGLE_BUDGET_V1 as u32 + 1,
        ..Default::default()
    };
    assert_eq!(
        evaluate_creature_performance_v1(&above).unwrap_err().code,
        "CREATURE-PERFORMANCE-TRIANGLE-BUDGET"
    );
}
