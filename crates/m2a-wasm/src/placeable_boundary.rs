//! Placeable-specific public WASM boundary.
//!
//! This module keeps JSON validation, transferable artifact composition and
//! the V1/V2/V3 ABI wrappers together without changing their JavaScript names.

use wasm_bindgen::prelude::*;

use super::{
    PlaceableBoundaryErrorV1, StudioModelPackageArtifactV1, parse_project_build_identity_v1,
    serialize_json,
};

pub(crate) fn build_meshy_static_placeable_package_v1_inner(
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
    let placement = parse_placement(placement_json)?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v1(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact)
}

pub(crate) fn build_meshy_static_placeable_package_v2_inner(
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
    let placement = parse_placement(placement_json)?;
    let authoring = parse_authoring(authoring_json)?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v2(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact)
}

pub(crate) fn build_meshy_static_placeable_package_v3_project_v1_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    project_identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: Option<&str>,
) -> Result<StudioModelPackageArtifactV1, String> {
    let project_identity = parse_project_build_identity_v1(project_identity_json)?;
    let placement = parse_placement(placement_json)?;
    let authoring = authoring_json
        .filter(|json| !json.trim().is_empty())
        .map(parse_authoring)
        .transpose()?;
    let artifact =
        m2a_core::placeable::build_meshy_static_placeable_package_v3_with_project_identity(
            source_glb,
            placeables_two_da,
            &project_identity,
            placement,
            palette_id,
            authoring.as_ref(),
        )
        .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact)
}

fn parse_placement(
    placement_json: &str,
) -> Result<m2a_core::placeable::PlaceablePlacementV1, String> {
    serde_json::from_str(placement_json).map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })
}

fn parse_authoring(
    authoring_json: &str,
) -> Result<m2a_core::placeable_authoring::PlaceableAuthoringDocumentV1, String> {
    serde_json::from_str(authoring_json).map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring schema",
        })
    })
}

fn finish_static_placeable_artifact_v1(
    artifact: m2a_core::placeable::StaticPlaceablePackageArtifactV1,
) -> Result<StudioModelPackageArtifactV1, String> {
    let hak = m2a_core::erf::ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| serialize_json(&error))?;
    let model = hak
        .find(
            &artifact.report.model_resref,
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

/// Compatibility V1 boundary with caller-supplied historical resource names.
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

/// Compatibility V2 authored boundary with caller-supplied historical names.
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

/// Project-owned V3 Placeable boundary used by the current Studio.
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV3ProjectV1)]
pub fn build_meshy_static_placeable_package_v3_project_v1(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    project_identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: Option<String>,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v3_project_v1_inner(
        source_glb,
        placeables_two_da,
        project_identity_json,
        placement_json,
        palette_id,
        authoring_json.as_deref(),
    )
    .map_err(|error| JsValue::from_str(&error))
}
