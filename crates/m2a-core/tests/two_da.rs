use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::two_da::{
    APPEND_U16_OVERFLOW, ASSIGNMENT_COLUMN_MISSING, ASSIGNMENT_DUPLICATE, COLUMN_AMBIGUOUS,
    COLUMN_INVALID, DEFAULT_INVALID, HEADER_INVALID, LIMIT_EXCEEDED, NEWLINE_INVALID,
    NUL_FORBIDDEN, QUOTE_INVALID, ROW_ARITY_INVALID, ROW_LABEL_INVALID, ROW_LABEL_MISMATCH,
    TAB_FORBIDDEN, TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
    TwoDaNewlineV1, VALUE_INVALID, append_two_da_row_v1, inspect_two_da_v2,
};

fn text(value: &str) -> TwoDaCellValueV1 {
    TwoDaCellValueV1::Text {
        value: value.to_owned(),
    }
}

fn request(cells: Vec<(&str, TwoDaCellValueV1)>) -> TwoDaAppendRequestV1 {
    TwoDaAppendRequestV1 {
        schema_version: 1,
        cells: cells
            .into_iter()
            .map(|(column_name, value)| TwoDaCellAssignmentV1 {
                column_name: column_name.to_owned(),
                value,
            })
            .collect(),
    }
}

#[test]
fn inspection_preserves_lexical_null_text_and_physical_row_identity() {
    let source = b"2DA V2.0\r\n\r\n  LABEL VALUE  \r\n0 \"A B\" ****\r\n0 C \"****\"\r\n";
    let report = inspect_two_da_v2(source, &TwoDaLimitsV1::default()).unwrap();

    assert_eq!(report.format, "2DA");
    assert_eq!(report.version, "V2.0");
    assert_eq!(report.newline, TwoDaNewlineV1::CrLf);
    assert!(report.terminal_newline);
    assert_eq!(report.columns, ["LABEL", "VALUE"]);
    assert_eq!(report.physical_row_count, 2);
    assert_eq!(report.next_append_index, Some(2));
    assert_eq!(report.row_label_mismatch_count, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, ROW_LABEL_MISMATCH);
    assert_eq!(report.diagnostics[0].severity, "WARNING");
}

#[test]
fn append_is_exact_prefix_plus_deterministic_source_eol_suffix() {
    let source = b"2DA V2.0\n\nLABEL VALUE OPTIONAL\n0 old **** keep";
    let first_request = request(vec![("value", text("A B")), ("LABEL", text("****"))]);
    let shuffled_request = request(vec![("label", text("****")), ("VALUE", text("A B"))]);

    let first = append_two_da_row_v1(source, &first_request, &TwoDaLimitsV1::default()).unwrap();
    let second =
        append_two_da_row_v1(source, &shuffled_request, &TwoDaLimitsV1::default()).unwrap();

    assert_eq!(first.payload, second.payload);
    assert_eq!(first.report, second.report);
    assert_eq!(&first.payload[..source.len()], source);
    assert_eq!(
        &first.payload[source.len()..],
        b"\n1 \"****\" \"A B\" ****\n"
    );
    assert_eq!(first.report.appended_row_index, 1);
    assert_eq!(first.report.physical_rows_before, 1);
    assert_eq!(first.report.physical_rows_after, 2);
    assert!(first.report.source_prefix_preserved);
    assert!(first.report.inserted_separator_newline);
    assert_eq!(first.report.changed_cells.len(), 2);
    assert_eq!(first.report.changed_cells[0].column_name, "LABEL");
    assert_eq!(first.report.changed_cells[1].column_name, "VALUE");
    assert_eq!(first.report.source_sha256.len(), 64);
    assert_eq!(first.report.output_sha256.len(), 64);
}

#[test]
fn node_boundary_append_fixture_has_frozen_native_length_and_hashes() {
    let source = b"2DA V2.0\n\nA B\n0 old ****\n";
    let artifact = append_two_da_row_v1(
        source,
        &request(vec![("A", text("new"))]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap();

    assert_eq!(
        artifact.payload,
        b"2DA V2.0\n\nA B\n0 old ****\n1 new ****\n"
    );
    assert_eq!(artifact.payload.len(), 36);
    assert_eq!(
        artifact.report.source_sha256,
        "13742e2d1fc92fdb18ac59689e03e601957c381ee48a20faddd84637c162ca24"
    );
    assert_eq!(
        artifact.report.output_sha256,
        "fed4b73584a864c1a5532b1dfea78f07a603116fa59ef4b6f5f70b84fc96cb67"
    );
}

#[test]
fn starred_row_is_not_a_hole_and_existing_bad_labels_are_only_warnings() {
    let source = b"2DA V2.0\r\n\r\nLABEL\r\n0 first\r\n0 ****\r\n2 third\r\n";
    let artifact = append_two_da_row_v1(
        source,
        &request(vec![("LABEL", text("fourth"))]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap();

    assert!(artifact.payload.ends_with(b"3 fourth\r\n"));
    assert_eq!(artifact.report.appended_row_index, 3);
    assert_eq!(artifact.report.diagnostics.len(), 1);
    assert_eq!(artifact.report.diagnostics[0].code, ROW_LABEL_MISMATCH);
}

#[test]
fn defaults_empty_text_and_quoted_stars_are_distinct() {
    let source = b"2DA V2.0\nDEFAULT: \"fallback text\"\nA B C\n";
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default()).unwrap();
    assert_eq!(inspection.default_value, Some(text("fallback text")));

    let artifact = append_two_da_row_v1(
        source,
        &request(vec![
            ("A", TwoDaCellValueV1::Null),
            ("B", text("")),
            ("C", text("****")),
        ]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap();
    assert!(artifact.payload.ends_with(b"0 **** \"\" \"****\"\n"));
}

#[test]
fn strict_grammar_has_stable_codes_and_never_panics() {
    let cases: &[(&[u8], &str)] = &[
        (b"", HEADER_INVALID),
        (b"2DA V1.0\n\nA\n", HEADER_INVALID),
        (b"2DA V2.0\nDEFAULT:   \nA\n", DEFAULT_INVALID),
        (b"2DA V2.0\r\n\nA\r\n", NEWLINE_INVALID),
        (b"2DA V2.0\n\nA\n0\tvalue\n", TAB_FORBIDDEN),
        (b"2DA V2.0\n\nA B\n0 one\n", ROW_ARITY_INVALID),
        (b"2DA V2.0\n\nA\nzero one\n", "M5-2DA-ROW-LABEL-INVALID"),
        (b"2DA V2.0\n\nA\n0 \"open\n", QUOTE_INVALID),
        (b"2DA V2.0\n\nA\n0 \"x\"tail\n", QUOTE_INVALID),
    ];
    for &(bytes, code) in cases {
        let result = catch_unwind(AssertUnwindSafe(|| {
            inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
                .map(|_| ())
                .map_err(|error| error.code)
        }));
        assert!(result.is_ok(), "parser panicked for {bytes:?}");
        assert_eq!(result.unwrap().unwrap_err(), code);
    }

    for length in 0..b"2DA V2.0\n\nA\n0 value\n".len() {
        let bytes = &b"2DA V2.0\n\nA\n0 value\n"[..length];
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default());
            }))
            .is_ok()
        );
    }
}

#[test]
fn default_token_error_uses_physical_line_column() {
    let source = b"2DA V2.0\nDEFAULT: \"x\"tail\nA\n";
    let error = inspect_two_da_v2(source, &TwoDaLimitsV1::default()).unwrap_err();

    assert_eq!(error.code, QUOTE_INVALID);
    assert_eq!(error.byte_offset, 21);
    assert_eq!(error.line, Some(2));
    assert_eq!(error.column, Some(13));
}

#[test]
fn dense_line_and_token_inputs_hit_configured_limits_without_panicking() {
    let line_limits = TwoDaLimitsV1 {
        max_rows: 1,
        ..TwoDaLimitsV1::default()
    };
    let mut dense_lines = b"2DA V2.0\n\nA\n0 retained\n".to_vec();
    dense_lines.resize(2 * 1024 * 1024, b'\n');
    let line_result = catch_unwind(AssertUnwindSafe(|| {
        inspect_two_da_v2(&dense_lines, &line_limits)
            .map(|_| ())
            .map_err(|error| (error.code, error.path))
    }));
    let line_error = line_result.expect("dense line scan panicked").unwrap_err();
    assert_eq!(line_error.0, LIMIT_EXCEEDED);
    assert_eq!(line_error.1, "limits.maxRows");

    dense_lines.push(0);
    let encoding_error = inspect_two_da_v2(&dense_lines, &line_limits).unwrap_err();
    assert_eq!(encoding_error.code, NUL_FORBIDDEN);

    let token_limits = TwoDaLimitsV1 {
        max_columns: 2,
        ..TwoDaLimitsV1::default()
    };
    let mut dense_columns = b"2DA V2.0\n\n".to_vec();
    dense_columns.extend(std::iter::repeat_n(b"A ".as_slice(), 512 * 1024).flatten());
    dense_columns.push(b'\n');
    let token_result = catch_unwind(AssertUnwindSafe(|| {
        inspect_two_da_v2(&dense_columns, &token_limits)
            .map(|_| ())
            .map_err(|error| (error.code, error.path))
    }));
    let token_error = token_result
        .expect("dense token parse panicked")
        .unwrap_err();
    assert_eq!(token_error.0, LIMIT_EXCEEDED);
    assert_eq!(token_error.1, "limits.maxColumns");
}

#[test]
fn row_validation_and_row_limit_precede_column_casefold_collision() {
    let limits = TwoDaLimitsV1 {
        max_rows: 1,
        ..TwoDaLimitsV1::default()
    };
    let cases: &[(&[u8], &str)] = &[
        (
            b"2DA V2.0\n\nBAD-NAME\nnot_a_label retained\n",
            COLUMN_INVALID,
        ),
        (
            b"2DA V2.0\n\nName NAME\n0 only_one_cell\n",
            ROW_ARITY_INVALID,
        ),
        (
            b"2DA V2.0\n\nName NAME\nnot_a_label x y\n",
            ROW_LABEL_INVALID,
        ),
        (
            b"2DA V2.0\n\nName NAME\n0 x y\n1 later row\n",
            LIMIT_EXCEEDED,
        ),
    ];

    for &(source, expected_code) in cases {
        let error = inspect_two_da_v2(source, &limits).unwrap_err();
        assert_eq!(
            error.code, expected_code,
            "unexpected precedence for {source:?}"
        );
    }
}

#[test]
fn deferred_row_limit_waits_for_every_retained_row_validation() {
    let limits = TwoDaLimitsV1 {
        max_rows: 2,
        ..TwoDaLimitsV1::default()
    };
    let source = b"2DA V2.0\n\nA\n0 valid\nnot_a_label retained\n2 discarded\n";

    let error = inspect_two_da_v2(source, &limits).unwrap_err();
    assert_eq!(error.code, ROW_LABEL_INVALID);
    assert_eq!(error.path, "rows[1].label");
}

#[test]
fn deterministic_arbitrary_byte_corpus_never_panics() {
    let request = request(Vec::new());
    let mut state = 0x6d_32_61_32_u32;
    for case_index in 0..512usize {
        let length = case_index % 257;
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            bytes.push((state >> 24) as u8);
        }

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _ = inspect_two_da_v2(&bytes, &TwoDaLimitsV1::default());
            let _ = append_two_da_row_v1(&bytes, &request, &TwoDaLimitsV1::default());
        }));
        assert!(
            result.is_ok(),
            "parser panicked for corpus case {case_index}"
        );
    }
}

#[test]
fn append_rejects_ambiguous_missing_duplicate_and_invalid_values() {
    let ambiguous = b"2DA V2.0\n\nName NAME\n";
    assert_eq!(
        append_two_da_row_v1(
            ambiguous,
            &request(vec![("name", text("x"))]),
            &TwoDaLimitsV1::default(),
        )
        .unwrap_err()
        .code,
        COLUMN_AMBIGUOUS
    );

    let source = b"2DA V2.0\n\nA\n";
    assert_eq!(
        append_two_da_row_v1(
            source,
            &request(vec![("missing", text("x"))]),
            &TwoDaLimitsV1::default(),
        )
        .unwrap_err()
        .code,
        ASSIGNMENT_COLUMN_MISSING
    );
    assert_eq!(
        append_two_da_row_v1(
            source,
            &request(vec![("A", text("x")), ("a", text("y"))]),
            &TwoDaLimitsV1::default(),
        )
        .unwrap_err()
        .code,
        ASSIGNMENT_DUPLICATE
    );
    assert_eq!(
        append_two_da_row_v1(
            source,
            &request(vec![("A", text("bad\"quote"))]),
            &TwoDaLimitsV1::default(),
        )
        .unwrap_err()
        .code,
        VALUE_INVALID
    );
}

fn table_with_rows(count: usize) -> Vec<u8> {
    let mut bytes = b"2DA V2.0\n\nA\n".to_vec();
    for index in 0..count {
        bytes.extend_from_slice(index.to_string().as_bytes());
        bytes.extend_from_slice(b" value\n");
    }
    bytes
}

#[test]
fn physical_append_index_has_exact_u16_boundary() {
    let success_source = table_with_rows(usize::from(u16::MAX));
    let success = append_two_da_row_v1(
        &success_source,
        &request(vec![("A", text("last"))]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap();
    assert_eq!(success.report.appended_row_index, u16::MAX);
    assert!(success.payload.ends_with(b"65535 last\n"));

    let overflow_source = table_with_rows(usize::from(u16::MAX) + 1);
    let error = append_two_da_row_v1(
        &overflow_source,
        &request(vec![("A", text("overflow"))]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, APPEND_U16_OVERFLOW);
}

#[test]
fn local_15219_row_label_gap_pattern_appends_at_physical_index() {
    let mut source = b"2DA V2.0\n\nA\n".to_vec();
    for physical_index in 0..15_219u32 {
        let printed_label = if physical_index == 15_153 {
            15_152
        } else {
            physical_index
        };
        source.extend_from_slice(printed_label.to_string().as_bytes());
        source.extend_from_slice(b" value\n");
    }

    let inspection = inspect_two_da_v2(&source, &TwoDaLimitsV1::default()).unwrap();
    assert_eq!(inspection.physical_row_count, 15_219);
    assert_eq!(inspection.row_label_mismatch_count, 1);
    assert_eq!(inspection.diagnostics[0].path, "rows[15153].label");

    let artifact = append_two_da_row_v1(
        &source,
        &request(vec![("A", text("appended"))]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap();
    assert_eq!(artifact.report.appended_row_index, 15_219);
    assert_eq!(&artifact.payload[..source.len()], source);
    assert!(artifact.payload.ends_with(b"15219 appended\n"));
}

#[test]
fn owned_35_column_append_artifact_is_full_width_and_readable() {
    let columns: Vec<String> = (0..35).map(|index| format!("COL_{index:02}")).collect();
    let source = format!("2DA V2.0\n\n{}\n", columns.join(" ")).into_bytes();
    let artifact = append_two_da_row_v1(
        &source,
        &request(vec![
            ("COL_00", text("15219")),
            ("col_34", text("mesh_resref")),
        ]),
        &TwoDaLimitsV1::default(),
    )
    .unwrap();

    assert_eq!(&artifact.payload[..source.len()], source);
    let appended_row = std::str::from_utf8(&artifact.payload[source.len()..])
        .unwrap()
        .trim_end();
    let tokens: Vec<&str> = appended_row.split(' ').collect();
    assert_eq!(tokens.len(), 36);
    assert_eq!(tokens[0], "0");
    assert_eq!(tokens[1], "15219");
    assert!(tokens[2..35].iter().all(|token| *token == "****"));
    assert_eq!(tokens[35], "mesh_resref");

    let readback = inspect_two_da_v2(&artifact.payload, &TwoDaLimitsV1::default()).unwrap();
    assert_eq!(readback.columns, columns);
    assert_eq!(readback.physical_row_count, 1);
    assert_eq!(artifact.report.output_sha256, readback.source_sha256);
}

#[test]
fn request_json_is_strict_and_tagged_cells_are_unambiguous() {
    let request = request(vec![("A", TwoDaCellValueV1::Null), ("B", text("****"))]);
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["cells"][0]["value"]["kind"], "NULL");
    assert_eq!(json["cells"][1]["value"]["kind"], "TEXT");
    assert!(serde_json::from_value::<TwoDaAppendRequestV1>(json).is_ok());
    assert!(
        serde_json::from_str::<TwoDaAppendRequestV1>(
            r#"{"schemaVersion":1,"cells":[],"unknown":true}"#
        )
        .is_err()
    );
}
