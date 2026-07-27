//! Materializes the isolated NeverBlender/CleanModels comparison package.
//!
//! This is a proof harness, not a converter: its inputs are an already-native
//! reference MDL, an owned texture and the current generated proof package.
//! It exercises the project's own 2DA, HAK and MOD writers to bind the new
//! `incaxje` model to a distinct appearance row.

use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    erf::ErfArchive,
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    mdl::inspect_binary_mdl,
    proof_module::build_creature_proof_module_v1,
    two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        append_two_da_row_v1, inspect_two_da_v2,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};

const MODEL_RESREF: &str = "incaxje";
const TEXTURE_RESREF: &str = "incaxjet";
const APPEARANCE_LABEL: &str = "INCAXJE_NEVERBLENDER";
const HAK_FILE_NAME: &str = "m2a_codex_aproof.hak";
const MODULE_FILE_NAME: &str = "m2a_codex_aproof.mod";

#[derive(Debug)]
struct Arguments {
    base_appearance: PathBuf,
    original_model: PathBuf,
    original_texture: PathBuf,
    reference_model: PathBuf,
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
    let args = parse_arguments(env::args().skip(1))?;
    let base_appearance = read(&args.base_appearance)?;
    let original_model = read(&args.original_model)?;
    let original_texture = read(&args.original_texture)?;
    let reference_model = read(&args.reference_model)?;
    let reference_model_provenance = reference_model_provenance(&reference_model);
    let reference_reader_status = match inspect_binary_mdl(&reference_model) {
        Ok(reference_readback) => {
            if reference_readback.model.name != MODEL_RESREF {
                return Err(format!(
                    "INCAXJE-REFERENCE-MODEL-RESREF-MISMATCH: expected {MODEL_RESREF}, got {}",
                    reference_readback.model.name
                ));
            }
            json!({
                "status": "PASS",
                "format": reference_readback.format,
                "modelName": reference_readback.model.name,
                "nodeCount": reference_readback.node_tree.node_count,
                "animationCount": reference_readback.animations.len(),
                "unsupported": reference_readback.unsupported,
                "diagnosticCount": reference_readback.diagnostics.len(),
            })
        }
        Err(error) => json!({
            "status": "UNSUPPORTED_BY_OWN_READER",
            "error": error.to_string(),
        }),
    };

    // The input is the existing proof table (whose last row is the owned M6
    // model).  This makes Incaxje a genuine second, distinct appearance.
    let inspection = inspect_two_da_v2(&base_appearance, &TwoDaLimitsV1::default())
        .map_err(|error| error.to_string())?;
    for column in [
        "LABEL",
        "MOVERATE",
        "MODELTYPE",
        "RACE",
        "PORTRAIT",
        "ENVMAP",
        "BLOODCOLR",
        "WEAPONSCALE",
        "SIZECATEGORY",
    ] {
        if !inspection
            .columns
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(column))
        {
            return Err(format!("INCAXJE-APPEARANCE-COLUMN-MISSING: {column}"));
        }
    }

    let mut cells = vec![
        text("LABEL", APPEARANCE_LABEL),
        text("MOVERATE", "NORM"),
        text("MODELTYPE", "S"),
        text("RACE", MODEL_RESREF),
        null("PORTRAIT"),
        null("ENVMAP"),
        text("BLOODCOLR", "R"),
        text("WEAPONSCALE", "1.0"),
        text("SIZECATEGORY", "4"),
    ];
    if let Some(column) = inspection.columns.iter().find(|candidate| {
        [
            "DefaultPhenoType",
            "DefaultPhenotype",
            "DefaultPhenotypeID",
            "DefaultPheno",
        ]
        .iter()
        .any(|alias| candidate.eq_ignore_ascii_case(alias))
    }) {
        cells.push(text(column, "0"));
    }
    let appearance = append_two_da_row_v1(
        &base_appearance,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| error.to_string())?;

    let hak = write_hak_v1(
        &[
            resource("m2a_m6p01", 2002, original_model),
            resource("m2a_m6t01", 3, original_texture.clone()),
            resource(MODEL_RESREF, 2002, reference_model),
            resource(TEXTURE_RESREF, 3, original_texture),
            resource("appearance", 2017, appearance.payload.clone()),
        ],
        &HakWriterOptionsV1::default(),
    )
    .map_err(|error| error.to_string())?;
    let module = build_creature_proof_module_v1(appearance.report.appended_row_index)
        .map_err(|error| error.to_string())?;

    let archive = ErfArchive::parse(&hak.payload).map_err(|error| error.to_string())?;
    for (resref, resource_type) in [
        ("m2a_m6p01", 2002),
        ("m2a_m6t01", 3),
        (MODEL_RESREF, 2002),
        (TEXTURE_RESREF, 3),
        ("appearance", 2017),
    ] {
        archive
            .find(resref, resource_type)
            .map_err(|error| error.to_string())?;
    }
    let output_appearance = archive
        .find("appearance", 2017)
        .map_err(|error| error.to_string())?;
    if output_appearance != appearance.payload.as_slice() {
        return Err("INCAXJE-HAK-APPEARANCE-READBACK-MISMATCH".to_owned());
    }

    fs::create_dir_all(&args.output_dir).map_err(|error| error.to_string())?;
    fs::write(args.output_dir.join(HAK_FILE_NAME), &hak.payload)
        .map_err(|error| error.to_string())?;
    fs::write(args.output_dir.join(MODULE_FILE_NAME), &module.payload)
        .map_err(|error| error.to_string())?;
    fs::write(args.output_dir.join("appearance.2da"), &appearance.payload)
        .map_err(|error| error.to_string())?;
    fs::write(
        args.output_dir.join("incaxje-proof-manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "provenance": format!("reference-only {reference_model_provenance}; package/module/2DA generated by Meshy2Aurora"),
            "appearance": {
                "label": APPEARANCE_LABEL,
                "physicalRow": appearance.report.appended_row_index,
                "modelType": "S",
                "race": MODEL_RESREF,
            },
            "resources": {
                "model": {"resref": MODEL_RESREF, "type": 2002, "sha256": sha256(archive.find(MODEL_RESREF, 2002).unwrap())},
                "texture": {"resref": TEXTURE_RESREF, "type": 3, "sha256": sha256(archive.find(TEXTURE_RESREF, 3).unwrap())},
                "appearance": {"resref": "appearance", "type": 2017, "sha256": sha256(output_appearance)},
            },
            "referenceModelReadback": reference_reader_status,
            "outputs": {
                "hak": {"file": HAK_FILE_NAME, "sha256": sha256(&hak.payload)},
                "module": {"file": MODULE_FILE_NAME, "sha256": sha256(&module.payload)},
            },
            "hakReadback": "PASS",
            "moduleReadback": module.report.semantic_readback_status,
        }))
        .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "INCAXJE_REFERENCE_PROOF={{\"appearanceRow\":{},\"model\":\"{}\",\"hakSha256\":\"{}\",\"moduleSha256\":\"{}\"}}",
        appearance.report.appended_row_index,
        MODEL_RESREF,
        sha256(&hak.payload),
        sha256(&module.payload),
    );
    Ok(())
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut base_appearance = None;
    let mut original_model = None;
    let mut original_texture = None;
    let mut reference_model = None;
    let mut output_dir = None;
    let mut items = arguments.into_iter();
    while let Some(argument) = items.next() {
        match argument.as_str() {
            "--base-appearance" => base_appearance = Some(required_value(&mut items, &argument)?),
            "--original-model" => original_model = Some(required_value(&mut items, &argument)?),
            "--original-texture" => original_texture = Some(required_value(&mut items, &argument)?),
            "--reference-model" => reference_model = Some(required_value(&mut items, &argument)?),
            "--output-dir" => output_dir = Some(required_value(&mut items, &argument)?),
            "--help" | "-h" => return Err(usage()),
            _ => return Err(format!("INCAXJE-ARGUMENT-UNKNOWN: {argument}\n{}", usage())),
        }
    }
    Ok(Arguments {
        base_appearance: base_appearance.ok_or_else(usage)?,
        original_model: original_model.ok_or_else(usage)?,
        original_texture: original_texture.ok_or_else(usage)?,
        reference_model: reference_model.ok_or_else(usage)?,
        output_dir: output_dir.ok_or_else(usage)?,
    })
}

fn required_value(
    iterator: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, String> {
    iterator
        .next()
        .filter(|candidate| !candidate.starts_with("--"))
        .map(PathBuf::from)
        .ok_or_else(|| format!("INCAXJE-ARGUMENT-MISSING: {flag}"))
}

fn usage() -> String {
    "usage: materialize_incaxje_reference_proof --base-appearance <path> --original-model <path> --original-texture <path> --reference-model <path> --output-dir <path>".to_owned()
}

fn read(path: &PathBuf) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("INCAXJE-INPUT-READ-FAILED {}: {error}", path.display()))
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

fn text(column_name: impl Into<String>, value: impl Into<String>) -> TwoDaCellAssignmentV1 {
    TwoDaCellAssignmentV1 {
        column_name: column_name.into(),
        value: TwoDaCellValueV1::Text {
            value: value.into(),
        },
    }
}

fn null(column_name: impl Into<String>) -> TwoDaCellAssignmentV1 {
    TwoDaCellAssignmentV1 {
        column_name: column_name.into(),
        value: TwoDaCellValueV1::Null,
    }
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn reference_model_provenance(bytes: &[u8]) -> &'static str {
    let ascii_prefix = bytes
        .iter()
        .copied()
        .take(512)
        .skip_while(u8::is_ascii_whitespace)
        .collect::<Vec<_>>();
    if ascii_prefix.starts_with(b"newmodel") || ascii_prefix.starts_with(b"# NeverBlender") {
        "NeverBlender ASCII model (on-load parser A/B)"
    } else {
        "NeverBlender ASCII -> CleanModels native MDL"
    }
}

#[cfg(test)]
mod tests {
    use super::reference_model_provenance;

    #[test]
    fn labels_neverblender_ascii_as_on_load_parser_test() {
        assert_eq!(
            reference_model_provenance(b"\n# NeverBlender 4.1.0\nnewmodel incaxje\n"),
            "NeverBlender ASCII model (on-load parser A/B)"
        );
    }

    #[test]
    fn labels_non_ascii_payload_as_cleanmodels_native() {
        assert_eq!(
            reference_model_provenance(&[0, 0, 0, 0, 12, 0, 0, 0]),
            "NeverBlender ASCII -> CleanModels native MDL"
        );
    }
}
