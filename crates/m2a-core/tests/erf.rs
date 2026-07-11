use m2a_core::erf::{
    DEFAULT_MAX_ENTRY_COUNT, DUPLICATE_KEY, ErfArchive, ErfFileType, HEADER_OOB, HEADER_SIZE,
    KEY_ENTRY_SIZE, KEY_TABLE_OOB, LIMIT_EXCEEDED, PAYLOAD_OOB, RESOURCE_ENTRY_SIZE,
    RESOURCE_ID_INVALID, RESOURCE_MISSING, RESOURCE_TABLE_OOB, RESREF_INVALID,
    SIGNATURE_UNSUPPORTED, VERSION_UNSUPPORTED,
};

#[derive(Clone, Copy)]
struct FixtureEntry<'a> {
    resref: &'a str,
    resource_type: u16,
    payload: &'a [u8],
}

fn build_erf(signature: [u8; 4], entries: &[FixtureEntry<'_>]) -> Vec<u8> {
    let key_offset = HEADER_SIZE;
    let resource_offset = key_offset + entries.len() * KEY_ENTRY_SIZE;
    let payload_offset = resource_offset + entries.len() * RESOURCE_ENTRY_SIZE;
    let payload_size: usize = entries.iter().map(|entry| entry.payload.len()).sum();
    let mut bytes = vec![0; payload_offset + payload_size];

    bytes[0..4].copy_from_slice(&signature);
    bytes[4..8].copy_from_slice(b"V1.0");
    write_u32(&mut bytes, 16, entries.len() as u32);
    write_u32(&mut bytes, 24, key_offset as u32);
    write_u32(&mut bytes, 28, resource_offset as u32);

    let mut next_payload_offset = payload_offset;
    for (index, entry) in entries.iter().enumerate() {
        let key = key_offset + index * KEY_ENTRY_SIZE;
        bytes[key..key + entry.resref.len()].copy_from_slice(entry.resref.as_bytes());
        write_u32(&mut bytes, key + 16, index as u32);
        write_u16(&mut bytes, key + 20, entry.resource_type);

        let resource = resource_offset + index * RESOURCE_ENTRY_SIZE;
        write_u32(&mut bytes, resource, next_payload_offset as u32);
        write_u32(&mut bytes, resource + 4, entry.payload.len() as u32);
        bytes[next_payload_offset..next_payload_offset + entry.payload.len()]
            .copy_from_slice(entry.payload);
        next_payload_offset += entry.payload.len();
    }

    bytes
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn reads_supported_v10_signatures() {
    for (signature, expected) in [(*b"ERF ", ErfFileType::Erf), (*b"HAK ", ErfFileType::Hak)] {
        let bytes = build_erf(signature, &[]);
        let archive = ErfArchive::parse(&bytes).unwrap();
        assert_eq!(archive.file_type(), expected);
        assert_eq!(expected.signature(), signature);
    }
}

#[test]
fn lookup_is_case_insensitive_type_exact_and_borrows_payload() {
    let bytes = build_erf(
        *b"HAK ",
        &[
            FixtureEntry {
                resref: "appearance",
                resource_type: 2017,
                payload: b"2DA V2.0\n",
            },
            FixtureEntry {
                resref: "m2a_koc01",
                resource_type: 2002,
                payload: b"MDL bytes",
            },
            FixtureEntry {
                resref: "m2a_koc01",
                resource_type: 3,
                payload: b"TGA bytes",
            },
        ],
    );

    let archive = ErfArchive::parse(&bytes).unwrap();
    let mdl = archive.find("M2A_KOC01", 2002).unwrap();
    let tga = archive.find("m2a_koc01", 3).unwrap();
    assert_eq!(mdl, b"MDL bytes");
    assert_eq!(tga, b"TGA bytes");

    let metadata = &archive.resources()[1];
    assert_eq!(mdl.as_ptr(), bytes[metadata.offset..].as_ptr());
}

#[test]
fn inventory_is_deterministic_in_key_table_order() {
    let bytes = build_erf(
        *b"HAK ",
        &[
            FixtureEntry {
                resref: "z_model",
                resource_type: 2002,
                payload: b"z",
            },
            FixtureEntry {
                resref: "appearance",
                resource_type: 2017,
                payload: b"a",
            },
            FixtureEntry {
                resref: "a_texture",
                resource_type: 3,
                payload: b"t",
            },
        ],
    );

    let first = ErfArchive::parse(&bytes).unwrap();
    let second = ErfArchive::parse(&bytes).unwrap();
    assert_eq!(first.resources(), second.resources());
    assert_eq!(
        first
            .resources()
            .iter()
            .map(|resource| resource.resref.as_str())
            .collect::<Vec<_>>(),
        ["z_model", "appearance", "a_texture"]
    );
}

#[test]
fn missing_resource_has_stable_diagnostic() {
    let bytes = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    let archive = ErfArchive::parse(&bytes).unwrap();

    let error = archive.find("model", 2017).unwrap_err();
    assert_eq!(error.code, RESOURCE_MISSING);
    assert_eq!(error.schema_version, 1);
}

#[test]
fn rejects_truncated_header_and_unsupported_identity() {
    let error = ErfArchive::parse(&[0; HEADER_SIZE - 1]).unwrap_err();
    assert_eq!(error.code, HEADER_OOB);

    let mut bytes = build_erf(*b"SAV ", &[]);
    let error = ErfArchive::parse(&bytes).unwrap_err();
    assert_eq!(error.code, SIGNATURE_UNSUPPORTED);

    bytes[0..4].copy_from_slice(b"HAK ");
    bytes[4..8].copy_from_slice(b"V1.1");
    let error = ErfArchive::parse(&bytes).unwrap_err();
    assert_eq!(error.code, VERSION_UNSUPPORTED);
}

#[test]
fn rejects_entry_count_above_project_limit_before_reading_tables() {
    let mut bytes = build_erf(*b"HAK ", &[]);
    write_u32(&mut bytes, 16, (DEFAULT_MAX_ENTRY_COUNT + 1) as u32);

    let error = ErfArchive::parse(&bytes).unwrap_err();
    assert_eq!(error.code, LIMIT_EXCEEDED);
    assert_eq!(error.offset, 16);
}

#[test]
fn rejects_key_resource_and_payload_ranges_outside_input() {
    let mut key_oob = build_erf(*b"HAK ", &[]);
    write_u32(&mut key_oob, 16, 1);
    let key_oob_len = key_oob.len() as u32;
    write_u32(&mut key_oob, 24, key_oob_len);
    let error = ErfArchive::parse(&key_oob).unwrap_err();
    assert_eq!(error.code, KEY_TABLE_OOB);

    let mut resource_oob = build_erf(*b"HAK ", &[]);
    write_u32(&mut resource_oob, 16, 1);
    write_u32(&mut resource_oob, 24, HEADER_SIZE as u32);
    resource_oob.resize(HEADER_SIZE + KEY_ENTRY_SIZE, 0);
    resource_oob[HEADER_SIZE] = b'x';
    write_u32(&mut resource_oob, 28, u32::MAX);
    let error = ErfArchive::parse(&resource_oob).unwrap_err();
    assert_eq!(error.code, RESOURCE_TABLE_OOB);

    let mut payload_oob = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    let resource_offset = HEADER_SIZE + KEY_ENTRY_SIZE;
    let payload_oob_len = payload_oob.len() as u32;
    write_u32(&mut payload_oob, resource_offset, payload_oob_len);
    write_u32(&mut payload_oob, resource_offset + 4, 1);
    let error = ErfArchive::parse(&payload_oob).unwrap_err();
    assert_eq!(error.code, PAYLOAD_OOB);
}

#[test]
fn rejects_duplicate_keys_case_insensitively() {
    let bytes = build_erf(
        *b"HAK ",
        &[
            FixtureEntry {
                resref: "model",
                resource_type: 2002,
                payload: b"one",
            },
            FixtureEntry {
                resref: "model",
                resource_type: 2002,
                payload: b"two",
            },
        ],
    );

    let error = ErfArchive::parse(&bytes).unwrap_err();
    assert_eq!(error.code, DUPLICATE_KEY);
}

#[test]
fn rejects_resource_ids_that_do_not_equal_their_key_index() {
    let mut out_of_range = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    write_u32(&mut out_of_range, HEADER_SIZE + 16, 1);
    let error = ErfArchive::parse(&out_of_range).unwrap_err();
    assert_eq!(error.code, RESOURCE_ID_INVALID);

    let mut swapped = build_erf(
        *b"HAK ",
        &[
            FixtureEntry {
                resref: "model_a",
                resource_type: 2002,
                payload: b"a",
            },
            FixtureEntry {
                resref: "model_b",
                resource_type: 2002,
                payload: b"b",
            },
        ],
    );
    write_u32(&mut swapped, HEADER_SIZE + 16, 1);
    write_u32(&mut swapped, HEADER_SIZE + KEY_ENTRY_SIZE + 16, 0);
    let error = ErfArchive::parse(&swapped).unwrap_err();
    assert_eq!(error.code, RESOURCE_ID_INVALID);
}

#[test]
fn enforces_resref_ascii_and_nul_padding_rules() {
    let mut non_ascii = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    non_ascii[HEADER_SIZE] = 0xff;
    let error = ErfArchive::parse(&non_ascii).unwrap_err();
    assert_eq!(error.code, RESREF_INVALID);

    let mut data_after_nul = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    data_after_nul[HEADER_SIZE + 8] = b'x';
    let error = ErfArchive::parse(&data_after_nul).unwrap_err();
    assert_eq!(error.code, RESREF_INVALID);

    let bytes = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    let archive = ErfArchive::parse(&bytes).unwrap();
    assert_eq!(
        archive.find("módel", 2002).unwrap_err().code,
        RESREF_INVALID
    );
    assert_eq!(archive.find("", 2002).unwrap_err().code, RESREF_INVALID);
    for invalid_query in ["model.name", "model name", "model/name", "model\u{7f}"] {
        assert_eq!(
            archive.find(invalid_query, 2002).unwrap_err().code,
            RESREF_INVALID,
            "query {invalid_query:?} should be invalid"
        );
    }
}

#[test]
fn accepts_uppercase_stored_resref_and_matches_case_insensitively() {
    let bytes = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "Aelephant",
            resource_type: 2033,
            payload: b"utc",
        }],
    );

    let archive = ErfArchive::parse(&bytes).unwrap();
    assert_eq!(archive.find("aelephant", 2033).unwrap(), b"utc");
    assert_eq!(archive.resources()[0].resref, "Aelephant");
}

#[test]
fn accepts_canonical_hyphenated_resref_in_storage_and_query() {
    let bytes = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "c_jelly-mst-l",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );

    let archive = ErfArchive::parse(&bytes).unwrap();
    assert_eq!(archive.find("C_JELLY-MST-L", 2002).unwrap(), b"mdl");
}

#[test]
fn rejects_stored_resrefs_with_punctuation_or_control_bytes() {
    for (label, replacement) in [("space", b' '), ("dot", b'.'), ("control", 0x1f)] {
        let mut bytes = build_erf(
            *b"HAK ",
            &[FixtureEntry {
                resref: "model",
                resource_type: 2002,
                payload: b"mdl",
            }],
        );
        bytes[HEADER_SIZE] = replacement;

        let error = ErfArchive::parse(&bytes).unwrap_err();
        assert_eq!(error.code, RESREF_INVALID, "{label} should be rejected");
    }
}

#[test]
fn zero_size_payload_is_valid_at_exact_eof_but_not_past_eof() {
    let mut bytes = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "empty",
            resource_type: 2002,
            payload: b"",
        }],
    );
    let resource_offset = HEADER_SIZE + KEY_ENTRY_SIZE;
    let eof = bytes.len() as u32;
    write_u32(&mut bytes, resource_offset, eof);

    let archive = ErfArchive::parse(&bytes).unwrap();
    assert_eq!(archive.find("empty", 2002).unwrap(), b"");

    write_u32(&mut bytes, resource_offset, eof + 1);
    let error = ErfArchive::parse(&bytes).unwrap_err();
    assert_eq!(error.code, PAYLOAD_OOB);
}

#[test]
fn rejects_metadata_and_nonempty_payload_overlaps() {
    let mut tables_overlap = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    write_u32(&mut tables_overlap, 28, HEADER_SIZE as u32);
    let error = ErfArchive::parse(&tables_overlap).unwrap_err();
    assert_eq!(error.code, RESOURCE_TABLE_OOB);

    let mut payload_metadata_overlap = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "model",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );
    let resource_offset = HEADER_SIZE + KEY_ENTRY_SIZE;
    write_u32(&mut payload_metadata_overlap, resource_offset, 0);
    write_u32(&mut payload_metadata_overlap, resource_offset + 4, 1);
    let error = ErfArchive::parse(&payload_metadata_overlap).unwrap_err();
    assert_eq!(error.code, PAYLOAD_OOB);

    let mut payloads_overlap = build_erf(
        *b"HAK ",
        &[
            FixtureEntry {
                resref: "model_a",
                resource_type: 2002,
                payload: b"aaaa",
            },
            FixtureEntry {
                resref: "model_b",
                resource_type: 2002,
                payload: b"bbbb",
            },
        ],
    );
    let resource_table_offset = HEADER_SIZE + 2 * KEY_ENTRY_SIZE;
    let first_payload_offset = u32::from_le_bytes(
        payloads_overlap[resource_table_offset..resource_table_offset + 4]
            .try_into()
            .unwrap(),
    );
    write_u32(
        &mut payloads_overlap,
        resource_table_offset + RESOURCE_ENTRY_SIZE,
        first_payload_offset + 2,
    );
    let error = ErfArchive::parse(&payloads_overlap).unwrap_err();
    assert_eq!(error.code, PAYLOAD_OOB);
}

#[test]
fn every_truncated_prefix_returns_without_panicking() {
    let bytes = build_erf(
        *b"HAK ",
        &[
            FixtureEntry {
                resref: "appearance",
                resource_type: 2017,
                payload: b"2da",
            },
            FixtureEntry {
                resref: "model",
                resource_type: 2002,
                payload: b"mdl",
            },
        ],
    );

    for end in 0..bytes.len() {
        let result = std::panic::catch_unwind(|| ErfArchive::parse(&bytes[..end]));
        assert!(result.is_ok(), "parser panicked for prefix length {end}");
        assert!(
            result.unwrap().is_err(),
            "truncated prefix {end} was accepted"
        );
    }
    assert!(ErfArchive::parse(&bytes).is_ok());
}

#[test]
fn accepts_full_sixteen_byte_resref_without_nul_terminator() {
    let bytes = build_erf(
        *b"HAK ",
        &[FixtureEntry {
            resref: "abcdefghijklmnop",
            resource_type: 2002,
            payload: b"mdl",
        }],
    );

    let archive = ErfArchive::parse(&bytes).unwrap();
    assert_eq!(archive.find("ABCDEFGHIJKLMNOP", 2002).unwrap(), b"mdl");
}

#[test]
fn arbitrary_byte_inputs_never_panic() {
    for seed in 0_u32..32 {
        for length in [0, 1, 4, 8, 31, 159, 160, 161, 255, 512] {
            let mut state = seed.wrapping_add(1);
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                bytes.push((state >> 24) as u8);
            }

            let result = std::panic::catch_unwind(|| ErfArchive::parse(&bytes));
            assert!(
                result.is_ok(),
                "parser panicked for seed {seed} and length {length}"
            );
        }
    }
}
