//! Byte-producing bridge from compiled glTF materials to Aurora resources.
//!
//! The compiler in `aurora_material` decides channel fidelity. This module
//! executes that decision for a converted common model IR and returns the
//! exact MDL material state plus TGA/MTR/TXI HAK resources.

use std::{collections::BTreeMap, fmt};

use serde::Serialize;

use crate::{
    aurora_material::{
        AuroraAlphaModeV1, AuroraMaterialCompilationSetV1, AuroraMaterialCompileStatusV1,
        AuroraMaterialIrV1, AuroraRenderHintV1,
    },
    aurora_material_bake::{bake_classic_diffuse_v1, bake_specular_gloss_v1},
    glb::{
        EmbeddedImageDecodeLimitsV1, GlbIngestResult, GlbLimits, decode_embedded_image_to_tga_v1,
    },
    hak::HakResourceInputV1,
    mdl::{MdlMaterialStateV1, MdlMaterialTextureBindingV1, MdlRenderHintV1},
    model_ir::{AuroraModelIrV1, AuroraModelSegmentV1},
    model_source_quality::{
        MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1, ModelSourceQualityDiagnosticV1,
        ModelSourceQualityInputV1, ModelSourceQualityReportV1, ModelSourceQualityStatusV1,
        SourceMaterialQualityInputV1, inspect_model_source_quality_v1,
    },
    model_texture_authoring::{
        TextureMipReadabilityStatusV1, TextureMipReadabilityV1, inspect_texture_mip_readability_v1,
    },
    mtr::{
        MTR_RESOURCE_TYPE_V1, MtrMaterialResourceNamesV1, compile_mtr_document_v1, parse_mtr_v1,
        write_mtr_v1,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, read_tga_image_v1, write_tga_v1},
    txi::{TXI_RESOURCE_TYPE_V1, TxiBlendingV1, TxiDocumentV1, parse_txi_v1, write_txi_v1},
};

pub const AURORA_MATERIAL_PACKAGE_SCHEMA_VERSION_V1: u32 = 1;
const TGA_RESOURCE_TYPE_V1: u16 = 3;

#[derive(Clone, Debug, PartialEq)]
pub struct AuroraMaterialPackageSlotV1 {
    pub slot: u32,
    pub state: MdlMaterialStateV1,
    pub diffuse_binding: MdlMaterialTextureBindingV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuroraMaterialPackageV1 {
    pub slots: Vec<AuroraMaterialPackageSlotV1>,
    pub resources: Vec<HakResourceInputV1>,
    pub source_quality: ModelSourceQualityReportV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuroraDiffuseOverrideV1 {
    pub material_slot: u32,
    pub source_material_id: u32,
    pub resref: String,
    pub tga_payload: Vec<u8>,
    pub texture_readability: crate::model_texture_authoring::TextureMipReadabilityV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuroraMaterialPackageErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for AuroraMaterialPackageErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for AuroraMaterialPackageErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> AuroraMaterialPackageErrorV1 {
    AuroraMaterialPackageErrorV1 {
        schema_version: AURORA_MATERIAL_PACKAGE_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

/// Produces target resources for every output material slot. Authored material
/// separation remains source-bound: each output slot must identify the glTF
/// material whose PBR channels are compiled.
pub fn package_aurora_materials_v1(
    source_glb: &[u8],
    ingest: &GlbIngestResult,
    model: &AuroraModelIrV1,
    compilation: &AuroraMaterialCompilationSetV1,
    texture_resref: &str,
    diffuse_overrides: &[AuroraDiffuseOverrideV1],
) -> Result<AuroraMaterialPackageV1, AuroraMaterialPackageErrorV1> {
    if compilation.source_sha256 != ingest.ir.source.sha256 {
        return Err(error(
            "AURORA-MATERIAL-PACKAGE-SOURCE-MISMATCH",
            "materialCompilation.sourceSha256",
            "material compilation belongs to another GLB",
        ));
    }
    if compilation.status == AuroraMaterialCompileStatusV1::Blocked {
        return Err(error(
            "AURORA-MATERIAL-PACKAGE-COMPILER-BLOCKED",
            "materialCompilation.status",
            "blocked material channels cannot be packaged",
        ));
    }
    let compiled = compilation
        .materials
        .iter()
        .map(|entry| (entry.material.source_material_id, &entry.material))
        .collect::<BTreeMap<_, _>>();
    let override_by_slot = diffuse_overrides
        .iter()
        .map(|entry| (entry.material_slot, entry))
        .collect::<BTreeMap<_, _>>();
    if override_by_slot.len() != diffuse_overrides.len() {
        return Err(error(
            "AURORA-MATERIAL-PACKAGE-DIFFUSE-OVERRIDE-DUPLICATE",
            "diffuseOverrides",
            "diffuse override material slots must be unique",
        ));
    }
    let mut slots = Vec::new();
    let mut resources = Vec::new();
    for binding in &model.material_source_bindings {
        let source_id = binding.source_material_id.ok_or_else(|| {
            error(
                "AURORA-MATERIAL-PACKAGE-SOURCE-MATERIAL-MISSING",
                "model.materialSourceBindings",
                format!(
                    "output material slot {} has no source fallback material",
                    binding.slot
                ),
            )
        })?;
        let material = compiled.get(&source_id).copied().ok_or_else(|| {
            error(
                "AURORA-MATERIAL-PACKAGE-COMPILED-MATERIAL-MISSING",
                "materialCompilation.materials",
                format!("source material {source_id} has no compiler output"),
            )
        })?;
        let packaged = package_slot_v1(
            source_glb,
            ingest,
            binding.slot,
            material,
            texture_resref,
            override_by_slot.get(&binding.slot).copied(),
        )?;
        resources.extend(packaged.1);
        slots.push(packaged.0);
    }
    slots.sort_by_key(|entry| entry.slot);
    resources.sort_by(|left, right| {
        left.resref
            .cmp(&right.resref)
            .then(left.resource_type.cmp(&right.resource_type))
    });
    for pair in resources.windows(2) {
        if pair[0].resref.eq_ignore_ascii_case(&pair[1].resref)
            && pair[0].resource_type == pair[1].resource_type
            && pair[0].payload != pair[1].payload
        {
            return Err(error(
                "AURORA-MATERIAL-PACKAGE-RESOURCE-DUPLICATE",
                "resources",
                format!(
                    "duplicate resource {}:{}",
                    pair[0].resref, pair[0].resource_type
                ),
            ));
        }
    }
    resources.dedup_by(|left, right| {
        left.resref.eq_ignore_ascii_case(&right.resref)
            && left.resource_type == right.resource_type
            && left.payload == right.payload
    });
    let source_quality = inspect_source_quality_v1(source_glb, ingest, diffuse_overrides)?;
    if source_quality.status == ModelSourceQualityStatusV1::Blocked {
        let codes = source_quality
            .diagnostics
            .iter()
            .filter(|item| item.status == ModelSourceQualityStatusV1::Blocked)
            .map(|item| item.code.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(error(
            "AURORA-MATERIAL-PACKAGE-SOURCE-QUALITY-BLOCKED",
            "sourceQuality",
            format!("source quality gate blocked packaging: {codes}"),
        ));
    }
    Ok(AuroraMaterialPackageV1 {
        slots,
        resources,
        source_quality,
    })
}

fn package_slot_v1(
    source_glb: &[u8],
    ingest: &GlbIngestResult,
    slot: u32,
    material: &AuroraMaterialIrV1,
    base_resref: &str,
    diffuse_override: Option<&AuroraDiffuseOverrideV1>,
) -> Result<(AuroraMaterialPackageSlotV1, Vec<HakResourceInputV1>), AuroraMaterialPackageErrorV1> {
    let source = ingest
        .ir
        .materials
        .iter()
        .find(|value| value.id == material.source_material_id)
        .ok_or_else(|| {
            error(
                "AURORA-MATERIAL-PACKAGE-SOURCE-MATERIAL-MISSING",
                "source.materials",
                format!("source material {} is missing", material.source_material_id),
            )
        })?;
    let diffuse_resref = diffuse_override.map_or_else(
        || material_resref(base_resref, 'd', slot, slot == 0),
        |entry| Ok(entry.resref.clone()),
    )?;
    let normal_resref = material
        .normal_texture
        .as_ref()
        .map(|_| material_resref(base_resref, 'n', slot, false))
        .transpose()?;
    let specular_resref = material
        .specular_texture_plan
        .as_ref()
        .map(|_| material_resref(base_resref, 's', slot, false))
        .transpose()?;
    let mtr_resref = material
        .mtr_required
        .then(|| material_resref(base_resref, 'm', slot, false))
        .transpose()?;
    let base = if let Some(entry) = diffuse_override {
        read_tga_image_v1(&entry.tga_payload)
            .map_err(|source| error(&source.code, "diffuseOverrides.tgaPayload", source.message))?
    } else if let Some(binding) = &source.base_color_texture {
        decode_texture(source_glb, ingest, binding.texture_id)?
    } else {
        TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgba8,
            // The MDL diffuse controller carries the factor. A white texture
            // avoids applying it twice while satisfying the Placeable texture
            // resource contract.
            pixels: vec![255, 255, 255, 255],
        }
    };
    let metallic_roughness = source
        .metallic_roughness_texture
        .as_ref()
        .map(|binding| decode_texture(source_glb, ingest, binding.texture_id))
        .transpose()?;
    let diffuse_image = if let Some(plan) = &material.classic_diffuse_bake_plan {
        bake_classic_diffuse_v1(plan, &base, metallic_roughness.as_ref())
            .map_err(|source| error(&source.code, source.path, source.message))?
            .image
    } else {
        base.clone()
    };
    let diffuse_payload = if material.classic_diffuse_bake_plan.is_none() {
        if let Some(entry) = diffuse_override {
            entry.tga_payload.clone()
        } else {
            write_tga_v1(&diffuse_image, &TgaWriterOptionsV1::default())
                .map_err(|source| error(&source.code, source.path, source.message))?
                .payload
        }
    } else {
        write_tga_v1(&diffuse_image, &TgaWriterOptionsV1::default())
            .map_err(|source| error(&source.code, source.path, source.message))?
            .payload
    };
    let mut resources = vec![HakResourceInputV1 {
        resref: diffuse_resref.clone(),
        resource_type: TGA_RESOURCE_TYPE_V1,
        payload: diffuse_payload,
    }];
    let mut diffuse_txi = TxiDocumentV1 {
        schema_version: 1,
        mipmap: Some(true),
        filter: Some(true),
        gamma: Some(2.2),
        is_bump_map: None,
        clamp: Some(false),
        alpha_mean: None,
        is_diffuse_bump_map: None,
        is_specular_bump_map: None,
        bump_map_scaling: None,
        specular_color: None,
        blending: None,
    };
    if material.alpha_mode == AuroraAlphaModeV1::Mask {
        diffuse_txi.blending = Some(TxiBlendingV1::Punchthrough);
    }
    resources.push(txi_resource(&diffuse_resref, diffuse_txi)?);
    if let (Some(binding), Some(resref)) = (&source.normal_texture, &normal_resref) {
        let image = decode_texture(source_glb, ingest, binding.texture_id)?;
        let tga = write_tga_v1(&image, &TgaWriterOptionsV1::default())
            .map_err(|source| error(&source.code, source.path, source.message))?;
        resources.push(HakResourceInputV1 {
            resref: resref.clone(),
            resource_type: TGA_RESOURCE_TYPE_V1,
            payload: tga.payload,
        });
        resources.push(txi_resource(resref, TxiDocumentV1::normal_map_v1())?);
    }
    if let (Some(plan), Some(resref)) = (&material.specular_texture_plan, &specular_resref) {
        let bake = bake_specular_gloss_v1(plan, &base, metallic_roughness.as_ref())
            .map_err(|source| error(&source.code, source.path, source.message))?;
        resources.push(HakResourceInputV1 {
            resref: resref.clone(),
            resource_type: TGA_RESOURCE_TYPE_V1,
            payload: bake.tga.payload,
        });
        resources.push(txi_resource(
            resref,
            TxiDocumentV1 {
                schema_version: 1,
                mipmap: Some(true),
                filter: Some(true),
                gamma: None,
                is_bump_map: None,
                clamp: Some(false),
                alpha_mean: None,
                is_diffuse_bump_map: None,
                is_specular_bump_map: Some(true),
                bump_map_scaling: None,
                specular_color: Some(material.specular_color),
                blending: None,
            },
        )?);
    }
    if let Some(resref) = &mtr_resref {
        let document = compile_mtr_document_v1(
            material,
            &MtrMaterialResourceNamesV1 {
                diffuse: Some(diffuse_resref.clone()),
                normal: normal_resref.clone(),
                specular: specular_resref.clone(),
            },
        )
        .map_err(|source| error(&source.code, "mtr", source.message))?;
        let payload =
            write_mtr_v1(&document).map_err(|source| error(&source.code, "mtr", source.message))?;
        if parse_mtr_v1(&payload).map_err(|source| error(&source.code, "mtr", source.message))?
            != document
        {
            return Err(error(
                "AURORA-MATERIAL-PACKAGE-MTR-SEMANTIC-DIFF",
                "mtr",
                "MTR differs after canonical readback",
            ));
        }
        resources.push(HakResourceInputV1 {
            resref: resref.clone(),
            resource_type: MTR_RESOURCE_TYPE_V1,
            payload,
        });
    }
    let state = MdlMaterialStateV1 {
        material_slot: slot,
        diffuse: material.diffuse_color[..3]
            .try_into()
            .expect("three diffuse components"),
        ambient: material.ambient_color,
        specular: material.specular_color,
        shininess: material.shininess,
        alpha: material.diffuse_color[3],
        self_illum_color: material.self_illum_color,
        transparency_hint: material
            .mtr
            .as_ref()
            .is_some_and(|value| value.transparency),
        render_hint: match material.render_hint {
            AuroraRenderHintV1::Normal => MdlRenderHintV1::Normal,
            AuroraRenderHintV1::NormalAndSpecMapped => MdlRenderHintV1::NormalAndSpecMapped,
        },
        normal_texture_resref: normal_resref,
        specular_texture_resref: specular_resref,
        material_resref: mtr_resref,
    };
    Ok((
        AuroraMaterialPackageSlotV1 {
            slot,
            state,
            diffuse_binding: MdlMaterialTextureBindingV1 {
                material_slot: slot,
                resref: diffuse_resref,
            },
        },
        resources,
    ))
}

fn inspect_source_quality_v1(
    source_glb: &[u8],
    ingest: &GlbIngestResult,
    diffuse_overrides: &[AuroraDiffuseOverrideV1],
) -> Result<ModelSourceQualityReportV1, AuroraMaterialPackageErrorV1> {
    let override_readability = representative_override_readability_v1(diffuse_overrides);
    let materials = ingest
        .ir
        .materials
        .iter()
        .map(|material| {
            let texture_readability = if let Some(value) = override_readability.get(&material.id) {
                Some(value.clone())
            } else {
                material
                    .base_color_texture
                    .as_ref()
                    .map(|binding| decode_texture(source_glb, ingest, binding.texture_id))
                    .transpose()?
                    .as_ref()
                    .map(inspect_texture_mip_readability_v1)
            };
            Ok(SourceMaterialQualityInputV1 {
                material_id: material.id,
                texture_readability,
            })
        })
        .collect::<Result<Vec<_>, AuroraMaterialPackageErrorV1>>()?;
    let mut report = inspect_model_source_quality_v1(
        &ingest.ir,
        &ModelSourceQualityInputV1 {
            schema_version: MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1,
            source_sha256: ingest.ir.source.sha256.clone(),
            materials,
        },
    )
    .map_err(|source| error(&source.code, source.path, source.message))?;
    apply_override_contrast_diagnostics_v1(&mut report, diffuse_overrides);
    Ok(report)
}

fn representative_override_readability_v1(
    diffuse_overrides: &[AuroraDiffuseOverrideV1],
) -> BTreeMap<u32, TextureMipReadabilityV1> {
    let mut result = BTreeMap::new();
    for entry in diffuse_overrides {
        result
            .entry(entry.source_material_id)
            .and_modify(|current| {
                if readability_key_v1(&entry.texture_readability) < readability_key_v1(current) {
                    *current = entry.texture_readability.clone();
                }
            })
            .or_insert_with(|| entry.texture_readability.clone());
    }
    result
}

fn readability_key_v1(value: &TextureMipReadabilityV1) -> (u8, u64, u32, u32) {
    let status_rank = match value.status {
        TextureMipReadabilityStatusV1::Flat => 0,
        TextureMipReadabilityStatusV1::LowContrast => 1,
        TextureMipReadabilityStatusV1::Readable => 2,
    };
    (
        status_rank,
        u64::from(value.source_width) * u64::from(value.source_height),
        value.mip_16_luma_stddev_milli,
        value.contrast_retention_basis_points,
    )
}

fn apply_override_contrast_diagnostics_v1(
    report: &mut ModelSourceQualityReportV1,
    diffuse_overrides: &[AuroraDiffuseOverrideV1],
) {
    let overridden_paths = diffuse_overrides
        .iter()
        .map(|entry| format!("materials[{}].textureReadability", entry.source_material_id))
        .collect::<std::collections::BTreeSet<_>>();
    report.diagnostics.retain(|diagnostic| {
        !matches!(
            diagnostic.code.as_str(),
            "SOURCE-QUALITY-CONTRAST-LOW" | "SOURCE-QUALITY-CONTRAST-FLAT"
        ) || !overridden_paths.contains(&diagnostic.path)
    });
    let mut overrides = diffuse_overrides.iter().collect::<Vec<_>>();
    overrides.sort_by_key(|entry| entry.material_slot);
    for entry in overrides {
        let (code, status, message) = match entry.texture_readability.status {
            TextureMipReadabilityStatusV1::Readable => continue,
            TextureMipReadabilityStatusV1::LowContrast => (
                "SOURCE-QUALITY-CONTRAST-LOW",
                ModelSourceQualityStatusV1::Warning,
                format!(
                    "output material slot {} has only {} milli of 16x16 mip luma deviation",
                    entry.material_slot, entry.texture_readability.mip_16_luma_stddev_milli
                ),
            ),
            TextureMipReadabilityStatusV1::Flat => (
                "SOURCE-QUALITY-CONTRAST-FLAT",
                ModelSourceQualityStatusV1::Blocked,
                format!(
                    "output material slot {} texture has no useful macro contrast",
                    entry.material_slot
                ),
            ),
        };
        report.diagnostics.push(ModelSourceQualityDiagnosticV1 {
            code: code.to_owned(),
            status,
            path: format!(
                "diffuseOverrides[materialSlot={}].textureReadability",
                entry.material_slot
            ),
            message,
        });
    }
    report.status = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.status)
        .max()
        .unwrap_or(ModelSourceQualityStatusV1::Pass);
}

fn decode_texture(
    source_glb: &[u8],
    ingest: &GlbIngestResult,
    texture_id: u32,
) -> Result<TgaImageV1, AuroraMaterialPackageErrorV1> {
    let texture = ingest
        .ir
        .textures
        .iter()
        .find(|value| value.id == texture_id)
        .ok_or_else(|| {
            error(
                "AURORA-MATERIAL-PACKAGE-TEXTURE-MISSING",
                "source.textures",
                format!("texture {texture_id} is missing"),
            )
        })?;
    let image_index = ingest
        .ir
        .images
        .iter()
        .position(|image| image.id == texture.source_image_id)
        .ok_or_else(|| {
            error(
                "AURORA-MATERIAL-PACKAGE-IMAGE-MISSING",
                "source.images",
                format!("image {} is missing", texture.source_image_id),
            )
        })?;
    decode_embedded_image_to_tga_v1(
        source_glb,
        image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            &source.code,
            source
                .json_path
                .unwrap_or_else(|| "source.images".to_owned()),
            source.message,
        )
    })
}

fn txi_resource(
    resref: &str,
    document: TxiDocumentV1,
) -> Result<HakResourceInputV1, AuroraMaterialPackageErrorV1> {
    let payload =
        write_txi_v1(&document).map_err(|source| error(&source.code, "txi", source.message))?;
    if parse_txi_v1(&payload).map_err(|source| error(&source.code, "txi", source.message))?
        != document
    {
        return Err(error(
            "AURORA-MATERIAL-PACKAGE-TXI-SEMANTIC-DIFF",
            "txi",
            "TXI differs after canonical readback",
        ));
    }
    Ok(HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type: TXI_RESOURCE_TYPE_V1,
        payload,
    })
}

fn material_resref(
    base: &str,
    role: char,
    slot: u32,
    identity_diffuse: bool,
) -> Result<String, AuroraMaterialPackageErrorV1> {
    if identity_diffuse {
        return Ok(base.to_owned());
    }
    let suffix = format!("_{role}{slot:x}");
    let prefix_length = 16usize.checked_sub(suffix.len()).ok_or_else(|| {
        error(
            "AURORA-MATERIAL-PACKAGE-RESREF-EXHAUSTED",
            "textureResref",
            "material resource suffix exceeds the Aurora resref limit",
        )
    })?;
    let prefix = base.get(..base.len().min(prefix_length)).ok_or_else(|| {
        error(
            "AURORA-MATERIAL-PACKAGE-RESREF-INVALID",
            "textureResref",
            "texture resref is not ASCII",
        )
    })?;
    Ok(format!("{prefix}{suffix}"))
}

/// Generates finite tangent frames for material slots that require normal or
/// specular mapping. It never changes positions, UVs, indices or face count.
pub fn ensure_material_tangents_v1(
    model: &mut AuroraModelIrV1,
    slots: &[AuroraMaterialPackageSlotV1],
) -> Result<(), AuroraMaterialPackageErrorV1> {
    let needs_tangents = slots
        .iter()
        .filter(|entry| entry.state.render_hint == MdlRenderHintV1::NormalAndSpecMapped)
        .map(|entry| entry.slot)
        .collect::<std::collections::BTreeSet<_>>();
    for segment in &mut model.segments {
        if needs_tangents.contains(&segment.material_slot) && segment.tangents.is_none() {
            segment.tangents = Some(generate_tangents(segment)?);
        }
    }
    Ok(())
}

fn generate_tangents(
    segment: &AuroraModelSegmentV1,
) -> Result<Vec<[f32; 4]>, AuroraMaterialPackageErrorV1> {
    if segment.uv0.len() != segment.positions.len()
        || segment.normals.len() != segment.positions.len()
        || !segment.indices.len().is_multiple_of(3)
    {
        return Err(error(
            "AURORA-MATERIAL-PACKAGE-TANGENT-INPUT-INVALID",
            "model.segments",
            "normal-mapped segment requires positions, normals and UV0 per vertex",
        ));
    }
    let mut accumulated = vec![[0.0_f32; 3]; segment.positions.len()];
    for triangle in segment.indices.chunks_exact(3) {
        let [i0, i1, i2] = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if i0 >= segment.positions.len()
            || i1 >= segment.positions.len()
            || i2 >= segment.positions.len()
        {
            return Err(error(
                "AURORA-MATERIAL-PACKAGE-TANGENT-INDEX-OOB",
                "model.segments.indices",
                "tangent generation encountered an out-of-range index",
            ));
        }
        let p0 = segment.positions[i0];
        let p1 = segment.positions[i1];
        let p2 = segment.positions[i2];
        let uv0 = segment.uv0[i0];
        let uv1 = segment.uv0[i1];
        let uv2 = segment.uv0[i2];
        let edge1 = sub3(p1, p0);
        let edge2 = sub3(p2, p0);
        let duv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
        let duv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
        let denominator = duv1[0] * duv2[1] - duv1[1] * duv2[0];
        if denominator.abs() <= 1.0e-12 {
            continue;
        }
        let inverse = denominator.recip();
        let tangent = [
            (edge1[0] * duv2[1] - edge2[0] * duv1[1]) * inverse,
            (edge1[1] * duv2[1] - edge2[1] * duv1[1]) * inverse,
            (edge1[2] * duv2[1] - edge2[2] * duv1[1]) * inverse,
        ];
        for index in [i0, i1, i2] {
            accumulated[index] = add3(accumulated[index], tangent);
        }
    }
    Ok(accumulated
        .into_iter()
        .zip(&segment.normals)
        .map(|(tangent, normal)| {
            let projected = sub3(tangent, scale3(*normal, dot3(tangent, *normal)));
            let fallback = perpendicular(*normal);
            let tangent = normalize3(projected).unwrap_or(fallback);
            [tangent[0], tangent[1], tangent[2], 1.0]
        })
        .collect())
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn sub3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn normalize3(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot3(value, value).sqrt();
    (length.is_finite() && length > 1.0e-8).then(|| scale3(value, length.recip()))
}

fn perpendicular(normal: [f32; 3]) -> [f32; 3] {
    let axis = if normal[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    normalize3([
        normal[1] * axis[2] - normal[2] * axis[1],
        normal[2] * axis[0] - normal[0] * axis[2],
        normal[0] * axis[1] - normal[1] * axis[0],
    ])
    .unwrap_or([1.0, 0.0, 0.0])
}

pub fn validate_material_resource_semantics_v1(
    resource: &HakResourceInputV1,
) -> Result<(), AuroraMaterialPackageErrorV1> {
    match resource.resource_type {
        TGA_RESOURCE_TYPE_V1 => {
            read_tga_image_v1(&resource.payload)
                .map_err(|source| error(&source.code, "resources.tga", source.message))?;
        }
        MTR_RESOURCE_TYPE_V1 => {
            parse_mtr_v1(&resource.payload)
                .map_err(|source| error(&source.code, "resources.mtr", source.message))?;
        }
        TXI_RESOURCE_TYPE_V1 => {
            parse_txi_v1(&resource.payload)
                .map_err(|source| error(&source.code, "resources.txi", source.message))?;
        }
        _ => {
            return Err(error(
                "AURORA-MATERIAL-PACKAGE-RESOURCE-TYPE-UNSUPPORTED",
                "resources.resourceType",
                format!(
                    "unsupported material resource type {}",
                    resource.resource_type
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn readability(status: TextureMipReadabilityStatusV1) -> TextureMipReadabilityV1 {
        TextureMipReadabilityV1 {
            source_width: 64,
            source_height: 64,
            base_luma_stddev_milli: 10,
            mip_16_luma_stddev_milli: match status {
                TextureMipReadabilityStatusV1::Readable => 10,
                TextureMipReadabilityStatusV1::LowContrast => 2,
                TextureMipReadabilityStatusV1::Flat => 0,
            },
            contrast_retention_basis_points: 10_000,
            status,
        }
    }

    fn diffuse_override(
        material_slot: u32,
        status: TextureMipReadabilityStatusV1,
    ) -> AuroraDiffuseOverrideV1 {
        AuroraDiffuseOverrideV1 {
            material_slot,
            source_material_id: 0,
            resref: format!("override_{material_slot}"),
            tga_payload: Vec::new(),
            texture_readability: readability(status),
        }
    }

    fn passing_report() -> ModelSourceQualityReportV1 {
        ModelSourceQualityReportV1 {
            schema_version: MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1,
            source_sha256: "a".repeat(64),
            status: ModelSourceQualityStatusV1::Pass,
            triangle_count: 1,
            component_count: 1,
            tiny_component_count: 1,
            components_per_thousand_triangles: 1_000,
            missing_uv_triangle_count: 0,
            degenerate_uv_triangle_count: 0,
            uv_outside_unit_triangle_count: 0,
            minimum_texels_per_meter: 64,
            weighted_texels_per_meter: 64,
            maximum_texels_per_meter: 64,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn shared_source_overrides_fail_closed_per_output_slot_regardless_of_order() {
        let readable = diffuse_override(2, TextureMipReadabilityStatusV1::Readable);
        let flat = diffuse_override(7, TextureMipReadabilityStatusV1::Flat);
        for overrides in [
            vec![readable.clone(), flat.clone()],
            vec![flat.clone(), readable.clone()],
        ] {
            let representative = representative_override_readability_v1(&overrides);
            assert_eq!(
                representative[&0].status,
                TextureMipReadabilityStatusV1::Flat
            );
            let mut report = passing_report();
            apply_override_contrast_diagnostics_v1(&mut report, &overrides);
            assert_eq!(report.status, ModelSourceQualityStatusV1::Blocked);
            assert!(report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "SOURCE-QUALITY-CONTRAST-FLAT"
                    && diagnostic.path == "diffuseOverrides[materialSlot=7].textureReadability"
            }));
        }
    }
}
