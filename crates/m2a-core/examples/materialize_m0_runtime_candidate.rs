use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    direct_creature_contract::{
        SourceTopologyOriginV1, declare_m0_direct_creature_runtime_profile_v2,
    },
    model_pipeline::{
        build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2,
        verify_m0_binary_runtime_fixture_contract_v2,
        write_m0_canonical_runtime_proof_packet_with_profile_v2,
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
    if command.output.exists() {
        return Err(format!(
            "M0-RUNTIME-CANDIDATE-DESTINATION-EXISTS: {}",
            command.output.display()
        ));
    }
    let source_glb = fs::read(&command.source_glb).map_err(|error| {
        format!(
            "M0-RUNTIME-CANDIDATE-SOURCE-READ-FAILED {}: {error}",
            command.source_glb.display()
        )
    })?;
    let appearance_two_da = fs::read(&command.appearance_two_da).map_err(|error| {
        format!(
            "M0-RUNTIME-CANDIDATE-APPEARANCE-READ-FAILED {}: {error}",
            command.appearance_two_da.display()
        )
    })?;
    let informational_source_path = command
        .source_glb
        .canonicalize()
        .unwrap_or_else(|_| command.source_glb.clone())
        .display()
        .to_string();
    let runtime_profile = declare_m0_direct_creature_runtime_profile_v2(
        &source_glb,
        informational_source_path,
        SourceTopologyOriginV1::UserDeclared,
        "m2a_m0p01",
    )
    .map_err(|error| error.to_string())?;
    let artifact = build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2(
        &source_glb,
        &appearance_two_da,
        &command.identity,
        &runtime_profile,
    )
    .map_err(|error| error.to_string())?;
    let contract = artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .ok_or_else(|| "M0-RUNTIME-CANDIDATE-CONTRACT-MISSING".to_owned())?;
    verify_m0_binary_runtime_fixture_contract_v2(
        contract,
        &runtime_profile,
        &source_glb,
        &artifact.proof_module,
        &artifact.hak,
    )
    .map_err(|error| error.to_string())?;
    write_m0_canonical_runtime_proof_packet_with_profile_v2(
        &command.output,
        &artifact,
        &command.identity,
        &runtime_profile,
        &source_glb,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "status": "M0_RUNTIME_CANDIDATE_MATERIALIZED",
            "outputDirectory": command.output,
            "sourceGlb": artifact.summary.input_glb,
            "sourceAppearanceTwoDa": artifact.summary.input_appearance_two_da,
            "module": contract.module,
            "orderedHakList": [contract.hak.clone()],
            "areaResref": contract.binary_scene.area_resref,
            "appearanceRow": contract.appearance.physical_row,
            "appearanceTable": contract.appearance_table,
            "model": contract.model,
            "texture": contract.texture,
            "stateProjectionProfile": contract.state_projection_profile,
            "stateProjectionProvenance": contract.state_projection_provenance,
            "stateProjectionSummary": contract.state_projection_summary,
            "stateProjectionSummarySha256": contract.state_projection_summary_sha256,
            "sourceTopologyBinding": contract.source_topology_binding,
            "engineEnvelope": contract.engine_envelope,
            "engineEnvelopeSha256": contract.engine_envelope_sha256,
            "runtimeContractVerified": true,
            "startsToolset": false,
            "startsNwn": false,
        }))
        .map_err(|error| format!("M0-RUNTIME-CANDIDATE-REPORT-FAILED: {error}"))?
    );
    Ok(())
}

struct Command {
    source_glb: PathBuf,
    appearance_two_da: PathBuf,
    output: PathBuf,
    identity: BinaryM0VerticalSliceIdentityV1,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut appearance_two_da = None;
    let mut output = None;
    let mut module_resref = None;
    let mut area_resref = None;
    let mut hak_resref = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--appearance-2da" => &mut appearance_two_da,
            "--out" => &mut output,
            "--module-resref" => &mut module_resref,
            "--area-resref" => &mut area_resref,
            "--hak-resref" => &mut hak_resref,
            "--help" | "-h" => return Err(usage()),
            _ => {
                return Err(format!(
                    "M0-RUNTIME-CANDIDATE-ARGUMENT-UNKNOWN: {argument}\n{}",
                    usage()
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "M0-RUNTIME-CANDIDATE-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("M0-RUNTIME-CANDIDATE-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
        identity: BinaryM0VerticalSliceIdentityV1 {
            module_resref: module_resref.ok_or_else(usage)?,
            area_resref: area_resref.ok_or_else(usage)?,
            hak_resref: hak_resref.ok_or_else(usage)?,
        },
    })
}

fn usage() -> String {
    "usage: materialize_m0_runtime_candidate --source-glb <path> --appearance-2da <full-table-path> --out <new-output-dir> --module-resref <fresh-resref> --area-resref <fresh-resref> --hak-resref <fresh-resref>".to_owned()
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
            "--module-resref",
            "m2a_testmod1",
            "--area-resref",
            "m2a_testarea1",
            "--hak-resref",
            "m2a_testhak1",
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
