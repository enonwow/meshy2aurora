use std::{fs, path::PathBuf};

use m2a_core::{
    animated_donor_candidate::{
        M0_R37_ACCEPTED_R36_PROOF_SHA256, M0_R37_APPEARANCE_ROW, M0_R37_AREA_RESREF,
        M0_R37_HAK_RESREF, M0_R37_MODEL_RESREF, M0_R37_MODEL_SHA256, M0_R37_MODULE_RESREF,
        M0_R37_TEXTURE_RESREF, build_m0_r37_animated_donor_candidate_v1,
        verify_m0_r37_animated_donor_candidate_v1,
    },
    erf::ErfArchive,
    mdl::{MdlFormatProfileV1, MdlStateProjectionProfileV1},
    proof_module::inspect_binary_creature_multi_fixture_module_v1,
};

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the exact local M0/H1 GLBs"]
fn exact_m0_r37_historical_candidate_fails_closed_after_r45_skin_bind_upgrade() {
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

    let artifact = match build_m0_r37_animated_donor_candidate_v1(&source, &donor, &appearance) {
        Ok(artifact) => artifact,
        Err(error) => {
            assert_eq!(error.code, "M2A-R37-RETARGET-CONTRACT");
            assert!(error.message.contains(M0_R37_MODEL_SHA256));
            assert!(
                error
                    .message
                    .contains("historical SkinMesh lineage predates the mandatory r45")
            );
            return;
        }
    };
    verify_m0_r37_animated_donor_candidate_v1(
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
        artifact.contract.admitted_by_r36_proof_sha256,
        M0_R37_ACCEPTED_R36_PROOF_SHA256
    );
    assert_eq!(artifact.contract.model.resref, M0_R37_MODEL_RESREF);
    assert_eq!(artifact.contract.model.sha256, M0_R37_MODEL_SHA256);
    assert_eq!(artifact.contract.skin_readback.base_node_count, 26);
    assert_eq!(artifact.contract.skin_readback.rig_node_count, 25);
    assert_eq!(artifact.contract.skin_readback.skin_node_count, 1);
    assert_eq!(artifact.contract.skin_readback.weighted_vertex_count, 2_380);
    assert_eq!(artifact.contract.skin_readback.active_bone_count, 22);
    assert_eq!(artifact.contract.skin_readback.active_inline_slot_count, 22);
    assert_eq!(artifact.contract.skin_readback.unused_inline_tail_count, 42);
    assert_eq!(artifact.contract.skin_readback.unused_inline_tail_value, 0);
    assert_eq!(
        artifact.contract.skin_readback.aurora_root_name,
        M0_R37_MODEL_RESREF
    );
    assert_eq!(artifact.contract.skin_readback.skeleton_root_name, "Hips");
    assert_eq!(
        artifact.contract.skin_readback.skin_parent_name,
        M0_R37_MODEL_RESREF
    );
    assert_eq!(artifact.contract.skin_readback.aurora_root_tree_ordinal, 0);
    assert_eq!(artifact.contract.skin_readback.aurora_root_forward_slot, -1);
    assert_eq!(
        artifact.contract.skin_readback.first_active_inline_ordinal,
        1
    );
    assert_eq!(
        artifact.contract.skin_readback.last_active_inline_ordinal,
        22
    );
    assert_eq!(
        artifact.contract.skin_readback.animation_node_counts,
        vec![25; 7]
    );
    assert_eq!(
        artifact.contract.skin_readback.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2
    );
    assert_eq!(
        artifact.contract.skin_readback.state_projection_profile,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
    );

    let hak = ErfArchive::parse(&artifact.hak).expect("r37 HAK");
    assert_eq!(hak.resources().len(), 3);
    assert_eq!(hak.find(M0_R37_MODEL_RESREF, 2002).unwrap(), artifact.model);
    assert_eq!(
        hak.find(M0_R37_TEXTURE_RESREF, 3).unwrap(),
        artifact.texture
    );
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        artifact.appearance_two_da
    );

    let scene = inspect_binary_creature_multi_fixture_module_v1(&artifact.module).expect("r37 MOD");
    assert_eq!(scene.module_resref, M0_R37_MODULE_RESREF);
    assert_eq!(scene.area_resref, M0_R37_AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [M0_R37_HAK_RESREF]);
    assert_eq!(scene.fixtures.len(), 1);
    assert_eq!(scene.fixtures[0].appearance_row, M0_R37_APPEARANCE_ROW);

    let mut wrong_source = source.clone();
    *wrong_source.last_mut().expect("source byte") ^= 1;
    let error = build_m0_r37_animated_donor_candidate_v1(&wrong_source, &donor, &appearance)
        .expect_err("source identity attack");
    assert_eq!(error.code, "M2A-R37-INPUT-IDENTITY");

    let mut wrong_contract = artifact.contract.clone();
    wrong_contract.model.sha256 = "0".repeat(64);
    let error = verify_m0_r37_animated_donor_candidate_v1(
        &wrong_contract,
        &source,
        &donor,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .expect_err("contract mutation");
    assert_eq!(error.code, "M2A-R37-REPLAY-MISMATCH");
}
