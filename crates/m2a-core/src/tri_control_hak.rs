//! Offline-only HAK/appearance core for a three-fixture runtime diagnostic.
//!
//! This module emits no MOD, installs nothing, starts no process and cannot
//! assert resolver or renderer success. It is exported so callers and tests use
//! the same production implementation, but it is not an admission or live API.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::{ErfArchive, ErfFileType},
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        append_two_da_row_v1, inspect_two_da_v2, read_two_da_row_v2,
    },
};

pub const TRI_CONTROL_HAK_CONTRACT_V1: &str = "M2A_PROJECT_TRI_CONTROL_HAK_DIAGNOSTIC_V1";
pub const TRI_CONTROL_HAK_CONTRACT_V2: &str = "M2A_PROJECT_TRI_CONTROL_HAK_LAST_CITY_DIAGNOSTIC_V2";
pub const R31_MODEL_RESREF: &str = "m2a_m0p01";
pub const R31_TEXTURE_RESREF: &str = "m2a_m0t01";
pub const H1_MODEL_RESREF: &str = "m2a_m6p01";
pub const H1_TEXTURE_RESREF: &str = "m2a_m6t01";
pub const TRI_CONTROL_RESOURCE_ORDER_V1: [(&str, u16); 5] = [
    ("appearance", 2017),
    (R31_MODEL_RESREF, 2002),
    (R31_TEXTURE_RESREF, 3),
    (H1_MODEL_RESREF, 2002),
    (H1_TEXTURE_RESREF, 3),
];
pub const LAST_CITY_SOURCE_PHYSICAL_ROWS_V2: u32 = 15_219;
pub const LAST_CITY_M0_APPEARANCE_ROW_V2: u32 = 15_219;
pub const LAST_CITY_H1_APPEARANCE_ROW_V2: u32 = 15_220;
pub const LAST_CITY_OUTPUT_PHYSICAL_ROWS_V2: u32 = 15_221;

const BASE_APPEARANCE_LENGTH: u64 = 6_901_169;
const BASE_APPEARANCE_SHA256: &str =
    "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const R31_APPEARANCE_LENGTH: u64 = 6_901_360;
const R31_APPEARANCE_SHA256: &str =
    "48d313b75761809e2231c99ad51e17d67632c1ce10e70b87fa8b1ccdfbc474a6";
const R31_MDL_LENGTH: u64 = 151_328;
const R31_MDL_SHA256: &str = "fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6";
const R31_TGA_LENGTH: u64 = 12_582_956;
const R31_TGA_SHA256: &str = "079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b";
const H1_APPEARANCE_LENGTH: u64 = 6_901_354;
const H1_APPEARANCE_SHA256: &str =
    "e50e2f8c5fc42771f5c433dd3984b0f8e740938d6f722adf838942f1f6342f8e";
const H1_MDL_LENGTH: u64 = 557_268;
const H1_MDL_SHA256: &str = "6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd";
const H1_TGA_LENGTH: u64 = 12_582_956;
const H1_TGA_SHA256: &str = "ab8f11c7f448801d905594802a223a1ed5969bc5d09ce06e6588cbe83b6a86b4";
const LAST_CITY_APPEARANCE_LENGTH_V2: u64 = 7_655_336;
const LAST_CITY_APPEARANCE_SHA256_V2: &str =
    "ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2";
const LAST_CITY_OUTPUT_APPEARANCE_LENGTH_V2: u64 = 7_655_712;
const LAST_CITY_OUTPUT_APPEARANCE_SHA256_V2: &str =
    "f45437470d554d7ecd5e8fef15db0bbd799621308d625fad388950cf5f276c43";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlByteIdentityV1 {
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlResourceKeyV1 {
    pub resref: String,
    pub resource_type: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlResourceIdentityV1 {
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlAppearanceBindingV1 {
    pub physical_row: u32,
    pub label: String,
    pub model_type: String,
    pub race: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlHakContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub diagnostic_only: bool,
    pub runtime_admissible: bool,
    pub resolver_verified: bool,
    pub renderer_verified: bool,
    pub base_appearance: TriControlByteIdentityV1,
    pub h1_source_appearance: TriControlByteIdentityV1,
    pub appearance: TriControlByteIdentityV1,
    pub base_control: TriControlAppearanceBindingV1,
    pub r31_control: TriControlAppearanceBindingV1,
    pub h1_control: TriControlAppearanceBindingV1,
    pub resource_order: Vec<TriControlResourceKeyV1>,
    pub resources: Vec<TriControlResourceIdentityV1>,
    pub hak: TriControlByteIdentityV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriControlHakArtifactV1 {
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub contract: TriControlHakContractV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlPreservedAppearanceRowV2 {
    pub physical_row: u32,
    pub printed_row_label: u32,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlHakContractV2 {
    pub schema_version: u32,
    pub profile: String,
    pub diagnostic_only: bool,
    pub runtime_admissible: bool,
    pub resolver_verified: bool,
    pub renderer_verified: bool,
    pub base_appearance: TriControlByteIdentityV1,
    pub source_physical_rows: u32,
    pub output_physical_rows: u32,
    pub source_prefix_preserved: bool,
    pub preserved_project_q_rows: Vec<TriControlPreservedAppearanceRowV2>,
    pub h1_source_appearance: TriControlByteIdentityV1,
    pub appearance: TriControlByteIdentityV1,
    pub base_control: TriControlAppearanceBindingV1,
    pub r31_control: TriControlAppearanceBindingV1,
    pub h1_control: TriControlAppearanceBindingV1,
    pub resource_order: Vec<TriControlResourceKeyV1>,
    pub resources: Vec<TriControlResourceIdentityV1>,
    pub hak: TriControlByteIdentityV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriControlHakArtifactV2 {
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub contract: TriControlHakContractV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriControlHakErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for TriControlHakErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TriControlHakErrorV1 {}

pub fn build_tri_control_hak_v1(
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<TriControlHakArtifactV1, TriControlHakErrorV1> {
    require_pinned_inputs(
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )?;

    let r31_append = append_two_da_row_v1(
        base_appearance,
        &direct_creature_row("M2A_M0_MESHY_RIGID", R31_MODEL_RESREF, base_appearance)?,
        &TwoDaLimitsV1::default(),
    )
    .map_err(map_two_da)?;
    if r31_append.report.appended_row_index != 15_100
        || !r31_append.report.source_prefix_preserved
        || r31_append.payload.len() as u64 != R31_APPEARANCE_LENGTH
        || sha256(&r31_append.payload) != R31_APPEARANCE_SHA256
    {
        return Err(error(
            "M2A-TRI-CONTROL-R31-APPEND",
            "appearance.rows[15100]",
            "first append must reproduce the exact frozen r31 appearance mapping and bytes",
        ));
    }

    let h1_append = append_two_da_row_v1(
        &r31_append.payload,
        &clone_physical_row_request(h1_appearance, 15_100, &r31_append.payload)?,
        &TwoDaLimitsV1::default(),
    )
    .map_err(map_two_da)?;
    if h1_append.report.appended_row_index != 15_101 || !h1_append.report.source_prefix_preserved {
        return Err(error(
            "M2A-TRI-CONTROL-H1-APPEND",
            "appearance.rows[15101]",
            "H1 diagnostic row must be the sole append after exact r31 row15100",
        ));
    }
    let appearance = h1_append.payload;
    let base_control = require_binding(base_appearance, 102, None, "S", "c_horror")?;
    let r31_control = require_binding(
        &appearance,
        15_100,
        Some("M2A_M0_MESHY_RIGID"),
        "S",
        R31_MODEL_RESREF,
    )?;
    let h1_control = require_binding(
        &appearance,
        15_101,
        Some("M2A_M6_PROOF"),
        "S",
        H1_MODEL_RESREF,
    )?;

    let resources = resource_inputs(&appearance, r31_mdl, r31_tga, h1_mdl, h1_tga);
    let hak = write_hak_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|source| error(source.code, source.path, source.message))?;
    let resource_order = expected_resource_order();
    let resource_identities =
        expected_resource_payloads(&appearance, r31_mdl, r31_tga, h1_mdl, h1_tga);
    validate_archive(&hak.payload, &appearance, r31_mdl, r31_tga, h1_mdl, h1_tga)?;
    let contract = TriControlHakContractV1 {
        schema_version: 1,
        profile: TRI_CONTROL_HAK_CONTRACT_V1.to_owned(),
        diagnostic_only: true,
        runtime_admissible: false,
        resolver_verified: false,
        renderer_verified: false,
        base_appearance: identity(base_appearance),
        h1_source_appearance: identity(h1_appearance),
        appearance: identity(&appearance),
        base_control,
        r31_control,
        h1_control,
        resource_order,
        resources: resource_identities,
        hak: identity(&hak.payload),
    };
    Ok(TriControlHakArtifactV1 {
        appearance_two_da: appearance,
        hak: hak.payload,
        contract,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_tri_control_hak_v1(
    contract: &TriControlHakContractV1,
    hak: &[u8],
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlHakErrorV1> {
    if contract.schema_version != 1
        || contract.profile != TRI_CONTROL_HAK_CONTRACT_V1
        || !contract.diagnostic_only
        || contract.runtime_admissible
        || contract.resolver_verified
        || contract.renderer_verified
    {
        return Err(error(
            "M2A-TRI-CONTROL-CONTRACT",
            "contract",
            "tri-control output is diagnostic-only and cannot assert resolver, renderer or runtime admission",
        ));
    }
    let replay = build_tri_control_hak_v1(
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )?;
    if contract != &replay.contract {
        return Err(error(
            "M2A-TRI-CONTROL-CONTRACT",
            "contract",
            "contract differs from exact pinned-input deterministic replay",
        ));
    }
    validate_archive(
        hak,
        &replay.appearance_two_da,
        r31_mdl,
        r31_tga,
        h1_mdl,
        h1_tga,
    )?;
    if hak != replay.hak {
        return Err(error(
            "M2A-TRI-CONTROL-HAK-REPLAY",
            "hak",
            "HAK bytes differ from deterministic own-writer replay",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn build_tri_control_hak_last_city_v2(
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<TriControlHakArtifactV2, TriControlHakErrorV1> {
    require_pinned_last_city_inputs(
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )?;
    let limits = TwoDaLimitsV1::default();
    let source_inspection = inspect_two_da_v2(base_appearance, &limits).map_err(map_two_da)?;
    if source_inspection.physical_row_count != LAST_CITY_SOURCE_PHYSICAL_ROWS_V2 {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-ROWS",
            "baseAppearance.physicalRowCount",
            "Last City V2 requires exactly 15219 physical source rows",
        ));
    }
    let preserved_project_q_rows = vec![
        require_project_q_reserved_row(base_appearance, 15_100)?,
        require_project_q_reserved_row(base_appearance, 15_101)?,
    ];

    let r31_append = append_two_da_row_v1(
        base_appearance,
        &direct_creature_row("M2A_M0_MESHY_RIGID", R31_MODEL_RESREF, base_appearance)?,
        &limits,
    )
    .map_err(map_two_da)?;
    if u32::from(r31_append.report.appended_row_index) != LAST_CITY_M0_APPEARANCE_ROW_V2
        || r31_append.report.physical_rows_before != LAST_CITY_SOURCE_PHYSICAL_ROWS_V2
        || r31_append.report.physical_rows_after != LAST_CITY_SOURCE_PHYSICAL_ROWS_V2 + 1
        || !r31_append.report.source_prefix_preserved
    {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-M0-APPEND",
            "appearance.rows[15219]",
            "M0 must be the sole append after all 15219 Last City source rows",
        ));
    }
    let h1_append = append_two_da_row_v1(
        &r31_append.payload,
        &clone_physical_row_request(h1_appearance, 15_100, &r31_append.payload)?,
        &limits,
    )
    .map_err(map_two_da)?;
    let appearance = h1_append.payload;
    let output_inspection = inspect_two_da_v2(&appearance, &limits).map_err(map_two_da)?;
    if u32::from(h1_append.report.appended_row_index) != LAST_CITY_H1_APPEARANCE_ROW_V2
        || h1_append.report.physical_rows_before != LAST_CITY_SOURCE_PHYSICAL_ROWS_V2 + 1
        || h1_append.report.physical_rows_after != LAST_CITY_OUTPUT_PHYSICAL_ROWS_V2
        || !h1_append.report.source_prefix_preserved
        || output_inspection.physical_row_count != LAST_CITY_OUTPUT_PHYSICAL_ROWS_V2
    {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-H1-APPEND",
            "appearance.rows[15220]",
            "H1 must be the sole append after exact Last City row15219",
        ));
    }
    require_last_city_output_identity(&appearance)?;
    require_last_city_prefix(base_appearance, &appearance)?;
    for expected in &preserved_project_q_rows {
        if &require_project_q_reserved_row(&appearance, expected.physical_row)? != expected {
            return Err(error(
                "M2A-TRI-CONTROL-LAST-CITY-RESERVED",
                format!("appearance.rows[{}]", expected.physical_row),
                "Last City PROJECT_Q_RESERVED row changed during append",
            ));
        }
    }

    let base_control = require_binding(base_appearance, 102, None, "S", "c_horror")?;
    let r31_control = require_binding(
        &appearance,
        LAST_CITY_M0_APPEARANCE_ROW_V2,
        Some("M2A_M0_MESHY_RIGID"),
        "S",
        R31_MODEL_RESREF,
    )?;
    let h1_control = require_binding(
        &appearance,
        LAST_CITY_H1_APPEARANCE_ROW_V2,
        Some("M2A_M6_PROOF"),
        "S",
        H1_MODEL_RESREF,
    )?;
    let resources = resource_inputs(&appearance, r31_mdl, r31_tga, h1_mdl, h1_tga);
    let hak = write_hak_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|source| error(source.code, source.path, source.message))?;
    validate_archive_last_city_v2(
        &hak.payload,
        base_appearance,
        &appearance,
        r31_mdl,
        r31_tga,
        h1_mdl,
        h1_tga,
    )?;
    let contract = TriControlHakContractV2 {
        schema_version: 2,
        profile: TRI_CONTROL_HAK_CONTRACT_V2.to_owned(),
        diagnostic_only: true,
        runtime_admissible: false,
        resolver_verified: false,
        renderer_verified: false,
        base_appearance: identity(base_appearance),
        source_physical_rows: source_inspection.physical_row_count,
        output_physical_rows: output_inspection.physical_row_count,
        source_prefix_preserved: true,
        preserved_project_q_rows,
        h1_source_appearance: identity(h1_appearance),
        appearance: identity(&appearance),
        base_control,
        r31_control,
        h1_control,
        resource_order: expected_resource_order(),
        resources: expected_resource_payloads(&appearance, r31_mdl, r31_tga, h1_mdl, h1_tga),
        hak: identity(&hak.payload),
    };
    Ok(TriControlHakArtifactV2 {
        appearance_two_da: appearance,
        hak: hak.payload,
        contract,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_tri_control_hak_last_city_v2(
    contract: &TriControlHakContractV2,
    hak: &[u8],
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlHakErrorV1> {
    if contract.schema_version != 2
        || contract.profile != TRI_CONTROL_HAK_CONTRACT_V2
        || !contract.diagnostic_only
        || contract.runtime_admissible
        || contract.resolver_verified
        || contract.renderer_verified
        || !contract.source_prefix_preserved
    {
        return Err(error(
            "M2A-TRI-CONTROL-CONTRACT",
            "contract",
            "Last City V2 is diagnostic-only and requires an independently replayed source prefix",
        ));
    }
    let replay = build_tri_control_hak_last_city_v2(
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )?;
    if contract != &replay.contract {
        return Err(error(
            "M2A-TRI-CONTROL-CONTRACT",
            "contract",
            "Last City V2 contract differs from exact pinned-input deterministic replay",
        ));
    }
    validate_archive_last_city_v2(
        hak,
        base_appearance,
        &replay.appearance_two_da,
        r31_mdl,
        r31_tga,
        h1_mdl,
        h1_tga,
    )?;
    if hak != replay.hak {
        return Err(error(
            "M2A-TRI-CONTROL-HAK-REPLAY",
            "hak",
            "Last City V2 HAK bytes differ from deterministic own-writer replay",
        ));
    }
    Ok(())
}

fn require_pinned_last_city_inputs(
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlHakErrorV1> {
    require_identity(
        "baseAppearance",
        base_appearance,
        LAST_CITY_APPEARANCE_LENGTH_V2,
        LAST_CITY_APPEARANCE_SHA256_V2,
    )?;
    require_binding(base_appearance, 102, None, "S", "c_horror")?;
    require_identity("r31Mdl", r31_mdl, R31_MDL_LENGTH, R31_MDL_SHA256)?;
    require_identity("r31Tga", r31_tga, R31_TGA_LENGTH, R31_TGA_SHA256)?;
    require_identity(
        "h1Appearance",
        h1_appearance,
        H1_APPEARANCE_LENGTH,
        H1_APPEARANCE_SHA256,
    )?;
    require_binding(
        h1_appearance,
        15_100,
        Some("M2A_M6_PROOF"),
        "S",
        H1_MODEL_RESREF,
    )?;
    require_identity("h1Mdl", h1_mdl, H1_MDL_LENGTH, H1_MDL_SHA256)?;
    require_identity("h1Tga", h1_tga, H1_TGA_LENGTH, H1_TGA_SHA256)?;
    Ok(())
}

fn require_last_city_output_identity(appearance: &[u8]) -> Result<(), TriControlHakErrorV1> {
    if appearance.len() as u64 != LAST_CITY_OUTPUT_APPEARANCE_LENGTH_V2
        || sha256(appearance) != LAST_CITY_OUTPUT_APPEARANCE_SHA256_V2
    {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-OUTPUT-IDENTITY",
            "appearance",
            format!(
                "Last City V2 output must be exact {}-byte SHA-256 {}",
                LAST_CITY_OUTPUT_APPEARANCE_LENGTH_V2, LAST_CITY_OUTPUT_APPEARANCE_SHA256_V2
            ),
        ));
    }
    Ok(())
}

fn require_last_city_prefix(source: &[u8], output: &[u8]) -> Result<(), TriControlHakErrorV1> {
    if output.len() < source.len() || &output[..source.len()] != source {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-PREFIX",
            "appearance.sourcePrefix",
            "all user-provided Last City appearance bytes must remain an exact prefix",
        ));
    }
    Ok(())
}

fn require_project_q_reserved_row(
    appearance: &[u8],
    physical_row: u32,
) -> Result<TriControlPreservedAppearanceRowV2, TriControlHakErrorV1> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(appearance, &limits).map_err(map_two_da)?;
    let row = read_two_da_row_v2(appearance, physical_row, &limits).map_err(map_two_da)?;
    let name_index = inspection
        .columns
        .iter()
        .position(|column| column.eq_ignore_ascii_case("NAME"))
        .ok_or_else(|| {
            error(
                "M2A-TRI-CONTROL-LAST-CITY-RESERVED",
                "appearance.columns.NAME",
                "Last City appearance must expose NAME for reserved-row verification",
            )
        })?;
    let name = match row.cells.get(name_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == "PROJECT_Q_RESERVED" => value.clone(),
        _ => {
            return Err(error(
                "M2A-TRI-CONTROL-LAST-CITY-RESERVED",
                format!("appearance.rows[{physical_row}].NAME"),
                "physical rows 15100 and 15101 must remain PROJECT_Q_RESERVED",
            ));
        }
    };
    if row.printed_row_label != physical_row {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-RESERVED",
            format!("appearance.rows[{physical_row}].label"),
            "reserved row printed label must equal its physical row",
        ));
    }
    Ok(TriControlPreservedAppearanceRowV2 {
        physical_row,
        printed_row_label: row.printed_row_label,
        name,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_archive_last_city_v2(
    hak: &[u8],
    base_appearance: &[u8],
    appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlHakErrorV1> {
    let archive = ErfArchive::parse(hak).map_err(|source| {
        error(
            "M2A-TRI-CONTROL-HAK-READBACK",
            format!("hak@{}", source.offset),
            source.context,
        )
    })?;
    if archive.file_type() != ErfFileType::Hak {
        return Err(error(
            "M2A-TRI-CONTROL-HAK-SIGNATURE",
            "hak.signature",
            "tri-control archive must be an exact HAK V1.0 container",
        ));
    }
    let expected_order = expected_resource_order();
    let actual_order = archive
        .resources()
        .iter()
        .map(|resource| TriControlResourceKeyV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
        })
        .collect::<Vec<_>>();
    let expected_set = expected_order.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual_order.iter().cloned().collect::<BTreeSet<_>>();
    if actual_set != expected_set || actual_order.len() != expected_order.len() {
        return Err(error(
            "M2A-TRI-CONTROL-RESOURCE-SET",
            "hak.resources",
            "HAK must contain exactly appearance plus two MDLs and two TGAs",
        ));
    }
    if actual_order != expected_order {
        return Err(error(
            "M2A-TRI-CONTROL-RESOURCE-ORDER",
            "hak.resources",
            "HAK key-table order differs from the canonical five-resource order",
        ));
    }
    let actual_appearance = archive.find("appearance", 2017).map_err(map_erf)?;
    require_last_city_prefix(base_appearance, actual_appearance)?;
    let inspection =
        inspect_two_da_v2(actual_appearance, &TwoDaLimitsV1::default()).map_err(map_two_da)?;
    if inspection.physical_row_count != LAST_CITY_OUTPUT_PHYSICAL_ROWS_V2 {
        return Err(error(
            "M2A-TRI-CONTROL-LAST-CITY-ROWS",
            "appearance.physicalRowCount",
            "Last City V2 HAK appearance must contain exactly 15221 physical rows",
        ));
    }
    require_project_q_reserved_row(actual_appearance, 15_100)?;
    require_project_q_reserved_row(actual_appearance, 15_101)?;
    require_binding(actual_appearance, 102, None, "S", "c_horror")?;
    require_binding(
        actual_appearance,
        LAST_CITY_M0_APPEARANCE_ROW_V2,
        Some("M2A_M0_MESHY_RIGID"),
        "S",
        R31_MODEL_RESREF,
    )?;
    require_binding(
        actual_appearance,
        LAST_CITY_H1_APPEARANCE_ROW_V2,
        Some("M2A_M6_PROOF"),
        "S",
        H1_MODEL_RESREF,
    )?;
    for (resref, resource_type, expected) in [
        ("appearance", 2017, appearance),
        (R31_MODEL_RESREF, 2002, r31_mdl),
        (R31_TEXTURE_RESREF, 3, r31_tga),
        (H1_MODEL_RESREF, 2002, h1_mdl),
        (H1_TEXTURE_RESREF, 3, h1_tga),
    ] {
        let actual = archive.find(resref, resource_type).map_err(map_erf)?;
        if actual != expected {
            return Err(error(
                "M2A-TRI-CONTROL-RESOURCE-PAYLOAD",
                format!("hak.resources[{resref}:{resource_type}]"),
                "HAK payload differs from exact pinned/derived resource bytes",
            ));
        }
    }
    Ok(())
}

fn require_pinned_inputs(
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlHakErrorV1> {
    if base_appearance.len() as u64 != BASE_APPEARANCE_LENGTH {
        return Err(input_identity_error(
            "baseAppearance",
            BASE_APPEARANCE_LENGTH,
            BASE_APPEARANCE_SHA256,
        ));
    }
    require_binding(base_appearance, 102, None, "S", "c_horror")?;
    require_identity(
        "baseAppearance",
        base_appearance,
        BASE_APPEARANCE_LENGTH,
        BASE_APPEARANCE_SHA256,
    )?;
    require_identity("r31Mdl", r31_mdl, R31_MDL_LENGTH, R31_MDL_SHA256)?;
    require_identity("r31Tga", r31_tga, R31_TGA_LENGTH, R31_TGA_SHA256)?;
    require_identity(
        "h1Appearance",
        h1_appearance,
        H1_APPEARANCE_LENGTH,
        H1_APPEARANCE_SHA256,
    )?;
    require_binding(
        h1_appearance,
        15_100,
        Some("M2A_M6_PROOF"),
        "S",
        H1_MODEL_RESREF,
    )?;
    require_identity("h1Mdl", h1_mdl, H1_MDL_LENGTH, H1_MDL_SHA256)?;
    require_identity("h1Tga", h1_tga, H1_TGA_LENGTH, H1_TGA_SHA256)?;
    Ok(())
}

fn clone_physical_row_request(
    source: &[u8],
    physical_row: u32,
    target: &[u8],
) -> Result<TwoDaAppendRequestV1, TriControlHakErrorV1> {
    let limits = TwoDaLimitsV1::default();
    let source_inspection = inspect_two_da_v2(source, &limits).map_err(map_two_da)?;
    let target_inspection = inspect_two_da_v2(target, &limits).map_err(map_two_da)?;
    if source_inspection.columns != target_inspection.columns {
        return Err(error(
            "M2A-TRI-CONTROL-H1-COLUMNS",
            "h1Appearance.columns",
            "exact H1 source and exact r31 target must have identical ordered 2DA columns",
        ));
    }
    let row = read_two_da_row_v2(source, physical_row, &limits).map_err(map_two_da)?;
    if row.cells.len() != source_inspection.columns.len() {
        return Err(error(
            "M2A-TRI-CONTROL-H1-ROW",
            format!("h1Appearance.rows[{physical_row}]"),
            "exact H1 source row does not cover every declared column",
        ));
    }
    Ok(TwoDaAppendRequestV1 {
        schema_version: 1,
        cells: source_inspection
            .columns
            .into_iter()
            .zip(row.cells)
            .map(|(column_name, value)| TwoDaCellAssignmentV1 { column_name, value })
            .collect(),
    })
}

fn require_identity(
    path: &str,
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
) -> Result<(), TriControlHakErrorV1> {
    if bytes.len() as u64 != expected_length || sha256(bytes) != expected_sha256 {
        return Err(input_identity_error(path, expected_length, expected_sha256));
    }
    Ok(())
}

fn input_identity_error(
    path: &str,
    expected_length: u64,
    expected_sha256: &str,
) -> TriControlHakErrorV1 {
    error(
        "M2A-TRI-CONTROL-INPUT-IDENTITY",
        path,
        format!("input must be exact pinned {expected_length}-byte SHA-256 {expected_sha256}"),
    )
}

fn direct_creature_row(
    label: &str,
    race: &str,
    source: &[u8],
) -> Result<TwoDaAppendRequestV1, TriControlHakErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default()).map_err(map_two_da)?;
    let mut cells = [
        ("LABEL", Some(label)),
        ("MOVERATE", Some("NORM")),
        ("MODELTYPE", Some("S")),
        ("RACE", Some(race)),
        ("PORTRAIT", None),
        ("ENVMAP", None),
        ("BLOODCOLR", Some("R")),
        ("WEAPONSCALE", Some("1.0")),
        ("SIZECATEGORY", Some("4")),
    ]
    .into_iter()
    .map(|(column_name, value)| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: match value {
            Some(value) => TwoDaCellValueV1::Text {
                value: value.to_owned(),
            },
            None => TwoDaCellValueV1::Null,
        },
    })
    .collect::<Vec<_>>();
    let phenotype_aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            phenotype_aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(error(
            "M2A-TRI-CONTROL-PHENOTYPE-COLUMN",
            "appearance.columns",
            "appearance schema exposes more than one supported phenotype alias",
        ));
    }
    if let Some(column_name) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (**column_name).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    Ok(TwoDaAppendRequestV1 {
        schema_version: 1,
        cells,
    })
}

fn require_binding(
    appearance: &[u8],
    physical_row: u32,
    expected_label: Option<&str>,
    expected_model_type: &str,
    expected_race: &str,
) -> Result<TriControlAppearanceBindingV1, TriControlHakErrorV1> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(appearance, &limits).map_err(map_two_da)?;
    let row = read_two_da_row_v2(appearance, physical_row, &limits).map_err(map_two_da)?;
    let cell = |column: &str| -> Result<String, TriControlHakErrorV1> {
        let index = inspection
            .columns
            .iter()
            .position(|value| value.eq_ignore_ascii_case(column))
            .ok_or_else(|| {
                error(
                    "M2A-TRI-CONTROL-ROW-BINDING",
                    column,
                    "required appearance column is absent",
                )
            })?;
        match row.cells.get(index) {
            Some(TwoDaCellValueV1::Text { value }) => Ok(value.clone()),
            _ => Err(error(
                "M2A-TRI-CONTROL-ROW-BINDING",
                format!("appearance.rows[{physical_row}].{column}"),
                "required control binding cell is null or absent",
            )),
        }
    };
    let label = cell("LABEL")?;
    let model_type = cell("MODELTYPE")?;
    let race = cell("RACE")?;
    if expected_label.is_some_and(|expected| label != expected)
        || model_type != expected_model_type
        || race != expected_race
    {
        return Err(error(
            "M2A-TRI-CONTROL-ROW-BINDING",
            format!("appearance.rows[{physical_row}]"),
            format!(
                "expected LABEL={expected_label:?}, MODELTYPE={expected_model_type}, RACE={expected_race}; got LABEL={label:?}, MODELTYPE={model_type:?}, RACE={race:?}"
            ),
        ));
    }
    Ok(TriControlAppearanceBindingV1 {
        physical_row,
        label,
        model_type,
        race,
    })
}

fn resource_inputs(
    appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Vec<HakResourceInputV1> {
    vec![
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.to_vec(),
        },
        HakResourceInputV1 {
            resref: R31_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: r31_mdl.to_vec(),
        },
        HakResourceInputV1 {
            resref: R31_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: r31_tga.to_vec(),
        },
        HakResourceInputV1 {
            resref: H1_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: h1_mdl.to_vec(),
        },
        HakResourceInputV1 {
            resref: H1_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: h1_tga.to_vec(),
        },
    ]
}

fn expected_resource_order() -> Vec<TriControlResourceKeyV1> {
    TRI_CONTROL_RESOURCE_ORDER_V1
        .iter()
        .map(|(resref, resource_type)| TriControlResourceKeyV1 {
            resref: (*resref).to_owned(),
            resource_type: *resource_type,
        })
        .collect()
}

fn expected_resource_payloads(
    appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Vec<TriControlResourceIdentityV1> {
    [
        ("appearance", 2017, appearance),
        (R31_MODEL_RESREF, 2002, r31_mdl),
        (R31_TEXTURE_RESREF, 3, r31_tga),
        (H1_MODEL_RESREF, 2002, h1_mdl),
        (H1_TEXTURE_RESREF, 3, h1_tga),
    ]
    .into_iter()
    .map(
        |(resref, resource_type, bytes)| TriControlResourceIdentityV1 {
            resref: resref.to_owned(),
            resource_type,
            byte_length: bytes.len() as u64,
            sha256: sha256(bytes),
        },
    )
    .collect()
}

fn validate_archive(
    hak: &[u8],
    appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlHakErrorV1> {
    let archive = ErfArchive::parse(hak).map_err(|source| {
        error(
            "M2A-TRI-CONTROL-HAK-READBACK",
            format!("hak@{}", source.offset),
            source.context,
        )
    })?;
    if archive.file_type() != ErfFileType::Hak {
        return Err(error(
            "M2A-TRI-CONTROL-HAK-SIGNATURE",
            "hak.signature",
            "tri-control archive must be an exact HAK V1.0 container",
        ));
    }
    let expected_order = expected_resource_order();
    let actual_order = archive
        .resources()
        .iter()
        .map(|resource| TriControlResourceKeyV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
        })
        .collect::<Vec<_>>();
    let expected_set = expected_order.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual_order.iter().cloned().collect::<BTreeSet<_>>();
    if actual_set != expected_set || actual_order.len() != expected_order.len() {
        return Err(error(
            "M2A-TRI-CONTROL-RESOURCE-SET",
            "hak.resources",
            "HAK must contain exactly appearance plus two MDLs and two TGAs",
        ));
    }
    if actual_order != expected_order {
        return Err(error(
            "M2A-TRI-CONTROL-RESOURCE-ORDER",
            "hak.resources",
            "HAK key-table order differs from the own writer's canonical five-resource order",
        ));
    }
    let actual_appearance = archive.find("appearance", 2017).map_err(map_erf)?;
    require_binding(actual_appearance, 102, None, "S", "c_horror")?;
    require_binding(
        actual_appearance,
        15_100,
        Some("M2A_M0_MESHY_RIGID"),
        "S",
        R31_MODEL_RESREF,
    )?;
    require_binding(
        actual_appearance,
        15_101,
        Some("M2A_M6_PROOF"),
        "S",
        H1_MODEL_RESREF,
    )?;
    for (resref, resource_type, expected) in [
        ("appearance", 2017, appearance),
        (R31_MODEL_RESREF, 2002, r31_mdl),
        (R31_TEXTURE_RESREF, 3, r31_tga),
        (H1_MODEL_RESREF, 2002, h1_mdl),
        (H1_TEXTURE_RESREF, 3, h1_tga),
    ] {
        let actual = archive.find(resref, resource_type).map_err(map_erf)?;
        if actual != expected {
            return Err(error(
                "M2A-TRI-CONTROL-RESOURCE-PAYLOAD",
                format!("hak.resources[{resref}:{resource_type}]"),
                "HAK payload differs from exact pinned/derived resource bytes",
            ));
        }
    }
    Ok(())
}

fn map_two_da(source: crate::two_da::TwoDaError) -> TriControlHakErrorV1 {
    error(source.code, source.path, source.message)
}

fn map_erf(source: crate::erf::ErfError) -> TriControlHakErrorV1 {
    error(
        "M2A-TRI-CONTROL-HAK-READBACK",
        format!("hak@{}", source.offset),
        source.context,
    )
}

fn identity(bytes: &[u8]) -> TriControlByteIdentityV1 {
    TriControlByteIdentityV1 {
        byte_length: bytes.len() as u64,
        sha256: sha256(bytes),
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> TriControlHakErrorV1 {
    TriControlHakErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
