//! Deterministic partitioning of shared render-model IR into binary-MDL-safe
//! mesh streams.
//!
//! The product triangle budget applies to the whole model. Aurora's binary MDL
//! format independently limits one mesh stream to 65,535 index entries and a
//! 16-bit vertex address space. This module bridges those two contracts without
//! removing triangles or changing material, hierarchy, deformation or surface
//! metadata.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    mdl::NWN_EE_MAX_MESH_INDEX_COUNT_V1,
    model_ir::{AuroraModelIrV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSegmentationReportV1 {
    pub schema_version: u32,
    pub source_segment_count: usize,
    pub output_segment_count: usize,
    pub split_segment_count: usize,
    pub triangle_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSegmentationErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelSegmentationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelSegmentationErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ModelSegmentationErrorV1 {
    ModelSegmentationErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

/// Partitions only mesh streams that exceed the binary MDL per-stream
/// boundary. Streams already safe for the writer remain byte-for-byte equal.
///
/// The operation preserves the total ordered triangle sequence and copies all
/// vertex, material, hierarchy, deformation and optional surface metadata.
pub fn segment_model_for_binary_mdl_v1(
    model: &mut AuroraModelIrV1,
) -> Result<ModelSegmentationReportV1, ModelSegmentationErrorV1> {
    if model.segments.is_empty() {
        return Err(error(
            "M2A-MODEL-SEGMENTATION-EMPTY",
            "model.segments",
            "render model requires at least one segment",
        ));
    }

    let source_triangle_count =
        model
            .segments
            .iter()
            .enumerate()
            .try_fold(0usize, |sum, (segment_index, segment)| {
                validate_source_segment(segment, segment_index)?;
                sum.checked_add(segment.indices.len() / 3).ok_or_else(|| {
                    error(
                        "M2A-MODEL-SEGMENTATION-OVERFLOW",
                        "model.segments",
                        "source triangle count overflow",
                    )
                })
            })?;
    let source_segment_count = model.segments.len();
    let split_segment_count = model
        .segments
        .iter()
        .filter(|segment| !segment_is_writer_safe(segment))
        .count();
    if split_segment_count == 0 {
        return Ok(ModelSegmentationReportV1 {
            schema_version: 1,
            source_segment_count,
            output_segment_count: source_segment_count,
            split_segment_count: 0,
            triangle_count: source_triangle_count,
        });
    }

    let source_segments = std::mem::take(&mut model.segments);
    let mut output_segments = Vec::new();
    for segment in source_segments {
        if segment_is_writer_safe(&segment) {
            output_segments.push(segment);
        } else {
            partition_segment(segment, &mut output_segments)?;
        }
    }

    for (segment_index, segment) in output_segments.iter_mut().enumerate() {
        segment.segment_id = u32::try_from(segment_index + 1).map_err(|_| {
            error(
                "M2A-MODEL-SEGMENTATION-OVERFLOW",
                "model.segments",
                "partition segment id exceeds u32",
            )
        })?;
    }
    let output_triangle_count = output_segments.iter().try_fold(0usize, |sum, segment| {
        sum.checked_add(segment.indices.len() / 3).ok_or_else(|| {
            error(
                "M2A-MODEL-SEGMENTATION-OVERFLOW",
                "model.segments",
                "partition triangle count overflow",
            )
        })
    })?;
    if output_triangle_count != source_triangle_count || output_segments.is_empty() {
        return Err(error(
            "M2A-MODEL-SEGMENTATION-COUNT",
            "model.segments",
            format!(
                "partition changed triangle count from {source_triangle_count} to {output_triangle_count}"
            ),
        ));
    }
    let output_segment_count = output_segments.len();
    model.segments = output_segments;
    Ok(ModelSegmentationReportV1 {
        schema_version: 1,
        source_segment_count,
        output_segment_count,
        split_segment_count,
        triangle_count: output_triangle_count,
    })
}

fn validate_source_segment(
    segment: &AuroraModelSegmentV1,
    segment_index: usize,
) -> Result<(), ModelSegmentationErrorV1> {
    let path = format!("model.segments[{segment_index}]");
    let vertex_count = segment.positions.len();
    if segment.indices.is_empty()
        || !segment.indices.len().is_multiple_of(3)
        || vertex_count != segment.normals.len()
        || vertex_count != segment.uv0.len()
        || segment
            .tangents
            .as_ref()
            .is_some_and(|values| values.len() != vertex_count)
        || (segment.deformation == AuroraSegmentDeformationV1::Skin
            && segment.weights.len() != vertex_count)
        || (segment.deformation == AuroraSegmentDeformationV1::Rigid && !segment.weights.is_empty())
        || (!segment.face_surface_ids.is_empty()
            && segment.face_surface_ids.len() != segment.indices.len() / 3)
    {
        return Err(error(
            "M2A-MODEL-SEGMENTATION-SOURCE-INVALID",
            path,
            "source segment arrays do not form complete indexed triangles",
        ));
    }
    if segment
        .indices
        .iter()
        .any(|index| usize::try_from(*index).map_or(true, |index| index >= vertex_count))
    {
        return Err(error(
            "M2A-MODEL-SEGMENTATION-INDEX",
            path,
            "source vertex index escapes the segment arrays",
        ));
    }
    Ok(())
}

fn segment_is_writer_safe(segment: &AuroraModelSegmentV1) -> bool {
    segment.indices.len() <= NWN_EE_MAX_MESH_INDEX_COUNT_V1
        && segment.positions.len() <= usize::from(u16::MAX)
}

fn partition_segment(
    segment: AuroraModelSegmentV1,
    output_segments: &mut Vec<AuroraModelSegmentV1>,
) -> Result<(), ModelSegmentationErrorV1> {
    let triangles_per_stream = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 3;
    let source_triangle_total = segment.indices.len() / 3;
    for triangle_start in (0..source_triangle_total).step_by(triangles_per_stream) {
        let triangle_end = (triangle_start + triangles_per_stream).min(source_triangle_total);
        let source_indices = &segment.indices[triangle_start * 3..triangle_end * 3];
        let mut remap = vec![u32::MAX; segment.positions.len()];
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tangents = segment.tangents.as_ref().map(|_| Vec::new());
        let mut uv0 = Vec::new();
        let mut weights = Vec::new();
        let mut indices = Vec::with_capacity(source_indices.len());
        for &source_index in source_indices {
            let source_vertex = usize::try_from(source_index).map_err(|_| {
                error(
                    "M2A-MODEL-SEGMENTATION-INDEX",
                    "model.segments",
                    "source vertex index does not fit this platform",
                )
            })?;
            let output_index = if remap[source_vertex] == u32::MAX {
                let output_index = u32::try_from(positions.len()).map_err(|_| {
                    error(
                        "M2A-MODEL-SEGMENTATION-OVERFLOW",
                        "model.segments",
                        "partition vertex index exceeds u32",
                    )
                })?;
                remap[source_vertex] = output_index;
                positions.push(segment.positions[source_vertex]);
                normals.push(segment.normals[source_vertex]);
                uv0.push(segment.uv0[source_vertex]);
                if let (Some(source), Some(output)) = (segment.tangents.as_ref(), tangents.as_mut())
                {
                    output.push(source[source_vertex]);
                }
                if segment.deformation == AuroraSegmentDeformationV1::Skin {
                    weights.push(segment.weights[source_vertex].clone());
                }
                output_index
            } else {
                remap[source_vertex]
            };
            indices.push(output_index);
        }
        if positions.len() > usize::from(u16::MAX) || indices.len() > NWN_EE_MAX_MESH_INDEX_COUNT_V1
        {
            return Err(error(
                "M2A-MODEL-SEGMENTATION-LIMIT",
                "model.segments",
                "deterministic partition still exceeds the binary MDL mesh boundary",
            ));
        }
        output_segments.push(AuroraModelSegmentV1 {
            segment_id: 0,
            material_slot: segment.material_slot,
            deformation: segment.deformation.clone(),
            parent_node_id: segment.parent_node_id,
            cast_shadow: segment.cast_shadow,
            positions,
            normals,
            tangents,
            uv0,
            indices,
            face_surface_ids: if segment.face_surface_ids.is_empty() {
                Vec::new()
            } else {
                segment.face_surface_ids[triangle_start..triangle_end].to_vec()
            },
            weights,
        });
    }
    Ok(())
}
