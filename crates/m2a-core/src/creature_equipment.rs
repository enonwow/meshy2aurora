//! Deterministic Aurora hand-attachment authoring for humanoid Creature rigs.
//!
//! Meshy H1 skin joints remain byte-for-byte semantic skin inputs.  Aurora's
//! item renderer instead needs unweighted dummy children named `rhand` and
//! `lhand`.  In native humanoid models the similarly named `rhand_g` and
//! `lhand_g` nodes are the animated hand bones, not the item attachment hooks.
//! Renaming the actual Meshy skin joints would break animation mapping.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::profile_a::{CreatureRigNodeV1, CreatureRigProfileV1, canonical_profile_sha256};

pub const AURORA_RIGHT_HAND_ANCHOR_V1: &str = "rhand";
pub const AURORA_LEFT_HAND_ANCHOR_V1: &str = "lhand";
pub const MESHY_H1_HAND_ANCHOR_CALIBRATION_V1: &str = "MESHY_H1_NATIVE_ITEM_HOOK_IDENTITY_CHILD_V2";

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
        RigProvenanceV1, RigSegmentDeformationV1,
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
}
