use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mdl::{
    AnimationReport, InspectionReport, MdlAnimationClipV1, MdlAnimationEventV1,
    MdlAnimationInterpolationV1, MdlAnimationSetV1, MdlAnimationTrackPathV1, MdlAnimationTrackV1,
    NodeReport,
};

/// Versioned animation-completeness policy for direct-creature output.
///
/// The legacy gameplay floor deliberately permits identical aliases of the
/// caller-owned idle clip. `FullNative42ExplicitV1` requires all source clips.
/// `FullNative42ProceduralHumanoidV1` instead authors distinct clean-room
/// motion from the caller-owned pose and semantic humanoid rig.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureAnimationProfileV1 {
    GameplayFloor7IdleFallbackV1,
    FullNative42ExplicitV1,
    /// Authors a clean-room 42-state humanoid set from the caller-owned idle
    /// pose and caller-owned rig. No retail keyframes or timings are copied.
    FullNative42ProceduralHumanoidV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureAnimationClipLineageV2 {
    pub clip_name: String,
    pub origin: DirectCreatureAnimationClipOriginV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_clip_name: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureAnimationClipOriginV2 {
    PreservedSource,
    SourceDerived,
    Procedural,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureAnimationLineageV2 {
    pub schema_version: u32,
    pub input_source_clip_count: u32,
    pub preserved_source_clip_count: u32,
    pub source_derived_clip_count: u32,
    pub procedural_clip_count: u32,
    pub discarded_source_clips: Vec<String>,
    pub clips: Vec<DirectCreatureAnimationClipLineageV2>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredProceduralHumanoidAnimationSetV2 {
    pub animations: MdlAnimationSetV1,
    pub lineage: DirectCreatureAnimationLineageV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureAnimationCompletenessV2 {
    pub schema_version: u32,
    pub profile: DirectCreatureAnimationProfileV1,
    pub required_clip_count: u32,
    pub input_source_clip_count: u32,
    pub preserved_source_clip_count: u32,
    pub source_derived_clip_count: u32,
    pub procedural_clip_count: u32,
    pub discarded_source_clip_count: u32,
    pub discarded_source_clips: Vec<String>,
    pub clips: Vec<DirectCreatureAnimationClipLineageV2>,
    pub fallback_alias_count: u32,
    pub complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProceduralHumanoidRigV1 {
    pub hips: u32,
    pub spine: u32,
    pub head: u32,
    pub left_upper_arm: u32,
    pub left_forearm: u32,
    pub right_upper_arm: u32,
    pub right_forearm: u32,
    pub left_thigh: u32,
    pub left_shin: u32,
    pub right_thigh: u32,
    pub right_shin: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralHumanoidAnimationErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ProceduralHumanoidAnimationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ProceduralHumanoidAnimationErrorV1 {}

/// Authors a complete direct-creature namespace from caller-owned humanoid
/// clips and an explicit semantic joint binding.
///
/// Every source clip must already use one exact name from the confirmed native
/// 42-state namespace and `cpause1` must occur exactly once. Explicit source
/// clips are preserved after removal of Aurora-unsafe constant scale tracks;
/// only missing states receive deterministic clean-room motion layered over
/// the source idle pose. A moving source clip mapped as `cdead` without an
/// explicit `ckdbckdie` is treated as a caller-owned Meshy death action:
/// its full motion replaces the earlier `ckdbck` fall, while `ckdbckps`,
/// `ckdbckdie` and `cdead` become static holds derived from its terminal pose.
/// This preserves one visible fall and exact pose continuity across the NWN
/// death-family sequence instead of concatenating independent Meshy actions.
pub fn author_procedural_humanoid_full_native_42_v1(
    source: &MdlAnimationSetV1,
    rig: ProceduralHumanoidRigV1,
) -> Result<MdlAnimationSetV1, ProceduralHumanoidAnimationErrorV1> {
    author_procedural_humanoid_full_native_42_v2(source, rig).map(|authored| authored.animations)
}

/// V2 authoring returns exact per-output source lineage in addition to the
/// animation set. A source clip counts as preserved only when its complete
/// motion survives in one output state. Terminal holds derived from a source
/// pose and source clips deliberately displaced by death-family routing are
/// reported separately.
pub fn author_procedural_humanoid_full_native_42_v2(
    source: &MdlAnimationSetV1,
    rig: ProceduralHumanoidRigV1,
) -> Result<AuthoredProceduralHumanoidAnimationSetV2, ProceduralHumanoidAnimationErrorV1> {
    if source.schema_version != 1 {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-SCHEMA",
            "animations.schemaVersion",
            "source animation schemaVersion must be 1",
        ));
    }
    let idle_matches = source
        .clips
        .iter()
        .filter(|clip| clip.name.eq_ignore_ascii_case("cpause1"))
        .collect::<Vec<_>>();
    if idle_matches.len() != 1 {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-IDLE",
            "animations.clips",
            "procedural humanoid authoring requires exactly one mapped cpause1 clip",
        ));
    }
    let idle = idle_matches[0];
    if !idle.length_seconds.is_finite() || idle.length_seconds <= 0.0 {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-IDLE",
            "animations.clips.cpause1.lengthSeconds",
            "cpause1 length must be finite and positive",
        ));
    }

    let required_names = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut explicit_by_name = BTreeMap::new();
    for clip in &source.clips {
        let folded_name = clip.name.to_ascii_lowercase();
        if !required_names.contains(&folded_name) {
            return Err(procedural_error(
                "M6-PROCEDURAL-HUMANOID-CLIP-NAME",
                format!("animations.clips.{}", clip.name),
                "every explicit procedural-profile source clip must use one confirmed direct-creature state name",
            ));
        }
        if explicit_by_name.insert(folded_name, clip).is_some() {
            return Err(procedural_error(
                "M6-PROCEDURAL-HUMANOID-CLIP-DUPLICATE",
                format!("animations.clips.{}", clip.name),
                "explicit direct-creature state names must be unique after ASCII case-fold",
            ));
        }
        if !clip.length_seconds.is_finite() || clip.length_seconds <= 0.0 {
            return Err(procedural_error(
                "M6-PROCEDURAL-HUMANOID-CLIP-LENGTH",
                format!("animations.clips.{}.lengthSeconds", clip.name),
                "every explicit source clip length must be finite and positive",
            ));
        }
    }

    let required_rotation_nodes = [
        rig.hips,
        rig.spine,
        rig.head,
        rig.left_upper_arm,
        rig.left_forearm,
        rig.right_upper_arm,
        rig.right_forearm,
        rig.left_thigh,
        rig.left_shin,
        rig.right_thigh,
        rig.right_shin,
    ];
    for node_id in required_rotation_nodes {
        let matching = idle
            .tracks
            .iter()
            .filter(|track| {
                track.target_node_id == node_id && track.path == MdlAnimationTrackPathV1::Rotation
            })
            .count();
        if matching != 1 {
            return Err(procedural_error(
                "M6-PROCEDURAL-HUMANOID-RIG",
                format!("animations.clips.cpause1.rotation[{node_id}]"),
                "each semantic humanoid joint must resolve to exactly one rotation track",
            ));
        }
    }

    // Aurora's direct-creature controller profile cannot safely carry the
    // source exporter's per-joint scale channels. Admit only finite,
    // time-invariant uniform scale and remove it from the preserved idle as
    // well as every authored state. The bind hierarchy remains the single
    // source of scale truth.
    let mut normalized_explicit = BTreeMap::new();
    for (folded_name, source_clip) in explicit_by_name {
        let mut normalized = source_clip.clone();
        for track in normalized
            .tracks
            .iter()
            .filter(|track| track.path == MdlAnimationTrackPathV1::Scale)
        {
            require_removable_constant_scale_track(track)?;
        }
        normalized
            .tracks
            .retain(|track| track.path != MdlAnimationTrackPathV1::Scale);
        normalized_explicit.insert(folded_name, normalized);
    }
    let normalized_idle = normalized_explicit
        .get("cpause1")
        .expect("the exact idle cardinality gate established cpause1")
        .clone();
    let explicit_cdead = normalized_explicit.get("cdead").cloned();
    let explicit_death_transition = normalized_explicit.get("ckdbckdie").cloned();
    let route_moving_cdead_to_death_family = explicit_cdead
        .as_ref()
        .is_some_and(clip_has_changing_tracks)
        && explicit_death_transition.is_none();
    let derive_cdead_from_death_transition = explicit_death_transition
        .as_ref()
        .is_some_and(clip_has_changing_tracks)
        && explicit_cdead.is_none();
    let procedural_death_fall = if !route_moving_cdead_to_death_family
        && !normalized_explicit.contains_key("ckdbck")
        && explicit_death_transition.is_none()
        && explicit_cdead.is_none()
    {
        let clip_index = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .position(|name| *name == "ckdbck")
            .expect("the full native namespace contains ckdbck");
        Some(author_procedural_clip(
            &normalized_idle,
            rig,
            "ckdbck",
            clip_index,
        )?)
    } else {
        None
    };

    let mut clips = Vec::with_capacity(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len());
    for (clip_index, clip_name) in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().enumerate() {
        if route_moving_cdead_to_death_family && *clip_name == "ckdbck" {
            let mut death_fall = explicit_cdead
                .as_ref()
                .expect("the moving-cdead route established an explicit cdead")
                .clone();
            death_fall.name = (*clip_name).to_owned();
            clips.push(death_fall);
            continue;
        }
        if route_moving_cdead_to_death_family
            && matches!(*clip_name, "ckdbckps" | "ckdbckdie" | "cdead")
        {
            clips.push(author_terminal_pose_hold(
                explicit_cdead
                    .as_ref()
                    .expect("the moving-cdead route established an explicit cdead"),
                clip_name,
            )?);
            continue;
        }
        if let Some(death_fall) = &procedural_death_fall {
            if *clip_name == "ckdbck" {
                clips.push(death_fall.clone());
                continue;
            }
            if matches!(*clip_name, "ckdbckps" | "ckdbckdie" | "cdead") {
                clips.push(author_terminal_pose_hold(death_fall, clip_name)?);
                continue;
            }
        }
        if derive_cdead_from_death_transition && *clip_name == "cdead" {
            clips.push(author_terminal_pose_hold(
                explicit_death_transition
                    .as_ref()
                    .expect("the derived-cdead route established an explicit ckdbckdie"),
                clip_name,
            )?);
            continue;
        }
        if let Some(explicit) = normalized_explicit.get(&clip_name.to_ascii_lowercase()) {
            let mut explicit = explicit.clone();
            explicit.name = (*clip_name).to_owned();
            clips.push(explicit);
            continue;
        }
        clips.push(author_procedural_clip(
            &normalized_idle,
            rig,
            clip_name,
            clip_index,
        )?);
    }
    normalize_clip_time_zero_v2(&mut clips)?;
    normalize_knockdown_recovery_family_v2(&mut clips)?;

    let death_fall_source_name = if route_moving_cdead_to_death_family {
        explicit_cdead.as_ref().map(|clip| clip.name.clone())
    } else {
        normalized_explicit
            .get("ckdbck")
            .map(|clip| clip.name.clone())
    };
    let clip_lineage = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .map(|clip_name| {
            let (origin, source_clip_name) = if route_moving_cdead_to_death_family
                && *clip_name == "ckdbck"
            {
                (
                    DirectCreatureAnimationClipOriginV2::PreservedSource,
                    explicit_cdead.as_ref().map(|clip| clip.name.clone()),
                )
            } else if *clip_name == "ckdbckps" && death_fall_source_name.is_some() {
                (
                    DirectCreatureAnimationClipOriginV2::SourceDerived,
                    death_fall_source_name.clone(),
                )
            } else if *clip_name == "ckdbckps" {
                (DirectCreatureAnimationClipOriginV2::Procedural, None)
            } else if route_moving_cdead_to_death_family
                && matches!(*clip_name, "ckdbckdie" | "cdead")
            {
                (
                    DirectCreatureAnimationClipOriginV2::SourceDerived,
                    explicit_cdead.as_ref().map(|clip| clip.name.clone()),
                )
            } else if procedural_death_fall.is_some()
                && matches!(*clip_name, "ckdbck" | "ckdbckps" | "ckdbckdie" | "cdead")
            {
                (DirectCreatureAnimationClipOriginV2::Procedural, None)
            } else if derive_cdead_from_death_transition && *clip_name == "cdead" {
                (
                    DirectCreatureAnimationClipOriginV2::SourceDerived,
                    explicit_death_transition
                        .as_ref()
                        .map(|clip| clip.name.clone()),
                )
            } else if let Some(explicit) = normalized_explicit.get(&clip_name.to_ascii_lowercase())
            {
                (
                    DirectCreatureAnimationClipOriginV2::PreservedSource,
                    Some(explicit.name.clone()),
                )
            } else {
                (DirectCreatureAnimationClipOriginV2::Procedural, None)
            };
            DirectCreatureAnimationClipLineageV2 {
                clip_name: (*clip_name).to_owned(),
                origin,
                source_clip_name,
            }
        })
        .collect::<Vec<_>>();
    let retained_source_names = clip_lineage
        .iter()
        .filter_map(|clip| clip.source_clip_name.as_deref())
        .map(str::to_ascii_lowercase)
        .collect::<BTreeSet<_>>();
    let discarded_source_clips = source
        .clips
        .iter()
        .filter(|clip| !retained_source_names.contains(&clip.name.to_ascii_lowercase()))
        .map(|clip| clip.name.clone())
        .collect::<Vec<_>>();
    let count_origin = |origin| {
        clip_lineage
            .iter()
            .filter(|clip| clip.origin == origin)
            .count() as u32
    };

    Ok(AuthoredProceduralHumanoidAnimationSetV2 {
        animations: MdlAnimationSetV1 {
            schema_version: 1,
            clips,
        },
        lineage: DirectCreatureAnimationLineageV2 {
            schema_version: 2,
            input_source_clip_count: source.clips.len() as u32,
            preserved_source_clip_count: count_origin(
                DirectCreatureAnimationClipOriginV2::PreservedSource,
            ),
            source_derived_clip_count: count_origin(
                DirectCreatureAnimationClipOriginV2::SourceDerived,
            ),
            procedural_clip_count: count_origin(DirectCreatureAnimationClipOriginV2::Procedural),
            discarded_source_clips,
            clips: clip_lineage,
        },
    })
}

fn normalize_clip_time_zero_v2(
    clips: &mut [MdlAnimationClipV1],
) -> Result<(), ProceduralHumanoidAnimationErrorV1> {
    for clip in clips {
        for track in &mut clip.tracks {
            if track.times_seconds.len() != track.values.len() || track.times_seconds.is_empty() {
                return Err(procedural_error(
                    "M6-PROCEDURAL-HUMANOID-TIME-ZERO",
                    format!(
                        "animations.clips.{}.tracks.{}.{:?}",
                        clip.name, track.target_node_id, track.path
                    ),
                    "animation tracks require matching non-empty time and value rows",
                ));
            }
            let first_time = track.times_seconds[0];
            if !first_time.is_finite() || first_time < 0.0 {
                return Err(procedural_error(
                    "M6-PROCEDURAL-HUMANOID-TIME-ZERO",
                    format!(
                        "animations.clips.{}.tracks.{}.{:?}.times[0]",
                        clip.name, track.target_node_id, track.path
                    ),
                    "the first animation key must be finite and nonnegative",
                ));
            }
            if first_time > 0.0 {
                let first_value = track.values[0].clone();
                track.times_seconds.insert(0, 0.0);
                track.values.insert(0, first_value);
            }
        }
    }
    Ok(())
}

fn normalize_knockdown_recovery_family_v2(
    clips: &mut [MdlAnimationClipV1],
) -> Result<(), ProceduralHumanoidAnimationErrorV1> {
    let knockdown = clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("ckdbck"))
        .cloned()
        .ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY",
                "animations.clips.ckdbck",
                "the recovery family requires ckdbck",
            )
        })?;
    let knockdown_hold = author_terminal_pose_hold(&knockdown, "ckdbckps")?;
    let hold_index = clips
        .iter()
        .position(|clip| clip.name.eq_ignore_ascii_case("ckdbckps"))
        .ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY",
                "animations.clips.ckdbckps",
                "the recovery family requires ckdbckps",
            )
        })?;
    clips[hold_index] = knockdown_hold;

    let hold = clips[hold_index].clone();
    let arise_index = clips
        .iter()
        .position(|clip| clip.name.eq_ignore_ascii_case("cguptokdb"))
        .ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY",
                "animations.clips.cguptokdb",
                "the recovery family requires cguptokdb",
            )
        })?;
    align_clip_start_to_terminal_pose_v2(&hold, &mut clips[arise_index])?;

    let arise = clips[arise_index].clone();
    let idle = clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("cpause1"))
        .cloned()
        .ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY",
                "animations.clips.cpause1",
                "the recovery family requires cpause1",
            )
        })?;
    let stand_index = clips
        .iter()
        .position(|clip| clip.name.eq_ignore_ascii_case("cgustandb"))
        .ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY",
                "animations.clips.cgustandb",
                "the recovery family requires cgustandb",
            )
        })?;
    author_pose_bridge_v2(&arise, &idle, &mut clips[stand_index])
}

fn align_clip_start_to_terminal_pose_v2(
    predecessor: &MdlAnimationClipV1,
    successor: &mut MdlAnimationClipV1,
) -> Result<(), ProceduralHumanoidAnimationErrorV1> {
    let successor_name = successor.name.clone();
    for predecessor_track in &predecessor.tracks {
        let successor_track = successor
            .tracks
            .iter_mut()
            .find(|track| {
                track.target_node_id == predecessor_track.target_node_id
                    && track.path == predecessor_track.path
            })
            .ok_or_else(|| {
                procedural_error(
                    "M6-PROCEDURAL-HUMANOID-RECOVERY-TRACK",
                    format!(
                        "animations.clips.{}.tracks.{}.{:?}",
                        successor.name, predecessor_track.target_node_id, predecessor_track.path
                    ),
                    "successor is missing a controller required by the predecessor terminal pose",
                )
            })?;
        let target = predecessor_track.values.last().ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY-TRACK",
                format!(
                    "animations.clips.{}.tracks.{}.{:?}",
                    predecessor.name, predecessor_track.target_node_id, predecessor_track.path
                ),
                "predecessor track has no terminal value",
            )
        })?;
        let initial = successor_track.values.first().cloned().ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY-TRACK",
                format!(
                    "animations.clips.{}.tracks.{}.{:?}",
                    successor.name, successor_track.target_node_id, successor_track.path
                ),
                "successor track has no initial value",
            )
        })?;
        match successor_track.path {
            MdlAnimationTrackPathV1::Translation => {
                if target.len() != 3 || initial.len() != 3 {
                    return Err(recovery_arity_error(&successor_name, successor_track));
                }
                let delta = [
                    target[0] - initial[0],
                    target[1] - initial[1],
                    target[2] - initial[2],
                ];
                for value in &mut successor_track.values {
                    if value.len() != 3 {
                        return Err(recovery_arity_error(&successor_name, successor_track));
                    }
                    for axis in 0..3 {
                        value[axis] += delta[axis];
                    }
                }
            }
            MdlAnimationTrackPathV1::Rotation => {
                let target = normalized_quaternion_v2(target)?;
                let initial = normalized_quaternion_v2(&initial)?;
                let delta = quaternion_multiply_v2(target, quaternion_conjugate_v2(initial));
                for value in &mut successor_track.values {
                    *value =
                        quaternion_multiply_v2(delta, normalized_quaternion_v2(value)?).to_vec();
                }
            }
            _ => {
                if target != &initial {
                    return Err(procedural_error(
                        "M6-PROCEDURAL-HUMANOID-RECOVERY-TRACK",
                        format!(
                            "animations.clips.{}.tracks.{}.{:?}",
                            successor.name, successor_track.target_node_id, successor_track.path
                        ),
                        "non-transform recovery controllers must already match the predecessor",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn author_pose_bridge_v2(
    predecessor: &MdlAnimationClipV1,
    target: &MdlAnimationClipV1,
    bridge: &mut MdlAnimationClipV1,
) -> Result<(), ProceduralHumanoidAnimationErrorV1> {
    for bridge_track in &mut bridge.tracks {
        let predecessor_track = predecessor
            .tracks
            .iter()
            .find(|track| {
                track.target_node_id == bridge_track.target_node_id
                    && track.path == bridge_track.path
            })
            .ok_or_else(|| {
                procedural_error(
                    "M6-PROCEDURAL-HUMANOID-RECOVERY-BRIDGE",
                    format!(
                        "animations.clips.{}.tracks.{}.{:?}",
                        predecessor.name, bridge_track.target_node_id, bridge_track.path
                    ),
                    "recovery bridge predecessor is missing a required controller",
                )
            })?;
        let target_track = target
            .tracks
            .iter()
            .find(|track| {
                track.target_node_id == bridge_track.target_node_id
                    && track.path == bridge_track.path
            })
            .ok_or_else(|| {
                procedural_error(
                    "M6-PROCEDURAL-HUMANOID-RECOVERY-BRIDGE",
                    format!(
                        "animations.clips.{}.tracks.{}.{:?}",
                        target.name, bridge_track.target_node_id, bridge_track.path
                    ),
                    "recovery bridge target is missing a required controller",
                )
            })?;
        let from = predecessor_track.values.last().ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY-BRIDGE",
                format!("animations.clips.{}.tracks", predecessor.name),
                "recovery bridge predecessor has no terminal value",
            )
        })?;
        let to = target_track.values.first().ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-RECOVERY-BRIDGE",
                format!("animations.clips.{}.tracks", target.name),
                "recovery bridge target has no initial value",
            )
        })?;
        for (time, value) in bridge_track
            .times_seconds
            .iter()
            .copied()
            .zip(&mut bridge_track.values)
        {
            let phase = if bridge.length_seconds > 0.0 {
                (time / bridge.length_seconds).clamp(0.0, 1.0)
            } else {
                0.0
            };
            *value = match bridge_track.path {
                MdlAnimationTrackPathV1::Rotation => quaternion_nlerp_v2(from, to, phase)?.to_vec(),
                _ => {
                    if from.len() != to.len() {
                        return Err(procedural_error(
                            "M6-PROCEDURAL-HUMANOID-RECOVERY-BRIDGE",
                            format!(
                                "animations.clips.{}.tracks.{}.{:?}",
                                bridge.name, bridge_track.target_node_id, bridge_track.path
                            ),
                            "recovery bridge endpoints must have identical arity",
                        ));
                    }
                    from.iter()
                        .zip(to)
                        .map(|(left, right)| left + (right - left) * phase)
                        .collect()
                }
            };
        }
    }
    Ok(())
}

fn recovery_arity_error(
    clip_name: &str,
    track: &MdlAnimationTrackV1,
) -> ProceduralHumanoidAnimationErrorV1 {
    procedural_error(
        "M6-PROCEDURAL-HUMANOID-RECOVERY-TRACK",
        format!(
            "animations.clips.{}.tracks.{}.{:?}",
            clip_name, track.target_node_id, track.path
        ),
        "recovery transform controller has invalid arity",
    )
}

fn normalized_quaternion_v2(value: &[f32]) -> Result<[f32; 4], ProceduralHumanoidAnimationErrorV1> {
    if value.len() != 4 {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-RECOVERY-QUATERNION",
            "animations.recovery.rotation",
            "recovery rotations must contain XYZW quaternions",
        ));
    }
    let length = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !length.is_finite() || length <= f32::EPSILON {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-RECOVERY-QUATERNION",
            "animations.recovery.rotation",
            "recovery rotation must be finite and nonzero",
        ));
    }
    Ok([
        value[0] / length,
        value[1] / length,
        value[2] / length,
        value[3] / length,
    ])
}

fn quaternion_conjugate_v2(value: [f32; 4]) -> [f32; 4] {
    [-value[0], -value[1], -value[2], value[3]]
}

fn quaternion_multiply_v2(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    let product = [
        left[3] * right[0] + left[0] * right[3] + left[1] * right[2] - left[2] * right[1],
        left[3] * right[1] - left[0] * right[2] + left[1] * right[3] + left[2] * right[0],
        left[3] * right[2] + left[0] * right[1] - left[1] * right[0] + left[2] * right[3],
        left[3] * right[3] - left[0] * right[0] - left[1] * right[1] - left[2] * right[2],
    ];
    let length = product
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    product.map(|component| component / length)
}

fn quaternion_nlerp_v2(
    from: &[f32],
    to: &[f32],
    phase: f32,
) -> Result<[f32; 4], ProceduralHumanoidAnimationErrorV1> {
    let from = normalized_quaternion_v2(from)?;
    let mut to = normalized_quaternion_v2(to)?;
    if from
        .iter()
        .zip(to)
        .map(|(left, right)| left * right)
        .sum::<f32>()
        < 0.0
    {
        to = to.map(|component| -component);
    }
    normalized_quaternion_v2(&std::array::from_fn::<_, 4, _>(|index| {
        from[index] + (to[index] - from[index]) * phase
    }))
}

fn clip_has_changing_tracks(clip: &MdlAnimationClipV1) -> bool {
    clip.tracks
        .iter()
        .any(|track| track.values.windows(2).any(|pair| pair[0] != pair[1]))
}

fn author_terminal_pose_hold(
    source: &MdlAnimationClipV1,
    output_name: &str,
) -> Result<MdlAnimationClipV1, ProceduralHumanoidAnimationErrorV1> {
    let duration = 1.0 / 30.0;
    let tracks = source
        .tracks
        .iter()
        .map(|source_track| {
            let terminal = source_track.values.last().ok_or_else(|| {
                procedural_error(
                    "M6-PROCEDURAL-HUMANOID-DEATH-HOLD",
                    format!(
                        "animations.clips.{}.tracks.{}.{:?}",
                        source.name, source_track.target_node_id, source_track.path
                    ),
                    "the explicit death-motion track has no terminal pose",
                )
            })?;
            Ok(MdlAnimationTrackV1 {
                target_node_id: source_track.target_node_id,
                path: source_track.path,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, duration],
                values: vec![terminal.clone(), terminal.clone()],
            })
        })
        .collect::<Result<Vec<_>, ProceduralHumanoidAnimationErrorV1>>()?;
    Ok(MdlAnimationClipV1 {
        name: output_name.to_owned(),
        animation_root: source.animation_root.clone(),
        length_seconds: duration,
        transition_seconds: 0.0,
        events: Vec::new(),
        tracks,
    })
}

/// Creates clean-room gameplay callback timings from kinematic peaks in the
/// exact authored animation tracks.
///
/// Names come from the independently confirmed public 42-state/event
/// namespace. V2 no longer guesses fixed percentages of clip duration:
/// attacks/casts prefer arm motion, footsteps prefer leg motion and ground
/// contact prefers hips/spine motion. When a preferred group is static the
/// strongest changing semantic-rig controller is used, still binding every
/// callback to an observed motion interval.
pub fn author_procedural_common_native_events_v2(
    animations: &MdlAnimationSetV1,
    rig: ProceduralHumanoidRigV1,
) -> Result<DirectCreatureEventAuthoringV1, ProceduralHumanoidAnimationErrorV1> {
    let mut clips = Vec::new();
    for clip_name in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 {
        let matching = animations
            .clips
            .iter()
            .filter(|clip| clip.name.eq_ignore_ascii_case(clip_name))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(procedural_error(
                "M6-PROCEDURAL-HUMANOID-EVENT-CLIP",
                format!("animations.clips.{clip_name}"),
                "procedural event authoring requires every direct-creature clip exactly once",
            ));
        }
        let clip = matching[0];
        let events = COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1
            .iter()
            .filter(|(required_clip, _)| required_clip.eq_ignore_ascii_case(clip_name))
            .map(|(_, event_name)| {
                kinematic_event_time_v2(clip, event_name, rig).map(|time_seconds| {
                    MdlAnimationEventV1 {
                        time_seconds,
                        name: (*event_name).to_owned(),
                    }
                })
            })
            .collect::<Result<Vec<_>, ProceduralHumanoidAnimationErrorV1>>()?;
        if !events.is_empty() {
            clips.push(DirectCreatureClipEventAuthoringV1 {
                clip_name: clip_name.to_owned(),
                events,
            });
        }
    }
    Ok(DirectCreatureEventAuthoringV1 {
        schema_version: 1,
        clips,
    })
}

fn kinematic_event_time_v2(
    clip: &MdlAnimationClipV1,
    event_name: &str,
    rig: ProceduralHumanoidRigV1,
) -> Result<f32, ProceduralHumanoidAnimationErrorV1> {
    let preferred_nodes: &[u32] = match event_name {
        "hit" | "cast" => &[
            rig.left_upper_arm,
            rig.left_forearm,
            rig.right_upper_arm,
            rig.right_forearm,
        ],
        "snd_footstep" => &[
            rig.left_thigh,
            rig.left_shin,
            rig.right_thigh,
            rig.right_shin,
        ],
        "snd_hitground" => &[rig.hips, rig.spine],
        _ => &[],
    };
    let preferred = strongest_motion_interval_v2(clip, |track| {
        preferred_nodes.contains(&track.target_node_id)
            && matches!(
                track.path,
                MdlAnimationTrackPathV1::Translation | MdlAnimationTrackPathV1::Rotation
            )
    });
    let observed = preferred.or_else(|| {
        strongest_motion_interval_v2(clip, |track| {
            matches!(
                track.path,
                MdlAnimationTrackPathV1::Translation | MdlAnimationTrackPathV1::Rotation
            )
        })
    });
    observed.map(|(_, time)| time).ok_or_else(|| {
        procedural_error(
            "M6-PROCEDURAL-HUMANOID-EVENT-MOTION",
            format!("animations.clips.{}.events.{event_name}", clip.name),
            "a procedural gameplay event requires one observed changing transform interval",
        )
    })
}

fn strongest_motion_interval_v2(
    clip: &MdlAnimationClipV1,
    include: impl Fn(&MdlAnimationTrackV1) -> bool,
) -> Option<(f32, f32)> {
    let mut strongest = None::<(f32, f32)>;
    for track in clip.tracks.iter().filter(|track| include(track)) {
        for index in 0..track
            .times_seconds
            .len()
            .saturating_sub(1)
            .min(track.values.len().saturating_sub(1))
        {
            let duration = track.times_seconds[index + 1] - track.times_seconds[index];
            if !duration.is_finite() || duration <= 0.0 {
                continue;
            }
            let left = &track.values[index];
            let right = &track.values[index + 1];
            let distance = match track.path {
                MdlAnimationTrackPathV1::Translation if left.len() == 3 && right.len() == 3 => left
                    .iter()
                    .zip(right)
                    .map(|(from, to)| (to - from).powi(2))
                    .sum::<f32>()
                    .sqrt(),
                MdlAnimationTrackPathV1::Rotation if left.len() == 4 && right.len() == 4 => {
                    quaternion_angle_radians_v2(left, right)?
                }
                _ => continue,
            };
            let score = distance / duration;
            if score.is_finite()
                && score > 1.0e-6
                && strongest.as_ref().is_none_or(|(best, _)| score > *best)
            {
                strongest = Some((score, track.times_seconds[index + 1]));
            }
        }
    }
    strongest
}

fn quaternion_angle_radians_v2(left: &[f32], right: &[f32]) -> Option<f32> {
    let left_length = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_length = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if !left_length.is_finite()
        || !right_length.is_finite()
        || left_length <= f32::EPSILON
        || right_length <= f32::EPSILON
    {
        return None;
    }
    let dot = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f32>()
        .abs()
        / (left_length * right_length);
    Some(2.0 * dot.clamp(-1.0, 1.0).acos())
}

fn author_procedural_clip(
    idle: &MdlAnimationClipV1,
    rig: ProceduralHumanoidRigV1,
    clip_name: &str,
    clip_index: usize,
) -> Result<MdlAnimationClipV1, ProceduralHumanoidAnimationErrorV1> {
    let terminal = matches!(
        clip_name,
        "ckdbck" | "ckdbckdie" | "cdead" | "cdisappear" | "cdisappearlp"
    );
    let length_seconds = procedural_duration(clip_name, clip_index);
    let phases: [f32; 5] = if terminal {
        [0.0, 0.35, 0.75, 1.0, 1.0]
    } else {
        [0.0, 1.0, 0.0, -1.0, 0.0]
    };
    let times_seconds = [0.0, 0.25, 0.5, 0.75, 1.0]
        .map(|time| time * length_seconds)
        .to_vec();
    let amplitude = 0.10 + clip_index as f32 * 0.004;
    let mut tracks = Vec::with_capacity(idle.tracks.len());
    for source_track in &idle.tracks {
        if source_track.path == MdlAnimationTrackPathV1::Scale {
            require_removable_constant_scale_track(source_track)?;
            continue;
        }
        let base = source_track.values.first().ok_or_else(|| {
            procedural_error(
                "M6-PROCEDURAL-HUMANOID-TRACK",
                format!(
                    "animations.clips.cpause1.tracks.{}.{:?}",
                    source_track.target_node_id, source_track.path
                ),
                "source track has no value rows",
            )
        })?;
        let mut output = MdlAnimationTrackV1 {
            target_node_id: source_track.target_node_id,
            path: source_track.path,
            interpolation: MdlAnimationInterpolationV1::Linear,
            times_seconds: times_seconds.clone(),
            values: vec![base.clone(); phases.len()],
        };
        match source_track.path {
            MdlAnimationTrackPathV1::Rotation => {
                if base.len() != 4 {
                    return Err(procedural_error(
                        "M6-PROCEDURAL-HUMANOID-TRACK",
                        format!(
                            "animations.clips.cpause1.rotation[{}]",
                            source_track.target_node_id
                        ),
                        "rotation rows must contain XYZW quaternions",
                    ));
                }
                for (phase_index, phase) in phases.iter().copied().enumerate() {
                    let (axis, signed_angle) = procedural_rotation_delta(
                        clip_name,
                        source_track.target_node_id,
                        rig,
                        amplitude,
                        phase,
                    );
                    output.values[phase_index] =
                        quaternion_with_axis_angle(base, axis, signed_angle)?;
                }
            }
            MdlAnimationTrackPathV1::Translation => {
                if base.len() != 3 {
                    return Err(procedural_error(
                        "M6-PROCEDURAL-HUMANOID-TRACK",
                        format!(
                            "animations.clips.cpause1.translation[{}]",
                            source_track.target_node_id
                        ),
                        "translation rows must contain XYZ vectors",
                    ));
                }
                if source_track.target_node_id == rig.hips {
                    for (phase_index, phase) in phases.iter().copied().enumerate() {
                        let delta =
                            procedural_hips_translation(clip_name, amplitude, phase, terminal);
                        output.values[phase_index] =
                            vec![base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]];
                    }
                }
            }
            MdlAnimationTrackPathV1::Scale => unreachable!("constant scale tracks are removed"),
            MdlAnimationTrackPathV1::Weights => {}
        }
        tracks.push(output);
    }
    Ok(MdlAnimationClipV1 {
        name: clip_name.to_owned(),
        animation_root: idle.animation_root.clone(),
        length_seconds,
        transition_seconds: 0.25_f32.min(length_seconds * 0.25),
        events: Vec::new(),
        tracks,
    })
}

fn require_removable_constant_scale_track(
    track: &MdlAnimationTrackV1,
) -> Result<(), ProceduralHumanoidAnimationErrorV1> {
    let first = track.values.first().ok_or_else(|| {
        procedural_error(
            "M6-PROCEDURAL-HUMANOID-SCALE",
            format!("animations.clips.cpause1.scale[{}]", track.target_node_id),
            "scale track has no values",
        )
    })?;
    if first.len() != 1 && first.len() != 3 {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-SCALE",
            format!("animations.clips.cpause1.scale[{}]", track.target_node_id),
            "scale rows must contain one uniform value or XYZ values",
        ));
    }
    let uniform = |row: &[f32]| {
        row.iter().all(|value| value.is_finite())
            && (row.len() == 1 || row.iter().all(|value| (*value - row[0]).abs() <= 1.0e-5))
    };
    if !uniform(first)
        || track.values.iter().any(|row| {
            row.len() != first.len()
                || !uniform(row)
                || row
                    .iter()
                    .zip(first)
                    .any(|(actual, expected)| (*actual - *expected).abs() > 1.0e-5)
        })
    {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-SCALE",
            format!("animations.clips.cpause1.scale[{}]", track.target_node_id),
            "procedural authoring removes only finite, constant, uniform scale tracks",
        ));
    }
    Ok(())
}

fn procedural_duration(clip_name: &str, clip_index: usize) -> f32 {
    let base = match clip_name {
        "crun" => 0.72,
        "cwalk" | "ccwalkf" | "ccwalkb" | "ccwalkl" | "ccwalkr" => 1.05,
        "ca1slashl" | "ca1slashr" | "ca1stab" | "creach" => 0.82,
        "ckdbckdie" | "cdead" => 1.55,
        "cappear" | "cdisappear" | "cdisappearlp" => 1.25,
        _ => 0.90,
    };
    base + (clip_index % 5) as f32 * 0.01
}

fn procedural_rotation_delta(
    clip_name: &str,
    node_id: u32,
    rig: ProceduralHumanoidRigV1,
    amplitude: f32,
    phase: f32,
) -> ([f32; 3], f32) {
    let locomotion = matches!(
        clip_name,
        "cwalk" | "crun" | "ccwalkf" | "ccwalkb" | "ccwalkl" | "ccwalkr" | "ccturnr"
    );
    let attack = matches!(
        clip_name,
        "ca1slashl" | "ca1slashr" | "ca1stab" | "creach" | "cclosel" | "ccloseh"
    );
    let cast = matches!(clip_name, "cconjure1" | "ccastout" | "ccastoutlp");
    let collapse = matches!(
        clip_name,
        "ckdbck" | "ckdbckps" | "ckdbckdie" | "ckdbckdmg" | "cdead"
    );

    let side_sign = if node_id == rig.left_upper_arm
        || node_id == rig.left_forearm
        || node_id == rig.left_thigh
        || node_id == rig.left_shin
    {
        1.0
    } else if node_id == rig.right_upper_arm
        || node_id == rig.right_forearm
        || node_id == rig.right_thigh
        || node_id == rig.right_shin
    {
        -1.0
    } else {
        1.0
    };
    let (axis, multiplier) = if locomotion
        && matches!(
            node_id,
            id if id == rig.left_upper_arm
                || id == rig.right_upper_arm
                || id == rig.left_thigh
                || id == rig.right_thigh
        ) {
        ([1.0, 0.0, 0.0], side_sign * 1.6)
    } else if locomotion
        && matches!(
            node_id,
            id if id == rig.left_shin
                || id == rig.right_shin
                || id == rig.left_forearm
                || id == rig.right_forearm
        )
    {
        ([1.0, 0.0, 0.0], -side_sign)
    } else if attack
        && matches!(
            node_id,
            id if id == rig.left_upper_arm
                || id == rig.left_forearm
                || id == rig.right_upper_arm
                || id == rig.right_forearm
        )
    {
        ([0.0, 0.0, 1.0], side_sign * 2.1)
    } else if cast
        && matches!(
            node_id,
            id if id == rig.left_upper_arm
                || id == rig.right_upper_arm
                || id == rig.left_forearm
                || id == rig.right_forearm
        )
    {
        ([1.0, 0.0, 0.0], 1.7)
    } else if collapse && (node_id == rig.hips || node_id == rig.spine) {
        ([1.0, 0.0, 0.0], 2.4)
    } else if node_id == rig.spine {
        ([0.0, 0.0, 1.0], 0.75)
    } else if node_id == rig.head {
        ([0.0, 1.0, 0.0], -0.45)
    } else {
        ([0.0, 0.0, 1.0], 0.0)
    };
    (axis, amplitude * phase * multiplier)
}

fn procedural_hips_translation(
    clip_name: &str,
    amplitude: f32,
    phase: f32,
    terminal: bool,
) -> [f32; 3] {
    if matches!(
        clip_name,
        "cwalk" | "crun" | "ccwalkf" | "ccwalkb" | "ccwalkl" | "ccwalkr"
    ) {
        return [
            0.0,
            amplitude * 0.10 * phase,
            amplitude * 0.18 * phase.abs(),
        ];
    }
    if terminal {
        return [
            0.0,
            -amplitude * 0.45 * phase.max(0.0),
            -amplitude * 0.18 * phase.max(0.0),
        ];
    }
    [
        amplitude * 0.04 * phase,
        0.0,
        amplitude * 0.03 * phase.abs(),
    ]
}

fn quaternion_with_axis_angle(
    base: &[f32],
    axis: [f32; 3],
    angle: f32,
) -> Result<Vec<f32>, ProceduralHumanoidAnimationErrorV1> {
    let base_length =
        (base[0] * base[0] + base[1] * base[1] + base[2] * base[2] + base[3] * base[3]).sqrt();
    if !base_length.is_finite() || base_length <= f32::EPSILON {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-QUATERNION",
            "animations.clips.cpause1.rotation",
            "source quaternion must be finite and nonzero",
        ));
    }
    let base = [
        base[0] / base_length,
        base[1] / base_length,
        base[2] / base_length,
        base[3] / base_length,
    ];
    let half = angle * 0.5;
    let sine = half.sin();
    let delta = [axis[0] * sine, axis[1] * sine, axis[2] * sine, half.cos()];
    let product = [
        base[3] * delta[0] + base[0] * delta[3] + base[1] * delta[2] - base[2] * delta[1],
        base[3] * delta[1] - base[0] * delta[2] + base[1] * delta[3] + base[2] * delta[0],
        base[3] * delta[2] + base[0] * delta[1] - base[1] * delta[0] + base[2] * delta[3],
        base[3] * delta[3] - base[0] * delta[0] - base[1] * delta[1] - base[2] * delta[2],
    ];
    let length = (product[0] * product[0]
        + product[1] * product[1]
        + product[2] * product[2]
        + product[3] * product[3])
        .sqrt();
    if !length.is_finite() || length <= f32::EPSILON {
        return Err(procedural_error(
            "M6-PROCEDURAL-HUMANOID-QUATERNION",
            "animations.procedural.rotation",
            "authored quaternion became invalid",
        ));
    }
    Ok(product.map(|component| component / length).to_vec())
}

fn procedural_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProceduralHumanoidAnimationErrorV1 {
    ProceduralHumanoidAnimationErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

/// Local direct-creature animation namespace confirmed independently by the
/// own binary reader on the retail `c_horror` and CEP R3 42-state families.
///
/// This list is public format knowledge only. No reference keyframes, event
/// payloads, skeletons or model bytes are copied into generated output.
pub const FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1: [&str; 42] = [
    "ca1slashl",
    "ca1slashr",
    "ca1stab",
    "creach",
    "cconjure1",
    "ccastout",
    "cparryl",
    "cparryr",
    "cdodgelr",
    "cdodges",
    "creadyr",
    "creadyl",
    "cdamagel",
    "cdamager",
    "cdamages",
    "ckdbck",
    "ckdbckps",
    "ckdbckdie",
    "cguptokdb",
    "cgustandb",
    "cwalk",
    "crun",
    "ccwalkf",
    "ccwalkb",
    "ccwalkl",
    "ccwalkr",
    "cpause1",
    "chturnl",
    "chturnr",
    "ctaunt",
    "cclosel",
    "ccloseh",
    "cgetmid",
    "ckdbckdmg",
    "ccastoutlp",
    "cspasm",
    "cappear",
    "cdisappear",
    "cgetmidlp",
    "cdead",
    "cdisappearlp",
    "ccturnr",
];

/// Event hooks independently observed on all three exact namespace witnesses:
/// retail `c_Direwolf`, retail `c_horror` and CEP R3 `c_phod_horror_b`.
///
/// This is an opt-in coverage profile, not a source of event timing. Generated
/// output must receive caller-owned event times and never copies reference
/// event tables.
pub const COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1: [(&str, &str); 23] = [
    ("ca1slashl", "hit"),
    ("ca1slashr", "hit"),
    ("ca1stab", "hit"),
    ("ca1stab", "snd_footstep"),
    ("ccastout", "cast"),
    ("ccloseh", "hit"),
    ("cclosel", "hit"),
    ("ccturnr", "snd_footstep"),
    ("ccwalkb", "snd_footstep"),
    ("ccwalkf", "snd_footstep"),
    ("ccwalkl", "snd_footstep"),
    ("ccwalkr", "snd_footstep"),
    ("cdamagel", "snd_footstep"),
    ("cdamager", "snd_footstep"),
    ("cdamages", "snd_footstep"),
    ("cdodgelr", "snd_footstep"),
    ("cdodges", "snd_footstep"),
    ("ckdbck", "snd_hitground"),
    ("creach", "hit"),
    ("creach", "snd_footstep"),
    ("crun", "snd_footstep"),
    ("ctaunt", "snd_footstep"),
    ("cwalk", "snd_footstep"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureAnimationEventProfileV1 {
    OptionalExplicitV1,
    CommonNativeGameplayHooksExplicitV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureEventTimingPolicyV2 {
    CallerOwnedExplicitV1,
    KinematicPeakV2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectCreatureClipEventAuthoringV1 {
    pub clip_name: String,
    pub events: Vec<MdlAnimationEventV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectCreatureEventAuthoringV1 {
    pub schema_version: u32,
    pub clips: Vec<DirectCreatureClipEventAuthoringV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureEventAuthoringErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for DirectCreatureEventAuthoringErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for DirectCreatureEventAuthoringErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureAnimationEventConformanceV1 {
    pub schema_version: u32,
    pub profile: DirectCreatureAnimationEventProfileV1,
    pub required_pair_count: u32,
    pub satisfied_pair_count: u32,
    pub total_event_count: u32,
    pub unknown_event_names: Vec<String>,
    pub missing_pairs: Vec<String>,
    pub complete: bool,
}

/// Applies exact caller-owned event tables to already mapped output clips.
///
/// Authoring entries replace only the named clip's event table. Unlisted clips
/// retain their current events. No timing is inferred from native witnesses.
pub fn apply_direct_creature_event_authoring_v1(
    source: &MdlAnimationSetV1,
    authoring: &DirectCreatureEventAuthoringV1,
) -> Result<MdlAnimationSetV1, DirectCreatureEventAuthoringErrorV1> {
    if authoring.schema_version != 1 {
        return Err(event_authoring_error(
            "M6-ANIMATION-EVENT-AUTHORING-SCHEMA",
            "eventAuthoring.schemaVersion",
            "event authoring schemaVersion must be 1",
        ));
    }
    let mut folded_names = BTreeSet::new();
    for (clip_index, authored_clip) in authoring.clips.iter().enumerate() {
        if !folded_names.insert(authored_clip.clip_name.to_ascii_lowercase()) {
            return Err(event_authoring_error(
                "M6-ANIMATION-EVENT-AUTHORING-DUPLICATE",
                format!("eventAuthoring.clips[{clip_index}].clipName"),
                "authored clip names must be unique after ASCII case-fold",
            ));
        }
    }

    let mut output = source.clone();
    for (clip_index, authored_clip) in authoring.clips.iter().enumerate() {
        let matching = output
            .clips
            .iter()
            .enumerate()
            .filter(|(_, clip)| clip.name.eq_ignore_ascii_case(&authored_clip.clip_name))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(event_authoring_error(
                "M6-ANIMATION-EVENT-AUTHORING-CLIP",
                format!("eventAuthoring.clips[{clip_index}].clipName"),
                format!(
                    "authored clip must resolve to exactly one mapped output clip: {}",
                    authored_clip.clip_name
                ),
            ));
        }
        let target = &mut output.clips[matching[0]];
        for (event_index, event) in authored_clip.events.iter().enumerate() {
            let event_path = format!("eventAuthoring.clips[{clip_index}].events[{event_index}]");
            if !valid_event_name(&event.name) {
                return Err(event_authoring_error(
                    "M6-ANIMATION-EVENT-AUTHORING-NAME",
                    format!("{event_path}.name"),
                    "event name must be non-empty ASCII without NUL and at most 31 bytes",
                ));
            }
            if !event.time_seconds.is_finite()
                || event.time_seconds < 0.0
                || event.time_seconds > target.length_seconds
            {
                return Err(event_authoring_error(
                    "M6-ANIMATION-EVENT-AUTHORING-TIME",
                    format!("{event_path}.timeSeconds"),
                    format!(
                        "event time must be finite and inside 0..{} for {}",
                        target.length_seconds, target.name
                    ),
                ));
            }
        }
        target.events = authored_clip.events.clone();
    }
    Ok(output)
}

pub fn evaluate_direct_creature_event_conformance_v1(
    animations: &MdlAnimationSetV1,
    profile: DirectCreatureAnimationEventProfileV1,
) -> DirectCreatureAnimationEventConformanceV1 {
    let observed_pairs = animations
        .clips
        .iter()
        .flat_map(|clip| {
            clip.events.iter().map(|event| {
                (
                    clip.name.to_ascii_lowercase(),
                    event.name.to_ascii_lowercase(),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let total_event_count = animations
        .clips
        .iter()
        .map(|clip| clip.events.len() as u32)
        .sum();
    evaluate_event_pairs(profile, observed_pairs, total_event_count)
}

/// Evaluates the event profile against the exact own binary MDL readback.
pub fn evaluate_direct_creature_event_conformance_from_inspection_v1(
    report: &InspectionReport,
    profile: DirectCreatureAnimationEventProfileV1,
) -> DirectCreatureAnimationEventConformanceV1 {
    let observed_pairs = report
        .animations
        .iter()
        .flat_map(|clip| {
            clip.events.iter().map(|event| {
                (
                    clip.name.to_ascii_lowercase(),
                    event.name.to_ascii_lowercase(),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let total_event_count = report
        .animations
        .iter()
        .map(|clip| clip.events.len() as u32)
        .sum();
    evaluate_event_pairs(profile, observed_pairs, total_event_count)
}

fn evaluate_event_pairs(
    profile: DirectCreatureAnimationEventProfileV1,
    observed_pairs: BTreeSet<(String, String)>,
    total_event_count: u32,
) -> DirectCreatureAnimationEventConformanceV1 {
    let required = match profile {
        DirectCreatureAnimationEventProfileV1::OptionalExplicitV1 => &[][..],
        DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1 => {
            &COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1[..]
        }
    };
    let missing_pairs = required
        .iter()
        .filter(|(clip, event)| {
            !observed_pairs.contains(&(clip.to_ascii_lowercase(), event.to_ascii_lowercase()))
        })
        .map(|(clip, event)| format!("{clip}:{event}"))
        .collect::<Vec<_>>();
    let known_names = COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1
        .iter()
        .map(|(_, event)| event.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let unknown_event_names = observed_pairs
        .iter()
        .map(|(_, event)| event.clone())
        .filter(|name| !known_names.contains(name))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let satisfied_pair_count = required.len() as u32 - missing_pairs.len() as u32;
    let complete = missing_pairs.is_empty();
    DirectCreatureAnimationEventConformanceV1 {
        schema_version: 1,
        profile,
        required_pair_count: required.len() as u32,
        satisfied_pair_count,
        total_event_count,
        unknown_event_names,
        missing_pairs,
        complete,
    }
}

fn valid_event_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 31 && name.is_ascii() && !name.as_bytes().contains(&0)
}

fn event_authoring_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> DirectCreatureEventAuthoringErrorV1 {
    DirectCreatureEventAuthoringErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

/// These states are structurally populated in every native witness, but their
/// controller values are static in some families and changing in others. They
/// therefore require explicit controller content without imposing either
/// motion or stillness globally.
const FAMILY_VARIABLE_MOTION_CLIPS_V1: [&str; 5] =
    ["ccastoutlp", "cgetmidlp", "ckdbckps", "ckdbckdie", "cdead"];
const ESSENTIAL_BEHAVIOR_CLIPS_V1: [&str; 6] = [
    "cpause1",
    "cwalk",
    "crun",
    "ca1slashl",
    "cdamagel",
    "ckdbck",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureClipBehaviorV1 {
    pub name: String,
    pub decoded_controller_count: u32,
    pub changing_controller_count: u32,
    pub terminal_pose_controller_count: u32,
    pub motion_sha256: String,
    pub semantic_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureAnimationBehaviorV2 {
    pub schema_version: u32,
    pub required_namespace_clip_count: u32,
    pub observed_clip_count: u32,
    pub full_namespace_complete: bool,
    pub all_required_content_present: bool,
    pub active_motion_complete: bool,
    pub walk_run_distinct: bool,
    pub essential_states_distinct: bool,
    pub death_transition_terminal_pose: bool,
    pub death_family_boundary_continuous: bool,
    pub behavior_candidate_eligible: bool,
    pub clips: Vec<DirectCreatureClipBehaviorV1>,
    pub violations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureTransitionBoundaryV2 {
    pub predecessor: String,
    pub successor: String,
    pub max_position_delta_meters: Option<f32>,
    pub max_rotation_delta_degrees: Option<f32>,
    pub max_other_controller_delta: Option<f32>,
    pub continuous: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectCreatureRootMotionV2 {
    pub clip_name: String,
    pub horizontal_displacement_meters: f32,
    pub vertical_displacement_meters: f32,
    pub locomotion_state: bool,
    pub in_place: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralHumanoidKinematicsConformanceV2 {
    pub schema_version: u32,
    pub all_tracks_start_at_zero: bool,
    pub required_transition_boundaries_continuous: bool,
    pub transition_boundaries: Vec<DirectCreatureTransitionBoundaryV2>,
    pub locomotion_root_motion_assessed: bool,
    pub locomotion_root_motion_in_place: bool,
    pub root_motion: Vec<DirectCreatureRootMotionV2>,
    pub complete: bool,
    pub violations: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ControllerSemanticV1 {
    node_name: String,
    controller_type: i32,
    times_bits: Vec<u32>,
    values_bits: Vec<Vec<u32>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipSemanticV1 {
    length_bits: u32,
    transition_bits: u32,
    events: Vec<(u32, String)>,
    controllers: Vec<ControllerSemanticV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipMotionSemanticV1 {
    length_bits: u32,
    controllers: Vec<ControllerSemanticV1>,
}

/// Evaluates behavior-level offline evidence from the exact output of the own
/// binary MDL reader. This is intentionally stricter than namespace
/// completeness but remains weaker than an NWN runtime verdict.
///
/// The candidate floor requires controller content for all 42 exact states,
/// changing controllers for every consistently active state and distinct
/// walk/run semantics. The one-shot `ckdbck` state owns the visible fall and
/// must reach a terminal pose. The `ckdbckdie` and `cdead` states are
/// family-variable, but every admitted death family must preserve pose
/// continuity across `ckdbck -> ckdbckdie -> cdead`.
pub fn evaluate_direct_creature_animation_behavior_v2(
    report: &InspectionReport,
) -> DirectCreatureAnimationBehaviorV2 {
    let clips = report
        .animations
        .iter()
        .map(clip_behavior)
        .collect::<Vec<_>>();
    let mut name_counts = BTreeMap::<String, u32>::new();
    for clip in &clips {
        *name_counts
            .entry(clip.name.to_ascii_lowercase())
            .or_default() += 1;
    }
    // "Base 42 complete" means every engine-facing base slot occurs exactly
    // once. Caller-authored custom animations are an additive namespace and
    // must not invalidate the Base 42 behavior oracle.
    let full_namespace_complete = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .all(|name| name_counts.get(*name) == Some(&1));
    let all_required_content_present = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().all(|name| {
        clips
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(name))
            .is_some_and(|clip| clip.decoded_controller_count > 0)
    });
    let knockdown = clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("ckdbck"));
    let knockdown_animation = report
        .animations
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("ckdbck"));
    let death_transition_animation = report
        .animations
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("ckdbckdie"));
    let corpse_hold_animation = report
        .animations
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("cdead"));
    let active_motion_complete = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .filter(|name| !FAMILY_VARIABLE_MOTION_CLIPS_V1.contains(name))
        .all(|name| {
            clips
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(name))
                .is_some_and(|clip| clip.changing_controller_count > 0)
        });
    let walk = clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("cwalk"));
    let run = clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("crun"));
    let walk_run_distinct = walk
        .zip(run)
        .is_some_and(|(walk, run)| walk.motion_sha256 != run.motion_sha256);
    let essential_semantics = ESSENTIAL_BEHAVIOR_CLIPS_V1
        .iter()
        .filter_map(|name| {
            clips
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(name))
                .map(|clip| clip.motion_sha256.as_str())
        })
        .collect::<Vec<_>>();
    let essential_states_distinct = essential_semantics.len() == ESSENTIAL_BEHAVIOR_CLIPS_V1.len()
        && essential_semantics
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            == ESSENTIAL_BEHAVIOR_CLIPS_V1.len();
    let death_transition_terminal_pose =
        knockdown.is_some_and(|clip| clip.terminal_pose_controller_count > 0);
    let knockdown_to_death_continuous = knockdown_animation
        .zip(death_transition_animation)
        .is_some_and(|(before, after)| animation_boundary_continuous(before, after));
    let death_to_corpse_continuous = death_transition_animation
        .zip(corpse_hold_animation)
        .is_some_and(|(before, after)| animation_boundary_continuous(before, after));
    let death_family_boundary_continuous =
        knockdown_to_death_continuous && death_to_corpse_continuous;

    let mut violations = Vec::new();
    if !full_namespace_complete {
        violations.push("FULL_NAMESPACE_NOT_EXACT".to_owned());
    }
    for name in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 {
        let content = clips
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(name))
            .is_some_and(|clip| clip.decoded_controller_count > 0);
        if !content {
            violations.push(format!("CLIP_CONTENT_MISSING:{name}"));
        }
        if FAMILY_VARIABLE_MOTION_CLIPS_V1.contains(&name) {
            continue;
        }
        let has_motion = clips
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(name))
            .is_some_and(|clip| clip.changing_controller_count > 0);
        if !has_motion {
            violations.push(format!("ACTIVE_MOTION_MISSING:{name}"));
        }
    }
    if !walk_run_distinct {
        violations.push("WALK_RUN_NOT_DISTINCT".to_owned());
    }
    if !essential_states_distinct {
        violations.push("ESSENTIAL_STATES_NOT_DISTINCT".to_owned());
    }
    if !death_transition_terminal_pose {
        violations.push("DEATH_TRANSITION_TERMINAL_POSE_MISSING".to_owned());
    }
    if !knockdown_to_death_continuous {
        violations.push("DEATH_FAMILY_BOUNDARY_DISCONTINUOUS:ckdbck->ckdbckdie".to_owned());
    }
    if !death_to_corpse_continuous {
        violations.push("DEATH_FAMILY_BOUNDARY_DISCONTINUOUS:ckdbckdie->cdead".to_owned());
    }
    let behavior_candidate_eligible = violations.is_empty();

    DirectCreatureAnimationBehaviorV2 {
        schema_version: 2,
        required_namespace_clip_count: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() as u32,
        observed_clip_count: clips.len() as u32,
        full_namespace_complete,
        all_required_content_present,
        active_motion_complete,
        walk_run_distinct,
        essential_states_distinct,
        death_transition_terminal_pose,
        death_family_boundary_continuous,
        behavior_candidate_eligible,
        clips,
        violations,
    }
}

/// Product-specific V2 kinematics gate for the authored procedural humanoid.
///
/// Native witnesses remain governed by the broader behavior oracle. Our own
/// procedural output additionally requires time-zero controller coverage,
/// continuous death/recovery state boundaries and in-place locomotion at the
/// weighted `Hips` skeleton root. Non-locomotion root displacement is
/// reported for audit but is not rejected because falls, dodges and damage
/// reactions may intentionally move the body relative to the Aurora root.
pub fn evaluate_procedural_humanoid_kinematics_v2(
    report: &InspectionReport,
) -> ProceduralHumanoidKinematicsConformanceV2 {
    const REQUIRED_TRANSITIONS: [(&str, &str); 6] = [
        ("ckdbck", "ckdbckps"),
        ("ckdbckps", "cguptokdb"),
        ("cguptokdb", "cgustandb"),
        ("cgustandb", "cpause1"),
        ("ckdbck", "ckdbckdie"),
        ("ckdbckdie", "cdead"),
    ];
    const LOCOMOTION_STATES: [&str; 7] = [
        "cwalk", "crun", "ccwalkf", "ccwalkb", "ccwalkl", "ccwalkr", "ccturnr",
    ];
    const MAX_IN_PLACE_HORIZONTAL_METERS: f32 = 0.05;
    const MAX_IN_PLACE_VERTICAL_METERS: f32 = 0.05;

    let all_tracks_start_at_zero = report.animations.iter().all(|animation| {
        animation
            .node_tree
            .roots
            .iter()
            .all(node_tracks_start_at_zero)
    });
    let transition_boundaries = REQUIRED_TRANSITIONS
        .iter()
        .map(|(predecessor, successor)| {
            let before = report
                .animations
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(predecessor));
            let after = report
                .animations
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(successor));
            transition_boundary_report_v2(predecessor, successor, before, after)
        })
        .collect::<Vec<_>>();
    let required_transition_boundaries_continuous = transition_boundaries
        .iter()
        .all(|boundary| boundary.continuous);

    let mut root_motion = Vec::new();
    for animation in &report.animations {
        let Some(hips) = find_node_by_name_v2(&animation.node_tree.roots, "Hips") else {
            continue;
        };
        let Some(position) = hips.controllers.iter().find(|controller| {
            controller.decoded
                && controller.controller_type == 8
                && controller.values.first().is_some_and(|row| row.len() == 3)
                && controller.values.last().is_some_and(|row| row.len() == 3)
        }) else {
            continue;
        };
        let first = position.values.first().expect("position first row");
        let last = position.values.last().expect("position last row");
        let horizontal = ((last[0] - first[0]).powi(2) + (last[1] - first[1]).powi(2)).sqrt();
        let vertical = (last[2] - first[2]).abs();
        let locomotion_state = LOCOMOTION_STATES
            .iter()
            .any(|name| animation.name.eq_ignore_ascii_case(name));
        root_motion.push(DirectCreatureRootMotionV2 {
            clip_name: animation.name.clone(),
            horizontal_displacement_meters: horizontal,
            vertical_displacement_meters: vertical,
            locomotion_state,
            in_place: !locomotion_state
                || (horizontal <= MAX_IN_PLACE_HORIZONTAL_METERS
                    && vertical <= MAX_IN_PLACE_VERTICAL_METERS),
        });
    }
    let locomotion_root_motion_assessed = LOCOMOTION_STATES.iter().all(|required| {
        root_motion
            .iter()
            .any(|motion| motion.clip_name.eq_ignore_ascii_case(required))
    });
    let locomotion_root_motion_in_place = locomotion_root_motion_assessed
        && root_motion
            .iter()
            .filter(|motion| motion.locomotion_state)
            .all(|motion| motion.in_place);

    let mut violations = Vec::new();
    if !all_tracks_start_at_zero {
        violations.push("ANIMATION_TRACK_TIME_ZERO_MISSING".to_owned());
    }
    for boundary in &transition_boundaries {
        if !boundary.continuous {
            violations.push(format!(
                "REQUIRED_TRANSITION_DISCONTINUOUS:{}->{}",
                boundary.predecessor, boundary.successor
            ));
        }
    }
    if !locomotion_root_motion_assessed {
        violations.push("LOCOMOTION_ROOT_MOTION_NOT_ASSESSED".to_owned());
    } else if !locomotion_root_motion_in_place {
        violations.push("LOCOMOTION_ROOT_MOTION_NOT_IN_PLACE".to_owned());
    }
    let complete = violations.is_empty();
    ProceduralHumanoidKinematicsConformanceV2 {
        schema_version: 2,
        all_tracks_start_at_zero,
        required_transition_boundaries_continuous,
        transition_boundaries,
        locomotion_root_motion_assessed,
        locomotion_root_motion_in_place,
        root_motion,
        complete,
        violations,
    }
}

fn node_tracks_start_at_zero(node: &NodeReport) -> bool {
    node.controllers
        .iter()
        .filter(|controller| controller.decoded)
        .all(|controller| controller.times.first() == Some(&0.0))
        && node.children.iter().all(node_tracks_start_at_zero)
}

fn find_node_by_name_v2<'a>(nodes: &'a [NodeReport], name: &str) -> Option<&'a NodeReport> {
    for node in nodes {
        if node.name.eq_ignore_ascii_case(name) {
            return Some(node);
        }
        if let Some(found) = find_node_by_name_v2(&node.children, name) {
            return Some(found);
        }
    }
    None
}

fn transition_boundary_report_v2(
    predecessor: &str,
    successor: &str,
    before: Option<&AnimationReport>,
    after: Option<&AnimationReport>,
) -> DirectCreatureTransitionBoundaryV2 {
    let Some((before, after)) = before.zip(after) else {
        return DirectCreatureTransitionBoundaryV2 {
            predecessor: predecessor.to_owned(),
            successor: successor.to_owned(),
            max_position_delta_meters: None,
            max_rotation_delta_degrees: None,
            max_other_controller_delta: None,
            continuous: false,
        };
    };
    let mut terminal = BTreeMap::<(String, i32), Vec<f32>>::new();
    collect_boundary_controller_values(&before.node_tree.roots, false, &mut terminal);
    let mut initial = BTreeMap::<(String, i32), Vec<f32>>::new();
    collect_boundary_controller_values(&after.node_tree.roots, true, &mut initial);
    let mut max_position = None::<f32>;
    let mut max_rotation = None::<f32>;
    let mut max_other = None::<f32>;
    if !terminal.is_empty() && terminal.len() == initial.len() {
        for (key, before_value) in &terminal {
            let Some(after_value) = initial.get(key) else {
                continue;
            };
            match (key.1, before_value.len(), after_value.len()) {
                (8, 3, 3) => {
                    let delta = before_value
                        .iter()
                        .zip(after_value)
                        .map(|(left, right)| (left - right).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    max_position = Some(max_position.unwrap_or(0.0).max(delta));
                }
                (20, 4, 4) => {
                    let left_norm = before_value
                        .iter()
                        .map(|value| value * value)
                        .sum::<f32>()
                        .sqrt();
                    let right_norm = after_value
                        .iter()
                        .map(|value| value * value)
                        .sum::<f32>()
                        .sqrt();
                    if left_norm > f32::EPSILON && right_norm > f32::EPSILON {
                        let dot = before_value
                            .iter()
                            .zip(after_value)
                            .map(|(left, right)| left * right)
                            .sum::<f32>()
                            .abs()
                            / (left_norm * right_norm);
                        let degrees = (2.0 * dot.clamp(-1.0, 1.0).acos()).to_degrees();
                        max_rotation = Some(max_rotation.unwrap_or(0.0).max(degrees));
                    }
                }
                _ if before_value.len() == after_value.len() => {
                    let delta = before_value
                        .iter()
                        .zip(after_value)
                        .map(|(left, right)| (left - right).abs())
                        .fold(0.0f32, f32::max);
                    max_other = Some(max_other.unwrap_or(0.0).max(delta));
                }
                _ => {}
            }
        }
    }
    DirectCreatureTransitionBoundaryV2 {
        predecessor: predecessor.to_owned(),
        successor: successor.to_owned(),
        max_position_delta_meters: max_position,
        max_rotation_delta_degrees: max_rotation,
        max_other_controller_delta: max_other,
        continuous: animation_boundary_continuous(before, after),
    }
}

fn animation_boundary_continuous(before: &AnimationReport, after: &AnimationReport) -> bool {
    let mut terminal = BTreeMap::<(String, i32), Vec<f32>>::new();
    collect_boundary_controller_values(&before.node_tree.roots, false, &mut terminal);
    let mut initial = BTreeMap::<(String, i32), Vec<f32>>::new();
    collect_boundary_controller_values(&after.node_tree.roots, true, &mut initial);
    if terminal.is_empty() || terminal.len() != initial.len() {
        return false;
    }
    terminal.iter().all(|(key, before_value)| {
        initial.get(key).is_some_and(|after_value| {
            controller_boundary_values_match(key.1, before_value, after_value)
        })
    })
}

fn collect_boundary_controller_values(
    nodes: &[NodeReport],
    first: bool,
    output: &mut BTreeMap<(String, i32), Vec<f32>>,
) {
    for node in nodes {
        for controller in &node.controllers {
            if !controller.decoded {
                continue;
            }
            let value = if first {
                controller.values.first()
            } else {
                controller.values.last()
            };
            if let Some(value) = value {
                output.insert(
                    (node.name.to_ascii_lowercase(), controller.controller_type),
                    value.clone(),
                );
            }
        }
        collect_boundary_controller_values(&node.children, first, output);
    }
}

fn controller_boundary_values_match(controller_type: i32, before: &[f32], after: &[f32]) -> bool {
    if before.len() != after.len() || before.iter().chain(after).any(|value| !value.is_finite()) {
        return false;
    }
    match (controller_type, before.len()) {
        (8, 3) => {
            let squared_distance = before
                .iter()
                .zip(after)
                .map(|(left, right)| (left - right) * (left - right))
                .sum::<f32>();
            squared_distance.sqrt() <= 0.001
        }
        (20, 4) => {
            let dot = before
                .iter()
                .zip(after)
                .map(|(left, right)| left * right)
                .sum::<f32>()
                .abs()
                .clamp(-1.0, 1.0);
            2.0 * dot.acos() <= 1.0_f32.to_radians()
        }
        _ => before
            .iter()
            .zip(after)
            .all(|(left, right)| (left - right).abs() <= 0.001),
    }
}

fn clip_behavior(animation: &AnimationReport) -> DirectCreatureClipBehaviorV1 {
    let mut decoded_controller_count = 0u32;
    let mut changing_controller_count = 0u32;
    let mut terminal_pose_controller_count = 0u32;
    let mut controllers = Vec::new();
    for root in &animation.node_tree.roots {
        collect_controller_semantics(
            root,
            &mut controllers,
            &mut decoded_controller_count,
            &mut changing_controller_count,
            &mut terminal_pose_controller_count,
        );
    }
    let motion = ClipMotionSemanticV1 {
        length_bits: animation.length.to_bits(),
        controllers: controllers.clone(),
    };
    let motion_sha256 = Sha256::digest(
        serde_json::to_vec(&motion).expect("animation motion semantics are serializable"),
    )
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect();
    let semantic = ClipSemanticV1 {
        length_bits: animation.length.to_bits(),
        transition_bits: animation.transition.to_bits(),
        events: animation
            .events
            .iter()
            .map(|event| (event.time.to_bits(), event.name.clone()))
            .collect(),
        controllers,
    };
    let semantic_sha256 = Sha256::digest(
        serde_json::to_vec(&semantic).expect("animation behavior semantics are serializable"),
    )
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect();
    DirectCreatureClipBehaviorV1 {
        name: animation.name.clone(),
        decoded_controller_count,
        changing_controller_count,
        terminal_pose_controller_count,
        motion_sha256,
        semantic_sha256,
    }
}

fn collect_controller_semantics(
    node: &NodeReport,
    output: &mut Vec<ControllerSemanticV1>,
    decoded_controller_count: &mut u32,
    changing_controller_count: &mut u32,
    terminal_pose_controller_count: &mut u32,
) {
    for controller in &node.controllers {
        if !controller.decoded {
            continue;
        }
        *decoded_controller_count += 1;
        if controller.values.windows(2).any(|rows| rows[0] != rows[1]) {
            *changing_controller_count += 1;
        }
        if controller
            .values
            .first()
            .zip(controller.values.last())
            .is_some_and(|(first, last)| first != last)
        {
            *terminal_pose_controller_count += 1;
        }
        output.push(ControllerSemanticV1 {
            node_name: node.name.clone(),
            controller_type: controller.controller_type,
            times_bits: controller
                .times
                .iter()
                .map(|value| value.to_bits())
                .collect(),
            values_bits: controller
                .values
                .iter()
                .map(|row| row.iter().map(|value| value.to_bits()).collect())
                .collect(),
        });
    }
    for child in &node.children {
        collect_controller_semantics(
            child,
            output,
            decoded_controller_count,
            changing_controller_count,
            terminal_pose_controller_count,
        );
    }
}

#[cfg(test)]
mod procedural_tests {
    use super::*;

    fn semantic_rig() -> ProceduralHumanoidRigV1 {
        ProceduralHumanoidRigV1 {
            hips: 1,
            spine: 2,
            head: 3,
            left_upper_arm: 4,
            left_forearm: 5,
            right_upper_arm: 6,
            right_forearm: 7,
            left_thigh: 8,
            left_shin: 9,
            right_thigh: 10,
            right_shin: 11,
        }
    }

    fn idle_with_required_rotations() -> MdlAnimationSetV1 {
        let rig = semantic_rig();
        let node_ids = [
            rig.hips,
            rig.spine,
            rig.head,
            rig.left_upper_arm,
            rig.left_forearm,
            rig.right_upper_arm,
            rig.right_forearm,
            rig.left_thigh,
            rig.left_shin,
            rig.right_thigh,
            rig.right_shin,
        ];
        MdlAnimationSetV1 {
            schema_version: 1,
            clips: vec![MdlAnimationClipV1 {
                name: "cpause1".to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: node_ids
                    .into_iter()
                    .map(|target_node_id| MdlAnimationTrackV1 {
                        target_node_id,
                        path: MdlAnimationTrackPathV1::Rotation,
                        interpolation: MdlAnimationInterpolationV1::Linear,
                        times_seconds: vec![0.0, 1.0],
                        values: vec![vec![0.0, 0.0, 0.0, 1.0]; 2],
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn procedural_profile_fails_closed_when_a_semantic_joint_track_is_missing() {
        let mut source = idle_with_required_rotations();
        source.clips[0]
            .tracks
            .retain(|track| track.target_node_id != semantic_rig().right_shin);

        let error = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect_err("missing semantic joint motion must be rejected");
        assert_eq!(error.code, "M6-PROCEDURAL-HUMANOID-RIG");
        assert!(error.path.contains("11"));
    }

    #[test]
    fn procedural_profile_rejects_animated_scale_instead_of_baking_an_unsafe_guess() {
        let mut source = idle_with_required_rotations();
        source.clips[0].tracks.push(MdlAnimationTrackV1 {
            target_node_id: semantic_rig().hips,
            path: MdlAnimationTrackPathV1::Scale,
            interpolation: MdlAnimationInterpolationV1::Linear,
            times_seconds: vec![0.0, 1.0],
            values: vec![vec![1.0, 1.0, 1.0], vec![1.0, 1.1, 1.0]],
        });

        let error = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect_err("animated non-uniform scale must be rejected");
        assert_eq!(error.code, "M6-PROCEDURAL-HUMANOID-SCALE");
    }

    #[test]
    fn procedural_profile_removes_only_constant_uniform_scale_and_authors_42_states() {
        let mut source = idle_with_required_rotations();
        source.clips[0].tracks.push(MdlAnimationTrackV1 {
            target_node_id: semantic_rig().hips,
            path: MdlAnimationTrackPathV1::Scale,
            interpolation: MdlAnimationInterpolationV1::Linear,
            times_seconds: vec![0.0, 1.0],
            values: vec![vec![0.01, 0.01, 0.01]; 2],
        });

        let output = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect("constant uniform source scale is safely removable");
        assert_eq!(output.clips.len(), 42);
        assert_eq!(
            output
                .clips
                .iter()
                .map(|clip| clip.name.as_str())
                .collect::<Vec<_>>(),
            FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
        );
        assert!(output.clips.iter().all(|clip| {
            clip.tracks
                .iter()
                .all(|track| track.path != MdlAnimationTrackPathV1::Scale)
        }));
    }

    #[test]
    fn procedural_profile_preserves_explicit_native_clips_and_authors_only_missing_states() {
        let mut source = idle_with_required_rotations();
        let mut walk = source.clips[0].clone();
        walk.name = "cwalk".to_owned();
        walk.length_seconds = 1.5;
        walk.tracks[0].values[1] = vec![0.0, 0.1, 0.0, 0.995];
        source.clips.push(walk.clone());

        let output = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect("a native-name Meshy subset must be preserved");
        assert_eq!(output.clips.len(), 42);
        let preserved_walk = output
            .clips
            .iter()
            .find(|clip| clip.name == "cwalk")
            .expect("cwalk must remain explicit");
        assert_eq!(preserved_walk.length_seconds, 1.5);
        assert_eq!(preserved_walk.tracks[0].values, walk.tracks[0].values);
    }

    #[test]
    fn moving_explicit_cdead_owns_the_complete_continuous_death_family() {
        let mut source = idle_with_required_rotations();
        let mut dead = source.clips[0].clone();
        dead.name = "cdead".to_owned();
        dead.length_seconds = 3.0;
        dead.tracks[0].values[1] = vec![0.0, 0.4, 0.0, 0.9165151];
        dead.events.push(MdlAnimationEventV1 {
            time_seconds: 1.5,
            name: "owned_death_event".to_owned(),
        });
        source.clips.push(dead.clone());

        let output = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect("a moving Meshy death action must own the NWN death-family fall");
        let death_fall = output
            .clips
            .iter()
            .find(|clip| clip.name == "ckdbck")
            .expect("one-shot death-family fall");
        assert_eq!(death_fall.length_seconds, 3.0);
        assert_eq!(death_fall.transition_seconds, dead.transition_seconds);
        assert_eq!(death_fall.events, dead.events);
        assert_eq!(death_fall.tracks, dead.tracks);

        for hold_name in ["ckdbckps", "ckdbckdie", "cdead"] {
            let hold = output
                .clips
                .iter()
                .find(|clip| clip.name == hold_name)
                .expect("terminal death-family hold");
            assert_eq!(hold.length_seconds, 1.0 / 30.0);
            assert_eq!(hold.transition_seconds, 0.0);
            assert!(hold.events.is_empty());
            assert!(hold.tracks.iter().all(|track| {
                track.times_seconds == [0.0, 1.0 / 30.0]
                    && track.values.len() == 2
                    && track.values[0] == track.values[1]
            }));
            assert_eq!(
                hold.tracks[0].values[0],
                *dead.tracks[0].values.last().expect("terminal death pose")
            );
        }
    }

    #[test]
    fn moving_explicit_cdead_replaces_a_separate_knockdown_in_the_death_family() {
        let mut source = idle_with_required_rotations();
        let mut knockdown = source.clips[0].clone();
        knockdown.name = "ckdbck".to_owned();
        knockdown.length_seconds = 2.5;
        knockdown.tracks[0].values[1] = vec![0.4, 0.0, 0.0, 0.9165151];
        source.clips.push(knockdown);

        let mut dead = source.clips[0].clone();
        dead.name = "cdead".to_owned();
        dead.length_seconds = 3.0;
        dead.tracks[0].values[1] = vec![0.0, 0.4, 0.0, 0.9165151];
        source.clips.push(dead.clone());

        let authored = author_procedural_humanoid_full_native_42_v2(&source, semantic_rig())
            .expect("the accepted Meshy Dead clip must replace a concatenated Knock_Down");
        let output = authored.animations;
        let death_fall = output
            .clips
            .iter()
            .find(|clip| clip.name == "ckdbck")
            .expect("one death-family fall");
        assert_eq!(death_fall.length_seconds, dead.length_seconds);
        assert_eq!(death_fall.tracks, dead.tracks);
        assert_ne!(death_fall.length_seconds, 2.5);
        assert_eq!(authored.lineage.schema_version, 2);
        assert_eq!(authored.lineage.input_source_clip_count, 3);
        assert_eq!(authored.lineage.preserved_source_clip_count, 2);
        assert_eq!(authored.lineage.source_derived_clip_count, 3);
        assert_eq!(authored.lineage.procedural_clip_count, 37);
        assert_eq!(authored.lineage.discarded_source_clips, ["ckdbck"]);
        assert_eq!(
            authored
                .lineage
                .clips
                .iter()
                .find(|clip| clip.clip_name == "ckdbck")
                .expect("routed death clip")
                .source_clip_name
                .as_deref(),
            Some("cdead"),
        );
    }

    #[test]
    fn explicit_death_transition_derives_cdead_from_its_terminal_pose() {
        let mut source = idle_with_required_rotations();
        let mut dead = source.clips[0].clone();
        dead.name = "ckdbckdie".to_owned();
        dead.length_seconds = 3.0;
        dead.tracks[0].values[1] = vec![0.0, 0.4, 0.0, 0.9165151];
        source.clips.push(dead.clone());

        let output = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect("an explicit one-shot death transition must derive its corpse hold");
        let death_transition = output
            .clips
            .iter()
            .find(|clip| clip.name == "ckdbckdie")
            .expect("one-shot death transition");
        assert_eq!(death_transition.length_seconds, dead.length_seconds);
        assert_eq!(death_transition.tracks, dead.tracks);

        let corpse_hold = output
            .clips
            .iter()
            .find(|clip| clip.name == "cdead")
            .expect("derived terminal corpse hold");
        assert_eq!(corpse_hold.length_seconds, 1.0 / 30.0);
        assert!(corpse_hold.tracks.iter().all(|track| {
            track.times_seconds == [0.0, 1.0 / 30.0]
                && track.values.len() == 2
                && track.values[0] == track.values[1]
        }));
        assert_eq!(
            corpse_hold.tracks[0].values[0],
            *dead.tracks[0].values.last().expect("terminal death pose")
        );
    }

    #[test]
    fn procedural_profile_normalizes_time_zero_and_closes_the_knockdown_recovery_chain() {
        let mut source = idle_with_required_rotations();
        source.clips[0].tracks.push(MdlAnimationTrackV1 {
            target_node_id: semantic_rig().hips,
            path: MdlAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            times_seconds: vec![0.0, 1.0],
            values: vec![vec![0.0, 0.0, 0.0]; 2],
        });

        let mut dead = source.clips[0].clone();
        dead.name = "cdead".to_owned();
        dead.length_seconds = 3.0;
        for track in &mut dead.tracks {
            match track.path {
                MdlAnimationTrackPathV1::Rotation => {
                    track.values[1] = vec![0.70710677, 0.0, 0.0, 0.70710677];
                }
                MdlAnimationTrackPathV1::Translation => {
                    track.values[1] = vec![0.4, -0.7, 0.2];
                }
                _ => {}
            }
        }
        source.clips.push(dead);

        let mut arise = source.clips[0].clone();
        arise.name = "cguptokdb".to_owned();
        arise.length_seconds = 2.0;
        for track in &mut arise.tracks {
            track.times_seconds[0] = 1.0 / 30.0;
            match track.path {
                MdlAnimationTrackPathV1::Rotation => {
                    track.values[0] = vec![0.0, 0.70710677, 0.0, 0.70710677];
                }
                MdlAnimationTrackPathV1::Translation => {
                    track.values[0] = vec![1.0, 0.0, 0.0];
                }
                _ => {}
            }
        }
        source.clips.push(arise);

        let output = author_procedural_humanoid_full_native_42_v2(&source, semantic_rig())
            .expect("recovery must be normalized without concatenating unrelated source origins")
            .animations;
        assert!(
            output
                .clips
                .iter()
                .flat_map(|clip| &clip.tracks)
                .all(|track| track.times_seconds.first() == Some(&0.0))
        );

        for (before_name, after_name) in [
            ("ckdbck", "ckdbckps"),
            ("ckdbckps", "cguptokdb"),
            ("cguptokdb", "cgustandb"),
            ("cgustandb", "cpause1"),
        ] {
            let before = output
                .clips
                .iter()
                .find(|clip| clip.name == before_name)
                .expect("predecessor");
            let after = output
                .clips
                .iter()
                .find(|clip| clip.name == after_name)
                .expect("successor");
            for before_track in &before.tracks {
                let after_track = after
                    .tracks
                    .iter()
                    .find(|track| {
                        track.target_node_id == before_track.target_node_id
                            && track.path == before_track.path
                    })
                    .expect("matching recovery track");
                let before_value = before_track.values.last().expect("terminal value");
                let after_value = after_track.values.first().expect("initial value");
                assert!(
                    controller_boundary_values_match(
                        match before_track.path {
                            MdlAnimationTrackPathV1::Translation => 8,
                            MdlAnimationTrackPathV1::Rotation => 20,
                            _ => 0,
                        },
                        before_value,
                        after_value,
                    ),
                    "{before_name} -> {after_name} differs for node {} {:?}: {:?} -> {:?}",
                    before_track.target_node_id,
                    before_track.path,
                    before_value,
                    after_value,
                );
            }
        }
    }

    #[test]
    fn procedural_event_timing_uses_motion_peaks_instead_of_fixed_clip_percentages() {
        let mut source = idle_with_required_rotations();
        let mut attack = source.clips[0].clone();
        attack.name = "ca1slashl".to_owned();
        attack.length_seconds = 1.0;
        let arm = attack
            .tracks
            .iter_mut()
            .find(|track| track.target_node_id == semantic_rig().right_forearm)
            .expect("right forearm");
        arm.times_seconds = vec![0.0, 0.2, 0.7, 1.0];
        arm.values = vec![
            vec![0.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 0.05, 0.9987492],
            vec![0.0, 0.0, 0.8, 0.6],
            vec![0.0, 0.0, 0.81, 0.5864299],
        ];
        source.clips.push(attack);

        let animations = author_procedural_humanoid_full_native_42_v2(&source, semantic_rig())
            .expect("procedural animation set")
            .animations;
        let authoring = author_procedural_common_native_events_v2(&animations, semantic_rig())
            .expect("kinematic event authoring");
        let hit = authoring
            .clips
            .iter()
            .find(|clip| clip.clip_name == "ca1slashl")
            .and_then(|clip| clip.events.iter().find(|event| event.name == "hit"))
            .expect("attack hit event");
        assert_eq!(hit.time_seconds, 0.7);
        assert_ne!(hit.time_seconds, 0.55);
    }

    #[test]
    fn procedural_profile_rejects_unknown_explicit_clip_names() {
        let mut source = idle_with_required_rotations();
        let mut unknown = source.clips[0].clone();
        unknown.name = "meshy_action_198".to_owned();
        source.clips.push(unknown);

        let error = author_procedural_humanoid_full_native_42_v1(&source, semantic_rig())
            .expect_err("paid clips without an explicit NWN semantic must fail closed");
        assert_eq!(error.code, "M6-PROCEDURAL-HUMANOID-CLIP-NAME");
    }
}
