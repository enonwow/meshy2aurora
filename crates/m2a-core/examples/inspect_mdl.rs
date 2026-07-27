use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::inspect_binary_mdl;

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
        .ok_or_else(|| "usage: inspect_mdl <model.mdl>".to_owned())?;
    let payload = fs::read(&path).map_err(|error| format!("model read failed: {error}"))?;
    let inspection = inspect_binary_mdl(&payload).map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&inspection).map_err(|error| error.to_string())?
    );
    Ok(())
}
