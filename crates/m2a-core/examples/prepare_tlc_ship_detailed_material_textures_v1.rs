use std::{
    collections::BTreeMap,
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use image::{DynamicImage, ImageFormat, Rgb, RgbImage, imageops::FilterType};
use m2a_core::{
    glb::{GlbLimits, ingest_glb},
    model_material_separation::{ModelMaterialSeparationDocumentV2, resolve_model_materials_v2},
    model_texture_authoring::{
        ModelTextureBindingModeV1, ModelTexturePayloadDescriptorV1,
        default_model_texture_authoring_v1, model_texture_authoring_hash_v1,
        resolve_model_texture_authoring_v1,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const TARGET_DIMENSION: u32 = 1024;
const BASE_TEXTURE_RESREF: &str = "m2a_tlcdm4_tex";
const DIRECT_MATERIALS: [&str; 4] = ["cloth", "metal", "rope", "sail"];
const WOOD_MATERIALS: [(&str, &str); 8] = [
    ("wood_bow", "wood_medium"),
    ("wood_deck", "wood_light"),
    ("wood_frame", "wood_dark"),
    ("wood_hull", "wood_medium"),
    ("wood_masts", "wood_dark"),
    ("wood_rails", "wood_light"),
    ("wood_stern", "wood_medium"),
    ("wood_supports", "wood_dark"),
];

#[derive(Clone, Copy, Debug)]
enum WoodGrade {
    Medium,
    Light,
    Dark,
}

impl WoodGrade {
    fn asset_name(self) -> &'static str {
        match self {
            Self::Medium => "wood_medium",
            Self::Light => "wood_light",
            Self::Dark => "wood_dark",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetReport {
    asset_name: String,
    source_path: PathBuf,
    output_path: PathBuf,
    output_sha256: String,
    output_byte_length: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparationReport {
    schema_version: u32,
    status: String,
    source_sha256: String,
    separation_sha256: String,
    texture_authoring_sha256: String,
    material_count: usize,
    distinct_texture_resource_count: usize,
    shared_wood_texture_families: BTreeMap<String, Vec<String>>,
    target_dimension: u32,
    filter: String,
    geometry_cleanup: bool,
    assets: Vec<AssetReport>,
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn load_normalized_rgb(path: &Path) -> Result<RgbImage, Box<dyn std::error::Error>> {
    let payload = fs::read(path)?;
    let decoded = image::load_from_memory_with_format(&payload, ImageFormat::Png)?.into_rgb8();
    Ok(image::imageops::resize(
        &decoded,
        TARGET_DIMENSION,
        TARGET_DIMENSION,
        FilterType::Lanczos3,
    ))
}

fn average_luma(image: &RgbImage) -> f32 {
    image
        .pixels()
        .map(|pixel| 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32)
        .sum::<f32>()
        / (image.width() * image.height()).max(1) as f32
}

fn grade_wood(source: &RgbImage, grade: WoodGrade) -> RgbImage {
    let center = average_luma(source);
    let (gain, bias, contrast) = match grade {
        WoodGrade::Medium => ([1.06, 1.00, 0.92], [2.0, 0.0, -1.0], 1.55),
        WoodGrade::Light => ([1.16, 1.10, 1.00], [7.0, 4.0, 2.0], 1.60),
        WoodGrade::Dark => ([0.80, 0.75, 0.68], [-2.0, -3.0, -4.0], 1.90),
    };
    ImageBufferExt::map_rgb(source, |pixel| {
        let luma = 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
        let detail = (luma - center) * (contrast - 1.0);
        Rgb(std::array::from_fn(|channel| {
            (pixel[channel] as f32 * gain[channel] + bias[channel] + detail)
                .round()
                .clamp(0.0, 255.0) as u8
        }))
    })
}

struct ImageBufferExt;

impl ImageBufferExt {
    fn map_rgb(source: &RgbImage, map: impl Fn(Rgb<u8>) -> Rgb<u8>) -> RgbImage {
        let mut output = RgbImage::new(source.width(), source.height());
        for (x, y, pixel) in source.enumerate_pixels() {
            output.put_pixel(x, y, map(*pixel));
        }
        output
    }
}

fn encode_png(image: RgbImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(image).write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source_path = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_detailed_material_textures_v1 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    let recipe_path = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_detailed_material_textures_v1 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    let raw_texture_directory = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_detailed_material_textures_v1 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_detailed_material_textures_v1 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    if args.next().is_some() {
        return Err("usage: prepare_tlc_ship_detailed_material_textures_v1 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>".into());
    }
    if output.exists() {
        return Err(format!("output already exists: {}", output.display()).into());
    }

    let source = fs::read(&source_path)?;
    let limits = GlbLimits::default();
    let ingest = ingest_glb(&source, &limits)?;
    if ingest.ir.source.sha256 != SOURCE_SHA256 {
        return Err("exact source identity mismatch".into());
    }
    let recipe: ModelMaterialSeparationDocumentV2 =
        serde_json::from_slice(&fs::read(&recipe_path)?)?;
    let materials = resolve_model_materials_v2(&ingest.ir, &recipe)?;
    if materials.report.material_slots.len() != 12
        || materials.report.output_triangle_count != materials.report.source_triangle_count
        || materials.report.unassigned_face_count != 0
    {
        return Err("detailed material recipe invariant failed".into());
    }
    let mut authoring = default_model_texture_authoring_v1(&ingest, materials.projection_v1())?;
    let normalized_directory = output.join("normalized");
    fs::create_dir_all(&normalized_directory)?;

    let mut asset_payloads = BTreeMap::<String, Vec<u8>>::new();
    let mut source_paths = BTreeMap::<String, PathBuf>::new();
    for material in DIRECT_MATERIALS {
        let source_path = raw_texture_directory.join(format!("{material}.png"));
        let payload = encode_png(load_normalized_rgb(&source_path)?)?;
        asset_payloads.insert(material.to_owned(), payload);
        source_paths.insert(material.to_owned(), source_path);
    }
    let wood_source_path = raw_texture_directory.join("wood.png");
    let wood = load_normalized_rgb(&wood_source_path)?;
    for grade in [WoodGrade::Medium, WoodGrade::Light, WoodGrade::Dark] {
        asset_payloads.insert(
            grade.asset_name().to_owned(),
            encode_png(grade_wood(&wood, grade))?,
        );
        source_paths.insert(grade.asset_name().to_owned(), wood_source_path.clone());
    }

    let mut blob = Vec::new();
    let mut descriptors = Vec::new();
    let mut reports = Vec::new();
    for (asset_name, payload) in &asset_payloads {
        let path = normalized_directory.join(format!("{asset_name}.png"));
        fs::write(&path, payload)?;
        if fs::read(&path)? != *payload {
            return Err(format!("normalized texture readback mismatch: {asset_name}").into());
        }
        let asset_id = format!("tlc-ship-detailed:{asset_name}:1024");
        let digest = sha256(payload);
        descriptors.push(ModelTexturePayloadDescriptorV1 {
            schema_version: 1,
            asset_id,
            sha256: digest.clone(),
            mime_type: "image/png".to_owned(),
            byte_offset: blob.len() as u64,
            byte_length: payload.len() as u64,
        });
        blob.extend_from_slice(payload);
        reports.push(AssetReport {
            asset_name: asset_name.clone(),
            source_path: source_paths[asset_name].clone(),
            output_path: path,
            output_sha256: digest,
            output_byte_length: payload.len() as u64,
        });
    }
    let material_preview_directory = output.join("material-normalized");
    fs::create_dir_all(&material_preview_directory)?;
    for material in DIRECT_MATERIALS {
        fs::write(
            material_preview_directory.join(format!("{material}.png")),
            &asset_payloads[material],
        )?;
    }
    for (material, asset_name) in WOOD_MATERIALS {
        fs::write(
            material_preview_directory.join(format!("{material}.png")),
            &asset_payloads[asset_name],
        )?;
    }
    let descriptor_by_name = descriptors
        .iter()
        .map(|descriptor| {
            let name = descriptor
                .asset_id
                .strip_prefix("tlc-ship-detailed:")
                .and_then(|value| value.strip_suffix(":1024"))
                .expect("locally generated asset id");
            (name.to_owned(), descriptor)
        })
        .collect::<BTreeMap<_, _>>();
    for binding in &mut authoring.bindings {
        let asset_name = if DIRECT_MATERIALS.contains(&binding.authored_material_id.as_str()) {
            binding.authored_material_id.as_str()
        } else {
            WOOD_MATERIALS
                .iter()
                .find(|(material, _)| *material == binding.authored_material_id)
                .map(|(_, asset)| *asset)
                .ok_or_else(|| {
                    format!(
                        "unexpected detailed material: {}",
                        binding.authored_material_id
                    )
                })?
        };
        let descriptor = descriptor_by_name[asset_name];
        binding.mode = ModelTextureBindingModeV1::Override;
        binding.override_asset_id = Some(descriptor.asset_id.clone());
        binding.override_sha256 = Some(descriptor.sha256.clone());
        binding.override_mime_type = Some(descriptor.mime_type.clone());
        binding.override_byte_length = Some(descriptor.byte_length);
    }

    let resolved = resolve_model_texture_authoring_v1(
        &source,
        &limits,
        &ingest,
        materials.projection_v1(),
        BASE_TEXTURE_RESREF,
        &authoring,
        &blob,
        &descriptors,
    )?;
    if resolved.report.bindings.len() != 12 || resolved.report.resources.len() != 7 {
        return Err(format!(
            "expected 12 bindings and 7 deduplicated textures, got {} and {}",
            resolved.report.bindings.len(),
            resolved.report.resources.len()
        )
        .into());
    }
    let tga_directory = output.join("tga");
    fs::create_dir_all(&tga_directory)?;
    for texture in &resolved.textures {
        fs::write(
            tga_directory.join(format!("{}.tga", texture.resref)),
            &texture.payload,
        )?;
    }
    let texture_authoring_sha256 = model_texture_authoring_hash_v1(&authoring)?;
    if texture_authoring_sha256 != resolved.report.authoring_sha256 {
        return Err("texture authoring hash readback mismatch".into());
    }
    let mut shared = BTreeMap::<String, Vec<String>>::new();
    for (material, asset) in WOOD_MATERIALS {
        shared
            .entry(asset.to_owned())
            .or_default()
            .push(material.to_owned());
    }
    let report = PreparationReport {
        schema_version: 1,
        status: "offline_detailed_texture_set_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        separation_sha256: materials.report.separation_sha256.clone(),
        texture_authoring_sha256,
        material_count: resolved.report.bindings.len(),
        distinct_texture_resource_count: resolved.report.resources.len(),
        shared_wood_texture_families: shared,
        target_dimension: TARGET_DIMENSION,
        filter: "Lanczos3 plus deterministic warm timber value/contrast grades".to_owned(),
        geometry_cleanup: false,
        assets: reports,
    };
    fs::write(
        output.join("texture-authoring.json"),
        serde_json::to_vec_pretty(&authoring)?,
    )?;
    fs::write(
        output.join("texture-payload-descriptors.json"),
        serde_json::to_vec_pretty(&descriptors)?,
    )?;
    fs::write(output.join("texture-payload.bin"), &blob)?;
    fs::write(
        output.join("texture-resolution-report.json"),
        serde_json::to_vec_pretty(&resolved.report)?,
    )?;
    fs::write(
        output.join("preparation-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(pixel: Rgb<u8>) -> f32 {
        0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
    }

    #[test]
    fn wood_grades_are_warm_and_value_ordered() {
        let source = RgbImage::from_pixel(2, 2, Rgb([100, 75, 50]));
        let light = grade_wood(&source, WoodGrade::Light)
            .get_pixel(0, 0)
            .to_owned();
        let medium = grade_wood(&source, WoodGrade::Medium)
            .get_pixel(0, 0)
            .to_owned();
        let dark = grade_wood(&source, WoodGrade::Dark)
            .get_pixel(0, 0)
            .to_owned();
        assert!(luma(light) > luma(medium));
        assert!(luma(medium) > luma(dark));
        for pixel in [light, medium, dark] {
            assert!(pixel[0] > pixel[1] && pixel[1] > pixel[2]);
        }
    }
}
