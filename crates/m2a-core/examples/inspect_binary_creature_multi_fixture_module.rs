use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::proof_module::inspect_binary_creature_multi_fixture_module_v1;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: inspect_binary_creature_multi_fixture_module <module.mod>");
        return ExitCode::FAILURE;
    };
    let result = fs::read(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))
        .and_then(|bytes| {
            inspect_binary_creature_multi_fixture_module_v1(&bytes)
                .map_err(|error| error.to_string())
        })
        .and_then(|report| {
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
        });
    match result {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
