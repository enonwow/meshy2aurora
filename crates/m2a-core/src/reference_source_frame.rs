//! Explicit declaration for geometry authored in the exact selected bind frame.
//! This changes registration only. Skin, topology and motion admission remain mandatory.
use crate::reference_supermodel_generic::ReferenceSupermodelGenericErrorV2;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegisteredSourceV1 {
    schema_version: u32,
    coordinate_space: String,
    supermodel_resref: String,
    reference_sha256: String,
    original_source_sha256: String,
    algorithm: String,
    fit_matrix_aurora: [f32; 16],
}
fn fail(message: impl Into<String>) -> ReferenceSupermodelGenericErrorV2 {
    ReferenceSupermodelGenericErrorV2 {
        schema_version: 2,
        code: "M2A-SOURCE-REFERENCE-FRAME-INVALID".into(),
        path: "asset.extras.m2aReferenceBindV1".into(),
        message: message.into(),
    }
}
pub fn source_uses_reference_bind_frame_v1(
    bytes: &[u8],
    selected: &str,
    reference_sha: &str,
    source_forward: &str,
) -> Result<bool, ReferenceSupermodelGenericErrorV2> {
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err(fail("Expected a validated GLB input"));
    }
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let json_bytes = bytes
        .get(
            20..20usize
                .checked_add(length)
                .ok_or_else(|| fail("JSON range overflow"))?,
        )
        .ok_or_else(|| fail("JSON range exceeds GLB"))?;
    let json: serde_json::Value =
        serde_json::from_slice(json_bytes).map_err(|e| fail(e.to_string()))?;
    let Some(marker) = json.pointer("/asset/extras/m2aReferenceBindV1") else {
        return Ok(false);
    };
    let declaration: RegisteredSourceV1 =
        serde_json::from_value(marker.clone()).map_err(|e| fail(e.to_string()))?;
    if declaration.schema_version != 1
        || declaration.coordinate_space != "GLTF_POSITIVE_Z_TO_AURORA_BIND"
        || source_forward != "POSITIVE_Z"
        || declaration.algorithm != "AFFINE_SOURCE_FIT_V1"
    {
        return Err(fail("Unsupported source-frame declaration"));
    }
    if declaration.supermodel_resref != selected.to_ascii_lowercase()
        || declaration.reference_sha256 != reference_sha
    {
        return Err(fail(
            "Registered source belongs to a different exact supermodel",
        ));
    }
    if declaration.original_source_sha256.len() != 64
        || !declaration
            .original_source_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || declaration.fit_matrix_aurora.iter().any(|v| !v.is_finite())
    {
        return Err(fail("Invalid registration provenance"));
    }
    let matrix = declaration.fit_matrix_aurora;
    if [0, 5, 10].iter().any(|&i| matrix[i] <= 0.)
        || [1, 2, 3, 4, 6, 7, 8, 9, 11]
            .iter()
            .any(|&i| matrix[i] != 0.)
        || matrix[15] != 1.
    {
        return Err(fail(
            "V1 registration requires positive axis scales and translation",
        ));
    }
    // The declaration describes baked positions. Object transforms would create
    // a second, incompatible frame between preparation and Profile A conversion.
    let nodes = json
        .get("nodes")
        .and_then(|n| n.as_array())
        .ok_or_else(|| fail("Source nodes missing"))?;
    if nodes.len() != 1
        || nodes[0].get("mesh").is_none()
        || [
            "matrix",
            "translation",
            "rotation",
            "scale",
            "skin",
            "children",
        ]
        .iter()
        .any(|k| nodes[0].get(*k).is_some())
    {
        return Err(fail(
            "Registered source requires one baked, identity mesh node",
        ));
    }
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn glb(marker: serde_json::Value) -> Vec<u8> {
        let j=serde_json::to_vec(&serde_json::json!({"asset":{"version":"2.0","extras":{"m2aReferenceBindV1":marker}},"nodes":[{"mesh":0}]})).unwrap();
        let mut b = vec![0; 20];
        b[0..4].copy_from_slice(b"glTF");
        b[12..16].copy_from_slice(&(j.len() as u32).to_le_bytes());
        b.extend(j);
        b
    }
    fn marker() -> serde_json::Value {
        serde_json::json!({"schemaVersion":1,"coordinateSpace":"GLTF_POSITIVE_Z_TO_AURORA_BIND","supermodelResref":"fixture","referenceSha256":"a".repeat(64),"originalSourceSha256":"b".repeat(64),"algorithm":"AFFINE_SOURCE_FIT_V1","fitMatrixAurora":[1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.]})
    }
    #[test]
    fn frame_requires_exact_reference_and_basis() {
        let bytes = glb(marker());
        assert!(
            source_uses_reference_bind_frame_v1(&bytes, "fixture", &"a".repeat(64), "POSITIVE_Z")
                .unwrap()
        );
        assert!(
            source_uses_reference_bind_frame_v1(&bytes, "fixture", &"c".repeat(64), "POSITIVE_Z")
                .is_err()
        );
        assert!(
            source_uses_reference_bind_frame_v1(&bytes, "fixture", &"a".repeat(64), "NEGATIVE_Z")
                .is_err()
        );
    }
    #[test]
    fn truncated_declaration_is_rejected() {
        assert!(
            source_uses_reference_bind_frame_v1(b"glTF", "fixture", "a", "POSITIVE_Z").is_err()
        );
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    #[test]
    fn rejects_reflection_and_shear() {
        for (index, value) in [(0, -1.), (1, 0.1), (15, 0.)] {
            let mut m = serde_json::json!({"schemaVersion":1,"coordinateSpace":"GLTF_POSITIVE_Z_TO_AURORA_BIND","supermodelResref":"fixture","referenceSha256":"a".repeat(64),"originalSourceSha256":"b".repeat(64),"algorithm":"AFFINE_SOURCE_FIT_V1","fitMatrixAurora":[1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.]});
            m["fitMatrixAurora"][index] = serde_json::json!(value);
            let j=serde_json::to_vec(&serde_json::json!({"asset":{"extras":{"m2aReferenceBindV1":m}},"nodes":[{"mesh":0}]})).unwrap();
            let mut b = vec![0; 20];
            b[..4].copy_from_slice(b"glTF");
            b[12..16].copy_from_slice(&(j.len() as u32).to_le_bytes());
            b.extend(j);
            assert!(
                source_uses_reference_bind_frame_v1(&b, "fixture", &"a".repeat(64), "POSITIVE_Z")
                    .is_err()
            );
        }
    }
}
