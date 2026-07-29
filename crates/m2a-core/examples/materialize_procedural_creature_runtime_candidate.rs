use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    model_pipeline::{
        MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1, ProceduralCreaturePackageIdentityV1,
        build_meshy_procedural_humanoid_model_package_with_identity_v1,
        build_meshy_procedural_humanoid_p100k_experiment_with_identity_v1,
        write_procedural_creature_proof_packet_with_identity_v1,
    },
    proof_module::BinaryCreatureModuleIdentityV1,
};
use sha2::{Digest, Sha256};

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let command = parse_command(env::args().skip(1))?;
    if command.output.exists() {
        return Err(format!(
            "PROCEDURAL-CREATURE-CANDIDATE-DESTINATION-EXISTS: {}",
            command.output.display()
        ));
    }
    let source_glb = fs::read(&command.source_glb).map_err(|error| {
        format!(
            "PROCEDURAL-CREATURE-CANDIDATE-SOURCE-READ-FAILED {}: {error}",
            command.source_glb.display()
        )
    })?;
    let appearance_two_da = fs::read(&command.appearance_two_da).map_err(|error| {
        format!(
            "PROCEDURAL-CREATURE-CANDIDATE-APPEARANCE-READ-FAILED {}: {error}",
            command.appearance_two_da.display()
        )
    })?;
    let artifact = if command.p100k_experiment {
        build_meshy_procedural_humanoid_p100k_experiment_with_identity_v1(
            &source_glb,
            &appearance_two_da,
            &command.identity,
        )
    } else {
        build_meshy_procedural_humanoid_model_package_with_identity_v1(
            &source_glb,
            &appearance_two_da,
            &command.identity,
        )
    }
    .map_err(|error| error.to_string())?;
    write_procedural_creature_proof_packet_with_identity_v1(
        &command.output,
        &artifact,
        &command.identity,
    )
    .map_err(|error| error.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "status": "PROCEDURAL_CREATURE_RUNTIME_CANDIDATE_MATERIALIZED",
        "outputDirectory": command.output,
        "identity": command.identity,
        "sourceGlb": binding(&artifact.source_glb),
        "sourceAppearanceTwoDa": binding(&appearance_two_da),
        "model": binding(&artifact.model),
        "texture": binding(&artifact.texture),
        "runtimeAppearanceTwoDa": binding(&artifact.appearance_two_da),
        "hak": binding(&artifact.hak),
        "module": binding(&artifact.proof_module),
        "appearanceRow": artifact.report.appearance.appended_row_index,
        "animationProfile": artifact.report.animation_completeness,
        "animationBehavior": artifact.report.animation_behavior,
        "animationEvents": artifact.report.animation_event_conformance,
        "skinAnimationConformance": artifact.report.skin_animation_conformance,
        "triangleExperiment": command.p100k_experiment.then(|| serde_json::json!({
            "policy": "MESHY_P100K_SEGMENTED_EXPERIMENT_V1",
            "requestedTriangleCount": MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1,
            "sanitizedSourceTriangleCount": artifact.report.geometry.triangle_count,
            "writtenTriangleCount": artifact.report.model.projection.triangle_count,
            "binaryMdlMeshStreamCount": artifact.report.model.layout.mesh_nodes.len(),
            "productionTriangleBudgetChanged": false
        })),
        "runtimeContractVerified": true,
        "startsToolset": false,
        "startsNwn": false
    }))
    .map_err(|error| format!("PROCEDURAL-CREATURE-CANDIDATE-REPORT-FAILED: {error}"))
}

fn binding(bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "byteLength": bytes.len(),
        "sha256": Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    })
}

struct Command {
    source_glb: PathBuf,
    appearance_two_da: PathBuf,
    output: PathBuf,
    identity: ProceduralCreaturePackageIdentityV1,
    p100k_experiment: bool,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut appearance_two_da = None;
    let mut output = None;
    let mut model_resref = None;
    let mut texture_resref = None;
    let mut module_resref = None;
    let mut area_resref = None;
    let mut hak_resref = None;
    let mut creature_resref = None;
    let mut p100k_experiment = false;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        if argument == "--p100k-experiment" {
            if p100k_experiment {
                return Err(format!(
                    "PROCEDURAL-CREATURE-CANDIDATE-ARGUMENT-DUPLICATE: {argument}"
                ));
            }
            p100k_experiment = true;
            continue;
        }
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--appearance-2da" => &mut appearance_two_da,
            "--out" => &mut output,
            "--model-resref" => &mut model_resref,
            "--texture-resref" => &mut texture_resref,
            "--module-resref" => &mut module_resref,
            "--area-resref" => &mut area_resref,
            "--hak-resref" => &mut hak_resref,
            "--creature-resref" => &mut creature_resref,
            "--help" | "-h" => return Err(usage()),
            _ => {
                return Err(format!(
                    "PROCEDURAL-CREATURE-CANDIDATE-ARGUMENT-UNKNOWN: {argument}\n{}",
                    usage()
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "PROCEDURAL-CREATURE-CANDIDATE-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!(
                "PROCEDURAL-CREATURE-CANDIDATE-ARGUMENT-MISSING: {argument}"
            ));
        }
    }

    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
        identity: ProceduralCreaturePackageIdentityV1 {
            model_resref: model_resref.ok_or_else(usage)?,
            texture_resref: texture_resref.ok_or_else(usage)?,
            module: BinaryCreatureModuleIdentityV1 {
                module_resref: module_resref.ok_or_else(usage)?,
                area_resref: area_resref.ok_or_else(usage)?,
                hak_resref: hak_resref.ok_or_else(usage)?,
            },
            creature_resref: creature_resref.ok_or_else(usage)?,
        },
        p100k_experiment,
    })
}

fn usage() -> String {
    "usage: materialize_procedural_creature_runtime_candidate [--p100k-experiment] --source-glb <path> --appearance-2da <full-table-path> --out <new-output-dir> --model-resref <fresh-resref> --texture-resref <fresh-resref> --module-resref <fresh-resref> --area-resref <fresh-resref> --hak-resref <fresh-resref> --creature-resref <fresh-resref>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::parse_command;

    fn arguments() -> Vec<String> {
        [
            "--source-glb",
            "source.glb",
            "--appearance-2da",
            "appearance.2da",
            "--out",
            "candidate",
            "--model-resref",
            "m2a_testmdl",
            "--texture-resref",
            "m2a_testtex",
            "--module-resref",
            "m2a_testmod",
            "--area-resref",
            "m2a_testarea",
            "--hak-resref",
            "m2a_testhak",
            "--creature-resref",
            "m2a_testutc",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    #[test]
    fn requires_every_input_and_fresh_identity_field_once() {
        assert!(parse_command(arguments()).is_ok());
        for missing_index in (0..arguments().len()).step_by(2) {
            let mut missing = arguments();
            missing.drain(missing_index..missing_index + 2);
            assert!(parse_command(missing).is_err());
        }
        let mut duplicate = arguments();
        duplicate.extend(["--out".to_owned(), "other".to_owned()]);
        assert!(parse_command(duplicate).is_err());
    }
}
