use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mdl::{
    DirectCreatureEngineEnvelopeVerdictV1, direct_creature_engine_envelope_digest_v1,
};
use crate::{
    direct_creature_contract::DirectCreatureRuntimeProfileV2,
    model_pipeline::{
        M0AppearanceTableBindingV1, M0RuntimeAppearanceBindingV1, M0RuntimeFixtureContractV2,
        M0RuntimeResourceBindingV1, verify_m0_binary_runtime_fixture_contract_v2,
    },
    proof_module::BinaryM0VerticalSliceReadbackV1,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvidenceErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for RuntimeEvidenceErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for RuntimeEvidenceErrorV1 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogWindowVerdictV1 {
    ResourceLoadFailed,
    ResourceLoadObserved,
    ResourceLoadNotObservable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeModelVisibilityV1 {
    Visible,
    NotVisible,
    NotTested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProofCompletenessV1 {
    Verified,
    Failed,
    Missing,
}

/// Independent quality axis for a visible runtime draw.
///
/// Visibility alone is deliberately insufficient: a renderer may issue a draw
/// for malformed geometry or transforms. Such an observation is useful only
/// for proving that a byte family reaches the draw path.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeRenderIntegrityV1 {
    Correct,
    Corrupt,
    NotTested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeWitnessObservationV1 {
    pub exact_identity_bound: bool,
    pub model_visibility: RuntimeModelVisibilityV1,
    pub proof_completeness: RuntimeProofCompletenessV1,
    pub render_integrity: RuntimeRenderIntegrityV1,
}

/// The strongest use that may be made of one visual runtime observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeWitnessUseV1 {
    /// Visible and visually correct; eligible to ground creature correctness.
    CorrectCreatureBaseline,
    /// Visible but corrupt; proves only that the renderer issued a draw.
    DrawPathOnly,
    /// Missing, unbound, invisible, or visually unclassified.
    NotAdmitted,
}

/// Separates "NWN drew bytes" from "the creature model is correct".
///
/// A correctness baseline must be exact-identity-bound, fully verified,
/// visible, and visually correct. In particular, the historical H1 v20
/// deformed blob classifies as `DrawPathOnly` and cannot qualify another
/// model's hierarchy, geometry, transforms, animation, or render integrity.
pub fn classify_runtime_witness_use_v1(
    observation: &RuntimeWitnessObservationV1,
) -> RuntimeWitnessUseV1 {
    if !observation.exact_identity_bound
        || observation.proof_completeness != RuntimeProofCompletenessV1::Verified
        || observation.model_visibility != RuntimeModelVisibilityV1::Visible
    {
        return RuntimeWitnessUseV1::NotAdmitted;
    }

    match observation.render_integrity {
        RuntimeRenderIntegrityV1::Correct => RuntimeWitnessUseV1::CorrectCreatureBaseline,
        RuntimeRenderIntegrityV1::Corrupt => RuntimeWitnessUseV1::DrawPathOnly,
        RuntimeRenderIntegrityV1::NotTested => RuntimeWitnessUseV1::NotAdmitted,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogProcessIdentityV1 {
    pub pid: u32,
    pub start_time_utc: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileSnapshotV1 {
    pub informational_path: String,
    pub file_identity: String,
    pub byte_length: u64,
    pub sha256: String,
    pub modified_time_utc: String,
    pub read_time_utc: String,
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

impl LogFileSnapshotV1 {
    /// Creates a pure byte-parser input. The caller-supplied metadata is not
    /// OS-attested and this constructor can never establish runtime admission.
    pub fn from_bytes(
        informational_path: impl Into<String>,
        file_identity: impl Into<String>,
        modified_time_utc: impl Into<String>,
        read_time_utc: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            informational_path: informational_path.into(),
            file_identity: file_identity.into(),
            byte_length: bytes.len() as u64,
            sha256: sha256_bytes(&bytes),
            modified_time_utc: modified_time_utc.into(),
            read_time_utc: read_time_utc.into(),
            bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSliceBindingV1 {
    pub start_offset: u64,
    pub end_offset: u64,
    pub byte_length: u64,
    pub sha256: String,
    pub utf8: bool,
    pub normalized_observed_tokens: Vec<String>,
    pub matched_expected_tokens: Vec<String>,
    pub fatal_bound_resources: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileWindowBindingV1 {
    pub informational_path: String,
    pub file_identity: String,
    pub pre_byte_length: u64,
    pub pre_sha256: String,
    pub pre_modified_time_utc: String,
    pub pre_read_time_utc: String,
    pub post_byte_length: u64,
    pub post_sha256: String,
    pub post_modified_time_utc: String,
    pub post_read_time_utc: String,
    pub contiguous_suffix: bool,
    pub slice: LogSliceBindingV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCandidateIdentityV1 {
    pub fixture_id: String,
    pub fixture_scale: String,
    pub binary_scene: BinaryM0VerticalSliceReadbackV1,
    pub module: M0RuntimeResourceBindingV1,
    pub ordered_haks: Vec<M0RuntimeResourceBindingV1>,
    pub model: M0RuntimeResourceBindingV1,
    pub texture_tga: M0RuntimeResourceBindingV1,
    pub appearance_two_da: M0RuntimeResourceBindingV1,
    pub appearance_table: M0AppearanceTableBindingV1,
    pub appearance: M0RuntimeAppearanceBindingV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogWindowV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate: RuntimeCandidateIdentityV1,
    pub expected_normalized_resource_tokens: Vec<String>,
    pub process_pre: LogProcessIdentityV1,
    pub process_post: LogProcessIdentityV1,
    pub nwclient_log1: LogFileWindowBindingV1,
    pub nwengine_log: LogFileWindowBindingV1,
    pub observed_at_utc: String,
    pub verdict: LogWindowVerdictV1,
    pub diagnostic_only: bool,
    pub os_attested: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvidenceArtifactIdentityV1 {
    pub informational_path: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRuntimeObservationClaimV1 {
    pub schema_version: u32,
    pub profile: String,
    pub runtime_packet: RuntimeEvidenceArtifactIdentityV1,
    pub runtime_capture: RuntimeEvidenceArtifactIdentityV1,
    pub model_visibility: RuntimeModelVisibilityV1,
    pub proof_completeness: RuntimeProofCompletenessV1,
    pub diagnostic_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct PinnedRuntimeObservationV1 {
    id: String,
    candidate_sha256: String,
    runtime_packet: RuntimeEvidenceArtifactIdentityV1,
    runtime_capture: RuntimeEvidenceArtifactIdentityV1,
    model_visibility: RuntimeModelVisibilityV1,
    proof_completeness: RuntimeProofCompletenessV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRuntimeEvidenceSidecarV2 {
    pub schema_version: u32,
    pub profile: String,
    pub runtime_fixture_contract_sha256: String,
    pub engine_envelope_sha256: String,
    pub engine_structural_verdict: DirectCreatureEngineEnvelopeVerdictV1,
    pub candidate: RuntimeCandidateIdentityV1,
    pub log_window: LogWindowV1,
    pub runtime_packet: RuntimeEvidenceArtifactIdentityV1,
    pub runtime_capture: RuntimeEvidenceArtifactIdentityV1,
    pub pinned_observation_id: String,
    pub model_visibility: RuntimeModelVisibilityV1,
    pub proof_completeness: RuntimeProofCompletenessV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownRuntimeNegativeV1 {
    pub schema_version: u32,
    pub id: String,
    pub source_glb: RuntimeEvidenceArtifactIdentityV1,
    pub module: RuntimeEvidenceArtifactIdentityV1,
    pub hak: RuntimeEvidenceArtifactIdentityV1,
    pub model: RuntimeEvidenceArtifactIdentityV1,
    pub runtime_packet: RuntimeEvidenceArtifactIdentityV1,
    pub runtime_capture: RuntimeEvidenceArtifactIdentityV1,
    pub engine_log: RuntimeEvidenceArtifactIdentityV1,
    pub model_visibility: RuntimeModelVisibilityV1,
    pub proof_completeness: RuntimeProofCompletenessV1,
    pub structural_pass: bool,
    pub structural_verdict: DirectCreatureEngineEnvelopeVerdictV1,
}

pub fn inspect_log_window_v1(
    candidate: RuntimeCandidateIdentityV1,
    expected_resource_tokens: &[String],
    process_pre: LogProcessIdentityV1,
    process_post: LogProcessIdentityV1,
    nwclient_pre: LogFileSnapshotV1,
    nwclient_post: LogFileSnapshotV1,
    nwengine_pre: LogFileSnapshotV1,
    nwengine_post: LogFileSnapshotV1,
    observed_at_utc: impl Into<String>,
) -> Result<LogWindowV1, RuntimeEvidenceErrorV1> {
    require_candidate(&candidate)?;
    let mut expected = expected_resource_tokens
        .iter()
        .map(|token| normalize_expected_token(token))
        .collect::<Result<Vec<_>, _>>()?;
    if expected.is_empty() || has_duplicates(&expected) {
        return Err(runtime_error(
            "M2A-LOG-WINDOW-TOKEN-SET",
            "expectedResourceTokens",
            "expected resource tokens must be a non-empty unique normalized set",
        ));
    }
    expected.sort();
    let mut candidate_tokens = candidate
        .ordered_haks
        .iter()
        .map(|binding| &binding.resref)
        .chain([&candidate.module.resref, &candidate.model.resref])
        .map(|token| normalize_expected_token(token))
        .collect::<Result<Vec<_>, _>>()?;
    candidate_tokens.sort();
    candidate_tokens.dedup();
    if expected != candidate_tokens {
        return Err(runtime_error(
            "M2A-LOG-WINDOW-TOKEN-BINDING",
            "expectedResourceTokens",
            "the expected token set must equal the candidate module, ordered HAK, and model resrefs",
        ));
    }
    let observed_at_utc = observed_at_utc.into();
    let same_process = process_pre == process_post
        && process_pre.pid != 0
        && valid_utc_timestamp(&process_pre.start_time_utc);
    let client = inspect_log_file_window(&nwclient_pre, &nwclient_post, &expected)?;
    let engine = inspect_log_file_window(&nwengine_pre, &nwengine_post, &expected)?;
    let temporal_valid = valid_utc_timestamp(&observed_at_utc)
        && process_pre.start_time_utc.as_str() <= client.pre_read_time_utc.as_str()
        && process_pre.start_time_utc.as_str() <= engine.pre_read_time_utc.as_str()
        && client.post_read_time_utc.as_str() <= observed_at_utc.as_str()
        && engine.post_read_time_utc.as_str() <= observed_at_utc.as_str();
    let observation_valid = same_process
        && temporal_valid
        && client.contiguous_suffix
        && engine.contiguous_suffix
        && client.slice.utf8
        && engine.slice.utf8;
    let fatal = union_sorted(
        &client.slice.fatal_bound_resources,
        &engine.slice.fatal_bound_resources,
    );
    let matched = union_sorted(
        &client.slice.matched_expected_tokens,
        &engine.slice.matched_expected_tokens,
    );
    let verdict = if !observation_valid {
        LogWindowVerdictV1::ResourceLoadNotObservable
    } else if !fatal.is_empty() {
        LogWindowVerdictV1::ResourceLoadFailed
    } else if expected.iter().all(|token| matched.contains(token)) {
        LogWindowVerdictV1::ResourceLoadObserved
    } else {
        LogWindowVerdictV1::ResourceLoadNotObservable
    };
    Ok(LogWindowV1 {
        schema_version: 1,
        profile: "PROJECT_LOG_WINDOW_V1".to_owned(),
        candidate,
        expected_normalized_resource_tokens: expected,
        process_pre,
        process_post,
        nwclient_log1: client,
        nwengine_log: engine,
        observed_at_utc,
        verdict,
        diagnostic_only: true,
        os_attested: false,
    })
}

/// Internal admission path. The expected observation is constructed only by
/// the project-owned pinned registry; caller-supplied diagnostic claims are
/// deliberately not accepted by this API.
#[allow(clippy::too_many_arguments)]
fn build_project_runtime_evidence_sidecar_v2(
    runtime_fixture_contract_bytes: &[u8],
    expected_runtime_profile: &DirectCreatureRuntimeProfileV2,
    source_glb: &[u8],
    module: &[u8],
    hak: &[u8],
    log_window: LogWindowV1,
    runtime_packet_bytes: &[u8],
    runtime_capture_bytes: &[u8],
    pinned_observation: &PinnedRuntimeObservationV1,
) -> Result<ProjectRuntimeEvidenceSidecarV2, RuntimeEvidenceErrorV1> {
    let contract: M0RuntimeFixtureContractV2 =
        serde_json::from_slice(runtime_fixture_contract_bytes).map_err(|error| {
            runtime_error(
                "M2A-RUNTIME-SIDECAR-CONTRACT-PARSE",
                "runtimeFixtureContract",
                format!("exact V2 runtime contract bytes are not valid JSON: {error}"),
            )
        })?;
    verify_m0_binary_runtime_fixture_contract_v2(
        &contract,
        expected_runtime_profile,
        source_glb,
        module,
        hak,
    )
    .map_err(|error| runtime_error(error.code, error.path, error.message))?;

    let candidate = runtime_candidate_from_contract_v1(&contract)?;
    let candidate_sha256 = runtime_candidate_digest_v1(&candidate)?;
    if candidate_sha256 != pinned_observation.candidate_sha256 {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-PINNED-CANDIDATE",
            "candidate",
            "verified V2 contract candidate differs from the independently pinned registry identity",
        ));
    }
    if log_window.schema_version != 1
        || log_window.profile != "PROJECT_LOG_WINDOW_V1"
        || !log_window.diagnostic_only
        || log_window.os_attested
        || log_window.candidate != candidate
    {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-LOG-DIAGNOSTIC",
            "logWindow",
            "only a pure diagnostic, non-attested log window for the exact V2 candidate may be attached",
        ));
    }
    let recomputed_envelope_sha =
        direct_creature_engine_envelope_digest_v1(&contract.engine_envelope)
            .map_err(|error| runtime_error(error.code, error.path, error.message))?;
    if contract.engine_envelope.verdict
        != DirectCreatureEngineEnvelopeVerdictV1::StructuralPassRuntimeNotWitnessed
        || contract.engine_envelope_sha256 != recomputed_envelope_sha
    {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-ENVELOPE",
            "engineEnvelope",
            "the only legal structural statement is STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED recomputed from the verified ordered-HAK MDL",
        ));
    }

    let runtime_packet = artifact_from_bytes(
        pinned_observation.runtime_packet.informational_path.clone(),
        runtime_packet_bytes,
    )?;
    let runtime_capture = artifact_from_bytes(
        pinned_observation
            .runtime_capture
            .informational_path
            .clone(),
        runtime_capture_bytes,
    )?;
    require_pinned_runtime_observation(
        pinned_observation,
        &candidate_sha256,
        &runtime_packet,
        &runtime_capture,
    )?;
    let packet =
        parse_and_verify_runtime_packet_v1(runtime_packet_bytes, &runtime_capture, &candidate)?;
    if packet.model_visibility != pinned_observation.model_visibility
        || packet.proof_completeness != pinned_observation.proof_completeness
    {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-PINNED-VERDICT",
            "runtimePacket",
            "packet axes differ from the independently pinned registry observation",
        ));
    }

    Ok(ProjectRuntimeEvidenceSidecarV2 {
        schema_version: 2,
        profile: "PROJECT_DIRECT_CREATURE_RUNTIME_EVIDENCE_SIDECAR_V2".to_owned(),
        runtime_fixture_contract_sha256: sha256_bytes(runtime_fixture_contract_bytes),
        engine_envelope_sha256: recomputed_envelope_sha,
        engine_structural_verdict: contract.engine_envelope.verdict,
        candidate,
        log_window,
        runtime_packet,
        runtime_capture,
        pinned_observation_id: pinned_observation.id.clone(),
        model_visibility: packet.model_visibility,
        proof_completeness: packet.proof_completeness,
    })
}

pub fn project_runtime_evidence_sidecar_digest_v2(
    sidecar: &ProjectRuntimeEvidenceSidecarV2,
) -> Result<String, RuntimeEvidenceErrorV1> {
    if sidecar.schema_version != 2
        || sidecar.profile != "PROJECT_DIRECT_CREATURE_RUNTIME_EVIDENCE_SIDECAR_V2"
    {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-VERSION",
            "runtimeEvidence",
            "canonical sidecar digest requires the exact V2 profile; hashing alone never admits evidence",
        ));
    }
    let bytes = serde_json::to_vec(sidecar).map_err(|error| {
        runtime_error(
            "M2A-RUNTIME-SIDECAR-SERIALIZATION",
            "runtimeEvidence",
            format!("runtime evidence sidecar cannot be serialized: {error}"),
        )
    })?;
    Ok(sha256_bytes(&bytes))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketHakV1 {
    resref: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketModelV1 {
    resref: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketFixtureV1 {
    id: String,
    resref: String,
    template_res_ref: String,
    appearance_row: u16,
    position: [f64; 3],
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketVisibilityV1 {
    status: RuntimeModelVisibilityV1,
    blank: bool,
    placeholder: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketSpatialV1 {
    entry_point: [f64; 3],
    entry_direction: [f64; 2],
    fixture_position: [f64; 3],
    relative_vector: [f64; 3],
    forward_distance: f64,
    lateral_offset: f64,
    scale: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketArtifactV1 {
    kind: String,
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimePacketVisualInspectionV1 {
    capture_sha256: String,
    finding: String,
    model_visibility: RuntimeModelVisibilityV1,
    proof_completeness: RuntimeProofCompletenessV1,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePacketV1 {
    version: String,
    module_resref: String,
    module_sha256: String,
    area: String,
    entry_point: [f64; 3],
    entry_direction: [f64; 2],
    fixtures: Vec<RuntimePacketFixtureV1>,
    ordered_hak_list: Vec<RuntimePacketHakV1>,
    model_resource: RuntimePacketModelV1,
    model_visibility: RuntimePacketVisibilityV1,
    proof_completeness: RuntimeProofCompletenessV1,
    spatial_binding: RuntimePacketSpatialV1,
    artifacts: Vec<RuntimePacketArtifactV1>,
    visual_inspection: RuntimePacketVisualInspectionV1,
}

struct ParsedRuntimeAxesV1 {
    model_visibility: RuntimeModelVisibilityV1,
    proof_completeness: RuntimeProofCompletenessV1,
}

/// Parses same-caller packet/capture bytes into an explicitly non-admitting
/// diagnostic claim. No admission API accepts this type.
pub fn inspect_diagnostic_runtime_observation_claim_v1(
    runtime_packet_path: impl Into<String>,
    runtime_packet_bytes: &[u8],
    runtime_capture_path: impl Into<String>,
    runtime_capture_bytes: &[u8],
) -> Result<DiagnosticRuntimeObservationClaimV1, RuntimeEvidenceErrorV1> {
    let runtime_packet = artifact_from_bytes(runtime_packet_path, runtime_packet_bytes)?;
    let runtime_capture = artifact_from_bytes(runtime_capture_path, runtime_capture_bytes)?;
    let packet: RuntimePacketV1 =
        serde_json::from_slice(runtime_packet_bytes).map_err(|error| {
            runtime_error(
                "M2A-RUNTIME-DIAGNOSTIC-PACKET-PARSE",
                "runtimePacket",
                format!("diagnostic runtime packet is not valid v1 JSON: {error}"),
            )
        })?;
    let capture_artifacts = packet
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == "runtime-capture")
        .collect::<Vec<_>>();
    if packet.version != "aur-s07-runtime-packet/v1"
        || capture_artifacts.len() != 1
        || capture_artifacts[0].path != runtime_capture.informational_path
        || capture_artifacts[0].sha256 != runtime_capture.sha256
        || packet.visual_inspection.capture_sha256 != runtime_capture.sha256
        || packet.model_visibility.blank
        || packet.model_visibility.placeholder
        || packet.model_visibility.status != packet.visual_inspection.model_visibility
        || packet.proof_completeness != packet.visual_inspection.proof_completeness
    {
        return Err(runtime_error(
            "M2A-RUNTIME-DIAGNOSTIC-CLAIM-MISMATCH",
            "runtimePacket",
            "diagnostic claim requires internally consistent packet/capture bytes but establishes no candidate identity or runtime admission",
        ));
    }
    Ok(DiagnosticRuntimeObservationClaimV1 {
        schema_version: 1,
        profile: "DIAGNOSTIC_RUNTIME_OBSERVATION_CLAIM_V1".to_owned(),
        runtime_packet,
        runtime_capture,
        model_visibility: packet.model_visibility.status,
        proof_completeness: packet.proof_completeness,
        diagnostic_only: true,
    })
}

fn runtime_candidate_from_contract_v1(
    contract: &M0RuntimeFixtureContractV2,
) -> Result<RuntimeCandidateIdentityV1, RuntimeEvidenceErrorV1> {
    let candidate = RuntimeCandidateIdentityV1 {
        fixture_id: "m0_fixture".to_owned(),
        fixture_scale: "1.00".to_owned(),
        binary_scene: contract.binary_scene.clone(),
        module: contract.module.clone(),
        ordered_haks: vec![contract.hak.clone()],
        model: contract.model.clone(),
        texture_tga: contract.texture.clone(),
        appearance_two_da: contract.appearance_two_da.clone(),
        appearance_table: contract.appearance_table.clone(),
        appearance: contract.appearance.clone(),
    };
    require_candidate(&candidate)?;
    Ok(candidate)
}

fn runtime_candidate_digest_v1(
    candidate: &RuntimeCandidateIdentityV1,
) -> Result<String, RuntimeEvidenceErrorV1> {
    require_candidate(candidate)?;
    let bytes =
        serde_json::to_vec(&("RUNTIME_CANDIDATE_IDENTITY_V1", candidate)).map_err(|error| {
            runtime_error(
                "M2A-RUNTIME-CANDIDATE-SERIALIZATION",
                "candidate",
                format!("runtime candidate cannot be canonically serialized: {error}"),
            )
        })?;
    Ok(sha256_bytes(&bytes))
}

fn parse_and_verify_runtime_packet_v1(
    packet_bytes: &[u8],
    runtime_capture: &RuntimeEvidenceArtifactIdentityV1,
    candidate: &RuntimeCandidateIdentityV1,
) -> Result<ParsedRuntimeAxesV1, RuntimeEvidenceErrorV1> {
    let packet: RuntimePacketV1 = serde_json::from_slice(packet_bytes).map_err(|error| {
        runtime_error(
            "M2A-RUNTIME-PACKET-PARSE",
            "runtimePacket",
            format!("central runtime packet is not valid v1 JSON: {error}"),
        )
    })?;
    if packet.version != "aur-s07-runtime-packet/v1" {
        return Err(runtime_error(
            "M2A-RUNTIME-PACKET-VERSION",
            "runtimePacket.version",
            "expected exact central aur-s07-runtime-packet/v1",
        ));
    }
    let scene = &candidate.binary_scene;
    let fixture = &scene.fixture;
    let expected_entry = [
        f64::from(scene.entry_position.x),
        f64::from(scene.entry_position.y),
        f64::from(scene.entry_position.z),
    ];
    let expected_direction = [
        f64::from(scene.entry_direction.x),
        f64::from(scene.entry_direction.y),
    ];
    let expected_fixture = [
        f64::from(fixture.position.x),
        f64::from(fixture.position.y),
        f64::from(fixture.position.z),
    ];
    let expected_relative = [
        expected_fixture[0] - expected_entry[0],
        expected_fixture[1] - expected_entry[1],
        expected_fixture[2] - expected_entry[2],
    ];
    let expected_forward =
        expected_relative[0] * expected_direction[0] + expected_relative[1] * expected_direction[1];
    let expected_lateral = expected_relative[0] * -expected_direction[1]
        + expected_relative[1] * expected_direction[0];
    let packet_fixture = packet
        .fixtures
        .first()
        .filter(|_| packet.fixtures.len() == 1);
    let packet_hak = packet
        .ordered_hak_list
        .first()
        .filter(|_| packet.ordered_hak_list.len() == 1);
    let artifact_capture = packet
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == "runtime-capture")
        .collect::<Vec<_>>();
    let capture_sha = &runtime_capture.sha256;
    let identity_matches = packet.module_resref == candidate.module.resref
        && packet.module_sha256 == candidate.module.sha256
        && packet.area == scene.area_resref
        && packet.entry_point == expected_entry
        && packet.entry_direction == expected_direction
        && packet_fixture.is_some_and(|value| {
            value.id == candidate.fixture_id
                && value.resref == candidate.model.resref
                && value.template_res_ref == fixture.template_resref
                && value.appearance_row == fixture.appearance_row
                && value.position == expected_fixture
        })
        && packet_hak.is_some_and(|value| {
            value.resref == candidate.ordered_haks[0].resref
                && value.sha256 == candidate.ordered_haks[0].sha256
        })
        && packet.model_resource.resref == candidate.model.resref
        && packet.model_resource.sha256 == candidate.model.sha256
        && packet.spatial_binding.entry_point == expected_entry
        && packet.spatial_binding.entry_direction == expected_direction
        && packet.spatial_binding.fixture_position == expected_fixture
        && packet.spatial_binding.relative_vector == expected_relative
        && packet.spatial_binding.forward_distance == expected_forward
        && packet.spatial_binding.lateral_offset == expected_lateral
        && packet.spatial_binding.scale == 1.0
        && artifact_capture.len() == 1
        && artifact_capture[0].path == runtime_capture.informational_path
        && artifact_capture[0].sha256 == *capture_sha
        && packet.visual_inspection.capture_sha256 == *capture_sha
        && !packet.visual_inspection.finding.trim().is_empty();
    if !identity_matches {
        return Err(runtime_error(
            "M2A-RUNTIME-PACKET-CANDIDATE-MISMATCH",
            "runtimePacket",
            "packet module/Area/fixture/ordered-HAK/model/spatial/capture binding differs from the verified V2 candidate",
        ));
    }
    if packet.model_visibility.blank
        || packet.model_visibility.placeholder
        || packet.model_visibility.status != packet.visual_inspection.model_visibility
        || packet.proof_completeness != packet.visual_inspection.proof_completeness
    {
        return Err(runtime_error(
            "M2A-RUNTIME-PACKET-VERDICT-MISMATCH",
            "runtimePacket.visualInspection",
            "runtime axes must be internally consistent and cannot be inferred from blank/placeholder capture states",
        ));
    }
    Ok(ParsedRuntimeAxesV1 {
        model_visibility: packet.model_visibility.status,
        proof_completeness: packet.proof_completeness,
    })
}

fn artifact_from_bytes(
    informational_path: impl Into<String>,
    bytes: &[u8],
) -> Result<RuntimeEvidenceArtifactIdentityV1, RuntimeEvidenceErrorV1> {
    let identity = RuntimeEvidenceArtifactIdentityV1 {
        informational_path: informational_path.into(),
        byte_length: bytes.len() as u64,
        sha256: sha256_bytes(bytes),
    };
    require_artifact_identity(&identity, "artifact")?;
    Ok(identity)
}

fn require_pinned_runtime_observation(
    pinned: &PinnedRuntimeObservationV1,
    candidate_sha256: &str,
    runtime_packet: &RuntimeEvidenceArtifactIdentityV1,
    runtime_capture: &RuntimeEvidenceArtifactIdentityV1,
) -> Result<(), RuntimeEvidenceErrorV1> {
    require_artifact_identity(&pinned.runtime_packet, "pinnedObservation.runtimePacket")?;
    require_artifact_identity(&pinned.runtime_capture, "pinnedObservation.runtimeCapture")?;
    if pinned.id.trim().is_empty()
        || !is_sha256(&pinned.candidate_sha256)
        || pinned.candidate_sha256 != candidate_sha256
        || &pinned.runtime_packet != runtime_packet
        || &pinned.runtime_capture != runtime_capture
    {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-PINNED-IDENTITY",
            "pinnedObservation",
            "candidate digest and exact packet/capture bytes/path must equal the private registry pin",
        ));
    }
    Ok(())
}

pub fn known_r30_runtime_negative_v1() -> KnownRuntimeNegativeV1 {
    KnownRuntimeNegativeV1 {
        schema_version: 1,
        id: "m0-r30-retail-runtime-conformance-20260721".to_owned(),
        source_glb: artifact(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb",
            8_581_684,
            "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1",
        ),
        module: artifact(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0r30.mod",
            13_173,
            "8c87317956053cecdf40d939c9061a174943b982a8db2c94d35aa2fba44043df",
        ),
        hak: artifact(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0r30.hak",
            19_634_596,
            "6694c0fb7f16665025d312dfd35c4bcbb293fccdb0760c110492a5eb2d576d09",
        ),
        model: artifact(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/generated/m2a_m0p01.mdl",
            150_024,
            "43a5cbfa1a20146ec0990ce7ee980d70a54d7f72689781a58c7d9614bc35b8c7",
        ),
        runtime_packet: artifact(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/live/runtime/aur-s07-runtime-packet.json",
            5_093,
            "a3ff0ae6708b8235beee1161b06ea7654e999578be39a174ee08c1389176af21",
        ),
        runtime_capture: artifact(
            "C:\\Projects\\meshy2aurora\\proof-output\\m0-r30-retail-runtime-conformance-20260721\\live\\runtime\\r30-nwn-runtime.png",
            4_875_265,
            "a09fdaeea3f2e1c400e88cc332b3a6437ca535502d60ffed5be7be1f95aea9dc",
        ),
        engine_log: artifact(
            "proof-output/m0-r30-retail-runtime-conformance-20260721/live/runtime/engine-load-log.txt",
            1_122,
            "5063c46fcc90e577bb85234337c0ef03794271c6ecd561e72007599ca84ae9a8",
        ),
        model_visibility: RuntimeModelVisibilityV1::NotVisible,
        proof_completeness: RuntimeProofCompletenessV1::Failed,
        structural_pass: true,
        structural_verdict:
            DirectCreatureEngineEnvelopeVerdictV1::StructuralPassRuntimeNotWitnessed,
    }
}

const R30_RUNTIME_CANDIDATE_SHA256: &str =
    "616b2064be4fde23ed1103ebbc5743ff499f1556c69b258a17e29679ff88b317";

/// Private registry pin for the frozen r30 negative. No public constructor
/// accepts caller-supplied packet/capture identities or axes.
fn known_r30_pinned_runtime_observation_v1() -> PinnedRuntimeObservationV1 {
    let negative = known_r30_runtime_negative_v1();
    PinnedRuntimeObservationV1 {
        id: negative.id,
        candidate_sha256: R30_RUNTIME_CANDIDATE_SHA256.to_owned(),
        runtime_packet: negative.runtime_packet,
        runtime_capture: negative.runtime_capture,
        model_visibility: RuntimeModelVisibilityV1::NotVisible,
        proof_completeness: RuntimeProofCompletenessV1::Failed,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn verify_known_r30_runtime_negative_v1(
    runtime_fixture_contract_bytes: &[u8],
    expected_runtime_profile: &DirectCreatureRuntimeProfileV2,
    source_glb: &[u8],
    module: &[u8],
    hak: &[u8],
    log_window: LogWindowV1,
    runtime_packet_bytes: &[u8],
    runtime_capture_bytes: &[u8],
) -> Result<ProjectRuntimeEvidenceSidecarV2, RuntimeEvidenceErrorV1> {
    let pinned = known_r30_pinned_runtime_observation_v1();
    build_project_runtime_evidence_sidecar_v2(
        runtime_fixture_contract_bytes,
        expected_runtime_profile,
        source_glb,
        module,
        hak,
        log_window,
        runtime_packet_bytes,
        runtime_capture_bytes,
        &pinned,
    )
}

fn inspect_log_file_window(
    pre: &LogFileSnapshotV1,
    post: &LogFileSnapshotV1,
    expected: &[String],
) -> Result<LogFileWindowBindingV1, RuntimeEvidenceErrorV1> {
    let metadata_valid = !pre.informational_path.trim().is_empty()
        && pre.informational_path == post.informational_path
        && !pre.file_identity.trim().is_empty()
        && pre.file_identity == post.file_identity
        && pre.byte_length == pre.bytes.len() as u64
        && post.byte_length == post.bytes.len() as u64
        && pre.sha256 == sha256_bytes(&pre.bytes)
        && post.sha256 == sha256_bytes(&post.bytes)
        && !pre.modified_time_utc.trim().is_empty()
        && valid_utc_timestamp(&pre.modified_time_utc)
        && valid_utc_timestamp(&post.modified_time_utc)
        && valid_utc_timestamp(&pre.read_time_utc)
        && valid_utc_timestamp(&post.read_time_utc)
        && pre.modified_time_utc.as_str() <= pre.read_time_utc.as_str()
        && post.modified_time_utc.as_str() <= post.read_time_utc.as_str()
        && pre.read_time_utc.as_str() <= post.read_time_utc.as_str();
    let contiguous =
        metadata_valid && post.bytes.len() >= pre.bytes.len() && post.bytes.starts_with(&pre.bytes);
    let slice = if contiguous {
        &post.bytes[pre.bytes.len()..]
    } else {
        &[]
    };
    let text = std::str::from_utf8(slice).ok();
    let observed_tokens = text.map(tokenize).unwrap_or_default();
    let matched = text
        .map(|text| positively_observed_resources(text, expected))
        .unwrap_or_default();
    let fatal = text
        .map(|text| fatal_bound_resources(text, expected))
        .unwrap_or_default();
    Ok(LogFileWindowBindingV1 {
        informational_path: pre.informational_path.clone(),
        file_identity: pre.file_identity.clone(),
        pre_byte_length: pre.byte_length,
        pre_sha256: pre.sha256.clone(),
        pre_modified_time_utc: pre.modified_time_utc.clone(),
        pre_read_time_utc: pre.read_time_utc.clone(),
        post_byte_length: post.byte_length,
        post_sha256: post.sha256.clone(),
        post_modified_time_utc: post.modified_time_utc.clone(),
        post_read_time_utc: post.read_time_utc.clone(),
        contiguous_suffix: contiguous,
        slice: LogSliceBindingV1 {
            start_offset: pre.bytes.len() as u64,
            end_offset: if contiguous {
                post.bytes.len() as u64
            } else {
                pre.bytes.len() as u64
            },
            byte_length: slice.len() as u64,
            sha256: sha256_bytes(slice),
            utf8: text.is_some(),
            normalized_observed_tokens: observed_tokens,
            matched_expected_tokens: matched,
            fatal_bound_resources: fatal,
        },
    })
}

fn positively_observed_resources(text: &str, expected: &[String]) -> Vec<String> {
    let positive_markers = [
        "loading module",
        "loading resource",
        "loaded resource",
        "using hak",
    ];
    let mut matched = Vec::new();
    for line in text.lines() {
        let folded = line.to_ascii_lowercase();
        if !positive_markers
            .iter()
            .any(|marker| folded.contains(marker))
        {
            continue;
        }
        let tokens = tokenize(line);
        for expected in expected {
            if tokens.contains(expected) && !matched.contains(expected) {
                matched.push(expected.clone());
            }
        }
    }
    matched.sort();
    matched
}

fn fatal_bound_resources(text: &str, expected: &[String]) -> Vec<String> {
    let fatal_markers = [
        "failed to load",
        "could not load",
        "cannot load",
        "missing resource",
        "resource load failed",
    ];
    let mut matched = Vec::new();
    for line in text.lines() {
        let folded = line.to_ascii_lowercase();
        if !fatal_markers.iter().any(|marker| folded.contains(marker)) {
            continue;
        }
        let tokens = tokenize(line);
        for expected in expected {
            if tokens.contains(expected) && !matched.contains(expected) {
                matched.push(expected.clone());
            }
        }
    }
    matched.sort();
    matched
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = text
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    tokens.sort();
    tokens.dedup();
    tokens
}

fn normalize_expected_token(token: &str) -> Result<String, RuntimeEvidenceErrorV1> {
    let normalized = token.to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.len() > 32
        || !normalized
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(runtime_error(
            "M2A-LOG-WINDOW-TOKEN",
            "expectedResourceTokens",
            "resource tokens must be complete ASCII resrefs, not suffixes, rows, or free text",
        ));
    }
    Ok(normalized)
}

fn require_candidate(candidate: &RuntimeCandidateIdentityV1) -> Result<(), RuntimeEvidenceErrorV1> {
    let resources = candidate.ordered_haks.iter().chain([
        &candidate.module,
        &candidate.model,
        &candidate.texture_tga,
        &candidate.appearance_two_da,
    ]);
    if candidate.fixture_id != "m0_fixture"
        || candidate.fixture_scale != "1.00"
        || candidate.binary_scene.schema_version != 1
        || candidate.binary_scene.area_resref.is_empty()
        || candidate.ordered_haks.is_empty()
        || candidate.binary_scene.ordered_hak_resrefs
            != candidate
                .ordered_haks
                .iter()
                .map(|binding| binding.resref.clone())
                .collect::<Vec<_>>()
        || candidate.binary_scene.module_resref != candidate.module.resref
        || candidate.binary_scene.fixture.appearance_row != candidate.appearance.physical_row
        || candidate.appearance_table.appended_physical_row != candidate.appearance.physical_row
        || candidate.appearance_table.output_sha256 != candidate.appearance_two_da.sha256
        || resources.into_iter().any(|resource| {
            resource.resref.is_empty() || resource.byte_length == 0 || !is_sha256(&resource.sha256)
        })
    {
        return Err(runtime_error(
            "M2A-LOG-WINDOW-CANDIDATE",
            "candidate",
            "log diagnostics require the full exact V2 module/Area/fixture/TGA/appearance/ordered-HAK/model identity",
        ));
    }
    Ok(())
}

fn require_artifact_identity(
    artifact: &RuntimeEvidenceArtifactIdentityV1,
    path: &str,
) -> Result<(), RuntimeEvidenceErrorV1> {
    if artifact.informational_path.trim().is_empty()
        || artifact.byte_length == 0
        || !is_sha256(&artifact.sha256)
    {
        return Err(runtime_error(
            "M2A-RUNTIME-SIDECAR-ARTIFACT",
            path,
            "runtime evidence artifacts require an informational path, nonzero length, and exact SHA-256",
        ));
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

fn has_duplicates(values: &[String]) -> bool {
    values.windows(2).any(|pair| pair[0] == pair[1])
        || values
            .iter()
            .enumerate()
            .any(|(index, value)| values[..index].contains(value))
}

fn union_sorted(left: &[String], right: &[String]) -> Vec<String> {
    let mut values = left.iter().chain(right).cloned().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn artifact(path: &str, byte_length: u64, sha256: &str) -> RuntimeEvidenceArtifactIdentityV1 {
    RuntimeEvidenceArtifactIdentityV1 {
        informational_path: path.to_owned(),
        byte_length,
        sha256: sha256.to_owned(),
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn runtime_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> RuntimeEvidenceErrorV1 {
    RuntimeEvidenceErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn candidate() -> RuntimeCandidateIdentityV1 {
        RuntimeCandidateIdentityV1 {
            fixture_id: "m0_fixture".to_owned(),
            fixture_scale: "1.00".to_owned(),
            binary_scene: crate::proof_module::BinaryM0VerticalSliceReadbackV1 {
                schema_version: 1,
                module_resref: "m2a_m0r30".to_owned(),
                area_resref: "m2a_m0a30".to_owned(),
                ordered_hak_resrefs: vec!["m2a_m0r30".to_owned()],
                entry_position: crate::proof_module::M0RuntimePositionV1 {
                    x: 10.0,
                    y: 10.0,
                    z: 0.0,
                },
                entry_direction: crate::proof_module::M0RuntimeDirectionV1 { x: 0.0, y: 1.0 },
                fixture: crate::proof_module::BinaryM0FixtureReadbackV1 {
                    template_resref: "nw_dwarfmerc001".to_owned(),
                    appearance_row: 15_100,
                    position: crate::proof_module::M0RuntimePositionV1 {
                        x: 10.0,
                        y: 14.5,
                        z: 0.0,
                    },
                    orientation: crate::proof_module::M0RuntimeDirectionV1 { x: 0.0, y: 0.0 },
                },
            },
            module: resource("m2a_m0r30", 'a'),
            ordered_haks: vec![resource("m2a_m0r30", 'b')],
            model: resource("m2a_m0p01", 'c'),
            texture_tga: resource("m2a_m0t01", 'd'),
            appearance_two_da: resource("appearance", 'e'),
            appearance_table: crate::model_pipeline::M0AppearanceTableBindingV1 {
                schema_version: 1,
                scope: crate::model_pipeline::M0AppearanceTableScopeV1::FullRuntimeAppendV1,
                input_physical_rows: 15_100,
                output_physical_rows: 15_101,
                appended_physical_row: 15_100,
                input_byte_length: 9,
                output_byte_length: 10,
                input_sha256: "f".repeat(64),
                output_sha256: "e".repeat(64),
                source_prefix_preserved: true,
            },
            appearance: crate::model_pipeline::M0RuntimeAppearanceBindingV1 {
                physical_row: 15_100,
                label: "M2A_M0_STATIC".to_owned(),
                model_type: "S".to_owned(),
                race: "m2a_m0p01".to_owned(),
            },
        }
    }

    fn resource(resref: &str, hash_digit: char) -> M0RuntimeResourceBindingV1 {
        M0RuntimeResourceBindingV1 {
            resref: resref.to_owned(),
            byte_length: 1,
            sha256: hash_digit.to_string().repeat(64),
        }
    }

    fn process() -> LogProcessIdentityV1 {
        LogProcessIdentityV1 {
            pid: 1234,
            start_time_utc: "2026-07-21T22:00:00Z".to_owned(),
        }
    }

    fn snapshot(path: &str, identity: &str, read: &str, bytes: &[u8]) -> LogFileSnapshotV1 {
        LogFileSnapshotV1::from_bytes(path, identity, "2026-07-21T22:01:00Z", read, bytes.to_vec())
    }

    fn inspect(
        client_pre: &[u8],
        client_post: &[u8],
        engine_pre: &[u8],
        engine_post: &[u8],
        pre_process: LogProcessIdentityV1,
        post_process: LogProcessIdentityV1,
    ) -> LogWindowV1 {
        inspect_log_window_v1(
            candidate(),
            &["m2a_m0r30".to_owned(), "m2a_m0p01".to_owned()],
            pre_process,
            post_process,
            snapshot(
                "nwclientLog1.txt",
                "client-file-1",
                "2026-07-21T22:01:01Z",
                client_pre,
            ),
            snapshot(
                "nwclientLog1.txt",
                "client-file-1",
                "2026-07-21T22:01:02Z",
                client_post,
            ),
            snapshot(
                "nwengineLog.txt",
                "engine-file-1",
                "2026-07-21T22:01:01Z",
                engine_pre,
            ),
            snapshot(
                "nwengineLog.txt",
                "engine-file-1",
                "2026-07-21T22:01:02Z",
                engine_post,
            ),
            "2026-07-21T22:01:03Z",
        )
        .expect("pure log-window report")
    }

    #[test]
    fn c_log_window_requires_new_contiguous_token_bound_positive_observations() {
        let report = inspect(
            b"old line\n",
            b"old line\nLoading Module: m2a_m0r30\n",
            b"engine old\n",
            b"engine old\nLoaded Resource: m2a_m0p01\n",
            process(),
            process(),
        );
        assert_eq!(report.verdict, LogWindowVerdictV1::ResourceLoadObserved);
        assert!(report.diagnostic_only);
        assert!(!report.os_attested);
        assert_eq!(report.nwclient_log1.slice.start_offset, 9);
        assert_eq!(report.nwengine_log.slice.start_offset, 11);
    }

    #[test]
    fn c_no_log_token_old_window_suffix_substring_bare_row_and_warning_are_not_loaded() {
        let cases: [(&[u8], &[u8], &[u8], &[u8]); 5] = [
            (b"old\n", b"old\nnew unrelated line\n", b"e\n", b"e\nnone\n"),
            (
                b"Loading Module: m2a_m0r30\n",
                b"Loading Module: m2a_m0r30\nquiet\n",
                b"Loaded Resource: m2a_m0p01\n",
                b"Loaded Resource: m2a_m0p01\nquiet\n",
            ),
            (
                b"",
                b"Loading Module: xm2a_m0r30\n",
                b"",
                b"Loaded Resource: m2a_m0p01_suffix\n",
            ),
            (
                b"",
                b"Loading Module: 15100\n",
                b"",
                b"Loaded Resource: 15100\n",
            ),
            (b"", b"warning m2a_m0r30\n", b"", b"warning m2a_m0p01\n"),
        ];
        for (client_pre, client_post, engine_pre, engine_post) in cases {
            let report = inspect(
                client_pre,
                client_post,
                engine_pre,
                engine_post,
                process(),
                process(),
            );
            assert_eq!(
                report.verdict,
                LogWindowVerdictV1::ResourceLoadNotObservable
            );
        }
    }

    #[test]
    fn c_only_defined_fatal_diagnostic_naming_bound_resource_is_failed() {
        let unrelated = inspect(
            b"",
            b"failed to load unrelated_model\n",
            b"",
            b"warning m2a_m0p01\n",
            process(),
            process(),
        );
        assert_eq!(
            unrelated.verdict,
            LogWindowVerdictV1::ResourceLoadNotObservable
        );
        let bound = inspect(
            b"",
            b"failed to load m2a_m0r30\n",
            b"",
            b"Loaded Resource: m2a_m0p01\n",
            process(),
            process(),
        );
        assert_eq!(bound.verdict, LogWindowVerdictV1::ResourceLoadFailed);
    }

    #[test]
    fn c_regenerated_rotation_pid_encoding_and_noncontiguous_attacks_are_not_observable() {
        let mut changed_process = process();
        changed_process.pid += 1;
        let pid = inspect(
            b"old\n",
            b"old\nLoading Module: m2a_m0r30\n",
            b"e\n",
            b"e\nLoaded Resource: m2a_m0p01\n",
            process(),
            changed_process,
        );
        assert_eq!(pid.verdict, LogWindowVerdictV1::ResourceLoadNotObservable);
        let rotated = inspect(
            b"old content\n",
            b"Loading Module: m2a_m0r30\n",
            b"engine old\n",
            b"Loaded Resource: m2a_m0p01\n",
            process(),
            process(),
        );
        assert_eq!(
            rotated.verdict,
            LogWindowVerdictV1::ResourceLoadNotObservable
        );
        let encoding = inspect(
            b"",
            &[b"Loading Module: m2a_m0r30\n".as_slice(), &[0xff]].concat(),
            b"",
            b"Loaded Resource: m2a_m0p01\n",
            process(),
            process(),
        );
        assert_eq!(
            encoding.verdict,
            LogWindowVerdictV1::ResourceLoadNotObservable
        );
    }

    #[test]
    fn c_candidate_token_and_timestamp_binding_fail_closed_without_promoting_load() {
        let token_error = inspect_log_window_v1(
            candidate(),
            &["m2a_m0p01".to_owned()],
            process(),
            process(),
            snapshot(
                "nwclientLog1.txt",
                "client-file-1",
                "2026-07-21T22:01:01Z",
                b"",
            ),
            snapshot(
                "nwclientLog1.txt",
                "client-file-1",
                "2026-07-21T22:01:02Z",
                b"Loading Module: m2a_m0r30\n",
            ),
            snapshot(
                "nwengineLog.txt",
                "engine-file-1",
                "2026-07-21T22:01:01Z",
                b"",
            ),
            snapshot(
                "nwengineLog.txt",
                "engine-file-1",
                "2026-07-21T22:01:02Z",
                b"Loaded Resource: m2a_m0p01\n",
            ),
            "2026-07-21T22:01:03Z",
        )
        .expect_err("a partial arbitrary token set must not be admissible");
        assert_eq!(token_error.code, "M2A-LOG-WINDOW-TOKEN-BINDING");

        let out_of_order = inspect_log_window_v1(
            candidate(),
            &["m2a_m0r30".to_owned(), "m2a_m0p01".to_owned()],
            process(),
            process(),
            snapshot(
                "nwclientLog1.txt",
                "client-file-1",
                "2026-07-21T22:01:05Z",
                b"",
            ),
            snapshot(
                "nwclientLog1.txt",
                "client-file-1",
                "2026-07-21T22:01:04Z",
                b"Loading Module: m2a_m0r30\n",
            ),
            snapshot(
                "nwengineLog.txt",
                "engine-file-1",
                "2026-07-21T22:01:05Z",
                b"",
            ),
            snapshot(
                "nwengineLog.txt",
                "engine-file-1",
                "2026-07-21T22:01:04Z",
                b"Loaded Resource: m2a_m0p01\n",
            ),
            "2026-07-21T22:01:03Z",
        )
        .expect("invalid chronology is a neutral diagnostic report");
        assert_eq!(
            out_of_order.verdict,
            LogWindowVerdictV1::ResourceLoadNotObservable
        );
    }

    #[test]
    fn c_r30_structural_pass_is_permanently_separate_from_runtime_failure() {
        let negative = known_r30_runtime_negative_v1();
        assert!(negative.structural_pass);
        assert_eq!(
            negative.structural_verdict,
            DirectCreatureEngineEnvelopeVerdictV1::StructuralPassRuntimeNotWitnessed
        );
        assert_eq!(
            negative.model_visibility,
            RuntimeModelVisibilityV1::NotVisible
        );
        assert_eq!(
            negative.proof_completeness,
            RuntimeProofCompletenessV1::Failed
        );
        assert_eq!(
            negative.source_glb.sha256,
            "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1"
        );
        assert_eq!(
            negative.model.sha256,
            "43a5cbfa1a20146ec0990ce7ee980d70a54d7f72689781a58c7d9614bc35b8c7"
        );
    }

    #[test]
    fn c_generic_caller_minted_observation_has_no_public_admission_api() {
        let source = include_str!("runtime_evidence.rs");
        assert!(!source.contains("\npub struct SealedRuntimeObservationV1"));
        assert!(!source.contains("\npub fn build_project_runtime_evidence_sidecar_v2"));
        assert!(!source.contains("\npub fn verify_project_runtime_evidence_sidecar_v2"));
        assert!(source.contains("\npub struct DiagnosticRuntimeObservationClaimV1"));
        assert!(source.contains("\npub fn verify_known_r30_runtime_negative_v1"));
    }

    #[test]
    #[ignore = "requires the exact project-owned r30 proof-output negative fixture"]
    fn a_b_c_exact_r30_mod_hak_source_replay_and_registry_pinned_runtime_negative() {
        use crate::{
            direct_creature_contract::{
                SourceTopologyOriginV1, declare_m0_direct_creature_runtime_profile_v2,
                inspect_m0_source_topology_binding_v1,
            },
            erf::ErfArchive,
            model_pipeline::build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2,
            proof_module::{
                BinaryM0VerticalSliceIdentityV1, inspect_binary_m0_vertical_slice_module_v1,
            },
        };

        let negative = known_r30_runtime_negative_v1();
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for artifact in [
            &negative.source_glb,
            &negative.module,
            &negative.hak,
            &negative.model,
            &negative.runtime_packet,
            &negative.runtime_capture,
            &negative.engine_log,
        ] {
            let bytes = fs::read(workspace.join(&artifact.informational_path))
                .expect("exact r30 negative artifact must remain present when this test is forced");
            assert_eq!(bytes.len() as u64, artifact.byte_length);
            assert_eq!(sha256_bytes(&bytes), artifact.sha256);
        }
        let source = fs::read(workspace.join(&negative.source_glb.informational_path))
            .expect("exact r30 source");
        let module =
            fs::read(workspace.join(&negative.module.informational_path)).expect("exact r30 MOD");
        let hak =
            fs::read(workspace.join(&negative.hak.informational_path)).expect("exact r30 HAK");
        let mdl =
            fs::read(workspace.join(&negative.model.informational_path)).expect("exact r30 model");
        let full_appearance = fs::read(workspace.join("local-reference-assets/appearance.2da"))
            .expect("exact full runtime appearance input");
        let packet = fs::read(workspace.join(&negative.runtime_packet.informational_path))
            .expect("exact r30 runtime packet");
        let capture = fs::read(workspace.join(&negative.runtime_capture.informational_path))
            .expect("exact r30 runtime capture");

        let source_path = workspace
            .join(&negative.source_glb.informational_path)
            .canonicalize()
            .expect("canonical exact r30 source path");
        let profile = declare_m0_direct_creature_runtime_profile_v2(
            &source,
            source_path.to_string_lossy(),
            SourceTopologyOriginV1::UserDerived,
            "m2a_m0p01",
        )
        .expect("caller-owned r30 V2 profile");
        let identity = BinaryM0VerticalSliceIdentityV1 {
            module_resref: "m2a_m0r30".to_owned(),
            area_resref: "m2a_m0a30".to_owned(),
            hak_resref: "m2a_m0r30".to_owned(),
        };
        let replayed = build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2(
            &source,
            &full_appearance,
            &identity,
            &profile,
        )
        .expect("fully replayed r30 V2 package");
        assert_eq!(replayed.proof_module, module, "frozen r30 MOD replay");
        assert_eq!(replayed.hak, hak, "frozen r30 HAK replay");
        assert_eq!(replayed.model, mdl, "frozen r30 MDL replay");

        let scene = inspect_binary_m0_vertical_slice_module_v1(&module)
            .expect("parse frozen exact r30 MOD");
        assert_eq!(scene.module_resref, "m2a_m0r30");
        assert_eq!(scene.area_resref, "m2a_m0a30");
        assert_eq!(scene.ordered_hak_resrefs, ["m2a_m0r30"]);
        let archive = ErfArchive::parse(&hak).expect("parse frozen exact r30 HAK");
        assert_eq!(
            archive.find("m2a_m0p01", 2002).expect("ordered HAK MDL"),
            mdl
        );

        let contract = replayed
            .summary
            .m0_runtime_fixture_contract
            .as_ref()
            .expect("production V2 runtime contract");
        crate::model_pipeline::verify_m0_binary_runtime_fixture_contract_v2(
            contract, &profile, &source, &module, &hak,
        )
        .expect("frozen r30 MOD -> ordered HAK -> source replay contract");
        let topology = inspect_m0_source_topology_binding_v1(&source, &mdl, &profile)
            .expect("independent r30 source-topology readback");
        assert_eq!(topology, contract.source_topology_binding);
        let (envelope, envelope_sha) = crate::mdl::inspect_direct_creature_engine_envelope_v1(&mdl)
            .expect("r30 retains a structurally valid engine envelope");
        assert_eq!(envelope.verdict, negative.structural_verdict);
        assert_eq!(envelope, contract.engine_envelope);
        assert_eq!(envelope_sha, contract.engine_envelope_sha256);
        assert_eq!(
            contract.source_topology_binding.topology_summary_sha256,
            "efb39972365d58f093c8d77a23a90595eaf956b8a2777d3d46c1949ba084df21",
            "frozen r30 topology digest"
        );
        assert_eq!(
            contract.engine_envelope_sha256,
            "2b99838587f1a7ec60cafc3edd01402941620710c1a4102a46e047e955e57c17",
            "frozen r30 engine envelope digest"
        );

        let candidate = runtime_candidate_from_contract_v1(contract)
            .expect("full r30 runtime candidate identity");
        assert_eq!(
            runtime_candidate_digest_v1(&candidate).expect("canonical r30 candidate digest"),
            R30_RUNTIME_CANDIDATE_SHA256,
            "frozen r30 full candidate digest"
        );
        let log_window = inspect_log_window_v1(
            candidate,
            &["m2a_m0r30".to_owned(), "m2a_m0p01".to_owned()],
            process(),
            process(),
            snapshot(
                "nwclientLog1.txt",
                "pure-parser-client",
                "2026-07-21T22:01:01Z",
                b"",
            ),
            snapshot(
                "nwclientLog1.txt",
                "pure-parser-client",
                "2026-07-21T22:01:02Z",
                b"",
            ),
            snapshot(
                "nwengineLog.txt",
                "pure-parser-engine",
                "2026-07-21T22:01:01Z",
                b"",
            ),
            snapshot(
                "nwengineLog.txt",
                "pure-parser-engine",
                "2026-07-21T22:01:02Z",
                b"",
            ),
            "2026-07-21T22:01:03Z",
        )
        .expect("diagnostic-only r30 log window");
        let contract_bytes = serde_json::to_vec(contract).expect("exact V2 contract bytes");
        let sidecar = verify_known_r30_runtime_negative_v1(
            &contract_bytes,
            &profile,
            &source,
            &module,
            &hak,
            log_window.clone(),
            &packet,
            &capture,
        )
        .expect("registry-pinned exact r30 not_visible/failed observation");
        assert_eq!(
            sidecar.model_visibility,
            RuntimeModelVisibilityV1::NotVisible
        );
        assert_eq!(
            sidecar.proof_completeness,
            RuntimeProofCompletenessV1::Failed
        );

        let replacement_capture = b"arbitrary caller replacement capture";
        let replacement_capture_path = "test://caller-replacement-capture.png";
        let replacement_capture_sha = sha256_bytes(replacement_capture);
        let mut replacement_packet: serde_json::Value =
            serde_json::from_slice(&packet).expect("exact r30 packet JSON");
        replacement_packet["modelVisibility"] = serde_json::json!({
            "status": "visible",
            "blank": false,
            "placeholder": false
        });
        replacement_packet["proofCompleteness"] = serde_json::json!("verified");
        replacement_packet["visualInspection"]["captureSha256"] =
            serde_json::json!(replacement_capture_sha);
        replacement_packet["visualInspection"]["modelVisibility"] = serde_json::json!("visible");
        replacement_packet["visualInspection"]["proofCompleteness"] = serde_json::json!("verified");
        replacement_packet["visualInspection"]["finding"] =
            serde_json::json!("caller claims the exact candidate is visible");
        let replacement_artifact = replacement_packet["artifacts"]
            .as_array_mut()
            .expect("runtime artifacts")
            .iter_mut()
            .find(|artifact| artifact["kind"] == "runtime-capture")
            .expect("runtime capture artifact");
        replacement_artifact["path"] = serde_json::json!(replacement_capture_path);
        replacement_artifact["sha256"] = serde_json::json!(replacement_capture_sha);
        let replacement_packet =
            serde_json::to_vec(&replacement_packet).expect("recomputed replacement packet");
        let caller_claim = inspect_diagnostic_runtime_observation_claim_v1(
            "test://caller-replacement-packet.json",
            &replacement_packet,
            replacement_capture_path,
            replacement_capture,
        )
        .expect("same-caller packet/capture may form only a diagnostic claim");
        assert!(caller_claim.diagnostic_only);
        assert_eq!(
            caller_claim.model_visibility,
            RuntimeModelVisibilityV1::Visible
        );
        assert_eq!(
            caller_claim.proof_completeness,
            RuntimeProofCompletenessV1::Verified
        );
        let error = verify_known_r30_runtime_negative_v1(
            &contract_bytes,
            &profile,
            &source,
            &module,
            &hak,
            log_window,
            &replacement_packet,
            replacement_capture,
        )
        .expect_err("caller-minted visible/verified claim must not replace the r30 registry pin");
        assert_eq!(error.code, "M2A-RUNTIME-SIDECAR-PINNED-IDENTITY");
    }
}
