use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    process::ExitCode,
};

use m2a_core::gff::{GffLimitsV1, read_gff_v32};
use serde_json::json;
use sha2::{Digest, Sha256};

fn main() -> ExitCode {
    match run() {
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

fn run() -> Result<String, String> {
    let command = parse(env::args().skip(1))?;
    let mut file = File::open(&command.path)
        .map_err(|error| format!("GFF-RANGE-OPEN {}: {error}", command.path.display()))?;
    let file_length = file
        .metadata()
        .map_err(|error| format!("GFF-RANGE-METADATA {}: {error}", command.path.display()))?
        .len();
    let end = command
        .offset
        .checked_add(command.size as u64)
        .ok_or_else(|| "GFF-RANGE-OVERFLOW: offset + size exceeds u64".to_owned())?;
    if end > file_length {
        return Err(format!(
            "GFF-RANGE-OOB: requested {end} bytes from a {file_length}-byte file"
        ));
    }
    file.seek(SeekFrom::Start(command.offset))
        .map_err(|error| format!("GFF-RANGE-SEEK: {error}"))?;
    let mut payload = vec![0; command.size];
    file.read_exact(&mut payload)
        .map_err(|error| format!("GFF-RANGE-READ: {error}"))?;
    let document = read_gff_v32(&payload, &GffLimitsV1::default())
        .map_err(|error| format!("GFF-RANGE-PARSE: {error}"))?;
    serde_json::to_string_pretty(&json!({
        "path": command.path,
        "offset": command.offset,
        "size": command.size,
        "sha256": sha256(&payload),
        "document": document,
    }))
    .map_err(|error| format!("GFF-RANGE-JSON: {error}"))
}

struct Command {
    path: PathBuf,
    offset: u64,
    size: usize,
}

fn parse(arguments: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Command, String> {
    let values = arguments
        .into_iter()
        .map(|value| value.as_ref().to_owned())
        .collect::<Vec<_>>();
    if values.len() != 3 {
        return Err(usage());
    }
    let offset = values[1]
        .parse::<u64>()
        .map_err(|_| "GFF-RANGE-OFFSET: expected an unsigned integer".to_owned())?;
    let size = values[2]
        .parse::<usize>()
        .map_err(|_| "GFF-RANGE-SIZE: expected a positive integer".to_owned())?;
    if size == 0 {
        return Err("GFF-RANGE-SIZE: expected a positive integer".to_owned());
    }
    Ok(Command {
        path: PathBuf::from(&values[0]),
        offset,
        size,
    })
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn usage() -> String {
    "usage: inspect_gff_range <container.bif> <offset> <size>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn requires_exact_bif_path_offset_and_size() {
        assert!(parse(["xp2_templates.bif", "3858039", "3457"]).is_ok());
        assert!(parse(["xp2_templates.bif", "3858039"]).is_err());
        assert!(parse(["xp2_templates.bif", "3858039", "0"]).is_err());
        assert!(parse(["xp2_templates.bif", "offset", "3457"]).is_err());
        assert!(parse(["xp2_templates.bif", "3858039", "3457", "extra"]).is_err());
    }
}
