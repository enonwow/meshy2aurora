//! Model-kind-neutral intermediate representation shared by every Aurora MDL
//! route.
//!
//! Creature, placeable and tile resolvers have different 2DA/GFF packaging
//! contracts, but Aurora ultimately sends their model resources through the
//! same binary MDL loader.  These types therefore describe only model
//! geometry, hierarchy and source-material bindings.  Domain-specific
//! metadata belongs in the corresponding resolver/package module.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuroraSegmentDeformationV1 {
    Skin,
    Rigid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraMaterialSourceBindingV1 {
    pub slot: u32,
    pub source_material_id: Option<u32>,
    pub source_material_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraModelIrV1 {
    pub schema_version: u32,
    pub profile_id: String,
    pub source_sha256: String,
    pub basis_status: String,
    pub engine_facing_proof: String,
    pub uv_runtime_proof: String,
    pub nodes: Vec<AuroraModelNodeV1>,
    pub material_source_bindings: Vec<AuroraMaterialSourceBindingV1>,
    pub segments: Vec<AuroraModelSegmentV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraModelNodeV1 {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub bind_local_matrix: [f32; 16],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraModelSegmentV1 {
    pub segment_id: u32,
    pub material_slot: u32,
    pub deformation: AuroraSegmentDeformationV1,
    pub parent_node_id: u32,
    /// Controls Aurora's mesh-level shadow participation flag.
    ///
    /// It defaults to `true` for every pre-authoring caller. Placeable
    /// authoring may split material buckets so individual elements can opt out
    /// without changing render or collision participation.
    #[serde(
        default = "default_cast_shadow",
        skip_serializing_if = "cast_shadow_is_enabled"
    )]
    pub cast_shadow: bool,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Option<Vec<[f32; 4]>>,
    pub uv0: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Optional Aurora walkmesh surface identifier for every triangle.
    ///
    /// Render models leave this empty and the shared writer emits surface
    /// `0`. PWK/WOK/DWK routes provide exactly one value per triangle.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub face_surface_ids: Vec<i32>,
    pub weights: Vec<AuroraVertexWeightsV1>,
}

fn default_cast_shadow() -> bool {
    true
}

fn cast_shadow_is_enabled(value: &bool) -> bool {
    *value
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuroraVertexWeightsV1 {
    pub bone_node_ids: [Option<u32>; 4],
    pub values: [f32; 4],
    pub influence_count: u8,
}
