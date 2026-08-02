//! Portable, repository-owned animation preset contracts.
//!
//! Library presets deliberately use bone names and a strict rest-rig signature.
//! Applying a preset never mutates the immutable preset: it creates a local,
//! source-bound Animation Studio draft with complete provenance.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_studio::{
        ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE, ANIMATION_STUDIO_MAX_DURATION_SECONDS,
        ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP, AnimationKeyframeV1,
        AnimationLibraryPresetProvenanceV1, AnimationStudioRigV1, AuthoredAnimationClipKindV1,
        AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1, AuthoredAnimationEventV1,
        AuthoredAnimationSourceKindV1, AuthoredAnimationSourceV1, AuthoredAnimationTrackPathV1,
        AuthoredAnimationTrackV1,
    },
    mdl::MdlAnimationInterpolationV1,
};

pub const ANIMATION_PRESET_SCHEMA_VERSION_V1: u32 = 1;
pub const COMMUNITY_ANIMATION_CATALOG_SCHEMA_VERSION_V1: u32 = 1;
pub const ANIMATION_RIG_PROFILE_SCHEMA_VERSION_V1: u32 = 1;
pub const ANIMATION_LIBRARY_MAX_TRACKS_V1: usize = 256;
pub const ANIMATION_LIBRARY_MAX_EVENTS_V1: usize = 256;
pub const ANIMATION_LIBRARY_MAX_PRESETS_V1: usize = 2_048;
pub const ANIMATION_LIBRARY_MAX_ANIMATION_BYTES_V1: u64 = 2 * 1024 * 1024;
pub const ANIMATION_LIBRARY_MAX_PREVIEW_BYTES_V1: u64 = 2 * 1024 * 1024;
pub const ANIMATION_LIBRARY_MAX_TOTAL_BYTES_V1: u64 = 128 * 1024 * 1024;
pub const ANIMATION_LIBRARY_INSTANTIATION_MODE_V1: &str = "STRICT_RIG_V1";
/// Stable decimal precision used by the V1 rig identity.
///
/// GLTF rest transforms can move by one or two `f32` ULPs after a lossless
/// load/decompose/write cycle. Hashing those representation-only differences
/// would reject the same semantic rig. Six decimal places remain far below a
/// meaningful rest-pose edit while making the signature stable across that
/// round trip.
pub const ANIMATION_RIG_SIGNATURE_QUANTIZATION_SCALE_V1: f64 = 1_000_000.0;

pub const ANIMATION_LIBRARY_DEFAULT_TAGS_V1: [&str; 17] = [
    "attack",
    "boxing",
    "combat",
    "defense",
    "dodge",
    "full-body",
    "guard",
    "humanoid",
    "left-hand",
    "locomotion",
    "loop",
    "one-shot",
    "right-hand",
    "sword",
    "unarmed",
    "upper-body",
    "utility",
];

pub const ANIMATION_LIBRARY_DEFAULT_LICENSES_V1: [&str; 3] = [
    "CC0-1.0",
    "CC-BY-4.0",
    "LicenseRef-Meshy2Aurora-Project-Generated",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationPresetSourceV1 {
    BuiltIn,
    Community,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationPresetPlaybackV1 {
    OneShot,
    Loop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationLibraryValidationStatusV1 {
    PipelineVerified,
    OwnerNwnVerified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationLibraryCatalogSourceV1 {
    EmbeddedRelease,
    RepositoryCheck,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetAuthorV1 {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetManifestV1 {
    pub schema_version: u32,
    pub preset_id: String,
    pub preset_version: u32,
    pub output_name: String,
    pub label: String,
    pub summary: String,
    pub source: AnimationPresetSourceV1,
    pub authors: Vec<AnimationPresetAuthorV1>,
    pub license: String,
    pub tags: Vec<String>,
    pub playback: AnimationPresetPlaybackV1,
    pub duration_seconds: f32,
    pub rig_profile: String,
    pub rig_signature_sha256: String,
    pub required_bones: Vec<String>,
    pub animation_path: String,
    pub animation_byte_length: u64,
    pub animation_sha256: String,
    pub motion_sha256: String,
    pub preview_path: Option<String>,
    pub preview_byte_length: Option<u64>,
    pub preview_sha256: Option<String>,
    pub validation_status: AnimationLibraryValidationStatusV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetKeyframeV1 {
    pub time_seconds: f32,
    pub value: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetTrackV1 {
    pub target_bone_name: String,
    pub path: AuthoredAnimationTrackPathV1,
    pub interpolation: MdlAnimationInterpolationV1,
    pub keyframes: Vec<AnimationPresetKeyframeV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetEventV1 {
    pub time_seconds: f32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetPayloadV1 {
    pub schema_version: u32,
    pub duration_seconds: f32,
    pub animation_root_bone_name: String,
    pub tracks: Vec<AnimationPresetTrackV1>,
    pub events: Vec<AnimationPresetEventV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetV1 {
    pub manifest: AnimationPresetManifestV1,
    pub animation: AnimationPresetPayloadV1,
    pub catalog_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationContributionV1 {
    pub schema_version: u32,
    pub preset_id: String,
    pub preset_version: u32,
    pub output_name: String,
    pub label: String,
    pub summary: String,
    pub authors: Vec<AnimationPresetAuthorV1>,
    pub license: String,
    pub tags: Vec<String>,
    pub playback: AnimationPresetPlaybackV1,
    pub rig_profile: String,
    pub rig_signature_sha256: String,
    pub required_bones: Vec<String>,
    pub motion_sha256: String,
    pub validation_status: AnimationLibraryValidationStatusV1,
    pub animation: AnimationPresetPayloadV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportAnimationContributionMetadataV1 {
    pub preset_id: String,
    pub preset_version: u32,
    pub output_name: String,
    pub label: String,
    pub summary: String,
    pub authors: Vec<AnimationPresetAuthorV1>,
    pub license: String,
    pub tags: Vec<String>,
    pub playback: AnimationPresetPlaybackV1,
    pub rig_profile: String,
    pub validation_status: AnimationLibraryValidationStatusV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationRigProfileNodeV1 {
    pub name: String,
    pub parent_name: Option<String>,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationRigProfileV1 {
    pub schema_version: u32,
    pub animation_root_bone_name: String,
    pub nodes: Vec<AnimationRigProfileNodeV1>,
    pub signature_sha256: String,
}

impl AnimationRigProfileV1 {
    pub fn from_rig(rig: &AnimationStudioRigV1) -> Result<Self, Vec<AnimationLibraryDiagnosticV1>> {
        let (animation_root_bone_name, nodes) = canonical_rig_rows_v1(rig)?;
        let signature_sha256 = rig_rows_sha256_v1(&animation_root_bone_name, &nodes)?;
        Ok(Self {
            schema_version: ANIMATION_RIG_PROFILE_SCHEMA_VERSION_V1,
            animation_root_bone_name,
            nodes,
            signature_sha256,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationRigCompatibilityStatusV1 {
    Compatible,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationLibraryDiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
    pub action: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationRigCompatibilityV1 {
    pub schema_version: u32,
    pub status: AnimationRigCompatibilityStatusV1,
    pub expected_rig_signature_sha256: String,
    pub actual_rig_signature_sha256: Option<String>,
    pub missing_bones: Vec<String>,
    pub diagnostics: Vec<AnimationLibraryDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommunityAnimationCatalogV1 {
    pub schema_version: u32,
    pub source: AnimationLibraryCatalogSourceV1,
    pub entries: Vec<AnimationPresetManifestV1>,
    pub catalog_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnsignedCommunityAnimationCatalogV1<'a> {
    schema_version: u32,
    source: AnimationLibraryCatalogSourceV1,
    entries: &'a [AnimationPresetManifestV1],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MotionHashPayloadV1<'a> {
    schema_version: u32,
    duration_seconds: f32,
    animation_root_bone_name: &'a str,
    tracks: Vec<MotionHashTrackV1<'a>>,
    events: Vec<MotionHashEventV1<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MotionHashTrackV1<'a> {
    target_bone_name: &'a str,
    path: AuthoredAnimationTrackPathV1,
    interpolation: MdlAnimationInterpolationV1,
    keyframes: Vec<MotionHashKeyframeV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MotionHashKeyframeV1 {
    time_seconds: f32,
    value: Vec<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MotionHashEventV1<'a> {
    time_seconds: f32,
    name: &'a str,
}

pub fn sha256_hex_v1(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn parse_animation_preset_v1(
    json: &str,
) -> Result<AnimationPresetV1, Vec<AnimationLibraryDiagnosticV1>> {
    serde_json::from_str(json).map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-SCHEMA",
            "presetJson",
            format!("preset JSON does not match the strict V1 contract: {error}"),
            "Export the preset again or repair the exact reported field.",
        )]
    })
}

pub fn parse_animation_preset_assets_v1(
    manifest_json: &str,
    animation_json: &str,
    catalog_sha256: &str,
) -> Result<AnimationPresetV1, Vec<AnimationLibraryDiagnosticV1>> {
    let manifest: AnimationPresetManifestV1 =
        serde_json::from_str(manifest_json).map_err(|error| {
            vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-SCHEMA",
                "manifestJson",
                format!("manifest JSON does not match the strict V1 contract: {error}"),
                "Regenerate the exact preset manifest.",
            )]
        })?;
    let animation: AnimationPresetPayloadV1 =
        serde_json::from_str(animation_json).map_err(|error| {
            vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-SCHEMA",
                "animationJson",
                format!("animation JSON does not match the strict V1 contract: {error}"),
                "Regenerate the exact animation payload.",
            )]
        })?;
    validate_animation_asset_bytes_v1(&manifest, animation_json.as_bytes())?;
    let preset = AnimationPresetV1 {
        manifest,
        animation,
        catalog_sha256: catalog_sha256.to_owned(),
    };
    let diagnostics = validate_animation_preset_v1(
        &preset,
        &ANIMATION_LIBRARY_DEFAULT_TAGS_V1
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    );
    if diagnostics.is_empty() {
        Ok(preset)
    } else {
        Err(diagnostics)
    }
}

pub fn canonical_animation_preset_json_v1(
    preset: &AnimationPresetV1,
) -> Result<String, Vec<AnimationLibraryDiagnosticV1>> {
    serde_json::to_string(preset).map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-SERIALIZE",
            "preset",
            format!("preset cannot be serialized canonically: {error}"),
            "Repair non-finite or unsupported preset values.",
        )]
    })
}

pub fn animation_preset_motion_sha256_v1(
    payload: &AnimationPresetPayloadV1,
) -> Result<String, Vec<AnimationLibraryDiagnosticV1>> {
    let mut tracks = payload
        .tracks
        .iter()
        .map(|track| MotionHashTrackV1 {
            target_bone_name: track.target_bone_name.as_str(),
            path: track.path,
            interpolation: track.interpolation,
            keyframes: track
                .keyframes
                .iter()
                .map(|keyframe| MotionHashKeyframeV1 {
                    time_seconds: canonical_f32(keyframe.time_seconds),
                    value: canonical_track_value_v1(track.path, &keyframe.value),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    tracks.sort_by(|left, right| {
        left.target_bone_name
            .cmp(right.target_bone_name)
            .then(left.path.cmp(&right.path))
    });
    let mut events = payload
        .events
        .iter()
        .map(|event| MotionHashEventV1 {
            time_seconds: canonical_f32(event.time_seconds),
            name: event.name.as_str(),
        })
        .collect::<Vec<_>>();
    events.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then(left.name.cmp(right.name))
    });
    let canonical = MotionHashPayloadV1 {
        schema_version: payload.schema_version,
        duration_seconds: canonical_f32(payload.duration_seconds),
        animation_root_bone_name: payload.animation_root_bone_name.as_str(),
        tracks,
        events,
    };
    serde_json::to_vec(&canonical)
        .map(|bytes| sha256_hex_v1(&bytes))
        .map_err(|error| {
            vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-MOTION-HASH",
                "animation",
                format!("motion cannot be serialized canonically: {error}"),
                "Repair non-finite animation values.",
            )]
        })
}

pub fn animation_rig_signature_v1(
    rig: &AnimationStudioRigV1,
) -> Result<String, Vec<AnimationLibraryDiagnosticV1>> {
    let (animation_root, nodes) = canonical_rig_rows_v1(rig)?;
    rig_rows_sha256_v1(&animation_root, &nodes)
}

pub fn inspect_animation_preset_compatibility_v1(
    preset: &AnimationPresetV1,
    rig: &AnimationStudioRigV1,
) -> AnimationRigCompatibilityV1 {
    let names = rig
        .nodes
        .iter()
        .map(|node| node.name.as_str())
        .collect::<BTreeSet<_>>();
    let missing_bones = preset
        .manifest
        .required_bones
        .iter()
        .filter(|name| !names.contains(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let signature = animation_rig_signature_v1(rig);
    let actual = signature.as_ref().ok().cloned();
    let mut diagnostics = signature.err().unwrap_or_default();
    if !missing_bones.is_empty() {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG-BONE-MISSING",
            "rig.nodes",
            format!("rig is missing required bones: {}", missing_bones.join(", ")),
            "Use a preset made for this exact rig profile; automatic retargeting is not part of V1.",
        ));
    }
    if actual.as_deref() != Some(preset.manifest.rig_signature_sha256.as_str()) {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG-SIGNATURE",
            "rig",
            "rig hierarchy or rest pose differs from the preset's strict rig signature",
            "Choose a compatible preset. V1 does not guess or automatically retarget different rigs.",
        ));
    }
    diagnostics.sort_by(|left, right| left.code.cmp(&right.code).then(left.path.cmp(&right.path)));
    diagnostics.dedup();
    AnimationRigCompatibilityV1 {
        schema_version: 1,
        status: if diagnostics.is_empty() {
            AnimationRigCompatibilityStatusV1::Compatible
        } else {
            AnimationRigCompatibilityStatusV1::Incompatible
        },
        expected_rig_signature_sha256: preset.manifest.rig_signature_sha256.clone(),
        actual_rig_signature_sha256: actual,
        missing_bones,
        diagnostics,
    }
}

pub fn validate_animation_preset_v1(
    preset: &AnimationPresetV1,
    allowed_tags: &[String],
) -> Vec<AnimationLibraryDiagnosticV1> {
    let mut diagnostics = validate_manifest_shape_v1(&preset.manifest, allowed_tags);
    diagnostics.extend(validate_animation_payload_v1(&preset.animation));
    let expected_required_bones = preset
        .animation
        .tracks
        .iter()
        .map(|track| track.target_bone_name.clone())
        .chain(std::iter::once(
            preset.animation.animation_root_bone_name.clone(),
        ))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if preset.manifest.required_bones != expected_required_bones {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG-BONES-MISMATCH",
            "manifest.requiredBones",
            "requiredBones differs from the exact animation root and track targets",
            "Regenerate requiredBones from animationRootBoneName and every targetBoneName.",
        ));
    }
    if preset.manifest.duration_seconds != preset.animation.duration_seconds {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-DURATION",
            "manifest.durationSeconds",
            "manifest and animation durationSeconds differ",
            "Regenerate the manifest from the exact animation payload.",
        ));
    }
    match animation_preset_motion_sha256_v1(&preset.animation) {
        Ok(hash) if hash != preset.manifest.motion_sha256 => diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-MOTION-HASH",
            "manifest.motionSha256",
            "declared motion SHA-256 is stale",
            "Regenerate the preset manifest after changing motion.",
        )),
        Err(errors) => diagnostics.extend(errors),
        _ => {}
    }
    if !is_sha256(&preset.catalog_sha256) {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-CATALOG-HASH",
            "catalogSha256",
            "resolved preset requires the exact catalog SHA-256",
            "Load the preset through the generated catalog.",
        ));
    }
    diagnostics
}

pub fn instantiate_animation_preset_v1(
    preset: &AnimationPresetV1,
    rig: &AnimationStudioRigV1,
    source_revision: &str,
    id: &str,
    output_name: &str,
) -> Result<AuthoredAnimationClipV1, Vec<AnimationLibraryDiagnosticV1>> {
    let mut diagnostics = validate_animation_preset_v1(
        preset,
        &ANIMATION_LIBRARY_DEFAULT_TAGS_V1
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    );
    if !is_sha256(source_revision) || source_revision != rig.source_revision {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-SOURCE-STALE",
            "sourceRevision",
            "preset instantiation must target the exact current rig source revision",
            "Re-inspect the current GLB and retry with its exact SHA-256.",
        ));
    }
    if id.trim().is_empty() || !is_portable_output_name(output_name) {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-INSTANCE",
            "clipInput",
            "instance id must be non-empty and output name must be 1..=16 portable ASCII characters",
            "Choose a stable local id and an Aurora-safe output name.",
        ));
    }
    let compatibility = inspect_animation_preset_compatibility_v1(preset, rig);
    diagnostics.extend(compatibility.diagnostics);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let node_ids = rig
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node.node_id))
        .collect::<BTreeMap<_, _>>();
    let tracks = preset
        .animation
        .tracks
        .iter()
        .enumerate()
        .map(|(track_index, track)| AuthoredAnimationTrackV1 {
            id: format!("library-track-{track_index:04}"),
            target_node_id: node_ids[track.target_bone_name.as_str()],
            path: track.path,
            interpolation: track.interpolation,
            keyframes: track
                .keyframes
                .iter()
                .enumerate()
                .map(|(key_index, keyframe)| AnimationKeyframeV1 {
                    id: format!("library-key-{track_index:04}-{key_index:04}"),
                    time_seconds: keyframe.time_seconds,
                    value: canonical_track_value_v1(track.path, &keyframe.value),
                })
                .collect(),
        })
        .collect();
    let events = preset
        .animation
        .events
        .iter()
        .enumerate()
        .map(|(index, event)| AuthoredAnimationEventV1 {
            id: format!("library-event-{index:04}"),
            time_seconds: event.time_seconds,
            name: event.name.clone(),
        })
        .collect();
    Ok(AuthoredAnimationClipV1 {
        id: id.to_owned(),
        name: output_name.to_owned(),
        kind: AuthoredAnimationClipKindV1::Motion,
        status: AuthoredAnimationClipStatusV1::Draft,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::LibraryPresetCopy,
            source_revision: source_revision.to_owned(),
            source_clip_name: None,
            source_clip_fingerprint: Some(preset.manifest.motion_sha256.clone()),
            procedural_template: None,
            library_preset: Some(AnimationLibraryPresetProvenanceV1 {
                preset_id: preset.manifest.preset_id.clone(),
                preset_version: preset.manifest.preset_version,
                preset_motion_sha256: preset.manifest.motion_sha256.clone(),
                catalog_sha256: preset.catalog_sha256.clone(),
                source: match preset.manifest.source {
                    AnimationPresetSourceV1::BuiltIn => "BUILT_IN",
                    AnimationPresetSourceV1::Community => "COMMUNITY",
                }
                .to_owned(),
                authors: preset
                    .manifest
                    .authors
                    .iter()
                    .map(|author| author.name.clone())
                    .collect(),
                license: preset.manifest.license.clone(),
                rig_signature_sha256: preset.manifest.rig_signature_sha256.clone(),
                instantiation_mode: ANIMATION_LIBRARY_INSTANTIATION_MODE_V1.to_owned(),
            }),
            retarget: None,
        },
        length_seconds: preset.animation.duration_seconds,
        transition_seconds: preset.animation.duration_seconds.min(0.1),
        animation_root: rig.animation_root.clone(),
        tracks,
        events,
        revision: 1,
    })
}

pub fn export_animation_contribution_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    metadata: ExportAnimationContributionMetadataV1,
) -> Result<AnimationContributionV1, Vec<AnimationLibraryDiagnosticV1>> {
    let mut diagnostics = Vec::new();
    if clip.status != AuthoredAnimationClipStatusV1::Valid {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-CONTRIBUTION-CLIP",
            "clip.status",
            "only a VALID local Custom clip can be exported as a contribution",
            "Resolve diagnostics and use Save to Custom before exporting.",
        ));
    }
    if clip.source.source_revision != rig.source_revision {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-SOURCE-STALE",
            "clip.source.sourceRevision",
            "clip and inspected rig refer to different source revisions",
            "Reconcile the clip with the exact current source GLB.",
        ));
    }
    let names = rig
        .nodes
        .iter()
        .map(|node| (node.node_id, node.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut required_bones = BTreeSet::from([rig.animation_root.clone()]);
    let mut tracks = Vec::with_capacity(clip.tracks.len());
    for (index, track) in clip.tracks.iter().enumerate() {
        let Some(name) = names.get(&track.target_node_id) else {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-LIBRARY-RIG-BONE-MISSING",
                format!("clip.tracks[{index}].targetNodeId"),
                "track target does not exist in the inspected rig",
                "Reconcile the clip with the current rig.",
            ));
            continue;
        };
        required_bones.insert((*name).to_owned());
        tracks.push(AnimationPresetTrackV1 {
            target_bone_name: (*name).to_owned(),
            path: track.path,
            interpolation: track.interpolation,
            keyframes: track
                .keyframes
                .iter()
                .map(|keyframe| AnimationPresetKeyframeV1 {
                    time_seconds: keyframe.time_seconds,
                    value: canonical_track_value_v1(track.path, &keyframe.value),
                })
                .collect(),
        });
    }
    tracks.sort_by(|left, right| {
        left.target_bone_name
            .cmp(&right.target_bone_name)
            .then(left.path.cmp(&right.path))
    });
    let mut events = clip
        .events
        .iter()
        .map(|event| AnimationPresetEventV1 {
            time_seconds: event.time_seconds,
            name: event.name.clone(),
        })
        .collect::<Vec<_>>();
    events.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then(left.name.cmp(&right.name))
    });
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let animation = AnimationPresetPayloadV1 {
        schema_version: 1,
        duration_seconds: clip.length_seconds,
        animation_root_bone_name: rig.animation_root.clone(),
        tracks,
        events,
    };
    let rig_signature_sha256 = animation_rig_signature_v1(rig)?;
    let motion_sha256 = animation_preset_motion_sha256_v1(&animation)?;
    let contribution = AnimationContributionV1 {
        schema_version: 1,
        preset_id: metadata.preset_id,
        preset_version: metadata.preset_version,
        output_name: metadata.output_name,
        label: metadata.label,
        summary: metadata.summary,
        authors: metadata.authors,
        license: metadata.license,
        tags: metadata.tags,
        playback: metadata.playback,
        rig_profile: metadata.rig_profile,
        rig_signature_sha256,
        required_bones: required_bones.into_iter().collect(),
        motion_sha256,
        validation_status: metadata.validation_status,
        animation,
    };
    let errors = validate_animation_contribution_v1(&contribution);
    if errors.is_empty() {
        Ok(contribution)
    } else {
        Err(errors)
    }
}

pub fn validate_animation_contribution_v1(
    contribution: &AnimationContributionV1,
) -> Vec<AnimationLibraryDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if contribution.validation_status == AnimationLibraryValidationStatusV1::OwnerNwnVerified {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-OWNER-PROOF-CLAIM",
            "validationStatus",
            "a contribution cannot self-assert owner-verified NWN playback",
            "Export as PIPELINE_VERIFIED; maintainers may promote the tracked preset only after binding exact owner proof evidence.",
        ));
    }
    let animation_bytes = serde_json::to_vec(&contribution.animation).unwrap_or_default();
    let preset = AnimationPresetV1 {
        manifest: AnimationPresetManifestV1 {
            schema_version: contribution.schema_version,
            preset_id: contribution.preset_id.clone(),
            preset_version: contribution.preset_version,
            output_name: contribution.output_name.clone(),
            label: contribution.label.clone(),
            summary: contribution.summary.clone(),
            source: AnimationPresetSourceV1::Community,
            authors: contribution.authors.clone(),
            license: contribution.license.clone(),
            tags: contribution.tags.clone(),
            playback: contribution.playback,
            duration_seconds: contribution.animation.duration_seconds,
            rig_profile: contribution.rig_profile.clone(),
            rig_signature_sha256: contribution.rig_signature_sha256.clone(),
            required_bones: contribution.required_bones.clone(),
            animation_path: "animation.json".to_owned(),
            animation_byte_length: animation_bytes.len() as u64,
            animation_sha256: sha256_hex_v1(&animation_bytes),
            motion_sha256: contribution.motion_sha256.clone(),
            preview_path: None,
            preview_byte_length: None,
            preview_sha256: None,
            validation_status: contribution.validation_status,
        },
        animation: contribution.animation.clone(),
        catalog_sha256: "0".repeat(64),
    };
    diagnostics.extend(validate_animation_preset_v1(
        &preset,
        &ANIMATION_LIBRARY_DEFAULT_TAGS_V1
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    ));
    diagnostics
}

pub fn install_animation_contribution_v1(
    root: &Path,
    contribution: &AnimationContributionV1,
) -> Result<std::path::PathBuf, Vec<AnimationLibraryDiagnosticV1>> {
    let diagnostics = validate_animation_contribution_v1(contribution);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let presets_root = root.join("presets");
    let directory_name = if contribution.preset_version == 1 {
        contribution.preset_id.clone()
    } else {
        format!(
            "{}-v{}",
            contribution.preset_id, contribution.preset_version
        )
    };
    let destination = presets_root.join(directory_name);
    if destination.exists() {
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CONTRIBUTION-IMMUTABLE",
            destination.display().to_string(),
            "the exact contribution preset directory already exists",
            "Publish a higher presetVersion; never overwrite a published id and version.",
        )]);
    }
    let temporary = presets_root.join(format!(
        ".m2a-import-{}-{}-v{}",
        contribution.preset_id,
        std::process::id(),
        contribution.preset_version,
    ));
    if temporary.exists() {
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CONTRIBUTION-TEMP",
            temporary.display().to_string(),
            "the exact import staging directory already exists",
            "Remove only the reported stale staging directory after verifying no import owns it.",
        )]);
    }
    fs::create_dir(&temporary).map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CONTRIBUTION-WRITE",
            temporary.display().to_string(),
            format!("cannot create import staging directory: {error}"),
            "Verify the animation-library/presets directory is writable.",
        )]
    })?;
    let write = (|| -> Result<(), std::io::Error> {
        let animation_bytes =
            serde_json::to_vec(&contribution.animation).map_err(std::io::Error::other)?;
        let manifest = AnimationPresetManifestV1 {
            schema_version: 1,
            preset_id: contribution.preset_id.clone(),
            preset_version: contribution.preset_version,
            output_name: contribution.output_name.clone(),
            label: contribution.label.clone(),
            summary: contribution.summary.clone(),
            source: AnimationPresetSourceV1::Community,
            authors: contribution.authors.clone(),
            license: contribution.license.clone(),
            tags: contribution.tags.clone(),
            playback: contribution.playback,
            duration_seconds: contribution.animation.duration_seconds,
            rig_profile: contribution.rig_profile.clone(),
            rig_signature_sha256: contribution.rig_signature_sha256.clone(),
            required_bones: contribution.required_bones.clone(),
            animation_path: "animation.json".to_owned(),
            animation_byte_length: animation_bytes.len() as u64,
            animation_sha256: sha256_hex_v1(&animation_bytes),
            motion_sha256: contribution.motion_sha256.clone(),
            preview_path: None,
            preview_byte_length: None,
            preview_sha256: None,
            validation_status: contribution.validation_status,
        };
        fs::write(temporary.join("animation.json"), animation_bytes)?;
        fs::write(
            temporary.join("manifest.json"),
            serde_json::to_vec(&manifest).map_err(std::io::Error::other)?,
        )?;
        fs::write(
            temporary.join("README.md"),
            format!(
                "# {}\n\n{}\n\nAuthors: {}\nLicense: {}\n",
                contribution.label,
                contribution.summary,
                contribution
                    .authors
                    .iter()
                    .map(|author| author.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                contribution.license,
            ),
        )?;
        fs::rename(&temporary, &destination)
    })();
    if let Err(error) = write {
        let _ = fs::remove_dir_all(&temporary);
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CONTRIBUTION-WRITE",
            destination.display().to_string(),
            format!("cannot materialize the contribution atomically: {error}"),
            "Resolve the reported filesystem error and retry the exact contribution.",
        )]);
    }
    Ok(destination)
}

pub fn build_community_animation_catalog_v1(
    presets: &[AnimationPresetV1],
    source: AnimationLibraryCatalogSourceV1,
) -> Result<CommunityAnimationCatalogV1, Vec<AnimationLibraryDiagnosticV1>> {
    if presets.len() > ANIMATION_LIBRARY_MAX_PRESETS_V1 {
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CATALOG-LIMIT",
            "presets",
            "preset count exceeds the catalog limit",
            "Split or reduce the repository catalog.",
        )]);
    }
    let mut entries = presets
        .iter()
        .map(|preset| preset.manifest.clone())
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.preset_id
            .cmp(&right.preset_id)
            .then(left.preset_version.cmp(&right.preset_version))
    });
    let mut identities = BTreeSet::new();
    for entry in &entries {
        if !identities.insert((entry.preset_id.as_str(), entry.preset_version)) {
            return Err(vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-DUPLICATE",
                format!("{}@{}", entry.preset_id, entry.preset_version),
                "duplicate preset id and version",
                "Publish a new version or choose a unique preset id.",
            )]);
        }
    }
    let unsigned = UnsignedCommunityAnimationCatalogV1 {
        schema_version: COMMUNITY_ANIMATION_CATALOG_SCHEMA_VERSION_V1,
        source,
        entries: &entries,
    };
    let bytes = serde_json::to_vec(&unsigned).map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CATALOG-SERIALIZE",
            "catalog",
            format!("catalog cannot be serialized: {error}"),
            "Repair the exact preset reported by validation.",
        )]
    })?;
    Ok(CommunityAnimationCatalogV1 {
        schema_version: COMMUNITY_ANIMATION_CATALOG_SCHEMA_VERSION_V1,
        source,
        entries,
        catalog_sha256: sha256_hex_v1(&bytes),
    })
}

pub fn verify_community_animation_catalog_v1(
    catalog: &CommunityAnimationCatalogV1,
    presets: &[AnimationPresetV1],
) -> Result<(), Vec<AnimationLibraryDiagnosticV1>> {
    let rebuilt = build_community_animation_catalog_v1(presets, catalog.source)?;
    if &rebuilt == catalog {
        Ok(())
    } else {
        Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CATALOG-STALE",
            "contracts.communityAnimationCatalogV1",
            "tracked catalog differs from the deterministic Core projection",
            "Run the animation library generator in write mode and commit the result.",
        )])
    }
}

pub fn build_community_animation_catalog_from_root_v1(
    root: &Path,
    source: AnimationLibraryCatalogSourceV1,
) -> Result<(CommunityAnimationCatalogV1, Vec<AnimationPresetV1>), Vec<AnimationLibraryDiagnosticV1>>
{
    let tags = read_tags_v1(root)?;
    let presets_root = root.join("presets");
    let directories = fs::read_dir(&presets_root).map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-READ",
            "animation-library/presets",
            format!("cannot read preset root: {error}"),
            "Create the canonical animation-library/presets directory.",
        )]
    })?;
    let mut presets = Vec::new();
    let mut total_bytes = 0_u64;
    for entry in directories {
        let entry = entry.map_err(|error| vec![io_diagnostic("presets", error)])?;
        let file_type = entry
            .file_type()
            .map_err(|error| vec![io_diagnostic("presets", error)])?;
        if !file_type.is_dir() || file_type.is_symlink() {
            return Err(vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-PATH",
                entry.path().display().to_string(),
                "preset root may contain only real directories",
                "Remove files and symlinks from animation-library/presets.",
            )]);
        }
        validate_preset_directory_files_v1(&entry.path())?;
        let manifest_bytes = fs::read(entry.path().join("manifest.json"))
            .map_err(|error| vec![io_diagnostic("manifest.json", error)])?;
        let manifest: AnimationPresetManifestV1 =
            serde_json::from_slice(&manifest_bytes).map_err(|error| {
                vec![diagnostic(
                    "M2A-ANIMATION-LIBRARY-SCHEMA",
                    entry.path().join("manifest.json").display().to_string(),
                    format!("manifest does not match strict V1: {error}"),
                    "Export or regenerate the preset manifest.",
                )]
            })?;
        let expected_directory = if manifest.preset_version == 1 {
            manifest.preset_id.clone()
        } else {
            format!("{}-v{}", manifest.preset_id, manifest.preset_version)
        };
        if entry.file_name().to_string_lossy() != expected_directory {
            return Err(vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-PRESET-PATH",
                entry.path().display().to_string(),
                "preset directory does not match its canonical id and version",
                format!(
                    "Rename the directory to {expected_directory}; never alias a published preset path."
                ),
            )]);
        }
        let animation_path = safe_child_path_v1(&entry.path(), &manifest.animation_path)?;
        let animation_bytes = fs::read(&animation_path)
            .map_err(|error| vec![io_diagnostic(&animation_path.display().to_string(), error)])?;
        total_bytes =
            total_bytes.saturating_add(manifest_bytes.len() as u64 + animation_bytes.len() as u64);
        validate_animation_asset_bytes_v1(&manifest, &animation_bytes)?;
        if let Some(preview_path) = manifest.preview_path.as_deref() {
            let preview_path = safe_child_path_v1(&entry.path(), preview_path)?;
            let preview = fs::read(&preview_path)
                .map_err(|error| vec![io_diagnostic(&preview_path.display().to_string(), error)])?;
            total_bytes = total_bytes.saturating_add(preview.len() as u64);
            if Some(preview.len() as u64) != manifest.preview_byte_length
                || manifest.preview_sha256.as_deref() != Some(sha256_hex_v1(&preview).as_str())
            {
                return Err(vec![diagnostic(
                    "M2A-ANIMATION-LIBRARY-ASSET-HASH",
                    preview_path.display().to_string(),
                    "preview bytes do not match manifest size and SHA-256",
                    "Regenerate the manifest from the exact preview asset.",
                )]);
            }
        }
        let animation: AnimationPresetPayloadV1 = serde_json::from_slice(&animation_bytes)
            .map_err(|error| {
                vec![diagnostic(
                    "M2A-ANIMATION-LIBRARY-SCHEMA",
                    animation_path.display().to_string(),
                    format!("animation does not match strict V1: {error}"),
                    "Export or regenerate the animation payload.",
                )]
            })?;
        let preset = AnimationPresetV1 {
            manifest,
            animation,
            catalog_sha256: "0".repeat(64),
        };
        let errors = validate_animation_preset_v1(&preset, &tags);
        if !errors.is_empty() {
            return Err(errors);
        }
        presets.push(preset);
    }
    if total_bytes > ANIMATION_LIBRARY_MAX_TOTAL_BYTES_V1 {
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-CATALOG-LIMIT",
            "animation-library",
            "catalog payload exceeds the total byte limit",
            "Reduce previews or split a future separately signed release.",
        )]);
    }
    let catalog = build_community_animation_catalog_v1(&presets, source)?;
    for preset in &mut presets {
        preset.catalog_sha256 = catalog.catalog_sha256.clone();
    }
    Ok((catalog, presets))
}

fn read_tags_v1(root: &Path) -> Result<Vec<String>, Vec<AnimationLibraryDiagnosticV1>> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct TagsV1 {
        schema_version: u32,
        tags: Vec<String>,
    }
    let path = root.join("tags-v1.json");
    let bytes =
        fs::read(&path).map_err(|error| vec![io_diagnostic(&path.display().to_string(), error)])?;
    let tags: TagsV1 = serde_json::from_slice(&bytes).map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-TAGS",
            path.display().to_string(),
            format!("tag dictionary does not match strict V1: {error}"),
            "Regenerate the canonical tag dictionary.",
        )]
    })?;
    let stable_tags = ANIMATION_LIBRARY_DEFAULT_TAGS_V1
        .iter()
        .map(|tag| (*tag).to_owned())
        .collect::<Vec<_>>();
    if tags.schema_version != 1 || tags.tags != stable_tags {
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-TAGS",
            path.display().to_string(),
            "tag dictionary must exactly match the stable, sorted Core V1 vocabulary",
            "Restore the canonical V1 tag list or introduce a versioned V2 contract.",
        )]);
    }
    Ok(tags.tags)
}

fn validate_preset_directory_files_v1(
    path: &Path,
) -> Result<(), Vec<AnimationLibraryDiagnosticV1>> {
    let allowed = BTreeSet::from([
        "README.md",
        "animation.json",
        "manifest.json",
        "preview.webp",
    ]);
    let mut found = BTreeSet::new();
    for entry in fs::read_dir(path)
        .map_err(|error| vec![io_diagnostic(&path.display().to_string(), error)])?
    {
        let entry =
            entry.map_err(|error| vec![io_diagnostic(&path.display().to_string(), error)])?;
        let file_type = entry
            .file_type()
            .map_err(|error| vec![io_diagnostic("preset", error)])?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !file_type.is_file() || file_type.is_symlink() || !allowed.contains(name.as_str()) {
            return Err(vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-FILE-TYPE",
                entry.path().display().to_string(),
                "preset contains a directory, symlink, executable, model payload, or unsupported file",
                "Keep only manifest.json, animation.json, README.md and optional preview.webp.",
            )]);
        }
        found.insert(name);
    }
    for required in ["README.md", "animation.json", "manifest.json"] {
        if !found.contains(required) {
            return Err(vec![diagnostic(
                "M2A-ANIMATION-LIBRARY-FILE-MISSING",
                path.join(required).display().to_string(),
                "preset directory is missing a required repository asset",
                format!(
                    "Add the canonical {required} file generated by the contribution workflow."
                ),
            )]);
        }
    }
    Ok(())
}

fn safe_child_path_v1(
    root: &Path,
    relative: &str,
) -> Result<std::path::PathBuf, Vec<AnimationLibraryDiagnosticV1>> {
    let candidate = Path::new(relative);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || relative.contains('\\')
    {
        return Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-PATH-TRAVERSAL",
            relative,
            "asset path must be one normalized relative filename",
            "Use animation.json or preview.webp without parent components.",
        )]);
    }
    Ok(root.join(candidate))
}

fn validate_manifest_shape_v1(
    manifest: &AnimationPresetManifestV1,
    allowed_tags: &[String],
) -> Vec<AnimationLibraryDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != ANIMATION_PRESET_SCHEMA_VERSION_V1
        || manifest.preset_version == 0
        || !is_portable_id(&manifest.preset_id, 64)
        || !is_portable_output_name(&manifest.output_name)
        || manifest.label.trim().is_empty()
        || manifest.label.len() > 120
        || manifest.summary.trim().is_empty()
        || manifest.summary.len() > 500
        || !is_portable_id(&manifest.rig_profile, 64)
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-MANIFEST",
            "manifest",
            "manifest identity, version, labels, output name, or rig profile are invalid",
            "Use schemaVersion 1, a positive version, bounded labels and portable identifiers.",
        ));
    }
    if manifest.authors.is_empty()
        || manifest.authors.len() > 16
        || manifest
            .authors
            .iter()
            .any(|author| author.name.trim().is_empty() || author.name.len() > 120)
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-AUTHOR",
            "manifest.authors",
            "manifest requires one to sixteen bounded author names",
            "Declare every contributor using a non-empty display name.",
        ));
    }
    if !ANIMATION_LIBRARY_DEFAULT_LICENSES_V1.contains(&manifest.license.as_str()) {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-LICENSE",
            "manifest.license",
            "license is not in the approved animation contribution allowlist",
            "Choose an approved SPDX id or the project-generated LicenseRef.",
        ));
    }
    let allowed = allowed_tags
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if manifest.tags.is_empty()
        || manifest.tags.len() > 16
        || !is_sorted_unique(&manifest.tags)
        || manifest
            .tags
            .iter()
            .any(|tag| !allowed.contains(tag.as_str()))
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-TAG",
            "manifest.tags",
            "tags must be sorted, unique and present in tags-v1.json",
            "Choose one to sixteen tags from the stable V1 dictionary and sort them.",
        ));
    }
    if !manifest.duration_seconds.is_finite()
        || manifest.duration_seconds <= 0.0
        || manifest.duration_seconds > ANIMATION_STUDIO_MAX_DURATION_SECONDS
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-DURATION",
            "manifest.durationSeconds",
            "duration is outside the supported range",
            "Use a finite positive duration within the Animation Studio limit.",
        ));
    }
    if !is_sha256(&manifest.rig_signature_sha256)
        || !is_sha256(&manifest.animation_sha256)
        || !is_sha256(&manifest.motion_sha256)
        || manifest.animation_path != "animation.json"
        || manifest.animation_byte_length == 0
        || manifest.animation_byte_length > ANIMATION_LIBRARY_MAX_ANIMATION_BYTES_V1
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-ASSET",
            "manifest.animationPath",
            "animation identity, path, size or hashes are invalid",
            "Regenerate the canonical manifest from animation.json.",
        ));
    }
    let preview_shape = match (
        manifest.preview_path.as_deref(),
        manifest.preview_byte_length,
        manifest.preview_sha256.as_deref(),
    ) {
        (None, None, None) => true,
        (Some("preview.webp"), Some(length), Some(hash)) => {
            length > 0 && length <= ANIMATION_LIBRARY_MAX_PREVIEW_BYTES_V1 && is_sha256(hash)
        }
        _ => false,
    };
    if !preview_shape {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-PREVIEW",
            "manifest.previewPath",
            "preview path, size and hash must be absent together or form one valid preview.webp asset",
            "Remove all preview fields or regenerate them from preview.webp.",
        ));
    }
    if manifest.required_bones.is_empty()
        || manifest.required_bones.len() > ANIMATION_LIBRARY_MAX_TRACKS_V1 + 1
        || !is_sorted_unique(&manifest.required_bones)
        || manifest
            .required_bones
            .iter()
            .any(|bone| bone.trim().is_empty() || bone.len() > 120)
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG-BONES",
            "manifest.requiredBones",
            "required bones must be sorted, unique and bounded",
            "Regenerate required bones from the exact portable track targets.",
        ));
    }
    diagnostics
}

fn validate_animation_asset_bytes_v1(
    manifest: &AnimationPresetManifestV1,
    animation_bytes: &[u8],
) -> Result<(), Vec<AnimationLibraryDiagnosticV1>> {
    if animation_bytes.len() as u64 != manifest.animation_byte_length
        || sha256_hex_v1(animation_bytes) != manifest.animation_sha256
    {
        Err(vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-ASSET-HASH",
            "manifest.animationSha256",
            "animation bytes do not match the declared size and SHA-256",
            "Regenerate the manifest from the exact animation.json bytes.",
        )])
    } else {
        Ok(())
    }
}

fn validate_animation_payload_v1(
    payload: &AnimationPresetPayloadV1,
) -> Vec<AnimationLibraryDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if payload.schema_version != 1
        || !payload.duration_seconds.is_finite()
        || payload.duration_seconds <= 0.0
        || payload.duration_seconds > ANIMATION_STUDIO_MAX_DURATION_SECONDS
        || payload.animation_root_bone_name.trim().is_empty()
        || payload.tracks.is_empty()
        || payload.tracks.len() > ANIMATION_LIBRARY_MAX_TRACKS_V1
        || payload.events.len() > ANIMATION_LIBRARY_MAX_EVENTS_V1
    {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-PAYLOAD",
            "animation",
            "animation schema, duration, root, track count or event count is invalid",
            "Export the animation again from a bounded valid Custom clip.",
        ));
    }
    let mut track_keys = BTreeSet::new();
    let mut total_keyframes = 0usize;
    for (track_index, track) in payload.tracks.iter().enumerate() {
        total_keyframes = total_keyframes.saturating_add(track.keyframes.len());
        if track.target_bone_name.trim().is_empty()
            || !track_keys.insert((track.target_bone_name.as_str(), track.path))
            || track.interpolation != MdlAnimationInterpolationV1::Linear
            || track.keyframes.is_empty()
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-LIBRARY-TRACK",
                format!("animation.tracks[{track_index}]"),
                "track target/path must be unique and use non-empty LINEAR keyframes",
                "Remove duplicate targets and export LINEAR tracks only.",
            ));
        }
        let mut previous = None;
        for (key_index, keyframe) in track.keyframes.iter().enumerate() {
            if !keyframe.time_seconds.is_finite()
                || keyframe.time_seconds < 0.0
                || keyframe.time_seconds > payload.duration_seconds
                || previous.is_some_and(|time| keyframe.time_seconds <= time)
            {
                diagnostics.push(diagnostic(
                    "M2A-ANIMATION-LIBRARY-TIME",
                    format!("animation.tracks[{track_index}].keyframes[{key_index}].timeSeconds"),
                    "keyframe times must be finite, in range and strictly increasing",
                    "Sort, deduplicate and clamp keyframe times.",
                ));
            }
            previous = Some(keyframe.time_seconds);
            let expected = match track.path {
                AuthoredAnimationTrackPathV1::Translation => 3,
                AuthoredAnimationTrackPathV1::Rotation => 4,
            };
            let finite_bounded = keyframe.value.len() == expected
                && keyframe.value.iter().all(|value| {
                    value.is_finite() && value.abs() <= ANIMATION_STUDIO_MAX_ABSOLUTE_TRACK_VALUE
                });
            let unit_rotation = track.path != AuthoredAnimationTrackPathV1::Rotation
                || (finite_bounded
                    && quaternion_norm(&keyframe.value)
                        .is_some_and(|norm| (norm - 1.0).abs() <= 1.0e-4));
            if !finite_bounded || !unit_rotation {
                diagnostics.push(diagnostic(
                    "M2A-ANIMATION-LIBRARY-VALUE",
                    format!("animation.tracks[{track_index}].keyframes[{key_index}].value"),
                    "track value has the wrong width, is non-finite/out-of-range, or is not a unit quaternion",
                    "Export finite translations and normalized quaternion rotations.",
                ));
            }
        }
    }
    if total_keyframes > ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-KEYFRAME-LIMIT",
            "animation.tracks",
            "animation exceeds the per-preset keyframe limit",
            "Reduce the sample rate or split the motion.",
        ));
    }
    for (index, event) in payload.events.iter().enumerate() {
        if !event.time_seconds.is_finite()
            || event.time_seconds < 0.0
            || event.time_seconds > payload.duration_seconds
            || event.name.is_empty()
            || event.name.len() > 31
            || !event.name.is_ascii()
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-LIBRARY-EVENT",
                format!("animation.events[{index}]"),
                "event must have an in-range time and a 1..=31 byte ASCII name",
                "Repair or remove the event.",
            ));
        }
    }
    diagnostics
}

fn canonical_rig_rows_v1(
    rig: &AnimationStudioRigV1,
) -> Result<(String, Vec<AnimationRigProfileNodeV1>), Vec<AnimationLibraryDiagnosticV1>> {
    let mut diagnostics = Vec::new();
    if rig.schema_version != 1 || rig.nodes.is_empty() {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG",
            "rig",
            "rig requires schemaVersion 1 and at least one node",
            "Re-inspect the exact current source GLB.",
        ));
    }
    let mut names = BTreeSet::new();
    let mut ids = BTreeMap::new();
    for node in &rig.nodes {
        if node.name.trim().is_empty()
            || !names.insert(node.name.as_str())
            || ids.insert(node.node_id, node.name.as_str()).is_some()
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-LIBRARY-RIG",
                "rig.nodes",
                "rig node ids and names must be non-empty and unique",
                "Repair the exact source rig inspection.",
            ));
        }
    }
    if !names.contains(rig.animation_root.as_str()) {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG-ROOT",
            "rig.animationRoot",
            "animation root is not present in the rig",
            "Re-inspect the exact source rig.",
        ));
    }
    let mut nodes = Vec::new();
    for node in &rig.nodes {
        let parent_name = match node.parent_id {
            Some(parent_id) => match ids.get(&parent_id) {
                Some(parent_name) if parent_id != node.node_id => Some((*parent_name).to_owned()),
                _ => {
                    diagnostics.push(diagnostic(
                        "M2A-ANIMATION-LIBRARY-RIG-PARENT",
                        format!("rig.nodes[{}].parentId", node.node_id),
                        "rig parent is missing or self-referential",
                        "Repair the exact source hierarchy.",
                    ));
                    None
                }
            },
            None => None,
        };
        let finite = node.translation.iter().all(|value| value.is_finite())
            && node.rotation.iter().all(|value| value.is_finite());
        let Some(rotation) = canonical_unit_quaternion(node.rotation) else {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-LIBRARY-RIG-REST",
                format!("rig.nodes[{}]", node.node_id),
                "rest transform is non-finite or has an invalid rotation",
                "Repair the source rest pose before authoring portable animation.",
            ));
            continue;
        };
        if !finite {
            continue;
        }
        nodes.push(AnimationRigProfileNodeV1 {
            name: node.name.clone(),
            parent_name,
            translation: node.translation.map(canonical_rig_f32_v1),
            rotation: rotation.map(canonical_rig_f32_v1),
        });
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    nodes.sort_by(|left, right| left.name.cmp(&right.name));
    Ok((rig.animation_root.clone(), nodes))
}

fn rig_rows_sha256_v1(
    animation_root_bone_name: &str,
    nodes: &[AnimationRigProfileNodeV1],
) -> Result<String, Vec<AnimationLibraryDiagnosticV1>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct UnsignedRig<'a> {
        schema_version: u32,
        animation_root_bone_name: &'a str,
        nodes: &'a [AnimationRigProfileNodeV1],
    }
    serde_json::to_vec(&UnsignedRig {
        schema_version: 1,
        animation_root_bone_name,
        nodes,
    })
    .map(|bytes| sha256_hex_v1(&bytes))
    .map_err(|error| {
        vec![diagnostic(
            "M2A-ANIMATION-LIBRARY-RIG-SERIALIZE",
            "rig",
            format!("rig signature cannot be serialized: {error}"),
            "Repair the exact source rig.",
        )]
    })
}

fn canonical_track_value_v1(path: AuthoredAnimationTrackPathV1, value: &[f32]) -> Vec<f32> {
    match path {
        AuthoredAnimationTrackPathV1::Translation => {
            value.iter().copied().map(canonical_f32).collect()
        }
        AuthoredAnimationTrackPathV1::Rotation => {
            if value.len() == 4 {
                canonical_unit_quaternion([value[0], value[1], value[2], value[3]])
                    .map(Vec::from)
                    .unwrap_or_else(|| value.to_vec())
            } else {
                value.to_vec()
            }
        }
    }
}

fn canonical_unit_quaternion(value: [f32; 4]) -> Option<[f32; 4]> {
    if value.iter().any(|component| !component.is_finite()) {
        return None;
    }
    let norm = quaternion_norm(&value)?;
    if norm <= f32::EPSILON {
        return None;
    }
    let mut normalized = value.map(|component| canonical_f32(component / norm));
    let first_non_zero = [normalized[3], normalized[2], normalized[1], normalized[0]]
        .into_iter()
        .find(|component| component.abs() > 1.0e-7)
        .unwrap_or(1.0);
    if first_non_zero < 0.0 {
        normalized = normalized.map(|component| canonical_f32(-component));
    }
    Some(normalized)
}

fn quaternion_norm(value: &[f32]) -> Option<f32> {
    if value.len() != 4 || value.iter().any(|component| !component.is_finite()) {
        return None;
    }
    Some(
        value
            .iter()
            .map(|component| component * component)
            .sum::<f32>()
            .sqrt(),
    )
}

fn canonical_f32(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

fn canonical_rig_f32_v1(value: f32) -> f32 {
    let quantized = ((value as f64 * ANIMATION_RIG_SIGNATURE_QUANTIZATION_SCALE_V1).round()
        / ANIMATION_RIG_SIGNATURE_QUANTIZATION_SCALE_V1) as f32;
    canonical_f32(quantized)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_portable_id(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn is_portable_output_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
}

fn io_diagnostic(path: &str, error: std::io::Error) -> AnimationLibraryDiagnosticV1 {
    diagnostic(
        "M2A-ANIMATION-LIBRARY-READ",
        path,
        format!("cannot read animation library asset: {error}"),
        "Restore the exact tracked library file and retry.",
    )
}

fn diagnostic(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
    action: impl Into<String>,
) -> AnimationLibraryDiagnosticV1 {
    AnimationLibraryDiagnosticV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
        action: action.into(),
    }
}
