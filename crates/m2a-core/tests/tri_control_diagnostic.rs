use std::{fs, path::PathBuf};

use m2a_core::{
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1, M0RuntimeDirectionV1,
        M0RuntimePositionV1,
    },
    tri_control_diagnostic::{
        TriControlDiagnosticFixtureV1, build_tri_control_diagnostic_last_city_v2,
        build_tri_control_diagnostic_v1, verify_tri_control_diagnostic_last_city_v2,
        verify_tri_control_diagnostic_v1,
    },
};

struct Inputs {
    base_appearance: Vec<u8>,
    r31_mdl: Vec<u8>,
    r31_tga: Vec<u8>,
    h1_appearance: Vec<u8>,
    h1_mdl: Vec<u8>,
    h1_tga: Vec<u8>,
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact project-owned r31/H1 artifacts"]
fn exact_tri_control_composes_one_single_hak_module_without_runtime_claims() {
    let inputs = exact_inputs();
    let identity = diagnostic_identity();
    let artifact = build(&identity, &inputs).expect("offline tri-control composition");

    assert!(artifact.contract.diagnostic_only);
    assert!(!artifact.contract.runtime_admissible);
    assert!(!artifact.contract.resolver_verified);
    assert!(!artifact.contract.renderer_verified);
    assert_eq!(artifact.contract.module_identity, identity);
    assert_eq!(
        artifact.contract.module_readback.ordered_hak_resrefs,
        ["m2a_diagh01"]
    );
    let expected = expected_fixtures();
    assert_eq!(artifact.contract.fixtures, expected);
    assert_eq!(
        artifact.contract.module_readback.fixtures,
        expected
            .iter()
            .map(|fixture| BinaryCreatureOwnedFixtureV1 {
                id: fixture.id.clone(),
                template_resref: fixture.template_resref.clone(),
                display_name: fixture.display_name.clone(),
                appearance_row: fixture.appearance_row,
                position: fixture.position,
                orientation: fixture.orientation,
            })
            .collect::<Vec<_>>()
    );

    verify_tri_control_diagnostic_v1(
        &artifact.contract,
        &artifact.module,
        &artifact.hak,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
    .expect("independent deterministic composition replay");
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact project-owned r31/H1 artifacts"]
fn tri_control_composition_rejects_mutation_and_self_promotion() {
    let inputs = exact_inputs();
    let artifact = build(&diagnostic_identity(), &inputs).expect("offline tri-control composition");

    let mut changed_module = artifact.module.clone();
    let last = changed_module.len() - 1;
    changed_module[last] ^= 1;
    let error = verify_tri_control_diagnostic_v1(
        &artifact.contract,
        &changed_module,
        &artifact.hak,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
    .expect_err("changed MOD must fail replay");
    assert_eq!(error.code, "M2A-TRI-CONTROL-DIAGNOSTIC-MODULE-REPLAY");

    let mut changed_hak = artifact.hak.clone();
    let last = changed_hak.len() - 1;
    changed_hak[last] ^= 1;
    let error = verify_tri_control_diagnostic_v1(
        &artifact.contract,
        &artifact.module,
        &changed_hak,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
    .expect_err("changed HAK must fail exact payload binding");
    assert_eq!(error.code, "M2A-TRI-CONTROL-RESOURCE-PAYLOAD");

    let mut promoted = artifact.contract.clone();
    promoted.diagnostic_only = false;
    promoted.runtime_admissible = true;
    promoted.renderer_verified = true;
    let error = verify_tri_control_diagnostic_v1(
        &promoted,
        &artifact.module,
        &artifact.hak,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
    .expect_err("offline contract must not self-promote");
    assert_eq!(error.code, "M2A-TRI-CONTROL-DIAGNOSTIC-CONTRACT");
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact Last City/r31/H1 artifacts"]
fn last_city_v2_uses_fresh_caller_resrefs_and_appended_fixture_rows() {
    let inputs = exact_last_city_inputs();
    let identity = BinaryCreatureModuleIdentityV1 {
        module_resref: "m2a_diaglc01".to_owned(),
        area_resref: "m2a_diagla01".to_owned(),
        hak_resref: "m2a_diaglh01".to_owned(),
    };
    let artifact = build_tri_control_diagnostic_last_city_v2(
        &identity,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
    .expect("Last City V2 diagnostic composition");

    assert_eq!(artifact.contract.schema_version, 2);
    assert_eq!(artifact.contract.module_identity, identity);
    assert_eq!(
        artifact.contract.module_readback.ordered_hak_resrefs,
        ["m2a_diaglh01"]
    );
    assert_eq!(artifact.contract.fixtures, expected_last_city_fixtures());
    assert_eq!(artifact.contract.hak_contract.source_physical_rows, 15_219);
    assert_eq!(artifact.contract.hak_contract.output_physical_rows, 15_221);
    verify_tri_control_diagnostic_last_city_v2(
        &artifact.contract,
        &artifact.module,
        &artifact.hak,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
    .expect("Last City V2 independent composition replay");
}

fn diagnostic_identity() -> BinaryCreatureModuleIdentityV1 {
    BinaryCreatureModuleIdentityV1 {
        module_resref: "m2a_diag01".to_owned(),
        area_resref: "m2a_diaga01".to_owned(),
        hak_resref: "m2a_diagh01".to_owned(),
    }
}

fn expected_fixtures() -> Vec<TriControlDiagnosticFixtureV1> {
    vec![
        expected_fixture(
            "stock_renderer_control",
            "stock_control",
            "m2a_d_stock",
            "Stock c_horror control",
            102,
            "c_horror",
            5.0,
            18.0,
        ),
        expected_fixture(
            "exact_r31_candidate",
            "candidate_m0",
            "m2a_d_m0",
            "Exact r31 M0 candidate",
            15_100,
            "m2a_m0p01",
            10.0,
            14.5,
        ),
        expected_fixture(
            "project_owned_positive_control",
            "custom_control_h1",
            "m2a_d_h1",
            "Project-owned H1 v20 control",
            15_101,
            "m2a_m6p01",
            15.0,
            18.0,
        ),
    ]
}

fn expected_last_city_fixtures() -> Vec<TriControlDiagnosticFixtureV1> {
    let mut fixtures = expected_fixtures();
    fixtures[1].appearance_row = 15_219;
    fixtures[2].appearance_row = 15_220;
    fixtures
}

#[allow(clippy::too_many_arguments)]
fn expected_fixture(
    role: &str,
    id: &str,
    template_resref: &str,
    display_name: &str,
    appearance_row: u16,
    model_resref: &str,
    x: f32,
    y: f32,
) -> TriControlDiagnosticFixtureV1 {
    TriControlDiagnosticFixtureV1 {
        role: role.to_owned(),
        id: id.to_owned(),
        template_resref: template_resref.to_owned(),
        display_name: display_name.to_owned(),
        appearance_row,
        model_resref: model_resref.to_owned(),
        position: M0RuntimePositionV1 { x, y, z: 0.0 },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }
}

fn build(
    identity: &BinaryCreatureModuleIdentityV1,
    inputs: &Inputs,
) -> Result<
    m2a_core::tri_control_diagnostic::TriControlDiagnosticArtifactV1,
    m2a_core::tri_control_diagnostic::TriControlDiagnosticErrorV1,
> {
    build_tri_control_diagnostic_v1(
        identity,
        &inputs.base_appearance,
        &inputs.r31_mdl,
        &inputs.r31_tga,
        &inputs.h1_appearance,
        &inputs.h1_mdl,
        &inputs.h1_tga,
    )
}

fn exact_inputs() -> Inputs {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "exact tri-control artifact tests require M2A_REQUIRE_RUNTIME_WITNESSES=1"
    );
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let r31 = repo.join("proof-output/m0-r31-hierarchy-only-20260722/generated");
    let h1 = repo.join("proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated");
    let r31_appearance = fs::read(r31.join("appearance.2da")).expect("exact r31 appearance");
    Inputs {
        base_appearance: r31_appearance[..6_901_169].to_vec(),
        r31_mdl: fs::read(r31.join("m2a_m0p01.mdl")).expect("exact r31 MDL"),
        r31_tga: fs::read(r31.join("m2a_m0t01.tga")).expect("exact r31 TGA"),
        h1_appearance: fs::read(h1.join("appearance.2da")).expect("exact H1 appearance"),
        h1_mdl: fs::read(h1.join("m2a_m6p01.mdl")).expect("exact H1 MDL"),
        h1_tga: fs::read(h1.join("m2a_m6t01.tga")).expect("exact H1 TGA"),
    }
}

fn exact_last_city_inputs() -> Inputs {
    let mut input = exact_inputs();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    input.base_appearance = fs::read(
        repo.join(
            "proof-output/lc-hd-animals-c-squirrel-reference-audit-2026-07-18/source-copies/appearance.2da",
        ),
    )
    .expect("exact user-provided Last City appearance copy");
    input
}
