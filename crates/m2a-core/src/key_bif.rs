//! Read-only Neverwinter Nights KEY/BIF resource resolver.
//!
//! This module deliberately accepts caller-owned byte slices. It never opens
//! the installation, rewrites a KEY/BIF, or exposes local filesystem paths in
//! its report.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const KEY_HEADER_SIZE_V1: usize = 24;
const KEY_FILE_ENTRY_SIZE_V1: usize = 12;
const KEY_RESOURCE_ENTRY_SIZE_V1: usize = 22;
const BIF_HEADER_SIZE_V1: usize = 20;
const BIF_VARIABLE_ENTRY_SIZE_V1: usize = 16;

#[derive(Clone, Debug)]
pub struct KeyBifFileInputV1<'a> {
    pub logical_name: String,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct KeyBifContextInputV1<'a> {
    pub schema_version: u32,
    pub key_file_name: String,
    pub key_bytes: &'a [u8],
    pub bif_files: Vec<KeyBifFileInputV1<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyBifFileReportV1 {
    pub logical_name: String,
    pub byte_length: usize,
    pub sha256: String,
    pub variable_resource_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyBifContextReportV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub key_file_name: String,
    pub key_byte_length: usize,
    pub key_sha256: String,
    pub key_resource_count: u32,
    pub bif_files: Vec<KeyBifFileReportV1>,
    pub context_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBifResolvedResourceV1<'a> {
    pub schema_version: u32,
    pub resref: String,
    pub resource_type: u16,
    pub bif_logical_name: String,
    pub bif_index: u32,
    pub resource_index: u32,
    pub payload: &'a [u8],
    pub payload_sha256: String,
    pub context_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyBifResourceLocatorV1 {
    pub schema_version: u32,
    pub key_file_name: String,
    pub key_sha256: String,
    pub resref: String,
    pub resource_type: u16,
    pub bif_logical_name: String,
    pub bif_index: u32,
    pub resource_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBifErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for KeyBifErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for KeyBifErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> KeyBifErrorV1 {
    KeyBifErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_u16(bytes: &[u8], offset: usize, path: &str) -> Result<u16, KeyBifErrorV1> {
    let value = bytes.get(offset..offset + 2).ok_or_else(|| {
        error(
            "KEY-BIF-RANGE-INVALID",
            path,
            format!("WORD at byte {offset} exceeds the input"),
        )
    })?;
    Ok(u16::from_le_bytes(
        value.try_into().expect("two-byte slice"),
    ))
}

fn read_u32(bytes: &[u8], offset: usize, path: &str) -> Result<u32, KeyBifErrorV1> {
    let value = bytes.get(offset..offset + 4).ok_or_else(|| {
        error(
            "KEY-BIF-RANGE-INVALID",
            path,
            format!("DWORD at byte {offset} exceeds the input"),
        )
    })?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("four-byte slice"),
    ))
}

fn checked_table<'a>(
    bytes: &'a [u8],
    offset: u32,
    count: u32,
    stride: usize,
    path: &str,
) -> Result<&'a [u8], KeyBifErrorV1> {
    let offset = usize::try_from(offset).map_err(|_| {
        error(
            "KEY-BIF-RANGE-INVALID",
            path,
            "offset does not fit the platform",
        )
    })?;
    let length = usize::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(stride))
        .ok_or_else(|| error("KEY-BIF-RANGE-INVALID", path, "table length overflowed"))?;
    bytes.get(offset..offset + length).ok_or_else(|| {
        error(
            "KEY-BIF-RANGE-INVALID",
            path,
            "declared table exceeds the input",
        )
    })
}

fn normalize_name(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_matches('/')
        .to_ascii_lowercase()
}

fn validate_context_v1(
    context: &KeyBifContextInputV1<'_>,
) -> Result<(u32, u32, u32), KeyBifErrorV1> {
    if context.schema_version != 1
        || context.key_file_name.trim().is_empty()
        || context.key_bytes.len() < KEY_HEADER_SIZE_V1
        || context.key_bytes.get(..8) != Some(b"KEY V1  ")
    {
        return Err(error(
            "KEY-BIF-KEY-INVALID",
            "context.keyBytes",
            "context requires a KEY V1 file and schemaVersion 1",
        ));
    }
    let bif_count = read_u32(context.key_bytes, 8, "context.keyBytes.bifCount")?;
    let key_count = read_u32(context.key_bytes, 12, "context.keyBytes.keyCount")?;
    let file_table_offset = read_u32(context.key_bytes, 16, "context.keyBytes.fileTableOffset")?;
    let key_table_offset = read_u32(context.key_bytes, 20, "context.keyBytes.keyTableOffset")?;
    checked_table(
        context.key_bytes,
        file_table_offset,
        bif_count,
        KEY_FILE_ENTRY_SIZE_V1,
        "context.keyBytes.fileTable",
    )?;
    checked_table(
        context.key_bytes,
        key_table_offset,
        key_count,
        KEY_RESOURCE_ENTRY_SIZE_V1,
        "context.keyBytes.keyTable",
    )?;
    if context.bif_files.len() != bif_count as usize {
        return Err(error(
            "KEY-BIF-FILE-SET-INCOMPLETE",
            "context.bifFiles",
            format!(
                "KEY declares {bif_count} BIF files but {} were supplied",
                context.bif_files.len()
            ),
        ));
    }
    Ok((bif_count, key_count, file_table_offset))
}

fn validate_key_v1(
    key_file_name: &str,
    key_bytes: &[u8],
) -> Result<(u32, u32, u32, u32), KeyBifErrorV1> {
    if key_file_name.trim().is_empty()
        || key_bytes.len() < KEY_HEADER_SIZE_V1
        || key_bytes.get(..8) != Some(b"KEY V1  ")
    {
        return Err(error(
            "KEY-BIF-KEY-INVALID",
            "keyBytes",
            "a logical filename and KEY V1 payload are required",
        ));
    }
    let bif_count = read_u32(key_bytes, 8, "keyBytes.bifCount")?;
    let key_count = read_u32(key_bytes, 12, "keyBytes.keyCount")?;
    let file_table_offset = read_u32(key_bytes, 16, "keyBytes.fileTableOffset")?;
    let key_table_offset = read_u32(key_bytes, 20, "keyBytes.keyTableOffset")?;
    checked_table(
        key_bytes,
        file_table_offset,
        bif_count,
        KEY_FILE_ENTRY_SIZE_V1,
        "keyBytes.fileTable",
    )?;
    checked_table(
        key_bytes,
        key_table_offset,
        key_count,
        KEY_RESOURCE_ENTRY_SIZE_V1,
        "keyBytes.keyTable",
    )?;
    Ok((bif_count, key_count, file_table_offset, key_table_offset))
}

/// Resolves the exact BIF locator for one KEY resource without opening or
/// requiring any BIF payload. This lets local callers load only the winning
/// read-only BIF instead of materializing the complete installation corpus.
pub fn locate_key_bif_resource_v1(
    key_file_name: &str,
    key_bytes: &[u8],
    resref: &str,
    resource_type: u16,
) -> Result<KeyBifResourceLocatorV1, KeyBifErrorV1> {
    if resref.is_empty()
        || resref.len() > 16
        || !resref
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(error(
            "KEY-BIF-RESREF-INVALID",
            "resref",
            "resref must match [A-Za-z0-9_-]{1,16}",
        ));
    }
    let (bif_count, key_count, file_table_offset, key_table_offset) =
        validate_key_v1(key_file_name, key_bytes)?;
    let key_table_offset = key_table_offset as usize;
    let mut resource_id = None;
    for index in 0..key_count as usize {
        let offset = key_table_offset + index * KEY_RESOURCE_ENTRY_SIZE_V1;
        let raw_resref = &key_bytes[offset..offset + 16];
        let end = raw_resref.iter().position(|byte| *byte == 0).unwrap_or(16);
        let candidate = std::str::from_utf8(&raw_resref[..end]).map_err(|_| {
            error(
                "KEY-BIF-RESREF-INVALID",
                format!("key.resources[{index}].resref"),
                "KEY resource resref is not UTF-8",
            )
        })?;
        if candidate.eq_ignore_ascii_case(resref)
            && read_u16(key_bytes, offset + 16, "key.resourceType")? == resource_type
            && resource_id
                .replace(read_u32(key_bytes, offset + 18, "key.resourceId")?)
                .is_some()
        {
            return Err(error(
                "KEY-BIF-RESOURCE-AMBIGUOUS",
                "key.resources",
                "KEY contains the requested resource more than once",
            ));
        }
    }
    let resource_id = resource_id.ok_or_else(|| {
        error(
            "KEY-BIF-RESOURCE-MISSING",
            "key.resources",
            format!("resource ({resref}, {resource_type}) was not found"),
        )
    })?;
    let bif_index = resource_id >> 20;
    let resource_index = resource_id & 0x000f_ffff;
    if bif_index >= bif_count {
        return Err(error(
            "KEY-BIF-RESOURCE-ID-INVALID",
            "key.resourceId",
            "resource BIF index exceeds the declared file table",
        ));
    }
    Ok(KeyBifResourceLocatorV1 {
        schema_version: 1,
        key_file_name: key_file_name.to_owned(),
        key_sha256: sha256(key_bytes),
        resref: resref.to_ascii_lowercase(),
        resource_type,
        bif_logical_name: key_bif_declared_name_v1(key_bytes, file_table_offset, bif_index)?,
        bif_index,
        resource_index,
    })
}

/// Reads one resource from the single BIF named by the KEY locator. The
/// returned context identity binds the complete KEY, the winning BIF and the
/// requested resource; unrelated BIF payloads are neither required nor read.
pub fn resolve_key_bif_resource_sparse_v1<'a>(
    key_file_name: &str,
    key_bytes: &[u8],
    bif_file: &'a KeyBifFileInputV1<'a>,
    resref: &str,
    resource_type: u16,
) -> Result<KeyBifResolvedResourceV1<'a>, KeyBifErrorV1> {
    let locator = locate_key_bif_resource_v1(key_file_name, key_bytes, resref, resource_type)?;
    let declared = normalize_name(&locator.bif_logical_name);
    let supplied = normalize_name(&bif_file.logical_name);
    let declared_base = declared.rsplit('/').next().unwrap_or(&declared);
    if supplied != declared && supplied.rsplit('/').next() != Some(declared_base) {
        return Err(error(
            "KEY-BIF-FILE-MISMATCH",
            "bifFile.logicalName",
            format!(
                "KEY selected {} but caller supplied {}",
                locator.bif_logical_name, bif_file.logical_name
            ),
        ));
    }
    let variable_count = inspect_bif_v1(bif_file.bytes, "bifFile.bytes")?;
    if locator.resource_index >= variable_count {
        return Err(error(
            "KEY-BIF-RESOURCE-ID-INVALID",
            "key.resourceId",
            "resource index exceeds the BIF variable table",
        ));
    }
    let variable_table_offset = read_u32(bif_file.bytes, 16, "bif.variableTableOffset")? as usize;
    let entry =
        variable_table_offset + locator.resource_index as usize * BIF_VARIABLE_ENTRY_SIZE_V1;
    let bif_resource_id = read_u32(bif_file.bytes, entry, "bif.resourceId")?;
    let payload_offset = read_u32(bif_file.bytes, entry + 4, "bif.payloadOffset")? as usize;
    let payload_size = read_u32(bif_file.bytes, entry + 8, "bif.payloadSize")? as usize;
    let bif_resource_type = read_u32(bif_file.bytes, entry + 12, "bif.resourceType")?;
    if bif_resource_id & 0x000f_ffff != locator.resource_index
        || bif_resource_type as u16 != resource_type
    {
        return Err(error(
            "KEY-BIF-RESOURCE-BINDING-MISMATCH",
            "bif.variableTable",
            "BIF resource id/type does not match the KEY binding",
        ));
    }
    let payload = bif_file
        .bytes
        .get(payload_offset..payload_offset + payload_size)
        .ok_or_else(|| {
            error(
                "KEY-BIF-PAYLOAD-RANGE-INVALID",
                "bif.payload",
                "BIF resource payload exceeds the input",
            )
        })?;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SparseIdentity<'a> {
        schema_version: u32,
        algorithm: &'a str,
        key_file_name: &'a str,
        key_sha256: &'a str,
        bif_logical_name: &'a str,
        bif_sha256: String,
        resref: &'a str,
        resource_type: u16,
        payload_sha256: String,
    }
    let payload_sha256 = sha256(payload);
    let identity = serde_json::to_vec(&SparseIdentity {
        schema_version: 1,
        algorithm: "NWN_KEY_BIF_SPARSE_RESOURCE_RESOLUTION_V1",
        key_file_name,
        key_sha256: &locator.key_sha256,
        bif_logical_name: &locator.bif_logical_name,
        bif_sha256: sha256(bif_file.bytes),
        resref: &locator.resref,
        resource_type,
        payload_sha256: payload_sha256.clone(),
    })
    .map_err(|_| {
        error(
            "KEY-BIF-CONTEXT-SERIALIZE-FAILED",
            "context",
            "sparse KEY/BIF identity could not be serialized",
        )
    })?;
    Ok(KeyBifResolvedResourceV1 {
        schema_version: 1,
        resref: locator.resref,
        resource_type,
        bif_logical_name: locator.bif_logical_name,
        bif_index: locator.bif_index,
        resource_index: locator.resource_index,
        payload,
        payload_sha256,
        context_sha256: sha256(&identity),
    })
}

fn key_bif_declared_name_v1(
    key: &[u8],
    file_table_offset: u32,
    bif_index: u32,
) -> Result<String, KeyBifErrorV1> {
    let entry = usize::try_from(file_table_offset)
        .ok()
        .and_then(|offset| {
            usize::try_from(bif_index)
                .ok()
                .and_then(|index| index.checked_mul(KEY_FILE_ENTRY_SIZE_V1))
                .and_then(|relative| offset.checked_add(relative))
        })
        .ok_or_else(|| error("KEY-BIF-RANGE-INVALID", "key.fileTable", "entry overflowed"))?;
    let name_offset = read_u32(key, entry + 4, "key.fileTable.nameOffset")? as usize;
    let name_size = read_u16(key, entry + 8, "key.fileTable.nameSize")? as usize;
    let raw = key
        .get(name_offset..name_offset + name_size)
        .ok_or_else(|| {
            error(
                "KEY-BIF-RANGE-INVALID",
                "key.fileTable.name",
                "declared BIF filename exceeds KEY input",
            )
        })?;
    let text = std::str::from_utf8(raw)
        .map_err(|_| {
            error(
                "KEY-BIF-NAME-INVALID",
                "key.fileTable.name",
                "BIF name is not UTF-8",
            )
        })?
        .trim_end_matches('\0');
    if text.is_empty() {
        return Err(error(
            "KEY-BIF-NAME-INVALID",
            "key.fileTable.name",
            "BIF name is empty",
        ));
    }
    Ok(normalize_name(text))
}

fn match_bif_input_v1<'a>(
    context: &'a KeyBifContextInputV1<'a>,
    declared_name: &str,
) -> Result<&'a KeyBifFileInputV1<'a>, KeyBifErrorV1> {
    let declared_base = declared_name.rsplit('/').next().unwrap_or(declared_name);
    let matches = context
        .bif_files
        .iter()
        .filter(|input| {
            let supplied = normalize_name(&input.logical_name);
            supplied == declared_name || supplied.rsplit('/').next() == Some(declared_base)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [input] => Ok(*input),
        [] => Err(error(
            "KEY-BIF-FILE-MISSING",
            "context.bifFiles",
            format!("KEY requires BIF {declared_name}"),
        )),
        _ => Err(error(
            "KEY-BIF-FILE-AMBIGUOUS",
            "context.bifFiles",
            format!("multiple supplied BIFs match {declared_name}"),
        )),
    }
}

fn inspect_bif_v1(bytes: &[u8], path: &str) -> Result<u32, KeyBifErrorV1> {
    if bytes.len() < BIF_HEADER_SIZE_V1 || bytes.get(..8) != Some(b"BIFFV1  ") {
        return Err(error(
            "KEY-BIF-BIF-INVALID",
            path,
            "expected a BIFF V1 file",
        ));
    }
    let variable_count = read_u32(bytes, 8, path)?;
    let variable_table_offset = read_u32(bytes, 16, path)?;
    checked_table(
        bytes,
        variable_table_offset,
        variable_count,
        BIF_VARIABLE_ENTRY_SIZE_V1,
        path,
    )?;
    Ok(variable_count)
}

pub fn inspect_key_bif_context_v1(
    context: &KeyBifContextInputV1<'_>,
) -> Result<KeyBifContextReportV1, KeyBifErrorV1> {
    let (bif_count, key_count, file_table_offset) = validate_context_v1(context)?;
    let mut bif_files = Vec::with_capacity(bif_count as usize);
    for bif_index in 0..bif_count {
        let declared = key_bif_declared_name_v1(context.key_bytes, file_table_offset, bif_index)?;
        let input = match_bif_input_v1(context, &declared)?;
        let variable_resource_count = inspect_bif_v1(input.bytes, "context.bifFiles.bytes")?;
        bif_files.push(KeyBifFileReportV1 {
            logical_name: declared,
            byte_length: input.bytes.len(),
            sha256: sha256(input.bytes),
            variable_resource_count,
        });
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema_version: u32,
        algorithm: &'a str,
        key_file_name: &'a str,
        key_sha256: String,
        bif_files: &'a [KeyBifFileReportV1],
    }
    let identity = serde_json::to_vec(&Identity {
        schema_version: 1,
        algorithm: "NWN_KEY_BIF_READONLY_RESOLUTION_V1",
        key_file_name: &context.key_file_name,
        key_sha256: sha256(context.key_bytes),
        bif_files: &bif_files,
    })
    .map_err(|_| {
        error(
            "KEY-BIF-CONTEXT-SERIALIZE-FAILED",
            "context",
            "KEY/BIF context identity could not be serialized",
        )
    })?;
    Ok(KeyBifContextReportV1 {
        schema_version: 1,
        algorithm: "NWN_KEY_BIF_READONLY_RESOLUTION_V1".to_owned(),
        status: "PASSED".to_owned(),
        key_file_name: context.key_file_name.clone(),
        key_byte_length: context.key_bytes.len(),
        key_sha256: sha256(context.key_bytes),
        key_resource_count: key_count,
        bif_files,
        context_sha256: sha256(&identity),
    })
}

pub fn resolve_key_bif_resource_v1<'a>(
    context: &'a KeyBifContextInputV1<'a>,
    resref: &str,
    resource_type: u16,
) -> Result<KeyBifResolvedResourceV1<'a>, KeyBifErrorV1> {
    if resref.is_empty()
        || resref.len() > 16
        || !resref
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(error(
            "KEY-BIF-RESREF-INVALID",
            "resref",
            "resref must match [A-Za-z0-9_-]{1,16}",
        ));
    }
    let report = inspect_key_bif_context_v1(context)?;
    let key_count = read_u32(context.key_bytes, 12, "key.keyCount")?;
    let key_table_offset = read_u32(context.key_bytes, 20, "key.keyTableOffset")? as usize;
    let mut resource_id = None;
    for index in 0..key_count as usize {
        let offset = key_table_offset + index * KEY_RESOURCE_ENTRY_SIZE_V1;
        let raw_resref = &context.key_bytes[offset..offset + 16];
        let end = raw_resref.iter().position(|byte| *byte == 0).unwrap_or(16);
        let candidate = std::str::from_utf8(&raw_resref[..end]).map_err(|_| {
            error(
                "KEY-BIF-RESREF-INVALID",
                format!("key.resources[{index}].resref"),
                "KEY resource resref is not UTF-8",
            )
        })?;
        if candidate.eq_ignore_ascii_case(resref)
            && read_u16(context.key_bytes, offset + 16, "key.resourceType")? == resource_type
        {
            if resource_id
                .replace(read_u32(context.key_bytes, offset + 18, "key.resourceId")?)
                .is_some()
            {
                return Err(error(
                    "KEY-BIF-RESOURCE-AMBIGUOUS",
                    "key.resources",
                    "KEY contains the requested resource more than once",
                ));
            }
        }
    }
    let resource_id = resource_id.ok_or_else(|| {
        error(
            "KEY-BIF-RESOURCE-MISSING",
            "key.resources",
            format!("resource ({resref}, {resource_type}) was not found"),
        )
    })?;
    let bif_index = resource_id >> 20;
    let resource_index = resource_id & 0x000f_ffff;
    if bif_index >= report.bif_files.len() as u32 {
        return Err(error(
            "KEY-BIF-RESOURCE-ID-INVALID",
            "key.resourceId",
            "resource BIF index exceeds the declared file table",
        ));
    }
    let file_table_offset = read_u32(context.key_bytes, 16, "key.fileTableOffset")?;
    let declared = key_bif_declared_name_v1(context.key_bytes, file_table_offset, bif_index)?;
    let bif = match_bif_input_v1(context, &declared)?;
    let variable_count = inspect_bif_v1(bif.bytes, "bif")?;
    if resource_index >= variable_count {
        return Err(error(
            "KEY-BIF-RESOURCE-ID-INVALID",
            "key.resourceId",
            "resource index exceeds the BIF variable table",
        ));
    }
    let variable_table_offset = read_u32(bif.bytes, 16, "bif.variableTableOffset")? as usize;
    let entry = variable_table_offset + resource_index as usize * BIF_VARIABLE_ENTRY_SIZE_V1;
    let bif_resource_id = read_u32(bif.bytes, entry, "bif.resourceId")?;
    let payload_offset = read_u32(bif.bytes, entry + 4, "bif.payloadOffset")? as usize;
    let payload_size = read_u32(bif.bytes, entry + 8, "bif.payloadSize")? as usize;
    let bif_resource_type = read_u32(bif.bytes, entry + 12, "bif.resourceType")?;
    if bif_resource_id & 0x000f_ffff != resource_index || bif_resource_type as u16 != resource_type
    {
        return Err(error(
            "KEY-BIF-RESOURCE-BINDING-MISMATCH",
            "bif.variableTable",
            "BIF resource id/type does not match the KEY binding",
        ));
    }
    let payload = bif
        .bytes
        .get(payload_offset..payload_offset + payload_size)
        .ok_or_else(|| {
            error(
                "KEY-BIF-PAYLOAD-RANGE-INVALID",
                "bif.payload",
                "BIF resource payload exceeds the input",
            )
        })?;
    Ok(KeyBifResolvedResourceV1 {
        schema_version: 1,
        resref: resref.to_ascii_lowercase(),
        resource_type,
        bif_logical_name: declared,
        bif_index,
        resource_index,
        payload,
        payload_sha256: sha256(payload),
        context_sha256: report.context_sha256,
    })
}
