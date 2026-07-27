//! Offline materialization contract for a one-variable Hook Horror control.
//!
//! The owner-provided Last City `appearance.2da` is retained byte-for-byte as
//! a prefix. Physical row 102 is appended with all 35 cells cloned and only
//! `LABEL` changed. The HAK contains no retail model payload: it contains only
//! the resulting `appearance.2da`, whose unchanged `RACE=c_horror` resolves
//! the stock game model at runtime.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureMultiFixtureModuleReadbackV1,
        BinaryCreatureOwnedFixtureV1, BinaryM0VerticalSliceIdentityV1, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_multi_fixture_module_v1,
    },
    two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        TwoDaRowReadbackV1, append_two_da_row_v1, inspect_two_da_v2, read_two_da_row_v2,
    },
};

pub const HOOK_HORROR_CLONE_PROFILE_V1: &str = "M2A_HOOK_HORROR_LABEL_CLONE_DIAGNOSTIC_V1";
pub const HOOK_HORROR_SOURCE_ROW_V1: u32 = 102;
pub const HOOK_HORROR_CLONE_LABEL_V1: &str = "M2A Hook Horror clone control";
pub const LAST_CITY_APPEARANCE_LENGTH_V1: u64 = 7_655_336;
pub const LAST_CITY_APPEARANCE_SHA256_V1: &str =
    "ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HookHorrorCloneByteIdentityV1 {
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HookHorrorCloneDiagnosticContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub diagnostic_only: bool,
    pub source_appearance: HookHorrorCloneByteIdentityV1,
    pub output_appearance: HookHorrorCloneByteIdentityV1,
    pub source_prefix_preserved: bool,
    pub source_row: u32,
    pub clone_row: u16,
    pub source_row_readback: TwoDaRowReadbackV1,
    pub clone_row_readback: TwoDaRowReadbackV1,
    pub changed_columns: Vec<String>,
    pub module: HookHorrorCloneByteIdentityV1,
    pub hak: HookHorrorCloneByteIdentityV1,
    pub module_readback: BinaryCreatureMultiFixtureModuleReadbackV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HookHorrorCloneDiagnosticArtifactV1 {
    pub module: Vec<u8>,
    pub hak: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub contract: HookHorrorCloneDiagnosticContractV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookHorrorCloneDiagnosticErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for HookHorrorCloneDiagnosticErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for HookHorrorCloneDiagnosticErrorV1 {}

pub fn build_hook_horror_clone_diagnostic_v1(
    source_appearance: &[u8],
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<HookHorrorCloneDiagnosticArtifactV1, HookHorrorCloneDiagnosticErrorV1> {
    require_exact_source(source_appearance)?;
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(source_appearance, &limits).map_err(map_two_da)?;
    if inspection.columns.len() != 35 || inspection.physical_row_count != 15_219 {
        return Err(error(
            "M2A-HOOK-HORROR-SOURCE-SCHEMA",
            "appearance",
            "expected the exact 35-column, 15219-row Last City appearance table",
        ));
    }
    let source_row = read_two_da_row_v2(source_appearance, HOOK_HORROR_SOURCE_ROW_V1, &limits)
        .map_err(map_two_da)?;
    require_hook_horror_identity(&inspection.columns, &source_row)?;

    let cells = inspection
        .columns
        .iter()
        .zip(&source_row.cells)
        .map(|(column_name, value)| TwoDaCellAssignmentV1 {
            column_name: column_name.clone(),
            value: if column_name.eq_ignore_ascii_case("LABEL") {
                TwoDaCellValueV1::Text {
                    value: HOOK_HORROR_CLONE_LABEL_V1.to_owned(),
                }
            } else {
                value.clone()
            },
        })
        .collect::<Vec<_>>();
    let append = append_two_da_row_v1(
        source_appearance,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &limits,
    )
    .map_err(map_two_da)?;
    let clone_row = read_two_da_row_v2(
        &append.payload,
        u32::from(append.report.appended_row_index),
        &limits,
    )
    .map_err(map_two_da)?;
    require_only_label_changed(&inspection.columns, &source_row, &clone_row)?;

    let hak = write_hak_v1(
        &[HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: append.payload.clone(),
        }],
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| error("M2A-HOOK-HORROR-HAK-WRITE", "hak", source.to_string()))?;
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: identity.module_resref.clone(),
            area_resref: identity.area_resref.clone(),
            hak_resref: identity.hak_resref.clone(),
        },
        &[BinaryCreatureOwnedFixtureV1 {
            id: "hook_horror_clone".to_owned(),
            template_resref: "m2a_vhook".to_owned(),
            display_name: HOOK_HORROR_CLONE_LABEL_V1.to_owned(),
            appearance_row: append.report.appended_row_index,
            position: M0RuntimePositionV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        }],
    )
    .map_err(|source| error("M2A-HOOK-HORROR-MODULE-WRITE", "module", source.to_string()))?;

    Ok(HookHorrorCloneDiagnosticArtifactV1 {
        module: module.payload.clone(),
        hak: hak.payload.clone(),
        appearance_two_da: append.payload.clone(),
        contract: HookHorrorCloneDiagnosticContractV1 {
            schema_version: 1,
            profile: HOOK_HORROR_CLONE_PROFILE_V1.to_owned(),
            diagnostic_only: true,
            source_appearance: identity_of(source_appearance),
            output_appearance: identity_of(&append.payload),
            source_prefix_preserved: append.payload.starts_with(source_appearance),
            source_row: HOOK_HORROR_SOURCE_ROW_V1,
            clone_row: append.report.appended_row_index,
            source_row_readback: source_row,
            clone_row_readback: clone_row,
            changed_columns: vec!["LABEL".to_owned()],
            module: identity_of(&module.payload),
            hak: identity_of(&hak.payload),
            module_readback: module.readback,
        },
    })
}

pub fn verify_hook_horror_clone_diagnostic_v1(
    artifact: &HookHorrorCloneDiagnosticArtifactV1,
    source_appearance: &[u8],
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<(), HookHorrorCloneDiagnosticErrorV1> {
    let replay = build_hook_horror_clone_diagnostic_v1(source_appearance, identity)?;
    if artifact != &replay {
        return Err(error(
            "M2A-HOOK-HORROR-DETERMINISTIC-REPLAY",
            "artifact",
            "artifact bytes or contract differ from deterministic replay",
        ));
    }
    Ok(())
}

fn require_exact_source(bytes: &[u8]) -> Result<(), HookHorrorCloneDiagnosticErrorV1> {
    if bytes.len() as u64 != LAST_CITY_APPEARANCE_LENGTH_V1
        || sha256(bytes) != LAST_CITY_APPEARANCE_SHA256_V1
    {
        return Err(error(
            "M2A-HOOK-HORROR-SOURCE-IDENTITY",
            "appearance",
            "source appearance bytes do not match the pinned owner-provided Last City table",
        ));
    }
    Ok(())
}

fn require_hook_horror_identity(
    columns: &[String],
    row: &TwoDaRowReadbackV1,
) -> Result<(), HookHorrorCloneDiagnosticErrorV1> {
    for (column, expected) in [
        ("LABEL", Some("Hook Horror")),
        ("STRING_REF", None),
        ("NAME", Some("Hook_Horror")),
        ("RACE", Some("c_horror")),
        ("MODELTYPE", Some("S")),
    ] {
        let index = columns
            .iter()
            .position(|actual| actual.eq_ignore_ascii_case(column))
            .ok_or_else(|| error("M2A-HOOK-HORROR-SOURCE-COLUMN", column, "column missing"))?;
        let expected_cell = match expected {
            Some(value) => TwoDaCellValueV1::Text {
                value: value.to_owned(),
            },
            None => TwoDaCellValueV1::Null,
        };
        if row.cells.get(index) != Some(&expected_cell) {
            return Err(error(
                "M2A-HOOK-HORROR-SOURCE-ROW",
                format!("appearance.rows[102].{column}"),
                format!("expected {expected:?}"),
            ));
        }
    }
    Ok(())
}

fn require_only_label_changed(
    columns: &[String],
    source: &TwoDaRowReadbackV1,
    clone: &TwoDaRowReadbackV1,
) -> Result<(), HookHorrorCloneDiagnosticErrorV1> {
    if source.cells.len() != 35 || clone.cells.len() != 35 {
        return Err(error(
            "M2A-HOOK-HORROR-CLONE-WIDTH",
            "appearance.cloneRow",
            "source and clone must each expose all 35 cells",
        ));
    }
    for (index, (before, after)) in source.cells.iter().zip(&clone.cells).enumerate() {
        let column = &columns[index];
        let expected = if column.eq_ignore_ascii_case("LABEL") {
            TwoDaCellValueV1::Text {
                value: HOOK_HORROR_CLONE_LABEL_V1.to_owned(),
            }
        } else {
            before.clone()
        };
        if *after != expected {
            return Err(error(
                "M2A-HOOK-HORROR-CLONE-DIFF",
                format!("appearance.cloneRow.{column}"),
                "clone differs from source outside the one allowed LABEL delta",
            ));
        }
    }
    Ok(())
}

fn identity_of(bytes: &[u8]) -> HookHorrorCloneByteIdentityV1 {
    HookHorrorCloneByteIdentityV1 {
        byte_length: bytes.len() as u64,
        sha256: sha256(bytes),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn map_two_da(source: crate::two_da::TwoDaError) -> HookHorrorCloneDiagnosticErrorV1 {
    let message = source.to_string();
    error("M2A-HOOK-HORROR-2DA", source.path, message)
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> HookHorrorCloneDiagnosticErrorV1 {
    HookHorrorCloneDiagnosticErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
