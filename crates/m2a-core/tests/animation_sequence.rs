use m2a_core::{
    animation_sequence::{
        AnimationSequenceDocumentV1, AnimationSequencePhaseMarkerV1,
        AnimationSequencePreviewRequestV1, AnimationSequenceSegmentRecipeV1,
        AnimationSequenceTransitionKindV1, AnimationSequenceTransitionV1,
        bake_animation_sequence_document_v1, build_animation_sequence_preview_v1,
    },
    animation_studio::{
        AnimationKeyframeV1, AuthoredAnimationClipKindV1, AuthoredAnimationClipStatusV1,
        AuthoredAnimationClipV1, AuthoredAnimationSourceKindV1, AuthoredAnimationSourceV1,
        AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1,
    },
    mdl::MdlAnimationInterpolationV1,
};

fn clip(id: &str, start: f32, end: f32) -> AuthoredAnimationClipV1 {
    AuthoredAnimationClipV1 {
        id: id.into(),
        name: id.into(),
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
        animation_root: "root".into(),
        tracks: vec![AuthoredAnimationTrackV1 {
            id: format!("{id}-translation"),
            target_node_id: 0,
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: vec![
                AnimationKeyframeV1 {
                    id: "k0".into(),
                    time_seconds: 0.0,
                    value: vec![start, 0.0, 0.0],
                },
                AnimationKeyframeV1 {
                    id: "k1".into(),
                    time_seconds: 1.0,
                    value: vec![end, 0.0, 0.0],
                },
            ],
        }],
        events: vec![],
        revision: 1,
    }
}

#[test]
fn builds_idle_attack_idle_preview_and_reports_transition_jumps() {
    let idle = clip("idle", 0.0, 0.0);
    let attack = clip("attack", 1.0, 2.0);
    let request = AnimationSequencePreviewRequestV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        sequence_id: "preview-idle-attack-idle".into(),
        output_name: "idle_attack_idle".into(),
        clip_ids: vec!["idle".into(), "attack".into(), "idle".into()],
    };

    let result = build_animation_sequence_preview_v1(&request, &[idle, attack]).unwrap();

    assert_eq!(result.segments.len(), 3);
    assert_eq!(result.transition_jumps.len(), 2);
    assert_eq!(result.preview_clip.length_seconds, 3.0);
    assert_eq!(result.preview_clip.tracks[0].keyframes.len(), 4);
    assert!(result.transition_jumps[0].max_translation_delta > 0.9);
    assert_eq!(result.fingerprint_sha256.len(), 64);
}

#[test]
fn stale_sequence_request_fails_without_partial_preview() {
    let mut stale = clip("idle", 0.0, 0.0);
    stale.source.source_revision = "b".repeat(64);
    let request = AnimationSequencePreviewRequestV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        sequence_id: "preview".into(),
        output_name: "preview".into(),
        clip_ids: vec!["idle".into(), "idle".into()],
    };

    assert!(build_animation_sequence_preview_v1(&request, &[stale]).is_err());
}

fn segment(id: &str, clip_id: &str, blend: f32) -> AnimationSequenceSegmentRecipeV1 {
    AnimationSequenceSegmentRecipeV1 {
        segment_id: id.into(),
        clip_id: clip_id.into(),
        source_in_seconds: 0.0,
        source_out_seconds: 1.0,
        repeat_count: 1,
        transition_from_previous: AnimationSequenceTransitionV1 {
            kind: if blend == 0.0 {
                AnimationSequenceTransitionKindV1::Cut
            } else {
                AnimationSequenceTransitionKindV1::CrossFade
            },
            duration_seconds: blend,
        },
        phase_markers: vec![AnimationSequencePhaseMarkerV1 {
            kind: "IMPACT".into(),
            time_seconds: 0.5,
        }],
    }
}

#[test]
fn composer_reorders_trims_repeats_blends_four_segments_and_keeps_sources_immutable() {
    let clips = vec![
        clip("idle", 0.0, 0.0),
        clip("attack-a", 1.0, 2.0),
        clip("attack-b", 2.0, -1.0),
        clip("recovery", -1.0, 0.0),
    ];
    let before = serde_json::to_vec(&clips).unwrap();
    let mut attack_a = segment("s1", "attack-a", 0.0);
    attack_a.source_in_seconds = 0.25;
    attack_a.source_out_seconds = 0.75;
    attack_a.repeat_count = 2;
    attack_a.phase_markers[0].time_seconds = 0.5;
    let document = AnimationSequenceDocumentV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        sequence_id: "combo".into(),
        output_name: "Combo".into(),
        segments: vec![
            segment("s0", "idle", 0.0),
            attack_a,
            segment("s2", "attack-b", 0.2),
            segment("s3", "recovery", 0.15),
        ],
        revision: 4,
    };

    let result = bake_animation_sequence_document_v1(&document, &clips).unwrap();

    assert_eq!(result.placed_segments.len(), 4);
    assert_eq!(result.document_revision, 4);
    assert_eq!(result.custom_clip.id, "combo-custom");
    assert!(result.custom_clip.length_seconds > 3.0);
    assert_eq!(result.phase_markers.len(), 5);
    assert_eq!(result.source_provenance.len(), 4);
    assert_eq!(result.fingerprint_sha256.len(), 64);
    assert_eq!(serde_json::to_vec(&clips).unwrap(), before);
    let crossfade_start = result.placed_segments[2].start_seconds;
    let key = result.custom_clip.tracks[0]
        .keyframes
        .iter()
        .find(|key| (key.time_seconds - (crossfade_start + 0.1)).abs() < 0.009)
        .unwrap();
    assert!(key.value[0].is_finite());
}

#[test]
fn composer_fails_atomically_for_stale_or_invalid_recipe() {
    let clips = vec![clip("idle", 0.0, 0.0), clip("attack", 1.0, 2.0)];
    let mut document = AnimationSequenceDocumentV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        sequence_id: "invalid".into(),
        output_name: "Invalid".into(),
        segments: vec![segment("same", "idle", 0.0), segment("same", "attack", 0.1)],
        revision: 1,
    };
    assert!(bake_animation_sequence_document_v1(&document, &clips).is_err());
    document.segments[1].segment_id = "other".into();
    document.source_revision = "b".repeat(64);
    assert!(bake_animation_sequence_document_v1(&document, &clips).is_err());
}
