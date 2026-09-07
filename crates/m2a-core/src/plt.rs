//! Deterministic Aurora PLT V1 reader and writer.
//!
//! Canonical pixels are exposed in top-left row-major order. Aurora stores the
//! same `(colorIndex, layerIndex)` pairs bottom row first, so serialization and
//! readback perform the vertical flip explicitly.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PLT_SCHEMA_VERSION: u32 = 1;
pub const PLT_HEADER_LENGTH: u64 = 24;
pub const PLT_MAX_LAYERS: u32 = 10;
pub const PLT_MAX_PIXEL_BYTES: u64 = 64 * 1024 * 1024;
pub const PLT_MAX_OUTPUT_BYTES: u64 = PLT_HEADER_LENGTH + PLT_MAX_PIXEL_BYTES;
pub const PLT_TRANSPARENT_PIXEL_V1: PltPixelV1 = PltPixelV1 {
    color_index: 255,
    layer_index: 0,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PltPixelV1 {
    pub color_index: u8,
    pub layer_index: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PltImageV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub num_layers: u32,
    pub pixels: Vec<PltPixelV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PltWriterLimitsV1 {
    pub max_output_bytes: u64,
}

impl Default for PltWriterLimitsV1 {
    fn default() -> Self {
        Self {
            max_output_bytes: PLT_MAX_OUTPUT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PltWriterOptionsV1 {
    pub schema_version: u32,
    pub limits: PltWriterLimitsV1,
}

impl Default for PltWriterOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: PLT_SCHEMA_VERSION,
            limits: PltWriterLimitsV1::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PltWriterReportV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub num_layers: u32,
    pub pixel_data_offset: u64,
    pub pixel_data_length: u64,
    pub byte_length: u64,
    pub used_layers: Vec<u8>,
    pub input_pixel_sha256: String,
    pub output_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PltArtifactV1 {
    pub payload: Vec<u8>,
    pub report: PltWriterReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PltErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl PltErrorV1 {
    fn fatal(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: PLT_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for PltErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for PltErrorV1 {}

pub fn plt_image_pixels_sha256_v1(image: &PltImageV1) -> String {
    let mut hasher = Sha256::new();
    for pixel in &image.pixels {
        hasher.update([pixel.color_index, pixel.layer_index]);
    }
    format!("{:x}", hasher.finalize())
}

pub fn write_plt_v1(
    image: &PltImageV1,
    options: &PltWriterOptionsV1,
) -> Result<PltArtifactV1, PltErrorV1> {
    validate_image(image, options)?;
    let pixel_data_length = u64::from(image.width)
        .checked_mul(u64::from(image.height))
        .and_then(|value| value.checked_mul(2))
        .ok_or_else(|| {
            PltErrorV1::fatal(
                "M2A-PLT-DIMENSIONS-INVALID",
                "image",
                "pixel byte length overflows u64",
            )
        })?;
    let byte_length = PLT_HEADER_LENGTH
        .checked_add(pixel_data_length)
        .ok_or_else(|| {
            PltErrorV1::fatal(
                "M2A-PLT-DIMENSIONS-INVALID",
                "image",
                "output byte length overflows u64",
            )
        })?;
    if byte_length > options.limits.max_output_bytes {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-OUTPUT-LIMIT-EXCEEDED",
            "options.limits.maxOutputBytes",
            format!(
                "required {byte_length} bytes, limit is {}",
                options.limits.max_output_bytes
            ),
        ));
    }

    let capacity = usize::try_from(byte_length).map_err(|_| {
        PltErrorV1::fatal(
            "M2A-PLT-OUTPUT-LIMIT-EXCEEDED",
            "image",
            "output length does not fit this platform",
        )
    })?;
    let width = usize::try_from(image.width).expect("validated PLT width fits usize");
    let height = usize::try_from(image.height).expect("validated PLT height fits usize");
    let mut payload = Vec::with_capacity(capacity);
    payload.extend_from_slice(b"PLT V1  ");
    payload.extend_from_slice(&image.num_layers.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&image.width.to_le_bytes());
    payload.extend_from_slice(&image.height.to_le_bytes());
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = image.pixels[y * width + x];
            payload.extend_from_slice(&[pixel.color_index, pixel.layer_index]);
        }
    }
    debug_assert_eq!(payload.len(), capacity);

    let used_layers = image
        .pixels
        .iter()
        .map(|pixel| pixel.layer_index)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    Ok(PltArtifactV1 {
        report: PltWriterReportV1 {
            schema_version: PLT_SCHEMA_VERSION,
            width: image.width,
            height: image.height,
            num_layers: image.num_layers,
            pixel_data_offset: PLT_HEADER_LENGTH,
            pixel_data_length,
            byte_length,
            used_layers,
            input_pixel_sha256: plt_image_pixels_sha256_v1(image),
            output_sha256: format!("{:x}", Sha256::digest(&payload)),
        },
        payload,
    })
}

pub fn read_plt_image_v1(bytes: &[u8]) -> Result<PltImageV1, PltErrorV1> {
    if bytes.len() < PLT_HEADER_LENGTH as usize || &bytes[..8] != b"PLT V1  " {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-HEADER-INVALID",
            "payload",
            "expected exact PLT V1 header",
        ));
    }
    let num_layers = read_u32(bytes, 8)?;
    if read_u32(bytes, 12)? != 0 {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-HEADER-INVALID",
            "payload.reserved",
            "reserved header field must be zero",
        ));
    }
    let width = read_u32(bytes, 16)?;
    let height = read_u32(bytes, 20)?;
    let expected_length = PLT_HEADER_LENGTH
        .checked_add(
            u64::from(width)
                .checked_mul(u64::from(height))
                .and_then(|value| value.checked_mul(2))
                .ok_or_else(|| {
                    PltErrorV1::fatal(
                        "M2A-PLT-DIMENSIONS-INVALID",
                        "payload",
                        "pixel byte length overflows u64",
                    )
                })?,
        )
        .ok_or_else(|| {
            PltErrorV1::fatal(
                "M2A-PLT-DIMENSIONS-INVALID",
                "payload",
                "output byte length overflows u64",
            )
        })?;
    if bytes.len() as u64 != expected_length {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-LENGTH-INVALID",
            "payload",
            format!("expected {expected_length} bytes, got {}", bytes.len()),
        ));
    }
    let width_usize = usize::try_from(width).map_err(|_| {
        PltErrorV1::fatal(
            "M2A-PLT-DIMENSIONS-INVALID",
            "payload.width",
            "width does not fit this platform",
        )
    })?;
    let height_usize = usize::try_from(height).map_err(|_| {
        PltErrorV1::fatal(
            "M2A-PLT-DIMENSIONS-INVALID",
            "payload.height",
            "height does not fit this platform",
        )
    })?;
    let mut pixels = vec![
        PLT_TRANSPARENT_PIXEL_V1;
        width_usize
            .checked_mul(height_usize)
            .ok_or_else(|| PltErrorV1::fatal(
                "M2A-PLT-DIMENSIONS-INVALID",
                "payload",
                "pixel count overflows usize"
            ))?
    ];
    let mut offset = PLT_HEADER_LENGTH as usize;
    for file_y in 0..height_usize {
        let y = height_usize - 1 - file_y;
        for x in 0..width_usize {
            pixels[y * width_usize + x] = PltPixelV1 {
                color_index: bytes[offset],
                layer_index: bytes[offset + 1],
            };
            offset += 2;
        }
    }
    let image = PltImageV1 {
        schema_version: PLT_SCHEMA_VERSION,
        width,
        height,
        num_layers,
        pixels,
    };
    validate_image(&image, &PltWriterOptionsV1::default())?;
    Ok(image)
}

fn validate_image(image: &PltImageV1, options: &PltWriterOptionsV1) -> Result<(), PltErrorV1> {
    if image.schema_version != PLT_SCHEMA_VERSION || options.schema_version != PLT_SCHEMA_VERSION {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-SCHEMA-INVALID",
            "schemaVersion",
            format!("expected schema version {PLT_SCHEMA_VERSION}"),
        ));
    }
    if options.limits.max_output_bytes < PLT_HEADER_LENGTH
        || options.limits.max_output_bytes > PLT_MAX_OUTPUT_BYTES
    {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-SCHEMA-INVALID",
            "options.limits.maxOutputBytes",
            format!("limit must be in {PLT_HEADER_LENGTH}..={PLT_MAX_OUTPUT_BYTES}"),
        ));
    }
    if image.width == 0 || image.height == 0 {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-DIMENSIONS-INVALID",
            "image",
            "width and height must both be positive",
        ));
    }
    if image.num_layers == 0 || image.num_layers > PLT_MAX_LAYERS {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-LAYER-INVALID",
            "image.numLayers",
            format!("numLayers must be in 1..={PLT_MAX_LAYERS}"),
        ));
    }
    let expected = u64::from(image.width)
        .checked_mul(u64::from(image.height))
        .ok_or_else(|| {
            PltErrorV1::fatal(
                "M2A-PLT-DIMENSIONS-INVALID",
                "image",
                "pixel count overflows u64",
            )
        })?;
    if image.pixels.len() as u64 != expected {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-PIXEL-LENGTH-INVALID",
            "image.pixels",
            format!("expected {expected} pixels, got {}", image.pixels.len()),
        ));
    }
    if let Some((index, pixel)) = image
        .pixels
        .iter()
        .enumerate()
        .find(|(_, pixel)| u32::from(pixel.layer_index) >= image.num_layers)
    {
        return Err(PltErrorV1::fatal(
            "M2A-PLT-LAYER-INVALID",
            "image.pixels",
            format!(
                "pixel {index} uses layer {} outside numLayers {}",
                pixel.layer_index, image.num_layers
            ),
        ));
    }
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, PltErrorV1> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        PltErrorV1::fatal(
            "M2A-PLT-HEADER-INVALID",
            "payload",
            "u32 header read is out of bounds",
        )
    })?;
    Ok(u32::from_le_bytes(raw.try_into().expect("four-byte slice")))
}
