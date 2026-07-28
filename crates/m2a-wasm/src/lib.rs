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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MdlWriterJsonInputError<'a> {
    schema_version: u32,
    code: &'a str,
    severity: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct M5BoundaryJsonError<'a> {
    schema_version: u32,
    code: &'a str,
    severity: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HakResourceDescriptorV1 {
    resref: String,
    resource_type: u16,
    payload_offset: u32,
    payload_size: u32,
}

impl m2a_core::hak::HakResourceMetadataV1 for HakResourceDescriptorV1 {
    fn hak_resref(&self) -> &str {
        &self.resref
    }

    fn hak_resource_type(&self) -> u16 {
        self.resource_type
    }

    fn hak_payload_size(&self) -> Option<u64> {
        Some(u64::from(self.payload_size))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HakResourceDescriptorsV1 {
    schema_version: u32,
    resources: Vec<HakResourceDescriptorV1>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(
    tag = "role",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum M7PayloadDescriptorV1 {
    Source {
        relative_path: String,
        payload_offset: u32,
        payload_size: u32,
    },
    #[serde(rename = "RIGGED_HUMANOID_APPEARANCE_2DA")]
    RiggedHumanoidAppearance2da {
        sample_id: String,
        payload_offset: u32,
        payload_size: u32,
    },
}

impl M7PayloadDescriptorV1 {
    fn offset(&self) -> u32 {
        match self {
            Self::Source { payload_offset, .. }
            | Self::RiggedHumanoidAppearance2da { payload_offset, .. } => *payload_offset,
        }
    }

    fn size(&self) -> u32 {
        match self {
            Self::Source { payload_size, .. }
            | Self::RiggedHumanoidAppearance2da { payload_size, .. } => *payload_size,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct M7PayloadDescriptorsV1 {
    schema_version: u32,
    payloads: Vec<M7PayloadDescriptorV1>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct M7BoundaryErrorV1<'a> {
    schema_version: u32,
    code: &'a str,
    path: String,
    message: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaceableBoundaryErrorV1<'a> {
    schema_version: u32,
    code: &'a str,
    path: &'a str,
    message: &'a str,
}

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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct M7BatchBoundaryOutputV1 {
    schema_version: u32,
    report: m2a_core::m7_corpus::M7CorpusBatchReportV1,
    packets: Vec<m2a_core::m7_corpus::M7PerProfileProofPacketV1>,
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

/// Ingests a static placeable or tile GLB with the shared native single-mesh
/// triangle envelope. The parser and AuroraAssetIR stay identical to the
/// creature route; only target-specific admission diagnostics differ.
#[wasm_bindgen(js_name = ingestStaticRigidGlbJson)]
pub fn ingest_static_rigid_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::ingest_glb(
        bytes,
        &m2a_core::placeable::static_placeable_glb_limits_v1(),
    ) {
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
    let (rig, options) = match parse_profile_a_json(rig_json, options_json) {
        Ok(values) => values,
        Err(error) => return error,
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

/// Converts a Meshy-style animated GLB with explicit clean-room rig and
/// source-to-output animation mappings.
///
/// This boundary only performs strict JSON decoding and GLB ingestion. Mapping
/// validation, retargeting and all deterministic output generation are owned by
/// the public `m2a-core` M4A2 route.
#[wasm_bindgen(js_name = convertProfileAWithAnimationsGlbJson)]
pub fn convert_profile_a_with_animations_glb_json(
    bytes: &[u8],
    rig_json: &str,
    options_json: &str,
    mapping_json: &str,
) -> String {
    let (rig, options) = match parse_profile_a_json(rig_json, options_json) {
        Ok(values) => values,
        Err(error) => return error,
    };
    let mapping =
        match serde_json::from_str::<m2a_core::profile_a::ProfileAAnimationMappingV1>(mapping_json)
        {
            Ok(value) => value,
            Err(_) => {
                return serialize_json(&ProfileAJsonInputError {
                    schema_version: 1,
                    code: "M4A-MAPPING-JSON-INVALID",
                    severity: "FATAL",
                    path: "mappingJson",
                    message: "animation mapping JSON does not match the public schema",
                });
            }
        };
    let source = match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(value) => value,
        Err(error) => return serialize_json(&error),
    };
    match m2a_core::profile_a::convert_profile_a_with_animations_v1(
        &source, &rig, &options, &mapping,
    ) {
        Ok(outcome) => serialize_json(&outcome),
        Err(error) => serialize_json(&error),
    }
}

/// Concise alias retained for callers that adopted the initial M3 adapter name.
#[wasm_bindgen(js_name = convertProfileAJson)]
pub fn convert_profile_a_json(bytes: &[u8], rig_json: &str, options_json: &str) -> String {
    convert_profile_a_glb_json(bytes, rig_json, options_json)
}

fn parse_profile_a_json(
    rig_json: &str,
    options_json: &str,
) -> Result<
    (
        m2a_core::profile_a::CreatureRigProfileV1,
        m2a_core::profile_a::ProfileAOptionsV1,
    ),
    String,
> {
    let rig = serde_json::from_str(rig_json).map_err(|_| {
        serialize_json(&ProfileAJsonInputError {
            schema_version: 1,
            code: "M3A-PROFILE-JSON-INVALID",
            severity: "FATAL",
            path: "rigJson",
            message: "rig profile JSON does not match the public schema",
        })
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        serialize_json(&ProfileAJsonInputError {
            schema_version: 1,
            code: "M3A-OPTIONS-INVALID",
            severity: "FATAL",
            path: "optionsJson",
            message: "options JSON does not match the public schema",
        })
    })?;
    Ok((rig, options))
}

/// Writes an Aurora binary MDL from strict public JSON contracts.
///
/// The returned vector is mapped to a JavaScript `Uint8Array`. Failures are a
/// stable serialized JSON object carried by `JsValue`; no binary encoding or
/// validation is duplicated at the WASM boundary.
#[wasm_bindgen(js_name = writeBinaryMdlWithAnimations)]
pub fn write_binary_mdl_with_animations(
    creature_json: &str,
    animations_json: &str,
    options_json: &str,
) -> Result<Vec<u8>, JsValue> {
    let (creature, animations, options) =
        parse_mdl_writer_json(creature_json, animations_json, options_json)
            .map_err(|error| JsValue::from_str(&error))?;
    m2a_core::mdl::write_binary_mdl_with_animations(&creature, &animations, &options)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

/// Returns the deterministic core writer report or the same stable JSON error
/// used by `writeBinaryMdlWithAnimations`.
#[wasm_bindgen(js_name = writeBinaryMdlWithAnimationsReportJson)]
pub fn write_binary_mdl_with_animations_report_json(
    creature_json: &str,
    animations_json: &str,
    options_json: &str,
) -> String {
    let (creature, animations, options) =
        match parse_mdl_writer_json(creature_json, animations_json, options_json) {
            Ok(values) => values,
            Err(error) => return error,
        };
    match m2a_core::mdl::write_binary_mdl_with_animations(&creature, &animations, &options) {
        Ok(artifact) => serialize_json(&artifact.report),
        Err(error) => serialize_json(&error),
    }
}

fn parse_mdl_writer_json(
    creature_json: &str,
    animations_json: &str,
    options_json: &str,
) -> Result<
    (
        m2a_core::profile_a::AuroraCreatureIrV1,
        m2a_core::mdl::MdlAnimationSetV1,
        m2a_core::mdl::MdlWriterOptionsV1,
    ),
    String,
> {
    let creature = serde_json::from_str(creature_json).map_err(|_| {
        mdl_writer_json_error(
            "M4A-CREATURE-JSON-INVALID",
            "creatureJson",
            "creature JSON does not match the strict public schema",
        )
    })?;
    let animations = serde_json::from_str(animations_json).map_err(|_| {
        mdl_writer_json_error(
            "M4A-ANIMATION-JSON-INVALID",
            "animationsJson",
            "animation set JSON does not match the strict public schema",
        )
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        mdl_writer_json_error(
            "M4A-OPTIONS-JSON-INVALID",
            "optionsJson",
            "writer options JSON does not match the strict public schema",
        )
    })?;
    Ok((creature, animations, options))
}

fn mdl_writer_json_error(code: &str, path: &str, message: &str) -> String {
    serialize_json(&MdlWriterJsonInputError {
        schema_version: 1,
        code,
        severity: "FATAL",
        path,
        message,
    })
}

fn serialize_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| SERIALIZATION_ERROR_JSON.to_owned())
}

fn m5_boundary_error(code: &str, path: &str, message: &str) -> String {
    m5_error_json(code, "FATAL", path, message)
}

fn m5_error_json(code: &str, severity: &str, path: &str, message: &str) -> String {
    serialize_json(&M5BoundaryJsonError {
        schema_version: 1,
        code,
        severity,
        path,
        message,
    })
}

fn two_da_core_error_json(error: &m2a_core::two_da::TwoDaError) -> String {
    m5_error_json(&error.code, &error.severity, &error.path, &error.message)
}

fn parse_tga_json(
    image_json: &str,
    options_json: &str,
) -> Result<(m2a_core::tga::TgaImageV1, m2a_core::tga::TgaWriterOptionsV1), String> {
    let image = serde_json::from_str(image_json).map_err(|_| {
        m5_boundary_error(
            "M5-TGA-IMAGE-JSON-INVALID",
            "imageJson",
            "image JSON does not match the strict public schema",
        )
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        m5_boundary_error(
            "M5-TGA-OPTIONS-JSON-INVALID",
            "optionsJson",
            "TGA options JSON does not match the strict public schema",
        )
    })?;
    Ok((image, options))
}

fn write_tga_artifact_json(
    image_json: &str,
    options_json: &str,
) -> Result<m2a_core::tga::TgaArtifactV1, String> {
    let (image, options) = parse_tga_json(image_json, options_json)?;
    m2a_core::tga::write_tga_v1(&image, &options).map_err(|error| serialize_json(&error))
}

/// Writes a deterministic TGA from strict image/options JSON.
#[wasm_bindgen(js_name = writeTgaV1)]
pub fn write_tga_v1(image_json: &str, options_json: &str) -> Result<Vec<u8>, JsValue> {
    write_tga_artifact_json(image_json, options_json)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the report for the same deterministic TGA core operation.
#[wasm_bindgen(js_name = writeTgaV1ReportJson)]
pub fn write_tga_v1_report_json(image_json: &str, options_json: &str) -> Result<String, JsValue> {
    write_tga_artifact_json(image_json, options_json)
        .map(|artifact| serialize_json(&artifact.report))
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_two_da_limits_json(limits_json: &str) -> Result<m2a_core::two_da::TwoDaLimitsV1, String> {
    serde_json::from_str(limits_json).map_err(|_| {
        m5_boundary_error(
            "M5-2DA-LIMITS-JSON-INVALID",
            "limitsJson",
            "2DA limits JSON does not match the strict public schema",
        )
    })
}

fn inspect_two_da_v2_json_inner(bytes: &[u8], limits_json: &str) -> Result<String, String> {
    let limits = parse_two_da_limits_json(limits_json)?;
    m2a_core::two_da::inspect_two_da_v2(bytes, &limits)
        .map(|inspection| serialize_json(&inspection))
        .map_err(|error| two_da_core_error_json(&error))
}

/// Inspects strict 2DA V2.0 bytes with caller-supplied strict limits JSON.
#[wasm_bindgen(js_name = inspectTwoDaV2Json)]
pub fn inspect_two_da_v2_json(bytes: &[u8], limits_json: &str) -> Result<String, JsValue> {
    inspect_two_da_v2_json_inner(bytes, limits_json).map_err(|error| JsValue::from_str(&error))
}

fn parse_two_da_append_json(
    request_json: &str,
    limits_json: &str,
) -> Result<
    (
        m2a_core::two_da::TwoDaAppendRequestV1,
        m2a_core::two_da::TwoDaLimitsV1,
    ),
    String,
> {
    let request = serde_json::from_str(request_json).map_err(|_| {
        m5_boundary_error(
            "M5-2DA-REQUEST-JSON-INVALID",
            "requestJson",
            "2DA append request JSON does not match the strict public schema",
        )
    })?;
    let limits = parse_two_da_limits_json(limits_json)?;
    Ok((request, limits))
}

fn append_two_da_row_artifact_json(
    bytes: &[u8],
    request_json: &str,
    limits_json: &str,
) -> Result<m2a_core::two_da::TwoDaAppendArtifactV1, String> {
    let (request, limits) = parse_two_da_append_json(request_json, limits_json)?;
    m2a_core::two_da::append_two_da_row_v1(bytes, &request, &limits)
        .map_err(|error| two_da_core_error_json(&error))
}

/// Appends one full-width row while preserving every source byte.
#[wasm_bindgen(js_name = appendTwoDaRowV1)]
pub fn append_two_da_row_v1(
    bytes: &[u8],
    request_json: &str,
    limits_json: &str,
) -> Result<Vec<u8>, JsValue> {
    append_two_da_row_artifact_json(bytes, request_json, limits_json)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the report for the same deterministic append core operation.
#[wasm_bindgen(js_name = appendTwoDaRowV1ReportJson)]
pub fn append_two_da_row_v1_report_json(
    bytes: &[u8],
    request_json: &str,
    limits_json: &str,
) -> Result<String, JsValue> {
    append_two_da_row_artifact_json(bytes, request_json, limits_json)
        .map(|artifact| serialize_json(&artifact.report))
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_hak_resources_json(resources_json: &str) -> Result<HakResourceDescriptorsV1, String> {
    let descriptors: HakResourceDescriptorsV1 =
        serde_json::from_str(resources_json).map_err(|_| {
            m5_boundary_error(
                "M5-HAK-RESOURCES-JSON-INVALID",
                "resourcesJson",
                "HAK resources JSON does not match the strict public schema",
            )
        })?;
    if descriptors.schema_version != 1 {
        return Err(m5_boundary_error(
            "M5-HAK-RESOURCES-JSON-INVALID",
            "resourcesJson",
            "HAK resources schemaVersion must be 1",
        ));
    }
    Ok(descriptors)
}

fn parse_hak_options_json(options_json: &str) -> Result<m2a_core::hak::HakWriterOptionsV1, String> {
    serde_json::from_str(options_json).map_err(|_| {
        m5_boundary_error(
            "M5-HAK-OPTIONS-JSON-INVALID",
            "optionsJson",
            "HAK options JSON does not match the strict public schema",
        )
    })
}

fn hak_range_error(path: &str, message: &str) -> String {
    m5_boundary_error("M5-HAK-PAYLOAD-RANGE-INVALID", path, message)
}

fn hak_allocation_error(message: &str) -> String {
    m5_boundary_error("M5-HAK-ALLOCATION-FAILED", "output", message)
}

fn clone_hak_string(value: &str) -> Result<String, String> {
    let mut output = String::new();
    output
        .try_reserve_exact(value.len())
        .map_err(|_| hak_allocation_error("could not reserve HAK resource resref"))?;
    output.push_str(value);
    Ok(output)
}

fn validate_hak_payload_ranges(
    payload_blob: &[u8],
    descriptors: &HakResourceDescriptorsV1,
) -> Result<(), String> {
    let mut ranges = Vec::new();
    ranges
        .try_reserve_exact(descriptors.resources.len())
        .map_err(|_| hak_allocation_error("could not reserve HAK payload range plan"))?;
    for (index, descriptor) in descriptors.resources.iter().enumerate() {
        let start = usize::try_from(descriptor.payload_offset).map_err(|_| {
            hak_range_error(
                &format!("resources[{index}].payloadOffset"),
                "payloadOffset does not fit this platform",
            )
        })?;
        if start > payload_blob.len() {
            return Err(hak_range_error(
                &format!("resources[{index}].payloadOffset"),
                "payloadOffset is outside payloadBlob",
            ));
        }
        let size = usize::try_from(descriptor.payload_size).map_err(|_| {
            hak_range_error(
                &format!("resources[{index}].payloadSize"),
                "payloadSize does not fit this platform",
            )
        })?;
        let end = start.checked_add(size).ok_or_else(|| {
            hak_range_error(
                &format!("resources[{index}].payloadSize"),
                "payload range overflows this platform",
            )
        })?;
        if end > payload_blob.len() {
            return Err(hak_range_error(
                &format!("resources[{index}].payloadSize"),
                "payload range extends past payloadBlob",
            ));
        }
        if size != 0 {
            ranges.push((start, end));
        }
    }
    ranges.sort_unstable();
    let mut cursor = 0usize;
    for (start, end) in ranges {
        if start < cursor {
            return Err(hak_range_error(
                "payloadBlob",
                "non-empty resource payload ranges overlap",
            ));
        }
        if start > cursor {
            return Err(hak_range_error(
                "payloadBlob",
                "resource payload ranges leave a gap",
            ));
        }
        cursor = end;
    }
    if cursor != payload_blob.len() {
        return Err(hak_range_error(
            "payloadBlob",
            "resource payload ranges do not consume exact payloadBlob",
        ));
    }

    Ok(())
}

fn materialize_hak_resources(
    payload_blob: &[u8],
    descriptors: &HakResourceDescriptorsV1,
) -> Result<Vec<m2a_core::hak::HakResourceInputV1>, String> {
    let mut resources = Vec::new();
    resources
        .try_reserve_exact(descriptors.resources.len())
        .map_err(|_| hak_allocation_error("could not reserve HAK resources"))?;
    for descriptor in &descriptors.resources {
        let start = descriptor.payload_offset as usize;
        let end = start + descriptor.payload_size as usize;
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(descriptor.payload_size as usize)
            .map_err(|_| hak_allocation_error("could not reserve HAK resource payload"))?;
        payload.extend_from_slice(&payload_blob[start..end]);
        resources.push(m2a_core::hak::HakResourceInputV1 {
            resref: clone_hak_string(&descriptor.resref)?,
            resource_type: descriptor.resource_type,
            payload,
        });
    }
    Ok(resources)
}

fn parse_hak_boundary(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<
    (
        Vec<m2a_core::hak::HakResourceInputV1>,
        m2a_core::hak::HakWriterOptionsV1,
    ),
    String,
> {
    let descriptors = parse_hak_resources_json(resources_json)?;
    let options = parse_hak_options_json(options_json)?;
    validate_hak_payload_ranges(payload_blob, &descriptors)?;
    m2a_core::hak::preflight_hak_v1(&descriptors.resources, &options)
        .map_err(|error| serialize_json(&error))?;
    let resources = materialize_hak_resources(payload_blob, &descriptors)?;
    Ok((resources, options))
}

fn write_hak_artifact_json(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<m2a_core::hak::HakArtifactV1, String> {
    let (resources, options) = parse_hak_boundary(payload_blob, resources_json, options_json)?;
    m2a_core::hak::write_hak_v1(&resources, &options).map_err(|error| serialize_json(&error))
}

/// Writes deterministic HAK bytes from one blob and strict resource descriptors.
#[wasm_bindgen(js_name = writeHakV1)]
pub fn write_hak_v1(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<Vec<u8>, JsValue> {
    write_hak_artifact_json(payload_blob, resources_json, options_json)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the report for the same deterministic HAK core operation.
#[wasm_bindgen(js_name = writeHakV1ReportJson)]
pub fn write_hak_v1_report_json(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    write_hak_artifact_json(payload_blob, resources_json, options_json)
        .map(|artifact| serialize_json(&artifact.report))
        .map_err(|error| JsValue::from_str(&error))
}

fn write_package_manifest_v1_json_inner(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<String, String> {
    let (resources, options) = parse_hak_boundary(payload_blob, resources_json, options_json)?;
    m2a_core::package::write_package_manifest_v1(&resources, &options)
        .map(|manifest| serialize_json(&manifest))
        .map_err(|error| serialize_json(&error))
}

/// Model-only package result returned from one core composition pass.
///
/// JavaScript receives HAK bytes separately from JSON metadata, so binary
/// payloads never cross the boundary as base64.
#[wasm_bindgen]
pub struct ModelPackageArtifactV1 {
    hak_bytes: Vec<u8>,
    report_json: String,
    manifest_json: String,
}

#[wasm_bindgen]
impl ModelPackageArtifactV1 {
    /// Transfers ownership of the HAK buffer to JavaScript exactly once.
    /// Later calls deterministically return an empty `Uint8Array`.
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }
}

fn write_model_package_v1_inner(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<ModelPackageArtifactV1, String> {
    let (resources, options) = parse_hak_boundary(payload_blob, resources_json, options_json)?;
    let artifact = m2a_core::package::write_model_package_v1(&resources, &options)
        .map_err(|error| serialize_json(&error))?;
    Ok(ModelPackageArtifactV1 {
        report_json: serialize_json(&artifact.hak.report),
        manifest_json: serialize_json(&artifact.manifest),
        hak_bytes: artifact.hak.payload,
    })
}

/// Composes ready binary MDL+MDX, TGA and appended appearance.2da payloads.
///
/// One call performs one HAK write and returns its bytes, report and exact
/// manifest. Call `takeHakBytes()` once to transfer the binary buffer without
/// cloning it.
#[wasm_bindgen(js_name = writeModelPackageV1)]
pub fn write_model_package_v1(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<ModelPackageArtifactV1, JsValue> {
    write_model_package_v1_inner(payload_blob, resources_json, options_json)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the deterministic manifest sidecar after successful HAK own-readback.
#[wasm_bindgen(js_name = writePackageManifestV1Json)]
pub fn write_package_manifest_v1_json(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    write_package_manifest_v1_json_inner(payload_blob, resources_json, options_json)
        .map_err(|error| JsValue::from_str(&error))
}

/// Browser-transferable result of the static-mesh plus animated-donor route.
/// The binary model buffer can be taken once; the deterministic JSON sidecars
/// remain available for audit and UI presentation.
#[wasm_bindgen]
pub struct AnimatedDonorModelArtifactV1 {
    model_bytes: Vec<u8>,
    report_json: String,
    readback_json: String,
    animations_json: String,
    conversion_json: String,
}

#[wasm_bindgen]
impl AnimatedDonorModelArtifactV1 {
    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = animationsJson)]
    pub fn animations_json(&self) -> String {
        self.animations_json.clone()
    }

    #[wasm_bindgen(getter, js_name = conversionJson)]
    pub fn conversion_json(&self) -> String {
        self.conversion_json.clone()
    }
}

/// Retargets a browser-selected static GLB onto a separate browser-selected
/// Meshy H1-style animated donor and emits one self-validated binary MDL.
#[wasm_bindgen(js_name = retargetStaticMeshToAnimatedDonorV1)]
pub fn retarget_static_mesh_to_animated_donor_v1(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options_json: &str,
) -> Result<AnimatedDonorModelArtifactV1, JsValue> {
    let options = serde_json::from_str::<m2a_core::mdl::MdlWriterOptionsV1>(writer_options_json)
        .map_err(|_| {
            JsValue::from_str(&mdl_writer_json_error(
                "M7-DONOR-WRITER-OPTIONS-JSON-INVALID",
                "writerOptionsJson",
                "writer options JSON does not match the strict public schema",
            ))
        })?;
    let artifact = m2a_core::animated_donor::retarget_static_mesh_to_animated_donor_v1(
        source_glb,
        animated_donor_glb,
        &options,
    )
    .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(AnimatedDonorModelArtifactV1 {
        model_bytes: artifact.model.payload,
        report_json: serialize_json(&artifact.report),
        readback_json: serialize_json(&artifact.model.inspection),
        animations_json: serialize_json(&artifact.animations),
        conversion_json: serialize_json(&artifact.conversion),
    })
}

/// Browser Studio result from the existing canonical model-only pipeline.
/// Binary buffers cross the boundary separately and can each be taken once.
#[wasm_bindgen]
pub struct StudioModelPackageArtifactV1 {
    hak_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    proof_module_bytes: Vec<u8>,
    report_json: String,
    manifest_json: String,
    summary_json: String,
    readback_json: String,
}

#[wasm_bindgen]
impl StudioModelPackageArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    /// Transfers the self-contained generated proof module exactly once.
    #[wasm_bindgen(js_name = takeProofModuleBytes)]
    pub fn take_proof_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.proof_module_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }

    #[wasm_bindgen(getter, js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.summary_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

/// Executes the canonical Rust model-only GLB -> MDL/TGA/2DA/HAK pipeline for
/// browser-selected bytes. No filesystem, DOM or alternate conversion path is
/// involved at this boundary.
#[wasm_bindgen(js_name = buildM6ModelPackageV1)]
pub fn build_m6_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    let artifact =
        m2a_core::model_pipeline::build_m6_model_package_v1(source_glb, appearance_two_da)
            .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;
    let readback = m2a_core::inspect_binary_mdl(&artifact.model)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        readback_json: serialize_json(&readback),
    })
}

/// Executes the constrained real-Meshy H1 Studio route for browser-selected
/// bytes.  It derives its clean-room rig from the selected GLB and returns the
/// same one-shot transferable artifact surface as the synthetic M6 proof.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV1)]
pub fn build_meshy_h1_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    let artifact =
        m2a_core::model_pipeline::build_meshy_h1_model_package_v1(source_glb, appearance_two_da)
            .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;
    let readback = m2a_core::inspect_binary_mdl(&artifact.model)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds the self-contained H1 profile only when all 42 direct-creature
/// states are present as explicitly named source animations. Unlike V1, this
/// boundary never manufactures gameplay clips by renaming the idle motion.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV2)]
pub fn build_meshy_h1_model_package_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_h1_model_package_v2_inner(source_glb, appearance_two_da)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_h1_model_package_v2_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, String> {
    let artifact = m2a_core::model_pipeline::build_meshy_h1_model_package_v2(
        source_glb,
        appearance_two_da,
        m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds a skinned Meshy humanoid with one preserved idle and 41 distinct
/// clean-room gameplay clips. This boundary never falls back to the historical
/// seven aliases of the same idle motion.
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidModelPackageV1)]
pub fn build_meshy_procedural_humanoid_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_model_package_v1_inner(source_glb, appearance_two_da)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_model_package_v1_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, String> {
    let artifact = m2a_core::model_pipeline::build_meshy_procedural_humanoid_model_package_v1(
        source_glb,
        appearance_two_da,
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds the self-contained 42-state H1 profile with exact caller-owned
/// animation event tables. Event timings are parsed from the strict V1 JSON
/// sidecar and are never inferred from native witness assets.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV3)]
pub fn build_meshy_h1_model_package_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    event_authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_h1_model_package_v3_inner(source_glb, appearance_two_da, event_authoring_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_h1_model_package_v3_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    event_authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let event_authoring = serde_json::from_str::<
        m2a_core::model_pipeline::DirectCreatureEventAuthoringV1,
    >(event_authoring_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "ANIMATION".to_owned(),
            code: "M6-ANIMATION-EVENT-AUTHORING-JSON".to_owned(),
            path: "eventAuthoringJson".to_owned(),
            message: "event authoring JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let artifact = m2a_core::model_pipeline::build_meshy_h1_model_package_v3(
        source_glb,
        appearance_two_da,
        m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        m2a_core::model_pipeline::DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
        &event_authoring,
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Returns the canonical Rust catalog projection used for cross-layer parity
/// tests. The browser does not maintain a second semantic catalog.
#[wasm_bindgen(js_name = directCreatureAnimationCatalogV1Json)]
pub fn direct_creature_animation_catalog_v1_json() -> String {
    serialize_json(&m2a_core::creature_animation_mapping::direct_creature_base_catalog_v1())
}

/// Validates a strict V1 authoring document and always returns structured JSON,
/// including malformed-input diagnostics.
#[wasm_bindgen(js_name = validateCreatureAnimationAuthoringV1)]
pub fn validate_creature_animation_authoring_v1(animation_authoring_json: &str) -> String {
    match serde_json::from_str::<m2a_core::creature_animation_mapping::CreatureAnimationAuthoringV1>(
        animation_authoring_json,
    ) {
        Ok(authoring) => serialize_json(
            &m2a_core::creature_animation_mapping::validate_creature_animation_authoring_v1(
                &authoring,
            ),
        ),
        Err(error) => serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "status": "BLOCKED",
            "mappedBaseSlotCount": 0,
            "reviewCount": 0,
            "blockingCount": 1,
            "customAnimationCount": 0,
            "authoringFingerprintSha256": "",
            "diagnostics": [{
                "schemaVersion": 1,
                "code": "M2A-ANIMATION-AUTHORING-JSON",
                "path": "animationAuthoringJson",
                "level": "BLOCKING",
                "message": format!("animation authoring JSON does not match the strict V1 schema: {error}"),
                "action": "Open a compatible saved mapping or recreate the document."
            }]
        })),
    }
}

/// Resolves the exact Base 42/custom output projection. Failures remain data
/// so the worker can render diagnostics without parsing thrown text.
#[wasm_bindgen(js_name = resolveCreatureAnimationMappingV1)]
pub fn resolve_creature_animation_mapping_v1(animation_authoring_json: &str) -> String {
    let validation_json = validate_creature_animation_authoring_v1(animation_authoring_json);
    let authoring = match serde_json::from_str::<
        m2a_core::creature_animation_mapping::CreatureAnimationAuthoringV1,
    >(animation_authoring_json)
    {
        Ok(authoring) => authoring,
        Err(_) => {
            return serialize_json(&serde_json::json!({
                "schemaVersion": 1,
                "ok": false,
                "validation": serde_json::from_str::<serde_json::Value>(&validation_json)
                    .unwrap_or(serde_json::Value::Null),
                "mapping": serde_json::Value::Null
            }));
        }
    };
    match m2a_core::creature_animation_mapping::resolve_creature_animation_mapping_v1(&authoring) {
        Ok(mapping) => serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "ok": true,
            "validation": serde_json::from_str::<serde_json::Value>(&validation_json)
                .unwrap_or(serde_json::Value::Null),
            "mapping": mapping
        })),
        Err(_) => serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "ok": false,
            "validation": serde_json::from_str::<serde_json::Value>(&validation_json)
                .unwrap_or(serde_json::Value::Null),
            "mapping": serde_json::Value::Null
        })),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EditableAnimationRigNodeBoundaryV1 {
    id: u32,
    name: String,
    parent_id: Option<u32>,
    translation: [f32; 3],
    rotation: [f32; 4],
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EditableAnimationSourceBoundaryV1 {
    schema_version: u32,
    source_revision: String,
    rig: Vec<EditableAnimationRigNodeBoundaryV1>,
    clip: Option<m2a_core::animation_studio::AuthoredAnimationClipV1>,
}

fn animation_studio_boundary_diagnostic(
    code: &str,
    path: &str,
    message: impl Into<String>,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 1,
        "code": code,
        "path": path,
        "level": "BLOCKING",
        "message": message.into(),
        "action": "Repair the exact Animation Studio document or re-inspect the immutable source GLB."
    })
}

fn animation_studio_json_error(code: &str, path: &str, message: &str) -> String {
    serialize_json(&serde_json::json!({
        "schemaVersion": 1,
        "stage": "ANIMATION",
        "code": code,
        "path": path,
        "message": message
    }))
}

/// Projects the exact immutable source GLB to the canonical output rig. When
/// `clip_name` is supplied, the named source clip is cloned to an editable
/// draft without modifying or embedding the source GLB.
#[wasm_bindgen(js_name = inspectEditableAnimationSourceV1)]
pub fn inspect_editable_animation_source_v1(
    source_glb: &[u8],
    clip_name: Option<String>,
) -> Result<String, JsValue> {
    inspect_editable_animation_source_v1_inner(source_glb, clip_name.as_deref())
        .map_err(|error| JsValue::from_str(&error))
}

fn inspect_editable_animation_source_v1_inner(
    source_glb: &[u8],
    clip_name: Option<&str>,
) -> Result<String, String> {
    let inspection = m2a_core::model_pipeline::inspect_editable_animation_source_v1(source_glb)
        .map_err(|error| serialize_json(&error))?;
    let clip = clip_name
        .filter(|name| !name.trim().is_empty())
        .map(|name| {
            let source_clip = inspection
                .animations
                .clips
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    animation_studio_json_error(
                        "M2A-ANIMATION-EDIT-SCHEMA",
                        "clipName",
                        "requested source clip does not exist in the canonical output-rig inspection",
                    )
                })?;
            m2a_core::animation_studio::clone_source_clip_for_editing_v1(
                &inspection.animations,
                &source_clip.name,
                m2a_core::animation_studio::AuthoredAnimationClipInputV1 {
                    id: format!("source-clip-{}", source_clip.name.to_ascii_lowercase()),
                    name: source_clip.name.clone(),
                    source_revision: inspection.source_revision.clone(),
                    length_seconds: source_clip.length_seconds,
                    transition_seconds: source_clip.transition_seconds,
                    animation_root: source_clip.animation_root.clone(),
                },
            )
            .map_err(|error| serialize_json(&error))
        })
        .transpose()?;
    serialize_json_result(&EditableAnimationSourceBoundaryV1 {
        schema_version: 1,
        source_revision: inspection.source_revision,
        rig: inspection
            .rig
            .nodes
            .into_iter()
            .map(|node| EditableAnimationRigNodeBoundaryV1 {
                id: node.node_id,
                name: node.name,
                parent_id: node.parent_id,
                translation: node.translation,
                rotation: node.rotation,
            })
            .collect(),
        clip,
    })
}

/// Validates the strict Studio document against the exact canonical output rig
/// and returns structured diagnostics instead of maintaining a JS validator.
#[wasm_bindgen(js_name = validateAnimationStudioDocumentV1)]
pub fn validate_animation_studio_document_v1(
    animation_studio_document_json: &str,
    source_glb: &[u8],
) -> String {
    validate_animation_studio_document_v1_inner(animation_studio_document_json, source_glb)
}

fn validate_animation_studio_document_v1_inner(
    animation_studio_document_json: &str,
    source_glb: &[u8],
) -> String {
    let document = match serde_json::from_str::<m2a_core::animation_studio::AnimationStudioDocumentV1>(
        animation_studio_document_json,
    ) {
        Ok(document) => document,
        Err(error) => {
            return serialize_json(&serde_json::json!({
                "schemaVersion": 1,
                "status": "BLOCKED",
                "sourceRevision": "",
                "documentFingerprintSha256": "",
                "diagnostics": [animation_studio_boundary_diagnostic(
                    "M2A-ANIMATION-EDIT-SCHEMA",
                    "animationStudioDocumentJson",
                    format!("document JSON does not match the strict V1 schema: {error}"),
                )]
            }));
        }
    };
    let fingerprint =
        m2a_core::animation_studio::fingerprint_animation_studio_document_v1(&document);
    let inspection =
        match m2a_core::model_pipeline::inspect_editable_animation_source_v1(source_glb) {
            Ok(inspection) => inspection,
            Err(error) => {
                return serialize_json(&serde_json::json!({
                    "schemaVersion": 1,
                    "status": "BLOCKED",
                    "sourceRevision": document.source_revision,
                    "documentFingerprintSha256": fingerprint,
                    "diagnostics": [animation_studio_boundary_diagnostic(
                        &error.code,
                        &error.path,
                        error.message,
                    )]
                }));
            }
        };
    let diagnostics = match m2a_core::animation_studio::materialize_animation_studio_document_v1(
        &document,
        &inspection.rig,
    ) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics,
    };
    serialize_json(&serde_json::json!({
        "schemaVersion": 1,
        "status": if diagnostics.is_empty() { "READY" } else { "BLOCKED" },
        "sourceRevision": document.source_revision,
        "documentFingerprintSha256": fingerprint,
        "diagnostics": diagnostics
    }))
}

/// Materializes a strict Studio document to canonical MDL animation IR. The
/// adapter throws structured JSON on any schema, rig, or authoring diagnostic.
#[wasm_bindgen(js_name = materializeAnimationStudioDocumentV1)]
pub fn materialize_animation_studio_document_v1(
    animation_studio_document_json: &str,
    source_glb: &[u8],
) -> Result<String, JsValue> {
    materialize_animation_studio_document_v1_inner(animation_studio_document_json, source_glb)
        .map_err(|error| JsValue::from_str(&error))
}

fn materialize_animation_studio_document_v1_inner(
    animation_studio_document_json: &str,
    source_glb: &[u8],
) -> Result<String, String> {
    let document = serde_json::from_str::<m2a_core::animation_studio::AnimationStudioDocumentV1>(
        animation_studio_document_json,
    )
    .map_err(|_| {
        animation_studio_json_error(
            "M2A-ANIMATION-EDIT-SCHEMA",
            "animationStudioDocumentJson",
            "document JSON does not match the strict V1 schema",
        )
    })?;
    let inspection = m2a_core::model_pipeline::inspect_editable_animation_source_v1(source_glb)
        .map_err(|error| serialize_json(&error))?;
    let materialized = m2a_core::animation_studio::materialize_animation_studio_document_v1(
        &document,
        &inspection.rig,
    )
    .map_err(|diagnostics| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "stage": "ANIMATION",
            "code": diagnostics
                .first()
                .map(|diagnostic| diagnostic.code.as_str())
                .unwrap_or("M2A-ANIMATION-EDIT-SCHEMA"),
            "path": diagnostics
                .first()
                .map(|diagnostic| diagnostic.path.as_str())
                .unwrap_or("animationStudioDocument"),
            "message": diagnostics
                .first()
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("Animation Studio materialization failed"),
            "diagnostics": diagnostics
        }))
    })?;
    serialize_json_result(&materialized)
}

/// Produces one canonical preview clip from an authored Studio document.
/// Draft status is not a preview blocker; build-time exact-rig validation
/// remains owned by `validateAnimationStudioDocumentV1`.
#[wasm_bindgen(js_name = previewAuthoredAnimationClipV1)]
pub fn preview_authored_animation_clip_v1(
    animation_studio_document_json: &str,
    clip_id: &str,
) -> Result<String, JsValue> {
    preview_authored_animation_clip_v1_inner(animation_studio_document_json, clip_id)
        .map_err(|error| JsValue::from_str(&error))
}

fn preview_authored_animation_clip_v1_inner(
    animation_studio_document_json: &str,
    clip_id: &str,
) -> Result<String, String> {
    let document = serde_json::from_str::<m2a_core::animation_studio::AnimationStudioDocumentV1>(
        animation_studio_document_json,
    )
    .map_err(|_| {
        animation_studio_json_error(
            "M2A-ANIMATION-EDIT-SCHEMA",
            "animationStudioDocumentJson",
            "Animation Studio JSON does not match the strict V1 schema",
        )
    })?;
    let document_diagnostics =
        m2a_core::animation_studio::validate_animation_studio_schema_v1(&document);
    if !document_diagnostics.is_empty() {
        return Err(serialize_json(&document_diagnostics));
    }
    let clip = document
        .authored_clips
        .iter()
        .find(|clip| clip.id == clip_id)
        .cloned()
        .ok_or_else(|| {
            animation_studio_json_error(
                "M2A-ANIMATION-EDIT-SCHEMA",
                "clipId",
                "selected authored clip id does not exist in the Studio document",
            )
        })?;
    let mut node_ids = clip
        .tracks
        .iter()
        .map(|track| track.target_node_id)
        .collect::<Vec<_>>();
    node_ids.sort_unstable();
    node_ids.dedup();
    if node_ids.is_empty() {
        node_ids.push(0);
    }
    let preview_rig = m2a_core::animation_studio::AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: document.source_revision.clone(),
        animation_root: clip.animation_root.clone(),
        nodes: node_ids
            .into_iter()
            .map(
                |node_id| m2a_core::animation_studio::AnimationStudioRigNodeV1 {
                    node_id,
                    name: format!("preview-node-{node_id}"),
                    parent_id: None,
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                },
            )
            .collect(),
    };
    let diagnostics =
        m2a_core::animation_studio::validate_authored_animation_clip_v1(&clip, &preview_rig);
    if !diagnostics.is_empty() {
        return Err(serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "stage": "ANIMATION",
            "code": diagnostics[0].code,
            "path": diagnostics[0].path,
            "message": diagnostics[0].message,
            "diagnostics": diagnostics
        })));
    }
    let mut preview_clip = clip;
    preview_clip.status = m2a_core::animation_studio::AuthoredAnimationClipStatusV1::Valid;
    let document = m2a_core::animation_studio::AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: document.source_revision,
        authoring_revision: preview_clip.revision,
        status: m2a_core::animation_studio::AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![preview_clip],
    };
    let materialized = m2a_core::animation_studio::materialize_animation_studio_document_v1(
        &document,
        &preview_rig,
    )
    .map_err(|diagnostics| serialize_json(&diagnostics))?;
    serialize_json_result(&serde_json::json!({
        "schemaVersion": 1,
        "sourceRevision": document.source_revision,
        "clip": materialized.clips.into_iter().next()
    }))
}

/// Additive authored-animation build boundary. V2/V3 remain untouched.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV4)]
pub fn build_meshy_h1_model_package_v4(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring_json: &str,
    event_authoring_json: Option<String>,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_h1_model_package_v4_inner(
        source_glb,
        appearance_two_da,
        animation_authoring_json,
        event_authoring_json.as_deref(),
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_h1_model_package_v4_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring_json: &str,
    event_authoring_json: Option<&str>,
) -> Result<StudioModelPackageArtifactV1, String> {
    let authoring = serde_json::from_str::<
        m2a_core::creature_animation_mapping::CreatureAnimationAuthoringV1,
    >(animation_authoring_json)
    .map_err(|_| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "stage": "ANIMATION",
            "code": "M2A-ANIMATION-AUTHORING-JSON",
            "path": "animationAuthoringJson",
            "message": "animation authoring JSON does not match the strict V1 schema"
        }))
    })?;
    let event_authoring = event_authoring_json
        .filter(|json| !json.trim().is_empty())
        .map(|json| {
            serde_json::from_str::<m2a_core::model_pipeline::DirectCreatureEventAuthoringV1>(json)
                .map_err(|_| {
                    serialize_json(&serde_json::json!({
                        "schemaVersion": 1,
                        "stage": "ANIMATION",
                        "code": "M6-ANIMATION-EVENT-AUTHORING-JSON",
                        "path": "eventAuthoringJson",
                        "message": "event authoring JSON does not match the strict V1 schema"
                    }))
                })
        })
        .transpose()?;
    let artifact = m2a_core::model_pipeline::build_meshy_h1_model_package_v4_with_events(
        source_glb,
        appearance_two_da,
        &authoring,
        event_authoring.as_ref(),
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Additive Animation Studio build boundary. It accepts only the exact V2
/// mapping plus strict Studio V1 document and leaves every V4 byte path intact.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV5)]
pub fn build_meshy_h1_model_package_v5(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring_json: &str,
    animation_studio_document_json: &str,
    event_authoring_json: Option<String>,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_h1_model_package_v5_inner(
        source_glb,
        appearance_two_da,
        animation_authoring_json,
        animation_studio_document_json,
        event_authoring_json.as_deref(),
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_h1_model_package_v5_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring_json: &str,
    animation_studio_document_json: &str,
    event_authoring_json: Option<&str>,
) -> Result<StudioModelPackageArtifactV1, String> {
    let authoring =
        serde_json::from_str::<m2a_core::animation_studio::CreatureAnimationAuthoringV2>(
            animation_authoring_json,
        )
        .map_err(|_| {
            animation_studio_json_error(
                "M2A-ANIMATION-EDIT-SCHEMA",
                "animationAuthoringJson",
                "animation authoring JSON does not match the strict V2 schema",
            )
        })?;
    let studio = serde_json::from_str::<m2a_core::animation_studio::AnimationStudioDocumentV1>(
        animation_studio_document_json,
    )
    .map_err(|_| {
        animation_studio_json_error(
            "M2A-ANIMATION-EDIT-SCHEMA",
            "animationStudioDocumentJson",
            "Animation Studio JSON does not match the strict V1 schema",
        )
    })?;
    let event_authoring = event_authoring_json
        .filter(|json| !json.trim().is_empty())
        .map(|json| {
            serde_json::from_str::<m2a_core::model_pipeline::DirectCreatureEventAuthoringV1>(json)
                .map_err(|_| {
                    animation_studio_json_error(
                        "M6-ANIMATION-EVENT-AUTHORING-JSON",
                        "eventAuthoringJson",
                        "event authoring JSON does not match the strict V1 schema",
                    )
                })
        })
        .transpose()?;
    let artifact = m2a_core::model_pipeline::build_meshy_h1_model_package_v5_with_events(
        source_glb,
        appearance_two_da,
        &authoring,
        &studio,
        event_authoring.as_ref(),
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Executes the separate M0 control lane for one static, unskinned Meshy GLB.
/// The returned HAK/MOD names are distinct from H1 and the MOD uses the
/// validated, self-contained M0 vertical-slice area.
#[wasm_bindgen(js_name = buildMeshyM0StaticRigidPackageV1)]
pub fn build_meshy_m0_static_rigid_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    let artifact = m2a_core::model_pipeline::build_meshy_m0_static_rigid_package_v1(
        source_glb,
        appearance_two_da,
    )
    .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;
    let readback = m2a_core::inspect_binary_mdl(&artifact.model)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        readback_json: serialize_json(&readback),
    })
}

fn build_meshy_static_placeable_package_v1_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v1(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

fn build_meshy_static_placeable_package_v2_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV1,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v2(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

fn finish_static_placeable_artifact_v1(
    artifact: m2a_core::placeable::StaticPlaceablePackageArtifactV1,
    identity: &m2a_core::placeable::StaticPlaceableIdentityV1,
) -> Result<StudioModelPackageArtifactV1, String> {
    let hak = m2a_core::erf::ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| serialize_json(&error))?;
    let model = hak
        .find(
            &identity.model_resref,
            m2a_core::placeable::MDL_RESOURCE_TYPE,
        )
        .map_err(|error| serialize_json(&error))?
        .to_vec();
    let readback = m2a_core::inspect_binary_mdl(&model).map_err(|error| serialize_json(&error))?;
    let report_json = serialize_json(&artifact.report);

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak_payload,
        model_bytes: model,
        proof_module_bytes: artifact.module_payload,
        report_json: report_json.clone(),
        manifest_json: report_json.clone(),
        summary_json: report_json,
        readback_json: serialize_json(&readback),
    })
}

/// Inspects one placeable GLB and returns the default immutable element
/// authoring document plus stable connected-component identities.
#[wasm_bindgen(js_name = inspectMeshyStaticPlaceableAuthoringV1)]
pub fn inspect_meshy_static_placeable_authoring_v1(source_glb: &[u8]) -> Result<String, JsValue> {
    m2a_core::placeable::inspect_meshy_static_placeable_authoring_v1(source_glb)
        .map(|bootstrap| serialize_json(&bootstrap))
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

/// Executes the static Meshy GLB -> common model IR -> placeable resource
/// resolver pipeline. HAK, MOD and model bytes are transferred separately;
/// the three JSON getters expose the same immutable machine-readable report.
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV1)]
pub fn build_meshy_static_placeable_package_v1(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v1_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Executes the authored placeable lane. The immutable authoring document is
/// applied before the shared Profile A conversion, while PWK uses the collision
/// projection of the same document.
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV2)]
pub fn build_meshy_static_placeable_package_v2(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v2_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
    )
    .map_err(|error| JsValue::from_str(&error))
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

fn build_meshy_static_tile_package_v1_inner(
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

/// Validates the strict, versioned M7 corpus manifest through `m2a-core`.
#[wasm_bindgen(js_name = validateM7CorpusManifestV1Json)]
pub fn validate_m7_corpus_manifest_v1_json(manifest_json: &str) -> String {
    match m2a_core::m7_corpus::parse_m7_corpus_manifest_v1(manifest_json.as_bytes()) {
        Ok(manifest) => serialize_json(&manifest),
        Err(error) => serialize_json(&error),
    }
}

/// Inspects M7 intake from borrowed slices of one browser-owned binary blob.
#[wasm_bindgen(js_name = inspectM7CorpusIntakeV1Json)]
pub fn inspect_m7_corpus_intake_v1_json(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> String {
    match inspect_m7_corpus_intake_v1_inner(manifest_json, payload_blob, descriptors_json) {
        Ok(report) => serialize_json(&report),
        Err(error) => error,
    }
}

/// Runs the existing canonical M7 batch and returns its report plus ordered
/// proof packets. Ready humanoids are materialized only by the canonical M6
/// constructor; the other routes remain explicit M7-V5 deferrals in core.
#[wasm_bindgen(js_name = buildM7CorpusBatchV1)]
pub fn build_m7_corpus_batch_v1(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> Result<String, JsValue> {
    build_m7_corpus_batch_v1_inner(manifest_json, payload_blob, descriptors_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_m7_boundary<'a>(
    manifest_json: &str,
    payload_blob: &'a [u8],
    descriptors_json: &'a str,
) -> Result<
    (
        m2a_core::m7_corpus::M7CorpusManifestV1,
        M7PayloadDescriptorsV1,
    ),
    String,
> {
    let manifest = m2a_core::m7_corpus::parse_m7_corpus_manifest_v1(manifest_json.as_bytes())
        .map_err(|error| serialize_json(&error))?;
    let descriptors: M7PayloadDescriptorsV1 =
        serde_json::from_str(descriptors_json).map_err(|_| {
            m7_boundary_error(
                "M7-WASM-DESCRIPTORS-JSON-INVALID",
                "descriptorsJson",
                "payload descriptors JSON does not match the public schema",
            )
        })?;
    if descriptors.schema_version != 1 {
        return Err(m7_boundary_error(
            "M7-WASM-DESCRIPTORS-SCHEMA-UNSUPPORTED",
            "descriptorsJson.schemaVersion",
            format!("expected schema 1, got {}", descriptors.schema_version),
        ));
    }
    validate_m7_blob_layout(payload_blob, &descriptors.payloads)?;
    validate_m7_descriptor_semantics(&manifest, &descriptors.payloads)?;
    Ok((manifest, descriptors))
}

fn inspect_m7_corpus_intake_v1_inner(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> Result<m2a_core::m7_corpus::M7CorpusIntakeReportV1, String> {
    let (manifest, descriptors) = parse_m7_boundary(manifest_json, payload_blob, descriptors_json)?;
    let payloads = m7_source_payloads(payload_blob, &descriptors.payloads)?;
    m2a_core::m7_corpus::inspect_m7_corpus_intake_v1(&manifest, &payloads)
        .map_err(|error| serialize_json(&error))
}

fn build_m7_corpus_batch_v1_inner(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> Result<String, String> {
    use m2a_core::m7_corpus::{M7CorpusEntryV1, M7IntakeStatusV1};

    let (manifest, descriptors) = parse_m7_boundary(manifest_json, payload_blob, descriptors_json)?;
    let payloads = m7_source_payloads(payload_blob, &descriptors.payloads)?;
    let intake = m2a_core::m7_corpus::inspect_m7_corpus_intake_v1(&manifest, &payloads)
        .map_err(|error| serialize_json(&error))?;
    let mut canonical_artifacts = Vec::new();
    if intake.status == M7IntakeStatusV1::ReadyForM7V5 {
        for entry in &manifest.samples {
            let M7CorpusEntryV1::RiggedHumanoidSourceClips {
                sample_id, source, ..
            } = entry
            else {
                continue;
            };
            if intake
                .samples
                .iter()
                .find(|sample| sample.sample_id == *sample_id)
                .is_none_or(|sample| sample.status != M7IntakeStatusV1::ReadyForM7V5)
            {
                continue;
            }
            let Some(source) = source else { continue };
            let source_bytes = payloads
                .iter()
                .find(|payload| {
                    payload
                        .relative_path
                        .eq_ignore_ascii_case(&source.relative_path)
                })
                .map(|payload| payload.bytes)
                .ok_or_else(|| {
                    m7_boundary_error(
                        "M7-WASM-SOURCE-PAYLOAD-MISSING",
                        "descriptorsJson.payloads",
                        format!("missing SOURCE descriptor for sample {sample_id:?}"),
                    )
                })?;
            let appearance_bytes =
                m7_appearance_payload(payload_blob, &descriptors.payloads, sample_id)?;
            canonical_artifacts.push(
                m2a_core::m7_corpus::M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6(
                    sample_id,
                    source_bytes,
                    appearance_bytes,
                )
                .map_err(|error| serialize_json(&error))?,
            );
        }
    }
    let artifact =
        m2a_core::m7_corpus::build_m7_corpus_batch_v1(&manifest, &payloads, &canonical_artifacts)
            .map_err(|error| serialize_json(&error))?;
    serialize_json_result(&M7BatchBoundaryOutputV1 {
        schema_version: 1,
        report: artifact.report,
        packets: artifact
            .packets
            .into_iter()
            .map(|packet| packet.packet)
            .collect(),
    })
}

fn validate_m7_blob_layout(
    payload_blob: &[u8],
    descriptors: &[M7PayloadDescriptorV1],
) -> Result<(), String> {
    let mut ranges = descriptors
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            let start = usize::try_from(descriptor.offset()).map_err(|_| {
                m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-INVALID",
                    format!("descriptorsJson.payloads[{index}].payloadOffset"),
                    "payload offset does not fit this platform",
                )
            })?;
            let size = usize::try_from(descriptor.size()).map_err(|_| {
                m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-INVALID",
                    format!("descriptorsJson.payloads[{index}].payloadSize"),
                    "payload size does not fit this platform",
                )
            })?;
            if size == 0 {
                return Err(m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-EMPTY",
                    format!("descriptorsJson.payloads[{index}].payloadSize"),
                    "payload descriptors must cover at least one byte",
                ));
            }
            let end = start.checked_add(size).ok_or_else(|| {
                m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-OVERFLOW",
                    format!("descriptorsJson.payloads[{index}]"),
                    "payload range overflows address space",
                )
            })?;
            if end > payload_blob.len() {
                return Err(m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-OOB",
                    format!("descriptorsJson.payloads[{index}]"),
                    format!(
                        "payload range ends at {end}, blob length is {}",
                        payload_blob.len()
                    ),
                ));
            }
            Ok((start, end, index))
        })
        .collect::<Result<Vec<_>, String>>()?;
    ranges.sort_unstable();
    let mut cursor = 0_usize;
    for (start, end, index) in ranges {
        if start != cursor {
            let code = if start < cursor {
                "M7-WASM-PAYLOAD-RANGE-OVERLAP"
            } else {
                "M7-WASM-PAYLOAD-RANGE-GAP"
            };
            return Err(m7_boundary_error(
                code,
                format!("descriptorsJson.payloads[{index}]"),
                format!("expected next payload offset {cursor}, got {start}"),
            ));
        }
        cursor = end;
    }
    if cursor != payload_blob.len() {
        return Err(m7_boundary_error(
            "M7-WASM-PAYLOAD-RANGE-GAP",
            "descriptorsJson.payloads",
            format!(
                "descriptors cover {cursor} of {} blob bytes",
                payload_blob.len()
            ),
        ));
    }
    Ok(())
}

fn validate_m7_descriptor_semantics(
    manifest: &m2a_core::m7_corpus::M7CorpusManifestV1,
    descriptors: &[M7PayloadDescriptorV1],
) -> Result<(), String> {
    use m2a_core::m7_corpus::M7CorpusEntryV1;

    let humanoid_sample_ids = manifest
        .samples
        .iter()
        .filter_map(|entry| match entry {
            M7CorpusEntryV1::RiggedHumanoidSourceClips { sample_id, .. } => Some(sample_id),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let all_sample_ids = manifest
        .samples
        .iter()
        .map(M7CorpusEntryV1::sample_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut appearances = std::collections::BTreeSet::new();
    for (index, descriptor) in descriptors.iter().enumerate() {
        let M7PayloadDescriptorV1::RiggedHumanoidAppearance2da { sample_id, .. } = descriptor
        else {
            continue;
        };
        if !all_sample_ids.contains(sample_id.as_str()) {
            return Err(m7_boundary_error(
                "M7-WASM-APPEARANCE-2DA-SAMPLE-UNKNOWN",
                format!("descriptorsJson.payloads[{index}].sampleId"),
                format!("appearance descriptor names unknown sample {sample_id:?}"),
            ));
        }
        if !humanoid_sample_ids.contains(sample_id) {
            return Err(m7_boundary_error(
                "M7-WASM-APPEARANCE-2DA-ROLE-MISMATCH",
                format!("descriptorsJson.payloads[{index}].sampleId"),
                format!("appearance descriptor targets non-humanoid sample {sample_id:?}"),
            ));
        }
        if !appearances.insert(sample_id.to_ascii_lowercase()) {
            return Err(m7_boundary_error(
                "M7-WASM-APPEARANCE-2DA-DUPLICATE",
                format!("descriptorsJson.payloads[{index}].sampleId"),
                format!("duplicate appearance descriptor for sample {sample_id:?}"),
            ));
        }
    }
    Ok(())
}

fn m7_source_payloads<'a>(
    payload_blob: &'a [u8],
    descriptors: &'a [M7PayloadDescriptorV1],
) -> Result<Vec<m2a_core::m7_corpus::M7SourcePayloadV1<'a>>, String> {
    let mut paths = std::collections::BTreeSet::new();
    descriptors
        .iter()
        .enumerate()
        .filter_map(|(index, descriptor)| match descriptor {
            M7PayloadDescriptorV1::Source {
                relative_path,
                payload_offset,
                payload_size,
            } => Some((index, relative_path, *payload_offset, *payload_size)),
            M7PayloadDescriptorV1::RiggedHumanoidAppearance2da { .. } => None,
        })
        .map(|(index, relative_path, offset, size)| {
            if !paths.insert(relative_path.to_ascii_lowercase()) {
                return Err(m7_boundary_error(
                    "M7-WASM-SOURCE-DESCRIPTOR-DUPLICATE",
                    format!("descriptorsJson.payloads[{index}].relativePath"),
                    "SOURCE relative paths must be unique case-insensitively",
                ));
            }
            let start = offset as usize;
            let end = start + size as usize;
            Ok(m2a_core::m7_corpus::M7SourcePayloadV1 {
                relative_path,
                bytes: &payload_blob[start..end],
            })
        })
        .collect()
}

fn m7_appearance_payload<'a>(
    payload_blob: &'a [u8],
    descriptors: &'a [M7PayloadDescriptorV1],
    sample_id: &str,
) -> Result<&'a [u8], String> {
    let matches = descriptors
        .iter()
        .filter_map(|descriptor| match descriptor {
            M7PayloadDescriptorV1::RiggedHumanoidAppearance2da {
                sample_id: candidate,
                payload_offset,
                payload_size,
            } if candidate == sample_id => Some((*payload_offset, *payload_size)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(m7_boundary_error(
            "M7-WASM-APPEARANCE-2DA-CARDINALITY",
            "descriptorsJson.payloads",
            format!(
                "ready humanoid sample {sample_id:?} requires exactly one RIGGED_HUMANOID_APPEARANCE_2DA descriptor, got {}",
                matches.len()
            ),
        ));
    }
    let (offset, size) = matches[0];
    let start = offset as usize;
    Ok(&payload_blob[start..start + size as usize])
}

fn m7_boundary_error(
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> String {
    serialize_json(&M7BoundaryErrorV1 {
        schema_version: 1,
        code,
        path: path.into(),
        message: message.into(),
    })
}

fn serialize_json_result<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|_| SERIALIZATION_ERROR_JSON.to_owned())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod m7_native_tests {
    use super::{
        M7BatchBoundaryOutputV1, M7PayloadDescriptorV1, M7PayloadDescriptorsV1,
        build_m7_corpus_batch_v1_inner, inspect_m7_corpus_intake_v1_inner, serialize_json,
        serialize_json_result, validate_m7_corpus_manifest_v1_json,
    };
    use m2a_core::m7_corpus::{
        M7ByteIdentityV1, M7CanonicalPipelineArtifactV1, M7CorpusEntryV1, M7CorpusManifestV1,
        M7OriginalSourceProvenanceV1, M7SourceDescriptorV1, M7SourcePayloadV1, M7SourceProviderV1,
        M7StaticResourceKindV1,
    };
    use sha2::{Digest, Sha256};

    const DEFERRED_MANIFEST: &str = r#"{
      "schemaVersion":1,
      "corpusId":"browser_corpus",
      "artDirectionApprovalId":null,
      "samples":[
        {"role":"RIGGED_HUMANOID_SOURCE_CLIPS","sampleId":"humanoid","source":null,"requiredSourceClipNames":["walk"]},
        {"role":"NON_HUMANOID_REFERENCE_SUPERMODEL","sampleId":"creature","source":null,"referenceSupermodel":"c_dog"},
        {"role":"STATIC_PLACEABLE_OR_ITEM","sampleId":"placeable","source":null,"resourceKind":"PLACEABLE"}
      ]
    }"#;
    const EMPTY_DESCRIPTORS: &str = r#"{"schemaVersion":1,"payloads":[]}"#;
    const READY_BATCH_JSON_SHA256: &str =
        "e5c2535ad0221f09e97fa4de468b21f8c82fefd8c563ce2bb15d7f5109536cc2";
    const APPEARANCE: &[u8] =
        include_bytes!("../../../apps/studio-web/tests/fixtures/appearance.2da");

    fn source_descriptor(path: &str, bytes: &[u8], task: &str) -> M7SourceDescriptorV1 {
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
        fn normalize_js_integer_numbers(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Number(number) => {
                    if number.is_f64()
                        && let Some(float) = number.as_f64()
                        && float.fract() == 0.0
                    {
                        if float >= 0.0 && float <= u64::MAX as f64 {
                            *number = serde_json::Number::from(float as u64);
                        } else if float >= i64::MIN as f64 && float <= i64::MAX as f64 {
                            *number = serde_json::Number::from(float as i64);
                        }
                    }
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        normalize_js_integer_numbers(value);
                    }
                }
                serde_json::Value::Object(values) => {
                    for value in values.values_mut() {
                        normalize_js_integer_numbers(value);
                    }
                }
                _ => {}
            }
        }

        let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_len;
        let mut json: serde_json::Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
        let root = json.as_object_mut().unwrap();
        root.remove("skins");
        root.remove("animations");
        for node in root["nodes"].as_array_mut().unwrap() {
            node.as_object_mut().unwrap().remove("skin");
        }
        normalize_js_integer_numbers(&mut json);
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
            corpus_id: "m7-wasm-ready-fixture".to_owned(),
            art_direction_approval_id: Some("owned-test-approval".to_owned()),
            samples: vec![
                M7CorpusEntryV1::RiggedHumanoidSourceClips {
                    sample_id: "humanoid".to_owned(),
                    source: Some(source_descriptor(
                        "models/humanoid.glb",
                        &humanoid,
                        "task-h",
                    )),
                    required_source_clip_names: vec!["owned-linear-pause".to_owned()],
                },
                M7CorpusEntryV1::NonHumanoidReferenceSupermodel {
                    sample_id: "creature".to_owned(),
                    source: Some(source_descriptor(
                        "models/creature.glb",
                        &static_glb,
                        "task-c",
                    )),
                    reference_supermodel: "c_horror".to_owned(),
                },
                M7CorpusEntryV1::StaticPlaceableOrItem {
                    sample_id: "static-prop".to_owned(),
                    source: Some(source_descriptor(
                        "models/static.glb",
                        &static_glb,
                        "task-s",
                    )),
                    resource_kind: M7StaticResourceKindV1::Placeable,
                },
            ],
        };
        (manifest, humanoid, static_glb)
    }

    fn append_payload(
        blob: &mut Vec<u8>,
        payloads: &mut Vec<M7PayloadDescriptorV1>,
        role: impl FnOnce(u32, u32) -> M7PayloadDescriptorV1,
        bytes: &[u8],
    ) {
        let offset = blob.len() as u32;
        blob.extend_from_slice(bytes);
        payloads.push(role(offset, bytes.len() as u32));
    }

    fn ready_boundary(
        include_appearance: bool,
    ) -> (M7CorpusManifestV1, Vec<u8>, Vec<u8>, Vec<u8>, String) {
        let (manifest, humanoid, static_glb) = ready_corpus();
        let mut blob = Vec::new();
        let mut payloads = Vec::new();
        for (path, bytes) in [
            ("models/humanoid.glb", humanoid.as_slice()),
            ("models/creature.glb", static_glb.as_slice()),
            ("models/static.glb", static_glb.as_slice()),
        ] {
            append_payload(
                &mut blob,
                &mut payloads,
                |payload_offset, payload_size| M7PayloadDescriptorV1::Source {
                    relative_path: path.to_owned(),
                    payload_offset,
                    payload_size,
                },
                bytes,
            );
        }
        if include_appearance {
            append_payload(
                &mut blob,
                &mut payloads,
                |payload_offset, payload_size| M7PayloadDescriptorV1::RiggedHumanoidAppearance2da {
                    sample_id: "humanoid".to_owned(),
                    payload_offset,
                    payload_size,
                },
                APPEARANCE,
            );
        }
        let descriptors = serde_json::to_string(&M7PayloadDescriptorsV1 {
            schema_version: 1,
            payloads,
        })
        .unwrap();
        (manifest, humanoid, static_glb, blob, descriptors)
    }

    #[test]
    fn m7_deferred_boundary_is_exact_core_json_and_deterministic() {
        let manifest =
            m2a_core::m7_corpus::parse_m7_corpus_manifest_v1(DEFERRED_MANIFEST.as_bytes()).unwrap();
        assert_eq!(
            validate_m7_corpus_manifest_v1_json(DEFERRED_MANIFEST),
            serialize_json(&manifest)
        );
        let core = m2a_core::m7_corpus::inspect_m7_corpus_intake_v1(&manifest, &[]).unwrap();
        let adapter =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], EMPTY_DESCRIPTORS).unwrap();
        assert_eq!(serialize_json(&adapter), serialize_json(&core));

        let first =
            build_m7_corpus_batch_v1_inner(DEFERRED_MANIFEST, &[], EMPTY_DESCRIPTORS).unwrap();
        let second =
            build_m7_corpus_batch_v1_inner(DEFERRED_MANIFEST, &[], EMPTY_DESCRIPTORS).unwrap();
        assert_eq!(first, second);
        assert!(!first.to_ascii_lowercase().contains("base64"));
        let value: serde_json::Value = serde_json::from_str(&first).unwrap();
        assert_eq!(value["report"]["packetCount"], 3);
        assert_eq!(value["packets"].as_array().unwrap().len(), 3);
        assert_eq!(value["report"]["m7DoneClaimAllowed"], false);
    }

    #[test]
    fn m7_blob_descriptors_are_strict_checked_and_exact_covering() {
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], "{").unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-DESCRIPTORS-JSON-INVALID"
        );

        let unknown = r#"{"schemaVersion":1,"payloads":[],"extra":true}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], unknown).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-DESCRIPTORS-JSON-INVALID"
        );

        let oob = r#"{"schemaVersion":1,"payloads":[{"role":"SOURCE","relativePath":"source.glb","payloadOffset":0,"payloadSize":2}]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0], oob).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-OOB"
        );

        let gap = r#"{"schemaVersion":1,"payloads":[{"role":"SOURCE","relativePath":"source.glb","payloadOffset":1,"payloadSize":1}]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], gap).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-GAP"
        );

        let overlap = r#"{"schemaVersion":1,"payloads":[
          {"role":"SOURCE","relativePath":"a.glb","payloadOffset":0,"payloadSize":2},
          {"role":"SOURCE","relativePath":"b.glb","payloadOffset":1,"payloadSize":1}
        ]}"#;
        let error =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], overlap).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-OVERLAP"
        );

        let unsupported = r#"{"schemaVersion":2,"payloads":[]}"#;
        let error =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], unsupported).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-DESCRIPTORS-SCHEMA-UNSUPPORTED"
        );

        let zero = r#"{"schemaVersion":1,"payloads":[{"role":"SOURCE","relativePath":"source.glb","payloadOffset":0,"payloadSize":0}]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], zero).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-EMPTY"
        );

        let duplicate_source = r#"{"schemaVersion":1,"payloads":[
          {"role":"SOURCE","relativePath":"Models/Source.glb","payloadOffset":0,"payloadSize":1},
          {"role":"SOURCE","relativePath":"models/source.GLB","payloadOffset":1,"payloadSize":1}
        ]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], duplicate_source)
            .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-SOURCE-DESCRIPTOR-DUPLICATE"
        );
    }

    #[test]
    fn appearance_descriptors_are_semantic_and_never_ignored() {
        let descriptor = |sample_id: &str| {
            format!(
                r#"{{"schemaVersion":1,"payloads":[{{"role":"RIGGED_HUMANOID_APPEARANCE_2DA","sampleId":"{sample_id}","payloadOffset":0,"payloadSize":1}}]}}"#
            )
        };
        for (sample_id, code) in [
            ("missing", "M7-WASM-APPEARANCE-2DA-SAMPLE-UNKNOWN"),
            ("creature", "M7-WASM-APPEARANCE-2DA-ROLE-MISMATCH"),
        ] {
            let error =
                inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0], &descriptor(sample_id))
                    .unwrap_err();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
                code
            );
        }

        let duplicate = r#"{"schemaVersion":1,"payloads":[
          {"role":"RIGGED_HUMANOID_APPEARANCE_2DA","sampleId":"humanoid","payloadOffset":0,"payloadSize":1},
          {"role":"RIGGED_HUMANOID_APPEARANCE_2DA","sampleId":"humanoid","payloadOffset":1,"payloadSize":1}
        ]}"#;
        let error =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], duplicate).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-APPEARANCE-2DA-DUPLICATE"
        );

        let intake =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0], &descriptor("humanoid"))
                .unwrap();
        assert_eq!(
            intake.status,
            m2a_core::m7_corpus::M7IntakeStatusV1::InputDeferred
        );
    }

    #[test]
    fn ready_owned_boundary_is_exact_native_batch_oracle_and_immutable() {
        let (manifest, humanoid, static_glb, blob, descriptors) = ready_boundary(true);
        let manifest_json = serialize_json(&manifest);
        let blob_before = blob.clone();
        let intake =
            inspect_m7_corpus_intake_v1_inner(&manifest_json, &blob, &descriptors).unwrap();
        assert_eq!(
            intake.status,
            m2a_core::m7_corpus::M7IntakeStatusV1::ReadyForM7V5
        );
        assert_eq!(blob, blob_before);

        let adapter = build_m7_corpus_batch_v1_inner(&manifest_json, &blob, &descriptors).unwrap();
        assert_eq!(blob, blob_before);
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
            "humanoid", &humanoid, APPEARANCE,
        )
        .unwrap()];
        let native =
            m2a_core::m7_corpus::build_m7_corpus_batch_v1(&manifest, &payloads, &canonical)
                .unwrap();
        assert_eq!(native.report.materialized_packet_count, 1);
        assert_eq!(native.report.deferred_packet_count, 2);
        let expected = serialize_json_result(&M7BatchBoundaryOutputV1 {
            schema_version: 1,
            report: native.report,
            packets: native
                .packets
                .into_iter()
                .map(|packet| packet.packet)
                .collect(),
        })
        .unwrap();
        assert_eq!(adapter, expected);
        assert_eq!(
            format!("{:x}", Sha256::digest(adapter.as_bytes())),
            READY_BATCH_JSON_SHA256
        );
        assert_eq!(
            adapter,
            build_m7_corpus_batch_v1_inner(&manifest_json, &blob, &descriptors).unwrap()
        );
    }

    #[test]
    fn global_intake_gate_prevents_mixed_readiness_materialization() {
        let (manifest, humanoid, static_glb) = ready_corpus();
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        for (path, bytes) in [
            ("models/humanoid.glb", humanoid.as_slice()),
            ("models/creature.glb", static_glb.as_slice()),
        ] {
            append_payload(
                &mut blob,
                &mut descriptors,
                |payload_offset, payload_size| M7PayloadDescriptorV1::Source {
                    relative_path: path.to_owned(),
                    payload_offset,
                    payload_size,
                },
                bytes,
            );
        }
        append_payload(
            &mut blob,
            &mut descriptors,
            |payload_offset, payload_size| M7PayloadDescriptorV1::RiggedHumanoidAppearance2da {
                sample_id: "humanoid".to_owned(),
                payload_offset,
                payload_size,
            },
            APPEARANCE,
        );
        let descriptors = serde_json::to_string(&M7PayloadDescriptorsV1 {
            schema_version: 1,
            payloads: descriptors,
        })
        .unwrap();
        let output =
            build_m7_corpus_batch_v1_inner(&serialize_json(&manifest), &blob, &descriptors)
                .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["report"]["materializedPacketCount"], 0);
        assert_eq!(value["report"]["status"], "INPUT_DEFERRED");
    }

    #[test]
    fn invalid_intake_with_valid_appearance_preserves_core_diagnostics() {
        let (manifest, _, _, mut blob, descriptors) = ready_boundary(true);
        blob[0] ^= 0xff;
        let output =
            build_m7_corpus_batch_v1_inner(&serialize_json(&manifest), &blob, &descriptors)
                .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["report"]["status"], "INPUT_INVALID");
        assert_eq!(value["report"]["materializedPacketCount"], 0);
        let humanoid = value["packets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|packet| packet["sampleId"] == "humanoid")
            .unwrap();
        assert_eq!(humanoid["status"], "INPUT_INVALID");
        assert!(
            humanoid["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["code"] == "M7-SOURCE-IDENTITY-MISMATCH")
        );
    }

    #[test]
    fn ready_humanoid_requires_exactly_one_appearance_descriptor() {
        let (manifest, _, _, blob, descriptors) = ready_boundary(false);
        let error = build_m7_corpus_batch_v1_inner(&serialize_json(&manifest), &blob, &descriptors)
            .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-APPEARANCE-2DA-CARDINALITY"
        );
    }

    #[test]
    fn public_manifest_api_reports_malformed_unknown_and_unsupported_json() {
        for (manifest_json, code) in [
            ("{", "M7-MANIFEST-JSON-INVALID"),
            (
                r#"{"schemaVersion":2,"corpusId":"x","artDirectionApprovalId":null,"samples":[]}"#,
                "M7-MANIFEST-SCHEMA-UNSUPPORTED",
            ),
            (
                r#"{"schemaVersion":1,"corpusId":"x","artDirectionApprovalId":null,"samples":[],"unknown":true}"#,
                "M7-MANIFEST-JSON-INVALID",
            ),
        ] {
            let value: serde_json::Value =
                serde_json::from_str(&validate_m7_corpus_manifest_v1_json(manifest_json)).unwrap();
            assert_eq!(value["code"], code);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod m5_native_tests {
    use super::{
        HakResourceDescriptorV1, HakResourceDescriptorsV1, append_two_da_row_artifact_json,
        append_two_da_row_v1, append_two_da_row_v1_report_json, build_m6_model_package_v1,
        build_meshy_h1_model_package_v2_inner, build_meshy_h1_model_package_v3_inner,
        build_meshy_h1_model_package_v5_inner,
        build_meshy_procedural_humanoid_model_package_v1_inner,
        build_meshy_static_placeable_package_v1_inner,
        build_meshy_static_placeable_package_v2_inner, build_meshy_static_tile_package_v1_inner,
        direct_creature_animation_catalog_v1_json, inspect_editable_animation_source_v1_inner,
        inspect_two_da_v2_json, inspect_two_da_v2_json_inner,
        materialize_animation_studio_document_v1_inner, materialize_hak_resources,
        preview_authored_animation_clip_v1_inner, resolve_creature_animation_mapping_v1,
        serialize_json, validate_animation_studio_document_v1_inner,
        validate_creature_animation_authoring_v1, write_hak_artifact_json, write_hak_v1,
        write_hak_v1_report_json, write_model_package_v1, write_model_package_v1_inner,
        write_package_manifest_v1_json, write_package_manifest_v1_json_inner,
        write_tga_artifact_json, write_tga_v1, write_tga_v1_report_json,
    };
    use m2a_core::hak::{HakResourceInputV1, HakWriterOptionsV1};
    use m2a_core::tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1};
    use m2a_core::two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
    };

    const DIRECT_CREATURE_APPEARANCE: &[u8] = b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n";

    fn canonical_repository_root() -> std::path::PathBuf {
        let crate_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repository_root = crate_directory
            .parent()
            .and_then(std::path::Path::parent)
            .expect("m2a-wasm must be nested at crates/m2a-wasm");
        let worktrees_directory = repository_root.parent();
        if worktrees_directory
            .and_then(std::path::Path::file_name)
            .is_some_and(|name| name.eq_ignore_ascii_case(".worktrees"))
        {
            return worktrees_directory
                .and_then(std::path::Path::parent)
                .expect(".worktrees must have the canonical repository parent")
                .to_path_buf();
        }
        repository_root.to_path_buf()
    }

    #[test]
    fn authored_animation_wasm_boundaries_return_structured_data() {
        let catalog: serde_json::Value =
            serde_json::from_str(&direct_creature_animation_catalog_v1_json()).unwrap();
        assert_eq!(catalog.as_array().map(Vec::len), Some(42));

        let validation: serde_json::Value =
            serde_json::from_str(&validate_creature_animation_authoring_v1("{")).unwrap();
        assert_eq!(validation["status"], "BLOCKED");
        assert_eq!(
            validation["diagnostics"][0]["code"],
            "M2A-ANIMATION-AUTHORING-JSON"
        );

        let resolution: serde_json::Value =
            serde_json::from_str(&resolve_creature_animation_mapping_v1("{")).unwrap();
        assert_eq!(resolution["ok"], false);
        assert_eq!(resolution["validation"]["status"], "BLOCKED");
    }

    fn tga_image() -> TgaImageV1 {
        TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgb8,
            pixels: vec![255, 0, 128],
        }
    }

    fn static_owned_glb() -> Vec<u8> {
        let glb = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().expect("owned GLB fixture");
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_length;
        let mut root: serde_json::Value =
            serde_json::from_slice(&glb[20..json_end]).expect("GLB JSON");
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);
        root["nodes"] = serde_json::json!([{
            "name": "placeable-source-root",
            "mesh": 0
        }]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .expect("primitive attributes");
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");

        let mut json = serde_json::to_vec(&root).expect("serialize GLB JSON");
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let mut result = Vec::new();
        result.extend_from_slice(b"glTF");
        result.extend_from_slice(&2_u32.to_le_bytes());
        result.extend_from_slice(&0_u32.to_le_bytes());
        result.extend_from_slice(&(json.len() as u32).to_le_bytes());
        result.extend_from_slice(b"JSON");
        result.extend_from_slice(&json);
        result.extend_from_slice(&glb[json_end..]);
        let total_length = result.len() as u32;
        result[8..12].copy_from_slice(&total_length.to_le_bytes());
        result
    }

    fn full_native_42_owned_glb() -> Vec<u8> {
        m2a_core::owned_fixture::synthetic_owned_m6_full_native_42_glb_v1()
            .expect("owned full-native-42 GLB fixture")
    }

    fn full_native_42_authoring_v1(
        source: &[u8],
    ) -> m2a_core::creature_animation_mapping::CreatureAnimationAuthoringV1 {
        use m2a_core::creature_animation_mapping::{
            AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
            AnimationSourceAssignmentV1, AnimationSourceKindV1,
            CREATURE_ANIMATION_AUTHORING_PROFILE_V1, DirectCreatureBaseSlotV1,
            DirectCreatureModelTypeV1,
        };
        use sha2::{Digest, Sha256};

        m2a_core::creature_animation_mapping::CreatureAnimationAuthoringV1 {
            schema_version: 1,
            profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
            model_type: DirectCreatureModelTypeV1::Simple,
            source_revision: format!("{:x}", Sha256::digest(source)),
            authoring_revision: 1,
            assignments: m2a_core::direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
                .iter()
                .map(|slot| AnimationSourceAssignmentV1 {
                    target_slot: DirectCreatureBaseSlotV1::try_from(*slot).unwrap(),
                    source_kind: AnimationSourceKindV1::SourceClip,
                    source_clip_name: Some((*slot).to_owned()),
                    custom_animation_id: None,
                    provenance: AnimationMappingProvenanceV1 {
                        provider: AnimationProviderV1::SourceGlb,
                        asset_id: "owned-full-42-fixture".to_owned(),
                        ownership: AnimationOwnershipV1::UserOwned,
                    },
                })
                .collect(),
            fallbacks: vec![],
            custom_animations: vec![],
        }
    }

    fn common_native_event_authoring() -> m2a_core::model_pipeline::DirectCreatureEventAuthoringV1 {
        use std::collections::BTreeMap;

        let mut clips = BTreeMap::<String, Vec<m2a_core::mdl::MdlAnimationEventV1>>::new();
        for (clip_name, event_name) in
            m2a_core::model_pipeline::COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1
        {
            clips.entry((*clip_name).to_owned()).or_default().push(
                m2a_core::mdl::MdlAnimationEventV1 {
                    time_seconds: 0.5,
                    name: (*event_name).to_owned(),
                },
            );
        }
        m2a_core::model_pipeline::DirectCreatureEventAuthoringV1 {
            schema_version: 1,
            clips: clips
                .into_iter()
                .map(|(clip_name, events)| {
                    m2a_core::model_pipeline::DirectCreatureClipEventAuthoringV1 {
                        clip_name,
                        events,
                    }
                })
                .collect(),
        }
    }

    fn placeables_two_da() -> Vec<u8> {
        b"2DA V2.0\n\nLabel StrRef ModelName LightColor LightOffsetX LightOffsetY LightOffsetZ SoundAppType ShadowSize BodyBag LowGore Reflection Static\n0 EXISTING **** plc_a01 **** **** **** **** **** 1 0 **** **** 1\n".to_vec()
    }

    fn placeable_identity() -> m2a_core::placeable::StaticPlaceableIdentityV1 {
        m2a_core::placeable::StaticPlaceableIdentityV1 {
            module_resref: "m2a_plc_mod".to_owned(),
            module_file_name: "m2a_plc_mod.mod".to_owned(),
            module_display_name: "Meshy2Aurora placeable proof".to_owned(),
            area_resref: "m2a_plc_area".to_owned(),
            area_name: "Meshy2Aurora placeable area".to_owned(),
            hak_resref: "m2a_plc_hak".to_owned(),
            hak_file_name: "m2a_plc_hak.hak".to_owned(),
            model_resref: "m2a_plc_ped".to_owned(),
            texture_resref: "m2a_plc_tex".to_owned(),
            blueprint_resref: "m2a_plc_utp".to_owned(),
            object_tag: "m2a_plc_pedestal".to_owned(),
            display_name: "Meshy Ritual Pedestal".to_owned(),
        }
    }

    #[test]
    fn placeable_boundary_rejects_non_schema_identity_before_materialization() {
        let error = match build_meshy_static_placeable_package_v1_inner(
            &[],
            &[],
            r#"{"schemaVersion":1,"unknown":true}"#,
            r#"{"x":10.0,"y":14.5,"z":0.0,"bearing":0.0}"#,
            7,
        ) {
            Ok(_) => panic!("strict identity JSON must fail"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("JSON error");
        assert_eq!(value["code"], "PLACEABLE-IDENTITY-JSON-INVALID");
        assert_eq!(value["path"], "identityJson");
    }

    #[test]
    fn placeable_boundary_is_byte_identical_to_the_core_package() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v1(
            &source, &table, &identity, placement, 7,
        )
        .expect("core placeable");
        let mut boundary = build_meshy_static_placeable_package_v1_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
        )
        .expect("WASM boundary placeable");

        assert_eq!(boundary.take_hak_bytes(), core.hak_payload);
        assert_eq!(boundary.take_proof_module_bytes(), core.module_payload);
        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(boundary.take_hak_bytes().is_empty());
        assert!(boundary.take_proof_module_bytes().is_empty());
        assert!(!boundary.take_model_bytes().is_empty());
        assert!(boundary.take_model_bytes().is_empty());
    }

    #[test]
    fn authored_placeable_boundary_is_byte_identical_to_the_core_package() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let bootstrap = m2a_core::placeable::inspect_meshy_static_placeable_authoring_v1(&source)
            .expect("authoring bootstrap");
        let mut document = bootstrap.document;
        document.elements[0].transform.scale = [2.0, 2.0, 2.0];
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v2(
            &source, &table, &identity, placement, 7, &document,
        )
        .expect("authored core placeable");
        let mut boundary = build_meshy_static_placeable_package_v2_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
            &serde_json::to_string(&document).unwrap(),
        )
        .expect("authored WASM boundary placeable");

        assert_eq!(boundary.take_hak_bytes(), core.hak_payload);
        assert_eq!(boundary.take_proof_module_bytes(), core.module_payload);
        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(core.report.authoring.is_some());
    }

    #[test]
    fn authored_placeable_boundary_rejects_non_schema_document() {
        let error = match build_meshy_static_placeable_package_v2_inner(
            &[],
            &[],
            &serde_json::to_string(&placeable_identity()).unwrap(),
            r#"{"x":10.0,"y":14.5,"z":0.0,"bearing":0.0}"#,
            7,
            r#"{"schemaVersion":1,"unknown":true}"#,
        ) {
            Ok(_) => panic!("strict authoring JSON must fail"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("JSON error");
        assert_eq!(value["code"], "PLACEABLE-AUTHORING-JSON-INVALID");
        assert_eq!(value["path"], "authoringJson");
    }

    #[test]
    fn tile_boundary_is_byte_identical_to_the_common_core_pipeline() {
        let source = static_owned_glb();
        let identity = m2a_core::tile::StaticTileIdentityV1::owner_candidate_v1();
        let core = m2a_core::tile::build_meshy_static_tile_package_v1(
            &source,
            &identity,
            false,
            "Grass",
            m2a_core::walkmesh::TileSurfaceV1::Grass,
        )
        .expect("core tile");
        let options = serde_json::json!({
            "schemaVersion": 1,
            "identity": identity,
            "interior": false,
            "terrainName": "Grass",
            "surface": "GRASS"
        })
        .to_string();
        let mut boundary =
            build_meshy_static_tile_package_v1_inner(&source, &options).expect("WASM tile");

        assert_eq!(boundary.take_hak_bytes(), core.hak_payload);
        assert_eq!(boundary.take_module_bytes(), core.module_payload);
        assert_eq!(boundary.take_model_bytes(), core.mdl_payload);
        assert_eq!(boundary.take_wok_bytes(), core.wok_payload);
        assert_eq!(boundary.take_set_bytes(), core.set_payload);
        assert_eq!(boundary.take_texture_bytes(), core.texture_payload);
        assert_eq!(boundary.take_image_map_bytes(), core.image_map_payload);
        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(boundary.take_hak_bytes().is_empty());
        assert!(boundary.take_module_bytes().is_empty());
        assert!(boundary.take_model_bytes().is_empty());
        assert!(boundary.take_wok_bytes().is_empty());
        assert!(boundary.take_set_bytes().is_empty());
    }

    #[test]
    fn tile_boundary_rejects_unknown_options_before_reading_source() {
        let error = match build_meshy_static_tile_package_v1_inner(
            &[],
            r#"{"schemaVersion":1,"identity":{},"interior":false,"terrainName":"Grass","surface":"Grass","unknown":true}"#,
        ) {
            Ok(_) => panic!("strict tile options must fail"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("JSON error");
        assert_eq!(value["code"], "TILE-OPTIONS-JSON-INVALID");
        assert_eq!(value["path"], "optionsJson");
    }

    fn two_da_request() -> TwoDaAppendRequestV1 {
        TwoDaAppendRequestV1 {
            schema_version: 1,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "new".to_owned(),
                },
            }],
        }
    }

    fn hak_boundary(
        entries: &[(&str, u16, &[u8])],
    ) -> (Vec<u8>, HakResourceDescriptorsV1, Vec<HakResourceInputV1>) {
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        let mut resources = Vec::new();
        for &(resref, resource_type, payload) in entries {
            let payload_offset = blob.len() as u32;
            blob.extend_from_slice(payload);
            descriptors.push(HakResourceDescriptorV1 {
                resref: resref.to_owned(),
                resource_type,
                payload_offset,
                payload_size: payload.len() as u32,
            });
            resources.push(HakResourceInputV1 {
                resref: resref.to_owned(),
                resource_type,
                payload: payload.to_vec(),
            });
        }
        (
            blob,
            HakResourceDescriptorsV1 {
                schema_version: 1,
                resources: descriptors,
            },
            resources,
        )
    }

    fn package_entries() -> [(&'static str, u16, &'static [u8]); 3] {
        [
            ("texture", 3, b"tga"),
            ("appearance", 2017, b"2da"),
            ("model", 2002, b"mdl"),
        ]
    }

    #[test]
    fn tga_public_bytes_and_report_are_exact_core_and_inputs_are_immutable() {
        let image = tga_image();
        let options = TgaWriterOptionsV1::default();
        let image_json = serde_json::to_string(&image).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (image_json.clone(), options_json.clone());
        let core = m2a_core::tga::write_tga_v1(&image, &options).unwrap();

        assert_eq!(
            write_tga_v1(&image_json, &options_json).unwrap(),
            core.payload
        );
        assert_eq!(
            write_tga_v1_report_json(&image_json, &options_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((image_json, options_json), before);
    }

    #[test]
    fn two_da_public_inspect_append_and_report_are_exact_core_and_immutable() {
        let source = b"2DA V2.0\n\nA B\n0 old ****\n".to_vec();
        let request = two_da_request();
        let limits = TwoDaLimitsV1::default();
        let request_json = serde_json::to_string(&request).unwrap();
        let limits_json = serde_json::to_string(&limits).unwrap();
        let before = (source.clone(), request_json.clone(), limits_json.clone());

        let inspection = m2a_core::two_da::inspect_two_da_v2(&source, &limits).unwrap();
        assert_eq!(
            inspect_two_da_v2_json(&source, &limits_json).unwrap(),
            serde_json::to_string(&inspection).unwrap()
        );
        let core = m2a_core::two_da::append_two_da_row_v1(&source, &request, &limits).unwrap();
        assert_eq!(
            append_two_da_row_v1(&source, &request_json, &limits_json).unwrap(),
            core.payload
        );
        assert_eq!(
            append_two_da_row_v1_report_json(&source, &request_json, &limits_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((source, request_json, limits_json), before);
    }

    #[test]
    fn boundary_json_errors_are_frozen_strict_and_follow_argument_precedence() {
        let options_json = serde_json::to_string(&TgaWriterOptionsV1::default()).unwrap();
        let image_error = write_tga_artifact_json("{", &options_json).unwrap_err();
        assert_eq!(
            image_error,
            r#"{"schemaVersion":1,"code":"M5-TGA-IMAGE-JSON-INVALID","severity":"FATAL","path":"imageJson","message":"image JSON does not match the strict public schema"}"#
        );
        let mut image = serde_json::to_value(tga_image()).unwrap();
        image["unknown"] = serde_json::json!(true);
        assert_eq!(
            write_tga_artifact_json(&image.to_string(), &options_json).unwrap_err(),
            image_error
        );
        let options_error =
            write_tga_artifact_json(&serialize_json(&tga_image()), "{").unwrap_err();
        assert_eq!(
            options_error,
            r#"{"schemaVersion":1,"code":"M5-TGA-OPTIONS-JSON-INVALID","severity":"FATAL","path":"optionsJson","message":"TGA options JSON does not match the strict public schema"}"#
        );
        for path in ["unknown", "limits.unknown"] {
            let mut options = serde_json::to_value(TgaWriterOptionsV1::default()).unwrap();
            if path == "unknown" {
                options["unknown"] = serde_json::json!(true);
            } else {
                options["limits"]["unknown"] = serde_json::json!(true);
            }
            assert_eq!(
                write_tga_artifact_json(&serialize_json(&tga_image()), &options.to_string())
                    .unwrap_err(),
                options_error,
                "{path}"
            );
        }

        let source = b"2DA V2.0\n\nA\n";
        let request_error = append_two_da_row_artifact_json(source, "{", "{").unwrap_err();
        assert_eq!(
            request_error,
            r#"{"schemaVersion":1,"code":"M5-2DA-REQUEST-JSON-INVALID","severity":"FATAL","path":"requestJson","message":"2DA append request JSON does not match the strict public schema"}"#
        );
        let limits_error = inspect_two_da_v2_json_inner(source, "{").unwrap_err();
        assert_eq!(
            limits_error,
            r#"{"schemaVersion":1,"code":"M5-2DA-LIMITS-JSON-INVALID","severity":"FATAL","path":"limitsJson","message":"2DA limits JSON does not match the strict public schema"}"#
        );
        let mut limits = serde_json::to_value(TwoDaLimitsV1::default()).unwrap();
        limits["unknown"] = serde_json::json!(true);
        assert_eq!(
            inspect_two_da_v2_json_inner(source, &limits.to_string()).unwrap_err(),
            limits_error
        );
        for path in ["request", "cell", "value"] {
            let mut request = serde_json::to_value(two_da_request()).unwrap();
            match path {
                "request" => request["unknown"] = serde_json::json!(true),
                "cell" => request["cells"][0]["unknown"] = serde_json::json!(true),
                "value" => request["cells"][0]["value"]["unknown"] = serde_json::json!(true),
                _ => unreachable!(),
            }
            assert_eq!(
                append_two_da_row_artifact_json(
                    source,
                    &request.to_string(),
                    &serde_json::to_string(&TwoDaLimitsV1::default()).unwrap()
                )
                .unwrap_err(),
                request_error,
                "{path}"
            );
        }
    }

    #[test]
    fn core_errors_remain_exact_json_and_boundary_never_uses_base64() {
        let invalid_image = TgaImageV1 {
            width: 0,
            ..tga_image()
        };
        let options = TgaWriterOptionsV1::default();
        let direct = m2a_core::tga::write_tga_v1(&invalid_image, &options).unwrap_err();
        assert_eq!(
            write_tga_artifact_json(
                &serde_json::to_string(&invalid_image).unwrap(),
                &serde_json::to_string(&options).unwrap()
            )
            .unwrap_err(),
            serde_json::to_string(&direct).unwrap()
        );

        let limits = TwoDaLimitsV1::default();
        let direct = m2a_core::two_da::inspect_two_da_v2(b"bad\n", &limits).unwrap_err();
        let boundary =
            inspect_two_da_v2_json_inner(b"bad\n", &serde_json::to_string(&limits).unwrap())
                .unwrap_err();
        assert_eq!(boundary, super::two_da_core_error_json(&direct));
        assert_eq!(
            boundary,
            r#"{"schemaVersion":1,"code":"M5-2DA-HEADER-INVALID","severity":"FATAL","path":"header","message":"line 1 must be exactly 2DA V2.0"}"#
        );
        assert!(!boundary.contains("byteOffset"));

        let image_json = serde_json::to_string(&tga_image()).unwrap();
        let report =
            write_tga_v1_report_json(&image_json, &serde_json::to_string(&options).unwrap())
                .unwrap();
        assert!(!image_json.to_ascii_lowercase().contains("base64"));
        assert!(!report.to_ascii_lowercase().contains("base64"));
    }

    #[test]
    fn hak_and_package_match_core_are_deterministic_immutable_and_frozen() {
        let base_entries = package_entries();
        let (blob, descriptors, resources) = hak_boundary(&base_entries);
        let options = HakWriterOptionsV1::default();
        let resources_json = serde_json::to_string(&descriptors).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (blob.clone(), resources_json.clone(), options_json.clone());
        let core_hak = m2a_core::hak::write_hak_v1(&resources, &options).unwrap();
        let core_manifest =
            m2a_core::package::write_package_manifest_v1(&resources, &options).unwrap();

        assert_eq!(
            write_hak_v1(&blob, &resources_json, &options_json).unwrap(),
            core_hak.payload
        );
        assert_eq!(
            write_hak_v1_report_json(&blob, &resources_json, &options_json).unwrap(),
            serde_json::to_string(&core_hak.report).unwrap()
        );
        let manifest =
            write_package_manifest_v1_json(&blob, &resources_json, &options_json).unwrap();
        assert_eq!(manifest, serde_json::to_string(&core_manifest).unwrap());
        assert_eq!(
            manifest,
            "{\"schemaVersion\":1,\"packageSha256\":\"494862f6a12f91d5a269519d0579a05ace5bb50fd8f72b5711fcae7445444477\",\"resources\":[{\"role\":\"APPEARANCE_TABLE\",\"resref\":\"appearance\",\"type\":2017,\"byteLength\":3,\"sha256\":\"ddf81e9e4f364c6f086fd730b8f6d2bc4b46068045a085e1be8fc7470a615c6f\",\"hakResourceId\":0,\"hakPayloadOffset\":256},{\"role\":\"MODEL\",\"resref\":\"model\",\"type\":2002,\"byteLength\":3,\"sha256\":\"d3c3c54797643905c5cc97f7da4717058dbe6ad183ef1586104cadd197ca47c6\",\"hakResourceId\":1,\"hakPayloadOffset\":259},{\"role\":\"TEXTURE\",\"resref\":\"texture\",\"type\":3,\"byteLength\":3,\"sha256\":\"9dedca90fc9c44caeb39e0a6b8d28a157105bfba113872846ce0b2f5eff923d3\",\"hakResourceId\":2,\"hakPayloadOffset\":262}]}"
        );
        assert_eq!((blob, resources_json, options_json.clone()), before);
        assert!(!manifest.to_ascii_lowercase().contains("base64"));

        let expected = (core_hak, manifest);
        for order in [[2, 0, 1], [1, 2, 0]] {
            let entries = order.map(|index| base_entries[index]);
            let (blob, descriptors, _) = hak_boundary(&entries);
            let resources_json = serde_json::to_string(&descriptors).unwrap();
            assert_eq!(
                write_hak_artifact_json(&blob, &resources_json, &options_json).unwrap(),
                expected.0
            );
            assert_eq!(
                write_package_manifest_v1_json_inner(&blob, &resources_json, &options_json)
                    .unwrap(),
                expected.1
            );
        }
    }

    #[test]
    fn model_package_adapter_returns_one_core_artifact_with_native_parity() {
        let base_entries = package_entries();
        let (blob, descriptors, resources) = hak_boundary(&base_entries);
        let options = HakWriterOptionsV1::default();
        let resources_json = serde_json::to_string(&descriptors).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (blob.clone(), resources_json.clone(), options_json.clone());
        let core = m2a_core::package::write_model_package_v1(&resources, &options).unwrap();

        let mut wasm = write_model_package_v1_inner(&blob, &resources_json, &options_json).unwrap();
        assert_eq!(
            wasm.report_json(),
            serde_json::to_string(&core.hak.report).unwrap()
        );
        assert_eq!(
            wasm.manifest_json(),
            serde_json::to_string(&core.manifest).unwrap()
        );
        assert_eq!((blob, resources_json, options_json), before);
        assert!(!wasm.report_json().to_ascii_lowercase().contains("base64"));
        assert!(!wasm.manifest_json().to_ascii_lowercase().contains("base64"));
        assert_eq!(wasm.take_hak_bytes(), core.hak.payload);
        assert!(wasm.take_hak_bytes().is_empty());

        for order in [[2, 0, 1], [1, 2, 0]] {
            let entries = order.map(|index| base_entries[index]);
            let (blob, descriptors, _) = hak_boundary(&entries);
            let resources_json = serde_json::to_string(&descriptors).unwrap();
            let mut candidate =
                write_model_package_v1_inner(&blob, &resources_json, &before.2).unwrap();
            assert_eq!(candidate.manifest_json(), wasm.manifest_json());
            assert_eq!(candidate.take_hak_bytes(), core.hak.payload);
            assert!(candidate.take_hak_bytes().is_empty());
        }

        let mut public = write_model_package_v1(&before.0, &before.1, &before.2).unwrap();
        assert_eq!(public.manifest_json(), wasm.manifest_json());
        assert_eq!(public.take_hak_bytes(), core.hak.payload);
        assert!(public.take_hak_bytes().is_empty());
    }

    #[test]
    fn studio_model_package_adapter_is_exact_core_and_transfers_each_binary_once() {
        let source = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let core =
            m2a_core::model_pipeline::build_m6_model_package_v1(&source, appearance).unwrap();
        let mut studio = build_m6_model_package_v1(&source, appearance).unwrap();

        assert_eq!(
            studio.report_json(),
            String::from_utf8(core.report_json.clone()).unwrap()
        );
        assert_eq!(
            studio.manifest_json(),
            String::from_utf8(core.manifest_json.clone()).unwrap()
        );
        assert_eq!(
            studio.summary_json(),
            String::from_utf8(core.summary_json.clone()).unwrap()
        );
        assert_eq!(
            studio.readback_json(),
            serialize_json(&m2a_core::inspect_binary_mdl(&core.model).unwrap())
        );
        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert!(studio.take_hak_bytes().is_empty());
        assert_eq!(studio.take_model_bytes(), core.model);
        assert!(studio.take_model_bytes().is_empty());
    }

    #[test]
    fn studio_full_native_h1_v2_is_exact_core_and_fails_closed_without_42_source_clips() {
        let source = full_native_42_owned_glb();
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v2(
            &source,
            appearance,
            m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        )
        .expect("core full H1");
        let mut studio =
            build_meshy_h1_model_package_v2_inner(&source, appearance).expect("Studio full H1");
        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        assert_eq!(
            studio.readback_json(),
            serialize_json(&m2a_core::inspect_binary_mdl(&core.model).unwrap())
        );

        let idle_only = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let error = match build_meshy_h1_model_package_v2_inner(&idle_only, appearance) {
            Ok(_) => panic!("full Studio path must reject one idle source"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("structured error");
        assert_eq!(value["code"], "M6-ANIMATION-FULL-PROFILE-MISSING");
    }

    #[test]
    fn studio_h2_procedural_lane_materializes_the_owned_single_idle_source_as_full_42() {
        let source = std::fs::read(
            canonical_repository_root().join("sample-3d/h2-clockwork-sentinel-1500/source.glb"),
        )
        .expect("owned H2 GLB");
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v2(
            &source,
            appearance,
            m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
        )
        .expect("core procedural H2");
        let mut studio =
            build_meshy_procedural_humanoid_model_package_v1_inner(&source, appearance)
                .expect("Studio procedural H2");

        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        let report: serde_json::Value =
            serde_json::from_str(&studio.report_json()).expect("report JSON");
        assert_eq!(
            report["animationCompleteness"]["profile"],
            "FULL_NATIVE42_PROCEDURAL_HUMANOID_V1"
        );
        assert_eq!(report["animationCompleteness"]["requiredClipCount"], 42);
        assert_eq!(report["animationCompleteness"]["explicitClipCount"], 1);
        assert_eq!(report["animationCompleteness"]["proceduralClipCount"], 41);
        assert_eq!(report["animationCompleteness"]["fallbackAliasCount"], 0);
        assert_eq!(
            report["animationBehavior"]["behaviorCandidateEligible"],
            true
        );
        assert_eq!(report["animationEventConformance"]["complete"], true);
        assert_eq!(report["skinAnimationConformance"]["complete"], true);
    }

    #[test]
    fn studio_full_native_h1_v3_is_exact_core_and_rejects_invalid_or_incomplete_events() {
        let source = full_native_42_owned_glb();
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let event_authoring = common_native_event_authoring();
        let event_authoring_json =
            serde_json::to_string(&event_authoring).expect("event authoring JSON");
        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v3(
            &source,
            appearance,
            m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
            m2a_core::model_pipeline::DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
            &event_authoring,
        )
        .expect("core eventful full H1");
        let mut studio =
            build_meshy_h1_model_package_v3_inner(&source, appearance, &event_authoring_json)
                .expect("Studio eventful full H1");
        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        assert_eq!(
            studio.readback_json(),
            serialize_json(&m2a_core::inspect_binary_mdl(&core.model).unwrap())
        );
        let report: serde_json::Value =
            serde_json::from_slice(&core.report_json).expect("Core report");
        assert_eq!(report["animationEventConformance"]["requiredPairCount"], 23);
        assert_eq!(
            report["animationEventConformance"]["satisfiedPairCount"],
            23
        );
        assert_eq!(report["animationEventConformance"]["complete"], true);

        let malformed = match build_meshy_h1_model_package_v3_inner(&source, appearance, "{") {
            Ok(_) => panic!("malformed event authoring JSON must fail"),
            Err(error) => error,
        };
        let malformed: serde_json::Value =
            serde_json::from_str(&malformed).expect("structured JSON error");
        assert_eq!(malformed["code"], "M6-ANIMATION-EVENT-AUTHORING-JSON");
        assert_eq!(malformed["path"], "eventAuthoringJson");
        let unknown = match build_meshy_h1_model_package_v3_inner(
            &source,
            appearance,
            r#"{"schemaVersion":1,"clips":[],"unknown":true}"#,
        ) {
            Ok(_) => panic!("unknown event authoring JSON fields must fail"),
            Err(error) => error,
        };
        let unknown: serde_json::Value =
            serde_json::from_str(&unknown).expect("structured unknown-field error");
        assert_eq!(unknown["code"], "M6-ANIMATION-EVENT-AUTHORING-JSON");

        let mut incomplete = event_authoring;
        incomplete
            .clips
            .iter_mut()
            .find(|clip| clip.clip_name == "ccastout")
            .expect("ccastout authoring")
            .events
            .retain(|event| event.name != "cast");
        let error = match build_meshy_h1_model_package_v3_inner(
            &source,
            appearance,
            &serde_json::to_string(&incomplete).unwrap(),
        ) {
            Ok(_) => panic!("incomplete event authoring must fail"),
            Err(error) => error,
        };
        let error: serde_json::Value = serde_json::from_str(&error).expect("structured error");
        assert_eq!(error["code"], "M6-ANIMATION-EVENTS-INELIGIBLE");
        assert!(error["message"].as_str().unwrap().contains("ccastout:cast"));
    }

    #[test]
    fn animation_studio_wasm_boundaries_are_strict_and_v5_is_exact_core() {
        let source = full_native_42_owned_glb();
        let inspection_json = inspect_editable_animation_source_v1_inner(&source, Some("cpause1"))
            .expect("editable source inspection");
        let inspection: serde_json::Value =
            serde_json::from_str(&inspection_json).expect("inspection JSON");
        assert_eq!(inspection["schemaVersion"], 1);
        assert!(
            inspection["rig"]
                .as_array()
                .is_some_and(|rig| !rig.is_empty())
        );
        assert!(inspection["rig"].as_array().unwrap().iter().all(|node| {
            node["translation"].as_array().map(Vec::len) == Some(3)
                && node["rotation"].as_array().map(Vec::len) == Some(4)
        }));
        assert_eq!(inspection["clip"]["source"]["sourceClipName"], "cpause1");

        let v1 = full_native_42_authoring_v1(&source);
        let v2 = m2a_core::animation_studio::migrate_creature_animation_authoring_v1_to_v2(&v1);
        let studio = m2a_core::animation_studio::AnimationStudioDocumentV1 {
            schema_version: 1,
            source_revision: v1.source_revision.clone(),
            authoring_revision: 1,
            status: m2a_core::animation_studio::AnimationStudioDocumentStatusV1::Valid,
            authored_clips: vec![],
        };
        let studio_json = serde_json::to_string(&studio).unwrap();
        let validation: serde_json::Value = serde_json::from_str(
            &validate_animation_studio_document_v1_inner(&studio_json, &source),
        )
        .unwrap();
        assert_eq!(validation["status"], "READY");
        let materialized: serde_json::Value = serde_json::from_str(
            &materialize_animation_studio_document_v1_inner(&studio_json, &source)
                .expect("empty Studio materialization"),
        )
        .unwrap();
        assert_eq!(materialized["clips"].as_array().map(Vec::len), Some(0));

        let preview_clip: m2a_core::animation_studio::AuthoredAnimationClipV1 =
            serde_json::from_value(inspection["clip"].clone()).unwrap();
        let preview_clip_id = preview_clip.id.clone();
        let mut preview_document = studio.clone();
        preview_document.status =
            m2a_core::animation_studio::AnimationStudioDocumentStatusV1::Draft;
        preview_document.authored_clips = vec![preview_clip];
        let preview = preview_authored_animation_clip_v1_inner(
            &serde_json::to_string(&preview_document).unwrap(),
            &preview_clip_id,
        )
        .expect("canonical authored preview");
        let preview: serde_json::Value = serde_json::from_str(&preview).unwrap();
        assert_eq!(preview["clip"]["name"], "cpause1");

        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v5(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            &v2,
            &studio,
        )
        .expect("core V5");
        let mut boundary = build_meshy_h1_model_package_v5_inner(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            &serde_json::to_string(&v2).unwrap(),
            &studio_json,
            None,
        )
        .expect("WASM V5 boundary");
        assert_eq!(boundary.take_hak_bytes(), core.hak);
        assert_eq!(boundary.take_model_bytes(), core.model);
        assert_eq!(boundary.take_proof_module_bytes(), core.proof_module);
        assert_eq!(
            boundary.manifest_json(),
            String::from_utf8(core.manifest_json).unwrap()
        );

        let malformed: serde_json::Value =
            serde_json::from_str(&validate_animation_studio_document_v1_inner(
                r#"{"schemaVersion":1,"unknown":true}"#,
                &source,
            ))
            .unwrap();
        assert_eq!(malformed["status"], "BLOCKED");
        assert_eq!(
            malformed["diagnostics"][0]["code"],
            "M2A-ANIMATION-EDIT-SCHEMA"
        );
    }

    #[test]
    fn hak_boundary_precedence_ranges_zero_size_and_no_panic_are_stable() {
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let options_json = serde_json::to_string(&HakWriterOptionsV1::default()).unwrap();
        assert_eq!(
            write_hak_artifact_json(&[], "{", "{").unwrap_err(),
            r#"{"schemaVersion":1,"code":"M5-HAK-RESOURCES-JSON-INVALID","severity":"FATAL","path":"resourcesJson","message":"HAK resources JSON does not match the strict public schema"}"#
        );
        let (_, valid, _) = hak_boundary(&[("a", 1, b"x")]);
        let valid_json = serde_json::to_string(&valid).unwrap();
        assert_eq!(
            write_hak_artifact_json(&[], &valid_json, "{").unwrap_err(),
            r#"{"schemaVersion":1,"code":"M5-HAK-OPTIONS-JSON-INVALID","severity":"FATAL","path":"optionsJson","message":"HAK options JSON does not match the strict public schema"}"#
        );
        for nested in [false, true] {
            let mut options = serde_json::to_value(HakWriterOptionsV1::default()).unwrap();
            if nested {
                options["limits"]["unknown"] = serde_json::json!(true);
            } else {
                options["unknown"] = serde_json::json!(true);
            }
            let error =
                write_hak_artifact_json(&[], &valid_json, &options.to_string()).unwrap_err();
            assert!(error.contains("M5-HAK-OPTIONS-JSON-INVALID"));
        }
        for nested in [false, true] {
            let mut resources = serde_json::to_value(&valid).unwrap();
            if nested {
                resources["resources"][0]["unknown"] = serde_json::json!(true);
            } else {
                resources["unknown"] = serde_json::json!(true);
            }
            let error =
                write_hak_artifact_json(&[], &resources.to_string(), &options_json).unwrap_err();
            assert!(error.contains("M5-HAK-RESOURCES-JSON-INVALID"));
        }

        let cases = [
            (vec![1], vec![("a", 1, 2, 0)], "resources[0].payloadOffset"),
            (vec![1], vec![("a", 1, 0, 2)], "resources[0].payloadSize"),
            (
                vec![1, 2],
                vec![("a", 1, 0, 2), ("b", 1, 1, 1)],
                "payloadBlob",
            ),
            (vec![1, 2], vec![("a", 1, 1, 1)], "payloadBlob"),
            (vec![1, 2], vec![("a", 1, 0, 1)], "payloadBlob"),
        ];
        for (blob, raw, expected_path) in cases {
            let descriptors = HakResourceDescriptorsV1 {
                schema_version: 1,
                resources: raw
                    .into_iter()
                    .map(|(resref, resource_type, payload_offset, payload_size)| {
                        HakResourceDescriptorV1 {
                            resref: resref.to_owned(),
                            resource_type,
                            payload_offset,
                            payload_size,
                        }
                    })
                    .collect(),
            };
            let resources_json = serde_json::to_string(&descriptors).unwrap();
            let result = catch_unwind(AssertUnwindSafe(|| {
                write_hak_artifact_json(&blob, &resources_json, &options_json)
            }));
            let error = result
                .expect("range validation must not panic")
                .unwrap_err();
            let value: serde_json::Value = serde_json::from_str(&error).unwrap();
            assert_eq!(value["code"], "M5-HAK-PAYLOAD-RANGE-INVALID");
            assert_eq!(value["path"], expected_path);
        }

        let empty = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![HakResourceDescriptorV1 {
                resref: "empty".to_owned(),
                resource_type: 1,
                payload_offset: 0,
                payload_size: 0,
            }],
        };
        assert!(
            materialize_hak_resources(&[], &empty).unwrap()[0]
                .payload
                .is_empty()
        );
    }

    #[test]
    fn hak_range_precedes_borrowed_core_preflight_and_both_precede_payload_copy() {
        let descriptor = |resref: &str, payload_size: u32| HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![HakResourceDescriptorV1 {
                resref: resref.to_owned(),
                resource_type: 3,
                payload_offset: 99,
                payload_size,
            }],
        };

        let invalid_resref = serde_json::to_string(&descriptor("BAD", 1)).unwrap();
        let default_options = serde_json::to_string(&HakWriterOptionsV1::default()).unwrap();
        let error = write_hak_artifact_json(&[], &invalid_resref, &default_options).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let valid_descriptor = serde_json::to_string(&descriptor("texture", 1)).unwrap();
        let invalid_options = serde_json::to_string(&HakWriterOptionsV1 {
            schema_version: 2,
            ..HakWriterOptionsV1::default()
        })
        .unwrap();
        let error = write_hak_artifact_json(&[], &valid_descriptor, &invalid_options).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let entry_limited = serde_json::to_string(&HakWriterOptionsV1 {
            schema_version: 1,
            limits: m2a_core::hak::HakWriterLimitsV1 {
                max_entry_count: 0,
                max_output_bytes: m2a_core::hak::HAK_MAX_OUTPUT_BYTES,
            },
        })
        .unwrap();
        let error = write_hak_artifact_json(&[], &valid_descriptor, &entry_limited).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let output_limited = serde_json::to_string(&HakWriterOptionsV1 {
            schema_version: 1,
            limits: m2a_core::hak::HakWriterLimitsV1 {
                max_entry_count: 1,
                max_output_bytes: 160,
            },
        })
        .unwrap();
        let error = write_hak_artifact_json(&[], &valid_descriptor, &output_limited).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let duplicate = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![
                HakResourceDescriptorV1 {
                    resref: "same".to_owned(),
                    resource_type: 3,
                    payload_offset: 99,
                    payload_size: 1,
                },
                HakResourceDescriptorV1 {
                    resref: "same".to_owned(),
                    resource_type: 3,
                    payload_offset: 100,
                    payload_size: 1,
                },
            ],
        };
        let error = write_hak_artifact_json(
            &[],
            &serde_json::to_string(&duplicate).unwrap(),
            &default_options,
        )
        .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let range_error =
            write_hak_artifact_json(&[], &valid_descriptor, &default_options).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&range_error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let valid_range_invalid_resref = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![HakResourceDescriptorV1 {
                resref: "BAD".to_owned(),
                resource_type: 3,
                payload_offset: 0,
                payload_size: 1,
            }],
        };
        let error = write_hak_artifact_json(
            &[0],
            &serde_json::to_string(&valid_range_invalid_resref).unwrap(),
            &default_options,
        )
        .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-RESREF-INVALID"
        );
    }
}

#[cfg(test)]
#[path = "../../m2a-core/tests/fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod profile_a_animation_fixtures;

#[cfg(test)]
mod profile_a_test_support {
    use m2a_core::profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        ProfileAAnimationClipMappingV1, ProfileAAnimationMappingV1, ProfileAAnimationNodeMappingV1,
        ProfileAOptionsV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_profile_sha256,
    };
    use sha2::{Digest, Sha256};

    pub const RIGID_JSON_SHA256: &str =
        "d62b2444df8005b6bef0affb7f753767488ad33568096a47523cabbe5edefa06";
    pub const RIGID_JSON_LENGTH: usize = 3187;
    pub const SKIN_JSON_SHA256: &str =
        "273baf9dba1de9ac16dc499356045eb7d9ca7cd394a8dab89e8f85fc00462131";
    pub const SKIN_JSON_LENGTH: usize = 3563;
    pub const LIMIT_FATAL_JSON_SHA256: &str =
        "3bfb45cf36af0d4af174cea656ab669714a55c4c88a9661c7ea573be75bec4a2";
    pub const LIMIT_FATAL_JSON_LENGTH: usize = 162;
    pub const MALFORMED_JSON_SHA256: &str =
        "03bd6ebd5cdb45f738de87363d3ad6a95de9bbb5aa119a2fce3199255b8efa55";
    pub const MALFORMED_JSON_LENGTH: usize = 151;
    pub const ANIMATED_JSON_SHA256: &str =
        "65c3d6d5e3764ec4a256996f84b2631f665fa14bc56f3dd75de5091b31c673ee";
    pub const ANIMATED_JSON_LENGTH: usize = 3886;

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

    pub fn linear_animated_glb() -> Vec<u8> {
        super::profile_a_animation_fixtures::mutate_json(
            super::profile_a_animation_fixtures::skin_animation_with_inverse_bind_matrices(),
            |root| {
                root["animations"][0]["samplers"][1]["interpolation"] = serde_json::json!("LINEAR");
                root["animations"][0]["samplers"]
                    .as_array_mut()
                    .expect("synthetic animation samplers")
                    .truncate(2);
                root["animations"][0]["channels"]
                    .as_array_mut()
                    .expect("synthetic animation channels")
                    .truncate(2);
            },
        )
    }

    pub fn linear_nonplanar_animated_glb() -> Vec<u8> {
        super::profile_a_animation_fixtures::mutate_accessor_f32(linear_animated_glb(), 0, 8, 0.5)
    }

    pub fn animated_profile() -> CreatureRigProfileV1 {
        let mut profile = rigid_profile();
        profile.target_bounds.max[2] = 2.0;
        let mut child_bind = identity();
        child_bind[0] = 0.0;
        child_bind[1] = 1.0;
        child_bind[4] = -1.0;
        child_bind[5] = 0.0;
        child_bind[12] = 10.0;
        child_bind[13] = 20.0;
        child_bind[14] = 30.0;
        profile.nodes.push(CreatureRigNodeV1 {
            id: 71,
            name: "synthetic-animated-child".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: child_bind,
        });
        finish(profile)
    }

    pub fn animation_mapping() -> ProfileAAnimationMappingV1 {
        ProfileAAnimationMappingV1 {
            schema_version: 1,
            source_skin_id: 0,
            provenance: RigProvenanceV1 {
                kind: RigProvenanceKindV1::Synthetic,
                export_allowed: true,
                attestations: RigProvenanceAttestationsV1 {
                    controlled_construction: true,
                    no_reference_payload_copied: true,
                    rights_confirmed: true,
                },
            },
            source_translation_scale: 1.0,
            node_mappings: vec![
                ProfileAAnimationNodeMappingV1 {
                    source_node_id: 0,
                    output_rig_node_id: 70,
                },
                ProfileAAnimationNodeMappingV1 {
                    source_node_id: 1,
                    output_rig_node_id: 71,
                },
            ],
            clip_mappings: vec![ProfileAAnimationClipMappingV1 {
                source_animation_id: 0,
                output_clip_name: "cpause1".to_owned(),
                transition_seconds: 0.25,
            }],
        }
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
        convert_profile_a_glb_json, convert_profile_a_json,
        convert_profile_a_with_animations_glb_json, profile_a_test_support as support,
        retarget_static_mesh_to_animated_donor_v1,
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

        let rig = support::skin_multi_profile();
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

    #[test]
    fn native_animated_profile_a_adapter_matches_core_and_rejects_invalid_mapping_json() {
        let glb = support::linear_animated_glb();
        let rig = support::animated_profile();
        let options = support::options();
        let mapping = support::animation_mapping();
        let rig_json = serde_json::to_string(&rig).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let mapping_json = serde_json::to_string(&mapping).unwrap();

        let source = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default())
            .expect("controlled animated source ingest");
        let core = m2a_core::profile_a::convert_profile_a_with_animations_v1(
            &source, &rig, &options, &mapping,
        )
        .expect("controlled animated core conversion");
        let expected = serde_json::to_string(&core).unwrap();
        assert_eq!(expected.len(), support::ANIMATED_JSON_LENGTH);
        assert_eq!(support::sha256(&expected), support::ANIMATED_JSON_SHA256);
        assert_eq!(
            convert_profile_a_with_animations_glb_json(
                &glb,
                &rig_json,
                &options_json,
                &mapping_json,
            ),
            expected
        );

        let malformed =
            convert_profile_a_with_animations_glb_json(&glb, &rig_json, &options_json, "{");
        assert_eq!(
            malformed,
            r#"{"schemaVersion":1,"code":"M4A-MAPPING-JSON-INVALID","severity":"FATAL","path":"mappingJson","message":"animation mapping JSON does not match the public schema"}"#
        );
        let mut unknown = serde_json::to_value(&mapping).unwrap();
        unknown["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_with_animations_glb_json(
                &glb,
                &rig_json,
                &options_json,
                &unknown.to_string(),
            ),
            malformed
        );
    }

    #[test]
    fn native_animated_donor_boundary_is_byte_and_report_identical_to_core() {
        use m2a_core::mdl::{
            MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
            MdlWriterOptionsV1,
        };

        let source = support::minimal_glb();
        let donor = support::linear_nonplanar_animated_glb();
        let options = MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "wasm_donor".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "wasm_donortex".to_owned(),
            }],
        };
        let core = m2a_core::animated_donor::retarget_static_mesh_to_animated_donor_v1(
            &source, &donor, &options,
        )
        .unwrap();
        let mut boundary = retarget_static_mesh_to_animated_donor_v1(
            &source,
            &donor,
            &serde_json::to_string(&options).unwrap(),
        )
        .unwrap();

        assert_eq!(boundary.take_model_bytes(), core.model.payload);
        assert!(boundary.take_model_bytes().is_empty());
        assert_eq!(
            boundary.report_json(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!(
            boundary.readback_json(),
            serde_json::to_string(&core.model.inspection).unwrap()
        );
        assert_eq!(
            boundary.animations_json(),
            serde_json::to_string(&core.animations).unwrap()
        );
        assert_eq!(
            boundary.conversion_json(),
            serde_json::to_string(&core.conversion).unwrap()
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::{
        HakResourceDescriptorV1, HakResourceDescriptorsV1, append_two_da_row_v1,
        append_two_da_row_v1_report_json, build_m6_model_package_v1, convert_profile_a_glb_json,
        convert_profile_a_json, convert_profile_a_with_animations_glb_json, ingest_glb,
        ingest_glb_json, inspect_binary_mdl, inspect_glb, inspect_glb_json, inspect_two_da_v2_json,
        profile_a_test_support as profile_support,
        write_binary_mdl_with_animations as write_mdl_bytes,
        write_binary_mdl_with_animations_report_json as write_mdl_report_json, write_hak_v1,
        write_hak_v1_report_json, write_package_manifest_v1_json, write_tga_v1,
        write_tga_v1_report_json,
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
    fn m5_tga_boundary_matches_core_is_immutable_and_has_frozen_error() {
        use m2a_core::tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1};

        let image = TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgb8,
            pixels: vec![255, 0, 128],
        };
        let options = TgaWriterOptionsV1::default();
        let image_json = serde_json::to_string(&image).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (image_json.clone(), options_json.clone());
        let core = m2a_core::tga::write_tga_v1(&image, &options).unwrap();

        assert_eq!(
            write_tga_v1(&image_json, &options_json).unwrap(),
            core.payload
        );
        assert_eq!(
            write_tga_v1_report_json(&image_json, &options_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((image_json, options_json), before);

        let error = write_tga_v1("{", &before.1)
            .unwrap_err()
            .as_string()
            .unwrap();
        assert_eq!(
            error,
            r#"{"schemaVersion":1,"code":"M5-TGA-IMAGE-JSON-INVALID","severity":"FATAL","path":"imageJson","message":"image JSON does not match the strict public schema"}"#
        );
        assert!(!error.to_ascii_lowercase().contains("base64"));
        for nested in [false, true] {
            let mut invalid = serde_json::to_value(options).unwrap();
            if nested {
                invalid["limits"]["unknown"] = serde_json::json!(true);
            } else {
                invalid["unknown"] = serde_json::json!(true);
            }
            let error = write_tga_v1(&before.0, &invalid.to_string())
                .unwrap_err()
                .as_string()
                .unwrap();
            assert!(error.contains("M5-TGA-OPTIONS-JSON-INVALID"));
        }
    }

    #[wasm_bindgen_test]
    fn m5_two_da_boundary_matches_core_is_immutable_and_strict() {
        use m2a_core::two_da::{
            TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        };

        let source = b"2DA V2.0\n\nA B\n0 old ****\n".to_vec();
        let request = TwoDaAppendRequestV1 {
            schema_version: 1,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "new".to_owned(),
                },
            }],
        };
        let limits = TwoDaLimitsV1::default();
        let request_json = serde_json::to_string(&request).unwrap();
        let limits_json = serde_json::to_string(&limits).unwrap();
        let before = (source.clone(), request_json.clone(), limits_json.clone());

        let inspection = m2a_core::two_da::inspect_two_da_v2(&source, &limits).unwrap();
        assert_eq!(
            inspect_two_da_v2_json(&source, &limits_json).unwrap(),
            serde_json::to_string(&inspection).unwrap()
        );
        let core = m2a_core::two_da::append_two_da_row_v1(&source, &request, &limits).unwrap();
        assert_eq!(
            append_two_da_row_v1(&source, &request_json, &limits_json).unwrap(),
            core.payload
        );
        assert_eq!(
            append_two_da_row_v1_report_json(&source, &request_json, &limits_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((source, request_json, limits_json), before);

        let error = append_two_da_row_v1(&before.0, "{", "{")
            .unwrap_err()
            .as_string()
            .unwrap();
        assert_eq!(
            error,
            r#"{"schemaVersion":1,"code":"M5-2DA-REQUEST-JSON-INVALID","severity":"FATAL","path":"requestJson","message":"2DA append request JSON does not match the strict public schema"}"#
        );
        assert!(!error.to_ascii_lowercase().contains("base64"));
        let mut invalid_limits = serde_json::to_value(limits).unwrap();
        invalid_limits["unknown"] = serde_json::json!(true);
        let error = inspect_two_da_v2_json(&before.0, &invalid_limits.to_string())
            .unwrap_err()
            .as_string()
            .unwrap();
        assert!(error.contains("M5-2DA-LIMITS-JSON-INVALID"));
        for path in ["request", "cell", "value"] {
            let mut invalid = serde_json::to_value(&request).unwrap();
            match path {
                "request" => invalid["unknown"] = serde_json::json!(true),
                "cell" => invalid["cells"][0]["unknown"] = serde_json::json!(true),
                "value" => invalid["cells"][0]["value"]["unknown"] = serde_json::json!(true),
                _ => unreachable!(),
            }
            let error = append_two_da_row_v1(&before.0, &invalid.to_string(), &before.2)
                .unwrap_err()
                .as_string()
                .unwrap();
            assert!(error.contains("M5-2DA-REQUEST-JSON-INVALID"), "{path}");
        }
    }

    #[wasm_bindgen_test]
    fn m5_hak_package_boundary_matches_core_is_immutable_and_strict() {
        use m2a_core::hak::{HakResourceInputV1, HakWriterOptionsV1};

        let entries: [(&str, u16, &[u8]); 3] = [
            ("texture", 3, b"tga"),
            ("appearance", 2017, b"2da"),
            ("model", 2002, b"mdl"),
        ];
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        let mut resources = Vec::new();
        for (resref, resource_type, payload) in entries {
            let payload_offset = blob.len() as u32;
            blob.extend_from_slice(payload);
            descriptors.push(HakResourceDescriptorV1 {
                resref: resref.to_owned(),
                resource_type,
                payload_offset,
                payload_size: payload.len() as u32,
            });
            resources.push(HakResourceInputV1 {
                resref: resref.to_owned(),
                resource_type,
                payload: payload.to_vec(),
            });
        }
        let descriptors = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: descriptors,
        };
        let options = HakWriterOptionsV1::default();
        let resources_json = serde_json::to_string(&descriptors).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (blob.clone(), resources_json.clone(), options_json.clone());
        let hak = m2a_core::hak::write_hak_v1(&resources, &options).unwrap();
        let manifest = m2a_core::package::write_package_manifest_v1(&resources, &options).unwrap();

        assert_eq!(
            write_hak_v1(&blob, &resources_json, &options_json).unwrap(),
            hak.payload
        );
        assert_eq!(
            write_hak_v1_report_json(&blob, &resources_json, &options_json).unwrap(),
            serde_json::to_string(&hak.report).unwrap()
        );
        assert_eq!(
            write_package_manifest_v1_json(&blob, &resources_json, &options_json).unwrap(),
            serde_json::to_string(&manifest).unwrap()
        );
        assert_eq!((blob, resources_json, options_json), before);

        let error = write_hak_v1(&before.0, "{", "{")
            .unwrap_err()
            .as_string()
            .unwrap();
        assert_eq!(
            error,
            r#"{"schemaVersion":1,"code":"M5-HAK-RESOURCES-JSON-INVALID","severity":"FATAL","path":"resourcesJson","message":"HAK resources JSON does not match the strict public schema"}"#
        );
        assert!(!error.to_ascii_lowercase().contains("base64"));
    }

    #[wasm_bindgen_test]
    fn studio_model_package_public_wasm_boundary_matches_core() {
        let source = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let appearance = b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY\r\n0 Existing NORM P existing **** **** 0 R 1.0 4\r\n";
        let core =
            m2a_core::model_pipeline::build_m6_model_package_v1(&source, appearance).unwrap();
        let mut studio = build_m6_model_package_v1(&source, appearance).unwrap();

        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert!(studio.take_hak_bytes().is_empty());
        assert_eq!(studio.take_model_bytes(), core.model);
        assert!(studio.take_model_bytes().is_empty());
        assert_eq!(
            studio.report_json(),
            String::from_utf8(core.report_json).unwrap()
        );
        assert_eq!(
            studio.manifest_json(),
            String::from_utf8(core.manifest_json).unwrap()
        );
        assert_eq!(
            studio.summary_json(),
            String::from_utf8(core.summary_json).unwrap()
        );
    }

    #[wasm_bindgen_test]
    fn public_animation_writer_is_native_wasm_byte_and_report_identical() {
        use m2a_core::mdl::{
            MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
            MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlFormatProfileV1,
            MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1, MdlWriterOptionsV1,
        };
        use m2a_core::profile_a::{
            AuroraCreatureIrV1, AuroraCreatureNodeV1, AuroraCreatureSegmentV1,
            MaterialSourceBindingV1, RigSegmentDeformationV1,
        };

        let identity = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let creature = AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "wasm-owned-animation".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
            engine_facing_proof: "OPEN_M6".to_owned(),
            uv_runtime_proof: "OPEN_M6".to_owned(),
            nodes: vec![AuroraCreatureNodeV1 {
                id: 70,
                name: "root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity,
            }],
            material_source_bindings: vec![MaterialSourceBindingV1 {
                slot: 0,
                source_material_id: None,
                source_material_name: None,
            }],
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Rigid,
                parent_node_id: 70,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: Vec::new(),
            }],
        };
        let animations = MdlAnimationSetV1 {
            schema_version: 1,
            clips: vec![MdlAnimationClipV1 {
                name: "cpause1".to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: vec![MdlAnimationTrackV1 {
                    target_node_id: 70,
                    path: MdlAnimationTrackPathV1::Translation,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: vec![0.0, 1.0],
                    values: vec![vec![0.0, 0.0, 0.0], vec![0.25, 0.0, 0.0]],
                }],
            }],
        };
        let options = MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "wasm_mdl".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "wasm_tex".to_owned(),
            }],
        };
        let native =
            m2a_core::mdl::write_binary_mdl_with_animations(&creature, &animations, &options)
                .expect("native writer");
        let creature_json = serde_json::to_string(&creature).unwrap();
        let animations_json = serde_json::to_string(&animations).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();

        let wasm_bytes =
            write_mdl_bytes(&creature_json, &animations_json, &options_json).expect("WASM writer");
        let wasm_report = write_mdl_report_json(&creature_json, &animations_json, &options_json);
        assert_eq!(wasm_bytes, native.payload);
        assert_eq!(wasm_report, serde_json::to_string(&native.report).unwrap());

        let invalid = write_mdl_report_json(
            &creature_json.replacen("{", "{\"unknown\":true,", 1),
            &animations_json,
            &options_json,
        );
        let error: serde_json::Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(error["code"], "M4A-CREATURE-JSON-INVALID");
        assert_eq!(error["path"], "creatureJson");

        let invalid = write_mdl_report_json(
            &creature_json,
            &animations_json.replacen("{", "{\"unknown\":true,", 1),
            &options_json,
        );
        let error: serde_json::Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(error["code"], "M4A-ANIMATION-JSON-INVALID");
        assert_eq!(error["path"], "animationsJson");

        let invalid = write_mdl_report_json(
            &creature_json,
            &animations_json,
            &options_json.replacen("{", "{\"unknown\":true,", 1),
        );
        let error: serde_json::Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(error["code"], "M4A-OPTIONS-JSON-INVALID");
        assert_eq!(error["path"], "optionsJson");
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
    fn animated_profile_a_adapter_is_native_wasm_json_identical_and_strict() {
        let glb = profile_support::linear_animated_glb();
        let rig = profile_support::animated_profile();
        let options = profile_support::options();
        let mapping = profile_support::animation_mapping();
        let source = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default())
            .expect("controlled animated source ingest");
        let core = m2a_core::profile_a::convert_profile_a_with_animations_v1(
            &source, &rig, &options, &mapping,
        )
        .expect("controlled animated core conversion");
        let rig_json = serde_json::to_string(&rig).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let mapping_json = serde_json::to_string(&mapping).unwrap();
        let actual = convert_profile_a_with_animations_glb_json(
            &glb,
            &rig_json,
            &options_json,
            &mapping_json,
        );
        assert_eq!(actual, serde_json::to_string(&core).unwrap());
        assert_eq!(actual.len(), profile_support::ANIMATED_JSON_LENGTH);
        assert_eq!(
            profile_support::sha256(&actual),
            profile_support::ANIMATED_JSON_SHA256
        );

        let malformed =
            convert_profile_a_with_animations_glb_json(&glb, &rig_json, &options_json, "{");
        let error: serde_json::Value = serde_json::from_str(&malformed).unwrap();
        assert_eq!(error["code"], "M4A-MAPPING-JSON-INVALID");
        assert_eq!(error["severity"], "FATAL");
        assert_eq!(error["path"], "mappingJson");

        let mut unknown = serde_json::to_value(&mapping).unwrap();
        unknown["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_with_animations_glb_json(
                &glb,
                &rig_json,
                &options_json,
                &unknown.to_string(),
            ),
            malformed
        );
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
