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
pub struct DirectCreatureAnimationCompletenessV1 {
    pub profile: DirectCreatureAnimationProfileV1,
    pub required_clip_count: u32,
    pub explicit_clip_count: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub procedural_clip_count: u32,
    pub fallback_alias_count: u32,
    pub complete: bool,
}

fn is_zero_u32(value: &u32) -> bool {
    *value == 0
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

/// Authors a complete direct-creature namespace from one caller-owned humanoid
/// idle clip and an explicit semantic joint binding.
///
/// The motions are deterministic clean-room deltas layered over the source
/// pose. This route does not rename one idle clip 42 times: each non-idle state
/// receives distinct controller content, locomotion uses opposed limb motion,
/// attacks/casts use arm and torso motion, and terminal states preserve a
/// non-looping final pose.
pub fn author_procedural_humanoid_full_native_42_v1(
    source: &MdlAnimationSetV1,
    rig: ProceduralHumanoidRigV1,
) -> Result<MdlAnimationSetV1, ProceduralHumanoidAnimationErrorV1> {
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
    let mut normalized_idle = idle.clone();
    for track in normalized_idle
        .tracks
        .iter()
        .filter(|track| track.path == MdlAnimationTrackPathV1::Scale)
    {
        require_removable_constant_scale_track(track)?;
    }
    normalized_idle
        .tracks
        .retain(|track| track.path != MdlAnimationTrackPathV1::Scale);

    let mut clips = Vec::with_capacity(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len());
    for (clip_index, clip_name) in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().enumerate() {
        if *clip_name == "cpause1" {
            clips.push(normalized_idle.clone());
            continue;
        }
        clips.push(author_procedural_clip(
            &normalized_idle,
            rig,
            clip_name,
            clip_index,
        )?);
    }
    Ok(MdlAnimationSetV1 {
        schema_version: 1,
        clips,
    })
}

/// Creates clean-room gameplay callback timings for the procedural profile.
///
/// Names come from the independently confirmed public 42-state/event
/// namespace. Timings are authored here as normalized fractions of each
/// generated clip and are not copied from a retail animation.
pub fn author_procedural_common_native_events_v1(
    animations: &MdlAnimationSetV1,
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
                let fraction = match *event_name {
                    "hit" => 0.55,
                    "cast" => 0.60,
                    "snd_hitground" => 0.82,
                    "snd_footstep" => 0.35,
                    _ => 0.50,
                };
                MdlAnimationEventV1 {
                    time_seconds: clip.length_seconds * fraction,
                    name: (*event_name).to_owned(),
                }
            })
            .collect::<Vec<_>>();
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

fn author_procedural_clip(
    idle: &MdlAnimationClipV1,
    rig: ProceduralHumanoidRigV1,
    clip_name: &str,
    clip_index: usize,
) -> Result<MdlAnimationClipV1, ProceduralHumanoidAnimationErrorV1> {
    let terminal = matches!(
        clip_name,
        "ckdbckdie" | "cdead" | "cdisappear" | "cdisappearlp"
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
const FAMILY_VARIABLE_MOTION_CLIPS_V1: [&str; 3] = ["ccastoutlp", "cgetmidlp", "cdead"];
const ESSENTIAL_BEHAVIOR_CLIPS_V1: [&str; 7] = [
    "cpause1",
    "cwalk",
    "crun",
    "ca1slashl",
    "cdamagel",
    "ckdbckdie",
    "cdead",
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
pub struct DirectCreatureAnimationBehaviorV1 {
    pub schema_version: u32,
    pub required_namespace_clip_count: u32,
    pub observed_clip_count: u32,
    pub full_namespace_complete: bool,
    pub all_required_content_present: bool,
    pub active_motion_complete: bool,
    pub walk_run_distinct: bool,
    pub essential_states_distinct: bool,
    pub death_transition_terminal_pose: bool,
    pub behavior_candidate_eligible: bool,
    pub clips: Vec<DirectCreatureClipBehaviorV1>,
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
/// changing controllers for every consistently active state, distinct walk/run
/// semantics, and a terminal pose transition in `ckdbckdie`. The three
/// family-variable states require content but may be static or moving; local
/// native families demonstrate both valid forms.
pub fn evaluate_direct_creature_animation_behavior_v1(
    report: &InspectionReport,
) -> DirectCreatureAnimationBehaviorV1 {
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
    let death_transition_terminal_pose = clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("ckdbckdie"))
        .is_some_and(|clip| clip.terminal_pose_controller_count > 0);

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
    let behavior_candidate_eligible = violations.is_empty();

    DirectCreatureAnimationBehaviorV1 {
        schema_version: 1,
        required_namespace_clip_count: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() as u32,
        observed_clip_count: clips.len() as u32,
        full_namespace_complete,
        all_required_content_present,
        active_motion_complete,
        walk_run_distinct,
        essential_states_distinct,
        death_transition_terminal_pose,
        behavior_candidate_eligible,
        clips,
        violations,
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
}
