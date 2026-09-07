use std::{
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use image::{DynamicImage, ImageFormat, imageops::FilterType};
use m2a_core::{
    glb::{GlbLimits, ingest_glb},
    model_material_separation::{ModelMaterialSeparationDocumentV2, resolve_model_materials_v2},
    model_texture_authoring::{
        ModelTextureBindingModeV1, ModelTexturePayloadDescriptorV1,
        default_model_texture_authoring_v1, resolve_model_texture_authoring_v1,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const TARGET_DIMENSION: u32 = 1024;
const BASE_TEXTURE_RESREF: &str = "m2a_tlcsm_tex";
const MATERIALS: [&str; 5] = ["wood", "rope", "sail", "cloth", "metal"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparedTextureReport {
    authored_material_id: String,
    source_path: PathBuf,
    source_sha256: String,
    source_width: u32,
    source_height: u32,
    normalized_path: PathBuf,
    normalized_sha256: String,
    normalized_width: u32,
    normalized_height: u32,
    output_resref: String,
    output_tga_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparationReport {
    schema_version: u32,
    status: String,
    source_sha256: String,
    separation_sha256: String,
    texture_authoring_sha256: String,
    target_dimension: u32,
    filter: String,
    geometry_cleanup: bool,
    source_uv0_preserved: bool,
    textures: Vec<PreparedTextureReport>,
}

struct NormalizedPng {
    payload: Vec<u8>,
    source_sha256: String,
    source_width: u32,
    source_height: u32,
}

struct PreparedTextureIntermediate {
    material: String,
    raw_path: PathBuf,
    source_sha256: String,
    source_width: u32,
    source_height: u32,
    normalized_path: PathBuf,
    normalized_sha256: String,
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn normalize_png(path: &Path) -> Result<NormalizedPng, Box<dyn std::error::Error>> {
    let source = fs::read(path)?;
    let source_sha256 = sha256(&source);
    let decoded = image::load_from_memory_with_format(&source, ImageFormat::Png)?;
    let source_width = decoded.width();
    let source_height = decoded.height();
    let rgb = decoded.into_rgb8();
    let normalized = DynamicImage::ImageRgb8(image::imageops::resize(
        &rgb,
        TARGET_DIMENSION,
        TARGET_DIMENSION,
        FilterType::Lanczos3,
    ));
    let mut output = Cursor::new(Vec::new());
    normalized.write_to(&mut output, ImageFormat::Png)?;
    Ok(NormalizedPng {
        payload: output.into_inner(),
        source_sha256,
        source_width,
        source_height,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source_path = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_material_textures_v2 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    let recipe_path = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_material_textures_v2 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    let raw_texture_directory = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_material_textures_v2 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_material_textures_v2 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>",
    )?);
    if args.next().is_some() {
        return Err(
            "usage: prepare_tlc_ship_material_textures_v2 <source.glb> <recipe.json> <raw-texture-directory> <output-directory>".into(),
        );
    }
    if output.exists() {
        return Err(format!("output already exists: {}", output.display()).into());
    }

    let source = fs::read(&source_path)?;
    let limits = GlbLimits::default();
    let ingest = ingest_glb(&source, &limits)?;
    if ingest.ir.source.sha256 != SOURCE_SHA256 {
        return Err(format!(
            "source hash mismatch: expected {SOURCE_SHA256}, got {}",
            ingest.ir.source.sha256
        )
        .into());
    }
    let recipe: ModelMaterialSeparationDocumentV2 =
        serde_json::from_slice(&fs::read(&recipe_path)?)?;
    let materials = resolve_model_materials_v2(&ingest.ir, &recipe)?;
    if materials.report.output_triangle_count != materials.report.source_triangle_count
        || materials.report.unassigned_face_count != 0
        || materials.report.duplicated_boundary_vertex_count != 0
    {
        return Err("material recipe is not exact geometry-preserving coverage".into());
    }
    let mut authoring = default_model_texture_authoring_v1(&ingest, materials.projection_v1())?;

    let normalized_directory = output.join("normalized");
    fs::create_dir_all(&normalized_directory)?;
    let mut blob = Vec::new();
    let mut descriptors = Vec::new();
    let mut prepared = Vec::new();
    for material in MATERIALS {
        let raw_path = raw_texture_directory.join(format!("{material}.png"));
        let normalized = normalize_png(&raw_path)?;
        let payload = normalized.payload;
        let normalized_sha256 = sha256(&payload);
        let normalized_path = normalized_directory.join(format!("{material}.png"));
        fs::write(&normalized_path, &payload)?;
        if fs::read(&normalized_path)? != payload {
            return Err(format!("normalized texture readback mismatch: {material}").into());
        }
        let asset_id = format!("tlc-ship-material:{material}:1024");
        let offset = blob.len() as u64;
        blob.extend_from_slice(&payload);
        descriptors.push(ModelTexturePayloadDescriptorV1 {
            schema_version: 1,
            asset_id: asset_id.clone(),
            sha256: normalized_sha256.clone(),
            mime_type: "image/png".to_owned(),
            byte_offset: offset,
            byte_length: payload.len() as u64,
        });
        let binding = authoring
            .bindings
            .iter_mut()
            .find(|binding| binding.authored_material_id == material)
            .ok_or_else(|| format!("missing material binding: {material}"))?;
        binding.mode = ModelTextureBindingModeV1::Override;
        binding.override_asset_id = Some(asset_id);
        binding.override_sha256 = Some(normalized_sha256.clone());
        binding.override_mime_type = Some("image/png".to_owned());
        binding.override_byte_length = Some(payload.len() as u64);
        prepared.push(PreparedTextureIntermediate {
            material: material.to_owned(),
            raw_path,
            source_sha256: normalized.source_sha256,
            source_width: normalized.source_width,
            source_height: normalized.source_height,
            normalized_path,
            normalized_sha256,
        });
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
    if resolved.report.resources.len() != MATERIALS.len()
        || resolved.textures.len() != MATERIALS.len()
    {
        return Err(format!(
            "expected {} distinct texture resources, got {}",
            MATERIALS.len(),
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
    let texture_authoring_sha256 =
        m2a_core::model_texture_authoring::model_texture_authoring_hash_v1(&authoring)?;
    if texture_authoring_sha256 != resolved.report.authoring_sha256 {
        return Err("texture authoring hash readback mismatch".into());
    }
    let texture_reports = prepared
        .into_iter()
        .map(|prepared| {
            let binding = resolved
                .report
                .bindings
                .iter()
                .find(|binding| binding.authored_material_id == prepared.material)
                .expect("resolved binding");
            PreparedTextureReport {
                authored_material_id: prepared.material,
                source_path: prepared.raw_path,
                source_sha256: prepared.source_sha256,
                source_width: prepared.source_width,
                source_height: prepared.source_height,
                normalized_path: prepared.normalized_path,
                normalized_sha256: prepared.normalized_sha256,
                normalized_width: TARGET_DIMENSION,
                normalized_height: TARGET_DIMENSION,
                output_resref: binding.output_resref.clone(),
                output_tga_sha256: binding.output_sha256.clone(),
            }
        })
        .collect::<Vec<_>>();
    let report = PreparationReport {
        schema_version: 1,
        status: "offline_texture_set_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        separation_sha256: materials.report.separation_sha256.clone(),
        texture_authoring_sha256,
        target_dimension: TARGET_DIMENSION,
        filter: "Lanczos3".to_owned(),
        geometry_cleanup: false,
        source_uv0_preserved: true,
        textures: texture_reports,
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
