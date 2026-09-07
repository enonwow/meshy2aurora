//! Byte-producing material bake executor for Aurora target profiles.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    aurora_material::{AuroraClassicDiffuseBakePlanV1, AuroraSpecularTexturePlanV1},
    tga::{
        TgaArtifactV1, TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, read_tga_image_v1,
        write_tga_v1,
    },
};

pub const AURORA_MATERIAL_BAKE_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraMaterialBakeKindV1 {
    ClassicDiffusePbrEnergyApproximation,
    EeSpecularGloss,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuroraMaterialBakeReportV1 {
    pub schema_version: u32,
    pub kind: AuroraMaterialBakeKindV1,
    pub width: u32,
    pub height: u32,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_source_width: Option<u32>,
    pub metallic_roughness_source_height: Option<u32>,
    pub metallic_roughness_resampled: bool,
    pub base_color_pixel_sha256: String,
    pub metallic_roughness_pixel_sha256: Option<String>,
    pub output_pixel_sha256: String,
    pub output_tga_sha256: String,
    pub readback_pixel_sha256: String,
    pub semantic_readback_equal: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuroraMaterialBakeArtifactV1 {
    pub image: TgaImageV1,
    pub tga: TgaArtifactV1,
    pub report: AuroraMaterialBakeReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuroraMaterialBakeErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for AuroraMaterialBakeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for AuroraMaterialBakeErrorV1 {}

fn error(code: &str, path: &str, message: impl Into<String>) -> AuroraMaterialBakeErrorV1 {
    AuroraMaterialBakeErrorV1 {
        schema_version: AURORA_MATERIAL_BAKE_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn channels(format: TgaPixelFormatV1) -> usize {
    match format {
        TgaPixelFormatV1::Rgb8 => 3,
        TgaPixelFormatV1::Rgba8 => 4,
    }
}

fn validate_image(image: &TgaImageV1, path: &str) -> Result<(), AuroraMaterialBakeErrorV1> {
    let expected = usize::try_from(image.width)
        .ok()
        .and_then(|width| {
            usize::try_from(image.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(channels(image.pixel_format)));
    if image.schema_version != 1
        || image.width == 0
        || image.height == 0
        || expected != Some(image.pixels.len())
    {
        return Err(error(
            "AURORA-BAKE-IMAGE-INVALID",
            path,
            "image schema, dimensions, format, and pixel length must agree",
        ));
    }
    Ok(())
}

fn validate_inputs(
    base_color: &TgaImageV1,
    metallic_roughness: Option<&TgaImageV1>,
    metallic_factor: f32,
    roughness_factor: f32,
) -> Result<(), AuroraMaterialBakeErrorV1> {
    validate_image(base_color, "baseColor")?;
    if !(0.0..=1.0).contains(&metallic_factor) || !(0.0..=1.0).contains(&roughness_factor) {
        return Err(error(
            "AURORA-BAKE-FACTOR-RANGE",
            "material",
            "metallic and roughness factors must be finite values in 0..=1",
        ));
    }
    if let Some(image) = metallic_roughness {
        validate_image(image, "metallicRoughness")?;
    }
    Ok(())
}

fn srgb_to_linear(value: u8) -> f32 {
    let value = f32::from(value) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let encoded = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

fn mr_values(
    image: Option<&TgaImageV1>,
    pixel: usize,
    output_width: u32,
    output_height: u32,
    metallic_factor: f32,
    roughness_factor: f32,
) -> (f32, f32) {
    match image {
        Some(image) => {
            // glTF metallic-roughness: G=roughness, B=metallic.
            (
                sample_channel_bilinear(image, pixel, output_width, output_height, 2)
                    * metallic_factor,
                sample_channel_bilinear(image, pixel, output_width, output_height, 1)
                    * roughness_factor,
            )
        }
        None => (metallic_factor, roughness_factor),
    }
}

fn sample_channel_bilinear(
    image: &TgaImageV1,
    output_pixel: usize,
    output_width: u32,
    output_height: u32,
    channel: usize,
) -> f32 {
    let output_width_usize = output_width as usize;
    let output_x = output_pixel % output_width_usize;
    let output_y = output_pixel / output_width_usize;
    let source_x = ((output_x as f64 + 0.5) * f64::from(image.width) / f64::from(output_width)
        - 0.5)
        .clamp(0.0, f64::from(image.width - 1));
    let source_y = ((output_y as f64 + 0.5) * f64::from(image.height) / f64::from(output_height)
        - 0.5)
        .clamp(0.0, f64::from(image.height - 1));
    let x0 = source_x.floor() as u32;
    let y0 = source_y.floor() as u32;
    let x1 = (x0 + 1).min(image.width - 1);
    let y1 = (y0 + 1).min(image.height - 1);
    let tx = (source_x - f64::from(x0)) as f32;
    let ty = (source_y - f64::from(y0)) as f32;
    let stride = channels(image.pixel_format);
    let sample = |x: u32, y: u32| {
        let pixel = y as usize * image.width as usize + x as usize;
        f32::from(image.pixels[pixel * stride + channel]) / 255.0
    };
    let top = sample(x0, y0) * (1.0 - tx) + sample(x1, y0) * tx;
    let bottom = sample(x0, y1) * (1.0 - tx) + sample(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn finish(
    kind: AuroraMaterialBakeKindV1,
    base_color: &TgaImageV1,
    metallic_roughness: Option<&TgaImageV1>,
    metallic_factor: f32,
    roughness_factor: f32,
    image: TgaImageV1,
) -> Result<AuroraMaterialBakeArtifactV1, AuroraMaterialBakeErrorV1> {
    let output_pixel_sha256 = sha256_hex(&image.pixels);
    let tga = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|source| error("AURORA-BAKE-TGA-WRITE", "output", source.to_string()))?;
    let readback = read_tga_image_v1(&tga.payload)
        .map_err(|source| error("AURORA-BAKE-TGA-READBACK", "output", source.to_string()))?;
    let semantic_readback_equal = readback == image;
    if !semantic_readback_equal {
        return Err(error(
            "AURORA-BAKE-SEMANTIC-DIFF",
            "output",
            "final TGA readback differs from baked pixels",
        ));
    }
    Ok(AuroraMaterialBakeArtifactV1 {
        report: AuroraMaterialBakeReportV1 {
            schema_version: AURORA_MATERIAL_BAKE_SCHEMA_VERSION_V1,
            kind,
            width: image.width,
            height: image.height,
            metallic_factor,
            roughness_factor,
            metallic_roughness_source_width: metallic_roughness.map(|value| value.width),
            metallic_roughness_source_height: metallic_roughness.map(|value| value.height),
            metallic_roughness_resampled: metallic_roughness.is_some_and(|value| {
                (value.width, value.height) != (base_color.width, base_color.height)
            }),
            base_color_pixel_sha256: sha256_hex(&base_color.pixels),
            metallic_roughness_pixel_sha256: metallic_roughness
                .map(|value| sha256_hex(&value.pixels)),
            output_pixel_sha256,
            output_tga_sha256: tga.report.output_sha256.clone(),
            readback_pixel_sha256: sha256_hex(&readback.pixels),
            semantic_readback_equal,
        },
        image,
        tga,
    })
}

/// Bakes PBR response into a classic diffuse texture while retaining metal color readability.
pub fn bake_classic_diffuse_v1(
    plan: &AuroraClassicDiffuseBakePlanV1,
    base_color: &TgaImageV1,
    metallic_roughness: Option<&TgaImageV1>,
) -> Result<AuroraMaterialBakeArtifactV1, AuroraMaterialBakeErrorV1> {
    validate_inputs(
        base_color,
        metallic_roughness,
        plan.metallic_factor,
        plan.roughness_factor,
    )?;
    let stride = channels(base_color.pixel_format);
    let count = base_color.pixels.len() / stride;
    let mut pixels = Vec::with_capacity(count * stride);
    for pixel in 0..count {
        let offset = pixel * stride;
        let (metallic, roughness) = mr_values(
            metallic_roughness,
            pixel,
            base_color.width,
            base_color.height,
            plan.metallic_factor,
            plan.roughness_factor,
        );
        // Classic Aurora has no PBR BRDF. This deterministic approximation retains
        // metal albedo (65% at full metal) and folds roughness into diffuse energy.
        let energy = (1.0 - 0.35 * metallic) * (0.85 + 0.15 * roughness);
        for channel in 0..3 {
            pixels.push(linear_to_srgb(
                srgb_to_linear(base_color.pixels[offset + channel]) * energy,
            ));
        }
        if stride == 4 {
            pixels.push(base_color.pixels[offset + 3]);
        }
    }
    let image = TgaImageV1 {
        schema_version: 1,
        width: base_color.width,
        height: base_color.height,
        pixel_format: base_color.pixel_format,
        pixels,
    };
    finish(
        AuroraMaterialBakeKindV1::ClassicDiffusePbrEnergyApproximation,
        base_color,
        metallic_roughness,
        plan.metallic_factor,
        plan.roughness_factor,
        image,
    )
}

/// Bakes glTF F0 and gloss into RGB+A for the EE texture2 material route.
pub fn bake_specular_gloss_v1(
    plan: &AuroraSpecularTexturePlanV1,
    base_color: &TgaImageV1,
    metallic_roughness: Option<&TgaImageV1>,
) -> Result<AuroraMaterialBakeArtifactV1, AuroraMaterialBakeErrorV1> {
    validate_inputs(
        base_color,
        metallic_roughness,
        plan.metallic_factor,
        plan.roughness_factor,
    )?;
    let stride = channels(base_color.pixel_format);
    let count = base_color.pixels.len() / stride;
    let mut pixels = Vec::with_capacity(count * 4);
    for pixel in 0..count {
        let offset = pixel * stride;
        let (metallic, roughness) = mr_values(
            metallic_roughness,
            pixel,
            base_color.width,
            base_color.height,
            plan.metallic_factor,
            plan.roughness_factor,
        );
        for channel in 0..3 {
            let base = srgb_to_linear(base_color.pixels[offset + channel]);
            pixels.push(linear_to_srgb(0.04 * (1.0 - metallic) + base * metallic));
        }
        pixels.push(((1.0 - roughness).clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    let image = TgaImageV1 {
        schema_version: 1,
        width: base_color.width,
        height: base_color.height,
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels,
    };
    finish(
        AuroraMaterialBakeKindV1::EeSpecularGloss,
        base_color,
        metallic_roughness,
        plan.metallic_factor,
        plan.roughness_factor,
        image,
    )
}
