use m2a_core::m7_corpus::{
    M7BatchStatusV1, M7ByteIdentityV1, M7CanonicalPipelineArtifactV1, M7CanonicalRouteV1,
    M7CorpusEntryV1, M7CorpusManifestV1, M7IntakeStatusV1, M7OriginalSourceProvenanceV1,
    M7PerProfileProofPacketArtifactV1, M7ProofPacketStatusV1, M7SourceDescriptorV1,
    M7SourcePayloadV1, M7SourceProviderV1, M7StaticResourceKindV1, build_m7_corpus_batch_v1,
    inspect_m7_corpus_intake_v1, parse_m7_corpus_manifest_v1, validate_m7_corpus_manifest_v1,
};
use sha2::{Digest, Sha256};

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

fn descriptor(path: &str, bytes: &[u8], task: &str) -> M7SourceDescriptorV1 {
    M7SourceDescriptorV1 {
        relative_path: path.to_owned(),
        identity: M7ByteIdentityV1 {
            byte_length: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
        },
        provenance: M7OriginalSourceProvenanceV1 {
            provider: M7SourceProviderV1::Meshy,
            provider_task_id: task.to_owned(),
            original_export_attested: true,
            rights_confirmed: true,
            not_synthetic_fixture_attested: true,
        },
    }
}

fn remove_rig_and_animations(glb: &[u8]) -> Vec<u8> {
    let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let json_end = 20 + json_len;
    let mut json: serde_json::Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
    let root = json.as_object_mut().unwrap();
    root.remove("skins");
    root.remove("animations");
    for node in root["nodes"].as_array_mut().unwrap() {
        node.as_object_mut().unwrap().remove("skin");
    }
    let mut json_bytes = serde_json::to_vec(&json).unwrap();
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    let mut result = Vec::new();
    result.extend_from_slice(b"glTF");
    result.extend_from_slice(&2_u32.to_le_bytes());
    result.extend_from_slice(&0_u32.to_le_bytes());
    result.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(b"JSON");
    result.extend_from_slice(&json_bytes);
    result.extend_from_slice(&glb[json_end..]);
    let total_len = result.len() as u32;
    result[8..12].copy_from_slice(&total_len.to_le_bytes());
    result
}

fn ready_corpus() -> (M7CorpusManifestV1, Vec<u8>, Vec<u8>) {
    let humanoid = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
    let static_glb = remove_rig_and_animations(&humanoid);
    let manifest = M7CorpusManifestV1 {
        schema_version: 1,
        corpus_id: "m7-ready-code-fixture".to_owned(),
        art_direction_approval_id: Some("owner-approved-test-contract".to_owned()),
        samples: vec![
            M7CorpusEntryV1::RiggedHumanoidSourceClips {
                sample_id: "humanoid".to_owned(),
                source: Some(descriptor("models/humanoid.glb", &humanoid, "task-h")),
                required_source_clip_names: vec!["owned-linear-pause".to_owned()],
            },
            M7CorpusEntryV1::NonHumanoidReferenceSupermodel {
                sample_id: "creature".to_owned(),
                source: Some(descriptor("models/creature.glb", &static_glb, "task-c")),
                reference_supermodel: "c_horror".to_owned(),
            },
            M7CorpusEntryV1::StaticPlaceableOrItem {
                sample_id: "static-prop".to_owned(),
                source: Some(descriptor("models/static.glb", &static_glb, "task-s")),
                resource_kind: M7StaticResourceKindV1::Placeable,
            },
        ],
    };
    (manifest, humanoid, static_glb)
}

fn appearance_two_da() -> &'static [u8] {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY\r\n0 Existing NORM P existing **** **** 0 R 1.0 4\r\n"
}

#[test]
fn canonical_artifact_is_bound_to_its_real_source_and_exact_writer_replay() {
    let (manifest, humanoid, static_glb) = ready_corpus();
    let payloads = [
        M7SourcePayloadV1 {
            relative_path: "models/humanoid.glb",
            bytes: &humanoid,
        },
        M7SourcePayloadV1 {
            relative_path: "models/creature.glb",
            bytes: &static_glb,
        },
        M7SourcePayloadV1 {
            relative_path: "models/static.glb",
            bytes: &static_glb,
        },
    ];
    let canonical = [M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6(
        "humanoid",
        &humanoid,
        appearance_two_da(),
    )
    .unwrap()];
    let batch = build_m7_corpus_batch_v1(&manifest, &payloads, &canonical).unwrap();
    assert_eq!(batch.report.materialized_packet_count, 1);
    assert_eq!(batch.report.deferred_packet_count, 2);
    assert!(batch.packets.iter().any(|packet| {
        packet.packet.sample_id == "humanoid"
            && packet.packet.status == M7ProofPacketStatusV1::CanonicalPackageMaterialized
            && packet.packet.canonical_route == M7CanonicalRouteV1::RiggedHumanoidM6
            && packet.packet.canonical_package_readback == "OWN_ERF_READBACK_PASS"
    }));
    assert!(batch.packets.iter().any(|packet| {
        packet.packet.sample_id == "creature"
            && packet.packet.canonical_route
                == M7CanonicalRouteV1::NonHumanoidReferenceSupermodelDeferredM7V5
            && packet.packet.status == M7ProofPacketStatusV1::InputDeferred
            && packet.packet.canonical_package_readback == "ROUTE_DEFERRED_M7_V5"
            && packet
                .packet
                .diagnostics
                .iter()
                .any(|item| item.code == "M7-NON-HUMANOID-CANONICAL-ROUTE-DEFERRED-M7V5")
    }));
    assert!(batch.packets.iter().any(|packet| {
        packet.packet.sample_id == "static-prop"
            && packet.packet.canonical_route
                == M7CanonicalRouteV1::StaticPlaceableOrItemDeferredM7V5
            && packet.packet.status == M7ProofPacketStatusV1::InputDeferred
            && packet.packet.canonical_package_readback == "ROUTE_DEFERRED_M7_V5"
            && packet
                .packet
                .diagnostics
                .iter()
                .any(|item| item.code == "M7-STATIC-CANONICAL-ROUTE-DEFERRED-M7V5")
    }));

    let wrong_sample = [M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6(
        "static-prop",
        &humanoid,
        appearance_two_da(),
    )
    .unwrap()];
    let error = build_m7_corpus_batch_v1(&manifest, &payloads, &wrong_sample).unwrap_err();
    assert_eq!(error.code, "M7-BATCH-ARTIFACT-ROUTE-MISMATCH");
}

#[test]
fn approval_and_exact_resource_metadata_gate_canonical_materialization() {
    let (mut manifest, humanoid, static_glb) = ready_corpus();
    let payloads = [
        M7SourcePayloadV1 {
            relative_path: "models/humanoid.glb",
            bytes: &humanoid,
        },
        M7SourcePayloadV1 {
            relative_path: "models/creature.glb",
            bytes: &static_glb,
        },
        M7SourcePayloadV1 {
            relative_path: "models/static.glb",
            bytes: &static_glb,
        },
    ];
    manifest.art_direction_approval_id = None;
    let canonical = [M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6(
        "humanoid",
        &humanoid,
        appearance_two_da(),
    )
    .unwrap()];
    let error = build_m7_corpus_batch_v1(&manifest, &payloads, &canonical).unwrap_err();
    assert_eq!(error.code, "M7-BATCH-ARTIFACT-WITHOUT-READY-SOURCE");
}

#[test]
fn unsafe_paths_identity_drift_and_truncated_glb_are_rejected() {
    let (mut manifest, humanoid, static_glb) = ready_corpus();
    if let M7CorpusEntryV1::StaticPlaceableOrItem {
        source: Some(source),
        ..
    } = &mut manifest.samples[2]
    {
        source.relative_path = "../static.glb".to_owned();
    }
    assert_eq!(
        validate_m7_corpus_manifest_v1(&manifest).unwrap_err().code,
        "M7-SOURCE-PATH-INVALID"
    );

    let (mut manifest, _, _) = ready_corpus();
    let truncated = b"bad";
    if let M7CorpusEntryV1::StaticPlaceableOrItem {
        source: Some(source),
        ..
    } = &mut manifest.samples[2]
    {
        *source = descriptor("models/static.glb", truncated, "task-s");
    }
    let payloads = [
        M7SourcePayloadV1 {
            relative_path: "models/humanoid.glb",
            bytes: &humanoid,
        },
        M7SourcePayloadV1 {
            relative_path: "models/creature.glb",
            bytes: &static_glb,
        },
        M7SourcePayloadV1 {
            relative_path: "models/static.glb",
            bytes: truncated,
        },
    ];
    let report = inspect_m7_corpus_intake_v1(&manifest, &payloads).unwrap();
    assert_eq!(report.status, M7IntakeStatusV1::InputInvalid);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| item.code == "M7-SOURCE-GLB-INVALID")
    );

    let (manifest, _, _) = ready_corpus();
    let drifted = b"different";
    let payloads = [
        M7SourcePayloadV1 {
            relative_path: "models/humanoid.glb",
            bytes: &humanoid,
        },
        M7SourcePayloadV1 {
            relative_path: "models/creature.glb",
            bytes: &static_glb,
        },
        M7SourcePayloadV1 {
            relative_path: "models/static.glb",
            bytes: drifted,
        },
    ];
    let report = inspect_m7_corpus_intake_v1(&manifest, &payloads).unwrap();
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| item.code == "M7-SOURCE-IDENTITY-MISMATCH")
    );
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
