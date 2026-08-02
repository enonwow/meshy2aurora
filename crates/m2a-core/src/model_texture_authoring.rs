//! Target-neutral texture authoring for resolved material slots.
//!
//! The source GLB and Material Separation recipe remain immutable. Each
//! authored slot independently selects its exact source fallback or an exact
//! external PNG/JPEG payload. Output TGA resources are deduplicated only when
//! their complete encoded bytes have the same SHA-256.

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
    model_material_separation::{ModelMaterialSeparationReportV1, ResolvedModelMaterialsV1},
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1},
};

pub const MODEL_TEXTURE_AUTHORING_SCHEMA_VERSION_V1: u32 = 1;
pub const MODEL_TEXTURE_PAYLOAD_DESCRIPTOR_SCHEMA_VERSION_V1: u32 = 1;
pub const MODEL_TEXTURE_RESOLUTION_SCHEMA_VERSION_V1: u32 = 1;
pub const MODEL_TEXTURE_TGA_RESOURCE_TYPE_V1: u16 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelTextureBindingModeV1 {
    Source,
    Override,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelTextureAlphaPolicyV1 {
    OpaqueOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelTextureBindingAuthoringV1 {
    pub authored_material_id: String,
    pub material_slot: u32,
    pub source_material_id: Option<u32>,
    pub source_image_sha256: Option<String>,
    pub mode: ModelTextureBindingModeV1,
    pub override_asset_id: Option<String>,
    pub override_sha256: Option<String>,
    pub override_mime_type: Option<String>,
    pub override_byte_length: Option<u64>,
    pub alpha_policy: ModelTextureAlphaPolicyV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelTextureAuthoringDocumentV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub separation_sha256: String,
    pub bindings: Vec<ModelTextureBindingAuthoringV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelTexturePayloadDescriptorV1 {
    pub schema_version: u32,
    pub asset_id: String,
    pub sha256: String,
    pub mime_type: String,
    pub byte_offset: u64,
    pub byte_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedModelTextureBindingV1 {
    pub authored_material_id: String,
    pub material_slot: u32,
    pub source_material_id: Option<u32>,
    pub source_image_sha256: Option<String>,
    pub mode: ModelTextureBindingModeV1,
    pub input_sha256: String,
    pub output_resref: String,
    pub output_sha256: String,
    pub source_alpha_mode: String,
    pub ignored_source_pbr_maps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedModelTextureResourceV1 {
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
    pub material_slots: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelTextureResolutionReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub separation_sha256: String,
    pub authoring_sha256: String,
    pub alpha_policy: ModelTextureAlphaPolicyV1,
    pub uv_policy: String,
    pub bindings: Vec<ResolvedModelTextureBindingV1>,
    pub resources: Vec<ResolvedModelTextureResourceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedModelTexturePayloadV1 {
    pub resref: String,
    pub resource_type: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedModelTextureSetV1 {
    pub report: ModelTextureResolutionReportV1,
    pub material_textures: Vec<MdlMaterialTextureBindingV1>,
    pub textures: Vec<ResolvedModelTexturePayloadV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelTextureErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelTextureErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelTextureErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> ModelTextureErrorV1 {
    ModelTextureErrorV1 {
        schema_version: MODEL_TEXTURE_AUTHORING_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
}

pub fn model_texture_authoring_hash_v1(
    document: &ModelTextureAuthoringDocumentV1,
) -> Result<String, ModelTextureErrorV1> {
    let mut canonical = document.clone();
    canonical.bindings.sort_by(|left, right| {
        left.material_slot
            .cmp(&right.material_slot)
            .then_with(|| left.authored_material_id.cmp(&right.authored_material_id))
    });
    serde_json::to_vec(&canonical)
        .map(|payload| sha256(&payload))
        .map_err(|source| {
            error(
                "MODEL-TEXTURE-AUTHORING-SERIALIZE",
                "textureAuthoring",
                source.to_string(),
            )
        })
}

pub fn default_model_texture_authoring_v1(
    ingest: &GlbIngestResult,
    materials: &ResolvedModelMaterialsV1,
) -> Result<ModelTextureAuthoringDocumentV1, ModelTextureErrorV1> {
    default_model_texture_authoring_from_report_v1(ingest, &materials.report)
}

pub(crate) fn default_model_texture_authoring_from_report_v1(
    ingest: &GlbIngestResult,
    materials: &ModelMaterialSeparationReportV1,
) -> Result<ModelTextureAuthoringDocumentV1, ModelTextureErrorV1> {
    if materials.source_sha256 != ingest.ir.source.sha256 {
        return Err(error(
            "MODEL-TEXTURE-SOURCE-MISMATCH",
            "materials.sourceSha256",
            "resolved materials belong to a different source GLB",
        ));
    }
    Ok(ModelTextureAuthoringDocumentV1 {
        schema_version: MODEL_TEXTURE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: ingest.ir.source.sha256.clone(),
        separation_sha256: materials.separation_sha256.clone(),
        bindings: materials
            .material_slots
            .iter()
            .map(|slot| ModelTextureBindingAuthoringV1 {
                authored_material_id: slot.authored_material_id.clone(),
                material_slot: slot.material_slot,
                source_material_id: slot.source_material_id,
                source_image_sha256: slot.source_image_sha256.clone(),
                mode: ModelTextureBindingModeV1::Source,
                override_asset_id: None,
                override_sha256: None,
                override_mime_type: None,
                override_byte_length: None,
                alpha_policy: ModelTextureAlphaPolicyV1::OpaqueOnly,
            })
            .collect(),
    })
}

fn validate_resref(value: &str) -> Result<(), ModelTextureErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == b'_')
    {
        return Err(error(
            "MODEL-TEXTURE-RESREF-INVALID",
            "baseTextureResref",
            "texture resref must be 1..=16 lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

fn derived_resref(base: &str, index: usize) -> Result<String, ModelTextureErrorV1> {
    if index == 0 {
        return Ok(base.to_owned());
    }
    let suffix = format!("_m{index}");
    let prefix_len = 16usize.checked_sub(suffix.len()).ok_or_else(|| {
        error(
            "MODEL-TEXTURE-RESREF-EXHAUSTED",
            "baseTextureResref",
            "material texture suffix cannot fit the Aurora resref limit",
        )
    })?;
    if prefix_len == 0 {
        return Err(error(
            "MODEL-TEXTURE-RESREF-EXHAUSTED",
            "baseTextureResref",
            "material texture suffix leaves no base resref bytes",
        ));
    }
    Ok(format!("{}{}", &base[..base.len().min(prefix_len)], suffix))
}

fn descriptor_payload<'a>(
    descriptor: &ModelTexturePayloadDescriptorV1,
    payload_blob: &'a [u8],
) -> Result<&'a [u8], ModelTextureErrorV1> {
    if descriptor.schema_version != MODEL_TEXTURE_PAYLOAD_DESCRIPTOR_SCHEMA_VERSION_V1 {
        return Err(error(
            "MODEL-TEXTURE-DESCRIPTOR-SCHEMA-INVALID",
            "texturePayloadDescriptors.schemaVersion",
            "descriptor schemaVersion must be 1",
        ));
    }
    if descriptor.asset_id.is_empty() || descriptor.asset_id.len() > 128 {
        return Err(error(
            "MODEL-TEXTURE-ASSET-ID-INVALID",
            "texturePayloadDescriptors.assetId",
            "assetId must contain 1..=128 characters",
        ));
    }
    if !valid_sha256(&descriptor.sha256) {
        return Err(error(
            "MODEL-TEXTURE-SHA256-INVALID",
            "texturePayloadDescriptors.sha256",
            "descriptor SHA-256 must be lowercase hexadecimal",
        ));
    }
    if !matches!(descriptor.mime_type.as_str(), "image/png" | "image/jpeg") {
        return Err(error(
            "MODEL-TEXTURE-MIME-UNSUPPORTED",
            "texturePayloadDescriptors.mimeType",
            "override image must be image/png or image/jpeg",
        ));
    }
    let start = usize::try_from(descriptor.byte_offset).map_err(|_| {
        error(
            "MODEL-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors.byteOffset",
            "payload offset does not fit this platform",
        )
    })?;
    let length = usize::try_from(descriptor.byte_length).map_err(|_| {
        error(
            "MODEL-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors.byteLength",
            "payload length does not fit this platform",
        )
    })?;
    if descriptor.byte_length > EmbeddedImageDecodeLimitsV1::default().max_decoded_bytes {
        return Err(error(
            "MODEL-TEXTURE-PAYLOAD-LIMIT-EXCEEDED",
            "texturePayloadDescriptors.byteLength",
            "encoded override payload exceeds the V1 per-image byte limit",
        ));
    }
    let end = start.checked_add(length).ok_or_else(|| {
        error(
            "MODEL-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors",
            "payload range overflows",
        )
    })?;
    let payload = payload_blob.get(start..end).ok_or_else(|| {
        error(
            "MODEL-TEXTURE-PAYLOAD-RANGE-INVALID",
            "texturePayloadDescriptors",
            "payload range escapes the supplied blob",
        )
    })?;
    if sha256(payload) != descriptor.sha256 {
        return Err(error(
            "MODEL-TEXTURE-PAYLOAD-HASH-MISMATCH",
            "texturePayloadDescriptors.sha256",
            "payload bytes do not match the descriptor SHA-256",
        ));
    }
    Ok(payload)
}

fn decode_override_image_v1(
    payload: &[u8],
    mime_type: &str,
) -> Result<TgaImageV1, ModelTextureErrorV1> {
    let format = match mime_type {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        _ => {
            return Err(error(
                "MODEL-TEXTURE-MIME-UNSUPPORTED",
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
            "MODEL-TEXTURE-MIME-MISMATCH",
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
            "MODEL-TEXTURE-DECODE-FAILED",
            "textureOverride",
            source.to_string(),
        )
    })?;
    let (width, height) = decoder.dimensions();
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| {
            error(
                "MODEL-TEXTURE-DECODE-LIMIT-EXCEEDED",
                "textureOverride",
                "decoded pixel count overflows",
            )
        })?;
    if pixels > limits.max_pixels {
        return Err(error(
            "MODEL-TEXTURE-DECODE-LIMIT-EXCEEDED",
            "textureOverride",
            "decoded pixel count exceeds the V1 limit",
        ));
    }
    let decoded = DynamicImage::from_decoder(decoder).map_err(|source| {
        error(
            "MODEL-TEXTURE-DECODE-FAILED",
            "textureOverride",
            source.to_string(),
        )
    })?;
    let rgba = decoded.into_rgba8();
    if rgba.pixels().any(|pixel| pixel.0[3] != u8::MAX) {
        return Err(error(
            "MODEL-TEXTURE-ALPHA-UNSUPPORTED",
            "textureOverride.alpha",
            "V1 OPAQUE_ONLY rejects pixels whose alpha is not 255",
        ));
    }
    let mut rgb = Vec::with_capacity(usize::try_from(pixels.saturating_mul(3)).unwrap_or(0));
    for pixel in rgba.pixels() {
        rgb.extend_from_slice(&pixel.0[..3]);
    }
    if u64::try_from(rgb.len()).unwrap_or(u64::MAX) > limits.max_decoded_bytes {
        return Err(error(
            "MODEL-TEXTURE-DECODE-LIMIT-EXCEEDED",
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

#[allow(clippy::too_many_arguments)]
pub fn resolve_model_texture_authoring_v1(
    source_glb: &[u8],
    glb_limits: &GlbLimits,
    ingest: &GlbIngestResult,
    materials: &ResolvedModelMaterialsV1,
    base_texture_resref: &str,
    authoring: &ModelTextureAuthoringDocumentV1,
    payload_blob: &[u8],
    descriptors: &[ModelTexturePayloadDescriptorV1],
) -> Result<ResolvedModelTextureSetV1, ModelTextureErrorV1> {
    resolve_model_texture_authoring_from_report_v1(
        source_glb,
        glb_limits,
        ingest,
        &materials.report,
        base_texture_resref,
        authoring,
        payload_blob,
        descriptors,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_model_texture_authoring_from_report_v1(
    source_glb: &[u8],
    glb_limits: &GlbLimits,
    ingest: &GlbIngestResult,
    materials: &ModelMaterialSeparationReportV1,
    base_texture_resref: &str,
    authoring: &ModelTextureAuthoringDocumentV1,
    payload_blob: &[u8],
    descriptors: &[ModelTexturePayloadDescriptorV1],
) -> Result<ResolvedModelTextureSetV1, ModelTextureErrorV1> {
    validate_resref(base_texture_resref)?;
    if authoring.schema_version != MODEL_TEXTURE_AUTHORING_SCHEMA_VERSION_V1 {
        return Err(error(
            "MODEL-TEXTURE-AUTHORING-SCHEMA-INVALID",
            "textureAuthoring.schemaVersion",
            "texture authoring schemaVersion must be 1",
        ));
    }
    if authoring.source_sha256 != ingest.ir.source.sha256
        || materials.source_sha256 != ingest.ir.source.sha256
    {
        return Err(error(
            "MODEL-TEXTURE-SOURCE-MISMATCH",
            "textureAuthoring.sourceSha256",
            "texture authoring and resolved materials must match the exact source GLB",
        ));
    }
    if authoring.separation_sha256 != materials.separation_sha256 {
        return Err(error(
            "MODEL-TEXTURE-SEPARATION-MISMATCH",
            "textureAuthoring.separationSha256",
            "texture authoring is bound to a different Material Separation recipe",
        ));
    }
    if authoring.bindings.len() != materials.material_slots.len() {
        return Err(error(
            "MODEL-TEXTURE-BINDING-COUNT-MISMATCH",
            "textureAuthoring.bindings",
            "texture authoring must contain one binding for every resolved material slot",
        ));
    }

    let mut descriptor_by_id = BTreeMap::new();
    let mut ranges = Vec::with_capacity(descriptors.len());
    for descriptor in descriptors {
        if descriptor_by_id
            .insert(descriptor.asset_id.clone(), descriptor)
            .is_some()
        {
            return Err(error(
                "MODEL-TEXTURE-DESCRIPTOR-DUPLICATE",
                "texturePayloadDescriptors",
                "descriptor assetId must be unique",
            ));
        }
        descriptor_payload(descriptor, payload_blob)?;
        ranges.push((descriptor.byte_offset, descriptor.byte_length));
    }
    ranges.sort_unstable();
    let mut described_bytes = 0u64;
    for (offset, length) in ranges {
        if offset != described_bytes {
            return Err(error(
                "MODEL-TEXTURE-PAYLOAD-ENVELOPE-INVALID",
                "texturePayloadDescriptors",
                "payload descriptors must cover the blob exactly without gaps or overlaps",
            ));
        }
        described_bytes = described_bytes.checked_add(length).ok_or_else(|| {
            error(
                "MODEL-TEXTURE-PAYLOAD-ENVELOPE-INVALID",
                "texturePayloadDescriptors",
                "payload descriptor envelope overflows",
            )
        })?;
    }
    if described_bytes != payload_blob.len() as u64 {
        return Err(error(
            "MODEL-TEXTURE-PAYLOAD-ENVELOPE-INVALID",
            "texturePayloadDescriptors",
            "payload descriptors must cover every supplied byte exactly",
        ));
    }

    let slots = materials
        .material_slots
        .iter()
        .map(|slot| (slot.material_slot, slot))
        .collect::<BTreeMap<_, _>>();
    let mut authoring_by_slot = BTreeMap::new();
    let mut referenced_assets = BTreeSet::new();
    for binding in &authoring.bindings {
        if authoring_by_slot
            .insert(binding.material_slot, binding)
            .is_some()
        {
            return Err(error(
                "MODEL-TEXTURE-BINDING-DUPLICATE",
                "textureAuthoring.bindings",
                "materialSlot must be unique",
            ));
        }
        let slot = slots.get(&binding.material_slot).ok_or_else(|| {
            error(
                "MODEL-TEXTURE-BINDING-STALE",
                "textureAuthoring.bindings.materialSlot",
                "binding references a missing resolved material slot",
            )
        })?;
        if binding.authored_material_id != slot.authored_material_id
            || binding.source_material_id != slot.source_material_id
            || binding.source_image_sha256 != slot.source_image_sha256
        {
            return Err(error(
                "MODEL-TEXTURE-BINDING-STALE",
                "textureAuthoring.bindings",
                "binding identity no longer matches its resolved material slot",
            ));
        }
        match binding.mode {
            ModelTextureBindingModeV1::Source => {
                if binding.override_asset_id.is_some()
                    || binding.override_sha256.is_some()
                    || binding.override_mime_type.is_some()
                    || binding.override_byte_length.is_some()
                {
                    return Err(error(
                        "MODEL-TEXTURE-SOURCE-BINDING-INVALID",
                        "textureAuthoring.bindings",
                        "SOURCE binding must not contain override metadata",
                    ));
                }
            }
            ModelTextureBindingModeV1::Override => {
                let asset_id = binding.override_asset_id.as_ref().ok_or_else(|| {
                    error(
                        "MODEL-TEXTURE-OVERRIDE-INCOMPLETE",
                        "textureAuthoring.bindings.overrideAssetId",
                        "OVERRIDE binding requires overrideAssetId",
                    )
                })?;
                let descriptor = descriptor_by_id.get(asset_id).ok_or_else(|| {
                    error(
                        "MODEL-TEXTURE-PAYLOAD-MISSING",
                        "texturePayloadDescriptors",
                        "override asset has no payload descriptor",
                    )
                })?;
                if binding.override_sha256.as_deref() != Some(descriptor.sha256.as_str())
                    || binding.override_mime_type.as_deref() != Some(descriptor.mime_type.as_str())
                    || binding.override_byte_length != Some(descriptor.byte_length)
                {
                    return Err(error(
                        "MODEL-TEXTURE-OVERRIDE-DESCRIPTOR-MISMATCH",
                        "textureAuthoring.bindings",
                        "override metadata does not match its payload descriptor",
                    ));
                }
                referenced_assets.insert(asset_id.clone());
            }
        }
    }
    if authoring_by_slot.len() != slots.len() {
        return Err(error(
            "MODEL-TEXTURE-BINDING-MISSING",
            "textureAuthoring.bindings",
            "one or more resolved material slots have no texture binding",
        ));
    }
    if referenced_assets.len() != descriptor_by_id.len() {
        return Err(error(
            "MODEL-TEXTURE-PAYLOAD-UNUSED",
            "texturePayloadDescriptors",
            "every supplied override payload must be referenced",
        ));
    }

    let mut material_textures = Vec::with_capacity(slots.len());
    let mut textures = Vec::<ResolvedModelTexturePayloadV1>::new();
    let mut resource_index_by_sha = BTreeMap::<String, usize>::new();
    let mut resource_slots = Vec::<Vec<u32>>::new();
    let mut binding_reports = Vec::with_capacity(slots.len());

    for (material_slot, slot) in slots {
        let binding = authoring_by_slot[&material_slot];
        if binding.alpha_policy != ModelTextureAlphaPolicyV1::OpaqueOnly {
            return Err(error(
                "MODEL-TEXTURE-ALPHA-POLICY-UNSUPPORTED",
                "textureAuthoring.bindings.alphaPolicy",
                "V1 supports only OPAQUE_ONLY",
            ));
        }
        let source_material = slot.source_material_id.and_then(|id| {
            ingest
                .ir
                .materials
                .iter()
                .find(|material| material.id == id)
        });
        if slot.source_material_id.is_some() && source_material.is_none() {
            return Err(error(
                "MODEL-TEXTURE-SOURCE-MATERIAL-MISSING",
                "textureAuthoring.bindings.sourceMaterialId",
                "source fallback material is absent",
            ));
        }
        let source_alpha_mode = source_material
            .map(|material| material.alpha_mode.clone())
            .unwrap_or_else(|| "OPAQUE".to_owned());
        if binding.mode == ModelTextureBindingModeV1::Override && source_alpha_mode != "OPAQUE" {
            return Err(error(
                "MODEL-TEXTURE-ALPHA-MODE-UNSUPPORTED",
                "textureAuthoring.bindings.alphaPolicy",
                "V1 override requires an OPAQUE source fallback material",
            ));
        }
        let ignored_source_pbr_maps = source_material
            .map(|material| {
                [
                    (material.normal_texture.is_some(), "normalTexture"),
                    (
                        material.metallic_roughness_texture.is_some(),
                        "metallicRoughnessTexture",
                    ),
                    (material.emissive_texture.is_some(), "emissiveTexture"),
                ]
                .into_iter()
                .filter_map(|(present, name)| present.then_some(name.to_owned()))
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let (input_sha256, image) = match binding.mode {
            ModelTextureBindingModeV1::Source => {
                let source_image_sha256 = slot.source_image_sha256.as_ref().ok_or_else(|| {
                    error(
                        "MODEL-TEXTURE-SOURCE-FALLBACK-MISSING",
                        "textureAuthoring.bindings.sourceImageSha256",
                        "SOURCE mode requires an exact source base-color image fallback",
                    )
                })?;
                let source_material = source_material.ok_or_else(|| {
                    error(
                        "MODEL-TEXTURE-SOURCE-FALLBACK-MISSING",
                        "textureAuthoring.bindings.sourceMaterialId",
                        "SOURCE mode requires a source material fallback",
                    )
                })?;
                let texture_id = source_material
                    .base_color_texture
                    .as_ref()
                    .ok_or_else(|| {
                        error(
                            "MODEL-TEXTURE-SOURCE-FALLBACK-MISSING",
                            "materials.baseColorTexture",
                            "SOURCE mode fallback has no base-color texture",
                        )
                    })?
                    .texture_id;
                let texture = ingest
                    .ir
                    .textures
                    .iter()
                    .find(|texture| texture.id == texture_id)
                    .ok_or_else(|| {
                        error(
                            "MODEL-TEXTURE-SOURCE-FALLBACK-MISSING",
                            "textures",
                            "SOURCE mode fallback texture is absent",
                        )
                    })?;
                let image_index = ingest
                    .ir
                    .images
                    .iter()
                    .position(|image| image.id == texture.source_image_id)
                    .ok_or_else(|| {
                        error(
                            "MODEL-TEXTURE-SOURCE-FALLBACK-MISSING",
                            "images",
                            "SOURCE mode fallback image is absent",
                        )
                    })?;
                if ingest.ir.images[image_index].sha256 != *source_image_sha256 {
                    return Err(error(
                        "MODEL-TEXTURE-BINDING-STALE",
                        "textureAuthoring.bindings.sourceImageSha256",
                        "source fallback image hash no longer matches",
                    ));
                }
                let image = decode_embedded_image_to_tga_v1(
                    source_glb,
                    image_index,
                    glb_limits,
                    &EmbeddedImageDecodeLimitsV1::default(),
                )
                .map_err(|source| {
                    error(
                        &format!("MODEL-{}", source.code),
                        source.json_path.unwrap_or_else(|| "images".to_owned()),
                        source.message,
                    )
                })?;
                (source_image_sha256.clone(), image)
            }
            ModelTextureBindingModeV1::Override => {
                let asset_id = binding
                    .override_asset_id
                    .as_ref()
                    .expect("validated override");
                let descriptor = descriptor_by_id[asset_id];
                let payload = descriptor_payload(descriptor, payload_blob)?;
                (
                    descriptor.sha256.clone(),
                    decode_override_image_v1(payload, &descriptor.mime_type)?,
                )
            }
        };
        let tga = write_tga_v1(&image, &TgaWriterOptionsV1::default()).map_err(|source| {
            error(
                &format!("MODEL-{}", source.code),
                source.path,
                source.message,
            )
        })?;
        let output_sha256 = tga.report.output_sha256.clone();
        let resource_index = if let Some(index) = resource_index_by_sha.get(&output_sha256) {
            *index
        } else {
            let index = textures.len();
            textures.push(ResolvedModelTexturePayloadV1 {
                resref: derived_resref(base_texture_resref, index)?,
                resource_type: MODEL_TEXTURE_TGA_RESOURCE_TYPE_V1,
                payload: tga.payload,
            });
            resource_slots.push(Vec::new());
            resource_index_by_sha.insert(output_sha256.clone(), index);
            index
        };
        resource_slots[resource_index].push(material_slot);
        let output_resref = textures[resource_index].resref.clone();
        material_textures.push(MdlMaterialTextureBindingV1 {
            material_slot,
            resref: output_resref.clone(),
        });
        binding_reports.push(ResolvedModelTextureBindingV1 {
            authored_material_id: slot.authored_material_id.clone(),
            material_slot,
            source_material_id: slot.source_material_id,
            source_image_sha256: slot.source_image_sha256.clone(),
            mode: binding.mode,
            input_sha256,
            output_resref,
            output_sha256,
            source_alpha_mode,
            ignored_source_pbr_maps,
        });
    }
    let resources = textures
        .iter()
        .zip(resource_slots)
        .map(|(texture, material_slots)| ResolvedModelTextureResourceV1 {
            resref: texture.resref.clone(),
            resource_type: texture.resource_type,
            byte_length: texture.payload.len() as u64,
            sha256: sha256(&texture.payload),
            material_slots,
        })
        .collect();
    Ok(ResolvedModelTextureSetV1 {
        report: ModelTextureResolutionReportV1 {
            schema_version: MODEL_TEXTURE_RESOLUTION_SCHEMA_VERSION_V1,
            source_sha256: ingest.ir.source.sha256.clone(),
            separation_sha256: materials.separation_sha256.clone(),
            authoring_sha256: model_texture_authoring_hash_v1(authoring)?,
            alpha_policy: ModelTextureAlphaPolicyV1::OpaqueOnly,
            uv_policy: "SOURCE_UV0_UNCHANGED".to_owned(),
            bindings: binding_reports,
            resources,
        },
        material_textures,
        textures,
    })
}
