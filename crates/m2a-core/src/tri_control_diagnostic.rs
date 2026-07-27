//! Offline composition for the three-creature resolver/renderer diagnostic.
//!
//! The artifact deliberately keeps the exact r31 M0 model and texture bytes,
//! adds the exact project-owned H1 v20 positive-control bytes, and references
//! stock `c_horror` only through appearance row 102. It does not install or
//! launch anything and cannot promote its structural result to a runtime
//! resolver or renderer claim.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureMultiFixtureModuleReadbackV1,
        BinaryCreatureOwnedFixtureV1, M0RuntimeDirectionV1, M0RuntimePositionV1,
        ProofModuleErrorV1, build_binary_creature_multi_fixture_module_v1,
    },
    tri_control_hak::{
        LAST_CITY_H1_APPEARANCE_ROW_V2, LAST_CITY_M0_APPEARANCE_ROW_V2, TriControlByteIdentityV1,
        TriControlHakContractV1, TriControlHakContractV2, TriControlHakErrorV1,
        build_tri_control_hak_last_city_v2, build_tri_control_hak_v1,
        verify_tri_control_hak_last_city_v2, verify_tri_control_hak_v1,
    },
};

pub const TRI_CONTROL_DIAGNOSTIC_CONTRACT_V1: &str =
    "M2A_PROJECT_DIRECT_CREATURE_TRI_CONTROL_DIAGNOSTIC_V1";
pub const TRI_CONTROL_DIAGNOSTIC_CONTRACT_V2: &str =
    "M2A_PROJECT_DIRECT_CREATURE_TRI_CONTROL_LAST_CITY_DIAGNOSTIC_V2";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlDiagnosticFixtureV1 {
    pub role: String,
    pub id: String,
    pub template_resref: String,
    pub display_name: String,
    pub appearance_row: u16,
    pub model_resref: String,
    pub position: M0RuntimePositionV1,
    pub orientation: M0RuntimeDirectionV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlDiagnosticContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub diagnostic_only: bool,
    pub runtime_admissible: bool,
    pub resolver_verified: bool,
    pub renderer_verified: bool,
    pub module_identity: BinaryCreatureModuleIdentityV1,
    pub module: TriControlByteIdentityV1,
    pub module_readback: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub hak_contract: TriControlHakContractV1,
    pub fixtures: Vec<TriControlDiagnosticFixtureV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TriControlDiagnosticArtifactV1 {
    pub module: Vec<u8>,
    pub hak: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub contract: TriControlDiagnosticContractV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriControlDiagnosticContractV2 {
    pub schema_version: u32,
    pub profile: String,
    pub diagnostic_only: bool,
    pub runtime_admissible: bool,
    pub resolver_verified: bool,
    pub renderer_verified: bool,
    pub module_identity: BinaryCreatureModuleIdentityV1,
    pub module: TriControlByteIdentityV1,
    pub module_readback: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub hak_contract: TriControlHakContractV2,
    pub fixtures: Vec<TriControlDiagnosticFixtureV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TriControlDiagnosticArtifactV2 {
    pub module: Vec<u8>,
    pub hak: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub contract: TriControlDiagnosticContractV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriControlDiagnosticErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for TriControlDiagnosticErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TriControlDiagnosticErrorV1 {}

#[allow(clippy::too_many_arguments)]
pub fn build_tri_control_diagnostic_v1(
    identity: &BinaryCreatureModuleIdentityV1,
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<TriControlDiagnosticArtifactV1, TriControlDiagnosticErrorV1> {
    let hak = build_tri_control_hak_v1(
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )
    .map_err(map_hak_error)?;
    let fixtures = diagnostic_fixtures();
    let module_fixtures = fixtures
        .iter()
        .map(|fixture| BinaryCreatureOwnedFixtureV1 {
            id: fixture.id.clone(),
            template_resref: fixture.template_resref.clone(),
            display_name: fixture.display_name.clone(),
            appearance_row: fixture.appearance_row,
            position: fixture.position,
            orientation: fixture.orientation,
        })
        .collect::<Vec<_>>();
    let module = build_binary_creature_multi_fixture_module_v1(identity, &module_fixtures)
        .map_err(map_module_error)?;
    if module.readback.fixtures != module_fixtures
        || module.readback.ordered_hak_resrefs != [identity.hak_resref.clone()]
    {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-MODULE-BINDING",
            "module",
            "module readback differs from the exact diagnostic fixtures or singleton HAK",
        ));
    }
    Ok(TriControlDiagnosticArtifactV1 {
        module: module.payload,
        hak: hak.hak,
        appearance_two_da: hak.appearance_two_da,
        contract: TriControlDiagnosticContractV1 {
            schema_version: 1,
            profile: TRI_CONTROL_DIAGNOSTIC_CONTRACT_V1.to_owned(),
            diagnostic_only: true,
            runtime_admissible: false,
            resolver_verified: false,
            renderer_verified: false,
            module_identity: identity.clone(),
            module: TriControlByteIdentityV1 {
                byte_length: module.byte_length,
                sha256: module.sha256,
            },
            module_readback: module.readback,
            hak_contract: hak.contract,
            fixtures,
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_tri_control_diagnostic_v1(
    contract: &TriControlDiagnosticContractV1,
    module: &[u8],
    hak: &[u8],
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlDiagnosticErrorV1> {
    if contract.schema_version != 1
        || contract.profile != TRI_CONTROL_DIAGNOSTIC_CONTRACT_V1
        || !contract.diagnostic_only
        || contract.runtime_admissible
        || contract.resolver_verified
        || contract.renderer_verified
    {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-CONTRACT",
            "contract",
            "offline tri-control evidence cannot self-promote to runtime, resolver or renderer admission",
        ));
    }
    let replay = build_tri_control_diagnostic_v1(
        &contract.module_identity,
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )?;
    if contract != &replay.contract {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-CONTRACT",
            "contract",
            "contract differs from independent deterministic replay",
        ));
    }
    if module != replay.module {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-MODULE-REPLAY",
            "module",
            "MOD bytes differ from independent deterministic replay",
        ));
    }
    verify_tri_control_hak_v1(
        &contract.hak_contract,
        hak,
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )
    .map_err(map_hak_error)?;
    if hak != replay.hak {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-HAK-REPLAY",
            "hak",
            "HAK bytes differ from independent deterministic replay",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn build_tri_control_diagnostic_last_city_v2(
    identity: &BinaryCreatureModuleIdentityV1,
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<TriControlDiagnosticArtifactV2, TriControlDiagnosticErrorV1> {
    let hak = build_tri_control_hak_last_city_v2(
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )
    .map_err(map_hak_error)?;
    let fixtures = diagnostic_fixtures_last_city_v2();
    let module_fixtures = fixtures
        .iter()
        .map(|fixture| BinaryCreatureOwnedFixtureV1 {
            id: fixture.id.clone(),
            template_resref: fixture.template_resref.clone(),
            display_name: fixture.display_name.clone(),
            appearance_row: fixture.appearance_row,
            position: fixture.position,
            orientation: fixture.orientation,
        })
        .collect::<Vec<_>>();
    let module = build_binary_creature_multi_fixture_module_v1(identity, &module_fixtures)
        .map_err(map_module_error)?;
    if module.readback.fixtures != module_fixtures
        || module.readback.ordered_hak_resrefs != [identity.hak_resref.clone()]
    {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-MODULE-BINDING",
            "module",
            "Last City V2 module readback differs from exact fixtures or singleton HAK",
        ));
    }
    Ok(TriControlDiagnosticArtifactV2 {
        module: module.payload,
        hak: hak.hak,
        appearance_two_da: hak.appearance_two_da,
        contract: TriControlDiagnosticContractV2 {
            schema_version: 2,
            profile: TRI_CONTROL_DIAGNOSTIC_CONTRACT_V2.to_owned(),
            diagnostic_only: true,
            runtime_admissible: false,
            resolver_verified: false,
            renderer_verified: false,
            module_identity: identity.clone(),
            module: TriControlByteIdentityV1 {
                byte_length: module.byte_length,
                sha256: module.sha256,
            },
            module_readback: module.readback,
            hak_contract: hak.contract,
            fixtures,
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_tri_control_diagnostic_last_city_v2(
    contract: &TriControlDiagnosticContractV2,
    module: &[u8],
    hak: &[u8],
    base_appearance: &[u8],
    r31_mdl: &[u8],
    r31_tga: &[u8],
    h1_appearance: &[u8],
    h1_mdl: &[u8],
    h1_tga: &[u8],
) -> Result<(), TriControlDiagnosticErrorV1> {
    if contract.schema_version != 2
        || contract.profile != TRI_CONTROL_DIAGNOSTIC_CONTRACT_V2
        || !contract.diagnostic_only
        || contract.runtime_admissible
        || contract.resolver_verified
        || contract.renderer_verified
    {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-CONTRACT",
            "contract",
            "Last City V2 diagnostic cannot self-promote to runtime, resolver or renderer admission",
        ));
    }
    let replay = build_tri_control_diagnostic_last_city_v2(
        &contract.module_identity,
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )?;
    if contract != &replay.contract {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-CONTRACT",
            "contract",
            "Last City V2 contract differs from independent deterministic replay",
        ));
    }
    if module != replay.module {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-MODULE-REPLAY",
            "module",
            "Last City V2 MOD bytes differ from independent deterministic replay",
        ));
    }
    verify_tri_control_hak_last_city_v2(
        &contract.hak_contract,
        hak,
        base_appearance,
        r31_mdl,
        r31_tga,
        h1_appearance,
        h1_mdl,
        h1_tga,
    )
    .map_err(map_hak_error)?;
    if hak != replay.hak {
        return Err(error(
            "M2A-TRI-CONTROL-DIAGNOSTIC-HAK-REPLAY",
            "hak",
            "Last City V2 HAK bytes differ from independent deterministic replay",
        ));
    }
    Ok(())
}

fn diagnostic_fixtures() -> Vec<TriControlDiagnosticFixtureV1> {
    vec![
        diagnostic_fixture(
            "stock_renderer_control",
            "stock_control",
            "m2a_d_stock",
            "Stock c_horror control",
            102,
            "c_horror",
            5.0,
            18.0,
        ),
        diagnostic_fixture(
            "exact_r31_candidate",
            "candidate_m0",
            "m2a_d_m0",
            "Exact r31 M0 candidate",
            15_100,
            "m2a_m0p01",
            10.0,
            14.5,
        ),
        diagnostic_fixture(
            "project_owned_positive_control",
            "custom_control_h1",
            "m2a_d_h1",
            "Project-owned H1 v20 control",
            15_101,
            "m2a_m6p01",
            15.0,
            18.0,
        ),
    ]
}

fn diagnostic_fixtures_last_city_v2() -> Vec<TriControlDiagnosticFixtureV1> {
    let mut fixtures = diagnostic_fixtures();
    fixtures[1].appearance_row = LAST_CITY_M0_APPEARANCE_ROW_V2 as u16;
    fixtures[2].appearance_row = LAST_CITY_H1_APPEARANCE_ROW_V2 as u16;
    fixtures
}

#[allow(clippy::too_many_arguments)]
fn diagnostic_fixture(
    role: &str,
    id: &str,
    template_resref: &str,
    display_name: &str,
    appearance_row: u16,
    model_resref: &str,
    x: f32,
    y: f32,
) -> TriControlDiagnosticFixtureV1 {
    TriControlDiagnosticFixtureV1 {
        role: role.to_owned(),
        id: id.to_owned(),
        template_resref: template_resref.to_owned(),
        display_name: display_name.to_owned(),
        appearance_row,
        model_resref: model_resref.to_owned(),
        position: M0RuntimePositionV1 { x, y, z: 0.0 },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }
}

fn map_hak_error(source: TriControlHakErrorV1) -> TriControlDiagnosticErrorV1 {
    error(source.code, source.path, source.message)
}

fn map_module_error(source: ProofModuleErrorV1) -> TriControlDiagnosticErrorV1 {
    error(source.code, source.path, source.message)
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> TriControlDiagnosticErrorV1 {
    TriControlDiagnosticErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
