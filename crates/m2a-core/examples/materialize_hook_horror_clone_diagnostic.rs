use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    hook_horror_clone_diagnostic::{
        build_hook_horror_clone_diagnostic_v1, verify_hook_horror_clone_diagnostic_v1,
    },
    proof_module::BinaryM0VerticalSliceIdentityV1,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let command = parse_command(env::args().skip(1))?;
    if command.out_dir.exists() {
        return Err(format!(
            "M2A-HOOK-HORROR-DESTINATION-EXISTS: {}",
            command.out_dir.display()
        ));
    }
    let source = fs::read(&command.source_appearance).map_err(|error| {
        format!(
            "M2A-HOOK-HORROR-SOURCE-READ {}: {error}",
            command.source_appearance.display()
        )
    })?;
    let artifact = build_hook_horror_clone_diagnostic_v1(&source, &command.identity)
        .map_err(|error| error.to_string())?;
    verify_hook_horror_clone_diagnostic_v1(&artifact, &source, &command.identity)
        .map_err(|error| error.to_string())?;

    let generated = command.out_dir.join("generated");
    fs::create_dir_all(&generated).map_err(|error| {
        format!(
            "M2A-HOOK-HORROR-DESTINATION-CREATE {}: {error}",
            generated.display()
        )
    })?;
    let module_path = generated.join(format!("{}.mod", command.identity.module_resref));
    let hak_path = generated.join(format!("{}.hak", command.identity.hak_resref));
    let appearance_path = generated.join("appearance.2da");
    let contract_path = command
        .out_dir
        .join("hook-horror-clone-diagnostic-contract-v1.json");
    fs::write(&module_path, &artifact.module).map_err(write_error(&module_path))?;
    fs::write(&hak_path, &artifact.hak).map_err(write_error(&hak_path))?;
    fs::write(&appearance_path, &artifact.appearance_two_da)
        .map_err(write_error(&appearance_path))?;
    let mut contract = serde_json::to_vec_pretty(&artifact.contract)
        .map_err(|error| format!("M2A-HOOK-HORROR-CONTRACT-JSON: {error}"))?;
    contract.push(b'\n');
    fs::write(&contract_path, contract).map_err(write_error(&contract_path))?;

    println!(
        "{{\"ok\":true,\"modulePath\":\"{}\",\"moduleSha256\":\"{}\",\"hakPath\":\"{}\",\"hakSha256\":\"{}\",\"appearancePath\":\"{}\",\"appearanceSha256\":\"{}\",\"appearanceRow\":{},\"contractPath\":\"{}\"}}",
        module_path.display(),
        artifact.contract.module.sha256,
        hak_path.display(),
        artifact.contract.hak.sha256,
        appearance_path.display(),
        artifact.contract.output_appearance.sha256,
        artifact.contract.clone_row,
        contract_path.display(),
    );
    Ok(())
}

struct Command {
    source_appearance: PathBuf,
    out_dir: PathBuf,
    identity: BinaryM0VerticalSliceIdentityV1,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_appearance = None;
    let mut out_dir = None;
    let mut module_resref = None;
    let mut area_resref = None;
    let mut hak_resref = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let slot = match argument.as_str() {
            "--source-appearance" => &mut source_appearance,
            "--out-dir" => &mut out_dir,
            "--module-resref" => &mut module_resref,
            "--area-resref" => &mut area_resref,
            "--hak-resref" => &mut hak_resref,
            "--help" | "-h" => return Err(usage()),
            _ => {
                return Err(format!(
                    "M2A-HOOK-HORROR-ARGUMENT-UNKNOWN: {argument}\n{}",
                    usage()
                ));
            }
        };
        if slot.is_some() {
            return Err(format!("M2A-HOOK-HORROR-ARGUMENT-DUPLICATE: {argument}"));
        }
        *slot = values.next().filter(|value| !value.starts_with("--"));
        if slot.is_none() {
            return Err(format!("M2A-HOOK-HORROR-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        source_appearance: PathBuf::from(source_appearance.ok_or_else(usage)?),
        out_dir: PathBuf::from(out_dir.ok_or_else(usage)?),
        identity: BinaryM0VerticalSliceIdentityV1 {
            module_resref: module_resref.ok_or_else(usage)?,
            area_resref: area_resref.ok_or_else(usage)?,
            hak_resref: hak_resref.ok_or_else(usage)?,
        },
    })
}

fn write_error(path: &PathBuf) -> impl FnOnce(std::io::Error) -> String + '_ {
    move |error| format!("M2A-HOOK-HORROR-WRITE {}: {error}", path.display())
}

fn usage() -> String {
    "usage: materialize_hook_horror_clone_diagnostic --source-appearance <exact-path> --out-dir <absent-dir> --module-resref <fresh-resref> --area-resref <fresh-resref> --hak-resref <fresh-resref>".to_owned()
}
