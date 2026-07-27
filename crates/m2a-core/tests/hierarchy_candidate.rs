use std::{fs, path::PathBuf};

use m2a_core::{
    direct_creature_contract::SourceTopologyOriginV1,
    glb::{GlbLimits, ingest_glb},
    hierarchy_candidate::{
        build_meshy_hierarchy_package_v4, declare_meshy_hierarchy_package_profile_v4,
        verify_meshy_hierarchy_package_v4,
    },
    hierarchy_experiment::{
        MESHY_HIERARCHY_CANDIDATE_PROFILE_V4, build_meshy_hierarchy_candidate_model_v4,
        declare_meshy_hierarchy_candidate_profile_v4, derive_meshy_m0_hierarchy_source_v4,
        verify_meshy_hierarchy_candidate_model_v4,
    },
    proof_module::BinaryM0VerticalSliceIdentityV1,
};

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact project-owned r30 artifacts"]
fn exact_meshy_derived_source_and_candidate_model_are_topology_only_v4() {
    require_forced_mode();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let original_path =
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb");
    let r30_model_path = repo
        .join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0p01.mdl");
    let original = fs::read(&original_path).expect("frozen exact M0 source");
    let r30_model = fs::read(&r30_model_path).expect("frozen exact r30 model");

    let derived = derive_meshy_m0_hierarchy_source_v4(
        &original,
        original_path.display().to_string(),
        "generated/derived-source.glb",
    )
    .expect("deterministic project-owned topology derivation");
    assert_eq!(
        derived.binding.original_source_sha256,
        "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1"
    );
    assert_eq!(derived.binding.original_source_byte_length, 8_581_684);
    assert_eq!(
        derived.binding.original_bin_sha256,
        derived.binding.derived_bin_sha256
    );
    assert_eq!(
        derived.binding.original_geometry_sha256,
        derived.binding.derived_geometry_sha256
    );

    let ingest = ingest_glb(&derived.bytes, &GlbLimits::default()).expect("derived GLB ingest");
    assert_eq!(ingest.ir.scenes.len(), 1);
    assert_eq!(ingest.ir.nodes.len(), 3);
    assert_eq!(
        ingest
            .ir
            .nodes
            .iter()
            .map(|node| (
                node.id,
                node.parent_ids.clone(),
                node.child_ids.clone(),
                node.mesh_id
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, vec![], vec![1], None),
            (1, vec![0], vec![2], None),
            (2, vec![1], vec![], Some(0)),
        ]
    );

    let profile = declare_meshy_hierarchy_candidate_profile_v4(
        &original,
        &derived,
        &r30_model,
        SourceTopologyOriginV1::UserDerived,
        "m2a_m0p01",
        "m2a_m0t01",
    )
    .expect("explicit candidate V4 trust root");
    assert_eq!(profile.profile, MESHY_HIERARCHY_CANDIDATE_PROFILE_V4);
    let artifact =
        build_meshy_hierarchy_candidate_model_v4(&original, &derived.bytes, &r30_model, &profile)
            .expect("candidate-capable hierarchy model");
    assert!(artifact.contract.candidate_admissible);
    assert_eq!(artifact.contract.source_to_output_nodes.len(), 3);
    assert_eq!(artifact.contract.mesh_attachment.source_node_id, 2);
    assert_eq!(
        artifact.contract.candidate_raw_mdx_sha256,
        artifact.contract.r30_raw_mdx_sha256
    );
    assert_eq!(
        artifact.contract.candidate_protected_writer_fields_sha256,
        artifact.contract.r30_protected_writer_fields_sha256
    );
    verify_meshy_hierarchy_candidate_model_v4(
        &artifact.contract,
        &profile,
        &original,
        &derived.bytes,
        &r30_model,
        &artifact.model.payload,
    )
    .expect("independent exact-source candidate replay");

    let appearance = fs::read(repo.join("local-reference-assets/appearance.2da"))
        .expect("exact full runtime appearance table");
    let package_profile =
        declare_meshy_hierarchy_package_profile_v4(&profile, &r31_identity(), &appearance)
            .expect("explicit r31 package profile");
    let package = build_meshy_hierarchy_package_v4(
        &original,
        &derived.bytes,
        &r30_model,
        &appearance,
        &package_profile,
    )
    .expect("production candidate package route");
    verify_meshy_hierarchy_package_v4(
        &package.contract,
        &package_profile,
        &original,
        &derived.bytes,
        &r30_model,
        &appearance,
        &package.module,
        &package.hak,
    )
    .expect("independent package/MOD/HAK replay");
    assert_eq!(package.contract.binary_scene.module_resref, "m2a_m0r31");
    assert_eq!(package.contract.binary_scene.area_resref, "m2a_m0a31");
    assert_eq!(
        package.contract.binary_scene.ordered_hak_resrefs,
        ["m2a_m0r31"]
    );
    assert_eq!(package.contract.appearance.physical_row, 15_100);
    assert_eq!(package.contract.materialization_count, 1);
    assert!(!package.contract.runtime_profile_materialized);
    assert_eq!(
        package.texture,
        fs::read(repo.join(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0t01.tga"
        ))
        .expect("frozen r30 texture"),
        "hierarchy-only package must preserve the exact Meshy texture",
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact project-owned r30 artifacts"]
fn exact_meshy_candidate_rejects_self_consistent_source_baseline_and_package_attacks() {
    require_forced_mode();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let original = fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("frozen source");
    let r30_model =
        fs::read(repo.join(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0p01.mdl",
        ))
        .expect("frozen model");
    let appearance =
        fs::read(repo.join("local-reference-assets/appearance.2da")).expect("full appearance");
    let derived = derive_meshy_m0_hierarchy_source_v4(
        &original,
        "frozen://m0/source.glb",
        "generated/derived-source.glb",
    )
    .expect("derived source");
    let profile = declare_meshy_hierarchy_candidate_profile_v4(
        &original,
        &derived,
        &r30_model,
        SourceTopologyOriginV1::UserDerived,
        "m2a_m0p01",
        "m2a_m0t01",
    )
    .expect("model profile");

    let mut wrong_original = original.clone();
    *wrong_original.last_mut().expect("source byte") ^= 1;
    let error = derive_meshy_m0_hierarchy_source_v4(
        &wrong_original,
        "frozen://m0/source.glb",
        "generated/derived-source.glb",
    )
    .expect_err("rehash cannot replace frozen lineage identity");
    assert_eq!(error.code, "M2A-HIERARCHY-CANDIDATE-FROZEN-SOURCE");

    let mut forged_derived = derived.bytes.clone();
    let name = b"m2a_hier_1";
    let replacement = b"m2a_hier_x";
    let offset = forged_derived
        .windows(name.len())
        .position(|window| window == name)
        .expect("derived neutral node name");
    forged_derived[offset..offset + name.len()].copy_from_slice(replacement);
    let mut self_consistent_profile = profile.clone();
    self_consistent_profile.derived_source_sha256 = sha256(&forged_derived);
    self_consistent_profile.derived_source_byte_length = forged_derived.len() as u64;
    let error = build_meshy_hierarchy_candidate_model_v4(
        &original,
        &forged_derived,
        &r30_model,
        &self_consistent_profile,
    )
    .expect_err("self-consistent changed derived source cannot replace deterministic derivation");
    assert_eq!(
        error.code,
        "M2A-HIERARCHY-CANDIDATE-DERIVED-SOURCE-MISMATCH"
    );

    let mut wrong_r30 = r30_model.clone();
    *wrong_r30.last_mut().expect("r30 byte") ^= 1;
    let error =
        build_meshy_hierarchy_candidate_model_v4(&original, &derived.bytes, &wrong_r30, &profile)
            .expect_err("rehash cannot replace frozen r30 baseline");
    assert_eq!(error.code, "M2A-HIERARCHY-CANDIDATE-FROZEN-R30-MODEL");

    let mut wrong_identity = r31_identity();
    wrong_identity.hak_resref = "m2a_m0r30".to_owned();
    let error = declare_meshy_hierarchy_package_profile_v4(&profile, &wrong_identity, &appearance)
        .expect_err("legacy/cross-lineage HAK identity must fail");
    assert_eq!(error.code, "M2A-HIERARCHY-PACKAGE-LINEAGE-IDENTITY");

    let mut wrong_appearance = appearance.clone();
    *wrong_appearance.last_mut().expect("appearance byte") ^= 1;
    let error =
        declare_meshy_hierarchy_package_profile_v4(&profile, &r31_identity(), &wrong_appearance)
            .expect_err("self-consistent truncated/changed table cannot replace full input");
    assert_eq!(error.code, "M2A-HIERARCHY-PACKAGE-APPEARANCE-TRUST-ROOT");
}

fn r31_identity() -> BinaryM0VerticalSliceIdentityV1 {
    BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_m0r31".to_owned(),
        area_resref: "m2a_m0a31".to_owned(),
        hak_resref: "m2a_m0r31".to_owned(),
    }
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn require_forced_mode() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "forced exact-artifact mode must be explicit",
    );
}
