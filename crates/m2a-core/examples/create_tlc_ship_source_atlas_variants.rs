use std::{env, fs, path::PathBuf};

use image::{ImageFormat, Rgb, RgbImage, imageops};
use m2a_core::{
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    tga::TgaPixelFormatV1,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const POLYHAVEN_WOOD_PLANKS_SHA256: &str =
    "3b0669f683e4bf10f5a55a381cfa9669a7b8dfd921901829daa3b35acc2bbdec";
const MATERIALS: [&str; 5] = ["wood", "rope", "sail", "cloth", "metal"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantReport {
    authored_material_id: String,
    transform: String,
    output_path: PathBuf,
    output_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema_version: u32,
    status: String,
    source_sha256: String,
    source_image_sha256: String,
    source_width: u32,
    source_height: u32,
    strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_wood_source_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_wood_source_path: Option<PathBuf>,
    source_uv0_preserved: bool,
    geometry_cleanup: bool,
    variants: Vec<VariantReport>,
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn clamp(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn mix(left: f32, right: f32, amount: f32) -> f32 {
    left * (1.0 - amount) + right * amount
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn grade(material: &str, source: Rgb<u8>, blurred: Rgb<u8>) -> Rgb<u8> {
    let [r, g, b] = source.0.map(f32::from);
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let [blurred_r, blurred_g, blurred_b] = blurred.0.map(f32::from);
    let blurred_luma = 0.2126 * blurred_r + 0.7152 * blurred_g + 0.0722 * blurred_b;
    let graded = match material {
        // Re-map the existing painted values to a warm timber palette. Preserve
        // dark source seams, but suppress positive high-pass halos that made v7
        // look chalky and carved. No grain or plank layout is synthesized here.
        "wood" => {
            let tone = smoothstep((luma - 30.0) / 190.0);
            let local_delta = luma - blurred_luma;
            let seam = if local_delta < 0.0 {
                (local_delta * 0.20).max(-8.0)
            } else {
                (local_delta * 0.035).min(1.5)
            };
            let source_chroma = [r - luma, g - luma, b - luma];
            let shadow = [43.0, 29.0, 18.0];
            let highlight = [164.0, 108.0, 61.0];
            [
                mix(shadow[0], highlight[0], tone) + source_chroma[0] * 0.16 + seam,
                mix(shadow[1], highlight[1], tone) + source_chroma[1] * 0.12 + seam,
                mix(shadow[2], highlight[2], tone) + source_chroma[2] * 0.08 + seam,
            ]
        }
        "rope" => [
            mix(r, luma * 0.95 + 35.0, 0.55),
            mix(g, luma * 0.75 + 28.0, 0.55),
            mix(b, luma * 0.45 + 15.0, 0.55),
        ],
        "sail" => [
            mix(r, luma * 1.25 + 45.0, 0.75),
            mix(g, luma * 1.18 + 42.0, 0.75),
            mix(b, luma * 1.05 + 36.0, 0.75),
        ],
        "cloth" => [
            mix(r, luma * 0.95 + 35.0, 0.60),
            mix(g, luma * 0.78 + 28.0, 0.60),
            mix(b, luma * 0.62 + 23.0, 0.60),
        ],
        "metal" => [
            mix(r, luma * 0.30 + 8.0, 0.82),
            mix(g, luma * 0.35 + 10.0, 0.82),
            mix(b, luma * 0.42 + 14.0, 0.82),
        ],
        _ => unreachable!("fixed material inventory"),
    };
    Rgb(graded.map(clamp))
}

fn transform_description(material: &str, external_wood: bool) -> &'static str {
    match material {
        "wood" if external_wood => {
            "CC0 Poly Haven Wood Planks diffuse regraded to the NWN ship-reference midtones"
        }
        "wood" => {
            "source-preserving warm timber palette with dark-seam retention and bright-halo suppression"
        }
        "rope" => "source-preserving high-separation golden hemp grade",
        "sail" => "source-preserving high-separation ivory canvas lift",
        "cloth" => "source-preserving medium-value muted taupe grade",
        "metal" => "source-preserving low-value cool forged-metal grade",
        _ => unreachable!("fixed material inventory"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wood_grade_maps_neutral_source_to_warm_brown() {
        let output = grade("wood", Rgb([120, 120, 120]), Rgb([120, 120, 120]));

        assert!(output[0] > output[1]);
        assert!(output[1] > output[2]);
        assert!(output[0] - output[2] >= 40);
    }

    #[test]
    fn wood_grade_suppresses_bright_high_pass_halos() {
        let flat = grade("wood", Rgb([140, 140, 140]), Rgb([140, 140, 140]));
        let positive_edge = grade("wood", Rgb([140, 140, 140]), Rgb([100, 100, 100]));

        for channel in 0..3 {
            assert!(positive_edge[channel].saturating_sub(flat[channel]) <= 2);
        }
    }

    #[test]
    fn wood_grade_retains_dark_source_seams() {
        let flat = grade("wood", Rgb([80, 80, 80]), Rgb([80, 80, 80]));
        let seam = grade("wood", Rgb([80, 80, 80]), Rgb([120, 120, 120]));

        for channel in 0..3 {
            assert!(flat[channel].saturating_sub(seam[channel]) >= 7);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source_path =
        PathBuf::from(args.next().ok_or(
            "usage: create_tlc_ship_source_atlas_variants <source.glb> <output-directory>",
        )?);
    let output =
        PathBuf::from(args.next().ok_or(
            "usage: create_tlc_ship_source_atlas_variants <source.glb> <output-directory> [external-wood-image]",
        )?);
    let external_wood_path = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err(
            "usage: create_tlc_ship_source_atlas_variants <source.glb> <output-directory> [external-wood-image]".into(),
        );
    }
    if output.exists() {
        return Err(format!("output already exists: {}", output.display()).into());
    }

    let source = fs::read(&source_path)?;
    let limits = GlbLimits::default();
    let ingest = ingest_glb(&source, &limits)?;
    if ingest.ir.source.sha256 != SOURCE_SHA256 || ingest.ir.images.len() != 1 {
        return Err("exact TLC ship source/image identity mismatch".into());
    }
    let decoded = decode_embedded_image_to_tga_v1(
        &source,
        0,
        &limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )?;
    let channels = match decoded.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3,
        TgaPixelFormatV1::Rgba8 => 4,
    };
    let base = RgbImage::from_fn(decoded.width, decoded.height, |x, y| {
        let offset = ((y * decoded.width + x) as usize) * channels;
        Rgb([
            decoded.pixels[offset],
            decoded.pixels[offset + 1],
            decoded.pixels[offset + 2],
        ])
    });
    let blurred = imageops::blur(&base, 4.0);
    let external_wood = external_wood_path
        .as_ref()
        .map(|path| -> Result<RgbImage, Box<dyn std::error::Error>> {
            let payload = fs::read(path)?;
            let digest = sha256(&payload);
            if digest != POLYHAVEN_WOOD_PLANKS_SHA256 {
                return Err(format!(
                    "external wood identity mismatch: expected {POLYHAVEN_WOOD_PLANKS_SHA256}, got {digest}"
                )
                .into());
            }
            Ok(image::open(path)?.into_rgb8())
        })
        .transpose()?;
    let external_wood_blurred = external_wood
        .as_ref()
        .map(|image| imageops::blur(image, 4.0));

    fs::create_dir_all(&output)?;
    let mut variants = Vec::new();
    for material in MATERIALS {
        let (material_base, material_blurred) = if material == "wood"
            && let (Some(wood), Some(wood_blurred)) =
                (external_wood.as_ref(), external_wood_blurred.as_ref())
        {
            (wood, wood_blurred)
        } else {
            (&base, &blurred)
        };
        let image = RgbImage::from_fn(material_base.width(), material_base.height(), |x, y| {
            grade(
                material,
                *material_base.get_pixel(x, y),
                *material_blurred.get_pixel(x, y),
            )
        });
        let path = output.join(format!("{material}.png"));
        image.save_with_format(&path, ImageFormat::Png)?;
        let payload = fs::read(&path)?;
        variants.push(VariantReport {
            authored_material_id: material.to_owned(),
            transform: transform_description(material, external_wood.is_some()).to_owned(),
            output_path: path,
            output_sha256: sha256(&payload),
        });
    }
    let report = Report {
        schema_version: 1,
        status: "offline_source_atlas_variants_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        source_image_sha256: ingest.ir.images[0].sha256.clone(),
        source_width: decoded.width,
        source_height: decoded.height,
        strategy: if external_wood.is_some() {
            "material-local texture set: CC0 plank diffuse for projected Wood UV; exact source-atlas material grades for Rope, Sail, Cloth and Metal"
        } else {
            "material-specific full-atlas color grades preserving the exact source UV layout and local painted detail"
        }
        .to_owned(),
        external_wood_source_sha256: external_wood
            .as_ref()
            .map(|_| POLYHAVEN_WOOD_PLANKS_SHA256.to_owned()),
        external_wood_source_path: external_wood_path,
        source_uv0_preserved: true,
        geometry_cleanup: false,
        variants,
    };
    fs::write(
        output.join("source-atlas-variant-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
