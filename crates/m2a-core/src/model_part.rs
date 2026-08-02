//! Target-neutral Material Separation boundary for one future assembled model part.
//!
//! The Item pipeline can resolve Bottom/Middle/Top or armor parts independently
//! through this adapter without importing Creature, Placeable, Tile or Item
//! packaging into the shared material core.

use std::fmt;

use serde::Serialize;

use crate::{
    glb::AuroraAssetIr,
    model_material_capabilities::{
        ModelMaterialCapabilitiesV1, ModelMaterialCapabilityErrorV1, ModelRenderTargetV1,
        validate_material_separation_counts_v1,
    },
    model_material_separation::{
        ModelMaterialSeparationDocumentV1, ModelMaterialSeparationErrorV1,
        ResolvedModelMaterialsV1, resolve_model_materials_v1,
    },
};

pub const MODEL_PART_INPUT_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct ModelPartInputV1<'a> {
    pub schema_version: u32,
    pub part_id: &'a str,
    pub source_ir: &'a AuroraAssetIr,
    pub material_separation: &'a ModelMaterialSeparationDocumentV1,
}

#[derive(Clone, Debug)]
pub struct ResolvedModelPartInputV1 {
    pub schema_version: u32,
    pub part_id: String,
    pub capabilities: ModelMaterialCapabilitiesV1,
    pub materials: ResolvedModelMaterialsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPartInputErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelPartInputErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelPartInputErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> ModelPartInputErrorV1 {
    ModelPartInputErrorV1 {
        schema_version: MODEL_PART_INPUT_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn material_error(value: ModelMaterialSeparationErrorV1) -> ModelPartInputErrorV1 {
    error(value.code.as_str(), value.path, value.message)
}

fn capability_error(value: ModelMaterialCapabilityErrorV1) -> ModelPartInputErrorV1 {
    error(value.code.as_str(), value.path, value.message)
}

fn valid_part_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.'))
}

pub fn resolve_model_part_input_v1(
    input: ModelPartInputV1<'_>,
) -> Result<ResolvedModelPartInputV1, ModelPartInputErrorV1> {
    if input.schema_version != MODEL_PART_INPUT_SCHEMA_VERSION_V1 {
        return Err(error(
            "MODEL-PART-INPUT-SCHEMA-UNSUPPORTED",
            "schemaVersion",
            "only ModelPart input schema version 1 is supported",
        ));
    }
    if !valid_part_id(input.part_id) {
        return Err(error(
            "MODEL-PART-INPUT-ID-INVALID",
            "partId",
            "part id must be 1..64 ASCII alphanumeric, colon, underscore, dash or dot bytes",
        ));
    }

    let materials = resolve_model_materials_v1(input.source_ir, input.material_separation)
        .map_err(material_error)?;
    let capabilities = validate_material_separation_counts_v1(
        ModelRenderTargetV1::ModelPart,
        materials.report.material_slots.len(),
        materials.report.output_section_count,
    )
    .map_err(capability_error)?;

    Ok(ResolvedModelPartInputV1 {
        schema_version: MODEL_PART_INPUT_SCHEMA_VERSION_V1,
        part_id: input.part_id.to_owned(),
        capabilities,
        materials,
    })
}
