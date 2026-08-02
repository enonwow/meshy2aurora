//! Single strict request boundary for advanced animation authoring operations.
//! Keeping one tagged contract prevents Core/WASM/Worker from drifting while
//! still returning strongly typed, versioned operation results.

use serde::{Deserialize, Serialize};

use crate::{
    animation_motion_tools::{
        AnimationVariantRecipeV1, CombatAttackPhaseTimelineV1, CombatQualityContextV1,
        CombatQualityPolicyV1, MotionTrailRequestV1, analyze_combat_animation_quality_v1,
        authored_clip_fingerprint_v1, build_motion_visualization_v1, create_animation_variant_v1,
        create_combat_attack_phase_timeline_v1, retime_combat_attack_phases_v1,
    },
    animation_pose::{
        AnimationMirrorAxisV1, AnimationPosePasteModeV1, AnimationPoseSnapshotV1,
        HumanoidMirrorMapV1, HumanoidMirrorPairV1, apply_animation_pose_v1,
        build_humanoid_mirror_map_v1, capture_animation_pose_v1, mirror_animation_pose_v1,
        reset_animation_pose_to_rest_v1,
    },
    animation_sequence::{AnimationSequenceDocumentV1, bake_animation_sequence_document_v1},
    animation_studio::{AnimationStudioRigV1, AuthoredAnimationClipV1},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AnimationWorkbenchRequestV1 {
    FingerprintClip {
        clip: AuthoredAnimationClipV1,
    },
    CapturePose {
        clip: AuthoredAnimationClipV1,
        rig: AnimationStudioRigV1,
        time_seconds: f32,
        selected_node_ids: Vec<u32>,
    },
    BuildMirrorMap {
        rig: AnimationStudioRigV1,
        axis: AnimationMirrorAxisV1,
        pairs: Vec<HumanoidMirrorPairV1>,
        center_node_ids: Vec<u32>,
    },
    MirrorPose {
        pose: AnimationPoseSnapshotV1,
        rig: AnimationStudioRigV1,
        mapping: HumanoidMirrorMapV1,
    },
    ApplyPose {
        clip: AuthoredAnimationClipV1,
        rig: AnimationStudioRigV1,
        pose: AnimationPoseSnapshotV1,
        time_seconds: f32,
        mode: AnimationPosePasteModeV1,
        selected_node_ids: Vec<u32>,
    },
    ResetPose {
        clip: AuthoredAnimationClipV1,
        rig: AnimationStudioRigV1,
        time_seconds: f32,
        selected_node_ids: Vec<u32>,
    },
    CreateCombatPhases {
        clip: AuthoredAnimationClipV1,
        phase_times: [f32; 5],
    },
    RetimeCombatPhases {
        clip: AuthoredAnimationClipV1,
        timeline: CombatAttackPhaseTimelineV1,
        target: CombatAttackPhaseTimelineV1,
    },
    AnalyzeCombatQuality {
        clip: AuthoredAnimationClipV1,
        rig: AnimationStudioRigV1,
        timeline: CombatAttackPhaseTimelineV1,
        policy: CombatQualityPolicyV1,
        context: CombatQualityContextV1,
    },
    BuildMotionVisualization {
        clip: AuthoredAnimationClipV1,
        rig: AnimationStudioRigV1,
        request: MotionTrailRequestV1,
    },
    CreateVariant {
        source: AuthoredAnimationClipV1,
        rig: AnimationStudioRigV1,
        recipe: AnimationVariantRecipeV1,
    },
    BakeSequence {
        document: AnimationSequenceDocumentV1,
        available_clips: Vec<AuthoredAnimationClipV1>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "result",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AnimationWorkbenchResultV1 {
    ClipFingerprinted {
        value: String,
    },
    PoseCaptured {
        value: crate::animation_pose::AnimationPoseSnapshotV1,
    },
    MirrorMapBuilt {
        value: crate::animation_pose::HumanoidMirrorMapV1,
    },
    PoseMirrored {
        value: crate::animation_pose::AnimationPoseSnapshotV1,
    },
    PoseApplied {
        value: crate::animation_pose::AnimationPoseApplyResultV1,
    },
    PoseReset {
        value: crate::animation_pose::AnimationPoseApplyResultV1,
    },
    CombatPhasesCreated {
        value: crate::animation_motion_tools::CombatAttackPhaseTimelineV1,
    },
    CombatPhasesRetimed {
        value: AuthoredAnimationClipV1,
    },
    CombatQualityAnalyzed {
        value: crate::animation_motion_tools::CombatQualityReportV1,
    },
    MotionVisualizationBuilt {
        value: crate::animation_motion_tools::MotionVisualizationV1,
    },
    VariantCreated {
        value: crate::animation_motion_tools::AnimationVariantResultV1,
    },
    SequenceBaked {
        value: Box<crate::animation_sequence::AnimationSequenceBakeV1>,
    },
}

pub fn apply_animation_workbench_operation_v1(
    request: AnimationWorkbenchRequestV1,
) -> Result<AnimationWorkbenchResultV1, String> {
    Ok(match request {
        AnimationWorkbenchRequestV1::FingerprintClip { clip } => {
            AnimationWorkbenchResultV1::ClipFingerprinted {
                value: authored_clip_fingerprint_v1(&clip),
            }
        }
        AnimationWorkbenchRequestV1::CapturePose {
            clip,
            rig,
            time_seconds,
            selected_node_ids,
        } => AnimationWorkbenchResultV1::PoseCaptured {
            value: capture_animation_pose_v1(&clip, &rig, time_seconds, &selected_node_ids)?,
        },
        AnimationWorkbenchRequestV1::BuildMirrorMap {
            rig,
            axis,
            pairs,
            center_node_ids,
        } => AnimationWorkbenchResultV1::MirrorMapBuilt {
            value: build_humanoid_mirror_map_v1(&rig, axis, pairs, center_node_ids)?,
        },
        AnimationWorkbenchRequestV1::MirrorPose { pose, rig, mapping } => {
            AnimationWorkbenchResultV1::PoseMirrored {
                value: mirror_animation_pose_v1(&pose, &rig, &mapping)?,
            }
        }
        AnimationWorkbenchRequestV1::ApplyPose {
            clip,
            rig,
            pose,
            time_seconds,
            mode,
            selected_node_ids,
        } => AnimationWorkbenchResultV1::PoseApplied {
            value: apply_animation_pose_v1(
                &clip,
                &rig,
                &pose,
                time_seconds,
                mode,
                &selected_node_ids,
            )?,
        },
        AnimationWorkbenchRequestV1::ResetPose {
            clip,
            rig,
            time_seconds,
            selected_node_ids,
        } => AnimationWorkbenchResultV1::PoseReset {
            value: reset_animation_pose_to_rest_v1(&clip, &rig, time_seconds, &selected_node_ids)?,
        },
        AnimationWorkbenchRequestV1::CreateCombatPhases { clip, phase_times } => {
            AnimationWorkbenchResultV1::CombatPhasesCreated {
                value: create_combat_attack_phase_timeline_v1(&clip, phase_times)?,
            }
        }
        AnimationWorkbenchRequestV1::RetimeCombatPhases {
            clip,
            timeline,
            target,
        } => AnimationWorkbenchResultV1::CombatPhasesRetimed {
            value: retime_combat_attack_phases_v1(&clip, &timeline, &target)?,
        },
        AnimationWorkbenchRequestV1::AnalyzeCombatQuality {
            clip,
            rig,
            timeline,
            policy,
            context,
        } => AnimationWorkbenchResultV1::CombatQualityAnalyzed {
            value: analyze_combat_animation_quality_v1(&clip, &rig, &timeline, &policy, &context)?,
        },
        AnimationWorkbenchRequestV1::BuildMotionVisualization { clip, rig, request } => {
            AnimationWorkbenchResultV1::MotionVisualizationBuilt {
                value: build_motion_visualization_v1(&clip, &rig, &request)?,
            }
        }
        AnimationWorkbenchRequestV1::CreateVariant {
            source,
            rig,
            recipe,
        } => AnimationWorkbenchResultV1::VariantCreated {
            value: create_animation_variant_v1(&source, &rig, &recipe)?,
        },
        AnimationWorkbenchRequestV1::BakeSequence {
            document,
            available_clips,
        } => AnimationWorkbenchResultV1::SequenceBaked {
            value: Box::new(bake_animation_sequence_document_v1(
                &document,
                &available_clips,
            )?),
        },
    })
}
