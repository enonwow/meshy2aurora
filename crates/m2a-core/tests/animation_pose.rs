use m2a_core::{
    animation_pose::{
        AnimationMirrorAxisV1, AnimationPosePasteModeV1, HumanoidMirrorPairV1,
        apply_animation_pose_v1, build_humanoid_mirror_map_v1, capture_animation_pose_v1,
        mirror_animation_pose_v1, reset_animation_pose_to_rest_v1,
    },
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
                translation: [0.0; 3],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: 2,
                name: "LeftHand".into(),
                parent_id: Some(1),
                translation: [-1.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: 3,
                name: "RightHand".into(),
                parent_id: Some(1),
                translation: [1.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    }
}

fn clip() -> AuthoredAnimationClipV1 {
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
            id: "left-translation".into(),
            target_node_id: 2,
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: vec![
                AnimationKeyframeV1 {
                    id: "l0".into(),
                    time_seconds: 0.0,
                    value: vec![-1.0, 0.0, 0.0],
                },
                AnimationKeyframeV1 {
                    id: "l1".into(),
                    time_seconds: 1.0,
                    value: vec![-2.0, 0.5, 0.0],
                },
            ],
        }],
        events: vec![],
        revision: 1,
    }
}

#[test]
fn copy_paste_selected_bones_preserves_unselected_tracks() {
    let source = clip();
    let pose = capture_animation_pose_v1(&source, &rig(), 1.0, &[2]).unwrap();
    let result = apply_animation_pose_v1(
        &source,
        &rig(),
        &pose,
        0.5,
        AnimationPosePasteModeV1::SelectedBones,
        &[2],
    )
    .unwrap();
    assert_eq!(result.changed_node_ids, vec![2]);
    assert_eq!(result.clip.tracks.len(), 2);
    assert_eq!(result.clip.revision, 2);
    assert_eq!(result.fingerprint_sha256.len(), 64);
}

#[test]
fn mirror_twice_returns_to_the_canonical_pose() {
    let source = clip();
    let pose = capture_animation_pose_v1(&source, &rig(), 1.0, &[]).unwrap();
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
    let first = mirror_animation_pose_v1(&pose, &rig(), &mapping).unwrap();
    let second = mirror_animation_pose_v1(&first, &rig(), &mapping).unwrap();
    assert_eq!(pose.bones, second.bones);
    assert_ne!(pose.fingerprint_sha256, first.fingerprint_sha256);
}

#[test]
fn reset_writes_exact_rest_transform_only_for_selection() {
    let result = reset_animation_pose_to_rest_v1(&clip(), &rig(), 1.0, &[2]).unwrap();
    let translation = result
        .clip
        .tracks
        .iter()
        .find(|track| {
            track.target_node_id == 2 && track.path == AuthoredAnimationTrackPathV1::Translation
        })
        .unwrap();
    assert_eq!(
        translation.keyframes.last().unwrap().value,
        vec![-1.0, 0.0, 0.0]
    );
    assert_eq!(result.changed_node_ids, vec![2]);
}

#[test]
fn stale_pose_and_ambiguous_mirror_map_fail_closed() {
    let mut pose = capture_animation_pose_v1(&clip(), &rig(), 0.0, &[]).unwrap();
    pose.source_revision = "b".repeat(64);
    assert!(
        apply_animation_pose_v1(
            &clip(),
            &rig(),
            &pose,
            0.0,
            AnimationPosePasteModeV1::WholeBody,
            &[]
        )
        .is_err()
    );
    assert!(
        build_humanoid_mirror_map_v1(
            &rig(),
            AnimationMirrorAxisV1::X,
            vec![
                HumanoidMirrorPairV1 {
                    left_node_id: 2,
                    right_node_id: 3
                },
                HumanoidMirrorPairV1 {
                    left_node_id: 2,
                    right_node_id: 1
                },
            ],
            vec![],
        )
        .is_err()
    );
}
