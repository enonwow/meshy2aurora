use std::{collections::BTreeSet, fs, path::PathBuf};

use m2a_core::{
    erf,
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    tri_control_hak, two_da,
};
use sha2::{Digest, Sha256};
use tri_control_hak::{
    H1_MODEL_RESREF, H1_TEXTURE_RESREF, LAST_CITY_H1_APPEARANCE_ROW_V2,
    LAST_CITY_M0_APPEARANCE_ROW_V2, LAST_CITY_OUTPUT_PHYSICAL_ROWS_V2,
    LAST_CITY_SOURCE_PHYSICAL_ROWS_V2, R31_MODEL_RESREF, R31_TEXTURE_RESREF,
    TRI_CONTROL_RESOURCE_ORDER_V1, build_tri_control_hak_last_city_v2, build_tri_control_hak_v1,
    verify_tri_control_hak_last_city_v2, verify_tri_control_hak_v1,
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
fn exact_project_owned_tri_control_hak_is_diagnostic_only_and_replays() {
    let input = exact_inputs();
    let artifact = build(&input).expect("exact pinned tri-control inputs");

    assert!(artifact.contract.diagnostic_only);
    assert!(!artifact.contract.runtime_admissible);
    assert!(!artifact.contract.resolver_verified);
    assert!(!artifact.contract.renderer_verified);
    assert_eq!(artifact.contract.base_control.physical_row, 102);
    assert_eq!(artifact.contract.base_control.model_type, "S");
    assert_eq!(artifact.contract.base_control.race, "c_horror");
    assert_eq!(artifact.contract.r31_control.physical_row, 15_100);
    assert_eq!(artifact.contract.r31_control.race, R31_MODEL_RESREF);
    assert_eq!(artifact.contract.h1_control.physical_row, 15_101);
    assert_eq!(artifact.contract.h1_control.race, H1_MODEL_RESREF);
    assert_eq!(
        artifact
            .contract
            .resource_order
            .iter()
            .map(|resource| (resource.resref.as_str(), resource.resource_type))
            .collect::<Vec<_>>(),
        TRI_CONTROL_RESOURCE_ORDER_V1
    );
    assert_eq!(artifact.contract.resources.len(), 5);
    assert_eq!(artifact.contract.hak.byte_length, artifact.hak.len() as u64);
    assert_eq!(artifact.contract.hak.sha256, sha256(&artifact.hak));
    verify(&artifact.contract, &artifact.hak, &input).expect("independent exact replay");

    let limits = two_da::TwoDaLimitsV1::default();
    let h1_source = two_da::read_two_da_row_v2(&input.h1_appearance, 15_100, &limits)
        .expect("exact H1 source row15100");
    let cloned = two_da::read_two_da_row_v2(&artifact.appearance_two_da, 15_101, &limits)
        .expect("cloned H1 row15101");
    assert_eq!(
        cloned.cells, h1_source.cells,
        "clone every H1 column verbatim"
    );

    let archive = erf::ErfArchive::parse(&artifact.hak).expect("own HAK readback");
    assert_eq!(
        archive
            .resources()
            .iter()
            .map(|resource| (resource.resref.as_str(), resource.resource_type))
            .collect::<Vec<_>>(),
        vec![
            ("appearance", 2017),
            (R31_MODEL_RESREF, 2002),
            (R31_TEXTURE_RESREF, 3),
            (H1_MODEL_RESREF, 2002),
            (H1_TEXTURE_RESREF, 3),
        ]
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact Last City/r31/H1 artifacts"]
fn exact_last_city_v2_appends_after_full_source_and_preserves_every_bound_payload() {
    let input = exact_last_city_inputs();
    let artifact = build_tri_control_hak_last_city_v2(
        &input.base_appearance,
        &input.r31_mdl,
        &input.r31_tga,
        &input.h1_appearance,
        &input.h1_mdl,
        &input.h1_tga,
    )
    .expect("exact Last City tri-control V2 inputs");

    let limits = two_da::TwoDaLimitsV1::default();
    let source = two_da::inspect_two_da_v2(&input.base_appearance, &limits)
        .expect("exact Last City source inspection");
    let output = two_da::inspect_two_da_v2(&artifact.appearance_two_da, &limits)
        .expect("exact Last City V2 output inspection");
    assert_eq!(source.physical_row_count, LAST_CITY_SOURCE_PHYSICAL_ROWS_V2);
    assert_eq!(output.physical_row_count, LAST_CITY_OUTPUT_PHYSICAL_ROWS_V2);
    assert!(
        artifact
            .appearance_two_da
            .starts_with(&input.base_appearance)
    );
    assert_eq!(artifact.appearance_two_da.len(), 7_655_712);
    assert_eq!(
        sha256(&artifact.appearance_two_da),
        "f45437470d554d7ecd5e8fef15db0bbd799621308d625fad388950cf5f276c43"
    );
    assert_eq!(artifact.contract.source_physical_rows, 15_219);
    assert_eq!(artifact.contract.output_physical_rows, 15_221);
    assert_eq!(
        artifact.contract.r31_control.physical_row,
        LAST_CITY_M0_APPEARANCE_ROW_V2
    );
    assert_eq!(
        artifact.contract.h1_control.physical_row,
        LAST_CITY_H1_APPEARANCE_ROW_V2
    );
    assert_eq!(
        artifact
            .contract
            .preserved_project_q_rows
            .iter()
            .map(|row| (row.physical_row, row.printed_row_label, row.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (15_100, 15_100, "PROJECT_Q_RESERVED"),
            (15_101, 15_101, "PROJECT_Q_RESERVED")
        ]
    );

    let h1_source = two_da::read_two_da_row_v2(&input.h1_appearance, 15_100, &limits)
        .expect("exact H1 source row15100");
    let h1_appended = two_da::read_two_da_row_v2(
        &artifact.appearance_two_da,
        LAST_CITY_H1_APPEARANCE_ROW_V2,
        &limits,
    )
    .expect("Last City appended H1 row15220");
    assert_eq!(h1_appended.cells, h1_source.cells);

    let archive = erf::ErfArchive::parse(&artifact.hak).expect("own V2 HAK readback");
    for (resref, resource_type, expected) in [
        (R31_MODEL_RESREF, 2002, input.r31_mdl.as_slice()),
        (R31_TEXTURE_RESREF, 3, input.r31_tga.as_slice()),
        (H1_MODEL_RESREF, 2002, input.h1_mdl.as_slice()),
        (H1_TEXTURE_RESREF, 3, input.h1_tga.as_slice()),
    ] {
        assert_eq!(archive.find(resref, resource_type).unwrap(), expected);
    }
    verify_tri_control_hak_last_city_v2(
        &artifact.contract,
        &artifact.hak,
        &input.base_appearance,
        &input.r31_mdl,
        &input.r31_tga,
        &input.h1_appearance,
        &input.h1_mdl,
        &input.h1_tga,
    )
    .expect("independent Last City V2 replay");
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact Last City/r31/H1 artifacts"]
fn last_city_v2_rejects_retail_base_drift_prefix_mutation_and_row_shift() {
    let input = exact_last_city_inputs();
    let artifact = build_tri_control_hak_last_city_v2(
        &input.base_appearance,
        &input.r31_mdl,
        &input.r31_tga,
        &input.h1_appearance,
        &input.h1_mdl,
        &input.h1_tga,
    )
    .expect("exact Last City tri-control V2 inputs");

    let retail = exact_inputs();
    assert_code(
        build_tri_control_hak_last_city_v2(
            &retail.base_appearance,
            &input.r31_mdl,
            &input.r31_tga,
            &input.h1_appearance,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("retail 15100-row base must not enter Last City V2"),
        "M2A-TRI-CONTROL-INPUT-IDENTITY",
    );

    let mut drift = input.base_appearance.clone();
    let drift_index = drift.len() / 2;
    drift[drift_index] ^= 1;
    assert_code(
        build_tri_control_hak_last_city_v2(
            &drift,
            &input.r31_mdl,
            &input.r31_tga,
            &input.h1_appearance,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("same-size Last City source drift must fail pinned identity"),
        "M2A-TRI-CONTROL-INPUT-IDENTITY",
    );

    let mut wrong_prefix = artifact.appearance_two_da.clone();
    replace_token_in_physical_row(
        &mut wrong_prefix,
        15_100,
        b"PROJECT_Q_RESERVED",
        b"PROJECT_Q_RESERVEd",
    );
    let wrong_prefix_hak = write_hak(&[
        ("appearance", 2017, wrong_prefix),
        (R31_MODEL_RESREF, 2002, input.r31_mdl.clone()),
        (R31_TEXTURE_RESREF, 3, input.r31_tga.clone()),
        (H1_MODEL_RESREF, 2002, input.h1_mdl.clone()),
        (H1_TEXTURE_RESREF, 3, input.h1_tga.clone()),
    ]);
    assert_code(
        verify_tri_control_hak_last_city_v2(
            &artifact.contract,
            &wrong_prefix_hak,
            &input.base_appearance,
            &input.r31_mdl,
            &input.r31_tga,
            &input.h1_appearance,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("reserved-row source-prefix mutation must fail before replay"),
        "M2A-TRI-CONTROL-LAST-CITY-PREFIX",
    );

    let mut shifted = artifact.appearance_two_da.clone();
    replace_token_in_physical_row(
        &mut shifted,
        LAST_CITY_M0_APPEARANCE_ROW_V2 as usize,
        b"m2a_m0p01",
        b"m2a_m6p01",
    );
    replace_token_in_physical_row(
        &mut shifted,
        LAST_CITY_H1_APPEARANCE_ROW_V2 as usize,
        b"m2a_m6p01",
        b"m2a_m0p01",
    );
    let shifted_hak = write_hak(&[
        ("appearance", 2017, shifted),
        (R31_MODEL_RESREF, 2002, input.r31_mdl.clone()),
        (R31_TEXTURE_RESREF, 3, input.r31_tga.clone()),
        (H1_MODEL_RESREF, 2002, input.h1_mdl.clone()),
        (H1_TEXTURE_RESREF, 3, input.h1_tga.clone()),
    ]);
    assert_code(
        verify_tri_control_hak_last_city_v2(
            &artifact.contract,
            &shifted_hak,
            &input.base_appearance,
            &input.r31_mdl,
            &input.r31_tga,
            &input.h1_appearance,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("M0/H1 row shift must fail exact physical-row binding"),
        "M2A-TRI-CONTROL-ROW-BINDING",
    );
}

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact project-owned r31/H1 artifacts"]
fn tri_control_hak_rejects_identity_row_resource_order_and_payload_attacks() {
    let input = exact_inputs();
    let artifact = build(&input).expect("exact pinned tri-control inputs");

    let mut wrong_file_type = artifact.hak.clone();
    wrong_file_type[..4].copy_from_slice(b"MOD ");
    assert_code(
        verify(&artifact.contract, &wrong_file_type, &input)
            .expect_err("HAK resource bytes in a MOD container must fail before replay"),
        "M2A-TRI-CONTROL-HAK-SIGNATURE",
    );

    let mut wrong_identity = input.r31_mdl.clone();
    wrong_identity[0] ^= 1;
    assert_code(
        build_tri_control_hak_v1(
            &input.base_appearance,
            &wrong_identity,
            &input.r31_tga,
            &input.h1_appearance,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("changed r31 MDL identity must fail"),
        "M2A-TRI-CONTROL-INPUT-IDENTITY",
    );

    let mut wrong_h1_appearance_identity = input.h1_appearance.clone();
    wrong_h1_appearance_identity[0] ^= 1;
    assert_code(
        build_tri_control_hak_v1(
            &input.base_appearance,
            &input.r31_mdl,
            &input.r31_tga,
            &wrong_h1_appearance_identity,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("changed H1 appearance identity must fail"),
        "M2A-TRI-CONTROL-INPUT-IDENTITY",
    );

    let mut wrong_base_row = input.base_appearance.clone();
    replace_token_in_physical_row(&mut wrong_base_row, 102, b"c_horror", b"c_horr0r");
    assert_code(
        build_tri_control_hak_v1(
            &wrong_base_row,
            &input.r31_mdl,
            &input.r31_tga,
            &input.h1_appearance,
            &input.h1_mdl,
            &input.h1_tga,
        )
        .expect_err("wrong stock row102 binding must fail"),
        "M2A-TRI-CONTROL-ROW-BINDING",
    );

    let missing_resource = write_hak(&[
        ("appearance", 2017, artifact.appearance_two_da.clone()),
        (R31_MODEL_RESREF, 2002, input.r31_mdl.clone()),
        (R31_TEXTURE_RESREF, 3, input.r31_tga.clone()),
        (H1_MODEL_RESREF, 2002, input.h1_mdl.clone()),
    ]);
    assert_code(
        verify(&artifact.contract, &missing_resource, &input)
            .expect_err("missing H1 TGA must fail exact resource set"),
        "M2A-TRI-CONTROL-RESOURCE-SET",
    );

    let mut wrong_order = artifact.hak.clone();
    let key_start = u32::from_le_bytes(wrong_order[24..28].try_into().unwrap()) as usize;
    let key_size = 24;
    let first = wrong_order[key_start..key_start + key_size].to_vec();
    let second = wrong_order[key_start + key_size..key_start + key_size * 2].to_vec();
    wrong_order[key_start..key_start + key_size].copy_from_slice(&second);
    wrong_order[key_start + key_size..key_start + key_size * 2].copy_from_slice(&first);
    wrong_order[key_start + 16..key_start + 20].copy_from_slice(&0_u32.to_le_bytes());
    wrong_order[key_start + key_size + 16..key_start + key_size + 20]
        .copy_from_slice(&1_u32.to_le_bytes());
    assert_code(
        verify(&artifact.contract, &wrong_order, &input)
            .expect_err("key-table reordering must fail"),
        "M2A-TRI-CONTROL-RESOURCE-ORDER",
    );

    let mut wrong_h1_row = artifact.appearance_two_da.clone();
    replace_token_in_physical_row(&mut wrong_h1_row, 15_101, b"m2a_m6p01", b"m2a_m6x01");
    let wrong_row_hak = write_hak(&[
        ("appearance", 2017, wrong_h1_row),
        (R31_MODEL_RESREF, 2002, input.r31_mdl.clone()),
        (R31_TEXTURE_RESREF, 3, input.r31_tga.clone()),
        (H1_MODEL_RESREF, 2002, input.h1_mdl.clone()),
        (H1_TEXTURE_RESREF, 3, input.h1_tga.clone()),
    ]);
    assert_code(
        verify(&artifact.contract, &wrong_row_hak, &input)
            .expect_err("changed appended H1 row must fail semantic readback"),
        "M2A-TRI-CONTROL-ROW-BINDING",
    );

    let mut wrong_payload = input.r31_mdl.clone();
    let last = wrong_payload.len() - 1;
    wrong_payload[last] ^= 1;
    let wrong_payload_hak = write_hak(&[
        ("appearance", 2017, artifact.appearance_two_da.clone()),
        (R31_MODEL_RESREF, 2002, wrong_payload),
        (R31_TEXTURE_RESREF, 3, input.r31_tga.clone()),
        (H1_MODEL_RESREF, 2002, input.h1_mdl.clone()),
        (H1_TEXTURE_RESREF, 3, input.h1_tga.clone()),
    ]);
    assert_code(
        verify(&artifact.contract, &wrong_payload_hak, &input)
            .expect_err("changed model payload must fail exact HAK readback"),
        "M2A-TRI-CONTROL-RESOURCE-PAYLOAD",
    );

    let mut promoted = artifact.contract.clone();
    promoted.diagnostic_only = false;
    promoted.renderer_verified = true;
    assert_code(
        verify(&promoted, &artifact.hak, &input)
            .expect_err("offline diagnostic must not self-promote"),
        "M2A-TRI-CONTROL-CONTRACT",
    );
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
        h1_appearance: fs::read(h1.join("appearance.2da")).expect("exact H1 v20 appearance"),
        h1_mdl: fs::read(h1.join("m2a_m6p01.mdl")).expect("exact H1 v20 MDL"),
        h1_tga: fs::read(h1.join("m2a_m6t01.tga")).expect("exact H1 v20 TGA"),
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

fn build(
    input: &Inputs,
) -> Result<tri_control_hak::TriControlHakArtifactV1, tri_control_hak::TriControlHakErrorV1> {
    build_tri_control_hak_v1(
        &input.base_appearance,
        &input.r31_mdl,
        &input.r31_tga,
        &input.h1_appearance,
        &input.h1_mdl,
        &input.h1_tga,
    )
}

fn verify(
    contract: &tri_control_hak::TriControlHakContractV1,
    hak: &[u8],
    input: &Inputs,
) -> Result<(), tri_control_hak::TriControlHakErrorV1> {
    verify_tri_control_hak_v1(
        contract,
        hak,
        &input.base_appearance,
        &input.r31_mdl,
        &input.r31_tga,
        &input.h1_appearance,
        &input.h1_mdl,
        &input.h1_tga,
    )
}

fn write_hak(resources: &[(&str, u16, Vec<u8>)]) -> Vec<u8> {
    write_hak_v1(
        &resources
            .iter()
            .map(|(resref, resource_type, payload)| HakResourceInputV1 {
                resref: (*resref).to_owned(),
                resource_type: *resource_type,
                payload: payload.clone(),
            })
            .collect::<Vec<_>>(),
        &HakWriterOptionsV1::default(),
    )
    .expect("own deterministic HAK writer")
    .payload
}

fn replace_token_in_physical_row(bytes: &mut [u8], row: usize, from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let mut starts = vec![0usize];
    starts.extend(
        bytes
            .iter()
            .enumerate()
            .filter_map(|(index, byte)| (*byte == b'\n').then_some(index + 1)),
    );
    let start = starts[row + 3];
    let end = starts.get(row + 4).copied().unwrap_or(bytes.len());
    let offset = bytes[start..end]
        .windows(from.len())
        .position(|window| window == from)
        .expect("token in exact physical row");
    bytes[start + offset..start + offset + from.len()].copy_from_slice(to);
}

fn assert_code(error: tri_control_hak::TriControlHakErrorV1, code: &str) {
    assert_eq!(error.code, code, "unexpected error: {error}");
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn resource_order_constant_has_no_duplicates() {
    assert_eq!(
        TRI_CONTROL_RESOURCE_ORDER_V1
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        TRI_CONTROL_RESOURCE_ORDER_V1.len()
    );
}
