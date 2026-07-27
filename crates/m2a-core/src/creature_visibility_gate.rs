//! Fail-closed offline gate for the frozen H2 r43 owner proof.
//!
//! This module never starts Aurora Toolset or NWN and never writes native
//! files. It verifies caller-supplied bytes, reads the exact frozen scene and
//! classifies an owner-supplied visual result without allowing a broken
//! synthetic control to erase an exact, judgeable owner verdict for H2.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animated_donor_candidate::{
        H2_R43_APPEARANCE_ROW, H2_R43_AREA_RESREF, H2_R43_FIXTURE_ID, H2_R43_HAK_RESREF,
        H2_R43_HAK_SHA256, H2_R43_MODULE_RESREF, H2_R43_MODULE_SHA256,
        H2_R43_STOCK_CONTROL_APPEARANCE_ROW, H2_R43_STOCK_CONTROL_ID,
    },
    proof_module::{
        BinaryCreatureOwnedFixtureV1, M0RuntimeDirectionV1, M0RuntimePositionV1,
        inspect_binary_creature_multi_fixture_module_v1,
    },
    runtime_evidence::RuntimeProofCompletenessV1,
};

const H2_R43_MODULE_BYTE_LENGTH: u64 = 20_280;
const H2_R43_HAK_BYTE_LENGTH: u64 = 20_084_361;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureVisibilityGateErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for CreatureVisibilityGateErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for CreatureVisibilityGateErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R43InstalledArtifactV1 {
    pub resref: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R43InstalledLineageReportV1 {
    pub schema_version: u32,
    pub profile: String,
    pub module: H2R43InstalledArtifactV1,
    pub hak: H2R43InstalledArtifactV1,
    pub source_matches_installed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R43OwnerProofFixtureV1 {
    pub id: String,
    pub appearance_row: u16,
    pub position: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R43OwnerProofSceneV1 {
    pub schema_version: u32,
    pub profile: String,
    pub module_resref: String,
    pub area_resref: String,
    pub ordered_hak_resrefs: Vec<String>,
    pub entry_position: [f32; 3],
    pub entry_direction: [f32; 2],
    pub h2_fixture: H2R43OwnerProofFixtureV1,
    pub stock_control: H2R43OwnerProofFixtureV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureVisualStateV1 {
    VisibleCorrect,
    VisibleCorrupt,
    NotVisible,
    NotTested,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct H2R43OwnerObservationV1 {
    pub exact_candidate_bound: bool,
    pub capture_judgeable: bool,
    pub proof_completeness: RuntimeProofCompletenessV1,
    pub stock_control: CreatureVisualStateV1,
    pub h2_candidate: CreatureVisualStateV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum H2R43DiagnosticVerdictV1 {
    ProofIncomplete,
    R42OuterRootConfirmed,
    CandidateVisibleStockControlFailed,
    DrawAcceptedTransformCorrupt,
    CustomModelRejected,
    CustomModelRejectedControlUntrusted,
    SceneOrInstantiationUnisolated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum H2R43NextActionV1 {
    CompleteSameCandidateProof,
    ContinueToSkinDeformation,
    DiagnoseStockControlInstantiation,
    DiagnoseHierarchyTransform,
    DiagnoseCustomMdlMdx,
    DiagnoseCustomMdlMdxAndRepairControl,
    RunExistingStockOnlyVan02,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R43OwnerGateDecisionV1 {
    pub schema_version: u32,
    pub profile: String,
    pub verdict: H2R43DiagnosticVerdictV1,
    pub next_action: H2R43NextActionV1,
    pub model_iteration_admitted: bool,
    pub visibility_gate_closed: bool,
}

/// Verifies the exact frozen source and caller-supplied native readback bytes.
///
/// The function is intentionally byte-only. Filesystem resolution, no-clobber
/// copying and OS attestation belong to the caller; this core gate proves that
/// both sides carry the one admitted r43 identity.
pub fn verify_h2_r43_installed_lineage_v1(
    source_module: &[u8],
    installed_module: &[u8],
    source_hak: &[u8],
    installed_hak: &[u8],
) -> Result<H2R43InstalledLineageReportV1, CreatureVisibilityGateErrorV1> {
    require_artifact(
        source_module,
        H2_R43_MODULE_BYTE_LENGTH,
        H2_R43_MODULE_SHA256,
        "source.module",
        "H2-R43-SOURCE-LINEAGE-MISMATCH",
    )?;
    require_artifact(
        source_hak,
        H2_R43_HAK_BYTE_LENGTH,
        H2_R43_HAK_SHA256,
        "source.hak",
        "H2-R43-SOURCE-LINEAGE-MISMATCH",
    )?;
    require_artifact(
        installed_module,
        H2_R43_MODULE_BYTE_LENGTH,
        H2_R43_MODULE_SHA256,
        "installed.module",
        "H2-R43-INSTALLED-LINEAGE-MISMATCH",
    )?;
    require_artifact(
        installed_hak,
        H2_R43_HAK_BYTE_LENGTH,
        H2_R43_HAK_SHA256,
        "installed.hak",
        "H2-R43-INSTALLED-LINEAGE-MISMATCH",
    )?;
    if source_module != installed_module {
        return Err(gate_error(
            "H2-R43-INSTALLED-LINEAGE-MISMATCH",
            "installed.module",
            "installed module bytes differ from the exact frozen source",
        ));
    }
    if source_hak != installed_hak {
        return Err(gate_error(
            "H2-R43-INSTALLED-LINEAGE-MISMATCH",
            "installed.hak",
            "installed HAK bytes differ from the exact frozen source",
        ));
    }

    Ok(H2R43InstalledLineageReportV1 {
        schema_version: 1,
        profile: "H2_R43_INSTALLED_LINEAGE_V1".to_owned(),
        module: artifact(H2_R43_MODULE_RESREF, installed_module),
        hak: artifact(H2_R43_HAK_RESREF, installed_hak),
        source_matches_installed: true,
    })
}

/// Reads and pins every scene field needed by the human-owned r43 proof.
pub fn verify_h2_r43_owner_proof_scene_v1(
    module: &[u8],
) -> Result<H2R43OwnerProofSceneV1, CreatureVisibilityGateErrorV1> {
    require_artifact(
        module,
        H2_R43_MODULE_BYTE_LENGTH,
        H2_R43_MODULE_SHA256,
        "module",
        "H2-R43-OWNER-SCENE-MISMATCH",
    )?;
    let scene = inspect_binary_creature_multi_fixture_module_v1(module).map_err(|error| {
        gate_error(
            "H2-R43-OWNER-SCENE-MISMATCH",
            format!("module.{}", error.path),
            error.message,
        )
    })?;
    require_scene(
        scene.module_resref == H2_R43_MODULE_RESREF,
        "module.moduleResref",
        "module resref differs from exact r43",
    )?;
    require_scene(
        scene.area_resref == H2_R43_AREA_RESREF,
        "module.areaResref",
        "Area resref differs from exact r43",
    )?;
    require_scene(
        scene.ordered_hak_resrefs == [H2_R43_HAK_RESREF],
        "module.orderedHakResrefs",
        "ordered HAK list differs from exact r43",
    )?;
    require_scene(
        scene.entry_position
            == (M0RuntimePositionV1 {
                x: 10.0,
                y: 10.0,
                z: 0.0,
            }),
        "module.entryPosition",
        "player entry differs from exact r43",
    )?;
    require_scene(
        scene.entry_direction == (M0RuntimeDirectionV1 { x: 0.0, y: 1.0 }),
        "module.entryDirection",
        "player direction differs from exact r43",
    )?;
    require_scene(
        scene.fixtures.len() == 2,
        "module.fixtures",
        "exact r43 requires exactly one H2 fixture and one stock control",
    )?;

    let h2 = exact_fixture(
        &scene.fixtures,
        H2_R43_FIXTURE_ID,
        H2_R43_APPEARANCE_ROW,
        M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        "module.fixtures.h2",
    )?;
    let stock = exact_fixture(
        &scene.fixtures,
        H2_R43_STOCK_CONTROL_ID,
        H2_R43_STOCK_CONTROL_APPEARANCE_ROW,
        M0RuntimePositionV1 {
            x: 7.0,
            y: 14.5,
            z: 0.0,
        },
        "module.fixtures.stockControl",
    )?;

    Ok(H2R43OwnerProofSceneV1 {
        schema_version: 1,
        profile: "H2_R43_OWNER_PROOF_SCENE_V1".to_owned(),
        module_resref: scene.module_resref,
        area_resref: scene.area_resref,
        ordered_hak_resrefs: scene.ordered_hak_resrefs,
        entry_position: position(scene.entry_position),
        entry_direction: direction(scene.entry_direction),
        h2_fixture: proof_fixture(h2),
        stock_control: proof_fixture(stock),
    })
}

/// Classifies the human-owned proof without broadening the model-iteration
/// gate. An exact-candidate-bound, judgeable H2 absence admits model iteration
/// even if a synthetic side control is corrupt, absent, or not classified. A
/// failed side control remains a separate harness defect; it cannot downgrade
/// H2 from `not_visible` to `not_tested`. A failed proof packet likewise does
/// not erase the owner visual verdict; a missing or unjudgeable H2 observation
/// does.
pub fn classify_h2_r43_owner_observation_v1(
    observation: &H2R43OwnerObservationV1,
) -> H2R43OwnerGateDecisionV1 {
    let incomplete = !observation.exact_candidate_bound
        || !observation.capture_judgeable
        || observation.proof_completeness == RuntimeProofCompletenessV1::Missing
        || observation.h2_candidate == CreatureVisualStateV1::NotTested;
    if incomplete {
        return decision(
            H2R43DiagnosticVerdictV1::ProofIncomplete,
            H2R43NextActionV1::CompleteSameCandidateProof,
            false,
            false,
        );
    }

    match observation.h2_candidate {
        CreatureVisualStateV1::VisibleCorrect => {
            if observation.stock_control != CreatureVisualStateV1::VisibleCorrect {
                decision(
                    H2R43DiagnosticVerdictV1::CandidateVisibleStockControlFailed,
                    H2R43NextActionV1::DiagnoseStockControlInstantiation,
                    false,
                    true,
                )
            } else {
                decision(
                    H2R43DiagnosticVerdictV1::R42OuterRootConfirmed,
                    H2R43NextActionV1::ContinueToSkinDeformation,
                    false,
                    true,
                )
            }
        }
        CreatureVisualStateV1::VisibleCorrupt => decision(
            H2R43DiagnosticVerdictV1::DrawAcceptedTransformCorrupt,
            H2R43NextActionV1::DiagnoseHierarchyTransform,
            false,
            true,
        ),
        CreatureVisualStateV1::NotVisible => match observation.stock_control {
            CreatureVisualStateV1::VisibleCorrect => decision(
                H2R43DiagnosticVerdictV1::CustomModelRejected,
                H2R43NextActionV1::DiagnoseCustomMdlMdx,
                true,
                false,
            ),
            CreatureVisualStateV1::VisibleCorrupt
            | CreatureVisualStateV1::NotVisible
            | CreatureVisualStateV1::NotTested => decision(
                H2R43DiagnosticVerdictV1::CustomModelRejectedControlUntrusted,
                H2R43NextActionV1::DiagnoseCustomMdlMdxAndRepairControl,
                true,
                false,
            ),
        },
        CreatureVisualStateV1::NotTested => unreachable!("handled by incomplete proof gate"),
    }
}

fn exact_fixture<'a>(
    fixtures: &'a [BinaryCreatureOwnedFixtureV1],
    id: &str,
    appearance_row: u16,
    expected_position: M0RuntimePositionV1,
    path: &str,
) -> Result<&'a BinaryCreatureOwnedFixtureV1, CreatureVisibilityGateErrorV1> {
    let fixture = fixtures
        .iter()
        .find(|fixture| fixture.id == id)
        .ok_or_else(|| {
            gate_error(
                "H2-R43-OWNER-SCENE-MISMATCH",
                path,
                format!("required fixture {id} is absent"),
            )
        })?;
    require_scene(
        fixture.appearance_row == appearance_row,
        format!("{path}.appearanceRow"),
        "fixture appearance row differs from exact r43",
    )?;
    require_scene(
        fixture.position == expected_position,
        format!("{path}.position"),
        "fixture position differs from exact r43",
    )?;
    require_scene(
        fixture.orientation == (M0RuntimeDirectionV1 { x: 1.0, y: 0.0 }),
        format!("{path}.orientation"),
        "fixture orientation differs from exact r43",
    )?;
    Ok(fixture)
}

fn require_artifact(
    bytes: &[u8],
    expected_byte_length: u64,
    expected_sha256: &str,
    path: &str,
    code: &str,
) -> Result<(), CreatureVisibilityGateErrorV1> {
    let actual_sha256 = sha256(bytes);
    if bytes.len() as u64 != expected_byte_length || actual_sha256 != expected_sha256 {
        return Err(gate_error(
            code,
            path,
            format!(
                "expected {expected_byte_length} bytes and SHA-256 {expected_sha256}, got {} bytes and {actual_sha256}",
                bytes.len()
            ),
        ));
    }
    Ok(())
}

fn require_scene(
    condition: bool,
    path: impl Into<String>,
    message: impl Into<String>,
) -> Result<(), CreatureVisibilityGateErrorV1> {
    if condition {
        Ok(())
    } else {
        Err(gate_error("H2-R43-OWNER-SCENE-MISMATCH", path, message))
    }
}

fn artifact(resref: &str, bytes: &[u8]) -> H2R43InstalledArtifactV1 {
    H2R43InstalledArtifactV1 {
        resref: resref.to_owned(),
        byte_length: bytes.len() as u64,
        sha256: sha256(bytes),
    }
}

fn proof_fixture(fixture: &BinaryCreatureOwnedFixtureV1) -> H2R43OwnerProofFixtureV1 {
    H2R43OwnerProofFixtureV1 {
        id: fixture.id.clone(),
        appearance_row: fixture.appearance_row,
        position: position(fixture.position),
    }
}

fn position(value: M0RuntimePositionV1) -> [f32; 3] {
    [value.x, value.y, value.z]
}

fn direction(value: M0RuntimeDirectionV1) -> [f32; 2] {
    [value.x, value.y]
}

fn decision(
    verdict: H2R43DiagnosticVerdictV1,
    next_action: H2R43NextActionV1,
    model_iteration_admitted: bool,
    visibility_gate_closed: bool,
) -> H2R43OwnerGateDecisionV1 {
    H2R43OwnerGateDecisionV1 {
        schema_version: 1,
        profile: "H2_R43_OWNER_GATE_DECISION_V1".to_owned(),
        verdict,
        next_action,
        model_iteration_admitted,
        visibility_gate_closed,
    }
}

fn gate_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> CreatureVisibilityGateErrorV1 {
    CreatureVisibilityGateErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
