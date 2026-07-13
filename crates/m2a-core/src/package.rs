use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::erf::{ErfArchive, ErfFileType};
use crate::hak::{
    ALLOCATION_FAILED, HAK_WRITER_SCHEMA_VERSION, HakArtifactV1, HakResourceInputV1, HakWriteError,
    HakWriterOptionsV1, write_hak_v1,
};

pub const PACKAGE_ROLE_MISSING: &str = "M5-PACKAGE-ROLE-MISSING";
pub const PACKAGE_ROLE_DUPLICATE: &str = "M5-PACKAGE-ROLE-DUPLICATE";
pub const PACKAGE_RESOURCE_INVALID: &str = "M5-PACKAGE-RESOURCE-INVALID";
pub const PACKAGE_HAK_MISMATCH: &str = "M5-PACKAGE-HAK-MISMATCH";
pub const PACKAGE_SEMANTIC_DIFF: &str = "M5-PACKAGE-SEMANTIC-DIFF";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackageResourceRoleV1 {
    Model,
    Texture,
    AppearanceTable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageManifestResourceV1 {
    pub role: PackageResourceRoleV1,
    pub resref: String,
    #[serde(rename = "type")]
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
    pub hak_resource_id: u32,
    pub hak_payload_offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageManifestV1 {
    pub schema_version: u32,
    pub package_sha256: String,
    pub resources: Vec<PackageManifestResourceV1>,
}

pub fn write_package_manifest_v1(
    resources: &[HakResourceInputV1],
    options: &HakWriterOptionsV1,
) -> Result<PackageManifestV1, HakWriteError> {
    let artifact = write_hak_v1(resources, options)?;
    build_package_manifest_from_artifact(resources, &artifact)
}

fn build_package_manifest_from_artifact(
    source_resources: &[HakResourceInputV1],
    artifact: &HakArtifactV1,
) -> Result<PackageManifestV1, HakWriteError> {
    let reports = &artifact.report.resources;
    if reports.len() < 3 {
        return Err(package_error(
            PACKAGE_ROLE_MISSING,
            "resources",
            "package profile requires exactly three resources",
        ));
    }
    if reports.len() > 3 {
        return Err(package_error(
            PACKAGE_RESOURCE_INVALID,
            "resources[3].resourceType",
            "package profile does not allow extra resources",
        ));
    }

    let mut roles = [None; 3];
    let mut seen = [false; 3];
    for (index, report) in reports.iter().enumerate() {
        let role = classify_role(&report.resref, report.resource_type, index)?;
        let slot = role_index(role);
        if seen[slot] {
            return Err(package_error(
                PACKAGE_ROLE_DUPLICATE,
                &format!("resources[{index}]"),
                "package resource role is duplicated",
            ));
        }
        seen[slot] = true;
        roles[index] = Some(role);
    }
    if seen.iter().any(|present| !present) {
        return Err(package_error(
            PACKAGE_ROLE_MISSING,
            "resources",
            "package profile is missing a required resource role",
        ));
    }

    validate_hak_artifact(source_resources, artifact)?;

    let mut manifest_resources = Vec::new();
    manifest_resources.try_reserve_exact(3).map_err(|_| {
        package_error(
            ALLOCATION_FAILED,
            "output",
            "could not reserve package manifest resources",
        )
    })?;
    for (index, report) in reports.iter().enumerate() {
        manifest_resources.push(PackageManifestResourceV1 {
            role: roles[index].expect("all package roles were classified"),
            resref: clone_string_fallible(&report.resref)?,
            resource_type: report.resource_type,
            byte_length: u64::from(report.payload_size),
            sha256: clone_string_fallible(&report.payload_sha256)?,
            hak_resource_id: report.resource_id,
            hak_payload_offset: report.payload_offset,
        });
    }
    let manifest = PackageManifestV1 {
        schema_version: HAK_WRITER_SCHEMA_VERSION,
        package_sha256: clone_string_fallible(&artifact.report.archive_sha256)?,
        resources: manifest_resources,
    };
    verify_package_manifest(&manifest, artifact)?;
    Ok(manifest)
}

fn classify_role(
    resref: &str,
    resource_type: u16,
    index: usize,
) -> Result<PackageResourceRoleV1, HakWriteError> {
    if resref == "appearance" {
        return if resource_type == 2017 {
            Ok(PackageResourceRoleV1::AppearanceTable)
        } else {
            Err(package_error(
                PACKAGE_RESOURCE_INVALID,
                &format!("resources[{index}].resourceType"),
                "appearance resource must have type 2017",
            ))
        };
    }
    match resource_type {
        2002 => Ok(PackageResourceRoleV1::Model),
        3 => Ok(PackageResourceRoleV1::Texture),
        2017 => Err(package_error(
            PACKAGE_RESOURCE_INVALID,
            &format!("resources[{index}].resref"),
            "appearance table type 2017 requires exact resref appearance",
        )),
        _ => Err(package_error(
            PACKAGE_RESOURCE_INVALID,
            &format!("resources[{index}].resourceType"),
            "resource type is not valid for the package profile",
        )),
    }
}

const fn role_index(role: PackageResourceRoleV1) -> usize {
    match role {
        PackageResourceRoleV1::Model => 0,
        PackageResourceRoleV1::Texture => 1,
        PackageResourceRoleV1::AppearanceTable => 2,
    }
}

fn validate_hak_artifact(
    source_resources: &[HakResourceInputV1],
    artifact: &HakArtifactV1,
) -> Result<(), HakWriteError> {
    let archive = ErfArchive::parse(&artifact.payload).map_err(|error| {
        package_error(
            PACKAGE_HAK_MISMATCH,
            "hakReport",
            format!("generated HAK no longer parses: {error}"),
        )
    })?;
    if archive.file_type() != ErfFileType::Hak
        || artifact.report.entry_count as usize != archive.resources().len()
        || artifact.report.resources.len() != archive.resources().len()
        || artifact.report.byte_length != artifact.payload.len() as u64
        || artifact.report.archive_sha256 != sha256_hex(&artifact.payload)?
    {
        return Err(hak_mismatch("HAK report header or package hash differs"));
    }

    for (index, (reported, actual)) in artifact
        .report
        .resources
        .iter()
        .zip(archive.resources())
        .enumerate()
    {
        let source = source_resources
            .iter()
            .find(|resource| {
                resource.resref == reported.resref
                    && resource.resource_type == reported.resource_type
            })
            .ok_or_else(|| hak_mismatch("HAK report resource is absent from source input"))?;
        let payload = archive
            .find(&reported.resref, reported.resource_type)
            .map_err(|error| hak_mismatch(format!("HAK resource lookup failed: {error}")))?;
        if reported.resource_id != index as u32
            || reported.resource_id != actual.resource_id
            || reported.resref != actual.resref
            || reported.resource_type != actual.resource_type
            || reported.payload_offset as usize != actual.offset
            || reported.payload_size as usize != actual.size
            || payload != source.payload
            || reported.payload_sha256 != sha256_hex(payload)?
        {
            return Err(hak_mismatch(format!(
                "HAK report resource {index} differs from payload or source"
            )));
        }
    }
    Ok(())
}

fn verify_package_manifest(
    manifest: &PackageManifestV1,
    artifact: &HakArtifactV1,
) -> Result<(), HakWriteError> {
    if manifest.schema_version != HAK_WRITER_SCHEMA_VERSION
        || manifest.package_sha256 != artifact.report.archive_sha256
    {
        return Err(package_error(
            PACKAGE_SEMANTIC_DIFF,
            "manifest.packageSha256",
            "package manifest header or package hash differs",
        ));
    }
    if manifest.resources.len() != artifact.report.resources.len() {
        return Err(package_error(
            PACKAGE_SEMANTIC_DIFF,
            "manifest.resources[0]",
            "package manifest resource count differs",
        ));
    }
    for (index, (manifest_resource, report)) in manifest
        .resources
        .iter()
        .zip(&artifact.report.resources)
        .enumerate()
    {
        let expected_role = classify_role(&report.resref, report.resource_type, index)?;
        if manifest_resource.role != expected_role
            || manifest_resource.resref != report.resref
            || manifest_resource.resource_type != report.resource_type
            || manifest_resource.byte_length != u64::from(report.payload_size)
            || manifest_resource.sha256 != report.payload_sha256
            || manifest_resource.hak_resource_id != report.resource_id
            || manifest_resource.hak_payload_offset != report.payload_offset
        {
            return Err(package_error(
                PACKAGE_SEMANTIC_DIFF,
                &format!("manifest.resources[{index}]"),
                "package manifest resource differs from HAK report",
            ));
        }
    }
    Ok(())
}

fn package_error(code: &str, path: &str, message: impl fmt::Display) -> HakWriteError {
    HakWriteError {
        schema_version: HAK_WRITER_SCHEMA_VERSION,
        code: code.to_owned(),
        severity: "FATAL".to_owned(),
        path: path.to_owned(),
        message: message.to_string(),
    }
}

fn hak_mismatch(message: impl fmt::Display) -> HakWriteError {
    package_error(PACKAGE_HAK_MISMATCH, "hakReport", message)
}

fn clone_string_fallible(value: &str) -> Result<String, HakWriteError> {
    let mut output = String::new();
    output.try_reserve_exact(value.len()).map_err(|_| {
        package_error(
            ALLOCATION_FAILED,
            "output",
            "could not reserve package manifest string",
        )
    })?;
    output.push_str(value);
    Ok(output)
}

fn sha256_hex(bytes: &[u8]) -> Result<String, HakWriteError> {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::new();
    output.try_reserve_exact(64).map_err(|_| {
        package_error(
            ALLOCATION_FAILED,
            "output",
            "could not reserve package SHA-256 string",
        )
    })?;
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    fn resource(resref: &str, resource_type: u16, payload: &[u8]) -> HakResourceInputV1 {
        HakResourceInputV1 {
            resref: resref.to_owned(),
            resource_type,
            payload: payload.to_vec(),
        }
    }

    fn resources() -> Vec<HakResourceInputV1> {
        vec![
            resource("texture", 3, b"tga"),
            resource("appearance", 2017, b"2da"),
            resource("model", 2002, b"mdl"),
        ]
    }

    #[test]
    fn private_artifact_and_manifest_mutation_seams_have_stable_classification() {
        let resources = resources();
        let artifact = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap();
        let manifest = build_package_manifest_from_artifact(&resources, &artifact).unwrap();

        for mutation in 0..6 {
            let mut changed = artifact.clone();
            match mutation {
                0 => changed.payload[changed.report.payload_offset as usize] ^= 1,
                1 => changed.report.archive_sha256.replace_range(0..1, "!"),
                2 => changed.report.resources[0]
                    .payload_sha256
                    .replace_range(0..1, "!"),
                3 => changed.report.resources[0].resource_id += 1,
                4 => changed.report.resources[0].payload_offset += 1,
                5 => changed.report.resources[0].payload_size += 1,
                _ => unreachable!(),
            }
            let result = catch_unwind(AssertUnwindSafe(|| {
                build_package_manifest_from_artifact(&resources, &changed)
            }));
            let error = result
                .expect("mutated HAK artifact must not panic")
                .unwrap_err();
            assert_eq!(error.code, PACKAGE_HAK_MISMATCH, "mutation {mutation}");
            assert_eq!(error.path, "hakReport");
        }

        for mutation in 0..7 {
            let mut changed = manifest.clone();
            match mutation {
                0 => changed.resources[0].role = PackageResourceRoleV1::Model,
                1 => changed.resources[0].sha256.replace_range(0..1, "!"),
                2 => changed.resources[0].hak_resource_id += 1,
                3 => changed.resources[0].hak_payload_offset += 1,
                4 => changed.resources[0].byte_length += 1,
                5 => changed.resources[0].resref.push('x'),
                6 => changed.resources[0].resource_type += 1,
                _ => unreachable!(),
            }
            let error = verify_package_manifest(&changed, &artifact).unwrap_err();
            assert_eq!(error.code, PACKAGE_SEMANTIC_DIFF);
            assert_eq!(error.path, "manifest.resources[0]");
        }
        let mut changed = manifest;
        changed.package_sha256.replace_range(0..1, "!");
        let error = verify_package_manifest(&changed, &artifact).unwrap_err();
        assert_eq!(error.code, PACKAGE_SEMANTIC_DIFF);
        assert_eq!(error.path, "manifest.packageSha256");
    }
}
