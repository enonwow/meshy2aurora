//! Materializes one fresh runtime-complete MOD around the immutable r31 HAK.
//! No HAK, model, texture, 2DA, runtime profile, Toolset process, or NWN
//! process is created or modified by this example.

use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    hierarchy_candidate::{
        M0_R31_HAK_RESREF, M0_R31_HAK_SHA256, M0_R32_AREA_RESREF, M0_R32_MODULE_RESREF,
        build_meshy_r31_corrected_container_module_v1,
    },
    proof_module::BinaryM0VerticalSliceIdentityV1,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const R31_MODULE_SHA256: &str = "8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd";
const CONTRACT_FILE: &str = "m0-r32-corrected-container-lineage-contract-v1.json";

#[derive(Debug)]
struct Arguments {
    output: PathBuf,
    r31_module: PathBuf,
    r31_hak: PathBuf,
    installed_r31_hak: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Contract<'a> {
    version: &'static str,
    id: &'static str,
    status: &'static str,
    source_r31_module: FileBinding<'a>,
    corrected_module: FileBinding<'a>,
    ordered_hak_list: [HakBinding<'a>; 1],
    area: AreaBinding,
    fixture: FixtureBinding,
    intended_delta: IntendedDelta,
    materialization: Materialization,
    proof_state: ProofState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileBinding<'a> {
    path: &'a str,
    byte_length: u64,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HakBinding<'a> {
    resref: &'static str,
    source_path: &'a str,
    installed_path: &'a str,
    byte_length: u64,
    sha256: &'static str,
    reused_byte_identical: bool,
    copied_or_rebuilt: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AreaBinding {
    resref: &'static str,
    width: u32,
    height: u32,
    entry_point: [f32; 3],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureBinding {
    id: &'static str,
    template_resref: &'static str,
    tree_object_text: &'static str,
    appearance_row: u16,
    position: [f32; 3],
    orientation: [f32; 2],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IntendedDelta {
    only: [&'static str; 2],
    model_delta: bool,
    texture_delta: bool,
    appearance_two_da_delta: bool,
    hak_delta: bool,
    historical_r31_sparse_rejection: &'static str,
    corrected_container_readback: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Materialization {
    materialization_count: u32,
    starts_toolset: bool,
    starts_nwn: bool,
    runtime_profile_materialized: bool,
    generated_hak: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofState {
    toolset: Axis,
    nwn: Axis,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Axis {
    model_visibility: &'static str,
    proof_completeness: &'static str,
}

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let args = parse(env::args().skip(1))?;
    if args.output.exists() {
        return Err(format!(
            "M2A-R32-CORRECTED-CONTAINER-OUTPUT-EXISTS: {}",
            args.output.display()
        ));
    }
    let r31_module = read_exact(&args.r31_module, R31_MODULE_SHA256, "r31Module")?;
    let r31_hak = read_exact(&args.r31_hak, M0_R31_HAK_SHA256, "r31Hak")?;
    let installed_r31_hak = read_exact(
        &args.installed_r31_hak,
        M0_R31_HAK_SHA256,
        "installedR31Hak",
    )?;
    if installed_r31_hak != r31_hak {
        return Err("M2A-R32-CORRECTED-CONTAINER-HAK-BYTE-IDENTITY".to_owned());
    }

    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: M0_R32_MODULE_RESREF.to_owned(),
        area_resref: M0_R32_AREA_RESREF.to_owned(),
        hak_resref: M0_R31_HAK_RESREF.to_owned(),
    };
    let module = build_meshy_r31_corrected_container_module_v1(&identity)
        .map_err(|error| error.to_string())?;
    let generated = args.output.join("generated");
    fs::create_dir_all(&generated).map_err(|error| format!("create output: {error}"))?;
    let module_path = generated.join("m2a_m0r32.mod");
    fs::write(&module_path, &module.payload).map_err(|error| format!("write MOD: {error}"))?;
    let written_module = fs::read(&module_path).map_err(|error| format!("reread MOD: {error}"))?;
    if written_module != module.payload {
        return Err("M2A-R32-CORRECTED-CONTAINER-MOD-READBACK".to_owned());
    }

    let r31_module_path = display(&args.r31_module);
    let module_path_text = display(&module_path);
    let r31_hak_path = display(&args.r31_hak);
    let installed_hak_path = display(&args.installed_r31_hak);
    let contract = Contract {
        version: "m2a-m0-r32-corrected-container-lineage-contract/v1",
        id: "m2a-m0-r32-corrected-container",
        status: "offline_materialized_gate_b_pending",
        source_r31_module: FileBinding {
            path: &r31_module_path,
            byte_length: r31_module.len() as u64,
            sha256: sha256(&r31_module),
        },
        corrected_module: FileBinding {
            path: &module_path_text,
            byte_length: module.payload.len() as u64,
            sha256: sha256(&module.payload),
        },
        ordered_hak_list: [HakBinding {
            resref: M0_R31_HAK_RESREF,
            source_path: &r31_hak_path,
            installed_path: &installed_hak_path,
            byte_length: r31_hak.len() as u64,
            sha256: M0_R31_HAK_SHA256,
            reused_byte_identical: true,
            copied_or_rebuilt: false,
        }],
        area: AreaBinding {
            resref: M0_R32_AREA_RESREF,
            width: 2,
            height: 2,
            entry_point: [10.0, 10.0, 0.0],
        },
        fixture: FixtureBinding {
            id: "m0_fixture",
            template_resref: "nw_dwarfmerc001",
            tree_object_text: "Meshy M0 binary vertical-slice fixture",
            appearance_row: 15_100,
            position: [10.0, 14.5, 0.0],
            orientation: [1.0, 0.0],
        },
        intended_delta: IntendedDelta {
            only: ["MOD_AREA_IDENTITY", "RUNTIME_COMPLETE_CREATURE_ENVELOPE"],
            model_delta: false,
            texture_delta: false,
            appearance_two_da_delta: false,
            hak_delta: false,
            historical_r31_sparse_rejection: "area.git.Creature List[0].MaxHitPoints requires 13",
            corrected_container_readback: "independent generic parser PASS",
        },
        materialization: Materialization {
            materialization_count: 1,
            starts_toolset: false,
            starts_nwn: false,
            runtime_profile_materialized: false,
            generated_hak: false,
        },
        proof_state: ProofState {
            toolset: Axis {
                model_visibility: "not_tested",
                proof_completeness: "missing",
            },
            nwn: Axis {
                model_visibility: "not_tested",
                proof_completeness: "missing",
            },
        },
    };
    let mut contract_bytes = serde_json::to_vec_pretty(&contract)
        .map_err(|error| format!("serialize contract: {error}"))?;
    contract_bytes.push(b'\n');
    fs::write(args.output.join(CONTRACT_FILE), &contract_bytes)
        .map_err(|error| format!("write contract: {error}"))?;
    Ok(format!(
        "m0_r32_corrected_container_materialized moduleSha256={} contractSha256={} noHakWritten=true",
        sha256(&module.payload),
        sha256(&contract_bytes)
    ))
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut output = None;
    let mut r31_module = None;
    let mut r31_hak = None;
    let mut installed_r31_hak = None;
    let mut args = arguments.into_iter();
    while let Some(argument) = args.next() {
        let slot = match argument.as_str() {
            "--out" => &mut output,
            "--r31-module" => &mut r31_module,
            "--r31-hak" => &mut r31_hak,
            "--installed-r31-hak" => &mut installed_r31_hak,
            _ => return Err(format!("M2A-R32-CORRECTED-CONTAINER-ARG: {argument}")),
        };
        *slot = Some(PathBuf::from(
            args.next()
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("M2A-R32-CORRECTED-CONTAINER-ARG-VALUE: {argument}"))?,
        ));
    }
    Ok(Arguments {
        output: output.ok_or_else(usage)?,
        r31_module: r31_module.ok_or_else(usage)?,
        r31_hak: r31_hak.ok_or_else(usage)?,
        installed_r31_hak: installed_r31_hak.ok_or_else(usage)?,
    })
}

fn usage() -> String {
    "usage: materialize_m0_corrected_container --out <absent-dir> --r31-module <path> --r31-hak <path> --installed-r31-hak <path>".to_owned()
}

fn read_exact(path: &PathBuf, expected_sha256: &str, label: &str) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| format!("{label} read: {error}"))?;
    let actual = sha256(&bytes);
    if actual != expected_sha256 {
        return Err(format!(
            "M2A-R32-CORRECTED-CONTAINER-INPUT-IDENTITY: {label}: expected {expected_sha256}, got {actual}"
        ));
    }
    Ok(bytes)
}

fn display(path: &PathBuf) -> String {
    path.display().to_string()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
