use std::{collections::BTreeSet, fs};

use m2a_core::{
    animation_retarget::{
        AnimationTransferBatchClipRequestV1, AnimationTransferBatchCommitV1,
        AnimationTransferBatchRequestV2, AnimationTransferCompatibilityStatusV1,
        AnimationTransferModeV1, AnimationTransferOptionsV1, HumanoidBoneSemanticV2,
        HumanoidRetargetCompatibilityStatusV2, HumanoidSemanticOverrideV2,
        build_semantic_rig_mapping_v1, copy_animation_clip_between_models_v1,
        inspect_animation_transfer_compatibility_v1, inspect_humanoid_retarget_compatibility_v2,
        prepare_animation_transfer_batch_v2, retarget_animation_clip_humanoid_v2,
        retarget_animation_clip_same_hierarchy_v1,
    },
    animation_studio::{
        ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V2, ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3,
        AnimationStudioDocumentStatusV1, AnimationStudioDocumentV1, AnimationStudioRigNodeV1,
        AnimationStudioRigV1, AuthoredAnimationClipStatusV1, AuthoredAnimationSourceKindV1,
        AuthoredAnimationTrackPathV1, CreatureAnimationAuthoringV2,
        CustomAnimationClipReferenceKindV2, CustomAnimationClipReferenceV2,
        CustomAnimationDefinitionV2, materialize_creature_animation_authoring_v2,
        migrate_animation_studio_document_v1_or_v2_to_v3, validate_animation_studio_schema_v1,
    },
    creature_animation_mapping::{
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CustomAnimationPlaybackV1,
        DirectCreatureBaseSlotV1, DirectCreatureModelTypeV1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationTrackPathV1,
        MdlAnimationTrackV1,
    },
    owned_fixture::synthetic_owned_m6_full_native_42_glb_v1,
};

#[path = "support/canonical_workspace.rs"]
mod canonical_workspace;

const S: f32 = std::f32::consts::FRAC_1_SQRT_2;

fn rig(ids: [u32; 3], scale: f32, target_rest_rotation: bool) -> AnimationStudioRigV1 {
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: if scale == 1.0 { "d" } else { "b" }.repeat(64),
        animation_root: "Hips".to_owned(),
        nodes: vec![
            AnimationStudioRigNodeV1 {
                node_id: ids[0],
                name: "Hips".to_owned(),
                parent_id: None,
                translation: [0.0, scale, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: ids[1],
                name: "Spine".to_owned(),
                parent_id: Some(ids[0]),
                translation: [0.0, scale, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            AnimationStudioRigNodeV1 {
                node_id: ids[2],
                name: "Hand".to_owned(),
                parent_id: Some(ids[1]),
                translation: [scale, 0.0, 0.0],
                rotation: if target_rest_rotation {
                    [0.0, S, 0.0, S]
                } else {
                    [0.0, 0.0, 0.0, 1.0]
                },
            },
        ],
    }
}

fn attack_clip(ids: [u32; 3]) -> MdlAnimationClipV1 {
    MdlAnimationClipV1 {
        name: "ca1slashl".to_owned(),
        animation_root: "Hips".to_owned(),
        length_seconds: 1.0,
        transition_seconds: 0.1,
        events: vec![],
        tracks: vec![
            MdlAnimationTrackV1 {
                target_node_id: ids[0],
                path: MdlAnimationTrackPathV1::Translation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 1.0, 0.0], vec![1.0, 1.0, 0.0]],
            },
            MdlAnimationTrackV1 {
                target_node_id: ids[1],
                path: MdlAnimationTrackPathV1::Translation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 1.0, 0.0], vec![0.0, 1.0, 0.0]],
            },
            MdlAnimationTrackV1 {
                target_node_id: ids[2],
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 0.0, 0.0, 1.0], vec![S, 0.0, 0.0, S]],
            },
        ],
    }
}

fn semantic_humanoid_rig(base_id: u32, mixamo_names: bool) -> AnimationStudioRigV1 {
    let names = if mixamo_names {
        [
            "mixamorig:Hips",
            "mixamorig:Spine",
            "mixamorig:Head",
            "mixamorig:LeftArm",
            "mixamorig:LeftForeArm",
            "mixamorig:LeftHand",
            "mixamorig:RightArm",
            "mixamorig:RightForeArm",
            "mixamorig:RightHand",
            "mixamorig:LeftUpLeg",
            "mixamorig:LeftLeg",
            "mixamorig:LeftFoot",
            "mixamorig:RightUpLeg",
            "mixamorig:RightLeg",
            "mixamorig:RightFoot",
        ]
    } else {
        [
            "pelvis",
            "spine01",
            "head01",
            "lupperarm",
            "llowerarm",
            "lhand",
            "rupperarm",
            "rlowerarm",
            "rhand",
            "lthigh",
            "lcalf",
            "lfoot",
            "rthigh",
            "rcalf",
            "rfoot",
        ]
    };
    let parent_offsets: [Option<usize>; 15] = [
        None,
        Some(0),
        Some(1),
        Some(1),
        Some(3),
        Some(4),
        Some(1),
        Some(6),
        Some(7),
        Some(0),
        Some(9),
        Some(10),
        Some(0),
        Some(12),
        Some(13),
    ];
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: if mixamo_names { "c" } else { "e" }.repeat(64),
        animation_root: names[0].to_owned(),
        nodes: names
            .into_iter()
            .enumerate()
            .map(|(index, name)| AnimationStudioRigNodeV1 {
                node_id: base_id + index as u32,
                name: name.to_owned(),
                parent_id: parent_offsets[index].map(|parent| base_id + parent as u32),
                translation: if index == 0 {
                    [0.0, 1.0, 0.0]
                } else {
                    [0.0, 0.5, 0.0]
                },
                rotation: [0.0, 0.0, 0.0, 1.0],
            })
            .collect(),
    }
}

fn semantic_hand_clip(right_hand_id: u32, animation_root: &str) -> MdlAnimationClipV1 {
    MdlAnimationClipV1 {
        name: "semantic_attack".into(),
        animation_root: animation_root.into(),
        length_seconds: 1.0,
        transition_seconds: 0.1,
        events: vec![],
        tracks: vec![
            MdlAnimationTrackV1 {
                target_node_id: right_hand_id,
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 0.0, 0.0, 1.0], vec![S, 0.0, 0.0, S]],
            },
            MdlAnimationTrackV1 {
                target_node_id: 99_999,
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0, 1.0]],
            },
        ],
    }
}

#[test]
fn compatibility_uses_unique_names_and_parent_names_not_numeric_ids() {
    let donor = rig([1, 2, 3], 1.0, false);
    let target = rig([100, 200, 300], 1.0, false);
    let report = inspect_animation_transfer_compatibility_v1(&target, &donor);

    assert_eq!(
        report.status,
        AnimationTransferCompatibilityStatusV1::ExactCopy
    );
    assert_eq!(report.allowed_modes.len(), 2);
    assert_eq!(
        report
            .diagnostics
            .iter()
            .filter(|diagnostic| { diagnostic.code == "M2A-ANIMATION-RETARGET-NODE-ID" })
            .count(),
        3,
    );
    let mapping = report.mapping.unwrap();
    assert_eq!(mapping.root_name, "Hips");
    assert!(mapping.entries.iter().any(|entry| {
        entry.bone_name == "Hand" && entry.donor_node_id == 3 && entry.target_node_id == 300
    }));
}

#[test]
fn rest_pose_differences_are_retargetable_but_parent_graph_differences_are_not() {
    let donor = rig([1, 2, 3], 1.0, false);
    let target = rig([10, 20, 30], 2.0, true);
    assert_eq!(
        inspect_animation_transfer_compatibility_v1(&target, &donor).status,
        AnimationTransferCompatibilityStatusV1::RetargetableSameHierarchy
    );

    let mut incompatible = target.clone();
    incompatible.nodes[2].parent_id = Some(10);
    let report = inspect_animation_transfer_compatibility_v1(&incompatible, &donor);
    assert_eq!(
        report.status,
        AnimationTransferCompatibilityStatusV1::Incompatible
    );
    assert!(report.allowed_modes.is_empty());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("parent differs"))
    );
    assert!(build_semantic_rig_mapping_v1(&incompatible, &donor).is_err());
}

#[test]
fn semantic_v2_maps_versioned_aliases_and_ignores_unmapped_optional_tracks() {
    let donor = semantic_humanoid_rig(100, true);
    let target = semantic_humanoid_rig(200, false);
    let compatibility = inspect_humanoid_retarget_compatibility_v2(&target, &donor, &[], false);
    assert_eq!(
        compatibility.status,
        HumanoidRetargetCompatibilityStatusV2::Compatible
    );
    assert!(compatibility.entries.iter().any(|entry| {
        entry.semantic == HumanoidBoneSemanticV2::RightHand
            && entry.donor_node_id == 108
            && entry.target_node_id == 208
    }));
    let result = retarget_animation_clip_humanoid_v2(
        &semantic_hand_clip(108, &donor.animation_root),
        &donor,
        &target,
        &compatibility,
        "semantic-attack",
        "semattack",
    )
    .unwrap();
    assert_eq!(result.clip.tracks.len(), 1);
    assert_eq!(result.clip.tracks[0].target_node_id, 208);
    assert_eq!(
        result
            .clip
            .source
            .retarget
            .as_ref()
            .unwrap()
            .algorithm_version,
        "M2A_HUMANOID_SEMANTIC_CHAIN_V2"
    );
}

#[test]
fn semantic_v2_requires_confirmation_for_manual_mapping() {
    let donor = semantic_humanoid_rig(100, true);
    let target = semantic_humanoid_rig(200, false);
    let override_mapping = HumanoidSemanticOverrideV2 {
        semantic: HumanoidBoneSemanticV2::RightHand,
        donor_node_id: 108,
        target_node_id: 208,
    };
    let pending = inspect_humanoid_retarget_compatibility_v2(
        &target,
        &donor,
        std::slice::from_ref(&override_mapping),
        false,
    );
    assert_eq!(
        pending.status,
        HumanoidRetargetCompatibilityStatusV2::ManualConfirmationRequired
    );
    let approved =
        inspect_humanoid_retarget_compatibility_v2(&target, &donor, &[override_mapping], true);
    assert_eq!(
        approved.status,
        HumanoidRetargetCompatibilityStatusV2::Compatible
    );
}

#[test]
fn retarget_applies_rest_delta_root_scale_and_target_node_ids_deterministically() {
    let donor = rig([1, 2, 3], 1.0, false);
    let target = rig([10, 20, 30], 2.0, true);
    let clip = attack_clip([1, 2, 3]);

    let first = retarget_animation_clip_same_hierarchy_v1(
        &clip,
        &donor,
        &target,
        "fogbound-slash",
        "fogslash",
    )
    .unwrap();
    let second = retarget_animation_clip_same_hierarchy_v1(
        &clip,
        &donor,
        &target,
        "fogbound-slash",
        "fogslash",
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.required_document_schema_version, 3);
    assert_eq!(
        first.clip.source.kind,
        AuthoredAnimationSourceKindV1::RetargetedModelCopy
    );
    assert!(
        first
            .clip
            .tracks
            .iter()
            .all(|track| [10, 20, 30].contains(&track.target_node_id))
    );

    let hips = first
        .clip
        .tracks
        .iter()
        .find(|track| {
            track.target_node_id == 10 && track.path == AuthoredAnimationTrackPathV1::Translation
        })
        .unwrap();
    assert_eq!(hips.keyframes[0].value, vec![0.0, 2.0, 0.0]);
    assert_eq!(hips.keyframes[1].value, vec![2.0, 2.0, 0.0]);

    let spine = first
        .clip
        .tracks
        .iter()
        .find(|track| track.target_node_id == 20)
        .unwrap();
    assert!(
        spine
            .keyframes
            .iter()
            .all(|key| key.value == vec![0.0, 2.0, 0.0])
    );

    let hand = first
        .clip
        .tracks
        .iter()
        .find(|track| track.target_node_id == 30)
        .unwrap();
    let expected = [0.5_f32, 0.5, -0.5, 0.5];
    assert!(
        hand.keyframes[1]
            .value
            .iter()
            .zip(expected)
            .all(|(actual, expected)| { (*actual - expected).abs() <= 1.0e-5 })
    );
    let provenance = first.clip.source.retarget.as_ref().unwrap();
    assert_eq!(provenance.root_motion_scale, 2.0);
    assert_eq!(provenance.mode, "SAME_HIERARCHY_RETARGET_V1");
    assert_eq!(provenance.target_source_revision, target.source_revision);
}

#[test]
fn retargeted_copy_requires_and_validates_document_v3() {
    let donor = rig([1, 2, 3], 1.0, false);
    let target = rig([10, 20, 30], 2.0, true);
    let result = retarget_animation_clip_same_hierarchy_v1(
        &attack_clip([1, 2, 3]),
        &donor,
        &target,
        "fogbound-slash",
        "fogslash",
    )
    .unwrap();
    let v2 = AnimationStudioDocumentV1 {
        schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V2,
        source_revision: target.source_revision.clone(),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Draft,
        authored_clips: vec![result.clip],
    };
    assert!(
        validate_animation_studio_schema_v1(&v2)
            .iter()
            .any(|diagnostic| { diagnostic.message.contains("schemaVersion 3") })
    );
    let v3 = migrate_animation_studio_document_v1_or_v2_to_v3(&v2);
    assert_eq!(
        v3.schema_version,
        ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3
    );
    let diagnostics = validate_animation_studio_schema_v1(&v3);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
#[ignore = "requires canonical local Fogbound and Void Crystal Knight GLBs"]
fn exact_fogbound_slash_retargets_to_void_crystal_knight_through_product_core() {
    let repo = canonical_workspace::canonical_repository_root();
    let target = fs::read(repo.join("sample-3d/void-crystal-knight-h1-v1/source.glb"))
        .expect("canonical Void Crystal Knight source");
    let donor = fs::read(
        repo.join("sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb"),
    )
    .expect("canonical Fogbound continuous-animation source");
    let result = copy_animation_clip_between_models_v1(
        &target,
        &donor,
        "ca1slashl",
        &AnimationTransferOptionsV1 {
            mode: AnimationTransferModeV1::SameHierarchyRetargetV1,
            clip_id: "fogbound-ca1slashl-v1".to_owned(),
            output_name: "fogslash".to_owned(),
        },
    )
    .unwrap();

    assert_eq!(
        result.compatibility.status,
        AnimationTransferCompatibilityStatusV1::RetargetableSameHierarchy
    );
    assert_eq!(result.clip.tracks.len(), 48);
    assert_eq!(
        result
            .compatibility
            .diagnostics
            .iter()
            .filter(|diagnostic| { diagnostic.code == "M2A-ANIMATION-RETARGET-REST-TRANSLATION" })
            .count(),
        24
    );
    assert_eq!(
        result
            .compatibility
            .diagnostics
            .iter()
            .filter(|diagnostic| { diagnostic.code == "M2A-ANIMATION-RETARGET-REST-ROTATION" })
            .count(),
        23
    );
    assert_eq!(
        result.clip.source.kind,
        AuthoredAnimationSourceKindV1::RetargetedModelCopy
    );
    assert_eq!(
        result.clip.source.source_revision,
        "ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af"
    );
    let provenance = result.clip.source.retarget.as_ref().unwrap();
    assert_eq!(
        provenance.target_source_revision,
        "d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7"
    );
    assert!(provenance.root_motion_scale.is_finite());
    assert!(provenance.root_motion_scale > 0.0);

    let target_inspection =
        m2a_core::model_pipeline::inspect_editable_animation_source_v1(&target).unwrap();
    let mut authored_clip = result.clip.clone();
    authored_clip.status = AuthoredAnimationClipStatusV1::Valid;
    let studio = AnimationStudioDocumentV1 {
        schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3,
        source_revision: target_inspection.source_revision.clone(),
        authoring_revision: 2,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![authored_clip.clone()],
    };
    let custom_id = "fogbound-ca1slashl-custom".to_owned();
    let custom_provenance = AnimationMappingProvenanceV1 {
        provider: AnimationProviderV1::UserCustom,
        asset_id: authored_clip.id.clone(),
        ownership: AnimationOwnershipV1::UserOwned,
    };
    let authoring = CreatureAnimationAuthoringV2 {
        schema_version: 2,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: target_inspection.source_revision.clone(),
        authoring_revision: 2,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                source_kind: AnimationSourceKindV1::Custom,
                source_clip_name: None,
                custom_animation_id: Some(custom_id.clone()),
                provenance: custom_provenance.clone(),
            })
            .collect(),
        fallbacks: vec![],
        custom_animations: vec![CustomAnimationDefinitionV2 {
            id: custom_id,
            name: "fogslash".to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            clip_reference: Some(CustomAnimationClipReferenceV2 {
                source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
                source_clip_name: None,
                authored_clip_id: Some(authored_clip.id.clone()),
            }),
            phases: vec![],
            provenance: custom_provenance,
        }],
    };
    let materialized = materialize_creature_animation_authoring_v2(
        &target_inspection.animations,
        None,
        &authoring,
        &studio,
        &target_inspection.rig,
    )
    .unwrap();
    assert!(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().all(|name| {
        materialized
            .animations
            .clips
            .iter()
            .any(|clip| clip.name == *name)
    }));
    assert!(
        materialized
            .animations
            .clips
            .iter()
            .any(|clip| { clip.name == "fogslash" && clip.tracks.len() == 48 })
    );
    assert_eq!(materialized.custom_runtime_exposures.len(), 1);
    assert_eq!(
        materialized.custom_runtime_exposures[0]
            .runtime_base_slots
            .len(),
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len(),
    );
}

#[test]
fn batch_v2_prepares_nine_clips_and_rejects_one_failure_atomically() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let inspection =
        m2a_core::model_pipeline::inspect_editable_animation_source_v1(&source).unwrap();
    let names = inspection
        .animations
        .clips
        .iter()
        .filter(|clip| {
            clip.tracks
                .iter()
                .any(|track| track.values.windows(2).any(|pair| pair[0] != pair[1]))
        })
        .take(9)
        .map(|clip| clip.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 9);
    let request = AnimationTransferBatchRequestV2 {
        schema_version: 2,
        commit: AnimationTransferBatchCommitV1::AllOrNothing,
        donor_source_revision: inspection.source_revision.clone(),
        target_source_revision: inspection.source_revision.clone(),
        clips: names
            .iter()
            .map(|name| AnimationTransferBatchClipRequestV1 {
                donor_clip_name: name.clone(),
                mode: AnimationTransferModeV1::ExactRigCopyV1,
                clip_id: format!("batch-{name}"),
                output_name: format!("copy_{name}"),
            })
            .collect(),
        semantic_map: None,
    };
    let before = source.clone();
    let result = prepare_animation_transfer_batch_v2(&source, &source, &request).unwrap();
    assert_eq!(result.clips.len(), 9);
    assert!(result.clips.iter().all(|clip| clip.status == "READY"));
    assert_eq!(
        source, before,
        "batch preparation must not mutate source bytes"
    );

    let mut rejected = request;
    rejected.clips[4].donor_clip_name = "missing_clip".to_owned();
    let error = prepare_animation_transfer_batch_v2(&source, &source, &rejected).unwrap_err();
    assert!(error.path.starts_with("batch.clips[4]"));
}

#[test]
#[ignore = "requires canonical local Fogbound and Void Crystal Knight GLBs"]
fn exact_every_fogbound_clip_retargets_and_materializes_on_void_crystal_knight() {
    let repo = canonical_workspace::canonical_repository_root();
    let target = fs::read(repo.join("sample-3d/void-crystal-knight-h1-v1/source.glb"))
        .expect("canonical Void Crystal Knight source");
    let donor = fs::read(
        repo.join("sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb"),
    )
    .expect("canonical Fogbound continuous-animation source");
    let target_inspection =
        m2a_core::model_pipeline::inspect_editable_animation_source_v1(&target).unwrap();
    let donor_inspection =
        m2a_core::model_pipeline::inspect_editable_animation_source_v1(&donor).unwrap();

    assert_eq!(
        target_inspection.source_revision,
        "d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7"
    );
    assert_eq!(
        donor_inspection.source_revision,
        "ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af"
    );

    let expected_clip_names = [
        "ca1slashl",
        "ca1slashr",
        "cdead",
        "cdamagel",
        "cguptokdb",
        "cpause1",
        "crun",
        "ctaunt",
        "cwalk",
    ];
    let actual_clip_names = donor_inspection
        .animations
        .clips
        .iter()
        .map(|clip| clip.name.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_clip_names,
        expected_clip_names.into_iter().collect::<BTreeSet<_>>()
    );

    let batch_request = AnimationTransferBatchRequestV2 {
        schema_version: 2,
        commit: AnimationTransferBatchCommitV1::AllOrNothing,
        donor_source_revision: donor_inspection.source_revision.clone(),
        target_source_revision: target_inspection.source_revision.clone(),
        clips: expected_clip_names
            .iter()
            .map(|source_name| AnimationTransferBatchClipRequestV1 {
                donor_clip_name: (*source_name).to_owned(),
                mode: AnimationTransferModeV1::SameHierarchyRetargetV1,
                clip_id: format!("batch-{source_name}-v2"),
                output_name: format!("fog_{source_name}"),
            })
            .collect(),
        semantic_map: None,
    };
    let batch = prepare_animation_transfer_batch_v2(&target, &donor, &batch_request)
        .expect("the exact nine-clip batch must prepare atomically");
    assert_eq!(batch.clips.len(), 9);
    assert!(batch.clips.iter().all(|clip| clip.status == "READY"));
    assert_eq!(batch.batch_fingerprint_sha256.len(), 64);

    let mut rejected_request = batch_request.clone();
    rejected_request.clips[8].donor_clip_name = "missing_tenth_clip".to_owned();
    let rejection = prepare_animation_transfer_batch_v2(&target, &donor, &rejected_request)
        .expect_err("one missing clip must reject the entire batch");
    assert!(rejection.path.starts_with("batch.clips[8]"));

    let target_node_ids = target_inspection
        .rig
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<BTreeSet<_>>();
    let mut authored_clips = Vec::with_capacity(expected_clip_names.len());
    let mut output_names = Vec::with_capacity(expected_clip_names.len());
    for source_name in expected_clip_names {
        let output_name = format!("fog_{source_name}");
        let result = copy_animation_clip_between_models_v1(
            &target,
            &donor,
            source_name,
            &AnimationTransferOptionsV1 {
                mode: AnimationTransferModeV1::SameHierarchyRetargetV1,
                clip_id: format!("fogbound-{source_name}-retarget-v1"),
                output_name: output_name.clone(),
            },
        )
        .unwrap_or_else(|error| panic!("{source_name} must retarget: {error:#?}"));

        assert_eq!(
            result.compatibility.status,
            AnimationTransferCompatibilityStatusV1::RetargetableSameHierarchy,
            "{source_name} compatibility"
        );
        assert_eq!(
            result.compatibility.mapping.as_ref().unwrap().entries.len(),
            24,
            "{source_name} mapping"
        );
        assert_eq!(result.clip.name, output_name, "{source_name} output name");
        assert_eq!(result.clip.tracks.len(), 48, "{source_name} track count");
        assert!(
            result
                .clip
                .tracks
                .iter()
                .all(|track| target_node_ids.contains(&track.target_node_id)),
            "{source_name} must use only target node IDs"
        );
        assert!(
            result
                .clip
                .tracks
                .iter()
                .flat_map(|track| &track.keyframes)
                .all(|keyframe| keyframe.time_seconds.is_finite()
                    && keyframe.value.iter().all(|value| value.is_finite())),
            "{source_name} must contain only finite motion"
        );
        let provenance = result.clip.source.retarget.as_ref().unwrap();
        assert_eq!(provenance.donor_clip_name, source_name);
        assert_eq!(
            provenance.target_source_revision,
            target_inspection.source_revision
        );
        assert_eq!(provenance.mode, "SAME_HIERARCHY_RETARGET_V1");
        assert!(provenance.root_motion_scale.is_finite());
        assert!(provenance.root_motion_scale > 0.0);

        let mut authored_clip = result.clip;
        authored_clip.status = AuthoredAnimationClipStatusV1::Valid;
        authored_clips.push(authored_clip);
        output_names.push(output_name);
    }

    let studio = AnimationStudioDocumentV1 {
        schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3,
        source_revision: target_inspection.source_revision.clone(),
        authoring_revision: 3,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips,
    };
    let diagnostics = validate_animation_studio_schema_v1(&studio);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    let custom_ids = expected_clip_names
        .iter()
        .map(|source_name| format!("fogbound-{source_name}-custom"))
        .collect::<Vec<_>>();
    let assignments = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .map(|slot| {
            let source_index = expected_clip_names
                .iter()
                .position(|source_name| source_name == slot)
                .unwrap_or(0);
            let authored_clip = &studio.authored_clips[source_index];
            AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                source_kind: AnimationSourceKindV1::Custom,
                source_clip_name: None,
                custom_animation_id: Some(custom_ids[source_index].clone()),
                provenance: AnimationMappingProvenanceV1 {
                    provider: AnimationProviderV1::UserCustom,
                    asset_id: authored_clip.id.clone(),
                    ownership: AnimationOwnershipV1::UserOwned,
                },
            }
        })
        .collect();
    let custom_animations = studio
        .authored_clips
        .iter()
        .zip(custom_ids.iter())
        .map(|(authored_clip, custom_id)| CustomAnimationDefinitionV2 {
            id: custom_id.clone(),
            name: authored_clip.name.clone(),
            playback: CustomAnimationPlaybackV1::OneShot,
            clip_reference: Some(CustomAnimationClipReferenceV2 {
                source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
                source_clip_name: None,
                authored_clip_id: Some(authored_clip.id.clone()),
            }),
            phases: vec![],
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: authored_clip.id.clone(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        })
        .collect();
    let authoring = CreatureAnimationAuthoringV2 {
        schema_version: 2,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: target_inspection.source_revision.clone(),
        authoring_revision: 3,
        assignments,
        fallbacks: vec![],
        custom_animations,
    };
    let materialized = materialize_creature_animation_authoring_v2(
        &target_inspection.animations,
        None,
        &authoring,
        &studio,
        &target_inspection.rig,
    )
    .unwrap();

    assert!(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().all(|name| {
        materialized
            .animations
            .clips
            .iter()
            .any(|clip| clip.name == *name)
    }));
    assert!(output_names.iter().all(|name| {
        materialized
            .animations
            .clips
            .iter()
            .any(|clip| clip.name == *name && clip.tracks.len() == 48)
    }));
    assert_eq!(
        materialized.custom_runtime_exposures.len(),
        expected_clip_names.len()
    );
}
