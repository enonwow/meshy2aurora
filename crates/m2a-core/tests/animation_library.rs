use m2a_core::{
    animation_library::{
        AnimationLibraryCatalogSourceV1, AnimationLibraryValidationStatusV1,
        AnimationPresetAuthorV1, AnimationPresetKeyframeV1, AnimationPresetManifestV1,
        AnimationPresetPayloadV1, AnimationPresetPlaybackV1, AnimationPresetSourceV1,
        AnimationPresetTrackV1, AnimationPresetV1, AnimationRigCompatibilityStatusV1,
        AnimationRigProfileNodeV1, AnimationRigProfileV1, ExportAnimationContributionMetadataV1,
        animation_preset_motion_sha256_v1, animation_rig_signature_v1,
        build_community_animation_catalog_from_root_v1, build_community_animation_catalog_v1,
        canonical_animation_preset_json_v1, export_animation_contribution_v1,
        inspect_animation_preset_compatibility_v1, install_animation_contribution_v1,
        instantiate_animation_preset_v1, parse_animation_preset_assets_v1,
        parse_animation_preset_v1, sha256_hex_v1, validate_animation_contribution_v1,
        validate_animation_preset_v1,
    },
    animation_studio::{
        ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION, ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V2,
        AnimationStudioDocumentStatusV1, AnimationStudioDocumentV1, AnimationStudioRigNodeV1,
        AnimationStudioRigV1, AuthoredAnimationClipStatusV1, AuthoredAnimationSourceKindV1,
        AuthoredAnimationTrackPathV1, migrate_animation_studio_document_v1_to_v2,
        validate_animation_studio_schema_v1, validate_authored_animation_clip_v1,
    },
    mdl::MdlAnimationInterpolationV1,
    model_pipeline::inspect_editable_animation_source_v1,
    owned_fixture::synthetic_owned_animation_library_full_native_42_glb_v1,
};

fn rig(offset: u32) -> AnimationStudioRigV1 {
    let rows = [
        ("Hips", None, [0.0, 0.0, 0.0]),
        ("Spine", Some("Hips"), [0.0, 0.2, 0.0]),
        ("Spine01", Some("Spine"), [0.0, 0.2, 0.0]),
        ("Spine02", Some("Spine01"), [0.0, 0.2, 0.0]),
        ("RightShoulder", Some("Spine02"), [0.1, 0.1, 0.0]),
        ("RightArm", Some("RightShoulder"), [0.2, 0.0, 0.0]),
        ("RightForeArm", Some("RightArm"), [0.3, 0.0, 0.0]),
        ("RightHand", Some("RightForeArm"), [0.25, 0.0, 0.0]),
        ("LeftShoulder", Some("Spine02"), [-0.1, 0.1, 0.0]),
        ("LeftArm", Some("LeftShoulder"), [-0.2, 0.0, 0.0]),
        ("LeftForeArm", Some("LeftArm"), [-0.3, 0.0, 0.0]),
        ("LeftHand", Some("LeftForeArm"), [-0.25, 0.0, 0.0]),
    ];
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: "a".repeat(64),
        animation_root: "Hips".to_owned(),
        nodes: rows
            .iter()
            .enumerate()
            .map(
                |(index, (name, parent, translation))| AnimationStudioRigNodeV1 {
                    node_id: offset + index as u32 * 7,
                    name: (*name).to_owned(),
                    parent_id: parent.and_then(|parent_name| {
                        rows.iter()
                            .position(|(candidate, _, _)| *candidate == parent_name)
                            .map(|parent_index| offset + parent_index as u32 * 7)
                    }),
                    translation: *translation,
                    rotation: [0.0, 0.0, 0.0, 1.0],
                },
            )
            .collect(),
    }
}

fn payload() -> AnimationPresetPayloadV1 {
    AnimationPresetPayloadV1 {
        schema_version: 1,
        duration_seconds: 1.0,
        animation_root_bone_name: "Hips".to_owned(),
        tracks: vec![AnimationPresetTrackV1 {
            target_bone_name: "RightArm".to_owned(),
            path: AuthoredAnimationTrackPathV1::Rotation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: vec![
                AnimationPresetKeyframeV1 {
                    time_seconds: 0.0,
                    value: vec![0.0, 0.0, 0.0, 1.0],
                },
                AnimationPresetKeyframeV1 {
                    time_seconds: 0.5,
                    value: vec![0.0, 0.25881904, 0.0, 0.9659258],
                },
                AnimationPresetKeyframeV1 {
                    time_seconds: 1.0,
                    value: vec![0.0, 0.0, 0.0, 1.0],
                },
            ],
        }],
        events: Vec::new(),
    }
}

fn preset() -> AnimationPresetV1 {
    let rig = rig(3);
    let animation = payload();
    let animation_json = serde_json::to_string(&animation).unwrap();
    let motion_sha256 = animation_preset_motion_sha256_v1(&animation).unwrap();
    AnimationPresetV1 {
        manifest: AnimationPresetManifestV1 {
            schema_version: 1,
            preset_id: "m2a_right_cross".to_owned(),
            preset_version: 1,
            output_name: "m2a_rightcross".to_owned(),
            label: "Right cross".to_owned(),
            summary: "Compact guard, straight right and recoil.".to_owned(),
            source: AnimationPresetSourceV1::BuiltIn,
            authors: vec![AnimationPresetAuthorV1 {
                name: "Meshy2Aurora contributors".to_owned(),
            }],
            license: "LicenseRef-Meshy2Aurora-Project-Generated".to_owned(),
            tags: vec![
                "attack".to_owned(),
                "boxing".to_owned(),
                "one-shot".to_owned(),
            ],
            playback: AnimationPresetPlaybackV1::OneShot,
            duration_seconds: 1.0,
            rig_profile: "M2A_HUMANOID_STRICT_V1".to_owned(),
            rig_signature_sha256: animation_rig_signature_v1(&rig).unwrap(),
            required_bones: vec!["Hips".to_owned(), "RightArm".to_owned()],
            animation_path: "animation.json".to_owned(),
            animation_byte_length: animation_json.len() as u64,
            animation_sha256: sha256_hex_v1(animation_json.as_bytes()),
            motion_sha256,
            preview_path: None,
            preview_byte_length: None,
            preview_sha256: None,
            validation_status: AnimationLibraryValidationStatusV1::PipelineVerified,
        },
        animation,
        catalog_sha256: "b".repeat(64),
    }
}

fn tags() -> Vec<String> {
    ["attack", "boxing", "one-shot", "unarmed"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn preset_json_is_strict_canonical_and_portable_payload_has_no_editor_ids() {
    let preset = preset();
    assert!(validate_animation_preset_v1(&preset, &tags()).is_empty());
    let canonical = canonical_animation_preset_json_v1(&preset).unwrap();
    assert_eq!(
        canonical,
        canonical_animation_preset_json_v1(&preset).unwrap()
    );
    assert_eq!(parse_animation_preset_v1(&canonical).unwrap(), preset);

    assert!(
        !serde_json::to_string(&preset.animation)
            .unwrap()
            .contains("targetNodeId")
    );
    assert!(
        !serde_json::to_string(&preset.animation)
            .unwrap()
            .contains("\"id\"")
    );
    let unknown = canonical.replacen("{", "{\"unknown\":true,", 1);
    assert!(parse_animation_preset_v1(&unknown).is_err());
}

#[test]
fn raw_asset_hash_unknown_tag_nonunit_quaternion_and_stale_motion_fail_closed() {
    let preset = preset();
    let manifest = serde_json::to_string(&preset.manifest).unwrap();
    let animation = serde_json::to_string(&preset.animation).unwrap();
    assert!(parse_animation_preset_assets_v1(&manifest, &animation, &"b".repeat(64)).is_ok());
    assert!(
        parse_animation_preset_assets_v1(&manifest, &format!("{animation} "), &"b".repeat(64),)
            .is_err()
    );

    let mut unknown_tag = preset.clone();
    unknown_tag.manifest.tags = vec!["not-in-dictionary".to_owned()];
    assert!(
        validate_animation_preset_v1(&unknown_tag, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-TAG")
    );

    let mut nonunit = preset.clone();
    nonunit.animation.tracks[0].keyframes[1].value = vec![0.0, 0.0, 0.0, 2.0];
    assert!(
        validate_animation_preset_v1(&nonunit, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-VALUE")
    );

    let mut changed_motion = preset.clone();
    changed_motion.animation.tracks[0].keyframes[1].value = vec![0.0, 0.38268343, 0.0, 0.9238795];
    assert_ne!(
        animation_preset_motion_sha256_v1(&preset.animation).unwrap(),
        animation_preset_motion_sha256_v1(&changed_motion.animation).unwrap(),
    );
    assert!(
        validate_animation_preset_v1(&changed_motion, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-MOTION-HASH")
    );
}

#[test]
fn required_bones_must_exactly_match_the_animation_root_and_track_targets() {
    let mut missing_target = preset();
    missing_target.manifest.required_bones = vec!["Hips".to_owned()];
    assert!(
        validate_animation_preset_v1(&missing_target, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-RIG-BONES-MISMATCH")
    );

    let mut undeclared_extra = preset();
    undeclared_extra
        .manifest
        .required_bones
        .push("Spine".to_owned());
    undeclared_extra.manifest.required_bones.sort();
    assert!(
        validate_animation_preset_v1(&undeclared_extra, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-RIG-BONES-MISMATCH")
    );
}

#[test]
fn rig_signature_ignores_numeric_ids_but_rejects_parent_and_rest_changes() {
    let original = rig(3);
    let renumbered = rig(900);
    assert_eq!(
        animation_rig_signature_v1(&original).unwrap(),
        animation_rig_signature_v1(&renumbered).unwrap(),
    );
    let preset = preset();
    assert_eq!(
        inspect_animation_preset_compatibility_v1(&preset, &renumbered).status,
        AnimationRigCompatibilityStatusV1::Compatible,
    );

    let mut representation_round_trip = renumbered.clone();
    representation_round_trip
        .nodes
        .iter_mut()
        .find(|node| node.name == "RightArm")
        .unwrap()
        .translation[0] += 2.0e-8;
    assert_eq!(
        animation_rig_signature_v1(&original).unwrap(),
        animation_rig_signature_v1(&representation_round_trip).unwrap(),
        "sub-micro-unit f32 decomposition noise must not change strict semantic rig identity",
    );

    let mut wrong_parent = renumbered.clone();
    let right_arm = wrong_parent
        .nodes
        .iter()
        .position(|node| node.name == "RightArm")
        .unwrap();
    wrong_parent.nodes[right_arm].parent_id = Some(
        wrong_parent
            .nodes
            .iter()
            .find(|node| node.name == "Hips")
            .unwrap()
            .node_id,
    );
    assert_eq!(
        inspect_animation_preset_compatibility_v1(&preset, &wrong_parent).status,
        AnimationRigCompatibilityStatusV1::Incompatible,
    );

    let mut wrong_rest = renumbered;
    wrong_rest.nodes[right_arm].translation[0] += 0.01;
    assert_eq!(
        inspect_animation_preset_compatibility_v1(&preset, &wrong_rest).status,
        AnimationRigCompatibilityStatusV1::Incompatible,
    );
}

#[test]
fn instantiate_maps_bone_names_and_records_complete_library_provenance() {
    let rig = rig(900);
    let source_revision = rig.source_revision.clone();
    let mut clip = instantiate_animation_preset_v1(
        &preset(),
        &rig,
        &source_revision,
        "clip-local-1",
        "m2a_rc_copy",
    )
    .unwrap();
    assert_eq!(clip.status, AuthoredAnimationClipStatusV1::Draft);
    assert_eq!(
        clip.source.kind,
        AuthoredAnimationSourceKindV1::LibraryPresetCopy
    );
    assert_eq!(clip.source.source_revision, source_revision);
    assert_eq!(
        clip.tracks[0].target_node_id,
        rig.nodes
            .iter()
            .find(|node| node.name == "RightArm")
            .unwrap()
            .node_id,
    );
    let provenance = clip.source.library_preset.as_ref().unwrap();
    assert_eq!(provenance.preset_id, "m2a_right_cross");
    assert_eq!(provenance.preset_version, 1);
    assert_eq!(provenance.catalog_sha256, "b".repeat(64));
    assert_eq!(provenance.instantiation_mode, "STRICT_RIG_V1");

    let legacy_document = AnimationStudioDocumentV1 {
        schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION,
        source_revision: source_revision.clone(),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Draft,
        authored_clips: vec![clip.clone()],
    };
    assert!(
        validate_animation_studio_schema_v1(&legacy_document)
            .iter()
            .any(|diagnostic| diagnostic.path == "authoredClips[0].source.kind")
    );
    let migrated = migrate_animation_studio_document_v1_to_v2(&legacy_document);
    assert_eq!(
        migrated.schema_version,
        ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V2
    );
    assert!(
        !validate_animation_studio_schema_v1(&migrated)
            .iter()
            .any(|diagnostic| diagnostic.path == "authoredClips[0].source.kind")
    );

    clip.source.source_clip_fingerprint = Some("c".repeat(64));
    assert!(
        validate_authored_animation_clip_v1(&clip, &rig)
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with(".source"))
    );
}

#[test]
fn contribution_export_is_portable_deterministic_and_contains_no_source_glb_identity() {
    let rig = rig(3);
    let mut clip = instantiate_animation_preset_v1(
        &preset(),
        &rig,
        &rig.source_revision,
        "clip-local-2",
        "m2a_rc_copy",
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    let metadata = ExportAnimationContributionMetadataV1 {
        preset_id: "community_right_cross".to_owned(),
        preset_version: 1,
        output_name: "comm_rightcross".to_owned(),
        label: "Community right cross".to_owned(),
        summary: "A portable boxing right cross.".to_owned(),
        authors: vec![AnimationPresetAuthorV1 {
            name: "Test Author".to_owned(),
        }],
        license: "CC0-1.0".to_owned(),
        tags: vec![
            "attack".to_owned(),
            "boxing".to_owned(),
            "one-shot".to_owned(),
        ],
        playback: AnimationPresetPlaybackV1::OneShot,
        rig_profile: "M2A_HUMANOID_STRICT_V1".to_owned(),
        validation_status: AnimationLibraryValidationStatusV1::PipelineVerified,
    };
    let first = export_animation_contribution_v1(&clip, &rig, metadata.clone()).unwrap();
    let second = export_animation_contribution_v1(&clip, &rig, metadata).unwrap();
    assert_eq!(first, second);
    let json = serde_json::to_string(&first).unwrap();
    assert!(!json.contains(&rig.source_revision));
    assert!(!json.to_ascii_lowercase().contains(".glb"));
    assert!(!json.contains("targetNodeId"));
    assert!(json.contains("targetBoneName"));

    let temporary = std::env::temp_dir().join(format!(
        "m2a-animation-contribution-install-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temporary);
    std::fs::create_dir_all(temporary.join("presets")).unwrap();
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    std::fs::copy(
        repository_root.join("animation-library/tags-v1.json"),
        temporary.join("tags-v1.json"),
    )
    .unwrap();
    let installed = install_animation_contribution_v1(&temporary, &first).unwrap();
    assert!(installed.ends_with("community_right_cross"));
    assert!(installed.join("manifest.json").is_file());
    assert!(
        install_animation_contribution_v1(&temporary, &first)
            .unwrap_err()
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "M2A-ANIMATION-LIBRARY-CONTRIBUTION-IMMUTABLE"
            })
    );
    let mut version_two = first.clone();
    version_two.preset_version = 2;
    let installed_v2 = install_animation_contribution_v1(&temporary, &version_two).unwrap();
    assert!(installed_v2.ends_with("community_right_cross-v2"));
    let (catalog, _) = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::RepositoryCheck,
    )
    .unwrap();
    assert_eq!(catalog.entries.len(), 2);
    assert!(
        catalog
            .entries
            .iter()
            .all(|entry| entry.source == AnimationPresetSourceV1::Community)
    );
    std::fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn contribution_cannot_self_assert_owner_nwn_verification() {
    let rig = rig(3);
    let mut clip = instantiate_animation_preset_v1(
        &preset(),
        &rig,
        &rig.source_revision,
        "clip-owner-claim",
        "m2a_ownerclaim",
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    let mut contribution = export_animation_contribution_v1(
        &clip,
        &rig,
        ExportAnimationContributionMetadataV1 {
            preset_id: "community_owner_claim".to_owned(),
            preset_version: 1,
            output_name: "comm_ownerclaim".to_owned(),
            label: "Unverified owner claim".to_owned(),
            summary: "A contribution must not promote its own runtime proof status.".to_owned(),
            authors: vec![AnimationPresetAuthorV1 {
                name: "Test Author".to_owned(),
            }],
            license: "CC0-1.0".to_owned(),
            tags: vec![
                "attack".to_owned(),
                "boxing".to_owned(),
                "one-shot".to_owned(),
            ],
            playback: AnimationPresetPlaybackV1::OneShot,
            rig_profile: "M2A_HUMANOID_STRICT_V1".to_owned(),
            validation_status: AnimationLibraryValidationStatusV1::PipelineVerified,
        },
    )
    .unwrap();
    contribution.validation_status = AnimationLibraryValidationStatusV1::OwnerNwnVerified;
    assert!(
        validate_animation_contribution_v1(&contribution)
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-OWNER-PROOF-CLAIM")
    );
}

#[test]
fn catalog_build_is_sorted_deterministic_and_detects_duplicate_identity() {
    let first = preset();
    let mut second = preset();
    second.manifest.preset_id = "m2a_left_jab".to_owned();
    second.manifest.output_name = "m2a_leftjab".to_owned();
    second.manifest.label = "Left jab".to_owned();
    let a = build_community_animation_catalog_v1(
        &[first.clone(), second.clone()],
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap();
    let b = build_community_animation_catalog_v1(
        &[second, first.clone()],
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap();
    assert_eq!(a, b);
    assert_eq!(a.entries[0].preset_id, "m2a_left_jab");
    assert_eq!(a.entries[1].preset_id, "m2a_right_cross");
    assert!(
        build_community_animation_catalog_v1(
            &[first.clone(), first],
            AnimationLibraryCatalogSourceV1::EmbeddedRelease,
        )
        .is_err()
    );
}

#[test]
fn catalog_root_requires_canonical_versioned_directory_and_readme() {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let source = repository_root.join("animation-library/presets/m2a_right_cross");
    let temporary = std::env::temp_dir().join(format!(
        "m2a-animation-library-directory-contract-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temporary);
    std::fs::create_dir_all(temporary.join("presets/wrong-directory")).unwrap();
    std::fs::copy(
        repository_root.join("animation-library/tags-v1.json"),
        temporary.join("tags-v1.json"),
    )
    .unwrap();
    for name in [
        "manifest.json",
        "animation.json",
        "README.md",
        "preview.webp",
    ] {
        std::fs::copy(
            source.join(name),
            temporary.join("presets/wrong-directory").join(name),
        )
        .unwrap();
    }
    let wrong_directory = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::RepositoryCheck,
    )
    .unwrap_err();
    assert!(
        wrong_directory
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-PRESET-PATH")
    );

    std::fs::remove_dir_all(temporary.join("presets/wrong-directory")).unwrap();
    std::fs::create_dir_all(temporary.join("presets/m2a_right_cross")).unwrap();
    for name in ["manifest.json", "animation.json", "preview.webp"] {
        std::fs::copy(
            source.join(name),
            temporary.join("presets/m2a_right_cross").join(name),
        )
        .unwrap();
    }
    let missing_readme = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::RepositoryCheck,
    )
    .unwrap_err();
    assert!(
        missing_readme
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-FILE-MISSING")
    );
    std::fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn catalog_tag_dictionary_must_match_the_stable_core_v1_vocabulary() {
    let temporary = std::env::temp_dir().join(format!(
        "m2a-animation-library-tags-contract-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temporary);
    std::fs::create_dir_all(temporary.join("presets")).unwrap();
    std::fs::write(
        temporary.join("tags-v1.json"),
        r#"{"schemaVersion":1,"tags":["attack","unversioned-new-tag"]}"#,
    )
    .unwrap();
    let diagnostics = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::RepositoryCheck,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-TAGS")
    );
    std::fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn exported_profile_can_be_reconstructed_without_numeric_node_ids() {
    let rig = rig(70);
    let profile = AnimationRigProfileV1::from_rig(&rig).unwrap();
    assert_eq!(profile.schema_version, 1);
    assert_eq!(profile.nodes[0].name, "Hips");
    assert_eq!(profile.nodes[0].parent_name, None);
    assert_eq!(
        profile.signature_sha256,
        animation_rig_signature_v1(&rig).unwrap()
    );
    let _: &AnimationRigProfileNodeV1 = &profile.nodes[0];
}

#[test]
fn tracked_starter_library_has_seven_distinct_multiphase_core_validated_presets() {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let library_root = repository_root.join("animation-library");
    let (catalog, presets) = build_community_animation_catalog_from_root_v1(
        &library_root,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap();
    assert_eq!(presets.len(), 7);
    assert_eq!(catalog.entries.len(), 7);
    assert_eq!(
        catalog.catalog_sha256,
        "6a0ff531039d7c6d977f95c8df90295c45d7cb510f7a53102e6cf36b1cfadb97",
    );
    let tracked =
        std::fs::read(repository_root.join("contracts/community-animation-catalog-v1.json"))
            .unwrap();
    let mut rebuilt = serde_json::to_vec_pretty(&catalog).unwrap();
    rebuilt.push(b'\n');
    assert_eq!(tracked, rebuilt);
    let motion_hashes = presets
        .iter()
        .map(|preset| preset.manifest.motion_sha256.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(motion_hashes.len(), 7);
    for preset in presets {
        let moments = preset
            .animation
            .tracks
            .iter()
            .flat_map(|track| track.keyframes.iter().map(|key| key.time_seconds.to_bits()))
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            moments.len() >= 5,
            "{} has no distinct motion phases",
            preset.manifest.preset_id
        );
        assert!(preset.animation.tracks.iter().any(|track| {
            track.keyframes.first().is_some_and(|first| {
                track
                    .keyframes
                    .iter()
                    .skip(1)
                    .any(|key| key.value != first.value)
            })
        }));
    }
}

#[test]
fn tracked_right_cross_binds_to_the_owned_exact_profile_fixture() {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let profile: AnimationRigProfileV1 = serde_json::from_slice(
        &std::fs::read(
            repository_root.join("animation-library/rig-profiles/m2a-humanoid-strict-v1.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let source = synthetic_owned_animation_library_full_native_42_glb_v1(&profile).unwrap();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let actual_profile = AnimationRigProfileV1::from_rig(&inspection.rig).unwrap();
    assert_eq!(actual_profile.signature_sha256, profile.signature_sha256);

    let (_, presets) = build_community_animation_catalog_from_root_v1(
        &repository_root.join("animation-library"),
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap();
    let preset = presets
        .iter()
        .find(|preset| preset.manifest.preset_id == "m2a_right_cross")
        .unwrap();
    assert_eq!(
        inspect_animation_preset_compatibility_v1(preset, &inspection.rig).status,
        AnimationRigCompatibilityStatusV1::Compatible
    );
    let clip = instantiate_animation_preset_v1(
        preset,
        &inspection.rig,
        &inspection.source_revision,
        "right-cross-e2e",
        "m2a_rightcross",
    )
    .unwrap();
    assert_eq!(
        clip.source.source_clip_fingerprint,
        Some(preset.manifest.motion_sha256.clone())
    );
    assert_eq!(clip.tracks.len(), preset.animation.tracks.len());
}

#[test]
fn preview_bytes_are_hash_bound_without_changing_the_motion_identity() {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let source = repository_root.join("animation-library/presets/m2a_right_cross");
    let temporary = std::env::temp_dir().join(format!(
        "m2a-animation-library-preview-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temporary);
    let preset = temporary.join("presets/m2a_right_cross");
    std::fs::create_dir_all(&preset).unwrap();
    std::fs::copy(
        repository_root.join("animation-library/tags-v1.json"),
        temporary.join("tags-v1.json"),
    )
    .unwrap();
    for name in [
        "manifest.json",
        "animation.json",
        "preview.webp",
        "README.md",
    ] {
        std::fs::copy(source.join(name), preset.join(name)).unwrap();
    }

    let (first, first_presets) = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap();
    let motion = first_presets[0].manifest.motion_sha256.clone();
    let mut preview = std::fs::read(preset.join("preview.webp")).unwrap();
    let last = preview.last_mut().unwrap();
    *last ^= 1;
    std::fs::write(preset.join("preview.webp"), &preview).unwrap();

    let stale = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap_err();
    assert!(
        stale
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-ASSET-HASH")
    );

    let mut manifest: AnimationPresetManifestV1 =
        serde_json::from_slice(&std::fs::read(preset.join("manifest.json")).unwrap()).unwrap();
    manifest.preview_byte_length = Some(preview.len() as u64);
    manifest.preview_sha256 = Some(sha256_hex_v1(&preview));
    std::fs::write(
        preset.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let (second, second_presets) = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap();
    assert_eq!(second_presets[0].manifest.motion_sha256, motion);
    assert_ne!(second.catalog_sha256, first.catalog_sha256);

    std::fs::write(preset.join("payload.glb"), b"forbidden model payload").unwrap();
    let forbidden = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap_err();
    assert!(
        forbidden
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-FILE-TYPE")
    );
    std::fs::remove_file(preset.join("payload.glb")).unwrap();

    manifest.animation_path = "../animation.json".to_owned();
    std::fs::write(
        preset.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let traversal = build_community_animation_catalog_from_root_v1(
        &temporary,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .unwrap_err();
    assert!(
        traversal
            .iter()
            .any(|diagnostic| { diagnostic.code == "M2A-ANIMATION-LIBRARY-PATH-TRAVERSAL" })
    );
    std::fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn preset_limits_accept_the_boundary_and_reject_the_first_value_above_it() {
    let mut boundary = preset();
    boundary.animation.tracks = (0..256)
        .map(|index| AnimationPresetTrackV1 {
            target_bone_name: format!("Bone{index:03}"),
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: vec![AnimationPresetKeyframeV1 {
                time_seconds: 0.0,
                value: vec![0.0, 0.0, 0.0],
            }],
        })
        .collect();
    boundary.animation.events = (0..256)
        .map(
            |index| m2a_core::animation_library::AnimationPresetEventV1 {
                time_seconds: 0.0,
                name: format!("e{index}"),
            },
        )
        .collect();
    boundary.manifest.required_bones = boundary
        .animation
        .tracks
        .iter()
        .map(|track| track.target_bone_name.clone())
        .chain(std::iter::once("Hips".to_owned()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    boundary.manifest.motion_sha256 =
        animation_preset_motion_sha256_v1(&boundary.animation).unwrap();
    let at_limit = validate_animation_preset_v1(&boundary, &tags());
    assert!(!at_limit.iter().any(|diagnostic| {
        diagnostic.code == "M2A-ANIMATION-LIBRARY-PAYLOAD"
            || diagnostic.code == "M2A-ANIMATION-LIBRARY-KEYFRAME-LIMIT"
    }));

    let mut too_many_tracks = boundary.clone();
    too_many_tracks
        .animation
        .tracks
        .push(AnimationPresetTrackV1 {
            target_bone_name: "Bone256".to_owned(),
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: vec![AnimationPresetKeyframeV1 {
                time_seconds: 0.0,
                value: vec![0.0, 0.0, 0.0],
            }],
        });
    assert!(
        validate_animation_preset_v1(&too_many_tracks, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-PAYLOAD")
    );

    let mut too_many_keys = preset();
    too_many_keys.animation.tracks[0].keyframes = (0..=10_000)
        .map(|index| AnimationPresetKeyframeV1 {
            time_seconds: index as f32 / 10_000.0,
            value: vec![index as f32 / 10_000.0, 0.0, 0.0],
        })
        .collect();
    too_many_keys.manifest.motion_sha256 =
        animation_preset_motion_sha256_v1(&too_many_keys.animation).unwrap();
    assert!(
        validate_animation_preset_v1(&too_many_keys, &tags())
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-KEYFRAME-LIMIT")
    );

    let mut too_large = preset();
    too_large.manifest.animation_byte_length = 2 * 1024 * 1024 + 1;
    too_large.manifest.preview_path = Some("preview.webp".to_owned());
    too_large.manifest.preview_byte_length = Some(2 * 1024 * 1024 + 1);
    too_large.manifest.preview_sha256 = Some("f".repeat(64));
    let size_errors = validate_animation_preset_v1(&too_large, &tags());
    assert!(
        size_errors
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-ASSET")
    );
    assert!(
        size_errors
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-LIBRARY-PREVIEW")
    );
}
