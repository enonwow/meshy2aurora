use serde::Serialize;

/// Product guardrails. These values are not claims about Aurora engine limits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParserLimits {
    pub max_input_bytes: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_diagnostics: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_nodes: 65_536,
            max_depth: 256,
            max_diagnostics: 1_024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionReport {
    pub schema_version: u32,
    pub format: String,
    pub byte_length: usize,
    pub file_header: FileHeaderReport,
    pub model: ModelReport,
    pub node_tree: NodeTreeReport,
    pub unsupported: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHeaderReport {
    pub binary_mdl_id: u32,
    pub mdx_start: u32,
    pub mdx_size: u32,
    pub mdx_range_in_bounds: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelReport {
    pub name: String,
    pub root_node_offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeTreeReport {
    pub node_count: usize,
    pub max_depth: usize,
    pub roots: Vec<NodeReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeReport {
    pub offset: u32,
    pub number: u32,
    pub name: String,
    pub parent_offset: Option<u32>,
    pub content_flags: u32,
    pub children: Vec<NodeReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub offset: usize,
    pub context: String,
}

impl Diagnostic {
    pub(crate) fn unsupported(offset: usize, context: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            code: "M2A-MDL-UNSUPPORTED".to_owned(),
            severity: "warning".to_owned(),
            offset,
            context: context.into(),
        }
    }
}
