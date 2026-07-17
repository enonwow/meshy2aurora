use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::read_gff_v32,
};
use serde_json::json;

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
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: inspect_module <module.mod>".to_owned())?;
    let bytes = fs::read(&path).map_err(|error| format!("module read failed: {error}"))?;
    let archive = ErfArchive::parse(&bytes).map_err(|error| error.to_string())?;
    if archive.file_type() != ErfFileType::Module {
        return Err("input is not MOD V1.0".to_owned());
    }

    let mut resources = Vec::new();
    for resource in archive.resources() {
        let payload = archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| error.to_string())?;
        let document = match resource.resource_type {
            2012 | 2014 | 2023 | 2027 | 2038 | 2046 => Some(
                read_gff_v32(payload, &Default::default()).map_err(|error| error.to_string())?,
            ),
            _ => None,
        };
        resources.push(json!({
            "descriptor": resource,
            "document": document,
        }));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "path": path,
            "byteLength": bytes.len(),
            "resources": resources,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}
