use m2a_core::{
    animation_authoring_v2::sample_animation_world_pose_v1,
    animation_motion_tools::{
        AnimationVariantOperationV1, AnimationVariantRecipeV1, CombatQualityContextV1,
        CombatQualityPolicyV1, MotionTrailRequestV1, analyze_combat_animation_quality_v1,
        authored_clip_fingerprint_v1, build_motion_visualization_v1, create_animation_variant_v1,
        create_combat_attack_phase_timeline_v1, retime_combat_attack_phases_v1,
    },
    animation_pose::{AnimationMirrorAxisV1, HumanoidMirrorPairV1, build_humanoid_mirror_map_v1},
    animation_studio::{
        AnimationKeyframeV1, AnimationStudioRigNodeV1, AnimationStudioRigV1,
        AuthoredAnimationClipKindV1, AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1,
        AuthoredAnimationSourceKindV1, AuthoredAnimationSourceV1, AuthoredAnimationTrackPathV1,
        AuthoredAnimationTrackV1,
    },
    mdl::MdlAnimationInterpolationV1,
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
                name: "LeftHand".into(),
                parent_id: Some(1),
                translation: [-0.5, 0.0, 1.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: 3,
                name: "RightHand".into(),
                parent_id: Some(1),
                translation: [0.5, 0.0, 1.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    }
}

fn clip() -> AuthoredAnimationClipV1 {
    let positions = [
        [0.5, 0.0, 1.0],
        [0.35, -0.2, 1.0],
        [0.6, 0.2, 1.0],
        [1.2, 0.5, 1.0],
        [0.5, 0.0, 1.0],
    ];
    AuthoredAnimationClipV1 {
        id: "attack".into(),
        name: "attack".into(),
        kind: AuthoredAnimationClipKindV1::Motion,
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
        tracks: vec![AuthoredAnimationTrackV1 {
            id: "right".into(),
            target_node_id: 3,
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: positions
                .into_iter()
                .enumerate()
                .map(|(index, value)| AnimationKeyframeV1 {
                    id: format!("k{index}"),
                    time_seconds: [0.0, 0.2, 0.4, 0.6, 1.0][index],
                    value: value.to_vec(),
                })
                .collect(),
        }],
        events: vec![],
        revision: 1,
    }
}

#[test]
fn phases_retime_keys_without_changing_source() {
    let source = clip();
    let from = create_combat_attack_phase_timeline_v1(&source, [0.0, 0.2, 0.4, 0.6, 1.0]).unwrap();
    let to = create_combat_attack_phase_timeline_v1(&source, [0.0, 0.1, 0.3, 0.5, 1.0]).unwrap();
    let output = retime_combat_attack_phases_v1(&source, &from, &to).unwrap();
    assert_eq!(source.tracks[0].keyframes[1].time_seconds, 0.2);
    assert_eq!(output.tracks[0].keyframes[1].time_seconds, 0.1);
}

#[test]
fn combat_report_measures_arc_peak_and_recovery_without_artistic_blockers() {
    let clip = clip();
    let timeline =
        create_combat_attack_phase_timeline_v1(&clip, [0.0, 0.2, 0.4, 0.6, 1.0]).unwrap();
    let report = analyze_combat_animation_quality_v1(
        &clip,
        &rig(),
        &timeline,
        &CombatQualityPolicyV1::default(),
        &CombatQualityContextV1 {
            schema_version: 1,
            attack_node_id: 3,
            root_node_id: 1,
            grip_max_error: Some(0.0),
            contact_node_ids: vec![],
            contribution_node_ids: vec![1, 2, 3],
        },
    )
    .unwrap();
    assert!(report.attack_arc_length > 0.5);
    assert!(report.peak_speed > 0.0);
    assert!(
        report
            .issues
            .iter()
            .all(|issue| format!("{:?}", issue.severity) != "Blocking")
    );
}

#[test]
fn combat_report_detects_declared_impact_contact_motion_and_reports_body_share() {
    let mut exact_rig = rig();
    exact_rig.nodes.push(AnimationStudioRigNodeV1 {
        node_id: 4,
        name: "LeftFoot".into(),
        parent_id: Some(1),
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
    });
    let mut attack = clip();
    attack.tracks.push(AuthoredAnimationTrackV1 {
        id: "moving-contact".into(),
        target_node_id: 4,
        path: AuthoredAnimationTrackPathV1::Translation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: vec![
            AnimationKeyframeV1 {
                id: "contact-strike".into(),
                time_seconds: 0.4,
                value: vec![0.0, 0.0, 0.0],
            },
            AnimationKeyframeV1 {
                id: "contact-impact".into(),
                time_seconds: 0.6,
                value: vec![0.8, 0.0, 0.0],
            },
        ],
    });
    attack.tracks.push(AuthoredAnimationTrackV1 {
        id: "moving-root".into(),
        target_node_id: 1,
        path: AuthoredAnimationTrackPathV1::Translation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: vec![
            AnimationKeyframeV1 {
                id: "root-ready".into(),
                time_seconds: 0.0,
                value: vec![0.0, 0.0, 0.0],
            },
            AnimationKeyframeV1 {
                id: "root-impact".into(),
                time_seconds: 0.7,
                value: vec![0.4, 0.0, 0.0],
            },
        ],
    });
    let timeline =
        create_combat_attack_phase_timeline_v1(&attack, [0.0, 0.2, 0.4, 0.7, 1.0]).unwrap();
    let policy = CombatQualityPolicyV1 {
        minimum_anticipation_per_height: 0.5,
        maximum_impact_peak_offset_seconds: 0.01,
        maximum_direction_reversals: 0,
        maximum_recovery_error_per_height: 0.0,
        maximum_root_displacement_per_height: 0.0,
        ..CombatQualityPolicyV1::default()
    };
    let report = analyze_combat_animation_quality_v1(
        &attack,
        &exact_rig,
        &timeline,
        &policy,
        &CombatQualityContextV1 {
            schema_version: 1,
            attack_node_id: 3,
            root_node_id: 1,
            grip_max_error: Some(1.0),
            contact_node_ids: vec![4],
            contribution_node_ids: vec![1, 2, 3],
        },
    )
    .unwrap();
    assert!(report.impact_contact_displacement > 0.5);
    assert_eq!(report.body_contributions.len(), 3);
    let kinds = report
        .issues
        .iter()
        .map(|issue| format!("{:?}", issue.kind))
        .collect::<std::collections::BTreeSet<_>>();
    for expected in [
        "WeakAnticipation",
        "ImpactTiming",
        "AttackArcReversal",
        "RecoveryMismatch",
        "RootInstability",
        "ContactInstability",
        "GripError",
    ] {
        assert!(kinds.contains(expected), "missing combat issue {expected}");
    }
    assert!(
        report
            .issues
            .iter()
            .all(|issue| format!("{:?}", issue.severity) != "Blocking")
    );
}

#[test]
fn trails_and_onion_poses_are_revision_bound_and_deterministic() {
    let clip = clip();
    let before = clip.clone();
    let request = MotionTrailRequestV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        clip_id: "attack".into(),
        clip_revision: 1,
        node_ids: vec![3],
        start_seconds: 0.0,
        end_seconds: 1.0,
        sample_count: 9,
        onion_before: 2,
        onion_after: 2,
        current_time_seconds: 0.5,
    };
    let first = build_motion_visualization_v1(&clip, &rig(), &request).unwrap();
    let second = build_motion_visualization_v1(&clip, &rig(), &request).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.trails[0].points.len(), 9);
    assert_eq!(first.onion_poses.len(), 4);
    assert_eq!(clip, before);
    for point in &first.trails[0].points {
        let pose = sample_animation_world_pose_v1(&clip, &rig(), point.time_seconds).unwrap();
        let exact = pose.bones.iter().find(|bone| bone.node_id == 3).unwrap();
        assert_eq!(point.translation, exact.translation);
    }
}

#[test]
fn variants_keep_source_immutable_and_record_speed_mirror_amplitude() {
    let source = clip();
    let mapping = build_humanoid_mirror_map_v1(
        &rig(),
        AnimationMirrorAxisV1::X,
        vec![HumanoidMirrorPairV1 {
            left_node_id: 2,
            right_node_id: 3,
        }],
        vec![1],
    )
    .unwrap();
    let before = source.clone();
    let recipe = AnimationVariantRecipeV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        source_clip_id: source.id.clone(),
        source_clip_revision: source.revision,
        source_clip_fingerprint_sha256: authored_clip_fingerprint_v1(&source),
        output_clip_id: "attack_fast_left".into(),
        output_name: "atk_fast_left".into(),
        inherited_tags: vec!["attack".into()],
        added_tags: vec!["left-hand".into()],
        operations: vec![
            AnimationVariantOperationV1::Speed { factor: 2.0 },
            AnimationVariantOperationV1::Mirror { mapping },
            AnimationVariantOperationV1::Amplitude {
                factor: 1.2,
                node_ids: vec![2],
            },
        ],
    };
    let result = create_animation_variant_v1(&source, &rig(), &recipe).unwrap();
    assert_eq!(source, before);
    assert_eq!(result.clip.length_seconds, 0.5);
    assert_eq!(result.effective_tags, vec!["attack", "left-hand"]);
    assert!(
        result
            .clip
            .source
            .procedural_template
            .as_deref()
            .unwrap()
            .starts_with("ANIMATION_VARIANT_RECIPE_V1:")
    );
}

#[test]
fn three_distinct_variants_keep_one_source_and_provenance_immutable() {
    let source = clip();
    let before = source.clone();
    let source_fingerprint = authored_clip_fingerprint_v1(&source);
    let mapping = build_humanoid_mirror_map_v1(
        &rig(),
        AnimationMirrorAxisV1::X,
        vec![HumanoidMirrorPairV1 {
            left_node_id: 2,
            right_node_id: 3,
        }],
        vec![1],
    )
    .unwrap();
    let operations = [
        vec![AnimationVariantOperationV1::Speed { factor: 1.4 }],
        vec![AnimationVariantOperationV1::Mirror { mapping }],
        vec![AnimationVariantOperationV1::Amplitude {
            factor: 1.5,
            node_ids: vec![3],
        }],
    ];
    let results = operations
        .into_iter()
        .enumerate()
        .map(|(index, operations)| {
            create_animation_variant_v1(
                &source,
                &rig(),
                &AnimationVariantRecipeV1 {
                    schema_version: 1,
                    source_revision: "a".repeat(64),
                    source_clip_id: source.id.clone(),
                    source_clip_revision: source.revision,
                    source_clip_fingerprint_sha256: source_fingerprint.clone(),
                    output_clip_id: format!("attack_variant_{index}"),
                    output_name: format!("attack variant {index}"),
                    inherited_tags: vec!["attack".into(), "combat".into()],
                    added_tags: vec![format!("variant-{index}")],
                    operations,
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(source, before);
    assert_eq!(authored_clip_fingerprint_v1(&source), source_fingerprint);
    assert_eq!(
        results
            .iter()
            .map(|result| result.fingerprint_sha256.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    for (index, result) in results.iter().enumerate() {
        assert!(result.effective_tags.contains(&"attack".to_owned()));
        assert!(result.effective_tags.contains(&"combat".to_owned()));
        assert!(result.effective_tags.contains(&format!("variant-{index}")));
        assert_eq!(
            result.clip.source.source_clip_fingerprint.as_deref(),
            Some(source_fingerprint.as_str())
        );
    }
}

#[test]
fn stale_visualization_and_invalid_phase_order_fail_closed() {
    let clip = clip();
    assert!(create_combat_attack_phase_timeline_v1(&clip, [0.0, 0.4, 0.2, 0.6, 1.0]).is_err());
    let request = MotionTrailRequestV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        clip_id: "attack".into(),
        clip_revision: 2,
        node_ids: vec![3],
        start_seconds: 0.0,
        end_seconds: 1.0,
        sample_count: 8,
        onion_before: 1,
        onion_after: 1,
        current_time_seconds: 0.5,
    };
    assert!(build_motion_visualization_v1(&clip, &rig(), &request).is_err());
}
