use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const TGA_SCHEMA_VERSION: u32 = 1;
pub const TEXTURE_ARTIFACT_CLEANUP_ALGORITHM_V3: &str = "EDGE_AWARE_HAMPEL_MEDIAN_V3";
const TEXTURE_ARTIFACT_CLEANUP_PASS_COUNT_V3: u32 = 2;
const TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3: usize = 2;
const TEXTURE_ARTIFACT_CLEANUP_MIN_OPAQUE_NEIGHBORS_V3: usize = 20;
const TEXTURE_ARTIFACT_CLEANUP_MIN_LUMA_DIFFERENCE_V3: u8 = 24;
const TEXTURE_ARTIFACT_CLEANUP_MAD_MULTIPLIER_V3: u8 = 4;
const TEXTURE_ARTIFACT_CLEANUP_CENTER_SUPPORT_DISTANCE_V3: u8 = 10;
const TEXTURE_ARTIFACT_CLEANUP_MAX_CENTER_SUPPORT_V3: usize = 2;
const TEXTURE_ARTIFACT_CLEANUP_MEDIAN_COHERENCE_DISTANCE_V3: u8 = 12;
const HEADER_LENGTH: u64 = 18;
const FOOTER_LENGTH: u64 = 26;
/// Product pixel-payload budget. The container cap additionally includes the
/// fixed 18-byte header and 26-byte footer, so an exact 4096x4096 RGBA image
/// is representable without weakening the pixel budget.
pub const TGA_MAX_PIXEL_BYTES: u64 = 64 * 1024 * 1024;
pub const TGA_MAX_OUTPUT_BYTES: u64 = TGA_MAX_PIXEL_BYTES + HEADER_LENGTH + FOOTER_LENGTH;
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
pub struct TextureArtifactCleanupOptionsV1 {
    pub schema_version: u32,
    pub enabled: bool,
}

impl Default for TextureArtifactCleanupOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: TGA_SCHEMA_VERSION,
            enabled: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureArtifactCleanupReportV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub enabled: bool,
    pub pass_count: u32,
    pub inspected_pixel_count: u64,
    pub repaired_color_outlier_count: u64,
    pub repaired_transparent_hole_count: u64,
    pub input_pixel_sha256: String,
    pub output_pixel_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureArtifactCleanupArtifactV1 {
    pub image: TgaImageV1,
    pub report: TextureArtifactCleanupReportV1,
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

/// Applies the fixed edge-aware local-neighbor base-color cleanup used by Studio.
///
/// Replacement channels are always the median of opaque pixels in a local 5x5
/// neighborhood; no random color is generated. A Hampel-style median absolute
/// deviation gate rejects ordinary texture variation. A same-color support
/// gate preserves coherent highlights and material borders, while still
/// admitting isolated points and runs up to three samples. Two fixed passes
/// repair remaining bright impulse artifacts without applying a global blur.
/// One-pixel alpha holes retain their separate conservative 3x3 repair. The
/// caller-owned image is never mutated.
pub fn cleanup_texture_artifacts_v1(
    image: &TgaImageV1,
    options: &TextureArtifactCleanupOptionsV1,
) -> Result<TextureArtifactCleanupArtifactV1, TgaWriteError> {
    if options.schema_version != TGA_SCHEMA_VERSION {
        return Err(TgaWriteError::fatal(
            "M5-TEXTURE-CLEANUP-SCHEMA-INVALID",
            "options.schemaVersion",
            format!(
                "expected texture cleanup schema {}, got {}",
                TGA_SCHEMA_VERSION, options.schema_version
            ),
        ));
    }
    validate_schema_and_options(image, &TgaWriterOptionsV1::default())?;
    validate_dimensions(image)?;

    let input_pixel_sha256 = sha256_hex(&image.pixels);
    let mut output = image.clone();
    let width = usize::try_from(image.width).map_err(|_| {
        TgaWriteError::fatal(
            "M5-TGA-DIMENSIONS-INVALID",
            "image.width",
            "width does not fit this platform",
        )
    })?;
    let height = usize::try_from(image.height).map_err(|_| {
        TgaWriteError::fatal(
            "M5-TGA-DIMENSIONS-INVALID",
            "image.height",
            "height does not fit this platform",
        )
    })?;
    let channels = usize::try_from(image.pixel_format.channels())
        .expect("the fixed RGB channel count fits usize");
    let minimum_dimension = TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3 * 2 + 1;
    let inspected_pixel_count =
        if options.enabled && width >= minimum_dimension && height >= minimum_dimension {
            u64::try_from(
                (width - TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3 * 2)
                    * (height - TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3 * 2),
            )
            .unwrap_or(u64::MAX)
        } else {
            0
        };
    let mut repaired_color_outlier_count = 0u64;
    let mut repaired_transparent_hole_count = 0u64;

    let pass_count = if options.enabled && width >= minimum_dimension && height >= minimum_dimension
    {
        TEXTURE_ARTIFACT_CLEANUP_PASS_COUNT_V3
    } else {
        0
    };

    if pass_count > 0 {
        for _pass in 0..pass_count {
            let input = output.pixels.clone();
            for y in TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3..height - TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3
            {
                for x in
                    TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3..width - TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3
                {
                    let center_offset = (y * width + x) * channels;
                    let center_alpha = if channels == 4 {
                        input[center_offset + 3]
                    } else {
                        u8::MAX
                    };
                    let mut immediate_opaque_offsets = [0usize; 8];
                    let mut immediate_opaque_count = 0usize;
                    let mut opaque_offsets = [0usize; 24];
                    let mut opaque_count = 0usize;
                    for neighbor_y in y - TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3
                        ..=y + TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3
                    {
                        for neighbor_x in x - TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3
                            ..=x + TEXTURE_ARTIFACT_CLEANUP_RADIUS_V3
                        {
                            if neighbor_x == x && neighbor_y == y {
                                continue;
                            }
                            let offset = (neighbor_y * width + neighbor_x) * channels;
                            let alpha = if channels == 4 {
                                input[offset + 3]
                            } else {
                                u8::MAX
                            };
                            if alpha < 224 {
                                continue;
                            }
                            opaque_offsets[opaque_count] = offset;
                            opaque_count += 1;
                            if neighbor_x.abs_diff(x) <= 1 && neighbor_y.abs_diff(y) <= 1 {
                                immediate_opaque_offsets[immediate_opaque_count] = offset;
                                immediate_opaque_count += 1;
                            }
                        }
                    }

                    if channels == 4 && center_alpha <= 32 && immediate_opaque_count >= 7 {
                        let mut changed = false;
                        for channel in 0..4 {
                            let replacement = median_channel(
                                &input,
                                &immediate_opaque_offsets,
                                immediate_opaque_count,
                                channel,
                            );
                            changed |= output.pixels[center_offset + channel] != replacement;
                            output.pixels[center_offset + channel] = replacement;
                        }
                        if changed {
                            repaired_transparent_hole_count += 1;
                        }
                        continue;
                    }
                    if center_alpha < 224
                        || opaque_count < TEXTURE_ARTIFACT_CLEANUP_MIN_OPAQUE_NEIGHBORS_V3
                    {
                        continue;
                    }

                    let mut neighbor_luma = [0u8; 24];
                    for index in 0..opaque_count {
                        neighbor_luma[index] = pixel_luma(&input, opaque_offsets[index]);
                    }
                    neighbor_luma[..opaque_count].sort_unstable();
                    let median_luma = median_sorted(&neighbor_luma, opaque_count);
                    let center_luma = pixel_luma(&input, center_offset);
                    let center_difference = center_luma.saturating_sub(median_luma);
                    if center_difference < TEXTURE_ARTIFACT_CLEANUP_MIN_LUMA_DIFFERENCE_V3 {
                        continue;
                    }

                    let mut median_deviations = [0u8; 24];
                    let mut center_support = 0usize;
                    let mut median_coherence = 0usize;
                    for index in 0..opaque_count {
                        let luma = pixel_luma(&input, opaque_offsets[index]);
                        median_deviations[index] = luma.abs_diff(median_luma);
                        if luma.abs_diff(center_luma)
                            <= TEXTURE_ARTIFACT_CLEANUP_CENTER_SUPPORT_DISTANCE_V3
                        {
                            center_support += 1;
                        }
                        if luma.abs_diff(median_luma)
                            <= TEXTURE_ARTIFACT_CLEANUP_MEDIAN_COHERENCE_DISTANCE_V3
                        {
                            median_coherence += 1;
                        }
                    }
                    median_deviations[..opaque_count].sort_unstable();
                    let median_absolute_deviation = median_sorted(&median_deviations, opaque_count);
                    let adaptive_difference = median_absolute_deviation
                        .max(1)
                        .saturating_mul(TEXTURE_ARTIFACT_CLEANUP_MAD_MULTIPLIER_V3)
                        .max(TEXTURE_ARTIFACT_CLEANUP_MIN_LUMA_DIFFERENCE_V3);
                    if center_difference < adaptive_difference
                        || center_support > TEXTURE_ARTIFACT_CLEANUP_MAX_CENTER_SUPPORT_V3
                        || median_coherence < opaque_count.div_ceil(2)
                    {
                        continue;
                    }

                    let mut changed = false;
                    for channel in 0..3 {
                        let replacement =
                            median_channel(&input, &opaque_offsets, opaque_count, channel);
                        changed |= output.pixels[center_offset + channel] != replacement;
                        output.pixels[center_offset + channel] = replacement;
                    }
                    if changed {
                        repaired_color_outlier_count += 1;
                    }
                }
            }
        }
    } else {
        debug_assert_eq!(inspected_pixel_count, 0);
    }

    let output_pixel_sha256 = sha256_hex(&output.pixels);
    Ok(TextureArtifactCleanupArtifactV1 {
        image: output,
        report: TextureArtifactCleanupReportV1 {
            schema_version: TGA_SCHEMA_VERSION,
            algorithm: TEXTURE_ARTIFACT_CLEANUP_ALGORITHM_V3.to_owned(),
            enabled: options.enabled,
            pass_count,
            inspected_pixel_count,
            repaired_color_outlier_count,
            repaired_transparent_hole_count,
            input_pixel_sha256,
            output_pixel_sha256,
        },
    })
}

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

/// Reads a TGA written by the locked V1 profile back into canonical top-left RGB(A) pixels.
pub fn read_tga_image_v1(payload: &[u8]) -> Result<TgaImageV1, TgaWriteError> {
    let readback = readback_tga_v1(payload)
        .map_err(|message| TgaWriteError::fatal("M5-TGA-READBACK-FAILED", "input", message))?;
    Ok(TgaImageV1 {
        schema_version: TGA_SCHEMA_VERSION,
        width: readback.width,
        height: readback.height,
        pixel_format: readback.pixel_format,
        pixels: readback.pixels,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn median_channel<const N: usize>(
    pixels: &[u8],
    offsets: &[usize; N],
    count: usize,
    channel: usize,
) -> u8 {
    let mut values = [0u8; N];
    for index in 0..count {
        values[index] = pixels[offsets[index] + channel];
    }
    values[..count].sort_unstable();
    median_sorted(&values, count)
}

fn median_sorted(values: &[u8], count: usize) -> u8 {
    if count.is_multiple_of(2) {
        let left = u16::from(values[count / 2 - 1]);
        let right = u16::from(values[count / 2]);
        u8::try_from((left + right) / 2).expect("the average of two u8 values fits u8")
    } else {
        values[count / 2]
    }
}

fn pixel_luma(pixels: &[u8], offset: usize) -> u8 {
    let red = u16::from(pixels[offset]);
    let green = u16::from(pixels[offset + 1]);
    let blue = u16::from(pixels[offset + 2]);
    u8::try_from((54 * red + 183 * green + 19 * blue) >> 8).expect("fixed-point RGB luma fits u8")
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
