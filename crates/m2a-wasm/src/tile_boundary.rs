//! Tile-specific public WASM boundary.
//!
//! This module owns the strict tile options schema, transferable buffers and
//! public ABI wrapper without changing the generated JavaScript names.

use wasm_bindgen::prelude::*;

use super::serialize_json;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StaticTileBoundaryOptionsV1 {
    schema_version: u32,
    identity: m2a_core::tile::StaticTileIdentityV1,
    interior: bool,
    terrain_name: String,
    surface: m2a_core::tile::TileSurfaceV1,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TileBoundaryErrorV1<'a> {
    schema_version: u32,
    code: &'a str,
    path: &'a str,
    message: &'a str,
}

/// Browser-transferable result for the complete static tile lane. Every
/// binary payload remains an owned byte buffer; JSON getters contain only
/// reports/readbacks and never filesystem paths.
#[wasm_bindgen]
pub struct StudioTilePackageArtifactV1 {
    hak_bytes: Vec<u8>,
    module_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    wok_bytes: Vec<u8>,
    set_bytes: Vec<u8>,
    texture_bytes: Vec<u8>,
    image_map_bytes: Vec<u8>,
    report_json: String,
    model_readback_json: String,
    wok_readback_json: String,
    set_readback_json: String,
}

#[wasm_bindgen]
impl StudioTilePackageArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModuleBytes)]
    pub fn take_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.module_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(js_name = takeWokBytes)]
    pub fn take_wok_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.wok_bytes)
    }

    #[wasm_bindgen(js_name = takeSetBytes)]
    pub fn take_set_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.set_bytes)
    }

    #[wasm_bindgen(js_name = takeTextureBytes)]
    pub fn take_texture_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_bytes)
    }

    #[wasm_bindgen(js_name = takeImageMapBytes)]
    pub fn take_image_map_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.image_map_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = modelReadbackJson)]
    pub fn model_readback_json(&self) -> String {
        self.model_readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = wokReadbackJson)]
    pub fn wok_readback_json(&self) -> String {
        self.wok_readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = setReadbackJson)]
    pub fn set_readback_json(&self) -> String {
        self.set_readback_json.clone()
    }
}

pub(crate) fn build_meshy_static_tile_package_v1_inner(
    source_glb: &[u8],
    options_json: &str,
) -> Result<StudioTilePackageArtifactV1, String> {
    let options = serde_json::from_str::<StaticTileBoundaryOptionsV1>(options_json).map_err(|_| {
        serialize_json(&TileBoundaryErrorV1 {
            schema_version: 1,
            code: "TILE-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict StaticTileBoundaryOptionsV1 schema",
        })
    })?;
    if options.schema_version != 1 {
        return Err(serialize_json(&TileBoundaryErrorV1 {
            schema_version: 1,
            code: "TILE-OPTIONS-SCHEMA-INVALID",
            path: "optionsJson.schemaVersion",
            message: "tile boundary options must use schema version 1",
        }));
    }
    let artifact = m2a_core::tile::build_meshy_static_tile_package_v1(
        source_glb,
        &options.identity,
        options.interior,
        &options.terrain_name,
        options.surface,
    )
    .map_err(|error| serialize_json(&error))?;
    let model_readback = m2a_core::inspect_binary_mdl(&artifact.mdl_payload)
        .map_err(|error| serialize_json(&error))?;
    let wok_readback = m2a_core::walkmesh::inspect_ascii_tile_wok_v1(&artifact.wok_payload)
        .map_err(|error| serialize_json(&error))?;
    let set_readback = m2a_core::tile::parse_tileset_v1(&artifact.set_payload)
        .map_err(|error| serialize_json(&error))?;
    Ok(StudioTilePackageArtifactV1 {
        hak_bytes: artifact.hak_payload,
        module_bytes: artifact.module_payload,
        model_bytes: artifact.mdl_payload,
        wok_bytes: artifact.wok_payload,
        set_bytes: artifact.set_payload,
        texture_bytes: artifact.texture_payload,
        image_map_bytes: artifact.image_map_payload,
        report_json: serialize_json(&artifact.report),
        model_readback_json: serialize_json(&model_readback),
        wok_readback_json: serialize_json(&wok_readback),
        set_readback_json: serialize_json(&set_readback),
    })
}

/// Executes `GLB bytes + strict JSON options -> HAK/MOD/report/readbacks`
/// entirely inside Rust/WASM. No DOM, backend or filesystem is consulted.
#[wasm_bindgen(js_name = buildMeshyStaticTilePackageV1)]
pub fn build_meshy_static_tile_package_v1(
    source_glb: &[u8],
    options_json: &str,
) -> Result<StudioTilePackageArtifactV1, JsValue> {
    build_meshy_static_tile_package_v1_inner(source_glb, options_json)
        .map_err(|error| JsValue::from_str(&error))
}
