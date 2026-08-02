//! Deterministic multi-clip preview composition. Sequence previews are
//! ephemeral authoring evidence; they never replace the ordinary authored
//! clips consumed by the Aurora writer.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_studio::{
        AnimationKeyframeV1, AuthoredAnimationClipKindV1, AuthoredAnimationClipStatusV1,
        AuthoredAnimationClipV1, AuthoredAnimationEventV1, AuthoredAnimationSourceKindV1,
        AuthoredAnimationSourceV1, AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1,
        sample_animation_track_linear_v1,
    },
    mdl::MdlAnimationInterpolationV1,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequencePreviewRequestV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub sequence_id: String,
    pub output_name: String,
    pub clip_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceSegmentV1 {
    pub clip_id: String,
    pub clip_name: String,
    pub start_seconds: f32,
    pub end_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceTransitionJumpV1 {
    pub from_clip_id: String,
    pub to_clip_id: String,
    pub boundary_seconds: f32,
    pub max_translation_delta: f32,
    pub max_translation_node_id: Option<u32>,
    pub max_angular_delta_radians: f32,
    pub max_angular_node_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequencePreviewV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub segments: Vec<AnimationSequenceSegmentV1>,
    pub transition_jumps: Vec<AnimationSequenceTransitionJumpV1>,
    pub preview_clip: AuthoredAnimationClipV1,
    pub fingerprint_sha256: String,
}

pub fn build_animation_sequence_preview_v1(
    request: &AnimationSequencePreviewRequestV1,
    available_clips: &[AuthoredAnimationClipV1],
) -> Result<AnimationSequencePreviewV1, String> {
    if request.schema_version != 1
        || request.sequence_id.trim().is_empty()
        || request.output_name.trim().is_empty()
        || request.clip_ids.len() < 2
        || request.clip_ids.len() > 16
        || !is_sha256(&request.source_revision)
    {
        return Err("sequence request has invalid schema, identity, lineage or clip count".into());
    }
    let by_id = available_clips
        .iter()
        .map(|clip| (clip.id.as_str(), clip))
        .collect::<BTreeMap<_, _>>();
    let mut clips = Vec::with_capacity(request.clip_ids.len());
    for clip_id in &request.clip_ids {
        let clip = by_id
            .get(clip_id.as_str())
            .copied()
            .ok_or_else(|| format!("sequence clip {clip_id:?} does not exist"))?;
        if clip.source.source_revision != request.source_revision
            || !clip.length_seconds.is_finite()
            || clip.length_seconds <= 0.0
        {
            return Err(format!(
                "sequence clip {:?} is stale or has an invalid duration",
                clip.id
            ));
        }
        clips.push(clip);
    }

    let mut segments = Vec::with_capacity(clips.len());
    let mut cursor = 0.0_f32;
    for clip in &clips {
        let start_seconds = canonical_f32(cursor);
        cursor += clip.length_seconds;
        segments.push(AnimationSequenceSegmentV1 {
            clip_id: clip.id.clone(),
            clip_name: clip.name.clone(),
            start_seconds,
            end_seconds: canonical_f32(cursor),
        });
    }
    let transition_jumps = clips
        .windows(2)
        .zip(segments.iter().skip(1))
        .map(|(pair, next_segment)| {
            measure_transition_jump(pair[0], pair[1], next_segment.start_seconds)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut track_keys =
        BTreeMap::<(u32, AuthoredAnimationTrackPathV1), Vec<AnimationKeyframeV1>>::new();
    for (clip_index, (clip, segment)) in clips.iter().zip(&segments).enumerate() {
        for track in &clip.tracks {
            if track.interpolation != MdlAnimationInterpolationV1::Linear {
                return Err(format!("sequence track {:?} is not LINEAR", track.id));
            }
            let output = track_keys
                .entry((track.target_node_id, track.path))
                .or_default();
            for (key_index, key) in track.keyframes.iter().enumerate() {
                let shifted = canonical_f32(segment.start_seconds + key.time_seconds);
                if let Some(last) = output.last_mut()
                    && (last.time_seconds - shifted).abs() <= 1.0e-7
                {
                    *last = AnimationKeyframeV1 {
                        id: format!("sequence-{clip_index}-{key_index:05}"),
                        time_seconds: shifted,
                        value: key.value.clone(),
                    };
                    continue;
                }
                output.push(AnimationKeyframeV1 {
                    id: format!("sequence-{clip_index}-{key_index:05}"),
                    time_seconds: shifted,
                    value: key.value.clone(),
                });
            }
        }
    }
    let tracks = track_keys
        .into_iter()
        .map(
            |((target_node_id, path), keyframes)| AuthoredAnimationTrackV1 {
                id: format!("sequence-{target_node_id}-{}", path_label(path)),
                target_node_id,
                path,
                interpolation: MdlAnimationInterpolationV1::Linear,
                keyframes,
            },
        )
        .collect::<Vec<_>>();
    let events = clips
        .iter()
        .zip(&segments)
        .enumerate()
        .flat_map(|(clip_index, (clip, segment))| {
            clip.events
                .iter()
                .enumerate()
                .map(move |(event_index, event)| AuthoredAnimationEventV1 {
                    id: format!("sequence-event-{clip_index}-{event_index:05}"),
                    time_seconds: canonical_f32(segment.start_seconds + event.time_seconds),
                    name: event.name.clone(),
                })
        })
        .collect::<Vec<_>>();
    let preview_clip = AuthoredAnimationClipV1 {
        id: request.sequence_id.clone(),
        name: request.output_name.clone(),
        kind: AuthoredAnimationClipKindV1::Motion,
        status: AuthoredAnimationClipStatusV1::Draft,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::ProceduralTemplate,
            source_revision: request.source_revision.clone(),
            source_clip_name: None,
            source_clip_fingerprint: None,
            procedural_template: Some("ANIMATION_SEQUENCE_PREVIEW_V1".into()),
            library_preset: None,
            retarget: None,
        },
        length_seconds: canonical_f32(cursor),
        transition_seconds: 0.0,
        animation_root: clips[0].animation_root.clone(),
        tracks,
        events,
        revision: 1,
    };
    let fingerprint_sha256 =
        fingerprint_json(&(request, &segments, &transition_jumps, &preview_clip));
    Ok(AnimationSequencePreviewV1 {
        schema_version: 1,
        source_revision: request.source_revision.clone(),
        segments,
        transition_jumps,
        preview_clip,
        fingerprint_sha256,
    })
}

fn measure_transition_jump(
    from: &AuthoredAnimationClipV1,
    to: &AuthoredAnimationClipV1,
    boundary_seconds: f32,
) -> Result<AnimationSequenceTransitionJumpV1, String> {
    let from_tracks = from
        .tracks
        .iter()
        .map(|track| ((track.target_node_id, track.path), track))
        .collect::<BTreeMap<_, _>>();
    let to_tracks = to
        .tracks
        .iter()
        .map(|track| ((track.target_node_id, track.path), track))
        .collect::<BTreeMap<_, _>>();
    let pairs = from_tracks
        .keys()
        .chain(to_tracks.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut max_translation_delta = 0.0_f32;
    let mut max_translation_node_id = None;
    let mut max_angular_delta_radians = 0.0_f32;
    let mut max_angular_node_id = None;
    for (node_id, path) in pairs {
        let Some((from_track, to_track)) = from_tracks
            .get(&(node_id, path))
            .zip(to_tracks.get(&(node_id, path)))
        else {
            continue;
        };
        let left = sample_animation_track_linear_v1(from_track, from.length_seconds)
            .map_err(|diagnostic| diagnostic.message)?;
        let right = sample_animation_track_linear_v1(to_track, 0.0)
            .map_err(|diagnostic| diagnostic.message)?;
        match path {
            AuthoredAnimationTrackPathV1::Translation => {
                let delta = left
                    .iter()
                    .zip(&right)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f32>()
                    .sqrt();
                if delta > max_translation_delta {
                    max_translation_delta = delta;
                    max_translation_node_id = Some(node_id);
                }
            }
            AuthoredAnimationTrackPathV1::Rotation => {
                if left.len() != 4 || right.len() != 4 {
                    return Err("rotation transition value must contain four components".into());
                }
                let dot = left
                    .iter()
                    .zip(&right)
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
                    .abs();
                let delta = 2.0 * dot.clamp(-1.0, 1.0).acos();
                if delta > max_angular_delta_radians {
                    max_angular_delta_radians = delta;
                    max_angular_node_id = Some(node_id);
                }
            }
        }
    }
    Ok(AnimationSequenceTransitionJumpV1 {
        from_clip_id: from.id.clone(),
        to_clip_id: to.id.clone(),
        boundary_seconds,
        max_translation_delta: canonical_f32(max_translation_delta),
        max_translation_node_id,
        max_angular_delta_radians: canonical_f32(max_angular_delta_radians),
        max_angular_node_id,
    })
}

fn path_label(path: AuthoredAnimationTrackPathV1) -> &'static str {
    match path {
        AuthoredAnimationTrackPathV1::Translation => "translation",
        AuthoredAnimationTrackPathV1::Rotation => "rotation",
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonical_f32(value: f32) -> f32 {
    f32::from_bits(value.to_bits())
}

fn fingerprint_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("sequence contracts serialize");
    format!("{:x}", Sha256::digest(bytes))
}

/// Persistent, editable sequence authoring contract. Unlike the legacy preview
/// request above, this document records trim, repeat, transitions and phase
/// markers and can be saved in a project before it is baked to a Custom clip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceDocumentV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub sequence_id: String,
    pub output_name: String,
    pub segments: Vec<AnimationSequenceSegmentRecipeV1>,
    pub revision: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationSequenceTransitionKindV1 {
    Cut,
    CrossFade,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceTransitionV1 {
    pub kind: AnimationSequenceTransitionKindV1,
    pub duration_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequencePhaseMarkerV1 {
    pub kind: String,
    pub time_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceSegmentRecipeV1 {
    pub segment_id: String,
    pub clip_id: String,
    pub source_in_seconds: f32,
    pub source_out_seconds: f32,
    pub repeat_count: u32,
    pub transition_from_previous: AnimationSequenceTransitionV1,
    #[serde(default)]
    pub phase_markers: Vec<AnimationSequencePhaseMarkerV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequencePlacedSegmentV1 {
    pub segment_id: String,
    pub clip_id: String,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub source_in_seconds: f32,
    pub source_out_seconds: f32,
    pub repeat_count: u32,
    pub transition_from_previous: AnimationSequenceTransitionV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequencePlacedPhaseMarkerV1 {
    pub segment_id: String,
    pub kind: String,
    pub time_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceSourceProvenanceV1 {
    pub clip_id: String,
    pub clip_revision: u64,
    pub clip_fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSequenceBakeV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub document_revision: u32,
    pub placed_segments: Vec<AnimationSequencePlacedSegmentV1>,
    pub phase_markers: Vec<AnimationSequencePlacedPhaseMarkerV1>,
    pub source_provenance: Vec<AnimationSequenceSourceProvenanceV1>,
    pub preview_clip: AuthoredAnimationClipV1,
    pub custom_clip: AuthoredAnimationClipV1,
    pub fingerprint_sha256: String,
}

/// Bakes an arbitrary ordered set of trimmed/repeated clips. Cross-fades are
/// sampled at 60 Hz and baked to ordinary LINEAR tracks, so no new runtime
/// interpolation semantics leak into the Aurora writer.
pub fn bake_animation_sequence_document_v1(
    document: &AnimationSequenceDocumentV1,
    available_clips: &[AuthoredAnimationClipV1],
) -> Result<AnimationSequenceBakeV1, String> {
    if document.schema_version != 1
        || document.revision == 0
        || document.sequence_id.trim().is_empty()
        || document.output_name.trim().is_empty()
        || !is_sha256(&document.source_revision)
        || document.segments.is_empty()
        || document.segments.len() > 64
    {
        return Err(
            "sequence document has invalid schema, identity, revision or segment count".into(),
        );
    }
    let by_id = available_clips
        .iter()
        .map(|clip| (clip.id.as_str(), clip))
        .collect::<BTreeMap<_, _>>();
    let mut seen_segments = BTreeSet::new();
    let mut placed = Vec::with_capacity(document.segments.len());
    let mut sources = Vec::with_capacity(document.segments.len());
    let mut cursor = 0.0_f32;
    for (index, segment) in document.segments.iter().enumerate() {
        if segment.segment_id.trim().is_empty()
            || !seen_segments.insert(segment.segment_id.as_str())
        {
            return Err("sequence segment ids must be non-empty and unique".into());
        }
        let clip = by_id
            .get(segment.clip_id.as_str())
            .copied()
            .ok_or_else(|| format!("sequence clip {:?} does not exist", segment.clip_id))?;
        if clip.source.source_revision != document.source_revision
            || !clip.length_seconds.is_finite()
            || clip.length_seconds <= 0.0
            || !segment.source_in_seconds.is_finite()
            || !segment.source_out_seconds.is_finite()
            || segment.source_in_seconds < 0.0
            || segment.source_out_seconds <= segment.source_in_seconds
            || segment.source_out_seconds > clip.length_seconds + 1.0e-6
            || !(1..=64).contains(&segment.repeat_count)
        {
            return Err(format!(
                "sequence segment {:?} is stale or has invalid trim/repeat",
                segment.segment_id
            ));
        }
        let transition = &segment.transition_from_previous;
        if !transition.duration_seconds.is_finite() || transition.duration_seconds < 0.0 {
            return Err("sequence transition duration must be finite and non-negative".into());
        }
        if index == 0 && transition.duration_seconds > 0.0 {
            return Err("the first sequence segment cannot transition from a predecessor".into());
        }
        if transition.kind == AnimationSequenceTransitionKindV1::Cut
            && transition.duration_seconds != 0.0
        {
            return Err("CUT transition duration must be zero".into());
        }
        let span =
            (segment.source_out_seconds - segment.source_in_seconds) * segment.repeat_count as f32;
        if transition.duration_seconds >= span || transition.duration_seconds > cursor {
            return Err("cross-fade must be shorter than both available sequence spans".into());
        }
        let start = cursor - transition.duration_seconds;
        let end = start + span;
        placed.push(AnimationSequencePlacedSegmentV1 {
            segment_id: segment.segment_id.clone(),
            clip_id: segment.clip_id.clone(),
            start_seconds: canonical_f32(start),
            end_seconds: canonical_f32(end),
            source_in_seconds: segment.source_in_seconds,
            source_out_seconds: segment.source_out_seconds,
            repeat_count: segment.repeat_count,
            transition_from_previous: transition.clone(),
        });
        sources.push(AnimationSequenceSourceProvenanceV1 {
            clip_id: clip.id.clone(),
            clip_revision: clip.revision,
            clip_fingerprint_sha256: fingerprint_json(clip),
        });
        cursor = end;
    }
    if cursor > 120.0 {
        return Err("sequence duration exceeds the 120 second authoring limit".into());
    }

    let keys = by_id
        .values()
        .flat_map(|clip| clip.tracks.iter())
        .map(|track| (track.target_node_id, track.path))
        .collect::<BTreeSet<_>>();
    let frame_count = (cursor * 60.0).ceil() as usize;
    let sample_times = (0..=frame_count)
        .map(|frame| canonical_f32((frame as f32 / 60.0).min(cursor)))
        .collect::<Vec<_>>();
    let mut tracks = Vec::with_capacity(keys.len());
    for (node_id, path) in keys {
        let mut keyframes = Vec::with_capacity(sample_times.len());
        for (frame, time) in sample_times.iter().copied().enumerate() {
            let value = sample_sequence_value(document, &placed, &by_id, node_id, path, time)?;
            if let Some(value) = value {
                keyframes.push(AnimationKeyframeV1 {
                    id: format!("sequence-{node_id}-{}-{frame:05}", path_label(path)),
                    time_seconds: time,
                    value,
                });
            }
        }
        if !keyframes.is_empty() {
            tracks.push(AuthoredAnimationTrackV1 {
                id: format!("sequence-{node_id}-{}", path_label(path)),
                target_node_id: node_id,
                path,
                interpolation: MdlAnimationInterpolationV1::Linear,
                keyframes,
            });
        }
    }

    let mut events = Vec::new();
    let mut phase_markers = Vec::new();
    for (segment, placed_segment) in document.segments.iter().zip(&placed) {
        let clip = by_id[segment.clip_id.as_str()];
        let source_span = segment.source_out_seconds - segment.source_in_seconds;
        for repeat in 0..segment.repeat_count {
            let repeat_start = placed_segment.start_seconds + repeat as f32 * source_span;
            for event in &clip.events {
                if event.time_seconds + 1.0e-6 >= segment.source_in_seconds
                    && event.time_seconds <= segment.source_out_seconds + 1.0e-6
                {
                    events.push(AuthoredAnimationEventV1 {
                        id: format!(
                            "sequence-event-{}-{repeat}-{}",
                            segment.segment_id, event.id
                        ),
                        time_seconds: canonical_f32(
                            repeat_start + event.time_seconds - segment.source_in_seconds,
                        ),
                        name: event.name.clone(),
                    });
                }
            }
            for marker in &segment.phase_markers {
                if !marker.time_seconds.is_finite()
                    || marker.time_seconds < segment.source_in_seconds
                    || marker.time_seconds > segment.source_out_seconds
                    || marker.kind.trim().is_empty()
                {
                    return Err(format!(
                        "sequence phase marker in {:?} is invalid",
                        segment.segment_id
                    ));
                }
                phase_markers.push(AnimationSequencePlacedPhaseMarkerV1 {
                    segment_id: segment.segment_id.clone(),
                    kind: marker.kind.clone(),
                    time_seconds: canonical_f32(
                        repeat_start + marker.time_seconds - segment.source_in_seconds,
                    ),
                });
            }
        }
    }
    events.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then(left.id.cmp(&right.id))
    });
    phase_markers.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then(left.segment_id.cmp(&right.segment_id))
    });
    let animation_root = by_id[document.segments[0].clip_id.as_str()]
        .animation_root
        .clone();
    let base = AuthoredAnimationClipV1 {
        id: document.sequence_id.clone(),
        name: document.output_name.clone(),
        kind: AuthoredAnimationClipKindV1::Motion,
        status: AuthoredAnimationClipStatusV1::Draft,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::ProceduralTemplate,
            source_revision: document.source_revision.clone(),
            source_clip_name: None,
            source_clip_fingerprint: None,
            procedural_template: Some("ANIMATION_SEQUENCE_COMPOSER_V1".into()),
            library_preset: None,
            retarget: None,
        },
        length_seconds: canonical_f32(cursor),
        transition_seconds: 0.0,
        animation_root,
        tracks,
        events,
        revision: 1,
    };
    let mut custom_clip = base.clone();
    custom_clip.id = format!("{}-custom", document.sequence_id);
    custom_clip.status = AuthoredAnimationClipStatusV1::Valid;
    let fingerprint_sha256 =
        fingerprint_json(&(document, &placed, &phase_markers, &sources, &base));
    Ok(AnimationSequenceBakeV1 {
        schema_version: 1,
        source_revision: document.source_revision.clone(),
        document_revision: document.revision,
        placed_segments: placed,
        phase_markers,
        source_provenance: sources,
        preview_clip: base,
        custom_clip,
        fingerprint_sha256,
    })
}

fn sample_sequence_value(
    document: &AnimationSequenceDocumentV1,
    placed: &[AnimationSequencePlacedSegmentV1],
    by_id: &BTreeMap<&str, &AuthoredAnimationClipV1>,
    node_id: u32,
    path: AuthoredAnimationTrackPathV1,
    time: f32,
) -> Result<Option<Vec<f32>>, String> {
    let mut active = placed
        .iter()
        .enumerate()
        .filter(|(_, segment)| {
            time + 1.0e-6 >= segment.start_seconds && time <= segment.end_seconds + 1.0e-6
        })
        .collect::<Vec<_>>();
    if active.is_empty() {
        return Ok(None);
    }
    active.sort_by_key(|(index, _)| *index);
    let sample = |index: usize,
                  segment: &AnimationSequencePlacedSegmentV1|
     -> Result<Option<Vec<f32>>, String> {
        let recipe = &document.segments[index];
        let clip = by_id[segment.clip_id.as_str()];
        let source_span = recipe.source_out_seconds - recipe.source_in_seconds;
        let elapsed =
            (time - segment.start_seconds).clamp(0.0, segment.end_seconds - segment.start_seconds);
        let local = if (time - segment.end_seconds).abs() <= 1.0e-6 {
            recipe.source_out_seconds
        } else {
            recipe.source_in_seconds + elapsed.rem_euclid(source_span)
        };
        let Some(track) = clip
            .tracks
            .iter()
            .find(|track| track.target_node_id == node_id && track.path == path)
        else {
            return Ok(None);
        };
        sample_animation_track_linear_v1(track, local)
            .map(Some)
            .map_err(|diagnostic| diagnostic.message)
    };
    let (last_index, last_segment) = active[active.len() - 1];
    let right = sample(last_index, last_segment)?;
    if active.len() < 2
        || last_segment.transition_from_previous.kind == AnimationSequenceTransitionKindV1::Cut
    {
        return Ok(right);
    }
    let (previous_index, previous_segment) = active[active.len() - 2];
    let left = sample(previous_index, previous_segment)?;
    let alpha = ((time - last_segment.start_seconds)
        / last_segment.transition_from_previous.duration_seconds)
        .clamp(0.0, 1.0);
    match (left, right) {
        (Some(left), Some(right)) => blend_sequence_values(path, &left, &right, alpha).map(Some),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn blend_sequence_values(
    path: AuthoredAnimationTrackPathV1,
    left: &[f32],
    right: &[f32],
    alpha: f32,
) -> Result<Vec<f32>, String> {
    match path {
        AuthoredAnimationTrackPathV1::Translation if left.len() == 3 && right.len() == 3 => {
            Ok(left
                .iter()
                .zip(right)
                .map(|(a, b)| canonical_f32(a + (b - a) * alpha))
                .collect())
        }
        AuthoredAnimationTrackPathV1::Rotation if left.len() == 4 && right.len() == 4 => {
            let mut right = [right[0], right[1], right[2], right[3]];
            let left = [left[0], left[1], left[2], left[3]];
            if left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>() < 0.0 {
                right
                    .iter_mut()
                    .for_each(|component| *component = -*component);
            }
            let mut value = [0.0_f32; 4];
            for index in 0..4 {
                value[index] = left[index] + (right[index] - left[index]) * alpha;
            }
            let norm = value
                .iter()
                .map(|component| component * component)
                .sum::<f32>()
                .sqrt();
            if norm <= 1.0e-8 {
                return Err("sequence cross-fade produced an invalid quaternion".into());
            }
            Ok(value
                .into_iter()
                .map(|component| canonical_f32(component / norm))
                .collect())
        }
        _ => Err("sequence transition values have an invalid component count".into()),
    }
}
