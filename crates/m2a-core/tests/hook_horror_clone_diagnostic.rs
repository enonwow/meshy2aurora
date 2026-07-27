use std::fs;

use m2a_core::{
    erf::ErfArchive,
    hook_horror_clone_diagnostic::{
        HOOK_HORROR_CLONE_LABEL_V1, HOOK_HORROR_SOURCE_ROW_V1,
        build_hook_horror_clone_diagnostic_v1, verify_hook_horror_clone_diagnostic_v1,
    },
    proof_module::{
        BinaryM0VerticalSliceIdentityV1, inspect_binary_creature_multi_fixture_module_v1,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};

const LAST_CITY_APPEARANCE: &str = "C:\\Users\\enonw\\Pictures\\appearance.2da";

#[test]
fn exact_hook_horror_control_clones_all_35_cells_and_changes_only_label() {
    let source = fs::read(LAST_CITY_APPEARANCE).expect("owner-provided Last City appearance.2da");
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_van01".to_owned(),
        area_resref: "m2a_vana01".to_owned(),
        hak_resref: "m2a_vanh01".to_owned(),
    };
    let artifact = build_hook_horror_clone_diagnostic_v1(&source, &identity)
        .expect("exact Hook Horror clone diagnostic");

    verify_hook_horror_clone_diagnostic_v1(&artifact, &source, &identity)
        .expect("independent deterministic replay");
    let runtime_complete = inspect_binary_creature_multi_fixture_module_v1(&artifact.module)
        .expect("the diagnostic must use the runtime-complete creature GIT/UTC envelope");
    assert_eq!(runtime_complete.fixtures.len(), 1);
    assert_eq!(runtime_complete.fixtures[0].appearance_row, 15_219);
    assert_eq!(artifact.contract.source_row, HOOK_HORROR_SOURCE_ROW_V1);
    assert_eq!(artifact.contract.clone_row, 15_219);
    assert_eq!(artifact.contract.changed_columns, ["LABEL"]);
    assert!(artifact.appearance_two_da.starts_with(&source));

    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(&artifact.appearance_two_da, &limits).unwrap();
    assert_eq!(inspection.columns.len(), 35);
    let source_row = read_two_da_row_v2(&source, HOOK_HORROR_SOURCE_ROW_V1, &limits).unwrap();
    let clone_row = read_two_da_row_v2(&artifact.appearance_two_da, 15_219, &limits).unwrap();
    assert_eq!(source_row.cells.len(), 35);
    assert_eq!(clone_row.cells.len(), 35);
    for (index, (before, after)) in source_row.cells.iter().zip(&clone_row.cells).enumerate() {
        let column = &inspection.columns[index];
        if column == "LABEL" {
            assert_eq!(
                after,
                &TwoDaCellValueV1::Text {
                    value: HOOK_HORROR_CLONE_LABEL_V1.to_owned(),
                }
            );
        } else {
            assert_eq!(after, before, "unexpected mutation in {column}");
        }
    }

    let hak = ErfArchive::parse(&artifact.hak).unwrap();
    assert_eq!(hak.resources().len(), 1);
    assert_eq!(hak.resources()[0].resref, "appearance");
    assert_eq!(hak.resources()[0].resource_type, 2017);
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        artifact.appearance_two_da
    );

    assert_eq!(artifact.contract.module_readback.fixtures.len(), 1);
    assert_eq!(
        artifact.contract.module_readback.fixtures[0].appearance_row,
        15_219
    );
    assert_eq!(
        artifact.contract.module_readback.ordered_hak_resrefs,
        ["m2a_vanh01"]
    );
}
