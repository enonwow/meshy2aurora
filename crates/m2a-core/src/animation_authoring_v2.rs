//! Canonical Animation Authoring V2 commands, pose sampling and motion-quality
//! analysis. The module is deliberately independent from the web viewport:
//! Studio may preview a gesture optimistically, but durable results are owned
//! by this Core boundary.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::animation_studio::{
    AnimationKeyframePatchV1, AnimationKeyframeV1, AnimationStudioDocumentStatusV1,
    AnimationStudioDocumentV1, AnimationStudioRigV1, AuthoredAnimationClipStatusV1,
    AuthoredAnimationClipV1, AuthoredAnimationEventPatchV1, AuthoredAnimationEventV1,
    AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1, add_authored_animation_event_v1,
    move_authored_animation_event_v1, remove_authored_animation_event_v1,
    retime_authored_animation_clip_v1, sample_animation_track_linear_v1,
    set_authored_animation_transition_v1, trim_authored_animation_clip_v1,
    update_animation_keyframe_v1, update_authored_animation_event_v1,
    validate_authored_animation_clip_v1,
};
use crate::mdl::MdlAnimationInterpolationV1;

pub const ANIMATION_AUTHORING_V2_SCHEMA_VERSION: u32 = 1;
pub const ANIMATION_EDIT_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const ANIMATION_POSE_PARITY_POLICY_SCHEMA_VERSION: u32 = 1;
pub const ANIMATION_QUALITY_POLICY_SCHEMA_VERSION: u32 = 1;
const MAX_COMMANDS_PER_BATCH: usize = 4_096;
const MAX_QUALITY_SAMPLES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationAuthoringV2DiagnosticV1 {
    pub code: String,
    pub path: String,
    pub message: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEditCommandContextV1 {
    pub source_revision: String,
    pub document_authoring_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum AnimationEditCommandV1 {
    SetTransition {
        clip_id: String,
        transition_seconds: f32,
    },
    SetBoneKey {
        clip_id: String,
        track_id: String,
        key_id: String,
        target_node_id: u32,
        path: AuthoredAnimationTrackPathV1,
        time_seconds: f32,
        value: Vec<f32>,
    },
    MoveKey {
        clip_id: String,
        track_id: String,
        key_id: String,
        time_seconds: f32,
    },
    RemoveKey {
        clip_id: String,
        track_id: String,
        key_id: String,
    },
    TrimClip {
        clip_id: String,
        start_seconds: f32,
        end_seconds: f32,
    },
    RetimeClip {
        clip_id: String,
        new_length_seconds: f32,
    },
    AddEvent {
        clip_id: String,
        event: AuthoredAnimationEventV1,
    },
    UpdateEvent {
        clip_id: String,
        event_id: String,
        time_seconds: Option<f32>,
        name: Option<String>,
    },
    MoveEvent {
        clip_id: String,
        event_id: String,
        time_seconds: f32,
    },
    RemoveEvent {
        clip_id: String,
        event_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEditCommandBatchV1 {
    pub schema_version: u32,
    pub command_id: String,
    pub context: AnimationEditCommandContextV1,
    pub commands: Vec<AnimationEditCommandV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEditCommandResultV1 {
    pub schema_version: u32,
    pub command_id: String,
    pub command_fingerprint_sha256: String,
    pub document_fingerprint_sha256: String,
    pub document: AnimationStudioDocumentV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationBoneWorldTransformV1 {
    pub node_id: u32,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationWorldPoseV1 {
    pub schema_version: u32,
    pub time_seconds: f32,
    pub bones: Vec<AnimationBoneWorldTransformV1>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoseParityPolicyV1 {
    pub schema_version: u32,
    pub sample_rate_hz: u32,
    pub translation_epsilon: f32,
    pub angular_epsilon_radians: f32,
    pub max_samples: usize,
}

impl Default for AnimationPoseParityPolicyV1 {
    fn default() -> Self {
        Self {
            schema_version: ANIMATION_POSE_PARITY_POLICY_SCHEMA_VERSION,
            sample_rate_hz: 30,
            translation_epsilon: 1.0e-4,
            angular_epsilon_radians: 1.0e-3,
            max_samples: 2_048,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoseParityReportV1 {
    pub schema_version: u32,
    pub matches: bool,
    pub sample_count: usize,
    pub max_translation_delta: f32,
    pub max_translation_bone_id: Option<u32>,
    pub max_translation_time_seconds: Option<f32>,
    pub max_angular_delta_radians: f32,
    pub max_angular_bone_id: Option<u32>,
    pub max_angular_time_seconds: Option<f32>,
    pub expected_motion_fingerprint_sha256: String,
    pub actual_motion_fingerprint_sha256: String,
    pub policy_fingerprint_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationQualityIssueKindV1 {
    StaticPlayback,
    LoopDiscontinuity,
    RootDrift,
    GroundPenetration,
    FootSliding,
    MotionSpike,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationQualitySeverityV1 {
    Info,
    Warning,
    Blocking,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationQualityIssueV1 {
    pub kind: AnimationQualityIssueKindV1,
    pub severity: AnimationQualitySeverityV1,
    pub node_id: Option<u32>,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub metric: f32,
    pub threshold: f32,
    pub message: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationQualityPolicyV1 {
    pub schema_version: u32,
    pub sample_rate_hz: u32,
    pub loop_expected: bool,
    pub static_translation_epsilon_per_height: f32,
    pub static_angular_epsilon_radians: f32,
    pub loop_translation_epsilon_per_height: f32,
    pub loop_angular_epsilon_radians: f32,
    pub root_drift_warning_per_height: f32,
    pub ground_penetration_warning_per_height: f32,
    pub contact_height_per_height: f32,
    pub foot_slide_warning_per_height: f32,
    pub max_translation_speed_per_height: f32,
    pub max_angular_speed_radians: f32,
    pub max_samples: usize,
}

impl Default for AnimationQualityPolicyV1 {
    fn default() -> Self {
        Self {
            schema_version: ANIMATION_QUALITY_POLICY_SCHEMA_VERSION,
            sample_rate_hz: 30,
            loop_expected: false,
            static_translation_epsilon_per_height: 1.0e-5,
            static_angular_epsilon_radians: 1.0e-4,
            loop_translation_epsilon_per_height: 0.01,
            loop_angular_epsilon_radians: 0.05,
            root_drift_warning_per_height: 0.1,
            ground_penetration_warning_per_height: 0.01,
            contact_height_per_height: 0.03,
            foot_slide_warning_per_height: 0.02,
            max_translation_speed_per_height: 8.0,
            max_angular_speed_radians: 20.0,
            max_samples: 2_048,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationQualityContextV1 {
    pub root_node_id: u32,
    pub contact_node_ids: Vec<u32>,
    pub ground_axis: u8,
    pub ground_height: f32,
}

impl Default for AnimationQualityContextV1 {
    fn default() -> Self {
        Self {
            root_node_id: 0,
            contact_node_ids: Vec::new(),
            ground_axis: 2,
            ground_height: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationQualityReportV1 {
    pub schema_version: u32,
    pub clip_id: String,
    pub sample_count: usize,
    pub skeleton_height: f32,
    pub distinct_pose_count: usize,
    pub issues: Vec<AnimationQualityIssueV1>,
    pub policy_fingerprint_sha256: String,
    pub report_fingerprint_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum AnimationRootMotionPolicyV1 {
    Preserve,
    InPlace,
    Scale { factor: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationLoopSeamPolicyV1 {
    pub schema_version: u32,
    pub blend_window_seconds: f32,
}

impl Default for AnimationLoopSeamPolicyV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            blend_window_seconds: 0.12,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationLoopSeamReportV1 {
    pub schema_version: u32,
    pub clip_id: String,
    pub max_position_jump: f32,
    pub max_position_bone_id: Option<u32>,
    pub max_angular_jump_radians: f32,
    pub max_angular_bone_id: Option<u32>,
    pub max_velocity_jump: f32,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationLoopBlendResultV1 {
    pub schema_version: u32,
    pub before: AnimationLoopSeamReportV1,
    pub after: AnimationLoopSeamReportV1,
    pub clip: AuthoredAnimationClipV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationRootMotionLayerV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub clip_id: String,
    pub source_clip_revision: u64,
    pub root_node_id: u32,
    pub original_track: AuthoredAnimationTrackV1,
    pub original_track_fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationRootMotionTransformResultV1 {
    pub schema_version: u32,
    pub policy: AnimationRootMotionPolicyV1,
    pub displacement_before: [f32; 3],
    pub displacement_after: [f32; 3],
    pub reversible_layer: AnimationRootMotionLayerV1,
    pub clip: AuthoredAnimationClipV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationContactIntervalV1 {
    pub node_id: u32,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub anchor_world_translation: [f32; 3],
    pub max_slide_before: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationContactBoneSetV1 {
    pub schema_version: u32,
    pub node_ids: Vec<u32>,
    pub ground_axis: u8,
    pub ground_height: f32,
    pub contact_height: f32,
    pub max_contact_speed: f32,
    pub sample_rate_hz: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationContactLockResultV1 {
    pub schema_version: u32,
    pub interval: AnimationContactIntervalV1,
    pub max_slide_before: f32,
    pub max_slide_after: f32,
    pub clip: AuthoredAnimationClipV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationKeyReductionPolicyV1 {
    pub schema_version: u32,
    pub max_position_error: f32,
    pub max_angular_error_radians: f32,
}

impl Default for AnimationKeyReductionPolicyV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            max_position_error: 1.0e-4,
            max_angular_error_radians: 1.0e-3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationKeyReductionReportV1 {
    pub schema_version: u32,
    pub clip_id: String,
    pub keys_before: usize,
    pub keys_after: usize,
    pub removed_keys: usize,
    pub max_position_error: f32,
    pub max_angular_error_radians: f32,
    pub clip: AuthoredAnimationClipV1,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum AnimationAuthoringToolRequestV1 {
    BlendLoop {
        policy: AnimationLoopSeamPolicyV1,
    },
    TransformRootMotion {
        policy: AnimationRootMotionPolicyV1,
    },
    LockSuggestedContacts {
        contact_set: AnimationContactBoneSetV1,
    },
    ReduceKeys {
        policy: AnimationKeyReductionPolicyV1,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum AnimationAuthoringToolResultV1 {
    LoopBlended {
        result: AnimationLoopBlendResultV1,
    },
    RootMotionTransformed {
        result: AnimationRootMotionTransformResultV1,
    },
    SuggestedContactsLocked {
        intervals: Vec<AnimationContactIntervalV1>,
        results: Vec<AnimationContactLockResultV1>,
        clip: AuthoredAnimationClipV1,
    },
    KeysReduced {
        report: AnimationKeyReductionReportV1,
    },
}

pub fn fingerprint_animation_edit_command_batch_v1(batch: &AnimationEditCommandBatchV1) -> String {
    fingerprint_json(batch)
}

pub fn apply_animation_edit_command_batch_v1(
    document: &AnimationStudioDocumentV1,
    rig: &AnimationStudioRigV1,
    batch: &AnimationEditCommandBatchV1,
) -> Result<AnimationEditCommandResultV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    let mut errors = Vec::new();
    if batch.schema_version != ANIMATION_EDIT_COMMAND_SCHEMA_VERSION {
        errors.push(error(
            "M2A-ANIMATION-COMMAND-SCHEMA",
            "batch.schemaVersion",
            "animation edit command schema version must be 1",
            "Recreate the command with the current Studio runtime.",
        ));
    }
    if batch.command_id.trim().is_empty() || batch.command_id.len() > 128 {
        errors.push(error(
            "M2A-ANIMATION-COMMAND-ID",
            "batch.commandId",
            "command ID must contain 1..=128 bytes",
            "Generate one stable operation identity.",
        ));
    }
    if batch.commands.is_empty() || batch.commands.len() > MAX_COMMANDS_PER_BATCH {
        errors.push(error(
            "M2A-ANIMATION-COMMAND-LIMIT",
            "batch.commands",
            "command batch must contain 1..=4096 operations",
            "Split the edit into bounded user gestures.",
        ));
    }
    if batch.context.source_revision != document.source_revision
        || batch.context.source_revision != rig.source_revision
    {
        errors.push(error(
            "M2A-ANIMATION-COMMAND-SOURCE-STALE",
            "batch.context.sourceRevision",
            "command, document and rig source revisions differ",
            "Discard the optimistic edit and re-inspect the current source.",
        ));
    }
    if batch.context.document_authoring_revision != document.authoring_revision {
        errors.push(error(
            "M2A-ANIMATION-COMMAND-REVISION-STALE",
            "batch.context.documentAuthoringRevision",
            "command targets a stale Animation Studio document revision",
            "Discard the stale result and replay the gesture on the current document.",
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut candidate = document.clone();
    for (index, command) in batch.commands.iter().enumerate() {
        if let Err(diagnostic) = apply_command(&mut candidate, command) {
            return Err(vec![AnimationAuthoringV2DiagnosticV1 {
                path: format!("batch.commands[{index}].{}", diagnostic.path),
                ..diagnostic
            }]);
        }
    }
    candidate.authoring_revision = document.authoring_revision.saturating_add(1);
    candidate.status = AnimationStudioDocumentStatusV1::Draft;

    let mut diagnostics = Vec::new();
    for clip in &candidate.authored_clips {
        diagnostics.extend(
            validate_authored_animation_clip_v1(clip, rig)
                .into_iter()
                .map(from_studio_diagnostic),
        );
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(AnimationEditCommandResultV1 {
        schema_version: ANIMATION_AUTHORING_V2_SCHEMA_VERSION,
        command_id: batch.command_id.clone(),
        command_fingerprint_sha256: fingerprint_animation_edit_command_batch_v1(batch),
        document_fingerprint_sha256: fingerprint_json(&candidate),
        document: candidate,
    })
}

pub fn sample_animation_world_pose_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    time_seconds: f32,
) -> Result<AnimationWorldPoseV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if !time_seconds.is_finite() || time_seconds < 0.0 || time_seconds > clip.length_seconds {
        return Err(vec![error(
            "M2A-ANIMATION-POSE-TIME",
            "timeSeconds",
            "pose sample time must be finite and inside the clip",
            "Clamp the sample to the exact clip duration.",
        )]);
    }
    let mut local = BTreeMap::new();
    for node in &rig.nodes {
        local.insert(
            node.node_id,
            (node.translation, canonical_quaternion(node.rotation)?),
        );
    }
    for track in &clip.tracks {
        let value = sample_animation_track_linear_v1(track, time_seconds)
            .map_err(|diagnostic| vec![from_studio_diagnostic(diagnostic)])?;
        let Some(transform) = local.get_mut(&track.target_node_id) else {
            return Err(vec![error(
                "M2A-ANIMATION-POSE-BONE",
                format!("tracks[{}].targetNodeId", track.id),
                "animation track targets a bone absent from the rig",
                "Reconcile the clip with the exact output rig.",
            )]);
        };
        match track.path {
            AuthoredAnimationTrackPathV1::Translation => {
                transform.0 = vec3(&value).ok_or_else(|| {
                    vec![error(
                        "M2A-ANIMATION-POSE-VALUE",
                        format!("tracks[{}].value", track.id),
                        "translation sample must contain three finite values",
                        "Repair the authored translation track.",
                    )]
                })?;
            }
            AuthoredAnimationTrackPathV1::Rotation => {
                transform.1 = quaternion(&value).ok_or_else(|| {
                    vec![error(
                        "M2A-ANIMATION-POSE-VALUE",
                        format!("tracks[{}].value", track.id),
                        "rotation sample must contain a finite unit quaternion",
                        "Normalize or repair the authored rotation track.",
                    )]
                })?;
            }
        }
    }

    let nodes = rig
        .nodes
        .iter()
        .map(|node| (node.node_id, node.parent_id))
        .collect::<BTreeMap<_, _>>();
    let mut world = BTreeMap::<u32, ([f32; 3], [f32; 4])>::new();
    let mut visiting = BTreeSet::new();
    for node in &rig.nodes {
        resolve_world_transform(node.node_id, &nodes, &local, &mut world, &mut visiting)?;
    }
    let bones = rig
        .nodes
        .iter()
        .map(|node| {
            let transform = world[&node.node_id];
            AnimationBoneWorldTransformV1 {
                node_id: node.node_id,
                translation: canonical_vec3(transform.0),
                rotation: canonical_quaternion(transform.1).expect("resolved unit quaternion"),
            }
        })
        .collect::<Vec<_>>();
    Ok(AnimationWorldPoseV1 {
        schema_version: ANIMATION_AUTHORING_V2_SCHEMA_VERSION,
        time_seconds: canonical_f32(time_seconds),
        fingerprint_sha256: fingerprint_json(&bones),
        bones,
    })
}

pub fn evaluate_animation_pose_parity_v1(
    expected: &AuthoredAnimationClipV1,
    actual: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    policy: &AnimationPoseParityPolicyV1,
) -> Result<AnimationPoseParityReportV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    validate_pose_policy(policy)?;
    if (expected.length_seconds - actual.length_seconds).abs() > f32::EPSILON {
        return Err(vec![error(
            "M2A-ANIMATION-POSE-DURATION",
            "actual.lengthSeconds",
            "pose parity requires clips with the same duration",
            "Compare the writer/package clip bound to the same authored output.",
        )]);
    }
    let times = parity_sample_times(expected, actual, policy);
    let mut max_translation = (0.0_f32, None, None);
    let mut max_angular = (0.0_f32, None, None);
    let mut expected_fingerprints = Vec::with_capacity(times.len());
    let mut actual_fingerprints = Vec::with_capacity(times.len());
    for time in &times {
        let expected_pose = sample_animation_world_pose_v1(expected, rig, *time)?;
        let actual_pose = sample_animation_world_pose_v1(actual, rig, *time)?;
        expected_fingerprints.push(expected_pose.fingerprint_sha256.clone());
        actual_fingerprints.push(actual_pose.fingerprint_sha256.clone());
        let actual_by_id = actual_pose
            .bones
            .iter()
            .map(|bone| (bone.node_id, bone))
            .collect::<BTreeMap<_, _>>();
        for expected_bone in &expected_pose.bones {
            let Some(actual_bone) = actual_by_id.get(&expected_bone.node_id) else {
                return Err(vec![error(
                    "M2A-ANIMATION-POSE-BONE",
                    format!("actual.bones[{}]", expected_bone.node_id),
                    "actual pose is missing an expected bone",
                    "Rebuild from the exact same output rig.",
                )]);
            };
            let translation = distance3(expected_bone.translation, actual_bone.translation);
            if translation > max_translation.0 {
                max_translation = (translation, Some(expected_bone.node_id), Some(*time));
            }
            let angular = quaternion_angle(expected_bone.rotation, actual_bone.rotation);
            if angular > max_angular.0 {
                max_angular = (angular, Some(expected_bone.node_id), Some(*time));
            }
        }
    }
    Ok(AnimationPoseParityReportV1 {
        schema_version: ANIMATION_AUTHORING_V2_SCHEMA_VERSION,
        matches: max_translation.0 <= policy.translation_epsilon
            && max_angular.0 <= policy.angular_epsilon_radians,
        sample_count: times.len(),
        max_translation_delta: canonical_f32(max_translation.0),
        max_translation_bone_id: max_translation.1,
        max_translation_time_seconds: max_translation.2.map(canonical_f32),
        max_angular_delta_radians: canonical_f32(max_angular.0),
        max_angular_bone_id: max_angular.1,
        max_angular_time_seconds: max_angular.2.map(canonical_f32),
        expected_motion_fingerprint_sha256: fingerprint_json(&expected_fingerprints),
        actual_motion_fingerprint_sha256: fingerprint_json(&actual_fingerprints),
        policy_fingerprint_sha256: fingerprint_json(policy),
    })
}

pub fn analyze_animation_motion_quality_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    policy: &AnimationQualityPolicyV1,
    context: &AnimationQualityContextV1,
) -> Result<AnimationQualityReportV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    validate_quality_policy(policy, context)?;
    let sample_times = uniform_sample_times(
        clip.length_seconds,
        policy.sample_rate_hz,
        policy.max_samples.min(MAX_QUALITY_SAMPLES),
    );
    let poses = sample_times
        .iter()
        .map(|time| sample_animation_world_pose_v1(clip, rig, *time))
        .collect::<Result<Vec<_>, _>>()?;
    let height = skeleton_height(&poses[0]).max(1.0e-6);
    let mut issues = Vec::new();
    let distinct = poses
        .iter()
        .map(|pose| pose.fingerprint_sha256.as_str())
        .collect::<BTreeSet<_>>()
        .len();

    let (max_static_translation, max_static_angular) = pose_delta(&poses[0], poses.last().unwrap());
    let any_motion = poses.iter().skip(1).any(|pose| {
        let (translation, angular) = pose_delta(&poses[0], pose);
        translation / height > policy.static_translation_epsilon_per_height
            || angular > policy.static_angular_epsilon_radians
    });
    if !any_motion {
        issues.push(issue(
            AnimationQualityIssueKindV1::StaticPlayback,
            AnimationQualitySeverityV1::Blocking,
            None,
            0.0,
            clip.length_seconds,
            (max_static_translation / height).max(max_static_angular),
            policy.static_translation_epsilon_per_height,
            "MOTION clip has no measurable pose change",
            "Add changing keys or classify the clip as STATIC_POSE.",
        ));
    }

    if policy.loop_expected {
        let (translation, angular) = pose_delta(&poses[0], poses.last().unwrap());
        let normalized = translation / height;
        if normalized > policy.loop_translation_epsilon_per_height
            || angular > policy.loop_angular_epsilon_radians
        {
            issues.push(issue(
                AnimationQualityIssueKindV1::LoopDiscontinuity,
                AnimationQualitySeverityV1::Warning,
                None,
                0.0,
                clip.length_seconds,
                normalized.max(angular),
                policy.loop_translation_epsilon_per_height,
                "first and last loop poses are discontinuous",
                "Inspect the loop seam or blend a bounded start/end window.",
            ));
        }
    }

    if let (Some(first), Some(last)) = (
        find_bone(&poses[0], context.root_node_id),
        find_bone(poses.last().unwrap(), context.root_node_id),
    ) {
        let drift = distance3(first.translation, last.translation) / height;
        if drift > policy.root_drift_warning_per_height {
            issues.push(issue(
                AnimationQualityIssueKindV1::RootDrift,
                AnimationQualitySeverityV1::Warning,
                Some(context.root_node_id),
                0.0,
                clip.length_seconds,
                drift,
                policy.root_drift_warning_per_height,
                "root displacement exceeds the configured quality threshold",
                "Choose Preserve, InPlace or Scale root-motion policy explicitly.",
            ));
        }
    }

    let axis = usize::from(context.ground_axis);
    for node_id in &context.contact_node_ids {
        let mut maximum_slide = 0.0_f32;
        let mut maximum_penetration = 0.0_f32;
        for (sample_index, pair) in poses.windows(2).enumerate() {
            let Some(left) = find_bone(&pair[0], *node_id) else {
                continue;
            };
            let Some(right) = find_bone(&pair[1], *node_id) else {
                continue;
            };
            let penetration = (context.ground_height - right.translation[axis]) / height;
            maximum_penetration = maximum_penetration.max(penetration);
            let contact_limit = context.ground_height + policy.contact_height_per_height * height;
            if left.translation[axis] <= contact_limit && right.translation[axis] <= contact_limit {
                let slide = horizontal_distance(left.translation, right.translation, axis) / height;
                maximum_slide = maximum_slide.max(slide);
                if slide > policy.foot_slide_warning_per_height {
                    issues.push(issue(
                        AnimationQualityIssueKindV1::FootSliding,
                        AnimationQualitySeverityV1::Warning,
                        Some(*node_id),
                        sample_times[sample_index],
                        sample_times[sample_index + 1],
                        slide,
                        policy.foot_slide_warning_per_height,
                        "contact bone moves horizontally while touching the ground plane",
                        "Inspect the contact interval or apply a bounded contact lock.",
                    ));
                    break;
                }
            }
        }
        if maximum_penetration > policy.ground_penetration_warning_per_height {
            issues.push(issue(
                AnimationQualityIssueKindV1::GroundPenetration,
                AnimationQualitySeverityV1::Warning,
                Some(*node_id),
                0.0,
                clip.length_seconds,
                maximum_penetration,
                policy.ground_penetration_warning_per_height,
                "contact bone penetrates the configured ground plane",
                "Adjust the pose, root height or the explicit ground plane.",
            ));
        }
        let _ = maximum_slide;
    }

    'samples: for (sample_index, pair) in poses.windows(2).enumerate() {
        let dt = (sample_times[sample_index + 1] - sample_times[sample_index]).max(1.0e-6);
        let right_by_id = pair[1]
            .bones
            .iter()
            .map(|bone| (bone.node_id, bone))
            .collect::<BTreeMap<_, _>>();
        for left in &pair[0].bones {
            let Some(right) = right_by_id.get(&left.node_id) else {
                continue;
            };
            let translation_speed = distance3(left.translation, right.translation) / height / dt;
            let angular_speed = quaternion_angle(left.rotation, right.rotation) / dt;
            if translation_speed > policy.max_translation_speed_per_height
                || angular_speed > policy.max_angular_speed_radians
            {
                issues.push(issue(
                    AnimationQualityIssueKindV1::MotionSpike,
                    AnimationQualitySeverityV1::Warning,
                    Some(left.node_id),
                    sample_times[sample_index],
                    sample_times[sample_index + 1],
                    translation_speed.max(angular_speed),
                    policy.max_translation_speed_per_height,
                    "bone velocity exceeds the configured motion-spike threshold",
                    "Inspect nearby key spacing and values.",
                ));
                break 'samples;
            }
        }
    }

    issues.sort_by(|left, right| {
        left.start_seconds
            .total_cmp(&right.start_seconds)
            .then(left.kind.cmp(&right.kind))
            .then(left.node_id.cmp(&right.node_id))
    });
    let policy_hash = fingerprint_json(&(policy, context));
    let report_hash = fingerprint_json(&(
        clip.id.as_str(),
        sample_times.len(),
        canonical_f32(height),
        distinct,
        &issues,
        policy_hash.as_str(),
    ));
    Ok(AnimationQualityReportV1 {
        schema_version: ANIMATION_AUTHORING_V2_SCHEMA_VERSION,
        clip_id: clip.id.clone(),
        sample_count: sample_times.len(),
        skeleton_height: canonical_f32(height),
        distinct_pose_count: distinct,
        issues,
        policy_fingerprint_sha256: policy_hash,
        report_fingerprint_sha256: report_hash,
    })
}

pub fn apply_animation_root_motion_policy_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    policy: AnimationRootMotionPolicyV1,
) -> Result<AuthoredAnimationClipV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if policy == AnimationRootMotionPolicyV1::Preserve {
        return Ok(clip.clone());
    }
    let root = rig
        .nodes
        .iter()
        .find(|node| node.name == rig.animation_root)
        .ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-ROOT-MISSING",
                "rig.animationRoot",
                "animation root is absent from the exact rig",
                "Re-inspect the source rig before applying root-motion policy.",
            )]
        })?;
    let mut output = clip.clone();
    let Some(track) = output.tracks.iter_mut().find(|track| {
        track.target_node_id == root.node_id
            && track.path == AuthoredAnimationTrackPathV1::Translation
    }) else {
        return Err(vec![error(
            "M2A-ANIMATION-ROOT-TRACK-MISSING",
            "clip.tracks",
            "root-motion policy requires a root translation track",
            "Add or select a clip with root translation keys.",
        )]);
    };
    let first = track
        .keyframes
        .first()
        .and_then(|key| vec3(&key.value))
        .ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-ROOT-TRACK",
                "clip.tracks.root.keyframes",
                "root translation track has no valid first key",
                "Repair the root translation track.",
            )]
        })?;
    let factor = match policy {
        AnimationRootMotionPolicyV1::Preserve => unreachable!(),
        AnimationRootMotionPolicyV1::InPlace => 0.0,
        AnimationRootMotionPolicyV1::Scale { factor } if factor.is_finite() && factor >= 0.0 => {
            factor
        }
        AnimationRootMotionPolicyV1::Scale { .. } => {
            return Err(vec![error(
                "M2A-ANIMATION-ROOT-SCALE",
                "policy.factor",
                "root-motion scale must be finite and non-negative",
                "Choose a finite scale factor greater than or equal to zero.",
            )]);
        }
    };
    for key in &mut track.keyframes {
        let value = vec3(&key.value).ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-ROOT-TRACK",
                format!("clip.tracks.root.keyframes[{}]", key.id),
                "root translation key is malformed",
                "Repair the root translation track.",
            )]
        })?;
        key.value = vec![
            canonical_f32(first[0] + (value[0] - first[0]) * factor),
            canonical_f32(first[1] + (value[1] - first[1]) * factor),
            canonical_f32(first[2] + (value[2] - first[2]) * factor),
        ];
    }
    output.status = AuthoredAnimationClipStatusV1::Draft;
    output.revision = output.revision.saturating_add(1);
    Ok(output)
}

pub fn inspect_loop_seam_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
) -> Result<AnimationLoopSeamReportV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if !clip.length_seconds.is_finite() || clip.length_seconds <= 0.0 {
        return Err(vec![error(
            "M2A-ANIMATION-LOOP-DURATION",
            "clip.lengthSeconds",
            "loop inspection requires a positive finite clip duration",
            "Repair the clip duration before inspecting its loop seam.",
        )]);
    }
    let dt = (1.0 / 30.0_f32).min(clip.length_seconds * 0.25);
    let start = sample_animation_world_pose_v1(clip, rig, 0.0)?;
    let start_next = sample_animation_world_pose_v1(clip, rig, dt)?;
    let end_previous = sample_animation_world_pose_v1(clip, rig, clip.length_seconds - dt)?;
    let end = sample_animation_world_pose_v1(clip, rig, clip.length_seconds)?;
    let by_id = |pose: &AnimationWorldPoseV1| {
        pose.bones
            .iter()
            .map(|bone| (bone.node_id, *bone))
            .collect::<BTreeMap<_, _>>()
    };
    let start_by_id = by_id(&start);
    let start_next_by_id = by_id(&start_next);
    let end_previous_by_id = by_id(&end_previous);
    let end_by_id = by_id(&end);
    let mut max_position_jump = 0.0_f32;
    let mut max_position_bone_id = None;
    let mut max_angular_jump_radians = 0.0_f32;
    let mut max_angular_bone_id = None;
    let mut max_velocity_jump = 0.0_f32;
    for node in &rig.nodes {
        let Some((first, first_next, last_previous, last)) = start_by_id
            .get(&node.node_id)
            .zip(start_next_by_id.get(&node.node_id))
            .zip(end_previous_by_id.get(&node.node_id))
            .zip(end_by_id.get(&node.node_id))
            .map(|(((first, first_next), last_previous), last)| {
                (first, first_next, last_previous, last)
            })
        else {
            continue;
        };
        let position_jump = distance3(first.translation, last.translation);
        if position_jump > max_position_jump {
            max_position_jump = position_jump;
            max_position_bone_id = Some(node.node_id);
        }
        let angular_jump = quaternion_angle(first.rotation, last.rotation);
        if angular_jump > max_angular_jump_radians {
            max_angular_jump_radians = angular_jump;
            max_angular_bone_id = Some(node.node_id);
        }
        let start_velocity = scale3(
            subtract3(first_next.translation, first.translation),
            1.0 / dt,
        );
        let end_velocity = scale3(
            subtract3(last.translation, last_previous.translation),
            1.0 / dt,
        );
        max_velocity_jump = max_velocity_jump.max(distance3(start_velocity, end_velocity));
    }
    let fingerprint_sha256 = fingerprint_json(&(
        clip.id.as_str(),
        canonical_f32(max_position_jump),
        max_position_bone_id,
        canonical_f32(max_angular_jump_radians),
        max_angular_bone_id,
        canonical_f32(max_velocity_jump),
    ));
    Ok(AnimationLoopSeamReportV1 {
        schema_version: 1,
        clip_id: clip.id.clone(),
        max_position_jump: canonical_f32(max_position_jump),
        max_position_bone_id,
        max_angular_jump_radians: canonical_f32(max_angular_jump_radians),
        max_angular_bone_id,
        max_velocity_jump: canonical_f32(max_velocity_jump),
        fingerprint_sha256,
    })
}

pub fn blend_animation_loop_seam_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    policy: &AnimationLoopSeamPolicyV1,
) -> Result<AnimationLoopBlendResultV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if policy.schema_version != 1
        || !policy.blend_window_seconds.is_finite()
        || policy.blend_window_seconds <= 0.0
        || policy.blend_window_seconds > clip.length_seconds * 0.5
    {
        return Err(vec![error(
            "M2A-ANIMATION-LOOP-WINDOW",
            "policy.blendWindowSeconds",
            "loop blend window must be finite, positive and no longer than half the clip",
            "Choose a smaller blend window.",
        )]);
    }
    let before = inspect_loop_seam_v1(clip, rig)?;
    let original_events = clip.events.clone();
    let mut output = clip.clone();
    for track in &mut output.tracks {
        if track.keyframes.is_empty() {
            continue;
        }
        let start_value = sample_animation_track_linear_v1(track, 0.0)
            .map_err(|diagnostic| vec![from_studio_diagnostic(diagnostic)])?;
        let end_value = sample_animation_track_linear_v1(track, clip.length_seconds)
            .map_err(|diagnostic| vec![from_studio_diagnostic(diagnostic)])?;
        let seam_value = blend_track_value(track.path, &start_value, &end_value, 0.5)?;
        if !track
            .keyframes
            .iter()
            .any(|key| key.time_seconds.abs() <= 1.0e-7)
        {
            track.keyframes.push(AnimationKeyframeV1 {
                id: format!("{}-loop-start", track.id),
                time_seconds: 0.0,
                value: start_value,
            });
        }
        if !track
            .keyframes
            .iter()
            .any(|key| (key.time_seconds - clip.length_seconds).abs() <= 1.0e-7)
        {
            track.keyframes.push(AnimationKeyframeV1 {
                id: format!("{}-loop-end", track.id),
                time_seconds: clip.length_seconds,
                value: end_value,
            });
        }
        track.keyframes.sort_by(|left, right| {
            left.time_seconds
                .total_cmp(&right.time_seconds)
                .then(left.id.cmp(&right.id))
        });
        for key in &mut track.keyframes {
            let distance_to_seam = key.time_seconds.min(clip.length_seconds - key.time_seconds);
            if distance_to_seam > policy.blend_window_seconds {
                continue;
            }
            let weight = 1.0 - distance_to_seam / policy.blend_window_seconds;
            key.value = blend_track_value(track.path, &key.value, &seam_value, weight)?;
        }
    }
    debug_assert_eq!(output.events, original_events);
    output.status = AuthoredAnimationClipStatusV1::Draft;
    output.revision = output.revision.saturating_add(1);
    let diagnostics = validate_authored_animation_clip_v1(&output, rig);
    if !diagnostics.is_empty() {
        return Err(diagnostics
            .into_iter()
            .map(from_studio_diagnostic)
            .collect());
    }
    let after = inspect_loop_seam_v1(&output, rig)?;
    Ok(AnimationLoopBlendResultV1 {
        schema_version: 1,
        before,
        after,
        clip: output,
    })
}

pub fn extract_root_motion_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
) -> Result<AnimationRootMotionLayerV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    let root = resolve_animation_root_node_v1(rig)?;
    let original_track = clip
        .tracks
        .iter()
        .find(|track| {
            track.target_node_id == root.node_id
                && track.path == AuthoredAnimationTrackPathV1::Translation
        })
        .cloned()
        .ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-ROOT-TRACK-MISSING",
                "clip.tracks",
                "root-motion extraction requires a root translation track",
                "Choose a clip with root translation or author the root track first.",
            )]
        })?;
    Ok(AnimationRootMotionLayerV1 {
        schema_version: 1,
        source_revision: rig.source_revision.clone(),
        clip_id: clip.id.clone(),
        source_clip_revision: clip.revision,
        root_node_id: root.node_id,
        original_track_fingerprint_sha256: fingerprint_json(&original_track),
        original_track,
    })
}

pub fn transform_root_motion_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    policy: AnimationRootMotionPolicyV1,
) -> Result<AnimationRootMotionTransformResultV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    let reversible_layer = extract_root_motion_v1(clip, rig)?;
    let displacement_before = root_track_displacement_v1(&reversible_layer.original_track)?;
    let output = apply_animation_root_motion_policy_v1(clip, rig, policy)?;
    let transformed_track = output
        .tracks
        .iter()
        .find(|track| track.id == reversible_layer.original_track.id)
        .ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-ROOT-TRACK-MISSING",
                "result.tracks",
                "root-motion transform lost the reversible root track",
                "Discard this transform and keep the source clip.",
            )]
        })?;
    let displacement_after = root_track_displacement_v1(transformed_track)?;
    Ok(AnimationRootMotionTransformResultV1 {
        schema_version: 1,
        policy,
        displacement_before,
        displacement_after,
        reversible_layer,
        clip: output,
    })
}

pub fn restore_root_motion_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    layer: &AnimationRootMotionLayerV1,
) -> Result<AuthoredAnimationClipV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if layer.schema_version != 1
        || layer.source_revision != rig.source_revision
        || layer.clip_id != clip.id
        || fingerprint_json(&layer.original_track) != layer.original_track_fingerprint_sha256
    {
        return Err(vec![error(
            "M2A-ANIMATION-ROOT-LAYER-STALE",
            "rootMotionLayer",
            "root-motion layer does not match this exact source, clip or fingerprint",
            "Use the reversible layer created for this clip lineage.",
        )]);
    }
    let mut output = clip.clone();
    let current = output.tracks.iter_mut().find(|track| {
        track.target_node_id == layer.root_node_id
            && track.path == AuthoredAnimationTrackPathV1::Translation
    });
    match current {
        Some(track) => *track = layer.original_track.clone(),
        None => output.tracks.push(layer.original_track.clone()),
    }
    output.status = AuthoredAnimationClipStatusV1::Draft;
    output.revision = output.revision.saturating_add(1);
    Ok(output)
}

pub fn reduce_animation_keyframes_v1(
    clip: &AuthoredAnimationClipV1,
    policy: &AnimationKeyReductionPolicyV1,
) -> Result<AnimationKeyReductionReportV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if policy.schema_version != 1
        || !policy.max_position_error.is_finite()
        || policy.max_position_error < 0.0
        || !policy.max_angular_error_radians.is_finite()
        || policy.max_angular_error_radians < 0.0
    {
        return Err(vec![error(
            "M2A-ANIMATION-REDUCTION-POLICY",
            "policy",
            "key reduction tolerances must be finite and non-negative",
            "Choose explicit finite error tolerances.",
        )]);
    }
    let mut output = clip.clone();
    let keys_before = output
        .tracks
        .iter()
        .map(|track| track.keyframes.len())
        .sum();
    let mut max_position_error = 0.0_f32;
    let mut max_angular_error_radians = 0.0_f32;
    for track in &mut output.tracks {
        if track.keyframes.len() <= 2 {
            continue;
        }
        let original = track.keyframes.clone();
        let mut keep = vec![false; original.len()];
        keep[0] = true;
        keep[original.len() - 1] = true;
        reduce_key_segment_v1(
            track.path,
            &original,
            0,
            original.len() - 1,
            policy,
            &mut keep,
            &mut max_position_error,
            &mut max_angular_error_radians,
        )?;
        track.keyframes = original
            .into_iter()
            .zip(keep)
            .filter_map(|(key, keep)| keep.then_some(key))
            .collect();
    }
    let keys_after = output
        .tracks
        .iter()
        .map(|track| track.keyframes.len())
        .sum();
    if keys_after != keys_before {
        output.status = AuthoredAnimationClipStatusV1::Draft;
        output.revision = output.revision.saturating_add(1);
    }
    let fingerprint_sha256 = fingerprint_json(&(
        clip.id.as_str(),
        keys_before,
        keys_after,
        canonical_f32(max_position_error),
        canonical_f32(max_angular_error_radians),
        &output,
    ));
    Ok(AnimationKeyReductionReportV1 {
        schema_version: 1,
        clip_id: clip.id.clone(),
        keys_before,
        keys_after,
        removed_keys: keys_before - keys_after,
        max_position_error: canonical_f32(max_position_error),
        max_angular_error_radians: canonical_f32(max_angular_error_radians),
        clip: output,
        fingerprint_sha256,
    })
}

pub fn suggest_contact_intervals_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    contact_set: &AnimationContactBoneSetV1,
) -> Result<Vec<AnimationContactIntervalV1>, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if contact_set.schema_version != 1
        || contact_set.ground_axis > 2
        || contact_set.sample_rate_hz == 0
        || contact_set.sample_rate_hz > 240
        || !contact_set.ground_height.is_finite()
        || !contact_set.contact_height.is_finite()
        || contact_set.contact_height < 0.0
        || !contact_set.max_contact_speed.is_finite()
        || contact_set.max_contact_speed < 0.0
    {
        return Err(vec![error(
            "M2A-ANIMATION-CONTACT-POLICY",
            "contactSet",
            "contact policy has an invalid axis, sample rate, ground or threshold",
            "Choose axis 0..2 and finite non-negative thresholds.",
        )]);
    }
    let known = rig
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<BTreeSet<_>>();
    if contact_set.node_ids.iter().any(|id| !known.contains(id)) {
        return Err(vec![error(
            "M2A-ANIMATION-CONTACT-BONE",
            "contactSet.nodeIds",
            "contact set contains a bone absent from the exact rig",
            "Choose contact bones from the current output rig.",
        )]);
    }
    let times = uniform_sample_times(
        clip.length_seconds,
        contact_set.sample_rate_hz,
        MAX_QUALITY_SAMPLES,
    );
    let poses = times
        .iter()
        .map(|time| sample_animation_world_pose_v1(clip, rig, *time))
        .collect::<Result<Vec<_>, _>>()?;
    let mut intervals = Vec::new();
    for node_id in &contact_set.node_ids {
        let positions = poses
            .iter()
            .map(|pose| {
                pose.bones
                    .iter()
                    .find(|bone| bone.node_id == *node_id)
                    .map(|bone| bone.translation)
                    .ok_or_else(|| {
                        vec![error(
                            "M2A-ANIMATION-CONTACT-BONE",
                            "pose.bones",
                            "contact bone disappeared from a sampled pose",
                            "Re-inspect the exact rig.",
                        )]
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut active_start = None;
        for index in 0..positions.len() {
            let height =
                positions[index][contact_set.ground_axis as usize] - contact_set.ground_height;
            let speed = if index == 0 {
                0.0
            } else {
                horizontal_distance3(
                    positions[index - 1],
                    positions[index],
                    contact_set.ground_axis,
                ) / (times[index] - times[index - 1]).max(1.0e-6)
            };
            let active =
                height <= contact_set.contact_height && speed <= contact_set.max_contact_speed;
            match (active_start, active) {
                (None, true) => active_start = Some(index),
                (Some(start), false) => {
                    append_contact_interval_v1(
                        &mut intervals,
                        *node_id,
                        start,
                        index.saturating_sub(1),
                        &times,
                        &positions,
                        contact_set.ground_axis,
                    );
                    active_start = None;
                }
                _ => {}
            }
        }
        if let Some(start) = active_start {
            append_contact_interval_v1(
                &mut intervals,
                *node_id,
                start,
                positions.len() - 1,
                &times,
                &positions,
                contact_set.ground_axis,
            );
        }
    }
    intervals.sort_by(|left, right| {
        left.start_seconds
            .total_cmp(&right.start_seconds)
            .then(left.node_id.cmp(&right.node_id))
    });
    Ok(intervals)
}

pub fn lock_contact_bone_interval_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    interval: &AnimationContactIntervalV1,
    sample_rate_hz: u32,
    ground_axis: u8,
) -> Result<AnimationContactLockResultV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    if ground_axis > 2
        || sample_rate_hz == 0
        || sample_rate_hz > 240
        || !interval.start_seconds.is_finite()
        || !interval.end_seconds.is_finite()
        || interval.start_seconds < 0.0
        || interval.end_seconds <= interval.start_seconds
        || interval.end_seconds > clip.length_seconds
    {
        return Err(vec![error(
            "M2A-ANIMATION-CONTACT-INTERVAL",
            "interval",
            "contact interval must be finite, ordered and inside the clip",
            "Choose a valid interval from the current clip.",
        )]);
    }
    let node = rig
        .nodes
        .iter()
        .find(|node| node.node_id == interval.node_id)
        .ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-CONTACT-BONE",
                "interval.nodeId",
                "contact interval targets a bone absent from the exact rig",
                "Choose a current rig bone.",
            )]
        })?;
    let mut times = uniform_sample_times(
        interval.end_seconds - interval.start_seconds,
        sample_rate_hz,
        MAX_QUALITY_SAMPLES,
    )
    .into_iter()
    .map(|time| canonical_f32(time + interval.start_seconds))
    .collect::<Vec<_>>();
    if let Some(track) = clip.tracks.iter().find(|track| {
        track.target_node_id == interval.node_id
            && track.path == AuthoredAnimationTrackPathV1::Translation
    }) {
        times.extend(
            track
                .keyframes
                .iter()
                .filter(|key| {
                    key.time_seconds >= interval.start_seconds
                        && key.time_seconds <= interval.end_seconds
                })
                .map(|key| key.time_seconds),
        );
    }
    times.sort_by(f32::total_cmp);
    times.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-6);
    let before_positions = sample_node_positions_v1(clip, rig, interval.node_id, &times)?;
    let max_slide_before = max_horizontal_slide_v1(
        &before_positions,
        interval.anchor_world_translation,
        ground_axis,
    );
    let mut output = clip.clone();
    let track_index = output.tracks.iter().position(|track| {
        track.target_node_id == interval.node_id
            && track.path == AuthoredAnimationTrackPathV1::Translation
    });
    if track_index.is_none() {
        output.tracks.push(AuthoredAnimationTrackV1 {
            id: format!("contact-lock-{}", interval.node_id),
            target_node_id: interval.node_id,
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: Vec::new(),
        });
    }
    let track = output
        .tracks
        .iter_mut()
        .find(|track| {
            track.target_node_id == interval.node_id
                && track.path == AuthoredAnimationTrackPathV1::Translation
        })
        .expect("contact translation track exists");
    track.keyframes.retain(|key| {
        key.time_seconds < interval.start_seconds || key.time_seconds > interval.end_seconds
    });
    for (index, time) in times.iter().enumerate() {
        let pose = sample_animation_world_pose_v1(clip, rig, *time)?;
        let local_translation = if let Some(parent_id) = node.parent_id {
            let parent = pose
                .bones
                .iter()
                .find(|bone| bone.node_id == parent_id)
                .ok_or_else(|| {
                    vec![error(
                        "M2A-ANIMATION-CONTACT-PARENT",
                        "rig.nodes.parentId",
                        "contact bone parent is absent from the sampled pose",
                        "Repair the exact rig hierarchy.",
                    )]
                })?;
            rotate3(
                conjugate_quaternion(parent.rotation),
                subtract3(interval.anchor_world_translation, parent.translation),
            )
        } else {
            interval.anchor_world_translation
        };
        track.keyframes.push(AnimationKeyframeV1 {
            id: format!("contact-{}-{index:04}", interval.node_id),
            time_seconds: *time,
            value: local_translation.into_iter().map(canonical_f32).collect(),
        });
    }
    track.keyframes.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then(left.id.cmp(&right.id))
    });
    output.status = AuthoredAnimationClipStatusV1::Draft;
    output.revision = output.revision.saturating_add(1);
    let after_positions = sample_node_positions_v1(&output, rig, interval.node_id, &times)?;
    let max_slide_after = max_horizontal_slide_v1(
        &after_positions,
        interval.anchor_world_translation,
        ground_axis,
    );
    Ok(AnimationContactLockResultV1 {
        schema_version: 1,
        interval: interval.clone(),
        max_slide_before: canonical_f32(max_slide_before),
        max_slide_after: canonical_f32(max_slide_after),
        clip: output,
    })
}

pub fn apply_animation_authoring_tool_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    request: &AnimationAuthoringToolRequestV1,
) -> Result<AnimationAuthoringToolResultV1, Vec<AnimationAuthoringV2DiagnosticV1>> {
    match request {
        AnimationAuthoringToolRequestV1::BlendLoop { policy } => {
            Ok(AnimationAuthoringToolResultV1::LoopBlended {
                result: blend_animation_loop_seam_v1(clip, rig, policy)?,
            })
        }
        AnimationAuthoringToolRequestV1::TransformRootMotion { policy } => {
            Ok(AnimationAuthoringToolResultV1::RootMotionTransformed {
                result: transform_root_motion_v1(clip, rig, *policy)?,
            })
        }
        AnimationAuthoringToolRequestV1::LockSuggestedContacts { contact_set } => {
            let intervals = suggest_contact_intervals_v1(clip, rig, contact_set)?;
            let mut output = clip.clone();
            let mut results = Vec::new();
            for interval in &intervals {
                let result = lock_contact_bone_interval_v1(
                    &output,
                    rig,
                    interval,
                    contact_set.sample_rate_hz,
                    contact_set.ground_axis,
                )?;
                output = result.clip.clone();
                results.push(result);
            }
            Ok(AnimationAuthoringToolResultV1::SuggestedContactsLocked {
                intervals,
                results,
                clip: output,
            })
        }
        AnimationAuthoringToolRequestV1::ReduceKeys { policy } => {
            Ok(AnimationAuthoringToolResultV1::KeysReduced {
                report: reduce_animation_keyframes_v1(clip, policy)?,
            })
        }
    }
}

fn apply_command(
    document: &mut AnimationStudioDocumentV1,
    command: &AnimationEditCommandV1,
) -> Result<(), AnimationAuthoringV2DiagnosticV1> {
    match command {
        AnimationEditCommandV1::SetTransition {
            clip_id,
            transition_seconds,
        } => set_authored_animation_transition_v1(
            find_clip_mut(document, clip_id)?,
            *transition_seconds,
        )
        .map_err(from_studio_diagnostic),
        AnimationEditCommandV1::SetBoneKey {
            clip_id,
            track_id,
            key_id,
            target_node_id,
            path,
            time_seconds,
            value,
        } => {
            let clip = find_clip_mut(document, clip_id)?;
            let track_index = clip
                .tracks
                .iter()
                .position(|track| track.target_node_id == *target_node_id && track.path == *path);
            let track = if let Some(index) = track_index {
                &mut clip.tracks[index]
            } else {
                if track_id.trim().is_empty() {
                    return Err(error(
                        "M2A-ANIMATION-COMMAND-TRACK-ID",
                        "trackId",
                        "new track requires a non-empty stable ID",
                        "Generate a stable track identity for this node/path pair.",
                    ));
                }
                clip.tracks.push(AuthoredAnimationTrackV1 {
                    id: track_id.clone(),
                    target_node_id: *target_node_id,
                    path: *path,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    keyframes: Vec::new(),
                });
                clip.tracks.last_mut().expect("just pushed")
            };
            if let Some(existing) = track.keyframes.iter().position(|key| {
                key.id == *key_id || (key.time_seconds - *time_seconds).abs() <= 1.0e-7
            }) {
                let existing_id = track.keyframes[existing].id.clone();
                update_animation_keyframe_v1(
                    track,
                    &existing_id,
                    AnimationKeyframePatchV1 {
                        time_seconds: Some(*time_seconds),
                        value: Some(value.clone()),
                    },
                )
                .map_err(from_studio_diagnostic)?;
            } else {
                if key_id.trim().is_empty() {
                    return Err(error(
                        "M2A-ANIMATION-COMMAND-KEY-ID",
                        "keyId",
                        "new key requires a non-empty stable ID",
                        "Generate one stable key identity for the gesture.",
                    ));
                }
                track.keyframes.push(AnimationKeyframeV1 {
                    id: key_id.clone(),
                    time_seconds: *time_seconds,
                    value: value.clone(),
                });
                track.keyframes.sort_by(|left, right| {
                    left.time_seconds
                        .total_cmp(&right.time_seconds)
                        .then(left.id.cmp(&right.id))
                });
            }
            touch_clip(clip);
            Ok(())
        }
        AnimationEditCommandV1::MoveKey {
            clip_id,
            track_id,
            key_id,
            time_seconds,
        } => {
            let clip = find_clip_mut(document, clip_id)?;
            let track = find_track_mut(clip, track_id)?;
            update_animation_keyframe_v1(
                track,
                key_id,
                AnimationKeyframePatchV1 {
                    time_seconds: Some(*time_seconds),
                    value: None,
                },
            )
            .map_err(from_studio_diagnostic)?;
            touch_clip(clip);
            Ok(())
        }
        AnimationEditCommandV1::RemoveKey {
            clip_id,
            track_id,
            key_id,
        } => {
            let clip = find_clip_mut(document, clip_id)?;
            let track = find_track_mut(clip, track_id)?;
            let index = track
                .keyframes
                .iter()
                .position(|key| key.id == *key_id)
                .ok_or_else(|| {
                    error(
                        "M2A-ANIMATION-COMMAND-KEY-MISSING",
                        "keyId",
                        "animation key is absent",
                        "Select a current keyframe.",
                    )
                })?;
            track.keyframes.remove(index);
            touch_clip(clip);
            Ok(())
        }
        AnimationEditCommandV1::TrimClip {
            clip_id,
            start_seconds,
            end_seconds,
        } => trim_authored_animation_clip_v1(
            find_clip_mut(document, clip_id)?,
            *start_seconds,
            *end_seconds,
        )
        .map_err(from_studio_diagnostic),
        AnimationEditCommandV1::RetimeClip {
            clip_id,
            new_length_seconds,
        } => retime_authored_animation_clip_v1(
            find_clip_mut(document, clip_id)?,
            *new_length_seconds,
        )
        .map_err(from_studio_diagnostic),
        AnimationEditCommandV1::AddEvent { clip_id, event } => {
            add_authored_animation_event_v1(find_clip_mut(document, clip_id)?, event.clone())
                .map_err(from_studio_diagnostic)
        }
        AnimationEditCommandV1::UpdateEvent {
            clip_id,
            event_id,
            time_seconds,
            name,
        } => update_authored_animation_event_v1(
            find_clip_mut(document, clip_id)?,
            event_id,
            AuthoredAnimationEventPatchV1 {
                time_seconds: *time_seconds,
                name: name.clone(),
            },
        )
        .map_err(from_studio_diagnostic),
        AnimationEditCommandV1::MoveEvent {
            clip_id,
            event_id,
            time_seconds,
        } => move_authored_animation_event_v1(
            find_clip_mut(document, clip_id)?,
            event_id,
            *time_seconds,
        )
        .map_err(from_studio_diagnostic),
        AnimationEditCommandV1::RemoveEvent { clip_id, event_id } => {
            remove_authored_animation_event_v1(find_clip_mut(document, clip_id)?, event_id)
                .map_err(from_studio_diagnostic)
        }
    }
}

fn find_clip_mut<'a>(
    document: &'a mut AnimationStudioDocumentV1,
    clip_id: &str,
) -> Result<&'a mut AuthoredAnimationClipV1, AnimationAuthoringV2DiagnosticV1> {
    document
        .authored_clips
        .iter_mut()
        .find(|clip| clip.id == clip_id)
        .ok_or_else(|| {
            error(
                "M2A-ANIMATION-COMMAND-CLIP-MISSING",
                "clipId",
                format!("authored clip {clip_id:?} is absent"),
                "Select a current authored clip.",
            )
        })
}

fn find_track_mut<'a>(
    clip: &'a mut AuthoredAnimationClipV1,
    track_id: &str,
) -> Result<&'a mut AuthoredAnimationTrackV1, AnimationAuthoringV2DiagnosticV1> {
    clip.tracks
        .iter_mut()
        .find(|track| track.id == track_id)
        .ok_or_else(|| {
            error(
                "M2A-ANIMATION-COMMAND-TRACK-MISSING",
                "trackId",
                format!("animation track {track_id:?} is absent"),
                "Select a current animation track.",
            )
        })
}

fn touch_clip(clip: &mut AuthoredAnimationClipV1) {
    clip.status = AuthoredAnimationClipStatusV1::Draft;
    clip.revision = clip.revision.saturating_add(1);
}

fn resolve_world_transform(
    node_id: u32,
    nodes: &BTreeMap<u32, Option<u32>>,
    local: &BTreeMap<u32, ([f32; 3], [f32; 4])>,
    world: &mut BTreeMap<u32, ([f32; 3], [f32; 4])>,
    visiting: &mut BTreeSet<u32>,
) -> Result<([f32; 3], [f32; 4]), Vec<AnimationAuthoringV2DiagnosticV1>> {
    if let Some(transform) = world.get(&node_id) {
        return Ok(*transform);
    }
    if !visiting.insert(node_id) {
        return Err(vec![error(
            "M2A-ANIMATION-POSE-CYCLE",
            format!("rig.nodes[{node_id}].parentId"),
            "rig parent graph contains a cycle",
            "Repair or re-inspect the source rig.",
        )]);
    }
    let local_transform = *local.get(&node_id).ok_or_else(|| {
        vec![error(
            "M2A-ANIMATION-POSE-BONE",
            format!("rig.nodes[{node_id}]"),
            "rig node transform is absent",
            "Re-inspect the exact source rig.",
        )]
    })?;
    let result = match nodes.get(&node_id).copied().flatten() {
        None => local_transform,
        Some(parent_id) => {
            if !nodes.contains_key(&parent_id) {
                return Err(vec![error(
                    "M2A-ANIMATION-POSE-PARENT",
                    format!("rig.nodes[{node_id}].parentId"),
                    "rig node references an absent parent",
                    "Repair or re-inspect the source rig.",
                )]);
            }
            let parent = resolve_world_transform(parent_id, nodes, local, world, visiting)?;
            (
                add3(parent.0, rotate3(parent.1, local_transform.0)),
                canonical_quaternion(multiply_quaternion(parent.1, local_transform.1))?,
            )
        }
    };
    visiting.remove(&node_id);
    world.insert(node_id, result);
    Ok(result)
}

fn parity_sample_times(
    expected: &AuthoredAnimationClipV1,
    actual: &AuthoredAnimationClipV1,
    policy: &AnimationPoseParityPolicyV1,
) -> Vec<f32> {
    let mut times = uniform_sample_times(
        expected.length_seconds,
        policy.sample_rate_hz,
        policy.max_samples,
    );
    times.extend(
        expected
            .tracks
            .iter()
            .chain(&actual.tracks)
            .flat_map(|track| track.keyframes.iter().map(|key| key.time_seconds)),
    );
    times.extend(
        expected
            .events
            .iter()
            .chain(&actual.events)
            .map(|event| event.time_seconds),
    );
    times.sort_by(f32::total_cmp);
    times.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-7);
    if times.len() > policy.max_samples {
        let mut bounded = uniform_sample_times(
            expected.length_seconds,
            policy.sample_rate_hz,
            policy.max_samples,
        );
        bounded.truncate(policy.max_samples);
        bounded
    } else {
        times
    }
}

fn uniform_sample_times(duration: f32, rate: u32, max_samples: usize) -> Vec<f32> {
    let max_samples = max_samples.max(2);
    let requested = ((duration * rate as f32).ceil() as usize)
        .saturating_add(1)
        .clamp(2, max_samples);
    (0..requested)
        .map(|index| duration * index as f32 / (requested - 1) as f32)
        .collect()
}

fn skeleton_height(pose: &AnimationWorldPoseV1) -> f32 {
    if pose.bones.is_empty() {
        return 1.0;
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for bone in &pose.bones {
        for axis in 0..3 {
            min[axis] = min[axis].min(bone.translation[axis]);
            max[axis] = max[axis].max(bone.translation[axis]);
        }
    }
    distance3(min, max).max(1.0)
}

fn pose_delta(left: &AnimationWorldPoseV1, right: &AnimationWorldPoseV1) -> (f32, f32) {
    let right_by_id = right
        .bones
        .iter()
        .map(|bone| (bone.node_id, bone))
        .collect::<BTreeMap<_, _>>();
    let mut translation = 0.0_f32;
    let mut angular = 0.0_f32;
    for left_bone in &left.bones {
        let Some(right_bone) = right_by_id.get(&left_bone.node_id) else {
            continue;
        };
        translation = translation.max(distance3(left_bone.translation, right_bone.translation));
        angular = angular.max(quaternion_angle(left_bone.rotation, right_bone.rotation));
    }
    (translation, angular)
}

fn find_bone(pose: &AnimationWorldPoseV1, node_id: u32) -> Option<&AnimationBoneWorldTransformV1> {
    pose.bones.iter().find(|bone| bone.node_id == node_id)
}

fn validate_pose_policy(
    policy: &AnimationPoseParityPolicyV1,
) -> Result<(), Vec<AnimationAuthoringV2DiagnosticV1>> {
    if policy.schema_version != ANIMATION_POSE_PARITY_POLICY_SCHEMA_VERSION
        || policy.sample_rate_hz == 0
        || policy.sample_rate_hz > 240
        || policy.max_samples < 2
        || policy.max_samples > MAX_QUALITY_SAMPLES
        || !policy.translation_epsilon.is_finite()
        || policy.translation_epsilon < 0.0
        || !policy.angular_epsilon_radians.is_finite()
        || policy.angular_epsilon_radians < 0.0
    {
        return Err(vec![error(
            "M2A-ANIMATION-POSE-POLICY",
            "policy",
            "pose parity policy is unsupported or non-finite",
            "Use the current bounded AnimationPoseParityPolicyV1.",
        )]);
    }
    Ok(())
}

fn validate_quality_policy(
    policy: &AnimationQualityPolicyV1,
    context: &AnimationQualityContextV1,
) -> Result<(), Vec<AnimationAuthoringV2DiagnosticV1>> {
    let values = [
        policy.static_translation_epsilon_per_height,
        policy.static_angular_epsilon_radians,
        policy.loop_translation_epsilon_per_height,
        policy.loop_angular_epsilon_radians,
        policy.root_drift_warning_per_height,
        policy.ground_penetration_warning_per_height,
        policy.contact_height_per_height,
        policy.foot_slide_warning_per_height,
        policy.max_translation_speed_per_height,
        policy.max_angular_speed_radians,
    ];
    if policy.schema_version != ANIMATION_QUALITY_POLICY_SCHEMA_VERSION
        || policy.sample_rate_hz == 0
        || policy.sample_rate_hz > 240
        || policy.max_samples < 2
        || policy.max_samples > MAX_QUALITY_SAMPLES
        || values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        || context.ground_axis > 2
        || !context.ground_height.is_finite()
    {
        return Err(vec![error(
            "M2A-ANIMATION-QUALITY-POLICY",
            "policy",
            "motion quality policy or context is unsupported or non-finite",
            "Use a bounded Core-owned AnimationQualityPolicyV1.",
        )]);
    }
    Ok(())
}

fn from_studio_diagnostic(
    diagnostic: crate::animation_studio::AnimationStudioDiagnosticV1,
) -> AnimationAuthoringV2DiagnosticV1 {
    AnimationAuthoringV2DiagnosticV1 {
        code: diagnostic.code,
        path: diagnostic.path,
        message: diagnostic.message,
        action: diagnostic.action,
    }
}

#[allow(clippy::too_many_arguments)]
fn issue(
    kind: AnimationQualityIssueKindV1,
    severity: AnimationQualitySeverityV1,
    node_id: Option<u32>,
    start_seconds: f32,
    end_seconds: f32,
    metric: f32,
    threshold: f32,
    message: impl Into<String>,
    action: impl Into<String>,
) -> AnimationQualityIssueV1 {
    AnimationQualityIssueV1 {
        kind,
        severity,
        node_id,
        start_seconds: canonical_f32(start_seconds),
        end_seconds: canonical_f32(end_seconds),
        metric: canonical_f32(metric),
        threshold: canonical_f32(threshold),
        message: message.into(),
        action: action.into(),
    }
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
    action: impl Into<String>,
) -> AnimationAuthoringV2DiagnosticV1 {
    AnimationAuthoringV2DiagnosticV1 {
        code: code.into(),
        path: path.into(),
        message: message.into(),
        action: action.into(),
    }
}

fn resolve_animation_root_node_v1(
    rig: &AnimationStudioRigV1,
) -> Result<&crate::animation_studio::AnimationStudioRigNodeV1, Vec<AnimationAuthoringV2DiagnosticV1>>
{
    rig.nodes
        .iter()
        .find(|node| node.name == rig.animation_root)
        .ok_or_else(|| {
            vec![error(
                "M2A-ANIMATION-ROOT-MISSING",
                "rig.animationRoot",
                "animation root is absent from the exact rig",
                "Re-inspect the source rig before editing root motion.",
            )]
        })
}

fn root_track_displacement_v1(
    track: &AuthoredAnimationTrackV1,
) -> Result<[f32; 3], Vec<AnimationAuthoringV2DiagnosticV1>> {
    let first = track.keyframes.first().and_then(|key| vec3(&key.value));
    let last = track.keyframes.last().and_then(|key| vec3(&key.value));
    match first.zip(last) {
        Some((first, last)) => Ok([
            canonical_f32(last[0] - first[0]),
            canonical_f32(last[1] - first[1]),
            canonical_f32(last[2] - first[2]),
        ]),
        None => Err(vec![error(
            "M2A-ANIMATION-ROOT-TRACK",
            "clip.tracks.root.keyframes",
            "root translation track requires valid first and last keys",
            "Repair the root translation keys.",
        )]),
    }
}

fn blend_track_value(
    path: AuthoredAnimationTrackPathV1,
    left: &[f32],
    right: &[f32],
    amount: f32,
) -> Result<Vec<f32>, Vec<AnimationAuthoringV2DiagnosticV1>> {
    let amount = amount.clamp(0.0, 1.0);
    match path {
        AuthoredAnimationTrackPathV1::Translation => {
            let left = vec3(left).ok_or_else(|| {
                vec![error(
                    "M2A-ANIMATION-TRACK-VALUE",
                    "track.keyframes.value",
                    "translation track value must contain three finite numbers",
                    "Repair the translation track.",
                )]
            })?;
            let right = vec3(right).ok_or_else(|| {
                vec![error(
                    "M2A-ANIMATION-TRACK-VALUE",
                    "track.keyframes.value",
                    "translation track value must contain three finite numbers",
                    "Repair the translation track.",
                )]
            })?;
            Ok((0..3)
                .map(|axis| canonical_f32(left[axis] + (right[axis] - left[axis]) * amount))
                .collect())
        }
        AuthoredAnimationTrackPathV1::Rotation => {
            let left = quaternion(left).ok_or_else(|| {
                vec![error(
                    "M2A-ANIMATION-TRACK-VALUE",
                    "track.keyframes.value",
                    "rotation track value must contain a finite quaternion",
                    "Repair the rotation track.",
                )]
            })?;
            let mut right = quaternion(right).ok_or_else(|| {
                vec![error(
                    "M2A-ANIMATION-TRACK-VALUE",
                    "track.keyframes.value",
                    "rotation track value must contain a finite quaternion",
                    "Repair the rotation track.",
                )]
            })?;
            if left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>() < 0.0 {
                right = right.map(|value| -value);
            }
            let blended = canonical_quaternion([
                left[0] + (right[0] - left[0]) * amount,
                left[1] + (right[1] - left[1]) * amount,
                left[2] + (right[2] - left[2]) * amount,
                left[3] + (right[3] - left[3]) * amount,
            ])?;
            Ok(blended.into_iter().map(canonical_f32).collect())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn reduce_key_segment_v1(
    path: AuthoredAnimationTrackPathV1,
    keys: &[AnimationKeyframeV1],
    start: usize,
    end: usize,
    policy: &AnimationKeyReductionPolicyV1,
    keep: &mut [bool],
    max_position_error: &mut f32,
    max_angular_error_radians: &mut f32,
) -> Result<(), Vec<AnimationAuthoringV2DiagnosticV1>> {
    if end <= start + 1 {
        return Ok(());
    }
    let duration = keys[end].time_seconds - keys[start].time_seconds;
    if duration <= 0.0 || !duration.is_finite() {
        return Err(vec![error(
            "M2A-ANIMATION-REDUCTION-TIME",
            "track.keyframes",
            "key reduction requires strictly increasing finite key times",
            "Sort and repair the track before reducing it.",
        )]);
    }
    let mut worst_index = None;
    let mut worst_ratio = 0.0_f32;
    let mut segment_position_error = 0.0_f32;
    let mut segment_angular_error = 0.0_f32;
    for index in start + 1..end {
        let amount = (keys[index].time_seconds - keys[start].time_seconds) / duration;
        let expected = blend_track_value(path, &keys[start].value, &keys[end].value, amount)?;
        let (position_error, angular_error, ratio) = match path {
            AuthoredAnimationTrackPathV1::Translation => {
                let actual = vec3(&keys[index].value).ok_or_else(|| {
                    vec![error(
                        "M2A-ANIMATION-REDUCTION-VALUE",
                        "track.keyframes.value",
                        "translation reduction encountered a malformed value",
                        "Repair the track before reducing it.",
                    )]
                })?;
                let expected = vec3(&expected).expect("blended translation is valid");
                let error = distance3(actual, expected);
                let ratio = if policy.max_position_error == 0.0 {
                    if error == 0.0 { 0.0 } else { f32::INFINITY }
                } else {
                    error / policy.max_position_error
                };
                (error, 0.0, ratio)
            }
            AuthoredAnimationTrackPathV1::Rotation => {
                let actual = quaternion(&keys[index].value).ok_or_else(|| {
                    vec![error(
                        "M2A-ANIMATION-REDUCTION-VALUE",
                        "track.keyframes.value",
                        "rotation reduction encountered a malformed quaternion",
                        "Repair the track before reducing it.",
                    )]
                })?;
                let expected = quaternion(&expected).expect("blended quaternion is valid");
                let error = quaternion_angle(actual, expected);
                let ratio = if policy.max_angular_error_radians == 0.0 {
                    if error == 0.0 { 0.0 } else { f32::INFINITY }
                } else {
                    error / policy.max_angular_error_radians
                };
                (0.0, error, ratio)
            }
        };
        segment_position_error = segment_position_error.max(position_error);
        segment_angular_error = segment_angular_error.max(angular_error);
        if ratio > worst_ratio {
            worst_ratio = ratio;
            worst_index = Some(index);
        }
    }
    if worst_ratio > 1.0 {
        let index = worst_index.expect("a non-empty segment has a worst key");
        keep[index] = true;
        reduce_key_segment_v1(
            path,
            keys,
            start,
            index,
            policy,
            keep,
            max_position_error,
            max_angular_error_radians,
        )?;
        reduce_key_segment_v1(
            path,
            keys,
            index,
            end,
            policy,
            keep,
            max_position_error,
            max_angular_error_radians,
        )?;
    } else {
        *max_position_error = max_position_error.max(segment_position_error);
        *max_angular_error_radians = max_angular_error_radians.max(segment_angular_error);
    }
    Ok(())
}

fn subtract3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn horizontal_distance3(left: [f32; 3], right: [f32; 3], ground_axis: u8) -> f32 {
    let mut delta = subtract3(left, right);
    delta[ground_axis as usize] = 0.0;
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

fn append_contact_interval_v1(
    intervals: &mut Vec<AnimationContactIntervalV1>,
    node_id: u32,
    start: usize,
    end: usize,
    times: &[f32],
    positions: &[[f32; 3]],
    ground_axis: u8,
) {
    if end <= start {
        return;
    }
    let count = end - start + 1;
    let mut anchor = [0.0_f32; 3];
    for position in &positions[start..=end] {
        for axis in 0..3 {
            anchor[axis] += position[axis] / count as f32;
        }
    }
    let max_slide_before = positions[start..=end]
        .iter()
        .map(|position| horizontal_distance3(*position, anchor, ground_axis))
        .fold(0.0_f32, f32::max);
    intervals.push(AnimationContactIntervalV1 {
        node_id,
        start_seconds: times[start],
        end_seconds: times[end],
        anchor_world_translation: canonical_vec3(anchor),
        max_slide_before: canonical_f32(max_slide_before),
    });
}

fn sample_node_positions_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    node_id: u32,
    times: &[f32],
) -> Result<Vec<[f32; 3]>, Vec<AnimationAuthoringV2DiagnosticV1>> {
    times
        .iter()
        .map(|time| {
            sample_animation_world_pose_v1(clip, rig, *time).and_then(|pose| {
                pose.bones
                    .iter()
                    .find(|bone| bone.node_id == node_id)
                    .map(|bone| bone.translation)
                    .ok_or_else(|| {
                        vec![error(
                            "M2A-ANIMATION-CONTACT-BONE",
                            "pose.bones",
                            "contact bone is absent from the sampled pose",
                            "Re-inspect the exact rig.",
                        )]
                    })
            })
        })
        .collect()
}

fn max_horizontal_slide_v1(positions: &[[f32; 3]], anchor: [f32; 3], ground_axis: u8) -> f32 {
    positions
        .iter()
        .map(|position| horizontal_distance3(*position, anchor, ground_axis))
        .fold(0.0_f32, f32::max)
}

fn conjugate_quaternion(value: [f32; 4]) -> [f32; 4] {
    [-value[0], -value[1], -value[2], value[3]]
}

fn scale3(value: [f32; 3], factor: f32) -> [f32; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

fn vec3(value: &[f32]) -> Option<[f32; 3]> {
    if value.len() != 3 || value.iter().any(|entry| !entry.is_finite()) {
        None
    } else {
        Some([value[0], value[1], value[2]])
    }
}

fn quaternion(value: &[f32]) -> Option<[f32; 4]> {
    if value.len() != 4 || value.iter().any(|entry| !entry.is_finite()) {
        None
    } else {
        canonical_quaternion([value[0], value[1], value[2], value[3]]).ok()
    }
}

fn canonical_quaternion(
    mut value: [f32; 4],
) -> Result<[f32; 4], Vec<AnimationAuthoringV2DiagnosticV1>> {
    if value.iter().any(|entry| !entry.is_finite()) {
        return Err(vec![error(
            "M2A-ANIMATION-POSE-QUATERNION",
            "rotation",
            "quaternion contains a non-finite value",
            "Repair the rotation track.",
        )]);
    }
    let norm = value.iter().map(|entry| entry * entry).sum::<f32>().sqrt();
    if !norm.is_finite() || norm <= 1.0e-8 {
        return Err(vec![error(
            "M2A-ANIMATION-POSE-QUATERNION",
            "rotation",
            "quaternion cannot be normalized",
            "Repair the zero-length rotation key.",
        )]);
    }
    for entry in &mut value {
        *entry = canonical_f32(*entry / norm);
    }
    let dominant = value
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))
        .map(|(index, _)| index)
        .unwrap_or(3);
    if value[dominant] < 0.0 {
        for entry in &mut value {
            *entry = canonical_f32(-*entry);
        }
    }
    Ok(value)
}

fn multiply_quaternion(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    [
        left[3] * right[0] + left[0] * right[3] + left[1] * right[2] - left[2] * right[1],
        left[3] * right[1] - left[0] * right[2] + left[1] * right[3] + left[2] * right[0],
        left[3] * right[2] + left[0] * right[1] - left[1] * right[0] + left[2] * right[3],
        left[3] * right[3] - left[0] * right[0] - left[1] * right[1] - left[2] * right[2],
    ]
}

fn rotate3(rotation: [f32; 4], vector: [f32; 3]) -> [f32; 3] {
    let pure = [vector[0], vector[1], vector[2], 0.0];
    let inverse = [-rotation[0], -rotation[1], -rotation[2], rotation[3]];
    let result = multiply_quaternion(multiply_quaternion(rotation, pure), inverse);
    [result[0], result[1], result[2]]
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn distance3(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2))
        .sqrt()
}

fn horizontal_distance(left: [f32; 3], right: [f32; 3], up_axis: usize) -> f32 {
    let mut squared = 0.0;
    for axis in 0..3 {
        if axis != up_axis {
            squared += (left[axis] - right[axis]).powi(2);
        }
    }
    squared.sqrt()
}

fn quaternion_angle(left: [f32; 4], right: [f32; 4]) -> f32 {
    let dot = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f32>()
        .abs()
        .clamp(0.0, 1.0);
    2.0 * dot.acos()
}

fn canonical_vec3(value: [f32; 3]) -> [f32; 3] {
    [
        canonical_f32(value[0]),
        canonical_f32(value[1]),
        canonical_f32(value[2]),
    ]
}

fn canonical_f32(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

fn fingerprint_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("serializable Animation Authoring V2 contract");
    format!("{:x}", Sha256::digest(bytes))
}
