use m2a_core::{
    animation_curves::{
        AnimationCurveKeyV1, AnimationCurveResamplePolicyV1, AnimationEditorCurveTrackV1,
        resample_animation_curve_to_linear_v1,
    },
    animation_studio::AuthoredAnimationTrackPathV1,
};

#[test]
fn cubic_translation_is_adaptively_resampled_to_linear_keys() {
    let curve = AnimationEditorCurveTrackV1 {
        schema_version: 1,
        id: "hand-translation".into(),
        target_node_id: 7,
        path: AuthoredAnimationTrackPathV1::Translation,
        keys: vec![
            AnimationCurveKeyV1 {
                id: "start".into(),
                time_seconds: 0.0,
                value: vec![0.0, 0.0, 0.0],
                in_tangent: Some(vec![0.0, 0.0, 0.0]),
                out_tangent: Some(vec![4.0, 0.0, 0.0]),
            },
            AnimationCurveKeyV1 {
                id: "end".into(),
                time_seconds: 1.0,
                value: vec![1.0, 0.0, 0.0],
                in_tangent: Some(vec![-3.0, 0.0, 0.0]),
                out_tangent: Some(vec![0.0, 0.0, 0.0]),
            },
        ],
    };
    let report = resample_animation_curve_to_linear_v1(
        &curve,
        &AnimationCurveResamplePolicyV1 {
            schema_version: 1,
            max_position_error: 0.01,
            max_angular_error_radians: 0.01,
            max_subdivision_depth: 10,
        },
    )
    .unwrap();

    assert!(report.linear_key_count > 2);
    assert_eq!(
        report.track.interpolation,
        m2a_core::mdl::MdlAnimationInterpolationV1::Linear
    );
    assert_eq!(report.track.keyframes.first().unwrap().time_seconds, 0.0);
    assert_eq!(report.track.keyframes.last().unwrap().time_seconds, 1.0);
    assert_eq!(report.fingerprint_sha256.len(), 64);
}

#[test]
fn rotation_curve_rejects_unsafe_component_tangents() {
    let curve = AnimationEditorCurveTrackV1 {
        schema_version: 1,
        id: "hand-rotation".into(),
        target_node_id: 7,
        path: AuthoredAnimationTrackPathV1::Rotation,
        keys: vec![
            AnimationCurveKeyV1 {
                id: "a".into(),
                time_seconds: 0.0,
                value: vec![0.0, 0.0, 0.0, 1.0],
                in_tangent: None,
                out_tangent: Some(vec![0.0; 4]),
            },
            AnimationCurveKeyV1 {
                id: "b".into(),
                time_seconds: 1.0,
                value: vec![0.0, 0.0, 0.0, -1.0],
                in_tangent: None,
                out_tangent: None,
            },
        ],
    };
    assert!(
        resample_animation_curve_to_linear_v1(
            &curve,
            &AnimationCurveResamplePolicyV1 {
                schema_version: 1,
                max_position_error: 0.01,
                max_angular_error_radians: 0.01,
                max_subdivision_depth: 8,
            }
        )
        .is_err()
    );
}
