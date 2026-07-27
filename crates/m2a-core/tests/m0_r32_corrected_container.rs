use std::{fs, path::PathBuf};

use m2a_core::{
    erf::ErfArchive,
    gff::{GffValueV1, read_gff_v32},
    hierarchy_candidate::{M0_R31_HAK_SHA256, build_meshy_r31_corrected_container_module_v1},
    proof_module::{
        BinaryM0VerticalSliceIdentityV1, inspect_binary_creature_multi_fixture_module_v1,
    },
};
use sha2::{Digest, Sha256};

const R31_MODULE_SHA256: &str = "8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd";

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and exact frozen r31/native HAK bytes"]
fn exact_r31_sparse_container_is_rejected_and_r32_corrected_container_is_runtime_complete() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "exact corrected-container audit must be explicitly forced",
    );
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let r31_module_path =
        repo.join("proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0r31.mod");
    let r31_hak_path =
        repo.join("proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0r31.hak");
    let installed_hak_path =
        PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r31.hak");
    let r31_module = fs::read(r31_module_path).expect("exact frozen r31 MOD");
    let r31_hak = fs::read(r31_hak_path).expect("exact frozen r31 HAK");
    let installed_hak = fs::read(installed_hak_path).expect("exact installed r31 HAK");
    assert_eq!(sha256(&r31_module), R31_MODULE_SHA256);
    assert_eq!(sha256(&r31_hak), M0_R31_HAK_SHA256);
    assert_eq!(
        installed_hak, r31_hak,
        "r32 must reuse the installed r31 HAK byte-for-byte"
    );

    let error = inspect_binary_creature_multi_fixture_module_v1(&r31_module)
        .expect_err("historical r31 sparse creature envelope must remain a frozen negative");
    assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-READBACK-INVALID");
    assert_eq!(error.path, "area.git.Creature List[0].MaxHitPoints");

    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_m0r32".to_owned(),
        area_resref: "m2a_m0a32".to_owned(),
        hak_resref: "m2a_m0r31".to_owned(),
    };
    let corrected = build_meshy_r31_corrected_container_module_v1(&identity)
        .expect("exact r31 payload in a runtime-complete r32 MOD container");
    let readback = inspect_binary_creature_multi_fixture_module_v1(&corrected.payload)
        .expect("independent corrected-container readback");
    assert_eq!(readback, corrected.readback);
    assert_eq!(readback.module_resref, "m2a_m0r32");
    assert_eq!(readback.area_resref, "m2a_m0a32");
    assert_eq!(readback.ordered_hak_resrefs, ["m2a_m0r31"]);
    assert_eq!(readback.entry_position.x, 10.0);
    assert_eq!(readback.entry_position.y, 10.0);
    assert_eq!(readback.area_width, 2);
    assert_eq!(readback.area_height, 2);
    assert_eq!(readback.fixtures.len(), 1);
    let fixture = &readback.fixtures[0];
    assert_eq!(fixture.id, "m0_fixture");
    assert_eq!(fixture.template_resref, "nw_dwarfmerc001");
    assert_eq!(
        fixture.display_name,
        "Meshy M0 binary vertical-slice fixture"
    );
    assert_eq!(fixture.appearance_row, 15_100);
    assert_eq!(
        [fixture.position.x, fixture.position.y, fixture.position.z],
        [10.0, 14.5, 0.0]
    );
    assert_eq!([fixture.orientation.x, fixture.orientation.y], [1.0, 0.0]);

    let archive = ErfArchive::parse(&corrected.payload).expect("corrected MOD archive");
    assert_eq!(archive.resources().len(), 6);
    assert!(
        archive
            .resources()
            .iter()
            .any(|resource| resource.resref == "nw_dwarfmerc001" && resource.resource_type == 2027)
    );
    let git = read_gff_v32(
        archive.find("m2a_m0a32", 2023).unwrap(),
        &Default::default(),
    )
    .expect("corrected GIT");
    let utc = read_gff_v32(
        archive.find("nw_dwarfmerc001", 2027).unwrap(),
        &Default::default(),
    )
    .expect("corrected UTC");
    let creatures = list_field(&git.root.fields, "Creature List");
    assert_eq!(creatures.len(), 1);
    assert_runtime_complete(&creatures[0].fields);
    assert_runtime_complete(&utc.root.fields);

    assert!(
        !repo
            .join("proof-output/m0-r32-corrected-container-20260722/generated/m2a_m0r32.hak")
            .exists()
    );
    assert!(
        !PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r32.hak").exists()
    );
}

fn list_field<'a>(
    fields: &'a [m2a_core::gff::GffFieldV1],
    label: &str,
) -> &'a [m2a_core::gff::GffStructV1] {
    match fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
    {
        Some(GffValueV1::List(values)) => values,
        _ => panic!("missing list field {label}"),
    }
}

fn assert_runtime_complete(fields: &[m2a_core::gff::GffFieldV1]) {
    assert!(
        fields
            .iter()
            .any(|field| field.label == "MaxHitPoints" && field.value == GffValueV1::Short(13))
    );
    assert!(matches!(
        fields.iter().find(|field| field.label == "SkillList").map(|field| &field.value),
        Some(GffValueV1::List(skills))
            if skills.len() == 28
                && skills.iter().all(|skill| skill.struct_id == 0
                    && skill.fields.as_slice() == [m2a_core::gff::GffFieldV1 {
                        label: "Rank".to_owned(),
                        value: GffValueV1::Byte(0),
                    }])
    ));
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
