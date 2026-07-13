use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::erf::{ErfArchive, ErfFileType};
use m2a_core::hak::{
    DUPLICATE_KEY, ENTRY_LIMIT_EXCEEDED, HAK_MAX_ENTRY_COUNT, HAK_MAX_OUTPUT_BYTES,
    HakResourceInputV1, HakWriterLimitsV1, HakWriterOptionsV1, OPTIONS_INVALID,
    OUTPUT_LIMIT_EXCEEDED, RESREF_INVALID, write_hak_v1,
};
use m2a_core::two_da::{
    TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
    append_two_da_row_v1,
};

fn resource(resref: &str, resource_type: u16, payload: &[u8]) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload: payload.to_vec(),
    }
}

fn options(max_entry_count: u64, max_output_bytes: u64) -> HakWriterOptionsV1 {
    HakWriterOptionsV1 {
        schema_version: 1,
        limits: HakWriterLimitsV1 {
            max_entry_count,
            max_output_bytes,
        },
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[test]
fn empty_hak_is_exact_frozen_160_byte_archive_and_report_json() {
    let artifact = write_hak_v1(&[], &HakWriterOptionsV1::default()).unwrap();
    assert_eq!(artifact.payload.len(), 160);
    assert_eq!(&artifact.payload[0..8], b"HAK V1.0");
    assert_eq!(u32_at(&artifact.payload, 0x08), 0);
    assert_eq!(u32_at(&artifact.payload, 0x0c), 0);
    assert_eq!(u32_at(&artifact.payload, 0x10), 0);
    assert_eq!(u32_at(&artifact.payload, 0x14), 0xa0);
    assert_eq!(u32_at(&artifact.payload, 0x18), 0xa0);
    assert_eq!(u32_at(&artifact.payload, 0x1c), 0xa0);
    assert_eq!(u32_at(&artifact.payload, 0x20), 0);
    assert_eq!(u32_at(&artifact.payload, 0x24), 0);
    assert_eq!(u32_at(&artifact.payload, 0x28), u32::MAX);
    assert!(artifact.payload[0x2c..].iter().all(|byte| *byte == 0));
    assert_eq!(artifact.report.entry_count, 0);
    assert_eq!(artifact.report.payload_offset, 160);
    assert_eq!(artifact.report.byte_length, 160);
    assert!(artifact.report.resources.is_empty());
    assert_eq!(
        artifact.report.archive_sha256,
        "e424e1f5f5bbdca88f4e1476df2c90de9573d18fb0c415c82667102b661e5132"
    );
    assert_eq!(
        serde_json::to_string(&artifact.report).unwrap(),
        "{\"schemaVersion\":1,\"entryCount\":0,\"keyTableOffset\":160,\"resourceTableOffset\":160,\"payloadOffset\":160,\"byteLength\":160,\"archiveSha256\":\"e424e1f5f5bbdca88f4e1476df2c90de9573d18fb0c415c82667102b661e5132\",\"resources\":[]}"
    );
}

#[test]
fn one_resource_has_exact_key_descriptor_payload_and_hashes() {
    let input = vec![resource("a", 3, &[1, 2, 3])];
    let before = input.clone();
    let artifact = write_hak_v1(&input, &HakWriterOptionsV1::default()).unwrap();
    assert_eq!(input, before);
    assert_eq!(artifact.payload.len(), 195);
    assert_eq!(
        &artifact.payload[160..176],
        b"a\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
    );
    assert_eq!(u32_at(&artifact.payload, 176), 0);
    assert_eq!(&artifact.payload[180..182], &3_u16.to_le_bytes());
    assert_eq!(&artifact.payload[182..184], &[0, 0]);
    assert_eq!(u32_at(&artifact.payload, 184), 192);
    assert_eq!(u32_at(&artifact.payload, 188), 3);
    assert_eq!(&artifact.payload[192..], &[1, 2, 3]);
    assert_eq!(artifact.report.resources[0].payload_offset, 192);
    assert_eq!(artifact.report.resources[0].payload_size, 3);
    assert_eq!(
        artifact.report.resources[0].payload_sha256,
        "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81"
    );
    assert_eq!(
        serde_json::to_string(&artifact.report).unwrap(),
        "{\"schemaVersion\":1,\"entryCount\":1,\"keyTableOffset\":160,\"resourceTableOffset\":184,\"payloadOffset\":192,\"byteLength\":195,\"archiveSha256\":\"baad4ab97734ae745c8828f00104b2ab280875143fa1f93701ead6beb5172310\",\"resources\":[{\"resref\":\"a\",\"resourceId\":0,\"resourceType\":3,\"payloadOffset\":192,\"payloadSize\":3,\"payloadSha256\":\"039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81\"}]}"
    );
}

#[test]
fn all_three_resource_permutations_are_byte_and_report_identical() {
    let items = [
        resource("appearance", 2017, b"2da"),
        resource("model", 2002, b"mdl"),
        resource("texture", 3, b"tga"),
    ];
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let expected = write_hak_v1(&items, &HakWriterOptionsV1::default()).unwrap();
    for permutation in permutations {
        let input = permutation.map(|index| items[index].clone());
        let actual = write_hak_v1(&input, &HakWriterOptionsV1::default()).unwrap();
        assert_eq!(actual, expected);
    }
    assert_eq!(
        expected
            .report
            .resources
            .iter()
            .map(|item| (item.resref.as_str(), item.resource_type, item.resource_id))
            .collect::<Vec<_>>(),
        [
            ("appearance", 2017, 0),
            ("model", 2002, 1),
            ("texture", 3, 2)
        ]
    );
}

#[test]
fn sorting_is_bytewise_then_numeric_type_and_same_resref_different_type_is_legal() {
    let artifact = write_hak_v1(
        &[
            resource("b", 1, b"b"),
            resource("a", 2017, b"two"),
            resource("a", 3, b"one"),
        ],
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    assert_eq!(
        artifact
            .report
            .resources
            .iter()
            .map(|item| (item.resref.as_str(), item.resource_type, item.resource_id))
            .collect::<Vec<_>>(),
        [("a", 3, 0), ("a", 2017, 1), ("b", 1, 2)]
    );
    assert_eq!(
        &artifact.payload[artifact.report.payload_offset as usize..],
        b"onetwob"
    );
}

#[test]
fn resref_length_one_and_sixteen_have_exact_padding_policy() {
    let artifact = write_hak_v1(
        &[resource("a", 1, b""), resource("abcdefghijklmnop", 2, b"")],
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    assert_eq!(
        &artifact.payload[160..176],
        b"a\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
    );
    assert_eq!(&artifact.payload[184..200], b"abcdefghijklmnop");
}

#[test]
fn zero_length_payloads_share_cursor_and_final_empty_payload_is_exact_eof() {
    let artifact = write_hak_v1(
        &[
            resource("a", 1, b""),
            resource("b", 1, b"x"),
            resource("c", 1, b""),
        ],
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    let resources = &artifact.report.resources;
    assert_eq!(resources[0].payload_size, 0);
    assert_eq!(resources[0].payload_offset, resources[1].payload_offset);
    assert_eq!(resources[1].payload_size, 1);
    assert_eq!(resources[2].payload_size, 0);
    assert_eq!(
        u64::from(resources[2].payload_offset),
        artifact.report.byte_length
    );
    assert_eq!(artifact.payload.last(), Some(&b'x'));
}

#[test]
fn configured_limits_are_inclusive_and_validation_precedence_is_stable() {
    assert!(write_hak_v1(&[], &options(0, 160)).is_ok());
    let single = [resource("a", 1, b"x")];
    assert!(write_hak_v1(&single, &options(1, 193)).is_ok());
    assert_eq!(
        write_hak_v1(&single, &options(0, 193)).unwrap_err().code,
        ENTRY_LIMIT_EXCEEDED
    );
    assert_eq!(
        write_hak_v1(&single, &options(1, 192)).unwrap_err().code,
        OUTPUT_LIMIT_EXCEEDED
    );

    let invalid = [resource("BAD", 1, b"")];
    let mut invalid_options = options(0, 159);
    invalid_options.schema_version = 2;
    assert_eq!(
        write_hak_v1(&invalid, &invalid_options).unwrap_err().code,
        OPTIONS_INVALID
    );
    assert_eq!(
        write_hak_v1(&invalid, &options(0, 160)).unwrap_err().code,
        ENTRY_LIMIT_EXCEEDED
    );
    assert!(write_hak_v1(&[], &options(HAK_MAX_ENTRY_COUNT, HAK_MAX_OUTPUT_BYTES)).is_ok());
}

#[test]
fn both_limits_above_hard_are_options_errors_before_invalid_input() {
    let invalid = [resource("BAD", 1, b"")];
    assert_eq!(
        write_hak_v1(
            &invalid,
            &options(HAK_MAX_ENTRY_COUNT + 1, HAK_MAX_OUTPUT_BYTES)
        )
        .unwrap_err()
        .code,
        OPTIONS_INVALID
    );
    assert_eq!(
        write_hak_v1(
            &invalid,
            &options(HAK_MAX_ENTRY_COUNT, HAK_MAX_OUTPUT_BYTES + 1)
        )
        .unwrap_err()
        .code,
        OPTIONS_INVALID
    );
}

#[test]
fn invalid_public_matrix_never_panics_and_preserves_error_classification() {
    let invalid_schema = HakWriterOptionsV1 {
        schema_version: 2,
        ..HakWriterOptionsV1::default()
    };
    let cases = vec![
        ("options schema", vec![], invalid_schema, OPTIONS_INVALID),
        (
            "options entry hard limit",
            vec![],
            options(HAK_MAX_ENTRY_COUNT + 1, HAK_MAX_OUTPUT_BYTES),
            OPTIONS_INVALID,
        ),
        (
            "options output hard limit",
            vec![],
            options(HAK_MAX_ENTRY_COUNT, HAK_MAX_OUTPUT_BYTES + 1),
            OPTIONS_INVALID,
        ),
        (
            "options output below minimum",
            vec![],
            options(HAK_MAX_ENTRY_COUNT, 159),
            OPTIONS_INVALID,
        ),
        (
            "entry limit",
            vec![resource("a", 1, b"")],
            options(0, 160),
            ENTRY_LIMIT_EXCEEDED,
        ),
        (
            "resref",
            vec![resource("BAD", 1, b"")],
            HakWriterOptionsV1::default(),
            RESREF_INVALID,
        ),
        (
            "duplicate",
            vec![resource("a", 1, b""), resource("a", 1, b"")],
            HakWriterOptionsV1::default(),
            DUPLICATE_KEY,
        ),
        (
            "output limit",
            vec![resource("a", 1, b"x")],
            options(1, 192),
            OUTPUT_LIMIT_EXCEEDED,
        ),
    ];
    for (label, resources, options, expected_code) in cases {
        let result = catch_unwind(AssertUnwindSafe(|| write_hak_v1(&resources, &options)));
        let error = result
            .expect("invalid public input must not panic")
            .unwrap_err();
        assert_eq!(error.code, expected_code, "{label}");
    }
}

#[test]
fn invalid_resrefs_and_duplicate_original_position_have_stable_errors_without_panics() {
    for value in ["", "A", "a-b", "a b", "a/b", "abcdefghijklmnopq", "zaż"] {
        let input = [resource(value, 1, b"")];
        let result = catch_unwind(AssertUnwindSafe(|| {
            write_hak_v1(&input, &HakWriterOptionsV1::default())
        }));
        let error = result.expect("invalid resref must not panic").unwrap_err();
        assert_eq!(error.code, RESREF_INVALID, "unexpected error for {value:?}");
        assert_eq!(error.path, "resources[0].resref");
    }

    let duplicate = [resource("a", 1, b"first"), resource("a", 1, b"second")];
    let error = write_hak_v1(&duplicate, &HakWriterOptionsV1::default()).unwrap_err();
    assert_eq!(error.code, DUPLICATE_KEY);
    assert_eq!(error.path, "resources[1]");

    for duplicate_position in [2, 4] {
        let mut resources = vec![
            resource("dup", 1, b"first"),
            resource("b", 1, b"b"),
            resource("c", 1, b"c"),
            resource("d", 1, b"d"),
            resource("e", 1, b"e"),
        ];
        resources[duplicate_position] = resource("dup", 1, b"later");
        let error = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap_err();
        assert_eq!(error.code, DUPLICATE_KEY);
        assert_eq!(error.path, format!("resources[{duplicate_position}]"));
    }
}

#[test]
fn strict_json_and_representative_error_shape_are_frozen() {
    let input = resource("a", 3, &[1, 2]);
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, r#"{"resref":"a","resourceType":3,"payload":[1,2]}"#);
    assert!(
        serde_json::from_str::<HakResourceInputV1>(
            r#"{"resref":"a","resourceType":3,"payload":[],"unknown":true}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<HakWriterOptionsV1>(
            r#"{"schemaVersion":1,"limits":{"maxEntryCount":1,"maxOutputBytes":160},"unknown":true}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<HakWriterOptionsV1>(
            r#"{"schemaVersion":1,"limits":{"maxEntryCount":1,"maxOutputBytes":160,"unknown":true}}"#
        )
        .is_err()
    );
    let options_json = serde_json::to_string(&HakWriterOptionsV1::default()).unwrap();
    assert_eq!(
        options_json,
        r#"{"schemaVersion":1,"limits":{"maxEntryCount":262144,"maxOutputBytes":268435456}}"#
    );

    let error = write_hak_v1(&[], &options(0, 159)).unwrap_err();
    assert_eq!(error.code, OPTIONS_INVALID);
    assert_eq!(error.path, "options.limits.maxOutputBytes");
    assert_eq!(error.severity, "FATAL");
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        r#"{"schemaVersion":1,"code":"M5-HAK-OPTIONS-INVALID","severity":"FATAL","path":"options.limits.maxOutputBytes","message":"maxOutputBytes must be in 160..=268435456"}"#
    );
}

#[test]
fn owned_full_width_two_da_roundtrips_exactly_through_generated_hak() {
    let columns: Vec<String> = (0..35).map(|index| format!("COL_{index:02}")).collect();
    let source = format!("2DA V2.0\n\n{}\n", columns.join(" ")).into_bytes();
    let request = TwoDaAppendRequestV1 {
        schema_version: 1,
        cells: vec![
            TwoDaCellAssignmentV1 {
                column_name: "COL_00".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "15219".to_owned(),
                },
            },
            TwoDaCellAssignmentV1 {
                column_name: "col_34".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "mesh_resref".to_owned(),
                },
            },
        ],
    };
    let source_before = source.clone();
    let request_before = request.clone();
    let two_da = append_two_da_row_v1(&source, &request, &TwoDaLimitsV1::default()).unwrap();
    let repeated_two_da =
        append_two_da_row_v1(&source, &request, &TwoDaLimitsV1::default()).unwrap();
    assert_eq!(source, source_before);
    assert_eq!(request, request_before);
    assert_eq!(two_da, repeated_two_da);
    assert_eq!(&two_da.payload[..source.len()], source);

    let resources = vec![
        resource("texture", 3, b"owned-tga"),
        resource("appearance", 2017, &two_da.payload),
        resource("model", 2002, b"owned-mdl"),
    ];
    let resources_before = resources.clone();
    let hak = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap();
    let repeated_hak = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap();
    assert_eq!(resources, resources_before);
    assert_eq!(hak, repeated_hak);

    let archive = ErfArchive::parse(&hak.payload).unwrap();
    assert_eq!(archive.file_type(), ErfFileType::Hak);
    let appearance_bytes = archive.find("appearance", 2017).unwrap();
    assert_eq!(appearance_bytes, two_da.payload);

    let reported = hak
        .report
        .resources
        .iter()
        .find(|resource| resource.resref == "appearance" && resource.resource_type == 2017)
        .unwrap();
    let readback = archive
        .resources()
        .iter()
        .find(|resource| resource.resref == "appearance" && resource.resource_type == 2017)
        .unwrap();
    assert_eq!(reported.resource_id, readback.resource_id);
    assert_eq!(reported.payload_offset as usize, readback.offset);
    assert_eq!(reported.payload_size as usize, readback.size);
    assert_eq!(reported.payload_size as usize, two_da.payload.len());
    assert_eq!(reported.payload_sha256, two_da.report.output_sha256);
}
