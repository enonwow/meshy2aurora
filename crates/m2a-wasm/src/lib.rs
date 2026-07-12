use wasm_bindgen::prelude::*;

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
#[wasm_bindgen(js_name = inspectGlb)]
pub fn inspect_glb(bytes: &[u8]) -> String {
    match m2a_core::glb::inspect_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Ingests a GLB selected by JavaScript into source-preserving AuroraAssetIR.
///
/// The adapter deliberately delegates the complete operation to `m2a-core`.
#[wasm_bindgen(js_name = ingestGlb)]
pub fn ingest_glb(bytes: &[u8]) -> String {
    match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

fn serialize_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| SERIALIZATION_ERROR_JSON.to_owned())
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::{ingest_glb, inspect_binary_mdl, inspect_glb};
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
