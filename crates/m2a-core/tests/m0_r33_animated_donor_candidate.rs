use std::{fs, path::PathBuf};

use m2a_core::{
    animated_donor_candidate::{
        M0_R33_ACCEPTED_R32_PROOF_SHA256, M0_R33_APPEARANCE_LABEL, M0_R33_APPEARANCE_ROW,
        M0_R33_AREA_RESREF, M0_R33_DONOR_SHA256, M0_R33_FIXTURE_DISPLAY_NAME, M0_R33_HAK_RESREF,
        M0_R33_MODEL_RESREF, M0_R33_MODEL_SHA256, M0_R33_MODULE_RESREF, M0_R33_SOURCE_SHA256,
        M0_R33_TEXTURE_RESREF, M0_R33_TEXTURE_SHA256, build_m0_r33_animated_donor_candidate_v1,
        verify_m0_r33_animated_donor_candidate_v1,
    },
    erf::ErfArchive,
    proof_module::inspect_binary_creature_multi_fixture_module_v1,
};

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the exact local M0/H1 GLBs"]
fn exact_m0_r33_weighted_animated_candidate_is_deterministic_and_runtime_complete() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source");
    let donor =
        fs::read(repo.join("sample-3d/h1-humanoid-1500/source.glb")).expect("exact H1 donor");
    let appearance =
        fs::read(repo.join("local-reference-assets/appearance.2da")).expect("exact appearance");

    let artifact = build_m0_r33_animated_donor_candidate_v1(&source, &donor, &appearance)
        .expect("exact r33 candidate");
    verify_m0_r33_animated_donor_candidate_v1(
        &artifact.contract,
        &source,
        &donor,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .expect("deterministic exact-input replay");

    assert!(artifact.contract.candidate_admissible);
    assert_eq!(
        artifact.contract.admitted_by_r32_proof_sha256,
        M0_R33_ACCEPTED_R32_PROOF_SHA256
    );
    assert_eq!(artifact.contract.source.sha256, M0_R33_SOURCE_SHA256);
    assert_eq!(artifact.contract.donor.sha256, M0_R33_DONOR_SHA256);
    assert_eq!(artifact.contract.model.resref, M0_R33_MODEL_RESREF);
    assert_eq!(artifact.contract.model.sha256, M0_R33_MODEL_SHA256);
    assert_eq!(artifact.contract.texture.resref, M0_R33_TEXTURE_RESREF);
    assert_eq!(artifact.contract.texture.sha256, M0_R33_TEXTURE_SHA256);
    assert_eq!(artifact.contract.skin_readback.node_count, 25);
    assert_eq!(artifact.contract.skin_readback.skin_node_count, 1);
    assert_eq!(artifact.contract.skin_readback.weighted_vertex_count, 2_380);
    assert_eq!(artifact.contract.skin_readback.active_bone_count, 22);
    assert_eq!(artifact.contract.retarget.local_animation_count, 7);
    assert_eq!(artifact.contract.materialization_count, 1);
    assert_eq!(artifact.contract.toolset_model_visibility, "not_tested");
    assert_eq!(artifact.contract.nwn_model_visibility, "not_tested");

    let hak = ErfArchive::parse(&artifact.hak).expect("r33 HAK");
    assert_eq!(hak.resources().len(), 3);
    assert_eq!(hak.find(M0_R33_MODEL_RESREF, 2002).unwrap(), artifact.model);
    assert_eq!(
        hak.find(M0_R33_TEXTURE_RESREF, 3).unwrap(),
        artifact.texture
    );
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        artifact.appearance_two_da
    );

    let scene = inspect_binary_creature_multi_fixture_module_v1(&artifact.module).expect("r33 MOD");
    assert_eq!(scene.module_resref, M0_R33_MODULE_RESREF);
    assert_eq!(scene.area_resref, M0_R33_AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [M0_R33_HAK_RESREF]);
    assert_eq!(scene.fixtures.len(), 1);
    assert_eq!(scene.fixtures[0].appearance_row, M0_R33_APPEARANCE_ROW);
    assert_eq!(scene.fixtures[0].display_name, M0_R33_FIXTURE_DISPLAY_NAME);
    assert!(
        String::from_utf8_lossy(&artifact.appearance_two_da)
            .contains(&format!("{M0_R33_APPEARANCE_LABEL}"))
    );

    let mut wrong_source = source.clone();
    *wrong_source.last_mut().expect("source byte") ^= 1;
    let error = build_m0_r33_animated_donor_candidate_v1(&wrong_source, &donor, &appearance)
        .expect_err("source identity attack");
    assert_eq!(error.code, "M2A-R33-INPUT-IDENTITY");
    assert_eq!(error.path, "sourceGlb");

    let mut wrong_contract = artifact.contract.clone();
    wrong_contract.model.sha256 = "0".repeat(64);
    let error = verify_m0_r33_animated_donor_candidate_v1(
        &wrong_contract,
        &source,
        &donor,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .expect_err("contract mutation");
    assert_eq!(error.code, "M2A-R33-REPLAY-MISMATCH");
}
