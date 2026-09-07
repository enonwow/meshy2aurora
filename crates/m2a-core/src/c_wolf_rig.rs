//! Construction of a `c_wolf`-compatible quadruped rig.
//!
//! Source geometry and skin weights remain owner-authored. The production V3
//! inherited-motion contract reads exact neutral carrier matrices from the
//! inspected supermodel because NWN controller inheritance depends on that
//! structural bind metadata. No retail render geometry, skin weights,
//! controller keys or animation payload are copied into the output model.

use std::{collections::BTreeMap, fmt};

#[cfg(test)]
use std::collections::HashMap;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    glb::{GlbIngestResult, IrTransform},
    mdl::InspectionReport,
    profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1, RigSegmentDeformationV1,
        RigWeightInfluenceV1, canonical_profile_sha256,
    },
    reference_supermodel_motion::{
        REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2, ReferenceSupermodelCarrierClassV3,
        ReferenceSupermodelExactContractOptionsV3, ReferenceSupermodelMotionContractV2,
        ReferenceSupermodelMotionNodeV2, ReferenceSupermodelRejectedBaselineV1,
        ReferenceSupermodelRequiredEventV2, ReferenceSupermodelSemanticNodeV3,
        build_exact_reference_supermodel_motion_contract_v3,
        canonical_reference_supermodel_motion_contract_sha256_v2,
        default_reference_supermodel_motion_tolerances_v2,
    },
    reference_supermodel_product::{
        ReferenceSupermodelAppearanceProfileV1, ReferenceSupermodelRigProfileV1,
        ReferenceSupermodelSurfaceContinuityPolicyV1,
    },
};

pub const C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION: u32 = 1;

pub const C_WOLF_REFERENCE_PROFILE_ID_V1: &str = "nwn-c-wolf-reference-rig-v1";

pub fn rejected_c_wolf_v8_surface_baseline_v1() -> ReferenceSupermodelRejectedBaselineV1 {
    ReferenceSupermodelRejectedBaselineV1 {
        model_sha256: "a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f".to_owned(),
        visible_surface_semantic_sha256:
            "ff75e44d8c903e68fdded68061c594013374cd4255b9b31b356a9c77e64ab15a".to_owned(),
    }
}

/// Family adapter consumed by the common reference-supermodel application
/// contract. All retail-family knowledge remains here, outside the shared
/// chain/admission/packaging layer.
pub fn c_wolf_reference_supermodel_profile_v1() -> ReferenceSupermodelRigProfileV1 {
    ReferenceSupermodelRigProfileV1 {
        schema_version: 1,
        profile_id: C_WOLF_REFERENCE_PROFILE_ID_V1.to_owned(),
        supermodel_resref: "c_wolf".to_owned(),
        motion_provider_resref: "c_wolf".to_owned(),
        visible_tail_roles: vec!["tail_base".to_owned(), "tail_tip".to_owned()],
        required_tail_clips: vec!["cpause1".to_owned(), "cwalk".to_owned(), "crun".to_owned()],
        surface_continuity: ReferenceSupermodelSurfaceContinuityPolicyV1 {
            fail_on_any_seam_violation: true,
        },
        appearance: ReferenceSupermodelAppearanceProfileV1 {
            retained_physical_rows: 848,
            donor_physical_row: 176,
            expected_donor_model_type: "S".to_owned(),
            expected_donor_race: "c_dog".to_owned(),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfCompatibleRigErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for CWolfCompatibleRigErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for CWolfCompatibleRigErrorV1 {}

#[derive(Clone, Copy)]
struct NodeSpec {
    name: &'static str,
    parent: Option<u32>,
    normalized_world: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfContactClusterV2 {
    pub role: String,
    pub sample_count: usize,
    pub centroid_world: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfCompatibleRigFitReportV2 {
    pub schema_version: u32,
    pub landmark_algorithm: String,
    pub weight_algorithm: String,
    pub rootdummy_world: [f32; 3],
    pub contact_clusters: Vec<CWolfContactClusterV2>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CWolfCompatibleRigArtifactV2 {
    pub rig: CreatureRigProfileV1,
    pub fit_report: CWolfCompatibleRigFitReportV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfBonePrimaryRegionV3 {
    pub bone_node_id: u32,
    pub bone_name: String,
    pub primary_vertex_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfCompatibleRigFitReportV3 {
    pub schema_version: u32,
    pub landmark_algorithm: String,
    pub weight_algorithm: String,
    pub rootdummy_world: [f32; 3],
    pub contact_clusters: Vec<CWolfContactClusterV2>,
    pub primary_regions: Vec<CWolfBonePrimaryRegionV3>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CWolfCompatibleRigArtifactV3 {
    pub rig: CreatureRigProfileV1,
    pub fit_report: CWolfCompatibleRigFitReportV3,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfComponentCoherenceReportV4 {
    pub component_count: usize,
    pub tiny_triangle_component_count: usize,
    pub locked_component_count: usize,
    pub locked_component_vertex_count: usize,
    pub largest_component_vertex_count: usize,
    pub multi_primary_component_count_before: usize,
    pub multi_primary_component_count_after: usize,
    pub unlocked_components: Vec<CWolfUnlockedComponentV4>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfUnlockedComponentV4 {
    pub triangle_count: usize,
    pub vertex_count: usize,
    pub bounds: Bounds3V1,
    pub extent: f32,
    pub primary_bone_node_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfWeightSmoothingReportV4 {
    pub iteration_count: usize,
    pub max_adjacent_weight_l1_before: f32,
    pub max_adjacent_weight_l1_after: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfForcedComponentAssignmentV4 {
    pub bone_node_id: u32,
    pub bone_name: String,
    pub component_vertex_count: usize,
    pub component_centroid: [f32; 3],
    pub distance_to_bone: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfCompatibleRigFitReportV4 {
    pub schema_version: u32,
    pub landmark_algorithm: String,
    pub weight_algorithm: String,
    pub handedness: String,
    pub rootdummy_world: [f32; 3],
    pub contact_clusters: Vec<CWolfContactClusterV2>,
    pub primary_regions: Vec<CWolfBonePrimaryRegionV3>,
    pub component_coherence: CWolfComponentCoherenceReportV4,
    pub weight_smoothing: CWolfWeightSmoothingReportV4,
    pub forced_primary_assignments: Vec<CWolfForcedComponentAssignmentV4>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CWolfCompatibleRigArtifactV4 {
    pub rig: CreatureRigProfileV1,
    pub fit_report: CWolfCompatibleRigFitReportV4,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfDeformationCageReportV5 {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub body_ring_count: usize,
    pub leg_ring_count: usize,
    pub transfer_algorithm: String,
    pub render_component_count: usize,
    pub coherent_component_count: usize,
    pub coherent_component_vertex_count: usize,
    pub weight_smoothing: CWolfWeightSmoothingReportV4,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CWolfCompatibleRigFitReportV5 {
    pub schema_version: u32,
    pub landmark_algorithm: String,
    pub weight_algorithm: String,
    pub handedness: String,
    pub rootdummy_world: [f32; 3],
    pub contact_clusters: Vec<CWolfContactClusterV2>,
    pub primary_regions: Vec<CWolfBonePrimaryRegionV3>,
    pub deformation_cage: CWolfDeformationCageReportV5,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CWolfCompatibleRigArtifactV5 {
    pub rig: CreatureRigProfileV1,
    pub fit_report: CWolfCompatibleRigFitReportV5,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(test)]
pub struct CWolfCompatibleRigFitReportV6 {
    pub schema_version: u32,
    pub base_fit: CWolfCompatibleRigFitReportV5,
    pub motion_weight_smoothing: CWolfWeightSmoothingReportV4,
    pub seam_weight_smoothing: CWolfSeamWeightSmoothingReportV6,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(test)]
pub struct CWolfSeamWeightSmoothingReportV6 {
    pub iteration_count: usize,
    pub seam_pair_count: usize,
    pub max_pair_weight_l1_before: f32,
    pub max_pair_weight_l1_after: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg(test)]
pub struct CWolfCompatibleRigArtifactV6 {
    pub rig: CreatureRigProfileV1,
    pub fit_report: CWolfCompatibleRigFitReportV6,
}

/// Ordered clean-room compatibility topology for the retail `c_wolf`
/// supermodel. Part numbers are the array indices.
pub fn c_wolf_compatibility_topology_v1() -> Vec<(u32, &'static str, Option<u32>)> {
    c_wolf_node_specs()
        .iter()
        .enumerate()
        .map(|(part, spec)| (part as u32, spec.name, spec.parent))
        .collect()
}

/// Independently authored semantic motion contract for the exact base-game
/// `c_wolf` reference model. The carrier dimensions and joint axes below are
/// product-owned clean-room data; no retail bind transform, controller key or
/// animation payload is returned or embedded in an output model.
pub fn build_c_wolf_motion_contract_v2(
    source_model_sha256: &str,
) -> Result<ReferenceSupermodelMotionContractV2, CWolfCompatibleRigErrorV1> {
    if source_model_sha256.len() != 64
        || !source_model_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(error(
            "M2A-CWOLF-MOTION-SOURCE-HASH-INVALID",
            "sourceModelSha256",
            "the inspected read-only c_wolf source requires a lowercase SHA-256",
        ));
    }

    let specs = c_wolf_node_specs();
    // Fixed clean-room carrier envelope: 1.0 m wide, 1.5 m long and 1.0 m
    // high. It intentionally does not depend on the imported dog's bounds.
    let worlds = specs
        .iter()
        .map(|spec| {
            [
                spec.normalized_world[0] - 0.5,
                (spec.normalized_world[1] - 0.5) * 1.5,
                spec.normalized_world[2],
            ]
        })
        .collect::<Vec<_>>();
    let required_clips = [
        "cpause1",
        "cwalk",
        "crun",
        "ca1slashl",
        "ca1slashr",
        "ca1stab",
        "ckdbck",
        "cdead",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let nodes = specs
        .iter()
        .enumerate()
        .map(|(part, spec)| {
            let parent_world = spec
                .parent
                .map(|parent| worlds[parent as usize])
                .unwrap_or([0.0; 3]);
            let position_controller_required = part == 1;
            let orientation_controller_required =
                (1..=10).contains(&part) || (13..=27).contains(&part);
            ReferenceSupermodelMotionNodeV2 {
                part_number: part as u32,
                name: spec.name.to_owned(),
                parent_part_number: spec.parent,
                carrier_bind_local_matrix: translation_matrix([
                    worlds[part][0] - parent_world[0],
                    worlds[part][1] - parent_world[1],
                    worlds[part][2] - parent_world[2],
                ]),
                position_controller_required,
                orientation_controller_required,
                scale_controller_required: false,
                anchor_role: (part > 0
                    && (position_controller_required || orientation_controller_required))
                    .then(|| {
                        c_wolf_anchor_role(part)
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("joint_{part}"))
                    }),
                joint_axis: orientation_controller_required
                    .then_some(c_wolf_clean_room_joint_axis(part)),
                carrier_class: if part > 0
                    && (position_controller_required || orientation_controller_required)
                {
                    ReferenceSupermodelCarrierClassV3::SkinRelevant
                } else {
                    ReferenceSupermodelCarrierClassV3::PassiveStructural
                },
                structural_role: if part == 0 {
                    "PASSIVE_STRUCTURAL"
                } else if position_controller_required || orientation_controller_required {
                    "ANIMATED_CHAIN_JOINT"
                } else {
                    "PASSIVE_ATTACHMENT_OR_END"
                }
                .to_owned(),
                controlling_clips: if position_controller_required
                    || orientation_controller_required
                {
                    required_clips.clone()
                } else {
                    Vec::new()
                },
                dynamic_clips: if part > 0
                    && (position_controller_required || orientation_controller_required)
                {
                    required_clips.clone()
                } else {
                    Vec::new()
                },
            }
        })
        .collect::<Vec<_>>();
    let clean_room_profile_sha256 = {
        let bytes = serde_json::to_vec(&nodes).map_err(|cause| {
            error(
                "M2A-CWOLF-MOTION-PROFILE-SERIALIZE",
                "nodes",
                cause.to_string(),
            )
        })?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    };
    let mut tolerances = default_reference_supermodel_motion_tolerances_v2();
    // Owner V8 proof showed that a density-normalized allowance can hide a
    // plainly visible split surface. The c_wolf export adapter is therefore a
    // zero-visible-seam profile; the legacy allowed count remains diagnostic.
    tolerances.fail_on_any_seam_violation = true;
    let mut contract = ReferenceSupermodelMotionContractV2 {
        schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
        contract_id: "base-nwn-c-wolf-motion-v2".to_owned(),
        content_sha256: String::new(),
        supermodel_resref: "c_wolf".to_owned(),
        source_model_sha256: source_model_sha256.to_owned(),
        inspected_read_only: true,
        no_payload_copied: true,
        classification: 4,
        animation_scale: 1.0,
        clean_room_profile_id: "m2a-c-wolf-clean-room-carrier-v2".to_owned(),
        clean_room_profile_sha256,
        nodes,
        carrier_exclusions: Vec::new(),
        required_clips,
        required_events: [
            ("cwalk", "snd_footstep"),
            ("crun", "snd_footstep"),
            ("ca1slashl", "hit"),
            ("ca1slashr", "hit"),
            ("ca1stab", "hit"),
            ("ckdbck", "snd_hitground"),
        ]
        .into_iter()
        .map(
            |(clip_name, event_name)| ReferenceSupermodelRequiredEventV2 {
                clip_name: clip_name.to_owned(),
                event_name: event_name.to_owned(),
                minimum_count: 1,
            },
        )
        .collect(),
        tolerances,
    };
    contract.content_sha256 = canonical_reference_supermodel_motion_contract_sha256_v2(&contract)
        .map_err(|cause| {
        error(
            "M2A-CWOLF-MOTION-CONTRACT-HASH",
            "contract.contentSha256",
            cause.to_string(),
        )
    })?;
    Ok(contract)
}

/// Builds the production c_wolf contract from the exact inspected neutral
/// carrier pose. The older V2 builder remains an annotation/topology template
/// for backwards-compatible callers; it is not used by the materializer.
pub fn build_c_wolf_motion_contract_exact_v3(
    source_model_sha256: &str,
    reference: &InspectionReport,
) -> Result<ReferenceSupermodelMotionContractV2, CWolfCompatibleRigErrorV1> {
    let semantic_template = build_c_wolf_motion_contract_v2(source_model_sha256)?;
    let semantic_nodes = semantic_template
        .nodes
        .iter()
        .map(|node| ReferenceSupermodelSemanticNodeV3 {
            node_name: node.name.clone(),
            anchor_role: node.anchor_role.clone(),
            joint_axis: node.joint_axis,
        })
        .collect();
    build_exact_reference_supermodel_motion_contract_v3(
        reference,
        &ReferenceSupermodelExactContractOptionsV3 {
            contract_id: "base-nwn-c-wolf-exact-motion-v3".to_owned(),
            supermodel_resref: "c_wolf".to_owned(),
            source_model_sha256: source_model_sha256.to_owned(),
            required_clips: semantic_template.required_clips,
            required_events: semantic_template.required_events,
            semantic_nodes,
            tolerances: semantic_template.tolerances,
        },
    )
    .map_err(|cause| {
        error(
            "M2A-CWOLF-EXACT-MOTION-CONTRACT",
            cause.path,
            format!("{}: {}", cause.code, cause.message),
        )
    })
}

fn c_wolf_anchor_role(part: usize) -> Option<&'static str> {
    match part {
        0 => Some("root"),
        1 => Some("motion_root"),
        2 => Some("torso"),
        6 => Some("left_front_paw"),
        7 => Some("neck"),
        8 => Some("head"),
        16 => Some("right_front_paw"),
        17 => Some("pelvis"),
        18 => Some("tail_base"),
        19 => Some("tail_tip"),
        23 => Some("left_rear_paw"),
        27 => Some("right_rear_paw"),
        _ => None,
    }
}

fn c_wolf_clean_room_joint_axis(part: usize) -> [f32; 3] {
    if (3..=6).contains(&part) || (13..=16).contains(&part) || (20..=27).contains(&part) {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Builds a one-surface owned quadruped rig for an unskinned, unanimated,
/// identity-instanced Meshy GLB whose authored forward axis is glTF +Z.
pub fn build_c_wolf_compatible_rig_v1(
    source: &GlbIngestResult,
) -> Result<CreatureRigProfileV1, CWolfCompatibleRigErrorV1> {
    validate_source_shape(source)?;
    let primitive = &source.ir.primitives[0];

    let mut target_positions = primitive
        .positions
        .iter()
        .copied()
        .map(gltf_positive_z_to_aurora_positive_y)
        .collect::<Vec<_>>();
    let raw_bounds = bounds(&target_positions)?;
    let center_x = (raw_bounds.min[0] + raw_bounds.max[0]) * 0.5;
    let center_y = (raw_bounds.min[1] + raw_bounds.max[1]) * 0.5;
    let ground_z = raw_bounds.min[2];
    for point in &mut target_positions {
        point[0] -= center_x;
        point[1] -= center_y;
        point[2] -= ground_z;
    }
    let target_bounds = bounds(&target_positions)?;
    let size = [
        target_bounds.max[0] - target_bounds.min[0],
        target_bounds.max[1] - target_bounds.min[1],
        target_bounds.max[2] - target_bounds.min[2],
    ];
    if size
        .iter()
        .any(|extent| !extent.is_finite() || *extent <= 1.0e-6)
    {
        return Err(error(
            "M2A-CWOLF-RIG-DEGENERATE-BOUNDS",
            "source.ir.primitives[0].positions",
            "c_wolf rig construction requires non-degenerate width, length and height",
        ));
    }

    let specs = c_wolf_node_specs();
    let world_positions = specs
        .iter()
        .map(|spec| normalized_world_position(target_bounds, spec.normalized_world))
        .collect::<Vec<_>>();
    let nodes = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let world = world_positions[index];
            let parent_world = spec
                .parent
                .map(|parent| world_positions[parent as usize])
                .unwrap_or([0.0; 3]);
            CreatureRigNodeV1 {
                id: index as u32,
                name: spec.name.to_owned(),
                parent_id: spec.parent,
                bind_local_matrix: translation_matrix([
                    world[0] - parent_world[0],
                    world[1] - parent_world[1],
                    world[2] - parent_world[2],
                ]),
            }
        })
        .collect::<Vec<_>>();

    let reference_weights = target_positions
        .iter()
        .copied()
        .map(|position| procedural_weights(position, target_bounds))
        .collect::<Vec<_>>();
    let allowed_bone_node_ids = (2..=27).collect::<Vec<_>>();
    let segment = CreatureRigSegmentV1 {
        id: 100,
        name: "owned_borzoi_skin".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 0,
        surface_positions: target_positions,
        surface_indices: primitive.indices.clone(),
        allowed_bone_node_ids,
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION,
        profile_id: "owned-borzoi-c-wolf-compatible-rig-v1".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|cause| {
        error(
            "M2A-CWOLF-RIG-HASH-FAILED",
            "rig.contentSha256",
            cause.to_string(),
        )
    })?;
    Ok(rig)
}

/// Source-fitted V2 rig used after the first owner NWN quality result. V2
/// preserves the V1 topology but fits paw chains to actual ground-contact
/// clusters, authors a source-scaled neutral root frame and guards torso/fur
/// vertices from broad leg-box assignment.
pub fn build_c_wolf_compatible_rig_v2(
    source: &GlbIngestResult,
) -> Result<CWolfCompatibleRigArtifactV2, CWolfCompatibleRigErrorV1> {
    validate_source_shape(source)?;
    let primitive = &source.ir.primitives[0];
    let mut target_positions = primitive
        .positions
        .iter()
        .copied()
        .map(gltf_positive_z_to_aurora_positive_y)
        .collect::<Vec<_>>();
    let raw_bounds = bounds(&target_positions)?;
    let center_x = (raw_bounds.min[0] + raw_bounds.max[0]) * 0.5;
    let center_y = (raw_bounds.min[1] + raw_bounds.max[1]) * 0.5;
    let ground_z = raw_bounds.min[2];
    for point in &mut target_positions {
        point[0] -= center_x;
        point[1] -= center_y;
        point[2] -= ground_z;
    }
    let target_bounds = bounds(&target_positions)?;
    require_non_degenerate_bounds(target_bounds)?;

    let mut specs = c_wolf_node_specs();
    let contact_clusters = fit_source_landmarks_v2(&mut specs, &target_positions, target_bounds)?;
    let world_positions = specs
        .iter()
        .map(|spec| normalized_world_position(target_bounds, spec.normalized_world))
        .collect::<Vec<_>>();
    let nodes = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let world = world_positions[index];
            let parent_world = spec
                .parent
                .map(|parent| world_positions[parent as usize])
                .unwrap_or([0.0; 3]);
            CreatureRigNodeV1 {
                id: index as u32,
                name: spec.name.to_owned(),
                parent_id: spec.parent,
                bind_local_matrix: translation_matrix([
                    world[0] - parent_world[0],
                    world[1] - parent_world[1],
                    world[2] - parent_world[2],
                ]),
            }
        })
        .collect::<Vec<_>>();
    let reference_weights = target_positions
        .iter()
        .copied()
        .map(|position| procedural_weights_v2(position, target_bounds, &world_positions))
        .collect::<Vec<_>>();
    let segment = CreatureRigSegmentV1 {
        id: 100,
        name: "owned_borzoi_skin_v2".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 0,
        surface_positions: target_positions,
        surface_indices: primitive.indices.clone(),
        allowed_bone_node_ids: (2..=27).collect(),
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION,
        profile_id: "owned-borzoi-c-wolf-source-fitted-rig-v2".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|cause| {
        error(
            "M2A-CWOLF-RIG-HASH-FAILED",
            "rig.contentSha256",
            cause.to_string(),
        )
    })?;
    Ok(CWolfCompatibleRigArtifactV2 {
        fit_report: CWolfCompatibleRigFitReportV2 {
            schema_version: 2,
            landmark_algorithm: "SOURCE_CONTACT_CLUSTER_CENTROIDS_V2".to_owned(),
            weight_algorithm: "CHAIN_DISTANCE_WITH_TORSO_FUR_GUARD_V2".to_owned(),
            rootdummy_world: world_positions[1],
            contact_clusters,
        },
        rig,
    })
}

/// Source-fitted V3 rig admitted by the owner-reported V2 quality failure.
/// V3 retains the source-fitted landmarks but replaces nearest-two-node leg
/// weighting with continuous shoulder-to-paw chain parameterization and
/// refuses to emit a rig unless every leg bone owns a primary vertex region.
pub fn build_c_wolf_compatible_rig_v3(
    source: &GlbIngestResult,
) -> Result<CWolfCompatibleRigArtifactV3, CWolfCompatibleRigErrorV1> {
    validate_source_shape(source)?;
    let primitive = &source.ir.primitives[0];
    let mut target_positions = primitive
        .positions
        .iter()
        .copied()
        .map(gltf_positive_z_to_aurora_positive_y)
        .collect::<Vec<_>>();
    let raw_bounds = bounds(&target_positions)?;
    let center_x = (raw_bounds.min[0] + raw_bounds.max[0]) * 0.5;
    let center_y = (raw_bounds.min[1] + raw_bounds.max[1]) * 0.5;
    let ground_z = raw_bounds.min[2];
    for point in &mut target_positions {
        point[0] -= center_x;
        point[1] -= center_y;
        point[2] -= ground_z;
    }
    let target_bounds = bounds(&target_positions)?;
    require_non_degenerate_bounds(target_bounds)?;

    let mut specs = c_wolf_node_specs();
    let contact_clusters = fit_source_landmarks_v2(&mut specs, &target_positions, target_bounds)?;
    let world_positions = specs
        .iter()
        .map(|spec| normalized_world_position(target_bounds, spec.normalized_world))
        .collect::<Vec<_>>();
    let nodes = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let world = world_positions[index];
            let parent_world = spec
                .parent
                .map(|parent| world_positions[parent as usize])
                .unwrap_or([0.0; 3]);
            CreatureRigNodeV1 {
                id: index as u32,
                name: spec.name.to_owned(),
                parent_id: spec.parent,
                bind_local_matrix: translation_matrix([
                    world[0] - parent_world[0],
                    world[1] - parent_world[1],
                    world[2] - parent_world[2],
                ]),
            }
        })
        .collect::<Vec<_>>();
    let reference_weights = target_positions
        .iter()
        .copied()
        .map(|position| procedural_weights_v3(position, target_bounds, &world_positions))
        .collect::<Vec<_>>();
    let primary_regions = primary_leg_regions_v3(&reference_weights, &nodes)?;
    let segment = CreatureRigSegmentV1 {
        id: 100,
        name: "owned_borzoi_skin_v3".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 0,
        surface_positions: target_positions,
        surface_indices: primitive.indices.clone(),
        allowed_bone_node_ids: (2..=27).collect(),
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION,
        profile_id: "owned-borzoi-c-wolf-continuous-chain-rig-v3".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|cause| {
        error(
            "M2A-CWOLF-RIG-HASH-FAILED",
            "rig.contentSha256",
            cause.to_string(),
        )
    })?;
    Ok(CWolfCompatibleRigArtifactV3 {
        fit_report: CWolfCompatibleRigFitReportV3 {
            schema_version: 3,
            landmark_algorithm: "SOURCE_CONTACT_CLUSTER_CENTROIDS_V2".to_owned(),
            weight_algorithm: "CONTINUOUS_SHOULDER_TO_PAW_CHAIN_V3".to_owned(),
            rootdummy_world: world_positions[1],
            contact_clusters,
            primary_regions,
        },
        rig,
    })
}

/// Quality-remediation rig admitted by the owner-reported V3 deformation
/// failure. V4 corrects Aurora left/right handedness, fits the neutral root and
/// major quadruped pivots in source space, and gives small disconnected fur
/// islands one coherent weight signature so they cannot stretch between bones.
pub fn build_c_wolf_compatible_rig_v4(
    source: &GlbIngestResult,
) -> Result<CWolfCompatibleRigArtifactV4, CWolfCompatibleRigErrorV1> {
    validate_source_shape(source)?;
    let primitive = &source.ir.primitives[0];
    let mut target_positions = primitive
        .positions
        .iter()
        .copied()
        .map(gltf_positive_z_to_aurora_positive_y)
        .collect::<Vec<_>>();
    let raw_bounds = bounds(&target_positions)?;
    let center_x = (raw_bounds.min[0] + raw_bounds.max[0]) * 0.5;
    let center_y = (raw_bounds.min[1] + raw_bounds.max[1]) * 0.5;
    let ground_z = raw_bounds.min[2];
    for point in &mut target_positions {
        point[0] -= center_x;
        point[1] -= center_y;
        point[2] -= ground_z;
    }
    let target_bounds = bounds(&target_positions)?;
    require_non_degenerate_bounds(target_bounds)?;

    let mut specs = c_wolf_node_specs();
    let contact_clusters = fit_source_landmarks_v4(&mut specs, &target_positions, target_bounds)?;
    let world_positions = specs
        .iter()
        .map(|spec| normalized_world_position(target_bounds, spec.normalized_world))
        .collect::<Vec<_>>();
    let nodes = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let world = world_positions[index];
            let parent_world = spec
                .parent
                .map(|parent| world_positions[parent as usize])
                .unwrap_or([0.0; 3]);
            CreatureRigNodeV1 {
                id: index as u32,
                name: spec.name.to_owned(),
                parent_id: spec.parent,
                bind_local_matrix: translation_matrix([
                    world[0] - parent_world[0],
                    world[1] - parent_world[1],
                    world[2] - parent_world[2],
                ]),
            }
        })
        .collect::<Vec<_>>();
    let mut reference_weights = target_positions
        .iter()
        .copied()
        .map(|position| procedural_weights_v4(position, target_bounds, &world_positions))
        .collect::<Vec<_>>();
    let component_coherence = apply_component_coherence_v4(
        &target_positions,
        &primitive.indices,
        &mut reference_weights,
    )?;
    let forced_primary_assignments = force_leg_component_coverage_v4(
        &target_positions,
        &primitive.indices,
        &world_positions,
        &nodes,
        &mut reference_weights,
    )?;
    let weight_smoothing =
        smooth_connected_weights_v4(&primitive.indices, &mut reference_weights, 0)?;
    let primary_regions = primary_leg_regions_v3(&reference_weights, &nodes)?;
    if let Some(region) = primary_regions
        .iter()
        .find(|region| region.primary_vertex_count < 256)
    {
        return Err(error(
            "M2A-CWOLF-RIG-V4-PRIMARY-REGION-TOO-SMALL",
            format!("rig.nodes[{}]", region.bone_node_id),
            format!(
                "V4 requires at least 256 primary vertices for {}, got {}",
                region.bone_name, region.primary_vertex_count
            ),
        ));
    }
    let segment = CreatureRigSegmentV1 {
        id: 100,
        name: "owned_borzoi_skin_v4".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 0,
        surface_positions: target_positions,
        surface_indices: primitive.indices.clone(),
        allowed_bone_node_ids: (2..=27).collect(),
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION,
        profile_id: "owned-borzoi-c-wolf-component-coherent-rig-v4".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|cause| {
        error(
            "M2A-CWOLF-RIG-HASH-FAILED",
            "rig.contentSha256",
            cause.to_string(),
        )
    })?;
    Ok(CWolfCompatibleRigArtifactV4 {
        fit_report: CWolfCompatibleRigFitReportV4 {
            schema_version: 4,
            landmark_algorithm: "SOURCE_ANATOMICAL_PIVOTS_AND_CONTACT_CLUSTERS_V4".to_owned(),
            weight_algorithm: "DISCONNECTED_COMPONENT_DOMINANT_BONE_V4".to_owned(),
            handedness: "AURORA_NEGATIVE_X_LEFT_V4".to_owned(),
            rootdummy_world: world_positions[1],
            contact_clusters,
            primary_regions,
            component_coherence,
            weight_smoothing,
            forced_primary_assignments,
        },
        rig,
    })
}

/// Continuous deformation-cage remediation for the owner-reported V3/V4
/// failures. The dense, 1,836-component fur shell is never weighted directly.
/// A compact owned quadruped cage carries smooth anatomical weights and Profile
/// A transfers them barycentrically to the render surface.
pub fn build_c_wolf_compatible_rig_v5(
    source: &GlbIngestResult,
) -> Result<CWolfCompatibleRigArtifactV5, CWolfCompatibleRigErrorV1> {
    validate_source_shape(source)?;
    let primitive = &source.ir.primitives[0];
    let mut target_positions = primitive
        .positions
        .iter()
        .copied()
        .map(gltf_positive_z_to_aurora_positive_y)
        .collect::<Vec<_>>();
    let raw_bounds = bounds(&target_positions)?;
    let center_x = (raw_bounds.min[0] + raw_bounds.max[0]) * 0.5;
    let center_y = (raw_bounds.min[1] + raw_bounds.max[1]) * 0.5;
    let ground_z = raw_bounds.min[2];
    for point in &mut target_positions {
        point[0] -= center_x;
        point[1] -= center_y;
        point[2] -= ground_z;
    }
    let target_bounds = bounds(&target_positions)?;
    require_non_degenerate_bounds(target_bounds)?;

    let mut specs = c_wolf_node_specs();
    let contact_clusters = fit_source_landmarks_v4(&mut specs, &target_positions, target_bounds)?;
    let world_positions = specs
        .iter()
        .map(|spec| normalized_world_position(target_bounds, spec.normalized_world))
        .collect::<Vec<_>>();
    let nodes = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let world = world_positions[index];
            let parent_world = spec
                .parent
                .map(|parent| world_positions[parent as usize])
                .unwrap_or([0.0; 3]);
            CreatureRigNodeV1 {
                id: index as u32,
                name: spec.name.to_owned(),
                parent_id: spec.parent,
                bind_local_matrix: translation_matrix([
                    world[0] - parent_world[0],
                    world[1] - parent_world[1],
                    world[2] - parent_world[2],
                ]),
            }
        })
        .collect::<Vec<_>>();

    let width = target_bounds.max[0] - target_bounds.min[0];
    let height = target_bounds.max[2] - target_bounds.min[2];
    let mut cage_positions = Vec::new();
    let mut cage_indices = Vec::new();
    let mut cage_weights = Vec::new();

    let body_rings = [
        (19_u32, 0.05, 0.05),
        (18_u32, 0.10, 0.09),
        (17_u32, 0.30, 0.25),
        (2_u32, 0.34, 0.29),
        (7_u32, 0.20, 0.20),
        (8_u32, 0.17, 0.15),
    ];
    let mut previous = None;
    for (bone, width_fraction, _) in body_rings {
        let pair = push_cage_pair_v5(
            &mut cage_positions,
            &mut cage_weights,
            [
                [
                    world_positions[bone as usize][0] - width * width_fraction,
                    world_positions[bone as usize][1],
                    world_positions[bone as usize][2],
                ],
                [
                    world_positions[bone as usize][0] + width * width_fraction,
                    world_positions[bone as usize][1],
                    world_positions[bone as usize][2],
                ],
            ],
            bone,
            None,
        )?;
        if let Some(previous) = previous {
            connect_cage_pairs_v5(&mut cage_indices, previous, pair)?;
        }
        previous = Some(pair);
    }
    let mut previous = None;
    for (bone, _, height_fraction) in body_rings.into_iter().skip(1) {
        let pair = push_cage_pair_v5(
            &mut cage_positions,
            &mut cage_weights,
            [
                [
                    world_positions[bone as usize][0],
                    world_positions[bone as usize][1],
                    world_positions[bone as usize][2] - height * height_fraction,
                ],
                [
                    world_positions[bone as usize][0],
                    world_positions[bone as usize][1],
                    world_positions[bone as usize][2] + height * height_fraction,
                ],
            ],
            bone,
            None,
        )?;
        if let Some(previous) = previous {
            connect_cage_pairs_v5(&mut cage_indices, previous, pair)?;
        }
        previous = Some(pair);
    }

    let leg_chains = [
        ([3_u32, 4, 5, 6], 2_u32),
        ([13_u32, 14, 15, 16], 2_u32),
        ([20_u32, 21, 22, 23], 17_u32),
        ([24_u32, 25, 26, 27], 17_u32),
    ];
    for (chain, torso_bone) in leg_chains {
        let mut previous = None;
        for (ring_index, bone) in chain.into_iter().enumerate() {
            let radius = width * if ring_index == 0 { 0.075 } else { 0.055 };
            let pair = push_cage_pair_v5(
                &mut cage_positions,
                &mut cage_weights,
                [
                    [
                        world_positions[bone as usize][0] - radius,
                        world_positions[bone as usize][1],
                        world_positions[bone as usize][2],
                    ],
                    [
                        world_positions[bone as usize][0] + radius,
                        world_positions[bone as usize][1],
                        world_positions[bone as usize][2],
                    ],
                ],
                bone,
                (ring_index == 0).then_some(torso_bone),
            )?;
            if let Some(previous) = previous {
                connect_cage_pairs_v5(&mut cage_indices, previous, pair)?;
            }
            previous = Some(pair);
        }
    }
    let cage_vertex_count = cage_positions.len();
    let cage_triangle_count = cage_indices.len() / 3;
    let (reference_weights, component_transfer, weight_smoothing) =
        transfer_cage_weights_to_render_v5(
            &target_positions,
            &primitive.indices,
            &cage_positions,
            &cage_indices,
            &cage_weights,
        )?;
    let primary_regions = primary_leg_regions_v3(&reference_weights, &nodes)?;
    let deformation_cage = CWolfDeformationCageReportV5 {
        vertex_count: cage_vertex_count,
        triangle_count: cage_triangle_count,
        body_ring_count: body_rings.len() + body_rings.len() - 1,
        leg_ring_count: leg_chains.len() * 4,
        transfer_algorithm:
            "COMPONENT_AWARE_NEAREST_CAGE_TRIANGLE_BARYCENTRIC_AND_TOPOLOGY_SMOOTH_V5".to_owned(),
        render_component_count: component_transfer.component_count,
        coherent_component_count: component_transfer.coherent_component_count,
        coherent_component_vertex_count: component_transfer.coherent_component_vertex_count,
        weight_smoothing,
    };
    let segment = CreatureRigSegmentV1 {
        id: 100,
        name: "owned_borzoi_deformation_cage_v5".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 0,
        surface_positions: target_positions,
        surface_indices: primitive.indices.clone(),
        allowed_bone_node_ids: (2..=27).collect(),
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION,
        profile_id: "owned-borzoi-c-wolf-deformation-cage-rig-v5".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|cause| {
        error(
            "M2A-CWOLF-RIG-HASH-FAILED",
            "rig.contentSha256",
            cause.to_string(),
        )
    })?;
    Ok(CWolfCompatibleRigArtifactV5 {
        fit_report: CWolfCompatibleRigFitReportV5 {
            schema_version: 5,
            landmark_algorithm: "SOURCE_ANATOMICAL_PIVOTS_AND_CONTACT_CLUSTERS_V5".to_owned(),
            weight_algorithm: "CONTINUOUS_DEFORMATION_CAGE_TRANSFER_V5".to_owned(),
            handedness: "AURORA_NEGATIVE_X_LEFT_V5".to_owned(),
            rootdummy_world: world_positions[1],
            contact_clusters,
            primary_regions,
            deformation_cage,
        },
        rig,
    })
}

/// Exact-supermodel motion preparation for the owned Borzoi surface.
///
/// V5 remains immutable. V6 starts from its continuous deformation-cage
/// transfer and performs additional topology-local diffusion. No vertices,
/// triangles or retail weights are introduced; only the owner-authored weight
/// field is smoothed along edges already present in each render component.
#[cfg(test)]
fn build_c_wolf_compatible_rig_v6(
    source: &GlbIngestResult,
) -> Result<CWolfCompatibleRigArtifactV6, CWolfCompatibleRigErrorV1> {
    const MOTION_SMOOTHING_ITERATIONS_V6: usize = 12;

    let v5 = build_c_wolf_compatible_rig_v5(source)?;
    let mut rig = v5.rig;
    if rig.segments.len() != 1 {
        return Err(error(
            "M2A-CWOLF-RIG-V6-SEGMENT-SHAPE",
            "rig.segments",
            "V6 exact-motion smoothing requires the one-surface V5 rig",
        ));
    }
    let segment = &mut rig.segments[0];
    let motion_weight_smoothing = smooth_connected_weights_v4(
        &segment.surface_indices,
        &mut segment.reference_weights,
        MOTION_SMOOTHING_ITERATIONS_V6,
    )?;
    let seam_weight_smoothing = smooth_cross_component_seam_weights_v6(
        &segment.surface_positions,
        &segment.surface_indices,
        &mut segment.reference_weights,
        0.005,
        12,
    )?;
    let primary_regions = primary_leg_regions_v3(&segment.reference_weights, &rig.nodes)?;

    rig.profile_id = "owned-borzoi-c-wolf-exact-motion-smooth-rig-v6".to_owned();
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|cause| {
        error(
            "M2A-CWOLF-RIG-HASH-FAILED",
            "rig.contentSha256",
            cause.to_string(),
        )
    })?;

    let mut base_fit = v5.fit_report;
    base_fit.primary_regions = primary_regions;
    Ok(CWolfCompatibleRigArtifactV6 {
        rig,
        fit_report: CWolfCompatibleRigFitReportV6 {
            schema_version: 6,
            base_fit,
            motion_weight_smoothing,
            seam_weight_smoothing,
        },
    })
}

#[cfg(test)]
fn smooth_cross_component_seam_weights_v6(
    positions: &[[f32; 3]],
    indices: &[u32],
    weights: &mut [Vec<RigWeightInfluenceV1>],
    threshold_diagonal_fraction: f32,
    iteration_count: usize,
) -> Result<CWolfSeamWeightSmoothingReportV6, CWolfCompatibleRigErrorV1> {
    if positions.len() != weights.len()
        || positions.is_empty()
        || indices.is_empty()
        || !indices.len().is_multiple_of(3)
        || !threshold_diagonal_fraction.is_finite()
        || threshold_diagonal_fraction <= 0.0
    {
        return Err(error(
            "M2A-CWOLF-RIG-SEAM-SMOOTHING-SHAPE",
            "rig.segments[0]",
            "seam smoothing requires paired positions/weights, indexed triangles and a positive finite threshold",
        ));
    }

    let components = raw_connected_components_v4(indices);
    let mut component_by_vertex = vec![usize::MAX; positions.len()];
    for (component_index, component) in components.iter().enumerate() {
        for &vertex in &component.vertex_indices {
            if vertex >= positions.len() || component_by_vertex[vertex] != usize::MAX {
                return Err(error(
                    "M2A-CWOLF-RIG-SEAM-SMOOTHING-COMPONENT",
                    "rig.segments[0].surfaceIndices",
                    "component inventory contains an invalid or duplicate vertex",
                ));
            }
            component_by_vertex[vertex] = component_index;
        }
    }
    if component_by_vertex.contains(&usize::MAX) {
        return Err(error(
            "M2A-CWOLF-RIG-SEAM-SMOOTHING-COMPONENT",
            "rig.segments[0].surfaceIndices",
            "every weighted vertex must belong to one render component",
        ));
    }

    let bounds = bounds(positions)?;
    let diagonal = length_v6([
        bounds.max[0] - bounds.min[0],
        bounds.max[1] - bounds.min[1],
        bounds.max[2] - bounds.min[2],
    ]);
    let threshold = threshold_diagonal_fraction * diagonal;
    if !threshold.is_finite() || threshold <= 1.0e-8 {
        return Err(error(
            "M2A-CWOLF-RIG-SEAM-SMOOTHING-THRESHOLD",
            "rig.targetBounds",
            "seam smoothing threshold is degenerate",
        ));
    }
    let inverse_cell = threshold.recip();
    let mut grid = HashMap::<(i32, i32, i32), Vec<usize>>::new();
    for (index, &point) in positions.iter().enumerate() {
        grid.entry(spatial_cell_v6(point, inverse_cell))
            .or_default()
            .push(index);
    }

    let threshold_squared = threshold * threshold;
    let mut pairs = Vec::new();
    for (index, &point) in positions.iter().enumerate() {
        let base_cell = spatial_cell_v6(point, inverse_cell);
        let mut nearest = None::<(usize, f32)>;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell = (base_cell.0 + dx, base_cell.1 + dy, base_cell.2 + dz);
                    let Some(candidates) = grid.get(&cell) else {
                        continue;
                    };
                    for &candidate in candidates {
                        if candidate <= index
                            || component_by_vertex[candidate] == component_by_vertex[index]
                        {
                            continue;
                        }
                        let distance_squared = squared_distance_v6(point, positions[candidate]);
                        if distance_squared <= threshold_squared
                            && nearest.is_none_or(|(_, best)| distance_squared < best)
                        {
                            nearest = Some((candidate, distance_squared));
                        }
                    }
                }
            }
        }
        if let Some((candidate, _)) = nearest {
            pairs.push((index, candidate));
        }
    }
    pairs.sort_unstable();
    pairs.dedup();

    let max_pair_weight_l1_before = max_weight_l1_for_pairs_v6(&pairs, weights);
    let mut topology_neighbors = vec![Vec::<usize>::new(); weights.len()];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= weights.len()) {
            return Err(error(
                "M2A-CWOLF-RIG-SEAM-SMOOTHING-INDEX",
                "rig.segments[0].surfaceIndices",
                "combined topology/seam graph index exceeds the weight array",
            ));
        }
        for (left, right) in [
            (vertices[0], vertices[1]),
            (vertices[1], vertices[2]),
            (vertices[2], vertices[0]),
        ] {
            if left != right {
                topology_neighbors[left].push(right);
                topology_neighbors[right].push(left);
            }
        }
    }
    for row in &mut topology_neighbors {
        row.sort_unstable();
        row.dedup();
    }

    let mut donor_vertices = vec![Vec::<usize>::new(); weights.len()];
    for &(left, right) in &pairs {
        let left_component = component_by_vertex[left];
        let right_component = component_by_vertex[right];
        let left_size = components[left_component].vertex_indices.len();
        let right_size = components[right_component].vertex_indices.len();
        let (recipient, donor) = if left_size < right_size {
            (left, right)
        } else if right_size < left_size {
            (right, left)
        } else if left_component > right_component {
            (left, right)
        } else {
            (right, left)
        };
        donor_vertices[recipient].push(donor);
    }
    for row in &mut donor_vertices {
        row.sort_unstable();
        row.dedup();
    }
    for _ in 0..iteration_count {
        let previous = weights.to_vec();
        let mut projected = previous.clone();
        for (vertex, donors) in donor_vertices.iter().enumerate() {
            if donors.is_empty() {
                continue;
            }
            let mut totals = BTreeMap::<u32, f64>::new();
            for influence in &previous[vertex] {
                *totals.entry(influence.bone_node_id).or_default() +=
                    f64::from(influence.value) * 0.5;
            }
            let donor_scale = 0.5 / donors.len() as f64;
            for &donor in donors {
                for influence in &previous[donor] {
                    *totals.entry(influence.bone_node_id).or_default() +=
                        f64::from(influence.value) * donor_scale;
                }
            }
            projected[vertex] = normalize_top_four_weights_v5(totals)?;
        }

        let mut next = projected.clone();
        for (vertex, neighbors) in topology_neighbors.iter().enumerate() {
            if neighbors.is_empty() {
                continue;
            }
            let mut totals = BTreeMap::<u32, f64>::new();
            for influence in &projected[vertex] {
                *totals.entry(influence.bone_node_id).or_default() +=
                    f64::from(influence.value) * 0.5;
            }
            let neighbor_scale = 0.5 / neighbors.len() as f64;
            for &neighbor in neighbors {
                for influence in &projected[neighbor] {
                    *totals.entry(influence.bone_node_id).or_default() +=
                        f64::from(influence.value) * neighbor_scale;
                }
            }
            next[vertex] = normalize_top_four_weights_v5(totals)?;
        }
        weights.clone_from_slice(&next);
    }

    Ok(CWolfSeamWeightSmoothingReportV6 {
        iteration_count,
        seam_pair_count: pairs.len(),
        max_pair_weight_l1_before,
        max_pair_weight_l1_after: max_weight_l1_for_pairs_v6(&pairs, weights),
    })
}

#[cfg(test)]
fn spatial_cell_v6(point: [f32; 3], inverse_cell: f32) -> (i32, i32, i32) {
    (
        (point[0] * inverse_cell).floor() as i32,
        (point[1] * inverse_cell).floor() as i32,
        (point[2] * inverse_cell).floor() as i32,
    )
}

#[cfg(test)]
fn squared_distance_v6(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2)
}

#[cfg(test)]
fn length_v6(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

#[cfg(test)]
fn max_weight_l1_for_pairs_v6(
    pairs: &[(usize, usize)],
    weights: &[Vec<RigWeightInfluenceV1>],
) -> f32 {
    pairs.iter().fold(0.0_f32, |maximum, &(left, right)| {
        let mut difference = BTreeMap::<u32, f32>::new();
        for influence in &weights[left] {
            *difference.entry(influence.bone_node_id).or_default() += influence.value;
        }
        for influence in &weights[right] {
            *difference.entry(influence.bone_node_id).or_default() -= influence.value;
        }
        maximum.max(difference.values().map(|value| value.abs()).sum())
    })
}

#[derive(Clone, Copy, Debug)]
struct CageComponentTransferV5 {
    component_count: usize,
    coherent_component_count: usize,
    coherent_component_vertex_count: usize,
}

fn transfer_cage_weights_to_render_v5(
    render_positions: &[[f32; 3]],
    render_indices: &[u32],
    cage_positions: &[[f32; 3]],
    cage_indices: &[u32],
    cage_weights: &[Vec<RigWeightInfluenceV1>],
) -> Result<
    (
        Vec<Vec<RigWeightInfluenceV1>>,
        CageComponentTransferV5,
        CWolfWeightSmoothingReportV4,
    ),
    CWolfCompatibleRigErrorV1,
> {
    if cage_positions.len() != cage_weights.len()
        || cage_indices.is_empty()
        || !cage_indices.len().is_multiple_of(3)
    {
        return Err(error(
            "M2A-CWOLF-CAGE-SHAPE",
            "rig.deformationCage",
            "deformation cage requires paired vertices/weights and indexed triangles",
        ));
    }
    let mut reference_weights = Vec::with_capacity(render_positions.len());
    for &point in render_positions {
        let mut nearest = None::<(f32, usize, [f32; 3])>;
        for (triangle_index, triangle) in cage_indices.chunks_exact(3).enumerate() {
            let indices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            if indices.iter().any(|index| *index >= cage_positions.len()) {
                return Err(error(
                    "M2A-CWOLF-CAGE-INDEX",
                    "rig.deformationCage.indices",
                    "deformation cage index exceeds the vertex array",
                ));
            }
            let Some((distance_squared, barycentric)) = closest_triangle_barycentric_v5(
                point,
                cage_positions[indices[0]],
                cage_positions[indices[1]],
                cage_positions[indices[2]],
            ) else {
                return Err(error(
                    "M2A-CWOLF-CAGE-DEGENERATE",
                    format!("rig.deformationCage.triangles[{triangle_index}]"),
                    "deformation cage triangle is degenerate",
                ));
            };
            if nearest.as_ref().is_none_or(|current| {
                distance_squared < current.0
                    || (distance_squared == current.0 && triangle_index < current.1)
            }) {
                nearest = Some((distance_squared, triangle_index, barycentric));
            }
        }
        let (_, triangle_index, barycentric) = nearest.ok_or_else(|| {
            error(
                "M2A-CWOLF-CAGE-EMPTY",
                "rig.deformationCage",
                "deformation cage contains no usable triangle",
            )
        })?;
        let triangle = &cage_indices[triangle_index * 3..triangle_index * 3 + 3];
        reference_weights.push(interpolate_cage_weights_v5(
            triangle,
            barycentric,
            cage_weights,
        )?);
    }

    let weight_smoothing = smooth_connected_weights_v4(render_indices, &mut reference_weights, 4)?;
    let components = raw_connected_components_v4(render_indices);
    Ok((
        reference_weights,
        CageComponentTransferV5 {
            component_count: components.len(),
            // Meshy fur cards intentionally contain many small, disconnected
            // components. Averaging weights per component (or across coincident
            // vertices) made those cards move as rigid islands and produced the
            // large tears seen in V4. V5 therefore preserves the continuous
            // cage interpolation for every render vertex.
            coherent_component_count: 0,
            coherent_component_vertex_count: 0,
        },
        weight_smoothing,
    ))
}

fn interpolate_cage_weights_v5(
    triangle: &[u32],
    barycentric: [f32; 3],
    cage_weights: &[Vec<RigWeightInfluenceV1>],
) -> Result<Vec<RigWeightInfluenceV1>, CWolfCompatibleRigErrorV1> {
    let mut totals = BTreeMap::<u32, f64>::new();
    for corner in 0..3 {
        let weights = cage_weights.get(triangle[corner] as usize).ok_or_else(|| {
            error(
                "M2A-CWOLF-CAGE-WEIGHT-INDEX",
                "rig.deformationCage.weights",
                "deformation cage weight index exceeds the vertex array",
            )
        })?;
        for influence in weights {
            *totals.entry(influence.bone_node_id).or_default() +=
                f64::from(influence.value) * f64::from(barycentric[corner]);
        }
    }
    normalize_top_four_weights_v5(totals)
}

fn normalize_top_four_weights_v5(
    totals: BTreeMap<u32, f64>,
) -> Result<Vec<RigWeightInfluenceV1>, CWolfCompatibleRigErrorV1> {
    let mut ranked = totals
        .into_iter()
        .filter(|(_, value)| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(4);
    let sum = ranked.iter().map(|(_, value)| *value).sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 {
        return Err(error(
            "M2A-CWOLF-CAGE-WEIGHT-EMPTY",
            "rig.deformationCage.weights",
            "cage transfer produced no positive finite influence",
        ));
    }
    Ok(ranked
        .into_iter()
        .map(|(bone_node_id, value)| influence(bone_node_id, (value / sum) as f32))
        .collect())
}

fn closest_triangle_barycentric_v5(
    point: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> Option<(f32, [f32; 3])> {
    let ab = sub_v5(b, a);
    let ac = sub_v5(c, a);
    let cross = cross_v5(ab, ac);
    if dot_v5(cross, cross) <= 1.0e-12 {
        return None;
    }
    let ap = sub_v5(point, a);
    let d1 = dot_v5(ab, ap);
    let d2 = dot_v5(ac, ap);
    let barycentric = if d1 <= 0.0 && d2 <= 0.0 {
        [1.0, 0.0, 0.0]
    } else {
        let bp = sub_v5(point, b);
        let d3 = dot_v5(ab, bp);
        let d4 = dot_v5(ac, bp);
        if d3 >= 0.0 && d4 <= d3 {
            [0.0, 1.0, 0.0]
        } else {
            let vc = d1 * d4 - d3 * d2;
            if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
                let v = d1 / (d1 - d3);
                [1.0 - v, v, 0.0]
            } else {
                let cp = sub_v5(point, c);
                let d5 = dot_v5(ab, cp);
                let d6 = dot_v5(ac, cp);
                if d6 >= 0.0 && d5 <= d6 {
                    [0.0, 0.0, 1.0]
                } else {
                    let vb = d5 * d2 - d1 * d6;
                    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
                        let w = d2 / (d2 - d6);
                        [1.0 - w, 0.0, w]
                    } else {
                        let va = d3 * d6 - d5 * d4;
                        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
                            let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
                            [0.0, 1.0 - w, w]
                        } else {
                            let denominator = va + vb + vc;
                            if !denominator.is_finite() || denominator.abs() <= f32::EPSILON {
                                return None;
                            }
                            let inverse = 1.0 / denominator;
                            let v = vb * inverse;
                            let w = vc * inverse;
                            [1.0 - v - w, v, w]
                        }
                    }
                }
            }
        }
    };
    let closest = [
        a[0] * barycentric[0] + b[0] * barycentric[1] + c[0] * barycentric[2],
        a[1] * barycentric[0] + b[1] * barycentric[1] + c[1] * barycentric[2],
        a[2] * barycentric[0] + b[2] * barycentric[1] + c[2] * barycentric[2],
    ];
    let delta = sub_v5(point, closest);
    Some((dot_v5(delta, delta), barycentric))
}

fn sub_v5(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn dot_v5(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross_v5(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn push_cage_pair_v5(
    positions: &mut Vec<[f32; 3]>,
    weights: &mut Vec<Vec<RigWeightInfluenceV1>>,
    pair: [[f32; 3]; 2],
    bone: u32,
    torso_bone: Option<u32>,
) -> Result<u32, CWolfCompatibleRigErrorV1> {
    let start = u32::try_from(positions.len()).map_err(|_| {
        error(
            "M2A-CWOLF-CAGE-INDEX-OVERFLOW",
            "rig.segments.surfacePositions",
            "deformation cage vertex index exceeds u32",
        )
    })?;
    positions.extend(pair);
    let row = torso_bone.map_or_else(
        || vec![influence(bone, 1.0)],
        |torso| vec![influence(bone, 0.8), influence(torso, 0.2)],
    );
    weights.extend((0..2).map(|_| row.clone()));
    Ok(start)
}

fn connect_cage_pairs_v5(
    indices: &mut Vec<u32>,
    first: u32,
    second: u32,
) -> Result<(), CWolfCompatibleRigErrorV1> {
    let first_next = first.checked_add(1).ok_or_else(|| {
        error(
            "M2A-CWOLF-CAGE-INDEX-OVERFLOW",
            "rig.segments.surfaceIndices",
            "deformation cage index overflows u32",
        )
    })?;
    let second_next = second.checked_add(1).ok_or_else(|| {
        error(
            "M2A-CWOLF-CAGE-INDEX-OVERFLOW",
            "rig.segments.surfaceIndices",
            "deformation cage index overflows u32",
        )
    })?;
    indices.extend([first, second, first_next, first_next, second, second_next]);
    Ok(())
}

fn primary_leg_regions_v3(
    reference_weights: &[Vec<RigWeightInfluenceV1>],
    nodes: &[CreatureRigNodeV1],
) -> Result<Vec<CWolfBonePrimaryRegionV3>, CWolfCompatibleRigErrorV1> {
    const LEG_BONES: [u32; 16] = [3, 4, 5, 6, 13, 14, 15, 16, 20, 21, 22, 23, 24, 25, 26, 27];
    let mut result = LEG_BONES
        .into_iter()
        .map(|bone_node_id| CWolfBonePrimaryRegionV3 {
            bone_node_id,
            bone_name: nodes[bone_node_id as usize].name.clone(),
            primary_vertex_count: 0,
        })
        .collect::<Vec<_>>();
    for row in reference_weights {
        let Some(primary) = row
            .iter()
            .max_by(|left, right| left.value.total_cmp(&right.value))
        else {
            continue;
        };
        if let Some(region) = result
            .iter_mut()
            .find(|region| region.bone_node_id == primary.bone_node_id)
        {
            region.primary_vertex_count += 1;
        }
    }
    if let Some(region) = result
        .iter()
        .find(|region| region.primary_vertex_count == 0)
    {
        return Err(error(
            "M2A-CWOLF-RIG-PRIMARY-REGION-MISSING",
            format!("rig.nodes[{}]", region.bone_node_id),
            format!(
                "continuous chain rig requires a primary deformation region for {}",
                region.bone_name
            ),
        ));
    }
    Ok(result)
}

fn require_non_degenerate_bounds(bounds: Bounds3V1) -> Result<(), CWolfCompatibleRigErrorV1> {
    let size = [
        bounds.max[0] - bounds.min[0],
        bounds.max[1] - bounds.min[1],
        bounds.max[2] - bounds.min[2],
    ];
    if size
        .iter()
        .any(|extent| !extent.is_finite() || *extent <= 1.0e-6)
    {
        return Err(error(
            "M2A-CWOLF-RIG-DEGENERATE-BOUNDS",
            "source.ir.primitives[0].positions",
            "c_wolf rig construction requires non-degenerate width, length and height",
        ));
    }
    Ok(())
}

fn fit_source_landmarks_v2(
    specs: &mut [NodeSpec; 30],
    positions: &[[f32; 3]],
    bounds: Bounds3V1,
) -> Result<Vec<CWolfContactClusterV2>, CWolfCompatibleRigErrorV1> {
    // `c_wolf` animation translates rootdummy around (-0.39 Y, 0.79 Z) in
    // its neutral clips. Express that neutral frame as source-normalized
    // coordinates so the inherited controller does not lift the owned mesh
    // as soon as animation starts.
    specs[1].normalized_world = [0.5, 0.295, 0.635];

    let cluster_specs = [
        ("left_front_paw", true, 0.56, 0.79, [3, 4, 5, 6]),
        ("right_front_paw", false, 0.56, 0.79, [13, 14, 15, 16]),
        ("left_back_paw", true, 0.05, 0.22, [20, 21, 22, 23]),
        ("right_back_paw", false, 0.05, 0.22, [24, 25, 26, 27]),
    ];
    let x_mid = (bounds.min[0] + bounds.max[0]) * 0.5;
    let mut result = Vec::with_capacity(cluster_specs.len());

    for (role, left, y_min, y_max, chain) in cluster_specs {
        let selected = positions
            .iter()
            .copied()
            .filter(|point| {
                let x_side = if left {
                    point[0] >= x_mid
                } else {
                    point[0] < x_mid
                };
                let y = normalized_axis(point[1], bounds.min[1], bounds.max[1]);
                let z = normalized_axis(point[2], bounds.min[2], bounds.max[2]);
                x_side && y >= y_min && y < y_max && z <= 0.14
            })
            .collect::<Vec<_>>();
        if selected.len() < 8 {
            return Err(error(
                "M2A-CWOLF-RIG-CONTACT-CLUSTER-MISSING",
                format!("source.contactClusters.{role}"),
                format!(
                    "source-fitted rig requires at least 8 ground-contact samples for {role}, found {}",
                    selected.len()
                ),
            ));
        }
        let centroid = centroid(&selected);
        let paw = world_to_normalized(bounds, centroid);
        fit_leg_chain(specs, chain, paw, left, role.contains("front"));
        result.push(CWolfContactClusterV2 {
            role: role.to_owned(),
            sample_count: selected.len(),
            centroid_world: centroid,
        });
    }
    Ok(result)
}

fn fit_source_landmarks_v4(
    specs: &mut [NodeSpec; 30],
    positions: &[[f32; 3]],
    bounds: Bounds3V1,
) -> Result<Vec<CWolfContactClusterV2>, CWolfCompatibleRigErrorV1> {
    // Keep the neutral root close to the inherited controller's source-fitted
    // frame while deriving every persisted transform from the owned surface.
    specs[1].normalized_world = [0.5, 0.295, 0.58];
    specs[2].normalized_world = [0.5, 0.54, 0.60];
    specs[7].normalized_world = [0.5, 0.70, 0.70];
    specs[8].normalized_world = [0.5, 0.82, 0.78];
    specs[17].normalized_world = [0.5, 0.33, 0.56];
    specs[18].normalized_world = [0.5, 0.18, 0.56];
    specs[19].normalized_world = [0.5, 0.05, 0.45];

    let cluster_specs = [
        ("left_front_paw", true, 0.56, 0.79, [3, 4, 5, 6]),
        ("right_front_paw", false, 0.56, 0.79, [13, 14, 15, 16]),
        ("left_back_paw", true, 0.05, 0.22, [20, 21, 22, 23]),
        ("right_back_paw", false, 0.05, 0.22, [24, 25, 26, 27]),
    ];
    let x_mid = (bounds.min[0] + bounds.max[0]) * 0.5;
    let mut result = Vec::with_capacity(cluster_specs.len());
    for (role, left, y_min, y_max, chain) in cluster_specs {
        let selected = positions
            .iter()
            .copied()
            .filter(|point| {
                // Aurora/c_wolf names the negative-X side "left".
                let x_side = if left {
                    point[0] < x_mid
                } else {
                    point[0] >= x_mid
                };
                let y = normalized_axis(point[1], bounds.min[1], bounds.max[1]);
                let z = normalized_axis(point[2], bounds.min[2], bounds.max[2]);
                x_side && y >= y_min && y < y_max && z <= 0.14
            })
            .collect::<Vec<_>>();
        if selected.len() < 8 {
            return Err(error(
                "M2A-CWOLF-RIG-CONTACT-CLUSTER-MISSING",
                format!("source.contactClusters.{role}"),
                format!(
                    "source-fitted rig requires at least 8 ground-contact samples for {role}, found {}",
                    selected.len()
                ),
            ));
        }
        let centroid = centroid(&selected);
        let paw = world_to_normalized(bounds, centroid);
        fit_leg_chain_v4(specs, chain, paw, left, role.contains("front"));
        result.push(CWolfContactClusterV2 {
            role: role.to_owned(),
            sample_count: selected.len(),
            centroid_world: centroid,
        });
    }
    Ok(result)
}

fn fit_leg_chain_v4(
    specs: &mut [NodeSpec; 30],
    chain: [usize; 4],
    paw: [f32; 3],
    left: bool,
    front: bool,
) {
    let side_anchor = if left { 0.36 } else { 0.64 };
    let upper = [
        lerp(0.5, paw[0], 0.72),
        if front { paw[1] - 0.04 } else { paw[1] + 0.08 },
        if front { 0.58 } else { 0.56 },
    ];
    let middle_heights = if front { [0.39, 0.21] } else { [0.38, 0.20] };
    specs[chain[0]].normalized_world = [
        lerp(side_anchor, upper[0], 0.75),
        upper[1].clamp(0.0, 1.0),
        upper[2],
    ];
    specs[chain[1]].normalized_world = [
        lerp(specs[chain[0]].normalized_world[0], paw[0], 0.35),
        lerp(specs[chain[0]].normalized_world[1], paw[1], 0.35),
        middle_heights[0],
    ];
    specs[chain[2]].normalized_world = [
        lerp(specs[chain[1]].normalized_world[0], paw[0], 0.62),
        lerp(specs[chain[1]].normalized_world[1], paw[1], 0.62),
        middle_heights[1],
    ];
    specs[chain[3]].normalized_world = [paw[0], paw[1], paw[2].clamp(0.02, 0.10)];
}

fn centroid(points: &[[f32; 3]]) -> [f32; 3] {
    let mut sum = [0.0_f64; 3];
    for point in points {
        for axis in 0..3 {
            sum[axis] += f64::from(point[axis]);
        }
    }
    let count = points.len() as f64;
    [
        (sum[0] / count) as f32,
        (sum[1] / count) as f32,
        (sum[2] / count) as f32,
    ]
}

fn world_to_normalized(bounds: Bounds3V1, point: [f32; 3]) -> [f32; 3] {
    [
        normalized_axis(point[0], bounds.min[0], bounds.max[0]),
        normalized_axis(point[1], bounds.min[1], bounds.max[1]),
        normalized_axis(point[2], bounds.min[2], bounds.max[2]),
    ]
}

fn fit_leg_chain(
    specs: &mut [NodeSpec; 30],
    chain: [usize; 4],
    paw: [f32; 3],
    left: bool,
    front: bool,
) {
    let torso_y = if front { 0.62 } else { 0.36 };
    let torso_x = if left { 0.64 } else { 0.36 };
    let chain_heights = if front {
        [0.55, 0.38, 0.22]
    } else {
        [0.52, 0.36, 0.21]
    };
    for (step, node_id) in chain[..3].iter().copied().enumerate() {
        let toward_paw = (step as f32 + 1.0) / 4.0;
        specs[node_id].normalized_world = [
            lerp(torso_x, paw[0], toward_paw),
            lerp(torso_y, paw[1], toward_paw),
            chain_heights[step],
        ];
    }
    specs[chain[3]].normalized_world = [paw[0], paw[1], paw[2].clamp(0.02, 0.10)];
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

fn validate_source_shape(source: &GlbIngestResult) -> Result<(), CWolfCompatibleRigErrorV1> {
    if !source.ir.skins.is_empty() || !source.ir.animations.is_empty() {
        return Err(error(
            "M2A-CWOLF-RIG-SOURCE-NOT-STATIC",
            "source.ir",
            "clean-room c_wolf rig construction requires an unskinned, unanimated source",
        ));
    }
    if source.ir.primitives.len() != 1 || source.ir.meshes.len() != 1 {
        return Err(error(
            "M2A-CWOLF-RIG-SOURCE-SHAPE-UNSUPPORTED",
            "source.ir.primitives",
            "version 1 requires exactly one mesh primitive",
        ));
    }
    let primitive = &source.ir.primitives[0];
    if primitive.topology != "TRIANGLES"
        || primitive.positions.is_empty()
        || primitive.indices.is_empty()
        || !primitive.indices.len().is_multiple_of(3)
    {
        return Err(error(
            "M2A-CWOLF-RIG-SOURCE-SHAPE-UNSUPPORTED",
            "source.ir.primitives[0]",
            "version 1 requires a non-empty indexed triangle list",
        ));
    }
    let mesh_nodes = source
        .ir
        .nodes
        .iter()
        .filter(|node| node.mesh_id.is_some())
        .collect::<Vec<_>>();
    if mesh_nodes.len() != 1 {
        return Err(error(
            "M2A-CWOLF-RIG-SOURCE-TRANSFORM-UNSUPPORTED",
            "source.ir.nodes",
            format!(
                "version 1 requires one mesh instance, found {}",
                mesh_nodes.len()
            ),
        ));
    }
    if !identity_transform(&mesh_nodes[0].transform) {
        return Err(error(
            "M2A-CWOLF-RIG-SOURCE-TRANSFORM-UNSUPPORTED",
            "source.ir.nodes",
            format!(
                "version 1 requires an identity-transformed mesh instance, got {:?}",
                mesh_nodes[0].transform
            ),
        ));
    }
    Ok(())
}

fn identity_transform(transform: &IrTransform) -> bool {
    const IDENTITY: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    transform.matrix.is_none_or(|value| value == IDENTITY)
        && transform.translation.is_none_or(|value| value == [0.0; 3])
        && transform
            .rotation
            .is_none_or(|value| value == [0.0, 0.0, 0.0, 1.0])
        && transform.scale.is_none_or(|value| value == [1.0; 3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_identity_matrix_is_accepted() {
        assert!(identity_transform(&IrTransform {
            kind: "MATRIX".to_owned(),
            matrix: Some([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]),
            translation: None,
            rotation: None,
            scale: None,
        }));
    }

    #[test]
    fn v2_fits_paw_chains_to_source_contact_clusters_and_neutral_root() {
        let bounds = Bounds3V1 {
            min: [-1.0, -2.0, 0.0],
            max: [1.0, 2.0, 2.0],
        };
        let points = synthetic_contact_points();
        let mut specs = c_wolf_node_specs();
        let report = fit_source_landmarks_v2(&mut specs, &points, bounds).unwrap();

        assert_eq!(report.len(), 4);
        assert!(report.iter().all(|cluster| cluster.sample_count == 8));
        assert!((specs[6].normalized_world[0] - 0.8).abs() < 1.0e-5);
        assert!((specs[16].normalized_world[0] - 0.2).abs() < 1.0e-5);
        let root = normalized_world_position(bounds, specs[1].normalized_world);
        assert!((root[1] - -0.82).abs() < 1.0e-5);
        assert!((root[2] - 1.27).abs() < 1.0e-5);
    }

    #[test]
    fn v2_weights_keep_center_chest_out_of_leg_chains_and_bind_paw() {
        let bounds = Bounds3V1 {
            min: [-1.0, -2.0, 0.0],
            max: [1.0, 2.0, 2.0],
        };
        let points = synthetic_contact_points();
        let mut specs = c_wolf_node_specs();
        fit_source_landmarks_v2(&mut specs, &points, bounds).unwrap();
        let world_positions = specs
            .iter()
            .map(|spec| normalized_world_position(bounds, spec.normalized_world))
            .collect::<Vec<_>>();

        let chest = procedural_weights_v2([0.0, 0.8, 0.4], bounds, &world_positions);
        assert!(
            chest
                .iter()
                .all(|weight| ![3, 4, 5, 6, 13, 14, 15, 16].contains(&weight.bone_node_id))
        );

        let paw = world_positions[6];
        let paw_weights = procedural_weights_v2(paw, bounds, &world_positions);
        assert!(paw_weights.iter().any(|weight| weight.bone_node_id == 6));
        let sum = paw_weights.iter().map(|weight| weight.value).sum::<f32>();
        assert!((sum - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn v3_continuous_chain_parameterization_gives_every_front_bone_a_primary_region() {
        let bounds = Bounds3V1 {
            min: [-1.0, -2.0, 0.0],
            max: [1.0, 2.0, 2.0],
        };
        let points = synthetic_contact_points();
        let mut specs = c_wolf_node_specs();
        fit_source_landmarks_v2(&mut specs, &points, bounds).unwrap();
        let world_positions = specs
            .iter()
            .map(|spec| normalized_world_position(bounds, spec.normalized_world))
            .collect::<Vec<_>>();

        for chain in [[3, 4, 5, 6], [13, 14, 15, 16]] {
            for bone_node_id in chain {
                let weights = continuous_chain_weights_v3(
                    world_positions[bone_node_id as usize],
                    chain,
                    2,
                    &world_positions,
                );
                let primary = weights
                    .iter()
                    .max_by(|left, right| left.value.total_cmp(&right.value))
                    .unwrap();
                assert_eq!(primary.bone_node_id, bone_node_id);
            }
        }
    }

    #[test]
    fn v4_handedness_places_left_chain_on_negative_aurora_x() {
        let bounds = Bounds3V1 {
            min: [-1.0, -2.0, 0.0],
            max: [1.0, 2.0, 2.0],
        };
        let points = synthetic_contact_points();
        let mut specs = c_wolf_node_specs();
        let report = fit_source_landmarks_v4(&mut specs, &points, bounds).unwrap();

        let left = report
            .iter()
            .find(|cluster| cluster.role == "left_front_paw")
            .unwrap();
        let right = report
            .iter()
            .find(|cluster| cluster.role == "right_front_paw")
            .unwrap();
        assert!(left.centroid_world[0] < 0.0);
        assert!(right.centroid_world[0] > 0.0);
        assert!(specs[3].normalized_world[0] < 0.5);
        assert!(specs[13].normalized_world[0] > 0.5);
        assert!(specs[6].normalized_world[0] < 0.5);
        assert!(specs[16].normalized_world[0] > 0.5);
    }

    #[test]
    fn v4_tiny_components_receive_one_coherent_weight_signature() {
        let positions = vec![
            [-1.0, 0.0, 0.0],
            [-0.8, 0.0, 0.0],
            [-0.9, 0.2, 0.0],
            [0.8, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.9, 0.2, 0.0],
        ];
        let indices = vec![0, 1, 2, 3, 4, 5];
        let mut weights = vec![
            vec![influence(3, 1.0)],
            vec![influence(4, 1.0)],
            vec![influence(5, 1.0)],
            vec![influence(13, 1.0)],
            vec![influence(14, 1.0)],
            vec![influence(15, 1.0)],
        ];
        let report = apply_component_coherence_v4(&positions, &indices, &mut weights).unwrap();

        assert_eq!(report.component_count, 2);
        assert_eq!(report.locked_component_count, 2);
        assert_eq!(report.locked_component_vertex_count, 6);
        assert_eq!(weights[0], weights[1]);
        assert_eq!(weights[1], weights[2]);
        assert_eq!(weights[3], weights[4]);
        assert_eq!(weights[4], weights[5]);
        assert_ne!(weights[0], weights[3]);
    }

    #[test]
    fn v4_topology_smoothing_reduces_abrupt_adjacent_weight_changes() {
        let indices = vec![0, 1, 2, 1, 3, 2];
        let mut weights = vec![
            vec![influence(2, 1.0)],
            vec![influence(2, 1.0)],
            vec![influence(17, 1.0)],
            vec![influence(17, 1.0)],
        ];

        let report = smooth_connected_weights_v4(&indices, &mut weights, 8).unwrap();

        assert_eq!(report.iteration_count, 8);
        assert_eq!(report.max_adjacent_weight_l1_before, 2.0);
        assert!(report.max_adjacent_weight_l1_after < 0.05);
        assert!(weights.iter().all(|row| row.len() == 2));
    }

    #[test]
    fn v6_seam_smoothing_reconciles_nearby_disconnected_surfaces() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2, 3, 4, 5];
        let mut weights = vec![vec![influence(2, 1.0)]; 3];
        weights.extend(vec![vec![influence(7, 1.0)]; 3]);

        let report =
            smooth_cross_component_seam_weights_v6(&positions, &indices, &mut weights, 0.005, 8)
                .unwrap();

        assert_eq!(report.seam_pair_count, 3);
        assert_eq!(report.max_pair_weight_l1_before, 2.0);
        assert!(
            report.max_pair_weight_l1_after < report.max_pair_weight_l1_before,
            "combined graph diffusion must reduce seam weight discontinuity"
        );
        assert!(weights.iter().all(|row| {
            (row.iter().map(|influence| influence.value).sum::<f32>() - 1.0).abs() < 1.0e-6
        }));
    }

    fn synthetic_contact_points() -> Vec<[f32; 3]> {
        let mut result = Vec::new();
        for (x, y) in [(0.6, 0.8), (-0.6, 0.8), (0.6, -1.4), (-0.6, -1.4)] {
            for index in 0..8 {
                let delta = (index as f32 - 3.5) * 0.002;
                result.push([x + delta, y - delta, 0.05]);
            }
        }
        result
    }
}

fn gltf_positive_z_to_aurora_positive_y(point: [f32; 3]) -> [f32; 3] {
    [-point[0], point[2], point[1]]
}

fn bounds(points: &[[f32; 3]]) -> Result<Bounds3V1, CWolfCompatibleRigErrorV1> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in points {
        if point.iter().any(|value| !value.is_finite()) {
            return Err(error(
                "M2A-CWOLF-RIG-NONFINITE-SOURCE",
                "source.ir.primitives[0].positions",
                "source positions must be finite",
            ));
        }
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    if points.is_empty() {
        return Err(error(
            "M2A-CWOLF-RIG-EMPTY-SOURCE",
            "source.ir.primitives[0].positions",
            "source surface cannot be empty",
        ));
    }
    Ok(Bounds3V1 { min, max })
}

fn normalized_world_position(bounds: Bounds3V1, normalized: [f32; 3]) -> [f32; 3] {
    [
        bounds.min[0] + normalized[0] * (bounds.max[0] - bounds.min[0]),
        bounds.min[1] + normalized[1] * (bounds.max[1] - bounds.min[1]),
        bounds.min[2] + normalized[2] * (bounds.max[2] - bounds.min[2]),
    ]
}

fn translation_matrix(translation: [f32; 3]) -> [f32; 16] {
    [
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
        translation[0],
        translation[1],
        translation[2],
        1.0,
    ]
}

fn influence(bone_node_id: u32, value: f32) -> RigWeightInfluenceV1 {
    RigWeightInfluenceV1 {
        bone_node_id,
        value,
    }
}

fn procedural_weights(position: [f32; 3], bounds: Bounds3V1) -> Vec<RigWeightInfluenceV1> {
    let x_mid = (bounds.min[0] + bounds.max[0]) * 0.5;
    let y = normalized_axis(position[1], bounds.min[1], bounds.max[1]);
    let z = normalized_axis(position[2], bounds.min[2], bounds.max[2]);
    let left = position[0] >= x_mid;

    if y >= 0.79 {
        return vec![influence(8, 0.85), influence(7, 0.15)];
    }
    if y >= 0.66 && z >= 0.42 {
        return vec![influence(7, 0.75), influence(2, 0.25)];
    }
    if y >= 0.56 && z < 0.58 {
        let bone = match (left, z) {
            (true, value) if value < 0.14 => 6,
            (true, value) if value < 0.31 => 5,
            (true, value) if value < 0.47 => 4,
            (true, _) => 3,
            (false, value) if value < 0.14 => 16,
            (false, value) if value < 0.31 => 15,
            (false, value) if value < 0.47 => 14,
            (false, _) => 13,
        };
        return vec![influence(bone, 0.9), influence(2, 0.1)];
    }
    if y <= 0.46 && z < 0.58 {
        let bone = match (left, z) {
            (true, value) if value < 0.14 => 23,
            (true, value) if value < 0.31 => 22,
            (true, value) if value < 0.47 => 21,
            (true, _) => 20,
            (false, value) if value < 0.14 => 27,
            (false, value) if value < 0.31 => 26,
            (false, value) if value < 0.47 => 25,
            (false, _) => 24,
        };
        return vec![influence(bone, 0.9), influence(17, 0.1)];
    }
    if y <= 0.20 && z >= 0.34 {
        return if y <= 0.08 {
            vec![influence(19, 0.8), influence(18, 0.2)]
        } else {
            vec![influence(18, 0.8), influence(17, 0.2)]
        };
    }
    if y >= 0.48 {
        vec![influence(2, 0.8), influence(17, 0.2)]
    } else {
        vec![influence(17, 0.8), influence(2, 0.2)]
    }
}

fn procedural_weights_v2(
    position: [f32; 3],
    bounds: Bounds3V1,
    world_positions: &[[f32; 3]],
) -> Vec<RigWeightInfluenceV1> {
    let y = normalized_axis(position[1], bounds.min[1], bounds.max[1]);
    let z = normalized_axis(position[2], bounds.min[2], bounds.max[2]);
    let width = bounds.max[0] - bounds.min[0];

    if y >= 0.79 {
        return vec![influence(8, 0.85), influence(7, 0.15)];
    }
    if y >= 0.66 && z >= 0.42 {
        return vec![influence(7, 0.75), influence(2, 0.25)];
    }
    if y <= 0.20 && z >= 0.34 {
        return if y <= 0.08 {
            vec![influence(19, 0.8), influence(18, 0.2)]
        } else {
            vec![influence(18, 0.8), influence(17, 0.2)]
        };
    }

    let left = position[0] >= (bounds.min[0] + bounds.max[0]) * 0.5;
    if (0.53..0.79).contains(&y) && z < 0.61 {
        let chain = if left { [3, 4, 5, 6] } else { [13, 14, 15, 16] };
        if chain_xy_distance(position, chain, world_positions) <= width * 0.12 {
            return chain_weights(position, chain, 2, world_positions);
        }
    }
    if (0.05..0.53).contains(&y) && z < 0.61 {
        let chain = if left {
            [20, 21, 22, 23]
        } else {
            [24, 25, 26, 27]
        };
        if chain_xy_distance(position, chain, world_positions) <= width * 0.12 {
            return chain_weights(position, chain, 17, world_positions);
        }
    }

    if y >= 0.48 {
        vec![influence(2, 0.8), influence(17, 0.2)]
    } else {
        vec![influence(17, 0.8), influence(2, 0.2)]
    }
}

fn procedural_weights_v3(
    position: [f32; 3],
    bounds: Bounds3V1,
    world_positions: &[[f32; 3]],
) -> Vec<RigWeightInfluenceV1> {
    let y = normalized_axis(position[1], bounds.min[1], bounds.max[1]);
    let z = normalized_axis(position[2], bounds.min[2], bounds.max[2]);
    let width = bounds.max[0] - bounds.min[0];

    if y >= 0.79 {
        return vec![influence(8, 0.85), influence(7, 0.15)];
    }
    if y >= 0.66 && z >= 0.42 {
        return vec![influence(7, 0.75), influence(2, 0.25)];
    }
    if y <= 0.20 && z >= 0.34 {
        return if y <= 0.08 {
            vec![influence(19, 0.8), influence(18, 0.2)]
        } else {
            vec![influence(18, 0.8), influence(17, 0.2)]
        };
    }

    let x_mid = (bounds.min[0] + bounds.max[0]) * 0.5;
    let left = position[0] >= x_mid;
    let outside_center = (position[0] - x_mid).abs() >= width * 0.08;
    if outside_center && (0.50..0.80).contains(&y) && z < 0.66 {
        let chain = if left { [3, 4, 5, 6] } else { [13, 14, 15, 16] };
        if chain_xyz_distance(position, chain, world_positions) <= width * 0.31 {
            return continuous_chain_weights_v3(position, chain, 2, world_positions);
        }
    }
    if outside_center && (0.03..0.54).contains(&y) && z < 0.64 {
        let chain = if left {
            [20, 21, 22, 23]
        } else {
            [24, 25, 26, 27]
        };
        if chain_xyz_distance(position, chain, world_positions) <= width * 0.31 {
            return continuous_chain_weights_v3(position, chain, 17, world_positions);
        }
    }

    if y >= 0.48 {
        vec![influence(2, 0.8), influence(17, 0.2)]
    } else {
        vec![influence(17, 0.8), influence(2, 0.2)]
    }
}

fn procedural_weights_v4(
    position: [f32; 3],
    bounds: Bounds3V1,
    world_positions: &[[f32; 3]],
) -> Vec<RigWeightInfluenceV1> {
    let y = normalized_axis(position[1], bounds.min[1], bounds.max[1]);
    let z = normalized_axis(position[2], bounds.min[2], bounds.max[2]);
    let width = bounds.max[0] - bounds.min[0];

    if y >= 0.81 {
        return vec![influence(8, 0.88), influence(7, 0.12)];
    }
    if y >= 0.65 && z >= 0.40 {
        return vec![influence(7, 0.78), influence(2, 0.22)];
    }
    if y <= 0.19 && z >= 0.34 {
        return if y <= 0.075 {
            vec![influence(19, 0.82), influence(18, 0.18)]
        } else {
            vec![influence(18, 0.82), influence(17, 0.18)]
        };
    }

    let x_mid = (bounds.min[0] + bounds.max[0]) * 0.5;
    let left = position[0] < x_mid;
    let outside_center = (position[0] - x_mid).abs() >= width * 0.10;
    if outside_center && (0.50..0.80).contains(&y) && z < 0.65 {
        let chain = if left { [3, 4, 5, 6] } else { [13, 14, 15, 16] };
        if chain_xyz_distance(position, chain, world_positions) <= width * 0.24 {
            return continuous_chain_weights_v3(position, chain, 2, world_positions);
        }
    }
    if outside_center && (0.03..0.54).contains(&y) && z < 0.63 {
        let chain = if left {
            [20, 21, 22, 23]
        } else {
            [24, 25, 26, 27]
        };
        if chain_xyz_distance(position, chain, world_positions) <= width * 0.24 {
            return continuous_chain_weights_v3(position, chain, 17, world_positions);
        }
    }

    if y >= 0.47 {
        vec![influence(2, 0.88), influence(17, 0.12)]
    } else {
        vec![influence(17, 0.88), influence(2, 0.12)]
    }
}

#[derive(Clone, Debug)]
struct RawConnectedComponentV4 {
    triangle_indices: Vec<usize>,
    vertex_indices: Vec<usize>,
}

fn apply_component_coherence_v4(
    positions: &[[f32; 3]],
    indices: &[u32],
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<CWolfComponentCoherenceReportV4, CWolfCompatibleRigErrorV1> {
    if positions.len() != weights.len() || indices.is_empty() || !indices.len().is_multiple_of(3) {
        return Err(error(
            "M2A-CWOLF-RIG-COMPONENT-SHAPE",
            "source.ir.primitives[0]",
            "component-coherent weighting requires matching positions/weights and indexed triangles",
        ));
    }
    if indices.iter().any(|index| {
        usize::try_from(*index)
            .ok()
            .is_none_or(|index| index >= positions.len())
    }) {
        return Err(error(
            "M2A-CWOLF-RIG-COMPONENT-INDEX",
            "source.ir.primitives[0].indices",
            "component index exceeds the source vertex array",
        ));
    }
    let components = raw_connected_components_v4(indices);
    let height = positions
        .iter()
        .map(|point| point[2])
        .fold(f32::NEG_INFINITY, f32::max)
        - positions
            .iter()
            .map(|point| point[2])
            .fold(f32::INFINITY, f32::min);
    let multi_primary_component_count_before = components
        .iter()
        .filter(|component| primary_bone_count_v4(component, weights) > 1)
        .count();
    let mut locked_component_count = 0usize;
    let mut locked_component_vertex_count = 0usize;
    for component in &components {
        let extent = component_extent_v4(component, positions);
        let lock = should_lock_component_v4(component, extent, height);
        if !lock {
            continue;
        }
        let coherent = averaged_component_weights_v4(component, weights);
        for &vertex in &component.vertex_indices {
            weights[vertex] = coherent.clone();
        }
        locked_component_count += 1;
        locked_component_vertex_count += component.vertex_indices.len();
    }
    let multi_primary_component_count_after = components
        .iter()
        .filter(|component| primary_bone_count_v4(component, weights) > 1)
        .count();
    let unlocked_components = components
        .iter()
        .filter_map(|component| {
            let component_bounds = component_bounds_v4(component, positions);
            let extent = component_extent_from_bounds_v4(component_bounds);
            (!should_lock_component_v4(component, extent, height)).then(|| {
                CWolfUnlockedComponentV4 {
                    triangle_count: component.triangle_indices.len(),
                    vertex_count: component.vertex_indices.len(),
                    bounds: component_bounds,
                    extent,
                    primary_bone_node_ids: component_primary_bones_v4(component, weights),
                }
            })
        })
        .collect();
    Ok(CWolfComponentCoherenceReportV4 {
        component_count: components.len(),
        tiny_triangle_component_count: components
            .iter()
            .filter(|component| component.triangle_indices.len() <= 2)
            .count(),
        locked_component_count,
        locked_component_vertex_count,
        largest_component_vertex_count: components
            .iter()
            .map(|component| component.vertex_indices.len())
            .max()
            .unwrap_or(0),
        multi_primary_component_count_before,
        multi_primary_component_count_after,
        unlocked_components,
    })
}

fn should_lock_component_v4(
    _component: &RawConnectedComponentV4,
    _extent: f32,
    _model_height: f32,
) -> bool {
    // The selected Meshy source is a fur-shell asset made of 1,836 disconnected
    // surfaces. Letting one such surface cross bone boundaries produces the
    // needle-like spikes visible in V3. One dominant bone per connected surface
    // preserves every triangle exactly while the collection of surfaces still
    // follows the compatible c_wolf hierarchy.
    true
}

fn force_leg_component_coverage_v4(
    positions: &[[f32; 3]],
    indices: &[u32],
    world_positions: &[[f32; 3]],
    nodes: &[CreatureRigNodeV1],
    weights: &mut [Vec<RigWeightInfluenceV1>],
) -> Result<Vec<CWolfForcedComponentAssignmentV4>, CWolfCompatibleRigErrorV1> {
    const LEG_BONES: [u32; 16] = [3, 4, 5, 6, 13, 14, 15, 16, 20, 21, 22, 23, 24, 25, 26, 27];
    let components = raw_connected_components_v4(indices);
    let centroids = components
        .iter()
        .map(|component| component_centroid_v4(component, positions))
        .collect::<Vec<_>>();
    let mut primary_counts = BTreeMap::<u32, usize>::new();
    for row in weights.iter() {
        if let Some(primary) = row
            .iter()
            .max_by(|left, right| left.value.total_cmp(&right.value))
        {
            *primary_counts.entry(primary.bone_node_id).or_default() += 1;
        }
    }
    let mut claimed = std::collections::BTreeSet::<usize>::new();
    let mut assignments = Vec::with_capacity(LEG_BONES.len());
    for bone_node_id in LEG_BONES {
        let existing_primary_count = primary_counts.get(&bone_node_id).copied().unwrap_or(0);
        if existing_primary_count >= 256 {
            continue;
        }
        let required_component_size = (256 - existing_primary_count).max(128);
        let bone_index = bone_node_id as usize;
        let bone = world_positions.get(bone_index).copied().ok_or_else(|| {
            error(
                "M2A-CWOLF-RIG-COVERAGE-BONE",
                format!("rig.nodes[{bone_node_id}]"),
                "required leg bone has no world-space pivot",
            )
        })?;
        let (component_index, distance) = components
            .iter()
            .enumerate()
            .filter(|(index, component)| {
                !claimed.contains(index)
                    && component.vertex_indices.len() >= required_component_size
            })
            .map(|(index, _)| (index, distance_3d_v4(centroids[index], bone)))
            .min_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| left.0.cmp(&right.0))
            })
            .ok_or_else(|| {
                error(
                    "M2A-CWOLF-RIG-COVERAGE-COMPONENT",
                    format!("rig.nodes[{bone_node_id}]"),
                    "no distinct source component is available for required leg coverage",
                )
            })?;
        claimed.insert(component_index);
        let component = &components[component_index];
        primary_counts.insert(
            bone_node_id,
            existing_primary_count + component.vertex_indices.len(),
        );
        for &vertex in &component.vertex_indices {
            weights[vertex] = vec![influence(bone_node_id, 1.0)];
        }
        assignments.push(CWolfForcedComponentAssignmentV4 {
            bone_node_id,
            bone_name: nodes[bone_index].name.clone(),
            component_vertex_count: component.vertex_indices.len(),
            component_centroid: centroids[component_index],
            distance_to_bone: distance,
        });
    }
    Ok(assignments)
}

fn component_centroid_v4(component: &RawConnectedComponentV4, positions: &[[f32; 3]]) -> [f32; 3] {
    let mut total = [0.0f64; 3];
    for &vertex in &component.vertex_indices {
        for axis in 0..3 {
            total[axis] += f64::from(positions[vertex][axis]);
        }
    }
    let divisor = component.vertex_indices.len() as f64;
    total.map(|value| (value / divisor) as f32)
}

fn distance_3d_v4(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2))
        .sqrt()
}

fn raw_connected_components_v4(indices: &[u32]) -> Vec<RawConnectedComponentV4> {
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
        if left != right {
            let (root, child) = if left < right {
                (left, right)
            } else {
                (right, left)
            };
            parent[child] = root;
        }
    }

    let triangle_count = indices.len() / 3;
    let mut parent = (0..triangle_count).collect::<Vec<_>>();
    let mut first_triangle_by_vertex = BTreeMap::<u32, usize>::new();
    for (triangle_index, triangle) in indices.chunks_exact(3).enumerate() {
        for vertex in triangle {
            if let Some(first) = first_triangle_by_vertex.get(vertex).copied() {
                union(&mut parent, triangle_index, first);
            } else {
                first_triangle_by_vertex.insert(*vertex, triangle_index);
            }
        }
    }
    let mut grouped = BTreeMap::<usize, RawConnectedComponentV4>::new();
    for triangle_index in 0..triangle_count {
        let root = find(&mut parent, triangle_index);
        let component = grouped
            .entry(root)
            .or_insert_with(|| RawConnectedComponentV4 {
                triangle_indices: Vec::new(),
                vertex_indices: Vec::new(),
            });
        component.triangle_indices.push(triangle_index);
        component.vertex_indices.extend(
            indices[triangle_index * 3..triangle_index * 3 + 3]
                .iter()
                .map(|index| *index as usize),
        );
    }
    for component in grouped.values_mut() {
        component.vertex_indices.sort_unstable();
        component.vertex_indices.dedup();
    }
    grouped.into_values().collect()
}

fn primary_bone_count_v4(
    component: &RawConnectedComponentV4,
    weights: &[Vec<RigWeightInfluenceV1>],
) -> usize {
    component
        .vertex_indices
        .iter()
        .filter_map(|vertex| {
            weights[*vertex]
                .iter()
                .max_by(|left, right| left.value.total_cmp(&right.value))
                .map(|weight| weight.bone_node_id)
        })
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn component_primary_bones_v4(
    component: &RawConnectedComponentV4,
    weights: &[Vec<RigWeightInfluenceV1>],
) -> Vec<u32> {
    component
        .vertex_indices
        .iter()
        .filter_map(|vertex| {
            weights[*vertex]
                .iter()
                .max_by(|left, right| left.value.total_cmp(&right.value))
                .map(|weight| weight.bone_node_id)
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn averaged_component_weights_v4(
    component: &RawConnectedComponentV4,
    weights: &[Vec<RigWeightInfluenceV1>],
) -> Vec<RigWeightInfluenceV1> {
    let mut totals = BTreeMap::<u32, f64>::new();
    for &vertex in &component.vertex_indices {
        for influence in &weights[vertex] {
            *totals.entry(influence.bone_node_id).or_default() += f64::from(influence.value);
        }
    }
    let mut ranked = totals.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(1);
    let sum = ranked.iter().map(|(_, value)| *value).sum::<f64>();
    ranked
        .into_iter()
        .map(|(bone_node_id, value)| influence(bone_node_id, (value / sum) as f32))
        .collect()
}

fn smooth_connected_weights_v4(
    indices: &[u32],
    weights: &mut [Vec<RigWeightInfluenceV1>],
    iteration_count: usize,
) -> Result<CWolfWeightSmoothingReportV4, CWolfCompatibleRigErrorV1> {
    if weights.is_empty() || indices.is_empty() || !indices.len().is_multiple_of(3) {
        return Err(error(
            "M2A-CWOLF-RIG-SMOOTHING-SHAPE",
            "source.ir.primitives[0]",
            "topology smoothing requires non-empty weights and indexed triangles",
        ));
    }
    let mut adjacency = vec![Vec::<usize>::new(); weights.len()];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= weights.len()) {
            return Err(error(
                "M2A-CWOLF-RIG-SMOOTHING-INDEX",
                "source.ir.primitives[0].indices",
                "smoothing index exceeds the source vertex array",
            ));
        }
        for (left, right) in [
            (vertices[0], vertices[1]),
            (vertices[1], vertices[2]),
            (vertices[2], vertices[0]),
        ] {
            if left != right {
                adjacency[left].push(right);
                adjacency[right].push(left);
            }
        }
    }
    for neighbors in &mut adjacency {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    let max_adjacent_weight_l1_before = max_adjacent_weight_l1_v4(&adjacency, weights);
    for _ in 0..iteration_count {
        let previous = weights.to_vec();
        for (vertex, neighbors) in adjacency.iter().enumerate() {
            if neighbors.is_empty() {
                continue;
            }
            let mut totals = BTreeMap::<u32, f64>::new();
            for influence in &previous[vertex] {
                *totals.entry(influence.bone_node_id).or_default() +=
                    f64::from(influence.value) * 0.5;
            }
            let neighbor_scale = 0.5 / neighbors.len() as f64;
            for &neighbor in neighbors {
                for influence in &previous[neighbor] {
                    *totals.entry(influence.bone_node_id).or_default() +=
                        f64::from(influence.value) * neighbor_scale;
                }
            }
            let mut ranked = totals.into_iter().collect::<Vec<_>>();
            ranked.sort_by(|left, right| {
                right
                    .1
                    .total_cmp(&left.1)
                    .then_with(|| left.0.cmp(&right.0))
            });
            ranked.truncate(4);
            let sum = ranked.iter().map(|(_, value)| *value).sum::<f64>();
            weights[vertex] = ranked
                .into_iter()
                .map(|(bone_node_id, value)| influence(bone_node_id, (value / sum) as f32))
                .collect();
        }
    }
    Ok(CWolfWeightSmoothingReportV4 {
        iteration_count,
        max_adjacent_weight_l1_before,
        max_adjacent_weight_l1_after: max_adjacent_weight_l1_v4(&adjacency, weights),
    })
}

fn max_adjacent_weight_l1_v4(
    adjacency: &[Vec<usize>],
    weights: &[Vec<RigWeightInfluenceV1>],
) -> f32 {
    let mut maximum = 0.0f32;
    for (vertex, neighbors) in adjacency.iter().enumerate() {
        for neighbor in neighbors
            .iter()
            .copied()
            .filter(|neighbor| *neighbor > vertex)
        {
            let mut difference = BTreeMap::<u32, f32>::new();
            for influence in &weights[vertex] {
                *difference.entry(influence.bone_node_id).or_default() += influence.value;
            }
            for influence in &weights[neighbor] {
                *difference.entry(influence.bone_node_id).or_default() -= influence.value;
            }
            maximum = maximum.max(difference.values().map(|value| value.abs()).sum());
        }
    }
    maximum
}

fn component_extent_v4(component: &RawConnectedComponentV4, positions: &[[f32; 3]]) -> f32 {
    component_extent_from_bounds_v4(component_bounds_v4(component, positions))
}

fn component_bounds_v4(component: &RawConnectedComponentV4, positions: &[[f32; 3]]) -> Bounds3V1 {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for &vertex in &component.vertex_indices {
        for axis in 0..3 {
            min[axis] = min[axis].min(positions[vertex][axis]);
            max[axis] = max[axis].max(positions[vertex][axis]);
        }
    }
    Bounds3V1 { min, max }
}

fn component_extent_from_bounds_v4(bounds: Bounds3V1) -> f32 {
    ((bounds.max[0] - bounds.min[0]).powi(2)
        + (bounds.max[1] - bounds.min[1]).powi(2)
        + (bounds.max[2] - bounds.min[2]).powi(2))
    .sqrt()
}

fn chain_xyz_distance(point: [f32; 3], chain: [u32; 4], world_positions: &[[f32; 3]]) -> f32 {
    chain
        .windows(2)
        .map(|pair| {
            point_segment_projection_3d(
                point,
                world_positions[pair[0] as usize],
                world_positions[pair[1] as usize],
            )
            .1
        })
        .fold(f32::INFINITY, f32::min)
}

fn continuous_chain_weights_v3(
    position: [f32; 3],
    chain: [u32; 4],
    torso_node_id: u32,
    world_positions: &[[f32; 3]],
) -> Vec<RigWeightInfluenceV1> {
    let (segment_index, amount, _) = chain
        .windows(2)
        .enumerate()
        .map(|(segment_index, pair)| {
            let (amount, distance) = point_segment_projection_3d(
                position,
                world_positions[pair[0] as usize],
                world_positions[pair[1] as usize],
            );
            (segment_index, amount, distance)
        })
        .min_by(|left, right| {
            left.2
                .total_cmp(&right.2)
                .then_with(|| left.0.cmp(&right.0))
        })
        .expect("four-node chain always contains three segments");
    let first = chain[segment_index];
    let second = chain[segment_index + 1];
    let first_value = 0.92 * (1.0 - amount);
    let second_value = 0.92 * amount;
    let mut weights = Vec::with_capacity(3);
    if first_value > 1.0e-6 {
        weights.push(influence(first, first_value));
    }
    if second_value > 1.0e-6 {
        weights.push(influence(second, second_value));
    }
    weights.push(influence(torso_node_id, 0.08));
    weights
}

fn point_segment_projection_3d(point: [f32; 3], start: [f32; 3], end: [f32; 3]) -> (f32, f32) {
    let segment = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let from_start = [
        point[0] - start[0],
        point[1] - start[1],
        point[2] - start[2],
    ];
    let length_squared = segment.iter().map(|value| value * value).sum::<f32>();
    let amount = if length_squared <= 1.0e-12 {
        0.0
    } else {
        (from_start
            .iter()
            .zip(segment.iter())
            .map(|(left, right)| left * right)
            .sum::<f32>()
            / length_squared)
            .clamp(0.0, 1.0)
    };
    let closest = [
        start[0] + segment[0] * amount,
        start[1] + segment[1] * amount,
        start[2] + segment[2] * amount,
    ];
    let distance = ((point[0] - closest[0]).powi(2)
        + (point[1] - closest[1]).powi(2)
        + (point[2] - closest[2]).powi(2))
    .sqrt();
    (amount, distance)
}

fn chain_xy_distance(point: [f32; 3], chain: [u32; 4], world_positions: &[[f32; 3]]) -> f32 {
    chain
        .windows(2)
        .map(|pair| {
            point_segment_distance_xy(
                point,
                world_positions[pair[0] as usize],
                world_positions[pair[1] as usize],
            )
        })
        .fold(f32::INFINITY, f32::min)
}

fn point_segment_distance_xy(point: [f32; 3], start: [f32; 3], end: [f32; 3]) -> f32 {
    let segment = [end[0] - start[0], end[1] - start[1]];
    let from_start = [point[0] - start[0], point[1] - start[1]];
    let length_squared = segment[0] * segment[0] + segment[1] * segment[1];
    let amount = if length_squared <= 1.0e-12 {
        0.0
    } else {
        ((from_start[0] * segment[0] + from_start[1] * segment[1]) / length_squared).clamp(0.0, 1.0)
    };
    let closest = [
        start[0] + segment[0] * amount,
        start[1] + segment[1] * amount,
    ];
    ((point[0] - closest[0]).powi(2) + (point[1] - closest[1]).powi(2)).sqrt()
}

fn chain_weights(
    position: [f32; 3],
    chain: [u32; 4],
    torso_node_id: u32,
    world_positions: &[[f32; 3]],
) -> Vec<RigWeightInfluenceV1> {
    let mut distances = chain
        .into_iter()
        .map(|bone_node_id| {
            let bone = world_positions[bone_node_id as usize];
            let distance = ((position[0] - bone[0]).powi(2)
                + (position[1] - bone[1]).powi(2)
                + (position[2] - bone[2]).powi(2))
            .sqrt();
            (bone_node_id, distance)
        })
        .collect::<Vec<_>>();
    distances.sort_by(|left, right| left.1.total_cmp(&right.1));
    let inverse_a = 1.0 / (distances[0].1 + 1.0e-4);
    let inverse_b = 1.0 / (distances[1].1 + 1.0e-4);
    let inverse_sum = inverse_a + inverse_b;
    vec![
        influence(distances[0].0, 0.92 * inverse_a / inverse_sum),
        influence(distances[1].0, 0.92 * inverse_b / inverse_sum),
        influence(torso_node_id, 0.08),
    ]
}

fn normalized_axis(value: f32, min: f32, max: f32) -> f32 {
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn c_wolf_node_specs() -> [NodeSpec; 30] {
    [
        NodeSpec {
            name: "c_Wolf",
            parent: None,
            normalized_world: [0.5, 0.5, 0.0],
        },
        NodeSpec {
            name: "Wolf_rootdummy",
            parent: Some(0),
            normalized_world: [0.5, 0.5, 0.0],
        },
        NodeSpec {
            name: "Wolf_ribcage",
            parent: Some(1),
            normalized_world: [0.5, 0.62, 0.58],
        },
        NodeSpec {
            name: "Wolf_Lfrontupperleg",
            parent: Some(2),
            normalized_world: [0.68, 0.66, 0.55],
        },
        NodeSpec {
            name: "Wolf_LfrontbotlegA",
            parent: Some(3),
            normalized_world: [0.69, 0.67, 0.38],
        },
        NodeSpec {
            name: "Wolf_LfrontbotlegB",
            parent: Some(4),
            normalized_world: [0.69, 0.68, 0.22],
        },
        NodeSpec {
            name: "Wolf_Lfrontpaw",
            parent: Some(5),
            normalized_world: [0.69, 0.71, 0.06],
        },
        NodeSpec {
            name: "Wolf_neck",
            parent: Some(2),
            normalized_world: [0.5, 0.72, 0.69],
        },
        NodeSpec {
            name: "Wolf_head",
            parent: Some(7),
            normalized_world: [0.5, 0.84, 0.76],
        },
        NodeSpec {
            name: "Wolf_Rear",
            parent: Some(8),
            normalized_world: [0.40, 0.82, 0.84],
        },
        NodeSpec {
            name: "Wolf_Lear",
            parent: Some(8),
            normalized_world: [0.60, 0.82, 0.84],
        },
        NodeSpec {
            name: "head",
            parent: Some(8),
            normalized_world: [0.5, 0.91, 0.74],
        },
        NodeSpec {
            name: "impact",
            parent: Some(7),
            normalized_world: [0.5, 0.79, 0.64],
        },
        NodeSpec {
            name: "Wolf_Rfrontupperleg",
            parent: Some(2),
            normalized_world: [0.32, 0.66, 0.55],
        },
        NodeSpec {
            name: "Wolf_RfrontbotlegA",
            parent: Some(13),
            normalized_world: [0.31, 0.67, 0.38],
        },
        NodeSpec {
            name: "Wolf_RfrontbotlegB",
            parent: Some(14),
            normalized_world: [0.31, 0.68, 0.22],
        },
        NodeSpec {
            name: "Wolf_Rfrontpaw",
            parent: Some(15),
            normalized_world: [0.31, 0.71, 0.06],
        },
        NodeSpec {
            name: "Wolf_pelvis",
            parent: Some(1),
            normalized_world: [0.5, 0.36, 0.54],
        },
        NodeSpec {
            name: "Wolf_tail",
            parent: Some(17),
            normalized_world: [0.5, 0.22, 0.54],
        },
        NodeSpec {
            name: "Wolf_tailend",
            parent: Some(18),
            normalized_world: [0.5, 0.08, 0.42],
        },
        NodeSpec {
            name: "Wolf_Lbacktopleg",
            parent: Some(17),
            normalized_world: [0.68, 0.36, 0.52],
        },
        NodeSpec {
            name: "Wolf_Lbackmidleg",
            parent: Some(20),
            normalized_world: [0.69, 0.34, 0.36],
        },
        NodeSpec {
            name: "Wolf_Lbackbotleg",
            parent: Some(21),
            normalized_world: [0.69, 0.32, 0.21],
        },
        NodeSpec {
            name: "Wolf_Lbackpaw",
            parent: Some(22),
            normalized_world: [0.69, 0.29, 0.06],
        },
        NodeSpec {
            name: "Wolf_Rbacktopleg",
            parent: Some(17),
            normalized_world: [0.32, 0.36, 0.52],
        },
        NodeSpec {
            name: "Wolf_Rbackmidleg",
            parent: Some(24),
            normalized_world: [0.31, 0.34, 0.36],
        },
        NodeSpec {
            name: "Wolf_Rbackbotleg",
            parent: Some(25),
            normalized_world: [0.31, 0.32, 0.21],
        },
        NodeSpec {
            name: "Wolf_Rbackpaw",
            parent: Some(26),
            normalized_world: [0.31, 0.29, 0.06],
        },
        NodeSpec {
            name: "headconjure",
            parent: Some(0),
            normalized_world: [0.5, 0.87, 0.79],
        },
        NodeSpec {
            name: "handconjure",
            parent: Some(0),
            normalized_world: [0.5, 0.62, 0.56],
        },
    ]
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> CWolfCompatibleRigErrorV1 {
    CWolfCompatibleRigErrorV1 {
        schema_version: C_WOLF_COMPATIBLE_RIG_SCHEMA_VERSION,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
