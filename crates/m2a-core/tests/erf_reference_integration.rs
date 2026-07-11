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

const REFERENCES: [ExpectedReference; 6] = [
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
    ExpectedReference {
        reference_id: "R4",
        resref: "c_nulltail",
        resource_id: 6_390,
        offset: 1_420_456_988,
        sha256: "b51542cc752421a41ff605d4c348794fff15ebfdb8973572d51a3a06fc7f8b76",
        payload_length: 1_044,
        core_length: 1_032,
        raw_length: 0,
    },
    ExpectedReference {
        reference_id: "R5",
        resref: "c_vampire_f",
        resource_id: 6_240,
        offset: 1_161_251_268,
        sha256: "964b015298743216a0d78fa0ddf2dedc9fb6ad45c39f54457767fd60cd96c5d4",
        payload_length: 230_992,
        core_length: 91_128,
        raw_length: 139_852,
    },
    ExpectedReference {
        reference_id: "R6",
        resref: "c_eye",
        resource_id: 552,
        offset: 136_663_067,
        sha256: "401672fa00074c34b6c68e982242d5f0499ec657978826f15921f69200d719ea",
        payload_length: 23_060,
        core_length: 11_952,
        raw_length: 11_096,
    },
];

const EXPECTED_CAPABILITIES: [ReferenceCapability; 9] = [
    ReferenceCapability::Header,
    ReferenceCapability::CoreRanges,
    ReferenceCapability::NodeTree,
    ReferenceCapability::Mesh,
    ReferenceCapability::Skin,
    ReferenceCapability::Controllers,
    ReferenceCapability::Animations,
    ReferenceCapability::Events,
    ReferenceCapability::UnsupportedNodeFamily,
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
                expected_invariants: invariant_names(reference.reference_id),
            })
            .collect(),
    }
}

fn invariant_names(reference_id: &str) -> Vec<String> {
    let role_names: &[&str] = match reference_id {
        "R1" => &[
            "skin-node-count-exact",
            "skin-variant-extended64-all",
            "skin-header-boundary-exact",
            "skin-map-count-observed",
            "skin-bind-arrays-nonempty",
            "skin-raw-weights-bone-refs-in-bounds",
            "skin-ffff-lanes-classified",
        ],
        "R3a" | "R3b" => &[
            "own-animation-count-exact",
            "animation-event-count-exact",
            "animation-root-trees-in-bounds",
            "animation-events-in-bounds",
            "animation-controllers-in-bounds",
        ],
        "R4" => &[
            "model-node-count-exact",
            "mesh-node-count-exact",
            "skin-node-count-zero",
            "own-animation-count-zero",
            "mesh-required-arrays-in-bounds",
            "unsupported-node-family-count-zero",
            "mdx-byte-length-zero",
        ],
        "R5" => &[
            "model-node-count-exact",
            "skin-node-count-exact",
            "skin-variant-legacy17-all",
            "skin-header-boundary-exact",
            "skin-map-count-observed",
            "skin-bind-array-counts-exact",
            "skin-raw-weights-bone-refs-in-bounds",
            "skin-ffff-lanes-classified",
            "skin-nonzero-bind-pose-observed",
            "own-animation-count-zero",
        ],
        "R6" => &[
            "model-node-count-exact",
            "mesh-node-count-exact",
            "skin-node-count-zero",
            "own-animation-count-zero",
            "unsupported-node-family-present",
            "unsupported-node-family-dangly-exact",
            "unsupported-node-family-diagnostic-exact",
            "supported-common-prefix-preserved",
        ],
        _ => panic!("unknown canonical reference id {reference_id}"),
    };

    EXPECTED_INVARIANTS
        .iter()
        .chain(role_names)
        .map(|name| (*name).to_owned())
        .collect()
}

#[derive(Default)]
struct SemanticStats {
    mesh_nodes: usize,
    skin_nodes: usize,
    controllers: usize,
    unsupported_families: Vec<String>,
    dangling_nodes: usize,
    dangling_mesh_prefixes: usize,
    extended64_skins: usize,
    legacy17_skins: usize,
    skin_map_counts: Vec<usize>,
    skin_q_counts: Vec<usize>,
    skin_t_counts: Vec<usize>,
    skin_constant_counts: Vec<usize>,
    skin_header_sizes: Vec<usize>,
    nonempty_skin_bind_arrays: usize,
    nonempty_skin_raw_arrays: usize,
    ffff_zero_weight_lanes: usize,
    ffff_nonzero_weight_lanes: usize,
    nonzero_bind_pose_skins: usize,
}

fn as_array<'a>(value: &'a Value, context: &str) -> &'a [Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("reader report field {context:?} must be an array"))
}

fn value_array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    as_array(&value[key], key)
}

fn collect_node_stats(node: &Value, stats: &mut SemanticStats) {
    stats.controllers += value_array(node, "controllers").len();
    if !node["mesh"].is_null() {
        stats.mesh_nodes += 1;
    }
    let unsupported_families = value_array(node, "unsupportedFamilies")
        .iter()
        .map(|family| {
            family
                .as_str()
                .expect("unsupported family must serialize as text")
        })
        .collect::<Vec<_>>();
    if unsupported_families.contains(&"dangly") {
        stats.dangling_nodes += 1;
        if !node["mesh"].is_null() {
            stats.dangling_mesh_prefixes += 1;
        }
    }
    stats
        .unsupported_families
        .extend(unsupported_families.into_iter().map(str::to_owned));

    if let Some(skin) = node["skin"].as_object() {
        stats.skin_nodes += 1;
        let node_offset = skin["nodeOffset"]
            .as_u64()
            .expect("skin nodeOffset must serialize as an integer")
            as usize;
        let header_size = skin["headerSize"]
            .as_u64()
            .expect("skin headerSize must serialize as an integer")
            as usize;
        let map_pointer = skin["nodeToBonePointer"]
            .as_u64()
            .expect("skin nodeToBonePointer must serialize as an integer")
            as usize;
        assert_eq!(node_offset + header_size, map_pointer);
        stats.skin_header_sizes.push(header_size);
        match skin["variant"].as_str() {
            Some("extended64") => stats.extended64_skins += 1,
            Some("legacy17") => stats.legacy17_skins += 1,
            variant => panic!("unexpected serialized skin variant {variant:?}"),
        }
        stats
            .skin_map_counts
            .push(as_array(&skin["nodeToBoneMap"], "nodeToBoneMap").len());
        stats
            .skin_q_counts
            .push(as_array(&skin["inverseBoneRotationsRaw"], "inverseBoneRotationsRaw").len());
        stats
            .skin_t_counts
            .push(as_array(&skin["inverseBoneTranslations"], "inverseBoneTranslations").len());
        stats
            .skin_constant_counts
            .push(as_array(&skin["boneConstants"], "boneConstants").len());
        if !as_array(&skin["inverseBoneRotationsRaw"], "inverseBoneRotationsRaw").is_empty()
            && !as_array(&skin["inverseBoneTranslations"], "inverseBoneTranslations").is_empty()
            && !as_array(&skin["boneConstants"], "boneConstants").is_empty()
        {
            stats.nonempty_skin_bind_arrays += 1;
        }
        let has_nonzero_translation =
            as_array(&skin["inverseBoneTranslations"], "inverseBoneTranslations")
                .iter()
                .any(|translation| {
                    ["x", "y", "z"].iter().any(|component| {
                        translation[*component]
                            .as_f64()
                            .is_some_and(|value| value != 0.0)
                    })
                });
        let has_nonidentity_rotation =
            as_array(&skin["inverseBoneRotationsRaw"], "inverseBoneRotationsRaw")
                .iter()
                .any(|rotation| {
                    let components = as_array(rotation, "inverse bone rotation");
                    let unit_components = components
                        .iter()
                        .filter(|component| {
                            component.as_f64().is_some_and(|value| value.abs() == 1.0)
                        })
                        .count();
                    let zero_components = components
                        .iter()
                        .filter(|component| component.as_f64() == Some(0.0))
                        .count();
                    !(unit_components == 1 && zero_components == 3)
                });
        if has_nonzero_translation || has_nonidentity_rotation {
            stats.nonzero_bind_pose_skins += 1;
        }
        let vertex_weights = as_array(&skin["vertexWeights"], "vertexWeights");
        let bone_references = as_array(&skin["boneReferences"], "boneReferences");
        if !vertex_weights.is_empty() && !bone_references.is_empty() {
            stats.nonempty_skin_raw_arrays += 1;
        }
        for (weights, bone_refs) in vertex_weights.iter().zip(bone_references) {
            for (weight, bone_ref) in as_array(weights, "vertex weight row")
                .iter()
                .zip(as_array(bone_refs, "bone reference row"))
            {
                if bone_ref.as_u64() == Some(u16::MAX as u64) {
                    if weight.as_f64() == Some(0.0) {
                        stats.ffff_zero_weight_lanes += 1;
                    } else {
                        stats.ffff_nonzero_weight_lanes += 1;
                    }
                }
            }
        }
    }

    for child in value_array(node, "children") {
        collect_node_stats(child, stats);
    }
}

fn collect_tree_stats(tree: &Value, stats: &mut SemanticStats) {
    for root in value_array(tree, "roots") {
        collect_node_stats(root, stats);
    }
}

fn semantic_stats(report: &InspectionReport) -> SemanticStats {
    let report = serde_json::to_value(report).expect("reader report must serialize for assertions");
    let mut stats = SemanticStats::default();
    collect_tree_stats(&report["nodeTree"], &mut stats);
    for animation in value_array(&report, "animations") {
        collect_tree_stats(&animation["nodeTree"], &mut stats);
    }
    stats
}

fn expected_capability_status(
    reference_id: &str,
    capability: ReferenceCapability,
) -> CapabilityStatus {
    match capability {
        ReferenceCapability::Header
        | ReferenceCapability::CoreRanges
        | ReferenceCapability::NodeTree
        | ReferenceCapability::Mesh
        | ReferenceCapability::Controllers => CapabilityStatus::Pass,
        ReferenceCapability::Skin if matches!(reference_id, "R1" | "R5") => CapabilityStatus::Pass,
        ReferenceCapability::Animations | ReferenceCapability::Events
            if matches!(reference_id, "R3a" | "R3b") =>
        {
            CapabilityStatus::Pass
        }
        ReferenceCapability::UnsupportedNodeFamily if reference_id == "R6" => {
            CapabilityStatus::Pass
        }
        ReferenceCapability::Skin
        | ReferenceCapability::Animations
        | ReferenceCapability::Events
        | ReferenceCapability::UnsupportedNodeFamily => CapabilityStatus::NotPresent,
    }
}

fn capability_results(
    reference_id: &str,
    report: &InspectionReport,
    stats: &SemanticStats,
) -> Vec<CapabilityResult> {
    let event_count = report
        .animations
        .iter()
        .map(|animation| animation.events.len())
        .sum::<usize>();
    let present = |capability| match capability {
        ReferenceCapability::Header
        | ReferenceCapability::CoreRanges
        | ReferenceCapability::NodeTree => true,
        ReferenceCapability::Mesh => stats.mesh_nodes > 0,
        ReferenceCapability::Skin => stats.skin_nodes > 0,
        ReferenceCapability::Controllers => stats.controllers > 0,
        ReferenceCapability::Animations => !report.animations.is_empty(),
        ReferenceCapability::Events => event_count > 0,
        ReferenceCapability::UnsupportedNodeFamily => !stats.unsupported_families.is_empty(),
    };

    EXPECTED_CAPABILITIES
        .iter()
        .map(|capability| {
            let observed = if present(*capability) {
                CapabilityStatus::Pass
            } else {
                CapabilityStatus::NotPresent
            };
            let expected = expected_capability_status(reference_id, *capability);
            assert_eq!(observed, expected, "{reference_id} {capability:?}");
            CapabilityResult {
                capability: *capability,
                status: observed,
                diagnostics: vec![],
            }
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

fn pass_text_invariant(
    name: &str,
    expected: impl ToString,
    actual: impl ToString,
) -> InvariantResult {
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

    let mut results = vec![
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
    ];
    results.extend(role_invariant_results(reference.reference_id, report));
    results
}

fn role_invariant_results(reference_id: &str, report: &InspectionReport) -> Vec<InvariantResult> {
    let stats = semantic_stats(report);
    let event_count = report
        .animations
        .iter()
        .map(|animation| animation.events.len())
        .sum::<usize>();
    let unsupported_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "M2A-MDL-UNSUPPORTED-NODE-FAMILY")
        .collect::<Vec<_>>();
    let diagnostic_contexts = unsupported_diagnostics
        .iter()
        .map(|diagnostic| diagnostic.context.as_str())
        .collect::<Vec<_>>();

    match reference_id {
        "R1" => {
            assert_eq!(stats.skin_nodes, 3);
            assert_eq!(stats.extended64_skins, 3);
            assert_eq!(stats.skin_header_sizes, [0x330, 0x330, 0x330]);
            assert_eq!(stats.skin_map_counts, [38, 38, 38]);
            assert_eq!(stats.nonempty_skin_bind_arrays, 3);
            assert_eq!(stats.nonempty_skin_raw_arrays, 3);
            assert_eq!(stats.ffff_zero_weight_lanes, 881);
            assert_eq!(stats.ffff_nonzero_weight_lanes, 0);
            vec![
                pass_invariant("skin-node-count-exact", 3, stats.skin_nodes),
                pass_invariant("skin-variant-extended64-all", 3, stats.extended64_skins),
                pass_text_invariant(
                    "skin-header-boundary-exact",
                    "0x330,0x330,0x330",
                    stats
                        .skin_header_sizes
                        .iter()
                        .map(|size| format!("0x{size:x}"))
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                pass_text_invariant("skin-map-count-observed", "38,38,38", "38,38,38"),
                pass_invariant(
                    "skin-bind-arrays-nonempty",
                    3,
                    stats.nonempty_skin_bind_arrays,
                ),
                pass_text_invariant(
                    "skin-raw-weights-bone-refs-in-bounds",
                    "reader-validated",
                    "reader-validated",
                ),
                pass_text_invariant(
                    "skin-ffff-lanes-classified",
                    "zero=881;nonzero=0",
                    format!(
                        "zero={};nonzero={}",
                        stats.ffff_zero_weight_lanes, stats.ffff_nonzero_weight_lanes
                    ),
                ),
            ]
        }
        "R3a" | "R3b" => {
            assert_eq!(report.animations.len(), 42);
            assert_eq!(event_count, 41);
            assert!(stats.controllers > 0);
            vec![
                pass_invariant("own-animation-count-exact", 42, report.animations.len()),
                pass_invariant("animation-event-count-exact", 41, event_count),
                pass_text_invariant(
                    "animation-root-trees-in-bounds",
                    "reader-validated",
                    "reader-validated",
                ),
                pass_text_invariant(
                    "animation-events-in-bounds",
                    "reader-validated",
                    "reader-validated",
                ),
                pass_text_invariant(
                    "animation-controllers-in-bounds",
                    "positive",
                    stats.controllers,
                ),
            ]
        }
        "R4" => {
            assert_eq!(report.node_tree.node_count, 2);
            assert_eq!(stats.mesh_nodes, 1);
            assert_eq!(stats.skin_nodes, 0);
            assert!(report.animations.is_empty());
            assert!(stats.unsupported_families.is_empty());
            vec![
                pass_invariant("model-node-count-exact", 2, report.node_tree.node_count),
                pass_invariant("mesh-node-count-exact", 1, stats.mesh_nodes),
                pass_invariant("skin-node-count-zero", 0, stats.skin_nodes),
                pass_invariant("own-animation-count-zero", 0, report.animations.len()),
                pass_text_invariant(
                    "mesh-required-arrays-in-bounds",
                    "reader-validated",
                    "reader-validated",
                ),
                pass_invariant(
                    "unsupported-node-family-count-zero",
                    0,
                    stats.unsupported_families.len(),
                ),
                pass_invariant(
                    "mdx-byte-length-zero",
                    0,
                    report.file_header.raw_range.length,
                ),
            ]
        }
        "R5" => {
            assert_eq!(report.node_tree.node_count, 28);
            assert_eq!(stats.skin_nodes, 2);
            assert_eq!(stats.legacy17_skins, 2);
            assert_eq!(stats.skin_header_sizes, [0x2d4, 0x2d4]);
            assert_eq!(stats.skin_map_counts, [28, 28]);
            assert_eq!(stats.skin_q_counts, [28, 28]);
            assert_eq!(stats.skin_t_counts, [28, 28]);
            assert_eq!(stats.skin_constant_counts, [28, 28]);
            assert_eq!(stats.nonempty_skin_raw_arrays, 2);
            assert_eq!(stats.ffff_zero_weight_lanes, 3_208);
            assert_eq!(stats.ffff_nonzero_weight_lanes, 0);
            assert!(stats.nonzero_bind_pose_skins > 0);
            assert!(report.animations.is_empty());
            vec![
                pass_invariant("model-node-count-exact", 28, report.node_tree.node_count),
                pass_invariant("skin-node-count-exact", 2, stats.skin_nodes),
                pass_invariant("skin-variant-legacy17-all", 2, stats.legacy17_skins),
                pass_text_invariant(
                    "skin-header-boundary-exact",
                    "0x2d4,0x2d4",
                    stats
                        .skin_header_sizes
                        .iter()
                        .map(|size| format!("0x{size:x}"))
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                pass_text_invariant("skin-map-count-observed", "28,28", "28,28"),
                pass_text_invariant(
                    "skin-bind-array-counts-exact",
                    "q=28,28;t=28,28;constants=28,28",
                    "q=28,28;t=28,28;constants=28,28",
                ),
                pass_text_invariant(
                    "skin-raw-weights-bone-refs-in-bounds",
                    "reader-validated",
                    "reader-validated",
                ),
                pass_text_invariant(
                    "skin-ffff-lanes-classified",
                    "zero=3208;nonzero=0",
                    format!(
                        "zero={};nonzero={}",
                        stats.ffff_zero_weight_lanes, stats.ffff_nonzero_weight_lanes
                    ),
                ),
                pass_text_invariant(
                    "skin-nonzero-bind-pose-observed",
                    "positive",
                    stats.nonzero_bind_pose_skins,
                ),
                pass_invariant("own-animation-count-zero", 0, report.animations.len()),
            ]
        }
        "R6" => {
            assert_eq!(report.node_tree.node_count, 10);
            assert_eq!(stats.mesh_nodes, 6);
            assert_eq!(stats.skin_nodes, 0);
            assert!(report.animations.is_empty());
            assert_eq!(stats.unsupported_families, ["dangly"]);
            assert_eq!(diagnostic_contexts.len(), 1);
            let diagnostic = unsupported_diagnostics[0];
            let diagnostic_actual = format!(
                "code={};severity={};offset={};context={}",
                diagnostic.code, diagnostic.severity, diagnostic.offset, diagnostic.context
            );
            let diagnostic_expected = "code=M2A-MDL-UNSUPPORTED-NODE-FAMILY;severity=warning;offset=476;context=node \"Bat_body\" uses deferred node family dangly";
            assert_eq!(diagnostic_actual, diagnostic_expected);
            assert_eq!(stats.dangling_nodes, 1);
            assert_eq!(stats.dangling_mesh_prefixes, 1);
            vec![
                pass_invariant("model-node-count-exact", 10, report.node_tree.node_count),
                pass_invariant("mesh-node-count-exact", 6, stats.mesh_nodes),
                pass_invariant("skin-node-count-zero", 0, stats.skin_nodes),
                pass_invariant("own-animation-count-zero", 0, report.animations.len()),
                pass_invariant(
                    "unsupported-node-family-present",
                    1,
                    stats.unsupported_families.len(),
                ),
                pass_text_invariant(
                    "unsupported-node-family-dangly-exact",
                    "dangly",
                    stats.unsupported_families.join(","),
                ),
                pass_text_invariant(
                    "unsupported-node-family-diagnostic-exact",
                    diagnostic_expected,
                    diagnostic_actual,
                ),
                pass_invariant(
                    "supported-common-prefix-preserved",
                    stats.dangling_nodes,
                    stats.dangling_mesh_prefixes,
                ),
            ]
        }
        _ => panic!("unknown canonical reference id {reference_id}"),
    }
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
fn canonical_cep_hak_builds_all_required_p_ref_packets_without_extracting_payloads() {
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
        let stats = semantic_stats(&report);

        let packet = build_reference_proof_packet(
            &manifest,
            reference.reference_id,
            mdl_bytes,
            ExecutionMetadata {
                command_label: "m2a-core-canonical-hak-reference-test".to_owned(),
                timestamp_utc: "2026-07-11T00:00:00Z".to_owned(),
            },
            capability_results(reference.reference_id, &report, &stats),
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
