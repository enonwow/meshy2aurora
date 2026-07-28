use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
};

use m2a_core::{
    creature_animation_mapping::{
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CreatureAnimationAuthoringV1,
        DirectCreatureBaseSlotV1, DirectCreatureModelTypeV1,
    },
    direct_creature_animation::{
        COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1, DirectCreatureAnimationEventProfileV1,
        DirectCreatureClipEventAuthoringV1, DirectCreatureEventAuthoringV1,
        apply_direct_creature_event_authoring_v1, evaluate_direct_creature_event_conformance_v1,
    },
    direct_creature_contract::{
        DirectCreatureRuntimeProfileV2, SourceTopologyOriginV1,
        declare_m0_direct_creature_runtime_profile_v2, inspect_m0_source_topology_binding_v1,
        source_topology_summary_digest_v1,
    },
    erf::ErfArchive,
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    inspect_binary_mdl,
    mdl::{
        DirectCreatureEngineEnvelopeVerdictV1, MdlAnimationClipV1, MdlAnimationEventV1,
        MdlAnimationSetV1, MdlFormatProfileV1, MdlStateProjectionProfileV1,
        MdlStateProjectionProvenanceV1, direct_creature_engine_envelope_digest_v1,
        direct_creature_structural_summary_digest_v1, evaluate_skin_deformation_v1,
        inspect_direct_creature_engine_envelope_v1, summarize_direct_creature_structure_v1,
        verify_direct_creature_state_projection_v1,
        verify_direct_creature_state_projection_with_expected_provenance_v1,
    },
    model_pipeline::{
        DirectCreatureAnimationProfileV1, FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
        M0_APPEARANCE_LABEL, M0_CONTROL_APPEARANCE_LABEL, M0_MODEL_RESREF, M0_TEXTURE_RESREF,
        M0RuntimeResourceBindingV1, M6_APPEARANCE_LABEL, M6_HAK_FILE_NAME, M6_MODEL_RESREF,
        M6_PROOF_MODULE_FILE_NAME, M6_TEXTURE_RESREF, ProceduralCreaturePackageIdentityV1,
        build_m6_model_package_v1, build_m6_model_package_with_profile_v1,
        build_m6_model_package_with_profile_v2, build_m6_model_package_with_profile_v3,
        build_meshy_h1_model_package_v2, build_meshy_h1_model_package_v3,
        build_meshy_h1_model_package_v4, build_meshy_h1_model_package_v4_with_identity,
        build_meshy_h1_rigid_runtime_diagnostic_package_v1,
        build_meshy_m0_canonical_runtime_package_v1,
        build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2,
        build_meshy_m0_canonical_runtime_package_with_identity_v1,
        build_meshy_m0_canonical_runtime_package_with_profile_v2,
        build_meshy_m0_static_rigid_package_v1,
        build_meshy_m0_static_rigid_package_with_profile_v2,
        build_meshy_procedural_humanoid_model_package_with_identity_v1,
        inspect_m0_runtime_mesh_eligibility_v1, materialize_direct_creature_runtime_clips_v1,
        verify_m0_binary_runtime_fixture_contract_v2,
        verify_m0_full_runtime_appearance_table_binding_v1, write_m0_canonical_proof_packet_v1,
        write_m0_canonical_runtime_proof_packet_with_profile_v2, write_m0_proof_packet_v1,
        write_m6_proof_packet_v1,
    },
    owned_fixture::{
        synthetic_owned_m6_animation_mapping_v1, synthetic_owned_m6_full_native_42_glb_v1,
        synthetic_owned_m6_glb_v1, synthetic_owned_m6_rig_v1,
    },
    profile_a::{ProfileAAnimationClipMappingV1, canonical_profile_sha256},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureRuntimeProfileV2,
        BinaryM0VerticalSliceIdentityV1, M0_CANONICAL_PROOF_CREATURE_RESREF,
        M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y, M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y,
        M0_RUNTIME_FIXTURE_X, M0_RUNTIME_FIXTURE_Y, PROOF_AREA_RESREF, PROOF_HAK_RESREF,
        PROOF_MODULE_RESREF, inspect_binary_creature_profile_matrix_module_v2,
    },
    two_da::{TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[path = "support/canonical_workspace.rs"]
mod canonical_workspace;

fn appearance_fixture() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n".to_vec()
}

fn appearance_fixture_without_phenotype() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n".to_vec()
}

fn appearance_fixture_with_toolset_invisible_tail() -> Vec<u8> {
    let mut payload = b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n".to_vec();
    for row in 0..15_105 {
        let label = if row < 848 {
            format!("Visible_{row}")
        } else {
            format!("OS_RESERVED_{row}")
        };
        payload.extend_from_slice(
            format!("{row} {label} NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n").as_bytes(),
        );
    }
    payload
}

fn common_native_event_authoring(length_seconds: f32) -> DirectCreatureEventAuthoringV1 {
    let mut grouped = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for (clip_name, event_name) in COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1 {
        grouped.entry(clip_name).or_default().push(event_name);
    }
    DirectCreatureEventAuthoringV1 {
        schema_version: 1,
        clips: grouped
            .into_iter()
            .map(
                |(clip_name, event_names)| DirectCreatureClipEventAuthoringV1 {
                    clip_name: clip_name.to_owned(),
                    events: event_names
                        .into_iter()
                        .enumerate()
                        .map(|(index, name)| MdlAnimationEventV1 {
                            time_seconds: (length_seconds * (index as f32 + 1.0) / 4.0)
                                .min(length_seconds),
                            name: name.to_owned(),
                        })
                        .collect(),
                },
            )
            .collect(),
    }
}

fn static_meshy_m0_fixture() -> Vec<u8> {
    mutate_glb(
        synthetic_owned_m6_glb_v1().expect("owned GLB fixture"),
        |root| {
            root["skins"] = serde_json::json!([]);
            root["animations"] = serde_json::json!([]);
            root["scenes"][0]["nodes"] = serde_json::json!([0]);
            root["nodes"] = serde_json::json!([{
                "name": "m0-source-root",
                "mesh": 0
            }]);
            let attributes = root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .expect("synthetic primitive attributes must be an object");
            attributes.remove("JOINTS_0");
            attributes.remove("WEIGHTS_0");
        },
    )
}

fn static_meshy_m0_fixture_below_ground() -> Vec<u8> {
    mutate_glb(static_meshy_m0_fixture(), |root| {
        let root_node = root["nodes"]
            .as_array_mut()
            .expect("synthetic nodes must be an array")
            .iter_mut()
            .find(|node| node["name"] == "m0-source-root")
            .expect("synthetic M0 fixture must retain its scene root");
        root_node["translation"] = serde_json::json!([0.0, -2.0, 0.0]);
    })
}

fn temp_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("m2a-{label}-{}", std::process::id()))
}

#[test]
fn full_native_creature_profile_requires_all_42_explicit_clips_without_idle_fallback() {
    let idle = MdlAnimationClipV1 {
        name: "cpause1".to_owned(),
        animation_root: "owned_root".to_owned(),
        length_seconds: 1.0,
        transition_seconds: 0.25,
        events: Vec::new(),
        tracks: Vec::new(),
    };
    let loader_smoke = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![idle.clone()],
    };

    let error = materialize_direct_creature_runtime_clips_v1(
        &loader_smoke,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect_err("a single idle clip must never satisfy the full creature profile");
    assert_eq!(error.code, "M6-ANIMATION-FULL-PROFILE-MISSING");
    assert_eq!(error.path, "conversion.animations.clips");
    assert!(error.message.contains("ca1slashl"));
    assert!(error.message.contains("cdead"));

    let gameplay = materialize_direct_creature_runtime_clips_v1(
        &loader_smoke,
        DirectCreatureAnimationProfileV1::GameplayFloor7IdleFallbackV1,
    )
    .expect("the legacy gameplay-floor profile retains its explicit fallback policy");
    assert_eq!(gameplay.clips.len(), 7);

    let full = MdlAnimationSetV1 {
        schema_version: 1,
        clips: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .enumerate()
            .map(|(index, name)| MdlAnimationClipV1 {
                name: (*name).to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: index as f32 + 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: Vec::new(),
            })
            .collect(),
    };
    let materialized = materialize_direct_creature_runtime_clips_v1(
        &full,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect("all 42 explicitly supplied clips satisfy the full profile");
    assert_eq!(materialized, full, "full mode must not synthesize aliases");
    assert_eq!(materialized.clips.len(), 42);
    assert_eq!(
        materialized
            .clips
            .iter()
            .map(|clip| clip.name.as_str())
            .collect::<Vec<_>>(),
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
    );

    let mut full_with_unknown = full.clone();
    let mut unknown = idle;
    unknown.name = "m2a_unknown_state".to_owned();
    full_with_unknown.clips.push(unknown);
    let error = materialize_direct_creature_runtime_clips_v1(
        &full_with_unknown,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect_err("the exact full namespace must reject unclassified extra clips");
    assert_eq!(error.code, "M6-ANIMATION-FULL-PROFILE-UNKNOWN");
    assert!(error.message.contains("m2a_unknown_state"));
}

#[test]
fn procedural_humanoid_profile_authors_a_distinct_owned_42_state_set_from_h2_idle() {
    let repo = canonical_workspace::canonical_repository_root();
    let h2 = fs::read(repo.join("sample-3d/h2-clockwork-sentinel-1500/source.glb"))
        .expect("owned H2 humanoid source");

    let identity = ProceduralCreaturePackageIdentityV1 {
        model_resref: "m2a_idmdl".to_owned(),
        texture_resref: "m2a_idtex".to_owned(),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: "m2a_idmod".to_owned(),
            area_resref: "m2a_idarea".to_owned(),
            hak_resref: "m2a_idhak".to_owned(),
        },
        creature_resref: "m2a_idutc".to_owned(),
    };
    let output = build_meshy_procedural_humanoid_model_package_with_identity_v1(
        &h2,
        &appearance_fixture(),
        &identity,
    )
    .expect("idle-only owned H2 must receive clean-room procedural direct-creature motion");
    let completeness = output
        .report
        .animation_completeness
        .as_ref()
        .expect("procedural profile completeness");
    assert_eq!(completeness.required_clip_count, 42);
    assert_eq!(completeness.explicit_clip_count, 1);
    assert_eq!(completeness.procedural_clip_count, 41);
    assert_eq!(completeness.fallback_alias_count, 0);
    assert!(completeness.complete);

    let readback = inspect_binary_mdl(&output.model).expect("procedural H2 binary readback");
    assert_eq!(
        readback
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>(),
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    );
    assert!(
        readback
            .animations
            .iter()
            .all(|animation| animation.animation_type == 5),
    );
    let root = readback
        .node_tree
        .roots
        .first()
        .expect("dedicated procedural Aurora root");
    assert_eq!(
        (root.name.as_str(), root.number),
        (identity.model_resref.as_str(), 0),
        "the dedicated procedural model root must use the native part-0 identity"
    );
    assert!(
        root.controllers.is_empty(),
        "the correct native direct-creature root is controllerless",
    );
    assert!(
        root.children
            .iter()
            .any(|child| child.name == "Hips" && child.skin.is_none()),
        "the weighted source skeleton must remain below the dedicated Aurora root",
    );
    assert!(
        root.children.iter().any(|child| child.skin.is_some()),
        "SkinMesh must be attached directly to the dedicated Aurora root",
    );
    assert!(readback.animations.iter().all(|animation| {
        animation.node_tree.roots.iter().all(|state_root| {
            state_root.name == identity.model_resref
                && state_root.number == 0
                && state_root.controllers.is_empty()
        })
    }));
    assert_eq!(
        output.report.appearance.changed_cells.len(),
        36,
        "the procedural product path must append a full-width donor clone, including the fixture phenotype column"
    );
    let bind_sample =
        evaluate_skin_deformation_v1(&readback, "cpause1", 0.0).expect("bind-pose sample");
    let skin_node = root
        .children
        .iter()
        .find(|child| child.skin.is_some())
        .expect("direct-root SkinMesh");
    assert_eq!(
        skin_node
            .controllers
            .iter()
            .map(|controller| controller.controller_type)
            .collect::<Vec<_>>(),
        [8, 20],
        "every native SkinMesh in the local CEP corpus has at least explicit position and orientation bind controllers",
    );
    assert_eq!(skin_node.controllers[0].times, [0.0]);
    assert_eq!(skin_node.controllers[0].values, [[0.0, 0.0, 0.0]]);
    assert_eq!(skin_node.controllers[1].times, [0.0]);
    assert_eq!(skin_node.controllers[1].values, [[0.0, 0.0, 0.0, 1.0]],);
    let mesh = skin_node.mesh.as_ref().expect("SkinMesh payload");
    let max_bind_error = bind_sample.skins[0]
        .vertices
        .iter()
        .zip(&mesh.vertices)
        .map(|(sample, vertex)| {
            ((sample.bind_world[0] - vertex.x).powi(2)
                + (sample.bind_world[1] - vertex.y).powi(2)
                + (sample.bind_world[2] - vertex.z).powi(2))
            .sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(
        max_bind_error <= 1.0e-4,
        "SkinMesh bind reconstruction must agree with direct-root geometry, error={max_bind_error}",
    );
    let (mesh_min, mesh_max) = mesh.vertices.iter().fold(
        ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]),
        |(mut min, mut max), vertex| {
            for (axis, value) in [vertex.x, vertex.y, vertex.z].into_iter().enumerate() {
                min[axis] = min[axis].min(value);
                max[axis] = max[axis].max(value);
            }
            (min, max)
        },
    );
    assert!(mesh_min.into_iter().chain(mesh_max).all(f32::is_finite));
    assert!(
        mesh_min[2].abs() <= 1.0e-4,
        "procedural H2 must remain grounded, minZ={}",
        mesh_min[2],
    );
    assert!(
        (1.6..=1.8).contains(&(mesh_max[2] - mesh_min[2])),
        "procedural H2 must retain its expected 1.7m scale, bounds={mesh_min:?}..{mesh_max:?}",
    );
    assert!(
        mesh_min[0] >= -1.0 && mesh_max[0] <= 1.0 && mesh_min[1] >= -1.0 && mesh_max[1] <= 1.0,
        "procedural H2 must not regress to the oversized corrupt blob, bounds={mesh_min:?}..{mesh_max:?}",
    );
    for animation in &readback.animations {
        evaluate_skin_deformation_v1(&readback, &animation.name, 0.0).unwrap_or_else(|error| {
            panic!(
                "every procedural state must remain inside the unit-scale SkinMesh oracle; {} failed: {}",
                animation.name, error
            )
        });
    }
    let behavior = output
        .report
        .animation_behavior
        .as_ref()
        .expect("procedural behavior report");
    assert!(behavior.behavior_candidate_eligible);
    assert!(behavior.active_motion_complete);
    assert!(behavior.walk_run_distinct);
    assert!(behavior.essential_states_distinct);
    assert!(behavior.death_transition_terminal_pose);
    assert!(behavior.violations.is_empty());
    let events = output
        .report
        .animation_event_conformance
        .as_ref()
        .expect("procedural gameplay event conformance");
    assert!(events.complete);
    assert_eq!(events.required_pair_count, 23);
    assert_eq!(events.satisfied_pair_count, 23);
    assert_eq!(events.total_event_count, 23);

    let skin = output
        .report
        .skin_animation_conformance
        .as_ref()
        .expect("procedural H2 SkinMesh conformance");
    assert!(skin.complete);
    assert!(
        skin.clips
            .iter()
            .all(|clip| clip.max_moved_vertex_count > 0),
    );
    assert!(
        skin.clips
            .iter()
            .all(|clip| clip.non_rigid_deformation_observed),
    );

    let module = inspect_binary_creature_profile_matrix_module_v2(&output.proof_module)
        .expect("procedural H2 fixture must use the independently read-back profiled module");
    assert_eq!(module.fixtures.len(), 1);
    assert_eq!(
        module.fixtures[0].runtime_profile,
        BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
    );
    assert_eq!(module.scene.fixtures.len(), 1);
    assert_eq!(module.scene.module_resref, identity.module.module_resref);
    assert_eq!(module.scene.area_resref, identity.module.area_resref);
    assert_eq!(
        module.scene.ordered_hak_resrefs,
        std::slice::from_ref(&identity.module.hak_resref)
    );
    assert_eq!(
        module.scene.fixtures[0].template_resref,
        identity.creature_resref
    );
    assert_eq!(
        module.scene.fixtures[0].appearance_row,
        output.report.appearance.appended_row_index
    );
    assert_eq!(module.scene.fixtures[0].position.x, M0_RUNTIME_FIXTURE_X);
    assert_eq!(module.scene.fixtures[0].position.y, M0_RUNTIME_FIXTURE_Y);
}

#[test]
fn caller_owned_event_authoring_preserves_times_and_enforces_the_common_native_hook_floor() {
    let source = MdlAnimationSetV1 {
        schema_version: 1,
        clips: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|name| MdlAnimationClipV1 {
                name: (*name).to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: Vec::new(),
            })
            .collect(),
    };
    let mut authoring = common_native_event_authoring(1.0);
    authoring
        .clips
        .iter_mut()
        .find(|clip| clip.clip_name == "cwalk")
        .expect("cwalk authoring")
        .events
        .push(MdlAnimationEventV1 {
            time_seconds: 0.75,
            name: "owned_custom_marker".to_owned(),
        });

    let authored = apply_direct_creature_event_authoring_v1(&source, &authoring)
        .expect("caller-owned events must be attached by exact output clip name");
    let conformance = evaluate_direct_creature_event_conformance_v1(
        &authored,
        DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
    );
    assert!(conformance.complete);
    assert_eq!(conformance.required_pair_count, 23);
    assert_eq!(conformance.satisfied_pair_count, 23);
    assert_eq!(conformance.total_event_count, 24);
    assert_eq!(
        conformance.unknown_event_names,
        vec!["owned_custom_marker".to_owned()]
    );
    assert!(conformance.missing_pairs.is_empty());
    let cwalk = authored
        .clips
        .iter()
        .find(|clip| clip.name == "cwalk")
        .expect("authored cwalk");
    assert_eq!(
        cwalk.events.last(),
        Some(&MdlAnimationEventV1 {
            time_seconds: 0.75,
            name: "owned_custom_marker".to_owned(),
        })
    );

    let mut missing = authoring.clone();
    missing
        .clips
        .iter_mut()
        .find(|clip| clip.clip_name == "ccastout")
        .expect("ccastout authoring")
        .events
        .clear();
    let missing = apply_direct_creature_event_authoring_v1(&source, &missing).unwrap();
    let report = evaluate_direct_creature_event_conformance_v1(
        &missing,
        DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
    );
    assert!(!report.complete);
    assert_eq!(report.missing_pairs, vec!["ccastout:cast".to_owned()]);

    let mut invalid_time = authoring.clone();
    invalid_time.clips[0].events[0].time_seconds = 2.0;
    let error = apply_direct_creature_event_authoring_v1(&source, &invalid_time)
        .expect_err("event times outside the exact output clip must fail closed");
    assert_eq!(error.code, "M6-ANIMATION-EVENT-AUTHORING-TIME");

    let mut duplicate = authoring;
    duplicate.clips.push(duplicate.clips[0].clone());
    let error = apply_direct_creature_event_authoring_v1(&source, &duplicate)
        .expect_err("duplicate authored clip entries must fail after ASCII case-fold");
    assert_eq!(error.code, "M6-ANIMATION-EVENT-AUTHORING-DUPLICATE");
}

#[test]
fn full_native_42_profile_materializes_end_to_end_without_changing_legacy_v1() {
    let glb = synthetic_owned_m6_full_native_42_glb_v1().expect("owned full native 42 GLB fixture");
    let rig = synthetic_owned_m6_rig_v1().expect("owned rig");
    let mut mapping = synthetic_owned_m6_animation_mapping_v1();
    mapping.clip_mappings = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .enumerate()
        .map(
            |(source_animation_id, output_clip_name)| ProfileAAnimationClipMappingV1 {
                source_animation_id: source_animation_id as u32,
                output_clip_name: (*output_clip_name).to_owned(),
                transition_seconds: 0.25,
            },
        )
        .collect();

    let full = build_m6_model_package_with_profile_v2(
        &glb,
        &appearance_fixture(),
        &rig,
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect("42 explicitly mapped owned clips must materialize through the complete pipeline");
    let completeness = full
        .report
        .animation_completeness
        .as_ref()
        .expect("V2 full output must report its animation completeness");
    assert_eq!(
        completeness.profile,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1
    );
    assert_eq!(completeness.required_clip_count, 42);
    assert_eq!(completeness.explicit_clip_count, 42);
    assert_eq!(completeness.fallback_alias_count, 0);
    assert!(completeness.complete);
    let behavior = full
        .report
        .animation_behavior
        .as_ref()
        .expect("V2 full output must expose behavior evidence separately");
    assert!(behavior.full_namespace_complete);
    assert!(behavior.all_required_content_present);
    assert!(behavior.active_motion_complete);
    assert!(behavior.behavior_candidate_eligible);
    assert!(behavior.walk_run_distinct);
    assert!(behavior.essential_states_distinct);
    assert!(behavior.death_transition_terminal_pose);
    assert!(behavior.violations.is_empty());
    assert!(full.report.animation_event_conformance.is_none());
    let readback = inspect_binary_mdl(&full.model).expect("full profile MDL readback");
    assert_eq!(readback.animations.len(), 42);
    assert_eq!(
        readback
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>(),
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
    );
    assert!(
        readback
            .animations
            .iter()
            .all(|animation| animation.animation_type == 5)
    );
    let walk = readback
        .animations
        .iter()
        .find(|animation| animation.name == "cwalk")
        .expect("cwalk readback");
    let walk_deformation = evaluate_skin_deformation_v1(&readback, "cwalk", walk.length)
        .expect("full profile cwalk must deform the emitted SkinMesh");
    assert!(
        walk_deformation.moved_vertex_count > 0,
        "a child-bone clip must move weighted vertices"
    );
    let distinct_displacements = walk_deformation.skins[0]
        .vertices
        .iter()
        .map(|vertex| (vertex.displacement * 100_000.0).round() as i64)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        distinct_displacements.len() > 1,
        "weighted cwalk must produce non-uniform per-vertex displacement"
    );
    let skin_animation = full
        .report
        .skin_animation_conformance
        .as_ref()
        .expect("full profile must expose SkinMesh animation conformance");
    assert!(skin_animation.complete);
    assert_eq!(skin_animation.active_joint_count, 2);
    assert!(
        skin_animation
            .clips
            .iter()
            .all(|clip| clip.non_rigid_deformation_observed)
    );
    assert!(skin_animation.violations.is_empty());

    let event_authoring = common_native_event_authoring(1.25);
    let eventful = build_m6_model_package_with_profile_v3(
        &glb,
        &appearance_fixture(),
        &rig,
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
        &event_authoring,
    )
    .expect("full V3 must materialize caller-owned common gameplay hooks");
    let event_conformance = eventful
        .report
        .animation_event_conformance
        .as_ref()
        .expect("V3 event conformance report");
    assert!(event_conformance.complete);
    assert_eq!(event_conformance.required_pair_count, 23);
    assert_eq!(event_conformance.satisfied_pair_count, 23);
    assert_eq!(event_conformance.total_event_count, 23);
    let canonical_event_bytes =
        serde_json::to_vec(&event_authoring).expect("canonical event authoring bytes");
    let canonical_event_identity = eventful
        .report
        .animation_event_authoring_canonical
        .as_ref()
        .expect("V3 canonical event authoring identity");
    assert_eq!(
        canonical_event_identity.byte_length,
        canonical_event_bytes.len() as u64
    );
    assert_eq!(
        canonical_event_identity.sha256,
        format!("{:x}", Sha256::digest(&canonical_event_bytes))
    );
    let eventful_readback = inspect_binary_mdl(&eventful.model).expect("eventful V3 readback");
    let castout = eventful_readback
        .animations
        .iter()
        .find(|clip| clip.name == "ccastout")
        .expect("eventful ccastout");
    assert_eq!(castout.events.len(), 1);
    assert_eq!(castout.events[0].name, "cast");

    let mut incomplete_events = event_authoring;
    incomplete_events
        .clips
        .iter_mut()
        .find(|clip| clip.clip_name == "ccastout")
        .expect("ccastout authoring")
        .events
        .clear();
    let error = build_m6_model_package_with_profile_v3(
        &glb,
        &appearance_fixture(),
        &rig,
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
        &incomplete_events,
    )
    .expect_err("V3 must fail closed when one required gameplay hook is absent");
    assert_eq!(error.code, "M6-ANIMATION-EVENTS-INELIGIBLE");
    assert!(error.message.contains("ccastout:cast"));

    let root_only_glb = mutate_glb(glb.clone(), |root| {
        for animation in root["animations"]
            .as_array_mut()
            .expect("full fixture animations")
        {
            animation["channels"][0]["target"]["node"] = serde_json::json!(0);
        }
    });
    let error = build_m6_model_package_with_profile_v2(
        &root_only_glb,
        &appearance_fixture(),
        &rig,
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect_err("root-only motion must not satisfy full SkinMesh animation conformance");
    assert_eq!(error.code, "M6-SKIN-ANIMATION-INELIGIBLE");
    assert!(error.message.contains("NON_RIGID_DEFORMATION_MISSING"));

    mapping
        .clip_mappings
        .last_mut()
        .expect("42-state mapping")
        .output_clip_name = "m2a_custom_state".to_owned();
    let error = build_m6_model_package_with_profile_v2(
        &glb,
        &appearance_fixture(),
        &rig,
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect_err("full mode must reject a mapping with one missing native state");
    assert_eq!(error.code, "M6-ANIMATION-FULL-PROFILE-MISSING");
    assert!(error.message.contains("ccturnr"));

    let legacy = build_m6_model_package_v1(
        &synthetic_owned_m6_glb_v1().expect("legacy owned GLB"),
        &appearance_fixture(),
    )
    .expect("legacy V1 remains available for frozen lineages");
    assert!(legacy.report.animation_completeness.is_none());
    assert!(legacy.report.animation_behavior.is_none());
    assert_eq!(
        inspect_binary_mdl(&legacy.model)
            .expect("legacy V1 readback")
            .animations
            .len(),
        7
    );
}

#[test]
fn automatic_h1_v2_accepts_exactly_named_full_source_and_rejects_idle_only_source() {
    let full_source =
        synthetic_owned_m6_full_native_42_glb_v1().expect("owned full native 42 GLB fixture");
    let full = build_meshy_h1_model_package_v2(
        &full_source,
        &appearance_fixture(),
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect("automatic H1 V2 must preserve exact native source animation names");
    let readback = inspect_binary_mdl(&full.model).expect("automatic full H1 readback");
    assert_eq!(
        readback
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>(),
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
    );
    let eventful = build_meshy_h1_model_package_v3(
        &full_source,
        &appearance_fixture(),
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
        &common_native_event_authoring(1.25),
    )
    .expect("automatic H1 V3 must attach caller-owned event hooks");
    assert!(
        eventful
            .report
            .animation_event_conformance
            .as_ref()
            .is_some_and(|report| report.complete)
    );
    let authored = CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: hex_sha256(&full_source),
        authoring_revision: 7,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).expect("base slot"),
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
        fallbacks: Vec::new(),
        custom_animations: Vec::new(),
    };
    let v4 = build_meshy_h1_model_package_v4(&full_source, &appearance_fixture(), &authored)
        .expect("authored H1 V4 must materialize the exact ready mapping");
    assert_eq!(
        v4.report
            .animation_authoring
            .as_ref()
            .expect("report authoring")
            .authoring_revision,
        7
    );
    assert_eq!(
        v4.manifest
            .authored_animation_conformance
            .as_ref()
            .expect("manifest conformance")
            .materialized_base_slot_count,
        42
    );
    assert_eq!(v4.model, full.model, "authored V4 binary MDL drift");
    assert_eq!(v4.texture, full.texture, "authored V4 TGA drift");
    assert_eq!(
        v4.appearance_two_da, full.appearance_two_da,
        "authored V4 2DA drift"
    );
    assert_eq!(v4.hak, full.hak, "authored V4 HAK drift");
    assert_eq!(v4.proof_module, full.proof_module, "authored V4 MOD drift");
    let custom_identity = ProceduralCreaturePackageIdentityV1 {
        model_resref: "m2a_v4mdl".to_owned(),
        texture_resref: "m2a_v4tex".to_owned(),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: "m2a_v4mod".to_owned(),
            area_resref: "m2a_v4area".to_owned(),
            hak_resref: "m2a_v4hak".to_owned(),
        },
        creature_resref: "m2a_v4utc".to_owned(),
    };
    let custom_v4 = build_meshy_h1_model_package_v4_with_identity(
        &full_source,
        &appearance_fixture(),
        &authored,
        &custom_identity,
    )
    .expect("authored V4 must freeze a caller-owned runtime identity");
    assert_eq!(custom_v4.summary.model_resref, custom_identity.model_resref);
    assert!(
        ErfArchive::parse(&custom_v4.hak)
            .expect("custom V4 HAK")
            .find(&custom_identity.model_resref, 2002)
            .is_ok()
    );
    let custom_module = inspect_binary_creature_profile_matrix_module_v2(&custom_v4.proof_module)
        .expect("custom V4 module");
    assert_eq!(
        custom_module.scene.module_resref,
        custom_identity.module.module_resref
    );
    assert_eq!(
        custom_module.scene.area_resref,
        custom_identity.module.area_resref
    );
    assert_eq!(
        custom_module.scene.ordered_hak_resrefs,
        [custom_identity.module.hak_resref]
    );
    assert_eq!(
        custom_module.scene.fixtures[0].template_resref,
        custom_identity.creature_resref
    );

    let indistinguishable_movement = mutate_glb(full_source.clone(), |root| {
        let animations = root["animations"]
            .as_array_mut()
            .expect("owned full animations");
        let walk_sampler = animations
            .iter()
            .find(|animation| animation["name"] == "cwalk")
            .expect("cwalk source")["samplers"]
            .clone();
        animations
            .iter_mut()
            .find(|animation| animation["name"] == "crun")
            .expect("crun source")["samplers"] = walk_sampler;
    });
    let error = build_meshy_h1_model_package_v2(
        &indistinguishable_movement,
        &appearance_fixture(),
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect_err("full H1 V2 must reject behavior-identical walk and run");
    assert_eq!(error.code, "M6-ANIMATION-BEHAVIOR-INELIGIBLE");
    assert!(error.message.contains("WALK_RUN_NOT_DISTINCT"));

    let error = build_meshy_h1_model_package_v2(
        &synthetic_owned_m6_glb_v1().expect("idle-only owned GLB"),
        &appearance_fixture(),
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .expect_err("automatic H1 full mode must not rename idle into 41 missing states");
    assert_eq!(error.code, "M6-ANIMATION-FULL-PROFILE-MISSING");
}

#[test]
fn owned_fixture_materializes_deterministically_through_the_complete_model_pipeline() {
    let glb = synthetic_owned_m6_glb_v1().expect("owned GLB fixture");
    let first = build_m6_model_package_v1(&glb, &appearance_fixture()).expect("first package");
    let second = build_m6_model_package_v1(&glb, &appearance_fixture()).expect("second package");

    assert_eq!(first.source_glb, second.source_glb);
    assert_eq!(first.model, second.model);
    assert_eq!(first.texture, second.texture);
    assert_eq!(first.appearance_two_da, second.appearance_two_da);
    assert_eq!(first.hak, second.hak);
    assert_eq!(first.manifest_json, second.manifest_json);
    assert_eq!(first.report_json, second.report_json);
    assert_eq!(first.summary_json, second.summary_json);

    assert_eq!(first.summary.status, "M6_MODEL_PACKAGE_MATERIALIZED");
    assert_eq!(first.summary.model_resref, M6_MODEL_RESREF);
    assert_eq!(first.summary.texture_resref, M6_TEXTURE_RESREF);
    assert_eq!(first.summary.appended_physical_row, 1);
    assert_eq!(first.summary.animation.output_name, "cpause1");
    assert!(first.summary.animation.has_motion);
    assert!(first.summary.provenance.export_allowed);
    assert!(first.summary.zero_reference_model_payload_copied);
    assert_eq!(
        first.summary.appearance_payload_policy,
        "PRESERVED_AND_APPENDED"
    );
    assert_eq!(first.summary.input_glb.sha256, hex_sha256(&glb));
    assert_eq!(
        first.summary.input_appearance_two_da.sha256,
        hex_sha256(&appearance_fixture())
    );
    assert_ne!(
        first.summary.input_appearance_two_da.sha256,
        first.summary.outputs.appearance_two_da.sha256
    );
    assert_eq!(first.report.resolved_base_color_image_index, 1);
    assert!(first.report.geometry.triangle_count > 1);
    assert!(first.report.geometry.bounds_min != first.report.geometry.bounds_max);
    assert_eq!(first.report.geometry.output_segment_deformation, "SKIN");
    assert_eq!(first.report.geometry.active_joint_count, 2);
    assert_eq!(first.report.texture_selection.source_texture_id, 1);
    assert_eq!(first.report.texture_selection.source_image_id, 1);
    assert_eq!(first.report.texture.width, 2);
    assert_eq!(first.report.texture.height, 2);
    assert_eq!(first.report.appearance.changed_cells.len(), 36);
    assert_eq!(
        first.report.appearance.changed_cells[0].column_name,
        "LABEL"
    );
    assert!(
        first
            .appearance_two_da
            .windows(M6_APPEARANCE_LABEL.len())
            .any(|w| w == M6_APPEARANCE_LABEL.as_bytes())
    );
    assert!(
        first
            .appearance_two_da
            .windows(b"m2a_m6p01".len())
            .any(|w| w == b"m2a_m6p01")
    );

    let mdl_readback = inspect_binary_mdl(&first.model).expect("binary MDL own readback");
    assert_eq!(mdl_readback.node_tree.roots[0].name, M6_MODEL_RESREF);
    assert_eq!(mdl_readback.animations.len(), 7);
    assert_eq!(
        mdl_readback
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "cpause1",
            "cappear",
            "cwalk",
            "crun",
            "ca1slashl",
            "cdamagel",
            "cdead",
        ]
    );
    assert!(
        mdl_readback
            .animations
            .iter()
            .all(|animation| animation.animation_root == M6_MODEL_RESREF)
    );
    let cpause1 = mdl_readback
        .animations
        .iter()
        .find(|animation| animation.name == "cpause1")
        .expect("exact cpause1 readback");
    assert!(cpause1.node_tree.node_count >= 1);
    assert!(
        has_decoded_motion_controller(&serde_json::to_value(cpause1).unwrap()),
        "cpause1 own-readback must contain a decoded changing position controller"
    );
    let deformation = evaluate_skin_deformation_v1(&mdl_readback, "cpause1", cpause1.length)
        .expect("owned M6 output must be evaluable as one complete SkinMesh motion sample");
    assert_eq!(deformation.skin_count, 1);
    assert_eq!(
        deformation.vertex_count,
        first.report.geometry.vertex_count as u32
    );
    assert!(deformation.moved_vertex_count > 0);
    assert!(deformation.max_displacement.is_finite() && deformation.max_displacement > 0.0);

    let archive = ErfArchive::parse(&first.hak).expect("generated HAK readback");
    assert_eq!(archive.resources().len(), 3);
    assert_eq!(archive.find(M6_MODEL_RESREF, 2002).unwrap(), first.model);
    assert_eq!(archive.find(M6_TEXTURE_RESREF, 3).unwrap(), first.texture);
    assert_eq!(
        archive.find("appearance", 2017).unwrap(),
        first.appearance_two_da
    );

    let parsed_manifest: m2a_core::model_pipeline::M6MaterializationManifestV1 =
        serde_json::from_slice(&first.manifest_json).unwrap();
    assert_eq!(parsed_manifest, first.manifest);
    assert_eq!(
        parsed_manifest.package_manifest.package_sha256,
        first.summary.outputs.hak.sha256
    );
    assert_eq!(parsed_manifest.appended_physical_row, 1);
    assert_eq!(parsed_manifest.generated_files.len(), 8);
    assert_eq!(
        first.summary.outputs.proof_module.byte_length,
        first.proof_module.len() as u64
    );
    assert_eq!(
        first.summary.outputs.proof_module.sha256,
        hex_sha256(&first.proof_module)
    );
    assert_eq!(
        parsed_manifest.appearance_payload_policy,
        "PRESERVED_AND_APPENDED"
    );
    let summary_text = String::from_utf8(first.summary_json.clone()).unwrap();
    assert!(summary_text.contains("zeroReferenceModelPayloadCopied"));
    assert!(!summary_text.contains("zeroRetailPayloadCopied"));
    for file in &parsed_manifest.generated_files {
        let bytes = match file.relative_path.as_str() {
            "generated/source.glb" => first.source_glb.as_slice(),
            "generated/m2a_m6p01.mdl" => first.model.as_slice(),
            "generated/m2a_m6t01.tga" => first.texture.as_slice(),
            "generated/appearance.2da" => first.appearance_two_da.as_slice(),
            "generated/m2a_codex_aproof.hak" => first.hak.as_slice(),
            "generated/m2a_codex_aproof.mod" => first.proof_module.as_slice(),
            "reports/materialization-report.json" => first.report_json.as_slice(),
            "reports/summary.json" => first.summary_json.as_slice(),
            path => panic!("unexpected manifest path {path}"),
        };
        assert_eq!(file.byte_length, bytes.len() as u64);
        assert_eq!(file.sha256, hex_sha256(bytes));
    }
}

#[test]
fn static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice() {
    let source = static_meshy_m0_fixture();
    let runtime_profile = m0_runtime_profile(&source);
    let first = build_meshy_m0_static_rigid_package_with_profile_v2(
        &source,
        &appearance_fixture(),
        &runtime_profile,
    )
    .expect("static Meshy M0 package");
    let second = build_meshy_m0_static_rigid_package_with_profile_v2(
        &source,
        &appearance_fixture(),
        &runtime_profile,
    )
    .expect("deterministic static Meshy M0 package");

    assert_eq!(first.model, second.model);
    assert_eq!(first.texture, second.texture);
    assert_eq!(first.appearance_two_da, second.appearance_two_da);
    assert_eq!(first.hak, second.hak);
    assert_eq!(first.proof_module, second.proof_module);
    assert_eq!(
        first.summary.status,
        "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED"
    );
    assert_eq!(first.summary.model_resref, M0_MODEL_RESREF);
    assert_eq!(first.summary.texture_resref, M0_TEXTURE_RESREF);
    assert_eq!(first.summary.animation.output_name, "cpause1");
    assert!(!first.summary.animation.has_motion);
    assert_eq!(first.report.geometry.output_segment_deformation, "RIGID");
    assert_eq!(first.report.geometry.active_joint_count, 0);
    assert!(
        first
            .appearance_two_da
            .windows(M0_APPEARANCE_LABEL.len())
            .any(|value| value == M0_APPEARANCE_LABEL.as_bytes())
    );
    assert_eq!(
        first.summary.appearance_payload_policy,
        "AURORA_VISIBLE_PREFIX_AND_ONE_ROW_APPENDED"
    );
    assert_eq!(first.report.proof_module.module_resref, "m2a_bm0p1");
    assert_eq!(first.report.proof_module.area_resref, "m2a_bm0a1");
    assert_eq!(first.report.proof_module.creature_resref, "nw_dwarfmerc001");

    let runtime_contract = first
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("isolated M0 package must emit a runtime fixture contract");
    assert_eq!(
        first.report.m0_runtime_fixture_contract.as_ref(),
        Some(runtime_contract),
        "the detailed report and summary must bind the same M0 packet"
    );
    assert_eq!(
        first.manifest.m0_runtime_fixture_contract.as_ref(),
        Some(runtime_contract),
        "the portable manifest must carry the same capture binding"
    );
    assert_eq!(runtime_contract.schema_version, 2);
    assert_eq!(runtime_contract.lane, "M0_BINARY_VERTICAL_SLICE_V2");
    assert_eq!(runtime_contract.runtime_profile.schema_version, 2);
    assert_eq!(
        runtime_contract
            .source_topology_binding
            .topology_summary
            .nodes
            .len(),
        1
    );
    assert_eq!(
        runtime_contract
            .source_topology_binding
            .topology_summary
            .ignored_node_count,
        0
    );
    assert_eq!(
        runtime_contract.engine_envelope.verdict,
        DirectCreatureEngineEnvelopeVerdictV1::StructuralPassRuntimeNotWitnessed
    );
    assert_eq!(
        runtime_contract.engine_envelope_sha256,
        direct_creature_engine_envelope_digest_v1(&runtime_contract.engine_envelope)
            .expect("canonical engine-envelope digest")
    );
    assert_eq!(
        runtime_contract.module.sha256,
        hex_sha256(&first.proof_module)
    );
    assert_eq!(runtime_contract.hak.sha256, hex_sha256(&first.hak));
    assert_eq!(runtime_contract.model.sha256, hex_sha256(&first.model));
    assert_eq!(runtime_contract.texture.sha256, hex_sha256(&first.texture));
    assert_eq!(
        runtime_contract.state_projection_profile,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
    );
    assert_eq!(runtime_contract.state_projection_provenance, None);
    let state_readback = inspect_binary_mdl(&first.model).expect("contract-bound M0 model");
    let recomputed_state_summary = verify_direct_creature_state_projection_v1(
        &state_readback,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect("contract-bound M0 model must pass Retail state projection");
    assert_eq!(
        runtime_contract.state_projection_summary,
        recomputed_state_summary
    );
    assert_eq!(
        runtime_contract.state_projection_summary_sha256,
        direct_creature_structural_summary_digest_v1(&recomputed_state_summary)
            .expect("state summary digest")
    );
    let mut wrong_projection_profile = runtime_contract.clone();
    wrong_projection_profile.state_projection_profile =
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1;
    wrong_projection_profile.state_projection_provenance = Some(cep_r3_provenance());
    let error = verify_test_m0_runtime_contract_v2(
        &wrong_projection_profile,
        &first.proof_module,
        &first.hak,
    )
    .expect_err("M0 contract must not switch state-projection families");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-STATE-PROJECTION-PROFILE");

    let mut wrong_projection_digest = runtime_contract.clone();
    wrong_projection_digest.state_projection_summary_sha256 = "0".repeat(64);
    let error = verify_test_m0_runtime_contract_v2(
        &wrong_projection_digest,
        &first.proof_module,
        &first.hak,
    )
    .expect_err("M0 contract must bind the recomputed state-projection digest");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.stateProjectionSummary");
    assert_eq!(
        runtime_contract.appearance_two_da.sha256,
        hex_sha256(&first.appearance_two_da)
    );
    assert_eq!(runtime_contract.appearance.label, M0_APPEARANCE_LABEL);
    assert_eq!(runtime_contract.appearance.model_type, "S");
    assert_eq!(runtime_contract.appearance.race, M0_MODEL_RESREF);
    assert_eq!(
        runtime_contract.binary_scene.entry_position.x,
        M0_RUNTIME_ENTRY_X
    );
    assert_eq!(
        runtime_contract.binary_scene.entry_position.y,
        M0_RUNTIME_ENTRY_Y
    );
    assert_eq!(
        runtime_contract.binary_scene.entry_direction.x,
        M0_RUNTIME_ENTRY_DIR_X
    );
    assert_eq!(
        runtime_contract.binary_scene.entry_direction.y,
        M0_RUNTIME_ENTRY_DIR_Y
    );
    assert_eq!(
        runtime_contract.binary_scene.fixture.position.x,
        M0_RUNTIME_FIXTURE_X
    );
    assert_eq!(
        runtime_contract.binary_scene.fixture.position.y,
        M0_RUNTIME_FIXTURE_Y
    );
    assert!(runtime_contract.mesh_eligibility.eligible);
    assert_eq!(runtime_contract.mesh_eligibility.checked_mesh_count, 1);
    assert_eq!(runtime_contract.mesh_eligibility.eligible_mesh_count, 1);
    assert_eq!(runtime_contract.mesh_eligibility.mesh_types, [3]);
    assert_eq!(
        runtime_contract.mesh_eligibility.texture_resrefs,
        [M0_TEXTURE_RESREF]
    );
    assert_eq!(
        runtime_contract.mesh_eligibility.rule,
        "render == 1 && meshType == 3 && vertexCount > 0 && faces.nonEmpty && faceIndicesMatchSingleRawIndexStream && positions.len == vertexCount"
    );
    let error =
        verify_test_m0_runtime_contract_v2(runtime_contract, &first.proof_module, &first.hak)
            .expect_err("a Toolset-only table slice must fail the full-runtime profile validator");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-SCOPE-NOT-RUNTIME");

    let mut wrong_fixture_contract = runtime_contract.clone();
    wrong_fixture_contract.binary_scene.fixture.position.x += 0.25;
    let error = verify_test_m0_runtime_contract_v2(
        &wrong_fixture_contract,
        &first.proof_module,
        &first.hak,
    )
    .expect_err("one altered GIT coordinate must invalidate the capture binding");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.binaryScene");

    let mut wrong_appearance_contract = runtime_contract.clone();
    wrong_appearance_contract.appearance.race = "not_m0".to_owned();
    let error = verify_test_m0_runtime_contract_v2(
        &wrong_appearance_contract,
        &first.proof_module,
        &first.hak,
    )
    .expect_err("one altered appearance RACE must invalidate the capture binding");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.appearance");

    let mut wrong_hak_contract = runtime_contract.clone();
    wrong_hak_contract.hak.sha256 = "0".repeat(64);
    let error =
        verify_test_m0_runtime_contract_v2(&wrong_hak_contract, &first.proof_module, &first.hak)
            .expect_err("one altered HAK hash must invalidate the capture binding");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.hak");

    let ineligible_model = mutate_m0_direct_mesh_u32(&first.model, 0xdc, 0);
    let ineligible_eligibility = inspect_m0_runtime_mesh_eligibility_v1(&ineligible_model)
        .expect("render-off M0 remains structurally readable");
    assert!(!ineligible_eligibility.eligible);
    let ineligible_hak = rewrite_m0_hak(
        &first,
        ineligible_model.clone(),
        first.appearance_two_da.clone(),
    );
    let mut self_consistent_ineligible_contract = runtime_contract.clone();
    self_consistent_ineligible_contract.model =
        runtime_resource(M0_MODEL_RESREF, &ineligible_model);
    self_consistent_ineligible_contract.hak =
        runtime_resource(&runtime_contract.hak.resref, &ineligible_hak);
    self_consistent_ineligible_contract.mesh_eligibility = ineligible_eligibility;
    let error = verify_test_m0_runtime_contract_v2(
        &self_consistent_ineligible_contract,
        &first.proof_module,
        &ineligible_hak,
    )
    .expect_err("a self-consistent ineligible mesh must not receive a runtime contract");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.engineEnvelope");

    let model_type_p_appearance = replace_exactly_once(
        &first.appearance_two_da,
        b"M2A_M0_MESHY_RIGID NORM S m2a_m0p01",
        b"M2A_M0_MESHY_RIGID NORM P m2a_m0p01",
    );
    let model_type_p_hak =
        rewrite_m0_hak(&first, first.model.clone(), model_type_p_appearance.clone());
    let mut self_consistent_model_type_p_contract = runtime_contract.clone();
    self_consistent_model_type_p_contract.appearance.model_type = "P".to_owned();
    self_consistent_model_type_p_contract.appearance_two_da =
        runtime_resource("appearance", &model_type_p_appearance);
    self_consistent_model_type_p_contract.hak =
        runtime_resource(&runtime_contract.hak.resref, &model_type_p_hak);
    let error = verify_test_m0_runtime_contract_v2(
        &self_consistent_model_type_p_contract,
        &first.proof_module,
        &model_type_p_hak,
    )
    .expect_err("a self-consistent composite appearance must not pass as direct M0");
    assert_eq!(
        error.code,
        "M0-RUNTIME-CONTRACT-APPEARANCE-MODELTYPE-INVALID"
    );
    assert_eq!(error.path, "appearance.rows[1].MODELTYPE");

    let wrong_race_appearance = replace_exactly_once(
        &first.appearance_two_da,
        b"M2A_M0_MESHY_RIGID NORM S m2a_m0p01",
        b"M2A_M0_MESHY_RIGID NORM S invalidm0",
    );
    let wrong_race_hak = rewrite_m0_hak(&first, first.model.clone(), wrong_race_appearance.clone());
    let mut self_consistent_wrong_race_contract = runtime_contract.clone();
    self_consistent_wrong_race_contract.appearance.race = "invalidm0".to_owned();
    self_consistent_wrong_race_contract.appearance_two_da =
        runtime_resource("appearance", &wrong_race_appearance);
    self_consistent_wrong_race_contract.hak =
        runtime_resource(&runtime_contract.hak.resref, &wrong_race_hak);
    let error = verify_test_m0_runtime_contract_v2(
        &self_consistent_wrong_race_contract,
        &first.proof_module,
        &wrong_race_hak,
    )
    .expect_err("a self-consistent foreign RACE must not pass as direct M0");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-APPEARANCE-RACE-MISMATCH");
    assert_eq!(error.path, "appearance.rows[1].RACE");

    let wrong_texture_model =
        replace_exactly_once(&first.model, M0_TEXTURE_RESREF.as_bytes(), b"m2a_bad01");
    let wrong_texture_eligibility = inspect_m0_runtime_mesh_eligibility_v1(&wrong_texture_model)
        .expect("a texture-resref-only mutation remains structurally readable");
    assert!(wrong_texture_eligibility.eligible);
    let wrong_texture_hak = rewrite_m0_hak(
        &first,
        wrong_texture_model.clone(),
        first.appearance_two_da.clone(),
    );
    let mut self_consistent_wrong_texture_contract = runtime_contract.clone();
    self_consistent_wrong_texture_contract.model =
        runtime_resource(M0_MODEL_RESREF, &wrong_texture_model);
    self_consistent_wrong_texture_contract.hak =
        runtime_resource(&runtime_contract.hak.resref, &wrong_texture_hak);
    self_consistent_wrong_texture_contract.mesh_eligibility = wrong_texture_eligibility;
    let error = verify_test_m0_runtime_contract_v2(
        &self_consistent_wrong_texture_contract,
        &first.proof_module,
        &wrong_texture_hak,
    )
    .expect_err("a model pointing at an unbound texture must not receive an M0 runtime contract");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.engineEnvelope");

    let readback = inspect_binary_mdl(&first.model).expect("M0 binary MDL readback");
    assert_eq!(readback.node_tree.roots[0].name, M0_MODEL_RESREF);
    assert_eq!(
        first.report.model.format_profile,
        MdlFormatProfileV1::M0StaticRigidNativeV1
    );
    let m0_mesh = readback.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("M0 rigid child must be a mesh");
    assert_eq!(
        m0_mesh.mesh_type, 3,
        "M0 uses the native direct-creature mesh mode observed in c_Direwolf"
    );
    assert!(
        m0_mesh.validated_raw_pointers.iter().any(|pointer| {
            pointer.field == "colors"
                && pointer.pointer.is_some()
                && pointer.validated_length == m0_mesh.vertex_count * 4
        }),
        "M0 emits the native direct-creature RGBA vertex-color stream"
    );
    assert_eq!(
        m0_base_semantic_digest(&readback),
        "d50b63de1142b48972e4b6a95bd414df352a00de294b6ca23581184a0a1731e2",
        "the r29 animation-only delta must preserve the complete base-model semantics"
    );
    assert_eq!(readback.animations.len(), 7);
    assert!(readback.animations.iter().all(|animation| {
        let absolute = 12 + animation.offset as usize;
        animation.runtime_68 == 0
            && animation.animation_type == 5
            && animation.animation_type_padding == [0, 0, 0]
            && first.model[absolute + 0x6d..absolute + 0x70] == [0, 0, 0]
    }));
    assert!(
        readback
            .animations
            .iter()
            .all(|animation| animation.animation_root == M0_MODEL_RESREF)
    );
    assert_eq!(
        readback
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "ca1slashl",
            "cappear",
            "cdead",
            "cdamagel",
            "cpause1",
            "crun",
            "cwalk",
        ]
        .into_iter()
        .collect(),
        "r29 must neither add nor remove a direct-creature state"
    );
    for animation in &readback.animations {
        let identity_controllers = &animation.node_tree.roots[0].controllers;
        assert_eq!(
            identity_controllers.len(),
            2,
            "{} must carry both identity root streams",
            animation.name
        );
        assert_eq!(identity_controllers[0].controller_type, 8);
        assert_eq!(identity_controllers[0].times, [0.0, 1.0]);
        assert_eq!(
            identity_controllers[0].values,
            [vec![0.0, 0.0, 0.0], vec![0.0, 0.0, 0.0]]
        );
        assert_eq!(identity_controllers[1].controller_type, 20);
        assert_eq!(identity_controllers[1].times, [0.0, 1.0]);
        assert_eq!(
            identity_controllers[1].values,
            [vec![0.0, 0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0, 1.0]]
        );
        assert!(
            animation.events.is_empty(),
            "{} must not gain an event stream",
            animation.name
        );
    }
    assert_eq!(
        readback
            .animations
            .iter()
            .filter(|animation| animation.name != "cpause1")
            .map(|animation| animation.node_tree.roots[0].controllers.len())
            .sum::<usize>(),
        12,
        "the delta is exactly two streams on each of the six non-cpause1 clips"
    );
    assert!(
        readback.animations.iter().all(|animation| {
            animation.node_tree.node_count == 2
                && animation.node_tree.roots.len() == 1
                && animation.node_tree.roots[0].name == M0_MODEL_RESREF
                && animation.node_tree.roots[0].children.len() == 1
                && animation.node_tree.roots[0].children[0].name == "m2a_seg_1"
                && animation.node_tree.roots[0].children[0].number == 1
                && animation.node_tree.roots[0].children[0].content_flags == 0x01
                && animation.node_tree.roots[0].children[0]
                    .controllers
                    .is_empty()
                && animation.node_tree.roots[0].children[0].mesh.is_none()
                && animation.node_tree.roots[0].children[0].skin.is_none()
        }),
        "every static M0 animation mirrors the base mesh identity as a retail 0x01 dummy"
    );
    let summary = verify_direct_creature_state_projection_v1(
        &readback,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect("generated M0 must pass the retail state-projection oracle");
    assert_eq!(summary.base_node_count, 2);
    assert_eq!(summary.base_mesh_node_count, 1);
    assert_eq!(summary.animation_count, 7);
    assert_eq!(summary.full_base_topology_projection_count, 7);
    assert_eq!(summary.all_generic_dummy_projection_count, 7);

    let mut missing_leaf = readback.clone();
    missing_leaf.animations[0].node_tree.roots[0]
        .children
        .clear();
    let error = verify_direct_creature_state_projection_v1(
        &missing_leaf,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect_err("a missing state leaf must fail conformance");
    assert_eq!(error.code, "M2A-MDL-CONFORMANCE-STATE-NODE-MISSING");

    for (label, mutation) in [("name", 0u8), ("part", 1u8), ("parent", 2u8)] {
        let mut corrupted = readback.clone();
        match mutation {
            0 => corrupted.animations[0].node_tree.roots[0].children[0]
                .name
                .push_str("_wrong"),
            1 => corrupted.animations[0].node_tree.roots[0].children[0].number += 10,
            2 => {
                let child = corrupted.animations[0].node_tree.roots[0]
                    .children
                    .remove(0);
                corrupted.animations[0].node_tree.roots.push(child);
            }
            _ => unreachable!(),
        }
        let error = verify_direct_creature_state_projection_v1(
            &corrupted,
            MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.code, "M2A-MDL-CONFORMANCE-STATE-IDENTITY",
            "wrong {label} must fail the identity projection"
        );
    }

    let mut wrong_family = readback.clone();
    wrong_family.animations[0].node_tree.roots[0].children[0].content_flags = 0x21;
    let error = verify_direct_creature_state_projection_v1(
        &wrong_family,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect_err("a CEP placeholder flag must not enter the retail family");
    assert_eq!(error.code, "M2A-MDL-CONFORMANCE-PROFILE-MIXED");

    let mut payload_in_dummy = readback.clone();
    payload_in_dummy.animations[0].node_tree.roots[0].children[0].mesh = Some(m0_mesh.clone());
    let error = verify_direct_creature_state_projection_v1(
        &payload_in_dummy,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect_err("a retail dummy must never carry base mesh payload");
    assert_eq!(error.code, "M2A-MDL-CONFORMANCE-RETAIL-DUMMY-PAYLOAD");

    let mut wrong_type = readback.clone();
    wrong_type.animations[0].animation_type = 0;
    let error = verify_direct_creature_state_projection_v1(
        &wrong_type,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect_err("the retail family requires type 5");
    assert_eq!(error.code, "M2A-MDL-CONFORMANCE-ANIMATION-TYPE");

    let actual_cep_provenance = cep_r3_provenance();
    let expected_cep_provenance = cep_r3_provenance();
    let error = verify_direct_creature_state_projection_with_expected_provenance_v1(
        &readback,
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1,
        Some(&actual_cep_provenance),
        Some(&expected_cep_provenance),
    )
    .expect_err("the oracle must reject profile mixing");
    assert_eq!(error.code, "M2A-MDL-CONFORMANCE-PROFILE-MIXED");

    let archive = ErfArchive::parse(&first.hak).expect("M0 HAK readback");
    assert_eq!(archive.resources().len(), 3);
    assert_eq!(archive.find(M0_MODEL_RESREF, 2002).unwrap(), first.model);
    assert_eq!(archive.find(M0_TEXTURE_RESREF, 3).unwrap(), first.texture);
    assert_eq!(
        archive.find("appearance", 2017).unwrap(),
        first.appearance_two_da
    );
}

#[test]
fn exact_r29_adds_only_six_identity_root_stream_pairs_to_the_r28_model() {
    let repo_root = canonical_workspace::canonical_repository_root();
    let r28_model = fs::read(
        repo_root.join("proof-output/m0-r27-animation-type5-20260721/generated/m2a_m0p01.mdl"),
    )
    .expect("exact byte-identical r28 model source");
    let r29_model = fs::read(
        repo_root.join("proof-output/m0-r29-all-state-identity-20260721/generated/m2a_m0p01.mdl"),
    )
    .expect("exact r29 model");
    assert_eq!(
        hex_sha256(&r28_model),
        "bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578"
    );
    assert_eq!(
        hex_sha256(&r29_model),
        "a69fc655d5975b866215f17b142089fa29c0c1f7ea8286db8c0f5136f9e890e7"
    );

    let r28 = inspect_binary_mdl(&r28_model).expect("r28 MDL readback");
    let r29 = inspect_binary_mdl(&r29_model).expect("r29 MDL readback");
    let r28_base_digest = m0_base_semantic_digest(&r28);
    assert_eq!(r28_base_digest, m0_base_semantic_digest(&r29));
    assert_eq!(
        r28_base_digest,
        "f4172130a98ff6b3b4f3c9e67e9b915d0b9da72f7dff6293eabed51e369e7b6c"
    );
    assert_eq!(r28.animations.len(), 7);
    assert_eq!(r29.animations.len(), 7);
    assert_eq!(
        r28.animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>(),
        r29.animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(r28.animations.iter().all(|animation| {
        animation.animation_type == 5
            && animation.node_tree.node_count == 2
            && animation.node_tree.roots[0].children[0].content_flags == 0x21
    }));
    assert!(r29.animations.iter().all(|animation| {
        animation.animation_type == 5
            && animation.node_tree.node_count == 2
            && animation.node_tree.roots[0].children[0].content_flags == 0x21
    }));
    assert_eq!(
        r28.animations
            .iter()
            .map(|animation| animation.node_tree.roots[0].controllers.len())
            .sum::<usize>(),
        2,
        "r28 has identity root streams only on cpause1"
    );
    assert_eq!(
        r29.animations
            .iter()
            .map(|animation| animation.node_tree.roots[0].controllers.len())
            .sum::<usize>(),
        14,
        "r29 has exactly two identity root streams on all seven clips"
    );
    for (before, after) in r28.animations.iter().zip(&r29.animations) {
        assert_eq!(before.name, after.name);
        assert_eq!(before.animation_type, after.animation_type);
        assert_eq!(before.animation_root, after.animation_root);
        assert_eq!(before.length, after.length);
        assert_eq!(before.transition, after.transition);
        assert_eq!(before.events, after.events);
        assert!(after.node_tree.roots[0].children[0].controllers.is_empty());
        if before.name == "cpause1" {
            assert_eq!(before.node_tree.roots[0].controllers.len(), 2);
        } else {
            assert!(before.node_tree.roots[0].controllers.is_empty());
        }
        assert_eq!(after.node_tree.roots[0].controllers.len(), 2);
    }
}

#[test]
fn retail_state_projection_preserves_exact_r29_mesh_material_texture_and_raw_mdx() {
    let repo_root = canonical_workspace::canonical_repository_root();
    let lineage = repo_root.join("proof-output/m0-r29-all-state-identity-20260721/generated");
    let source = fs::read(lineage.join("source.glb")).expect("preserved owned Meshy source GLB");
    let r29_model = fs::read(lineage.join("m2a_m0p01.mdl")).expect("preserved r29 model");
    let r29_texture = fs::read(lineage.join("m2a_m0t01.tga")).expect("preserved r29 texture");
    let rebuilt = build_meshy_m0_static_rigid_package_v1(&source, &appearance_fixture())
        .expect("retail state-projection build from the exact owned source");

    let before = inspect_binary_mdl(&r29_model).expect("r29 readback");
    let after = inspect_binary_mdl(&rebuilt.model).expect("retail projection readback");
    assert_eq!(
        m0_base_semantic_digest(&before),
        m0_base_semantic_digest(&after),
        "state projection must not alter any base model/mesh/material semantic"
    );
    assert_eq!(
        raw_mdx_bytes(&r29_model),
        raw_mdx_bytes(&rebuilt.model),
        "state projection must preserve the exact raw MDX geometry streams"
    );
    assert_eq!(rebuilt.texture, r29_texture, "TGA bytes must remain exact");
    assert_eq!(
        after.node_tree.roots[0].children[0]
            .mesh
            .as_ref()
            .expect("M0 base mesh")
            .textures[0],
        M0_TEXTURE_RESREF
    );
    verify_direct_creature_state_projection_v1(
        &after,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect("new output must pass the retail conformance oracle");
}

#[test]
fn m0_runtime_mesh_eligibility_requires_triangle_topology_and_nonempty_faces() {
    let artifact =
        build_meshy_m0_static_rigid_package_v1(&static_meshy_m0_fixture(), &appearance_fixture())
            .expect("static Meshy M0 package");

    for (label, model) in [
        (
            "non-triangle mesh type",
            mutate_m0_direct_mesh_u32(&artifact.model, 0x224, 0),
        ),
        (
            "empty face array",
            mutate_m0_direct_mesh_array_count(&artifact.model, 0x78, 0),
        ),
        (
            "raw index stream differs from face topology",
            mutate_m0_direct_mesh_first_raw_index(&artifact.model),
        ),
    ] {
        let eligibility = inspect_m0_runtime_mesh_eligibility_v1(&model)
            .unwrap_or_else(|error| panic!("{label} mutation must remain readable: {error:?}"));
        assert!(
            !eligibility.eligible,
            "{label} must be rejected by the M0 direct-creature renderability gate"
        );
    }
}

#[test]
fn static_meshy_m0_is_grounded_when_source_origin_is_below_its_mesh() {
    let artifact = build_meshy_m0_static_rigid_package_v1(
        &static_meshy_m0_fixture_below_ground(),
        &appearance_fixture(),
    )
    .expect("grounded static Meshy M0 package");
    let readback = inspect_binary_mdl(&artifact.model).expect("grounded M0 MDL readback");
    let mesh = readback.node_tree.roots[0].children[0]
        .mesh
        .as_ref()
        .expect("grounded M0 child must be a mesh");

    assert!(
        mesh.bounds_min.z.abs() <= 1.0e-5,
        "M0 mesh bottom must align to the creature placement Z, got {}",
        mesh.bounds_min.z
    );
    assert!(
        mesh.bounds_max.z > 1.0,
        "grounded M0 must retain positive height"
    );
}

#[test]
fn static_meshy_m0_uses_the_toolset_visible_appearance_prefix() {
    let source_appearance = appearance_fixture_with_toolset_invisible_tail();
    let artifact =
        build_meshy_m0_static_rigid_package_v1(&static_meshy_m0_fixture(), &source_appearance)
            .expect("M0 package with an Aurora-invisible appearance tail");

    assert_eq!(artifact.report.appearance.appended_row_index, 848);
    assert_eq!(artifact.report.proof_module.appearance_row, 848);
    assert_eq!(
        artifact.summary.appearance_payload_policy,
        "AURORA_VISIBLE_PREFIX_AND_ONE_ROW_APPENDED"
    );
    let inspection = inspect_two_da_v2(&artifact.appearance_two_da, &TwoDaLimitsV1::default())
        .expect("normalized M0 appearance.2da readback");
    assert_eq!(inspection.physical_row_count, 849);
    assert_eq!(
        artifact.summary.m0_appearance_table.as_ref().unwrap().scope,
        m2a_core::model_pipeline::M0AppearanceTableScopeV1::IsolatedToolsetVerticalSlice
    );
    assert_eq!(
        artifact
            .summary
            .m0_appearance_table
            .as_ref()
            .unwrap()
            .input_physical_rows,
        848
    );
    assert_eq!(
        artifact
            .summary
            .m0_appearance_table
            .as_ref()
            .unwrap()
            .output_physical_rows,
        849
    );
    let error = m2a_core::model_pipeline::require_m0_full_runtime_appearance_table_v1(
        artifact.summary.m0_appearance_table.as_ref().unwrap(),
    )
    .expect_err("an isolated Toolset table must not be distributable as general runtime data");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-SCOPE-NOT-RUNTIME");
    assert!(
        artifact
            .appearance_two_da
            .windows(M0_APPEARANCE_LABEL.len())
            .any(|window| { window == M0_APPEARANCE_LABEL.as_bytes() })
    );
    assert!(
        !artifact
            .appearance_two_da
            .windows(b"OS_RESERVED_848".len())
            .any(|window| window == b"OS_RESERVED_848")
    );
}

#[test]
fn static_meshy_m0_materializes_in_the_one_canonical_runtime_package() {
    let source = static_meshy_m0_fixture();
    let runtime_profile = m0_runtime_profile(&source);
    let source_appearance = appearance_fixture_with_toolset_invisible_tail();
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: PROOF_MODULE_RESREF.to_owned(),
        area_resref: PROOF_AREA_RESREF.to_owned(),
        hak_resref: PROOF_HAK_RESREF.to_owned(),
    };
    let first = build_meshy_m0_canonical_runtime_package_with_profile_v2(
        &source,
        &source_appearance,
        &runtime_profile,
    )
    .expect("canonical static Meshy M0 package");
    let second = build_meshy_m0_canonical_runtime_package_with_profile_v2(
        &source,
        &source_appearance,
        &runtime_profile,
    )
    .expect("deterministic canonical static Meshy M0 package");

    assert_eq!(first.hak, second.hak);
    assert_eq!(first.proof_module, second.proof_module);
    assert_eq!(
        first.summary.status,
        "M0_MESHY_STATIC_RIGID_CANONICAL_PACKAGE_MATERIALIZED"
    );
    assert_eq!(first.summary.model_resref, M0_MODEL_RESREF);
    assert_eq!(first.report.proof_module.module_resref, PROOF_MODULE_RESREF);
    assert_eq!(first.report.proof_module.area_resref, PROOF_AREA_RESREF);
    assert_eq!(
        first.report.proof_module.creature_resref,
        M0_CANONICAL_PROOF_CREATURE_RESREF
    );
    assert_eq!(first.report.proof_module.hak_resref, PROOF_HAK_RESREF);
    assert_eq!(first.report.proof_module.resource_count, 6);
    assert_eq!(
        first.summary.appearance_payload_policy,
        "FULL_RUNTIME_TABLE_AND_ONE_ROW_APPENDED"
    );
    assert_eq!(first.report.appearance.appended_row_index, 15_105);
    assert_eq!(first.report.proof_module.appearance_row, 15_105);
    assert_eq!(first.report.appearance.physical_rows_before, 15_105);
    assert_eq!(first.report.appearance.physical_rows_after, 15_106);
    assert!(first.report.appearance.source_prefix_preserved);
    assert!(first.appearance_two_da.starts_with(&source_appearance));
    let table = first
        .summary
        .m0_appearance_table
        .as_ref()
        .expect("canonical M0 must persist its runtime table scope");
    assert_eq!(
        table.scope,
        m2a_core::model_pipeline::M0AppearanceTableScopeV1::FullRuntimeAppendV1
    );
    assert_eq!(table.input_physical_rows, 15_105);
    assert_eq!(table.output_physical_rows, 15_106);
    assert_eq!(
        first.manifest.m0_appearance_table.as_ref(),
        first.summary.m0_appearance_table.as_ref()
    );
    m2a_core::model_pipeline::require_m0_full_runtime_appearance_table_v1(table)
        .expect("canonical M0 must pass the general-runtime table gate");
    let runtime_contract = first
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("canonical M0 must bind a full-runtime fixture contract");
    assert_eq!(&runtime_contract.appearance_table, table);
    verify_test_m0_runtime_contract_v2(runtime_contract, &first.proof_module, &first.hak)
        .expect("canonical M0 contract must verify the full table prefix and fixture row");

    let repo_root = canonical_workspace::canonical_repository_root();
    let mixed_model = fs::read(
        repo_root.join("proof-output/m0-r29-all-state-identity-20260721/generated/m2a_m0p01.mdl"),
    )
    .expect("preserved r29 0x21 state-tree adversarial fixture");
    let mixed_readback = inspect_binary_mdl(&mixed_model).expect("r29 mixed-family readback");
    assert!(
        mixed_readback
            .animations
            .iter()
            .all(|animation| { animation.node_tree.roots[0].children[0].content_flags == 0x21 })
    );
    let mixed_hak = rewrite_m0_hak(&first, mixed_model.clone(), first.appearance_two_da.clone());
    let mut self_consistent_mixed_contract = runtime_contract.clone();
    self_consistent_mixed_contract.model = runtime_resource(M0_MODEL_RESREF, &mixed_model);
    self_consistent_mixed_contract.hak = runtime_resource(&runtime_contract.hak.resref, &mixed_hak);
    self_consistent_mixed_contract.mesh_eligibility =
        inspect_m0_runtime_mesh_eligibility_v1(&mixed_model)
            .expect("r29 base mesh remains eligible");
    self_consistent_mixed_contract.state_projection_summary =
        summarize_direct_creature_structure_v1(&mixed_readback);
    self_consistent_mixed_contract.state_projection_summary_sha256 =
        direct_creature_structural_summary_digest_v1(
            &self_consistent_mixed_contract.state_projection_summary,
        )
        .expect("recomputed adversarial state summary digest");
    let error = verify_test_m0_runtime_contract_v2(
        &self_consistent_mixed_contract,
        &first.proof_module,
        &mixed_hak,
    )
    .expect_err(
        "recomputed MOD/HAK/MDL/mesh/summary identities must not admit a 0x21 tree under Retail",
    );
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.engineEnvelope");

    let mut isolated = table.clone();
    isolated.scope =
        m2a_core::model_pipeline::M0AppearanceTableScopeV1::IsolatedToolsetVerticalSlice;
    let error = verify_m0_full_runtime_appearance_table_binding_v1(
        &isolated,
        &first.appearance_two_da,
        first.report.proof_module.appearance_row,
    )
    .expect_err("an isolated table must not pass a runtime-profile validator");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-SCOPE-NOT-RUNTIME");

    let mut wrong_rows = table.clone();
    wrong_rows.output_physical_rows += 1;
    let error = verify_m0_full_runtime_appearance_table_binding_v1(
        &wrong_rows,
        &first.appearance_two_da,
        first.report.proof_module.appearance_row,
    )
    .expect_err("a runtime table must append exactly one physical row");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-ROW-COUNT-DIFF");

    let error = verify_m0_full_runtime_appearance_table_binding_v1(
        table,
        &first.appearance_two_da,
        first.report.proof_module.appearance_row - 1,
    )
    .expect_err("the fixture must bind the exact appended physical row");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-PHYSICAL-ROW-MISMATCH");

    let mut changed_prefix = first.appearance_two_da.clone();
    changed_prefix[0] ^= 1;
    let mut self_consistent_output = table.clone();
    self_consistent_output.output_sha256 = hex_sha256(&changed_prefix);
    let error = verify_m0_full_runtime_appearance_table_binding_v1(
        &self_consistent_output,
        &changed_prefix,
        first.report.proof_module.appearance_row,
    )
    .expect_err("the complete input prefix must remain byte-identical");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-PREFIX-MISMATCH");

    let truncated = &first.appearance_two_da[..first.appearance_two_da.len() - 1];
    let error = verify_m0_full_runtime_appearance_table_binding_v1(
        table,
        truncated,
        first.report.proof_module.appearance_row,
    )
    .expect_err("a truncated runtime table must fail its output binding");
    assert_eq!(error.code, "M0-APPEARANCE-TABLE-OUTPUT-MISMATCH");
    assert!(
        !first
            .appearance_two_da
            .windows(M0_CONTROL_APPEARANCE_LABEL.len())
            .any(|value| value == M0_CONTROL_APPEARANCE_LABEL.as_bytes())
    );
    assert!(
        first
            .appearance_two_da
            .windows(M0_APPEARANCE_LABEL.len())
            .any(|value| value == M0_APPEARANCE_LABEL.as_bytes())
    );

    let archive = ErfArchive::parse(&first.hak).expect("canonical M0 HAK readback");
    assert_eq!(archive.resources().len(), 3);
    assert_eq!(archive.find(M0_MODEL_RESREF, 2002).unwrap(), first.model);
    assert_eq!(archive.find(M0_TEXTURE_RESREF, 3).unwrap(), first.texture);

    let path = temp_path("m0-canonical-write");
    let _ = fs::remove_dir_all(&path);
    write_m0_canonical_runtime_proof_packet_with_profile_v2(
        &path,
        &first,
        &identity,
        &runtime_profile,
        &source,
    )
    .expect("canonical M0 proof packet");
    assert_eq!(
        fs::read(path.join(format!("generated/{M6_HAK_FILE_NAME}"))).unwrap(),
        first.hak
    );
    assert_eq!(
        fs::read(path.join(format!("generated/{M6_PROOF_MODULE_FILE_NAME}"))).unwrap(),
        first.proof_module
    );
    assert!(!path.join("generated/m2a_m0_proof.hak").exists());
    assert!(!path.join("generated/m2a_m0_proof.mod").exists());
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn canonical_m0_runtime_package_accepts_one_fresh_caller_owned_identity() {
    let source = static_meshy_m0_fixture();
    let runtime_profile = m0_runtime_profile(&source);
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_testmod1".to_owned(),
        area_resref: "m2a_testarea1".to_owned(),
        hak_resref: "m2a_testhak1".to_owned(),
    };
    let artifact = build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2(
        &source,
        &appearance_fixture_with_toolset_invisible_tail(),
        &identity,
        &runtime_profile,
    )
    .expect("fresh-identity canonical M0 runtime package");
    let contract = artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("fresh runtime fixture contract");

    assert_eq!(
        artifact.report.proof_module.module_resref,
        identity.module_resref
    );
    assert_eq!(
        artifact.report.proof_module.area_resref,
        identity.area_resref
    );
    assert_eq!(artifact.report.proof_module.hak_resref, identity.hak_resref);
    assert_eq!(contract.binary_scene.module_resref, identity.module_resref);
    assert_eq!(contract.binary_scene.area_resref, identity.area_resref);
    assert_eq!(
        contract.binary_scene.ordered_hak_resrefs,
        std::slice::from_ref(&identity.hak_resref)
    );
    assert_eq!(
        contract.state_projection_profile,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
    );
    assert!(contract.state_projection_provenance.is_none());
    assert_eq!(
        contract.appearance_table.scope,
        m2a_core::model_pipeline::M0AppearanceTableScopeV1::FullRuntimeAppendV1
    );
    assert_eq!(
        contract.appearance_table.output_physical_rows,
        contract.appearance_table.input_physical_rows + 1
    );
    verify_test_m0_runtime_contract_v2(contract, &artifact.proof_module, &artifact.hak)
        .expect("fresh identity must pass the production runtime verifier");

    let path = temp_path("m0-canonical-fresh-identity-write");
    let _ = fs::remove_dir_all(&path);
    write_m0_canonical_runtime_proof_packet_with_profile_v2(
        &path,
        &artifact,
        &identity,
        &runtime_profile,
        &source,
    )
    .expect("fresh identity packet");
    assert!(
        path.join(format!("generated/{}.mod", identity.module_resref))
            .is_file()
    );
    assert!(
        path.join(format!("generated/{}.hak", identity.hak_resref))
            .is_file()
    );
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn legacy_m0_construction_paths_cannot_obtain_v2_runtime_admission() {
    let source = static_meshy_m0_fixture();
    let appearance = appearance_fixture_with_toolset_invisible_tail();
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_legacy01".to_owned(),
        area_resref: "m2a_legacya1".to_owned(),
        hak_resref: "m2a_legacyh1".to_owned(),
    };
    let error = build_meshy_m0_canonical_runtime_package_v1(&source, &appearance)
        .expect_err("public legacy canonical runtime builder must be executable-dead");
    assert_eq!(error.code, "M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN");
    let error =
        build_meshy_m0_canonical_runtime_package_with_identity_v1(&source, &appearance, &identity)
            .expect_err("public legacy caller-identity runtime builder must be executable-dead");
    assert_eq!(error.code, "M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN");

    let artifact = build_meshy_m0_static_rigid_package_v1(&source, &appearance)
        .expect("legacy isolated artifact remains offline-only");
    assert!(artifact.report.m0_runtime_fixture_contract.is_none());
    assert!(artifact.summary.m0_runtime_fixture_contract.is_none());
    assert!(artifact.manifest.m0_runtime_fixture_contract.is_none());
    let output = temp_path("legacy-canonical-emission-rejected");
    let _ = fs::remove_dir_all(&output);
    let error = write_m0_canonical_proof_packet_v1(&output, &artifact)
        .expect_err("legacy canonical writer must reject before directory emission");
    assert_eq!(error.code, "M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN");
    assert!(!output.exists());

    let profile = m0_runtime_profile(&source);
    let error = write_m0_canonical_runtime_proof_packet_with_profile_v2(
        &output, &artifact, &identity, &profile, &source,
    )
    .expect_err("legacy isolated construction must never become a V2 runtime packet");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISSING");
    assert!(!output.exists());
}

#[test]
fn v2_a_source_topology_trust_root_rejects_self_consistent_regenerated_source_and_mapping_drift() {
    let source = static_meshy_m0_fixture();
    let appearance = appearance_fixture_with_toolset_invisible_tail();
    let profile = m0_runtime_profile(&source);
    let artifact =
        build_meshy_m0_canonical_runtime_package_with_profile_v2(&source, &appearance, &profile)
            .expect("explicit V2 source/profile build");
    let contract = artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("V2 runtime contract");
    verify_m0_binary_runtime_fixture_contract_v2(
        contract,
        &profile,
        &source,
        &artifact.proof_module,
        &artifact.hak,
    )
    .expect("unaltered caller-owned trust root");

    let mut forged_mapping = contract.clone();
    forged_mapping.source_topology_binding.selected.node_id = 77;
    forged_mapping
        .source_topology_binding
        .topology_summary_sha256 =
        source_topology_summary_digest_v1(&forged_mapping.source_topology_binding.topology_summary)
            .expect("rehashed forged topology summary");
    let error = verify_m0_binary_runtime_fixture_contract_v2(
        &forged_mapping,
        &profile,
        &source,
        &artifact.proof_module,
        &artifact.hak,
    )
    .expect_err("recomputed JSON identity cannot replace source/MDL readback");
    assert_eq!(error.code, "M2A-SOURCE-TOPOLOGY-BINDING-MISMATCH");

    let changed_source = mutate_glb(source.clone(), |root| {
        root["nodes"][0]["name"] = serde_json::json!("self-consistent-other-lineage");
    });
    let changed_profile = m0_runtime_profile(&changed_source);
    let changed_artifact = build_meshy_m0_canonical_runtime_package_with_profile_v2(
        &changed_source,
        &appearance,
        &changed_profile,
    )
    .expect("fully regenerated internally consistent other lineage");
    let changed_contract = changed_artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("other-lineage V2 runtime contract");
    verify_m0_binary_runtime_fixture_contract_v2(
        changed_contract,
        &changed_profile,
        &changed_source,
        &changed_artifact.proof_module,
        &changed_artifact.hak,
    )
    .expect("other lineage is internally consistent");
    let error = verify_m0_binary_runtime_fixture_contract_v2(
        changed_contract,
        &profile,
        &source,
        &changed_artifact.proof_module,
        &changed_artifact.hak,
    )
    .expect_err("a regenerated source and contract cannot cross the caller-owned trust root");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.runtimeProfile");

    let ignored_node_source = mutate_glb(source, |root| {
        root["nodes"]
            .as_array_mut()
            .expect("fixture nodes")
            .push(serde_json::json!({ "name": "unreachable-owned-node" }));
    });
    let ignored_profile = m0_runtime_profile(&ignored_node_source);
    let error = build_meshy_m0_canonical_runtime_package_with_profile_v2(
        &ignored_node_source,
        &appearance,
        &ignored_profile,
    )
    .expect_err("M0 runtime admission rejects ignored source topology");
    assert_eq!(error.code, "M2A-SOURCE-TOPOLOGY-M0-SHAPE");
}

#[test]
fn v2_a_table_driven_scene_root_name_child_mesh_primitive_skin_attribute_and_transform_attacks_fail()
 {
    let source = static_meshy_m0_fixture();
    let appearance = appearance_fixture_with_toolset_invisible_tail();
    let original_profile = m0_runtime_profile(&source);
    let cases = [
        "multiple-scenes",
        "root-multiplicity-order",
        "null-name-drift",
        "duplicate-name-unreachable",
        "children-and-reachable-node",
        "mesh-multiplicity",
        "primitive-multiplicity",
        "skin-joint-attributes",
        "attribute-set-drift",
        "transform-drift",
    ];
    for case in cases {
        let changed = mutate_glb(source.clone(), |root| match case {
            "multiple-scenes" => {
                let scene = root["scenes"][0].clone();
                root["scenes"].as_array_mut().unwrap().push(scene);
            }
            "root-multiplicity-order" => root["scenes"][0]["nodes"] = serde_json::json!([0, 0]),
            "null-name-drift" => root["nodes"][0]["name"] = serde_json::Value::Null,
            "duplicate-name-unreachable" => root["nodes"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({ "name": "m0-source-root" })),
            "children-and-reachable-node" => {
                root["nodes"][0]["children"] = serde_json::json!([1]);
                root["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({ "name": "owned-child" }));
            }
            "mesh-multiplicity" => {
                let mesh = root["meshes"][0].clone();
                root["meshes"].as_array_mut().unwrap().push(mesh);
            }
            "primitive-multiplicity" => {
                let primitive = root["meshes"][0]["primitives"][0].clone();
                root["meshes"][0]["primitives"]
                    .as_array_mut()
                    .unwrap()
                    .push(primitive);
            }
            "skin-joint-attributes" => {
                root["nodes"][0]["skin"] = serde_json::json!(0);
                root["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({ "name": "owned-joint" }));
                root["skins"] = serde_json::json!([{
                    "name": "owned-rig",
                    "joints": [1],
                    "skeleton": 1,
                    "inverseBindMatrices": 6
                }]);
                root["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"] = serde_json::json!(4);
                root["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"] =
                    serde_json::json!(5);
            }
            "attribute-set-drift" => {
                root["meshes"][0]["primitives"][0]["attributes"]
                    .as_object_mut()
                    .unwrap()
                    .remove("NORMAL");
            }
            "transform-drift" => {
                root["nodes"][0]["translation"] = serde_json::json!([1.0, 0.0, 0.0]);
            }
            _ => unreachable!(),
        });
        let changed_profile = m0_runtime_profile(&changed);
        let self_consistent = build_meshy_m0_canonical_runtime_package_with_profile_v2(
            &changed,
            &appearance,
            &changed_profile,
        );
        if matches!(case, "null-name-drift" | "transform-drift") {
            // These source features are representable and fully summarized,
            // but cannot cross the independently declared original trust root.
            let error = build_meshy_m0_canonical_runtime_package_with_profile_v2(
                &changed,
                &appearance,
                &original_profile,
            )
            .expect_err("source identity drift must fail before construction");
            assert_eq!(error.code, "M2A-DIRECT-CREATURE-SOURCE-TRUST-ROOT-MISMATCH");
        } else {
            let error = self_consistent
                .expect_err("M0 exact one-root/mesh/primitive/unskinned shape must reject attack");
            assert!(
                !error.code.is_empty(),
                "attack {case} returned unexpected error: {error:?}"
            );
        }
    }
}

#[test]
fn v2_b_engine_envelope_replay_rejects_self_consistent_rehashed_mdl_hak_and_swapped_envelope() {
    let source = static_meshy_m0_fixture();
    let appearance = appearance_fixture_with_toolset_invisible_tail();
    let profile = m0_runtime_profile(&source);
    let artifact =
        build_meshy_m0_canonical_runtime_package_with_profile_v2(&source, &appearance, &profile)
            .expect("explicit V2 source/profile build");
    let contract = artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("V2 runtime contract");

    let mutated_model = mutate_m0_direct_mesh_u32(&artifact.model, 0xdc, 0);
    let mutated_hak = rewrite_m0_hak(
        &artifact,
        mutated_model.clone(),
        artifact.appearance_two_da.clone(),
    );
    let mut regenerated = contract.clone();
    regenerated.model = runtime_resource(M0_MODEL_RESREF, &mutated_model);
    regenerated.hak = runtime_resource(&contract.binary_scene.ordered_hak_resrefs[0], &mutated_hak);
    regenerated.mesh_eligibility =
        inspect_m0_runtime_mesh_eligibility_v1(&mutated_model).expect("mutated mesh readback");
    regenerated.source_topology_binding =
        inspect_m0_source_topology_binding_v1(&source, &mutated_model, &profile)
            .expect("self-consistent mutated source mapping");
    (
        regenerated.engine_envelope,
        regenerated.engine_envelope_sha256,
    ) = inspect_direct_creature_engine_envelope_v1(&mutated_model)
        .expect("self-consistent mutated engine envelope");
    let error = verify_m0_binary_runtime_fixture_contract_v2(
        &regenerated,
        &profile,
        &source,
        &artifact.proof_module,
        &mutated_hak,
    )
    .expect_err("a fully rehashed external MDL/HAK cannot replace deterministic replay");
    assert_eq!(
        error.code,
        "M0-RUNTIME-CONTRACT-DETERMINISTIC-REPLAY-MISMATCH"
    );
    assert_eq!(error.path, "hak.model");

    let mut swapped = contract.clone();
    swapped.engine_envelope.fog ^= 1;
    swapped.engine_envelope_sha256 =
        direct_creature_engine_envelope_digest_v1(&swapped.engine_envelope)
            .expect("rehashed swapped envelope");
    let error = verify_m0_binary_runtime_fixture_contract_v2(
        &swapped,
        &profile,
        &source,
        &artifact.proof_module,
        &artifact.hak,
    )
    .expect_err("an internally rehashed envelope cannot replace exact HAK readback");
    assert_eq!(error.code, "M0-RUNTIME-CONTRACT-MISMATCH");
    assert_eq!(error.path, "contract.engineEnvelope");
}

#[test]
fn v2_b_actual_ordered_hak_bytes_reject_unknown0_start_mdx_alias_and_trailing_before_replay() {
    let source = static_meshy_m0_fixture();
    let appearance = appearance_fixture_with_toolset_invisible_tail();
    let profile = m0_runtime_profile(&source);
    let artifact =
        build_meshy_m0_canonical_runtime_package_with_profile_v2(&source, &appearance, &profile)
            .expect("explicit V2 source/profile build");
    let contract = artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .expect("V2 runtime contract");
    let mesh = m0_direct_mesh_offset(&artifact.model);
    let uv0_pointer = u32_at(&artifact.model, mesh + 0x234);
    let mut trailing = artifact.model.clone();
    trailing.push(0xa5);
    let attacks = [
        (
            "unknown0-present",
            mutate_m0_direct_mesh_u32(&artifact.model, 0x21c, 0),
            "M2A-MDL-ENGINE-ENVELOPE-RAW-MARKER",
        ),
        (
            "start-mdx-non-origin",
            mutate_m0_direct_mesh_u32(&artifact.model, 0x228, 1),
            "M2A-MDL-ENGINE-ENVELOPE-RAW-MARKER",
        ),
        (
            "vertices-alias-uv0",
            mutate_m0_direct_mesh_u32(&artifact.model, 0x22c, uv0_pointer),
            "M2A-MDL-ENGINE-ENVELOPE-RAW-ALIAS",
        ),
        ("trailing-byte", trailing, "M2A-MDL-ENGINE-ENVELOPE-PARSE"),
    ];
    for (name, mutated_model, expected_code) in attacks {
        let mutated_hak = rewrite_m0_hak(
            &artifact,
            mutated_model.clone(),
            artifact.appearance_two_da.clone(),
        );
        let mut rehashed = contract.clone();
        rehashed.model = runtime_resource(M0_MODEL_RESREF, &mutated_model);
        rehashed.hak =
            runtime_resource(&contract.binary_scene.ordered_hak_resrefs[0], &mutated_hak);
        let error = verify_m0_binary_runtime_fixture_contract_v2(
            &rehashed,
            &profile,
            &source,
            &artifact.proof_module,
            &mutated_hak,
        )
        .expect_err("actual ordered-HAK raw-byte corruption must fail admission");
        assert_eq!(error.code, expected_code, "attack {name}");
        assert_ne!(
            error.code, "M0-RUNTIME-CONTRACT-DETERMINISTIC-REPLAY-MISMATCH",
            "attack {name} must fail independently before replay"
        );
    }
}

#[test]
fn meshy_h1_rigid_runtime_diagnostic_preserves_geometry_but_emits_no_skin_node() {
    let source = synthetic_owned_m6_glb_v1().expect("owned GLB fixture");
    let artifact =
        build_meshy_h1_rigid_runtime_diagnostic_package_v1(&source, &appearance_fixture())
            .expect("rigid runtime diagnostic package");

    assert_eq!(artifact.report.geometry.output_segment_deformation, "RIGID");
    assert_eq!(artifact.report.geometry.active_joint_count, 0);
    assert!(artifact.report.geometry.vertex_count > 0);
    assert!(artifact.report.geometry.triangle_count > 0);

    let readback = inspect_binary_mdl(&artifact.model).expect("binary MDL own readback");
    let readback_json = serde_json::to_value(readback).expect("serializable readback");
    let mut mesh_nodes = Vec::new();
    for root in readback_json["nodeTree"]["roots"]
        .as_array()
        .expect("readback roots")
    {
        collect_mesh_nodes(root, &mut mesh_nodes);
    }
    assert_eq!(mesh_nodes.len(), 1);
    assert!(mesh_nodes[0]["skin"].is_null());
    assert_eq!(
        mesh_nodes[0]["mesh"]["faces"].as_array().unwrap().len(),
        artifact.report.geometry.triangle_count
    );
}

#[test]
fn appearance_without_any_phenotype_alias_clones_all_thirty_five_runtime_cells() {
    let source = synthetic_owned_m6_glb_v1().unwrap();
    let base = appearance_fixture_without_phenotype();
    let artifact = build_m6_model_package_v1(&source, &base).unwrap();

    assert_eq!(artifact.report.appearance.appended_row_index, 1);
    assert_eq!(artifact.report.appearance.changed_cells.len(), 35);
    assert!(artifact.appearance_two_da.starts_with(&base));
    assert_eq!(
        artifact.summary.appearance_payload_policy,
        "PRESERVED_AND_APPENDED"
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the exact read-only Last City appearance table"]
fn exact_last_city_direct_s_donor_is_cloned_full_width_with_only_label_and_race_changed() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 for the in-place reference test"
    );
    let table_path = canonical_workspace::canonical_repository_root().join(
        "proof-output/lc-hd-animals-c-squirrel-reference-audit-2026-07-18/source-copies/appearance.2da",
    );
    let table = fs::read(&table_path).expect("exact read-only Last City appearance table");
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &table).unwrap();
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(&table, &limits).unwrap();
    assert_eq!(inspection.columns.len(), 35);
    let donor = read_two_da_row_v2(&table, 102, &limits).unwrap();
    let appended = read_two_da_row_v2(
        &artifact.appearance_two_da,
        u32::from(artifact.report.appearance.appended_row_index),
        &limits,
    )
    .unwrap();
    assert_eq!(artifact.report.appearance.changed_cells.len(), 35);
    for (index, column) in inspection.columns.iter().enumerate() {
        match column.as_str() {
            "LABEL" => assert_eq!(
                appended.cells[index],
                m2a_core::two_da::TwoDaCellValueV1::Text {
                    value: M6_APPEARANCE_LABEL.to_owned()
                }
            ),
            "RACE" => assert_eq!(
                appended.cells[index],
                m2a_core::two_da::TwoDaCellValueV1::Text {
                    value: M6_MODEL_RESREF.to_owned()
                }
            ),
            _ => assert_eq!(
                appended.cells[index], donor.cells[index],
                "column {column} must remain the exact stock direct-S donor value"
            ),
        }
    }
}

#[test]
fn invalid_or_incomplete_appearance_is_a_stable_error_and_never_panics() {
    let glb = synthetic_owned_m6_glb_v1().unwrap();
    for bytes in [
        b"not a 2da".as_slice(),
        b"2DA V2.0\n\nLABEL RACE\n0 Existing existing\n".as_slice(),
    ] {
        let result = catch_unwind(AssertUnwindSafe(|| build_m6_model_package_v1(&glb, bytes)));
        let error = result
            .expect("invalid appearance must not panic")
            .unwrap_err();
        assert_eq!(error.stage, "APPEARANCE");
        assert!(!error.code.is_empty());
    }
}

#[test]
fn missing_animation_base_color_and_ineligible_profile_are_stable_and_panic_free() {
    let appearance = appearance_fixture();
    let missing_animation = mutate_glb(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["animations"] = serde_json::json!([]);
    });
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_m6_model_package_v1(&missing_animation, &appearance)
    }));
    let error = result
        .expect("missing animation must not panic")
        .unwrap_err();
    assert_eq!(error.stage, "ANIMATION");
    assert!(error.code.starts_with("M4A-") || error.code == "M6-ANIMATION-MISSING");

    let missing_texture = mutate_glb(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["materials"][0]["pbrMetallicRoughness"]
            .as_object_mut()
            .unwrap()
            .remove("baseColorTexture");
    });
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_m6_model_package_v1(&missing_texture, &appearance)
    }));
    let error = result.expect("missing texture must not panic").unwrap_err();
    assert_eq!(
        (error.stage.as_str(), error.code.as_str()),
        ("TEXTURE", "M6-BASE-COLOR-TEXTURE-MISSING")
    );

    let mut rig = synthetic_owned_m6_rig_v1().unwrap();
    for weights in &mut rig.segments[0].reference_weights {
        for influence in weights {
            influence.value = 0.0;
        }
    }
    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_m6_model_package_with_profile_v1(
            &synthetic_owned_m6_glb_v1().unwrap(),
            &appearance,
            &rig,
            &synthetic_owned_m6_animation_mapping_v1(),
        )
    }));
    let error = result
        .expect("ineligible profile must not panic")
        .unwrap_err();
    assert_eq!(
        (error.stage.as_str(), error.code.as_str()),
        ("PROFILE", "M6-PROFILE-INELIGIBLE")
    );
}

#[test]
fn proof_packet_refuses_nonempty_output_and_never_overwrites_it() {
    let path = temp_path("m6-collision");
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("keep.txt"), b"owned by caller").unwrap();
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &appearance_fixture())
            .unwrap();

    let result = catch_unwind(AssertUnwindSafe(|| {
        write_m6_proof_packet_v1(&path, &artifact)
    }));
    let error = result.expect("collision must not panic").unwrap_err();
    assert_eq!(error.code, "M6-OUTPUT-EXISTS");
    assert_eq!(fs::read(path.join("keep.txt")).unwrap(), b"owned by caller");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn proof_packet_writes_generated_reports_and_empty_live_directories() {
    let path = temp_path("m6-write");
    let _ = fs::remove_dir_all(&path);
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &appearance_fixture())
            .unwrap();
    write_m6_proof_packet_v1(&path, &artifact).unwrap();

    assert_eq!(
        fs::read(path.join("generated/m2a_m6p01.mdl")).unwrap(),
        artifact.model
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_m6t01.tga")).unwrap(),
        artifact.texture
    );
    assert_eq!(
        fs::read(path.join("generated/appearance.2da")).unwrap(),
        artifact.appearance_two_da
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_codex_aproof.hak")).unwrap(),
        artifact.hak
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_codex_aproof.mod")).unwrap(),
        artifact.proof_module
    );
    assert_eq!(
        fs::read(path.join("reports/materialization-manifest.json")).unwrap(),
        artifact.manifest_json
    );
    assert!(path.join("live").is_dir());
    assert_eq!(fs::read_dir(path.join("live")).unwrap().count(), 0);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn static_m0_proof_packet_writes_only_its_own_runtime_names() {
    let path = temp_path("m0-write");
    let _ = fs::remove_dir_all(&path);
    let artifact =
        build_meshy_m0_static_rigid_package_v1(&static_meshy_m0_fixture(), &appearance_fixture())
            .unwrap();

    write_m0_proof_packet_v1(&path, &artifact).unwrap();

    assert_eq!(
        fs::read(path.join("generated/m2a_m0p01.mdl")).unwrap(),
        artifact.model
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_m0t01.tga")).unwrap(),
        artifact.texture
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_m0_proof.hak")).unwrap(),
        artifact.hak
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_bm0p1.mod")).unwrap(),
        artifact.proof_module
    );
    assert!(!path.join("generated/m2a_m6p01.mdl").exists());
    assert!(!path.join("generated/m2a_codex_aproof.hak").exists());
    assert!(path.join("live").is_dir());
    assert_eq!(fs::read_dir(path.join("live")).unwrap().count(), 0);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn proof_packet_never_deletes_a_preexisting_staging_directory() {
    let output = temp_path("m6-stage-owner");
    let staging = output.parent().unwrap().join(format!(
        ".{}.m2a-stage-{}",
        output.file_name().unwrap().to_string_lossy(),
        std::process::id(),
    ));
    let _ = fs::remove_dir_all(&output);
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("keep.txt"), b"caller staging").unwrap();
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &appearance_fixture())
            .unwrap();
    let error = write_m6_proof_packet_v1(&output, &artifact).unwrap_err();
    assert_eq!(error.code, "M6-STAGING-EXISTS");
    assert_eq!(
        fs::read(staging.join("keep.txt")).unwrap(),
        b"caller staging"
    );
    assert!(!output.exists());
    fs::remove_dir_all(staging).unwrap();
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn m0_base_semantic_digest(readback: &m2a_core::InspectionReport) -> String {
    let root = &readback.node_tree.roots[0];
    let mesh_node = &root.children[0];
    let mesh = mesh_node.mesh.as_ref().expect("M0 base mesh");
    let root_controllers = root
        .controllers
        .iter()
        .map(|controller| {
            serde_json::json!({
                "controllerType": controller.controller_type,
                "controllerName": controller.controller_name,
                "packedByte": controller.packed_byte,
                "interpolationFlags": controller.interpolation_flags,
                "decoded": controller.decoded,
                "paddingByte": controller.padding_byte,
                "rowCount": controller.row_count,
                "timeIndex": controller.time_index,
                "dataIndex": controller.data_index,
                "columnCount": controller.column_count,
                "times": controller.times,
                "values": controller.values,
            })
        })
        .collect::<Vec<_>>();
    let mesh_controllers = mesh_node
        .controllers
        .iter()
        .map(|controller| {
            serde_json::json!({
                "controllerType": controller.controller_type,
                "controllerName": controller.controller_name,
                "packedByte": controller.packed_byte,
                "interpolationFlags": controller.interpolation_flags,
                "decoded": controller.decoded,
                "paddingByte": controller.padding_byte,
                "rowCount": controller.row_count,
                "timeIndex": controller.time_index,
                "dataIndex": controller.data_index,
                "columnCount": controller.column_count,
                "times": controller.times,
                "values": controller.values,
            })
        })
        .collect::<Vec<_>>();
    let semantic = serde_json::json!({
        "model": {
            "name": readback.model.name,
            "geometryType": readback.model.geometry_type,
            "classification": readback.model.classification,
            "fog": readback.model.fog,
            "childModelCount": readback.model.child_model_count,
            "boundsMin": readback.model.bounds_min,
            "boundsMax": readback.model.bounds_max,
            "radius": readback.model.radius,
            "animationScale": readback.model.animation_scale,
            "supermodelName": readback.model.supermodel_name,
        },
        "root": {
            "number": root.number,
            "name": root.name,
            "inheritColor": root.inherit_color,
            "contentFlags": root.content_flags,
            "controllers": root_controllers,
        },
        "meshNode": {
            "number": mesh_node.number,
            "name": mesh_node.name,
            "inheritColor": mesh_node.inherit_color,
            "contentFlags": mesh_node.content_flags,
            "controllers": mesh_controllers,
            "mesh": {
                "textures": mesh.textures,
                "vertexCount": mesh.vertex_count,
                "textureCount": mesh.texture_count,
                "boundsMin": mesh.bounds_min,
                "boundsMax": mesh.bounds_max,
                "radius": mesh.radius,
                "average": mesh.average,
                "diffuse": mesh.diffuse,
                "ambient": mesh.ambient,
                "specular": mesh.specular,
                "shininess": mesh.shininess,
                "shadow": mesh.shadow,
                "beaming": mesh.beaming,
                "render": mesh.render,
                "transparency": mesh.transparency,
                "renderHint": mesh.render_hint,
                "tileFade": mesh.tile_fade,
                "meshType": mesh.mesh_type,
                "startMdx": mesh.start_mdx,
                "faces": mesh.faces,
                "indexCounts": mesh.index_counts,
                "rawIndices": mesh.raw_indices,
                "vertices": mesh.vertices,
                "uv0": mesh.uv0,
                "normals": mesh.normals,
                "vertexColors": mesh.vertex_colors,
            }
        }
    });
    hex_sha256(&serde_json::to_vec(&semantic).expect("M0 base semantics serialize"))
}

fn runtime_resource(resref: &str, bytes: &[u8]) -> M0RuntimeResourceBindingV1 {
    M0RuntimeResourceBindingV1 {
        resref: resref.to_owned(),
        byte_length: bytes.len() as u64,
        sha256: hex_sha256(bytes),
    }
}

// Test-only compatibility helper: production exposes no V1 verifier. The
// expected profile is declared independently from exact source bytes; the
// embedded contract profile is never reused as the trust root.
fn verify_test_m0_runtime_contract_v2(
    contract: &m2a_core::model_pipeline::M0RuntimeFixtureContractV2,
    module: &[u8],
    hak: &[u8],
) -> Result<(), m2a_core::model_pipeline::M6PipelineErrorV1> {
    let source = static_meshy_m0_fixture();
    let expected_runtime_profile = m0_runtime_profile(&source);
    verify_m0_binary_runtime_fixture_contract_v2(
        contract,
        &expected_runtime_profile,
        &source,
        module,
        hak,
    )
}

fn m0_runtime_profile(source: &[u8]) -> DirectCreatureRuntimeProfileV2 {
    declare_m0_direct_creature_runtime_profile_v2(
        source,
        "test://owned/static-m0-source.glb",
        SourceTopologyOriginV1::UserDeclared,
        M0_MODEL_RESREF,
    )
    .expect("explicit test-owned M0 runtime profile")
}

fn cep_r3_provenance() -> MdlStateProjectionProvenanceV1 {
    MdlStateProjectionProvenanceV1 {
        schema_version: 1,
        source_family: "CEP3_CORE1_R3_RIGID_PLACEHOLDER".to_owned(),
        container_sha256: "6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a"
            .to_owned(),
        resource_resref: "c_phod_horror_b".to_owned(),
        resource_sha256: "62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a"
            .to_owned(),
    }
}

fn rewrite_m0_hak(
    artifact: &m2a_core::model_pipeline::M6ModelPackageArtifactV1,
    model: Vec<u8>,
    appearance: Vec<u8>,
) -> Vec<u8> {
    let archive = ErfArchive::parse(&artifact.hak).expect("M0 HAK readback");
    write_hak_v1(
        &[
            HakResourceInputV1 {
                resref: M0_MODEL_RESREF.to_owned(),
                resource_type: 2002,
                payload: model,
            },
            HakResourceInputV1 {
                resref: M0_TEXTURE_RESREF.to_owned(),
                resource_type: 3,
                payload: archive
                    .find(M0_TEXTURE_RESREF, 3)
                    .expect("M0 texture")
                    .to_vec(),
            },
            HakResourceInputV1 {
                resref: "appearance".to_owned(),
                resource_type: 2017,
                payload: appearance,
            },
        ],
        &HakWriterOptionsV1::default(),
    )
    .expect("rewritten M0 HAK")
    .payload
}

fn replace_exactly_once(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(
        needle.len(),
        replacement.len(),
        "mutation must preserve binary offsets"
    );
    let first = bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("test fixture must contain the exact mutation source");
    assert!(
        bytes[first + needle.len()..]
            .windows(needle.len())
            .all(|window| window != needle),
        "test fixture must contain the mutation source exactly once"
    );
    let mut output = bytes.to_vec();
    output[first..first + needle.len()].copy_from_slice(replacement);
    output
}

fn mutate_m0_direct_mesh_u32(model: &[u8], field_offset: usize, value: u32) -> Vec<u8> {
    let mut output = model.to_vec();
    let offset = m0_direct_mesh_offset(&output) + field_offset;
    output[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    output
}

fn mutate_m0_direct_mesh_array_count(model: &[u8], field_offset: usize, value: u32) -> Vec<u8> {
    let mut output = model.to_vec();
    let offset = m0_direct_mesh_offset(&output) + field_offset;
    output[offset + 4..offset + 8].copy_from_slice(&value.to_le_bytes());
    output[offset + 8..offset + 12].copy_from_slice(&value.to_le_bytes());
    output
}

fn mutate_m0_direct_mesh_first_raw_index(model: &[u8]) -> Vec<u8> {
    let mut output = model.to_vec();
    let core = 12usize;
    let mesh = m0_direct_mesh_offset(&output);
    let raw_index_offsets = u32_at(&output, mesh + 0x210) as usize;
    let raw_index_offset = i32::from_le_bytes(
        output[core + raw_index_offsets..core + raw_index_offsets + 4]
            .try_into()
            .expect("raw index offset"),
    );
    assert!(raw_index_offset >= 0, "M0 raw index stream must be present");
    let vertex_count = u16::from_le_bytes(
        output[mesh + 0x230..mesh + 0x232]
            .try_into()
            .expect("vertex count"),
    );
    assert!(vertex_count > 1, "fixture needs two valid index values");
    let raw = core + u32_at(&output, 4) as usize + raw_index_offset as usize;
    let current = u16::from_le_bytes(output[raw..raw + 2].try_into().expect("first raw index"));
    let replacement = (current + 1) % vertex_count;
    output[raw..raw + 2].copy_from_slice(&replacement.to_le_bytes());
    output
}

fn m0_direct_mesh_offset(model: &[u8]) -> usize {
    let core = 12usize;
    let root = u32_at(model, core + 0x48) as usize;
    let children = u32_at(model, core + root + 0x48) as usize;
    core + u32_at(model, core + children) as usize
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 slice"))
}

fn raw_mdx_bytes(model: &[u8]) -> &[u8] {
    let start = 12 + u32_at(model, 4) as usize;
    let length = u32_at(model, 8) as usize;
    &model[start..start + length]
}

fn collect_mesh_nodes<'a>(node: &'a Value, output: &mut Vec<&'a Value>) {
    if !node["mesh"].is_null() {
        output.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect_mesh_nodes(child, output);
    }
}

fn mutate_glb(mut glb: Vec<u8>, mutation: impl FnOnce(&mut Value)) -> Vec<u8> {
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let json_end = 20 + json_length;
    let mut root: Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
    let bin = glb.split_off(json_end + 8);
    mutation(&mut root);
    let mut json = serde_json::to_vec(&root).unwrap();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let length = 12 + 8 + json.len() + 8 + bin.len();
    let mut output = Vec::with_capacity(length);
    output.extend_from_slice(b"glTF");
    output.extend_from_slice(&2_u32.to_le_bytes());
    output.extend_from_slice(&(length as u32).to_le_bytes());
    output.extend_from_slice(&(json.len() as u32).to_le_bytes());
    output.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    output.extend_from_slice(&json);
    output.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    output.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
    output.extend_from_slice(&bin);
    output
}

fn has_decoded_motion_controller(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            let is_motion = object.get("decoded") == Some(&Value::Bool(true))
                && object.get("controllerName").and_then(Value::as_str) == Some("position")
                && object
                    .get("times")
                    .and_then(Value::as_array)
                    .is_some_and(|times| times.len() >= 2)
                && object
                    .get("values")
                    .and_then(Value::as_array)
                    .is_some_and(|values| {
                        values.len() >= 2 && values.windows(2).any(|pair| pair[0] != pair[1])
                    });
            is_motion || object.values().any(has_decoded_motion_controller)
        }
        Value::Array(values) => values.iter().any(has_decoded_motion_controller),
        _ => false,
    }
}
