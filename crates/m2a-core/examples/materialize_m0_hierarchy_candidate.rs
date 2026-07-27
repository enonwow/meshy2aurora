use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    direct_creature_contract::SourceTopologyOriginV1,
    hierarchy_candidate::{
        build_meshy_hierarchy_package_v4, declare_meshy_hierarchy_package_profile_v4,
        verify_meshy_hierarchy_package_v4, write_meshy_hierarchy_package_v4,
    },
    hierarchy_experiment::{
        declare_meshy_hierarchy_candidate_profile_v4, derive_meshy_m0_hierarchy_source_v4,
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
            "M2A-HIERARCHY-PACKAGE-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }
    let original = read(&command.original_source_glb, "ORIGINAL-SOURCE")?;
    let r30_model = read(&command.r30_model, "R30-MODEL")?;
    let appearance = read(&command.appearance_two_da, "APPEARANCE")?;
    let original_path = command
        .original_source_glb
        .canonicalize()
        .unwrap_or_else(|_| command.original_source_glb.clone())
        .display()
        .to_string();
    let derived = derive_meshy_m0_hierarchy_source_v4(
        &original,
        original_path,
        "generated/derived-source.glb",
    )
    .map_err(|error| error.to_string())?;
    let model_profile = declare_meshy_hierarchy_candidate_profile_v4(
        &original,
        &derived,
        &r30_model,
        SourceTopologyOriginV1::UserDerived,
        "m2a_m0p01",
        "m2a_m0t01",
    )
    .map_err(|error| error.to_string())?;
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_m0r31".to_owned(),
        area_resref: "m2a_m0a31".to_owned(),
        hak_resref: "m2a_m0r31".to_owned(),
    };
    let package_profile =
        declare_meshy_hierarchy_package_profile_v4(&model_profile, &identity, &appearance)
            .map_err(|error| error.to_string())?;
    let artifact = build_meshy_hierarchy_package_v4(
        &original,
        &derived.bytes,
        &r30_model,
        &appearance,
        &package_profile,
    )
    .map_err(|error| error.to_string())?;
    verify_meshy_hierarchy_package_v4(
        &artifact.contract,
        &package_profile,
        &original,
        &derived.bytes,
        &r30_model,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .map_err(|error| error.to_string())?;
    write_meshy_hierarchy_package_v4(&command.output, &artifact, &r30_model, &appearance)
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "status": "R31_HIERARCHY_CANDIDATE_MATERIALIZED_GATE_B_PENDING",
            "outputDirectory": command.output,
            "materializationCount": artifact.contract.materialization_count,
            "module": artifact.contract.module,
            "areaResref": artifact.contract.binary_scene.area_resref,
            "orderedHakList": [artifact.contract.hak.clone()],
            "fixture": artifact.contract.binary_scene.fixture,
            "originalSource": artifact.contract.original_source,
            "derivedSource": artifact.contract.derived_source,
            "derivedSourceBinding": artifact.contract.model_contract.derived_source_binding,
            "model": artifact.contract.model,
            "texture": artifact.contract.texture,
            "appearanceTwoDa": artifact.contract.appearance_two_da,
            "appearanceTable": artifact.contract.appearance_table,
            "stateProjectionSha256": artifact.contract.model_contract.candidate_state_projection_sha256,
            "engineEnvelopeSha256": artifact.contract.model_contract.candidate_engine_envelope_sha256,
            "rawMdxSha256": artifact.contract.model_contract.candidate_raw_mdx_sha256,
            "protectedWriterFieldsSha256": artifact.contract.model_contract.candidate_protected_writer_fields_sha256,
            "runtimeProfileMaterialized": false,
            "startsToolset": false,
            "startsNwn": false,
            "noLocalToolsetAdapter": true,
        }))
        .map_err(|error| format!("M2A-HIERARCHY-PACKAGE-REPORT: {error}"))?
    );
    Ok(())
}

struct Command {
    original_source_glb: PathBuf,
    r30_model: PathBuf,
    appearance_two_da: PathBuf,
    output: PathBuf,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut original_source_glb = None;
    let mut r30_model = None;
    let mut appearance_two_da = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--original-source-glb" => &mut original_source_glb,
            "--r30-model" => &mut r30_model,
            "--appearance-2da" => &mut appearance_two_da,
            "--out" => &mut output,
            "--help" | "-h" => return Err(usage()),
            _ => {
                return Err(format!(
                    "M2A-HIERARCHY-PACKAGE-ARGUMENT: {argument}\n{}",
                    usage()
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "M2A-HIERARCHY-PACKAGE-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!(
                "M2A-HIERARCHY-PACKAGE-ARGUMENT-MISSING: {argument}"
            ));
        }
    }
    Ok(Command {
        original_source_glb: PathBuf::from(original_source_glb.ok_or_else(usage)?),
        r30_model: PathBuf::from(r30_model.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn read(path: &PathBuf, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| {
        format!(
            "M2A-HIERARCHY-PACKAGE-{label}-READ {}: {error}",
            path.display()
        )
    })
}

fn usage() -> String {
    "usage: materialize_m0_hierarchy_candidate --original-source-glb <exact-frozen-M0.glb> --r30-model <exact-frozen-r30.mdl> --appearance-2da <exact-full-table> --out <absent-output-dir>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::parse_command;

    #[test]
    fn requires_every_trust_root_and_has_no_caller_selected_lineage_identity() {
        let arguments = [
            "--original-source-glb",
            "source.glb",
            "--r30-model",
            "r30.mdl",
            "--appearance-2da",
            "appearance.2da",
            "--out",
            "candidate",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        assert!(parse_command(arguments.clone()).is_ok());
        for missing in (0..arguments.len()).step_by(2) {
            let mut changed = arguments.clone();
            changed.drain(missing..missing + 2);
            assert!(parse_command(changed).is_err());
        }
        let mut forbidden_identity_override = arguments;
        forbidden_identity_override.extend(["--module-resref".to_owned(), "other".to_owned()]);
        assert!(parse_command(forbidden_identity_override).is_err());
    }
}
