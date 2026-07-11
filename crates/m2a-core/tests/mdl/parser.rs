use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::{ParserLimits, inspect_binary_mdl, inspect_binary_mdl_with_limits};

use crate::fixtures::{
    FILE_HEADER_SIZE, ROOT_NODE_ABSOLUTE, build_minimal_binary_mdl, build_two_node_binary_mdl,
    make_root_cycle, write_u32,
};

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
fn validates_geometry_and_controller_pointers_without_parsing_payloads() {
    for field in [0x40, 0x54, 0x60] {
        let mut bytes = build_minimal_binary_mdl();
        write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + field, u32::MAX);
        if field != 0x40 {
            write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + field + 4, 1);
            write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + field + 8, 1);
        }

        let error = inspect_binary_mdl(&bytes).unwrap_err();
        assert_eq!(error.code, "M2A-MDL-POINTER-OOB", "field 0x{field:x}");
    }
}

#[test]
fn geometry_pointer_requires_a_complete_geometry_header() {
    let mut bytes = build_minimal_binary_mdl();
    let last_core_byte = (bytes.len() - FILE_HEADER_SIZE - 1) as u32;
    write_u32(&mut bytes, ROOT_NODE_ABSOLUTE + 0x40, last_core_byte);

    let error = inspect_binary_mdl(&bytes).unwrap_err();
    assert_eq!(error.code, "M2A-MDL-POINTER-OOB");
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
