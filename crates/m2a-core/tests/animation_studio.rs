use m2a_core::{
    animation_studio::{
        ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION, ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE,
        ANIMATION_STUDIO_MAX_AUTHORED_CLIPS, ANIMATION_STUDIO_MAX_DURATION_SECONDS,
        ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP, ANIMATION_STUDIO_MAX_TOTAL_KEYFRAMES,
        AnimationKeyframePatchV1, AnimationKeyframeV1, AnimationStudioDocumentStatusV1,
        AnimationStudioDocumentV1, AnimationStudioRigNodeV1, AnimationStudioRigV1,
        AuthoredAnimationClipInputV1, AuthoredAnimationClipKindV1, AuthoredAnimationClipStatusV1,
        AuthoredAnimationEventPatchV1, AuthoredAnimationEventV1, AuthoredAnimationSourceKindV1,
        AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1, CreatureAnimationAuthoringV2,
        CustomAnimationClipReferenceKindV2, KeyframeDeduplicationPolicyV1,
        ProceduralAnimationTemplateV1, RemoveAuthoredClipPolicyV1, add_animation_track_v1,
        add_authored_animation_event_v1, apply_integrated_authored_events_v1,
        clamp_authored_animation_keys_v1, clone_source_clip_for_editing_v1,
        create_blank_pose_clip_v1, create_procedural_template_clip_v1, detect_authored_motion_v1,
        duplicate_authored_clip_v1, evaluate_authored_clip_status_v1,
        fingerprint_animation_studio_document_v1, fingerprint_creature_animation_authoring_v2,
        insert_animation_keyframe_v1, materialize_animation_studio_document_v1,
        materialize_authored_animation_library_v1, materialize_creature_animation_authoring_v2,
        materialize_custom_animation_definition_v2, migrate_creature_animation_authoring_v1_to_v2,
        move_animation_keyframe_v1, move_authored_animation_event_v1,
        normalize_animation_quaternions_v1, parse_animation_studio_document_v1,
        remove_animation_keyframe_v1, remove_animation_track_v1,
        remove_authored_animation_event_v1, rename_authored_clip_v1,
        retime_authored_animation_clip_v1, sample_animation_track_linear_v1,
        serialize_animation_studio_document_v1, set_authored_animation_transition_v1,
        shift_authored_animation_keys_v1, sort_and_deduplicate_keyframes_v1,
        sort_authored_animation_events_stable_v1, trim_authored_animation_clip_v1,
        update_animation_keyframe_v1, update_authored_animation_event_v1,
        validate_animation_studio_schema_v1, validate_authored_animation_clip_v1,
        validate_authored_event_v1, validate_authored_track_target_v1,
        validate_authored_track_times_v1, validate_authored_track_values_v1,
        validate_creature_animation_authoring_v2,
    },
    creature_animation_mapping::{
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CreatureAnimationAuthoringV1,
        CustomAnimationDefinitionV1, CustomAnimationPlaybackV1, DirectCreatureBaseSlotV1,
        DirectCreatureModelTypeV1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
        MdlAnimationTrackPathV1, MdlAnimationTrackV1,
    },
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AnimationStudioOperationsFixtureV1 {
    schema_version: u32,
    initial_document: AnimationStudioDocumentV1,
    operations: Vec<AnimationStudioGoldenOperationV1>,
    expected_document: AnimationStudioDocumentV1,
    expected_fingerprint_sha256: String,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "SCREAMING_SNAKE_CASE")]
enum AnimationStudioGoldenOperationV1 {
    InsertKeyframe {
        #[serde(rename = "trackId")]
        track_id: String,
        #[serde(rename = "timeSeconds")]
        time_seconds: f32,
        value: Vec<f32>,
    },
    UpdateKeyframe {
        #[serde(rename = "trackId")]
        track_id: String,
        #[serde(rename = "keyId")]
        key_id: String,
        #[serde(rename = "timeSeconds")]
        time_seconds: f32,
        value: Vec<f32>,
    },
    MoveKeyframe {
        #[serde(rename = "trackId")]
        track_id: String,
        #[serde(rename = "keyId")]
        key_id: String,
        #[serde(rename = "timeSeconds")]
        time_seconds: f32,
    },
    NormalizeQuaternions {
        #[serde(rename = "trackId")]
        track_id: String,
    },
    TrimClip {
        #[serde(rename = "startSeconds")]
        start_seconds: f32,
        #[serde(rename = "endSeconds")]
        end_seconds: f32,
    },
    RetimeClip {
        #[serde(rename = "newLengthSeconds")]
        new_length_seconds: f32,
    },
}

fn rig() -> AnimationStudioRigV1 {
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        animation_root: "root".to_owned(),
        nodes: vec![
            AnimationStudioRigNodeV1 {
                node_id: 0,
                name: "root".to_owned(),
                parent_id: None,
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: 7,
                name: "hand".to_owned(),
                parent_id: Some(0),
                translation: [1.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
        ],
    }
}

fn input(id: &str, name: &str) -> AuthoredAnimationClipInputV1 {
    AuthoredAnimationClipInputV1 {
        id: id.to_owned(),
        name: name.to_owned(),
        source_revision: "a".repeat(64),
        length_seconds: 1.0,
        transition_seconds: 0.1,
        animation_root: "root".to_owned(),
    }
}

fn document() -> AnimationStudioDocumentV1 {
    AnimationStudioDocumentV1 {
        schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION,
        source_revision: "a".repeat(64),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Draft,
        authored_clips: vec![create_blank_pose_clip_v1(&rig(), input("pose", "pose")).unwrap()],
    }
}

#[test]
fn studio_document_serde_is_strict_stable_and_round_trips() {
    let document = document();
    let json = serialize_animation_studio_document_v1(&document).unwrap();
    let decoded = parse_animation_studio_document_v1(&json).unwrap();
    assert_eq!(decoded, document);
    assert_eq!(
        fingerprint_animation_studio_document_v1(&document),
        fingerprint_animation_studio_document_v1(&decoded)
    );

    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(parse_animation_studio_document_v1(&serde_json::to_string(&value).unwrap()).is_err());

    value.as_object_mut().unwrap().remove("unexpected");
    value["schemaVersion"] = serde_json::json!(2);
    let diagnostics = validate_animation_studio_schema_v1(&serde_json::from_value(value).unwrap());
    assert_eq!(diagnostics[0].code, "M2A-ANIMATION-EDIT-SCHEMA");
}

#[test]
fn rust_and_typescript_share_the_exact_wire_fixture_and_fingerprint() {
    let fixture = include_str!(
        "../../../apps/studio-web/src/features/animation-studio/fixtures/animation-studio-document-v1.json"
    )
    .trim();
    let document = parse_animation_studio_document_v1(fixture).unwrap();
    assert_eq!(
        serialize_animation_studio_document_v1(&document).unwrap(),
        fixture
    );
    assert_eq!(
        fingerprint_animation_studio_document_v1(&document),
        "4a3b66cd909292c33e7eae7f4e881b359525cd0234b95098084a6d462311d36a"
    );
}

#[test]
fn canonical_f2_operation_sequence_matches_the_shared_exact_golden() {
    let fixture: AnimationStudioOperationsFixtureV1 = serde_json::from_str(include_str!(
        "../../../apps/studio-web/src/features/animation-studio/fixtures/animation-studio-operations-v1.json"
    ))
    .unwrap();
    assert_eq!(fixture.schema_version, 1);
    let mut document = fixture.initial_document;
    for operation in fixture.operations {
        let clip = document.authored_clips.first_mut().unwrap();
        match operation {
            AnimationStudioGoldenOperationV1::InsertKeyframe {
                track_id,
                time_seconds,
                value,
            } => {
                let track = clip
                    .tracks
                    .iter_mut()
                    .find(|track| track.id == track_id)
                    .unwrap();
                assert_eq!(
                    insert_animation_keyframe_v1(track, time_seconds, value).unwrap(),
                    "key-0000"
                );
                clip.revision += 1;
                clip.status = AuthoredAnimationClipStatusV1::Draft;
            }
            AnimationStudioGoldenOperationV1::UpdateKeyframe {
                track_id,
                key_id,
                time_seconds,
                value,
            } => {
                let track = clip
                    .tracks
                    .iter_mut()
                    .find(|track| track.id == track_id)
                    .unwrap();
                update_animation_keyframe_v1(
                    track,
                    &key_id,
                    AnimationKeyframePatchV1 {
                        time_seconds: Some(time_seconds),
                        value: Some(value),
                    },
                )
                .unwrap();
                clip.revision += 1;
                clip.status = AuthoredAnimationClipStatusV1::Draft;
            }
            AnimationStudioGoldenOperationV1::MoveKeyframe {
                track_id,
                key_id,
                time_seconds,
            } => {
                let track = clip
                    .tracks
                    .iter_mut()
                    .find(|track| track.id == track_id)
                    .unwrap();
                move_animation_keyframe_v1(track, &key_id, time_seconds).unwrap();
                clip.revision += 1;
                clip.status = AuthoredAnimationClipStatusV1::Draft;
            }
            AnimationStudioGoldenOperationV1::NormalizeQuaternions { track_id } => {
                let track = clip
                    .tracks
                    .iter_mut()
                    .find(|track| track.id == track_id)
                    .unwrap();
                normalize_animation_quaternions_v1(track).unwrap();
                clip.revision += 1;
                clip.status = AuthoredAnimationClipStatusV1::Draft;
            }
            AnimationStudioGoldenOperationV1::TrimClip {
                start_seconds,
                end_seconds,
            } => trim_authored_animation_clip_v1(clip, start_seconds, end_seconds).unwrap(),
            AnimationStudioGoldenOperationV1::RetimeClip { new_length_seconds } => {
                retime_authored_animation_clip_v1(clip, new_length_seconds).unwrap();
            }
        }
    }
    assert_eq!(document, fixture.expected_document);
    assert_eq!(
        serialize_animation_studio_document_v1(&document).unwrap(),
        serialize_animation_studio_document_v1(&fixture.expected_document).unwrap()
    );
    assert_eq!(
        fingerprint_animation_studio_document_v1(&document),
        fingerprint_animation_studio_document_v1(&fixture.expected_document)
    );
    assert_eq!(
        fingerprint_animation_studio_document_v1(&document),
        fixture.expected_fingerprint_sha256
    );
}

#[test]
fn v1_to_v2_migration_is_lossless_and_keeps_source_clip_references() {
    let provenance = AnimationMappingProvenanceV1 {
        provider: AnimationProviderV1::SourceGlb,
        asset_id: "asset-1".to_owned(),
        ownership: AnimationOwnershipV1::UserOwned,
    };
    let v1 = CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: "a".repeat(64),
        authoring_revision: 9,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                source_kind: AnimationSourceKindV1::SourceClip,
                source_clip_name: Some((*slot).to_owned()),
                custom_animation_id: None,
                provenance: provenance.clone(),
            })
            .collect(),
        fallbacks: vec![],
        custom_animations: vec![CustomAnimationDefinitionV1 {
            id: "custom-1".to_owned(),
            name: "dance".to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            source_clip_name: Some("Dance".to_owned()),
            phases: vec![],
            provenance,
        }],
    };

    let v2 = migrate_creature_animation_authoring_v1_to_v2(&v1);
    assert_eq!(v2.schema_version, 2);
    assert_eq!(v2.assignments.len(), 42);
    assert_eq!(v2.assignments, v1.assignments);
    assert_eq!(v2.fallbacks, v1.fallbacks);
    let reference = v2.custom_animations[0].clip_reference.as_ref().unwrap();
    assert_eq!(
        reference.source_kind,
        CustomAnimationClipReferenceKindV2::SourceClip
    );
    assert_eq!(reference.source_clip_name.as_deref(), Some("Dance"));
    assert_eq!(reference.authored_clip_id, None);

    let first = serde_json::to_string(&v2).unwrap();
    let second =
        serde_json::to_string(&migrate_creature_animation_authoring_v1_to_v2(&v1)).unwrap();
    assert_eq!(first, second);
}

#[test]
fn schema_validation_blocks_duplicate_ids_names_and_stale_source() {
    let mut document = document();
    let mut duplicate = document.authored_clips[0].clone();
    duplicate.name = "POSE".to_owned();
    document.authored_clips.push(duplicate);
    document.source_revision = "b".repeat(64);
    let diagnostics = validate_animation_studio_schema_v1(&document);
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"M2A-ANIMATION-EDIT-CLIP-DUPLICATE"));
    assert!(codes.contains(&"M2A-ANIMATION-EDIT-SOURCE-STALE"));
}

#[test]
fn product_limits_and_portable_event_names_fail_closed() {
    let mut too_many_clips = document();
    let template = too_many_clips.authored_clips[0].clone();
    too_many_clips.authored_clips = (0..=ANIMATION_STUDIO_MAX_AUTHORED_CLIPS)
        .map(|index| {
            let mut clip = template.clone();
            clip.id = format!("clip-{index}");
            clip.name = format!("clip_{index}");
            clip
        })
        .collect();
    assert!(
        validate_animation_studio_schema_v1(&too_many_clips)
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "M2A-ANIMATION-EDIT-ROW-LIMIT"
                    && diagnostic.message.contains("authored clips")
            })
    );

    let mut too_many_per_clip = document();
    too_many_per_clip.authored_clips[0].tracks[0].keyframes = (0
        ..=ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP)
        .map(|index| AnimationKeyframeV1 {
            id: format!("key-{index}"),
            time_seconds: 0.0,
            value: vec![0.0, 0.0, 0.0],
        })
        .collect();
    assert!(
        validate_authored_animation_clip_v1(&too_many_per_clip.authored_clips[0], &rig())
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "M2A-ANIMATION-EDIT-ROW-LIMIT"
                    && diagnostic.message.contains("product limit")
            })
    );

    let mut too_many_total = document();
    too_many_total.authored_clips = (0..=(ANIMATION_STUDIO_MAX_TOTAL_KEYFRAMES
        / ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP))
        .map(|index| {
            let mut clip = template.clone();
            clip.id = format!("total-{index}");
            clip.name = format!("total_{index}");
            clip.tracks[0].keyframes = (0..ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP)
                .map(|key_index| AnimationKeyframeV1 {
                    id: format!("key-{key_index}"),
                    time_seconds: 0.0,
                    value: vec![0.0, 0.0, 0.0],
                })
                .collect();
            clip
        })
        .collect();
    assert!(
        validate_animation_studio_schema_v1(&too_many_total)
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "M2A-ANIMATION-EDIT-ROW-LIMIT"
                    && diagnostic.message.contains("Studio document contains")
                    && diagnostic.message.contains("keyframes")
            })
    );

    let too_long = AuthoredAnimationEventV1 {
        id: "long".to_owned(),
        time_seconds: 0.5,
        name: "a".repeat(32),
    };
    assert_eq!(
        validate_authored_event_v1(&too_long, 1.0).unwrap_err().code,
        "M2A-ANIMATION-EDIT-EVENT"
    );
    let non_ascii = AuthoredAnimationEventV1 {
        id: "unicode".to_owned(),
        time_seconds: 0.5,
        name: "uderzenie-ż".to_owned(),
    };
    assert_eq!(
        validate_authored_event_v1(&non_ascii, 1.0)
            .unwrap_err()
            .code,
        "M2A-ANIMATION-EDIT-EVENT"
    );

    let mut out_of_product_range = document().authored_clips.remove(0);
    out_of_product_range.length_seconds = ANIMATION_STUDIO_MAX_DURATION_SECONDS + 1.0;
    out_of_product_range.tracks[0].keyframes[0].value[0] = 1.0e20;
    let range_diagnostics = validate_authored_animation_clip_v1(&out_of_product_range, &rig());
    assert!(range_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "M2A-ANIMATION-EDIT-TIME-OOB"
            && diagnostic.path.ends_with("lengthSeconds")
    }));
    assert!(range_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "M2A-ANIMATION-EDIT-VALUE-NONFINITE"
            && diagnostic.message.contains("1000000")
    }));

    let mut exact_track = document().authored_clips.remove(0).tracks.remove(0);
    exact_track.path = AuthoredAnimationTrackPathV1::Translation;
    exact_track.keyframes[0].value = vec![ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE, 0.0, 0.0];
    assert!(validate_authored_track_values_v1(&exact_track).is_empty());
    assert_eq!(
        insert_animation_keyframe_v1(
            &mut exact_track,
            0.5,
            vec![ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE + 1.0, 0.0, 0.0],
        )
        .unwrap_err()
        .code,
        "M2A-ANIMATION-EDIT-VALUE-NONFINITE"
    );
}

#[test]
fn blank_clone_and_procedural_creation_are_deterministic() {
    let blank = create_blank_pose_clip_v1(&rig(), input("blank", "blank_pose")).unwrap();
    assert_eq!(blank.kind, AuthoredAnimationClipKindV1::StaticPose);
    assert_eq!(blank.tracks.len(), 4);
    assert!(blank.tracks.iter().all(|track| track.keyframes.len() == 1));
    assert!(validate_authored_animation_clip_v1(&blank, &rig()).is_empty());
    let mut mislabeled_pose = blank.clone();
    insert_animation_keyframe_v1(&mut mislabeled_pose.tracks[0], 1.0, vec![0.5, 0.0, 0.0]).unwrap();
    assert!(
        validate_authored_animation_clip_v1(&mislabeled_pose, &rig())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-EDIT-NO-MOTION")
    );

    let source = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "Wave".to_owned(),
            animation_root: "root".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            events: vec![],
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 7,
                path: MdlAnimationTrackPathV1::Translation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0]],
            }],
        }],
    };
    let cloned =
        clone_source_clip_for_editing_v1(&source, "wave", input("wave-edit", "my_wave")).unwrap();
    assert_eq!(
        cloned.source.kind,
        AuthoredAnimationSourceKindV1::SourceClipCopy
    );
    assert_eq!(cloned.source.source_clip_name.as_deref(), Some("Wave"));
    assert_eq!(cloned.tracks[0].keyframes[1].value, vec![1.0, 0.0, 0.0]);
    assert!(detect_authored_motion_v1(&cloned));

    let procedural = create_procedural_template_clip_v1(
        &rig(),
        ProceduralAnimationTemplateV1::RootTranslationPulse,
        input("pulse", "pulse"),
    )
    .unwrap();
    assert!(detect_authored_motion_v1(&procedural));
    assert_eq!(
        serde_json::to_string(&procedural).unwrap(),
        serde_json::to_string(
            &create_procedural_template_clip_v1(
                &rig(),
                ProceduralAnimationTemplateV1::RootTranslationPulse,
                input("pulse", "pulse"),
            )
            .unwrap()
        )
        .unwrap()
    );
}

#[test]
fn keyframe_crud_sampling_dedup_and_quaternion_canonicalization_work() {
    let mut track = AuthoredAnimationTrackV1 {
        id: "track".to_owned(),
        target_node_id: 7,
        path: AuthoredAnimationTrackPathV1::Translation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: vec![],
    };
    let first = insert_animation_keyframe_v1(&mut track, 0.0, vec![0.0, 0.0, 0.0]).unwrap();
    let second = insert_animation_keyframe_v1(&mut track, 1.0, vec![2.0, 0.0, 0.0]).unwrap();
    assert_eq!(
        sample_animation_track_linear_v1(&track, 0.5).unwrap(),
        vec![1.0, 0.0, 0.0]
    );
    update_animation_keyframe_v1(
        &mut track,
        &second,
        AnimationKeyframePatchV1 {
            time_seconds: None,
            value: Some(vec![4.0, 0.0, 0.0]),
        },
    )
    .unwrap();
    move_animation_keyframe_v1(&mut track, &second, 0.5).unwrap();
    assert_eq!(
        sample_animation_track_linear_v1(&track, 0.25).unwrap(),
        vec![2.0, 0.0, 0.0]
    );
    remove_animation_keyframe_v1(&mut track, &first).unwrap();
    assert_eq!(track.keyframes.len(), 1);

    track.keyframes.push(AnimationKeyframeV1 {
        id: "replacement".to_owned(),
        time_seconds: 0.5,
        value: vec![8.0, 0.0, 0.0],
    });
    assert!(
        sort_and_deduplicate_keyframes_v1(&mut track, KeyframeDeduplicationPolicyV1::Fail).is_err()
    );
    sort_and_deduplicate_keyframes_v1(&mut track, KeyframeDeduplicationPolicyV1::ReplaceLast)
        .unwrap();
    assert_eq!(track.keyframes[0].id, "replacement");

    let mut rotation = AuthoredAnimationTrackV1 {
        id: "rotation".to_owned(),
        target_node_id: 7,
        path: AuthoredAnimationTrackPathV1::Rotation,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: vec![AnimationKeyframeV1 {
            id: "q".to_owned(),
            time_seconds: 0.0,
            value: vec![0.0, 0.0, 0.0, -2.0],
        }],
    };
    normalize_animation_quaternions_v1(&mut rotation).unwrap();
    assert_eq!(rotation.keyframes[0].value, vec![0.0, 0.0, 0.0, 1.0]);
    assert!(
        rotation.keyframes[0].value[..3]
            .iter()
            .all(|component| component.to_bits() == 0.0_f32.to_bits())
    );
}

#[test]
fn trim_samples_boundaries_and_retime_preserves_relative_times() {
    let source = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "Move".to_owned(),
            animation_root: "root".to_owned(),
            length_seconds: 2.0,
            transition_seconds: 0.1,
            events: vec![],
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 7,
                path: MdlAnimationTrackPathV1::Translation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0, 2.0],
                values: vec![
                    vec![0.0, 0.0, 0.0],
                    vec![1.0, 0.0, 0.0],
                    vec![2.0, 0.0, 0.0],
                ],
            }],
        }],
    };
    let mut clip =
        clone_source_clip_for_editing_v1(&source, "Move", input("move", "move")).unwrap();
    clip.length_seconds = 2.0;
    clip.events.push(AuthoredAnimationEventV1 {
        id: "impact".to_owned(),
        time_seconds: 1.0,
        name: "impact".to_owned(),
    });

    let mut boundary_clip = clip.clone();
    trim_authored_animation_clip_v1(&mut boundary_clip, 1.0, 2.0).unwrap();
    assert_eq!(boundary_clip.tracks[0].keyframes[0].id, "key-0001");
    assert_eq!(boundary_clip.tracks[0].keyframes[0].time_seconds, 0.0);
    assert_eq!(
        boundary_clip.tracks[0].keyframes[0].value,
        vec![1.0, 0.0, 0.0]
    );

    trim_authored_animation_clip_v1(&mut clip, 0.5, 1.5).unwrap();
    assert_eq!(clip.length_seconds, 1.0);
    assert_eq!(clip.tracks[0].keyframes[0].time_seconds, 0.0);
    assert_eq!(clip.tracks[0].keyframes[0].value, vec![0.5, 0.0, 0.0]);
    assert_eq!(clip.tracks[0].keyframes[2].time_seconds, 1.0);
    assert_eq!(clip.tracks[0].keyframes[2].value, vec![1.5, 0.0, 0.0]);
    assert_eq!(clip.events[0].time_seconds, 0.5);

    retime_authored_animation_clip_v1(&mut clip, 2.0).unwrap();
    assert_eq!(clip.tracks[0].keyframes[2].time_seconds, 2.0);
    assert_eq!(clip.events[0].time_seconds, 1.0);
}

#[test]
fn direct_track_time_status_and_validation_helpers_are_covered() {
    let mut clip = create_procedural_template_clip_v1(
        &rig(),
        ProceduralAnimationTemplateV1::RootTranslationPulse,
        input("direct-ops", "direct_ops"),
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    add_authored_animation_event_v1(
        &mut clip,
        AuthoredAnimationEventV1 {
            id: "first".to_owned(),
            time_seconds: 0.5,
            name: "first".to_owned(),
        },
    )
    .unwrap();
    add_authored_animation_event_v1(
        &mut clip,
        AuthoredAnimationEventV1 {
            id: "second".to_owned(),
            time_seconds: 0.5,
            name: "second".to_owned(),
        },
    )
    .unwrap();
    sort_authored_animation_events_stable_v1(&mut clip);
    assert_eq!(
        clip.events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );

    shift_authored_animation_keys_v1(&mut clip, 0.25).unwrap();
    assert_eq!(clip.tracks[0].keyframes[0].time_seconds, 0.25);
    assert_eq!(clip.events[0].time_seconds, 0.75);
    clamp_authored_animation_keys_v1(&mut clip);
    assert_eq!(
        clip.tracks[0].keyframes.last().unwrap().time_seconds,
        clip.length_seconds
    );
    set_authored_animation_transition_v1(&mut clip, 0.4).unwrap();
    assert_eq!(clip.transition_seconds, 0.4);
    assert!(set_authored_animation_transition_v1(&mut clip, 1.1).is_err());
    assert_eq!(
        evaluate_authored_clip_status_v1(&clip, &rig()),
        AuthoredAnimationClipStatusV1::Valid
    );

    let removed_target = clip.tracks[0].target_node_id;
    let removed_path = clip.tracks[0].path;
    remove_animation_track_v1(&mut clip, removed_target, removed_path).unwrap();
    assert!(remove_animation_track_v1(&mut clip, removed_target, removed_path).is_err());

    let mut invalid_track = clip.tracks[0].clone();
    invalid_track.target_node_id = 999;
    assert!(!validate_authored_track_target_v1(&invalid_track, &rig()).is_empty());
    invalid_track.target_node_id = 0;
    invalid_track.keyframes[0].time_seconds = clip.length_seconds + 1.0;
    assert!(!validate_authored_track_times_v1(&invalid_track, clip.length_seconds).is_empty());
    invalid_track.keyframes[0].value = vec![0.0];
    assert!(!validate_authored_track_values_v1(&invalid_track).is_empty());
    clip.tracks[0].keyframes[0].value[0] = f32::NAN;
    assert_eq!(
        evaluate_authored_clip_status_v1(&clip, &rig()),
        AuthoredAnimationClipStatusV1::Invalid
    );
}

#[test]
fn public_materialization_helpers_are_exact_and_deterministic() {
    let source = MdlAnimationSetV1 {
        schema_version: 1,
        clips: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|name| MdlAnimationClipV1 {
                name: (*name).to_owned(),
                animation_root: "root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.1,
                events: vec![],
                tracks: vec![],
            })
            .collect(),
    };
    let mut authored = create_procedural_template_clip_v1(
        &rig(),
        ProceduralAnimationTemplateV1::RootTranslationPulse,
        input("authored-pulse", "authored_pulse"),
    )
    .unwrap();
    authored.status = AuthoredAnimationClipStatusV1::Valid;
    authored.events.push(AuthoredAnimationEventV1 {
        id: "impact".to_owned(),
        time_seconds: 0.5,
        name: "impact".to_owned(),
    });
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        authoring_revision: 2,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![authored],
    };
    let library = materialize_authored_animation_library_v1(&source, &studio, &rig()).unwrap();
    assert_eq!(
        library,
        materialize_authored_animation_library_v1(&source, &studio, &rig()).unwrap()
    );

    let custom = m2a_core::animation_studio::CustomAnimationDefinitionV2 {
        id: "custom-pulse".to_owned(),
        name: "custpulse".to_owned(),
        playback: CustomAnimationPlaybackV1::OneShot,
        clip_reference: Some(m2a_core::animation_studio::CustomAnimationClipReferenceV2 {
            source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
            source_clip_name: None,
            authored_clip_id: Some("authored-pulse".to_owned()),
        }),
        phases: vec![],
        provenance: AnimationMappingProvenanceV1 {
            provider: AnimationProviderV1::UserCustom,
            asset_id: "authored-pulse".to_owned(),
            ownership: AnimationOwnershipV1::UserOwned,
        },
    };
    let custom_output =
        materialize_custom_animation_definition_v2(&custom, &source, &library).unwrap();
    assert_eq!(custom_output[0].clip.name, "custpulse");
    assert_eq!(
        custom_output[0].authored_clip_id.as_deref(),
        Some("authored-pulse")
    );

    let authoring = CreatureAnimationAuthoringV2 {
        schema_version: 2,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: "a".repeat(64),
        authoring_revision: 3,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                source_kind: AnimationSourceKindV1::SourceClip,
                source_clip_name: Some((*slot).to_owned()),
                custom_animation_id: None,
                provenance: AnimationMappingProvenanceV1 {
                    provider: AnimationProviderV1::SourceGlb,
                    asset_id: "fixture".to_owned(),
                    ownership: AnimationOwnershipV1::UserOwned,
                },
            })
            .collect(),
        fallbacks: vec![],
        custom_animations: vec![custom],
    };
    assert_eq!(
        fingerprint_creature_animation_authoring_v2(&authoring),
        fingerprint_creature_animation_authoring_v2(&authoring.clone())
    );
    let materialized =
        materialize_creature_animation_authoring_v2(&source, None, &authoring, &studio, &rig())
            .unwrap();
    assert_eq!(materialized.animations.clips.len(), 43);
    assert_eq!(materialized.authored_usages.len(), 1);
    assert_eq!(
        materialized.authored_usages[0].output_clip_name,
        "custpulse"
    );
    assert_eq!(
        materialized,
        materialize_creature_animation_authoring_v2(&source, None, &authoring, &studio, &rig())
            .unwrap()
    );

    let mut without_events = materialize_animation_studio_document_v1(&studio, &rig()).unwrap();
    without_events.clips[0].events.clear();
    let with_events = apply_integrated_authored_events_v1(&without_events, &studio).unwrap();
    assert_eq!(with_events.clips[0].events[0].name, "impact");
}

#[test]
fn event_crud_is_stable_and_clip_crud_preserves_identity_rules() {
    let mut document = document();
    rename_authored_clip_v1(&mut document, "pose", "renamed").unwrap();
    let copy_id = duplicate_authored_clip_v1(&mut document, "pose", "copy").unwrap();
    assert_eq!(copy_id, "pose_copy");
    assert_eq!(document.authored_clips[0].id, "pose");

    let clip = &mut document.authored_clips[0];
    add_authored_animation_event_v1(
        clip,
        AuthoredAnimationEventV1 {
            id: "event".to_owned(),
            time_seconds: 0.7,
            name: "hit".to_owned(),
        },
    )
    .unwrap();
    update_authored_animation_event_v1(
        clip,
        "event",
        AuthoredAnimationEventPatchV1 {
            time_seconds: None,
            name: Some("impact".to_owned()),
        },
    )
    .unwrap();
    move_authored_animation_event_v1(clip, "event", 0.3).unwrap();
    assert_eq!(clip.events[0].name, "impact");
    assert_eq!(clip.events[0].time_seconds, 0.3);
    remove_authored_animation_event_v1(clip, "event").unwrap();
    assert!(clip.events.is_empty());

    m2a_core::animation_studio::remove_authored_clip_v1(
        &mut document,
        &copy_id,
        RemoveAuthoredClipPolicyV1::Explicit,
    )
    .unwrap();
    assert_eq!(document.authored_clips.len(), 1);
}

#[test]
fn validation_and_materialization_fail_closed_then_emit_mdl_animation_set() {
    let mut document = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Draft,
        authored_clips: vec![
            create_procedural_template_clip_v1(
                &rig(),
                ProceduralAnimationTemplateV1::RootTranslationPulse,
                input("pulse", "custom_pulse"),
            )
            .unwrap(),
        ],
    };
    document.authored_clips[0].status = AuthoredAnimationClipStatusV1::Valid;
    assert!(validate_authored_animation_clip_v1(&document.authored_clips[0], &rig()).is_empty());

    let set = materialize_animation_studio_document_v1(&document, &rig()).unwrap();
    assert_eq!(set.schema_version, 1);
    assert_eq!(set.clips.len(), 1);
    assert_eq!(set.clips[0].name, "custom_pulse");
    assert!(set.clips[0].tracks.iter().all(|track| {
        track.path == MdlAnimationTrackPathV1::Translation
            || track.path == MdlAnimationTrackPathV1::Rotation
    }));

    add_animation_track_v1(
        &mut document.authored_clips[0],
        999,
        AuthoredAnimationTrackPathV1::Translation,
    )
    .unwrap();
    let diagnostics = validate_authored_animation_clip_v1(&document.authored_clips[0], &rig());
    assert_eq!(
        diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "M2A-ANIMATION-EDIT-BONE-MISSING")
            .unwrap()
            .path,
        "authoredClips[pulse].tracks[track-999-translation].targetNodeId"
    );
    assert!(materialize_animation_studio_document_v1(&document, &rig()).is_err());
}

#[test]
fn nonfinite_oob_and_v2_missing_authored_clip_are_blocking() {
    let mut document = document();
    let track = &mut document.authored_clips[0].tracks[0];
    track.keyframes[0].value[0] = f32::NAN;
    track.keyframes.push(AnimationKeyframeV1 {
        id: "late".to_owned(),
        time_seconds: 2.0,
        value: vec![0.0, 0.0, 0.0],
    });
    let diagnostics = validate_authored_animation_clip_v1(&document.authored_clips[0], &rig());
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"M2A-ANIMATION-EDIT-VALUE-NONFINITE"));
    assert!(codes.contains(&"M2A-ANIMATION-EDIT-TIME-OOB"));
    clamp_authored_animation_keys_v1(&mut document.authored_clips[0]);
    assert_eq!(
        document.authored_clips[0].tracks[0].keyframes[1].time_seconds,
        1.0
    );

    let v1 = CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: "a".repeat(64),
        authoring_revision: 1,
        assignments: vec![],
        fallbacks: vec![],
        custom_animations: vec![],
    };
    let mut v2 = migrate_creature_animation_authoring_v1_to_v2(&v1);
    v2.custom_animations
        .push(m2a_core::animation_studio::CustomAnimationDefinitionV2 {
            id: "missing".to_owned(),
            name: "missing".to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            clip_reference: Some(m2a_core::animation_studio::CustomAnimationClipReferenceV2 {
                source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
                source_clip_name: None,
                authored_clip_id: Some("does-not-exist".to_owned()),
            }),
            phases: vec![],
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: "missing".to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });
    assert!(
        validate_creature_animation_authoring_v2(&v2, &document)
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-EDIT-SCHEMA")
    );
}

#[test]
fn studio_mvp_rejects_step_and_requires_complete_unique_custom_phases() {
    let mut studio = document();
    studio.authored_clips[0].status = AuthoredAnimationClipStatusV1::Valid;
    studio.authored_clips[0].tracks[0].interpolation = MdlAnimationInterpolationV1::Step;
    assert!(
        validate_authored_animation_clip_v1(&studio.authored_clips[0], &rig())
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "M2A-ANIMATION-EDIT-PATH-UNSUPPORTED"
                    && diagnostic.message.contains("LINEAR interpolation only")
            })
    );
    studio.authored_clips[0].tracks[0].interpolation = MdlAnimationInterpolationV1::Linear;

    let v1 = CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: "a".repeat(64),
        authoring_revision: 1,
        assignments: vec![],
        fallbacks: vec![],
        custom_animations: vec![],
    };
    let mut v2 = migrate_creature_animation_authoring_v1_to_v2(&v1);
    let reference = m2a_core::animation_studio::CustomAnimationClipReferenceV2 {
        source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
        source_clip_name: None,
        authored_clip_id: Some("pose".to_owned()),
    };
    v2.custom_animations
        .push(m2a_core::animation_studio::CustomAnimationDefinitionV2 {
            id: "phased".to_owned(),
            name: "phased".to_owned(),
            playback: CustomAnimationPlaybackV1::LoopingPhased,
            clip_reference: None,
            phases: vec![
                m2a_core::animation_studio::CustomAnimationPhaseV2 {
                    phase: m2a_core::creature_animation_mapping::CustomAnimationPhaseKindV1::Start,
                    clip_reference: reference.clone(),
                },
                m2a_core::animation_studio::CustomAnimationPhaseV2 {
                    phase: m2a_core::creature_animation_mapping::CustomAnimationPhaseKindV1::Start,
                    clip_reference: reference.clone(),
                },
                m2a_core::animation_studio::CustomAnimationPhaseV2 {
                    phase: m2a_core::creature_animation_mapping::CustomAnimationPhaseKindV1::Loop,
                    clip_reference: reference,
                },
            ],
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: "pose".to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });
    let diagnostics = validate_creature_animation_authoring_v2(&v2, &studio);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate Start phase"))
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("exactly one END phase"))
    );
}
