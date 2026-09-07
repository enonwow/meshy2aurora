//! Deterministic target-surface anatomy used before reference-supermodel fitting.
//!
//! The analysis separates the primary deforming body from disconnected cards,
//! fur and ornaments.  Only the primary body is authoritative for joint pivots;
//! auxiliary components receive an explicit projection/binding provenance.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    profile_a::Bounds3V1, reference_supermodel_generic::ReferenceSupermodelGenericErrorV2,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSurfaceComponentV1 {
    pub component_index: usize,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub surface_area: f32,
    pub bounds: Bounds3V1,
    pub role: String,
    pub binding_provenance: String,
    pub analysis_group_vertex_fraction: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSurfaceLandmarkV1 {
    pub landmark_id: String,
    pub position: [f32; 3],
    pub confidence: f32,
    pub provenance: String,
    pub semantic_region: String,
    pub residual_fraction: f32,
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSurfaceAnatomyV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub content_sha256: String,
    pub canonical_lateral_axis: String,
    pub canonical_forward_axis: String,
    pub canonical_up_axis: String,
    pub surface_vertex_count: usize,
    pub surface_triangle_count: usize,
    pub component_count: usize,
    pub virtual_weld_group_count: usize,
    pub virtual_welded_vertex_count: usize,
    pub primary_component_index: usize,
    pub primary_component_vertex_count: usize,
    pub authoritative_component_indices: Vec<usize>,
    pub authoritative_component_count: usize,
    pub authoritative_surface_vertex_count: usize,
    pub analysis_surface_vertex_count: usize,
    pub authoritative_vertex_fraction: f32,
    pub auxiliary_component_count: usize,
    pub duplicate_position_group_count: usize,
    pub symmetry_plane_x: f32,
    pub symmetry_confidence: f32,
    pub primary_bounds: Bounds3V1,
    pub authoritative_bounds: Bounds3V1,
    pub components: Vec<TargetSurfaceComponentV1>,
    pub medial_axis: Vec<TargetSurfaceLandmarkV1>,
    pub ground_contact_candidates: Vec<TargetSurfaceLandmarkV1>,
    pub ambiguities: Vec<String>,
    pub family_or_resref_rule_used: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TargetSurfaceAnatomyArtifactV1 {
    pub report: TargetSurfaceAnatomyV1,
    pub joint_fit_vertex_indices: Vec<usize>,
    pub vertex_component_indices: Vec<usize>,
}

pub fn analyze_target_surface_anatomy_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Result<TargetSurfaceAnatomyArtifactV1, ReferenceSupermodelGenericErrorV2> {
    validate_surface(positions, indices)?;
    let surface_bounds = bounds(positions)?;
    let surface_diagonal = distance(surface_bounds.min, surface_bounds.max).max(1.0e-6);
    let weld = build_virtual_weld_map_v2(positions, surface_diagonal * 1.0e-5);
    let component_vertices = virtually_welded_components_v2(
        positions.len(),
        indices,
        &weld.vertex_to_group,
        weld.group_count,
    );
    let vertex_to_component = component_vertices
        .iter()
        .enumerate()
        .flat_map(|(component, vertices)| vertices.iter().map(move |vertex| (*vertex, component)))
        .collect::<BTreeMap<_, _>>();
    let mut triangle_count = vec![0usize; component_vertices.len()];
    let mut surface_area = vec![0.0_f32; component_vertices.len()];
    for triangle in indices.chunks_exact(3) {
        let component = vertex_to_component[&(triangle[0] as usize)];
        triangle_count[component] += 1;
        surface_area[component] += triangle_area(
            positions[triangle[0] as usize],
            positions[triangle[1] as usize],
            positions[triangle[2] as usize],
        );
    }
    let primary_component_index = (0..component_vertices.len())
        .max_by(|left, right| {
            surface_area[*left]
                .total_cmp(&surface_area[*right])
                .then(
                    component_vertices[*left]
                        .len()
                        .cmp(&component_vertices[*right].len()),
                )
                .then_with(|| right.cmp(left))
        })
        .expect("validated non-empty surface");
    let primary_component_vertex_indices = component_vertices[primary_component_index].clone();
    let primary_positions = primary_component_vertex_indices
        .iter()
        .map(|vertex| positions[*vertex])
        .collect::<Vec<_>>();
    let primary_bounds = bounds(&primary_positions)?;
    let primary_diagonal = distance(primary_bounds.min, primary_bounds.max).max(1.0e-6);
    let primary_area = surface_area[primary_component_index].max(1.0e-12);
    let primary_vertex_count = primary_component_vertex_indices.len().max(1);
    let authoritative_component_indices = component_vertices
        .iter()
        .enumerate()
        .filter_map(|(component, vertices)| {
            let component_positions = vertices
                .iter()
                .map(|vertex| positions[*vertex])
                .collect::<Vec<_>>();
            let component_bounds = bounds_unchecked(&component_positions);
            let diagonal_ratio =
                distance(component_bounds.min, component_bounds.max) / primary_diagonal;
            let area_ratio = surface_area[component] / primary_area;
            let vertex_ratio = vertices.len() as f32 / primary_vertex_count as f32;
            (component == primary_component_index
                || (area_ratio >= 0.15 && vertex_ratio >= 0.05)
                || (area_ratio >= 0.05 && diagonal_ratio >= 0.5))
                .then_some(component)
        })
        .collect::<Vec<_>>();
    let authoritative_set = authoritative_component_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let authoritative_component_count = authoritative_component_indices.len();
    let authoritative_vertex_indices = authoritative_component_indices
        .iter()
        .flat_map(|component| component_vertices[*component].iter().copied())
        .collect::<Vec<_>>();
    let authoritative_set_vertices = authoritative_vertex_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let joint_fit_vertex_indices = weld
        .representatives
        .iter()
        .copied()
        .filter(|vertex| authoritative_set_vertices.contains(vertex))
        .collect::<Vec<_>>();
    let authoritative_positions = joint_fit_vertex_indices
        .iter()
        .map(|vertex| positions[*vertex])
        .collect::<Vec<_>>();
    let authoritative_bounds = bounds(&authoritative_positions)?;
    let symmetry_plane_x = robust_midpoint(
        authoritative_positions
            .iter()
            .map(|point| point[0])
            .collect(),
    );
    let symmetry_confidence = symmetry_confidence(&authoritative_positions, symmetry_plane_x);
    let medial_axis = medial_axis(&authoritative_positions, authoritative_bounds);
    let ground_contact_candidates = ground_contacts(&authoritative_positions, authoritative_bounds);
    let components = component_vertices
        .iter()
        .enumerate()
        .map(|(component_index, vertices)| {
            let component_positions = vertices
                .iter()
                .map(|vertex| positions[*vertex])
                .collect::<Vec<_>>();
            Ok(TargetSurfaceComponentV1 {
                component_index,
                vertex_count: vertices.len(),
                triangle_count: triangle_count[component_index],
                surface_area: surface_area[component_index],
                bounds: bounds(&component_positions)?,
                role: if component_index == primary_component_index {
                    "PRIMARY_DEFORMING_BODY"
                } else if authoritative_set.contains(&component_index) {
                    "AUTHORITATIVE_STRUCTURAL_SURFACE"
                } else {
                    "AUXILIARY_SURFACE"
                }
                .to_owned(),
                binding_provenance: if authoritative_set.contains(&component_index) {
                    "virtual_welded_authoritative_body_proxy"
                } else {
                    "project_to_semantically_compatible_authoritative_surface"
                }
                .to_owned(),
                analysis_group_vertex_fraction: vertices.len() as f32 / positions.len() as f32,
            })
        })
        .collect::<Result<Vec<_>, ReferenceSupermodelGenericErrorV2>>()?;
    let duplicate_position_group_count = {
        let mut counts = BTreeMap::<[u32; 3], usize>::new();
        for position in positions {
            *counts.entry(position.map(f32::to_bits)).or_default() += 1;
        }
        counts.values().filter(|count| **count > 1).count()
    };
    let mut ambiguities = Vec::new();
    if symmetry_confidence < 0.35 {
        ambiguities.push("LOW_SYMMETRY_CONFIDENCE".to_owned());
    }
    if medial_axis.len() < 5 {
        ambiguities.push("MEDIAL_AXIS_SPARSE".to_owned());
    }
    if ground_contact_candidates.is_empty() {
        ambiguities.push("GROUND_CONTACTS_MISSING".to_owned());
    }
    if joint_fit_vertex_indices.len() < 32 {
        ambiguities.push("ANALYSIS_SURFACE_SPARSE".to_owned());
    }
    let status = if ambiguities.is_empty() {
        "READY"
    } else {
        "NEEDS_AUTHORING"
    };
    let mut report = TargetSurfaceAnatomyV1 {
        schema_version: 2,
        algorithm: "VIRTUAL_WELD_BODY_PROXY_ANATOMY_V2".to_owned(),
        status: status.to_owned(),
        content_sha256: String::new(),
        canonical_lateral_axis: "X".to_owned(),
        canonical_forward_axis: "Y".to_owned(),
        canonical_up_axis: "Z".to_owned(),
        surface_vertex_count: positions.len(),
        surface_triangle_count: indices.len() / 3,
        component_count: components.len(),
        virtual_weld_group_count: weld.group_count,
        virtual_welded_vertex_count: positions.len().saturating_sub(weld.group_count),
        primary_component_index,
        primary_component_vertex_count: primary_component_vertex_indices.len(),
        authoritative_component_indices,
        authoritative_component_count,
        authoritative_surface_vertex_count: authoritative_vertex_indices.len(),
        analysis_surface_vertex_count: joint_fit_vertex_indices.len(),
        authoritative_vertex_fraction: authoritative_vertex_indices.len() as f32
            / positions.len() as f32,
        auxiliary_component_count: components
            .len()
            .saturating_sub(authoritative_component_count),
        duplicate_position_group_count,
        symmetry_plane_x,
        symmetry_confidence,
        primary_bounds,
        authoritative_bounds,
        components,
        medial_axis,
        ground_contact_candidates,
        ambiguities,
        family_or_resref_rule_used: false,
    };
    report.content_sha256 = sha256_json(&report)?;
    Ok(TargetSurfaceAnatomyArtifactV1 {
        report,
        joint_fit_vertex_indices,
        vertex_component_indices: (0..positions.len())
            .map(|vertex| vertex_to_component[&vertex])
            .collect(),
    })
}

fn medial_axis(positions: &[[f32; 3]], bounds: Bounds3V1) -> Vec<TargetSurfaceLandmarkV1> {
    const SLICE_COUNT: usize = 9;
    let extent = (bounds.max[1] - bounds.min[1]).max(1.0e-6);
    (0..SLICE_COUNT)
        .filter_map(|slice| {
            let low = bounds.min[1] + extent * slice as f32 / SLICE_COUNT as f32;
            let high = bounds.min[1] + extent * (slice + 1) as f32 / SLICE_COUNT as f32;
            let points = positions
                .iter()
                .copied()
                .filter(|point| point[1] >= low && (slice + 1 == SLICE_COUNT || point[1] < high))
                .collect::<Vec<_>>();
            if points.is_empty() {
                return None;
            }
            Some(TargetSurfaceLandmarkV1 {
                landmark_id: format!("medial_axis_{slice}"),
                position: [
                    median_coordinate(&points, 0),
                    median_coordinate(&points, 1),
                    median_coordinate(&points, 2),
                ],
                confidence: (points.len() as f32 / 64.0).clamp(0.1, 1.0),
                provenance: "virtual_welded_body_proxy_slice_median".to_owned(),
                semantic_region: "central_body".to_owned(),
                residual_fraction: 0.0,
                constraints: vec![
                    "target_symmetry_plane".to_owned(),
                    "authoritative_body_proxy".to_owned(),
                ],
            })
        })
        .collect()
}

fn median_coordinate(points: &[[f32; 3]], axis: usize) -> f32 {
    let mut values = points.iter().map(|point| point[axis]).collect::<Vec<_>>();
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) * 0.5
    } else {
        values[middle]
    }
}

fn ground_contacts(positions: &[[f32; 3]], bounds: Bounds3V1) -> Vec<TargetSurfaceLandmarkV1> {
    let height = (bounds.max[2] - bounds.min[2]).max(1.0e-6);
    let ceiling = bounds.min[2] + height * 0.04;
    let mut low = positions
        .iter()
        .copied()
        .filter(|point| point[2] <= ceiling)
        .collect::<Vec<_>>();
    low.sort_by(|left, right| {
        left[1]
            .total_cmp(&right[1])
            .then(left[0].total_cmp(&right[0]))
            .then(left[2].total_cmp(&right[2]))
    });
    if low.is_empty() {
        return Vec::new();
    }
    let bucket_count = low.len().min(8);
    (0..bucket_count)
        .filter_map(|bucket| {
            let start = bucket * low.len() / bucket_count;
            let end = (bucket + 1) * low.len() / bucket_count;
            let points = &low[start..end];
            if points.is_empty() {
                return None;
            }
            let count = points.len() as f32;
            Some(TargetSurfaceLandmarkV1 {
                landmark_id: format!("ground_contact_candidate_{bucket}"),
                position: [
                    points.iter().map(|point| point[0]).sum::<f32>() / count,
                    points.iter().map(|point| point[1]).sum::<f32>() / count,
                    bounds.min[2],
                ],
                confidence: (points.len() as f32 / 32.0).clamp(0.1, 1.0),
                provenance: "virtual_welded_authoritative_low_surface".to_owned(),
                semantic_region: "ground_contact_terminal_candidate".to_owned(),
                residual_fraction: 0.0,
                constraints: vec!["detected_ground_plane".to_owned()],
            })
        })
        .collect()
}

fn symmetry_confidence(positions: &[[f32; 3]], plane: f32) -> f32 {
    let (left, right) = positions
        .iter()
        .fold((0usize, 0usize), |mut counts, point| {
            if point[0] < plane {
                counts.0 += 1;
            } else if point[0] > plane {
                counts.1 += 1;
            }
            counts
        });
    let total = left + right;
    if total == 0 {
        return 0.0;
    }
    1.0 - left.abs_diff(right) as f32 / total as f32
}

fn robust_midpoint(mut values: Vec<f32>) -> f32 {
    values.sort_by(f32::total_cmp);
    let low = values[values.len() / 20];
    let high = values[values.len() - 1 - values.len() / 20];
    (low + high) * 0.5
}

#[derive(Debug)]
struct VirtualWeldMapV2 {
    vertex_to_group: Vec<usize>,
    representatives: Vec<usize>,
    group_count: usize,
}

fn build_virtual_weld_map_v2(positions: &[[f32; 3]], tolerance: f32) -> VirtualWeldMapV2 {
    let inverse = 1.0 / tolerance.max(1.0e-9);
    let mut groups = BTreeMap::<[i64; 3], usize>::new();
    let mut representatives = Vec::new();
    let mut vertex_to_group = Vec::with_capacity(positions.len());
    for (vertex, position) in positions.iter().enumerate() {
        let key = position.map(|component| (component * inverse).round() as i64);
        let group = *groups.entry(key).or_insert_with(|| {
            let group = representatives.len();
            representatives.push(vertex);
            group
        });
        vertex_to_group.push(group);
    }
    VirtualWeldMapV2 {
        vertex_to_group,
        group_count: representatives.len(),
        representatives,
    }
}

fn virtually_welded_components_v2(
    vertex_count: usize,
    indices: &[u32],
    vertex_to_weld_group: &[usize],
    weld_group_count: usize,
) -> Vec<Vec<usize>> {
    let mut union = DisjointSetV2::new(vertex_count);
    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        union.join(a, b);
        union.join(b, c);
        union.join(c, a);
    }
    let mut first_by_weld_group = vec![None; weld_group_count];
    for (vertex, group) in vertex_to_weld_group.iter().copied().enumerate() {
        if let Some(first) = first_by_weld_group[group] {
            union.join(first, vertex);
        } else {
            first_by_weld_group[group] = Some(vertex);
        }
    }
    let mut components = BTreeMap::<usize, Vec<usize>>::new();
    for vertex in 0..vertex_count {
        components
            .entry(union.find(vertex))
            .or_default()
            .push(vertex);
    }
    let mut output = components.into_values().collect::<Vec<_>>();
    output.sort_by_key(|vertices| vertices[0]);
    output
}

#[derive(Debug)]
struct DisjointSetV2 {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl DisjointSetV2 {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, value: usize) -> usize {
        if self.parent[value] != value {
            self.parent[value] = self.find(self.parent[value]);
        }
        self.parent[value]
    }

    fn join(&mut self, left: usize, right: usize) {
        let mut left_root = self.find(left);
        let mut right_root = self.find(right);
        if left_root == right_root {
            return;
        }
        if self.rank[left_root] < self.rank[right_root] {
            std::mem::swap(&mut left_root, &mut right_root);
        }
        self.parent[right_root] = left_root;
        if self.rank[left_root] == self.rank[right_root] {
            self.rank[left_root] = self.rank[left_root].saturating_add(1);
        }
    }
}

fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * cross.iter().map(|value| value * value).sum::<f32>().sqrt()
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2))
        .sqrt()
}

fn bounds(points: &[[f32; 3]]) -> Result<Bounds3V1, ReferenceSupermodelGenericErrorV2> {
    if points.is_empty() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-ANATOMY-EMPTY-COMPONENT",
            "surface.components",
            "surface component has no vertices",
        ));
    }
    Ok(bounds_unchecked(points))
}

fn bounds_unchecked(points: &[[f32; 3]]) -> Bounds3V1 {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    Bounds3V1 { min, max }
}

fn validate_surface(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    if positions.is_empty()
        || indices.is_empty()
        || indices.len() % 3 != 0
        || positions.iter().flatten().any(|value| !value.is_finite())
        || indices
            .iter()
            .any(|index| *index as usize >= positions.len())
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INVALID",
            "surface",
            "surface anatomy requires a finite non-empty indexed triangle surface",
        ));
    }
    Ok(())
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String, ReferenceSupermodelGenericErrorV2> {
    let bytes = serde_json::to_vec(value).map_err(|source| {
        error(
            "M2A-REFERENCE-SUPERMODEL-ANATOMY-SERIALIZE",
            "surfaceAnatomy",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReferenceSupermodelGenericErrorV2 {
    ReferenceSupermodelGenericErrorV2 {
        schema_version: 2,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod virtual_weld_tests {
    use super::*;

    #[test]
    fn virtual_weld_merges_topological_uv_seams_without_mutating_render_vertices() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let artifact = analyze_target_surface_anatomy_v1(&positions, &[0, 1, 2, 3, 4, 5])
            .expect("analyze duplicated render seam");

        assert_eq!(artifact.report.surface_vertex_count, 6);
        assert_eq!(artifact.report.virtual_weld_group_count, 3);
        assert_eq!(artifact.report.virtual_welded_vertex_count, 3);
        assert_eq!(artifact.report.component_count, 1);
        assert_eq!(artifact.joint_fit_vertex_indices.len(), 3);
    }

    #[test]
    fn a_small_detached_surface_is_explicitly_auxiliary() {
        let positions = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [0.0, 0.0, 0.1],
            [0.01, 0.0, 0.1],
            [0.0, 0.01, 0.1],
        ];
        let artifact = analyze_target_surface_anatomy_v1(&positions, &[0, 1, 2, 0, 2, 3, 4, 5, 6])
            .expect("analyze auxiliary card");

        assert_eq!(artifact.report.component_count, 2);
        assert_eq!(artifact.report.authoritative_component_count, 1);
        assert_eq!(artifact.report.auxiliary_component_count, 1);
        assert_eq!(artifact.report.components[1].role, "AUXILIARY_SURFACE");
        assert_eq!(
            artifact.report.components[1].binding_provenance,
            "project_to_semantically_compatible_authoritative_surface"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_surface_component_is_the_only_authoritative_joint_fit_body() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [10.0, 10.0, 10.0],
            [10.1, 10.0, 10.0],
            [10.0, 10.1, 10.0],
        ];
        let artifact =
            analyze_target_surface_anatomy_v1(&positions, &[0, 1, 2, 1, 3, 2, 4, 5, 6]).unwrap();
        assert_eq!(artifact.report.component_count, 2);
        assert_eq!(artifact.report.primary_component_index, 0);
        assert_eq!(artifact.joint_fit_vertex_indices, vec![0, 1, 2, 3]);
        assert_eq!(artifact.report.components[1].role, "AUXILIARY_SURFACE");
        assert_eq!(
            artifact.report.components[1].binding_provenance,
            "project_to_primary_surface"
        );
    }

    #[test]
    fn disconnected_authoritative_contact_surface_contributes_ground_landmarks() {
        let positions = vec![
            [0.0, 0.0, 1.0],
            [2.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [2.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.4, 0.0, 0.0],
            [0.0, 0.5, 0.0],
        ];
        let artifact =
            analyze_target_surface_anatomy_v1(&positions, &[0, 1, 2, 1, 3, 2, 4, 5, 6]).unwrap();

        assert_eq!(artifact.report.primary_bounds.min[2], 1.0);
        assert_eq!(artifact.report.authoritative_bounds.min[2], 0.0);
        assert_eq!(artifact.report.authoritative_component_indices, vec![0, 1]);
        assert!(
            artifact
                .report
                .ground_contact_candidates
                .iter()
                .all(|landmark| landmark.position[2] == 0.0)
        );
        assert!(
            artifact
                .report
                .ground_contact_candidates
                .iter()
                .all(|landmark| { landmark.provenance == "authoritative_component_low_surface" })
        );
    }
}
