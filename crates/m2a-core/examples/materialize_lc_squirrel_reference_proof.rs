//! Materializes the isolated Last City `c_squirrel` reference control.
//!
//! This is deliberately not a Meshy2Aurora model-writer proof.  It carries an
//! unmodified external ASCII model plus all of its discovered texture
//! dependencies through the project's own 2DA, HAK and MOD writers.  Its only
//! purpose is to test the HAK -> `appearance.2da` -> Aurora/NWN path.

use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    erf::ErfArchive,
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    proof_module::build_creature_proof_module_v1,
    two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        append_two_da_row_v1, inspect_two_da_v2,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};

const MODEL_RESREF: &str = "c_squirrel";
const APPEARANCE_LABEL: &str = "M2A_LC_SQUIRREL_REF";
const HAK_FILE_NAME: &str = "m2a_codex_aproof.hak";
const MODULE_FILE_NAME: &str = "m2a_codex_aproof.mod";

const SQUIRREL_MODEL_SHA256: &str =
    "e381bd13f4f10ab9c33a9751f409c061b6579e2a042d12170d74e23f5b669767";
const SQUIRREL_DDS_SHA256: &str =
    "c26e36494debc0d8eb3491d01eadfd28ef24cb6b36667caf28e1afe149d4f8c1";
const BADGER_TGA_SHA256: &str = "993467cf6632d918a355074df4b9170ed57aff6d0a74fdb713eae33839a2537b";
const BADGER_DDS_SHA256: &str = "d6e1b01dc6a797e8e3683461bc243a2b6bbf509508f543bcc4c5ff62ab7ed2eb";

#[derive(Debug)]
struct Arguments {
    base_appearance: PathBuf,
    squirrel_model: PathBuf,
    squirrel_dds: PathBuf,
    badger_tga: PathBuf,
    badger_dds: PathBuf,
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
    let squirrel_model = read_and_verify(
        &args.squirrel_model,
        "LC-SQUIRREL-MODEL",
        SQUIRREL_MODEL_SHA256,
    )?;
    let squirrel_dds = read_and_verify(&args.squirrel_dds, "LC-SQUIRREL-DDS", SQUIRREL_DDS_SHA256)?;
    let badger_tga = read_and_verify(&args.badger_tga, "LC-BADGER-TGA", BADGER_TGA_SHA256)?;
    let badger_dds = read_and_verify(&args.badger_dds, "LC-BADGER-DDS", BADGER_DDS_SHA256)?;

    // The whole Last City row is deliberately replicated, except for LABEL.
    // That keeps the control's creature semantics (including size, movement
    // and head settings) independent of the model under test.
    let appearance_inspection = inspect_two_da_v2(&base_appearance, &TwoDaLimitsV1::default())
        .map_err(|error| error.to_string())?;
    let cells = squirrel_appearance_cells(&appearance_inspection.columns)?;
    let appearance = append_two_da_row_v1(
        &base_appearance,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| error.to_string())?;

    let resources = [
        resource(MODEL_RESREF, 2002, squirrel_model.clone()),
        resource(MODEL_RESREF, 2033, squirrel_dds.clone()),
        resource("c_badger", 3, badger_tga.clone()),
        resource("c_badger", 2033, badger_dds.clone()),
        resource("appearance", 2017, appearance.payload.clone()),
    ];
    let hak = write_hak_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| error.to_string())?;
    let module = build_creature_proof_module_v1(appearance.report.appended_row_index)
        .map_err(|error| error.to_string())?;

    let archive = ErfArchive::parse(&hak.payload).map_err(|error| error.to_string())?;
    for (resref, resource_type, expected) in [
        (MODEL_RESREF, 2002, squirrel_model.as_slice()),
        (MODEL_RESREF, 2033, squirrel_dds.as_slice()),
        ("c_badger", 3, badger_tga.as_slice()),
        ("c_badger", 2033, badger_dds.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let payload = archive
            .find(resref, resource_type)
            .map_err(|error| error.to_string())?;
        if payload != expected {
            return Err(format!(
                "LC-SQUIRREL-HAK-READBACK-MISMATCH: {resref} type {resource_type}"
            ));
        }
    }

    fs::create_dir_all(&args.output_dir).map_err(|error| error.to_string())?;
    fs::write(args.output_dir.join(HAK_FILE_NAME), &hak.payload)
        .map_err(|error| error.to_string())?;
    fs::write(args.output_dir.join(MODULE_FILE_NAME), &module.payload)
        .map_err(|error| error.to_string())?;
    fs::write(args.output_dir.join("appearance.2da"), &appearance.payload)
        .map_err(|error| error.to_string())?;
    fs::write(
        args.output_dir.join("lc-squirrel-reference-proof-manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "purpose": "external-reference-control; not a Meshy2Aurora model-writer proof",
            "source": {
                "logicalHak": "lc_hd_animals.hak",
                "model": {"resref": MODEL_RESREF, "type": 2002, "sha256": sha256(&squirrel_model)},
                "squirrelDds": {"resref": MODEL_RESREF, "type": 2033, "sha256": sha256(&squirrel_dds)},
                "badgerTga": {"resref": "c_badger", "type": 3, "sha256": sha256(&badger_tga)},
                "badgerDds": {"resref": "c_badger", "type": 2033, "sha256": sha256(&badger_dds)},
            },
            "appearance": {
                "label": APPEARANCE_LABEL,
                "physicalRow": appearance.report.appended_row_index,
                "modelType": "S",
                "race": MODEL_RESREF,
                "sourceRow": 15216,
                "sourceRowLabel": "(HD-ANIMAL) Squirrel - Brown",
                "allSourceRowCellsReplicatedExceptLabel": true,
                "appendReadback": appearance.report,
            },
            "outputs": {
                "hak": {"file": HAK_FILE_NAME, "sha256": sha256(&hak.payload)},
                "module": {"file": MODULE_FILE_NAME, "sha256": sha256(&module.payload)},
                "appearance": {"file": "appearance.2da", "sha256": sha256(&appearance.payload)},
            },
            "hakReadback": "PASS",
            "moduleReadback": module.report.semantic_readback_status,
        }))
        .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "LC_SQUIRREL_REFERENCE_PROOF={{\"appearanceRow\":{},\"model\":\"{}\",\"hakSha256\":\"{}\",\"moduleSha256\":\"{}\"}}",
        appearance.report.appended_row_index,
        MODEL_RESREF,
        sha256(&hak.payload),
        sha256(&module.payload),
    );
    Ok(())
}

fn squirrel_appearance_cells(columns: &[String]) -> Result<Vec<TwoDaCellAssignmentV1>, String> {
    let source_cells = [
        ("LABEL", Some(APPEARANCE_LABEL)),
        ("STRING_REF", None),
        ("NAME", Some("Squirrel")),
        ("RACE", Some(MODEL_RESREF)),
        ("ENVMAP", None),
        ("BLOODCOLR", Some("R")),
        ("MODELTYPE", Some("S")),
        ("WEAPONSCALE", None),
        ("WING_TAIL_SCALE", Some("1")),
        ("HELMET_SCALE_M", Some("1")),
        ("HELMET_SCALE_F", Some("1")),
        ("MOVERATE", Some("FAST")),
        ("WALKDIST", Some("0.77")),
        ("RUNDIST", Some("1.2")),
        ("PERSPACE", Some("0.3")),
        ("CREPERSPACE", Some("0.3")),
        ("HEIGHT", Some("1")),
        ("HITDIST", Some("0.3")),
        ("PREFATCKDIST", Some("2")),
        ("TARGETHEIGHT", Some("L")),
        ("ABORTONPARRY", Some("0")),
        ("RACIALTYPE", Some("2")),
        ("HASLEGS", Some("1")),
        ("HASARMS", Some("1")),
        ("PORTRAIT", Some("po_Rat")),
        ("SIZECATEGORY", Some("1")),
        ("PERCEPTIONDIST", Some("9")),
        ("FOOTSTEPTYPE", Some("3")),
        ("SOUNDAPPTYPE", Some("2")),
        ("HEADTRACK", Some("1")),
        ("HEAD_ARC_H", Some("60")),
        ("HEAD_ARC_V", Some("30")),
        ("HEAD_NAME", Some("Rat_head")),
        ("BODY_BAG", Some("0")),
        ("TARGETABLE", Some("1")),
    ];
    let mut cells = Vec::with_capacity(source_cells.len());
    for (column_name, value) in source_cells {
        if !columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(column_name))
        {
            return Err(format!(
                "LC-SQUIRREL-APPEARANCE-COLUMN-MISSING: {column_name}"
            ));
        }
        cells.push(TwoDaCellAssignmentV1 {
            column_name: column_name.to_owned(),
            value: match value {
                Some(value) => TwoDaCellValueV1::Text {
                    value: value.to_owned(),
                },
                None => TwoDaCellValueV1::Null,
            },
        });
    }
    Ok(cells)
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut base_appearance = None;
    let mut squirrel_model = None;
    let mut squirrel_dds = None;
    let mut badger_tga = None;
    let mut badger_dds = None;
    let mut output_dir = None;
    let mut items = arguments.into_iter();
    while let Some(argument) = items.next() {
        match argument.as_str() {
            "--base-appearance" => base_appearance = Some(required_value(&mut items, &argument)?),
            "--squirrel-model" => squirrel_model = Some(required_value(&mut items, &argument)?),
            "--squirrel-dds" => squirrel_dds = Some(required_value(&mut items, &argument)?),
            "--badger-tga" => badger_tga = Some(required_value(&mut items, &argument)?),
            "--badger-dds" => badger_dds = Some(required_value(&mut items, &argument)?),
            "--output-dir" => output_dir = Some(required_value(&mut items, &argument)?),
            "--help" | "-h" => return Err(usage()),
            _ => {
                return Err(format!(
                    "LC-SQUIRREL-ARGUMENT-UNKNOWN: {argument}\n{}",
                    usage()
                ));
            }
        }
    }
    Ok(Arguments {
        base_appearance: base_appearance.ok_or_else(usage)?,
        squirrel_model: squirrel_model.ok_or_else(usage)?,
        squirrel_dds: squirrel_dds.ok_or_else(usage)?,
        badger_tga: badger_tga.ok_or_else(usage)?,
        badger_dds: badger_dds.ok_or_else(usage)?,
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
        .ok_or_else(|| format!("LC-SQUIRREL-ARGUMENT-MISSING: {flag}"))
}

fn usage() -> String {
    "usage: materialize_lc_squirrel_reference_proof --base-appearance <path> --squirrel-model <path> --squirrel-dds <path> --badger-tga <path> --badger-dds <path> --output-dir <path>".to_owned()
}

fn read(path: &PathBuf) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|error| format!("LC-SQUIRREL-INPUT-READ-FAILED {}: {error}", path.display()))
}

fn read_and_verify(path: &PathBuf, label: &str, expected_sha256: &str) -> Result<Vec<u8>, String> {
    let bytes = read(path)?;
    let actual_sha256 = sha256(&bytes);
    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "{label}-SOURCE-HASH-MISMATCH: expected {expected_sha256}, got {actual_sha256}"
        ));
    }
    Ok(bytes)
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_row_covers_every_last_city_squirrel_column() {
        let columns = [
            "LABEL",
            "STRING_REF",
            "NAME",
            "RACE",
            "ENVMAP",
            "BLOODCOLR",
            "MODELTYPE",
            "WEAPONSCALE",
            "WING_TAIL_SCALE",
            "HELMET_SCALE_M",
            "HELMET_SCALE_F",
            "MOVERATE",
            "WALKDIST",
            "RUNDIST",
            "PERSPACE",
            "CREPERSPACE",
            "HEIGHT",
            "HITDIST",
            "PREFATCKDIST",
            "TARGETHEIGHT",
            "ABORTONPARRY",
            "RACIALTYPE",
            "HASLEGS",
            "HASARMS",
            "PORTRAIT",
            "SIZECATEGORY",
            "PERCEPTIONDIST",
            "FOOTSTEPTYPE",
            "SOUNDAPPTYPE",
            "HEADTRACK",
            "HEAD_ARC_H",
            "HEAD_ARC_V",
            "HEAD_NAME",
            "BODY_BAG",
            "TARGETABLE",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let cells = squirrel_appearance_cells(&columns).unwrap();
        assert_eq!(cells.len(), columns.len());
        assert!(cells.iter().any(|cell| {
            cell.column_name == "RACE"
                && cell.value
                    == TwoDaCellValueV1::Text {
                        value: MODEL_RESREF.to_owned(),
                    }
        }));
    }
}
