use std::collections::BTreeSet;

use serde::Deserialize;

use m2a_core::{
    creature_animation_mapping::{
        AURORA_ANIMATION_STATE_CATALOG_V1, AnimationFallbackDecisionV1, AnimationFallbackReviewV1,
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CreatureAnimationAuthoringV1,
        CreatureAnimationMappingDiagnosticV1, CustomAnimationDefinitionV1,
        CustomAnimationPhaseKindV1, CustomAnimationPhaseV1, CustomAnimationPlaybackV1,
        DirectCreatureBaseSlotV1, DirectCreatureModelTypeV1, PlaybackPolicyV1,
        SourceAnimationClipV1, creature_animation_authoring_fingerprint_v1,
        direct_creature_base_catalog_v1, evaluate_authored_animation_conformance_v1,
        materialize_authored_direct_creature_clips_v1, resolve_base_slot_for_state_v1,
        resolve_creature_animation_mapping_v1, validate_creature_animation_authoring_contract_v1,
        validate_creature_animation_authoring_v1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    mdl::{MdlAnimationClipV1, MdlAnimationSetV1},
};

fn assignment(slot: &str) -> AnimationSourceAssignmentV1 {
    AnimationSourceAssignmentV1 {
        target_slot: DirectCreatureBaseSlotV1::try_from(slot).expect("known slot"),
        source_kind: AnimationSourceKindV1::SourceClip,
        source_clip_name: Some(slot.to_owned()),
        custom_animation_id: None,
        provenance: AnimationMappingProvenanceV1 {
            provider: AnimationProviderV1::SourceGlb,
            asset_id: "selected-source-glb".to_owned(),
            ownership: AnimationOwnershipV1::UserOwned,
        },
    }
}

fn complete_authoring() -> CreatureAnimationAuthoringV1 {
    CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: "sha256:owner-source".to_owned(),
        authoring_revision: 1,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| assignment(slot))
            .collect(),
        fallbacks: Vec::new(),
        custom_animations: Vec::new(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SharedCatalogContractV1 {
    schema_version: u32,
    profile: String,
    supported_model_types: Vec<String>,
    playback_policy: String,
    states: Vec<SharedCatalogStateV1>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SharedCatalogStateV1 {
    state_id: String,
    label: String,
    description: String,
    slot: String,
    gameplay_floor: bool,
}

#[test]
fn catalog_is_exactly_the_existing_full_native_42_namespace() {
    let catalog = direct_creature_base_catalog_v1();
    let names = catalog
        .iter()
        .map(|definition| definition.slot.as_str())
        .collect::<Vec<_>>();

    assert_eq!(catalog.len(), 42);
    assert_eq!(names, FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);
    assert_eq!(
        catalog
            .iter()
            .map(|definition| definition.state_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        42
    );
    assert_eq!(AURORA_ANIMATION_STATE_CATALOG_V1.len(), 42);
}

#[test]
fn rust_catalog_matches_the_shared_cross_layer_contract() {
    let shared: SharedCatalogContractV1 = serde_json::from_str(include_str!(
        "../../../contracts/creature-animation-catalog-v1.json"
    ))
    .expect("shared animation catalog");

    assert_eq!(shared.schema_version, 1);
    assert_eq!(shared.profile, "DIRECT_CREATURE_S_L_BASE_42_CATALOG_V1");
    assert_eq!(shared.supported_model_types, ["S", "L"]);
    assert_eq!(shared.playback_policy, "ENGINE_MANAGED");
    assert_eq!(shared.states.len(), 42);
    for (actual, expected) in AURORA_ANIMATION_STATE_CATALOG_V1.iter().zip(shared.states) {
        assert_eq!(actual.state_id, expected.state_id);
        assert_eq!(actual.label, expected.label);
        assert_eq!(actual.description, expected.description);
        assert_eq!(actual.slot, expected.slot);
        assert_eq!(
            m2a_core::creature_animation_mapping::is_gameplay_floor_slot_v1(actual.slot),
            expected.gameplay_floor
        );
    }
    assert_eq!(
        AURORA_ANIMATION_STATE_CATALOG_V1
            .iter()
            .filter(|state| {
                m2a_core::creature_animation_mapping::is_gameplay_floor_slot_v1(state.slot)
            })
            .count(),
        7
    );
}

#[test]
fn resolver_projects_state_through_s_or_l_model_type_without_guessing_pf() {
    let state = AURORA_ANIMATION_STATE_CATALOG_V1
        .iter()
        .find(|definition| definition.slot == "cwalk")
        .expect("walk state");

    assert_eq!(
        resolve_base_slot_for_state_v1(state.state_id, DirectCreatureModelTypeV1::Simple)
            .expect("simple slot")
            .as_str(),
        "cwalk"
    );
    assert_eq!(
        resolve_base_slot_for_state_v1(state.state_id, DirectCreatureModelTypeV1::Limited)
            .expect("limited slot")
            .as_str(),
        "cwalk"
    );
    assert_eq!(state.playback_policy, PlaybackPolicyV1::EngineManaged);
    assert!(
        resolve_base_slot_for_state_v1(
            "aurora.direct-creature.unknown",
            DirectCreatureModelTypeV1::Simple
        )
        .is_err()
    );
}

#[test]
fn versioned_authoring_round_trips_as_the_camel_case_cross_layer_contract() {
    let authoring = complete_authoring();
    let json = serde_json::to_value(&authoring).expect("serialize authoring");

    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["modelType"], "S");
    assert_eq!(json["assignments"][0]["sourceKind"], "SOURCE_CLIP");
    assert_eq!(
        json["assignments"][0]["provenance"]["provider"],
        "SOURCE_GLB"
    );
    assert!(json.get("supermodelPath").is_none());

    let decoded: CreatureAnimationAuthoringV1 =
        serde_json::from_value(json).expect("deserialize authoring");
    assert_eq!(decoded, authoring);

    let source_clip = SourceAnimationClipV1 {
        clip_id: "clip-1".to_owned(),
        name: "Walk".to_owned(),
        duration_seconds: 1.25,
        track_count: 8,
        target_node_ids: vec![1, 2, 3],
        target_paths: vec!["Hips.position".to_owned()],
    };
    assert_eq!(
        serde_json::to_value(source_clip).expect("serialize source clip")["durationSeconds"],
        1.25
    );
}

#[test]
fn contract_validation_uses_stable_codes_for_paths_fallback_review_and_custom_phases() {
    let mut authoring = complete_authoring();
    authoring.assignments[0].source_kind = AnimationSourceKindV1::InheritedSupermodel;
    authoring.assignments[0].provenance.provider = AnimationProviderV1::CompatibleSupermodel;
    authoring.assignments[0].provenance.asset_id =
        r"C:\Neverwinter Nights\data\c_horror.mdl".to_owned();
    authoring.assignments[1].source_kind = AnimationSourceKindV1::InheritedSupermodel;
    authoring.assignments[1].source_clip_name = None;
    authoring.assignments[1].provenance.provider = AnimationProviderV1::CompatibleSupermodel;
    authoring.assignments[1].provenance.asset_id = "builtin:c_horror".to_owned();
    authoring.fallbacks.push(AnimationFallbackDecisionV1 {
        id: "fallback-1".to_owned(),
        target_slot: DirectCreatureBaseSlotV1::try_from("cdamager").expect("target"),
        source_slot: DirectCreatureBaseSlotV1::try_from("cdamagel").expect("source"),
        reason: "Mirrored damage response".to_owned(),
        review: AnimationFallbackReviewV1::Pending,
    });
    authoring
        .custom_animations
        .push(CustomAnimationDefinitionV1 {
            id: "custom-1".to_owned(),
            name: "custom1".to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            source_clip_name: Some("Wave".to_owned()),
            phases: vec![CustomAnimationPhaseV1 {
                phase: CustomAnimationPhaseKindV1::End,
                source_clip_name: "WaveEnd".to_owned(),
            }],
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: "selected-source-glb".to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });
    authoring
        .custom_animations
        .push(CustomAnimationDefinitionV1 {
            id: "custom-phased".to_owned(),
            name: "wave".to_owned(),
            playback: CustomAnimationPlaybackV1::LoopingPhased,
            source_clip_name: None,
            phases: vec![
                CustomAnimationPhaseV1 {
                    phase: CustomAnimationPhaseKindV1::Start,
                    source_clip_name: "WaveStart".to_owned(),
                },
                CustomAnimationPhaseV1 {
                    phase: CustomAnimationPhaseKindV1::Loop,
                    source_clip_name: "WaveLoop".to_owned(),
                },
            ],
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: "selected-source-glb".to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });
    authoring
        .custom_animations
        .push(CustomAnimationDefinitionV1 {
            id: "custom-start-collision".to_owned(),
            name: "wave_s".to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            source_clip_name: Some("WaveStartAgain".to_owned()),
            phases: Vec::new(),
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: "selected-source-glb".to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });

    let diagnostics = validate_creature_animation_authoring_contract_v1(&authoring);
    let codes = diagnostics
        .iter()
        .map(|diagnostic: &CreatureAnimationMappingDiagnosticV1| diagnostic.code.as_str())
        .collect::<BTreeSet<_>>();

    assert!(codes.contains("M2A-ANIMATION-SUPERMODEL-PATH-FORBIDDEN"));
    assert!(codes.contains("M2A-ANIMATION-SUPERMODEL-PROVIDER-UNAVAILABLE"));
    assert!(codes.contains("M2A-ANIMATION-FALLBACK-REVIEW-REQUIRED"));
    assert!(codes.contains("M2A-ANIMATION-CUSTOM-ONE-SHOT-PHASES"));
    assert!(codes.contains("M2A-ANIMATION-CUSTOM-OUTPUT-NAME-CONFLICT"));
}

#[test]
fn canonical_validation_and_resolver_require_exact_ready_base_42() {
    let mut authoring = complete_authoring();
    let validation = validate_creature_animation_authoring_v1(&authoring);
    assert_eq!(
        validation.status,
        m2a_core::creature_animation_mapping::CreatureAnimationMappingStatusV1::Ready
    );
    assert_eq!(validation.mapped_base_slot_count, 42);
    assert_eq!(
        validation.authoring_fingerprint_sha256,
        creature_animation_authoring_fingerprint_v1(&authoring)
    );
    assert_eq!(
        resolve_creature_animation_mapping_v1(&authoring)
            .expect("ready mapping")
            .base_animations
            .len(),
        42
    );

    authoring.assignments.pop();
    let blocked = validate_creature_animation_authoring_v1(&authoring);
    assert_eq!(
        blocked.status,
        m2a_core::creature_animation_mapping::CreatureAnimationMappingStatusV1::Blocked
    );
    assert!(
        blocked
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-ANIMATION-BASE-SLOT-MISSING")
    );
    assert!(resolve_creature_animation_mapping_v1(&authoring).is_err());
}

#[test]
fn authored_materialization_renames_explicit_sources_without_idle_fallback() {
    let authoring = complete_authoring();
    let source = MdlAnimationSetV1 {
        schema_version: 1,
        clips: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|name| MdlAnimationClipV1 {
                name: name.to_ascii_uppercase(),
                animation_root: "root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.1,
                events: Vec::new(),
                tracks: Vec::new(),
            })
            .collect(),
    };

    let materialized = materialize_authored_direct_creature_clips_v1(&source, None, &authoring)
        .expect("materialized authored clips");
    assert_eq!(materialized.clips.len(), 42);
    assert_eq!(
        materialized
            .clips
            .iter()
            .map(|clip| clip.name.as_str())
            .collect::<Vec<_>>(),
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
    );
    let conformance = evaluate_authored_animation_conformance_v1(&authoring, &materialized);
    assert_eq!(conformance.materialized_base_slot_count, 42);
    assert!(conformance.diagnostics.is_empty());
}
