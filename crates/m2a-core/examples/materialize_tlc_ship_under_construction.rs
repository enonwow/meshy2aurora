use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    mdl::{NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, inspect_binary_mdl},
    placeable::{
        ARE_RESOURCE_TYPE, FAC_RESOURCE_TYPE, GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE,
        IFO_RESOURCE_TYPE, ITP_RESOURCE_TYPE, MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE,
        PWK_RESOURCE_TYPE, PlaceablePlacementV1, StaticPlaceableIdentityV1, TGA_RESOURCE_TYPE,
        UTP_RESOURCE_TYPE, build_meshy_static_placeable_package_v2,
        inspect_meshy_static_placeable_authoring_v1,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SOURCE_PATH: &str =
    r"C:\Projects\meshy2aurora\sample-3d\tlc-ship-under-construction-s1-p150k-v1\source.glb";
const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const SOURCE_TRIANGLES: u32 = 152_574;
const REFERENCE_TOP_SHA256: &str =
    "0530e60430361ecccf935fb2cbbc8000ee56dbdab8bfba907f48c0515c20c441";
const REFERENCE_STERN_SHA256: &str =
    "f907e64bf290291a3f808625ddf0b657229bc4899ca16446d6e35541de8e3fa4";
const REFERENCE_SIDE_SHA256: &str =
    "5ac81677a92eea70c83e870227761f89941295d8b9c78912ad43e334aba7ec48";
const REFERENCE_BOW_SHA256: &str =
    "a03ed01fa916f51f7abc927c5d918a919a72c9bf097f246bfab0a247f92640ae";
const BASE_PLACEABLES_PATH: &str = r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-v1-20260725\generated\base-placeables.2da";
const BASE_PLACEABLES_SHA256: &str =
    "b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90";
const OUTPUT_PATH: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-ship-under-construction-placeable-v1-20260731";
const NATIVE_MOD_PATH: &str =
    r"C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcs1_mod.mod";
const NATIVE_HAK_PATH: &str = r"C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcs1_hak.hak";
const SHIP_SCALE: f32 = 8.0;

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
    let output = PathBuf::from(OUTPUT_PATH);
    if output.exists() {
        return Err(format!("SHIP-DEMO-OUTPUT-EXISTS: {}", output.display()));
    }
    let source = read(Path::new(SOURCE_PATH), "SOURCE")?;
    require_hash(&source, SOURCE_SHA256, "SOURCE")?;
    let base_placeables = read(Path::new(BASE_PLACEABLES_PATH), "BASE-PLACEABLES")?;
    require_hash(&base_placeables, BASE_PLACEABLES_SHA256, "BASE-PLACEABLES")?;

    let identity = StaticPlaceableIdentityV1 {
        module_resref: "m2a_tlcs1_mod".to_owned(),
        module_file_name: "m2a_tlcs1_mod.mod".to_owned(),
        module_display_name: "The Last City - Ship Under Construction Demo".to_owned(),
        area_resref: "m2a_tlcs1_ar".to_owned(),
        area_name: "The Last City Shipyard Construction Demo".to_owned(),
        hak_resref: "m2a_tlcs1_hak".to_owned(),
        hak_file_name: "m2a_tlcs1_hak.hak".to_owned(),
        model_resref: "m2a_tlcs1_mdl".to_owned(),
        texture_resref: "m2a_tlcs1_tex".to_owned(),
        blueprint_resref: "m2a_tlcs1_utp".to_owned(),
        object_tag: "m2a_tlcs1_building_ship".to_owned(),
        display_name: "TLC Ship Under Construction".to_owned(),
    };
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let mut bootstrap = inspect_meshy_static_placeable_authoring_v1(&source)
        .map_err(|error| exact_error("SHIP-DEMO-INSPECTION", &error))?;
    if bootstrap.document.elements.len() != 1 {
        return Err(format!(
            "SHIP-DEMO-SOURCE-NODES: expected one source element, got {}",
            bootstrap.document.elements.len()
        ));
    }
    let sanitized_source_min_z = bootstrap
        .inspection
        .nodes
        .iter()
        .flat_map(|node| node.primitives.iter())
        .flat_map(|primitive| primitive.components.iter())
        .map(|component| component.bounds_min[2])
        .fold(f32::INFINITY, f32::min);
    if !sanitized_source_min_z.is_finite() {
        return Err("SHIP-DEMO-SOURCE-BOUNDS: no finite sanitized minimum Z".to_owned());
    }
    let element = &mut bootstrap.document.elements[0];
    element.transform.scale = [SHIP_SCALE; 3];
    element.transform.translation = [0.0, 0.0, -sanitized_source_min_z * SHIP_SCALE];
    let authoring_json = pretty(&bootstrap.document, "AUTHORING")?;

    let artifact = build_meshy_static_placeable_package_v2(
        &source,
        &base_placeables,
        &identity,
        placement,
        7,
        &bootstrap.document,
    )
    .map_err(|error| exact_error("SHIP-DEMO-PIPELINE", &error))?;
    if artifact.report.experimental_aggressive_geometry_cleanup {
        return Err(
            "SHIP-DEMO-CLEANUP: aggressive geometry cleanup must remain disabled".to_owned(),
        );
    }
    let authoring = artifact
        .report
        .authoring
        .as_ref()
        .ok_or_else(|| "SHIP-DEMO-AUTHORING-REPORT-MISSING".to_owned())?;
    let dimensions = [
        authoring.bounds_max[0] - authoring.bounds_min[0],
        authoring.bounds_max[1] - authoring.bounds_min[1],
        authoring.bounds_max[2] - authoring.bounds_min[2],
    ];
    if dimensions[0] < 12.0 || dimensions[1] < 6.0 || dimensions[2] < 5.0 {
        return Err(format!(
            "SHIP-DEMO-SCALE: expected ship-scale bounds, got {dimensions:?}"
        ));
    }
    if authoring.bounds_min[2].abs() > 0.001 {
        return Err(format!(
            "SHIP-DEMO-GROUNDING: expected minimum Z near zero, got {}",
            authoring.bounds_min[2]
        ));
    }
    if artifact.report.texture_resources.len() != 1 {
        return Err(format!(
            "SHIP-DEMO-TEXTURES: expected one generated texture, got {}",
            artifact.report.texture_resources.len()
        ));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("SHIP-DEMO-HAK-READBACK: {error}"))?;
    let module = ErfArchive::parse(&artifact.module_payload)
        .map_err(|error| format!("SHIP-DEMO-MOD-READBACK: {error}"))?;
    let mdl = hak
        .find(&identity.model_resref, MDL_RESOURCE_TYPE)
        .map_err(|error| format!("SHIP-DEMO-MDL-FIND: {error}"))?;
    let mdl_inspection =
        inspect_binary_mdl(mdl).map_err(|error| format!("SHIP-DEMO-MDL-READBACK: {error}"))?;
    let mut pending = mdl_inspection.node_tree.roots.iter().collect::<Vec<_>>();
    let mut render_stream_triangles = Vec::new();
    while let Some(node) = pending.pop() {
        if let Some(mesh) = node.mesh.as_ref().filter(|mesh| mesh.render != 0) {
            render_stream_triangles.push(mesh.faces.len());
        }
        pending.extend(&node.children);
    }
    let mdl_triangle_count = render_stream_triangles.iter().sum::<usize>();
    if mdl_triangle_count != authoring.output_triangle_count as usize
        || render_stream_triangles.is_empty()
        || render_stream_triangles
            .iter()
            .any(|count| *count > NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1)
    {
        return Err(format!(
            "SHIP-DEMO-MDL-SEGMENTATION: triangles={mdl_triangle_count} streams={render_stream_triangles:?}"
        ));
    }

    let report_json = pretty(&artifact.report, "REPORT")?;
    let geometry_readback = json!({
        "schemaVersion": 1,
        "sourceMeshyTriangles": SOURCE_TRIANGLES,
        "auroraSafeTriangles": authoring.output_triangle_count,
        "removedTrulyInvalidTriangles": SOURCE_TRIANGLES - authoring.output_triangle_count,
        "experimentalAggressiveGeometryCleanup": false,
        "renderStreamCount": render_stream_triangles.len(),
        "renderStreamTriangles": render_stream_triangles,
        "perStreamTriangleLimit": NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1,
        "dimensionsMetersAtDemoScale": dimensions,
        "boundsMin": authoring.bounds_min,
        "boundsMax": authoring.bounds_max,
        "uniformScale": SHIP_SCALE,
        "manualMeshOrTextureEdits": false,
    });
    let geometry_readback_json = pretty(&geometry_readback, "GEOMETRY-READBACK")?;

    let generated = output.join("generated");
    fs::create_dir_all(&generated).map_err(|error| format!("SHIP-DEMO-OUTPUT-CREATE: {error}"))?;
    let outputs = vec![
        (generated.join("source.glb"), source.clone()),
        (
            generated.join("base-placeables.2da"),
            base_placeables.clone(),
        ),
        (generated.join("authoring.json"), authoring_json.clone()),
        (
            generated.join("placeables.2da"),
            hak.find("placeables", PLACEABLES_2DA_RESOURCE_TYPE)
                .map_err(|error| error.to_string())?
                .to_vec(),
        ),
        (
            generated.join(format!("{}.mdl", identity.model_resref)),
            mdl.to_vec(),
        ),
        (
            generated.join(format!("{}.pwk", identity.model_resref)),
            hak.find(&identity.model_resref, PWK_RESOURCE_TYPE)
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
        (
            generated.join("geometry-readback.json"),
            geometry_readback_json.clone(),
        ),
    ];
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        if read(path, "OUTPUT-READBACK")? != *expected {
            return Err(format!("SHIP-DEMO-OUTPUT-MISMATCH: {}", path.display()));
        }
    }

    let installed_mod = install_exact(
        &generated.join(&identity.module_file_name),
        Path::new(NATIVE_MOD_PATH),
        "MOD",
    )?;
    let installed_hak = install_exact(
        &generated.join(&identity.hak_file_name),
        Path::new(NATIVE_HAK_PATH),
        "HAK",
    )?;

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
        "scale": SHIP_SCALE,
        "dimensionsMeters": dimensions,
        "source": {
            "path": SOURCE_PATH,
            "sha256": SOURCE_SHA256,
            "byteLength": source.len(),
            "referenceImages": [
                { "path": "concept-top.jpg", "sha256": REFERENCE_TOP_SHA256 },
                { "path": "concept-stern.jpg", "sha256": REFERENCE_STERN_SHA256 },
                { "path": "concept-side-three-quarter.jpg", "sha256": REFERENCE_SIDE_SHA256 },
                { "path": "concept-bow.jpg", "sha256": REFERENCE_BOW_SHA256 },
            ],
            "meshyLocalBridgeRunId": "51a32abd-9fe9-46de-ba15-1c8bddd0c689",
            "meshyTaskId": "019fb9aa-379a-7024-9e10-8418206ac9b7",
            "requestedPolycount": 150_000,
            "generatedTriangles": SOURCE_TRIANGLES,
            "spentCredits": 30,
            "manualMeshOrTextureEdits": false,
        },
        "geometry": geometry_readback,
        "outputs": {
            "mdlSha256": artifact.report.mdl_sha256,
            "pwkSha256": artifact.report.pwk_sha256,
            "textureSha256": artifact.report.texture_sha256,
            "placeables2daSha256": artifact.report.placeables_2da_sha256,
            "utpSha256": artifact.report.utp_sha256,
            "itpSha256": artifact.report.itp_sha256,
            "gitSha256": artifact.report.git_sha256,
            "gicSha256": artifact.report.gic_sha256,
            "hakSha256": artifact.report.hak_sha256,
            "moduleSha256": artifact.report.module_sha256,
            "reportSha256": sha256(&report_json),
            "geometryReadbackSha256": sha256(&geometry_readback_json),
            "authoringSha256": sha256(&authoring_json),
        },
        "nativeInstallation": {
            "module": installed_mod,
            "hak": installed_hak,
        },
        "componentStatuses": artifact.report.component_statuses,
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "paletteCompleteness": artifact.report.palette_completeness,
        "collisionCompleteness": artifact.report.collision_completeness,
        "ownerProofRequired": true,
        "startsToolset": false,
        "startsNwn": false,
        "materializationCount": 1,
    });
    let handoff_json = pretty(&handoff, "HANDOFF")?;
    let handoff_path = output.join("ready-for-owner-proof.json");
    write_new(&handoff_path, &handoff_json)?;
    if read(&handoff_path, "HANDOFF-READBACK")? != handoff_json {
        return Err("SHIP-DEMO-HANDOFF-MISMATCH".to_owned());
    }
    serde_json::to_string_pretty(&handoff).map_err(|error| format!("SHIP-DEMO-SUMMARY: {error}"))
}

fn install_exact(source: &Path, destination: &Path, label: &str) -> Result<Value, String> {
    let source = fs::canonicalize(source)
        .map_err(|error| format!("SHIP-DEMO-{label}-SOURCE-RESOLVE: {error}"))?;
    let source_bytes = read(&source, &format!("{label}-INSTALL-SOURCE"))?;
    let source_hash = sha256(&source_bytes);
    let destination_parent = destination
        .parent()
        .ok_or_else(|| format!("SHIP-DEMO-{label}-DESTINATION-PARENT"))?;
    let destination_parent = fs::canonicalize(destination_parent)
        .map_err(|error| format!("SHIP-DEMO-{label}-DESTINATION-RESOLVE: {error}"))?;
    let destination = destination_parent.join(
        destination
            .file_name()
            .ok_or_else(|| format!("SHIP-DEMO-{label}-DESTINATION-NAME"))?,
    );
    let reused = if destination.exists() {
        let existing = read(&destination, &format!("{label}-INSTALL-EXISTING"))?;
        if existing != source_bytes {
            return Err(format!(
                "SHIP-DEMO-{label}-NATIVE-COLLISION: {} existingSha256={} sourceSha256={source_hash}",
                destination.display(),
                sha256(&existing)
            ));
        }
        true
    } else {
        write_new(&destination, &source_bytes)?;
        false
    };
    let installed = read(&destination, &format!("{label}-INSTALL-READBACK"))?;
    if installed != source_bytes {
        return Err(format!("SHIP-DEMO-{label}-INSTALL-HASH-MISMATCH"));
    }
    Ok(json!({
        "sourcePath": source,
        "destinationPath": destination,
        "byteLength": source_bytes.len(),
        "sha256": source_hash,
        "reusedIdenticalExisting": reused,
        "byteIdentical": true,
    }))
}

fn exact_error(label: &str, error: &impl serde::Serialize) -> String {
    serde_json::to_string(error).unwrap_or_else(|_| format!("{label}: serialization failed"))
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("SHIP-DEMO-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("SHIP-DEMO-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("SHIP-DEMO-WRITE {}: {error}", path.display()))
}

fn require_hash(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256(bytes);
    if actual != expected {
        return Err(format!(
            "SHIP-DEMO-{label}-HASH: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn pretty(value: &impl serde::Serialize, label: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value)
        .map_err(|error| format!("SHIP-DEMO-{label}-SERIALIZE: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
