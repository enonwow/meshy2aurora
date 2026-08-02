//! Deterministic, source-bound texture authoring for static placeables.
//!
//! Binary image payloads remain outside JSON. A strict descriptor binds every
//! override to exact bytes, while material/source-image identities prevent a
//! recipe from silently moving to a different GLB material after reimport.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::Cursor,
};

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits as ImageLimits};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    glb::{
        EmbeddedImageDecodeLimitsV1, GlbIngestResult, GlbLimits, decode_embedded_image_to_tga_v1,
    },
    mdl::MdlMaterialTextureBindingV1,
    model_ir::AuroraModelIrV1,
    model_material_separation::{default_model_material_separation_v1, resolve_model_materials_v1},
    model_pipeline::resolve_base_color_image_indices_v1,
    model_texture_authoring::{
        ModelTextureAlphaPolicyV1, ModelTextureAuthoringDocumentV1, ModelTextureBindingAuthoringV1,
        ModelTextureBindingModeV1, ModelTexturePayloadDescriptorV1,
        resolve_model_texture_authoring_from_report_v1,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1},
};

pub const PLACEABLE_TEXTURE_AUTHORING_SCHEMA_VERSION_V1: u32 = 1;
pub const PLACEABLE_TEXTURE_PAYLOAD_DESCRIPTOR_SCHEMA_VERSION_V1: u32 = 1;
pub const PLACEABLE_TEXTURE_INSPECTION_SCHEMA_VERSION_V1: u32 = 1;
pub const PLACEABLE_TEXTURE_RESOLUTION_SCHEMA_VERSION_V1: u32 = 1;
pub const PLACEABLE_TEXTURE_TGA_RESOURCE_TYPE_V1: u16 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceableTextureBindingModeV1 {
    Source,
    Override,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceableTextureAlphaPolicyV1 {
    OpaqueOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableTextureBindingAuthoringV1 {
    pub material_slot: u32,
    pub source_material_id: u32,
    pub source_material_name: Option<String>,
    pub source_image_sha256: String,
    pub mode: PlaceableTextureBindingModeV1,
    pub override_asset_id: Option<String>,
    pub override_sha256: Option<String>,
    pub override_mime_type: Option<String>,
    pub override_byte_length: Option<u64>,
    pub alpha_policy: PlaceableTextureAlphaPolicyV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableTextureAuthoringDocumentV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub bindings: Vec<PlaceableTextureBindingAuthoringV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableTexturePayloadDescriptorV1 {
    pub schema_version: u32,
    pub asset_id: String,
    pub sha256: String,
    pub mime_type: String,
    pub byte_offset: u64,
    pub byte_length: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableMaterialTextureInspectionV1 {
    pub material_slot: u32,
    pub source_material_id: u32,
    pub source_material_name: Option<String>,
    pub primitive_ids: Vec<u32>,
    pub source_texture_id: u32,
    pub source_image_id: u32,
    pub source_image_sha256: String,
    pub source_mime_type: String,
    pub source_image_byte_length: u64,
    pub has_uv0: bool,
    pub base_color_factor: [f32; 4],
    pub alpha_mode: String,
    pub alpha_cutoff: Option<f32>,
    pub normal_texture_present: bool,
    pub metallic_roughness_texture_present: bool,
    pub emissive_texture_present: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableTextureInspectionV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub materials: Vec<PlaceableMaterialTextureInspectionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableTextureAuthoringBootstrapV1 {
    pub schema_version: u32,
    pub inspection: PlaceableTextureInspectionV1,
    pub document: PlaceableTextureAuthoringDocumentV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPlaceableTextureBindingV1 {
    pub material_slot: u32,
    pub source_material_id: u32,
    pub source_image_sha256: String,
    pub mode: PlaceableTextureBindingModeV1,
    pub input_sha256: String,
    pub output_resref: String,
    pub output_sha256: String,
    pub source_alpha_mode: String,
    #[serde(default)]
    pub ignored_source_pbr_maps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPlaceableTextureResourceV1 {
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
    pub material_slots: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableTextureResolutionReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub authoring_sha256: String,
    pub alpha_policy: PlaceableTextureAlphaPolicyV1,
    pub bindings: Vec<ResolvedPlaceableTextureBindingV1>,
    pub resources: Vec<ResolvedPlaceableTextureResourceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPlaceableTexturePayloadV1 {
    pub resref: String,
    pub resource_type: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPlaceableTextureSetV1 {
    pub report: PlaceableTextureResolutionReportV1,
    pub material_textures: Vec<MdlMaterialTextureBindingV1>,
    pub textures: Vec<ResolvedPlaceableTexturePayloadV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableTextureErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for PlaceableTextureErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for PlaceableTextureErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> PlaceableTextureErrorV1 {
    PlaceableTextureErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

pub fn placeable_texture_authoring_hash_v1(
    document: &PlaceableTextureAuthoringDocumentV1,
) -> Result<String, PlaceableTextureErrorV1> {
    let mut canonical = document.clone();
    canonical
        .bindings
        .sort_by_key(|binding| binding.material_slot);
    serde_json::to_vec(&canonical)
        .map(|payload| sha256(&payload))
        .map_err(|source| {
            error(
                "PLACEABLE-TEXTURE-AUTHORING-SERIALIZE",
                "textureAuthoring",
                source.to_string(),
            )
        })
}

pub fn inspect_placeable_material_textures_v1(
    ingest: &GlbIngestResult,
    model: &AuroraModelIrV1,
) -> Result<PlaceableTextureAuthoringBootstrapV1, PlaceableTextureErrorV1> {
    let selections = resolve_base_color_image_indices_v1(ingest, model).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let mut materials = Vec::with_capacity(selections.len());
    for selection in selections {
        let material = ingest
            .ir
            .materials
            .iter()
            .find(|material| material.id == selection.source_material_id)
            .ok_or_else(|| {
                error(
                    "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                    format!("materials[{}]", selection.source_material_id),
                    "resolved source material is absent",
                )
            })?;
        let image = ingest
            .ir
            .images
            .iter()
            .find(|image| image.id == selection.source_image_id)
            .ok_or_else(|| {
                error(
                    "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                    format!("images[{}]", selection.source_image_id),
                    "resolved source image is absent",
                )
            })?;
        let mut primitives = ingest
            .ir
            .primitives
            .iter()
            .filter(|primitive| primitive.material_id == Some(selection.source_material_id))
            .collect::<Vec<_>>();
        primitives.sort_by_key(|primitive| primitive.id);
        let has_uv0 = !primitives.is_empty()
            && primitives.iter().all(|primitive| {
                !primitive.uv0.is_empty() && primitive.uv0.len() == primitive.positions.len()
            });
        materials.push(PlaceableMaterialTextureInspectionV1 {
            material_slot: selection.material_slot,
            source_material_id: selection.source_material_id,
            source_material_name: material.name.clone(),
            primitive_ids: primitives.iter().map(|primitive| primitive.id).collect(),
            source_texture_id: selection.source_texture_id,
            source_image_id: selection.source_image_id,
            source_image_sha256: selection.source_image_sha256,
            source_mime_type: image.mime_type.clone(),
            source_image_byte_length: image.byte_length as u64,
            has_uv0,
            base_color_factor: material.base_color_factor,
            alpha_mode: material.alpha_mode.clone(),
            alpha_cutoff: material.alpha_cutoff,
            normal_texture_present: material.normal_texture.is_some(),
            metallic_roughness_texture_present: material.metallic_roughness_texture.is_some(),
            emissive_texture_present: material.emissive_texture.is_some(),
        });
    }
    materials.sort_by_key(|material| material.material_slot);
    let document = PlaceableTextureAuthoringDocumentV1 {
        schema_version: PLACEABLE_TEXTURE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: ingest.ir.source.sha256.clone(),
        bindings: materials
            .iter()
            .map(|material| PlaceableTextureBindingAuthoringV1 {
                material_slot: material.material_slot,
                source_material_id: material.source_material_id,
                source_material_name: material.source_material_name.clone(),
                source_image_sha256: material.source_image_sha256.clone(),
                mode: PlaceableTextureBindingModeV1::Source,
                override_asset_id: None,
                override_sha256: None,
                override_mime_type: None,
                override_byte_length: None,
                alpha_policy: PlaceableTextureAlphaPolicyV1::OpaqueOnly,
            })
            .collect(),
    };
    Ok(PlaceableTextureAuthoringBootstrapV1 {
        schema_version: PLACEABLE_TEXTURE_AUTHORING_SCHEMA_VERSION_V1,
        inspection: PlaceableTextureInspectionV1 {
            schema_version: PLACEABLE_TEXTURE_INSPECTION_SCHEMA_VERSION_V1,
            source_sha256: ingest.ir.source.sha256.clone(),
            materials,
        },
        document,
    })
}

fn validate_resref(value: &str) -> Result<(), PlaceableTextureErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == b'_')
    {
        return Err(error(
            "PLACEABLE-TEXTURE-RESREF-INVALID",
            "baseTextureResref",
            "texture resref must be 1..=16 lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

fn derived_resref(base: &str, index: usize) -> Result<String, PlaceableTextureErrorV1> {
    if index == 0 {
        return Ok(base.to_owned());
    }
    let suffix = format!("_m{index}");
    let prefix_len = 16usize.checked_sub(suffix.len()).ok_or_else(|| {
        error(
            "PLACEABLE-TEXTURE-RESREF-EXHAUSTED",
            "baseTextureResref",
            "material texture suffix cannot fit the Aurora resref limit",
        )
    })?;
    if prefix_len == 0 {
        return Err(error(
            "PLACEABLE-TEXTURE-RESREF-EXHAUSTED",
            "baseTextureResref",
            "material texture suffix leaves no base resref bytes",
        ));
    }
    Ok(format!("{}{}", &base[..base.len().min(prefix_len)], suffix))
}

fn descriptor_payload<'a>(
    descriptor: &PlaceableTexturePayloadDescriptorV1,
    payload_blob: &'a [u8],
) -> Result<&'a [u8], PlaceableTextureErrorV1> {
    if descriptor.schema_version != PLACEABLE_TEXTURE_PAYLOAD_DESCRIPTOR_SCHEMA_VERSION_V1 {
        return Err(error(
            "PLACEABLE-TEXTURE-DESCRIPTOR-SCHEMA-INVALID",
            "texturePayloadDescriptors.schemaVersion",
            "descriptor schemaVersion must be 1",
        ));
    }
    if descriptor.asset_id.is_empty() || descriptor.asset_id.len() > 128 {
        return Err(error(
            "PLACEABLE-TEXTURE-ASSET-ID-INVALID",
            "texturePayloadDescriptors.assetId",
            "assetId must contain 1..=128 characters",
        ));
    }
    if !valid_sha256(&descriptor.sha256) {
        return Err(error(
            "PLACEABLE-TEXTURE-SHA256-INVALID",
            "texturePayloadDescriptors.sha256",
            "descriptor SHA-256 must be lowercase hexadecimal",
        ));
    }
    if !matches!(descriptor.mime_type.as_str(), "image/png" | "image/jpeg") {
        return Err(error(
            "PLACEABLE-TEXTURE-MIME-UNSUPPORTED",
            "texturePayloadDescriptors.mimeType",
            "override image must be image/png or image/jpeg",
        ));
    }
    let start = usize::try_from(descriptor.byte_offset).map_err(|_| {
        error(
            "PLACEABLE-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors.byteOffset",
            "payload offset does not fit this platform",
        )
    })?;
    let length = usize::try_from(descriptor.byte_length).map_err(|_| {
        error(
            "PLACEABLE-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors.byteLength",
            "payload length does not fit this platform",
        )
    })?;
    if descriptor.byte_length > EmbeddedImageDecodeLimitsV1::default().max_decoded_bytes {
        return Err(error(
            "PLACEABLE-TEXTURE-PAYLOAD-LIMIT-EXCEEDED",
            "texturePayloadDescriptors.byteLength",
            "encoded override payload exceeds the V1 per-image byte limit",
        ));
    }
    let end = start.checked_add(length).ok_or_else(|| {
        error(
            "PLACEABLE-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors",
            "payload range overflows",
        )
    })?;
    let payload = payload_blob.get(start..end).ok_or_else(|| {
        error(
            "PLACEABLE-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors",
            "payload range escapes the supplied blob",
        )
    })?;
    if sha256(payload) != descriptor.sha256 {
        return Err(error(
            "PLACEABLE-TEXTURE-PAYLOAD-HASH-MISMATCH",
            "texturePayloadDescriptors.sha256",
            "payload bytes do not match the descriptor SHA-256",
        ));
    }
    Ok(payload)
}

fn decode_override_image_v1(
    payload: &[u8],
    mime_type: &str,
) -> Result<TgaImageV1, PlaceableTextureErrorV1> {
    let format = match mime_type {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        _ => {
            return Err(error(
                "PLACEABLE-TEXTURE-MIME-UNSUPPORTED",
                "textureOverride.mimeType",
                "override image must be image/png or image/jpeg",
            ));
        }
    };
    let signature_matches = match format {
        ImageFormat::Png => payload.starts_with(b"\x89PNG\r\n\x1a\n"),
        ImageFormat::Jpeg => payload.starts_with(&[0xff, 0xd8, 0xff]),
        _ => false,
    };
    if !signature_matches {
        return Err(error(
            "PLACEABLE-TEXTURE-MIME-MISMATCH",
            "textureOverride.mimeType",
            "declared MIME does not match image bytes",
        ));
    }
    let limits = EmbeddedImageDecodeLimitsV1::default();
    let mut decoder_limits = ImageLimits::default();
    decoder_limits.max_image_width = Some(limits.max_width);
    decoder_limits.max_image_height = Some(limits.max_height);
    decoder_limits.max_alloc = Some(limits.max_decoded_bytes.saturating_mul(2));
    let mut reader = ImageReader::with_format(Cursor::new(payload), format);
    reader.limits(decoder_limits);
    let decoder = reader.into_decoder().map_err(|source| {
        error(
            "PLACEABLE-TEXTURE-DECODE-FAILED",
            "textureOverride",
            source.to_string(),
        )
    })?;
    let (width, height) = decoder.dimensions();
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| {
            error(
                "PLACEABLE-TEXTURE-DECODE-LIMIT-EXCEEDED",
                "textureOverride",
                "decoded pixel count overflows",
            )
        })?;
    if pixels > limits.max_pixels {
        return Err(error(
            "PLACEABLE-TEXTURE-DECODE-LIMIT-EXCEEDED",
            "textureOverride",
            format!(
                "decoded image has {pixels} pixels but the limit is {}",
                limits.max_pixels
            ),
        ));
    }
    let decoded = DynamicImage::from_decoder(decoder).map_err(|source| {
        error(
            "PLACEABLE-TEXTURE-DECODE-FAILED",
            "textureOverride",
            source.to_string(),
        )
    })?;
    let rgba = decoded.into_rgba8();
    if rgba.pixels().any(|pixel| pixel.0[3] != u8::MAX) {
        return Err(error(
            "PLACEABLE-TEXTURE-ALPHA-UNSUPPORTED",
            "textureOverride.alpha",
            "V1 OPAQUE_ONLY rejects pixels whose alpha is not 255",
        ));
    }
    let mut rgb = Vec::with_capacity(usize::try_from(pixels.saturating_mul(3)).unwrap_or(0));
    for pixel in rgba.pixels() {
        rgb.extend_from_slice(&pixel.0[..3]);
    }
    if u64::try_from(rgb.len()).ok() != Some(pixels.saturating_mul(3))
        || u64::try_from(rgb.len()).unwrap_or(u64::MAX) > limits.max_decoded_bytes
    {
        return Err(error(
            "PLACEABLE-TEXTURE-DECODE-LIMIT-EXCEEDED",
            "textureOverride",
            "decoded RGB payload exceeds the V1 output limit",
        ));
    }
    Ok(TgaImageV1 {
        schema_version: 1,
        width,
        height,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: rgb,
    })
}

pub fn resolve_placeable_texture_overrides_v1(
    source_glb: &[u8],
    glb_limits: &GlbLimits,
    ingest: &GlbIngestResult,
    model: &AuroraModelIrV1,
    base_texture_resref: &str,
    authoring: &PlaceableTextureAuthoringDocumentV1,
    payload_blob: &[u8],
    descriptors: &[PlaceableTexturePayloadDescriptorV1],
) -> Result<ResolvedPlaceableTextureSetV1, PlaceableTextureErrorV1> {
    let identity = default_model_material_separation_v1(&ingest.ir);
    let materials = resolve_model_materials_v1(&ingest.ir, &identity).map_err(|source| {
        error(
            &source
                .code
                .replacen("MATERIAL-SEPARATION", "PLACEABLE-TEXTURE", 1),
            source.path,
            source.message,
        )
    })?;
    let mut material_report = materials.report.clone();
    material_report.material_slots = model
        .material_source_bindings
        .iter()
        .map(|binding| {
            let source_material_id = binding.source_material_id.ok_or_else(|| {
                error(
                    "PLACEABLE-TEXTURE-BASE-COLOR-MATERIAL-MISSING",
                    "model.materialSourceBindings.sourceMaterialId",
                    "legacy Placeable texture authoring requires an explicit source material",
                )
            })?;
            let mut slot = materials
                .report
                .material_slots
                .iter()
                .find(|slot| slot.source_material_id == Some(source_material_id))
                .cloned()
                .ok_or_else(|| {
                    error(
                        "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                        "model.materialSourceBindings",
                        "retained model material is absent from the exact source inventory",
                    )
                })?;
            slot.material_slot = binding.slot;
            Ok(slot)
        })
        .collect::<Result<Vec<_>, PlaceableTextureErrorV1>>()?;
    let bindings = material_report
        .material_slots
        .iter()
        .map(|slot| {
            let source_material_id = slot.source_material_id.ok_or_else(|| {
                error(
                    "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                    "materialReport.materialSlots.sourceMaterialId",
                    "legacy Placeable material slot has no explicit source material",
                )
            })?;
            let binding = authoring
                .bindings
                .iter()
                .find(|binding| binding.source_material_id == source_material_id)
                .ok_or_else(|| {
                    error(
                        "PLACEABLE-TEXTURE-BINDING-MISSING",
                        "textureAuthoring.bindings",
                        "retained source material has no legacy texture binding",
                    )
                })?;
            if slot.source_image_sha256.as_deref() != Some(binding.source_image_sha256.as_str()) {
                return Err(error(
                    "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                    "textureAuthoring.bindings",
                    "legacy Placeable binding no longer matches the neutral material slot",
                ));
            }
            Ok(ModelTextureBindingAuthoringV1 {
                authored_material_id: slot.authored_material_id.clone(),
                material_slot: slot.material_slot,
                source_material_id: Some(binding.source_material_id),
                source_image_sha256: Some(binding.source_image_sha256.clone()),
                mode: match binding.mode {
                    PlaceableTextureBindingModeV1::Source => ModelTextureBindingModeV1::Source,
                    PlaceableTextureBindingModeV1::Override => ModelTextureBindingModeV1::Override,
                },
                override_asset_id: binding.override_asset_id.clone(),
                override_sha256: binding.override_sha256.clone(),
                override_mime_type: binding.override_mime_type.clone(),
                override_byte_length: binding.override_byte_length,
                alpha_policy: ModelTextureAlphaPolicyV1::OpaqueOnly,
            })
        })
        .collect::<Result<Vec<_>, PlaceableTextureErrorV1>>()?;
    let neutral_authoring = ModelTextureAuthoringDocumentV1 {
        schema_version: 1,
        source_sha256: authoring.source_sha256.clone(),
        separation_sha256: material_report.separation_sha256.clone(),
        bindings,
    };
    let neutral_descriptors = descriptors
        .iter()
        .map(|descriptor| ModelTexturePayloadDescriptorV1 {
            schema_version: descriptor.schema_version,
            asset_id: descriptor.asset_id.clone(),
            sha256: descriptor.sha256.clone(),
            mime_type: descriptor.mime_type.clone(),
            byte_offset: descriptor.byte_offset,
            byte_length: descriptor.byte_length,
        })
        .collect::<Vec<_>>();
    let resolved = resolve_model_texture_authoring_from_report_v1(
        source_glb,
        glb_limits,
        ingest,
        &material_report,
        base_texture_resref,
        &neutral_authoring,
        payload_blob,
        &neutral_descriptors,
    )
    .map_err(|source| {
        error(
            &source
                .code
                .replacen("MODEL-TEXTURE", "PLACEABLE-TEXTURE", 1),
            source.path,
            source.message,
        )
    })?;
    Ok(ResolvedPlaceableTextureSetV1 {
        report: PlaceableTextureResolutionReportV1 {
            schema_version: PLACEABLE_TEXTURE_RESOLUTION_SCHEMA_VERSION_V1,
            source_sha256: resolved.report.source_sha256,
            authoring_sha256: placeable_texture_authoring_hash_v1(authoring)?,
            alpha_policy: PlaceableTextureAlphaPolicyV1::OpaqueOnly,
            bindings: resolved
                .report
                .bindings
                .into_iter()
                .map(|binding| ResolvedPlaceableTextureBindingV1 {
                    material_slot: binding.material_slot,
                    source_material_id: binding
                        .source_material_id
                        .expect("legacy Placeable adapter validated an explicit source material"),
                    source_image_sha256: binding
                        .source_image_sha256
                        .expect("legacy Placeable adapter validated a source image"),
                    mode: match binding.mode {
                        ModelTextureBindingModeV1::Source => PlaceableTextureBindingModeV1::Source,
                        ModelTextureBindingModeV1::Override => {
                            PlaceableTextureBindingModeV1::Override
                        }
                    },
                    input_sha256: binding.input_sha256,
                    output_resref: binding.output_resref,
                    output_sha256: binding.output_sha256,
                    source_alpha_mode: binding.source_alpha_mode,
                    ignored_source_pbr_maps: binding.ignored_source_pbr_maps,
                })
                .collect(),
            resources: resolved
                .report
                .resources
                .into_iter()
                .map(|resource| ResolvedPlaceableTextureResourceV1 {
                    resref: resource.resref,
                    resource_type: resource.resource_type,
                    byte_length: resource.byte_length,
                    sha256: resource.sha256,
                    material_slots: resource.material_slots,
                })
                .collect(),
        },
        material_textures: resolved.material_textures,
        textures: resolved
            .textures
            .into_iter()
            .map(|texture| ResolvedPlaceableTexturePayloadV1 {
                resref: texture.resref,
                resource_type: texture.resource_type,
                payload: texture.payload,
            })
            .collect(),
    })
}

#[allow(dead_code)]
fn resolve_placeable_texture_overrides_legacy_v1(
    source_glb: &[u8],
    glb_limits: &GlbLimits,
    ingest: &GlbIngestResult,
    model: &AuroraModelIrV1,
    base_texture_resref: &str,
    authoring: &PlaceableTextureAuthoringDocumentV1,
    payload_blob: &[u8],
    descriptors: &[PlaceableTexturePayloadDescriptorV1],
) -> Result<ResolvedPlaceableTextureSetV1, PlaceableTextureErrorV1> {
    validate_resref(base_texture_resref)?;
    if authoring.schema_version != PLACEABLE_TEXTURE_AUTHORING_SCHEMA_VERSION_V1 {
        return Err(error(
            "PLACEABLE-TEXTURE-AUTHORING-SCHEMA-INVALID",
            "textureAuthoring.schemaVersion",
            "texture authoring schemaVersion must be 1",
        ));
    }
    if authoring.source_sha256 != ingest.ir.source.sha256 {
        return Err(error(
            "PLACEABLE-TEXTURE-SOURCE-MISMATCH",
            "textureAuthoring.sourceSha256",
            "texture authoring must be bound to the exact source GLB",
        ));
    }
    if authoring.bindings.len() > glb_limits.max_materials {
        return Err(error(
            "PLACEABLE-TEXTURE-BINDING-LIMIT-EXCEEDED",
            "textureAuthoring.bindings",
            "texture binding count exceeds the GLB material limit",
        ));
    }
    let bootstrap = inspect_placeable_material_textures_v1(ingest, model)?;
    if authoring.bindings.len() < bootstrap.document.bindings.len() {
        return Err(error(
            "PLACEABLE-TEXTURE-BINDING-COUNT-MISMATCH",
            "textureAuthoring.bindings",
            "texture authoring must contain a binding for every retained material slot",
        ));
    }
    let mut descriptor_by_id = BTreeMap::new();
    let mut descriptor_ranges = Vec::with_capacity(descriptors.len());
    for descriptor in descriptors {
        if descriptor_by_id
            .insert(descriptor.asset_id.clone(), descriptor)
            .is_some()
        {
            return Err(error(
                "PLACEABLE-TEXTURE-DESCRIPTOR-DUPLICATE",
                "texturePayloadDescriptors",
                "descriptor assetId must be unique",
            ));
        }
        descriptor_payload(descriptor, payload_blob)?;
        descriptor_ranges.push((descriptor.byte_offset, descriptor.byte_length));
    }
    descriptor_ranges.sort_unstable();
    let mut described_bytes = 0_u64;
    for (offset, length) in descriptor_ranges {
        if offset != described_bytes {
            return Err(error(
                "PLACEABLE-TEXTURE-PAYLOAD-ENVELOPE-INVALID",
                "texturePayloadDescriptors",
                "payload descriptors must cover the blob exactly without gaps or overlaps",
            ));
        }
        described_bytes = described_bytes.checked_add(length).ok_or_else(|| {
            error(
                "PLACEABLE-TEXTURE-PAYLOAD-ENVELOPE-INVALID",
                "texturePayloadDescriptors",
                "payload descriptor envelope overflows",
            )
        })?;
    }
    if described_bytes != payload_blob.len() as u64 {
        return Err(error(
            "PLACEABLE-TEXTURE-PAYLOAD-ENVELOPE-INVALID",
            "texturePayloadDescriptors",
            "payload descriptors must cover every supplied byte exactly",
        ));
    }
    let mut authoring_by_slot = BTreeMap::new();
    let mut referenced_assets = BTreeSet::new();
    for binding in &authoring.bindings {
        if authoring_by_slot
            .insert(binding.material_slot, binding)
            .is_some()
        {
            return Err(error(
                "PLACEABLE-TEXTURE-BINDING-DUPLICATE",
                "textureAuthoring.bindings",
                "materialSlot must be unique",
            ));
        }
        match binding.mode {
            PlaceableTextureBindingModeV1::Source => {
                if binding.override_asset_id.is_some()
                    || binding.override_sha256.is_some()
                    || binding.override_mime_type.is_some()
                    || binding.override_byte_length.is_some()
                {
                    return Err(error(
                        "PLACEABLE-TEXTURE-SOURCE-BINDING-INVALID",
                        "textureAuthoring.bindings",
                        "SOURCE binding must not contain override metadata",
                    ));
                }
            }
            PlaceableTextureBindingModeV1::Override => {
                let asset_id = binding.override_asset_id.as_ref().ok_or_else(|| {
                    error(
                        "PLACEABLE-TEXTURE-OVERRIDE-INCOMPLETE",
                        "textureAuthoring.bindings.overrideAssetId",
                        "OVERRIDE binding requires overrideAssetId",
                    )
                })?;
                let descriptor = descriptor_by_id.get(asset_id).ok_or_else(|| {
                    error(
                        "PLACEABLE-TEXTURE-PAYLOAD-MISSING",
                        "texturePayloadDescriptors",
                        format!("override asset {asset_id} has no payload descriptor"),
                    )
                })?;
                if binding.override_sha256.as_deref() != Some(descriptor.sha256.as_str())
                    || binding.override_mime_type.as_deref() != Some(descriptor.mime_type.as_str())
                    || binding.override_byte_length != Some(descriptor.byte_length)
                {
                    return Err(error(
                        "PLACEABLE-TEXTURE-OVERRIDE-DESCRIPTOR-MISMATCH",
                        "textureAuthoring.bindings",
                        "override metadata does not match its payload descriptor",
                    ));
                }
                referenced_assets.insert(asset_id.clone());
            }
        }
    }
    if referenced_assets.len() != descriptor_by_id.len() {
        return Err(error(
            "PLACEABLE-TEXTURE-PAYLOAD-UNUSED",
            "texturePayloadDescriptors",
            "every supplied override payload must be referenced exactly by the recipe",
        ));
    }

    let selections = resolve_base_color_image_indices_v1(ingest, model).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let mut material_textures = Vec::with_capacity(selections.len());
    let mut textures = Vec::<ResolvedPlaceableTexturePayloadV1>::new();
    let mut resource_index_by_output_sha = BTreeMap::<String, usize>::new();
    let mut binding_reports = Vec::with_capacity(selections.len());
    let mut resource_slots = Vec::<Vec<u32>>::new();

    for selection in selections {
        let material_inspection = bootstrap
            .inspection
            .materials
            .iter()
            .find(|material| material.material_slot == selection.material_slot)
            .ok_or_else(|| {
                error(
                    "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                    "textureInspection.materials",
                    "retained material inspection is missing",
                )
            })?;
        let binding = authoring_by_slot
            .get(&selection.material_slot)
            .ok_or_else(|| {
                error(
                    "PLACEABLE-TEXTURE-BINDING-MISSING",
                    "textureAuthoring.bindings",
                    format!("material slot {} is missing", selection.material_slot),
                )
            })?;
        if binding.source_material_id != selection.source_material_id
            || binding.source_image_sha256 != selection.source_image_sha256
        {
            return Err(error(
                "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE",
                format!(
                    "textureAuthoring.bindings[materialSlot={}]",
                    selection.material_slot
                ),
                "source material or source image identity no longer matches the GLB",
            ));
        }
        if binding.alpha_policy != PlaceableTextureAlphaPolicyV1::OpaqueOnly {
            return Err(error(
                "PLACEABLE-TEXTURE-ALPHA-POLICY-UNSUPPORTED",
                "textureAuthoring.bindings.alphaPolicy",
                "V1 supports only OPAQUE_ONLY",
            ));
        }
        if binding.mode == PlaceableTextureBindingModeV1::Override
            && material_inspection.alpha_mode != "OPAQUE"
        {
            return Err(error(
                "PLACEABLE-TEXTURE-ALPHA-MODE-UNSUPPORTED",
                format!(
                    "textureAuthoring.bindings[materialSlot={}].alphaPolicy",
                    selection.material_slot
                ),
                "V1 override requires an OPAQUE source material; MASK/BLEND require a future TXI/MTR contract",
            ));
        }
        let ignored_source_pbr_maps = if binding.mode == PlaceableTextureBindingModeV1::Override {
            [
                (material_inspection.normal_texture_present, "normalTexture"),
                (
                    material_inspection.metallic_roughness_texture_present,
                    "metallicRoughnessTexture",
                ),
                (
                    material_inspection.emissive_texture_present,
                    "emissiveTexture",
                ),
            ]
            .into_iter()
            .filter_map(|(present, name)| present.then_some(name.to_owned()))
            .collect()
        } else {
            Vec::new()
        };
        let (input_sha256, image) = match binding.mode {
            PlaceableTextureBindingModeV1::Source => {
                if binding.override_asset_id.is_some()
                    || binding.override_sha256.is_some()
                    || binding.override_mime_type.is_some()
                    || binding.override_byte_length.is_some()
                {
                    return Err(error(
                        "PLACEABLE-TEXTURE-SOURCE-BINDING-INVALID",
                        "textureAuthoring.bindings",
                        "SOURCE binding must not contain override metadata",
                    ));
                }
                let image = decode_embedded_image_to_tga_v1(
                    source_glb,
                    selection.source_image_index,
                    glb_limits,
                    &EmbeddedImageDecodeLimitsV1::default(),
                )
                .map_err(|source| {
                    error(
                        &format!("PLACEABLE-{}", source.code),
                        source.json_path.unwrap_or_else(|| "images".to_owned()),
                        source.message,
                    )
                })?;
                (selection.source_image_sha256.clone(), image)
            }
            PlaceableTextureBindingModeV1::Override => {
                let asset_id = binding.override_asset_id.as_ref().ok_or_else(|| {
                    error(
                        "PLACEABLE-TEXTURE-OVERRIDE-INCOMPLETE",
                        "textureAuthoring.bindings.overrideAssetId",
                        "OVERRIDE binding requires overrideAssetId",
                    )
                })?;
                let descriptor = descriptor_by_id.get(asset_id).ok_or_else(|| {
                    error(
                        "PLACEABLE-TEXTURE-PAYLOAD-MISSING",
                        "texturePayloadDescriptors",
                        format!("override asset {asset_id} has no payload descriptor"),
                    )
                })?;
                if binding.override_sha256.as_deref() != Some(descriptor.sha256.as_str())
                    || binding.override_mime_type.as_deref() != Some(descriptor.mime_type.as_str())
                    || binding.override_byte_length != Some(descriptor.byte_length)
                {
                    return Err(error(
                        "PLACEABLE-TEXTURE-OVERRIDE-DESCRIPTOR-MISMATCH",
                        "textureAuthoring.bindings",
                        "override metadata does not match its payload descriptor",
                    ));
                }
                let payload = descriptor_payload(descriptor, payload_blob)?;
                (
                    descriptor.sha256.clone(),
                    decode_override_image_v1(payload, &descriptor.mime_type)?,
                )
            }
        };
        let tga = write_tga_v1(&image, &TgaWriterOptionsV1::default()).map_err(|source| {
            error(
                &format!("PLACEABLE-{}", source.code),
                source.path,
                source.message,
            )
        })?;
        let output_sha256 = tga.report.output_sha256.clone();
        let resource_index = if let Some(index) = resource_index_by_output_sha.get(&output_sha256) {
            *index
        } else {
            let index = textures.len();
            let resref = derived_resref(base_texture_resref, index)?;
            textures.push(ResolvedPlaceableTexturePayloadV1 {
                resref,
                resource_type: PLACEABLE_TEXTURE_TGA_RESOURCE_TYPE_V1,
                payload: tga.payload,
            });
            resource_slots.push(Vec::new());
            resource_index_by_output_sha.insert(output_sha256.clone(), index);
            index
        };
        resource_slots[resource_index].push(selection.material_slot);
        let output_resref = textures[resource_index].resref.clone();
        material_textures.push(MdlMaterialTextureBindingV1 {
            material_slot: selection.material_slot,
            resref: output_resref.clone(),
        });
        binding_reports.push(ResolvedPlaceableTextureBindingV1 {
            material_slot: selection.material_slot,
            source_material_id: selection.source_material_id,
            source_image_sha256: selection.source_image_sha256,
            mode: binding.mode,
            input_sha256,
            output_resref,
            output_sha256,
            source_alpha_mode: material_inspection.alpha_mode.clone(),
            ignored_source_pbr_maps,
        });
    }
    let resources = textures
        .iter()
        .zip(resource_slots)
        .map(
            |(texture, material_slots)| ResolvedPlaceableTextureResourceV1 {
                resref: texture.resref.clone(),
                resource_type: texture.resource_type,
                byte_length: texture.payload.len() as u64,
                sha256: sha256(&texture.payload),
                material_slots,
            },
        )
        .collect();
    Ok(ResolvedPlaceableTextureSetV1 {
        report: PlaceableTextureResolutionReportV1 {
            schema_version: PLACEABLE_TEXTURE_RESOLUTION_SCHEMA_VERSION_V1,
            source_sha256: ingest.ir.source.sha256.clone(),
            authoring_sha256: placeable_texture_authoring_hash_v1(authoring)?,
            alpha_policy: PlaceableTextureAlphaPolicyV1::OpaqueOnly,
            bindings: binding_reports,
            resources,
        },
        material_textures,
        textures,
    })
}
