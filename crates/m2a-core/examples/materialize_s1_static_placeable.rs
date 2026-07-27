use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    placeable::{
        ARE_RESOURCE_TYPE, FAC_RESOURCE_TYPE, GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE,
        IFO_RESOURCE_TYPE, ITP_RESOURCE_TYPE, MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE,
        PlaceablePlacementV1, StaticPlaceableIdentityV1, TGA_RESOURCE_TYPE, UTP_RESOURCE_TYPE,
        build_meshy_static_placeable_package_v1,
    },
};
use sha2::{Digest, Sha256};

const CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\s1-placeable-ritual-pedestal-p1-20260725";
const SOURCE_SHA256: &str = "dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372";
const BASE_PLACEABLES_SHA256: &str =
    "b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90";

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
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
    if command.output != PathBuf::from(CANONICAL_OUTPUT) {
        return Err(format!(
            "PLACEABLE-S1-OUTPUT-IDENTITY: exact output must be {CANONICAL_OUTPUT}"
        ));
    }
    if command.output.exists() {
        return Err(format!(
            "PLACEABLE-S1-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }
    let source = read(&command.source_glb, "SOURCE")?;
    require_hash(&source, SOURCE_SHA256, "SOURCE")?;
    let base_placeables = read(&command.placeables_two_da, "PLACEABLES-2DA")?;
    require_hash(&base_placeables, BASE_PLACEABLES_SHA256, "PLACEABLES-2DA")?;

    let identity = StaticPlaceableIdentityV1 {
        module_resref: "m2a_s1_plc_mod".to_owned(),
        module_file_name: "m2a_s1_plc_mod.mod".to_owned(),
        module_display_name: "Meshy2Aurora S1 Placeable Proof".to_owned(),
        area_resref: "m2a_s1_plc_ar".to_owned(),
        area_name: "Meshy2Aurora S1 Ritual Pedestal".to_owned(),
        hak_resref: "m2a_s1_plc_hak".to_owned(),
        hak_file_name: "m2a_s1_plc_hak.hak".to_owned(),
        model_resref: "m2a_s1_plc_ped".to_owned(),
        texture_resref: "m2a_s1_plc_tex".to_owned(),
        blueprint_resref: "m2a_s1_plc_utp".to_owned(),
        object_tag: "m2a_s1_ritual_pedestal".to_owned(),
        display_name: "Meshy Ritual Pedestal".to_owned(),
    };
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let artifact =
        build_meshy_static_placeable_package_v1(&source, &base_placeables, &identity, placement, 7)
            .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    if artifact.report.appearance_row.value != 16_500 {
        return Err(format!(
            "PLACEABLE-S1-APPEARANCE-ROW: expected 16500, got {}",
            artifact.report.appearance_row.value
        ));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("PLACEABLE-S1-HAK-READBACK: {error}"))?;
    let module = ErfArchive::parse(&artifact.module_payload)
        .map_err(|error| format!("PLACEABLE-S1-MOD-READBACK: {error}"))?;
    let report_json = serde_json::to_vec_pretty(&artifact.report)
        .map_err(|error| format!("PLACEABLE-S1-REPORT: {error}"))?;
    let generated = command.output.join("generated");
    fs::create_dir_all(&generated)
        .map_err(|error| format!("PLACEABLE-S1-OUTPUT-CREATE: {error}"))?;

    let outputs = vec![
        (generated.join("source.glb"), source.clone()),
        (
            generated.join("placeables.2da"),
            hak.find("placeables", PLACEABLES_2DA_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.mdl", identity.model_resref)),
            hak.find(&identity.model_resref, MDL_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.tga", identity.texture_resref)),
            hak.find(&identity.texture_resref, TGA_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.utp", identity.blueprint_resref)),
            module
                .find(&identity.blueprint_resref, UTP_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join("placeablepalcus.itp"),
            module
                .find("placeablepalcus", ITP_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.git", identity.area_resref)),
            module
                .find(&identity.area_resref, GIT_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.gic", identity.area_resref)),
            module
                .find(&identity.area_resref, GIC_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.are", identity.area_resref)),
            module
                .find(&identity.area_resref, ARE_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join("module.ifo"),
            module
                .find("module", IFO_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join("repute.fac"),
            module
                .find("repute", FAC_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(&identity.hak_file_name),
            artifact.hak_payload.clone(),
        ),
        (
            generated.join(&identity.module_file_name),
            artifact.module_payload.clone(),
        ),
        (generated.join("placeable-report.json"), report_json.clone()),
    ];
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        let actual = read(path, "WRITTEN-OUTPUT")?;
        if actual != *expected {
            return Err(format!("PLACEABLE-S1-OUTPUT-READBACK: {}", path.display()));
        }
    }

    let handoff = serde_json::json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "testModuleFileName": identity.module_file_name,
        "toolsetModuleName": identity.module_display_name,
        "areaName": identity.area_name,
        "areaResref": identity.area_resref,
        "orderedHakFiles": [identity.hak_file_name],
        "orderedHakResrefs": [identity.hak_resref],
        "modelResref": identity.model_resref,
        "textureResref": identity.texture_resref,
        "blueprintResref": identity.blueprint_resref,
        "objectTag": identity.object_tag,
        "appearanceRow": artifact.report.appearance_row.value,
        "placement": placement,
        "sourceGlb": {
            "path": command.source_glb,
            "byteLength": source.len(),
            "sha256": sha256(&source),
        },
        "basePlaceables2da": {
            "path": command.placeables_two_da,
            "byteLength": base_placeables.len(),
            "sha256": sha256(&base_placeables),
        },
        "outputs": {
            "mdlSha256": artifact.report.mdl_sha256,
            "textureSha256": artifact.report.texture_sha256,
            "placeables2daSha256": artifact.report.placeables_2da_sha256,
            "utpSha256": artifact.report.utp_sha256,
            "itpSha256": artifact.report.itp_sha256,
            "gitSha256": artifact.report.git_sha256,
            "gicSha256": artifact.report.gic_sha256,
            "hakSha256": artifact.report.hak_sha256,
            "moduleSha256": artifact.report.module_sha256,
            "reportSha256": sha256(&report_json),
        },
        "componentStatuses": artifact.report.component_statuses,
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "paletteCompleteness": artifact.report.palette_completeness,
        "collisionCompleteness": artifact.report.collision_completeness,
        "ownerProofRequired": true,
        "startsToolset": false,
        "startsNwn": false,
        "installsNativeNwnArtifacts": false,
        "materializationCount": 1,
    });
    let handoff_json = serde_json::to_vec_pretty(&handoff)
        .map_err(|error| format!("PLACEABLE-S1-HANDOFF: {error}"))?;
    let handoff_path = command.output.join("ready-for-owner-proof.json");
    write_new(&handoff_path, &handoff_json)?;
    if read(&handoff_path, "HANDOFF-READBACK")? != handoff_json {
        return Err("PLACEABLE-S1-HANDOFF-READBACK".to_owned());
    }

    serde_json::to_string_pretty(&handoff).map_err(|error| format!("PLACEABLE-S1-SUMMARY: {error}"))
}

struct Command {
    source_glb: PathBuf,
    placeables_two_da: PathBuf,
    output: PathBuf,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut placeables_two_da = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--placeables-2da" => &mut placeables_two_da,
            "--out" => &mut output,
            _ => return Err(format!("PLACEABLE-S1-ARGUMENT: {argument}\n{}", usage())),
        };
        if target.is_some() {
            return Err(format!("PLACEABLE-S1-ARGUMENT-DUPLICATE: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("PLACEABLE-S1-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        placeables_two_da: PathBuf::from(placeables_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn usage() -> String {
    format!(
        "usage: materialize_s1_static_placeable --source-glb <exact-s1.glb> --placeables-2da <exact-full-table> --out {CANONICAL_OUTPUT}"
    )
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("PLACEABLE-S1-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("PLACEABLE-S1-OUTPUT-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("PLACEABLE-S1-OUTPUT-WRITE {}: {error}", path.display()))
}

fn require_hash(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256(bytes);
    if actual != expected {
        return Err(format!(
            "PLACEABLE-S1-{label}-HASH: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
