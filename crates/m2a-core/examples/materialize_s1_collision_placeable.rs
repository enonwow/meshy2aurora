use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    placeable::{
        MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE, PWK_RESOURCE_TYPE, PlaceablePlacementV1,
        StaticPlaceableIdentityV1, TGA_RESOURCE_TYPE, build_meshy_static_placeable_package_v1,
    },
    placeable_collision::inspect_ascii_placeable_walkmesh_v1,
};
use serde_json::json;
use sha2::{Digest, Sha256};

const V1_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v1-20260725";
const V2_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v2-adjacency-20260725";
const V3_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v3-ascii-pwk-20260725";
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
    let candidate = candidate(&command.candidate)?;
    if command.output.as_path() != Path::new(candidate.canonical_output) {
        return Err(format!(
            "PLACEABLE-COLLISION-OUTPUT-IDENTITY: exact output must be {}",
            candidate.canonical_output
        ));
    }
    if command.output.exists() {
        return Err(format!(
            "PLACEABLE-COLLISION-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }
    let source = read(&command.source_glb, "SOURCE")?;
    require_hash(&source, SOURCE_SHA256, "SOURCE")?;
    let base_placeables = read(&command.placeables_two_da, "PLACEABLES-2DA")?;
    require_hash(&base_placeables, BASE_PLACEABLES_SHA256, "PLACEABLES-2DA")?;

    let identity = StaticPlaceableIdentityV1 {
        module_resref: candidate.module_resref.to_owned(),
        module_file_name: candidate.module_file_name.to_owned(),
        module_display_name: candidate.module_display_name.to_owned(),
        area_resref: candidate.area_resref.to_owned(),
        area_name: "Meshy2Aurora M0 binary vertical-slice area".to_owned(),
        hak_resref: candidate.hak_resref.to_owned(),
        hak_file_name: candidate.hak_file_name.to_owned(),
        model_resref: candidate.model_resref.to_owned(),
        texture_resref: candidate.texture_resref.to_owned(),
        blueprint_resref: candidate.blueprint_resref.to_owned(),
        object_tag: candidate.object_tag.to_owned(),
        display_name: candidate.display_name.to_owned(),
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
            "PLACEABLE-COLLISION-APPEARANCE-ROW: expected 16500, got {}",
            artifact.report.appearance_row.value
        ));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("PLACEABLE-COLLISION-HAK-READBACK: {error}"))?;
    let pwk = hak
        .find(&identity.model_resref, PWK_RESOURCE_TYPE)
        .map_err(|error| format!("PLACEABLE-COLLISION-PWK-RESOURCE: {error}"))?;
    require_pwk_contract(pwk)?;

    let generated = command.output.join("generated");
    fs::create_dir_all(&generated)
        .map_err(|error| format!("PLACEABLE-COLLISION-OUTPUT-CREATE: {error}"))?;
    let report_json = serde_json::to_vec_pretty(&artifact.report)
        .map_err(|error| format!("PLACEABLE-COLLISION-REPORT: {error}"))?;
    let outputs = vec![
        (
            generated.join(&identity.module_file_name),
            artifact.module_payload.clone(),
        ),
        (
            generated.join(&identity.hak_file_name),
            artifact.hak_payload.clone(),
        ),
        (
            generated.join(format!("{}.mdl", identity.model_resref)),
            hak.find(&identity.model_resref, MDL_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.pwk", identity.model_resref)),
            pwk.to_vec(),
        ),
        (
            generated.join(format!("{}.tga", identity.texture_resref)),
            hak.find(&identity.texture_resref, TGA_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join("placeables.2da"),
            hak.find("placeables", PLACEABLES_2DA_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (generated.join("placeable-report.json"), report_json.clone()),
    ];
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        if read(path, "OUTPUT-READBACK")? != *expected {
            return Err(format!(
                "PLACEABLE-COLLISION-OUTPUT-MISMATCH: {}",
                path.display()
            ));
        }
    }

    let handoff = json!({
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
            "moduleSha256": artifact.report.module_sha256,
            "hakSha256": artifact.report.hak_sha256,
            "mdlSha256": artifact.report.mdl_sha256,
            "pwkSha256": artifact.report.pwk_sha256,
            "textureSha256": artifact.report.texture_sha256,
            "placeables2daSha256": artifact.report.placeables_2da_sha256,
            "reportSha256": sha256(&report_json),
        },
        "nativeInstallation": {
            "status": "not_installed",
            "reason": "owner-run proof policy requires a separate exact installation instruction",
            "canonicalModule": generated.join(&identity.module_file_name),
            "canonicalHak": generated.join(&identity.hak_file_name),
        },
        "componentStatuses": artifact.report.component_statuses,
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "collisionCompleteness": artifact.report.collision_completeness,
        "collisionRuntimeVerdict": "not_tested",
        "collisionContract": {
            "resourceType": PWK_RESOURCE_TYPE,
            "sameResrefAsMdl": true,
            "vertexCount": 4,
            "faceCount": 2,
            "surfaceId": 7,
            "format": "nwn1-ascii-pwk",
            "runtimeLoader": "CNWPlaceableSurfaceMesh::LoadWalkMesh",
            "adjacentFaces": [
                [0, 0, 0],
                [0, 0, 0]
            ],
            "usePointCount": 0,
            "shape": "conservative_world_xy_rectangle",
        },
        "ownerProofRequired": true,
        "ownerTest": [
            "Open the exact module and Area named above.",
            "Install the exact canonical MOD and HAK only after approving this handoff.",
            "Run Test Module without rebuilding or replacing the exact HAK.",
            "Walk the player into the pedestal from at least four directions.",
            "Pass requires the player to be blocked by the pedestal footprint."
        ],
        "startsToolset": false,
        "startsNwn": false,
        "installsNativeNwnArtifacts": false,
        "materializationCount": 1,
        "featureLane": candidate.feature_lane,
        "iterationBasis": candidate.iteration_basis,
        "priorVisibilityEvidencePreserved": "s1-placeable-ritual-pedestal-p1-owner-visual-result-2026-07-25.json",
    });
    let handoff_json = serde_json::to_vec_pretty(&handoff)
        .map_err(|error| format!("PLACEABLE-COLLISION-HANDOFF: {error}"))?;
    let handoff_path = command.output.join("ready-for-owner-proof.json");
    write_new(&handoff_path, &handoff_json)?;
    if read(&handoff_path, "HANDOFF-READBACK")? != handoff_json {
        return Err("PLACEABLE-COLLISION-HANDOFF-READBACK".to_owned());
    }

    serde_json::to_string_pretty(&handoff)
        .map_err(|error| format!("PLACEABLE-COLLISION-SUMMARY: {error}"))
}

fn require_pwk_contract(pwk: &[u8]) -> Result<(), String> {
    let report = inspect_ascii_placeable_walkmesh_v1(pwk)
        .map_err(|error| format!("PLACEABLE-COLLISION-PWK-ASCII-READBACK: {error}"))?;
    let mesh = report
        .mesh_nodes
        .first()
        .ok_or_else(|| "PLACEABLE-COLLISION-PWK-MESH-MISSING".to_owned())?;
    let surface_ids = mesh
        .faces
        .iter()
        .map(|face| face.surface_id)
        .collect::<Vec<_>>();
    let adjacent_faces = mesh
        .faces
        .iter()
        .map(|face| face.adjacent_faces)
        .collect::<Vec<_>>();
    if report.format != "nwn1-ascii-pwk"
        || report.mesh_nodes.len() != 1
        || mesh.vertices.len() != 4
        || mesh.faces.len() != 2
        || surface_ids != [7, 7]
        || adjacent_faces != [[0, 0, 0], [0, 0, 0]]
        || !report.use_points.is_empty()
    {
        return Err(format!(
            "PLACEABLE-COLLISION-PWK-SEMANTICS: format={}, meshes={}, vertices={}, faces={}, surfaces={surface_ids:?}, adjacency={adjacent_faces:?}, usepoints={}",
            report.format,
            report.mesh_nodes.len(),
            mesh.vertices.len(),
            mesh.faces.len(),
            report.use_points.len(),
        ));
    }
    Ok(())
}

struct Candidate {
    canonical_output: &'static str,
    module_resref: &'static str,
    module_file_name: &'static str,
    module_display_name: &'static str,
    area_resref: &'static str,
    hak_resref: &'static str,
    hak_file_name: &'static str,
    model_resref: &'static str,
    texture_resref: &'static str,
    blueprint_resref: &'static str,
    object_tag: &'static str,
    display_name: &'static str,
    feature_lane: &'static str,
    iteration_basis: &'static str,
}

fn candidate(version: &str) -> Result<Candidate, String> {
    match version {
        "v1" => Ok(Candidate {
            canonical_output: V1_CANONICAL_OUTPUT,
            module_resref: "m2a_s1_col_mod",
            module_file_name: "m2a_s1_col_mod.mod",
            module_display_name: "Meshy2Aurora S1 Collision Proof",
            area_resref: "m2a_s1_col_ar",
            hak_resref: "m2a_s1_col_hak",
            hak_file_name: "m2a_s1_col_hak.hak",
            model_resref: "m2a_s1_col_ped",
            texture_resref: "m2a_s1_col_tex",
            blueprint_resref: "m2a_s1_col_utp",
            object_tag: "m2a_s1_collision",
            display_name: "Meshy Ritual Pedestal Collision",
            feature_lane: "P8_STATIC_PLACEABLE_COLLISION_V1",
            iteration_basis: "initial_collision_candidate",
        }),
        "v2" => Ok(Candidate {
            canonical_output: V2_CANONICAL_OUTPUT,
            module_resref: "m2a_s1_c2_mod",
            module_file_name: "m2a_s1_c2_mod.mod",
            module_display_name: "Meshy2Aurora S1 Collision V2",
            area_resref: "m2a_s1_c2_ar",
            hak_resref: "m2a_s1_c2_hak",
            hak_file_name: "m2a_s1_c2_hak.hak",
            model_resref: "m2a_s1_c2_ped",
            texture_resref: "m2a_s1_c2_tex",
            blueprint_resref: "m2a_s1_c2_utp",
            object_tag: "m2a_s1_c2_obj",
            display_name: "Meshy Ritual Pedestal Collision V2",
            feature_lane: "P8_STATIC_PLACEABLE_COLLISION_V2_ADJACENCY",
            iteration_basis: "owner_reported_v1_not_blocking_and_offline_adjacency_diagnosis",
        }),
        "v3" => Ok(Candidate {
            canonical_output: V3_CANONICAL_OUTPUT,
            module_resref: "m2a_s1_c3_mod",
            module_file_name: "m2a_s1_c3_mod.mod",
            module_display_name: "Meshy2Aurora S1 Collision V3 ASCII PWK",
            area_resref: "m2a_s1_c3_ar",
            hak_resref: "m2a_s1_c3_hak",
            hak_file_name: "m2a_s1_c3_hak.hak",
            model_resref: "m2a_s1_c3_ped",
            texture_resref: "m2a_s1_c3_tex",
            blueprint_resref: "m2a_s1_c3_utp",
            object_tag: "m2a_s1_c3_obj",
            display_name: "Meshy Ritual Pedestal Collision V3 ASCII PWK",
            feature_lane: "P8_STATIC_PLACEABLE_COLLISION_V3_ASCII_PWK",
            iteration_basis: "owner_reported_v2_not_blocking_and_confirmed_binary_pwk_loader_mismatch",
        }),
        _ => Err(format!(
            "PLACEABLE-COLLISION-CANDIDATE: expected v1, v2 or v3, got {version}"
        )),
    }
}

struct Command {
    candidate: String,
    source_glb: PathBuf,
    placeables_two_da: PathBuf,
    output: PathBuf,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut candidate = Some("v3".to_owned());
    let mut source_glb = None;
    let mut placeables_two_da = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--candidate" => &mut candidate,
            "--source-glb" => &mut source_glb,
            "--placeables-2da" => &mut placeables_two_da,
            "--out" => &mut output,
            _ => return Err(format!("PLACEABLE-COLLISION-ARGUMENT: {argument}")),
        };
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("PLACEABLE-COLLISION-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        candidate: candidate.ok_or("PLACEABLE-COLLISION-CANDIDATE-ARGUMENT-MISSING")?,
        source_glb: PathBuf::from(source_glb.ok_or("PLACEABLE-COLLISION-SOURCE-ARGUMENT-MISSING")?),
        placeables_two_da: PathBuf::from(
            placeables_two_da.ok_or("PLACEABLE-COLLISION-2DA-ARGUMENT-MISSING")?,
        ),
        output: PathBuf::from(output.ok_or("PLACEABLE-COLLISION-OUTPUT-ARGUMENT-MISSING")?),
    })
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| {
        format!(
            "PLACEABLE-COLLISION-{label}-READ {}: {error}",
            path.display()
        )
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("PLACEABLE-COLLISION-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("PLACEABLE-COLLISION-WRITE {}: {error}", path.display()))
}

fn require_hash(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256(bytes);
    if actual != expected {
        return Err(format!(
            "PLACEABLE-COLLISION-{label}-HASH: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
