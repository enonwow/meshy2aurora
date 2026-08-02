//! Higher-level animation authoring primitives: combat phase timelines,
//! motion trails/onion poses, combat-quality telemetry and immutable-source
//! procedural variants. Every output remains an ordinary LINEAR authored clip.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_authoring_v2::{AnimationWorldPoseV1, sample_animation_world_pose_v1},
    animation_pose::{HumanoidMirrorMapV1, reflect_pose_rotation_v1, reflect_pose_translation_v1},
    animation_retarget::rig_signature_sha256_v1,
    animation_studio::{
        AnimationStudioRigV1, AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1,
        AuthoredAnimationSourceKindV1, AuthoredAnimationTrackPathV1,
        retime_authored_animation_clip_v1,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CombatAttackPhaseKindV1 {
    Ready,
    WindUp,
    Strike,
    Impact,
    Recovery,
}

impl CombatAttackPhaseKindV1 {
    const ORDERED: [Self; 5] = [
        Self::Ready,
        Self::WindUp,
        Self::Strike,
        Self::Impact,
        Self::Recovery,
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatAttackPhaseMarkerV1 {
    pub phase: CombatAttackPhaseKindV1,
    pub time_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatAttackPhaseTimelineV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub clip_id: String,
    pub clip_revision: u64,
    pub markers: Vec<CombatAttackPhaseMarkerV1>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CombatQualitySeverityV1 {
    Info,
    Warning,
    Blocking,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CombatQualityIssueKindV1 {
    PhaseStructure,
    WeakAnticipation,
    ImpactTiming,
    AttackArcReversal,
    RecoveryMismatch,
    RootInstability,
    ContactInstability,
    GripError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatQualityIssueV1 {
    pub kind: CombatQualityIssueKindV1,
    pub severity: CombatQualitySeverityV1,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub measured: f32,
    pub threshold: f32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatQualityPolicyV1 {
    pub schema_version: u32,
    pub sample_rate_hz: u32,
    pub minimum_phase_seconds: f32,
    pub minimum_anticipation_per_height: f32,
    pub maximum_impact_peak_offset_seconds: f32,
    pub maximum_direction_reversals: u32,
    pub maximum_recovery_error_per_height: f32,
    pub maximum_root_displacement_per_height: f32,
    pub maximum_grip_error_per_height: f32,
}

impl Default for CombatQualityPolicyV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            sample_rate_hz: 60,
            minimum_phase_seconds: 0.02,
            minimum_anticipation_per_height: 0.015,
            maximum_impact_peak_offset_seconds: 0.12,
            maximum_direction_reversals: 2,
            maximum_recovery_error_per_height: 0.08,
            maximum_root_displacement_per_height: 0.20,
            maximum_grip_error_per_height: 0.04,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatQualityContextV1 {
    pub schema_version: u32,
    pub attack_node_id: u32,
    pub root_node_id: u32,
    pub grip_max_error: Option<f32>,
    #[serde(default)]
    pub contact_node_ids: Vec<u32>,
    #[serde(default)]
    pub contribution_node_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatBodyContributionV1 {
    pub node_id: u32,
    pub displacement: f32,
    pub normalized_share: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatQualityReportV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub clip_id: String,
    pub phase_fingerprint_sha256: String,
    pub skeleton_height: f32,
    pub anticipation_displacement: f32,
    pub attack_arc_length: f32,
    pub peak_speed: f32,
    pub peak_speed_time_seconds: f32,
    pub impact_peak_offset_seconds: f32,
    pub direction_reversals: u32,
    pub recovery_error: f32,
    pub root_displacement: f32,
    pub impact_contact_displacement: f32,
    pub body_contributions: Vec<CombatBodyContributionV1>,
    pub issues: Vec<CombatQualityIssueV1>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionTrailRequestV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub clip_id: String,
    pub clip_revision: u64,
    pub node_ids: Vec<u32>,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub sample_count: usize,
    pub onion_before: usize,
    pub onion_after: usize,
    pub current_time_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionTrailPointV1 {
    pub time_seconds: f32,
    pub translation: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionTrailV1 {
    pub node_id: u32,
    pub points: Vec<MotionTrailPointV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OnionPoseV1 {
    pub offset_samples: i32,
    pub time_seconds: f32,
    pub pose: AnimationWorldPoseV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionVisualizationV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub clip_id: String,
    pub clip_revision: u64,
    pub trails: Vec<MotionTrailV1>,
    pub onion_poses: Vec<OnionPoseV1>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum AnimationVariantOperationV1 {
    Speed { factor: f32 },
    Mirror { mapping: HumanoidMirrorMapV1 },
    Amplitude { factor: f32, node_ids: Vec<u32> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationVariantRecipeV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub source_clip_id: String,
    pub source_clip_revision: u64,
    pub source_clip_fingerprint_sha256: String,
    pub output_clip_id: String,
    pub output_name: String,
    pub inherited_tags: Vec<String>,
    pub added_tags: Vec<String>,
    pub operations: Vec<AnimationVariantOperationV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationVariantResultV1 {
    pub schema_version: u32,
    pub recipe_fingerprint_sha256: String,
    pub effective_tags: Vec<String>,
    pub clip: AuthoredAnimationClipV1,
    pub fingerprint_sha256: String,
}

pub fn create_combat_attack_phase_timeline_v1(
    clip: &AuthoredAnimationClipV1,
    phase_times: [f32; 5],
) -> Result<CombatAttackPhaseTimelineV1, String> {
    let markers = CombatAttackPhaseKindV1::ORDERED
        .into_iter()
        .zip(phase_times)
        .map(|(phase, time_seconds)| CombatAttackPhaseMarkerV1 {
            phase,
            time_seconds: canonical_f32(time_seconds),
        })
        .collect::<Vec<_>>();
    validate_phase_markers(clip, &markers, 0.0)?;
    let fingerprint_sha256 = fingerprint_json(&(
        1_u32,
        clip.source.source_revision.as_str(),
        clip.id.as_str(),
        clip.revision,
        &markers,
    ));
    Ok(CombatAttackPhaseTimelineV1 {
        schema_version: 1,
        source_revision: clip.source.source_revision.clone(),
        clip_id: clip.id.clone(),
        clip_revision: clip.revision,
        markers,
        fingerprint_sha256,
    })
}

pub fn retime_combat_attack_phases_v1(
    clip: &AuthoredAnimationClipV1,
    source: &CombatAttackPhaseTimelineV1,
    target: &CombatAttackPhaseTimelineV1,
) -> Result<AuthoredAnimationClipV1, String> {
    validate_timeline_identity(clip, source, false)?;
    if target.schema_version != 1
        || target.source_revision != clip.source.source_revision
        || target.clip_id != clip.id
    {
        return Err("target phase timeline is stale for the exact clip".into());
    }
    validate_phase_markers(clip, &target.markers, 0.0)?;
    let mut output = clip.clone();
    for track in &mut output.tracks {
        for key in &mut track.keyframes {
            key.time_seconds = map_phase_time(key.time_seconds, &source.markers, &target.markers);
        }
    }
    for event in &mut output.events {
        event.time_seconds = map_phase_time(event.time_seconds, &source.markers, &target.markers);
    }
    output.revision = output.revision.saturating_add(1);
    output.status = AuthoredAnimationClipStatusV1::Draft;
    Ok(output)
}

pub fn analyze_combat_animation_quality_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    timeline: &CombatAttackPhaseTimelineV1,
    policy: &CombatQualityPolicyV1,
    context: &CombatQualityContextV1,
) -> Result<CombatQualityReportV1, String> {
    validate_timeline_identity(clip, timeline, true)?;
    validate_combat_policy(policy, context)?;
    let known = rig
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<BTreeSet<_>>();
    if clip.source.source_revision != rig.source_revision
        || !known.contains(&context.attack_node_id)
        || !known.contains(&context.root_node_id)
        || context
            .contact_node_ids
            .iter()
            .any(|id| !known.contains(id))
        || context
            .contribution_node_ids
            .iter()
            .any(|id| !known.contains(id))
    {
        return Err("combat context references a stale clip or missing exact-rig node".into());
    }
    validate_phase_markers(clip, &timeline.markers, policy.minimum_phase_seconds)?;
    let height = skeleton_height(rig).max(1.0e-6);
    let marker_times = timeline
        .markers
        .iter()
        .map(|marker| marker.time_seconds)
        .collect::<Vec<_>>();
    let marker_poses = marker_times
        .iter()
        .map(|time| sample_animation_world_pose_v1(clip, rig, *time).map_err(format_diagnostics))
        .collect::<Result<Vec<_>, _>>()?;
    let attack_positions = marker_poses
        .iter()
        .map(|pose| bone_position(pose, context.attack_node_id))
        .collect::<Result<Vec<_>, _>>()?;
    let root_positions = marker_poses
        .iter()
        .map(|pose| bone_position(pose, context.root_node_id))
        .collect::<Result<Vec<_>, _>>()?;
    let anticipation_displacement = distance3(attack_positions[0], attack_positions[1]);
    let recovery_error = distance3(attack_positions[0], attack_positions[4]);
    let root_displacement = distance3(root_positions[0], root_positions[3]);
    let impact_contact_displacement = context
        .contact_node_ids
        .iter()
        .map(|node_id| {
            Ok(distance3(
                bone_position(&marker_poses[2], *node_id)?,
                bone_position(&marker_poses[3], *node_id)?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .fold(0.0_f32, f32::max);
    let mut body_contributions = context
        .contribution_node_ids
        .iter()
        .map(|node_id| {
            Ok(CombatBodyContributionV1 {
                node_id: *node_id,
                displacement: distance3(
                    bone_position(&marker_poses[1], *node_id)?,
                    bone_position(&marker_poses[3], *node_id)?,
                ),
                normalized_share: 0.0,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let contribution_total = body_contributions
        .iter()
        .map(|entry| entry.displacement)
        .sum::<f32>();
    for entry in &mut body_contributions {
        entry.normalized_share = canonical_f32(if contribution_total <= 1.0e-8 {
            0.0
        } else {
            entry.displacement / contribution_total
        });
        entry.displacement = canonical_f32(entry.displacement);
    }

    let strike_start = marker_times[2];
    let recovery_end = marker_times[4];
    let sample_count =
        (((recovery_end - strike_start) * policy.sample_rate_hz as f32).ceil() as usize + 1)
            .clamp(3, 4096);
    let times = (0..sample_count)
        .map(|index| {
            strike_start + (recovery_end - strike_start) * index as f32 / (sample_count - 1) as f32
        })
        .collect::<Vec<_>>();
    let positions = times
        .iter()
        .map(|time| sample_animation_world_pose_v1(clip, rig, *time).map_err(format_diagnostics))
        .map(|pose| pose.and_then(|pose| bone_position(&pose, context.attack_node_id)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut attack_arc_length = 0.0_f32;
    let mut peak_speed = 0.0_f32;
    let mut peak_speed_time_seconds = strike_start;
    let mut direction_reversals = 0_u32;
    let mut previous_direction: Option<[f32; 3]> = None;
    for index in 1..positions.len() {
        let delta = subtract3(positions[index], positions[index - 1]);
        let segment = length3(delta);
        attack_arc_length += segment;
        let speed = segment / (times[index] - times[index - 1]).max(1.0e-6);
        if speed > peak_speed {
            peak_speed = speed;
            peak_speed_time_seconds = times[index];
        }
        if segment > height * 1.0e-5 {
            let direction = scale3(delta, 1.0 / segment);
            if previous_direction.is_some_and(|previous| dot3(previous, direction) < -0.25) {
                direction_reversals = direction_reversals.saturating_add(1);
            }
            previous_direction = Some(direction);
        }
    }
    let impact_time = marker_times[3];
    let impact_peak_offset_seconds = (peak_speed_time_seconds - impact_time).abs();
    let mut issues = Vec::new();
    push_warning_if(
        &mut issues,
        anticipation_displacement / height < policy.minimum_anticipation_per_height,
        CombatQualityIssueKindV1::WeakAnticipation,
        marker_times[0],
        marker_times[1],
        anticipation_displacement / height,
        policy.minimum_anticipation_per_height,
        "attack endpoint barely separates READY from WIND_UP",
    );
    push_warning_if(
        &mut issues,
        impact_peak_offset_seconds > policy.maximum_impact_peak_offset_seconds,
        CombatQualityIssueKindV1::ImpactTiming,
        strike_start,
        impact_time,
        impact_peak_offset_seconds,
        policy.maximum_impact_peak_offset_seconds,
        "peak endpoint speed is too far from the declared IMPACT phase",
    );
    push_warning_if(
        &mut issues,
        direction_reversals > policy.maximum_direction_reversals,
        CombatQualityIssueKindV1::AttackArcReversal,
        strike_start,
        recovery_end,
        direction_reversals as f32,
        policy.maximum_direction_reversals as f32,
        "attack arc changes direction repeatedly",
    );
    push_warning_if(
        &mut issues,
        recovery_error / height > policy.maximum_recovery_error_per_height,
        CombatQualityIssueKindV1::RecoveryMismatch,
        impact_time,
        recovery_end,
        recovery_error / height,
        policy.maximum_recovery_error_per_height,
        "RECOVERY remains far from the READY endpoint pose",
    );
    push_warning_if(
        &mut issues,
        root_displacement / height > policy.maximum_root_displacement_per_height,
        CombatQualityIssueKindV1::RootInstability,
        marker_times[0],
        impact_time,
        root_displacement / height,
        policy.maximum_root_displacement_per_height,
        "root displacement before IMPACT exceeds the configured combat guide",
    );
    let contact_threshold = policy.maximum_root_displacement_per_height * 0.25;
    push_warning_if(
        &mut issues,
        impact_contact_displacement / height > contact_threshold,
        CombatQualityIssueKindV1::ContactInstability,
        marker_times[2],
        impact_time,
        impact_contact_displacement / height,
        contact_threshold,
        "declared foot contact moves excessively between STRIKE and IMPACT",
    );
    if let Some(grip) = context.grip_max_error {
        push_warning_if(
            &mut issues,
            grip / height > policy.maximum_grip_error_per_height,
            CombatQualityIssueKindV1::GripError,
            marker_times[0],
            recovery_end,
            grip / height,
            policy.maximum_grip_error_per_height,
            "held equipment grip error exceeds the configured guide",
        );
    }
    let fingerprint_sha256 = fingerprint_json(&(
        clip.id.as_str(),
        clip.revision,
        timeline.fingerprint_sha256.as_str(),
        policy,
        context,
        anticipation_displacement,
        attack_arc_length,
        peak_speed,
        peak_speed_time_seconds,
        direction_reversals,
        recovery_error,
        root_displacement,
        impact_contact_displacement,
        &body_contributions,
        &issues,
    ));
    Ok(CombatQualityReportV1 {
        schema_version: 1,
        source_revision: rig.source_revision.clone(),
        clip_id: clip.id.clone(),
        phase_fingerprint_sha256: timeline.fingerprint_sha256.clone(),
        skeleton_height: canonical_f32(height),
        anticipation_displacement: canonical_f32(anticipation_displacement),
        attack_arc_length: canonical_f32(attack_arc_length),
        peak_speed: canonical_f32(peak_speed),
        peak_speed_time_seconds: canonical_f32(peak_speed_time_seconds),
        impact_peak_offset_seconds: canonical_f32(impact_peak_offset_seconds),
        direction_reversals,
        recovery_error: canonical_f32(recovery_error),
        root_displacement: canonical_f32(root_displacement),
        impact_contact_displacement: canonical_f32(impact_contact_displacement),
        body_contributions,
        issues,
        fingerprint_sha256,
    })
}

pub fn build_motion_visualization_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    request: &MotionTrailRequestV1,
) -> Result<MotionVisualizationV1, String> {
    if request.schema_version != 1
        || request.source_revision != rig.source_revision
        || request.source_revision != clip.source.source_revision
        || request.clip_id != clip.id
        || request.clip_revision != clip.revision
        || request.node_ids.is_empty()
        || request.node_ids.len() > 32
        || request.sample_count < 2
        || request.sample_count > 512
        || request.onion_before > 8
        || request.onion_after > 8
        || !request.start_seconds.is_finite()
        || !request.end_seconds.is_finite()
        || request.start_seconds < 0.0
        || request.end_seconds <= request.start_seconds
        || request.end_seconds > clip.length_seconds
        || request.current_time_seconds < request.start_seconds
        || request.current_time_seconds > request.end_seconds
    {
        return Err("motion visualization request is invalid or stale".into());
    }
    let node_ids = request.node_ids.iter().copied().collect::<BTreeSet<_>>();
    if node_ids.len() != request.node_ids.len()
        || node_ids
            .iter()
            .any(|id| !rig.nodes.iter().any(|node| node.node_id == *id))
    {
        return Err("motion trail node selection is duplicate or absent from the exact rig".into());
    }
    let times = (0..request.sample_count)
        .map(|index| {
            request.start_seconds
                + (request.end_seconds - request.start_seconds) * index as f32
                    / (request.sample_count - 1) as f32
        })
        .collect::<Vec<_>>();
    let poses = times
        .iter()
        .map(|time| sample_animation_world_pose_v1(clip, rig, *time).map_err(format_diagnostics))
        .collect::<Result<Vec<_>, _>>()?;
    let trails = request
        .node_ids
        .iter()
        .map(|node_id| {
            let points = poses
                .iter()
                .zip(&times)
                .map(|(pose, time)| {
                    Ok(MotionTrailPointV1 {
                        time_seconds: canonical_f32(*time),
                        translation: bone_position(pose, *node_id)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(MotionTrailV1 {
                node_id: *node_id,
                points,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let step = (request.end_seconds - request.start_seconds) / (request.sample_count - 1) as f32;
    let onion_poses = (-(request.onion_before as i32)..=(request.onion_after as i32))
        .filter(|offset| *offset != 0)
        .map(|offset| {
            let time = (request.current_time_seconds + offset as f32 * step)
                .clamp(request.start_seconds, request.end_seconds);
            Ok(OnionPoseV1 {
                offset_samples: offset,
                time_seconds: canonical_f32(time),
                pose: sample_animation_world_pose_v1(clip, rig, time)
                    .map_err(format_diagnostics)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let fingerprint_sha256 = fingerprint_json(&(request, &trails, &onion_poses));
    Ok(MotionVisualizationV1 {
        schema_version: 1,
        source_revision: rig.source_revision.clone(),
        clip_id: clip.id.clone(),
        clip_revision: clip.revision,
        trails,
        onion_poses,
        fingerprint_sha256,
    })
}

pub fn create_animation_variant_v1(
    source: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    recipe: &AnimationVariantRecipeV1,
) -> Result<AnimationVariantResultV1, String> {
    if recipe.schema_version != 1
        || recipe.source_revision != rig.source_revision
        || source.source.source_revision != rig.source_revision
        || recipe.source_clip_id != source.id
        || recipe.source_clip_revision != source.revision
        || recipe.source_clip_fingerprint_sha256 != fingerprint_json(source)
        || recipe.output_clip_id.trim().is_empty()
        || recipe.output_name.trim().is_empty()
        || recipe.operations.is_empty()
        || recipe.operations.len() > 16
    {
        return Err(
            "animation variant recipe is invalid or stale for the exact source clip".into(),
        );
    }
    let mut output = source.clone();
    for operation in &recipe.operations {
        match operation {
            AnimationVariantOperationV1::Speed { factor } => {
                if !factor.is_finite() || *factor <= 0.05 || *factor > 20.0 {
                    return Err("variant speed factor must be within 0.05..=20".into());
                }
                let new_length = output.length_seconds / factor;
                retime_authored_animation_clip_v1(&mut output, new_length)
                    .map_err(|diagnostic| diagnostic.message)?;
            }
            AnimationVariantOperationV1::Mirror { mapping } => {
                mirror_clip(&mut output, rig, mapping)?;
            }
            AnimationVariantOperationV1::Amplitude { factor, node_ids } => {
                scale_clip_amplitude(&mut output, rig, *factor, node_ids)?;
            }
        }
    }
    let recipe_fingerprint_sha256 = fingerprint_json(recipe);
    output.id = recipe.output_clip_id.clone();
    output.name = recipe.output_name.clone();
    output.status = AuthoredAnimationClipStatusV1::Draft;
    output.source.kind = AuthoredAnimationSourceKindV1::ProceduralTemplate;
    output.source.source_clip_name = Some(source.name.clone());
    output.source.source_clip_fingerprint = Some(recipe.source_clip_fingerprint_sha256.clone());
    output.source.procedural_template = Some(format!(
        "ANIMATION_VARIANT_RECIPE_V1:{}",
        recipe_fingerprint_sha256
    ));
    output.source.library_preset = None;
    output.source.retarget = None;
    output.revision = 1;
    let mut effective_tags = recipe
        .inherited_tags
        .iter()
        .chain(&recipe.added_tags)
        .map(|tag| tag.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    effective_tags.sort();
    effective_tags.dedup();
    if effective_tags.is_empty() || effective_tags.iter().any(|tag| tag.is_empty()) {
        return Err("variant requires at least one non-empty effective tag".into());
    }
    let fingerprint_sha256 =
        fingerprint_json(&(recipe_fingerprint_sha256.as_str(), &effective_tags, &output));
    Ok(AnimationVariantResultV1 {
        schema_version: 1,
        recipe_fingerprint_sha256,
        effective_tags,
        clip: output,
        fingerprint_sha256,
    })
}

pub fn authored_clip_fingerprint_v1(clip: &AuthoredAnimationClipV1) -> String {
    fingerprint_json(clip)
}

fn mirror_clip(
    clip: &mut AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    mapping: &HumanoidMirrorMapV1,
) -> Result<(), String> {
    if mapping.schema_version != 1
        || mapping.source_revision != rig.source_revision
        || mapping.rig_signature_sha256 != rig_signature_sha256_v1(rig)
    {
        return Err("variant mirror map is stale for the exact rig".into());
    }
    let mut remap = BTreeMap::new();
    for pair in &mapping.pairs {
        remap.insert(pair.left_node_id, pair.right_node_id);
        remap.insert(pair.right_node_id, pair.left_node_id);
    }
    for id in &mapping.center_node_ids {
        remap.insert(*id, *id);
    }
    for track in &mut clip.tracks {
        track.target_node_id = remap
            .get(&track.target_node_id)
            .copied()
            .unwrap_or(track.target_node_id);
        for key in &mut track.keyframes {
            key.value = match track.path {
                AuthoredAnimationTrackPathV1::Translation => {
                    reflect_pose_translation_v1(vec3(&key.value)?, mapping.axis).to_vec()
                }
                AuthoredAnimationTrackPathV1::Rotation => {
                    reflect_pose_rotation_v1(vec4(&key.value)?, mapping.axis).to_vec()
                }
            };
        }
    }
    clip.tracks
        .sort_by_key(|track| (track.target_node_id, track.path));
    clip.revision = clip.revision.saturating_add(1);
    Ok(())
}

fn scale_clip_amplitude(
    clip: &mut AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    factor: f32,
    node_ids: &[u32],
) -> Result<(), String> {
    if !factor.is_finite() || !(0.0..=4.0).contains(&factor) {
        return Err("variant amplitude factor must be within 0..=4".into());
    }
    let selected = node_ids.iter().copied().collect::<BTreeSet<_>>();
    if selected.len() != node_ids.len() {
        return Err("variant amplitude mask contains duplicate nodes".into());
    }
    let rest = rig
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    for track in &mut clip.tracks {
        if !selected.is_empty() && !selected.contains(&track.target_node_id) {
            continue;
        }
        let node = rest
            .get(&track.target_node_id)
            .ok_or_else(|| "variant track targets a node absent from the exact rig".to_owned())?;
        for key in &mut track.keyframes {
            key.value = match track.path {
                AuthoredAnimationTrackPathV1::Translation => {
                    let value = vec3(&key.value)?;
                    (0..3)
                        .map(|axis| {
                            node.translation[axis] + (value[axis] - node.translation[axis]) * factor
                        })
                        .collect()
                }
                AuthoredAnimationTrackPathV1::Rotation => {
                    scale_rotation_delta(node.rotation, vec4(&key.value)?, factor).to_vec()
                }
            };
        }
    }
    clip.revision = clip.revision.saturating_add(1);
    Ok(())
}

fn scale_rotation_delta(rest: [f32; 4], value: [f32; 4], factor: f32) -> [f32; 4] {
    let rest = normalize_quaternion(rest);
    let value = normalize_quaternion(value);
    let inverse = [-rest[0], -rest[1], -rest[2], rest[3]];
    let delta = multiply_quaternion(inverse, value);
    let scaled = quaternion_power(delta, factor);
    canonical_quaternion(multiply_quaternion(rest, scaled))
}

fn quaternion_power(value: [f32; 4], factor: f32) -> [f32; 4] {
    let value = canonical_quaternion(value);
    let half_angle = value[3].clamp(-1.0, 1.0).acos();
    let sine = half_angle.sin();
    if sine.abs() <= 1.0e-7 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let axis = [value[0] / sine, value[1] / sine, value[2] / sine];
    let scaled = half_angle * factor;
    canonical_quaternion([
        axis[0] * scaled.sin(),
        axis[1] * scaled.sin(),
        axis[2] * scaled.sin(),
        scaled.cos(),
    ])
}

fn validate_timeline_identity(
    clip: &AuthoredAnimationClipV1,
    timeline: &CombatAttackPhaseTimelineV1,
    require_revision: bool,
) -> Result<(), String> {
    if timeline.schema_version != 1
        || timeline.source_revision != clip.source.source_revision
        || timeline.clip_id != clip.id
        || require_revision && timeline.clip_revision != clip.revision
        || timeline.fingerprint_sha256
            != fingerprint_json(&(
                1_u32,
                timeline.source_revision.as_str(),
                timeline.clip_id.as_str(),
                timeline.clip_revision,
                &timeline.markers,
            ))
    {
        return Err("combat phase timeline is stale or has a mismatched fingerprint".into());
    }
    Ok(())
}

fn validate_phase_markers(
    clip: &AuthoredAnimationClipV1,
    markers: &[CombatAttackPhaseMarkerV1],
    minimum_phase_seconds: f32,
) -> Result<(), String> {
    if markers.len() != 5
        || markers
            .iter()
            .map(|marker| marker.phase)
            .ne(CombatAttackPhaseKindV1::ORDERED)
        || markers.iter().any(|marker| {
            !marker.time_seconds.is_finite()
                || marker.time_seconds < 0.0
                || marker.time_seconds > clip.length_seconds
        })
        || markers
            .windows(2)
            .any(|window| window[1].time_seconds - window[0].time_seconds < minimum_phase_seconds)
    {
        return Err("combat phases must be READY, WIND_UP, STRIKE, IMPACT, RECOVERY with increasing in-range times".into());
    }
    Ok(())
}

fn validate_combat_policy(
    policy: &CombatQualityPolicyV1,
    context: &CombatQualityContextV1,
) -> Result<(), String> {
    let values = [
        policy.minimum_phase_seconds,
        policy.minimum_anticipation_per_height,
        policy.maximum_impact_peak_offset_seconds,
        policy.maximum_recovery_error_per_height,
        policy.maximum_root_displacement_per_height,
        policy.maximum_grip_error_per_height,
    ];
    if policy.schema_version != 1
        || context.schema_version != 1
        || policy.sample_rate_hz == 0
        || policy.sample_rate_hz > 240
        || values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        || context
            .grip_max_error
            .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err("combat quality policy or context is invalid".into());
    }
    Ok(())
}

fn map_phase_time(
    time: f32,
    source: &[CombatAttackPhaseMarkerV1],
    target: &[CombatAttackPhaseMarkerV1],
) -> f32 {
    if time <= source[0].time_seconds {
        return canonical_f32(target[0].time_seconds);
    }
    for index in 0..source.len() - 1 {
        if time <= source[index + 1].time_seconds {
            let source_span = source[index + 1].time_seconds - source[index].time_seconds;
            let alpha = if source_span <= 1.0e-8 {
                0.0
            } else {
                (time - source[index].time_seconds) / source_span
            };
            return canonical_f32(
                target[index].time_seconds
                    + (target[index + 1].time_seconds - target[index].time_seconds) * alpha,
            );
        }
    }
    canonical_f32(target.last().unwrap().time_seconds)
}

fn skeleton_height(rig: &AnimationStudioRigV1) -> f32 {
    let minimum = rig
        .nodes
        .iter()
        .map(|node| node.translation[2])
        .fold(f32::INFINITY, f32::min);
    let maximum = rig
        .nodes
        .iter()
        .map(|node| node.translation[2])
        .fold(f32::NEG_INFINITY, f32::max);
    (maximum - minimum).abs().max(1.0)
}

#[allow(clippy::too_many_arguments)]
fn push_warning_if(
    issues: &mut Vec<CombatQualityIssueV1>,
    condition: bool,
    kind: CombatQualityIssueKindV1,
    start_seconds: f32,
    end_seconds: f32,
    measured: f32,
    threshold: f32,
    message: &str,
) {
    if condition {
        issues.push(CombatQualityIssueV1 {
            kind,
            severity: CombatQualitySeverityV1::Warning,
            start_seconds: canonical_f32(start_seconds),
            end_seconds: canonical_f32(end_seconds),
            measured: canonical_f32(measured),
            threshold: canonical_f32(threshold),
            message: message.into(),
        });
    }
}

fn bone_position(pose: &AnimationWorldPoseV1, node_id: u32) -> Result<[f32; 3], String> {
    pose.bones
        .iter()
        .find(|bone| bone.node_id == node_id)
        .map(|bone| bone.translation)
        .ok_or_else(|| format!("sampled pose misses exact node {node_id}"))
}

fn format_diagnostics(
    diagnostics: Vec<crate::animation_authoring_v2::AnimationAuthoringV2DiagnosticV1>,
) -> String {
    diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>()
        .join(" ")
}

fn vec3(value: &[f32]) -> Result<[f32; 3], String> {
    value
        .try_into()
        .map_err(|_| "translation value must contain three components".into())
}

fn vec4(value: &[f32]) -> Result<[f32; 4], String> {
    value
        .try_into()
        .map_err(|_| "rotation value must contain four components".into())
}

fn multiply_quaternion(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = left;
    let [bx, by, bz, bw] = right;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn normalize_quaternion(value: [f32; 4]) -> [f32; 4] {
    let norm = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= 1.0e-8 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        value.map(|component| component / norm)
    }
}

fn canonical_quaternion(value: [f32; 4]) -> [f32; 4] {
    let mut value = normalize_quaternion(value);
    if [value[3], value[2], value[1], value[0]]
        .into_iter()
        .find(|component| component.abs() > 1.0e-7)
        .unwrap_or(1.0)
        < 0.0
    {
        value = value.map(|component| -component);
    }
    value.map(canonical_f32)
}

fn subtract3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale3(value: [f32; 3], factor: f32) -> [f32; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn length3(value: [f32; 3]) -> f32 {
    dot3(value, value).sqrt()
}

fn distance3(left: [f32; 3], right: [f32; 3]) -> f32 {
    length3(subtract3(left, right))
}

fn canonical_f32(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        f32::from_bits(value.to_bits())
    }
}

fn fingerprint_json<T: Serialize + ?Sized>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("animation motion tools serialize");
    format!("{:x}", Sha256::digest(bytes))
}
