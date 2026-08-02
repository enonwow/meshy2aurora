//! Revision-bound pose authoring tools shared by the viewport, combat phase
//! editor and procedural variant pipeline. Poses are local-rig transforms;
//! world-space sampling remains owned by `animation_authoring_v2`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_retarget::rig_signature_sha256_v1,
    animation_studio::{
        AnimationKeyframeV1, AnimationStudioRigV1, AuthoredAnimationClipStatusV1,
        AuthoredAnimationClipV1, AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1,
        sample_animation_track_linear_v1,
    },
    mdl::MdlAnimationInterpolationV1,
};

pub const ANIMATION_POSE_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationPosePasteModeV1 {
    WholeBody,
    SelectedBones,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationMirrorAxisV1 {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoseBoneV1 {
    pub node_id: u32,
    pub node_name: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoseSnapshotV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub rig_signature_sha256: String,
    pub clip_id: String,
    pub clip_revision: u64,
    pub time_seconds: f32,
    pub bones: Vec<AnimationPoseBoneV1>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanoidMirrorPairV1 {
    pub left_node_id: u32,
    pub right_node_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanoidMirrorMapV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub rig_signature_sha256: String,
    pub axis: AnimationMirrorAxisV1,
    pub pairs: Vec<HumanoidMirrorPairV1>,
    pub center_node_ids: Vec<u32>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoseApplyResultV1 {
    pub schema_version: u32,
    pub mode: AnimationPosePasteModeV1,
    pub changed_node_ids: Vec<u32>,
    pub clip: AuthoredAnimationClipV1,
    pub fingerprint_sha256: String,
}

pub fn capture_animation_pose_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    time_seconds: f32,
    selected_node_ids: &[u32],
) -> Result<AnimationPoseSnapshotV1, String> {
    validate_clip_rig_time(clip, rig, time_seconds)?;
    let selection = selected_node_ids.iter().copied().collect::<BTreeSet<_>>();
    if selection.len() != selected_node_ids.len() {
        return Err("pose selection contains duplicate node ids".into());
    }
    let mut bones = rig
        .nodes
        .iter()
        .filter(|node| selection.is_empty() || selection.contains(&node.node_id))
        .map(|node| {
            let translation = sample_local_track(
                clip,
                node.node_id,
                AuthoredAnimationTrackPathV1::Translation,
                time_seconds,
            )?
            .map(vec3)
            .transpose()?
            .unwrap_or(node.translation);
            let rotation = sample_local_track(
                clip,
                node.node_id,
                AuthoredAnimationTrackPathV1::Rotation,
                time_seconds,
            )?
            .map(vec4)
            .transpose()?
            .map(normalize_quaternion)
            .transpose()?
            .unwrap_or(node.rotation);
            Ok(AnimationPoseBoneV1 {
                node_id: node.node_id,
                node_name: node.name.clone(),
                translation,
                rotation,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    bones.sort_by_key(|bone| bone.node_id);
    if bones.len()
        != if selection.is_empty() {
            rig.nodes.len()
        } else {
            selection.len()
        }
    {
        return Err("pose selection references a node absent from the exact rig".into());
    }
    let rig_signature_sha256 = rig_signature_sha256_v1(rig);
    let fingerprint_sha256 = fingerprint_json(&(
        ANIMATION_POSE_SCHEMA_VERSION_V1,
        rig.source_revision.as_str(),
        rig_signature_sha256.as_str(),
        clip.id.as_str(),
        clip.revision,
        canonical_f32(time_seconds),
        &bones,
    ));
    Ok(AnimationPoseSnapshotV1 {
        schema_version: ANIMATION_POSE_SCHEMA_VERSION_V1,
        source_revision: rig.source_revision.clone(),
        rig_signature_sha256,
        clip_id: clip.id.clone(),
        clip_revision: clip.revision,
        time_seconds: canonical_f32(time_seconds),
        bones,
        fingerprint_sha256,
    })
}

pub fn build_humanoid_mirror_map_v1(
    rig: &AnimationStudioRigV1,
    axis: AnimationMirrorAxisV1,
    pairs: Vec<HumanoidMirrorPairV1>,
    mut center_node_ids: Vec<u32>,
) -> Result<HumanoidMirrorMapV1, String> {
    let known = rig
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<BTreeSet<_>>();
    let mut used = BTreeSet::new();
    for pair in &pairs {
        if pair.left_node_id == pair.right_node_id
            || !known.contains(&pair.left_node_id)
            || !known.contains(&pair.right_node_id)
            || !used.insert(pair.left_node_id)
            || !used.insert(pair.right_node_id)
        {
            return Err(
                "mirror pairs must contain unique, distinct nodes from the exact rig".into(),
            );
        }
    }
    center_node_ids.sort_unstable();
    center_node_ids.dedup();
    if center_node_ids
        .iter()
        .any(|id| !known.contains(id) || !used.insert(*id))
    {
        return Err("mirror center nodes must be unique exact-rig nodes outside side pairs".into());
    }
    let rig_signature_sha256 = rig_signature_sha256_v1(rig);
    let fingerprint_sha256 = fingerprint_json(&(
        ANIMATION_POSE_SCHEMA_VERSION_V1,
        rig.source_revision.as_str(),
        rig_signature_sha256.as_str(),
        axis,
        &pairs,
        &center_node_ids,
    ));
    Ok(HumanoidMirrorMapV1 {
        schema_version: ANIMATION_POSE_SCHEMA_VERSION_V1,
        source_revision: rig.source_revision.clone(),
        rig_signature_sha256,
        axis,
        pairs,
        center_node_ids,
        fingerprint_sha256,
    })
}

pub fn mirror_animation_pose_v1(
    pose: &AnimationPoseSnapshotV1,
    rig: &AnimationStudioRigV1,
    mapping: &HumanoidMirrorMapV1,
) -> Result<AnimationPoseSnapshotV1, String> {
    validate_pose_identity(pose, rig)?;
    if mapping.schema_version != 1
        || mapping.source_revision != rig.source_revision
        || mapping.rig_signature_sha256 != rig_signature_sha256_v1(rig)
    {
        return Err("mirror map is stale for the exact rig".into());
    }
    let source = pose
        .bones
        .iter()
        .map(|bone| (bone.node_id, bone))
        .collect::<BTreeMap<_, _>>();
    let mut source_by_target = BTreeMap::new();
    for pair in &mapping.pairs {
        source_by_target.insert(pair.left_node_id, pair.right_node_id);
        source_by_target.insert(pair.right_node_id, pair.left_node_id);
    }
    for id in &mapping.center_node_ids {
        source_by_target.insert(*id, *id);
    }
    let mut bones = Vec::with_capacity(pose.bones.len());
    for original in &pose.bones {
        let donor_id = source_by_target
            .get(&original.node_id)
            .copied()
            .unwrap_or(original.node_id);
        let donor = source
            .get(&donor_id)
            .copied()
            .ok_or_else(|| format!("mirror pose does not contain paired node {donor_id}"))?;
        bones.push(AnimationPoseBoneV1 {
            node_id: original.node_id,
            node_name: original.node_name.clone(),
            translation: reflect_pose_translation_v1(donor.translation, mapping.axis),
            rotation: reflect_pose_rotation_v1(donor.rotation, mapping.axis),
        });
    }
    bones.sort_by_key(|bone| bone.node_id);
    let fingerprint_sha256 = fingerprint_json(&(
        pose.source_revision.as_str(),
        pose.rig_signature_sha256.as_str(),
        pose.clip_id.as_str(),
        pose.clip_revision,
        pose.time_seconds,
        mapping.fingerprint_sha256.as_str(),
        &bones,
    ));
    Ok(AnimationPoseSnapshotV1 {
        bones,
        fingerprint_sha256,
        ..pose.clone()
    })
}

pub fn apply_animation_pose_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    pose: &AnimationPoseSnapshotV1,
    time_seconds: f32,
    mode: AnimationPosePasteModeV1,
    selected_node_ids: &[u32],
) -> Result<AnimationPoseApplyResultV1, String> {
    validate_clip_rig_time(clip, rig, time_seconds)?;
    validate_pose_identity(pose, rig)?;
    let selected = selected_node_ids.iter().copied().collect::<BTreeSet<_>>();
    if selected.len() != selected_node_ids.len() {
        return Err("paste selection contains duplicate node ids".into());
    }
    if mode == AnimationPosePasteModeV1::SelectedBones && selected.is_empty() {
        return Err("SELECTED_BONES paste requires at least one node".into());
    }
    let known = rig
        .nodes
        .iter()
        .map(|node| (node.node_id, node.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut output = clip.clone();
    let mut changed_node_ids = Vec::new();
    for bone in &pose.bones {
        if mode == AnimationPosePasteModeV1::SelectedBones && !selected.contains(&bone.node_id) {
            continue;
        }
        if known.get(&bone.node_id).copied() != Some(bone.node_name.as_str()) {
            return Err(format!(
                "pose node {} has stale exact-rig identity",
                bone.node_id
            ));
        }
        upsert_pose_key(
            &mut output,
            bone.node_id,
            AuthoredAnimationTrackPathV1::Translation,
            time_seconds,
            bone.translation.to_vec(),
        )?;
        upsert_pose_key(
            &mut output,
            bone.node_id,
            AuthoredAnimationTrackPathV1::Rotation,
            time_seconds,
            normalize_quaternion(bone.rotation)?.to_vec(),
        )?;
        changed_node_ids.push(bone.node_id);
    }
    if changed_node_ids.is_empty() {
        return Err("pose operation selected no bones present in the snapshot".into());
    }
    changed_node_ids.sort_unstable();
    output.revision = output.revision.saturating_add(1);
    output.status = AuthoredAnimationClipStatusV1::Draft;
    let fingerprint_sha256 = fingerprint_json(&(
        pose.fingerprint_sha256.as_str(),
        canonical_f32(time_seconds),
        mode,
        &changed_node_ids,
        &output,
    ));
    Ok(AnimationPoseApplyResultV1 {
        schema_version: 1,
        mode,
        changed_node_ids,
        clip: output,
        fingerprint_sha256,
    })
}

pub fn reset_animation_pose_to_rest_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    time_seconds: f32,
    selected_node_ids: &[u32],
) -> Result<AnimationPoseApplyResultV1, String> {
    if selected_node_ids.is_empty() {
        return Err("reset requires at least one selected node".into());
    }
    let selected = selected_node_ids.iter().copied().collect::<BTreeSet<_>>();
    let bones = rig
        .nodes
        .iter()
        .filter(|node| selected.contains(&node.node_id))
        .map(|node| AnimationPoseBoneV1 {
            node_id: node.node_id,
            node_name: node.name.clone(),
            translation: node.translation,
            rotation: node.rotation,
        })
        .collect::<Vec<_>>();
    if bones.len() != selected.len() {
        return Err("reset selection contains a node absent from the exact rig".into());
    }
    let rig_signature_sha256 = rig_signature_sha256_v1(rig);
    let pose = AnimationPoseSnapshotV1 {
        schema_version: 1,
        source_revision: rig.source_revision.clone(),
        rig_signature_sha256: rig_signature_sha256.clone(),
        clip_id: clip.id.clone(),
        clip_revision: clip.revision,
        time_seconds,
        fingerprint_sha256: fingerprint_json(&(rig_signature_sha256.as_str(), &bones)),
        bones,
    };
    apply_animation_pose_v1(
        clip,
        rig,
        &pose,
        time_seconds,
        AnimationPosePasteModeV1::SelectedBones,
        selected_node_ids,
    )
}

fn validate_clip_rig_time(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    time_seconds: f32,
) -> Result<(), String> {
    if clip.source.source_revision != rig.source_revision
        || !time_seconds.is_finite()
        || time_seconds < 0.0
        || time_seconds > clip.length_seconds
    {
        return Err("clip, exact rig and pose time do not share one valid lineage".into());
    }
    Ok(())
}

fn validate_pose_identity(
    pose: &AnimationPoseSnapshotV1,
    rig: &AnimationStudioRigV1,
) -> Result<(), String> {
    if pose.schema_version != 1
        || pose.source_revision != rig.source_revision
        || pose.rig_signature_sha256 != rig_signature_sha256_v1(rig)
        || pose.bones.is_empty()
    {
        return Err("pose snapshot is empty or stale for the exact rig".into());
    }
    Ok(())
}

fn sample_local_track(
    clip: &AuthoredAnimationClipV1,
    node_id: u32,
    path: AuthoredAnimationTrackPathV1,
    time_seconds: f32,
) -> Result<Option<Vec<f32>>, String> {
    clip.tracks
        .iter()
        .find(|track| track.target_node_id == node_id && track.path == path)
        .map(|track| {
            sample_animation_track_linear_v1(track, time_seconds).map_err(|error| error.message)
        })
        .transpose()
}

fn upsert_pose_key(
    clip: &mut AuthoredAnimationClipV1,
    node_id: u32,
    path: AuthoredAnimationTrackPathV1,
    time_seconds: f32,
    value: Vec<f32>,
) -> Result<(), String> {
    let track_index = clip
        .tracks
        .iter()
        .position(|track| track.target_node_id == node_id && track.path == path);
    if let Some(index) = track_index {
        let track = &mut clip.tracks[index];
        if track.interpolation != MdlAnimationInterpolationV1::Linear {
            return Err("pose tools require LINEAR output tracks".into());
        }
        if let Some(key) = track
            .keyframes
            .iter_mut()
            .find(|key| (key.time_seconds - time_seconds).abs() <= 1.0e-6)
        {
            key.value = value;
        } else {
            track.keyframes.push(AnimationKeyframeV1 {
                id: format!(
                    "pose-{node_id}-{}-{:08x}",
                    path_label(path),
                    time_seconds.to_bits()
                ),
                time_seconds: canonical_f32(time_seconds),
                value,
            });
            track
                .keyframes
                .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
        }
    } else {
        clip.tracks.push(AuthoredAnimationTrackV1 {
            id: format!("pose-{node_id}-{}", path_label(path)),
            target_node_id: node_id,
            path,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: vec![AnimationKeyframeV1 {
                id: format!(
                    "pose-{node_id}-{}-{:08x}",
                    path_label(path),
                    time_seconds.to_bits()
                ),
                time_seconds: canonical_f32(time_seconds),
                value,
            }],
        });
        clip.tracks
            .sort_by_key(|track| (track.target_node_id, track.path));
    }
    Ok(())
}

pub fn reflect_pose_translation_v1(mut value: [f32; 3], axis: AnimationMirrorAxisV1) -> [f32; 3] {
    value[axis_index(axis)] = -value[axis_index(axis)];
    value.map(canonical_f32)
}

/// For an improper reflection matrix S, mirrored orientation is S * R * S.
/// Quaternion vector components therefore transform as det(S) * S * v.
pub fn reflect_pose_rotation_v1(mut value: [f32; 4], axis: AnimationMirrorAxisV1) -> [f32; 4] {
    for component in &mut value[..3] {
        *component = -*component;
    }
    value[axis_index(axis)] = -value[axis_index(axis)];
    canonical_quaternion(value).unwrap_or([0.0, 0.0, 0.0, 1.0])
}

fn canonical_quaternion(value: [f32; 4]) -> Result<[f32; 4], String> {
    let mut value = normalize_quaternion(value)?;
    let pivot = [value[3], value[2], value[1], value[0]]
        .into_iter()
        .find(|component| component.abs() > 1.0e-7)
        .unwrap_or(1.0);
    if pivot < 0.0 {
        value = value.map(|component| -component);
    }
    Ok(value.map(canonical_f32))
}

fn normalize_quaternion(value: [f32; 4]) -> Result<[f32; 4], String> {
    let norm = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= 1.0e-8 {
        return Err("pose rotation must be a finite non-zero quaternion".into());
    }
    Ok(value.map(|component| component / norm))
}

fn vec3(value: Vec<f32>) -> Result<[f32; 3], String> {
    value
        .try_into()
        .map_err(|_| "translation track must contain three values".into())
}

fn vec4(value: Vec<f32>) -> Result<[f32; 4], String> {
    value
        .try_into()
        .map_err(|_| "rotation track must contain four values".into())
}

fn axis_index(axis: AnimationMirrorAxisV1) -> usize {
    match axis {
        AnimationMirrorAxisV1::X => 0,
        AnimationMirrorAxisV1::Y => 1,
        AnimationMirrorAxisV1::Z => 2,
    }
}

fn path_label(path: AuthoredAnimationTrackPathV1) -> &'static str {
    match path {
        AuthoredAnimationTrackPathV1::Translation => "translation",
        AuthoredAnimationTrackPathV1::Rotation => "rotation",
    }
}

fn canonical_f32(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        f32::from_bits(value.to_bits())
    }
}

fn fingerprint_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("animation pose contracts serialize");
    format!("{:x}", Sha256::digest(bytes))
}
