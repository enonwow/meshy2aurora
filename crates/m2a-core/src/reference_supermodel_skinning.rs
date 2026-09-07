//! Local, structure-constrained skinning for reference-supermodel targets.
//!
//! Weights are generated around fitted carrier segments.  Diffusion is
//! bounded to immediate topology neighbours, so a connected high-density mesh
//! cannot spread every joint through the whole Creature.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    profile_a::RigWeightInfluenceV1,
    reference_supermodel_generic::ReferenceSupermodelGenericErrorV2,
    reference_supermodel_motion::ReferenceSupermodelMotionContractV2,
    reference_supermodel_surface_anatomy::TargetSurfaceAnatomyArtifactV1,
};

// A triangle may cross only one carrier edge. Larger jumps rotate its corners
// around non-adjacent pivots and create visible spikes.
const MAX_LOCAL_LABEL_TRANSITION_DISTANCE: usize = 1;
const MAX_LOCAL_INFLUENCE_TO_NEIGHBOR_LABEL_DISTANCE: usize = 1;
// The binary skin format has four influence lanes, so one vertex row may
// contain at most four consecutive carriers. A triangle crossing from that
// row into the next child-side field may consequently contain five
// consecutive carriers even though no single vertex does.
// Sibling branches remain forbidden at both levels.
const MAX_LOCAL_VERTEX_INFLUENCE_SPAN: usize = 3;
const MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN: usize = 4;
// Static skin authoring guards the maximum change across one renderer edge.
// The actual allowance is additionally scaled by that edge's length and the
// fitted carrier span; this ceiling prevents coarse geometry from receiving
// an excessively sharp transition.
const MAX_RENDER_EDGE_WEIGHT_DELTA: f32 = 0.05;
const MAX_NORMALIZED_RENDER_WEIGHT_SLOPE: f32 = 64.0;
// The initializer uses one fitted carrier span. The independent motion oracle
// applies the full two-pivot-span deformation bound; label allocation must not
// multiply this value globally because sequential thickening can otherwise
// consume distal labels (paws, tail end and short leg segments).
const GEODESIC_DERIVATIVE_SAFETY_FACTOR: f32 = 1.0;
const LOCAL_BOUNDARY_SMOOTHING_ITERATIONS: usize = 128;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelSkinningOptionsV1 {
    #[serde(default)]
    pub allow_excessive_branch_boundary_repair: bool,
    /// Return quality failures as editable, blocked drafts; malformed input still fails.
    #[serde(default)]
    pub retain_editable_draft_on_quality_failure: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelSkinningReportV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub content_sha256: String,
    pub surface_vertex_count: usize,
    pub smoothing_iteration_count: usize,
    pub primary_label_iteration_count: usize,
    pub primary_label_reassignment_count: usize,
    pub branch_boundary_repair_iteration_count: usize,
    pub branch_boundary_repair_vertex_count: usize,
    pub branch_boundary_repair_maximum_observed_vertex_count: usize,
    pub branch_boundary_repair_limit_vertex_count: usize,
    pub branch_boundary_repair_limit_exceeded: bool,
    pub branch_boundary_repair_limit_bypass_enabled: bool,
    pub initial_cross_branch_triangle_count: usize,
    pub average_positive_influence_count: f32,
    pub maximum_positive_influence_count: usize,
    pub average_weight_entropy: f32,
    pub maximum_weight_entropy: f32,
    pub cross_side_leakage_vertex_count: usize,
    pub cross_branch_leakage_vertex_count: usize,
    pub cross_branch_triangle_count: usize,
    pub locally_promoted_joint_count: usize,
    pub component_projection_count: usize,
    pub component_projection_vertex_count: usize,
    pub auxiliary_component_count: usize,
    pub auxiliary_projection_count: usize,
    pub auxiliary_projection_vertex_count: usize,
    pub auxiliary_projection_coverage: bool,
    pub duplicate_position_group_count: usize,
    pub local_boundary_shared_carrier_group_count: usize,
    pub local_boundary_rigid_group_count: usize,
    pub edge_cliff_relaxation_iteration_count: usize,
    pub edge_cliff_relaxed_group_count: usize,
    pub primary_label_projection_group_count: usize,
    pub primary_label_projection_vertex_count: usize,
    pub chain_label_thickening_vertex_count: usize,
    pub chain_label_topology_repair_vertex_count: usize,
    pub geodesic_boundary_seed_group_count: usize,
    pub weight_gradient_relaxation_update_count: usize,
    pub weight_gradient_violation_edge_count: usize,
    pub maximum_weight_gradient_limit_ratio: f32,
    pub worst_weight_gradient_edges: Vec<ReferenceSupermodelWeightGradientEdgeV1>,
    pub skin_region_feasibility: Vec<ReferenceSupermodelSkinRegionFeasibilityV1>,
    pub arbitrary_component_reservation_applied: bool,
    pub warnings: Vec<String>,
    pub validation_violations: Vec<String>,
    pub joint_clusters: Vec<ReferenceSupermodelJointSkinClusterV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelSkinRegionFeasibilityV1 {
    pub part_number: u32,
    pub joint_name: String,
    pub parent_boundary_group_count: usize,
    pub available_geodesic_depth_fraction: f32,
    pub required_blend_width_fraction: f32,
    pub available_to_required_ratio: f32,
    pub minimum_child_boundary_depth_fraction: Option<f32>,
    pub dominant_core_feasible: bool,
    pub full_core_feasible: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelWeightGradientEdgeV1 {
    pub left_vertex: usize,
    pub right_vertex: usize,
    pub left_position: [f32; 3],
    pub right_position: [f32; 3],
    pub edge_length_fraction: f32,
    pub left_label_part_number: u32,
    pub left_label_name: String,
    pub right_label_part_number: u32,
    pub right_label_name: String,
    pub left_weights: Vec<RigWeightInfluenceV1>,
    pub right_weights: Vec<RigWeightInfluenceV1>,
    pub maximum_weight_delta: f32,
    pub permitted_weight_delta: f32,
    pub limit_ratio: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelJointSkinClusterV1 {
    pub part_number: u32,
    pub joint_name: String,
    pub dominant_vertex_count: usize,
    pub positive_vertex_count: usize,
    pub maximum_surface_distance_fraction: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSupermodelSkinningArtifactV1 {
    pub weights: Vec<Vec<RigWeightInfluenceV1>>,
    pub report: ReferenceSupermodelSkinningReportV1,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct AuxiliaryProjectionStatsV3 {
    required_component_count: usize,
    projected_component_count: usize,
    projected_vertex_count: usize,
}

fn project_auxiliary_surface_components_v3(
    positions: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    anatomy: &TargetSurfaceAnatomyArtifactV1,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<AuxiliaryProjectionStatsV3, ReferenceSupermodelGenericErrorV2> {
    if anatomy.vertex_component_indices.len() != positions.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-ANATOMY-MAPPING-INVALID",
            "skinning.surfaceAnatomy",
            "semantic component mapping must contain one entry per render vertex",
        ));
    }
    let auxiliary_components = anatomy
        .report
        .components
        .iter()
        .filter(|component| component.role == "AUXILIARY_SURFACE")
        .map(|component| component.component_index)
        .collect::<BTreeSet<_>>();
    if auxiliary_components.is_empty() {
        return Ok(AuxiliaryProjectionStatsV3::default());
    }
    let authoritative = anatomy
        .joint_fit_vertex_indices
        .iter()
        .copied()
        .filter(|vertex| *vertex < positions.len() && !weights[*vertex].is_empty())
        .collect::<Vec<_>>();
    if authoritative.is_empty() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-AUTHORITATIVE-SURFACE-MISSING",
            "skinning.surfaceAnatomy",
            "auxiliary surfaces cannot be projected without an authoritative body proxy",
        ));
    }
    let mut vertices_by_component = BTreeMap::<usize, Vec<usize>>::new();
    for (vertex, component) in anatomy.vertex_component_indices.iter().copied().enumerate() {
        if auxiliary_components.contains(&component) {
            vertices_by_component
                .entry(component)
                .or_default()
                .push(vertex);
        }
    }
    let mut stats = AuxiliaryProjectionStatsV3 {
        required_component_count: auxiliary_components.len(),
        ..AuxiliaryProjectionStatsV3::default()
    };
    for component in auxiliary_components {
        let vertices = vertices_by_component.remove(&component).unwrap_or_default();
        if vertices.is_empty() {
            continue;
        }
        let center = std::array::from_fn(|axis| {
            vertices
                .iter()
                .map(|vertex| positions[*vertex][axis])
                .sum::<f32>()
                / vertices.len() as f32
        });
        let mut anchors = authoritative
            .iter()
            .copied()
            .map(|vertex| (squared_distance(center, positions[vertex]), vertex))
            .collect::<Vec<_>>();
        anchors.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1)));
        anchors.truncate(32);
        if anchors.is_empty() {
            continue;
        }
        for vertex in &vertices {
            let anchor = anchors
                .iter()
                .min_by(|left, right| {
                    squared_distance(positions[*vertex], positions[left.1])
                        .total_cmp(&squared_distance(positions[*vertex], positions[right.1]))
                        .then(left.1.cmp(&right.1))
                })
                .expect("non-empty semantic projection anchor set")
                .1;
            let row = weights[anchor].clone();
            if row.iter().any(|influence| {
                influence.value <= 0.0 || influence.bone_node_id as usize >= contract.nodes.len()
            }) {
                return Err(error(
                    "M2A-REFERENCE-SUPERMODEL-SKIN-AUXILIARY-PROJECTION-INVALID",
                    format!("skinning.weights[{vertex}]"),
                    "authoritative projection anchor has an invalid carrier row",
                ));
            }
            weights[*vertex] = row;
        }
        stats.projected_component_count += 1;
        stats.projected_vertex_count += vertices.len();
    }
    if stats.projected_component_count != stats.required_component_count {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-AUXILIARY-PROJECTION-INCOMPLETE",
            "skinning.surfaceAnatomy.components",
            format!(
                "projected {} of {} auxiliary surface components",
                stats.projected_component_count, stats.required_component_count
            ),
        ));
    }
    Ok(stats)
}

fn trace_skin_labels(stage: &str, labels: &[u32], contract: &ReferenceSupermodelMotionContractV2) {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("M2A_TRACE_SKIN_LABELS").is_some() {
        let mut counts = BTreeMap::<String, usize>::new();
        for part in labels {
            *counts
                .entry(contract.nodes[*part as usize].name.clone())
                .or_default() += 1;
        }
        eprintln!(
            "M2A_SKIN_LABELS:{stage}:{}",
            serde_json::to_string(&counts).unwrap()
        );
    }
}

pub fn derive_local_reference_skinning_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed_bones: &[u32],
    fitted_worlds: &[[f32; 3]],
    anatomy: &TargetSurfaceAnatomyArtifactV1,
) -> Result<ReferenceSupermodelSkinningArtifactV1, ReferenceSupermodelGenericErrorV2> {
    derive_local_reference_skinning_with_options_v1(
        contract,
        positions,
        indices,
        allowed_bones,
        fitted_worlds,
        anatomy,
        ReferenceSupermodelSkinningOptionsV1::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn derive_local_reference_skinning_with_options_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed_bones: &[u32],
    fitted_worlds: &[[f32; 3]],
    anatomy: &TargetSurfaceAnatomyArtifactV1,
    options: ReferenceSupermodelSkinningOptionsV1,
) -> Result<ReferenceSupermodelSkinningArtifactV1, ReferenceSupermodelGenericErrorV2> {
    derive_local_reference_skinning_with_seed_labels_v2(
        contract,
        positions,
        indices,
        allowed_bones,
        fitted_worlds,
        anatomy,
        options,
        None,
    )
}

pub fn derive_local_reference_skinning_with_seed_labels_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed_bones: &[u32],
    fitted_worlds: &[[f32; 3]],
    anatomy: &TargetSurfaceAnatomyArtifactV1,
    options: ReferenceSupermodelSkinningOptionsV1,
    seed_labels: Option<&[u32]>,
) -> Result<ReferenceSupermodelSkinningArtifactV1, ReferenceSupermodelGenericErrorV2> {
    eprintln!("M2A_SKIN_TIMING:derive:start");
    validate_inputs(contract, positions, indices, allowed_bones, fitted_worlds)?;
    if seed_labels.is_some_and(|labels| {
        labels.len() != positions.len() || labels.iter().any(|part| !allowed_bones.contains(part))
    }) {
        return Err(error(
            "M2A-SURFACE-GUIDE-LABELS-INVALID",
            "skinning.seedLabels",
            "Expected one allowed carrier label per source vertex",
        ));
    }
    let diagonal = model_diagonal(positions).max(1.0e-5);
    let seed_weights = positions
        .iter()
        .enumerate()
        .map(|(vertex, position)| {
            let primary = seed_labels.map_or_else(
                || {
                    nearest_segment_bone(
                        *position,
                        allowed_bones,
                        fitted_worlds,
                        contract,
                        diagonal,
                    )
                },
                |labels| labels[vertex],
            );
            local_segment_weights(
                *position,
                primary,
                allowed_bones,
                fitted_worlds,
                contract,
                diagonal,
            )
        })
        .collect::<Vec<_>>();
    let seed_primary = seed_weights
        .iter()
        .map(|row| row[0].bone_node_id)
        .collect::<Vec<_>>();
    trace_skin_labels("nearest", &seed_primary, contract);
    let adjacency = surface_adjacency(positions, indices)?;
    const PRIMARY_LABEL_ITERATIONS: usize = 24;
    let (primary, primary_label_reassignment_count) = if seed_labels.is_some() {
        (seed_primary, 0)
    } else {
        regularize_primary_labels(
            contract,
            positions,
            &adjacency,
            allowed_bones,
            fitted_worlds,
            diagonal,
            &seed_weights,
            seed_primary,
            PRIMARY_LABEL_ITERATIONS,
        )
    };
    trace_skin_labels("regularized", &primary, contract);
    eprintln!("M2A_SKIN_TIMING:derive:labels-ready");
    let mut weights = positions
        .iter()
        .zip(&primary)
        .map(|(position, primary)| {
            local_segment_weights(
                *position,
                *primary,
                allowed_bones,
                fitted_worlds,
                contract,
                diagonal,
            )
        })
        .collect::<Vec<_>>();
    const SMOOTHING_ITERATIONS: usize = 4;
    for _ in 0..SMOOTHING_ITERATIONS {
        let previous = weights.clone();
        for vertex in 0..weights.len() {
            let compatible_neighbors = adjacency[vertex]
                .iter()
                .copied()
                .filter(|neighbor| {
                    carrier_topology_distance(primary[vertex], primary[*neighbor], contract) <= 1
                })
                .collect::<Vec<_>>();
            if compatible_neighbors.is_empty() {
                continue;
            }
            let neighbor_share = 0.15 / compatible_neighbors.len() as f32;
            let mut accumulated = BTreeMap::<u32, f32>::new();
            for influence in &previous[vertex] {
                *accumulated.entry(influence.bone_node_id).or_default() += influence.value * 0.85;
            }
            for neighbor in compatible_neighbors {
                for influence in &previous[neighbor] {
                    if carrier_topology_distance(primary[vertex], influence.bone_node_id, contract)
                        <= 1
                    {
                        *accumulated.entry(influence.bone_node_id).or_default() +=
                            influence.value * neighbor_share;
                    }
                }
            }
            weights[vertex] = normalize_top_weights(accumulated, 4, 0.04)?;
        }
    }
    eprintln!("M2A_SKIN_TIMING:derive:initial-smoothing-ready");

    // Do not derive seam identity from provisional weight support.  Those
    // rows are discarded after branch repair, and building a full
    // weight-topology partition here was both semantically circular and the
    // dominant cost on dense generated meshes.  The label repair below owns
    // duplicate-label synchronization; exact seam rows are welded after its
    // authoritative labels are available.
    // Provisional weight smoothing must not relabel the anatomical guide.
    // The finalizer replaces these rows; primary labels remain authoritative.
    let boundary_seed = primary.clone();
    let mut initialization_failures = Vec::new();
    let boundary_repair_result = repair_triangle_branch_boundaries(
        positions,
        indices,
        allowed_bones,
        fitted_worlds,
        contract,
        diagonal,
        boundary_seed.clone(),
        options,
    );
    let boundary_repair = match boundary_repair_result {
        Ok(result) => result,
        Err(failure)
            if options.retain_editable_draft_on_quality_failure
                && failure.code.starts_with("M2A-REFERENCE-SUPERMODEL-SKIN-") =>
        {
            initialization_failures.push(format!("{}: {}", failure.code, failure.message));
            BranchBoundaryRepairV1 {
                initial_violation_count: cross_branch_triangle_count_from_labels(
                    positions,
                    indices,
                    &boundary_seed,
                    contract,
                )?,
                labels: boundary_seed,
                iteration_count: 0,
                reassigned_vertex_count: 0,
                maximum_reassigned_vertex_count_observed: 0,
                limit_vertex_count: (positions.len() / 20).max(16),
                limit_exceeded: true,
            }
        }
        Err(failure) => return Err(failure),
    };
    trace_skin_labels("branch-repair", &boundary_repair.labels, contract);
    eprintln!("M2A_SKIN_TIMING:derive:branch-repair-ready");
    // The finalizer owns the only production weight sequence.  Earlier code
    // smoothed and projected provisional rows here, then replaced every row
    // again inside finalization; those expensive passes could not affect the
    // emitted rig.  Seed, auxiliary transfer, component stabilization, seam
    // welding and gradient projection now execute once in output order.
    let final_boundary_result = finalize_projected_boundary_weights_v2(
        positions,
        indices,
        &boundary_repair.labels,
        contract,
        fitted_worlds,
        Some(anatomy),
        &mut weights,
    );
    let final_boundary_relaxation = match final_boundary_result {
        Ok(result) => result,
        Err(failure)
            if options.retain_editable_draft_on_quality_failure
                && failure.code.starts_with("M2A-REFERENCE-SUPERMODEL-SKIN-") =>
        {
            initialization_failures.push(format!("{}: {}", failure.code, failure.message));
            FinalBoundaryRelaxationStatsV2 {
                gradient: audit_geometry_scaled_weight_gradients_v2(
                    positions,
                    indices,
                    &boundary_repair.labels,
                    contract,
                    fitted_worlds,
                    &weights,
                )
                .unwrap_or_default(),
                ..Default::default()
            }
        }
        Err(failure) => return Err(failure),
    };
    let auxiliary_projection = final_boundary_relaxation.auxiliary_projection;
    let component_projection = final_boundary_relaxation.component_projection;
    let component_projection_count = component_projection.component_count;
    let component_projection_vertex_count = component_projection.vertex_count;
    let mut local_boundary_constraints = LocalBoundaryConstraintStatsV1 {
        shared_carrier_group_count: final_boundary_relaxation.shared_carrier_group_count,
        rigid_group_count: final_boundary_relaxation.rigid_group_count,
        ..LocalBoundaryConstraintStatsV1::default()
    };
    let worst_weight_gradient_edges = final_boundary_relaxation.gradient.worst_edges.clone();
    let skin_region_feasibility = final_boundary_relaxation.skin_region_feasibility.clone();
    local_boundary_constraints.edge_cliff_relaxation_iteration_count =
        final_boundary_relaxation.gradient.processed_edge_count;
    local_boundary_constraints.edge_cliff_relaxed_group_count =
        final_boundary_relaxation.gradient.relaxed_group_count;
    local_boundary_constraints.primary_label_projection_group_count =
        final_boundary_relaxation.projection.projected_group_count;
    local_boundary_constraints.primary_label_projection_vertex_count =
        final_boundary_relaxation.projection.projected_vertex_count;
    local_boundary_constraints.chain_label_thickening_vertex_count =
        final_boundary_relaxation.chain_label_thickening_vertex_count;
    local_boundary_constraints.chain_label_topology_repair_vertex_count =
        final_boundary_relaxation.chain_label_topology_repair_vertex_count;
    local_boundary_constraints.geodesic_boundary_seed_group_count =
        final_boundary_relaxation.geodesic_boundary_seed_group_count;
    local_boundary_constraints.weight_gradient_relaxation_update_count =
        final_boundary_relaxation.gradient.update_count;
    local_boundary_constraints.weight_gradient_violation_edge_count =
        final_boundary_relaxation.gradient.violation_edge_count;
    local_boundary_constraints.maximum_weight_gradient_limit_ratio =
        final_boundary_relaxation.gradient.maximum_limit_ratio;
    let locally_promoted_joint_count = 0;

    let mut cross_branch_leakage_vertex_count = 0;
    let mut cross_side_leakage_vertex_count = 0;
    let mut positive_influence_total = 0usize;
    let mut maximum_positive_influence_count = 0usize;
    let mut entropy_total = 0.0_f64;
    let mut maximum_weight_entropy = 0.0_f32;
    let mut joint_clusters = allowed_bones
        .iter()
        .copied()
        .map(|part_number| {
            (
                part_number,
                ReferenceSupermodelJointSkinClusterV1 {
                    part_number,
                    joint_name: contract.nodes[part_number as usize].name.clone(),
                    dominant_vertex_count: 0,
                    positive_vertex_count: 0,
                    maximum_surface_distance_fraction: 0.0,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (vertex, row) in weights.iter().enumerate() {
        let dominant = row[0].bone_node_id;
        let positive = row.iter().filter(|influence| influence.value > 0.0).count();
        positive_influence_total += positive;
        maximum_positive_influence_count = maximum_positive_influence_count.max(positive);
        let entropy = row
            .iter()
            .filter(|influence| influence.value > 0.0)
            .map(|influence| -influence.value * influence.value.ln())
            .sum::<f32>();
        entropy_total += f64::from(entropy);
        maximum_weight_entropy = maximum_weight_entropy.max(entropy);
        if let Some(cluster) = joint_clusters.get_mut(&dominant) {
            cluster.dominant_vertex_count += 1;
        }
        let mut row_parts = BTreeSet::new();
        for influence in row.iter().filter(|influence| influence.value > 0.0) {
            if row_parts.insert(influence.bone_node_id) {
                if let Some(cluster) = joint_clusters.get_mut(&influence.bone_node_id) {
                    cluster.positive_vertex_count += 1;
                    cluster.maximum_surface_distance_fraction =
                        cluster.maximum_surface_distance_fraction.max(
                            segment_distance(
                                positions[vertex],
                                influence.bone_node_id,
                                fitted_worlds,
                                contract,
                            ) / diagonal,
                        );
                }
            }
        }
        if row.iter().any(|influence| {
            influence.value >= 0.1
                && is_cross_side_influence(
                    positions[vertex],
                    influence.bone_node_id,
                    fitted_worlds,
                    diagonal,
                    contract,
                )
        }) {
            cross_side_leakage_vertex_count += 1;
        }
        if !carrier_support_is_single_lineage(&row_parts, contract, MAX_LOCAL_VERTEX_INFLUENCE_SPAN)
        {
            cross_branch_leakage_vertex_count += 1;
        }
    }
    if cross_branch_leakage_vertex_count > 0 && !options.retain_editable_draft_on_quality_failure {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-CROSS-BRANCH-LEAKAGE",
            "skinning.weights",
            format!(
                "{cross_branch_leakage_vertex_count} vertices contain influences from unrelated carrier branches"
            ),
        ));
    }
    if cross_side_leakage_vertex_count > 0 && !options.retain_editable_draft_on_quality_failure {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-CROSS-SIDE-LEAKAGE",
            "skinning.weights",
            format!(
                "{cross_side_leakage_vertex_count} vertices contain material influence from the opposite fitted symmetry side"
            ),
        ));
    }
    let cross_branch_triangle_count =
        cross_branch_triangle_count(positions, indices, &weights, contract)?;
    if cross_branch_triangle_count > 0 && !options.retain_editable_draft_on_quality_failure {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-TRIANGLE-BRANCH-DISCONTINUITY",
            "skinning.triangles",
            format!("{cross_branch_triangle_count} triangles bridge unrelated carrier branches"),
        ));
    }
    let mut validation_violations =
        if local_boundary_constraints.weight_gradient_violation_edge_count == 0 {
            Vec::new()
        } else {
            vec![format!(
                "WEIGHT_GRADIENT_VIOLATIONS:{}:MAX_RATIO:{:.6}",
                local_boundary_constraints.weight_gradient_violation_edge_count,
                local_boundary_constraints.maximum_weight_gradient_limit_ratio,
            )]
        };
    validation_violations.extend(
        initialization_failures
            .iter()
            .map(|s| format!("INITIALIZATION_FAILED:{s}")),
    );
    if cross_branch_leakage_vertex_count > 0 {
        validation_violations.push(format!(
            "CROSS_BRANCH_LEAKAGE:{cross_branch_leakage_vertex_count}"
        ));
    }
    if cross_side_leakage_vertex_count > 0 {
        validation_violations.push(format!(
            "CROSS_SIDE_LEAKAGE:{cross_side_leakage_vertex_count}"
        ));
    }
    if cross_branch_triangle_count > 0 {
        validation_violations.push(format!(
            "CROSS_BRANCH_TRIANGLES:{cross_branch_triangle_count}"
        ));
    }
    let mut warnings =
        if boundary_repair.limit_exceeded && options.allow_excessive_branch_boundary_repair {
            vec![format!(
                "EXPERIMENTAL_BRANCH_BOUNDARY_REPAIR_LIMIT_BYPASSED:{}:{}",
                boundary_repair.maximum_reassigned_vertex_count_observed,
                boundary_repair.limit_vertex_count,
            )]
        } else {
            Vec::new()
        };
    warnings.extend(initialization_failures);
    let mut report = ReferenceSupermodelSkinningReportV1 {
        schema_version: 3,
        algorithm: "REFERENCE_SURFACE_REGION_PRESERVING_V3".to_owned(),
        status: if validation_violations.is_empty() {
            "READY"
        } else {
            "BLOCKED"
        }
        .to_owned(),
        content_sha256: String::new(),
        surface_vertex_count: positions.len(),
        smoothing_iteration_count: SMOOTHING_ITERATIONS,
        primary_label_iteration_count: PRIMARY_LABEL_ITERATIONS,
        primary_label_reassignment_count,
        branch_boundary_repair_iteration_count: boundary_repair.iteration_count,
        branch_boundary_repair_vertex_count: boundary_repair.reassigned_vertex_count,
        branch_boundary_repair_maximum_observed_vertex_count: boundary_repair
            .maximum_reassigned_vertex_count_observed,
        branch_boundary_repair_limit_vertex_count: boundary_repair.limit_vertex_count,
        branch_boundary_repair_limit_exceeded: boundary_repair.limit_exceeded,
        branch_boundary_repair_limit_bypass_enabled: options.allow_excessive_branch_boundary_repair,
        initial_cross_branch_triangle_count: boundary_repair.initial_violation_count,
        average_positive_influence_count: positive_influence_total as f32 / positions.len() as f32,
        maximum_positive_influence_count,
        average_weight_entropy: (entropy_total / positions.len() as f64) as f32,
        maximum_weight_entropy,
        cross_side_leakage_vertex_count,
        cross_branch_leakage_vertex_count,
        cross_branch_triangle_count,
        locally_promoted_joint_count,
        component_projection_count,
        component_projection_vertex_count,
        auxiliary_component_count: auxiliary_projection.required_component_count,
        auxiliary_projection_count: auxiliary_projection.projected_component_count,
        auxiliary_projection_vertex_count: auxiliary_projection.projected_vertex_count,
        auxiliary_projection_coverage: auxiliary_projection.required_component_count
            == auxiliary_projection.projected_component_count,
        duplicate_position_group_count: duplicate_position_group_count(positions),
        local_boundary_shared_carrier_group_count: local_boundary_constraints
            .shared_carrier_group_count,
        local_boundary_rigid_group_count: local_boundary_constraints.rigid_group_count,
        edge_cliff_relaxation_iteration_count: local_boundary_constraints
            .edge_cliff_relaxation_iteration_count,
        edge_cliff_relaxed_group_count: local_boundary_constraints.edge_cliff_relaxed_group_count,
        primary_label_projection_group_count: local_boundary_constraints
            .primary_label_projection_group_count,
        primary_label_projection_vertex_count: local_boundary_constraints
            .primary_label_projection_vertex_count,
        chain_label_thickening_vertex_count: local_boundary_constraints
            .chain_label_thickening_vertex_count,
        chain_label_topology_repair_vertex_count: local_boundary_constraints
            .chain_label_topology_repair_vertex_count,
        geodesic_boundary_seed_group_count: local_boundary_constraints
            .geodesic_boundary_seed_group_count,
        weight_gradient_relaxation_update_count: local_boundary_constraints
            .weight_gradient_relaxation_update_count,
        weight_gradient_violation_edge_count: local_boundary_constraints
            .weight_gradient_violation_edge_count,
        maximum_weight_gradient_limit_ratio: local_boundary_constraints
            .maximum_weight_gradient_limit_ratio,
        worst_weight_gradient_edges,
        skin_region_feasibility,
        arbitrary_component_reservation_applied: false,
        warnings,
        validation_violations,
        joint_clusters: joint_clusters.into_values().collect(),
    };
    report.content_sha256 = sha256_json(&report)?;
    Ok(ReferenceSupermodelSkinningArtifactV1 { report, weights })
}

#[allow(clippy::too_many_arguments)]
pub fn validate_authored_reference_skinning_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed_bones: &[u32],
    fitted_worlds: &[[f32; 3]],
    weights: &[Vec<RigWeightInfluenceV1>],
    base_report: &ReferenceSupermodelSkinningReportV1,
) -> Result<ReferenceSupermodelSkinningReportV1, ReferenceSupermodelGenericErrorV2> {
    validate_inputs(contract, positions, indices, allowed_bones, fitted_worlds)?;
    if weights.len() != positions.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORED-SKINNING-ROW-COUNT",
            "skinning.weights",
            "authored skinning requires one weight row per render vertex",
        ));
    }
    let allowed = allowed_bones.iter().copied().collect::<BTreeSet<_>>();
    let diagonal = model_diagonal(positions).max(1.0e-6);
    // Revalidate the current authored rows; initialization history is not a current verdict.
    let mut validation_violations = Vec::new();
    let mut invalid_row_count = 0usize;
    let mut cross_side_leakage_vertex_count = 0usize;
    let mut cross_branch_leakage_vertex_count = 0usize;
    let mut positive_influence_total = 0usize;
    let mut maximum_positive_influence_count = 0usize;
    let mut entropy_total = 0.0_f64;
    let mut maximum_weight_entropy = 0.0_f32;
    let mut joint_clusters = allowed_bones
        .iter()
        .copied()
        .map(|part_number| {
            (
                part_number,
                ReferenceSupermodelJointSkinClusterV1 {
                    part_number,
                    joint_name: contract.nodes[part_number as usize].name.clone(),
                    dominant_vertex_count: 0,
                    positive_vertex_count: 0,
                    maximum_surface_distance_fraction: 0.0,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (vertex, row) in weights.iter().enumerate() {
        let sum = row.iter().map(|influence| influence.value).sum::<f32>();
        let unique = row
            .iter()
            .map(|influence| influence.bone_node_id)
            .collect::<BTreeSet<_>>();
        let valid = (1..=4).contains(&row.len())
            && unique.len() == row.len()
            && row.iter().all(|influence| {
                influence.value.is_finite()
                    && influence.value > 0.0
                    && allowed.contains(&influence.bone_node_id)
            })
            && sum.is_finite()
            && (sum - 1.0).abs() <= 1.0e-4;
        if !valid {
            invalid_row_count += 1;
            continue;
        }
        let dominant = row[0].bone_node_id;
        if let Some(cluster) = joint_clusters.get_mut(&dominant) {
            cluster.dominant_vertex_count += 1;
        }
        let positive = row.len();
        positive_influence_total += positive;
        maximum_positive_influence_count = maximum_positive_influence_count.max(positive);
        let entropy = row
            .iter()
            .map(|influence| -influence.value * influence.value.ln())
            .sum::<f32>();
        entropy_total += f64::from(entropy);
        maximum_weight_entropy = maximum_weight_entropy.max(entropy);
        for influence in row {
            if let Some(cluster) = joint_clusters.get_mut(&influence.bone_node_id) {
                cluster.positive_vertex_count += 1;
                cluster.maximum_surface_distance_fraction =
                    cluster.maximum_surface_distance_fraction.max(
                        segment_distance(
                            positions[vertex],
                            influence.bone_node_id,
                            fitted_worlds,
                            contract,
                        ) / diagonal,
                    );
            }
        }
        if row.iter().any(|influence| {
            influence.value >= 0.1
                && is_cross_side_influence(
                    positions[vertex],
                    influence.bone_node_id,
                    fitted_worlds,
                    diagonal,
                    contract,
                )
        }) {
            cross_side_leakage_vertex_count += 1;
        }
        if !carrier_support_is_local_neighborhood_v1(
            &unique,
            contract,
            MAX_LOCAL_VERTEX_INFLUENCE_SPAN,
        ) {
            cross_branch_leakage_vertex_count += 1;
        }
    }
    if invalid_row_count > 0 {
        validation_violations.push(format!("INVALID_WEIGHT_ROWS:{invalid_row_count}"));
    }
    if cross_side_leakage_vertex_count > 0 {
        validation_violations.push(format!(
            "CROSS_SIDE_LEAKAGE:{cross_side_leakage_vertex_count}"
        ));
    }
    if cross_branch_leakage_vertex_count > 0 {
        validation_violations.push(format!(
            "CROSS_BRANCH_LEAKAGE:{cross_branch_leakage_vertex_count}"
        ));
    }
    let cross_branch_triangle_count =
        authored_nonlocal_triangle_count_v1(positions, indices, weights, contract)?;
    if cross_branch_triangle_count > 0 {
        validation_violations.push(format!(
            "CROSS_BRANCH_TRIANGLES:{cross_branch_triangle_count}"
        ));
    }
    let duplicate_groups =
        duplicate_position_weight_topology_groups_v1(positions, indices, weights, contract)?;
    let duplicate_row_mismatch_count = duplicate_groups
        .iter()
        .filter(|group| {
            group
                .windows(2)
                .any(|pair| weights[pair[0]] != weights[pair[1]])
        })
        .count();
    if duplicate_row_mismatch_count > 0 {
        validation_violations.push(format!(
            "DUPLICATE_SEAM_WEIGHT_MISMATCH:{duplicate_row_mismatch_count}"
        ));
    }
    let missing_clusters = joint_clusters
        .values()
        .filter(|cluster| cluster.dominant_vertex_count == 0 || cluster.positive_vertex_count == 0)
        .map(|cluster| cluster.part_number)
        .collect::<Vec<_>>();
    if !missing_clusters.is_empty() {
        validation_violations.push(format!("MISSING_SEMANTIC_CLUSTERS:{missing_clusters:?}"));
    }
    let labels = weights
        .iter()
        .map(|row| row.first().map_or(0, |w| w.bone_node_id))
        .collect::<Vec<_>>();
    let gradient = if invalid_row_count == 0 {
        match audit_geometry_scaled_weight_gradients_v2(
            positions,
            indices,
            &labels,
            contract,
            fitted_worlds,
            weights,
        ) {
            Ok(g) => {
                if g.violation_edge_count > 0 {
                    validation_violations.push(format!(
                        "WEIGHT_GRADIENT_VIOLATIONS:{}:MAX_RATIO:{:.6}",
                        g.violation_edge_count, g.maximum_limit_ratio
                    ));
                }
                Some(g)
            }
            Err(e) => {
                validation_violations.push(format!("WEIGHT_GRADIENT_AUDIT_FAILED:{}", e.code));
                None
            }
        }
    } else {
        None
    };
    let mut report = base_report.clone();
    report.weight_gradient_violation_edge_count =
        gradient.as_ref().map_or(0, |g| g.violation_edge_count);
    report.maximum_weight_gradient_limit_ratio =
        gradient.as_ref().map_or(0., |g| g.maximum_limit_ratio);
    report.worst_weight_gradient_edges = gradient.map_or_else(Vec::new, |g| g.worst_edges);
    report.schema_version = 3;
    report.algorithm = "AUTHORED_SEMANTIC_SKINNING_VALIDATION_V2".to_owned();
    report.status = if validation_violations.is_empty() {
        "READY"
    } else {
        "BLOCKED"
    }
    .to_owned();
    report.content_sha256.clear();
    report.average_positive_influence_count =
        positive_influence_total as f32 / positions.len().max(1) as f32;
    report.maximum_positive_influence_count = maximum_positive_influence_count;
    report.average_weight_entropy = (entropy_total / positions.len().max(1) as f64) as f32;
    report.maximum_weight_entropy = maximum_weight_entropy;
    report.cross_side_leakage_vertex_count = cross_side_leakage_vertex_count;
    report.cross_branch_leakage_vertex_count = cross_branch_leakage_vertex_count;
    report.cross_branch_triangle_count = cross_branch_triangle_count;
    report.validation_violations = validation_violations;
    report.joint_clusters = joint_clusters.into_values().collect();
    report.content_sha256 = sha256_json(&report)?;
    Ok(report)
}

fn promote_existing_local_influence_clusters_v1(
    positions: &[[f32; 3]],
    allowed_bones: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    let groups = duplicate_position_groups(positions);
    let diagonal = model_diagonal(positions).max(1.0e-5);
    let initially_dominant = weights
        .iter()
        .filter_map(|row| row.first().map(|influence| influence.bone_node_id))
        .collect::<BTreeSet<_>>();
    let mut promoted_joint_count = 0usize;
    for bone in allowed_bones.iter().copied() {
        if initially_dominant.contains(&bone) {
            continue;
        }
        let positive_vertex_count = weights
            .iter()
            .filter(|row| {
                row.iter()
                    .any(|influence| influence.bone_node_id == bone && influence.value > 0.0)
            })
            .count();
        if positive_vertex_count == 0 {
            continue;
        }
        let mut candidates = groups
            .values()
            .filter_map(|group| {
                let rows = group
                    .iter()
                    .map(|vertex| &weights[*vertex])
                    .collect::<Vec<_>>();
                let bone_weight = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .find(|influence| influence.bone_node_id == bone)
                            .map_or(0.0, |influence| influence.value)
                    })
                    .sum::<f32>()
                    / rows.len() as f32;
                if bone_weight <= 0.0
                    || rows.iter().flat_map(|row| row.iter()).any(|influence| {
                        influence.value > 0.0
                            && !carrier_transition_is_local(
                                bone,
                                influence.bone_node_id,
                                contract,
                                1,
                            )
                    })
                {
                    return None;
                }
                let center = std::array::from_fn(|axis| {
                    group
                        .iter()
                        .map(|vertex| positions[*vertex][axis])
                        .sum::<f32>()
                        / group.len() as f32
                });
                if is_cross_side_influence(center, bone, fitted_worlds, diagonal, contract) {
                    return None;
                }
                Some((
                    bone_weight,
                    segment_distance(center, bone, fitted_worlds, contract),
                    group,
                ))
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .0
                .total_cmp(&left.0)
                .then(left.1.total_cmp(&right.1))
                .then(left.2[0].cmp(&right.2[0]))
        });
        let target_vertex_count = (positive_vertex_count / 16).clamp(1, 64);
        let mut promoted_vertex_count = 0usize;
        for (_, _, group) in candidates {
            let mut accumulated = BTreeMap::<u32, f32>::new();
            for &vertex in group {
                for influence in &weights[vertex] {
                    *accumulated.entry(influence.bone_node_id).or_default() += influence.value;
                }
            }
            for value in accumulated.values_mut() {
                *value /= group.len() as f32;
            }
            let competing_maximum = accumulated
                .iter()
                .filter(|(candidate, _)| **candidate != bone)
                .map(|(_, value)| *value)
                .max_by(f32::total_cmp)
                .unwrap_or(0.0);
            accumulated.insert(bone, next_f32_up(competing_maximum));
            let promoted = normalize_top_weights(accumulated, 4, 0.04)?;
            if promoted.first().map(|influence| influence.bone_node_id) != Some(bone) {
                continue;
            }
            for &vertex in group {
                weights[vertex] = promoted.clone();
            }
            promoted_vertex_count += group.len();
            if promoted_vertex_count >= target_vertex_count {
                break;
            }
        }
        promoted_joint_count += usize::from(promoted_vertex_count > 0);
    }
    Ok(promoted_joint_count)
}

#[derive(Debug)]
struct BranchBoundaryRepairV1 {
    labels: Vec<u32>,
    iteration_count: usize,
    reassigned_vertex_count: usize,
    maximum_reassigned_vertex_count_observed: usize,
    limit_vertex_count: usize,
    limit_exceeded: bool,
    initial_violation_count: usize,
}

fn enforce_branch_boundary_repair_limit_v1(
    reassigned_vertex_count: usize,
    limit_vertex_count: usize,
    options: ReferenceSupermodelSkinningOptionsV1,
) -> Result<bool, ReferenceSupermodelGenericErrorV2> {
    let exceeded = reassigned_vertex_count > limit_vertex_count;
    if exceeded && !options.allow_excessive_branch_boundary_repair {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-EXCESSIVE",
            "skinning.branchBoundaryRepair",
            format!(
                "local branch continuity would reassign {reassigned_vertex_count} vertices; the structural limit is {limit_vertex_count}; enable the explicit experimental branch-repair bypass to continue"
            ),
        ));
    }
    Ok(exceeded)
}

/// Makes the surface-to-skeleton map locally continuous without weakening the
/// final triangle audit.
///
/// ICM smoothing is intentionally a soft geometric optimization; at a branch
/// junction it can leave sibling labels on one small triangle.  Such a triangle
/// becomes a deformation bridge when the siblings rotate independently.  This
/// bounded pass resolves only those local triangles.  Every affected triangle
/// chooses one carrier from the selected supermodel topology, and coincident UV
/// seam vertices move as one atomic group.  A source that would require changing
/// more than five percent of its vertices is rejected as structurally mismatched
/// unless the caller explicitly enables the experimental bypass.
#[allow(clippy::too_many_arguments)]
fn repair_triangle_branch_boundaries(
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
    mut labels: Vec<u32>,
    options: ReferenceSupermodelSkinningOptionsV1,
) -> Result<BranchBoundaryRepairV1, ReferenceSupermodelGenericErrorV2> {
    let original = labels.clone();
    let maximum_reassigned_vertex_count = (positions.len() / 20).max(16);
    // Every indexed edge is renderer-visible and therefore participates in
    // the deformation contract.  The former diagonal-relative cut-off hid
    // long fur-card triangles from label repair even though the inherited-
    // motion oracle correctly measured them, allowing distant carriers to
    // tear one real triangle apart.
    let adjacency = surface_adjacency(positions, indices)?;
    let topology = carrier_topology_distance_matrix(contract);
    let initial_violation_count =
        cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)?;
    if initial_violation_count == 0 {
        return Ok(BranchBoundaryRepairV1 {
            labels,
            iteration_count: 0,
            reassigned_vertex_count: 0,
            maximum_reassigned_vertex_count_observed: 0,
            limit_vertex_count: maximum_reassigned_vertex_count,
            limit_exceeded: false,
            initial_violation_count: 0,
        });
    }
    promote_midline_bilateral_branch_groups_to_common_carrier(
        positions,
        allowed,
        fitted_worlds,
        contract,
        diagonal,
        &mut labels,
    );
    if cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)? == 0 {
        let reassigned_vertex_count = labels
            .iter()
            .zip(&original)
            .filter(|(left, right)| *left != *right)
            .count();
        let limit_exceeded = enforce_branch_boundary_repair_limit_v1(
            reassigned_vertex_count,
            maximum_reassigned_vertex_count,
            options,
        )?;
        return Ok(BranchBoundaryRepairV1 {
            reassigned_vertex_count,
            maximum_reassigned_vertex_count_observed: reassigned_vertex_count,
            labels,
            iteration_count: 1,
            limit_vertex_count: maximum_reassigned_vertex_count,
            limit_exceeded,
            initial_violation_count,
        });
    }

    const MAXIMUM_ITERATIONS: usize = 24;
    let mut iteration_count = 0;
    let mut maximum_reassigned_vertex_count_observed = 0;
    let mut limit_exceeded = false;
    for iteration in 0..MAXIMUM_ITERATIONS {
        let promoted = promote_incomparable_triangles_to_common_carrier(
            positions,
            indices,
            allowed,
            contract,
            &mut labels,
        );
        if promoted > 0 {
            synchronize_duplicate_label_groups(
                positions,
                &adjacency,
                allowed,
                fitted_worlds,
                contract,
                diagonal,
                &topology,
                &mut labels,
            );
        }
        if cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)? == 0 {
            iteration_count = iteration + 1;
            break;
        }
        let active = (0..positions.len())
            .filter(|vertex| {
                adjacency[*vertex].iter().any(|neighbor| {
                    !carrier_transition_is_local(
                        labels[*vertex],
                        labels[*neighbor],
                        contract,
                        MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
                    )
                })
            })
            .collect::<Vec<_>>();
        if active.is_empty() {
            break;
        }
        let mut changed = false;
        for vertex in active {
            let current = labels[vertex];
            let best = allowed
                .iter()
                .copied()
                .min_by(|left, right| {
                    branch_boundary_vertex_energy(
                        vertex,
                        *left,
                        positions,
                        &labels,
                        &adjacency,
                        &topology,
                        fitted_worlds,
                        contract,
                        diagonal,
                    )
                    .total_cmp(&branch_boundary_vertex_energy(
                        vertex,
                        *right,
                        positions,
                        &labels,
                        &adjacency,
                        &topology,
                        fitted_worlds,
                        contract,
                        diagonal,
                    ))
                    .then(left.cmp(right))
                })
                .expect("validated non-empty allowed carrier set");
            let current_energy = branch_boundary_vertex_energy(
                vertex,
                current,
                positions,
                &labels,
                &adjacency,
                &topology,
                fitted_worlds,
                contract,
                diagonal,
            );
            let best_energy = branch_boundary_vertex_energy(
                vertex,
                best,
                positions,
                &labels,
                &adjacency,
                &topology,
                fitted_worlds,
                contract,
                diagonal,
            );
            if best != current && best_energy + 1.0e-6 < current_energy {
                labels[vertex] = best;
                changed = true;
            }
        }
        iteration_count = iteration + 1;
        let reassigned = labels
            .iter()
            .zip(&original)
            .filter(|(left, right)| *left != *right)
            .count();
        maximum_reassigned_vertex_count_observed =
            maximum_reassigned_vertex_count_observed.max(reassigned);
        limit_exceeded |= enforce_branch_boundary_repair_limit_v1(
            reassigned,
            maximum_reassigned_vertex_count,
            options,
        )?;
        if !changed {
            changed = repair_conflict_groups(
                positions,
                &adjacency,
                allowed,
                fitted_worlds,
                contract,
                diagonal,
                &topology,
                &mut labels,
            );
        }
        changed |= synchronize_duplicate_label_groups(
            positions,
            &adjacency,
            allowed,
            fitted_worlds,
            contract,
            diagonal,
            &topology,
            &mut labels,
        );
        if cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)? == 0 {
            break;
        }
        if !changed {
            break;
        }
    }
    let mut remaining =
        cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)?;
    if remaining > 0 {
        iteration_count += repair_remaining_conflicts_via_carrier_path(
            positions,
            &adjacency,
            allowed,
            fitted_worlds,
            contract,
            diagonal,
            &topology,
            &mut labels,
        );
        remaining = cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)?;
    }
    if remaining > 0 {
        iteration_count += repair_conflict_triangles_transactionally(
            positions,
            indices,
            allowed,
            contract,
            &mut labels,
        );
        remaining = cross_branch_triangle_count_from_labels(positions, indices, &labels, contract)?;
    }
    if remaining > 0 {
        let pairs = cross_branch_triangle_pair_summary(positions, indices, &labels, contract, 12)?;
        let details =
            cross_branch_triangle_detail_summary(positions, indices, &labels, contract, 8)?;
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-NONCONVERGENT",
            "skinning.branchBoundaryRepair",
            format!(
                "bounded topology repair left {remaining} local cross-branch triangles after {iteration_count} iterations; dominantPairs={pairs:?}; details={details:?}"
            ),
        ));
    }
    let reassigned_vertex_count = labels
        .iter()
        .zip(original)
        .filter(|(left, right)| **left != *right)
        .count();
    maximum_reassigned_vertex_count_observed =
        maximum_reassigned_vertex_count_observed.max(reassigned_vertex_count);
    limit_exceeded |= enforce_branch_boundary_repair_limit_v1(
        reassigned_vertex_count,
        maximum_reassigned_vertex_count,
        options,
    )?;
    Ok(BranchBoundaryRepairV1 {
        labels,
        iteration_count,
        reassigned_vertex_count,
        maximum_reassigned_vertex_count_observed,
        limit_vertex_count: maximum_reassigned_vertex_count,
        limit_exceeded,
        initial_violation_count,
    })
}

fn promote_midline_bilateral_branch_groups_to_common_carrier(
    positions: &[[f32; 3]],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
    labels: &mut [u32],
) -> usize {
    let groups = duplicate_position_groups(positions);
    let min_x = positions
        .iter()
        .map(|position| position[0])
        .min_by(f32::total_cmp)
        .unwrap_or(0.0);
    let max_x = positions
        .iter()
        .map(|position| position[0])
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    let symmetry_plane_x = (min_x + max_x) * 0.5;
    let dead_zone = diagonal * 0.015;
    let mut changed = 0usize;
    for group in groups.values() {
        let center_x = group
            .iter()
            .map(|vertex| positions[*vertex][0])
            .sum::<f32>()
            / group.len() as f32;
        if (center_x - symmetry_plane_x).abs() > dead_zone {
            continue;
        }
        let proposals = group
            .iter()
            .filter_map(|vertex| {
                bilateral_branch_common_carrier(
                    labels[*vertex],
                    symmetry_plane_x,
                    dead_zone,
                    fitted_worlds,
                    contract,
                )
            })
            .collect::<Vec<_>>();
        let Some(anchor) = lowest_common_carrier(&proposals, contract) else {
            continue;
        };
        if !allowed.contains(&anchor) {
            continue;
        }
        for vertex in group {
            changed += usize::from(labels[*vertex] != anchor);
            labels[*vertex] = anchor;
        }
    }
    changed
}

fn bilateral_branch_common_carrier(
    mut part: u32,
    symmetry_plane_x: f32,
    dead_zone: f32,
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Option<u32> {
    loop {
        let parent = contract.nodes[part as usize].parent_part_number?;
        let part_offset = fitted_worlds[part as usize][0] - symmetry_plane_x;
        if part_offset.abs() > dead_zone
            && contract.nodes.iter().any(|sibling| {
                sibling.part_number != part
                    && sibling.parent_part_number == Some(parent)
                    && sibling.structural_role == contract.nodes[part as usize].structural_role
                    && {
                        let sibling_offset =
                            fitted_worlds[sibling.part_number as usize][0] - symmetry_plane_x;
                        sibling_offset.abs() > dead_zone && sibling_offset * part_offset < 0.0
                    }
            })
        {
            return Some(parent);
        }
        part = parent;
    }
}

fn cross_branch_triangle_detail_summary(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    maximum: usize,
) -> Result<Vec<String>, ReferenceSupermodelGenericErrorV2> {
    let adjacency = surface_adjacency(positions, indices)?;
    let groups = duplicate_position_groups(positions);
    let mut output = Vec::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if triangle_labels_are_local(vertices, labels, contract) {
            continue;
        }
        let vertices = vertices
            .iter()
            .map(|vertex| {
                let key = positions[*vertex].map(f32::to_bits);
                let neighbors = adjacency[*vertex]
                    .iter()
                    .map(|neighbor| labels[*neighbor])
                    .collect::<BTreeSet<_>>();
                format!(
                    "v{vertex}:{}({}) group={} neighbors={neighbors:?} pos={:?}",
                    contract.nodes[labels[*vertex] as usize].name,
                    labels[*vertex],
                    groups[&key].len(),
                    positions[*vertex],
                )
            })
            .collect::<Vec<_>>();
        output.push(vertices.join(" | "));
        if output.len() == maximum {
            break;
        }
    }
    Ok(output)
}

fn repair_conflict_triangles_transactionally(
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    labels: &mut [u32],
) -> usize {
    let groups = duplicate_position_groups(positions);
    let vertex_group = groups
        .iter()
        .flat_map(|(key, vertices)| vertices.iter().map(move |vertex| (*vertex, *key)))
        .collect::<BTreeMap<_, _>>();
    let triangles = indices
        .chunks_exact(3)
        .map(|triangle| {
            [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ]
        })
        .collect::<Vec<_>>();
    let mut incident = vec![Vec::<usize>::new(); positions.len()];
    for (triangle_index, vertices) in triangles.iter().enumerate() {
        for vertex in vertices {
            incident[*vertex].push(triangle_index);
        }
    }
    let mut completed_sweeps = 0usize;
    for sweep in 0..contract.nodes.len().saturating_mul(2).max(1) {
        let mut changed = false;
        for vertices in triangles.iter().copied() {
            if triangle_labels_are_local(vertices, labels, contract) {
                continue;
            }
            let triangle_labels = vertices.map(|vertex| labels[vertex]);
            let (left, right, distance) = triangle_labels
                .iter()
                .flat_map(|left| triangle_labels.iter().map(move |right| (*left, *right)))
                .map(|(left, right)| {
                    (
                        left,
                        right,
                        carrier_topology_distance(left, right, contract),
                    )
                })
                .max_by_key(|(_, _, distance)| *distance)
                .expect("triangle labels are non-empty");
            let path = carrier_path(left, right, contract);
            if distance != 2 || path.len() != 3 || !allowed.contains(&path[1]) {
                continue;
            }
            let middle = path[1];
            let affected = vertices
                .iter()
                .flat_map(|vertex| groups[&vertex_group[vertex]].iter().copied())
                .collect::<BTreeSet<_>>();
            let previous = affected
                .iter()
                .map(|vertex| (*vertex, labels[*vertex]))
                .collect::<Vec<_>>();
            for vertex in &affected {
                labels[*vertex] = middle;
            }
            let touched_triangles = affected
                .iter()
                .flat_map(|vertex| incident[*vertex].iter().copied())
                .collect::<BTreeSet<_>>();
            let remains_local = touched_triangles.into_iter().all(|triangle_index| {
                triangle_labels_are_local(triangles[triangle_index], labels, contract)
            });
            if remains_local {
                changed = true;
            } else {
                for (vertex, label) in previous {
                    labels[vertex] = label;
                }
            }
        }
        completed_sweeps = sweep + 1;
        let no_conflicts = triangles
            .iter()
            .copied()
            .all(|vertices| triangle_labels_are_local(vertices, labels, contract));
        if !changed || no_conflicts {
            break;
        }
    }
    completed_sweeps
}

fn promote_incomparable_triangles_to_common_carrier(
    _positions: &[[f32; 3]],
    indices: &[u32],
    allowed: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    labels: &mut [u32],
) -> usize {
    let previous = labels.to_vec();
    let mut proposals = vec![BTreeSet::<u32>::new(); labels.len()];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if triangle_labels_are_local(vertices, &previous, contract) {
            continue;
        }
        let triangle_labels = vertices.map(|vertex| previous[vertex]);
        let Some(common) = lowest_common_carrier(&triangle_labels, contract) else {
            continue;
        };
        if !allowed.contains(&common) {
            continue;
        }
        for vertex in vertices {
            proposals[vertex].insert(common);
        }
    }
    let mut changed = 0usize;
    for (vertex, candidates) in proposals.into_iter().enumerate() {
        let Some(candidate) = candidates.into_iter().min_by(|left, right| {
            carrier_topology_distance(previous[vertex], *left, contract)
                .cmp(&carrier_topology_distance(
                    previous[vertex],
                    *right,
                    contract,
                ))
                .then(left.cmp(right))
        }) else {
            continue;
        };
        if labels[vertex] != candidate {
            labels[vertex] = candidate;
            changed += 1;
        }
    }
    changed
}

fn lowest_common_carrier(
    parts: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Option<u32> {
    let (&first, rest) = parts.split_first()?;
    let mut ancestors = Vec::new();
    let mut current = first;
    loop {
        ancestors.push(current);
        let Some(parent) = contract.nodes[current as usize].parent_part_number else {
            break;
        };
        current = parent;
    }
    ancestors.into_iter().find(|candidate| {
        rest.iter()
            .all(|part| carrier_is_ancestor_or_self(*candidate, *part, contract))
    })
}

#[allow(clippy::too_many_arguments)]
fn repair_remaining_conflicts_via_carrier_path(
    positions: &[[f32; 3]],
    adjacency: &[Vec<usize>],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
    topology: &[Vec<usize>],
    labels: &mut [u32],
) -> usize {
    let duplicate_groups = duplicate_position_groups(positions);
    const MAXIMUM_PATH_SWEEPS: usize = 16;
    let mut completed_sweeps = 0;
    for sweep in 0..MAXIMUM_PATH_SWEEPS {
        let mut changed = false;
        let mut visited_edges = BTreeSet::new();
        for vertex in 0..positions.len() {
            for &neighbor in &adjacency[vertex] {
                let edge = if vertex < neighbor {
                    (vertex, neighbor)
                } else {
                    (neighbor, vertex)
                };
                if !visited_edges.insert(edge)
                    || carrier_transition_is_local(
                        labels[vertex],
                        labels[neighbor],
                        contract,
                        MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
                    )
                {
                    continue;
                }
                let path = carrier_path(labels[vertex], labels[neighbor], contract);
                let mut moves = Vec::<(f32, usize, u32)>::new();
                for candidate in path.into_iter().filter(|part| allowed.contains(part)) {
                    if carrier_transition_is_local(
                        candidate,
                        labels[neighbor],
                        contract,
                        MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
                    ) {
                        let group = duplicate_groups
                            .get(&positions[vertex].map(f32::to_bits))
                            .expect("every vertex has one duplicate group");
                        moves.push((
                            branch_boundary_group_energy(
                                group,
                                Some(candidate),
                                positions,
                                labels,
                                adjacency,
                                topology,
                                fitted_worlds,
                                contract,
                                diagonal,
                            ),
                            vertex,
                            candidate,
                        ));
                    }
                    if carrier_transition_is_local(
                        candidate,
                        labels[vertex],
                        contract,
                        MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
                    ) {
                        let group = duplicate_groups
                            .get(&positions[neighbor].map(f32::to_bits))
                            .expect("every vertex has one duplicate group");
                        moves.push((
                            branch_boundary_group_energy(
                                group,
                                Some(candidate),
                                positions,
                                labels,
                                adjacency,
                                topology,
                                fitted_worlds,
                                contract,
                                diagonal,
                            ),
                            neighbor,
                            candidate,
                        ));
                    }
                }
                let Some((_, endpoint, candidate)) = moves.into_iter().min_by(|left, right| {
                    left.0
                        .total_cmp(&right.0)
                        .then(left.1.cmp(&right.1))
                        .then(left.2.cmp(&right.2))
                }) else {
                    continue;
                };
                let group = duplicate_groups
                    .get(&positions[endpoint].map(f32::to_bits))
                    .expect("every vertex has one duplicate group");
                for &coincident in group {
                    changed |= labels[coincident] != candidate;
                    labels[coincident] = candidate;
                }
            }
        }
        completed_sweeps = sweep + 1;
        if !changed {
            break;
        }
        if !labels_have_cross_branch_edges(adjacency, contract, labels) {
            break;
        }
    }
    completed_sweeps
}

fn duplicate_position_groups(positions: &[[f32; 3]]) -> BTreeMap<[u32; 3], Vec<usize>> {
    let mut groups = BTreeMap::<[u32; 3], Vec<usize>>::new();
    for (vertex, position) in positions.iter().enumerate() {
        groups
            .entry(position.map(f32::to_bits))
            .or_default()
            .push(vertex);
    }
    groups
}

fn split_coincident_vertices_by_local_lineage_v1(
    positions: &[[f32; 3]],
    supports: &[BTreeSet<u32>],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Vec<Vec<usize>> {
    let mut output = Vec::<Vec<usize>>::new();
    for coincident in duplicate_position_groups(positions).into_values() {
        let mut local_groups = Vec::<(Vec<usize>, BTreeSet<u32>)>::new();
        for vertex in coincident {
            let mut selected = None;
            for (group_index, (_, group_support)) in local_groups.iter().enumerate() {
                let union = group_support
                    .union(&supports[vertex])
                    .copied()
                    .collect::<BTreeSet<_>>();
                if carrier_support_is_single_lineage(
                    &union,
                    contract,
                    MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
                ) {
                    selected = Some((group_index, union));
                    break;
                }
            }
            if let Some((group_index, union)) = selected {
                local_groups[group_index].0.push(vertex);
                local_groups[group_index].1 = union;
            } else {
                local_groups.push((vec![vertex], supports[vertex].clone()));
            }
        }
        output.extend(local_groups.into_iter().map(|(vertices, _)| vertices));
    }
    output
}

fn duplicate_position_label_topology_groups_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<Vec<Vec<usize>>, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != labels.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-SEAM-LABEL-LAYOUT",
            "skinning.labels",
            "topology-aware seam grouping requires one label per source vertex",
        ));
    }
    let mut supports = labels
        .iter()
        .map(|label| BTreeSet::from([*label]))
        .collect::<Vec<_>>();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "topology-aware seam grouping encountered an out-of-range index",
            ));
        }
        let triangle_support = vertices
            .iter()
            .map(|vertex| labels[*vertex])
            .collect::<BTreeSet<_>>();
        for vertex in vertices {
            supports[vertex].extend(triangle_support.iter().copied());
        }
    }
    Ok(split_coincident_vertices_by_local_lineage_v1(
        positions, &supports, contract,
    ))
}

fn duplicate_position_weight_topology_groups_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    weights: &[Vec<RigWeightInfluenceV1>],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<Vec<Vec<usize>>, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-SEAM-WEIGHT-LAYOUT",
            "skinning.weights",
            "topology-aware seam grouping requires one weight row per source vertex",
        ));
    }
    let mut supports = weights
        .iter()
        .map(|row| {
            row.iter()
                .filter(|influence| influence.value > 0.0)
                .map(|influence| influence.bone_node_id)
                .collect::<BTreeSet<_>>()
        })
        .collect::<Vec<_>>();
    let direct_supports = supports.clone();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "weight-topology seam grouping encountered an out-of-range index",
            ));
        }
        let triangle_support = vertices
            .iter()
            .flat_map(|vertex| direct_supports[*vertex].iter().copied())
            .collect::<BTreeSet<_>>();
        for vertex in vertices {
            supports[vertex].extend(triangle_support.iter().copied());
        }
    }
    // A collapsed group may represent a renderer vertex only when every row
    // is identical. Compatible bone sets alone do not make unequal weights
    // interchangeable; substituting the first row invents edges and supports.
    let groups = split_coincident_vertices_by_local_lineage_v1(positions, &supports, contract);
    Ok(groups
        .into_iter()
        .flat_map(|group| {
            let mut by_row = BTreeMap::<Vec<(u32, u32)>, Vec<usize>>::new();
            for vertex in group {
                let mut key = weights[vertex]
                    .iter()
                    .map(|w| (w.bone_node_id, w.value.to_bits()))
                    .collect::<Vec<_>>();
                key.sort_unstable();
                by_row.entry(key).or_default().push(vertex);
            }
            by_row.into_values()
        })
        .collect())
}

fn labels_have_cross_branch_edges(
    adjacency: &[Vec<usize>],
    contract: &ReferenceSupermodelMotionContractV2,
    labels: &[u32],
) -> bool {
    adjacency.iter().enumerate().any(|(vertex, neighbors)| {
        neighbors.iter().any(|neighbor| {
            !carrier_transition_is_local(
                labels[vertex],
                labels[*neighbor],
                contract,
                MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
            )
        })
    })
}

fn carrier_path(left: u32, right: u32, contract: &ReferenceSupermodelMotionContractV2) -> Vec<u32> {
    let ancestors = |mut part: u32| {
        let mut output = Vec::new();
        loop {
            output.push(part);
            let Some(parent) = contract.nodes[part as usize].parent_part_number else {
                break;
            };
            part = parent;
        }
        output
    };
    let left_ancestors = ancestors(left);
    let right_ancestors = ancestors(right);
    let right_set = right_ancestors.iter().copied().collect::<BTreeSet<_>>();
    let common = left_ancestors
        .iter()
        .copied()
        .find(|part| right_set.contains(part))
        .expect("validated carrier tree has a common root");
    let mut path = left_ancestors
        .iter()
        .copied()
        .take_while(|part| *part != common)
        .collect::<Vec<_>>();
    path.push(common);
    let mut right_branch = right_ancestors
        .iter()
        .copied()
        .take_while(|part| *part != common)
        .collect::<Vec<_>>();
    right_branch.reverse();
    path.extend(right_branch);
    path
}

#[allow(clippy::too_many_arguments)]
fn synchronize_duplicate_label_groups(
    positions: &[[f32; 3]],
    adjacency: &[Vec<usize>],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
    topology: &[Vec<usize>],
    labels: &mut [u32],
) -> bool {
    let mut groups = BTreeMap::<[u32; 3], Vec<usize>>::new();
    for (vertex, position) in positions.iter().enumerate() {
        groups
            .entry(position.map(f32::to_bits))
            .or_default()
            .push(vertex);
    }
    let mut changed = false;
    for group in groups.values().filter(|group| group.len() > 1) {
        // UV duplication is not permission to relabel a consistent surface.
        // Previously every duplicate group was re-scored geometrically, which
        // erased anatomical guides across most of a textured source mesh.
        if group
            .iter()
            .all(|vertex| labels[*vertex] == labels[group[0]])
        {
            continue;
        }
        let mut candidates = group
            .iter()
            .map(|vertex| labels[*vertex])
            .collect::<BTreeSet<_>>();
        if let Some(common) =
            lowest_common_carrier(&candidates.iter().copied().collect::<Vec<_>>(), contract)
                .filter(|part| allowed.contains(part))
        {
            candidates.insert(common);
        }
        let anchor = candidates
            .iter()
            .copied()
            .min_by(|left, right| {
                branch_boundary_group_energy(
                    group,
                    Some(*left),
                    positions,
                    labels,
                    adjacency,
                    topology,
                    fitted_worlds,
                    contract,
                    diagonal,
                )
                .total_cmp(&branch_boundary_group_energy(
                    group,
                    Some(*right),
                    positions,
                    labels,
                    adjacency,
                    topology,
                    fitted_worlds,
                    contract,
                    diagonal,
                ))
                .then(left.cmp(right))
            })
            .expect("validated non-empty allowed carrier set");
        for vertex in group {
            changed |= labels[*vertex] != anchor;
            labels[*vertex] = anchor;
        }
    }
    changed
}

fn cross_branch_triangle_pair_summary(
    _positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    maximum: usize,
) -> Result<Vec<String>, ReferenceSupermodelGenericErrorV2> {
    let mut counts = BTreeMap::<(u32, u32), usize>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= labels.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "triangle pair audit encountered an out-of-range vertex",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = (labels[vertices[left]], labels[vertices[right]]);
            if carrier_transition_is_local(
                pair.0,
                pair.1,
                contract,
                MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
            ) {
                continue;
            }
            let pair = if pair.0 < pair.1 {
                pair
            } else {
                (pair.1, pair.0)
            };
            *counts.entry(pair).or_default() += 1;
        }
    }
    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    Ok(counts
        .into_iter()
        .take(maximum)
        .map(|((left, right), count)| {
            format!(
                "{}({left})<->{}({right}):{count}",
                contract.nodes[left as usize].name, contract.nodes[right as usize].name,
            )
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
fn repair_conflict_groups(
    positions: &[[f32; 3]],
    adjacency: &[Vec<usize>],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
    topology: &[Vec<usize>],
    labels: &mut [u32],
) -> bool {
    let mut pending = (0..positions.len())
        .filter(|vertex| {
            adjacency[*vertex].iter().any(|neighbor| {
                !carrier_transition_is_local(
                    labels[*vertex],
                    labels[*neighbor],
                    contract,
                    MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let mut changed = false;
    while let Some(start) = pending.pop_first() {
        let mut group = BTreeSet::from([start]);
        let mut frontier = vec![start];
        while let Some(vertex) = frontier.pop() {
            for &neighbor in &adjacency[vertex] {
                if !carrier_transition_is_local(
                    labels[vertex],
                    labels[neighbor],
                    contract,
                    MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
                ) && pending.remove(&neighbor)
                    && group.insert(neighbor)
                {
                    frontier.push(neighbor);
                }
            }
        }
        let group = group.into_iter().collect::<Vec<_>>();
        let current_energy = branch_boundary_group_energy(
            &group,
            None,
            positions,
            labels,
            adjacency,
            topology,
            fitted_worlds,
            contract,
            diagonal,
        );
        let Some((anchor, best_energy)) = allowed
            .iter()
            .copied()
            .map(|anchor| {
                (
                    anchor,
                    branch_boundary_group_energy(
                        &group,
                        Some(anchor),
                        positions,
                        labels,
                        adjacency,
                        topology,
                        fitted_worlds,
                        contract,
                        diagonal,
                    ),
                )
            })
            .min_by(|left, right| left.1.total_cmp(&right.1).then(left.0.cmp(&right.0)))
        else {
            continue;
        };
        if best_energy + 1.0e-6 < current_energy {
            for vertex in group {
                changed |= labels[vertex] != anchor;
                labels[vertex] = anchor;
            }
        }
    }
    changed
}

#[allow(clippy::too_many_arguments)]
fn branch_boundary_group_energy(
    group: &[usize],
    common_label: Option<u32>,
    positions: &[[f32; 3]],
    labels: &[u32],
    adjacency: &[Vec<usize>],
    topology: &[Vec<usize>],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
) -> f32 {
    let group_set = group.iter().copied().collect::<BTreeSet<_>>();
    let mut energy = 0.0_f32;
    for &vertex in group {
        let label = common_label.unwrap_or(labels[vertex]);
        let geometry =
            segment_distance(positions[vertex], label, fitted_worlds, contract) / diagonal;
        let wrong_side =
            is_cross_side_influence(positions[vertex], label, fitted_worlds, diagonal, contract);
        if wrong_side {
            return f32::INFINITY;
        }
        energy += geometry;
    }
    let mut visited_edges = BTreeSet::new();
    for &vertex in group {
        for &neighbor in &adjacency[vertex] {
            let edge = if vertex < neighbor {
                (vertex, neighbor)
            } else {
                (neighbor, vertex)
            };
            if !visited_edges.insert(edge) {
                continue;
            }
            let left_label = common_label.unwrap_or(labels[vertex]);
            let right_label = if group_set.contains(&neighbor) {
                common_label.unwrap_or(labels[neighbor])
            } else {
                labels[neighbor]
            };
            let distance = topology[left_label as usize][right_label as usize];
            let excess = if carrier_transition_is_local(
                left_label,
                right_label,
                contract,
                MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
            ) {
                0
            } else {
                distance
                    .saturating_sub(MAX_LOCAL_LABEL_TRANSITION_DISTANCE)
                    .max(1)
            };
            energy += f32::from(excess > 0) * 1_000_000.0 + (excess * excess) as f32 * 10_000.0;
        }
    }
    energy
}

#[allow(clippy::too_many_arguments)]
fn branch_boundary_vertex_energy(
    vertex: usize,
    candidate: u32,
    positions: &[[f32; 3]],
    labels: &[u32],
    adjacency: &[Vec<usize>],
    topology: &[Vec<usize>],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
) -> f32 {
    let (violating_edges, squared_excess) = adjacency[vertex]
        .iter()
        .map(|neighbor| {
            let other = labels[*neighbor];
            let distance = topology[candidate as usize][other as usize];
            if carrier_transition_is_local(
                candidate,
                other,
                contract,
                MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
            ) {
                0
            } else {
                distance
                    .saturating_sub(MAX_LOCAL_LABEL_TRANSITION_DISTANCE)
                    .max(1)
            }
        })
        .fold((0usize, 0usize), |(count, sum), excess| {
            (count + usize::from(excess > 0), sum + excess * excess)
        });
    let geometry =
        segment_distance(positions[vertex], candidate, fitted_worlds, contract) / diagonal;
    let wrong_side = is_cross_side_influence(
        positions[vertex],
        candidate,
        fitted_worlds,
        diagonal,
        contract,
    );
    if wrong_side {
        return f32::INFINITY;
    }
    violating_edges as f32 * 1_000_000.0 + squared_excess as f32 * 10_000.0 + geometry
}

#[cfg(test)]
fn local_surface_adjacency(
    positions: &[[f32; 3]],
    indices: &[u32],
    edge_limit: f32,
) -> Result<Vec<Vec<usize>>, ReferenceSupermodelGenericErrorV2> {
    let mut adjacency = surface_adjacency(positions, indices)?;
    for (vertex, neighbors) in adjacency.iter_mut().enumerate() {
        neighbors.retain(|neighbor| {
            squared_distance(positions[vertex], positions[*neighbor]).sqrt() <= edge_limit
        });
    }
    Ok(adjacency)
}

fn carrier_topology_distance_matrix(
    contract: &ReferenceSupermodelMotionContractV2,
) -> Vec<Vec<usize>> {
    (0..contract.nodes.len())
        .map(|left| {
            (0..contract.nodes.len())
                .map(|right| carrier_topology_distance(left as u32, right as u32, contract))
                .collect()
        })
        .collect()
}

fn local_segment_weights(
    position: [f32; 3],
    primary: u32,
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
) -> Vec<RigWeightInfluenceV1> {
    let regularizer = diagonal * 0.012;
    let segment_distance = |part: u32| {
        let end = fitted_worlds[part as usize];
        let start = contract.nodes[part as usize]
            .parent_part_number
            .map(|parent| fitted_worlds[parent as usize])
            .unwrap_or(end);
        squared_distance_to_segment(position, start, end).sqrt()
    };
    let mut accumulated = allowed
        .iter()
        .copied()
        .filter(|bone| {
            carrier_topology_distance(primary, *bone, contract) <= 1
                && (*bone == primary
                    || !is_cross_side_influence(position, *bone, fitted_worlds, diagonal, contract))
        })
        .map(|bone| {
            let distance = segment_distance(bone);
            let topology = carrier_topology_distance(primary, bone, contract) as f32;
            let value = 1.0 / ((distance + regularizer).powi(2) * (1.0 + topology));
            (bone, value)
        })
        .collect::<BTreeMap<_, _>>();
    // `primary` is the topology label selected by the structural solver, not
    // merely another candidate in the blend.  A geometrically closer child or
    // parent used to become row[0] here, silently undoing the repaired label
    // field before the final triangle audit.  Keep the blend, but make the
    // selected carrier deterministically dominant by the smallest possible
    // finite margin.
    if let Some(primary_value) = accumulated.get(&primary).copied() {
        let competing_maximum = accumulated
            .iter()
            .filter(|(bone, _)| **bone != primary)
            .map(|(_, value)| *value)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);
        if primary_value <= competing_maximum {
            accumulated.insert(primary, next_f32_up(competing_maximum));
        }
    }
    normalize_top_weights(accumulated, 3, 0.08)
        .expect("finite geometry and regularizer produce a positive weight row")
}

fn next_f32_up(value: f32) -> f32 {
    if value.is_nan() || value == f32::INFINITY {
        return value;
    }
    if value == -0.0 {
        return f32::from_bits(1);
    }
    if value >= 0.0 {
        f32::from_bits(value.to_bits() + 1)
    } else {
        f32::from_bits(value.to_bits() - 1)
    }
}

fn nearest_segment_bone(
    position: [f32; 3],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    diagonal: f32,
) -> u32 {
    allowed
        .iter()
        .copied()
        .filter(|bone| !is_cross_side_influence(position, *bone, fitted_worlds, diagonal, contract))
        .min_by(|left, right| {
            segment_distance(position, *left, fitted_worlds, contract)
                .total_cmp(&segment_distance(position, *right, fitted_worlds, contract))
                .then(left.cmp(right))
        })
        .unwrap_or_else(|| {
            allowed
                .iter()
                .copied()
                .min_by(|left, right| {
                    segment_distance(position, *left, fitted_worlds, contract)
                        .total_cmp(&segment_distance(position, *right, fitted_worlds, contract))
                        .then(left.cmp(right))
                })
                .expect("validated non-empty allowed bone set")
        })
}

#[allow(clippy::too_many_arguments)]
fn regularize_primary_labels(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    adjacency: &[Vec<usize>],
    allowed: &[u32],
    fitted_worlds: &[[f32; 3]],
    diagonal: f32,
    seed_weights: &[Vec<RigWeightInfluenceV1>],
    mut labels: Vec<u32>,
    maximum_iterations: usize,
) -> (Vec<u32>, usize) {
    let original = labels.clone();
    for _ in 0..maximum_iterations {
        let previous = labels.clone();
        let mut changed = 0usize;
        for vertex in 0..positions.len() {
            let mut candidates = seed_weights[vertex]
                .iter()
                .map(|influence| influence.bone_node_id)
                .collect::<BTreeSet<_>>();
            candidates.insert(previous[vertex]);
            for &neighbor in &adjacency[vertex] {
                candidates.insert(previous[neighbor]);
                candidates.extend(
                    seed_weights[neighbor]
                        .iter()
                        .map(|influence| influence.bone_node_id),
                );
            }
            candidates.retain(|candidate| allowed.contains(candidate));
            let best = candidates
                .into_iter()
                .min_by(|left, right| {
                    let energy = |candidate: u32| {
                        let geometric =
                            segment_distance(positions[vertex], candidate, fitted_worlds, contract)
                                / diagonal;
                        let neighbor = if adjacency[vertex].is_empty() {
                            0.0
                        } else {
                            adjacency[vertex]
                                .iter()
                                .map(|other| {
                                    carrier_topology_distance(candidate, previous[*other], contract)
                                        .min(4) as f32
                                })
                                .sum::<f32>()
                                / adjacency[vertex].len() as f32
                        };
                        let wrong_side = is_cross_side_influence(
                            positions[vertex],
                            candidate,
                            fitted_worlds,
                            diagonal,
                            contract,
                        );
                        geometric + neighbor + if wrong_side { 1_000.0 } else { 0.0 }
                    };
                    energy(*left)
                        .total_cmp(&energy(*right))
                        .then(left.cmp(right))
                })
                .unwrap_or(previous[vertex]);
            labels[vertex] = best;
            changed += usize::from(best != previous[vertex]);
        }
        if changed == 0 {
            break;
        }
    }
    let reassigned = labels
        .iter()
        .zip(original)
        .filter(|(left, right)| **left != *right)
        .count();
    (labels, reassigned)
}

fn segment_distance(
    position: [f32; 3],
    bone: u32,
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
) -> f32 {
    let end = fitted_worlds[bone as usize];
    let start = contract.nodes[bone as usize]
        .parent_part_number
        .map(|parent| fitted_worlds[parent as usize])
        .unwrap_or(end);
    squared_distance_to_segment(position, start, end).sqrt()
}

fn normalize_top_weights(
    accumulated: BTreeMap<u32, f32>,
    maximum: usize,
    relative_floor: f32,
) -> Result<Vec<RigWeightInfluenceV1>, ReferenceSupermodelGenericErrorV2> {
    let mut sorted = accumulated
        .into_iter()
        .filter(|(_, value)| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let Some(maximum_value) = sorted.first().map(|row| row.1) else {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-INVALID",
            "skinning.weights",
            "local skinning produced an empty influence row",
        ));
    };
    sorted.retain(|row| row.1 >= maximum_value * relative_floor);
    sorted.truncate(maximum);
    let total = sorted.iter().map(|row| row.1).sum::<f32>();
    if !total.is_finite() || total <= 0.0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-INVALID",
            "skinning.weights",
            "local skinning produced a non-positive influence row",
        ));
    }
    Ok(sorted
        .into_iter()
        .map(|(bone_node_id, value)| RigWeightInfluenceV1 {
            bone_node_id,
            value: value / total,
        })
        .collect())
}

fn cross_branch_triangle_count(
    _positions: &[[f32; 3]],
    indices: &[u32],
    weights: &[Vec<RigWeightInfluenceV1>],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    let mut count = 0usize;
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= weights.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "triangle influence-continuity audit encountered an out-of-range vertex",
            ));
        }
        let influences = vertices
            .iter()
            .flat_map(|vertex| {
                weights[*vertex]
                    .iter()
                    .filter(|influence| influence.value > 0.0)
                    .map(|influence| influence.bone_node_id)
            })
            .collect::<BTreeSet<_>>();
        let local = carrier_support_is_single_lineage(
            &influences,
            contract,
            MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
        );
        count += usize::from(!local);
    }
    Ok(count)
}

fn authored_nonlocal_triangle_count_v1(
    _positions: &[[f32; 3]],
    indices: &[u32],
    weights: &[Vec<RigWeightInfluenceV1>],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    let mut count = 0usize;
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= weights.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "triangle influence-continuity audit encountered an out-of-range vertex",
            ));
        }
        let influences = vertices
            .iter()
            .flat_map(|vertex| {
                weights[*vertex]
                    .iter()
                    .filter(|influence| influence.value > 0.0)
                    .map(|influence| influence.bone_node_id)
            })
            .collect::<BTreeSet<_>>();
        let local = carrier_support_is_local_neighborhood_v1(
            &influences,
            contract,
            MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
        );
        count += usize::from(!local);
    }
    Ok(count)
}

#[derive(Clone, Debug, Default, PartialEq)]
struct LocalBoundaryConstraintStatsV1 {
    shared_carrier_group_count: usize,
    rigid_group_count: usize,
    promoted_joint_count: usize,
    edge_cliff_relaxation_iteration_count: usize,
    edge_cliff_relaxed_group_count: usize,
    primary_label_projection_group_count: usize,
    primary_label_projection_vertex_count: usize,
    chain_label_thickening_vertex_count: usize,
    chain_label_topology_repair_vertex_count: usize,
    geodesic_boundary_seed_group_count: usize,
    weight_gradient_relaxation_update_count: usize,
    weight_gradient_violation_edge_count: usize,
    maximum_weight_gradient_limit_ratio: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SmallComponentProjectionStatsV1 {
    component_count: usize,
    vertex_count: usize,
}

/// Prevents a small disconnected render island from being sheared internally
/// by a blended carrier pair. This runs on the original indexed source surface,
/// before the binary writer partitions one source component into several MDL
/// streams. Consequently a writer boundary cannot masquerade as a standalone
/// card and the projection remains independent of the output stream limit.
///
/// A component attached at exact-position seam vertices retains those boundary
/// rows byte-for-byte. When their local weight field differs only slightly, the
/// averaged attachment row is projected onto the component interior. The seam
/// therefore cannot open, while the component still inherits every terminal
/// paw, tail or generic appendage carrier present at its attachment.
fn stabilize_small_surface_components_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<SmallComponentProjectionStatsV1, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-COMPONENT-LAYOUT",
            "skinning.weights",
            "component projection requires one weight row per source vertex",
        ));
    }
    let mut adjacency = vec![Vec::<usize>::new(); positions.len()];
    let mut referenced = vec![false; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "small-component projection encountered an out-of-range index",
            ));
        }
        for vertex in vertices {
            referenced[vertex] = true;
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            adjacency[vertices[left]].push(vertices[right]);
            adjacency[vertices[right]].push(vertices[left]);
        }
    }
    for neighbors in &mut adjacency {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    let mut component_by_vertex = vec![usize::MAX; positions.len()];
    let mut components = Vec::<Vec<usize>>::new();
    for start in 0..positions.len() {
        if !referenced[start] || component_by_vertex[start] != usize::MAX {
            continue;
        }
        let component_index = components.len();
        component_by_vertex[start] = component_index;
        let mut pending = vec![start];
        let mut component = Vec::new();
        while let Some(vertex) = pending.pop() {
            component.push(vertex);
            for &neighbor in &adjacency[vertex] {
                if component_by_vertex[neighbor] == usize::MAX {
                    component_by_vertex[neighbor] = component_index;
                    pending.push(neighbor);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }

    let duplicate_groups = duplicate_position_groups(positions);
    let frozen_terminal_bones = contract
        .nodes
        .iter()
        .filter(|node| {
            node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL"
                || node.structural_role == "APPENDAGE_TERMINAL"
                || node.anchor_role.as_deref().is_some_and(|role| {
                    let role = role.to_ascii_lowercase();
                    role.contains("paw")
                        || role.starts_with("tail_")
                        || role.starts_with("appendage_")
                })
        })
        .map(|node| node.part_number)
        .collect::<BTreeSet<_>>();
    const MAXIMUM_STABILIZED_COMPONENT_VERTEX_COUNT: usize = 512;
    const MAXIMUM_ATTACHMENT_WEIGHT_DELTA: f32 = 0.1;
    let mut stats = SmallComponentProjectionStatsV1::default();
    for (component_index, component) in components.iter().enumerate() {
        if component.is_empty() || component.len() > MAXIMUM_STABILIZED_COMPONENT_VERTEX_COUNT {
            continue;
        }
        let boundary = component
            .iter()
            .copied()
            .filter(|vertex| {
                duplicate_groups[&positions[*vertex].map(f32::to_bits)]
                    .iter()
                    .copied()
                    .filter(|member| referenced[*member])
                    .any(|member| component_by_vertex[member] != component_index)
            })
            .collect::<Vec<_>>();
        if !boundary.is_empty() {
            let first = &weights[boundary[0]];
            let maximum_delta = boundary
                .iter()
                .map(|vertex| rig_weight_row_maximum_delta_v1(first, &weights[*vertex]))
                .fold(0.0_f32, f32::max);
            if maximum_delta > MAXIMUM_ATTACHMENT_WEIGHT_DELTA {
                continue;
            }
            let boundary_set = boundary.iter().copied().collect::<BTreeSet<_>>();
            let interior = component
                .iter()
                .copied()
                .filter(|vertex| !boundary_set.contains(vertex))
                .collect::<Vec<_>>();
            if interior.is_empty() {
                continue;
            }
            let mut accumulated = BTreeMap::<u32, f32>::new();
            let share = 1.0 / boundary.len() as f32;
            for vertex in boundary {
                for influence in &weights[vertex] {
                    if influence.value.is_finite() && influence.value > 0.0 {
                        *accumulated.entry(influence.bone_node_id).or_default() +=
                            influence.value * share;
                    }
                }
            }
            let attachment = normalize_local_branch_weights(accumulated, contract, 4, 0.0)?;
            let changed = interior.iter().any(|vertex| weights[*vertex] != attachment);
            if !changed {
                continue;
            }
            for vertex in interior.iter().copied() {
                weights[vertex] = attachment.clone();
            }
            stats.component_count += 1;
            stats.vertex_count += interior.len();
            continue;
        }

        // A genuinely isolated island has no seam contract. It may be made
        // rigid around its dominant carrier, except when that would erase a
        // terminal paw/tail/appendage deformation cluster.
        let mut accumulated = BTreeMap::<u32, f32>::new();
        for &vertex in component {
            for influence in &weights[vertex] {
                if influence.value.is_finite() && influence.value > 0.0 {
                    *accumulated.entry(influence.bone_node_id).or_default() += influence.value;
                }
            }
        }
        if accumulated
            .keys()
            .any(|bone| frozen_terminal_bones.contains(bone))
        {
            continue;
        }
        let Some(dominant) = accumulated
            .iter()
            .max_by(|left, right| left.1.total_cmp(right.1).then_with(|| right.0.cmp(left.0)))
            .map(|(bone, _)| *bone)
        else {
            continue;
        };
        let rigid = vec![RigWeightInfluenceV1 {
            bone_node_id: dominant,
            value: 1.0,
        }];
        let changed = component.iter().any(|vertex| weights[*vertex] != rigid);
        if !changed {
            continue;
        }
        for &vertex in component {
            weights[vertex] = rigid.clone();
        }
        stats.component_count += 1;
        stats.vertex_count += component.len();
    }
    Ok(stats)
}

fn rig_weight_row_maximum_delta_v1(
    left: &[RigWeightInfluenceV1],
    right: &[RigWeightInfluenceV1],
) -> f32 {
    let mut maximum = 0.0_f32;
    for influence in left {
        let other = right
            .iter()
            .find(|candidate| candidate.bone_node_id == influence.bone_node_id)
            .map_or(0.0, |candidate| candidate.value);
        maximum = maximum.max((influence.value - other).abs());
    }
    for influence in right {
        if left
            .iter()
            .all(|candidate| candidate.bone_node_id != influence.bone_node_id)
        {
            maximum = maximum.max(influence.value.abs());
        }
    }
    maximum
}

fn blend_weight_rows_with_domain_v1(
    own: &[RigWeightInfluenceV1],
    other: &[RigWeightInfluenceV1],
    alpha: f32,
    allowed: &BTreeSet<u32>,
) -> Result<Vec<RigWeightInfluenceV1>, ReferenceSupermodelGenericErrorV2> {
    let mut entries = [(0_u32, 0.0_f32); 8];
    let mut entry_count = 0usize;
    let mut accumulate = |bone: u32, value: f32| {
        if !value.is_finite() || value <= 0.0 || !allowed.contains(&bone) {
            return;
        }
        if let Some(entry) = entries[..entry_count]
            .iter_mut()
            .find(|entry| entry.0 == bone)
        {
            entry.1 += value;
        } else {
            debug_assert!(entry_count < entries.len());
            entries[entry_count] = (bone, value);
            entry_count += 1;
        }
    };
    for influence in own {
        accumulate(influence.bone_node_id, influence.value * (1.0 - alpha));
    }
    for influence in other {
        accumulate(influence.bone_node_id, influence.value * alpha);
    }
    entries[..entry_count].sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let kept = entry_count.min(4);
    let total = entries[..kept].iter().map(|entry| entry.1).sum::<f32>();
    if !total.is_finite() || total <= 0.0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-INVALID",
            "skinning.weights",
            "local edge blend produced a non-positive influence row",
        ));
    }
    Ok(entries[..kept]
        .iter()
        .map(|(bone_node_id, value)| RigWeightInfluenceV1 {
            bone_node_id: *bone_node_id,
            value: *value / total,
        })
        .collect())
}

fn constrain_weights_to_local_surface_label_field_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    allowed_bones: &[u32],
    fitted_worlds: &[[f32; 3]],
    diagonal: f32,
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<LocalBoundaryConstraintStatsV1, ReferenceSupermodelGenericErrorV2> {
    if labels.len() != positions.len() || weights.len() != positions.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-LABEL-LAYOUT",
            "skinning.weights",
            "surface labels, weights and positions must have identical lengths",
        ));
    }
    let adjacency = surface_adjacency(positions, indices)?;
    let groups = duplicate_position_groups(positions);
    for group in groups.values() {
        let group_center = std::array::from_fn(|axis| {
            group
                .iter()
                .map(|vertex| positions[*vertex][axis])
                .sum::<f32>()
                / group.len() as f32
        });
        let neighboring_labels = group
            .iter()
            .flat_map(|vertex| adjacency[*vertex].iter().map(|neighbor| labels[*neighbor]))
            .chain(group.iter().map(|vertex| labels[*vertex]))
            .collect::<BTreeSet<_>>();
        let group_labels = group
            .iter()
            .map(|vertex| labels[*vertex])
            .collect::<BTreeSet<_>>();
        let neighboring_labels = neighboring_labels.into_iter().collect::<Vec<_>>();
        let group_labels = group_labels.into_iter().collect::<Vec<_>>();
        let one_lineage = group_labels.iter().all(|left| {
            group_labels.iter().all(|right| {
                carrier_is_ancestor_or_self(*left, *right, contract)
                    || carrier_is_ancestor_or_self(*right, *left, contract)
            })
        });
        let anchor = if one_lineage {
            group_labels
                .iter()
                .copied()
                .max_by_key(|part| carrier_depth(*part, contract))
        } else {
            lowest_common_carrier(&group_labels, contract)
        }
        .ok_or_else(|| {
            error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-LINEAGE-MISSING",
                "skinning.weights",
                format!(
                    "surface group containing vertex {} has no common carrier lineage",
                    group[0]
                ),
            )
        })?;
        let regularizer = diagonal * 0.012;
        let mut accumulated = allowed_bones
            .iter()
            .copied()
            .filter(|candidate| {
                carrier_is_ancestor_or_self(*candidate, anchor, contract)
                    && carrier_topology_distance(*candidate, anchor, contract) <= 1
                    && neighboring_labels.iter().all(|label| {
                        carrier_transition_is_local(
                            *candidate,
                            *label,
                            contract,
                            MAX_LOCAL_INFLUENCE_TO_NEIGHBOR_LABEL_DISTANCE,
                        )
                    })
                    && (*candidate == anchor
                        || !is_cross_side_influence(
                            group_center,
                            *candidate,
                            fitted_worlds,
                            diagonal,
                            contract,
                        ))
            })
            .map(|candidate| {
                let distance = segment_distance(group_center, candidate, fitted_worlds, contract);
                (candidate, 1.0 / (distance + regularizer).powi(2))
            })
            .collect::<BTreeMap<_, _>>();
        if accumulated.is_empty() {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-LINEAGE-COVERAGE-MISSING",
                "skinning.weights",
                format!(
                    "surface group containing vertex {} has no allowed local carrier on lineage ending at {anchor}",
                    group[0]
                ),
            ));
        }
        if let Some(anchor_value) = accumulated.get(&anchor).copied() {
            let competing_maximum = accumulated
                .iter()
                .filter(|(candidate, _)| **candidate != anchor)
                .map(|(_, value)| *value)
                .max_by(f32::total_cmp)
                .unwrap_or(0.0);
            if anchor_value <= competing_maximum {
                accumulated.insert(anchor, next_f32_up(competing_maximum));
            }
        }
        let constrained = normalize_top_weights(accumulated, 4, 0.04)?;
        for &vertex in group {
            weights[vertex] = constrained.clone();
        }
    }
    let promoted_joint_count = promote_existing_local_influence_clusters_v1(
        positions,
        allowed_bones,
        fitted_worlds,
        contract,
        weights,
    )?;
    smooth_single_edge_lineage_weights_v1(
        positions,
        indices,
        &adjacency,
        fitted_worlds,
        diagonal,
        contract,
        weights,
        LOCAL_BOUNDARY_SMOOTHING_ITERATIONS,
    )?;
    let mut stats = LocalBoundaryConstraintStatsV1::default();
    stats.promoted_joint_count = promoted_joint_count;
    for group in groups.values() {
        let influence_count = weights[group[0]].len();
        let has_surface_edge = group.iter().any(|vertex| !adjacency[*vertex].is_empty());
        if influence_count == 1 && has_surface_edge {
            stats.rigid_group_count += 1;
        } else if influence_count > 1 {
            stats.shared_carrier_group_count += 1;
        }
    }
    Ok(stats)
}

fn smooth_single_edge_lineage_weights_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    adjacency: &[Vec<usize>],
    fitted_worlds: &[[f32; 3]],
    diagonal: f32,
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
    iterations: usize,
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    let groups = duplicate_position_groups(positions);
    let triangles = indices
        .chunks_exact(3)
        .map(|triangle| {
            [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ]
        })
        .collect::<Vec<_>>();
    let mut incident = vec![Vec::<usize>::new(); positions.len()];
    for (triangle_index, vertices) in triangles.iter().enumerate() {
        for vertex in vertices {
            incident[*vertex].push(triangle_index);
        }
    }
    for _ in 0..iterations {
        let previous = weights.to_vec();
        for group in groups.values() {
            let neighbors = group
                .iter()
                .flat_map(|vertex| adjacency[*vertex].iter().copied())
                .filter(|neighbor| !group.contains(neighbor))
                .collect::<BTreeSet<_>>();
            if neighbors.is_empty() {
                continue;
            }
            let support = group
                .iter()
                .copied()
                .chain(neighbors.iter().copied())
                .flat_map(|candidate| {
                    previous[candidate]
                        .iter()
                        .filter(|influence| influence.value > 0.0)
                        .map(|influence| influence.bone_node_id)
                })
                .collect::<BTreeSet<_>>();
            let one_edge_lineage = support.iter().all(|left| {
                support.iter().all(|right| {
                    carrier_transition_is_local(
                        *left,
                        *right,
                        contract,
                        MAX_LOCAL_INFLUENCE_TO_NEIGHBOR_LABEL_DISTANCE,
                    )
                })
            });
            if !one_edge_lineage {
                continue;
            }
            let group_share = 0.5 / group.len() as f32;
            let neighbor_share = 0.5 / neighbors.len() as f32;
            let mut accumulated = BTreeMap::<u32, f32>::new();
            for vertex in group {
                for influence in &previous[*vertex] {
                    *accumulated.entry(influence.bone_node_id).or_default() +=
                        influence.value * group_share;
                }
            }
            for neighbor in &neighbors {
                for influence in &previous[*neighbor] {
                    *accumulated.entry(influence.bone_node_id).or_default() +=
                        influence.value * neighbor_share;
                }
            }
            let group_center = std::array::from_fn(|axis| {
                group
                    .iter()
                    .map(|vertex| positions[*vertex][axis])
                    .sum::<f32>()
                    / group.len() as f32
            });
            accumulated.retain(|bone, _| {
                !is_cross_side_influence(group_center, *bone, fitted_worlds, diagonal, contract)
            });
            if accumulated.is_empty() {
                continue;
            }
            let candidate = normalize_top_weights(accumulated, 2, 0.01)?;
            let saved = group
                .iter()
                .map(|vertex| (*vertex, weights[*vertex].clone()))
                .collect::<Vec<_>>();
            for vertex in group {
                weights[*vertex] = candidate.clone();
            }
            let touched = group
                .iter()
                .flat_map(|vertex| incident[*vertex].iter().copied())
                .collect::<BTreeSet<_>>();
            let remains_local = touched.into_iter().all(|triangle_index| {
                triangle_weight_support_is_local(triangles[triangle_index], weights, contract)
            });
            if !remains_local {
                for (vertex, row) in saved {
                    weights[vertex] = row;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EdgeCliffRelaxationStatsV1 {
    iteration_count: usize,
    relaxed_group_count: usize,
}

/// Removes a renderer-visible 1.0-to-0.0 weight cliff on one actual surface
/// edge even when the wider one-ring also touches a third carrier. The earlier
/// group smoother rejects that whole one-ring to protect branch locality; this
/// pass instead edits one carrier edge transactionally and validates every
/// incident triangle before committing it.
#[cfg(test)]
fn relax_local_edge_weight_cliffs_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<EdgeCliffRelaxationStatsV1, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-EDGE-CLIFF-LAYOUT",
            "skinning.weights",
            "edge-cliff relaxation requires one weight row per source vertex",
        ));
    }
    let groups =
        duplicate_position_weight_topology_groups_v1(positions, indices, weights, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    for (group_index, group) in groups.iter().enumerate() {
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
        }
    }
    let mut triangles = Vec::<[usize; 3]>::new();
    let mut incident = vec![Vec::<usize>::new(); positions.len()];
    let mut edges = BTreeSet::<(usize, usize)>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "edge-cliff relaxation encountered an out-of-range index",
            ));
        }
        let triangle_index = triangles.len();
        triangles.push(vertices);
        for vertex in vertices {
            incident[vertex].push(triangle_index);
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = if vertices[left] < vertices[right] {
                (vertices[left], vertices[right])
            } else {
                (vertices[right], vertices[left])
            };
            edges.insert(pair);
        }
    }

    // Dense fur surfaces can contain a long, narrow blend strip.  The
    // deterministic pairwise projection converges monotonically, but the
    // former 64-round cap stopped with a small residual (105 of roughly
    // 900,000 real edges on the borzoi corpus, maximum delta 0.160420), while
    // 128 rounds left one 0.124473 edge on the rear-leg strip. Keep the hard
    // postcondition and provide enough bounded rounds for those strips to
    // converge instead of accepting or hiding the residual.
    const MAXIMUM_ITERATIONS: usize = 256;
    const MAXIMUM_RETAINED_EDGE_WEIGHT_DELTA: f32 = 0.1;
    let mut relaxed_groups = BTreeSet::<usize>::new();
    let mut completed_iterations = 0usize;
    for iteration in 0..MAXIMUM_ITERATIONS {
        let mut changed = false;
        for &(left_vertex, right_vertex) in &edges {
            let left_group = group_by_vertex[left_vertex];
            let right_group = group_by_vertex[right_vertex];
            if left_group == right_group {
                continue;
            }
            let left = weights[groups[left_group][0]].clone();
            let right = weights[groups[right_group][0]].clone();
            if rig_weight_row_maximum_delta_v1(&left, &right) <= MAXIMUM_RETAINED_EDGE_WEIGHT_DELTA
            {
                continue;
            }
            let support = left
                .iter()
                .chain(&right)
                .filter(|influence| influence.value.is_finite() && influence.value > 0.0)
                .map(|influence| influence.bone_node_id)
                .collect::<BTreeSet<_>>();
            let one_carrier_edge = support.iter().all(|left| {
                support.iter().all(|right| {
                    carrier_transition_is_local(
                        *left,
                        *right,
                        contract,
                        MAX_LOCAL_INFLUENCE_TO_NEIGHBOR_LABEL_DISTANCE,
                    )
                })
            });
            if !one_carrier_edge {
                continue;
            }
            let mut accumulated = BTreeMap::<u32, f32>::new();
            for influence in left.iter().chain(&right) {
                if influence.value.is_finite() && influence.value > 0.0 {
                    *accumulated.entry(influence.bone_node_id).or_default() +=
                        influence.value * 0.5;
                }
            }
            let candidate = normalize_local_branch_weights(accumulated, contract, 4, 0.0)?;
            let touched_groups = [left_group, right_group];
            let touched_vertices = touched_groups
                .iter()
                .flat_map(|group| groups[*group].iter().copied())
                .collect::<BTreeSet<_>>();
            let saved = touched_vertices
                .iter()
                .map(|vertex| (*vertex, weights[*vertex].clone()))
                .collect::<Vec<_>>();
            if saved.iter().all(|(_, row)| *row == candidate) {
                continue;
            }
            let touched_triangles = touched_vertices
                .iter()
                .flat_map(|vertex| incident[*vertex].iter().copied())
                .collect::<BTreeSet<_>>();
            for vertex in touched_vertices.iter().copied() {
                weights[vertex] = candidate.clone();
            }
            let remains_local = touched_triangles.iter().all(|triangle_index| {
                triangle_weight_support_is_local(triangles[*triangle_index], weights, contract)
            });
            if remains_local {
                for group in touched_groups {
                    relaxed_groups.insert(group);
                }
                changed = true;
                continue;
            }
            for (vertex, row) in saved {
                weights[vertex] = row;
            }

            // At an anatomical branch junction, changing the parent-side
            // vertex can introduce the selected child into a sibling limb's
            // one-ring.  The child-side row can still be moved safely toward
            // its parent: any grandchild encountered there remains on the
            // same carrier lineage.  Try that directional projection before
            // declaring the edge unresolved.
            if support.len() != 2 {
                continue;
            }
            let support_parts = support.iter().copied().collect::<Vec<_>>();
            let (ancestor, descendant) =
                if carrier_is_ancestor_or_self(support_parts[0], support_parts[1], contract) {
                    (support_parts[0], support_parts[1])
                } else if carrier_is_ancestor_or_self(support_parts[1], support_parts[0], contract)
                {
                    (support_parts[1], support_parts[0])
                } else {
                    continue;
                };
            if carrier_topology_distance(ancestor, descendant, contract) != 1 {
                continue;
            }
            let descendant_weight = |row: &[RigWeightInfluenceV1]| {
                row.iter()
                    .find(|influence| influence.bone_node_id == descendant)
                    .map_or(0.0, |influence| influence.value)
            };
            let directional_group = if descendant_weight(&left) > descendant_weight(&right) {
                left_group
            } else {
                right_group
            };
            let directional_vertices = groups[directional_group]
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let directional_saved = directional_vertices
                .iter()
                .map(|vertex| (*vertex, weights[*vertex].clone()))
                .collect::<Vec<_>>();
            let directional_triangles = directional_vertices
                .iter()
                .flat_map(|vertex| incident[*vertex].iter().copied())
                .collect::<BTreeSet<_>>();
            for vertex in directional_vertices.iter().copied() {
                weights[vertex] = candidate.clone();
            }
            let remains_single_lineage = directional_triangles.iter().all(|triangle_index| {
                triangle_weight_support_is_local(triangles[*triangle_index], weights, contract)
            });
            if !remains_single_lineage {
                for (vertex, row) in directional_saved {
                    weights[vertex] = row;
                }
                continue;
            }
            relaxed_groups.insert(directional_group);
            changed = true;
        }
        if !changed {
            break;
        }
        completed_iterations = iteration + 1;
    }
    Ok(EdgeCliffRelaxationStatsV1 {
        iteration_count: completed_iterations,
        relaxed_group_count: relaxed_groups.len(),
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PrimaryLabelProjectionStatsV1 {
    projected_group_count: usize,
    projected_vertex_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct WeightGradientRelaxationStatsV1 {
    processed_edge_count: usize,
    update_count: usize,
    relaxed_group_count: usize,
    violation_edge_count: usize,
    maximum_limit_ratio: f32,
    worst_edges: Vec<ReferenceSupermodelWeightGradientEdgeV1>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FinalBoundaryRelaxationStatsV2 {
    chain_label_thickening_vertex_count: usize,
    chain_label_topology_repair_vertex_count: usize,
    geodesic_boundary_seed_group_count: usize,
    auxiliary_projection: AuxiliaryProjectionStatsV3,
    component_projection: SmallComponentProjectionStatsV1,
    shared_carrier_group_count: usize,
    rigid_group_count: usize,
    projection: PrimaryLabelProjectionStatsV1,
    gradient: WeightGradientRelaxationStatsV1,
    skin_region_feasibility: Vec<ReferenceSupermodelSkinRegionFeasibilityV1>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct GeodesicSeedStatsV1 {
    changed_group_count: usize,
    skin_region_feasibility: Vec<ReferenceSupermodelSkinRegionFeasibilityV1>,
}

/// Restricts every renderer triangle to one local carrier lineage. A
/// parent-middle-child window is valid and necessary for a continuous blend
/// through a chain junction; sibling branches and larger hierarchy jumps are
/// projected to one exact parent-child edge.
fn project_triangle_support_to_one_carrier_edge_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<PrimaryLabelProjectionStatsV1, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != weights.len() || positions.len() != labels.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-TRIANGLE-PROJECTION-LAYOUT",
            "skinning.weights",
            "triangle support projection requires one label and weight row per source vertex",
        ));
    }
    let groups =
        duplicate_position_weight_topology_groups_v1(positions, indices, weights, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    for (group_index, group) in groups.iter().enumerate() {
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
        }
    }
    let triangles = indices
        .chunks_exact(3)
        .map(|triangle| {
            [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ]
        })
        .collect::<Vec<_>>();
    if triangles
        .iter()
        .flatten()
        .any(|vertex| *vertex >= positions.len())
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
            "surface.indices",
            "triangle support projection encountered an out-of-range index",
        ));
    }
    for (triangle_index, vertices) in triangles.iter().enumerate() {
        let label_support = vertices
            .iter()
            .map(|vertex| labels[*vertex])
            .collect::<BTreeSet<_>>();
        if !carrier_support_is_single_lineage(&label_support, contract, 1) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-PRIMARY-LABEL-TOPOLOGY",
                format!("skinning.triangles[{triangle_index}]"),
                format!(
                    "repaired primary labels span more than one carrier edge: {label_support:?}"
                ),
            ));
        }
    }

    let mut projected_groups = BTreeSet::<usize>::new();
    const MAXIMUM_SWEEPS: usize = 64;
    for _ in 0..MAXIMUM_SWEEPS {
        let mut changed = false;
        for vertices in &triangles {
            let group_indices = vertices.map(|vertex| group_by_vertex[vertex]);
            let rows = group_indices.map(|group| weights[groups[group][0]].clone());
            let support = rows
                .iter()
                .flat_map(|row| {
                    row.iter()
                        .filter(|influence| influence.value > 0.0)
                        .map(|influence| influence.bone_node_id)
                })
                .collect::<BTreeSet<_>>();
            if carrier_support_is_single_lineage(
                &support,
                contract,
                MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
            ) {
                continue;
            }
            let allowed = best_triangle_carrier_edge_v1(&rows, &support, contract)?;
            for group in group_indices.into_iter().collect::<BTreeSet<_>>() {
                let row = &weights[groups[group][0]];
                let mut retained = row
                    .iter()
                    .filter(|influence| allowed.contains(&influence.bone_node_id))
                    .map(|influence| (influence.bone_node_id, influence.value))
                    .collect::<BTreeMap<_, _>>();
                if retained.is_empty() {
                    let dominant = row
                        .iter()
                        .max_by(|left, right| {
                            left.value
                                .total_cmp(&right.value)
                                .then_with(|| right.bone_node_id.cmp(&left.bone_node_id))
                        })
                        .map(|influence| influence.bone_node_id)
                        .ok_or_else(|| {
                            error(
                                "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-INVALID",
                                "skinning.weights",
                                "triangle support projection received an empty row",
                            )
                        })?;
                    let replacement = allowed
                        .iter()
                        .copied()
                        .min_by_key(|candidate| {
                            (
                                carrier_topology_distance(dominant, *candidate, contract),
                                *candidate,
                            )
                        })
                        .expect("best carrier edge is non-empty");
                    retained.insert(replacement, 1.0);
                }
                let replacement = normalize_top_weights(retained, 2, 0.0)?;
                if replacement == *row {
                    continue;
                }
                for &vertex in &groups[group] {
                    weights[vertex] = replacement.clone();
                }
                projected_groups.insert(group);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let remaining = triangles
        .iter()
        .filter(|vertices| !triangle_weight_support_is_local_lineage(**vertices, weights, contract))
        .count();
    if remaining > 0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-TRIANGLE-PROJECTION-UNRESOLVED",
            "skinning.triangles",
            format!(
                "{remaining} triangles still span more than one carrier edge after {MAXIMUM_SWEEPS} deterministic projection sweeps"
            ),
        ));
    }
    Ok(PrimaryLabelProjectionStatsV1 {
        projected_group_count: projected_groups.len(),
        projected_vertex_count: projected_groups
            .iter()
            .map(|group| groups[*group].len())
            .sum(),
    })
}

fn best_triangle_carrier_edge_v1(
    rows: &[Vec<RigWeightInfluenceV1>; 3],
    support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<BTreeSet<u32>, ReferenceSupermodelGenericErrorV2> {
    let mut candidates = support
        .iter()
        .copied()
        .map(|bone| [bone, bone])
        .collect::<Vec<_>>();
    for &bone in support {
        if let Some(parent) = contract.nodes[bone as usize].parent_part_number {
            candidates.push(if parent < bone {
                [parent, bone]
            } else {
                [bone, parent]
            });
        }
        for child in contract
            .nodes
            .iter()
            .filter(|node| node.parent_part_number == Some(bone))
            .map(|node| node.part_number)
        {
            candidates.push(if bone < child {
                [bone, child]
            } else {
                [child, bone]
            });
        }
    }
    candidates.sort_unstable();
    candidates.dedup();
    let best = candidates
        .into_iter()
        .max_by(|left, right| {
            let score = |candidate: [u32; 2]| {
                rows.iter()
                    .flat_map(|row| row.iter())
                    .filter(|influence| candidate.contains(&influence.bone_node_id))
                    .map(|influence| influence.value)
                    .sum::<f32>()
            };
            score(*left)
                .total_cmp(&score(*right))
                .then_with(|| right.cmp(left))
        })
        .ok_or_else(|| {
            error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-TRIANGLE-SUPPORT-EMPTY",
                "skinning.weights",
                "triangle support projection found no carrier candidate",
            )
        })?;
    Ok(best.into_iter().collect())
}

fn triangle_weight_support_is_local_lineage(
    vertices: [usize; 3],
    weights: &[Vec<RigWeightInfluenceV1>],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    let support = vertices
        .iter()
        .flat_map(|vertex| {
            weights[*vertex]
                .iter()
                .filter(|influence| influence.value > 0.0)
                .map(|influence| influence.bone_node_id)
        })
        .collect::<BTreeSet<_>>();
    carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN)
}

#[derive(Clone, Copy, Debug)]
struct GeodesicBoundaryQueueEntryV1 {
    distance: f32,
    group: usize,
}

impl PartialEq for GeodesicBoundaryQueueEntryV1 {
    fn eq(&self, other: &Self) -> bool {
        self.distance.to_bits() == other.distance.to_bits() && self.group == other.group
    }
}

impl Eq for GeodesicBoundaryQueueEntryV1 {}

impl PartialOrd for GeodesicBoundaryQueueEntryV1 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GeodesicBoundaryQueueEntryV1 {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap; reverse both keys for a deterministic
        // minimum-distance queue.
        other
            .distance
            .total_cmp(&self.distance)
            .then_with(|| other.group.cmp(&self.group))
    }
}

/// Builds a finite-width blend strip directly from geodesic distance to each
/// local carrier-label boundary. This is a deterministic initial solution for
/// the geometry-scaled gradient constraint, rather than millions of pairwise
/// averaging steps across a dense cyclic mesh.
fn seed_local_lineage_geodesic_boundary_field_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != labels.len() || positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-BOUNDARY-LAYOUT",
            "skinning.weights",
            "geodesic boundary seeding requires one label and weight row per source vertex",
        ));
    }
    if fitted_worlds.len() != contract.nodes.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-FITTED-WORLDS",
            "jointFit.fittedWorldPositions",
            "geodesic boundary seeding requires one fitted world position per exact carrier",
        ));
    }
    let diagonal = model_diagonal(positions).max(1.0e-6);
    // Exact-position duplicates are one deformation seam only while their
    // incident semantic supports stay on one carrier lineage.  Raw position
    // welding would incorrectly join coincident fur cards from sibling limbs.
    let groups = duplicate_position_label_topology_groups_v1(positions, indices, labels, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    let mut group_labels = Vec::<u32>::with_capacity(groups.len());
    for (group_index, group) in groups.iter().enumerate() {
        let mut counts = BTreeMap::<u32, usize>::new();
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
            *counts.entry(labels[vertex]).or_default() += 1;
        }
        let selected = counts
            .into_iter()
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
            .map(|(label, _)| label)
            .expect("duplicate-position groups are non-empty");
        group_labels.push(selected);
    }
    let mut edge_set = BTreeSet::<(usize, usize)>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "geodesic boundary seeding encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = [
                group_by_vertex[vertices[left]],
                group_by_vertex[vertices[right]],
            ];
            if pair[0] != pair[1] {
                edge_set.insert(if pair[0] < pair[1] {
                    (pair[0], pair[1])
                } else {
                    (pair[1], pair[0])
                });
            }
        }
    }
    let mut adjacency = vec![Vec::<(usize, f32)>::new(); groups.len()];
    let mut boundary_seeds = BTreeMap::<(u32, u32), BTreeSet<usize>>::new();
    for (left, right) in edge_set {
        let length = squared_distance(positions[groups[left][0]], positions[groups[right][0]])
            .sqrt()
            .max(1.0e-9);
        adjacency[left].push((right, length));
        adjacency[right].push((left, length));
        let left_label = group_labels[left];
        let right_label = group_labels[right];
        if left_label == right_label {
            continue;
        }
        if carrier_topology_distance(left_label, right_label, contract) != 1 {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-BOUNDARY-TOPOLOGY",
                "skinning.labels",
                format!(
                    "surface edge ({},{}) crosses nonadjacent carrier labels ({left_label},{right_label})",
                    groups[left][0], groups[right][0]
                ),
            ));
        }
        let pair = if left_label < right_label {
            (left_label, right_label)
        } else {
            (right_label, left_label)
        };
        boundary_seeds
            .entry(pair)
            .or_default()
            .extend([left, right]);
    }
    for neighbors in &mut adjacency {
        neighbors.sort_by(|left, right| left.0.cmp(&right.0));
    }

    let mut boundary_shares = vec![Vec::<(u32, f32)>::new(); groups.len()];
    for ((left_label, right_label), seeds) in boundary_seeds {
        let support = BTreeSet::from([left_label, right_label]);
        let mut distances = vec![f32::INFINITY; groups.len()];
        let mut queue = BinaryHeap::<GeodesicBoundaryQueueEntryV1>::new();
        for seed in seeds {
            distances[seed] = 0.0;
            queue.push(GeodesicBoundaryQueueEntryV1 {
                distance: 0.0,
                group: seed,
            });
        }
        while let Some(entry) = queue.pop() {
            if entry.distance > distances[entry.group] || entry.distance >= 0.5 {
                continue;
            }
            let label = group_labels[entry.group];
            for &(neighbor, edge_length) in &adjacency[entry.group] {
                // Distance propagates inward on each side of the boundary;
                // crossing to the other label would create an artificial
                // shortcut through the opposite semantic region.
                if group_labels[neighbor] != label {
                    continue;
                }
                // Measure geodesic distance directly in admissible weight
                // change.  The shortest-path triangle inequality then makes
                // the resulting scalar field satisfy the exact same discrete
                // edge bound as the independent renderer audit.
                let edge_cost = geometry_scaled_weight_delta_limit_v1(
                    edge_length,
                    diagonal,
                    &support,
                    contract,
                    fitted_worlds,
                )?;
                let candidate = entry.distance + edge_cost;
                if candidate < distances[neighbor] && candidate < 0.5 {
                    distances[neighbor] = candidate;
                    queue.push(GeodesicBoundaryQueueEntryV1 {
                        distance: candidate,
                        group: neighbor,
                    });
                }
            }
        }
        for (group, distance) in distances.into_iter().enumerate() {
            if !distance.is_finite() || distance >= 0.5 {
                continue;
            }
            let own = group_labels[group];
            let other = if own == left_label {
                right_label
            } else if own == right_label {
                left_label
            } else {
                continue;
            };
            let share = (0.5 - 1.0e-5 - distance).max(0.0);
            boundary_shares[group].push((other, share));
        }
    }

    let mut changed_groups = 0usize;
    for (group_index, group) in groups.iter().enumerate() {
        let own = group_labels[group_index];
        let parent = contract.nodes[own as usize].parent_part_number;
        let parent_share = parent
            .and_then(|parent| {
                boundary_shares[group_index]
                    .iter()
                    .find(|(other, _)| *other == parent)
                    .map(|(_, share)| *share)
            })
            .unwrap_or(0.0);
        let child = boundary_shares[group_index]
            .iter()
            .copied()
            .filter(|(other, _)| contract.nodes[*other as usize].parent_part_number == Some(own))
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            });
        // This is the complete semantic weight field, not an overlay on a
        // different segment parameterization.  A group outside every blend
        // strip is rigidly owned by its anatomical label; otherwise the
        // boundary strip would meet an unrelated fitted-segment field and
        // recreate the exact cliff it is meant to remove.
        let child_share = child.map_or(0.0, |(_, share)| share);
        let mut accumulated = BTreeMap::from([(own, 1.0 - parent_share - child_share)]);
        if let Some(parent) = parent {
            if parent_share > 0.0 {
                accumulated.insert(parent, parent_share);
            }
        }
        if let Some((child, _)) = child {
            if child_share > 0.0 {
                accumulated.insert(child, child_share);
            }
        }
        let replacement = normalize_top_weights(accumulated, 3, 0.0)?;
        if weights[group[0]] != replacement {
            changed_groups += 1;
        }
        for &vertex in group {
            weights[vertex] = replacement.clone();
        }
    }
    Ok(changed_groups)
}

#[cfg(test)]
fn carrier_geodesic_blend_width_v1(
    carrier: u32,
    diagonal: f32,
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
) -> f32 {
    let incident_span = contract
        .nodes
        .iter()
        .filter(|node| {
            node.parent_part_number == Some(carrier)
                || contract.nodes[carrier as usize].parent_part_number == Some(node.part_number)
        })
        .map(|node| {
            squared_distance(
                fitted_worlds[carrier as usize],
                fitted_worlds[node.part_number as usize],
            )
            .sqrt()
        })
        .fold(diagonal * 0.01, f32::max);
    let permitted_extra_stretch = (contract.tolerances.edge_hard_max_ratio - 1.0).max(1.0e-6);
    // The multi-source solver never propagates beyond the four-lane carrier
    // window. Keep this width moderate so the oldest influence has naturally
    // decayed before the next semantic window begins.
    (GEODESIC_DERIVATIVE_SAFETY_FACTOR * incident_span / permitted_extra_stretch)
        .max(diagonal / 128.0)
}

fn carrier_label_minimum_gradient_work_v2() -> f32 {
    // A selected carrier needs a vertex whose own weight is strictly greater
    // than 0.5. Label allocation therefore uses the same dimensionless work
    // metric as the renderer-gradient solver and reserves one complete edge
    // allowance beyond the dominance boundary. Measuring this in Euclidean
    // model units was incorrect on dense/coarse mixed meshes: an edge capped
    // at MAX_RENDER_EDGE_WEIGHT_DELTA contributes less usable weight work than
    // its physical length suggests.
    0.5 + MAX_RENDER_EDGE_WEIGHT_DELTA + 1.0e-6
}

/// Ensures a middle carrier owns a finite geodesic strip between its parent
/// boundary and each child boundary. Without that strip a single triangle fan
/// can touch P-M-C (and a sibling branch at P) at once, making continuous
/// local-lineage weights mathematically impossible.
fn thicken_middle_carrier_label_regions_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &mut [u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    let diagonal = model_diagonal(positions).max(1.0e-6);
    let mut changed_vertices = BTreeSet::<usize>::new();
    for _ in 0..4 {
        let groups =
            duplicate_position_label_topology_groups_v1(positions, indices, labels, contract)?;
        let mut group_by_vertex = vec![usize::MAX; positions.len()];
        let mut group_labels = Vec::<u32>::with_capacity(groups.len());
        for (group_index, group) in groups.iter().enumerate() {
            for &vertex in group {
                group_by_vertex[vertex] = group_index;
            }
            group_labels.push(labels[group[0]]);
        }
        let mut edge_set = BTreeSet::<(usize, usize)>::new();
        for triangle in indices.chunks_exact(3) {
            let vertices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            for (left, right) in [(0, 1), (1, 2), (2, 0)] {
                let pair = [
                    group_by_vertex[vertices[left]],
                    group_by_vertex[vertices[right]],
                ];
                if pair[0] != pair[1] {
                    edge_set.insert(if pair[0] < pair[1] {
                        (pair[0], pair[1])
                    } else {
                        (pair[1], pair[0])
                    });
                }
            }
        }
        let mut adjacency = vec![Vec::<(usize, f32)>::new(); groups.len()];
        for (left, right) in edge_set {
            let length = squared_distance(positions[groups[left][0]], positions[groups[right][0]])
                .sqrt()
                .max(1.0e-9);
            adjacency[left].push((right, length));
            adjacency[right].push((left, length));
        }
        let mut relabel = BTreeMap::<usize, u32>::new();
        for middle in &contract.nodes {
            let Some(parent) = middle.parent_part_number else {
                continue;
            };
            let children = contract
                .nodes
                .iter()
                .filter(|node| node.parent_part_number == Some(middle.part_number))
                .map(|node| node.part_number)
                .collect::<Vec<_>>();
            if children.is_empty() {
                continue;
            }
            let parent_boundary = (0..groups.len())
                .filter(|group| group_labels[*group] == middle.part_number)
                .filter(|group| {
                    adjacency[*group]
                        .iter()
                        .any(|(neighbor, _)| group_labels[*neighbor] == parent)
                })
                .collect::<Vec<_>>();
            if parent_boundary.is_empty() {
                continue;
            }
            for child in children {
                let minimum_work = carrier_label_minimum_gradient_work_v2();
                let support = BTreeSet::from([parent, middle.part_number, child]);
                let mut distances = vec![f32::INFINITY; groups.len()];
                let mut queue = BinaryHeap::<GeodesicBoundaryQueueEntryV1>::new();
                for &seed in &parent_boundary {
                    distances[seed] = 0.0;
                    queue.push(GeodesicBoundaryQueueEntryV1 {
                        distance: 0.0,
                        group: seed,
                    });
                }
                while let Some(entry) = queue.pop() {
                    if entry.distance > distances[entry.group] || entry.distance >= minimum_work {
                        continue;
                    }
                    for &(neighbor, edge_length) in &adjacency[entry.group] {
                        if ![middle.part_number, child].contains(&group_labels[neighbor]) {
                            continue;
                        }
                        let edge_work = geometry_scaled_weight_delta_limit_v1(
                            edge_length,
                            diagonal,
                            &support,
                            contract,
                            fitted_worlds,
                        )?;
                        let candidate = entry.distance + edge_work;
                        if candidate < distances[neighbor] && candidate < minimum_work {
                            distances[neighbor] = candidate;
                            queue.push(GeodesicBoundaryQueueEntryV1 {
                                distance: candidate,
                                group: neighbor,
                            });
                        }
                    }
                }
                for (group, distance) in distances.into_iter().enumerate() {
                    if group_labels[group] == child && distance.is_finite() {
                        relabel.insert(group, middle.part_number);
                    }
                }
            }
        }
        if relabel.is_empty() {
            break;
        }
        let mut changed = false;
        for (group, replacement) in relabel {
            for &vertex in &groups[group] {
                changed |= labels[vertex] != replacement;
                labels[vertex] = replacement;
                changed_vertices.insert(vertex);
            }
        }
        if !changed {
            break;
        }
    }
    Ok(changed_vertices.len())
}

fn repair_nonlocal_label_transitions_after_thickening_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &mut [u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    let mut changed_vertices = BTreeSet::<usize>::new();
    for _ in 0..contract.nodes.len().max(1) {
        let groups =
            duplicate_position_label_topology_groups_v1(positions, indices, labels, contract)?;
        let mut group_by_vertex = vec![usize::MAX; positions.len()];
        for (group_index, group) in groups.iter().enumerate() {
            for &vertex in group {
                group_by_vertex[vertex] = group_index;
            }
        }
        let mut replacements = BTreeMap::<usize, u32>::new();
        for triangle in indices.chunks_exact(3) {
            let vertices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            let support = vertices
                .iter()
                .map(|vertex| labels[*vertex])
                .collect::<BTreeSet<_>>();
            if carrier_support_is_single_lineage(
                &support,
                contract,
                MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
            ) {
                continue;
            }
            for vertex in vertices {
                let label = labels[vertex];
                let needs_parent_step = support.iter().any(|other| {
                    carrier_is_ancestor_or_self(*other, label, contract)
                        && carrier_topology_distance(*other, label, contract)
                            > MAX_LOCAL_LABEL_TRANSITION_DISTANCE
                });
                if needs_parent_step {
                    if let Some(parent) = contract.nodes[label as usize].parent_part_number {
                        replacements.insert(group_by_vertex[vertex], parent);
                    }
                }
            }
        }
        if replacements.is_empty() {
            break;
        }
        for (group, replacement) in replacements {
            for &vertex in &groups[group] {
                labels[vertex] = replacement;
                changed_vertices.insert(vertex);
            }
        }
    }
    let unresolved = indices
        .chunks_exact(3)
        .filter(|triangle| {
            let support = triangle
                .iter()
                .map(|vertex| labels[*vertex as usize])
                .collect::<BTreeSet<_>>();
            !carrier_support_is_single_lineage(
                &support,
                contract,
                MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
            )
        })
        .count();
    if unresolved > 0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-LABEL-THICKENING-TOPOLOGY",
            "skinning.labels",
            format!(
                "{unresolved} triangles remain nonlocal after deterministic chain-label repair"
            ),
        ));
    }
    Ok(changed_vertices.len())
}

fn advance_lineage_row_toward_carrier_v1(
    source: &[RigWeightInfluenceV1],
    target: u32,
    mut progress: f32,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<Vec<RigWeightInfluenceV1>, ReferenceSupermodelGenericErrorV2> {
    let mut accumulated = source
        .iter()
        .filter(|influence| influence.value > 0.0)
        .map(|influence| (influence.bone_node_id, influence.value))
        .collect::<BTreeMap<_, _>>();
    let support = accumulated
        .keys()
        .copied()
        .chain(std::iter::once(target))
        .collect::<BTreeSet<_>>();
    if !carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN) {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-SOURCE-LINEAGE",
            "skinning.weights",
            format!("boundary source {support:?} is not one local carrier lineage"),
        ));
    }

    while progress > 1.0e-7 {
        let Some(oldest) = accumulated
            .keys()
            .copied()
            .filter(|bone| carrier_is_ancestor_or_self(*bone, target, contract))
            .max_by_key(|bone| carrier_topology_distance(*bone, target, contract))
        else {
            break;
        };
        if carrier_topology_distance(oldest, target, contract) <= MAX_LOCAL_VERTEX_INFLUENCE_SPAN {
            break;
        }
        let mut next = target;
        while contract.nodes[next as usize].parent_part_number != Some(oldest) {
            next = contract.nodes[next as usize]
                .parent_part_number
                .ok_or_else(|| {
                    error(
                        "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-LINEAGE-PATH",
                        "skinning.weights",
                        format!("carrier {oldest} is not an ancestor of target {target}"),
                    )
                })?;
        }
        let available = accumulated.get(&oldest).copied().unwrap_or(0.0);
        let transfer = available.min(progress);
        if transfer <= 0.0 {
            break;
        }
        if let Some(value) = accumulated.get_mut(&oldest) {
            *value -= transfer;
        }
        *accumulated.entry(next).or_default() += transfer;
        accumulated.retain(|_, value| *value > 1.0e-7);
        progress -= transfer;
    }

    if progress > 1.0e-7 {
        let alpha = progress.min(1.0);
        for value in accumulated.values_mut() {
            *value *= 1.0 - alpha;
        }
        *accumulated.entry(target).or_default() += alpha;
        accumulated.retain(|_, value| *value > 1.0e-7);
    }
    normalize_top_weights(accumulated, 4, 0.0)
}

/// Builds a standard fitted-segment partition inside each semantic branch.
/// A carrier label owns exactly its incoming `parent -> carrier` segment; it
/// must never choose between the incoming and outgoing segments by Euclidean
/// proximity.  Such a nearest-segment Voronoi switch is discontinuous even
/// inside one semantic label (for example `ribcage-neck` versus `neck-head`)
/// and produces a renderer-visible 100%-weight cliff.  Projection onto the
/// one canonical incoming segment supplies `(1-t, t)` directly, while the
/// child label starts the next segment at the shared carrier.
fn seed_local_lineage_multisource_geodesic_field_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<GeodesicSeedStatsV1, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != labels.len() || positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-MULTISOURCE-LAYOUT",
            "skinning.weights",
            "multi-source geodesic seeding requires one label and weight row per source vertex",
        ));
    }
    if fitted_worlds.len() != contract.nodes.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-MULTISOURCE-FITTED-WORLDS",
            "jointFit.fittedWorldPositions",
            "multi-source geodesic seeding requires one fitted world position per exact carrier",
        ));
    }
    let diagonal = model_diagonal(positions).max(1.0e-6);
    // The initializer owns the new field, so grouping it by stale incoming
    // weights makes UV duplicates choose different boundary sources and forces
    // the later weld to rewrite an already validated result.  Semantic label
    // plus local topology is the stable identity for this construction; true
    // sibling seams remain separate through that topology key.
    let groups = duplicate_position_label_topology_groups_v1(positions, indices, labels, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    let mut group_labels = Vec::<u32>::with_capacity(groups.len());
    for (group_index, group) in groups.iter().enumerate() {
        let mut counts = BTreeMap::<u32, usize>::new();
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
            *counts.entry(labels[vertex]).or_default() += 1;
        }
        group_labels.push(
            counts
                .into_iter()
                .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
                .map(|(label, _)| label)
                .expect("duplicate-position groups are non-empty"),
        );
    }
    let mut edge_set = BTreeSet::<(usize, usize)>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "multi-source geodesic seeding encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = [
                group_by_vertex[vertices[left]],
                group_by_vertex[vertices[right]],
            ];
            if pair[0] != pair[1] {
                edge_set.insert(if pair[0] < pair[1] {
                    (pair[0], pair[1])
                } else {
                    (pair[1], pair[0])
                });
            }
        }
    }
    let mut adjacency = vec![Vec::<(usize, f32)>::new(); groups.len()];
    for (left, right) in edge_set {
        let length = squared_distance(positions[groups[left][0]], positions[groups[right][0]])
            .sqrt()
            .max(1.0e-9);
        adjacency[left].push((right, length));
        adjacency[right].push((left, length));
    }
    for neighbors in &mut adjacency {
        neighbors.sort_by_key(|(group, _)| *group);
    }

    let mut labels_present = group_labels.iter().copied().collect::<Vec<_>>();
    labels_present.sort_unstable();
    labels_present.dedup();
    labels_present.sort_by_key(|carrier| (carrier_depth(*carrier, contract), *carrier));
    let label_set = labels_present.iter().copied().collect::<BTreeSet<_>>();
    let mut changed_groups = 0usize;
    let mut skin_region_feasibility = Vec::new();
    for own in labels_present {
        let own_groups = group_labels
            .iter()
            .enumerate()
            .filter_map(|(group, label)| (*label == own).then_some(group))
            .collect::<Vec<_>>();
        // A semantic carrier can own several disconnected surface islands
        // (for example mirrored limbs or separate fur cards).  Boundary rows
        // and child-seam cones are only continuous inside one such island;
        // averaging them across the shared label creates an artificial weight
        // jump between unrelated pieces of geometry.
        let mut component_by_group = vec![usize::MAX; groups.len()];
        let mut component_count = 0usize;
        for &start in &own_groups {
            if component_by_group[start] != usize::MAX {
                continue;
            }
            component_by_group[start] = component_count;
            let mut pending = vec![start];
            while let Some(group) = pending.pop() {
                for &(neighbor, _) in &adjacency[group] {
                    if group_labels[neighbor] == own && component_by_group[neighbor] == usize::MAX {
                        component_by_group[neighbor] = component_count;
                        pending.push(neighbor);
                    }
                }
            }
            component_count += 1;
        }
        // Some reference hierarchies contain technical transform nodes which
        // intentionally are not deformation carriers.  Never emit such a
        // parent into a skin row merely because it is present in the skeleton.
        let parent = contract.nodes[own as usize]
            .parent_part_number
            .filter(|parent| label_set.contains(parent));
        let Some(parent) = parent else {
            for group_index in own_groups {
                let replacement = vec![RigWeightInfluenceV1 {
                    bone_node_id: own,
                    value: 1.0,
                }];
                changed_groups += usize::from(weights[groups[group_index][0]] != replacement);
                for &vertex in &groups[group_index] {
                    weights[vertex] = replacement.clone();
                }
            }
            skin_region_feasibility.push(ReferenceSupermodelSkinRegionFeasibilityV1 {
                part_number: own,
                joint_name: contract.nodes[own as usize].name.clone(),
                parent_boundary_group_count: 0,
                available_geodesic_depth_fraction: 0.0,
                required_blend_width_fraction: 0.0,
                available_to_required_ratio: 1.0,
                minimum_child_boundary_depth_fraction: None,
                dominant_core_feasible: true,
                full_core_feasible: true,
            });
            continue;
        };
        let boundary = own_groups
            .iter()
            .copied()
            .filter(|group| {
                adjacency[*group]
                    .iter()
                    .any(|(neighbor, _)| group_labels[*neighbor] == parent)
            })
            .collect::<Vec<_>>();
        let mut boundary_rows = BTreeMap::<usize, Vec<RigWeightInfluenceV1>>::new();
        for &seed in &boundary {
            let parent_neighbors = adjacency[seed]
                .iter()
                .filter(|(neighbor, _)| group_labels[*neighbor] == parent)
                .map(|(neighbor, _)| *neighbor)
                .collect::<Vec<_>>();
            let source_row = if parent_neighbors.is_empty() {
                vec![RigWeightInfluenceV1 {
                    bone_node_id: parent,
                    value: 1.0,
                }]
            } else {
                let parent_share = 1.0 / parent_neighbors.len() as f32;
                let mut accumulated = BTreeMap::<u32, f32>::new();
                for neighbor in parent_neighbors {
                    for influence in &weights[groups[neighbor][0]] {
                        *accumulated.entry(influence.bone_node_id).or_default() +=
                            influence.value * parent_share;
                    }
                }
                normalize_top_weights(accumulated, 4, 0.0)?
            };
            boundary_rows.insert(seed, source_row);
        }
        let mut boundary_seeds_by_component = BTreeMap::<usize, Vec<usize>>::new();
        for &seed in &boundary {
            boundary_seeds_by_component
                .entry(component_by_group[seed])
                .or_default()
                .push(seed);
        }
        for seeds in boundary_seeds_by_component.values() {
            let share = 1.0 / seeds.len() as f32;
            let mut accumulated = BTreeMap::<u32, f32>::new();
            for seed in seeds {
                for influence in &boundary_rows[seed] {
                    *accumulated.entry(influence.bone_node_id).or_default() +=
                        influence.value * share;
                }
            }
            let component_source = normalize_top_weights(accumulated, 4, 0.0)?;
            for seed in seeds {
                boundary_rows.insert(*seed, component_source.clone());
            }
        }
        let support = boundary_rows
            .values()
            .flat_map(|row| row.iter().map(|influence| influence.bone_node_id))
            .chain(std::iter::once(own))
            .collect::<BTreeSet<_>>();
        if !carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN)
        {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-BOUNDARY-LINEAGE",
                "skinning.weights",
                format!(
                    "parent-child region {parent}->{own} received nonlocal boundary support {support:?}"
                ),
            ));
        }
        let required_blend_width =
            geometry_scaled_weight_blend_width_v1(diagonal, &support, contract, fitted_worlds)?;
        let child_labels = contract
            .nodes
            .iter()
            .filter(|node| node.parent_part_number == Some(own))
            .filter(|node| label_set.contains(&node.part_number))
            .map(|node| node.part_number)
            .collect::<BTreeSet<_>>();
        // Each connected child surface receives one canonical row from this
        // parent region. Collapse all parent-side groups touching that child
        // component into a zero-cost seam before solving the distance field.
        // Quotient-graph distance is Lipschitz on every real renderer edge and
        // exactly constant around every inherited child seam.
        let mut collapsed_child_seams = Vec::<Vec<usize>>::new();
        for child in child_labels.iter().copied() {
            let child_groups = group_labels
                .iter()
                .enumerate()
                .filter_map(|(group, label)| (*label == child).then_some(group))
                .collect::<Vec<_>>();
            let mut visited = BTreeSet::<usize>::new();
            for start in child_groups {
                if !visited.insert(start) {
                    continue;
                }
                let mut pending = vec![start];
                let mut parent_seam = BTreeSet::<usize>::new();
                while let Some(group) = pending.pop() {
                    for &(neighbor, _) in &adjacency[group] {
                        if group_labels[neighbor] == child && visited.insert(neighbor) {
                            pending.push(neighbor);
                        } else if group_labels[neighbor] == own {
                            parent_seam.insert(neighbor);
                        }
                    }
                }
                if !parent_seam.is_empty() {
                    collapsed_child_seams.push(parent_seam.into_iter().collect());
                }
            }
        }
        let mut collapsed_seams_by_group = vec![Vec::<usize>::new(); groups.len()];
        for (seam_index, seam) in collapsed_child_seams.iter().enumerate() {
            for &group in seam {
                collapsed_seams_by_group[group].push(seam_index);
            }
        }
        let mut distances = vec![f32::INFINITY; groups.len()];
        let mut source_seed_by_group = vec![usize::MAX; groups.len()];
        let mut expanded_collapsed_seam = vec![false; collapsed_child_seams.len()];
        let mut queue = BinaryHeap::<GeodesicBoundaryQueueEntryV1>::new();
        for &seed in &boundary {
            distances[seed] = 0.0;
            source_seed_by_group[seed] = seed;
            queue.push(GeodesicBoundaryQueueEntryV1 {
                distance: 0.0,
                group: seed,
            });
        }
        while let Some(entry) = queue.pop() {
            if entry.distance > distances[entry.group] || entry.distance >= 2.0 {
                continue;
            }
            for &seam_index in &collapsed_seams_by_group[entry.group] {
                if expanded_collapsed_seam[seam_index] {
                    continue;
                }
                expanded_collapsed_seam[seam_index] = true;
                let candidate_source = source_seed_by_group[entry.group];
                for &neighbor in &collapsed_child_seams[seam_index] {
                    let improves_distance = entry.distance + 1.0e-7 < distances[neighbor];
                    let resolves_tie = (entry.distance - distances[neighbor]).abs() <= 1.0e-7
                        && candidate_source < source_seed_by_group[neighbor];
                    if improves_distance || resolves_tie {
                        distances[neighbor] = entry.distance;
                        source_seed_by_group[neighbor] = candidate_source;
                        queue.push(GeodesicBoundaryQueueEntryV1 {
                            distance: entry.distance,
                            group: neighbor,
                        });
                    }
                }
            }
            for &(neighbor, edge_length) in &adjacency[entry.group] {
                if group_labels[neighbor] != own {
                    continue;
                }
                let edge_cost = geometry_scaled_weight_delta_limit_v1(
                    edge_length,
                    diagonal,
                    &support,
                    contract,
                    fitted_worlds,
                )?;
                let candidate = entry.distance + edge_cost;
                let candidate_source = source_seed_by_group[entry.group];
                let improves_distance = candidate + 1.0e-7 < distances[neighbor];
                let resolves_tie = (candidate - distances[neighbor]).abs() <= 1.0e-7
                    && candidate_source < source_seed_by_group[neighbor];
                if (improves_distance || resolves_tie) && candidate < 2.0 {
                    distances[neighbor] = candidate;
                    source_seed_by_group[neighbor] = candidate_source;
                    queue.push(GeodesicBoundaryQueueEntryV1 {
                        distance: candidate,
                        group: neighbor,
                    });
                }
            }
        }
        let maximum_transition = own_groups
            .iter()
            .map(|group| distances[*group].min(2.0))
            .fold(0.0_f32, f32::max);
        let available_depth = maximum_transition * required_blend_width;
        let minimum_child_boundary_depth = own_groups
            .iter()
            .filter(|group| {
                adjacency[**group]
                    .iter()
                    .any(|(neighbor, _)| child_labels.contains(&group_labels[*neighbor]))
            })
            .map(|group| distances[*group].min(2.0) * required_blend_width)
            .fold(None::<f32>, |minimum, distance| {
                Some(minimum.map_or(distance, |current| current.min(distance)))
            });
        let ratio = available_depth / required_blend_width.max(1.0e-9);
        let mut maximum_own_weight = 0.0_f32;
        skin_region_feasibility.push(ReferenceSupermodelSkinRegionFeasibilityV1 {
            part_number: own,
            joint_name: contract.nodes[own as usize].name.clone(),
            parent_boundary_group_count: boundary.len(),
            available_geodesic_depth_fraction: available_depth / diagonal,
            required_blend_width_fraction: required_blend_width / diagonal,
            available_to_required_ratio: ratio,
            minimum_child_boundary_depth_fraction: minimum_child_boundary_depth
                .map(|distance| distance / diagonal),
            dominant_core_feasible: maximum_transition > 0.5,
            full_core_feasible: maximum_transition >= 1.0 - 1.0e-6,
        });
        for group_index in own_groups {
            // `advance_lineage_row_toward_carrier_v1` consumes a real
            // L-infinity work budget, not a normalized interpolation alpha.
            // A full four-lane parent row can require up to one additional
            // unit of work to roll its oldest carrier forward before the new
            // child can reach weight 1.0. The Dijkstra field is deliberately
            // explored to 2.0 above for exactly that case. Capping it at 1.0
            // stranded short distal regions without a dominant carrier even
            // when their surface had enough geodesic depth.
            let parameter = distances[group_index].min(2.0);
            let replacement = if !distances[group_index].is_finite() {
                vec![RigWeightInfluenceV1 {
                    bone_node_id: own,
                    value: 1.0,
                }]
            } else {
                let source_seed = source_seed_by_group[group_index];
                let source_row = boundary_rows.get(&source_seed).ok_or_else(|| {
                    error(
                        "M2A-REFERENCE-SUPERMODEL-SKIN-GEODESIC-SOURCE-MISSING",
                        "skinning.weights",
                        format!(
                            "parent-child region {parent}->{own} has no boundary row for geodesic source {source_seed}"
                        ),
                    )
                })?;
                advance_lineage_row_toward_carrier_v1(source_row, own, parameter, contract)?
            };
            maximum_own_weight = maximum_own_weight.max(
                replacement
                    .iter()
                    .find(|influence| influence.bone_node_id == own)
                    .map_or(0.0, |influence| influence.value),
            );
            changed_groups += usize::from(weights[groups[group_index][0]] != replacement);
            for &vertex in &groups[group_index] {
                weights[vertex] = replacement.clone();
            }
        }
        if let Some(region) = skin_region_feasibility.last_mut() {
            region.dominant_core_feasible = maximum_own_weight > 0.5;
            region.full_core_feasible = maximum_own_weight >= 1.0 - 1.0e-6;
        }
    }
    Ok(GeodesicSeedStatsV1 {
        changed_group_count: changed_groups,
        skin_region_feasibility,
    })
}

fn edge_weight_proposal_preserves_local_triangle_support_v1(
    touched_triangles: &BTreeSet<usize>,
    triangle_groups: &[[usize; 3]],
    groups: &[Vec<usize>],
    weights: &[Vec<RigWeightInfluenceV1>],
    left_group: usize,
    right_group: usize,
    candidate_left: &[RigWeightInfluenceV1],
    candidate_right: &[RigWeightInfluenceV1],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    touched_triangles.iter().all(|triangle_index| {
        let support = triangle_groups[*triangle_index]
            .iter()
            .flat_map(|group| {
                let row = if *group == left_group {
                    candidate_left
                } else if *group == right_group {
                    candidate_right
                } else {
                    &weights[groups[*group][0]]
                };
                row.iter()
                    .filter(|influence| influence.value > 0.0)
                    .map(|influence| influence.bone_node_id)
            })
            .collect::<BTreeSet<_>>();
        carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN)
    })
}

/// Projects the weight field onto the renderer-edge continuity bound.  The
/// motion oracle remains the authority for actual animated stretch; this pass
/// only prevents abrupt changes between adjacent renderer vertices while
/// preserving one local carrier lineage and the four-influence MDL limit.

fn relax_geometry_scaled_weight_gradients_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<WeightGradientRelaxationStatsV1, ReferenceSupermodelGenericErrorV2> {
    let diagonal = model_diagonal(positions).max(1.0e-6);
    let groups =
        duplicate_position_weight_topology_groups_v1(positions, indices, weights, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    for (group_index, group) in groups.iter().enumerate() {
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
        }
    }
    let mut edge_set = BTreeSet::<(usize, usize)>::new();
    let mut triangle_groups = Vec::<[usize; 3]>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "weight-gradient relaxation encountered an out-of-range index",
            ));
        }
        triangle_groups.push(vertices.map(|vertex| group_by_vertex[vertex]));
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = [
                group_by_vertex[vertices[left]],
                group_by_vertex[vertices[right]],
            ];
            if pair[0] == pair[1] {
                continue;
            }
            edge_set.insert(if pair[0] < pair[1] {
                (pair[0], pair[1])
            } else {
                (pair[1], pair[0])
            });
        }
    }
    // A duplicate-position group may sit exactly at the junction of two
    // consecutive carrier edges.  Its row must remain inside the intersection
    // of every incident triangle's already-projected support; otherwise
    // smoothing A-B can introduce A into a neighbouring B-C triangle.
    let mut allowed_by_group = vec![None::<BTreeSet<u32>>; groups.len()];
    let mut triangles_by_group = vec![Vec::<usize>::new(); groups.len()];
    for (triangle_index, triangle) in triangle_groups.iter().enumerate() {
        let support = triangle
            .iter()
            .flat_map(|group| {
                weights[groups[*group][0]]
                    .iter()
                    .filter(|influence| influence.value > 0.0)
                    .map(|influence| influence.bone_node_id)
            })
            .collect::<BTreeSet<_>>();
        if !carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN)
        {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-INITIAL-TRIANGLE",
                "skinning.triangles",
                format!("gradient relaxation received an unprojected triangle support {support:?}"),
            ));
        }
        for group in triangle.iter().copied().collect::<BTreeSet<_>>() {
            triangles_by_group[group].push(triangle_index);
            allowed_by_group[group] = Some(match allowed_by_group[group].take() {
                Some(previous) => previous.intersection(&support).copied().collect(),
                None => support.clone(),
            });
        }
    }
    for (group_index, allowed) in allowed_by_group.iter_mut().enumerate() {
        let current = weights[groups[group_index][0]]
            .iter()
            .filter(|influence| influence.value > 0.0)
            .map(|influence| influence.bone_node_id)
            .collect::<BTreeSet<_>>();
        let domain = allowed.get_or_insert_with(|| current.clone());
        if domain.is_empty() || !current.is_subset(domain) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-DOMAIN",
                "skinning.weights",
                format!(
                    "duplicate-position group {group_index} has row support {current:?} outside incident triangle intersection {domain:?}"
                ),
            ));
        }
    }
    // Domain widening below may open one immediate parent-child blend strip,
    // but it must never become transitive across queue revisits.  Keep the
    // incident-triangle intersection captured before relaxation as the fixed
    // authority for deciding which neighbouring carrier can be introduced.
    let base_allowed_by_group = allowed_by_group.clone();
    let edges = edge_set.into_iter().collect::<Vec<_>>();
    let mut incident = vec![Vec::<usize>::new(); groups.len()];
    for (edge_index, (left, right)) in edges.iter().copied().enumerate() {
        incident[left].push(edge_index);
        incident[right].push(edge_index);
    }
    let mut queue = (0..edges.len()).collect::<VecDeque<_>>();
    let mut queued = vec![true; edges.len()];
    let mut relaxed_groups = BTreeSet::<usize>::new();
    let mut processed = 0usize;
    let mut updates = 0usize;
    let maximum_updates = edges.len().saturating_mul(64).max(1_024);
    let mut edge_limit_cache = vec![([u32::MAX; 8], 0_u8, 0.0_f32); edges.len()];
    while let Some(edge_index) = queue.pop_front() {
        queued[edge_index] = false;
        processed += 1;
        let (left_group, right_group) = edges[edge_index];
        let left_vertex = groups[left_group][0];
        let right_vertex = groups[right_group][0];
        let left = weights[left_vertex].clone();
        let right = weights[right_vertex].clone();
        let edge_length = squared_distance(positions[left_vertex], positions[right_vertex]).sqrt();
        let mut support_ids = [u32::MAX; 8];
        let mut support_count = 0usize;
        for influence in left.iter().chain(&right).filter(|row| row.value > 0.0) {
            if support_ids[..support_count]
                .iter()
                .all(|bone| *bone != influence.bone_node_id)
            {
                support_ids[support_count] = influence.bone_node_id;
                support_count += 1;
            }
        }
        support_ids[..support_count].sort_unstable();
        let support = &support_ids[..support_count];
        let cached = &mut edge_limit_cache[edge_index];
        let maximum_delta = if usize::from(cached.1) == support_count
            && cached.0[..support_count] == support[..]
        {
            cached.2
        } else {
            let support_set = support.iter().copied().collect::<BTreeSet<_>>();
            if !carrier_support_is_single_lineage(
                &support_set,
                contract,
                MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
            ) {
                return Err(error(
                    "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-SUPPORT",
                    "skinning.weights",
                    format!(
                        "edge ({left_vertex},{right_vertex}) spans nonlocal carriers {support:?}"
                    ),
                ));
            }
            let limit = geometry_scaled_weight_delta_limit_v1(
                edge_length,
                diagonal,
                &support_set,
                contract,
                fitted_worlds,
            )?;
            cached.0 = support_ids;
            cached.1 = support_count as u8;
            cached.2 = limit;
            limit
        };
        let current_delta = rig_weight_row_maximum_delta_v1(&left, &right);
        if current_delta <= maximum_delta + 1.0e-6 {
            continue;
        }
        // Widen a rigid endpoint to this exact parent-child pair only when
        // the complete endpoint one-ring remains on one carrier edge. This
        // creates a finite blend strip at A-B boundaries without allowing an
        // A-B-C junction to leak into the neighbouring branch.
        for group in [left_group, right_group] {
            let current_domain = allowed_by_group[group].as_ref().unwrap();
            let base_domain = base_allowed_by_group[group].as_ref().unwrap();
            let mut candidate_domain = current_domain.clone();
            candidate_domain.extend(support.iter().copied().filter(|candidate| {
                base_domain
                    .iter()
                    .any(|existing| carrier_transition_is_local(*existing, *candidate, contract, 1))
            }));
            if candidate_domain == *current_domain {
                continue;
            }
            let safe = triangles_by_group[group].iter().all(|triangle_index| {
                let potential = triangle_groups[*triangle_index]
                    .iter()
                    .flat_map(|triangle_group| {
                        weights[groups[*triangle_group][0]]
                            .iter()
                            .filter(|influence| influence.value > 0.0)
                            .map(|influence| influence.bone_node_id)
                    })
                    .chain(candidate_domain.iter().copied())
                    .collect::<BTreeSet<_>>();
                carrier_support_is_single_lineage(
                    &potential,
                    contract,
                    MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
                )
            });
            if safe {
                allowed_by_group[group] = Some(candidate_domain);
            }
        }
        let alpha = ((1.0 - maximum_delta / current_delta) * 0.5).clamp(0.0, 0.5);
        let blend = |own: &[RigWeightInfluenceV1],
                     other: &[RigWeightInfluenceV1],
                     allowed: &BTreeSet<u32>| {
            // Both endpoint rows can already occupy different three-wide
            // windows inside one valid four-carrier lineage.  Truncating their
            // blend back to three drops an outer carrier, increases another
            // component delta, and can make a solvable edge report false
            // no-progress.  Aurora's skin row supports four influences, and
            // the transactional triangle audit below prevents branch leakage.
            blend_weight_rows_with_domain_v1(own, other, alpha, allowed)
        };
        let proposed_left = blend(
            &left,
            &right,
            allowed_by_group[left_group].as_ref().unwrap(),
        )?;
        let proposed_right = blend(
            &right,
            &left,
            allowed_by_group[right_group].as_ref().unwrap(),
        )?;
        // A pair update can be individually valid for both endpoint domains
        // yet combine the two outer carriers of adjacent A-B-C and B-C-D
        // windows in their shared triangle.  Admit the pair only as one
        // transaction.  Otherwise apply the one safe endpoint which reduces
        // the current edge delta the most; the queue will revisit the edge.
        // Once a proposal only changes values of carriers already present in
        // that endpoint row, its incident triangle support can only stay the
        // same or shrink. Avoid rescanning the complete one-ring for those
        // millions of scalar-only convergence steps; perform the expensive
        // topology transaction only when a proposal introduces a carrier.
        let introduces_support =
            |candidate: &[RigWeightInfluenceV1], current: &[RigWeightInfluenceV1]| {
                candidate
                    .iter()
                    .filter(|influence| influence.value > 0.0)
                    .any(|influence| {
                        current
                            .iter()
                            .all(|existing| existing.bone_node_id != influence.bone_node_id)
                    })
            };
        let proposed_left_introduces = introduces_support(&proposed_left, &left);
        let proposed_right_introduces = introduces_support(&proposed_right, &right);
        let proposal_preserves =
            |candidate_left: &[RigWeightInfluenceV1], candidate_right: &[RigWeightInfluenceV1]| {
                let touched_triangles = triangles_by_group[left_group]
                    .iter()
                    .chain(&triangles_by_group[right_group])
                    .copied()
                    .collect::<BTreeSet<_>>();
                edge_weight_proposal_preserves_local_triangle_support_v1(
                    &touched_triangles,
                    &triangle_groups,
                    &groups,
                    weights,
                    left_group,
                    right_group,
                    candidate_left,
                    candidate_right,
                    contract,
                )
            };
        let pair_delta = rig_weight_row_maximum_delta_v1(&proposed_left, &proposed_right);
        let pair_is_valid = pair_delta + 1.0e-7 < current_delta
            && ((!proposed_left_introduces && !proposed_right_introduces)
                || proposal_preserves(&proposed_left, &proposed_right));
        let selected = if pair_is_valid {
            Some((proposed_left, proposed_right))
        } else {
            // The one-sided candidates are rare and only need to be audited
            // when the combined transaction is rejected. Avoiding two extra
            // one-ring scans on the normal pair path keeps the full 300k
            // triangle audit practical without weakening the invariant.
            let left_only_delta = rig_weight_row_maximum_delta_v1(&proposed_left, &right);
            let right_only_delta = rig_weight_row_maximum_delta_v1(&left, &proposed_right);
            let left_only_valid = left_only_delta + 1.0e-7 < current_delta
                && (!proposed_left_introduces || proposal_preserves(&proposed_left, &right));
            let right_only_valid = right_only_delta + 1.0e-7 < current_delta
                && (!proposed_right_introduces || proposal_preserves(&left, &proposed_right));
            match (left_only_valid, right_only_valid) {
                (true, true) if left_only_delta <= right_only_delta => {
                    Some((proposed_left, right.clone()))
                }
                (true, true) => Some((left.clone(), proposed_right)),
                (true, false) => Some((proposed_left, right.clone())),
                (false, true) => Some((left.clone(), proposed_right)),
                (false, false) => None,
            }
        };
        let Some((new_left, new_right)) = selected else {
            let triangle_supports = |group: usize| {
                triangles_by_group[group]
                    .iter()
                    .take(12)
                    .map(|triangle_index| {
                        triangle_groups[*triangle_index]
                            .iter()
                            .flat_map(|triangle_group| {
                                weights[groups[*triangle_group][0]]
                                    .iter()
                                    .filter(|influence| influence.value > 0.0)
                                    .map(|influence| influence.bone_node_id)
                            })
                            .collect::<BTreeSet<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-NO-PROGRESS",
                "skinning.weights",
                format!(
                    "edge ({left_vertex},{right_vertex}) exceeds its geometry-scaled gradient but cannot be adjusted; positions={:?}->{:?}, edgeLength={edge_length:.9}, currentDelta={current_delta:.9}, maximumDelta={maximum_delta:.9}, support={support:?}, leftRow={left:?}, rightRow={right:?}, leftGroupSize={}, rightGroupSize={}, leftDomain={:?}, rightDomain={:?}, leftTriangleSupports={:?}, rightTriangleSupports={:?}",
                    positions[left_vertex],
                    positions[right_vertex],
                    groups[left_group].len(),
                    groups[right_group].len(),
                    allowed_by_group[left_group],
                    allowed_by_group[right_group],
                    triangle_supports(left_group),
                    triangle_supports(right_group),
                ),
            ));
        };
        for &vertex in &groups[left_group] {
            weights[vertex] = new_left.clone();
        }
        for &vertex in &groups[right_group] {
            weights[vertex] = new_right.clone();
        }
        relaxed_groups.insert(left_group);
        relaxed_groups.insert(right_group);
        updates += 1;
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var_os("M2A_TRACE_SKIN_LABELS").is_some() && updates % 250_000 == 0 {
            let current_labels = weights
                .iter()
                .map(|row| row[0].bone_node_id)
                .collect::<Vec<_>>();
            let audit = audit_geometry_scaled_weight_gradients_v2(
                positions,
                indices,
                &current_labels,
                contract,
                fitted_worlds,
                weights,
            )?;
            eprintln!(
                "M2A_GRADIENT_PROGRESS:{}",
                serde_json::json!({"updates":updates,"violations":audit.violation_edge_count,"ratio":audit.maximum_limit_ratio,"worst":audit.worst_edges.first()})
            );
        }
        if updates % 1_000_000 == 0 {
            eprintln!(
                "M2A_SKIN_TIMING:gradient-queue:updates:{updates}:processed:{processed}:queued:{}",
                queue.len()
            );
        }
        if updates > maximum_updates {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-NONCONVERGENT",
                "skinning.weights",
                format!(
                    "geometry-scaled weight relaxation exceeded {maximum_updates} deterministic updates"
                ),
            ));
        }
        for group in [left_group, right_group] {
            for &candidate in &incident[group] {
                if !queued[candidate] {
                    queued[candidate] = true;
                    queue.push_back(candidate);
                }
            }
        }
    }

    let mut maximum_limit_ratio = 0.0_f32;
    let mut violations = 0usize;
    for (left_group, right_group) in edges.iter().copied() {
        let left_vertex = groups[left_group][0];
        let right_vertex = groups[right_group][0];
        let edge_length = squared_distance(positions[left_vertex], positions[right_vertex]).sqrt();
        let support = weights[left_vertex]
            .iter()
            .chain(&weights[right_vertex])
            .filter(|influence| influence.value > 0.0)
            .map(|influence| influence.bone_node_id)
            .collect::<BTreeSet<_>>();
        let maximum_delta = geometry_scaled_weight_delta_limit_v1(
            edge_length,
            diagonal,
            &support,
            contract,
            fitted_worlds,
        )?;
        let delta = rig_weight_row_maximum_delta_v1(&weights[left_vertex], &weights[right_vertex]);
        maximum_limit_ratio = maximum_limit_ratio.max(delta / maximum_delta.max(1.0e-6));
        violations += usize::from(delta > maximum_delta + 2.0e-6);
    }
    if violations > 0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-UNRESOLVED",
            "skinning.weights",
            format!("{violations} renderer edges exceed the geometry-scaled weight-gradient bound"),
        ));
    }
    Ok(WeightGradientRelaxationStatsV1 {
        processed_edge_count: processed,
        update_count: updates,
        relaxed_group_count: relaxed_groups.len(),
        violation_edge_count: 0,
        maximum_limit_ratio,
        worst_edges: Vec::new(),
    })
}

/// Deterministic batched projection of the renderer weight field onto the
/// geometry-scaled edge bound.  The older transactional queue is useful as a
/// small-oracle implementation, but it revisits and revalidates a complete
/// one-ring for every single edge update.  On dense generated surfaces that
/// turns a local continuity repair into millions of allocations and triangle
/// scans.  V3 computes the same symmetric edge projection in Jacobi batches,
/// normalizes every touched group back onto one carrier lineage, and validates
/// the complete triangle field only at bounded phase boundaries.
fn project_geometry_scaled_weight_gradients_v3(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<WeightGradientRelaxationStatsV1, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != labels.len() || positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-PROJECTION-LAYOUT",
            "skinning.weights",
            "gradient projection requires one label and weight row per source vertex",
        ));
    }
    let diagonal = model_diagonal(positions).max(1.0e-6);
    let groups =
        duplicate_position_weight_topology_groups_v1(positions, indices, weights, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    for (group_index, group) in groups.iter().enumerate() {
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
        }
    }
    let mut edge_set = BTreeSet::<(usize, usize)>::new();
    let mut triangle_groups = Vec::<[usize; 3]>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "gradient projection encountered an out-of-range index",
            ));
        }
        triangle_groups.push(vertices.map(|vertex| group_by_vertex[vertex]));
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = [
                group_by_vertex[vertices[left]],
                group_by_vertex[vertices[right]],
            ];
            if pair[0] != pair[1] {
                edge_set.insert(if pair[0] < pair[1] {
                    (pair[0], pair[1])
                } else {
                    (pair[1], pair[0])
                });
            }
        }
    }
    let edges = edge_set.into_iter().collect::<Vec<_>>();
    let mut group_rows = groups
        .iter()
        .map(|group| weights[group[0]].clone())
        .collect::<Vec<_>>();
    // Freeze each duplicate-position group's admissible carriers to the
    // intersection of its incident, already topology-projected triangles.
    // Every later Jacobi proposal is a subset of this domain, so simultaneous
    // edge updates cannot create a new branch or five-carrier triangle and no
    // corrective triangle projection can fight the gradient solver.
    let mut allowed_by_group = vec![None::<BTreeSet<u32>>; groups.len()];
    for triangle in &triangle_groups {
        let support = triangle
            .iter()
            .flat_map(|group| {
                group_rows[*group]
                    .iter()
                    .filter(|influence| influence.value > 0.0)
                    .map(|influence| influence.bone_node_id)
            })
            .collect::<BTreeSet<_>>();
        if !carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN)
        {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-INITIAL-TRIANGLE",
                "skinning.triangles",
                format!(
                    "batched gradient projection received nonlocal triangle support {support:?}"
                ),
            ));
        }
        for group in triangle.iter().copied().collect::<BTreeSet<_>>() {
            allowed_by_group[group] = Some(match allowed_by_group[group].take() {
                Some(previous) => previous.intersection(&support).copied().collect(),
                None => support.clone(),
            });
        }
    }
    for (group_index, allowed) in allowed_by_group.iter_mut().enumerate() {
        let current = group_rows[group_index]
            .iter()
            .filter(|influence| influence.value > 0.0)
            .map(|influence| influence.bone_node_id)
            .collect::<BTreeSet<_>>();
        let domain = allowed.get_or_insert_with(|| current.clone());
        if domain.is_empty() || !current.is_subset(domain) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-DOMAIN",
                "skinning.weights",
                format!(
                    "batched group {group_index} has row support {current:?} outside incident triangle intersection {domain:?}"
                ),
            ));
        }
    }
    let mut relaxed_groups = BTreeSet::<usize>::new();
    let mut update_count = 0usize;
    let mut processed_edge_count = 0usize;

    const MAXIMUM_PHASES: usize = 8;
    const SWEEPS_PER_PHASE: usize = 32;
    for phase in 0..MAXIMUM_PHASES {
        eprintln!("M2A_SKIN_TIMING:gradient:phase:{phase}:start");
        let mut phase_changed = false;
        for _sweep in 0..SWEEPS_PER_PHASE {
            let mut proposals = BTreeMap::<usize, (BTreeMap<u32, f32>, usize)>::new();
            let mut violating_edge_count = 0usize;
            for &(left_group, right_group) in &edges {
                processed_edge_count += 1;
                let left = group_rows[left_group].clone();
                let right = group_rows[right_group].clone();
                let support = left
                    .iter()
                    .chain(&right)
                    .filter(|influence| influence.value > 0.0)
                    .map(|influence| influence.bone_node_id)
                    .collect::<BTreeSet<_>>();
                if !carrier_support_is_single_lineage(
                    &support,
                    contract,
                    MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
                ) {
                    return Err(error(
                        "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-SUPPORT",
                        "skinning.weights",
                        format!(
                            "batched edge ({},{}) spans nonlocal carriers {support:?}",
                            groups[left_group][0], groups[right_group][0]
                        ),
                    ));
                }
                let edge_length = squared_distance(
                    positions[groups[left_group][0]],
                    positions[groups[right_group][0]],
                )
                .sqrt();
                let maximum_delta = geometry_scaled_weight_delta_limit_v1(
                    edge_length,
                    diagonal,
                    &support,
                    contract,
                    fitted_worlds,
                )?;
                let current_delta = rig_weight_row_maximum_delta_v1(&left, &right);
                if current_delta <= maximum_delta + 1.0e-6 {
                    continue;
                }
                violating_edge_count += 1;
                let alpha = ((1.0 - maximum_delta / current_delta) * 0.5).clamp(0.0, 0.5);
                let projected = |own: &[RigWeightInfluenceV1],
                                 other: &[RigWeightInfluenceV1],
                                 allowed: &BTreeSet<u32>|
                 -> Result<
                    Vec<RigWeightInfluenceV1>,
                    ReferenceSupermodelGenericErrorV2,
                > {
                    let mut accumulated = BTreeMap::<u32, f32>::new();
                    for influence in own {
                        *accumulated.entry(influence.bone_node_id).or_default() +=
                            influence.value * (1.0 - alpha);
                    }
                    for influence in other {
                        *accumulated.entry(influence.bone_node_id).or_default() +=
                            influence.value * alpha;
                    }
                    accumulated.retain(|bone, _| allowed.contains(bone));
                    normalize_top_weights(accumulated, 4, 0.0)
                };
                for (group, candidate) in [
                    (
                        left_group,
                        projected(
                            &left,
                            &right,
                            allowed_by_group[left_group].as_ref().unwrap(),
                        )?,
                    ),
                    (
                        right_group,
                        projected(
                            &right,
                            &left,
                            allowed_by_group[right_group].as_ref().unwrap(),
                        )?,
                    ),
                ] {
                    let (accumulated, count) = proposals.entry(group).or_default();
                    for influence in candidate {
                        *accumulated.entry(influence.bone_node_id).or_default() += influence.value;
                    }
                    *count += 1;
                }
            }
            if violating_edge_count == 0 {
                break;
            }
            let mut sweep_changed = false;
            for (group, (mut accumulated, count)) in proposals {
                let inverse = 1.0 / count.max(1) as f32;
                for value in accumulated.values_mut() {
                    *value *= inverse;
                }
                accumulated
                    .retain(|bone, _| allowed_by_group[group].as_ref().unwrap().contains(bone));
                let replacement = normalize_top_weights(accumulated, 4, 0.0)?;
                if replacement != group_rows[group] {
                    group_rows[group] = replacement;
                    relaxed_groups.insert(group);
                    update_count += 1;
                    sweep_changed = true;
                }
            }
            phase_changed |= sweep_changed;
            if !sweep_changed {
                break;
            }
        }

        for (group_index, group) in groups.iter().enumerate() {
            for &vertex in group {
                weights[vertex] = group_rows[group_index].clone();
            }
        }
        let audit = audit_geometry_scaled_weight_gradients_v2(
            positions,
            indices,
            labels,
            contract,
            fitted_worlds,
            weights,
        )?;
        eprintln!(
            "M2A_SKIN_TIMING:gradient:phase:{phase}:violations:{}:updates:{update_count}",
            audit.violation_edge_count
        );
        if audit.violation_edge_count == 0 || !phase_changed {
            return Ok(WeightGradientRelaxationStatsV1 {
                processed_edge_count,
                update_count,
                relaxed_group_count: relaxed_groups.len(),
                ..audit
            });
        }
    }

    let audit = audit_geometry_scaled_weight_gradients_v2(
        positions,
        indices,
        labels,
        contract,
        fitted_worlds,
        weights,
    )?;
    Ok(WeightGradientRelaxationStatsV1 {
        processed_edge_count,
        update_count,
        relaxed_group_count: relaxed_groups.len(),
        ..audit
    })
}

fn audit_geometry_scaled_weight_gradients_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
    weights: &[Vec<RigWeightInfluenceV1>],
) -> Result<WeightGradientRelaxationStatsV1, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != labels.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-LABEL-LAYOUT",
            "skinning.labels",
            "weight-gradient audit requires one effective label per source vertex",
        ));
    }
    let diagonal = model_diagonal(positions).max(1.0e-6);
    let groups =
        duplicate_position_weight_topology_groups_v1(positions, indices, weights, contract)?;
    let mut group_by_vertex = vec![usize::MAX; positions.len()];
    for (group_index, group) in groups.iter().enumerate() {
        for &vertex in group {
            group_by_vertex[vertex] = group_index;
        }
    }
    let mut edges = BTreeSet::<(usize, usize)>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "weight-gradient audit encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            let pair = [
                group_by_vertex[vertices[left]],
                group_by_vertex[vertices[right]],
            ];
            if pair[0] != pair[1] {
                edges.insert(if pair[0] < pair[1] {
                    (pair[0], pair[1])
                } else {
                    (pair[1], pair[0])
                });
            }
        }
    }
    let mut violations = 0usize;
    let mut maximum_limit_ratio = 0.0_f32;
    let mut worst_edges = Vec::<ReferenceSupermodelWeightGradientEdgeV1>::new();
    for (left_group, right_group) in edges.iter().copied() {
        let left_vertex = groups[left_group][0];
        let right_vertex = groups[right_group][0];
        let support = weights[left_vertex]
            .iter()
            .chain(&weights[right_vertex])
            .filter(|influence| influence.value > 0.0)
            .map(|influence| influence.bone_node_id)
            .collect::<BTreeSet<_>>();
        if !carrier_support_is_local_neighborhood_v1(
            &support,
            contract,
            MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
        ) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-SUPPORT",
                "skinning.weights",
                format!("edge ({left_vertex},{right_vertex}) spans nonlocal carriers {support:?}"),
            ));
        }
        let edge_length = squared_distance(positions[left_vertex], positions[right_vertex]).sqrt();
        let maximum_delta = geometry_scaled_weight_delta_limit_v1(
            edge_length,
            diagonal,
            &support,
            contract,
            fitted_worlds,
        )?;
        let delta = rig_weight_row_maximum_delta_v1(&weights[left_vertex], &weights[right_vertex]);
        let ratio = delta / maximum_delta.max(1.0e-6);
        maximum_limit_ratio = maximum_limit_ratio.max(ratio);
        let enters_worst_set = worst_edges.len() < 16
            || worst_edges
                .last()
                .is_some_and(|worst| ratio > worst.limit_ratio);
        if enters_worst_set {
            worst_edges.push(ReferenceSupermodelWeightGradientEdgeV1 {
                left_vertex,
                right_vertex,
                left_position: positions[left_vertex],
                right_position: positions[right_vertex],
                edge_length_fraction: edge_length / diagonal,
                left_label_part_number: labels[left_vertex],
                left_label_name: contract.nodes[labels[left_vertex] as usize].name.clone(),
                right_label_part_number: labels[right_vertex],
                right_label_name: contract.nodes[labels[right_vertex] as usize].name.clone(),
                left_weights: weights[left_vertex].clone(),
                right_weights: weights[right_vertex].clone(),
                maximum_weight_delta: delta,
                permitted_weight_delta: maximum_delta,
                limit_ratio: ratio,
            });
            worst_edges.sort_by(|left, right| {
                right
                    .limit_ratio
                    .partial_cmp(&left.limit_ratio)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| left.left_vertex.cmp(&right.left_vertex))
                    .then_with(|| left.right_vertex.cmp(&right.right_vertex))
            });
            worst_edges.truncate(16);
        }
        if delta > maximum_delta + 2.0e-6 {
            violations += 1;
        }
    }
    Ok(WeightGradientRelaxationStatsV1 {
        processed_edge_count: edges.len(),
        update_count: 0,
        relaxed_group_count: 0,
        violation_edge_count: violations,
        maximum_limit_ratio,
        worst_edges,
    })
}

fn geometry_scaled_weight_delta_limit_v1(
    edge_length: f32,
    diagonal: f32,
    support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
) -> Result<f32, ReferenceSupermodelGenericErrorV2> {
    if !carrier_support_is_local_neighborhood_v1(
        support,
        contract,
        MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
    ) {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-CARRIER-LINEAGE",
            "skinning.weights",
            format!("gradient support {support:?} is not one local carrier lineage"),
        ));
    }
    if !edge_length.is_finite() || edge_length < 0.0 || !diagonal.is_finite() || diagonal <= 0.0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-GEOMETRY",
            "skinning.weights",
            "weight-gradient calibration requires a finite edge length and positive model diagonal",
        ));
    }
    let normalized_limit =
        geometry_scaled_weight_normalized_limit_v1(diagonal, support, contract, fitted_worlds)?;
    // This is a gradient (weight change per unit length), not a fixed change
    // per renderer edge. A 5% cap made the same linear field fail on a coarse
    // triangle and pass after subdivision, and exhausted narrow carrier cores
    // during relaxation. The physical [0,1] cap and inherited-motion gates remain.
    Ok(((edge_length / diagonal) * normalized_limit)
        .min(1.0)
        .max(1.0e-6))
}

fn geometry_scaled_weight_blend_width_v1(
    diagonal: f32,
    support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
) -> Result<f32, ReferenceSupermodelGenericErrorV2> {
    Ok(diagonal
        / geometry_scaled_weight_normalized_limit_v1(diagonal, support, contract, fitted_worlds)?)
}

fn geometry_scaled_weight_normalized_limit_v1(
    diagonal: f32,
    support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
) -> Result<f32, ReferenceSupermodelGenericErrorV2> {
    if fitted_worlds.len() != contract.nodes.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-FITTED-WORLDS",
            "jointFit.fittedWorldPositions",
            "weight-gradient calibration requires one fitted world position per exact carrier",
        ));
    }
    if !diagonal.is_finite() || diagonal <= 0.0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-GEOMETRY",
            "skinning.weights",
            "weight-gradient calibration requires a positive finite model diagonal",
        ));
    }
    if !carrier_support_is_local_neighborhood_v1(
        support,
        contract,
        MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN,
    ) {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-CARRIER-LINEAGE",
            "skinning.weights",
            format!("gradient support {support:?} is not one local carrier lineage"),
        ));
    }
    let support_parts = support.iter().copied().collect::<Vec<_>>();
    let maximum_fitted_span = support_parts
        .iter()
        .enumerate()
        .flat_map(|(left_index, left)| {
            support_parts
                .iter()
                .skip(left_index + 1)
                .filter(|right| carrier_topology_distance(*left, **right, contract) == 1)
                .map(move |right| {
                    squared_distance(
                        fitted_worlds[*left as usize],
                        fitted_worlds[*right as usize],
                    )
                    .sqrt()
                })
        })
        .fold(0.0_f32, f32::max);
    let span_fraction = maximum_fitted_span / diagonal;
    let permitted_extra_stretch = (contract.tolerances.edge_hard_max_ratio - 1.0).max(1.0e-6);
    // Weight transport in the geodesic field is sequential: mass moves across
    // one parent-child carrier edge at a time. Its deformation scale is thus
    // the longest adjacent fitted span in the active window, not the distance
    // between the window's remote endpoints. Using that remote distance made
    // a four-lane row consume entire short terminal regions before their paw
    // or head carrier could own any surface.
    Ok((permitted_extra_stretch / span_fraction.max(0.01))
        .clamp(2.0, MAX_NORMALIZED_RENDER_WEIGHT_SLOPE))
}

/// Finalizes the exact weight rows that will reach the binary writer.
/// Component projection is followed by a topology projection and a
/// geometry-scaled gradient gate; motion quality remains the independent final
/// oracle and no post-admission weight mutation is performed.

/// Diffusion cannot reallocate semantic labels. This initializer is used when
/// strip growth would erase an existing distal region. Independent topology,
/// gradient and motion checks still decide admission.
fn seed_region_preserving_diffusion_v3(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    let adjacency = surface_adjacency(positions, indices)?;
    for (row, label) in weights.iter_mut().zip(labels) {
        *row = vec![RigWeightInfluenceV1 {
            bone_node_id: *label,
            value: 1.0,
        }];
    }
    for _ in 0..20 {
        let previous = weights.to_vec();
        for vertex in 0..positions.len() {
            let own = labels[vertex];
            let mut values = BTreeMap::from([(own, 0.05)]);
            for influence in &previous[vertex] {
                *values.entry(influence.bone_node_id).or_default() += 0.475 * influence.value;
            }
            let neighbors = &adjacency[vertex];
            if neighbors.is_empty() {
                continue;
            }
            for other in neighbors {
                for influence in &previous[*other] {
                    let part = influence.bone_node_id;
                    if carrier_topology_distance(own, part, contract)
                        <= MAX_LOCAL_VERTEX_INFLUENCE_SPAN
                        && (carrier_is_ancestor_or_self(own, part, contract)
                            || carrier_is_ancestor_or_self(part, own, contract))
                    {
                        *values.entry(part).or_default() +=
                            0.475 * influence.value / neighbors.len() as f32;
                    }
                }
            }
            weights[vertex] = normalize_local_branch_weights(values, contract, 4, 0.001)?;
        }
        weld_duplicate_position_label_topology_groups_v1(
            positions, indices, labels, contract, weights,
        )?;
    }
    Ok(())
}

fn finalize_projected_boundary_weights_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_worlds: &[[f32; 3]],
    anatomy: Option<&TargetSurfaceAnatomyArtifactV1>,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<FinalBoundaryRelaxationStatsV2, ReferenceSupermodelGenericErrorV2> {
    eprintln!("M2A_SKIN_TIMING:finalize:start");
    let mut effective_labels = labels.to_vec();
    let mut chain_label_thickening_vertex_count = thicken_middle_carrier_label_regions_v1(
        positions,
        indices,
        &mut effective_labels,
        contract,
        fitted_worlds,
    )?;
    let mut chain_label_topology_repair_vertex_count =
        repair_nonlocal_label_transitions_after_thickening_v1(
            positions,
            indices,
            &mut effective_labels,
            contract,
        )?;
    trace_skin_labels("thickened", &effective_labels, contract);
    let original_regions = labels.iter().copied().collect::<BTreeSet<_>>();
    let resulting_regions = effective_labels.iter().copied().collect::<BTreeSet<_>>();
    let region_loss = !original_regions.is_subset(&resulting_regions);
    let geodesic_seed = if region_loss {
        effective_labels.copy_from_slice(labels);
        chain_label_thickening_vertex_count = 0;
        chain_label_topology_repair_vertex_count = 0;
        seed_region_preserving_diffusion_v3(positions, indices, labels, contract, weights)?;
        GeodesicSeedStatsV1 {
            changed_group_count: positions.len(),
            skin_region_feasibility: Vec::new(),
        }
    } else {
        seed_local_lineage_multisource_geodesic_field_v2(
            positions,
            indices,
            &effective_labels,
            contract,
            fitted_worlds,
            weights,
        )?
    };
    let trace_gradient = |stage: &str,
                          rows: &[Vec<RigWeightInfluenceV1>]|
     -> Result<(), ReferenceSupermodelGenericErrorV2> {
        trace_skin_labels(
            stage,
            &rows.iter().map(|r| r[0].bone_node_id).collect::<Vec<_>>(),
            contract,
        );
        let trace_result = audit_geometry_scaled_weight_gradients_v2(
            positions,
            indices,
            &effective_labels,
            contract,
            fitted_worlds,
            rows,
        );
        let trace = match trace_result {
            Ok(trace) => trace,
            Err(error) => {
                eprintln!("M2A_SKIN_TIMING:finalize:{stage}:pending-topology:{error}");
                return Ok(());
            }
        };
        eprintln!(
            "M2A_SKIN_TIMING:finalize:{stage}:violations:{}:maximum-ratio:{}",
            trace.violation_edge_count, trace.maximum_limit_ratio
        );
        Ok(())
    };
    trace_gradient("after-geodesic", weights)?;
    eprintln!("M2A_SKIN_TIMING:finalize:segment-field-ready");
    let geodesic_boundary_seed_group_count = geodesic_seed.changed_group_count;
    let auxiliary_projection = match anatomy {
        Some(anatomy) => {
            project_auxiliary_surface_components_v3(positions, contract, anatomy, weights)?
        }
        None => AuxiliaryProjectionStatsV3::default(),
    };
    weld_duplicate_position_label_topology_groups_v1(
        positions,
        indices,
        &effective_labels,
        contract,
        weights,
    )?;
    trace_gradient("after-first-weld", weights)?;
    let component_projection = SmallComponentProjectionStatsV1::default();
    trace_gradient("after-component-projection", weights)?;
    let mut projection = project_triangle_support_to_one_carrier_edge_v1(
        positions,
        indices,
        &effective_labels,
        contract,
        weights,
    )?;
    trace_gradient("after-first-triangle-projection", weights)?;
    weld_duplicate_position_label_topology_groups_v1(
        positions,
        indices,
        &effective_labels,
        contract,
        weights,
    )?;
    trace_gradient("after-second-weld", weights)?;
    let post_weld_projection = project_triangle_support_to_one_carrier_edge_v1(
        positions,
        indices,
        &effective_labels,
        contract,
        weights,
    )?;
    trace_gradient("after-second-triangle-projection", weights)?;
    projection.projected_group_count = projection
        .projected_group_count
        .saturating_add(post_weld_projection.projected_group_count);
    projection.projected_vertex_count = projection
        .projected_vertex_count
        .saturating_add(post_weld_projection.projected_vertex_count);
    let pre_gradient = audit_geometry_scaled_weight_gradients_v2(
        positions,
        indices,
        &effective_labels,
        contract,
        fitted_worlds,
        weights,
    )?;
    eprintln!(
        "M2A_SKIN_TIMING:finalize:pre-gradient-violations:{}:maximum-ratio:{}",
        pre_gradient.violation_edge_count, pre_gradient.maximum_limit_ratio
    );
    for edge in &pre_gradient.worst_edges {
        eprintln!("M2A_SKIN_TIMING:finalize:pre-gradient-worst:{edge:?}");
    }
    let gradient_relaxation = relax_geometry_scaled_weight_gradients_v1(
        positions,
        indices,
        contract,
        fitted_worlds,
        weights,
    )?;
    let mut gradient = audit_geometry_scaled_weight_gradients_v2(
        positions,
        indices,
        &effective_labels,
        contract,
        fitted_worlds,
        weights,
    )?;
    gradient.processed_edge_count = gradient_relaxation.processed_edge_count;
    gradient.update_count = gradient_relaxation.update_count;
    gradient.relaxed_group_count = gradient_relaxation.relaxed_group_count;
    trace_gradient("after-gradient", weights)?;
    eprintln!("M2A_SKIN_TIMING:finalize:gradient-ready");
    let remaining_triangles = indices
        .chunks_exact(3)
        .filter(|triangle| {
            !triangle_weight_support_is_local_lineage(
                [
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                ],
                weights,
                contract,
            )
        })
        .count();
    if remaining_triangles > 0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-TRIANGLE-SUPPORT-REGRESSION",
            "skinning.triangles",
            format!(
                "gradient relaxation reopened {remaining_triangles} nonlocal triangle supports"
            ),
        ));
    }
    let final_groups = duplicate_position_label_topology_groups_v1(
        positions,
        indices,
        &effective_labels,
        contract,
    )?;
    let mut shared_carrier_group_count = 0usize;
    let mut rigid_group_count = 0usize;
    for group in final_groups {
        if weights[group[0]].len() > 1 {
            shared_carrier_group_count += 1;
        } else {
            rigid_group_count += 1;
        }
    }
    Ok(FinalBoundaryRelaxationStatsV2 {
        chain_label_thickening_vertex_count,
        chain_label_topology_repair_vertex_count,
        geodesic_boundary_seed_group_count,
        auxiliary_projection,
        component_projection,
        shared_carrier_group_count,
        rigid_group_count,
        projection,
        gradient,
        skin_region_feasibility: geodesic_seed.skin_region_feasibility,
    })
}

#[cfg(test)]
fn local_edge_weight_cliff_count_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &[Vec<RigWeightInfluenceV1>],
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    if positions.len() != weights.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-EDGE-CLIFF-LAYOUT",
            "skinning.weights",
            "edge-cliff audit requires one weight row per source vertex",
        ));
    }
    let mut edges = BTreeSet::<(usize, usize)>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "edge-cliff audit encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            edges.insert(if vertices[left] < vertices[right] {
                (vertices[left], vertices[right])
            } else {
                (vertices[right], vertices[left])
            });
        }
    }
    Ok(edges
        .into_iter()
        .filter(|(left_vertex, right_vertex)| {
            let left = &weights[*left_vertex];
            let right = &weights[*right_vertex];
            local_edge_weight_is_cliff_v1(left, right, contract)
        })
        .count())
}

#[cfg(test)]
#[allow(dead_code)]
fn local_edge_weight_cliff_examples_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &[Vec<RigWeightInfluenceV1>],
    maximum: usize,
) -> Result<Vec<String>, ReferenceSupermodelGenericErrorV2> {
    let mut edges = BTreeSet::<(usize, usize)>::new();
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "edge-cliff example audit encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            edges.insert(if vertices[left] < vertices[right] {
                (vertices[left], vertices[right])
            } else {
                (vertices[right], vertices[left])
            });
        }
    }
    Ok(edges
        .into_iter()
        .filter(|(left, right)| {
            local_edge_weight_is_cliff_v1(&weights[*left], &weights[*right], contract)
        })
        .take(maximum)
        .map(|(left, right)| {
            let row = |vertex: usize| {
                weights[vertex]
                    .iter()
                    .map(|influence| {
                        format!(
                            "{}:{:.6}",
                            contract.nodes[influence.bone_node_id as usize].name, influence.value
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("+")
            };
            format!(
                "{left}@{:?}[{}]--{right}@{:?}[{}] delta={:.6}",
                positions[left],
                row(left),
                positions[right],
                row(right),
                rig_weight_row_maximum_delta_v1(&weights[left], &weights[right]),
            )
        })
        .collect())
}

#[cfg(test)]
fn local_edge_weight_is_cliff_v1(
    left: &[RigWeightInfluenceV1],
    right: &[RigWeightInfluenceV1],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    const MAXIMUM_RETAINED_EDGE_WEIGHT_DELTA: f32 = 0.1;
    let support = left
        .iter()
        .chain(right)
        .filter(|influence| influence.value.is_finite() && influence.value > 0.0)
        .map(|influence| influence.bone_node_id)
        .collect::<BTreeSet<_>>();
    let one_carrier_edge = support.iter().all(|left| {
        support.iter().all(|right| {
            carrier_transition_is_local(
                *left,
                *right,
                contract,
                MAX_LOCAL_INFLUENCE_TO_NEIGHBOR_LABEL_DISTANCE,
            )
        })
    });
    one_carrier_edge
        && rig_weight_row_maximum_delta_v1(left, right)
            > MAXIMUM_RETAINED_EDGE_WEIGHT_DELTA + 1.0e-6
}

fn triangle_weight_support_is_local(
    vertices: [usize; 3],
    weights: &[Vec<RigWeightInfluenceV1>],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    let support = vertices
        .iter()
        .flat_map(|vertex| {
            weights[*vertex]
                .iter()
                .filter(|influence| influence.value > 0.0)
                .map(|influence| influence.bone_node_id)
        })
        .collect::<BTreeSet<_>>();
    carrier_support_is_single_lineage(&support, contract, MAX_LOCAL_TRIANGLE_INFLUENCE_SPAN)
}

// A joint fork is a valid LBS neighborhood. Single ancestry is a solver
// parametrization, not a requirement of Aurora skinning. Keep hop bounds,
// per-vertex lane limits, side-leak checks and the independent motion oracle.
fn carrier_support_is_local_neighborhood_v1(
    support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
    maximum_span: usize,
) -> bool {
    !support.is_empty() && support.iter().all(|left| support.iter().all(|right| {
        // Passive transform-only intermediates do not add a deformation
        // region. Inserting such a helper must not invalidate identical skin.
        let deformation_nodes = carrier_path(*left,*right,contract).into_iter()
            .filter(|part| part == left || part == right || contract.nodes[*part as usize].carrier_class
                == crate::reference_supermodel_motion::ReferenceSupermodelCarrierClassV3::SkinRelevant)
            .count();
        deformation_nodes.saturating_sub(1) <= maximum_span
    }))
}

fn carrier_support_is_single_lineage(
    support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
    maximum_span: usize,
) -> bool {
    support.iter().all(|left| {
        support.iter().all(|right| {
            carrier_topology_distance(*left, *right, contract) <= maximum_span
                && (carrier_is_ancestor_or_self(*left, *right, contract)
                    || carrier_is_ancestor_or_self(*right, *left, contract))
        })
    })
}

fn carrier_depth(mut part: u32, contract: &ReferenceSupermodelMotionContractV2) -> usize {
    let mut depth = 0usize;
    while let Some(parent) = contract.nodes[part as usize].parent_part_number {
        part = parent;
        depth += 1;
    }
    depth
}

fn is_cross_side_influence(
    position: [f32; 3],
    bone: u32,
    fitted_worlds: &[[f32; 3]],
    diagonal: f32,
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    let dead_zone = diagonal * 0.015;
    let Some((symmetry_plane_x, bone_side)) =
        inherited_symmetric_branch_side_v2(bone, fitted_worlds, contract, dead_zone)
    else {
        // A displaced joint is not automatically a left/right joint.  Tail,
        // head and other unpaired branches may legitimately cross the target
        // symmetry plane.  Side barriers are enabled only by exact topology.
        return false;
    };
    let vertex_side = position[0] - symmetry_plane_x;
    vertex_side.abs() > dead_zone && vertex_side * bone_side < 0.0
}

fn inherited_symmetric_branch_side_v2(
    bone: u32,
    fitted_worlds: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    dead_zone: f32,
) -> Option<(f32, f32)> {
    let mut current = bone;
    while let Some(parent) = contract.nodes[current as usize].parent_part_number {
        let plane_x = fitted_worlds[parent as usize][0];
        let current_side = fitted_worlds[current as usize][0] - plane_x;
        if current_side.abs() > dead_zone {
            let role = &contract.nodes[current as usize].structural_role;
            let has_opposite_sibling = contract.nodes.iter().any(|candidate| {
                candidate.part_number != current
                    && candidate.parent_part_number == Some(parent)
                    && candidate.structural_role == *role
                    && {
                        let candidate_side =
                            fitted_worlds[candidate.part_number as usize][0] - plane_x;
                        candidate_side.abs() > dead_zone && candidate_side * current_side < 0.0
                    }
            });
            if has_opposite_sibling {
                return Some((plane_x, current_side));
            }
        }
        current = parent;
    }
    None
}

fn cross_branch_triangle_count_from_labels(
    _positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<usize, ReferenceSupermodelGenericErrorV2> {
    let mut count = 0usize;
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= labels.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "triangle continuity audit encountered an out-of-range vertex",
            ));
        }
        if !triangle_labels_are_local(vertices, labels, contract) {
            count += 1;
        }
    }
    Ok(count)
}

fn triangle_labels_are_local(
    vertices: [usize; 3],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    [(0, 1), (1, 2), (2, 0)].iter().all(|(left, right)| {
        carrier_transition_is_local(
            labels[vertices[*left]],
            labels[vertices[*right]],
            contract,
            MAX_LOCAL_LABEL_TRANSITION_DISTANCE,
        )
    })
}

fn carrier_transition_is_local(
    left: u32,
    right: u32,
    contract: &ReferenceSupermodelMotionContractV2,
    maximum_distance: usize,
) -> bool {
    carrier_topology_distance(left, right, contract) <= maximum_distance
        && (carrier_is_ancestor_or_self(left, right, contract)
            || carrier_is_ancestor_or_self(right, left, contract))
}

fn carrier_is_ancestor_or_self(
    ancestor: u32,
    mut descendant: u32,
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    loop {
        if ancestor == descendant {
            return true;
        }
        let Some(parent) = contract.nodes[descendant as usize].parent_part_number else {
            return false;
        };
        descendant = parent;
    }
}

/// Selects one coherent carrier neighbourhood before normalization.
///
/// A UV seam can contain coincident vertices whose independently sampled rows
/// lie on opposite sides of an anatomical branch junction.  Averaging the raw
/// rows would retain both sibling branches and create a deformation bridge.
/// The accumulated row is therefore projected onto the strongest contiguous
/// ancestor/descendant window. This preserves a legal three-carrier chain at
/// a short joint while excluding every sibling combination; filtering only by
/// distance from one anchor incorrectly removed the third chain carrier and
/// could still retain multiple children of a parent anchor.
fn normalize_local_branch_weights(
    accumulated: BTreeMap<u32, f32>,
    contract: &ReferenceSupermodelMotionContractV2,
    maximum: usize,
    relative_floor: f32,
) -> Result<Vec<RigWeightInfluenceV1>, ReferenceSupermodelGenericErrorV2> {
    let positive = accumulated
        .iter()
        .filter(|(_, value)| value.is_finite() && **value > 0.0)
        .map(|(bone, value)| (*bone, *value))
        .collect::<BTreeMap<_, _>>();
    if positive.is_empty() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-INVALID",
            "skinning.weights",
            "local branch normalization received no positive influence",
        ));
    }
    let mut best = Vec::<(u32, f32)>::new();
    let mut best_total = f32::NEG_INFINITY;
    for &top in positive.keys() {
        for &bottom in positive.keys() {
            if !carrier_is_ancestor_or_self(top, bottom, contract)
                || carrier_topology_distance(top, bottom, contract)
                    > MAX_LOCAL_VERTEX_INFLUENCE_SPAN
            {
                continue;
            }
            let window = positive
                .iter()
                .filter(|(candidate, _)| {
                    carrier_is_ancestor_or_self(top, **candidate, contract)
                        && carrier_is_ancestor_or_self(**candidate, bottom, contract)
                })
                .map(|(bone, value)| (*bone, *value))
                .collect::<Vec<_>>();
            let total = window.iter().map(|(_, value)| *value).sum::<f32>();
            let replace = total > best_total + 1.0e-7
                || ((total - best_total).abs() <= 1.0e-7
                    && window
                        .iter()
                        .map(|(bone, _)| *bone)
                        .cmp(best.iter().map(|(bone, _)| *bone))
                        == Ordering::Less);
            if replace {
                best = window;
                best_total = total;
            }
        }
    }
    normalize_top_weights(best.into_iter().collect(), maximum, relative_floor)
}

#[cfg(test)]
fn weld_duplicate_positions(
    positions: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    let mut groups = BTreeMap::<[u32; 3], Vec<usize>>::new();
    for (vertex, position) in positions.iter().enumerate() {
        groups
            .entry(position.map(f32::to_bits))
            .or_default()
            .push(vertex);
    }
    for group in groups.values().filter(|group| group.len() > 1) {
        let mut accumulated = BTreeMap::<u32, f32>::new();
        for &vertex in group {
            for influence in &weights[vertex] {
                *accumulated.entry(influence.bone_node_id).or_default() += influence.value;
            }
        }
        let welded = normalize_local_branch_weights(accumulated, contract, 4, 0.04)?;
        for &vertex in group {
            weights[vertex] = welded.clone();
        }
    }
    Ok(())
}

fn weld_duplicate_position_label_topology_groups_v1(
    positions: &[[f32; 3]],
    indices: &[u32],
    labels: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    if weights.len() != positions.len() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-SEAM-WEIGHT-LAYOUT",
            "skinning.weights",
            "label-topology seam welding requires one weight row per source vertex",
        ));
    }
    let groups = duplicate_position_label_topology_groups_v1(positions, indices, labels, contract)?;
    for group in groups.iter().filter(|group| group.len() > 1) {
        let mut accumulated = BTreeMap::<u32, f32>::new();
        for &vertex in group {
            for influence in &weights[vertex] {
                *accumulated.entry(influence.bone_node_id).or_default() += influence.value;
            }
        }
        let welded = normalize_local_branch_weights(accumulated, contract, 4, 0.0)?;
        for &vertex in group {
            weights[vertex] = welded.clone();
        }
    }
    Ok(())
}

fn surface_adjacency(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Result<Vec<Vec<usize>>, ReferenceSupermodelGenericErrorV2> {
    let mut adjacency = vec![Vec::new(); positions.len()];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= positions.len()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "surface adjacency encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            adjacency[vertices[left]].push(vertices[right]);
            adjacency[vertices[right]].push(vertices[left]);
        }
    }
    let mut representative = BTreeMap::<[u32; 3], usize>::new();
    for (vertex, position) in positions.iter().enumerate() {
        let key = position.map(f32::to_bits);
        if let Some(&first) = representative.get(&key) {
            adjacency[first].push(vertex);
            adjacency[vertex].push(first);
        } else {
            representative.insert(key, vertex);
        }
    }
    for row in &mut adjacency {
        row.sort_unstable();
        row.dedup();
    }
    Ok(adjacency)
}

fn carrier_topology_distance(
    mut left: u32,
    mut right: u32,
    contract: &ReferenceSupermodelMotionContractV2,
) -> usize {
    // This is one of the hottest primitives in label regularization and
    // branch repair.  The previous implementation allocated two BTreeMaps on
    // every call; a 300k-triangle model invokes it millions of times.  Align
    // both nodes by depth and walk to their LCA with no allocation.
    let mut left_depth = carrier_depth(left, contract);
    let mut right_depth = carrier_depth(right, contract);
    let mut distance = 0usize;
    while left_depth > right_depth {
        let Some(parent) = contract.nodes[left as usize].parent_part_number else {
            return contract.nodes.len();
        };
        left = parent;
        left_depth -= 1;
        distance += 1;
    }
    while right_depth > left_depth {
        let Some(parent) = contract.nodes[right as usize].parent_part_number else {
            return contract.nodes.len();
        };
        right = parent;
        right_depth -= 1;
        distance += 1;
    }
    while left != right {
        let Some(left_parent) = contract.nodes[left as usize].parent_part_number else {
            return contract.nodes.len();
        };
        let Some(right_parent) = contract.nodes[right as usize].parent_part_number else {
            return contract.nodes.len();
        };
        left = left_parent;
        right = right_parent;
        distance += 2;
    }
    distance
}

fn squared_distance_to_segment(point: [f32; 3], start: [f32; 3], end: [f32; 3]) -> f32 {
    let segment = std::array::from_fn::<_, 3, _>(|axis| end[axis] - start[axis]);
    let relative = std::array::from_fn::<_, 3, _>(|axis| point[axis] - start[axis]);
    let length_squared = segment.iter().map(|value| value * value).sum::<f32>();
    let parameter = if length_squared > 1.0e-12 {
        relative
            .iter()
            .zip(segment)
            .map(|(left, right)| left * right)
            .sum::<f32>()
            / length_squared
    } else {
        0.0
    }
    .clamp(0.0, 1.0);
    let closest = std::array::from_fn::<_, 3, _>(|axis| start[axis] + segment[axis] * parameter);
    squared_distance(point, closest)
}

fn squared_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2)
}

fn model_diagonal(positions: &[[f32; 3]]) -> f32 {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    squared_distance(min, max).sqrt()
}

fn duplicate_position_group_count(positions: &[[f32; 3]]) -> usize {
    let mut counts = BTreeMap::<[u32; 3], usize>::new();
    for position in positions {
        *counts.entry(position.map(f32::to_bits)).or_default() += 1;
    }
    counts.values().filter(|count| **count > 1).count()
}

fn validate_inputs(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    indices: &[u32],
    allowed: &[u32],
    fitted: &[[f32; 3]],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    if positions.is_empty()
        || indices.is_empty()
        || indices.len() % 3 != 0
        || fitted.len() != contract.nodes.len()
        || allowed.is_empty()
        || positions.iter().flatten().any(|value| !value.is_finite())
        || fitted.iter().flatten().any(|value| !value.is_finite())
        || indices
            .iter()
            .any(|index| *index as usize >= positions.len())
        || allowed
            .iter()
            .any(|part| *part as usize >= contract.nodes.len())
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-LOCAL-SKINNING-INVALID",
            "skinning.inputs",
            "local skinning requires a finite indexed surface, fitted carrier topology and non-empty allowed bones",
        ));
    }
    Ok(())
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

fn sha256_json<T: Serialize>(value: &T) -> Result<String, ReferenceSupermodelGenericErrorV2> {
    let bytes = serde_json::to_vec(value).map_err(|source| {
        error(
            "M2A-REFERENCE-SUPERMODEL-SKINNING-SERIALIZE",
            "skinning",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference_supermodel_motion::{
        ReferenceSupermodelCarrierClassV3, ReferenceSupermodelMotionNodeV2,
        default_reference_supermodel_motion_tolerances_v2,
    };

    #[test]
    fn authored_validation_recomputes_gradients_instead_of_reusing_old_verdict() {
        let c = contract();
        let p = vec![
            [0., 0., 0.],
            [0., 0.001, 0.],
            [0., 0., 0.001],
            [0., 1., 0.],
            [0., 1.001, 0.],
            [0., 1., 0.001],
        ];
        let indices = vec![0, 1, 2, 3, 4, 5];
        let worlds = vec![[0., 0., 0.]; c.nodes.len()];
        let anatomy =
            crate::reference_supermodel_surface_anatomy::analyze_target_surface_anatomy_v1(
                &p, &indices,
            )
            .unwrap();
        let mut base =
            derive_local_reference_skinning_v1(&c, &p, &indices, &[1], &worlds, &anatomy)
                .unwrap()
                .report;
        base.validation_violations = vec!["WEIGHT_GRADIENT_VIOLATIONS:99:MAX_RATIO:20".into()];
        base.weight_gradient_violation_edge_count = 99;
        let mut rows = vec![
            vec![RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 1.
            }];
            6
        ];
        for row in &mut rows[3..] {
            *row = vec![RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 1.,
            }];
        }
        let fixed = validate_authored_reference_skinning_v2(
            &c,
            &p,
            &indices,
            &[1, 3],
            &worlds,
            &rows,
            &base,
        )
        .unwrap();
        assert_eq!(fixed.status, "READY");
        assert_eq!(fixed.weight_gradient_violation_edge_count, 0);
        rows[1] = vec![RigWeightInfluenceV1 {
            bone_node_id: 3,
            value: 1.,
        }];
        let broken = validate_authored_reference_skinning_v2(
            &c,
            &p,
            &indices,
            &[1, 3],
            &worlds,
            &rows,
            &fixed,
        )
        .unwrap();
        assert_eq!(broken.status, "BLOCKED");
        assert!(broken.weight_gradient_violation_edge_count > 0);
    }

    #[test]
    fn weight_gradient_verdict_is_invariant_under_linear_edge_subdivision() {
        let c = contract();
        let worlds = [[0., 0., 0.], [0., 0., 0.5], [0.2, 0., 0.5], [-0.2, 0., 0.5]];
        let support = BTreeSet::from([0, 1]);
        // Same continuous weight field, represented by one edge or ten edges.
        // The per-unit-length verdict must not depend on tessellation density.
        let coarse = geometry_scaled_weight_delta_limit_v1(0.1, 1., &support, &c, &worlds).unwrap();
        let fine = geometry_scaled_weight_delta_limit_v1(0.01, 1., &support, &c, &worlds).unwrap();
        assert!((0.2 / coarse - 0.02 / fine).abs() < 1.0e-5);
        assert!(0.2 <= coarse);
        assert!(
            1.0 > fine,
            "a rigid carrier switch on a short edge must still fail"
        );
    }

    #[test]
    fn authored_joint_fork_is_local_but_remote_carriers_are_rejected() {
        let c = contract();
        let fork = BTreeSet::from([1, 2, 3]);
        assert!(!carrier_support_is_single_lineage(&fork, &c, 3));
        assert!(carrier_support_is_local_neighborhood_v1(&fork, &c, 3));
        let mut chain = c.clone();
        chain.nodes[2].parent_part_number = Some(1);
        chain.nodes[3].parent_part_number = Some(2);
        assert!(!carrier_support_is_local_neighborhood_v1(
            &BTreeSet::from([0, 3]),
            &chain,
            2
        ));
        let worlds = [[0., 0., 0.], [0., 0., 0.1], [0.1, 0., 0.1], [-0.1, 0., 0.1]];
        assert!(
            geometry_scaled_weight_delta_limit_v1(0.001, 1., &fork, &c, &worlds).unwrap() < 0.1
        );
    }

    #[test]
    fn passive_transform_insertion_does_not_change_deformation_locality() {
        let mut c = contract();
        c.nodes[2].parent_part_number = Some(1);
        c.nodes[3].parent_part_number = Some(2);
        assert!(!carrier_support_is_local_neighborhood_v1(
            &BTreeSet::from([0, 3]),
            &c,
            2
        ));
        c.nodes[1].carrier_class = ReferenceSupermodelCarrierClassV3::PassiveStructural;
        assert!(carrier_support_is_local_neighborhood_v1(
            &BTreeSet::from([0, 3]),
            &c,
            2
        ));
    }

    fn contract() -> ReferenceSupermodelMotionContractV2 {
        let node = |part, parent| ReferenceSupermodelMotionNodeV2 {
            part_number: part,
            name: format!("joint_{part}"),
            parent_part_number: parent,
            carrier_bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            position_controller_required: false,
            orientation_controller_required: part > 0,
            scale_controller_required: false,
            anchor_role: None,
            joint_axis: None,
            carrier_class: if part == 0 {
                ReferenceSupermodelCarrierClassV3::PassiveStructural
            } else {
                ReferenceSupermodelCarrierClassV3::SkinRelevant
            },
            structural_role: if part == 0 { "ROOT" } else { "CHAIN" }.to_owned(),
            controlling_clips: Vec::new(),
            dynamic_clips: Vec::new(),
        };
        ReferenceSupermodelMotionContractV2 {
            schema_version: 2,
            contract_id: "test".to_owned(),
            content_sha256: String::new(),
            supermodel_resref: "c_test".to_owned(),
            source_model_sha256: "0".repeat(64),
            inspected_read_only: true,
            no_payload_copied: true,
            classification: 4,
            animation_scale: 1.0,
            clean_room_profile_id: "test".to_owned(),
            clean_room_profile_sha256: "0".repeat(64),
            nodes: vec![
                node(0, None),
                node(1, Some(0)),
                node(2, Some(0)),
                node(3, Some(1)),
            ],
            carrier_exclusions: Vec::new(),
            required_clips: vec!["move".to_owned()],
            required_events: Vec::new(),
            tolerances: default_reference_supermodel_motion_tolerances_v2(),
        }
    }

    #[test]
    fn gradient_groups_never_alias_different_renderer_weights() {
        let positions = vec![
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., 0., 0.],
            [-1., 0., 0.],
            [0., -1., 0.],
        ];
        let mut weights = vec![
            vec![RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 1.
            }];
            6
        ];
        weights[3] = vec![
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.5,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 0.5,
            },
        ];
        let indices = vec![0, 1, 2, 3, 4, 5];
        let groups = duplicate_position_weight_topology_groups_v1(
            &positions,
            &indices,
            &weights,
            &contract(),
        )
        .unwrap();
        assert!(!groups.iter().any(|g| g.contains(&0) && g.contains(&3)));
        weights[3] = weights[0].clone();
        let groups = duplicate_position_weight_topology_groups_v1(
            &positions,
            &indices,
            &weights,
            &contract(),
        )
        .unwrap();
        assert!(groups.iter().any(|g| g.contains(&0) && g.contains(&3)));
        let reverse = vec![3, 4, 5, 0, 1, 2];
        assert_eq!(
            groups,
            duplicate_position_weight_topology_groups_v1(
                &positions,
                &reverse,
                &weights,
                &contract()
            )
            .unwrap()
        );
    }
    #[test]
    fn excessive_branch_repair_is_fail_closed_unless_explicitly_allowed() {
        let strict = ReferenceSupermodelSkinningOptionsV1::default();
        let error = enforce_branch_boundary_repair_limit_v1(11_931, 8_117, strict)
            .expect_err("the default policy must keep the structural guard active");
        assert_eq!(
            error.code,
            "M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-EXCESSIVE"
        );

        let opt_in = ReferenceSupermodelSkinningOptionsV1 {
            allow_excessive_branch_boundary_repair: true,
            ..Default::default()
        };
        assert!(
            enforce_branch_boundary_repair_limit_v1(11_931, 8_117, opt_in)
                .expect("the explicit experimental opt-in must continue")
        );
        assert!(!enforce_branch_boundary_repair_limit_v1(8_117, 8_117, opt_in).unwrap());
    }

    #[test]
    fn detached_auxiliary_card_projects_from_authoritative_surface() {
        let positions = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [0.0, 0.0, 1.1],
            [0.01, 0.0, 1.1],
            [0.0, 0.01, 1.1],
        ];
        let indices = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7,
            3, 3, 7, 4, 3, 4, 0, 8, 9, 10,
        ];
        let anatomy =
            crate::reference_supermodel_surface_anatomy::analyze_target_surface_anatomy_v1(
                &positions, &indices,
            )
            .unwrap();
        let artifact = derive_local_reference_skinning_v1(
            &contract(),
            &positions,
            &indices,
            &[1],
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.5],
                [0.0, 0.0, 0.75],
                [0.0, 0.0, 1.0],
            ],
            &anatomy,
        )
        .unwrap();

        assert_eq!(artifact.report.auxiliary_component_count, 1);
        assert_eq!(artifact.report.auxiliary_projection_count, 1);
        assert_eq!(artifact.report.auxiliary_projection_vertex_count, 3);
        assert!(artifact.report.auxiliary_projection_coverage);
        assert_eq!(artifact.weights[8], artifact.weights[9]);
        assert_eq!(artifact.weights[9], artifact.weights[10]);
    }

    #[test]
    fn isolated_small_source_component_is_rigidified_before_stream_partitioning() {
        let positions = vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]];
        let mut weights = vec![
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.7,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.3,
                },
            ];
            3
        ];

        let stats = stabilize_small_surface_components_v2(
            &positions,
            &[0, 1, 2],
            &contract(),
            &mut weights,
        )
        .unwrap();

        assert_eq!(stats.component_count, 1);
        assert_eq!(stats.vertex_count, 3);
        assert!(weights.iter().all(|row| {
            row == &vec![RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 1.0,
            }]
        }));
    }

    #[test]
    fn attached_small_components_copy_their_seam_row_only_to_the_interior() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.0, 0.1, 0.0],
            [0.0, 0.0, 0.0],
            [-0.1, 0.0, 0.0],
            [0.0, -0.1, 0.0],
        ];
        let attachment = vec![
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.6,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 0.4,
            },
        ];
        let interior = vec![
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.55,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 0.45,
            },
        ];
        let mut weights = vec![
            attachment.clone(),
            interior.clone(),
            interior.clone(),
            attachment.clone(),
            interior.clone(),
            interior,
        ];

        let stats = stabilize_small_surface_components_v2(
            &positions,
            &[0, 1, 2, 3, 4, 5],
            &contract(),
            &mut weights,
        )
        .unwrap();

        assert_eq!(stats.component_count, 2);
        assert_eq!(stats.vertex_count, 4);
        assert_eq!(weights[0], attachment);
        assert_eq!(weights[3], attachment);
        assert!(weights.iter().all(|row| row == &attachment));
    }

    #[test]
    fn terminal_motion_is_not_removed_from_a_small_component() {
        let positions = vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]];
        let mut contract = contract();
        contract.nodes[3].structural_role = "APPENDAGE_TERMINAL".to_owned();
        contract.nodes[3].anchor_role = Some("tail_tip".to_owned());
        let original = vec![
            RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 0.6,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.4,
            },
        ];
        let mut weights = vec![original.clone(); 3];

        let stats =
            stabilize_small_surface_components_v2(&positions, &[0, 1, 2], &contract, &mut weights)
                .unwrap();

        assert_eq!(
            stats,
            SmallComponentProjectionStatsV1 {
                component_count: 0,
                vertex_count: 0,
            }
        );
        assert!(weights.iter().all(|row| row == &original));
    }

    #[test]
    fn local_triangle_cannot_bridge_sibling_carrier_branches() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [0.0, 0.001, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let row = |bone| {
            vec![RigWeightInfluenceV1 {
                bone_node_id: bone,
                value: 1.0,
            }]
        };
        let weights = vec![row(3), row(2), row(3), row(1)];
        assert_eq!(
            cross_branch_triangle_count(&positions, &[0, 1, 2], &weights, &contract()).unwrap(),
            1
        );
    }

    #[test]
    fn positive_influence_union_cannot_hide_behind_local_dominant_labels() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [0.0, 0.001, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let row = |dominant, neighbor| {
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: dominant,
                    value: 0.8,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: neighbor,
                    value: 0.2,
                },
            ]
        };
        // Dominants 1/1/3 span only one edge, but influence 2 and influence 3
        // are three topology edges apart in this test carrier tree.
        let weights = vec![row(1, 2), row(1, 2), row(3, 1), row(1, 1)];
        assert_eq!(
            cross_branch_triangle_count(&positions, &[0, 1, 2], &weights, &contract()).unwrap(),
            1
        );
    }

    #[test]
    fn bounded_boundary_repair_maps_a_local_triangle_to_one_carrier_neighbourhood() {
        let positions = vec![
            [-0.001, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [0.0, 0.001, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let contract = contract();
        let fitted = vec![
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.5, 0.0, 0.0],
        ];
        let repaired = repair_triangle_branch_boundaries(
            &positions,
            &[0, 1, 2],
            &[1, 2, 3],
            &fitted,
            &contract,
            model_diagonal(&positions),
            vec![3, 2, 3, 1],
            ReferenceSupermodelSkinningOptionsV1::default(),
        )
        .unwrap();
        assert_eq!(repaired.initial_violation_count, 1);
        assert!(repaired.reassigned_vertex_count > 0);
        assert_eq!(
            cross_branch_triangle_count_from_labels(
                &positions,
                &[0, 1, 2],
                &repaired.labels,
                &contract,
            )
            .unwrap(),
            0
        );
        let mut rebuilt = positions
            .iter()
            .zip(&repaired.labels)
            .map(|(position, label)| {
                local_segment_weights(
                    *position,
                    *label,
                    &[1, 2, 3],
                    &fitted,
                    &contract,
                    model_diagonal(&positions),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rebuilt
                .iter()
                .map(|row| row[0].bone_node_id)
                .collect::<Vec<_>>(),
            repaired.labels
        );
        constrain_weights_to_local_surface_label_field_v1(
            &positions,
            &[0, 1, 2],
            &repaired.labels,
            &[1, 2, 3],
            &fitted,
            model_diagonal(&positions),
            &contract,
            &mut rebuilt,
        )
        .unwrap();
        assert_eq!(
            cross_branch_triangle_count(&positions, &[0, 1, 2], &rebuilt, &contract).unwrap(),
            0
        );
    }

    #[test]
    fn sibling_triangle_junction_is_promoted_only_to_its_common_carrier() {
        let mut contract = contract();
        contract.nodes[2].parent_part_number = Some(1);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [0.0, 0.01, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let mut labels = vec![2, 3, 2, 3];

        let changed = promote_incomparable_triangles_to_common_carrier(
            &positions,
            &[0, 1, 2],
            &[1, 2, 3],
            &contract,
            &mut labels,
        );

        assert_eq!(changed, 3);
        assert_eq!(&labels[..3], &[1, 1, 1]);
        assert_eq!(labels[3], 3);
    }

    #[test]
    fn selected_topology_label_remains_dominant_when_a_child_segment_is_closer() {
        let contract = contract();
        let fitted = vec![
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.5, 0.0, 0.0],
        ];
        let row = local_segment_weights([-1.5, 0.0, 0.0], 1, &[1, 2, 3], &fitted, &contract, 3.0);
        assert_eq!(row[0].bone_node_id, 1);
        assert!(row.iter().any(|influence| influence.bone_node_id == 3));
    }

    #[test]
    fn sequential_joint_transition_keeps_a_graded_local_blend() {
        let mut contract = contract();
        contract.nodes[2].parent_part_number = Some(1);
        contract.nodes[3].parent_part_number = Some(2);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [0.0, 0.01, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let indices = [0, 1, 2];
        let labels = vec![1, 2, 2, 1];
        let fitted = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.25],
            [0.0, 0.0, 0.5],
            [0.0, 0.0, 0.75],
        ];
        let mut weights = vec![
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.6,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.4,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.5,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.25,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.25,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.6,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.4,
                },
            ],
            vec![RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 1.0,
            }],
        ];

        let stats = constrain_weights_to_local_surface_label_field_v1(
            &positions,
            &indices,
            &labels,
            &[1, 2, 3],
            &fitted,
            model_diagonal(&positions),
            &contract,
            &mut weights,
        )
        .unwrap();

        assert!(stats.rigid_group_count <= 1);
        assert!(
            weights[0]
                .iter()
                .any(|influence| influence.bone_node_id == 1)
        );
        assert!(weights[1].len() > 1);
        assert!(
            weights[2]
                .iter()
                .any(|influence| influence.bone_node_id == 2)
        );
        assert_eq!(
            cross_branch_triangle_count(&positions, &indices, &weights, &contract).unwrap(),
            0
        );
    }

    #[test]
    fn local_edge_cliff_is_relaxed_without_crossing_a_carrier_branch() {
        let positions = vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]];
        let row = |bone| {
            vec![RigWeightInfluenceV1 {
                bone_node_id: bone,
                value: 1.0,
            }]
        };
        let mut weights = vec![row(1), row(3), row(1)];

        let stats =
            relax_local_edge_weight_cliffs_v1(&positions, &[0, 1, 2], &contract(), &mut weights)
                .unwrap();

        assert!(stats.iteration_count > 0);
        assert!(stats.relaxed_group_count >= 2);
        assert!([(0, 1), (1, 2), (2, 0)].into_iter().all(|(left, right)| {
            rig_weight_row_maximum_delta_v1(&weights[left], &weights[right]) <= 0.1 + 1.0e-6
        }));
        assert_eq!(
            cross_branch_triangle_count(&positions, &[0, 1, 2], &weights, &contract()).unwrap(),
            0
        );
    }

    #[test]
    fn branch_junction_relaxes_only_the_selected_child_side() {
        let mut contract = contract();
        contract.nodes[2].parent_part_number = Some(1);
        let mut grandchild = contract.nodes[3].clone();
        grandchild.part_number = 4;
        grandchild.name = "joint_4".to_owned();
        grandchild.parent_part_number = Some(3);
        contract.nodes.push(grandchild);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [0.0, 0.01, 0.0],
            [0.02, 0.0, 0.0],
            [-0.01, 0.0, 0.0],
            [0.02, 0.01, 0.0],
        ];
        // The parent-side vertex (0) also borders sibling child 2.  The
        // selected child-side vertex (1) borders grandchild 4.  Blending both
        // endpoints would leak child 3 into the sibling triangle; blending
        // only vertex 1 toward parent 1 remains on one lineage.
        let indices = [0, 1, 2, 0, 4, 2, 1, 3, 5];
        let row = |bone| {
            vec![RigWeightInfluenceV1 {
                bone_node_id: bone,
                value: 1.0,
            }]
        };
        let mut weights = vec![row(1), row(3), row(1), row(4), row(2), row(3)];
        assert_eq!(
            cross_branch_triangle_count(&positions, &indices, &weights, &contract).unwrap(),
            0
        );

        let stats =
            relax_local_edge_weight_cliffs_v1(&positions, &indices, &contract, &mut weights)
                .unwrap();

        assert!(stats.iteration_count > 0);
        assert_eq!(weights[0], row(1));
        assert!(
            weights[1]
                .iter()
                .any(|influence| influence.bone_node_id == 1)
        );
        assert!(
            !weights[0]
                .iter()
                .any(|influence| influence.bone_node_id == 3)
        );
        assert_eq!(
            cross_branch_triangle_count(&positions, &indices, &weights, &contract).unwrap(),
            0
        );
        assert_eq!(
            local_edge_weight_cliff_count_v1(&positions, &indices, &contract, &weights).unwrap(),
            0
        );
    }

    #[test]
    fn final_boundary_pass_repairs_cliff_reintroduced_by_component_projection() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [0.0, 0.01, 0.0],
            [0.0, 0.0, 0.0],
            [-0.01, 0.0, 0.0],
            [0.0, -0.01, 0.0],
        ];
        let indices = [0, 1, 2, 3, 4, 5];
        let row = |bone| {
            vec![RigWeightInfluenceV1 {
                bone_node_id: bone,
                value: 1.0,
            }]
        };
        let blended = || {
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.5,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.5,
                },
            ]
        };
        let mut weights = vec![row(1), blended(), blended(), row(3), blended(), blended()];

        let projection =
            stabilize_small_surface_components_v2(&positions, &indices, &contract(), &mut weights)
                .unwrap();
        assert_eq!(projection.component_count, 2);

        // Seam welding after component projection blends the coincident
        // attachment only, leaving a renderer-visible cliff to each rigid
        // island interior.  The final production pass must repair that exact
        // post-projection state.
        weld_duplicate_positions(&positions, &contract(), &mut weights).unwrap();
        assert!(
            local_edge_weight_cliff_count_v1(&positions, &indices, &contract(), &weights).unwrap()
                > 0
        );

        let labels = [1, 1, 1, 3, 3, 3];
        let stats = finalize_projected_boundary_weights_v2(
            &positions,
            &indices,
            &labels,
            &contract(),
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.5],
            ],
            None,
            &mut weights,
        )
        .unwrap();

        assert!(stats.geodesic_boundary_seed_group_count > 0);
        assert_eq!(stats.gradient.violation_edge_count, 0);
        assert_eq!(weights[0], weights[3]);
    }

    #[test]
    fn triangle_projection_removes_a_five_edge_lineage_support() {
        let mut test_contract = contract();
        let mut grandchild = test_contract.nodes[3].clone();
        grandchild.part_number = 4;
        grandchild.name = "joint_4".to_owned();
        grandchild.parent_part_number = Some(3);
        test_contract.nodes.push(grandchild);
        let mut terminal = test_contract.nodes[4].clone();
        terminal.part_number = 5;
        terminal.name = "joint_5".to_owned();
        terminal.parent_part_number = Some(4);
        test_contract.nodes.push(terminal);
        let mut terminal_child = test_contract.nodes[5].clone();
        terminal_child.part_number = 6;
        terminal_child.name = "joint_6".to_owned();
        terminal_child.parent_part_number = Some(5);
        test_contract.nodes.push(terminal_child);
        let positions = vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]];
        let indices = [0, 1, 2];
        let labels = [1, 3, 3];
        let mut weights = vec![
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 0,
                    value: 0.9,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.1,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.9,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.1,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 5,
                    value: 0.2,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 6,
                    value: 0.8,
                },
            ],
        ];

        let stats = project_triangle_support_to_one_carrier_edge_v1(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &mut weights,
        )
        .unwrap();

        assert!(stats.projected_group_count > 0);
        assert!(triangle_weight_support_is_local_lineage(
            [0, 1, 2],
            &weights,
            &test_contract
        ));
    }

    #[test]
    fn microscopic_edges_receive_a_geometry_scaled_weight_gradient() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.0001, 0.0, 0.0],
            [0.0, 0.0001, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = [0, 1, 2];
        let mut weights = vec![
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.95,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.05,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.05,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.95,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.5,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.5,
                },
            ],
            vec![RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 1.0,
            }],
        ];

        let stats = relax_geometry_scaled_weight_gradients_v1(
            &positions,
            &indices,
            &contract(),
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.5],
            ],
            &mut weights,
        )
        .unwrap();

        assert!(stats.update_count > 0);
        assert!(stats.maximum_limit_ratio <= 1.01);
        assert!(weights.iter().all(|row| {
            (row.iter().map(|influence| influence.value).sum::<f32>() - 1.0).abs() <= 1.0e-4
        }));
    }

    #[test]
    fn four_carrier_edge_relaxation_does_not_drop_an_outer_joint() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        test_contract.nodes[3].parent_part_number = Some(2);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = [0, 1, 2];
        let left = vec![
            RigWeightInfluenceV1 {
                bone_node_id: 2,
                value: 0.78541666,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 0,
                value: 0.1276294,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 0.086953975,
            },
        ];
        let right = vec![
            RigWeightInfluenceV1 {
                bone_node_id: 2,
                value: 0.77560014,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 0,
                value: 0.12603492,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 0.06786496,
            },
            RigWeightInfluenceV1 {
                bone_node_id: 3,
                value: 0.03050003,
            },
        ];
        let mut weights = vec![
            left,
            right.clone(),
            right,
            vec![RigWeightInfluenceV1 {
                bone_node_id: 0,
                value: 1.0,
            }],
        ];

        let stats = relax_geometry_scaled_weight_gradients_v1(
            &positions,
            &indices,
            &test_contract,
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.2],
                [0.0, 0.0, 0.3],
            ],
            &mut weights,
        )
        .unwrap();

        assert!(stats.update_count > 0);
        assert!(stats.maximum_limit_ratio <= 1.01);
        assert!(
            weights[0]
                .iter()
                .any(|influence| influence.bone_node_id == 3)
        );
        assert!(
            weights[1]
                .iter()
                .any(|influence| influence.bone_node_id == 1)
        );
    }

    #[test]
    fn label_thickening_reserves_a_dominant_gradient_core() {
        let test_contract = contract();
        let diagonal = 1.0;
        let fitted = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.1],
            [0.0, 0.0, -0.2],
            [0.0, 0.0, 0.3],
        ];
        let permitted = (test_contract.tolerances.edge_hard_max_ratio - 1.0).max(1.0e-6);
        let width = carrier_geodesic_blend_width_v1(1, diagonal, &test_contract, &fitted);
        let minimum_work = carrier_label_minimum_gradient_work_v2();

        assert!(width + 1.0e-6 >= GEODESIC_DERIVATIVE_SAFETY_FACTOR * 0.2 / permitted);
        assert!(width >= diagonal / 128.0);
        assert!(minimum_work > 0.5);
        assert!(minimum_work <= 0.5 + MAX_RENDER_EDGE_WEIGHT_DELTA + 2.0e-6);
    }

    #[test]
    fn semantic_geodesic_field_reaches_the_shared_carrier_at_a_child_boundary() {
        let test_contract = contract();
        let column_count = 25usize;
        let mut positions = Vec::with_capacity(column_count * 2);
        let mut labels = Vec::with_capacity(column_count * 2);
        for row in 0..2 {
            for column in 0..column_count {
                positions.push([column as f32 * 0.1, row as f32 * 0.1, 0.0]);
                labels.push(if column == 0 {
                    0
                } else if column == column_count - 1 {
                    3
                } else {
                    1
                });
            }
        }
        let mut indices = Vec::new();
        for column in 0..column_count - 1 {
            let lower_left = column as u32;
            let lower_right = (column + 1) as u32;
            let upper_left = (column_count + column) as u32;
            let upper_right = (column_count + column + 1) as u32;
            indices.extend_from_slice(&[
                lower_left,
                lower_right,
                upper_right,
                lower_left,
                upper_right,
                upper_left,
            ]);
        }
        let fitted = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.05],
            [0.0, 0.0, 0.05],
            [0.0, 0.0, 0.10],
        ];
        let mut weights = vec![Vec::new(); positions.len()];

        seed_local_lineage_multisource_geodesic_field_v2(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &fitted,
            &mut weights,
        )
        .unwrap();

        let parent_side = column_count - 2;
        let child_side = column_count - 1;
        assert_eq!(weights[parent_side], weights[child_side]);
        assert!(
            weights[parent_side]
                .iter()
                .any(|influence| influence.bone_node_id == 1 && influence.value >= 0.5)
        );
        let gradient = audit_geometry_scaled_weight_gradients_v2(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &fitted,
            &weights,
        )
        .unwrap();
        assert_eq!(gradient.violation_edge_count, 0);
    }

    #[test]
    fn four_lane_window_rolls_before_a_fifth_carrier_is_added() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        test_contract.nodes[3].parent_part_number = Some(2);
        let mut terminal = test_contract.nodes[3].clone();
        terminal.part_number = 4;
        terminal.name = "joint_4".to_owned();
        terminal.parent_part_number = Some(3);
        test_contract.nodes.push(terminal);
        let source = (0..=3)
            .map(|bone_node_id| RigWeightInfluenceV1 {
                bone_node_id,
                value: 0.25,
            })
            .collect::<Vec<_>>();

        let before_roll =
            advance_lineage_row_toward_carrier_v1(&source, 4, 0.24, &test_contract).unwrap();
        let after_roll =
            advance_lineage_row_toward_carrier_v1(&source, 4, 0.26, &test_contract).unwrap();

        for row in [&before_roll, &after_roll] {
            let support = row
                .iter()
                .map(|influence| influence.bone_node_id)
                .collect::<BTreeSet<_>>();
            assert!(row.len() <= 4);
            assert!(carrier_support_is_single_lineage(
                &support,
                &test_contract,
                MAX_LOCAL_VERTEX_INFLUENCE_SPAN,
            ));
        }
        assert!(rig_weight_row_maximum_delta_v1(&before_roll, &after_roll) <= 0.020001);
        assert!(
            after_roll
                .iter()
                .any(|influence| influence.bone_node_id == 4)
        );

        let terminal =
            advance_lineage_row_toward_carrier_v1(&source, 4, 1.25, &test_contract).unwrap();
        assert_eq!(
            terminal,
            vec![RigWeightInfluenceV1 {
                bone_node_id: 4,
                value: 1.0,
            }]
        );
    }

    #[test]
    fn rigid_parent_child_boundary_opens_only_a_safe_one_ring_blend_strip() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.0001, 0.0, 0.0],
            [0.0001, 0.0001, 0.0],
            [0.0, -0.0001, 0.0],
            [0.0002, 0.0001, 0.0],
            [0.0002, 0.0002, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = [0, 1, 2, 0, 3, 1, 2, 4, 5];
        let row = |bone| {
            vec![RigWeightInfluenceV1 {
                bone_node_id: bone,
                value: 1.0,
            }]
        };
        let mut weights = vec![row(1), row(1), row(3), row(1), row(3), row(3), row(1)];

        let stats = relax_geometry_scaled_weight_gradients_v1(
            &positions,
            &indices,
            &contract(),
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.5],
            ],
            &mut weights,
        )
        .unwrap();

        assert!(stats.update_count > 0);
        assert!(stats.maximum_limit_ratio <= 1.01);
        assert!(indices.chunks_exact(3).all(|triangle| {
            triangle_weight_support_is_local_lineage(
                [
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                ],
                &weights,
                &contract(),
            )
        }));
    }

    #[test]
    fn chain_junction_expands_only_toward_the_shared_middle_carrier() {
        let mut test_contract = contract();
        let mut lower = test_contract.nodes[3].clone();
        lower.part_number = 4;
        lower.name = "joint_4".to_owned();
        lower.parent_part_number = Some(3);
        test_contract.nodes.push(lower);
        let mut terminal = test_contract.nodes[4].clone();
        terminal.part_number = 5;
        terminal.name = "joint_5".to_owned();
        terminal.parent_part_number = Some(4);
        test_contract.nodes.push(terminal);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.0001, 0.0, 0.0],
            [0.0001, 0.0001, 0.0],
            [0.0, -0.0001, 0.0],
            [0.0002, 0.0001, 0.0],
            [0.0002, 0.0002, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = [0, 1, 2, 0, 3, 1, 2, 4, 5];
        let upper = || {
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.2,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.8,
                },
            ]
        };
        let lower = || {
            vec![RigWeightInfluenceV1 {
                bone_node_id: 4,
                value: 1.0,
            }]
        };
        let terminal = || {
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 4,
                    value: 0.8,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 5,
                    value: 0.2,
                },
            ]
        };
        let mut weights = vec![
            upper(),
            upper(),
            lower(),
            upper(),
            terminal(),
            terminal(),
            upper(),
        ];
        let fitted = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.1],
            [0.0, 0.0, 0.1],
            [0.0, 0.0, 0.3],
            [0.0, 0.0, 0.5],
            [0.0, 0.0, 0.7],
        ];

        let stats = relax_geometry_scaled_weight_gradients_v1(
            &positions,
            &indices,
            &test_contract,
            &fitted,
            &mut weights,
        )
        .unwrap();

        assert!(stats.update_count > 0);
        assert!(stats.maximum_limit_ratio <= 1.01);
        assert!(
            weights[2]
                .iter()
                .any(|influence| influence.bone_node_id == 3)
        );
        assert!(
            weights[2]
                .iter()
                .all(|influence| influence.bone_node_id != 1)
        );
        assert!(indices.chunks_exact(3).all(|triangle| {
            triangle_weight_support_is_local_lineage(
                [
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                ],
                &weights,
                &test_contract,
            )
        }));
    }

    #[test]
    fn paired_edge_proposal_cannot_combine_two_outer_chain_carriers() {
        let mut test_contract = contract();
        let mut lower = test_contract.nodes[3].clone();
        lower.part_number = 4;
        lower.name = "joint_4".to_owned();
        lower.parent_part_number = Some(3);
        test_contract.nodes.push(lower);
        let mut terminal = test_contract.nodes[4].clone();
        terminal.part_number = 5;
        terminal.name = "joint_5".to_owned();
        terminal.parent_part_number = Some(4);
        test_contract.nodes.push(terminal);
        let mut terminal_child = test_contract.nodes[5].clone();
        terminal_child.part_number = 6;
        terminal_child.name = "joint_6".to_owned();
        terminal_child.parent_part_number = Some(5);
        test_contract.nodes.push(terminal_child);
        let mut terminal_grandchild = test_contract.nodes[6].clone();
        terminal_grandchild.part_number = 7;
        terminal_grandchild.name = "joint_7".to_owned();
        terminal_grandchild.parent_part_number = Some(6);
        test_contract.nodes.push(terminal_grandchild);
        let row = |parts: &[(u32, f32)]| {
            parts
                .iter()
                .map(|(bone_node_id, value)| RigWeightInfluenceV1 {
                    bone_node_id: *bone_node_id,
                    value: *value,
                })
                .collect::<Vec<_>>()
        };
        let triangle_groups = [[0, 1, 2]];
        let groups = vec![vec![0], vec![1], vec![2]];
        let weights = vec![
            row(&[(3, 1.0)]),
            row(&[(4, 1.0)]),
            row(&[(3, 0.5), (4, 0.5)]),
        ];
        let touched = [0].into_iter().collect::<BTreeSet<_>>();

        assert!(!edge_weight_proposal_preserves_local_triangle_support_v1(
            &touched,
            &triangle_groups,
            &groups,
            &weights,
            0,
            1,
            &row(&[(1, 0.2), (3, 0.8)]),
            &row(&[(4, 0.8), (7, 0.2)]),
            &test_contract,
        ));
        assert!(edge_weight_proposal_preserves_local_triangle_support_v1(
            &touched,
            &triangle_groups,
            &groups,
            &weights,
            0,
            1,
            &row(&[(3, 1.0)]),
            &row(&[(4, 1.0)]),
            &test_contract,
        ));
    }

    #[test]
    fn symmetric_edge_relaxation_progresses_when_another_one_ring_cliff_ties_the_maximum() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [0.0, 0.01, 0.0],
            [0.01, 0.01, 0.0],
        ];
        let indices = [0, 1, 2, 1, 2, 3];
        let row = |bone| {
            vec![RigWeightInfluenceV1 {
                bone_node_id: bone,
                value: 1.0,
            }]
        };
        let mut weights = vec![row(1), row(3), row(3), row(1)];

        let stats =
            relax_local_edge_weight_cliffs_v1(&positions, &indices, &contract(), &mut weights)
                .unwrap();

        assert!(stats.iteration_count > 0);
        assert_eq!(stats.relaxed_group_count, 4);
        assert!(
            [(0, 1), (1, 2), (2, 0), (1, 3), (3, 2)]
                .into_iter()
                .all(|(left, right)| {
                    rig_weight_row_maximum_delta_v1(&weights[left], &weights[right]) <= 0.1 + 1.0e-6
                })
        );
    }

    #[test]
    fn existing_single_lineage_field_can_recover_a_missing_dominant_cluster() {
        let mut contract = contract();
        contract.nodes[2].parent_part_number = Some(1);
        contract.nodes[3].parent_part_number = Some(2);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [0.02, 0.0, 0.0],
            [0.03, 0.0, 0.0],
        ];
        let fitted = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.25],
            [0.0, 0.0, 0.5],
            [0.0, 0.0, 0.75],
        ];
        let mut weights = vec![
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.6,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.4,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 0.55,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.45,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.55,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.45,
                },
            ],
            vec![
                RigWeightInfluenceV1 {
                    bone_node_id: 3,
                    value: 0.6,
                },
                RigWeightInfluenceV1 {
                    bone_node_id: 2,
                    value: 0.4,
                },
            ],
        ];

        let promoted = promote_existing_local_influence_clusters_v1(
            &positions,
            &[1, 2, 3],
            &fitted,
            &contract,
            &mut weights,
        )
        .unwrap();

        assert_eq!(promoted, 1);
        assert!(weights.iter().any(|row| {
            row.first()
                .is_some_and(|influence| influence.bone_node_id == 2)
        }));
    }

    #[test]
    fn duplicate_uv_vertices_are_an_atomic_label_group() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.0, 0.1, 0.0],
            [0.0, 0.0, 0.0],
            [-0.1, 0.0, 0.0],
            [0.0, -0.1, 0.0],
        ];
        let indices = [0, 1, 2, 3, 4, 5];
        let contract = contract();
        let fitted = vec![
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.5, 0.0, 0.0],
        ];
        let diagonal = model_diagonal(&positions);
        let adjacency = local_surface_adjacency(&positions, &indices, diagonal).unwrap();
        let topology = carrier_topology_distance_matrix(&contract);
        let mut labels = vec![3, 3, 3, 2, 2, 2];
        assert!(synchronize_duplicate_label_groups(
            &positions,
            &adjacency,
            &[1, 2, 3],
            &fitted,
            &contract,
            diagonal,
            &topology,
            &mut labels,
        ));
        assert_eq!(labels[0], labels[3]);
    }

    #[test]
    fn coincident_vertices_on_sibling_surface_junctions_are_not_one_seam() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.0, 0.1, 0.0],
            [0.0, 0.0, 0.0],
            [-0.1, 0.0, 0.0],
            [0.0, -0.1, 0.0],
        ];
        let indices = [0, 1, 2, 3, 4, 5];
        let labels = [1, 2, 1, 1, 3, 3];

        let groups = duplicate_position_label_topology_groups_v1(
            &positions,
            &indices,
            &labels,
            &test_contract,
        )
        .unwrap();

        assert!(
            !groups
                .iter()
                .any(|group| group.contains(&0) && group.contains(&3))
        );
    }

    #[test]
    fn authored_weight_seams_keep_coincident_sibling_surfaces_separate() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.0, 0.1, 0.0],
            [0.0, 0.0, 0.0],
            [-0.1, 0.0, 0.0],
            [0.0, -0.1, 0.0],
        ];
        let indices = [0, 1, 2, 3, 4, 5];
        let row = |bone_node_id| {
            vec![RigWeightInfluenceV1 {
                bone_node_id,
                value: 1.0,
            }]
        };
        let weights = vec![row(1), row(2), row(1), row(1), row(3), row(3)];

        let groups = duplicate_position_weight_topology_groups_v1(
            &positions,
            &indices,
            &weights,
            &test_contract,
        )
        .unwrap();

        assert!(
            !groups
                .iter()
                .any(|group| group.contains(&0) && group.contains(&3))
        );
    }

    #[test]
    fn final_seam_weld_uses_semantic_topology_not_a_divergent_weight_window() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        test_contract.nodes[3].parent_part_number = Some(2);
        for (part_number, parent_part_number) in [(4, 3), (5, 4), (6, 5)] {
            let mut node = test_contract.nodes[3].clone();
            node.part_number = part_number;
            node.name = format!("joint_{part_number}");
            node.parent_part_number = Some(parent_part_number);
            test_contract.nodes.push(node);
        }
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.0, 0.1, 0.0],
            [0.0, 0.0, 0.0],
            [-0.1, 0.0, 0.0],
            [0.0, -0.1, 0.0],
        ];
        let indices = [0, 1, 2, 3, 4, 5];
        let labels = [3, 3, 3, 3, 3, 3];
        let row = |bones: [u32; 4]| {
            bones
                .into_iter()
                .map(|bone_node_id| RigWeightInfluenceV1 {
                    bone_node_id,
                    value: 0.25,
                })
                .collect::<Vec<_>>()
        };
        let mut weights = vec![
            row([1, 2, 3, 4]),
            row([1, 2, 3, 4]),
            row([1, 2, 3, 4]),
            row([3, 4, 5, 6]),
            row([3, 4, 5, 6]),
            row([3, 4, 5, 6]),
        ];

        weld_duplicate_position_label_topology_groups_v1(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &mut weights,
        )
        .unwrap();

        assert_eq!(weights[0], weights[3]);
    }

    #[test]
    fn seam_normalization_preserves_one_three_carrier_chain_and_excludes_siblings() {
        let mut chain_contract = contract();
        let mut grandchild = chain_contract.nodes[3].clone();
        grandchild.part_number = 4;
        grandchild.name = "joint_4".to_owned();
        grandchild.parent_part_number = Some(3);
        chain_contract.nodes.push(grandchild);
        let chain = normalize_local_branch_weights(
            BTreeMap::from([(1, 0.3), (3, 0.3), (4, 0.4)]),
            &chain_contract,
            4,
            0.0,
        )
        .unwrap();
        assert_eq!(
            chain
                .iter()
                .map(|influence| influence.bone_node_id)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([1, 3, 4])
        );

        let mut branch_contract = contract();
        branch_contract.nodes[2].parent_part_number = Some(1);
        let branch = normalize_local_branch_weights(
            BTreeMap::from([(1, 0.2), (2, 0.4), (3, 0.4)]),
            &branch_contract,
            4,
            0.0,
        )
        .unwrap();
        let support = branch
            .iter()
            .map(|influence| influence.bone_node_id)
            .collect::<BTreeSet<_>>();
        assert!(carrier_support_is_single_lineage(
            &support,
            &branch_contract,
            MAX_LOCAL_VERTEX_INFLUENCE_SPAN
        ));
        assert!(!(support.contains(&2) && support.contains(&3)));
    }

    #[test]
    fn chain_thickening_repair_reinserts_a_skipped_middle_label() {
        let mut test_contract = contract();
        let mut grandchild = test_contract.nodes[3].clone();
        grandchild.part_number = 4;
        grandchild.name = "joint_4".to_owned();
        grandchild.parent_part_number = Some(3);
        test_contract.nodes.push(grandchild);
        let positions = vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]];
        let indices = [0, 1, 2];
        let mut labels = [1, 4, 4];

        let changed = repair_nonlocal_label_transitions_after_thickening_v1(
            &positions,
            &indices,
            &mut labels,
            &test_contract,
        )
        .unwrap();

        assert_eq!(changed, 2);
        assert_eq!(labels, [1, 3, 3]);
    }

    #[test]
    fn child_weight_starts_one_sided_and_never_leaks_into_parent_region() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [0.001, 0.001, 0.0],
            [0.0, 1.0, 0.0],
            [0.001, 1.0, 0.0],
            [0.001, 1.001, 0.0],
        ];
        let indices = [0, 1, 2, 3, 4, 5];
        let labels = [1, 2, 2, 1, 3, 3];
        let mut weights = vec![Vec::new(); positions.len()];

        seed_local_lineage_multisource_geodesic_field_v2(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.2, 0.0, 0.1],
                [-0.2, 0.0, 0.1],
            ],
            &mut weights,
        )
        .unwrap();

        let value = |vertex: usize, bone: u32| {
            weights[vertex]
                .iter()
                .find(|influence| influence.bone_node_id == bone)
                .map_or(0.0, |influence| influence.value)
        };
        assert_eq!(value(0, 2), 0.0);
        assert_eq!(value(3, 3), 0.0);
        assert!(value(1, 1) > 0.9);
        assert!(value(1, 2) < 0.1);
        assert!(value(4, 1) > 0.9);
        assert!(value(4, 3) < 0.1);
    }

    #[test]
    fn four_lane_support_remains_continuous_at_the_next_child_boundary() {
        let mut test_contract = contract();
        test_contract.nodes[2].parent_part_number = Some(1);
        test_contract.nodes[3].parent_part_number = Some(2);
        let positions = vec![
            [0.0, 0.0, 0.1],
            [0.0, 0.001, 0.1],
            [0.0, 0.0, 0.101],
            [0.0, 0.001, 0.101],
            [0.0, 0.0, 0.2],
            [0.0, 0.001, 0.2],
            [0.0, 0.0, 0.201],
            [0.0, 0.001, 0.201],
        ];
        let indices = [0, 2, 1, 1, 2, 3, 2, 4, 3, 3, 4, 5, 4, 6, 5, 5, 6, 7];
        let labels = [1, 1, 2, 2, 2, 2, 3, 3];
        let mut weights = vec![Vec::new(); positions.len()];

        seed_local_lineage_multisource_geodesic_field_v2(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.2],
                [0.0, 0.0, 0.3],
            ],
            &mut weights,
        )
        .unwrap();

        let value = |vertex: usize, bone: u32| {
            weights[vertex]
                .iter()
                .find(|influence| influence.bone_node_id == bone)
                .map_or(0.0, |influence| influence.value)
        };
        assert!((value(4, 1) - value(6, 1)).abs() < 0.1);
        assert!((value(4, 2) - value(6, 2)).abs() < 0.1);
        assert!(value(4, 2) > 0.0);
        assert!(value(6, 2) > 0.0);
        assert!(value(6, 3) < 0.1);
        let gradient = audit_geometry_scaled_weight_gradients_v2(
            &positions,
            &indices,
            &labels,
            &test_contract,
            &[
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1],
                [0.0, 0.0, 0.2],
                [0.0, 0.0, 0.3],
            ],
            &weights,
        )
        .unwrap();
        assert_eq!(gradient.violation_edge_count, 0);
    }

    #[test]
    fn consistent_uv_labels_are_not_reassigned_to_geometrically_nearer_bones() {
        let contract = contract();
        let positions = vec![[1., 0., 0.], [1., 0., 0.], [1.1, 0., 0.]];
        let adjacency = vec![vec![2], vec![2], vec![0, 1]];
        let fitted = vec![[0.; 3], [-1., 0., 0.], [1., 0., 0.], [-2., 0., 0.]];
        let mut labels = vec![3, 3, 3];
        assert!(!synchronize_duplicate_label_groups(
            &positions,
            &adjacency,
            &[1, 2, 3],
            &fitted,
            &contract,
            4.,
            &carrier_topology_distance_matrix(&contract),
            &mut labels
        ));
        assert_eq!(labels, vec![3, 3, 3]);
    }

    #[test]
    fn region_diffusion_preserves_distal_and_middle_cores() {
        let mut contract = contract();
        contract.nodes[2].parent_part_number = Some(1);
        contract.nodes[3].parent_part_number = Some(2);
        let mut positions = Vec::new();
        let mut labels = Vec::new();
        let mut indices = Vec::new();
        for ring in 0..90 {
            for side in 0..2 {
                positions.push([ring as f32 / 90., side as f32 * 0.01, 0.]);
                labels.push(if ring < 30 {
                    1
                } else if ring < 60 {
                    2
                } else {
                    3
                });
            }
            if ring > 0 {
                let a = (ring * 2) as u32;
                indices.extend([a - 2, a - 1, a, a - 1, a + 1, a]);
            }
        }
        let mut rows = vec![Vec::new(); positions.len()];
        seed_region_preserving_diffusion_v3(&positions, &indices, &labels, &contract, &mut rows)
            .unwrap();
        for part in [1, 2, 3] {
            assert!(
                rows.iter()
                    .filter(|row| row[0].bone_node_id == part && row[0].value > 0.9)
                    .count()
                    > 20
            );
        }
        for row in rows {
            assert!((row.iter().map(|w| w.value).sum::<f32>() - 1.).abs() < 1e-5);
            assert!(row.len() <= 4);
        }
    }
}
