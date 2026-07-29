use m2a_core::{
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
        MdlAnimationTrackPathV1, MdlAnimationTrackV1,
    },
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
        AuroraSegmentDeformationV1, AuroraVertexWeightsV1,
    },
    skin_accessory::{
        SkinAccessoryComponentActionV1, SkinAccessoryStabilizationModeV1,
        SkinAccessoryStabilizationOptionsV1, audit_and_stabilize_skin_accessories_v1,
    },
};

fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

fn hard_weight(bone: u32) -> AuroraVertexWeightsV1 {
    AuroraVertexWeightsV1 {
        bone_node_ids: [Some(bone), None, None, None],
        values: [1.0, 0.0, 0.0, 0.0],
        influence_count: 1,
    }
}

fn mixed_weight(first: u32, first_value: f32, second: u32) -> AuroraVertexWeightsV1 {
    AuroraVertexWeightsV1 {
        bone_node_ids: [Some(first), Some(second), None, None],
        values: [first_value, 1.0 - first_value, 0.0, 0.0],
        influence_count: 2,
    }
}

fn push_triangle(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uv0: &mut Vec<[f32; 2]>,
    weights: &mut Vec<AuroraVertexWeightsV1>,
    indices: &mut Vec<u32>,
    vertices: [[f32; 3]; 3],
    vertex_weights: [AuroraVertexWeightsV1; 3],
) {
    let start = u32::try_from(positions.len()).unwrap();
    positions.extend(vertices);
    normals.extend([[0.0, 1.0, 0.0]; 3]);
    uv0.extend([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
    weights.extend(vertex_weights);
    indices.extend([start, start + 1, start + 2]);
}

fn synthetic_creature() -> AuroraModelIrV1 {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uv0 = Vec::new();
    let mut weights = Vec::new();
    let mut indices = Vec::new();

    // Four body triangles deliberately duplicate every shared seam vertex.
    // Raw-index connectivity sees four islands; spatial welding must see one.
    for vertices in [
        [[-0.3, 0.0, 0.8], [0.3, 0.0, 0.8], [-0.3, 0.0, 1.4]],
        [[0.3, 0.0, 0.8], [0.3, 0.0, 1.4], [-0.3, 0.0, 1.4]],
        [[-0.3, 0.0, 1.4], [0.3, 0.0, 1.4], [-0.2, 0.0, 1.8]],
        [[0.3, 0.0, 1.4], [0.2, 0.0, 1.8], [-0.2, 0.0, 1.8]],
    ] {
        push_triangle(
            &mut positions,
            &mut normals,
            &mut uv0,
            &mut weights,
            &mut indices,
            vertices,
            [hard_weight(1), hard_weight(1), hard_weight(1)],
        );
    }

    // Lower detached accessory near Spine02. Different arm/hip blends across
    // the vertices make it stretch when the arm rotates.
    for vertices in [
        [[-0.9, 0.0, 1.05], [-0.6, 0.0, 1.05], [-0.75, 0.0, 1.35]],
        [[-0.6, 0.0, 1.05], [-0.6, 0.0, 1.35], [-0.75, 0.0, 1.35]],
    ] {
        push_triangle(
            &mut positions,
            &mut normals,
            &mut uv0,
            &mut weights,
            &mut indices,
            vertices,
            [
                mixed_weight(4, 0.85, 1),
                mixed_weight(4, 0.25, 1),
                mixed_weight(4, 0.55, 1),
            ],
        );
    }

    // Upper detached accessory is already a correct rigid attachment.
    for vertices in [
        [[0.6, 0.0, 1.65], [0.9, 0.0, 1.65], [0.75, 0.0, 1.95]],
        [[0.6, 0.0, 1.65], [0.75, 0.0, 1.95], [0.6, 0.0, 1.95]],
    ] {
        push_triangle(
            &mut positions,
            &mut normals,
            &mut uv0,
            &mut weights,
            &mut indices,
            vertices,
            [hard_weight(2), hard_weight(2), hard_weight(2)],
        );
    }

    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "synthetic-accessory".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "LOCKED".to_owned(),
        engine_facing_proof: "SYNTHETIC".to_owned(),
        uv_runtime_proof: "SYNTHETIC".to_owned(),
        nodes: vec![
            AuroraModelNodeV1 {
                id: 0,
                name: "Root".to_owned(),
                parent_id: None,
                bind_local_matrix: translation(0.0, 0.0, 0.0),
            },
            AuroraModelNodeV1 {
                id: 1,
                name: "Hips".to_owned(),
                parent_id: Some(0),
                bind_local_matrix: translation(0.0, 0.0, 0.8),
            },
            AuroraModelNodeV1 {
                id: 3,
                name: "Spine02".to_owned(),
                parent_id: Some(1),
                bind_local_matrix: translation(0.0, 0.0, 0.35),
            },
            AuroraModelNodeV1 {
                id: 2,
                name: "Spine".to_owned(),
                parent_id: Some(3),
                bind_local_matrix: translation(0.0, 0.0, 0.65),
            },
            AuroraModelNodeV1 {
                id: 4,
                name: "LeftArm".to_owned(),
                parent_id: Some(2),
                bind_local_matrix: translation(-0.45, 0.0, 0.0),
            },
            AuroraModelNodeV1 {
                id: 5,
                name: "RightArm".to_owned(),
                parent_id: Some(2),
                bind_local_matrix: translation(0.45, 0.0, 0.0),
            },
        ],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: Some(0),
            source_material_name: Some("synthetic".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 1,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            positions,
            normals,
            tangents: None,
            uv0,
            indices,
            face_surface_ids: Vec::new(),
            weights,
        }],
    }
}

fn animation_set() -> MdlAnimationSetV1 {
    let half_turn_z = std::f32::consts::FRAC_1_SQRT_2;
    MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "cpause1".to_owned(),
            animation_root: "Root".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.0,
            events: Vec::new(),
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 4,
                path: MdlAnimationTrackPathV1::Rotation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![
                    vec![0.0, 0.0, 0.0, 1.0],
                    vec![0.0, 0.0, half_turn_z, half_turn_z],
                ],
            }],
        }],
    }
}

fn options(mode: SkinAccessoryStabilizationModeV1) -> SkinAccessoryStabilizationOptionsV1 {
    SkinAccessoryStabilizationOptionsV1 {
        schema_version: 1,
        mode,
        selected_bone_name: None,
    }
}

#[test]
fn spatial_weld_connects_seams_and_primary_body_is_never_stabilized() {
    let mut creature = synthetic_creature();
    let original_body_weights = creature.segments[0].weights[..12].to_vec();
    let report = audit_and_stabilize_skin_accessories_v1(
        &mut creature,
        &animation_set(),
        &options(SkinAccessoryStabilizationModeV1::Auto),
    )
    .expect("accessory audit");

    assert_eq!(report.component_count, 3);
    let primary = report
        .components
        .iter()
        .find(|component| component.is_primary_body)
        .expect("primary body");
    assert_eq!(primary.triangle_count, 4);
    assert_eq!(primary.action, SkinAccessoryComponentActionV1::PrimaryBody);
    assert_eq!(&creature.segments[0].weights[..12], &original_body_weights);
}

#[test]
fn auto_stabilizes_mixed_detached_accessory_and_improves_deformation_metrics() {
    let mut creature = synthetic_creature();
    let report = audit_and_stabilize_skin_accessories_v1(
        &mut creature,
        &animation_set(),
        &options(SkinAccessoryStabilizationModeV1::Auto),
    )
    .expect("accessory stabilization");

    let repaired = report
        .components
        .iter()
        .find(|component| component.action == SkinAccessoryComponentActionV1::Stabilized)
        .expect("mixed lower accessory must be repaired");
    assert_eq!(repaired.selected_bone_name.as_deref(), Some("Spine02"));
    assert_eq!(repaired.changed_vertex_count, 6);
    assert!(repaired.before.max_pair_distance_ratio > 1.1);
    assert!(repaired.after.max_pair_distance_ratio <= 1.000_01);
    assert!(repaired.after.max_pair_distance_error <= 0.000_01);
    for vertex in &creature.segments[0].weights[12..18] {
        assert_eq!(vertex, &hard_weight(3));
    }
}

#[test]
fn correct_rigid_accessory_is_reported_without_weight_changes() {
    let mut creature = synthetic_creature();
    let original = creature.clone();
    let report = audit_and_stabilize_skin_accessories_v1(
        &mut creature,
        &animation_set(),
        &options(SkinAccessoryStabilizationModeV1::Auto),
    )
    .expect("accessory audit");

    let stable = report
        .components
        .iter()
        .find(|component| component.action == SkinAccessoryComponentActionV1::StableAccessory)
        .expect("rigid accessory");
    assert_eq!(stable.changed_vertex_count, 0);
    assert_eq!(
        &creature.segments[0].weights[18..],
        &original.segments[0].weights[18..]
    );
}

#[test]
fn keep_source_weights_is_auditable_and_does_not_mutate_the_model() {
    let mut creature = synthetic_creature();
    let original = creature.clone();
    let report = audit_and_stabilize_skin_accessories_v1(
        &mut creature,
        &animation_set(),
        &options(SkinAccessoryStabilizationModeV1::KeepSourceWeights),
    )
    .expect("keep-source audit");

    assert_eq!(creature, original);
    assert_eq!(report.changed_vertex_count, 0);
    assert!(report.components.iter().any(|component| {
        component.action == SkinAccessoryComponentActionV1::KeptSourceWeights
            && component.before.max_pair_distance_ratio > 1.1
    }));
}

#[test]
fn explicit_bone_selection_is_applied_only_to_approved_accessories() {
    let mut creature = synthetic_creature();
    let mut selected = options(SkinAccessoryStabilizationModeV1::SelectBone);
    selected.selected_bone_name = Some("Spine".to_owned());
    let report =
        audit_and_stabilize_skin_accessories_v1(&mut creature, &animation_set(), &selected)
            .expect("selected-bone stabilization");

    let repaired = report
        .components
        .iter()
        .find(|component| component.action == SkinAccessoryComponentActionV1::Stabilized)
        .expect("mixed accessory");
    assert_eq!(repaired.selected_bone_name.as_deref(), Some("Spine"));
    for vertex in &creature.segments[0].weights[12..18] {
        assert_eq!(vertex, &hard_weight(2));
    }
}

#[test]
fn stabilization_is_deterministic_and_preserves_non_weight_payloads() {
    let animations = animation_set();
    let original_animations = animations.clone();
    let mut first = synthetic_creature();
    let mut second = first.clone();
    let original = first.clone();
    let first_report = audit_and_stabilize_skin_accessories_v1(
        &mut first,
        &animations,
        &options(SkinAccessoryStabilizationModeV1::Auto),
    )
    .expect("first");
    let second_report = audit_and_stabilize_skin_accessories_v1(
        &mut second,
        &animations,
        &options(SkinAccessoryStabilizationModeV1::Auto),
    )
    .expect("second");

    assert_eq!(first_report, second_report);
    assert_eq!(first, second);
    assert_eq!(animations, original_animations);
    assert_eq!(first.nodes, original.nodes);
    assert_eq!(
        first.material_source_bindings,
        original.material_source_bindings
    );
    assert_eq!(first.segments[0].positions, original.segments[0].positions);
    assert_eq!(first.segments[0].normals, original.segments[0].normals);
    assert_eq!(first.segments[0].tangents, original.segments[0].tangents);
    assert_eq!(first.segments[0].uv0, original.segments[0].uv0);
    assert_eq!(first.segments[0].indices, original.segments[0].indices);
    assert_eq!(
        first.segments[0].material_slot,
        original.segments[0].material_slot
    );
}
