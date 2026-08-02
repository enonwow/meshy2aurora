//! Deterministic non-destructive animation layers. Layers are editor data;
//! the confirmed Aurora/MDL boundary still receives ordinary LINEAR tracks.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_studio::{
        AnimationKeyframeV1, AnimationStudioRigV1, AuthoredAnimationClipStatusV1,
        AuthoredAnimationClipV1, AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1,
        sample_animation_track_linear_v1, validate_authored_animation_clip_v1,
    },
    mdl::MdlAnimationInterpolationV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationLayerModeV1 {
    Base,
    Additive,
    Override,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationBoneMaskV1 {
    pub schema_version: u32,
    pub node_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEditLayerV1 {
    pub schema_version: u32,
    pub id: String,
    pub stable_order: u32,
    pub mode: AnimationLayerModeV1,
    pub weight: f32,
    pub mute: bool,
    pub solo: bool,
    pub bone_mask: AnimationBoneMaskV1,
    pub clip: AuthoredAnimationClipV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationLayerBakeReportV1 {
    pub schema_version: u32,
    pub active_layer_ids: Vec<String>,
    pub key_count_before: usize,
    pub key_count_after: usize,
    pub clip: AuthoredAnimationClipV1,
    pub fingerprint_sha256: String,
}

pub fn bake_animation_layers_v1(
    layers: &[AnimationEditLayerV1],
    rig: &AnimationStudioRigV1,
) -> Result<AnimationLayerBakeReportV1, String> {
    if layers.is_empty() {
        return Err("animation layer stack must not be empty".to_owned());
    }
    let rig_nodes = rig
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut orders = BTreeSet::new();
    for layer in layers {
        if layer.schema_version != 1
            || layer.bone_mask.schema_version != 1
            || layer.id.trim().is_empty()
            || !ids.insert(layer.id.as_str())
            || !orders.insert(layer.stable_order)
            || !layer.weight.is_finite()
            || !(0.0..=1.0).contains(&layer.weight)
            || layer
                .bone_mask
                .node_ids
                .iter()
                .any(|node_id| !rig_nodes.contains_key(node_id))
        {
            return Err(
                "layer stack contains an invalid schema, identity, order, weight or bone mask"
                    .to_owned(),
            );
        }
        if layer.clip.source.source_revision != rig.source_revision {
            return Err(format!(
                "layer {:?} does not match the exact rig source revision",
                layer.id
            ));
        }
    }
    let has_solo = layers.iter().any(|layer| layer.solo && !layer.mute);
    let mut active = layers
        .iter()
        .filter(|layer| {
            !layer.mute && (!has_solo || layer.solo || layer.mode == AnimationLayerModeV1::Base)
        })
        .collect::<Vec<_>>();
    active.sort_by_key(|layer| layer.stable_order);
    let base_layers = active
        .iter()
        .filter(|layer| layer.mode == AnimationLayerModeV1::Base)
        .copied()
        .collect::<Vec<_>>();
    if base_layers.len() != 1 {
        return Err("active layer stack must contain exactly one BASE layer".to_owned());
    }
    let base = base_layers[0];
    let key_count_before = active
        .iter()
        .flat_map(|layer| &layer.clip.tracks)
        .map(|track| track.keyframes.len())
        .sum();
    if active.len() == 1 && base.weight == 1.0 {
        let clip = base.clip.clone();
        let fingerprint_sha256 = fingerprint_json(&(
            active
                .iter()
                .map(|layer| layer.id.as_str())
                .collect::<Vec<_>>(),
            &clip,
        ));
        return Ok(AnimationLayerBakeReportV1 {
            schema_version: 1,
            active_layer_ids: vec![base.id.clone()],
            key_count_before,
            key_count_after: clip.tracks.iter().map(|track| track.keyframes.len()).sum(),
            clip,
            fingerprint_sha256,
        });
    }

    let mut pairs = BTreeSet::new();
    for layer in &active {
        for track in &layer.clip.tracks {
            if layer.bone_mask.node_ids.is_empty()
                || layer.bone_mask.node_ids.contains(&track.target_node_id)
            {
                pairs.insert((track.target_node_id, track.path));
            }
        }
    }
    let mut tracks = Vec::new();
    for (node_id, path) in pairs {
        let node = rig_nodes[&node_id];
        let mut times = active
            .iter()
            .flat_map(|layer| {
                layer
                    .clip
                    .tracks
                    .iter()
                    .filter(move |track| track.target_node_id == node_id && track.path == path)
                    .flat_map(|track| track.keyframes.iter().map(|key| key.time_seconds))
            })
            .collect::<Vec<_>>();
        times.sort_by(f32::total_cmp);
        times.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-7);
        let mut keyframes = Vec::with_capacity(times.len());
        for (index, time) in times.into_iter().enumerate() {
            let mut value =
                sample_layer_value(base, node_id, path, time)?.unwrap_or_else(|| match path {
                    AuthoredAnimationTrackPathV1::Translation => node.translation.to_vec(),
                    AuthoredAnimationTrackPathV1::Rotation => node.rotation.to_vec(),
                });
            for layer in &active {
                if layer.id == base.id
                    || (!layer.bone_mask.node_ids.is_empty()
                        && !layer.bone_mask.node_ids.contains(&node_id))
                {
                    continue;
                }
                let Some(layer_value) = sample_layer_value(layer, node_id, path, time)? else {
                    continue;
                };
                value = apply_layer_value(path, &value, &layer_value, layer.mode, layer.weight)?;
            }
            keyframes.push(AnimationKeyframeV1 {
                id: format!("layer-{node_id}-{}-{index:05}", path_label(path)),
                time_seconds: canonical_f32(time),
                value: value.into_iter().map(canonical_f32).collect(),
            });
        }
        tracks.push(AuthoredAnimationTrackV1 {
            id: format!("layer-{node_id}-{}", path_label(path)),
            target_node_id: node_id,
            path,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes,
        });
    }
    let mut clip = base.clip.clone();
    clip.status = AuthoredAnimationClipStatusV1::Draft;
    clip.revision = clip.revision.saturating_add(1);
    clip.tracks = tracks;
    if let Some(diagnostic) = validate_authored_animation_clip_v1(&clip, rig).first() {
        return Err(format!("{}: {}", diagnostic.path, diagnostic.message));
    }
    let key_count_after = clip.tracks.iter().map(|track| track.keyframes.len()).sum();
    let active_layer_ids = active
        .iter()
        .map(|layer| layer.id.clone())
        .collect::<Vec<_>>();
    let fingerprint_sha256 = fingerprint_json(&(&active_layer_ids, &clip));
    Ok(AnimationLayerBakeReportV1 {
        schema_version: 1,
        active_layer_ids,
        key_count_before,
        key_count_after,
        clip,
        fingerprint_sha256,
    })
}

fn sample_layer_value(
    layer: &AnimationEditLayerV1,
    node_id: u32,
    path: AuthoredAnimationTrackPathV1,
    time: f32,
) -> Result<Option<Vec<f32>>, String> {
    layer
        .clip
        .tracks
        .iter()
        .find(|track| track.target_node_id == node_id && track.path == path)
        .map(|track| {
            sample_animation_track_linear_v1(track, time)
                .map_err(|diagnostic| format!("{}: {}", diagnostic.path, diagnostic.message))
        })
        .transpose()
}

fn apply_layer_value(
    path: AuthoredAnimationTrackPathV1,
    base: &[f32],
    layer: &[f32],
    mode: AnimationLayerModeV1,
    weight: f32,
) -> Result<Vec<f32>, String> {
    if mode == AnimationLayerModeV1::Base {
        return Err("only the first active layer may use BASE mode".to_owned());
    }
    match path {
        AuthoredAnimationTrackPathV1::Translation => {
            let base: [f32; 3] = base
                .try_into()
                .map_err(|_| "translation layer value must contain three numbers".to_owned())?;
            let layer: [f32; 3] = layer
                .try_into()
                .map_err(|_| "translation layer value must contain three numbers".to_owned())?;
            Ok(match mode {
                AnimationLayerModeV1::Additive => [
                    base[0] + layer[0] * weight,
                    base[1] + layer[1] * weight,
                    base[2] + layer[2] * weight,
                ],
                AnimationLayerModeV1::Override => [
                    base[0] + (layer[0] - base[0]) * weight,
                    base[1] + (layer[1] - base[1]) * weight,
                    base[2] + (layer[2] - base[2]) * weight,
                ],
                AnimationLayerModeV1::Base => unreachable!(),
            }
            .to_vec())
        }
        AuthoredAnimationTrackPathV1::Rotation => {
            let base = normalized_quaternion(base)?;
            let layer = normalized_quaternion(layer)?;
            let value = match mode {
                AnimationLayerModeV1::Additive => {
                    multiply_quaternion(base, nlerp([0.0, 0.0, 0.0, 1.0], layer, weight))
                }
                AnimationLayerModeV1::Override => nlerp(base, layer, weight),
                AnimationLayerModeV1::Base => unreachable!(),
            };
            Ok(value.to_vec())
        }
    }
}

fn nlerp(left: [f32; 4], mut right: [f32; 4], amount: f32) -> [f32; 4] {
    if left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>() < 0.0 {
        right = right.map(|value| -value);
    }
    normalize([
        left[0] + (right[0] - left[0]) * amount,
        left[1] + (right[1] - left[1]) * amount,
        left[2] + (right[2] - left[2]) * amount,
        left[3] + (right[3] - left[3]) * amount,
    ])
}

fn normalized_quaternion(value: &[f32]) -> Result<[f32; 4], String> {
    let value: [f32; 4] = value
        .try_into()
        .map_err(|_| "rotation layer value must contain four numbers".to_owned())?;
    if value.iter().any(|component| !component.is_finite()) {
        return Err("rotation layer quaternion must be finite".to_owned());
    }
    let length = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if length <= 1.0e-8 {
        return Err("rotation layer quaternion must be non-zero".to_owned());
    }
    Ok(value.map(|component| component / length))
}

fn normalize(value: [f32; 4]) -> [f32; 4] {
    let length = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    value.map(|component| component / length.max(1.0e-8))
}

fn multiply_quaternion(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    normalize([
        left[3] * right[0] + left[0] * right[3] + left[1] * right[2] - left[2] * right[1],
        left[3] * right[1] - left[0] * right[2] + left[1] * right[3] + left[2] * right[0],
        left[3] * right[2] + left[0] * right[1] - left[1] * right[0] + left[2] * right[3],
        left[3] * right[3] - left[0] * right[0] - left[1] * right[1] - left[2] * right[2],
    ])
}

fn path_label(path: AuthoredAnimationTrackPathV1) -> &'static str {
    match path {
        AuthoredAnimationTrackPathV1::Translation => "translation",
        AuthoredAnimationTrackPathV1::Rotation => "rotation",
    }
}

fn canonical_f32(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

fn fingerprint_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("serializable animation layer value");
    format!("{:x}", Sha256::digest(bytes))
}
