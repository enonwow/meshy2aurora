//! Deterministic Aurora hand-attachment authoring for humanoid Creature rigs.
//!
//! Meshy H1 skin joints remain byte-for-byte semantic skin inputs.  Aurora's
//! item renderer instead needs unweighted dummy children named `rhand` and
//! `lhand`.  In native humanoid models the similarly named `rhand_g` and
//! `lhand_g` nodes are the animated hand bones, not the item attachment hooks.
//! Renaming the actual Meshy skin joints would break animation mapping.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    creature_product::{CreatureItemGripFamilyV1, creature_item_grip_profile_v1},
    profile_a::{CreatureRigNodeV1, CreatureRigProfileV1, canonical_profile_sha256},
};

pub const AURORA_RIGHT_HAND_ANCHOR_V1: &str = "rhand";
pub const AURORA_LEFT_HAND_ANCHOR_V1: &str = "lhand";
pub const MESHY_H1_HAND_ANCHOR_CALIBRATION_V1: &str = "MESHY_H1_PALM_CENTER_NATIVE_ITEM_BASIS_V6";

// Measured from the public bind hierarchy of the retail `c_lich` direct
// Creature.  The hook is about one third of the forearm-to-hand segment beyond
// the hand joint.  Separate ratios preserve the small native left/right
// asymmetry without copying any model payload.
const NATIVE_RIGHT_HAND_HOOK_EXTENSION_RATIO_V2: f32 = 0.328_56;
const NATIVE_LEFT_HAND_HOOK_EXTENSION_RATIO_V2: f32 = 0.325_11;
const PALM_VERTEX_MINIMUM_WEIGHT_V5: f32 = 0.5;
const PALM_VERTEX_MINIMUM_UNIQUE_COUNT_V5: usize = 8;
const PALM_SPATIAL_WELD_EPSILON_METERS_V5: f32 = 1.0e-5;

const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 1.0, 0.0, // column 2
    0.0, 0.0, 0.0, 1.0, // translation column
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponAnchorOptionsV1 {
    /// Rigid local transform from the semantic right-hand bone to `rhand`.
    pub right_hand_local_matrix: [f32; 16],
    /// Rigid local transform from the semantic left-hand bone to `lhand`.
    pub left_hand_local_matrix: [f32; 16],
}

impl Default for CreatureWeaponAnchorOptionsV1 {
    fn default() -> Self {
        Self {
            right_hand_local_matrix: IDENTITY_MATRIX,
            left_hand_local_matrix: IDENTITY_MATRIX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureWeaponGripModeV1 {
    #[default]
    Auto,
    AutoPlusOffsets,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponEulerOffsetV1 {
    pub roll_degrees: f32,
    pub pitch_degrees: f32,
    pub yaw_degrees: f32,
}

impl CreatureWeaponEulerOffsetV1 {
    fn is_zero(self) -> bool {
        self.roll_degrees == 0.0 && self.pitch_degrees == 0.0 && self.yaw_degrees == 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponGripOptionsV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub item_family: CreatureItemGripFamilyV1,
    #[serde(default)]
    pub mode: CreatureWeaponGripModeV1,
    #[serde(default)]
    pub right_hand: CreatureWeaponEulerOffsetV1,
    #[serde(default)]
    pub left_hand: CreatureWeaponEulerOffsetV1,
}

impl Default for CreatureWeaponGripOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            item_family: CreatureItemGripFamilyV1::Sword,
            mode: CreatureWeaponGripModeV1::Auto,
            right_hand: CreatureWeaponEulerOffsetV1::default(),
            left_hand: CreatureWeaponEulerOffsetV1::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponGripHandReportV1 {
    pub requested: CreatureWeaponEulerOffsetV1,
    pub automatic_local_matrix: [f32; 16],
    pub final_local_matrix: [f32; 16],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponGripAdjustmentReportV1 {
    pub schema_version: u32,
    pub item_family: CreatureItemGripFamilyV1,
    pub mode: CreatureWeaponGripModeV1,
    pub composition_order: String,
    pub right_hand: CreatureWeaponGripHandReportV1,
    pub left_hand: CreatureWeaponGripHandReportV1,
}

/// Derives native item-hook transforms from the exact humanoid bind pose and
/// the skin-weighted surface of each hand.
///
/// Meshy rigs do not guarantee the same joint basis as native NWN Creature
/// models.  Copying an identity child therefore places the item at the wrist
/// in the Meshy hand frame and can leave it visibly detached and rotated.  The
/// derived transform:
///
/// - spatially welds seam duplicates before measuring the hand surface;
/// - places the hook at the coordinate-wise median of vertices whose hand
///   influence is at least 0.5;
/// - keeps the complete audited native item basis, including roll, instead of
///   rebuilding it from a world-up seed; and
/// - falls back to the audited native grip ratio only when the rig fixture has
///   no sufficient hand-weighted surface.
///
/// Runtime animation remains inherited from the hand.  No geometry, weights,
/// materials or animation tracks are changed.
pub fn derive_humanoid_weapon_anchor_options_v2(
    profile: &CreatureRigProfileV1,
) -> Result<CreatureWeaponAnchorOptionsV1, CreatureWeaponAnchorErrorV1> {
    derive_humanoid_weapon_anchor_options_for_family_v3(profile, CreatureItemGripFamilyV1::Sword)
}

/// Derives the hand hooks through the selected item-family basis. This is the
/// product path; V2 remains a sword-compatible wrapper for frozen callers.
pub fn derive_humanoid_weapon_anchor_options_for_family_v3(
    profile: &CreatureRigProfileV1,
    family: CreatureItemGripFamilyV1,
) -> Result<CreatureWeaponAnchorOptionsV1, CreatureWeaponAnchorErrorV1> {
    let right = find_semantic_hand(profile, "RightHand", "rig.nodes.RightHand")?;
    let left = find_semantic_hand(profile, "LeftHand", "rig.nodes.LeftHand")?;
    let item_basis_rotation = creature_item_grip_profile_v1(family).primary_item_basis_rotation;
    Ok(CreatureWeaponAnchorOptionsV1 {
        right_hand_local_matrix: derive_skin_weighted_hook_matrix_v6(
            profile,
            right,
            NATIVE_RIGHT_HAND_HOOK_EXTENSION_RATIO_V2,
            item_basis_rotation,
            "rig.nodes.RightHand",
        )?,
        left_hand_local_matrix: derive_skin_weighted_hook_matrix_v6(
            profile,
            left,
            NATIVE_LEFT_HAND_HOOK_EXTENSION_RATIO_V2,
            item_basis_rotation,
            "rig.nodes.LeftHand",
        )?,
    })
}

/// Applies caller-owned rotations in the local NWN item basis after automatic
/// palm placement and bind-basis recovery. The item blade axis is local +Y:
/// roll rotates around +Y, pitch around +X and yaw around +Z. For column
/// vectors the exact composition is `auto * Rz(yaw) * Rx(pitch) * Ry(roll)`.
pub fn apply_humanoid_weapon_grip_offsets_v1(
    automatic: CreatureWeaponAnchorOptionsV1,
    options: CreatureWeaponGripOptionsV1,
) -> Result<
    (
        CreatureWeaponAnchorOptionsV1,
        CreatureWeaponGripAdjustmentReportV1,
    ),
    CreatureWeaponAnchorErrorV1,
> {
    if options.schema_version != 1 {
        return Err(error(
            "M4A-WEAPON-GRIP-SCHEMA",
            "weaponGrip.schemaVersion",
            format!(
                "expected Creature weapon-grip schema 1, got {}",
                options.schema_version
            ),
        ));
    }
    for (path, offset) in [
        ("weaponGrip.rightHand", options.right_hand),
        ("weaponGrip.leftHand", options.left_hand),
    ] {
        validate_weapon_euler_offset_v1(offset, path)?;
    }
    if options.mode == CreatureWeaponGripModeV1::Auto
        && (!options.right_hand.is_zero() || !options.left_hand.is_zero())
    {
        return Err(error(
            "M4A-WEAPON-GRIP-AUTO-OFFSET-CONFLICT",
            "weaponGrip.mode",
            "AUTO mode requires zero right- and left-hand offsets",
        ));
    }
    validate_rigid_matrix(
        automatic.right_hand_local_matrix,
        "weaponGrip.automatic.rightHandLocalMatrix",
    )?;
    validate_rigid_matrix(
        automatic.left_hand_local_matrix,
        "weaponGrip.automatic.leftHandLocalMatrix",
    )?;

    let apply = |matrix: [f32; 16], offset: CreatureWeaponEulerOffsetV1| {
        if options.mode == CreatureWeaponGripModeV1::Auto {
            matrix
        } else {
            let rotation = multiply_rotation3(rotation3(matrix), weapon_euler_rotation_v1(offset));
            matrix4_from_rotation_translation(rotation, translation3(matrix))
        }
    };
    let adjusted = CreatureWeaponAnchorOptionsV1 {
        right_hand_local_matrix: apply(automatic.right_hand_local_matrix, options.right_hand),
        left_hand_local_matrix: apply(automatic.left_hand_local_matrix, options.left_hand),
    };
    validate_rigid_matrix(
        adjusted.right_hand_local_matrix,
        "weaponGrip.final.rightHandLocalMatrix",
    )?;
    validate_rigid_matrix(
        adjusted.left_hand_local_matrix,
        "weaponGrip.final.leftHandLocalMatrix",
    )?;
    let report = CreatureWeaponGripAdjustmentReportV1 {
        schema_version: 1,
        item_family: options.item_family,
        mode: options.mode,
        composition_order: "AUTO_X_RZ_YAW_X_RX_PITCH_X_RY_ROLL_LOCAL_ITEM_AXES".to_owned(),
        right_hand: CreatureWeaponGripHandReportV1 {
            requested: options.right_hand,
            automatic_local_matrix: automatic.right_hand_local_matrix,
            final_local_matrix: adjusted.right_hand_local_matrix,
        },
        left_hand: CreatureWeaponGripHandReportV1 {
            requested: options.left_hand,
            automatic_local_matrix: automatic.left_hand_local_matrix,
            final_local_matrix: adjusted.left_hand_local_matrix,
        },
    };
    Ok((adjusted, report))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureWeaponAnchorDispositionV1 {
    Added,
    ReusedCompatible,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponAnchorBindingV1 {
    pub anchor_name: String,
    pub anchor_node_id: u32,
    pub parent_bone_name: String,
    pub parent_bone_node_id: u32,
    pub local_matrix: [f32; 16],
    pub disposition: CreatureWeaponAnchorDispositionV1,
    /// Attachment anchors are hierarchy dummies and must never receive skin weights.
    pub weighted_vertex_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponAnchorReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub calibration: String,
    pub profile_sha256_before: String,
    pub profile_sha256_after: String,
    pub anchors: Vec<CreatureWeaponAnchorBindingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grip_adjustment: Option<CreatureWeaponGripAdjustmentReportV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureWeaponAnchorErrorV1 {
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for CreatureWeaponAnchorErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for CreatureWeaponAnchorErrorV1 {}

/// Adds or validates the two Aurora hand attachment dummies on a humanoid rig.
///
/// The operation is deterministic and idempotent.  It never changes segment
/// geometry, UVs, material bindings, allowed skin bones or vertex weights.
pub fn author_humanoid_weapon_anchors_v1(
    profile: &mut CreatureRigProfileV1,
    options: CreatureWeaponAnchorOptionsV1,
) -> Result<CreatureWeaponAnchorReportV1, CreatureWeaponAnchorErrorV1> {
    validate_rigid_matrix(
        options.right_hand_local_matrix,
        "options.rightHandLocalMatrix",
    )?;
    validate_rigid_matrix(
        options.left_hand_local_matrix,
        "options.leftHandLocalMatrix",
    )?;

    let before = profile.content_sha256.clone();
    let right_parent = find_semantic_hand(profile, "RightHand", "rig.nodes.RightHand")?;
    let left_parent = find_semantic_hand(profile, "LeftHand", "rig.nodes.LeftHand")?;

    let mut next_id = profile
        .nodes
        .iter()
        .map(|node| node.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| {
            error(
                "M4A-WEAPON-ANCHOR-ID-OVERFLOW",
                "rig.nodes",
                "no node id remains for attachment anchors",
            )
        })?;

    let right = add_or_validate_anchor(
        profile,
        AURORA_RIGHT_HAND_ANCHOR_V1,
        right_parent,
        options.right_hand_local_matrix,
        &mut next_id,
    )?;
    let left = add_or_validate_anchor(
        profile,
        AURORA_LEFT_HAND_ANCHOR_V1,
        left_parent,
        options.left_hand_local_matrix,
        &mut next_id,
    )?;

    // Re-hash only after the complete deterministic mutation.  No anchor id is
    // admitted to a segment's allowed bone set, so it remains an unweighted dummy.
    profile.content_sha256 = canonical_profile_sha256(profile).map_err(|source| {
        error(
            "M4A-WEAPON-ANCHOR-PROFILE-HASH",
            "rig.contentSha256",
            source.to_string(),
        )
    })?;
    Ok(CreatureWeaponAnchorReportV1 {
        schema_version: 1,
        status: "weapon_anchors_ready".to_owned(),
        calibration: MESHY_H1_HAND_ANCHOR_CALIBRATION_V1.to_owned(),
        profile_sha256_before: before,
        profile_sha256_after: profile.content_sha256.clone(),
        anchors: vec![right, left],
        grip_adjustment: None,
    })
}

fn find_semantic_hand(
    profile: &CreatureRigProfileV1,
    semantic_name: &str,
    path: &str,
) -> Result<usize, CreatureWeaponAnchorErrorV1> {
    let key = semantic_name.to_ascii_lowercase();
    let matches = profile
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| semantic_node_key(&node.name) == key)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        [] => Err(error(
            "M4A-WEAPON-ANCHOR-HAND-MISSING",
            path,
            format!("humanoid rig has no unambiguous semantic {semantic_name} bone"),
        )),
        _ => Err(error(
            "M4A-WEAPON-ANCHOR-HAND-AMBIGUOUS",
            path,
            format!("humanoid rig has multiple semantic {semantic_name} bones"),
        )),
    }
}

fn semantic_node_key(name: &str) -> String {
    name.rsplit([':', '|', '/', '\\'])
        .next()
        .unwrap_or(name)
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn derive_skin_weighted_hook_matrix_v6(
    profile: &CreatureRigProfileV1,
    hand_index: usize,
    fallback_extension_ratio: f32,
    item_basis_rotation: [[f32; 3]; 3],
    path: &str,
) -> Result<[f32; 16], CreatureWeaponAnchorErrorV1> {
    let native_basis = derive_bind_normalized_hook_matrix_v2(
        profile,
        hand_index,
        fallback_extension_ratio,
        item_basis_rotation,
        path,
    )?;
    let Some(palm_local) = skin_weighted_palm_center_v5(profile, hand_index, path)? else {
        return Ok(native_basis);
    };
    if length3(palm_local) <= 1.0e-6 {
        return Err(error(
            "M4A-WEAPON-ANCHOR-PALM-CENTER-DEGENERATE",
            path,
            "skin-weighted palm center must be separated from the hand joint",
        ));
    }

    let matrix = matrix4_from_rotation_translation(rotation3(native_basis), palm_local);
    validate_rigid_matrix(matrix, path)?;
    Ok(matrix)
}

fn skin_weighted_palm_center_v5(
    profile: &CreatureRigProfileV1,
    hand_index: usize,
    path: &str,
) -> Result<Option<[f32; 3]>, CreatureWeaponAnchorErrorV1> {
    let hand = &profile.nodes[hand_index];
    let hand_world = bind_world_matrix_v5(profile, hand_index, path)?;
    let hand_world_rotation_inverse = transpose_rotation3(rotation3(hand_world));
    let hand_world_translation = translation3(hand_world);
    let mut welded = BTreeMap::<(i64, i64, i64), ([f64; 3], u64)>::new();

    for (segment_index, segment) in profile.segments.iter().enumerate() {
        if segment.reference_weights.is_empty() {
            continue;
        }
        if segment.reference_weights.len() != segment.surface_positions.len() {
            return Err(error(
                "M4A-WEAPON-ANCHOR-SKIN-LAYOUT",
                &format!("rig.segments[{segment_index}].referenceWeights"),
                "skin reference-weight rows must match surface positions",
            ));
        }
        let parent_index = profile
            .nodes
            .iter()
            .position(|node| node.id == segment.parent_node_id)
            .ok_or_else(|| {
                error(
                    "M4A-WEAPON-ANCHOR-RIG-PARENT-MISSING",
                    &format!("rig.segments[{segment_index}].parentNodeId"),
                    "segment parent id does not resolve",
                )
            })?;
        let segment_parent_world = bind_world_matrix_v5(profile, parent_index, path)?;
        for (position, weights) in segment
            .surface_positions
            .iter()
            .zip(&segment.reference_weights)
        {
            let hand_weight = weights
                .iter()
                .filter(|influence| influence.bone_node_id == hand.id)
                .map(|influence| influence.value)
                .sum::<f32>();
            if hand_weight < PALM_VERTEX_MINIMUM_WEIGHT_V5 {
                continue;
            }
            let world = transform_point4(segment_parent_world, *position);
            let local = rotate3(
                hand_world_rotation_inverse,
                [
                    world[0] - hand_world_translation[0],
                    world[1] - hand_world_translation[1],
                    world[2] - hand_world_translation[2],
                ],
            );
            if local.iter().any(|value| !value.is_finite()) {
                return Err(error(
                    "M4A-WEAPON-ANCHOR-PALM-NONFINITE",
                    path,
                    "skin-weighted hand surface contains a non-finite bind-space point",
                ));
            }
            let key = (
                (f64::from(local[0]) / f64::from(PALM_SPATIAL_WELD_EPSILON_METERS_V5)).round()
                    as i64,
                (f64::from(local[1]) / f64::from(PALM_SPATIAL_WELD_EPSILON_METERS_V5)).round()
                    as i64,
                (f64::from(local[2]) / f64::from(PALM_SPATIAL_WELD_EPSILON_METERS_V5)).round()
                    as i64,
            );
            let entry = welded.entry(key).or_insert(([0.0; 3], 0));
            for (sum, coordinate) in entry.0.iter_mut().zip(local) {
                *sum += f64::from(coordinate);
            }
            entry.1 += 1;
        }
    }

    if welded.len() < PALM_VERTEX_MINIMUM_UNIQUE_COUNT_V5 {
        return Ok(None);
    }
    let mut axes = [Vec::new(), Vec::new(), Vec::new()];
    for (sum, count) in welded.values() {
        for (values, coordinate_sum) in axes.iter_mut().zip(sum) {
            values.push((*coordinate_sum / *count as f64) as f32);
        }
    }
    Ok(Some(std::array::from_fn(|axis| median_v5(&mut axes[axis]))))
}

fn median_v5(values: &mut [f32]) -> f32 {
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) * 0.5
    } else {
        values[middle]
    }
}

fn derive_bind_normalized_hook_matrix_v2(
    profile: &CreatureRigProfileV1,
    hand_index: usize,
    extension_ratio: f32,
    item_basis_rotation: [[f32; 3]; 3],
    path: &str,
) -> Result<[f32; 16], CreatureWeaponAnchorErrorV1> {
    let hand = &profile.nodes[hand_index];
    let parent_id = hand.parent_id.ok_or_else(|| {
        error(
            "M4A-WEAPON-ANCHOR-HAND-PARENT-MISSING",
            path,
            "semantic hand must have a forearm parent",
        )
    })?;
    let parent_index = profile
        .nodes
        .iter()
        .position(|node| node.id == parent_id)
        .ok_or_else(|| {
            error(
                "M4A-WEAPON-ANCHOR-HAND-PARENT-MISSING",
                path,
                "semantic hand parent id does not resolve in the rig",
            )
        })?;
    let local_direction = [
        hand.bind_local_matrix[12],
        hand.bind_local_matrix[13],
        hand.bind_local_matrix[14],
    ];
    let segment_length = length3(local_direction);
    if !segment_length.is_finite() || segment_length <= 1.0e-6 {
        return Err(error(
            "M4A-WEAPON-ANCHOR-HAND-SEGMENT-DEGENERATE",
            path,
            "forearm-to-hand bind segment must have a finite nonzero length",
        ));
    }

    let parent_world_rotation = bind_world_rotation_v2(profile, parent_index, path)?;
    let hand_world_rotation =
        multiply_rotation3(parent_world_rotation, rotation3(hand.bind_local_matrix));
    if !is_rigid_rotation3(hand_world_rotation) {
        return Err(error(
            "M4A-WEAPON-ANCHOR-HAND-BIND-NONRIGID",
            path,
            "hand bind-world transform must contain a finite proper rotation",
        ));
    }

    let world_extension_direction = normalize3(rotate3(
        parent_world_rotation,
        [
            local_direction[0] / segment_length,
            local_direction[1] / segment_length,
            local_direction[2] / segment_length,
        ],
    ));
    let inverse_hand_world_rotation = transpose_rotation3(hand_world_rotation);
    let local_extension = rotate3(
        inverse_hand_world_rotation,
        [
            world_extension_direction[0] * segment_length * extension_ratio,
            world_extension_direction[1] * segment_length * extension_ratio,
            world_extension_direction[2] * segment_length * extension_ratio,
        ],
    );
    let local_hook_rotation = multiply_rotation3(inverse_hand_world_rotation, item_basis_rotation);
    let matrix = matrix4_from_rotation_translation(local_hook_rotation, local_extension);
    validate_rigid_matrix(matrix, path)?;
    Ok(matrix)
}

fn bind_world_rotation_v2(
    profile: &CreatureRigProfileV1,
    node_index: usize,
    path: &str,
) -> Result<[[f32; 3]; 3], CreatureWeaponAnchorErrorV1> {
    let mut chain = Vec::new();
    let mut current = Some(node_index);
    while let Some(index) = current {
        if chain.contains(&index) || chain.len() >= profile.nodes.len() {
            return Err(error(
                "M4A-WEAPON-ANCHOR-RIG-CYCLE",
                path,
                "rig parent hierarchy contains a cycle",
            ));
        }
        chain.push(index);
        current = match profile.nodes[index].parent_id {
            Some(parent_id) => Some(
                profile
                    .nodes
                    .iter()
                    .position(|node| node.id == parent_id)
                    .ok_or_else(|| {
                        error(
                            "M4A-WEAPON-ANCHOR-RIG-PARENT-MISSING",
                            path,
                            "rig parent id does not resolve",
                        )
                    })?,
            ),
            None => None,
        };
    }
    let mut world = identity_rotation3();
    for index in chain.into_iter().rev() {
        let local = rotation3(profile.nodes[index].bind_local_matrix);
        if !is_rigid_rotation3(local) {
            return Err(error(
                "M4A-WEAPON-ANCHOR-HAND-BIND-NONRIGID",
                path,
                "bind hierarchy must contain finite proper rotations",
            ));
        }
        world = multiply_rotation3(world, local);
    }
    Ok(world)
}

fn bind_world_matrix_v5(
    profile: &CreatureRigProfileV1,
    node_index: usize,
    path: &str,
) -> Result<[f32; 16], CreatureWeaponAnchorErrorV1> {
    let mut chain = Vec::new();
    let mut current = Some(node_index);
    while let Some(index) = current {
        if chain.contains(&index) || chain.len() >= profile.nodes.len() {
            return Err(error(
                "M4A-WEAPON-ANCHOR-RIG-CYCLE",
                path,
                "rig parent hierarchy contains a cycle",
            ));
        }
        chain.push(index);
        current = match profile.nodes[index].parent_id {
            Some(parent_id) => Some(
                profile
                    .nodes
                    .iter()
                    .position(|node| node.id == parent_id)
                    .ok_or_else(|| {
                        error(
                            "M4A-WEAPON-ANCHOR-RIG-PARENT-MISSING",
                            path,
                            "rig parent id does not resolve",
                        )
                    })?,
            ),
            None => None,
        };
    }
    let mut world = IDENTITY_MATRIX;
    for index in chain.into_iter().rev() {
        let local = profile.nodes[index].bind_local_matrix;
        validate_rigid_matrix(local, path)?;
        world = multiply_matrix4(world, local);
    }
    Ok(world)
}

fn rotation3(matrix: [f32; 16]) -> [[f32; 3]; 3] {
    [
        [matrix[0], matrix[4], matrix[8]],
        [matrix[1], matrix[5], matrix[9]],
        [matrix[2], matrix[6], matrix[10]],
    ]
}

fn matrix4_from_rotation_translation(rotation: [[f32; 3]; 3], translation: [f32; 3]) -> [f32; 16] {
    [
        rotation[0][0],
        rotation[1][0],
        rotation[2][0],
        0.0,
        rotation[0][1],
        rotation[1][1],
        rotation[2][1],
        0.0,
        rotation[0][2],
        rotation[1][2],
        rotation[2][2],
        0.0,
        translation[0],
        translation[1],
        translation[2],
        1.0,
    ]
}

fn multiply_matrix4(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    std::array::from_fn(|index| {
        let column = index / 4;
        let row = index % 4;
        (0..4)
            .map(|axis| left[axis * 4 + row] * right[column * 4 + axis])
            .sum()
    })
}

fn transform_point4(matrix: [f32; 16], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn translation3(matrix: [f32; 16]) -> [f32; 3] {
    [matrix[12], matrix[13], matrix[14]]
}

fn identity_rotation3() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn multiply_rotation3(left: [[f32; 3]; 3], right: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            (0..3)
                .map(|axis| left[row][axis] * right[axis][column])
                .sum()
        })
    })
}

fn transpose_rotation3(rotation: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|row| std::array::from_fn(|column| rotation[column][row]))
}

fn rotate3(rotation: [[f32; 3]; 3], value: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|row| {
        rotation[row][0] * value[0] + rotation[row][1] * value[1] + rotation[row][2] * value[2]
    })
}

fn length3(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = length3(value);
    [value[0] / length, value[1] / length, value[2] / length]
}

fn is_rigid_rotation3(rotation: [[f32; 3]; 3]) -> bool {
    let columns = [
        [rotation[0][0], rotation[1][0], rotation[2][0]],
        [rotation[0][1], rotation[1][1], rotation[2][1]],
        [rotation[0][2], rotation[1][2], rotation[2][2]],
    ];
    let dot = |left: [f32; 3], right: [f32; 3]| {
        left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
    };
    let determinant = columns[0][0]
        * (columns[1][1] * columns[2][2] - columns[1][2] * columns[2][1])
        - columns[1][0] * (columns[0][1] * columns[2][2] - columns[0][2] * columns[2][1])
        + columns[2][0] * (columns[0][1] * columns[1][2] - columns[0][2] * columns[1][1]);
    rotation.iter().flatten().all(|value| value.is_finite())
        && columns
            .iter()
            .all(|column| (dot(*column, *column) - 1.0).abs() <= 1.0e-4)
        && dot(columns[0], columns[1]).abs() <= 1.0e-4
        && dot(columns[0], columns[2]).abs() <= 1.0e-4
        && dot(columns[1], columns[2]).abs() <= 1.0e-4
        && (determinant - 1.0).abs() <= 1.0e-4
}

fn add_or_validate_anchor(
    profile: &mut CreatureRigProfileV1,
    anchor_name: &str,
    parent_index: usize,
    local_matrix: [f32; 16],
    next_id: &mut u32,
) -> Result<CreatureWeaponAnchorBindingV1, CreatureWeaponAnchorErrorV1> {
    let parent = profile.nodes[parent_index].clone();
    let existing = profile
        .nodes
        .iter()
        .position(|node| node.name.eq_ignore_ascii_case(anchor_name));
    let (anchor_node_id, disposition) = if let Some(index) = existing {
        let node = &profile.nodes[index];
        if node.parent_id != Some(parent.id) || node.bind_local_matrix != local_matrix {
            return Err(error(
                "M4A-WEAPON-ANCHOR-CONFLICT",
                &format!("rig.nodes[{index}]"),
                format!(
                    "existing {anchor_name} is not the requested rigid child of {}",
                    parent.name
                ),
            ));
        }
        (node.id, CreatureWeaponAnchorDispositionV1::ReusedCompatible)
    } else {
        let node_id = *next_id;
        *next_id = next_id.checked_add(1).ok_or_else(|| {
            error(
                "M4A-WEAPON-ANCHOR-ID-OVERFLOW",
                "rig.nodes",
                "no node id remains for the second attachment anchor",
            )
        })?;
        profile.nodes.push(CreatureRigNodeV1 {
            id: node_id,
            name: anchor_name.to_owned(),
            parent_id: Some(parent.id),
            bind_local_matrix: local_matrix,
        });
        (node_id, CreatureWeaponAnchorDispositionV1::Added)
    };
    let weighted_vertex_count = profile
        .segments
        .iter()
        .flat_map(|segment| &segment.reference_weights)
        .flatten()
        .filter(|influence| influence.bone_node_id == anchor_node_id && influence.value > 0.0)
        .count() as u64;
    if weighted_vertex_count != 0
        || profile
            .segments
            .iter()
            .any(|segment| segment.allowed_bone_node_ids.contains(&anchor_node_id))
    {
        return Err(error(
            "M4A-WEAPON-ANCHOR-WEIGHTED",
            "rig.segments",
            format!("attachment dummy {anchor_name} must not participate in skinning"),
        ));
    }
    Ok(CreatureWeaponAnchorBindingV1 {
        anchor_name: anchor_name.to_owned(),
        anchor_node_id,
        parent_bone_name: parent.name,
        parent_bone_node_id: parent.id,
        local_matrix,
        disposition,
        weighted_vertex_count,
    })
}

fn validate_weapon_euler_offset_v1(
    offset: CreatureWeaponEulerOffsetV1,
    path: &str,
) -> Result<(), CreatureWeaponAnchorErrorV1> {
    for (field, value) in [
        ("rollDegrees", offset.roll_degrees),
        ("pitchDegrees", offset.pitch_degrees),
        ("yawDegrees", offset.yaw_degrees),
    ] {
        let field_path = format!("{path}.{field}");
        if !value.is_finite() {
            return Err(error(
                "M4A-WEAPON-GRIP-ANGLE-NONFINITE",
                &field_path,
                "weapon-grip angle must be finite",
            ));
        }
        if !(-180.0..=180.0).contains(&value) {
            return Err(error(
                "M4A-WEAPON-GRIP-ANGLE-RANGE",
                &field_path,
                "weapon-grip angle must be within -180..=180 degrees",
            ));
        }
    }
    Ok(())
}

fn weapon_euler_rotation_v1(offset: CreatureWeaponEulerOffsetV1) -> [[f32; 3]; 3] {
    let radians = |degrees: f32| degrees.to_radians();
    let (roll_sine, roll_cosine) = radians(offset.roll_degrees).sin_cos();
    let (pitch_sine, pitch_cosine) = radians(offset.pitch_degrees).sin_cos();
    let (yaw_sine, yaw_cosine) = radians(offset.yaw_degrees).sin_cos();
    let roll_y = [
        [roll_cosine, 0.0, roll_sine],
        [0.0, 1.0, 0.0],
        [-roll_sine, 0.0, roll_cosine],
    ];
    let pitch_x = [
        [1.0, 0.0, 0.0],
        [0.0, pitch_cosine, -pitch_sine],
        [0.0, pitch_sine, pitch_cosine],
    ];
    let yaw_z = [
        [yaw_cosine, -yaw_sine, 0.0],
        [yaw_sine, yaw_cosine, 0.0],
        [0.0, 0.0, 1.0],
    ];
    multiply_rotation3(yaw_z, multiply_rotation3(pitch_x, roll_y))
}

fn validate_rigid_matrix(matrix: [f32; 16], path: &str) -> Result<(), CreatureWeaponAnchorErrorV1> {
    if matrix.iter().any(|value| !value.is_finite())
        || matrix[3].abs() > 1.0e-6
        || matrix[7].abs() > 1.0e-6
        || matrix[11].abs() > 1.0e-6
        || (matrix[15] - 1.0).abs() > 1.0e-6
    {
        return Err(error(
            "M4A-WEAPON-ANCHOR-TRANSFORM-NONRIGID",
            path,
            "attachment transform must be a finite affine matrix",
        ));
    }
    let x = [matrix[0], matrix[1], matrix[2]];
    let y = [matrix[4], matrix[5], matrix[6]];
    let z = [matrix[8], matrix[9], matrix[10]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let norm = |a: [f32; 3]| dot(a, a).sqrt();
    let determinant = x[0] * (y[1] * z[2] - y[2] * z[1]) - y[0] * (x[1] * z[2] - x[2] * z[1])
        + z[0] * (x[1] * y[2] - x[2] * y[1]);
    if (norm(x) - 1.0).abs() > 1.0e-4
        || (norm(y) - 1.0).abs() > 1.0e-4
        || (norm(z) - 1.0).abs() > 1.0e-4
        || dot(x, y).abs() > 1.0e-4
        || dot(x, z).abs() > 1.0e-4
        || dot(y, z).abs() > 1.0e-4
        || (determinant - 1.0).abs() > 1.0e-4
    {
        return Err(error(
            "M4A-WEAPON-ANCHOR-TRANSFORM-NONRIGID",
            path,
            "attachment transform must contain only rotation and translation (no scale, shear or reflection)",
        ));
    }
    Ok(())
}

fn error(code: &str, path: &str, message: impl Into<String>) -> CreatureWeaponAnchorErrorV1 {
    CreatureWeaponAnchorErrorV1 {
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile_a::{
        Bounds3V1, CreatureRigSegmentV1, RigProvenanceAttestationsV1, RigProvenanceKindV1,
        RigProvenanceV1, RigSegmentDeformationV1, RigWeightInfluenceV1,
    };

    fn rig() -> CreatureRigProfileV1 {
        CreatureRigProfileV1 {
            schema_version: 1,
            profile_id: "test".to_owned(),
            content_sha256: String::new(),
            provenance: RigProvenanceV1 {
                kind: RigProvenanceKindV1::Synthetic,
                export_allowed: true,
                attestations: RigProvenanceAttestationsV1 {
                    controlled_construction: true,
                    no_reference_payload_copied: true,
                    rights_confirmed: true,
                },
            },
            target_bounds: Bounds3V1 {
                min: [0.0; 3],
                max: [1.0; 3],
            },
            alignment_anchor: [0.0; 3],
            nodes: vec![
                CreatureRigNodeV1 {
                    id: 1,
                    name: "Hips".to_owned(),
                    parent_id: None,
                    bind_local_matrix: IDENTITY_MATRIX,
                },
                CreatureRigNodeV1 {
                    id: 2,
                    name: "mixamorig:RightHand".to_owned(),
                    parent_id: Some(1),
                    bind_local_matrix: IDENTITY_MATRIX,
                },
                CreatureRigNodeV1 {
                    id: 3,
                    name: "LeftHand".to_owned(),
                    parent_id: Some(1),
                    bind_local_matrix: IDENTITY_MATRIX,
                },
            ],
            segments: vec![CreatureRigSegmentV1 {
                id: 1,
                name: "body".to_owned(),
                deformation: RigSegmentDeformationV1::Skin,
                parent_node_id: 1,
                surface_positions: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                surface_indices: vec![0, 1, 2],
                allowed_bone_node_ids: vec![1, 2, 3],
                reference_weights: vec![vec![], vec![], vec![]],
            }],
        }
    }

    fn calibrated_rig() -> CreatureRigProfileV1 {
        let root_rotation = [
            0.0, 1.0, 0.0, 0.0, // column 0
            -1.0, 0.0, 0.0, 0.0, // column 1
            0.0, 0.0, 1.0, 0.0, // column 2
            0.0, 0.0, 0.0, 1.0,
        ];
        let right_hand_rotation = [
            -1.0, 0.0, 0.0, 0.0, // column 0
            0.0, 1.0, 0.0, 0.0, // column 1
            0.0, 0.0, -1.0, 0.0, // column 2
            0.0, 0.0, 0.30, 1.0,
        ];
        let left_hand_rotation = [
            1.0, 0.0, 0.0, 0.0, // column 0
            0.0, 0.0, 1.0, 0.0, // column 1
            0.0, -1.0, 0.0, 0.0, // column 2
            0.0, 0.0, 0.28, 1.0,
        ];
        let mut profile = rig();
        profile.nodes = vec![
            CreatureRigNodeV1 {
                id: 1,
                name: "Hips".to_owned(),
                parent_id: None,
                bind_local_matrix: root_rotation,
            },
            CreatureRigNodeV1 {
                id: 2,
                name: "RightForeArm".to_owned(),
                parent_id: Some(1),
                bind_local_matrix: IDENTITY_MATRIX,
            },
            CreatureRigNodeV1 {
                id: 3,
                name: "RightHand".to_owned(),
                parent_id: Some(2),
                bind_local_matrix: right_hand_rotation,
            },
            CreatureRigNodeV1 {
                id: 4,
                name: "LeftForeArm".to_owned(),
                parent_id: Some(1),
                bind_local_matrix: IDENTITY_MATRIX,
            },
            CreatureRigNodeV1 {
                id: 5,
                name: "LeftHand".to_owned(),
                parent_id: Some(4),
                bind_local_matrix: left_hand_rotation,
            },
        ];
        profile.segments[0].allowed_bone_node_ids = vec![1, 2, 3, 4, 5];
        profile.content_sha256 = canonical_profile_sha256(&profile).unwrap();
        profile
    }

    fn geometry_calibrated_rig() -> CreatureRigProfileV1 {
        let translation = |value: [f32; 3]| {
            let mut matrix = IDENTITY_MATRIX;
            matrix[12..15].copy_from_slice(&value);
            matrix
        };
        let mut profile = rig();
        profile.nodes = vec![
            CreatureRigNodeV1 {
                id: 1,
                name: "Hips".to_owned(),
                parent_id: None,
                bind_local_matrix: IDENTITY_MATRIX,
            },
            CreatureRigNodeV1 {
                id: 2,
                name: "RightForeArm".to_owned(),
                parent_id: Some(1),
                bind_local_matrix: IDENTITY_MATRIX,
            },
            CreatureRigNodeV1 {
                id: 3,
                name: "RightHand".to_owned(),
                parent_id: Some(2),
                bind_local_matrix: translation([0.0, 0.0, 0.30]),
            },
            CreatureRigNodeV1 {
                id: 4,
                name: "LeftForeArm".to_owned(),
                parent_id: Some(1),
                bind_local_matrix: translation([1.0, 0.0, 0.0]),
            },
            CreatureRigNodeV1 {
                id: 5,
                name: "LeftHand".to_owned(),
                parent_id: Some(4),
                bind_local_matrix: translation([0.0, 0.0, 0.30]),
            },
        ];
        let right_center = [0.02, 0.08, 0.01];
        let left_center = [-0.02, 0.07, -0.01];
        let corners = [
            [-0.01, -0.02, -0.005],
            [-0.01, -0.02, 0.005],
            [-0.01, 0.02, -0.005],
            [-0.01, 0.02, 0.005],
            [0.01, -0.02, -0.005],
            [0.01, -0.02, 0.005],
            [0.01, 0.02, -0.005],
            [0.01, 0.02, 0.005],
        ];
        let mut positions = Vec::new();
        let mut weights = Vec::new();
        for (center, hand_id, forearm_offset) in [(right_center, 3, 0.0), (left_center, 5, 1.0)] {
            for corner in corners {
                positions.push([
                    forearm_offset + center[0] + corner[0],
                    center[1] + corner[1],
                    0.30 + center[2] + corner[2],
                ]);
                weights.push(vec![RigWeightInfluenceV1 {
                    bone_node_id: hand_id,
                    value: 1.0,
                }]);
            }
            // Exact seam duplicates must not bias the recovered palm center.
            for _ in 0..8 {
                let corner = corners[0];
                positions.push([
                    forearm_offset + center[0] + corner[0],
                    center[1] + corner[1],
                    0.30 + center[2] + corner[2],
                ]);
                weights.push(vec![RigWeightInfluenceV1 {
                    bone_node_id: hand_id,
                    value: 1.0,
                }]);
            }
        }
        profile.segments[0] = CreatureRigSegmentV1 {
            id: 1,
            name: "body".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 1,
            surface_positions: positions,
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: vec![1, 2, 3, 4, 5],
            reference_weights: weights,
        };
        profile.content_sha256 = canonical_profile_sha256(&profile).unwrap();
        profile
    }

    #[test]
    fn adds_unweighted_children_and_is_idempotent() {
        let mut rig = rig();
        rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
        let segments_before = rig.segments.clone();
        let first = author_humanoid_weapon_anchors_v1(&mut rig, Default::default()).unwrap();
        assert_eq!(first.anchors.len(), 2);
        assert_eq!(
            first
                .anchors
                .iter()
                .map(|anchor| anchor.anchor_name.as_str())
                .collect::<Vec<_>>(),
            ["rhand", "lhand"]
        );
        assert!(rig.nodes.iter().all(|node| {
            !node.name.eq_ignore_ascii_case("rhand_g") && !node.name.eq_ignore_ascii_case("lhand_g")
        }));
        assert!(
            first
                .anchors
                .iter()
                .all(|anchor| anchor.weighted_vertex_count == 0)
        );
        assert_eq!(rig.segments, segments_before);
        let nodes_after_first = rig.nodes.clone();
        let hash_after_first = rig.content_sha256.clone();
        let second = author_humanoid_weapon_anchors_v1(&mut rig, Default::default()).unwrap();
        assert_eq!(rig.nodes, nodes_after_first);
        assert_eq!(rig.content_sha256, hash_after_first);
        assert!(second.anchors.iter().all(
            |anchor| anchor.disposition == CreatureWeaponAnchorDispositionV1::ReusedCompatible
        ));
    }

    #[test]
    fn rejects_non_rigid_manual_calibration() {
        let mut rig = rig();
        let mut options = CreatureWeaponAnchorOptionsV1::default();
        options.right_hand_local_matrix[0] = 2.0;
        let error = author_humanoid_weapon_anchors_v1(&mut rig, options).unwrap_err();
        assert_eq!(error.code, "M4A-WEAPON-ANCHOR-TRANSFORM-NONRIGID");
    }

    #[test]
    fn fallback_bind_calibration_places_the_grip_beyond_the_hand_and_points_the_blade_outward() {
        let mut rig = calibrated_rig();
        let options = derive_humanoid_weapon_anchor_options_v2(&rig).unwrap();
        assert_ne!(options.right_hand_local_matrix, IDENTITY_MATRIX);
        assert_ne!(options.left_hand_local_matrix, IDENTITY_MATRIX);

        let right_hand_index = find_semantic_hand(&rig, "RightHand", "right").unwrap();
        let left_hand_index = find_semantic_hand(&rig, "LeftHand", "left").unwrap();
        for (hand_index, matrix, ratio) in [
            (
                right_hand_index,
                options.right_hand_local_matrix,
                NATIVE_RIGHT_HAND_HOOK_EXTENSION_RATIO_V2,
            ),
            (
                left_hand_index,
                options.left_hand_local_matrix,
                NATIVE_LEFT_HAND_HOOK_EXTENSION_RATIO_V2,
            ),
        ] {
            let hand = &rig.nodes[hand_index];
            let parent_index = rig
                .nodes
                .iter()
                .position(|node| Some(node.id) == hand.parent_id)
                .unwrap();
            let parent_world = bind_world_rotation_v2(&rig, parent_index, "test").unwrap();
            let hand_world = multiply_rotation3(parent_world, rotation3(hand.bind_local_matrix));
            let hook_world = multiply_rotation3(hand_world, rotation3(matrix));
            let expected_world = [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]];
            for row in 0..3 {
                for column in 0..3 {
                    let expected = expected_world[row][column];
                    assert!((hook_world[row][column] - expected).abs() <= 1.0e-5);
                }
            }

            let hand_segment = [
                hand.bind_local_matrix[12],
                hand.bind_local_matrix[13],
                hand.bind_local_matrix[14],
            ];
            let expected_world_extension =
                rotate3(parent_world, hand_segment).map(|value| value * ratio);
            let actual_world_extension = rotate3(hand_world, [matrix[12], matrix[13], matrix[14]]);
            for axis in 0..3 {
                assert!(
                    (actual_world_extension[axis] - expected_world_extension[axis]).abs() <= 1.0e-5
                );
            }
        }

        let report = author_humanoid_weapon_anchors_v1(&mut rig, options).unwrap();
        assert_eq!(
            report.calibration,
            "MESHY_H1_PALM_CENTER_NATIVE_ITEM_BASIS_V6"
        );
        assert!(report.anchors.iter().all(|anchor| {
            anchor.weighted_vertex_count == 0 && anchor.local_matrix != IDENTITY_MATRIX
        }));
    }

    #[test]
    fn skin_weighted_calibration_centers_each_hook_but_retains_the_native_item_basis() {
        let rig = geometry_calibrated_rig();
        let options = derive_humanoid_weapon_anchor_options_v2(&rig).unwrap();
        let sword_basis = creature_item_grip_profile_v1(CreatureItemGripFamilyV1::Sword)
            .primary_item_basis_rotation;
        for (hand_name, matrix, expected_center) in [
            (
                "RightHand",
                options.right_hand_local_matrix,
                [0.02, 0.08, 0.01],
            ),
            (
                "LeftHand",
                options.left_hand_local_matrix,
                [-0.02, 0.07, -0.01],
            ),
        ] {
            for axis in 0..3 {
                assert!((matrix[12 + axis] - expected_center[axis]).abs() <= 1.0e-6);
            }

            let hand_index = find_semantic_hand(&rig, hand_name, "test").unwrap();
            let hand_world = rotation3(bind_world_matrix_v5(&rig, hand_index, "test").unwrap());
            let hook_world = multiply_rotation3(hand_world, rotation3(matrix));
            for row in 0..3 {
                for column in 0..3 {
                    assert!((hook_world[row][column] - sword_basis[row][column]).abs() <= 1.0e-5);
                }
            }
            assert!(is_rigid_rotation3(rotation3(matrix)));
        }
    }

    #[test]
    fn bind_normalized_calibration_rejects_a_zero_length_hand_segment() {
        let rig = rig();
        let error = derive_humanoid_weapon_anchor_options_v2(&rig).unwrap_err();
        assert_eq!(error.code, "M4A-WEAPON-ANCHOR-HAND-SEGMENT-DEGENERATE");
    }

    #[test]
    fn missing_or_ambiguous_semantic_hand_fails_closed() {
        let mut missing = rig();
        missing.nodes.retain(|node| node.name != "LeftHand");
        let error =
            author_humanoid_weapon_anchors_v1(&mut missing, Default::default()).unwrap_err();
        assert_eq!(error.code, "M4A-WEAPON-ANCHOR-HAND-MISSING");

        let mut ambiguous = rig();
        ambiguous.nodes.push(CreatureRigNodeV1 {
            id: 4,
            name: "other:RightHand".to_owned(),
            parent_id: Some(1),
            bind_local_matrix: IDENTITY_MATRIX,
        });
        let error =
            author_humanoid_weapon_anchors_v1(&mut ambiguous, Default::default()).unwrap_err();
        assert_eq!(error.code, "M4A-WEAPON-ANCHOR-HAND-AMBIGUOUS");
    }

    #[test]
    fn auto_plus_offsets_preserves_translation_and_applies_local_weapon_axes() {
        let automatic = CreatureWeaponAnchorOptionsV1 {
            right_hand_local_matrix: matrix4_from_rotation_translation(
                identity_rotation3(),
                [0.1, 0.2, 0.3],
            ),
            left_hand_local_matrix: matrix4_from_rotation_translation(
                identity_rotation3(),
                [-0.1, 0.4, 0.5],
            ),
        };
        let adjustment = CreatureWeaponGripOptionsV1 {
            schema_version: 1,
            item_family: CreatureItemGripFamilyV1::Sword,
            mode: CreatureWeaponGripModeV1::AutoPlusOffsets,
            right_hand: CreatureWeaponEulerOffsetV1 {
                roll_degrees: 90.0,
                pitch_degrees: 0.0,
                yaw_degrees: 0.0,
            },
            left_hand: CreatureWeaponEulerOffsetV1 {
                roll_degrees: 0.0,
                pitch_degrees: 90.0,
                yaw_degrees: 90.0,
            },
        };

        let (adjusted, report) = apply_humanoid_weapon_grip_offsets_v1(automatic, adjustment)
            .expect("finite in-range local weapon-axis offsets");

        assert_eq!(
            translation3(adjusted.right_hand_local_matrix),
            [0.1, 0.2, 0.3]
        );
        assert_eq!(
            translation3(adjusted.left_hand_local_matrix),
            [-0.1, 0.4, 0.5]
        );
        let right = rotation3(adjusted.right_hand_local_matrix);
        let expected_right = [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]];
        for row in 0..3 {
            for column in 0..3 {
                assert!((right[row][column] - expected_right[row][column]).abs() <= 1.0e-5);
            }
        }
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.mode, CreatureWeaponGripModeV1::AutoPlusOffsets);
        assert_eq!(report.right_hand.requested, adjustment.right_hand);
        assert_eq!(
            report.right_hand.automatic_local_matrix,
            automatic.right_hand_local_matrix
        );
        assert_eq!(
            report.right_hand.final_local_matrix,
            adjusted.right_hand_local_matrix
        );
        assert!(is_rigid_rotation3(rotation3(
            adjusted.right_hand_local_matrix
        )));
        assert!(is_rigid_rotation3(rotation3(
            adjusted.left_hand_local_matrix
        )));
    }

    #[test]
    fn automatic_weapon_grip_is_byte_semantically_unchanged_and_rejects_hidden_offsets() {
        let sword_basis = creature_item_grip_profile_v1(CreatureItemGripFamilyV1::Sword)
            .primary_item_basis_rotation;
        let automatic = CreatureWeaponAnchorOptionsV1 {
            right_hand_local_matrix: matrix4_from_rotation_translation(
                sword_basis,
                [0.02, 0.03, 0.08],
            ),
            left_hand_local_matrix: matrix4_from_rotation_translation(
                sword_basis,
                [-0.02, 0.03, 0.08],
            ),
        };
        let (unchanged, _) = apply_humanoid_weapon_grip_offsets_v1(
            automatic,
            CreatureWeaponGripOptionsV1::default(),
        )
        .unwrap();
        assert_eq!(unchanged, automatic);

        let error = apply_humanoid_weapon_grip_offsets_v1(
            automatic,
            CreatureWeaponGripOptionsV1 {
                right_hand: CreatureWeaponEulerOffsetV1 {
                    roll_degrees: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "M4A-WEAPON-GRIP-AUTO-OFFSET-CONFLICT");
    }

    #[test]
    fn weapon_grip_offsets_fail_closed_for_schema_range_and_nonfinite_values() {
        let automatic = CreatureWeaponAnchorOptionsV1::default();
        for (options, expected_code, expected_path) in [
            (
                CreatureWeaponGripOptionsV1 {
                    schema_version: 2,
                    ..Default::default()
                },
                "M4A-WEAPON-GRIP-SCHEMA",
                "weaponGrip.schemaVersion",
            ),
            (
                CreatureWeaponGripOptionsV1 {
                    mode: CreatureWeaponGripModeV1::AutoPlusOffsets,
                    right_hand: CreatureWeaponEulerOffsetV1 {
                        yaw_degrees: 181.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                "M4A-WEAPON-GRIP-ANGLE-RANGE",
                "weaponGrip.rightHand.yawDegrees",
            ),
            (
                CreatureWeaponGripOptionsV1 {
                    mode: CreatureWeaponGripModeV1::AutoPlusOffsets,
                    left_hand: CreatureWeaponEulerOffsetV1 {
                        pitch_degrees: f32::NAN,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                "M4A-WEAPON-GRIP-ANGLE-NONFINITE",
                "weaponGrip.leftHand.pitchDegrees",
            ),
        ] {
            let error = apply_humanoid_weapon_grip_offsets_v1(automatic, options).unwrap_err();
            assert_eq!(error.code, expected_code);
            assert_eq!(error.path, expected_path);
        }
    }
}
