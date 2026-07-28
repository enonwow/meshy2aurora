use std::fs;

use m2a_core::{
    animated_donor_candidate::build_h2_r43_h1_root_layout_candidate_v1,
    creature_visibility_gate::{
        CreatureVisualStateV1, H2R43DiagnosticVerdictV1, H2R43NextActionV1,
        H2R43OwnerObservationV1, classify_h2_r43_owner_observation_v1,
        verify_h2_r43_installed_lineage_v1, verify_h2_r43_owner_proof_scene_v1,
    },
    runtime_evidence::RuntimeProofCompletenessV1,
};

#[path = "support/canonical_workspace.rs"]
mod canonical_workspace;

fn exact_candidate() -> m2a_core::animated_donor_candidate::H2R43H1RootLayoutCandidateArtifactV1 {
    let repo = canonical_workspace::canonical_repository_root();
    let source = fs::read(repo.join("sample-3d/h2-clockwork-sentinel-1500/source.glb"))
        .expect("exact generated Meshy H2 source");
    let appearance = fs::read(repo.join("local-reference-assets/appearance.2da"))
        .expect("exact full runtime appearance table");
    build_h2_r43_h1_root_layout_candidate_v1(&source, &appearance)
        .expect("exact frozen r43 candidate")
}

fn observation(
    stock_control: CreatureVisualStateV1,
    h2_candidate: CreatureVisualStateV1,
) -> H2R43OwnerObservationV1 {
    H2R43OwnerObservationV1 {
        exact_candidate_bound: true,
        capture_judgeable: true,
        proof_completeness: RuntimeProofCompletenessV1::Verified,
        stock_control,
        h2_candidate,
    }
}

#[test]
fn exact_r43_installation_and_scene_are_verified_without_mutating_the_candidate() {
    let candidate = exact_candidate();

    let installed = verify_h2_r43_installed_lineage_v1(
        &candidate.module,
        &candidate.module,
        &candidate.hak,
        &candidate.hak,
    )
    .expect("byte-identical exact r43 installation");
    assert_eq!(installed.profile, "H2_R43_INSTALLED_LINEAGE_V1");
    assert_eq!(installed.module.byte_length, 20_280);
    assert_eq!(installed.hak.byte_length, 20_084_361);
    assert!(installed.source_matches_installed);

    let scene =
        verify_h2_r43_owner_proof_scene_v1(&candidate.module).expect("exact r43 owner scene");
    assert_eq!(scene.profile, "H2_R43_OWNER_PROOF_SCENE_V1");
    assert_eq!(scene.module_resref, "m2a_h2r43");
    assert_eq!(scene.area_resref, "m2a_h2a43");
    assert_eq!(scene.ordered_hak_resrefs, ["m2a_h2r43"]);
    assert_eq!(scene.entry_position, [10.0, 10.0, 0.0]);
    assert_eq!(scene.entry_direction, [0.0, 1.0]);
    assert_eq!(scene.h2_fixture.id, "h2_fixture");
    assert_eq!(scene.h2_fixture.appearance_row, 15_100);
    assert_eq!(scene.h2_fixture.position, [10.0, 14.5, 0.0]);
    assert_eq!(scene.stock_control.id, "stock_control");
    assert_eq!(scene.stock_control.appearance_row, 102);
    assert_eq!(scene.stock_control.position, [7.0, 14.5, 0.0]);
}

#[test]
fn installed_lineage_fails_closed_on_any_destination_drift() {
    let candidate = exact_candidate();
    let mut changed_module = candidate.module.clone();
    let last = changed_module
        .last_mut()
        .expect("exact r43 module is nonempty");
    *last ^= 0x01;

    let error = verify_h2_r43_installed_lineage_v1(
        &candidate.module,
        &changed_module,
        &candidate.hak,
        &candidate.hak,
    )
    .expect_err("changed installed module must fail");
    assert_eq!(error.code, "H2-R43-INSTALLED-LINEAGE-MISMATCH");
    assert_eq!(error.path, "installed.module");
}

#[test]
#[ignore = "requires M2A_H2_R43_INSTALLED_MOD and M2A_H2_R43_INSTALLED_HAK"]
fn exact_native_installation_matches_the_frozen_candidate() {
    let candidate = exact_candidate();
    let installed_module_path =
        std::env::var_os("M2A_H2_R43_INSTALLED_MOD").expect("installed MOD path environment");
    let installed_hak_path =
        std::env::var_os("M2A_H2_R43_INSTALLED_HAK").expect("installed HAK path environment");
    let installed_module = fs::read(installed_module_path).expect("read installed exact r43 MOD");
    let installed_hak = fs::read(installed_hak_path).expect("read installed exact r43 HAK");

    let report = verify_h2_r43_installed_lineage_v1(
        &candidate.module,
        &installed_module,
        &candidate.hak,
        &installed_hak,
    )
    .expect("native r43 installation must be byte-identical to the frozen candidate");
    assert!(report.source_matches_installed);
}

#[test]
fn owner_result_matrix_admits_only_an_isolated_judgeable_custom_absence() {
    let isolated_absence = classify_h2_r43_owner_observation_v1(&observation(
        CreatureVisualStateV1::VisibleCorrect,
        CreatureVisualStateV1::NotVisible,
    ));
    assert_eq!(
        isolated_absence.verdict,
        H2R43DiagnosticVerdictV1::CustomModelRejected
    );
    assert_eq!(
        isolated_absence.next_action,
        H2R43NextActionV1::DiagnoseCustomMdlMdx
    );
    assert!(isolated_absence.model_iteration_admitted);
    assert!(!isolated_absence.visibility_gate_closed);

    let both_absent = classify_h2_r43_owner_observation_v1(&observation(
        CreatureVisualStateV1::NotVisible,
        CreatureVisualStateV1::NotVisible,
    ));
    assert_eq!(
        both_absent.verdict,
        H2R43DiagnosticVerdictV1::CustomModelRejectedControlUntrusted
    );
    assert_eq!(
        both_absent.next_action,
        H2R43NextActionV1::DiagnoseCustomMdlMdxAndRepairControl
    );
    assert!(
        both_absent.model_iteration_admitted,
        "an exact, judgeable owner-visible H2 absence remains not_visible even when the synthetic control also fails",
    );

    let visible = classify_h2_r43_owner_observation_v1(&observation(
        CreatureVisualStateV1::VisibleCorrect,
        CreatureVisualStateV1::VisibleCorrect,
    ));
    assert_eq!(
        visible.verdict,
        H2R43DiagnosticVerdictV1::R42OuterRootConfirmed
    );
    assert_eq!(
        visible.next_action,
        H2R43NextActionV1::ContinueToSkinDeformation
    );
    assert!(!visible.model_iteration_admitted);
    assert!(visible.visibility_gate_closed);

    let corrupt = classify_h2_r43_owner_observation_v1(&observation(
        CreatureVisualStateV1::VisibleCorrect,
        CreatureVisualStateV1::VisibleCorrupt,
    ));
    assert_eq!(
        corrupt.verdict,
        H2R43DiagnosticVerdictV1::DrawAcceptedTransformCorrupt
    );
    assert_eq!(
        corrupt.next_action,
        H2R43NextActionV1::DiagnoseHierarchyTransform
    );
    assert!(!corrupt.model_iteration_admitted);
}

#[test]
fn judgeable_owner_absence_survives_a_failed_proof_packet() {
    let decision = classify_h2_r43_owner_observation_v1(&H2R43OwnerObservationV1 {
        proof_completeness: RuntimeProofCompletenessV1::Failed,
        ..observation(
            CreatureVisualStateV1::VisibleCorrect,
            CreatureVisualStateV1::NotVisible,
        )
    });
    assert_eq!(
        decision.verdict,
        H2R43DiagnosticVerdictV1::CustomModelRejected
    );
    assert!(decision.model_iteration_admitted);
    assert!(!decision.visibility_gate_closed);
}

#[test]
fn corrupt_stock_control_does_not_erase_custom_absence() {
    let decision = classify_h2_r43_owner_observation_v1(&observation(
        CreatureVisualStateV1::VisibleCorrupt,
        CreatureVisualStateV1::NotVisible,
    ));
    assert_eq!(
        decision.verdict,
        H2R43DiagnosticVerdictV1::CustomModelRejectedControlUntrusted
    );
    assert_eq!(
        decision.next_action,
        H2R43NextActionV1::DiagnoseCustomMdlMdxAndRepairControl
    );
    assert!(decision.model_iteration_admitted);
    assert!(!decision.visibility_gate_closed);
}

#[test]
fn missing_stock_result_does_not_erase_an_exact_judgeable_h2_absence() {
    let decision = classify_h2_r43_owner_observation_v1(&observation(
        CreatureVisualStateV1::NotTested,
        CreatureVisualStateV1::NotVisible,
    ));
    assert_eq!(
        decision.verdict,
        H2R43DiagnosticVerdictV1::CustomModelRejectedControlUntrusted
    );
    assert_eq!(
        decision.next_action,
        H2R43NextActionV1::DiagnoseCustomMdlMdxAndRepairControl
    );
    assert!(decision.model_iteration_admitted);
    assert!(!decision.visibility_gate_closed);
}

#[test]
fn incomplete_or_unbound_owner_result_never_opens_the_iteration_gate() {
    for result in [
        H2R43OwnerObservationV1 {
            exact_candidate_bound: false,
            ..observation(
                CreatureVisualStateV1::VisibleCorrect,
                CreatureVisualStateV1::NotVisible,
            )
        },
        H2R43OwnerObservationV1 {
            capture_judgeable: false,
            ..observation(
                CreatureVisualStateV1::VisibleCorrect,
                CreatureVisualStateV1::NotVisible,
            )
        },
        H2R43OwnerObservationV1 {
            proof_completeness: RuntimeProofCompletenessV1::Missing,
            ..observation(
                CreatureVisualStateV1::VisibleCorrect,
                CreatureVisualStateV1::NotVisible,
            )
        },
        observation(
            CreatureVisualStateV1::VisibleCorrect,
            CreatureVisualStateV1::NotTested,
        ),
    ] {
        let decision = classify_h2_r43_owner_observation_v1(&result);
        assert_eq!(decision.verdict, H2R43DiagnosticVerdictV1::ProofIncomplete);
        assert_eq!(
            decision.next_action,
            H2R43NextActionV1::CompleteSameCandidateProof
        );
        assert!(!decision.model_iteration_admitted);
        assert!(!decision.visibility_gate_closed);
    }
}
