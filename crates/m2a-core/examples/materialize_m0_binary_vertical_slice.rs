use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::proof_module::{
    BinaryM0VerticalSliceIdentityV1, build_binary_m0_vertical_slice_module_with_identity_v1,
};

const DEFAULT_M0_APPEARANCE_ROW: u16 = 848;

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
    if command.output.exists() {
        return Err(format!(
            "M0-BINARY-VERTICAL-SLICE-DESTINATION-EXISTS: {}",
            command.output.display()
        ));
    }
    let parent = command
        .output
        .parent()
        .ok_or_else(|| "M0-BINARY-VERTICAL-SLICE-DESTINATION-HAS-NO-PARENT".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "M0-BINARY-VERTICAL-SLICE-CREATE-PARENT-FAILED {}: {error}",
            parent.display()
        )
    })?;

    let artifact = build_binary_m0_vertical_slice_module_with_identity_v1(
        command.appearance_row,
        &command.identity,
    )
    .map_err(|error| error.to_string())?;
    fs::write(&command.output, &artifact.payload).map_err(|error| {
        format!(
            "M0-BINARY-VERTICAL-SLICE-WRITE-FAILED {}: {error}",
            command.output.display()
        )
    })?;
    println!(
        "{{\"ok\":true,\"moduleResref\":\"{}\",\"areaResref\":\"{}\",\"fixtureTemplateResRef\":\"{}\",\"appearanceType\":{},\"modulePath\":\"{}\",\"sha256\":\"{}\"}}",
        artifact.report.module_resref,
        artifact.report.area_resref,
        artifact.report.creature_resref,
        artifact.report.appearance_row,
        command.output.display(),
        artifact.report.sha256,
    );
    Ok(())
}

struct Command {
    output: PathBuf,
    identity: BinaryM0VerticalSliceIdentityV1,
    appearance_row: u16,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut output = None;
    let mut module_resref = None;
    let mut area_resref = None;
    let mut hak_resref = None;
    let mut appearance_row = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        match argument.as_str() {
            "--out" => {
                if output.is_some() {
                    return Err("M0-BINARY-VERTICAL-SLICE-OUT-DUPLICATE".to_owned());
                }
                output = values
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .map(PathBuf::from);
                if output.is_none() {
                    return Err("M0-BINARY-VERTICAL-SLICE-OUT-MISSING".to_owned());
                }
            }
            "--module-resref" => module_resref = next_value(&mut values, "--module-resref")?,
            "--area-resref" => area_resref = next_value(&mut values, "--area-resref")?,
            "--hak-resref" => hak_resref = next_value(&mut values, "--hak-resref")?,
            "--appearance-row" => {
                if appearance_row.is_some() {
                    return Err("M0-BINARY-VERTICAL-SLICE-APPEARANCE-ROW-DUPLICATE".to_owned());
                }
                let value = next_value(&mut values, "--appearance-row")?
                    .ok_or_else(|| "M0-BINARY-VERTICAL-SLICE-APPEARANCE-ROW-MISSING".to_owned())?;
                appearance_row =
                    Some(value.parse::<u16>().map_err(|_| {
                        "M0-BINARY-VERTICAL-SLICE-APPEARANCE-ROW-INVALID".to_owned()
                    })?);
            }
            "--help" | "-h" => return Err(usage()),
            _ => {
                return Err(format!(
                    "M0-BINARY-VERTICAL-SLICE-ARGUMENT-UNKNOWN: {argument}\n{}",
                    usage()
                ));
            }
        }
    }
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: module_resref.unwrap_or_else(|| "m2a_bm0p1".to_owned()),
        area_resref: area_resref.unwrap_or_else(|| "m2a_bm0a1".to_owned()),
        hak_resref: hak_resref.unwrap_or_else(|| "m2a_m0_proof".to_owned()),
    };
    Ok(Command {
        output: output.ok_or_else(usage)?,
        identity,
        appearance_row: appearance_row.unwrap_or(DEFAULT_M0_APPEARANCE_ROW),
    })
}

fn next_value(
    values: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<Option<String>, String> {
    let value = values.next().filter(|value| !value.starts_with("--"));
    if value.is_none() {
        return Err(format!(
            "M0-BINARY-VERTICAL-SLICE-{}-MISSING",
            &flag[2..].to_uppercase()
        ));
    }
    Ok(value)
}

fn usage() -> String {
    "usage: materialize_m0_binary_vertical_slice --out <new-module-path> [--module-resref <resref> --area-resref <resref> --hak-resref <resref> --appearance-row <u16>]".to_owned()
}

#[cfg(test)]
mod tests {
    use super::parse_command;

    #[test]
    fn requires_exactly_one_output_destination() {
        assert!(parse_command(Vec::<String>::new()).is_err());
        assert!(parse_command(["--out".to_owned(), "a.mod".to_owned()]).is_ok());
        assert!(
            parse_command([
                "--out".to_owned(),
                "a.mod".to_owned(),
                "--out".to_owned(),
                "b.mod".to_owned()
            ])
            .is_err()
        );
    }

    #[test]
    fn accepts_a_fresh_module_area_and_hak_identity() {
        let command = parse_command([
            "--out".to_owned(),
            "m2a_bm0v10.mod".to_owned(),
            "--module-resref".to_owned(),
            "m2a_bm0v10".to_owned(),
            "--area-resref".to_owned(),
            "m2a_bm0a10".to_owned(),
            "--hak-resref".to_owned(),
            "m2a_m0v10".to_owned(),
            "--appearance-row".to_owned(),
            "15100".to_owned(),
        ])
        .unwrap();
        assert_eq!(command.identity.module_resref, "m2a_bm0v10");
        assert_eq!(command.identity.area_resref, "m2a_bm0a10");
        assert_eq!(command.identity.hak_resref, "m2a_m0v10");
        assert_eq!(command.appearance_row, 15_100);
    }

    #[test]
    fn rejects_an_invalid_appearance_row() {
        assert!(
            parse_command([
                "--out".to_owned(),
                "a.mod".to_owned(),
                "--appearance-row".to_owned(),
                "not-a-row".to_owned(),
            ])
            .is_err()
        );
    }
}
