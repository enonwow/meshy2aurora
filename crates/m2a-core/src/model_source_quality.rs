//! Deterministic source-quality preflight for render-model authoring.
//!
//! This gate measures source limitations before Aurora packaging. It does not
//! mutate geometry or textures and does not replace an owner-owned visual
//! proof in Toolset/NWN.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    glb::AuroraAssetIr,
    model_components::connected_components_v1,
    model_texture_authoring::{TextureMipReadabilityStatusV1, TextureMipReadabilityV1},
};

pub const MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelSourceQualityStatusV1 {
    Pass,
    Warning,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceMaterialQualityInputV1 {
    pub material_id: u32,
    /// `None` is a deliberate color-only material. A missing material entry is
    /// different and fails closed.
    pub texture_readability: Option<TextureMipReadabilityV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSourceQualityInputV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub materials: Vec<SourceMaterialQualityInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSourceQualityDiagnosticV1 {
    pub code: String,
    pub status: ModelSourceQualityStatusV1,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSourceQualityReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub status: ModelSourceQualityStatusV1,
    pub triangle_count: u32,
    pub component_count: u32,
    pub tiny_component_count: u32,
    pub components_per_thousand_triangles: u32,
    pub missing_uv_triangle_count: u32,
    pub degenerate_uv_triangle_count: u32,
    pub uv_outside_unit_triangle_count: u32,
    pub minimum_texels_per_meter: u32,
    pub weighted_texels_per_meter: u32,
    pub maximum_texels_per_meter: u32,
    pub diagnostics: Vec<ModelSourceQualityDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSourceQualityErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelSourceQualityErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelSourceQualityErrorV1 {}

pub fn inspect_model_source_quality_v1(
    ir: &AuroraAssetIr,
    input: &ModelSourceQualityInputV1,
) -> Result<ModelSourceQualityReportV1, ModelSourceQualityErrorV1> {
    validate_input(ir, input)?;
    let materials = input
        .materials
        .iter()
        .map(|material| (material.material_id, material))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    let mut triangle_count = 0usize;
    let mut component_count = 0usize;
    let mut tiny_component_count = 0usize;
    let mut missing_uv_triangle_count = 0usize;
    let mut degenerate_uv_triangle_count = 0usize;
    let mut uv_outside_unit_triangle_count = 0usize;
    let mut density_samples = Vec::<(f64, f64)>::new();

    for primitive in &ir.primitives {
        if primitive.topology != "TRIANGLES" || !primitive.indices.len().is_multiple_of(3) {
            return Err(error(
                "SOURCE-QUALITY-TOPOLOGY-UNSUPPORTED",
                format!("primitives[{}]", primitive.id),
                "source quality requires indexed TRIANGLES",
            ));
        }
        let primitive_triangles = primitive.indices.len() / 3;
        triangle_count += primitive_triangles;
        let components =
            connected_components_v1(primitive, &format!("primitives[{}].indices", primitive.id))
                .map_err(|source| {
                    error(
                        "SOURCE-QUALITY-COMPONENTS-FAILED",
                        source.path,
                        source.message,
                    )
                })?;
        component_count += components.len();
        tiny_component_count += components
            .iter()
            .filter(|component| component.triangle_indices.len() <= 2)
            .count();
        let material = primitive
            .material_id
            .map(|material_id| materials[&material_id]);
        let texture = material.and_then(|material| material.texture_readability.as_ref());
        let has_uv = primitive.uv0.len() == primitive.positions.len();
        if !has_uv {
            missing_uv_triangle_count += primitive_triangles;
        }
        for triangle in primitive.indices.chunks_exact(3) {
            let indices = triangle
                .iter()
                .map(|index| usize::try_from(*index).expect("u32 fits usize on supported hosts"))
                .collect::<Vec<_>>();
            if indices
                .iter()
                .any(|index| *index >= primitive.positions.len())
            {
                return Err(error(
                    "SOURCE-QUALITY-INDEX-OOB",
                    format!("primitives[{}].indices", primitive.id),
                    "triangle references a missing position",
                ));
            }
            if !has_uv {
                continue;
            }
            let uv = [
                primitive.uv0[indices[0]],
                primitive.uv0[indices[1]],
                primitive.uv0[indices[2]],
            ];
            if uv.iter().flatten().any(|value| !value.is_finite()) {
                return Err(error(
                    "SOURCE-QUALITY-UV-NONFINITE",
                    format!("primitives[{}].uv0", primitive.id),
                    "UV coordinates must be finite",
                ));
            }
            if uv
                .iter()
                .flatten()
                .any(|value| !(0.0..=1.0).contains(value))
            {
                uv_outside_unit_triangle_count += 1;
            }
            let uv_area = triangle_area_2d(uv);
            if uv_area <= 1.0e-12 {
                degenerate_uv_triangle_count += 1;
                continue;
            }
            let Some(texture) = texture else { continue };
            let positions = [
                primitive.positions[indices[0]],
                primitive.positions[indices[1]],
                primitive.positions[indices[2]],
            ];
            let geometry_area = triangle_area_3d(positions);
            if geometry_area <= 1.0e-12 {
                continue;
            }
            let pixel_area = f64::from(texture.source_width) * f64::from(texture.source_height);
            let density = (uv_area * pixel_area / geometry_area).sqrt();
            if density.is_finite() {
                density_samples.push((density, geometry_area));
            }
        }
    }

    let components_per_thousand_triangles = if triangle_count == 0 {
        0
    } else {
        ((component_count as u64 * 1_000) / triangle_count as u64) as u32
    };
    let minimum_texels_per_meter = density_samples
        .iter()
        .map(|(density, _)| *density)
        .reduce(f64::min)
        .unwrap_or(0.0)
        .round() as u32;
    let maximum_texels_per_meter = density_samples
        .iter()
        .map(|(density, _)| *density)
        .reduce(f64::max)
        .unwrap_or(0.0)
        .round() as u32;
    let total_area = density_samples.iter().map(|(_, area)| *area).sum::<f64>();
    let weighted_texels_per_meter = if total_area == 0.0 {
        0
    } else {
        (density_samples
            .iter()
            .map(|(density, area)| density * area)
            .sum::<f64>()
            / total_area)
            .round() as u32
    };

    if missing_uv_triangle_count > 0 {
        push(
            &mut diagnostics,
            "SOURCE-QUALITY-UV-MISSING",
            ModelSourceQualityStatusV1::Blocked,
            "primitives.uv0",
            format!("{missing_uv_triangle_count} triangles have no complete UV0 stream"),
        );
    }
    if degenerate_uv_triangle_count > 0 {
        let ratio = degenerate_uv_triangle_count * 10_000 / triangle_count.max(1);
        push(
            &mut diagnostics,
            "SOURCE-QUALITY-UV-DEGENERATE",
            if ratio > 200 {
                ModelSourceQualityStatusV1::Blocked
            } else {
                ModelSourceQualityStatusV1::Warning
            },
            "primitives.uv0",
            format!(
                "{degenerate_uv_triangle_count} triangles ({ratio} bp) collapse to zero UV area"
            ),
        );
    }
    if uv_outside_unit_triangle_count > 0 {
        push(
            &mut diagnostics,
            "SOURCE-QUALITY-UV-TILED",
            ModelSourceQualityStatusV1::Warning,
            "primitives.uv0",
            format!(
                "{uv_outside_unit_triangle_count} triangles use UV coordinates outside 0..1; verify wrap/TXI policy"
            ),
        );
    }
    if components_per_thousand_triangles > 750 {
        push(
            &mut diagnostics,
            "SOURCE-QUALITY-FRAGMENTATION-EXTREME",
            ModelSourceQualityStatusV1::Blocked,
            "primitives.indices",
            format!(
                "fragmentation is {components_per_thousand_triangles} components per 1000 triangles"
            ),
        );
    } else if components_per_thousand_triangles > 250 || tiny_component_count > 256 {
        push(
            &mut diagnostics,
            "SOURCE-QUALITY-FRAGMENTATION-HIGH",
            ModelSourceQualityStatusV1::Warning,
            "primitives.indices",
            format!(
                "{component_count} components, including {tiny_component_count} components of at most two triangles"
            ),
        );
    }
    if !density_samples.is_empty() {
        if weighted_texels_per_meter < 16 {
            push(
                &mut diagnostics,
                "SOURCE-QUALITY-TEXEL-DENSITY-BLOCKED",
                ModelSourceQualityStatusV1::Blocked,
                "materials.texture",
                format!("weighted density is only {weighted_texels_per_meter} texels/metre"),
            );
        } else if weighted_texels_per_meter < 64 {
            push(
                &mut diagnostics,
                "SOURCE-QUALITY-TEXEL-DENSITY-LOW",
                ModelSourceQualityStatusV1::Warning,
                "materials.texture",
                format!("weighted density is {weighted_texels_per_meter} texels/metre"),
            );
        }
        if minimum_texels_per_meter > 0
            && maximum_texels_per_meter / minimum_texels_per_meter.max(1) > 16
        {
            push(
                &mut diagnostics,
                "SOURCE-QUALITY-TEXEL-DENSITY-INCONSISTENT",
                ModelSourceQualityStatusV1::Warning,
                "materials.texture",
                format!(
                    "density spans {minimum_texels_per_meter}..{maximum_texels_per_meter} texels/metre"
                ),
            );
        }
    }
    for material in &input.materials {
        let Some(texture) = &material.texture_readability else {
            continue;
        };
        match texture.status {
            TextureMipReadabilityStatusV1::Readable => {}
            TextureMipReadabilityStatusV1::LowContrast => push(
                &mut diagnostics,
                "SOURCE-QUALITY-CONTRAST-LOW",
                ModelSourceQualityStatusV1::Warning,
                format!("materials[{}].textureReadability", material.material_id),
                format!(
                    "16x16 mip luma deviation is {} milli",
                    texture.mip_16_luma_stddev_milli
                ),
            ),
            TextureMipReadabilityStatusV1::Flat => push(
                &mut diagnostics,
                "SOURCE-QUALITY-CONTRAST-FLAT",
                ModelSourceQualityStatusV1::Blocked,
                format!("materials[{}].textureReadability", material.material_id),
                "texture has no useful macro contrast",
            ),
        }
    }
    let status = diagnostics
        .iter()
        .map(|item| item.status)
        .max()
        .unwrap_or(ModelSourceQualityStatusV1::Pass);
    Ok(ModelSourceQualityReportV1 {
        schema_version: MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1,
        source_sha256: input.source_sha256.clone(),
        status,
        triangle_count: as_u32(triangle_count, "triangleCount")?,
        component_count: as_u32(component_count, "componentCount")?,
        tiny_component_count: as_u32(tiny_component_count, "tinyComponentCount")?,
        components_per_thousand_triangles,
        missing_uv_triangle_count: as_u32(missing_uv_triangle_count, "missingUvTriangleCount")?,
        degenerate_uv_triangle_count: as_u32(
            degenerate_uv_triangle_count,
            "degenerateUvTriangleCount",
        )?,
        uv_outside_unit_triangle_count: as_u32(
            uv_outside_unit_triangle_count,
            "uvOutsideUnitTriangleCount",
        )?,
        minimum_texels_per_meter,
        weighted_texels_per_meter,
        maximum_texels_per_meter,
        diagnostics,
    })
}

fn validate_input(
    ir: &AuroraAssetIr,
    input: &ModelSourceQualityInputV1,
) -> Result<(), ModelSourceQualityErrorV1> {
    if input.schema_version != MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1 {
        return Err(error(
            "SOURCE-QUALITY-SCHEMA-UNSUPPORTED",
            "schemaVersion",
            "expected schema version 1",
        ));
    }
    if input.source_sha256 != ir.source.sha256 {
        return Err(error(
            "SOURCE-QUALITY-SOURCE-MISMATCH",
            "sourceSha256",
            "quality input belongs to another GLB",
        ));
    }
    let mut materials = BTreeMap::new();
    for (index, material) in input.materials.iter().enumerate() {
        if materials.insert(material.material_id, index).is_some() {
            return Err(error(
                "SOURCE-QUALITY-MATERIAL-DUPLICATE",
                format!("materials[{index}].materialId"),
                "material quality binding must be unique",
            ));
        }
    }
    for primitive in &ir.primitives {
        if let Some(material_id) = primitive.material_id
            && !materials.contains_key(&material_id)
        {
            return Err(error(
                "SOURCE-QUALITY-MATERIAL-MISSING",
                format!("primitives[{}].materialId", primitive.id),
                format!("material {material_id} has no quality binding"),
            ));
        }
    }
    Ok(())
}

fn triangle_area_2d(value: [[f32; 2]; 3]) -> f64 {
    let ax = f64::from(value[1][0] - value[0][0]);
    let ay = f64::from(value[1][1] - value[0][1]);
    let bx = f64::from(value[2][0] - value[0][0]);
    let by = f64::from(value[2][1] - value[0][1]);
    (ax * by - ay * bx).abs() * 0.5
}

fn triangle_area_3d(value: [[f32; 3]; 3]) -> f64 {
    let a = [
        f64::from(value[1][0] - value[0][0]),
        f64::from(value[1][1] - value[0][1]),
        f64::from(value[1][2] - value[0][2]),
    ];
    let b = [
        f64::from(value[2][0] - value[0][0]),
        f64::from(value[2][1] - value[0][1]),
        f64::from(value[2][2] - value[0][2]),
    ];
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    (cross.iter().map(|value| value * value).sum::<f64>()).sqrt() * 0.5
}

fn push(
    diagnostics: &mut Vec<ModelSourceQualityDiagnosticV1>,
    code: &str,
    status: ModelSourceQualityStatusV1,
    path: impl Into<String>,
    message: impl Into<String>,
) {
    diagnostics.push(ModelSourceQualityDiagnosticV1 {
        code: code.to_owned(),
        status,
        path: path.into(),
        message: message.into(),
    });
}

fn as_u32(value: usize, path: &str) -> Result<u32, ModelSourceQualityErrorV1> {
    u32::try_from(value)
        .map_err(|_| error("SOURCE-QUALITY-COUNT-OVERFLOW", path, "count exceeds u32"))
}
fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ModelSourceQualityErrorV1 {
    ModelSourceQualityErrorV1 {
        schema_version: MODEL_SOURCE_QUALITY_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}
