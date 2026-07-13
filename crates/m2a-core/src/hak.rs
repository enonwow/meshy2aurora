use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::erf::{ErfArchive, ErfFileType, ErfLimits};

pub const HAK_WRITER_SCHEMA_VERSION: u32 = 1;
pub const HAK_MAX_ENTRY_COUNT: u64 = 262_144;
pub const HAK_MAX_OUTPUT_BYTES: u64 = 256 * 1024 * 1024;

pub const OPTIONS_INVALID: &str = "M5-HAK-OPTIONS-INVALID";
pub const RESREF_INVALID: &str = "M5-HAK-RESREF-INVALID";
pub const DUPLICATE_KEY: &str = "M5-HAK-DUPLICATE-KEY";
pub const ENTRY_LIMIT_EXCEEDED: &str = "M5-HAK-ENTRY-LIMIT-EXCEEDED";
pub const OUTPUT_LIMIT_EXCEEDED: &str = "M5-HAK-OUTPUT-LIMIT-EXCEEDED";
pub const U32_OVERFLOW: &str = "M5-HAK-U32-OVERFLOW";
pub const ALLOCATION_FAILED: &str = "M5-HAK-ALLOCATION-FAILED";
pub const READBACK_FAILED: &str = "M5-HAK-READBACK-FAILED";
pub const SEMANTIC_DIFF: &str = "M5-HAK-SEMANTIC-DIFF";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HakResourceInputV1 {
    pub resref: String,
    pub resource_type: u16,
    pub payload: Vec<u8>,
}

/// Borrowed metadata required to validate and plan a HAK before payload bytes
/// are copied into owned resource buffers.
pub trait HakResourceMetadataV1 {
    fn hak_resref(&self) -> &str;
    fn hak_resource_type(&self) -> u16;
    fn hak_payload_size(&self) -> Option<u64>;
}

impl HakResourceMetadataV1 for HakResourceInputV1 {
    fn hak_resref(&self) -> &str {
        &self.resref
    }

    fn hak_resource_type(&self) -> u16 {
        self.resource_type
    }

    fn hak_payload_size(&self) -> Option<u64> {
        u64::try_from(self.payload.len()).ok()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HakPreflightPlanV1 {
    pub entry_count: u32,
    pub key_table_offset: u32,
    pub resource_table_offset: u32,
    pub payload_offset: u32,
    pub byte_length: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HakWriterLimitsV1 {
    pub max_entry_count: u64,
    pub max_output_bytes: u64,
}

impl Default for HakWriterLimitsV1 {
    fn default() -> Self {
        Self {
            max_entry_count: HAK_MAX_ENTRY_COUNT,
            max_output_bytes: HAK_MAX_OUTPUT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HakWriterOptionsV1 {
    pub schema_version: u32,
    pub limits: HakWriterLimitsV1,
}

impl Default for HakWriterOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: HAK_WRITER_SCHEMA_VERSION,
            limits: HakWriterLimitsV1::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HakResourceReportV1 {
    pub resref: String,
    pub resource_id: u32,
    pub resource_type: u16,
    pub payload_offset: u32,
    pub payload_size: u32,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HakWriterReportV1 {
    pub schema_version: u32,
    pub entry_count: u32,
    pub key_table_offset: u32,
    pub resource_table_offset: u32,
    pub payload_offset: u32,
    pub byte_length: u64,
    pub archive_sha256: String,
    pub resources: Vec<HakResourceReportV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HakArtifactV1 {
    pub payload: Vec<u8>,
    pub report: HakWriterReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HakWriteError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl HakWriteError {
    fn fatal(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: HAK_WRITER_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for HakWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for HakWriteError {}

pub fn write_hak_v1(
    resources: &[HakResourceInputV1],
    options: &HakWriterOptionsV1,
) -> Result<HakArtifactV1, HakWriteError> {
    write_hak_v1_inner(
        resources,
        options,
        #[cfg(test)]
        false,
    )
}

/// Validates options, limits, keys and the complete output layout using only
/// borrowed resource metadata. No resource payload is copied or materialized.
pub fn preflight_hak_v1<T: HakResourceMetadataV1>(
    resources: &[T],
    options: &HakWriterOptionsV1,
) -> Result<HakPreflightPlanV1, HakWriteError> {
    let (_, plan) = preflight_hak_layout(resources, options)?;
    Ok(HakPreflightPlanV1 {
        entry_count: plan.scalar.entry_count,
        key_table_offset: 0xa0,
        resource_table_offset: plan.scalar.resource_table_offset,
        payload_offset: plan.scalar.payload_offset,
        byte_length: plan.scalar.byte_length,
    })
}

fn write_hak_v1_inner(
    resources: &[HakResourceInputV1],
    options: &HakWriterOptionsV1,
    #[cfg(test)] force_output_allocation_failure: bool,
) -> Result<HakArtifactV1, HakWriteError> {
    let (sorted, plan) = preflight_hak_layout(resources, options)?;
    emit_hak_v1(
        &sorted,
        &plan,
        #[cfg(test)]
        force_output_allocation_failure,
    )
}

fn preflight_hak_layout<'a, T: HakResourceMetadataV1>(
    resources: &'a [T],
    options: &HakWriterOptionsV1,
) -> Result<(Vec<&'a T>, LayoutPlan), HakWriteError> {
    validate_options(options)?;
    let entry_count = u64::try_from(resources.len()).map_err(|_| {
        HakWriteError::fatal(
            ENTRY_LIMIT_EXCEEDED,
            "resources",
            "resource count does not fit u64",
        )
    })?;
    if entry_count > options.limits.max_entry_count {
        return Err(HakWriteError::fatal(
            ENTRY_LIMIT_EXCEEDED,
            "resources",
            format!(
                "resource count {entry_count} exceeds configured limit {}",
                options.limits.max_entry_count
            ),
        ));
    }

    for (index, resource) in resources.iter().enumerate() {
        if !valid_resref(resource.hak_resref()) {
            return Err(HakWriteError::fatal(
                RESREF_INVALID,
                &format!("resources[{index}].resref"),
                "resref must match lowercase [a-z0-9_]{1,16}",
            ));
        }
    }

    let mut keys = HashSet::new();
    keys.try_reserve(resources.len()).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve duplicate-key validation state",
        )
    })?;
    for (index, resource) in resources.iter().enumerate() {
        if !keys.insert((resource.hak_resref(), resource.hak_resource_type())) {
            return Err(HakWriteError::fatal(
                DUPLICATE_KEY,
                &format!("resources[{index}]"),
                "duplicate (resref, resourceType) key",
            ));
        }
    }

    let mut sorted = Vec::new();
    sorted.try_reserve_exact(resources.len()).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve sorted resource state",
        )
    })?;
    sorted.extend(resources.iter());
    sorted.sort_unstable_by(|left, right| {
        left.hak_resref()
            .as_bytes()
            .cmp(right.hak_resref().as_bytes())
            .then_with(|| left.hak_resource_type().cmp(&right.hak_resource_type()))
    });

    let total_payload_bytes = sorted.iter().try_fold(0_u64, |total, resource| {
        let length = resource.hak_payload_size().ok_or_else(|| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "payload length does not fit u64")
        })?;
        total.checked_add(length).ok_or_else(|| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "total payload length overflows u64")
        })
    })?;
    let scalar_plan = plan_scalar_layout(
        entry_count,
        total_payload_bytes,
        options.limits.max_output_bytes,
    )?;
    let plan = finish_layout_plan(
        scalar_plan,
        entry_count,
        sorted.iter().map(|resource| {
            resource.hak_payload_size().ok_or_else(|| {
                HakWriteError::fatal(U32_OVERFLOW, "layout", "payload length does not fit u64")
            })
        }),
        sorted.len(),
    )?;
    Ok((sorted, plan))
}

fn emit_hak_v1(
    sorted: &[&HakResourceInputV1],
    plan: &LayoutPlan,
    #[cfg(test)] force_output_allocation_failure: bool,
) -> Result<HakArtifactV1, HakWriteError> {
    let output_capacity = usize::try_from(plan.scalar.byte_length).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "HAK output length does not fit this platform",
        )
    })?;
    #[cfg(test)]
    if force_output_allocation_failure {
        return Err(HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "forced output allocation failure",
        ));
    }
    let mut payload = Vec::new();
    payload.try_reserve_exact(output_capacity).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve HAK output buffer",
        )
    })?;
    payload.resize(160, 0);
    payload[0..4].copy_from_slice(b"HAK ");
    payload[4..8].copy_from_slice(b"V1.0");
    write_u32(&mut payload, 0x10, plan.scalar.entry_count);
    write_u32(&mut payload, 0x14, 0xa0);
    write_u32(&mut payload, 0x18, 0xa0);
    write_u32(&mut payload, 0x1c, plan.scalar.resource_table_offset);
    write_u32(&mut payload, 0x28, u32::MAX);

    for (resource, resource_plan) in sorted.iter().zip(&plan.resources) {
        let resource_id = resource_plan.resource_id as usize;
        let key_offset = 160 + resource_id * 24;
        payload.resize(key_offset + 24, 0);
        payload[key_offset..key_offset + resource.resref.len()]
            .copy_from_slice(resource.resref.as_bytes());
        write_u32(&mut payload, key_offset + 16, resource_plan.resource_id);
        write_u16(&mut payload, key_offset + 20, resource.resource_type);
    }
    payload.resize(
        usize::try_from(plan.scalar.payload_offset).expect("planned payload offset fits usize"),
        0,
    );

    let mut reports = Vec::new();
    reports.try_reserve_exact(sorted.len()).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve HAK resource report",
        )
    })?;
    for (resource, resource_plan) in sorted.iter().zip(&plan.resources) {
        let resource_id = resource_plan.resource_id as usize;
        let descriptor_offset = usize::try_from(plan.scalar.resource_table_offset)
            .expect("planned resource table offset fits usize")
            + resource_id * 8;
        write_u32(
            &mut payload,
            descriptor_offset,
            resource_plan.payload_offset,
        );
        write_u32(
            &mut payload,
            descriptor_offset + 4,
            resource_plan.payload_size,
        );
        payload.extend_from_slice(&resource.payload);
        reports.push(HakResourceReportV1 {
            resref: clone_string_fallible(&resource.resref)?,
            resource_id: resource_plan.resource_id,
            resource_type: resource.resource_type,
            payload_offset: resource_plan.payload_offset,
            payload_size: resource_plan.payload_size,
            payload_sha256: sha256_hex(&resource.payload)?,
        });
    }

    let archive_sha256 = sha256_hex(&payload)?;
    let report = HakWriterReportV1 {
        schema_version: HAK_WRITER_SCHEMA_VERSION,
        entry_count: plan.scalar.entry_count,
        key_table_offset: 0xa0,
        resource_table_offset: plan.scalar.resource_table_offset,
        payload_offset: plan.scalar.payload_offset,
        byte_length: plan.scalar.byte_length,
        archive_sha256,
        resources: reports,
    };
    verify_exact_layout(&payload, sorted, plan)?;
    verify_semantic_readback(&payload, sorted, plan, &report)?;
    Ok(HakArtifactV1 { payload, report })
}

fn verify_exact_layout(
    bytes: &[u8],
    sorted: &[&HakResourceInputV1],
    plan: &LayoutPlan,
) -> Result<(), HakWriteError> {
    let scalar = plan.scalar;
    let expected_length = usize::try_from(scalar.byte_length)
        .map_err(|_| readback_failed("planned HAK byte length does not fit this platform"))?;
    if bytes.len() != expected_length {
        return Err(readback_failed(format!(
            "exact EOF mismatch: expected {expected_length} bytes, got {}",
            bytes.len()
        )));
    }
    if bytes.len() < 160 || sorted.len() != plan.resources.len() {
        return Err(readback_failed("internal HAK plan is inconsistent"));
    }

    if &bytes[0..4] != b"HAK " || &bytes[4..8] != b"V1.0" {
        return Err(readback_failed(
            "HAK identity fields changed after emission",
        ));
    }
    let header_fields = [
        (0x08, 0),
        (0x0c, 0),
        (0x10, scalar.entry_count),
        (0x14, 0xa0),
        (0x18, 0xa0),
        (0x1c, scalar.resource_table_offset),
        (0x20, 0),
        (0x24, 0),
        (0x28, u32::MAX),
    ];
    for (offset, expected) in header_fields {
        if read_u32(bytes, offset) != expected {
            return Err(readback_failed(format!(
                "HAK header field at 0x{offset:02x} changed after emission"
            )));
        }
    }
    if bytes[0x2c..0xa0].iter().any(|byte| *byte != 0) {
        return Err(readback_failed("HAK reserved header bytes must be zero"));
    }

    let mut previous_key: Option<(&[u8], u16)> = None;
    for (resource, resource_plan) in sorted.iter().zip(&plan.resources) {
        let resource_id = resource_plan.resource_id as usize;
        let key_offset = 160 + resource_id * 24;
        let key = &bytes[key_offset..key_offset + 24];
        let resref_bytes = resource.resref.as_bytes();
        if &key[..resref_bytes.len()] != resref_bytes
            || key[resref_bytes.len()..16].iter().any(|byte| *byte != 0)
        {
            return Err(readback_failed(format!(
                "resource key {resource_id} has invalid resref bytes or NUL padding"
            )));
        }
        let actual_id = read_u32(key, 16);
        let actual_type = read_u16(key, 20);
        if actual_id != resource_plan.resource_id {
            return Err(readback_failed(format!(
                "resource key {resource_id} has non-sequential id {actual_id}"
            )));
        }
        if actual_type != resource.resource_type {
            return Err(readback_failed(format!(
                "resource key {resource_id} has unexpected resource type"
            )));
        }
        if read_u16(key, 22) != 0 {
            return Err(readback_failed(format!(
                "resource key {resource_id} unused field must be zero"
            )));
        }
        if let Some((previous_resref, previous_type)) = previous_key {
            let order = previous_resref
                .cmp(&key[..16])
                .then_with(|| previous_type.cmp(&actual_type));
            if order != Ordering::Less {
                return Err(readback_failed("resource keys are not strictly sorted"));
            }
        }
        previous_key = Some((&key[..16], actual_type));
    }

    let expected_resource_offset = 160_u32
        .checked_add(scalar.entry_count.checked_mul(24).ok_or_else(|| {
            readback_failed("key table length overflowed during exact verification")
        })?)
        .ok_or_else(|| readback_failed("resource table offset overflowed"))?;
    if expected_resource_offset != scalar.resource_table_offset {
        return Err(readback_failed(
            "resource table is not contiguous with key table",
        ));
    }
    let expected_payload_offset = scalar
        .resource_table_offset
        .checked_add(scalar.entry_count.checked_mul(8).ok_or_else(|| {
            readback_failed("resource table length overflowed during exact verification")
        })?)
        .ok_or_else(|| readback_failed("payload offset overflowed"))?;
    if expected_payload_offset != scalar.payload_offset {
        return Err(readback_failed(
            "payload is not contiguous with resource table",
        ));
    }

    let resource_table_offset = scalar.resource_table_offset as usize;
    let mut payload_cursor = scalar.payload_offset;
    for ((_resource, resource_plan), resource_id) in
        sorted.iter().zip(&plan.resources).zip(0_usize..)
    {
        let descriptor_offset = resource_table_offset + resource_id * 8;
        let descriptor = &bytes[descriptor_offset..descriptor_offset + 8];
        let actual_offset = read_u32(descriptor, 0);
        let actual_size = read_u32(descriptor, 4);
        if actual_offset != payload_cursor || actual_size != resource_plan.payload_size {
            return Err(readback_failed(format!(
                "resource descriptor {resource_id} is not contiguous"
            )));
        }
        payload_cursor = payload_cursor
            .checked_add(resource_plan.payload_size)
            .ok_or_else(|| readback_failed("payload cursor overflowed"))?;
        if payload_cursor as usize > bytes.len() {
            return Err(readback_failed(format!(
                "resource payload {resource_id} extends past EOF"
            )));
        }
    }
    if u64::from(payload_cursor) != scalar.byte_length {
        return Err(readback_failed(
            "final payload cursor does not equal exact HAK EOF",
        ));
    }
    Ok(())
}

fn verify_semantic_readback(
    bytes: &[u8],
    sorted: &[&HakResourceInputV1],
    plan: &LayoutPlan,
    report: &HakWriterReportV1,
) -> Result<(), HakWriteError> {
    let scalar = plan.scalar;
    let archive = ErfArchive::parse_with_limits(
        bytes,
        ErfLimits {
            max_entry_count: sorted.len(),
        },
    )
    .map_err(|error| readback_failed(format!("ErfArchive rejected generated HAK: {error}")))?;
    if archive.file_type() != ErfFileType::Hak {
        return Err(semantic_diff("readback file type is not HAK"));
    }
    if archive.resources().len() != sorted.len() || report.resources.len() != sorted.len() {
        return Err(semantic_diff("readback resource count differs from input"));
    }
    if report.schema_version != HAK_WRITER_SCHEMA_VERSION
        || report.entry_count != scalar.entry_count
        || report.key_table_offset != 0xa0
        || report.resource_table_offset != scalar.resource_table_offset
        || report.payload_offset != scalar.payload_offset
        || report.byte_length != bytes.len() as u64
        || report.archive_sha256 != sha256_hex(bytes)?
    {
        return Err(semantic_diff("HAK report metadata or archive hash differs"));
    }

    for (((expected, actual), reported), resource_plan) in sorted
        .iter()
        .zip(archive.resources())
        .zip(&report.resources)
        .zip(&plan.resources)
    {
        let resource_id = resource_plan.resource_id as usize;
        let expected_offset = usize::try_from(reported.payload_offset)
            .map_err(|_| semantic_diff("reported payload offset does not fit usize"))?;
        let expected_size = expected.payload.len();
        if actual.resref != expected.resref
            || actual.resource_id != resource_plan.resource_id
            || actual.resource_type != expected.resource_type
            || actual.offset != expected_offset
            || actual.size != expected_size
            || reported.resref != expected.resref
            || reported.resource_id != resource_plan.resource_id
            || reported.resource_type != expected.resource_type
            || reported.payload_size as usize != expected_size
        {
            return Err(semantic_diff(format!(
                "resource {resource_id} metadata differs after readback"
            )));
        }
        let found = archive
            .find(&expected.resref, expected.resource_type)
            .map_err(|error| {
                semantic_diff(format!("resource {resource_id} lookup failed: {error}"))
            })?;
        let expected_hash = sha256_hex(&expected.payload)?;
        if found != expected.payload
            || sha256_hex(found)? != expected_hash
            || reported.payload_sha256 != expected_hash
        {
            return Err(semantic_diff(format!(
                "resource {resource_id} payload bytes or hash differ"
            )));
        }
    }
    Ok(())
}

fn readback_failed(message: impl Into<String>) -> HakWriteError {
    HakWriteError::fatal(READBACK_FAILED, "output", message)
}

fn semantic_diff(message: impl Into<String>) -> HakWriteError {
    HakWriteError::fatal(SEMANTIC_DIFF, "output", message)
}

fn validate_options(options: &HakWriterOptionsV1) -> Result<(), HakWriteError> {
    if options.schema_version != HAK_WRITER_SCHEMA_VERSION {
        return Err(HakWriteError::fatal(
            OPTIONS_INVALID,
            "options.schemaVersion",
            "schemaVersion must be 1",
        ));
    }
    if options.limits.max_entry_count > HAK_MAX_ENTRY_COUNT {
        return Err(HakWriteError::fatal(
            OPTIONS_INVALID,
            "options.limits.maxEntryCount",
            format!("maxEntryCount must be in 0..={HAK_MAX_ENTRY_COUNT}"),
        ));
    }
    if options.limits.max_output_bytes < 160
        || options.limits.max_output_bytes > HAK_MAX_OUTPUT_BYTES
    {
        return Err(HakWriteError::fatal(
            OPTIONS_INVALID,
            "options.limits.maxOutputBytes",
            format!("maxOutputBytes must be in 160..={HAK_MAX_OUTPUT_BYTES}"),
        ));
    }
    Ok(())
}

fn valid_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScalarLayoutPlan {
    entry_count: u32,
    resource_table_offset: u32,
    payload_offset: u32,
    byte_length: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResourceLayoutPlan {
    resource_id: u32,
    payload_offset: u32,
    payload_size: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LayoutPlan {
    scalar: ScalarLayoutPlan,
    resources: Vec<ResourceLayoutPlan>,
}

fn finish_layout_plan<I>(
    scalar: ScalarLayoutPlan,
    entry_count: u64,
    payload_lengths: I,
    length_count: usize,
) -> Result<LayoutPlan, HakWriteError>
where
    I: IntoIterator<Item = Result<u64, HakWriteError>>,
{
    let length_count_u64 = u64::try_from(length_count).map_err(|_| {
        HakWriteError::fatal(U32_OVERFLOW, "layout", "resource count does not fit u64")
    })?;
    if entry_count != u64::from(scalar.entry_count) || length_count_u64 != entry_count {
        return Err(HakWriteError::fatal(
            U32_OVERFLOW,
            "layout",
            "resource count differs from scalar layout plan",
        ));
    }

    let mut resources = Vec::new();
    resources.try_reserve_exact(length_count).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve resource layout plan",
        )
    })?;
    let mut payload_cursor = scalar.payload_offset;
    for (resource_index, payload_length) in payload_lengths.into_iter().enumerate() {
        if resource_index >= length_count {
            return Err(HakWriteError::fatal(
                U32_OVERFLOW,
                "layout",
                "resource length iterator exceeds planned count",
            ));
        }
        let resource_id = u32::try_from(resource_index)
            .map_err(|_| HakWriteError::fatal(U32_OVERFLOW, "layout", "resource id exceeds u32"))?;
        let payload_size = u32::try_from(payload_length?).map_err(|_| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "payload size exceeds u32")
        })?;
        resources.push(ResourceLayoutPlan {
            resource_id,
            payload_offset: payload_cursor,
            payload_size,
        });
        payload_cursor = payload_cursor.checked_add(payload_size).ok_or_else(|| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "payload cursor exceeds u32")
        })?;
    }
    if resources.len() != length_count {
        return Err(HakWriteError::fatal(
            U32_OVERFLOW,
            "layout",
            "resource length iterator ended before planned count",
        ));
    }
    if u64::from(payload_cursor) != scalar.byte_length {
        return Err(HakWriteError::fatal(
            U32_OVERFLOW,
            "layout",
            "final payload cursor differs from planned byte length",
        ));
    }
    Ok(LayoutPlan { scalar, resources })
}

fn plan_scalar_layout(
    entry_count: u64,
    total_payload_bytes: u64,
    max_output_bytes: u64,
) -> Result<ScalarLayoutPlan, HakWriteError> {
    let key_bytes = entry_count.checked_mul(24).ok_or_else(|| {
        HakWriteError::fatal(U32_OVERFLOW, "layout", "key table length overflows u64")
    })?;
    let table_bytes = entry_count.checked_mul(32).ok_or_else(|| {
        HakWriteError::fatal(
            U32_OVERFLOW,
            "layout",
            "combined table length overflows u64",
        )
    })?;
    plan_scalar_layout_from_lengths(
        entry_count,
        key_bytes,
        table_bytes,
        total_payload_bytes,
        max_output_bytes,
    )
}

fn plan_scalar_layout_from_lengths(
    entry_count: u64,
    key_bytes: u64,
    table_bytes: u64,
    total_payload_bytes: u64,
    max_output_bytes: u64,
) -> Result<ScalarLayoutPlan, HakWriteError> {
    let resource_table_offset = 160_u64.checked_add(key_bytes).ok_or_else(|| {
        HakWriteError::fatal(
            U32_OVERFLOW,
            "layout",
            "resource table offset overflows u64",
        )
    })?;
    let payload_offset = 160_u64.checked_add(table_bytes).ok_or_else(|| {
        HakWriteError::fatal(U32_OVERFLOW, "layout", "payload offset overflows u64")
    })?;
    let byte_length = payload_offset
        .checked_add(total_payload_bytes)
        .ok_or_else(|| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "HAK byte length overflows u64")
        })?;
    if byte_length > max_output_bytes {
        return Err(HakWriteError::fatal(
            OUTPUT_LIMIT_EXCEEDED,
            "options.limits.maxOutputBytes",
            format!(
                "HAK output requires {byte_length} bytes but configured limit is {max_output_bytes}"
            ),
        ));
    }
    Ok(ScalarLayoutPlan {
        entry_count: u32::try_from(entry_count)
            .map_err(|_| HakWriteError::fatal(U32_OVERFLOW, "layout", "entry count exceeds u32"))?,
        resource_table_offset: u32::try_from(resource_table_offset).map_err(|_| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "resource table offset exceeds u32")
        })?,
        payload_offset: u32::try_from(payload_offset).map_err(|_| {
            HakWriteError::fatal(U32_OVERFLOW, "layout", "payload offset exceeds u32")
        })?,
        byte_length,
    })
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("fixed u16 field"),
    )
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("fixed u32 field"),
    )
}

fn clone_string_fallible(value: &str) -> Result<String, HakWriteError> {
    let mut output = String::new();
    output.try_reserve_exact(value.len()).map_err(|_| {
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve HAK report resref",
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
        HakWriteError::fatal(
            ALLOCATION_FAILED,
            "output",
            "could not reserve SHA-256 report string",
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

    fn fixture_resources() -> Vec<HakResourceInputV1> {
        vec![
            HakResourceInputV1 {
                resref: "b".to_owned(),
                resource_type: 1,
                payload: vec![],
            },
            HakResourceInputV1 {
                resref: "a".to_owned(),
                resource_type: 3,
                payload: vec![1, 2, 3],
            },
            HakResourceInputV1 {
                resref: "abcdefghijklmnop".to_owned(),
                resource_type: 2,
                payload: vec![4],
            },
        ]
    }

    fn sorted_fixture(resources: &[HakResourceInputV1]) -> Vec<&HakResourceInputV1> {
        let mut sorted = resources.iter().collect::<Vec<_>>();
        sorted.sort_unstable_by(|left, right| {
            left.resref
                .as_bytes()
                .cmp(right.resref.as_bytes())
                .then_with(|| left.resource_type.cmp(&right.resource_type))
        });
        sorted
    }

    fn report_plan(report: &HakWriterReportV1) -> LayoutPlan {
        LayoutPlan {
            scalar: ScalarLayoutPlan {
                entry_count: report.entry_count,
                resource_table_offset: report.resource_table_offset,
                payload_offset: report.payload_offset,
                byte_length: report.byte_length,
            },
            resources: report
                .resources
                .iter()
                .map(|resource| ResourceLayoutPlan {
                    resource_id: resource.resource_id,
                    payload_offset: resource.payload_offset,
                    payload_size: resource.payload_size,
                })
                .collect(),
        }
    }

    fn run_private_gates(
        bytes: &[u8],
        sorted: &[&HakResourceInputV1],
        plan: &LayoutPlan,
        report: &HakWriterReportV1,
    ) -> Result<(), HakWriteError> {
        verify_exact_layout(bytes, sorted, plan)?;
        verify_semantic_readback(bytes, sorted, plan, report)
    }

    #[test]
    fn private_scalar_planner_preserves_output_limit_precedence_and_u32_overflow_seam() {
        let limit = plan_scalar_layout(u64::from(u32::MAX) + 1, 0, 160).unwrap_err();
        assert_eq!(limit.code, OUTPUT_LIMIT_EXCEEDED);

        let overflow = plan_scalar_layout(u64::from(u32::MAX) + 1, 0, u64::MAX).unwrap_err();
        assert_eq!(overflow.code, U32_OVERFLOW);
        assert_eq!(overflow.path, "layout");
    }

    #[test]
    fn private_planner_has_separate_checked_multiply_add_and_boundary_seams() {
        let key_multiply_boundary = plan_scalar_layout(u64::MAX / 24, 0, u64::MAX).unwrap_err();
        assert_eq!(key_multiply_boundary.code, U32_OVERFLOW);
        assert!(
            key_multiply_boundary
                .message
                .contains("combined table length")
        );
        let key_multiply_overflow = plan_scalar_layout(u64::MAX / 24 + 1, 0, u64::MAX).unwrap_err();
        assert_eq!(key_multiply_overflow.code, U32_OVERFLOW);
        assert!(key_multiply_overflow.message.contains("key table length"));

        let combined_multiply_boundary =
            plan_scalar_layout(u64::MAX / 32, 0, u64::MAX).unwrap_err();
        assert_eq!(combined_multiply_boundary.code, U32_OVERFLOW);
        assert!(
            combined_multiply_boundary
                .message
                .contains("payload offset")
        );
        let combined_multiply_overflow =
            plan_scalar_layout(u64::MAX / 32 + 1, 0, u64::MAX).unwrap_err();
        assert_eq!(combined_multiply_overflow.code, U32_OVERFLOW);
        assert!(
            combined_multiply_overflow
                .message
                .contains("combined table length")
        );

        let table_add = plan_scalar_layout_from_lengths(0, u64::MAX, 0, 0, u64::MAX).unwrap_err();
        assert_eq!(table_add.code, U32_OVERFLOW);
        assert!(table_add.message.contains("resource table offset"));
        let payload_add = plan_scalar_layout_from_lengths(0, 0, u64::MAX, 0, u64::MAX).unwrap_err();
        assert_eq!(payload_add.code, U32_OVERFLOW);
        assert!(payload_add.message.contains("payload offset"));
        let file_add = plan_scalar_layout_from_lengths(0, 0, 0, u64::MAX, u64::MAX).unwrap_err();
        assert_eq!(file_add.code, U32_OVERFLOW);
        assert!(file_add.message.contains("byte length"));
    }

    #[test]
    fn resource_u32_planner_seams_run_before_output_allocation_and_never_panic() {
        let result = catch_unwind(AssertUnwindSafe(|| {
            plan_scalar_layout(1, u64::from(u32::MAX) + 1, 160)
        }));
        assert_eq!(
            result
                .expect("output precedence seam must not panic")
                .unwrap_err()
                .code,
            OUTPUT_LIMIT_EXCEEDED
        );

        let scalar = plan_scalar_layout(1, u64::from(u32::MAX) + 1, u64::MAX).unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            finish_layout_plan(scalar, 1, [Ok(u64::from(u32::MAX) + 1)], 1)
        }));
        let error = result
            .expect("payload-size seam must not panic")
            .unwrap_err();
        assert_eq!(error.code, U32_OVERFLOW);
        assert!(error.message.contains("payload size"));

        let scalar = plan_scalar_layout(1, u64::from(u32::MAX), u64::MAX).unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            finish_layout_plan(scalar, 1, [Ok(u64::from(u32::MAX))], 1)
        }));
        let error = result
            .expect("payload-cursor seam must not panic")
            .unwrap_err();
        assert_eq!(error.code, U32_OVERFLOW);
        assert!(error.message.contains("payload cursor"));

        let scalar = plan_scalar_layout(1, 0, u64::MAX).unwrap();
        for (entry_count, lengths, length_count) in [
            (0, vec![Ok(0)], 1),
            (1, vec![], 1),
            (1, vec![Ok(0), Ok(0)], 1),
        ] {
            let result = catch_unwind(AssertUnwindSafe(|| {
                finish_layout_plan(scalar, entry_count, lengths, length_count)
            }));
            assert_eq!(
                result.expect("count seam must not panic").unwrap_err().code,
                U32_OVERFLOW
            );
        }

        let boundary_results = [
            catch_unwind(AssertUnwindSafe(|| {
                plan_scalar_layout(u64::MAX / 24 + 1, 0, u64::MAX)
            })),
            catch_unwind(AssertUnwindSafe(|| {
                plan_scalar_layout(u64::MAX / 32 + 1, 0, u64::MAX)
            })),
            catch_unwind(AssertUnwindSafe(|| {
                plan_scalar_layout_from_lengths(0, u64::MAX, 0, 0, u64::MAX)
            })),
            catch_unwind(AssertUnwindSafe(|| {
                plan_scalar_layout_from_lengths(0, 0, u64::MAX, 0, u64::MAX)
            })),
            catch_unwind(AssertUnwindSafe(|| {
                plan_scalar_layout_from_lengths(0, 0, 0, u64::MAX, u64::MAX)
            })),
        ];
        for result in boundary_results {
            assert!(
                result
                    .expect("planner arithmetic boundary must not panic")
                    .is_err()
            );
        }
    }

    #[test]
    fn deterministic_test_only_allocation_failure_seam_is_stable() {
        let error = write_hak_v1_inner(
            &[HakResourceInputV1 {
                resref: "a".to_owned(),
                resource_type: 1,
                payload: vec![1],
            }],
            &HakWriterOptionsV1::default(),
            true,
        )
        .unwrap_err();
        assert_eq!(error.code, ALLOCATION_FAILED);
        assert_eq!(error.path, "output");
    }

    #[test]
    fn exact_verifier_rejects_every_header_byte_and_truncated_prefix_without_panicking() {
        let resources = fixture_resources();
        let artifact = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap();
        let sorted = sorted_fixture(&resources);
        let plan = report_plan(&artifact.report);

        for offset in 0..160 {
            let mut mutated = artifact.payload.clone();
            mutated[offset] ^= 1;
            let result = catch_unwind(AssertUnwindSafe(|| {
                run_private_gates(&mutated, &sorted, &plan, &artifact.report)
            }));
            let error = result.expect("header mutation must not panic").unwrap_err();
            assert_eq!(error.code, READBACK_FAILED, "header byte {offset}");
            assert_eq!(error.path, "output");
        }

        for length in 0..artifact.payload.len() {
            let result = catch_unwind(AssertUnwindSafe(|| {
                run_private_gates(
                    &artifact.payload[..length],
                    &sorted,
                    &plan,
                    &artifact.report,
                )
            }));
            let error = result
                .expect("truncated generated HAK must not panic")
                .unwrap_err();
            assert_eq!(error.code, READBACK_FAILED, "prefix length {length}");
        }
    }

    #[test]
    fn exact_verifier_rejects_key_descriptor_gap_overlap_and_trailing_mutations() {
        let resources = fixture_resources();
        let artifact = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap();
        let sorted = sorted_fixture(&resources);
        let plan = report_plan(&artifact.report);
        let resource_table = plan.scalar.resource_table_offset as usize;

        let mutations: &[(usize, u8, &str)] = &[
            (160, b'z', "resref"),
            (161, 1, "resref NUL padding"),
            (176, 1, "resource id"),
            (180, 1, "resource type"),
            (182, 1, "key unused"),
            (resource_table, 1, "payload gap or overlap"),
            (resource_table + 4, 1, "payload size"),
        ];
        for &(offset, xor, label) in mutations {
            let mut mutated = artifact.payload.clone();
            mutated[offset] ^= xor;
            let error = run_private_gates(&mutated, &sorted, &plan, &artifact.report).unwrap_err();
            assert_eq!(error.code, READBACK_FAILED, "{label}");
        }

        let mut reordered = artifact.payload.clone();
        let first = reordered[160..184].to_vec();
        let second = reordered[184..208].to_vec();
        reordered[160..184].copy_from_slice(&second);
        reordered[184..208].copy_from_slice(&first);
        assert_eq!(
            run_private_gates(&reordered, &sorted, &plan, &artifact.report)
                .unwrap_err()
                .code,
            READBACK_FAILED
        );

        let mut metadata_overlap = artifact.payload.clone();
        metadata_overlap[resource_table..resource_table + 4]
            .copy_from_slice(&160_u32.to_le_bytes());
        assert_eq!(
            run_private_gates(&metadata_overlap, &sorted, &plan, &artifact.report)
                .unwrap_err()
                .code,
            READBACK_FAILED
        );

        let mut trailing = artifact.payload.clone();
        trailing.push(0);
        assert_eq!(
            run_private_gates(&trailing, &sorted, &plan, &artifact.report)
                .unwrap_err()
                .code,
            READBACK_FAILED
        );
    }

    #[test]
    fn parseable_payload_and_report_mutations_are_semantic_diff() {
        let resources = fixture_resources();
        let artifact = write_hak_v1(&resources, &HakWriterOptionsV1::default()).unwrap();
        let sorted = sorted_fixture(&resources);
        let plan = report_plan(&artifact.report);

        let mut payload_mutation = artifact.payload.clone();
        payload_mutation[plan.scalar.payload_offset as usize] ^= 1;
        assert_eq!(
            run_private_gates(&payload_mutation, &sorted, &plan, &artifact.report)
                .unwrap_err()
                .code,
            SEMANTIC_DIFF
        );

        let mut archive_hash = artifact.report.clone();
        archive_hash.archive_sha256.replace_range(0..1, "!");
        assert_eq!(
            run_private_gates(&artifact.payload, &sorted, &plan, &archive_hash)
                .unwrap_err()
                .code,
            SEMANTIC_DIFF
        );

        let mut payload_hash = artifact.report.clone();
        payload_hash.resources[0]
            .payload_sha256
            .replace_range(0..1, "!");
        assert_eq!(
            run_private_gates(&artifact.payload, &sorted, &plan, &payload_hash)
                .unwrap_err()
                .code,
            SEMANTIC_DIFF
        );

        for field in 0..6 {
            let mut metadata = artifact.report.clone();
            match field {
                0 => metadata.entry_count += 1,
                1 => metadata.resources[0].resref = "changed".to_owned(),
                2 => metadata.resources[0].resource_id += 1,
                3 => metadata.resources[0].resource_type += 1,
                4 => metadata.resources[0].payload_offset += 1,
                5 => metadata.resources[0].payload_size += 1,
                _ => unreachable!(),
            }
            assert_eq!(
                run_private_gates(&artifact.payload, &sorted, &plan, &metadata)
                    .unwrap_err()
                    .code,
                SEMANTIC_DIFF,
                "report metadata field {field}"
            );
        }
    }
}
