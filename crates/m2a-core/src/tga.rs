use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const TGA_SCHEMA_VERSION: u32 = 1;
pub const TGA_MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;

const HEADER_LENGTH: u64 = 18;
const FOOTER_LENGTH: u64 = 26;
const FOOTER: &[u8; 26] = b"\0\0\0\0\0\0\0\0TRUEVISION-XFILE.\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TgaPixelFormatV1 {
    #[serde(rename = "RGB8")]
    Rgb8,
    #[serde(rename = "RGBA8")]
    Rgba8,
}

impl TgaPixelFormatV1 {
    const fn channels(self) -> u64 {
        match self {
            Self::Rgb8 => 3,
            Self::Rgba8 => 4,
        }
    }

    const fn pixel_depth(self) -> u8 {
        match self {
            Self::Rgb8 => 24,
            Self::Rgba8 => 32,
        }
    }

    const fn descriptor(self) -> u8 {
        match self {
            Self::Rgb8 => 0,
            Self::Rgba8 => 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TgaImageV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_format: TgaPixelFormatV1,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TgaWriterLimitsV1 {
    pub max_output_bytes: u64,
}

impl Default for TgaWriterLimitsV1 {
    fn default() -> Self {
        Self {
            max_output_bytes: TGA_MAX_OUTPUT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TgaWriterOptionsV1 {
    pub schema_version: u32,
    pub limits: TgaWriterLimitsV1,
}

impl Default for TgaWriterOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: TGA_SCHEMA_VERSION,
            limits: TgaWriterLimitsV1::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TgaWriterReportV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_format: TgaPixelFormatV1,
    pub pixel_depth: u8,
    pub descriptor: u8,
    pub pixel_data_offset: u64,
    pub pixel_data_length: u64,
    pub footer_offset: u64,
    pub byte_length: u64,
    pub input_sha256: String,
    pub output_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TgaArtifactV1 {
    pub payload: Vec<u8>,
    pub report: TgaWriterReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TgaWriteError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl TgaWriteError {
    fn fatal(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: TGA_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for TgaWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TgaWriteError {}

pub fn write_tga_v1(
    image: &TgaImageV1,
    options: &TgaWriterOptionsV1,
) -> Result<TgaArtifactV1, TgaWriteError> {
    validate_schema_and_options(image, options)?;
    validate_dimensions(image)?;

    let pixel_data_length = u64::from(image.width)
        .checked_mul(u64::from(image.height))
        .and_then(|value| value.checked_mul(image.pixel_format.channels()))
        .ok_or_else(|| {
            TgaWriteError::fatal(
                "M5-TGA-OUTPUT-LIMIT-EXCEEDED",
                "image.dimensions",
                "pixel byte length overflows u64",
            )
        })?;
    let footer_offset = HEADER_LENGTH
        .checked_add(pixel_data_length)
        .ok_or_else(|| {
            TgaWriteError::fatal(
                "M5-TGA-OUTPUT-LIMIT-EXCEEDED",
                "output",
                "TGA footer offset overflows u64",
            )
        })?;
    let output_length = footer_offset.checked_add(FOOTER_LENGTH).ok_or_else(|| {
        TgaWriteError::fatal(
            "M5-TGA-OUTPUT-LIMIT-EXCEEDED",
            "output",
            "TGA output length overflows u64",
        )
    })?;

    if output_length > options.limits.max_output_bytes {
        return Err(TgaWriteError::fatal(
            "M5-TGA-OUTPUT-LIMIT-EXCEEDED",
            "options.limits.maxOutputBytes",
            format!(
                "TGA output requires {output_length} bytes but the configured limit is {}",
                options.limits.max_output_bytes
            ),
        ));
    }

    if u64::try_from(image.pixels.len()).ok() != Some(pixel_data_length) {
        return Err(TgaWriteError::fatal(
            "M5-TGA-PIXEL-LENGTH-INVALID",
            "image.pixels",
            format!(
                "pixel buffer has {} bytes but {pixel_data_length} are required",
                image.pixels.len()
            ),
        ));
    }

    let output_capacity = usize::try_from(output_length).map_err(|_| {
        TgaWriteError::fatal(
            "M5-TGA-ALLOCATION-FAILED",
            "output",
            "TGA output length does not fit this platform",
        )
    })?;
    let row_length = usize::try_from(u64::from(image.width) * image.pixel_format.channels())
        .map_err(|_| {
            TgaWriteError::fatal(
                "M5-TGA-ALLOCATION-FAILED",
                "output",
                "TGA row length does not fit this platform",
            )
        })?;
    let height = usize::try_from(image.height).map_err(|_| {
        TgaWriteError::fatal(
            "M5-TGA-ALLOCATION-FAILED",
            "output",
            "TGA height does not fit this platform",
        )
    })?;
    let channels = usize::try_from(image.pixel_format.channels()).expect("channels fit usize");

    let mut payload = Vec::new();
    payload.try_reserve_exact(output_capacity).map_err(|_| {
        TgaWriteError::fatal(
            "M5-TGA-ALLOCATION-FAILED",
            "output",
            "could not reserve the TGA output buffer",
        )
    })?;
    emit_header(&mut payload, image);
    for source_row in (0..height).rev() {
        let row_start = source_row * row_length;
        for pixel in image.pixels[row_start..row_start + row_length].chunks_exact(channels) {
            payload.push(pixel[2]);
            payload.push(pixel[1]);
            payload.push(pixel[0]);
            if channels == 4 {
                payload.push(pixel[3]);
            }
        }
    }
    payload.extend_from_slice(FOOTER);

    verify_readback(&payload, image)?;

    Ok(TgaArtifactV1 {
        report: TgaWriterReportV1 {
            schema_version: TGA_SCHEMA_VERSION,
            width: image.width,
            height: image.height,
            pixel_format: image.pixel_format,
            pixel_depth: image.pixel_format.pixel_depth(),
            descriptor: image.pixel_format.descriptor(),
            pixel_data_offset: HEADER_LENGTH,
            pixel_data_length,
            footer_offset,
            byte_length: output_length,
            input_sha256: sha256_hex(&image.pixels),
            output_sha256: sha256_hex(&payload),
        },
        payload,
    })
}

fn verify_readback(payload: &[u8], image: &TgaImageV1) -> Result<(), TgaWriteError> {
    let readback = readback_tga_v1(payload)
        .map_err(|message| TgaWriteError::fatal("M5-TGA-READBACK-FAILED", "output", message))?;
    if readback.width != image.width
        || readback.height != image.height
        || readback.pixel_format != image.pixel_format
        || readback.pixels != image.pixels
    {
        return Err(TgaWriteError::fatal(
            "M5-TGA-SEMANTIC-DIFF",
            "output",
            "TGA readback differs from the requested image",
        ));
    }

    Ok(())
}

fn validate_schema_and_options(
    image: &TgaImageV1,
    options: &TgaWriterOptionsV1,
) -> Result<(), TgaWriteError> {
    if image.schema_version != TGA_SCHEMA_VERSION {
        return Err(TgaWriteError::fatal(
            "M5-TGA-SCHEMA-INVALID",
            "image.schemaVersion",
            "image schemaVersion must be 1",
        ));
    }
    if options.schema_version != TGA_SCHEMA_VERSION {
        return Err(TgaWriteError::fatal(
            "M5-TGA-SCHEMA-INVALID",
            "options.schemaVersion",
            "writer options schemaVersion must be 1",
        ));
    }
    if options.limits.max_output_bytes == 0
        || options.limits.max_output_bytes > TGA_MAX_OUTPUT_BYTES
    {
        return Err(TgaWriteError::fatal(
            "M5-TGA-SCHEMA-INVALID",
            "options.limits.maxOutputBytes",
            format!("maxOutputBytes must be in 1..={TGA_MAX_OUTPUT_BYTES}"),
        ));
    }
    Ok(())
}

fn validate_dimensions(image: &TgaImageV1) -> Result<(), TgaWriteError> {
    if image.width == 0 || image.width > u16::MAX.into() {
        return Err(TgaWriteError::fatal(
            "M5-TGA-DIMENSIONS-INVALID",
            "image.width",
            "width must be in 1..=65535",
        ));
    }
    if image.height == 0 || image.height > u16::MAX.into() {
        return Err(TgaWriteError::fatal(
            "M5-TGA-DIMENSIONS-INVALID",
            "image.height",
            "height must be in 1..=65535",
        ));
    }
    Ok(())
}

fn emit_header(payload: &mut Vec<u8>, image: &TgaImageV1) {
    payload.extend_from_slice(&[0, 0, 2]);
    payload.extend_from_slice(&[0; 9]);
    payload.extend_from_slice(&(image.width as u16).to_le_bytes());
    payload.extend_from_slice(&(image.height as u16).to_le_bytes());
    payload.push(image.pixel_format.pixel_depth());
    payload.push(image.pixel_format.descriptor());
}

#[derive(Debug, Eq, PartialEq)]
struct TgaReadback {
    width: u32,
    height: u32,
    pixel_format: TgaPixelFormatV1,
    pixels: Vec<u8>,
}

fn readback_tga_v1(payload: &[u8]) -> Result<TgaReadback, String> {
    if payload.len() < usize::try_from(HEADER_LENGTH + FOOTER_LENGTH).unwrap() {
        return Err("TGA payload is shorter than its header and footer".to_owned());
    }
    if payload[0] != 0 || payload[1] != 0 || payload[2] != 2 || payload[3..12] != [0; 9] {
        return Err("TGA header fields do not match the locked type-2 profile".to_owned());
    }
    let width = u32::from(u16::from_le_bytes([payload[12], payload[13]]));
    let height = u32::from(u16::from_le_bytes([payload[14], payload[15]]));
    if width == 0 || height == 0 {
        return Err("TGA readback dimensions are zero".to_owned());
    }
    let pixel_format = match (payload[16], payload[17]) {
        (24, 0) => TgaPixelFormatV1::Rgb8,
        (32, 8) => TgaPixelFormatV1::Rgba8,
        _ => return Err("TGA depth and descriptor do not match RGB8/RGBA8".to_owned()),
    };
    let pixel_length = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|value| value.checked_mul(pixel_format.channels()))
        .ok_or_else(|| "TGA readback pixel length overflows u64".to_owned())?;
    let expected_length = HEADER_LENGTH
        .checked_add(pixel_length)
        .and_then(|value| value.checked_add(FOOTER_LENGTH))
        .ok_or_else(|| "TGA readback output length overflows u64".to_owned())?;
    if u64::try_from(payload.len()).ok() != Some(expected_length) {
        return Err("TGA payload length does not match its dimensions".to_owned());
    }
    let footer_offset = usize::try_from(HEADER_LENGTH + pixel_length)
        .map_err(|_| "TGA footer offset does not fit this platform".to_owned())?;
    if &payload[footer_offset..] != FOOTER {
        return Err("TGA footer or exact EOF is invalid".to_owned());
    }

    let pixel_capacity = usize::try_from(pixel_length)
        .map_err(|_| "TGA pixel length does not fit this platform".to_owned())?;
    let channels = usize::try_from(pixel_format.channels()).expect("channels fit usize");
    let width_usize = usize::try_from(width).map_err(|_| "width does not fit usize".to_owned())?;
    let height_usize =
        usize::try_from(height).map_err(|_| "height does not fit usize".to_owned())?;
    let row_length = width_usize
        .checked_mul(channels)
        .ok_or_else(|| "TGA row length overflows usize".to_owned())?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(pixel_capacity)
        .map_err(|_| "could not allocate TGA readback pixels".to_owned())?;
    pixels.resize(pixel_capacity, 0);
    for target_row in 0..height_usize {
        let source_row = height_usize - 1 - target_row;
        let source_start = usize::try_from(HEADER_LENGTH).unwrap() + source_row * row_length;
        let target_start = target_row * row_length;
        for x in 0..width_usize {
            let source = source_start + x * channels;
            let target = target_start + x * channels;
            pixels[target] = payload[source + 2];
            pixels[target + 1] = payload[source + 1];
            pixels[target + 2] = payload[source];
            if channels == 4 {
                pixels[target + 3] = payload[source + 3];
            }
        }
    }
    Ok(TgaReadback {
        width,
        height,
        pixel_format,
        pixels,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_readback_rejects_every_truncation_and_mutated_locked_field_without_panicking() {
        let image = TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgba8,
            pixels: vec![1, 2, 3, 4],
        };
        let payload = write_tga_v1(&image, &TgaWriterOptionsV1::default())
            .unwrap()
            .payload;
        for length in 0..payload.len() {
            let result = std::panic::catch_unwind(|| readback_tga_v1(&payload[..length]));
            assert!(
                result.is_ok(),
                "readback panicked at prefix length {length}"
            );
            assert!(result.unwrap().is_err());
        }
        for offset in [0, 1, 2, 3, 12, 14, 16, 17, payload.len() - 1] {
            let mut mutated = payload.clone();
            mutated[offset] ^= 0x01;
            let result = std::panic::catch_unwind(|| readback_tga_v1(&mutated));
            assert!(result.is_ok(), "readback panicked for mutation at {offset}");
            assert!(result.unwrap().is_err());
        }

        let mut pixel_mutation = payload.clone();
        pixel_mutation[18] ^= 0x01;
        let error = verify_readback(&pixel_mutation, &image).unwrap_err();
        assert_eq!(error.code, "M5-TGA-SEMANTIC-DIFF");

        let mut header_mutation = payload.clone();
        header_mutation[2] = 10;
        let error = verify_readback(&header_mutation, &image).unwrap_err();
        assert_eq!(error.code, "M5-TGA-READBACK-FAILED");

        let mut trailing_byte = payload.clone();
        trailing_byte.push(0);
        let result = std::panic::catch_unwind(|| readback_tga_v1(&trailing_byte));
        assert!(result.is_ok(), "readback panicked for a trailing byte");
        assert!(result.unwrap().is_err());
    }
}
