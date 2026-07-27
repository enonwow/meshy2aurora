use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    model_pipeline::{
        DirectCreatureAnimationProfileV1, build_m6_model_package_v1,
        build_meshy_h1_model_package_v1, build_meshy_h1_model_package_v2,
        build_meshy_h1_rigid_runtime_diagnostic_package_v1, build_meshy_m0_static_rigid_package_v1,
        write_m0_proof_packet_v1, write_m6_proof_packet_v1,
    },
    owned_fixture::{synthetic_owned_m6_full_native_42_glb_v1, synthetic_owned_m6_glb_v1},
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
    SyntheticOwnedH1Full42,
    MeshyH1(PathBuf),
    MeshyH1RigidRuntimeDiagnostic(PathBuf),
    MeshyM0StaticIsolated(PathBuf),
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
    let (artifact_result, m0_packet_kind) = match arguments.source {
        SourceInput::SyntheticOwned => {
            let source_glb = synthetic_owned_m6_glb_v1().map_err(|error| error.to_string())?;
            (build_m6_model_package_v1(&source_glb, &appearance), None)
        }
        SourceInput::SyntheticOwnedH1 => {
            let source_glb = synthetic_owned_m6_glb_v1().map_err(|error| error.to_string())?;
            (
                build_meshy_h1_model_package_v1(&source_glb, &appearance),
                None,
            )
        }
        SourceInput::SyntheticOwnedH1Full42 => {
            let source_glb =
                synthetic_owned_m6_full_native_42_glb_v1().map_err(|error| error.to_string())?;
            (
                build_meshy_h1_model_package_v2(
                    &source_glb,
                    &appearance,
                    DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
                ),
                None,
            )
        }
        SourceInput::MeshyH1(path) => {
            let source_glb =
                fs::read(&path).map_err(|error| format!("M6-INPUT-READ-FAILED: {error}"))?;
            (
                build_meshy_h1_model_package_v1(&source_glb, &appearance),
                None,
            )
        }
        SourceInput::MeshyH1RigidRuntimeDiagnostic(path) => {
            let source_glb =
                fs::read(&path).map_err(|error| format!("M6-INPUT-READ-FAILED: {error}"))?;
            (
                build_meshy_h1_rigid_runtime_diagnostic_package_v1(&source_glb, &appearance),
                None,
            )
        }
        SourceInput::MeshyM0StaticIsolated(path) => {
            let source_glb =
                fs::read(&path).map_err(|error| format!("M6-INPUT-READ-FAILED: {error}"))?;
            (
                build_meshy_m0_static_rigid_package_v1(&source_glb, &appearance),
                Some(()),
            )
        }
    };
    let artifact = artifact_result
        .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    let write_result = match m0_packet_kind {
        Some(()) => write_m0_proof_packet_v1(&arguments.output_dir, &artifact),
        None => write_m6_proof_packet_v1(&arguments.output_dir, &artifact),
    };
    write_result
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
            "--synthetic-owned-h1-full-42" => {
                set_source(&mut source, SourceInput::SyntheticOwnedH1Full42)?
            }
            "--meshy-h1-source" => {
                let path = required_value(&mut iterator, "--meshy-h1-source")?;
                set_source(&mut source, SourceInput::MeshyH1(path.into()))?;
            }
            "--meshy-h1-rigid-runtime-diagnostic-source" => {
                let path =
                    required_value(&mut iterator, "--meshy-h1-rigid-runtime-diagnostic-source")?;
                set_source(
                    &mut source,
                    SourceInput::MeshyH1RigidRuntimeDiagnostic(path.into()),
                )?;
            }
            "--meshy-m0-static-source" => {
                return Err(
                    "M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN: use the explicit V2 materializer with a caller-owned runtime profile"
                        .to_owned(),
                );
            }
            "--meshy-m0-static-isolated-source" => {
                let path = required_value(&mut iterator, "--meshy-m0-static-isolated-source")?;
                set_source(&mut source, SourceInput::MeshyM0StaticIsolated(path.into()))?;
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
    "usage: materialize_m6 (--synthetic-owned | --synthetic-owned-h1 | --synthetic-owned-h1-full-42 | --meshy-h1-source <path> | --meshy-h1-rigid-runtime-diagnostic-source <path> | --meshy-m0-static-isolated-source <path>) (--appearance-hak <path> | --appearance-2da <path>) --output-dir <path>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::parse_arguments;

    #[test]
    fn legacy_m0_canonical_cli_route_is_rejected_before_any_emission() {
        let error = parse_arguments(
            [
                "--meshy-m0-static-source",
                "source.glb",
                "--appearance-2da",
                "appearance.2da",
                "--output-dir",
                "must-not-exist",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect_err("legacy canonical M0 CLI mode must be executable-dead");
        assert!(error.starts_with("M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN"));
    }
}
