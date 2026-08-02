//! Minimal, deterministic Aurora PLT V1 writer and semantic readback.
//!
//! PLT stores one palette color index and one palette-layer index per pixel.
//! The six item material colors map to layers 2..=7; the selected UTI color
//! values choose palette rows at runtime.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::tga::{TGA_SCHEMA_VERSION, TgaImageV1, TgaPixelFormatV1};

pub const PLT_SCHEMA_VERSION_V1: u32 = 1;
pub const PLT_LAYER_COUNT_V1: u32 = 10;
pub const PLT_HEADER_BYTE_LENGTH_V1: usize = 24;
pub const PLT_MAX_OUTPUT_BYTES_V1: u64 = 64 * 1024 * 1024;
const PLT_SIGNATURE_V1: &[u8; 8] = b"PLT V1  ";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum ItemPltLayerV1 {
    Metal1 = 2,
    Metal2 = 3,
    Cloth1 = 4,
    Cloth2 = 5,
    Leather1 = 6,
    Leather2 = 7,
}

impl ItemPltLayerV1 {
    pub const fn index(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PltWriterOptionsV1 {
    pub schema_version: u32,
    pub layer: ItemPltLayerV1,
    pub max_output_bytes: u64,
}

impl Default for PltWriterOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: PLT_SCHEMA_VERSION_V1,
            layer: ItemPltLayerV1::Leather1,
            max_output_bytes: PLT_MAX_OUTPUT_BYTES_V1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PltWriterReportV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub layer: ItemPltLayerV1,
    pub pixel_count: u64,
    pub byte_length: u64,
    pub source_pixel_sha256: String,
    pub output_sha256: String,
    pub semantic_readback_status: String,
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
    pub path: String,
    pub message: String,
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

fn error(code: &str, path: &str, message: impl Into<String>) -> PltErrorV1 {
    PltErrorV1 {
        schema_version: PLT_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn checked_layout(
    image: &TgaImageV1,
    options: &PltWriterOptionsV1,
) -> Result<(u64, usize), PltErrorV1> {
    if image.schema_version != TGA_SCHEMA_VERSION {
        return Err(error(
            "ITEM-PLT-SCHEMA-INVALID",
            "image.schemaVersion",
            "source image schemaVersion must be 1",
        ));
    }
    if options.schema_version != PLT_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-PLT-SCHEMA-INVALID",
            "options.schemaVersion",
            "PLT writer schemaVersion must be 1",
        ));
    }
    if image.width == 0 || image.height == 0 {
        return Err(error(
            "ITEM-PLT-DIMENSIONS-INVALID",
            "image.dimensions",
            "PLT width and height must be non-zero",
        ));
    }
    let channels = match image.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3usize,
        TgaPixelFormatV1::Rgba8 => 4usize,
    };
    let pixel_count = u64::from(image.width)
        .checked_mul(u64::from(image.height))
        .ok_or_else(|| {
            error(
                "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
                "image.dimensions",
                "PLT pixel count overflowed",
            )
        })?;
    let source_length = pixel_count.checked_mul(channels as u64).ok_or_else(|| {
        error(
            "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
            "image.pixels",
            "source pixel length overflowed",
        )
    })?;
    if source_length != image.pixels.len() as u64 {
        return Err(error(
            "ITEM-PLT-PIXEL-LENGTH-INVALID",
            "image.pixels",
            format!(
                "source image has {} bytes but {source_length} are required",
                image.pixels.len()
            ),
        ));
    }
    let output_length = (PLT_HEADER_BYTE_LENGTH_V1 as u64)
        .checked_add(pixel_count.checked_mul(2).ok_or_else(|| {
            error(
                "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
                "output",
                "PLT payload length overflowed",
            )
        })?)
        .ok_or_else(|| {
            error(
                "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
                "output",
                "PLT output length overflowed",
            )
        })?;
    if options.max_output_bytes == 0
        || options.max_output_bytes > PLT_MAX_OUTPUT_BYTES_V1
        || output_length > options.max_output_bytes
    {
        return Err(error(
            "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
            "options.maxOutputBytes",
            format!(
                "PLT output requires {output_length} bytes; configured limit is {}",
                options.max_output_bytes
            ),
        ));
    }
    Ok((pixel_count, channels))
}

fn luma_index(pixel: &[u8]) -> u8 {
    // Integer BT.601 luma, rounded, keeps output deterministic across hosts.
    let value = 77u32 * u32::from(pixel[0])
        + 150u32 * u32::from(pixel[1])
        + 29u32 * u32::from(pixel[2])
        + 128;
    (value >> 8) as u8
}

pub fn write_item_plt_v1(
    image: &TgaImageV1,
    options: &PltWriterOptionsV1,
) -> Result<PltArtifactV1, PltErrorV1> {
    let (pixel_count, channels) = checked_layout(image, options)?;
    let width = image.width as usize;
    let height = image.height as usize;
    let row_bytes = width
        .checked_mul(channels)
        .ok_or_else(|| error("ITEM-PLT-OUTPUT-LIMIT-EXCEEDED", "output", "row overflow"))?;
    let output_length = PLT_HEADER_BYTE_LENGTH_V1
        .checked_add(
            usize::try_from(pixel_count)
                .map_err(|_| {
                    error(
                        "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
                        "output",
                        "pixel count does not fit this platform",
                    )
                })?
                .checked_mul(2)
                .ok_or_else(|| {
                    error(
                        "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
                        "output",
                        "payload overflow",
                    )
                })?,
        )
        .ok_or_else(|| {
            error(
                "ITEM-PLT-OUTPUT-LIMIT-EXCEEDED",
                "output",
                "output overflow",
            )
        })?;
    let mut payload = Vec::new();
    payload.try_reserve_exact(output_length).map_err(|_| {
        error(
            "ITEM-PLT-ALLOCATION-FAILED",
            "output",
            "unable to reserve PLT output",
        )
    })?;
    payload.extend_from_slice(PLT_SIGNATURE_V1);
    payload.extend_from_slice(&PLT_LAYER_COUNT_V1.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&image.width.to_le_bytes());
    payload.extend_from_slice(&image.height.to_le_bytes());
    for source_row in (0..height).rev() {
        let start = source_row * row_bytes;
        for pixel in image.pixels[start..start + row_bytes].chunks_exact(channels) {
            if channels == 4 && pixel[3] != 255 {
                return Err(error(
                    "ITEM-PLT-ALPHA-UNSUPPORTED",
                    "image.pixels",
                    "PLT V1 has no alpha lane; the selected Meshy texture must be fully opaque",
                ));
            }
            payload.push(luma_index(pixel));
            payload.push(options.layer.index());
        }
    }
    let readback = inspect_item_plt_v1(&payload)?;
    if readback.width != image.width
        || readback.height != image.height
        || readback.layer != options.layer
        || readback.color_indices
            != image
                .pixels
                .chunks_exact(channels)
                .map(luma_index)
                .collect::<Vec<_>>()
    {
        return Err(error(
            "ITEM-PLT-SEMANTIC-DIFF",
            "output",
            "PLT readback differs from the requested image/layer",
        ));
    }
    Ok(PltArtifactV1 {
        report: PltWriterReportV1 {
            schema_version: PLT_SCHEMA_VERSION_V1,
            width: image.width,
            height: image.height,
            layer: options.layer,
            pixel_count,
            byte_length: payload.len() as u64,
            source_pixel_sha256: sha256(&image.pixels),
            output_sha256: sha256(&payload),
            semantic_readback_status: "PASS".to_owned(),
        },
        payload,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PltReadbackV1 {
    pub width: u32,
    pub height: u32,
    pub layer: ItemPltLayerV1,
    pub color_indices: Vec<u8>,
}

pub fn inspect_item_plt_v1(bytes: &[u8]) -> Result<PltReadbackV1, PltErrorV1> {
    if bytes.len() < PLT_HEADER_BYTE_LENGTH_V1 || &bytes[..8] != PLT_SIGNATURE_V1 {
        return Err(error(
            "ITEM-PLT-HEADER-INVALID",
            "header",
            "payload must start with the 24-byte PLT V1 header",
        ));
    }
    let read_u32 = |offset: usize| {
        u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("validated PLT header"),
        )
    };
    if read_u32(8) != PLT_LAYER_COUNT_V1 || read_u32(12) != 0 {
        return Err(error(
            "ITEM-PLT-HEADER-INVALID",
            "header",
            "PLT header must declare ten layers and reserved value zero",
        ));
    }
    let width = read_u32(16);
    let height = read_u32(20);
    let pixel_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| error("ITEM-PLT-LAYOUT-INVALID", "header", "pixel count overflow"))?;
    let expected = (PLT_HEADER_BYTE_LENGTH_V1 as u64)
        .checked_add(pixel_count.checked_mul(2).ok_or_else(|| {
            error(
                "ITEM-PLT-LAYOUT-INVALID",
                "payload",
                "payload length overflow",
            )
        })?)
        .ok_or_else(|| {
            error(
                "ITEM-PLT-LAYOUT-INVALID",
                "payload",
                "output length overflow",
            )
        })?;
    if expected != bytes.len() as u64 {
        return Err(error(
            "ITEM-PLT-LAYOUT-INVALID",
            "payload",
            format!(
                "payload has {} bytes but {expected} are required",
                bytes.len()
            ),
        ));
    }
    let mut layer = None;
    let mut stored = Vec::with_capacity(pixel_count as usize);
    for pair in bytes[PLT_HEADER_BYTE_LENGTH_V1..].chunks_exact(2) {
        let current = match pair[1] {
            2 => ItemPltLayerV1::Metal1,
            3 => ItemPltLayerV1::Metal2,
            4 => ItemPltLayerV1::Cloth1,
            5 => ItemPltLayerV1::Cloth2,
            6 => ItemPltLayerV1::Leather1,
            7 => ItemPltLayerV1::Leather2,
            value => {
                return Err(error(
                    "ITEM-PLT-LAYER-INVALID",
                    "payload.layer",
                    format!("item PLT layer {value} is outside the supported material layers 2..7"),
                ));
            }
        };
        if layer
            .replace(current)
            .is_some_and(|previous| previous != current)
        {
            return Err(error(
                "ITEM-PLT-LAYER-MIXED",
                "payload.layer",
                "single-source Item PLT payload must use exactly one material layer",
            ));
        }
        stored.push(pair[0]);
    }
    let mut color_indices = Vec::with_capacity(stored.len());
    let row_width = width as usize;
    for row in (0..height as usize).rev() {
        color_indices.extend_from_slice(&stored[row * row_width..(row + 1) * row_width]);
    }
    Ok(PltReadbackV1 {
        width,
        height,
        layer: layer.ok_or_else(|| {
            error(
                "ITEM-PLT-DIMENSIONS-INVALID",
                "header",
                "PLT width and height must be non-zero",
            )
        })?,
        color_indices,
    })
}
