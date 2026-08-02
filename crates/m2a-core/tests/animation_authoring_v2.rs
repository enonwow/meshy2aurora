use m2a_core::{
    animation_authoring_v2::{
        AnimationContactBoneSetV1, AnimationContactIntervalV1, AnimationEditCommandBatchV1,
        AnimationEditCommandContextV1, AnimationEditCommandV1, AnimationKeyReductionPolicyV1,
        AnimationLoopSeamPolicyV1, AnimationPoseParityPolicyV1, AnimationQualityContextV1,
        AnimationQualityIssueKindV1, AnimationQualityPolicyV1, AnimationRootMotionPolicyV1,
        analyze_animation_motion_quality_v1, apply_animation_edit_command_batch_v1,
        apply_animation_root_motion_policy_v1, blend_animation_loop_seam_v1,
        evaluate_animation_pose_parity_v1, inspect_loop_seam_v1, lock_contact_bone_interval_v1,
        reduce_animation_keyframes_v1, restore_root_motion_v1, sample_animation_world_pose_v1,
        suggest_contact_intervals_v1, transform_root_motion_v1,
    },
    animation_studio::{
        AnimationKeyframeV1, AnimationStudioDocumentStatusV1, AnimationStudioDocumentV1,
        AnimationStudioRigNodeV1, AnimationStudioRigV1, AuthoredAnimationClipKindV1,
        AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1, AuthoredAnimationEventV1,
        AuthoredAnimationSourceKindV1, AuthoredAnimationSourceV1, AuthoredAnimationTrackPathV1,
        AuthoredAnimationTrackV1,
    },
    mdl::MdlAnimationInterpolationV1,
};

fn source_revision() -> String {
    "a".repeat(64)
}

fn rig() -> AnimationStudioRigV1 {
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: source_revision(),
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
                name: "Foot".into(),
                parent_id: Some(1),
                translation: [0.0, 0.0, 1.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    }
}

fn track(
    id: &str,
    target_node_id: u32,
    path: AuthoredAnimationTrackPathV1,
    values: &[[f32; 4]],
) -> AuthoredAnimationTrackV1 {
    AuthoredAnimationTrackV1 {
        id: id.into(),
        target_node_id,
        path,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: values
            .iter()
            .enumerate()
            .map(|(index, value)| AnimationKeyframeV1 {
                id: format!("{id}-{index}"),
                time_seconds: index as f32,
                value: if path == AuthoredAnimationTrackPathV1::Translation {
                    value[..3].to_vec()
                } else {
                    value.to_vec()
                },
            })
            .collect(),
    }
}

fn clip() -> AuthoredAnimationClipV1 {
    AuthoredAnimationClipV1 {
        id: "clip-1".into(),
        name: "test_motion".into(),
        kind: AuthoredAnimationClipKindV1::Motion,
        status: AuthoredAnimationClipStatusV1::Valid,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::BlankPose,
            source_revision: source_revision(),
            source_clip_name: None,
            source_clip_fingerprint: None,
            procedural_template: None,
            library_preset: None,
            retarget: None,
        },
        length_seconds: 1.0,
        transition_seconds: 0.1,
        animation_root: "Root".into(),
        tracks: vec![track(
            "root-translation",
            1,
            AuthoredAnimationTrackPathV1::Translation,
            &[[0.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
        )],
        events: vec![],
        revision: 1,
    }
}

fn document() -> AnimationStudioDocumentV1 {
    AnimationStudioDocumentV1 {
        schema_version: 3,
        source_revision: source_revision(),
        authoring_revision: 7,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip()],
    }
}

#[test]
fn world_pose_sampler_composes_parent_translation_deterministically() {
    let pose = sample_animation_world_pose_v1(&clip(), &rig(), 0.5).unwrap();
    let root = pose.bones.iter().find(|bone| bone.node_id == 1).unwrap();
    let foot = pose.bones.iter().find(|bone| bone.node_id == 2).unwrap();
    assert_eq!(root.translation, [0.5, 0.0, 0.0]);
    assert_eq!(foot.translation, [0.5, 0.0, 1.0]);
    assert_eq!(
        pose,
        sample_animation_world_pose_v1(&clip(), &rig(), 0.5).unwrap()
    );
}

#[test]
fn command_batch_is_revision_bound_atomic_and_deterministic() {
    let batch = AnimationEditCommandBatchV1 {
        schema_version: 1,
        command_id: "move-and-transition".into(),
        context: AnimationEditCommandContextV1 {
            source_revision: source_revision(),
            document_authoring_revision: 7,
        },
        commands: vec![
            AnimationEditCommandV1::SetTransition {
                clip_id: "clip-1".into(),
                transition_seconds: 0.2,
            },
            AnimationEditCommandV1::SetBoneKey {
                clip_id: "clip-1".into(),
                track_id: "foot-translation".into(),
                key_id: "foot-key-050".into(),
                target_node_id: 2,
                path: AuthoredAnimationTrackPathV1::Translation,
                time_seconds: 0.5,
                value: vec![0.0, 0.0, 1.0],
            },
        ],
    };
    let first = apply_animation_edit_command_batch_v1(&document(), &rig(), &batch).unwrap();
    let second = apply_animation_edit_command_batch_v1(&document(), &rig(), &batch).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.document.authoring_revision, 8);
    assert_eq!(first.document.authored_clips[0].transition_seconds, 0.2);

    let mut stale = batch.clone();
    stale.context.document_authoring_revision = 8;
    assert!(apply_animation_edit_command_batch_v1(&document(), &rig(), &stale).is_err());

    let mut invalid = batch;
    invalid
        .commands
        .push(AnimationEditCommandV1::SetTransition {
            clip_id: "missing".into(),
            transition_seconds: 0.1,
        });
    assert!(apply_animation_edit_command_batch_v1(&document(), &rig(), &invalid).is_err());
    assert_eq!(document().authored_clips[0].transition_seconds, 0.1);
}

#[test]
fn pose_parity_reports_exact_match_and_targeted_mismatch() {
    let policy = AnimationPoseParityPolicyV1::default();
    let exact = evaluate_animation_pose_parity_v1(&clip(), &clip(), &rig(), &policy).unwrap();
    assert!(exact.matches);

    let mut changed = clip();
    changed.tracks[0].keyframes[1].value[0] = 1.25;
    let mismatch = evaluate_animation_pose_parity_v1(&clip(), &changed, &rig(), &policy).unwrap();
    assert!(!mismatch.matches);
    assert_eq!(mismatch.max_translation_bone_id, Some(1));
    assert!(mismatch.max_translation_delta > 0.2);
}

#[test]
fn quality_analyzer_detects_static_loop_root_ground_slide_and_spike() {
    let mut problematic = clip();
    problematic.tracks.push(track(
        "foot-translation",
        2,
        AuthoredAnimationTrackPathV1::Translation,
        &[[0.0, 0.0, -1.1, 0.0], [2.0, 0.0, -1.1, 0.0]],
    ));
    let report = analyze_animation_motion_quality_v1(
        &problematic,
        &rig(),
        &AnimationQualityPolicyV1 {
            sample_rate_hz: 30,
            loop_expected: true,
            max_translation_speed_per_height: 1.0,
            ..AnimationQualityPolicyV1::default()
        },
        &AnimationQualityContextV1 {
            root_node_id: 1,
            contact_node_ids: vec![2],
            ground_axis: 2,
            ground_height: 0.0,
        },
    )
    .unwrap();
    let kinds = report
        .issues
        .iter()
        .map(|issue| issue.kind)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(kinds.contains(&AnimationQualityIssueKindV1::LoopDiscontinuity));
    assert!(kinds.contains(&AnimationQualityIssueKindV1::RootDrift));
    assert!(kinds.contains(&AnimationQualityIssueKindV1::GroundPenetration));
    assert!(kinds.contains(&AnimationQualityIssueKindV1::FootSliding));
    assert!(kinds.contains(&AnimationQualityIssueKindV1::MotionSpike));

    let mut static_clip = clip();
    static_clip.tracks[0].keyframes[1].value = vec![0.0, 0.0, 0.0];
    let static_report = analyze_animation_motion_quality_v1(
        &static_clip,
        &rig(),
        &AnimationQualityPolicyV1::default(),
        &AnimationQualityContextV1::default(),
    )
    .unwrap();
    assert!(
        static_report
            .issues
            .iter()
            .any(|issue| issue.kind == AnimationQualityIssueKindV1::StaticPlayback)
    );
}

#[test]
fn root_motion_policy_can_preserve_make_in_place_and_scale() {
    let original = clip();
    assert_eq!(
        apply_animation_root_motion_policy_v1(
            &original,
            &rig(),
            AnimationRootMotionPolicyV1::Preserve,
        )
        .unwrap(),
        original
    );
    let in_place = apply_animation_root_motion_policy_v1(
        &original,
        &rig(),
        AnimationRootMotionPolicyV1::InPlace,
    )
    .unwrap();
    assert_eq!(in_place.tracks[0].keyframes[0].value, vec![0.0, 0.0, 0.0]);
    assert_eq!(in_place.tracks[0].keyframes[1].value, vec![0.0, 0.0, 0.0]);

    let scaled = apply_animation_root_motion_policy_v1(
        &original,
        &rig(),
        AnimationRootMotionPolicyV1::Scale { factor: 2.0 },
    )
    .unwrap();
    assert_eq!(scaled.tracks[0].keyframes[1].value, vec![2.0, 0.0, 0.0]);
}

#[test]
fn loop_blend_reduces_the_seam_and_preserves_events() {
    let mut source = clip();
    source.tracks[0].keyframes.insert(
        1,
        AnimationKeyframeV1 {
            id: "root-mid".into(),
            time_seconds: 0.5,
            value: vec![1.0, 0.0, 0.0],
        },
    );
    source.events.push(AuthoredAnimationEventV1 {
        id: "hit".into(),
        time_seconds: 0.5,
        name: "hit".into(),
    });
    let before = inspect_loop_seam_v1(&source, &rig()).unwrap();
    let result = blend_animation_loop_seam_v1(
        &source,
        &rig(),
        &AnimationLoopSeamPolicyV1 {
            schema_version: 1,
            blend_window_seconds: 0.25,
        },
    )
    .unwrap();
    assert!(before.max_position_jump > 0.9);
    assert!(result.after.max_position_jump < 1.0e-5);
    assert_eq!(result.clip.events, source.events);
    assert_eq!(result.clip.revision, source.revision + 1);
}

#[test]
fn reversible_root_motion_restores_the_exact_source_track() {
    let source = clip();
    let transformed =
        transform_root_motion_v1(&source, &rig(), AnimationRootMotionPolicyV1::InPlace).unwrap();
    assert_eq!(transformed.displacement_before, [1.0, 0.0, 0.0]);
    assert_eq!(transformed.displacement_after, [0.0, 0.0, 0.0]);
    let restored =
        restore_root_motion_v1(&transformed.clip, &rig(), &transformed.reversible_layer).unwrap();
    assert_eq!(restored.tracks[0], source.tracks[0]);
}

#[test]
fn contact_lock_bakes_a_stationary_world_space_foot() {
    let source = clip();
    let suggested = suggest_contact_intervals_v1(
        &source,
        &rig(),
        &AnimationContactBoneSetV1 {
            schema_version: 1,
            node_ids: vec![2],
            ground_axis: 2,
            ground_height: 0.0,
            contact_height: 2.0,
            max_contact_speed: 2.0,
            sample_rate_hz: 30,
        },
    )
    .unwrap();
    assert_eq!(suggested.len(), 1);
    let locked = lock_contact_bone_interval_v1(
        &source,
        &rig(),
        &AnimationContactIntervalV1 {
            node_id: 2,
            start_seconds: 0.0,
            end_seconds: 1.0,
            anchor_world_translation: [0.0, 0.0, 1.0],
            max_slide_before: 1.0,
        },
        30,
        2,
    )
    .unwrap();
    assert!(locked.max_slide_before > 0.9);
    assert!(locked.max_slide_after < 1.0e-5);
    assert_eq!(source.tracks.len(), 1, "source clip remains immutable");
}

#[test]
fn key_reduction_removes_linear_middle_keys_with_bounded_error() {
    let mut source = clip();
    source.tracks[0].keyframes = vec![
        AnimationKeyframeV1 {
            id: "a".into(),
            time_seconds: 0.0,
            value: vec![0.0, 0.0, 0.0],
        },
        AnimationKeyframeV1 {
            id: "b".into(),
            time_seconds: 0.5,
            value: vec![0.5, 0.0, 0.0],
        },
        AnimationKeyframeV1 {
            id: "c".into(),
            time_seconds: 1.0,
            value: vec![1.0, 0.0, 0.0],
        },
    ];
    let report =
        reduce_animation_keyframes_v1(&source, &AnimationKeyReductionPolicyV1::default()).unwrap();
    assert_eq!(report.keys_before, 3);
    assert_eq!(report.keys_after, 2);
    assert_eq!(report.removed_keys, 1);
    assert!(report.max_position_error <= 1.0e-4);
    assert_eq!(source.tracks[0].keyframes.len(), 3);
}
