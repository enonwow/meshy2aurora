#[path = "fixtures/build_synthetic_glb.rs"]
mod build_synthetic_glb;

use m2a_core::{
    erf::ErfArchive,
    model_material_e2e::{ModelMaterialE2eRequestV1, build_model_material_e2e_v1},
    proof_module::inspect_binary_m0_vertical_slice_module_v1,
};

fn appearance_fixture() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n".to_vec()
}

fn request() -> ModelMaterialE2eRequestV1 {
    ModelMaterialE2eRequestV1 {
        schema_version: 1,
        model_resref: "m2a_e2emdl".to_owned(),
        hak_resref: "m2a_e2ehak".to_owned(),
        module_resref: "m2a_e2emod".to_owned(),
        area_resref: "m2a_e2ear".to_owned(),
    }
}

#[test]
fn glb_materials_mdl_tga_mtr_txi_hak_and_mod_roundtrip_deterministically() {
    let source =
        build_synthetic_glb::one_primitive_two_disconnected_triangles_with_embedded_texture();
    let first = build_model_material_e2e_v1(&source, &appearance_fixture(), &request())
        .expect("first complete material E2E");
    let second = build_model_material_e2e_v1(&source, &appearance_fixture(), &request())
        .expect("second complete material E2E");
    assert_eq!(first.hak.payload, second.hak.payload);
    assert_eq!(first.module.payload, second.module.payload);
    assert_eq!(first.report, second.report);
    assert_eq!(first.report.semantic_readback_status, "PASS");
    assert!(!first.report.ready_for_owner_proof);
    assert_eq!(
        first.report.owner_proof_blocker,
        "ARTIFACTS_NOT_FROZEN_OR_INSTALLED"
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .all(|entry| entry.readback_equal)
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|entry| entry.resource_type == 2002)
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|entry| entry.resource_type == 3)
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|entry| entry.resource_type == 2072)
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|entry| entry.resource_type == 2022)
    );
    let archive = ErfArchive::parse(&first.hak.payload).expect("HAK readback");
    assert_eq!(archive.resources().len(), first.report.resources.len());
    let module =
        inspect_binary_m0_vertical_slice_module_v1(&first.module.payload).expect("MOD readback");
    assert_eq!(module.module_resref, request().module_resref);
    assert_eq!(module.area_resref, request().area_resref);
    assert_eq!(module.ordered_hak_resrefs, [request().hak_resref]);
}

#[test]
fn material_e2e_rejects_noncanonical_identity_before_any_package_write() {
    let mut invalid = request();
    invalid.hak_resref = "INVALID-HAK".to_owned();
    let error = build_model_material_e2e_v1(
        &build_synthetic_glb::one_primitive_two_disconnected_triangles_with_embedded_texture(),
        &appearance_fixture(),
        &invalid,
    )
    .unwrap_err();
    assert_eq!(error.code, "MATERIAL-E2E-RESREF-INVALID");
}
