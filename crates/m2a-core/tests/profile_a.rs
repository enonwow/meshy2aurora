#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod fixtures;

use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::{
    glb::{GlbLimits, IrNode, IrTransform, ingest_glb},
    mdl::{
        MdlAnimationTrackPathV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlWriterOptionsV1, write_binary_mdl_with_animations,
    },
    profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        ProfileAAnimationClipMappingV1, ProfileAAnimationMappingV1, ProfileAAnimationNodeMappingV1,
        ProfileAOptionsV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_creature_sha256,
        canonical_profile_sha256, convert_profile_a, convert_profile_a_with_animations_v1,
    },
};

fn identity() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn profile(height: f32) -> CreatureRigProfileV1 {
    let mut profile = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: "synthetic-rigid-axis-profile".to_owned(),
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
            min: [-10.0, -10.0, 0.0],
            max: [10.0, 10.0, height],
        },
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes: vec![CreatureRigNodeV1 {
            id: 70,
            name: "synthetic-rigid-root".to_owned(),
            parent_id: None,
            bind_local_matrix: identity(),
        }],
        segments: vec![CreatureRigSegmentV1 {
            id: 9,
            name: "synthetic-rigid-segment".to_owned(),
            deformation: RigSegmentDeformationV1::Rigid,
            parent_node_id: 70,
            surface_positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: vec![],
            reference_weights: vec![],
        }],
    };
    profile.content_sha256 = canonical_profile_sha256(&profile).unwrap();
    profile
}

fn minimal_source() -> m2a_core::glb::GlbIngestResult {
    ingest_glb(&fixtures::minimal_indexed_triangle(), &GlbLimits::default()).unwrap()
}

fn rehash(mut rig: CreatureRigProfileV1) -> CreatureRigProfileV1 {
    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    rig
}

fn linear_animated_source() -> m2a_core::glb::GlbIngestResult {
    let input = fixtures::mutate_json(
        fixtures::skin_animation_with_inverse_bind_matrices(),
        |root| {
            root["animations"][0]["samplers"][1]["interpolation"] = serde_json::json!("LINEAR");
            root["animations"][0]["samplers"]
                .as_array_mut()
                .expect("synthetic animation samplers")
                .truncate(2);
            root["animations"][0]["channels"]
                .as_array_mut()
                .expect("synthetic animation channels")
                .truncate(2);
        },
    );
    ingest_glb(&input, &GlbLimits::default()).expect("synthetic LINEAR animation source")
}

fn animated_profile() -> CreatureRigProfileV1 {
    let mut rig = profile(2.0);
    let mut child_bind = identity();
    child_bind[0] = 0.0;
    child_bind[1] = 1.0;
    child_bind[4] = -1.0;
    child_bind[5] = 0.0;
    child_bind[12] = 10.0;
    child_bind[13] = 20.0;
    child_bind[14] = 30.0;
    rig.nodes.push(CreatureRigNodeV1 {
        id: 71,
        name: "synthetic-animated-child".to_owned(),
        parent_id: Some(70),
        bind_local_matrix: child_bind,
    });
    rehash(rig)
}

fn animation_mapping() -> ProfileAAnimationMappingV1 {
    ProfileAAnimationMappingV1 {
        schema_version: 1,
        source_skin_id: 0,
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Synthetic,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        node_mappings: vec![
            ProfileAAnimationNodeMappingV1 {
                source_node_id: 0,
                output_rig_node_id: 70,
            },
            ProfileAAnimationNodeMappingV1 {
                source_node_id: 1,
                output_rig_node_id: 71,
            },
        ],
        clip_mappings: vec![ProfileAAnimationClipMappingV1 {
            source_animation_id: 0,
            output_clip_name: "cpause1".to_owned(),
            transition_seconds: 0.25,
        }],
    }
}

fn animation_writer_options() -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
        model_resource_resref: "m2a_anim".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_tex".to_owned(),
        }],
    }
}

fn assert_animation_fatal(
    source: &m2a_core::glb::GlbIngestResult,
    rig: &CreatureRigProfileV1,
    mapping: &ProfileAAnimationMappingV1,
    expected_code: &str,
) {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        convert_profile_a_with_animations_v1(source, rig, &ProfileAOptionsV1::default(), mapping)
    }));
    let error = outcome
        .expect("M4A2 invalid input must not panic")
        .expect_err("M4A2 invalid input must be fatal");
    assert_eq!(
        error.code, expected_code,
        "unexpected M4A2 error: {error:?}"
    );
}

fn skin_profile(reference_weights: Vec<Vec<RigWeightInfluenceV1>>) -> CreatureRigProfileV1 {
    let mut rig = profile(1.0);
    rig.profile_id = "synthetic-skin-transfer-profile".to_owned();
    rig.nodes.extend((1..=6).map(|id| CreatureRigNodeV1 {
        id,
        name: format!("synthetic-bone-{id}"),
        parent_id: Some(70),
        bind_local_matrix: identity(),
    }));
    rig.segments = vec![CreatureRigSegmentV1 {
        id: 12,
        name: "synthetic-skin-surface".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 70,
        surface_positions: vec![
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        surface_indices: vec![0, 1, 2, 1, 3, 2],
        allowed_bone_node_ids: (1..=6).collect(),
        reference_weights,
    }];
    rehash(rig)
}

fn square_source() -> m2a_core::glb::GlbIngestResult {
    let mut source = minimal_source();
    let primitive = &mut source.ir.primitives[0];
    primitive.positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    primitive.normals = vec![[0.0, 0.0, 1.0]; 4];
    primitive.uv0 = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    primitive.indices = vec![0, 1, 2, 1, 3, 2];
    primitive.bounds_min = [0.0, 0.0, 0.0];
    primitive.bounds_max = [1.0, 1.0, 0.0];
    source.report.statistics.vertex_count = 4;
    source.report.statistics.index_count = 6;
    source.report.statistics.triangle_count = 2;
    source.report.statistics.bounds_min = Some([0.0, 0.0, 0.0]);
    source.report.statistics.bounds_max = Some([1.0, 1.0, 0.0]);
    source
}

fn approx(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1.0e-5, "{actual} != {expected}");
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[test]
fn basis_scale_alignment_winding_normal_tangent_and_uv_are_exact() {
    let mut source = minimal_source();
    source.ir.primitives[0].tangents = vec![[1.0, 0.0, 0.0, 1.0]; 3];
    let source_before = serde_json::to_vec(&source).unwrap();

    let outcome = convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    assert!(outcome.report.conversion_eligible);
    let creature = outcome.creature.unwrap();
    assert_eq!(creature.nodes.len(), 1);
    assert_eq!(
        creature.nodes[0].id, 70,
        "output hierarchy must be the rig hierarchy"
    );
    let segment = &creature.segments[0];
    assert_eq!(segment.indices, [0, 2, 1]);
    assert!(
        segment.weights.is_empty(),
        "RIGID attachment is encoded by parentNodeId only"
    );
    assert_eq!(
        segment.positions,
        [[-0.5, 0.0, 0.0], [0.5, 0.0, 0.0], [-0.5, 0.0, 1.0]]
    );
    assert_eq!(segment.normals, [[0.0, 1.0, 0.0]; 3]);
    assert_eq!(
        segment.tangents.as_ref().unwrap(),
        &vec![[1.0, 0.0, 0.0, -1.0]; 3]
    );
    assert_eq!(segment.uv0, [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]]);

    let a = segment.positions[segment.indices[0] as usize];
    let b = segment.positions[segment.indices[1] as usize];
    let c = segment.positions[segment.indices[2] as usize];
    assert!(dot(cross(sub(b, a), sub(c, a)), segment.normals[0]) > 0.0);
    for (source_uv, target_uv) in source.ir.primitives[0].uv0.iter().zip(&segment.uv0) {
        approx(1.0 - target_uv[1], source_uv[1]);
    }
    assert_eq!(
        source_before,
        serde_json::to_vec(&source).unwrap(),
        "source IR and report are immutable"
    );
}

#[test]
fn target_height_controls_uniform_scale_and_zero_height_is_blocked() {
    let source = minimal_source();
    let doubled = convert_profile_a(&source, &profile(2.0), &ProfileAOptionsV1::default()).unwrap();
    approx(doubled.report.transform.scale.unwrap(), 2.0);
    approx(doubled.report.transform.target_bounds.unwrap().max[2], 2.0);

    let mut flat = source;
    flat.ir.primitives[0].positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
    flat.ir.primitives[0].normals = vec![[0.0, -1.0, 0.0]; 3];
    flat.ir.primitives[0].bounds_min = [0.0, 0.0, 0.0];
    flat.ir.primitives[0].bounds_max = [1.0, 0.0, 1.0];
    flat.report.statistics.bounds_min = Some([0.0, 0.0, 0.0]);
    flat.report.statistics.bounds_max = Some([1.0, 0.0, 1.0]);
    let blocked = convert_profile_a(&flat, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    assert!(blocked.creature.is_none());
    assert!(
        blocked
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-ZERO-HEIGHT")
    );
}

#[test]
fn default_scene_only_bakes_nested_matrix_and_trs_once() {
    let mut source = minimal_source();
    source.ir.nodes = vec![
        IrNode {
            id: 0,
            name: Some("matrix-root".to_owned()),
            child_ids: vec![1],
            parent_ids: vec![],
            transform: IrTransform {
                kind: "MATRIX".to_owned(),
                matrix: Some([
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 30.0,
                    1.0,
                ]),
                translation: None,
                rotation: None,
                scale: None,
            },
            mesh_id: None,
            skin_id: None,
        },
        IrNode {
            id: 1,
            name: Some("trs-mesh".to_owned()),
            child_ids: vec![],
            parent_ids: vec![0],
            transform: IrTransform {
                kind: "TRS".to_owned(),
                matrix: None,
                translation: Some([1.0, 2.0, 3.0]),
                rotation: Some([0.0, 0.0, 0.0, 1.0]),
                scale: Some([1.0, 2.0, 1.0]),
            },
            mesh_id: Some(0),
            skin_id: None,
        },
        IrNode {
            id: 2,
            name: Some("unreachable-mesh".to_owned()),
            child_ids: vec![],
            parent_ids: vec![],
            transform: IrTransform {
                kind: "TRS".to_owned(),
                matrix: None,
                translation: Some([999.0, 999.0, 999.0]),
                rotation: Some([0.0, 0.0, 0.0, 1.0]),
                scale: Some([1.0, 1.0, 1.0]),
            },
            mesh_id: Some(0),
            skin_id: None,
        },
    ];
    source.report.inventory.node_count = 3;
    source.ir.scenes[0].root_node_ids = vec![0];
    let outcome = convert_profile_a(&source, &profile(2.0), &ProfileAOptionsV1::default()).unwrap();
    assert_eq!(outcome.report.source_selection.reachable_node_count, 2);
    assert_eq!(
        outcome
            .report
            .source_selection
            .reachable_mesh_instance_count,
        1
    );
    assert_eq!(outcome.report.source_selection.ignored_node_count, 1);
    assert_eq!(
        outcome.creature.unwrap().nodes.len(),
        1,
        "source nodes must never leak into target rig"
    );
    let bounds = outcome.report.transform.source_world_bounds.unwrap();
    assert_eq!(bounds.min, [11.0, 22.0, 33.0]);
    assert_eq!(bounds.max, [12.0, 24.0, 33.0]);
}

#[test]
fn shared_mesh_is_duplicated_per_default_scene_instance() {
    let mut source = minimal_source();
    let mut second = source.ir.nodes[0].clone();
    second.id = 1;
    second.name = Some("second-instance".to_owned());
    second.transform.translation = Some([2.0, 0.0, 0.0]);
    source.ir.nodes.push(second);
    source.report.inventory.node_count = 2;
    source.ir.scenes[0].root_node_ids.push(1);
    let outcome = convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    assert_eq!(
        outcome
            .report
            .source_selection
            .duplicated_mesh_instance_count,
        1
    );
    assert_eq!(outcome.report.geometry.output_vertex_count, 6);
    assert_eq!(outcome.creature.unwrap().segments[0].indices.len(), 6);
}

#[test]
fn material_and_source_gates_block_without_silent_output() {
    let two_materials = ingest_glb(
        &fixtures::material_image_two_primitives(),
        &GlbLimits::default(),
    )
    .unwrap();
    let blocked =
        convert_profile_a(&two_materials, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    assert!(blocked.creature.is_none());
    assert!(
        blocked
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-MATERIAL-LIMIT")
    );
    assert_eq!(blocked.report.materials.unique_used_count, 2);
    assert_eq!(
        blocked
            .report
            .materials
            .bindings
            .iter()
            .map(|binding| binding.source_material_id)
            .collect::<Vec<_>>(),
        vec![Some(0), Some(1)]
    );

    let missing_normals = ingest_glb(&fixtures::missing_normals(), &GlbLimits::default()).unwrap();
    let blocked = convert_profile_a(
        &missing_normals,
        &profile(1.0),
        &ProfileAOptionsV1::default(),
    )
    .unwrap();
    assert!(blocked.creature.is_none());
    assert!(
        blocked
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-NORMALS-REQUIRED")
    );
    assert_eq!(blocked.report.materials.unique_used_count, 1);
    assert_eq!(
        blocked.report.materials.bindings[0].source_material_id,
        Some(0)
    );
}

#[test]
fn source_rig_and_animation_are_explicit_blocking_gates() {
    let source = ingest_glb(
        &fixtures::skin_animation_with_inverse_bind_matrices(),
        &GlbLimits::default(),
    )
    .unwrap();
    let outcome = convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    assert!(outcome.creature.is_none());
    assert!(
        outcome
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-SOURCE-RIG-DEFERRED")
    );
    assert!(
        outcome
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-SOURCE-ANIMATION-DEFERRED")
    );
}

#[test]
fn default_m3_rejects_animated_source_unchanged() {
    let source = linear_animated_source();
    let source_before = serde_json::to_vec(&source).unwrap();
    let outcome = convert_profile_a(&source, &animated_profile(), &ProfileAOptionsV1::default())
        .expect("default M3 returns its established blocking outcome");

    assert!(outcome.creature.is_none());
    assert_eq!(
        outcome
            .report
            .gates
            .iter()
            .filter(|gate| {
                matches!(
                    gate.code.as_str(),
                    "M3A-SOURCE-RIG-DEFERRED" | "M3A-SOURCE-ANIMATION-DEFERRED"
                )
            })
            .map(|gate| (
                gate.code.as_str(),
                gate.path.as_str(),
                gate.severity.as_str()
            ))
            .collect::<Vec<_>>(),
        [
            (
                "M3A-SOURCE-ANIMATION-DEFERRED",
                "source.ir.animations",
                "BLOCKING"
            ),
            ("M3A-SOURCE-RIG-DEFERRED", "source.ir.skins", "BLOCKING"),
        ]
    );
    assert_eq!(source_before, serde_json::to_vec(&source).unwrap());
}

#[test]
fn mapped_linear_animation_retargets_rest_delta_and_hands_off_to_writer() {
    let source = linear_animated_source();
    let rig = animated_profile();
    let mapping = animation_mapping();
    let source_before = serde_json::to_vec(&source).unwrap();
    let rig_before = serde_json::to_vec(&rig).unwrap();
    let mapping_before = serde_json::to_vec(&mapping).unwrap();

    let first = convert_profile_a_with_animations_v1(
        &source,
        &rig,
        &ProfileAOptionsV1::default(),
        &mapping,
    )
    .expect("fully mapped synthetic LINEAR animation");
    let second = convert_profile_a_with_animations_v1(
        &source,
        &rig,
        &ProfileAOptionsV1::default(),
        &mapping,
    )
    .expect("deterministic mapped animation");

    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert_eq!(source_before, serde_json::to_vec(&source).unwrap());
    assert_eq!(rig_before, serde_json::to_vec(&rig).unwrap());
    assert_eq!(mapping_before, serde_json::to_vec(&mapping).unwrap());
    assert_eq!(first.base.report.transform.scale, Some(2.0));
    assert_eq!(
        first.base.report.transform.translation,
        Some([-1.0, 0.0, 0.0])
    );

    let creature = first.base.creature.as_ref().expect("mapped base creature");
    let animations = first.animations.as_ref().expect("mapped animation set");
    assert_eq!(animations.clips.len(), 1);
    let clip = &animations.clips[0];
    assert_eq!(clip.name, "cpause1");
    assert_eq!(clip.animation_root, "synthetic-rigid-root");
    assert_eq!(clip.length_seconds.to_bits(), 1.25_f32.to_bits());
    assert_eq!(clip.transition_seconds.to_bits(), 0.25_f32.to_bits());
    assert!(clip.events.is_empty());
    assert_eq!(clip.tracks.len(), 2);

    let translation = clip
        .tracks
        .iter()
        .find(|track| track.path == MdlAnimationTrackPathV1::Translation)
        .expect("mapped translation track");
    let rotation = clip
        .tracks
        .iter()
        .find(|track| track.path == MdlAnimationTrackPathV1::Rotation)
        .expect("mapped rotation track");
    for track in [translation, rotation] {
        assert_eq!(track.target_node_id, 71);
        assert_eq!(
            track
                .times_seconds
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            [0.0_f32.to_bits(), 0.5_f32.to_bits(), 1.25_f32.to_bits()]
        );
    }

    assert_eq!(
        translation.values,
        [
            vec![10.0, 20.0, 28.0],
            vec![4.0, 22.0, 32.0],
            vec![-2.0, 28.0, 38.0],
        ]
    );
    assert_eq!(rotation.values.len(), 3);
    let half = std::f32::consts::FRAC_1_SQRT_2;
    for (actual, expected) in rotation.values[0].iter().zip([0.0, 0.0, half, half]) {
        approx(*actual, expected);
    }
    for (actual, expected) in rotation.values[1].iter().zip([0.0, 0.0, 0.0, 1.0]) {
        approx(*actual, expected);
    }
    approx(rotation.values[2][0], 0.0);
    approx(rotation.values[2][1], 0.0);
    approx(rotation.values[2][2], -half);
    approx(rotation.values[2][3], half);

    let artifact =
        write_binary_mdl_with_animations(creature, animations, &animation_writer_options())
            .expect("mapped animation writer handoff");
    assert!(artifact.report.semantic_diff.is_empty());
    assert_eq!(artifact.inspection.animations.len(), 1);
}

#[test]
fn mapped_animation_requires_every_channel_target_mapping() {
    let source = linear_animated_source();
    let rig = animated_profile();
    let mut mapping = animation_mapping();
    mapping
        .node_mappings
        .retain(|entry| entry.source_node_id != 1);

    assert_animation_fatal(&source, &rig, &mapping, "M4A-MAPPER-TARGET-MISSING");
}

#[test]
fn mapped_animation_schema_provenance_and_skin_fail_with_stable_codes() {
    let source = linear_animated_source();
    let rig = animated_profile();

    let mut mapping = animation_mapping();
    mapping.schema_version = 2;
    assert_animation_fatal(&source, &rig, &mapping, "M4A-MAPPING-SCHEMA-INVALID");

    let mut mapping = animation_mapping();
    mapping.provenance.kind = RigProvenanceKindV1::ReferenceOnly;
    assert_animation_fatal(&source, &rig, &mapping, "M4A-MAPPER-PROVENANCE-FORBIDDEN");

    let mut mapping = animation_mapping();
    mapping.provenance.export_allowed = false;
    assert_animation_fatal(&source, &rig, &mapping, "M4A-MAPPER-PROVENANCE-FORBIDDEN");

    let mut mapping = animation_mapping();
    mapping.provenance.attestations.rights_confirmed = false;
    assert_animation_fatal(&source, &rig, &mapping, "M4A-MAPPER-PROVENANCE-FORBIDDEN");

    let mut mapping = animation_mapping();
    mapping.source_skin_id = 99;
    assert_animation_fatal(&source, &rig, &mapping, "M4A-MAPPER-SKIN-INVALID");
}

#[test]
fn mapped_animation_rejects_duplicate_missing_and_nonhierarchical_mappings() {
    let source = linear_animated_source();
    let rig = animated_profile();

    let mut duplicate_node = animation_mapping();
    duplicate_node
        .node_mappings
        .push(duplicate_node.node_mappings[1].clone());
    assert_animation_fatal(
        &source,
        &rig,
        &duplicate_node,
        "M4A-MAPPER-TARGET-AMBIGUOUS",
    );

    let mut missing_clip = animation_mapping();
    missing_clip.clip_mappings.clear();
    assert_animation_fatal(&source, &rig, &missing_clip, "M4A-MAPPER-TARGET-MISSING");

    let mut duplicate_clip = animation_mapping();
    duplicate_clip
        .clip_mappings
        .push(duplicate_clip.clip_mappings[0].clone());
    assert_animation_fatal(
        &source,
        &rig,
        &duplicate_clip,
        "M4A-MAPPER-TARGET-AMBIGUOUS",
    );

    let mut nonhierarchical = animation_mapping();
    nonhierarchical.node_mappings[0].output_rig_node_id = 71;
    nonhierarchical.node_mappings[1].output_rig_node_id = 70;
    assert_animation_fatal(&source, &rig, &nonhierarchical, "M4A-MAPPER-BASIS-INVALID");
}

#[test]
fn mapped_animation_rejects_unsupported_interpolation_and_target_paths() {
    let rig = animated_profile();
    let mapping = animation_mapping();

    let mut step = linear_animated_source();
    step.ir.animations[0].samplers[0].interpolation = "STEP".to_owned();
    assert_animation_fatal(&step, &rig, &mapping, "M4A-INTERPOLATION-UNSUPPORTED");

    for path in ["SCALE", "WEIGHTS"] {
        let mut source = linear_animated_source();
        source.ir.animations[0].channels[0].target_path = path.to_owned();
        assert_animation_fatal(&source, &rig, &mapping, "M4A-TRACK-PATH-UNSUPPORTED");
    }
}

#[test]
fn mapped_animation_rejects_invalid_times_values_and_arity_without_panicking() {
    let rig = animated_profile();
    let mapping = animation_mapping();

    let mut nonfinite_time = linear_animated_source();
    nonfinite_time.ir.animations[0].samplers[0].input_times_seconds[1] = f32::NAN;
    assert_animation_fatal(&nonfinite_time, &rig, &mapping, "M4A-TRACK-TIME-NOT-STRICT");

    let mut nonfinite_value = linear_animated_source();
    nonfinite_value.ir.animations[0].samplers[0].output_values[0] = f32::INFINITY;
    assert_animation_fatal(
        &nonfinite_value,
        &rig,
        &mapping,
        "M4A-TRACK-VALUE-NONFINITE",
    );

    let mut arity = linear_animated_source();
    arity.ir.animations[0].samplers[0].output_values.pop();
    assert_animation_fatal(&arity, &rig, &mapping, "M4A-TRACK-ARITY-INVALID");
}

#[test]
fn mapped_animation_finite_translation_subtraction_overflow_is_fatal() {
    let mut source = linear_animated_source();
    source.ir.nodes[1].transform.translation = Some([-f32::MAX, 1.0, 0.0]);
    source.ir.animations[0].samplers[0].output_values[0] = f32::MAX;

    assert_animation_fatal(
        &source,
        &animated_profile(),
        &animation_mapping(),
        "M4A-TRACK-VALUE-NONFINITE",
    );
}

#[test]
fn mapped_animation_rejects_nonunit_source_scale_and_nonrigid_target_bind() {
    let mapping = animation_mapping();

    let mut scaled_source = linear_animated_source();
    scaled_source.ir.nodes[1].transform.scale = Some([1.0, 2.0, 1.0]);
    assert_animation_fatal(
        &scaled_source,
        &animated_profile(),
        &mapping,
        "M4A-MAPPER-BASIS-INVALID",
    );

    let source = linear_animated_source();
    let mut nonrigid = animated_profile();
    nonrigid.nodes[1].bind_local_matrix[0] = 2.0;
    let nonrigid = rehash(nonrigid);
    assert_animation_fatal(&source, &nonrigid, &mapping, "M4A-MAPPER-BASIS-INVALID");
}

#[test]
fn provenance_is_a_gate_but_profile_hash_mismatch_is_fatal() {
    let source = minimal_source();
    let mut forbidden = profile(1.0);
    forbidden.provenance.export_allowed = false;
    forbidden.content_sha256 = canonical_profile_sha256(&forbidden).unwrap();
    let blocked = convert_profile_a(&source, &forbidden, &ProfileAOptionsV1::default()).unwrap();
    assert!(blocked.creature.is_none());
    assert!(
        blocked
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-PROFILE-PROVENANCE-FORBIDDEN")
    );

    let mut changed = profile(1.0);
    changed.profile_id.push_str("-changed");
    assert_eq!(
        convert_profile_a(&source, &changed, &ProfileAOptionsV1::default())
            .unwrap_err()
            .code,
        "M3A-PROFILE-HASH-MISMATCH"
    );
}

#[test]
fn canonical_hash_normalizes_negative_zero_and_conversion_is_deterministic() {
    let source = minimal_source();
    let positive = profile(1.0);
    let mut negative = positive.clone();
    negative.alignment_anchor[0] = -0.0;
    assert_eq!(
        canonical_profile_sha256(&positive).unwrap(),
        canonical_profile_sha256(&negative).unwrap()
    );

    let first = convert_profile_a(&source, &positive, &ProfileAOptionsV1::default()).unwrap();
    let second = convert_profile_a(&source, &positive, &ProfileAOptionsV1::default()).unwrap();
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert_eq!(first.report.creature_sha256.as_deref().unwrap().len(), 64);
    let creature = first.creature.unwrap();
    assert_eq!(
        canonical_creature_sha256(&creature).unwrap(),
        first.report.creature_sha256.unwrap()
    );
    let mut negative_creature = creature.clone();
    negative_creature.segments[0].positions[0][1] = -0.0;
    assert_eq!(
        canonical_creature_sha256(&creature).unwrap(),
        canonical_creature_sha256(&negative_creature).unwrap()
    );
}

#[test]
fn typed_invalid_inputs_do_not_panic() {
    let source = minimal_source();
    let mut invalid = profile(1.0);
    invalid.nodes[0].bind_local_matrix = [0.0; 16];
    invalid.content_sha256 = canonical_profile_sha256(&invalid).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        convert_profile_a(&source, &invalid, &ProfileAOptionsV1::default())
    }));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().unwrap_err().code,
        "M3A-PROFILE-HIERARCHY-INVALID"
    );
}

#[test]
fn source_contract_mismatches_are_exact_fatal_and_never_panic() {
    let mut cases = Vec::new();
    let mut wrong_format = minimal_source();
    wrong_format.ir.source.format = "GLTF_2_0".to_owned();
    cases.push(wrong_format);
    let mut wrong_policy = minimal_source();
    wrong_policy.ir.coordinate_space.target_transform_status = "ALREADY_CHANGED".to_owned();
    cases.push(wrong_policy);
    let mut nonfinite_uv = minimal_source();
    nonfinite_uv.ir.primitives[0].uv0[0][0] = f32::NAN;
    cases.push(nonfinite_uv);
    let mut bad_index = minimal_source();
    bad_index.ir.primitives[0].indices[0] = 999;
    cases.push(bad_index);
    let mut bad_mesh_owner = minimal_source();
    bad_mesh_owner.ir.primitives[0].source_mesh_id = 88;
    cases.push(bad_mesh_owner);
    let mut reverse_parent_mismatch = minimal_source();
    reverse_parent_mismatch.ir.nodes[0].parent_ids = vec![0];
    cases.push(reverse_parent_mismatch);

    for source in cases {
        let result = catch_unwind(AssertUnwindSafe(|| {
            convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default())
        }));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().unwrap_err().code,
            "M3A-SOURCE-CONTRACT-MISMATCH"
        );
    }
}

#[test]
fn legal_blocked_m2_geometry_is_correlated_both_directions() {
    let cases = [
        (
            fixtures::non_triangle_lines(),
            "M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED",
        ),
        (
            fixtures::incomplete_triangle(),
            "M2A-GLB-INCOMPLETE-TRIANGLES",
        ),
        (fixtures::index_out_of_bounds(), "M2A-GLB-INDEX-OOB"),
        (
            fixtures::mismatched_attributes(),
            "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
        ),
        (
            fixtures::degenerate_triangle(),
            "M2A-GLB-DEGENERATE-TRIANGLES",
        ),
    ];
    for (bytes, code) in cases {
        let source = ingest_glb(&bytes, &GlbLimits::default()).unwrap();
        let outcome =
            convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
        assert!(outcome.creature.is_none());
        assert!(
            outcome
                .report
                .gates
                .iter()
                .any(|gate| gate.code == "M3A-SOURCE-BLOCKED")
        );

        let mut missing_gate = source.clone();
        missing_gate.report.gates.retain(|gate| gate.code != code);
        missing_gate.report.conversion_eligible = !missing_gate
            .report
            .gates
            .iter()
            .any(|gate| gate.severity == "BLOCKING");
        assert_eq!(
            convert_profile_a(&missing_gate, &profile(1.0), &ProfileAOptionsV1::default())
                .unwrap_err()
                .code,
            "M3A-SOURCE-CONTRACT-MISMATCH"
        );

        let mut spurious = minimal_source();
        let mut gate = source
            .report
            .gates
            .iter()
            .find(|gate| gate.code == code)
            .unwrap()
            .clone();
        gate.path = "meshes[0].primitives[0]".to_owned();
        spurious.report.gates.push(gate);
        spurious.report.conversion_eligible = false;
        assert_eq!(
            convert_profile_a(&spurious, &profile(1.0), &ProfileAOptionsV1::default())
                .unwrap_err()
                .code,
            "M3A-SOURCE-CONTRACT-MISMATCH"
        );
    }
}

#[test]
fn missing_empty_or_ambiguous_default_scene_is_a_blocking_gate() {
    let mut cases = Vec::new();
    let mut missing = minimal_source();
    missing.ir.default_scene_id = None;
    cases.push(missing);
    let mut no_mesh = minimal_source();
    no_mesh.ir.nodes[0].mesh_id = None;
    cases.push(no_mesh);
    let mut duplicate_root = minimal_source();
    duplicate_root.ir.scenes[0].root_node_ids = vec![0, 0];
    cases.push(duplicate_root);

    for source in cases {
        let outcome =
            convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
        assert!(outcome.creature.is_none());
        assert!(
            outcome
                .report
                .gates
                .iter()
                .any(|gate| gate.code == "M3A-DEFAULT-SCENE-REQUIRED"
                    || gate.code == "M3A-DEFAULT-SCENE-HIERARCHY-INVALID")
        );
    }
}

#[test]
fn material_null_binding_and_path_name_omission_are_deterministic() {
    let mut path_name = minimal_source();
    path_name.ir.materials[0].name = Some(r"C:\private\source.png".to_owned());
    let outcome =
        convert_profile_a(&path_name, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    let creature = outcome.creature.unwrap();
    assert_eq!(
        creature.material_source_bindings[0].source_material_name,
        None
    );
    assert!(
        outcome
            .report
            .diagnostics
            .iter()
            .any(|item| item.code == "M3A-SOURCE-MATERIAL-NAME-OMITTED")
    );
    let json = serde_json::to_string(&(creature, outcome.report)).unwrap();
    assert!(!json.contains("private"));

    let mut blocked_path_name = path_name;
    blocked_path_name.ir.primitives[0].normals.clear();
    let blocked = convert_profile_a(
        &blocked_path_name,
        &profile(1.0),
        &ProfileAOptionsV1::default(),
    )
    .unwrap();
    assert!(blocked.creature.is_none());
    assert_eq!(blocked.report.materials.unique_used_count, 1);
    assert_eq!(
        blocked.report.materials.bindings[0].source_material_name,
        None
    );
    assert!(
        blocked
            .report
            .diagnostics
            .iter()
            .any(|item| item.code == "M3A-SOURCE-MATERIAL-NAME-OMITTED")
    );

    let mut no_material = minimal_source();
    no_material.ir.primitives[0].material_id = None;
    let creature = convert_profile_a(&no_material, &profile(1.0), &ProfileAOptionsV1::default())
        .unwrap()
        .creature
        .unwrap();
    assert_eq!(creature.material_source_bindings[0].slot, 0);
    assert_eq!(
        creature.material_source_bindings[0].source_material_id,
        None
    );
}

#[test]
fn options_and_preallocation_limits_fail_before_output_allocation() {
    let source = minimal_source();
    let wrong_schema = ProfileAOptionsV1 {
        schema_version: 2,
        ..ProfileAOptionsV1::default()
    };
    let error = convert_profile_a(&source, &profile(1.0), &wrong_schema).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        ("M3A-OPTIONS-INVALID", "options.schemaVersion")
    );

    let mut wrong_threshold = ProfileAOptionsV1::default();
    wrong_threshold.limits.triangle_warning_above = 4_999;
    assert_eq!(
        convert_profile_a(&source, &profile(1.0), &wrong_threshold)
            .unwrap_err()
            .code,
        "M3A-OPTIONS-INVALID"
    );

    let baseline =
        convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    let exact_peak = baseline.report.work.work_bytes_peak;
    let mut exact = ProfileAOptionsV1::default();
    exact.limits.max_output_vertices = 3;
    exact.limits.max_output_indices = 3;
    exact.limits.max_work_bytes = exact_peak;
    convert_profile_a(&source, &profile(1.0), &exact).expect("exact output/work boundaries");
    let mut below = exact.clone();
    below.limits.max_output_vertices = 2;
    assert_eq!(
        convert_profile_a(&source, &profile(1.0), &below)
            .unwrap_err()
            .code,
        "M3A-LIMIT-EXCEEDED"
    );

    let mut path_source = minimal_source();
    path_source.ir.materials[0].name = Some("data:image/png;base64,AAAA".to_owned());
    let mut one_diagnostic = ProfileAOptionsV1::default();
    one_diagnostic.limits.max_diagnostics = 1;
    let exact_diagnostic = convert_profile_a(&path_source, &profile(1.0), &one_diagnostic).unwrap();
    assert_eq!(exact_diagnostic.report.diagnostics.len(), 1);
    let mut warning_source =
        ingest_glb(&fixtures::triangle_budget(5_001), &GlbLimits::default()).unwrap();
    warning_source.ir.materials[0].name = Some("data:image/png;base64,AAAA".to_owned());
    assert_eq!(
        convert_profile_a(&warning_source, &profile(1.0), &one_diagnostic)
            .unwrap_err()
            .code,
        "M3A-LIMIT-EXCEEDED"
    );
    let mut below = exact;
    below.limits.max_work_bytes = exact_peak - 1;
    assert_eq!(
        convert_profile_a(&source, &profile(1.0), &below)
            .unwrap_err()
            .code,
        "M3A-LIMIT-EXCEEDED"
    );
}

#[test]
fn instanced_multi_primitive_budget_is_checked_before_materialization() {
    let mut source = ingest_glb(
        &fixtures::material_image_two_primitives(),
        &GlbLimits::default(),
    )
    .unwrap();
    source.ir.primitives[1].material_id = Some(0);
    let mut second = source.ir.nodes[0].clone();
    second.id = 1;
    second.name = Some("second-budget-instance".to_owned());
    source.ir.nodes.push(second);
    source.ir.scenes[0].root_node_ids.push(1);
    source.report.inventory.node_count = 2;
    let mut exact = ProfileAOptionsV1::default();
    exact.limits.max_output_vertices = 12;
    exact.limits.max_output_indices = 12;
    let outcome = convert_profile_a(&source, &profile(1.0), &exact).unwrap();
    assert_eq!(outcome.report.geometry.output_vertex_count, 12);
    let mut below = exact;
    below.limits.max_output_vertices = 11;
    assert_eq!(
        convert_profile_a(&source, &profile(1.0), &below)
            .unwrap_err()
            .code,
        "M3A-LIMIT-EXCEEDED"
    );
}

#[test]
fn deep_default_scene_hierarchy_is_iterative_and_no_panic() {
    let mut source = minimal_source();
    let template = source.ir.nodes[0].clone();
    let depth = 2_048usize;
    source.ir.nodes = (0..depth)
        .map(|index| {
            let mut node = template.clone();
            node.id = u32::try_from(index).unwrap();
            node.name = Some(format!("deep-{index}"));
            node.parent_ids = (index > 0)
                .then(|| u32::try_from(index - 1).unwrap())
                .into_iter()
                .collect();
            node.child_ids = (index + 1 < depth)
                .then(|| u32::try_from(index + 1).unwrap())
                .into_iter()
                .collect();
            node.mesh_id = (index + 1 == depth).then_some(0);
            node
        })
        .collect();
    source.ir.scenes[0].root_node_ids = vec![0];
    source.report.inventory.node_count = depth;
    let result = catch_unwind(AssertUnwindSafe(|| {
        convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default())
    }));
    assert!(result.is_ok());
    let outcome = result.unwrap().unwrap();
    assert_eq!(outcome.report.source_selection.reachable_node_count, 2_048);
    assert!(outcome.creature.is_some());
}

#[test]
fn exact_max_depth_rig_hierarchy_is_iterative_and_no_panic() {
    let source = minimal_source();
    let mut rig = profile(1.0);
    rig.nodes = (0..4_096u32)
        .rev()
        .map(|id| CreatureRigNodeV1 {
            id,
            name: format!("rig-{id}"),
            parent_id: (id > 0).then(|| id - 1),
            bind_local_matrix: identity(),
        })
        .collect();
    rig.segments[0].parent_node_id = 4_095;
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        convert_profile_a(&source, &rig, &ProfileAOptionsV1::default())
    }));
    assert!(result.is_ok());
    let outcome = result.unwrap().unwrap();
    assert_eq!(outcome.creature.unwrap().nodes.len(), 4_096);
}

#[test]
fn multi_segment_exact_tie_uses_lowest_id_and_distance_limit_is_exact() {
    let source = minimal_source();
    let source_before = serde_json::to_vec(&source).unwrap();
    let mut rig = profile(1.0);
    let mut high = rig.segments[0].clone();
    high.id = 20;
    high.name = "synthetic-high-id".to_owned();
    let mut low = high.clone();
    low.id = 10;
    low.name = "synthetic-low-id".to_owned();
    rig.segments = vec![high, low];
    rig = rehash(rig);

    let outcome = convert_profile_a(&source, &rig, &ProfileAOptionsV1::default()).unwrap();
    assert_eq!(outcome.report.work.distance_evaluations, 6);
    let creature = outcome.creature.unwrap();
    assert_eq!(creature.segments.len(), 1);
    assert_eq!(creature.segments[0].segment_id, 10);
    assert_eq!(serde_json::to_vec(&source).unwrap(), source_before);

    let mut exact = ProfileAOptionsV1::default();
    exact.limits.max_distance_evaluations = 6;
    convert_profile_a(&source, &rig, &exact).expect("exact distance boundary");
    exact.limits.max_distance_evaluations = 5;
    let error = convert_profile_a(&source, &rig, &exact).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        ("M3A-LIMIT-EXCEEDED", "distanceEvaluations")
    );
}

#[test]
fn skin_barycentric_interpolation_is_exhaustive_and_normalized() {
    let weights = vec![
        vec![RigWeightInfluenceV1 {
            bone_node_id: 1,
            value: 1.0,
        }],
        vec![RigWeightInfluenceV1 {
            bone_node_id: 2,
            value: 1.0,
        }],
        vec![RigWeightInfluenceV1 {
            bone_node_id: 3,
            value: 1.0,
        }],
        vec![RigWeightInfluenceV1 {
            bone_node_id: 4,
            value: 1.0,
        }],
    ];
    let outcome = convert_profile_a(
        &minimal_source(),
        &skin_profile(weights),
        &ProfileAOptionsV1::default(),
    )
    .unwrap();
    assert_eq!(outcome.report.work.distance_evaluations, 12);
    assert_eq!(outcome.report.weights.skinned_vertex_count, 3);
    assert_eq!(outcome.report.weights.normalized_vertex_count, 3);
    let segment = &outcome.creature.unwrap().segments[0];
    assert_eq!(segment.deformation, RigSegmentDeformationV1::Skin);
    assert_eq!(segment.weights.len(), 3);
    assert_eq!(
        segment.weights[0].bone_node_ids,
        [Some(1), Some(2), None, None]
    );
    approx(segment.weights[0].values[0], 0.75);
    approx(segment.weights[0].values[1], 0.25);
    assert_eq!(
        segment.weights[1].bone_node_ids,
        [Some(2), Some(1), None, None]
    );
    approx(segment.weights[1].values[0], 0.75);
    approx(segment.weights[1].values[1], 0.25);
    assert_eq!(
        segment.weights[2].bone_node_ids,
        [Some(3), Some(4), None, None]
    );
    approx(segment.weights[2].values[0], 0.75);
    approx(segment.weights[2].values[1], 0.25);
}

#[test]
fn skin_duplicate_merge_top_four_and_large_finite_values_are_stable() {
    let lanes = vec![
        RigWeightInfluenceV1 {
            bone_node_id: 1,
            value: f32::MAX,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 1,
            value: f32::MAX,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 2,
            value: f32::MAX,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 3,
            value: 4.0,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 4,
            value: 3.0,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 5,
            value: 2.0,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 6,
            value: 1.0,
        },
        RigWeightInfluenceV1 {
            bone_node_id: 6,
            value: 0.0,
        },
    ];
    let outcome = convert_profile_a(
        &minimal_source(),
        &skin_profile(vec![lanes.clone(), lanes.clone(), lanes.clone(), lanes]),
        &ProfileAOptionsV1::default(),
    )
    .unwrap();
    assert!(outcome.report.conversion_eligible);
    assert!(outcome.report.weights.merged_duplicate_influence_count > 0);
    assert!(outcome.report.weights.dropped_after_top_four_count > 0);
    assert_eq!(outcome.report.weights.max_influences_before, 6);
    assert_eq!(outcome.report.weights.max_influences_after, 4);
    for weights in &outcome.creature.unwrap().segments[0].weights {
        assert_eq!(weights.influence_count, 4);
        assert_eq!(weights.bone_node_ids, [Some(1), Some(2), Some(3), Some(4)]);
        approx(weights.values.iter().sum(), 1.0);
        assert!(weights.values.iter().all(|value| value.is_finite()));
    }
}

#[test]
fn forbidden_profile_bone_is_fatal_and_zero_sum_is_a_deterministic_gate() {
    let forbidden = vec![
        vec![RigWeightInfluenceV1 {
            bone_node_id: 999,
            value: 1.0
        }];
        4
    ];
    let error = convert_profile_a(
        &minimal_source(),
        &skin_profile(forbidden),
        &ProfileAOptionsV1::default(),
    )
    .unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        (
            "M3A-PROFILE-SEGMENT-INVALID",
            "rig.segments.referenceWeights"
        )
    );

    let zero = vec![
        vec![RigWeightInfluenceV1 {
            bone_node_id: 1,
            value: 0.0
        }];
        4
    ];
    let mut one_diagnostic = ProfileAOptionsV1::default();
    one_diagnostic.limits.max_diagnostics = 1;
    let blocked =
        convert_profile_a(&minimal_source(), &skin_profile(zero), &one_diagnostic).unwrap();
    assert!(blocked.creature.is_none());
    assert!(
        blocked
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-WEIGHT-SUM-INVALID")
    );
}

#[test]
fn mixed_rigid_skin_buckets_duplicate_boundary_vertices_only() {
    let source = square_source();
    let source_before = serde_json::to_vec(&source).unwrap();
    let mut rig = profile(1.0);
    rig.nodes.push(CreatureRigNodeV1 {
        id: 1,
        name: "synthetic-skin-bone".to_owned(),
        parent_id: Some(70),
        bind_local_matrix: identity(),
    });
    rig.segments[0].id = 5;
    rig.segments[0].name = "synthetic-rigid-lower".to_owned();
    rig.segments[0].surface_positions = vec![[-0.5, 0.0, 0.0], [0.5, 0.0, 0.0], [-0.5, 0.0, 1.0]];
    rig.segments.push(CreatureRigSegmentV1 {
        id: 10,
        name: "synthetic-skin-upper".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: 70,
        surface_positions: vec![[0.5, 0.0, 0.0], [0.5, 0.0, 1.0], [-0.5, 0.0, 1.0]],
        surface_indices: vec![0, 1, 2],
        allowed_bone_node_ids: vec![1],
        reference_weights: vec![
            vec![RigWeightInfluenceV1 {
                bone_node_id: 1,
                value: 1.0
            }];
            3
        ],
    });
    rig = rehash(rig);
    let outcome = convert_profile_a(&source, &rig, &ProfileAOptionsV1::default()).unwrap();
    assert_eq!(outcome.report.work.distance_evaluations, 15);
    assert_eq!(outcome.report.geometry.source_vertex_instance_count, 4);
    assert_eq!(outcome.report.geometry.output_vertex_count, 6);
    assert_eq!(outcome.report.geometry.duplicated_vertex_count, 2);
    assert_eq!(outcome.report.weights.rigid_vertex_count, 3);
    assert_eq!(outcome.report.weights.skinned_vertex_count, 3);
    let creature = outcome.creature.unwrap();
    assert_eq!(
        creature
            .segments
            .iter()
            .map(|segment| segment.segment_id)
            .collect::<Vec<_>>(),
        [5, 10]
    );
    assert!(creature.segments[0].weights.is_empty());
    assert_eq!(creature.segments[1].weights.len(), 3);
    assert_eq!(serde_json::to_vec(&source).unwrap(), source_before);
}

#[test]
fn unreferenced_vertex_is_explicitly_blocked_without_panic_or_silent_drop() {
    let mut source = minimal_source();
    source.ir.primitives[0].positions.push([0.25, 0.25, 0.25]);
    source.ir.primitives[0].normals.push([0.0, 0.0, 1.0]);
    source.ir.primitives[0].uv0.push([0.25, 0.25]);
    source.ir.primitives[0].bounds_max = [1.0, 1.0, 0.25];
    source.report.statistics.vertex_count = 4;
    source.report.statistics.bounds_max = Some([1.0, 1.0, 0.25]);
    let result = catch_unwind(AssertUnwindSafe(|| {
        convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default())
    }));
    let blocked = result.expect("must not panic").unwrap();
    assert!(blocked.creature.is_none());
    assert!(
        blocked
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-SEGMENT-ASSIGNMENT-FAILED")
    );
}

#[test]
fn target_world_surface_rounding_degeneracy_is_stable_fatal() {
    let mut rig = profile(1.0);
    rig.nodes[0].bind_local_matrix[12] = 1.0e20;
    rig.target_bounds.min[0] = -1.0e21;
    rig.target_bounds.max[0] = 1.0e21;
    rig = rehash(rig);
    let error =
        convert_profile_a(&minimal_source(), &rig, &ProfileAOptionsV1::default()).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.path.as_str()),
        (
            "M3A-PROFILE-SEGMENT-INVALID",
            "rig.segments.surfacePositions"
        )
    );
}

#[test]
fn mixed_tangent_coverage_is_blocking_not_fatal_or_silent() {
    let mut source = ingest_glb(
        &fixtures::material_image_two_primitives(),
        &GlbLimits::default(),
    )
    .unwrap();
    source.ir.primitives[1].material_id = Some(0);
    source.ir.primitives[0].tangents = vec![[1.0, 0.0, 0.0, 1.0]; 3];
    let outcome = convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default()).unwrap();
    assert!(outcome.creature.is_none());
    assert!(
        outcome
            .report
            .gates
            .iter()
            .any(|gate| gate.code == "M3A-TANGENT-COVERAGE-MIXED")
    );
}

#[test]
fn reflected_source_uses_full_composite_parity_without_double_flip() {
    let mut source = minimal_source();
    source.ir.nodes[0].transform.scale = Some([-1.0, 1.0, 1.0]);
    source.ir.primitives[0].tangents = vec![[1.0, 0.0, 0.0, 1.0]; 3];
    let creature = convert_profile_a(&source, &profile(1.0), &ProfileAOptionsV1::default())
        .unwrap()
        .creature
        .unwrap();
    let segment = &creature.segments[0];
    assert_eq!(
        segment.indices,
        [0, 1, 2],
        "source reflection and basis reflection cancel"
    );
    assert_eq!(segment.tangents.as_ref().unwrap()[0][3], 1.0);
    let a = segment.positions[0];
    let b = segment.positions[1];
    let c = segment.positions[2];
    assert!(dot(cross(sub(b, a), sub(c, a)), segment.normals[0]) > 0.0);
}

#[test]
fn public_creature_ir_contains_only_contract_fields() {
    let creature = convert_profile_a(
        &minimal_source(),
        &profile(1.0),
        &ProfileAOptionsV1::default(),
    )
    .unwrap()
    .creature
    .unwrap();
    let object = serde_json::to_value(creature).unwrap();
    assert!(object.get("bounds").is_none());
    assert!(object["segments"][0].get("sourcePrimitiveIds").is_none());
}

#[test]
fn canonical_creature_hash_rejects_every_nonfinite_domain() {
    let creature = convert_profile_a(
        &minimal_source(),
        &profile(1.0),
        &ProfileAOptionsV1::default(),
    )
    .unwrap()
    .creature
    .unwrap();
    let mut cases = Vec::new();
    let mut matrix = creature.clone();
    matrix.nodes[0].bind_local_matrix[0] = f32::INFINITY;
    cases.push(matrix);
    let mut position = creature.clone();
    position.segments[0].positions[0][0] = f32::NAN;
    cases.push(position);
    let mut normal = creature.clone();
    normal.segments[0].normals[0][0] = f32::NEG_INFINITY;
    cases.push(normal);
    let mut uv = creature.clone();
    uv.segments[0].uv0[0][0] = f32::NAN;
    cases.push(uv);
    for invalid in cases {
        assert_eq!(
            canonical_creature_sha256(&invalid).unwrap_err().code,
            "M3A-NONFINITE-FLOAT"
        );
    }
}

#[test]
fn profile_logical_names_reject_paths_schemes_and_overlength_labels() {
    let source = minimal_source();
    let invalid_names = vec![
        "C:profile".to_owned(),
        "data:model".to_owned(),
        "folder/profile".to_owned(),
        "x".repeat(129),
    ];
    for invalid_name in invalid_names {
        let mut rig = profile(1.0);
        rig.profile_id = invalid_name;
        rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
        assert_eq!(
            convert_profile_a(&source, &rig, &ProfileAOptionsV1::default())
                .unwrap_err()
                .code,
            "M3A-PROFILE-JSON-INVALID"
        );
    }
}
