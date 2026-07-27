//! Deterministic element authoring for static placeables.
//!
//! The browser and native package builder consume the same versioned document.
//! Authoring is applied to the source-preserving GLB IR before Profile A, so
//! viewport edits cannot diverge from MDL, bounds, shadow geometry or PWK
//! inputs. This module deliberately edits objects/components, not individual
//! vertices, edges or faces.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::glb::{AuroraAssetIr, GlbIngestResult, IrMesh, IrNode, IrPrimitive, IrTransform};

pub const PLACEABLE_AUTHORING_SCHEMA_VERSION_V1: u32 = 1;
pub const SHADOWLESS_MATERIAL_SUFFIX_V1: &str = "__m2a_no_shadow";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceableElementKindV1 {
    SourceNode,
    SourceComponent,
    Copy,
    Group,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceableAuthoringProjectionV1 {
    Render,
    Collision,
    Shadow,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableElementSourceV1 {
    pub node_id: u32,
    pub primitive_id: Option<u32>,
    pub component_index: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableElementTransformV1 {
    /// Additional translation in source model space.
    pub translation: [f32; 3],
    /// Additional rotation as an XYZW quaternion.
    pub rotation_xyzw: [f32; 4],
    /// Per-axis scale. Uniform scale remains the safe default.
    pub scale: [f32; 3],
    /// Pivot expressed in source model space.
    pub pivot: [f32; 3],
}

impl Default for PlaceableElementTransformV1 {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
            pivot: [0.0; 3],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableElementFlagsV1 {
    /// Viewport-only state. Hidden elements still participate in output.
    pub hidden: bool,
    /// Viewport-only state. Locked elements cannot be manipulated by the UI.
    pub locked: bool,
    pub renderable: bool,
    pub include_in_collision: bool,
    pub cast_shadow: bool,
}

impl Default for PlaceableElementFlagsV1 {
    fn default() -> Self {
        Self {
            hidden: false,
            locked: false,
            renderable: true,
            include_in_collision: true,
            cast_shadow: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableAuthoringElementV1 {
    pub id: String,
    pub name: String,
    pub kind: PlaceableElementKindV1,
    pub source: Option<PlaceableElementSourceV1>,
    pub parent_id: Option<String>,
    pub transform: PlaceableElementTransformV1,
    pub flags: PlaceableElementFlagsV1,
    pub deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableAuthoringDocumentV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub elements: Vec<PlaceableAuthoringElementV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableComponentInspectionV1 {
    pub element_id: String,
    pub component_index: u32,
    pub triangle_count: u32,
    pub vertex_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceablePrimitiveInspectionV1 {
    pub primitive_id: u32,
    pub material_id: Option<u32>,
    pub triangle_count: u32,
    pub components: Vec<PlaceableComponentInspectionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableNodeInspectionV1 {
    pub element_id: String,
    pub node_id: u32,
    pub name: String,
    pub mesh_id: u32,
    pub primitives: Vec<PlaceablePrimitiveInspectionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableElementInspectionV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub render_node_count: u32,
    pub primitive_count: u32,
    pub connected_component_count: u32,
    pub nodes: Vec<PlaceableNodeInspectionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableAuthoringApplyReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub authoring_sha256: String,
    pub source_triangle_count: u32,
    pub output_triangle_count: u32,
    pub renderable_element_count: u32,
    pub collision_element_count: u32,
    pub shadow_element_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub collision_bounds_min: Option<[f32; 3]>,
    pub collision_bounds_max: Option<[f32; 3]>,
    pub shadow_bounds_min: Option<[f32; 3]>,
    pub shadow_bounds_max: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableAuthoringErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for PlaceableAuthoringErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for PlaceableAuthoringErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> PlaceableAuthoringErrorV1 {
    PlaceableAuthoringErrorV1 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

#[derive(Clone, Debug)]
struct ConnectedComponent {
    triangle_indices: Vec<usize>,
    vertex_indices: BTreeSet<u32>,
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: [f32; 3],
    max: [f32; 3],
    populated: bool,
}

impl Bounds {
    fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
            populated: false,
        }
    }

    fn include(&mut self, point: [f32; 3]) {
        for (axis, value) in point.into_iter().enumerate() {
            self.min[axis] = self.min[axis].min(value);
            self.max[axis] = self.max[axis].max(value);
        }
        self.populated = true;
    }

    fn required(self, path: &str) -> Result<([f32; 3], [f32; 3]), PlaceableAuthoringErrorV1> {
        self.populated
            .then_some((self.min, self.max))
            .ok_or_else(|| {
                error(
                    "PLACEABLE-AUTHORING-EMPTY",
                    path,
                    "authoring removed every renderable triangle",
                )
            })
    }

    fn optional(self) -> (Option<[f32; 3]>, Option<[f32; 3]>) {
        if self.populated {
            (Some(self.min), Some(self.max))
        } else {
            (None, None)
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Mat4([f32; 16]);

impl Mat4 {
    fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    fn translation(value: [f32; 3]) -> Self {
        let mut matrix = Self::identity();
        matrix.0[12] = value[0];
        matrix.0[13] = value[1];
        matrix.0[14] = value[2];
        matrix
    }

    fn scale(value: [f32; 3]) -> Self {
        Self([
            value[0], 0.0, 0.0, 0.0, //
            0.0, value[1], 0.0, 0.0, //
            0.0, 0.0, value[2], 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    fn rotation_xyzw(value: [f32; 4]) -> Self {
        let [x, y, z, w] = value;
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;
        Self([
            1.0 - 2.0 * (yy + zz),
            2.0 * (xy + wz),
            2.0 * (xz - wy),
            0.0,
            2.0 * (xy - wz),
            1.0 - 2.0 * (xx + zz),
            2.0 * (yz + wx),
            0.0,
            2.0 * (xz + wy),
            2.0 * (yz - wx),
            1.0 - 2.0 * (xx + yy),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    fn mul(self, rhs: Self) -> Self {
        let mut output = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                output[column * 4 + row] = (0..4)
                    .map(|index| self.0[index * 4 + row] * rhs.0[column * 4 + index])
                    .sum();
            }
        }
        Self(output)
    }

    fn transform_point(self, point: [f32; 3]) -> [f32; 3] {
        [
            self.0[0] * point[0] + self.0[4] * point[1] + self.0[8] * point[2] + self.0[12],
            self.0[1] * point[0] + self.0[5] * point[1] + self.0[9] * point[2] + self.0[13],
            self.0[2] * point[0] + self.0[6] * point[1] + self.0[10] * point[2] + self.0[14],
        ]
    }

    fn linear(self) -> [[f32; 3]; 3] {
        [
            [self.0[0], self.0[4], self.0[8]],
            [self.0[1], self.0[5], self.0[9]],
            [self.0[2], self.0[6], self.0[10]],
        ]
    }

    fn determinant(self) -> f32 {
        determinant3(self.linear())
    }

    fn inverse_affine(self) -> Option<Self> {
        let inverse = inverse3(self.linear())?;
        let translation = [self.0[12], self.0[13], self.0[14]];
        let inverse_translation = mul3(inverse, translation).map(|value| -value);
        Some(Self([
            inverse[0][0],
            inverse[1][0],
            inverse[2][0],
            0.0,
            inverse[0][1],
            inverse[1][1],
            inverse[2][1],
            0.0,
            inverse[0][2],
            inverse[1][2],
            inverse[2][2],
            0.0,
            inverse_translation[0],
            inverse_translation[1],
            inverse_translation[2],
            1.0,
        ]))
    }

    fn normal_matrix(self) -> Option<[[f32; 3]; 3]> {
        inverse3(self.linear()).map(transpose3)
    }
}

fn determinant3(matrix: [[f32; 3]; 3]) -> f32 {
    matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0])
}

fn inverse3(matrix: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    let determinant = determinant3(matrix);
    if !determinant.is_finite() || determinant.abs() <= 1.0e-8 {
        return None;
    }
    let inverse = determinant.recip();
    Some([
        [
            (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1]) * inverse,
            (matrix[0][2] * matrix[2][1] - matrix[0][1] * matrix[2][2]) * inverse,
            (matrix[0][1] * matrix[1][2] - matrix[0][2] * matrix[1][1]) * inverse,
        ],
        [
            (matrix[1][2] * matrix[2][0] - matrix[1][0] * matrix[2][2]) * inverse,
            (matrix[0][0] * matrix[2][2] - matrix[0][2] * matrix[2][0]) * inverse,
            (matrix[0][2] * matrix[1][0] - matrix[0][0] * matrix[1][2]) * inverse,
        ],
        [
            (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]) * inverse,
            (matrix[0][1] * matrix[2][0] - matrix[0][0] * matrix[2][1]) * inverse,
            (matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]) * inverse,
        ],
    ])
}

fn transpose3(matrix: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [matrix[0][0], matrix[1][0], matrix[2][0]],
        [matrix[0][1], matrix[1][1], matrix[2][1]],
        [matrix[0][2], matrix[1][2], matrix[2][2]],
    ]
}

fn mul3(matrix: [[f32; 3]; 3], value: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * value[0] + matrix[0][1] * value[1] + matrix[0][2] * value[2],
        matrix[1][0] * value[0] + matrix[1][1] * value[1] + matrix[1][2] * value[2],
        matrix[2][0] * value[0] + matrix[2][1] * value[1] + matrix[2][2] * value[2],
    ]
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn normalize3(value: [f32; 3], path: &str) -> Result<[f32; 3], PlaceableAuthoringErrorV1> {
    let length_squared = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>();
    if !length_squared.is_finite() || length_squared <= 1.0e-12 {
        return Err(error(
            "PLACEABLE-AUTHORING-NORMAL-INVALID",
            path,
            "transformed direction is zero or non-finite",
        ));
    }
    let inverse = length_squared.sqrt().recip();
    Ok(value.map(|component| component * inverse))
}

fn matrix_from_ir_transform(
    transform: &IrTransform,
    path: &str,
) -> Result<Mat4, PlaceableAuthoringErrorV1> {
    if transform.kind == "MATRIX" {
        let matrix = transform.matrix.ok_or_else(|| {
            error(
                "PLACEABLE-AUTHORING-TRANSFORM-INVALID",
                path,
                "MATRIX transform has no matrix",
            )
        })?;
        if matrix.iter().all(|value| value.is_finite()) {
            return Ok(Mat4(matrix));
        }
    } else if transform.kind == "TRS" {
        let translation = transform.translation.unwrap_or([0.0; 3]);
        let mut rotation = transform.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let scale = transform.scale.unwrap_or([1.0; 3]);
        if translation
            .iter()
            .chain(rotation.iter())
            .chain(scale.iter())
            .all(|value| value.is_finite())
        {
            let length = rotation
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt();
            if length > 1.0e-8 {
                rotation.iter_mut().for_each(|value| *value /= length);
                return Ok(Mat4::translation(translation)
                    .mul(Mat4::rotation_xyzw(rotation))
                    .mul(Mat4::scale(scale)));
            }
        }
    }
    Err(error(
        "PLACEABLE-AUTHORING-TRANSFORM-INVALID",
        path,
        "source node transform is malformed, singular or non-finite",
    ))
}

fn matrix_from_authoring_transform(
    transform: &PlaceableElementTransformV1,
    path: &str,
) -> Result<Mat4, PlaceableAuthoringErrorV1> {
    if !transform
        .translation
        .iter()
        .chain(transform.rotation_xyzw.iter())
        .chain(transform.scale.iter())
        .chain(transform.pivot.iter())
        .all(|value| value.is_finite())
        || transform.scale.iter().any(|value| value.abs() <= 1.0e-6)
    {
        return Err(error(
            "PLACEABLE-AUTHORING-TRANSFORM-INVALID",
            path,
            "translation, rotation, scale and pivot must be finite; scale cannot be zero",
        ));
    }
    let mut rotation = transform.rotation_xyzw;
    let length = rotation
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if !length.is_finite() || length <= 1.0e-8 {
        return Err(error(
            "PLACEABLE-AUTHORING-TRANSFORM-INVALID",
            path,
            "rotation must be a finite nonzero XYZW quaternion",
        ));
    }
    rotation.iter_mut().for_each(|value| *value /= length);
    let negative_pivot = transform.pivot.map(|value| -value);
    Ok(Mat4::translation(transform.translation)
        .mul(Mat4::translation(transform.pivot))
        .mul(Mat4::rotation_xyzw(rotation))
        .mul(Mat4::scale(transform.scale))
        .mul(Mat4::translation(negative_pivot)))
}

fn connected_components(
    primitive: &IrPrimitive,
    path: &str,
) -> Result<Vec<ConnectedComponent>, PlaceableAuthoringErrorV1> {
    if primitive.topology != "TRIANGLES" || !primitive.indices.len().is_multiple_of(3) {
        return Err(error(
            "PLACEABLE-AUTHORING-TOPOLOGY-UNSUPPORTED",
            path,
            "element editing requires indexed TRIANGLES",
        ));
    }
    if primitive.indices.iter().any(|index| {
        usize::try_from(*index)
            .ok()
            .is_none_or(|index| index >= primitive.positions.len())
    }) {
        return Err(error(
            "PLACEABLE-AUTHORING-INDEX-OOB",
            path,
            "primitive index exceeds the position array",
        ));
    }
    let triangle_count = primitive.indices.len() / 3;
    let mut vertex_triangles = BTreeMap::<u32, Vec<usize>>::new();
    for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
        for index in triangle {
            vertex_triangles
                .entry(*index)
                .or_default()
                .push(triangle_index);
        }
    }
    let mut visited = vec![false; triangle_count];
    let mut output = Vec::new();
    for seed in 0..triangle_count {
        if visited[seed] {
            continue;
        }
        let mut pending = vec![seed];
        let mut triangles = Vec::new();
        let mut vertices = BTreeSet::new();
        visited[seed] = true;
        while let Some(triangle_index) = pending.pop() {
            triangles.push(triangle_index);
            for index in &primitive.indices[triangle_index * 3..triangle_index * 3 + 3] {
                vertices.insert(*index);
                if let Some(neighbors) = vertex_triangles.get(index) {
                    for neighbor in neighbors {
                        if !visited[*neighbor] {
                            visited[*neighbor] = true;
                            pending.push(*neighbor);
                        }
                    }
                }
            }
        }
        triangles.sort_unstable();
        output.push(ConnectedComponent {
            triangle_indices: triangles,
            vertex_indices: vertices,
        });
    }
    Ok(output)
}

fn component_bounds(
    primitive: &IrPrimitive,
    component: &ConnectedComponent,
    source_world: Mat4,
) -> ([f32; 3], [f32; 3]) {
    let mut bounds = Bounds::empty();
    for index in &component.vertex_indices {
        bounds.include(source_world.transform_point(primitive.positions[*index as usize]));
    }
    (bounds.min, bounds.max)
}

fn source_node_name(node: &IrNode) -> String {
    node.name
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("Node {}", node.id))
}

pub fn inspect_placeable_elements_v1(
    ir: &AuroraAssetIr,
) -> Result<PlaceableElementInspectionV1, PlaceableAuthoringErrorV1> {
    let source_worlds = source_world_matrices(ir)?;
    let mut nodes = Vec::new();
    let mut primitive_count = 0u32;
    let mut component_count = 0u32;
    for node in &ir.nodes {
        let Some(mesh_id) = node.mesh_id else {
            continue;
        };
        let mesh = ir
            .meshes
            .get(mesh_id as usize)
            .filter(|mesh| mesh.id == mesh_id)
            .ok_or_else(|| {
                error(
                    "PLACEABLE-AUTHORING-SOURCE-INVALID",
                    format!("nodes[{}].meshId", node.id),
                    "node references a missing mesh",
                )
            })?;
        let mut primitives = Vec::new();
        for primitive_id in &mesh.primitive_ids {
            let primitive = ir
                .primitives
                .get(*primitive_id as usize)
                .filter(|primitive| primitive.id == *primitive_id)
                .ok_or_else(|| {
                    error(
                        "PLACEABLE-AUTHORING-SOURCE-INVALID",
                        format!("meshes[{}].primitiveIds", mesh.id),
                        "mesh references a missing primitive",
                    )
                })?;
            let components =
                connected_components(primitive, &format!("primitives[{}].indices", primitive.id))?;
            let mut inspections = Vec::new();
            for (component_index, component) in components.iter().enumerate() {
                let (bounds_min, bounds_max) = component_bounds(
                    primitive,
                    component,
                    *source_worlds.get(&node.id).ok_or_else(|| {
                        error(
                            "PLACEABLE-AUTHORING-SOURCE-INVALID",
                            format!("nodes[{}]", node.id),
                            "source world transform is missing",
                        )
                    })?,
                );
                inspections.push(PlaceableComponentInspectionV1 {
                    element_id: format!(
                        "node:{}:primitive:{}:component:{component_index}",
                        node.id, primitive.id
                    ),
                    component_index: component_index as u32,
                    triangle_count: component.triangle_indices.len() as u32,
                    vertex_count: component.vertex_indices.len() as u32,
                    bounds_min,
                    bounds_max,
                });
            }
            primitive_count = primitive_count.saturating_add(1);
            component_count = component_count.saturating_add(inspections.len() as u32);
            primitives.push(PlaceablePrimitiveInspectionV1 {
                primitive_id: primitive.id,
                material_id: primitive.material_id,
                triangle_count: (primitive.indices.len() / 3) as u32,
                components: inspections,
            });
        }
        nodes.push(PlaceableNodeInspectionV1 {
            element_id: format!("node:{}", node.id),
            node_id: node.id,
            name: source_node_name(node),
            mesh_id,
            primitives,
        });
    }
    Ok(PlaceableElementInspectionV1 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: ir.source.sha256.clone(),
        render_node_count: nodes.len() as u32,
        primitive_count,
        connected_component_count: component_count,
        nodes,
    })
}

pub fn default_placeable_authoring_v1(
    ir: &AuroraAssetIr,
) -> Result<PlaceableAuthoringDocumentV1, PlaceableAuthoringErrorV1> {
    let inspection = inspect_placeable_elements_v1(ir)?;
    Ok(PlaceableAuthoringDocumentV1 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: inspection.source_sha256,
        elements: inspection
            .nodes
            .into_iter()
            .map(|node| PlaceableAuthoringElementV1 {
                id: node.element_id,
                name: node.name,
                kind: PlaceableElementKindV1::SourceNode,
                source: Some(PlaceableElementSourceV1 {
                    node_id: node.node_id,
                    primitive_id: None,
                    component_index: None,
                }),
                parent_id: None,
                transform: PlaceableElementTransformV1::default(),
                flags: PlaceableElementFlagsV1::default(),
                deleted: false,
            })
            .collect(),
    })
}

pub fn split_placeable_node_components_v1(
    ir: &AuroraAssetIr,
    document: &mut PlaceableAuthoringDocumentV1,
    node_id: u32,
) -> Result<(), PlaceableAuthoringErrorV1> {
    let inspection = inspect_placeable_elements_v1(ir)?;
    let node = inspection
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .ok_or_else(|| {
            error(
                "PLACEABLE-AUTHORING-NODE-MISSING",
                "nodeId",
                format!("source render node {node_id} does not exist"),
            )
        })?;
    let source_index = document
        .elements
        .iter()
        .position(|element| {
            element.kind == PlaceableElementKindV1::SourceNode
                && element
                    .source
                    .as_ref()
                    .is_some_and(|source| source.node_id == node_id)
        })
        .ok_or_else(|| {
            error(
                "PLACEABLE-AUTHORING-SPLIT-STATE",
                "elements",
                "node is already split or has no source-node element",
            )
        })?;
    let template = document.elements.remove(source_index);
    let mut components = Vec::new();
    for primitive in &node.primitives {
        for component in &primitive.components {
            components.push(PlaceableAuthoringElementV1 {
                id: component.element_id.clone(),
                name: format!(
                    "{} · component {}",
                    node.name,
                    component.component_index + 1
                ),
                kind: PlaceableElementKindV1::SourceComponent,
                source: Some(PlaceableElementSourceV1 {
                    node_id,
                    primitive_id: Some(primitive.primitive_id),
                    component_index: Some(component.component_index),
                }),
                parent_id: template.parent_id.clone(),
                transform: template.transform.clone(),
                flags: template.flags,
                deleted: template.deleted,
            });
        }
    }
    document
        .elements
        .splice(source_index..source_index, components);
    Ok(())
}

/// Projects one immutable authoring document onto a concrete pipeline lane.
///
/// Render, collision and shadow generation therefore resolve the same element
/// hierarchy and transforms; only the lane-specific participation flag changes.
pub fn project_placeable_authoring_v1(
    document: &PlaceableAuthoringDocumentV1,
    projection: PlaceableAuthoringProjectionV1,
) -> PlaceableAuthoringDocumentV1 {
    let mut projected = document.clone();
    for element in &mut projected.elements {
        element.flags.renderable = match projection {
            PlaceableAuthoringProjectionV1::Render => element.flags.renderable,
            PlaceableAuthoringProjectionV1::Collision => element.flags.include_in_collision,
            PlaceableAuthoringProjectionV1::Shadow => element.flags.cast_shadow,
        };
    }
    projected
}

fn authoring_hash(
    document: &PlaceableAuthoringDocumentV1,
) -> Result<String, PlaceableAuthoringErrorV1> {
    let mut canonical = document.clone();
    canonical
        .elements
        .sort_by(|left, right| left.id.cmp(&right.id));
    let bytes = serde_json::to_vec(&canonical).map_err(|source| {
        error(
            "PLACEABLE-AUTHORING-SERIALIZE",
            "document",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_document<'a>(
    ir: &AuroraAssetIr,
    document: &'a PlaceableAuthoringDocumentV1,
) -> Result<BTreeMap<&'a str, &'a PlaceableAuthoringElementV1>, PlaceableAuthoringErrorV1> {
    if document.schema_version != PLACEABLE_AUTHORING_SCHEMA_VERSION_V1 {
        return Err(error(
            "PLACEABLE-AUTHORING-SCHEMA-UNSUPPORTED",
            "schemaVersion",
            "only placeable authoring schema version 1 is supported",
        ));
    }
    if document.source_sha256 != ir.source.sha256 {
        return Err(error(
            "PLACEABLE-AUTHORING-SOURCE-MISMATCH",
            "sourceSha256",
            "authoring document is bound to a different source GLB",
        ));
    }
    let mut elements = BTreeMap::new();
    for (index, element) in document.elements.iter().enumerate() {
        let path = format!("elements[{index}]");
        if element.id.is_empty() || element.id.len() > 160 {
            return Err(error(
                "PLACEABLE-AUTHORING-ID-INVALID",
                format!("{path}.id"),
                "element id must contain 1..=160 bytes",
            ));
        }
        if elements.insert(element.id.as_str(), element).is_some() {
            return Err(error(
                "PLACEABLE-AUTHORING-ID-DUPLICATE",
                format!("{path}.id"),
                "element ids must be unique",
            ));
        }
        matrix_from_authoring_transform(&element.transform, &format!("{path}.transform"))?;
        match element.kind {
            PlaceableElementKindV1::Group if element.source.is_some() => {
                return Err(error(
                    "PLACEABLE-AUTHORING-SOURCE-INVALID",
                    format!("{path}.source"),
                    "groups cannot reference source geometry",
                ));
            }
            PlaceableElementKindV1::Group => {}
            PlaceableElementKindV1::SourceNode => {
                let source = element.source.as_ref().ok_or_else(|| {
                    error(
                        "PLACEABLE-AUTHORING-SOURCE-INVALID",
                        format!("{path}.source"),
                        "source-node element requires a selector",
                    )
                })?;
                if source.primitive_id.is_some() || source.component_index.is_some() {
                    return Err(error(
                        "PLACEABLE-AUTHORING-SOURCE-INVALID",
                        format!("{path}.source"),
                        "source-node selector cannot name a primitive or component",
                    ));
                }
            }
            PlaceableElementKindV1::SourceComponent => {
                let source = element.source.as_ref().ok_or_else(|| {
                    error(
                        "PLACEABLE-AUTHORING-SOURCE-INVALID",
                        format!("{path}.source"),
                        "source-component element requires a selector",
                    )
                })?;
                if source.primitive_id.is_none() || source.component_index.is_none() {
                    return Err(error(
                        "PLACEABLE-AUTHORING-SOURCE-INVALID",
                        format!("{path}.source"),
                        "source-component selector requires primitiveId and componentIndex",
                    ));
                }
            }
            PlaceableElementKindV1::Copy if element.source.is_none() => {
                return Err(error(
                    "PLACEABLE-AUTHORING-SOURCE-INVALID",
                    format!("{path}.source"),
                    "copy element requires a source selector",
                ));
            }
            PlaceableElementKindV1::Copy => {}
        }
    }
    for element in document.elements.iter() {
        if let Some(parent) = element.parent_id.as_deref()
            && !elements.contains_key(parent)
        {
            return Err(error(
                "PLACEABLE-AUTHORING-PARENT-MISSING",
                format!("elements[{}].parentId", element.id),
                format!("parent element {parent:?} does not exist"),
            ));
        }
        let mut visited = BTreeSet::new();
        let mut current = Some(element.id.as_str());
        while let Some(id) = current {
            if !visited.insert(id) {
                return Err(error(
                    "PLACEABLE-AUTHORING-HIERARCHY-CYCLE",
                    format!("elements[{}].parentId", element.id),
                    "element parenting must be acyclic",
                ));
            }
            current = elements.get(id).and_then(|item| item.parent_id.as_deref());
        }
    }
    Ok(elements)
}

fn source_world_matrices(
    ir: &AuroraAssetIr,
) -> Result<BTreeMap<u32, Mat4>, PlaceableAuthoringErrorV1> {
    fn resolve(
        node_id: u32,
        ir: &AuroraAssetIr,
        worlds: &mut BTreeMap<u32, Mat4>,
        visiting: &mut BTreeSet<u32>,
    ) -> Result<Mat4, PlaceableAuthoringErrorV1> {
        if let Some(world) = worlds.get(&node_id) {
            return Ok(*world);
        }
        if !visiting.insert(node_id) {
            return Err(error(
                "PLACEABLE-AUTHORING-SOURCE-CYCLE",
                format!("nodes[{node_id}]"),
                "source node hierarchy contains a cycle",
            ));
        }
        let node = ir
            .nodes
            .get(node_id as usize)
            .filter(|node| node.id == node_id)
            .ok_or_else(|| {
                error(
                    "PLACEABLE-AUTHORING-SOURCE-INVALID",
                    format!("nodes[{node_id}]"),
                    "source node is missing",
                )
            })?;
        if node.parent_ids.len() > 1 {
            return Err(error(
                "PLACEABLE-AUTHORING-SOURCE-INVALID",
                format!("nodes[{node_id}].parentIds"),
                "element authoring requires a single-parent node tree",
            ));
        }
        let local =
            matrix_from_ir_transform(&node.transform, &format!("nodes[{node_id}].transform"))?;
        let world = if let Some(parent) = node.parent_ids.first() {
            resolve(*parent, ir, worlds, visiting)?.mul(local)
        } else {
            local
        };
        visiting.remove(&node_id);
        worlds.insert(node_id, world);
        Ok(world)
    }

    let mut worlds = BTreeMap::new();
    for node in &ir.nodes {
        resolve(node.id, ir, &mut worlds, &mut BTreeSet::new())?;
    }
    Ok(worlds)
}

fn authoring_world_matrices(
    document: &PlaceableAuthoringDocumentV1,
    elements: &BTreeMap<&str, &PlaceableAuthoringElementV1>,
) -> Result<BTreeMap<String, Mat4>, PlaceableAuthoringErrorV1> {
    fn resolve(
        id: &str,
        elements: &BTreeMap<&str, &PlaceableAuthoringElementV1>,
        output: &mut BTreeMap<String, Mat4>,
    ) -> Result<Mat4, PlaceableAuthoringErrorV1> {
        if let Some(value) = output.get(id) {
            return Ok(*value);
        }
        let element = elements[id];
        let local = matrix_from_authoring_transform(
            &element.transform,
            &format!("elements[{id}].transform"),
        )?;
        let world = if let Some(parent) = element.parent_id.as_deref() {
            resolve(parent, elements, output)?.mul(local)
        } else {
            local
        };
        output.insert(id.to_owned(), world);
        Ok(world)
    }
    let mut output = BTreeMap::new();
    for element in &document.elements {
        resolve(&element.id, elements, &mut output)?;
    }
    Ok(output)
}

fn selector_components(
    ir: &AuroraAssetIr,
    source: &PlaceableElementSourceV1,
    kind: PlaceableElementKindV1,
) -> Result<Vec<(u32, u32)>, PlaceableAuthoringErrorV1> {
    let node = ir
        .nodes
        .get(source.node_id as usize)
        .filter(|node| node.id == source.node_id)
        .ok_or_else(|| {
            error(
                "PLACEABLE-AUTHORING-NODE-MISSING",
                "element.source.nodeId",
                "source node does not exist",
            )
        })?;
    let mesh_id = node.mesh_id.ok_or_else(|| {
        error(
            "PLACEABLE-AUTHORING-NODE-NOT-RENDERABLE",
            "element.source.nodeId",
            "source node has no mesh",
        )
    })?;
    let mesh = ir
        .meshes
        .get(mesh_id as usize)
        .filter(|mesh| mesh.id == mesh_id)
        .ok_or_else(|| {
            error(
                "PLACEABLE-AUTHORING-SOURCE-INVALID",
                "element.source.nodeId",
                "source node references a missing mesh",
            )
        })?;
    if matches!(kind, PlaceableElementKindV1::SourceNode)
        || (matches!(kind, PlaceableElementKindV1::Copy)
            && source.primitive_id.is_none()
            && source.component_index.is_none())
    {
        let mut output = Vec::new();
        for primitive_id in &mesh.primitive_ids {
            let primitive = &ir.primitives[*primitive_id as usize];
            for index in 0..connected_components(primitive, "element.source")?.len() {
                output.push((*primitive_id, index as u32));
            }
        }
        return Ok(output);
    }
    let primitive_id = source.primitive_id.ok_or_else(|| {
        error(
            "PLACEABLE-AUTHORING-SOURCE-INVALID",
            "element.source.primitiveId",
            "component selector requires primitiveId",
        )
    })?;
    if !mesh.primitive_ids.contains(&primitive_id) {
        return Err(error(
            "PLACEABLE-AUTHORING-SOURCE-INVALID",
            "element.source.primitiveId",
            "primitive does not belong to the selected source node",
        ));
    }
    let component_index = source.component_index.ok_or_else(|| {
        error(
            "PLACEABLE-AUTHORING-SOURCE-INVALID",
            "element.source.componentIndex",
            "component selector requires componentIndex",
        )
    })?;
    let primitive = &ir.primitives[primitive_id as usize];
    if component_index as usize >= connected_components(primitive, "element.source")?.len() {
        return Err(error(
            "PLACEABLE-AUTHORING-COMPONENT-MISSING",
            "element.source.componentIndex",
            "connected component does not exist",
        ));
    }
    Ok(vec![(primitive_id, component_index)])
}

fn append_component(
    output: &mut IrPrimitive,
    source: &IrPrimitive,
    component: &ConnectedComponent,
    transform: Mat4,
    path: &str,
) -> Result<(), PlaceableAuthoringErrorV1> {
    if !source.joints0.is_empty() || !source.weights0.is_empty() {
        return Err(error(
            "PLACEABLE-AUTHORING-SKINNED-UNSUPPORTED",
            path,
            "static placeable element editing does not accept skinned geometry",
        ));
    }
    let normal_matrix = transform.normal_matrix().ok_or_else(|| {
        error(
            "PLACEABLE-AUTHORING-TRANSFORM-SINGULAR",
            path,
            "element transform has no inverse normal matrix",
        )
    })?;
    let tangent_matrix = transform.linear();
    let mirrored = transform.determinant() < 0.0;
    let mut remap = BTreeMap::new();
    for source_index in &component.vertex_indices {
        let source_usize = *source_index as usize;
        let output_index = u32::try_from(output.positions.len()).map_err(|_| {
            error(
                "PLACEABLE-AUTHORING-LIMIT-EXCEEDED",
                path,
                "authored vertex index exceeds u32",
            )
        })?;
        remap.insert(*source_index, output_index);
        output
            .positions
            .push(transform.transform_point(source.positions[source_usize]));
        let transformed_normal = if source.normals.is_empty() {
            None
        } else {
            let normal = normalize3(mul3(normal_matrix, source.normals[source_usize]), path)?;
            output.normals.push(normal);
            Some(normal)
        };
        if !source.tangents.is_empty() {
            let tangent = source.tangents[source_usize];
            let mut direction = mul3(tangent_matrix, tangent[..3].try_into().unwrap());
            if let Some(normal) = transformed_normal {
                let projection = dot3(direction, normal);
                direction = [
                    direction[0] - projection * normal[0],
                    direction[1] - projection * normal[1],
                    direction[2] - projection * normal[2],
                ];
            }
            let direction = normalize3(direction, path)?;
            output.tangents.push([
                direction[0],
                direction[1],
                direction[2],
                if mirrored { -tangent[3] } else { tangent[3] },
            ]);
        }
        if !source.uv0.is_empty() {
            output.uv0.push(source.uv0[source_usize]);
        }
    }
    for triangle_index in &component.triangle_indices {
        let triangle = &source.indices[triangle_index * 3..triangle_index * 3 + 3];
        let order = if mirrored { [0, 2, 1] } else { [0, 1, 2] };
        for corner in order {
            output.indices.push(remap[&triangle[corner]]);
        }
    }
    Ok(())
}

fn empty_authored_primitive(source: &IrPrimitive) -> IrPrimitive {
    IrPrimitive {
        id: 0,
        source_mesh_id: 0,
        source_primitive_index: 0,
        topology: source.topology.clone(),
        material_id: source.material_id,
        positions: Vec::new(),
        normals: Vec::new(),
        tangents: Vec::new(),
        uv0: Vec::new(),
        joints0: Vec::new(),
        weights0: Vec::new(),
        indices: Vec::new(),
        bounds_min: [0.0; 3],
        bounds_max: [0.0; 3],
        source_was_indexed: true,
    }
}

fn update_primitive_bounds(primitive: &mut IrPrimitive) {
    let mut bounds = Bounds::empty();
    primitive
        .positions
        .iter()
        .for_each(|point| bounds.include(*point));
    if bounds.populated {
        primitive.bounds_min = bounds.min;
        primitive.bounds_max = bounds.max;
    }
}

fn update_ingest_report(ingest: &mut GlbIngestResult) {
    ingest.report.inventory.mesh_count = ingest.ir.meshes.len();
    ingest.report.inventory.primitive_count = ingest.ir.primitives.len();
    ingest.report.inventory.material_count = ingest.ir.materials.len();
    ingest.report.statistics.vertex_count = ingest
        .ir
        .primitives
        .iter()
        .map(|primitive| primitive.positions.len())
        .sum();
    ingest.report.statistics.index_count = ingest
        .ir
        .primitives
        .iter()
        .map(|primitive| primitive.indices.len())
        .sum();
    ingest.report.statistics.triangle_count = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.topology == "TRIANGLES")
        .map(|primitive| primitive.indices.len() / 3)
        .sum();
    let mut bounds = Bounds::empty();
    ingest
        .ir
        .primitives
        .iter()
        .flat_map(|primitive| primitive.positions.iter())
        .for_each(|point| bounds.include(*point));
    let (min, max) = bounds.optional();
    ingest.report.statistics.bounds_min = min;
    ingest.report.statistics.bounds_max = max;
}

pub fn apply_placeable_authoring_to_ingest_v1(
    ingest: &mut GlbIngestResult,
    document: &PlaceableAuthoringDocumentV1,
) -> Result<PlaceableAuthoringApplyReportV1, PlaceableAuthoringErrorV1> {
    let report = apply_placeable_authoring_v1(&mut ingest.ir, document)?;
    update_ingest_report(ingest);
    Ok(report)
}

pub fn apply_placeable_authoring_v1(
    ir: &mut AuroraAssetIr,
    document: &PlaceableAuthoringDocumentV1,
) -> Result<PlaceableAuthoringApplyReportV1, PlaceableAuthoringErrorV1> {
    let original = ir.clone();
    let elements = validate_document(&original, document)?;
    let source_worlds = source_world_matrices(&original)?;
    let authoring_worlds = authoring_world_matrices(document, &elements)?;
    let authoring_sha256 = authoring_hash(document)?;
    let source_triangle_count = original
        .primitives
        .iter()
        .map(|primitive| primitive.indices.len() / 3)
        .sum::<usize>();

    let mut components = BTreeMap::<u32, Vec<ConnectedComponent>>::new();
    for primitive in &original.primitives {
        components.insert(
            primitive.id,
            connected_components(primitive, &format!("primitives[{}]", primitive.id))?,
        );
    }

    let mut coverage = BTreeMap::<(u32, u32, u32), u32>::new();
    for node in original.nodes.iter().filter(|node| node.mesh_id.is_some()) {
        let mesh = &original.meshes[node.mesh_id.unwrap() as usize];
        for primitive_id in &mesh.primitive_ids {
            for component_index in 0..components[primitive_id].len() {
                coverage.insert((node.id, *primitive_id, component_index as u32), 0);
            }
        }
    }

    for element in &document.elements {
        if !matches!(
            element.kind,
            PlaceableElementKindV1::SourceNode | PlaceableElementKindV1::SourceComponent
        ) {
            continue;
        }
        let source = element.source.as_ref().expect("validated source");
        for (primitive_id, component_index) in selector_components(&original, source, element.kind)?
        {
            let entry = coverage
                .get_mut(&(source.node_id, primitive_id, component_index))
                .ok_or_else(|| {
                    error(
                        "PLACEABLE-AUTHORING-SOURCE-INVALID",
                        format!("elements[{}].source", element.id),
                        "selector is outside the source render inventory",
                    )
                })?;
            *entry += 1;
            if *entry > 1 {
                return Err(error(
                    "PLACEABLE-AUTHORING-SOURCE-OVERLAP",
                    format!("elements[{}].source", element.id),
                    "source geometry is covered by more than one source element",
                ));
            }
        }
    }
    if let Some(((node, primitive, component), _)) = coverage.iter().find(|(_, count)| **count != 1)
    {
        return Err(error(
            "PLACEABLE-AUTHORING-SOURCE-COVERAGE",
            "elements",
            format!(
                "source node {node}, primitive {primitive}, component {component} must be covered exactly once"
            ),
        ));
    }

    let mut renderable_element_count = 0u32;
    let mut collision_element_count = 0u32;
    let mut shadow_element_count = 0u32;
    let mut collision_bounds = Bounds::empty();
    let mut shadow_bounds = Bounds::empty();
    let mut output_meshes = Vec::new();
    let mut output_primitives = Vec::new();
    let mut node_meshes = BTreeMap::<u32, Option<u32>>::new();
    let needs_shadowless_materials = document
        .elements
        .iter()
        .any(|element| !element.deleted && element.flags.renderable && !element.flags.cast_shadow);
    let mut output_materials = original.materials.clone();
    let mut shadowless_material_ids = BTreeMap::<u32, u32>::new();
    if needs_shadowless_materials {
        for material in &original.materials {
            let mut shadowless = material.clone();
            let id = u32::try_from(output_materials.len()).map_err(|_| {
                error(
                    "PLACEABLE-AUTHORING-LIMIT-EXCEEDED",
                    "materials",
                    "shadow material count exceeds u32",
                )
            })?;
            shadowless.id = id;
            shadowless.name = Some(format!(
                "{}{SHADOWLESS_MATERIAL_SUFFIX_V1}",
                material.name.as_deref().unwrap_or("m2a_material")
            ));
            output_materials.push(shadowless);
            shadowless_material_ids.insert(material.id, id);
        }
    }

    for node in original.nodes.iter().filter(|node| node.mesh_id.is_some()) {
        let source_mesh = &original.meshes[node.mesh_id.unwrap() as usize];
        let mut authored_primitives = BTreeMap::<(u32, bool), IrPrimitive>::new();

        for element in &document.elements {
            let Some(source) = element.source.as_ref() else {
                continue;
            };
            if source.node_id != node.id
                || !matches!(
                    element.kind,
                    PlaceableElementKindV1::SourceNode
                        | PlaceableElementKindV1::SourceComponent
                        | PlaceableElementKindV1::Copy
                )
            {
                continue;
            }
            let selected = selector_components(&original, source, element.kind)?;
            let source_world = source_worlds[&node.id];
            let local_bake = source_world
                .inverse_affine()
                .ok_or_else(|| {
                    error(
                        "PLACEABLE-AUTHORING-SOURCE-TRANSFORM-SINGULAR",
                        format!("nodes[{}].transform", node.id),
                        "source node world transform is singular",
                    )
                })?
                .mul(authoring_worlds[&element.id])
                .mul(source_world);

            if !element.deleted && element.flags.renderable {
                renderable_element_count += 1;
            }
            if !element.deleted && element.flags.include_in_collision {
                collision_element_count += 1;
            }
            if !element.deleted && element.flags.cast_shadow {
                shadow_element_count += 1;
            }

            for (primitive_id, component_index) in selected {
                let source_primitive = &original.primitives[primitive_id as usize];
                let component = &components[&primitive_id][component_index as usize];
                if !element.deleted && element.flags.renderable {
                    let target_material_id =
                        if element.flags.cast_shadow {
                            source_primitive.material_id
                        } else {
                            let source_material_id = source_primitive.material_id.ok_or_else(|| {
                            error(
                                "PLACEABLE-AUTHORING-SHADOW-MATERIAL-MISSING",
                                format!("primitives[{primitive_id}].materialId"),
                                "per-element shadow control requires an explicit source material",
                            )
                        })?;
                            Some(*shadowless_material_ids.get(&source_material_id).ok_or_else(
                            || {
                                error(
                                    "PLACEABLE-AUTHORING-SHADOW-MATERIAL-MISSING",
                                    format!("materials[{source_material_id}]"),
                                    "source material cannot be duplicated for shadow control",
                                )
                            },
                        )?)
                        };
                    let target = authored_primitives
                        .entry((primitive_id, element.flags.cast_shadow))
                        .or_insert_with(|| {
                            let mut primitive = empty_authored_primitive(source_primitive);
                            primitive.material_id = target_material_id;
                            primitive
                        });
                    append_component(
                        target,
                        source_primitive,
                        component,
                        local_bake,
                        &format!("elements[{}]", element.id),
                    )?;
                }
                if !element.deleted
                    && (element.flags.include_in_collision || element.flags.cast_shadow)
                {
                    for index in &component.vertex_indices {
                        let point = source_world
                            .mul(local_bake)
                            .transform_point(source_primitive.positions[*index as usize]);
                        if element.flags.include_in_collision {
                            collision_bounds.include(point);
                        }
                        if element.flags.cast_shadow {
                            shadow_bounds.include(point);
                        }
                    }
                }
            }
        }

        let mut primitive_ids = Vec::new();
        for (_, mut primitive) in authored_primitives {
            let new_id = u32::try_from(output_primitives.len()).map_err(|_| {
                error(
                    "PLACEABLE-AUTHORING-LIMIT-EXCEEDED",
                    "primitives",
                    "authored primitive count exceeds u32",
                )
            })?;
            primitive.id = new_id;
            primitive.source_mesh_id = output_meshes.len() as u32;
            primitive.source_primitive_index = primitive_ids.len() as u32;
            update_primitive_bounds(&mut primitive);
            primitive_ids.push(new_id);
            output_primitives.push(primitive);
        }
        if primitive_ids.is_empty() {
            node_meshes.insert(node.id, None);
        } else {
            let mesh_id = output_meshes.len() as u32;
            node_meshes.insert(node.id, Some(mesh_id));
            output_meshes.push(IrMesh {
                id: mesh_id,
                name: source_mesh.name.clone(),
                primitive_ids,
            });
        }
    }

    for node in &mut ir.nodes {
        if node.mesh_id.is_some() {
            node.mesh_id = node_meshes.get(&node.id).copied().flatten();
        }
    }
    ir.meshes = output_meshes;
    ir.primitives = output_primitives;
    ir.materials = output_materials;

    let mut output_bounds = Bounds::empty();
    let output_worlds = source_world_matrices(ir)?;
    for node in ir.nodes.iter().filter(|node| node.mesh_id.is_some()) {
        let mesh = &ir.meshes[node.mesh_id.expect("filtered mesh") as usize];
        let world = output_worlds[&node.id];
        for primitive_id in &mesh.primitive_ids {
            for point in &ir.primitives[*primitive_id as usize].positions {
                output_bounds.include(world.transform_point(*point));
            }
        }
    }
    let (bounds_min, bounds_max) = output_bounds.required("elements")?;
    let (collision_bounds_min, collision_bounds_max) = collision_bounds.optional();
    let (shadow_bounds_min, shadow_bounds_max) = shadow_bounds.optional();

    Ok(PlaceableAuthoringApplyReportV1 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: document.source_sha256.clone(),
        authoring_sha256,
        source_triangle_count: source_triangle_count as u32,
        output_triangle_count: ir
            .primitives
            .iter()
            .map(|primitive| primitive.indices.len() / 3)
            .sum::<usize>() as u32,
        renderable_element_count,
        collision_element_count,
        shadow_element_count,
        bounds_min,
        bounds_max,
        collision_bounds_min,
        collision_bounds_max,
        shadow_bounds_min,
        shadow_bounds_max,
    })
}
