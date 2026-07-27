use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::animated_donor_candidate::{
    H2_R42_AREA_RESREF, H2_R42_HAK_RESREF, H2_R42_MODEL_RESREF, H2_R42_MODULE_RESREF,
    H2_R42_TEXTURE_RESREF, build_h2_r42_visibility_candidate_v1,
    verify_h2_r42_visibility_candidate_v1,
};
use sha2::{Digest, Sha256};

const CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\h2-r42-root-rigid-type0-20260724";
const CONTRACT_FILE: &str = "h2-r42-root-rigid-type0-lineage-contract-v1.json";

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
    let command = parse(env::args().skip(1))?;
    if command.output.as_path() != Path::new(CANONICAL_OUTPUT) {
        return Err(format!(
            "M2A-H2-R42-OUTPUT-IDENTITY: exact output must be {CANONICAL_OUTPUT}"
        ));
    }
    if command.output.exists() {
        return Err(format!(
            "M2A-H2-R42-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }
    let source = read(&command.source_glb, "source")?;
    let appearance = read(&command.appearance_two_da, "appearance")?;
    let artifact = build_h2_r42_visibility_candidate_v1(&source, &appearance)
        .map_err(|error| error.to_string())?;
    verify_h2_r42_visibility_candidate_v1(
        &artifact.contract,
        &source,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .map_err(|error| error.to_string())?;

    let generated = command.output.join("generated");
    fs::create_dir_all(&generated).map_err(|error| format!("M2A-H2-R42-OUTPUT-CREATE: {error}"))?;
    let outputs = [
        (
            generated.join(format!("{H2_R42_MODEL_RESREF}.mdl")),
            artifact.model.as_slice(),
        ),
        (
            generated.join(format!("{H2_R42_TEXTURE_RESREF}.tga")),
            artifact.texture.as_slice(),
        ),
        (
            generated.join("appearance.2da"),
            artifact.appearance_two_da.as_slice(),
        ),
        (
            generated.join(format!("{H2_R42_HAK_RESREF}.hak")),
            artifact.hak.as_slice(),
        ),
        (
            generated.join(format!("{H2_R42_MODULE_RESREF}.mod")),
            artifact.module.as_slice(),
        ),
        (
            command.output.join(CONTRACT_FILE),
            artifact.contract_json.as_slice(),
        ),
    ];
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        let actual = read(path, "writtenOutput")?;
        if actual != *expected {
            return Err(format!("M2A-H2-R42-OUTPUT-READBACK: {}", path.display()));
        }
    }

    serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "status": "H2_R42_ROOT_RIGID_TYPE0_CANDIDATE_OFFLINE_MATERIALIZED",
        "outputDirectory": command.output,
        "module": {
            "fileName": format!("{H2_R42_MODULE_RESREF}.mod"),
            "resref": H2_R42_MODULE_RESREF,
            "areaResref": H2_R42_AREA_RESREF,
            "byteLength": artifact.module.len(),
            "sha256": sha256(&artifact.module),
        },
        "orderedHakList": [{
            "fileName": format!("{H2_R42_HAK_RESREF}.hak"),
            "resref": H2_R42_HAK_RESREF,
            "byteLength": artifact.hak.len(),
            "sha256": sha256(&artifact.hak),
        }],
        "model": artifact.contract.model,
        "texture": artifact.contract.texture,
        "appearanceTwoDa": artifact.contract.appearance_two_da,
        "triangleCount": artifact.contract.triangle_count,
        "rigNodeCount": artifact.contract.rig_node_count,
        "animationNames": artifact.contract.animation_names,
        "stateProjectionProfile": artifact.contract.state_projection_profile,
        "contractSha256": sha256(&artifact.contract_json),
        "materializationCount": 1,
        "startsToolset": false,
        "startsNwn": false,
        "proofRunCreated": false,
    }))
    .map_err(|error| format!("M2A-H2-R42-SUMMARY: {error}"))
}

struct Command {
    source_glb: PathBuf,
    appearance_two_da: PathBuf,
    output: PathBuf,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut appearance_two_da = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--appearance-2da" => &mut appearance_two_da,
            "--out" => &mut output,
            _ => {
                return Err(format!("M2A-H2-R42-ARGUMENT: {argument}\n{}", usage()));
            }
        };
        if target.is_some() {
            return Err(format!("M2A-H2-R42-ARGUMENT-DUPLICATE: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("M2A-H2-R42-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("M2A-H2-R42-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("M2A-H2-R42-OUTPUT-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("M2A-H2-R42-OUTPUT-WRITE {}: {error}", path.display()))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn usage() -> String {
    "usage: materialize_h2_r42_visibility_candidate --source-glb <exact-H2.glb> --appearance-2da <exact-full-table> --out <exact-r42-dir>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::{CANONICAL_OUTPUT, parse};

    #[test]
    fn requires_exact_inputs_and_has_no_identity_override() {
        let arguments = [
            "--source-glb",
            "source.glb",
            "--appearance-2da",
            "appearance.2da",
            "--out",
            CANONICAL_OUTPUT,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        assert!(parse(arguments.clone()).is_ok());
        for missing in (0..arguments.len()).step_by(2) {
            let mut changed = arguments.clone();
            changed.drain(missing..missing + 2);
            assert!(parse(changed).is_err());
        }
        let mut forbidden = arguments;
        forbidden.extend(["--model-resref".to_owned(), "other".to_owned()]);
        assert!(parse(forbidden).is_err());
    }
}
