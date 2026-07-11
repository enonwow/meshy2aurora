use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::{ParserLimits, inspect_binary_mdl, inspect_binary_mdl_with_limits};

use crate::fixtures::{
    FILE_HEADER_SIZE, ROOT_NODE_ABSOLUTE, build_deep_binary_mdl, build_minimal_binary_mdl,
    build_skin_binary_mdl, build_two_node_binary_mdl, make_animation_declared_too_small,
    make_animation_root_cycle, make_root_cycle, write_i16, write_i32, write_u32,
};

#[test]
fn parses_deep_model_controllers_trimesh_and_all_animation_roots() {
    let report = inspect_binary_mdl(&build_deep_binary_mdl()).expect("deep fixture must parse");
    assert_eq!(report.model.name, "m2a_deep");
    assert_eq!(report.model.classification, 4);
    assert_eq!(report.model.fog, 1);
    assert_eq!(report.model.bounds_min.x, -1.0);
    assert_eq!(report.model.bounds_max.z, 3.0);
    assert_eq!(report.model.radius, 3.5);
    assert_eq!(report.model.animation_scale, 1.25);
    assert_eq!(report.model.supermodel_name, "null");
    assert_eq!(report.node_tree.declared_node_count, 2);
    assert_eq!(report.node_tree.node_count, 1);

    let root = &report.node_tree.roots[0];
    let names: Vec<_> = root
        .controllers
        .iter()
        .map(|controller| controller.controller_name.as_deref())
        .collect();
    assert_eq!(
        names,
        [
            Some("position"),
            Some("orientation"),
            Some("scale"),
            Some("selfIllumination"),
            Some("alpha")
        ]
    );
    let mesh = root.mesh.as_ref().expect("trimesh report");
    assert_eq!(mesh.vertex_count, 3);
    assert_eq!(mesh.faces.len(), 1);
    assert_eq!(mesh.faces[0].vertex_indices, [0, 1, 2]);
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.uv0.len(), 3);
    assert_eq!(mesh.normals.len(), 3);
    assert_eq!(mesh.textures[0], "m2a_diffuse");

    assert_eq!(report.animations.len(), 2);
    assert_eq!(report.animations[0].name, "walk");
    assert_eq!(report.animations[0].events[0].name, "footstep");
    assert_eq!(report.animations[0].node_tree.declared_node_count, 2);
    assert_eq!(report.animations[0].node_tree.node_count, 1);
    assert_eq!(
        report.animations[0].node_tree.roots[0].controllers[0]
            .controller_name
            .as_deref(),
        Some("position")
    );
    assert_eq!(report.animations[1].name, "idle");
    assert_eq!(report.animations[1].events[0].name, "loop");
}

#[test]
fn parses_both_explicit_skin_variants_without_using_map_count_as_classifier() {
    let legacy = inspect_binary_mdl(&build_skin_binary_mdl(false)).expect("legacy17 skin");
    let extended = inspect_binary_mdl(&build_skin_binary_mdl(true)).expect("extended64 skin");
    let legacy = legacy.node_tree.roots[0].skin.as_ref().unwrap();
    let extended = extended.node_tree.roots[0].skin.as_ref().unwrap();
    assert_eq!(format!("{:?}", legacy.variant), "Legacy17");
    assert_eq!(format!("{:?}", extended.variant), "Extended64");
    assert_eq!(legacy.node_to_bone_map.len(), 3);
    assert_eq!(extended.node_to_bone_map.len(), 3);
    assert_eq!(legacy.inline_mapping.len(), 17);
    assert_eq!(extended.inline_mapping.len(), 64);
    assert_eq!(legacy.vertex_weights.len(), 3);
    assert_eq!(extended.bone_references.len(), 3);
    assert_eq!(legacy.inverse_bone_rotations_raw[0], [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(extended.bone_constants[0], [7, 8]);
}

#[test]
fn rejects_ambiguous_skin_variant_raw_oob_and_bone_ref_oob() {
    let mut ambiguous = build_skin_binary_mdl(false);
    write_i32(&mut ambiguous, ROOT_NODE_ABSOLUTE + 0x284, 0);
    assert_eq!(
        inspect_binary_mdl(&ambiguous).unwrap_err().code,
        "M2A-MDL-SKIN-VARIANT-AMBIGUOUS"
    );

    let mut raw_oob = build_skin_binary_mdl(false);
    write_i32(&mut raw_oob, ROOT_NODE_ABSOLUTE + 0x27c, i32::MAX);
    assert_eq!(
        inspect_binary_mdl(&raw_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    let mut bone_oob = build_skin_binary_mdl(false);
    let core_length = u32::from_le_bytes(bone_oob[4..8].try_into().unwrap()) as usize;
    write_i16(&mut bone_oob, FILE_HEADER_SIZE + core_length + 144, 17);
    assert_eq!(
        inspect_binary_mdl(&bone_oob).unwrap_err().code,
        "M2A-MDL-BONE-REF-OOB"
    );
}

#[test]
fn deep_arrays_and_controller_indices_have_stable_failures() {
    let mut used_over_allocated = build_deep_binary_mdl();
    write_u32(&mut used_over_allocated, ROOT_NODE_ABSOLUTE + 0x7c, 2);
    assert_eq!(
        inspect_binary_mdl(&used_over_allocated).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );

    let mut raw_oob = build_deep_binary_mdl();
    write_i32(&mut raw_oob, ROOT_NODE_ABSOLUTE + 0x22c, i32::MAX);
    assert_eq!(
        inspect_binary_mdl(&raw_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    let mut index_oob = build_deep_binary_mdl();
    let key_pointer = u32::from_le_bytes(
        index_oob[ROOT_NODE_ABSOLUTE + 0x54..ROOT_NODE_ABSOLUTE + 0x58]
            .try_into()
            .unwrap(),
    ) as usize;
    write_i16(&mut index_oob, FILE_HEADER_SIZE + key_pointer + 6, i16::MAX);
    assert_eq!(
        inspect_binary_mdl(&index_oob).unwrap_err().code,
        "M2A-MDL-CONTROLLER-INDEX-OOB"
    );
}

#[test]
fn unknown_controller_and_node_families_keep_inventory_with_stable_diagnostics() {
    let mut unknown_controller = build_deep_binary_mdl();
    let key_pointer = u32::from_le_bytes(
        unknown_controller[ROOT_NODE_ABSOLUTE + 0x54..ROOT_NODE_ABSOLUTE + 0x58]
            .try_into()
            .unwrap(),
    ) as usize;
    write_i32(&mut unknown_controller, FILE_HEADER_SIZE + key_pointer, 999);
    let report = inspect_binary_mdl(&unknown_controller).unwrap();
    assert_eq!(
        report.node_tree.roots[0].controllers[0].controller_type,
        999
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "M2A-MDL-CONTROLLER-TYPE-UNKNOWN")
    );

    let mut unknown_families = build_minimal_binary_mdl();
    write_u32(
        &mut unknown_families,
        ROOT_NODE_ABSOLUTE + 0x6c,
        0x001 | 0x002 | 0x400,
    );
    let report = inspect_binary_mdl(&unknown_families).unwrap();
    assert!(
        report.node_tree.roots[0]
            .unsupported_families
            .contains(&"light".to_owned())
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "M2A-MDL-UNSUPPORTED-NODE-FAMILY" })
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "M2A-MDL-NODE-FLAGS-UNKNOWN" })
    );
}

#[test]
fn every_deep_fixture_truncation_returns_without_panicking() {
    for complete in [
        build_deep_binary_mdl(),
        build_skin_binary_mdl(false),
        build_skin_binary_mdl(true),
    ] {
        for length in 0..complete.len() {
            let outcome =
                catch_unwind(AssertUnwindSafe(|| inspect_binary_mdl(&complete[..length])));
            assert!(outcome.is_ok(), "deep parser panicked for length {length}");
            assert!(outcome.unwrap().is_err(), "truncated deep fixture parsed");
        }
    }
}

#[test]
fn common_controller_layouts_require_exact_signed_columns() {
    let bytes = build_deep_binary_mdl();
    let keys = read_u32_at(&bytes, ROOT_NODE_ABSOLUTE + 0x54) as usize;
    for (index, invalid_columns) in [(0, 1_i8), (1, 3), (2, 2), (3, 2), (4, 2)] {
        let mut invalid = bytes.clone();
        invalid[FILE_HEADER_SIZE + keys + index * 0x0c + 10] = invalid_columns as u8;
        assert_eq!(
            inspect_binary_mdl(&invalid).unwrap_err().code,
            "M2A-MDL-CONTROLLER-LAYOUT-INVALID",
            "controller key {index}"
        );
    }

    let mut negative_rows = bytes.clone();
    write_i16(&mut negative_rows, FILE_HEADER_SIZE + keys + 4, -1);
    assert_eq!(
        inspect_binary_mdl(&negative_rows).unwrap_err().code,
        "M2A-MDL-CONTROLLER-LAYOUT-INVALID"
    );

    let mut negative_columns = bytes;
    negative_columns[FILE_HEADER_SIZE + keys + 10] = (-1_i8) as u8;
    assert_eq!(
        inspect_binary_mdl(&negative_columns).unwrap_err().code,
        "M2A-MDL-CONTROLLER-LAYOUT-INVALID"
    );
}

#[test]
fn skin_profile_limits_map_bind_counts_and_preserves_ffff_bone_refs() {
    let mut map_count = build_skin_binary_mdl(false);
    write_i32(&mut map_count, ROOT_NODE_ABSOLUTE + 0x288, 18);
    assert_eq!(
        inspect_binary_mdl(&map_count).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );

    for header in [0x28c, 0x298, 0x2a4] {
        let mut bind_count = build_skin_binary_mdl(false);
        write_u32(&mut bind_count, ROOT_NODE_ABSOLUTE + header + 4, 18);
        write_u32(&mut bind_count, ROOT_NODE_ABSOLUTE + header + 8, 18);
        assert_eq!(
            inspect_binary_mdl(&bind_count).unwrap_err().code,
            "M2A-MDL-HEADER-INVALID",
            "skin bind header 0x{header:x}"
        );
    }

    let mut sentinel = build_skin_binary_mdl(false);
    let raw = FILE_HEADER_SIZE + read_u32_at(&sentinel, 4) as usize;
    write_i16(&mut sentinel, raw + 144, -1);
    let report = inspect_binary_mdl(&sentinel).expect("0xffff bone ref remains open/preserved");
    assert_eq!(
        report.node_tree.roots[0]
            .skin
            .as_ref()
            .unwrap()
            .bone_references[0][0],
        u16::MAX
    );
}

#[test]
fn extended64_bone_reference_accepts_63_and_rejects_64() {
    let mut boundary = build_skin_binary_mdl(true);
    let raw = FILE_HEADER_SIZE + read_u32_at(&boundary, 4) as usize;
    write_i16(&mut boundary, raw + 144, 63);
    inspect_binary_mdl(&boundary).expect("extended64 bone 63");
    write_i16(&mut boundary, raw + 144, 64);
    assert_eq!(
        inspect_binary_mdl(&boundary).unwrap_err().code,
        "M2A-MDL-BONE-REF-OOB"
    );
}

#[test]
fn additive_trailing_families_keep_common_mesh_prefix_and_only_report_unsupported() {
    for deferred in [0x080_u32, 0x100, 0x200] {
        let mut bytes = build_deep_binary_mdl();
        write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x6c, 0x021 | deferred);
        let report = inspect_binary_mdl(&bytes).expect("trailing family keeps mesh prefix");
        let root = &report.node_tree.roots[0];
        assert_eq!(root.mesh.as_ref().unwrap().faces.len(), 1);
        assert!(!root.unsupported_families.contains(&"header".to_owned()));
        assert!(!root.unsupported_families.contains(&"mesh".to_owned()));
        assert_eq!(root.unsupported_families.len(), 1);
    }
}

#[test]
fn face_adjacency_is_signed() {
    let report = inspect_binary_mdl(&build_deep_binary_mdl()).unwrap();
    assert_eq!(
        report.node_tree.roots[0].mesh.as_ref().unwrap().faces[0].adjacent_faces,
        [-1, -1, -1]
    );
}

#[test]
fn partial_core_range_overlap_is_rejected_but_node_mesh_nesting_is_allowed() {
    inspect_binary_mdl(&build_deep_binary_mdl()).expect("intentional node/mesh nesting");
    let mut overlap = build_deep_binary_mdl();
    let faces = read_u32_at(&overlap, ROOT_NODE_ABSOLUTE + 0x78);
    write_u32(&mut overlap, ROOT_NODE_ABSOLUTE + 0x60, faces + 4);
    assert_eq!(
        inspect_binary_mdl(&overlap).unwrap_err().code,
        "M2A-MDL-OFFSET-TYPE-CONFLICT"
    );
}

#[test]
fn animation_cycle_and_declared_budgets_are_enforced() {
    let mut cycle = build_deep_binary_mdl();
    make_animation_root_cycle(&mut cycle);
    assert_eq!(
        inspect_binary_mdl(&cycle).unwrap_err().code,
        "M2A-MDL-NODE-CYCLE"
    );

    let mut base_small = build_two_node_binary_mdl();
    write_u32(&mut base_small, FILE_HEADER_SIZE + 0x4c, 1);
    assert_eq!(
        inspect_binary_mdl(&base_small).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );

    let mut animation_small = build_deep_binary_mdl();
    make_animation_declared_too_small(&mut animation_small);
    assert_eq!(
        inspect_binary_mdl(&animation_small).unwrap_err().code,
        "M2A-MDL-HEADER-INVALID"
    );
}

#[test]
fn animation_core_pointer_matrix_is_bounds_checked() {
    let original = build_deep_binary_mdl();
    let pointer_list = read_u32_at(&original, FILE_HEADER_SIZE + 0x78) as usize;
    let animation = read_u32_at(&original, FILE_HEADER_SIZE + pointer_list) as usize;

    let mut list_oob = original.clone();
    write_u32(&mut list_oob, FILE_HEADER_SIZE + 0x78, u32::MAX);
    assert_eq!(
        inspect_binary_mdl(&list_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    let mut header_oob = original.clone();
    write_u32(&mut header_oob, FILE_HEADER_SIZE + pointer_list, u32::MAX);
    assert_eq!(
        inspect_binary_mdl(&header_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    let mut events_oob = original;
    write_u32(
        &mut events_oob,
        FILE_HEADER_SIZE + animation + 0xb8,
        u32::MAX,
    );
    assert_eq!(
        inspect_binary_mdl(&events_oob).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );
}

#[test]
fn controller_skin_and_mesh_pointer_matrices_are_bounds_checked() {
    let mut controller_data = build_deep_binary_mdl();
    write_u32(&mut controller_data, ROOT_NODE_ABSOLUTE + 0x60, u32::MAX);
    assert_eq!(
        inspect_binary_mdl(&controller_data).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );

    for field in [0x28c, 0x298, 0x2a4] {
        let mut bind = build_skin_binary_mdl(false);
        write_u32(&mut bind, ROOT_NODE_ABSOLUTE + field, u32::MAX);
        assert_eq!(
            inspect_binary_mdl(&bind).unwrap_err().code,
            "M2A-MDL-POINTER-OOB",
            "skin bind field 0x{field:x}"
        );
    }

    for field in [0x234, 0x244, 0x238, 0x23c, 0x240, 0x248, 0x24c, 0x260] {
        let mut raw = build_deep_binary_mdl();
        write_i32(&mut raw, ROOT_NODE_ABSOLUTE + field, i32::MAX);
        assert_eq!(
            inspect_binary_mdl(&raw).unwrap_err().code,
            "M2A-MDL-POINTER-OOB",
            "mesh raw field 0x{field:x}"
        );
    }

    let mut bone_refs = build_skin_binary_mdl(false);
    write_i32(&mut bone_refs, ROOT_NODE_ABSOLUTE + 0x280, i32::MAX);
    assert_eq!(
        inspect_binary_mdl(&bone_refs).unwrap_err().code,
        "M2A-MDL-POINTER-OOB"
    );
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[test]
fn parses_minimal_header_and_root_node() {
    let bytes = build_minimal_binary_mdl();
    let report = inspect_binary_mdl(&bytes).expect("minimal fixture must parse");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.format, "nwn1-binary-mdl");
    assert_eq!(report.byte_length, bytes.len());
    assert_eq!(report.file_header.binary_mdl_id, 0);
    assert!(report.file_header.mdx_range_in_bounds);
    assert_eq!(report.model.name, "m2a_minimal");
    assert_eq!(report.node_tree.node_count, 1);
    assert_eq!(report.node_tree.max_depth, 0);
    assert_eq!(report.node_tree.roots[0].name, "root");
    assert_eq!(report.node_tree.roots[0].parent_offset, None);
}

#[test]
fn declared_node_count_is_a_budget_not_reachable_tree_equality() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, FILE_HEADER_SIZE + 0x4c, 2);

    let report = inspect_binary_mdl(&bytes).expect("unreachable runtime nodes may be declared");
    assert_eq!(report.node_tree.declared_node_count, 2);
    assert_eq!(report.node_tree.node_count, 1);
}

#[test]
fn deterministic_json_is_byte_identical() {
    let bytes = build_minimal_binary_mdl();
    let first = serde_json::to_string(&inspect_binary_mdl(&bytes).unwrap()).unwrap();
    let second = serde_json::to_string(&inspect_binary_mdl(&bytes).unwrap()).unwrap();

    assert_eq!(first, second);
    assert!(first.contains(r#""schemaVersion":1"#));
    assert!(first.contains(r#""byteLength":"#));
}

#[test]
fn rejects_nonzero_binary_mdl_id() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, 0, 1);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");
}

#[test]
fn empty_and_short_headers_use_stable_header_error() {
    for bytes in [&[][..], &[0_u8; 11][..]] {
        let error = inspect_binary_mdl(bytes).unwrap_err();
        assert_eq!(error.code, "M2A-MDL-HEADER-INVALID");
    }
}

#[test]
fn rejects_mdx_range_outside_input() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, 8, 1);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
}

#[test]
fn rejects_root_pointer_outside_input() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, FILE_HEADER_SIZE + 0x48, u32::MAX);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
}

#[test]
fn rejects_root_pointer_into_mdx_even_when_payload_exists() {
    let mut bytes = build_minimal_binary_mdl();
    let core_length = bytes.len() - FILE_HEADER_SIZE;
    bytes.extend(std::iter::repeat_n(0_u8, 0x70));
    write_u32(&mut bytes, 8, 0x70);
    write_u32(&mut bytes, FILE_HEADER_SIZE + 0x48, core_length as u32);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
}

#[test]
fn rejects_root_overlapping_model_header() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, FILE_HEADER_SIZE + 0x48, 0x70);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
}

#[test]
fn rejects_child_overlapping_model_header() {
    let mut bytes = build_two_node_binary_mdl();
    let child_pointer_entry = bytes.len() - 4;
    write_u32(&mut bytes, child_pointer_entry, 0x70);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
}

#[test]
fn rejects_child_array_outside_input_before_allocation() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x48, u32::MAX);
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x4c, 1);
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x50, 1);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
}

#[test]
fn validates_controller_pointers_without_parsing_payloads() {
    for field in [0x54, 0x60] {
        let mut bytes = build_minimal_binary_mdl();
        write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + field, u32::MAX);
        write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + field + 4, 1);
        write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + field + 8, 1);

        let error = inspect_binary_mdl(&bytes).unwrap_err();
        assert_eq!(error.code, "M2A-MDL-POINTER-OOB", "field 0x{field:x}");
    }
}

#[test]
fn runtime_geometry_and_parent_values_do_not_define_file_tree_links() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x40, 0x7a11_ce55);
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x44, 7_733_349);

    let report = inspect_binary_mdl(&bytes).expect("runtime values are not file pointers");
    assert_eq!(report.node_tree.roots[0].parent_offset, None);
}

#[test]
fn rejects_unbounded_children_allocated_count() {
    let mut bytes = build_minimal_binary_mdl();
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x50, u32::MAX);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-LIMIT-EXCEEDED");
}

#[test]
fn rejects_node_cycle() {
    let mut bytes = build_minimal_binary_mdl();
    make_root_cycle(&mut bytes);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-NODE-CYCLE");
}

#[test]
fn every_truncation_returns_without_panicking() {
    let complete = build_minimal_binary_mdl();
    for length in 0..complete.len() {
        let outcome = catch_unwind(AssertUnwindSafe(|| inspect_binary_mdl(&complete[..length])));
        assert!(outcome.is_ok(), "parser panicked for length {length}");
        assert!(
            outcome.unwrap().is_err(),
            "truncated length {length} parsed"
        );
    }
}

#[test]
fn input_node_depth_and_diagnostic_limits_are_enforced() {
    let minimal = build_minimal_binary_mdl();

    let input_error = inspect_binary_mdl_with_limits(
        &minimal,
        &ParserLimits {
            max_input_bytes: minimal.len() - 1,
            ..ParserLimits::default()
        },
    )
    .unwrap_err();
    assert_eq!(input_error.code, "M2A-LIMIT-EXCEEDED");

    let node_error = inspect_binary_mdl_with_limits(
        &minimal,
        &ParserLimits {
            max_nodes: 0,
            ..ParserLimits::default()
        },
    )
    .unwrap_err();
    assert_eq!(node_error.code, "M2A-LIMIT-EXCEEDED");

    let two_nodes = build_two_node_binary_mdl();
    let depth_error = inspect_binary_mdl_with_limits(
        &two_nodes,
        &ParserLimits {
            max_depth: 0,
            ..ParserLimits::default()
        },
    )
    .unwrap_err();
    assert_eq!(depth_error.code, "M2A-LIMIT-EXCEEDED");

    let mut deferred = minimal.clone();
    write_u32(&mut deferred, ROOT_NODE_ABSOLUTE + 0x6c, 0x003);
    let diagnostic_error = inspect_binary_mdl_with_limits(
        &deferred,
        &ParserLimits {
            max_diagnostics: 0,
            ..ParserLimits::default()
        },
    )
    .unwrap_err();
    assert_eq!(diagnostic_error.code, "M2A-LIMIT-EXCEEDED");
}

#[test]
fn parser_limits_accept_values_exactly_on_the_boundary() {
    let minimal = build_minimal_binary_mdl();
    let report = inspect_binary_mdl_with_limits(
        &minimal,
        &ParserLimits {
            max_input_bytes: minimal.len(),
            max_nodes: 1,
            max_depth: 0,
            max_diagnostics: 0,
        },
    )
    .expect("exact guardrail boundary must be accepted");
    assert_eq!(report.node_tree.node_count, 1);
    assert!(report.diagnostics.is_empty());

    let mut deferred = minimal;
    write_u32(&mut deferred, ROOT_NODE_ABSOLUTE + 0x6c, 0x003);
    let report = inspect_binary_mdl_with_limits(
        &deferred,
        &ParserLimits {
            max_input_bytes: deferred.len(),
            max_nodes: 1,
            max_depth: 0,
            max_diagnostics: 1,
        },
    )
    .expect("one diagnostic at a limit of one must be accepted");
    assert_eq!(report.diagnostics.len(), 1);
}

#[test]
fn input_is_not_mutated() {
    let bytes = build_minimal_binary_mdl();
    let original = bytes.clone();
    inspect_binary_mdl(&bytes).unwrap();
    assert_eq!(bytes, original);
}
