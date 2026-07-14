//! M7 original-Meshy corpus contract and deferred-input intake.
//!
//! M7-V1 and M7-V2 intentionally work before the real corpus is supplied.
//! Missing files stay `INPUT_DEFERRED`; they are never replaced by fixtures
//! and never allow an M7 completion claim.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::glb::{GlbLimits, ingest_glb};

pub const M7_CORPUS_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum M7CorpusRoleV1 {
    RiggedHumanoidSourceClips,
    NonHumanoidReferenceSupermodel,
    StaticPlaceableOrItem,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum M7StaticResourceKindV1 {
    Placeable,
    Item,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum M7SourceProviderV1 {
    Meshy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct M7ByteIdentityV1 {
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct M7OriginalSourceProvenanceV1 {
    pub provider: M7SourceProviderV1,
    pub provider_task_id: String,
    pub original_export_attested: bool,
    pub rights_confirmed: bool,
    pub not_synthetic_fixture_attested: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct M7SourceDescriptorV1 {
    pub relative_path: String,
    pub identity: M7ByteIdentityV1,
    pub provenance: M7OriginalSourceProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum M7CorpusEntryV1 {
    RiggedHumanoidSourceClips {
        sample_id: String,
        source: Option<M7SourceDescriptorV1>,
        required_source_clip_names: Vec<String>,
    },
    NonHumanoidReferenceSupermodel {
        sample_id: String,
        source: Option<M7SourceDescriptorV1>,
        reference_supermodel: String,
    },
    StaticPlaceableOrItem {
        sample_id: String,
        source: Option<M7SourceDescriptorV1>,
        resource_kind: M7StaticResourceKindV1,
    },
}

impl M7CorpusEntryV1 {
    pub fn role(&self) -> M7CorpusRoleV1 {
        match self {
            Self::RiggedHumanoidSourceClips { .. } => {
                M7CorpusRoleV1::RiggedHumanoidSourceClips
            }
            Self::NonHumanoidReferenceSupermodel { .. } => {
                M7CorpusRoleV1::NonHumanoidReferenceSupermodel
            }
            Self::StaticPlaceableOrItem { .. } => M7CorpusRoleV1::StaticPlaceableOrItem,
        }
    }

    pub fn sample_id(&self) -> &str {
        match self {
            Self::RiggedHumanoidSourceClips { sample_id, .. }
            | Self::NonHumanoidReferenceSupermodel { sample_id, .. }
            | Self::StaticPlaceableOrItem { sample_id, .. } => sample_id,
        }
    }

    pub fn source(&self) -> Option<&M7SourceDescriptorV1> {
        match self {
            Self::RiggedHumanoidSourceClips { source, .. }
            | Self::NonHumanoidReferenceSupermodel { source, .. }
            | Self::StaticPlaceableOrItem { source, .. } => source.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct M7CorpusManifestV1 {
    pub schema_version: u32,
    pub corpus_id: String,
    pub art_direction_approval_id: Option<String>,
    pub samples: Vec<M7CorpusEntryV1>,
}

#[derive(Clone, Copy, Debug)]
pub struct M7SourcePayloadV1<'a> {
    pub relative_path: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum M7IntakeStatusV1 {
    InputDeferred,
    InputInvalid,
    ReadyForM7V5,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum M7IntakeSeverityV1 {
    Deferred,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M7IntakeDiagnosticV1 {
    pub code: String,
    pub severity: M7IntakeSeverityV1,
    pub path: String,
    pub sample_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M7IntakeSampleReportV1 {
    pub sample_id: String,
    pub role: M7CorpusRoleV1,
    pub status: M7IntakeStatusV1,
    pub source_path: Option<String>,
    pub declared_identity: Option<M7ByteIdentityV1>,
    pub observed_identity: Option<M7ByteIdentityV1>,
    pub mesh_count: usize,
    pub primitive_count: usize,
    pub triangle_count: usize,
    pub skin_count: usize,
    pub animation_count: usize,
    pub profile_evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M7CorpusIntakeReportV1 {
    pub schema_version: u32,
    pub corpus_id: String,
    pub status: M7IntakeStatusV1,
    pub ready_source_count: usize,
    pub required_source_count: usize,
    pub real_execution_ready: bool,
    pub m7_done_claim_allowed: bool,
    pub samples: Vec<M7IntakeSampleReportV1>,
    pub diagnostics: Vec<M7IntakeDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M7CorpusContractErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for M7CorpusContractErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at {}: {}", self.code, self.path, self.message)
    }
}

impl std::error::Error for M7CorpusContractErrorV1 {}

pub fn parse_m7_corpus_manifest_v1(
    bytes: &[u8],
) -> Result<M7CorpusManifestV1, M7CorpusContractErrorV1> {
    let manifest = serde_json::from_slice(bytes).map_err(|error| {
        contract_error(
            "M7-MANIFEST-JSON-INVALID",
            "manifest",
            format!("manifest is not valid M7 corpus JSON: {error}"),
        )
    })?;
    validate_m7_corpus_manifest_v1(&manifest)?;
    Ok(manifest)
}

pub fn validate_m7_corpus_manifest_v1(
    manifest: &M7CorpusManifestV1,
) -> Result<(), M7CorpusContractErrorV1> {
    if manifest.schema_version != M7_CORPUS_SCHEMA_VERSION {
        return Err(contract_error(
            "M7-MANIFEST-SCHEMA-UNSUPPORTED",
            "manifest.schemaVersion",
            format!(
                "expected schema {}, got {}",
                M7_CORPUS_SCHEMA_VERSION, manifest.schema_version
            ),
        ));
    }
    validate_slug(&manifest.corpus_id, "manifest.corpusId")?;
    if let Some(approval_id) = &manifest.art_direction_approval_id {
        if approval_id.trim().is_empty()
            || approval_id.len() > 128
            || approval_id.chars().any(char::is_control)
        {
            return Err(contract_error(
                "M7-ART-DIRECTION-APPROVAL-ID-INVALID",
                "manifest.artDirectionApprovalId",
                "approval id must be 1..128 non-control characters when present",
            ));
        }
    }
    if manifest.samples.len() != 3 {
        return Err(contract_error(
            "M7-CORPUS-CARDINALITY-INVALID",
            "manifest.samples",
            format!(
                "M7 requires exactly three role entries, got {}",
                manifest.samples.len()
            ),
        ));
    }

    let mut roles = BTreeSet::new();
    let mut sample_ids = BTreeSet::new();
    let mut source_paths = BTreeSet::new();
    for (index, entry) in manifest.samples.iter().enumerate() {
        let base = format!("manifest.samples[{index}]");
        if !roles.insert(entry.role()) {
            return Err(contract_error(
                "M7-CORPUS-ROLE-DUPLICATE",
                format!("{base}.role"),
                "each M7 corpus role must occur exactly once",
            ));
        }
        validate_slug(entry.sample_id(), &format!("{base}.sampleId"))?;
        if !sample_ids.insert(entry.sample_id()) {
            return Err(contract_error(
                "M7-SAMPLE-ID-DUPLICATE",
                format!("{base}.sampleId"),
                "sample ids must be unique",
            ));
        }
        if let Some(source) = entry.source() {
            validate_source_descriptor(source, &base)?;
            if !source_paths.insert(source.relative_path.to_ascii_lowercase()) {
                return Err(contract_error(
                    "M7-SOURCE-PATH-DUPLICATE",
                    format!("{base}.source.relativePath"),
                    "source paths must be unique under case-insensitive Windows semantics",
                ));
            }
        }
        match entry {
            M7CorpusEntryV1::RiggedHumanoidSourceClips {
                required_source_clip_names,
                ..
            } => validate_clip_names(required_source_clip_names, &base)?,
            M7CorpusEntryV1::NonHumanoidReferenceSupermodel {
                reference_supermodel,
                ..
            } => validate_resref(reference_supermodel, &format!("{base}.referenceSupermodel"))?,
            M7CorpusEntryV1::StaticPlaceableOrItem { .. } => {}
        }
    }
    Ok(())
}

pub fn inspect_m7_corpus_intake_v1(
    manifest: &M7CorpusManifestV1,
    payloads: &[M7SourcePayloadV1<'_>],
) -> Result<M7CorpusIntakeReportV1, M7CorpusContractErrorV1> {
    validate_m7_corpus_manifest_v1(manifest)?;

    let mut payload_by_path = BTreeMap::new();
    for (index, payload) in payloads.iter().enumerate() {
        validate_relative_glb_path(
            payload.relative_path,
            &format!("payloads[{index}].relativePath"),
        )?;
        if payload_by_path
            .insert(payload.relative_path.to_ascii_lowercase(), payload.bytes)
            .is_some()
        {
            return Err(contract_error(
                "M7-INTAKE-PAYLOAD-DUPLICATE",
                format!("payloads[{index}].relativePath"),
                "the same case-insensitive source path was supplied more than once",
            ));
        }
    }

    let declared_paths = manifest
        .samples
        .iter()
        .filter_map(M7CorpusEntryV1::source)
        .map(|source| source.relative_path.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    let mut any_invalid = false;
    let mut any_deferred = false;
    if manifest.art_direction_approval_id.is_none() {
        any_deferred = true;
        diagnostics.push(diagnostic(
            "M7-ART-DIRECTION-INPUT-DEFERRED",
            M7IntakeSeverityV1::Deferred,
            "manifest.artDirectionApprovalId",
            None,
            "art direction approval is deferred until M7-V5",
        ));
    }
    for path in payload_by_path.keys() {
        if !declared_paths.contains(path) {
            any_invalid = true;
            diagnostics.push(diagnostic(
                "M7-INTAKE-UNDECLARED-PAYLOAD",
                M7IntakeSeverityV1::Error,
                "payloads",
                None,
                format!("payload {path:?} is not declared by any corpus entry"),
            ));
        }
    }

    let mut entries = manifest.samples.iter().enumerate().collect::<Vec<_>>();
    entries.sort_by_key(|(_, entry)| entry.role());
    let mut reports = Vec::with_capacity(3);
    let mut ready_source_count = 0_usize;
    for (manifest_index, entry) in entries {
        let base = format!("manifest.samples[{manifest_index}]");
        let Some(source) = entry.source() else {
            any_deferred = true;
            diagnostics.push(diagnostic(
                "M7-SOURCE-DESCRIPTOR-INPUT-DEFERRED",
                M7IntakeSeverityV1::Deferred,
                format!("{base}.source"),
                Some(entry.sample_id()),
                "original Meshy source descriptor is deferred until M7-V5",
            ));
            reports.push(empty_sample_report(
                entry,
                M7IntakeStatusV1::InputDeferred,
                None,
                "ORIGINAL_SOURCE_DESCRIPTOR_DEFERRED",
            ));
            continue;
        };
        let Some(bytes) = payload_by_path
            .get(&source.relative_path.to_ascii_lowercase())
            .copied()
        else {
            any_deferred = true;
            diagnostics.push(diagnostic(
                "M7-SOURCE-PAYLOAD-INPUT-DEFERRED",
                M7IntakeSeverityV1::Deferred,
                format!("{base}.source"),
                Some(entry.sample_id()),
                "manifest-bound original Meshy GLB has not been supplied",
            ));
            reports.push(empty_sample_report(
                entry,
                M7IntakeStatusV1::InputDeferred,
                Some(source),
                "ORIGINAL_SOURCE_PAYLOAD_DEFERRED",
            ));
            continue;
        };

        let observed_identity = M7ByteIdentityV1 {
            byte_length: u64::try_from(bytes.len()).map_err(|_| {
                contract_error(
                    "M7-SOURCE-LENGTH-OVERFLOW",
                    format!("{base}.source"),
                    "source byte length does not fit the M7 identity contract",
                )
            })?,
            sha256: hex_sha256(bytes),
        };
        if observed_identity != source.identity {
            any_invalid = true;
            diagnostics.push(diagnostic(
                "M7-SOURCE-IDENTITY-MISMATCH",
                M7IntakeSeverityV1::Error,
                format!("{base}.source.identity"),
                Some(entry.sample_id()),
                "source bytes differ from the manifest-bound length or SHA-256",
            ));
            let mut report = empty_sample_report(
                entry,
                M7IntakeStatusV1::InputInvalid,
                Some(source),
                "SOURCE_IDENTITY_MISMATCH",
            );
            report.observed_identity = Some(observed_identity);
            reports.push(report);
            continue;
        }

        let ingest = match ingest_glb(bytes, &GlbLimits::default()) {
            Ok(ingest) => ingest,
            Err(error) => {
                any_invalid = true;
                diagnostics.push(diagnostic(
                    "M7-SOURCE-GLB-INVALID",
                    M7IntakeSeverityV1::Error,
                    format!("{base}.source"),
                    Some(entry.sample_id()),
                    format!("{}: {}", error.code, error.message),
                ));
                let mut report = empty_sample_report(
                    entry,
                    M7IntakeStatusV1::InputInvalid,
                    Some(source),
                    "GLB_INGEST_REJECTED",
                );
                report.observed_identity = Some(observed_identity);
                reports.push(report);
                continue;
            }
        };

        let mut sample_invalid = false;
        if !ingest.report.conversion_eligible {
            sample_invalid = true;
            diagnostics.push(diagnostic(
                "M7-SOURCE-INGEST-GATES-FAILED",
                M7IntakeSeverityV1::Error,
                format!("{base}.source"),
                Some(entry.sample_id()),
                "canonical GLB ingest did not mark the source conversion-eligible",
            ));
        }
        if ingest.ir.meshes.is_empty() || ingest.ir.primitives.is_empty() {
            sample_invalid = true;
            diagnostics.push(diagnostic(
                "M7-SOURCE-GEOMETRY-MISSING",
                M7IntakeSeverityV1::Error,
                format!("{base}.source"),
                Some(entry.sample_id()),
                "source must contain at least one mesh and primitive",
            ));
        }
        let profile_evidence = match entry {
            M7CorpusEntryV1::RiggedHumanoidSourceClips {
                required_source_clip_names,
                ..
            } => {
                if ingest.ir.skins.is_empty() || ingest.ir.animations.is_empty() {
                    sample_invalid = true;
                    diagnostics.push(diagnostic(
                        "M7-HUMANOID-RIG-OR-CLIPS-MISSING",
                        M7IntakeSeverityV1::Error,
                        format!("{base}.source"),
                        Some(entry.sample_id()),
                        "rigged humanoid role requires a skin and source animations",
                    ));
                }
                for clip_name in required_source_clip_names {
                    if !ingest
                        .ir
                        .animations
                        .iter()
                        .any(|animation| animation.name.as_deref() == Some(clip_name))
                    {
                        sample_invalid = true;
                        diagnostics.push(diagnostic(
                            "M7-HUMANOID-REQUIRED-CLIP-MISSING",
                            M7IntakeSeverityV1::Error,
                            format!("{base}.source.animations"),
                            Some(entry.sample_id()),
                            format!("required source clip {clip_name:?} is absent"),
                        ));
                    }
                }
                "SOURCE_SKIN_AND_REQUIRED_CLIPS_PRESENT"
            }
            M7CorpusEntryV1::NonHumanoidReferenceSupermodel { .. } => {
                if !ingest.ir.skins.is_empty() || !ingest.ir.animations.is_empty() {
                    sample_invalid = true;
                    diagnostics.push(diagnostic(
                        "M7-REFERENCE-SUPERMODEL-SOURCE-NOT-STATIC",
                        M7IntakeSeverityV1::Error,
                        format!("{base}.source"),
                        Some(entry.sample_id()),
                        "reference-supermodel route requires an unrigged source without source clips",
                    ));
                }
                "UNRIGGED_SOURCE_BOUND_TO_REFERENCE_SUPERMODEL_ROUTE"
            }
            M7CorpusEntryV1::StaticPlaceableOrItem { .. } => {
                if !ingest.ir.skins.is_empty() || !ingest.ir.animations.is_empty() {
                    sample_invalid = true;
                    diagnostics.push(diagnostic(
                        "M7-STATIC-SOURCE-HAS-SKELETON-OR-ANIMATION",
                        M7IntakeSeverityV1::Error,
                        format!("{base}.source"),
                        Some(entry.sample_id()),
                        "static placeable/item source must not contain skins or animations",
                    ));
                }
                "NO_SKIN_OR_ANIMATION_PRESENT"
            }
        };
        let status = if sample_invalid {
            any_invalid = true;
            M7IntakeStatusV1::InputInvalid
        } else {
            ready_source_count += 1;
            M7IntakeStatusV1::ReadyForM7V5
        };
        reports.push(M7IntakeSampleReportV1 {
            sample_id: entry.sample_id().to_owned(),
            role: entry.role(),
            status,
            source_path: Some(source.relative_path.clone()),
            declared_identity: Some(source.identity.clone()),
            observed_identity: Some(observed_identity),
            mesh_count: ingest.report.inventory.mesh_count,
            primitive_count: ingest.report.inventory.primitive_count,
            triangle_count: ingest.report.statistics.triangle_count,
            skin_count: ingest.report.inventory.skin_count,
            animation_count: ingest.report.inventory.animation_count,
            profile_evidence: profile_evidence.to_owned(),
        });
    }

    let status = if any_invalid {
        M7IntakeStatusV1::InputInvalid
    } else if any_deferred || ready_source_count != 3 {
        M7IntakeStatusV1::InputDeferred
    } else {
        M7IntakeStatusV1::ReadyForM7V5
    };
    let real_execution_ready = status == M7IntakeStatusV1::ReadyForM7V5;
    Ok(M7CorpusIntakeReportV1 {
        schema_version: M7_CORPUS_SCHEMA_VERSION,
        corpus_id: manifest.corpus_id.clone(),
        status,
        ready_source_count,
        required_source_count: 3,
        real_execution_ready,
        m7_done_claim_allowed: false,
        samples: reports,
        diagnostics,
    })
}

fn validate_source_descriptor(
    source: &M7SourceDescriptorV1,
    base: &str,
) -> Result<(), M7CorpusContractErrorV1> {
    validate_relative_glb_path(
        &source.relative_path,
        &format!("{base}.source.relativePath"),
    )?;
    if source.identity.byte_length == 0 || !is_lower_hex_sha256(&source.identity.sha256) {
        return Err(contract_error(
            "M7-SOURCE-IDENTITY-INVALID",
            format!("{base}.source.identity"),
            "source identity requires a non-zero byte length and lowercase SHA-256",
        ));
    }
    let provenance = &source.provenance;
    if provenance.provider_task_id.trim().is_empty()
        || provenance.provider_task_id.len() > 256
        || provenance.provider_task_id.chars().any(char::is_control)
        || !provenance.original_export_attested
        || !provenance.rights_confirmed
        || !provenance.not_synthetic_fixture_attested
    {
        return Err(contract_error(
            "M7-ORIGINAL-MESHY-PROVENANCE-MISSING",
            format!("{base}.source.provenance"),
            "provider task id and every original-source attestation are required",
        ));
    }
    Ok(())
}

fn validate_slug(value: &str, path: &str) -> Result<(), M7CorpusContractErrorV1> {
    if value.is_empty()
        || value.len() > 64
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'_'
                || byte == b'-'
        })
    {
        return Err(contract_error(
            "M7-ID-INVALID",
            path,
            "id must be 1..64 lowercase ASCII letters, digits, '_' or '-'",
        ));
    }
    Ok(())
}

fn validate_relative_glb_path(
    value: &str,
    path: &str,
) -> Result<(), M7CorpusContractErrorV1> {
    let safe = !value.is_empty()
        && value.len() <= 240
        && !value.starts_with('/')
        && !value.contains('\\')
        && value.to_ascii_lowercase().ends_with(".glb")
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric()
                        || byte == b'.'
                        || byte == b'_'
                        || byte == b'-'
                })
        });
    if !safe {
        return Err(contract_error(
            "M7-SOURCE-PATH-INVALID",
            path,
            "source path must be a safe forward-slash relative .glb path",
        ));
    }
    Ok(())
}

fn validate_clip_names(
    names: &[String],
    base: &str,
) -> Result<(), M7CorpusContractErrorV1> {
    if names.is_empty() {
        return Err(contract_error(
            "M7-HUMANOID-CLIP-ROUTE-MISSING",
            format!("{base}.requiredSourceClipNames"),
            "humanoid role requires at least one explicitly named source clip",
        ));
    }
    let mut unique = BTreeSet::new();
    for (index, name) in names.iter().enumerate() {
        if name.trim().is_empty()
            || name.len() > 64
            || name.chars().any(char::is_control)
            || !unique.insert(name)
        {
            return Err(contract_error(
                "M7-HUMANOID-CLIP-NAME-INVALID",
                format!("{base}.requiredSourceClipNames[{index}]"),
                "source clip names must be non-empty, unique and at most 64 characters",
            ));
        }
    }
    Ok(())
}

fn validate_resref(value: &str, path: &str) -> Result<(), M7CorpusContractErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(contract_error(
            "M7-REFERENCE-SUPERMODEL-INVALID",
            path,
            "reference supermodel must be a 1..16 character ASCII resref",
        ));
    }
    Ok(())
}

fn empty_sample_report(
    entry: &M7CorpusEntryV1,
    status: M7IntakeStatusV1,
    source: Option<&M7SourceDescriptorV1>,
    profile_evidence: &str,
) -> M7IntakeSampleReportV1 {
    M7IntakeSampleReportV1 {
        sample_id: entry.sample_id().to_owned(),
        role: entry.role(),
        status,
        source_path: source.map(|source| source.relative_path.clone()),
        declared_identity: source.map(|source| source.identity.clone()),
        observed_identity: None,
        mesh_count: 0,
        primitive_count: 0,
        triangle_count: 0,
        skin_count: 0,
        animation_count: 0,
        profile_evidence: profile_evidence.to_owned(),
    }
}

fn diagnostic(
    code: &str,
    severity: M7IntakeSeverityV1,
    path: impl Into<String>,
    sample_id: Option<&str>,
    message: impl Into<String>,
) -> M7IntakeDiagnosticV1 {
    M7IntakeDiagnosticV1 {
        code: code.to_owned(),
        severity,
        path: path.into(),
        sample_id: sample_id.map(str::to_owned),
        message: message.into(),
    }
}

fn contract_error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> M7CorpusContractErrorV1 {
    M7CorpusContractErrorV1 {
        schema_version: M7_CORPUS_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
