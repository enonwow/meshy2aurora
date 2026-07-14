use m2a_core::m7_corpus::{
    M7BatchStatusV1, M7CorpusEntryV1, M7CorpusManifestV1, M7IntakeStatusV1,
    M7PerProfileProofPacketArtifactV1, M7ProofPacketStatusV1, M7StaticResourceKindV1,
    build_m7_corpus_batch_v1, inspect_m7_corpus_intake_v1, parse_m7_corpus_manifest_v1,
    validate_m7_corpus_manifest_v1,
};

fn deferred_manifest() -> M7CorpusManifestV1 {
    M7CorpusManifestV1 {
        schema_version: 1,
        corpus_id: "m7-first-pass".to_owned(),
        art_direction_approval_id: None,
        samples: vec![
            M7CorpusEntryV1::StaticPlaceableOrItem {
                sample_id: "static-prop".to_owned(),
                source: None,
                resource_kind: M7StaticResourceKindV1::Placeable,
            },
            M7CorpusEntryV1::RiggedHumanoidSourceClips {
                sample_id: "humanoid".to_owned(),
                source: None,
                required_source_clip_names: vec!["walk".to_owned()],
            },
            M7CorpusEntryV1::NonHumanoidReferenceSupermodel {
                sample_id: "creature".to_owned(),
                source: None,
                reference_supermodel: "c_horror".to_owned(),
            },
        ],
    }
}

#[test]
fn deferred_intake_and_batch_are_deterministic_and_never_claim_done() {
    let manifest = deferred_manifest();
    validate_m7_corpus_manifest_v1(&manifest).unwrap();

    let intake = inspect_m7_corpus_intake_v1(&manifest, &[]).unwrap();
    assert_eq!(intake.status, M7IntakeStatusV1::InputDeferred);
    assert_eq!(intake.ready_source_count, 0);
    assert!(!intake.real_execution_ready);
    assert!(!intake.m7_done_claim_allowed);
    assert_eq!(intake.samples.len(), 3);

    let first = build_m7_corpus_batch_v1(&manifest, &[], &[]).unwrap();
    let second = build_m7_corpus_batch_v1(&manifest, &[], &[]).unwrap();
    assert_eq!(first.report.status, M7BatchStatusV1::InputDeferred);
    assert_eq!(first.report.packet_count, 3);
    assert_eq!(first.report.deferred_packet_count, 3);
    assert_eq!(first.report.materialized_packet_count, 0);
    assert!(!first.report.m7_done_claim_allowed);
    assert_eq!(first.report_json, second.report_json);
    assert_eq!(packet_json(&first.packets), packet_json(&second.packets));
    for packet in first.packets {
        assert_eq!(packet.packet.status, M7ProofPacketStatusV1::InputDeferred);
        assert!(packet.packet.canonical_outputs.is_empty());
        assert!(!packet.packet.m7_done_claim_allowed);
    }
}

fn packet_json(packets: &[M7PerProfileProofPacketArtifactV1]) -> Vec<&[u8]> {
    packets
        .iter()
        .map(|packet| packet.packet_json.as_slice())
        .collect()
}

#[test]
fn manifest_json_uses_strict_camel_case_fields() {
    let bytes = br#"{
      "schemaVersion":1,
      "corpusId":"m7-first-pass",
      "artDirectionApprovalId":null,
      "samples":[
        {"role":"RIGGED_HUMANOID_SOURCE_CLIPS","sampleId":"humanoid","source":null,"requiredSourceClipNames":["walk"]},
        {"role":"NON_HUMANOID_REFERENCE_SUPERMODEL","sampleId":"creature","source":null,"referenceSupermodel":"c_horror"},
        {"role":"STATIC_PLACEABLE_OR_ITEM","sampleId":"static-prop","source":null,"resourceKind":"PLACEABLE"}
      ]
    }"#;
    let manifest = parse_m7_corpus_manifest_v1(bytes).unwrap();
    assert_eq!(manifest.corpus_id, "m7-first-pass");
    assert_eq!(manifest.samples.len(), 3);

    let with_unknown = String::from_utf8(bytes.to_vec()).unwrap().replacen(
        "\"sampleId\":\"humanoid\"",
        "\"sampleId\":\"humanoid\",\"unknown\":true",
        1,
    );
    let error = parse_m7_corpus_manifest_v1(with_unknown.as_bytes()).unwrap_err();
    assert_eq!(error.code, "M7-MANIFEST-JSON-INVALID");
}

#[test]
fn duplicate_roles_and_malformed_json_have_stable_contract_errors() {
    let mut manifest = deferred_manifest();
    manifest.samples[2] = M7CorpusEntryV1::StaticPlaceableOrItem {
        sample_id: "other-static".to_owned(),
        source: None,
        resource_kind: M7StaticResourceKindV1::Item,
    };
    let error = validate_m7_corpus_manifest_v1(&manifest).unwrap_err();
    assert_eq!(error.code, "M7-CORPUS-ROLE-DUPLICATE");

    let error = parse_m7_corpus_manifest_v1(b"not-json").unwrap_err();
    assert_eq!(error.code, "M7-MANIFEST-JSON-INVALID");
}
