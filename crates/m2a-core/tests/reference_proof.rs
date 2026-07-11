#[path = "fixtures/build_minimal_binary_mdl.rs"]
#[allow(dead_code)]
mod fixtures;

use std::collections::BTreeSet;

use m2a_core::{
    CapabilityResult, CapabilityStatus, ExecutionMetadata, HashAlgorithm, InputFingerprint,
    InvariantResult, InvariantStatus, ReferenceCapability, ReferenceIdentity, ReferenceManifest,
    ReferenceManifestEntry, ReferenceSource, build_reference_proof_packet,
};
use serde_json::Value;

use fixtures::build_minimal_binary_mdl;

const TIMESTAMP: &str = "2026-07-11T12:34:56Z";

fn synthetic_identity() -> ReferenceIdentity {
    ReferenceIdentity {
        reference_id: "R0".to_owned(),
        source: ReferenceSource::Synthetic,
        resref: "m2a_minimal".to_owned(),
        resource_type: 2002,
    }
}

fn capabilities() -> Vec<CapabilityResult> {
    vec![
        CapabilityResult {
            capability: ReferenceCapability::Header,
            status: CapabilityStatus::Pass,
            diagnostics: vec![],
        },
        CapabilityResult {
            capability: ReferenceCapability::NodeTree,
            status: CapabilityStatus::Pass,
            diagnostics: vec![],
        },
    ]
}

fn invariants() -> Vec<InvariantResult> {
    vec![InvariantResult {
        invariant: "binary-mdl-id-is-zero".to_owned(),
        status: InvariantStatus::Pass,
        expected: Some("0".to_owned()),
        actual: Some("0".to_owned()),
        diagnostics: vec![],
    }]
}

fn manifest_for(bytes: &[u8]) -> ReferenceManifest {
    ReferenceManifest {
        schema_version: 1,
        entries: vec![ReferenceManifestEntry {
            identity: synthetic_identity(),
            expected_input: InputFingerprint::from_bytes(bytes),
            expected_capabilities: vec![ReferenceCapability::Header, ReferenceCapability::NodeTree],
            expected_invariants: vec!["binary-mdl-id-is-zero".to_owned()],
        }],
    }
}

fn execution() -> ExecutionMetadata {
    ExecutionMetadata {
        command_label: "m2a-core-inspect-reference".to_owned(),
        timestamp_utc: TIMESTAMP.to_owned(),
    }
}

fn build(
    manifest: &ReferenceManifest,
    bytes: &[u8],
) -> Result<m2a_core::ReferenceProofPacket, m2a_core::ReferenceProofError> {
    build_reference_proof_packet(
        manifest,
        "R0",
        bytes,
        execution(),
        capabilities(),
        invariants(),
    )
}

#[test]
fn manifest_binds_packet_to_identity_hash_length_reader_and_execution() {
    let bytes = build_minimal_binary_mdl();
    let manifest = manifest_for(&bytes);
    let packet = build(&manifest, &bytes).unwrap();

    assert_eq!(packet.identity, manifest.entries[0].identity);
    assert_eq!(packet.input, manifest.entries[0].expected_input);
    assert_eq!(packet.packet_id, "P-REF-R0");
    assert_eq!(packet.schema_version, 1);
    assert_eq!(packet.reader_report.model.name, "m2a_minimal");
    assert_eq!(packet.reader_report.byte_length, bytes.len());
    assert_eq!(packet.reader.report_schema_version, 1);
    assert_eq!(packet.execution.command_label, "m2a-core-inspect-reference");
    assert_eq!(packet.execution.timestamp_utc, TIMESTAMP);
}

#[test]
fn sha256_fingerprint_matches_standard_vector() {
    let fingerprint = InputFingerprint::from_bytes(b"abc");

    assert_eq!(fingerprint.byte_length, 3);
    assert_eq!(fingerprint.algorithm, HashAlgorithm::Sha256);
    assert_eq!(
        fingerprint.sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn packet_rejects_same_length_different_content() {
    let original = build_minimal_binary_mdl();
    let manifest = manifest_for(&original);
    let mut changed = original.clone();
    changed[0x0c + 0x08] = b'x';

    assert_eq!(changed.len(), original.len());
    let error = build(&manifest, &changed).unwrap_err();

    assert_eq!(error.code, "M2A-PREF-INPUT-FINGERPRINT-MISMATCH");
}

#[test]
fn builder_runs_reader_on_the_exact_hashed_bytes() {
    let bytes = build_minimal_binary_mdl();
    let packet = build(&manifest_for(&bytes), &bytes).unwrap();

    assert_eq!(packet.input, InputFingerprint::from_bytes(&bytes));
    assert_eq!(packet.reader_report.byte_length, packet.input.byte_length);
    assert_eq!(packet.reader_report.model.name, "m2a_minimal");
}

#[test]
fn supported_manifest_schema_is_enforced() {
    let bytes = build_minimal_binary_mdl();
    let mut manifest = manifest_for(&bytes);
    manifest.schema_version = 2;

    let error = build(&manifest, &bytes).unwrap_err();

    assert_eq!(error.code, "M2A-PREF-SCHEMA-UNSUPPORTED");
}

#[test]
fn mdl_proof_builder_rejects_a_non_mdl_resource_type() {
    let bytes = build_minimal_binary_mdl();
    let mut manifest = manifest_for(&bytes);
    manifest.entries[0].identity.resource_type = 2001;

    let error = build(&manifest, &bytes).unwrap_err();

    assert_eq!(error.code, "M2A-PREF-RESOURCE-TYPE-UNSUPPORTED");
}

#[test]
fn capabilities_and_invariants_must_be_nonempty_unique_and_cover_manifest() {
    let bytes = build_minimal_binary_mdl();
    let manifest = manifest_for(&bytes);

    let cases = [
        (vec![], invariants(), "M2A-PREF-CAPABILITIES-EMPTY"),
        (
            vec![capabilities()[0].clone(), capabilities()[0].clone()],
            invariants(),
            "M2A-PREF-CAPABILITY-DUPLICATE",
        ),
        (
            vec![capabilities()[0].clone()],
            invariants(),
            "M2A-PREF-CAPABILITY-COVERAGE-MISMATCH",
        ),
        (capabilities(), vec![], "M2A-PREF-INVARIANTS-EMPTY"),
        (
            capabilities(),
            vec![invariants()[0].clone(), invariants()[0].clone()],
            "M2A-PREF-INVARIANT-DUPLICATE",
        ),
        (
            capabilities(),
            vec![InvariantResult {
                invariant: "unexpected-invariant".to_owned(),
                ..invariants()[0].clone()
            }],
            "M2A-PREF-INVARIANT-COVERAGE-MISMATCH",
        ),
    ];

    for (capability_results, invariant_results, expected_code) in cases {
        let error = build_reference_proof_packet(
            &manifest,
            "R0",
            &bytes,
            execution(),
            capability_results,
            invariant_results,
        )
        .unwrap_err();
        assert_eq!(error.code, expected_code);
    }
}

#[test]
fn manifest_expectations_must_be_nonempty_and_unique() {
    let bytes = build_minimal_binary_mdl();
    let mut manifest = manifest_for(&bytes);
    manifest.entries[0].expected_capabilities.clear();
    assert_eq!(
        build(&manifest, &bytes).unwrap_err().code,
        "M2A-PREF-MANIFEST-CAPABILITIES-EMPTY"
    );

    let mut manifest = manifest_for(&bytes);
    manifest.entries[0].expected_capabilities =
        vec![ReferenceCapability::Header, ReferenceCapability::Header];
    assert_eq!(
        build(&manifest, &bytes).unwrap_err().code,
        "M2A-PREF-MANIFEST-CAPABILITY-DUPLICATE"
    );

    let mut manifest = manifest_for(&bytes);
    manifest.entries[0].expected_invariants.clear();
    assert_eq!(
        build(&manifest, &bytes).unwrap_err().code,
        "M2A-PREF-MANIFEST-INVARIANTS-EMPTY"
    );

    let mut manifest = manifest_for(&bytes);
    manifest.entries[0].expected_invariants = vec!["same".to_owned(), "same".to_owned()];
    assert_eq!(
        build(&manifest, &bytes).unwrap_err().code,
        "M2A-PREF-MANIFEST-INVARIANT-DUPLICATE"
    );
}

#[test]
fn logical_identity_and_source_reject_private_paths_and_separators() {
    let bytes = build_minimal_binary_mdl();
    let invalid_identities = [
        ReferenceIdentity {
            reference_id: "C:\\private\\R0".to_owned(),
            ..synthetic_identity()
        },
        ReferenceIdentity {
            resref: "models/private".to_owned(),
            ..synthetic_identity()
        },
        ReferenceIdentity {
            source: ReferenceSource::NamedHak {
                name: "C:\\Games\\private.hak".to_owned(),
            },
            ..synthetic_identity()
        },
        ReferenceIdentity {
            source: ReferenceSource::DirectFile {
                label: "../private/model.mdl".to_owned(),
            },
            ..synthetic_identity()
        },
    ];

    for identity in invalid_identities {
        let mut manifest = manifest_for(&bytes);
        manifest.entries[0].identity = identity;
        let error = build(&manifest, &bytes).unwrap_err();
        assert_eq!(error.code, "M2A-PREF-IDENTITY-INVALID");
    }
}

#[test]
fn execution_requires_a_safe_command_label_and_valid_utc_timestamp() {
    let bytes = build_minimal_binary_mdl();
    let manifest = manifest_for(&bytes);

    for command_label in ["C:\\tools\\reader.exe", "reader --out private/file"] {
        let error = build_reference_proof_packet(
            &manifest,
            "R0",
            &bytes,
            ExecutionMetadata {
                command_label: command_label.to_owned(),
                timestamp_utc: TIMESTAMP.to_owned(),
            },
            capabilities(),
            invariants(),
        )
        .unwrap_err();
        assert_eq!(error.code, "M2A-PREF-COMMAND-LABEL-INVALID");
    }

    for timestamp_utc in [
        "2026-07-11T12:34:56+02:00",
        "2026-02-30T12:34:56Z",
        "2026-07-11 12:34:56Z",
    ] {
        let error = build_reference_proof_packet(
            &manifest,
            "R0",
            &bytes,
            ExecutionMetadata {
                command_label: "reader".to_owned(),
                timestamp_utc: timestamp_utc.to_owned(),
            },
            capabilities(),
            invariants(),
        )
        .unwrap_err();
        assert_eq!(error.code, "M2A-PREF-TIMESTAMP-INVALID");
    }
}

#[test]
fn capability_and_invariant_statuses_have_stable_json_values() {
    let bytes = build_minimal_binary_mdl();
    let mut manifest = manifest_for(&bytes);
    manifest.entries[0].expected_capabilities =
        vec![ReferenceCapability::Skin, ReferenceCapability::Animations];
    manifest.entries[0].expected_invariants = vec!["skin-layout-confirmed".to_owned()];

    let packet = build_reference_proof_packet(
        &manifest,
        "R0",
        &bytes,
        execution(),
        vec![
            CapabilityResult {
                capability: ReferenceCapability::Skin,
                status: CapabilityStatus::Unsupported,
                diagnostics: vec!["M2A-MDL-UNSUPPORTED".to_owned()],
            },
            CapabilityResult {
                capability: ReferenceCapability::Animations,
                status: CapabilityStatus::NotPresent,
                diagnostics: vec![],
            },
        ],
        vec![InvariantResult {
            invariant: "skin-layout-confirmed".to_owned(),
            status: InvariantStatus::NotEvaluated,
            expected: None,
            actual: None,
            diagnostics: vec!["GB-001-SKIN".to_owned()],
        }],
    )
    .unwrap();

    let json = serde_json::to_value(packet).unwrap();
    assert_eq!(json["capabilityResults"][0]["status"], "UNSUPPORTED");
    assert_eq!(json["capabilityResults"][1]["status"], "NOT_PRESENT");
    assert_eq!(json["invariantResults"][0]["status"], "NOT_EVALUATED");
    assert_eq!(json["reader"]["name"], "m2a-core::mdl");
    assert_eq!(json["reader"]["reportSchemaVersion"], 1);
}

#[test]
fn serialized_manifest_and_packet_recursively_exclude_payload_and_bytes_keys() {
    fn forbidden_keys(value: &Value, found: &mut BTreeSet<String>) {
        match value {
            Value::Object(object) => {
                for (key, child) in object {
                    if matches!(key.to_ascii_lowercase().as_str(), "payload" | "bytes") {
                        found.insert(key.clone());
                    }
                    forbidden_keys(child, found);
                }
            }
            Value::Array(array) => {
                for child in array {
                    forbidden_keys(child, found);
                }
            }
            _ => {}
        }
    }

    let bytes = build_minimal_binary_mdl();
    let manifest = manifest_for(&bytes);
    let packet = build(&manifest, &bytes).unwrap();
    let combined = serde_json::json!({
        "manifest": manifest,
        "packet": packet,
    });
    let mut found = BTreeSet::new();
    forbidden_keys(&combined, &mut found);

    assert!(found.is_empty(), "forbidden embedded data keys: {found:?}");
}

#[test]
fn direct_file_smoke_skips_cleanly_without_environment_variable() {
    let Some(path) = std::env::var_os("M2A_REFERENCE_MDL_FILE") else {
        eprintln!("skipped: M2A_REFERENCE_MDL_FILE is not set");
        return;
    };

    let bytes = std::fs::read(&path).expect("env-selected reference MDL must be readable");
    let identity = ReferenceIdentity {
        reference_id: "DIRECT".to_owned(),
        source: ReferenceSource::DirectFile {
            label: "env-selected-mdl".to_owned(),
        },
        resref: "selected_model".to_owned(),
        resource_type: 2002,
    };
    let manifest = ReferenceManifest {
        schema_version: 1,
        entries: vec![ReferenceManifestEntry {
            identity,
            expected_input: InputFingerprint::from_bytes(&bytes),
            expected_capabilities: vec![ReferenceCapability::Header],
            expected_invariants: vec!["parses".to_owned()],
        }],
    };
    let packet = build_reference_proof_packet(
        &manifest,
        "DIRECT",
        &bytes,
        execution(),
        vec![CapabilityResult {
            capability: ReferenceCapability::Header,
            status: CapabilityStatus::Pass,
            diagnostics: vec![],
        }],
        vec![InvariantResult {
            invariant: "parses".to_owned(),
            status: InvariantStatus::Pass,
            expected: Some("true".to_owned()),
            actual: Some("true".to_owned()),
            diagnostics: vec![],
        }],
    )
    .unwrap();

    assert_eq!(packet.input.byte_length, bytes.len());
    let json = serde_json::to_string(&packet).unwrap();
    let private_path = path.to_string_lossy();
    assert!(!json.contains(private_path.as_ref()));
}
