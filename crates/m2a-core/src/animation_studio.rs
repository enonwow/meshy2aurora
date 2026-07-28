//! Versioned, local-first animation authoring contracts and canonical editing
//! operations.
//!
//! This module is additive: the V1 creature mapping contract remains unchanged.
//! Studio clips own editable motion while V2 mapping definitions only route
//! stable clip identities to Aurora output names.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    creature_animation_mapping::{
        AnimationFallbackDecisionV1, AnimationFallbackReviewV1, AnimationMappingProvenanceV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1, CreatureAnimationAuthoringV1,
        CustomAnimationPhaseKindV1, CustomAnimationPlaybackV1, DirectCreatureModelTypeV1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    mdl::{
        AnimationReport, InspectionReport, MdlAnimationClipV1, MdlAnimationEventV1,
        MdlAnimationInterpolationV1, MdlAnimationSetV1, MdlAnimationTrackPathV1,
        MdlAnimationTrackV1, NodeReport,
    },
};

pub const ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION: u32 = 1;
pub const CREATURE_ANIMATION_AUTHORING_SCHEMA_VERSION_V2: u32 = 2;
pub const ANIMATION_STUDIO_DIAGNOSTIC_SCHEMA_VERSION: u32 = 1;
pub const ANIMATION_STUDIO_READBACK_SCHEMA_VERSION: u32 = 1;
pub const ANIMATION_STUDIO_RIG_SCHEMA_VERSION: u32 = 1;
/// Product limits for one local-first Studio document. They are deliberately
/// independent from the binary writer's per-controller `u16` row boundary.
pub const ANIMATION_STUDIO_MAX_AUTHORED_CLIPS: usize = 64;
pub const ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP: usize = 10_000;
pub const ANIMATION_STUDIO_MAX_TOTAL_KEYFRAMES: usize = 100_000;
pub const ANIMATION_STUDIO_MDL_ROW_LIMIT: usize = 65_535;
pub const ANIMATION_STUDIO_EVENT_NAME_MAX_ASCII_BYTES: usize = 31;
/// A local authored clip cannot exceed one day. This is a product/editor
/// boundary, independent from binary row limits, and keeps the shared f32 JSON
/// fingerprint in one unambiguous decimal notation domain.
pub const ANIMATION_STUDIO_MAX_DURATION_SECONDS: f32 = 86_400.0;
/// Absolute transform component boundary for local authoring. Rotation values
/// are normalized separately; translations beyond this range are not useful in
/// an Aurora model and are rejected before serialization/build.
pub const ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE: f32 = 1_000_000.0;

pub const DIAGNOSTIC_SCHEMA: &str = "M2A-ANIMATION-EDIT-SCHEMA";
pub const DIAGNOSTIC_SOURCE_STALE: &str = "M2A-ANIMATION-EDIT-SOURCE-STALE";
pub const DIAGNOSTIC_CLIP_NAME: &str = "M2A-ANIMATION-EDIT-CLIP-NAME";
pub const DIAGNOSTIC_CLIP_DUPLICATE: &str = "M2A-ANIMATION-EDIT-CLIP-DUPLICATE";
pub const DIAGNOSTIC_BONE_MISSING: &str = "M2A-ANIMATION-EDIT-BONE-MISSING";
pub const DIAGNOSTIC_PATH_UNSUPPORTED: &str = "M2A-ANIMATION-EDIT-PATH-UNSUPPORTED";
pub const DIAGNOSTIC_TIME_NOT_STRICT: &str = "M2A-ANIMATION-EDIT-TIME-NOT-STRICT";
pub const DIAGNOSTIC_TIME_OOB: &str = "M2A-ANIMATION-EDIT-TIME-OOB";
pub const DIAGNOSTIC_VALUE_NONFINITE: &str = "M2A-ANIMATION-EDIT-VALUE-NONFINITE";
pub const DIAGNOSTIC_QUATERNION: &str = "M2A-ANIMATION-EDIT-QUATERNION";
pub const DIAGNOSTIC_NO_MOTION: &str = "M2A-ANIMATION-EDIT-NO-MOTION";
pub const DIAGNOSTIC_EVENT: &str = "M2A-ANIMATION-EDIT-EVENT";
pub const DIAGNOSTIC_ROW_LIMIT: &str = "M2A-ANIMATION-EDIT-ROW-LIMIT";
pub const DIAGNOSTIC_READBACK_MISMATCH: &str = "M2A-ANIMATION-EDIT-READBACK-MISMATCH";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationStudioDocumentStatusV1 {
    Draft,
    Valid,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationStudioDocumentV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub authoring_revision: u64,
    pub status: AnimationStudioDocumentStatusV1,
    pub authored_clips: Vec<AuthoredAnimationClipV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthoredAnimationClipKindV1 {
    Motion,
    StaticPose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthoredAnimationClipStatusV1 {
    Draft,
    Valid,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthoredAnimationSourceKindV1 {
    BlankPose,
    SourceClipCopy,
    ProceduralTemplate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationSourceV1 {
    pub kind: AuthoredAnimationSourceKindV1,
    pub source_revision: String,
    pub source_clip_name: Option<String>,
    pub source_clip_fingerprint: Option<String>,
    pub procedural_template: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthoredAnimationTrackPathV1 {
    Translation,
    Rotation,
}

impl AuthoredAnimationTrackPathV1 {
    fn expected_value_count(self) -> usize {
        match self {
            Self::Translation => 3,
            Self::Rotation => 4,
        }
    }

    fn id_suffix(self) -> &'static str {
        match self {
            Self::Translation => "translation",
            Self::Rotation => "rotation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationKeyframeV1 {
    pub id: String,
    pub time_seconds: f32,
    pub value: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationTrackV1 {
    pub id: String,
    pub target_node_id: u32,
    pub path: AuthoredAnimationTrackPathV1,
    pub interpolation: MdlAnimationInterpolationV1,
    pub keyframes: Vec<AnimationKeyframeV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationEventV1 {
    pub id: String,
    pub time_seconds: f32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationClipV1 {
    pub id: String,
    pub name: String,
    pub kind: AuthoredAnimationClipKindV1,
    pub status: AuthoredAnimationClipStatusV1,
    pub source: AuthoredAnimationSourceV1,
    pub length_seconds: f32,
    pub transition_seconds: f32,
    pub animation_root: String,
    pub tracks: Vec<AuthoredAnimationTrackV1>,
    pub events: Vec<AuthoredAnimationEventV1>,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationClipInputV1 {
    pub id: String,
    pub name: String,
    /// SHA-256 of the exact immutable source GLB.
    pub source_revision: String,
    pub length_seconds: f32,
    pub transition_seconds: f32,
    pub animation_root: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationStudioRigV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub animation_root: String,
    pub nodes: Vec<AnimationStudioRigNodeV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationStudioRigNodeV1 {
    pub node_id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProceduralAnimationTemplateV1 {
    BindPose,
    RootTranslationPulse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomAnimationClipReferenceKindV2 {
    SourceClip,
    AuthoredClip,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomAnimationClipReferenceV2 {
    pub source_kind: CustomAnimationClipReferenceKindV2,
    pub source_clip_name: Option<String>,
    pub authored_clip_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomAnimationPhaseV2 {
    pub phase: CustomAnimationPhaseKindV1,
    pub clip_reference: CustomAnimationClipReferenceV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomAnimationDefinitionV2 {
    pub id: String,
    pub name: String,
    pub playback: CustomAnimationPlaybackV1,
    pub clip_reference: Option<CustomAnimationClipReferenceV2>,
    pub phases: Vec<CustomAnimationPhaseV2>,
    pub provenance: AnimationMappingProvenanceV1,
}

pub type AnimationSourceAssignmentV2 = AnimationSourceAssignmentV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureAnimationAuthoringV2 {
    pub schema_version: u32,
    pub profile: String,
    pub model_type: DirectCreatureModelTypeV1,
    pub source_revision: String,
    pub authoring_revision: u64,
    pub assignments: Vec<AnimationSourceAssignmentV2>,
    pub fallbacks: Vec<AnimationFallbackDecisionV1>,
    pub custom_animations: Vec<CustomAnimationDefinitionV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationStudioDiagnosticLevelV1 {
    Info,
    Warning,
    Blocking,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationStudioDiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub level: AnimationStudioDiagnosticLevelV1,
    pub message: String,
    pub action: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationStudioReadbackStatusV1 {
    Match,
    Mismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationStudioReadbackClipV1 {
    pub authored_clip_id: String,
    pub output_clip_name: String,
    pub materialized_fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationStudioReadbackV1 {
    pub schema_version: u32,
    pub studio_fingerprint: String,
    pub source_revision: String,
    pub status: AnimationStudioReadbackStatusV1,
    pub clips: Vec<AnimationStudioReadbackClipV1>,
    pub diagnostics: Vec<AnimationStudioDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomAnimationLibraryItemV1 {
    pub authored_clip_id: String,
    pub name: String,
    pub status: AuthoredAnimationClipStatusV1,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredAnimationLibraryClipV1 {
    pub authored_clip_id: String,
    pub status: AuthoredAnimationClipStatusV1,
    pub source: AuthoredAnimationSourceV1,
    pub clip: MdlAnimationClipV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredAnimationLibraryV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub clips: Vec<AuthoredAnimationLibraryClipV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthoredAnimationUsageKindV1 {
    BaseSlot,
    CustomOneShot,
    CustomPhase,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredAnimationUsageV1 {
    pub authored_clip_id: String,
    pub output_clip_name: String,
    pub usage_kind: AuthoredAnimationUsageKindV1,
    pub base_slot: Option<String>,
    pub custom_animation_id: Option<String>,
    pub phase: Option<CustomAnimationPhaseKindV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedCustomAnimationClipV2 {
    pub clip: MdlAnimationClipV1,
    pub authored_clip_id: Option<String>,
    pub phase: Option<CustomAnimationPhaseKindV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedCreatureAnimationAuthoringV2 {
    pub schema_version: u32,
    pub animations: MdlAnimationSetV1,
    pub authored_usages: Vec<AuthoredAnimationUsageV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationKeyframePatchV1 {
    pub time_seconds: Option<f32>,
    pub value: Option<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationEventPatchV1 {
    pub time_seconds: Option<f32>,
    pub name: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyframeDeduplicationPolicyV1 {
    Fail,
    ReplaceLast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoveAuthoredClipPolicyV1 {
    Explicit,
}

pub fn serialize_animation_studio_document_v1(
    document: &AnimationStudioDocumentV1,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(document).map(|json| normalize_integral_json_floats(&json))
}

pub fn parse_animation_studio_document_v1(
    json: &str,
) -> Result<AnimationStudioDocumentV1, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn fingerprint_animation_studio_document_v1(document: &AnimationStudioDocumentV1) -> String {
    let canonical = serialize_animation_studio_document_v1(document)
        .expect("AnimationStudioDocumentV1 contains only JSON-serializable fields");
    sha256_hex(canonical.as_bytes())
}

pub fn fingerprint_creature_animation_authoring_v2(
    authoring: &CreatureAnimationAuthoringV2,
) -> String {
    fingerprint_json(authoring)
}

pub fn migrate_creature_animation_authoring_v1_to_v2(
    v1: &CreatureAnimationAuthoringV1,
) -> CreatureAnimationAuthoringV2 {
    CreatureAnimationAuthoringV2 {
        schema_version: CREATURE_ANIMATION_AUTHORING_SCHEMA_VERSION_V2,
        profile: v1.profile.clone(),
        model_type: v1.model_type,
        source_revision: v1.source_revision.clone(),
        authoring_revision: v1.authoring_revision,
        assignments: v1.assignments.clone(),
        fallbacks: v1.fallbacks.clone(),
        custom_animations: v1
            .custom_animations
            .iter()
            .map(|custom| CustomAnimationDefinitionV2 {
                id: custom.id.clone(),
                name: custom.name.clone(),
                playback: custom.playback,
                clip_reference: custom.source_clip_name.as_ref().map(|source_clip_name| {
                    CustomAnimationClipReferenceV2 {
                        source_kind: CustomAnimationClipReferenceKindV2::SourceClip,
                        source_clip_name: Some(source_clip_name.clone()),
                        authored_clip_id: None,
                    }
                }),
                phases: custom
                    .phases
                    .iter()
                    .map(|phase| CustomAnimationPhaseV2 {
                        phase: phase.phase,
                        clip_reference: CustomAnimationClipReferenceV2 {
                            source_kind: CustomAnimationClipReferenceKindV2::SourceClip,
                            source_clip_name: Some(phase.source_clip_name.clone()),
                            authored_clip_id: None,
                        },
                    })
                    .collect(),
                provenance: custom.provenance.clone(),
            })
            .collect(),
    }
}

pub fn validate_animation_studio_schema_v1(
    document: &AnimationStudioDocumentV1,
) -> Vec<AnimationStudioDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if document.schema_version != ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "schemaVersion",
            format!(
                "Animation Studio schema version must be {}, got {}",
                ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION, document.schema_version
            ),
            "Open or migrate a supported Animation Studio document.",
        ));
    }
    if !is_sha256(&document.source_revision) {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "sourceRevision",
            "sourceRevision must be a lowercase or uppercase 64-character SHA-256",
            "Re-inspect the exact source GLB and save its SHA-256.",
        ));
    }
    if document.authored_clips.len() > ANIMATION_STUDIO_MAX_AUTHORED_CLIPS {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_ROW_LIMIT,
            "authoredClips",
            format!(
                "Studio document contains {} authored clips; product limit is {}",
                document.authored_clips.len(),
                ANIMATION_STUDIO_MAX_AUTHORED_CLIPS
            ),
            "Remove or split authored clips before build.",
        ));
    }
    let total_keyframes = document
        .authored_clips
        .iter()
        .flat_map(|clip| &clip.tracks)
        .map(|track| track.keyframes.len())
        .sum::<usize>();
    if total_keyframes > ANIMATION_STUDIO_MAX_TOTAL_KEYFRAMES {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_ROW_LIMIT,
            "authoredClips",
            format!(
                "Studio document contains {total_keyframes} keyframes; product limit is {}",
                ANIMATION_STUDIO_MAX_TOTAL_KEYFRAMES
            ),
            "Reduce or resample keyframes across the Studio document.",
        ));
    }

    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, clip) in document.authored_clips.iter().enumerate() {
        if clip.id.trim().is_empty() || !ids.insert(clip.id.clone()) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_DUPLICATE,
                format!("authoredClips[{index}].id"),
                format!("authored clip id {:?} is empty or duplicated", clip.id),
                "Choose a non-empty stable id unique in this Studio document.",
            ));
        }
        let normalized_name = clip.name.trim().to_ascii_lowercase();
        if normalized_name.is_empty() {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_NAME,
                format!("authoredClips[{index}].name"),
                "authored clip output name cannot be empty",
                "Enter a portable Aurora animation name.",
            ));
        } else if !names.insert(normalized_name) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_DUPLICATE,
                format!("authoredClips[{index}].name"),
                format!("authored output name {:?} is duplicated", clip.name),
                "Rename one clip so output names are unique case-insensitively.",
            ));
        }
        if clip.source.source_revision != document.source_revision {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SOURCE_STALE,
                format!("authoredClips[{index}].source.sourceRevision"),
                format!(
                    "clip source revision {} does not match document source revision {}",
                    clip.source.source_revision, document.source_revision
                ),
                "Reconcile or recreate this draft against the current exact source GLB.",
            ));
        }
    }
    diagnostics
}

pub fn validate_creature_animation_authoring_v2(
    authoring: &CreatureAnimationAuthoringV2,
    studio: &AnimationStudioDocumentV1,
) -> Vec<AnimationStudioDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if authoring.schema_version != CREATURE_ANIMATION_AUTHORING_SCHEMA_VERSION_V2 {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "schemaVersion",
            "CreatureAnimationAuthoringV2 requires schemaVersion 2",
            "Migrate the V1 mapping explicitly before editing it as V2.",
        ));
    }
    if authoring.source_revision != studio.source_revision {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SOURCE_STALE,
            "sourceRevision",
            "mapping and Studio document refer to different source revisions",
            "Reconcile both documents against the exact current source GLB.",
        ));
    }
    let authored_by_id = studio
        .authored_clips
        .iter()
        .map(|clip| (clip.id.as_str(), clip))
        .collect::<BTreeMap<_, _>>();
    let custom_ids = authoring
        .custom_animations
        .iter()
        .map(|custom| custom.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_ids = BTreeSet::new();
    let mut seen_names = BTreeSet::new();
    for (index, custom) in authoring.custom_animations.iter().enumerate() {
        if custom.id.trim().is_empty() || !seen_ids.insert(custom.id.as_str()) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_DUPLICATE,
                format!("customAnimations[{index}].id"),
                "custom animation ids must be non-empty and unique",
                "Choose a stable unique custom animation id.",
            ));
        }
        if custom.name.trim().is_empty()
            || !seen_names.insert(custom.name.trim().to_ascii_lowercase())
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_DUPLICATE,
                format!("customAnimations[{index}].name"),
                "custom output names must be non-empty and unique",
                "Choose a unique Aurora output name.",
            ));
        }
        match custom.playback {
            CustomAnimationPlaybackV1::OneShot => {
                if custom.phases.is_empty() {
                    validate_clip_reference(
                        custom.clip_reference.as_ref(),
                        &format!("customAnimations[{index}].clipReference"),
                        &authored_by_id,
                        &mut diagnostics,
                    );
                } else {
                    diagnostics.push(diagnostic(
                        DIAGNOSTIC_SCHEMA,
                        format!("customAnimations[{index}].phases"),
                        "ONE_SHOT custom animation cannot contain phased references",
                        "Remove phases or select LOOPING_PHASED playback.",
                    ));
                }
            }
            CustomAnimationPlaybackV1::LoopingPhased => {
                if custom.clip_reference.is_some() {
                    diagnostics.push(diagnostic(
                        DIAGNOSTIC_SCHEMA,
                        format!("customAnimations[{index}].clipReference"),
                        "LOOPING_PHASED custom animation cannot contain a one-shot reference",
                        "Set clipReference to null and use START/LOOP/END phases.",
                    ));
                }
                let mut phases = BTreeSet::new();
                for (phase_index, phase) in custom.phases.iter().enumerate() {
                    let phase_name = format!("{:?}", phase.phase);
                    if !phases.insert(phase_name.clone()) {
                        diagnostics.push(diagnostic(
                            DIAGNOSTIC_SCHEMA,
                            format!("customAnimations[{index}].phases[{phase_index}].phase"),
                            format!("LOOPING_PHASED contains duplicate {phase_name} phase"),
                            "Provide exactly one START, one LOOP, and one END phase.",
                        ));
                    }
                    validate_clip_reference(
                        Some(&phase.clip_reference),
                        &format!("customAnimations[{index}].phases[{phase_index}].clipReference"),
                        &authored_by_id,
                        &mut diagnostics,
                    );
                }
                for required in ["Start", "Loop", "End"] {
                    if !phases.contains(required) {
                        diagnostics.push(diagnostic(
                            DIAGNOSTIC_SCHEMA,
                            format!("customAnimations[{index}].phases"),
                            format!(
                                "LOOPING_PHASED custom animation requires exactly one {} phase",
                                required.to_ascii_uppercase()
                            ),
                            "Provide exactly one START, one LOOP, and one END phase.",
                        ));
                    }
                }
            }
        }
    }
    for (index, assignment) in authoring.assignments.iter().enumerate() {
        if let Some(id) = assignment.custom_animation_id.as_deref()
            && !custom_ids.contains(id)
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("assignments[{index}].customAnimationId"),
                format!("assignment refers to missing custom animation id {id:?}"),
                "Select an existing Custom library definition.",
            ));
        }
    }
    let packaged_authored_ids = authoring
        .custom_animations
        .iter()
        .flat_map(|custom| {
            custom
                .clip_reference
                .iter()
                .chain(custom.phases.iter().map(|phase| &phase.clip_reference))
                .filter_map(|reference| {
                    (reference.source_kind == CustomAnimationClipReferenceKindV2::AuthoredClip)
                        .then_some(reference.authored_clip_id.as_deref())
                        .flatten()
                })
        })
        .collect::<BTreeSet<_>>();
    for clip in &studio.authored_clips {
        if clip.status == AuthoredAnimationClipStatusV1::Valid
            && !packaged_authored_ids.contains(clip.id.as_str())
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("authoredClips[{}]", clip.id),
                "VALID authored clip has no Custom definition and would be omitted from the package",
                "Add the clip to the Custom library or remove it before building.",
            ));
        }
    }
    diagnostics
}

fn validate_clip_reference(
    reference: Option<&CustomAnimationClipReferenceV2>,
    path: &str,
    authored_by_id: &BTreeMap<&str, &AuthoredAnimationClipV1>,
    diagnostics: &mut Vec<AnimationStudioDiagnosticV1>,
) {
    let Some(reference) = reference else {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SCHEMA,
            path,
            "animation clip reference is required",
            "Select a source or authored clip.",
        ));
        return;
    };
    match reference.source_kind {
        CustomAnimationClipReferenceKindV2::SourceClip => {
            if reference
                .source_clip_name
                .as_deref()
                .is_none_or(|name| name.trim().is_empty())
                || reference.authored_clip_id.is_some()
            {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_SCHEMA,
                    path,
                    "SOURCE_CLIP requires sourceClipName and forbids authoredClipId",
                    "Repair the clip reference shape.",
                ));
            }
        }
        CustomAnimationClipReferenceKindV2::AuthoredClip => {
            let valid_shape =
                reference.source_clip_name.is_none() && reference.authored_clip_id.is_some();
            let found = reference
                .authored_clip_id
                .as_deref()
                .and_then(|id| authored_by_id.get(id));
            if !valid_shape || found.is_none() {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_SCHEMA,
                    path,
                    "AUTHORED_CLIP requires an existing authoredClipId and forbids sourceClipName",
                    "Select a valid clip from the Custom library.",
                ));
            } else if found.is_some_and(|clip| clip.status != AuthoredAnimationClipStatusV1::Valid)
            {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_SCHEMA,
                    path,
                    "only VALID authored clips can be mapped or built",
                    "Resolve clip diagnostics and save it as VALID.",
                ));
            }
        }
    }
}

pub fn materialize_authored_animation_library_v1(
    source: &MdlAnimationSetV1,
    studio: &AnimationStudioDocumentV1,
    rig: &AnimationStudioRigV1,
) -> Result<AuthoredAnimationLibraryV1, Vec<AnimationStudioDiagnosticV1>> {
    let materialized = materialize_animation_studio_document_v1(studio, rig)?;
    let mut diagnostics = Vec::new();
    for (index, authored) in studio.authored_clips.iter().enumerate() {
        if authored.source.kind != AuthoredAnimationSourceKindV1::SourceClipCopy {
            continue;
        }
        let source_clip = authored
            .source
            .source_clip_name
            .as_deref()
            .and_then(|name| {
                source
                    .clips
                    .iter()
                    .find(|clip| clip.name.eq_ignore_ascii_case(name))
            });
        let expected_fingerprint = authored.source.source_clip_fingerprint.as_deref();
        if source_clip.is_none()
            || source_clip.map(fingerprint_json).as_deref() != expected_fingerprint
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SOURCE_STALE,
                format!("authoredClips[{index}].source.sourceClipFingerprint"),
                "source clip copy no longer matches its exact source clip fingerprint",
                "Reconcile or recreate this authored clip against the current source GLB.",
            ));
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(AuthoredAnimationLibraryV1 {
        schema_version: 1,
        source_revision: studio.source_revision.clone(),
        clips: studio
            .authored_clips
            .iter()
            .zip(materialized.clips)
            .map(|(authored, clip)| AuthoredAnimationLibraryClipV1 {
                authored_clip_id: authored.id.clone(),
                status: authored.status,
                source: authored.source.clone(),
                clip,
            })
            .collect(),
    })
}

pub fn materialize_custom_animation_definition_v2(
    custom: &CustomAnimationDefinitionV2,
    source: &MdlAnimationSetV1,
    library: &AuthoredAnimationLibraryV1,
) -> Result<Vec<MaterializedCustomAnimationClipV2>, Vec<AnimationStudioDiagnosticV1>> {
    let mut outputs = Vec::new();
    match custom.playback {
        CustomAnimationPlaybackV1::OneShot => {
            let (mut clip, authored_clip_id) = resolve_v2_clip_reference(
                custom.clip_reference.as_ref(),
                source,
                library,
                &format!("customAnimations[{}].clipReference", custom.id),
            )?;
            clip.name = custom.name.clone();
            outputs.push(MaterializedCustomAnimationClipV2 {
                clip,
                authored_clip_id,
                phase: None,
            });
        }
        CustomAnimationPlaybackV1::LoopingPhased => {
            for phase in &custom.phases {
                let (mut clip, authored_clip_id) = resolve_v2_clip_reference(
                    Some(&phase.clip_reference),
                    source,
                    library,
                    &format!(
                        "customAnimations[{}].phases[{:?}].clipReference",
                        custom.id, phase.phase
                    ),
                )?;
                clip.name = custom_phase_output_name_v2(&custom.name, phase.phase);
                outputs.push(MaterializedCustomAnimationClipV2 {
                    clip,
                    authored_clip_id,
                    phase: Some(phase.phase),
                });
            }
        }
    }
    Ok(outputs)
}

pub fn materialize_creature_animation_authoring_v2(
    source: &MdlAnimationSetV1,
    procedural: Option<&MdlAnimationSetV1>,
    authoring: &CreatureAnimationAuthoringV2,
    studio: &AnimationStudioDocumentV1,
    rig: &AnimationStudioRigV1,
) -> Result<MaterializedCreatureAnimationAuthoringV2, Vec<AnimationStudioDiagnosticV1>> {
    let mut diagnostics = validate_creature_animation_authoring_v2(authoring, studio);
    diagnostics.extend(validate_animation_studio_schema_v1(studio));
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let library = materialize_authored_animation_library_v1(source, studio, rig)?;
    let assignments = authoring
        .assignments
        .iter()
        .map(|assignment| (assignment.target_slot.as_str(), assignment))
        .collect::<BTreeMap<_, _>>();
    let fallbacks = authoring
        .fallbacks
        .iter()
        .map(|fallback| (fallback.target_slot.as_str(), fallback))
        .collect::<BTreeMap<_, _>>();
    let custom_by_id = authoring
        .custom_animations
        .iter()
        .map(|custom| (custom.id.as_str(), custom))
        .collect::<BTreeMap<_, _>>();
    let mut clips = Vec::new();
    let mut usages = Vec::new();

    for slot in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 {
        let assignment = match resolve_v2_base_assignment(slot, &assignments, &fallbacks) {
            Ok(assignment) => assignment,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        let resolved = match assignment.source_kind {
            AnimationSourceKindV1::SourceClip => assignment
                .source_clip_name
                .as_deref()
                .and_then(|name| {
                    source
                        .clips
                        .iter()
                        .find(|clip| clip.name.eq_ignore_ascii_case(name))
                })
                .cloned()
                .map(|clip| (clip, None)),
            AnimationSourceKindV1::Procedural => procedural
                .and_then(|set| {
                    set.clips
                        .iter()
                        .find(|clip| clip.name.eq_ignore_ascii_case(slot))
                })
                .cloned()
                .map(|clip| (clip, None)),
            AnimationSourceKindV1::Custom => assignment
                .custom_animation_id
                .as_deref()
                .and_then(|id| custom_by_id.get(id).copied())
                .and_then(primary_custom_reference_v2)
                .and_then(|reference| {
                    resolve_v2_clip_reference(
                        Some(reference),
                        source,
                        &library,
                        &format!("baseSlots.{slot}.customAnimationId"),
                    )
                    .ok()
                }),
            AnimationSourceKindV1::InheritedSupermodel => {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_PATH_UNSUPPORTED,
                    format!("baseSlots.{slot}"),
                    "the local V5 build cannot materialize an inherited supermodel clip",
                    "Choose SOURCE_CLIP, PROCEDURAL, or a VALID Custom animation.",
                ));
                None
            }
        };
        let Some((mut clip, authored_clip_id)) = resolved else {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("baseSlots.{slot}"),
                "Base 42 assignment could not be materialized from its exact source",
                "Repair the assignment or referenced clip.",
            ));
            continue;
        };
        clip.name = slot.to_owned();
        if let Some(authored_clip_id) = authored_clip_id {
            usages.push(AuthoredAnimationUsageV1 {
                authored_clip_id,
                output_clip_name: slot.to_owned(),
                usage_kind: AuthoredAnimationUsageKindV1::BaseSlot,
                base_slot: Some(slot.to_owned()),
                custom_animation_id: assignment.custom_animation_id.clone(),
                phase: None,
            });
        }
        clips.push(clip);
    }

    for custom in &authoring.custom_animations {
        match materialize_custom_animation_definition_v2(custom, source, &library) {
            Ok(custom_clips) => {
                for materialized in custom_clips {
                    if let Some(authored_clip_id) = materialized.authored_clip_id {
                        usages.push(AuthoredAnimationUsageV1 {
                            authored_clip_id,
                            output_clip_name: materialized.clip.name.clone(),
                            usage_kind: if materialized.phase.is_some() {
                                AuthoredAnimationUsageKindV1::CustomPhase
                            } else {
                                AuthoredAnimationUsageKindV1::CustomOneShot
                            },
                            base_slot: None,
                            custom_animation_id: Some(custom.id.clone()),
                            phase: materialized.phase,
                        });
                    }
                    clips.push(materialized.clip);
                }
            }
            Err(errors) => diagnostics.extend(errors),
        }
    }

    let mut output_names = BTreeSet::new();
    for (index, clip) in clips.iter().enumerate() {
        if !output_names.insert(clip.name.to_ascii_lowercase()) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_DUPLICATE,
                format!("materialized.animations.clips[{index}].name"),
                format!(
                    "materialized output clip name {:?} is duplicated",
                    clip.name
                ),
                "Rename the Custom definition so it does not collide with Base 42.",
            ));
        }
    }
    if clips.len() < FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "baseSlots",
            "V5 requires all 42 base slots to materialize",
            "Complete every Base 42 assignment or accepted fallback.",
        ));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    usages.sort_by(|left, right| {
        left.authored_clip_id
            .cmp(&right.authored_clip_id)
            .then_with(|| left.output_clip_name.cmp(&right.output_clip_name))
    });
    Ok(MaterializedCreatureAnimationAuthoringV2 {
        schema_version: 1,
        animations: MdlAnimationSetV1 {
            schema_version: source.schema_version,
            clips,
        },
        authored_usages: usages,
    })
}

fn resolve_v2_base_assignment<'a>(
    slot: &'a str,
    assignments: &BTreeMap<&str, &'a AnimationSourceAssignmentV1>,
    fallbacks: &BTreeMap<&str, &'a AnimationFallbackDecisionV1>,
) -> Result<&'a AnimationSourceAssignmentV1, AnimationStudioDiagnosticV1> {
    let mut current = slot;
    let mut visited = BTreeSet::new();
    for _ in 0..=FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() {
        if let Some(assignment) = assignments.get(current) {
            return Ok(*assignment);
        }
        if !visited.insert(current) {
            return Err(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("baseSlots.{slot}"),
                "fallback cycle prevents Base 42 resolution",
                "Remove one fallback edge from the cycle.",
            ));
        }
        let fallback = fallbacks.get(current).ok_or_else(|| {
            diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("baseSlots.{slot}"),
                "required Base 42 slot has no assignment or accepted fallback",
                "Assign an animation source.",
            )
        })?;
        if fallback.review != AnimationFallbackReviewV1::Accepted {
            return Err(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("fallbacks[{}].review", fallback.id),
                "only ACCEPTED fallbacks can participate in a V5 build",
                "Review and explicitly accept this fallback.",
            ));
        }
        current = fallback.source_slot.as_str();
    }
    Err(diagnostic(
        DIAGNOSTIC_SCHEMA,
        format!("baseSlots.{slot}"),
        "fallback chain exceeds the Base 42 resolution bound",
        "Shorten the fallback chain.",
    ))
}

fn primary_custom_reference_v2(
    custom: &CustomAnimationDefinitionV2,
) -> Option<&CustomAnimationClipReferenceV2> {
    match custom.playback {
        CustomAnimationPlaybackV1::OneShot => custom.clip_reference.as_ref(),
        CustomAnimationPlaybackV1::LoopingPhased => custom
            .phases
            .iter()
            .find(|phase| phase.phase == CustomAnimationPhaseKindV1::Loop)
            .map(|phase| &phase.clip_reference),
    }
}

fn resolve_v2_clip_reference(
    reference: Option<&CustomAnimationClipReferenceV2>,
    source: &MdlAnimationSetV1,
    library: &AuthoredAnimationLibraryV1,
    path: &str,
) -> Result<(MdlAnimationClipV1, Option<String>), Vec<AnimationStudioDiagnosticV1>> {
    let Some(reference) = reference else {
        return Err(vec![diagnostic(
            DIAGNOSTIC_SCHEMA,
            path,
            "animation clip reference is required",
            "Select an exact source or authored clip.",
        )]);
    };
    match reference.source_kind {
        CustomAnimationClipReferenceKindV2::SourceClip => {
            let clip = reference
                .source_clip_name
                .as_deref()
                .and_then(|name| {
                    source
                        .clips
                        .iter()
                        .find(|clip| clip.name.eq_ignore_ascii_case(name))
                })
                .cloned()
                .ok_or_else(|| {
                    vec![diagnostic(
                        DIAGNOSTIC_SCHEMA,
                        path,
                        "referenced source clip is unavailable",
                        "Select an exact source clip from the current GLB.",
                    )]
                })?;
            Ok((clip, None))
        }
        CustomAnimationClipReferenceKindV2::AuthoredClip => {
            let authored_clip_id = reference.authored_clip_id.as_deref().ok_or_else(|| {
                vec![diagnostic(
                    DIAGNOSTIC_SCHEMA,
                    path,
                    "AUTHORED_CLIP reference is missing authoredClipId",
                    "Select a VALID clip from the Custom library.",
                )]
            })?;
            let item = library
                .clips
                .iter()
                .find(|item| item.authored_clip_id == authored_clip_id)
                .ok_or_else(|| {
                    vec![diagnostic(
                        DIAGNOSTIC_SCHEMA,
                        path,
                        "referenced authored clip is unavailable",
                        "Select a VALID clip from the Custom library.",
                    )]
                })?;
            Ok((item.clip.clone(), Some(authored_clip_id.to_owned())))
        }
    }
}

fn custom_phase_output_name_v2(name: &str, phase: CustomAnimationPhaseKindV1) -> String {
    match phase {
        CustomAnimationPhaseKindV1::Start => format!("{name}_s"),
        CustomAnimationPhaseKindV1::Loop => name.to_owned(),
        CustomAnimationPhaseKindV1::End => format!("{name}_e"),
    }
}

pub fn create_blank_pose_clip_v1(
    rig: &AnimationStudioRigV1,
    input: AuthoredAnimationClipInputV1,
) -> Result<AuthoredAnimationClipV1, AnimationStudioDiagnosticV1> {
    validate_rig_and_input(rig, &input)?;
    let mut tracks = Vec::with_capacity(rig.nodes.len() * 2);
    for node in &rig.nodes {
        tracks.push(track_with_values(
            node.node_id,
            AuthoredAnimationTrackPathV1::Translation,
            &[0.0],
            &[node.translation.to_vec()],
        ));
        tracks.push(track_with_values(
            node.node_id,
            AuthoredAnimationTrackPathV1::Rotation,
            &[0.0],
            &[canonical_unit_quaternion(node.rotation.to_vec())?],
        ));
    }
    Ok(AuthoredAnimationClipV1 {
        id: input.id,
        name: input.name,
        kind: AuthoredAnimationClipKindV1::StaticPose,
        status: AuthoredAnimationClipStatusV1::Draft,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::BlankPose,
            source_revision: input.source_revision,
            source_clip_name: None,
            source_clip_fingerprint: None,
            procedural_template: None,
        },
        length_seconds: input.length_seconds,
        transition_seconds: input.transition_seconds,
        animation_root: input.animation_root,
        tracks,
        events: Vec::new(),
        revision: 1,
    })
}

pub fn clone_source_clip_for_editing_v1(
    source: &MdlAnimationSetV1,
    clip_name: &str,
    input: AuthoredAnimationClipInputV1,
) -> Result<AuthoredAnimationClipV1, AnimationStudioDiagnosticV1> {
    validate_input(&input)?;
    let source_clip = source
        .clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            diagnostic(
                DIAGNOSTIC_SCHEMA,
                "sourceClipName",
                format!("source clip {clip_name:?} does not exist"),
                "Choose an existing source animation clip.",
            )
        })?;
    let fingerprint = fingerprint_json(source_clip);
    let mut tracks = Vec::new();
    for source_track in &source_clip.tracks {
        let path = match source_track.path {
            MdlAnimationTrackPathV1::Translation => AuthoredAnimationTrackPathV1::Translation,
            MdlAnimationTrackPathV1::Rotation => AuthoredAnimationTrackPathV1::Rotation,
            MdlAnimationTrackPathV1::Scale
                if source_track.values.first().is_some_and(|first| {
                    (first.len() == 1 || first.len() == 3)
                        && first.iter().all(|value| value.is_finite())
                        && (first.len() == 1
                            || first
                                .iter()
                                .all(|value| (*value - first[0]).abs() <= 1.0e-5))
                        && source_track.values.iter().all(|row| {
                            row.len() == first.len()
                                && row.iter().all(|value| value.is_finite())
                                && (row.len() == 1
                                    || row.iter().all(|value| (*value - row[0]).abs() <= 1.0e-5))
                                && row
                                    .iter()
                                    .zip(first)
                                    .all(|(actual, expected)| (*actual - *expected).abs() <= 1.0e-5)
                        })
                }) =>
            {
                continue;
            }
            MdlAnimationTrackPathV1::Scale | MdlAnimationTrackPathV1::Weights => {
                return Err(diagnostic(
                    DIAGNOSTIC_PATH_UNSUPPORTED,
                    format!("sourceAnimations.{clip_name}.tracks"),
                    "Animation Studio supports TRANSLATION/ROTATION and can only discard a finite constant SCALE track",
                    "Remove changing SCALE or WEIGHTS tracks before cloning for editing.",
                ));
            }
        };
        if source_track.times_seconds.len() != source_track.values.len() {
            return Err(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("sourceAnimations.{clip_name}.tracks"),
                "source track time/value row counts differ",
                "Re-inspect the exact source GLB.",
            ));
        }
        tracks.push(track_with_values(
            source_track.target_node_id,
            path,
            &source_track.times_seconds,
            &source_track.values,
        ));
        tracks.last_mut().expect("just pushed").interpolation = source_track.interpolation;
        if path == AuthoredAnimationTrackPathV1::Rotation {
            normalize_animation_quaternions_v1(tracks.last_mut().expect("just pushed"))?;
        }
    }
    let events = source_clip
        .events
        .iter()
        .enumerate()
        .map(|(index, event)| AuthoredAnimationEventV1 {
            id: format!("event-{index:04}"),
            time_seconds: event.time_seconds,
            name: event.name.clone(),
        })
        .collect();
    let mut clip = AuthoredAnimationClipV1 {
        id: input.id,
        name: input.name,
        kind: AuthoredAnimationClipKindV1::Motion,
        status: AuthoredAnimationClipStatusV1::Draft,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::SourceClipCopy,
            source_revision: input.source_revision,
            source_clip_name: Some(source_clip.name.clone()),
            source_clip_fingerprint: Some(fingerprint),
            procedural_template: None,
        },
        length_seconds: source_clip.length_seconds,
        transition_seconds: input.transition_seconds,
        animation_root: if input.animation_root.trim().is_empty() {
            source_clip.animation_root.clone()
        } else {
            input.animation_root
        },
        tracks,
        events,
        revision: 1,
    };
    sort_authored_animation_events_stable_v1(&mut clip);
    Ok(clip)
}

pub fn create_procedural_template_clip_v1(
    rig: &AnimationStudioRigV1,
    template: ProceduralAnimationTemplateV1,
    input: AuthoredAnimationClipInputV1,
) -> Result<AuthoredAnimationClipV1, AnimationStudioDiagnosticV1> {
    let mut clip = create_blank_pose_clip_v1(rig, input)?;
    clip.source.kind = AuthoredAnimationSourceKindV1::ProceduralTemplate;
    clip.source.procedural_template = Some(
        match template {
            ProceduralAnimationTemplateV1::BindPose => "BIND_POSE",
            ProceduralAnimationTemplateV1::RootTranslationPulse => "ROOT_TRANSLATION_PULSE",
        }
        .to_owned(),
    );
    if template == ProceduralAnimationTemplateV1::RootTranslationPulse {
        clip.kind = AuthoredAnimationClipKindV1::Motion;
        let root_id = rig
            .nodes
            .iter()
            .find(|node| node.name == rig.animation_root)
            .map(|node| node.node_id)
            .or_else(|| rig.nodes.first().map(|node| node.node_id))
            .ok_or_else(|| {
                diagnostic(
                    DIAGNOSTIC_BONE_MISSING,
                    "rig.nodes",
                    "procedural template requires at least one rig node",
                    "Inspect a rig with an animation root node.",
                )
            })?;
        let track = clip
            .tracks
            .iter_mut()
            .find(|track| {
                track.target_node_id == root_id
                    && track.path == AuthoredAnimationTrackPathV1::Translation
            })
            .expect("blank pose creates the root translation track");
        let base = track.keyframes[0].value.clone();
        track.keyframes = vec![
            AnimationKeyframeV1 {
                id: "key-0000".to_owned(),
                time_seconds: 0.0,
                value: base.clone(),
            },
            AnimationKeyframeV1 {
                id: "key-0001".to_owned(),
                time_seconds: clip.length_seconds * 0.5,
                value: vec![base[0] + 0.1, base[1], base[2]],
            },
            AnimationKeyframeV1 {
                id: "key-0002".to_owned(),
                time_seconds: clip.length_seconds,
                value: base,
            },
        ];
    }
    Ok(clip)
}

pub fn rename_authored_clip_v1(
    document: &mut AnimationStudioDocumentV1,
    clip_id: &str,
    name: &str,
) -> Result<(), AnimationStudioDiagnosticV1> {
    validate_output_name(name)?;
    if document
        .authored_clips
        .iter()
        .any(|clip| clip.id != clip_id && clip.name.trim().eq_ignore_ascii_case(name.trim()))
    {
        return Err(diagnostic(
            DIAGNOSTIC_CLIP_DUPLICATE,
            format!("authoredClips[{clip_id}].name"),
            format!("output name {name:?} is already used"),
            "Choose a unique output name.",
        ));
    }
    let clip = find_clip_mut(document, clip_id)?;
    clip.name = name.trim().to_owned();
    touch_clip(clip);
    touch_document(document);
    Ok(())
}

pub fn duplicate_authored_clip_v1(
    document: &mut AnimationStudioDocumentV1,
    clip_id: &str,
    new_name: &str,
) -> Result<String, AnimationStudioDiagnosticV1> {
    validate_output_name(new_name)?;
    if document
        .authored_clips
        .iter()
        .any(|clip| clip.name.trim().eq_ignore_ascii_case(new_name.trim()))
    {
        return Err(diagnostic(
            DIAGNOSTIC_CLIP_DUPLICATE,
            "authoredClips",
            format!("output name {new_name:?} is already used"),
            "Choose a unique output name.",
        ));
    }
    let source = document
        .authored_clips
        .iter()
        .find(|clip| clip.id == clip_id)
        .cloned()
        .ok_or_else(|| clip_missing(clip_id))?;
    let base = format!("{}_copy", source.id);
    let mut id = base.clone();
    let mut suffix = 2_u32;
    while document.authored_clips.iter().any(|clip| clip.id == id) {
        id = format!("{base}_{suffix}");
        suffix += 1;
    }
    let mut copy = source;
    copy.id = id.clone();
    copy.name = new_name.trim().to_owned();
    copy.status = AuthoredAnimationClipStatusV1::Draft;
    touch_clip(&mut copy);
    document.authored_clips.push(copy);
    touch_document(document);
    Ok(id)
}

pub fn remove_authored_clip_v1(
    document: &mut AnimationStudioDocumentV1,
    clip_id: &str,
    _policy: RemoveAuthoredClipPolicyV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    let index = document
        .authored_clips
        .iter()
        .position(|clip| clip.id == clip_id)
        .ok_or_else(|| clip_missing(clip_id))?;
    document.authored_clips.remove(index);
    touch_document(document);
    Ok(())
}

pub fn add_animation_track_v1(
    clip: &mut AuthoredAnimationClipV1,
    target_node_id: u32,
    path: AuthoredAnimationTrackPathV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if clip
        .tracks
        .iter()
        .any(|track| track.target_node_id == target_node_id && track.path == path)
    {
        return Err(diagnostic(
            DIAGNOSTIC_CLIP_DUPLICATE,
            format!("authoredClips[{}].tracks", clip.id),
            "a clip can contain only one track for each node/path pair",
            "Edit the existing track.",
        ));
    }
    clip.tracks.push(AuthoredAnimationTrackV1 {
        id: track_id(target_node_id, path),
        target_node_id,
        path,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: Vec::new(),
    });
    touch_clip(clip);
    Ok(())
}

pub fn remove_animation_track_v1(
    clip: &mut AuthoredAnimationClipV1,
    target_node_id: u32,
    path: AuthoredAnimationTrackPathV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    let index = clip
        .tracks
        .iter()
        .position(|track| track.target_node_id == target_node_id && track.path == path)
        .ok_or_else(|| {
            diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("authoredClips[{}].tracks", clip.id),
                "animation track does not exist",
                "Select an existing track.",
            )
        })?;
    clip.tracks.remove(index);
    touch_clip(clip);
    Ok(())
}

pub fn insert_animation_keyframe_v1(
    track: &mut AuthoredAnimationTrackV1,
    time_seconds: f32,
    value: Vec<f32>,
) -> Result<String, AnimationStudioDiagnosticV1> {
    validate_key_value(track.path, time_seconds, &value, &track.id)?;
    if track
        .keyframes
        .iter()
        .any(|key| same_time(key.time_seconds, time_seconds))
    {
        return Err(diagnostic(
            DIAGNOSTIC_TIME_NOT_STRICT,
            format!("tracks[{}].keyframes", track.id),
            format!("a keyframe already exists at {time_seconds} seconds"),
            "Move the existing key or use an explicit replace policy.",
        ));
    }
    let id = next_prefixed_id("key", track.keyframes.iter().map(|key| key.id.as_str()));
    track.keyframes.push(AnimationKeyframeV1 {
        id: id.clone(),
        time_seconds,
        value,
    });
    sort_keyframes(track);
    Ok(id)
}

pub fn update_animation_keyframe_v1(
    track: &mut AuthoredAnimationTrackV1,
    key_id: &str,
    patch: AnimationKeyframePatchV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    let index = key_index(track, key_id)?;
    let time = patch
        .time_seconds
        .unwrap_or(track.keyframes[index].time_seconds);
    let value = patch
        .value
        .unwrap_or_else(|| track.keyframes[index].value.clone());
    validate_key_value(track.path, time, &value, &track.id)?;
    if track
        .keyframes
        .iter()
        .enumerate()
        .any(|(other, key)| other != index && same_time(key.time_seconds, time))
    {
        return Err(diagnostic(
            DIAGNOSTIC_TIME_NOT_STRICT,
            format!("tracks[{}].keyframes[{key_id}].timeSeconds", track.id),
            "keyframe times must remain strictly increasing",
            "Choose a time not occupied by another keyframe.",
        ));
    }
    track.keyframes[index].time_seconds = time;
    track.keyframes[index].value = value;
    sort_keyframes(track);
    Ok(())
}

pub fn move_animation_keyframe_v1(
    track: &mut AuthoredAnimationTrackV1,
    key_id: &str,
    time_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    update_animation_keyframe_v1(
        track,
        key_id,
        AnimationKeyframePatchV1 {
            time_seconds: Some(time_seconds),
            value: None,
        },
    )
}

pub fn remove_animation_keyframe_v1(
    track: &mut AuthoredAnimationTrackV1,
    key_id: &str,
) -> Result<(), AnimationStudioDiagnosticV1> {
    let index = key_index(track, key_id)?;
    track.keyframes.remove(index);
    Ok(())
}

pub fn sample_animation_track_linear_v1(
    track: &AuthoredAnimationTrackV1,
    time_seconds: f32,
) -> Result<Vec<f32>, AnimationStudioDiagnosticV1> {
    if !time_seconds.is_finite() {
        return Err(nonfinite_time(&format!("tracks[{}]", track.id)));
    }
    if track.keyframes.is_empty() {
        return Err(diagnostic(
            DIAGNOSTIC_SCHEMA,
            format!("tracks[{}].keyframes", track.id),
            "cannot sample an empty animation track",
            "Insert at least one keyframe.",
        ));
    }
    let mut sorted = track.keyframes.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    if time_seconds <= sorted[0].time_seconds {
        return Ok(sorted[0].value.clone());
    }
    if time_seconds >= sorted[sorted.len() - 1].time_seconds {
        return Ok(sorted[sorted.len() - 1].value.clone());
    }
    for pair in sorted.windows(2) {
        let left = pair[0];
        let right = pair[1];
        if time_seconds >= left.time_seconds && time_seconds <= right.time_seconds {
            let span = right.time_seconds - left.time_seconds;
            if span <= 0.0 {
                return Err(diagnostic(
                    DIAGNOSTIC_TIME_NOT_STRICT,
                    format!("tracks[{}].keyframes", track.id),
                    "track contains equal or descending keyframe times",
                    "Sort and deduplicate keyframes explicitly.",
                ));
            }
            if track.interpolation == MdlAnimationInterpolationV1::Step {
                return Ok(left.value.clone());
            }
            if track.interpolation == MdlAnimationInterpolationV1::CubicSpline {
                return Err(diagnostic(
                    DIAGNOSTIC_PATH_UNSUPPORTED,
                    format!("tracks[{}].interpolation", track.id),
                    "Animation Studio authoring does not support CUBIC_SPLINE",
                    "Resample the source track to LINEAR or STEP.",
                ));
            }
            let alpha = (time_seconds - left.time_seconds) / span;
            let value = left
                .value
                .iter()
                .zip(&right.value)
                .map(|(a, b)| a + (b - a) * alpha)
                .collect::<Vec<_>>();
            return if track.path == AuthoredAnimationTrackPathV1::Rotation {
                canonical_unit_quaternion(value)
            } else {
                Ok(value)
            };
        }
    }
    unreachable!("time is bounded by first and last keyframes")
}

pub fn normalize_animation_quaternions_v1(
    track: &mut AuthoredAnimationTrackV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if track.path != AuthoredAnimationTrackPathV1::Rotation {
        return Err(diagnostic(
            DIAGNOSTIC_PATH_UNSUPPORTED,
            format!("tracks[{}].path", track.id),
            "quaternion normalization applies only to ROTATION tracks",
            "Select a rotation track.",
        ));
    }
    for key in &mut track.keyframes {
        key.value = canonical_unit_quaternion(key.value.clone())?;
    }
    Ok(())
}

pub fn sort_and_deduplicate_keyframes_v1(
    track: &mut AuthoredAnimationTrackV1,
    policy: KeyframeDeduplicationPolicyV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    for key in &track.keyframes {
        validate_key_value(track.path, key.time_seconds, &key.value, &track.id)?;
    }
    let mut input = track.keyframes.clone();
    input.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut output: Vec<AnimationKeyframeV1> = Vec::with_capacity(input.len());
    for key in input {
        if output
            .last()
            .is_some_and(|previous| same_time(previous.time_seconds, key.time_seconds))
        {
            match policy {
                KeyframeDeduplicationPolicyV1::Fail => {
                    return Err(diagnostic(
                        DIAGNOSTIC_TIME_NOT_STRICT,
                        format!("tracks[{}].keyframes", track.id),
                        "track contains equal keyframe times",
                        "Choose the explicit REPLACE_LAST policy or move one key.",
                    ));
                }
                KeyframeDeduplicationPolicyV1::ReplaceLast => {
                    *output.last_mut().expect("checked") = key;
                }
            }
        } else {
            output.push(key);
        }
    }
    track.keyframes = output;
    Ok(())
}

pub fn detect_authored_motion_v1(clip: &AuthoredAnimationClipV1) -> bool {
    clip.tracks.iter().any(|track| {
        let Some(first) = track.keyframes.first() else {
            return false;
        };
        track.keyframes.iter().skip(1).any(|key| {
            key.value.len() != first.value.len()
                || key
                    .value
                    .iter()
                    .zip(&first.value)
                    .any(|(left, right)| (left - right).abs() > 1.0e-6)
        })
    })
}

pub fn trim_authored_animation_clip_v1(
    clip: &mut AuthoredAnimationClipV1,
    start_seconds: f32,
    end_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if !start_seconds.is_finite()
        || !end_seconds.is_finite()
        || start_seconds < 0.0
        || end_seconds > clip.length_seconds
        || end_seconds <= start_seconds
    {
        return Err(diagnostic(
            DIAGNOSTIC_TIME_OOB,
            format!("authoredClips[{}].lengthSeconds", clip.id),
            "trim range must be finite, ordered, and inside the clip",
            "Choose 0 <= start < end <= clip length.",
        ));
    }
    let new_length = end_seconds - start_seconds;
    for track in &mut clip.tracks {
        if track.keyframes.is_empty() {
            continue;
        }
        let start_value = sample_animation_track_linear_v1(track, start_seconds)?;
        let end_value = sample_animation_track_linear_v1(track, end_seconds)?;
        let start_id = track
            .keyframes
            .iter()
            .find(|key| same_time(key.time_seconds, start_seconds))
            .map(|key| key.id.clone())
            .unwrap_or_else(|| "trim-start".to_owned());
        let end_id = track
            .keyframes
            .iter()
            .find(|key| same_time(key.time_seconds, end_seconds))
            .map(|key| key.id.clone())
            .unwrap_or_else(|| "trim-end".to_owned());
        let mut keys = vec![AnimationKeyframeV1 {
            id: start_id,
            time_seconds: 0.0,
            value: start_value,
        }];
        keys.extend(
            track
                .keyframes
                .iter()
                .filter(|key| key.time_seconds > start_seconds && key.time_seconds < end_seconds)
                .cloned()
                .map(|mut key| {
                    key.time_seconds -= start_seconds;
                    key
                }),
        );
        if new_length > 0.0 {
            keys.push(AnimationKeyframeV1 {
                id: end_id,
                time_seconds: new_length,
                value: end_value,
            });
        }
        track.keyframes = keys;
        sort_and_deduplicate_keyframes_v1(track, KeyframeDeduplicationPolicyV1::ReplaceLast)?;
    }
    clip.events
        .retain(|event| event.time_seconds >= start_seconds && event.time_seconds <= end_seconds);
    for event in &mut clip.events {
        event.time_seconds -= start_seconds;
    }
    clip.length_seconds = new_length;
    sort_authored_animation_events_stable_v1(clip);
    touch_clip(clip);
    Ok(())
}

pub fn retime_authored_animation_clip_v1(
    clip: &mut AuthoredAnimationClipV1,
    new_length_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if !new_length_seconds.is_finite()
        || new_length_seconds <= 0.0
        || new_length_seconds > ANIMATION_STUDIO_MAX_DURATION_SECONDS
        || clip.length_seconds <= 0.0
    {
        return Err(diagnostic(
            DIAGNOSTIC_TIME_OOB,
            format!("authoredClips[{}].lengthSeconds", clip.id),
            format!(
                "retime length must be within 0..={} seconds",
                ANIMATION_STUDIO_MAX_DURATION_SECONDS
            ),
            "Choose a positive duration within the Animation Studio product limit.",
        ));
    }
    let ratio = new_length_seconds / clip.length_seconds;
    for track in &mut clip.tracks {
        for key in &mut track.keyframes {
            key.time_seconds *= ratio;
        }
    }
    for event in &mut clip.events {
        event.time_seconds *= ratio;
    }
    clip.length_seconds = new_length_seconds;
    touch_clip(clip);
    Ok(())
}

pub fn shift_authored_animation_keys_v1(
    clip: &mut AuthoredAnimationClipV1,
    delta_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if !delta_seconds.is_finite() {
        return Err(nonfinite_time(&format!("authoredClips[{}]", clip.id)));
    }
    for track in &mut clip.tracks {
        for key in &mut track.keyframes {
            key.time_seconds += delta_seconds;
        }
    }
    for event in &mut clip.events {
        event.time_seconds += delta_seconds;
    }
    touch_clip(clip);
    Ok(())
}

pub fn clamp_authored_animation_keys_v1(clip: &mut AuthoredAnimationClipV1) {
    for track in &mut clip.tracks {
        for key in &mut track.keyframes {
            key.time_seconds = key.time_seconds.clamp(0.0, clip.length_seconds);
        }
        let _ =
            sort_and_deduplicate_keyframes_v1(track, KeyframeDeduplicationPolicyV1::ReplaceLast);
    }
    for event in &mut clip.events {
        event.time_seconds = event.time_seconds.clamp(0.0, clip.length_seconds);
    }
    sort_authored_animation_events_stable_v1(clip);
    touch_clip(clip);
}

pub fn set_authored_animation_transition_v1(
    clip: &mut AuthoredAnimationClipV1,
    transition_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if !transition_seconds.is_finite()
        || transition_seconds < 0.0
        || transition_seconds > clip.length_seconds
    {
        return Err(diagnostic(
            DIAGNOSTIC_TIME_OOB,
            format!("authoredClips[{}].transitionSeconds", clip.id),
            "transition must be finite and within clip duration",
            "Choose 0 <= transition <= clip length.",
        ));
    }
    clip.transition_seconds = transition_seconds;
    touch_clip(clip);
    Ok(())
}

pub fn add_authored_animation_event_v1(
    clip: &mut AuthoredAnimationClipV1,
    event: AuthoredAnimationEventV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if clip.events.iter().any(|existing| existing.id == event.id) {
        return Err(diagnostic(
            DIAGNOSTIC_EVENT,
            format!("authoredClips[{}].events", clip.id),
            format!("event id {:?} is duplicated", event.id),
            "Choose a stable unique event id.",
        ));
    }
    validate_authored_event_v1(&event, clip.length_seconds)?;
    clip.events.push(event);
    sort_authored_animation_events_stable_v1(clip);
    touch_clip(clip);
    Ok(())
}

pub fn update_authored_animation_event_v1(
    clip: &mut AuthoredAnimationClipV1,
    event_id: &str,
    patch: AuthoredAnimationEventPatchV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    let index = event_index(clip, event_id)?;
    let mut event = clip.events[index].clone();
    if let Some(time) = patch.time_seconds {
        event.time_seconds = time;
    }
    if let Some(name) = patch.name {
        event.name = name;
    }
    validate_authored_event_v1(&event, clip.length_seconds)?;
    clip.events[index] = event;
    sort_authored_animation_events_stable_v1(clip);
    touch_clip(clip);
    Ok(())
}

pub fn move_authored_animation_event_v1(
    clip: &mut AuthoredAnimationClipV1,
    event_id: &str,
    time_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    update_authored_animation_event_v1(
        clip,
        event_id,
        AuthoredAnimationEventPatchV1 {
            time_seconds: Some(time_seconds),
            name: None,
        },
    )
}

pub fn remove_authored_animation_event_v1(
    clip: &mut AuthoredAnimationClipV1,
    event_id: &str,
) -> Result<(), AnimationStudioDiagnosticV1> {
    let index = event_index(clip, event_id)?;
    clip.events.remove(index);
    touch_clip(clip);
    Ok(())
}

pub fn sort_authored_animation_events_stable_v1(clip: &mut AuthoredAnimationClipV1) {
    clip.events
        .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
}

pub fn validate_authored_animation_clip_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
) -> Vec<AnimationStudioDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if let Err(error) = validate_output_name(&clip.name) {
        diagnostics.push(error);
    }
    if !clip.length_seconds.is_finite()
        || clip.length_seconds <= 0.0
        || clip.length_seconds > ANIMATION_STUDIO_MAX_DURATION_SECONDS
    {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_TIME_OOB,
            format!("authoredClips[{}].lengthSeconds", clip.id),
            format!(
                "clip length must be within 0..={} seconds",
                ANIMATION_STUDIO_MAX_DURATION_SECONDS
            ),
            "Choose a positive duration within the Animation Studio product limit.",
        ));
    }
    if !clip.transition_seconds.is_finite()
        || clip.transition_seconds < 0.0
        || clip.transition_seconds > clip.length_seconds
    {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_TIME_OOB,
            format!("authoredClips[{}].transitionSeconds", clip.id),
            "transition must be finite and within clip duration",
            "Choose 0 <= transition <= clip length.",
        ));
    }
    diagnostics.extend(validate_authored_source_shape(clip));
    let mut track_pairs = BTreeSet::new();
    let mut track_ids = BTreeSet::new();
    let row_count = clip
        .tracks
        .iter()
        .map(|track| track.keyframes.len())
        .sum::<usize>();
    if row_count > ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_ROW_LIMIT,
            format!("authoredClips[{}].tracks", clip.id),
            format!(
                "clip contains {row_count} keyframes; product limit is {}",
                ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP
            ),
            "Reduce or resample animation keys before build.",
        ));
    }
    if row_count > ANIMATION_STUDIO_MDL_ROW_LIMIT {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_ROW_LIMIT,
            format!("authoredClips[{}].tracks", clip.id),
            format!(
                "clip contains {row_count} authored rows; limit is {ANIMATION_STUDIO_MDL_ROW_LIMIT}"
            ),
            "Reduce or resample animation keys before build.",
        ));
    }
    for track in &clip.tracks {
        if !track_ids.insert(track.id.as_str())
            || !track_pairs.insert((track.target_node_id, track.path))
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_CLIP_DUPLICATE,
                format!("authoredClips[{}].tracks[{}]", clip.id, track.id),
                "track ids and node/path pairs must be unique",
                "Merge or remove the duplicate track.",
            ));
        }
        diagnostics.extend(
            validate_authored_track_target_v1(track, rig)
                .into_iter()
                .map(|mut diagnostic| {
                    if diagnostic.path.starts_with("authoredClips[unknown]") {
                        diagnostic.path = diagnostic.path.replacen("unknown", &clip.id, 1);
                    }
                    diagnostic
                }),
        );
        diagnostics.extend(validate_authored_track_times_v1(track, clip.length_seconds));
        diagnostics.extend(validate_authored_track_values_v1(track));
        if track.interpolation != MdlAnimationInterpolationV1::Linear {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_PATH_UNSUPPORTED,
                format!(
                    "authoredClips[{}].tracks[{}].interpolation",
                    clip.id, track.id
                ),
                "Animation Studio MVP authoring supports LINEAR interpolation only",
                "Resample the track to LINEAR.",
            ));
        }
    }
    let mut event_ids = BTreeSet::new();
    for event in &clip.events {
        if !event_ids.insert(event.id.as_str()) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_EVENT,
                format!("authoredClips[{}].events[{}].id", clip.id, event.id),
                "event ids must be unique inside a clip",
                "Choose a stable unique event id.",
            ));
        }
        if let Err(error) = validate_authored_event_v1(event, clip.length_seconds) {
            diagnostics.push(error);
        }
    }
    match clip.kind {
        AuthoredAnimationClipKindV1::Motion if !detect_authored_motion_v1(clip) => {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_NO_MOTION,
                format!("authoredClips[{}].kind", clip.id),
                "MOTION clip requires at least one changing track",
                "Edit key values or classify the clip explicitly as STATIC_POSE.",
            ));
        }
        AuthoredAnimationClipKindV1::StaticPose if detect_authored_motion_v1(clip) => {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_NO_MOTION,
                format!("authoredClips[{}].kind", clip.id),
                "STATIC_POSE cannot contain a changing track",
                "Remove motion or classify the clip explicitly as MOTION.",
            ));
        }
        _ => {}
    }
    diagnostics
}

fn validate_authored_source_shape(
    clip: &AuthoredAnimationClipV1,
) -> Vec<AnimationStudioDiagnosticV1> {
    let source = &clip.source;
    if !is_sha256(&source.source_revision) {
        return vec![diagnostic(
            DIAGNOSTIC_SCHEMA,
            format!("authoredClips[{}].source.sourceRevision", clip.id),
            "authored source revision must be the SHA-256 of the exact source GLB",
            "Reconcile the clip against the exact source asset.",
        )];
    }
    let valid = match source.kind {
        AuthoredAnimationSourceKindV1::BlankPose => {
            source.source_clip_name.is_none()
                && source.source_clip_fingerprint.is_none()
                && source.procedural_template.is_none()
        }
        AuthoredAnimationSourceKindV1::SourceClipCopy => {
            source
                .source_clip_name
                .as_deref()
                .is_some_and(|name| !name.trim().is_empty())
                && source
                    .source_clip_fingerprint
                    .as_deref()
                    .is_some_and(is_sha256)
                && source.procedural_template.is_none()
        }
        AuthoredAnimationSourceKindV1::ProceduralTemplate => {
            source.source_clip_name.is_none()
                && source.source_clip_fingerprint.is_none()
                && source
                    .procedural_template
                    .as_deref()
                    .is_some_and(|template| !template.trim().is_empty())
        }
    };
    if valid {
        Vec::new()
    } else {
        vec![diagnostic(
            DIAGNOSTIC_SCHEMA,
            format!("authoredClips[{}].source", clip.id),
            "authored source fields do not match its source kind",
            "Repair the source provenance shape without storing paths or GLB bytes.",
        )]
    }
}

pub fn validate_authored_track_target_v1(
    track: &AuthoredAnimationTrackV1,
    rig: &AnimationStudioRigV1,
) -> Vec<AnimationStudioDiagnosticV1> {
    if rig
        .nodes
        .iter()
        .any(|node| node.node_id == track.target_node_id)
    {
        Vec::new()
    } else {
        vec![diagnostic(
            DIAGNOSTIC_BONE_MISSING,
            format!("authoredClips[unknown].tracks[{}].targetNodeId", track.id),
            format!(
                "target node {} does not exist in the rig",
                track.target_node_id
            ),
            "Select a node from the current exact source rig.",
        )]
    }
}

pub fn validate_authored_track_times_v1(
    track: &AuthoredAnimationTrackV1,
    clip_length_seconds: f32,
) -> Vec<AnimationStudioDiagnosticV1> {
    let mut diagnostics = Vec::new();
    let mut previous = None;
    for (index, key) in track.keyframes.iter().enumerate() {
        if !key.time_seconds.is_finite()
            || key.time_seconds < 0.0
            || key.time_seconds > clip_length_seconds
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_TIME_OOB,
                format!("tracks[{}].keyframes[{index}].timeSeconds", track.id),
                format!(
                    "key time {} is outside 0..={clip_length_seconds}",
                    key.time_seconds
                ),
                "Move or clamp the key into the clip range.",
            ));
        }
        if previous.is_some_and(|time| key.time_seconds <= time) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_TIME_NOT_STRICT,
                format!("tracks[{}].keyframes[{index}].timeSeconds", track.id),
                "keyframe times must be strictly increasing",
                "Sort keys and resolve equal times explicitly.",
            ));
        }
        previous = Some(key.time_seconds);
    }
    diagnostics
}

pub fn validate_authored_track_values_v1(
    track: &AuthoredAnimationTrackV1,
) -> Vec<AnimationStudioDiagnosticV1> {
    let mut diagnostics = Vec::new();
    for (index, key) in track.keyframes.iter().enumerate() {
        if key.value.len() != track.path.expected_value_count() {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_PATH_UNSUPPORTED,
                format!("tracks[{}].keyframes[{index}].value", track.id),
                format!(
                    "{:?} requires {} values, got {}",
                    track.path,
                    track.path.expected_value_count(),
                    key.value.len()
                ),
                "Provide a value with the correct dimension.",
            ));
            continue;
        }
        if key.value.iter().any(|value| {
            !value.is_finite() || value.abs() > ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE
        }) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_VALUE_NONFINITE,
                format!("tracks[{}].keyframes[{index}].value", track.id),
                format!(
                    "keyframe values must be finite and within +/-{}",
                    ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE
                ),
                "Replace the transform with values inside the Animation Studio product range.",
            ));
        }
        if track.path == AuthoredAnimationTrackPathV1::Rotation {
            let norm_squared = key.value.iter().map(|value| value * value).sum::<f32>();
            if !norm_squared.is_finite() || norm_squared <= 1.0e-12 {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_QUATERNION,
                    format!("tracks[{}].keyframes[{index}].value", track.id),
                    "rotation quaternion must have a finite non-zero norm",
                    "Normalize or replace the rotation quaternion.",
                ));
            }
        }
    }
    diagnostics
}

pub fn validate_authored_event_v1(
    event: &AuthoredAnimationEventV1,
    clip_length_seconds: f32,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if event.id.trim().is_empty()
        || event.name.trim().is_empty()
        || !event.time_seconds.is_finite()
        || event.time_seconds < 0.0
        || event.time_seconds > clip_length_seconds
    {
        return Err(diagnostic(
            DIAGNOSTIC_EVENT,
            format!("events[{}]", event.id),
            "event requires non-empty id/name and a finite time inside the clip",
            "Repair the event id, name, or time.",
        ));
    }
    if !event.name.is_ascii() || event.name.len() > ANIMATION_STUDIO_EVENT_NAME_MAX_ASCII_BYTES {
        return Err(diagnostic(
            DIAGNOSTIC_EVENT,
            format!("events[{}].name", event.id),
            format!(
                "event name must be ASCII and at most {} bytes",
                ANIMATION_STUDIO_EVENT_NAME_MAX_ASCII_BYTES
            ),
            "Use a portable Aurora event name containing at most 31 ASCII bytes.",
        ));
    }
    Ok(())
}

pub fn evaluate_authored_clip_status_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
) -> AuthoredAnimationClipStatusV1 {
    if validate_authored_animation_clip_v1(clip, rig).is_empty() {
        AuthoredAnimationClipStatusV1::Valid
    } else {
        AuthoredAnimationClipStatusV1::Invalid
    }
}

pub fn apply_integrated_authored_events_v1(
    animation_set: &MdlAnimationSetV1,
    studio: &AnimationStudioDocumentV1,
) -> Result<MdlAnimationSetV1, Vec<AnimationStudioDiagnosticV1>> {
    let mut output = animation_set.clone();
    let mut diagnostics = Vec::new();
    for authored in &studio.authored_clips {
        for event in &authored.events {
            if let Err(error) = validate_authored_event_v1(event, authored.length_seconds) {
                diagnostics.push(error);
            }
        }
        let Some(output_clip) = output
            .clips
            .iter_mut()
            .find(|clip| clip.name.eq_ignore_ascii_case(&authored.name))
        else {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("authoredClips[{}].events", authored.id),
                "authored clip is missing from the materialized animation set",
                "Materialize the exact Studio document before applying integrated events.",
            ));
            continue;
        };
        output_clip.events = authored
            .events
            .iter()
            .map(|event| MdlAnimationEventV1 {
                time_seconds: event.time_seconds,
                name: event.name.clone(),
            })
            .collect();
        output_clip
            .events
            .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    }
    if diagnostics.is_empty() {
        Ok(output)
    } else {
        Err(diagnostics)
    }
}

pub fn materialize_animation_studio_document_v1(
    document: &AnimationStudioDocumentV1,
    rig: &AnimationStudioRigV1,
) -> Result<MdlAnimationSetV1, Vec<AnimationStudioDiagnosticV1>> {
    let mut diagnostics = validate_animation_studio_schema_v1(document);
    if rig.schema_version != ANIMATION_STUDIO_RIG_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "rig.schemaVersion",
            "Animation Studio rig schema version must be 1",
            "Re-inspect the exact source rig.",
        ));
    }
    if rig.source_revision != document.source_revision {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_SOURCE_STALE,
            "rig.sourceRevision",
            "rig and Studio document source revisions differ",
            "Reconcile the document against the exact current source GLB.",
        ));
    }
    for clip in &document.authored_clips {
        diagnostics.extend(
            validate_authored_animation_clip_v1(clip, rig)
                .into_iter()
                .map(|mut diagnostic| {
                    if diagnostic.path.starts_with("authoredClips[unknown]") {
                        diagnostic.path = diagnostic.path.replacen("unknown", &clip.id, 1);
                    }
                    diagnostic
                }),
        );
        if clip.status != AuthoredAnimationClipStatusV1::Valid {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("authoredClips[{}].status", clip.id),
                "Draft or invalid authored clips cannot be materialized",
                "Resolve diagnostics and explicitly save the clip as VALID.",
            ));
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let materialized = MdlAnimationSetV1 {
        schema_version: 1,
        clips: document
            .authored_clips
            .iter()
            .map(|clip| MdlAnimationClipV1 {
                name: clip.name.clone(),
                animation_root: clip.animation_root.clone(),
                length_seconds: clip.length_seconds,
                transition_seconds: clip.transition_seconds,
                events: Vec::new(),
                tracks: clip
                    .tracks
                    .iter()
                    .map(|track| MdlAnimationTrackV1 {
                        target_node_id: track.target_node_id,
                        path: match track.path {
                            AuthoredAnimationTrackPathV1::Translation => {
                                MdlAnimationTrackPathV1::Translation
                            }
                            AuthoredAnimationTrackPathV1::Rotation => {
                                MdlAnimationTrackPathV1::Rotation
                            }
                        },
                        interpolation: track.interpolation,
                        times_seconds: track.keyframes.iter().map(|key| key.time_seconds).collect(),
                        values: track
                            .keyframes
                            .iter()
                            .map(|key| key.value.clone())
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    apply_integrated_authored_events_v1(&materialized, document)
}

pub fn reconcile_animation_studio_readback_v1(
    studio: &AnimationStudioDocumentV1,
    expected: &MaterializedCreatureAnimationAuthoringV2,
    rig: &AnimationStudioRigV1,
    readback: &InspectionReport,
) -> AnimationStudioReadbackV1 {
    let mut diagnostics = Vec::new();
    let mut clips = Vec::new();
    for authored in &studio.authored_clips {
        if authored.status == AuthoredAnimationClipStatusV1::Valid
            && !expected
                .authored_usages
                .iter()
                .any(|usage| usage.authored_clip_id == authored.id)
        {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_READBACK_MISMATCH,
                format!("readback.authoredClips.{}", authored.id),
                "VALID authored clip has no expected packaged output usage",
                "Block Download and add the clip to the Custom library.",
            ));
        }
    }
    for usage in &expected.authored_usages {
        let expected_clip = expected
            .animations
            .clips
            .iter()
            .find(|clip| clip.name == usage.output_clip_name);
        let actual_clip = readback
            .animations
            .iter()
            .find(|clip| clip.name == usage.output_clip_name);
        match (expected_clip, actual_clip) {
            (Some(expected_clip), Some(actual_clip)) => {
                reconcile_clip(expected_clip, actual_clip, rig, &mut diagnostics);
                clips.push(AnimationStudioReadbackClipV1 {
                    authored_clip_id: usage.authored_clip_id.clone(),
                    output_clip_name: usage.output_clip_name.clone(),
                    materialized_fingerprint: fingerprint_json(actual_clip),
                });
            }
            _ => diagnostics.push(diagnostic(
                DIAGNOSTIC_READBACK_MISMATCH,
                format!("readback.animations.{}", usage.output_clip_name),
                "authored output clip is missing from expected materialization or binary readback",
                "Block Download and rebuild from the same exact document and source GLB.",
            )),
        }
    }
    AnimationStudioReadbackV1 {
        schema_version: ANIMATION_STUDIO_READBACK_SCHEMA_VERSION,
        studio_fingerprint: fingerprint_animation_studio_document_v1(studio),
        source_revision: studio.source_revision.clone(),
        status: if diagnostics.is_empty() {
            AnimationStudioReadbackStatusV1::Match
        } else {
            AnimationStudioReadbackStatusV1::Mismatch
        },
        clips,
        diagnostics,
    }
}

/// Reconciles the complete expected Studio materialization with the writer
/// readback, then independently verifies that the packaged MDL preserves that
/// exact writer result.
pub fn evaluate_edited_animation_conformance_v1(
    studio: &AnimationStudioDocumentV1,
    expected: &MaterializedCreatureAnimationAuthoringV2,
    rig: &AnimationStudioRigV1,
    writer_readback: &InspectionReport,
    package_readback: &InspectionReport,
) -> AnimationStudioReadbackV1 {
    let expected_to_writer =
        reconcile_animation_studio_readback_v1(studio, expected, rig, writer_readback);
    let mut diagnostics = expected_to_writer.diagnostics;
    let clips = expected_to_writer.clips;
    for usage in &expected.authored_usages {
        let writer_clip = writer_readback
            .animations
            .iter()
            .find(|clip| clip.name == usage.output_clip_name);
        let packaged_clip = package_readback
            .animations
            .iter()
            .find(|clip| clip.name == usage.output_clip_name);
        match (writer_clip, packaged_clip) {
            (Some(writer_clip), Some(packaged_clip))
                if fingerprint_json(writer_clip) == fingerprint_json(packaged_clip) => {}
            (Some(_), Some(_)) => diagnostics.push(diagnostic(
                DIAGNOSTIC_READBACK_MISMATCH,
                format!("readback.animations.{}", usage.output_clip_name),
                "packaged binary MDL clip differs from the canonical writer readback",
                "Block Download and inspect package extraction or binary serialization.",
            )),
            _ => diagnostics.push(diagnostic(
                DIAGNOSTIC_READBACK_MISMATCH,
                format!("readback.animations.{}", usage.output_clip_name),
                "authored output usage is missing from writer or packaged binary readback",
                "Block Download and inspect stable-id output routing.",
            )),
        }
    }
    AnimationStudioReadbackV1 {
        schema_version: ANIMATION_STUDIO_READBACK_SCHEMA_VERSION,
        studio_fingerprint: fingerprint_animation_studio_document_v1(studio),
        source_revision: studio.source_revision.clone(),
        status: if diagnostics.is_empty() {
            AnimationStudioReadbackStatusV1::Match
        } else {
            AnimationStudioReadbackStatusV1::Mismatch
        },
        clips,
        diagnostics,
    }
}

fn reconcile_clip(
    expected: &MdlAnimationClipV1,
    actual: &AnimationReport,
    rig: &AnimationStudioRigV1,
    diagnostics: &mut Vec<AnimationStudioDiagnosticV1>,
) {
    let base_path = format!("readback.animations.{}", expected.name);
    if !same_float(expected.length_seconds, actual.length)
        || !same_float(expected.transition_seconds, actual.transition)
        || expected.animation_root != actual.animation_root
    {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_READBACK_MISMATCH,
            &base_path,
            "authored clip name/length/transition/animationRoot differs after binary MDL readback",
            "Block Download and inspect the canonical writer/readback delta.",
        ));
    }
    if expected.events.len() != actual.events.len()
        || expected
            .events
            .iter()
            .zip(&actual.events)
            .any(|(left, right)| {
                left.name != right.name || !same_float(left.time_seconds, right.time)
            })
    {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_READBACK_MISMATCH,
            format!("{base_path}.events"),
            "authored event name, time, or stable ordering differs after binary MDL readback",
            "Block Download and inspect event serialization.",
        ));
    }
    for track in &expected.tracks {
        let controller_type = match track.path {
            MdlAnimationTrackPathV1::Translation => 8,
            MdlAnimationTrackPathV1::Rotation => 20,
            MdlAnimationTrackPathV1::Scale => 36,
            MdlAnimationTrackPathV1::Weights => -1,
        };
        let node = rig
            .nodes
            .iter()
            .find(|node| node.node_id == track.target_node_id)
            .and_then(|target| find_readback_node_by_name(&actual.node_tree.roots, &target.name));
        let controller = node.and_then(|node| {
            node.controllers
                .iter()
                .find(|controller| controller.controller_type == controller_type)
        });
        let Some(controller) = controller else {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_READBACK_MISMATCH,
                format!(
                    "{base_path}.tracks[{}.{:?}]",
                    track.target_node_id, track.path
                ),
                "authored target/path controller is missing after binary MDL readback",
                "Block Download and inspect controller emission.",
            ));
            continue;
        };
        let times_match = track.times_seconds.len() == controller.times.len()
            && track
                .times_seconds
                .iter()
                .zip(&controller.times)
                .all(|(left, right)| same_float(*left, *right));
        let values_match = track.values.len() == controller.values.len()
            && track
                .values
                .iter()
                .zip(&controller.values)
                .all(|(left, right)| {
                    canonical_track_value(track.path, left)
                        .zip(canonical_track_value(track.path, right))
                        .is_some_and(|(left, right)| {
                            left.len() == right.len()
                                && left
                                    .iter()
                                    .zip(right)
                                    .all(|(left, right)| same_float(*left, right))
                        })
                });
        if !times_match || !values_match {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_READBACK_MISMATCH,
                format!(
                    "{base_path}.tracks[{}.{:?}]",
                    track.target_node_id, track.path
                ),
                "authored key times or values differ after binary MDL readback",
                "Block Download and inspect canonical quaternion/value serialization.",
            ));
        }
    }
}

fn find_readback_node_by_name<'a>(
    nodes: &'a [NodeReport],
    target_node_name: &str,
) -> Option<&'a NodeReport> {
    for node in nodes {
        if node.name == target_node_name {
            return Some(node);
        }
        if let Some(found) = find_readback_node_by_name(&node.children, target_node_name) {
            return Some(found);
        }
    }
    None
}

fn canonical_track_value(path: MdlAnimationTrackPathV1, value: &[f32]) -> Option<Vec<f32>> {
    if value.iter().any(|component| !component.is_finite()) {
        return None;
    }
    match path {
        MdlAnimationTrackPathV1::Rotation => canonical_unit_quaternion(value.to_vec()).ok(),
        _ => Some(value.to_vec()),
    }
}

fn same_float(left: f32, right: f32) -> bool {
    (left - right).abs() <= 1.0e-5
}

fn validate_rig_and_input(
    rig: &AnimationStudioRigV1,
    input: &AuthoredAnimationClipInputV1,
) -> Result<(), AnimationStudioDiagnosticV1> {
    validate_input(input)?;
    if rig.schema_version != ANIMATION_STUDIO_RIG_SCHEMA_VERSION || !is_sha256(&rig.source_revision)
    {
        return Err(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "rig",
            "rig requires schemaVersion 1 and a SHA-256 sourceRevision",
            "Re-inspect the exact source GLB rig.",
        ));
    }
    if input.source_revision != rig.source_revision {
        return Err(diagnostic(
            DIAGNOSTIC_SOURCE_STALE,
            "clipInput.sourceRevision",
            "clip input and rig refer to different exact source GLB revisions",
            "Recreate or reconcile the draft against the current rig.",
        ));
    }
    let mut node_ids = BTreeSet::new();
    if rig.nodes.is_empty() || rig.nodes.iter().any(|node| !node_ids.insert(node.node_id)) {
        return Err(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "rig.nodes",
            "rig nodes must be non-empty and have unique ids",
            "Repair the source rig inspection.",
        ));
    }
    Ok(())
}

fn validate_input(input: &AuthoredAnimationClipInputV1) -> Result<(), AnimationStudioDiagnosticV1> {
    if input.id.trim().is_empty() {
        return Err(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "id",
            "authored clip id cannot be empty",
            "Choose a stable non-empty id.",
        ));
    }
    if !is_sha256(&input.source_revision) {
        return Err(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "clipInput.sourceRevision",
            "sourceRevision must be the SHA-256 of the exact source GLB",
            "Inspect the exact source GLB before creating the draft.",
        ));
    }
    validate_output_name(&input.name)?;
    if !input.length_seconds.is_finite()
        || input.length_seconds <= 0.0
        || input.length_seconds > ANIMATION_STUDIO_MAX_DURATION_SECONDS
        || !input.transition_seconds.is_finite()
        || input.transition_seconds < 0.0
        || input.transition_seconds > input.length_seconds
        || input.animation_root.trim().is_empty()
    {
        return Err(diagnostic(
            DIAGNOSTIC_SCHEMA,
            "clipInput",
            "clip input requires a positive duration, bounded transition, and animation root",
            "Repair the clip creation fields.",
        ));
    }
    Ok(())
}

fn validate_output_name(name: &str) -> Result<(), AnimationStudioDiagnosticV1> {
    let trimmed = name.trim();
    let portable = !trimmed.is_empty()
        && trimmed.len() <= 16
        && trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    if portable {
        Ok(())
    } else {
        Err(diagnostic(
            DIAGNOSTIC_CLIP_NAME,
            "name",
            "animation name must contain 1..=16 ASCII letters, digits, or underscores",
            "Enter a portable Aurora animation output name.",
        ))
    }
}

fn track_with_values(
    node_id: u32,
    path: AuthoredAnimationTrackPathV1,
    times: &[f32],
    values: &[Vec<f32>],
) -> AuthoredAnimationTrackV1 {
    AuthoredAnimationTrackV1 {
        id: track_id(node_id, path),
        target_node_id: node_id,
        path,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes: times
            .iter()
            .zip(values)
            .enumerate()
            .map(|(index, (time, value))| AnimationKeyframeV1 {
                id: format!("key-{index:04}"),
                time_seconds: *time,
                value: value.clone(),
            })
            .collect(),
    }
}

fn track_id(node_id: u32, path: AuthoredAnimationTrackPathV1) -> String {
    format!("track-{node_id}-{}", path.id_suffix())
}

fn validate_key_value(
    path: AuthoredAnimationTrackPathV1,
    time_seconds: f32,
    value: &[f32],
    track_id: &str,
) -> Result<(), AnimationStudioDiagnosticV1> {
    if !time_seconds.is_finite() || time_seconds.abs() > ANIMATION_STUDIO_MAX_DURATION_SECONDS {
        return Err(nonfinite_time(&format!("tracks[{track_id}]")));
    }
    if value.len() != path.expected_value_count() {
        return Err(diagnostic(
            DIAGNOSTIC_PATH_UNSUPPORTED,
            format!("tracks[{track_id}].keyframes.value"),
            format!(
                "{path:?} key requires {} values, got {}",
                path.expected_value_count(),
                value.len()
            ),
            "Provide a value with the correct dimension.",
        ));
    }
    if value.iter().any(|component| {
        !component.is_finite() || component.abs() > ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE
    }) {
        return Err(diagnostic(
            DIAGNOSTIC_VALUE_NONFINITE,
            format!("tracks[{track_id}].keyframes.value"),
            format!(
                "keyframe values must be finite and within +/-{}",
                ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE
            ),
            "Replace the transform with values inside the Animation Studio product range.",
        ));
    }
    Ok(())
}

fn key_index(
    track: &AuthoredAnimationTrackV1,
    key_id: &str,
) -> Result<usize, AnimationStudioDiagnosticV1> {
    track
        .keyframes
        .iter()
        .position(|key| key.id == key_id)
        .ok_or_else(|| {
            diagnostic(
                DIAGNOSTIC_SCHEMA,
                format!("tracks[{}].keyframes[{key_id}]", track.id),
                "keyframe does not exist",
                "Select an existing keyframe.",
            )
        })
}

fn event_index(
    clip: &AuthoredAnimationClipV1,
    event_id: &str,
) -> Result<usize, AnimationStudioDiagnosticV1> {
    clip.events
        .iter()
        .position(|event| event.id == event_id)
        .ok_or_else(|| {
            diagnostic(
                DIAGNOSTIC_EVENT,
                format!("authoredClips[{}].events[{event_id}]", clip.id),
                "animation event does not exist",
                "Select an existing event.",
            )
        })
}

fn canonical_unit_quaternion(mut value: Vec<f32>) -> Result<Vec<f32>, AnimationStudioDiagnosticV1> {
    if value.len() != 4 || value.iter().any(|component| !component.is_finite()) {
        return Err(diagnostic(
            DIAGNOSTIC_QUATERNION,
            "rotation",
            "rotation quaternion requires four finite components",
            "Provide a valid quaternion.",
        ));
    }
    let norm = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= 1.0e-6 {
        return Err(diagnostic(
            DIAGNOSTIC_QUATERNION,
            "rotation",
            "rotation quaternion must have a non-zero finite norm",
            "Provide a valid quaternion.",
        ));
    }
    for component in &mut value {
        *component /= norm;
    }
    let flip = value[3] < 0.0
        || (value[3] == 0.0
            && value
                .iter()
                .take(3)
                .find(|component| **component != 0.0)
                .is_some_and(|component| *component < 0.0));
    if flip {
        for component in &mut value {
            *component = -*component;
        }
    }
    for component in &mut value {
        if *component == 0.0 {
            *component = 0.0;
        }
    }
    Ok(value)
}

fn sort_keyframes(track: &mut AuthoredAnimationTrackV1) {
    track.keyframes.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn same_time(left: f32, right: f32) -> bool {
    left.to_bits() == right.to_bits() || (left - right).abs() <= 1.0e-6
}

fn next_prefixed_id<'a>(prefix: &str, existing: impl Iterator<Item = &'a str>) -> String {
    let ids = existing.collect::<BTreeSet<_>>();
    for index in 0_u64.. {
        let candidate = format!("{prefix}-{index:04}");
        if !ids.contains(candidate.as_str()) {
            return candidate;
        }
    }
    unreachable!("u64 id space exhausted")
}

fn find_clip_mut<'a>(
    document: &'a mut AnimationStudioDocumentV1,
    clip_id: &str,
) -> Result<&'a mut AuthoredAnimationClipV1, AnimationStudioDiagnosticV1> {
    document
        .authored_clips
        .iter_mut()
        .find(|clip| clip.id == clip_id)
        .ok_or_else(|| clip_missing(clip_id))
}

fn clip_missing(clip_id: &str) -> AnimationStudioDiagnosticV1 {
    diagnostic(
        DIAGNOSTIC_SCHEMA,
        format!("authoredClips[{clip_id}]"),
        "authored clip does not exist",
        "Select an existing clip from the Custom library.",
    )
}

fn touch_clip(clip: &mut AuthoredAnimationClipV1) {
    clip.revision = clip.revision.saturating_add(1);
    clip.status = AuthoredAnimationClipStatusV1::Draft;
}

fn touch_document(document: &mut AnimationStudioDocumentV1) {
    document.authoring_revision = document.authoring_revision.saturating_add(1);
    document.status = AnimationStudioDocumentStatusV1::Draft;
}

fn nonfinite_time(path: &str) -> AnimationStudioDiagnosticV1 {
    diagnostic(
        DIAGNOSTIC_VALUE_NONFINITE,
        path,
        "time must be finite",
        "Replace NaN or infinity with a finite time.",
    )
}

fn diagnostic(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
    action: impl Into<String>,
) -> AnimationStudioDiagnosticV1 {
    AnimationStudioDiagnosticV1 {
        schema_version: ANIMATION_STUDIO_DIAGNOSTIC_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        level: AnimationStudioDiagnosticLevelV1::Blocking,
        message: message.into(),
        action: action.into(),
    }
}

fn fingerprint_json(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("animation source contains serializable values");
    sha256_hex(&bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// `JSON.stringify` emits integral numbers without a decimal suffix, while
/// serde_json preserves the Rust floating-point type as `0.0`. Studio uses one
/// cross-language wire fingerprint, so normalize only numeric tokens outside
/// JSON strings. Object and array ordering stays exactly as emitted by serde.
fn normalize_integral_json_floats(json: &str) -> String {
    let bytes = json.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            output.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            output.push(byte);
            index += 1;
            continue;
        }
        if byte == b'-' || byte.is_ascii_digit() {
            let start = index;
            index += 1;
            while index < bytes.len()
                && matches!(bytes[index], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
            {
                index += 1;
            }
            let token = &json[start..index];
            if !token.contains('e') && !token.contains('E') && token.ends_with(".0") {
                let integer = &token[..token.len() - 2];
                if integer == "-0" {
                    output.push(b'0');
                } else {
                    output.extend_from_slice(integer.as_bytes());
                }
            } else {
                output.extend_from_slice(token.as_bytes());
            }
            continue;
        }
        output.push(byte);
        index += 1;
    }
    String::from_utf8(output).expect("normalization preserves UTF-8 byte sequences")
}
