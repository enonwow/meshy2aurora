//! Pure byte-to-report primitives for reference proof packets.
//!
//! This module deliberately has no filesystem, clock, or environment access.
//! Callers supply already-read bytes and non-private execution metadata. The
//! packet builder hashes and parses the same borrowed byte slice so a report
//! cannot be paired with a different input.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::mdl::{InspectionReport, inspect_binary_mdl};

pub const REFERENCE_PROOF_SCHEMA_VERSION: u32 = 1;
const INSPECTION_REPORT_SCHEMA_VERSION: u32 = 1;
const MDL_RESOURCE_TYPE: u16 = 2002;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceManifest {
    pub schema_version: u32,
    pub entries: Vec<ReferenceManifestEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceManifestEntry {
    pub identity: ReferenceIdentity,
    pub expected_input: InputFingerprint,
    pub expected_capabilities: Vec<ReferenceCapability>,
    pub expected_invariants: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceIdentity {
    /// Stable corpus identifier such as `R0` or `R1`.
    pub reference_id: String,
    /// Logical source identity only; never a host path.
    pub source: ReferenceSource,
    pub resref: String,
    pub resource_type: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "class", rename_all = "snake_case")]
pub enum ReferenceSource {
    Synthetic,
    BaseNwn,
    NamedHak { name: String },
    DirectFile { label: String },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceProofPacket {
    pub schema_version: u32,
    pub packet_id: String,
    pub identity: ReferenceIdentity,
    pub input: InputFingerprint,
    pub execution: ExecutionMetadata,
    pub reader: ReaderIdentity,
    pub reader_report: InspectionReport,
    pub capability_results: Vec<CapabilityResult>,
    pub invariant_results: Vec<InvariantResult>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionMetadata {
    /// A logical operation label, not a shell command or executable path.
    pub command_label: String,
    /// UTC instant in the intentionally narrow `YYYY-MM-DDTHH:MM:SSZ` form.
    pub timestamp_utc: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceProofError {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ReferenceProofError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ReferenceProofError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFingerprint {
    pub algorithm: HashAlgorithm,
    pub sha256: String,
    pub byte_length: usize,
}

impl InputFingerprint {
    #[must_use]
    pub fn from_bytes(input: &[u8]) -> Self {
        let digest = Sha256::digest(input);
        let sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();

        Self {
            algorithm: HashAlgorithm::Sha256,
            sha256,
            byte_length: input.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum HashAlgorithm {
    #[serde(rename = "SHA-256")]
    Sha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderIdentity {
    pub name: String,
    pub version: String,
    pub report_schema_version: u32,
}

impl ReaderIdentity {
    fn current_mdl() -> Self {
        Self {
            name: "m2a-core::mdl".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            report_schema_version: INSPECTION_REPORT_SCHEMA_VERSION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceCapability {
    Header,
    CoreRanges,
    NodeTree,
    Mesh,
    Skin,
    Controllers,
    Animations,
    Events,
    UnsupportedNodeFamily,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityResult {
    pub capability: ReferenceCapability,
    pub status: CapabilityStatus,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityStatus {
    Pass,
    Unsupported,
    NotPresent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvariantResult {
    pub invariant: String,
    pub status: InvariantStatus,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvariantStatus {
    Pass,
    Fail,
    NotEvaluated,
}

/// Builds one proof packet from a validated manifest entry.
///
/// The builder computes the fingerprint and invokes the product reader itself.
/// Consequently there is no API surface on which a caller can submit a report
/// produced from bytes other than `input`.
pub fn build_reference_proof_packet(
    manifest: &ReferenceManifest,
    reference_id: &str,
    input: &[u8],
    execution: ExecutionMetadata,
    capability_results: Vec<CapabilityResult>,
    invariant_results: Vec<InvariantResult>,
) -> Result<ReferenceProofPacket, ReferenceProofError> {
    validate_manifest(manifest)?;
    validate_execution(&execution)?;
    if !is_logical_label(reference_id, 64, false) {
        return Err(error(
            "M2A-PREF-IDENTITY-INVALID",
            "selected reference id must be a non-private logical label",
        ));
    }

    let entry = manifest
        .entries
        .iter()
        .find(|entry| entry.identity.reference_id == reference_id)
        .ok_or_else(|| {
            error(
                "M2A-PREF-MANIFEST-ENTRY-NOT-FOUND",
                format!("manifest has no entry for logical reference id {reference_id:?}"),
            )
        })?;

    validate_results(entry, &capability_results, &invariant_results)?;

    let input_fingerprint = InputFingerprint::from_bytes(input);
    if input_fingerprint != entry.expected_input {
        return Err(error(
            "M2A-PREF-INPUT-FINGERPRINT-MISMATCH",
            format!(
                "input SHA-256/length does not match manifest entry {}",
                entry.identity.reference_id
            ),
        ));
    }

    // This is deliberately invoked here, after hashing the exact same immutable
    // slice. Supplying a separately-built reader report is not supported.
    let reader_report = inspect_binary_mdl(input).map_err(|parse_error| {
        error(
            "M2A-PREF-READER-FAILED",
            format!(
                "own reader rejected manifest-bound input with {} at offset {}",
                parse_error.code, parse_error.offset
            ),
        )
    })?;
    if reader_report.schema_version != INSPECTION_REPORT_SCHEMA_VERSION {
        return Err(error(
            "M2A-PREF-READER-SCHEMA-UNSUPPORTED",
            format!(
                "reader report schema {} is unsupported; expected {}",
                reader_report.schema_version, INSPECTION_REPORT_SCHEMA_VERSION
            ),
        ));
    }
    if reader_report.byte_length != input_fingerprint.byte_length {
        return Err(error(
            "M2A-PREF-INPUT-REPORT-MISMATCH",
            "own reader report length differs from the manifest-bound input",
        ));
    }

    let identity = entry.identity.clone();
    let packet_id = format!("P-REF-{}", identity.reference_id);

    Ok(ReferenceProofPacket {
        schema_version: REFERENCE_PROOF_SCHEMA_VERSION,
        packet_id,
        identity,
        input: input_fingerprint,
        execution,
        reader: ReaderIdentity::current_mdl(),
        reader_report,
        capability_results,
        invariant_results,
    })
}

fn validate_manifest(manifest: &ReferenceManifest) -> Result<(), ReferenceProofError> {
    if manifest.schema_version != REFERENCE_PROOF_SCHEMA_VERSION {
        return Err(error(
            "M2A-PREF-SCHEMA-UNSUPPORTED",
            format!(
                "manifest schema {} is unsupported; expected {}",
                manifest.schema_version, REFERENCE_PROOF_SCHEMA_VERSION
            ),
        ));
    }
    if manifest.entries.is_empty() {
        return Err(error(
            "M2A-PREF-MANIFEST-EMPTY",
            "reference manifest must contain at least one entry",
        ));
    }

    let mut reference_ids = BTreeSet::new();
    for entry in &manifest.entries {
        validate_identity(&entry.identity)?;
        if !reference_ids.insert(entry.identity.reference_id.as_str()) {
            return Err(error(
                "M2A-PREF-MANIFEST-IDENTITY-DUPLICATE",
                format!(
                    "duplicate logical reference id {:?}",
                    entry.identity.reference_id
                ),
            ));
        }
        validate_expected_fingerprint(&entry.expected_input)?;
        validate_manifest_expectations(entry)?;
    }

    Ok(())
}

fn validate_expected_fingerprint(
    fingerprint: &InputFingerprint,
) -> Result<(), ReferenceProofError> {
    let valid_sha256 = fingerprint.sha256.len() == 64
        && fingerprint
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid_sha256 {
        return Err(error(
            "M2A-PREF-MANIFEST-FINGERPRINT-INVALID",
            "expected SHA-256 must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    Ok(())
}

fn validate_manifest_expectations(
    entry: &ReferenceManifestEntry,
) -> Result<(), ReferenceProofError> {
    if entry.expected_capabilities.is_empty() {
        return Err(error(
            "M2A-PREF-MANIFEST-CAPABILITIES-EMPTY",
            "manifest entry must declare at least one expected capability",
        ));
    }
    if collect_unique_capabilities(&entry.expected_capabilities).is_none() {
        return Err(error(
            "M2A-PREF-MANIFEST-CAPABILITY-DUPLICATE",
            "manifest entry contains a duplicate expected capability",
        ));
    }
    if entry.expected_invariants.is_empty() {
        return Err(error(
            "M2A-PREF-MANIFEST-INVARIANTS-EMPTY",
            "manifest entry must declare at least one expected invariant",
        ));
    }
    let mut names = BTreeSet::new();
    for invariant in &entry.expected_invariants {
        if !is_logical_label(invariant, 128, true) {
            return Err(error(
                "M2A-PREF-MANIFEST-INVARIANT-INVALID",
                format!("invalid logical invariant name {invariant:?}"),
            ));
        }
        if !names.insert(invariant.as_str()) {
            return Err(error(
                "M2A-PREF-MANIFEST-INVARIANT-DUPLICATE",
                format!("duplicate expected invariant {invariant:?}"),
            ));
        }
    }
    Ok(())
}

fn validate_results(
    entry: &ReferenceManifestEntry,
    capability_results: &[CapabilityResult],
    invariant_results: &[InvariantResult],
) -> Result<(), ReferenceProofError> {
    if capability_results.is_empty() {
        return Err(error(
            "M2A-PREF-CAPABILITIES-EMPTY",
            "proof packet must contain capability results",
        ));
    }
    let actual_capabilities: Vec<_> = capability_results
        .iter()
        .map(|result| result.capability)
        .collect();
    let Some(actual_capabilities) = collect_unique_capabilities(&actual_capabilities) else {
        return Err(error(
            "M2A-PREF-CAPABILITY-DUPLICATE",
            "proof packet contains a duplicate capability result",
        ));
    };
    let expected_capabilities = collect_unique_capabilities(&entry.expected_capabilities)
        .expect("validated manifest capabilities must be unique");
    if actual_capabilities != expected_capabilities {
        return Err(error(
            "M2A-PREF-CAPABILITY-COVERAGE-MISMATCH",
            "capability results must cover exactly the manifest expectations",
        ));
    }

    if invariant_results.is_empty() {
        return Err(error(
            "M2A-PREF-INVARIANTS-EMPTY",
            "proof packet must contain invariant results",
        ));
    }
    let mut actual_invariants = BTreeSet::new();
    for result in invariant_results {
        if !is_logical_label(&result.invariant, 128, true) {
            return Err(error(
                "M2A-PREF-INVARIANT-INVALID",
                format!("invalid logical invariant name {:?}", result.invariant),
            ));
        }
        if !actual_invariants.insert(result.invariant.as_str()) {
            return Err(error(
                "M2A-PREF-INVARIANT-DUPLICATE",
                format!("duplicate invariant result {:?}", result.invariant),
            ));
        }
    }
    let expected_invariants: BTreeSet<_> = entry
        .expected_invariants
        .iter()
        .map(String::as_str)
        .collect();
    if actual_invariants != expected_invariants {
        return Err(error(
            "M2A-PREF-INVARIANT-COVERAGE-MISMATCH",
            "invariant results must cover exactly the manifest expectations",
        ));
    }

    Ok(())
}

fn collect_unique_capabilities(
    capabilities: &[ReferenceCapability],
) -> Option<BTreeSet<ReferenceCapability>> {
    let values: BTreeSet<_> = capabilities.iter().copied().collect();
    (values.len() == capabilities.len()).then_some(values)
}

fn validate_identity(identity: &ReferenceIdentity) -> Result<(), ReferenceProofError> {
    let valid_reference_id = is_logical_label(&identity.reference_id, 64, false);
    let valid_resref = is_resref(&identity.resref);
    let valid_source = match &identity.source {
        ReferenceSource::Synthetic | ReferenceSource::BaseNwn => true,
        ReferenceSource::NamedHak { name } => is_logical_label(name, 128, true),
        ReferenceSource::DirectFile { label } => is_logical_label(label, 128, true),
    };

    if !(valid_reference_id && valid_resref && valid_source) {
        return Err(error(
            "M2A-PREF-IDENTITY-INVALID",
            "identity fields must be non-private logical labels without path separators",
        ));
    }
    if identity.resource_type != MDL_RESOURCE_TYPE {
        return Err(error(
            "M2A-PREF-RESOURCE-TYPE-UNSUPPORTED",
            format!(
                "MDL reference proof requires resource type {MDL_RESOURCE_TYPE}, got {}",
                identity.resource_type
            ),
        ));
    }
    Ok(())
}

fn validate_execution(execution: &ExecutionMetadata) -> Result<(), ReferenceProofError> {
    if !is_logical_label(&execution.command_label, 128, true) {
        return Err(error(
            "M2A-PREF-COMMAND-LABEL-INVALID",
            "command label must be a logical operation label, not a command or path",
        ));
    }
    if !is_utc_timestamp(&execution.timestamp_utc) {
        return Err(error(
            "M2A-PREF-TIMESTAMP-INVALID",
            "timestamp must be a valid UTC instant in YYYY-MM-DDTHH:MM:SSZ form",
        ));
    }
    Ok(())
}

fn is_logical_label(value: &str, max_len: usize, allow_dot: bool) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && !value.contains("..")
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || byte == b'_'
                || byte == b'-'
                || (allow_dot && byte == b'.')
        })
}

fn is_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    let digit_ranges = [0..4, 5..7, 8..10, 11..13, 14..16, 17..19];
    if digit_ranges
        .iter()
        .any(|range| !bytes[range.clone()].iter().all(u8::is_ascii_digit))
    {
        return false;
    }

    let number = |start: usize, end: usize| -> u32 {
        bytes[start..end]
            .iter()
            .fold(0, |value, byte| value * 10 + u32::from(byte - b'0'))
    };
    let year = number(0, 4);
    let month = number(5, 7);
    let day = number(8, 10);
    let hour = number(11, 13);
    let minute = number(14, 16);
    let second = number(17, 19);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap_year =
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_month = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days_in_month).contains(&day)
}

fn error(code: &'static str, message: impl Into<String>) -> ReferenceProofError {
    ReferenceProofError {
        code,
        message: message.into(),
    }
}
