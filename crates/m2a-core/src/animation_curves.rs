//! Editor-only curve representation with deterministic, error-bounded
//! resampling to the ordinary LINEAR animation tracks accepted by the writer.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_studio::{
        AnimationKeyframeV1, AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1,
    },
    mdl::MdlAnimationInterpolationV1,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveKeyV1 {
    pub id: String,
    pub time_seconds: f32,
    pub value: Vec<f32>,
    pub in_tangent: Option<Vec<f32>>,
    pub out_tangent: Option<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEditorCurveTrackV1 {
    pub schema_version: u32,
    pub id: String,
    pub target_node_id: u32,
    pub path: AuthoredAnimationTrackPathV1,
    pub keys: Vec<AnimationCurveKeyV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveResamplePolicyV1 {
    pub schema_version: u32,
    pub max_position_error: f32,
    pub max_angular_error_radians: f32,
    pub max_subdivision_depth: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveResampleReportV1 {
    pub schema_version: u32,
    pub editor_key_count: usize,
    pub linear_key_count: usize,
    pub max_position_error: f32,
    pub max_angular_error_radians: f32,
    pub track: AuthoredAnimationTrackV1,
    pub fingerprint_sha256: String,
}

pub fn resample_animation_curve_to_linear_v1(
    curve: &AnimationEditorCurveTrackV1,
    policy: &AnimationCurveResamplePolicyV1,
) -> Result<AnimationCurveResampleReportV1, String> {
    validate_curve(curve, policy)?;
    let mut sampled = vec![(curve.keys[0].time_seconds, curve.keys[0].value.clone())];
    let mut max_position_error = 0.0_f32;
    let mut max_angular_error_radians = 0.0_f32;
    for pair in curve.keys.windows(2) {
        adaptive_segment(
            curve.path,
            &pair[0],
            &pair[1],
            policy,
            0,
            &mut sampled,
            &mut max_position_error,
            &mut max_angular_error_radians,
        )?;
    }
    let keyframes = sampled
        .into_iter()
        .enumerate()
        .map(|(index, (time_seconds, value))| AnimationKeyframeV1 {
            id: format!("curve-linear-{index:05}"),
            time_seconds: canonical_f32(time_seconds),
            value: value.into_iter().map(canonical_f32).collect(),
        })
        .collect::<Vec<_>>();
    let track = AuthoredAnimationTrackV1 {
        id: curve.id.clone(),
        target_node_id: curve.target_node_id,
        path: curve.path,
        interpolation: MdlAnimationInterpolationV1::Linear,
        keyframes,
    };
    let fingerprint_sha256 = fingerprint_json(&(curve, policy, &track));
    Ok(AnimationCurveResampleReportV1 {
        schema_version: 1,
        editor_key_count: curve.keys.len(),
        linear_key_count: track.keyframes.len(),
        max_position_error: canonical_f32(max_position_error),
        max_angular_error_radians: canonical_f32(max_angular_error_radians),
        track,
        fingerprint_sha256,
    })
}

#[allow(clippy::too_many_arguments)]
fn adaptive_segment(
    path: AuthoredAnimationTrackPathV1,
    start: &AnimationCurveKeyV1,
    end: &AnimationCurveKeyV1,
    policy: &AnimationCurveResamplePolicyV1,
    depth: u8,
    output: &mut Vec<(f32, Vec<f32>)>,
    max_position_error: &mut f32,
    max_angular_error_radians: &mut f32,
) -> Result<(), String> {
    let mid_time = (start.time_seconds + end.time_seconds) * 0.5;
    let curve_mid = evaluate_segment(path, start, end, 0.5)?;
    let linear_mid = interpolate_linear(path, &start.value, &end.value, 0.5)?;
    let error = curve_error(path, &curve_mid, &linear_mid)?;
    let tolerance = match path {
        AuthoredAnimationTrackPathV1::Translation => policy.max_position_error,
        AuthoredAnimationTrackPathV1::Rotation => policy.max_angular_error_radians,
    };
    match path {
        AuthoredAnimationTrackPathV1::Translation => {
            *max_position_error = (*max_position_error).max(error)
        }
        AuthoredAnimationTrackPathV1::Rotation => {
            *max_angular_error_radians = (*max_angular_error_radians).max(error)
        }
    }
    if error > tolerance && depth < policy.max_subdivision_depth {
        let mid = AnimationCurveKeyV1 {
            id: format!("adaptive-{depth}"),
            time_seconds: mid_time,
            value: curve_mid,
            in_tangent: tangent_at(path, start, end, 0.5)?,
            out_tangent: tangent_at(path, start, end, 0.5)?,
        };
        adaptive_segment(
            path,
            start,
            &mid,
            policy,
            depth + 1,
            output,
            max_position_error,
            max_angular_error_radians,
        )?;
        adaptive_segment(
            path,
            &mid,
            end,
            policy,
            depth + 1,
            output,
            max_position_error,
            max_angular_error_radians,
        )?;
    } else {
        output.push((end.time_seconds, end.value.clone()));
    }
    Ok(())
}

fn validate_curve(
    curve: &AnimationEditorCurveTrackV1,
    policy: &AnimationCurveResamplePolicyV1,
) -> Result<(), String> {
    if curve.schema_version != 1
        || policy.schema_version != 1
        || curve.id.trim().is_empty()
        || curve.keys.len() < 2
        || curve.keys.len() > 4096
        || !policy.max_position_error.is_finite()
        || policy.max_position_error <= 0.0
        || !policy.max_angular_error_radians.is_finite()
        || policy.max_angular_error_radians <= 0.0
        || !(1..=16).contains(&policy.max_subdivision_depth)
    {
        return Err("curve or resampling policy is invalid".into());
    }
    let expected = match curve.path {
        AuthoredAnimationTrackPathV1::Translation => 3,
        AuthoredAnimationTrackPathV1::Rotation => 4,
    };
    for (index, key) in curve.keys.iter().enumerate() {
        if key.id.trim().is_empty()
            || !key.time_seconds.is_finite()
            || key.value.len() != expected
            || key.value.iter().any(|value| !value.is_finite())
            || index > 0 && key.time_seconds <= curve.keys[index - 1].time_seconds
        {
            return Err(format!(
                "curve key {index} is invalid or not strictly ordered"
            ));
        }
        match curve.path {
            AuthoredAnimationTrackPathV1::Translation => {
                for tangent in [&key.in_tangent, &key.out_tangent] {
                    if tangent.as_ref().is_some_and(|value| {
                        value.len() != 3 || value.iter().any(|v| !v.is_finite())
                    }) {
                        return Err(format!("translation tangent at key {index} is invalid"));
                    }
                }
            }
            AuthoredAnimationTrackPathV1::Rotation => {
                if key.in_tangent.is_some() || key.out_tangent.is_some() {
                    return Err("rotation curves use shortest-arc quaternion interpolation without component tangents".into());
                }
                normalize_quaternion(&key.value)?;
            }
        }
    }
    Ok(())
}

fn evaluate_segment(
    path: AuthoredAnimationTrackPathV1,
    start: &AnimationCurveKeyV1,
    end: &AnimationCurveKeyV1,
    t: f32,
) -> Result<Vec<f32>, String> {
    match path {
        AuthoredAnimationTrackPathV1::Translation => {
            let duration = end.time_seconds - start.time_seconds;
            let m0 = start.out_tangent.clone().unwrap_or_else(|| vec![0.0; 3]);
            let m1 = end.in_tangent.clone().unwrap_or_else(|| vec![0.0; 3]);
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            Ok((0..3)
                .map(|axis| {
                    h00 * start.value[axis]
                        + h10 * duration * m0[axis]
                        + h01 * end.value[axis]
                        + h11 * duration * m1[axis]
                })
                .collect())
        }
        AuthoredAnimationTrackPathV1::Rotation => {
            interpolate_linear(path, &start.value, &end.value, t)
        }
    }
}

fn tangent_at(
    path: AuthoredAnimationTrackPathV1,
    start: &AnimationCurveKeyV1,
    end: &AnimationCurveKeyV1,
    t: f32,
) -> Result<Option<Vec<f32>>, String> {
    if path == AuthoredAnimationTrackPathV1::Rotation {
        return Ok(None);
    }
    let duration = end.time_seconds - start.time_seconds;
    let m0 = start.out_tangent.clone().unwrap_or_else(|| vec![0.0; 3]);
    let m1 = end.in_tangent.clone().unwrap_or_else(|| vec![0.0; 3]);
    let t2 = t * t;
    Ok(Some(
        (0..3)
            .map(|axis| {
                ((6.0 * t2 - 6.0 * t) * start.value[axis]
                    + (3.0 * t2 - 4.0 * t + 1.0) * duration * m0[axis]
                    + (-6.0 * t2 + 6.0 * t) * end.value[axis]
                    + (3.0 * t2 - 2.0 * t) * duration * m1[axis])
                    / duration
            })
            .collect(),
    ))
}

fn interpolate_linear(
    path: AuthoredAnimationTrackPathV1,
    start: &[f32],
    end: &[f32],
    t: f32,
) -> Result<Vec<f32>, String> {
    if path == AuthoredAnimationTrackPathV1::Translation {
        return Ok(start
            .iter()
            .zip(end)
            .map(|(a, b)| a + (b - a) * t)
            .collect());
    }
    let left = normalize_quaternion(start)?;
    let mut right = normalize_quaternion(end)?;
    if left.iter().zip(&right).map(|(a, b)| a * b).sum::<f32>() < 0.0 {
        right.iter_mut().for_each(|value| *value = -*value);
    }
    normalize_quaternion(
        &left
            .iter()
            .zip(&right)
            .map(|(a, b)| a + (b - a) * t)
            .collect::<Vec<_>>(),
    )
}

fn curve_error(
    path: AuthoredAnimationTrackPathV1,
    curve: &[f32],
    linear: &[f32],
) -> Result<f32, String> {
    if path == AuthoredAnimationTrackPathV1::Translation {
        return Ok(curve
            .iter()
            .zip(linear)
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt());
    }
    let curve = normalize_quaternion(curve)?;
    let linear = normalize_quaternion(linear)?;
    let dot = curve
        .iter()
        .zip(&linear)
        .map(|(a, b)| a * b)
        .sum::<f32>()
        .abs();
    Ok(2.0 * dot.clamp(-1.0, 1.0).acos())
}

fn normalize_quaternion(value: &[f32]) -> Result<Vec<f32>, String> {
    if value.len() != 4 || value.iter().any(|component| !component.is_finite()) {
        return Err("quaternion must contain four finite components".into());
    }
    let length = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !length.is_finite() || length <= 1.0e-8 {
        return Err("quaternion must have non-zero finite length".into());
    }
    Ok(value.iter().map(|component| component / length).collect())
}

fn canonical_f32(value: f32) -> f32 {
    f32::from_bits(value.to_bits())
}

fn fingerprint_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("curve contracts serialize");
    format!("{:x}", Sha256::digest(bytes))
}
