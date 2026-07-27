use m2a_core::proof_module::{
    BinaryM0VerticalSliceIdentityV1, M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y,
    M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z, M0_RUNTIME_FIXTURE_X,
    M0_RUNTIME_FIXTURE_Y, M0_RUNTIME_FIXTURE_Z,
    build_binary_m0_vertical_slice_module_with_identity_v1,
    inspect_binary_m0_vertical_slice_module_v1,
};

const MODULE_RESREF: &str = "m2a_m0r28";
const AREA_RESREF: &str = "m2a_m0a28";
const ONLY_HAK_RESREF: &str = "m2a_m0r27";
const FIXTURE_NAME: &str = "Meshy M0 binary vertical-slice fixture";

#[test]
fn r28_removes_multi_hak_ambiguity_without_changing_fixture_semantics() {
    let artifact = build_binary_m0_vertical_slice_module_with_identity_v1(
        848,
        &BinaryM0VerticalSliceIdentityV1 {
            module_resref: MODULE_RESREF.to_owned(),
            area_resref: AREA_RESREF.to_owned(),
            hak_resref: ONLY_HAK_RESREF.to_owned(),
        },
    )
    .expect("single-r27-HAK module must materialize");
    let scene = inspect_binary_m0_vertical_slice_module_v1(&artifact.payload)
        .expect("generated module must pass independent semantic readback");

    assert_eq!(scene.module_resref, MODULE_RESREF);
    assert_eq!(scene.area_resref, AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [ONLY_HAK_RESREF]);
    assert_eq!(scene.entry_position.x, M0_RUNTIME_ENTRY_X);
    assert_eq!(scene.entry_position.y, M0_RUNTIME_ENTRY_Y);
    assert_eq!(scene.entry_position.z, M0_RUNTIME_ENTRY_Z);
    assert_eq!(scene.entry_direction.x, M0_RUNTIME_ENTRY_DIR_X);
    assert_eq!(scene.entry_direction.y, M0_RUNTIME_ENTRY_DIR_Y);
    assert_eq!(scene.fixture.template_resref, "nw_dwarfmerc001");
    assert_eq!(scene.fixture.appearance_row, 848);
    assert_eq!(scene.fixture.position.x, M0_RUNTIME_FIXTURE_X);
    assert_eq!(scene.fixture.position.y, M0_RUNTIME_FIXTURE_Y);
    assert_eq!(scene.fixture.position.z, M0_RUNTIME_FIXTURE_Z);
    assert_eq!(scene.fixture.orientation.x, 1.0);
    assert_eq!(scene.fixture.orientation.y, 0.0);

    assert_eq!(count_bytes(&artifact.payload, FIXTURE_NAME.as_bytes()), 2);
    assert_eq!(count_bytes(&artifact.payload, b"Dwarf Mercenary"), 0);
    assert_eq!(count_bytes(&artifact.payload, b"m2a_m0r26"), 0);
    assert_eq!(count_bytes(&artifact.payload, b"m2a_m0r21"), 0);
}

fn count_bytes(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|candidate| *candidate == needle)
        .count()
}
