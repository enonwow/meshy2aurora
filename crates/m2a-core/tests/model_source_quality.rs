#[path = "fixtures/build_synthetic_glb.rs"]
mod build_synthetic_glb;

use m2a_core::{
    glb::{GlbLimits, ingest_glb},
    model_source_quality::{
        ModelSourceQualityInputV1, ModelSourceQualityStatusV1, SourceMaterialQualityInputV1,
        inspect_model_source_quality_v1,
    },
    model_texture_authoring::{TextureMipReadabilityStatusV1, TextureMipReadabilityV1},
};

fn readability(status: TextureMipReadabilityStatusV1) -> TextureMipReadabilityV1 {
    TextureMipReadabilityV1 {
        source_width: 1024,
        source_height: 1024,
        base_luma_stddev_milli: 24_000,
        mip_16_luma_stddev_milli: if status == TextureMipReadabilityStatusV1::Readable {
            12_000
        } else {
            0
        },
        contrast_retention_basis_points: if status == TextureMipReadabilityStatusV1::Readable {
            5_000
        } else {
            0
        },
        status,
    }
}

#[test]
fn source_quality_reports_uv_density_fragmentation_and_contrast_deterministically() {
    let ingest = ingest_glb(
        &build_synthetic_glb::one_primitive_two_disconnected_triangles_with_embedded_texture(),
        &GlbLimits::default(),
    )
    .expect("source ingest");
    let material_id = ingest.ir.primitives[0].material_id.expect("material");
    let input = ModelSourceQualityInputV1 {
        schema_version: 1,
        source_sha256: ingest.ir.source.sha256.clone(),
        materials: vec![SourceMaterialQualityInputV1 {
            material_id,
            texture_readability: Some(readability(TextureMipReadabilityStatusV1::Readable)),
        }],
    };
    let first = inspect_model_source_quality_v1(&ingest.ir, &input).expect("quality gate");
    let second = inspect_model_source_quality_v1(&ingest.ir, &input).expect("repeat quality gate");
    assert_eq!(first, second);
    assert_eq!(first.triangle_count, 2);
    assert_eq!(first.component_count, 2);
    assert_eq!(first.missing_uv_triangle_count, 0);
    assert_eq!(first.degenerate_uv_triangle_count, 0);
    assert!(first.weighted_texels_per_meter > 0);
}

#[test]
fn flat_texture_is_blocked_and_source_identity_is_fail_closed() {
    let ingest = ingest_glb(
        &build_synthetic_glb::one_primitive_two_disconnected_triangles_with_embedded_texture(),
        &GlbLimits::default(),
    )
    .expect("source ingest");
    let material_id = ingest.ir.primitives[0].material_id.expect("material");
    let mut input = ModelSourceQualityInputV1 {
        schema_version: 1,
        source_sha256: ingest.ir.source.sha256.clone(),
        materials: vec![SourceMaterialQualityInputV1 {
            material_id,
            texture_readability: Some(readability(TextureMipReadabilityStatusV1::Flat)),
        }],
    };
    let report = inspect_model_source_quality_v1(&ingest.ir, &input).expect("quality gate");
    assert_eq!(report.status, ModelSourceQualityStatusV1::Blocked);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|entry| entry.code == "SOURCE-QUALITY-CONTRAST-FLAT")
    );
    input.source_sha256 = "f".repeat(64);
    let error = inspect_model_source_quality_v1(&ingest.ir, &input).unwrap_err();
    assert_eq!(error.code, "SOURCE-QUALITY-SOURCE-MISMATCH");
}
