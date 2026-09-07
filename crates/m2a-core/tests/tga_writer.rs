use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1};
use m2a_core::tga::{
    TGA_MAX_OUTPUT_BYTES, TGA_MAX_PIXEL_BYTES, TextureArtifactCleanupOptionsV1, TgaImageV1,
    TgaPixelFormatV1, TgaWriterLimitsV1, TgaWriterOptionsV1, cleanup_texture_artifacts_v1,
    write_tga_v1,
};

const FOOTER: &[u8] = b"\0\0\0\0\0\0\0\0TRUEVISION-XFILE.\0";

fn rgb_image() -> TgaImageV1 {
    TgaImageV1 {
        schema_version: 1,
        width: 2,
        height: 2,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: vec![
            255, 0, 0, 0, 255, 0, // top: red, green
            0, 0, 255, 255, 255, 255, // bottom: blue, white
        ],
    }
}

fn rgba_image() -> TgaImageV1 {
    TgaImageV1 {
        schema_version: 1,
        width: 2,
        height: 2,
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: vec![
            255, 0, 0, 1, 0, 255, 0, 2, // top
            0, 0, 255, 3, 255, 255, 255, 4, // bottom
        ],
    }
}

fn solid_rgba_image(width: u32, height: u32, pixel: [u8; 4]) -> TgaImageV1 {
    TgaImageV1 {
        schema_version: 1,
        width,
        height,
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: pixel
            .into_iter()
            .cycle()
            .take(width as usize * height as usize * 4)
            .collect(),
    }
}

fn set_rgba(image: &mut TgaImageV1, x: usize, y: usize, pixel: [u8; 4]) {
    let offset = (y * image.width as usize + x) * 4;
    image.pixels[offset..offset + 4].copy_from_slice(&pixel);
}

fn rgba_at(image: &TgaImageV1, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * image.width as usize + x) * 4;
    image.pixels[offset..offset + 4].try_into().unwrap()
}

fn options(max_output_bytes: u64) -> TgaWriterOptionsV1 {
    TgaWriterOptionsV1 {
        schema_version: 1,
        limits: TgaWriterLimitsV1 { max_output_bytes },
    }
}

fn assert_fatal(image: &TgaImageV1, options: &TgaWriterOptionsV1, expected_code: &str) {
    let result = catch_unwind(AssertUnwindSafe(|| write_tga_v1(image, options)));
    let error = result
        .expect("invalid TGA input must not panic")
        .expect_err("invalid TGA input must be fatal");
    assert_eq!(error.code, expected_code, "unexpected error: {error:?}");
    assert_eq!(error.severity, "FATAL");
}

#[test]
fn exact_rgb8_2x2_is_bottom_left_bgr_with_locked_header_footer_and_eof() {
    let artifact = write_tga_v1(&rgb_image(), &TgaWriterOptionsV1::default()).unwrap();
    assert_eq!(
        &artifact.payload[..18],
        &[0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0]
    );
    assert_eq!(
        &artifact.payload[18..30],
        &[
            255, 0, 0, // bottom-left blue -> BGR
            255, 255, 255, // bottom-right white
            0, 0, 255, // top-left red
            0, 255, 0, // top-right green
        ]
    );
    assert_eq!(&artifact.payload[30..], FOOTER);
    assert_eq!(artifact.payload.len(), 56);
    assert_eq!(artifact.report.pixel_depth, 24);
    assert_eq!(artifact.report.descriptor, 0);
    assert_eq!(artifact.report.pixel_data_offset, 18);
    assert_eq!(artifact.report.pixel_data_length, 12);
    assert_eq!(artifact.report.footer_offset, 30);
    assert_eq!(artifact.report.byte_length, 56);
}

#[test]
fn exact_rgba8_2x2_is_bottom_left_bgra_with_locked_header_footer_and_eof() {
    let artifact = write_tga_v1(&rgba_image(), &TgaWriterOptionsV1::default()).unwrap();
    assert_eq!(
        &artifact.payload[..18],
        &[0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 32, 8]
    );
    assert_eq!(
        &artifact.payload[18..34],
        &[
            255, 0, 0, 3, // bottom-left blue
            255, 255, 255, 4, // bottom-right white
            0, 0, 255, 1, // top-left red
            0, 255, 0, 2, // top-right green
        ]
    );
    assert_eq!(&artifact.payload[34..], FOOTER);
    assert_eq!(artifact.payload.len(), 60);
    assert_eq!(artifact.report.pixel_depth, 32);
    assert_eq!(artifact.report.descriptor, 8);
    assert_eq!(artifact.report.pixel_data_length, 16);
    assert_eq!(artifact.report.footer_offset, 34);
    assert_eq!(artifact.report.byte_length, 60);
}

#[test]
fn output_is_deterministic_frozen_and_input_is_immutable() {
    let image = rgba_image();
    let before = image.clone();
    let first = write_tga_v1(&image, &TgaWriterOptionsV1::default()).unwrap();
    let second = write_tga_v1(&image, &TgaWriterOptionsV1::default()).unwrap();
    assert_eq!(image, before);
    assert_eq!(first, second);
    assert_eq!(
        first.report.input_sha256,
        "5be79646f5b2e0be833635fd9c0c3ec6bc98122f6efad830247bcaae21abb66a"
    );
    assert_eq!(
        first.report.output_sha256,
        "ab5365a31f1ef4d57b33943ae01735a33e5337d4d0d6b9eba5b715a3fb360c79"
    );
    assert_eq!(
        serde_json::to_string(&first.report).unwrap(),
        r#"{"schemaVersion":1,"width":2,"height":2,"pixelFormat":"RGBA8","pixelDepth":32,"descriptor":8,"pixelDataOffset":18,"pixelDataLength":16,"footerOffset":34,"byteLength":60,"inputSha256":"5be79646f5b2e0be833635fd9c0c3ec6bc98122f6efad830247bcaae21abb66a","outputSha256":"ab5365a31f1ef4d57b33943ae01735a33e5337d4d0d6b9eba5b715a3fb360c79"}"#
    );
}

#[test]
fn strict_json_contract_uses_exact_pixel_format_tokens_and_rejects_unknown_fields() {
    let image = rgb_image();
    let json = serde_json::to_string(&image).unwrap();
    assert!(json.contains(r#""pixelFormat":"RGB8""#));
    assert_eq!(serde_json::from_str::<TgaImageV1>(&json).unwrap(), image);
    assert!(serde_json::from_str::<TgaImageV1>(&json.replacen('{', "{\"unknown\":1,", 1)).is_err());

    let options = TgaWriterOptionsV1::default();
    let json = serde_json::to_string(&options).unwrap();
    assert!(
        serde_json::from_str::<TgaWriterOptionsV1>(&json.replacen(
            "{\"maxOutputBytes\"",
            "{\"unknown\":1,\"maxOutputBytes\"",
            1
        ))
        .is_err()
    );
}

#[test]
fn validation_order_and_stable_taxonomy_cover_schema_dimensions_limit_and_length() {
    let mut image = rgb_image();
    image.schema_version = 2;
    image.width = 0;
    assert_fatal(&image, &options(1), "M5-TGA-SCHEMA-INVALID");

    let image = rgb_image();
    let mut invalid_options = options(0);
    invalid_options.schema_version = 2;
    assert_fatal(&image, &invalid_options, "M5-TGA-SCHEMA-INVALID");
    assert_fatal(&image, &options(0), "M5-TGA-SCHEMA-INVALID");
    assert_fatal(
        &image,
        &options(TGA_MAX_OUTPUT_BYTES + 1),
        "M5-TGA-SCHEMA-INVALID",
    );

    let mut zero = rgb_image();
    zero.width = 0;
    let error = write_tga_v1(&zero, &TgaWriterOptionsV1::default()).unwrap_err();
    assert_eq!(error.code, "M5-TGA-DIMENSIONS-INVALID");
    assert_eq!(error.path, "image.width");
    assert_eq!(error.severity, "FATAL");
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        r#"{"schemaVersion":1,"code":"M5-TGA-DIMENSIONS-INVALID","severity":"FATAL","path":"image.width","message":"width must be in 1..=65535"}"#
    );
    let mut too_large = rgb_image();
    too_large.height = 65_536;
    assert_fatal(
        &too_large,
        &TgaWriterOptionsV1::default(),
        "M5-TGA-DIMENSIONS-INVALID",
    );

    assert_fatal(&image, &options(55), "M5-TGA-OUTPUT-LIMIT-EXCEEDED");
    let mut short_over_limit = rgb_image();
    short_over_limit.pixels.pop();
    assert_fatal(
        &short_over_limit,
        &options(55),
        "M5-TGA-OUTPUT-LIMIT-EXCEEDED",
    );
    let mut short = rgb_image();
    short.pixels.pop();
    assert_fatal(
        &short,
        &TgaWriterOptionsV1::default(),
        "M5-TGA-PIXEL-LENGTH-INVALID",
    );
    let mut long = rgb_image();
    long.pixels.push(0);
    assert_fatal(
        &long,
        &TgaWriterOptionsV1::default(),
        "M5-TGA-PIXEL-LENGTH-INVALID",
    );
}

#[test]
fn exact_output_limit_and_maximum_dimension_are_inclusive() {
    assert_eq!(TGA_MAX_OUTPUT_BYTES, TGA_MAX_PIXEL_BYTES + 44);
    let small = rgb_image();
    let artifact = write_tga_v1(&small, &options(56)).expect("exact output limit is legal");
    assert_eq!(artifact.report.byte_length, 56);

    let maximum_width = TgaImageV1 {
        schema_version: 1,
        width: 65_535,
        height: 1,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: vec![0x5a; 65_535 * 3],
    };
    let exact_length = 18 + u64::from(maximum_width.width) * 3 + 26;
    let artifact = write_tga_v1(&maximum_width, &options(exact_length))
        .expect("u16::MAX width and exact output limit are legal");
    assert_eq!(artifact.report.width, 65_535);
    assert_eq!(artifact.report.height, 1);
    assert_eq!(artifact.report.pixel_data_length, 65_535 * 3);
    assert_eq!(artifact.report.byte_length, exact_length);
    assert_eq!(artifact.payload.len() as u64, exact_length);
}

#[test]
fn maximum_height_and_exact_output_limit_are_inclusive() {
    let maximum_height = TgaImageV1 {
        schema_version: 1,
        width: 1,
        height: 65_535,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: vec![0xa5; 65_535 * 3],
    };
    let exact_length = 18 + u64::from(maximum_height.height) * 3 + 26;
    let artifact = write_tga_v1(&maximum_height, &options(exact_length))
        .expect("u16::MAX height and exact output limit are legal");
    assert_eq!(artifact.report.width, 1);
    assert_eq!(artifact.report.height, 65_535);
    assert_eq!(artifact.report.pixel_data_length, 65_535 * 3);
    assert_eq!(artifact.report.byte_length, exact_length);
    assert_eq!(artifact.payload.len() as u64, exact_length);
}

#[test]
fn texture_artifact_cleanup_repairs_bright_impulse_and_alpha_hole_without_dark_blur() {
    let mut image = solid_rgba_image(9, 5, [80, 90, 100, 255]);
    set_rgba(&mut image, 2, 2, [230, 230, 230, 255]);
    set_rgba(&mut image, 4, 2, [0, 0, 0, 255]);
    set_rgba(&mut image, 6, 2, [0, 0, 0, 0]);
    let before = image.clone();

    let artifact = cleanup_texture_artifacts_v1(
        &image,
        &TextureArtifactCleanupOptionsV1 {
            schema_version: 1,
            enabled: true,
        },
    )
    .unwrap();

    assert_eq!(image, before, "cleanup must not mutate caller-owned input");
    assert_eq!(rgba_at(&artifact.image, 2, 2), [80, 90, 100, 255]);
    assert_eq!(rgba_at(&artifact.image, 4, 2), [0, 0, 0, 255]);
    assert_eq!(rgba_at(&artifact.image, 6, 2), [80, 90, 100, 255]);
    assert_eq!(artifact.report.algorithm, "EDGE_AWARE_HAMPEL_MEDIAN_V3");
    assert_eq!(artifact.report.pass_count, 2);
    assert_eq!(artifact.report.repaired_color_outlier_count, 1);
    assert_eq!(artifact.report.repaired_transparent_hole_count, 1);
    assert_ne!(
        artifact.report.input_pixel_sha256,
        artifact.report.output_pixel_sha256
    );
}

#[test]
fn texture_artifact_cleanup_repairs_short_runs_from_local_neighbors() {
    let mut image = solid_rgba_image(9, 7, [80, 90, 100, 255]);
    for x in 2..=4 {
        set_rgba(&mut image, x, 2, [230, 230, 230, 255]);
    }

    let artifact = cleanup_texture_artifacts_v1(
        &image,
        &TextureArtifactCleanupOptionsV1 {
            schema_version: 1,
            enabled: true,
        },
    )
    .unwrap();

    for x in 2..=4 {
        assert_eq!(rgba_at(&artifact.image, x, 2), [80, 90, 100, 255]);
    }
    assert_eq!(artifact.report.repaired_color_outlier_count, 3);
}

#[test]
fn texture_artifact_cleanup_preserves_material_edges_and_transparent_regions() {
    let mut image = solid_rgba_image(13, 7, [80, 90, 100, 255]);
    for y in 0..7 {
        for x in 8..13 {
            set_rgba(&mut image, x, y, [190, 190, 190, 255]);
        }
    }
    for (x, y) in [(4, 2), (5, 2), (4, 3), (5, 3)] {
        set_rgba(&mut image, x, y, [0, 0, 0, 0]);
    }

    let artifact = cleanup_texture_artifacts_v1(
        &image,
        &TextureArtifactCleanupOptionsV1 {
            schema_version: 1,
            enabled: true,
        },
    )
    .unwrap();

    assert_eq!(rgba_at(&artifact.image, 7, 3), [80, 90, 100, 255]);
    assert_eq!(rgba_at(&artifact.image, 8, 3), [190, 190, 190, 255]);
    for (x, y) in [(4, 2), (5, 2), (4, 3), (5, 3)] {
        assert_eq!(rgba_at(&artifact.image, x, y), [0, 0, 0, 0]);
    }
    assert_eq!(artifact.report.repaired_color_outlier_count, 0);
    assert_eq!(artifact.report.repaired_transparent_hole_count, 0);
}

#[test]
fn texture_artifact_cleanup_preserves_coherent_highlight_patch() {
    let mut image = solid_rgba_image(11, 11, [80, 90, 100, 255]);
    for y in 4..=6 {
        for x in 4..=6 {
            set_rgba(&mut image, x, y, [190, 190, 190, 255]);
        }
    }

    let artifact = cleanup_texture_artifacts_v1(
        &image,
        &TextureArtifactCleanupOptionsV1 {
            schema_version: 1,
            enabled: true,
        },
    )
    .unwrap();

    assert_eq!(artifact.image, image);
    assert_eq!(artifact.report.repaired_color_outlier_count, 0);
    assert_eq!(artifact.report.repaired_transparent_hole_count, 0);
}

#[test]
fn disabled_texture_artifact_cleanup_is_byte_exact_and_reported() {
    let mut image = solid_rgba_image(3, 3, [80, 90, 100, 255]);
    set_rgba(&mut image, 1, 1, [255, 255, 255, 255]);

    let first =
        cleanup_texture_artifacts_v1(&image, &TextureArtifactCleanupOptionsV1::default()).unwrap();
    let second =
        cleanup_texture_artifacts_v1(&image, &TextureArtifactCleanupOptionsV1::default()).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.image, image);
    assert!(!first.report.enabled);
    assert_eq!(first.report.repaired_color_outlier_count, 0);
    assert_eq!(first.report.repaired_transparent_hole_count, 0);
    assert_eq!(
        first.report.input_pixel_sha256,
        first.report.output_pixel_sha256
    );
}

#[test]
fn exact_stoneback_meshy_base_color_has_repairable_isolated_artifacts() {
    if std::env::var_os("M2A_REQUIRE_TLC_STONEBACK_TEXTURE_CLEANUP").is_none() {
        return;
    }
    let source = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb"),
    )
    .expect("canonical local Stoneback Meshy GLB");
    let image = decode_embedded_image_to_tga_v1(
        &source,
        0,
        &GlbLimits {
            max_input_bytes: 256 * 1024 * 1024,
            ..GlbLimits::default()
        },
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .expect("decode exact base-color image");
    let artifact = cleanup_texture_artifacts_v1(
        &image,
        &TextureArtifactCleanupOptionsV1 {
            schema_version: 1,
            enabled: true,
        },
    )
    .expect("clean exact Meshy base-color image");

    assert_eq!(artifact.report.repaired_color_outlier_count, 4_588);
    assert_eq!(artifact.report.repaired_transparent_hole_count, 0);
    assert_eq!(
        artifact.report.input_pixel_sha256,
        "fd05864b65c21dc98cc52131d0c6bc012ad546646a04940ef4135aaf9566f163"
    );
    assert_eq!(
        artifact.report.output_pixel_sha256,
        "62ebd7a04eddcdb13ae43133c2aa78513cb426ce2ed056c59db11aa74be50937"
    );
    let tga = write_tga_v1(&artifact.image, &TgaWriterOptionsV1::default())
        .expect("write cleaned exact Meshy base color");
    assert_eq!(
        tga.report.output_sha256,
        "ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc"
    );
}
