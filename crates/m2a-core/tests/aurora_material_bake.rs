use m2a_core::{
    aurora_material::{
        AuroraClassicDiffuseBakeOperationV1, AuroraClassicDiffuseBakePlanV1,
        AuroraSpecularBakeOperationV1, AuroraSpecularTexturePlanV1,
    },
    aurora_material_bake::{
        AuroraMaterialBakeKindV1, bake_classic_diffuse_v1, bake_specular_gloss_v1,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, read_tga_image_v1},
};

fn rgb(pixels: Vec<u8>) -> TgaImageV1 {
    TgaImageV1 {
        schema_version: 1,
        width: 2,
        height: 1,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels,
    }
}

#[test]
fn classic_bake_produces_real_pixels_and_verified_tga_readback() {
    let base = rgb(vec![200, 100, 50, 80, 160, 240]);
    let mr = rgb(vec![0, 255, 255, 0, 0, 0]);
    let plan = AuroraClassicDiffuseBakePlanV1 {
        operation: AuroraClassicDiffuseBakeOperationV1::PbrEnergyApproximation,
        source_texture_ids: vec![11],
        metallic_factor: 1.0,
        roughness_factor: 1.0,
    };
    let artifact = bake_classic_diffuse_v1(&plan, &base, Some(&mr)).unwrap();
    assert_eq!(
        artifact.report.kind,
        AuroraMaterialBakeKindV1::ClassicDiffusePbrEnergyApproximation
    );
    assert_ne!(artifact.image.pixels, base.pixels);
    assert!(artifact.report.semantic_readback_equal);
    assert_eq!(
        artifact.report.output_pixel_sha256,
        artifact.report.readback_pixel_sha256
    );
    assert_eq!(
        read_tga_image_v1(&artifact.tga.payload).unwrap(),
        artifact.image
    );
}

#[test]
fn ee_specular_gloss_uses_gltf_green_roughness_blue_metallic() {
    let base = rgb(vec![255, 0, 0, 128, 128, 128]);
    let mr = rgb(vec![0, 64, 255, 0, 255, 0]);
    let plan = AuroraSpecularTexturePlanV1 {
        operation: AuroraSpecularBakeOperationV1::MetallicRoughnessToSpecularGloss,
        source_texture_ids: vec![11],
        tex_coord_set: 0,
        metallic_factor: 1.0,
        roughness_factor: 1.0,
    };
    let artifact = bake_specular_gloss_v1(&plan, &base, Some(&mr)).unwrap();
    assert_eq!(artifact.image.pixel_format, TgaPixelFormatV1::Rgba8);
    assert!(artifact.image.pixels[0] > 245);
    assert!(artifact.image.pixels[1] < 10);
    assert_eq!(artifact.image.pixels[3], 191);
    assert!(artifact.image.pixels[4] < 70);
    assert_eq!(artifact.image.pixels[7], 0);
}

#[test]
fn bake_resamples_dimension_mismatch_and_rejects_out_of_range_factors() {
    let base = rgb(vec![0; 6]);
    let mismatched = TgaImageV1 {
        schema_version: 1,
        width: 1,
        height: 1,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: vec![0; 3],
    };
    let mut plan = AuroraClassicDiffuseBakePlanV1 {
        operation: AuroraClassicDiffuseBakeOperationV1::PbrEnergyApproximation,
        source_texture_ids: vec![],
        metallic_factor: 0.0,
        roughness_factor: 1.0,
    };
    let resampled = bake_classic_diffuse_v1(&plan, &base, Some(&mismatched)).unwrap();
    assert!(resampled.report.metallic_roughness_resampled);
    assert_eq!(resampled.report.metallic_roughness_source_width, Some(1));
    assert_eq!(resampled.report.metallic_roughness_source_height, Some(1));
    assert_eq!((resampled.report.width, resampled.report.height), (2, 1));
    plan.metallic_factor = 1.2;
    assert_eq!(
        bake_classic_diffuse_v1(&plan, &base, None)
            .unwrap_err()
            .code,
        "AURORA-BAKE-FACTOR-RANGE"
    );
}
