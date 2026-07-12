#[path = "fixtures/build_synthetic_glb.rs"]
mod fixtures;

use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::glb::{GlbLimits, ingest_glb, inspect_glb};

#[test]
fn minimal_indexed_triangle_produces_stable_report_and_ir() {
    let input = fixtures::minimal_indexed_triangle();
    let original = input.clone();
    let limits = GlbLimits::default();

    let report = inspect_glb(&input, &limits).expect("minimal GLB report");
    let result = ingest_glb(&input, &limits).expect("minimal GLB ingest");
    assert_eq!(report.schema_version, 1);
    assert_eq!(result.schema_version, 1);
    assert_eq!(result.ir.schema_version, 1);
    assert_eq!(result.report, report);
    assert!(report.conversion_eligible);
    assert_eq!(report.inventory.scene_count, 1);
    assert_eq!(report.inventory.node_count, 1);
    assert_eq!(report.inventory.primitive_count, 1);
    assert_eq!(report.statistics.vertex_count, 3);
    assert_eq!(report.statistics.index_count, 3);
    assert_eq!(report.statistics.triangle_count, 1);
    assert_eq!(result.ir.primitives[0].indices, [0, 1, 2]);
    assert_eq!(
        result.ir.materials[0].base_color_factor,
        [0.1, 0.2, 0.3, 0.4]
    );
    assert_eq!(input, original, "native APIs must not mutate source bytes");

    let report_json = serde_json::to_vec(&report).unwrap();
    assert_eq!(
        report_json,
        serde_json::to_vec(&inspect_glb(&input, &limits).unwrap()).unwrap()
    );
    let result_json = serde_json::to_vec(&result).unwrap();
    assert_eq!(
        result_json,
        serde_json::to_vec(&ingest_glb(&input, &limits).unwrap()).unwrap()
    );
}

#[test]
fn axis_hierarchy_winding_normals_and_units_are_source_preserved() {
    let result = ingest_glb(
        &fixtures::axis_hierarchy_asymmetric(),
        &GlbLimits::default(),
    )
    .unwrap();
    let primitive = &result.ir.primitives[0];
    assert_eq!(
        primitive.positions,
        [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]
    );
    assert_eq!(
        primitive.normals,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    );
    assert_eq!(primitive.indices, [2, 0, 1]);

    let ir = serde_json::to_value(&result.ir).unwrap();
    assert_eq!(ir["coordinateSpace"]["storedSpace"], "GLTF_SOURCE");
    assert_eq!(ir["coordinateSpace"]["up"], "POSITIVE_Y");
    assert_eq!(ir["coordinateSpace"]["forwardConvention"], "POSITIVE_Z");
    assert_eq!(ir["coordinateSpace"]["handedness"], "RIGHT_HANDED");
    assert_eq!(ir["coordinateSpace"]["units"], "METERS_DECLARED_BY_MESHY");
    assert_eq!(ir["coordinateSpace"]["windingPolicy"], "PRESERVED");
    assert_eq!(
        ir["coordinateSpace"]["targetTransformStatus"],
        "UNRESOLVED_M3"
    );
    assert_eq!(ir["nodes"][0]["childIds"], serde_json::json!([1]));
    assert_eq!(ir["nodes"][1]["parentIds"], serde_json::json!([0]));
    assert_eq!(
        ir["nodes"][1]["transform"]["translation"],
        serde_json::json!([1.0, 2.0, 3.0])
    );
    assert_eq!(
        ir["nodes"][1]["transform"]["scale"],
        serde_json::json!([2.0, 3.0, 4.0])
    );
}

#[test]
fn uv_corners_and_out_of_range_values_are_not_flipped_or_clamped() {
    let result = ingest_glb(
        &fixtures::uv_corners_and_out_of_range(),
        &GlbLimits::default(),
    )
    .unwrap();
    assert_eq!(
        result.ir.primitives[0].uv0,
        [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [-0.25, 1.25],
            [2.0, -1.0]
        ]
    );
}

#[test]
fn missing_channels_and_nontriangle_mode_are_stable_nonfatal_gates() {
    let limits = GlbLimits::default();
    let missing_uv = inspect_glb(&fixtures::missing_uv(), &limits).unwrap();
    assert_gate(&missing_uv, "M2A-GLB-UV0-MISSING", "BLOCKING");
    assert!(!missing_uv.conversion_eligible);

    let missing_normals = inspect_glb(&fixtures::missing_normals(), &limits).unwrap();
    assert_gate(&missing_normals, "M2A-GLB-NORMALS-MISSING", "WARNING");
    assert!(missing_normals.conversion_eligible);

    let lines = inspect_glb(&fixtures::non_triangle_lines(), &limits).unwrap();
    assert_gate(&lines, "M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED", "BLOCKING");
    assert!(!lines.conversion_eligible);
}

#[test]
fn triangle_warning_and_blocking_thresholds_are_exact() {
    let limits = GlbLimits::default();
    let warning = inspect_glb(&fixtures::triangle_budget(5_001), &limits).unwrap();
    assert_eq!(warning.statistics.triangle_count, 5_001);
    assert_gate(&warning, "M2A-GLB-GEOMETRY-WARNING", "WARNING");
    assert!(warning.conversion_eligible);

    let blocking = inspect_glb(&fixtures::triangle_budget(10_001), &limits).unwrap();
    assert_eq!(blocking.statistics.triangle_count, 10_001);
    assert_gate(&blocking, "M2A-GLB-GEOMETRY-OVER-BUDGET", "BLOCKING");
    assert!(!blocking.conversion_eligible);
}

#[test]
fn cumulative_asset_geometry_limits_and_triangle_gates_are_preflighted() {
    let duplicated = fixtures::two_primitive_triangle_budget(1);
    assert_eq!(
        inspect_glb(
            &duplicated,
            &GlbLimits {
                max_vertices: 5,
                ..GlbLimits::default()
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );
    assert_eq!(
        inspect_glb(
            &duplicated,
            &GlbLimits {
                max_indices: 5,
                ..GlbLimits::default()
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );

    let warning = inspect_glb(
        &fixtures::two_primitive_triangle_budget(5_000),
        &GlbLimits::default(),
    )
    .unwrap();
    assert_eq!(warning.statistics.triangle_count, 10_000);
    assert_gate(&warning, "M2A-GLB-GEOMETRY-WARNING", "WARNING");
    assert!(warning.conversion_eligible);

    let blocking = inspect_glb(
        &fixtures::two_primitive_triangle_budget(6_000),
        &GlbLimits::default(),
    )
    .unwrap();
    assert_eq!(blocking.statistics.triangle_count, 12_000);
    assert_gate(&blocking, "M2A-GLB-GEOMETRY-OVER-BUDGET", "BLOCKING");
    assert!(!blocking.conversion_eligible);
}

#[test]
fn decoded_geometry_budget_counts_actual_lanes_and_each_primitive_materialization() {
    let large_attributes = fixtures::tiny_positions_with_large_normals_and_uv(100);
    inspect_glb(
        &large_attributes,
        &GlbLimits {
            max_decoded_geometry_bytes: 2_048,
            ..GlbLimits::default()
        },
    )
    .expect("exact POSITION + NORMAL + UV + index output bytes are accepted");
    assert_eq!(
        inspect_glb(
            &large_attributes,
            &GlbLimits {
                max_decoded_geometry_bytes: 2_047,
                ..GlbLimits::default()
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );

    let reused_accessors = fixtures::two_primitive_triangle_budget(1);
    inspect_glb(
        &reused_accessors,
        &GlbLimits {
            max_decoded_geometry_bytes: 216,
            ..GlbLimits::default()
        },
    )
    .expect("each of two 108-byte primitive materializations fits exact total boundary");
    assert_eq!(
        inspect_glb(
            &reused_accessors,
            &GlbLimits {
                max_decoded_geometry_bytes: 215,
                ..GlbLimits::default()
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );
}

#[test]
fn raw_json_preflight_rejects_unsupported_embedded_only_and_layout_cases() {
    let limits = GlbLimits::default();
    for (input, code) in [
        (
            fixtures::external_image_uri(),
            "M2A-GLB-EXTERNAL-URI-UNSUPPORTED",
        ),
        (
            fixtures::sparse_position_accessor(),
            "M2A-GLB-SPARSE-ACCESSOR-UNSUPPORTED",
        ),
        (
            fixtures::required_extension("UNKNOWN_required"),
            "M2A-GLB-REQUIRED-EXTENSION-UNSUPPORTED",
        ),
        (
            fixtures::required_extension("KHR_draco_mesh_compression"),
            "M2A-GLB-COMPRESSION-UNSUPPORTED",
        ),
        (
            fixtures::required_extension("EXT_meshopt_compression"),
            "M2A-GLB-COMPRESSION-UNSUPPORTED",
        ),
        (
            fixtures::used_extension("EXT_meshopt_compression"),
            "M2A-GLB-COMPRESSION-UNSUPPORTED",
        ),
        (fixtures::buffer_view_oob(), "M2A-GLB-BUFFER-VIEW-OOB"),
        (fixtures::accessor_oob(), "M2A-GLB-ACCESSOR-OOB"),
        (
            fixtures::invalid_accessor_layout(),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        ),
    ] {
        assert_eq!(inspect_glb(&input, &limits).unwrap_err().code, code);
    }
}

#[test]
fn source_asset_metadata_matrix_shape_and_nonfinite_transform_are_exact() {
    let result = ingest_glb(
        &fixtures::source_metadata_and_matrix(),
        &GlbLimits::default(),
    )
    .unwrap();
    assert_eq!(result.ir.source.asset_version, "2.0");
    assert_eq!(
        result.ir.source.generator.as_deref(),
        Some("m2a-matrix-probe")
    );
    let value = serde_json::to_value(&result.ir.nodes[0].transform).unwrap();
    assert_eq!(value["kind"], "MATRIX");
    assert_eq!(
        value["matrix"],
        serde_json::json!([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0
        ])
    );
    assert_eq!(
        inspect_glb(&fixtures::nonfinite_node_transform(), &GlbLimits::default(),)
            .unwrap_err()
            .code,
        "M2A-GLB-NONFINITE-FLOAT"
    );
}

#[test]
fn metadata_and_geometry_limits_accept_boundary_and_reject_overage() {
    let minimal = fixtures::minimal_indexed_triangle();
    inspect_glb(
        &minimal,
        &GlbLimits {
            max_vertices: 3,
            max_indices: 3,
            ..GlbLimits::default()
        },
    )
    .expect("exact geometry limits are accepted");
    assert_eq!(
        inspect_glb(
            &minimal,
            &GlbLimits {
                max_vertices: 2,
                ..GlbLimits::default()
            }
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );

    let hierarchy = fixtures::axis_hierarchy_asymmetric();
    inspect_glb(
        &hierarchy,
        &GlbLimits {
            max_nodes: 2,
            ..GlbLimits::default()
        },
    )
    .expect("exact node limit is accepted");
    assert_eq!(
        inspect_glb(
            &hierarchy,
            &GlbLimits {
                max_nodes: 1,
                ..GlbLimits::default()
            }
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );
}

#[test]
fn malformed_truncated_and_arbitrary_inputs_never_panic() {
    let complete = fixtures::minimal_indexed_triangle();
    let limits = GlbLimits::default();
    assert_eq!(
        inspect_glb(&[], &limits).unwrap_err().code,
        "M2A-GLB-INPUT-EMPTY"
    );

    let mut bad_magic = complete.clone();
    bad_magic[0] = b'X';
    assert_eq!(
        inspect_glb(&bad_magic, &limits).unwrap_err().code,
        "M2A-GLB-HEADER-INVALID"
    );
    let mut bad_version = complete.clone();
    bad_version[4..8].copy_from_slice(&1_u32.to_le_bytes());
    assert_eq!(
        inspect_glb(&bad_version, &limits).unwrap_err().code,
        "M2A-GLB-VERSION-UNSUPPORTED"
    );
    let mut bad_length = complete.clone();
    bad_length[8..12].copy_from_slice(&0_u32.to_le_bytes());
    assert_eq!(
        inspect_glb(&bad_length, &limits).unwrap_err().code,
        "M2A-GLB-LENGTH-MISMATCH"
    );

    for length in 0..complete.len() {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            inspect_glb(&complete[..length], &limits)
        }));
        assert!(outcome.is_ok(), "panic for truncated prefix {length}");
        assert!(
            outcome.unwrap().is_err(),
            "truncated prefix parsed at {length}"
        );
    }

    let mut state = 0x5eed_u32;
    for length in 0..=256 {
        let mut bytes = vec![0_u8; length];
        for byte in &mut bytes {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *byte = (state >> 24) as u8;
        }
        let outcome = catch_unwind(AssertUnwindSafe(|| inspect_glb(&bytes, &limits)));
        assert!(outcome.is_ok(), "panic for arbitrary input length {length}");
    }
}

fn assert_gate(report: &m2a_core::glb::GlbInspectionReport, code: &str, severity: &str) {
    assert!(
        report
            .gates
            .iter()
            .any(|gate| gate.code == code && gate.severity == severity),
        "missing gate {code}/{severity}: {:?}",
        report.gates
    );
}
