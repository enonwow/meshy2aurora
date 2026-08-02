//! Target-neutral connected-component inspection for render-model geometry.
//!
//! Material authoring, Placeable element editing and future Item-part editing
//! must resolve the same source component identity. The source GLB stays
//! immutable; component ordinals are bound to its exact SHA-256.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::{
    AURORA_MODEL_TRIANGLE_BUDGET_V1,
    glb::{AuroraAssetIr, IrPrimitive, IrTransform},
};

pub const MODEL_COMPONENT_INSPECTION_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceComponentKeyV1 {
    pub scene_id: u32,
    pub node_id: u32,
    pub primitive_id: u32,
    pub component_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelComponentInspectionV1 {
    pub key: SourceComponentKeyV1,
    pub source_material_id: Option<u32>,
    pub source_material_name: Option<String>,
    pub triangle_count: u32,
    pub vertex_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelComponentInventoryV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub scene_id: u32,
    pub render_node_count: u32,
    pub primitive_instance_count: u32,
    pub triangle_count: u32,
    pub components: Vec<ModelComponentInspectionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelComponentErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelComponentErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelComponentErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> ModelComponentErrorV1 {
    ModelComponentErrorV1 {
        schema_version: MODEL_COMPONENT_INSPECTION_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConnectedComponentV1 {
    pub(crate) triangle_indices: Vec<usize>,
    pub(crate) vertex_indices: BTreeSet<u32>,
}

pub(crate) fn connected_components_v1(
    primitive: &IrPrimitive,
    path: &str,
) -> Result<Vec<ConnectedComponentV1>, ModelComponentErrorV1> {
    if primitive.topology != "TRIANGLES" || !primitive.indices.len().is_multiple_of(3) {
        return Err(error(
            "MODEL-COMPONENTS-TOPOLOGY-UNSUPPORTED",
            path,
            "component inspection requires indexed TRIANGLES",
        ));
    }
    if primitive.indices.iter().any(|index| {
        usize::try_from(*index)
            .ok()
            .is_none_or(|index| index >= primitive.positions.len())
    }) {
        return Err(error(
            "MODEL-COMPONENTS-INDEX-OOB",
            path,
            "primitive index exceeds the position array",
        ));
    }

    let triangle_count = primitive.indices.len() / 3;
    if triangle_count > AURORA_MODEL_TRIANGLE_BUDGET_V1 {
        return Err(error(
            "MODEL-COMPONENTS-TRIANGLE-BUDGET-EXCEEDED",
            path,
            format!(
                "component inspection accepts at most {AURORA_MODEL_TRIANGLE_BUDGET_V1} triangles, got {triangle_count}"
            ),
        ));
    }
    fn find(parent: &mut [usize], mut value: usize) -> usize {
        let mut root = value;
        while parent[root] != root {
            root = parent[root];
        }
        while parent[value] != value {
            let next = parent[value];
            parent[value] = root;
            value = next;
        }
        root
    }

    fn union(parent: &mut [usize], left: usize, right: usize) {
        let left = find(parent, left);
        let right = find(parent, right);
        if left == right {
            return;
        }
        let (root, child) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        parent[child] = root;
    }

    // A vertex stores only the first triangle that referenced it. Union-find
    // then connects later triangles in near-linear time without materializing
    // a potentially quadratic high-valence triangle adjacency list.
    let mut parent = (0..triangle_count).collect::<Vec<_>>();
    let mut first_triangle_by_vertex = BTreeMap::<u32, usize>::new();
    for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
        for vertex_index in triangle {
            if let Some(first_triangle) = first_triangle_by_vertex.get(vertex_index) {
                union(&mut parent, triangle_index, *first_triangle);
            } else {
                first_triangle_by_vertex.insert(*vertex_index, triangle_index);
            }
        }
    }

    let mut grouped = BTreeMap::<usize, ConnectedComponentV1>::new();
    for triangle_index in 0..triangle_count {
        let root = find(&mut parent, triangle_index);
        let component = grouped.entry(root).or_insert_with(|| ConnectedComponentV1 {
            triangle_indices: Vec::new(),
            vertex_indices: BTreeSet::new(),
        });
        component.triangle_indices.push(triangle_index);
        component.vertex_indices.extend(
            primitive.indices[triangle_index * 3..triangle_index * 3 + 3]
                .iter()
                .copied(),
        );
    }
    Ok(grouped.into_values().collect())
}

#[derive(Clone, Copy)]
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
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
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
}

fn matrix_from_ir_transform(
    transform: &IrTransform,
    path: &str,
) -> Result<Mat4, ModelComponentErrorV1> {
    if transform.kind == "MATRIX" {
        if let Some(matrix) = transform.matrix
            && matrix.iter().all(|value| value.is_finite())
        {
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
        "MODEL-COMPONENTS-TRANSFORM-INVALID",
        path,
        "source node transform is malformed or non-finite",
    ))
}

fn scene_world_matrices(
    ir: &AuroraAssetIr,
    scene_id: u32,
) -> Result<BTreeMap<u32, Mat4>, ModelComponentErrorV1> {
    let scene = ir
        .scenes
        .iter()
        .find(|scene| scene.id == scene_id)
        .ok_or_else(|| {
            error(
                "MODEL-COMPONENTS-SCENE-MISSING",
                "defaultSceneId",
                "default scene does not exist",
            )
        })?;
    let mut worlds = BTreeMap::new();
    let mut visiting = BTreeSet::new();

    fn visit(
        ir: &AuroraAssetIr,
        node_id: u32,
        parent_world: Mat4,
        worlds: &mut BTreeMap<u32, Mat4>,
        visiting: &mut BTreeSet<u32>,
    ) -> Result<(), ModelComponentErrorV1> {
        if worlds.contains_key(&node_id) {
            return Err(error(
                "MODEL-COMPONENTS-NODE-MULTI-PARENT",
                format!("nodes[{node_id}]"),
                "a node is reached more than once in the selected scene",
            ));
        }
        if !visiting.insert(node_id) {
            return Err(error(
                "MODEL-COMPONENTS-NODE-CYCLE",
                format!("nodes[{node_id}]"),
                "selected scene contains a node cycle",
            ));
        }
        let node = ir
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| {
                error(
                    "MODEL-COMPONENTS-NODE-MISSING",
                    format!("nodes[{node_id}]"),
                    "scene references a missing node",
                )
            })?;
        let local =
            matrix_from_ir_transform(&node.transform, &format!("nodes[{node_id}].transform"))?;
        let world = parent_world.mul(local);
        worlds.insert(node_id, world);
        for child_id in &node.child_ids {
            visit(ir, *child_id, world, worlds, visiting)?;
        }
        visiting.remove(&node_id);
        Ok(())
    }

    for root_node_id in &scene.root_node_ids {
        visit(
            ir,
            *root_node_id,
            Mat4::identity(),
            &mut worlds,
            &mut visiting,
        )?;
    }
    Ok(worlds)
}

pub fn inspect_model_components_v1(
    ir: &AuroraAssetIr,
) -> Result<ModelComponentInventoryV1, ModelComponentErrorV1> {
    let scene_id = ir.default_scene_id.ok_or_else(|| {
        error(
            "MODEL-COMPONENTS-DEFAULT-SCENE-MISSING",
            "defaultSceneId",
            "component inspection requires an explicit default scene",
        )
    })?;
    let worlds = scene_world_matrices(ir, scene_id)?;
    let mut components = Vec::new();
    let mut primitive_instance_count = 0u32;
    let mut triangle_count = 0u32;

    for (node_id, world) in &worlds {
        let node = ir
            .nodes
            .iter()
            .find(|node| node.id == *node_id)
            .ok_or_else(|| {
                error(
                    "MODEL-COMPONENTS-NODE-MISSING",
                    format!("nodes[{node_id}]"),
                    "resolved scene node is missing",
                )
            })?;
        let Some(mesh_id) = node.mesh_id else {
            continue;
        };
        let mesh = ir
            .meshes
            .iter()
            .find(|mesh| mesh.id == mesh_id)
            .ok_or_else(|| {
                error(
                    "MODEL-COMPONENTS-MESH-MISSING",
                    format!("nodes[{node_id}].meshId"),
                    "node references a missing mesh",
                )
            })?;
        for primitive_id in &mesh.primitive_ids {
            let primitive = ir
                .primitives
                .iter()
                .find(|primitive| primitive.id == *primitive_id)
                .ok_or_else(|| {
                    error(
                        "MODEL-COMPONENTS-PRIMITIVE-MISSING",
                        format!("meshes[{mesh_id}].primitiveIds"),
                        "mesh references a missing primitive",
                    )
                })?;
            let connected =
                connected_components_v1(primitive, &format!("primitives[{primitive_id}].indices"))?;
            primitive_instance_count =
                primitive_instance_count.checked_add(1).ok_or_else(|| {
                    error(
                        "MODEL-COMPONENTS-COUNT-OVERFLOW",
                        "primitiveInstanceCount",
                        "primitive instance count exceeds u32",
                    )
                })?;
            triangle_count = triangle_count
                .checked_add(u32::try_from(primitive.indices.len() / 3).map_err(|_| {
                    error(
                        "MODEL-COMPONENTS-COUNT-OVERFLOW",
                        format!("primitives[{primitive_id}].indices"),
                        "triangle count exceeds u32",
                    )
                })?)
                .ok_or_else(|| {
                    error(
                        "MODEL-COMPONENTS-COUNT-OVERFLOW",
                        "triangleCount",
                        "triangle count exceeds u32",
                    )
                })?;
            if triangle_count as usize > AURORA_MODEL_TRIANGLE_BUDGET_V1 {
                return Err(error(
                    "MODEL-COMPONENTS-TRIANGLE-BUDGET-EXCEEDED",
                    "triangleCount",
                    format!(
                        "component inspection accepts at most {AURORA_MODEL_TRIANGLE_BUDGET_V1} triangles, got {triangle_count}"
                    ),
                ));
            }
            let source_material_name = primitive.material_id.and_then(|material_id| {
                ir.materials
                    .iter()
                    .find(|material| material.id == material_id)
                    .and_then(|material| material.name.clone())
            });
            for (component_index, connected) in connected.iter().enumerate() {
                let mut bounds_min = [f32::INFINITY; 3];
                let mut bounds_max = [f32::NEG_INFINITY; 3];
                for vertex_index in &connected.vertex_indices {
                    let point = world.transform_point(primitive.positions[*vertex_index as usize]);
                    for axis in 0..3 {
                        bounds_min[axis] = bounds_min[axis].min(point[axis]);
                        bounds_max[axis] = bounds_max[axis].max(point[axis]);
                    }
                }
                components.push(ModelComponentInspectionV1 {
                    key: SourceComponentKeyV1 {
                        scene_id,
                        node_id: *node_id,
                        primitive_id: *primitive_id,
                        component_index: u32::try_from(component_index).map_err(|_| {
                            error(
                                "MODEL-COMPONENTS-COUNT-OVERFLOW",
                                format!("primitives[{primitive_id}].components"),
                                "component index exceeds u32",
                            )
                        })?,
                    },
                    source_material_id: primitive.material_id,
                    source_material_name: source_material_name.clone(),
                    triangle_count: u32::try_from(connected.triangle_indices.len()).map_err(
                        |_| {
                            error(
                                "MODEL-COMPONENTS-COUNT-OVERFLOW",
                                format!("primitives[{primitive_id}].components"),
                                "component triangle count exceeds u32",
                            )
                        },
                    )?,
                    vertex_count: u32::try_from(connected.vertex_indices.len()).map_err(|_| {
                        error(
                            "MODEL-COMPONENTS-COUNT-OVERFLOW",
                            format!("primitives[{primitive_id}].components"),
                            "component vertex count exceeds u32",
                        )
                    })?,
                    bounds_min,
                    bounds_max,
                });
            }
        }
    }
    components.sort_by_key(|component| component.key);
    Ok(ModelComponentInventoryV1 {
        schema_version: MODEL_COMPONENT_INSPECTION_SCHEMA_VERSION_V1,
        source_sha256: ir.source.sha256.clone(),
        scene_id,
        render_node_count: u32::try_from(
            worlds
                .keys()
                .filter(|node_id| {
                    ir.nodes
                        .iter()
                        .find(|node| node.id == **node_id)
                        .is_some_and(|node| node.mesh_id.is_some())
                })
                .count(),
        )
        .map_err(|_| {
            error(
                "MODEL-COMPONENTS-COUNT-OVERFLOW",
                "renderNodeCount",
                "render node count exceeds u32",
            )
        })?,
        primitive_instance_count,
        triangle_count,
        components,
    })
}
