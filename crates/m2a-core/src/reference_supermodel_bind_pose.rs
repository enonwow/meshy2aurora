//! Bind-pose validation between fitted anatomy, semantic skin clusters and motion.
//!
//! This gate intentionally runs before animation sampling.  A complete carrier
//! inventory is not proof that pivots and the surface regions they drive form a
//! usable creature skeleton.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    profile_a::RigWeightInfluenceV1,
    reference_supermodel_generic::ReferenceSupermodelGenericErrorV2,
    reference_supermodel_motion::{
        ReferenceSupermodelCarrierClassV3, ReferenceSupermodelMotionContractV2,
    },
    reference_supermodel_skinning::ReferenceSupermodelSkinningReportV1,
    reference_supermodel_structure::{
        ReferenceSupermodelJointFitReportV1, ReferenceSupermodelStructuralProfileV1,
    },
    reference_supermodel_surface_anatomy::TargetSurfaceAnatomyV1,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelBindPoseViolationV1 {
    pub code: String,
    pub part_numbers: Vec<u32>,
    pub measured_fraction: f32,
    pub allowed_fraction: f32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelBindPoseJointClusterV1 {
    pub part_number: u32,
    pub joint_name: String,
    pub dominant_vertex_count: usize,
    pub barycenter: Option<[f32; 3]>,
    pub maximum_segment_distance_fraction: f32,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelBindPoseReportV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub content_sha256: String,
    pub required_check_count: usize,
    pub pass_count: usize,
    pub violations: Vec<ReferenceSupermodelBindPoseViolationV1>,
    pub joint_clusters: Vec<ReferenceSupermodelBindPoseJointClusterV1>,
}

#[allow(clippy::too_many_arguments)]
pub fn validate_reference_supermodel_bind_pose_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    anatomy: &TargetSurfaceAnatomyV1,
    joint_fit: &ReferenceSupermodelJointFitReportV1,
    skinning: &ReferenceSupermodelSkinningReportV1,
    positions: &[[f32; 3]],
    weights: &[Vec<RigWeightInfluenceV1>],
) -> Result<ReferenceSupermodelBindPoseReportV1, ReferenceSupermodelGenericErrorV2> {
    if positions.is_empty()
        || positions.len() != weights.len()
        || joint_fit.joints.len() != contract.nodes.len()
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-BIND-POSE-INPUT-INVALID",
            "bindPose",
            "bind-pose validation requires one weight row per target vertex and one fitted pivot per exact carrier",
        ));
    }
    let fitted = joint_fit
        .joints
        .iter()
        .map(|joint| joint.target_world_position)
        .collect::<Vec<_>>();
    let diagonal = distance(
        anatomy.authoritative_bounds.min,
        anatomy.authoritative_bounds.max,
    )
    .max(1.0e-6);
    let height =
        (anatomy.authoritative_bounds.max[2] - anatomy.authoritative_bounds.min[2]).max(1.0e-6);
    let mut checks = Vec::<ReferenceSupermodelBindPoseViolationV1>::new();

    push_check(
        &mut checks,
        "ANATOMY_READY",
        Vec::new(),
        f32::from(anatomy.status != "READY"),
        0.0,
        anatomy.status == "READY",
        format!("surface anatomy status is {}", anatomy.status),
    );
    push_check(
        &mut checks,
        "JOINT_FIT_READY",
        joint_fit
            .constraint_violations
            .iter()
            .flat_map(|constraint| constraint.part_numbers.iter().copied())
            .collect(),
        f32::from(joint_fit.status != "READY"),
        0.0,
        joint_fit.status == "READY",
        format!("joint fit status is {}", joint_fit.status),
    );
    push_check(
        &mut checks,
        "SKINNING_READY",
        Vec::new(),
        f32::from(skinning.status != "READY"),
        0.0,
        skinning.status == "READY",
        format!("skinning status is {}", skinning.status),
    );

    let mut invalid_segments = Vec::new();
    let mut minimum_segment_fraction = f32::INFINITY;
    for node in contract.nodes.iter().skip(1) {
        let Some(parent) = node.parent_part_number else {
            invalid_segments.push(node.part_number);
            continue;
        };
        let fraction =
            distance(fitted[parent as usize], fitted[node.part_number as usize]) / diagonal;
        minimum_segment_fraction = minimum_segment_fraction.min(fraction);
        if !fraction.is_finite() || fraction <= 0.005 {
            invalid_segments.push(node.part_number);
        }
    }
    push_check(
        &mut checks,
        "POSITIVE_BIND_SEGMENT_LENGTHS",
        invalid_segments.clone(),
        if minimum_segment_fraction.is_finite() {
            minimum_segment_fraction
        } else {
            0.0
        },
        0.005,
        invalid_segments.is_empty(),
        "every non-root carrier must retain a measurable parent-to-child segment".to_owned(),
    );

    let terminal_parts = structural_profile
        .ground_contact_chains
        .iter()
        .map(|chain| chain.terminal_part_number)
        .collect::<Vec<_>>();
    let immutable_reference_bind = joint_fit.algorithm == "IMMUTABLE_REFERENCE_BIND_V1";
    if immutable_reference_bind {
        push_check(
            &mut checks,
            "IMMUTABLE_GROUND_CONTACT_SEMANTICS",
            terminal_parts.clone(),
            0.0,
            0.0,
            true,
            "the selected supermodel's authored terminal offset is authoritative; ground placement is controlled by owned-mesh registration"
                .to_owned(),
        );
    } else {
        let maximum_terminal_height = terminal_parts
            .iter()
            .map(|part| {
                (fitted[*part as usize][2] - anatomy.authoritative_bounds.min[2]).abs() / height
            })
            .max_by(f32::total_cmp)
            .unwrap_or(f32::INFINITY);
        push_check(
            &mut checks,
            "GROUND_TERMINAL_HEIGHT",
            terminal_parts.clone(),
            maximum_terminal_height,
            0.01,
            terminal_parts.is_empty() || maximum_terminal_height <= 0.01,
            "every fitted ground terminal must lie on the detected contact plane".to_owned(),
        );
    }
    let mut minimum_terminal_separation = f32::INFINITY;
    for left in 0..terminal_parts.len() {
        for right in (left + 1)..terminal_parts.len() {
            let a = fitted[terminal_parts[left] as usize];
            let b = fitted[terminal_parts[right] as usize];
            minimum_terminal_separation = minimum_terminal_separation
                .min(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() / diagonal);
        }
    }
    push_check(
        &mut checks,
        "GROUND_TERMINALS_DISTINCT",
        terminal_parts.clone(),
        if minimum_terminal_separation.is_finite() {
            minimum_terminal_separation
        } else {
            0.0
        },
        0.015,
        terminal_parts.len() == structural_profile.ground_contact_chains.len()
            && (terminal_parts.len() < 2 || minimum_terminal_separation >= 0.015),
        "ground-contact chains must terminate in distinct planar clusters".to_owned(),
    );

    let mut wrong_side_parts = Vec::new();
    let mut maximum_pair_mirror_error = 0.0_f32;
    for pair in &structural_profile.symmetric_pairs {
        let left = fitted[pair.left_part_number as usize];
        let right = fitted[pair.right_part_number as usize];
        let parent = fitted[pair.parent_part_number as usize];
        let mirror_error = ((left[0] - parent[0]).abs() - (right[0] - parent[0]).abs())
            .abs()
            .hypot(left[1] - right[1])
            .hypot(left[2] - right[2])
            / diagonal;
        maximum_pair_mirror_error = maximum_pair_mirror_error.max(mirror_error);
        if (left[0] - parent[0]) * (right[0] - parent[0]) >= 0.0 || mirror_error > 0.015 {
            wrong_side_parts.extend([pair.left_part_number, pair.right_part_number]);
        }
    }
    push_check(
        &mut checks,
        "SYMMETRIC_BIND_ALIGNMENT",
        wrong_side_parts.clone(),
        maximum_pair_mirror_error,
        0.015,
        wrong_side_parts.is_empty(),
        "symmetric branch roots must remain mirrored around their fitted parent".to_owned(),
    );

    let mut appendage_failures = Vec::new();
    let mut minimum_appendage_progress = f32::INFINITY;
    for chain in &structural_profile.appendage_chains {
        let Some(first) = chain.part_numbers.first().copied() else {
            appendage_failures.push(chain.terminal_part_number);
            continue;
        };
        let Some(root) = contract.nodes[first as usize].parent_part_number else {
            appendage_failures.push(first);
            continue;
        };
        let origin = fitted[root as usize];
        let mut previous = 0.0_f32;
        for part in &chain.part_numbers {
            let progress = distance(origin, fitted[*part as usize]) / diagonal;
            minimum_appendage_progress = minimum_appendage_progress.min(progress - previous);
            if progress <= previous + 0.005 {
                appendage_failures.push(*part);
            }
            previous = progress;
        }
    }
    push_check(
        &mut checks,
        "APPENDAGE_OUTWARD_ORDER",
        appendage_failures.clone(),
        if minimum_appendage_progress.is_finite() {
            minimum_appendage_progress
        } else {
            0.0
        },
        0.005,
        appendage_failures.is_empty(),
        "appendage pivots must progress from branch root to terminal".to_owned(),
    );

    let mut dominant_vertices = BTreeMap::<u32, Vec<usize>>::new();
    for (vertex, row) in weights.iter().enumerate() {
        if let Some(dominant) = row.first().filter(|influence| influence.value > 0.0) {
            dominant_vertices
                .entry(dominant.bone_node_id)
                .or_default()
                .push(vertex);
        }
    }
    let mut joint_clusters = Vec::new();
    let mut invalid_cluster_parts = Vec::new();
    for node in contract
        .nodes
        .iter()
        .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
    {
        let vertices = dominant_vertices
            .get(&node.part_number)
            .cloned()
            .unwrap_or_default();
        let barycenter = (!vertices.is_empty()).then(|| {
            std::array::from_fn(|axis| {
                vertices
                    .iter()
                    .map(|vertex| positions[*vertex][axis])
                    .sum::<f32>()
                    / vertices.len() as f32
            })
        });
        let maximum_distance = vertices
            .iter()
            .map(|vertex| {
                segment_distance(positions[*vertex], node.part_number, &fitted, contract) / diagonal
            })
            .max_by(f32::total_cmp)
            .unwrap_or(f32::INFINITY);
        let pass = !vertices.is_empty() && maximum_distance.is_finite() && maximum_distance <= 0.35;
        if !pass {
            invalid_cluster_parts.push(node.part_number);
        }
        joint_clusters.push(ReferenceSupermodelBindPoseJointClusterV1 {
            part_number: node.part_number,
            joint_name: node.name.clone(),
            dominant_vertex_count: vertices.len(),
            barycenter,
            maximum_segment_distance_fraction: maximum_distance,
            status: if pass { "PASS" } else { "BLOCKED" }.to_owned(),
        });
    }
    push_check(
        &mut checks,
        "SEMANTIC_DOMINANT_CLUSTERS",
        invalid_cluster_parts.clone(),
        invalid_cluster_parts.len() as f32,
        0.0,
        invalid_cluster_parts.is_empty(),
        "every skin-relevant carrier requires a local dominant surface cluster".to_owned(),
    );
    push_check(
        &mut checks,
        "NO_NONLOCAL_BRANCH_TRIANGLES",
        Vec::new(),
        skinning.cross_branch_triangle_count as f32,
        0.0,
        skinning.cross_branch_triangle_count == 0,
        "a rendered triangle may not bridge non-adjacent carrier branches".to_owned(),
    );

    let required_check_count = checks.len();
    let violations = checks
        .into_iter()
        .filter(|check| check.code.starts_with("BLOCKED_"))
        .map(|mut check| {
            check.code = check.code.trim_start_matches("BLOCKED_").to_owned();
            check
        })
        .collect::<Vec<_>>();
    let pass_count = required_check_count.saturating_sub(violations.len());
    let mut report = ReferenceSupermodelBindPoseReportV1 {
        schema_version: 1,
        algorithm: "ANATOMY_CLUSTER_BIND_POSE_VALIDATION_V1".to_owned(),
        status: if violations.is_empty() {
            "PASS"
        } else {
            "BLOCKED"
        }
        .to_owned(),
        content_sha256: String::new(),
        required_check_count,
        pass_count,
        violations,
        joint_clusters,
    };
    report.content_sha256 = sha256_json(&report)?;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn push_check(
    checks: &mut Vec<ReferenceSupermodelBindPoseViolationV1>,
    code: &str,
    part_numbers: Vec<u32>,
    measured_fraction: f32,
    allowed_fraction: f32,
    passed: bool,
    message: String,
) {
    checks.push(ReferenceSupermodelBindPoseViolationV1 {
        code: if passed {
            format!("PASS_{code}")
        } else {
            format!("BLOCKED_{code}")
        },
        part_numbers: part_numbers
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        measured_fraction,
        allowed_fraction,
        message,
    });
}

fn segment_distance(
    point: [f32; 3],
    part: u32,
    fitted: &[[f32; 3]],
    contract: &ReferenceSupermodelMotionContractV2,
) -> f32 {
    let end = fitted[part as usize];
    let start = contract.nodes[part as usize]
        .parent_part_number
        .map(|parent| fitted[parent as usize])
        .unwrap_or(end);
    let delta = sub(end, start);
    let length_squared = dot(delta, delta);
    if length_squared <= 1.0e-12 {
        return distance(point, end);
    }
    let progress = (dot(sub(point, start), delta) / length_squared).clamp(0.0, 1.0);
    distance(point, add(start, scale(delta, progress)))
}

fn add(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|axis| left[axis] + right[axis])
}

fn sub(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|axis| left[axis] - right[axis])
}

fn scale(value: [f32; 3], amount: f32) -> [f32; 3] {
    value.map(|component| component * amount)
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    dot(sub(left, right), sub(left, right)).sqrt()
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
            "M2A-REFERENCE-SUPERMODEL-BIND-POSE-SERIALIZE",
            "bindPose",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
