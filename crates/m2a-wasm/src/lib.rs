use wasm_bindgen::prelude::*;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileAJsonInputError<'a> {
    schema_version: u32,
    code: &'a str,
    severity: &'a str,
    path: &'a str,
    message: &'a str,
}

const SERIALIZATION_ERROR_JSON: &str = concat!(
    r#"{"schemaVersion":1,"code":"M2A-JSON-SERIALIZATION","severity":"error","offset":0,"context":""#,
    "WASM adapter JSON serialization",
    r#""}"#,
);

/// Inspects an Aurora/NWN binary MDL selected by JavaScript.
///
/// `wasm-bindgen` maps the borrowed byte slice to a JavaScript `Uint8Array`.
/// The adapter deliberately owns only the JS/WASM boundary and JSON encoding;
/// all format parsing remains in `m2a-core`.
#[wasm_bindgen(js_name = inspectBinaryMdl)]
pub fn inspect_binary_mdl(bytes: &[u8]) -> String {
    let result = m2a_core::inspect_binary_mdl(bytes);

    match result {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Inspects a GLB selected by JavaScript with the default project guardrails.
///
/// This adapter owns only the JS/WASM boundary and JSON encoding. GLB parsing,
/// validation, gates and diagnostics remain exclusively in `m2a-core`.
#[wasm_bindgen(js_name = inspectGlbJson)]
pub fn inspect_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::inspect_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Backward-compatible alias for the original M2 WASM export.
#[wasm_bindgen(js_name = inspectGlb)]
pub fn inspect_glb(bytes: &[u8]) -> String {
    inspect_glb_json(bytes)
}

/// Ingests a GLB selected by JavaScript into source-preserving AuroraAssetIR.
///
/// The adapter deliberately delegates the complete operation to `m2a-core`.
#[wasm_bindgen(js_name = ingestGlbJson)]
pub fn ingest_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

/// Backward-compatible alias for the original M2 WASM export.
#[wasm_bindgen(js_name = ingestGlb)]
pub fn ingest_glb(bytes: &[u8]) -> String {
    ingest_glb_json(bytes)
}

/// Converts a Meshy-style GLB with an explicit clean-room Profile A rig.
///
/// The JSON boundary is strict (`deny_unknown_fields` is defined by the core
/// input types). GLB ingestion and every transformation remain in `m2a-core`;
/// this adapter only parses public JSON and serializes the deterministic core
/// outcome or fatal error.
#[wasm_bindgen(js_name = convertProfileAGlbJson)]
pub fn convert_profile_a_glb_json(bytes: &[u8], rig_json: &str, options_json: &str) -> String {
    let rig = match serde_json::from_str::<m2a_core::profile_a::CreatureRigProfileV1>(rig_json) {
        Ok(value) => value,
        Err(_) => {
            return serialize_json(&ProfileAJsonInputError {
                schema_version: 1,
                code: "M3A-PROFILE-JSON-INVALID",
                severity: "FATAL",
                path: "rigJson",
                message: "rig profile JSON does not match the public schema",
            });
        }
    };
    let options = match serde_json::from_str::<m2a_core::profile_a::ProfileAOptionsV1>(options_json)
    {
        Ok(value) => value,
        Err(_) => {
            return serialize_json(&ProfileAJsonInputError {
                schema_version: 1,
                code: "M3A-OPTIONS-INVALID",
                severity: "FATAL",
                path: "optionsJson",
                message: "options JSON does not match the public schema",
            });
        }
    };
    let source = match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(value) => value,
        Err(error) => return serialize_json(&error),
    };
    match m2a_core::profile_a::convert_profile_a(&source, &rig, &options) {
        Ok(outcome) => serialize_json(&outcome),
        Err(error) => serialize_json(&error),
    }
}

/// Concise alias retained for callers that adopted the initial M3 adapter name.
#[wasm_bindgen(js_name = convertProfileAJson)]
pub fn convert_profile_a_json(bytes: &[u8], rig_json: &str, options_json: &str) -> String {
    convert_profile_a_glb_json(bytes, rig_json, options_json)
}

fn serialize_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| SERIALIZATION_ERROR_JSON.to_owned())
}

#[cfg(test)]
mod profile_a_test_support {
    use m2a_core::profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        ProfileAOptionsV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_profile_sha256,
    };
    use sha2::{Digest, Sha256};

    pub const RIGID_JSON_SHA256: &str =
        "bb1a7a8564be2938bc694b1ffb928e11b904aa62b61b2687bb8a0013bc6c10a1";
    pub const RIGID_JSON_LENGTH: usize = 3186;
    pub const SKIN_JSON_SHA256: &str =
        "e474aa01d1108e2278e6cfaf6c9d2b71b71e4e393b55dbc729b7c8c4c6a8d9dd";
    pub const SKIN_JSON_LENGTH: usize = 3562;
    pub const LIMIT_FATAL_JSON_SHA256: &str =
        "3bfb45cf36af0d4af174cea656ab669714a55c4c88a9661c7ea573be75bec4a2";
    pub const LIMIT_FATAL_JSON_LENGTH: usize = 162;
    pub const MALFORMED_JSON_SHA256: &str =
        "03bd6ebd5cdb45f738de87363d3ad6a95de9bbb5aa119a2fce3199255b8efa55";
    pub const MALFORMED_JSON_LENGTH: usize = 151;

    pub fn identity() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn base_profile() -> CreatureRigProfileV1 {
        CreatureRigProfileV1 {
            schema_version: 1,
            profile_id: "synthetic-rigid-axis-profile".to_owned(),
            content_sha256: String::new(),
            provenance: RigProvenanceV1 {
                kind: RigProvenanceKindV1::Synthetic,
                export_allowed: true,
                attestations: RigProvenanceAttestationsV1 {
                    controlled_construction: true,
                    no_reference_payload_copied: true,
                    rights_confirmed: true,
                },
            },
            target_bounds: Bounds3V1 {
                min: [-10.0, -10.0, 0.0],
                max: [10.0, 10.0, 1.0],
            },
            alignment_anchor: [0.0, 0.0, 0.0],
            nodes: vec![CreatureRigNodeV1 {
                id: 70,
                name: "synthetic-rigid-root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity(),
            }],
            segments: Vec::new(),
        }
    }

    fn finish(mut profile: CreatureRigProfileV1) -> CreatureRigProfileV1 {
        profile.content_sha256 = canonical_profile_sha256(&profile).unwrap();
        profile
    }

    pub fn rigid_profile() -> CreatureRigProfileV1 {
        let mut profile = base_profile();
        profile.segments.push(CreatureRigSegmentV1 {
            id: 9,
            name: "synthetic-rigid-segment".to_owned(),
            deformation: RigSegmentDeformationV1::Rigid,
            parent_node_id: 70,
            surface_positions: vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 1.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: Vec::new(),
            reference_weights: Vec::new(),
        });
        finish(profile)
    }

    pub fn skin_multi_profile() -> CreatureRigProfileV1 {
        let mut profile = base_profile();
        profile.profile_id = "wasm-controlled-skin-multi".to_owned();
        profile.nodes.push(CreatureRigNodeV1 {
            id: 1,
            name: "wasm-controlled-bone".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: identity(),
        });
        profile.segments.push(CreatureRigSegmentV1 {
            id: 5,
            name: "wasm-controlled-rigid-far".to_owned(),
            deformation: RigSegmentDeformationV1::Rigid,
            parent_node_id: 70,
            surface_positions: vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 0.0, 1.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: Vec::new(),
            reference_weights: Vec::new(),
        });
        profile.segments.push(CreatureRigSegmentV1 {
            id: 10,
            name: "wasm-controlled-skin-near".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 70,
            surface_positions: vec![
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [-1.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            surface_indices: vec![0, 1, 2, 1, 3, 2],
            allowed_bone_node_ids: vec![1],
            reference_weights: vec![
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 1.0,
                }];
                4
            ],
        });
        finish(profile)
    }

    pub fn options() -> ProfileAOptionsV1 {
        ProfileAOptionsV1::default()
    }

    pub fn profile_json(profile: &CreatureRigProfileV1) -> String {
        serde_json::to_string(profile).unwrap()
    }

    pub fn options_json(options: &ProfileAOptionsV1) -> String {
        serde_json::to_string(options).unwrap()
    }

    pub fn sha256(value: &str) -> String {
        let digest = Sha256::digest(value.as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn minimal_glb() -> Vec<u8> {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = [[0.0_f32, 0.0, 1.0]; 3];
        let uv0 = [[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let mut bin = Vec::new();
        let positions_offset = bin.len();
        append_rows(&mut bin, &positions);
        let normals_offset = bin.len();
        append_rows(&mut bin, &normals);
        let uv_offset = bin.len();
        append_rows(&mut bin, &uv0);
        let indices_offset = bin.len();
        for index in [0_u16, 1, 2] {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        align4(&mut bin, 0);
        let root = serde_json::json!({
            "asset": {"version": "2.0", "generator": "m2a-profile-a-byte-proof"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"name": "proof-root", "mesh": 0}],
            "meshes": [{"primitives": [{
                "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                "indices": 3,
                "mode": 4
            }]}],
            "buffers": [{"byteLength": bin.len()}],
            "bufferViews": [
                {"buffer": 0, "byteOffset": positions_offset, "byteLength": normals_offset - positions_offset},
                {"buffer": 0, "byteOffset": normals_offset, "byteLength": uv_offset - normals_offset},
                {"buffer": 0, "byteOffset": uv_offset, "byteLength": indices_offset - uv_offset},
                {"buffer": 0, "byteOffset": indices_offset, "byteLength": 6}
            ],
            "accessors": [
                {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0]},
                {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3"},
                {"bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2"},
                {"bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR"}
            ]
        });
        make_glb(root, bin)
    }

    fn append_rows<const N: usize>(output: &mut Vec<u8>, rows: &[[f32; N]]) {
        for row in rows {
            for value in row {
                output.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    fn align4(bytes: &mut Vec<u8>, fill: u8) {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(fill);
        }
    }

    fn make_glb(root: serde_json::Value, mut bin: Vec<u8>) -> Vec<u8> {
        let mut json = serde_json::to_vec(&root).unwrap();
        align4(&mut json, b' ');
        align4(&mut bin, 0);
        let total = 12 + 8 + json.len() + 8 + bin.len();
        let mut glb = Vec::with_capacity(total);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin);
        glb
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod profile_a_native_tests {
    use super::{
        convert_profile_a_glb_json, convert_profile_a_json, profile_a_test_support as support,
    };

    fn direct_core_json(
        bytes: &[u8],
        rig: &m2a_core::profile_a::CreatureRigProfileV1,
        options: &m2a_core::profile_a::ProfileAOptionsV1,
    ) -> String {
        let source = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
            .expect("controlled source ingest");
        match m2a_core::profile_a::convert_profile_a(&source, rig, options) {
            Ok(value) => serde_json::to_string(&value).unwrap(),
            Err(error) => serde_json::to_string(&error).unwrap(),
        }
    }

    #[test]
    fn native_profile_a_adapter_is_exact_core_json_and_records_byte_proof() {
        let glb = support::minimal_glb();
        for (rig, expected_sha, expected_length) in [
            (
                support::rigid_profile(),
                support::RIGID_JSON_SHA256,
                support::RIGID_JSON_LENGTH,
            ),
            (
                support::skin_multi_profile(),
                support::SKIN_JSON_SHA256,
                support::SKIN_JSON_LENGTH,
            ),
        ] {
            let options = support::options();
            let actual = convert_profile_a_json(
                &glb,
                &support::profile_json(&rig),
                &support::options_json(&options),
            );
            assert_eq!(
                actual,
                convert_profile_a_glb_json(
                    &glb,
                    &support::profile_json(&rig),
                    &support::options_json(&options),
                )
            );
            assert_eq!(actual, direct_core_json(&glb, &rig, &options));
            assert_eq!(
                actual.len(),
                expected_length,
                "sha={}",
                support::sha256(&actual)
            );
            assert_eq!(support::sha256(&actual), expected_sha);
        }
    }

    #[test]
    fn native_profile_a_json_errors_are_stable_and_strict() {
        let glb = support::minimal_glb();
        let options = support::options_json(&support::options());
        let malformed = convert_profile_a_glb_json(&glb, "{", &options);
        assert_eq!(malformed, convert_profile_a_json(&glb, "{", &options));
        assert_eq!(
            malformed,
            r#"{"schemaVersion":1,"code":"M3A-PROFILE-JSON-INVALID","severity":"FATAL","path":"rigJson","message":"rig profile JSON does not match the public schema"}"#
        );
        assert_eq!(
            malformed.len(),
            support::MALFORMED_JSON_LENGTH,
            "sha={}",
            support::sha256(&malformed)
        );
        assert_eq!(support::sha256(&malformed), support::MALFORMED_JSON_SHA256);
        let mut unknown = serde_json::to_value(support::rigid_profile()).unwrap();
        unknown["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_json(&glb, &unknown.to_string(), &options),
            convert_profile_a_json(&glb, "{", &options)
        );

        let rig = support::rigid_profile();
        let mut limited = support::options();
        limited.limits.max_distance_evaluations = 2;
        let fatal = convert_profile_a_json(
            &glb,
            &support::profile_json(&rig),
            &support::options_json(&limited),
        );
        assert_eq!(fatal, direct_core_json(&glb, &rig, &limited));
        assert_eq!(
            fatal.len(),
            support::LIMIT_FATAL_JSON_LENGTH,
            "sha={}",
            support::sha256(&fatal)
        );
        assert_eq!(support::sha256(&fatal), support::LIMIT_FATAL_JSON_SHA256);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::{
        convert_profile_a_glb_json, convert_profile_a_json, ingest_glb, ingest_glb_json,
        inspect_binary_mdl, inspect_glb, inspect_glb_json,
        profile_a_test_support as profile_support,
    };
    use wasm_bindgen_test::*;

    #[allow(dead_code)]
    mod fixtures {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../m2a-core/tests/fixtures/build_minimal_binary_mdl.rs"
        ));
    }

    #[wasm_bindgen_test]
    fn public_adapter_returns_stable_json_error_for_empty_input() {
        let first = inspect_binary_mdl(&[]);
        let second = inspect_binary_mdl(&[]);

        assert_eq!(first, second);
        let error: serde_json::Value =
            serde_json::from_str(&first).expect("adapter must return JSON");
        assert_eq!(error["schemaVersion"], 1);
        assert_eq!(error["severity"], "error");
        assert_eq!(error["code"], "M2A-MDL-HEADER-INVALID");
        assert!(error["offset"].is_number());
        assert!(error["context"].is_string());
    }

    #[wasm_bindgen_test]
    fn profile_a_rigid_and_skin_are_native_wasm_byte_identical() {
        let glb = profile_support::minimal_glb();
        for (rig, expected_sha, expected_length, expected_deformation) in [
            (
                profile_support::rigid_profile(),
                profile_support::RIGID_JSON_SHA256,
                profile_support::RIGID_JSON_LENGTH,
                "RIGID",
            ),
            (
                profile_support::skin_multi_profile(),
                profile_support::SKIN_JSON_SHA256,
                profile_support::SKIN_JSON_LENGTH,
                "SKIN",
            ),
        ] {
            let options = profile_support::options_json(&profile_support::options());
            let rig_json = profile_support::profile_json(&rig);
            let first = convert_profile_a_json(&glb, &rig_json, &options);
            let second = convert_profile_a_json(&glb, &rig_json, &options);
            assert_eq!(first, second);
            assert_eq!(first, convert_profile_a_glb_json(&glb, &rig_json, &options));
            assert_eq!(first.len(), expected_length);
            assert_eq!(profile_support::sha256(&first), expected_sha);
            let outcome: serde_json::Value = serde_json::from_str(&first).unwrap();
            assert_eq!(outcome["report"]["conversionEligible"], true);
            assert_eq!(
                outcome["creature"]["segments"][0]["deformation"],
                expected_deformation
            );
        }
    }

    #[wasm_bindgen_test]
    fn profile_a_json_boundary_and_fatal_paths_match_core_exactly() {
        let glb = profile_support::minimal_glb();
        let rig = profile_support::rigid_profile();
        let rig_json = profile_support::profile_json(&rig);
        let options = profile_support::options();
        let options_json = profile_support::options_json(&options);

        let malformed = convert_profile_a_json(&glb, "{", &options_json);
        assert_eq!(malformed, convert_profile_a_json(&glb, "[]", &options_json));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&malformed).unwrap()["code"],
            "M3A-PROFILE-JSON-INVALID"
        );
        assert_eq!(malformed.len(), profile_support::MALFORMED_JSON_LENGTH);
        assert_eq!(
            profile_support::sha256(&malformed),
            profile_support::MALFORMED_JSON_SHA256
        );

        let mut unknown_rig = serde_json::to_value(&rig).unwrap();
        unknown_rig["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_json(&glb, &unknown_rig.to_string(), &options_json),
            malformed
        );
        let mut unknown_options = serde_json::to_value(&options).unwrap();
        unknown_options["unknownField"] = serde_json::json!(true);
        let options_error = convert_profile_a_json(&glb, &rig_json, &unknown_options.to_string());
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&options_error).unwrap()["code"],
            "M3A-OPTIONS-INVALID"
        );

        let truncated = &glb[..glb.len() - 1];
        let expected_ingest =
            m2a_core::glb::ingest_glb(truncated, &m2a_core::glb::GlbLimits::default()).unwrap_err();
        assert_eq!(
            convert_profile_a_json(truncated, &rig_json, &options_json),
            serde_json::to_string(&expected_ingest).unwrap()
        );

        let mut limited = options.clone();
        limited.limits.max_distance_evaluations = 2;
        let limited_json =
            convert_profile_a_json(&glb, &rig_json, &profile_support::options_json(&limited));
        let source = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default()).unwrap();
        let expected_limit =
            m2a_core::profile_a::convert_profile_a(&source, &rig, &limited).unwrap_err();
        assert_eq!(
            limited_json,
            serde_json::to_string(&expected_limit).unwrap()
        );
        assert_eq!(expected_limit.code, "M3A-LIMIT-EXCEEDED");
        assert_eq!(limited_json.len(), profile_support::LIMIT_FATAL_JSON_LENGTH);
        assert_eq!(
            profile_support::sha256(&limited_json),
            profile_support::LIMIT_FATAL_JSON_SHA256
        );

        let mut provenance = rig.clone();
        provenance.provenance.kind = m2a_core::profile_a::RigProvenanceKindV1::ReferenceOnly;
        provenance.provenance.export_allowed = false;
        provenance.content_sha256.clear();
        provenance.content_sha256 =
            m2a_core::profile_a::canonical_profile_sha256(&provenance).unwrap();
        let provenance_json = convert_profile_a_json(
            &glb,
            &profile_support::profile_json(&provenance),
            &options_json,
        );
        let expected_provenance =
            m2a_core::profile_a::convert_profile_a(&source, &provenance, &options).unwrap();
        assert_eq!(
            provenance_json,
            serde_json::to_string(&expected_provenance).unwrap()
        );
        let blocked: serde_json::Value = serde_json::from_str(&provenance_json).unwrap();
        assert!(blocked["creature"].is_null());
        assert!(
            blocked["report"]["gates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|gate| gate["code"] == "M3A-PROFILE-PROVENANCE-FORBIDDEN")
        );
    }

    #[wasm_bindgen_test]
    fn public_adapter_returns_deterministic_json_for_synthetic_mdl() {
        let mdl = minimal_binary_mdl();
        let original = mdl.clone();
        let first = inspect_binary_mdl(&mdl);
        let second = inspect_binary_mdl(&mdl);

        assert_eq!(mdl, original, "adapter must not mutate selected-file bytes");
        assert_eq!(first, second);

        let report: serde_json::Value =
            serde_json::from_str(&first).expect("adapter must return JSON");
        assert_eq!(report["schemaVersion"], 1);
        assert_eq!(report["format"], "nwn1-binary-mdl");
        assert_eq!(report["byteLength"].as_u64(), Some(mdl.len() as u64));
        assert_eq!(report["nodeTree"]["nodeCount"], 1);
    }

    #[wasm_bindgen_test]
    fn public_adapter_exposes_deep_m1b_report_deterministically() {
        let mdl = fixtures::build_deep_binary_mdl();
        let first = inspect_binary_mdl(&mdl);
        let second = inspect_binary_mdl(&mdl);

        assert_eq!(first, second);

        let report: serde_json::Value =
            serde_json::from_str(&first).expect("adapter must return JSON");
        let root = &report["nodeTree"]["roots"][0];
        let controllers = root["controllers"]
            .as_array()
            .expect("deep fixture controllers");
        assert_eq!(controllers.len(), 5);
        assert_eq!(controllers[0]["controllerName"], "position");
        assert_eq!(controllers[0]["packedByte"], 3);
        assert_eq!(controllers[0]["interpolationFlags"], 0);
        assert_eq!(controllers[0]["decoded"], true);
        assert_eq!(controllers[1]["controllerName"], "orientation");
        assert_eq!(controllers[1]["packedByte"], 4);
        assert_eq!(controllers[1]["interpolationFlags"], 0);
        assert_eq!(controllers[1]["decoded"], true);
        assert_eq!(controllers[4]["controllerName"], "alpha");

        let mesh = root["mesh"].as_object().expect("deep fixture mesh");
        assert_eq!(mesh["vertexCount"], 3);
        assert_eq!(mesh["faces"].as_array().map(Vec::len), Some(1));
        assert_eq!(mesh["textures"][0], "m2a_diffuse");

        let animations = report["animations"]
            .as_array()
            .expect("deep fixture animations");
        assert_eq!(animations.len(), 2);
        assert_eq!(animations[0]["name"], "walk");
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["controllerName"],
            "position"
        );
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["packedByte"],
            3
        );
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["interpolationFlags"],
            0
        );
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["decoded"],
            true
        );
        assert_eq!(animations[1]["name"], "idle");
    }

    #[wasm_bindgen_test]
    fn public_adapter_exposes_both_m1b_skin_variants_deterministically() {
        for (extended64, expected_variant, expected_inline_count) in
            [(false, "legacy17", 17), (true, "extended64", 64)]
        {
            let mdl = fixtures::build_skin_binary_mdl(extended64);
            let first = inspect_binary_mdl(&mdl);
            let second = inspect_binary_mdl(&mdl);

            assert_eq!(first, second);

            let report: serde_json::Value =
                serde_json::from_str(&first).expect("adapter must return JSON");
            let root = &report["nodeTree"]["roots"][0];
            assert!(root["mesh"].is_object());
            assert_eq!(root["skin"]["variant"], expected_variant);
            assert_eq!(
                root["skin"]["inlineMapping"].as_array().map(Vec::len),
                Some(expected_inline_count)
            );
            assert_eq!(
                root["skin"]["nodeToBoneMap"].as_array().map(Vec::len),
                Some(3)
            );
            assert_eq!(
                root["skin"]["vertexWeights"].as_array().map(Vec::len),
                Some(3)
            );
            assert_eq!(
                root["skin"]["boneReferences"].as_array().map(Vec::len),
                Some(3)
            );
        }
    }

    #[wasm_bindgen_test]
    fn public_glb_adapters_match_core_json_and_are_deterministic() {
        let glb = minimal_synthetic_glb();
        let original = glb.clone();

        let inspect_first = inspect_glb(&glb);
        let inspect_second = inspect_glb(&glb);
        let expected_report =
            m2a_core::glb::inspect_glb(&glb, &m2a_core::glb::GlbLimits::default())
                .expect("synthetic GLB report");
        assert_eq!(inspect_first, inspect_second);
        assert_eq!(
            inspect_first,
            serde_json::to_string(&expected_report).unwrap()
        );

        let ingest_first = ingest_glb(&glb);
        let ingest_second = ingest_glb(&glb);
        let expected_ingest = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default())
            .expect("synthetic GLB ingest");
        assert_eq!(ingest_first, ingest_second);
        assert_eq!(
            ingest_first,
            serde_json::to_string(&expected_ingest).unwrap()
        );
        assert_eq!(glb, original, "WASM adapters must not mutate source bytes");

        let report: serde_json::Value = serde_json::from_str(&inspect_first).unwrap();
        assert_eq!(report["schemaVersion"], 1);
        assert_eq!(report["format"], "GLB_2_0");
        assert_eq!(report["inventory"]["sceneCount"], 1);
        assert_eq!(report["statistics"]["triangleCount"], 1);
        assert_eq!(report["coordinatePolicy"]["storedSpace"], "GLTF_SOURCE");

        let result: serde_json::Value = serde_json::from_str(&ingest_first).unwrap();
        assert_eq!(result["schemaVersion"], 1);
        assert_eq!(result["ir"]["schemaVersion"], 1);
        assert_eq!(
            result["ir"]["primitives"][0]["indices"],
            serde_json::json!([0, 1, 2])
        );
        assert_eq!(result["report"], report);
    }

    #[wasm_bindgen_test]
    fn public_glb_adapters_return_the_same_stable_empty_error_as_core() {
        let expected =
            m2a_core::glb::inspect_glb(&[], &m2a_core::glb::GlbLimits::default()).unwrap_err();
        let expected_json = serde_json::to_string(&expected).unwrap();

        assert_eq!(inspect_glb(&[]), expected_json);
        assert_eq!(ingest_glb(&[]), expected_json);
        assert_eq!(inspect_glb(&[]), inspect_glb(&[]));
        assert_eq!(ingest_glb(&[]), ingest_glb(&[]));

        let error: serde_json::Value = serde_json::from_str(&expected_json).unwrap();
        assert_eq!(error["schemaVersion"], 1);
        assert_eq!(error["code"], "M2A-GLB-INPUT-EMPTY");
        assert!(error["message"].is_string());
    }

    #[wasm_bindgen_test]
    fn public_contract_names_and_legacy_aliases_are_byte_identical() {
        let glb = minimal_synthetic_glb();

        assert_eq!(inspect_glb_json(&glb), inspect_glb(&glb));
        assert_eq!(ingest_glb_json(&glb), ingest_glb(&glb));
    }

    #[wasm_bindgen_test]
    fn public_fixture_d_preserves_material_image_identity_without_payload() {
        let glb = material_image_synthetic_glb();
        let (report, result) = assert_public_glb_parity(&glb);

        assert_eq!(report["inventory"]["primitiveCount"], 2);
        assert_eq!(report["inventory"]["materialCount"], 2);
        assert_eq!(report["inventory"]["textureCount"], 1);
        assert_eq!(report["inventory"]["samplerCount"], 1);
        assert_eq!(report["inventory"]["imageCount"], 1);

        let material = &result["ir"]["materials"][0];
        assert_eq!(
            material["baseColorFactor"],
            serde_json::json!([0.8, 0.7, 0.6, 0.5])
        );
        assert_eq!(material["baseColorTexture"]["textureId"], 0);
        assert_eq!(material["metallicFactor"], 0.35);
        assert_eq!(material["roughnessFactor"], 0.65);
        assert_eq!(material["alphaMode"], "MASK");

        let image = &result["ir"]["images"][0];
        assert_eq!(image["name"], "wasm-one-pixel");
        assert_eq!(image["mimeType"], "image/png");
        assert_eq!(image["byteLength"], MINIMAL_PNG.len());
        assert_eq!(image["payloadEmbeddedInJson"], false);
        assert_eq!(image["sha256"].as_str().map(str::len), Some(64));

        let json = ingest_glb_json(&glb);
        assert!(!json.contains("data:image"));
        assert!(!json.contains("iVBORw0KGgo"));
        assert!(!json.contains("imageBytes"));
        assert!(!json.contains("\"payload\":"));
        assert!(!json.contains("C:\\\\"));
    }

    #[wasm_bindgen_test]
    fn public_fixture_e_preserves_skin_and_animation_schema_in_source_basis() {
        let glb = skin_animation_synthetic_glb(false);
        let (report, result) = assert_public_glb_parity(&glb);

        assert_eq!(report["inventory"]["skinCount"], 1);
        assert_eq!(report["inventory"]["jointReferenceCount"], 2);
        assert_eq!(report["inventory"]["animationCount"], 1);
        assert_eq!(report["inventory"]["keyframeCount"], 9);

        let skin = &result["ir"]["skins"][0];
        assert_eq!(skin["skeletonRootNodeId"], 0);
        assert_eq!(skin["jointNodeIds"], serde_json::json!([0, 1]));
        assert_eq!(
            skin["inverseBindMatrices"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(skin["inverseBindMatrices"][1][12], 2.0);
        assert_eq!(skin["inverseBindMatrices"][1][13], 3.0);
        assert_eq!(skin["inverseBindMatrices"][1][14], 4.0);

        let primitive = &result["ir"]["primitives"][0];
        assert_eq!(primitive["joints0"][0], serde_json::json!([0, 1, 0, 0]));
        assert_eq!(
            primitive["weights0"][0],
            serde_json::json!([0.75, 0.25, 0.0, 0.0])
        );

        let animation = &result["ir"]["animations"][0];
        assert_eq!(animation["durationSeconds"], 1.25);
        assert_eq!(animation["samplers"][0]["interpolation"], "LINEAR");
        assert_eq!(animation["samplers"][1]["interpolation"], "STEP");
        assert_eq!(animation["samplers"][2]["interpolation"], "CUBICSPLINE");
        assert_eq!(
            animation["samplers"][2]["outputValues"]
                .as_array()
                .map(Vec::len),
            Some(27)
        );
        assert_eq!(animation["channels"][0]["targetPath"], "TRANSLATION");
        assert_eq!(animation["channels"][1]["targetPath"], "ROTATION");
        assert_eq!(animation["channels"][2]["targetPath"], "SCALE");

        let without_inverse_bind = mutate_synthetic_glb(glb, |root| {
            root["skins"][0]
                .as_object_mut()
                .unwrap()
                .remove("inverseBindMatrices");
        });
        let (_, without_inverse_bind) = assert_public_glb_parity(&without_inverse_bind);
        assert_eq!(
            without_inverse_bind["ir"]["skins"][0]["inverseBindMatrices"],
            serde_json::json!([])
        );
    }

    #[wasm_bindgen_test]
    fn public_fixture_e_weights_channel_is_explicitly_blocking() {
        let glb = skin_animation_synthetic_glb(true);
        let (report, result) = assert_public_glb_parity(&glb);

        assert_eq!(
            result["ir"]["animations"][0]["channels"][3]["targetPath"],
            "WEIGHTS"
        );
        assert_eq!(report["conversionEligible"], false);
        assert_gate(&report, "M2A-GLB-ANIMATION-WEIGHTS-DEFERRED", "BLOCKING");
    }

    #[wasm_bindgen_test]
    fn public_fixture_f_exposes_stable_blocking_gates() {
        for (glb, code) in [
            (missing_uv_synthetic_glb(), "M2A-GLB-UV0-MISSING"),
            (missing_position_synthetic_glb(), "M2A-GLB-POSITION-MISSING"),
            (
                morph_target_synthetic_glb(),
                "M2A-GLB-MORPH-TARGETS-DEFERRED",
            ),
            (
                mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
                    root["meshes"][0]["primitives"][0]["mode"] = serde_json::json!(1);
                }),
                "M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED",
            ),
            (
                mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
                    root["accessors"][2]["count"] = serde_json::json!(2);
                }),
                "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
            ),
        ] {
            let (report, result) = assert_public_glb_parity(&glb);
            assert_eq!(report["conversionEligible"], false);
            assert_gate(&report, code, "BLOCKING");
            assert_eq!(result["report"], report);
        }
    }

    #[wasm_bindgen_test]
    fn public_glb_contract_returns_stable_fatal_errors_and_never_accepts_truncation() {
        let complete = material_image_synthetic_glb();
        let mut bad_magic = complete.clone();
        bad_magic[0] = b'X';
        let mut bad_version = complete.clone();
        bad_version[4..8].copy_from_slice(&1_u32.to_le_bytes());
        let mut bad_length = complete.clone();
        bad_length[8..12].copy_from_slice(&0_u32.to_le_bytes());

        for (bytes, code) in [
            (Vec::new(), "M2A-GLB-INPUT-EMPTY"),
            (bad_magic, "M2A-GLB-HEADER-INVALID"),
            (bad_version, "M2A-GLB-VERSION-UNSUPPORTED"),
            (bad_length, "M2A-GLB-LENGTH-MISMATCH"),
            (oversized_json_chunk_glb(), "M2A-GLB-LIMIT-EXCEEDED"),
        ] {
            assert_stable_fatal_parity(&bytes, code);
        }

        let fixture = minimal_synthetic_glb();
        for length in 0..fixture.len() {
            let prefix = &fixture[..length];
            let inspect = inspect_glb_json(prefix);
            let ingest = ingest_glb_json(prefix);
            assert_eq!(inspect, inspect_glb_json(prefix));
            assert_eq!(ingest, ingest_glb_json(prefix));
            let inspect_error: serde_json::Value = serde_json::from_str(&inspect).unwrap();
            let ingest_error: serde_json::Value = serde_json::from_str(&ingest).unwrap();
            assert_eq!(inspect_error["schemaVersion"], 1);
            assert_eq!(ingest_error["schemaVersion"], 1);
            assert!(
                inspect_error["code"]
                    .as_str()
                    .is_some_and(|code| code.starts_with("M2A-GLB-"))
            );
            assert!(
                ingest_error["code"]
                    .as_str()
                    .is_some_and(|code| code.starts_with("M2A-GLB-"))
            );
        }
    }

    fn assert_public_glb_parity(glb: &[u8]) -> (serde_json::Value, serde_json::Value) {
        let original = glb.to_vec();
        let inspect_first = inspect_glb_json(glb);
        let inspect_second = inspect_glb_json(glb);
        let ingest_first = ingest_glb_json(glb);
        let ingest_second = ingest_glb_json(glb);

        assert_eq!(
            glb,
            original.as_slice(),
            "public adapters must not mutate input"
        );
        assert_eq!(inspect_first, inspect_second);
        assert_eq!(ingest_first, ingest_second);
        assert_eq!(inspect_first, inspect_glb(glb));
        assert_eq!(ingest_first, ingest_glb(glb));

        let core_report =
            m2a_core::glb::inspect_glb(glb, &m2a_core::glb::GlbLimits::default()).unwrap();
        let core_result =
            m2a_core::glb::ingest_glb(glb, &m2a_core::glb::GlbLimits::default()).unwrap();
        assert_eq!(inspect_first, serde_json::to_string(&core_report).unwrap());
        assert_eq!(ingest_first, serde_json::to_string(&core_result).unwrap());

        (
            serde_json::from_str(&inspect_first).unwrap(),
            serde_json::from_str(&ingest_first).unwrap(),
        )
    }

    fn assert_stable_fatal_parity(bytes: &[u8], expected_code: &str) {
        let expected =
            m2a_core::glb::inspect_glb(bytes, &m2a_core::glb::GlbLimits::default()).unwrap_err();
        let expected_json = serde_json::to_string(&expected).unwrap();
        assert_eq!(expected.code, expected_code);
        assert_eq!(inspect_glb_json(bytes), expected_json);
        assert_eq!(ingest_glb_json(bytes), expected_json);
        assert_eq!(inspect_glb(bytes), expected_json);
        assert_eq!(ingest_glb(bytes), expected_json);
    }

    fn assert_gate(report: &serde_json::Value, code: &str, severity: &str) {
        assert!(
            report["gates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|gate| { gate["code"] == code && gate["severity"] == severity })
        );
    }

    fn minimal_binary_mdl() -> Vec<u8> {
        const FILE_HEADER_SIZE: usize = 0x0c;
        const MODEL_HEADER_SIZE: usize = 0xe8;
        const NODE_HEADER_SIZE: usize = 0x70;
        const ROOT_NODE_OFFSET: u32 = MODEL_HEADER_SIZE as u32;
        const MODEL_DATA_SIZE: usize = MODEL_HEADER_SIZE + NODE_HEADER_SIZE;

        let mut bytes = vec![0_u8; FILE_HEADER_SIZE + MODEL_DATA_SIZE];

        write_u32(&mut bytes, 0x00, 0);
        write_u32(&mut bytes, 0x04, MODEL_DATA_SIZE as u32);
        write_u32(&mut bytes, 0x08, 0);

        let model = FILE_HEADER_SIZE;
        write_c_string(&mut bytes, model + 0x08, 64, "m2a_minimal");
        write_u32(&mut bytes, model + 0x48, ROOT_NODE_OFFSET);
        write_u32(&mut bytes, model + 0x4c, 1);

        let root = FILE_HEADER_SIZE + ROOT_NODE_OFFSET as usize;
        write_u32(&mut bytes, root + 0x1c, 0);
        write_c_string(&mut bytes, root + 0x20, 32, "root");
        write_u32(&mut bytes, root + 0x6c, 0x001);

        bytes
    }

    fn minimal_synthetic_glb() -> Vec<u8> {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = [[0.0_f32, 0.0, 1.0]; 3];
        let uv0 = [[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = [0_u16, 1, 2];
        let mut bin = Vec::new();

        let positions_offset = bin.len();
        append_f32_rows(&mut bin, &positions);
        let positions_length = bin.len() - positions_offset;
        let normals_offset = bin.len();
        append_f32_rows(&mut bin, &normals);
        let normals_length = bin.len() - normals_offset;
        let uv_offset = bin.len();
        append_f32_rows(&mut bin, &uv0);
        let uv_length = bin.len() - uv_offset;
        let indices_offset = bin.len();
        for index in indices {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        let indices_length = bin.len() - indices_offset;
        align4(&mut bin, 0);

        let root = serde_json::json!({
            "asset": {"version": "2.0", "generator": "m2a-wasm-synthetic"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"name": "root", "mesh": 0}],
            "meshes": [{"primitives": [{
                "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                "indices": 3,
                "mode": 4
            }]}],
            "buffers": [{"byteLength": bin.len()}],
            "bufferViews": [
                {"buffer": 0, "byteOffset": positions_offset, "byteLength": positions_length},
                {"buffer": 0, "byteOffset": normals_offset, "byteLength": normals_length},
                {"buffer": 0, "byteOffset": uv_offset, "byteLength": uv_length},
                {"buffer": 0, "byteOffset": indices_offset, "byteLength": indices_length}
            ],
            "accessors": [
                {
                    "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
                    "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]
                },
                {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3"},
                {"bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2"},
                {"bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR"}
            ]
        });
        let mut json = serde_json::to_vec(&root).unwrap();
        align4(&mut json, b' ');

        let total_length = 12 + 8 + json.len() + 8 + bin.len();
        let mut glb = Vec::with_capacity(total_length);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin);
        glb
    }

    const MINIMAL_PNG: [u8; 68] = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xb5,
        0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xfc,
        0xff, 0x1f, 0x00, 0x03, 0x03, 0x02, 0x00, 0xef, 0xa3, 0xe1, 0x1d, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    fn material_image_synthetic_glb() -> Vec<u8> {
        let (mut root, mut bin) = split_synthetic_glb(&minimal_synthetic_glb());
        align4(&mut bin, 0);
        let image_offset = bin.len();
        bin.extend_from_slice(&MINIMAL_PNG);
        let image_view = push_view(&mut root, image_offset, MINIMAL_PNG.len());
        root["buffers"][0]["byteLength"] = serde_json::json!(bin.len());

        let first = root["meshes"][0]["primitives"][0].clone();
        root["meshes"][0]["primitives"] = serde_json::json!([first.clone(), first]);
        root["meshes"][0]["primitives"][0]["material"] = serde_json::json!(0);
        root["meshes"][0]["primitives"][1]["material"] = serde_json::json!(1);
        root["samplers"] = serde_json::json!([{
            "name": "wasm-sampler",
            "magFilter": 9728,
            "minFilter": 9987,
            "wrapS": 33071,
            "wrapT": 33648
        }]);
        root["images"] = serde_json::json!([{
            "name": "wasm-one-pixel",
            "bufferView": image_view,
            "mimeType": "image/png"
        }]);
        root["textures"] = serde_json::json!([{"sampler": 0, "source": 0}]);
        root["materials"] = serde_json::json!([
            {
                "name": "wasm-painted-mask",
                "pbrMetallicRoughness": {
                    "baseColorFactor": [0.8, 0.7, 0.6, 0.5],
                    "baseColorTexture": {"index": 0, "texCoord": 0},
                    "metallicFactor": 0.35,
                    "roughnessFactor": 0.65
                },
                "alphaMode": "MASK",
                "alphaCutoff": 0.33,
                "doubleSided": true
            },
            {
                "name": "wasm-detail",
                "pbrMetallicRoughness": {
                    "baseColorTexture": {"index": 0},
                    "metallicFactor": 0.05,
                    "roughnessFactor": 0.15
                }
            }
        ]);
        make_synthetic_glb(root, bin)
    }

    fn skin_animation_synthetic_glb(include_weights_channel: bool) -> Vec<u8> {
        let (mut root, mut bin) = split_synthetic_glb(&minimal_synthetic_glb());

        align4(&mut bin, 0);
        let joints_offset = bin.len();
        for joints in [[0_u8, 1, 0, 0], [1, 0, 0, 0], [0, 1, 0, 0]] {
            bin.extend_from_slice(&joints);
        }
        let joints_view = push_view(&mut root, joints_offset, 12);
        let joints_accessor = push_accessor(
            &mut root,
            serde_json::json!({
                "bufferView": joints_view,
                "componentType": 5121,
                "count": 3,
                "type": "VEC4"
            }),
        );

        let weights_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[0.75, 0.25, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.4, 0.6, 0.0, 0.0],
            3,
            "VEC4",
            None,
            None,
        );
        root["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"] =
            serde_json::json!(joints_accessor);
        root["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"] =
            serde_json::json!(weights_accessor);

        root["nodes"] = serde_json::json!([
            {"name": "rig-root", "children": [1, 2]},
            {"name": "joint-one", "translation": [0.0, 1.0, 0.0]},
            {"name": "skinned-mesh", "mesh": 0, "skin": 0}
        ]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);

        let inverse_bind_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
            ],
            2,
            "MAT4",
            None,
            None,
        );
        root["skins"] = serde_json::json!([{
            "name": "wasm-two-joint-skin",
            "joints": [0, 1],
            "skeleton": 0,
            "inverseBindMatrices": inverse_bind_accessor
        }]);

        let times_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[0.0, 0.5, 1.25],
            3,
            "SCALAR",
            Some(serde_json::json!([0.0])),
            Some(serde_json::json!([1.25])),
        );
        let translation_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            3,
            "VEC3",
            None,
            None,
        );
        let rotation_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[
                0.0, 0.0, 0.0, 1.0, 0.0, 0.70710677, 0.0, 0.70710677, 0.0, 1.0, 0.0, 0.0,
            ],
            3,
            "VEC4",
            None,
            None,
        );
        let scale_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[
                -0.1, 0.0, 0.0, 1.0, 1.0, 1.0, 0.1, 0.0, 0.0, -0.2, 0.0, 0.0, 1.5, 2.0, 2.5, 0.2,
                0.0, 0.0, -0.3, 0.0, 0.0, 2.0, 3.0, 4.0, 0.3, 0.0, 0.0,
            ],
            9,
            "VEC3",
            None,
            None,
        );
        let mut channels = vec![
            serde_json::json!({"sampler": 0, "target": {"node": 1, "path": "translation"}}),
            serde_json::json!({"sampler": 1, "target": {"node": 1, "path": "rotation"}}),
            serde_json::json!({"sampler": 2, "target": {"node": 1, "path": "scale"}}),
        ];
        if include_weights_channel {
            channels
                .push(serde_json::json!({"sampler": 0, "target": {"node": 2, "path": "weights"}}));
        }
        root["animations"] = serde_json::json!([{
            "name": "wasm-source-trs",
            "samplers": [
                {"input": times_accessor, "output": translation_accessor, "interpolation": "LINEAR"},
                {"input": times_accessor, "output": rotation_accessor, "interpolation": "STEP"},
                {"input": times_accessor, "output": scale_accessor, "interpolation": "CUBICSPLINE"}
            ],
            "channels": channels
        }]);
        root["buffers"][0]["byteLength"] = serde_json::json!(bin.len());
        make_synthetic_glb(root, bin)
    }

    fn missing_uv_synthetic_glb() -> Vec<u8> {
        mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("TEXCOORD_0");
        })
    }

    fn missing_position_synthetic_glb() -> Vec<u8> {
        mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("POSITION");
        })
    }

    fn morph_target_synthetic_glb() -> Vec<u8> {
        mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
            root["meshes"][0]["primitives"][0]["targets"] = serde_json::json!([{"POSITION": 0}]);
        })
    }

    fn oversized_json_chunk_glb() -> Vec<u8> {
        const OVERSIZED_JSON_LENGTH: usize = 16 * 1024 * 1024 + 4;
        let total_length = 12 + 8 + OVERSIZED_JSON_LENGTH;
        let mut glb = Vec::with_capacity(total_length);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());
        glb.extend_from_slice(&(OVERSIZED_JSON_LENGTH as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.resize(total_length, b' ');
        glb
    }

    fn push_f32_accessor(
        root: &mut serde_json::Value,
        bin: &mut Vec<u8>,
        values: &[f32],
        count: usize,
        element_type: &str,
        min: Option<serde_json::Value>,
        max: Option<serde_json::Value>,
    ) -> usize {
        align4(bin, 0);
        let offset = bin.len();
        for value in values {
            bin.extend_from_slice(&value.to_le_bytes());
        }
        let view = push_view(root, offset, bin.len() - offset);
        let mut accessor = serde_json::json!({
            "bufferView": view,
            "componentType": 5126,
            "count": count,
            "type": element_type
        });
        if let Some(min) = min {
            accessor["min"] = min;
        }
        if let Some(max) = max {
            accessor["max"] = max;
        }
        push_accessor(root, accessor)
    }

    fn push_view(root: &mut serde_json::Value, offset: usize, length: usize) -> usize {
        let views = root["bufferViews"].as_array_mut().unwrap();
        let index = views.len();
        views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": length
        }));
        index
    }

    fn push_accessor(root: &mut serde_json::Value, accessor: serde_json::Value) -> usize {
        let accessors = root["accessors"].as_array_mut().unwrap();
        let index = accessors.len();
        accessors.push(accessor);
        index
    }

    fn mutate_synthetic_glb(
        glb: Vec<u8>,
        mutation: impl FnOnce(&mut serde_json::Value),
    ) -> Vec<u8> {
        let (mut root, bin) = split_synthetic_glb(&glb);
        mutation(&mut root);
        make_synthetic_glb(root, bin)
    }

    fn split_synthetic_glb(glb: &[u8]) -> (serde_json::Value, Vec<u8>) {
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_length;
        let root = serde_json::from_slice(&glb[20..json_end]).unwrap();
        let bin_start = json_end + 8;
        (root, glb[bin_start..].to_vec())
    }

    fn make_synthetic_glb(root: serde_json::Value, mut bin: Vec<u8>) -> Vec<u8> {
        let mut json = serde_json::to_vec(&root).unwrap();
        align4(&mut json, b' ');
        align4(&mut bin, 0);
        let total_length = 12 + 8 + json.len() + 8 + bin.len();
        let mut glb = Vec::with_capacity(total_length);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin);
        glb
    }

    fn append_f32_rows<const N: usize>(bytes: &mut Vec<u8>, rows: &[[f32; N]]) {
        for row in rows {
            for value in row {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    fn align4(bytes: &mut Vec<u8>, padding: u8) {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(padding);
        }
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_c_string(bytes: &mut [u8], offset: usize, capacity: usize, value: &str) {
        assert!(value.len() < capacity);
        bytes[offset..offset + value.len()].copy_from_slice(value.as_bytes());
    }
}
