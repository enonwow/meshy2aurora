use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::tga::{
    TGA_MAX_OUTPUT_BYTES, TgaImageV1, TgaPixelFormatV1, TgaWriterLimitsV1, TgaWriterOptionsV1,
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
