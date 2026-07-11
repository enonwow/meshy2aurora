use m2a_core::erf::{ErfArchive, ErfFileType};
use m2a_core::{
    CapabilityResult, CapabilityStatus, ExecutionMetadata, HashAlgorithm, InputFingerprint,
    InspectionReport, InvariantResult, InvariantStatus, ReferenceCapability, ReferenceIdentity,
    ReferenceManifest, ReferenceManifestEntry, ReferenceSource, build_reference_proof_packet,
    inspect_binary_mdl,
};
use serde_json::Value;

const MDL_RESOURCE_TYPE: u16 = 2002;
const FILE_HEADER_SIZE: usize = 12;

#[derive(Clone, Copy)]
struct ExpectedReference {
    reference_id: &'static str,
    resref: &'static str,
    resource_id: u32,
    offset: usize,
    sha256: &'static str,
    payload_length: usize,
    core_length: usize,
    raw_length: usize,
}

const REFERENCES: [ExpectedReference; 3] = [
    ExpectedReference {
        reference_id: "R1",
        resref: "c_kocrachn",
        resource_id: 724,
        offset: 179_725_952,
        sha256: "f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270",
        payload_length: 163_192,
        core_length: 76_048,
        raw_length: 87_132,
    },
    ExpectedReference {
        reference_id: "R3a",
        resref: "c_phod_horror_b",
        resource_id: 1_026,
        offset: 264_142_176,
        sha256: "62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a",
        payload_length: 846_064,
        core_length: 788_416,
        raw_length: 57_636,
    },
    ExpectedReference {
        reference_id: "R3b",
        resref: "c_phod_horror_p",
        resource_id: 1_027,
        offset: 264_988_240,
        sha256: "09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f",
        payload_length: 846_064,
        core_length: 788_416,
        raw_length: 57_636,
    },
];

const EXPECTED_CAPABILITIES: [ReferenceCapability; 3] = [
    ReferenceCapability::Header,
    ReferenceCapability::CoreRanges,
    ReferenceCapability::NodeTree,
];

const EXPECTED_INVARIANTS: [&str; 5] = [
    "binary-mdl-id-is-zero",
    "payload-byte-length-exact",
    "core-byte-length-exact",
    "raw-byte-length-exact",
    "file-header-core-raw-cover-payload",
];

fn canonical_manifest() -> ReferenceManifest {
    ReferenceManifest {
        schema_version: 1,
        entries: REFERENCES
            .iter()
            .map(|reference| ReferenceManifestEntry {
                identity: ReferenceIdentity {
                    reference_id: reference.reference_id.to_owned(),
                    source: ReferenceSource::NamedHak {
                        name: "cep3_core1".to_owned(),
                    },
                    resref: reference.resref.to_owned(),
                    resource_type: MDL_RESOURCE_TYPE,
                },
                expected_input: InputFingerprint {
                    algorithm: HashAlgorithm::Sha256,
                    sha256: reference.sha256.to_owned(),
                    byte_length: reference.payload_length,
                },
                expected_capabilities: EXPECTED_CAPABILITIES.to_vec(),
                expected_invariants: EXPECTED_INVARIANTS
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect(),
            })
            .collect(),
    }
}

fn capability_results() -> Vec<CapabilityResult> {
    EXPECTED_CAPABILITIES
        .iter()
        .map(|capability| CapabilityResult {
            capability: *capability,
            status: CapabilityStatus::Pass,
            diagnostics: vec![],
        })
        .collect()
}

fn pass_invariant(name: &str, expected: usize, actual: usize) -> InvariantResult {
    InvariantResult {
        invariant: name.to_owned(),
        status: InvariantStatus::Pass,
        expected: Some(expected.to_string()),
        actual: Some(actual.to_string()),
        diagnostics: vec![],
    }
}

fn invariant_results(
    reference: ExpectedReference,
    report: &InspectionReport,
) -> Vec<InvariantResult> {
    let covered_length = FILE_HEADER_SIZE
        + report.file_header.core_range.length
        + report.file_header.raw_range.length;

    vec![
        pass_invariant(
            "binary-mdl-id-is-zero",
            0,
            report.file_header.binary_mdl_id as usize,
        ),
        pass_invariant(
            "payload-byte-length-exact",
            reference.payload_length,
            report.byte_length,
        ),
        pass_invariant(
            "core-byte-length-exact",
            reference.core_length,
            report.file_header.core_range.length,
        ),
        pass_invariant(
            "raw-byte-length-exact",
            reference.raw_length,
            report.file_header.raw_range.length,
        ),
        pass_invariant(
            "file-header-core-raw-cover-payload",
            reference.payload_length,
            covered_length,
        ),
    ]
}

fn assert_exact_mdl_ranges(reference: ExpectedReference, report: &InspectionReport) {
    assert_eq!(report.file_header.binary_mdl_id, 0, "{}", reference.resref);
    assert_eq!(
        report.byte_length, reference.payload_length,
        "{}",
        reference.resref
    );
    assert_eq!(
        report.file_header.core_range.start, FILE_HEADER_SIZE,
        "{}",
        reference.resref
    );
    assert_eq!(
        report.file_header.core_range.length, reference.core_length,
        "{}",
        reference.resref
    );
    assert_eq!(
        report.file_header.core_range.end,
        FILE_HEADER_SIZE + reference.core_length,
        "{}",
        reference.resref
    );
    assert_eq!(
        report.file_header.raw_range.start,
        FILE_HEADER_SIZE + reference.core_length,
        "{}",
        reference.resref
    );
    assert_eq!(
        report.file_header.raw_range.length, reference.raw_length,
        "{}",
        reference.resref
    );
    assert_eq!(
        report.file_header.raw_range.end, reference.payload_length,
        "{}",
        reference.resref
    );
    assert_eq!(
        FILE_HEADER_SIZE + reference.core_length + reference.raw_length,
        reference.payload_length,
        "{}",
        reference.resref
    );
}

fn assert_slice_is_borrowed_from_container(container: &[u8], payload: &[u8], resref: &str) {
    let container_start = container.as_ptr() as usize;
    let container_end = container_start
        .checked_add(container.len())
        .expect("container pointer range must not overflow");
    let payload_start = payload.as_ptr() as usize;
    let payload_end = payload_start
        .checked_add(payload.len())
        .expect("payload pointer range must not overflow");

    assert!(
        payload_start >= container_start && payload_end <= container_end,
        "{resref} lookup must borrow a subrange of the HAK input"
    );
}

fn assert_no_private_path_or_embedded_bytes(value: &Value, private_path: &str) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                assert!(
                    !matches!(key.to_ascii_lowercase().as_str(), "payload" | "bytes"),
                    "P-REF must not embed resource data under key {key:?}"
                );
                assert_no_private_path_or_embedded_bytes(child, private_path);
            }
        }
        Value::Array(array) => {
            for child in array {
                assert_no_private_path_or_embedded_bytes(child, private_path);
            }
        }
        Value::String(text) => {
            let normalized = text.replace('\\', "/").to_ascii_lowercase();
            assert!(
                !normalized.contains(private_path),
                "P-REF contains the private host path"
            );
        }
        _ => {}
    }
}

#[test]
fn canonical_cep_hak_builds_r1_and_r3_p_ref_packets_without_extracting_payloads() {
    let Some(path) = std::env::var_os("M2A_REFERENCE_CEP_HAK") else {
        eprintln!("skipped: M2A_REFERENCE_CEP_HAK is not set");
        return;
    };

    let hak_bytes = std::fs::read(&path).expect("env-selected CEP HAK must be readable in place");
    let archive = ErfArchive::parse(&hak_bytes)
        .unwrap_or_else(|error| panic!("own ERF reader rejected env-selected CEP HAK: {error}"));
    assert_eq!(archive.file_type(), ErfFileType::Hak);

    let manifest = canonical_manifest();
    let normalized_private_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();

    for reference in REFERENCES {
        let metadata = archive
            .resources()
            .iter()
            .find(|resource| {
                resource.resource_type == MDL_RESOURCE_TYPE && resource.resref == reference.resref
            })
            .unwrap_or_else(|| {
                panic!(
                    "own ERF reader did not expose metadata for ({}, {})",
                    reference.resref, MDL_RESOURCE_TYPE
                )
            });
        assert_eq!(
            metadata.resource_id, reference.resource_id,
            "{} resource id differs from the canonical HAK audit",
            reference.resref
        );
        assert_eq!(
            metadata.offset, reference.offset,
            "{} payload offset differs from the canonical HAK audit",
            reference.resref
        );
        assert_eq!(
            metadata.size, reference.payload_length,
            "{}",
            reference.resref
        );

        let mdl_bytes = archive
            .find(reference.resref, MDL_RESOURCE_TYPE)
            .unwrap_or_else(|error| {
                panic!(
                    "own ERF reader lookup failed for ({}, {}): {error}",
                    reference.resref, MDL_RESOURCE_TYPE
                )
            });
        assert_eq!(
            mdl_bytes.len(),
            reference.payload_length,
            "{}",
            reference.resref
        );
        assert_eq!(
            mdl_bytes.as_ptr(),
            hak_bytes[metadata.offset..metadata.offset + metadata.size].as_ptr(),
            "{} lookup must return the metadata-selected HAK subrange",
            reference.resref
        );
        assert_slice_is_borrowed_from_container(&hak_bytes, mdl_bytes, reference.resref);

        let expected_fingerprint = &manifest
            .entries
            .iter()
            .find(|entry| entry.identity.reference_id == reference.reference_id)
            .expect("manifest entry must exist")
            .expected_input;
        assert_eq!(
            InputFingerprint::from_bytes(mdl_bytes),
            *expected_fingerprint,
            "{} hash/length differs from the current canonical audit",
            reference.resref
        );

        let report = inspect_binary_mdl(mdl_bytes).unwrap_or_else(|error| {
            panic!(
                "own MDL reader rejected canonical {} with exact error: {error}",
                reference.resref
            )
        });
        assert_exact_mdl_ranges(reference, &report);

        let packet = build_reference_proof_packet(
            &manifest,
            reference.reference_id,
            mdl_bytes,
            ExecutionMetadata {
                command_label: "m2a-core-canonical-hak-reference-test".to_owned(),
                timestamp_utc: "2026-07-11T00:00:00Z".to_owned(),
            },
            capability_results(),
            invariant_results(reference, &report),
        )
        .unwrap_or_else(|error| {
            panic!(
                "P-REF construction failed for canonical {}: {error}",
                reference.resref
            )
        });

        assert_eq!(packet.reader_report, report, "{}", reference.resref);
        assert_eq!(packet.input, *expected_fingerprint, "{}", reference.resref);
        assert_eq!(packet.identity.resref, reference.resref);
        assert_eq!(packet.identity.resource_type, MDL_RESOURCE_TYPE);
        assert_eq!(
            packet.identity.source,
            ReferenceSource::NamedHak {
                name: "cep3_core1".to_owned()
            }
        );

        let packet_value = serde_json::to_value(&packet).expect("P-REF must serialize in memory");
        assert_no_private_path_or_embedded_bytes(&packet_value, &normalized_private_path);
    }
}
