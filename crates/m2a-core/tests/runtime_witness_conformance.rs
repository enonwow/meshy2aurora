use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use m2a_core::mdl::{
    DirectCreatureEnvelopeDifferenceClassV1, MdlStateProjectionProfileV1,
    MdlStateProjectionProvenanceV1, direct_creature_envelope_differential_v1, inspect_binary_mdl,
    inspect_direct_creature_envelope_projection_v1,
    summarize_direct_creature_state_skin_profile_v1, summarize_direct_creature_structure_v1,
    verify_direct_creature_state_projection_v1,
    verify_direct_creature_state_projection_with_expected_provenance_v1,
};
use m2a_core::{
    direct_creature_animation::{
        COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1,
        evaluate_direct_creature_animation_behavior_v2,
    },
    direct_creature_contract::SourceTopologyOriginV1,
    erf::ErfArchive,
    hierarchy_experiment::{
        build_source_topology_rigid_experiment_v3,
        declare_source_topology_rigid_experiment_profile_v3,
    },
    model_pipeline::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    runtime_evidence::{
        RuntimeModelVisibilityV1, RuntimeProofCompletenessV1, RuntimeRenderIntegrityV1,
        RuntimeWitnessObservationV1, RuntimeWitnessUseV1, classify_runtime_witness_use_v1,
    },
};
use sha2::{Digest, Sha256};

fn assert_native_root_part_zero(report: &m2a_core::mdl::InspectionReport, witness: &str) {
    assert_eq!(
        report.node_tree.roots.len(),
        1,
        "{witness} must have one base root"
    );
    assert_eq!(
        report.node_tree.roots[0].number, 0,
        "{witness} base root must use native part 0"
    );
    assert!(
        report.animations.iter().all(|animation| {
            animation.node_tree.roots.len() == 1 && animation.node_tree.roots[0].number == 0
        }),
        "{witness} local animation roots must use the same native part 0 identity"
    );
}

#[test]
fn a_corrupt_draw_witness_is_never_a_correct_creature_baseline() {
    let h1_v20 = RuntimeWitnessObservationV1 {
        exact_identity_bound: true,
        model_visibility: RuntimeModelVisibilityV1::Visible,
        proof_completeness: RuntimeProofCompletenessV1::Verified,
        render_integrity: RuntimeRenderIntegrityV1::Corrupt,
    };
    assert_eq!(
        classify_runtime_witness_use_v1(&h1_v20),
        RuntimeWitnessUseV1::DrawPathOnly,
        "H1 v20 proves only that NWN issued a draw; its deformed blob cannot qualify layout, geometry, transforms, animation, or creature correctness",
    );

    let retail_correct = RuntimeWitnessObservationV1 {
        render_integrity: RuntimeRenderIntegrityV1::Correct,
        ..h1_v20
    };
    assert_eq!(
        classify_runtime_witness_use_v1(&retail_correct),
        RuntimeWitnessUseV1::CorrectCreatureBaseline,
    );

    for incomplete in [
        RuntimeWitnessObservationV1 {
            exact_identity_bound: false,
            ..retail_correct
        },
        RuntimeWitnessObservationV1 {
            proof_completeness: RuntimeProofCompletenessV1::Missing,
            ..retail_correct
        },
        RuntimeWitnessObservationV1 {
            model_visibility: RuntimeModelVisibilityV1::NotVisible,
            ..retail_correct
        },
        RuntimeWitnessObservationV1 {
            render_integrity: RuntimeRenderIntegrityV1::NotTested,
            ..retail_correct
        },
    ] {
        assert_eq!(
            classify_runtime_witness_use_v1(&incomplete),
            RuntimeWitnessUseV1::NotAdmitted,
        );
    }
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the complete read-only witness set"]
fn corrupt_h1_draw_and_three_in_place_native_families_lock_the_offline_oracle() {
    require_forced_witness_mode();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let h1_path = std::env::var_os("M2A_OWNED_H1_RUNTIME_WITNESS_MDL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join(
                "proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated/m2a_m6p01.mdl",
            )
        });
    let direwolf_path = std::env::var_os("M2A_REFERENCE_DIREWOLF_MDL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join(
                "local-reference-assets/neverwintervault-2026-07-18/extracted/nwn2-creature-conversion/models02/HellHound/c_direwolf.mdl",
            )
        });
    let c_horror_bif = path_from_env_or(
        "M2A_REFERENCE_NWN_MODELS_01_BIF",
        r"C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\models_01.bif",
    );
    let cep_hak = path_from_env_or(
        "M2A_REFERENCE_CEP_HAK",
        r"C:\Users\enonw\Documents\Neverwinter Nights\hak\cep3_core1.hak",
    );
    require_complete_witness_set(&[&h1_path, &direwolf_path, &c_horror_bif, &cep_hak]);

    let h1 = fs::read(&h1_path).expect("owned corrupt-draw H1 v20 witness");
    assert_eq!(
        sha256(&h1),
        "6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd"
    );
    let h1_summary = summarize_direct_creature_structure_v1(
        &inspect_binary_mdl(&h1).expect("owned H1 binary readback"),
    );
    assert_eq!(h1_summary.base_node_count, 25);
    assert_eq!(h1_summary.base_mesh_node_count, 1);
    assert_eq!(h1_summary.animation_count, 7);
    assert_eq!(
        h1_summary.animation_type_histogram,
        BTreeMap::from([(0, 7)])
    );
    assert_eq!(h1_summary.full_base_topology_projection_count, 0);
    assert_eq!(h1_summary.all_generic_dummy_projection_count, 7);
    assert_eq!(
        h1_summary.cep_rigid_placeholder_projection_count, 0,
        "the corrupt-draw owned witness omits its rigid mesh from local state trees"
    );

    let direwolf = fs::read(&direwolf_path).expect("read-only c_Direwolf witness in place");
    assert_eq!(
        sha256(&direwolf),
        "121b63cd51ff46c3633c740951752110bebdeab7db139719ff6ff7db077f0fd9"
    );
    let direwolf_report = inspect_binary_mdl(&direwolf).expect("c_Direwolf binary readback");
    assert_native_root_part_zero(&direwolf_report, "retail c_Direwolf");
    let direwolf_summary = verify_direct_creature_state_projection_v1(
        &direwolf_report,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect("c_Direwolf must pass the retail dummy family");
    assert_eq!(direwolf_summary.base_node_count, 30);
    assert_eq!(direwolf_summary.base_mesh_node_count, 24);
    assert_eq!(direwolf_summary.animation_count, 42);
    assert_eq!(
        direwolf_summary.animation_type_histogram,
        BTreeMap::from([(5, 42)])
    );
    assert_eq!(direwolf_summary.full_base_topology_projection_count, 42);
    assert_eq!(direwolf_summary.all_generic_dummy_projection_count, 42);
    assert_eq!(
        animation_name_set(&direwolf_report),
        expected_full_native_animation_name_set(),
        "c_Direwolf must carry the same complete direct-creature namespace"
    );
    assert_native_full_animation_behavior(&direwolf_report, "retail c_Direwolf");
    assert_native_gameplay_event_floor(&direwolf_report, "retail c_Direwolf");

    let c_horror = read_exact_range(&c_horror_bif, 29_729_840, 393_692);
    assert_eq!(
        sha256(&c_horror),
        "2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e"
    );
    let c_horror_report = inspect_binary_mdl(&c_horror).expect("retail c_horror binary readback");
    assert_native_root_part_zero(&c_horror_report, "retail c_horror");
    let c_horror_summary = verify_direct_creature_state_projection_v1(
        &c_horror_report,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect("c_horror must pass the retail dummy family");
    assert_eq!(c_horror_summary.base_node_count, 27);
    assert_eq!(c_horror_summary.base_mesh_node_count, 21);
    assert_eq!(c_horror_summary.base_max_depth, 7);
    assert_eq!(c_horror_summary.animation_count, 42);
    assert_eq!(
        c_horror_summary.animation_type_histogram,
        BTreeMap::from([(5, 42)])
    );
    assert_eq!(c_horror_summary.full_base_topology_projection_count, 42);
    assert_eq!(c_horror_summary.all_generic_dummy_projection_count, 42);
    assert_eq!(
        animation_name_set(&c_horror_report),
        expected_full_native_animation_name_set(),
        "retail c_horror is the primary 42-state namespace witness"
    );
    assert_native_full_animation_behavior(&c_horror_report, "retail c_horror");
    assert_native_gameplay_event_floor(&c_horror_report, "retail c_horror");

    let cep_r3 = read_exact_range(&cep_hak, 264_142_176, 846_064);
    assert_eq!(
        sha256(&cep_r3),
        "62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a"
    );
    let cep_report = inspect_binary_mdl(&cep_r3).expect("CEP R3 binary readback");
    assert_native_root_part_zero(&cep_report, "CEP R3 c_phod_horror_b");
    let actual_provenance = cep_r3_provenance();
    let expected_provenance = cep_r3_provenance();
    let cep_summary = verify_direct_creature_state_projection_with_expected_provenance_v1(
        &cep_report,
        MdlStateProjectionProfileV1::CepRigidPlaceholderV1,
        Some(&actual_provenance),
        Some(&expected_provenance),
    )
    .expect("CEP R3 must pass only the provenance-bound placeholder family");
    assert_eq!(cep_summary.base_node_count, 27);
    assert_eq!(cep_summary.base_mesh_node_count, 21);
    assert_eq!(cep_summary.animation_count, 42);
    assert_eq!(cep_summary.full_base_topology_projection_count, 42);
    assert_eq!(cep_summary.cep_rigid_placeholder_projection_count, 42);
    assert_eq!(
        animation_name_set(&cep_report),
        expected_full_native_animation_name_set(),
        "CEP R3 must independently preserve the same 42-state namespace"
    );
    assert_native_full_animation_behavior(&cep_report, "CEP R3 c_phod_horror_b");
    assert_native_gameplay_event_floor(&cep_report, "CEP R3 c_phod_horror_b");
    let direwolf_events = native_animation_event_pairs(&direwolf_report);
    let horror_events = native_animation_event_pairs(&c_horror_report);
    let cep_events = native_animation_event_pairs(&cep_report);
    let common_events = direwolf_events
        .intersection(&horror_events)
        .filter(|pair| cep_events.contains(*pair))
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        common_events,
        COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1
            .iter()
            .map(|(clip, event)| ((*clip).to_owned(), (*event).to_owned()))
            .collect(),
        "the selected event coverage profile must remain the exact three-family intersection"
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and read-only cep3_core1.hak"]
fn cep_state_skin_profiles_are_preserved_and_reader_coverage_gap_is_explicit() {
    require_forced_witness_mode();
    let cep_hak = path_from_env_or(
        "M2A_REFERENCE_CEP_HAK",
        r"C:\Users\enonw\Documents\Neverwinter Nights\hak\cep3_core1.hak",
    );
    require_complete_witness_set(&[&cep_hak]);
    let bytes = fs::read(&cep_hak).expect("read-only CEP HAK");
    assert_eq!(
        sha256(&bytes),
        "6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a"
    );
    let archive = ErfArchive::parse(&bytes).expect("CEP HAK tables");

    let mut model_resource_count = 0u32;
    let mut binary_signature_count = 0u32;
    let mut parsed_binary_model_count = 0u32;
    let mut animated_skin_model_count = 0u32;
    let mut no_state_skin_model_count = 0u32;
    let mut generic_matching_model_count = 0u32;
    let mut generic_matching_node_count = 0u32;
    let mut generic_matching_leaf_no_controller_count = 0u32;
    let mut matching_state_skin_model_count = 0u32;
    let mut matching_state_skin_node_count = 0u32;
    for resource in archive
        .resources()
        .iter()
        .filter(|resource| resource.resource_type == 2002)
    {
        model_resource_count += 1;
        let payload = archive
            .find(&resource.resref, resource.resource_type)
            .expect("indexed CEP model payload");
        binary_signature_count += u32::from(payload.starts_with(&[0, 0, 0, 0]));
        let Ok(report) = inspect_binary_mdl(payload) else {
            continue;
        };
        parsed_binary_model_count += 1;
        let summary = summarize_direct_creature_state_skin_profile_v1(&report);
        if summary.base_skin_count == 0 || summary.animation_count == 0 {
            continue;
        }
        animated_skin_model_count += 1;
        no_state_skin_model_count += u32::from(summary.matching_state_skin_node_count == 0);
        if summary.matching_generic_node_count > 0 {
            generic_matching_model_count += 1;
        }
        generic_matching_node_count += summary.matching_generic_node_count;
        generic_matching_leaf_no_controller_count +=
            summary.matching_generic_leaf_no_controller_count;
        if summary.matching_state_skin_node_count > 0 {
            matching_state_skin_model_count += 1;
        }
        matching_state_skin_node_count += summary.matching_state_skin_node_count;
    }

    println!(
        "{}",
        serde_json::json!({
            "modelResourceCount": model_resource_count,
            "binarySignatureCount": binary_signature_count,
            "parsedBinaryModelCount": parsed_binary_model_count,
            "unsupportedBinaryModelCount": binary_signature_count - parsed_binary_model_count,
            "animatedSkinModelCount": animated_skin_model_count,
            "noStateSkinModelCount": no_state_skin_model_count,
            "genericMatchingModelCount": generic_matching_model_count,
            "genericMatchingNodeCount": generic_matching_node_count,
            "genericMatchingLeafNoControllerCount": generic_matching_leaf_no_controller_count,
            "matchingStateSkinModelCount": matching_state_skin_model_count,
            "matchingStateSkinNodeCount": matching_state_skin_node_count,
        })
    );
    assert_eq!(model_resource_count, 3_517);
    assert_eq!(binary_signature_count, 3_430);
    assert_eq!(parsed_binary_model_count, 2_151);
    assert_eq!(binary_signature_count - parsed_binary_model_count, 1_279);
    assert_eq!(animated_skin_model_count, 72);
    assert_eq!(no_state_skin_model_count, 66);
    assert_eq!(generic_matching_model_count, 6);
    assert_eq!(generic_matching_node_count, 214);
    assert_eq!(generic_matching_leaf_no_controller_count, 214);
    assert_eq!(matching_state_skin_model_count, 6);
    assert_eq!(matching_state_skin_node_count, 565);
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the complete read-only witness set"]
fn ascii_and_reference_compiler_outputs_are_not_silent_product_oracles() {
    require_forced_witness_mode();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ascii_path = std::env::var_os("M2A_REFERENCE_ASCII_SQUIRREL_MDL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join(
                "local-reference-assets/neverwintervault-2026-07-18/extracted/skinmesh-animals/animals/c_squirrel.mdl",
            )
        });
    let compiled_path = std::env::var_os("M2A_REFERENCE_CLEANMODELS_MDL")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join("proof-output/incaxje-neverblender-proof/incaxje.mdl"));
    require_complete_witness_set(&[&ascii_path, &compiled_path]);
    let ascii = fs::read(ascii_path).expect("read-only ASCII c_squirrel reference");
    assert!(ascii.starts_with(b"# Exported"));
    let error =
        inspect_binary_mdl(&ascii).expect_err("ASCII is not the product binary writer path");
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");

    let compiled = fs::read(compiled_path).expect("reference compiler output");
    assert_eq!(
        sha256(&compiled),
        "a30425cd49fe9cbe7cefc1fcdce255256ecf3a2361e779366a04038043acaeae"
    );
    assert!(
        inspect_binary_mdl(&compiled).is_err(),
        "the unsupported CleanModels skin profile must not become a silent golden oracle"
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the complete read-only witness set"]
fn renderer_differential_against_exact_r30_is_payload_free_and_confounded() {
    require_forced_witness_mode();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let r30_path = repo
        .join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0p01.mdl");
    let h1_path = std::env::var_os("M2A_OWNED_H1_RUNTIME_WITNESS_MDL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join(
                "proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated/m2a_m6p01.mdl",
            )
        });
    let direwolf_path = std::env::var_os("M2A_REFERENCE_DIREWOLF_MDL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join(
                "local-reference-assets/neverwintervault-2026-07-18/extracted/nwn2-creature-conversion/models02/HellHound/c_direwolf.mdl",
            )
        });
    let c_horror_bif = path_from_env_or(
        "M2A_REFERENCE_NWN_MODELS_01_BIF",
        r"C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\models_01.bif",
    );
    require_complete_witness_set(&[&r30_path, &h1_path, &direwolf_path, &c_horror_bif]);

    let r30 = fs::read(r30_path).expect("exact frozen r30 negative MDL");
    let h1 = fs::read(h1_path).expect("owned corrupt-draw H1 MDL");
    let direwolf = fs::read(direwolf_path).expect("read-only retail witness");
    let horror = read_exact_range(&c_horror_bif, 29_729_840, 393_692);
    assert_eq!(
        sha256(&r30),
        "43a5cbfa1a20146ec0990ce7ee980d70a54d7f72689781a58c7d9614bc35b8c7"
    );

    let (r30_projection, _) =
        inspect_direct_creature_envelope_projection_v1(&r30).expect("tolerant r30 projection");
    let policy_classes = [
        DirectCreatureEnvelopeDifferenceClassV1::ModelRuntimeDefaults,
        DirectCreatureEnvelopeDifferenceClassV1::MeshRuntimeDefaults,
        DirectCreatureEnvelopeDifferenceClassV1::VertexColorPresence,
        DirectCreatureEnvelopeDifferenceClassV1::FaceRuntimeDefaults,
    ];
    let mut consensus = None::<BTreeMap<(String, String), ()>>;
    for (label, bytes) in [
        ("owned-h1", h1),
        ("retail-a", direwolf),
        ("retail-b", horror),
    ] {
        let (positive, positive_sha) = inspect_direct_creature_envelope_projection_v1(&bytes)
            .expect("tolerant runtime-positive projection");
        let (differential, differential_sha) =
            direct_creature_envelope_differential_v1(&r30_projection, &positive)
                .expect("payload-free exact field differential");
        let policy = differential
            .differences
            .iter()
            .filter(|difference| policy_classes.contains(&difference.classification))
            .map(|difference| {
                (
                    (
                        difference.path.clone(),
                        serde_json::to_string(&difference.right).expect("scalar policy value"),
                    ),
                    (),
                )
            })
            .collect::<BTreeMap<_, _>>();
        consensus = Some(match consensus {
            None => policy,
            Some(previous) => previous
                .into_iter()
                .filter(|(key, _)| policy.contains_key(key))
                .collect(),
        });
        println!(
            "{}",
            serde_json::json!({
                "schemaVersion": 1,
                "profile": "DIRECT_CREATURE_ENVELOPE_DIFFERENTIAL_WITNESS_REPORT_V1",
                "witnessRole": label,
                "projectionSha256": positive_sha,
                "differentialSha256": differential_sha,
                "classificationCounts": differential.classification_counts,
                "fieldAssessments": positive.field_assessments,
                "policyDifferences": differential.differences.iter().filter(|difference| policy_classes.contains(&difference.classification)).map(|difference| serde_json::json!({
                    "path": difference.path,
                    "classification": difference.classification,
                    "negative": difference.left,
                    "positive": difference.right,
                })).collect::<Vec<_>>(),
            })
        );
    }
    assert!(
        consensus.expect("three positive projections").is_empty(),
        "H1 and both retail positives have no exact consensus renderer-default delta against r30"
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact frozen r30 project artifact"]
fn frozen_r30_negative_cannot_enter_the_hierarchy_experiment_profile() {
    require_forced_witness_mode();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path =
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb");
    require_complete_witness_set(&[&source_path]);
    let source = fs::read(source_path).expect("exact frozen r30 source GLB");
    assert_eq!(
        sha256(&source),
        "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1",
        "frozen r30 source identity must remain byte-identical"
    );
    let profile = declare_source_topology_rigid_experiment_profile_v3(
        &source,
        "owned://frozen-negative/r30/source.glb",
        SourceTopologyOriginV1::UserDerived,
        "m2a_m0p01",
    )
    .expect("explicit declaration is not admission");
    let error = build_source_topology_rigid_experiment_v3(&source, &profile)
        .expect_err("historical flat r30 source must not enter the new hierarchy profile");
    assert_eq!(error.code, "M2A-HIERARCHY-EXPERIMENT-DEPTH");
}

#[test]
fn absent_witness_policy_is_explicit_skip_or_fail_closed() {
    let absent = std::env::temp_dir().join(format!(
        "m2a-deliberately-absent-runtime-witness-{}",
        std::process::id()
    ));
    assert!(
        !absent.exists(),
        "test path must remain deliberately absent"
    );

    let skipped = classify_witness_set(&[absent.as_path()], false)
        .expect("optional discovery must return an explicit skip state");
    assert_eq!(
        skipped,
        WitnessSetAdmissionV1::Skipped {
            missing: vec![absent.display().to_string()]
        }
    );

    let required = classify_witness_set(&[absent.as_path()], true)
        .expect_err("required witness discovery must fail closed");
    assert!(required.contains("required read-only runtime witnesses are unavailable"));
    assert!(
        required.contains(
            absent
                .file_name()
                .expect("absent witness file name")
                .to_string_lossy()
                .as_ref()
        )
    );
}

fn animation_name_set(report: &m2a_core::InspectionReport) -> BTreeSet<String> {
    report
        .animations
        .iter()
        .map(|animation| animation.name.to_ascii_lowercase())
        .collect()
}

fn expected_full_native_animation_name_set() -> BTreeSet<String> {
    FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .map(|name| (*name).to_owned())
        .collect()
}

fn assert_native_full_animation_behavior(report: &m2a_core::mdl::InspectionReport, label: &str) {
    let behavior = evaluate_direct_creature_animation_behavior_v2(report);
    let essential_semantics = [
        "cpause1",
        "cwalk",
        "crun",
        "ca1slashl",
        "cdamagel",
        "ckdbckdie",
        "cdead",
    ];
    let unique_semantics = essential_semantics
        .iter()
        .map(|name| {
            behavior
                .clips
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(name))
                .expect("full namespace clip")
                .motion_sha256
                .as_str()
        })
        .collect::<BTreeSet<_>>();
    assert!(
        behavior.full_namespace_complete,
        "{label} namespace: {:?}",
        behavior.violations
    );
    assert!(
        behavior.all_required_content_present,
        "{label} controller content: {:?}",
        behavior.violations
    );
    assert!(
        behavior.active_motion_complete,
        "{label} active motion: {:?}",
        behavior.violations
    );
    assert!(
        behavior.walk_run_distinct,
        "{label} walk/run: {:?}",
        behavior.violations
    );
    assert!(
        behavior.essential_states_distinct,
        "{label} essential states: {:?}",
        behavior.violations
    );
    assert!(
        behavior.death_transition_terminal_pose,
        "{label} death transition: {:?}",
        behavior.violations
    );
    assert_eq!(
        unique_semantics.len(),
        essential_semantics.len(),
        "{label} essential idle/locomotion/attack/damage/death semantics must be distinct"
    );
    assert!(
        behavior.behavior_candidate_eligible,
        "{label} candidate behavior: {:?}",
        behavior.violations
    );
}

fn assert_native_gameplay_event_floor(report: &m2a_core::mdl::InspectionReport, label: &str) {
    for (clip_name, event_name) in [
        ("cwalk", "snd_footstep"),
        ("crun", "snd_footstep"),
        ("ca1slashl", "hit"),
        ("ccastout", "cast"),
        ("ckdbck", "snd_hitground"),
    ] {
        let clip = report
            .animations
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(clip_name))
            .expect("full native event-floor clip");
        assert!(
            clip.events
                .iter()
                .any(|event| event.name.eq_ignore_ascii_case(event_name)),
            "{label} must carry {event_name} in {clip_name}; observed: {:?}",
            clip.events
                .iter()
                .map(|event| event.name.as_str())
                .collect::<Vec<_>>()
        );
    }
}

fn native_animation_event_pairs(
    report: &m2a_core::mdl::InspectionReport,
) -> BTreeSet<(String, String)> {
    report
        .animations
        .iter()
        .flat_map(|clip| {
            clip.events.iter().map(|event| {
                (
                    clip.name.to_ascii_lowercase(),
                    event.name.to_ascii_lowercase(),
                )
            })
        })
        .collect()
}

fn path_from_env_or(name: &str, fallback: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fallback))
}

#[derive(Debug, Eq, PartialEq)]
enum WitnessSetAdmissionV1 {
    Ready,
    Skipped { missing: Vec<String> },
}

fn classify_witness_set(paths: &[&Path], required: bool) -> Result<WitnessSetAdmissionV1, String> {
    let missing = paths
        .iter()
        .filter(|path| !path.is_file())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(WitnessSetAdmissionV1::Ready);
    }
    if required {
        return Err(format!(
            "required read-only runtime witnesses are unavailable: {missing:?}"
        ));
    }
    Ok(WitnessSetAdmissionV1::Skipped { missing })
}

fn require_complete_witness_set(paths: &[&Path]) {
    classify_witness_set(paths, true).unwrap_or_else(|message| panic!("{message}"));
}

fn require_forced_witness_mode() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "ignored runtime witness tests must be invoked explicitly with M2A_REQUIRE_RUNTIME_WITNESSES=1"
    );
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

fn read_exact_range(path: &Path, offset: u64, length: usize) -> Vec<u8> {
    let mut file = File::open(path).unwrap_or_else(|error| {
        panic!(
            "read-only witness {} is unavailable: {error}",
            path.display()
        )
    });
    file.seek(SeekFrom::Start(offset))
        .expect("seek to exact read-only witness range");
    let mut bytes = vec![0u8; length];
    file.read_exact(&mut bytes)
        .expect("read exact witness bytes in memory");
    bytes
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
