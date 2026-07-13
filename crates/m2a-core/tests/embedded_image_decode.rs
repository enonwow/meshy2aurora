use std::panic::{AssertUnwindSafe, catch_unwind};

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use m2a_core::glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1};
use m2a_core::tga::{TGA_SCHEMA_VERSION, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1};
use serde_json::json;

#[test]
fn embedded_png_rgba_decodes_deterministically_to_tga_image() {
    let source_pixels = [255, 0, 0, 255, 0, 128, 255, 64];
    let png = encode_png(&source_pixels, 2, 1, ExtendedColorType::Rgba8);
    let glb = image_glb(&png, "image/png");

    let first = decode(&glb, 0, EmbeddedImageDecodeLimitsV1::default()).unwrap();
    let second = decode(&glb, 0, EmbeddedImageDecodeLimitsV1::default()).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.schema_version, TGA_SCHEMA_VERSION);
    assert_eq!((first.width, first.height), (2, 1));
    assert_eq!(first.pixel_format, TgaPixelFormatV1::Rgba8);
    assert_eq!(first.pixels, source_pixels);
    write_tga_v1(&first, &TgaWriterOptionsV1::default())
        .expect("decoded image must be accepted by the locked TGA writer");
}

#[test]
fn embedded_jpeg_decodes_deterministically_to_rgb8() {
    let source_pixels = [24, 80, 160, 24, 80, 160];
    let jpeg = encode_jpeg(&source_pixels, 2, 1);
    let glb = image_glb(&jpeg, "image/jpeg");

    let first = decode(&glb, 0, EmbeddedImageDecodeLimitsV1::default()).unwrap();
    let second = decode(&glb, 0, EmbeddedImageDecodeLimitsV1::default()).unwrap();

    assert_eq!(first, second);
    assert_eq!((first.width, first.height), (2, 1));
    assert_eq!(first.pixel_format, TgaPixelFormatV1::Rgb8);
    assert_eq!(first.pixels.len(), 6);
    for (actual, expected) in first.pixels.iter().zip(source_pixels) {
        assert!(
            actual.abs_diff(expected) <= 4,
            "{actual} differs from {expected}"
        );
    }
}

#[test]
fn sixteen_bit_png_separates_source_allocation_from_result_byte_limit() {
    let rgb16 = native_u16_bytes(&[0, 0x8080, u16::MAX]);
    let rgba16 = native_u16_bytes(&[u16::MAX, 0, 0x8080, 0x4040]);
    let cases = [
        (
            encode_png(&rgb16, 1, 1, ExtendedColorType::Rgb16),
            TgaPixelFormatV1::Rgb8,
            vec![0, 128, 255],
        ),
        (
            encode_png(&rgba16, 1, 1, ExtendedColorType::Rgba16),
            TgaPixelFormatV1::Rgba8,
            vec![255, 0, 128, 64],
        ),
    ];

    for (png, expected_format, expected_pixels) in cases {
        let exact_output_bytes = expected_pixels.len() as u64;
        let limits = EmbeddedImageDecodeLimitsV1 {
            max_width: 1,
            max_height: 1,
            max_pixels: 1,
            max_decoded_bytes: exact_output_bytes,
        };
        let glb = image_glb(&png, "image/png");
        let decoded = decode(&glb, 0, limits).unwrap();
        assert_eq!(decoded.pixel_format, expected_format);
        assert_eq!(decoded.pixels, expected_pixels);

        let error = decode(
            &glb,
            0,
            EmbeddedImageDecodeLimitsV1 {
                max_decoded_bytes: exact_output_bytes - 1,
                ..limits
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "M2A-GLB-IMAGE-DECODE-LIMIT-EXCEEDED");
    }
}

#[test]
fn declared_mime_must_match_encoded_image_format() {
    let png = encode_png(&[1, 2, 3], 1, 1, ExtendedColorType::Rgb8);
    let error = decode(
        &image_glb(&png, "image/jpeg"),
        0,
        EmbeddedImageDecodeLimitsV1::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M2A-GLB-IMAGE-MIME-MISMATCH");
    assert_eq!(error.json_path.as_deref(), Some("images[0].mimeType"));
}

#[test]
fn invalid_and_truncated_payloads_fail_without_panicking() {
    let png = encode_png(&[1, 2, 3, 4], 1, 1, ExtendedColorType::Rgba8);
    let cases = [
        image_glb(b"not an image", "image/png"),
        image_glb(&png[..png.len() / 2], "image/png"),
    ];
    for input in cases {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            decode(&input, 0, EmbeddedImageDecodeLimitsV1::default())
        }));
        let error = outcome
            .expect("invalid embedded image must not panic")
            .unwrap_err();
        assert_eq!(error.code, "M2A-GLB-IMAGE-DECODE-INVALID");
        assert_eq!(error.json_path.as_deref(), Some("images[0].bufferView"));
    }
}

#[test]
fn image_index_and_decoded_limits_are_strict_at_boundaries() {
    let pixels = [1, 2, 3, 4, 5, 6, 7, 8];
    let png = encode_png(&pixels, 2, 1, ExtendedColorType::Rgba8);
    let glb = image_glb(&png, "image/png");
    let exact = EmbeddedImageDecodeLimitsV1 {
        max_width: 2,
        max_height: 1,
        max_pixels: 2,
        max_decoded_bytes: 8,
    };
    assert_eq!(decode(&glb, 0, exact).unwrap().pixels, pixels);

    for limits in [
        EmbeddedImageDecodeLimitsV1 {
            max_width: 1,
            ..exact
        },
        EmbeddedImageDecodeLimitsV1 {
            max_height: 0,
            ..exact
        },
        EmbeddedImageDecodeLimitsV1 {
            max_pixels: 1,
            ..exact
        },
        EmbeddedImageDecodeLimitsV1 {
            max_decoded_bytes: 7,
            ..exact
        },
    ] {
        let error = decode(&glb, 0, limits).unwrap_err();
        assert_eq!(error.code, "M2A-GLB-IMAGE-DECODE-LIMIT-EXCEEDED");
    }

    let error = decode(&glb, 1, exact).unwrap_err();
    assert_eq!(error.code, "M2A-GLB-IMAGE-INDEX-INVALID");
    assert_eq!(error.json_path.as_deref(), Some("images[1]"));
}

fn decode(
    input: &[u8],
    image_index: usize,
    decode_limits: EmbeddedImageDecodeLimitsV1,
) -> Result<m2a_core::tga::TgaImageV1, m2a_core::glb::GlbFatalError> {
    decode_embedded_image_to_tga_v1(input, image_index, &GlbLimits::default(), &decode_limits)
}

fn encode_png(pixels: &[u8], width: u32, height: u32, color: ExtendedColorType) -> Vec<u8> {
    let mut output = Vec::new();
    PngEncoder::new(&mut output)
        .write_image(pixels, width, height, color)
        .unwrap();
    output
}

fn encode_jpeg(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut output = Vec::new();
    JpegEncoder::new_with_quality(&mut output, 100)
        .encode(pixels, width, height, ExtendedColorType::Rgb8)
        .unwrap();
    output
}

fn native_u16_bytes(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

fn image_glb(image: &[u8], mime_type: &str) -> Vec<u8> {
    let mut bin = image.to_vec();
    let declared_bin_length = bin.len();
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let root = json!({
        "asset": {"version": "2.0", "generator": "m2a-image-decode-test"},
        "buffers": [{"byteLength": declared_bin_length}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": image.len()}],
        "images": [{"bufferView": 0, "mimeType": mime_type}]
    });
    let mut json_bytes = serde_json::to_vec(&root).unwrap();
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }

    let total_length = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut glb = Vec::with_capacity(total_length);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4e4f534a_u32.to_le_bytes());
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004e4942_u32.to_le_bytes());
    glb.extend_from_slice(&bin);
    glb
}
