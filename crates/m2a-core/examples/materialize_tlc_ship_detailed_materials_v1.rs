use std::{
    collections::BTreeSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    mdl::{NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, NodeReport, inspect_binary_mdl},
    model_material_separation::{
        ModelMaterialSeparationDocumentV2, model_material_separation_hash_v2,
    },
    model_material_uv_projection::{
        ModelMaterialUvProjectionDocumentV1, ModelMaterialUvProjectionModeV1,
        model_material_uv_projection_hash_v1,
    },
    model_texture_authoring::{
        ModelTextureAuthoringDocumentV1, ModelTexturePayloadDescriptorV1,
        model_texture_authoring_hash_v1,
    },
    placeable::{
        MDL_RESOURCE_TYPE, PWK_RESOURCE_TYPE, PlaceablePlacementV1, StaticPlaceableIdentityV1,
        TGA_RESOURCE_TYPE, build_meshy_static_placeable_package_v8,
        inspect_meshy_static_placeable_authoring_v3,
    },
    placeable_collision::inspect_ascii_placeable_walkmesh_v1,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ROOT: &str = r"C:\Projects\meshy2aurora";
const SOURCE_PATH: &str =
    r"C:\Projects\meshy2aurora\sample-3d\tlc-ship-under-construction-s1-p150k-v1\source.glb";
const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const SOURCE_TRIANGLES: usize = 152_574;
const SOURCE_VERTICES: u32 = 176_201;
const BASE_PLACEABLES_PATH: &str = r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-v1-20260725\generated\base-placeables.2da";
const BASE_PLACEABLES_SHA256: &str =
    "b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90";
const RECIPE_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\recipe-detailed-v1\material-separation-v2.json";
const RECIPE_FILE_SHA256: &str = "82c6babab76cff8ee1f4f2db4d45553f4f95c2dd0a539e99de26a2c529ee004a";
const SEPARATION_SHA256: &str = "d606f15bf42d2d4d3a8f0ce1296fdb59d811b63f29064e9b2aa3787afff80fa0";
const CANDIDATE_INPUT: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\candidate-detailed-materials-v1";
const TEXTURE_AUTHORING_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\prepared-detailed-material-textures-v2\texture-authoring.json";
const TEXTURE_AUTHORING_FILE_SHA256: &str =
    "1a62bd4f253608bfeff5f453bc28512b8afc02bcdbe56db02ce4f3a00018d202";
const TEXTURE_AUTHORING_SHA256: &str =
    "4c2e58d0364f3cb8b158e9a4934fa6abf38ae6a21cb433e656ab41434434f1ee";
const TEXTURE_DESCRIPTORS_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\prepared-detailed-material-textures-v2\texture-payload-descriptors.json";
const TEXTURE_DESCRIPTORS_SHA256: &str =
    "4b9b71c26eaf155adc67cfb26c9d0b3f0e63253128a55b90d5ab726a54d22d1d";
const TEXTURE_PAYLOAD_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\prepared-detailed-material-textures-v2\texture-payload.bin";
const TEXTURE_PAYLOAD_SHA256: &str =
    "c2f380353dee100ace3c537e700dde6c9c1dd5780c83085ff085eb9b1813d532";
const PRIOR_MODULE_FILE: &str = "m2a_tlcsm3_mod.mod";
const PRIOR_MODULE_SHA256: &str =
    "ce3872e6d6bf24496ebdc44ec225a11356f200d3f89a8d1caa740cf88405ba7e";
const PRIOR_MODULE_PATH: &str = r"C:\Projects\meshy2aurora\proof-output\tlc-ship-world-scale-uv-v1-20260804\generated\m2a_tlcsm3_mod.mod";
const PRIOR_HAK_FILE: &str = "m2a_tlcsm3_hak.hak";
const PRIOR_HAK_SHA256: &str = "9a04b028e605cd2cfb79f38bd0e08a565cc1fc3da362b5c18186df1292e6f340";
const PRIOR_HAK_PATH: &str = r"C:\Projects\meshy2aurora\proof-output\tlc-ship-world-scale-uv-v1-20260804\generated\m2a_tlcsm3_hak.hak";
const PRIOR_PWK_SHA256: &str = "fd4dbaea322a8feb30ba023c89b0fc0a81770303b337a67eb0e957e90210a9bd";
const PRIOR_PWK_PATH: &str = r"C:\Projects\meshy2aurora\proof-output\tlc-ship-world-scale-uv-v1-20260804\generated\m2a_tlcsm3_mdl.pwk";
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
    let mut args = env::args_os().skip(1);
    let output = PathBuf::from(args.next().ok_or_else(usage)?);
    let native_module_directory = PathBuf::from(args.next().ok_or_else(usage)?);
    let native_hak_directory = PathBuf::from(args.next().ok_or_else(usage)?);
    if args.next().is_some() {
        return Err(usage());
    }
    let workspace = fs::canonicalize(".").map_err(|error| error.to_string())?;
    let workspace_text = workspace.to_string_lossy();
    if !workspace_text
        .trim_start_matches(r"\\?\")
        .eq_ignore_ascii_case(ROOT)
    {
        return Err("TLC-DM1-NONCANONICAL-WORKSPACE".to_owned());
    }
    if output.exists() {
        return Err(format!("TLC-DM1-OUTPUT-EXISTS: {}", output.display()));
    }
    let native_module_directory = canonical_directory(&native_module_directory, "MOD-DIR")?;
    let native_hak_directory = canonical_directory(&native_hak_directory, "HAK-DIR")?;

    let admission_path = Path::new(CANDIDATE_INPUT).join("admission.json");
    let identity_path = Path::new(CANDIDATE_INPUT).join("identity.json");
    let projection_path = Path::new(CANDIDATE_INPUT).join("material-uv-projection.json");
    let admission_bytes = read(&admission_path, "ADMISSION")?;
    let admission: Value = serde_json::from_slice(&admission_bytes)
        .map_err(|error| format!("TLC-DM1-ADMISSION-PARSE: {error}"))?;
    validate_admission(&admission)?;
    let identity_bytes = read(&identity_path, "IDENTITY")?;
    let identity: StaticPlaceableIdentityV1 = serde_json::from_slice(&identity_bytes)
        .map_err(|error| format!("TLC-DM1-IDENTITY-PARSE: {error}"))?;
    validate_identity(&identity)?;
    let native_module_path = native_module_directory.join(&identity.module_file_name);
    let native_hak_path = native_hak_directory.join(&identity.hak_file_name);
    if native_module_path.exists() || native_hak_path.exists() {
        return Err("TLC-DM1-NATIVE-TARGET-COLLISION".to_owned());
    }
    read_exact(
        Path::new(PRIOR_MODULE_PATH),
        PRIOR_MODULE_SHA256,
        "PRIOR-MOD",
    )?;
    read_exact(Path::new(PRIOR_HAK_PATH), PRIOR_HAK_SHA256, "PRIOR-HAK")?;

    let source = read_exact(Path::new(SOURCE_PATH), SOURCE_SHA256, "SOURCE")?;
    let base_placeables = read_exact(
        Path::new(BASE_PLACEABLES_PATH),
        BASE_PLACEABLES_SHA256,
        "BASE-PLACEABLES",
    )?;
    let recipe_bytes = read_exact(Path::new(RECIPE_PATH), RECIPE_FILE_SHA256, "SEPARATION")?;
    let separation: ModelMaterialSeparationDocumentV2 = serde_json::from_slice(&recipe_bytes)
        .map_err(|error| format!("TLC-DM1-SEPARATION-PARSE: {error}"))?;
    if model_material_separation_hash_v2(&separation)
        .map_err(|error| exact_error("SEPARATION", &error))?
        != SEPARATION_SHA256
    {
        return Err("TLC-DM1-SEPARATION-HASH-MISMATCH".to_owned());
    }

    let projection_bytes = read(&projection_path, "PROJECTION")?;
    let projection: ModelMaterialUvProjectionDocumentV1 = serde_json::from_slice(&projection_bytes)
        .map_err(|error| format!("TLC-DM1-PROJECTION-PARSE: {error}"))?;
    validate_projection(&projection)?;
    let projection_sha256 = model_material_uv_projection_hash_v1(&projection)
        .map_err(|error| exact_error("PROJECTION", &error))?;

    let texture_authoring_bytes = read_exact(
        Path::new(TEXTURE_AUTHORING_PATH),
        TEXTURE_AUTHORING_FILE_SHA256,
        "TEXTURE-AUTHORING",
    )?;
    let texture_authoring: ModelTextureAuthoringDocumentV1 =
        serde_json::from_slice(&texture_authoring_bytes)
            .map_err(|error| format!("TLC-DM1-TEXTURE-AUTHORING-PARSE: {error}"))?;
    if model_texture_authoring_hash_v1(&texture_authoring)
        .map_err(|error| exact_error("TEXTURE-AUTHORING", &error))?
        != TEXTURE_AUTHORING_SHA256
    {
        return Err("TLC-DM1-TEXTURE-AUTHORING-HASH-MISMATCH".to_owned());
    }
    let descriptor_bytes = read_exact(
        Path::new(TEXTURE_DESCRIPTORS_PATH),
        TEXTURE_DESCRIPTORS_SHA256,
        "TEXTURE-DESCRIPTORS",
    )?;
    let descriptors: Vec<ModelTexturePayloadDescriptorV1> =
        serde_json::from_slice(&descriptor_bytes)
            .map_err(|error| format!("TLC-DM1-TEXTURE-DESCRIPTORS-PARSE: {error}"))?;
    let texture_payload = read_exact(
        Path::new(TEXTURE_PAYLOAD_PATH),
        TEXTURE_PAYLOAD_SHA256,
        "TEXTURE-PAYLOAD",
    )?;

    let mut bootstrap = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .map_err(|error| exact_error("AUTHORING", &error))?;
    if bootstrap.document.elements.len() != 1 {
        return Err("TLC-DM1-SOURCE-ELEMENT-COUNT".to_owned());
    }
    let source_min_z = bootstrap
        .inspection
        .nodes
        .iter()
        .flat_map(|node| node.primitives.iter())
        .flat_map(|primitive| primitive.components.iter())
        .map(|component| component.bounds_min[2])
        .fold(f32::INFINITY, f32::min);
    if !source_min_z.is_finite() {
        return Err("TLC-DM1-SOURCE-BOUNDS".to_owned());
    }
    bootstrap.document.elements[0].transform.scale = [SHIP_SCALE; 3];
    bootstrap.document.elements[0].transform.translation = [0.0, 0.0, -source_min_z * SHIP_SCALE];
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let build = || {
        build_meshy_static_placeable_package_v8(
            &source,
            &base_placeables,
            &identity,
            placement,
            7,
            &bootstrap.document,
            &separation,
            &projection,
            &texture_authoring,
            &texture_payload,
            &descriptors,
            &Default::default(),
        )
        .map_err(|error| exact_error("BUILD", &error))
    };
    let artifact = build()?;
    let repeated = build()?;
    if artifact.hak_payload != repeated.hak_payload
        || artifact.module_payload != repeated.module_payload
        || serde_json::to_vec(&artifact.report).ok() != serde_json::to_vec(&repeated.report).ok()
    {
        return Err("TLC-DM1-NONDETERMINISTIC".to_owned());
    }
    validate_report(&artifact.report)?;

    let authoring_report = artifact
        .report
        .authoring
        .as_ref()
        .ok_or_else(|| "TLC-DM1-AUTHORING-REPORT-MISSING".to_owned())?;
    let projection_report = artifact
        .report
        .material_uv_projection
        .as_ref()
        .ok_or_else(|| "TLC-DM1-PROJECTION-REPORT-MISSING".to_owned())?;
    let texture_report = artifact
        .report
        .model_texture_authoring
        .as_ref()
        .ok_or_else(|| "TLC-DM1-TEXTURE-REPORT-MISSING".to_owned())?;
    let dimensions = [
        authoring_report.bounds_max[0] - authoring_report.bounds_min[0],
        authoring_report.bounds_max[1] - authoring_report.bounds_min[1],
        authoring_report.bounds_max[2] - authoring_report.bounds_min[2],
    ];
    if dimensions[0] < 12.0
        || dimensions[1] < 6.0
        || dimensions[2] < 5.0
        || authoring_report.bounds_min[2].abs() > 0.001
    {
        return Err(format!("TLC-DM1-SCALE-OR-GROUNDING: {dimensions:?}"));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("TLC-DM1-HAK-READBACK: {error}"))?;
    let mdl = hak
        .find(&identity.model_resref, MDL_RESOURCE_TYPE)
        .map_err(|error| format!("TLC-DM1-MDL-FIND: {error}"))?;
    let mdl_inspection =
        inspect_binary_mdl(mdl).map_err(|error| format!("TLC-DM1-MDL-READBACK: {error}"))?;
    let mut mdl_texture_resrefs = BTreeSet::new();
    let mut stream_triangles = Vec::new();
    for root in &mdl_inspection.node_tree.roots {
        collect_mdl_readback(root, &mut mdl_texture_resrefs, &mut stream_triangles);
    }
    let expected_texture_resrefs = texture_report
        .resources
        .iter()
        .map(|resource| resource.resref.clone())
        .collect::<BTreeSet<_>>();
    if stream_triangles.iter().sum::<usize>() != SOURCE_TRIANGLES
        || stream_triangles
            .iter()
            .any(|count| *count > NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1)
        || mdl_texture_resrefs != expected_texture_resrefs
    {
        return Err("TLC-DM1-MDL-INVARIANT".to_owned());
    }
    let pwk = hak
        .find(&identity.model_resref, PWK_RESOURCE_TYPE)
        .map_err(|error| format!("TLC-DM1-PWK-FIND: {error}"))?;
    if sha256(pwk) != artifact.report.pwk_sha256 {
        return Err("TLC-DM1-PWK-REPORT-HASH-MISMATCH".to_owned());
    }
    let prior_pwk = read_exact(Path::new(PRIOR_PWK_PATH), PRIOR_PWK_SHA256, "PRIOR-PWK")?;
    validate_collision_semantics(&prior_pwk, pwk)?;

    let generated = output.join("generated");
    fs::create_dir_all(&generated).map_err(|error| format!("TLC-DM1-OUTPUT-CREATE: {error}"))?;
    let report_bytes = pretty(&artifact.report, "REPORT")?;
    let authoring_bytes = pretty(&bootstrap.document, "AUTHORING")?;
    for (path, payload) in [
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
            mdl.to_vec(),
        ),
        (
            generated.join(format!("{}.pwk", identity.model_resref)),
            pwk.to_vec(),
        ),
        (generated.join("admission.json"), admission_bytes.clone()),
        (generated.join("identity.json"), identity_bytes.clone()),
        (
            generated.join("material-separation-v2.json"),
            recipe_bytes.clone(),
        ),
        (
            generated.join("material-uv-projection.json"),
            projection_bytes.clone(),
        ),
        (
            generated.join("texture-authoring.json"),
            texture_authoring_bytes.clone(),
        ),
        (
            generated.join("texture-payload-descriptors.json"),
            descriptor_bytes.clone(),
        ),
        (generated.join("authoring.json"), authoring_bytes),
        (generated.join("placeable-report.json"), report_bytes),
    ] {
        write_new(&path, &payload)?;
    }
    for resource in &texture_report.resources {
        let payload = hak
            .find(&resource.resref, TGA_RESOURCE_TYPE)
            .map_err(|error| format!("TLC-DM1-TGA-FIND {}: {error}", resource.resref))?;
        if sha256(payload) != resource.sha256 {
            return Err(format!("TLC-DM1-TGA-HASH: {}", resource.resref));
        }
        write_new(&generated.join(format!("{}.tga", resource.resref)), payload)?;
    }

    let native_module = install_exact(
        &generated.join(&identity.module_file_name),
        &native_module_path,
        "MOD",
    )?;
    let native_hak = install_exact(
        &generated.join(&identity.hak_file_name),
        &native_hak_path,
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
        "modelResref": identity.model_resref,
        "textureResrefs": expected_texture_resrefs,
        "blueprintResref": identity.blueprint_resref,
        "objectTag": identity.object_tag,
        "appearanceRow": artifact.report.appearance_row.value,
        "placement": placement,
        "uniformScale": SHIP_SCALE,
        "dimensionsMeters": dimensions,
        "sourceSha256": SOURCE_SHA256,
        "sourceTriangleCount": SOURCE_TRIANGLES,
        "outputTriangleCount": stream_triangles.iter().sum::<usize>(),
        "sourceVertexCount": SOURCE_VERTICES,
        "outputVertexCount": projection_report.output_vertex_count,
        "geometryCleanup": false,
        "separationSha256": SEPARATION_SHA256,
        "materialUvProjectionSha256": projection_sha256,
        "textureAuthoringSha256": TEXTURE_AUTHORING_SHA256,
        "textureUvPolicy": texture_report.uv_policy,
        "textureWarnings": texture_report.warnings,
        "textureQuality": texture_report.bindings.iter().map(|binding| json!({
            "authoredMaterialId": binding.authored_material_id,
            "status": binding.mip_readability.status,
            "baseLumaStddevMilli": binding.mip_readability.base_luma_stddev_milli,
            "mip16LumaStddevMilli": binding.mip_readability.mip_16_luma_stddev_milli,
            "contrastRetentionBasisPoints": binding.mip_readability.contrast_retention_basis_points,
            "sourceDoubleSided": binding.source_double_sided,
            "targetDoubleSidedPolicy": binding.target_double_sided_policy,
        })).collect::<Vec<_>>(),
        "pwkSha256": artifact.report.pwk_sha256,
        "mdlSha256": artifact.report.mdl_sha256,
        "hakSha256": artifact.report.hak_sha256,
        "moduleSha256": artifact.report.module_sha256,
        "nativeModule": native_module,
        "nativeHak": native_hak,
        "admission": admission,
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "ownerProofRequired": true,
        "startsToolset": false,
        "startsNwn": false
    });
    let handoff_bytes = pretty(&handoff, "HANDOFF")?;
    write_new(&output.join("ready-for-owner-proof.json"), &handoff_bytes)?;
    serde_json::to_string_pretty(&handoff).map_err(|error| error.to_string())
}

fn validate_admission(value: &Value) -> Result<(), String> {
    let prior = value
        .get("priorCandidate")
        .ok_or_else(|| "TLC-DM1-ADMISSION-PRIOR-MISSING".to_owned())?;
    let exception = value
        .get("materialOnlyIterationException")
        .ok_or_else(|| "TLC-DM1-ADMISSION-EXCEPTION-MISSING".to_owned())?;
    if prior.get("moduleFileName").and_then(Value::as_str) != Some(PRIOR_MODULE_FILE)
        || prior.get("moduleSha256").and_then(Value::as_str) != Some(PRIOR_MODULE_SHA256)
        || prior.get("hakFileName").and_then(Value::as_str) != Some(PRIOR_HAK_FILE)
        || prior.get("hakSha256").and_then(Value::as_str) != Some(PRIOR_HAK_SHA256)
        || value
            .pointer("/toolset/modelVisibility")
            .and_then(Value::as_str)
            != Some("visible")
        || value
            .pointer("/toolset/proofCompleteness")
            .and_then(Value::as_str)
            != Some("verified")
        || value
            .pointer("/toolset/visualAcceptance")
            .and_then(Value::as_str)
            != Some("rejected")
        || exception.get("authorized").and_then(Value::as_bool) != Some(true)
        || exception
            .get("geometryChangeAuthorized")
            .and_then(Value::as_bool)
            != Some(false)
        || exception
            .get("ownerStatement")
            .and_then(Value::as_str)
            .is_none_or(|statement| statement.trim().is_empty())
    {
        return Err("TLC-DM1-ADMISSION-INVALID".to_owned());
    }
    Ok(())
}

fn validate_identity(identity: &StaticPlaceableIdentityV1) -> Result<(), String> {
    if identity.module_resref != "m2a_tlcdm4_mod"
        || identity.module_file_name != "m2a_tlcdm4_mod.mod"
        || identity.area_resref != "m2a_tlcdm4_ar"
        || identity.hak_resref != "m2a_tlcdm4_hak"
        || identity.hak_file_name != "m2a_tlcdm4_hak.hak"
        || identity.model_resref != "m2a_tlcdm4_mdl"
        || identity.texture_resref != "m2a_tlcdm4_tex"
        || identity.blueprint_resref != "m2a_tlcdm4_utp"
        || identity.object_tag != "m2a_tlcdm4_ship"
    {
        return Err("TLC-DM1-IDENTITY-INVALID".to_owned());
    }
    Ok(())
}

fn validate_projection(document: &ModelMaterialUvProjectionDocumentV1) -> Result<(), String> {
    let expected = [
        ("wood_bow", 0.45f32),
        ("wood_deck", 0.55f32),
        ("wood_frame", 0.75f32),
        ("wood_hull", 0.42f32),
        ("wood_masts", 0.28f32),
        ("wood_rails", 0.65f32),
        ("wood_stern", 0.42f32),
        ("wood_supports", 0.70f32),
    ];
    if document.source_sha256 != SOURCE_SHA256
        || document.separation_sha256 != SEPARATION_SHA256
        || document.rules.len() != expected.len()
    {
        return Err("TLC-DM1-PROJECTION-INVALID".to_owned());
    }
    for (rule, (material, repeats)) in document.rules.iter().zip(expected) {
        if rule.authored_material_id != material
            || rule.mode != ModelMaterialUvProjectionModeV1::MaterialBoxWorld
            || rule.u_repeats != repeats
            || rule.v_min != 0.0
            || rule.v_max != 1.0
            || rule.deterministic_u_phase
        {
            return Err("TLC-DM1-PROJECTION-INVALID".to_owned());
        }
    }
    Ok(())
}

fn validate_report(
    report: &m2a_core::placeable::StaticPlaceablePackageReportV1,
) -> Result<(), String> {
    let authoring = report
        .authoring
        .as_ref()
        .ok_or_else(|| "TLC-DM1-AUTHORING-REPORT-MISSING".to_owned())?;
    let separation = report
        .material_separation
        .as_ref()
        .ok_or_else(|| "TLC-DM1-SEPARATION-REPORT-MISSING".to_owned())?;
    let projection = report
        .material_uv_projection
        .as_ref()
        .ok_or_else(|| "TLC-DM1-PROJECTION-REPORT-MISSING".to_owned())?;
    let textures = report
        .model_texture_authoring
        .as_ref()
        .ok_or_else(|| "TLC-DM1-TEXTURE-REPORT-MISSING".to_owned())?;
    if report.experimental_aggressive_geometry_cleanup
        || authoring.output_triangle_count as usize != SOURCE_TRIANGLES
        || separation.source_triangle_count as usize != SOURCE_TRIANGLES
        || separation.output_triangle_count as usize != SOURCE_TRIANGLES
        || separation.output_vertex_count != SOURCE_VERTICES
        || separation.duplicated_boundary_vertex_count != 0
        || projection.source_triangle_count as usize != SOURCE_TRIANGLES
        || projection.output_triangle_count as usize != SOURCE_TRIANGLES
        || projection.projected_triangle_count != 141_183
        || projection.projected_material_ids
            != [
                "wood_bow",
                "wood_deck",
                "wood_frame",
                "wood_hull",
                "wood_masts",
                "wood_rails",
                "wood_stern",
                "wood_supports",
            ]
        || projection.geometry_cleanup
        || textures.resources.len() != 7
        || textures.bindings.len() != 12
        || textures.uv_policy != "MATERIAL_UV_PROJECTION_V1"
    {
        return Err("TLC-DM1-REPORT-INVARIANT".to_owned());
    }
    Ok(())
}

fn collect_mdl_readback(
    node: &NodeReport,
    textures: &mut BTreeSet<String>,
    triangles: &mut Vec<usize>,
) {
    if let Some(mesh) = node.mesh.as_ref().filter(|mesh| mesh.render != 0) {
        triangles.push(mesh.faces.len());
        if let Some(texture) = mesh.textures.first().filter(|value| !value.is_empty()) {
            textures.insert(texture.clone());
        }
    }
    for child in &node.children {
        collect_mdl_readback(child, textures, triangles);
    }
}

fn validate_collision_semantics(prior: &[u8], candidate: &[u8]) -> Result<(), String> {
    let prior = inspect_ascii_placeable_walkmesh_v1(prior)
        .map_err(|error| exact_error("PRIOR-PWK-READBACK", &error))?;
    let candidate = inspect_ascii_placeable_walkmesh_v1(candidate)
        .map_err(|error| exact_error("PWK-READBACK", &error))?;
    if prior.mesh_nodes.len() != candidate.mesh_nodes.len()
        || prior.use_points.len() != candidate.use_points.len()
    {
        return Err("TLC-DM1-PWK-SEMANTIC-COUNT-CHANGED".to_owned());
    }
    for (prior, candidate) in prior.mesh_nodes.iter().zip(&candidate.mesh_nodes) {
        if prior.position != candidate.position
            || prior.orientation != candidate.orientation
            || prior.vertices != candidate.vertices
            || prior.faces != candidate.faces
        {
            return Err("TLC-DM1-PWK-SEMANTICS-CHANGED".to_owned());
        }
    }
    for (prior, candidate) in prior.use_points.iter().zip(&candidate.use_points) {
        if prior.position != candidate.position || prior.orientation != candidate.orientation {
            return Err("TLC-DM1-PWK-USE-POINTS-CHANGED".to_owned());
        }
    }
    Ok(())
}

fn install_exact(source: &Path, destination: &Path, label: &str) -> Result<Value, String> {
    if destination.exists() {
        return Err(format!("TLC-DM1-{label}-DESTINATION-EXISTS"));
    }
    let source = fs::canonicalize(source)
        .map_err(|error| format!("TLC-DM1-{label}-SOURCE-RESOLVE: {error}"))?;
    let payload = read(&source, &format!("{label}-SOURCE"))?;
    write_new(destination, &payload)?;
    let installed = read(destination, &format!("{label}-READBACK"))?;
    if installed != payload {
        return Err(format!("TLC-DM1-{label}-INSTALL-MISMATCH"));
    }
    Ok(json!({
        "sourcePath": source,
        "destinationPath": destination,
        "byteLength": payload.len(),
        "sha256": sha256(&payload),
        "reusedIdenticalExisting": false,
        "byteIdentical": true
    }))
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let resolved =
        fs::canonicalize(path).map_err(|error| format!("TLC-DM1-{label}-RESOLVE: {error}"))?;
    resolved
        .is_dir()
        .then_some(resolved)
        .ok_or_else(|| format!("TLC-DM1-{label}-NOT-DIRECTORY"))
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("TLC-DM1-{label}-READ {}: {error}", path.display()))
}

fn read_exact(path: &Path, expected: &str, label: &str) -> Result<Vec<u8>, String> {
    let payload = read(path, label)?;
    let actual = sha256(&payload);
    if actual != expected {
        return Err(format!(
            "TLC-DM1-{label}-HASH: expected {expected}, got {actual}"
        ));
    }
    Ok(payload)
}

fn write_new(path: &Path, payload: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("TLC-DM1-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(payload)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("TLC-DM1-WRITE {}: {error}", path.display()))
}

fn pretty(value: &impl Serialize, label: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value).map_err(|error| format!("TLC-DM1-{label}: {error}"))
}

fn exact_error(label: &str, error: &impl Serialize) -> String {
    serde_json::to_string(error).unwrap_or_else(|_| format!("TLC-DM1-{label}"))
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn usage() -> String {
    "usage: materialize_tlc_ship_detailed_materials_v1 <output-directory> <native-module-directory> <native-hak-directory>".to_owned()
}
