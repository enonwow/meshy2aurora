//! Target-supplied Material Separation capabilities.
//!
//! Studio consumes this contract instead of carrying historical Creature or
//! Placeable limits in React. `ModelPart` is the neutral adapter boundary for
//! future Item parts; it is not an Item product target.

use std::fmt;

use serde::{Deserialize, Serialize};

pub const MODEL_MATERIAL_CAPABILITIES_SCHEMA_VERSION_V1: u32 = 1;
pub const MODEL_MATERIAL_CAPABILITIES_SCHEMA_VERSION_V2: u32 = 2;
pub const AURORA_WRITER_MAX_MATERIAL_SLOTS_V1: u32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelRenderTargetV1 {
    Creature,
    Placeable,
    Tile,
    ModelPart,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialCapabilitiesV1 {
    pub schema_version: u32,
    pub target: ModelRenderTargetV1,
    pub material_separation_supported: bool,
    pub max_material_slots: u32,
    pub max_output_sections: u32,
    pub selection_granularity: String,
    pub face_selection_supported: bool,
    pub automatic_material_inference: bool,
    pub preserves_source_uv0: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialCapabilityErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelMaterialCapabilityErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelMaterialCapabilityErrorV1 {}

fn capability_error(code: &str, path: &str, message: String) -> ModelMaterialCapabilityErrorV1 {
    ModelMaterialCapabilityErrorV1 {
        schema_version: MODEL_MATERIAL_CAPABILITIES_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.to_owned(),
        message,
    }
}

pub fn material_separation_capabilities_v1(
    target: ModelRenderTargetV1,
) -> ModelMaterialCapabilitiesV1 {
    ModelMaterialCapabilitiesV1 {
        schema_version: MODEL_MATERIAL_CAPABILITIES_SCHEMA_VERSION_V1,
        target,
        material_separation_supported: true,
        max_material_slots: AURORA_WRITER_MAX_MATERIAL_SLOTS_V1,
        max_output_sections: 4_096,
        selection_granularity: "CONNECTED_COMPONENTS".to_owned(),
        face_selection_supported: false,
        automatic_material_inference: false,
        preserves_source_uv0: true,
    }
}

pub fn material_separation_capabilities_v2(
    target: ModelRenderTargetV1,
) -> ModelMaterialCapabilitiesV1 {
    ModelMaterialCapabilitiesV1 {
        schema_version: MODEL_MATERIAL_CAPABILITIES_SCHEMA_VERSION_V2,
        target,
        material_separation_supported: true,
        max_material_slots: AURORA_WRITER_MAX_MATERIAL_SLOTS_V1,
        max_output_sections: 4_096,
        selection_granularity: "CONNECTED_COMPONENTS_AND_FACES".to_owned(),
        face_selection_supported: true,
        automatic_material_inference: false,
        preserves_source_uv0: true,
    }
}

pub fn validate_material_separation_counts_v2(
    target: ModelRenderTargetV1,
    material_slot_count: usize,
    output_section_count: u32,
) -> Result<ModelMaterialCapabilitiesV1, ModelMaterialCapabilityErrorV1> {
    let capabilities = material_separation_capabilities_v2(target);
    if material_slot_count > capabilities.max_material_slots as usize {
        return Err(capability_error(
            "MODEL-MATERIAL-SLOT-BUDGET-EXCEEDED",
            "report.materialSlots",
            format!(
                "resolved {material_slot_count} material slots exceed the target limit {}",
                capabilities.max_material_slots
            ),
        ));
    }
    if output_section_count > capabilities.max_output_sections {
        return Err(capability_error(
            "MODEL-MATERIAL-SECTION-BUDGET-EXCEEDED",
            "report.outputSectionCount",
            format!(
                "resolved {output_section_count} output sections exceed the target limit {}",
                capabilities.max_output_sections
            ),
        ));
    }
    Ok(capabilities)
}

pub fn validate_material_separation_counts_v1(
    target: ModelRenderTargetV1,
    material_slot_count: usize,
    output_section_count: u32,
) -> Result<ModelMaterialCapabilitiesV1, ModelMaterialCapabilityErrorV1> {
    let capabilities = material_separation_capabilities_v1(target);
    if material_slot_count > capabilities.max_material_slots as usize {
        return Err(capability_error(
            "MODEL-MATERIAL-SLOT-BUDGET-EXCEEDED",
            "report.materialSlots",
            format!(
                "resolved {material_slot_count} material slots exceed the target limit {}",
                capabilities.max_material_slots
            ),
        ));
    }
    if output_section_count > capabilities.max_output_sections {
        return Err(capability_error(
            "MODEL-MATERIAL-SECTION-BUDGET-EXCEEDED",
            "report.outputSectionCount",
            format!(
                "resolved {output_section_count} output sections exceed the target limit {}",
                capabilities.max_output_sections
            ),
        ));
    }
    Ok(capabilities)
}
