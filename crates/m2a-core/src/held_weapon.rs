//! Held-weapon authoring contract for direct creatures. This module owns
//! immutable source identity, hand resolution, local attachment transforms
//! and grip measurements. Geometry baking remains an explicit model-pipeline
//! operation and is never represented as runtime equipment swapping.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_authoring_v2::sample_animation_world_pose_v1,
    animation_studio::{AnimationStudioRigNodeV1, AnimationStudioRigV1, AuthoredAnimationClipV1},
    glb::{GlbLimits, ingest_glb},
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1,
        AuroraSegmentDeformationV1,
    },
    model_limits::validate_model_triangle_budget_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeldWeaponModeV1 {
    BakedDirectCreature,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeldWeaponHandV1 {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeldWeaponPivotPolicyV1 {
    SourceOrigin,
    BoundsCenter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeldWeaponMaterialPolicyV1 {
    PreserveSource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldWeaponSourceV1 {
    pub schema_version: u32,
    pub filename: String,
    pub byte_size: usize,
    pub sha256: String,
    pub provenance: String,
    pub triangle_count: usize,
    pub material_count: usize,
    pub texture_count: usize,
    pub bounds_min: Option<[f32; 3]>,
    pub bounds_max: Option<[f32; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldWeaponLocalTransformV1 {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for HeldWeaponLocalTransformV1 {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldWeaponAttachmentV1 {
    pub schema_version: u32,
    pub mode: HeldWeaponModeV1,
    pub source: HeldWeaponSourceV1,
    pub primary_hand: HeldWeaponHandV1,
    pub target_node_id: u32,
    pub target_node_name: String,
    pub local_transform: HeldWeaponLocalTransformV1,
    pub pivot_policy: HeldWeaponPivotPolicyV1,
    pub material_policy: HeldWeaponMaterialPolicyV1,
    pub secondary_hand_guide_node_id: Option<u32>,
    pub secondary_grip_point: Option<[f32; 3]>,
    pub attachment_revision: u64,
    pub canonical_fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldWeaponGripErrorReportV1 {
    pub schema_version: u32,
    pub sample_count: usize,
    pub primary_hand_max_error: f32,
    pub secondary_hand_max_error: Option<f32>,
    pub secondary_hand_mean_error: Option<f32>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldWeaponBakeReportV1 {
    pub schema_version: u32,
    pub attachment_fingerprint_sha256: String,
    pub target_triangle_count_before: usize,
    pub weapon_triangle_count: usize,
    pub target_triangle_count_after: usize,
    pub segment_count_before: usize,
    pub segment_count_after: usize,
    pub attachment_node_id: u32,
    pub model: AuroraModelIrV1,
    pub fingerprint_sha256: String,
}

pub fn inspect_held_weapon_source_v1(
    filename: &str,
    bytes: &[u8],
    provenance: &str,
) -> Result<HeldWeaponSourceV1, String> {
    if filename.trim().is_empty() || provenance.trim().is_empty() {
        return Err("weapon filename and provenance must not be empty".to_owned());
    }
    let inspection = ingest_glb(bytes, &GlbLimits::default())
        .map_err(|error| format!("{}: {}", error.code, error.message))?;
    if !inspection.report.conversion_eligible {
        return Err("weapon GLB failed the canonical ingestion gates".to_owned());
    }
    if inspection.report.inventory.skin_count != 0
        || inspection.report.inventory.animation_count != 0
    {
        return Err("held weapon source must be rigid and contain no skin or animation".to_owned());
    }
    if inspection.report.statistics.triangle_count == 0 {
        return Err("held weapon source contains no triangle geometry".to_owned());
    }
    Ok(HeldWeaponSourceV1 {
        schema_version: 1,
        filename: filename.to_owned(),
        byte_size: bytes.len(),
        sha256: format!("{:x}", Sha256::digest(bytes)),
        provenance: provenance.to_owned(),
        triangle_count: inspection.report.statistics.triangle_count,
        material_count: inspection.report.inventory.material_count,
        texture_count: inspection.report.inventory.texture_count,
        bounds_min: inspection.report.statistics.bounds_min,
        bounds_max: inspection.report.statistics.bounds_max,
    })
}

pub fn resolve_hand_attachment_target_v1(
    rig: &AnimationStudioRigV1,
    hand: HeldWeaponHandV1,
    explicit_node_id: Option<u32>,
) -> Result<&AnimationStudioRigNodeV1, String> {
    if let Some(node_id) = explicit_node_id {
        return rig
            .nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .ok_or_else(|| format!("explicit hand node {node_id} is absent from the exact rig"));
    }
    let aliases = match hand {
        HeldWeaponHandV1::Right => [
            "rhand",
            "handr",
            "righthand",
            "handright",
            "bip01rhand",
            "mixamorigrighthand",
        ],
        HeldWeaponHandV1::Left => [
            "lhand",
            "handl",
            "lefthand",
            "handleft",
            "bip01lhand",
            "mixamoriglefthand",
        ],
    };
    let candidates = rig
        .nodes
        .iter()
        .filter(|node| aliases.contains(&normalize_name(&node.name).as_str()))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [candidate] => Ok(*candidate),
        [] => Err(format!(
            "no {:?} hand alias matched; choose the target bone explicitly",
            hand
        )),
        _ => Err(format!(
            "multiple {:?} hand aliases matched; choose one target bone explicitly",
            hand
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn compose_held_weapon_attachment_v1(
    source: HeldWeaponSourceV1,
    rig: &AnimationStudioRigV1,
    hand: HeldWeaponHandV1,
    target_node_id: u32,
    local_transform: HeldWeaponLocalTransformV1,
    pivot_policy: HeldWeaponPivotPolicyV1,
    secondary_hand_guide_node_id: Option<u32>,
    secondary_grip_point: Option<[f32; 3]>,
    attachment_revision: u64,
) -> Result<HeldWeaponAttachmentV1, String> {
    let target = resolve_hand_attachment_target_v1(rig, hand, Some(target_node_id))?;
    if local_transform
        .translation
        .iter()
        .chain(local_transform.rotation.iter())
        .chain(local_transform.scale.iter())
        .any(|value| !value.is_finite())
        || local_transform.scale.iter().any(|value| *value <= 0.0)
    {
        return Err("weapon local transform must be finite with positive scale".to_owned());
    }
    let rotation = normalize_quaternion(local_transform.rotation)?;
    let secondary = match (secondary_hand_guide_node_id, secondary_grip_point) {
        (None, None) => None,
        (Some(node_id), Some(point))
            if rig.nodes.iter().any(|node| node.node_id == node_id)
                && point.iter().all(|value| value.is_finite()) =>
        {
            Some((node_id, point))
        }
        _ => {
            return Err(
                "secondary guide requires both a current rig node and a finite grip point"
                    .to_owned(),
            );
        }
    };
    let transform = HeldWeaponLocalTransformV1 {
        rotation,
        ..local_transform
    };
    let fingerprint = fingerprint_json(&(
        HeldWeaponModeV1::BakedDirectCreature,
        &source,
        hand,
        target.node_id,
        target.name.as_str(),
        transform,
        pivot_policy,
        HeldWeaponMaterialPolicyV1::PreserveSource,
        secondary,
        attachment_revision,
    ));
    Ok(HeldWeaponAttachmentV1 {
        schema_version: 1,
        mode: HeldWeaponModeV1::BakedDirectCreature,
        source,
        primary_hand: hand,
        target_node_id: target.node_id,
        target_node_name: target.name.clone(),
        local_transform: transform,
        pivot_policy,
        material_policy: HeldWeaponMaterialPolicyV1::PreserveSource,
        secondary_hand_guide_node_id: secondary.map(|value| value.0),
        secondary_grip_point: secondary.map(|value| value.1),
        attachment_revision,
        canonical_fingerprint_sha256: fingerprint,
    })
}

pub fn measure_held_weapon_grip_error_v1(
    clip: &AuthoredAnimationClipV1,
    rig: &AnimationStudioRigV1,
    attachment: &HeldWeaponAttachmentV1,
    sample_rate_hz: u32,
) -> Result<HeldWeaponGripErrorReportV1, String> {
    if sample_rate_hz == 0 || sample_rate_hz > 240 {
        return Err("grip sample rate must be within 1..=240 Hz".to_owned());
    }
    let sample_count =
        ((clip.length_seconds * sample_rate_hz as f32).ceil() as usize + 1).clamp(2, 4096);
    let primary_hand_max_error = length3(attachment.local_transform.translation);
    let mut secondary_errors = Vec::new();
    if let (Some(secondary_id), Some(grip_point)) = (
        attachment.secondary_hand_guide_node_id,
        attachment.secondary_grip_point,
    ) {
        for index in 0..sample_count {
            let time = clip.length_seconds * index as f32 / (sample_count - 1) as f32;
            let pose = sample_animation_world_pose_v1(clip, rig, time)
                .map_err(|diagnostics| format!("{:?}", diagnostics))?;
            let primary = pose
                .bones
                .iter()
                .find(|bone| bone.node_id == attachment.target_node_id)
                .ok_or_else(|| "primary hand disappeared from sampled pose".to_owned())?;
            let secondary = pose
                .bones
                .iter()
                .find(|bone| bone.node_id == secondary_id)
                .ok_or_else(|| "secondary hand disappeared from sampled pose".to_owned())?;
            let local_grip = add3(
                attachment.local_transform.translation,
                rotate3(attachment.local_transform.rotation, grip_point),
            );
            let world_grip = add3(primary.translation, rotate3(primary.rotation, local_grip));
            secondary_errors.push(distance3(world_grip, secondary.translation));
        }
    }
    let secondary_hand_max_error = secondary_errors.iter().copied().reduce(f32::max);
    let secondary_hand_mean_error = (!secondary_errors.is_empty())
        .then(|| secondary_errors.iter().sum::<f32>() / secondary_errors.len() as f32);
    let fingerprint_sha256 = fingerprint_json(&(
        clip.id.as_str(),
        attachment.canonical_fingerprint_sha256.as_str(),
        sample_count,
        primary_hand_max_error,
        secondary_hand_max_error,
        secondary_hand_mean_error,
    ));
    Ok(HeldWeaponGripErrorReportV1 {
        schema_version: 1,
        sample_count,
        primary_hand_max_error,
        secondary_hand_max_error,
        secondary_hand_mean_error,
        fingerprint_sha256,
    })
}

/// Bakes an already converted rigid weapon IR below the selected hand node.
/// The caller remains responsible for obtaining the weapon IR from the exact
/// source GLB and for merging its texture payloads into the final package.
pub fn bake_held_weapon_attachment_v1(
    target: &AuroraModelIrV1,
    weapon: &AuroraModelIrV1,
    attachment: &HeldWeaponAttachmentV1,
) -> Result<HeldWeaponBakeReportV1, String> {
    if attachment.schema_version != 1
        || attachment.mode != HeldWeaponModeV1::BakedDirectCreature
        || attachment.material_policy != HeldWeaponMaterialPolicyV1::PreserveSource
        || attachment.source.sha256 != weapon.source_sha256
    {
        return Err("held weapon attachment does not match the exact converted weapon IR".into());
    }
    if !target.nodes.iter().any(|node| {
        node.id == attachment.target_node_id && node.name == attachment.target_node_name
    }) {
        return Err("held weapon target node is absent or has a stale name".into());
    }
    if weapon.segments.is_empty()
        || weapon
            .segments
            .iter()
            .any(|segment| segment.deformation != AuroraSegmentDeformationV1::Rigid)
    {
        return Err("held weapon IR must contain rigid geometry only".into());
    }
    let target_triangle_count_before =
        validate_model_triangle_budget_v1(target).map_err(|error| error.to_string())?;
    let weapon_triangle_count =
        validate_model_triangle_budget_v1(weapon).map_err(|error| error.to_string())?;
    if attachment.source.triangle_count != weapon_triangle_count {
        return Err("held weapon source triangle identity differs from converted weapon IR".into());
    }
    let segment_count_before = target.segments.len();
    let mut model = target.clone();
    let next_node_id = model
        .nodes
        .iter()
        .map(|node| node.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "target node ID space is exhausted".to_owned())?;
    let attachment_node_id = next_node_id;
    let pivot = match attachment.pivot_policy {
        HeldWeaponPivotPolicyV1::SourceOrigin => [0.0; 3],
        HeldWeaponPivotPolicyV1::BoundsCenter => weapon_bounds_center(weapon)?,
    };
    model.nodes.push(AuroraModelNodeV1 {
        id: attachment_node_id,
        name: format!("held_weapon_{attachment_node_id}"),
        parent_id: Some(attachment.target_node_id),
        bind_local_matrix: attachment_matrix(attachment.local_transform, pivot),
    });
    let mut node_map = std::collections::BTreeMap::new();
    let mut cursor = attachment_node_id;
    for node in &weapon.nodes {
        cursor = cursor
            .checked_add(1)
            .ok_or_else(|| "weapon node ID remap overflowed".to_owned())?;
        node_map.insert(node.id, cursor);
    }
    for node in &weapon.nodes {
        model.nodes.push(AuroraModelNodeV1 {
            id: node_map[&node.id],
            name: held_weapon_child_node_name(node_map[&node.id], &node.name),
            parent_id: Some(
                node.parent_id
                    .and_then(|parent_id| node_map.get(&parent_id).copied())
                    .unwrap_or(attachment_node_id),
            ),
            bind_local_matrix: node.bind_local_matrix,
        });
    }
    let next_material_slot = model
        .material_source_bindings
        .iter()
        .map(|binding| binding.slot)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "material slot space is exhausted".to_owned())?;
    let mut material_map = std::collections::BTreeMap::new();
    let mut source_slots = weapon
        .segments
        .iter()
        .map(|segment| segment.material_slot)
        .collect::<Vec<_>>();
    source_slots.sort_unstable();
    source_slots.dedup();
    for (index, source_slot) in source_slots.into_iter().enumerate() {
        let slot = next_material_slot
            .checked_add(index as u32)
            .ok_or_else(|| "material slot remap overflowed".to_owned())?;
        material_map.insert(source_slot, slot);
        let source = weapon
            .material_source_bindings
            .iter()
            .find(|binding| binding.slot == source_slot);
        model
            .material_source_bindings
            .push(AuroraMaterialSourceBindingV1 {
                slot,
                source_material_id: source.and_then(|binding| binding.source_material_id),
                source_material_name: source
                    .and_then(|binding| binding.source_material_name.clone()),
            });
    }
    let mut next_segment_id = model
        .segments
        .iter()
        .map(|segment| segment.segment_id)
        .max()
        .unwrap_or(0);
    for source_segment in &weapon.segments {
        next_segment_id = next_segment_id
            .checked_add(1)
            .ok_or_else(|| "segment ID space is exhausted".to_owned())?;
        let mut segment = source_segment.clone();
        segment.segment_id = next_segment_id;
        segment.material_slot = material_map[&source_segment.material_slot];
        segment.deformation = AuroraSegmentDeformationV1::Rigid;
        segment.parent_node_id = node_map
            .get(&source_segment.parent_node_id)
            .copied()
            .unwrap_or(attachment_node_id);
        for weight in &mut segment.weights {
            for bone_node_id in &mut weight.bone_node_ids {
                *bone_node_id = bone_node_id
                    .and_then(|node_id| node_map.get(&node_id).copied())
                    .or(Some(segment.parent_node_id));
            }
        }
        model.segments.push(segment);
    }
    let target_triangle_count_after =
        validate_model_triangle_budget_v1(&model).map_err(|error| error.to_string())?;
    if target_triangle_count_after
        != target_triangle_count_before
            .checked_add(weapon_triangle_count)
            .ok_or_else(|| "combined triangle count overflowed".to_owned())?
    {
        return Err("held weapon bake changed triangle accounting".into());
    }
    let segment_count_after = model.segments.len();
    let fingerprint_sha256 = fingerprint_json(&(
        attachment.canonical_fingerprint_sha256.as_str(),
        target_triangle_count_before,
        weapon_triangle_count,
        &model,
    ));
    Ok(HeldWeaponBakeReportV1 {
        schema_version: 1,
        attachment_fingerprint_sha256: attachment.canonical_fingerprint_sha256.clone(),
        target_triangle_count_before,
        weapon_triangle_count,
        target_triangle_count_after,
        segment_count_before,
        segment_count_after,
        attachment_node_id,
        model,
        fingerprint_sha256,
    })
}

fn held_weapon_child_node_name(node_id: u32, source_name: &str) -> String {
    let prefix = format!("hwep{node_id}_");
    let available = 31usize.saturating_sub(prefix.len());
    let mut suffix = source_name
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        .take(available)
        .map(char::from)
        .collect::<String>();
    if suffix.is_empty() {
        suffix = "node".chars().take(available).collect();
    }
    format!("{prefix}{suffix}")
}

fn weapon_bounds_center(weapon: &AuroraModelIrV1) -> Result<[f32; 3], String> {
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for position in weapon
        .segments
        .iter()
        .flat_map(|segment| &segment.positions)
    {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(position[axis]);
            maximum[axis] = maximum[axis].max(position[axis]);
        }
    }
    if minimum
        .iter()
        .chain(maximum.iter())
        .any(|value| !value.is_finite())
    {
        return Err("weapon bounds are empty or non-finite".into());
    }
    Ok([
        (minimum[0] + maximum[0]) * 0.5,
        (minimum[1] + maximum[1]) * 0.5,
        (minimum[2] + maximum[2]) * 0.5,
    ])
}

fn attachment_matrix(transform: HeldWeaponLocalTransformV1, pivot: [f32; 3]) -> [f32; 16] {
    let [x, y, z, w] = transform.rotation;
    let [sx, sy, sz] = transform.scale;
    let pivot_offset = rotate3(
        transform.rotation,
        [-pivot[0] * sx, -pivot[1] * sy, -pivot[2] * sz],
    );
    [
        (1.0 - 2.0 * (y * y + z * z)) * sx,
        (2.0 * (x * y + z * w)) * sx,
        (2.0 * (x * z - y * w)) * sx,
        0.0,
        (2.0 * (x * y - z * w)) * sy,
        (1.0 - 2.0 * (x * x + z * z)) * sy,
        (2.0 * (y * z + x * w)) * sy,
        0.0,
        (2.0 * (x * z + y * w)) * sz,
        (2.0 * (y * z - x * w)) * sz,
        (1.0 - 2.0 * (x * x + y * y)) * sz,
        0.0,
        transform.translation[0] + pivot_offset[0],
        transform.translation[1] + pivot_offset[1],
        transform.translation[2] + pivot_offset[2],
        1.0,
    ]
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_quaternion(value: [f32; 4]) -> Result<[f32; 4], String> {
    let length = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !length.is_finite() || length <= 1.0e-8 {
        return Err("weapon rotation quaternion must be finite and non-zero".to_owned());
    }
    Ok(value.map(|component| component / length))
}

fn rotate3(rotation: [f32; 4], vector: [f32; 3]) -> [f32; 3] {
    let [x, y, z, w] = rotation;
    let uv = [
        y * vector[2] - z * vector[1],
        z * vector[0] - x * vector[2],
        x * vector[1] - y * vector[0],
    ];
    let uuv = [
        y * uv[2] - z * uv[1],
        z * uv[0] - x * uv[2],
        x * uv[1] - y * uv[0],
    ];
    add3(vector, add3(scale3(uv, 2.0 * w), scale3(uuv, 2.0)))
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn scale3(value: [f32; 3], factor: f32) -> [f32; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

fn length3(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn distance3(left: [f32; 3], right: [f32; 3]) -> f32 {
    length3([left[0] - right[0], left[1] - right[1], left[2] - right[2]])
}

fn fingerprint_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("serializable held-weapon value");
    format!("{:x}", Sha256::digest(bytes))
}
