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
    assert_gate(&report, "M2A-GLB-BASECOLOR-TEXTURE-MISSING", "WARNING");

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
fn material_image_inventory_preserves_ids_links_sampler_state_and_image_identity() {
    let input = fixtures::material_image_two_primitives();
    let original = input.clone();
    let first = ingest_glb(&input, &GlbLimits::default()).expect("material/image fixture ingest");
    let second = ingest_glb(&input, &GlbLimits::default()).expect("deterministic repeated ingest");
    assert_eq!(first, second);
    assert_eq!(
        input, original,
        "material/image ingest must not mutate source bytes"
    );

    assert_eq!(first.report.inventory.primitive_count, 2);
    assert_eq!(first.report.inventory.material_count, 2);
    assert_eq!(first.report.inventory.texture_count, 4);
    assert_eq!(first.report.inventory.sampler_count, 2);
    assert_eq!(first.report.inventory.image_count, 1);
    assert_eq!(
        first
            .ir
            .primitives
            .iter()
            .map(|primitive| primitive.material_id)
            .collect::<Vec<_>>(),
        [Some(0), Some(1)]
    );

    let paint = &first.ir.materials[0];
    assert_eq!(paint.id, 0);
    assert_eq!(paint.name.as_deref(), Some("painted-mask"));
    assert_eq!(paint.base_color_factor, [0.8, 0.7, 0.6, 0.5]);
    assert_eq!(paint.metallic_factor, 0.35);
    assert_eq!(paint.roughness_factor, 0.65);
    assert_eq!(paint.base_color_texture.as_ref().unwrap().texture_id, 0);
    assert_eq!(paint.base_color_texture.as_ref().unwrap().tex_coord_set, 0);
    assert_eq!(paint.normal_texture.as_ref().unwrap().texture_id, 1);
    assert_eq!(paint.normal_texture.as_ref().unwrap().tex_coord_set, 1);
    assert_eq!(
        paint
            .metallic_roughness_texture
            .as_ref()
            .unwrap()
            .texture_id,
        2
    );
    assert_eq!(
        paint
            .metallic_roughness_texture
            .as_ref()
            .unwrap()
            .tex_coord_set,
        2
    );
    assert_eq!(paint.emissive_factor, [0.03, 0.02, 0.01]);
    assert_eq!(paint.emissive_texture.as_ref().unwrap().texture_id, 3);
    assert_eq!(paint.emissive_texture.as_ref().unwrap().tex_coord_set, 3);
    assert_eq!(paint.alpha_mode, "MASK");
    assert_eq!(paint.alpha_cutoff, Some(0.33));
    assert!(paint.double_sided);

    let detail = &first.ir.materials[1];
    assert_eq!(detail.id, 1);
    assert_eq!(detail.name.as_deref(), Some("translucent-detail"));
    assert_eq!(detail.base_color_factor, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(detail.metallic_factor, 0.05);
    assert_eq!(detail.roughness_factor, 0.15);
    assert_eq!(detail.base_color_texture.as_ref().unwrap().texture_id, 0);
    assert_eq!(detail.base_color_texture.as_ref().unwrap().tex_coord_set, 0);
    assert!(detail.metallic_roughness_texture.is_none());
    assert_eq!(detail.normal_texture.as_ref().unwrap().texture_id, 1);
    assert_eq!(detail.normal_texture.as_ref().unwrap().tex_coord_set, 0);
    assert_eq!(detail.emissive_factor, [0.0, 0.0, 0.0]);
    assert!(detail.emissive_texture.is_none());
    assert_eq!(detail.alpha_mode, "BLEND");
    assert_eq!(detail.alpha_cutoff, None);
    assert!(!detail.double_sided);

    assert_eq!(first.ir.textures[0].id, 0);
    assert_eq!(first.ir.textures[0].source_image_id, 0);
    assert_eq!(first.ir.textures[0].sampler_index, Some(0));
    assert_eq!(first.ir.textures[1].id, 1);
    assert_eq!(first.ir.textures[1].source_image_id, 0);
    assert_eq!(first.ir.textures[1].sampler_index, Some(1));
    assert_eq!(first.ir.textures[2].id, 2);
    assert_eq!(first.ir.textures[2].source_image_id, 0);
    assert_eq!(first.ir.textures[2].sampler_index, Some(0));
    assert_eq!(first.ir.textures[3].id, 3);
    assert_eq!(first.ir.textures[3].source_image_id, 0);
    assert_eq!(first.ir.textures[3].sampler_index, Some(1));

    let sampler = &first.ir.samplers[0];
    assert_eq!(sampler.id, 0);
    assert_eq!(sampler.name.as_deref(), Some("nearest-clamped-mirrored"));
    assert_eq!(sampler.mag_filter.as_deref(), Some("NEAREST"));
    assert_eq!(sampler.min_filter.as_deref(), Some("LINEAR_MIPMAP_LINEAR"));
    assert_eq!(sampler.wrap_s, "CLAMP_TO_EDGE");
    assert_eq!(sampler.wrap_t, "MIRRORED_REPEAT");
    let sampler = &first.ir.samplers[1];
    assert_eq!(sampler.mag_filter.as_deref(), Some("LINEAR"));
    assert_eq!(
        sampler.min_filter.as_deref(),
        Some("NEAREST_MIPMAP_NEAREST")
    );
    assert_eq!(sampler.wrap_s, "REPEAT");
    assert_eq!(sampler.wrap_t, "CLAMP_TO_EDGE");

    let image = &first.ir.images[0];
    assert_eq!(image.id, 0);
    assert_eq!(image.name.as_deref(), Some("embedded-one-pixel"));
    assert_eq!(image.mime_type, "image/png");
    assert_eq!(image.byte_length, fixtures::MINIMAL_PNG.len());
    assert_eq!(
        image.sha256,
        "7495d6b81becce6ce9a16cbb139e1f390a9be4708f217d62bd2fa536fd728623"
    );
    assert!(!image.payload_embedded_in_json);

    let json = serde_json::to_value(&first).unwrap();
    let image_json = json["ir"]["images"][0].as_object().unwrap();
    assert_eq!(
        image_json.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "byteLength",
            "byteOffset",
            "id",
            "mimeType",
            "name",
            "payloadEmbeddedInJson",
            "sha256"
        ]
    );
    let serialized = serde_json::to_string(&first).unwrap();
    assert!(!serialized.contains("data:image"));
    assert!(!serialized.contains("iVBORw0KGgo"));
    assert!(!serialized.contains("imageBytes"));
    assert!(!serialized.contains("\"payload\":"));
}

#[test]
fn material_texture_and_image_limits_are_preflighted_at_exact_boundaries() {
    let input = fixtures::material_image_two_primitives();
    let exact = GlbLimits {
        max_materials: 2,
        max_textures: 4,
        max_samplers: 2,
        max_images: 1,
        max_single_image_bytes: fixtures::MINIMAL_PNG.len(),
        max_total_image_bytes: fixtures::MINIMAL_PNG.len(),
        ..GlbLimits::default()
    };
    inspect_glb(&input, &exact).expect("exact material/texture/image limits are accepted");

    for limits in [
        GlbLimits {
            max_materials: 1,
            ..exact.clone()
        },
        GlbLimits {
            max_textures: 1,
            ..exact.clone()
        },
        GlbLimits {
            max_samplers: 1,
            ..exact.clone()
        },
        GlbLimits {
            max_images: 0,
            ..exact.clone()
        },
        GlbLimits {
            max_single_image_bytes: fixtures::MINIMAL_PNG.len() - 1,
            ..exact.clone()
        },
        GlbLimits {
            max_total_image_bytes: fixtures::MINIMAL_PNG.len() - 1,
            ..exact.clone()
        },
    ] {
        assert_eq!(
            inspect_glb(&input, &limits).unwrap_err().code,
            "M2A-GLB-LIMIT-EXCEEDED"
        );
    }

    let duplicate = fixtures::material_image_duplicate_image_reference();
    inspect_glb(
        &duplicate,
        &GlbLimits {
            max_images: 2,
            max_total_image_bytes: fixtures::MINIMAL_PNG.len() * 2,
            ..GlbLimits::default()
        },
    )
    .expect("exact cumulative image-byte limit is accepted");
    assert_eq!(
        inspect_glb(
            &duplicate,
            &GlbLimits {
                max_images: 2,
                max_total_image_bytes: fixtures::MINIMAL_PNG.len() * 2 - 1,
                ..GlbLimits::default()
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );

    assert_eq!(
        inspect_glb(
            &fixtures::material_image_invalid_buffer_view(),
            &GlbLimits::default()
        )
        .unwrap_err()
        .code,
        "M2A-GLB-INTEGER-OVERFLOW"
    );
    let nonfinite = inspect_glb(
        &fixtures::nonfinite_material_factor(),
        &GlbLimits::default(),
    )
    .unwrap_err();
    assert_eq!(nonfinite.code, "M2A-GLB-NONFINITE-FLOAT");
    assert_eq!(nonfinite.json_path.as_deref(), Some("materials[0]"));
}

#[test]
fn material_sampler_defaults_full_enums_and_references_are_validated_before_document_use() {
    let defaults = fixtures::mutate_json(fixtures::material_image_two_primitives(), |root| {
        root["textures"][0]
            .as_object_mut()
            .unwrap()
            .remove("sampler");
        for field in ["magFilter", "minFilter", "wrapS", "wrapT"] {
            root["samplers"][0].as_object_mut().unwrap().remove(field);
        }
    });
    let defaults = ingest_glb(&defaults, &GlbLimits::default()).unwrap();
    assert_eq!(defaults.ir.textures[0].sampler_index, None);
    assert_eq!(defaults.ir.samplers[0].mag_filter, None);
    assert_eq!(defaults.ir.samplers[0].min_filter, None);
    assert_eq!(defaults.ir.samplers[0].wrap_s, "REPEAT");
    assert_eq!(defaults.ir.samplers[0].wrap_t, "REPEAT");

    for (field, values) in [
        ("magFilter", &[9728_u64, 9729][..]),
        ("minFilter", &[9728_u64, 9729, 9984, 9985, 9986, 9987][..]),
        ("wrapS", &[33071_u64, 33648, 10497][..]),
        ("wrapT", &[33071_u64, 33648, 10497][..]),
    ] {
        for value in values {
            let input = fixtures::mutate_json(fixtures::material_image_two_primitives(), |root| {
                root["samplers"][0][field] = serde_json::json!(value)
            });
            ingest_glb(&input, &GlbLimits::default())
                .unwrap_or_else(|error| panic!("valid {field}={value} rejected: {error}"));
        }
    }

    let missing_position_with_bad_texture =
        fixtures::mutate_json(fixtures::material_image_two_primitives(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("POSITION");
            root["textures"][0]["source"] = serde_json::json!(99);
        });
    let missing_position_with_bad_sampler =
        fixtures::mutate_json(fixtures::material_image_two_primitives(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("POSITION");
            root["textures"][0]["sampler"] = serde_json::json!(99);
        });
    let missing_position_with_bad_sampler_enum =
        fixtures::mutate_json(fixtures::material_image_two_primitives(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("POSITION");
            root["samplers"][0]["wrapS"] = serde_json::json!(12345);
        });
    for input in [
        missing_position_with_bad_texture,
        missing_position_with_bad_sampler,
        missing_position_with_bad_sampler_enum,
    ] {
        for outcome in [
            catch_unwind(AssertUnwindSafe(|| {
                inspect_glb(&input, &GlbLimits::default()).map(|_| ())
            })),
            catch_unwind(AssertUnwindSafe(|| {
                ingest_glb(&input, &GlbLimits::default()).map(|_| ())
            })),
        ] {
            let error = outcome.expect("crafted missing-POSITION input must never panic");
            assert_eq!(error.unwrap_err().code, "M2A-GLB-JSON-INVALID");
        }
    }
}

#[test]
fn embedded_image_mime_allowlist_uses_declared_value_without_magic_sniffing() {
    let jpeg_declared_over_png_bytes =
        fixtures::material_image_with_declared_mime(serde_json::json!("image/jpeg"));
    let accepted = ingest_glb(&jpeg_declared_over_png_bytes, &GlbLimits::default())
        .expect("allowed declared JPEG MIME is accepted without decoding or magic sniffing");
    assert_eq!(accepted.ir.images[0].mime_type, "image/jpeg");
    assert_eq!(
        accepted.ir.images[0].sha256,
        "7495d6b81becce6ce9a16cbb139e1f390a9be4708f217d62bd2fa536fd728623"
    );

    for input in [
        fixtures::material_image_with_declared_mime(serde_json::json!("image/webp")),
        fixtures::material_image_with_declared_mime(serde_json::json!(123)),
        fixtures::material_image_without_declared_mime(),
    ] {
        let error = inspect_glb(&input, &GlbLimits::default()).unwrap_err();
        assert_eq!(error.code, "M2A-GLB-IMAGE-MIME-UNSUPPORTED");
        assert_eq!(error.json_path.as_deref(), Some("images[0].mimeType"));
    }
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
        (fixtures::buffer_view_oob(), "M2A-GLB-INTEGER-OVERFLOW"),
        (fixtures::accessor_oob(), "M2A-GLB-INTEGER-OVERFLOW"),
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
    let validation_error = fixtures::mutate_json(fixtures::minimal_indexed_triangle(), |root| {
        root["nodes"][0]["rotation"] = serde_json::json!([0.0, 0.0, 1.0]);
    });
    for _ in 0..2 {
        assert_eq!(
            inspect_glb(&validation_error, &GlbLimits::default())
                .unwrap_err()
                .code,
            "M2A-GLB-JSON-INVALID"
        );
    }
}

#[test]
fn skin_animation_fixture_preserves_source_values_and_optional_ibm_semantics() {
    let without_ibm = fixtures::skin_animation_without_inverse_bind_matrices();
    let absent = ingest_glb(&without_ibm, &GlbLimits::default()).unwrap();
    assert_eq!(absent.ir.skins.len(), 1);
    assert_eq!(absent.ir.skins[0].joint_node_ids, [0, 1]);
    assert_eq!(absent.ir.skins[0].skeleton_root_node_id, Some(0));
    assert!(absent.ir.skins[0].inverse_bind_matrices.is_empty());

    let input = fixtures::skin_animation_with_inverse_bind_matrices();
    let original = input.clone();
    let first = ingest_glb(&input, &GlbLimits::default()).unwrap();
    let second = ingest_glb(&input, &GlbLimits::default()).unwrap();
    assert_eq!(first, second, "fixture E ingest must be deterministic");
    assert_eq!(
        input, original,
        "fixture E input bytes must remain immutable"
    );

    assert_eq!(first.report.inventory.skin_count, 1);
    assert_eq!(first.report.inventory.joint_reference_count, 2);
    assert_eq!(first.report.inventory.animation_count, 1);
    assert_eq!(first.report.inventory.keyframe_count, 9);
    let skin = &first.ir.skins[0];
    assert_eq!(skin.name.as_deref(), Some("two-joint-skin"));
    assert_eq!(skin.inverse_bind_matrices.len(), 2);
    assert_eq!(
        skin.inverse_bind_matrices[0],
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0
        ]
    );
    assert_eq!(skin.inverse_bind_matrices[1][12..16], [2.0, 3.0, 4.0, 1.0]);

    let primitive = &first.ir.primitives[0];
    assert_eq!(
        primitive.joints0,
        [[0, 1, 0, 0], [1, 0, 0, 0], [0, 1, 0, 0]]
    );
    assert_eq!(
        primitive.weights0,
        [
            [0.75, 0.25, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.4, 0.6, 0.0, 0.0]
        ]
    );

    let animation = &first.ir.animations[0];
    assert_eq!(animation.name.as_deref(), Some("source-trs"));
    assert_eq!(animation.duration_seconds, 1.25);
    assert_eq!(animation.samplers.len(), 3);
    assert_eq!(animation.channels.len(), 3);
    for sampler in &animation.samplers {
        assert_eq!(sampler.input_times_seconds, [0.0, 0.5, 1.25]);
    }
    assert_eq!(animation.samplers[0].interpolation, "LINEAR");
    assert_eq!(animation.samplers[0].output_accessor_type, "VEC3");
    assert_eq!(
        animation.samplers[0].output_values,
        [0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
    );
    assert_eq!(animation.samplers[1].interpolation, "STEP");
    assert_eq!(animation.samplers[1].output_accessor_type, "VEC4");
    assert_eq!(animation.samplers[2].interpolation, "CUBICSPLINE");
    assert_eq!(animation.samplers[2].output_accessor_type, "VEC3");
    assert_eq!(animation.samplers[2].output_values.len(), 27);
    assert_eq!(animation.samplers[2].output_values[3..6], [1.0, 1.0, 1.0]);
    assert_eq!(animation.samplers[2].output_values[21..24], [2.0, 3.0, 4.0]);
    assert_eq!(
        animation
            .channels
            .iter()
            .map(|channel| (
                channel.sampler_id,
                channel.target_node_id,
                channel.target_path.as_str()
            ))
            .collect::<Vec<_>>(),
        [(0, 1, "TRANSLATION"), (1, 1, "ROTATION"), (2, 1, "SCALE")]
    );

    let extra = ingest_glb(
        &fixtures::skin_animation_with_extra_inverse_bind_matrix(),
        &GlbLimits::default(),
    )
    .expect("inverse-bind count above joint count is valid source inventory");
    assert_eq!(extra.ir.skins[0].joint_node_ids.len(), 2);
    assert_eq!(extra.ir.skins[0].inverse_bind_matrices.len(), 3);
    assert_eq!(
        extra.ir.skins[0].inverse_bind_matrices[2][12..15],
        [7.0, 8.0, 9.0]
    );
}

#[test]
fn secondary_skin_influence_set_is_a_stable_warning() {
    let report = inspect_glb(
        &fixtures::secondary_skin_influence_set(),
        &GlbLimits::default(),
    )
    .unwrap();
    assert_gate(&report, "M2A-GLB-SKIN-INFLUENCE-COUNT", "WARNING");
    assert!(report.conversion_eligible);
}

#[test]
fn animation_default_linear_and_weights_blocking_gate_are_explicit() {
    let default_linear = fixtures::mutate_json(
        fixtures::skin_animation_with_inverse_bind_matrices(),
        |root| {
            root["animations"][0]["samplers"][0]
                .as_object_mut()
                .unwrap()
                .remove("interpolation");
        },
    );
    assert_eq!(
        ingest_glb(&default_linear, &GlbLimits::default())
            .unwrap()
            .ir
            .animations[0]
            .samplers[0]
            .interpolation,
        "LINEAR"
    );

    let weights = ingest_glb(
        &fixtures::with_weights_animation_channel(),
        &GlbLimits::default(),
    )
    .unwrap();
    assert_eq!(weights.ir.animations[0].channels[3].target_path, "WEIGHTS");
    assert_gate(
        &weights.report,
        "M2A-GLB-ANIMATION-WEIGHTS-DEFERRED",
        "BLOCKING",
    );
    assert!(!weights.report.conversion_eligible);
}

#[test]
fn skin_animation_limits_and_runtime_decoded_budget_have_exact_boundaries() {
    let input = fixtures::skin_animation_with_inverse_bind_matrices();
    let exact = GlbLimits {
        max_skins: 1,
        max_joints: 2,
        max_animations: 1,
        max_animation_samplers: 3,
        max_animation_channels: 3,
        max_keyframes: 9,
        // joint refs 8 + JOINTS 24 + WEIGHTS 48 + IBM 128 + times decoded per sampler 36
        // + translation 36 + rotation 48 + cubic scale 108
        max_decoded_skin_animation_bytes: 436,
        ..GlbLimits::default()
    };
    inspect_glb(&input, &exact)
        .expect("exact skin/animation counts and runtime materialization budget are accepted");

    for limits in [
        GlbLimits {
            max_skins: 0,
            ..exact.clone()
        },
        GlbLimits {
            max_joints: 1,
            ..exact.clone()
        },
        GlbLimits {
            max_animations: 0,
            ..exact.clone()
        },
        GlbLimits {
            max_animation_samplers: 2,
            ..exact.clone()
        },
        GlbLimits {
            max_animation_channels: 2,
            ..exact.clone()
        },
        GlbLimits {
            max_keyframes: 8,
            ..exact.clone()
        },
        GlbLimits {
            max_decoded_skin_animation_bytes: 435,
            ..exact.clone()
        },
    ] {
        assert_eq!(
            inspect_glb(&input, &limits).unwrap_err().code,
            "M2A-GLB-LIMIT-EXCEEDED"
        );
    }

    let two_skins = fixtures::mutate_json(input, |root| {
        let duplicate = root["skins"][0].clone();
        root["skins"].as_array_mut().unwrap().push(duplicate);
    });
    inspect_glb(
        &two_skins,
        &GlbLimits {
            max_skins: 2,
            max_joints: 4,
            ..GlbLimits::default()
        },
    )
    .expect("aggregate joint count is accepted exactly at the boundary");
    assert_eq!(
        inspect_glb(
            &two_skins,
            &GlbLimits {
                max_skins: 2,
                max_joints: 3,
                ..GlbLimits::default()
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );
}

#[test]
fn reused_animation_accessors_amplify_budget_per_sampler_materialization() {
    let input = fixtures::mutate_json(
        fixtures::skin_animation_with_inverse_bind_matrices(),
        |root| {
            let reused = root["animations"][0]["samplers"][0].clone();
            let samplers = root["animations"][0]["samplers"].as_array_mut().unwrap();
            for _ in 0..5 {
                samplers.push(reused.clone());
            }
        },
    );
    let exact = GlbLimits {
        max_animation_samplers: 8,
        max_keyframes: 24,
        // Base fixture runtime budget 436 plus five reused input/output decodes at 48 bytes each.
        max_decoded_skin_animation_bytes: 676,
        ..GlbLimits::default()
    };
    let result = ingest_glb(&input, &exact).expect("per-use decoded budget exact boundary");
    assert_eq!(result.ir.animations[0].samplers.len(), 8);
    assert_eq!(result.report.inventory.keyframe_count, 24);
    assert_eq!(
        inspect_glb(
            &input,
            &GlbLimits {
                max_decoded_skin_animation_bytes: 675,
                ..exact
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );
}

#[test]
fn skin_structure_layout_and_values_reject_the_focused_negative_matrix() {
    let base = fixtures::skin_animation_with_inverse_bind_matrices();
    let mut cases = Vec::new();
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["skins"][0]["joints"] = serde_json::json!([]);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["skins"][0]["joints"][1] = serde_json::json!(99);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["skins"][0]["skeleton"] = serde_json::json!(99);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][6]["componentType"] = serde_json::json!(5123);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][6]["type"] = serde_json::json!("VEC4");
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][6]["normalized"] = serde_json::json!(true);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][6]["count"] = serde_json::json!(1);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .unwrap()
            .remove("WEIGHTS_0");
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][5]["count"] = serde_json::json!(2);
    }));
    cases.push(fixtures::mutate_accessor_u8(base.clone(), 4, 0, 2));
    cases.push(fixtures::mutate_accessor_f32(base.clone(), 5, 0, -0.25));

    for input in cases {
        assert_eq!(
            inspect_glb(&input, &GlbLimits::default()).unwrap_err().code,
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID"
        );
    }
    assert_eq!(
        inspect_glb(
            &fixtures::mutate_accessor_f32(base.clone(), 5, 0, f32::INFINITY),
            &GlbLimits::default()
        )
        .unwrap_err()
        .code,
        "M2A-GLB-NONFINITE-FLOAT"
    );
    assert_eq!(
        inspect_glb(
            &fixtures::mutate_accessor_f32(base, 6, 0, f32::NAN),
            &GlbLimits::default()
        )
        .unwrap_err()
        .code,
        "M2A-GLB-NONFINITE-FLOAT"
    );
}

#[test]
fn animation_refs_layout_times_outputs_and_duplicates_reject_negative_matrix() {
    let base = fixtures::skin_animation_with_inverse_bind_matrices();
    let mut cases = Vec::new();
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["animations"][0]["channels"][0]["sampler"] = serde_json::json!(99);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["animations"][0]["channels"][0]["target"]["node"] = serde_json::json!(99);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["animations"][0]["samplers"][0]["interpolation"] =
            serde_json::json!("CATMULLROMSPLINE");
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][7]["componentType"] = serde_json::json!(5123);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][7]["count"] = serde_json::json!(0);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][8]["type"] = serde_json::json!("VEC2");
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][8]["count"] = serde_json::json!(2);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][10]["count"] = serde_json::json!(3);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        let duplicate = root["animations"][0]["channels"][0].clone();
        root["animations"][0]["channels"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
    }));
    cases.push(fixtures::mutate_json(base.clone(), |root| {
        root["accessors"][7]["min"] = serde_json::json!([0.25]);
    }));
    cases.push(fixtures::mutate_accessor_f32(base.clone(), 7, 1, 0.0));
    cases.push(fixtures::mutate_accessor_f32(base.clone(), 7, 0, -0.25));

    for input in cases {
        assert_eq!(
            inspect_glb(&input, &GlbLimits::default()).unwrap_err().code,
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID"
        );
    }
    for input in [
        fixtures::mutate_accessor_f32(base.clone(), 7, 1, f32::INFINITY),
        fixtures::mutate_accessor_f32(base, 8, 0, f32::NAN),
    ] {
        assert_eq!(
            inspect_glb(&input, &GlbLimits::default()).unwrap_err().code,
            "M2A-GLB-NONFINITE-FLOAT"
        );
    }
}

#[test]
fn metadata_and_geometry_limits_accept_boundary_and_reject_overage() {
    let minimal = fixtures::minimal_indexed_triangle();
    inspect_glb(
        &minimal,
        &GlbLimits {
            max_nodes: 1,
            max_meshes: 1,
            max_primitives: 1,
            max_accessors: 4,
            max_buffer_views: 4,
            max_materials: 1,
            max_vertices: 3,
            max_indices: 3,
            ..GlbLimits::default()
        },
    )
    .expect("exact metadata and geometry limits are accepted");
    for constrained in [
        GlbLimits {
            max_nodes: 0,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_meshes: 0,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_primitives: 0,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_accessors: 3,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_buffer_views: 3,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_materials: 0,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_vertices: 2,
            ..GlbLimits::default()
        },
        GlbLimits {
            max_indices: 2,
            ..GlbLimits::default()
        },
    ] {
        assert_eq!(
            inspect_glb(&minimal, &constrained).unwrap_err().code,
            "M2A-GLB-LIMIT-EXCEEDED"
        );
    }

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
fn fixture_f_covers_geometry_gates_and_all_index_encodings() {
    let limits = GlbLimits::default();

    let missing_position = inspect_glb(&fixtures::missing_position(), &limits).unwrap();
    assert_gate(&missing_position, "M2A-GLB-POSITION-MISSING", "BLOCKING");
    assert!(!missing_position.conversion_eligible);

    let accessorless = fixtures::missing_position_without_accessors_or_views();
    let original = accessorless.clone();
    let first_report = catch_unwind(AssertUnwindSafe(|| inspect_glb(&accessorless, &limits)))
        .expect("accessorless missing-POSITION inspect must not panic")
        .expect("accessorless missing-POSITION inspect must return a report");
    let second_report = inspect_glb(&accessorless, &limits).unwrap();
    assert_eq!(first_report, second_report);
    assert_gate(&first_report, "M2A-GLB-POSITION-MISSING", "BLOCKING");
    assert_eq!(first_report.statistics.vertex_count, 0);
    assert_eq!(first_report.statistics.index_count, 0);
    let first_ingest = catch_unwind(AssertUnwindSafe(|| ingest_glb(&accessorless, &limits)))
        .expect("accessorless missing-POSITION ingest must not panic")
        .expect("accessorless missing-POSITION ingest must return source IR");
    let second_ingest = ingest_glb(&accessorless, &limits).unwrap();
    assert_eq!(first_ingest, second_ingest);
    assert!(first_ingest.ir.primitives[0].positions.is_empty());
    assert!(first_ingest.ir.primitives[0].indices.is_empty());
    assert_eq!(accessorless, original);

    let mismatched = inspect_glb(&fixtures::mismatched_attributes(), &limits).unwrap();
    assert_gate(&mismatched, "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH", "BLOCKING");

    let morph = inspect_glb(&fixtures::morph_target(), &limits).unwrap();
    assert_gate(&morph, "M2A-GLB-MORPH-TARGETS-DEFERRED", "BLOCKING");

    let incomplete = inspect_glb(&fixtures::incomplete_triangle(), &limits).unwrap();
    assert_gate(&incomplete, "M2A-GLB-INCOMPLETE-TRIANGLES", "BLOCKING");
    let degenerate = inspect_glb(&fixtures::degenerate_triangle(), &limits).unwrap();
    assert_gate(&degenerate, "M2A-GLB-DEGENERATE-TRIANGLES", "BLOCKING");
    let index_oob = inspect_glb(&fixtures::index_out_of_bounds(), &limits).unwrap();
    assert_gate(&index_oob, "M2A-GLB-INDEX-OOB", "BLOCKING");

    for component_type in [5121, 5123, 5125] {
        let result = ingest_glb(
            &fixtures::indexed_triangle_with_component(component_type),
            &limits,
        )
        .unwrap();
        assert_eq!(result.ir.primitives[0].indices, [0, 1, 2]);
        assert!(result.ir.primitives[0].source_was_indexed);
    }
    let non_indexed = ingest_glb(&fixtures::non_indexed_triangle(), &limits).unwrap();
    assert_eq!(non_indexed.ir.primitives[0].indices, [0, 1, 2]);
    assert!(!non_indexed.ir.primitives[0].source_was_indexed);

    for component_type in [5122, 5126] {
        let source = if component_type == 5126 {
            fixtures::indexed_triangle_with_component(5125)
        } else {
            fixtures::minimal_indexed_triangle()
        };
        let invalid = fixtures::mutate_json(source, |root| {
            root["accessors"][3]["componentType"] = serde_json::json!(component_type);
        });
        assert_eq!(
            inspect_glb(&invalid, &limits).unwrap_err().code,
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID"
        );
    }
}

#[test]
fn header_chunk_json_and_bin_negative_matrix_has_stable_codes() {
    let limits = GlbLimits::default();
    let complete = fixtures::minimal_indexed_triangle();
    let json_len = u32::from_le_bytes(complete[12..16].try_into().unwrap()) as usize;
    let json_end = 20 + json_len;

    let mut missing_bin = complete[..json_end].to_vec();
    set_declared_length(&mut missing_bin);
    assert_eq!(
        inspect_glb(&missing_bin, &limits).unwrap_err().code,
        "M2A-GLB-BIN-MISSING"
    );

    let mut wrong_first_type = complete.clone();
    wrong_first_type[16..20].copy_from_slice(&0x004e_4942_u32.to_le_bytes());
    assert_eq!(
        inspect_glb(&wrong_first_type, &limits).unwrap_err().code,
        "M2A-GLB-CHUNK-INVALID"
    );
    let mut wrong_second_type = complete.clone();
    wrong_second_type[json_end + 4..json_end + 8].copy_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    assert_eq!(
        inspect_glb(&wrong_second_type, &limits).unwrap_err().code,
        "M2A-GLB-CHUNK-INVALID"
    );

    let mut duplicate_bin = complete.clone();
    duplicate_bin.extend_from_slice(&[0, 0, 0, 0, 0x42, 0x49, 0x4e, 0]);
    set_declared_length(&mut duplicate_bin);
    assert_eq!(
        inspect_glb(&duplicate_bin, &limits).unwrap_err().code,
        "M2A-GLB-CHUNK-INVALID"
    );

    let mut unaligned_json = complete.clone();
    unaligned_json[12..16].copy_from_slice(&((json_len - 1) as u32).to_le_bytes());
    assert_eq!(
        inspect_glb(&unaligned_json, &limits).unwrap_err().code,
        "M2A-GLB-CHUNK-INVALID"
    );
    let bin_len = u32::from_le_bytes(complete[json_end..json_end + 4].try_into().unwrap()) as usize;
    let mut unaligned_bin = complete.clone();
    unaligned_bin[json_end..json_end + 4].copy_from_slice(&((bin_len - 1) as u32).to_le_bytes());
    assert_eq!(
        inspect_glb(&unaligned_bin, &limits).unwrap_err().code,
        "M2A-GLB-CHUNK-INVALID"
    );

    let mut invalid_utf8 = complete.clone();
    invalid_utf8[20] = 0xff;
    assert_eq!(
        inspect_glb(&invalid_utf8, &limits).unwrap_err().code,
        "M2A-GLB-JSON-INVALID"
    );
    let invalid_root = fixtures::root_value(serde_json::json!([]));
    assert_eq!(
        inspect_glb(&invalid_root, &limits).unwrap_err().code,
        "M2A-GLB-JSON-INVALID"
    );
    let invalid_asset = fixtures::mutate_json(complete.clone(), |root| {
        root["asset"]["version"] = serde_json::json!("1.0");
    });
    assert_eq!(
        inspect_glb(&invalid_asset, &limits).unwrap_err().code,
        "M2A-GLB-JSON-INVALID"
    );

    inspect_glb(
        &complete,
        &GlbLimits {
            max_input_bytes: complete.len(),
            max_json_chunk_bytes: json_len,
            ..limits.clone()
        },
    )
    .expect("input and JSON exact boundaries are accepted");
    for constrained in [
        GlbLimits {
            max_input_bytes: complete.len() - 1,
            ..limits.clone()
        },
        GlbLimits {
            max_json_chunk_bytes: json_len - 1,
            ..limits.clone()
        },
    ] {
        assert!(matches!(
            inspect_glb(&complete, &constrained)
                .unwrap_err()
                .code
                .as_str(),
            "M2A-GLB-INPUT-LIMIT-EXCEEDED" | "M2A-GLB-LIMIT-EXCEEDED"
        ));
    }
}

#[test]
fn buffer_view_accessor_and_reference_matrix_distinguishes_overflow_from_oob() {
    let limits = GlbLimits::default();
    let minimal = fixtures::minimal_indexed_triangle();

    let external = fixtures::external_buffer_uri();
    assert_eq!(
        inspect_glb(&external, &limits).unwrap_err().code,
        "M2A-GLB-EXTERNAL-URI-UNSUPPORTED"
    );
    for input in [
        fixtures::primitive_compression_extension("KHR_draco_mesh_compression"),
        fixtures::buffer_view_compression_extension("EXT_meshopt_compression"),
    ] {
        assert_eq!(
            inspect_glb(&input, &limits).unwrap_err().code,
            "M2A-GLB-COMPRESSION-UNSUPPORTED"
        );
    }

    let bad_buffer_index = fixtures::mutate_json(minimal.clone(), |root| {
        root["bufferViews"][0]["buffer"] = serde_json::json!(1);
    });
    assert_eq!(
        inspect_glb(&bad_buffer_index, &limits).unwrap_err().code,
        "M2A-GLB-BUFFER-VIEW-OOB"
    );
    let range_oob = fixtures::mutate_json(minimal.clone(), |root| {
        root["bufferViews"][0]["byteLength"] = serde_json::json!(1_000_000);
    });
    assert_eq!(
        inspect_glb(&range_oob, &limits).unwrap_err().code,
        "M2A-GLB-BUFFER-VIEW-OOB"
    );
    let buffer_length_mismatch = fixtures::mutate_json(minimal.clone(), |root| {
        let declared = root["buffers"][0]["byteLength"].as_u64().unwrap();
        root["buffers"][0]["byteLength"] = serde_json::json!(declared - 4);
    });
    assert_eq!(
        inspect_glb(&buffer_length_mismatch, &limits)
            .unwrap_err()
            .code,
        "M2A-GLB-BUFFER-VIEW-OOB"
    );
    assert_eq!(
        inspect_glb(&fixtures::buffer_view_oob(), &limits)
            .unwrap_err()
            .code,
        "M2A-GLB-INTEGER-OVERFLOW"
    );

    let view_index_oob = fixtures::mutate_json(minimal.clone(), |root| {
        root["accessors"][0]["bufferView"] = serde_json::json!(999);
    });
    assert_eq!(
        inspect_glb(&view_index_oob, &limits).unwrap_err().code,
        "M2A-GLB-ACCESSOR-OOB"
    );
    let accessor_range_oob = fixtures::mutate_json(minimal.clone(), |root| {
        root["accessors"][0]["count"] = serde_json::json!(4);
    });
    assert_eq!(
        inspect_glb(&accessor_range_oob, &limits).unwrap_err().code,
        "M2A-GLB-ACCESSOR-OOB"
    );
    assert_eq!(
        inspect_glb(&fixtures::accessor_oob(), &limits)
            .unwrap_err()
            .code,
        "M2A-GLB-INTEGER-OVERFLOW"
    );
    let invalid_stride = fixtures::mutate_json(minimal.clone(), |root| {
        root["bufferViews"][0]["byteStride"] = serde_json::json!(2);
    });
    assert_eq!(
        inspect_glb(&invalid_stride, &limits).unwrap_err().code,
        "M2A-GLB-ACCESSOR-LAYOUT-INVALID"
    );
    let misaligned = fixtures::mutate_json(minimal.clone(), |root| {
        root["accessors"][3]["byteOffset"] = serde_json::json!(1);
        root["accessors"][3]["count"] = serde_json::json!(2);
    });
    assert_eq!(
        inspect_glb(&misaligned, &limits).unwrap_err().code,
        "M2A-GLB-ACCESSOR-LAYOUT-INVALID"
    );

    let bad_default_scene = fixtures::mutate_json(minimal.clone(), |root| {
        root["scene"] = serde_json::json!(1);
    });
    let bad_root = fixtures::mutate_json(minimal.clone(), |root| {
        root["scenes"][0]["nodes"][0] = serde_json::json!(10);
    });
    let bad_child = fixtures::mutate_json(minimal.clone(), |root| {
        root["nodes"][0]["children"] = serde_json::json!([10]);
    });
    for input in [bad_default_scene, bad_root, bad_child] {
        assert_eq!(
            inspect_glb(&input, &limits).unwrap_err().code,
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID"
        );
    }
    let cycle = fixtures::mutate_json(minimal.clone(), |root| {
        root["nodes"][0]["children"] = serde_json::json!([0]);
    });
    assert_eq!(
        inspect_glb(&cycle, &limits).unwrap_err().code,
        "M2A-GLB-NODE-CYCLE"
    );
    let hierarchy = fixtures::axis_hierarchy_asymmetric();
    inspect_glb(
        &hierarchy,
        &GlbLimits {
            max_node_depth: 1,
            ..limits.clone()
        },
    )
    .expect("exact depth boundary is accepted");
    assert_eq!(
        inspect_glb(
            &hierarchy,
            &GlbLimits {
                max_node_depth: 0,
                ..limits
            },
        )
        .unwrap_err()
        .code,
        "M2A-GLB-LIMIT-EXCEEDED"
    );
}

#[test]
fn every_json_offset_count_and_reference_uses_the_canonical_u32_domain() {
    let over = u64::from(u32::MAX) + 1;
    let minimal = fixtures::minimal_indexed_triangle();
    let mut cases = vec![
        fixtures::mutate_json(minimal.clone(), |root| {
            root["buffers"][0]["byteLength"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["bufferViews"][0]["buffer"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["bufferViews"][0]["byteOffset"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["bufferViews"][0]["byteLength"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["bufferViews"][0]["byteStride"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["accessors"][0]["componentType"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["accessors"][0]["count"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["accessors"][0]["bufferView"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["accessors"][0]["byteOffset"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["scene"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["scenes"][0]["nodes"][0] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["nodes"][0]["children"] = serde_json::json!([over]);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["nodes"][0]["mesh"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["meshes"][0]["primitives"][0]["indices"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal.clone(), |root| {
            root["meshes"][0]["primitives"][0]["material"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(minimal, |root| {
            root["meshes"][0]["primitives"][0]["mode"] = serde_json::json!(over);
        }),
    ];
    let material = fixtures::material_image_two_primitives();
    cases.extend([
        fixtures::mutate_json(material.clone(), |root| {
            root["textures"][0]["source"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(material.clone(), |root| {
            root["textures"][0]["sampler"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(material.clone(), |root| {
            root["images"][0]["bufferView"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(material, |root| {
            root["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["index"] =
                serde_json::json!(over);
        }),
    ]);
    let animated = fixtures::skin_animation_with_inverse_bind_matrices();
    cases.extend([
        fixtures::mutate_json(animated.clone(), |root| {
            root["skins"][0]["joints"][0] = serde_json::json!(over);
        }),
        fixtures::mutate_json(animated.clone(), |root| {
            root["skins"][0]["skeleton"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(animated.clone(), |root| {
            root["skins"][0]["inverseBindMatrices"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(animated.clone(), |root| {
            root["animations"][0]["samplers"][0]["input"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(animated.clone(), |root| {
            root["animations"][0]["samplers"][0]["output"] = serde_json::json!(over);
        }),
        fixtures::mutate_json(animated, |root| {
            root["animations"][0]["channels"][0]["sampler"] = serde_json::json!(over);
        }),
    ]);

    for (index, input) in cases.into_iter().enumerate() {
        let error = inspect_glb(&input, &GlbLimits::default()).unwrap_err();
        assert_eq!(
            error.code, "M2A-GLB-INTEGER-OVERFLOW",
            "u32-domain case {index} returned {error:?}"
        );
    }
}

#[test]
fn optional_extension_diagnostics_are_deterministic_capped_and_do_not_hide_gates() {
    let input = fixtures::mutate_json(fixtures::missing_uv(), |root| {
        root["extensionsUsed"] = serde_json::json!(["EXT_optional_a", "EXT_optional_b"]);
    });
    let full = inspect_glb(
        &input,
        &GlbLimits {
            max_diagnostics: 2,
            ..GlbLimits::default()
        },
    )
    .unwrap();
    assert_eq!(full.diagnostics.len(), 2);
    assert_eq!(
        full.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.json_path.as_deref())
            .collect::<Vec<_>>(),
        [Some("extensionsUsed[0]"), Some("extensionsUsed[1]")]
    );
    assert_gate(&full, "M2A-GLB-UV0-MISSING", "BLOCKING");
    assert!(!full.conversion_eligible);

    let capped = inspect_glb(
        &input,
        &GlbLimits {
            max_diagnostics: 1,
            ..GlbLimits::default()
        },
    )
    .unwrap();
    assert_eq!(capped.diagnostics.len(), 1);
    assert_gate(&capped, "M2A-GLB-UV0-MISSING", "BLOCKING");
    let none = inspect_glb(
        &input,
        &GlbLimits {
            max_diagnostics: 0,
            ..GlbLimits::default()
        },
    )
    .unwrap();
    assert!(none.diagnostics.is_empty());
    assert_gate(&none, "M2A-GLB-UV0-MISSING", "BLOCKING");
}

#[test]
fn every_serialized_geometry_float_domain_rejects_nonfinite_values() {
    for (accessor, scalar) in [(0, 0), (1, 0), (2, 0)] {
        for value in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            let input = fixtures::mutate_accessor_f32(
                fixtures::minimal_indexed_triangle(),
                accessor,
                scalar,
                value,
            );
            assert_eq!(
                inspect_glb(&input, &GlbLimits::default()).unwrap_err().code,
                "M2A-GLB-NONFINITE-FLOAT"
            );
        }
    }
    for value in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
        let input = fixtures::mutate_accessor_f32(
            fixtures::skin_animation_with_inverse_bind_matrices(),
            5,
            0,
            value,
        );
        assert_eq!(
            inspect_glb(&input, &GlbLimits::default()).unwrap_err().code,
            "M2A-GLB-NONFINITE-FLOAT"
        );
    }
}

fn set_declared_length(bytes: &mut [u8]) {
    let length = u32::try_from(bytes.len()).unwrap();
    bytes[8..12].copy_from_slice(&length.to_le_bytes());
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

    let fixtures_a_to_f = [
        fixtures::minimal_indexed_triangle(),
        fixtures::axis_hierarchy_asymmetric(),
        fixtures::uv_corners_and_out_of_range(),
        fixtures::material_image_two_primitives(),
        fixtures::skin_animation_with_inverse_bind_matrices(),
        fixtures::morph_target(),
    ];
    for (fixture_index, input) in fixtures_a_to_f.iter().enumerate() {
        let original = input.clone();
        inspect_glb(input, &limits).expect("complete A-F fixture inspect");
        ingest_glb(input, &limits).expect("complete A-F fixture ingest");
        assert_eq!(input, &original, "A-F fixture {fixture_index} was mutated");
        for length in 0..input.len() {
            let inspect_outcome = catch_unwind(AssertUnwindSafe(|| {
                inspect_glb(&input[..length], &limits).map(|_| ())
            }));
            let ingest_outcome = catch_unwind(AssertUnwindSafe(|| {
                ingest_glb(&input[..length], &limits).map(|_| ())
            }));
            for outcome in [inspect_outcome, ingest_outcome] {
                assert!(
                    outcome.is_ok(),
                    "panic for fixture {fixture_index} truncated prefix {length}"
                );
                assert!(
                    outcome.unwrap().is_err(),
                    "fixture {fixture_index} truncated prefix parsed at {length}"
                );
            }
        }
    }

    let mut state = 0x5eed_u32;
    for length in 0..=256 {
        let mut bytes = vec![0_u8; length];
        for byte in &mut bytes {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *byte = (state >> 24) as u8;
        }
        for outcome in [
            catch_unwind(AssertUnwindSafe(|| {
                inspect_glb(&bytes, &limits).map(|_| ())
            })),
            catch_unwind(AssertUnwindSafe(|| ingest_glb(&bytes, &limits).map(|_| ()))),
        ] {
            assert!(outcome.is_ok(), "panic for arbitrary input length {length}");
        }
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
