//! Shared material contract between glTF ingestion and Aurora targets.
//!
//! This module plans target resources. It does not write MDL/MTR/TXI bytes;
//! every required bake remains explicit until a later byte-producing stage
//! executes and verifies it.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::glb::{IrMaterial, IrTextureBinding};

pub const AURORA_MATERIAL_IR_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraMaterialTargetProfileV1 {
    AuroraClassicSafe,
    NwnEeMtr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraMaterialCompileStatusV1 {
    Ready,
    ReadyWithWarnings,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraMaterialDispositionV1 {
    Preserved,
    Baked,
    DroppedWithWarning,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraMaterialChannelV1 {
    BaseColorFactor,
    BaseColorTexture,
    MetallicFactor,
    RoughnessFactor,
    MetallicRoughnessTexture,
    NormalTexture,
    EmissiveFactor,
    EmissiveTexture,
    AlphaMode,
    AlphaCutoff,
    DoubleSided,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraAlphaModeV1 {
    Opaque,
    Mask,
    Blend,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraRenderHintV1 {
    Normal,
    NormalAndSpecMapped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraTextureSlotV1 {
    Texture0,
    Texture1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraSpecularBakeOperationV1 {
    MetallicRoughnessToSpecularGloss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraClassicDiffuseBakeOperationV1 {
    PbrEnergyApproximation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraMtrBlendingV1 {
    Punchthrough,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraTargetTextureBindingV1 {
    pub source_texture_id: u32,
    pub tex_coord_set: u32,
    pub target_slot: AuroraTextureSlotV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraSpecularTexturePlanV1 {
    pub operation: AuroraSpecularBakeOperationV1,
    pub source_texture_ids: Vec<u32>,
    pub tex_coord_set: u32,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraClassicDiffuseBakePlanV1 {
    pub operation: AuroraClassicDiffuseBakeOperationV1,
    pub source_texture_ids: Vec<u32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMtrStateV1 {
    pub render_hint: AuroraRenderHintV1,
    pub transparency: bool,
    pub blending: Option<AuroraMtrBlendingV1>,
    pub two_sided: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialIrV1 {
    pub schema_version: u32,
    pub source_material_id: u32,
    pub source_name: Option<String>,
    pub target_profile: AuroraMaterialTargetProfileV1,
    pub diffuse_color: [f32; 4],
    pub ambient_color: [f32; 3],
    pub specular_color: [f32; 3],
    pub shininess: f32,
    pub diffuse_texture: Option<AuroraTargetTextureBindingV1>,
    pub normal_texture: Option<AuroraTargetTextureBindingV1>,
    pub specular_texture_plan: Option<AuroraSpecularTexturePlanV1>,
    pub classic_diffuse_bake_plan: Option<AuroraClassicDiffuseBakePlanV1>,
    pub alpha_mode: AuroraAlphaModeV1,
    pub two_sided: bool,
    pub self_illum_color: [f32; 3],
    pub render_hint: AuroraRenderHintV1,
    pub mtr_required: bool,
    pub mtr: Option<AuroraMtrStateV1>,
    pub txi_required_texture_ids: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialFidelityEntryV1 {
    pub channel: AuroraMaterialChannelV1,
    pub disposition: AuroraMaterialDispositionV1,
    pub target: Option<String>,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialFidelityReportV1 {
    pub schema_version: u32,
    pub source_material_id: u32,
    pub target_profile: AuroraMaterialTargetProfileV1,
    pub status: AuroraMaterialCompileStatusV1,
    pub entries: Vec<AuroraMaterialFidelityEntryV1>,
}

impl AuroraMaterialFidelityReportV1 {
    pub fn disposition(
        &self,
        channel: AuroraMaterialChannelV1,
    ) -> Option<AuroraMaterialDispositionV1> {
        self.entries
            .iter()
            .find(|entry| entry.channel == channel)
            .map(|entry| entry.disposition)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialCompilationV1 {
    pub schema_version: u32,
    pub status: AuroraMaterialCompileStatusV1,
    pub material: AuroraMaterialIrV1,
    pub fidelity: AuroraMaterialFidelityReportV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialCompilationSetV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub target_profile: AuroraMaterialTargetProfileV1,
    pub status: AuroraMaterialCompileStatusV1,
    pub materials: Vec<AuroraMaterialCompilationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialCompileErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for AuroraMaterialCompileErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for AuroraMaterialCompileErrorV1 {}

fn compile_error(
    code: &str,
    path: &str,
    message: impl Into<String>,
) -> AuroraMaterialCompileErrorV1 {
    AuroraMaterialCompileErrorV1 {
        schema_version: AURORA_MATERIAL_IR_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
    }
}

fn entry(
    channel: AuroraMaterialChannelV1,
    disposition: AuroraMaterialDispositionV1,
    target: Option<&str>,
    code: &str,
    message: impl Into<String>,
) -> AuroraMaterialFidelityEntryV1 {
    AuroraMaterialFidelityEntryV1 {
        channel,
        disposition,
        target: target.map(str::to_owned),
        code: code.to_owned(),
        message: message.into(),
    }
}

fn absent(channel: AuroraMaterialChannelV1) -> AuroraMaterialFidelityEntryV1 {
    entry(
        channel,
        AuroraMaterialDispositionV1::Preserved,
        None,
        "AURORA-MATERIAL-SOURCE-CHANNEL-ABSENT",
        "source channel is absent and requires no target resource",
    )
}

fn parse_alpha(value: &str) -> Result<AuroraAlphaModeV1, AuroraMaterialCompileErrorV1> {
    match value {
        "OPAQUE" => Ok(AuroraAlphaModeV1::Opaque),
        "MASK" => Ok(AuroraAlphaModeV1::Mask),
        "BLEND" => Ok(AuroraAlphaModeV1::Blend),
        _ => Err(compile_error(
            "AURORA-MATERIAL-ALPHA-MODE-UNSUPPORTED",
            "material.alphaMode",
            format!("unsupported glTF alpha mode {value:?}"),
        )),
    }
}

fn validate_finite(source: &IrMaterial) -> Result<(), AuroraMaterialCompileErrorV1> {
    let finite = source
        .base_color_factor
        .iter()
        .chain([source.metallic_factor, source.roughness_factor].iter())
        .chain(source.emissive_factor.iter())
        .chain(source.alpha_cutoff.iter())
        .all(|value| value.is_finite());
    if !finite {
        return Err(compile_error(
            "AURORA-MATERIAL-NON-FINITE",
            "material",
            "material factors and alpha cutoff must be finite",
        ));
    }
    if source
        .alpha_cutoff
        .is_some_and(|cutoff| !(0.0..=1.0).contains(&cutoff))
    {
        return Err(compile_error(
            "AURORA-MATERIAL-ALPHA-CUTOFF-RANGE",
            "material.alphaCutoff",
            "alpha cutoff must be in the inclusive range 0..1",
        ));
    }
    Ok(())
}

fn binding(
    source: &IrTextureBinding,
    slot: AuroraTextureSlotV1,
    maximum_uv_set: u32,
) -> Option<AuroraTargetTextureBindingV1> {
    (source.tex_coord_set <= maximum_uv_set).then_some(AuroraTargetTextureBindingV1 {
        source_texture_id: source.texture_id,
        tex_coord_set: source.tex_coord_set,
        target_slot: slot,
    })
}

fn texture_entry(
    channel: AuroraMaterialChannelV1,
    source: Option<&IrTextureBinding>,
    maximum_uv_set: u32,
    target: &str,
) -> AuroraMaterialFidelityEntryV1 {
    match source {
        None => absent(channel),
        Some(source) if source.tex_coord_set <= maximum_uv_set => entry(
            channel,
            AuroraMaterialDispositionV1::Preserved,
            Some(target),
            "AURORA-MATERIAL-TEXTURE-PRESERVED",
            "source texture and UV channel are represented by the target plan",
        ),
        Some(source) => entry(
            channel,
            AuroraMaterialDispositionV1::Blocked,
            None,
            "AURORA-MATERIAL-UV-SET-UNSUPPORTED",
            format!(
                "source texture uses UV set {}, target supports 0..={maximum_uv_set}",
                source.tex_coord_set
            ),
        ),
    }
}

fn status(entries: &[AuroraMaterialFidelityEntryV1]) -> AuroraMaterialCompileStatusV1 {
    if entries
        .iter()
        .any(|item| item.disposition == AuroraMaterialDispositionV1::Blocked)
    {
        AuroraMaterialCompileStatusV1::Blocked
    } else if entries
        .iter()
        .any(|item| item.disposition == AuroraMaterialDispositionV1::DroppedWithWarning)
    {
        AuroraMaterialCompileStatusV1::ReadyWithWarnings
    } else {
        AuroraMaterialCompileStatusV1::Ready
    }
}

/// Creates the target material plan and a complete source-channel ledger.
pub fn compile_gltf_material_v1(
    source: &IrMaterial,
    target_profile: AuroraMaterialTargetProfileV1,
) -> Result<AuroraMaterialCompilationV1, AuroraMaterialCompileErrorV1> {
    validate_finite(source)?;
    let alpha_mode = parse_alpha(&source.alpha_mode)?;
    let maximum_uv_set = match target_profile {
        AuroraMaterialTargetProfileV1::AuroraClassicSafe => 0,
        AuroraMaterialTargetProfileV1::NwnEeMtr => 3,
    };
    let pbr_bake_required = source.metallic_factor != 0.0
        || source.roughness_factor != 1.0
        || source.metallic_roughness_texture.is_some();
    let pbr_target = match target_profile {
        AuroraMaterialTargetProfileV1::AuroraClassicSafe => "diffuseTextureBakePlan",
        AuroraMaterialTargetProfileV1::NwnEeMtr => "texture2SpecularGlossBakePlan",
    };

    let mut entries = Vec::with_capacity(11);
    entries.push(entry(
        AuroraMaterialChannelV1::BaseColorFactor,
        AuroraMaterialDispositionV1::Preserved,
        Some("mdl.diffuse"),
        "AURORA-MATERIAL-BASE-COLOR-FACTOR-PRESERVED",
        "base-color RGB maps to diffuse; OPAQUE alpha normalizes to one",
    ));
    entries.push(texture_entry(
        AuroraMaterialChannelV1::BaseColorTexture,
        source.base_color_texture.as_ref(),
        maximum_uv_set,
        "texture0",
    ));
    for (channel, label) in [
        (AuroraMaterialChannelV1::MetallicFactor, "metallic"),
        (AuroraMaterialChannelV1::RoughnessFactor, "roughness"),
    ] {
        entries.push(if pbr_bake_required {
            entry(
                channel,
                AuroraMaterialDispositionV1::Baked,
                Some(pbr_target),
                "AURORA-MATERIAL-PBR-FACTOR-BAKE-REQUIRED",
                format!("{label} response requires a deterministic target-profile bake"),
            )
        } else {
            entry(
                channel,
                AuroraMaterialDispositionV1::Preserved,
                None,
                "AURORA-MATERIAL-PBR-FACTOR-NOOP",
                format!("source {label} factor requires no target correction"),
            )
        });
    }
    entries.push(match source.metallic_roughness_texture.as_ref() {
        None => absent(AuroraMaterialChannelV1::MetallicRoughnessTexture),
        Some(source) if source.tex_coord_set <= maximum_uv_set => entry(
            AuroraMaterialChannelV1::MetallicRoughnessTexture,
            AuroraMaterialDispositionV1::Baked,
            Some(pbr_target),
            "AURORA-MATERIAL-METALLIC-ROUGHNESS-BAKE-REQUIRED",
            "packed glTF metallic/roughness data requires a target-profile bake",
        ),
        Some(source) => entry(
            AuroraMaterialChannelV1::MetallicRoughnessTexture,
            AuroraMaterialDispositionV1::Blocked,
            None,
            "AURORA-MATERIAL-UV-SET-UNSUPPORTED",
            format!(
                "metallic/roughness texture uses UV set {}, target supports 0..={maximum_uv_set}",
                source.tex_coord_set
            ),
        ),
    });

    let (normal_texture, normal_fidelity) = match target_profile {
        AuroraMaterialTargetProfileV1::AuroraClassicSafe => (
            None,
            if source.normal_texture.is_some() {
                entry(
                    AuroraMaterialChannelV1::NormalTexture,
                    AuroraMaterialDispositionV1::DroppedWithWarning,
                    None,
                    "AURORA-CLASSIC-NORMAL-MAP-UNSUPPORTED",
                    "classic-safe output does not emit MTR normal mapping",
                )
            } else {
                absent(AuroraMaterialChannelV1::NormalTexture)
            },
        ),
        AuroraMaterialTargetProfileV1::NwnEeMtr => (
            source
                .normal_texture
                .as_ref()
                .and_then(|item| binding(item, AuroraTextureSlotV1::Texture1, 3)),
            texture_entry(
                AuroraMaterialChannelV1::NormalTexture,
                source.normal_texture.as_ref(),
                3,
                "mtr.texture1",
            ),
        ),
    };
    entries.push(normal_fidelity);
    entries.push(entry(
        AuroraMaterialChannelV1::EmissiveFactor,
        AuroraMaterialDispositionV1::Preserved,
        Some("mdl.selfillumcolor"),
        "AURORA-MATERIAL-EMISSIVE-FACTOR-PRESERVED",
        "emissive RGB maps to self-illumination color",
    ));
    entries.push(if source.emissive_texture.is_some() {
        entry(
            AuroraMaterialChannelV1::EmissiveTexture,
            AuroraMaterialDispositionV1::DroppedWithWarning,
            None,
            "AURORA-MATERIAL-EMISSIVE-TEXTURE-UNMAPPED",
            "no engine-confirmed automatic emissive texture slot is enabled",
        )
    } else {
        absent(AuroraMaterialChannelV1::EmissiveTexture)
    });

    entries.push(match (target_profile, alpha_mode) {
        (_, AuroraAlphaModeV1::Opaque) => entry(
            AuroraMaterialChannelV1::AlphaMode,
            AuroraMaterialDispositionV1::Preserved,
            Some("opaque"),
            "AURORA-MATERIAL-ALPHA-OPAQUE-PRESERVED",
            "opaque source semantics are preserved",
        ),
        (AuroraMaterialTargetProfileV1::AuroraClassicSafe, _) => entry(
            AuroraMaterialChannelV1::AlphaMode,
            AuroraMaterialDispositionV1::Blocked,
            None,
            "AURORA-CLASSIC-ALPHA-MODE-UNSUPPORTED",
            "MASK and BLEND require an explicit EE MTR transparency profile",
        ),
        (AuroraMaterialTargetProfileV1::NwnEeMtr, AuroraAlphaModeV1::Mask) => entry(
            AuroraMaterialChannelV1::AlphaMode,
            AuroraMaterialDispositionV1::Preserved,
            Some("mtr.blending:PUNCHTHROUGH"),
            "AURORA-MTR-ALPHA-MASK-PRESERVED",
            "MASK maps to MTR punch-through blending",
        ),
        (AuroraMaterialTargetProfileV1::NwnEeMtr, AuroraAlphaModeV1::Blend) => entry(
            AuroraMaterialChannelV1::AlphaMode,
            AuroraMaterialDispositionV1::Preserved,
            Some("mtr.transparency"),
            "AURORA-MTR-ALPHA-BLEND-PRESERVED",
            "BLEND maps to the explicit MTR transparency profile",
        ),
    });
    entries.push(match source.alpha_cutoff {
        None => absent(AuroraMaterialChannelV1::AlphaCutoff),
        Some(_) if target_profile == AuroraMaterialTargetProfileV1::NwnEeMtr => entry(
            AuroraMaterialChannelV1::AlphaCutoff,
            AuroraMaterialDispositionV1::DroppedWithWarning,
            Some("mtr.blending:PUNCHTHROUGH"),
            "AURORA-MTR-ALPHA-CUTOFF-FIXED",
            "punch-through is selected, but the exact glTF cutoff is not represented",
        ),
        Some(_) => entry(
            AuroraMaterialChannelV1::AlphaCutoff,
            AuroraMaterialDispositionV1::Blocked,
            None,
            "AURORA-CLASSIC-ALPHA-CUTOFF-UNSUPPORTED",
            "classic-safe output does not represent alpha cutoff",
        ),
    });
    entries.push(match (target_profile, source.double_sided) {
        (_, false) => entry(
            AuroraMaterialChannelV1::DoubleSided,
            AuroraMaterialDispositionV1::Preserved,
            Some("backfaceCulling:on"),
            "AURORA-MATERIAL-SINGLE-SIDED-PRESERVED",
            "single-sided source semantics are preserved",
        ),
        (AuroraMaterialTargetProfileV1::AuroraClassicSafe, true) => entry(
            AuroraMaterialChannelV1::DoubleSided,
            AuroraMaterialDispositionV1::Blocked,
            None,
            "AURORA-CLASSIC-TWO-SIDED-UNSUPPORTED",
            "double-sided rendering requires EE MTR twosided",
        ),
        (AuroraMaterialTargetProfileV1::NwnEeMtr, true) => entry(
            AuroraMaterialChannelV1::DoubleSided,
            AuroraMaterialDispositionV1::Preserved,
            Some("mtr.twosided"),
            "AURORA-MTR-TWO-SIDED-PRESERVED",
            "double-sided source semantics map to MTR twosided",
        ),
    });

    let diffuse_texture = source
        .base_color_texture
        .as_ref()
        .and_then(|item| binding(item, AuroraTextureSlotV1::Texture0, maximum_uv_set));
    let valid_mr = source
        .metallic_roughness_texture
        .as_ref()
        .filter(|item| item.tex_coord_set <= maximum_uv_set);
    let source_texture_ids = valid_mr
        .map(|item| vec![item.texture_id])
        .unwrap_or_default();
    let specular_texture_plan = (target_profile == AuroraMaterialTargetProfileV1::NwnEeMtr
        && (pbr_bake_required || normal_texture.is_some()))
    .then(|| AuroraSpecularTexturePlanV1 {
        operation: AuroraSpecularBakeOperationV1::MetallicRoughnessToSpecularGloss,
        source_texture_ids: source_texture_ids.clone(),
        tex_coord_set: valid_mr
            .map(|item| item.tex_coord_set)
            .or_else(|| diffuse_texture.as_ref().map(|item| item.tex_coord_set))
            .unwrap_or(0),
        metallic_factor: source.metallic_factor,
        roughness_factor: source.roughness_factor,
    });
    let classic_diffuse_bake_plan =
        (target_profile == AuroraMaterialTargetProfileV1::AuroraClassicSafe && pbr_bake_required)
            .then_some(AuroraClassicDiffuseBakePlanV1 {
                operation: AuroraClassicDiffuseBakeOperationV1::PbrEnergyApproximation,
                source_texture_ids,
                metallic_factor: source.metallic_factor,
                roughness_factor: source.roughness_factor,
            });
    let render_hint = if target_profile == AuroraMaterialTargetProfileV1::NwnEeMtr
        && (normal_texture.is_some() || specular_texture_plan.is_some())
    {
        AuroraRenderHintV1::NormalAndSpecMapped
    } else {
        AuroraRenderHintV1::Normal
    };
    let mtr =
        (target_profile == AuroraMaterialTargetProfileV1::NwnEeMtr).then_some(AuroraMtrStateV1 {
            render_hint,
            transparency: alpha_mode == AuroraAlphaModeV1::Blend,
            blending: match alpha_mode {
                AuroraAlphaModeV1::Opaque | AuroraAlphaModeV1::Blend => None,
                AuroraAlphaModeV1::Mask => Some(AuroraMtrBlendingV1::Punchthrough),
            },
            two_sided: source.double_sided,
        });
    let mut txi_required_texture_ids = normal_texture
        .iter()
        .map(|item| item.source_texture_id)
        .collect::<Vec<_>>();
    txi_required_texture_ids.sort_unstable();
    txi_required_texture_ids.dedup();
    let compile_status = status(&entries);
    let material = AuroraMaterialIrV1 {
        schema_version: AURORA_MATERIAL_IR_SCHEMA_VERSION_V1,
        source_material_id: source.id,
        source_name: source.name.clone(),
        target_profile,
        diffuse_color: [
            source.base_color_factor[0],
            source.base_color_factor[1],
            source.base_color_factor[2],
            if alpha_mode == AuroraAlphaModeV1::Opaque {
                1.0
            } else {
                source.base_color_factor[3]
            },
        ],
        ambient_color: [1.0; 3],
        specular_color: [0.0; 3],
        shininess: 1.0,
        diffuse_texture,
        normal_texture,
        specular_texture_plan,
        classic_diffuse_bake_plan,
        alpha_mode,
        two_sided: target_profile == AuroraMaterialTargetProfileV1::NwnEeMtr && source.double_sided,
        self_illum_color: source.emissive_factor,
        render_hint,
        mtr_required: target_profile == AuroraMaterialTargetProfileV1::NwnEeMtr,
        mtr,
        txi_required_texture_ids,
    };
    let fidelity = AuroraMaterialFidelityReportV1 {
        schema_version: AURORA_MATERIAL_IR_SCHEMA_VERSION_V1,
        source_material_id: source.id,
        target_profile,
        status: compile_status,
        entries,
    };
    Ok(AuroraMaterialCompilationV1 {
        schema_version: AURORA_MATERIAL_IR_SCHEMA_VERSION_V1,
        status: compile_status,
        material,
        fidelity,
    })
}

/// Compiles the complete material table for one immutable GLB identity.
pub fn compile_gltf_materials_v1(
    source_sha256: &str,
    source_materials: &[IrMaterial],
    target_profile: AuroraMaterialTargetProfileV1,
) -> Result<AuroraMaterialCompilationSetV1, AuroraMaterialCompileErrorV1> {
    if source_sha256.len() != 64
        || !source_sha256
            .bytes()
            .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
    {
        return Err(compile_error(
            "AURORA-MATERIAL-SOURCE-SHA256-INVALID",
            "sourceSha256",
            "source identity must be an exact lowercase SHA-256",
        ));
    }
    let mut materials = Vec::with_capacity(source_materials.len());
    for source in source_materials {
        let mut compiled =
            compile_gltf_material_v1(source, target_profile).map_err(|mut error| {
                error.path = format!("materials[{}].{}", source.id, error.path);
                error
            })?;
        compiled.fidelity.source_material_id = source.id;
        materials.push(compiled);
    }
    let compile_status = if materials
        .iter()
        .any(|item| item.status == AuroraMaterialCompileStatusV1::Blocked)
    {
        AuroraMaterialCompileStatusV1::Blocked
    } else if materials
        .iter()
        .any(|item| item.status == AuroraMaterialCompileStatusV1::ReadyWithWarnings)
    {
        AuroraMaterialCompileStatusV1::ReadyWithWarnings
    } else {
        AuroraMaterialCompileStatusV1::Ready
    };
    Ok(AuroraMaterialCompilationSetV1 {
        schema_version: AURORA_MATERIAL_IR_SCHEMA_VERSION_V1,
        source_sha256: source_sha256.to_owned(),
        target_profile,
        status: compile_status,
        materials,
    })
}
