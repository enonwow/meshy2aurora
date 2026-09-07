use m2a_core::{
    mdl::NWN_EE_MAX_MESH_INDEX_COUNT_V1,
    model_ir::{
        AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1,
        AuroraVertexWeightsV1,
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

fn skin_model_with_vertex_identity(triangle_count: usize) -> AuroraModelIrV1 {
    let vertex_count = triangle_count * 3;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut weights = Vec::with_capacity(vertex_count);
    for vertex in 0..vertex_count {
        let triangle = vertex / 3;
        let corner = vertex % 3;
        let position = match corner {
            0 => [triangle as f32, 0.0, 0.0],
            1 => [triangle as f32, 1.0, 0.0],
            _ => [triangle as f32, 0.0, 1.0],
        };
        let tag = ((vertex % 1000) + 1) as f32 / 1001.0;
        positions.push(position);
        weights.push(AuroraVertexWeightsV1 {
            bone_node_ids: [Some(1), Some(2), None, None],
            values: [tag, 1.0 - tag, 0.0, 0.0],
            influence_count: 2,
        });
    }
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "segmentation-skin-identity-test".to_owned(),
        source_sha256: "1".repeat(64),
        basis_status: "PASS".to_owned(),
        engine_facing_proof: "PASS".to_owned(),
        uv_runtime_proof: "PASS".to_owned(),
        nodes: vec![
            AuroraModelNodeV1 {
                id: 1,
                name: "root".to_owned(),
                parent_id: None,
                bind_local_matrix: [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ],
            },
            AuroraModelNodeV1 {
                id: 2,
                name: "bone".to_owned(),
                parent_id: Some(1),
                bind_local_matrix: [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ],
            },
        ],
        material_source_bindings: vec![],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 9,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Skin,
            parent_node_id: 1,
            cast_shadow: true,
            positions,
            normals: vec![[1.0, 0.0, 0.0]; vertex_count],
            tangents: None,
            uv0: vec![[0.0, 0.0]; vertex_count],
            indices: (0..vertex_count as u32).collect(),
            face_surface_ids: vec![],
            weights,
        }],
    }
}

fn interleaved_two_component_model(triangles_per_component: usize) -> AuroraModelIrV1 {
    let mut model = rigid_model(0);
    let segment = &mut model.segments[0];
    segment.positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [10.0, 0.0, 0.0],
        [11.0, 0.0, 0.0],
        [10.0, 1.0, 0.0],
    ];
    segment.normals = vec![[0.0, 0.0, 1.0]; 6];
    segment.tangents = Some(vec![[1.0, 0.0, 0.0, 1.0]; 6]);
    segment.uv0 = vec![[0.0, 0.0]; 6];
    segment.indices = (0..triangles_per_component)
        .flat_map(|_| [0_u32, 1, 2, 3, 4, 5])
        .collect();
    segment.face_surface_ids = (0..triangles_per_component)
        .flat_map(|_| [11_i32, 22_i32])
        .collect();
    model
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
fn oversized_partition_keeps_interleaved_connected_components_whole() {
    let triangles_per_component = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 6 + 1;
    let mut model = interleaved_two_component_model(triangles_per_component);

    let report = segment_model_for_binary_mdl_v1(&mut model).expect("partition components");

    assert_eq!(report.output_segment_count, 2);
    assert_eq!(model.segments.len(), 2);
    for segment in &model.segments {
        assert_eq!(segment.indices.len() / 3, triangles_per_component);
        assert_eq!(segment.face_surface_ids.len(), triangles_per_component);
        let surface_ids = segment
            .face_surface_ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(surface_ids.len(), 1);
        assert!(
            surface_ids == [11].into_iter().collect() || surface_ids == [22].into_iter().collect()
        );
    }
}

#[test]
fn skin_partition_preserves_the_vertex_to_weight_identity_across_the_stream_boundary() {
    let triangle_count = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 3 + 1;
    let mut model = skin_model_with_vertex_identity(triangle_count);

    segment_model_for_binary_mdl_v1(&mut model).expect("partition skinned model");

    assert_eq!(model.segments.len(), 2);
    for segment in &model.segments {
        assert_eq!(segment.positions.len(), segment.weights.len());
        for (position, row) in segment.positions.iter().zip(&segment.weights) {
            let triangle = position[0] as usize;
            let corner = if position[1] == 1.0 {
                1
            } else if position[2] == 1.0 {
                2
            } else {
                0
            };
            let source_vertex = triangle * 3 + corner;
            let expected = ((source_vertex % 1000) + 1) as f32 / 1001.0;
            assert_eq!(row.bone_node_ids, [Some(1), Some(2), None, None]);
            assert_eq!(row.values, [expected, 1.0 - expected, 0.0, 0.0]);
        }
    }
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
