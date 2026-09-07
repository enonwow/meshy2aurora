//! Structure-derived target anatomy and joint fitting for reference supermodels.
//!
//! The selected supermodel contributes an exact carrier topology.  Target
//! pivots are fitted to caller-owned geometry without family/resref branches.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    profile_a::{Bounds3V1, CreatureRigProfileV1},
    reference_supermodel_generic::ReferenceSupermodelGenericErrorV2,
    reference_supermodel_motion::{
        ReferenceSupermodelCarrierClassV3, ReferenceSupermodelMotionContractV2,
    },
    reference_supermodel_surface_anatomy::TargetSurfaceAnatomyArtifactV1,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelStructuralChainV1 {
    pub chain_id: String,
    pub kind: String,
    pub part_numbers: Vec<u32>,
    pub terminal_part_number: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelSymmetricPairV1 {
    pub left_part_number: u32,
    pub right_part_number: u32,
    pub parent_part_number: u32,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelStructuralProfileV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub content_sha256: String,
    pub root_part_number: u32,
    pub motion_root_part_number: u32,
    pub central_chain_part_numbers: Vec<u32>,
    pub ground_contact_chains: Vec<ReferenceSupermodelStructuralChainV1>,
    pub appendage_chains: Vec<ReferenceSupermodelStructuralChainV1>,
    pub helper_part_numbers: Vec<u32>,
    pub symmetric_pairs: Vec<ReferenceSupermodelSymmetricPairV1>,
    pub required_surface_regions: Vec<String>,
    pub optional_surface_regions: Vec<String>,
    pub exact_topology_preserved: bool,
    pub family_or_resref_rule_used: bool,
}

pub fn build_reference_supermodel_structural_profile_v1(
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<ReferenceSupermodelStructuralProfileV1, ReferenceSupermodelGenericErrorV2> {
    if contract.nodes.is_empty() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
            "contract.nodes",
            "structural profiling requires an exact non-empty carrier topology",
        ));
    }
    let children = child_counts(contract);
    let reference_worlds = contract_world_matrices(contract)?;
    let reference_points = reference_worlds
        .iter()
        .map(|matrix| [matrix[12], matrix[13], matrix[14]])
        .collect::<Vec<_>>();
    let reference_bounds = bounds(&reference_points)?;
    let reference_diagonal = bounds_diagonal(reference_bounds).max(1.0e-5);
    let motion_root_part_number = contract
        .nodes
        .iter()
        .find(|node| node.position_controller_required)
        .map_or(0, |node| node.part_number);

    let ground_contact_chains = contract
        .nodes
        .iter()
        .filter(|node| node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL")
        .enumerate()
        .map(|(index, node)| ReferenceSupermodelStructuralChainV1 {
            chain_id: format!("ground_contact_{index}"),
            kind: "GROUND_CONTACT".to_owned(),
            part_numbers: terminal_chain(contract, node.part_number, &children),
            terminal_part_number: node.part_number,
        })
        .collect::<Vec<_>>();
    let appendage_terminals = contract
        .nodes
        .iter()
        .filter(|node| {
            node.structural_role == "APPENDAGE_TERMINAL"
                || (is_appendage_semantic(node)
                    && !contract.nodes.iter().any(|child| {
                        child.parent_part_number == Some(node.part_number)
                            && is_appendage_semantic(child)
                    }))
        })
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    let appendage_chains = appendage_terminals
        .into_iter()
        .enumerate()
        .map(|(index, terminal)| ReferenceSupermodelStructuralChainV1 {
            chain_id: format!("appendage_{index}"),
            kind: "APPENDAGE".to_owned(),
            part_numbers: terminal_chain(contract, terminal, &children),
            terminal_part_number: terminal,
        })
        .collect::<Vec<_>>();

    let branch_parts = ground_contact_chains
        .iter()
        .chain(&appendage_chains)
        .flat_map(|chain| chain.part_numbers.iter().copied())
        .collect::<BTreeSet<_>>();
    let reference_center_x = (reference_bounds.min[0] + reference_bounds.max[0]) * 0.5;
    let reference_width = (reference_bounds.max[0] - reference_bounds.min[0]).max(1.0e-5);
    let central_candidates = contract
        .nodes
        .iter()
        .filter(|candidate| {
            candidate.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant
                && !branch_parts.contains(&candidate.part_number)
                && (reference_points[candidate.part_number as usize][0] - reference_center_x).abs()
                    <= reference_width * 0.04
        })
        .map(|node| node.part_number)
        .collect::<BTreeSet<_>>();
    let central_chain_part_numbers =
        longest_rooted_candidate_chain(motion_root_part_number, &central_candidates, contract);
    let helper_part_numbers = contract
        .nodes
        .iter()
        .filter(|node| node.carrier_class != ReferenceSupermodelCarrierClassV3::SkinRelevant)
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    let symmetric_pairs = detect_symmetric_pairs(
        contract,
        &reference_points,
        reference_bounds,
        reference_diagonal,
    );
    let mut required_surface_regions = vec!["central_body".to_owned()];
    required_surface_regions.extend(
        ground_contact_chains
            .iter()
            .map(|chain| chain.chain_id.clone()),
    );
    required_surface_regions.extend(appendage_chains.iter().map(|chain| chain.chain_id.clone()));
    let optional_surface_regions = helper_part_numbers
        .iter()
        .map(|part| format!("helper_{part}"))
        .collect::<Vec<_>>();
    let mut profile = ReferenceSupermodelStructuralProfileV1 {
        schema_version: 1,
        algorithm: "EXACT_TOPOLOGY_STRUCTURAL_PROFILE_V1".to_owned(),
        status: "READY".to_owned(),
        content_sha256: String::new(),
        root_part_number: 0,
        motion_root_part_number,
        central_chain_part_numbers,
        ground_contact_chains,
        appendage_chains,
        helper_part_numbers,
        symmetric_pairs,
        required_surface_regions,
        optional_surface_regions,
        exact_topology_preserved: true,
        family_or_resref_rule_used: false,
    };
    profile.content_sha256 = sha256_json(&profile)?;
    Ok(profile)
}

fn longest_rooted_candidate_chain(
    motion_root: u32,
    candidates: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Vec<u32> {
    let mut best = Vec::new();
    for terminal in candidates.iter().copied() {
        let mut reverse = vec![terminal];
        let mut current = terminal;
        while current != motion_root {
            let Some(parent) = contract.nodes[current as usize].parent_part_number else {
                reverse.clear();
                break;
            };
            if parent != motion_root && !candidates.contains(&parent) {
                reverse.clear();
                break;
            }
            reverse.push(parent);
            current = parent;
        }
        if reverse.is_empty() || current != motion_root {
            continue;
        }
        reverse.reverse();
        if reverse.len() > best.len()
            || (reverse.len() == best.len() && reverse.as_slice() < best.as_slice())
        {
            best = reverse;
        }
    }
    if best.is_empty() && candidates.contains(&motion_root) {
        best.push(motion_root);
    }
    best
}

fn detect_symmetric_pairs(
    contract: &ReferenceSupermodelMotionContractV2,
    points: &[[f32; 3]],
    bounds: Bounds3V1,
    diagonal: f32,
) -> Vec<ReferenceSupermodelSymmetricPairV1> {
    let center_x = (bounds.min[0] + bounds.max[0]) * 0.5;
    let mut used = BTreeSet::new();
    let mut output = Vec::new();
    for left in contract.nodes.iter().filter(|node| {
        points[node.part_number as usize][0] < center_x && node.parent_part_number.is_some()
    }) {
        let Some((right, score)) = contract
            .nodes
            .iter()
            .filter(|right| {
                !used.contains(&right.part_number)
                    && right.parent_part_number == left.parent_part_number
                    && right.structural_role == left.structural_role
                    && points[right.part_number as usize][0] > center_x
            })
            .map(|right| {
                let left_point = points[left.part_number as usize];
                let right_point = points[right.part_number as usize];
                let mirror_error =
                    ((left_point[0] - center_x).abs() - (right_point[0] - center_x).abs()).abs();
                let axial_error = ((left_point[1] - right_point[1]).powi(2)
                    + (left_point[2] - right_point[2]).powi(2))
                .sqrt();
                (right, mirror_error + axial_error)
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
        else {
            continue;
        };
        if score > diagonal * 0.25 {
            continue;
        }
        used.insert(left.part_number);
        used.insert(right.part_number);
        output.push(ReferenceSupermodelSymmetricPairV1 {
            left_part_number: left.part_number,
            right_part_number: right.part_number,
            parent_part_number: left.parent_part_number.expect("filtered sibling parent"),
            confidence: (1.0 - score / (diagonal * 0.25)).clamp(0.0, 1.0),
        });
    }
    output.sort_by_key(|pair| (pair.parent_part_number, pair.left_part_number));
    output
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String, ReferenceSupermodelGenericErrorV2> {
    let bytes = serde_json::to_vec(value).map_err(|source| {
        error(
            "M2A-REFERENCE-SUPERMODEL-STRUCTURE-SERIALIZE",
            "structuralProfile",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelFittedJointV1 {
    pub part_number: u32,
    pub joint_name: String,
    pub reference_world_position: [f32; 3],
    pub initial_target_world_position: [f32; 3],
    pub target_world_position: [f32; 3],
    pub residual_fraction: f32,
    pub semantic_region: String,
    pub constraint_verdict: String,
    pub provenance: String,
    pub confidence: f32,
    pub constraints: Vec<String>,
}

/// One measurable, topology-derived joint-fit invariant.  Coverage counters
/// deliberately live outside this type: a carrier can be present and animated
/// while still occupying an anatomically invalid target position.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelJointFitConstraintV2 {
    pub code: String,
    pub status: String,
    pub part_numbers: Vec<u32>,
    pub measured_fraction: f32,
    pub allowed_fraction: f32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelJointFitReportV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub content_sha256: String,
    pub structural_profile_sha256: String,
    pub surface_anatomy_sha256: Option<String>,
    pub fitting_surface_vertex_count: usize,
    pub ignored_auxiliary_vertex_count: usize,
    pub carrier_node_count: usize,
    pub skin_joint_count: usize,
    pub fitted_joint_count: usize,
    pub fitted_ground_contact_chain_count: usize,
    pub fitted_appendage_chain_count: usize,
    pub minimum_confidence: f32,
    pub low_confidence_joint_part_numbers: Vec<u32>,
    pub used_aabb_seed_only_count: usize,
    pub constraint_required_count: usize,
    pub constraint_pass_count: usize,
    pub constraint_violations: Vec<ReferenceSupermodelJointFitConstraintV2>,
    pub constraints: Vec<ReferenceSupermodelJointFitConstraintV2>,
    pub joints: Vec<ReferenceSupermodelFittedJointV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSupermodelJointFitArtifactV1 {
    pub fitted_world_positions: Vec<[f32; 3]>,
    pub report: ReferenceSupermodelJointFitReportV1,
}

pub fn fit_reference_supermodel_joints_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
) -> Result<ReferenceSupermodelJointFitArtifactV1, ReferenceSupermodelGenericErrorV2> {
    let profile = build_reference_supermodel_structural_profile_v1(contract)?;
    fit_reference_supermodel_joints_from_positions_v1(
        contract,
        surface_positions,
        surface_positions.len(),
        &profile,
        None,
    )
}

pub fn fit_reference_supermodel_joints_with_anatomy_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    anatomy: &TargetSurfaceAnatomyArtifactV1,
) -> Result<ReferenceSupermodelJointFitArtifactV1, ReferenceSupermodelGenericErrorV2> {
    let fitting_positions = anatomy
        .joint_fit_vertex_indices
        .iter()
        .map(|vertex| surface_positions[*vertex])
        .collect::<Vec<_>>();
    fit_reference_supermodel_joints_from_positions_v1(
        contract,
        &fitting_positions,
        surface_positions.len(),
        structural_profile,
        Some(&anatomy.report),
    )
}

/// Re-runs the same anatomical constraints after hash-bound joint authoring.
/// An override can replace the confidence/provenance of that exact carrier,
/// but it cannot waive an unrelated or global surface-analysis failure.
pub fn revalidate_authored_reference_supermodel_joints_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    anatomy: &TargetSurfaceAnatomyArtifactV1,
    base_report: &ReferenceSupermodelJointFitReportV1,
    authored_fitted_world_positions: &[[f32; 3]],
    authored_part_numbers: &BTreeSet<u32>,
) -> Result<ReferenceSupermodelJointFitReportV1, ReferenceSupermodelGenericErrorV2> {
    if authored_fitted_world_positions.len() != contract.nodes.len()
        || base_report.joints.len() != contract.nodes.len()
        || authored_fitted_world_positions
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORED-JOINT-FIT-INVALID",
            "jointFit.authoring",
            "authored joint validation requires one finite world pivot per exact carrier",
        ));
    }
    if base_report.structural_profile_sha256 != structural_profile.content_sha256
        || base_report.surface_anatomy_sha256.as_deref()
            != Some(anatomy.report.content_sha256.as_str())
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORED-JOINT-FIT-CONTEXT-MISMATCH",
            "jointFit.authoring",
            "the authored rig belongs to a different structural profile or surface anatomy",
        ));
    }
    let reference_worlds = contract_world_matrices(contract)?;
    let reference_points = reference_worlds
        .iter()
        .map(|matrix| [matrix[12], matrix[13], matrix[14]])
        .collect::<Vec<_>>();
    let mut provenance = base_report
        .joints
        .iter()
        .map(|joint| joint.provenance.clone())
        .collect::<Vec<_>>();
    let mut confidence = base_report
        .joints
        .iter()
        .map(|joint| joint.confidence)
        .collect::<Vec<_>>();
    for part in authored_part_numbers {
        if *part as usize >= contract.nodes.len() {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORED-JOINT-NOT-FOUND",
                "jointFit.authoring.partNumbers",
                format!("authored carrier part {part} is absent from the exact topology"),
            ));
        }
        provenance[*part as usize] = "authoring_joint_override".to_owned();
        confidence[*part as usize] = 1.0;
    }
    let constraints = evaluate_joint_fit_constraints_v2(
        contract,
        structural_profile,
        Some(&anatomy.report),
        anatomy.report.authoritative_bounds,
        &reference_points,
        authored_fitted_world_positions,
        structural_profile.ground_contact_chains.len(),
        structural_profile.appendage_chains.len(),
        &provenance,
    );
    let constraint_pass_count = constraints
        .iter()
        .filter(|constraint| constraint.status == "PASS")
        .count();
    let constraint_violations = constraints
        .iter()
        .filter(|constraint| constraint.status != "PASS")
        .cloned()
        .collect::<Vec<_>>();
    const MINIMUM_SEMANTIC_JOINT_CONFIDENCE_V2: f32 = 0.5;
    let low_confidence_joint_part_numbers = contract
        .nodes
        .iter()
        .filter(|node| {
            node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant
                && confidence[node.part_number as usize] < MINIMUM_SEMANTIC_JOINT_CONFIDENCE_V2
        })
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    let minimum_confidence = contract
        .nodes
        .iter()
        .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
        .map(|node| confidence[node.part_number as usize])
        .min_by(f32::total_cmp)
        .unwrap_or(1.0);
    let diagonal = bounds_diagonal(anatomy.report.authoritative_bounds).max(1.0e-6);
    let mut joints = contract
        .nodes
        .iter()
        .map(|node| ReferenceSupermodelFittedJointV1 {
            part_number: node.part_number,
            joint_name: node.name.clone(),
            reference_world_position: reference_points[node.part_number as usize],
            initial_target_world_position: base_report.joints[node.part_number as usize]
                .initial_target_world_position,
            target_world_position: authored_fitted_world_positions[node.part_number as usize],
            residual_fraction: distance(
                authored_fitted_world_positions[node.part_number as usize],
                base_report.joints[node.part_number as usize].target_world_position,
            ) / diagonal,
            semantic_region: node.structural_role.clone(),
            constraint_verdict: "PASS".to_owned(),
            provenance: provenance[node.part_number as usize].clone(),
            confidence: confidence[node.part_number as usize],
            constraints: if authored_part_numbers.contains(&node.part_number) {
                vec![
                    "exact_carrier_topology".to_owned(),
                    "sealed_authoring_override".to_owned(),
                ]
            } else {
                base_report.joints[node.part_number as usize]
                    .constraints
                    .clone()
            },
        })
        .collect::<Vec<_>>();
    for joint in &mut joints {
        if constraints.iter().any(|constraint| {
            constraint.status != "PASS" && constraint.part_numbers.contains(&joint.part_number)
        }) {
            joint.constraint_verdict = "BLOCKED".to_owned();
        }
    }
    let ready = anatomy.report.status == "READY"
        && low_confidence_joint_part_numbers.is_empty()
        && constraint_violations.is_empty();
    let mut report = ReferenceSupermodelJointFitReportV1 {
        schema_version: 2,
        algorithm: "STRUCTURAL_ANATOMICAL_CHAIN_CONSTRAINTS_AUTHORED_V2".to_owned(),
        status: if ready { "READY" } else { "NEEDS_AUTHORING" }.to_owned(),
        content_sha256: String::new(),
        structural_profile_sha256: structural_profile.content_sha256.clone(),
        surface_anatomy_sha256: Some(anatomy.report.content_sha256.clone()),
        fitting_surface_vertex_count: anatomy.joint_fit_vertex_indices.len(),
        ignored_auxiliary_vertex_count: anatomy
            .report
            .surface_vertex_count
            .saturating_sub(anatomy.report.authoritative_surface_vertex_count),
        carrier_node_count: contract.nodes.len(),
        skin_joint_count: contract
            .nodes
            .iter()
            .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
            .count(),
        fitted_joint_count: joints.len(),
        fitted_ground_contact_chain_count: structural_profile.ground_contact_chains.len(),
        fitted_appendage_chain_count: structural_profile.appendage_chains.len(),
        minimum_confidence,
        low_confidence_joint_part_numbers,
        used_aabb_seed_only_count: 0,
        constraint_required_count: constraints.len(),
        constraint_pass_count,
        constraint_violations,
        constraints,
        joints,
    };
    report.content_sha256 = sha256_json(&report)?;
    Ok(report)
}

pub fn reference_supermodel_rig_world_positions_v1(
    rig: &CreatureRigProfileV1,
) -> Result<Vec<[f32; 3]>, ReferenceSupermodelGenericErrorV2> {
    let mut worlds = BTreeMap::<u32, [f32; 16]>::new();
    let mut positions = vec![[0.0; 3]; rig.nodes.len()];
    for node in &rig.nodes {
        if node.id as usize >= positions.len() {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-RIG-NODE-ID-OOB",
                "rig.nodes",
                "target rig node ids must form the exact contiguous carrier inventory",
            ));
        }
        let world = if let Some(parent) = node.parent_id {
            let parent_world = worlds.get(&parent).copied().ok_or_else(|| {
                error(
                    "M2A-REFERENCE-SUPERMODEL-RIG-PARENT-ORDER",
                    format!("rig.nodes[{}].parentId", node.id),
                    "target rig parents must precede children in exact carrier order",
                )
            })?;
            multiply_matrix(parent_world, node.bind_local_matrix)
        } else {
            node.bind_local_matrix
        };
        positions[node.id as usize] = [world[12], world[13], world[14]];
        worlds.insert(node.id, world);
    }
    Ok(positions)
}

/// Describes an application that keeps the selected supermodel's complete
/// carrier bind immutable.  This is deliberately not a retargeting fitter:
/// the owned render surface is registered into the reference coordinate
/// space, while every named carrier remains byte-semantic-equivalent to the
/// inspected supermodel contract.
pub fn build_immutable_reference_supermodel_joint_report_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    anatomy: &TargetSurfaceAnatomyArtifactV1,
    _target_bounds: Bounds3V1,
) -> Result<ReferenceSupermodelJointFitReportV1, ReferenceSupermodelGenericErrorV2> {
    let reference_worlds = contract_world_matrices(contract)?;
    let reference_points = reference_worlds
        .iter()
        .map(|matrix| [matrix[12], matrix[13], matrix[14]])
        .collect::<Vec<_>>();
    let all_parts = contract
        .nodes
        .iter()
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    let constraints = vec![
        ReferenceSupermodelJointFitConstraintV2 {
            code: "IMMUTABLE_CARRIER_INVENTORY".to_owned(),
            status: "PASS".to_owned(),
            part_numbers: all_parts.clone(),
            measured_fraction: contract.nodes.len() as f32,
            allowed_fraction: contract.nodes.len() as f32,
            message: "target carrier inventory is the exact selected-supermodel contract"
                .to_owned(),
        },
        ReferenceSupermodelJointFitConstraintV2 {
            code: "IMMUTABLE_REFERENCE_BIND".to_owned(),
            status: "PASS".to_owned(),
            part_numbers: all_parts,
            measured_fraction: 0.0,
            allowed_fraction: contract.tolerances.bind_max_abs_error,
            message: "every target local bind matrix is copied unchanged from the exact selected-supermodel contract"
                .to_owned(),
        },
        ReferenceSupermodelJointFitConstraintV2 {
            code: "OWNED_MESH_REGISTRATION".to_owned(),
            status: if anatomy.report.status == "READY" {
                "PASS"
            } else {
                "BLOCKED"
            }
            .to_owned(),
            part_numbers: Vec::new(),
            measured_fraction: if anatomy.report.status == "READY" {
                1.0
            } else {
                0.0
            },
            allowed_fraction: 1.0,
            message: "owned mesh anatomy must be ready before weights are derived in immutable reference space"
                .to_owned(),
        },
    ];
    let constraint_pass_count = constraints
        .iter()
        .filter(|constraint| constraint.status == "PASS")
        .count();
    let constraint_violations = constraints
        .iter()
        .filter(|constraint| constraint.status != "PASS")
        .cloned()
        .collect::<Vec<_>>();
    let mut joints = contract
        .nodes
        .iter()
        .map(|node| ReferenceSupermodelFittedJointV1 {
            part_number: node.part_number,
            joint_name: node.name.clone(),
            reference_world_position: reference_points[node.part_number as usize],
            initial_target_world_position: reference_points[node.part_number as usize],
            target_world_position: reference_points[node.part_number as usize],
            residual_fraction: 0.0,
            semantic_region: node.structural_role.clone(),
            constraint_verdict: "PASS".to_owned(),
            provenance: "immutable_reference_bind".to_owned(),
            confidence: 1.0,
            constraints: vec![
                "exact_carrier_topology".to_owned(),
                "immutable_reference_bind".to_owned(),
                "owned_mesh_registered_to_reference_space".to_owned(),
            ],
        })
        .collect::<Vec<_>>();
    if !constraint_violations.is_empty() {
        for joint in &mut joints {
            joint.constraint_verdict = "BLOCKED".to_owned();
        }
    }
    let ready = anatomy.report.status == "READY" && constraint_violations.is_empty();
    let mut report = ReferenceSupermodelJointFitReportV1 {
        schema_version: 3,
        algorithm: "IMMUTABLE_REFERENCE_BIND_V1".to_owned(),
        status: if ready { "READY" } else { "NEEDS_AUTHORING" }.to_owned(),
        content_sha256: String::new(),
        structural_profile_sha256: structural_profile.content_sha256.clone(),
        surface_anatomy_sha256: Some(anatomy.report.content_sha256.clone()),
        fitting_surface_vertex_count: anatomy.joint_fit_vertex_indices.len(),
        ignored_auxiliary_vertex_count: anatomy
            .report
            .surface_vertex_count
            .saturating_sub(anatomy.report.authoritative_surface_vertex_count),
        carrier_node_count: contract.nodes.len(),
        skin_joint_count: contract
            .nodes
            .iter()
            .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
            .count(),
        fitted_joint_count: joints.len(),
        fitted_ground_contact_chain_count: structural_profile.ground_contact_chains.len(),
        fitted_appendage_chain_count: structural_profile.appendage_chains.len(),
        minimum_confidence: 1.0,
        low_confidence_joint_part_numbers: Vec::new(),
        used_aabb_seed_only_count: 0,
        constraint_required_count: constraints.len(),
        constraint_pass_count,
        constraint_violations,
        constraints,
        joints,
    };
    report.content_sha256 = sha256_json(&report)?;
    Ok(report)
}

fn fit_reference_supermodel_joints_from_positions_v1(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
    total_surface_vertex_count: usize,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    surface_anatomy: Option<&crate::reference_supermodel_surface_anatomy::TargetSurfaceAnatomyV1>,
) -> Result<ReferenceSupermodelJointFitArtifactV1, ReferenceSupermodelGenericErrorV2> {
    validate_inputs(contract, surface_positions)?;
    let target_bounds = bounds(surface_positions)?;
    let reference_worlds = contract_world_matrices(contract)?;
    let reference_points = reference_worlds
        .iter()
        .map(|matrix| [matrix[12], matrix[13], matrix[14]])
        .collect::<Vec<_>>();
    let reference_bounds = bounds(&reference_points)?;
    let diagonal = bounds_diagonal(target_bounds).max(1.0e-5);
    let mut fitted = reference_points
        .iter()
        .map(|point| fit_point_between_bounds(*point, reference_bounds, target_bounds))
        .collect::<Vec<_>>();
    let seed = fitted.clone();
    let reference_center_x = (reference_bounds.min[0] + reference_bounds.max[0]) * 0.5;
    let reference_width = (reference_bounds.max[0] - reference_bounds.min[0]).max(1.0e-5);
    let mut provenance = vec!["aabb_seed".to_owned(); contract.nodes.len()];
    let mut confidence = vec![0.0_f32; contract.nodes.len()];
    let mut constraints = vec![vec!["exact_carrier_topology".to_owned()]; contract.nodes.len()];

    // AABB is only a seed.  A robust local surface envelope recentres every
    // deforming pivot inside the nearby owned geometry.  This avoids moving a
    // long-haired, narrow target as if its anatomy filled the global box.
    for (part, node) in contract.nodes.iter().enumerate().skip(1) {
        if node.carrier_class != ReferenceSupermodelCarrierClassV3::SkinRelevant {
            confidence[part] = 1.0;
            provenance[part] = "exact_topology_passive".to_owned();
            continue;
        }
        let (local_center, nearest_distance, sample_count) =
            robust_local_surface_center(surface_positions, seed[part], diagonal);
        let blend = if node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL" {
            0.15
        } else {
            0.55
        };
        for axis in 0..3 {
            fitted[part][axis] = seed[part][axis] * (1.0 - blend) + local_center[axis] * blend;
        }
        if (reference_points[part][0] - reference_center_x).abs() <= reference_width * 0.04 {
            fitted[part][0] = (target_bounds.min[0] + target_bounds.max[0]) * 0.5;
            constraints[part].push("target_symmetry_plane".to_owned());
        }
        provenance[part] = "surface_local_envelope".to_owned();
        confidence[part] = (1.0 - nearest_distance / (diagonal * 0.35)).clamp(0.0, 1.0)
            * (sample_count.min(64) as f32 / 64.0);
        constraints[part].push("local_surface_envelope".to_owned());
    }

    // Source positions are expressed in the target-root local space.  The
    // root remains identity and is restored by the carrier layer.
    fitted[0] = [0.0; 3];
    provenance[0] = "owned_root_identity".to_owned();
    confidence[0] = 1.0;

    if let Some(surface_anatomy) = surface_anatomy {
        fit_central_chain_to_medial_axis(
            contract,
            structural_profile,
            &reference_points,
            surface_anatomy,
            &mut fitted,
            &mut provenance,
            &mut confidence,
            &mut constraints,
        )?;
        fit_branch_hubs_to_medial_axis_v2(
            contract,
            structural_profile,
            &reference_points,
            reference_bounds,
            surface_anatomy,
            &mut fitted,
            &mut provenance,
            &mut confidence,
            &mut constraints,
        )?;
    }

    let (ground_count, ground_parts) = fit_ground_contact_chains(
        contract,
        surface_positions,
        target_bounds,
        &reference_points,
        &mut fitted,
    )?;
    for part in ground_parts {
        provenance[part as usize] = "surface_ground_contact_chain".to_owned();
        confidence[part as usize] = 1.0;
        constraints[part as usize].push("ground_contact".to_owned());
        constraints[part as usize].push("ordered_limb_chain".to_owned());
    }

    let (appendage_count, appendage_parts) = fit_appendage_chains(
        contract,
        surface_positions,
        target_bounds,
        &reference_points,
        &mut fitted,
    )?;
    for part in appendage_parts {
        provenance[part as usize] = "surface_appendage_centerline".to_owned();
        confidence[part as usize] = confidence[part as usize].max(0.75);
        constraints[part as usize].push("ordered_appendage_chain".to_owned());
    }

    // Parent/child pivots must remain distinct.  A collapsed segment makes
    // inherited rotations undefined and is never repaired silently.
    for (part, node) in contract.nodes.iter().enumerate().skip(1) {
        let parent = node
            .parent_part_number
            .expect("non-root carrier has a parent") as usize;
        let length = distance(fitted[parent], fitted[part]);
        if !length.is_finite() || length <= diagonal * 1.0e-5 {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-COLLAPSED",
                format!("jointFit.nodes[{part}]"),
                format!(
                    "fitted joint {:?} collapses onto parent {:?}",
                    node.name, contract.nodes[parent].name
                ),
            ));
        }
        constraints[part].push("positive_segment_length".to_owned());
    }

    let mut joints = contract
        .nodes
        .iter()
        .enumerate()
        .map(|(part, node)| ReferenceSupermodelFittedJointV1 {
            part_number: node.part_number,
            joint_name: node.name.clone(),
            reference_world_position: reference_points[part],
            initial_target_world_position: seed[part],
            target_world_position: fitted[part],
            residual_fraction: distance(seed[part], fitted[part]) / diagonal,
            semantic_region: node.structural_role.clone(),
            constraint_verdict: "PASS".to_owned(),
            provenance: provenance[part].clone(),
            confidence: confidence[part],
            constraints: constraints[part].clone(),
        })
        .collect::<Vec<_>>();
    let minimum_confidence = contract
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
        .map(|(part, _)| confidence[part])
        .min_by(f32::total_cmp)
        .unwrap_or(1.0);
    let used_aabb_seed_only_count = provenance
        .iter()
        .enumerate()
        .filter(|(part, value)| {
            contract.nodes[*part].carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant
                && value.as_str() == "aabb_seed"
        })
        .count();
    const MINIMUM_SEMANTIC_JOINT_CONFIDENCE_V2: f32 = 0.5;
    let low_confidence_joint_part_numbers = contract
        .nodes
        .iter()
        .enumerate()
        .filter(|(part, node)| {
            node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant
                && confidence[*part] < MINIMUM_SEMANTIC_JOINT_CONFIDENCE_V2
        })
        .map(|(_, node)| node.part_number)
        .collect::<Vec<_>>();
    if used_aabb_seed_only_count > 0 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-AMBIGUOUS",
            "jointFit",
            format!(
                "surface anatomy is not sufficient for a safe fit: minimumConfidence={minimum_confidence}, aabbSeedOnly={used_aabb_seed_only_count}"
            ),
        ));
    }
    let joint_constraints = evaluate_joint_fit_constraints_v2(
        contract,
        structural_profile,
        surface_anatomy,
        target_bounds,
        &reference_points,
        &fitted,
        ground_count,
        appendage_count,
        &provenance,
    );
    let constraint_pass_count = joint_constraints
        .iter()
        .filter(|constraint| constraint.status == "PASS")
        .count();
    let constraint_violations = joint_constraints
        .iter()
        .filter(|constraint| constraint.status != "PASS")
        .cloned()
        .collect::<Vec<_>>();
    for joint in &mut joints {
        if joint_constraints.iter().any(|constraint| {
            constraint.status != "PASS" && constraint.part_numbers.contains(&joint.part_number)
        }) {
            joint.constraint_verdict = "BLOCKED".to_owned();
        }
    }
    let anatomy_ready = surface_anatomy.is_some_and(|anatomy| anatomy.status == "READY");
    let fit_ready = anatomy_ready
        && minimum_confidence >= MINIMUM_SEMANTIC_JOINT_CONFIDENCE_V2
        && low_confidence_joint_part_numbers.is_empty()
        && constraint_violations.is_empty();
    let mut report = ReferenceSupermodelJointFitReportV1 {
        schema_version: 2,
        algorithm: "STRUCTURAL_ANATOMICAL_CHAIN_CONSTRAINTS_V2".to_owned(),
        status: if fit_ready {
            "READY"
        } else {
            "NEEDS_AUTHORING"
        }
        .to_owned(),
        content_sha256: String::new(),
        structural_profile_sha256: structural_profile.content_sha256.clone(),
        surface_anatomy_sha256: surface_anatomy.map(|report| report.content_sha256.clone()),
        fitting_surface_vertex_count: surface_positions.len(),
        ignored_auxiliary_vertex_count: total_surface_vertex_count
            .saturating_sub(surface_positions.len()),
        carrier_node_count: contract.nodes.len(),
        skin_joint_count: contract
            .nodes
            .iter()
            .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
            .count(),
        fitted_joint_count: joints.len(),
        fitted_ground_contact_chain_count: ground_count,
        fitted_appendage_chain_count: appendage_count,
        minimum_confidence,
        low_confidence_joint_part_numbers,
        used_aabb_seed_only_count,
        constraint_required_count: joint_constraints.len(),
        constraint_pass_count,
        constraint_violations,
        constraints: joint_constraints,
        joints,
    };
    report.content_sha256 = sha256_json(&report)?;
    Ok(ReferenceSupermodelJointFitArtifactV1 {
        fitted_world_positions: fitted,
        report,
    })
}

#[allow(clippy::too_many_arguments)]
fn evaluate_joint_fit_constraints_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    surface_anatomy: Option<&crate::reference_supermodel_surface_anatomy::TargetSurfaceAnatomyV1>,
    target_bounds: Bounds3V1,
    reference_points: &[[f32; 3]],
    fitted: &[[f32; 3]],
    fitted_ground_contact_chain_count: usize,
    fitted_appendage_chain_count: usize,
    provenance: &[String],
) -> Vec<ReferenceSupermodelJointFitConstraintV2> {
    let diagonal = bounds_diagonal(target_bounds).max(1.0e-6);
    let width = (target_bounds.max[0] - target_bounds.min[0]).max(1.0e-6);
    let height = (target_bounds.max[2] - target_bounds.min[2]).max(1.0e-6);
    let symmetry_plane = surface_anatomy.map_or(
        (target_bounds.min[0] + target_bounds.max[0]) * 0.5,
        |anatomy| anatomy.symmetry_plane_x,
    );
    let mut output = Vec::new();

    output.push(joint_fit_constraint_v2(
        "SURFACE_ANATOMY_READY",
        surface_anatomy.is_some_and(|anatomy| anatomy.status == "READY"),
        Vec::new(),
        surface_anatomy.map_or(0.0, |anatomy| anatomy.symmetry_confidence),
        1.0,
        "joint fitting requires a measured, unambiguous target-surface anatomy",
    ));

    let required_ground = structural_profile.ground_contact_chains.len();
    output.push(joint_fit_constraint_v2(
        "GROUND_CONTACT_CHAIN_COVERAGE",
        fitted_ground_contact_chain_count == required_ground,
        structural_profile
            .ground_contact_chains
            .iter()
            .map(|chain| chain.terminal_part_number)
            .collect(),
        fitted_ground_contact_chain_count as f32,
        required_ground as f32,
        "every topology-derived ground-contact chain must have a distinct fitted terminal",
    ));

    let required_appendages = structural_profile.appendage_chains.len();
    output.push(joint_fit_constraint_v2(
        "APPENDAGE_CHAIN_COVERAGE",
        fitted_appendage_chain_count == required_appendages,
        structural_profile
            .appendage_chains
            .iter()
            .map(|chain| chain.terminal_part_number)
            .collect(),
        fitted_appendage_chain_count as f32,
        required_appendages as f32,
        "every topology-derived appendage chain must have an explicit fitted centerline",
    ));

    let mut invalid_segments = Vec::new();
    let mut minimum_segment_fraction = f32::INFINITY;
    let mut maximum_segment_fraction = 0.0_f32;
    for node in contract.nodes.iter().skip(1) {
        if node.carrier_class != ReferenceSupermodelCarrierClassV3::SkinRelevant {
            continue;
        }
        let Some(parent) = node.parent_part_number else {
            invalid_segments.push(node.part_number);
            continue;
        };
        let fraction =
            distance(fitted[parent as usize], fitted[node.part_number as usize]) / diagonal;
        minimum_segment_fraction = minimum_segment_fraction.min(fraction);
        maximum_segment_fraction = maximum_segment_fraction.max(fraction);
        if !fraction.is_finite() || !(0.005..=0.5).contains(&fraction) {
            invalid_segments.push(node.part_number);
        }
    }
    if !minimum_segment_fraction.is_finite() {
        minimum_segment_fraction = 0.0;
    }
    output.push(joint_fit_constraint_v2(
        "POSITIVE_BOUNDED_SEGMENT_LENGTHS",
        invalid_segments.is_empty(),
        invalid_segments,
        minimum_segment_fraction,
        0.005,
        &format!(
            "skin-relevant target segments must remain positive and bounded; maximumFraction={maximum_segment_fraction}"
        ),
    ));

    let central_parts = structural_profile
        .central_chain_part_numbers
        .iter()
        .copied()
        .filter(|part| {
            contract.nodes[*part as usize].carrier_class
                == ReferenceSupermodelCarrierClassV3::SkinRelevant
        })
        .collect::<Vec<_>>();
    let central_max_lateral_fraction = central_parts
        .iter()
        .map(|part| (fitted[*part as usize][0] - symmetry_plane).abs() / width)
        .fold(0.0_f32, f32::max);
    output.push(joint_fit_constraint_v2(
        "CENTRAL_CHAIN_ON_SYMMETRY_PLANE",
        central_parts.is_empty() || central_max_lateral_fraction <= 0.025,
        central_parts,
        central_max_lateral_fraction,
        0.025,
        "the target central chain must remain on the measured symmetry plane",
    ));

    let mut failed_pairs = BTreeSet::new();
    let mut maximum_pair_residual_fraction = 0.0_f32;
    for pair in &structural_profile.symmetric_pairs {
        let left = fitted[pair.left_part_number as usize];
        let right = fitted[pair.right_part_number as usize];
        let mirrored_left = [2.0 * symmetry_plane - left[0], left[1], left[2]];
        let residual = distance(mirrored_left, right) / diagonal;
        maximum_pair_residual_fraction = maximum_pair_residual_fraction.max(residual);
        let has_opposite_sides = (left[0] - symmetry_plane) * (right[0] - symmetry_plane) < 0.0;
        if residual > 0.015 || !has_opposite_sides {
            failed_pairs.insert(pair.left_part_number);
            failed_pairs.insert(pair.right_part_number);
        }
    }
    output.push(joint_fit_constraint_v2(
        "SYMMETRIC_PAIR_TARGET_ALIGNMENT",
        failed_pairs.is_empty(),
        failed_pairs.into_iter().collect(),
        maximum_pair_residual_fraction,
        0.015,
        "topology-derived left/right pairs must remain mirrored and on opposite sides",
    ));

    let ground_terminals = structural_profile
        .ground_contact_chains
        .iter()
        .map(|chain| chain.terminal_part_number)
        .collect::<Vec<_>>();
    let maximum_ground_height_fraction = ground_terminals
        .iter()
        .map(|part| ((fitted[*part as usize][2] - target_bounds.min[2]) / height).abs())
        .fold(0.0_f32, f32::max);
    output.push(joint_fit_constraint_v2(
        "GROUND_TERMINAL_HEIGHT",
        ground_terminals.is_empty() || maximum_ground_height_fraction <= 0.01,
        ground_terminals.clone(),
        maximum_ground_height_fraction,
        0.01,
        "ground-contact terminals must remain on the measured target ground plane",
    ));
    let mut minimum_ground_separation_fraction = f32::INFINITY;
    for left in 0..ground_terminals.len() {
        for right in (left + 1)..ground_terminals.len() {
            let a = fitted[ground_terminals[left] as usize];
            let b = fitted[ground_terminals[right] as usize];
            let planar = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() / diagonal;
            minimum_ground_separation_fraction = minimum_ground_separation_fraction.min(planar);
        }
    }
    if !minimum_ground_separation_fraction.is_finite() {
        minimum_ground_separation_fraction = 1.0;
    }
    output.push(joint_fit_constraint_v2(
        "GROUND_TERMINALS_DISTINCT",
        ground_terminals.len() < 2 || minimum_ground_separation_fraction >= 0.015,
        ground_terminals,
        minimum_ground_separation_fraction,
        0.015,
        "different ground-contact chains may not collapse onto the same target paw cluster",
    ));

    let central_set = structural_profile
        .central_chain_part_numbers
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let branch_hubs = structural_profile
        .ground_contact_chains
        .iter()
        .chain(&structural_profile.appendage_chains)
        .filter_map(|chain| chain.part_numbers.first().copied())
        .filter_map(|part| contract.nodes[part as usize].parent_part_number)
        .filter(|part| {
            *part != structural_profile.root_part_number
                && !central_set.contains(part)
                && contract.nodes[*part as usize].carrier_class
                    == ReferenceSupermodelCarrierClassV3::SkinRelevant
        })
        .collect::<BTreeSet<_>>();
    let unconstrained_hubs = branch_hubs
        .iter()
        .copied()
        .filter(|part| {
            !matches!(
                provenance[*part as usize].as_str(),
                "surface_medial_branch_hub"
                    | "authoring_joint_override"
                    | "immutable_reference_bind"
            )
        })
        .collect::<Vec<_>>();
    output.push(joint_fit_constraint_v2(
        "BRANCH_HUB_SEMANTIC_LANDMARK",
        unconstrained_hubs.is_empty(),
        unconstrained_hubs,
        branch_hubs.len() as f32,
        branch_hubs.len() as f32,
        "a carrier owning limb or appendage branches must use an explicit medial-body landmark",
    ));

    let mut failed_chain_order = BTreeSet::new();
    let mut worst_negative_progress_fraction = 0.0_f32;
    for chain in structural_profile
        .ground_contact_chains
        .iter()
        .chain(&structural_profile.appendage_chains)
    {
        let Some(&first) = chain.part_numbers.first() else {
            continue;
        };
        let anchor = contract.nodes[first as usize]
            .parent_part_number
            .map(|parent| reference_points[parent as usize])
            .unwrap_or(reference_points[first as usize]);
        let reference_terminal = reference_points[chain.terminal_part_number as usize];
        let direction = normalize(sub(reference_terminal, anchor));
        let reference_diagonal =
            bounds_diagonal(bounds(reference_points).unwrap_or(target_bounds)).max(1.0e-6);
        let mut previous_reference = contract.nodes[first as usize]
            .parent_part_number
            .map(|parent| reference_points[parent as usize])
            .unwrap_or(reference_points[first as usize]);
        let mut previous = contract.nodes[first as usize]
            .parent_part_number
            .map(|parent| fitted[parent as usize])
            .unwrap_or(fitted[first as usize]);
        for part in &chain.part_numbers {
            let reference_progress = dot(
                sub(reference_points[*part as usize], previous_reference),
                direction,
            ) / reference_diagonal;
            let progress = dot(sub(fitted[*part as usize], previous), direction) / diagonal;
            // A hock or another naturally folded source segment may move
            // opposite to the overall branch direction. Preserve that exact
            // topology-derived fold; reject only a reversal absent from the
            // selected reference chain.
            let allowed_negative = if reference_progress < -0.002 {
                (reference_progress * 2.0).min(-0.005)
            } else {
                -0.005
            };
            if progress < allowed_negative {
                failed_chain_order.insert(*part);
                worst_negative_progress_fraction = worst_negative_progress_fraction.min(progress);
            }
            previous_reference = reference_points[*part as usize];
            previous = fitted[*part as usize];
        }
    }
    output.push(joint_fit_constraint_v2(
        "ORDERED_BRANCH_CHAINS",
        failed_chain_order.is_empty(),
        failed_chain_order.into_iter().collect(),
        worst_negative_progress_fraction,
        -0.005,
        "limb and appendage joint order must follow the selected reference branch direction",
    ));

    output
}

fn joint_fit_constraint_v2(
    code: &str,
    pass: bool,
    part_numbers: Vec<u32>,
    measured_fraction: f32,
    allowed_fraction: f32,
    message: &str,
) -> ReferenceSupermodelJointFitConstraintV2 {
    ReferenceSupermodelJointFitConstraintV2 {
        code: code.to_owned(),
        status: if pass { "PASS" } else { "BLOCKED" }.to_owned(),
        part_numbers,
        measured_fraction,
        allowed_fraction,
        message: message.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
fn fit_central_chain_to_medial_axis(
    contract: &ReferenceSupermodelMotionContractV2,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    reference_points: &[[f32; 3]],
    anatomy: &crate::reference_supermodel_surface_anatomy::TargetSurfaceAnatomyV1,
    fitted: &mut [[f32; 3]],
    provenance: &mut [String],
    confidence: &mut [f32],
    constraints: &mut [Vec<String>],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    let central_parts = structural_profile
        .central_chain_part_numbers
        .iter()
        .copied()
        .filter(|part| {
            *part != structural_profile.root_part_number
                && contract.nodes[*part as usize].carrier_class
                    == ReferenceSupermodelCarrierClassV3::SkinRelevant
        })
        .collect::<Vec<_>>();
    if central_parts.len() < 2 {
        return Ok(());
    }
    if anatomy.medial_axis.len() < 2 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-MEDIAL-AXIS-SPARSE",
            "jointFit.centralChain",
            "a reference rig with a central carrier chain requires at least two target medial-axis landmarks",
        ));
    }
    let mut target_axis = anatomy.medial_axis.iter().collect::<Vec<_>>();
    target_axis.sort_by(|left, right| {
        left.position[1]
            .total_cmp(&right.position[1])
            .then(left.landmark_id.cmp(&right.landmark_id))
    });
    let first = central_parts[0];
    let second = central_parts[1];
    let start_parent = contract.nodes[second as usize].parent_part_number;
    let start_siblings = contract
        .nodes
        .iter()
        .filter(|node| {
            node.part_number != second
                && node.parent_part_number == start_parent
                && node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant
        })
        .map(|node| fitted[node.part_number as usize][1])
        .collect::<Vec<_>>();
    let target_start_y = if start_siblings.is_empty() {
        fitted[first as usize][1]
    } else {
        start_siblings.iter().sum::<f32>() / start_siblings.len() as f32
    }
    .clamp(
        target_axis[0].position[1],
        target_axis[target_axis.len() - 1].position[1],
    );
    let forward = reference_points
        [*central_parts.last().expect("non-empty central chain") as usize][1]
        >= reference_points[first as usize][1];
    let target_end_y = if forward {
        target_axis[target_axis.len() - 1].position[1]
    } else {
        target_axis[0].position[1]
    };
    let mut cumulative = vec![0.0_f32; central_parts.len()];
    for index in 1..central_parts.len() {
        cumulative[index] = cumulative[index - 1]
            + distance(
                reference_points[central_parts[index - 1] as usize],
                reference_points[central_parts[index] as usize],
            );
    }
    let total = cumulative.last().copied().unwrap_or(0.0);
    if !total.is_finite() || total <= 1.0e-6 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-MEDIAL-AXIS-DEGENERATE",
            "jointFit.centralChain",
            "reference central carrier lineage has no usable ordered length",
        ));
    }
    for (chain_index, part) in central_parts.into_iter().enumerate() {
        let progress = cumulative[chain_index] / total;
        let seed_y = target_start_y + (target_end_y - target_start_y) * progress;
        let high = target_axis
            .partition_point(|landmark| landmark.position[1] < seed_y)
            .min(target_axis.len() - 1);
        let low = high.saturating_sub(1);
        let span = target_axis[high].position[1] - target_axis[low].position[1];
        let blend = if high == low || span.abs() <= 1.0e-6 {
            0.0
        } else {
            ((seed_y - target_axis[low].position[1]) / span).clamp(0.0, 1.0)
        };
        let mut position = std::array::from_fn(|axis| {
            target_axis[low].position[axis]
                + (target_axis[high].position[axis] - target_axis[low].position[axis]) * blend
        });
        position[1] = seed_y.clamp(
            target_axis[0].position[1],
            target_axis[target_axis.len() - 1].position[1],
        );
        position[0] = anatomy.symmetry_plane_x;
        fitted[part as usize] = position;
        provenance[part as usize] = "surface_medial_axis".to_owned();
        confidence[part as usize] = (target_axis[low].confidence
            + (target_axis[high].confidence - target_axis[low].confidence) * blend)
            .clamp(0.0, 1.0);
        constraints[part as usize].push("target_medial_axis".to_owned());
        constraints[part as usize].push("ordered_central_chain".to_owned());
        constraints[part as usize].push("topological_branch_anchor".to_owned());
    }
    Ok(())
}

/// Fits on-axis branch hubs (for example a pelvis-like carrier owning rear
/// limbs and an appendage) against the same target medial axis as the main
/// central chain.  The previous generic fitter left such hubs on a local AABB
/// envelope because they are siblings, not members, of the longest central
/// lineage.  That produced formally complete rigs whose most important branch
/// origin was never anatomically constrained.
#[allow(clippy::too_many_arguments)]
fn fit_branch_hubs_to_medial_axis_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    structural_profile: &ReferenceSupermodelStructuralProfileV1,
    reference_points: &[[f32; 3]],
    reference_bounds: Bounds3V1,
    anatomy: &crate::reference_supermodel_surface_anatomy::TargetSurfaceAnatomyV1,
    fitted: &mut [[f32; 3]],
    provenance: &mut [String],
    confidence: &mut [f32],
    constraints: &mut [Vec<String>],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    let central = structural_profile
        .central_chain_part_numbers
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let hubs = structural_profile
        .ground_contact_chains
        .iter()
        .chain(&structural_profile.appendage_chains)
        .filter_map(|chain| chain.part_numbers.first().copied())
        .filter_map(|part| contract.nodes[part as usize].parent_part_number)
        .filter(|part| {
            *part != structural_profile.root_part_number
                && !central.contains(part)
                && contract.nodes[*part as usize].carrier_class
                    == ReferenceSupermodelCarrierClassV3::SkinRelevant
        })
        .collect::<BTreeSet<_>>();
    if hubs.is_empty() {
        return Ok(());
    }
    if anatomy.medial_axis.len() < 2 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-BRANCH-HUB-AXIS-SPARSE",
            "jointFit.branchHubs",
            "a branch-owning carrier requires a target medial axis",
        ));
    }
    let mut target_axis = anatomy.medial_axis.iter().collect::<Vec<_>>();
    target_axis.sort_by(|left, right| {
        left.position[1]
            .total_cmp(&right.position[1])
            .then(left.landmark_id.cmp(&right.landmark_id))
    });
    let reference_extent = (reference_bounds.max[1] - reference_bounds.min[1]).max(1.0e-6);
    let target_min = target_axis[0].position[1];
    let target_max = target_axis[target_axis.len() - 1].position[1];
    for part in hubs.iter().copied() {
        let progress = ((reference_points[part as usize][1] - reference_bounds.min[1])
            / reference_extent)
            .clamp(0.0, 1.0);
        let target_y = target_min + (target_max - target_min) * progress;
        let (position, landmark_confidence) =
            interpolate_medial_axis_v2(&target_axis, target_y, anatomy.symmetry_plane_x);
        fitted[part as usize] = position;
        provenance[part as usize] = "surface_medial_branch_hub".to_owned();
        confidence[part as usize] = landmark_confidence;
        constraints[part as usize].push("target_medial_axis".to_owned());
        constraints[part as usize].push("topological_branch_hub".to_owned());
        constraints[part as usize].push("target_symmetry_plane".to_owned());
    }
    Ok(())
}

fn interpolate_medial_axis_v2(
    target_axis: &[&crate::reference_supermodel_surface_anatomy::TargetSurfaceLandmarkV1],
    target_y: f32,
    symmetry_plane_x: f32,
) -> ([f32; 3], f32) {
    let high = target_axis
        .partition_point(|landmark| landmark.position[1] < target_y)
        .min(target_axis.len() - 1);
    let low = high.saturating_sub(1);
    let span = target_axis[high].position[1] - target_axis[low].position[1];
    let blend = if high == low || span.abs() <= 1.0e-6 {
        0.0
    } else {
        ((target_y - target_axis[low].position[1]) / span).clamp(0.0, 1.0)
    };
    let mut position = std::array::from_fn(|axis| {
        target_axis[low].position[axis]
            + (target_axis[high].position[axis] - target_axis[low].position[axis]) * blend
    });
    position[0] = symmetry_plane_x;
    position[1] = target_y.clamp(
        target_axis[0].position[1],
        target_axis[target_axis.len() - 1].position[1],
    );
    let confidence = target_axis[low].confidence
        + (target_axis[high].confidence - target_axis[low].confidence) * blend;
    (position, confidence.clamp(0.0, 1.0))
}

fn fit_ground_contact_chains(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    bounds: Bounds3V1,
    reference_points: &[[f32; 3]],
    fitted: &mut [[f32; 3]],
) -> Result<(usize, BTreeSet<u32>), ReferenceSupermodelGenericErrorV2> {
    let terminals = contract
        .nodes
        .iter()
        .filter(|node| node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL")
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    if terminals.is_empty() {
        return Ok((0, BTreeSet::new()));
    }
    let height = (bounds.max[2] - bounds.min[2]).max(1.0e-5);
    let diagonal = bounds_diagonal(bounds).max(1.0e-5);
    let ground_ceiling = bounds.min[2] + height * 0.18;
    let candidates = positions
        .iter()
        .copied()
        .filter(|point| point[2] <= ground_ceiling)
        .collect::<Vec<_>>();
    if candidates.len() < terminals.len() * 4 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-GROUND-CONTACTS-MISSING",
            "surface.groundContacts",
            format!(
                "found {} low surface points for {} required terminals",
                candidates.len(),
                terminals.len()
            ),
        ));
    }
    let mut clusters = vec![Vec::<[f32; 3]>::new(); terminals.len()];
    for point in candidates {
        let cluster = terminals
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                planar_distance_squared(point, fitted[**left as usize])
                    .total_cmp(&planar_distance_squared(point, fitted[**right as usize]))
            })
            .map(|(index, _)| index)
            .expect("ground terminal set is non-empty");
        clusters[cluster].push(point);
    }
    let children = child_counts(contract);
    let mut fitted_count = 0;
    let mut touched = BTreeSet::new();
    for (terminal, mut cluster) in terminals.into_iter().zip(clusters) {
        if cluster.len() < 4 {
            continue;
        }
        cluster.sort_by(|left, right| {
            planar_distance_squared(*left, fitted[terminal as usize])
                .total_cmp(&planar_distance_squared(*right, fitted[terminal as usize]))
        });
        let keep = cluster.len().clamp(8, 512);
        cluster.truncate(keep);
        let count = cluster.len() as f64;
        let contact = [
            (cluster.iter().map(|point| f64::from(point[0])).sum::<f64>() / count) as f32,
            (cluster.iter().map(|point| f64::from(point[1])).sum::<f64>() / count) as f32,
            // Keep the carrier pivot slightly above the measured sole plane
            // to avoid numerical clipping, but inside the 1% anatomical
            // ground-contact contract.  The previous 1.5% literal made the
            // fitter deterministically reject every otherwise-correct paw.
            bounds.min[2] + height * 0.005,
        ];
        let chain = terminal_chain(contract, terminal, &children);
        fit_chain_between_endpoints(
            contract,
            reference_points,
            fitted,
            &chain,
            contact,
            diagonal,
        )?;
        touched.extend(chain.iter().copied());
        fitted_count += 1;
    }
    Ok((fitted_count, touched))
}

fn fit_appendage_chains(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
    bounds: Bounds3V1,
    reference_points: &[[f32; 3]],
    fitted: &mut [[f32; 3]],
) -> Result<(usize, BTreeSet<u32>), ReferenceSupermodelGenericErrorV2> {
    let children = child_counts(contract);
    let terminals = contract
        .nodes
        .iter()
        .filter(|node| {
            node.structural_role == "APPENDAGE_TERMINAL"
                || (is_appendage_semantic(node)
                    && !contract.nodes.iter().any(|child| {
                        child.parent_part_number == Some(node.part_number)
                            && is_appendage_semantic(child)
                    }))
        })
        .map(|node| node.part_number)
        .collect::<BTreeSet<_>>();
    let diagonal = bounds_diagonal(bounds).max(1.0e-5);
    let mut touched = BTreeSet::new();
    let mut fitted_count = 0;
    for terminal in terminals {
        if contract.nodes[terminal as usize].structural_role == "LIMB_GROUND_CONTACT_TERMINAL" {
            continue;
        }
        let chain = terminal_chain(contract, terminal, &children);
        if chain.is_empty() {
            continue;
        }
        let branch = contract.nodes[chain[0] as usize]
            .parent_part_number
            .map(|part| fitted[part as usize])
            .unwrap_or(fitted[chain[0] as usize]);
        let reference_direction = normalize(sub(
            reference_points[terminal as usize],
            reference_points[chain[0] as usize],
        ));
        let mut projected = positions
            .iter()
            .copied()
            .map(|point| (dot(sub(point, branch), reference_direction), point))
            .filter(|(projection, _)| projection.is_finite() && *projection > 0.0)
            .collect::<Vec<_>>();
        if projected.is_empty() {
            continue;
        }
        projected.sort_by(|left, right| right.0.total_cmp(&left.0));
        let keep = projected.len().min(256).max(1);
        let terminal_center = midpoint_bounds(
            &projected
                .iter()
                .take(keep)
                .map(|(_, point)| *point)
                .collect::<Vec<_>>(),
        )?;
        fit_chain_between_endpoints(
            contract,
            reference_points,
            fitted,
            &chain,
            terminal_center,
            diagonal,
        )?;
        touched.extend(chain.iter().copied());
        fitted_count += 1;
    }
    Ok((fitted_count, touched))
}

fn is_appendage_semantic(
    node: &crate::reference_supermodel_motion::ReferenceSupermodelMotionNodeV2,
) -> bool {
    node.structural_role.starts_with("APPENDAGE")
        || node
            .anchor_role
            .as_deref()
            .is_some_and(|role| role.starts_with("tail_") || role.starts_with("appendage_"))
}

fn terminal_chain(
    contract: &ReferenceSupermodelMotionContractV2,
    terminal: u32,
    children: &[usize],
) -> Vec<u32> {
    let mut chain = vec![terminal];
    let mut current = terminal;
    while let Some(parent) = contract.nodes[current as usize].parent_part_number {
        if contract.nodes[parent as usize].carrier_class
            != ReferenceSupermodelCarrierClassV3::SkinRelevant
            || children[parent as usize] > 1
        {
            break;
        }
        chain.push(parent);
        current = parent;
    }
    chain.reverse();
    chain
}

fn fit_chain_between_endpoints(
    contract: &ReferenceSupermodelMotionContractV2,
    reference_points: &[[f32; 3]],
    fitted: &mut [[f32; 3]],
    chain: &[u32],
    terminal: [f32; 3],
    diagonal: f32,
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    let Some(&first) = chain.first() else {
        return Ok(());
    };
    let anchor = contract.nodes[first as usize]
        .parent_part_number
        .map(|parent| fitted[parent as usize])
        .unwrap_or(fitted[first as usize]);
    let reference_anchor = contract.nodes[first as usize]
        .parent_part_number
        .map(|parent| reference_points[parent as usize])
        .unwrap_or(reference_points[first as usize]);
    let reference_chain = chain
        .iter()
        .map(|part| reference_points[*part as usize])
        .collect::<Vec<_>>();
    let Some(transformed) =
        shape_preserving_chain_points(reference_anchor, &reference_chain, anchor, terminal)
    else {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-CHAIN-DEGENERATE",
            "jointFit.chains",
            "reference or target chain has zero usable length",
        ));
    };
    if distance(anchor, terminal) <= diagonal * 1.0e-5 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-CHAIN-DEGENERATE",
            "jointFit.chains",
            "reference or target chain has zero usable length",
        ));
    }
    for (&part, position) in chain.iter().zip(transformed) {
        fitted[part as usize] = position;
    }
    Ok(())
}

fn shape_preserving_chain_points(
    reference_anchor: [f32; 3],
    reference_chain: &[[f32; 3]],
    target_anchor: [f32; 3],
    target_terminal: [f32; 3],
) -> Option<Vec<[f32; 3]>> {
    let reference_terminal = *reference_chain.last()?;
    let reference_vector = sub(reference_terminal, reference_anchor);
    let target_vector = sub(target_terminal, target_anchor);
    let reference_length = dot(reference_vector, reference_vector).sqrt();
    let target_length = dot(target_vector, target_vector).sqrt();
    if !reference_length.is_finite()
        || !target_length.is_finite()
        || reference_length <= 1.0e-8
        || target_length <= 1.0e-8
    {
        return None;
    }
    let reference_direction = reference_vector.map(|component| component / reference_length);
    let target_direction = target_vector.map(|component| component / target_length);
    let scale = target_length / reference_length;
    let mut output = reference_chain
        .iter()
        .map(|point| {
            let local = sub(*point, reference_anchor);
            let rotated = rotate_between_directions(local, reference_direction, target_direction);
            std::array::from_fn(|axis| target_anchor[axis] + rotated[axis] * scale)
        })
        .collect::<Vec<_>>();
    *output.last_mut()? = target_terminal;
    output
        .iter()
        .flatten()
        .all(|value| value.is_finite())
        .then_some(output)
}

fn rotate_between_directions(value: [f32; 3], from: [f32; 3], to: [f32; 3]) -> [f32; 3] {
    let cosine = dot(from, to).clamp(-1.0, 1.0);
    if cosine >= 1.0 - 1.0e-6 {
        return value;
    }
    if cosine <= -1.0 + 1.0e-6 {
        let basis = if from[0].abs() <= from[1].abs() && from[0].abs() <= from[2].abs() {
            [1.0, 0.0, 0.0]
        } else if from[1].abs() <= from[2].abs() {
            [0.0, 1.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
        let axis = normalize(cross(from, basis));
        let projection = dot(axis, value);
        return std::array::from_fn(|component| {
            2.0 * projection * axis[component] - value[component]
        });
    }
    let raw_axis = cross(from, to);
    let sine = dot(raw_axis, raw_axis).sqrt();
    let axis = raw_axis.map(|component| component / sine);
    let axis_cross_value = cross(axis, value);
    let axis_projection = dot(axis, value);
    std::array::from_fn(|component| {
        value[component] * cosine
            + axis_cross_value[component] * sine
            + axis[component] * axis_projection * (1.0 - cosine)
    })
}

fn robust_local_surface_center(
    positions: &[[f32; 3]],
    seed: [f32; 3],
    diagonal: f32,
) -> ([f32; 3], f32, usize) {
    let radius_squared = (diagonal * 0.12).powi(2);
    let mut candidates = positions
        .iter()
        .copied()
        .map(|point| (squared_distance(point, seed), point))
        .filter(|(distance, _)| *distance <= radius_squared)
        .collect::<Vec<_>>();
    if candidates.len() < 32 {
        candidates = positions
            .iter()
            .copied()
            .map(|point| (squared_distance(point, seed), point))
            .collect();
    }
    candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
    candidates.truncate(candidates.len().min(512).max(1));
    let nearest = candidates.first().map_or(diagonal, |row| row.0.sqrt());
    let points = candidates
        .iter()
        .map(|(_, point)| *point)
        .collect::<Vec<_>>();
    (
        midpoint_bounds(&points).unwrap_or(seed),
        nearest,
        candidates.len(),
    )
}

fn validate_inputs(
    contract: &ReferenceSupermodelMotionContractV2,
    positions: &[[f32; 3]],
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    if contract.nodes.is_empty() || contract.nodes[0].parent_part_number.is_some() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
            "contract.nodes",
            "joint fitting requires one ordered carrier root",
        ));
    }
    if positions.is_empty() || positions.iter().flatten().any(|value| !value.is_finite()) {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INVALID",
            "surface.positions",
            "joint fitting requires finite owned surface positions",
        ));
    }
    for (part, node) in contract.nodes.iter().enumerate() {
        if node.part_number as usize != part
            || (part > 0
                && node
                    .parent_part_number
                    .is_none_or(|parent| parent as usize >= part))
        {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
                format!("contract.nodes[{part}]"),
                "carrier parts must be dense and parents must precede children",
            ));
        }
    }
    Ok(())
}

fn contract_world_matrices(
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<Vec<[f32; 16]>, ReferenceSupermodelGenericErrorV2> {
    let mut worlds = Vec::with_capacity(contract.nodes.len());
    for node in &contract.nodes {
        let world = node
            .parent_part_number
            .map(|parent| multiply_matrix(worlds[parent as usize], node.carrier_bind_local_matrix))
            .unwrap_or(node.carrier_bind_local_matrix);
        if world.iter().any(|value| !value.is_finite()) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-REFERENCE-BIND-INVALID",
                "contract.nodes.carrierBindLocalMatrix",
                "reference carrier world matrices must be finite",
            ));
        }
        worlds.push(world);
    }
    Ok(worlds)
}

fn child_counts(contract: &ReferenceSupermodelMotionContractV2) -> Vec<usize> {
    let mut children = vec![0; contract.nodes.len()];
    for node in &contract.nodes {
        if let Some(parent) = node.parent_part_number {
            children[parent as usize] += 1;
        }
    }
    children
}

fn bounds(points: &[[f32; 3]]) -> Result<Bounds3V1, ReferenceSupermodelGenericErrorV2> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    if min.iter().chain(max.iter()).any(|value| !value.is_finite()) {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-BOUNDS-INVALID",
            "surface.positions",
            "surface bounds must be finite",
        ));
    }
    Ok(Bounds3V1 { min, max })
}

fn midpoint_bounds(points: &[[f32; 3]]) -> Result<[f32; 3], ReferenceSupermodelGenericErrorV2> {
    let bounds = bounds(points)?;
    Ok(std::array::from_fn(|axis| {
        (bounds.min[axis] + bounds.max[axis]) * 0.5
    }))
}

fn fit_point_between_bounds(point: [f32; 3], source: Bounds3V1, target: Bounds3V1) -> [f32; 3] {
    std::array::from_fn(|axis| {
        let source_extent = source.max[axis] - source.min[axis];
        let normalized = if source_extent.abs() > 1.0e-6 {
            (point[axis] - source.min[axis]) / source_extent
        } else {
            0.5
        };
        target.min[axis] + normalized.clamp(0.0, 1.0) * (target.max[axis] - target.min[axis])
    })
}

fn bounds_diagonal(bounds: Bounds3V1) -> f32 {
    distance(bounds.min, bounds.max)
}

fn multiply_matrix(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    output
}

fn planar_distance_squared(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2)
}

fn squared_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2)
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    squared_distance(left, right).sqrt()
}

fn sub(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|axis| left[axis] - right[axis])
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = dot(value, value).sqrt();
    if length > 1.0e-8 {
        value.map(|component| component / length)
    } else {
        [0.0, 1.0, 0.0]
    }
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
mod tests {
    use super::*;
    use crate::reference_supermodel_motion::{
        ReferenceSupermodelMotionNodeV2, default_reference_supermodel_motion_tolerances_v2,
    };

    fn branched_contract() -> ReferenceSupermodelMotionContractV2 {
        let node = |part, parent| ReferenceSupermodelMotionNodeV2 {
            part_number: part,
            name: format!("joint_{part}"),
            parent_part_number: parent,
            carrier_bind_local_matrix: [
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                part as f32,
                0.0,
                1.0,
            ],
            position_controller_required: part == 1,
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
            contract_id: "branched".to_owned(),
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
                node(2, Some(1)),
                node(3, Some(2)),
                node(4, Some(1)),
            ],
            carrier_exclusions: Vec::new(),
            required_clips: Vec::new(),
            required_events: Vec::new(),
            tolerances: default_reference_supermodel_motion_tolerances_v2(),
        }
    }

    #[test]
    fn central_profile_is_one_real_lineage_not_a_flattened_sibling_list() {
        let contract = branched_contract();
        let candidates = BTreeSet::from([1, 2, 3, 4]);

        assert_eq!(
            longest_rooted_candidate_chain(1, &candidates, &contract),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn endpoint_fit_preserves_reference_chain_shape_instead_of_straightening_it() {
        let fitted = shape_preserving_chain_points(
            [0.0, 0.0, 0.0],
            &[[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
        )
        .unwrap();

        assert!(distance(fitted[1], [0.0, 2.0, 0.0]) <= 1.0e-5);
        assert!(fitted[0][0].abs() > 0.5);
        assert!((distance([0.0, 0.0, 0.0], fitted[0]) - 2.0_f32.sqrt()).abs() <= 1.0e-5);
        assert!((distance(fitted[0], fitted[1]) - 2.0_f32.sqrt()).abs() <= 1.0e-5);
    }
}
