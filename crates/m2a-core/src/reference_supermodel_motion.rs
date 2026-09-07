//! Material-complete, motion-compatible static creature binding for an
//! installed Aurora reference supermodel.
//!
//! `reference_supermodel` V1/V2 intentionally proves only ordered topology.
//! This module adds an explicit neutral carrier pose, persistent correction
//! nodes, material preservation and a read-only inherited-motion oracle.  No
//! reference controller, animation, skeleton or geometry payload is copied to
//! the generated model.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    aurora_material::{
        AuroraMaterialCompilationSetV1, AuroraMaterialCompileStatusV1,
        AuroraMaterialFidelityEntryV1, AuroraMaterialTargetProfileV1, compile_gltf_materials_v1,
    },
    aurora_material_package::{
        AuroraMaterialPackageV1, ensure_material_tangents_v1, package_aurora_materials_v1,
        validate_material_resource_semantics_v1,
    },
    creature_product::{
        CreatureAlphaModeV2, CreatureMaterialProfileV2, CreatureMaterialTargetV2,
        validate_creature_material_profile_v2,
    },
    erf::ErfArchive,
    glb::{GlbLimits, ingest_glb},
    hak::{HakArtifactV1, HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    mdl::{
        BinaryMdlArtifactV1, InspectionReport, MdlAnimationClipV1, MdlAnimationEventV1,
        MdlAnimationInterpolationV1, MdlAnimationSetV1, MdlAnimationTrackPathV1,
        MdlAnimationTrackV1, MdlFormatProfileV1, MdlMaterialExtensionArtifactV1,
        MdlMaterialExtensionOptionsV1, MdlMaterialExtensionReportV1, MdlMaterialStateV1,
        MdlMaterialTextureBindingV1, MdlRenderHintV1, MdlSegmentMaterialStreamsV1,
        MdlWriterOptionsV1, NodeReport, evaluate_reference_supermodel_node_world_matrices_v2,
        evaluate_reference_supermodel_render_deformation_samples_v3,
        extend_binary_mdl_with_materials_v1,
        write_binary_mdl_with_animations_and_supermodel_exact_face_planes_v1,
        write_binary_mdl_with_supermodel_exact_face_planes_v1,
    },
    model_ir::{AuroraModelIrV1, AuroraVertexWeightsV1},
    model_segmentation::segment_model_for_binary_mdl_v1,
    mtr::{
        MTR_RESOURCE_TYPE_V1, MTR_SCHEMA_VERSION_V1, MtrDocumentV1, MtrRenderHintV1,
        MtrTextureBindingV1, parse_mtr_v1, write_mtr_v1,
    },
    profile_a::{
        CreatureRigNodeV1, CreatureRigProfileV1, CreatureSourceForwardV1,
        ProfileAConversionOutcomeV1, canonical_profile_sha256, convert_profile_a_exact_v1,
        direct_creature_profile_a_options_for_source_forward_v3,
    },
    txi::TXI_RESOURCE_TYPE_V1,
};

pub const REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceSupermodelCompatibilityLevelV2 {
    TopologyOnly,
    BindPoseCompatible,
    MotionCompatible,
}

/// Structural classification of one emitted animation carrier. The
/// classification is derived from the complete inherited controller corpus
/// and hierarchy, never from a species/resref whitelist.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceSupermodelCarrierClassV3 {
    /// A non-root carrier with a non-static local transform controller in at
    /// least one inherited clip. It must own a visible weighted cluster.
    SkinRelevant,
    /// An ancestor/root required to preserve the exact carrier hierarchy but
    /// not required to own surface weights.
    PassiveStructural,
    /// A passive terminal/attachment/end carrier. Zero weights are legal and
    /// are reported explicitly.
    PassiveAttachmentOrEnd,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelCarrierExclusionV3 {
    pub node_name: String,
    pub parent_name: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelMotionNodeV2 {
    pub part_number: u32,
    pub name: String,
    pub parent_part_number: Option<u32>,
    /// Column-major neutral local carrier transform. Production V3 contracts
    /// extract it exactly from the inspected reference supermodel.
    pub carrier_bind_local_matrix: [f32; 16],
    pub position_controller_required: bool,
    pub orientation_controller_required: bool,
    pub scale_controller_required: bool,
    pub anchor_role: Option<String>,
    /// Caller-authored semantic joint axis in carrier-local space.
    pub joint_axis: Option<[f32; 3]>,
    pub carrier_class: ReferenceSupermodelCarrierClassV3,
    pub structural_role: String,
    /// Every inherited clip that contains a decoded local transform
    /// controller for this carrier, including static controller rows.
    pub controlling_clips: Vec<String>,
    /// Subset of `controlling_clips` whose local controller is non-static.
    /// Every cell in joint × dynamicClips requires visible response.
    pub dynamic_clips: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRequiredEventV2 {
    pub clip_name: String,
    pub event_name: String,
    pub minimum_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelMotionTolerancesV2 {
    pub bind_max_abs_error: f32,
    pub edge_soft_min_ratio: f32,
    pub edge_soft_max_ratio: f32,
    pub edge_hard_min_ratio: f32,
    pub edge_hard_max_ratio: f32,
    pub soft_edge_min_bind_diagonal_fraction: f32,
    pub edge_soft_max_fraction: f32,
    pub edge_hard_max_fraction: f32,
    pub triangle_min_area_ratio: f32,
    pub triangle_max_area_ratio: f32,
    pub triangle_min_bind_area_diagonal_squared_fraction: f32,
    pub triangle_area_collapse_max_fraction: f32,
    pub triangle_area_expansion_max_fraction: f32,
    pub seam_source_max_diagonal_fraction: f32,
    pub seam_output_max_source_multiple: f32,
    pub seam_output_floor_diagonal_fraction: f32,
    pub seam_max_violation_fraction: f32,
    /// Profiles used for export may require literal zero visible seam breaks.
    /// The legacy density-normalized count remains in the report only.
    pub fail_on_any_seam_violation: bool,
    pub visible_anchor_min_amplitude_ratio: f32,
    pub visible_anchor_min_trajectory_alignment: f32,
    pub paw_ground_contact_max_height_fraction: f32,
    pub paw_controller_max_position_error_fraction: f32,
    pub clip_start_anchor_max_jump_fraction: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelMotionContractV2 {
    pub schema_version: u32,
    pub contract_id: String,
    pub content_sha256: String,
    pub supermodel_resref: String,
    pub source_model_sha256: String,
    pub inspected_read_only: bool,
    pub no_payload_copied: bool,
    pub classification: u8,
    pub animation_scale: f32,
    pub clean_room_profile_id: String,
    pub clean_room_profile_sha256: String,
    pub nodes: Vec<ReferenceSupermodelMotionNodeV2>,
    pub carrier_exclusions: Vec<ReferenceSupermodelCarrierExclusionV3>,
    pub required_clips: Vec<String>,
    pub required_events: Vec<ReferenceSupermodelRequiredEventV2>,
    pub tolerances: ReferenceSupermodelMotionTolerancesV2,
}

/// Optional semantic annotations layered on top of an exact inspected
/// supermodel bind contract.  They never author or move carrier bones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelSemanticNodeV3 {
    pub node_name: String,
    pub anchor_role: Option<String>,
    pub joint_axis: Option<[f32; 3]>,
}

/// Inputs that are not present in a binary MDL but are needed by the product
/// contract.  Neutral carrier transforms are always decoded from `reference`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelExactContractOptionsV3 {
    pub contract_id: String,
    pub supermodel_resref: String,
    pub source_model_sha256: String,
    pub required_clips: Vec<String>,
    pub required_events: Vec<ReferenceSupermodelRequiredEventV2>,
    pub semantic_nodes: Vec<ReferenceSupermodelSemanticNodeV3>,
    pub tolerances: ReferenceSupermodelMotionTolerancesV2,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelCorrectionNodeV1 {
    pub carrier_node_id: u32,
    pub carrier_name: String,
    pub correction_node_id: u32,
    pub correction_name: String,
    pub neutral_max_abs_error: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelCorrectionReportV1 {
    pub schema_version: u32,
    pub source_rig_sha256: String,
    pub output_rig_sha256: String,
    pub carrier_node_count: usize,
    pub correction_node_count: usize,
    pub weighted_reference_remap_count: usize,
    pub neutral_max_abs_error: f32,
    /// Difference between the caller-authored source rig and the exact
    /// inspected carrier bind.  This is diagnostic and may be non-zero.
    pub source_bind_max_abs_error: f32,
    /// Difference between emitted named carriers and the inspected reference.
    pub reference_bind_max_abs_error: f32,
    pub reference_bind_verified: bool,
    pub nodes: Vec<ReferenceSupermodelCorrectionNodeV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSupermodelCorrectionArtifactV1 {
    pub rig: CreatureRigProfileV1,
    pub report: ReferenceSupermodelCorrectionReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelMaterialOptionsV1 {
    pub schema_version: u32,
    pub texture_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelClassicMaterialOptionsV1 {
    pub schema_version: u32,
    pub texture_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelMinimalMtrMaterialOptionsV1 {
    pub schema_version: u32,
    pub texture_resref: String,
    pub material_resref: String,
    pub material_profile: CreatureMaterialProfileV2,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriangleAreaDiagnosticV1 {
    pub skin_node_name: String,
    pub vertex_indices: [u32; 3],
    pub vertex_influences: Vec<TriangleVertexInfluenceDiagnosticV1>,
    pub bind_centroid: [f32; 3],
    pub area_ratio: f32,
    pub sample_time_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeLengthDiagnosticV1 {
    pub skin_node_name: String,
    pub vertex_indices: [u32; 2],
    pub vertex_influences: Vec<TriangleVertexInfluenceDiagnosticV1>,
    pub bind_endpoints: [[f32; 3]; 2],
    pub sampled_endpoints: [[f32; 3]; 2],
    pub length_ratio: f32,
    pub sample_time_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriangleVertexInfluenceDiagnosticV1 {
    pub vertex_index: u32,
    pub influences: Vec<TriangleBoneInfluenceDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriangleBoneInfluenceDiagnosticV1 {
    pub bone_node_name: String,
    pub weight: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InheritedMotionClipQualityV1 {
    pub clip_name: String,
    pub sampled_times: Vec<f32>,
    pub vertex_sample_count: u64,
    pub edge_sample_count: u64,
    pub edge_soft_sample_count: u64,
    pub edge_outside_soft_limit_count: u64,
    pub edge_outside_hard_limit_count: u64,
    pub triangle_sample_count: u64,
    pub triangle_area_collapse_count: u64,
    pub triangle_area_expansion_count: u64,
    pub sampled_component_count: usize,
    pub component_sample_count: u64,
    pub per_component_geometry_coverage: bool,
    pub per_component_deformation_coverage: bool,
    pub components: Vec<InheritedMotionComponentQualityV1>,
    pub worst_triangle_area_collapse: Option<TriangleAreaDiagnosticV1>,
    pub worst_triangle_area_expansion: Option<TriangleAreaDiagnosticV1>,
    pub worst_edge_collapse: Option<EdgeLengthDiagnosticV1>,
    pub worst_edge_expansion: Option<EdgeLengthDiagnosticV1>,
    pub world_normal_opposition_count: u64,
    pub seam_pair_sample_count: u64,
    pub seam_pair_violation_count: u64,
    pub visible_anchor_trajectories: Vec<VisibleAnchorTrajectoryReportV1>,
    pub visible_anchor_motion_violation_count: u64,
    pub max_edge_ratio: f32,
    pub min_edge_ratio: f32,
    pub max_displacement: f32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub paw_gate_applied: bool,
    pub anchor_cluster_count: usize,
    pub anchor_cluster_missing_count: usize,
    pub paw_cluster_count: usize,
    pub paw_cluster_missing_count: usize,
    pub paw_contact_violation_count: u64,
    pub paw_side_violation_count: u64,
    pub clip_start_anchor_jump_violation_count: u64,
    pub max_paw_ground_height_error: f32,
    pub max_clip_start_anchor_jump: f32,
    pub root_motion_distance: f32,
    pub appendage_pair_required_count: usize,
    pub appendage_pair_pass_count: usize,
    pub appendage_relative_motion_violation_count: u64,
    pub appendage_relative_motion: Vec<AppendageRelativeMotionDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InheritedMotionComponentQualityV1 {
    pub component_index: usize,
    pub triangle_sample_count: u64,
    pub edge_sample_count: u64,
    pub edge_outside_hard_limit_count: u64,
    pub triangle_area_collapse_count: u64,
    pub triangle_area_expansion_count: u64,
    pub min_edge_ratio: f32,
    pub max_edge_ratio: f32,
    pub min_triangle_area_ratio: f32,
    pub max_triangle_area_ratio: f32,
    pub pass: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendageRelativeMotionDiagnosticV1 {
    pub parent_role: String,
    pub child_role: String,
    pub surface_relative_amplitude: f32,
    pub controller_relative_amplitude: f32,
    pub amplitude_ratio: f32,
    pub trajectory_alignment: f32,
    pub pass: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InheritedMotionQualityReportV1 {
    pub schema_version: u32,
    pub profile: String,
    pub status: String,
    pub clips: Vec<InheritedMotionClipQualityV1>,
    pub edge_sample_count: u64,
    pub edge_soft_sample_count: u64,
    pub edge_outside_soft_limit_count: u64,
    pub edge_outside_soft_allowed_count: u64,
    pub edge_outside_hard_limit_count: u64,
    pub edge_outside_hard_allowed_count: u64,
    pub triangle_sample_count: u64,
    pub triangle_area_collapse_count: u64,
    pub triangle_area_collapse_allowed_count: u64,
    pub triangle_area_expansion_count: u64,
    pub triangle_area_expansion_allowed_count: u64,
    pub world_normal_opposition_count: u64,
    pub world_normal_opposition_diagnostic_only: bool,
    pub seam_pair_sample_count: u64,
    pub seam_pair_violation_count: u64,
    pub seam_pair_allowed_count: u64,
    pub surface_seam_gate: SurfaceSeamGateReportV1,
    pub visible_anchor_motion_violation_count: u64,
    pub anchor_cluster_missing_count: usize,
    pub paw_cluster_missing_count: usize,
    pub paw_contact_violation_count: u64,
    pub paw_side_violation_count: u64,
    pub clip_start_anchor_jump_violation_count: u64,
    pub paw_motion_diagnostics_only: bool,
    pub required_clip_count: usize,
    pub sampled_clip_count: usize,
    pub inherited_clip_coverage: bool,
    pub required_joint_count: usize,
    pub joint_clip_required_count: usize,
    pub joint_clip_pass_count: usize,
    pub joint_clip_coverage: bool,
    pub visible_motion_coverage: bool,
    /// Every required clip contributed a non-empty edge and triangle domain.
    /// A zero-sized domain is an unevaluated deformation, never a PASS.
    pub per_clip_geometry_coverage: bool,
    /// Density-independent admission: every clip must satisfy the configured
    /// edge/triangle budgets on its own instead of borrowing a global budget
    /// from millions of unrelated samples.
    pub per_clip_deformation_coverage: bool,
    /// Clip-start anchor discontinuities are product blocking rather than a
    /// diagnostic-only counter.
    pub clip_start_anchor_continuity: bool,
    pub per_component_geometry_coverage: bool,
    pub per_component_deformation_coverage: bool,
    pub appendage_relative_motion_coverage: bool,
    pub appendage_pair_required_count: usize,
    pub appendage_pair_pass_count: usize,
    pub appendage_relative_motion_violation_count: u64,
    pub joint_clip_coverage_matrix: Vec<ReferenceSupermodelJointClipCoverageCellV3>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelJointClipCoverageCellV3 {
    pub schema_version: u32,
    pub joint_name: String,
    pub structural_role: String,
    pub clip_name: String,
    pub cluster_vertex_count: usize,
    pub surface_amplitude: f32,
    pub controller_probe_amplitude: f32,
    pub amplitude_ratio: f32,
    pub trajectory_alignment: f32,
    pub trajectory_required: bool,
    pub status: String,
    pub pass: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisibleAnchorTrajectorySampleV1 {
    pub time_seconds: f32,
    pub surface_centroid: [f32; 3],
    pub controller_centroid: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibleAnchorTrajectoryReportV1 {
    pub schema_version: u32,
    pub role: String,
    pub clip_name: String,
    pub sample_count: usize,
    pub surface_amplitude: f32,
    pub controller_amplitude: f32,
    pub amplitude_ratio: f32,
    pub trajectory_alignment: f32,
    pub status: String,
    pub pass: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceSeamGateReportV1 {
    pub schema_version: u32,
    pub sample_count: u64,
    pub violation_count: u64,
    pub legacy_allowed_count: u64,
    pub fail_on_any_seam_violation: bool,
    pub status: String,
    pub pass: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRejectedBaselineV1 {
    pub model_sha256: String,
    pub visible_surface_semantic_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelSemanticDeltaReportV1 {
    pub schema_version: u32,
    pub model_bytes_changed: bool,
    pub visible_surface_changed: bool,
    pub export_delta_proven: bool,
}

pub fn evaluate_visible_anchor_trajectory_v1(
    role: &str,
    clip_name: &str,
    samples: &[VisibleAnchorTrajectorySampleV1],
    minimum_amplitude_ratio: f32,
    minimum_trajectory_alignment: f32,
) -> Result<VisibleAnchorTrajectoryReportV1, ReferenceSupermodelMotionErrorV2> {
    if role.trim().is_empty()
        || clip_name.trim().is_empty()
        || samples.is_empty()
        || !minimum_amplitude_ratio.is_finite()
        || !(0.0..=1.0).contains(&minimum_amplitude_ratio)
        || !minimum_trajectory_alignment.is_finite()
        || !(-1.0..=1.0).contains(&minimum_trajectory_alignment)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-VISIBLE-ANCHOR-TRAJECTORY-INPUT",
            "visibleAnchorTrajectory",
            "trajectory evaluation requires a role, clip, a non-empty sample set and finite normalized thresholds",
        ));
    }
    let mut previous_time = f32::NEG_INFINITY;
    for (index, sample) in samples.iter().enumerate() {
        if !sample.time_seconds.is_finite()
            || sample.time_seconds <= previous_time
            || sample
                .surface_centroid
                .iter()
                .chain(&sample.controller_centroid)
                .any(|value| !value.is_finite())
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-VISIBLE-ANCHOR-TRAJECTORY-SAMPLE",
                format!("visibleAnchorTrajectory.samples[{index}]"),
                "sample times must be strictly increasing and every centroid finite",
            ));
        }
        previous_time = sample.time_seconds;
    }
    let amplitude = |select: fn(&VisibleAnchorTrajectorySampleV1) -> [f32; 3]| {
        samples
            .iter()
            .enumerate()
            .flat_map(|(left, a)| {
                samples
                    .iter()
                    .skip(left + 1)
                    .map(move |b| distance(select(a), select(b)))
            })
            .fold(0.0_f32, f32::max)
    };
    let surface_amplitude = amplitude(|sample| sample.surface_centroid);
    let controller_amplitude = amplitude(|sample| sample.controller_centroid);
    let amplitude_ratio = if controller_amplitude > 1.0e-7 {
        surface_amplitude / controller_amplitude
    } else {
        0.0
    };
    let mut dot_sum = 0.0_f64;
    let mut surface_norm_sum = 0.0_f64;
    let mut controller_norm_sum = 0.0_f64;
    for pair in samples.windows(2) {
        let surface = std::array::from_fn::<_, 3, _>(|axis| {
            pair[1].surface_centroid[axis] - pair[0].surface_centroid[axis]
        });
        let controller = std::array::from_fn::<_, 3, _>(|axis| {
            pair[1].controller_centroid[axis] - pair[0].controller_centroid[axis]
        });
        dot_sum += f64::from(dot(surface, controller));
        surface_norm_sum += f64::from(dot(surface, surface));
        controller_norm_sum += f64::from(dot(controller, controller));
    }
    let trajectory_alignment = if surface_norm_sum > 1.0e-14 && controller_norm_sum > 1.0e-14 {
        (dot_sum / (surface_norm_sum.sqrt() * controller_norm_sum.sqrt())) as f32
    } else {
        0.0
    };
    let status = if controller_amplitude <= 1.0e-7 {
        "BLOCKED_CONTROLLER_STATIC"
    } else if amplitude_ratio < minimum_amplitude_ratio {
        "BLOCKED_AMPLITUDE"
    } else if trajectory_alignment < minimum_trajectory_alignment {
        "BLOCKED_TRAJECTORY"
    } else {
        "PASS"
    };
    Ok(VisibleAnchorTrajectoryReportV1 {
        schema_version: 1,
        role: role.to_owned(),
        clip_name: clip_name.to_owned(),
        sample_count: samples.len(),
        surface_amplitude,
        controller_amplitude,
        amplitude_ratio,
        trajectory_alignment,
        status: status.to_owned(),
        pass: status == "PASS",
    })
}

pub fn evaluate_surface_seam_gate_v1(
    sample_count: u64,
    violation_count: u64,
    legacy_max_fraction: f32,
    fail_on_any_seam_violation: bool,
) -> Result<SurfaceSeamGateReportV1, ReferenceSupermodelMotionErrorV2> {
    if sample_count == 0
        || violation_count > sample_count
        || !legacy_max_fraction.is_finite()
        || !(0.0..=1.0).contains(&legacy_max_fraction)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-SEAM-GATE-INPUT",
            "surfaceSeamGate",
            "seam gate requires a non-empty sample domain, bounded count and normalized fraction",
        ));
    }
    let legacy_allowed_count =
        (sample_count as f64 * f64::from(legacy_max_fraction)).floor() as u64;
    let pass = if fail_on_any_seam_violation {
        violation_count == 0
    } else {
        violation_count <= legacy_allowed_count
    };
    Ok(SurfaceSeamGateReportV1 {
        schema_version: 1,
        sample_count,
        violation_count,
        legacy_allowed_count,
        fail_on_any_seam_violation,
        status: if pass { "PASS" } else { "BLOCKED_VISIBLE_SEAM" }.to_owned(),
        pass,
    })
}

pub fn evaluate_reference_supermodel_semantic_delta_v1(
    baseline: &ReferenceSupermodelRejectedBaselineV1,
    model_sha256: &str,
    visible_surface_semantic_sha256: &str,
) -> Result<ReferenceSupermodelSemanticDeltaReportV1, ReferenceSupermodelMotionErrorV2> {
    let valid_sha = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if !valid_sha(&baseline.model_sha256)
        || !valid_sha(&baseline.visible_surface_semantic_sha256)
        || !valid_sha(model_sha256)
        || !valid_sha(visible_surface_semantic_sha256)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-SEMANTIC-DELTA-SHA256",
            "semanticDelta",
            "baseline and candidate identities require exact lowercase SHA-256 values",
        ));
    }
    let model_bytes_changed = baseline.model_sha256 != model_sha256;
    let visible_surface_changed =
        baseline.visible_surface_semantic_sha256 != visible_surface_semantic_sha256;
    Ok(ReferenceSupermodelSemanticDeltaReportV1 {
        schema_version: 1,
        model_bytes_changed,
        visible_surface_changed,
        export_delta_proven: model_bytes_changed && visible_surface_changed,
    })
}

/// Hashes only renderer-visible vertices, topology and normalized weights that
/// are driven by the named repair-area controllers. Resource names and the
/// rest of the MDL are deliberately excluded, so a resref-only rebuild cannot
/// masquerade as a tail/surface correction.
pub fn visible_controller_surface_semantic_sha256_v1(
    target: &InspectionReport,
    controller_names: &[String],
) -> Result<String, ReferenceSupermodelMotionErrorV2> {
    if controller_names.is_empty() || controller_names.iter().any(|name| name.trim().is_empty()) {
        return Err(motion_error(
            "M2A-SUPERMODEL-VISIBLE-SURFACE-CONTROLLERS-INVALID",
            "controllerNames",
            "visible-surface identity requires at least one named controller",
        ));
    }
    let flattened = flatten_nodes_in_tree_order(&target.node_tree.roots);
    let requested = controller_names
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::<serde_json::Value>::new();
    for node in &flattened {
        let (Some(mesh), Some(skin)) = (&node.mesh, &node.skin) else {
            continue;
        };
        if skin.node_to_bone_map.len() != flattened.len()
            || skin.vertex_weights.len() != mesh.vertices.len()
            || skin.bone_references.len() != mesh.vertices.len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-VISIBLE-SURFACE-SKIN-LAYOUT",
                "target.nodeTree.skin",
                "visible-surface identity requires complete renderer tree and skin rows",
            ));
        }
        let controller_by_reference = skin
            .node_to_bone_map
            .iter()
            .enumerate()
            .filter_map(|(ordinal, &reference)| {
                (reference >= 0
                    && requested.contains(&flattened[ordinal].name.to_ascii_lowercase()))
                .then(|| {
                    (
                        reference as u16,
                        flattened[ordinal].name.to_ascii_lowercase(),
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        let mut selected_vertices = BTreeSet::new();
        for (vertex_index, ((position, weights), references)) in mesh
            .vertices
            .iter()
            .zip(&skin.vertex_weights)
            .zip(&skin.bone_references)
            .enumerate()
        {
            let mut normalized = BTreeMap::<String, u32>::new();
            for lane in 0..4 {
                if let Some(controller) = controller_by_reference.get(&references[lane]) {
                    if weights[lane] > 0.0 {
                        normalized.insert(controller.clone(), weights[lane].to_bits());
                    }
                }
            }
            if normalized.is_empty() {
                continue;
            }
            selected_vertices.insert(vertex_index as u16);
            rows.push(serde_json::json!({
                "skinTreeOrdinal": flattened.iter().position(|candidate| candidate.offset == node.offset),
                "vertexIndex": vertex_index,
                "positionBits": [position.x.to_bits(), position.y.to_bits(), position.z.to_bits()],
                "controllerWeightBits": normalized,
            }));
        }
        for (face_index, face) in mesh.faces.iter().enumerate() {
            if face
                .vertex_indices
                .iter()
                .any(|index| selected_vertices.contains(index))
            {
                rows.push(serde_json::json!({
                    "skinTreeOrdinal": flattened.iter().position(|candidate| candidate.offset == node.offset),
                    "faceIndex": face_index,
                    "indices": face.vertex_indices,
                }));
            }
        }
    }
    if rows.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-VISIBLE-SURFACE-EMPTY",
            "target.nodeTree.skin",
            "no renderer-visible surface is weighted to the requested controllers",
        ));
    }
    serde_json::to_vec(&rows)
        .map(|bytes| hex_sha256(&bytes))
        .map_err(|source| {
            motion_error(
                "M2A-SUPERMODEL-VISIBLE-SURFACE-SERIALIZE",
                "target.nodeTree.skin",
                source.to_string(),
            )
        })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelMotionBuildReportV2 {
    pub schema_version: u32,
    pub compatibility_level: ReferenceSupermodelCompatibilityLevelV2,
    pub supermodel_clip_lookup: bool,
    pub topology_compatible: bool,
    pub bind_pose_compatible: bool,
    pub skin_bind_compatible: bool,
    pub motion_correction_applied: bool,
    pub clip_inventory_verified: bool,
    pub motion_compatible: bool,
    pub source_sha256: String,
    pub source_rig_sha256: String,
    pub corrected_rig_sha256: String,
    pub motion_contract_sha256: String,
    pub supermodel_resref: String,
    pub model_resource_resref: String,
    pub model_sha256: String,
    pub material_status: AuroraMaterialCompileStatusV1,
    pub material_resource_count: usize,
    pub required_clip_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelCarrierReadbackCoverageV3 {
    pub schema_version: u32,
    pub required_carrier_count: usize,
    pub output_carrier_count: usize,
    pub missing_carrier_names: Vec<String>,
    pub extra_carrier_names: Vec<String>,
    pub incompatible_carrier_names: Vec<String>,
    pub full_carrier_coverage: bool,
    pub required_joint_coverage: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureMaterialSemanticStatusV1 {
    Ready,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureMaterialResourceExpectationV1 {
    pub resref: String,
    pub resource_type: u16,
    pub packaged: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureMaterialSemanticSlotV1 {
    pub source_material_id: u32,
    pub output_slot: u32,
    pub diffuse_output_resref: String,
    pub normal_output_resref: Option<String>,
    pub specular_output_resref: Option<String>,
    pub material_output_resref: Option<String>,
    pub base_color_factor: [f32; 4],
    pub base_color_texture_id: Option<u32>,
    pub base_color_tex_coord_set: Option<u32>,
    pub normal_texture_id: Option<u32>,
    pub normal_tex_coord_set: Option<u32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture_id: Option<u32>,
    pub metallic_roughness_tex_coord_set: Option<u32>,
    pub alpha_mode: String,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: bool,
    pub channel_dispositions: Vec<AuroraMaterialFidelityEntryV1>,
    pub resources: Vec<CreatureMaterialResourceExpectationV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureMaterialSemanticReportV1 {
    pub schema_version: u32,
    pub status: CreatureMaterialSemanticStatusV1,
    pub source_image_count: usize,
    pub source_images: Vec<CreatureSourceImageDispositionV1>,
    pub packaged_resource_count: usize,
    pub slots: Vec<CreatureMaterialSemanticSlotV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureSourceImageDispositionV1 {
    pub source_image_id: u32,
    pub source_texture_ids: Vec<u32>,
    pub source_channels: Vec<String>,
    pub output_resrefs: Vec<String>,
    pub disposition: String,
}

#[derive(Debug)]
pub struct ReferenceSupermodelMotionArtifactV2 {
    pub correction: ReferenceSupermodelCorrectionArtifactV1,
    pub conversion: ProfileAConversionOutcomeV1,
    pub model: BinaryMdlArtifactV1,
    pub material_compilation: AuroraMaterialCompilationSetV1,
    pub material_package: AuroraMaterialPackageV1,
    pub material_extension: MdlMaterialExtensionReportV1,
    pub material_semantics: CreatureMaterialSemanticReportV1,
    pub motion_quality: InheritedMotionQualityReportV1,
    pub report: ReferenceSupermodelMotionBuildReportV2,
}

#[derive(Debug)]
pub struct ReferenceSupermodelCreatureHakArtifactV1 {
    pub hak: HakArtifactV1,
    pub resource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelClassicMotionBuildReportV1 {
    pub schema_version: u32,
    pub motion_compatible: bool,
    pub bind_pose_compatible: bool,
    pub skin_bind_compatible: bool,
    pub source_sha256: String,
    pub source_rig_sha256: String,
    pub output_rig_sha256: String,
    pub motion_contract_sha256: String,
    pub supermodel_resref: String,
    pub model_resource_resref: String,
    pub model_sha256: String,
    pub material_profile: String,
    pub material_extension_applied: bool,
    pub tangent_stream_count: usize,
    pub material_resource_count: usize,
    pub runtime_readiness: String,
    pub required_clip_count: usize,
    pub carrier_coverage: ReferenceSupermodelCarrierReadbackCoverageV3,
    pub skin_influence_coverage: bool,
    pub allowed_bone_count: usize,
    pub active_weighted_bone_count: usize,
    pub unweighted_required_joint_names: Vec<String>,
    pub passive_unweighted_joint_names: Vec<String>,
    pub motion_weight_refinement_applied: bool,
    pub motion_weight_refinement_attempted_round_count: usize,
    pub motion_weight_refinement_examined_edge_count: usize,
    pub motion_weight_refinement_measured_triangle_count: usize,
    pub motion_weight_refinement_smoothing_candidate_group_count: usize,
    pub motion_weight_refinement_smoothing_shared_support_group_count: usize,
    pub motion_weight_refinement_smoothing_changed_group_count: usize,
    pub motion_weight_refinement_micro_triangle_candidate_count: usize,
    pub motion_weight_refinement_micro_triangle_too_large_count: usize,
    pub motion_weight_refinement_micro_triangle_protected_support_count: usize,
    pub motion_weight_refinement_micro_triangle_nonlocal_support_count: usize,
    pub motion_weight_refinement_micro_triangle_no_common_support_count: usize,
    pub motion_weight_refinement_micro_triangle_changed_count: usize,
    pub motion_weight_refinement_catastrophic_edge_candidate_count: usize,
    pub motion_weight_refinement_catastrophic_edge_count: usize,
    pub motion_weight_refinement_catastrophic_edge_topology_skip_count: usize,
    pub motion_weight_refinement_catastrophic_edge_conflict_skip_count: usize,
    pub motion_weight_refinement_catastrophic_anchor_restore_count: usize,
    pub motion_weight_refinement_catastrophic_peak_edge: Option<String>,
    pub motion_weight_refinement_catastrophic_peak_edge_changed: bool,
    pub motion_weight_refinement_backtracked_group_count: usize,
    pub motion_weight_refinement_iteration_count: usize,
    pub motion_weight_refinement_edge_count: usize,
    pub motion_weight_refinement_rigid_component_count: usize,
    pub motion_weight_refinement_projected_triangle_count: usize,
    pub motion_weight_refinement_rejected_round_count: usize,
    pub motion_weight_refinement_last_rejection: Option<String>,
}

#[derive(Debug)]
pub struct ReferenceSupermodelClassicMotionArtifactV1 {
    pub correction: ReferenceSupermodelCorrectionArtifactV1,
    pub conversion: ProfileAConversionOutcomeV1,
    pub model: BinaryMdlArtifactV1,
    pub motion_quality: InheritedMotionQualityReportV1,
    pub report: ReferenceSupermodelClassicMotionBuildReportV1,
}

#[derive(Debug)]
pub struct ReferenceSupermodelMinimalMtrMotionArtifactV1 {
    pub correction: ReferenceSupermodelCorrectionArtifactV1,
    pub conversion: ProfileAConversionOutcomeV1,
    pub model: BinaryMdlArtifactV1,
    pub material_compilation: AuroraMaterialCompilationSetV1,
    pub material_extension: MdlMaterialExtensionReportV1,
    pub mtr_resource: HakResourceInputV1,
    pub motion_quality: InheritedMotionQualityReportV1,
    pub report: ReferenceSupermodelClassicMotionBuildReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelMotionErrorV2 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ReferenceSupermodelMotionErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ReferenceSupermodelMotionErrorV2 {}

pub fn canonical_reference_supermodel_motion_contract_sha256_v2(
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<String, ReferenceSupermodelMotionErrorV2> {
    let mut canonical = contract.clone();
    canonical.content_sha256.clear();
    let bytes = serde_json::to_vec(&canonical).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-CONTRACT-SERIALIZE",
            "contract",
            source.to_string(),
        )
    })?;
    Ok(hex_sha256(&bytes))
}

/// Builds a reusable inherited-motion contract from the exact neutral bind
/// stored in an inspected binary MDL.  Unlike the historical clean-room
/// envelope, this function does not invent carrier positions or orientations.
/// It copies no render or animation payload into the generated model; the
/// decoded matrices are structural compatibility metadata required by NWN's
/// name-based supermodel controller inheritance.
fn surface_free_global_motion_ancestor_v1(
    part: usize,
    has_reference_skin: bool,
    has_own_surface: bool,
    parents: &[Option<usize>],
    dynamic_render_parts: &[usize],
) -> bool {
    !has_reference_skin
        && !has_own_surface
        && dynamic_render_parts.len() >= 2
        && dynamic_render_parts.iter().all(|&render_part| {
            let mut current = parents[render_part];
            for _ in 0..parents.len() {
                let Some(parent) = current else {
                    return false;
                };
                if parent == part {
                    return true;
                }
                current = parents[parent];
            }
            false
        })
}

pub fn build_exact_reference_supermodel_motion_contract_v3(
    reference: &InspectionReport,
    options: &ReferenceSupermodelExactContractOptionsV3,
) -> Result<ReferenceSupermodelMotionContractV2, ReferenceSupermodelMotionErrorV2> {
    if !logical_id(&options.contract_id)
        || !is_resref(&options.supermodel_resref)
        || options.supermodel_resref.eq_ignore_ascii_case("NULL")
        || !is_lower_sha256(&options.source_model_sha256)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-EXACT-CONTRACT-OPTIONS-INVALID",
            "options",
            "exact contract requires a logical id, non-NULL resref and lowercase SHA-256",
        ));
    }
    validate_motion_tolerances_v2(&options.tolerances)?;
    if reference.node_tree.roots.len() != 1 || reference.node_tree.node_count == 0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            "reference.nodeTree",
            "exact bind extraction requires one non-empty reference node tree",
        ));
    }
    if reference.model.classification != 4 {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-CLASSIFICATION-MISMATCH",
            "reference.model.classification",
            "exact inherited creature motion requires classification 4",
        ));
    }

    let all_flattened = flatten_nodes_in_tree_order(&reference.node_tree.roots);
    if all_flattened.len() != reference.node_tree.node_count {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            "reference.nodeTree.nodeCount",
            "flattened reference node count differs from the inspected count",
        ));
    }
    let semantics = options
        .semantic_nodes
        .iter()
        .map(|node| (node.node_name.to_ascii_lowercase(), node))
        .collect::<BTreeMap<_, _>>();
    if semantics.len() != options.semantic_nodes.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-SEMANTIC-NODE-DUPLICATE",
            "options.semanticNodes",
            "semantic node names must be case-fold unique",
        ));
    }

    let required_clips = options
        .required_clips
        .iter()
        .map(|required| {
            reference
                .animations
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(required))
                .ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-MOTION-CLIP-MISSING",
                        "options.requiredClips",
                        format!("required inherited clip {required} is absent"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Build the carrier inventory from the complete inherited clip corpus,
    // not from a locomotion subset. Identity is name+parent structural key so
    // state node numbers from different levels of a resolved chain cannot be
    // mistaken for one another. Every non-render base node is retained,
    // including passive attachment/end nodes; render-only nodes are excluded
    // with an auditable reason.
    let base_by_offset = all_flattened
        .iter()
        .map(|node| (node.offset, *node))
        .collect::<BTreeMap<_, _>>();
    if base_by_offset.len() != all_flattened.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            "reference.nodeTree.nodes.offset",
            "reference nodes require unique serialized offsets",
        ));
    }
    let structural_key = |node: &NodeReport,
                          by_offset: &BTreeMap<u32, &NodeReport>|
     -> Result<(String, String), ReferenceSupermodelMotionErrorV2> {
        let name = if node.parent_offset.is_none() {
            "<root>".to_owned()
        } else {
            node.name.to_ascii_lowercase()
        };
        let parent = match node.parent_offset {
            None => "<none>".to_owned(),
            Some(offset) => {
                let parent = by_offset.get(&offset).ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
                        "reference.nodeTree.parentOffset",
                        "carrier parent offset is absent",
                    )
                })?;
                if parent.parent_offset.is_none() {
                    "<root>".to_owned()
                } else {
                    parent.name.to_ascii_lowercase()
                }
            }
        };
        Ok((name, parent))
    };
    let mut base_by_key = BTreeMap::new();
    for node in &all_flattened {
        let key = structural_key(node, &base_by_offset)?;
        if base_by_key.insert(key.clone(), *node).is_some() {
            return Err(motion_error(
                "M2A-SUPERMODEL-CARRIER-IDENTITY-AMBIGUOUS",
                "reference.nodeTree",
                format!("carrier structural key {key:?} occurs more than once"),
            ));
        }
    }
    let mut clip_nodes_by_key = Vec::with_capacity(required_clips.len());
    for clip in &required_clips {
        let flattened = flatten_nodes_in_tree_order(&clip.node_tree.roots);
        let by_offset = flattened
            .iter()
            .map(|node| (node.offset, *node))
            .collect::<BTreeMap<_, _>>();
        let mut by_key = BTreeMap::new();
        for node in flattened {
            let key = structural_key(node, &by_offset)?;
            if by_key.insert(key.clone(), node).is_some() {
                return Err(motion_error(
                    "M2A-SUPERMODEL-CLIP-CARRIER-IDENTITY-AMBIGUOUS",
                    format!("reference.animations[{}]", clip.name),
                    format!("animation structural key {key:?} occurs more than once"),
                ));
            }
            if !base_by_key.contains_key(&key)
                && ["position", "orientation", "scale"]
                    .into_iter()
                    .any(|controller| has_decoded_controller(node, controller))
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-CLIP-CARRIER-MISSING-FROM-BASE",
                    format!("reference.animations[{}].nodeTree", clip.name),
                    format!("controlled carrier {key:?} has no compatible base node"),
                ));
            }
        }
        clip_nodes_by_key.push(by_key);
    }
    let mut carrier_numbers = all_flattened
        .iter()
        .filter(|node| {
            let key = structural_key(node, &base_by_offset)
                .expect("base hierarchy was validated before classification");
            let controlled = clip_nodes_by_key.iter().any(|nodes| {
                nodes.get(&key).is_some_and(|node| {
                    ["position", "orientation", "scale"]
                        .into_iter()
                        .any(|controller| has_decoded_controller(node, controller))
                })
            });
            node.mesh.is_none() && node.skin.is_none() || controlled
        })
        .map(|node| node.number)
        .collect::<BTreeSet<_>>();
    let mut changed = true;
    while changed {
        changed = false;
        for node in &all_flattened {
            if carrier_numbers.contains(&node.number) {
                if let Some(parent) = node
                    .parent_offset
                    .and_then(|offset| base_by_offset.get(&offset))
                {
                    changed |= carrier_numbers.insert(parent.number);
                }
            }
        }
    }
    let mut exclusion_reason_by_number = BTreeMap::<u32, String>::new();
    let controlled_numbers = all_flattened
        .iter()
        .filter(|node| {
            let key = structural_key(node, &base_by_offset)
                .expect("base hierarchy was validated before duplicate classification");
            clip_nodes_by_key.iter().any(|nodes| {
                nodes.get(&key).is_some_and(|node| {
                    ["position", "orientation", "scale"]
                        .into_iter()
                        .any(|controller| has_decoded_controller(node, controller))
                })
            })
        })
        .map(|node| node.number)
        .collect::<BTreeSet<_>>();
    let carrier_children = all_flattened
        .iter()
        .filter(|node| carrier_numbers.contains(&node.number))
        .filter_map(|node| node.parent_offset)
        .collect::<BTreeSet<_>>();
    let mut duplicate_names = BTreeMap::<String, Vec<&NodeReport>>::new();
    for node in all_flattened
        .iter()
        .filter(|node| carrier_numbers.contains(&node.number))
    {
        duplicate_names
            .entry(node.name.to_ascii_lowercase())
            .or_default()
            .push(*node);
    }
    for (name, candidates) in duplicate_names
        .into_iter()
        .filter(|(_, candidates)| candidates.len() > 1)
    {
        let indispensable = candidates
            .iter()
            .copied()
            .filter(|node| {
                controlled_numbers.contains(&node.number) || carrier_children.contains(&node.offset)
            })
            .collect::<Vec<_>>();
        if indispensable.len() != 1 {
            return Err(motion_error(
                "M2A-SUPERMODEL-CARRIER-NAME-AMBIGUOUS",
                "reference.nodeTree",
                format!(
                    "carrier name {name:?} resolves to {} indispensable controlled/structural nodes; exact Aurora name inheritance cannot select one compatible carrier",
                    indispensable.len()
                ),
            ));
        }
        let keep = indispensable[0].number;
        for node in candidates {
            if node.number == keep {
                continue;
            }
            carrier_numbers.remove(&node.number);
            exclusion_reason_by_number.insert(
                node.number,
                "DUPLICATE_PASSIVE_TERMINAL_NAME_SHADOWED_BY_CONTROLLED_OR_STRUCTURAL_CARRIER"
                    .to_owned(),
            );
        }
    }
    let carrier_exclusions = all_flattened
        .iter()
        .filter(|node| !carrier_numbers.contains(&node.number))
        .map(|node| ReferenceSupermodelCarrierExclusionV3 {
            node_name: node.name.clone(),
            parent_name: node
                .parent_offset
                .and_then(|offset| base_by_offset.get(&offset))
                .map(|parent| parent.name.clone()),
            reason: exclusion_reason_by_number
                .get(&node.number)
                .cloned()
                .unwrap_or_else(|| {
                    "RENDER_ONLY_MESH_NODE_NOT_REFERENCED_BY_INHERITED_TRANSFORM_CONTROLLERS"
                        .to_owned()
                }),
        })
        .collect::<Vec<_>>();
    let flattened = all_flattened
        .into_iter()
        .filter(|node| carrier_numbers.contains(&node.number))
        .collect::<Vec<_>>();
    let part_by_offset = flattened
        .iter()
        .enumerate()
        .map(|(part, node)| (node.offset, part as u32))
        .collect::<BTreeMap<_, _>>();

    let mut controlling_by_part = Vec::with_capacity(flattened.len());
    let mut dynamic_by_part = Vec::with_capacity(flattened.len());
    let mut controller_nodes_by_part = Vec::with_capacity(flattened.len());
    for node in &flattened {
        let key = structural_key(node, &base_by_offset)?;
        let rows = required_clips
            .iter()
            .zip(&clip_nodes_by_key)
            .filter_map(|(clip, nodes)| nodes.get(&key).map(|node| (clip.name.as_str(), *node)))
            .collect::<Vec<_>>();
        controlling_by_part.push(
            rows.iter()
                .filter(|(_, node)| {
                    ["position", "orientation", "scale"]
                        .into_iter()
                        .any(|controller| has_decoded_controller(node, controller))
                })
                .map(|(clip, _)| (*clip).to_owned())
                .collect::<Vec<_>>(),
        );
        dynamic_by_part.push(
            rows.iter()
                .filter(|(_, node)| node_has_dynamic_transform_controller_v3(node))
                .map(|(clip, _)| (*clip).to_owned())
                .collect::<Vec<_>>(),
        );
        controller_nodes_by_part.push(rows.into_iter().map(|(_, node)| node).collect::<Vec<_>>());
    }
    let dynamic_render_parts = dynamic_by_part
        .iter()
        .enumerate()
        .filter_map(|(part, clips)| {
            (!clips.is_empty() && flattened[part].mesh.is_some()).then_some(part)
        })
        .collect::<Vec<_>>();
    let has_reference_skin = flattened.iter().any(|node| node.skin.is_some());
    let parent_parts = flattened
        .iter()
        .map(|node| {
            node.parent_offset
                .and_then(|offset| part_by_offset.get(&offset))
                .map(|part| *part as usize)
        })
        .collect::<Vec<_>>();
    let global_transform_only = |part: usize| {
        surface_free_global_motion_ancestor_v1(
            part,
            has_reference_skin,
            flattened[part].mesh.is_some(),
            &parent_parts,
            &dynamic_render_parts,
        )
    };
    // A non-rendering common ancestor of all rigid surfaces transmits global
    // motion through its children. It remains in the exact hierarchy and clip
    // corpus, but must not steal surface vertices merely to satisfy coverage.
    // For skinned references retain the conservative inventory: such a node
    // may be explicitly referenced by the retail skin weights.
    let skin_relevant_parts = dynamic_by_part
        .iter()
        .enumerate()
        .filter_map(|(part, clips)| {
            (part > 0 && !clips.is_empty() && !global_transform_only(part)).then_some(part as u32)
        })
        .collect::<BTreeSet<_>>();
    let mut structural_ancestor_parts = BTreeSet::new();
    for &skin_part in &skin_relevant_parts {
        let mut current = flattened[skin_part as usize].parent_offset;
        while let Some(offset) = current {
            let Some(&parent) = part_by_offset.get(&offset) else {
                break;
            };
            structural_ancestor_parts.insert(parent);
            current = flattened[parent as usize].parent_offset;
        }
    }
    let mut children_by_part = vec![Vec::<u32>::new(); flattened.len()];
    for (part, node) in flattened.iter().enumerate() {
        if let Some(parent) = node
            .parent_offset
            .and_then(|offset| part_by_offset.get(&offset))
        {
            children_by_part[*parent as usize].push(part as u32);
        }
    }
    let terminal_chain_depth = |mut part: u32| {
        let mut depth = 0_usize;
        while let Some(parent) = flattened[part as usize]
            .parent_offset
            .and_then(|offset| part_by_offset.get(&offset))
            .copied()
        {
            depth += 1;
            if children_by_part[parent as usize].len() != 1 || parent == 0 {
                break;
            }
            part = parent;
        }
        depth
    };

    let mut nodes = Vec::with_capacity(flattened.len());
    for (part, node) in flattened.iter().enumerate() {
        let parent_part_number = node
            .parent_offset
            .map(|offset| {
                part_by_offset.get(&offset).copied().ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
                        format!("reference.nodeTree.nodes[{part}].parentOffset"),
                        "reference node parent offset is absent from tree traversal",
                    )
                })
            })
            .transpose()?;
        if part == 0 {
            if parent_part_number.is_some() {
                return Err(motion_error(
                    "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
                    "reference.nodeTree.nodes[0].parentOffset",
                    "reference root must not have a parent",
                ));
            }
        } else if parent_part_number.is_none_or(|parent| parent >= part as u32) {
            return Err(motion_error(
                "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
                format!("reference.nodeTree.nodes[{part}].parentOffset"),
                "reference parent must precede its child in tree order",
            ));
        }
        let semantic = semantics.get(&node.name.to_ascii_lowercase()).copied();
        let clip_nodes = &controller_nodes_by_part[part];
        let carrier_class = if skin_relevant_parts.contains(&(part as u32)) {
            ReferenceSupermodelCarrierClassV3::SkinRelevant
        } else if part == 0 || structural_ancestor_parts.contains(&(part as u32)) {
            ReferenceSupermodelCarrierClassV3::PassiveStructural
        } else {
            ReferenceSupermodelCarrierClassV3::PassiveAttachmentOrEnd
        };
        let is_terminal = children_by_part[part].is_empty();
        let chain_depth = terminal_chain_depth(part as u32);
        let structural_role = match carrier_class {
            ReferenceSupermodelCarrierClassV3::SkinRelevant if is_terminal && chain_depth >= 3 => {
                "LIMB_GROUND_CONTACT_TERMINAL"
            }
            ReferenceSupermodelCarrierClassV3::SkinRelevant if is_terminal && chain_depth >= 2 => {
                "APPENDAGE_TERMINAL"
            }
            ReferenceSupermodelCarrierClassV3::SkinRelevant if is_terminal => "TERMINAL_JOINT",
            ReferenceSupermodelCarrierClassV3::SkinRelevant => "ANIMATED_CHAIN_JOINT",
            ReferenceSupermodelCarrierClassV3::PassiveStructural => "PASSIVE_STRUCTURAL",
            ReferenceSupermodelCarrierClassV3::PassiveAttachmentOrEnd => {
                "PASSIVE_ATTACHMENT_OR_END"
            }
        }
        .to_owned();
        let derived_role = if carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant {
            Some(
                if let Some(role) = semantic.and_then(|node| node.anchor_role.clone()) {
                    role
                } else if structural_role == "LIMB_GROUND_CONTACT_TERMINAL" {
                    format!("limb_paw_{part}")
                } else if structural_role == "APPENDAGE_TERMINAL" {
                    format!("appendage_{part}")
                } else {
                    format!("joint_{part}")
                },
            )
        } else {
            None
        };
        nodes.push(ReferenceSupermodelMotionNodeV2 {
            part_number: part as u32,
            name: node.name.clone(),
            parent_part_number,
            carrier_bind_local_matrix: exact_node_bind_local_matrix_v3(node, part)?,
            position_controller_required: clip_nodes
                .iter()
                .any(|node| has_decoded_controller(node, "position")),
            orientation_controller_required: clip_nodes
                .iter()
                .any(|node| has_decoded_controller(node, "orientation")),
            scale_controller_required: clip_nodes
                .iter()
                .any(|node| has_decoded_controller(node, "scale")),
            anchor_role: derived_role,
            joint_axis: semantic.and_then(|node| node.joint_axis),
            carrier_class,
            structural_role,
            controlling_clips: controlling_by_part[part].clone(),
            dynamic_clips: dynamic_by_part[part].clone(),
        });
    }
    // Depth alone cannot identify a ground-contact terminal: ears, horns and
    // head appendages may be animated terminal chains just as deep as legs.
    // Keep the structural discovery family-independent, but require a paw
    // candidate to live in the lower quarter of the exact carrier bind.
    let carrier_locals = nodes
        .iter()
        .map(|node| node.carrier_bind_local_matrix)
        .collect::<Vec<_>>();
    let carrier_parents = nodes
        .iter()
        .map(|node| node.parent_part_number.map(|part| part as usize))
        .collect::<Vec<_>>();
    let carrier_worlds = ordered_world_matrices(
        &carrier_locals,
        &carrier_parents,
        "contract.nodes.groundContactClassification",
    )?;
    let skin_heights = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
        .map(|(index, _)| carrier_worlds[index][14])
        .collect::<Vec<_>>();
    if let (Some(minimum), Some(maximum)) = (
        skin_heights.iter().copied().min_by(f32::total_cmp),
        skin_heights.iter().copied().max_by(f32::total_cmp),
    ) {
        let ground_cutoff = minimum + (maximum - minimum) * 0.25;
        for (index, node) in nodes.iter_mut().enumerate() {
            if node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL"
                && carrier_worlds[index][14] > ground_cutoff
            {
                node.structural_role = "APPENDAGE_TERMINAL".to_owned();
                if !semantics.contains_key(&node.name.to_ascii_lowercase()) {
                    node.anchor_role = Some(format!("appendage_{}", node.part_number));
                }
            }
        }
    }
    let unknown_semantics = semantics
        .keys()
        .filter(|name| {
            !flattened
                .iter()
                .any(|node| node.name.eq_ignore_ascii_case(name))
        })
        .cloned()
        .collect::<Vec<_>>();
    if !unknown_semantics.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-SEMANTIC-NODE-MISSING",
            "options.semanticNodes",
            format!(
                "semantic annotations reference absent nodes: {}",
                unknown_semantics.join(", ")
            ),
        ));
    }

    let exact_profile_bytes = serde_json::to_vec(&nodes).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-EXACT-CONTRACT-SERIALIZE",
            "contract.nodes",
            source.to_string(),
        )
    })?;
    let mut contract = ReferenceSupermodelMotionContractV2 {
        schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
        contract_id: options.contract_id.clone(),
        content_sha256: String::new(),
        supermodel_resref: options.supermodel_resref.clone(),
        source_model_sha256: options.source_model_sha256.clone(),
        inspected_read_only: true,
        no_payload_copied: true,
        classification: reference.model.classification,
        animation_scale: reference.model.animation_scale,
        clean_room_profile_id: "exact-inspected-supermodel-bind-v3".to_owned(),
        clean_room_profile_sha256: hex_sha256(&exact_profile_bytes),
        nodes,
        carrier_exclusions,
        required_clips: options.required_clips.clone(),
        required_events: options.required_events.clone(),
        tolerances: options.tolerances.clone(),
    };
    contract.content_sha256 = canonical_reference_supermodel_motion_contract_sha256_v2(&contract)?;
    validate_motion_contract_v2(&contract)?;
    validate_reference_bind_contract_v3(&contract, reference)?;
    Ok(contract)
}

pub fn default_reference_supermodel_motion_tolerances_v2() -> ReferenceSupermodelMotionTolerancesV2
{
    ReferenceSupermodelMotionTolerancesV2 {
        bind_max_abs_error: 1.0e-4,
        edge_soft_min_ratio: 0.5,
        edge_soft_max_ratio: 2.0,
        edge_hard_min_ratio: 0.25,
        edge_hard_max_ratio: 4.0,
        // Soft-quality judgments below one thousandth of the model diagonal
        // are not visually stable on high-density card meshes. Hard edge
        // checks remain active for every finite edge.
        soft_edge_min_bind_diagonal_fraction: 0.01,
        // Profile V3 is a density-independent sampled-surface gate. The
        // previous absolute-zero rules rejected a retail c_dog consumer of
        // c_wolf and made high-density fur card meshes fail by raw count.
        edge_soft_max_fraction: 0.04,
        edge_hard_max_fraction: 0.01,
        triangle_min_area_ratio: 0.05,
        triangle_max_area_ratio: 20.0,
        // Dense render surfaces still require a real triangle-quality domain.
        // The previous 1e-5 threshold skipped every triangle of a 299,783-face
        // Creature and converted "not measured" into PASS. Keep only a tiny
        // numerical degeneracy floor; density is handled by local budgets.
        triangle_min_bind_area_diagonal_squared_fraction: 1.0e-12,
        triangle_area_collapse_max_fraction: 0.0005,
        triangle_area_expansion_max_fraction: 0.001,
        seam_source_max_diagonal_fraction: 0.005,
        seam_output_max_source_multiple: 2.0,
        seam_output_floor_diagonal_fraction: 0.01,
        // A visible split is a product defect, regardless of mesh density.
        // Keep the legacy fraction in the report for diagnostics, but the
        // authoritative admission rule is fail-on-any.
        seam_max_violation_fraction: 0.0,
        fail_on_any_seam_violation: true,
        visible_anchor_min_amplitude_ratio: 0.25,
        visible_anchor_min_trajectory_alignment: 0.25,
        paw_ground_contact_max_height_fraction: 0.01,
        paw_controller_max_position_error_fraction: 0.02,
        clip_start_anchor_max_jump_fraction: 0.05,
    }
}

/// Builds a carrier/correction hierarchy without touching source geometry.
///
/// The first N nodes are exact animation carriers named by the supermodel
/// contract. Every target bone receives one owned correction child. Skin
/// weights are remapped to correction nodes, so inherited controllers animate
/// the carrier layer while the target neutral pose remains explicit.
pub fn build_motion_corrected_rig_v1(
    target: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<ReferenceSupermodelCorrectionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    validate_motion_contract_v2(contract)?;
    validate_target_topology_v2(target, contract)?;
    let source_hash = canonical_profile_sha256(target).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH",
            "targetRig",
            source.to_string(),
        )
    })?;
    if source_hash != target.content_sha256 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH-MISMATCH",
            "targetRig.contentSha256",
            "target rig content hash does not match canonical content",
        ));
    }

    let target_worlds = rig_world_matrices(target)?;
    let carrier_locals = contract
        .nodes
        .iter()
        .map(|node| node.carrier_bind_local_matrix)
        .collect::<Vec<_>>();
    let carrier_parents = contract
        .nodes
        .iter()
        .map(|node| node.parent_part_number.map(|value| value as usize))
        .collect::<Vec<_>>();
    let carrier_worlds =
        ordered_world_matrices(&carrier_locals, &carrier_parents, "contract.nodes")?;

    let max_target_id = target
        .nodes
        .iter()
        .map(|node| node.id)
        .max()
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-RIG-EMPTY",
                "targetRig.nodes",
                "target rig must contain at least one node",
            )
        })?;
    let first_correction_id = max_target_id.checked_add(1).ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-NODE-ID-OVERFLOW",
            "targetRig.nodes",
            "correction node ids overflow u32",
        )
    })?;

    let mut nodes = Vec::with_capacity(target.nodes.len() * 2);
    for (target_node, contract_node) in target.nodes.iter().zip(&contract.nodes) {
        nodes.push(CreatureRigNodeV1 {
            id: target_node.id,
            name: contract_node.name.clone(),
            parent_id: contract_node
                .parent_part_number
                .map(|parent| target.nodes[parent as usize].id),
            bind_local_matrix: contract_node.carrier_bind_local_matrix,
        });
    }

    let mut correction_by_target_id = BTreeMap::new();
    let mut correction_reports = Vec::with_capacity(target.nodes.len());
    let mut neutral_max_abs_error = 0.0_f32;
    for (part, target_node) in target.nodes.iter().enumerate() {
        let correction_id = first_correction_id
            .checked_add(part as u32)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-NODE-ID-OVERFLOW",
                    format!("targetRig.nodes[{part}]"),
                    "correction node id overflows u32",
                )
            })?;
        let carrier_inverse = inverse_affine(carrier_worlds[part]).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-CARRIER-NONINVERTIBLE",
                format!("contract.nodes[{part}].carrierBindLocalMatrix"),
                "carrier world matrix is not an invertible affine transform",
            )
        })?;
        let correction_local = mul_mat4(carrier_inverse, target_worlds[part]);
        let rebuilt = mul_mat4(carrier_worlds[part], correction_local);
        let error = max_abs_matrix_difference(rebuilt, target_worlds[part]);
        neutral_max_abs_error = neutral_max_abs_error.max(error);
        if error > contract.tolerances.bind_max_abs_error {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NEUTRAL-CORRECTION-DIFF",
                format!("correctionNodes[{part}]"),
                format!(
                    "neutral carrier correction error {error} exceeds {}",
                    contract.tolerances.bind_max_abs_error
                ),
            ));
        }
        let correction_name = format!("m2a_corr_{part:02}");
        nodes.push(CreatureRigNodeV1 {
            id: correction_id,
            name: correction_name.clone(),
            parent_id: Some(target_node.id),
            bind_local_matrix: correction_local,
        });
        correction_by_target_id.insert(target_node.id, correction_id);
        correction_reports.push(ReferenceSupermodelCorrectionNodeV1 {
            carrier_node_id: target_node.id,
            carrier_name: contract.nodes[part].name.clone(),
            correction_node_id: correction_id,
            correction_name,
            neutral_max_abs_error: error,
        });
    }

    let mut weighted_reference_remap_count = 0usize;
    let mut segments = target.segments.clone();
    for (segment_index, segment) in segments.iter_mut().enumerate() {
        for bone in &mut segment.allowed_bone_node_ids {
            *bone = *correction_by_target_id.get(bone).ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-BONE-UNMAPPED",
                    format!("targetRig.segments[{segment_index}].allowedBoneNodeIds"),
                    "allowed bone does not belong to the target carrier topology",
                )
            })?;
        }
        for (row_index, row) in segment.reference_weights.iter_mut().enumerate() {
            for influence in row {
                influence.bone_node_id = *correction_by_target_id
                    .get(&influence.bone_node_id)
                    .ok_or_else(|| {
                        motion_error(
                            "M2A-SUPERMODEL-MOTION-BONE-UNMAPPED",
                            format!(
                                "targetRig.segments[{segment_index}].referenceWeights[{row_index}]"
                            ),
                            "weighted bone does not belong to the target carrier topology",
                        )
                    })?;
                weighted_reference_remap_count += 1;
            }
        }
    }

    let mut rig = CreatureRigProfileV1 {
        schema_version: target.schema_version,
        profile_id: "m2a-supermodel-motion-corrected-rig-v1".to_owned(),
        content_sha256: String::new(),
        provenance: target.provenance.clone(),
        target_bounds: target.target_bounds,
        alignment_anchor: target.alignment_anchor,
        nodes,
        segments,
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH",
            "correctedRig",
            source.to_string(),
        )
    })?;
    Ok(ReferenceSupermodelCorrectionArtifactV1 {
        report: ReferenceSupermodelCorrectionReportV1 {
            schema_version: 1,
            source_rig_sha256: target.content_sha256.clone(),
            output_rig_sha256: rig.content_sha256.clone(),
            carrier_node_count: target.nodes.len(),
            correction_node_count: target.nodes.len(),
            weighted_reference_remap_count,
            neutral_max_abs_error,
            source_bind_max_abs_error: target
                .nodes
                .iter()
                .zip(&contract.nodes)
                .map(|(target, carrier)| {
                    max_abs_matrix_difference(
                        target.bind_local_matrix,
                        carrier.carrier_bind_local_matrix,
                    )
                })
                .fold(0.0_f32, f32::max),
            // This legacy correction route has no reference inspection input,
            // therefore the reference bind is deliberately reported as
            // unverified. Keep the numeric field JSON-safe and make the
            // boolean the authority instead of serializing infinity.
            reference_bind_max_abs_error: 0.0,
            reference_bind_verified: false,
            nodes: correction_reports,
        },
        rig,
    })
}

/// Rebinds caller-owned target geometry to the exact named carrier bind pose
/// decoded from `reference`.  Geometry and authored weight rows are preserved;
/// only the named carrier transforms are replaced.  This is the common route
/// for every supermodel and is intentionally independent of its resref.
pub fn build_exact_motion_carrier_rig_v3(
    target: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
) -> Result<ReferenceSupermodelCorrectionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    validate_motion_contract_v2(contract)?;
    validate_reference_bind_contract_v3(contract, reference)?;
    validate_target_topology_v2(target, contract)?;
    let source_hash = canonical_profile_sha256(target).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH",
            "targetRig",
            source.to_string(),
        )
    })?;
    if source_hash != target.content_sha256 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH-MISMATCH",
            "targetRig.contentSha256",
            "target rig content hash does not match canonical content",
        ));
    }

    let source_bind_max_abs_error = target
        .nodes
        .iter()
        .zip(&contract.nodes)
        .map(|(target, exact)| {
            max_abs_matrix_difference(target.bind_local_matrix, exact.carrier_bind_local_matrix)
        })
        .fold(0.0_f32, f32::max);
    let mut rig = target.clone();
    rig.profile_id = "m2a-exact-supermodel-carrier-rig-v3".to_owned();
    for (part, (node, exact)) in rig.nodes.iter_mut().zip(&contract.nodes).enumerate() {
        node.name = exact.name.clone();
        node.parent_id = exact
            .parent_part_number
            .map(|parent| target.nodes[parent as usize].id);
        node.bind_local_matrix = exact.carrier_bind_local_matrix;
        if part == 0 {
            node.parent_id = None;
        }
    }
    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH",
            "exactCarrierRig",
            source.to_string(),
        )
    })?;
    let reference_bind_max_abs_error = exact_rig_reference_bind_error_v3(&rig, reference)?;
    if reference_bind_max_abs_error > contract.tolerances.bind_max_abs_error {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH",
            "exactCarrierRig.nodes",
            format!(
                "emitted exact carrier bind error {reference_bind_max_abs_error} exceeds {}",
                contract.tolerances.bind_max_abs_error
            ),
        ));
    }
    let nodes = target
        .nodes
        .iter()
        .zip(&contract.nodes)
        .map(|(target_node, exact)| ReferenceSupermodelCorrectionNodeV1 {
            carrier_node_id: target_node.id,
            carrier_name: exact.name.clone(),
            correction_node_id: target_node.id,
            correction_name: exact.name.clone(),
            neutral_max_abs_error: 0.0,
        })
        .collect();
    Ok(ReferenceSupermodelCorrectionArtifactV1 {
        report: ReferenceSupermodelCorrectionReportV1 {
            schema_version: 3,
            source_rig_sha256: target.content_sha256.clone(),
            output_rig_sha256: rig.content_sha256.clone(),
            carrier_node_count: target.nodes.len(),
            correction_node_count: 0,
            weighted_reference_remap_count: 0,
            neutral_max_abs_error: reference_bind_max_abs_error,
            source_bind_max_abs_error,
            reference_bind_max_abs_error,
            reference_bind_verified: true,
            nodes,
        },
        rig,
    })
}

/// Uses the owned source-fitted rig as the named animation carrier hierarchy.
/// This preserves the exact supermodel node topology and avoids inserting
/// correction children between ordered carrier branches. The report retains
/// the resolved binding map so the shared anatomical-anchor oracle can inspect
/// either direct or correction-node routes uniformly.
pub fn build_direct_motion_carrier_rig_v2(
    target: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<ReferenceSupermodelCorrectionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    validate_motion_contract_v2(contract)?;
    validate_target_topology_v2(target, contract)?;
    let source_hash = canonical_profile_sha256(target).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH",
            "targetRig",
            source.to_string(),
        )
    })?;
    if source_hash != target.content_sha256 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH-MISMATCH",
            "targetRig.contentSha256",
            "target rig content hash does not match canonical content",
        ));
    }
    let nodes = target
        .nodes
        .iter()
        .zip(&contract.nodes)
        .map(
            |(target_node, contract_node)| ReferenceSupermodelCorrectionNodeV1 {
                carrier_node_id: target_node.id,
                carrier_name: contract_node.name.clone(),
                correction_node_id: target_node.id,
                correction_name: target_node.name.clone(),
                neutral_max_abs_error: 0.0,
            },
        )
        .collect();
    Ok(ReferenceSupermodelCorrectionArtifactV1 {
        report: ReferenceSupermodelCorrectionReportV1 {
            schema_version: 2,
            source_rig_sha256: target.content_sha256.clone(),
            output_rig_sha256: target.content_sha256.clone(),
            carrier_node_count: target.nodes.len(),
            correction_node_count: 0,
            weighted_reference_remap_count: 0,
            neutral_max_abs_error: 0.0,
            source_bind_max_abs_error: target
                .nodes
                .iter()
                .zip(&contract.nodes)
                .map(|(target, carrier)| {
                    max_abs_matrix_difference(
                        target.bind_local_matrix,
                        carrier.carrier_bind_local_matrix,
                    )
                })
                .fold(0.0_f32, f32::max),
            // The v2 helper cannot prove a retail/reference bind without an
            // InspectionReport. `reference_bind_verified` is authoritative.
            reference_bind_max_abs_error: 0.0,
            reference_bind_verified: false,
            nodes,
        },
        rig: target.clone(),
    })
}

/// Builds a verified retarget carrier for a target with different proportions.
/// The immutable reference proves the exact carrier names, hierarchy and
/// animation contract, while the caller-owned neutral transforms remain the
/// pivots used by the derived model. This is the correct inherited-animation
/// strategy when source and target skeleton proportions are not identical.
pub fn build_retargeted_motion_carrier_rig_v4(
    target: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
) -> Result<ReferenceSupermodelCorrectionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    validate_reference_supermodel_motion_oracle_v2(contract, reference)?;
    let mut artifact = build_direct_motion_carrier_rig_v2(target, contract)?;
    artifact.rig.profile_id = "m2a-verified-supermodel-retarget-carrier-rig-v4".to_owned();
    for (node, exact) in artifact.rig.nodes.iter_mut().zip(&contract.nodes) {
        node.name = exact.name.clone();
        node.parent_id = exact
            .parent_part_number
            .map(|parent| target.nodes[parent as usize].id);
    }
    artifact.rig.content_sha256.clear();
    artifact.rig.content_sha256 = canonical_profile_sha256(&artifact.rig).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-RIG-HASH",
            "retargetCarrierRig",
            source.to_string(),
        )
    })?;
    artifact.report.schema_version = 4;
    artifact.report.output_rig_sha256 = artifact.rig.content_sha256.clone();
    artifact.report.nodes = target
        .nodes
        .iter()
        .zip(&contract.nodes)
        .map(|(target_node, exact)| ReferenceSupermodelCorrectionNodeV1 {
            carrier_node_id: target_node.id,
            carrier_name: exact.name.clone(),
            correction_node_id: target_node.id,
            correction_name: exact.name.clone(),
            neutral_max_abs_error: 0.0,
        })
        .collect();
    Ok(artifact)
}

/// Converts every required exact-supermodel clip into a caller-owned local
/// clip evaluated around the fitted target bind pose.
///
/// Aurora inherited position/orientation controllers are absolute local TRS
/// values. Reusing them unchanged on different pivots makes the animated pose
/// snap back toward the retail bind. This conversion preserves the exact clip
/// namespace, times, events, carrier ids and topology while applying each
/// reference local delta to the corresponding fitted target bind.
pub fn retarget_reference_supermodel_animation_set_v5(
    target: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
) -> Result<MdlAnimationSetV1, ReferenceSupermodelMotionErrorV2> {
    validate_reference_supermodel_motion_oracle_v2(contract, reference)?;
    validate_target_topology_v2(target, contract)?;
    let mut part_by_name = BTreeMap::<String, usize>::new();
    for (part, node) in contract.nodes.iter().enumerate() {
        if part_by_name
            .insert(node.name.to_ascii_lowercase(), part)
            .is_some()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-RETARGET-CARRIER-NAME-DUPLICATE",
                format!("contract.nodes[{part}].name"),
                "retargeted local clips require case-insensitively unique carrier names",
            ));
        }
    }

    let target_diagonal = distance(target.target_bounds.min, target.target_bounds.max).max(1.0e-6);
    let carrier_locals = contract
        .nodes
        .iter()
        .map(|node| node.carrier_bind_local_matrix)
        .collect::<Vec<_>>();
    let carrier_parents = contract
        .nodes
        .iter()
        .map(|node| node.parent_part_number.map(|value| value as usize))
        .collect::<Vec<_>>();
    let carrier_worlds =
        ordered_world_matrices(&carrier_locals, &carrier_parents, "contract.nodes")?;
    let reference_positions = carrier_worlds
        .iter()
        .map(|matrix| [matrix[12], matrix[13], matrix[14]])
        .collect::<Vec<_>>();
    let reference_min = std::array::from_fn(|axis| {
        reference_positions
            .iter()
            .map(|position| position[axis])
            .fold(f32::INFINITY, f32::min)
    });
    let reference_max = std::array::from_fn(|axis| {
        reference_positions
            .iter()
            .map(|position| position[axis])
            .fold(f32::NEG_INFINITY, f32::max)
    });
    let global_translation_scale =
        target_diagonal / distance(reference_min, reference_max).max(1.0e-6);

    let mut clips = Vec::with_capacity(contract.required_clips.len());
    for required in &contract.required_clips {
        let animation = reference
            .animations
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(required))
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-RETARGET-CLIP-MISSING",
                    "reference.animations",
                    format!("required reference clip {required:?} is absent"),
                )
            })?;
        let mut tracks = Vec::new();
        let mut seen = BTreeSet::<(u32, u8)>::new();
        for animation_node in flatten_nodes_in_tree_order(&animation.node_tree.roots) {
            let normalized = animation_node.name.to_ascii_lowercase();
            let part = if let Some(part) = part_by_name.get(&normalized).copied() {
                part
            } else if animation_node
                .name
                .eq_ignore_ascii_case(&reference.model.name)
            {
                0
            } else {
                continue;
            };
            let reference_bind = decompose_uniform_trs_v5(
                contract.nodes[part].carrier_bind_local_matrix,
                &format!("contract.nodes[{part}].carrierBindLocalMatrix"),
            )?;
            let target_bind = decompose_uniform_trs_v5(
                target.nodes[part].bind_local_matrix,
                &format!("target.nodes[{part}].bindLocalMatrix"),
            )?;
            let reference_length = length(reference_bind.translation);
            let target_length = length(target_bind.translation);
            let translation_scale = if reference_length > 1.0e-5 && target_length > 1.0e-5 {
                target_length / reference_length
            } else {
                global_translation_scale
            };
            for controller in &animation_node.controllers {
                let (path, expected_columns, order) = match controller.controller_type {
                    8 => (MdlAnimationTrackPathV1::Translation, 3, 0),
                    20 => (MdlAnimationTrackPathV1::Rotation, 4, 1),
                    36 => (MdlAnimationTrackPathV1::Scale, 1, 2),
                    _ => continue,
                };
                if !controller.decoded
                    || controller.column_count != expected_columns
                    || controller.times.is_empty()
                    || controller.times.len() != controller.values.len()
                    || controller.values.iter().any(|row| {
                        row.len() != expected_columns || row.iter().any(|value| !value.is_finite())
                    })
                {
                    return Err(motion_error(
                        "M2A-SUPERMODEL-RETARGET-CONTROLLER-INVALID",
                        format!(
                            "reference.animations.{}.nodes.{}.controllerType{}",
                            animation.name, animation_node.name, controller.controller_type
                        ),
                        "retargeted transform controller requires decoded finite rows with matching times",
                    ));
                }
                let key = (target.nodes[part].id, order);
                if !seen.insert(key) {
                    return Err(motion_error(
                        "M2A-SUPERMODEL-RETARGET-CONTROLLER-DUPLICATE",
                        format!(
                            "reference.animations.{}.nodes.{}",
                            animation.name, animation_node.name
                        ),
                        "one carrier may define at most one controller for each local TRS path",
                    ));
                }
                let mut values = match path {
                    MdlAnimationTrackPathV1::Translation => controller
                        .values
                        .iter()
                        .map(|row| {
                            (0..3)
                                .map(|axis| {
                                    target_bind.translation[axis]
                                        + (row[axis] - reference_bind.translation[axis])
                                            * translation_scale
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                    MdlAnimationTrackPathV1::Rotation => {
                        let alignment = quaternion_multiply_v5(
                            target_bind.rotation,
                            quaternion_conjugate_v5(reference_bind.rotation),
                        )?;
                        controller
                            .values
                            .iter()
                            .map(|row| {
                                let source = normalize_quaternion_v5(
                                    [row[0], row[1], row[2], row[3]],
                                    "reference.animations.rotation",
                                )?;
                                quaternion_multiply_v5(alignment, source)
                                    .map(|value| value.to_vec())
                            })
                            .collect::<Result<Vec<_>, _>>()?
                    }
                    MdlAnimationTrackPathV1::Scale => controller
                        .values
                        .iter()
                        .map(|row| vec![target_bind.scale * (row[0] / reference_bind.scale)])
                        .collect::<Vec<_>>(),
                    MdlAnimationTrackPathV1::Weights => unreachable!(),
                };
                if path == MdlAnimationTrackPathV1::Rotation {
                    for row in 1..values.len() {
                        let dot = values[row - 1]
                            .iter()
                            .zip(&values[row])
                            .map(|(left, right)| left * right)
                            .sum::<f32>();
                        if dot < 0.0 {
                            values[row].iter_mut().for_each(|value| *value = -*value);
                        }
                    }
                }
                tracks.push(MdlAnimationTrackV1 {
                    target_node_id: target.nodes[part].id,
                    path,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: controller.times.clone(),
                    values,
                });
            }
        }
        tracks.sort_by_key(|track| {
            let order = match track.path {
                MdlAnimationTrackPathV1::Translation => 0,
                MdlAnimationTrackPathV1::Rotation => 1,
                MdlAnimationTrackPathV1::Scale => 2,
                MdlAnimationTrackPathV1::Weights => 3,
            };
            (track.target_node_id, order)
        });
        if tracks.is_empty() {
            return Err(motion_error(
                "M2A-SUPERMODEL-RETARGET-CLIP-CONTROLLERS-EMPTY",
                format!("reference.animations.{}", animation.name),
                "a required retargeted clip must contain at least one carrier transform controller",
            ));
        }
        clips.push(MdlAnimationClipV1 {
            name: animation.name.clone(),
            animation_root: animation.animation_root.clone(),
            length_seconds: animation.length,
            transition_seconds: animation.transition,
            events: animation
                .events
                .iter()
                .map(|event| MdlAnimationEventV1 {
                    time_seconds: event.time,
                    name: event.name.clone(),
                })
                .collect(),
            tracks,
        });
    }
    Ok(MdlAnimationSetV1 {
        schema_version: 1,
        clips,
    })
}

#[derive(Clone, Copy)]
struct UniformTrsV5 {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: f32,
}

fn decompose_uniform_trs_v5(
    matrix: [f32; 16],
    path: &str,
) -> Result<UniformTrsV5, ReferenceSupermodelMotionErrorV2> {
    let scale = length([matrix[0], matrix[1], matrix[2]]);
    if !scale.is_finite() || scale <= 1.0e-8 {
        return Err(motion_error(
            "M2A-SUPERMODEL-RETARGET-BIND-INVALID",
            path,
            "retarget bind requires a finite positive uniform scale",
        ));
    }
    let rotation_matrix = [
        matrix[0] / scale,
        matrix[1] / scale,
        matrix[2] / scale,
        matrix[4] / scale,
        matrix[5] / scale,
        matrix[6] / scale,
        matrix[8] / scale,
        matrix[9] / scale,
        matrix[10] / scale,
    ];
    let rotation = quaternion_from_rotation_matrix_v5(rotation_matrix, path)?;
    Ok(UniformTrsV5 {
        translation: [matrix[12], matrix[13], matrix[14]],
        rotation,
        scale,
    })
}

fn quaternion_from_rotation_matrix_v5(
    matrix: [f32; 9],
    path: &str,
) -> Result<[f32; 4], ReferenceSupermodelMotionErrorV2> {
    let m00 = matrix[0];
    let m01 = matrix[3];
    let m02 = matrix[6];
    let m10 = matrix[1];
    let m11 = matrix[4];
    let m12 = matrix[7];
    let m20 = matrix[2];
    let m21 = matrix[5];
    let m22 = matrix[8];
    let trace = m00 + m11 + m22;
    let quaternion = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s]
    };
    normalize_quaternion_v5(quaternion, path)
}

fn normalize_quaternion_v5(
    value: [f32; 4],
    path: &str,
) -> Result<[f32; 4], ReferenceSupermodelMotionErrorV2> {
    let norm = value.iter().map(|value| value * value).sum::<f32>().sqrt();
    if !norm.is_finite() || norm <= 1.0e-8 {
        return Err(motion_error(
            "M2A-SUPERMODEL-RETARGET-QUATERNION-INVALID",
            path,
            "retargeted rotation must be finite and nonzero",
        ));
    }
    Ok(value.map(|value| value / norm))
}

fn quaternion_conjugate_v5(value: [f32; 4]) -> [f32; 4] {
    [-value[0], -value[1], -value[2], value[3]]
}

fn quaternion_multiply_v5(
    left: [f32; 4],
    right: [f32; 4],
) -> Result<[f32; 4], ReferenceSupermodelMotionErrorV2> {
    normalize_quaternion_v5(
        [
            left[3] * right[0] + left[0] * right[3] + left[1] * right[2] - left[2] * right[1],
            left[3] * right[1] - left[0] * right[2] + left[1] * right[3] + left[2] * right[0],
            left[3] * right[2] + left[0] * right[1] - left[1] * right[0] + left[2] * right[3],
            left[3] * right[3] - left[0] * right[0] - left[1] * right[1] - left[2] * right[2],
        ],
        "retargetedAnimation.rotation",
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReferenceSupermodelCarrierModeV4 {
    CorrectedExactBind,
    DirectExactBind,
    RetargetedTargetBind,
}

fn bind_pose_compatible_for_mode_v4(
    correction: &ReferenceSupermodelCorrectionReportV1,
    contract: &ReferenceSupermodelMotionContractV2,
    mode: ReferenceSupermodelCarrierModeV4,
) -> bool {
    match mode {
        ReferenceSupermodelCarrierModeV4::CorrectedExactBind
        | ReferenceSupermodelCarrierModeV4::DirectExactBind => {
            correction.reference_bind_verified
                && correction.reference_bind_max_abs_error <= contract.tolerances.bind_max_abs_error
        }
        ReferenceSupermodelCarrierModeV4::RetargetedTargetBind => {
            correction.schema_version == 4
                && !correction.reference_bind_verified
                && correction.carrier_node_count == contract.nodes.len()
                && correction.correction_node_count == 0
                && correction.nodes.len() == contract.nodes.len()
        }
    }
}

fn audit_output_carrier_readback_v3(
    output: &InspectionReport,
    contract: &ReferenceSupermodelMotionContractV2,
) -> ReferenceSupermodelCarrierReadbackCoverageV3 {
    let flattened = flatten_nodes_in_tree_order(&output.node_tree.roots);
    let non_render_nodes = flattened
        .iter()
        .copied()
        .filter(|node| node.mesh.is_none() && node.skin.is_none())
        .collect::<Vec<_>>();
    let mut missing = Vec::new();
    let mut incompatible = Vec::new();
    let mut matched = Vec::new();
    for (part, expected) in contract.nodes.iter().enumerate() {
        let candidates = non_render_nodes
            .iter()
            .copied()
            .filter(|actual| {
                if part == 0 {
                    actual.name.eq_ignore_ascii_case(&output.model.name)
                        || actual.name.eq_ignore_ascii_case(&expected.name)
                } else {
                    actual.name.eq_ignore_ascii_case(&expected.name)
                }
            })
            .collect::<Vec<_>>();
        let [actual] = candidates.as_slice() else {
            missing.push(expected.name.clone());
            continue;
        };
        matched.push(*actual);
        let parent_compatible = match expected.parent_part_number {
            None => actual.parent_offset.is_none(),
            Some(parent_part) => actual.parent_offset.is_some_and(|parent_offset| {
                flattened.iter().any(|parent| {
                    parent.offset == parent_offset
                        && if parent_part == 0 {
                            parent.name.eq_ignore_ascii_case(&output.model.name)
                                || parent.name.eq_ignore_ascii_case(&contract.nodes[0].name)
                        } else {
                            parent
                                .name
                                .eq_ignore_ascii_case(&contract.nodes[parent_part as usize].name)
                        }
                })
            }),
        };
        if !parent_compatible {
            incompatible.push(expected.name.clone());
        }
    }
    let carrier_names = contract
        .nodes
        .iter()
        .skip(1)
        .map(|node| node.name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let extra = non_render_nodes
        .iter()
        .filter(|node| carrier_names.contains(&node.name.to_ascii_lowercase()))
        .filter(|node| !matched.iter().any(|matched| matched.offset == node.offset))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let full = missing.is_empty()
        && extra.is_empty()
        && incompatible.is_empty()
        && matched.len() == contract.nodes.len();
    ReferenceSupermodelCarrierReadbackCoverageV3 {
        schema_version: 3,
        required_carrier_count: contract.nodes.len(),
        output_carrier_count: matched.len(),
        missing_carrier_names: missing,
        extra_carrier_names: extra,
        incompatible_carrier_names: incompatible,
        full_carrier_coverage: full,
        required_joint_coverage: full,
    }
}

fn audit_target_rig_skin_influences_v3(
    rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
) -> (bool, usize, usize, Vec<String>, Vec<String>) {
    let allowed = rig
        .segments
        .iter()
        .flat_map(|segment| segment.allowed_bone_node_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    let mut active = BTreeSet::new();
    let mut visible = BTreeSet::new();
    for influence in rig
        .segments
        .iter()
        .flat_map(|segment| segment.reference_weights.iter())
        .flatten()
    {
        if influence.value.is_finite() && influence.value > 0.0 {
            active.insert(influence.bone_node_id);
        }
        if influence.value.is_finite() && influence.value >= 0.1 {
            visible.insert(influence.bone_node_id);
        }
    }
    let mut unweighted_required = Vec::new();
    let mut passive_unweighted = Vec::new();
    for (part, node) in contract.nodes.iter().enumerate() {
        let Some(rig_node_id) = rig.nodes.get(part).map(|node| node.id) else {
            if node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant {
                unweighted_required.push(node.name.clone());
            } else {
                passive_unweighted.push(node.name.clone());
            }
            continue;
        };
        if node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant {
            if !active.contains(&rig_node_id) || !visible.contains(&rig_node_id) {
                unweighted_required.push(node.name.clone());
            }
        } else if !active.contains(&rig_node_id) {
            passive_unweighted.push(node.name.clone());
        }
    }
    (
        unweighted_required.is_empty(),
        allowed.len(),
        active.len(),
        unweighted_required,
        passive_unweighted,
    )
}

/// Complete Core route for a static GLB that obtains animations from an exact,
/// read-only inspected supermodel. It preserves NWN:EE material semantics and
/// refuses to report motion compatibility without sampling every required
/// clip.
#[allow(clippy::too_many_arguments)]
pub fn build_reference_supermodel_creature_package_v1(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMaterialOptionsV1,
) -> Result<ReferenceSupermodelMotionArtifactV2, ReferenceSupermodelMotionErrorV2> {
    build_reference_supermodel_creature_package_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_reference_supermodel_creature_package_direct_v2(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMaterialOptionsV1,
) -> Result<ReferenceSupermodelMotionArtifactV2, ReferenceSupermodelMotionErrorV2> {
    build_reference_supermodel_creature_package_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_reference_supermodel_motion_v1(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    carrier_mode: ReferenceSupermodelCarrierModeV4,
) -> Result<
    (
        ReferenceSupermodelCorrectionArtifactV1,
        crate::glb::GlbIngestResult,
        ProfileAConversionOutcomeV1,
    ),
    ReferenceSupermodelMotionErrorV2,
> {
    validate_reference_supermodel_motion_oracle_v2(contract, reference_supermodel)?;
    let mut correction = match carrier_mode {
        ReferenceSupermodelCarrierModeV4::CorrectedExactBind => {
            build_motion_corrected_rig_v1(target_rig, contract)?
        }
        ReferenceSupermodelCarrierModeV4::DirectExactBind => {
            build_exact_motion_carrier_rig_v3(target_rig, contract, reference_supermodel)?
        }
        ReferenceSupermodelCarrierModeV4::RetargetedTargetBind => {
            build_retargeted_motion_carrier_rig_v4(target_rig, contract, reference_supermodel)?
        }
    };
    // The correction-node route also uses the already validated exact
    // contract carrier nodes. Mark that fact only here, where the immutable
    // reference inspection is available; the standalone legacy helper cannot
    // make this claim on its own.
    if carrier_mode == ReferenceSupermodelCarrierModeV4::CorrectedExactBind {
        correction.report.reference_bind_max_abs_error = 0.0;
        correction.report.reference_bind_verified = true;
    }
    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|source| {
        motion_error(
            "M2A-SUPERMODEL-SOURCE-GLB-INVALID",
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            format!("{}: {}", source.code, source.message),
        )
    })?;
    let options = direct_creature_profile_a_options_for_source_forward_v3(source_forward);
    let mut conversion = convert_profile_a_exact_v1(&ingest, &correction.rig, &options)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    if !conversion.report.conversion_eligible {
        return Err(motion_error(
            "M2A-SUPERMODEL-SOURCE-INELIGIBLE",
            "conversion.report",
            "Profile A rejected the corrected reference-supermodel rig",
        ));
    }
    let creature = conversion.creature.as_mut().ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-SOURCE-INELIGIBLE",
            "conversion.creature",
            "eligible conversion did not produce creature IR",
        )
    })?;
    let roots = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    if roots.as_slice() != [0] {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-ROOT-MISMATCH",
            "conversion.creature.nodes",
            "corrected carrier hierarchy must have exactly one root at part 0",
        ));
    }
    creature.nodes[0].name = writer_options.model_resource_resref.clone();
    segment_model_for_binary_mdl_v1(creature)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    Ok((correction, ingest, conversion))
}

#[allow(clippy::too_many_arguments)]
fn build_reference_supermodel_creature_package_internal_v2(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMaterialOptionsV1,
    direct_named_carriers: bool,
) -> Result<ReferenceSupermodelMotionArtifactV2, ReferenceSupermodelMotionErrorV2> {
    if material_options.schema_version != 1 || !is_resref(&material_options.texture_resref) {
        return Err(motion_error(
            "M2A-SUPERMODEL-MATERIAL-OPTIONS-INVALID",
            "materialOptions",
            "material options require schema 1 and a valid base texture resref",
        ));
    }
    let (correction, ingest, mut conversion) = prepare_reference_supermodel_motion_v1(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        if direct_named_carriers {
            ReferenceSupermodelCarrierModeV4::DirectExactBind
        } else {
            ReferenceSupermodelCarrierModeV4::CorrectedExactBind
        },
    )?;
    let creature = conversion.creature.as_mut().ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-PREPARED-CREATURE-MISSING",
            "conversion.creature",
            "motion preparation returned no creature",
        )
    })?;

    let material_compilation = compile_gltf_materials_v1(
        &ingest.ir.source.sha256,
        &ingest.ir.materials,
        AuroraMaterialTargetProfileV1::NwnEeMtr,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    if material_compilation.status == AuroraMaterialCompileStatusV1::Blocked {
        return Err(motion_error(
            "M2A-SUPERMODEL-MATERIAL-BLOCKED",
            "materialCompilation.status",
            "NWN:EE material compilation is blocked",
        ));
    }
    let material_package = package_aurora_materials_v1(
        source_glb,
        &ingest,
        creature,
        &material_compilation,
        &material_options.texture_resref,
        &[],
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let material_semantics = build_creature_material_semantic_report_v1(
        &ingest,
        creature,
        &material_compilation,
        &material_package,
    )?;
    if material_semantics.status != CreatureMaterialSemanticStatusV1::Ready {
        return Err(motion_error(
            "M2A-SUPERMODEL-MATERIAL-SEMANTICS-BLOCKED",
            "materialSemantics",
            "one or more required creature material resources are absent",
        ));
    }
    ensure_material_tangents_v1(creature, &material_package.slots)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    for resource in &material_package.resources {
        validate_material_resource_semantics_v1(resource)
            .map_err(|source| motion_error(source.code, source.path, source.message))?;
    }

    let mut effective_writer = writer_options.clone();
    effective_writer.diffuse_texture_resref_by_material_slot = material_package
        .slots
        .iter()
        .map(|slot| slot.diffuse_binding.clone())
        .collect();
    let base_model = write_binary_mdl_with_supermodel_exact_face_planes_v1(
        creature,
        &contract.supermodel_resref,
        &effective_writer,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let extension = extend_binary_mdl_with_materials_v1(
        creature,
        base_model,
        &MdlMaterialExtensionOptionsV1 {
            schema_version: 1,
            materials: material_package
                .slots
                .iter()
                .map(|slot| slot.state.clone())
                .collect(),
            segment_streams: creature
                .segments
                .iter()
                .map(|segment| MdlSegmentMaterialStreamsV1 {
                    segment_id: segment.segment_id,
                    uv1: Vec::new(),
                    uv2: Vec::new(),
                    uv3: Vec::new(),
                })
                .collect(),
        },
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    if !extension.material_report.semantic_diff.is_empty()
        || !extension.binary.report.semantic_diff.is_empty()
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MATERIAL-MDL-SEMANTIC-DIFF",
            "model.readback",
            "material-complete MDL differs after semantic readback",
        ));
    }
    let motion_quality = inspect_inherited_motion_quality_for_contract_v2(
        &extension.binary.inspection,
        reference_supermodel,
        contract,
        &correction.report,
    )?;
    if motion_quality.status != "PASS" {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-QUALITY-BLOCKED",
            "motionQuality",
            format!(
                "inherited deformation failed the motion quality gate: hardEdges={}/{}, softEdges={}/{}, collapsedTriangles={}/{}, expandedTriangles={}/{}, opposedNormals={}/{}, seamViolations={}/{}, missingAnchorClusters={}, missingPawClusters={}, pawContactViolations={}, pawSideViolations={}, clipStartAnchorJumps={}",
                motion_quality.edge_outside_hard_limit_count,
                motion_quality.edge_sample_count,
                motion_quality.edge_outside_soft_limit_count,
                motion_quality.edge_sample_count,
                motion_quality.triangle_area_collapse_count,
                motion_quality.triangle_sample_count,
                motion_quality.triangle_area_expansion_count,
                motion_quality.triangle_sample_count,
                motion_quality.world_normal_opposition_count,
                motion_quality.triangle_sample_count,
                motion_quality.seam_pair_violation_count,
                motion_quality.seam_pair_sample_count,
                motion_quality.anchor_cluster_missing_count,
                motion_quality.paw_cluster_missing_count,
                motion_quality.paw_contact_violation_count,
                motion_quality.paw_side_violation_count,
                motion_quality.clip_start_anchor_jump_violation_count,
            ),
        ));
    }
    let supermodel_clip_lookup = extension
        .binary
        .inspection
        .model
        .supermodel_name
        .eq_ignore_ascii_case(&contract.supermodel_resref);
    let bind_pose_compatible = bind_pose_compatible_for_mode_v4(
        &correction.report,
        contract,
        if direct_named_carriers {
            ReferenceSupermodelCarrierModeV4::DirectExactBind
        } else {
            ReferenceSupermodelCarrierModeV4::CorrectedExactBind
        },
    );
    let skin_bind_compatible = extension.binary.report.semantic_diff.is_empty();
    let motion_compatible = supermodel_clip_lookup
        && bind_pose_compatible
        && skin_bind_compatible
        && motion_quality.status == "PASS";
    if !motion_compatible {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-COMPATIBILITY-NOT-PROVEN",
            "report.motionCompatible",
            "supermodel lookup, exact reference bind, skin readback and sampled motion must all pass",
        ));
    }
    let report = ReferenceSupermodelMotionBuildReportV2 {
        schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
        compatibility_level: ReferenceSupermodelCompatibilityLevelV2::MotionCompatible,
        supermodel_clip_lookup,
        topology_compatible: true,
        bind_pose_compatible,
        skin_bind_compatible,
        motion_correction_applied: !direct_named_carriers,
        clip_inventory_verified: true,
        motion_compatible,
        source_sha256: ingest.ir.source.sha256.clone(),
        source_rig_sha256: target_rig.content_sha256.clone(),
        corrected_rig_sha256: correction.rig.content_sha256.clone(),
        motion_contract_sha256: contract.content_sha256.clone(),
        supermodel_resref: contract.supermodel_resref.clone(),
        model_resource_resref: effective_writer.model_resource_resref.clone(),
        model_sha256: extension.binary.report.payload_sha256.clone(),
        material_status: material_compilation.status,
        material_resource_count: material_package.resources.len(),
        required_clip_count: contract.required_clips.len(),
    };
    Ok(ReferenceSupermodelMotionArtifactV2 {
        correction,
        conversion,
        model: extension.binary,
        material_compilation,
        material_package,
        material_extension: extension.material_report,
        material_semantics,
        motion_quality,
        report,
    })
}

/// Product-facing name for the full inherited-motion route. Unlike the legacy
/// topology-only emitter, this entry point requires a motion contract, applies
/// carrier/correction binding, samples the inherited clips and preserves the
/// complete material package before it can report compatibility.
#[allow(clippy::too_many_arguments)]
pub fn bind_static_mesh_for_inherited_supermodel_motion_v1(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMaterialOptionsV1,
) -> Result<ReferenceSupermodelMotionArtifactV2, ReferenceSupermodelMotionErrorV2> {
    build_reference_supermodel_creature_package_v1(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
    )
}

/// Full material/motion route for models whose owned source-fitted hierarchy
/// already supplies the exact named supermodel carriers. No correction nodes
/// are inserted, so the base model retains the ordered 30-node contract.
#[allow(clippy::too_many_arguments)]
pub fn bind_static_mesh_for_inherited_supermodel_motion_direct_v2(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMaterialOptionsV1,
) -> Result<ReferenceSupermodelMotionArtifactV2, ReferenceSupermodelMotionErrorV2> {
    build_reference_supermodel_creature_package_direct_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
    )
}

fn bound_source_material_ids_v1(
    ingest: &crate::glb::GlbIngestResult,
    model: &crate::model_ir::AuroraModelIrV1,
) -> Result<BTreeSet<u32>, ReferenceSupermodelMotionErrorV2> {
    let mut source_material_ids = BTreeSet::new();
    for binding in &model.material_source_bindings {
        let source_material_id = binding.source_material_id.ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MATERIAL-SOURCE-BINDING-MISSING",
                "conversion.creature.materialSourceBindings.sourceMaterialId",
                format!("material slot {} has no source material id", binding.slot),
            )
        })?;
        if !ingest
            .ir
            .materials
            .iter()
            .any(|material| material.id == source_material_id)
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MATERIAL-SOURCE-MISSING",
                "source.materials",
                format!("source material {source_material_id} is absent"),
            ));
        }
        source_material_ids.insert(source_material_id);
    }
    if source_material_ids.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MATERIAL-SOURCE-BINDINGS-EMPTY",
            "conversion.creature.materialSourceBindings",
            "the inherited-motion model has no source material binding",
        ));
    }
    Ok(source_material_ids)
}

/// Runtime-conservative inherited-motion route. It applies the direct named
/// carrier contract and the full motion oracle, but deliberately emits only
/// the established classic diffuse MDL state. It never invokes the material
/// extension writer and never emits tangent, TXI, normal/specular or MTR data.
#[allow(clippy::too_many_arguments)]
pub fn bind_static_mesh_for_inherited_supermodel_motion_direct_classic_v1(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelClassicMaterialOptionsV1,
) -> Result<ReferenceSupermodelClassicMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    if material_options.schema_version != 1 || !is_resref(&material_options.texture_resref) {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-MATERIAL-OPTIONS-INVALID",
            "materialOptions",
            "classic material options require schema 1 and a valid diffuse texture resref",
        ));
    }
    let (correction, ingest, conversion) = prepare_reference_supermodel_motion_v1(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        ReferenceSupermodelCarrierModeV4::DirectExactBind,
    )?;
    let creature = conversion.creature.as_ref().ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-PREPARED-CREATURE-MISSING",
            "conversion.creature",
            "motion preparation returned no creature",
        )
    })?;
    let source_material_ids = bound_source_material_ids_v1(&ingest, creature)?;
    if ingest
        .ir
        .materials
        .iter()
        .any(|material| source_material_ids.contains(&material.id) && material.double_sided)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-TWO-SIDED-BLOCKED",
            "source.materials.doubleSided",
            "classic diffuse packaging cannot discard a bound source material with doubleSided=true",
        ));
    }
    if creature
        .segments
        .iter()
        .any(|segment| segment.tangents.is_some())
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-TANGENTS-FORBIDDEN",
            "conversion.creature.segments.tangents",
            "classic-safe inherited motion forbids tangent streams",
        ));
    }
    let material_slots = creature
        .material_source_bindings
        .iter()
        .map(|binding| binding.slot)
        .collect::<BTreeSet<_>>();
    if material_slots.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-MATERIAL-SLOTS-EMPTY",
            "conversion.creature.materialSourceBindings",
            "classic-safe inherited motion requires at least one diffuse material slot",
        ));
    }
    let mut effective_writer = writer_options.clone();
    effective_writer.diffuse_texture_resref_by_material_slot = material_slots
        .into_iter()
        .map(|material_slot| MdlMaterialTextureBindingV1 {
            material_slot,
            resref: material_options.texture_resref.clone(),
        })
        .collect();
    let model = write_binary_mdl_with_supermodel_exact_face_planes_v1(
        creature,
        &contract.supermodel_resref,
        &effective_writer,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let tangent_stream_count = validate_classic_motion_model_v1(
        &model.inspection.node_tree.roots,
        &material_options.texture_resref,
    )?;
    let motion_quality = inspect_inherited_motion_quality_for_contract_v2(
        &model.inspection,
        reference_supermodel,
        contract,
        &correction.report,
    )?;
    if motion_quality.status != "PASS" {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-MOTION-QUALITY-BLOCKED",
            "motionQuality",
            motion_quality_blocked_summary_v4(
                "classic-safe inherited deformation failed the motion quality gate",
                &motion_quality,
            ),
        ));
    }
    let bind_pose_compatible = bind_pose_compatible_for_mode_v4(
        &correction.report,
        contract,
        ReferenceSupermodelCarrierModeV4::DirectExactBind,
    );
    let skin_bind_compatible = model.report.semantic_diff.is_empty();
    let carrier_coverage = audit_output_carrier_readback_v3(&model.inspection, contract);
    let (
        skin_influence_coverage,
        allowed_bone_count,
        active_weighted_bone_count,
        unweighted_required_joint_names,
        passive_unweighted_joint_names,
    ) = audit_target_rig_skin_influences_v3(target_rig, contract);
    let motion_compatible = bind_pose_compatible
        && skin_bind_compatible
        && carrier_coverage.full_carrier_coverage
        && skin_influence_coverage
        && model
            .inspection
            .model
            .supermodel_name
            .eq_ignore_ascii_case(&contract.supermodel_resref)
        && motion_quality.status == "PASS";
    if !motion_compatible {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-COMPATIBILITY-NOT-PROVEN",
            "report.motionCompatible",
            "supermodel lookup, exact reference bind, skin readback and sampled motion must all pass",
        ));
    }
    let report = ReferenceSupermodelClassicMotionBuildReportV1 {
        schema_version: 1,
        motion_compatible,
        bind_pose_compatible,
        skin_bind_compatible,
        source_sha256: ingest.ir.source.sha256,
        source_rig_sha256: target_rig.content_sha256.clone(),
        output_rig_sha256: correction.rig.content_sha256.clone(),
        motion_contract_sha256: contract.content_sha256.clone(),
        supermodel_resref: contract.supermodel_resref.clone(),
        model_resource_resref: effective_writer.model_resource_resref,
        model_sha256: model.report.payload_sha256.clone(),
        material_profile: "CLASSIC_DIFFUSE_TGA_SAFE_V1".to_owned(),
        material_extension_applied: false,
        tangent_stream_count,
        material_resource_count: 1,
        runtime_readiness: "RUNTIME_UNPROVEN".to_owned(),
        required_clip_count: contract.required_clips.len(),
        carrier_coverage,
        skin_influence_coverage,
        allowed_bone_count,
        active_weighted_bone_count,
        unweighted_required_joint_names,
        passive_unweighted_joint_names,
        motion_weight_refinement_applied: false,
        motion_weight_refinement_attempted_round_count: 0,
        motion_weight_refinement_examined_edge_count: 0,
        motion_weight_refinement_measured_triangle_count: 0,
        motion_weight_refinement_smoothing_candidate_group_count: 0,
        motion_weight_refinement_smoothing_shared_support_group_count: 0,
        motion_weight_refinement_smoothing_changed_group_count: 0,
        motion_weight_refinement_micro_triangle_candidate_count: 0,
        motion_weight_refinement_micro_triangle_too_large_count: 0,
        motion_weight_refinement_micro_triangle_protected_support_count: 0,
        motion_weight_refinement_micro_triangle_nonlocal_support_count: 0,
        motion_weight_refinement_micro_triangle_no_common_support_count: 0,
        motion_weight_refinement_micro_triangle_changed_count: 0,
        motion_weight_refinement_catastrophic_edge_candidate_count: 0,
        motion_weight_refinement_catastrophic_edge_count: 0,
        motion_weight_refinement_catastrophic_edge_topology_skip_count: 0,
        motion_weight_refinement_catastrophic_edge_conflict_skip_count: 0,
        motion_weight_refinement_catastrophic_anchor_restore_count: 0,
        motion_weight_refinement_catastrophic_peak_edge: None,
        motion_weight_refinement_catastrophic_peak_edge_changed: false,
        motion_weight_refinement_backtracked_group_count: 0,
        motion_weight_refinement_iteration_count: 0,
        motion_weight_refinement_edge_count: 0,
        motion_weight_refinement_rigid_component_count: 0,
        motion_weight_refinement_projected_triangle_count: 0,
        motion_weight_refinement_rejected_round_count: 0,
        motion_weight_refinement_last_rejection: None,
    };
    Ok(ReferenceSupermodelClassicMotionArtifactV1 {
        correction,
        conversion,
        model,
        motion_quality,
        report,
    })
}

/// Runtime-conservative NWN:EE route for thin or fragmented source geometry
/// that explicitly requires two-sided rendering. The route binds one minimal
/// MTR and deliberately suppresses tangent, normal, specular and TXI output.
#[allow(clippy::too_many_arguments)]
pub fn bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_v1(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMinimalMtrMaterialOptionsV1,
) -> Result<ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        ReferenceSupermodelCarrierModeV4::DirectExactBind,
        true,
    )
}

/// Runtime-conservative two-sided MTR route that keeps the inspected
/// supermodel bind on animation-carrier nodes and remaps owned skin weights to
/// correction children. This preserves the target neutral pose while inherited
/// controllers continue to resolve against the exact carrier hierarchy.
#[allow(clippy::too_many_arguments)]
pub fn bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_corrected_v2(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMinimalMtrMaterialOptionsV1,
) -> Result<ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        ReferenceSupermodelCarrierModeV4::CorrectedExactBind,
        true,
    )
}

/// Two-sided inherited-motion route for proportionally different targets.
/// It verifies the immutable supermodel contract but keeps the target's owned
/// neutral pivots so animation rotations are retargeted around its anatomy.
#[allow(clippy::too_many_arguments)]
pub fn bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_retargeted_v4(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMinimalMtrMaterialOptionsV1,
) -> Result<ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        ReferenceSupermodelCarrierModeV4::RetargetedTargetBind,
        true,
    )
}

/// Builds a read-only diagnostic artifact with the selected supermodel's exact
/// carrier hierarchy and exact local bind matrices. The owned mesh is already
/// registered and weighted in that immutable coordinate space by the generic
/// rig preparation route; no correction children and no local retargeted
/// animation clips are emitted here.
///
/// Callers must surface `motion_quality.status` and must not package a blocked
/// result as runtime-ready content.
#[allow(clippy::too_many_arguments)]
pub fn build_inherited_supermodel_motion_diagnostic_preview_v1(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMinimalMtrMaterialOptionsV1,
) -> Result<ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        ReferenceSupermodelCarrierModeV4::DirectExactBind,
        false,
    )
}

/// Builds a diagnostic preview for a proportionally different owned target.
///
/// The exact selected supermodel still supplies the complete named carrier
/// hierarchy and inherited controller corpus.  Unlike the exact-bind
/// diagnostic above, this route keeps the source-fitted target pivots so a
/// long-legged or otherwise differently proportioned Creature is evaluated
/// around its own anatomy.  Motion quality remains fail-closed for product
/// admission; the only difference is the same verified retarget carrier mode
/// used by the production retargeted writer.
#[allow(clippy::too_many_arguments)]
pub fn build_inherited_supermodel_motion_diagnostic_preview_retargeted_v4(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMinimalMtrMaterialOptionsV1,
) -> Result<ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_internal_v2(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        material_options,
        ReferenceSupermodelCarrierModeV4::RetargetedTargetBind,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_internal_v2(
    source_glb: &[u8],
    target_rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
    reference_supermodel: &InspectionReport,
    writer_options: &MdlWriterOptionsV1,
    source_forward: CreatureSourceForwardV1,
    material_options: &ReferenceSupermodelMinimalMtrMaterialOptionsV1,
    carrier_mode: ReferenceSupermodelCarrierModeV4,
    enforce_motion_quality: bool,
) -> Result<ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    if material_options.schema_version != 1
        || !is_resref(&material_options.texture_resref)
        || !is_resref(&material_options.material_resref)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-OPTIONS-INVALID",
            "materialOptions",
            "minimal MTR options require schema 1 and valid diffuse/material resrefs",
        ));
    }
    validate_creature_material_profile_v2(&material_options.material_profile)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let profile = &material_options.material_profile;
    if profile.target != CreatureMaterialTargetV2::NwnEeMtr
        || profile.normal_maps
        || profile.tangent_space_ready
        || profile.metallic_roughness_to_specular_gloss
        || profile.emissive_to_self_illumination
        || profile.alpha_mode != CreatureAlphaModeV2::Opaque
        || !profile.double_sided
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-PROFILE-INVALID",
            "materialOptions.materialProfile",
            "minimal two-sided MTR requires NWN_EE_MTR, OPAQUE and doubleSided=true with every extended map/channel disabled",
        ));
    }

    let (correction, ingest, mut conversion) = prepare_reference_supermodel_motion_v1(
        source_glb,
        target_rig,
        contract,
        reference_supermodel,
        writer_options,
        source_forward,
        carrier_mode,
    )?;
    let creature = conversion.creature.as_mut().ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-PREPARED-CREATURE-MISSING",
            "conversion.creature",
            "motion preparation returned no creature",
        )
    })?;
    if creature
        .segments
        .iter()
        .any(|segment| segment.tangents.is_some())
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-TANGENTS-FORBIDDEN",
            "conversion.creature.segments.tangents",
            "minimal two-sided MTR inherited motion forbids tangent streams",
        ));
    }
    // Dense two-sided fur cards self-shadow as dark dashed bands in Aurora.
    // The minimal card-material route therefore disables mesh shadow casting;
    // scene lighting and receiving shadows remain unchanged.
    for segment in &mut creature.segments {
        segment.cast_shadow = false;
    }

    let material_slots = creature
        .material_source_bindings
        .iter()
        .map(|binding| binding.slot)
        .collect::<BTreeSet<_>>();
    let source_material_ids = bound_source_material_ids_v1(&ingest, creature)?;
    if material_slots.len() != 1 || source_material_ids.len() != 1 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-MULTI-MATERIAL-BLOCKED",
            "conversion.creature.materialSourceBindings",
            "minimal two-sided MTR V1 requires exactly one bound output slot and one source material",
        ));
    }
    let material_slot = *material_slots.iter().next().expect("one material slot");
    let source_material_id = *source_material_ids
        .iter()
        .next()
        .expect("one source material");
    let source_material = ingest
        .ir
        .materials
        .iter()
        .find(|material| material.id == source_material_id)
        .expect("bound source material was validated");
    if !source_material.double_sided {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-SOURCE-NOT-TWO-SIDED",
            "source.materials.doubleSided",
            "minimal two-sided MTR may only be used when the bound source material declares doubleSided=true",
        ));
    }

    let material_compilation = compile_gltf_materials_v1(
        &ingest.ir.source.sha256,
        &ingest.ir.materials,
        AuroraMaterialTargetProfileV1::NwnEeMtr,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    if material_compilation.status == AuroraMaterialCompileStatusV1::Blocked {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-COMPILATION-BLOCKED",
            "materialCompilation.status",
            "NWN:EE material compilation is blocked",
        ));
    }
    let compiled = material_compilation
        .materials
        .iter()
        .find(|entry| entry.material.source_material_id == source_material_id)
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MINIMAL-MTR-COMPILATION-MISSING",
                "materialCompilation.materials",
                format!("source material {source_material_id} has no compiler result"),
            )
        })?;
    if !compiled.material.two_sided
        || !compiled
            .material
            .mtr
            .as_ref()
            .is_some_and(|mtr| mtr.two_sided)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-TWO-SIDED-NOT-PRESERVED",
            "materialCompilation.materials.mtr.twoSided",
            "the shared compiler did not preserve source doubleSided as NWN:EE MTR twosided",
        ));
    }

    let mtr_document = MtrDocumentV1 {
        schema_version: MTR_SCHEMA_VERSION_V1,
        textures: vec![MtrTextureBindingV1 {
            slot: 0,
            resref: material_options.texture_resref.clone(),
        }],
        render_hint: MtrRenderHintV1::Normal,
        transparency: false,
        two_sided: true,
        blending: None,
    };
    let mtr_payload = write_mtr_v1(&mtr_document)
        .map_err(|source| motion_error(source.code, "material.mtr", source.message))?;
    if parse_mtr_v1(&mtr_payload)
        .map_err(|source| motion_error(source.code, "material.mtr", source.message))?
        != mtr_document
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-SEMANTIC-DIFF",
            "material.mtr",
            "minimal two-sided MTR differs after canonical readback",
        ));
    }
    let mtr_resource = HakResourceInputV1 {
        resref: material_options.material_resref.clone(),
        resource_type: MTR_RESOURCE_TYPE_V1,
        payload: mtr_payload,
    };

    let mut effective_writer = writer_options.clone();
    effective_writer.diffuse_texture_resref_by_material_slot = vec![MdlMaterialTextureBindingV1 {
        material_slot,
        resref: material_options.texture_resref.clone(),
    }];
    let material_state = MdlMaterialStateV1 {
        material_slot,
        diffuse: compiled.material.diffuse_color[..3]
            .try_into()
            .expect("three diffuse components"),
        ambient: compiled.material.ambient_color,
        specular: [0.0; 3],
        shininess: 1.0,
        alpha: 1.0,
        self_illum_color: [0.0; 3],
        transparency_hint: false,
        render_hint: MdlRenderHintV1::Normal,
        normal_texture_resref: None,
        specular_texture_resref: None,
        material_resref: Some(material_options.material_resref.clone()),
    };
    let local_retargeted_animations = if carrier_mode
        == ReferenceSupermodelCarrierModeV4::RetargetedTargetBind
    {
        retarget_reference_supermodel_animation_set_v5(target_rig, contract, reference_supermodel)?
    } else {
        MdlAnimationSetV1::empty()
    };
    let mut extension = build_minimal_twosided_material_model_v1(
        creature,
        contract,
        &effective_writer,
        &material_state,
        &local_retargeted_animations,
    )?;
    let local_animation_source = !local_retargeted_animations.clips.is_empty();
    let animation_source = if local_animation_source {
        &extension.binary.inspection
    } else {
        reference_supermodel
    };
    let (mut motion_quality, mut weight_violations) =
        inspect_inherited_motion_quality_with_refinement_v1(
            &extension.binary.inspection,
            animation_source,
            contract,
            &correction.report,
        )?;
    let mut explored_refinement = MotionWeightRefinementReportV1::default();
    let mut best_refinement = MotionWeightRefinementReportV1::default();
    let mut best_segments = creature.segments.clone();
    let mut best_quality = motion_quality.clone();
    let mut best_violations = weight_violations.clone();
    const STANDARD_REFINEMENT_ROUNDS: usize = 24;
    const TARGETED_REFINEMENT_ROUNDS: usize = 48;
    const TARGETED_TRIANGLE_REFINEMENT_ROUNDS: usize = 32;
    const CORRECTED_BIND_SMOOTHING_ROUNDS: usize = 7;
    // r23 exhausted the former 32-round guard while the measured collapse
    // extremum was still improving and the next generic target/blend pair had
    // not yet been evaluated. Match the already established generic targeted
    // budget so a corrected-bind search can finish its current policy sweep;
    // PASS and exhaustive no-improvement still stop it earlier.
    // The measured solver retains only a bounded, full-oracle-validated
    // endpoint step. R32 reached the former 96-round guard at ratio 0.06024
    // (the hard minimum is 0.0625) immediately after line search selected the
    // next safer DeeperOnly/MeasuredMotionBasis step. Reserve one additional
    // complete targeted cycle so that deterministic endpoint/target search is
    // not cut between a backoff decision and its evaluation. PASS and
    // exhaustive no-improvement continue to stop earlier.
    const CORRECTED_BIND_TARGETED_ROUNDS: usize = TARGETED_REFINEMENT_ROUNDS * 3;
    const CORRECTED_BIND_REFINEMENT_ROUNDS: usize =
        CORRECTED_BIND_SMOOTHING_ROUNDS + CORRECTED_BIND_TARGETED_ROUNDS;
    const MAXIMUM_REFINEMENT_ROUNDS: usize = STANDARD_REFINEMENT_ROUNDS
        + TARGETED_REFINEMENT_ROUNDS
        + TARGETED_TRIANGLE_REFINEMENT_ROUNDS;
    // The initial fitted weights remain the authored baseline. Deterministic
    // measured repair rounds touch only edges/triangles proven defective by
    // all required clips, preserve one carrier lineage, and rerun the complete
    // motion/joint/seam oracle transactionally. Corrected bind modes finish
    // with an adaptive catastrophic-edge phase that stops on PASS or the first
    // non-improving candidate; the constant is only a fail-safe upper bound.
    let corrected_bind_mode = matches!(
        carrier_mode,
        ReferenceSupermodelCarrierModeV4::CorrectedExactBind
            | ReferenceSupermodelCarrierModeV4::RetargetedTargetBind
    );
    // A diagnostic measures the exact authored rows. Hidden repair would make
    // its rendered model differ from the authoring document and reported rig.
    let refinement_rounds = if !enforce_motion_quality {
        0
    } else if corrected_bind_mode {
        CORRECTED_BIND_REFINEMENT_ROUNDS
    } else {
        MAXIMUM_REFINEMENT_ROUNDS
    };
    let mut catastrophic_edge_blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
    let mut targeted_triangle_blend = TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1;
    let mut catastrophic_endpoint_policy = CatastrophicEndpointPolicyV1::Symmetric;
    let mut catastrophic_target_policy = CatastrophicTargetPolicyV1::MeasuredMotionBasis;
    let mut targeted_triangle_phase = false;
    for round in 0..refinement_rounds {
        if !corrected_bind_mode && round == STANDARD_REFINEMENT_ROUNDS {
            // Start the targeted catastrophic-edge phase from the best
            // standard snapshot, not from a later exploratory state.
            creature.segments = best_segments.clone();
            motion_quality = best_quality.clone();
            weight_violations = best_violations.clone();
            explored_refinement = best_refinement.clone();
        }
        if motion_quality.status == "PASS"
            || (weight_violations.edge_penalties.is_empty()
                && weight_violations.triangle_penalties.is_empty())
        {
            break;
        }
        let original_segments = creature.segments.clone();
        let severities =
            local_deformation_limit_severities_v1(&motion_quality, &contract.tolerances);
        let corrected_bind_targeted_phase =
            corrected_bind_mode && round >= CORRECTED_BIND_SMOOTHING_ROUNDS;
        let direct_bind_targeted_phase =
            !corrected_bind_mode && round >= STANDARD_REFINEMENT_ROUNDS;
        let targeted_phase = corrected_bind_targeted_phase || direct_bind_targeted_phase;
        let hard_edge_blocked = severities[0] > 1.0 || severities[1] > 1.0;
        let hard_triangle_blocked = severities[2] > 1.0 || severities[3] > 1.0;
        if targeted_phase && !hard_edge_blocked && hard_triangle_blocked {
            targeted_triangle_phase = true;
        }
        let corrected_bind_requires_local_deformation_repair =
            corrected_bind_targeted_phase && severities.into_iter().any(|severity| severity > 1.0);
        let corrected_bind_requires_density_repair = corrected_bind_targeted_phase
            && (!motion_density_passes_v1(&motion_quality)
                || !motion_quality.per_component_deformation_coverage);
        if corrected_bind_targeted_phase
            && !corrected_bind_requires_local_deformation_repair
            && !corrected_bind_requires_density_repair
        {
            break;
        }
        let targeted_triangle_expansion =
            if targeted_phase && targeted_triangle_phase && hard_triangle_blocked {
                Some(severities[2] >= severities[3])
            } else {
                None
            };
        let catastrophic_expansion =
            if targeted_phase && targeted_triangle_expansion.is_none() && hard_edge_blocked {
                Some(targeted_catastrophic_expansion_v1(severities))
            } else {
                // Once the absolute catastrophic fences pass, keep refining any
                // still-failing per-component or global density budget through the
                // generic measured-edge/triangle path. R35 reached valid extrema
                // but stopped here with 933 failing clip/component rows.
                None
            };
        let catastrophic_motion_basis_sets = if catastrophic_expansion.is_some()
            && catastrophic_target_policy == CatastrophicTargetPolicyV1::MeasuredMotionBasis
        {
            build_top_catastrophic_edge_motion_basis_sets_v1(
                &extension.binary.inspection,
                animation_source,
                creature,
                &weight_violations,
                catastrophic_expansion.unwrap_or(false),
                8,
            )?
        } else {
            None
        };
        let restrict_to_failed_components = targeted_density_repair_should_restrict_v1(
            targeted_phase,
            severities,
            motion_density_passes_v1(&motion_quality),
            motion_quality.per_component_deformation_coverage,
        );
        let attempted = match refine_motion_violating_skin_weights_targeted_v2(
            creature,
            contract,
            &weight_violations,
            catastrophic_expansion,
            targeted_triangle_expansion,
            restrict_to_failed_components,
            catastrophic_edge_blend,
            targeted_triangle_blend,
            catastrophic_endpoint_policy,
            catastrophic_target_policy,
            catastrophic_motion_basis_sets.as_deref(),
        ) {
            Ok(attempted) => attempted,
            Err(error) => {
                // Refinement is a speculative repair pass. A topology that
                // cannot be projected (for example the owned correction-bone
                // layer used by exact inherited carriers) must retain the
                // measured pre-refinement diagnostic instead of making the
                // diagnostic route itself unavailable.
                creature.segments = original_segments;
                explored_refinement.rejected_round_count += 1;
                explored_refinement.last_rejection = Some(error.to_string());
                break;
            }
        };
        record_motion_weight_refinement_attempt_v1(&mut explored_refinement, &attempted);
        if !attempted.applied {
            if targeted_triangle_expansion.is_some()
                && advance_targeted_triangle_blend_search_v1(&mut targeted_triangle_blend)
            {
                creature.segments = original_segments;
                continue;
            }
            if catastrophic_expansion.is_some() && catastrophic_target_policy.next().is_some() {
                creature.segments = original_segments;
                catastrophic_target_policy = catastrophic_target_policy
                    .next()
                    .expect("checked targeted target policy successor");
                catastrophic_edge_blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
                continue;
            }
            if catastrophic_expansion.is_some() && hard_triangle_blocked {
                creature.segments = best_segments.clone();
                motion_quality = best_quality.clone();
                weight_violations = best_violations.clone();
                targeted_triangle_phase = true;
                targeted_triangle_blend = TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1;
                continue;
            }
            if catastrophic_expansion.is_some() && catastrophic_endpoint_policy.next().is_some() {
                creature.segments = original_segments;
                catastrophic_endpoint_policy = catastrophic_endpoint_policy
                    .next()
                    .expect("checked targeted endpoint policy successor");
                catastrophic_target_policy = CatastrophicTargetPolicyV1::MeasuredMotionBasis;
                catastrophic_edge_blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
                continue;
            }
            if direct_exact_bind_has_pending_targeted_phase_v1(
                corrected_bind_mode,
                round,
                STANDARD_REFINEMENT_ROUNDS,
            ) {
                // A plateau in the broad smoothing phase is not a terminal
                // result for an exact inherited bind.  The measured-edge
                // phase owns catastrophic local collapse/expansion repair;
                // keep the best snapshot and advance to that phase instead
                // of exiting before a single catastrophic edge is examined.
                creature.segments = best_segments.clone();
                motion_quality = best_quality.clone();
                weight_violations = best_violations.clone();
                continue;
            }
            break;
        }
        let candidate_extension = build_minimal_twosided_material_model_v1(
            creature,
            contract,
            &effective_writer,
            &material_state,
            &local_retargeted_animations,
        )?;
        let candidate_animation_source = if local_animation_source {
            &candidate_extension.binary.inspection
        } else {
            reference_supermodel
        };
        let (mut candidate_quality, mut candidate_violations) =
            match inspect_inherited_motion_quality_with_refinement_v1(
                &candidate_extension.binary.inspection,
                candidate_animation_source,
                contract,
                &correction.report,
            ) {
                Ok(candidate) => candidate,
                Err(error) => {
                    creature.segments = original_segments;
                    explored_refinement.rejected_round_count += 1;
                    explored_refinement.last_rejection = Some(error.to_string());
                    break;
                }
            };
        if !motion_weight_refinement_is_strict_improvement_v1(
            &motion_quality,
            &candidate_quality,
            &contract.tolerances,
        ) {
            let backtracked_group_count = revert_regressed_motion_edge_groups_v1(
                creature,
                &original_segments,
                &weight_violations,
                &candidate_violations,
                catastrophic_expansion,
            )?;
            if backtracked_group_count > 0 {
                explored_refinement.backtracked_group_count += backtracked_group_count;
                let backtracked_extension = build_minimal_twosided_material_model_v1(
                    creature,
                    contract,
                    &effective_writer,
                    &material_state,
                    &local_retargeted_animations,
                )?;
                let backtracked_animation_source = if local_animation_source {
                    &backtracked_extension.binary.inspection
                } else {
                    reference_supermodel
                };
                (candidate_quality, candidate_violations) =
                    match inspect_inherited_motion_quality_with_refinement_v1(
                        &backtracked_extension.binary.inspection,
                        backtracked_animation_source,
                        contract,
                        &correction.report,
                    ) {
                        Ok(candidate) => candidate,
                        Err(error) => {
                            creature.segments = original_segments;
                            explored_refinement.rejected_round_count += 1;
                            explored_refinement.last_rejection = Some(error.to_string());
                            break;
                        }
                    };
            }
        }
        if restrict_to_failed_components {
            // Enforce the absolute deformation fence only after every other
            // candidate mutation has completed. The legacy edge-group
            // rollback above may restore one side of an edge to the retained
            // snapshot while leaving the other side refined; R46 proved that
            // this can create a fresh max-edge expansion after an earlier
            // absolute-fence pass. Attribute only actual regressions measured
            // by the full oracle, restore the single changed driver, and
            // re-run the oracle until the final transaction introduces no new
            // absolute edge or triangle regression.
            loop {
                let restored_driver_count = restore_absolute_deformation_regression_drivers_v1(
                    creature,
                    &original_segments,
                    &motion_quality,
                    &candidate_quality,
                    &contract.tolerances,
                )?;
                if restored_driver_count == 0 {
                    break;
                }
                explored_refinement.backtracked_group_count += restored_driver_count;
                let driver_backtracked_extension = build_minimal_twosided_material_model_v1(
                    creature,
                    contract,
                    &effective_writer,
                    &material_state,
                    &local_retargeted_animations,
                )?;
                let driver_backtracked_animation_source = if local_animation_source {
                    &driver_backtracked_extension.binary.inspection
                } else {
                    reference_supermodel
                };
                (candidate_quality, candidate_violations) =
                    inspect_inherited_motion_quality_with_refinement_v1(
                        &driver_backtracked_extension.binary.inspection,
                        driver_backtracked_animation_source,
                        contract,
                        &correction.report,
                    )?;
            }
        }
        let targeted_plateau_improvement = catastrophic_expansion.is_some_and(|expansion| {
            targeted_catastrophic_plateau_is_strict_improvement_v1(
                &motion_quality,
                &candidate_quality,
                &weight_violations,
                &candidate_violations,
                expansion,
                &contract.tolerances,
            )
        });
        if !motion_weight_refinement_exploration_is_admissible_v1(
            &motion_quality,
            &candidate_quality,
            &contract.tolerances,
        ) && !targeted_plateau_improvement
        {
            creature.segments = original_segments;
            let original_extrema =
                local_deformation_limit_severities_v1(&motion_quality, &contract.tolerances);
            let candidate_extrema =
                local_deformation_limit_severities_v1(&candidate_quality, &contract.tolerances);
            explored_refinement.rejected_round_count += 1;
            explored_refinement.last_rejection = Some(format!(
                "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-EXPLORATION-REJECTED: originalLimitSeverities={original_extrema:?}, candidateLimitSeverities={candidate_extrema:?}; original {}; candidate {}",
                motion_quality_blocked_summary_v4("quality", &motion_quality),
                motion_quality_blocked_summary_v4("quality", &candidate_quality),
            ));
            if catastrophic_expansion.is_some()
                && advance_targeted_refinement_search_v1(
                    &motion_quality,
                    &candidate_quality,
                    catastrophic_expansion.unwrap_or(false),
                    &contract.tolerances,
                    &mut catastrophic_edge_blend,
                    &mut catastrophic_endpoint_policy,
                    &mut catastrophic_target_policy,
                )
            {
                continue;
            }
            if targeted_triangle_expansion.is_some()
                && advance_targeted_triangle_blend_search_v1(&mut targeted_triangle_blend)
            {
                continue;
            }
            if catastrophic_expansion.is_some() && hard_triangle_blocked {
                targeted_triangle_phase = true;
                targeted_triangle_blend = TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1;
                continue;
            }
            break;
        }
        // Near the final fence, either a measured catastrophic repair or the
        // subsequent generic density repair can improve its intended metric
        // while a different incident edge crosses an already-satisfied global
        // fence. Regenerating a candidate with another local policy does not
        // necessarily scale every changed endpoint by the same amount.
        // Line-search the actual full-oracle transaction back toward the
        // retained snapshot for both phases, so the candidate is judged on
        // the exact rows that would be emitted.
        let best_extrema =
            local_deformation_limit_severities_v1(&best_quality, &contract.tolerances);
        let candidate_is_strict_improvement = motion_weight_refinement_is_strict_improvement_v1(
            &best_quality,
            &candidate_quality,
            &contract.tolerances,
        );
        let full_oracle_transaction_needs_backtrack = catastrophic_expansion.map_or_else(
            || {
                if let Some(expansion) = targeted_triangle_expansion {
                    let candidate_extrema = local_deformation_limit_severities_v1(
                        &candidate_quality,
                        &contract.tolerances,
                    );
                    let selected_axis = 2 + usize::from(!expansion);
                    !candidate_is_strict_improvement
                        && candidate_extrema[selected_axis] < best_extrema[selected_axis]
                } else {
                    best_extrema.into_iter().all(|severity| severity <= 1.1)
                        && !candidate_is_strict_improvement
                        && generic_density_transaction_has_repair_signal_v1(
                            &best_quality,
                            &candidate_quality,
                        )
                }
            },
            |expansion| {
                let selected_axis = usize::from(!expansion);
                catastrophic_target_policy == CatastrophicTargetPolicyV1::MeasuredMotionBasis
                    && best_extrema[selected_axis] <= 1.1
                    && !candidate_is_strict_improvement
                    && targeted_refinement_step_needs_backoff_v1(
                        &best_quality,
                        &candidate_quality,
                        expansion,
                        &contract.tolerances,
                    )
            },
        );
        if full_oracle_transaction_needs_backtrack {
            // A retained extremum can legitimately sit less than 0.1% from
            // its absolute fence. Seven halvings stop at 1/128, which R40
            // proved can still be too coarse (0.999715 -> 1.113945 at full
            // strength). Continue into the finite f32-effective range and let
            // the complete oracle, not an extrapolated severity, choose the
            // first safe transaction.
            const FULL_ORACLE_TRANSACTION_BACKTRACK_STEPS: usize = 16;
            let full_candidate_segments = creature.segments.clone();
            let full_candidate_quality = candidate_quality.clone();
            let full_candidate_violations = candidate_violations.clone();
            let mut retained_backtrack = false;
            for step in 1..=FULL_ORACLE_TRANSACTION_BACKTRACK_STEPS {
                let blend = 0.5_f32.powi(step as i32);
                let backtracked_rows = interpolate_motion_weight_transaction_v1(
                    creature,
                    &best_segments,
                    &full_candidate_segments,
                    blend,
                )?;
                let backtracked_extension = build_minimal_twosided_material_model_v1(
                    creature,
                    contract,
                    &effective_writer,
                    &material_state,
                    &local_retargeted_animations,
                )?;
                let backtracked_animation_source = if local_animation_source {
                    &backtracked_extension.binary.inspection
                } else {
                    reference_supermodel
                };
                let (backtracked_quality, backtracked_violations) =
                    inspect_inherited_motion_quality_with_refinement_v1(
                        &backtracked_extension.binary.inspection,
                        backtracked_animation_source,
                        contract,
                        &correction.report,
                    )?;
                let backtracked_retained = motion_weight_refinement_is_strict_improvement_v1(
                    &best_quality,
                    &backtracked_quality,
                    &contract.tolerances,
                ) || catastrophic_expansion.is_some_and(|expansion| {
                    targeted_catastrophic_plateau_is_strict_improvement_v1(
                        &best_quality,
                        &backtracked_quality,
                        &best_violations,
                        &backtracked_violations,
                        expansion,
                        &contract.tolerances,
                    )
                });
                if backtracked_retained {
                    explored_refinement.backtracked_group_count += backtracked_rows;
                    candidate_quality = backtracked_quality;
                    candidate_violations = backtracked_violations;
                    retained_backtrack = true;
                    break;
                }
            }
            if !retained_backtrack {
                creature.segments = full_candidate_segments;
                candidate_quality = full_candidate_quality;
                candidate_violations = full_candidate_violations;
            }
        }
        let retain_candidate = motion_weight_refinement_is_strict_improvement_v1(
            &best_quality,
            &candidate_quality,
            &contract.tolerances,
        ) || catastrophic_expansion.is_some_and(|expansion| {
            targeted_catastrophic_plateau_is_strict_improvement_v1(
                &best_quality,
                &candidate_quality,
                &best_violations,
                &candidate_violations,
                expansion,
                &contract.tolerances,
            )
        });
        motion_quality = candidate_quality;
        weight_violations = candidate_violations;
        if retain_candidate {
            explored_refinement.applied = true;
            explored_refinement.iteration_count += attempted.iteration_count;
            explored_refinement.edge_count += attempted.edge_count;
            explored_refinement.rigid_component_count += attempted.rigid_component_count;
            explored_refinement.projected_triangle_count += attempted.projected_triangle_count;
            best_segments = creature.segments.clone();
            best_quality = motion_quality.clone();
            best_violations = weight_violations.clone();
            best_refinement = explored_refinement.clone();
            catastrophic_edge_blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
            targeted_triangle_blend = TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1;
            catastrophic_endpoint_policy = CatastrophicEndpointPolicyV1::Symmetric;
            catastrophic_target_policy = CatastrophicTargetPolicyV1::MeasuredMotionBasis;
            let retained_extrema =
                local_deformation_limit_severities_v1(&best_quality, &contract.tolerances);
            targeted_triangle_phase = (retained_extrema[2] > 1.0 || retained_extrema[3] > 1.0)
                && (targeted_triangle_expansion.is_some()
                    || (retained_extrema[0] <= 1.0 && retained_extrema[1] <= 1.0));
        } else {
            let retry_targeted_search = catastrophic_expansion.is_some()
                && advance_targeted_refinement_search_v1(
                    &best_quality,
                    &motion_quality,
                    catastrophic_expansion.unwrap_or(false),
                    &contract.tolerances,
                    &mut catastrophic_edge_blend,
                    &mut catastrophic_endpoint_policy,
                    &mut catastrophic_target_policy,
                );
            explored_refinement.rejected_round_count += 1;
            explored_refinement.last_rejection = Some(format!(
                "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-NO-STRICT-IMPROVEMENT: catastrophicBlend={catastrophic_edge_blend}, nextEndpointPolicy={catastrophic_endpoint_policy:?}, nextTargetPolicy={catastrophic_target_policy:?}, originalLimitSeverities={:?}, candidateLimitSeverities={:?}; original {}; candidate {}",
                local_deformation_limit_severities_v1(&best_quality, &contract.tolerances),
                local_deformation_limit_severities_v1(&motion_quality, &contract.tolerances),
                motion_quality_blocked_summary_v4("quality", &best_quality),
                motion_quality_blocked_summary_v4("quality", &motion_quality),
            ));
            if retry_targeted_search {
                creature.segments = best_segments.clone();
                motion_quality = best_quality.clone();
                weight_violations = best_violations.clone();
                continue;
            }
            if targeted_triangle_expansion.is_some()
                && advance_targeted_triangle_blend_search_v1(&mut targeted_triangle_blend)
            {
                creature.segments = best_segments.clone();
                motion_quality = best_quality.clone();
                weight_violations = best_violations.clone();
                continue;
            }
            if catastrophic_expansion.is_some() && hard_triangle_blocked {
                creature.segments = best_segments.clone();
                motion_quality = best_quality.clone();
                weight_violations = best_violations.clone();
                targeted_triangle_phase = true;
                targeted_triangle_blend = TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1;
                continue;
            }
            if direct_exact_bind_has_pending_targeted_phase_v1(
                corrected_bind_mode,
                round,
                STANDARD_REFINEMENT_ROUNDS,
            ) {
                creature.segments = best_segments.clone();
                motion_quality = best_quality.clone();
                weight_violations = best_violations.clone();
                continue;
            }
            break;
        }
    }
    if explored_refinement.last_rejection.is_some() {
        best_refinement.rejected_round_count = explored_refinement.rejected_round_count;
        best_refinement.last_rejection = explored_refinement.last_rejection.clone();
    }
    copy_motion_weight_refinement_attempt_diagnostics_v1(
        &mut best_refinement,
        &explored_refinement,
    );
    creature.segments = best_segments;
    motion_quality = best_quality;
    let weight_refinement = best_refinement;
    extension = build_minimal_twosided_material_model_v1(
        creature,
        contract,
        &effective_writer,
        &material_state,
        &local_retargeted_animations,
    )?;
    let tangent_stream_count = validate_minimal_twosided_motion_model_v1(
        &extension.binary.inspection.node_tree.roots,
        &material_options.texture_resref,
        &material_options.material_resref,
    )?;
    if enforce_motion_quality && motion_quality.status != "PASS" {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-MOTION-QUALITY-BLOCKED",
            "motionQuality",
            motion_quality_blocked_summary_v4(
                "minimal two-sided inherited deformation failed the motion quality gate",
                &motion_quality,
            ),
        ));
    }
    let bind_pose_compatible =
        bind_pose_compatible_for_mode_v4(&correction.report, contract, carrier_mode);
    let skin_bind_compatible = extension.binary.report.semantic_diff.is_empty();
    let carrier_coverage = audit_output_carrier_readback_v3(&extension.binary.inspection, contract);
    let (
        skin_influence_coverage,
        allowed_bone_count,
        active_weighted_bone_count,
        unweighted_required_joint_names,
        passive_unweighted_joint_names,
    ) = audit_target_rig_skin_influences_v3(target_rig, contract);
    let motion_compatible = bind_pose_compatible
        && skin_bind_compatible
        && carrier_coverage.full_carrier_coverage
        && skin_influence_coverage
        && extension
            .binary
            .inspection
            .model
            .supermodel_name
            .eq_ignore_ascii_case(&contract.supermodel_resref)
        && motion_quality.status == "PASS";
    if enforce_motion_quality && !motion_compatible {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-COMPATIBILITY-NOT-PROVEN",
            "report.motionCompatible",
            "supermodel lookup, exact reference bind, skin readback and sampled motion must all pass",
        ));
    }
    let report = ReferenceSupermodelClassicMotionBuildReportV1 {
        schema_version: 1,
        motion_compatible,
        bind_pose_compatible,
        skin_bind_compatible,
        source_sha256: ingest.ir.source.sha256.clone(),
        source_rig_sha256: target_rig.content_sha256.clone(),
        output_rig_sha256: correction.rig.content_sha256.clone(),
        motion_contract_sha256: contract.content_sha256.clone(),
        supermodel_resref: contract.supermodel_resref.clone(),
        model_resource_resref: effective_writer.model_resource_resref,
        model_sha256: extension.binary.report.payload_sha256.clone(),
        material_profile: "NWN_EE_MTR_TWOSIDED_ONLY_V1".to_owned(),
        material_extension_applied: true,
        tangent_stream_count,
        material_resource_count: 2,
        runtime_readiness: if motion_quality.status == "PASS" {
            "RUNTIME_UNPROVEN"
        } else {
            "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED"
        }
        .to_owned(),
        required_clip_count: contract.required_clips.len(),
        carrier_coverage,
        skin_influence_coverage,
        allowed_bone_count,
        active_weighted_bone_count,
        unweighted_required_joint_names,
        passive_unweighted_joint_names,
        motion_weight_refinement_applied: weight_refinement.applied,
        motion_weight_refinement_attempted_round_count: weight_refinement.attempted_round_count,
        motion_weight_refinement_examined_edge_count: weight_refinement.examined_edge_count,
        motion_weight_refinement_measured_triangle_count: weight_refinement.measured_triangle_count,
        motion_weight_refinement_smoothing_candidate_group_count: weight_refinement
            .smoothing_candidate_group_count,
        motion_weight_refinement_smoothing_shared_support_group_count: weight_refinement
            .smoothing_shared_support_group_count,
        motion_weight_refinement_smoothing_changed_group_count: weight_refinement
            .smoothing_changed_group_count,
        motion_weight_refinement_micro_triangle_candidate_count: weight_refinement
            .micro_triangle_candidate_count,
        motion_weight_refinement_micro_triangle_too_large_count: weight_refinement
            .micro_triangle_too_large_count,
        motion_weight_refinement_micro_triangle_protected_support_count: weight_refinement
            .micro_triangle_protected_support_count,
        motion_weight_refinement_micro_triangle_nonlocal_support_count: weight_refinement
            .micro_triangle_nonlocal_support_count,
        motion_weight_refinement_micro_triangle_no_common_support_count: weight_refinement
            .micro_triangle_no_common_support_count,
        motion_weight_refinement_micro_triangle_changed_count: weight_refinement
            .micro_triangle_changed_count,
        motion_weight_refinement_catastrophic_edge_candidate_count: weight_refinement
            .catastrophic_edge_candidate_count,
        motion_weight_refinement_catastrophic_edge_count: weight_refinement.catastrophic_edge_count,
        motion_weight_refinement_catastrophic_edge_topology_skip_count: weight_refinement
            .catastrophic_edge_topology_skip_count,
        motion_weight_refinement_catastrophic_edge_conflict_skip_count: weight_refinement
            .catastrophic_edge_conflict_skip_count,
        motion_weight_refinement_catastrophic_anchor_restore_count: weight_refinement
            .catastrophic_anchor_restore_count,
        motion_weight_refinement_catastrophic_peak_edge: weight_refinement.catastrophic_peak_edge,
        motion_weight_refinement_catastrophic_peak_edge_changed: weight_refinement
            .catastrophic_peak_edge_changed,
        motion_weight_refinement_backtracked_group_count: weight_refinement.backtracked_group_count,
        motion_weight_refinement_iteration_count: weight_refinement.iteration_count,
        motion_weight_refinement_edge_count: weight_refinement.edge_count,
        motion_weight_refinement_rigid_component_count: weight_refinement.rigid_component_count,
        motion_weight_refinement_projected_triangle_count: weight_refinement
            .projected_triangle_count,
        motion_weight_refinement_rejected_round_count: weight_refinement.rejected_round_count,
        motion_weight_refinement_last_rejection: weight_refinement.last_rejection,
    };
    Ok(ReferenceSupermodelMinimalMtrMotionArtifactV1 {
        correction,
        conversion,
        model: extension.binary,
        material_compilation,
        material_extension: extension.material_report,
        mtr_resource,
        motion_quality,
        report,
    })
}

fn direct_exact_bind_has_pending_targeted_phase_v1(
    corrected_bind_mode: bool,
    current_round: usize,
    targeted_phase_start_round: usize,
) -> bool {
    !corrected_bind_mode && current_round < targeted_phase_start_round
}

fn targeted_density_repair_should_restrict_v1(
    targeted_phase: bool,
    severities: [f32; 4],
    global_density_passes: bool,
    per_component_deformation_coverage: bool,
) -> bool {
    targeted_phase
        && severities.into_iter().all(|severity| severity <= 1.0)
        && (!global_density_passes || !per_component_deformation_coverage)
}

fn build_minimal_twosided_material_model_v1(
    creature: &AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    writer_options: &MdlWriterOptionsV1,
    material_state: &MdlMaterialStateV1,
    animations: &MdlAnimationSetV1,
) -> Result<MdlMaterialExtensionArtifactV1, ReferenceSupermodelMotionErrorV2> {
    let base_model = write_binary_mdl_with_animations_and_supermodel_exact_face_planes_v1(
        creature,
        animations,
        &contract.supermodel_resref,
        writer_options,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let extension = extend_binary_mdl_with_materials_v1(
        creature,
        base_model,
        &MdlMaterialExtensionOptionsV1 {
            schema_version: 1,
            materials: vec![material_state.clone()],
            segment_streams: creature
                .segments
                .iter()
                .map(|segment| MdlSegmentMaterialStreamsV1 {
                    segment_id: segment.segment_id,
                    uv1: Vec::new(),
                    uv2: Vec::new(),
                    uv3: Vec::new(),
                })
                .collect(),
        },
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    if !extension.material_report.semantic_diff.is_empty()
        || !extension.binary.report.semantic_diff.is_empty()
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-MDL-SEMANTIC-DIFF",
            "model.readback",
            "minimal two-sided MDL material binding differs after semantic readback",
        ));
    }
    Ok(extension)
}

fn motion_quality_blocked_summary_v4(
    prefix: &str,
    quality: &InheritedMotionQualityReportV1,
) -> String {
    let clip_summary = quality
        .clips
        .iter()
        .map(|clip| {
            format!(
                "{}[hard={}/{},soft={}/{},collapse={}/{},expand={}/{},seam={}/{}]",
                clip.clip_name,
                clip.edge_outside_hard_limit_count,
                clip.edge_sample_count,
                clip.edge_outside_soft_limit_count,
                clip.edge_sample_count,
                clip.triangle_area_collapse_count,
                clip.triangle_sample_count,
                clip.triangle_area_expansion_count,
                clip.triangle_sample_count,
                clip.seam_pair_violation_count,
                clip.seam_pair_sample_count,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{prefix}: hardEdges={}/{}, softEdges={}/{}, collapsedTriangles={}/{}, expandedTriangles={}/{}, seamViolations={}/{}, missingAnchorClusters={}, missingPawClusters={}, clips={clip_summary}",
        quality.edge_outside_hard_limit_count,
        quality.edge_outside_hard_allowed_count,
        quality.edge_outside_soft_limit_count,
        quality.edge_outside_soft_allowed_count,
        quality.triangle_area_collapse_count,
        quality.triangle_area_collapse_allowed_count,
        quality.triangle_area_expansion_count,
        quality.triangle_area_expansion_allowed_count,
        quality.seam_pair_violation_count,
        quality.seam_pair_allowed_count,
        quality.anchor_cluster_missing_count,
        quality.paw_cluster_missing_count,
    )
}

const MOTION_WEIGHT_REFINEMENT_ITERATIONS_V1: usize = 1;
const MOTION_WEIGHT_REFINEMENT_NEIGHBOUR_BLEND_V1: f32 = 0.05;
const MOTION_WEIGHT_REFINEMENT_TRANSACTION_BLEND_V1: f32 = 0.10;
const CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1: f32 = 0.025;
const CATASTROPHIC_EDGE_MINIMUM_COHERENCE_BLEND_V1: f32 =
    CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1 / 128.0;
const CATASTROPHIC_EDGE_MAXIMUM_COHERENCE_BLEND_V1: f32 =
    CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1 * 4.0;
const TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1: f32 = 0.1;
const TARGETED_TRIANGLE_MINIMUM_RIGID_BLEND_V1: f32 =
    TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1 / 128.0;

fn build_motion_weight_duplicate_groups_v2(
    model: &AuroraModelIrV1,
) -> Result<Vec<Vec<(usize, usize)>>, ReferenceSupermodelMotionErrorV2> {
    let mut one_ring_position_keys = model
        .segments
        .iter()
        .map(|segment| vec![BTreeSet::<[u32; 3]>::new(); segment.positions.len()])
        .collect::<Vec<_>>();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for triangle in segment.indices.chunks_exact(3) {
            let vertices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            if vertices
                .iter()
                .any(|vertex| *vertex >= segment.positions.len())
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-DUPLICATE-INDEX-OOB",
                    format!("model.segments[{segment_index}].indices"),
                    "topology-aware duplicate grouping encountered an out-of-range triangle index",
                ));
            }
            for corner in 0..3 {
                for neighbour in 0..3 {
                    if corner != neighbour {
                        one_ring_position_keys[segment_index][vertices[corner]]
                            .insert(segment.positions[vertices[neighbour]].map(f32::to_bits));
                    }
                }
            }
        }
    }
    let mut buckets = BTreeMap::<[u32; 3], Vec<(usize, usize)>>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for (vertex_index, position) in segment.positions.iter().enumerate() {
            buckets
                .entry(position.map(f32::to_bits))
                .or_default()
                .push((segment_index, vertex_index));
        }
    }
    let mut groups = Vec::new();
    for members in buckets.into_values() {
        if members.len() == 1 {
            groups.push(members);
            continue;
        }
        let mut parents = (0..members.len()).collect::<Vec<_>>();
        let mut ranks = vec![0_u8; members.len()];
        for left in 0..members.len() {
            let (left_segment, left_vertex) = members[left];
            for right in (left + 1)..members.len() {
                let (right_segment, right_vertex) = members[right];
                if !one_ring_position_keys[left_segment][left_vertex]
                    .is_disjoint(&one_ring_position_keys[right_segment][right_vertex])
                {
                    union_weight_groups_v1(&mut parents, &mut ranks, left, right);
                }
            }
        }
        let mut groups_by_root = BTreeMap::<usize, Vec<(usize, usize)>>::new();
        for (member_index, member) in members.into_iter().enumerate() {
            let root = find_weight_group_root_v1(&mut parents, member_index);
            groups_by_root.entry(root).or_default().push(member);
        }
        groups.extend(groups_by_root.into_values());
    }
    for group in &mut groups {
        group.sort_unstable();
    }
    groups.sort_unstable_by_key(|group| group[0]);
    Ok(groups)
}

#[allow(clippy::too_many_arguments)]
fn refine_motion_violating_skin_weights_targeted_v2(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    violations: &MotionWeightViolationAccumulatorV1,
    catastrophic_expansion: Option<bool>,
    targeted_triangle_expansion: Option<bool>,
    restrict_to_failed_components: bool,
    catastrophic_edge_blend: f32,
    targeted_triangle_blend: f32,
    catastrophic_endpoint_policy: CatastrophicEndpointPolicyV1,
    catastrophic_target_policy: CatastrophicTargetPolicyV1,
    catastrophic_motion_basis_sets: Option<&[CatastrophicEdgeMotionBasisSetV1]>,
) -> Result<MotionWeightRefinementReportV1, ReferenceSupermodelMotionErrorV2> {
    let targeted_catastrophic_only =
        catastrophic_expansion.is_some() || targeted_triangle_expansion.is_some();
    let component_local_violations = if targeted_catastrophic_only || !restrict_to_failed_components
    {
        None
    } else {
        let subset = violations.failed_component_subset_v1();
        (!subset.edge_penalties.is_empty() || !subset.triangle_penalties.is_empty())
            .then_some(subset)
    };
    let violations = component_local_violations.as_ref().unwrap_or(violations);
    let original_weight_rows = model
        .segments
        .iter()
        .map(|segment| segment.weights.clone())
        .collect::<Vec<_>>();
    let protected_anchor_rows = snapshot_required_motion_anchor_rows_v1(model, contract)?;
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        if segment.positions.len() != segment.weights.len() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-LAYOUT",
                format!("model.segments[{segment_index}]"),
                "motion weight refinement requires one skin-weight row per render vertex",
            ));
        }
    }
    let groups = build_motion_weight_duplicate_groups_v2(model)?;
    let mut group_by_vertex = model
        .segments
        .iter()
        .map(|segment| vec![usize::MAX; segment.positions.len()])
        .collect::<Vec<_>>();
    for (group_index, members) in groups.iter().enumerate() {
        for &(segment_index, vertex_index) in members {
            group_by_vertex[segment_index][vertex_index] = group_index;
        }
    }
    let mut edge_penalties = BTreeMap::<(usize, usize), u64>::new();
    for ((node_name, left_vertex, right_vertex), penalty) in &violations.edge_penalties {
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-NODE-NAME",
                    "motionQuality.weightViolations.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        if *left_vertex >= group_by_vertex[segment_index].len()
            || *right_vertex >= group_by_vertex[segment_index].len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-VERTEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                format!(
                    "motion violation edge ({left_vertex},{right_vertex}) exceeds segment vertex count {}",
                    group_by_vertex[segment_index].len()
                ),
            ));
        }
        let left_group = group_by_vertex[segment_index][*left_vertex];
        let right_group = group_by_vertex[segment_index][*right_vertex];
        if left_group == right_group {
            continue;
        }
        let edge = if left_group < right_group {
            (left_group, right_group)
        } else {
            (right_group, left_group)
        };
        *edge_penalties.entry(edge).or_default() += *penalty;
    }
    if edge_penalties.is_empty() && violations.triangle_penalties.is_empty() {
        return Ok(MotionWeightRefinementReportV1::default());
    }
    let rigid_components = if targeted_catastrophic_only {
        RigidComponentRefinementReportV1::default()
    } else {
        rigidify_measured_isolated_components_v1(
            model,
            contract,
            &groups,
            &group_by_vertex,
            &edge_penalties,
        )?
    };
    let mut neighbours = vec![Vec::<(usize, u64)>::new(); groups.len()];
    for (&(left, right), &penalty) in &edge_penalties {
        neighbours[left].push((right, penalty));
        neighbours[right].push((left, penalty));
    }
    for row in &mut neighbours {
        row.sort_unstable_by_key(|(group, _)| *group);
    }
    let mut group_weights = groups
        .iter()
        .map(|members| {
            let mut accumulated = BTreeMap::<u32, f32>::new();
            let share = 1.0 / members.len() as f32;
            for &(segment_index, vertex_index) in members {
                accumulate_aurora_weight_row_v1(
                    &model.segments[segment_index].weights[vertex_index],
                    share,
                    &mut accumulated,
                );
            }
            normalize_aurora_weight_row_v1(accumulated)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let original_group_weights = group_weights.clone();
    let smoothing_candidate_group_count = neighbours
        .iter()
        .filter(|neighbours| !neighbours.is_empty())
        .count();
    let mut smoothing_shared_support_groups = BTreeSet::<usize>::new();
    for _ in 0..(MOTION_WEIGHT_REFINEMENT_ITERATIONS_V1 * usize::from(!targeted_catastrophic_only))
    {
        let previous = group_weights.clone();
        for group_index in 0..groups.len() {
            if neighbours[group_index].is_empty() {
                continue;
            }
            let neighbour_penalty_total = neighbours[group_index]
                .iter()
                .map(|(_, penalty)| *penalty as f64)
                .sum::<f64>();
            if !neighbour_penalty_total.is_finite() || neighbour_penalty_total <= 0.0 {
                continue;
            }
            let mut accumulated = BTreeMap::<u32, f32>::new();
            let own_support = aurora_weight_support_v1(&previous[group_index])
                .into_iter()
                .collect::<BTreeSet<_>>();
            let has_shared_influence = neighbours[group_index].iter().any(|(neighbour, _)| {
                aurora_weight_support_v1(&previous[*neighbour])
                    .into_iter()
                    .any(|bone| own_support.contains(&bone))
            });
            if !has_shared_influence {
                continue;
            }
            smoothing_shared_support_groups.insert(group_index);
            accumulate_aurora_weight_row_v1(
                &previous[group_index],
                1.0 - MOTION_WEIGHT_REFINEMENT_NEIGHBOUR_BLEND_V1,
                &mut accumulated,
            );
            for &(neighbour, penalty) in &neighbours[group_index] {
                let share = MOTION_WEIGHT_REFINEMENT_NEIGHBOUR_BLEND_V1
                    * (penalty as f64 / neighbour_penalty_total) as f32;
                for lane in 0..usize::from(previous[neighbour].influence_count.min(4)) {
                    let Some(bone) = previous[neighbour].bone_node_ids[lane] else {
                        continue;
                    };
                    let value = previous[neighbour].values[lane] * share;
                    if own_support.contains(&bone) && value.is_finite() && value > 0.0 {
                        *accumulated.entry(bone).or_default() += value;
                    }
                }
            }
            group_weights[group_index] = normalize_aurora_weight_row_v1(accumulated)?;
        }
    }
    let smoothing_changed_group_count = group_weights
        .iter()
        .zip(&original_group_weights)
        .filter(|(current, original)| current != original)
        .count();
    let smoothed = smoothing_changed_group_count > 0;
    if smoothed {
        for (group_index, members) in groups.iter().enumerate() {
            for &(segment_index, vertex_index) in members {
                model.segments[segment_index].weights[vertex_index] =
                    group_weights[group_index].clone();
            }
        }
    }
    restore_required_motion_anchor_rows_v1(model, &protected_anchor_rows);
    let mut projected_triangle_count = if targeted_catastrophic_only {
        0
    } else {
        cohere_measured_triangle_weights_v1(model, contract, violations)?
    };
    restore_required_motion_anchor_rows_v1(model, &protected_anchor_rows);
    let micro_triangles = if targeted_catastrophic_only {
        MicroTriangleRefinementReportV1::default()
    } else {
        cohere_micro_triangles_with_common_support_v1(model, contract, violations)?
    };
    projected_triangle_count += micro_triangles.changed_count;
    damp_motion_weight_refinement_to_original_support_v1(
        model,
        &original_weight_rows,
        MOTION_WEIGHT_REFINEMENT_TRANSACTION_BLEND_V1,
        &rigid_components.vertices,
    )?;
    restore_required_motion_anchor_rows_v1(model, &protected_anchor_rows);
    // Generic smoothing is support-preserving, but a measured catastrophic
    // edge can be caused precisely by two adjacent carrier rows having no
    // shared influence. Applying endpoint coherence before the support fence
    // made that repair a no-op: the fence immediately deleted the newly shared
    // parent/child influence. Run this narrowly measured repair after the
    // generic transaction, with its own conservative blend, and keep the same
    // full motion/joint/seam validation before the round can be retained.
    let catastrophic_edges = if let Some(expansion) = catastrophic_expansion {
        cohere_catastrophic_edge_endpoints_v1(
            model,
            contract,
            violations,
            expansion,
            catastrophic_edge_blend,
            catastrophic_endpoint_policy,
            catastrophic_target_policy,
            catastrophic_motion_basis_sets,
        )?
    } else {
        CatastrophicEdgeRefinementReportV1::default()
    };
    let targeted_triangles = if let Some(expansion) = targeted_triangle_expansion {
        cohere_catastrophic_triangle_toward_common_carrier_v1(
            model,
            violations,
            expansion,
            targeted_triangle_blend,
        )?
    } else {
        MicroTriangleRefinementReportV1::default()
    };
    let catastrophic_anchor_restore_count = protected_anchor_rows
        .iter()
        .filter(|(position, protected)| {
            model.segments.iter().any(|segment| {
                segment.positions.iter().zip(&segment.weights).any(
                    |(candidate_position, candidate_row)| {
                        candidate_position.map(f32::to_bits) == **position
                            && *candidate_row != protected.row
                    },
                )
            })
        })
        .count();
    restore_required_motion_anchor_rows_v1(model, &protected_anchor_rows);
    let applied = model
        .segments
        .iter()
        .zip(&original_weight_rows)
        .any(|(segment, original)| segment.weights != *original);
    Ok(MotionWeightRefinementReportV1 {
        applied,
        attempted_round_count: 1,
        examined_edge_count: edge_penalties.len(),
        measured_triangle_count: violations.triangle_penalties.len(),
        smoothing_candidate_group_count,
        smoothing_shared_support_group_count: smoothing_shared_support_groups.len(),
        smoothing_changed_group_count,
        micro_triangle_candidate_count: micro_triangles.candidate_count
            + targeted_triangles.candidate_count,
        micro_triangle_too_large_count: micro_triangles.too_large_count,
        micro_triangle_protected_support_count: micro_triangles.protected_support_count,
        micro_triangle_nonlocal_support_count: micro_triangles.nonlocal_support_count,
        micro_triangle_no_common_support_count: micro_triangles.no_common_support_count
            + targeted_triangles.no_common_support_count,
        micro_triangle_changed_count: micro_triangles.changed_count
            + targeted_triangles.changed_count,
        catastrophic_edge_candidate_count: catastrophic_edges.candidate_count,
        catastrophic_edge_count: catastrophic_edges.changed_count,
        catastrophic_edge_topology_skip_count: catastrophic_edges.topology_skip_count,
        catastrophic_edge_conflict_skip_count: catastrophic_edges.conflict_skip_count,
        catastrophic_anchor_restore_count,
        catastrophic_peak_edge: catastrophic_edges.peak_edge,
        catastrophic_peak_edge_changed: catastrophic_edges.peak_edge_changed,
        backtracked_group_count: 0,
        iteration_count: usize::from(applied) * MOTION_WEIGHT_REFINEMENT_ITERATIONS_V1,
        edge_count: edge_penalties.len(),
        rigid_component_count: rigid_components.count,
        projected_triangle_count,
        rejected_round_count: 0,
        last_rejection: None,
    })
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn refine_motion_violating_skin_weights_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    violations: &MotionWeightViolationAccumulatorV1,
    catastrophic_expansion: Option<bool>,
    restrict_to_failed_components: bool,
    catastrophic_edge_blend: f32,
    catastrophic_endpoint_policy: CatastrophicEndpointPolicyV1,
    catastrophic_target_policy: CatastrophicTargetPolicyV1,
    catastrophic_motion_basis_sets: Option<&[CatastrophicEdgeMotionBasisSetV1]>,
) -> Result<MotionWeightRefinementReportV1, ReferenceSupermodelMotionErrorV2> {
    refine_motion_violating_skin_weights_targeted_v2(
        model,
        contract,
        violations,
        catastrophic_expansion,
        None,
        restrict_to_failed_components,
        catastrophic_edge_blend,
        TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1,
        catastrophic_endpoint_policy,
        catastrophic_target_policy,
        catastrophic_motion_basis_sets,
    )
}

fn damp_motion_weight_refinement_to_original_support_v1(
    model: &mut AuroraModelIrV1,
    original_weight_rows: &[Vec<AuroraVertexWeightsV1>],
    blend: f32,
    rigid_vertices: &BTreeSet<(usize, usize)>,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    if !blend.is_finite() || !(0.0..=1.0).contains(&blend) {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-TRANSACTION-BLEND-INVALID",
            "model.segments.weights",
            "motion weight transaction blend must be finite and within [0,1]",
        ));
    }
    if original_weight_rows.len() != model.segments.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-TRANSACTION-LAYOUT",
            "model.segments.weights",
            "motion weight transaction snapshot must match the segment layout",
        ));
    }
    for (segment_index, (segment, original_rows)) in model
        .segments
        .iter_mut()
        .zip(original_weight_rows)
        .enumerate()
    {
        if segment.weights.len() != original_rows.len() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-TRANSACTION-LAYOUT",
                format!("model.segments[{segment_index}].weights"),
                "motion weight transaction snapshot must match the vertex layout",
            ));
        }
        for (vertex_index, (candidate, original)) in
            segment.weights.iter_mut().zip(original_rows).enumerate()
        {
            if candidate == original {
                continue;
            }
            if rigid_vertices.contains(&(segment_index, vertex_index)) {
                continue;
            }
            // A measured isolated rigid component may be reassigned as one
            // atomic island. Interpolating two distinct rigid carriers would
            // manufacture a bend inside geometry that intentionally has no
            // deformation boundary of its own.
            if original.influence_count == 1 && candidate.influence_count == 1 {
                continue;
            }
            let original_support = aurora_weight_support_v1(original)
                .into_iter()
                .collect::<BTreeSet<_>>();
            let mut accumulated = BTreeMap::<u32, f32>::new();
            accumulate_aurora_weight_row_v1(original, 1.0 - blend, &mut accumulated);
            for lane in 0..usize::from(candidate.influence_count.min(4)) {
                let Some(bone) = candidate.bone_node_ids[lane] else {
                    continue;
                };
                let value = candidate.values[lane] * blend;
                if original_support.contains(&bone) && value.is_finite() && value > 0.0 {
                    *accumulated.entry(bone).or_default() += value;
                }
            }
            *candidate = normalize_aurora_weight_row_v1(accumulated)?;
        }
    }
    Ok(())
}

fn interpolate_motion_weight_transaction_v1(
    model: &mut AuroraModelIrV1,
    original_segments: &[crate::model_ir::AuroraModelSegmentV1],
    candidate_segments: &[crate::model_ir::AuroraModelSegmentV1],
    candidate_blend: f32,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    if !candidate_blend.is_finite() || !(0.0..=1.0).contains(&candidate_blend) {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-ORACLE-BACKTRACK-BLEND",
            "model.segments.weights",
            "full-oracle transaction backtrack blend must be finite and within [0,1]",
        ));
    }
    if model.segments.len() != original_segments.len()
        || model.segments.len() != candidate_segments.len()
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-ORACLE-BACKTRACK-LAYOUT",
            "model.segments",
            "full-oracle transaction snapshots must match the model segment layout",
        ));
    }
    let mut backtracked_rows = 0usize;
    for (segment_index, ((segment, original), candidate)) in model
        .segments
        .iter_mut()
        .zip(original_segments)
        .zip(candidate_segments)
        .enumerate()
    {
        if segment.segment_id != original.segment_id
            || segment.segment_id != candidate.segment_id
            || segment.weights.len() != original.weights.len()
            || segment.weights.len() != candidate.weights.len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-ORACLE-BACKTRACK-LAYOUT",
                format!("model.segments[{segment_index}].weights"),
                "full-oracle transaction snapshots must match segment identity and weight rows",
            ));
        }
        for ((output, original), candidate) in segment
            .weights
            .iter_mut()
            .zip(&original.weights)
            .zip(&candidate.weights)
        {
            *output = if original == candidate {
                original.clone()
            } else {
                backtracked_rows += 1;
                blend_aurora_weight_rows_v1(original, candidate, candidate_blend)?
            };
        }
    }
    Ok(backtracked_rows)
}

fn restore_absolute_deformation_regression_drivers_v1(
    model: &mut AuroraModelIrV1,
    original_segments: &[crate::model_ir::AuroraModelSegmentV1],
    baseline: &InheritedMotionQualityReportV1,
    candidate: &InheritedMotionQualityReportV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    let absolute_min = tolerances.edge_hard_min_ratio.powi(2);
    let absolute_max = tolerances.edge_hard_max_ratio.powi(2);
    let absolute_triangle_min = tolerances.triangle_min_area_ratio.powi(2);
    let absolute_triangle_max = tolerances.triangle_max_area_ratio.powi(2);
    let baseline_clips = baseline
        .clips
        .iter()
        .map(|clip| (clip.clip_name.as_str(), clip))
        .collect::<BTreeMap<_, _>>();
    let mut endpoints = BTreeSet::<(String, usize)>::new();
    let mut regression_witness_count = 0usize;
    let mut attributed_witness_count = 0usize;
    for clip in &candidate.clips {
        let Some(baseline_clip) = baseline_clips.get(clip.clip_name.as_str()).copied() else {
            continue;
        };
        if baseline_clip.min_edge_ratio >= absolute_min && clip.min_edge_ratio < absolute_min {
            regression_witness_count += 1;
            if let Some(edge) = &clip.worst_edge_collapse {
                if let Some(endpoint) = largest_motion_endpoint_delta_v1(
                    model,
                    original_segments,
                    &edge.skin_node_name,
                    &edge.vertex_indices,
                )? {
                    endpoints.insert(endpoint);
                    attributed_witness_count += 1;
                }
            }
        }
        if baseline_clip.max_edge_ratio <= absolute_max && clip.max_edge_ratio > absolute_max {
            regression_witness_count += 1;
            if let Some(edge) = &clip.worst_edge_expansion {
                if let Some(endpoint) = largest_motion_endpoint_delta_v1(
                    model,
                    original_segments,
                    &edge.skin_node_name,
                    &edge.vertex_indices,
                )? {
                    endpoints.insert(endpoint);
                    attributed_witness_count += 1;
                }
            }
        }
        let baseline_triangle_min = baseline_clip
            .worst_triangle_area_collapse
            .as_ref()
            .map_or(1.0, |triangle| triangle.area_ratio);
        let candidate_triangle_min = clip
            .worst_triangle_area_collapse
            .as_ref()
            .map_or(1.0, |triangle| triangle.area_ratio);
        if baseline_triangle_min >= absolute_triangle_min
            && candidate_triangle_min < absolute_triangle_min
        {
            regression_witness_count += 1;
            if let Some(triangle) = &clip.worst_triangle_area_collapse {
                if let Some(endpoint) = largest_motion_endpoint_delta_v1(
                    model,
                    original_segments,
                    &triangle.skin_node_name,
                    &triangle.vertex_indices,
                )? {
                    endpoints.insert(endpoint);
                    attributed_witness_count += 1;
                }
            }
        }
        let baseline_triangle_max = baseline_clip
            .worst_triangle_area_expansion
            .as_ref()
            .map_or(1.0, |triangle| triangle.area_ratio);
        let candidate_triangle_max = clip
            .worst_triangle_area_expansion
            .as_ref()
            .map_or(1.0, |triangle| triangle.area_ratio);
        if baseline_triangle_max <= absolute_triangle_max
            && candidate_triangle_max > absolute_triangle_max
        {
            regression_witness_count += 1;
            if let Some(triangle) = &clip.worst_triangle_area_expansion {
                if let Some(endpoint) = largest_motion_endpoint_delta_v1(
                    model,
                    original_segments,
                    &triangle.skin_node_name,
                    &triangle.vertex_indices,
                )? {
                    endpoints.insert(endpoint);
                    attributed_witness_count += 1;
                }
            }
        }
    }
    let restored = restore_motion_endpoint_rows_v1(model, original_segments, &endpoints)?;
    if regression_witness_count > 0 {
        eprintln!(
            "M2A_MOTION_ABSOLUTE_DRIVER_BACKTRACK: witnesses={regression_witness_count}: attributed={attributed_witness_count}: uniqueEndpoints={}: restoredGroups={restored}",
            endpoints.len(),
        );
    }
    Ok(restored)
}

fn largest_motion_endpoint_delta_v1<const N: usize>(
    model: &AuroraModelIrV1,
    original_segments: &[crate::model_ir::AuroraModelSegmentV1],
    node_name: &str,
    vertices: &[u32; N],
) -> Result<Option<(String, usize)>, ReferenceSupermodelMotionErrorV2> {
    let segment_id = node_name
        .strip_prefix("m2a_seg_")
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-DRIVER-NODE-NAME",
                "motionQuality.clips.worstDeformation.nodeName",
                format!("renderer skin node {node_name:?} is not an owned segment node"),
            )
        })?;
    let segment_index = model
        .segments
        .iter()
        .position(|segment| segment.segment_id == segment_id)
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-DRIVER-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
    let segment = &model.segments[segment_index];
    let original = original_segments.get(segment_index).ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-DRIVER-LAYOUT",
            format!("model.segments[{segment_index}]"),
            "absolute deformation driver attribution requires the original segment layout",
        )
    })?;
    if original.segment_id != segment.segment_id
        || original.positions != segment.positions
        || original.weights.len() != segment.weights.len()
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-DRIVER-LAYOUT",
            format!("model.segments[{segment_index}]"),
            "absolute deformation driver attribution requires unchanged segment identity and geometry",
        ));
    }
    let mut selected = None::<(usize, f32)>;
    for vertex in vertices.iter().map(|vertex| *vertex as usize) {
        let current = segment.weights.get(vertex).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-DRIVER-VERTEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                format!("absolute deformation witness vertex {vertex} exceeds the segment"),
            )
        })?;
        let original_row = &original.weights[vertex];
        let delta = aurora_weight_row_l1_delta_v1(original_row, current);
        if delta > selected.map_or(1.0e-8, |(_, selected_delta)| selected_delta + 1.0e-8) {
            selected = Some((vertex, delta));
        }
    }
    Ok(selected.map(|(vertex, _)| (node_name.to_owned(), vertex)))
}

fn restore_motion_endpoint_rows_v1(
    model: &mut AuroraModelIrV1,
    original_segments: &[crate::model_ir::AuroraModelSegmentV1],
    endpoints: &BTreeSet<(String, usize)>,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    if model.segments.len() != original_segments.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-FENCE-PROTECTION-LAYOUT",
            "model.segments",
            "near-fence endpoint protection requires the original segment layout",
        ));
    }
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut protected_positions = BTreeSet::<[u32; 3]>::new();
    for (node_name, vertex) in endpoints {
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-FENCE-PROTECTION-NODE-NAME",
                    "motionQuality.clips.worstEdge.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-FENCE-PROTECTION-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        let position = model.segments[segment_index]
            .positions
            .get(*vertex)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-FENCE-PROTECTION-VERTEX-OOB",
                    format!("model.segments[{segment_index}].weights"),
                    format!("near-fence endpoint {vertex} exceeds the segment vertex count"),
                )
            })?;
        protected_positions.insert(position.map(f32::to_bits));
    }
    let mut restored_positions = BTreeSet::<[u32; 3]>::new();
    for (segment_index, (segment, original)) in
        model.segments.iter_mut().zip(original_segments).enumerate()
    {
        if segment.segment_id != original.segment_id
            || segment.positions != original.positions
            || segment.weights.len() != original.weights.len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-FENCE-PROTECTION-LAYOUT",
                format!("model.segments[{segment_index}]"),
                "near-fence endpoint protection requires unchanged segment identity and geometry",
            ));
        }
        for ((position, current), original_row) in segment
            .positions
            .iter()
            .zip(&mut segment.weights)
            .zip(&original.weights)
        {
            let key = position.map(f32::to_bits);
            if protected_positions.contains(&key) && current != original_row {
                *current = original_row.clone();
                restored_positions.insert(key);
            }
        }
    }
    Ok(restored_positions.len())
}

fn revert_regressed_motion_edge_groups_v1(
    model: &mut AuroraModelIrV1,
    original_segments: &[crate::model_ir::AuroraModelSegmentV1],
    baseline: &MotionWeightViolationAccumulatorV1,
    candidate: &MotionWeightViolationAccumulatorV1,
    protected_catastrophic_expansion: Option<bool>,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    if model.segments.len() != original_segments.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-BACKTRACK-LAYOUT",
            "model.segments",
            "motion weight backtracking requires the original segment layout",
        ));
    }
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut protected_positions = BTreeSet::<[u32; 3]>::new();
    if let Some(expansion) = protected_catastrophic_expansion {
        let baseline_catastrophic = if expansion {
            &baseline.catastrophic_expansion_edge_penalties
        } else {
            &baseline.catastrophic_collapse_edge_penalties
        };
        let candidate_catastrophic = if expansion {
            &candidate.catastrophic_expansion_edge_penalties
        } else {
            &candidate.catastrophic_collapse_edge_penalties
        };
        let mut measured = baseline_catastrophic.iter().collect::<Vec<_>>();
        measured.sort_unstable_by(|left, right| {
            let ((left_node, left_vertex, left_right_vertex), left_penalty) = left;
            let ((right_node, right_vertex, right_right_vertex), right_penalty) = right;
            right_penalty
                .cmp(left_penalty)
                .then_with(|| left_node.cmp(right_node))
                .then_with(|| left_vertex.cmp(right_vertex))
                .then_with(|| left_right_vertex.cmp(right_right_vertex))
        });
        for ((node_name, left, right), baseline_penalty) in measured {
            let candidate_penalty = candidate_catastrophic
                .get(&(node_name.clone(), *left, *right))
                .copied()
                .unwrap_or(0);
            if candidate_penalty < *baseline_penalty {
                if let Some(segment_id) = node_name
                    .strip_prefix("m2a_seg_")
                    .and_then(|value| value.parse::<u32>().ok())
                {
                    if let Some(segment_index) = segment_index_by_id.get(&segment_id).copied() {
                        let positions = &model.segments[segment_index].positions;
                        if *left < positions.len() && *right < positions.len() {
                            protected_positions.insert(positions[*left].map(f32::to_bits));
                            protected_positions.insert(positions[*right].map(f32::to_bits));
                        }
                    }
                }
            }
        }
    }
    let mut regressed_positions = BTreeSet::<[u32; 3]>::new();
    for ((node_name, left, right), candidate_penalty) in &candidate.edge_penalties {
        let baseline_penalty = baseline
            .edge_penalties
            .get(&(node_name.clone(), *left, *right))
            .copied()
            .unwrap_or(0);
        if *candidate_penalty <= baseline_penalty {
            continue;
        }
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-BACKTRACK-NODE-NAME",
                    "motionQuality.weightViolations.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-BACKTRACK-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        let positions = &model.segments[segment_index].positions;
        if *left >= positions.len() || *right >= positions.len() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-BACKTRACK-VERTEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                "regressed motion edge exceeds the segment vertex count",
            ));
        }
        for position in [positions[*left], positions[*right]] {
            let key = position.map(f32::to_bits);
            if !protected_positions.contains(&key) {
                regressed_positions.insert(key);
            }
        }
    }
    if regressed_positions.is_empty() {
        return Ok(0);
    }
    let mut reverted_positions = BTreeSet::<[u32; 3]>::new();
    for (segment_index, (segment, original)) in
        model.segments.iter_mut().zip(original_segments).enumerate()
    {
        if segment.segment_id != original.segment_id
            || segment.positions != original.positions
            || segment.weights.len() != original.weights.len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-BACKTRACK-LAYOUT",
                format!("model.segments[{segment_index}]"),
                "motion weight backtracking requires unchanged segment identity and geometry",
            ));
        }
        for ((position, current), original_row) in segment
            .positions
            .iter()
            .zip(&mut segment.weights)
            .zip(&original.weights)
        {
            let key = position.map(f32::to_bits);
            if regressed_positions.contains(&key) && current != original_row {
                *current = original_row.clone();
                reverted_positions.insert(key);
            }
        }
    }
    Ok(reverted_positions.len())
}

#[derive(Clone, Debug)]
struct ProtectedMotionAnchorRowV1 {
    row: AuroraVertexWeightsV1,
    required_bones: BTreeSet<u32>,
}

fn snapshot_required_motion_anchor_rows_v1(
    model: &AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<BTreeMap<[u32; 3], ProtectedMotionAnchorRowV1>, ReferenceSupermodelMotionErrorV2> {
    const REQUIRED_ANCHOR_INFLUENCE: f32 = 0.1;
    let mut selected = Vec::new();
    for node in &contract.nodes {
        let Some(role) = node.anchor_role.as_deref() else {
            continue;
        };
        if role.eq_ignore_ascii_case("root") || role.eq_ignore_ascii_case("motion_root") {
            continue;
        }
        let best = model
            .segments
            .iter()
            .flat_map(|segment| segment.positions.iter().zip(&segment.weights))
            .filter_map(|(position, row)| {
                let influence = aurora_weight_for_bone_v1(row, node.part_number);
                (influence >= REQUIRED_ANCHOR_INFLUENCE).then_some((
                    influence,
                    position.map(f32::to_bits),
                    row,
                ))
            })
            .max_by(|left, right| {
                left.0
                    .total_cmp(&right.0)
                    .then_with(|| right.1.cmp(&left.1))
            })
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-ANCHOR-MISSING",
                    "model.segments.weights",
                    format!(
                        "required joint role {role:?} (part {}) has no pre-refinement anchor at or above {REQUIRED_ANCHOR_INFLUENCE}",
                        node.part_number
                    ),
                )
            })?;
        selected.push((best.1, node.part_number, best.2.clone()));
    }
    let mut protected = BTreeMap::<[u32; 3], ProtectedMotionAnchorRowV1>::new();
    for (position, bone, row) in selected {
        if let Some(existing) = protected.get_mut(&position) {
            let mut accumulated = BTreeMap::<u32, f32>::new();
            accumulate_aurora_weight_row_v1(&existing.row, 1.0, &mut accumulated);
            accumulate_aurora_weight_row_v1(&row, 1.0, &mut accumulated);
            existing.row = normalize_aurora_weight_row_v1(accumulated)?;
            existing.required_bones.insert(bone);
        } else {
            protected.insert(
                position,
                ProtectedMotionAnchorRowV1 {
                    row,
                    required_bones: BTreeSet::from([bone]),
                },
            );
        }
    }
    Ok(protected)
}

fn restore_required_motion_anchor_rows_v1(
    model: &mut AuroraModelIrV1,
    protected: &BTreeMap<[u32; 3], ProtectedMotionAnchorRowV1>,
) {
    if protected.is_empty() {
        return;
    }
    for segment in &mut model.segments {
        for (position, row) in segment.positions.iter().zip(&mut segment.weights) {
            if let Some(protected_row) = protected.get(&position.map(f32::to_bits)) {
                *row = protected_row.row.clone();
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CatastrophicEdgeRefinementReportV1 {
    candidate_count: usize,
    changed_count: usize,
    topology_skip_count: usize,
    conflict_skip_count: usize,
    peak_edge: Option<String>,
    peak_edge_changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CatastrophicEndpointPolicyV1 {
    Symmetric,
    DeeperOnly,
    ShallowerOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CatastrophicTargetPolicyV1 {
    MeasuredMotionBasis,
    ProjectedCarrierEdge,
    FullCarrierLineage,
}

impl CatastrophicTargetPolicyV1 {
    fn next(self) -> Option<Self> {
        match self {
            Self::MeasuredMotionBasis => Some(Self::ProjectedCarrierEdge),
            Self::ProjectedCarrierEdge => Some(Self::FullCarrierLineage),
            Self::FullCarrierLineage => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CatastrophicEdgeMotionBasisV1 {
    node_name: String,
    vertices: [usize; 2],
    clip_name: String,
    sample_time_seconds: f32,
    measured_length_ratio: f32,
    bind_positions_by_bone: [BTreeMap<u32, [f32; 3]>; 2],
    sampled_positions_by_bone: [BTreeMap<u32, [f32; 3]>; 2],
}

#[derive(Clone, Debug, PartialEq)]
struct CatastrophicEdgeMotionBasisSetV1 {
    node_name: String,
    vertices: [usize; 2],
    bases: Vec<CatastrophicEdgeMotionBasisV1>,
}

fn build_peak_catastrophic_edge_motion_bases_v1(
    target: &InspectionReport,
    animation_source: &InspectionReport,
    model: &AuroraModelIrV1,
    violations: &MotionWeightViolationAccumulatorV1,
    expansion: bool,
) -> Result<Option<Vec<CatastrophicEdgeMotionBasisV1>>, ReferenceSupermodelMotionErrorV2> {
    let penalties = if expansion {
        &violations.catastrophic_expansion_edge_penalties
    } else {
        &violations.catastrophic_collapse_edge_penalties
    };
    let witnesses = if expansion {
        &violations.catastrophic_expansion_edge_witnesses
    } else {
        &violations.catastrophic_collapse_edge_witnesses
    };
    let peak = penalties
        .iter()
        .max_by(|left, right| left.1.cmp(right.1).then_with(|| right.0.cmp(left.0)));
    let Some(((node_name, left_vertex, right_vertex), _)) = peak else {
        return Ok(None);
    };
    let key = (node_name.clone(), *left_vertex, *right_vertex);
    let Some(witnesses) = witnesses.get(&key) else {
        return Ok(None);
    };
    let segment_id = node_name
        .strip_prefix("m2a_seg_")
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-BASIS-NODE-NAME",
                "motionQuality.weightViolations.nodeName",
                format!("renderer skin node {node_name:?} is not an owned segment node"),
            )
        })?;
    let segment = model
        .segments
        .iter()
        .find(|segment| segment.segment_id == segment_id)
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-BASIS-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
    if *left_vertex >= segment.positions.len() || *right_vertex >= segment.positions.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-BASIS-VERTEX-OOB",
            "model.segments.positions",
            "measured edge endpoint exceeds the conversion segment vertex count",
        ));
    }
    let support = [*left_vertex, *right_vertex]
        .into_iter()
        .flat_map(|vertex| aurora_weight_support_v1(&segment.weights[vertex]))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if support.is_empty() || support.len() > 4 {
        return Ok(None);
    }
    let flattened = flatten_nodes_in_tree_order(&target.node_tree.roots);
    let skin_nodes = flattened
        .iter()
        .copied()
        .filter(|node| node.name == *node_name && node.skin.is_some())
        .collect::<Vec<_>>();
    if skin_nodes.len() != 1 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-BASIS-SKIN-AMBIGUOUS",
            "target.nodeTree",
            format!("measured skin node {node_name:?} must resolve exactly once"),
        ));
    }
    let skin = skin_nodes[0].skin.as_ref().expect("filtered skin node");
    let local_positions = [
        segment.positions[*left_vertex],
        segment.positions[*right_vertex],
    ];
    let mut output = Vec::with_capacity(witnesses.len());
    for witness in witnesses.values() {
        let matrices = evaluate_reference_supermodel_node_world_matrices_v2(
            target,
            animation_source,
            &witness.clip_name,
            witness.sample_time_seconds,
            &support,
        )
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
        let mut bind_positions_by_bone = std::array::from_fn(|_| BTreeMap::new());
        let mut sampled_positions_by_bone = std::array::from_fn(|_| BTreeMap::new());
        for bone in support.iter().copied() {
            let ordinals = flattened
                .iter()
                .enumerate()
                .filter(|(_, node)| node.number == bone)
                .map(|(ordinal, _)| ordinal)
                .collect::<Vec<_>>();
            if ordinals.len() != 1 {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-BASIS-BONE-AMBIGUOUS",
                    "target.nodeTree",
                    format!("active carrier part {bone} must resolve exactly once"),
                ));
            }
            let ordinal = ordinals[0];
            if ordinal >= skin.inverse_bone_rotations_raw.len()
                || ordinal >= skin.inverse_bone_translations.len()
                || ordinal >= skin.node_to_bone_map.len()
                || skin.node_to_bone_map[ordinal] < 0
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-BASIS-INVERSE-BIND-MISSING",
                    "target.nodeTree.skin.inverseBind",
                    format!("active carrier part {bone} has no skin inverse-bind row"),
                ));
            }
            let raw = skin.inverse_bone_rotations_raw[ordinal];
            let translation = skin.inverse_bone_translations[ordinal];
            let inverse_bind = matrix_from_quaternion_translation_scale_v3(
                [raw[1], raw[2], raw[3], raw[0]],
                [translation.x, translation.y, translation.z],
                1.0,
            );
            let matrix = matrices
                .iter()
                .find(|matrix| matrix.node_part == bone)
                .ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-MOTION-BASIS-MATRIX-MISSING",
                        "motionQuality.catastrophicEdgeBasis",
                        format!("sampled carrier part {bone} is absent"),
                    )
                })?;
            let bind_skin_matrix = mul_mat4(matrix.bind_world_matrix, inverse_bind);
            let sampled_skin_matrix = mul_mat4(matrix.sampled_world_matrix, inverse_bind);
            for endpoint in 0..2 {
                bind_positions_by_bone[endpoint].insert(
                    bone,
                    transform_point_v3(bind_skin_matrix, local_positions[endpoint]),
                );
                sampled_positions_by_bone[endpoint].insert(
                    bone,
                    transform_point_v3(sampled_skin_matrix, local_positions[endpoint]),
                );
            }
        }
        let basis = CatastrophicEdgeMotionBasisV1 {
            node_name: node_name.clone(),
            vertices: [*left_vertex, *right_vertex],
            clip_name: witness.clip_name.clone(),
            sample_time_seconds: witness.sample_time_seconds,
            measured_length_ratio: witness.length_ratio,
            bind_positions_by_bone,
            sampled_positions_by_bone,
        };
        let rows = [
            segment.weights[*left_vertex].clone(),
            segment.weights[*right_vertex].clone(),
        ];
        let reconstructed_ratio = edge_motion_basis_ratio_v1(&rows, &basis).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-BASIS-RECONSTRUCTION-NONFINITE",
                "motionQuality.catastrophicEdgeBasis",
                format!(
                    "clip {:?} at {}s cannot reconstruct the measured edge ratio",
                    witness.clip_name, witness.sample_time_seconds
                ),
            )
        })?;
        let reconstruction_tolerance = witness.length_ratio.abs().max(1.0) * 1.0e-4;
        if (reconstructed_ratio - witness.length_ratio).abs() > reconstruction_tolerance {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-BASIS-RECONSTRUCTION-MISMATCH",
                "motionQuality.catastrophicEdgeBasis",
                format!(
                    "clip {:?} at {}s measured edge ratio {} but the exact joint/inverse-bind basis reconstructed {}",
                    witness.clip_name,
                    witness.sample_time_seconds,
                    witness.length_ratio,
                    reconstructed_ratio,
                ),
            ));
        }
        output.push(basis);
    }
    if output.is_empty() {
        Err(motion_error(
            "M2A-SUPERMODEL-MOTION-BASIS-WITNESS-EMPTY",
            "motionQuality.catastrophicEdgeBasis",
            "catastrophic edge witness set must not be empty",
        ))
    } else {
        Ok(Some(output))
    }
}

fn build_top_catastrophic_edge_motion_basis_sets_v1(
    target: &InspectionReport,
    animation_source: &InspectionReport,
    model: &AuroraModelIrV1,
    violations: &MotionWeightViolationAccumulatorV1,
    expansion: bool,
    maximum_sets: usize,
) -> Result<Option<Vec<CatastrophicEdgeMotionBasisSetV1>>, ReferenceSupermodelMotionErrorV2> {
    let penalties = if expansion {
        &violations.catastrophic_expansion_edge_penalties
    } else {
        &violations.catastrophic_collapse_edge_penalties
    };
    let witnesses = if expansion {
        &violations.catastrophic_expansion_edge_witnesses
    } else {
        &violations.catastrophic_collapse_edge_witnesses
    };
    let mut measured = penalties.iter().collect::<Vec<_>>();
    measured.sort_unstable_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    let mut output = Vec::new();
    for (key, penalty) in measured.into_iter().take(maximum_sets) {
        let Some(edge_witnesses) = witnesses.get(key) else {
            continue;
        };
        let mut selected = MotionWeightViolationAccumulatorV1::default();
        let selected_penalties = if expansion {
            &mut selected.catastrophic_expansion_edge_penalties
        } else {
            &mut selected.catastrophic_collapse_edge_penalties
        };
        selected_penalties.insert(key.clone(), *penalty);
        let selected_witnesses = if expansion {
            &mut selected.catastrophic_expansion_edge_witnesses
        } else {
            &mut selected.catastrophic_collapse_edge_witnesses
        };
        selected_witnesses.insert(key.clone(), edge_witnesses.clone());
        let Some(bases) = build_peak_catastrophic_edge_motion_bases_v1(
            target,
            animation_source,
            model,
            &selected,
            expansion,
        )?
        else {
            continue;
        };
        output.push(CatastrophicEdgeMotionBasisSetV1 {
            node_name: key.0.clone(),
            vertices: [key.1, key.2],
            bases,
        });
    }
    Ok((!output.is_empty()).then_some(output))
}

impl CatastrophicEndpointPolicyV1 {
    fn next(self) -> Option<Self> {
        match self {
            Self::Symmetric => Some(Self::DeeperOnly),
            Self::DeeperOnly => Some(Self::ShallowerOnly),
            Self::ShallowerOnly => None,
        }
    }
}

fn blend_aurora_weight_rows_v1(
    current: &AuroraVertexWeightsV1,
    target: &AuroraVertexWeightsV1,
    blend: f32,
) -> Result<AuroraVertexWeightsV1, ReferenceSupermodelMotionErrorV2> {
    let mut values = BTreeMap::<u32, f32>::new();
    accumulate_aurora_weight_row_v1(current, 1.0 - blend, &mut values);
    accumulate_aurora_weight_row_v1(target, blend, &mut values);
    normalize_aurora_weight_row_v1(values)
}

fn weighted_motion_basis_position_v1(
    row: &AuroraVertexWeightsV1,
    basis: &BTreeMap<u32, [f32; 3]>,
) -> Option<[f32; 3]> {
    let mut output = [0.0; 3];
    let mut sum = 0.0;
    for lane in 0..usize::from(row.influence_count) {
        let bone = row.bone_node_ids[lane]?;
        let weight = row.values[lane];
        let position = basis.get(&bone)?;
        for axis in 0..3 {
            output[axis] += position[axis] * weight;
        }
        sum += weight;
    }
    (sum.is_finite() && (sum - 1.0).abs() <= 1.0e-4).then_some(output)
}

fn edge_motion_basis_ratio_v1(
    rows: &[AuroraVertexWeightsV1; 2],
    basis: &CatastrophicEdgeMotionBasisV1,
) -> Option<f32> {
    let bind: [Option<[f32; 3]>; 2] = std::array::from_fn(|endpoint| {
        weighted_motion_basis_position_v1(&rows[endpoint], &basis.bind_positions_by_bone[endpoint])
    });
    let sampled: [Option<[f32; 3]>; 2] = std::array::from_fn(|endpoint| {
        weighted_motion_basis_position_v1(
            &rows[endpoint],
            &basis.sampled_positions_by_bone[endpoint],
        )
    });
    let bind = [bind[0]?, bind[1]?];
    let sampled = [sampled[0]?, sampled[1]?];
    let bind_length = distance(bind[0], bind[1]);
    if !bind_length.is_finite() || bind_length <= 1.0e-8 {
        return None;
    }
    let ratio = distance(sampled[0], sampled[1]) / bind_length;
    ratio.is_finite().then_some(ratio)
}

fn edge_motion_basis_extreme_ratio_v1(
    rows: &[AuroraVertexWeightsV1; 2],
    bases: &[CatastrophicEdgeMotionBasisV1],
    expansion: bool,
) -> Option<f32> {
    if bases.is_empty() {
        return None;
    }
    let mut ratios = bases
        .iter()
        .map(|basis| edge_motion_basis_ratio_v1(rows, basis));
    let first = ratios.next()??;
    ratios.try_fold(first, |extreme, ratio| {
        let ratio = ratio?;
        Some(if expansion {
            extreme.max(ratio)
        } else {
            extreme.min(ratio)
        })
    })
}

fn aurora_weight_row_l1_delta_v1(
    left: &AuroraVertexWeightsV1,
    right: &AuroraVertexWeightsV1,
) -> f32 {
    aurora_weight_support_v1(left)
        .into_iter()
        .chain(aurora_weight_support_v1(right))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|bone| {
            (aurora_weight_for_bone_v1(left, bone) - aurora_weight_for_bone_v1(right, bone)).abs()
        })
        .sum()
}

fn measured_motion_row_candidates_v1(
    current: &AuroraVertexWeightsV1,
    other: &AuroraVertexWeightsV1,
    support: &[u32],
    strength: f32,
) -> Result<Vec<AuroraVertexWeightsV1>, ReferenceSupermodelMotionErrorV2> {
    let mut average = BTreeMap::<u32, f32>::new();
    accumulate_aurora_weight_row_v1(current, 0.5, &mut average);
    accumulate_aurora_weight_row_v1(other, 0.5, &mut average);
    let average = normalize_aurora_weight_row_v1(average)?;
    let mut targets = vec![average, other.clone()];
    targets.extend(support.iter().copied().map(|bone| AuroraVertexWeightsV1 {
        bone_node_ids: [Some(bone), None, None, None],
        values: [1.0, 0.0, 0.0, 0.0],
        influence_count: 1,
    }));
    let mut candidates = vec![current.clone()];
    for target in targets {
        for fraction in [0.25, 0.5, 1.0] {
            let blend = (strength * fraction).clamp(0.0, 1.0);
            if blend <= 0.0 {
                continue;
            }
            let candidate = blend_aurora_weight_rows_v1(current, &target, blend)?;
            if !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }
    }
    Ok(candidates)
}

fn solve_catastrophic_edge_from_motion_basis_v1(
    rows: &[AuroraVertexWeightsV1; 2],
    bases: &[CatastrophicEdgeMotionBasisV1],
    expansion: bool,
    strength: f32,
    frozen_lane: Option<usize>,
    absolute_edge_min: f32,
    absolute_edge_max: f32,
) -> Result<Option<[AuroraVertexWeightsV1; 2]>, ReferenceSupermodelMotionErrorV2> {
    // Keep one measured edge repair inside a small trust region. The full
    // motion oracle still decides whether the transaction is retained, but
    // this cap prevents a locally attractive sampled-basis solution from
    // replacing an endpoint row wholesale before incident edges are checked.
    // Strength 0.25 is the normal first measured step. Below it, scale the
    // trust region with line search so a requested backoff immediately changes
    // the candidate instead of repeatedly selecting the same fixed 0.25 row
    // delta (the R32/R33 plateau).
    const INITIAL_MEASURED_EDGE_STRENGTH: f32 = 0.25;
    const MAXIMUM_INCREMENTAL_EDGE_L1_DELTA: f32 = 0.25;
    let maximum_incremental_edge_l1_delta = MAXIMUM_INCREMENTAL_EDGE_L1_DELTA
        * (strength / INITIAL_MEASURED_EDGE_STRENGTH).clamp(1.0 / 128.0, 1.0);
    let support = rows
        .iter()
        .flat_map(aurora_weight_support_v1)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if support.is_empty() || support.len() > 4 || !strength.is_finite() {
        return Ok(None);
    }
    let mut candidates = [
        measured_motion_row_candidates_v1(&rows[0], &rows[1], &support, strength)?,
        measured_motion_row_candidates_v1(&rows[1], &rows[0], &support, strength)?,
    ];
    if let Some(lane) = frozen_lane {
        if lane > 1 {
            return Ok(None);
        }
        candidates[lane] = vec![rows[lane].clone()];
    }
    let Some(original_ratio) = edge_motion_basis_extreme_ratio_v1(rows, bases, expansion) else {
        return Ok(None);
    };
    let mut best: Option<([AuroraVertexWeightsV1; 2], f32, f32)> = None;
    for left in &candidates[0] {
        for right in &candidates[1] {
            let candidate = [left.clone(), right.clone()];
            if candidate == *rows {
                continue;
            }
            let Some(ratio) = edge_motion_basis_extreme_ratio_v1(&candidate, bases, expansion)
            else {
                continue;
            };
            let improves = if expansion {
                ratio + 1.0e-5 < original_ratio
            } else {
                ratio > original_ratio + 1.0e-5
            };
            if !improves {
                continue;
            }
            // A repair is an incremental transaction, not a one-shot jump to
            // the final fence. Stay inside the trust region, preserve the
            // opposite catastrophic fence, then choose the strongest measured
            // improvement in that region. The former smallest-delta ordering
            // repeatedly selected the 0.25-fraction candidate and R31 exhausted
            // 103 full-oracle passes while the same collapse axis remained
            // blocked. The complete all-clip oracle and local backtracker still
            // decide whether this bounded local step may be retained.
            let preserves_opposite_fence = if expansion {
                ratio >= absolute_edge_min
            } else {
                ratio <= absolute_edge_max
            };
            if !preserves_opposite_fence {
                continue;
            }
            let delta = aurora_weight_row_l1_delta_v1(&rows[0], left)
                + aurora_weight_row_l1_delta_v1(&rows[1], right);
            if delta > maximum_incremental_edge_l1_delta + 1.0e-6 {
                continue;
            }
            let replace = best.as_ref().is_none_or(|(_, best_ratio, best_delta)| {
                let stronger = if expansion {
                    ratio < *best_ratio - 1.0e-6
                } else {
                    ratio > *best_ratio + 1.0e-6
                };
                stronger || ((ratio - *best_ratio).abs() <= 1.0e-6 && delta < *best_delta - 1.0e-6)
            });
            if replace {
                best = Some((candidate, ratio, delta));
            }
        }
    }
    Ok(best.map(|(rows, _, _)| rows))
}

fn cohere_catastrophic_edge_endpoints_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    violations: &MotionWeightViolationAccumulatorV1,
    expansion: bool,
    coherence_blend: f32,
    endpoint_policy: CatastrophicEndpointPolicyV1,
    target_policy: CatastrophicTargetPolicyV1,
    motion_basis_sets: Option<&[CatastrophicEdgeMotionBasisSetV1]>,
) -> Result<CatastrophicEdgeRefinementReportV1, ReferenceSupermodelMotionErrorV2> {
    let catastrophic_edge_penalties = if expansion {
        &violations.catastrophic_expansion_edge_penalties
    } else {
        &violations.catastrophic_collapse_edge_penalties
    };
    if catastrophic_edge_penalties.is_empty() {
        return Ok(CatastrophicEdgeRefinementReportV1::default());
    }
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut members_by_position = BTreeMap::<[u32; 3], Vec<(usize, usize)>>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for (vertex_index, position) in segment.positions.iter().enumerate() {
            members_by_position
                .entry(position.map(f32::to_bits))
                .or_default()
                .push((segment_index, vertex_index));
        }
    }
    let groups = members_by_position.into_values().collect::<Vec<_>>();
    let mut group_by_vertex = model
        .segments
        .iter()
        .map(|segment| vec![usize::MAX; segment.positions.len()])
        .collect::<Vec<_>>();
    for (group_index, members) in groups.iter().enumerate() {
        for &(segment_index, vertex_index) in members {
            group_by_vertex[segment_index][vertex_index] = group_index;
        }
    }
    let mut triangle_groups = Vec::<[usize; 3]>::new();
    let mut triangles_by_group = vec![Vec::<usize>::new(); groups.len()];
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for triangle in segment.indices.chunks_exact(3) {
            let mapped = [
                group_by_vertex[segment_index][triangle[0] as usize],
                group_by_vertex[segment_index][triangle[1] as usize],
                group_by_vertex[segment_index][triangle[2] as usize],
            ];
            let triangle_index = triangle_groups.len();
            triangle_groups.push(mapped);
            for group in mapped.into_iter().collect::<BTreeSet<_>>() {
                triangles_by_group[group].push(triangle_index);
            }
        }
    }
    let mut measured = catastrophic_edge_penalties
        .iter()
        .map(|((node_name, left, right), penalty)| (*penalty, node_name, *left, *right))
        .collect::<Vec<_>>();
    measured.sort_unstable_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(right.1))
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.3.cmp(&right.3))
    });
    let peak_edge = measured.first().map(|(_, node_name, left, right)| {
        format!(
            "{node_name}:{left}-{right}:{}",
            if expansion { "expansion" } else { "collapse" }
        )
    });
    // This phase is isolated from generic smoothing and every candidate is
    // fully remeasured plus locally backtracked before retention. The caller
    // line-searches this blend when the selected edge axis improves but an
    // orthogonal deformation axis regresses.
    if !coherence_blend.is_finite() || !(0.0..=1.0).contains(&coherence_blend) {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-CATASTROPHIC-EDGE-BLEND",
            "motionWeightRefinement.catastrophicEdgeBlend",
            "catastrophic edge coherence blend must be finite and inside [0, 1]",
        ));
    }
    const MAXIMUM_CATASTROPHIC_ADJUSTMENTS_PER_GROUP: usize = 2;
    let mut adjustment_count_by_group = vec![0usize; groups.len()];
    let mut coherent = 0usize;
    let mut topology_skip_count = 0usize;
    let mut conflict_skip_count = 0usize;
    let mut peak_edge_changed = false;
    // A measured-basis transaction may repair a small deterministic batch of
    // disjoint peaks. Every endpoint move is incremental and the caller still
    // remeasures every clip before retention, so this reduces expensive oracle
    // passes without coupling shared vertices. The heuristic fallback targets
    // remain one-edge transactions.
    let measured_edge_limit = if target_policy == CatastrophicTargetPolicyV1::MeasuredMotionBasis {
        8
    } else {
        1
    };
    const MAXIMUM_MEASURED_EDGE_CHANGES_PER_TRANSACTION: usize = 4;
    for (measured_index, (_, node_name, left_vertex, right_vertex)) in
        measured.into_iter().take(measured_edge_limit).enumerate()
    {
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-CATASTROPHIC-EDGE-NODE-NAME",
                    "motionQuality.weightViolations.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-CATASTROPHIC-EDGE-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        if left_vertex >= group_by_vertex[segment_index].len()
            || right_vertex >= group_by_vertex[segment_index].len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CATASTROPHIC-EDGE-VERTEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                "catastrophic edge endpoint exceeds the segment vertex count",
            ));
        }
        let edge_groups = [
            group_by_vertex[segment_index][left_vertex],
            group_by_vertex[segment_index][right_vertex],
        ];
        if edge_groups[0] == edge_groups[1] {
            conflict_skip_count += 1;
            continue;
        }
        if target_policy == CatastrophicTargetPolicyV1::MeasuredMotionBasis
            && edge_groups
                .iter()
                .any(|group| adjustment_count_by_group[*group] > 0)
        {
            conflict_skip_count += 1;
            continue;
        }
        let rows = edge_groups.map(|group| {
            let (member_segment, member_vertex) = groups[group][0];
            model.segments[member_segment].weights[member_vertex].clone()
        });
        let support = rows
            .iter()
            .flat_map(aurora_weight_support_v1)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let dominant_depth = |row: &AuroraVertexWeightsV1| {
            row.bone_node_ids
                .iter()
                .zip(row.values)
                .filter_map(|(bone, value)| bone.map(|bone| (bone, value)))
                .filter(|(_, value)| value.is_finite() && *value > 0.0)
                .max_by(|left, right| left.1.total_cmp(&right.1).then(left.0.cmp(&right.0)))
                .map_or(0, |(mut bone, _)| {
                    let mut depth = 0usize;
                    while let Some(parent) = contract.nodes[bone as usize].parent_part_number {
                        bone = parent;
                        depth += 1;
                    }
                    depth
                })
        };
        let deeper_lane = if dominant_depth(&rows[0]) >= dominant_depth(&rows[1]) {
            0
        } else {
            1
        };
        let shallower_lane = 1 - deeper_lane;
        let replacement_rows = if target_policy == CatastrophicTargetPolicyV1::MeasuredMotionBasis {
            let Some(bases) = motion_basis_sets
                .and_then(|sets| {
                    sets.iter().find(|set| {
                        set.node_name == **node_name && set.vertices == [left_vertex, right_vertex]
                    })
                })
                .map(|set| set.bases.as_slice())
            else {
                continue;
            };
            let frozen_lane = match endpoint_policy {
                CatastrophicEndpointPolicyV1::Symmetric => None,
                CatastrophicEndpointPolicyV1::DeeperOnly => Some(shallower_lane),
                CatastrophicEndpointPolicyV1::ShallowerOnly => Some(deeper_lane),
            };
            let strength = (coherence_blend * 10.0).clamp(0.0, 1.0);
            let Some(solved) = solve_catastrophic_edge_from_motion_basis_v1(
                &rows,
                bases,
                expansion,
                strength,
                frozen_lane,
                contract.tolerances.edge_hard_min_ratio.powi(2),
                contract.tolerances.edge_hard_max_ratio.powi(2),
            )?
            else {
                continue;
            };
            solved
        } else {
            let mut average = BTreeMap::<u32, f32>::new();
            for row in &rows {
                accumulate_aurora_weight_row_v1(row, 0.5, &mut average);
            }
            if target_policy == CatastrophicTargetPolicyV1::ProjectedCarrierEdge
                || !support_is_one_carrier_lineage_v1(&support, contract)
            {
                let scoring_rows = [rows[0].clone(), rows[1].clone(), rows[0].clone()];
                let allowed = best_one_edge_triangle_support_v1(&scoring_rows, &support, contract)?;
                average.retain(|bone, _| allowed.contains(bone));
            }
            if average.is_empty() {
                continue;
            }
            let target = normalize_aurora_weight_row_v1(average)?;
            [
                blend_aurora_weight_rows_v1(&rows[0], &target, coherence_blend)?,
                blend_aurora_weight_rows_v1(&rows[1], &target, coherence_blend)?,
            ]
        };
        let replacements = edge_groups
            .into_iter()
            .zip(replacement_rows)
            .collect::<Vec<_>>();
        // A duplicate-position seam group on the parent side can also belong
        // to a grandparent-facing triangle. Updating both edge endpoints would
        // then temporarily span two carrier edges and be rejected. Prefer the
        // symmetric update, but fall back to the deeper endpoint so the child
        // side gains a parent blend without leaking into that grandparent
        // triangle. The caller validates the resulting full motion report
        // transactionally before accepting the round.
        let options = match endpoint_policy {
            CatastrophicEndpointPolicyV1::Symmetric => vec![replacements.clone()],
            CatastrophicEndpointPolicyV1::DeeperOnly => {
                vec![vec![replacements[deeper_lane].clone()]]
            }
            CatastrophicEndpointPolicyV1::ShallowerOnly => {
                vec![vec![replacements[shallower_lane].clone()]]
            }
        };
        let has_capacity_option = options.iter().any(|option| {
            option.iter().all(|(group, _)| {
                adjustment_count_by_group[*group] < MAXIMUM_CATASTROPHIC_ADJUSTMENTS_PER_GROUP
            })
        });
        let selected = options.into_iter().find(|option| {
            if option.iter().any(|(group, _)| {
                adjustment_count_by_group[*group] >= MAXIMUM_CATASTROPHIC_ADJUSTMENTS_PER_GROUP
            }) {
                return false;
            }
            let affected_triangles = option
                .iter()
                .flat_map(|(group, _)| triangles_by_group[*group].iter().copied())
                .collect::<BTreeSet<_>>();
            affected_triangles.into_iter().all(|triangle_index| {
                let existing_support = triangle_groups[triangle_index]
                    .iter()
                    .flat_map(|group| {
                        let (segment_index, vertex_index) = groups[*group][0];
                        aurora_weight_support_v1(
                            &model.segments[segment_index].weights[vertex_index],
                        )
                    })
                    .collect::<BTreeSet<_>>();
                let candidate_support = triangle_groups[triangle_index]
                    .iter()
                    .flat_map(|group| {
                        let row = option
                            .iter()
                            .find_map(|(candidate_group, replacement)| {
                                (*candidate_group == *group).then_some(replacement)
                            })
                            .cloned()
                            .unwrap_or_else(|| {
                                let (segment_index, vertex_index) = groups[*group][0];
                                model.segments[segment_index].weights[vertex_index].clone()
                            });
                        aurora_weight_support_v1(&row)
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                support_is_one_carrier_edge_v1(&candidate_support, contract)
                    || (candidate_support
                        .iter()
                        .all(|bone| existing_support.contains(bone))
                        && support_is_one_carrier_lineage_v1(
                            &existing_support.iter().copied().collect::<Vec<_>>(),
                            contract,
                        )
                        && support_is_one_carrier_lineage_v1(&candidate_support, contract))
            })
        });
        let Some(replacements) = selected else {
            if has_capacity_option {
                topology_skip_count += 1;
            } else {
                conflict_skip_count += 1;
            }
            continue;
        };
        let mut changed = false;
        for (group, replacement) in replacements {
            for &(target_segment, target_vertex) in &groups[group] {
                changed |= model.segments[target_segment].weights[target_vertex] != replacement;
                model.segments[target_segment].weights[target_vertex] = replacement.clone();
            }
            adjustment_count_by_group[group] += 1;
        }
        coherent += usize::from(changed);
        if measured_index == 0 {
            peak_edge_changed = changed;
        }
        if target_policy == CatastrophicTargetPolicyV1::MeasuredMotionBasis
            && coherent >= MAXIMUM_MEASURED_EDGE_CHANGES_PER_TRANSACTION
        {
            break;
        }
    }
    Ok(CatastrophicEdgeRefinementReportV1 {
        candidate_count: catastrophic_edge_penalties.len(),
        changed_count: coherent,
        topology_skip_count,
        conflict_skip_count,
        peak_edge,
        peak_edge_changed,
    })
}

fn cohere_measured_triangle_weights_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    violations: &MotionWeightViolationAccumulatorV1,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    if violations.triangle_penalties.is_empty() {
        return Ok(0);
    }
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut members_by_position = BTreeMap::<[u32; 3], Vec<(usize, usize)>>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for (vertex_index, position) in segment.positions.iter().enumerate() {
            members_by_position
                .entry(position.map(f32::to_bits))
                .or_default()
                .push((segment_index, vertex_index));
        }
    }
    let groups = members_by_position.into_values().collect::<Vec<_>>();
    let mut group_by_vertex = model
        .segments
        .iter()
        .map(|segment| vec![usize::MAX; segment.positions.len()])
        .collect::<Vec<_>>();
    for (group_index, members) in groups.iter().enumerate() {
        for &(segment_index, vertex_index) in members {
            group_by_vertex[segment_index][vertex_index] = group_index;
        }
    }
    let mut measured = Vec::<(u64, usize, [usize; 3])>::new();
    for ((node_name, vertices), penalty) in &violations.triangle_penalties {
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-COHERENCE-NODE-NAME",
                    "motionQuality.weightViolations.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-COHERENCE-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        if vertices
            .iter()
            .any(|vertex| *vertex >= group_by_vertex[segment_index].len())
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-COHERENCE-VERTEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                format!(
                    "measured triangle {vertices:?} exceeds segment vertex count {}",
                    group_by_vertex[segment_index].len()
                ),
            ));
        }
        measured.push((*penalty, segment_index, *vertices));
    }
    measured.sort_unstable_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    let mut coherent = BTreeSet::<(usize, [usize; 3])>::new();
    const MEASURED_TRIANGLE_COHERENCE_SWEEPS: usize = 4;
    for _ in 0..MEASURED_TRIANGLE_COHERENCE_SWEEPS {
        for &(_, segment_index, vertices) in &measured {
            let group_indices = vertices.map(|vertex| group_by_vertex[segment_index][vertex]);
            let rows = group_indices.map(|group| {
                let (member_segment, member_vertex) = groups[group][0];
                model.segments[member_segment].weights[member_vertex].clone()
            });
            let support = rows
                .iter()
                .flat_map(aurora_weight_support_v1)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let allowed = best_one_edge_triangle_support_v1(&rows, &support, contract)?;
            let mut average = BTreeMap::<u32, f32>::new();
            for row in &rows {
                accumulate_aurora_weight_row_v1(row, 1.0 / 3.0, &mut average);
            }
            average.retain(|bone, _| allowed.contains(bone));
            if average.is_empty() {
                continue;
            }
            let replacement = normalize_aurora_weight_row_v1(average)?;
            let mut changed = false;
            for group_index in group_indices.into_iter().collect::<BTreeSet<_>>() {
                for &(target_segment, target_vertex) in &groups[group_index] {
                    changed |= model.segments[target_segment].weights[target_vertex] != replacement;
                    model.segments[target_segment].weights[target_vertex] = replacement.clone();
                }
            }
            if changed {
                coherent.insert((segment_index, vertices));
            }
        }
    }
    Ok(coherent.len())
}

fn project_conversion_triangle_support_to_one_edge_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    project_conversion_triangle_support_to_one_edge_preserving_v1(model, contract, &BTreeMap::new())
}

fn project_conversion_triangle_support_to_one_edge_preserving_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    protected_anchor_rows: &BTreeMap<[u32; 3], ProtectedMotionAnchorRowV1>,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    let mut members_by_position = BTreeMap::<[u32; 3], Vec<(usize, usize)>>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for (vertex_index, position) in segment.positions.iter().enumerate() {
            members_by_position
                .entry(position.map(f32::to_bits))
                .or_default()
                .push((segment_index, vertex_index));
        }
    }
    let groups = members_by_position.into_values().collect::<Vec<_>>();
    let mut group_by_vertex = model
        .segments
        .iter()
        .map(|segment| vec![usize::MAX; segment.positions.len()])
        .collect::<Vec<_>>();
    for (group_index, members) in groups.iter().enumerate() {
        for &(segment_index, vertex_index) in members {
            group_by_vertex[segment_index][vertex_index] = group_index;
        }
    }
    let protected_row_by_group = groups
        .iter()
        .map(|members| {
            let (segment_index, vertex_index) = members[0];
            protected_anchor_rows
                .get(&model.segments[segment_index].positions[vertex_index].map(f32::to_bits))
                .cloned()
        })
        .collect::<Vec<_>>();
    let mut triangles = Vec::<(usize, [usize; 3])>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for triangle in segment.indices.chunks_exact(3) {
            let vertices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            if vertices
                .iter()
                .any(|vertex| *vertex >= group_by_vertex[segment_index].len())
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-INDEX-OOB",
                    format!("model.segments[{segment_index}].indices"),
                    "triangle-support projection encountered an out-of-range vertex",
                ));
            }
            triangles.push((segment_index, vertices));
        }
    }
    let mut projected = BTreeSet::<(usize, [usize; 3])>::new();
    const MAXIMUM_PROJECTION_SWEEPS: usize = 64;
    for _ in 0..MAXIMUM_PROJECTION_SWEEPS {
        let mut changed = false;
        for &(segment_index, vertices) in &triangles {
            let group_indices = vertices.map(|vertex| group_by_vertex[segment_index][vertex]);
            let rows = group_indices.map(|group| {
                let (member_segment, member_vertex) = groups[group][0];
                model.segments[member_segment].weights[member_vertex].clone()
            });
            let support = rows
                .iter()
                .flat_map(aurora_weight_support_v1)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            if support_is_one_carrier_edge_v1(&support, contract) {
                continue;
            }
            let required_support = group_indices
                .iter()
                .filter_map(|group| protected_row_by_group[*group].as_ref())
                .flat_map(|protected| protected.required_bones.iter().copied())
                .collect::<BTreeSet<_>>();
            let allowed = best_one_edge_triangle_support_preserving_v1(
                &rows,
                &support,
                &required_support,
                contract,
            )?;
            for group_index in group_indices {
                let (member_segment, member_vertex) = groups[group_index][0];
                let current = model.segments[member_segment].weights[member_vertex].clone();
                let mut retained = BTreeMap::<u32, f32>::new();
                accumulate_aurora_weight_row_v1(&current, 1.0, &mut retained);
                retained.retain(|bone, _| allowed.contains(bone));
                let replacement = if retained.is_empty() {
                    let dominant = aurora_weight_support_v1(&current)
                        .into_iter()
                        .next()
                        .ok_or_else(|| {
                            motion_error(
                                "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-EMPTY",
                                "model.segments.weights",
                                "triangle-support projection found an empty source row",
                            )
                        })?;
                    let bone = allowed
                        .iter()
                        .copied()
                        .min_by(|left, right| {
                            carrier_topology_distance_for_motion_v1(dominant, *left, contract)
                                .cmp(&carrier_topology_distance_for_motion_v1(
                                    dominant, *right, contract,
                                ))
                                .then(left.cmp(right))
                        })
                        .expect("one-edge support candidate is non-empty");
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(bone), None, None, None],
                        values: [1.0, 0.0, 0.0, 0.0],
                        influence_count: 1,
                    }
                } else {
                    normalize_aurora_weight_row_v1(retained)?
                };
                if let Some(protected) = &protected_row_by_group[group_index] {
                    ensure_required_anchor_influences_v1(&replacement, protected)?;
                }
                for &(target_segment, target_vertex) in &groups[group_index] {
                    changed |= model.segments[target_segment].weights[target_vertex] != replacement;
                    model.segments[target_segment].weights[target_vertex] = replacement.clone();
                }
            }
            projected.insert((segment_index, vertices));
        }
        if !changed {
            break;
        }
    }
    let mut remaining = triangles
        .iter()
        .filter(|(segment_index, vertices)| {
            let support = vertices
                .iter()
                .flat_map(|vertex| {
                    aurora_weight_support_v1(&model.segments[*segment_index].weights[*vertex])
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            !support_is_one_carrier_edge_v1(&support, contract)
        })
        .count();
    if remaining > 0 {
        // Sequential one-edge choices can oscillate where several triangles
        // share an exact-position group. Finish with a monotonic work queue:
        // every still-invalid triangle moves only upward to the lowest common
        // carrier ancestor of its current support. Carrier depth is finite, so
        // the repair converges without inventing a species-specific mapping.
        let triangle_groups = triangles
            .iter()
            .map(|(segment_index, vertices)| {
                vertices.map(|vertex| group_by_vertex[*segment_index][vertex])
            })
            .collect::<Vec<_>>();
        let mut triangles_by_group = vec![Vec::<usize>::new(); groups.len()];
        for (triangle_index, triangle) in triangle_groups.iter().enumerate() {
            for group in triangle.into_iter().collect::<BTreeSet<_>>() {
                triangles_by_group[*group].push(triangle_index);
            }
        }
        let mut queued = vec![false; triangles.len()];
        let mut queue = VecDeque::new();
        for triangle_index in 0..triangles.len() {
            let support = triangle_groups[triangle_index]
                .iter()
                .flat_map(|group| {
                    let (segment_index, vertex_index) = groups[*group][0];
                    aurora_weight_support_v1(&model.segments[segment_index].weights[vertex_index])
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            if !support_is_one_carrier_edge_v1(&support, contract) {
                queued[triangle_index] = true;
                queue.push_back(triangle_index);
            }
        }
        let maximum_repairs = groups.len().saturating_mul(contract.nodes.len().max(1));
        let mut repair_count = 0usize;
        while let Some(triangle_index) = queue.pop_front() {
            queued[triangle_index] = false;
            let support = triangle_groups[triangle_index]
                .iter()
                .flat_map(|group| {
                    let (segment_index, vertex_index) = groups[*group][0];
                    aurora_weight_support_v1(&model.segments[segment_index].weights[vertex_index])
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            if support_is_one_carrier_edge_v1(&support, contract) {
                continue;
            }
            let group_indices = triangle_groups[triangle_index];
            let rows = group_indices.map(|group| {
                let (segment_index, vertex_index) = groups[group][0];
                model.segments[segment_index].weights[vertex_index].clone()
            });
            let required_support = group_indices
                .iter()
                .filter_map(|group| protected_row_by_group[*group].as_ref())
                .flat_map(|protected| protected.required_bones.iter().copied())
                .collect::<BTreeSet<_>>();
            let allowed = if required_support.is_empty() {
                let ancestor =
                    lowest_common_carrier_ancestor_v1(&support, contract).ok_or_else(|| {
                        motion_error(
                            "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-ANCESTOR-MISSING",
                            "model.segments.weights",
                            "nonconvergent triangle support has no common carrier ancestor",
                        )
                    })?;
                BTreeSet::from([ancestor])
            } else {
                best_one_edge_triangle_support_preserving_v1(
                    &rows,
                    &support,
                    &required_support,
                    contract,
                )?
            };
            let mut changed_groups = BTreeSet::new();
            for group in group_indices {
                let (segment_index, vertex_index) = groups[group][0];
                let current = model.segments[segment_index].weights[vertex_index].clone();
                let replacement =
                    project_aurora_weight_row_to_support_v1(&current, &allowed, contract)?;
                if let Some(protected) = &protected_row_by_group[group] {
                    ensure_required_anchor_influences_v1(&replacement, protected)?;
                }
                if current == replacement {
                    continue;
                }
                for &(target_segment, target_vertex) in &groups[group] {
                    model.segments[target_segment].weights[target_vertex] = replacement.clone();
                }
                changed_groups.insert(group);
            }
            if changed_groups.is_empty() {
                continue;
            }
            projected.insert(triangles[triangle_index]);
            repair_count += 1;
            if repair_count > maximum_repairs {
                break;
            }
            for group in changed_groups {
                for &neighbour_triangle in &triangles_by_group[group] {
                    if !queued[neighbour_triangle] {
                        queued[neighbour_triangle] = true;
                        queue.push_back(neighbour_triangle);
                    }
                }
            }
        }
        remaining = triangles
            .iter()
            .filter(|(segment_index, vertices)| {
                let support = vertices
                    .iter()
                    .flat_map(|vertex| {
                        aurora_weight_support_v1(&model.segments[*segment_index].weights[*vertex])
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                !support_is_one_carrier_edge_v1(&support, contract)
            })
            .count();
    }
    if remaining > 0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-NONCONVERGENT",
            "model.segments.weights",
            format!(
                "triangle-support projection left {remaining} triangles spanning more than one carrier edge"
            ),
        ));
    }
    Ok(projected.len())
}

fn lowest_common_carrier_ancestor_v1(
    support: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Option<u32> {
    let first = *support.first()?;
    let mut candidates = Vec::new();
    let mut current = Some(first);
    while let Some(bone) = current {
        candidates.push(bone);
        current = contract
            .nodes
            .get(bone as usize)
            .and_then(|node| node.parent_part_number);
    }
    candidates.into_iter().find(|candidate| {
        support.iter().all(|bone| {
            let mut current = Some(*bone);
            while let Some(ancestor) = current {
                if ancestor == *candidate {
                    return true;
                }
                current = contract
                    .nodes
                    .get(ancestor as usize)
                    .and_then(|node| node.parent_part_number);
            }
            false
        })
    })
}

fn cohere_micro_triangles_with_common_support_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    violations: &MotionWeightViolationAccumulatorV1,
) -> Result<MicroTriangleRefinementReportV1, ReferenceSupermodelMotionErrorV2> {
    if violations.triangle_penalties.is_empty() {
        return Ok(MicroTriangleRefinementReportV1::default());
    }
    let mut bounds_min = [f32::INFINITY; 3];
    let mut bounds_max = [f32::NEG_INFINITY; 3];
    let mut members_by_position = BTreeMap::<[u32; 3], Vec<(usize, usize)>>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for (vertex_index, position) in segment.positions.iter().enumerate() {
            for axis in 0..3 {
                bounds_min[axis] = bounds_min[axis].min(position[axis]);
                bounds_max[axis] = bounds_max[axis].max(position[axis]);
            }
            members_by_position
                .entry(position.map(f32::to_bits))
                .or_default()
                .push((segment_index, vertex_index));
        }
    }
    let global_diagonal = distance(bounds_min, bounds_max);
    if !global_diagonal.is_finite() || global_diagonal <= 1.0e-8 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-MICRO-TRIANGLE-BOUNDS",
            "model.segments.positions",
            "micro-triangle stabilization requires finite non-degenerate model bounds",
        ));
    }
    let groups = members_by_position.into_values().collect::<Vec<_>>();
    let mut group_by_vertex = model
        .segments
        .iter()
        .map(|segment| vec![usize::MAX; segment.positions.len()])
        .collect::<Vec<_>>();
    for (group_index, members) in groups.iter().enumerate() {
        for &(segment_index, vertex_index) in members {
            group_by_vertex[segment_index][vertex_index] = group_index;
        }
    }
    let frozen_terminal_anchor_bones = contract
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL"
                || node.structural_role == "APPENDAGE_TERMINAL"
                || node.anchor_role.as_deref().is_some_and(|role| {
                    let role = role.to_ascii_lowercase();
                    role.contains("paw")
                        || role.starts_with("tail_")
                        || role.starts_with("appendage_")
                })
        })
        .map(|(bone, _)| bone as u32)
        .collect::<BTreeSet<_>>();
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut triangles = Vec::<(usize, [usize; 3])>::new();
    for (node_name, vertices) in violations
        .triangle_penalties
        .keys()
        .map(|(node_name, vertices)| (node_name, *vertices))
    {
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-MICRO-TRIANGLE-NODE-NAME",
                    "motionQuality.weightViolations.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-MICRO-TRIANGLE-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        if vertices
            .iter()
            .any(|vertex| *vertex >= group_by_vertex[segment_index].len())
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-MICRO-TRIANGLE-INDEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                "measured micro-triangle stabilization encountered an out-of-range vertex",
            ));
        }
        triangles.push((segment_index, vertices));
    }
    triangles.sort_unstable();
    triangles.dedup();
    const MICRO_TRIANGLE_MAX_EDGE_DIAGONAL_FRACTION: f32 = 0.003;
    const MICRO_TRIANGLE_RIGID_BLEND: f32 = 0.25;
    let mut report = MicroTriangleRefinementReportV1 {
        candidate_count: triangles.len(),
        ..MicroTriangleRefinementReportV1::default()
    };
    let mut cohered = BTreeSet::<(usize, [usize; 3])>::new();
    let mut adjusted_groups = BTreeSet::<usize>::new();
    for &(segment_index, vertices) in &triangles {
        let positions = vertices.map(|vertex| model.segments[segment_index].positions[vertex]);
        let maximum_edge_length = [(0, 1), (1, 2), (2, 0)]
            .into_iter()
            .map(|(left, right)| distance(positions[left], positions[right]))
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);
        if maximum_edge_length > global_diagonal * MICRO_TRIANGLE_MAX_EDGE_DIAGONAL_FRACTION {
            report.too_large_count += 1;
            continue;
        }
        let group_indices = vertices.map(|vertex| group_by_vertex[segment_index][vertex]);
        let rows = group_indices.map(|group| {
            let (member_segment, member_vertex) = groups[group][0];
            model.segments[member_segment].weights[member_vertex].clone()
        });
        let support = rows
            .iter()
            .flat_map(aurora_weight_support_v1)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if support
            .iter()
            .any(|bone| frozen_terminal_anchor_bones.contains(bone))
        {
            report.protected_support_count += 1;
            continue;
        }
        if !support_is_one_carrier_edge_v1(&support, contract) {
            report.nonlocal_support_count += 1;
            continue;
        }
        let common = rows
            .iter()
            .map(|row| {
                aurora_weight_support_v1(row)
                    .into_iter()
                    .collect::<BTreeSet<_>>()
            })
            .reduce(|left, right| left.intersection(&right).copied().collect())
            .unwrap_or_default();
        let Some(bone) = common.into_iter().max_by(|left, right| {
            let total = |bone| {
                rows.iter()
                    .map(|row| aurora_weight_for_bone_v1(row, bone))
                    .sum::<f32>()
            };
            total(*left)
                .total_cmp(&total(*right))
                .then_with(|| right.cmp(left))
        }) else {
            report.no_common_support_count += 1;
            continue;
        };
        // Apply one bounded step toward the strongest already-common
        // carrier. A full rigid collapse repaired the measured triangle but
        // could create a worse discontinuity on an unmeasured neighbor. The
        // transactional outer rounds repeat this step only when the complete
        // motion oracle confirms monotonic improvement.
        let target_groups = group_indices.into_iter().collect::<BTreeSet<_>>();
        let mut triangle_changed = false;
        for group_index in target_groups {
            if !adjusted_groups.insert(group_index) {
                continue;
            }
            let (member_segment, member_vertex) = groups[group_index][0];
            let current = &model.segments[member_segment].weights[member_vertex];
            let replacement =
                blend_aurora_weight_row_toward_bone_v1(current, bone, MICRO_TRIANGLE_RIGID_BLEND)?;
            for &(target_segment, target_vertex) in &groups[group_index] {
                triangle_changed |=
                    model.segments[target_segment].weights[target_vertex] != replacement;
                model.segments[target_segment].weights[target_vertex] = replacement.clone();
            }
        }
        if triangle_changed {
            cohered.insert((segment_index, vertices));
        }
    }
    report.changed_count = cohered.len();
    Ok(report)
}

fn cohere_catastrophic_triangle_toward_common_carrier_v1(
    model: &mut AuroraModelIrV1,
    violations: &MotionWeightViolationAccumulatorV1,
    expansion: bool,
    blend: f32,
) -> Result<MicroTriangleRefinementReportV1, ReferenceSupermodelMotionErrorV2> {
    let penalties = if expansion {
        &violations.catastrophic_expansion_triangle_penalties
    } else {
        &violations.catastrophic_collapse_triangle_penalties
    };
    if penalties.is_empty() {
        return Ok(MicroTriangleRefinementReportV1::default());
    }
    let groups = build_motion_weight_duplicate_groups_v2(model)?;
    let mut group_by_vertex = model
        .segments
        .iter()
        .map(|segment| vec![usize::MAX; segment.positions.len()])
        .collect::<Vec<_>>();
    for (group_index, members) in groups.iter().enumerate() {
        for &(segment_index, vertex_index) in members {
            group_by_vertex[segment_index][vertex_index] = group_index;
        }
    }
    let segment_index_by_id = model
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (segment.segment_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut measured = penalties
        .iter()
        .map(|((node_name, vertices), penalty)| (*penalty, node_name, *vertices))
        .collect::<Vec<_>>();
    measured.sort_unstable_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    let mut report = MicroTriangleRefinementReportV1 {
        candidate_count: measured.len(),
        ..MicroTriangleRefinementReportV1::default()
    };
    // One measured triangle per transaction keeps the full motion oracle an
    // exact line search instead of mixing unrelated local repairs.
    for &(_, node_name, vertices) in &measured {
        let segment_id = node_name
            .strip_prefix("m2a_seg_")
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-CATASTROPHIC-TRIANGLE-NODE-NAME",
                    "motionQuality.weightViolations.nodeName",
                    format!("renderer skin node {node_name:?} is not an owned segment node"),
                )
            })?;
        let segment_index = *segment_index_by_id.get(&segment_id).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-CATASTROPHIC-TRIANGLE-SEGMENT-MISSING",
                "model.segments",
                format!("renderer skin node {node_name:?} has no conversion segment"),
            )
        })?;
        if vertices
            .iter()
            .any(|vertex| *vertex >= group_by_vertex[segment_index].len())
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CATASTROPHIC-TRIANGLE-INDEX-OOB",
                format!("model.segments[{segment_index}].weights"),
                "measured catastrophic triangle contains an out-of-range vertex",
            ));
        }
        let group_indices = vertices.map(|vertex| group_by_vertex[segment_index][vertex]);
        let rows = group_indices.map(|group| {
            let (member_segment, member_vertex) = groups[group][0];
            model.segments[member_segment].weights[member_vertex].clone()
        });
        let common = rows
            .iter()
            .map(|row| {
                aurora_weight_support_v1(row)
                    .into_iter()
                    .collect::<BTreeSet<_>>()
            })
            .reduce(|left, right| left.intersection(&right).copied().collect())
            .unwrap_or_default();
        let Some(carrier) = common.into_iter().max_by(|left, right| {
            let total = |bone| {
                rows.iter()
                    .map(|row| aurora_weight_for_bone_v1(row, bone))
                    .sum::<f32>()
            };
            total(*left)
                .total_cmp(&total(*right))
                .then_with(|| right.cmp(left))
        }) else {
            report.no_common_support_count += 1;
            continue;
        };
        let mut changed = false;
        for group in group_indices.into_iter().collect::<BTreeSet<_>>() {
            let (member_segment, member_vertex) = groups[group][0];
            let replacement = blend_aurora_weight_row_toward_bone_v1(
                &model.segments[member_segment].weights[member_vertex],
                carrier,
                blend,
            )?;
            for &(target_segment, target_vertex) in &groups[group] {
                changed |= model.segments[target_segment].weights[target_vertex] != replacement;
                model.segments[target_segment].weights[target_vertex] = replacement.clone();
            }
        }
        if changed {
            report.changed_count = 1;
            break;
        }
    }
    Ok(report)
}

fn blend_aurora_weight_row_toward_bone_v1(
    current: &AuroraVertexWeightsV1,
    bone: u32,
    blend: f32,
) -> Result<AuroraVertexWeightsV1, ReferenceSupermodelMotionErrorV2> {
    if !blend.is_finite() || !(0.0..=1.0).contains(&blend) {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-BLEND-INVALID",
            "model.segments.weights",
            "motion weight blend must be finite and within [0,1]",
        ));
    }
    let mut accumulated = BTreeMap::<u32, f32>::new();
    accumulate_aurora_weight_row_v1(current, 1.0 - blend, &mut accumulated);
    *accumulated.entry(bone).or_default() += blend;
    let maximum = accumulated.values().copied().fold(0.0_f32, f32::max);
    // The chosen carrier is common to every corner, so the blend must preserve
    // every existing positive support rather than pruning a joint cluster.
    let mut rows = accumulated
        .into_iter()
        .filter(|(_, value)| value.is_finite() && *value > maximum * 1.0e-7)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    if rows.is_empty() || rows.len() > 4 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-BLEND-LAYOUT",
            "model.segments.weights",
            "bounded motion weight blend produced an invalid influence layout",
        ));
    }
    let total = rows.iter().map(|(_, value)| *value).sum::<f32>();
    let mut output = AuroraVertexWeightsV1 {
        bone_node_ids: [None; 4],
        values: [0.0; 4],
        influence_count: rows.len() as u8,
    };
    for (lane, (row_bone, value)) in rows.into_iter().enumerate() {
        output.bone_node_ids[lane] = Some(row_bone);
        output.values[lane] = value / total;
    }
    Ok(output)
}

fn maximum_triangle_weight_delta_v1(rows: &[AuroraVertexWeightsV1; 3], support: &[u32]) -> f32 {
    support
        .iter()
        .copied()
        .flat_map(|bone| {
            [(0, 1), (1, 2), (2, 0)]
                .into_iter()
                .map(move |(left, right)| {
                    (aurora_weight_for_bone_v1(&rows[left], bone)
                        - aurora_weight_for_bone_v1(&rows[right], bone))
                    .abs()
                })
        })
        .max_by(f32::total_cmp)
        .unwrap_or(0.0)
}

fn aurora_weight_for_bone_v1(row: &AuroraVertexWeightsV1, bone: u32) -> f32 {
    (0..usize::from(row.influence_count.min(4)))
        .find_map(|lane| (row.bone_node_ids[lane] == Some(bone)).then_some(row.values[lane]))
        .unwrap_or(0.0)
}

fn best_one_edge_triangle_support_v1(
    rows: &[AuroraVertexWeightsV1; 3],
    support: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<BTreeSet<u32>, ReferenceSupermodelMotionErrorV2> {
    best_one_edge_triangle_support_preserving_v1(rows, support, &BTreeSet::new(), contract)
}

fn best_one_edge_triangle_support_preserving_v1(
    rows: &[AuroraVertexWeightsV1; 3],
    support: &[u32],
    required_support: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<BTreeSet<u32>, ReferenceSupermodelMotionErrorV2> {
    let mut candidates = support
        .iter()
        .copied()
        .map(|bone| BTreeSet::from([bone]))
        .collect::<Vec<_>>();
    for (index, left) in support.iter().copied().enumerate() {
        for right in support.iter().copied().skip(index + 1) {
            if support_is_one_carrier_edge_v1(&[left, right], contract) {
                candidates.push(BTreeSet::from([left, right]));
            }
        }
    }
    for &bone in required_support {
        candidates.push(BTreeSet::from([bone]));
        if let Some(parent) = contract
            .nodes
            .get(bone as usize)
            .and_then(|node| node.parent_part_number)
        {
            candidates.push(BTreeSet::from([parent, bone]));
        }
        for child in contract
            .nodes
            .iter()
            .filter(|node| node.parent_part_number == Some(bone))
        {
            candidates.push(BTreeSet::from([bone, child.part_number]));
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|candidate| required_support.is_subset(candidate))
        .filter(|candidate| {
            support_is_one_carrier_edge_v1(
                &candidate.iter().copied().collect::<Vec<_>>(),
                contract,
            )
        })
        .max_by(|left, right| {
            let score = |candidate: &BTreeSet<u32>| {
                rows.iter()
                    .flat_map(|row| {
                        (0..usize::from(row.influence_count.min(4))).filter_map(move |lane| {
                            let bone = row.bone_node_ids[lane]?;
                            candidate.contains(&bone).then_some(row.values[lane])
                        })
                    })
                    .sum::<f32>()
            };
            score(left)
                .total_cmp(&score(right))
                .then_with(|| left.len().cmp(&right.len()))
                .then_with(|| right.iter().cmp(left.iter()))
        })
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-CANDIDATE-MISSING",
                "model.segments.weights",
                format!(
                    "triangle-support projection found no local carrier candidate preserving required support {required_support:?}"
                ),
            )
        })
}

fn project_aurora_weight_row_to_support_v1(
    current: &AuroraVertexWeightsV1,
    allowed: &BTreeSet<u32>,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<AuroraVertexWeightsV1, ReferenceSupermodelMotionErrorV2> {
    let mut retained = BTreeMap::<u32, f32>::new();
    accumulate_aurora_weight_row_v1(current, 1.0, &mut retained);
    retained.retain(|bone, _| allowed.contains(bone));
    if !retained.is_empty() {
        return normalize_aurora_weight_row_v1(retained);
    }
    let dominant = aurora_weight_support_v1(current)
        .into_iter()
        .next()
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-EMPTY",
                "model.segments.weights",
                "triangle-support projection found an empty source row",
            )
        })?;
    let bone = allowed
        .iter()
        .copied()
        .min_by(|left, right| {
            carrier_topology_distance_for_motion_v1(dominant, *left, contract)
                .cmp(&carrier_topology_distance_for_motion_v1(
                    dominant, *right, contract,
                ))
                .then(left.cmp(right))
        })
        .ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-CANDIDATE-MISSING",
                "model.segments.weights",
                "triangle-support projection received an empty allowed support",
            )
        })?;
    Ok(AuroraVertexWeightsV1 {
        bone_node_ids: [Some(bone), None, None, None],
        values: [1.0, 0.0, 0.0, 0.0],
        influence_count: 1,
    })
}

fn ensure_required_anchor_influences_v1(
    row: &AuroraVertexWeightsV1,
    protected: &ProtectedMotionAnchorRowV1,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    const REQUIRED_ANCHOR_INFLUENCE: f32 = 0.1;
    let missing = protected
        .required_bones
        .iter()
        .copied()
        .filter(|bone| aurora_weight_for_bone_v1(row, *bone) < REQUIRED_ANCHOR_INFLUENCE)
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-PROJECTION-ANCHOR-LOSS",
            "model.segments.weights",
            format!(
                "triangle-support projection would reduce required joint anchors {missing:?} below {REQUIRED_ANCHOR_INFLUENCE}"
            ),
        ))
    }
}

fn aurora_weight_support_v1(row: &AuroraVertexWeightsV1) -> Vec<u32> {
    (0..usize::from(row.influence_count.min(4)))
        .filter_map(|lane| {
            (row.values[lane].is_finite() && row.values[lane] > 0.0)
                .then_some(row.bone_node_ids[lane])
                .flatten()
        })
        .collect()
}

fn carrier_topology_distance_for_motion_v1(
    left: u32,
    right: u32,
    contract: &ReferenceSupermodelMotionContractV2,
) -> usize {
    let mut ancestors = BTreeMap::<u32, usize>::new();
    let mut current = left;
    let mut distance = 0usize;
    loop {
        ancestors.insert(current, distance);
        let Some(parent) = contract
            .nodes
            .get(current as usize)
            .and_then(|node| node.parent_part_number)
        else {
            break;
        };
        current = parent;
        distance += 1;
    }
    let mut current = right;
    let mut right_distance = 0usize;
    loop {
        if let Some(left_distance) = ancestors.get(&current) {
            return left_distance + right_distance;
        }
        let Some(parent) = contract
            .nodes
            .get(current as usize)
            .and_then(|node| node.parent_part_number)
        else {
            return usize::MAX;
        };
        current = parent;
        right_distance += 1;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RigidComponentRefinementReportV1 {
    count: usize,
    vertices: BTreeSet<(usize, usize)>,
}

fn rigidify_measured_isolated_components_v1(
    model: &mut AuroraModelIrV1,
    contract: &ReferenceSupermodelMotionContractV2,
    duplicate_groups: &[Vec<(usize, usize)>],
    group_by_vertex: &[Vec<usize>],
    measured_edges: &BTreeMap<(usize, usize), u64>,
) -> Result<RigidComponentRefinementReportV1, ReferenceSupermodelMotionErrorV2> {
    // Topological components start from the actual indexed render streams.
    // The caller's duplicate groups contain only same-position vertices that
    // also share one-ring topology, so they can reconnect real writer/UV
    // boundaries without merging coincident independent fur layers.
    let mut vertex_offsets = Vec::with_capacity(model.segments.len());
    let mut vertex_total = 0usize;
    for segment in &model.segments {
        vertex_offsets.push(vertex_total);
        vertex_total += segment.positions.len();
    }
    let mut parents = (0..vertex_total).collect::<Vec<_>>();
    let mut ranks = vec![0_u8; vertex_total];
    let mut referenced_vertices = vec![false; vertex_total];
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for triangle in segment.indices.chunks_exact(3) {
            let vertices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            if vertices
                .iter()
                .any(|vertex| *vertex >= group_by_vertex[segment_index].len())
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-INDEX-OOB",
                    format!("model.segments[{segment_index}].indices"),
                    "component rigidification encountered an out-of-range triangle index",
                ));
            }
            union_weight_groups_v1(
                &mut parents,
                &mut ranks,
                vertex_offsets[segment_index] + vertices[0],
                vertex_offsets[segment_index] + vertices[1],
            );
            union_weight_groups_v1(
                &mut parents,
                &mut ranks,
                vertex_offsets[segment_index] + vertices[1],
                vertex_offsets[segment_index] + vertices[2],
            );
            for vertex in vertices {
                referenced_vertices[vertex_offsets[segment_index] + vertex] = true;
            }
        }
    }
    // The duplicate groups have already been proven to be real topological
    // seams by a shared one-ring position. Join those writer/UV boundaries so
    // a logical surface component is measured once even when binary stream
    // partitioning split it into several render nodes. Coincident layered
    // cards are intentionally absent from these groups.
    for group in duplicate_groups {
        let mut referenced = group
            .iter()
            .copied()
            .filter(|(segment, vertex)| referenced_vertices[vertex_offsets[*segment] + *vertex]);
        let Some((first_segment, first_vertex)) = referenced.next() else {
            continue;
        };
        let first = vertex_offsets[first_segment] + first_vertex;
        for (segment, vertex) in referenced {
            union_weight_groups_v1(
                &mut parents,
                &mut ranks,
                first,
                vertex_offsets[segment] + vertex,
            );
        }
    }
    let mut component_vertices = BTreeMap::<usize, Vec<(usize, usize)>>::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        for vertex_index in 0..segment.positions.len() {
            if !referenced_vertices[vertex_offsets[segment_index] + vertex_index] {
                continue;
            }
            let root = find_weight_group_root_v1(
                &mut parents,
                vertex_offsets[segment_index] + vertex_index,
            );
            component_vertices
                .entry(root)
                .or_default()
                .push((segment_index, vertex_index));
        }
    }
    let measured_components = measured_edges
        .keys()
        .flat_map(|(left, right)| [*left, *right])
        .flat_map(|group| duplicate_groups[group].iter().copied())
        .map(|(segment_index, vertex_index)| {
            find_weight_group_root_v1(&mut parents, vertex_offsets[segment_index] + vertex_index)
        })
        .collect::<BTreeSet<_>>();
    // Tiny disconnected cards and shell fragments cannot bend internally in
    // a useful way. Blending two rotating carriers applies a non-rigid affine
    // transform to the whole card, which is exactly the extreme area spike
    // reported by the oracle. Larger anatomical surfaces remain smoothly
    // skinned and are never admitted to this route.
    const MAXIMUM_RIGID_COMPONENT_VERTEX_COUNT: usize = 512;
    let required_visible_anchor_bones = contract
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL"
                || node.structural_role == "APPENDAGE_TERMINAL"
                || node.anchor_role.as_deref().is_some_and(|role| {
                    let role = role.to_ascii_lowercase();
                    role.contains("paw")
                        || role.starts_with("tail_")
                        || role.starts_with("appendage_")
                })
        })
        .map(|(bone, _)| bone as u32)
        .collect::<BTreeSet<_>>();
    let mut visible_anchor_vertex_counts = required_visible_anchor_bones
        .iter()
        .map(|bone| {
            let count = model
                .segments
                .iter()
                .flat_map(|segment| &segment.weights)
                .filter(|row| aurora_weight_for_bone_v1(row, *bone) >= 0.1)
                .count();
            (*bone, count)
        })
        .collect::<BTreeMap<_, _>>();
    let mut rigidified = RigidComponentRefinementReportV1::default();
    let mut candidates = measured_components
        .into_iter()
        .filter_map(|root| {
            component_vertices
                .get(&root)
                .map(|component| (component.len(), root))
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    let mut claimed_duplicate_groups = BTreeSet::<usize>::new();
    for (vertex_count, root) in candidates {
        let component = &component_vertices[&root];
        if vertex_count == 0 || vertex_count > MAXIMUM_RIGID_COMPONENT_VERTEX_COUNT {
            continue;
        }
        let mut accumulated = BTreeMap::<u32, f32>::new();
        for &(segment_index, vertex_index) in component {
            accumulate_aurora_weight_row_v1(
                &model.segments[segment_index].weights[vertex_index],
                1.0,
                &mut accumulated,
            );
        }
        let Some(dominant) = accumulated
            .iter()
            .max_by(|left, right| left.1.total_cmp(right.1).then_with(|| right.0.cmp(left.0)))
            .map(|(bone, _)| *bone)
        else {
            continue;
        };
        let rigid = AuroraVertexWeightsV1 {
            bone_node_ids: [Some(dominant), None, None, None],
            values: [1.0, 0.0, 0.0, 0.0],
            influence_count: 1,
        };
        let candidate_groups = component
            .iter()
            .map(|(segment_index, vertex_index)| group_by_vertex[*segment_index][*vertex_index])
            .filter(|group| !claimed_duplicate_groups.contains(group))
            .collect::<BTreeSet<_>>();
        let target_vertices = candidate_groups
            .iter()
            .flat_map(|group| duplicate_groups[*group].iter().copied())
            .collect::<Vec<_>>();
        if target_vertices.is_empty() {
            continue;
        }
        // A terminal paw/tail influence is not, by itself, a reason to leave
        // a disconnected card non-rigid. That blanket rule prevented every
        // measured fur card from reaching the density gate. Preserve the real
        // contract instead: rigidification may remove terminal influence from
        // this component only when another visible (>= 0.1) vertex continues
        // to carry that required anchor after the complete atomic update.
        let mut anchor_count_updates = Vec::with_capacity(required_visible_anchor_bones.len());
        let preserves_required_anchors = required_visible_anchor_bones.iter().all(|bone| {
            let removed = target_vertices
                .iter()
                .filter(|(segment_index, vertex_index)| {
                    aurora_weight_for_bone_v1(
                        &model.segments[*segment_index].weights[*vertex_index],
                        *bone,
                    ) >= 0.1
                })
                .count();
            let added = usize::from(dominant == *bone) * target_vertices.len();
            let current = visible_anchor_vertex_counts.get(bone).copied().unwrap_or(0);
            let next = current.saturating_sub(removed) + added;
            anchor_count_updates.push((*bone, next));
            next > 0
        });
        if !preserves_required_anchors {
            continue;
        }
        let mut changed = false;
        for group in candidate_groups {
            claimed_duplicate_groups.insert(group);
            for &(target_segment, target_vertex) in &duplicate_groups[group] {
                changed |= model.segments[target_segment].weights[target_vertex] != rigid;
                model.segments[target_segment].weights[target_vertex] = rigid.clone();
                rigidified.vertices.insert((target_segment, target_vertex));
            }
        }
        for (bone, count) in anchor_count_updates {
            visible_anchor_vertex_counts.insert(bone, count);
        }
        rigidified.count += usize::from(changed);
    }
    Ok(rigidified)
}

fn find_weight_group_root_v1(parents: &mut [usize], node: usize) -> usize {
    let parent = parents[node];
    if parent != node {
        parents[node] = find_weight_group_root_v1(parents, parent);
    }
    parents[node]
}

fn union_weight_groups_v1(parents: &mut [usize], ranks: &mut [u8], left: usize, right: usize) {
    let left = find_weight_group_root_v1(parents, left);
    let right = find_weight_group_root_v1(parents, right);
    if left == right {
        return;
    }
    if ranks[left] < ranks[right] {
        parents[left] = right;
    } else {
        parents[right] = left;
        if ranks[left] == ranks[right] {
            ranks[left] = ranks[left].saturating_add(1);
        }
    }
}

fn accumulate_aurora_weight_row_v1(
    row: &AuroraVertexWeightsV1,
    scale: f32,
    output: &mut BTreeMap<u32, f32>,
) {
    for lane in 0..usize::from(row.influence_count.min(4)) {
        let Some(bone) = row.bone_node_ids[lane] else {
            continue;
        };
        let value = row.values[lane] * scale;
        if value.is_finite() && value > 0.0 {
            *output.entry(bone).or_default() += value;
        }
    }
}

fn normalize_aurora_weight_row_v1(
    accumulated: BTreeMap<u32, f32>,
) -> Result<AuroraVertexWeightsV1, ReferenceSupermodelMotionErrorV2> {
    let mut rows = accumulated
        .into_iter()
        .filter(|(_, value)| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let maximum = rows.first().map(|(_, value)| *value).ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-EMPTY",
            "model.segments.weights",
            "motion weight refinement produced an empty positive row",
        )
    })?;
    rows.retain(|(_, value)| *value >= maximum * 0.005);
    rows.truncate(4);
    let total = rows.iter().map(|(_, value)| *value).sum::<f32>();
    if !total.is_finite() || total <= 0.0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-WEIGHT-REFINEMENT-NORMALIZATION",
            "model.segments.weights",
            "motion weight refinement produced a non-positive total",
        ));
    }
    let mut output = AuroraVertexWeightsV1 {
        bone_node_ids: [None; 4],
        values: [0.0; 4],
        influence_count: rows.len() as u8,
    };
    for (lane, (bone, value)) in rows.into_iter().enumerate() {
        output.bone_node_ids[lane] = Some(bone);
        output.values[lane] = value / total;
    }
    Ok(output)
}

fn support_is_one_carrier_edge_v1(
    support: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    support.iter().all(|left| {
        support.iter().all(|right| {
            left == right
                || contract
                    .nodes
                    .get(*left as usize)
                    .is_some_and(|node| node.parent_part_number == Some(*right))
                || contract
                    .nodes
                    .get(*right as usize)
                    .is_some_and(|node| node.parent_part_number == Some(*left))
        })
    })
}

fn support_is_one_carrier_lineage_v1(
    support: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
) -> bool {
    let is_ancestor_or_same = |ancestor: u32, mut descendant: u32| {
        loop {
            if ancestor == descendant {
                return true;
            }
            let Some(parent) = contract
                .nodes
                .get(descendant as usize)
                .and_then(|node| node.parent_part_number)
            else {
                return false;
            };
            descendant = parent;
        }
    };
    support.iter().all(|left| {
        support
            .iter()
            .all(|right| is_ancestor_or_same(*left, *right) || is_ancestor_or_same(*right, *left))
    })
}

fn motion_quality_invariants_hold_v1(quality: &InheritedMotionQualityReportV1) -> bool {
    quality.seam_pair_violation_count == 0
        && quality.inherited_clip_coverage
        && quality.visible_motion_coverage
        && quality.joint_clip_coverage
        && quality.appendage_relative_motion_coverage
        && quality.anchor_cluster_missing_count == 0
        && quality.paw_cluster_missing_count == 0
        && quality.paw_contact_violation_count == 0
        && quality.paw_side_violation_count == 0
        && quality.clip_start_anchor_jump_violation_count == 0
}

fn motion_density_passes_v1(quality: &InheritedMotionQualityReportV1) -> bool {
    quality.edge_outside_hard_limit_count <= quality.edge_outside_hard_allowed_count
        && quality.edge_outside_soft_limit_count <= quality.edge_outside_soft_allowed_count
        && quality.triangle_area_collapse_count <= quality.triangle_area_collapse_allowed_count
        && quality.triangle_area_expansion_count <= quality.triangle_area_expansion_allowed_count
}

fn generic_density_transaction_has_repair_signal_v1(
    original: &InheritedMotionQualityReportV1,
    candidate: &InheritedMotionQualityReportV1,
) -> bool {
    motion_quality_invariants_hold_v1(candidate)
        && motion_density_passes_v1(original)
        && motion_density_passes_v1(candidate)
        && any_motion_density_count_strictly_improves_v1(
            [
                original.triangle_area_expansion_count,
                original.edge_outside_hard_limit_count,
                original.triangle_area_collapse_count,
            ],
            [
                candidate.triangle_area_expansion_count,
                candidate.edge_outside_hard_limit_count,
                candidate.triangle_area_collapse_count,
            ],
        )
}

fn any_motion_density_count_strictly_improves_v1(original: [u64; 3], candidate: [u64; 3]) -> bool {
    original
        .into_iter()
        .zip(candidate)
        .any(|(original, candidate)| candidate < original)
}

fn normalized_density_excess_v1(count: u64, sample_count: u64, max_fraction: f32) -> f64 {
    if sample_count == 0 {
        return if count == 0 { 0.0 } else { f64::INFINITY };
    }
    let allowed = (sample_count as f64 * f64::from(max_fraction)).floor() as u64;
    count.saturating_sub(allowed) as f64 / sample_count as f64
}

fn component_local_density_excess_score_v1(
    quality: &InheritedMotionQualityReportV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> f64 {
    quality
        .clips
        .iter()
        .flat_map(|clip| &clip.components)
        .map(|component| {
            normalized_density_excess_v1(
                component.edge_outside_hard_limit_count,
                component.edge_sample_count,
                tolerances.edge_hard_max_fraction,
            ) + normalized_density_excess_v1(
                component.triangle_area_collapse_count,
                component.triangle_sample_count,
                tolerances.triangle_area_collapse_max_fraction,
            ) + normalized_density_excess_v1(
                component.triangle_area_expansion_count,
                component.triangle_sample_count,
                tolerances.triangle_area_expansion_max_fraction,
            )
        })
        .sum()
}

fn motion_weight_refinement_exploration_is_admissible_v1(
    current: &InheritedMotionQualityReportV1,
    candidate: &InheritedMotionQualityReportV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> bool {
    if !motion_quality_invariants_hold_v1(candidate) || !motion_density_passes_v1(candidate) {
        return false;
    }
    let current_extrema = local_deformation_limit_severities_v1(current, tolerances);
    let candidate_extrema = local_deformation_limit_severities_v1(candidate, tolerances);
    let current_peak = current_extrema.into_iter().fold(1.0_f32, f32::max);
    let candidate_peak = candidate_extrema.into_iter().fold(0.0_f32, f32::max);
    // The exploration state is never emitted; only a separately retained
    // strict-improvement snapshot may become output. Permit one bounded 10x
    // transient while severity-weighted smoothing removes a weight cliff.
    candidate_peak.is_finite() && candidate_peak <= current_peak * 10.0
}

fn motion_weight_refinement_is_strict_improvement_v1(
    original: &InheritedMotionQualityReportV1,
    candidate: &InheritedMotionQualityReportV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> bool {
    if !motion_quality_invariants_hold_v1(candidate) {
        return false;
    }
    if candidate.status == "PASS" {
        return true;
    }
    let original_density_passes = motion_density_passes_v1(original);
    let candidate_density_passes = motion_density_passes_v1(candidate);
    if original_density_passes && !candidate_density_passes {
        return false;
    }
    let original_extrema = local_deformation_limit_severities_v1(original, tolerances);
    let candidate_extrema = local_deformation_limit_severities_v1(candidate, tolerances);
    const LOCAL_EXTREMA_EPSILON: f32 = 1.0e-4;
    if original_extrema
        .into_iter()
        .zip(candidate_extrema)
        .any(|(original_axis, candidate_axis)| {
            candidate_axis > original_axis.max(1.0) + LOCAL_EXTREMA_EPSILON
        })
    {
        return false;
    }
    if !original_density_passes && candidate_density_passes {
        // Crossing every density gate is useful, but it may only become the
        // retained output after the same per-axis extrema guard as every other
        // strict improvement. Density counts must never buy a worse blocking
        // local deformation.
        return true;
    }
    if candidate_density_passes
        && blocking_local_extrema_strictly_improve_v1(original_extrema, candidate_extrema)
    {
        // Once every global density gate and structural invariant is already
        // satisfied, removing a still-blocking worst local deformation takes
        // precedence over merely reducing the number of milder violations.
        // The per-axis guard above still forbids trading that improvement for
        // a regression on any other deformation axis.
        return true;
    }
    let local_failures = |quality: &InheritedMotionQualityReportV1| {
        let clips = quality
            .clips
            .iter()
            .filter(|clip| !clip_is_within_local_deformation_budget_v3(clip, tolerances))
            .count();
        let components = quality
            .clips
            .iter()
            .flat_map(|clip| &clip.components)
            .filter(|component| {
                !component_is_within_local_deformation_budget_v1(component, tolerances)
            })
            .count();
        clips + components
    };
    let original_local_failures = local_failures(original);
    let candidate_local_failures = local_failures(candidate);
    if candidate_local_failures != original_local_failures {
        return candidate_local_failures < original_local_failures;
    }
    let original_component_excess = component_local_density_excess_score_v1(original, tolerances);
    let candidate_component_excess = component_local_density_excess_score_v1(candidate, tolerances);
    const LOCAL_DENSITY_EXCESS_EPSILON: f64 = 1.0e-12;
    if (candidate_component_excess - original_component_excess).abs() > LOCAL_DENSITY_EXCESS_EPSILON
    {
        // Once all global density and absolute extrema gates are protected,
        // the measured per-component excess is the remaining blocking
        // objective. Counts that remain below their own allowed budgets must
        // not veto a real reduction in that excess.
        return candidate_component_excess < original_component_excess;
    }
    (
        candidate.triangle_area_expansion_count,
        candidate.edge_outside_hard_limit_count,
        candidate.triangle_area_collapse_count,
    ) < (
        original.triangle_area_expansion_count,
        original.edge_outside_hard_limit_count,
        original.triangle_area_collapse_count,
    )
}

fn blocking_local_extrema_strictly_improve_v1(original: [f32; 4], candidate: [f32; 4]) -> bool {
    const LOCAL_EXTREMA_EPSILON: f32 = 1.0e-4;
    original
        .into_iter()
        .zip(candidate)
        .any(|(original_axis, candidate_axis)| {
            original_axis > 1.0 + LOCAL_EXTREMA_EPSILON
                && candidate_axis + LOCAL_EXTREMA_EPSILON < original_axis
        })
}

fn targeted_catastrophic_plateau_is_strict_improvement_v1(
    original_quality: &InheritedMotionQualityReportV1,
    candidate_quality: &InheritedMotionQualityReportV1,
    original_violations: &MotionWeightViolationAccumulatorV1,
    candidate_violations: &MotionWeightViolationAccumulatorV1,
    expansion: bool,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> bool {
    if !motion_quality_invariants_hold_v1(candidate_quality)
        || (motion_density_passes_v1(original_quality)
            && !motion_density_passes_v1(candidate_quality))
    {
        return false;
    }
    let original_extrema = local_deformation_limit_severities_v1(original_quality, tolerances);
    let candidate_extrema = local_deformation_limit_severities_v1(candidate_quality, tolerances);
    const LOCAL_EXTREMA_EPSILON: f32 = 1.0e-4;
    let selected_axis = usize::from(!expansion);
    if original_extrema[selected_axis] <= 1.0 + LOCAL_EXTREMA_EPSILON
        || candidate_extrema.into_iter().zip(original_extrema).any(
            |(candidate_axis, original_axis)| {
                candidate_axis > original_axis.max(1.0) + LOCAL_EXTREMA_EPSILON
            },
        )
    {
        return false;
    }
    let original_penalties = if expansion {
        &original_violations.catastrophic_expansion_edge_penalties
    } else {
        &original_violations.catastrophic_collapse_edge_penalties
    };
    let candidate_penalties = if expansion {
        &candidate_violations.catastrophic_expansion_edge_penalties
    } else {
        &candidate_violations.catastrophic_collapse_edge_penalties
    };
    descending_penalty_distribution_strictly_improves_v1(
        original_penalties.values().copied(),
        candidate_penalties.values().copied(),
    )
}

fn descending_penalty_distribution_strictly_improves_v1(
    original: impl IntoIterator<Item = u64>,
    candidate: impl IntoIterator<Item = u64>,
) -> bool {
    let mut original = original.into_iter().collect::<Vec<_>>();
    let mut candidate = candidate.into_iter().collect::<Vec<_>>();
    original.sort_unstable_by(|left, right| right.cmp(left));
    candidate.sort_unstable_by(|left, right| right.cmp(left));
    for index in 0..original.len().max(candidate.len()) {
        let original_penalty = original.get(index).copied().unwrap_or(0);
        let candidate_penalty = candidate.get(index).copied().unwrap_or(0);
        if candidate_penalty != original_penalty {
            return candidate_penalty < original_penalty;
        }
    }
    false
}

fn targeted_refinement_step_needs_backoff_v1(
    original: &InheritedMotionQualityReportV1,
    candidate: &InheritedMotionQualityReportV1,
    expansion: bool,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> bool {
    let original_extrema = local_deformation_limit_severities_v1(original, tolerances);
    let candidate_extrema = local_deformation_limit_severities_v1(candidate, tolerances);
    targeted_extrema_tradeoff_needs_backoff_v1(original_extrema, candidate_extrema, expansion)
        || (motion_density_passes_v1(original)
            && !motion_density_passes_v1(candidate)
            && targeted_extrema_axis_improves_v1(original_extrema, candidate_extrema, expansion))
}

fn advance_targeted_refinement_search_v1(
    original: &InheritedMotionQualityReportV1,
    candidate: &InheritedMotionQualityReportV1,
    expansion: bool,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
    blend: &mut f32,
    endpoint_policy: &mut CatastrophicEndpointPolicyV1,
    target_policy: &mut CatastrophicTargetPolicyV1,
) -> bool {
    let original_extrema = local_deformation_limit_severities_v1(original, tolerances);
    let candidate_extrema = local_deformation_limit_severities_v1(candidate, tolerances);
    advance_targeted_refinement_search_extrema_v1(
        original_extrema,
        candidate_extrema,
        targeted_refinement_step_needs_backoff_v1(original, candidate, expansion, tolerances),
        expansion,
        blend,
        endpoint_policy,
        target_policy,
    )
}

fn advance_targeted_refinement_search_extrema_v1(
    original_extrema: [f32; 4],
    candidate_extrema: [f32; 4],
    needs_backoff: bool,
    expansion: bool,
    blend: &mut f32,
    endpoint_policy: &mut CatastrophicEndpointPolicyV1,
    target_policy: &mut CatastrophicTargetPolicyV1,
) -> bool {
    if needs_backoff && *blend > CATASTROPHIC_EDGE_MINIMUM_COHERENCE_BLEND_V1 {
        *blend = (*blend * 0.5).max(CATASTROPHIC_EDGE_MINIMUM_COHERENCE_BLEND_V1);
        return true;
    }
    if targeted_extrema_axis_is_flat_v1(original_extrema, candidate_extrema, expansion)
        && *blend < CATASTROPHIC_EDGE_MAXIMUM_COHERENCE_BLEND_V1
    {
        *blend = (*blend * 2.0).min(CATASTROPHIC_EDGE_MAXIMUM_COHERENCE_BLEND_V1);
        return true;
    }
    if let Some(next) = target_policy.next() {
        *target_policy = next;
        *blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
        return true;
    }
    if let Some(next) = endpoint_policy.next() {
        *endpoint_policy = next;
        *target_policy = CatastrophicTargetPolicyV1::MeasuredMotionBasis;
        *blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
        return true;
    }
    false
}

fn targeted_extrema_axis_improves_v1(
    original: [f32; 4],
    candidate: [f32; 4],
    expansion: bool,
) -> bool {
    const LOCAL_EXTREMA_EPSILON: f32 = 1.0e-4;
    let selected_axis = usize::from(!expansion);
    candidate[selected_axis] + LOCAL_EXTREMA_EPSILON < original[selected_axis]
}

fn targeted_extrema_axis_is_flat_v1(
    original: [f32; 4],
    candidate: [f32; 4],
    expansion: bool,
) -> bool {
    const LOCAL_EXTREMA_EPSILON: f32 = 1.0e-4;
    let selected_axis = usize::from(!expansion);
    (candidate[selected_axis] - original[selected_axis]).abs() <= LOCAL_EXTREMA_EPSILON
}

fn targeted_extrema_tradeoff_needs_backoff_v1(
    original: [f32; 4],
    candidate: [f32; 4],
    expansion: bool,
) -> bool {
    const LOCAL_EXTREMA_EPSILON: f32 = 1.0e-4;
    targeted_extrema_axis_improves_v1(original, candidate, expansion)
        && original
            .into_iter()
            .zip(candidate)
            .any(|(original_axis, candidate_axis)| {
                candidate_axis > original_axis.max(1.0) + LOCAL_EXTREMA_EPSILON
            })
}

fn targeted_catastrophic_expansion_v1(limit_severities: [f32; 4]) -> bool {
    // Edge expansion and edge collapse have independent endpoint solvers. The
    // triangle phase is entered separately only after the edge search reaches
    // its measured plateau; otherwise a bad triangle could starve an edge that
    // must first be stabilized for the triangle transaction to be admissible.
    limit_severities[0] >= limit_severities[1]
}

fn advance_targeted_triangle_blend_search_v1(blend: &mut f32) -> bool {
    if *blend <= TARGETED_TRIANGLE_MINIMUM_RIGID_BLEND_V1 {
        return false;
    }
    *blend = (*blend * 0.5).max(TARGETED_TRIANGLE_MINIMUM_RIGID_BLEND_V1);
    true
}

fn local_deformation_limit_severities_v1(
    quality: &InheritedMotionQualityReportV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> [f32; 4] {
    let edge_minimum = tolerances.edge_hard_min_ratio.powi(2);
    let edge_maximum = tolerances.edge_hard_max_ratio.powi(2);
    let area_minimum = tolerances.triangle_min_area_ratio.powi(2);
    let area_maximum = tolerances.triangle_max_area_ratio.powi(2);
    quality.clips.iter().flat_map(|clip| &clip.components).fold(
        [0.0_f32; 4],
        |mut extrema, component| {
            extrema[0] = extrema[0].max(component.max_edge_ratio / edge_maximum);
            extrema[1] = extrema[1].max(if component.min_edge_ratio > 0.0 {
                edge_minimum / component.min_edge_ratio
            } else {
                f32::INFINITY
            });
            extrema[2] = extrema[2].max(component.max_triangle_area_ratio / area_maximum);
            extrema[3] = extrema[3].max(if component.min_triangle_area_ratio > 0.0 {
                area_minimum / component.min_triangle_area_ratio
            } else {
                f32::INFINITY
            });
            extrema
        },
    )
}

fn validate_minimal_twosided_motion_model_v1(
    nodes: &[crate::mdl::NodeReport],
    diffuse_resref: &str,
    material_resref: &str,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    let mut mesh_count = 0usize;
    let mut tangent_stream_count = 0usize;
    let mut pending = nodes.iter().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        pending.extend(node.children.iter());
        let Some(mesh) = node.mesh.as_ref() else {
            continue;
        };
        mesh_count += 1;
        tangent_stream_count += usize::from(!mesh.tangents.is_empty());
        let diffuse_matches = mesh
            .textures
            .first()
            .is_some_and(|texture| texture.eq_ignore_ascii_case(diffuse_resref));
        let normal_and_specular_empty = mesh.textures.get(1).is_none_or(String::is_empty)
            && mesh.textures.get(2).is_none_or(String::is_empty);
        let material_matches = mesh
            .textures
            .get(3)
            .is_some_and(|texture| texture.eq_ignore_ascii_case(material_resref));
        if !diffuse_matches
            || !normal_and_specular_empty
            || !material_matches
            || mesh.render_hint != 1
            || !mesh.tangents.is_empty()
            || mesh.transparency != 0
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MINIMAL-MTR-MDL-CONTRACT",
                format!("model.nodes[{}].mesh", node.name),
                "minimal two-sided mesh requires texture0 diffuse, texture3 MTR, renderHint Normal, no normal/specular textures, no tangents and no transparency",
            ));
        }
    }
    if mesh_count == 0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-MESH-MISSING",
            "model.nodeTree",
            "minimal two-sided inherited-motion model contains no render meshes",
        ));
    }
    Ok(tangent_stream_count)
}

fn validate_classic_motion_model_v1(
    nodes: &[crate::mdl::NodeReport],
    diffuse_resref: &str,
) -> Result<usize, ReferenceSupermodelMotionErrorV2> {
    let mut mesh_count = 0usize;
    let mut tangent_stream_count = 0usize;
    let mut pending = nodes.iter().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        pending.extend(node.children.iter());
        let Some(mesh) = node.mesh.as_ref() else {
            continue;
        };
        mesh_count += 1;
        tangent_stream_count += usize::from(!mesh.tangents.is_empty());
        let diffuse_matches = mesh
            .textures
            .first()
            .is_some_and(|texture| texture.eq_ignore_ascii_case(diffuse_resref));
        let extension_slots_empty = mesh.textures.iter().skip(1).all(String::is_empty);
        if !diffuse_matches
            || !extension_slots_empty
            || mesh.render_hint != 0
            || !mesh.tangents.is_empty()
            || mesh.transparency != 0
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-CLASSIC-MDL-CONTRACT",
                format!("model.nodes[{}].mesh", node.name),
                "classic-safe mesh must contain only texture0 diffuse, renderHint 0, no tangents and no transparency",
            ));
        }
    }
    if mesh_count == 0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-MESH-MISSING",
            "model.nodeTree",
            "classic-safe inherited-motion model contains no render meshes",
        ));
    }
    Ok(tangent_stream_count)
}

/// Packages the classic-safe motion artifact with exactly one diffuse TGA and
/// one appearance table. Any MTR/TXI/normal/specular resource is structurally
/// impossible through this API.
pub fn package_reference_supermodel_classic_creature_hak_v1(
    artifact: &ReferenceSupermodelClassicMotionArtifactV1,
    texture_resref: &str,
    diffuse_tga: &[u8],
    appearance_two_da: &[u8],
    options: &HakWriterOptionsV1,
) -> Result<ReferenceSupermodelCreatureHakArtifactV1, ReferenceSupermodelMotionErrorV2> {
    if !artifact.report.motion_compatible
        || artifact.motion_quality.status != "PASS"
        || artifact.report.material_extension_applied
        || artifact.report.tangent_stream_count != 0
        || artifact.report.material_profile != "CLASSIC_DIFFUSE_TGA_SAFE_V1"
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-HAK-ARTIFACT-NOT-READY",
            "artifact",
            "only a motion-compatible, extension-free classic artifact may enter packaging",
        ));
    }
    if !is_resref(texture_resref) || diffuse_tga.is_empty() || appearance_two_da.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-HAK-INPUT-INVALID",
            "package",
            "classic packaging requires a valid texture resref and non-empty diffuse/appearance payloads",
        ));
    }
    let resources = vec![
        HakResourceInputV1 {
            resref: artifact.report.model_resource_resref.clone(),
            resource_type: 2002,
            payload: artifact.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: texture_resref.to_owned(),
            resource_type: 3,
            payload: diffuse_tga.to_vec(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance_two_da.to_vec(),
        },
    ];
    let hak = write_hak_v1(&resources, options)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let archive = ErfArchive::parse(&hak.payload).map_err(|source| {
        motion_error(
            source.code,
            format!("hak@{}", source.offset),
            source.context,
        )
    })?;
    if archive.resources().len() != resources.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-CLASSIC-HAK-RESOURCE-COUNT-DIFF",
            "hak.inventory",
            "classic HAK must contain exactly MDL, diffuse TGA and appearance.2da",
        ));
    }
    for resource in &resources {
        let payload = archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|source| {
                motion_error(
                    source.code,
                    format!("hak.{}:{}", resource.resref, resource.resource_type),
                    source.context,
                )
            })?;
        if payload != resource.payload {
            return Err(motion_error(
                "M2A-SUPERMODEL-CLASSIC-HAK-RESOURCE-DIFF",
                format!("hak.{}:{}", resource.resref, resource.resource_type),
                "classic HAK payload differs after readback",
            ));
        }
    }
    Ok(ReferenceSupermodelCreatureHakArtifactV1 {
        resource_count: resources.len(),
        hak,
    })
}

/// Packages a minimal two-sided inherited-motion artifact as exactly one MDL,
/// one diffuse TGA, one canonical MTR and one appearance table.
pub fn package_reference_supermodel_minimal_twosided_creature_hak_v1(
    artifact: &ReferenceSupermodelMinimalMtrMotionArtifactV1,
    texture_resref: &str,
    diffuse_tga: &[u8],
    appearance_two_da: &[u8],
    options: &HakWriterOptionsV1,
) -> Result<ReferenceSupermodelCreatureHakArtifactV1, ReferenceSupermodelMotionErrorV2> {
    if !artifact.report.motion_compatible
        || artifact.motion_quality.status != "PASS"
        || !artifact.report.material_extension_applied
        || artifact.report.tangent_stream_count != 0
        || artifact.report.material_resource_count != 2
        || artifact.report.material_profile != "NWN_EE_MTR_TWOSIDED_ONLY_V1"
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-HAK-ARTIFACT-NOT-READY",
            "artifact",
            "only a motion-compatible minimal two-sided artifact without tangents may enter packaging",
        ));
    }
    if !is_resref(texture_resref) || diffuse_tga.is_empty() || appearance_two_da.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-HAK-INPUT-INVALID",
            "package",
            "minimal two-sided packaging requires a valid texture resref and non-empty diffuse/appearance payloads",
        ));
    }
    if artifact.mtr_resource.resource_type != MTR_RESOURCE_TYPE_V1
        || !is_resref(&artifact.mtr_resource.resref)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-RESOURCE-INVALID",
            "artifact.mtrResource",
            "the minimal material resource must be a valid MTR resref and payload",
        ));
    }
    let mtr = parse_mtr_v1(&artifact.mtr_resource.payload)
        .map_err(|source| motion_error(source.code, "artifact.mtrResource", source.message))?;
    let diffuse_binding_matches = mtr.textures.len() == 1
        && mtr.textures[0].slot == 0
        && mtr.textures[0].resref.eq_ignore_ascii_case(texture_resref);
    if !mtr.two_sided
        || mtr.transparency
        || mtr.render_hint != MtrRenderHintV1::Normal
        || !diffuse_binding_matches
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-RESOURCE-SEMANTICS",
            "artifact.mtrResource",
            "minimal MTR must contain only texture0 diffuse, renderhint normal, transparency 0 and twosided 1",
        ));
    }
    validate_minimal_twosided_motion_model_v1(
        &artifact.model.inspection.node_tree.roots,
        texture_resref,
        &artifact.mtr_resource.resref,
    )?;
    validate_material_resource_semantics_v1(&artifact.mtr_resource)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;

    let resources = vec![
        HakResourceInputV1 {
            resref: artifact.report.model_resource_resref.clone(),
            resource_type: 2002,
            payload: artifact.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: texture_resref.to_owned(),
            resource_type: 3,
            payload: diffuse_tga.to_vec(),
        },
        artifact.mtr_resource.clone(),
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance_two_da.to_vec(),
        },
    ];
    let hak = write_hak_v1(&resources, options)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let archive = ErfArchive::parse(&hak.payload).map_err(|source| {
        motion_error(
            source.code,
            format!("hak@{}", source.offset),
            source.context,
        )
    })?;
    if archive.resources().len() != resources.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MINIMAL-MTR-HAK-RESOURCE-COUNT-DIFF",
            "hak.inventory",
            "minimal two-sided HAK resource count differs after readback",
        ));
    }
    for resource in &resources {
        let payload = archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|source| {
                motion_error(
                    source.code,
                    format!("hak.{}:{}", resource.resref, resource.resource_type),
                    source.context,
                )
            })?;
        if payload != resource.payload {
            return Err(motion_error(
                "M2A-SUPERMODEL-MINIMAL-MTR-HAK-RESOURCE-DIFF",
                format!("hak.{}:{}", resource.resref, resource.resource_type),
                "minimal two-sided HAK payload differs after readback",
            ));
        }
    }
    Ok(ReferenceSupermodelCreatureHakArtifactV1 {
        resource_count: resources.len(),
        hak,
    })
}

/// Packages one already validated motion/material artifact with an exact
/// caller-supplied `appearance.2da`. The returned HAK is reparsed and every
/// resource is compared byte-for-byte before success is reported.
pub fn package_reference_supermodel_creature_hak_v1(
    artifact: &ReferenceSupermodelMotionArtifactV2,
    appearance_two_da: &[u8],
    options: &HakWriterOptionsV1,
) -> Result<ReferenceSupermodelCreatureHakArtifactV1, ReferenceSupermodelMotionErrorV2> {
    if appearance_two_da.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-HAK-APPEARANCE-EMPTY",
            "appearanceTwoDa",
            "the caller must supply a non-empty already resolved appearance.2da",
        ));
    }
    if artifact.material_semantics.status != CreatureMaterialSemanticStatusV1::Ready
        || artifact.motion_quality.status != "PASS"
        || !artifact.report.motion_compatible
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-HAK-ARTIFACT-NOT-READY",
            "artifact",
            "only a material-ready and motion-compatible artifact may enter HAK packaging",
        ));
    }
    let mut resources = artifact.material_package.resources.clone();
    resources.push(HakResourceInputV1 {
        resref: artifact.report.model_resource_resref.clone(),
        resource_type: 2002,
        payload: artifact.model.payload.clone(),
    });
    resources.push(HakResourceInputV1 {
        resref: "appearance".to_owned(),
        resource_type: 2017,
        payload: appearance_two_da.to_vec(),
    });
    let hak = write_hak_v1(&resources, options)
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let archive = ErfArchive::parse(&hak.payload).map_err(|source| {
        motion_error(
            source.code,
            format!("hak@{}", source.offset),
            source.context,
        )
    })?;
    if archive.resources().len() != resources.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-HAK-RESOURCE-COUNT-DIFF",
            "hak.inventory",
            "HAK resource count differs after readback",
        ));
    }
    for resource in &resources {
        let payload = archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|source| {
                motion_error(
                    source.code,
                    format!("hak.{}:{}", resource.resref, resource.resource_type),
                    source.context,
                )
            })?;
        if payload != resource.payload {
            return Err(motion_error(
                "M2A-SUPERMODEL-HAK-RESOURCE-DIFF",
                format!("hak.{}:{}", resource.resref, resource.resource_type),
                "HAK payload differs from the validated input bytes",
            ));
        }
    }
    Ok(ReferenceSupermodelCreatureHakArtifactV1 {
        resource_count: resources.len(),
        hak,
    })
}

pub fn build_creature_material_semantic_report_v1(
    ingest: &crate::glb::GlbIngestResult,
    model: &crate::model_ir::AuroraModelIrV1,
    compilation: &AuroraMaterialCompilationSetV1,
    package: &AuroraMaterialPackageV1,
) -> Result<CreatureMaterialSemanticReportV1, ReferenceSupermodelMotionErrorV2> {
    let mut slots = Vec::with_capacity(package.slots.len());
    let mut all_present = true;
    for output in &package.slots {
        let binding = model
            .material_source_bindings
            .iter()
            .find(|binding| binding.slot == output.slot)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MATERIAL-SOURCE-BINDING-MISSING",
                    "model.materialSourceBindings",
                    format!("output slot {} has no source binding", output.slot),
                )
            })?;
        let source_id = binding.source_material_id.ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MATERIAL-SOURCE-BINDING-MISSING",
                "model.materialSourceBindings.sourceMaterialId",
                format!("output slot {} has no source material id", output.slot),
            )
        })?;
        let source = ingest
            .ir
            .materials
            .iter()
            .find(|material| material.id == source_id)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MATERIAL-SOURCE-MISSING",
                    "source.materials",
                    format!("source material {source_id} is absent"),
                )
            })?;
        let compiled = compilation
            .materials
            .iter()
            .find(|material| material.material.source_material_id == source_id)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MATERIAL-COMPILATION-MISSING",
                    "materialCompilation.materials",
                    format!("source material {source_id} has no compiler report"),
                )
            })?;
        let mut expected = vec![
            (output.diffuse_binding.resref.clone(), 3_u16),
            (output.diffuse_binding.resref.clone(), TXI_RESOURCE_TYPE_V1),
        ];
        if let Some(resref) = &output.state.normal_texture_resref {
            expected.push((resref.clone(), 3));
            expected.push((resref.clone(), TXI_RESOURCE_TYPE_V1));
        }
        if let Some(resref) = &output.state.specular_texture_resref {
            expected.push((resref.clone(), 3));
            expected.push((resref.clone(), TXI_RESOURCE_TYPE_V1));
        }
        if let Some(resref) = &output.state.material_resref {
            expected.push((resref.clone(), MTR_RESOURCE_TYPE_V1));
        }
        let resources = expected
            .into_iter()
            .map(|(resref, resource_type)| {
                let packaged = package.resources.iter().any(|actual| {
                    actual.resref.eq_ignore_ascii_case(&resref)
                        && actual.resource_type == resource_type
                });
                all_present &= packaged;
                CreatureMaterialResourceExpectationV1 {
                    resref,
                    resource_type,
                    packaged,
                }
            })
            .collect();
        slots.push(CreatureMaterialSemanticSlotV1 {
            source_material_id: source_id,
            output_slot: output.slot,
            diffuse_output_resref: output.diffuse_binding.resref.clone(),
            normal_output_resref: output.state.normal_texture_resref.clone(),
            specular_output_resref: output.state.specular_texture_resref.clone(),
            material_output_resref: output.state.material_resref.clone(),
            base_color_factor: source.base_color_factor,
            base_color_texture_id: source
                .base_color_texture
                .as_ref()
                .map(|value| value.texture_id),
            base_color_tex_coord_set: source
                .base_color_texture
                .as_ref()
                .map(|value| value.tex_coord_set),
            normal_texture_id: source.normal_texture.as_ref().map(|value| value.texture_id),
            normal_tex_coord_set: source
                .normal_texture
                .as_ref()
                .map(|value| value.tex_coord_set),
            metallic_factor: source.metallic_factor,
            roughness_factor: source.roughness_factor,
            metallic_roughness_texture_id: source
                .metallic_roughness_texture
                .as_ref()
                .map(|value| value.texture_id),
            metallic_roughness_tex_coord_set: source
                .metallic_roughness_texture
                .as_ref()
                .map(|value| value.tex_coord_set),
            alpha_mode: source.alpha_mode.clone(),
            alpha_cutoff: source.alpha_cutoff,
            double_sided: source.double_sided,
            channel_dispositions: compiled.fidelity.entries.clone(),
            resources,
        });
    }
    Ok(CreatureMaterialSemanticReportV1 {
        schema_version: 1,
        status: if all_present {
            CreatureMaterialSemanticStatusV1::Ready
        } else {
            CreatureMaterialSemanticStatusV1::Blocked
        },
        source_image_count: ingest.ir.images.len(),
        source_images: build_source_image_dispositions_v1(ingest, &slots),
        packaged_resource_count: package.resources.len(),
        slots,
    })
}

fn build_source_image_dispositions_v1(
    ingest: &crate::glb::GlbIngestResult,
    slots: &[CreatureMaterialSemanticSlotV1],
) -> Vec<CreatureSourceImageDispositionV1> {
    ingest
        .ir
        .images
        .iter()
        .map(|image| {
            let mut texture_ids = ingest
                .ir
                .textures
                .iter()
                .filter(|texture| texture.source_image_id == image.id)
                .map(|texture| texture.id)
                .collect::<Vec<_>>();
            texture_ids.sort_unstable();
            let mut channels = Vec::new();
            let mut outputs = Vec::new();
            let mut baked = false;
            for slot in slots {
                for texture_id in &texture_ids {
                    if slot.base_color_texture_id == Some(*texture_id) {
                        channels.push(format!("materials[{}].baseColor", slot.source_material_id));
                        if packaged_texture_resref(slot, &slot.diffuse_output_resref) {
                            outputs.push(slot.diffuse_output_resref.clone());
                        }
                    }
                    if slot.normal_texture_id == Some(*texture_id) {
                        channels.push(format!("materials[{}].normal", slot.source_material_id));
                        if let Some(resref) = &slot.normal_output_resref
                            && packaged_texture_resref(slot, resref)
                        {
                            outputs.push(resref.clone());
                        }
                    }
                    if slot.metallic_roughness_texture_id == Some(*texture_id) {
                        channels.push(format!(
                            "materials[{}].metallicRoughness",
                            slot.source_material_id
                        ));
                        baked = true;
                        if let Some(resref) = &slot.specular_output_resref
                            && packaged_texture_resref(slot, resref)
                        {
                            outputs.push(resref.clone());
                        }
                    }
                }
            }
            channels.sort();
            channels.dedup();
            outputs.sort();
            outputs.dedup();
            CreatureSourceImageDispositionV1 {
                source_image_id: image.id,
                source_texture_ids: texture_ids,
                source_channels: channels,
                output_resrefs: outputs.clone(),
                disposition: if outputs.is_empty() {
                    "DROPPED_WITH_WARNING".to_owned()
                } else if baked {
                    "BAKED".to_owned()
                } else {
                    "PRESERVED".to_owned()
                },
            }
        })
        .collect()
}

fn packaged_texture_resref(slot: &CreatureMaterialSemanticSlotV1, resref: &str) -> bool {
    slot.resources.iter().any(|resource| {
        resource.resource_type == 3
            && resource.packaged
            && resource.resref.eq_ignore_ascii_case(resref)
    })
}

#[derive(Clone, Debug)]
struct AnchorVertexReferenceV2 {
    skin_node_part: u32,
    vertex_index: usize,
    influence: f32,
}

#[derive(Clone, Debug)]
struct AnchorMotionContextV2 {
    assignments: BTreeMap<String, Vec<AnchorVertexReferenceV2>>,
    controller_name_by_role: BTreeMap<String, String>,
    carrier_part_by_role: BTreeMap<String, u32>,
    structural_role_by_role: BTreeMap<String, String>,
    dynamic_clips_by_role: BTreeMap<String, Vec<String>>,
    required_roles: Vec<String>,
    paw_roles: Vec<String>,
    appendage_role_chains: Vec<Vec<String>>,
}

#[derive(Clone, Copy, Debug)]
struct SampleGeometrySummaryV2 {
    bind_min: [f32; 3],
    bind_max: [f32; 3],
    bind_centroid: [f32; 3],
    sampled_min: [f32; 3],
    sampled_max: [f32; 3],
    sampled_centroid: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
struct AnchorClusterSampleV2 {
    bind_centroid: [f32; 3],
    sampled_centroid: [f32; 3],
    sampled_min_z: f32,
    bind_probe: [f32; 3],
    sampled_probe: [f32; 3],
}

fn build_anchor_motion_context_v2(
    target: &InspectionReport,
    contract: &ReferenceSupermodelMotionContractV2,
    correction: &ReferenceSupermodelCorrectionReportV1,
) -> Result<AnchorMotionContextV2, ReferenceSupermodelMotionErrorV2> {
    if correction.nodes.len() != contract.nodes.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-ANCHOR-CORRECTION-MISMATCH",
            "correction.nodes",
            "anchor evaluation requires one correction node per carrier node",
        ));
    }
    // SkinMesh node-to-bone maps are indexed by renderer tree ordinal, not by
    // MDL part number. Correction children make those orders differ, so using
    // the part-sorted helper here silently loses otherwise valid paw anchors.
    let flattened = flatten_nodes_in_tree_order(&target.node_tree.roots);
    let mut role_by_correction_part = BTreeMap::new();
    let mut controller_name_by_role = BTreeMap::new();
    let mut carrier_part_by_role = BTreeMap::new();
    let mut structural_role_by_role = BTreeMap::new();
    let mut dynamic_clips_by_role = BTreeMap::new();
    let mut required_roles = Vec::new();
    let mut paw_roles = Vec::new();
    for (part, contract_node) in contract.nodes.iter().enumerate() {
        let correction_node = &correction.nodes[part];
        if !correction_node
            .carrier_name
            .eq_ignore_ascii_case(&contract_node.name)
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-ANCHOR-CORRECTION-MISMATCH",
                format!("correction.nodes[{part}].carrierName"),
                "correction report carrier identity differs from the motion contract",
            ));
        }
        let Some(role) = contract_node
            .anchor_role
            .as_ref()
            .map(|role| role.to_ascii_lowercase())
        else {
            continue;
        };
        if role.contains("paw") {
            paw_roles.push(role.clone());
        }
        if role != "root" && role != "motion_root" {
            required_roles.push(role.clone());
        }
        controller_name_by_role.insert(role.clone(), contract_node.name.clone());
        let matching_parts = if correction.correction_node_count == 0 {
            // Direct carrier rigs are emitted in contract/tree order. The
            // writer intentionally renames the root to the resource resref,
            // and authoring IDs are not MDL part numbers, so neither name nor
            // numeric-ID lookup is valid for the root. Resolve the already
            // topology-validated direct carrier by its stable tree ordinal.
            flattened
                .get(part)
                .map(|node| vec![node.number])
                .unwrap_or_default()
        } else {
            flattened
                .iter()
                .filter(|node| {
                    node.name
                        .eq_ignore_ascii_case(&correction_node.correction_name)
                })
                .map(|node| node.number)
                .collect::<Vec<_>>()
        };
        if matching_parts.len() != 1 {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-ANCHOR-CORRECTION-MISSING",
                format!("correction.nodes[{part}].correctionName"),
                "output MDL must contain exactly one correction node with the reported name",
            ));
        }
        role_by_correction_part.insert(matching_parts[0], role.clone());
        carrier_part_by_role.insert(role.clone(), matching_parts[0]);
        structural_role_by_role.insert(role.clone(), contract_node.structural_role.clone());
        dynamic_clips_by_role.insert(role, contract_node.dynamic_clips.clone());
    }
    paw_roles.sort();
    paw_roles.dedup();
    required_roles.sort();
    required_roles.dedup();
    let appendage_semantic = |part: usize| {
        let node = &contract.nodes[part];
        node.structural_role.starts_with("APPENDAGE")
            || node.anchor_role.as_deref().is_some_and(|role| {
                let role = role.to_ascii_lowercase();
                role.starts_with("tail_") || role.starts_with("appendage_")
            })
    };
    let appendage_role_chains = contract
        .nodes
        .iter()
        .enumerate()
        .filter(|(part, _)| {
            appendage_semantic(*part)
                && !contract.nodes.iter().any(|child| {
                    child.parent_part_number == Some(*part as u32)
                        && appendage_semantic(child.part_number as usize)
                })
        })
        .filter_map(|(terminal, _)| {
            let mut roles = Vec::new();
            let mut current = Some(terminal as u32);
            while let Some(part) = current {
                if !appendage_semantic(part as usize) {
                    break;
                }
                if let Some(role) = contract.nodes[part as usize].anchor_role.as_ref() {
                    roles.push(role.to_ascii_lowercase());
                }
                current = contract.nodes[part as usize].parent_part_number;
            }
            roles.reverse();
            (roles.len() >= 2).then_some(roles)
        })
        .collect::<Vec<_>>();

    let mut assignments = BTreeMap::<String, Vec<AnchorVertexReferenceV2>>::new();
    let mut active_bone_parts = BTreeSet::new();
    for node in &flattened {
        let Some(skin) = &node.skin else {
            continue;
        };
        if skin.node_to_bone_map.len() != flattened.len()
            || skin.vertex_weights.len() != skin.bone_references.len()
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-ANCHOR-SKIN-LAYOUT",
                format!("target.nodeTree.nodes[{}].skin", node.number),
                "anchor evaluation requires complete bone maps and paired weight rows",
            ));
        }
        let mut role_by_bone_reference = BTreeMap::new();
        for (ordinal, &bone_reference) in skin.node_to_bone_map.iter().enumerate() {
            if bone_reference < 0 {
                continue;
            }
            active_bone_parts.insert(flattened[ordinal].number);
            let Some(role) = role_by_correction_part.get(&flattened[ordinal].number) else {
                continue;
            };
            if role_by_bone_reference
                .insert(bone_reference, role.clone())
                .is_some()
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-ANCHOR-BONE-AMBIGUOUS",
                    format!("target.nodeTree.nodes[{}].skin.nodeToBoneMap", node.number),
                    "one skin bone reference resolves to multiple anatomical anchors",
                ));
            }
        }
        for (vertex_index, (&weights, &references)) in skin
            .vertex_weights
            .iter()
            .zip(&skin.bone_references)
            .enumerate()
        {
            let mut best = None::<(f32, String)>;
            for lane in 0..4 {
                let weight = weights[lane];
                if !weight.is_finite() || weight < 0.1 || references[lane] == u16::MAX {
                    continue;
                }
                let Some(role) = role_by_bone_reference.get(&(references[lane] as i16)) else {
                    continue;
                };
                if best.as_ref().is_none_or(|(current, _)| weight > *current) {
                    best = Some((weight, role.clone()));
                }
            }
            if let Some((influence, role)) = best {
                assignments
                    .entry(role)
                    .or_default()
                    .push(AnchorVertexReferenceV2 {
                        skin_node_part: node.number,
                        vertex_index,
                        influence,
                    });
            }
        }
    }
    let missing_roles = missing_required_anchor_roles_v3(&required_roles, &assignments);
    if !missing_roles.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-ANCHOR-CLUSTER-MISSING",
            "motionQuality.weightedAnchors",
            format!(
                "skin weights do not resolve renderer tree-ordinal clusters for required semantic regions: {}; active bone parts: {:?}",
                missing_roles.join(", "),
                active_bone_parts,
            ),
        ));
    }
    Ok(AnchorMotionContextV2 {
        assignments,
        controller_name_by_role,
        carrier_part_by_role,
        structural_role_by_role,
        dynamic_clips_by_role,
        required_roles,
        paw_roles,
        appendage_role_chains,
    })
}

fn missing_required_anchor_roles_v3(
    required_roles: &[String],
    assignments: &BTreeMap<String, Vec<AnchorVertexReferenceV2>>,
) -> Vec<String> {
    required_roles
        .iter()
        .filter(|role| assignments.get(*role).is_none_or(Vec::is_empty))
        .cloned()
        .collect()
}

fn sample_geometry_summary_v2(
    sample: &crate::mdl::SkinDeformationSampleV1,
) -> Result<SampleGeometrySummaryV2, ReferenceSupermodelMotionErrorV2> {
    let mut bind_min = [f32::INFINITY; 3];
    let mut bind_max = [f32::NEG_INFINITY; 3];
    let mut sampled_min = [f32::INFINITY; 3];
    let mut sampled_max = [f32::NEG_INFINITY; 3];
    let mut bind_sum = [0.0_f64; 3];
    let mut sampled_sum = [0.0_f64; 3];
    let mut count = 0_u64;
    for skin in &sample.skins {
        for vertex in &skin.vertices {
            if vertex
                .bind_world
                .iter()
                .chain(&vertex.sampled_world)
                .any(|value| !value.is_finite())
            {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-NONFINITE-SAMPLE",
                    "motionQuality.samples.vertices",
                    "anchor and bounds evaluation encountered a non-finite vertex",
                ));
            }
            for axis in 0..3 {
                bind_min[axis] = bind_min[axis].min(vertex.bind_world[axis]);
                bind_max[axis] = bind_max[axis].max(vertex.bind_world[axis]);
                sampled_min[axis] = sampled_min[axis].min(vertex.sampled_world[axis]);
                sampled_max[axis] = sampled_max[axis].max(vertex.sampled_world[axis]);
                bind_sum[axis] += f64::from(vertex.bind_world[axis]);
                sampled_sum[axis] += f64::from(vertex.sampled_world[axis]);
            }
            count += 1;
        }
    }
    if count == 0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-SAMPLE-EMPTY",
            "motionQuality.samples",
            "motion quality requires at least one sampled skin vertex",
        ));
    }
    let divisor = count as f64;
    Ok(SampleGeometrySummaryV2 {
        bind_min,
        bind_max,
        bind_centroid: bind_sum.map(|value| (value / divisor) as f32),
        sampled_min,
        sampled_max,
        sampled_centroid: sampled_sum.map(|value| (value / divisor) as f32),
    })
}

fn inspect_anchor_motion_sample_v2(
    context: &AnchorMotionContextV2,
    sample: &crate::mdl::SkinDeformationSampleV1,
    geometry: &SampleGeometrySummaryV2,
    is_clip_start: bool,
    is_idle_clip: bool,
    is_grounded_locomotion_clip: bool,
    paw_expectations: Option<&PawControllerExpectationsV3>,
    clip_start_expected_deltas: Option<&BTreeMap<String, [f32; 3]>>,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
    quality: &mut InheritedMotionClipQualityV1,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    let diagonal = distance(geometry.bind_min, geometry.bind_max);
    if !diagonal.is_finite() || diagonal <= 1.0e-8 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-ANCHOR-BOUNDS",
            "motionQuality.samples.bounds",
            "anchor evaluation requires non-degenerate finite bind bounds",
        ));
    }
    let clusters = sample_anchor_clusters_v2(context, sample)?;
    let sampled_paw_ground = clusters
        .iter()
        .filter(|(role, _)| context.paw_roles.contains(role))
        .map(|(_, cluster)| cluster.sampled_min_z)
        .min_by(f32::total_cmp);
    for (role, cluster) in &clusters {
        if is_clip_start {
            let actual_delta = sub(cluster.sampled_probe, cluster.bind_probe);
            let jump = clip_start_expected_deltas
                .and_then(|expectations| expectations.get(role))
                .map_or_else(
                    || distance(cluster.bind_centroid, cluster.sampled_centroid),
                    |expected_delta| distance(actual_delta, *expected_delta),
                );
            quality.max_clip_start_anchor_jump = quality.max_clip_start_anchor_jump.max(jump);
            if jump > diagonal * tolerances.clip_start_anchor_max_jump_fraction {
                quality.clip_start_anchor_jump_violation_count += 1;
            }
        }
        if !context.paw_roles.contains(role) {
            continue;
        }
        let bind_side = cluster.bind_centroid[0] - geometry.bind_centroid[0];
        let sampled_side = cluster.sampled_centroid[0] - geometry.sampled_centroid[0];
        if is_grounded_locomotion_clip {
            let expected_side_delta = paw_expectations
                .and_then(|expectations| expectations.paws.get(role))
                .map(|paw| paw.sampled[0] - paw.bind[0])
                .unwrap_or(0.0);
            // Compare the exact same surface probe that was projected through
            // the selected supermodel controller. Using the whole cluster and
            // whole-model centroids here mixed unrelated torso/root motion
            // into a terminal-paw assertion and produced false side swaps.
            let actual_side_delta = cluster.sampled_probe[0] - cluster.bind_probe[0];
            let controller_tolerance =
                diagonal * tolerances.paw_controller_max_position_error_fraction;
            if paw_expectations.is_some()
                && expected_side_delta.abs() > controller_tolerance
                && actual_side_delta.abs() > controller_tolerance
                && actual_side_delta * expected_side_delta < 0.0
            {
                quality.paw_side_violation_count += 1;
            } else if paw_expectations.is_none()
                && bind_side.abs() > diagonal * 1.0e-4
                && bind_side * sampled_side < 0.0
            {
                quality.paw_side_violation_count += 1;
            }
        }
        if is_idle_clip {
            let error = paw_expectations
                .and_then(|expectations| expectations.paws.get(role))
                .map(|paw| {
                    let expected_delta = paw.sampled[2] - paw.bind[2];
                    let actual_delta = cluster.sampled_centroid[2] - cluster.bind_centroid[2];
                    (actual_delta - expected_delta).abs()
                })
                .unwrap_or_else(|| {
                    // Legacy context-free callers compare paw clusters with
                    // the lowest paw. Product validation always supplies the
                    // exact selected-supermodel controller expectation.
                    sampled_paw_ground
                        .map_or(0.0, |ground| (cluster.sampled_min_z - ground).max(0.0))
                });
            quality.max_paw_ground_height_error = quality.max_paw_ground_height_error.max(error);
            if error > diagonal * tolerances.paw_ground_contact_max_height_fraction {
                quality.paw_contact_violation_count += 1;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct PawControllerSampleV3 {
    bind: [f32; 3],
    sampled: [f32; 3],
}

#[derive(Clone, Debug)]
struct PawControllerExpectationsV3 {
    paws: BTreeMap<String, PawControllerSampleV3>,
}

fn build_paw_controller_expectations_v3(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
    context: &AnchorMotionContextV2,
    deformation_sample: &crate::mdl::SkinDeformationSampleV1,
) -> Result<Option<PawControllerExpectationsV3>, ReferenceSupermodelMotionErrorV2> {
    if context.paw_roles.is_empty() {
        return Ok(None);
    }
    let mut parts = Vec::new();
    for role in &context.paw_roles {
        parts.push(*context.carrier_part_by_role.get(role).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-PAW-CARRIER-MISSING",
                "motionQuality.pawControllers",
                format!("paw role {role:?} has no carrier part"),
            )
        })?);
    }
    parts.sort_unstable();
    parts.dedup();
    let samples = evaluate_reference_supermodel_node_world_matrices_v2(
        target,
        supermodel,
        clip_name,
        time_seconds,
        &parts,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let clusters = sample_anchor_clusters_v2(context, deformation_sample)?;
    let as_paw_sample = |role: &str, part: u32| {
        let controller = samples
            .iter()
            .find(|sample| sample.node_part == part)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-PAW-CONTROLLER-SAMPLE-MISSING",
                    "motionQuality.pawControllers",
                    format!("controller sample for target part {part} is absent"),
                )
            })?;
        let cluster = clusters.get(role).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-PAW-CLUSTER-SAMPLE-MISSING",
                "motionQuality.pawControllers",
                format!("surface probe for paw role {role:?} is absent"),
            )
        })?;
        let bind_inverse = inverse_affine(controller.bind_world_matrix).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-PAW-BIND-NONINVERTIBLE",
                "motionQuality.pawControllers",
                format!("controller bind for paw role {role:?} is non-invertible"),
            )
        })?;
        let local_probe = transform_point_v3(bind_inverse, cluster.bind_probe);
        Ok(PawControllerSampleV3 {
            bind: cluster.bind_probe,
            sampled: transform_point_v3(controller.sampled_world_matrix, local_probe),
        })
    };
    let paws = context
        .paw_roles
        .iter()
        .map(|role| {
            let part = context.carrier_part_by_role[role];
            Ok((role.clone(), as_paw_sample(role, part)?))
        })
        .collect::<Result<BTreeMap<_, _>, ReferenceSupermodelMotionErrorV2>>()?;
    Ok(Some(PawControllerExpectationsV3 { paws }))
}

fn build_clip_start_controller_deltas_v4(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    clip_name: &str,
    time_seconds: f32,
    context: &AnchorMotionContextV2,
    deformation_sample: &crate::mdl::SkinDeformationSampleV1,
) -> Result<BTreeMap<String, [f32; 3]>, ReferenceSupermodelMotionErrorV2> {
    let mut roles = context.required_roles.clone();
    roles.sort();
    roles.dedup();
    let parts = roles
        .iter()
        .map(|role| {
            context
                .carrier_part_by_role
                .get(role)
                .copied()
                .ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-CLIP-START-CARRIER-MISSING",
                        "motionQuality.clipStartControllers",
                        format!("required role {role:?} has no carrier part"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let matrices = evaluate_reference_supermodel_node_world_matrices_v2(
        target,
        supermodel,
        clip_name,
        time_seconds,
        &parts,
    )
    .map_err(|source| motion_error(source.code, source.path, source.message))?;
    let clusters = sample_anchor_clusters_v2(context, deformation_sample)?;
    roles
        .into_iter()
        .zip(parts)
        .map(|(role, part)| {
            let controller = matrices
                .iter()
                .find(|sample| sample.node_part == part)
                .ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-CLIP-START-CONTROLLER-SAMPLE-MISSING",
                        "motionQuality.clipStartControllers",
                        format!("controller sample for role {role:?} is absent"),
                    )
                })?;
            let cluster = clusters.get(&role).ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-CLIP-START-CLUSTER-SAMPLE-MISSING",
                    "motionQuality.clipStartControllers",
                    format!("surface cluster for role {role:?} is absent"),
                )
            })?;
            let bind_inverse = inverse_affine(controller.bind_world_matrix).ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-CLIP-START-BIND-NONINVERTIBLE",
                    "motionQuality.clipStartControllers",
                    format!("controller bind for role {role:?} is non-invertible"),
                )
            })?;
            let local_probe = transform_point_v3(bind_inverse, cluster.bind_probe);
            let expected = transform_point_v3(controller.sampled_world_matrix, local_probe);
            Ok((role, sub(expected, cluster.bind_probe)))
        })
        .collect()
}

fn sample_anchor_clusters_v2(
    context: &AnchorMotionContextV2,
    sample: &crate::mdl::SkinDeformationSampleV1,
) -> Result<BTreeMap<String, AnchorClusterSampleV2>, ReferenceSupermodelMotionErrorV2> {
    let mut output = BTreeMap::new();
    for (role, references) in &context.assignments {
        let mut bind_sum = [0.0_f64; 3];
        let mut sampled_sum = [0.0_f64; 3];
        let mut sampled_min_z = f32::INFINITY;
        let mut count = 0_u64;
        let mut points = Vec::with_capacity(references.len());
        for reference in references {
            let skin = sample
                .skins
                .iter()
                .find(|skin| skin.node_part == reference.skin_node_part)
                .ok_or_else(|| {
                    motion_error(
                        "M2A-SUPERMODEL-MOTION-ANCHOR-SKIN-MISSING",
                        "motionQuality.samples.skins",
                        format!(
                            "sample is missing anchor skin node part {}",
                            reference.skin_node_part
                        ),
                    )
                })?;
            let vertex = skin.vertices.get(reference.vertex_index).ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-ANCHOR-VERTEX-MISSING",
                    "motionQuality.samples.skins.vertices",
                    "sample is missing an anchor-assigned vertex",
                )
            })?;
            for axis in 0..3 {
                bind_sum[axis] += f64::from(vertex.bind_world[axis]);
                sampled_sum[axis] += f64::from(vertex.sampled_world[axis]);
            }
            sampled_min_z = sampled_min_z.min(vertex.sampled_world[2]);
            points.push((reference.influence, vertex.bind_world, vertex.sampled_world));
            count += 1;
        }
        if count == 0 {
            continue;
        }
        let divisor = count as f64;
        let bind_centroid = bind_sum.map(|value| (value / divisor) as f32);
        let sampled_centroid = sampled_sum.map(|value| (value / divisor) as f32);
        let (_, bind_probe, sampled_probe) = points
            .into_iter()
            .max_by(|(left_weight, left, _), (right_weight, right, _)| {
                left_weight.total_cmp(right_weight).then_with(|| {
                    distance(*left, bind_centroid).total_cmp(&distance(*right, bind_centroid))
                })
            })
            .expect("non-empty anchor cluster");
        output.insert(
            role.clone(),
            AnchorClusterSampleV2 {
                bind_centroid,
                sampled_centroid,
                sampled_min_z,
                bind_probe,
                sampled_probe,
            },
        );
    }
    Ok(output)
}

pub fn inspect_inherited_motion_quality_v1(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    required_clips: &[String],
) -> Result<InheritedMotionQualityReportV1, ReferenceSupermodelMotionErrorV2> {
    inspect_inherited_motion_quality_with_tolerances_v2(
        target,
        supermodel,
        required_clips,
        &default_reference_supermodel_motion_tolerances_v2(),
    )
}

/// Runs the deformation gate with the anatomical anchors declared by the
/// motion contract. Quadruped contracts that declare paw roles additionally
/// require non-empty paw-weight clusters, stable left/right placement, idle
/// ground contact and a bounded clip-start jump.
pub fn inspect_inherited_motion_quality_for_contract_v2(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    contract: &ReferenceSupermodelMotionContractV2,
    correction: &ReferenceSupermodelCorrectionReportV1,
) -> Result<InheritedMotionQualityReportV1, ReferenceSupermodelMotionErrorV2> {
    let anchors = build_anchor_motion_context_v2(target, contract, correction)?;
    inspect_inherited_motion_quality_internal_v2(
        target,
        supermodel,
        &contract.required_clips,
        &contract.tolerances,
        Some(&anchors),
        None,
    )
}

fn inspect_inherited_motion_quality_with_refinement_v1(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    contract: &ReferenceSupermodelMotionContractV2,
    correction: &ReferenceSupermodelCorrectionReportV1,
) -> Result<
    (
        InheritedMotionQualityReportV1,
        MotionWeightViolationAccumulatorV1,
    ),
    ReferenceSupermodelMotionErrorV2,
> {
    let anchors = build_anchor_motion_context_v2(target, contract, correction)?;
    let mut violations = MotionWeightViolationAccumulatorV1::default();
    let quality = inspect_inherited_motion_quality_internal_v2(
        target,
        supermodel,
        &contract.required_clips,
        &contract.tolerances,
        Some(&anchors),
        Some(&mut violations),
    )?;
    Ok((quality, violations))
}

pub fn inspect_inherited_motion_quality_with_tolerances_v2(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    required_clips: &[String],
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> Result<InheritedMotionQualityReportV1, ReferenceSupermodelMotionErrorV2> {
    inspect_inherited_motion_quality_internal_v2(
        target,
        supermodel,
        required_clips,
        tolerances,
        None,
        None,
    )
}

fn inspect_inherited_motion_quality_internal_v2(
    target: &InspectionReport,
    supermodel: &InspectionReport,
    required_clips: &[String],
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
    anchor_context: Option<&AnchorMotionContextV2>,
    mut weight_violations: Option<&mut MotionWeightViolationAccumulatorV1>,
) -> Result<InheritedMotionQualityReportV1, ReferenceSupermodelMotionErrorV2> {
    if required_clips.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-CLIPS-EMPTY",
            "requiredClips",
            "at least one inherited clip is required",
        ));
    }
    let mut geometry_sampling_plan = build_geometry_sampling_plan_v3(target)?;
    let mut clips = Vec::with_capacity(required_clips.len());
    let mut joint_clip_coverage_matrix = Vec::new();
    for required in required_clips {
        let clip = supermodel
            .animations
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(required))
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-CLIP-MISSING",
                    "supermodel.animations",
                    format!("required inherited clip {required} is absent"),
                )
            })?;
        let sampled_times = animation_sample_times_v3(clip);
        let deformation_samples = evaluate_reference_supermodel_render_deformation_samples_v3(
            target,
            supermodel,
            required,
            &sampled_times,
        )
        .map_err(|source| motion_error(source.code, source.path, source.message))?;
        if geometry_sampling_plan.seam_pairs.is_none() {
            let bind_sample = deformation_samples.first().ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-GEOMETRY-SAMPLE-EMPTY",
                    "motionQuality.geometrySamplingPlan.logicalComponents",
                    format!("required inherited clip {required:?} produced no deformation samples"),
                )
            })?;
            finalize_geometry_sampling_plan_v3(
                target,
                bind_sample,
                &mut geometry_sampling_plan,
                tolerances,
            )?;
        }
        let paw_gate_applied = anchor_context.is_some_and(|context| !context.paw_roles.is_empty());
        let anchor_cluster_count = anchor_context.map_or(0, |context| {
            context
                .required_roles
                .iter()
                .filter(|role| {
                    context
                        .assignments
                        .get(*role)
                        .is_some_and(|rows| !rows.is_empty())
                })
                .count()
        });
        let anchor_cluster_missing_count = anchor_context
            .map_or(0, |context| context.required_roles.len())
            .saturating_sub(anchor_cluster_count);
        let paw_cluster_count = anchor_context.map_or(0, |context| {
            context
                .paw_roles
                .iter()
                .filter(|role| {
                    context
                        .assignments
                        .get(*role)
                        .is_some_and(|rows| !rows.is_empty())
                })
                .count()
        });
        let paw_cluster_missing_count = anchor_context
            .map_or(0, |context| context.paw_roles.len())
            .saturating_sub(paw_cluster_count);
        let mut quality = InheritedMotionClipQualityV1 {
            clip_name: clip.name.clone(),
            sampled_times: sampled_times.clone(),
            vertex_sample_count: 0,
            edge_sample_count: 0,
            edge_soft_sample_count: 0,
            edge_outside_soft_limit_count: 0,
            edge_outside_hard_limit_count: 0,
            triangle_sample_count: 0,
            triangle_area_collapse_count: 0,
            triangle_area_expansion_count: 0,
            sampled_component_count: geometry_sampling_plan.total_component_count,
            component_sample_count: (geometry_sampling_plan.total_component_count
                * sampled_times.len()) as u64,
            per_component_geometry_coverage: geometry_sampling_plan.total_component_count > 0
                && !sampled_times.is_empty(),
            per_component_deformation_coverage: false,
            components: (0..geometry_sampling_plan.total_component_count)
                .map(|component_index| InheritedMotionComponentQualityV1 {
                    component_index,
                    triangle_sample_count: 0,
                    edge_sample_count: 0,
                    edge_outside_hard_limit_count: 0,
                    triangle_area_collapse_count: 0,
                    triangle_area_expansion_count: 0,
                    min_edge_ratio: f32::INFINITY,
                    max_edge_ratio: 0.0,
                    min_triangle_area_ratio: f32::INFINITY,
                    max_triangle_area_ratio: 0.0,
                    pass: false,
                })
                .collect(),
            worst_triangle_area_collapse: None,
            worst_triangle_area_expansion: None,
            worst_edge_collapse: None,
            worst_edge_expansion: None,
            world_normal_opposition_count: 0,
            seam_pair_sample_count: 0,
            seam_pair_violation_count: 0,
            visible_anchor_trajectories: Vec::new(),
            visible_anchor_motion_violation_count: 0,
            max_edge_ratio: 0.0,
            min_edge_ratio: f32::INFINITY,
            max_displacement: 0.0,
            bounds_min: [f32::INFINITY; 3],
            bounds_max: [f32::NEG_INFINITY; 3],
            paw_gate_applied,
            anchor_cluster_count,
            anchor_cluster_missing_count,
            paw_cluster_count,
            paw_cluster_missing_count,
            paw_contact_violation_count: 0,
            paw_side_violation_count: 0,
            clip_start_anchor_jump_violation_count: 0,
            max_paw_ground_height_error: 0.0,
            max_clip_start_anchor_jump: 0.0,
            root_motion_distance: 0.0,
            appendage_pair_required_count: 0,
            appendage_pair_pass_count: 0,
            appendage_relative_motion_violation_count: 0,
            appendage_relative_motion: Vec::new(),
        };
        let mut first_center = None;
        let mut last_center = None;
        let dynamic_controllers = anchor_context
            .map(|context| {
                context
                    .controller_name_by_role
                    .iter()
                    .filter(|(role, _)| {
                        context
                            .dynamic_clips_by_role
                            .get(*role)
                            .is_some_and(|clips| {
                                clips.iter().any(|clip| clip.eq_ignore_ascii_case(required))
                            })
                    })
                    .map(|(role, name)| (role.clone(), name.clone()))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let mut visible_anchor_samples = dynamic_controllers
            .keys()
            .map(|role| (role.clone(), Vec::new()))
            .collect::<BTreeMap<_, Vec<VisibleAnchorTrajectorySampleV1>>>();
        for (sample_index, (time, sample)) in sampled_times
            .iter()
            .copied()
            .zip(deformation_samples)
            .enumerate()
        {
            quality.vertex_sample_count += u64::from(sample.vertex_count);
            quality.max_displacement = quality.max_displacement.max(sample.max_displacement);
            inspect_sample_geometry(
                target,
                &sample,
                &mut geometry_sampling_plan,
                tolerances,
                &mut quality,
                weight_violations.as_deref_mut(),
            )?;
            let geometry = sample_geometry_summary_v2(&sample)?;
            for axis in 0..3 {
                quality.bounds_min[axis] = quality.bounds_min[axis].min(geometry.sampled_min[axis]);
                quality.bounds_max[axis] = quality.bounds_max[axis].max(geometry.sampled_max[axis]);
            }
            first_center.get_or_insert(geometry.sampled_centroid);
            last_center = Some(geometry.sampled_centroid);
            if let Some(context) = anchor_context {
                let paw_expectations = build_paw_controller_expectations_v3(
                    target, supermodel, required, time, context, &sample,
                )?;
                let clip_start_expected_deltas = if sample_index == 0 {
                    Some(build_clip_start_controller_deltas_v4(
                        target, supermodel, required, time, context, &sample,
                    )?)
                } else {
                    None
                };
                inspect_anchor_motion_sample_v2(
                    context,
                    &sample,
                    &geometry,
                    sample_index == 0,
                    required.eq_ignore_ascii_case("cpause1"),
                    [
                        "cpause1", "cwalk", "crun", "ccwalkf", "ccwalkb", "ccwalkl", "ccwalkr",
                    ]
                    .iter()
                    .any(|clip| required.eq_ignore_ascii_case(clip)),
                    paw_expectations.as_ref(),
                    clip_start_expected_deltas.as_ref(),
                    tolerances,
                    &mut quality,
                )?;
                if !dynamic_controllers.is_empty() {
                    let clusters = sample_anchor_clusters_v2(context, &sample)?;
                    let controller_parts = dynamic_controllers
                        .keys()
                        .map(|role| {
                            context
                                .carrier_part_by_role
                                .get(role)
                                .copied()
                                .ok_or_else(|| {
                                    motion_error(
                                        "M2A-SUPERMODEL-VISIBLE-JOINT-PART-MISSING",
                                        "motionQuality.jointClipCoverage",
                                        format!("carrier part for role {role:?} is absent"),
                                    )
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let controller_samples = evaluate_reference_supermodel_node_world_matrices_v2(
                        target,
                        supermodel,
                        required,
                        time,
                        &controller_parts,
                    )
                    .map_err(|source| motion_error(source.code, source.path, source.message))?;
                    for role in dynamic_controllers.keys() {
                        let cluster = clusters.get(role).ok_or_else(|| {
                            motion_error(
                                "M2A-SUPERMODEL-VISIBLE-JOINT-CLUSTER-MISSING",
                                "motionQuality.jointClipCoverage",
                                format!("visible cluster {role:?} is absent"),
                            )
                        })?;
                        let part = context.carrier_part_by_role[role];
                        let controller = controller_samples
                            .iter()
                            .find(|sample| sample.node_part == part)
                            .ok_or_else(|| {
                                motion_error(
                                    "M2A-SUPERMODEL-VISIBLE-JOINT-CONTROLLER-MISSING",
                                    "motionQuality.jointClipCoverage",
                                    format!("controller carrier part {part} is absent"),
                                )
                            })?;
                        let bind_inverse = inverse_affine(controller.bind_world_matrix)
                            .ok_or_else(|| {
                                motion_error(
                                    "M2A-SUPERMODEL-VISIBLE-JOINT-BIND-NONINVERTIBLE",
                                    "motionQuality.jointClipCoverage",
                                    format!("carrier part {part} has a non-invertible bind"),
                                )
                            })?;
                        let local_probe = transform_point_v3(bind_inverse, cluster.bind_probe);
                        let controller_probe =
                            transform_point_v3(controller.sampled_world_matrix, local_probe);
                        visible_anchor_samples
                            .get_mut(role)
                            .expect("role initialized")
                            .push(VisibleAnchorTrajectorySampleV1 {
                                time_seconds: time,
                                surface_centroid: cluster.sampled_probe,
                                controller_centroid: controller_probe,
                            });
                    }
                }
            }
        }
        for component in &mut quality.components {
            if !component.min_edge_ratio.is_finite() {
                component.min_edge_ratio = 1.0;
            }
            if !component.min_triangle_area_ratio.is_finite() {
                component.min_triangle_area_ratio = 1.0;
            }
            component.pass = component_is_within_local_deformation_budget_v1(component, tolerances);
        }
        if let Some(violations) = weight_violations.as_deref_mut() {
            for component in quality
                .components
                .iter()
                .filter(|component| !component.pass)
            {
                violations.mark_failed_component(&quality.clip_name, component.component_index);
            }
        }
        quality.per_component_geometry_coverage = quality.components.iter().all(|component| {
            component.edge_sample_count > 0 && component.triangle_sample_count > 0
        });
        quality.per_component_deformation_coverage = quality.components.iter().all(|row| row.pass);
        if let Some(context) = anchor_context {
            for chain in &context.appendage_role_chains {
                for pair in chain.windows(2) {
                    let (Some(parent), Some(child)) = (
                        visible_anchor_samples.get(&pair[0]),
                        visible_anchor_samples.get(&pair[1]),
                    ) else {
                        continue;
                    };
                    let diagnostic = evaluate_appendage_relative_motion_v1(
                        &pair[0],
                        &pair[1],
                        parent,
                        child,
                        tolerances.visible_anchor_min_amplitude_ratio,
                        tolerances.visible_anchor_min_trajectory_alignment,
                    )?;
                    if diagnostic.controller_relative_amplitude > 1.0e-7 {
                        quality.appendage_pair_required_count += 1;
                        quality.appendage_pair_pass_count += usize::from(diagnostic.pass);
                        quality.appendage_relative_motion_violation_count +=
                            u64::from(!diagnostic.pass);
                    }
                    quality.appendage_relative_motion.push(diagnostic);
                }
            }
        }
        for (role, samples) in visible_anchor_samples {
            let mut report = evaluate_visible_anchor_trajectory_v1(
                &role,
                required,
                &samples,
                tolerances.visible_anchor_min_amplitude_ratio,
                tolerances.visible_anchor_min_trajectory_alignment,
            )?;
            let structural_role = anchor_context
                .and_then(|context| context.structural_role_by_role.get(&role))
                .cloned()
                .unwrap_or_else(|| "UNCLASSIFIED".to_owned());
            let trajectory_required =
                role.starts_with("tail") || structural_role == "APPENDAGE_TERMINAL";
            let pass = report.controller_amplitude > 1.0e-7
                && report.surface_amplitude > 1.0e-7
                && report.amplitude_ratio >= tolerances.visible_anchor_min_amplitude_ratio
                && (!trajectory_required
                    || report.trajectory_alignment
                        >= tolerances.visible_anchor_min_trajectory_alignment);
            report.pass = pass;
            report.status = if pass {
                "PASS"
            } else if report.controller_amplitude <= 1.0e-7 {
                "BLOCKED_CONTROLLER_STATIC"
            } else if report.surface_amplitude <= 1.0e-7 {
                "BLOCKED_STATIC_VISIBLE_CLUSTER"
            } else if report.amplitude_ratio < tolerances.visible_anchor_min_amplitude_ratio {
                "BLOCKED_AMPLITUDE"
            } else {
                "BLOCKED_TRAJECTORY"
            }
            .to_owned();
            let cluster_vertex_count = anchor_context
                .and_then(|context| context.assignments.get(&role))
                .map_or(0, Vec::len);
            joint_clip_coverage_matrix.push(ReferenceSupermodelJointClipCoverageCellV3 {
                schema_version: 3,
                joint_name: anchor_context
                    .and_then(|context| context.controller_name_by_role.get(&role))
                    .cloned()
                    .unwrap_or_else(|| role.clone()),
                structural_role,
                clip_name: required.clone(),
                cluster_vertex_count,
                surface_amplitude: report.surface_amplitude,
                controller_probe_amplitude: report.controller_amplitude,
                amplitude_ratio: report.amplitude_ratio,
                trajectory_alignment: report.trajectory_alignment,
                trajectory_required,
                status: report.status.clone(),
                pass,
            });
            quality.visible_anchor_motion_violation_count += u64::from(!pass);
            quality.visible_anchor_trajectories.push(report);
        }
        if let (Some(first), Some(last)) = (first_center, last_center) {
            quality.root_motion_distance = distance(first, last);
        }
        if !quality.min_edge_ratio.is_finite() {
            quality.min_edge_ratio = 1.0;
        }
        if quality
            .bounds_min
            .iter()
            .chain(&quality.bounds_max)
            .any(|value| !value.is_finite())
        {
            quality.bounds_min = [0.0; 3];
            quality.bounds_max = [0.0; 3];
        }
        clips.push(quality);
    }
    let edge_sample_count = clips.iter().map(|clip| clip.edge_sample_count).sum();
    let edge_soft_sample_count = clips.iter().map(|clip| clip.edge_soft_sample_count).sum();
    let edge_outside_soft_limit_count = clips
        .iter()
        .map(|clip| clip.edge_outside_soft_limit_count)
        .sum();
    let edge_outside_hard_limit_count = clips
        .iter()
        .map(|clip| clip.edge_outside_hard_limit_count)
        .sum();
    let triangle_sample_count = clips.iter().map(|clip| clip.triangle_sample_count).sum();
    let triangle_area_collapse_count = clips
        .iter()
        .map(|clip| clip.triangle_area_collapse_count)
        .sum();
    let triangle_area_expansion_count = clips
        .iter()
        .map(|clip| clip.triangle_area_expansion_count)
        .sum();
    let world_normal_opposition_count = clips
        .iter()
        .map(|clip| clip.world_normal_opposition_count)
        .sum();
    let seam_pair_sample_count = clips.iter().map(|clip| clip.seam_pair_sample_count).sum();
    let seam_pair_violation_count = clips
        .iter()
        .map(|clip| clip.seam_pair_violation_count)
        .sum();
    let visible_anchor_motion_violation_count = clips
        .iter()
        .map(|clip| clip.visible_anchor_motion_violation_count)
        .sum();
    let paw_cluster_missing_count = clips
        .iter()
        .map(|clip| clip.paw_cluster_missing_count)
        .sum();
    let anchor_cluster_missing_count = clips
        .iter()
        .map(|clip| clip.anchor_cluster_missing_count)
        .sum();
    let paw_contact_violation_count = clips
        .iter()
        .map(|clip| clip.paw_contact_violation_count)
        .sum();
    let paw_side_violation_count = clips.iter().map(|clip| clip.paw_side_violation_count).sum();
    let clip_start_anchor_jump_violation_count = clips
        .iter()
        .map(|clip| clip.clip_start_anchor_jump_violation_count)
        .sum();
    let appendage_pair_required_count = clips
        .iter()
        .map(|clip| clip.appendage_pair_required_count)
        .sum();
    let appendage_pair_pass_count = clips
        .iter()
        .map(|clip| clip.appendage_pair_pass_count)
        .sum();
    let appendage_relative_motion_violation_count = clips
        .iter()
        .map(|clip| clip.appendage_relative_motion_violation_count)
        .sum();
    let soft_allowed = (edge_soft_sample_count as f64
        * f64::from(tolerances.edge_soft_max_fraction))
    .floor() as u64;
    let hard_allowed =
        (edge_sample_count as f64 * f64::from(tolerances.edge_hard_max_fraction)).floor() as u64;
    let collapse_allowed = (triangle_sample_count as f64
        * f64::from(tolerances.triangle_area_collapse_max_fraction))
    .floor() as u64;
    let expansion_allowed = (triangle_sample_count as f64
        * f64::from(tolerances.triangle_area_expansion_max_fraction))
    .floor() as u64;
    let seam_allowed = (seam_pair_sample_count as f64
        * f64::from(tolerances.seam_max_violation_fraction))
    .floor() as u64;
    let surface_seam_gate = if seam_pair_sample_count == 0 {
        SurfaceSeamGateReportV1 {
            schema_version: 1,
            sample_count: 0,
            violation_count: 0,
            legacy_allowed_count: 0,
            fail_on_any_seam_violation: tolerances.fail_on_any_seam_violation,
            status: "PASS_NO_SEAM_PAIRS".to_owned(),
            pass: true,
        }
    } else {
        evaluate_surface_seam_gate_v1(
            seam_pair_sample_count,
            seam_pair_violation_count,
            tolerances.seam_max_violation_fraction,
            tolerances.fail_on_any_seam_violation,
        )?
    };
    let required_clip_count = required_clips.len();
    let sampled_clip_count = clips.len();
    let inherited_clip_coverage = sampled_clip_count == required_clip_count;
    let required_joint_count = anchor_context.map_or(0, |context| context.required_roles.len());
    let joint_clip_required_count = anchor_context.map_or(0, |context| {
        context
            .required_roles
            .iter()
            .map(|role| {
                context
                    .dynamic_clips_by_role
                    .get(role)
                    .map_or(0, |dynamic| {
                        required_clips
                            .iter()
                            .filter(|required| {
                                dynamic
                                    .iter()
                                    .any(|clip| clip.eq_ignore_ascii_case(required))
                            })
                            .count()
                    })
            })
            .sum()
    });
    let joint_clip_pass_count = joint_clip_coverage_matrix
        .iter()
        .filter(|cell| cell.pass)
        .count();
    let joint_clip_coverage = joint_clip_coverage_matrix.len() == joint_clip_required_count
        && joint_clip_pass_count == joint_clip_required_count;
    let visible_motion_coverage = joint_clip_coverage
        && visible_anchor_motion_violation_count == 0
        && anchor_cluster_missing_count == 0;
    let per_clip_geometry_coverage = clips.iter().all(clip_has_geometry_domain_v3);
    let per_component_geometry_coverage = clips
        .iter()
        .all(|clip| clip.per_component_geometry_coverage);
    let per_component_deformation_coverage = clips
        .iter()
        .all(|clip| clip.per_component_deformation_coverage);
    let per_clip_deformation_coverage = clips
        .iter()
        .all(|clip| clip_is_within_local_deformation_budget_v3(clip, tolerances));
    let clip_start_anchor_continuity = clip_start_anchor_jump_violation_count == 0;
    let appendage_relative_motion_coverage = appendage_pair_pass_count
        == appendage_pair_required_count
        && appendage_relative_motion_violation_count == 0;
    let pass = inherited_clip_coverage
        && visible_motion_coverage
        && per_clip_geometry_coverage
        && per_component_geometry_coverage
        && per_component_deformation_coverage
        && per_clip_deformation_coverage
        && clip_start_anchor_continuity
        && appendage_relative_motion_coverage
        && edge_outside_hard_limit_count <= hard_allowed
        && edge_outside_soft_limit_count <= soft_allowed
        && triangle_area_collapse_count <= collapse_allowed
        && triangle_area_expansion_count <= expansion_allowed
        && surface_seam_gate.pass
        && visible_anchor_motion_violation_count == 0
        && anchor_cluster_missing_count == 0
        && paw_cluster_missing_count == 0
        && paw_contact_violation_count == 0
        && paw_side_violation_count == 0;
    Ok(InheritedMotionQualityReportV1 {
        schema_version: 2,
        profile: if anchor_context.is_some_and(|context| !context.required_roles.is_empty()) {
            "REFERENCE_SUPERMODEL_FULL_JOINT_CLIP_SURFACE_COVERAGE_V6"
        } else {
            "REFERENCE_SUPERMODEL_SAMPLED_SURFACE_V3"
        }
        .to_owned(),
        status: if pass { "PASS" } else { "BLOCKED" }.to_owned(),
        clips,
        edge_sample_count,
        edge_soft_sample_count,
        edge_outside_soft_limit_count,
        edge_outside_soft_allowed_count: soft_allowed,
        edge_outside_hard_limit_count,
        edge_outside_hard_allowed_count: hard_allowed,
        triangle_sample_count,
        triangle_area_collapse_count,
        triangle_area_collapse_allowed_count: collapse_allowed,
        triangle_area_expansion_count,
        triangle_area_expansion_allowed_count: expansion_allowed,
        world_normal_opposition_count,
        // A world-space dot product changes sign under a legitimate rigid
        // rotation. Retail c_dog proves this metric cannot be blocking.
        world_normal_opposition_diagnostic_only: true,
        seam_pair_sample_count,
        seam_pair_violation_count,
        seam_pair_allowed_count: seam_allowed,
        surface_seam_gate,
        visible_anchor_motion_violation_count,
        anchor_cluster_missing_count,
        paw_cluster_missing_count,
        paw_contact_violation_count,
        paw_side_violation_count,
        clip_start_anchor_jump_violation_count,
        paw_motion_diagnostics_only: false,
        required_clip_count,
        sampled_clip_count,
        inherited_clip_coverage,
        required_joint_count,
        joint_clip_required_count,
        joint_clip_pass_count,
        joint_clip_coverage,
        visible_motion_coverage,
        per_clip_geometry_coverage,
        per_clip_deformation_coverage,
        clip_start_anchor_continuity,
        per_component_geometry_coverage,
        per_component_deformation_coverage,
        appendage_relative_motion_coverage,
        appendage_pair_required_count,
        appendage_pair_pass_count,
        appendage_relative_motion_violation_count,
        joint_clip_coverage_matrix,
    })
}

fn clip_has_geometry_domain_v3(clip: &InheritedMotionClipQualityV1) -> bool {
    clip.edge_sample_count > 0 && clip.triangle_sample_count > 0 && !clip.sampled_times.is_empty()
}

fn clip_is_within_local_deformation_budget_v3(
    clip: &InheritedMotionClipQualityV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> bool {
    let clip_hard_allowed = (clip.edge_sample_count as f64
        * f64::from(tolerances.edge_hard_max_fraction))
    .floor() as u64;
    let clip_soft_allowed = (clip.edge_soft_sample_count as f64
        * f64::from(tolerances.edge_soft_max_fraction))
    .floor() as u64;
    let clip_collapse_allowed = (clip.triangle_sample_count as f64
        * f64::from(tolerances.triangle_area_collapse_max_fraction))
    .floor() as u64;
    let clip_expansion_allowed = (clip.triangle_sample_count as f64
        * f64::from(tolerances.triangle_area_expansion_max_fraction))
    .floor() as u64;

    // Density-normalized budgets catch widespread damage. A second absolute
    // fence catches a single catastrophic spike that a large mesh could
    // otherwise dilute. The emergency range is deliberately wider than the
    // normal hard range and is derived from the same profile values.
    let absolute_edge_min = tolerances.edge_hard_min_ratio.powi(2);
    let absolute_edge_max = tolerances.edge_hard_max_ratio.powi(2);
    let absolute_triangle_min = tolerances.triangle_min_area_ratio.powi(2);
    let absolute_triangle_max = tolerances.triangle_max_area_ratio.powi(2);
    let triangle_absolute_limits = clip
        .worst_triangle_area_collapse
        .as_ref()
        .is_none_or(|sample| sample.area_ratio >= absolute_triangle_min)
        && clip
            .worst_triangle_area_expansion
            .as_ref()
            .is_none_or(|sample| sample.area_ratio <= absolute_triangle_max);

    clip.edge_outside_hard_limit_count <= clip_hard_allowed
        && clip.edge_outside_soft_limit_count <= clip_soft_allowed
        && clip.triangle_area_collapse_count <= clip_collapse_allowed
        && clip.triangle_area_expansion_count <= clip_expansion_allowed
        && clip.min_edge_ratio >= absolute_edge_min
        && clip.max_edge_ratio <= absolute_edge_max
        && triangle_absolute_limits
}

fn component_is_within_local_deformation_budget_v1(
    component: &InheritedMotionComponentQualityV1,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> bool {
    let hard_allowed = (component.edge_sample_count as f64
        * f64::from(tolerances.edge_hard_max_fraction))
    .floor() as u64;
    let collapse_allowed = (component.triangle_sample_count as f64
        * f64::from(tolerances.triangle_area_collapse_max_fraction))
    .floor() as u64;
    let expansion_allowed = (component.triangle_sample_count as f64
        * f64::from(tolerances.triangle_area_expansion_max_fraction))
    .floor() as u64;
    component.edge_sample_count > 0
        && component.triangle_sample_count > 0
        && component.edge_outside_hard_limit_count <= hard_allowed
        && component.triangle_area_collapse_count <= collapse_allowed
        && component.triangle_area_expansion_count <= expansion_allowed
        && component.min_edge_ratio >= tolerances.edge_hard_min_ratio.powi(2)
        && component.max_edge_ratio <= tolerances.edge_hard_max_ratio.powi(2)
        && component.min_triangle_area_ratio >= tolerances.triangle_min_area_ratio.powi(2)
        && component.max_triangle_area_ratio <= tolerances.triangle_max_area_ratio.powi(2)
}

fn evaluate_appendage_relative_motion_v1(
    parent_role: &str,
    child_role: &str,
    parent: &[VisibleAnchorTrajectorySampleV1],
    child: &[VisibleAnchorTrajectorySampleV1],
    minimum_amplitude_ratio: f32,
    minimum_trajectory_alignment: f32,
) -> Result<AppendageRelativeMotionDiagnosticV1, ReferenceSupermodelMotionErrorV2> {
    if parent.len() != child.len() || parent.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-APPENDAGE-SAMPLE-MISMATCH",
            "motionQuality.appendageRelativeMotion",
            "appendage parent and child require the same non-empty sample-time domain",
        ));
    }
    let surface_relative = parent
        .iter()
        .zip(child)
        .map(|(parent, child)| sub(child.surface_centroid, parent.surface_centroid))
        .collect::<Vec<_>>();
    let controller_relative = parent
        .iter()
        .zip(child)
        .map(|(parent, child)| sub(child.controller_centroid, parent.controller_centroid))
        .collect::<Vec<_>>();
    let surface_origin = surface_relative[0];
    let controller_origin = controller_relative[0];
    let surface_deltas = surface_relative
        .iter()
        .map(|sample| sub(*sample, surface_origin))
        .collect::<Vec<_>>();
    let controller_deltas = controller_relative
        .iter()
        .map(|sample| sub(*sample, controller_origin))
        .collect::<Vec<_>>();
    let surface_relative_amplitude = surface_deltas
        .iter()
        .map(|value| dot(*value, *value).sqrt())
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    let controller_relative_amplitude = controller_deltas
        .iter()
        .map(|value| dot(*value, *value).sqrt())
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    let amplitude_ratio = if controller_relative_amplitude > 1.0e-7 {
        surface_relative_amplitude / controller_relative_amplitude
    } else {
        1.0
    };
    let numerator = surface_deltas
        .iter()
        .zip(&controller_deltas)
        .map(|(surface, controller)| dot(*surface, *controller))
        .sum::<f32>();
    let surface_energy = surface_deltas
        .iter()
        .map(|value| dot(*value, *value))
        .sum::<f32>();
    let controller_energy = controller_deltas
        .iter()
        .map(|value| dot(*value, *value))
        .sum::<f32>();
    let trajectory_alignment = if surface_energy > 1.0e-12 && controller_energy > 1.0e-12 {
        (numerator / (surface_energy * controller_energy).sqrt()).clamp(-1.0, 1.0)
    } else if controller_relative_amplitude <= 1.0e-7 {
        1.0
    } else {
        0.0
    };
    let pass = controller_relative_amplitude <= 1.0e-7
        || (surface_relative_amplitude > 1.0e-7
            && amplitude_ratio >= minimum_amplitude_ratio
            && trajectory_alignment >= minimum_trajectory_alignment);
    Ok(AppendageRelativeMotionDiagnosticV1 {
        parent_role: parent_role.to_owned(),
        child_role: child_role.to_owned(),
        surface_relative_amplitude,
        controller_relative_amplitude,
        amplitude_ratio,
        trajectory_alignment,
        pass,
    })
}

fn animation_sample_times_v3(clip: &crate::mdl::AnimationReport) -> Vec<f32> {
    // Fixed phases alone miss short-lived controller extrema. Include exact
    // controller/event keys and deterministic interval midpoints, then apply a
    // bounded stratified reduction for very dense retail clips.
    let fixed = [0.0_f32, 0.25, 0.5, 0.75, 1.0]
        .into_iter()
        .map(|fraction| clip.length * fraction)
        .collect::<Vec<_>>();
    let mut mandatory = fixed.clone();
    mandatory.extend(
        clip.events
            .iter()
            .map(|event| event.time.clamp(0.0, clip.length)),
    );
    let mut keys = mandatory.clone();
    keys.extend(
        clip.events
            .iter()
            .map(|event| event.time.clamp(0.0, clip.length)),
    );
    collect_animation_controller_times(&clip.node_tree.roots, clip.length, &mut keys);
    keys.sort_by(f32::total_cmp);
    keys.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-6);
    let mut adaptive = keys.clone();
    adaptive.extend(
        keys.windows(2)
            .filter(|pair| pair[1] - pair[0] > 1.0e-5)
            .map(|pair| (pair[0] + pair[1]) * 0.5),
    );
    adaptive.sort_by(f32::total_cmp);
    adaptive.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-6);
    // Full-surface deformation is the expensive operation. Five fixed phases
    // remain mandatory, exact animation events are never discarded, and the
    // remaining budget is filled from controller keys/midpoints by temporal
    // farthest-point sampling. This gives deterministic extrema coverage
    // without turning a 42-clip, 300k-triangle audit into thousands of full
    // mesh evaluations.
    const MAXIMUM_SAMPLES: usize = 9;
    bounded_adaptive_sample_times_v1(mandatory, adaptive, MAXIMUM_SAMPLES)
}

fn bounded_adaptive_sample_times_v1(
    mut mandatory: Vec<f32>,
    mut candidates: Vec<f32>,
    maximum_samples: usize,
) -> Vec<f32> {
    let canonicalize = |values: &mut Vec<f32>| {
        values.retain(|value| value.is_finite());
        values.sort_by(f32::total_cmp);
        values.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-6);
    };
    canonicalize(&mut mandatory);
    canonicalize(&mut candidates);
    if mandatory.len() >= maximum_samples {
        return mandatory;
    }
    while mandatory.len() < maximum_samples {
        let Some(best) = candidates
            .iter()
            .copied()
            .filter(|candidate| {
                !mandatory
                    .iter()
                    .any(|selected| (*selected - *candidate).abs() <= 1.0e-6)
            })
            .max_by(|left, right| {
                let clearance = |candidate: f32| {
                    mandatory
                        .iter()
                        .map(|selected| (candidate - *selected).abs())
                        .min_by(f32::total_cmp)
                        .unwrap_or(f32::INFINITY)
                };
                clearance(*left)
                    .total_cmp(&clearance(*right))
                    .then_with(|| right.total_cmp(left))
            })
        else {
            break;
        };
        mandatory.push(best);
    }
    canonicalize(&mut mandatory);
    mandatory
}

fn collect_animation_controller_times(
    nodes: &[crate::mdl::NodeReport],
    clip_length: f32,
    output: &mut Vec<f32>,
) {
    for node in nodes {
        for controller in &node.controllers {
            output.extend(
                controller
                    .times
                    .iter()
                    .copied()
                    .filter(|time| time.is_finite())
                    .map(|time| time.clamp(0.0, clip_length)),
            );
        }
        collect_animation_controller_times(&node.children, clip_length, output);
    }
}

#[derive(Clone, Debug)]
struct SkinGeometrySamplingPlanV3 {
    sampled_face_indices: Vec<usize>,
    component_by_vertex: Vec<usize>,
    raw_component_count: usize,
}

#[derive(Clone, Debug)]
struct GeometrySamplingPlanV3 {
    by_skin_part: BTreeMap<u32, SkinGeometrySamplingPlanV3>,
    seam_pairs: Option<Vec<SeamPairSamplingPlanV3>>,
    total_component_count: usize,
}

#[derive(Clone, Debug)]
struct SeamPairSamplingPlanV3 {
    left_skin_part: u32,
    left_vertex: usize,
    right_skin_part: u32,
    right_vertex: usize,
    output_limit: f32,
}

#[derive(Clone, Debug, Default)]
struct MotionWeightViolationAccumulatorV1 {
    /// Renderer-node name plus its local edge. The MDL writer names every
    /// owned render segment `m2a_seg_<segment_id>`, so this remains an exact
    /// mapping back to the conversion IR without relying on traversal order.
    edge_penalties: BTreeMap<(String, usize, usize), u64>,
    /// Exact clip/component attribution for ordinary hard-edge violations.
    /// The global edge map is still required for whole-model diagnostics, but
    /// per-component density repair must not smooth unrelated passing islands.
    component_edge_penalties: BTreeMap<(String, usize, String, usize, usize), u64>,
    failed_clip_components: BTreeSet<(String, usize)>,
    /// Subset of measured edges that crosses the independent catastrophic
    /// local fence. These need direct endpoint coherence; aggregate Laplacian
    /// smoothing alone can leave one isolated weight cliff behind.
    catastrophic_collapse_edge_penalties: BTreeMap<(String, usize, usize), u64>,
    catastrophic_expansion_edge_penalties: BTreeMap<(String, usize, usize), u64>,
    catastrophic_collapse_edge_witnesses:
        BTreeMap<(String, usize, usize), BTreeMap<String, CatastrophicEdgeMotionWitnessV1>>,
    catastrophic_expansion_edge_witnesses:
        BTreeMap<(String, usize, usize), BTreeMap<String, CatastrophicEdgeMotionWitnessV1>>,
    catastrophic_collapse_triangle_penalties: BTreeMap<(String, [usize; 3]), u64>,
    catastrophic_expansion_triangle_penalties: BTreeMap<(String, [usize; 3]), u64>,
    /// Renderer-node name plus one local triangle whose area left the motion
    /// envelope. Edge locality alone is insufficient here: three vertices can
    /// all reference one valid parent-child pair while still carrying sharply
    /// different blend values that turn a microscopic source triangle into a
    /// long runtime spike.
    triangle_penalties: BTreeMap<(String, [usize; 3]), u64>,
    component_triangle_penalties: BTreeMap<(String, usize, String, [usize; 3]), u64>,
}

#[derive(Clone, Debug, PartialEq)]
struct CatastrophicEdgeMotionWitnessV1 {
    penalty: u64,
    clip_name: String,
    sample_time_seconds: f32,
    length_ratio: f32,
    bind_endpoints: [[f32; 3]; 2],
    sampled_endpoints: [[f32; 3]; 2],
}

impl MotionWeightViolationAccumulatorV1 {
    fn add_edge(&mut self, node_name: &str, left: usize, right: usize, penalty: u64) {
        if left == right || penalty == 0 {
            return;
        }
        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        *self
            .edge_penalties
            .entry((node_name.to_owned(), left, right))
            .or_default() += penalty;
    }

    fn add_component_edge(
        &mut self,
        clip_name: &str,
        component_index: usize,
        node_name: &str,
        left: usize,
        right: usize,
        penalty: u64,
    ) {
        if left == right || penalty == 0 {
            return;
        }
        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        *self
            .component_edge_penalties
            .entry((
                clip_name.to_owned(),
                component_index,
                node_name.to_owned(),
                left,
                right,
            ))
            .or_default() += penalty;
    }

    fn mark_failed_component(&mut self, clip_name: &str, component_index: usize) {
        self.failed_clip_components
            .insert((clip_name.to_owned(), component_index));
    }

    fn failed_component_subset_v1(&self) -> Self {
        let mut output = Self::default();
        output.failed_clip_components = self.failed_clip_components.clone();
        for ((clip_name, component_index, node_name, left, right), penalty) in
            &self.component_edge_penalties
        {
            if self
                .failed_clip_components
                .contains(&(clip_name.clone(), *component_index))
            {
                output.add_edge(node_name, *left, *right, *penalty);
            }
        }
        for ((clip_name, component_index, node_name, vertices), penalty) in
            &self.component_triangle_penalties
        {
            if self
                .failed_clip_components
                .contains(&(clip_name.clone(), *component_index))
            {
                *output
                    .triangle_penalties
                    .entry((node_name.clone(), *vertices))
                    .or_default() += *penalty;
            }
        }
        output
    }

    fn add_catastrophic_edge(
        &mut self,
        node_name: &str,
        left: usize,
        right: usize,
        penalty: u64,
        expansion: bool,
    ) {
        if left == right || penalty == 0 {
            return;
        }
        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        let penalties = if expansion {
            &mut self.catastrophic_expansion_edge_penalties
        } else {
            &mut self.catastrophic_collapse_edge_penalties
        };
        penalties
            .entry((node_name.to_owned(), left, right))
            .and_modify(|current| *current = (*current).max(penalty))
            .or_insert(penalty);
    }

    #[allow(clippy::too_many_arguments)]
    fn add_catastrophic_edge_witness(
        &mut self,
        node_name: &str,
        left: usize,
        right: usize,
        penalty: u64,
        expansion: bool,
        clip_name: &str,
        sample_time_seconds: f32,
        length_ratio: f32,
        mut bind_endpoints: [[f32; 3]; 2],
        mut sampled_endpoints: [[f32; 3]; 2],
    ) {
        if left == right
            || penalty == 0
            || !sample_time_seconds.is_finite()
            || !length_ratio.is_finite()
        {
            return;
        }
        let (left, right) = if left < right {
            (left, right)
        } else {
            bind_endpoints.swap(0, 1);
            sampled_endpoints.swap(0, 1);
            (right, left)
        };
        self.add_catastrophic_edge(node_name, left, right, penalty, expansion);
        let key = (node_name.to_owned(), left, right);
        let witnesses = if expansion {
            &mut self.catastrophic_expansion_edge_witnesses
        } else {
            &mut self.catastrophic_collapse_edge_witnesses
        };
        let candidate = CatastrophicEdgeMotionWitnessV1 {
            penalty,
            clip_name: clip_name.to_owned(),
            sample_time_seconds,
            length_ratio,
            bind_endpoints,
            sampled_endpoints,
        };
        // Retain the worst sample independently for every clip. One edge can
        // reach the same global extremum under several distinct controller
        // transforms; repairing only one of them leaves the global maximum
        // penalty unchanged and causes the transactional backtracker to
        // discard an otherwise useful endpoint change.
        let by_clip = witnesses.entry(key).or_default();
        let replace = by_clip.get(clip_name).is_none_or(|current| {
            candidate.penalty > current.penalty
                || (candidate.penalty == current.penalty
                    && if expansion {
                        candidate.length_ratio > current.length_ratio
                    } else {
                        candidate.length_ratio < current.length_ratio
                    })
                || (candidate.penalty == current.penalty
                    && candidate.length_ratio.to_bits() == current.length_ratio.to_bits()
                    && candidate.sample_time_seconds.to_bits()
                        < current.sample_time_seconds.to_bits())
        });
        if replace {
            by_clip.insert(clip_name.to_owned(), candidate);
        }
    }

    fn add_triangle(&mut self, node_name: &str, vertices: [usize; 3], penalty: u64) {
        if penalty == 0 {
            return;
        }
        let mut canonical = vertices;
        canonical.sort_unstable();
        if canonical[0] == canonical[2] {
            return;
        }
        *self
            .triangle_penalties
            .entry((node_name.to_owned(), canonical))
            .or_default() += penalty;
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            self.add_edge(node_name, vertices[left], vertices[right], penalty);
        }
    }

    fn add_catastrophic_triangle(
        &mut self,
        node_name: &str,
        vertices: [usize; 3],
        penalty: u64,
        expansion: bool,
    ) {
        if penalty == 0 {
            return;
        }
        let mut canonical = vertices;
        canonical.sort_unstable();
        if canonical[0] == canonical[2] {
            return;
        }
        let penalties = if expansion {
            &mut self.catastrophic_expansion_triangle_penalties
        } else {
            &mut self.catastrophic_collapse_triangle_penalties
        };
        penalties
            .entry((node_name.to_owned(), canonical))
            .and_modify(|current| *current = (*current).max(penalty))
            .or_insert(penalty);
    }

    fn add_component_triangle(
        &mut self,
        clip_name: &str,
        component_index: usize,
        node_name: &str,
        vertices: [usize; 3],
        penalty: u64,
    ) {
        if penalty == 0 {
            return;
        }
        let mut canonical = vertices;
        canonical.sort_unstable();
        if canonical[0] == canonical[2] {
            return;
        }
        *self
            .component_triangle_penalties
            .entry((
                clip_name.to_owned(),
                component_index,
                node_name.to_owned(),
                canonical,
            ))
            .or_default() += penalty;
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            self.add_component_edge(
                clip_name,
                component_index,
                node_name,
                vertices[left],
                vertices[right],
                penalty,
            );
        }
    }
}

#[derive(Clone, Debug, Default)]
struct MotionWeightRefinementReportV1 {
    applied: bool,
    attempted_round_count: usize,
    examined_edge_count: usize,
    measured_triangle_count: usize,
    smoothing_candidate_group_count: usize,
    smoothing_shared_support_group_count: usize,
    smoothing_changed_group_count: usize,
    micro_triangle_candidate_count: usize,
    micro_triangle_too_large_count: usize,
    micro_triangle_protected_support_count: usize,
    micro_triangle_nonlocal_support_count: usize,
    micro_triangle_no_common_support_count: usize,
    micro_triangle_changed_count: usize,
    catastrophic_edge_candidate_count: usize,
    catastrophic_edge_count: usize,
    catastrophic_edge_topology_skip_count: usize,
    catastrophic_edge_conflict_skip_count: usize,
    catastrophic_anchor_restore_count: usize,
    catastrophic_peak_edge: Option<String>,
    catastrophic_peak_edge_changed: bool,
    backtracked_group_count: usize,
    iteration_count: usize,
    edge_count: usize,
    rigid_component_count: usize,
    projected_triangle_count: usize,
    rejected_round_count: usize,
    last_rejection: Option<String>,
}

fn record_motion_weight_refinement_attempt_v1(
    total: &mut MotionWeightRefinementReportV1,
    attempted: &MotionWeightRefinementReportV1,
) {
    total.attempted_round_count += 1;
    total.examined_edge_count += attempted.edge_count;
    total.measured_triangle_count += attempted.measured_triangle_count;
    total.smoothing_candidate_group_count += attempted.smoothing_candidate_group_count;
    total.smoothing_shared_support_group_count += attempted.smoothing_shared_support_group_count;
    total.smoothing_changed_group_count += attempted.smoothing_changed_group_count;
    total.micro_triangle_candidate_count += attempted.micro_triangle_candidate_count;
    total.micro_triangle_too_large_count += attempted.micro_triangle_too_large_count;
    total.micro_triangle_protected_support_count +=
        attempted.micro_triangle_protected_support_count;
    total.micro_triangle_nonlocal_support_count += attempted.micro_triangle_nonlocal_support_count;
    total.micro_triangle_no_common_support_count +=
        attempted.micro_triangle_no_common_support_count;
    total.micro_triangle_changed_count += attempted.micro_triangle_changed_count;
    total.catastrophic_edge_candidate_count += attempted.catastrophic_edge_candidate_count;
    total.catastrophic_edge_count += attempted.catastrophic_edge_count;
    total.catastrophic_edge_topology_skip_count += attempted.catastrophic_edge_topology_skip_count;
    total.catastrophic_edge_conflict_skip_count += attempted.catastrophic_edge_conflict_skip_count;
    total.catastrophic_anchor_restore_count += attempted.catastrophic_anchor_restore_count;
    if attempted.catastrophic_peak_edge.is_some() {
        total.catastrophic_peak_edge = attempted.catastrophic_peak_edge.clone();
        total.catastrophic_peak_edge_changed = attempted.catastrophic_peak_edge_changed;
    }
    total.backtracked_group_count += attempted.backtracked_group_count;
}

fn copy_motion_weight_refinement_attempt_diagnostics_v1(
    target: &mut MotionWeightRefinementReportV1,
    source: &MotionWeightRefinementReportV1,
) {
    target.attempted_round_count = source.attempted_round_count;
    target.examined_edge_count = source.examined_edge_count;
    target.measured_triangle_count = source.measured_triangle_count;
    target.smoothing_candidate_group_count = source.smoothing_candidate_group_count;
    target.smoothing_shared_support_group_count = source.smoothing_shared_support_group_count;
    target.smoothing_changed_group_count = source.smoothing_changed_group_count;
    target.micro_triangle_candidate_count = source.micro_triangle_candidate_count;
    target.micro_triangle_too_large_count = source.micro_triangle_too_large_count;
    target.micro_triangle_protected_support_count = source.micro_triangle_protected_support_count;
    target.micro_triangle_nonlocal_support_count = source.micro_triangle_nonlocal_support_count;
    target.micro_triangle_no_common_support_count = source.micro_triangle_no_common_support_count;
    target.micro_triangle_changed_count = source.micro_triangle_changed_count;
    target.catastrophic_edge_candidate_count = source.catastrophic_edge_candidate_count;
    target.catastrophic_edge_count = source.catastrophic_edge_count;
    target.catastrophic_edge_topology_skip_count = source.catastrophic_edge_topology_skip_count;
    target.catastrophic_edge_conflict_skip_count = source.catastrophic_edge_conflict_skip_count;
    target.catastrophic_anchor_restore_count = source.catastrophic_anchor_restore_count;
    target.catastrophic_peak_edge = source.catastrophic_peak_edge.clone();
    target.catastrophic_peak_edge_changed = source.catastrophic_peak_edge_changed;
    target.backtracked_group_count = source.backtracked_group_count;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct MicroTriangleRefinementReportV1 {
    candidate_count: usize,
    too_large_count: usize,
    protected_support_count: usize,
    nonlocal_support_count: usize,
    no_common_support_count: usize,
    changed_count: usize,
}

/// Builds one density-independent topology plan and reuses it for every clip
/// and time sample. Every connected render component contributes a
/// deterministic stratified face sample; isolated vertices or empty
/// components are rejected instead of being counted as measured geometry.
fn build_geometry_sampling_plan_v3(
    target: &InspectionReport,
) -> Result<GeometrySamplingPlanV3, ReferenceSupermodelMotionErrorV2> {
    let mut by_skin_part = BTreeMap::new();
    for node in flatten_nodes_in_tree_order(&target.node_tree.roots) {
        let (Some(mesh), Some(skin)) = (&node.mesh, &node.skin) else {
            continue;
        };
        let vertex_count = skin.vertex_weights.len();
        if vertex_count == 0 || mesh.faces.is_empty() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-GEOMETRY-DOMAIN-EMPTY",
                format!("target.nodeTree.nodes[{}]", node.number),
                format!("skin node {:?} has no indexed triangle domain", node.name),
            ));
        }
        let mut union = UnionFind::new(vertex_count);
        for face in &mesh.faces {
            let indices = face.vertex_indices.map(usize::from);
            if indices.iter().any(|index| *index >= vertex_count) {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-FACE-INDEX-OOB",
                    "target.nodeTree.mesh.faces",
                    format!("skin node {:?} contains an out-of-range face", node.name),
                ));
            }
            union.union(indices[0], indices[1]);
            union.union(indices[1], indices[2]);
        }
        let mut component_ordinal = BTreeMap::<usize, usize>::new();
        let mut component_by_vertex = Vec::with_capacity(vertex_count);
        for vertex in 0..vertex_count {
            let root = union.find(vertex);
            let next = component_ordinal.len();
            let ordinal = *component_ordinal.entry(root).or_insert(next);
            component_by_vertex.push(ordinal);
        }
        let mut faces_by_component = BTreeMap::<usize, Vec<usize>>::new();
        for (face_index, face) in mesh.faces.iter().enumerate() {
            let component = component_by_vertex[usize::from(face.vertex_indices[0])];
            faces_by_component
                .entry(component)
                .or_default()
                .push(face_index);
        }
        if faces_by_component.len() != component_ordinal.len() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-COMPONENT-DOMAIN-EMPTY",
                format!("target.nodeTree.nodes[{}].mesh", node.number),
                format!(
                    "skin node {:?} exposes {} connected components but only {} contain triangles",
                    node.name,
                    component_ordinal.len(),
                    faces_by_component.len()
                ),
            ));
        }
        let mut sampled_face_indices = Vec::new();
        for faces in faces_by_component.values() {
            let desired = (((faces.len() as f64).sqrt().ceil() as usize) * 4)
                .clamp(8, 2_048)
                .min(faces.len());
            for sample_index in 0..desired {
                sampled_face_indices.push(faces[sample_index * faces.len() / desired]);
            }
        }
        sampled_face_indices.sort_unstable();
        sampled_face_indices.dedup();
        if sampled_face_indices.is_empty() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-GEOMETRY-SAMPLE-EMPTY",
                format!("target.nodeTree.nodes[{}].mesh", node.number),
                "deterministic component sampling produced no faces",
            ));
        }
        by_skin_part.insert(
            node.number,
            SkinGeometrySamplingPlanV3 {
                sampled_face_indices,
                component_by_vertex,
                raw_component_count: component_ordinal.len(),
            },
        );
    }
    if by_skin_part.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-GEOMETRY-DOMAIN-EMPTY",
            "target.nodeTree",
            "motion quality requires at least one skinned render surface",
        ));
    }
    let total_component_count = by_skin_part
        .values()
        .map(|plan| plan.raw_component_count)
        .sum();
    Ok(GeometrySamplingPlanV3 {
        by_skin_part,
        seam_pairs: None,
        total_component_count,
    })
}

fn build_logical_component_remap_v1(
    bind: &[[f32; 3]],
    raw_components: &[usize],
    one_ring_position_keys: &[BTreeSet<[u32; 3]>],
    raw_component_count: usize,
) -> Result<(Vec<usize>, usize), ReferenceSupermodelMotionErrorV2> {
    if raw_component_count == 0
        || bind.len() != raw_components.len()
        || bind.len() != one_ring_position_keys.len()
        || bind.is_empty()
        || raw_components
            .iter()
            .any(|component| *component >= raw_component_count)
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-LOGICAL-COMPONENT-DOMAIN",
            "motionQuality.geometrySamplingPlan.logicalComponents",
            "logical component remapping requires a non-empty, aligned raw topology domain",
        ));
    }

    let mut union = UnionFind::new(raw_component_count);
    let mut buckets = BTreeMap::<[u32; 3], Vec<usize>>::new();
    for (index, point) in bind.iter().enumerate() {
        buckets
            .entry(point.map(f32::to_bits))
            .or_default()
            .push(index);
    }
    for members in buckets.into_values() {
        for left in 0..members.len() {
            for right in (left + 1)..members.len() {
                let left_index = members[left];
                let right_index = members[right];
                let left_component = raw_components[left_index];
                let right_component = raw_components[right_index];
                if left_component != right_component
                    && !one_ring_position_keys[left_index]
                        .is_disjoint(&one_ring_position_keys[right_index])
                {
                    union.union(left_component, right_component);
                }
            }
        }
    }

    let mut ordinal_by_root = BTreeMap::<usize, usize>::new();
    let mut raw_to_logical = Vec::with_capacity(raw_component_count);
    for raw_component in 0..raw_component_count {
        let root = union.find(raw_component);
        let next = ordinal_by_root.len();
        raw_to_logical.push(*ordinal_by_root.entry(root).or_insert(next));
    }
    Ok((raw_to_logical, ordinal_by_root.len()))
}

fn finalize_geometry_sampling_plan_v3(
    target: &InspectionReport,
    sample: &crate::mdl::SkinDeformationSampleV1,
    sampling_plan: &mut GeometrySamplingPlanV3,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    if sampling_plan.seam_pairs.is_some() {
        return Ok(());
    }

    let mut all_bind = Vec::new();
    let mut all_raw_components = Vec::new();
    let mut all_origins = Vec::new();
    let mut all_one_ring_position_keys = Vec::new();
    let mut raw_component_offset_by_part = BTreeMap::<u32, usize>::new();
    let mut next_raw_component = 0usize;

    for (&skin_part, plan) in &sampling_plan.by_skin_part {
        let skin = sample
            .skins
            .iter()
            .find(|skin| skin.node_part == skin_part)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-GEOMETRY-PLAN-SKIN-MISSING",
                    "motionQuality.geometrySamplingPlan.logicalComponents",
                    format!("bind sample omits planned skin node part {skin_part}"),
                )
            })?;
        let node = find_node_by_part(&target.node_tree.roots, skin_part).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-SKIN-NODE-MISSING",
                "target.nodeTree",
                format!("skin node part {skin_part} is absent"),
            )
        })?;
        let mesh = node.mesh.as_ref().ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-SKIN-MESH-MISSING",
                "target.nodeTree.mesh",
                format!("skin node {} has no mesh", skin.node_name),
            )
        })?;
        if plan.component_by_vertex.len() != skin.vertices.len() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-GEOMETRY-PLAN-MISMATCH",
                "motionQuality.geometrySamplingPlan.logicalComponents",
                format!(
                    "skin node {} sampled {} vertices but its plan contains {}",
                    skin.node_name,
                    skin.vertices.len(),
                    plan.component_by_vertex.len()
                ),
            ));
        }

        raw_component_offset_by_part.insert(skin_part, next_raw_component);
        let mut one_ring_position_keys = vec![BTreeSet::<[u32; 3]>::new(); skin.vertices.len()];
        for face in &mesh.faces {
            let vertices = face.vertex_indices.map(usize::from);
            if vertices.iter().any(|vertex| *vertex >= skin.vertices.len()) {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-LOGICAL-COMPONENT-FACE-INDEX-OOB",
                    "target.nodeTree.mesh.faces",
                    "logical component topology contains an out-of-range vertex",
                ));
            }
            for corner in 0..3 {
                for neighbour in 0..3 {
                    if corner != neighbour {
                        one_ring_position_keys[vertices[corner]].insert(
                            skin.vertices[vertices[neighbour]]
                                .bind_world
                                .map(f32::to_bits),
                        );
                    }
                }
            }
        }
        for index in 0..skin.vertices.len() {
            all_bind.push(skin.vertices[index].bind_world);
            all_raw_components.push(next_raw_component + plan.component_by_vertex[index]);
            all_origins.push((skin_part, index));
            all_one_ring_position_keys.push(one_ring_position_keys[index].clone());
        }
        next_raw_component += plan.raw_component_count;
    }

    if next_raw_component != sampling_plan.total_component_count {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-LOGICAL-COMPONENT-COUNT-MISMATCH",
            "motionQuality.geometrySamplingPlan.logicalComponents",
            "raw component offsets do not match the sampling-plan component count",
        ));
    }

    let seam_pairs = build_cross_component_seam_plan_v3(
        &all_bind,
        &all_raw_components,
        &all_origins,
        &all_one_ring_position_keys,
        tolerances,
    );
    let (raw_to_logical, logical_component_count) = build_logical_component_remap_v1(
        &all_bind,
        &all_raw_components,
        &all_one_ring_position_keys,
        next_raw_component,
    )?;
    for (&skin_part, plan) in &mut sampling_plan.by_skin_part {
        let raw_offset = raw_component_offset_by_part[&skin_part];
        for component in &mut plan.component_by_vertex {
            *component = raw_to_logical[raw_offset + *component];
        }
    }
    sampling_plan.total_component_count = logical_component_count;
    sampling_plan.seam_pairs = Some(seam_pairs);
    Ok(())
}

fn inspect_sample_geometry(
    target: &InspectionReport,
    sample: &crate::mdl::SkinDeformationSampleV1,
    sampling_plan: &mut GeometrySamplingPlanV3,
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
    quality: &mut InheritedMotionClipQualityV1,
    mut weight_violations: Option<&mut MotionWeightViolationAccumulatorV1>,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    let flattened = flatten_nodes_in_tree_order(&target.node_tree.roots);
    let mut global_bind_min = [f32::INFINITY; 3];
    let mut global_bind_max = [f32::NEG_INFINITY; 3];
    for vertex in sample.skins.iter().flat_map(|skin| &skin.vertices) {
        for axis in 0..3 {
            global_bind_min[axis] = global_bind_min[axis].min(vertex.bind_world[axis]);
            global_bind_max[axis] = global_bind_max[axis].max(vertex.bind_world[axis]);
        }
    }
    let global_bind_diagonal = distance(global_bind_min, global_bind_max);
    if !global_bind_diagonal.is_finite() || global_bind_diagonal <= 1.0e-8 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-BIND-BOUNDS",
            "motionQuality.samples.bindBounds",
            "sampled skin geometry requires non-degenerate finite bind bounds",
        ));
    }
    let soft_edge_min_bind_length =
        global_bind_diagonal * tolerances.soft_edge_min_bind_diagonal_fraction;
    let triangle_min_bind_area = (global_bind_diagonal
        * global_bind_diagonal
        * tolerances.triangle_min_bind_area_diagonal_squared_fraction)
        .max(1.0e-8);
    for skin in &sample.skins {
        let node = find_node_by_part(&target.node_tree.roots, skin.node_part).ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-SKIN-NODE-MISSING",
                "target.nodeTree",
                format!("skin node part {} is absent", skin.node_part),
            )
        })?;
        let mesh = node.mesh.as_ref().ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-SKIN-MESH-MISSING",
                "target.nodeTree.mesh",
                format!("skin node {} has no mesh", skin.node_name),
            )
        })?;
        let skin_report = node.skin.as_ref().ok_or_else(|| {
            motion_error(
                "M2A-SUPERMODEL-MOTION-SKIN-REPORT-MISSING",
                "target.nodeTree.skin",
                format!("sampled skin node {} has no skin report", skin.node_name),
            )
        })?;
        let plan = sampling_plan
            .by_skin_part
            .get(&skin.node_part)
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-GEOMETRY-PLAN-MISSING",
                    "motionQuality.geometrySamplingPlan",
                    format!("skin node {} has no topology sampling plan", skin.node_name),
                )
            })?;
        if plan.component_by_vertex.len() != skin.vertices.len() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-GEOMETRY-PLAN-MISMATCH",
                "motionQuality.geometrySamplingPlan",
                format!(
                    "skin node {} sampled {} vertices but its plan contains {}",
                    skin.node_name,
                    skin.vertices.len(),
                    plan.component_by_vertex.len()
                ),
            ));
        }
        let vertex_influences = |indices: &[usize]| {
            indices
                .iter()
                .copied()
                .map(|index| {
                    let weights = skin_report.vertex_weights[index];
                    let references = skin_report.bone_references[index];
                    let influences = (0..4)
                        .filter_map(|lane| {
                            let reference = references[lane];
                            let weight = weights[lane];
                            if reference == u16::MAX || !weight.is_finite() || weight <= 0.0 {
                                return None;
                            }
                            let bone_node_name = skin_report
                                .node_to_bone_map
                                .iter()
                                .enumerate()
                                .find(|(_, mapped)| **mapped == reference as i16)
                                .and_then(|(ordinal, _)| flattened.get(ordinal))
                                .map(|node| node.name.clone())
                                .unwrap_or_else(|| {
                                    format!("unresolved_bone_reference_{reference}")
                                });
                            Some(TriangleBoneInfluenceDiagnosticV1 {
                                bone_node_name,
                                weight,
                            })
                        })
                        .collect();
                    TriangleVertexInfluenceDiagnosticV1 {
                        vertex_index: skin.vertices[index].vertex_index,
                        influences,
                    }
                })
                .collect()
        };
        for &face_index in &plan.sampled_face_indices {
            let face = &mesh.faces[face_index];
            let indices = face.vertex_indices.map(usize::from);
            if indices.iter().any(|index| *index >= skin.vertices.len()) {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-FACE-INDEX-OOB",
                    "target.nodeTree.mesh.faces",
                    "skin face index exceeds sampled vertex count",
                ));
            }
            let component_index = plan.component_by_vertex[indices[0]];
            let bind = indices.map(|index| skin.vertices[index].bind_world);
            let moved = indices.map(|index| skin.vertices[index].sampled_world);
            let bind_cross = triangle_cross(bind);
            let moved_cross = triangle_cross(moved);
            let bind_area = length(bind_cross);
            let moved_area = length(moved_cross);
            if bind_area > triangle_min_bind_area {
                quality.triangle_sample_count += 1;
                let ratio = moved_area / bind_area;
                {
                    let component = &mut quality.components[component_index];
                    component.triangle_sample_count += 1;
                    component.min_triangle_area_ratio =
                        component.min_triangle_area_ratio.min(ratio);
                    component.max_triangle_area_ratio =
                        component.max_triangle_area_ratio.max(ratio);
                }
                if ratio < tolerances.triangle_min_area_ratio {
                    quality.triangle_area_collapse_count += 1;
                    quality.components[component_index].triangle_area_collapse_count += 1;
                    if let Some(violations) = weight_violations.as_deref_mut() {
                        let penalty = motion_weight_violation_penalty_v1(
                            ratio,
                            tolerances.triangle_min_area_ratio,
                            tolerances.triangle_max_area_ratio,
                            4,
                        );
                        violations.add_triangle(&skin.node_name, indices, penalty);
                        if ratio < tolerances.triangle_min_area_ratio.powi(2) {
                            violations.add_catastrophic_triangle(
                                &skin.node_name,
                                indices,
                                penalty,
                                false,
                            );
                        }
                        violations.add_component_triangle(
                            &sample.clip_name,
                            component_index,
                            &skin.node_name,
                            indices,
                            penalty,
                        );
                    }
                    if quality
                        .worst_triangle_area_collapse
                        .as_ref()
                        .is_none_or(|worst| ratio < worst.area_ratio)
                    {
                        quality.worst_triangle_area_collapse = Some(TriangleAreaDiagnosticV1 {
                            skin_node_name: skin.node_name.clone(),
                            vertex_indices: indices.map(|index| skin.vertices[index].vertex_index),
                            vertex_influences: vertex_influences(&indices),
                            bind_centroid: std::array::from_fn(|axis| {
                                (bind[0][axis] + bind[1][axis] + bind[2][axis]) / 3.0
                            }),
                            area_ratio: ratio,
                            sample_time_seconds: sample.time_seconds,
                        });
                    }
                }
                if ratio > tolerances.triangle_max_area_ratio {
                    quality.triangle_area_expansion_count += 1;
                    quality.components[component_index].triangle_area_expansion_count += 1;
                    if let Some(violations) = weight_violations.as_deref_mut() {
                        let penalty = motion_weight_violation_penalty_v1(
                            ratio,
                            tolerances.triangle_min_area_ratio,
                            tolerances.triangle_max_area_ratio,
                            4,
                        );
                        violations.add_triangle(&skin.node_name, indices, penalty);
                        if ratio > tolerances.triangle_max_area_ratio.powi(2) {
                            violations.add_catastrophic_triangle(
                                &skin.node_name,
                                indices,
                                penalty,
                                true,
                            );
                        }
                        violations.add_component_triangle(
                            &sample.clip_name,
                            component_index,
                            &skin.node_name,
                            indices,
                            penalty,
                        );
                    }
                    if quality
                        .worst_triangle_area_expansion
                        .as_ref()
                        .is_none_or(|worst| ratio > worst.area_ratio)
                    {
                        quality.worst_triangle_area_expansion = Some(TriangleAreaDiagnosticV1 {
                            skin_node_name: skin.node_name.clone(),
                            vertex_indices: indices.map(|index| skin.vertices[index].vertex_index),
                            vertex_influences: vertex_influences(&indices),
                            bind_centroid: std::array::from_fn(|axis| {
                                (bind[0][axis] + bind[1][axis] + bind[2][axis]) / 3.0
                            }),
                            area_ratio: ratio,
                            sample_time_seconds: sample.time_seconds,
                        });
                    }
                }
                if moved_area > 1.0e-8 && dot(bind_cross, moved_cross) < 0.0 {
                    quality.world_normal_opposition_count += 1;
                }
            }
            for (left, right) in [(0, 1), (1, 2), (2, 0)] {
                let bind_length = distance(bind[left], bind[right]);
                if bind_length <= 1.0e-8 {
                    continue;
                }
                let ratio = distance(moved[left], moved[right]) / bind_length;
                quality.edge_sample_count += 1;
                let edge_indices = [indices[left], indices[right]];
                let edge_diagnostic = || EdgeLengthDiagnosticV1 {
                    skin_node_name: skin.node_name.clone(),
                    vertex_indices: edge_indices.map(|index| skin.vertices[index].vertex_index),
                    vertex_influences: vertex_influences(&edge_indices),
                    bind_endpoints: [bind[left], bind[right]],
                    sampled_endpoints: [moved[left], moved[right]],
                    length_ratio: ratio,
                    sample_time_seconds: sample.time_seconds,
                };
                if quality
                    .worst_edge_collapse
                    .as_ref()
                    .is_none_or(|worst| ratio < worst.length_ratio)
                {
                    quality.worst_edge_collapse = Some(edge_diagnostic());
                }
                if quality
                    .worst_edge_expansion
                    .as_ref()
                    .is_none_or(|worst| ratio > worst.length_ratio)
                {
                    quality.worst_edge_expansion = Some(edge_diagnostic());
                }
                {
                    let component = &mut quality.components[component_index];
                    component.edge_sample_count += 1;
                    component.min_edge_ratio = component.min_edge_ratio.min(ratio);
                    component.max_edge_ratio = component.max_edge_ratio.max(ratio);
                }
                quality.max_edge_ratio = quality.max_edge_ratio.max(ratio);
                quality.min_edge_ratio = quality.min_edge_ratio.min(ratio);
                if bind_length >= soft_edge_min_bind_length {
                    quality.edge_soft_sample_count += 1;
                    if !(tolerances.edge_soft_min_ratio..=tolerances.edge_soft_max_ratio)
                        .contains(&ratio)
                    {
                        quality.edge_outside_soft_limit_count += 1;
                    }
                }
                if !(tolerances.edge_hard_min_ratio..=tolerances.edge_hard_max_ratio)
                    .contains(&ratio)
                {
                    quality.edge_outside_hard_limit_count += 1;
                    quality.components[component_index].edge_outside_hard_limit_count += 1;
                    if let Some(violations) = weight_violations.as_deref_mut() {
                        let penalty = motion_weight_violation_penalty_v1(
                            ratio,
                            tolerances.edge_hard_min_ratio,
                            tolerances.edge_hard_max_ratio,
                            1,
                        );
                        violations.add_edge(
                            &skin.node_name,
                            indices[left],
                            indices[right],
                            penalty,
                        );
                        violations.add_component_edge(
                            &sample.clip_name,
                            component_index,
                            &skin.node_name,
                            indices[left],
                            indices[right],
                            penalty,
                        );
                        if ratio < tolerances.edge_hard_min_ratio.powi(2)
                            || ratio > tolerances.edge_hard_max_ratio.powi(2)
                        {
                            violations.add_catastrophic_edge_witness(
                                &skin.node_name,
                                indices[left],
                                indices[right],
                                penalty,
                                ratio > tolerances.edge_hard_max_ratio.powi(2),
                                &sample.clip_name,
                                sample.time_seconds,
                                ratio,
                                [bind[left], bind[right]],
                                [moved[left], moved[right]],
                            );
                        }
                    }
                }
            }
        }
    }
    inspect_cross_component_seam_pairs_v3(
        sample,
        sampling_plan
            .seam_pairs
            .as_deref()
            .expect("seam plan initialized"),
        quality,
    )?;
    Ok(())
}

fn motion_weight_violation_penalty_v1(ratio: f32, minimum: f32, maximum: f32, base: u64) -> u64 {
    const MAXIMUM_PENALTY: u64 = 1_000_000;
    let severity = if !ratio.is_finite() || ratio <= 0.0 {
        f64::INFINITY
    } else if ratio < minimum {
        f64::from(minimum / ratio)
    } else if ratio > maximum {
        f64::from(ratio / maximum)
    } else {
        1.0
    };
    if !severity.is_finite() {
        return MAXIMUM_PENALTY;
    }
    ((base as f64 * severity * severity).ceil() as u64).clamp(base, MAXIMUM_PENALTY)
}

fn build_cross_component_seam_plan_v3(
    bind: &[[f32; 3]],
    components: &[usize],
    origins: &[(u32, usize)],
    one_ring_position_keys: &[BTreeSet<[u32; 3]>],
    tolerances: &ReferenceSupermodelMotionTolerancesV2,
) -> Vec<SeamPairSamplingPlanV3> {
    if bind.len() != components.len()
        || bind.len() != origins.len()
        || bind.len() != one_ring_position_keys.len()
        || bind.is_empty()
    {
        return Vec::new();
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in bind {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    let diagonal = distance(min, max);
    if !diagonal.is_finite() || diagonal <= 1.0e-8 {
        return Vec::new();
    }
    // A seam is a pair of topologically disconnected vertices that occupy
    // the same bind-space point and share at least one one-ring neighbour
    // position. The second condition proves a duplicated boundary edge;
    // position alone misclassifies coincident layered cards as one seam.
    let threshold = (diagonal * 1.0e-6).max(f32::EPSILON * 32.0);
    let output_floor = threshold * 2.0;
    let inverse_cell = threshold.recip();
    let mut grid: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();
    let mut pairs = Vec::new();
    for (index, point) in bind.iter().enumerate() {
        let cell = spatial_cell(*point, inverse_cell);
        grid.entry(cell).or_default().push(index);
    }
    for (index, point) in bind.iter().enumerate() {
        let base_cell = spatial_cell(*point, inverse_cell);
        let mut nearest: Option<(usize, f32)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell = (base_cell.0 + dx, base_cell.1 + dy, base_cell.2 + dz);
                    let Some(candidates) = grid.get(&cell) else {
                        continue;
                    };
                    for &candidate in candidates {
                        if candidate <= index
                            || components[candidate] == components[index]
                            || one_ring_position_keys[index]
                                .is_disjoint(&one_ring_position_keys[candidate])
                        {
                            continue;
                        }
                        let d0 = distance(*point, bind[candidate]);
                        if d0 <= threshold && nearest.is_none_or(|(_, best)| d0 < best) {
                            nearest = Some((candidate, d0));
                        }
                    }
                }
            }
        }
        if let Some((candidate, d0)) = nearest {
            let limit = (tolerances.seam_output_max_source_multiple * d0).max(output_floor);
            pairs.push(SeamPairSamplingPlanV3 {
                left_skin_part: origins[index].0,
                left_vertex: origins[index].1,
                right_skin_part: origins[candidate].0,
                right_vertex: origins[candidate].1,
                output_limit: limit,
            });
        }
    }
    pairs
}

fn inspect_cross_component_seam_pairs_v3(
    sample: &crate::mdl::SkinDeformationSampleV1,
    pairs: &[SeamPairSamplingPlanV3],
    quality: &mut InheritedMotionClipQualityV1,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    for pair in pairs {
        let left = sample
            .skins
            .iter()
            .find(|skin| skin.node_part == pair.left_skin_part)
            .and_then(|skin| skin.vertices.get(pair.left_vertex))
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-SEAM-PLAN-MISMATCH",
                    "motionQuality.seamPairs",
                    "cached left seam vertex is absent from the deformation sample",
                )
            })?;
        let right = sample
            .skins
            .iter()
            .find(|skin| skin.node_part == pair.right_skin_part)
            .and_then(|skin| skin.vertices.get(pair.right_vertex))
            .ok_or_else(|| {
                motion_error(
                    "M2A-SUPERMODEL-MOTION-SEAM-PLAN-MISMATCH",
                    "motionQuality.seamPairs",
                    "cached right seam vertex is absent from the deformation sample",
                )
            })?;
        quality.seam_pair_sample_count += 1;
        if distance(left.sampled_world, right.sampled_world) > pair.output_limit {
            quality.seam_pair_violation_count += 1;
        }
    }
    Ok(())
}

fn validate_motion_contract_v2(
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    if contract.schema_version != REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2 {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-CONTRACT-SCHEMA",
            "contract.schemaVersion",
            "only ReferenceSupermodelMotionContractV2 schema 2 is supported",
        ));
    }
    if !logical_id(&contract.contract_id)
        || !logical_id(&contract.clean_room_profile_id)
        || !is_resref(&contract.supermodel_resref)
        || contract.supermodel_resref.eq_ignore_ascii_case("NULL")
        || !is_lower_sha256(&contract.source_model_sha256)
        || !is_lower_sha256(&contract.clean_room_profile_sha256)
        || !contract.inspected_read_only
        || !contract.no_payload_copied
        || !contract.animation_scale.is_finite()
        || contract.animation_scale <= 0.0
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-CONTRACT-INVALID",
            "contract",
            "motion contract identity, provenance, scale or hashes are invalid",
        ));
    }
    let expected = canonical_reference_supermodel_motion_contract_sha256_v2(contract)?;
    if contract.content_sha256 != expected {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-CONTRACT-HASH-MISMATCH",
            "contract.contentSha256",
            "motion contract hash does not match canonical content",
        ));
    }
    if contract.nodes.is_empty() || contract.required_clips.is_empty() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-CONTRACT-INCOMPLETE",
            "contract",
            "motion contract requires nodes and at least one clip",
        ));
    }
    validate_motion_tolerances_v2(&contract.tolerances)?;
    let mut names = BTreeMap::new();
    let mut anchor_roles = BTreeSet::new();
    for (part, node) in contract.nodes.iter().enumerate() {
        let path = format!("contract.nodes[{part}]");
        if node.part_number != part as u32 {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-INVALID",
                format!("{path}.partNumber"),
                format!(
                    "carrier part number {} must equal its dense inventory index {part}",
                    node.part_number
                ),
            ));
        }
        if !is_node_name(&node.name) {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-NAME-INVALID",
                format!("{path}.name"),
                format!(
                    "carrier node name {:?} is not a valid Aurora node name",
                    node.name
                ),
            ));
        }
        if let Some(previous_part) = names.insert(node.name.to_ascii_lowercase(), part) {
            let previous = &contract.nodes[previous_part];
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-NAME-DUPLICATE",
                format!("{path}.name"),
                format!(
                    "carrier node name {:?} duplicates contract.nodes[{previous_part}] after ASCII case folding; current parent={:?}, class={:?}, role={}; previous parent={:?}, class={:?}, role={}",
                    node.name,
                    node.parent_part_number,
                    node.carrier_class,
                    node.structural_role,
                    previous.parent_part_number,
                    previous.carrier_class,
                    previous.structural_role,
                ),
            ));
        }
        if inverse_affine(node.carrier_bind_local_matrix).is_none() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-BIND-NONINVERTIBLE",
                format!("{path}.carrierBindLocalMatrix"),
                format!(
                    "carrier node {:?} has a non-finite, non-affine or singular neutral bind transform: {:?}",
                    node.name, node.carrier_bind_local_matrix
                ),
            ));
        }
        if node
            .anchor_role
            .as_ref()
            .is_some_and(|role| !logical_id(role))
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-ROLE-INVALID",
                format!("{path}.anchorRole"),
                format!("carrier node {:?} has an invalid semantic role", node.name),
            ));
        }
        if node
            .anchor_role
            .as_ref()
            .is_some_and(|role| !anchor_roles.insert(role.to_ascii_lowercase()))
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-ROLE-DUPLICATE",
                format!("{path}.anchorRole"),
                format!(
                    "carrier node {:?} duplicates an earlier semantic role",
                    node.name
                ),
            ));
        }
        if node.joint_axis.is_some_and(|axis| {
            let magnitude = length(axis);
            axis.iter().any(|value| !value.is_finite())
                || !magnitude.is_finite()
                || (magnitude - 1.0).abs() > 1.0e-4
        }) {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-NODE-AXIS-INVALID",
                format!("{path}.jointAxis"),
                format!(
                    "carrier node {:?} requires a finite unit joint axis",
                    node.name
                ),
            ));
        }
        if part == 0 {
            if node.parent_part_number.is_some() {
                return Err(motion_error(
                    "M2A-SUPERMODEL-MOTION-TOPOLOGY-INVALID",
                    "contract.nodes[0].parentPartNumber",
                    "part 0 must be the sole root",
                ));
            }
        } else if node
            .parent_part_number
            .is_none_or(|parent| parent >= node.part_number)
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-TOPOLOGY-INVALID",
                format!("contract.nodes[{part}].parentPartNumber"),
                "non-root carrier parent must reference an earlier part",
            ));
        }
    }
    let mut clips = BTreeSet::new();
    for (index, clip) in contract.required_clips.iter().enumerate() {
        if !is_node_name(clip) || !clips.insert(clip.to_ascii_lowercase()) {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CLIP-INVALID",
                format!("contract.requiredClips[{index}]"),
                "required clips must be valid and case-fold unique",
            ));
        }
    }
    for (part, node) in contract.nodes.iter().enumerate() {
        let mut controlling = BTreeSet::new();
        let controlling_valid = node.controlling_clips.iter().all(|clip| {
            clips.contains(&clip.to_ascii_lowercase())
                && controlling.insert(clip.to_ascii_lowercase())
        });
        let mut dynamic = BTreeSet::new();
        let dynamic_valid = node.dynamic_clips.iter().all(|clip| {
            controlling.contains(&clip.to_ascii_lowercase())
                && dynamic.insert(clip.to_ascii_lowercase())
        });
        let class_valid = match node.carrier_class {
            ReferenceSupermodelCarrierClassV3::SkinRelevant => {
                node.anchor_role.is_some() && !node.dynamic_clips.is_empty()
            }
            // A structural root/ancestor can own global locomotion without a
            // direct skin reference; descendants carry that transform.
            ReferenceSupermodelCarrierClassV3::PassiveStructural => true,
            ReferenceSupermodelCarrierClassV3::PassiveAttachmentOrEnd => {
                node.dynamic_clips.is_empty()
            }
        };
        if node.structural_role.trim().is_empty()
            || !controlling_valid
            || !dynamic_valid
            || !class_valid
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CARRIER-CLASSIFICATION-INVALID",
                format!("contract.nodes[{part}]"),
                "carrier class, structural role and controlling/dynamic clip inventory are inconsistent",
            ));
        }
    }
    let mut excluded = BTreeSet::new();
    for (index, node) in contract.carrier_exclusions.iter().enumerate() {
        if !is_node_name(&node.node_name)
            || node
                .parent_name
                .as_ref()
                .is_some_and(|name| !is_node_name(name))
            || node.reason.trim().is_empty()
            || !excluded.insert((
                node.node_name.to_ascii_lowercase(),
                node.parent_name
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase(),
            ))
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CARRIER-EXCLUSION-INVALID",
                format!("contract.carrierExclusions[{index}]"),
                "every excluded render-only node requires unique identity and an auditable reason",
            ));
        }
    }
    let mut events = BTreeSet::new();
    for (index, event) in contract.required_events.iter().enumerate() {
        if !clips.contains(&event.clip_name.to_ascii_lowercase())
            || !is_node_name(&event.event_name)
            || event.minimum_count == 0
            || !events.insert((
                event.clip_name.to_ascii_lowercase(),
                event.event_name.to_ascii_lowercase(),
            ))
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-EVENT-INVALID",
                format!("contract.requiredEvents[{index}]"),
                "required events must reference a required clip, have a valid unique name and a positive minimum count",
            ));
        }
    }
    Ok(())
}

fn validate_motion_tolerances_v2(
    value: &ReferenceSupermodelMotionTolerancesV2,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    let finite_positive = [
        value.bind_max_abs_error,
        value.edge_soft_min_ratio,
        value.edge_soft_max_ratio,
        value.edge_hard_min_ratio,
        value.edge_hard_max_ratio,
        value.triangle_min_area_ratio,
        value.triangle_max_area_ratio,
        value.seam_source_max_diagonal_fraction,
        value.seam_output_max_source_multiple,
        value.seam_output_floor_diagonal_fraction,
    ]
    .into_iter()
    .all(|item| item.is_finite() && item > 0.0);
    let fractions = [
        value.edge_soft_max_fraction,
        value.edge_hard_max_fraction,
        value.triangle_area_collapse_max_fraction,
        value.triangle_area_expansion_max_fraction,
        value.seam_max_violation_fraction,
        value.visible_anchor_min_amplitude_ratio,
        value.visible_anchor_min_trajectory_alignment,
        value.paw_ground_contact_max_height_fraction,
        value.clip_start_anchor_max_jump_fraction,
    ]
    .into_iter()
    .all(|item| item.is_finite() && (0.0..=1.0).contains(&item));
    if !finite_positive
        || !fractions
        || value.edge_hard_min_ratio > value.edge_soft_min_ratio
        || value.edge_soft_min_ratio > 1.0
        || value.edge_soft_max_ratio < 1.0
        || value.edge_soft_max_ratio > value.edge_hard_max_ratio
        || value.triangle_min_area_ratio > 1.0
        || value.triangle_max_area_ratio < 1.0
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-TOLERANCES-INVALID",
            "contract.tolerances",
            "motion tolerances must be finite, ordered around 1.0 and use valid fractions",
        ));
    }
    Ok(())
}

pub fn validate_reference_supermodel_motion_oracle_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    validate_motion_contract_v2(contract)?;
    validate_reference_bind_contract_v3(contract, reference)?;
    if reference.model.classification != contract.classification
        || (reference.model.animation_scale - contract.animation_scale).abs() > 1.0e-6
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-ORACLE-MODEL-MISMATCH",
            "referenceSupermodel.model",
            "classification or animationScale differs from the motion contract",
        ));
    }
    let flat = reference_nodes_for_contract_v3(contract, reference)?;
    if flat.len() != contract.nodes.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-ORACLE-TOPOLOGY-MISMATCH",
            "referenceSupermodel.nodeTree",
            "reference node count differs from the motion contract",
        ));
    }
    let parent_by_offset = flat
        .iter()
        .enumerate()
        .map(|(part, node)| (node.offset, part as u32))
        .collect::<BTreeMap<_, _>>();
    for (part, (node, expected)) in flat.iter().zip(&contract.nodes).enumerate() {
        let parent = node
            .parent_offset
            .and_then(|offset| parent_by_offset.get(&offset).copied());
        if expected.part_number != part as u32
            || (part != 0 && !node.name.eq_ignore_ascii_case(&expected.name))
            || parent != expected.parent_part_number
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-ORACLE-TOPOLOGY-MISMATCH",
                format!("referenceSupermodel.nodeTree[{part}]"),
                "reference node number/name/parent differs from the motion contract",
            ));
        }
    }
    for required in &contract.required_clips {
        let clip = reference
            .animations
            .iter()
            .any(|clip| clip.name.eq_ignore_ascii_case(required))
            .then(|| {
                reference
                    .animations
                    .iter()
                    .find(|clip| clip.name.eq_ignore_ascii_case(required))
                    .expect("clip presence was just checked")
            });
        if clip.is_none() {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CLIP-MISSING",
                "referenceSupermodel.animations",
                format!("required inherited clip {required} is absent"),
            ));
        }
    }
    for (part, expected) in contract.nodes.iter().enumerate() {
        let mut position = false;
        let mut orientation = false;
        let mut scale = false;
        for clip in reference.animations.iter().filter(|clip| {
            contract
                .required_clips
                .iter()
                .any(|required| clip.name.eq_ignore_ascii_case(required))
        }) {
            if let Some(node) = flatten_nodes(&clip.node_tree.roots)
                .into_iter()
                .find(|node| node.name == expected.name)
            {
                position |= has_decoded_controller(node, "position");
                orientation |= has_decoded_controller(node, "orientation");
                scale |= has_decoded_controller(node, "scale");
            }
        }
        if (expected.position_controller_required && !position)
            || (expected.orientation_controller_required && !orientation)
            || (expected.scale_controller_required && !scale)
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-CONTROLLER-MISSING",
                format!("referenceSupermodel.nodeTree[{part}]"),
                "required decoded position/orientation/scale controller is absent from the acceptance clips",
            ));
        }
    }
    for event in &contract.required_events {
        let clip = reference
            .animations
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(&event.clip_name))
            .expect("required event clip inventory was validated");
        let count = clip
            .events
            .iter()
            .filter(|actual| actual.name.eq_ignore_ascii_case(&event.event_name))
            .count();
        if count < event.minimum_count as usize {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-EVENT-MISSING",
                "referenceSupermodel.animations.events",
                format!(
                    "clip {} requires at least {} event(s) named {}",
                    event.clip_name, event.minimum_count, event.event_name
                ),
            ));
        }
    }
    Ok(())
}

fn has_decoded_controller(node: &crate::mdl::NodeReport, name: &str) -> bool {
    node.controllers.iter().any(|controller| {
        controller.decoded
            && controller
                .controller_name
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
    })
}

fn node_has_dynamic_transform_controller_v3(node: &crate::mdl::NodeReport) -> bool {
    node.controllers.iter().any(|controller| {
        controller.decoded
            && controller.controller_name.as_deref().is_some_and(|name| {
                ["position", "orientation", "scale"]
                    .iter()
                    .any(|expected| name.eq_ignore_ascii_case(expected))
            })
            && controller.row_count > 1
            && controller.values.windows(2).any(|rows| rows[0] != rows[1])
    })
}

fn exact_node_bind_local_matrix_v3(
    node: &crate::mdl::NodeReport,
    part: usize,
) -> Result<[f32; 16], ReferenceSupermodelMotionErrorV2> {
    let position = exact_bind_controller_row_v3(node, "position", 3, part)?
        .map(|row| [row[0], row[1], row[2]])
        .unwrap_or([0.0, 0.0, 0.0]);
    let mut orientation = exact_bind_controller_row_v3(node, "orientation", 4, part)?
        .map(|row| [row[0], row[1], row[2], row[3]])
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let norm = orientation
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= 1.0e-8 {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            format!("reference.nodeTree.nodes[{part}].controllers.orientation"),
            "neutral orientation must be a finite nonzero XYZW quaternion",
        ));
    }
    orientation.iter_mut().for_each(|value| *value /= norm);
    let scale = exact_bind_controller_row_v3(node, "scale", 1, part)?
        .map(|row| row[0])
        .unwrap_or(1.0);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            format!("reference.nodeTree.nodes[{part}].controllers.scale"),
            "neutral scale must be finite and positive",
        ));
    }
    Ok(matrix_from_quaternion_translation_scale_v3(
        orientation,
        position,
        scale,
    ))
}

fn exact_bind_controller_row_v3<'a>(
    node: &'a crate::mdl::NodeReport,
    name: &str,
    columns: usize,
    part: usize,
) -> Result<Option<&'a [f32]>, ReferenceSupermodelMotionErrorV2> {
    let matching = node
        .controllers
        .iter()
        .filter(|controller| {
            controller
                .controller_name
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            format!("reference.nodeTree.nodes[{part}].controllers.{name}"),
            "neutral bind controller must be unique",
        ));
    }
    let Some(controller) = matching.first() else {
        return Ok(None);
    };
    let row = controller.values.first().ok_or_else(|| {
        motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            format!("reference.nodeTree.nodes[{part}].controllers.{name}"),
            "neutral bind controller contains no decoded row",
        )
    })?;
    if !controller.decoded
        || controller.column_count != columns
        || row.len() != columns
        || row.iter().any(|value| !value.is_finite())
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-INVALID",
            format!("reference.nodeTree.nodes[{part}].controllers.{name}"),
            format!("neutral {name} controller requires one finite {columns}-column row"),
        ));
    }
    Ok(Some(row))
}

fn matrix_from_quaternion_translation_scale_v3(q: [f32; 4], p: [f32; 3], scale: f32) -> [f32; 16] {
    let [x, y, z, w] = q;
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;
    [
        (1.0 - 2.0 * (yy + zz)) * scale,
        (2.0 * (xy + wz)) * scale,
        (2.0 * (xz - wy)) * scale,
        0.0,
        (2.0 * (xy - wz)) * scale,
        (1.0 - 2.0 * (xx + zz)) * scale,
        (2.0 * (yz + wx)) * scale,
        0.0,
        (2.0 * (xz + wy)) * scale,
        (2.0 * (yz - wx)) * scale,
        (1.0 - 2.0 * (xx + yy)) * scale,
        0.0,
        p[0],
        p[1],
        p[2],
        1.0,
    ]
}

fn validate_reference_bind_contract_v3(
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    let flattened = reference_nodes_for_contract_v3(contract, reference)?;
    if flattened.len() != contract.nodes.len()
        || reference.model.classification != contract.classification
        || (reference.model.animation_scale - contract.animation_scale).abs()
            > contract.tolerances.bind_max_abs_error
    {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH",
            "reference",
            "reference classification, animation scale or carrier count differs from contract",
        ));
    }
    let part_by_offset = flattened
        .iter()
        .enumerate()
        .map(|(part, node)| (node.offset, part as u32))
        .collect::<BTreeMap<_, _>>();
    let mut maximum = 0.0_f32;
    for (part, (node, expected)) in flattened.iter().zip(&contract.nodes).enumerate() {
        let parent = node
            .parent_offset
            .and_then(|offset| part_by_offset.get(&offset).copied());
        let name_matches = part == 0 || node.name.eq_ignore_ascii_case(&expected.name);
        let actual = exact_node_bind_local_matrix_v3(node, part)?;
        let error = max_abs_matrix_difference(actual, expected.carrier_bind_local_matrix);
        maximum = maximum.max(error);
        if !name_matches
            || parent != expected.parent_part_number
            || error > contract.tolerances.bind_max_abs_error
        {
            return Err(motion_error(
                "M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH",
                format!("contract.nodes[{part}]"),
                format!(
                    "name/parent/bind differs from inspected reference; maximum matrix error {maximum}"
                ),
            ));
        }
    }
    Ok(())
}

fn exact_rig_reference_bind_error_v3(
    rig: &CreatureRigProfileV1,
    reference: &InspectionReport,
) -> Result<f32, ReferenceSupermodelMotionErrorV2> {
    let pseudo_contract = ReferenceSupermodelMotionContractV2 {
        schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
        contract_id: "exact-rig-reference-lookup".to_owned(),
        content_sha256: String::new(),
        supermodel_resref: reference.model.name.to_ascii_lowercase(),
        source_model_sha256: "0".repeat(64),
        inspected_read_only: true,
        no_payload_copied: true,
        classification: reference.model.classification,
        animation_scale: reference.model.animation_scale,
        clean_room_profile_id: "exact-rig-reference-lookup".to_owned(),
        clean_room_profile_sha256: "0".repeat(64),
        nodes: rig
            .nodes
            .iter()
            .enumerate()
            .map(|(part, node)| ReferenceSupermodelMotionNodeV2 {
                part_number: part as u32,
                name: node.name.clone(),
                parent_part_number: node.parent_id.and_then(|parent_id| {
                    rig.nodes
                        .iter()
                        .position(|candidate| candidate.id == parent_id)
                        .map(|index| index as u32)
                }),
                carrier_bind_local_matrix: node.bind_local_matrix,
                position_controller_required: false,
                orientation_controller_required: false,
                scale_controller_required: false,
                anchor_role: None,
                joint_axis: None,
                carrier_class: ReferenceSupermodelCarrierClassV3::PassiveStructural,
                structural_role: "REFERENCE_BIND_LOOKUP".to_owned(),
                controlling_clips: Vec::new(),
                dynamic_clips: Vec::new(),
            })
            .collect(),
        carrier_exclusions: Vec::new(),
        required_clips: Vec::new(),
        required_events: Vec::new(),
        tolerances: default_reference_supermodel_motion_tolerances_v2(),
    };
    let flattened = reference_nodes_for_contract_v3(&pseudo_contract, reference)?;
    if flattened.len() != rig.nodes.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH",
            "exactCarrierRig.nodes",
            "exact carrier count differs from inspected reference",
        ));
    }
    let mut maximum = 0.0_f32;
    for (part, (node, carrier)) in flattened.iter().zip(&rig.nodes).enumerate() {
        maximum = maximum.max(max_abs_matrix_difference(
            exact_node_bind_local_matrix_v3(node, part)?,
            carrier.bind_local_matrix,
        ));
    }
    Ok(maximum)
}

fn reference_nodes_for_contract_v3<'a>(
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &'a InspectionReport,
) -> Result<Vec<&'a crate::mdl::NodeReport>, ReferenceSupermodelMotionErrorV2> {
    let all = flatten_nodes_in_tree_order(&reference.node_tree.roots);
    let mut selected: Vec<&'a crate::mdl::NodeReport> = Vec::with_capacity(contract.nodes.len());
    for (part, expected) in contract.nodes.iter().enumerate() {
        let matches = if part == 0 {
            reference.node_tree.roots.iter().collect::<Vec<_>>()
        } else {
            let expected_parent_offset = expected
                .parent_part_number
                .and_then(|parent| selected.get(parent as usize))
                .map(|parent| parent.offset);
            all.iter()
                .copied()
                .filter(|node| {
                    node.name.eq_ignore_ascii_case(&expected.name)
                        && node.parent_offset == expected_parent_offset
                        && exact_node_bind_local_matrix_v3(node, part).is_ok_and(|actual| {
                            max_abs_matrix_difference(actual, expected.carrier_bind_local_matrix)
                                <= contract.tolerances.bind_max_abs_error
                        })
                })
                .collect::<Vec<_>>()
        };
        let [node] = matches.as_slice() else {
            return Err(motion_error(
                "M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH",
                format!("contract.nodes[{part}].name"),
                format!(
                    "expected exactly one inspected carrier named {:?}, found {}",
                    expected.name,
                    matches.len()
                ),
            ));
        };
        selected.push(*node);
    }
    Ok(selected)
}

fn validate_target_topology_v2(
    target: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<(), ReferenceSupermodelMotionErrorV2> {
    if target.nodes.len() != contract.nodes.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-TOPOLOGY-MISMATCH",
            "targetRig.nodes",
            "target rig must provide exactly one target node per carrier node",
        ));
    }
    let part_by_id = target
        .nodes
        .iter()
        .enumerate()
        .map(|(part, node)| (node.id, part as u32))
        .collect::<BTreeMap<_, _>>();
    if part_by_id.len() != target.nodes.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-TOPOLOGY-MISMATCH",
            "targetRig.nodes",
            "target node ids must be unique",
        ));
    }
    for (part, (node, expected)) in target.nodes.iter().zip(&contract.nodes).enumerate() {
        let parent = node.parent_id.and_then(|id| part_by_id.get(&id).copied());
        let name_matches = part == 0 || node.name == expected.name;
        if parent != expected.parent_part_number || !name_matches {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-TOPOLOGY-MISMATCH",
                format!("targetRig.nodes[{part}]"),
                "target node name/parent topology differs from the carrier contract",
            ));
        }
    }
    Ok(())
}

fn rig_world_matrices(
    rig: &CreatureRigProfileV1,
) -> Result<Vec<[f32; 16]>, ReferenceSupermodelMotionErrorV2> {
    let part_by_id = rig
        .nodes
        .iter()
        .enumerate()
        .map(|(part, node)| (node.id, part))
        .collect::<BTreeMap<_, _>>();
    let locals = rig
        .nodes
        .iter()
        .map(|node| node.bind_local_matrix)
        .collect::<Vec<_>>();
    let parents = rig
        .nodes
        .iter()
        .enumerate()
        .map(|(part, node)| {
            node.parent_id
                .map(|id| {
                    part_by_id.get(&id).copied().ok_or_else(|| {
                        motion_error(
                            "M2A-SUPERMODEL-MOTION-PARENT-MISSING",
                            format!("targetRig.nodes[{part}].parentId"),
                            "target parent id is absent",
                        )
                    })
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    ordered_world_matrices(&locals, &parents, "targetRig.nodes")
}

fn ordered_world_matrices(
    locals: &[[f32; 16]],
    parents: &[Option<usize>],
    path: &str,
) -> Result<Vec<[f32; 16]>, ReferenceSupermodelMotionErrorV2> {
    if locals.len() != parents.len() {
        return Err(motion_error(
            "M2A-SUPERMODEL-MOTION-INTERNAL",
            path,
            "local and parent arrays differ in length",
        ));
    }
    let mut worlds = Vec::with_capacity(locals.len());
    for (part, (local, parent)) in locals.iter().zip(parents).enumerate() {
        if inverse_affine(*local).is_none() || parent.is_some_and(|value| value >= part) {
            return Err(motion_error(
                "M2A-SUPERMODEL-MOTION-BIND-INVALID",
                format!("{path}[{part}]"),
                "bind matrix must be finite affine and parent must precede child",
            ));
        }
        worlds.push(parent.map_or(*local, |parent| mul_mat4(worlds[parent], *local)));
    }
    Ok(worlds)
}

fn flatten_nodes(nodes: &[crate::mdl::NodeReport]) -> Vec<&crate::mdl::NodeReport> {
    let mut output = flatten_nodes_in_tree_order(nodes);
    output.sort_by_key(|node| node.number);
    output
}

fn flatten_nodes_in_tree_order(nodes: &[crate::mdl::NodeReport]) -> Vec<&crate::mdl::NodeReport> {
    fn visit<'a>(node: &'a crate::mdl::NodeReport, output: &mut Vec<&'a crate::mdl::NodeReport>) {
        output.push(node);
        for child in &node.children {
            visit(child, output);
        }
    }
    let mut output = Vec::new();
    for root in nodes {
        visit(root, &mut output);
    }
    output
}

fn find_node_by_part(
    nodes: &[crate::mdl::NodeReport],
    part: u32,
) -> Option<&crate::mdl::NodeReport> {
    for node in nodes {
        if node.number == part {
            return Some(node);
        }
        if let Some(found) = find_node_by_part(&node.children, part) {
            return Some(found);
        }
    }
    None
}

#[derive(Clone, Debug)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(len: usize) -> Self {
        Self {
            parent: (0..len).collect(),
            rank: vec![0; len],
        }
    }

    fn find(&mut self, value: usize) -> usize {
        if self.parent[value] != value {
            self.parent[value] = self.find(self.parent[value]);
        }
        self.parent[value]
    }

    fn union(&mut self, left: usize, right: usize) {
        let left = self.find(left);
        let right = self.find(right);
        if left == right {
            return;
        }
        match self.rank[left].cmp(&self.rank[right]) {
            std::cmp::Ordering::Less => self.parent[left] = right,
            std::cmp::Ordering::Greater => self.parent[right] = left,
            std::cmp::Ordering::Equal => {
                self.parent[right] = left;
                self.rank[left] = self.rank[left].saturating_add(1);
            }
        }
    }
}

fn spatial_cell(point: [f32; 3], inverse_cell: f32) -> (i32, i32, i32) {
    (
        (point[0] * inverse_cell).floor() as i32,
        (point[1] * inverse_cell).floor() as i32,
        (point[2] * inverse_cell).floor() as i32,
    )
}

fn triangle_cross(points: [[f32; 3]; 3]) -> [f32; 3] {
    let a = [
        points[1][0] - points[0][0],
        points[1][1] - points[0][1],
        points[1][2] - points[0][2],
    ];
    let b = [
        points[2][0] - points[0][0],
        points[2][1] - points[0][1],
        points[2][2] - points[0][2],
    ];
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    length([left[0] - right[0], left[1] - right[1], left[2] - right[2]])
}

fn length(value: [f32; 3]) -> f32 {
    dot(value, value).sqrt()
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn sub(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|axis| left[axis] - right[axis])
}

fn mul_mat4(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for row in 0..4 {
        for column in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|inner| left[inner * 4 + row] * right[column * 4 + inner])
                .sum();
        }
    }
    output
}

fn transform_point_v3(matrix: [f32; 16], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn inverse_affine(matrix: [f32; 16]) -> Option<[f32; 16]> {
    if matrix.iter().any(|value| !value.is_finite())
        || matrix[3].abs() > 1.0e-6
        || matrix[7].abs() > 1.0e-6
        || matrix[11].abs() > 1.0e-6
        || (matrix[15] - 1.0).abs() > 1.0e-6
    {
        return None;
    }
    let linear = [
        [matrix[0], matrix[4], matrix[8]],
        [matrix[1], matrix[5], matrix[9]],
        [matrix[2], matrix[6], matrix[10]],
    ];
    let inverse = inverse3(linear)?;
    let translation = [matrix[12], matrix[13], matrix[14]];
    let inverse_translation = mul3(inverse, translation.map(|value| -value));
    Some([
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
    ])
}

fn inverse3(matrix: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    let determinant = matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]);
    if !determinant.is_finite() || determinant.abs() <= 1.0e-12 {
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

fn mul3(matrix: [[f32; 3]; 3], value: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * value[0] + matrix[0][1] * value[1] + matrix[0][2] * value[2],
        matrix[1][0] * value[0] + matrix[1][1] * value[1] + matrix[1][2] * value[2],
        matrix[2][0] * value[0] + matrix[2][1] * value[1] + matrix[2][2] * value[2],
    ]
}

fn max_abs_matrix_difference(left: [f32; 16], right: [f32; 16]) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f32::max)
}

fn motion_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReferenceSupermodelMotionErrorV2 {
    ReferenceSupermodelMotionErrorV2 {
        schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

fn logical_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

fn is_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_node_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 31
        && value.is_ascii()
        && !value.bytes().any(|byte| byte == 0)
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn default_reference_supermodel_writer_options_v1(model_resref: &str) -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile:
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
        state_projection_profile:
            crate::mdl::MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: model_resref.to_owned(),
        diffuse_texture_resref_by_material_slot: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mdl::{
        SkinDeformationNodeSampleV1, SkinDeformationSampleV1, SkinDeformationVertexSampleV1,
    };

    #[test]
    fn global_motion_dummy_does_not_require_its_own_surface_region() {
        // Model root -> animated global dummy -> two rendered body branches.
        let parents = [None, Some(0), Some(1), Some(1), Some(2)];
        assert!(surface_free_global_motion_ancestor_v1(
            1,
            false,
            false,
            &parents,
            &[2, 3, 4]
        ));
        assert!(!surface_free_global_motion_ancestor_v1(
            2,
            false,
            false,
            &parents,
            &[3, 4]
        ));
        assert!(!surface_free_global_motion_ancestor_v1(
            1,
            false,
            true,
            &parents,
            &[2, 3, 4]
        ));
        assert!(!surface_free_global_motion_ancestor_v1(
            1,
            true,
            false,
            &parents,
            &[2, 3, 4]
        ));
    }

    fn refinement_contract() -> ReferenceSupermodelMotionContractV2 {
        let identity = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let node = |part_number, name: &str, parent_part_number| ReferenceSupermodelMotionNodeV2 {
            part_number,
            name: name.to_owned(),
            parent_part_number,
            carrier_bind_local_matrix: identity,
            position_controller_required: false,
            orientation_controller_required: false,
            scale_controller_required: false,
            anchor_role: None,
            joint_axis: None,
            carrier_class: ReferenceSupermodelCarrierClassV3::SkinRelevant,
            structural_role: "chain".to_owned(),
            controlling_clips: Vec::new(),
            dynamic_clips: Vec::new(),
        };
        ReferenceSupermodelMotionContractV2 {
            schema_version: REFERENCE_SUPERMODEL_MOTION_SCHEMA_VERSION_V2,
            contract_id: "refinement-test".to_owned(),
            content_sha256: "0".repeat(64),
            supermodel_resref: "c_test".to_owned(),
            source_model_sha256: "1".repeat(64),
            inspected_read_only: true,
            no_payload_copied: true,
            classification: 0,
            animation_scale: 1.0,
            clean_room_profile_id: "refinement-test".to_owned(),
            clean_room_profile_sha256: "2".repeat(64),
            nodes: vec![
                node(0, "root", None),
                node(1, "child", Some(0)),
                node(2, "grandchild", Some(1)),
            ],
            carrier_exclusions: Vec::new(),
            required_clips: Vec::new(),
            required_events: Vec::new(),
            tolerances: default_reference_supermodel_motion_tolerances_v2(),
        }
    }

    #[test]
    fn direct_exact_bind_plateau_does_not_skip_measured_catastrophic_edge_phase() {
        assert!(direct_exact_bind_has_pending_targeted_phase_v1(
            false, 0, 24
        ));
        assert!(direct_exact_bind_has_pending_targeted_phase_v1(
            false, 23, 24
        ));
        assert!(!direct_exact_bind_has_pending_targeted_phase_v1(
            false, 24, 24
        ));
        assert!(!direct_exact_bind_has_pending_targeted_phase_v1(
            true, 0, 24
        ));
    }

    #[test]
    fn targeted_density_repair_is_component_local_only_after_global_geometry_passes() {
        assert!(targeted_density_repair_should_restrict_v1(
            true,
            [0.98, 1.0, 0.14, 0.87],
            true,
            false,
        ));
        assert!(!targeted_density_repair_should_restrict_v1(
            false,
            [0.98, 1.0, 0.14, 0.87],
            true,
            false,
        ));
        assert!(!targeted_density_repair_should_restrict_v1(
            true,
            [1.01, 1.0, 0.14, 0.87],
            true,
            false,
        ));
        assert!(!targeted_density_repair_should_restrict_v1(
            true,
            [0.98, 1.0, 0.14, 0.87],
            true,
            true,
        ));
    }

    #[test]
    fn seam_plan_requires_a_shared_one_ring_position() {
        let bind = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let components = [0, 1, 2];
        let origins = [(30, 0), (31, 0), (32, 0)];
        let shared = [0.5_f32.to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits()];
        let unrelated = [(-0.5_f32).to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits()];
        let tolerances = default_reference_supermodel_motion_tolerances_v2();

        let false_layer = build_cross_component_seam_plan_v3(
            &bind,
            &components,
            &origins,
            &[
                BTreeSet::from([shared]),
                BTreeSet::from([unrelated]),
                BTreeSet::new(),
            ],
            &tolerances,
        );
        assert!(false_layer.is_empty());

        let duplicated_edge = build_cross_component_seam_plan_v3(
            &bind,
            &components,
            &origins,
            &[
                BTreeSet::from([shared]),
                BTreeSet::from([shared]),
                BTreeSet::new(),
            ],
            &tolerances,
        );
        assert_eq!(duplicated_edge.len(), 1);
    }

    #[test]
    fn logical_component_remap_welds_writer_seams_but_not_coincident_layers() {
        let shared = [1.0_f32.to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits()];
        let unrelated = [(-1.0_f32).to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits()];
        let bind = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ];
        let raw_components = [0, 0, 1, 1, 2, 2];
        let one_rings = [
            BTreeSet::from([shared]),
            BTreeSet::new(),
            BTreeSet::from([shared]),
            BTreeSet::new(),
            BTreeSet::from([unrelated]),
            BTreeSet::new(),
        ];

        let (remap, logical_count) =
            build_logical_component_remap_v1(&bind, &raw_components, &one_rings, 3).unwrap();

        assert_eq!(logical_count, 2);
        assert_eq!(remap[0], remap[1]);
        assert_ne!(remap[0], remap[2]);
    }

    fn rigid_weight(bone: u32) -> AuroraVertexWeightsV1 {
        AuroraVertexWeightsV1 {
            bone_node_ids: [Some(bone), None, None, None],
            values: [1.0, 0.0, 0.0, 0.0],
            influence_count: 1,
        }
    }

    fn anchor_context() -> AnchorMotionContextV2 {
        let assignments = [
            ("left_front_paw", 0_usize),
            ("right_front_paw", 1),
            ("left_rear_paw", 2),
            ("right_rear_paw", 3),
        ]
        .into_iter()
        .map(|(role, vertex_index)| {
            (
                role.to_owned(),
                vec![AnchorVertexReferenceV2 {
                    skin_node_part: 10,
                    vertex_index,
                    influence: 1.0,
                }],
            )
        })
        .collect();
        AnchorMotionContextV2 {
            assignments,
            controller_name_by_role: BTreeMap::new(),
            carrier_part_by_role: BTreeMap::new(),
            structural_role_by_role: BTreeMap::new(),
            dynamic_clips_by_role: BTreeMap::new(),
            required_roles: vec![
                "left_front_paw".to_owned(),
                "left_rear_paw".to_owned(),
                "right_front_paw".to_owned(),
                "right_rear_paw".to_owned(),
            ],
            paw_roles: vec![
                "left_front_paw".to_owned(),
                "left_rear_paw".to_owned(),
                "right_front_paw".to_owned(),
                "right_rear_paw".to_owned(),
            ],
            appendage_role_chains: Vec::new(),
        }
    }

    fn sample(left_front: [f32; 3]) -> SkinDeformationSampleV1 {
        let bind = [
            [-1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let mut sampled = bind;
        sampled[0] = left_front;
        SkinDeformationSampleV1 {
            schema_version: 1,
            profile: "SYNTHETIC_ANCHOR_SAMPLE".to_owned(),
            clip_name: "cpause1".to_owned(),
            time_seconds: 0.0,
            skin_count: 1,
            vertex_count: 5,
            moved_vertex_count: u32::from(left_front != bind[0]),
            max_displacement: distance(left_front, bind[0]),
            skins: vec![SkinDeformationNodeSampleV1 {
                node_part: 10,
                node_name: "skin".to_owned(),
                vertices: bind
                    .into_iter()
                    .zip(sampled)
                    .enumerate()
                    .map(|(vertex_index, (bind_world, sampled_world))| {
                        SkinDeformationVertexSampleV1 {
                            vertex_index: vertex_index as u32,
                            bind_world,
                            sampled_world,
                            displacement: distance(bind_world, sampled_world),
                        }
                    })
                    .collect(),
            }],
        }
    }

    fn quality() -> InheritedMotionClipQualityV1 {
        InheritedMotionClipQualityV1 {
            clip_name: "cpause1".to_owned(),
            sampled_times: vec![0.0],
            vertex_sample_count: 5,
            edge_sample_count: 0,
            edge_soft_sample_count: 0,
            edge_outside_soft_limit_count: 0,
            edge_outside_hard_limit_count: 0,
            triangle_sample_count: 0,
            triangle_area_collapse_count: 0,
            triangle_area_expansion_count: 0,
            sampled_component_count: 1,
            component_sample_count: 1,
            per_component_geometry_coverage: true,
            per_component_deformation_coverage: true,
            components: vec![InheritedMotionComponentQualityV1 {
                component_index: 0,
                triangle_sample_count: 1,
                edge_sample_count: 3,
                edge_outside_hard_limit_count: 0,
                triangle_area_collapse_count: 0,
                triangle_area_expansion_count: 0,
                min_edge_ratio: 1.0,
                max_edge_ratio: 1.0,
                min_triangle_area_ratio: 1.0,
                max_triangle_area_ratio: 1.0,
                pass: true,
            }],
            worst_triangle_area_collapse: None,
            worst_triangle_area_expansion: None,
            worst_edge_collapse: None,
            worst_edge_expansion: None,
            world_normal_opposition_count: 0,
            seam_pair_sample_count: 0,
            seam_pair_violation_count: 0,
            visible_anchor_trajectories: Vec::new(),
            visible_anchor_motion_violation_count: 0,
            max_edge_ratio: 1.0,
            min_edge_ratio: 1.0,
            max_displacement: 0.0,
            bounds_min: [-1.0, -1.0, 0.0],
            bounds_max: [1.0, 1.0, 1.0],
            paw_gate_applied: true,
            anchor_cluster_count: 4,
            anchor_cluster_missing_count: 0,
            paw_cluster_count: 4,
            paw_cluster_missing_count: 0,
            paw_contact_violation_count: 0,
            paw_side_violation_count: 0,
            clip_start_anchor_jump_violation_count: 0,
            max_paw_ground_height_error: 0.0,
            max_clip_start_anchor_jump: 0.0,
            root_motion_distance: 0.0,
            appendage_pair_required_count: 0,
            appendage_pair_pass_count: 0,
            appendage_relative_motion_violation_count: 0,
            appendage_relative_motion: Vec::new(),
        }
    }

    #[test]
    fn motion_weight_refinement_rigidifies_only_measured_small_component_and_welds_duplicates() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "refinement-test".to_owned(),
            source_sha256: "3".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![
                crate::model_ir::AuroraModelSegmentV1 {
                    segment_id: 7,
                    material_slot: 0,
                    deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                    parent_node_id: 0,
                    cast_shadow: true,
                    positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    normals: vec![[0.0, 0.0, 1.0]; 3],
                    tangents: None,
                    uv0: vec![[0.0, 0.0]; 3],
                    indices: vec![0, 1, 2],
                    face_surface_ids: Vec::new(),
                    weights: vec![rigid_weight(0), rigid_weight(1), rigid_weight(1)],
                },
                crate::model_ir::AuroraModelSegmentV1 {
                    segment_id: 8,
                    material_slot: 0,
                    deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                    parent_node_id: 0,
                    cast_shadow: true,
                    positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
                    normals: vec![[0.0, 0.0, 1.0]; 3],
                    tangents: None,
                    uv0: vec![[0.0, 0.0]; 3],
                    indices: vec![0, 1, 2],
                    face_surface_ids: Vec::new(),
                    weights: vec![rigid_weight(0), rigid_weight(1), rigid_weight(1)],
                },
            ],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_7", 0, 1, 10);
        violations.add_edge("m2a_seg_7", 0, 2, 10);
        let report = refine_motion_violating_skin_weights_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            None,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert!(report.applied);
        assert_eq!(report.edge_count, 2);
        assert_eq!(report.rigid_component_count, 1);
        assert_eq!(
            model.segments[0].weights[0], model.segments[1].weights[0],
            "exact duplicate vertices must remain an atomic seam group"
        );
        assert_eq!(model.segments[0].weights[0].influence_count, 1);
        assert_eq!(model.segments[0].weights[0].bone_node_ids[0], Some(1));
        assert!(support_is_one_carrier_edge_v1(
            &[0, 1],
            &refinement_contract()
        ));
    }

    #[test]
    fn motion_duplicate_groups_do_not_weld_coincident_layers_without_shared_one_ring() {
        let segment = |segment_id, positions| crate::model_ir::AuroraModelSegmentV1 {
            segment_id,
            material_slot: 0,
            deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            positions,
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            weights: vec![rigid_weight(0); 3],
        };
        let model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "coincident-layer-test".to_owned(),
            source_sha256: "6".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![
                segment(1, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
                segment(2, vec![[0.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]]),
            ],
        };

        let groups = build_motion_weight_duplicate_groups_v2(&model).unwrap();
        let first = groups
            .iter()
            .position(|group| group.contains(&(0, 0)))
            .unwrap();
        let second = groups
            .iter()
            .position(|group| group.contains(&(1, 0)))
            .unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn motion_weight_refinement_preserves_every_required_joint_anchor() {
        let mut contract = refinement_contract();
        contract.nodes[1].anchor_role = Some("joint_1".to_owned());
        contract.nodes[2].anchor_role = Some("limb_paw_2".to_owned());
        contract.nodes[2].structural_role = "LIMB_GROUND_CONTACT_TERMINAL".to_owned();
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "required-anchor-refinement-test".to_owned(),
            source_sha256: "9".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 13,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 3],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: vec![
                    rigid_weight(0),
                    rigid_weight(0),
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), Some(2), None],
                        values: [0.5, 0.3, 0.2, 0.0],
                        influence_count: 3,
                    },
                ],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_13", 0, 2, 100);

        refine_motion_violating_skin_weights_v1(
            &mut model,
            &contract,
            &violations,
            None,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert!(
            model
                .segments
                .iter()
                .flat_map(|segment| &segment.weights)
                .any(|row| aurora_weight_for_bone_v1(row, 2) >= 0.1)
        );
        assert!(
            model
                .segments
                .iter()
                .flat_map(|segment| &segment.weights)
                .any(|row| aurora_weight_for_bone_v1(row, 1) >= 0.1)
        );
    }

    #[test]
    fn small_component_terminal_weight_can_rigidify_when_anchor_survives_elsewhere() {
        let mut contract = refinement_contract();
        contract.nodes[2].anchor_role = Some("limb_paw_2".to_owned());
        contract.nodes[2].structural_role = "LIMB_GROUND_CONTACT_TERMINAL".to_owned();
        let terminal_blend = AuroraVertexWeightsV1 {
            bone_node_ids: [Some(0), Some(2), None, None],
            values: [0.8, 0.2, 0.0, 0.0],
            influence_count: 2,
        };
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "terminal-anchor-survives-rigidification-test".to_owned(),
            source_sha256: "b".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 15,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [10.0, 0.0, 0.0],
                    [11.0, 0.0, 0.0],
                    [10.0, 1.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 6],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 6],
                indices: vec![0, 1, 2, 3, 4, 5],
                face_surface_ids: Vec::new(),
                weights: vec![
                    rigid_weight(0),
                    rigid_weight(0),
                    terminal_blend,
                    rigid_weight(2),
                    rigid_weight(2),
                    rigid_weight(2),
                ],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_15", 0, 2, 100);

        let report = refine_motion_violating_skin_weights_v1(
            &mut model,
            &contract,
            &violations,
            None,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert_eq!(report.rigid_component_count, 1);
        assert_eq!(model.segments[0].weights[0], model.segments[0].weights[1]);
        assert_eq!(model.segments[0].weights[1], model.segments[0].weights[2]);
        assert_eq!(model.segments[0].weights[0].influence_count, 1);
        assert!(
            model.segments[0].weights[3..]
                .iter()
                .all(|row| aurora_weight_for_bone_v1(row, 2) >= 0.1)
        );
    }

    #[test]
    fn motion_weight_refinement_does_not_reproject_unmeasured_geometry() {
        let untouched = [rigid_weight(0), rigid_weight(1), rigid_weight(2)];
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "measured-locality-refinement-test".to_owned(),
            source_sha256: "a".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 14,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [10.0, 0.0, 0.0],
                    [11.0, 0.0, 0.0],
                    [10.0, 1.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 6],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 6],
                indices: vec![0, 1, 2, 3, 4, 5],
                face_surface_ids: Vec::new(),
                weights: vec![
                    rigid_weight(0),
                    rigid_weight(1),
                    rigid_weight(1),
                    untouched[0].clone(),
                    untouched[1].clone(),
                    untouched[2].clone(),
                ],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_14", 0, 1, 100);

        refine_motion_violating_skin_weights_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            None,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert_eq!(&model.segments[0].weights[3..6], &untouched);
    }

    #[test]
    fn motion_weight_refinement_smooths_across_one_multi_joint_lineage() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "multi-joint-lineage-refinement-test".to_owned(),
            source_sha256: "b".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 15,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 2],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 2],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: vec![
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), None, None],
                        values: [0.8, 0.2, 0.0, 0.0],
                        influence_count: 2,
                    },
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(1), Some(2), None, None],
                        values: [0.2, 0.8, 0.0, 0.0],
                        influence_count: 2,
                    },
                ],
            }],
        };
        let original = model.segments[0].weights.clone();
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_15", 0, 1, 100);

        let report = refine_motion_violating_skin_weights_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            None,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert!(report.applied);
        assert_ne!(model.segments[0].weights, original);
    }

    #[test]
    fn motion_weight_refinement_does_not_mix_disjoint_bone_supports() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "disjoint-support-refinement-test".to_owned(),
            source_sha256: "c".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 16,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 2],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 2],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: vec![rigid_weight(0), rigid_weight(2)],
            }],
        };
        let original = model.segments[0].weights.clone();
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_16", 0, 1, 100);

        let report = refine_motion_violating_skin_weights_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            None,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert!(!report.applied);
        assert_eq!(model.segments[0].weights, original);
    }

    #[test]
    fn conversion_triangle_projection_removes_two_joint_support_span() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "projection-test".to_owned(),
            source_sha256: "4".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 9,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 3],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: vec![rigid_weight(0), rigid_weight(1), rigid_weight(2)],
            }],
        };
        let projected =
            project_conversion_triangle_support_to_one_edge_v1(&mut model, &refinement_contract())
                .unwrap();
        let support = model.segments[0]
            .weights
            .iter()
            .flat_map(aurora_weight_support_v1)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(projected, 1);
        assert!(support_is_one_carrier_edge_v1(
            &support,
            &refinement_contract()
        ));
        assert_eq!(support, vec![0, 1]);
    }

    #[test]
    fn lowest_common_carrier_ancestor_is_deepest_shared_parent() {
        let contract = refinement_contract();
        assert_eq!(
            lowest_common_carrier_ancestor_v1(&[1, 2], &contract),
            Some(1)
        );
        assert_eq!(
            lowest_common_carrier_ancestor_v1(&[0, 2], &contract),
            Some(0)
        );
        assert_eq!(lowest_common_carrier_ancestor_v1(&[], &contract), None);
    }

    #[test]
    fn measured_triangle_coherence_equalizes_parent_child_blend_values() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "coherence-test".to_owned(),
            source_sha256: "5".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 12,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [0.0, 0.001, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 3],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: vec![
                    rigid_weight(0),
                    rigid_weight(1),
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), None, None],
                        values: [0.4, 0.6, 0.0, 0.0],
                        influence_count: 2,
                    },
                ],
            }],
        };
        let initial_rows: [AuroraVertexWeightsV1; 3] = model.segments[0].weights[..3]
            .to_vec()
            .try_into()
            .expect("three rows");
        let initial_support = vec![0, 1];
        let initial_delta = maximum_triangle_weight_delta_v1(&initial_rows, &initial_support);
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_triangle("m2a_seg_12", [0, 1, 2], 40);

        let projected =
            cohere_measured_triangle_weights_v1(&mut model, &refinement_contract(), &violations)
                .unwrap();

        assert_eq!(projected, 1);
        let refined_rows: [AuroraVertexWeightsV1; 3] = model.segments[0].weights[..3]
            .to_vec()
            .try_into()
            .expect("three rows");
        assert!(maximum_triangle_weight_delta_v1(&refined_rows, &initial_support) < initial_delta);
        assert!(support_is_one_carrier_edge_v1(
            &aurora_weight_support_v1(&model.segments[0].weights[0]),
            &refinement_contract()
        ));
    }

    #[test]
    fn micro_triangle_projection_smooths_abrupt_values_on_one_carrier_edge() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "micro-triangle-test".to_owned(),
            source_sha256: "6".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 13,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [0.001, 0.0, 0.0],
                    [0.0, 0.001, 0.0],
                    [1.0, 1.0, 1.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 4],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 4],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: vec![
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), None, None],
                        values: [0.8, 0.2, 0.0, 0.0],
                        influence_count: 2,
                    },
                    rigid_weight(1),
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), None, None],
                        values: [0.4, 0.6, 0.0, 0.0],
                        influence_count: 2,
                    },
                    rigid_weight(0),
                ],
            }],
        };

        let initial_rows: [AuroraVertexWeightsV1; 3] = model.segments[0].weights[..3]
            .to_vec()
            .try_into()
            .expect("three rows");
        let initial_support = vec![0, 1];
        let initial_delta = maximum_triangle_weight_delta_v1(&initial_rows, &initial_support);
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_triangle("m2a_seg_13", [0, 1, 2], 40);
        let projected = cohere_micro_triangles_with_common_support_v1(
            &mut model,
            &refinement_contract(),
            &violations,
        )
        .unwrap();

        assert_eq!(projected.changed_count, 1);
        assert_eq!(projected.candidate_count, 1);
        let refined_rows: [AuroraVertexWeightsV1; 3] = model.segments[0].weights[..3]
            .to_vec()
            .try_into()
            .expect("three rows");
        assert!(maximum_triangle_weight_delta_v1(&refined_rows, &initial_support) < initial_delta);
        assert_eq!(
            aurora_weight_support_v1(&model.segments[0].weights[0]),
            initial_support
        );
        assert_eq!(model.segments[0].weights[1], rigid_weight(1));
    }

    #[test]
    fn discarded_motion_weight_candidate_keeps_attempt_diagnostics() {
        let attempted = MotionWeightRefinementReportV1 {
            applied: true,
            edge_count: 17,
            measured_triangle_count: 9,
            smoothing_candidate_group_count: 12,
            smoothing_shared_support_group_count: 8,
            smoothing_changed_group_count: 4,
            micro_triangle_candidate_count: 9,
            micro_triangle_too_large_count: 3,
            micro_triangle_protected_support_count: 2,
            micro_triangle_nonlocal_support_count: 1,
            micro_triangle_no_common_support_count: 1,
            micro_triangle_changed_count: 2,
            ..MotionWeightRefinementReportV1::default()
        };
        let mut explored = MotionWeightRefinementReportV1::default();
        record_motion_weight_refinement_attempt_v1(&mut explored, &attempted);
        let mut accepted = MotionWeightRefinementReportV1::default();
        copy_motion_weight_refinement_attempt_diagnostics_v1(&mut accepted, &explored);

        assert!(!accepted.applied);
        assert_eq!(accepted.attempted_round_count, 1);
        assert_eq!(accepted.examined_edge_count, 17);
        assert_eq!(accepted.measured_triangle_count, 9);
        assert_eq!(accepted.smoothing_changed_group_count, 4);
        assert_eq!(accepted.micro_triangle_changed_count, 2);
    }

    #[test]
    fn regressed_motion_edge_backtracking_reverts_only_its_position_groups() {
        let segment = crate::model_ir::AuroraModelSegmentV1 {
            segment_id: 21,
            material_slot: 0,
            deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            weights: vec![rigid_weight(0); 3],
        };
        let duplicate = crate::model_ir::AuroraModelSegmentV1 {
            segment_id: 22,
            positions: vec![[0.0, 0.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]],
            uv0: vec![[0.0, 0.0]],
            weights: vec![rigid_weight(0)],
            indices: Vec::new(),
            ..segment.clone()
        };
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "backtracking-test".to_owned(),
            source_sha256: "e".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![segment, duplicate],
        };
        let original = model.segments.clone();
        model.segments[0].weights = vec![rigid_weight(1); 3];
        model.segments[1].weights[0] = rigid_weight(1);
        let mut baseline = MotionWeightViolationAccumulatorV1::default();
        baseline.add_edge("m2a_seg_21", 0, 1, 10);
        let mut candidate = MotionWeightViolationAccumulatorV1::default();
        candidate.add_edge("m2a_seg_21", 0, 1, 20);

        let reverted = revert_regressed_motion_edge_groups_v1(
            &mut model, &original, &baseline, &candidate, None,
        )
        .unwrap();

        assert_eq!(reverted, 2);
        assert_eq!(model.segments[0].weights[0], rigid_weight(0));
        assert_eq!(model.segments[0].weights[1], rigid_weight(0));
        assert_eq!(model.segments[1].weights[0], rigid_weight(0));
        assert_eq!(model.segments[0].weights[2], rigid_weight(1));
    }

    #[test]
    fn backtracking_keeps_an_endpoint_that_improves_the_target_catastrophic_edge() {
        let segment = crate::model_ir::AuroraModelSegmentV1 {
            segment_id: 23,
            material_slot: 0,
            deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            weights: vec![rigid_weight(0); 3],
        };
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "protected-catastrophic-backtracking-test".to_owned(),
            source_sha256: "f".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![segment],
        };
        let original = model.segments.clone();
        model.segments[0].weights = vec![rigid_weight(1); 3];
        let mut baseline = MotionWeightViolationAccumulatorV1::default();
        baseline.add_edge("m2a_seg_23", 1, 2, 10);
        baseline.add_catastrophic_edge("m2a_seg_23", 0, 1, 100, false);
        let mut candidate = MotionWeightViolationAccumulatorV1::default();
        candidate.add_edge("m2a_seg_23", 1, 2, 20);
        candidate.add_catastrophic_edge("m2a_seg_23", 0, 1, 50, false);

        let reverted = revert_regressed_motion_edge_groups_v1(
            &mut model,
            &original,
            &baseline,
            &candidate,
            Some(false),
        )
        .unwrap();

        assert_eq!(reverted, 1);
        assert_eq!(model.segments[0].weights[0], rigid_weight(1));
        assert_eq!(model.segments[0].weights[1], rigid_weight(1));
        assert_eq!(model.segments[0].weights[2], rigid_weight(0));
    }

    #[test]
    fn micro_triangle_projection_rigidifies_uniform_blend_that_still_collapses() {
        let blended = AuroraVertexWeightsV1 {
            bone_node_ids: [Some(0), Some(1), None, None],
            values: [0.6, 0.4, 0.0, 0.0],
            influence_count: 2,
        };
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "uniform-micro-triangle-test".to_owned(),
            source_sha256: "d".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 17,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [0.001, 0.0, 0.0],
                    [0.0, 0.001, 0.0],
                    [1.0, 1.0, 1.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 4],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 4],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: vec![blended.clone(), blended.clone(), blended, rigid_weight(0)],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_triangle("m2a_seg_17", [0, 1, 2], 100);

        let changed = cohere_micro_triangles_with_common_support_v1(
            &mut model,
            &refinement_contract(),
            &violations,
        )
        .unwrap();

        assert_eq!(changed.changed_count, 1);
        assert_eq!(changed.candidate_count, 1);
        assert!(
            model.segments[0].weights[..3]
                .iter()
                .all(|row| aurora_weight_for_bone_v1(row, 0) > 0.6)
        );
    }

    #[test]
    fn appendage_relative_gate_blocks_rigid_surface_when_controller_chain_bends() {
        let parent = [0.0_f32, 0.5, 1.0]
            .into_iter()
            .map(|time| VisibleAnchorTrajectorySampleV1 {
                time_seconds: time,
                surface_centroid: [0.0, 0.0, 0.0],
                controller_centroid: [0.0, 0.0, 0.0],
            })
            .collect::<Vec<_>>();
        let broken_child = [
            ([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            ([1.0, 0.0, 0.0], [0.7, 0.7, 0.0]),
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ]
        .into_iter()
        .enumerate()
        .map(
            |(index, (surface, controller))| VisibleAnchorTrajectorySampleV1 {
                time_seconds: index as f32 * 0.5,
                surface_centroid: surface,
                controller_centroid: controller,
            },
        )
        .collect::<Vec<_>>();
        let broken = evaluate_appendage_relative_motion_v1(
            "tail_base",
            "tail_tip",
            &parent,
            &broken_child,
            0.2,
            0.25,
        )
        .unwrap();
        assert!(!broken.pass);
        assert_eq!(broken.surface_relative_amplitude, 0.0);
        assert!(broken.controller_relative_amplitude > 0.5);

        let moving_child = broken_child
            .iter()
            .map(|sample| VisibleAnchorTrajectorySampleV1 {
                time_seconds: sample.time_seconds,
                surface_centroid: sample.controller_centroid,
                controller_centroid: sample.controller_centroid,
            })
            .collect::<Vec<_>>();
        let moving = evaluate_appendage_relative_motion_v1(
            "tail_base",
            "tail_tip",
            &parent,
            &moving_child,
            0.2,
            0.25,
        )
        .unwrap();
        assert!(moving.pass);
        assert!((moving.amplitude_ratio - 1.0).abs() < 1.0e-6);
        assert!((moving.trajectory_alignment - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn one_broken_render_component_cannot_borrow_another_components_budget() {
        let tolerances = default_reference_supermodel_motion_tolerances_v2();
        let mut component = InheritedMotionComponentQualityV1 {
            component_index: 4,
            triangle_sample_count: 10_000,
            edge_sample_count: 30_000,
            edge_outside_hard_limit_count: 0,
            triangle_area_collapse_count: 0,
            triangle_area_expansion_count: 0,
            min_edge_ratio: 1.0,
            max_edge_ratio: 1.0,
            min_triangle_area_ratio: 1.0,
            max_triangle_area_ratio: 1.0,
            pass: false,
        };
        assert!(component_is_within_local_deformation_budget_v1(
            &component,
            &tolerances,
        ));
        component.max_edge_ratio = tolerances.edge_hard_max_ratio.powi(2) + 0.1;
        assert!(!component_is_within_local_deformation_budget_v1(
            &component,
            &tolerances,
        ));
    }

    #[test]
    fn paw_anchor_gate_accepts_grounded_idle_and_blocks_lift_side_swap_and_start_jump() {
        let context = anchor_context();
        let tolerances = default_reference_supermodel_motion_tolerances_v2();
        let grounded = sample([-1.0, 1.0, 0.0]);
        let geometry = sample_geometry_summary_v2(&grounded).unwrap();
        let mut grounded_quality = quality();
        inspect_anchor_motion_sample_v2(
            &context,
            &grounded,
            &geometry,
            true,
            true,
            true,
            None,
            None,
            &tolerances,
            &mut grounded_quality,
        )
        .unwrap();
        assert_eq!(grounded_quality.paw_contact_violation_count, 0);
        assert_eq!(grounded_quality.paw_side_violation_count, 0);
        assert_eq!(grounded_quality.clip_start_anchor_jump_violation_count, 0);

        let broken = sample([1.5, 1.0, 0.5]);
        let geometry = sample_geometry_summary_v2(&broken).unwrap();
        let mut broken_quality = quality();
        inspect_anchor_motion_sample_v2(
            &context,
            &broken,
            &geometry,
            true,
            true,
            true,
            None,
            None,
            &tolerances,
            &mut broken_quality,
        )
        .unwrap();
        assert!(broken_quality.paw_contact_violation_count > 0);
        assert!(broken_quality.paw_side_violation_count > 0);
        assert!(broken_quality.clip_start_anchor_jump_violation_count > 0);

        let expected_deltas = BTreeMap::from([
            ("left_front_paw".to_owned(), [2.5, 0.0, 0.5]),
            ("right_front_paw".to_owned(), [0.0, 0.0, 0.0]),
            ("left_rear_paw".to_owned(), [0.0, 0.0, 0.0]),
            ("right_rear_paw".to_owned(), [0.0, 0.0, 0.0]),
        ]);
        let mut expected_motion_quality = quality();
        inspect_anchor_motion_sample_v2(
            &context,
            &broken,
            &geometry,
            true,
            false,
            false,
            None,
            Some(&expected_deltas),
            &tolerances,
            &mut expected_motion_quality,
        )
        .unwrap();
        assert_eq!(
            expected_motion_quality.clip_start_anchor_jump_violation_count,
            0
        );
    }

    #[test]
    fn weighted_anchor_completeness_is_generic_and_catches_a_static_tail() {
        let assignments = BTreeMap::from([(
            "left_front_paw".to_owned(),
            vec![AnchorVertexReferenceV2 {
                skin_node_part: 10,
                vertex_index: 0,
                influence: 1.0,
            }],
        )]);
        let required = vec!["left_front_paw".to_owned(), "tail_tip".to_owned()];

        assert_eq!(
            missing_required_anchor_roles_v3(&required, &assignments),
            vec!["tail_tip".to_owned()]
        );
    }

    #[test]
    fn dense_mesh_quality_requires_a_real_triangle_domain_for_every_clip() {
        let mut clip = quality();
        clip.edge_sample_count = 1_000_000;
        clip.triangle_sample_count = 0;
        assert!(!clip_has_geometry_domain_v3(&clip));

        clip.triangle_sample_count = 1_000_000;
        assert!(clip_has_geometry_domain_v3(&clip));
    }

    #[test]
    fn catastrophic_local_edge_outlier_cannot_hide_inside_a_large_clip_budget() {
        let tolerances = default_reference_supermodel_motion_tolerances_v2();
        let mut clip = quality();
        clip.edge_sample_count = 1_000_000;
        clip.edge_soft_sample_count = 1_000_000;
        clip.triangle_sample_count = 1_000_000;
        clip.edge_outside_hard_limit_count = 5_000;
        clip.max_edge_ratio = 37.34;

        // 0.5% is inside the legacy 1% aggregate budget, but the 37.34x
        // spike is an independently blocking local deformation.
        assert!(!clip_is_within_local_deformation_budget_v3(
            &clip,
            &tolerances
        ));

        clip.max_edge_ratio = 1.0;
        assert!(clip_is_within_local_deformation_budget_v3(
            &clip,
            &tolerances
        ));
    }

    #[test]
    fn motion_weight_violation_penalty_prioritizes_catastrophic_local_outliers() {
        assert_eq!(motion_weight_violation_penalty_v1(4.01, 0.25, 4.0, 1), 2);
        assert!(
            motion_weight_violation_penalty_v1(50.0, 0.25, 4.0, 1)
                > motion_weight_violation_penalty_v1(8.0, 0.25, 4.0, 1)
        );
        assert!(
            motion_weight_violation_penalty_v1(0.01, 0.25, 4.0, 1)
                > motion_weight_violation_penalty_v1(0.1, 0.25, 4.0, 1)
        );
        assert_eq!(
            motion_weight_violation_penalty_v1(0.0, 0.25, 4.0, 1),
            1_000_000
        );
    }

    #[test]
    fn catastrophic_edge_priority_tracks_peak_severity_not_occurrence_sum() {
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge("m2a_seg_1", 0, 1, 100, false);
        violations.add_catastrophic_edge("m2a_seg_1", 0, 1, 100, false);
        violations.add_catastrophic_edge("m2a_seg_1", 2, 3, 150, false);

        assert_eq!(
            violations
                .catastrophic_collapse_edge_penalties
                .get(&("m2a_seg_1".to_owned(), 0, 1)),
            Some(&100)
        );
        assert_eq!(
            violations
                .catastrophic_collapse_edge_penalties
                .get(&("m2a_seg_1".to_owned(), 2, 3)),
            Some(&150)
        );
    }

    #[test]
    fn catastrophic_edge_witness_keeps_the_exact_worst_canonical_sample() {
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge_witness(
            "m2a_seg_7",
            9,
            3,
            100,
            false,
            "walk",
            0.5,
            0.02,
            [[9.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
            [[9.0, 1.0, 0.0], [3.0, 1.0, 0.0]],
        );
        violations.add_catastrophic_edge_witness(
            "m2a_seg_7",
            3,
            9,
            90,
            false,
            "idle",
            0.25,
            0.001,
            [[30.0, 0.0, 0.0], [90.0, 0.0, 0.0]],
            [[30.0, 1.0, 0.0], [90.0, 1.0, 0.0]],
        );

        let witnesses =
            &violations.catastrophic_collapse_edge_witnesses[&("m2a_seg_7".to_owned(), 3, 9)];
        assert_eq!(witnesses.len(), 2);
        let witness = &witnesses["walk"];
        assert_eq!(witness.penalty, 100);
        assert_eq!(witness.clip_name, "walk");
        assert_eq!(witness.bind_endpoints, [[3.0, 0.0, 0.0], [9.0, 0.0, 0.0]]);
        assert_eq!(
            witness.sampled_endpoints,
            [[3.0, 1.0, 0.0], [9.0, 1.0, 0.0]]
        );
    }

    fn two_bone_edge_motion_basis(
        sampled_bone_one_translation: f32,
    ) -> CatastrophicEdgeMotionBasisV1 {
        CatastrophicEdgeMotionBasisV1 {
            node_name: "m2a_seg_1".to_owned(),
            vertices: [0, 1],
            clip_name: "walk".to_owned(),
            sample_time_seconds: 0.5,
            measured_length_ratio: 0.0,
            bind_positions_by_bone: [
                BTreeMap::from([(0, [0.0, 0.0, 0.0]), (1, [0.0, 0.0, 0.0])]),
                BTreeMap::from([(0, [1.0, 0.0, 0.0]), (1, [1.0, 0.0, 0.0])]),
            ],
            sampled_positions_by_bone: [
                BTreeMap::from([
                    (0, [0.0, 0.0, 0.0]),
                    (1, [sampled_bone_one_translation, 0.0, 0.0]),
                ]),
                BTreeMap::from([
                    (0, [1.0, 0.0, 0.0]),
                    (1, [sampled_bone_one_translation + 1.0, 0.0, 0.0]),
                ]),
            ],
        }
    }

    #[test]
    fn measured_motion_solver_opens_a_collapsed_edge_from_the_sampled_joint_basis() {
        let basis = two_bone_edge_motion_basis(1.0);
        let original = [rigid_weight(1), rigid_weight(0)];
        assert_eq!(edge_motion_basis_ratio_v1(&original, &basis), Some(0.0));

        let repaired = solve_catastrophic_edge_from_motion_basis_v1(
            &original,
            std::slice::from_ref(&basis),
            false,
            1.0,
            None,
            0.0625,
            16.0,
        )
        .unwrap()
        .expect("the sampled joint basis must expose a collapse repair");
        let repaired_ratio = edge_motion_basis_ratio_v1(&repaired, &basis).unwrap();
        assert!(repaired_ratio > 0.0);
        assert!(repaired_ratio <= 16.0);
        assert!(
            aurora_weight_row_l1_delta_v1(&original[0], &repaired[0])
                + aurora_weight_row_l1_delta_v1(&original[1], &repaired[1])
                < 0.5
        );
        assert_ne!(repaired, original);
    }

    #[test]
    fn measured_motion_solver_uses_the_strongest_step_inside_its_trust_region() {
        let basis = two_bone_edge_motion_basis(1.0);
        let original = [rigid_weight(1), rigid_weight(0)];
        let repaired = solve_catastrophic_edge_from_motion_basis_v1(
            &original,
            std::slice::from_ref(&basis),
            false,
            0.25,
            None,
            0.0625,
            16.0,
        )
        .unwrap()
        .expect("a bounded sampled-basis repair must exist");
        let delta = aurora_weight_row_l1_delta_v1(&original[0], &repaired[0])
            + aurora_weight_row_l1_delta_v1(&original[1], &repaired[1]);

        assert!((delta - 0.25).abs() <= 1.0e-6);
        assert!(edge_motion_basis_ratio_v1(&repaired, &basis).unwrap() >= 0.125);
    }

    #[test]
    fn measured_motion_solver_line_search_scales_the_actual_trust_region() {
        let basis = two_bone_edge_motion_basis(1.0);
        let original = [rigid_weight(1), rigid_weight(0)];
        let solve = |strength| {
            solve_catastrophic_edge_from_motion_basis_v1(
                &original,
                std::slice::from_ref(&basis),
                false,
                strength,
                None,
                0.0625,
                16.0,
            )
            .unwrap()
            .expect("a bounded sampled-basis repair must exist")
        };
        let normal = solve(0.25);
        let backed_off = solve(0.125);
        let delta = |rows: &[AuroraVertexWeightsV1; 2]| {
            aurora_weight_row_l1_delta_v1(&original[0], &rows[0])
                + aurora_weight_row_l1_delta_v1(&original[1], &rows[1])
        };

        assert!((delta(&normal) - 0.25).abs() <= 1.0e-6);
        assert!((delta(&backed_off) - 0.125).abs() <= 1.0e-6);
        assert!(
            edge_motion_basis_ratio_v1(&backed_off, &basis).unwrap()
                < edge_motion_basis_ratio_v1(&normal, &basis).unwrap()
        );
    }

    #[test]
    fn full_oracle_transaction_backtrack_interpolates_the_exact_candidate_rows() {
        let segment = |weights| crate::model_ir::AuroraModelSegmentV1 {
            segment_id: 1,
            material_slot: 0,
            deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]],
            tangents: None,
            uv0: vec![[0.0, 0.0]],
            indices: Vec::new(),
            face_surface_ids: Vec::new(),
            weights: vec![weights],
        };
        let original = segment(rigid_weight(0));
        let candidate = segment(rigid_weight(1));
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "oracle-backtrack-test".to_owned(),
            source_sha256: "3".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![candidate.clone()],
        };

        let count = interpolate_motion_weight_transaction_v1(
            &mut model,
            std::slice::from_ref(&original),
            std::slice::from_ref(&candidate),
            0.25,
        )
        .unwrap();
        let row = &model.segments[0].weights[0];
        assert_eq!(count, 1);
        assert!((aurora_weight_for_bone_v1(row, 0) - 0.75).abs() <= 1.0e-6);
        assert!((aurora_weight_for_bone_v1(row, 1) - 0.25).abs() <= 1.0e-6);

        let shared = |left: f32, right: f32| AuroraVertexWeightsV1 {
            influence_count: 2,
            bone_node_ids: [Some(0), Some(1), None, None],
            values: [left, right, 0.0, 0.0],
        };
        let tiny_original = segment(shared(0.5, 0.5));
        let tiny_candidate = segment(shared(0.6, 0.4));
        model.segments = vec![tiny_candidate.clone()];
        interpolate_motion_weight_transaction_v1(
            &mut model,
            std::slice::from_ref(&tiny_original),
            std::slice::from_ref(&tiny_candidate),
            0.5_f32.powi(16),
        )
        .unwrap();
        let tiny = aurora_weight_for_bone_v1(&model.segments[0].weights[0], 0);
        assert!(tiny > 0.5 && tiny < 0.500_01);
    }

    #[test]
    fn near_fence_endpoint_protection_restores_its_complete_position_group() {
        let segment =
            |segment_id, positions: Vec<[f32; 3]>, weights| crate::model_ir::AuroraModelSegmentV1 {
                segment_id,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                normals: vec![[0.0, 0.0, 1.0]; positions.len()],
                uv0: vec![[0.0, 0.0]; positions.len()],
                positions,
                tangents: None,
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights,
            };
        let originals = vec![
            segment(
                1,
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                vec![rigid_weight(0), rigid_weight(0)],
            ),
            segment(2, vec![[0.0, 0.0, 0.0]], vec![rigid_weight(0)]),
        ];
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "fence-protection-test".to_owned(),
            source_sha256: "4".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![
                segment(
                    1,
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                    vec![rigid_weight(1), rigid_weight(1)],
                ),
                segment(2, vec![[0.0, 0.0, 0.0]], vec![rigid_weight(1)]),
            ],
        };
        let endpoints = BTreeSet::from([("m2a_seg_1".to_owned(), 0_usize)]);

        let restored = restore_motion_endpoint_rows_v1(&mut model, &originals, &endpoints).unwrap();

        assert_eq!(restored, 1);
        assert_eq!(model.segments[0].weights[0], rigid_weight(0));
        assert_eq!(model.segments[1].weights[0], rigid_weight(0));
        assert_eq!(model.segments[0].weights[1], rigid_weight(1));
    }

    #[test]
    fn absolute_regression_driver_selects_only_the_endpoint_with_the_larger_weight_delta() {
        let segment = |weights| crate::model_ir::AuroraModelSegmentV1 {
            segment_id: 7,
            material_slot: 0,
            deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 2],
            tangents: None,
            uv0: vec![[0.0, 0.0]; 2],
            indices: Vec::new(),
            face_surface_ids: Vec::new(),
            weights,
        };
        let original = segment(vec![rigid_weight(0), rigid_weight(0)]);
        let small_delta = AuroraVertexWeightsV1 {
            influence_count: 2,
            bone_node_ids: [Some(0), Some(1), None, None],
            values: [0.9, 0.1, 0.0, 0.0],
        };
        let candidate = segment(vec![small_delta, rigid_weight(1)]);
        let model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "absolute-driver-test".to_owned(),
            source_sha256: "5".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![candidate],
        };

        let selected = largest_motion_endpoint_delta_v1(
            &model,
            std::slice::from_ref(&original),
            "m2a_seg_7",
            &[0, 1],
        )
        .unwrap();

        assert_eq!(selected, Some(("m2a_seg_7".to_owned(), 1)));
    }

    #[test]
    fn measured_motion_solver_reduces_an_expanded_edge_from_the_sampled_joint_basis() {
        let basis = two_bone_edge_motion_basis(10.0);
        let original = [rigid_weight(0), rigid_weight(1)];
        assert_eq!(edge_motion_basis_ratio_v1(&original, &basis), Some(11.0));

        let repaired = solve_catastrophic_edge_from_motion_basis_v1(
            &original,
            std::slice::from_ref(&basis),
            true,
            1.0,
            None,
            0.0625,
            4.0,
        )
        .unwrap()
        .expect("the sampled joint basis must expose an expansion repair");
        let repaired_ratio = edge_motion_basis_ratio_v1(&repaired, &basis).unwrap();
        assert!(repaired_ratio < 11.0);
        assert!(repaired_ratio >= 0.0625);
    }

    #[test]
    fn measured_motion_solver_optimizes_the_worst_ratio_across_all_clip_witnesses() {
        let first = two_bone_edge_motion_basis(1.0);
        let mut second = two_bone_edge_motion_basis(0.75);
        second.clip_name = "damage".to_owned();
        let bases = [first, second];
        let original = [rigid_weight(1), rigid_weight(0)];
        let original_extreme =
            edge_motion_basis_extreme_ratio_v1(&original, &bases, false).unwrap();

        let repaired = solve_catastrophic_edge_from_motion_basis_v1(
            &original, &bases, false, 1.0, None, 0.0625, 16.0,
        )
        .unwrap()
        .expect("all clip witnesses must participate in the collapse repair");
        let repaired_extreme =
            edge_motion_basis_extreme_ratio_v1(&repaired, &bases, false).unwrap();
        assert!(repaired_extreme > original_extreme);
        assert!(repaired_extreme <= 16.0);
    }

    #[test]
    fn catastrophic_edge_coherence_moves_only_measured_endpoints_toward_each_other() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "catastrophic-edge-test".to_owned(),
            source_sha256: "7".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 11,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 2],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 2],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: vec![
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), None, None],
                        values: [0.8, 0.2, 0.0, 0.0],
                        influence_count: 2,
                    },
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(0), Some(1), None, None],
                        values: [0.2, 0.8, 0.0, 0.0],
                        influence_count: 2,
                    },
                ],
            }],
        };
        let mut half_step_model = model.clone();
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge("m2a_seg_11", 0, 1, 100, true);

        let report = cohere_catastrophic_edge_endpoints_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            true,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert_eq!(report.changed_count, 1);
        assert!(report.peak_edge_changed);
        let left = &model.segments[0].weights[0];
        let right = &model.segments[0].weights[1];
        assert!(aurora_weight_for_bone_v1(left, 0) < 0.8);
        assert!(aurora_weight_for_bone_v1(right, 0) > 0.2);
        assert!(
            (aurora_weight_for_bone_v1(left, 0) - aurora_weight_for_bone_v1(right, 0)).abs() < 1.0
        );
        cohere_catastrophic_edge_endpoints_v1(
            &mut half_step_model,
            &refinement_contract(),
            &violations,
            true,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1 * 0.5,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();
        assert!(
            aurora_weight_for_bone_v1(&half_step_model.segments[0].weights[0], 0)
                > aurora_weight_for_bone_v1(left, 0),
            "a smaller coherence step must retain more of the original endpoint weight"
        );
    }

    #[test]
    fn catastrophic_full_lineage_target_preserves_mean_while_reducing_gradient() {
        let row = |child, parent, root| AuroraVertexWeightsV1 {
            bone_node_ids: [Some(2), Some(1), Some(0), None],
            values: [child, parent, root, 0.0],
            influence_count: 3,
        };
        let original = [row(0.56, 0.25, 0.19), row(0.59, 0.24, 0.17)];
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "catastrophic-full-lineage-target-test".to_owned(),
            source_sha256: "5".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 15,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 2],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 2],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: original.to_vec(),
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge("m2a_seg_15", 0, 1, 100, false);

        cohere_catastrophic_edge_endpoints_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::FullCarrierLineage,
            None,
        )
        .unwrap();

        let repaired = &model.segments[0].weights;
        for bone in [0, 1, 2] {
            let before = original
                .iter()
                .map(|weights| aurora_weight_for_bone_v1(weights, bone))
                .collect::<Vec<_>>();
            let after = [
                aurora_weight_for_bone_v1(&repaired[0], bone),
                aurora_weight_for_bone_v1(&repaired[1], bone),
            ];
            assert!(((after[0] + after[1]) - (before[0] + before[1])).abs() < 1.0e-6);
            assert!((after[0] - after[1]).abs() < (before[0] - before[1]).abs());
        }
    }

    #[test]
    fn catastrophic_edge_coherence_repairs_only_the_peak_measured_edge_per_transaction() {
        let blended_weight = |left, right| AuroraVertexWeightsV1 {
            bone_node_ids: [Some(0), Some(1), None, None],
            values: [left, right, 0.0, 0.0],
            influence_count: 2,
        };
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "catastrophic-shared-group-test".to_owned(),
            source_sha256: "6".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 14,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 3],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: vec![
                    blended_weight(0.8, 0.2),
                    blended_weight(0.2, 0.8),
                    blended_weight(0.8, 0.2),
                ],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge("m2a_seg_14", 0, 1, 200, false);
        violations.add_catastrophic_edge("m2a_seg_14", 1, 2, 100, false);

        let report = cohere_catastrophic_edge_endpoints_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert_eq!(report.changed_count, 1);
        assert_eq!(report.conflict_skip_count, 0);
        assert_eq!(model.segments[0].weights[2], blended_weight(0.8, 0.2));
    }

    #[test]
    fn measured_motion_batch_repairs_multiple_disjoint_peaks_in_one_transaction() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "catastrophic-measured-batch-test".to_owned(),
            source_sha256: "4".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [2.0, 0.0, 0.0],
                    [3.0, 0.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 4],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 4],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: vec![
                    rigid_weight(1),
                    rigid_weight(0),
                    rigid_weight(1),
                    rigid_weight(0),
                ],
            }],
        };
        let original = model.segments[0].weights.clone();
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge("m2a_seg_1", 0, 1, 200, false);
        violations.add_catastrophic_edge("m2a_seg_1", 2, 3, 150, false);
        let first = two_bone_edge_motion_basis(1.0);
        let mut second = two_bone_edge_motion_basis(1.0);
        second.vertices = [2, 3];
        let basis_sets = [
            CatastrophicEdgeMotionBasisSetV1 {
                node_name: "m2a_seg_1".to_owned(),
                vertices: [0, 1],
                bases: vec![first],
            },
            CatastrophicEdgeMotionBasisSetV1 {
                node_name: "m2a_seg_1".to_owned(),
                vertices: [2, 3],
                bases: vec![second],
            },
        ];

        let report = cohere_catastrophic_edge_endpoints_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::MeasuredMotionBasis,
            Some(&basis_sets),
        )
        .unwrap();

        assert_eq!(report.changed_count, 2);
        assert!(report.peak_edge_changed);
        assert!(
            model.segments[0].weights[0] != original[0]
                || model.segments[0].weights[1] != original[1]
        );
        assert!(
            model.segments[0].weights[2] != original[2]
                || model.segments[0].weights[3] != original[3]
        );
    }

    #[test]
    fn measured_catastrophic_edge_can_add_adjacent_support_after_generic_damping() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "catastrophic-support-bridge-test".to_owned(),
            source_sha256: "9".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 13,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 2],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 2],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: vec![rigid_weight(0), rigid_weight(1)],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_edge("m2a_seg_13", 0, 1, 100);
        violations.add_catastrophic_edge("m2a_seg_13", 0, 1, 100, false);

        let report = refine_motion_violating_skin_weights_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            Some(false),
            false,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::Symmetric,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert!(report.applied);
        assert_eq!(report.catastrophic_edge_count, 1);
        assert_eq!(report.smoothing_changed_group_count, 0);
        assert_eq!(report.micro_triangle_changed_count, 0);
        assert!(aurora_weight_for_bone_v1(&model.segments[0].weights[0], 1) > 0.0);
        assert!(aurora_weight_for_bone_v1(&model.segments[0].weights[1], 0) > 0.0);
    }

    #[test]
    fn catastrophic_edge_coherence_uses_child_side_at_a_three_joint_seam() {
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "catastrophic-seam-test".to_owned(),
            source_sha256: "8".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 12,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [-1.0, 1.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 6],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 6],
                indices: vec![0, 1, 2, 3, 4, 5],
                face_surface_ids: Vec::new(),
                weights: vec![
                    rigid_weight(1),
                    rigid_weight(2),
                    rigid_weight(2),
                    rigid_weight(1),
                    rigid_weight(0),
                    rigid_weight(0),
                ],
            }],
        };
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_edge("m2a_seg_12", 0, 1, 100, true);

        let report = cohere_catastrophic_edge_endpoints_v1(
            &mut model,
            &refinement_contract(),
            &violations,
            true,
            CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1,
            CatastrophicEndpointPolicyV1::DeeperOnly,
            CatastrophicTargetPolicyV1::ProjectedCarrierEdge,
            None,
        )
        .unwrap();

        assert_eq!(report.changed_count, 1);
        assert!(report.peak_edge_changed);
        assert_eq!(model.segments[0].weights[0], rigid_weight(1));
        assert_eq!(model.segments[0].weights[3], rigid_weight(1));
        assert!(aurora_weight_for_bone_v1(&model.segments[0].weights[1], 1) > 0.0);
        assert!(aurora_weight_for_bone_v1(&model.segments[0].weights[1], 2) < 1.0);
        assert_eq!(model.segments[0].weights[2], rigid_weight(2));
    }

    #[test]
    fn targeted_catastrophic_round_selects_the_worse_normalized_edge_axis() {
        assert!(targeted_catastrophic_expansion_v1([2.5, 1.8, 0.1, 0.1]));
        assert!(!targeted_catastrophic_expansion_v1([1.2, 3.0, 0.1, 0.1]));
        assert!(targeted_catastrophic_expansion_v1([2.0, 2.0, 9.0, 9.0]));
    }

    #[test]
    fn targeted_triangle_blend_search_is_bounded_and_monotonic() {
        let mut blend = TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1;
        let mut steps = 0usize;
        while advance_targeted_triangle_blend_search_v1(&mut blend) {
            steps += 1;
        }
        assert_eq!(blend, TARGETED_TRIANGLE_MINIMUM_RIGID_BLEND_V1);
        assert_eq!(steps, 7);
    }

    #[test]
    fn catastrophic_triangle_transaction_rigidifies_only_the_measured_triangle() {
        let shared = AuroraVertexWeightsV1 {
            bone_node_ids: [Some(1), Some(0), None, None],
            values: [0.55, 0.45, 0.0, 0.0],
            influence_count: 2,
        };
        let mut third = shared.clone();
        third.bone_node_ids[2] = Some(2);
        third.values = [0.53, 0.44, 0.03, 0.0];
        third.influence_count = 3;
        let untouched = rigid_weight(2);
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "targeted-triangle-test".to_owned(),
            source_sha256: "a".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![crate::model_ir::AuroraModelSegmentV1 {
                segment_id: 4,
                material_slot: 0,
                deformation: crate::model_ir::AuroraSegmentDeformationV1::Skin,
                parent_node_id: 0,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [10.0, 0.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 4],
                tangents: None,
                uv0: vec![[0.0, 0.0]; 4],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: vec![shared.clone(), shared, third, untouched.clone()],
            }],
        };
        let before = model.segments[0].weights.clone();
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_catastrophic_triangle("m2a_seg_4", [0, 1, 2], 100, false);

        let report = cohere_catastrophic_triangle_toward_common_carrier_v1(
            &mut model,
            &violations,
            false,
            TARGETED_TRIANGLE_INITIAL_RIGID_BLEND_V1,
        )
        .unwrap();

        assert_eq!(report.changed_count, 1);
        for vertex in 0..3 {
            assert!(
                aurora_weight_for_bone_v1(&model.segments[0].weights[vertex], 1)
                    > aurora_weight_for_bone_v1(&before[vertex], 1)
            );
        }
        assert_eq!(model.segments[0].weights[3], untouched);
    }

    #[test]
    fn carrier_lineage_accepts_ancestor_chain_and_rejects_sibling_branch() {
        let contract = refinement_contract();
        assert!(support_is_one_carrier_lineage_v1(&[0, 1, 2], &contract));
        let mut branched = contract.clone();
        branched.nodes.push(ReferenceSupermodelMotionNodeV2 {
            part_number: 3,
            name: "sibling".to_owned(),
            parent_part_number: Some(1),
            carrier_bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            position_controller_required: false,
            orientation_controller_required: false,
            scale_controller_required: false,
            anchor_role: None,
            joint_axis: None,
            carrier_class: ReferenceSupermodelCarrierClassV3::SkinRelevant,
            structural_role: "chain".to_owned(),
            controlling_clips: Vec::new(),
            dynamic_clips: Vec::new(),
        });
        assert!(!support_is_one_carrier_lineage_v1(&[2, 3], &branched));
    }

    #[test]
    fn blocking_extrema_improvement_precedes_nonblocking_violation_count_optimization() {
        assert!(blocking_local_extrema_strictly_improve_v1(
            [0.990_819_5, 3.932_649_1, 0.392_570_67, 0.548_131_05],
            [0.990_819_5, 2.850_620_5, 0.392_570_67, 0.548_131_05],
        ));
        assert!(!blocking_local_extrema_strictly_improve_v1(
            [0.9, 1.0, 0.4, 0.6],
            [0.8, 0.9, 0.3, 0.5],
        ));
        assert!(!blocking_local_extrema_strictly_improve_v1(
            [0.9, 2.0, 0.4, 0.6],
            [0.9, 2.0, 0.4, 0.6],
        ));
    }

    #[test]
    fn catastrophic_plateau_order_accepts_fewer_tied_peaks_without_accepting_a_worse_peak() {
        assert!(descending_penalty_distribution_strictly_improves_v1(
            [100, 100, 80],
            [100, 90, 90],
        ));
        assert!(descending_penalty_distribution_strictly_improves_v1(
            [100, 90, 80],
            [99, 99, 99],
        ));
        assert!(!descending_penalty_distribution_strictly_improves_v1(
            [100, 90, 80],
            [101, 1],
        ));
        assert!(!descending_penalty_distribution_strictly_improves_v1(
            [100, 90, 80],
            [100, 90, 80],
        ));
    }

    #[test]
    fn targeted_line_search_backs_off_when_selected_axis_improves_but_another_regresses() {
        assert!(targeted_extrema_tradeoff_needs_backoff_v1(
            [0.990_819_5, 2.540_462_7, 0.392_570_67, 0.548_131_05],
            [1.919_321_1, 2.256_31, 0.392_570_67, 0.548_131_05],
            false,
        ));
        assert!(!targeted_extrema_tradeoff_needs_backoff_v1(
            [0.990_819_5, 2.540_462_7, 0.392_570_67, 0.548_131_05],
            [0.990_819_5, 2.256_31, 0.392_570_67, 0.548_131_05],
            false,
        ));
        assert!(!targeted_extrema_tradeoff_needs_backoff_v1(
            [0.990_819_5, 2.540_462_7, 0.392_570_67, 0.548_131_05],
            [1.919_321_1, 2.540_462_7, 0.392_570_67, 0.548_131_05],
            false,
        ));
    }

    #[test]
    fn targeted_flat_axis_search_strengthens_each_generic_target_before_switching_endpoint() {
        let original = [0.940_401_5, 1.731_61, 0.392_570_67, 0.548_131_05];
        let candidate = original;
        let mut blend = CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1;
        let mut endpoint = CatastrophicEndpointPolicyV1::Symmetric;
        let mut target = CatastrophicTargetPolicyV1::MeasuredMotionBasis;

        assert!(advance_targeted_refinement_search_extrema_v1(
            original,
            candidate,
            false,
            false,
            &mut blend,
            &mut endpoint,
            &mut target,
        ));
        assert_eq!(blend, 0.05);
        assert_eq!(target, CatastrophicTargetPolicyV1::MeasuredMotionBasis);
        assert_eq!(endpoint, CatastrophicEndpointPolicyV1::Symmetric);

        assert!(advance_targeted_refinement_search_extrema_v1(
            original,
            candidate,
            false,
            false,
            &mut blend,
            &mut endpoint,
            &mut target,
        ));
        assert_eq!(blend, CATASTROPHIC_EDGE_MAXIMUM_COHERENCE_BLEND_V1);
        assert!(advance_targeted_refinement_search_extrema_v1(
            original,
            candidate,
            false,
            false,
            &mut blend,
            &mut endpoint,
            &mut target,
        ));
        assert_eq!(blend, CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1);
        assert_eq!(target, CatastrophicTargetPolicyV1::ProjectedCarrierEdge);

        blend = CATASTROPHIC_EDGE_MAXIMUM_COHERENCE_BLEND_V1;
        assert!(advance_targeted_refinement_search_extrema_v1(
            original,
            candidate,
            false,
            false,
            &mut blend,
            &mut endpoint,
            &mut target,
        ));
        assert_eq!(target, CatastrophicTargetPolicyV1::FullCarrierLineage);

        blend = CATASTROPHIC_EDGE_MAXIMUM_COHERENCE_BLEND_V1;
        assert!(advance_targeted_refinement_search_extrema_v1(
            original,
            candidate,
            false,
            false,
            &mut blend,
            &mut endpoint,
            &mut target,
        ));
        assert_eq!(endpoint, CatastrophicEndpointPolicyV1::DeeperOnly);
        assert_eq!(target, CatastrophicTargetPolicyV1::MeasuredMotionBasis);
        assert_eq!(blend, CATASTROPHIC_EDGE_INITIAL_COHERENCE_BLEND_V1);
    }

    #[test]
    fn adaptive_time_budget_keeps_mandatory_events_and_fills_largest_gaps() {
        let sampled = bounded_adaptive_sample_times_v1(
            vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.61],
            (0..=100).map(|index| index as f32 / 100.0).collect(),
            9,
        );
        assert_eq!(sampled.len(), 9);
        for mandatory in [0.0, 0.25, 0.5, 0.61, 0.75, 1.0] {
            assert!(sampled.contains(&mandatory));
        }
        assert!(sampled.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn generic_density_backtracking_can_probe_one_improved_axis() {
        assert!(any_motion_density_count_strictly_improves_v1(
            [314, 69_740, 690],
            [318, 68_256, 689],
        ));
        assert!(!any_motion_density_count_strictly_improves_v1(
            [314, 69_740, 690],
            [318, 69_740, 691],
        ));
    }

    #[test]
    fn component_density_objective_rewards_reduced_blocking_excess_only() {
        let tolerances = default_reference_supermodel_motion_tolerances_v2();
        let samples = 729_648;
        let original = [9_074, 7_758, 8_272, 8_543, 7_848]
            .into_iter()
            .map(|count| {
                normalized_density_excess_v1(count, samples, tolerances.edge_hard_max_fraction)
            })
            .sum::<f64>();
        let candidate = [8_864, 7_615, 8_147, 8_303, 7_677]
            .into_iter()
            .map(|count| {
                normalized_density_excess_v1(count, samples, tolerances.edge_hard_max_fraction)
            })
            .sum::<f64>();

        assert!(candidate < original);
        assert_eq!(
            normalized_density_excess_v1(
                342,
                10_242_096,
                tolerances.triangle_area_expansion_max_fraction,
            ),
            0.0
        );
    }

    #[test]
    fn component_local_refinement_excludes_passing_islands() {
        let mut violations = MotionWeightViolationAccumulatorV1::default();
        violations.add_component_edge("walk", 4, "m2a_seg_1", 1, 2, 3);
        violations.add_component_edge("walk", 5, "m2a_seg_1", 3, 4, 9);
        violations.add_component_triangle("walk", 4, "m2a_seg_1", [5, 6, 7], 4);
        violations.mark_failed_component("walk", 4);

        let subset = violations.failed_component_subset_v1();
        assert_eq!(
            subset.edge_penalties.get(&("m2a_seg_1".to_owned(), 1, 2)),
            Some(&3)
        );
        assert!(
            !subset
                .edge_penalties
                .contains_key(&("m2a_seg_1".to_owned(), 3, 4))
        );
        assert_eq!(
            subset
                .triangle_penalties
                .get(&("m2a_seg_1".to_owned(), [5, 6, 7])),
            Some(&4)
        );
    }
}
