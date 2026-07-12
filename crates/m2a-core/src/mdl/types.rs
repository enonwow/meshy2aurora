use serde::Serialize;

/// Product guardrails. These values are not claims about Aurora engine limits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParserLimits {
    pub max_input_bytes: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_diagnostics: usize,
    pub max_skin_bone_count: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_nodes: 65_536,
            max_depth: 256,
            max_diagnostics: 1_024,
            max_skin_bone_count: 4_096,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionReport {
    pub schema_version: u32,
    pub format: String,
    pub byte_length: usize,
    pub file_header: FileHeaderReport,
    pub model: ModelReport,
    pub node_tree: NodeTreeReport,
    pub animations: Vec<AnimationReport>,
    pub unsupported: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteRangeReport {
    pub start: usize,
    pub length: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHeaderReport {
    pub binary_mdl_id: u32,
    pub mdx_start: u32,
    pub mdx_size: u32,
    pub mdx_range_in_bounds: bool,
    pub core_range: ByteRangeReport,
    pub raw_range: ByteRangeReport,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelReport {
    pub name: String,
    pub root_node_offset: u32,
    pub geometry_type: u32,
    pub classification: u8,
    pub fog: u8,
    pub child_model_count: u32,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub radius: f32,
    pub animation_scale: f32,
    pub supermodel_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeTreeReport {
    pub declared_node_count: usize,
    pub node_count: usize,
    pub max_depth: usize,
    pub roots: Vec<NodeReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeReport {
    pub offset: u32,
    pub number: u32,
    pub name: String,
    pub parent_offset: Option<u32>,
    pub inherit_color: u32,
    pub content_flags: u32,
    pub unsupported_families: Vec<String>,
    pub controllers: Vec<ControllerReport>,
    pub mesh: Option<MeshReport>,
    pub skin: Option<SkinReport>,
    pub children: Vec<NodeReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerReport {
    pub controller_type: i32,
    pub controller_name: Option<String>,
    pub row_count: usize,
    pub time_index: usize,
    pub data_index: usize,
    pub column_count: usize,
    pub times: Vec<f32>,
    pub values: Vec<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshReport {
    pub textures: Vec<String>,
    pub vertex_count: usize,
    pub texture_count: usize,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub radius: f32,
    pub average: Option<Vec3>,
    pub diffuse: [f32; 3],
    pub ambient: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub shadow: u32,
    pub beaming: u32,
    pub render: u32,
    pub transparency: u32,
    pub render_hint: u32,
    pub tile_fade: u32,
    pub mesh_type: u32,
    pub start_mdx: i32,
    pub faces: Vec<FaceReport>,
    pub index_counts: Vec<u32>,
    pub raw_index_offsets: Vec<i32>,
    pub raw_indices: Vec<Vec<u16>>,
    pub vertices: Vec<Vec3>,
    pub uv0: Vec<Vec2>,
    pub normals: Vec<Vec3>,
    pub validated_raw_pointers: Vec<RawPointerReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaceReport {
    pub normal: Vec3,
    pub distance: f32,
    pub surface_id: i32,
    pub adjacent_faces: [i16; 3],
    pub vertex_indices: [u16; 3],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPointerReport {
    pub field: String,
    pub pointer: Option<i32>,
    pub validated_length: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkinVariant {
    Legacy17,
    Extended64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinReport {
    pub variant: SkinVariant,
    pub node_offset: u32,
    pub header_size: usize,
    pub node_to_bone_pointer: u32,
    pub raw_weights_pointer: i32,
    pub raw_refs_pointer: i32,
    pub weights_header: ArrayReport,
    pub q_header: ArrayReport,
    pub t_header: ArrayReport,
    pub constants_header: ArrayReport,
    pub node_to_bone_map: Vec<i16>,
    pub inverse_bone_rotations_raw: Vec<[f32; 4]>,
    pub inverse_bone_translations: Vec<Vec3>,
    pub bone_constants: Vec<[i16; 2]>,
    pub inline_mapping: Vec<i16>,
    pub vertex_weights: Vec<[f32; 4]>,
    pub bone_references: Vec<[u16; 4]>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayReport {
    pub pointer: u32,
    pub used: usize,
    pub allocated: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationReport {
    pub offset: u32,
    pub name: String,
    pub length: f32,
    pub transition: f32,
    pub animation_root: String,
    pub events: Vec<AnimationEventReport>,
    pub node_tree: NodeTreeReport,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationEventReport {
    pub time: f32,
    pub name: String,
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
    fn warning(code: &str, offset: usize, context: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            code: code.to_owned(),
            severity: "warning".to_owned(),
            offset,
            context: context.into(),
        }
    }

    pub(crate) fn unsupported_family(offset: usize, context: impl Into<String>) -> Self {
        Self::warning("M2A-MDL-UNSUPPORTED-NODE-FAMILY", offset, context)
    }

    pub(crate) fn unknown_controller(offset: usize, context: impl Into<String>) -> Self {
        Self::warning("M2A-MDL-CONTROLLER-TYPE-UNKNOWN", offset, context)
    }

    pub(crate) fn unknown_node_flags(offset: usize, context: impl Into<String>) -> Self {
        Self::warning("M2A-MDL-NODE-FLAGS-UNKNOWN", offset, context)
    }
}
