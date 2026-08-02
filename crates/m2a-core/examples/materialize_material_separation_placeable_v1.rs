use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    glb::{GlbLimits, ingest_glb},
    mdl::{NodeReport, inspect_binary_mdl},
    model_components::SourceComponentKeyV1,
    model_material_separation::{
        AuthoredMaterialV1, ModelMaterialAssignmentV1, ModelMaterialSeparationDocumentV1,
        resolve_model_materials_v1,
    },
    model_texture_authoring::{
        ModelTextureBindingModeV1, ModelTexturePayloadDescriptorV1,
        default_model_texture_authoring_v1,
    },
    placeable::{
        MDL_RESOURCE_TYPE, PlaceablePlacementV1, StaticPlaceableIdentityV1, TGA_RESOURCE_TYPE,
        build_meshy_static_placeable_package_v6, inspect_meshy_static_placeable_authoring_v3,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "../tests/fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod synthetic_glb;

const OUTPUT_PATH: &str =
    r"C:\Projects\meshy2aurora\proof-output\material-separation-placeable-v1-20260801";
const NATIVE_MOD_PATH: &str =
    r"C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ms1_mod.mod";
const NATIVE_HAK_PATH: &str = r"C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ms1_hak.hak";

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
        return Err(format!("MS1-OUTPUT-EXISTS: {}", output.display()));
    }
    let source = synthetic_glb::one_primitive_two_disconnected_triangles_with_embedded_texture();
    let source_sha256 = sha256(&source);
    let ingest = ingest_glb(&source, &GlbLimits::default())
        .map_err(|error| exact_error("MS1-INGEST", &error))?;
    if ingest.report.statistics.triangle_count != 2 {
        return Err(format!(
            "MS1-SOURCE-TRIANGLES: expected 2, got {}",
            ingest.report.statistics.triangle_count
        ));
    }
    let fallback_sha256 = ingest
        .ir
        .images
        .first()
        .ok_or_else(|| "MS1-SOURCE-IMAGE-MISSING".to_owned())?
        .sha256
        .clone();

    let material = |id: &str, name: &str, color: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: name.to_owned(),
        preview_color: color.to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: Some(fallback_sha256.clone()),
    };
    let separation = ModelMaterialSeparationDocumentV1 {
        schema_version: 1,
        source_sha256: source_sha256.clone(),
        materials: vec![
            material("material:red-source", "Red source panel", "#d05045"),
            material("material:blue-override", "Blue override panel", "#486fd0"),
        ],
        assignments: vec![
            ModelMaterialAssignmentV1 {
                component: SourceComponentKeyV1 {
                    scene_id: 0,
                    node_id: 0,
                    primitive_id: 0,
                    component_index: 0,
                },
                authored_material_id: "material:red-source".to_owned(),
            },
            ModelMaterialAssignmentV1 {
                component: SourceComponentKeyV1 {
                    scene_id: 0,
                    node_id: 0,
                    primitive_id: 0,
                    component_index: 1,
                },
                authored_material_id: "material:blue-override".to_owned(),
            },
        ],
    };
    let materials = resolve_model_materials_v1(&ingest.ir, &separation)
        .map_err(|error| exact_error("MS1-MATERIALS", &error))?;
    let mut texture_authoring = default_model_texture_authoring_v1(&ingest, &materials)
        .map_err(|error| exact_error("MS1-TEXTURES", &error))?;
    let override_bytes = synthetic_glb::OWNED_BLUE_RGBA_PNG.to_vec();
    let override_sha256 = sha256(&override_bytes);
    let blue_binding = texture_authoring
        .bindings
        .iter_mut()
        .find(|binding| binding.authored_material_id == "material:blue-override")
        .ok_or_else(|| "MS1-BLUE-BINDING-MISSING".to_owned())?;
    blue_binding.mode = ModelTextureBindingModeV1::Override;
    blue_binding.override_asset_id = Some("override:blue-panel".to_owned());
    blue_binding.override_sha256 = Some(override_sha256.clone());
    blue_binding.override_mime_type = Some("image/png".to_owned());
    blue_binding.override_byte_length = Some(override_bytes.len() as u64);
    let descriptors = [ModelTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "override:blue-panel".to_owned(),
        sha256: override_sha256,
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: override_bytes.len() as u64,
    }];

    let mut bootstrap = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .map_err(|error| exact_error("MS1-AUTHORING", &error))?;
    let element = bootstrap
        .document
        .elements
        .first_mut()
        .ok_or_else(|| "MS1-AUTHORING-ELEMENT-MISSING".to_owned())?;
    element.transform.scale = [2.5; 3];

    let identity = StaticPlaceableIdentityV1 {
        module_resref: "m2a_ms1_mod".to_owned(),
        module_file_name: "m2a_ms1_mod.mod".to_owned(),
        module_display_name: "Meshy2Aurora Material Separation V1".to_owned(),
        area_resref: "m2a_ms1_area".to_owned(),
        area_name: "Material Separation Two Material Proof".to_owned(),
        hak_resref: "m2a_ms1_hak".to_owned(),
        hak_file_name: "m2a_ms1_hak.hak".to_owned(),
        model_resref: "m2a_ms1_mdl".to_owned(),
        texture_resref: "m2a_ms1_tex".to_owned(),
        blueprint_resref: "m2a_ms1_utp".to_owned(),
        object_tag: "m2a_ms1_two_material_panels".to_owned(),
        display_name: "Material Separation Red Blue Panels".to_owned(),
    };
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let build = || {
        build_meshy_static_placeable_package_v6(
            &source,
            &base_placeables_2da(),
            &identity,
            placement,
            7,
            &bootstrap.document,
            &separation,
            &texture_authoring,
            &override_bytes,
            &descriptors,
            &Default::default(),
        )
        .map_err(|error| exact_error("MS1-PIPELINE", &error))
    };
    let artifact = build()?;
    let repeated = build()?;
    if artifact.hak_payload != repeated.hak_payload
        || artifact.module_payload != repeated.module_payload
        || serde_json::to_vec(&artifact.report).ok() != serde_json::to_vec(&repeated.report).ok()
    {
        return Err("MS1-NONDETERMINISTIC-PACKAGE".to_owned());
    }

    let separation_report = artifact
        .report
        .material_separation
        .as_ref()
        .ok_or_else(|| "MS1-SEPARATION-REPORT-MISSING".to_owned())?;
    let texture_report = artifact
        .report
        .model_texture_authoring
        .as_ref()
        .ok_or_else(|| "MS1-TEXTURE-REPORT-MISSING".to_owned())?;
    if separation_report.material_slots.len() != 2 || texture_report.resources.len() != 2 {
        return Err(format!(
            "MS1-MATERIAL-COUNT: slots={} textures={}",
            separation_report.material_slots.len(),
            texture_report.resources.len()
        ));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("MS1-HAK-READBACK: {error}"))?;
    for resource in &texture_report.resources {
        hak.find(&resource.resref, TGA_RESOURCE_TYPE)
            .map_err(|error| format!("MS1-TGA-READBACK {}: {error}", resource.resref))?;
    }
    let mdl = hak
        .find(&identity.model_resref, MDL_RESOURCE_TYPE)
        .map_err(|error| format!("MS1-MDL-FIND: {error}"))?;
    let inspection =
        inspect_binary_mdl(mdl).map_err(|error| format!("MS1-MDL-READBACK: {error}"))?;
    let mut mdl_textures = BTreeSet::new();
    let mut mdl_triangles = 0usize;
    for node in &inspection.node_tree.roots {
        collect_render_readback(node, &mut mdl_textures, &mut mdl_triangles);
    }
    let expected_textures = texture_report
        .resources
        .iter()
        .map(|resource| resource.resref.clone())
        .collect::<BTreeSet<_>>();
    if mdl_triangles != 2 || mdl_textures != expected_textures {
        return Err(format!(
            "MS1-MDL-MATERIAL-READBACK: triangles={mdl_triangles} textures={mdl_textures:?} expected={expected_textures:?}"
        ));
    }

    let generated = output.join("generated");
    fs::create_dir_all(&generated).map_err(|error| format!("MS1-OUTPUT-CREATE: {error}"))?;
    let report_json = pretty(&artifact.report, "REPORT")?;
    let separation_json = pretty(&separation, "SEPARATION")?;
    let texture_authoring_json = pretty(&texture_authoring, "TEXTURE-AUTHORING")?;
    let readback_json = pretty(
        &json!({
            "schemaVersion": 1,
            "sourceSha256": source_sha256,
            "sourceTriangles": 2,
            "outputTriangles": mdl_triangles,
            "materialSlotCount": separation_report.material_slots.len(),
            "textureResrefs": mdl_textures,
            "sourceGeometryPreserved": true,
            "uv0Preserved": true,
            "deterministicRepeat": true,
            "experimentalAggressiveGeometryCleanup": false,
        }),
        "READBACK",
    )?;
    let outputs = [
        (generated.join("synthetic-source.glb"), source.clone()),
        (
            generated.join("material-separation.json"),
            separation_json.clone(),
        ),
        (
            generated.join("texture-authoring.json"),
            texture_authoring_json.clone(),
        ),
        (generated.join("placeable-report.json"), report_json.clone()),
        (
            generated.join("binary-readback.json"),
            readback_json.clone(),
        ),
        (
            generated.join(&identity.hak_file_name),
            artifact.hak_payload.clone(),
        ),
        (
            generated.join(&identity.module_file_name),
            artifact.module_payload.clone(),
        ),
    ];
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        if read(path, "OUTPUT-READBACK")? != *expected {
            return Err(format!("MS1-OUTPUT-MISMATCH: {}", path.display()));
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
        "textureResrefs": expected_textures,
        "blueprintResref": identity.blueprint_resref,
        "objectTag": identity.object_tag,
        "appearanceRow": artifact.report.appearance_row.value,
        "placement": placement,
        "source": {
            "kind": "PROJECT_OWNED_SYNTHETIC_FIXTURE",
            "sha256": source_sha256,
            "byteLength": source.len(),
            "triangleCount": 2,
            "componentCount": 2,
            "meshyApiCalls": 0,
            "meshyCreditsSpent": 0,
        },
        "outputs": {
            "hakSha256": artifact.report.hak_sha256,
            "moduleSha256": artifact.report.module_sha256,
            "mdlSha256": artifact.report.mdl_sha256,
            "pwkSha256": artifact.report.pwk_sha256,
            "reportSha256": sha256(&report_json),
            "separationRecipeSha256": sha256(&separation_json),
            "textureAuthoringSha256": sha256(&texture_authoring_json),
            "binaryReadbackSha256": sha256(&readback_json),
        },
        "nativeInstallation": {
            "module": installed_mod,
            "hak": installed_hak,
        },
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "ownerProofRequired": true,
        "startsToolset": false,
        "startsNwn": false,
        "materializationCount": 1,
    });
    let handoff_json = pretty(&handoff, "HANDOFF")?;
    write_new(&output.join("ready-for-owner-proof.json"), &handoff_json)?;
    serde_json::to_string_pretty(&handoff).map_err(|error| format!("MS1-SUMMARY: {error}"))
}

fn collect_render_readback(
    node: &NodeReport,
    textures: &mut BTreeSet<String>,
    triangles: &mut usize,
) {
    if let Some(mesh) = node.mesh.as_ref().filter(|mesh| mesh.render != 0) {
        *triangles += mesh.faces.len();
        if let Some(texture) = mesh.textures.first().filter(|texture| !texture.is_empty()) {
            textures.insert(texture.clone());
        }
    }
    for child in &node.children {
        collect_render_readback(child, textures, triangles);
    }
}

fn base_placeables_2da() -> Vec<u8> {
    let columns = [
        "Label",
        "StrRef",
        "ModelName",
        "LightColor",
        "LightOffsetX",
        "LightOffsetY",
        "LightOffsetZ",
        "SoundAppType",
        "ShadowSize",
        "BodyBag",
        "LowGore",
        "Reflection",
        "Static",
    ];
    let row = |label: &str, model: &str| {
        [
            label, "****", model, "****", "****", "****", "****", "****", "1", "0", "****", "****",
            "1",
        ]
        .join(" ")
    };
    format!(
        "2DA V2.0\n\n{}\n0 {}\n1 {}\n2 {}\n",
        columns.join(" "),
        row("ARMOIRE", "plc_a01"),
        row("OS_RESERVED", "****"),
        row("ACTIVE_AFTER_RESERVED", "plc_b08"),
    )
    .into_bytes()
}

fn install_exact(source: &Path, destination: &Path, label: &str) -> Result<Value, String> {
    let source =
        fs::canonicalize(source).map_err(|error| format!("MS1-{label}-SOURCE-RESOLVE: {error}"))?;
    let source_bytes = read(&source, &format!("{label}-INSTALL-SOURCE"))?;
    let source_sha256 = sha256(&source_bytes);
    let destination_parent = destination
        .parent()
        .ok_or_else(|| format!("MS1-{label}-DESTINATION-PARENT"))?;
    let destination_parent = fs::canonicalize(destination_parent)
        .map_err(|error| format!("MS1-{label}-DESTINATION-RESOLVE: {error}"))?;
    let destination = destination_parent.join(
        destination
            .file_name()
            .ok_or_else(|| format!("MS1-{label}-DESTINATION-NAME"))?,
    );
    let reused = if destination.exists() {
        let existing = read(&destination, &format!("{label}-INSTALL-EXISTING"))?;
        if existing != source_bytes {
            return Err(format!(
                "MS1-{label}-NATIVE-COLLISION: {} existingSha256={} sourceSha256={source_sha256}",
                destination.display(),
                sha256(&existing),
            ));
        }
        true
    } else {
        write_new(&destination, &source_bytes)?;
        false
    };
    let installed = read(&destination, &format!("{label}-INSTALL-READBACK"))?;
    if installed != source_bytes {
        return Err(format!("MS1-{label}-INSTALL-HASH-MISMATCH"));
    }
    Ok(json!({
        "sourcePath": source,
        "destinationPath": destination,
        "byteLength": source_bytes.len(),
        "sha256": source_sha256,
        "reusedIdenticalExisting": reused,
        "byteIdentical": true,
    }))
}

fn exact_error(label: &str, error: &impl serde::Serialize) -> String {
    serde_json::to_string(error).unwrap_or_else(|_| format!("{label}: serialization failed"))
}

fn pretty(value: &impl serde::Serialize, label: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value).map_err(|error| format!("MS1-{label}-SERIALIZE: {error}"))
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("MS1-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("MS1-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("MS1-WRITE {}: {error}", path.display()))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
