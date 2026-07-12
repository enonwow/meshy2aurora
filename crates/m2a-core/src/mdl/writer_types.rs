use std::fmt;

use serde::{Deserialize, Serialize};

use super::InspectionReport;

pub const M4_WRITER_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdlFormatProfileV1 {
    M4DirectCreatureExtended64V1,
    Legacy17V1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlMaterialTextureBindingV1 {
    pub material_slot: u32,
    pub resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlWriterOptionsV1 {
    pub schema_version: u32,
    pub format_profile: MdlFormatProfileV1,
    pub model_resource_resref: String,
    pub diffuse_texture_resref_by_material_slot: Vec<MdlMaterialTextureBindingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlWriteError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl MdlWriteError {
    pub(crate) fn fatal(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: M4_WRITER_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for MdlWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for MdlWriteError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlRigNodeLayoutV1 {
    pub ir_node_id: u32,
    pub part_number: u32,
    pub core_offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlMeshNodeLayoutV1 {
    pub segment_id: u32,
    pub part_number: u32,
    pub core_offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlLayoutReportV1 {
    pub core_length: usize,
    pub raw_length: usize,
    pub file_length: usize,
    pub rig_nodes: Vec<MdlRigNodeLayoutV1>,
    pub mesh_nodes: Vec<MdlMeshNodeLayoutV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlWriterDeviationV1 {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M4SemanticProjectionV1 {
    pub model_resource_resref: String,
    pub animation_count: usize,
    pub rig_node_count: usize,
    pub mesh_node_count: usize,
    pub triangle_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlWriterReportV1 {
    pub schema_version: u32,
    pub format_profile: MdlFormatProfileV1,
    pub payload_sha256: String,
    pub layout: MdlLayoutReportV1,
    pub projection: M4SemanticProjectionV1,
    pub semantic_diff: Vec<String>,
    pub deviations: Vec<MdlWriterDeviationV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryMdlArtifactV1 {
    pub payload: Vec<u8>,
    pub inspection: InspectionReport,
    pub report: MdlWriterReportV1,
}
