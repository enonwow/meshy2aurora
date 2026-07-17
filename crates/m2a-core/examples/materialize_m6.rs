use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    model_pipeline::{
        build_m6_model_package_v1, build_meshy_h1_model_package_v1, write_m6_proof_packet_v1,
    },
    owned_fixture::synthetic_owned_m6_glb_v1,
};

#[derive(Debug)]
enum AppearanceInput {
    Hak(PathBuf),
    TwoDa(PathBuf),
}

#[derive(Debug)]
enum SourceInput {
    SyntheticOwned,
    SyntheticOwnedH1,
    MeshyH1(PathBuf),
}

#[derive(Debug)]
struct Arguments {
    source: SourceInput,
    appearance: AppearanceInput,
    output_dir: PathBuf,
}

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
    let arguments = parse_arguments(env::args().skip(1))?;
    let appearance = match arguments.appearance {
        AppearanceInput::Hak(path) => {
            let bytes =
                fs::read(&path).map_err(|error| format!("M6-INPUT-READ-FAILED: {error}"))?;
            let archive = ErfArchive::parse(&bytes).map_err(|error| error.to_string())?;
            if archive.file_type() != ErfFileType::Hak {
                return Err("M6-INPUT-NOT-HAK: --appearance-hak requires HAK V1.0".to_owned());
            }
            archive
                .find("appearance", 2017)
                .map_err(|error| error.to_string())?
                .to_vec()
        }
        AppearanceInput::TwoDa(path) => {
            fs::read(&path).map_err(|error| format!("M6-INPUT-READ-FAILED: {error}"))?
        }
    };
    let artifact = match arguments.source {
        SourceInput::SyntheticOwned => {
            let source_glb = synthetic_owned_m6_glb_v1().map_err(|error| error.to_string())?;
            build_m6_model_package_v1(&source_glb, &appearance)
        }
        SourceInput::SyntheticOwnedH1 => {
            let source_glb = synthetic_owned_m6_glb_v1().map_err(|error| error.to_string())?;
            build_meshy_h1_model_package_v1(&source_glb, &appearance)
        }
        SourceInput::MeshyH1(path) => {
            let source_glb =
                fs::read(&path).map_err(|error| format!("M6-INPUT-READ-FAILED: {error}"))?;
            build_meshy_h1_model_package_v1(&source_glb, &appearance)
        }
    }
    .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    write_m6_proof_packet_v1(&arguments.output_dir, &artifact)
        .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    println!("{}", String::from_utf8_lossy(&artifact.summary_json).trim());
    Ok(())
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut source = None;
    let mut appearance = None;
    let mut output_dir = None;
    let mut iterator = arguments.into_iter();
    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "--synthetic-owned" => set_source(&mut source, SourceInput::SyntheticOwned)?,
            "--synthetic-owned-h1" => set_source(&mut source, SourceInput::SyntheticOwnedH1)?,
            "--meshy-h1-source" => {
                let path = required_value(&mut iterator, "--meshy-h1-source")?;
                set_source(&mut source, SourceInput::MeshyH1(path.into()))?;
            }
            "--appearance-hak" => {
                let path = required_value(&mut iterator, "--appearance-hak")?;
                set_appearance(&mut appearance, AppearanceInput::Hak(path.into()))?;
            }
            "--appearance-2da" => {
                let path = required_value(&mut iterator, "--appearance-2da")?;
                set_appearance(&mut appearance, AppearanceInput::TwoDa(path.into()))?;
            }
            "--output-dir" => {
                if output_dir.is_some() {
                    return Err("M6-CLI-ARGUMENT-DUPLICATE: --output-dir".to_owned());
                }
                output_dir = Some(PathBuf::from(required_value(
                    &mut iterator,
                    "--output-dir",
                )?));
            }
            "--help" | "-h" => return Err(usage()),
            _ => return Err(format!("M6-CLI-ARGUMENT-UNKNOWN: {argument}\n{}", usage())),
        }
    }
    Ok(Arguments {
        source: source.ok_or_else(|| format!("M6-CLI-SOURCE-MISSING\n{}", usage()))?,
        appearance: appearance.ok_or_else(|| format!("M6-CLI-APPEARANCE-MISSING\n{}", usage()))?,
        output_dir: output_dir.ok_or_else(|| format!("M6-CLI-OUTPUT-MISSING\n{}", usage()))?,
    })
}

fn set_source(slot: &mut Option<SourceInput>, value: SourceInput) -> Result<(), String> {
    if slot.is_some() {
        return Err("M6-CLI-SOURCE-CONFLICT: select exactly one source route".to_owned());
    }
    *slot = Some(value);
    Ok(())
}

fn required_value(
    iterator: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, String> {
    iterator
        .next()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("M6-CLI-VALUE-MISSING: {flag}"))
}

fn set_appearance(
    slot: &mut Option<AppearanceInput>,
    value: AppearanceInput,
) -> Result<(), String> {
    if slot.is_some() {
        return Err("M6-CLI-APPEARANCE-CONFLICT: select exactly one of --appearance-hak or --appearance-2da".to_owned());
    }
    *slot = Some(value);
    Ok(())
}

fn usage() -> String {
    "usage: materialize_m6 (--synthetic-owned | --synthetic-owned-h1 | --meshy-h1-source <path>) (--appearance-hak <path> | --appearance-2da <path>) --output-dir <path>".to_owned()
}
