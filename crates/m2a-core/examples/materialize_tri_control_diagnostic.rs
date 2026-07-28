//! Offline-only materializer for the exact three-creature diagnostic fixture.
//!
//! This example writes one caller-named MOD, one singleton HAK, and two
//! canonical JSON records. It never installs artifacts, starts processes, or
//! asserts runtime, resolver, renderer, Toolset, or NWN success.

use std::{
    collections::BTreeMap,
    env,
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use m2a_core::{
    proof_module::BinaryCreatureModuleIdentityV1,
    tri_control_diagnostic::{
        TriControlDiagnosticContractV1, TriControlDiagnosticContractV2,
        build_tri_control_diagnostic_last_city_v2, build_tri_control_diagnostic_v1,
        verify_tri_control_diagnostic_last_city_v2, verify_tri_control_diagnostic_v1,
    },
    tri_control_hak::{
        TriControlAppearanceBindingV1, TriControlByteIdentityV1, TriControlResourceIdentityV1,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const PROFILE_VERSION: &str = "m2a-tri-control-diagnostic-materialization-profile/v1";
const CONTRACT_FILE: &str = "tri-control-diagnostic-contract-v1.json";
const PROFILE_FILE: &str = "tri-control-diagnostic-profile-v1.json";
const LAST_CITY_PROFILE_SELECTOR_V2: &str = "last-city-v2";
const LAST_CITY_PROFILE_VERSION_V2: &str =
    "m2a-tri-control-last-city-diagnostic-materialization-profile/v2";
const LAST_CITY_CONTRACT_FILE_V2: &str = "tri-control-diagnostic-contract-v2.json";
const LAST_CITY_PROFILE_FILE_V2: &str = "tri-control-diagnostic-profile-v2.json";
const CANONICAL_REPO_ROOT: &str = r"C:\Projects\meshy2aurora";
const WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
static PUBLICATION_NONCE: AtomicU64 = AtomicU64::new(0);
const EXPECTED_OPTIONS: [&str; 16] = [
    "outDir",
    "module-resref",
    "area-resref",
    "hak-resref",
    "base-appearance",
    "base-appearance-sha256",
    "r31-model",
    "r31-model-sha256",
    "r31-texture",
    "r31-texture-sha256",
    "h1-appearance",
    "h1-appearance-sha256",
    "h1-model",
    "h1-model-sha256",
    "h1-texture",
    "h1-texture-sha256",
];

const BASE_APPEARANCE: PinnedWitness = PinnedWitness {
    argument: "base-appearance",
    relative_path: "local-reference-assets/appearance.2da",
    byte_length: 6_901_169,
    sha256: "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a",
};
const LAST_CITY_BASE_APPEARANCE_V2: PinnedWitness = PinnedWitness {
    argument: "base-appearance",
    relative_path: "proof-output/lc-hd-animals-c-squirrel-reference-audit-2026-07-18/source-copies/appearance.2da",
    byte_length: 7_655_336,
    sha256: "ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2",
};
const R31_MODEL: PinnedWitness = PinnedWitness {
    argument: "r31-model",
    relative_path: "proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0p01.mdl",
    byte_length: 151_328,
    sha256: "fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6",
};
const R31_TEXTURE: PinnedWitness = PinnedWitness {
    argument: "r31-texture",
    relative_path: "proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0t01.tga",
    byte_length: 12_582_956,
    sha256: "079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b",
};
const H1_APPEARANCE: PinnedWitness = PinnedWitness {
    argument: "h1-appearance",
    relative_path: "proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated/appearance.2da",
    byte_length: 6_901_354,
    sha256: "e50e2f8c5fc42771f5c433dd3984b0f8e740938d6f722adf838942f1f6342f8e",
};
const H1_MODEL: PinnedWitness = PinnedWitness {
    argument: "h1-model",
    relative_path: "proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated/m2a_m6p01.mdl",
    byte_length: 557_268,
    sha256: "6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd",
};
const H1_TEXTURE: PinnedWitness = PinnedWitness {
    argument: "h1-texture",
    relative_path: "proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated/m2a_m6t01.tga",
    byte_length: 12_582_956,
    sha256: "ab8f11c7f448801d905594802a223a1ed5969bc5d09ce06e6588cbe83b6a86b4",
};
const PINNED_WITNESSES: [PinnedWitness; 6] = [
    BASE_APPEARANCE,
    R31_MODEL,
    R31_TEXTURE,
    H1_APPEARANCE,
    H1_MODEL,
    H1_TEXTURE,
];
const LAST_CITY_PINNED_WITNESSES_V2: [PinnedWitness; 6] = [
    LAST_CITY_BASE_APPEARANCE_V2,
    R31_MODEL,
    R31_TEXTURE,
    H1_APPEARANCE,
    H1_MODEL,
    H1_TEXTURE,
];

#[allow(dead_code)] // This file is also included directly by its contract test.
fn main() -> ExitCode {
    match run_with_arguments(env::args().skip(1)) {
        Ok(report) => match serde_json::to_string_pretty(&report) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("M2A-TRI-CONTROL-CLI-REPORT: {error}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_with_arguments(
    arguments: impl IntoIterator<Item = String>,
) -> Result<MaterializationReportV1, String> {
    let command = parse_command(arguments)?;
    let repo_root = canonical_repo_root()?;
    let output_dir = resolve_output_dir(&repo_root, &command.output_dir)?;
    if path_entry_exists(&output_dir)? {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-OUTPUT-EXISTS: {}",
            output_dir.display()
        ));
    }
    require_resref(&command.module_resref, "moduleResref")?;
    require_resref(&command.area_resref, "areaResref")?;
    require_resref(&command.hak_resref, "hakResref")?;

    let mut inputs = BTreeMap::new();
    for witness in command.variant.pinned_witnesses() {
        let supplied = command.inputs.get(witness.argument).ok_or_else(|| {
            format!(
                "M2A-TRI-CONTROL-CLI-ARGUMENT-MISSING: --{}",
                witness.argument
            )
        })?;
        let loaded = read_pinned_witness(&repo_root, *witness, supplied)?;
        inputs.insert(witness.argument, loaded);
    }

    let identity = BinaryCreatureModuleIdentityV1 {
        module_resref: command.module_resref,
        area_resref: command.area_resref,
        hak_resref: command.hak_resref,
    };
    let built = match command.variant {
        MaterializationVariant::RetailV1 => {
            let artifact = build_tri_control_diagnostic_v1(
                &identity,
                &inputs[BASE_APPEARANCE.argument].bytes,
                &inputs[R31_MODEL.argument].bytes,
                &inputs[R31_TEXTURE.argument].bytes,
                &inputs[H1_APPEARANCE.argument].bytes,
                &inputs[H1_MODEL.argument].bytes,
                &inputs[H1_TEXTURE.argument].bytes,
            )
            .map_err(|error| error.to_string())?;
            let contract_bytes = canonical_json(&artifact.contract, "CONTRACT")?;
            BuiltDiagnostic {
                module: artifact.module,
                hak: artifact.hak,
                contract_bytes,
                appearance: artifact.contract.hak_contract.appearance.clone(),
                appearance_rows: vec![
                    artifact.contract.hak_contract.base_control.clone(),
                    artifact.contract.hak_contract.r31_control.clone(),
                    artifact.contract.hak_contract.h1_control.clone(),
                ],
                fixtures: artifact.contract.fixtures.clone(),
                resources: artifact.contract.hak_contract.resources.clone(),
                contract: DiagnosticContract::RetailV1(artifact.contract),
            }
        }
        MaterializationVariant::LastCityV2 => {
            let artifact = build_tri_control_diagnostic_last_city_v2(
                &identity,
                &inputs[LAST_CITY_BASE_APPEARANCE_V2.argument].bytes,
                &inputs[R31_MODEL.argument].bytes,
                &inputs[R31_TEXTURE.argument].bytes,
                &inputs[H1_APPEARANCE.argument].bytes,
                &inputs[H1_MODEL.argument].bytes,
                &inputs[H1_TEXTURE.argument].bytes,
            )
            .map_err(|error| error.to_string())?;
            let contract_bytes = canonical_json(&artifact.contract, "CONTRACT")?;
            BuiltDiagnostic {
                module: artifact.module,
                hak: artifact.hak,
                contract_bytes,
                appearance: artifact.contract.hak_contract.appearance.clone(),
                appearance_rows: vec![
                    artifact.contract.hak_contract.base_control.clone(),
                    artifact.contract.hak_contract.r31_control.clone(),
                    artifact.contract.hak_contract.h1_control.clone(),
                ],
                fixtures: artifact.contract.fixtures.clone(),
                resources: artifact.contract.hak_contract.resources.clone(),
                contract: DiagnosticContract::LastCityV2(artifact.contract),
            }
        }
    };
    let module_file = format!("{}.mod", identity.module_resref);
    let hak_file = format!("{}.hak", identity.hak_resref);
    let profile = MaterializationProfileV1 {
        version: command.variant.profile_version().to_owned(),
        id: identity.module_resref.clone(),
        diagnostic_only: true,
        runtime_admissible: false,
        resolver_verified: false,
        renderer_verified: false,
        identity: identity.clone(),
        inputs: inputs.values().map(|input| input.binding.clone()).collect(),
        module: output_binding(&module_file, &built.module),
        hak: output_binding(&hak_file, &built.hak),
        contract: output_binding(command.variant.contract_file(), &built.contract_bytes),
        appearance: built.appearance.clone(),
        appearance_rows: built.appearance_rows.clone(),
        fixtures: built.fixtures.clone(),
        resources: built.resources.clone(),
        output_file_count: 4,
    };
    let profile_bytes = canonical_json(&profile, "PROFILE")?;
    let outputs = [
        (module_file.as_str(), built.module.as_slice()),
        (hak_file.as_str(), built.hak.as_slice()),
        (
            command.variant.contract_file(),
            built.contract_bytes.as_slice(),
        ),
        (command.variant.profile_file(), profile_bytes.as_slice()),
    ];
    publish_outputs_atomically_verified(
        &repo_root,
        &output_dir,
        &outputs,
        PublicationFaultV1::None,
        || verify_written_outputs(&output_dir, &outputs, &built.contract, &inputs),
    )?;

    Ok(MaterializationReportV1 {
        status: command.variant.status().to_owned(),
        output_directory: output_dir,
        output_file_count: outputs.len(),
        module: output_binding(&module_file, &built.module),
        hak: output_binding(&hak_file, &built.hak),
        contract: output_binding(command.variant.contract_file(), &built.contract_bytes),
        profile: output_binding(command.variant.profile_file(), &profile_bytes),
        diagnostic_only: true,
        runtime_admissible: false,
        starts_toolset: false,
        starts_nwn: false,
        installs_artifacts: false,
    })
}

#[derive(Clone, Copy, Debug)]
struct PinnedWitness {
    argument: &'static str,
    relative_path: &'static str,
    byte_length: u64,
    sha256: &'static str,
}

#[derive(Clone, Debug)]
struct InputArgument {
    path: PathBuf,
    declared_sha256: String,
}

#[derive(Clone, Debug)]
struct LoadedInput {
    bytes: Vec<u8>,
    binding: InputBindingV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MaterializationVariant {
    RetailV1,
    LastCityV2,
}

impl MaterializationVariant {
    fn pinned_witnesses(self) -> &'static [PinnedWitness] {
        match self {
            Self::RetailV1 => &PINNED_WITNESSES,
            Self::LastCityV2 => &LAST_CITY_PINNED_WITNESSES_V2,
        }
    }

    fn profile_version(self) -> &'static str {
        match self {
            Self::RetailV1 => PROFILE_VERSION,
            Self::LastCityV2 => LAST_CITY_PROFILE_VERSION_V2,
        }
    }

    fn contract_file(self) -> &'static str {
        match self {
            Self::RetailV1 => CONTRACT_FILE,
            Self::LastCityV2 => LAST_CITY_CONTRACT_FILE_V2,
        }
    }

    fn profile_file(self) -> &'static str {
        match self {
            Self::RetailV1 => PROFILE_FILE,
            Self::LastCityV2 => LAST_CITY_PROFILE_FILE_V2,
        }
    }

    fn status(self) -> &'static str {
        match self {
            Self::RetailV1 => "TRI_CONTROL_DIAGNOSTIC_MATERIALIZED_OFFLINE_ONLY",
            Self::LastCityV2 => "TRI_CONTROL_LAST_CITY_DIAGNOSTIC_MATERIALIZED_OFFLINE_ONLY",
        }
    }

    #[cfg(test)]
    fn selector(self) -> &'static str {
        match self {
            Self::RetailV1 => "retail-v1",
            Self::LastCityV2 => LAST_CITY_PROFILE_SELECTOR_V2,
        }
    }
}

enum DiagnosticContract {
    RetailV1(TriControlDiagnosticContractV1),
    LastCityV2(TriControlDiagnosticContractV2),
}

impl DiagnosticContract {
    fn module_identity(&self) -> &BinaryCreatureModuleIdentityV1 {
        match self {
            Self::RetailV1(contract) => &contract.module_identity,
            Self::LastCityV2(contract) => &contract.module_identity,
        }
    }
}

struct BuiltDiagnostic {
    module: Vec<u8>,
    hak: Vec<u8>,
    contract_bytes: Vec<u8>,
    appearance: TriControlByteIdentityV1,
    appearance_rows: Vec<TriControlAppearanceBindingV1>,
    fixtures: Vec<m2a_core::tri_control_diagnostic::TriControlDiagnosticFixtureV1>,
    resources: Vec<TriControlResourceIdentityV1>,
    contract: DiagnosticContract,
}

#[derive(Clone, Debug)]
struct Command {
    variant: MaterializationVariant,
    output_dir: PathBuf,
    module_resref: String,
    area_resref: String,
    hak_resref: String,
    inputs: BTreeMap<&'static str, InputArgument>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct InputBindingV1 {
    role: String,
    path: String,
    byte_length: u64,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputBindingV1 {
    pub file_name: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MaterializationProfileV1 {
    version: String,
    id: String,
    diagnostic_only: bool,
    runtime_admissible: bool,
    resolver_verified: bool,
    renderer_verified: bool,
    identity: BinaryCreatureModuleIdentityV1,
    inputs: Vec<InputBindingV1>,
    module: OutputBindingV1,
    hak: OutputBindingV1,
    contract: OutputBindingV1,
    appearance: TriControlByteIdentityV1,
    appearance_rows: Vec<TriControlAppearanceBindingV1>,
    fixtures: Vec<m2a_core::tri_control_diagnostic::TriControlDiagnosticFixtureV1>,
    resources: Vec<TriControlResourceIdentityV1>,
    output_file_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationReportV1 {
    pub status: String,
    pub output_directory: PathBuf,
    pub output_file_count: usize,
    pub module: OutputBindingV1,
    pub hak: OutputBindingV1,
    pub contract: OutputBindingV1,
    pub profile: OutputBindingV1,
    pub diagnostic_only: bool,
    pub runtime_admissible: bool,
    pub starts_toolset: bool,
    pub starts_nwn: bool,
    pub installs_artifacts: bool,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut values = arguments.into_iter();
    let mut options = BTreeMap::<String, String>::new();
    while let Some(argument) = values.next() {
        if argument == "--help" || argument == "-h" {
            return Err(usage());
        }
        let Some(name) = argument.strip_prefix("--") else {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-ARGUMENT-UNKNOWN: {argument}\n{}",
                usage()
            ));
        };
        if !expected_option(name) {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-ARGUMENT-UNKNOWN: {argument}\n{}",
                usage()
            ));
        }
        let value = values
            .next()
            .filter(|value| !value.starts_with("--") && !value.trim().is_empty())
            .ok_or_else(|| format!("M2A-TRI-CONTROL-CLI-ARGUMENT-MISSING: {argument}"))?;
        if options.insert(name.to_owned(), value).is_some() {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
    }
    for name in EXPECTED_OPTIONS {
        if !options.contains_key(name) {
            return Err(format!("M2A-TRI-CONTROL-CLI-ARGUMENT-MISSING: --{name}"));
        }
    }
    let variant = match options.remove("profile").as_deref() {
        None | Some("retail-v1") => MaterializationVariant::RetailV1,
        Some(LAST_CITY_PROFILE_SELECTOR_V2) => MaterializationVariant::LastCityV2,
        Some(value) => {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-PROFILE: unsupported explicit profile {value:?}"
            ));
        }
    };
    let mut inputs = BTreeMap::new();
    for witness in variant.pinned_witnesses() {
        inputs.insert(
            witness.argument,
            InputArgument {
                path: PathBuf::from(options.remove(witness.argument).unwrap()),
                declared_sha256: options
                    .remove(&format!("{}-sha256", witness.argument))
                    .unwrap(),
            },
        );
    }
    Ok(Command {
        variant,
        output_dir: PathBuf::from(options.remove("outDir").unwrap()),
        module_resref: options.remove("module-resref").unwrap(),
        area_resref: options.remove("area-resref").unwrap(),
        hak_resref: options.remove("hak-resref").unwrap(),
        inputs,
    })
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedMaterializationCommandForTestV1 {
    pub profile: String,
    pub module_resref: String,
    pub area_resref: String,
    pub hak_resref: String,
    pub base_relative_path: String,
    pub base_byte_length: u64,
    pub base_sha256: String,
    pub supplied_base_path: PathBuf,
    pub declared_base_sha256: String,
}

#[cfg(test)]
pub fn test_parse_command_summary(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ParsedMaterializationCommandForTestV1, String> {
    let command = parse_command(arguments)?;
    let base = command.variant.pinned_witnesses()[0];
    let supplied = command.inputs.get(base.argument).unwrap();
    Ok(ParsedMaterializationCommandForTestV1 {
        profile: command.variant.selector().to_owned(),
        module_resref: command.module_resref,
        area_resref: command.area_resref,
        hak_resref: command.hak_resref,
        base_relative_path: base.relative_path.to_owned(),
        base_byte_length: base.byte_length,
        base_sha256: base.sha256.to_owned(),
        supplied_base_path: supplied.path.clone(),
        declared_base_sha256: supplied.declared_sha256.clone(),
    })
}

fn expected_option(value: &str) -> bool {
    value == "profile" || EXPECTED_OPTIONS.contains(&value)
}

fn canonical_repo_root() -> Result<PathBuf, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let resolved = root
        .canonicalize()
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-REPO-ROOT: {error}"))?;
    let expected = PathBuf::from(CANONICAL_REPO_ROOT)
        .canonicalize()
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-CANONICAL-ROOT: {error}"))?;
    if resolved != expected {
        #[cfg(test)]
        if resolved.parent() == Some(expected.join(".worktrees").as_path()) {
            require_no_reparse_chain(&expected, false, "canonical repository root")?;
            return Ok(expected);
        }
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-NONCANONICAL-WORKSPACE: {}",
            resolved.display()
        ));
    }
    require_no_reparse_chain(&resolved, false, "canonical repository root")?;
    Ok(resolved)
}

fn resolve_output_dir(repo_root: &Path, value: &Path) -> Result<PathBuf, String> {
    require_clean_path(value, "outDir")?;
    let output = if value.is_absolute() {
        value.to_path_buf()
    } else {
        repo_root.join(value)
    };
    let parent = output
        .parent()
        .ok_or_else(|| "M2A-TRI-CONTROL-CLI-OUTPUT-PARENT-MISSING".to_owned())?;
    require_lexically_under_repo(repo_root, parent, "outDir parent")?;
    require_no_reparse_chain(parent, false, "outDir parent")?;
    let resolved_parent = parent.canonicalize().map_err(|error| {
        format!(
            "M2A-TRI-CONTROL-CLI-OUTPUT-PARENT: {}: {error}",
            parent.display()
        )
    })?;
    if !resolved_parent.starts_with(repo_root) {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-OUTPUT-OUTSIDE-CANONICAL-WORKSPACE: {}",
            output.display()
        ));
    }
    require_no_reparse_chain(&resolved_parent, false, "resolved outDir parent")?;
    let name = output
        .file_name()
        .ok_or_else(|| "M2A-TRI-CONTROL-CLI-OUTPUT-NAME-MISSING".to_owned())?;
    let resolved_output = resolved_parent.join(name);
    if path_entry_exists(&resolved_output)? {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-OUTPUT-EXISTS: {}",
            resolved_output.display()
        ));
    }
    Ok(resolved_output)
}

fn require_resref(value: &str, path: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!(
            "M2A-BINARY-CREATURE-IDENTITY: {path} must be 1..16 lowercase ASCII resref characters"
        ));
    }
    Ok(())
}

fn read_pinned_witness(
    repo_root: &Path,
    witness: PinnedWitness,
    supplied: &InputArgument,
) -> Result<LoadedInput, String> {
    if supplied.declared_sha256 != witness.sha256 {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-INPUT-DECLARED-HASH: --{}-sha256 must equal {}",
            witness.argument, witness.sha256
        ));
    }
    let expected_path = repo_root.join(witness.relative_path);
    require_lexically_under_repo(repo_root, &expected_path, witness.argument)?;
    require_no_reparse_chain(&expected_path, false, witness.argument)?;
    let expected = expected_path.canonicalize().map_err(|error| {
        format!(
            "M2A-TRI-CONTROL-CLI-INPUT-MISSING: {}: {error}",
            expected_path.display()
        )
    })?;
    require_clean_path(&supplied.path, witness.argument)?;
    require_lexically_under_repo(repo_root, &supplied.path, witness.argument)?;
    require_no_reparse_chain(&supplied.path, false, witness.argument)?;
    let actual = supplied.path.canonicalize().map_err(|error| {
        format!(
            "M2A-TRI-CONTROL-CLI-INPUT-PATH: --{} {}: {error}",
            witness.argument,
            supplied.path.display()
        )
    })?;
    if !actual.starts_with(repo_root) {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-INPUT-PATH: --{} escaped the canonical repository",
            witness.argument
        ));
    }
    if actual != expected {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-INPUT-PATH: --{} must resolve to {}",
            witness.argument,
            display_path(&expected)
        ));
    }
    let bytes = fs::read(&actual).map_err(|error| {
        format!(
            "M2A-TRI-CONTROL-CLI-INPUT-READ: {}: {error}",
            display_path(&actual)
        )
    })?;
    let actual_sha256 = sha256(&bytes);
    if bytes.len() as u64 != witness.byte_length || actual_sha256 != witness.sha256 {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-INPUT-BYTE-IDENTITY: --{} expected {} bytes SHA-256 {}",
            witness.argument, witness.byte_length, witness.sha256
        ));
    }
    Ok(LoadedInput {
        binding: InputBindingV1 {
            role: witness.argument.to_owned(),
            path: display_path(&actual),
            byte_length: bytes.len() as u64,
            sha256: actual_sha256,
        },
        bytes,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublicationFaultV1 {
    None,
    #[cfg(test)]
    PartialWrite,
    #[cfg(test)]
    DestinationRace,
    #[cfg(test)]
    ForeignAfterStagingValidation,
    #[cfg(test)]
    ForeignAfterQuarantineValidation,
    #[cfg(test)]
    ReplaceExpectedChildDuringCleanup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CleanupInjectionV1 {
    None,
    #[cfg(test)]
    ForeignChild,
    #[cfg(test)]
    ReplaceExpectedChild,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CleanupModeV1 {
    Staging,
    PublishedUntrusted,
    Published,
}

struct OwnedDirectoryCleanupGuardV1 {
    repo_root: PathBuf,
    path: PathBuf,
    outputs: Vec<(String, u64, String)>,
    mode: CleanupModeV1,
    fault: PublicationFaultV1,
    armed: bool,
}

impl OwnedDirectoryCleanupGuardV1 {
    fn new(
        repo_root: &Path,
        path: &Path,
        outputs: &[(&str, &[u8])],
        mode: CleanupModeV1,
        fault: PublicationFaultV1,
    ) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            path: path.to_path_buf(),
            outputs: outputs
                .iter()
                .map(|(name, bytes)| ((*name).to_owned(), bytes.len() as u64, sha256(bytes)))
                .collect(),
            mode,
            fault,
            armed: true,
        }
    }

    fn transition_to_published(&mut self, path: &Path) {
        self.path = path.to_path_buf();
        self.mode = CleanupModeV1::PublishedUntrusted;
    }

    fn trust_published_identity(&mut self) {
        self.mode = CleanupModeV1::Published;
    }

    fn disarm(&mut self) {
        self.armed = false;
    }

    fn cleanup_now(&mut self) -> Result<(), String> {
        match self.mode {
            CleanupModeV1::Staging => remove_exact_owned_directory(
                &self.repo_root,
                &self.path,
                &self.outputs,
                false,
                CleanupInjectionV1::None,
            ),
            CleanupModeV1::PublishedUntrusted => {
                Err("M2A-TRI-CONTROL-CLI-CLEANUP-UNTRUSTED-PUBLICATION-LEFT-IN-PLACE".to_owned())
            }
            CleanupModeV1::Published => quarantine_and_remove_exact_publication(
                &self.repo_root,
                &self.path,
                &self.outputs,
                match self.fault {
                    #[cfg(test)]
                    PublicationFaultV1::ForeignAfterQuarantineValidation => {
                        CleanupInjectionV1::ForeignChild
                    }
                    #[cfg(test)]
                    PublicationFaultV1::ReplaceExpectedChildDuringCleanup => {
                        CleanupInjectionV1::ReplaceExpectedChild
                    }
                    _ => CleanupInjectionV1::None,
                },
            ),
        }
    }
}

impl Drop for OwnedDirectoryCleanupGuardV1 {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // All Result-returning paths call cleanup_now and surface its error.
        // Drop is only a panic fallback and makes no cleanup-success claim.
        let _panic_fallback = self.cleanup_now();
    }
}

fn publish_outputs_atomically_verified(
    repo_root: &Path,
    output_dir: &Path,
    outputs: &[(&str, &[u8])],
    fault: PublicationFaultV1,
    verify: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    require_no_reparse_chain(output_dir.parent().unwrap(), false, "publish parent")?;
    if path_entry_exists(output_dir)? {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-OUTPUT-EXISTS: {}",
            output_dir.display()
        ));
    }
    let parent = output_dir.parent().unwrap();
    let name = output_dir.file_name().unwrap().to_string_lossy();
    let nonce = PUBLICATION_NONCE.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".{name}.m2a-tri-control-stage-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| {
        format!(
            "M2A-TRI-CONTROL-CLI-STAGING-CREATE: {}: {error}",
            staging.display()
        )
    })?;
    let mut cleanup = OwnedDirectoryCleanupGuardV1::new(
        repo_root,
        &staging,
        outputs,
        CleanupModeV1::Staging,
        fault,
    );
    let operation = (|| {
        require_no_reparse_chain(&staging, false, "staging directory")?;
        for (index, (file_name, bytes)) in outputs.iter().enumerate() {
            #[cfg(not(test))]
            let _ = index;
            require_plain_file_name(file_name)?;
            let path = staging.join(file_name);
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|error| {
                    format!("M2A-TRI-CONTROL-CLI-WRITE: {}: {error}", path.display())
                })?;
            file.write_all(bytes).map_err(|error| {
                format!("M2A-TRI-CONTROL-CLI-WRITE: {}: {error}", path.display())
            })?;
            file.sync_all().map_err(|error| {
                format!("M2A-TRI-CONTROL-CLI-WRITE: {}: {error}", path.display())
            })?;
            #[cfg(test)]
            if index == 0 && fault == PublicationFaultV1::PartialWrite {
                return Err("M2A-TRI-CONTROL-CLI-TEST-PARTIAL-WRITE".to_owned());
            }
        }
        #[cfg(windows)]
        let staged = open_exact_owned_directory(repo_root, &staging, &cleanup.outputs, true)?;
        #[cfg(not(windows))]
        return Err("M2A-TRI-CONTROL-CLI-PUBLISH-UNSUPPORTED-PLATFORM".to_owned());
        #[cfg(test)]
        if fault == PublicationFaultV1::ForeignAfterStagingValidation {
            fs::write(staging.join("foreign-sentinel.txt"), b"user bytes")
                .map_err(|error| format!("M2A-TRI-CONTROL-CLI-TEST-FOREIGN: {error}"))?;
            return Err("M2A-TRI-CONTROL-CLI-TEST-STAGING-FOREIGN".to_owned());
        }
        #[cfg(windows)]
        let staged_identity = opened_identity_snapshot(&staged);
        #[cfg(windows)]
        drop(staged);
        #[cfg(test)]
        if fault == PublicationFaultV1::DestinationRace {
            fs::create_dir(output_dir).map_err(|error| {
                format!(
                    "M2A-TRI-CONTROL-CLI-TEST-RACE-CREATE: {}: {error}",
                    output_dir.display()
                )
            })?;
            fs::write(output_dir.join("attacker-sentinel.txt"), b"preserve me")
                .map_err(|error| format!("M2A-TRI-CONTROL-CLI-TEST-RACE-WRITE: {error}"))?;
        }
        publish_directory_noreplace(&staging, output_dir)?;
        cleanup.transition_to_published(output_dir);
        require_publish_postconditions(&staging, output_dir)?;
        #[cfg(windows)]
        let published = open_exact_owned_directory(repo_root, output_dir, &cleanup.outputs, true)?;
        #[cfg(windows)]
        require_snapshot_identity(&staged_identity, &published, "staging-to-published")?;
        #[cfg(windows)]
        drop(published);
        cleanup.trust_published_identity();
        verify()
    })();
    match operation {
        Ok(()) => {
            cleanup.disarm();
            Ok(())
        }
        Err(primary) => {
            let cleanup_result = cleanup.cleanup_now();
            cleanup.disarm();
            match cleanup_result {
                Ok(()) => Err(primary),
                Err(cleanup_error) => Err(format!(
                    "{primary}; M2A-TRI-CONTROL-CLI-CLEANUP: {cleanup_error}"
                )),
            }
        }
    }
}

fn publish_directory_noreplace(staging: &Path, destination: &Path) -> Result<(), String> {
    // The Win32 wrapper calls MoveFileExW with a zero flag word. The move is
    // therefore the atomic no-clobber decision; the earlier occupancy probe is
    // diagnostic only and is not relied upon for safety.
    #[cfg(windows)]
    m2a_win32_noreplace::move_directory_noreplace(staging, destination).map_err(|error| {
        format!(
            "M2A-TRI-CONTROL-CLI-PUBLISH: {}: {error}",
            destination.display()
        )
    })?;
    #[cfg(not(windows))]
    return Err("M2A-TRI-CONTROL-CLI-PUBLISH-UNSUPPORTED-PLATFORM".to_owned());
    Ok(())
}

fn require_publish_postconditions(source: &Path, destination: &Path) -> Result<(), String> {
    if path_entry_exists(source)? || !path_entry_exists(destination)? {
        return Err("M2A-TRI-CONTROL-CLI-PUBLISH-POSTCONDITION".to_owned());
    }
    let metadata = fs::symlink_metadata(destination)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-PUBLISH-METADATA: {error}"))?;
    if !metadata.file_type().is_dir() || metadata_is_reparse(&metadata) {
        return Err("M2A-TRI-CONTROL-CLI-PUBLISH-TYPE".to_owned());
    }
    Ok(())
}

fn verify_written_outputs(
    output_dir: &Path,
    outputs: &[(&str, &[u8])],
    contract: &DiagnosticContract,
    inputs: &BTreeMap<&'static str, LoadedInput>,
) -> Result<(), String> {
    let entries = fs::read_dir(output_dir)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-READBACK: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-READBACK: {error}"))?;
    if entries.len() != outputs.len()
        || entries
            .iter()
            .any(|entry| !entry.file_type().is_ok_and(|kind| kind.is_file()))
    {
        return Err(
            "M2A-TRI-CONTROL-CLI-READBACK: output must contain exactly four files".to_owned(),
        );
    }
    for (file_name, expected) in outputs {
        let actual = fs::read(output_dir.join(file_name))
            .map_err(|error| format!("M2A-TRI-CONTROL-CLI-READBACK: {file_name}: {error}"))?;
        if actual.as_slice() != *expected {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-READBACK: {file_name} byte drift"
            ));
        }
    }
    let identity = contract.module_identity();
    let module = fs::read(output_dir.join(format!("{}.mod", identity.module_resref)))
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-READBACK-MOD: {error}"))?;
    let hak = fs::read(output_dir.join(format!("{}.hak", identity.hak_resref)))
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-READBACK-HAK: {error}"))?;
    match contract {
        DiagnosticContract::RetailV1(contract) => verify_tri_control_diagnostic_v1(
            contract,
            &module,
            &hak,
            &inputs[BASE_APPEARANCE.argument].bytes,
            &inputs[R31_MODEL.argument].bytes,
            &inputs[R31_TEXTURE.argument].bytes,
            &inputs[H1_APPEARANCE.argument].bytes,
            &inputs[H1_MODEL.argument].bytes,
            &inputs[H1_TEXTURE.argument].bytes,
        )
        .map_err(|error| error.to_string()),
        DiagnosticContract::LastCityV2(contract) => verify_tri_control_diagnostic_last_city_v2(
            contract,
            &module,
            &hak,
            &inputs[LAST_CITY_BASE_APPEARANCE_V2.argument].bytes,
            &inputs[R31_MODEL.argument].bytes,
            &inputs[R31_TEXTURE.argument].bytes,
            &inputs[H1_APPEARANCE.argument].bytes,
            &inputs[H1_MODEL.argument].bytes,
            &inputs[H1_TEXTURE.argument].bytes,
        )
        .map_err(|error| error.to_string()),
    }
}

fn quarantine_and_remove_exact_publication(
    repo_root: &Path,
    output_dir: &Path,
    outputs: &[(String, u64, String)],
    injection: CleanupInjectionV1,
) -> Result<(), String> {
    #[cfg(not(windows))]
    return Err("M2A-TRI-CONTROL-CLI-QUARANTINE-UNSUPPORTED-PLATFORM".to_owned());
    #[cfg(windows)]
    let published = open_exact_owned_directory(repo_root, output_dir, outputs, true)?;
    #[cfg(windows)]
    let published_identity = opened_identity_snapshot(&published);
    #[cfg(windows)]
    drop(published);
    let parent = output_dir.parent().unwrap();
    let name = output_dir.file_name().unwrap().to_string_lossy();
    let nonce = PUBLICATION_NONCE.fetch_add(1, Ordering::Relaxed);
    let quarantine = parent.join(format!(
        ".{name}.m2a-tri-control-quarantine-{}-{nonce}",
        std::process::id()
    ));
    if path_entry_exists(&quarantine)? {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-QUARANTINE-EXISTS: {}",
            quarantine.display()
        ));
    }
    publish_directory_noreplace(output_dir, &quarantine)?;
    require_publish_postconditions(output_dir, &quarantine)?;
    #[cfg(windows)]
    let quarantined = open_exact_owned_directory(repo_root, &quarantine, outputs, true)?;
    #[cfg(windows)]
    require_snapshot_identity(&published_identity, &quarantined, "published-to-quarantine")?;
    #[cfg(windows)]
    apply_cleanup_injection(&quarantine, &quarantined, injection)?;
    #[cfg(windows)]
    require_opened_directory_unchanged(&quarantine, &quarantined, true)?;
    #[cfg(windows)]
    delete_opened_owned_directory(quarantined)?;
    if path_entry_exists(&quarantine)? || path_entry_exists(output_dir)? {
        return Err("M2A-TRI-CONTROL-CLI-QUARANTINE-POSTCONDITION".to_owned());
    }
    Ok(())
}

fn remove_exact_owned_directory(
    repo_root: &Path,
    directory: &Path,
    outputs: &[(String, u64, String)],
    require_all: bool,
    injection: CleanupInjectionV1,
) -> Result<(), String> {
    if !path_entry_exists(directory)? {
        return Ok(());
    }
    #[cfg(not(windows))]
    return Err("M2A-TRI-CONTROL-CLI-CLEANUP-UNSUPPORTED-PLATFORM".to_owned());
    #[cfg(windows)]
    let opened = open_exact_owned_directory(repo_root, directory, outputs, require_all)?;
    #[cfg(windows)]
    apply_cleanup_injection(directory, &opened, injection)?;
    #[cfg(windows)]
    require_opened_directory_unchanged(directory, &opened, require_all)?;
    #[cfg(windows)]
    delete_opened_owned_directory(opened)?;
    if path_entry_exists(directory)? {
        return Err("M2A-TRI-CONTROL-CLI-OWNED-CLEANUP-POSTCONDITION".to_owned());
    }
    Ok(())
}

#[cfg(windows)]
struct OpenedOwnedFileV1 {
    name: String,
    byte_length: u64,
    sha256: String,
    handle: m2a_win32_noreplace::DeleteHandleV1,
}

#[cfg(windows)]
struct OpenedOwnedDirectoryV1 {
    directory: m2a_win32_noreplace::DeleteHandleV1,
    files: Vec<OpenedOwnedFileV1>,
    expected_names: Vec<String>,
}

#[cfg(windows)]
struct OwnedIdentitySnapshotV1 {
    directory: m2a_win32_noreplace::FileIdentityV1,
    files: Vec<(String, m2a_win32_noreplace::FileIdentityV1)>,
}

#[cfg(windows)]
fn opened_identity_snapshot(opened: &OpenedOwnedDirectoryV1) -> OwnedIdentitySnapshotV1 {
    OwnedIdentitySnapshotV1 {
        directory: opened.directory.identity(),
        files: opened
            .files
            .iter()
            .map(|file| (file.name.clone(), file.handle.identity()))
            .collect(),
    }
}

#[cfg(windows)]
fn require_snapshot_identity(
    before: &OwnedIdentitySnapshotV1,
    after: &OpenedOwnedDirectoryV1,
    transition: &str,
) -> Result<(), String> {
    if before.directory != after.directory.identity()
        || before.files.len() != after.files.len()
        || before
            .files
            .iter()
            .zip(&after.files)
            .any(|(left, right)| left.0 != right.name || left.1 != right.handle.identity())
    {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-OWNED-TRANSITION-IDENTITY: {transition}"
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn open_exact_owned_directory(
    repo_root: &Path,
    directory: &Path,
    outputs: &[(String, u64, String)],
    require_all: bool,
) -> Result<OpenedOwnedDirectoryV1, String> {
    require_lexically_under_repo(repo_root, directory, "owned publication")?;
    require_no_reparse_chain(directory, false, "owned publication")?;
    let directory_handle = m2a_win32_noreplace::DeleteHandleV1::open_nofollow(directory, true)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-OWNED-DIR-HANDLE: {error}"))?;
    if !directory_handle.is_directory() {
        return Err("M2A-TRI-CONTROL-CLI-OWNED-TYPE".to_owned());
    }
    let expected = outputs
        .iter()
        .map(|(name, length, hash)| (name.as_str(), (*length, hash.as_str())))
        .collect::<BTreeMap<_, _>>();
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-OWNED-READ: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-OWNED-READ: {error}"))?;
    if require_all && entries.len() != expected.len() {
        return Err("M2A-TRI-CONTROL-CLI-OWNED-COUNT".to_owned());
    }
    let mut files = Vec::with_capacity(entries.len());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some((length, hash)) = expected.get(name.as_str()) else {
            return Err(format!("M2A-TRI-CONTROL-CLI-OWNED-UNEXPECTED: {name}"));
        };
        let handle = m2a_win32_noreplace::DeleteHandleV1::open_nofollow(&entry.path(), false)
            .map_err(|error| format!("M2A-TRI-CONTROL-CLI-OWNED-ENTRY-HANDLE: {name}: {error}"))?;
        let bytes = handle
            .read_all()
            .map_err(|error| format!("M2A-TRI-CONTROL-CLI-OWNED-ENTRY-READ: {name}: {error}"))?;
        if bytes.len() as u64 != *length || sha256(&bytes) != *hash {
            return Err(format!("M2A-TRI-CONTROL-CLI-OWNED-IDENTITY: {name}"));
        }
        files.push(OpenedOwnedFileV1 {
            name,
            byte_length: *length,
            sha256: (*hash).to_owned(),
            handle,
        });
    }
    let directory_probe = m2a_win32_noreplace::DeleteHandleV1::open_nofollow(directory, true)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-OWNED-DIR-REOPEN: {error}"))?;
    if directory_probe.identity() != directory_handle.identity() {
        return Err("M2A-TRI-CONTROL-CLI-OWNED-DIR-REPLACED".to_owned());
    }
    let mut expected_names = expected
        .keys()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    expected_names.sort();
    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(OpenedOwnedDirectoryV1 {
        directory: directory_handle,
        files,
        expected_names,
    })
}

#[cfg(windows)]
fn apply_cleanup_injection(
    directory: &Path,
    opened: &OpenedOwnedDirectoryV1,
    injection: CleanupInjectionV1,
) -> Result<(), String> {
    #[cfg(not(test))]
    let _ = (directory, opened);
    match injection {
        CleanupInjectionV1::None => Ok(()),
        #[cfg(test)]
        CleanupInjectionV1::ForeignChild => {
            fs::write(directory.join("foreign-sentinel.txt"), b"user bytes")
                .map_err(|error| format!("M2A-TRI-CONTROL-CLI-TEST-FOREIGN: {error}"))
        }
        #[cfg(test)]
        CleanupInjectionV1::ReplaceExpectedChild => {
            let first = opened
                .files
                .first()
                .ok_or_else(|| "M2A-TRI-CONTROL-CLI-TEST-REPLACE-NO-FILE".to_owned())?;
            let path = directory.join(&first.name);
            fs::remove_file(&path)
                .map_err(|error| format!("M2A-TRI-CONTROL-CLI-TEST-REPLACE-REMOVE: {error}"))?;
            fs::write(&path, b"replacement user bytes")
                .map_err(|error| format!("M2A-TRI-CONTROL-CLI-TEST-REPLACE-WRITE: {error}"))
        }
    }
}

#[cfg(windows)]
fn require_opened_directory_unchanged(
    directory: &Path,
    opened: &OpenedOwnedDirectoryV1,
    require_all: bool,
) -> Result<(), String> {
    let mut current_names = fs::read_dir(directory)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-CLEANUP-READ: {error}"))?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-CLEANUP-READ: {error}"))?;
    current_names.sort();
    let mut opened_names = opened
        .files
        .iter()
        .map(|file| file.name.clone())
        .collect::<Vec<_>>();
    opened_names.sort();
    if current_names != opened_names || (require_all && current_names != opened.expected_names) {
        return Err(format!(
            "M2A-TRI-CONTROL-CLI-CLEANUP-FOREIGN-OR-MISSING: {current_names:?}"
        ));
    }
    for file in &opened.files {
        let bytes = file.handle.read_all().map_err(|error| {
            format!(
                "M2A-TRI-CONTROL-CLI-CLEANUP-READ-HANDLE: {}: {error}",
                file.name
            )
        })?;
        if bytes.len() as u64 != file.byte_length || sha256(&bytes) != file.sha256 {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-CLEANUP-HANDLE-DRIFT: {}",
                file.name
            ));
        }
        let probe =
            m2a_win32_noreplace::DeleteHandleV1::open_nofollow(&directory.join(&file.name), false)
                .map_err(|error| {
                    format!("M2A-TRI-CONTROL-CLI-CLEANUP-REOPEN: {}: {error}", file.name)
                })?;
        if probe.identity() != file.handle.identity() {
            return Err(format!(
                "M2A-TRI-CONTROL-CLI-CLEANUP-REPLACED: {}",
                file.name
            ));
        }
    }
    let directory_probe = m2a_win32_noreplace::DeleteHandleV1::open_nofollow(directory, true)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-CLEANUP-DIR-REOPEN: {error}"))?;
    if directory_probe.identity() != opened.directory.identity() {
        return Err("M2A-TRI-CONTROL-CLI-CLEANUP-DIR-REPLACED".to_owned());
    }
    Ok(())
}

#[cfg(windows)]
fn delete_opened_owned_directory(opened: OpenedOwnedDirectoryV1) -> Result<(), String> {
    for file in opened.files {
        file.handle.delete().map_err(|error| {
            format!(
                "M2A-TRI-CONTROL-CLI-CLEANUP-DELETE-FILE: {}: {error}",
                file.name
            )
        })?;
    }
    opened
        .directory
        .delete()
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-CLEANUP-DELETE-DIR: {error}"))
}

fn require_plain_file_name(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    if path.components().count() != 1
        || path.file_name().and_then(|name| name.to_str()) != Some(value)
    {
        return Err(format!("M2A-TRI-CONTROL-CLI-OUTPUT-NAME: {value}"));
    }
    Ok(())
}

fn path_entry_exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "M2A-TRI-CONTROL-CLI-PATH-PROBE: {}: {error}",
            path.display()
        )),
    }
}

fn require_clean_path(path: &Path, label: &str) -> Result<(), String> {
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir | std::path::Component::CurDir
        )
    }) {
        return Err(format!("M2A-TRI-CONTROL-CLI-PATH-DOT: {label}"));
    }
    Ok(())
}

fn require_lexically_under_repo(repo_root: &Path, path: &Path, label: &str) -> Result<(), String> {
    require_clean_path(path, label)?;
    let root = normalized_windows_path(repo_root);
    let candidate = normalized_windows_path(path);
    if candidate != root && !candidate.starts_with(&(root + "\\")) {
        return Err(format!("M2A-TRI-CONTROL-CLI-PATH-OUTSIDE: {label}"));
    }
    Ok(())
}

fn normalized_windows_path(path: &Path) -> String {
    display_path(path).replace('/', "\\").to_ascii_lowercase()
}

fn require_no_reparse_chain(
    path: &Path,
    allow_missing_final: bool,
    label: &str,
) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("M2A-TRI-CONTROL-CLI-PATH-ABSOLUTE: {label}"));
    }
    require_clean_path(path, label)?;
    let mut chain = path.ancestors().collect::<Vec<_>>();
    chain.reverse();
    for (index, ancestor) in chain.iter().enumerate() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                if metadata_is_reparse(&metadata) {
                    return Err(format!(
                        "M2A-TRI-CONTROL-CLI-REPARSE: {label}: {}",
                        ancestor.display()
                    ));
                }
                if index + 1 < chain.len() && !metadata.file_type().is_dir() {
                    return Err(format!(
                        "M2A-TRI-CONTROL-CLI-PATH-ANCESTOR-TYPE: {label}: {}",
                        ancestor.display()
                    ));
                }
            }
            Err(error)
                if error.kind() == ErrorKind::NotFound
                    && allow_missing_final
                    && index + 1 == chain.len() => {}
            Err(error) => {
                return Err(format!(
                    "M2A-TRI-CONTROL-CLI-PATH-METADATA: {label}: {}: {error}",
                    ancestor.display()
                ));
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_attributes() & WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationTestFaultV1 {
    PartialWrite,
    DestinationRace,
    PostPublishVerify,
    ForeignAfterStagingValidation,
    ForeignAfterQuarantineValidation,
    ReplaceExpectedChildDuringCleanup,
}

#[cfg(test)]
pub fn test_publish_tiny_outputs(
    output_dir: &Path,
    fault: PublicationTestFaultV1,
) -> Result<(), String> {
    let repo_root = canonical_repo_root()?;
    let output_dir = resolve_output_dir(&repo_root, output_dir)?;
    let outputs: [(&str, &[u8]); 2] = [("one.bin", b"one"), ("two.bin", b"two")];
    let internal_fault = match fault {
        PublicationTestFaultV1::PartialWrite => PublicationFaultV1::PartialWrite,
        PublicationTestFaultV1::DestinationRace => PublicationFaultV1::DestinationRace,
        PublicationTestFaultV1::PostPublishVerify => PublicationFaultV1::None,
        PublicationTestFaultV1::ForeignAfterStagingValidation => {
            PublicationFaultV1::ForeignAfterStagingValidation
        }
        PublicationTestFaultV1::ForeignAfterQuarantineValidation => {
            PublicationFaultV1::ForeignAfterQuarantineValidation
        }
        PublicationTestFaultV1::ReplaceExpectedChildDuringCleanup => {
            PublicationFaultV1::ReplaceExpectedChildDuringCleanup
        }
    };
    let verification = || {
        if matches!(
            fault,
            PublicationTestFaultV1::PostPublishVerify
                | PublicationTestFaultV1::ForeignAfterQuarantineValidation
                | PublicationTestFaultV1::ReplaceExpectedChildDuringCleanup
        ) {
            Err("M2A-TRI-CONTROL-CLI-TEST-VERIFY".to_owned())
        } else {
            Ok(())
        }
    };
    publish_outputs_atomically_verified(
        &repo_root,
        &output_dir,
        &outputs,
        internal_fault,
        verification,
    )
}

fn output_binding(file_name: &str, bytes: &[u8]) -> OutputBindingV1 {
    OutputBindingV1 {
        file_name: file_name.to_owned(),
        byte_length: bytes.len() as u64,
        sha256: sha256(bytes),
    }
}

fn canonical_json(value: &impl Serialize, name: &str) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("M2A-TRI-CONTROL-CLI-{name}-JSON: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn display_path(path: &Path) -> String {
    let value = path.display().to_string();
    value.strip_prefix(r"\\?\").unwrap_or(&value).to_owned()
}

fn usage() -> String {
    "usage: materialize_tri_control_diagnostic [--profile retail-v1|last-city-v2] --outDir <absent-project-dir> --module-resref <fresh-resref> --area-resref <fresh-resref> --hak-resref <fresh-resref> --base-appearance <exact-canonical-path> --base-appearance-sha256 <sha256> --r31-model <exact-path> --r31-model-sha256 <sha256> --r31-texture <exact-path> --r31-texture-sha256 <sha256> --h1-appearance <exact-path> --h1-appearance-sha256 <sha256> --h1-model <exact-path> --h1-model-sha256 <sha256> --h1-texture <exact-path> --h1-texture-sha256 <sha256>".to_owned()
}
