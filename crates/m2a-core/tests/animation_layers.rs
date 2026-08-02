use m2a_core::{
    animation_layers::{
        AnimationBoneMaskV1, AnimationEditLayerV1, AnimationLayerModeV1, bake_animation_layers_v1,
    },
    animation_studio::{
        AnimationKeyframeV1, AnimationStudioRigNodeV1, AnimationStudioRigV1,
        AuthoredAnimationClipKindV1, AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1,
        AuthoredAnimationSourceKindV1, AuthoredAnimationSourceV1, AuthoredAnimationTrackPathV1,
        AuthoredAnimationTrackV1,
    },
    mdl::MdlAnimationInterpolationV1,
};

fn source_revision() -> String {
    "a".repeat(64)
}

fn rig() -> AnimationStudioRigV1 {
    AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: source_revision(),
        animation_root: "Root".into(),
        nodes: vec![AnimationStudioRigNodeV1 {
            node_id: 1,
            name: "Root".into(),
            parent_id: None,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
        }],
    }
}

fn clip(id: &str, values: [[f32; 3]; 2]) -> AuthoredAnimationClipV1 {
    AuthoredAnimationClipV1 {
        id: id.into(),
        name: format!("{id}_motion"),
        kind: AuthoredAnimationClipKindV1::Motion,
        status: AuthoredAnimationClipStatusV1::Valid,
        source: AuthoredAnimationSourceV1 {
            kind: AuthoredAnimationSourceKindV1::BlankPose,
            source_revision: source_revision(),
            source_clip_name: None,
            source_clip_fingerprint: None,
            procedural_template: None,
            library_preset: None,
            retarget: None,
        },
        length_seconds: 1.0,
        transition_seconds: 0.1,
        animation_root: "Root".into(),
        tracks: vec![AuthoredAnimationTrackV1 {
            id: format!("{id}-root"),
            target_node_id: 1,
            path: AuthoredAnimationTrackPathV1::Translation,
            interpolation: MdlAnimationInterpolationV1::Linear,
            keyframes: values
                .into_iter()
                .enumerate()
                .map(|(index, value)| AnimationKeyframeV1 {
                    id: format!("{id}-{index}"),
                    time_seconds: index as f32,
                    value: value.to_vec(),
                })
                .collect(),
        }],
        events: vec![],
        revision: 1,
    }
}

fn layer(
    id: &str,
    order: u32,
    mode: AnimationLayerModeV1,
    weight: f32,
    clip: AuthoredAnimationClipV1,
) -> AnimationEditLayerV1 {
    AnimationEditLayerV1 {
        schema_version: 1,
        id: id.into(),
        stable_order: order,
        mode,
        weight,
        mute: false,
        solo: false,
        bone_mask: AnimationBoneMaskV1 {
            schema_version: 1,
            node_ids: vec![1],
        },
        clip,
    }
}

#[test]
fn muted_layer_returns_the_exact_base_clip() {
    let base_clip = clip("base", [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
    let base = layer(
        "base-layer",
        0,
        AnimationLayerModeV1::Base,
        1.0,
        base_clip.clone(),
    );
    let mut correction = layer(
        "correction",
        1,
        AnimationLayerModeV1::Additive,
        1.0,
        clip("delta", [[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]]),
    );
    correction.mute = true;
    let baked = bake_animation_layers_v1(&[base, correction], &rig()).unwrap();
    assert_eq!(baked.clip, base_clip);
    assert_eq!(baked.active_layer_ids, vec!["base-layer"]);
}

#[test]
fn additive_layer_is_weighted_masked_and_deterministic() {
    let layers = vec![
        layer(
            "base-layer",
            0,
            AnimationLayerModeV1::Base,
            1.0,
            clip("base", [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
        ),
        layer(
            "correction",
            1,
            AnimationLayerModeV1::Additive,
            0.5,
            clip("delta", [[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]]),
        ),
    ];
    let first = bake_animation_layers_v1(&layers, &rig()).unwrap();
    let second = bake_animation_layers_v1(&layers, &rig()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.clip.tracks[0].keyframes[0].value, vec![0.0, 0.5, 0.0]);
    assert_eq!(first.clip.tracks[0].keyframes[1].value, vec![1.0, 0.5, 0.0]);
}
