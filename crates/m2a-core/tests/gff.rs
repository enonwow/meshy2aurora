use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::gff::{
    GFF_SCHEMA_VERSION, GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLimitsV1, GffLocStringV1,
    GffLocSubstringV1, GffStructV1, GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
};

fn field(label: &str, value: GffValueV1) -> GffFieldV1 {
    GffFieldV1 {
        label: label.to_owned(),
        value,
    }
}

fn document(fields: Vec<GffFieldV1>) -> GffDocumentV1 {
    GffDocumentV1 {
        schema_version: GFF_SCHEMA_VERSION,
        file_type: GffFileTypeV1::Utc,
        root: GffStructV1 {
            struct_id: u32::MAX,
            fields,
        },
    }
}

#[test]
fn empty_root_has_exact_contiguous_layout_and_frozen_bytes() {
    let artifact = write_gff_v32(&document(vec![]), &GffWriterOptionsV1::default()).unwrap();
    let expected = [
        b'U', b'T', b'C', b' ', b'V', b'3', b'.', b'2', 56, 0, 0, 0, 1, 0, 0, 0, 68, 0, 0, 0, 0, 0,
        0, 0, 68, 0, 0, 0, 0, 0, 0, 0, 68, 0, 0, 0, 0, 0, 0, 0, 68, 0, 0, 0, 0, 0, 0, 0, 68, 0, 0,
        0, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    assert_eq!(artifact.payload, expected);
    assert_eq!(artifact.report.struct_count, 1);
    assert_eq!(artifact.report.field_count, 0);
    assert_eq!(artifact.report.label_count, 0);
    assert_eq!(artifact.report.byte_length, 68);
    assert_eq!(artifact.report.max_depth, 0);
    assert_eq!(artifact.report.semantic_readback_status, "PASS");
    assert_eq!(
        serde_json::to_string(&artifact.report).unwrap(),
        r#"{"schemaVersion":1,"fileType":"UTC ","structOffset":56,"structCount":1,"fieldOffset":68,"fieldCount":0,"labelOffset":68,"labelCount":0,"fieldDataOffset":68,"fieldDataCount":0,"fieldIndicesOffset":68,"fieldIndicesCount":0,"listIndicesOffset":68,"listIndicesCount":0,"byteLength":68,"outputSha256":"ac60860a9baae04bfc9948aff6cd1e062359200633848671bb30d8a0d3273fae","maxDepth":0,"semanticReadbackStatus":"PASS"}"#
    );
    assert_eq!(
        read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap(),
        document(vec![])
    );
}

#[test]
fn one_inline_field_has_exact_offsets_label_and_direct_field_index() {
    let artifact = write_gff_v32(
        &document(vec![field("Value", GffValueV1::Dword(0x1234_5678))]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    assert_eq!(
        u32::from_le_bytes(artifact.payload[8..12].try_into().unwrap()),
        56
    );
    assert_eq!(
        u32::from_le_bytes(artifact.payload[16..20].try_into().unwrap()),
        68
    );
    assert_eq!(
        u32::from_le_bytes(artifact.payload[24..28].try_into().unwrap()),
        80
    );
    assert_eq!(
        u32::from_le_bytes(artifact.payload[32..36].try_into().unwrap()),
        96
    );
    assert_eq!(artifact.payload.len(), 96);
    assert_eq!(&artifact.payload[80..85], b"Value");
    assert_eq!(
        u32::from_le_bytes(artifact.payload[60..64].try_into().unwrap()),
        0
    );
    assert_eq!(
        u32::from_le_bytes(artifact.payload[68..72].try_into().unwrap()),
        4
    );
    assert_eq!(
        u32::from_le_bytes(artifact.payload[76..80].try_into().unwrap()),
        0x1234_5678
    );
    assert_eq!(
        artifact.report.output_sha256,
        "954af919e592c1abc0a92edef52a0c2855c8940c48199db3c0bd01a62601e5f1"
    );
}

#[test]
fn all_sixteen_types_roundtrip_exact_typed_tree() {
    let nested = GffStructV1 {
        struct_id: 77,
        fields: vec![field("Nested", GffValueV1::Byte(9))],
    };
    let listed = GffStructV1 {
        struct_id: 88,
        fields: vec![field("Listed", GffValueV1::Int(-4))],
    };
    let input = document(vec![
        field("Byte", GffValueV1::Byte(255)),
        field("Char", GffValueV1::Char(-128)),
        field("Word", GffValueV1::Word(65535)),
        field("Short", GffValueV1::Short(-32768)),
        field("Dword", GffValueV1::Dword(u32::MAX)),
        field("Int", GffValueV1::Int(i32::MIN)),
        field("Dword64", GffValueV1::Dword64(u64::MAX)),
        field("Int64", GffValueV1::Int64(i64::MIN)),
        field("Float", GffValueV1::Float(-12.5)),
        field("Double", GffValueV1::Double(0.25)),
        field("String", GffValueV1::String(vec![0x41, 0xff])),
        field("ResRef", GffValueV1::ResRef("abc_123".to_owned())),
        field(
            "LocString",
            GffValueV1::LocString(GffLocStringV1 {
                string_ref: u32::MAX,
                substrings: vec![GffLocSubstringV1 {
                    string_id: 2,
                    bytes: vec![b'o', b'k'],
                }],
            }),
        ),
        field("Void", GffValueV1::Void(vec![0, 1, 2, 255])),
        field("Struct", GffValueV1::Struct(nested)),
        field("List", GffValueV1::List(vec![listed])),
    ]);
    let artifact = write_gff_v32(&input, &GffWriterOptionsV1::default()).unwrap();
    assert_eq!(
        read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap(),
        input
    );
}

#[test]
fn field_indices_list_order_and_global_label_dedup_are_canonical() {
    let shared_a = GffStructV1 {
        struct_id: 1,
        fields: vec![field("Shared", GffValueV1::Byte(1))],
    };
    let shared_b = GffStructV1 {
        struct_id: 2,
        fields: vec![field("Shared", GffValueV1::Byte(2))],
    };
    let input = document(vec![
        field("A", GffValueV1::Byte(3)),
        field("B", GffValueV1::Byte(4)),
        field("Children", GffValueV1::List(vec![shared_a, shared_b])),
    ]);
    let artifact = write_gff_v32(&input, &GffWriterOptionsV1::default()).unwrap();
    assert_eq!(artifact.report.struct_count, 3);
    assert_eq!(artifact.report.field_count, 5);
    assert_eq!(artifact.report.label_count, 4);
    assert_eq!(artifact.report.field_indices_count, 12);
    assert_eq!(artifact.report.list_indices_count, 12);
    assert_eq!(
        read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap(),
        input
    );
}

#[test]
fn invalid_generated_values_have_stable_fatal_taxonomy() {
    let cases = [
        (
            document(vec![field("Bad\0Label", GffValueV1::Byte(1))]),
            "M6-GFF-LABEL-INVALID",
        ),
        (
            document(vec![field("abcdefghijklmnopq", GffValueV1::Byte(1))]),
            "M6-GFF-LABEL-INVALID",
        ),
        (
            document(vec![
                field("Dup", GffValueV1::Byte(1)),
                field("Dup", GffValueV1::Byte(2)),
            ]),
            "M6-GFF-DUPLICATE-LABEL",
        ),
        (
            document(vec![field("Float", GffValueV1::Float(f32::NAN))]),
            "M6-GFF-VALUE-INVALID",
        ),
        (
            document(vec![field(
                "ResRef",
                GffValueV1::ResRef("UPPER".to_owned()),
            )]),
            "M6-GFF-VALUE-INVALID",
        ),
    ];
    for (input, code) in cases {
        let error = write_gff_v32(&input, &GffWriterOptionsV1::default()).unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.severity, "FATAL");
    }
}

#[test]
fn reader_rejects_every_truncated_prefix_trailing_byte_and_key_mutations_without_panicking() {
    let payload = write_gff_v32(
        &document(vec![field("Value", GffValueV1::String(b"abc".to_vec()))]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap()
    .payload;
    for length in 0..payload.len() {
        let result = catch_unwind(AssertUnwindSafe(|| {
            read_gff_v32(&payload[..length], &GffLimitsV1::default())
        }));
        assert!(result.is_ok(), "reader panicked at prefix {length}");
        assert!(result.unwrap().is_err(), "prefix {length} was accepted");
    }
    let mut trailing = payload.clone();
    trailing.push(0);
    assert!(read_gff_v32(&trailing, &GffLimitsV1::default()).is_err());
    for offset in [0usize, 4, 8, 12, 16, 24, 32, 40, 48, 56, 68, 72, 76, 80] {
        let mut mutated = payload.clone();
        mutated[offset] ^= 0x80;
        let result = catch_unwind(AssertUnwindSafe(|| {
            read_gff_v32(&mutated, &GffLimitsV1::default())
        }));
        assert!(result.is_ok(), "reader panicked for mutation {offset}");
        assert!(result.unwrap().is_err(), "mutation {offset} was accepted");
    }
}

#[test]
fn empty_resref_is_physical_and_roundtrips_but_uppercase_is_rejected() {
    let input = document(vec![field("Script", GffValueV1::ResRef(String::new()))]);
    let artifact = write_gff_v32(&input, &GffWriterOptionsV1::default()).unwrap();
    assert_eq!(artifact.report.field_data_count, 1);
    assert_eq!(
        artifact.payload[artifact.report.field_data_offset as usize],
        0
    );
    assert_eq!(
        read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap(),
        input
    );

    let uppercase = document(vec![field(
        "Script",
        GffValueV1::ResRef("OnSpawn".to_owned()),
    )]);
    assert_eq!(
        write_gff_v32(&uppercase, &GffWriterOptionsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );
}

#[test]
fn physical_struct_reuse_unreachable_and_unowned_field_are_layout_fatal() {
    let input = document(vec![
        field(
            "First",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 1,
                fields: vec![],
            }),
        ),
        field(
            "Second",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 2,
                fields: vec![],
            }),
        ),
    ]);
    let payload = write_gff_v32(&input, &GffWriterOptionsV1::default())
        .unwrap()
        .payload;
    let field_offset = u32::from_le_bytes(payload[16..20].try_into().unwrap()) as usize;
    let mut reuse = payload.clone();
    reuse[field_offset + 20..field_offset + 24].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&reuse, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let mut root_cycle = payload.clone();
    root_cycle[field_offset + 8..field_offset + 12].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&root_cycle, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let one = write_gff_v32(
        &document(vec![field("Only", GffValueV1::Byte(1))]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap()
    .payload;
    let mut unowned = one.clone();
    unowned[64..68].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&unowned, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );
}

#[test]
fn label_array_is_unique_and_fully_referenced_and_loc_ids_are_nonnegative() {
    let base = write_gff_v32(
        &document(vec![
            field("One", GffValueV1::Byte(1)),
            field("Two", GffValueV1::Byte(2)),
        ]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap()
    .payload;
    let label_offset = u32::from_le_bytes(base[24..28].try_into().unwrap()) as usize;

    let mut duplicate_value = base.clone();
    let first_label = duplicate_value[label_offset..label_offset + 16].to_vec();
    duplicate_value[label_offset + 16..label_offset + 32].copy_from_slice(&first_label);
    assert_eq!(
        read_gff_v32(&duplicate_value, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let one_label = write_gff_v32(
        &document(vec![field("Used", GffValueV1::Byte(1))]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut unused_label = one_label.payload;
    let insertion = one_label.report.field_data_offset as usize;
    let mut extra_label = [0u8; 16];
    extra_label[..6].copy_from_slice(b"Unused");
    unused_label.splice(insertion..insertion, extra_label);
    unused_label[28..32].copy_from_slice(&2u32.to_le_bytes());
    unused_label[32..36].copy_from_slice(&(one_label.report.field_data_offset + 16).to_le_bytes());
    unused_label[40..44]
        .copy_from_slice(&(one_label.report.field_indices_offset + 16).to_le_bytes());
    unused_label[48..52]
        .copy_from_slice(&(one_label.report.list_indices_offset + 16).to_le_bytes());
    assert_eq!(
        read_gff_v32(&unused_label, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let negative = document(vec![field(
        "Name",
        GffValueV1::LocString(GffLocStringV1 {
            string_ref: u32::MAX,
            substrings: vec![GffLocSubstringV1 {
                string_id: -1,
                bytes: vec![],
            }],
        }),
    )]);
    assert_eq!(
        write_gff_v32(&negative, &GffWriterOptionsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let duplicate_id = document(vec![field(
        "Name",
        GffValueV1::LocString(GffLocStringV1 {
            string_ref: u32::MAX,
            substrings: vec![
                GffLocSubstringV1 {
                    string_id: 0,
                    bytes: vec![],
                },
                GffLocSubstringV1 {
                    string_id: 0,
                    bytes: vec![1],
                },
            ],
        }),
    )]);
    assert_eq!(
        write_gff_v32(&duplicate_id, &GffWriterOptionsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let valid_loc = write_gff_v32(
        &document(vec![field(
            "Name",
            GffValueV1::LocString(GffLocStringV1 {
                string_ref: u32::MAX,
                substrings: vec![GffLocSubstringV1 {
                    string_id: 0,
                    bytes: vec![],
                }],
            }),
        )]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut negative_physical = valid_loc.payload;
    let field_data_offset = valid_loc.report.field_data_offset as usize;
    negative_physical[field_data_offset + 12..field_data_offset + 16]
        .copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(
        read_gff_v32(&negative_physical, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );
}

#[test]
fn physical_field_data_alias_overlap_gap_and_uncovered_bytes_are_layout_fatal() {
    let artifact = write_gff_v32(
        &document(vec![
            field("First", GffValueV1::String(vec![1])),
            field("Second", GffValueV1::String(vec![2])),
        ]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let field_offset = artifact.report.field_offset as usize;
    let field_data_offset = artifact.report.field_data_offset as usize;
    let mut alias = artifact.payload.clone();
    alias[field_offset + 20..field_offset + 24].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&alias, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let mut overlap = artifact.payload.clone();
    overlap[field_offset + 20..field_offset + 24].copy_from_slice(&4u32.to_le_bytes());
    overlap[field_data_offset + 4..field_data_offset + 8].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&overlap, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let mut gap = artifact.payload.clone();
    gap[field_offset + 20..field_offset + 24].copy_from_slice(&6u32.to_le_bytes());
    gap[field_data_offset + 6..field_data_offset + 10].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&gap, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let mut uncovered = artifact.payload;
    let old_field_indices = artifact.report.field_indices_offset as usize;
    uncovered.insert(old_field_indices, 0);
    uncovered[36..40].copy_from_slice(&(artifact.report.field_data_count + 1).to_le_bytes());
    uncovered[40..44].copy_from_slice(&(artifact.report.field_indices_offset + 1).to_le_bytes());
    uncovered[48..52].copy_from_slice(&(artifact.report.list_indices_offset + 1).to_le_bytes());
    assert_eq!(
        read_gff_v32(&uncovered, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );
}

#[test]
fn physical_unused_field_and_list_indices_records_are_layout_fatal() {
    let fields_artifact = write_gff_v32(
        &document(vec![
            field("A", GffValueV1::Byte(1)),
            field("B", GffValueV1::Byte(2)),
        ]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut reused_field = fields_artifact.payload.clone();
    let field_indices_offset = fields_artifact.report.field_indices_offset as usize;
    reused_field[field_indices_offset + 4..field_indices_offset + 8]
        .copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&reused_field, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );
    let mut unused_field_index = fields_artifact.payload;
    let insert_at = fields_artifact.report.list_indices_offset as usize;
    unused_field_index.splice(insert_at..insert_at, 0u32.to_le_bytes());
    unused_field_index[44..48]
        .copy_from_slice(&(fields_artifact.report.field_indices_count + 4).to_le_bytes());
    unused_field_index[48..52]
        .copy_from_slice(&(fields_artifact.report.list_indices_offset + 4).to_le_bytes());
    assert_eq!(
        read_gff_v32(&unused_field_index, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let list_artifact = write_gff_v32(
        &document(vec![field("Items", GffValueV1::List(vec![]))]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut unused_list_index = list_artifact.payload;
    unused_list_index.extend_from_slice(&0u32.to_le_bytes());
    unused_list_index[52..56]
        .copy_from_slice(&(list_artifact.report.list_indices_count + 4).to_le_bytes());
    assert_eq!(
        read_gff_v32(&unused_list_index, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );
}

#[test]
fn swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order() {
    let artifact = write_gff_v32(
        &document(vec![
            field("A", GffValueV1::Byte(1)),
            field("B", GffValueV1::Byte(2)),
        ]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut swapped = artifact.payload;
    let offset = artifact.report.field_indices_offset as usize;
    swapped[offset..offset + 4].copy_from_slice(&1u32.to_le_bytes());
    swapped[offset + 4..offset + 8].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&swapped, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LAYOUT-INVALID"
    );
}

#[test]
fn phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit() {
    let artifact = write_gff_v32(
        &document(vec![
            field(
                "Name",
                GffValueV1::LocString(GffLocStringV1 {
                    string_ref: u32::MAX,
                    substrings: vec![GffLocSubstringV1 {
                        string_id: 0,
                        bytes: vec![7; 64],
                    }],
                }),
            ),
            field("Other", GffValueV1::Byte(1)),
        ]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let limits = GffLimitsV1 {
        max_loc_string_bytes: 20,
        ..GffLimitsV1::default()
    };
    let offset = artifact.report.field_indices_offset as usize;

    let mut swapped = artifact.payload.clone();
    swapped[offset..offset + 4].copy_from_slice(&1u32.to_le_bytes());
    swapped[offset + 4..offset + 8].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&swapped, &limits).unwrap_err().code,
        "M6-GFF-LAYOUT-INVALID"
    );

    let mut reused = artifact.payload;
    reused[offset + 4..offset + 8].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&reused, &limits).unwrap_err().code,
        "M6-GFF-LAYOUT-INVALID"
    );
}

#[test]
fn phase_eight_label_and_value_errors_precede_phase_nine_alias_coverage() {
    let artifact = write_gff_v32(
        &document(vec![
            field("First", GffValueV1::String(vec![1])),
            field("Second", GffValueV1::String(vec![2])),
        ]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let field_offset = artifact.report.field_offset as usize;
    let label_offset = artifact.report.label_offset as usize;

    let mut invalid_label_and_alias = artifact.payload.clone();
    invalid_label_and_alias[field_offset + 20..field_offset + 24]
        .copy_from_slice(&0u32.to_le_bytes());
    invalid_label_and_alias[label_offset] = 0;
    assert_eq!(
        read_gff_v32(&invalid_label_and_alias, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LABEL-INVALID"
    );

    let mut invalid_value_and_alias = artifact.payload.clone();
    invalid_value_and_alias[field_offset..field_offset + 4].copy_from_slice(&8u32.to_le_bytes());
    invalid_value_and_alias[field_offset + 8..field_offset + 12]
        .copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
    invalid_value_and_alias[field_offset + 20..field_offset + 24]
        .copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&invalid_value_and_alias, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let mut all_three = invalid_value_and_alias;
    all_three[label_offset] = 0;
    assert_eq!(
        read_gff_v32(&all_three, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-LABEL-INVALID"
    );
}

#[test]
fn physical_locstring_duplicate_id_lengths_and_total_size_have_exact_codes() {
    let one = write_gff_v32(
        &document(vec![field(
            "Name",
            GffValueV1::LocString(GffLocStringV1 {
                string_ref: u32::MAX,
                substrings: vec![GffLocSubstringV1 {
                    string_id: 0,
                    bytes: vec![],
                }],
            }),
        )]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let data = one.report.field_data_offset as usize;

    let mut negative_length = one.payload.clone();
    negative_length[data + 16..data + 20].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(
        read_gff_v32(&negative_length, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let mut truncated_substring = one.payload.clone();
    truncated_substring[data + 16..data + 20].copy_from_slice(&1i32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&truncated_substring, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let mut short_total = one.payload.clone();
    short_total[data..data + 4].copy_from_slice(&15u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&short_total, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let mut long_total = one.payload;
    long_total[data..data + 4].copy_from_slice(&17u32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&long_total, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-INDEX-OOB"
    );

    let two = write_gff_v32(
        &document(vec![field(
            "Name",
            GffValueV1::LocString(GffLocStringV1 {
                string_ref: u32::MAX,
                substrings: vec![
                    GffLocSubstringV1 {
                        string_id: 0,
                        bytes: vec![],
                    },
                    GffLocSubstringV1 {
                        string_id: 1,
                        bytes: vec![],
                    },
                ],
            }),
        )]),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut duplicate = two.payload;
    let data = two.report.field_data_offset as usize;
    duplicate[data + 20..data + 24].copy_from_slice(&0i32.to_le_bytes());
    assert_eq!(
        read_gff_v32(&duplicate, &GffLimitsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );
}

#[test]
fn deep_struct_list_graph_mutations_are_layout_fatal_without_panicking() {
    let deep = document(vec![field(
        "Children",
        GffValueV1::List(vec![GffStructV1 {
            struct_id: 10,
            fields: vec![field(
                "Nested",
                GffValueV1::Struct(GffStructV1 {
                    struct_id: 20,
                    fields: vec![field("Leaf", GffValueV1::Byte(1))],
                }),
            )],
        }]),
    )]);
    let artifact = write_gff_v32(&deep, &GffWriterOptionsV1::default()).unwrap();
    let mut list_points_to_grandchild = artifact.payload.clone();
    let list_offset = artifact.report.list_indices_offset as usize;
    list_points_to_grandchild[list_offset + 4..list_offset + 8]
        .copy_from_slice(&2u32.to_le_bytes());
    let result = catch_unwind(AssertUnwindSafe(|| {
        read_gff_v32(&list_points_to_grandchild, &GffLimitsV1::default())
    }));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().unwrap_err().code, "M6-GFF-LAYOUT-INVALID");

    let mut nested_self_cycle = artifact.payload;
    let field_offset = artifact.report.field_offset as usize;
    nested_self_cycle[field_offset + 20..field_offset + 24].copy_from_slice(&1u32.to_le_bytes());
    let result = catch_unwind(AssertUnwindSafe(|| {
        read_gff_v32(&nested_self_cycle, &GffLimitsV1::default())
    }));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().unwrap_err().code, "M6-GFF-LAYOUT-INVALID");
}

#[test]
fn strict_limits_root_and_depth_boundaries_are_enforced() {
    let mut invalid_root = document(vec![]);
    invalid_root.root.struct_id = 0;
    assert_eq!(
        write_gff_v32(&invalid_root, &GffWriterOptionsV1::default())
            .unwrap_err()
            .code,
        "M6-GFF-VALUE-INVALID"
    );

    let one_child = document(vec![field(
        "Child",
        GffValueV1::Struct(GffStructV1 {
            struct_id: 1,
            fields: vec![],
        }),
    )]);
    let mut options = GffWriterOptionsV1::default();
    options.limits.max_depth = 1;
    assert!(write_gff_v32(&one_child, &options).is_ok());
    options.limits.max_depth = 0;
    assert_eq!(
        write_gff_v32(&one_child, &options).unwrap_err().code,
        "M6-GFF-OPTIONS-INVALID"
    );

    let artifact = write_gff_v32(&one_child, &GffWriterOptionsV1::default()).unwrap();
    let mut limits = GffLimitsV1 {
        max_depth: 1,
        ..GffLimitsV1::default()
    };
    assert!(read_gff_v32(&artifact.payload, &limits).is_ok());
    limits.max_structs = 1;
    assert_eq!(
        read_gff_v32(&artifact.payload, &limits).unwrap_err().code,
        "M6-GFF-LIMIT-EXCEEDED"
    );
}

#[test]
fn configured_limits_are_inclusive_and_fail_one_past_boundary() {
    let empty = document(vec![]);
    let exact_bytes = GffWriterOptionsV1 {
        schema_version: 1,
        limits: GffLimitsV1 {
            max_gff_bytes: 68,
            ..GffLimitsV1::default()
        },
    };
    let payload = write_gff_v32(&empty, &exact_bytes).unwrap().payload;
    let too_small = GffWriterOptionsV1 {
        schema_version: 1,
        limits: GffLimitsV1 {
            max_gff_bytes: 67,
            ..GffLimitsV1::default()
        },
    };
    assert_eq!(
        write_gff_v32(&empty, &too_small).unwrap_err().code,
        "M6-GFF-LIMIT-EXCEEDED"
    );
    assert!(read_gff_v32(&payload, &exact_bytes.limits).is_ok());

    let strings = document(vec![
        field("Text", GffValueV1::String(vec![1, 2, 3])),
        field("Void", GffValueV1::Void(vec![4, 5, 6, 7])),
        field(
            "Items",
            GffValueV1::List(vec![
                GffStructV1 {
                    struct_id: 1,
                    fields: vec![],
                },
                GffStructV1 {
                    struct_id: 2,
                    fields: vec![],
                },
            ]),
        ),
    ]);
    let exact = GffWriterOptionsV1 {
        schema_version: 1,
        limits: GffLimitsV1 {
            max_structs: 3,
            max_fields: 3,
            max_labels: 3,
            max_fields_per_struct: 3,
            max_list_elements: 2,
            max_string_bytes: 3,
            max_void_bytes: 4,
            ..GffLimitsV1::default()
        },
    };
    assert!(write_gff_v32(&strings, &exact).is_ok());
    for limits in [
        GffLimitsV1 {
            max_structs: 2,
            ..exact.limits
        },
        GffLimitsV1 {
            max_fields: 2,
            ..exact.limits
        },
        GffLimitsV1 {
            max_labels: 2,
            ..exact.limits
        },
        GffLimitsV1 {
            max_fields_per_struct: 2,
            ..exact.limits
        },
        GffLimitsV1 {
            max_list_elements: 1,
            ..exact.limits
        },
        GffLimitsV1 {
            max_string_bytes: 2,
            ..exact.limits
        },
        GffLimitsV1 {
            max_void_bytes: 3,
            ..exact.limits
        },
    ] {
        assert_eq!(
            write_gff_v32(
                &strings,
                &GffWriterOptionsV1 {
                    schema_version: 1,
                    limits
                }
            )
            .unwrap_err()
            .code,
            "M6-GFF-LIMIT-EXCEEDED"
        );
    }

    let loc = document(vec![field(
        "Loc",
        GffValueV1::LocString(GffLocStringV1 {
            string_ref: u32::MAX,
            substrings: vec![GffLocSubstringV1 {
                string_id: 0,
                bytes: vec![1, 2, 3],
            }],
        }),
    )]);
    assert!(
        write_gff_v32(
            &loc,
            &GffWriterOptionsV1 {
                schema_version: 1,
                limits: GffLimitsV1 {
                    max_loc_string_bytes: 23,
                    ..GffLimitsV1::default()
                },
            },
        )
        .is_ok()
    );
    assert_eq!(
        write_gff_v32(
            &loc,
            &GffWriterOptionsV1 {
                schema_version: 1,
                limits: GffLimitsV1 {
                    max_loc_string_bytes: 22,
                    ..GffLimitsV1::default()
                },
            },
        )
        .unwrap_err()
        .code,
        "M6-GFF-LIMIT-EXCEEDED"
    );

    let invalid_zero = GffLimitsV1 {
        max_diagnostics: 0,
        ..GffLimitsV1::default()
    };
    assert_eq!(
        read_gff_v32(&payload, &invalid_zero).unwrap_err().code,
        "M6-GFF-OPTIONS-INVALID"
    );
}

#[test]
fn deterministic_corpus_is_no_panic() {
    let mut state = 0x1234_5678u32;
    for length in 0..384usize {
        let mut bytes = vec![0u8; length];
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            *byte = state as u8;
        }
        assert!(
            catch_unwind(AssertUnwindSafe(|| read_gff_v32(
                &bytes,
                &GffLimitsV1::default()
            )))
            .is_ok()
        );
    }
}
