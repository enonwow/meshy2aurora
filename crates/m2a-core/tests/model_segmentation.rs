use m2a_core::{
    mdl::NWN_EE_MAX_MESH_INDEX_COUNT_V1,
    model_ir::{
        AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1,
    },
    model_limits::{
        AURORA_MODEL_TRIANGLE_BUDGET_V1, AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
        validate_model_triangle_budget_v1,
    },
    model_segmentation::segment_model_for_binary_mdl_v1,
};

fn rigid_model(triangle_count: usize) -> AuroraModelIrV1 {
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "segmentation-test".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "PASS".to_owned(),
        engine_facing_proof: "PASS".to_owned(),
        uv_runtime_proof: "PASS".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 1,
            name: "root".to_owned(),
            parent_id: None,
            bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }],
        material_source_bindings: vec![],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 7,
            material_slot: 3,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 1,
            cast_shadow: false,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: Some(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: [0_u32, 1, 2].repeat(triangle_count),
            face_surface_ids: (0..triangle_count)
                .map(|triangle| (triangle % 8) as i32)
                .collect(),
            weights: vec![],
        }],
    }
}

#[test]
fn shared_product_budget_is_three_hundred_thousand_triangles() {
    assert_eq!(AURORA_MODEL_TRIANGLE_BUDGET_V1, 300_000);
    assert_eq!(AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1, 150_000);
}

#[test]
fn safe_segment_is_preserved_byte_for_byte_at_the_writer_boundary() {
    let triangle_count = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 3;
    let mut model = rigid_model(triangle_count);
    let expected = model.clone();

    let report = segment_model_for_binary_mdl_v1(&mut model).expect("safe segment");

    assert_eq!(model, expected);
    assert_eq!(report.source_segment_count, 1);
    assert_eq!(report.output_segment_count, 1);
    assert_eq!(report.triangle_count, triangle_count);
    assert_eq!(report.split_segment_count, 0);
}

#[test]
fn oversized_segment_is_partitioned_without_losing_geometry_or_metadata() {
    let triangle_count = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 3 + 1;
    let mut model = rigid_model(triangle_count);
    let expected_surface_ids = model.segments[0].face_surface_ids.clone();

    let report = segment_model_for_binary_mdl_v1(&mut model).expect("partition model");

    assert_eq!(report.source_segment_count, 1);
    assert_eq!(report.output_segment_count, 2);
    assert_eq!(report.triangle_count, triangle_count);
    assert_eq!(report.split_segment_count, 1);
    assert!(model.segments.iter().all(|segment| {
        segment.indices.len() <= NWN_EE_MAX_MESH_INDEX_COUNT_V1
            && segment.positions.len() <= usize::from(u16::MAX)
            && segment.material_slot == 3
            && segment.parent_node_id == 1
            && !segment.cast_shadow
            && segment.deformation == AuroraSegmentDeformationV1::Rigid
    }));
    assert_eq!(
        model
            .segments
            .iter()
            .flat_map(|segment| segment.face_surface_ids.iter().copied())
            .collect::<Vec<_>>(),
        expected_surface_ids
    );
    assert_eq!(
        model
            .segments
            .iter()
            .map(|segment| segment.indices.len() / 3)
            .sum::<usize>(),
        triangle_count
    );
}

#[test]
fn full_product_budget_partitions_into_writer_safe_streams() {
    let mut model = rigid_model(AURORA_MODEL_TRIANGLE_BUDGET_V1);

    let report = segment_model_for_binary_mdl_v1(&mut model).expect("partition 300K model");

    assert_eq!(report.triangle_count, 300_000);
    assert!(report.output_segment_count > 1);
    assert!(
        model
            .segments
            .iter()
            .all(|segment| segment.indices.len() <= NWN_EE_MAX_MESH_INDEX_COUNT_V1)
    );
}

#[test]
fn shared_budget_accepts_exact_limit_and_rejects_one_more_triangle() {
    assert_eq!(
        validate_model_triangle_budget_v1(&rigid_model(300_000)).expect("exact budget"),
        300_000
    );

    let error = validate_model_triangle_budget_v1(&rigid_model(300_001)).expect_err("over budget");
    assert_eq!(error.code, "M2A-MODEL-TRIANGLE-BUDGET-EXCEEDED");
    assert_eq!(error.triangle_count, 300_001);
    assert_eq!(error.triangle_budget, 300_000);
}
