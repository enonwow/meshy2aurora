use std::fs;

use m2a_core::{
    animation_studio::{
        AnimationStudioDocumentStatusV1, AnimationStudioDocumentV1,
        AnimationStudioReadbackStatusV1, AuthoredAnimationClipInputV1,
        AuthoredAnimationClipStatusV1, AuthoredAnimationEventV1, AuthoredAnimationSourceKindV1,
        CustomAnimationClipReferenceKindV2, CustomAnimationClipReferenceV2,
        CustomAnimationDefinitionV2, ProceduralAnimationTemplateV1,
        apply_custom_attack_demo_route_v1, create_procedural_template_clip_v1,
        evaluate_edited_animation_conformance_v1, materialize_authored_animation_library_v1,
        migrate_creature_animation_authoring_v1_to_v2, reconcile_animation_studio_readback_v1,
    },
    creature_animation_mapping::{
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CreatureAnimationAuthoringV1,
        CustomAnimationPlaybackV1, DirectCreatureBaseSlotV1, DirectCreatureModelTypeV1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    held_weapon::{
        HeldWeaponHandV1, HeldWeaponLocalTransformV1, HeldWeaponPivotPolicyV1,
        compose_held_weapon_attachment_v1, inspect_held_weapon_source_v1,
    },
    inspect_binary_mdl,
    model_pipeline::{
        ProceduralCreaturePackageIdentityV1, ProjectBuildIdentityV1,
        build_meshy_h1_model_package_v4, build_meshy_h1_model_package_v5,
        build_meshy_h1_model_package_v5_with_identity,
        build_meshy_h1_model_package_v6_with_held_weapon_project_identity,
        inspect_editable_animation_source_v1,
        write_animation_studio_v5_demo_packet_with_identity_v1,
    },
    owned_fixture::synthetic_owned_m6_full_native_42_glb_v1,
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureRuntimeProfileV2,
        inspect_binary_creature_profile_matrix_module_v2,
    },
};
use sha2::{Digest, Sha256};

#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod build_synthetic_glb;

#[path = "support/canonical_workspace.rs"]
mod canonical_workspace;

fn appearance_fixture() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n".to_vec()
}

fn source_revision(source: &[u8]) -> String {
    format!("{:x}", Sha256::digest(source))
}

fn v1_authoring(source: &[u8]) -> CreatureAnimationAuthoringV1 {
    CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: source_revision(source),
        authoring_revision: 7,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                source_kind: AnimationSourceKindV1::SourceClip,
                source_clip_name: Some((*slot).to_owned()),
                custom_animation_id: None,
                provenance: AnimationMappingProvenanceV1 {
                    provider: AnimationProviderV1::SourceGlb,
                    asset_id: "owned-full-42-fixture".to_owned(),
                    ownership: AnimationOwnershipV1::UserOwned,
                },
            })
            .collect(),
        fallbacks: vec![],
        custom_animations: vec![],
    }
}

fn procedural_v1_authoring(source: &[u8]) -> CreatureAnimationAuthoringV1 {
    CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: source_revision(source),
        authoring_revision: 8,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                source_kind: AnimationSourceKindV1::Procedural,
                source_clip_name: None,
                custom_animation_id: None,
                provenance: AnimationMappingProvenanceV1 {
                    provider: AnimationProviderV1::ProceduralGenerator,
                    asset_id: "m2a:procedural-humanoid-v1".to_owned(),
                    ownership: AnimationOwnershipV1::ProjectGenerated,
                },
            })
            .collect(),
        fallbacks: vec![],
        custom_animations: vec![],
    }
}

fn empty_studio(source: &[u8]) -> AnimationStudioDocumentV1 {
    AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: source_revision(source),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![],
    }
}

#[test]
fn additive_v5_without_authored_clips_preserves_every_v4_binary() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let appearance = appearance_fixture();
    let v1 = v1_authoring(&source);
    let v2 = migrate_creature_animation_authoring_v1_to_v2(&v1);
    let studio = empty_studio(&source);

    let v4 = build_meshy_h1_model_package_v4(&source, &appearance, &v1).unwrap();
    let first = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap();
    let second = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap();

    assert_eq!(first.model, v4.model);
    assert_eq!(first.texture, v4.texture);
    assert_eq!(first.appearance_two_da, v4.appearance_two_da);
    assert_eq!(first.hak, v4.hak);
    assert_eq!(first.proof_module, v4.proof_module);
    assert_eq!(first.model, second.model);
    assert_eq!(first.manifest_json, second.manifest_json);
    assert_eq!(first.report_json, second.report_json);
    assert!(first.manifest.animation_studio.source_glb_unchanged);
    assert_eq!(first.manifest.animation_studio.authored_clip_count, 0);
    assert_eq!(
        first.animation_studio_readback.status,
        AnimationStudioReadbackStatusV1::Match
    );
}

#[test]
fn held_weapon_v6_is_present_in_final_mdl_and_reconciled_from_hak() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let appearance = appearance_fixture();
    let v2 = migrate_creature_animation_authoring_v1_to_v2(&v1_authoring(&source));
    let studio = empty_studio(&source);
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let target = inspection
        .rig
        .nodes
        .iter()
        .find(|node| node.parent_id.is_some())
        .expect("owned humanoid fixture has an attachable child node");
    let weapon = build_synthetic_glb::axis_hierarchy_asymmetric();
    let weapon_source =
        inspect_held_weapon_source_v1("owned-sword.glb", &weapon, "LOCAL_FILE").unwrap();
    let attachment = compose_held_weapon_attachment_v1(
        weapon_source.clone(),
        &inspection.rig,
        HeldWeaponHandV1::Right,
        target.node_id,
        HeldWeaponLocalTransformV1::default(),
        HeldWeaponPivotPolicyV1::SourceOrigin,
        None,
        None,
        4,
    )
    .unwrap();
    let project = ProjectBuildIdentityV1 {
        schema_version: 1,
        project_id: "held-weapon-v6-test".to_owned(),
        project_name: "Held Weapon V6 Test".to_owned(),
        project_revision: 4,
    };

    let artifact = build_meshy_h1_model_package_v6_with_held_weapon_project_identity(
        &source,
        &appearance,
        &v2,
        &studio,
        None,
        &project,
        &weapon,
        &attachment,
    )
    .unwrap();

    let evidence = artifact.report.base.held_weapon.as_ref().unwrap();
    assert_eq!(evidence.status, "MATCH");
    assert_eq!(evidence.source_sha256, weapon_source.sha256);
    assert!(evidence.attachment_node_present_in_binary_mdl);
    assert!(evidence.combined_triangle_count_match);
    assert!(evidence.texture_payload_match_in_hak);
    assert_eq!(evidence.bake.weapon_triangle_count, 1);
    assert_eq!(
        artifact.report.base.geometry.triangle_count,
        evidence.bake.target_triangle_count_after
    );
    assert!(
        artifact
            .report_json
            .windows(b"\"heldWeapon\"".len())
            .any(|window| window == b"\"heldWeapon\"")
    );
}

#[test]
fn imported_model_copy_materializes_without_reopening_the_donor_glb() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let mut imported = m2a_core::animation_studio::clone_source_clip_for_editing_v1(
        &inspection.animations,
        "cpause1",
        AuthoredAnimationClipInputV1 {
            id: "imported-idle".to_owned(),
            name: "imp_cpause1".to_owned(),
            source_revision: "b".repeat(64),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root.clone(),
        },
    )
    .unwrap();
    imported.source.kind = AuthoredAnimationSourceKindV1::ImportedModelCopy;
    imported.status = AuthoredAnimationClipStatusV1::Valid;
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision.clone(),
        authoring_revision: 2,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![imported],
    };

    let library =
        materialize_authored_animation_library_v1(&inspection.animations, &studio, &inspection.rig)
            .expect("imported tracks are self-contained after the exact rig gate");

    assert_eq!(library.clips.len(), 1);
    assert_eq!(library.clips[0].source.source_revision, "b".repeat(64));
    assert_eq!(
        library.clips[0].source.kind,
        AuthoredAnimationSourceKindV1::ImportedModelCopy
    );
}

#[test]
fn v5_routes_stable_authored_id_to_base_and_custom_outputs_with_events() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let appearance = appearance_fixture();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    assert!(inspection.rig.nodes.iter().all(|node| {
        node.translation.iter().all(|value| value.is_finite())
            && node.rotation.iter().all(|value| value.is_finite())
    }));
    let mut clip = m2a_core::animation_studio::clone_source_clip_for_editing_v1(
        &inspection.animations,
        "cpause1",
        AuthoredAnimationClipInputV1 {
            id: "authored-idle".to_owned(),
            name: "authidle".to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root.clone(),
        },
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    clip.events.push(AuthoredAnimationEventV1 {
        id: "event-impact".to_owned(),
        time_seconds: clip.length_seconds * 0.5,
        name: "impact".to_owned(),
    });
    clip.events.push(AuthoredAnimationEventV1 {
        id: "event-second".to_owned(),
        time_seconds: clip.length_seconds * 0.5,
        name: "second".to_owned(),
    });
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision.clone(),
        authoring_revision: 11,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip],
    };
    let mut v2 = migrate_creature_animation_authoring_v1_to_v2(&v1_authoring(&source));
    v2.custom_animations.push(CustomAnimationDefinitionV2 {
        id: "custom-stable-id".to_owned(),
        name: "custidle".to_owned(),
        playback: CustomAnimationPlaybackV1::OneShot,
        clip_reference: Some(CustomAnimationClipReferenceV2 {
            source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
            source_clip_name: None,
            authored_clip_id: Some("authored-idle".to_owned()),
        }),
        phases: vec![],
        provenance: AnimationMappingProvenanceV1 {
            provider: AnimationProviderV1::UserCustom,
            asset_id: "authored-idle".to_owned(),
            ownership: AnimationOwnershipV1::UserOwned,
        },
    });
    let custom_provenance = v2.custom_animations.last().unwrap().provenance.clone();
    let assignment = v2
        .assignments
        .iter_mut()
        .find(|assignment| assignment.target_slot.as_str() == "cpause1")
        .unwrap();
    assignment.source_kind = AnimationSourceKindV1::Custom;
    assignment.source_clip_name = None;
    assignment.custom_animation_id = Some("custom-stable-id".to_owned());
    assignment.provenance = custom_provenance;

    let artifact = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap();
    let readback = inspect_binary_mdl(&artifact.model).unwrap();
    assert!(
        readback
            .animations
            .iter()
            .any(|clip| clip.name == "cpause1")
    );
    let custom = readback
        .animations
        .iter()
        .find(|clip| clip.name == "custidle")
        .unwrap();
    assert_eq!(
        custom
            .events
            .iter()
            .map(|event| event.name.as_str())
            .collect::<Vec<_>>(),
        ["impact", "second"]
    );
    assert_eq!(artifact.manifest.animation_studio.authored_clip_count, 1);
    assert_eq!(
        artifact.manifest.animation_studio.authored_clip_ids,
        ["authored-idle"]
    );
    assert_eq!(
        artifact.manifest.animation_studio.custom_assignment_count,
        1
    );
    assert_eq!(
        artifact.manifest.animation_studio.custom_runtime_exposures[0].status,
        m2a_core::animation_studio::CustomAnimationRuntimeExposureStatusV1::Base42Routed
    );
    assert_eq!(
        artifact.manifest.animation_studio.custom_runtime_exposures[0].runtime_base_slots,
        ["cpause1"]
    );
    assert_eq!(
        artifact.animation_studio_readback.status,
        AnimationStudioReadbackStatusV1::Match
    );
    assert_eq!(
        artifact
            .manifest
            .animation_studio
            .animation_studio_readback
            .clips
            .iter()
            .map(|clip| (
                clip.authored_clip_id.as_str(),
                clip.output_clip_name.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![("authored-idle", "cpause1"), ("authored-idle", "custidle")]
    );
    assert!(
        artifact.manifest.animation_studio.authored_clips[0]
            .usages
            .iter()
            .any(|usage| usage.output_clip_name == "cpause1")
    );
    assert!(
        artifact.manifest.animation_studio.authored_clips[0]
            .usages
            .iter()
            .any(|usage| usage.output_clip_name == "custidle")
    );

    let mut corrupted = readback.clone();
    corrupted
        .animations
        .iter_mut()
        .find(|clip| clip.name == "custidle")
        .unwrap()
        .length += 0.25;
    let mismatch = reconcile_animation_studio_readback_v1(
        &studio,
        &artifact.materialized_animations,
        &inspection.rig,
        &corrupted,
    );
    assert_eq!(mismatch.status, AnimationStudioReadbackStatusV1::Mismatch);
    assert!(
        mismatch
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-EDIT-READBACK-MISMATCH")
    );
    let writer_gate_mismatch = evaluate_edited_animation_conformance_v1(
        &studio,
        &artifact.materialized_animations,
        &inspection.rig,
        &corrupted,
        &corrupted,
    );
    assert_eq!(
        writer_gate_mismatch.status,
        AnimationStudioReadbackStatusV1::Mismatch
    );
}

#[test]
fn v5_readback_resolves_authored_root_motion_after_runtime_root_rename() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let appearance = appearance_fixture();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let mut clip = create_procedural_template_clip_v1(
        &inspection.rig,
        ProceduralAnimationTemplateV1::RootTranslationPulse,
        AuthoredAnimationClipInputV1 {
            id: "authored-root-pulse".to_owned(),
            name: "rootpulse".to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root.clone(),
        },
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision,
        authoring_revision: 2,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip],
    };
    let mut v2 = migrate_creature_animation_authoring_v1_to_v2(&v1_authoring(&source));
    v2.custom_animations.push(CustomAnimationDefinitionV2 {
        id: "custom-root-pulse".to_owned(),
        name: "custpulse".to_owned(),
        playback: CustomAnimationPlaybackV1::OneShot,
        clip_reference: Some(CustomAnimationClipReferenceV2 {
            source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
            source_clip_name: None,
            authored_clip_id: Some("authored-root-pulse".to_owned()),
        }),
        phases: vec![],
        provenance: AnimationMappingProvenanceV1 {
            provider: AnimationProviderV1::UserCustom,
            asset_id: "authored-root-pulse".to_owned(),
            ownership: AnimationOwnershipV1::UserOwned,
        },
    });
    let assignment = v2
        .assignments
        .iter_mut()
        .find(|assignment| assignment.target_slot.as_str() == "cpause1")
        .unwrap();
    assignment.source_kind = AnimationSourceKindV1::Custom;
    assignment.source_clip_name = None;
    assignment.custom_animation_id = Some("custom-root-pulse".to_owned());
    assignment.provenance = AnimationMappingProvenanceV1 {
        provider: AnimationProviderV1::UserCustom,
        asset_id: "authored-root-pulse".to_owned(),
        ownership: AnimationOwnershipV1::UserOwned,
    };

    let artifact = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap();
    assert_eq!(
        artifact.animation_studio_readback.status,
        AnimationStudioReadbackStatusV1::Match
    );
    let readback = inspect_binary_mdl(&artifact.model).unwrap();
    let root_motion = readback
        .animations
        .iter()
        .find(|animation| animation.name == "cpause1")
        .and_then(|animation| animation.node_tree.roots.first())
        .and_then(|root| {
            root.controllers
                .iter()
                .find(|controller| controller.controller_type == 8)
        })
        .expect("authored root translation controller");
    assert_eq!(root_motion.times, [0.0, 0.5, 1.0]);
}

#[test]
fn v5_uses_the_same_procedural_library_for_build_and_authored_readback() {
    let source = fs::read(
        canonical_workspace::canonical_repository_root()
            .join("sample-3d/h2-clockwork-sentinel-1500/source.glb"),
    )
    .expect("canonical local H2 humanoid source");
    let appearance = appearance_fixture();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let source_clip_name = inspection
        .animations
        .clips
        .first()
        .expect("H2 source animation")
        .name
        .clone();
    let mut clip = m2a_core::animation_studio::clone_source_clip_for_editing_v1(
        &inspection.animations,
        &source_clip_name,
        AuthoredAnimationClipInputV1 {
            id: "authored-procedural-override".to_owned(),
            name: "authoverride".to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root.clone(),
        },
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    clip.events.push(AuthoredAnimationEventV1 {
        id: "studio-impact".to_owned(),
        time_seconds: clip.length_seconds * 0.25,
        name: "studio_impact".to_owned(),
    });
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision,
        authoring_revision: 12,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip],
    };
    let mut v2 = migrate_creature_animation_authoring_v1_to_v2(&procedural_v1_authoring(&source));
    v2.custom_animations.push(CustomAnimationDefinitionV2 {
        id: "custom-procedural-override".to_owned(),
        name: "custoverride".to_owned(),
        playback: CustomAnimationPlaybackV1::OneShot,
        clip_reference: Some(CustomAnimationClipReferenceV2 {
            source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
            source_clip_name: None,
            authored_clip_id: Some("authored-procedural-override".to_owned()),
        }),
        phases: vec![],
        provenance: AnimationMappingProvenanceV1 {
            provider: AnimationProviderV1::UserCustom,
            asset_id: "authored-procedural-override".to_owned(),
            ownership: AnimationOwnershipV1::UserOwned,
        },
    });
    let override_assignment = v2
        .assignments
        .iter_mut()
        .find(|assignment| assignment.target_slot.as_str() == "ca1slashl")
        .unwrap();
    override_assignment.source_kind = AnimationSourceKindV1::Custom;
    override_assignment.source_clip_name = None;
    override_assignment.custom_animation_id = Some("custom-procedural-override".to_owned());
    override_assignment.provenance = AnimationMappingProvenanceV1 {
        provider: AnimationProviderV1::UserCustom,
        asset_id: "authored-procedural-override".to_owned(),
        ownership: AnimationOwnershipV1::UserOwned,
    };

    let artifact = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap();
    assert_eq!(
        artifact.animation_studio_readback.status,
        AnimationStudioReadbackStatusV1::Match
    );
    assert_eq!(artifact.materialized_animations.animations.clips.len(), 43);
    assert!(
        artifact
            .materialized_animations
            .animations
            .clips
            .iter()
            .any(|clip| clip.name == "cpause1")
    );
    assert!(
        artifact
            .materialized_animations
            .animations
            .clips
            .iter()
            .any(|clip| clip.name == "custoverride")
    );
    let binary_readback = inspect_binary_mdl(&artifact.model).unwrap();
    let attack_events = &binary_readback
        .animations
        .iter()
        .find(|clip| clip.name == "ca1slashl")
        .expect("overridden base-42 attack clip")
        .events;
    assert!(
        attack_events
            .iter()
            .any(|event| event.name == "studio_impact")
    );
    assert!(attack_events.iter().any(|event| event.name == "hit"));
    let custom_events = &binary_readback
        .animations
        .iter()
        .find(|clip| clip.name == "custoverride")
        .expect("authored custom output clip")
        .events;
    assert_eq!(
        custom_events
            .iter()
            .map(|event| event.name.as_str())
            .collect::<Vec<_>>(),
        vec!["studio_impact"]
    );
    let usages = &artifact
        .manifest
        .animation_studio
        .authored_clips
        .first()
        .unwrap()
        .usages;
    assert_eq!(usages.len(), 2);
    assert!(
        usages
            .iter()
            .any(|usage| usage.output_clip_name == "ca1slashl")
    );
    assert!(
        usages
            .iter()
            .any(|usage| usage.output_clip_name == "custoverride")
    );
}

#[test]
#[ignore = "requires canonical local Void Crystal Knight source.glb"]
fn exact_void_crystal_knight_v5_preserves_geometry_and_stabilizes_accessories() {
    let source = fs::read(
        canonical_workspace::canonical_repository_root()
            .join("sample-3d/void-crystal-knight-h1-v1/source.glb"),
    )
    .expect("canonical local Void Crystal Knight source");
    let appearance = appearance_fixture();
    let v2 = migrate_creature_animation_authoring_v1_to_v2(&procedural_v1_authoring(&source));
    let studio = empty_studio(&source);

    let artifact = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap();

    assert_eq!(
        artifact.report.base.ingest.statistics.triangle_count,
        19_704
    );
    assert_eq!(artifact.report.base.geometry.triangle_count, 19_704);
    let stabilization = artifact
        .report
        .base
        .skin_accessory_stabilization
        .as_ref()
        .expect("V5 stabilization report");
    assert_eq!(stabilization.component_count, 5);
    assert_eq!(stabilization.stabilized_component_count, 4);
    assert_eq!(stabilization.changed_vertex_count, 341);
}

#[test]
#[ignore = "requires canonical local Void Crystal Knight source.glb"]
fn exact_void_crystal_knight_v5_materializes_a_real_authored_sword_attack() {
    let source = fs::read(
        canonical_workspace::canonical_repository_root()
            .join("sample-3d/void-crystal-knight-h1-v1/source.glb"),
    )
    .expect("canonical local Void Crystal Knight source");
    let appearance = appearance_fixture();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let mut clip = create_procedural_template_clip_v1(
        &inspection.rig,
        ProceduralAnimationTemplateV1::HumanoidSwordSlash,
        AuthoredAnimationClipInputV1 {
            id: "vck-authored-sword-slash-v1".to_owned(),
            name: "vck_cryslash".to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root.clone(),
        },
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision.clone(),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip],
    };
    let mut authoring =
        migrate_creature_animation_authoring_v1_to_v2(&procedural_v1_authoring(&source));
    authoring
        .custom_animations
        .push(CustomAnimationDefinitionV2 {
            id: "vck-custom-sword-slash-v1".to_owned(),
            name: "vck_cryslash".to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            clip_reference: Some(CustomAnimationClipReferenceV2 {
                source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
                source_clip_name: None,
                authored_clip_id: Some("vck-authored-sword-slash-v1".to_owned()),
            }),
            phases: vec![],
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: "vck-authored-sword-slash-v1".to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });
    let demo = apply_custom_attack_demo_route_v1(&authoring, "vck-custom-sword-slash-v1").unwrap();

    let identity = ProceduralCreaturePackageIdentityV1 {
        model_resref: "vckattack1".to_owned(),
        texture_resref: "vckattack1".to_owned(),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: "vckattack1".to_owned(),
            area_resref: "vckattack1".to_owned(),
            hak_resref: "vckattack1".to_owned(),
        },
        creature_resref: "vckattack1".to_owned(),
    };
    let artifact = build_meshy_h1_model_package_v5_with_identity(
        &source,
        &appearance,
        &demo.authoring,
        &studio,
        None,
        &identity,
    )
    .unwrap();
    assert_eq!(artifact.report.base.geometry.triangle_count, 19_704);
    assert_eq!(
        artifact
            .report
            .base
            .skin_accessory_stabilization
            .as_ref()
            .unwrap()
            .changed_vertex_count,
        341
    );
    assert_eq!(
        artifact.animation_studio_readback.status,
        AnimationStudioReadbackStatusV1::Match
    );
    assert!(demo.contract.production_idle_slot_preserved);
    assert!(demo.contract.all_native_attack_variants_routed);

    let behavior = artifact.report.base.animation_behavior.as_ref().unwrap();
    let attack_clips = ["ca1slashl", "ca1slashr", "ca1stab", "vck_cryslash"].map(|name| {
        behavior
            .clips
            .iter()
            .find(|clip| clip.name == name)
            .unwrap()
    });
    assert!(
        attack_clips
            .iter()
            .all(|clip| clip.changing_controller_count >= 12)
    );
    assert!(
        attack_clips
            .windows(2)
            .all(|pair| pair[0].motion_sha256 == pair[1].motion_sha256)
    );
    let production_idle = behavior
        .clips
        .iter()
        .find(|clip| clip.name == "cpause1")
        .unwrap();
    assert_ne!(production_idle.motion_sha256, attack_clips[0].motion_sha256);

    let binary_readback = inspect_binary_mdl(&artifact.model).unwrap();
    for name in ["ca1slashl", "ca1slashr", "ca1stab"] {
        let readback = binary_readback
            .animations
            .iter()
            .find(|clip| clip.name == name)
            .unwrap();
        assert!(
            readback.events.iter().any(|event| event.name == "hit"),
            "{name} must expose the native hit event"
        );
    }
    let module = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module).unwrap();
    assert_eq!(module.scene.module_resref, identity.module.module_resref);
    assert_eq!(module.scene.area_resref, identity.module.area_resref);
    assert_eq!(
        module.scene.ordered_hak_resrefs.as_slice(),
        std::slice::from_ref(&identity.module.hak_resref)
    );
    assert_eq!(
        module.scene.fixtures[0].template_resref,
        identity.creature_resref
    );
    assert_eq!(
        module.fixtures[0].runtime_profile,
        BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
    );

    let packet = std::env::temp_dir().join(format!(
        "m2a-vck-authored-sword-attack-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&packet);
    write_animation_studio_v5_demo_packet_with_identity_v1(&packet, &artifact, &identity).unwrap();
    assert_eq!(
        fs::read(packet.join("generated/vckattack1.mod")).unwrap(),
        artifact.proof_module
    );
    assert_eq!(
        fs::read(packet.join("generated/vckattack1.hak")).unwrap(),
        artifact.hak
    );
    assert_eq!(
        fs::read(packet.join("generated/vckattack1.mdl")).unwrap(),
        artifact.model
    );
    fs::remove_dir_all(packet).unwrap();
}

#[test]
fn v5_blocks_draft_and_stale_studio_documents() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let appearance = appearance_fixture();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let mut clip = m2a_core::animation_studio::clone_source_clip_for_editing_v1(
        &inspection.animations,
        "cpause1",
        AuthoredAnimationClipInputV1 {
            id: "draft".to_owned(),
            name: "draft".to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root,
        },
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Draft;
    let mut studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision,
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Draft,
        authored_clips: vec![clip],
    };
    let v2 = migrate_creature_animation_authoring_v1_to_v2(&v1_authoring(&source));
    let draft = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap_err();
    assert_eq!(draft.code, "M2A-ANIMATION-EDIT-SCHEMA");
    studio.source_revision = "b".repeat(64);
    let stale = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap_err();
    assert_eq!(stale.code, "M2A-ANIMATION-EDIT-SOURCE-STALE");
}

#[test]
fn v5_blocks_a_valid_authored_clip_without_any_packaged_usage() {
    let source = synthetic_owned_m6_full_native_42_glb_v1().unwrap();
    let appearance = appearance_fixture();
    let inspection = inspect_editable_animation_source_v1(&source).unwrap();
    let mut clip = m2a_core::animation_studio::clone_source_clip_for_editing_v1(
        &inspection.animations,
        "cpause1",
        AuthoredAnimationClipInputV1 {
            id: "valid-but-unused".to_owned(),
            name: "unused".to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root,
        },
    )
    .unwrap();
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision,
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip],
    };
    let v2 = migrate_creature_animation_authoring_v1_to_v2(&v1_authoring(&source));
    let error = build_meshy_h1_model_package_v5(&source, &appearance, &v2, &studio).unwrap_err();
    assert_eq!(error.code, "M2A-ANIMATION-EDIT-SCHEMA");
    assert_eq!(error.path, "authoredClips[valid-but-unused]");
    assert!(error.message.contains("omitted from the package"));
}
