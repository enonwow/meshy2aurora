use m2a_core::{
    glb::{GlbLimits, ingest_glb},
    model_material_capabilities::ModelRenderTargetV1,
    model_material_separation::default_model_material_separation_v1,
    model_part::{ModelPartInputV1, resolve_model_part_input_v1},
};

#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod fixtures;

#[test]
fn model_part_input_resolves_through_the_target_neutral_material_core() {
    let source = fixtures::one_primitive_four_disconnected_triangles();
    let ingest = ingest_glb(&source, &GlbLimits::default()).expect("owned GLB");
    let document = default_model_material_separation_v1(&ingest.ir);

    let resolved = resolve_model_part_input_v1(ModelPartInputV1 {
        schema_version: 1,
        part_id: "armor:torso",
        source_ir: &ingest.ir,
        material_separation: &document,
    })
    .expect("target-neutral ModelPart input");

    assert_eq!(resolved.schema_version, 1);
    assert_eq!(resolved.part_id, "armor:torso");
    assert_eq!(resolved.capabilities.target, ModelRenderTargetV1::ModelPart);
    assert_eq!(resolved.materials.report.source_triangle_count, 4);
    assert_eq!(resolved.materials.report.output_triangle_count, 4);
    assert_eq!(
        resolved.materials.material_slot_for_triangle(0, 0, 0, 0),
        Some(0)
    );
}

#[test]
fn model_part_input_fails_closed_on_schema_identity_and_part_id() {
    let source = fixtures::one_primitive_four_disconnected_triangles();
    let ingest = ingest_glb(&source, &GlbLimits::default()).expect("owned GLB");
    let document = default_model_material_separation_v1(&ingest.ir);

    let schema_error = resolve_model_part_input_v1(ModelPartInputV1 {
        schema_version: 2,
        part_id: "armor:torso",
        source_ir: &ingest.ir,
        material_separation: &document,
    })
    .expect_err("unsupported schema");
    assert_eq!(schema_error.code, "MODEL-PART-INPUT-SCHEMA-UNSUPPORTED");

    let id_error = resolve_model_part_input_v1(ModelPartInputV1 {
        schema_version: 1,
        part_id: "not a stable part id",
        source_ir: &ingest.ir,
        material_separation: &document,
    })
    .expect_err("invalid part id");
    assert_eq!(id_error.code, "MODEL-PART-INPUT-ID-INVALID");

    let mut stale = document.clone();
    stale.source_sha256 = "0".repeat(64);
    let stale_error = resolve_model_part_input_v1(ModelPartInputV1 {
        schema_version: 1,
        part_id: "armor:torso",
        source_ir: &ingest.ir,
        material_separation: &stale,
    })
    .expect_err("stale source identity");
    assert_eq!(stale_error.code, "MATERIAL-SEPARATION-SOURCE-MISMATCH");
}
