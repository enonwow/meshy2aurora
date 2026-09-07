//! Target-neutral per-material UV projection after Material Separation.
//!
//! The source GLB remains immutable. Projection is applied only to authored
//! render primitives after material assignment; collision geometry is not
//! touched.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    glb::{AuroraAssetIr, GlbIngestResult, IrPrimitive},
    model_components::connected_components_v1,
};

pub const MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelMaterialUvProjectionModeV1 {
    Source,
    ComponentLongAxis,
    /// Projects every disconnected component of one authored material in the
    /// same coordinate space. This keeps grain direction, scale and phase
    /// continuous across fragmented Meshy geometry.
    MaterialLongAxis,
    /// Uses one material-wide coordinate space while choosing the planar
    /// projection from each face normal. It is the low-cost automatic option
    /// for fragmented hard-surface assets that mix side, top and end faces.
    MaterialBox,
    /// Box projection anchored in authored model space. `u_repeats` is the
    /// number of texture repeats per authored metre on both projected axes,
    /// so disconnected surfaces keep one physical texel scale.
    MaterialBoxWorld,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialUvProjectionRuleV1 {
    pub authored_material_id: String,
    pub mode: ModelMaterialUvProjectionModeV1,
    pub u_repeats: f32,
    pub v_min: f32,
    pub v_max: f32,
    pub deterministic_u_phase: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialUvProjectionDocumentV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub separation_sha256: String,
    pub rules: Vec<ModelMaterialUvProjectionRuleV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialUvProjectionReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub separation_sha256: String,
    pub projection_sha256: String,
    pub source_triangle_count: u32,
    pub output_triangle_count: u32,
    pub source_vertex_count: u32,
    pub output_vertex_count: u32,
    pub duplicated_vertex_count: u32,
    pub projected_triangle_count: u32,
    pub projected_material_ids: Vec<String>,
    pub source_uv0_preserved_for_unprojected_materials: bool,
    pub geometry_cleanup: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialUvProjectionErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelMaterialUvProjectionErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelMaterialUvProjectionErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ModelMaterialUvProjectionErrorV1 {
    ModelMaterialUvProjectionErrorV1 {
        schema_version: MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

pub fn model_material_uv_projection_hash_v1(
    document: &ModelMaterialUvProjectionDocumentV1,
) -> Result<String, ModelMaterialUvProjectionErrorV1> {
    let mut canonical = document.clone();
    canonical
        .rules
        .sort_by(|left, right| left.authored_material_id.cmp(&right.authored_material_id));
    let payload = serde_json::to_vec(&canonical)
        .map_err(|source| error("MATERIAL-UV-PROJECTION-SERIALIZE", "$", source.to_string()))?;
    Ok(sha256(&payload))
}

fn validate_document(
    ir: &AuroraAssetIr,
    expected_separation_sha256: &str,
    document: &ModelMaterialUvProjectionDocumentV1,
) -> Result<(), ModelMaterialUvProjectionErrorV1> {
    if document.schema_version != MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1 {
        return Err(error(
            "MATERIAL-UV-PROJECTION-SCHEMA-UNSUPPORTED",
            "schemaVersion",
            format!("expected schema version {MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1}"),
        ));
    }
    if document.source_sha256 != ir.source.sha256 {
        return Err(error(
            "MATERIAL-UV-PROJECTION-SOURCE-MISMATCH",
            "sourceSha256",
            "UV projection is bound to another source model",
        ));
    }
    if document.separation_sha256 != expected_separation_sha256 {
        return Err(error(
            "MATERIAL-UV-PROJECTION-SEPARATION-MISMATCH",
            "separationSha256",
            "UV projection is bound to another Material Separation recipe",
        ));
    }
    let mut material_ids = BTreeSet::new();
    for (index, rule) in document.rules.iter().enumerate() {
        if rule.authored_material_id.trim().is_empty() {
            return Err(error(
                "MATERIAL-UV-PROJECTION-ID-INVALID",
                format!("rules[{index}].authoredMaterialId"),
                "authored material ID cannot be empty",
            ));
        }
        if !material_ids.insert(rule.authored_material_id.as_str()) {
            return Err(error(
                "MATERIAL-UV-PROJECTION-ID-DUPLICATE",
                format!("rules[{index}].authoredMaterialId"),
                "authored material ID must be unique",
            ));
        }
        if !rule.u_repeats.is_finite() || !(0.01..=128.0).contains(&rule.u_repeats) {
            return Err(error(
                "MATERIAL-UV-PROJECTION-SCALE-INVALID",
                format!("rules[{index}].uRepeats"),
                "U repeats must be finite and between 0.01 and 128",
            ));
        }
        if !rule.v_min.is_finite()
            || !rule.v_max.is_finite()
            || rule.v_min < 0.0
            || rule.v_max > 1.0
            || rule.v_min >= rule.v_max
        {
            return Err(error(
                "MATERIAL-UV-PROJECTION-RANGE-INVALID",
                format!("rules[{index}]"),
                "V range must be finite, ordered, and inside 0..1",
            ));
        }
        if rule.mode == ModelMaterialUvProjectionModeV1::MaterialBoxWorld
            && (rule.v_min != 0.0 || rule.v_max != 1.0 || rule.deterministic_u_phase)
        {
            return Err(error(
                "MATERIAL-UV-PROJECTION-WORLD-RULE-INVALID",
                format!("rules[{index}]"),
                "world-scale box projection requires the full 0..1 texture and no component phase",
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ComponentProjection {
    min: [f32; 3],
    max: [f32; 3],
    longitudinal_axis: usize,
    transverse_axis: usize,
    phase: f32,
}

fn projection_axes(min: [f32; 3], max: [f32; 3]) -> (usize, usize) {
    let mut axes = [0usize, 1, 2];
    axes.sort_by(|left, right| {
        let left_span = max[*left] - min[*left];
        let right_span = max[*right] - min[*right];
        right_span
            .partial_cmp(&left_span)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.cmp(right))
    });
    (axes[0], axes[1])
}

fn component_projection(
    source: &IrPrimitive,
    vertices: &BTreeSet<u32>,
    component_index: usize,
    deterministic_phase: bool,
) -> ComponentProjection {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        let point = source.positions[*vertex as usize];
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    let (longitudinal_axis, transverse_axis) = projection_axes(min, max);
    let phase = if deterministic_phase {
        let hash = (component_index as u32).wrapping_mul(0x9e37_79b9);
        ((hash >> 8) & 0xffff) as f32 / 65_535.0
    } else {
        0.0
    };
    ComponentProjection {
        min,
        max,
        longitudinal_axis,
        transverse_axis,
        phase,
    }
}

fn projected_uv(
    point: [f32; 3],
    projection: ComponentProjection,
    rule: &ModelMaterialUvProjectionRuleV1,
) -> [f32; 2] {
    let longitudinal_span = (projection.max[projection.longitudinal_axis]
        - projection.min[projection.longitudinal_axis])
        .max(1.0e-6);
    let transverse_span = (projection.max[projection.transverse_axis]
        - projection.min[projection.transverse_axis])
        .max(1.0e-6);
    let longitudinal = (point[projection.longitudinal_axis]
        - projection.min[projection.longitudinal_axis])
        / longitudinal_span;
    let transverse = (point[projection.transverse_axis]
        - projection.min[projection.transverse_axis])
        / transverse_span;
    [
        longitudinal * rule.u_repeats + projection.phase,
        rule.v_min + transverse.clamp(0.0, 1.0) * (rule.v_max - rule.v_min),
    ]
}

fn projected_uv_world(
    point: [f32; 3],
    projection: ComponentProjection,
    repeats_per_metre: f32,
) -> [f32; 2] {
    [
        (point[projection.longitudinal_axis] - projection.min[projection.longitudinal_axis])
            * repeats_per_metre,
        (point[projection.transverse_axis] - projection.min[projection.transverse_axis])
            * repeats_per_metre,
    ]
}

fn push_vertex(
    output: &mut IrPrimitive,
    source: &IrPrimitive,
    source_index: usize,
    uv: [f32; 2],
) -> Result<u32, ModelMaterialUvProjectionErrorV1> {
    let output_index = u32::try_from(output.positions.len()).map_err(|_| {
        error(
            "MATERIAL-UV-PROJECTION-LIMIT-EXCEEDED",
            "primitives.positions",
            "projected vertex index exceeds u32",
        )
    })?;
    output.positions.push(source.positions[source_index]);
    if source.normals.len() == source.positions.len() {
        output.normals.push(source.normals[source_index]);
    }
    if source.tangents.len() == source.positions.len() {
        output.tangents.push([0.0, 0.0, 0.0, 1.0]);
    }
    output.uv0.push(uv);
    if source.joints0.len() == source.positions.len() {
        output.joints0.push(source.joints0[source_index]);
    }
    if source.weights0.len() == source.positions.len() {
        output.weights0.push(source.weights0[source_index]);
    }
    Ok(output_index)
}

fn normalize(vector: [f32; 3]) -> Option<[f32; 3]> {
    let length = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    (length > 1.0e-12).then(|| vector.map(|value| value / length))
}

fn recompute_projected_tangents(output: &mut IrPrimitive) {
    if output.tangents.len() != output.positions.len() {
        return;
    }
    let mut accumulated = vec![[0.0f32; 3]; output.positions.len()];
    for triangle in output.indices.chunks_exact(3) {
        let indices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        let points = indices.map(|index| output.positions[index]);
        let uvs = indices.map(|index| output.uv0[index]);
        let edge1: [f32; 3] = std::array::from_fn(|axis| points[1][axis] - points[0][axis]);
        let edge2: [f32; 3] = std::array::from_fn(|axis| points[2][axis] - points[0][axis]);
        let duv1 = [uvs[1][0] - uvs[0][0], uvs[1][1] - uvs[0][1]];
        let duv2 = [uvs[2][0] - uvs[0][0], uvs[2][1] - uvs[0][1]];
        let denominator = duv1[0] * duv2[1] - duv1[1] * duv2[0];
        let fallback = normalize(edge1).unwrap_or([1.0, 0.0, 0.0]);
        let tangent = if denominator.abs() > 1.0e-12 {
            let inverse = denominator.recip();
            normalize(std::array::from_fn(|axis| {
                (edge1[axis] * duv2[1] - edge2[axis] * duv1[1]) * inverse
            }))
            .unwrap_or(fallback)
        } else {
            fallback
        };
        for index in indices {
            for axis in 0..3 {
                accumulated[index][axis] += tangent[axis];
            }
        }
    }
    for (index, tangent) in accumulated.into_iter().enumerate() {
        let tangent = normalize(tangent).unwrap_or([1.0, 0.0, 0.0]);
        output.tangents[index] = [tangent[0], tangent[1], tangent[2], 1.0];
    }
}

fn source_uv0(
    source: &IrPrimitive,
) -> Result<Vec<[[f32; 2]; 3]>, ModelMaterialUvProjectionErrorV1> {
    if source.uv0.len() != source.positions.len() {
        return Err(error(
            "MATERIAL-UV-PROJECTION-SOURCE-UV0-MISSING",
            format!("primitives[{}].uv0", source.id),
            "SOURCE mode requires one UV0 value per source vertex",
        ));
    }
    Ok(source
        .indices
        .chunks_exact(3)
        .map(|triangle| std::array::from_fn(|corner| source.uv0[triangle[corner] as usize]))
        .collect())
}

fn material_projection(source: &IrPrimitive) -> ComponentProjection {
    let vertices = source.indices.iter().copied().collect::<BTreeSet<_>>();
    component_projection(source, &vertices, 0, false)
}

fn material_box_projection(
    source: &IrPrimitive,
    triangle: &[u32],
    material: ComponentProjection,
) -> ComponentProjection {
    let points =
        std::array::from_fn::<_, 3, _>(|corner| source.positions[triangle[corner] as usize]);
    let edge1 = std::array::from_fn::<_, 3, _>(|axis| points[1][axis] - points[0][axis]);
    let edge2 = std::array::from_fn::<_, 3, _>(|axis| points[2][axis] - points[0][axis]);
    let normal = [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ];
    let normal_length_squared = normal.iter().map(|value| value * value).sum::<f32>();
    if normal_length_squared <= 1.0e-12 {
        return material;
    }

    let dominant_normal_axis = (0..3)
        .max_by(|left, right| {
            normal[*left]
                .abs()
                .partial_cmp(&normal[*right].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .unwrap_or(2);
    let mut surface_axes = (0..3)
        .filter(|axis| *axis != dominant_normal_axis)
        .collect::<Vec<_>>();
    surface_axes.sort_by(|left, right| {
        let left_span = material.max[*left] - material.min[*left];
        let right_span = material.max[*right] - material.min[*right];
        right_span
            .partial_cmp(&left_span)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.cmp(right))
    });
    ComponentProjection {
        longitudinal_axis: surface_axes[0],
        transverse_axis: surface_axes[1],
        ..material
    }
}

/// Resolves one projected UV0 triplet per triangle without mutating geometry.
/// This is shared by the package path and source-bound offline previews so both
/// sample the exact same projection.
pub fn project_primitive_uv0_v1(
    source: &IrPrimitive,
    rule: &ModelMaterialUvProjectionRuleV1,
) -> Result<Vec<[[f32; 2]; 3]>, ModelMaterialUvProjectionErrorV1> {
    if rule.mode == ModelMaterialUvProjectionModeV1::Source {
        return source_uv0(source);
    }
    if rule.mode == ModelMaterialUvProjectionModeV1::MaterialLongAxis {
        let projection = material_projection(source);
        return Ok(source
            .indices
            .chunks_exact(3)
            .map(|triangle| {
                std::array::from_fn(|corner| {
                    projected_uv(
                        source.positions[triangle[corner] as usize],
                        projection,
                        rule,
                    )
                })
            })
            .collect());
    }
    if matches!(
        rule.mode,
        ModelMaterialUvProjectionModeV1::MaterialBox
            | ModelMaterialUvProjectionModeV1::MaterialBoxWorld
    ) {
        let material = material_projection(source);
        return Ok(source
            .indices
            .chunks_exact(3)
            .map(|triangle| {
                let projection = material_box_projection(source, triangle, material);
                std::array::from_fn(|corner| {
                    let point = source.positions[triangle[corner] as usize];
                    if rule.mode == ModelMaterialUvProjectionModeV1::MaterialBoxWorld {
                        projected_uv_world(point, projection, rule.u_repeats)
                    } else {
                        projected_uv(point, projection, rule)
                    }
                })
            })
            .collect());
    }

    let components = connected_components_v1(source, &format!("primitives[{}]", source.id))
        .map_err(|source| {
            error(
                &source
                    .code
                    .replacen("MODEL-COMPONENTS", "MATERIAL-UV-PROJECTION", 1),
                source.path,
                source.message,
            )
        })?;
    let mut component_by_triangle = vec![usize::MAX; source.indices.len() / 3];
    let projections = components
        .iter()
        .enumerate()
        .map(|(component_index, component)| {
            for triangle in &component.triangle_indices {
                component_by_triangle[*triangle] = component_index;
            }
            component_projection(
                source,
                &component.vertex_indices,
                component_index,
                rule.deterministic_u_phase,
            )
        })
        .collect::<Vec<_>>();

    source
        .indices
        .chunks_exact(3)
        .enumerate()
        .map(|(triangle_index, triangle)| {
            let projection = projections[component_by_triangle[triangle_index]];
            Ok(std::array::from_fn(|corner| {
                projected_uv(
                    source.positions[triangle[corner] as usize],
                    projection,
                    rule,
                )
            }))
        })
        .collect()
}

fn project_primitive(
    source: &IrPrimitive,
    rule: &ModelMaterialUvProjectionRuleV1,
) -> Result<IrPrimitive, ModelMaterialUvProjectionErrorV1> {
    let projected_uv0 = project_primitive_uv0_v1(source, rule)?;
    let projection_seams = if matches!(
        rule.mode,
        ModelMaterialUvProjectionModeV1::MaterialBox
            | ModelMaterialUvProjectionModeV1::MaterialBoxWorld
    ) {
        let material = material_projection(source);
        source
            .indices
            .chunks_exact(3)
            .map(|triangle| {
                let projection = material_box_projection(source, triangle, material);
                (projection.longitudinal_axis * 3 + projection.transverse_axis) as u8
            })
            .collect::<Vec<_>>()
    } else {
        vec![0; source.indices.len() / 3]
    };

    let mut output = source.clone();
    output.positions.clear();
    output.normals.clear();
    output.tangents.clear();
    output.uv0.clear();
    output.joints0.clear();
    output.weights0.clear();
    output.indices.clear();
    output.source_was_indexed = true;

    let mut vertex_by_source_and_uv = BTreeMap::<(usize, u32, u32, u8), u32>::new();

    for (triangle_index, triangle) in source.indices.chunks_exact(3).enumerate() {
        for (corner, source_index) in triangle.iter().enumerate() {
            let source_index = *source_index as usize;
            let uv = projected_uv0[triangle_index][corner];
            let key = (
                source_index,
                uv[0].to_bits(),
                uv[1].to_bits(),
                projection_seams[triangle_index],
            );
            let output_index = if let Some(output_index) = vertex_by_source_and_uv.get(&key) {
                *output_index
            } else {
                let output_index = push_vertex(&mut output, source, source_index, uv)?;
                vertex_by_source_and_uv.insert(key, output_index);
                output_index
            };
            output.indices.push(output_index);
        }
    }
    recompute_projected_tangents(&mut output);
    Ok(output)
}

fn material_matches(name: &str, authored_material_id: &str) -> bool {
    name == authored_material_id
        || name
            .strip_prefix(authored_material_id)
            .is_some_and(|suffix| suffix.starts_with("__m2a_"))
}

/// Applies projection to already-authored render primitives. Material
/// assignment must run first so rules can bind by authored material ID.
pub fn apply_model_material_uv_projection_v1(
    ir: &mut AuroraAssetIr,
    expected_separation_sha256: &str,
    document: &ModelMaterialUvProjectionDocumentV1,
) -> Result<ModelMaterialUvProjectionReportV1, ModelMaterialUvProjectionErrorV1> {
    validate_document(ir, expected_separation_sha256, document)?;
    let projection_sha256 = model_material_uv_projection_hash_v1(document)?;
    let source_triangle_count = ir
        .primitives
        .iter()
        .map(|primitive| primitive.indices.len() / 3)
        .sum::<usize>();
    let source_vertex_count = ir
        .primitives
        .iter()
        .map(|primitive| primitive.positions.len())
        .sum::<usize>();
    let mut matched_rule_ids = BTreeSet::new();
    let mut projected_triangle_count = 0usize;

    for primitive in &mut ir.primitives {
        let material_name = primitive
            .material_id
            .and_then(|material_id| ir.materials.get(material_id as usize))
            .and_then(|material| material.name.as_deref());
        let rule = material_name.and_then(|name| {
            document
                .rules
                .iter()
                .find(|rule| material_matches(name, &rule.authored_material_id))
        });
        let Some(rule) = rule else {
            continue;
        };
        matched_rule_ids.insert(rule.authored_material_id.clone());
        if matches!(
            rule.mode,
            ModelMaterialUvProjectionModeV1::ComponentLongAxis
                | ModelMaterialUvProjectionModeV1::MaterialLongAxis
                | ModelMaterialUvProjectionModeV1::MaterialBox
                | ModelMaterialUvProjectionModeV1::MaterialBoxWorld
        ) {
            projected_triangle_count += primitive.indices.len() / 3;
            *primitive = project_primitive(primitive, rule)?;
        }
    }

    for (index, rule) in document.rules.iter().enumerate() {
        if !matched_rule_ids.contains(&rule.authored_material_id) {
            return Err(error(
                "MATERIAL-UV-PROJECTION-MATERIAL-MISSING",
                format!("rules[{index}].authoredMaterialId"),
                "authored material does not exist in the separated render model",
            ));
        }
    }

    let output_triangle_count = ir
        .primitives
        .iter()
        .map(|primitive| primitive.indices.len() / 3)
        .sum::<usize>();
    if output_triangle_count != source_triangle_count {
        return Err(error(
            "MATERIAL-UV-PROJECTION-TRIANGLE-COUNT-CHANGED",
            "primitives",
            format!(
                "projection changed triangle count from {source_triangle_count} to {output_triangle_count}"
            ),
        ));
    }
    let output_vertex_count = ir
        .primitives
        .iter()
        .map(|primitive| primitive.positions.len())
        .sum::<usize>();
    let mut projected_material_ids = matched_rule_ids.into_iter().collect::<Vec<_>>();
    projected_material_ids.sort();
    Ok(ModelMaterialUvProjectionReportV1 {
        schema_version: MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1,
        source_sha256: document.source_sha256.clone(),
        separation_sha256: document.separation_sha256.clone(),
        projection_sha256,
        source_triangle_count: source_triangle_count as u32,
        output_triangle_count: output_triangle_count as u32,
        source_vertex_count: source_vertex_count as u32,
        output_vertex_count: output_vertex_count as u32,
        duplicated_vertex_count: output_vertex_count.saturating_sub(source_vertex_count) as u32,
        projected_triangle_count: projected_triangle_count as u32,
        projected_material_ids,
        source_uv0_preserved_for_unprojected_materials: true,
        geometry_cleanup: false,
    })
}

/// Applies material-local UV projection and refreshes the ingest statistics
/// consumed by the shared model converter. Source identity, triangle count,
/// bounds, hierarchy and collision data remain unchanged.
pub fn apply_model_material_uv_projection_to_ingest_v1(
    ingest: &mut GlbIngestResult,
    expected_separation_sha256: &str,
    document: &ModelMaterialUvProjectionDocumentV1,
) -> Result<ModelMaterialUvProjectionReportV1, ModelMaterialUvProjectionErrorV1> {
    let report = apply_model_material_uv_projection_v1(
        &mut ingest.ir,
        expected_separation_sha256,
        document,
    )?;
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
    ingest.report.statistics.primitives_missing_normals = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.normals.len() != primitive.positions.len())
        .count();
    ingest.report.statistics.primitives_missing_uv0 = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.uv0.len() != primitive.positions.len())
        .count();
    ingest.report.statistics.non_triangle_primitives = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.topology != "TRIANGLES")
        .count();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glb::IrPrimitive;

    fn triangle() -> IrPrimitive {
        IrPrimitive {
            id: 0,
            source_mesh_id: 0,
            source_primitive_index: 0,
            topology: "TRIANGLES".to_owned(),
            material_id: Some(0),
            positions: vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uv0: vec![[0.0, 0.0], [0.1, 0.0], [0.0, 0.1]],
            joints0: Vec::new(),
            weights0: Vec::new(),
            indices: vec![0, 1, 2],
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [4.0, 1.0, 0.0],
            source_was_indexed: true,
        }
    }

    fn two_disconnected_triangles() -> IrPrimitive {
        let mut primitive = triangle();
        primitive.positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
        ];
        primitive.normals = vec![[0.0, 0.0, 1.0]; 6];
        primitive.tangents = vec![[1.0, 0.0, 0.0, 1.0]; 6];
        primitive.uv0 = vec![[0.0, 0.0]; 6];
        primitive.indices = vec![0, 1, 2, 3, 4, 5];
        primitive.bounds_min = [0.0, 0.0, 0.0];
        primitive.bounds_max = [4.0, 1.0, 0.0];
        primitive
    }

    fn indexed_quad() -> IrPrimitive {
        let mut primitive = triangle();
        primitive.positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        primitive.normals = vec![[0.0, 0.0, 1.0]; 4];
        primitive.tangents = vec![[1.0, 0.0, 0.0, 1.0]; 4];
        primitive.uv0 = vec![[0.0, 0.0]; 4];
        primitive.indices = vec![0, 1, 2, 0, 2, 3];
        primitive.bounds_min = [0.0, 0.0, 0.0];
        primitive.bounds_max = [1.0, 1.0, 0.0];
        primitive
    }

    #[test]
    fn component_long_axis_projection_replaces_uv_without_changing_triangles() {
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::ComponentLongAxis,
            u_repeats: 1.0,
            v_min: 0.5,
            v_max: 0.6,
            deterministic_u_phase: false,
        };
        let output = project_primitive(&triangle(), &rule).unwrap();
        assert_eq!(output.indices, vec![0, 1, 2]);
        assert_eq!(output.positions.len(), 3);
        assert_eq!(output.uv0, vec![[0.0, 0.5], [1.0, 0.5], [0.0, 0.6]]);
    }

    #[test]
    fn material_long_axis_projection_keeps_disconnected_geometry_in_one_uv_space() {
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialLongAxis,
            u_repeats: 1.0,
            v_min: 0.2,
            v_max: 0.4,
            deterministic_u_phase: true,
        };

        let projected = project_primitive_uv0_v1(&two_disconnected_triangles(), &rule).unwrap();

        assert_eq!(projected[0], [[0.0, 0.2], [0.25, 0.2], [0.0, 0.4]]);
        assert_eq!(projected[1], [[0.75, 0.2], [1.0, 0.2], [0.75, 0.4]]);
    }

    #[test]
    fn material_box_projection_uses_global_space_and_face_plane() {
        let mut primitive = two_disconnected_triangles();
        primitive.positions = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 0.0, 2.0],
        ];
        primitive.bounds_max = [4.0, 2.0, 2.0];
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialBox,
            u_repeats: 1.0,
            v_min: 0.0,
            v_max: 1.0,
            deterministic_u_phase: true,
        };

        let projected = project_primitive_uv0_v1(&primitive, &rule).unwrap();

        assert_eq!(projected[0], [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        assert_eq!(projected[1], [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
    }

    #[test]
    fn projected_indexed_geometry_reuses_vertices_with_identical_uvs() {
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialLongAxis,
            u_repeats: 1.0,
            v_min: 0.0,
            v_max: 1.0,
            deterministic_u_phase: false,
        };

        let output = project_primitive(&indexed_quad(), &rule).unwrap();

        assert_eq!(output.positions.len(), 4);
        assert_eq!(output.indices.len(), 6);
        assert_eq!(output.indices[0], output.indices[3]);
        assert_eq!(output.indices[2], output.indices[4]);
    }

    #[test]
    fn material_box_projection_splits_vertices_at_projection_plane_seams() {
        let mut primitive = indexed_quad();
        primitive.positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        primitive.indices = vec![0, 1, 2, 0, 1, 3];
        primitive.bounds_max = [1.0, 1.0, 1.0];
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialBox,
            u_repeats: 1.0,
            v_min: 0.0,
            v_max: 1.0,
            deterministic_u_phase: false,
        };

        let output = project_primitive(&primitive, &rule).unwrap();

        assert_eq!(output.positions.len(), 6);
        assert_ne!(output.indices[0], output.indices[3]);
        assert_ne!(output.indices[1], output.indices[4]);
    }

    #[test]
    fn material_box_world_projection_uses_repeatable_world_units() {
        let primitive = two_disconnected_triangles();
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialBoxWorld,
            u_repeats: 0.5,
            v_min: 0.0,
            v_max: 1.0,
            deterministic_u_phase: false,
        };

        let projected = project_primitive_uv0_v1(&primitive, &rule).unwrap();

        assert_eq!(projected[0][1][0] - projected[0][0][0], 0.5);
        assert_eq!(projected[1][1][0] - projected[1][0][0], 0.5);
        assert_eq!(projected[1][0][0] - projected[0][0][0], 1.5);
        assert_eq!(projected[0][2][1] - projected[0][0][1], 0.5);
    }

    #[test]
    fn material_box_world_projection_splits_only_projection_plane_seams() {
        let mut primitive = indexed_quad();
        primitive.positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        primitive.indices = vec![0, 1, 2, 0, 1, 3];
        primitive.bounds_max = [1.0, 1.0, 1.0];
        let rule = ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialBoxWorld,
            u_repeats: 0.5,
            v_min: 0.0,
            v_max: 1.0,
            deterministic_u_phase: false,
        };

        let output = project_primitive(&primitive, &rule).unwrap();

        assert_eq!(output.positions.len(), 6);
        assert_ne!(output.indices[0], output.indices[3]);
        assert_ne!(output.indices[1], output.indices[4]);
    }

    #[test]
    fn projection_axes_follow_longest_component_extent() {
        assert_eq!(projection_axes([0.0; 3], [6.0, 2.0, 1.0]), (0, 1));
        assert_eq!(projection_axes([0.0; 3], [1.0, 7.0, 3.0]), (1, 2));
    }
}
