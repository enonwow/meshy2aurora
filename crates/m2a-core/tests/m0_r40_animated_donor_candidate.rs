use std::{fs, path::PathBuf};

use m2a_core::{
    animated_donor_candidate::{
        M0_R40_ACCEPTED_R39_PROOF_SHA256, M0_R40_APPEARANCE_ROW, M0_R40_AREA_RESREF,
        M0_R40_HAK_RESREF, M0_R40_MODEL_RESREF, M0_R40_MODEL_SHA256, M0_R40_MODULE_RESREF,
        M0_R40_TEXTURE_RESREF, build_m0_r40_animated_donor_candidate_v1,
        verify_m0_r40_animated_donor_candidate_v1,
    },
    erf::ErfArchive,
    mdl::{MdlFormatProfileV1, MdlStateProjectionProfileV1, inspect_binary_mdl},
    proof_module::inspect_binary_creature_multi_fixture_module_v1,
};

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the exact local M0/H1 GLBs"]
fn exact_m0_r40_rigid_triangle_group_candidate_is_deterministic_and_runtime_complete() {
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

    let artifact = build_m0_r40_animated_donor_candidate_v1(&source, &donor, &appearance)
        .expect("exact r40 candidate");
    verify_m0_r40_animated_donor_candidate_v1(
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
        artifact.contract.admitted_by_r39_proof_sha256,
        M0_R40_ACCEPTED_R39_PROOF_SHA256
    );
    assert_eq!(artifact.contract.model.resref, M0_R40_MODEL_RESREF);
    assert_eq!(artifact.contract.model.sha256, M0_R40_MODEL_SHA256);
    assert_eq!(artifact.contract.rigid_readback.base_node_count, 45);
    assert_eq!(artifact.contract.rigid_readback.rig_node_count, 25);
    assert_eq!(artifact.contract.rigid_readback.rigid_mesh_node_count, 20);
    assert_eq!(artifact.contract.rigid_readback.skin_node_count, 0);
    assert_eq!(artifact.contract.rigid_readback.triangle_count, 1_569);
    assert_eq!(
        artifact.contract.rigid_readback.duplicated_vertex_count,
        4_707
    );
    assert_eq!(
        artifact.contract.rigid_readback.active_parent_bone_count,
        20
    );
    assert_eq!(
        artifact.contract.rigid_readback.aurora_root_name,
        M0_R40_MODEL_RESREF
    );
    assert_eq!(artifact.contract.rigid_readback.skeleton_root_name, "Hips");
    assert_eq!(
        artifact.contract.rigid_readback.animation_node_counts,
        vec![25; 7]
    );
    assert_eq!(
        artifact
            .contract
            .rigid_readback
            .animation_scale_controller_count,
        0
    );
    assert_eq!(
        artifact.contract.rigid_readback.base_root_controller_count,
        0
    );
    assert_eq!(
        artifact.contract.rigid_readback.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
    );
    assert_eq!(
        artifact.contract.rigid_readback.state_projection_profile,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1
    );

    let model = inspect_binary_mdl(&artifact.model).expect("r40 MDL readback");
    let mut mesh_count = 0;
    let mut skin_count = 0;
    let mut pending = model.node_tree.roots.iter().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        mesh_count += usize::from(node.mesh.is_some());
        skin_count += usize::from(node.skin.is_some());
        pending.extend(&node.children);
    }
    assert_eq!(mesh_count, 20);
    assert_eq!(skin_count, 0);

    let hak = ErfArchive::parse(&artifact.hak).expect("r40 HAK");
    assert_eq!(hak.resources().len(), 3);
    assert_eq!(hak.find(M0_R40_MODEL_RESREF, 2002).unwrap(), artifact.model);
    assert_eq!(
        hak.find(M0_R40_TEXTURE_RESREF, 3).unwrap(),
        artifact.texture
    );
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        artifact.appearance_two_da
    );

    let scene = inspect_binary_creature_multi_fixture_module_v1(&artifact.module).expect("r40 MOD");
    assert_eq!(scene.module_resref, M0_R40_MODULE_RESREF);
    assert_eq!(scene.area_resref, M0_R40_AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [M0_R40_HAK_RESREF]);
    assert_eq!(scene.fixtures.len(), 1);
    assert_eq!(scene.fixtures[0].appearance_row, M0_R40_APPEARANCE_ROW);

    let mut wrong_source = source.clone();
    *wrong_source.last_mut().expect("source byte") ^= 1;
    let error = build_m0_r40_animated_donor_candidate_v1(&wrong_source, &donor, &appearance)
        .expect_err("source identity attack");
    assert_eq!(error.code, "M2A-R40-INPUT-IDENTITY");

    let mut wrong_contract = artifact.contract.clone();
    wrong_contract.model.sha256 = "0".repeat(64);
    let error = verify_m0_r40_animated_donor_candidate_v1(
        &wrong_contract,
        &source,
        &donor,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .expect_err("contract mutation");
    assert_eq!(error.code, "M2A-R40-REPLAY-MISMATCH");
}
