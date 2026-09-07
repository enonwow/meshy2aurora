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
        ModelMaterialUvProjectionDocumentV1, model_material_uv_projection_hash_v1,
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
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SOURCE_PATH: &str =
    r"C:\Projects\meshy2aurora\sample-3d\tlc-ship-under-construction-s1-p150k-v1\source.glb";
const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const SOURCE_TRIANGLES: usize = 152_574;
const SOURCE_VERTICES: u32 = 176_201;
const BASE_PLACEABLES_PATH: &str = r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-v1-20260725\generated\base-placeables.2da";
const BASE_PLACEABLES_SHA256: &str =
    "b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90";
const RECIPE_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\recipe-correct-gltf-uv\material-separation-v2.json";
const RECIPE_FILE_SHA256: &str = "fa34001f10a8670dea9bd9b1a6d644ea9fa7a32bc0effd2d13ac59eb9b042fd0";
const SEPARATION_SHA256: &str = "6eeb7877b24feb3f4a315a9ae68809dc5620ca4272bfb7702d3d159f67df4b3a";
const MATERIAL_UV_PROJECTION_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\material-uv-projection-v3\material-uv-projection.json";
const MATERIAL_UV_PROJECTION_FILE_SHA256: &str =
    "20194ceff818e0a7728ffa8c4da4f4fcb70ccc7697bddaf2f9419d61f6877754";
const MATERIAL_UV_PROJECTION_SHA256: &str =
    "4cc440b988709537ff2404556212c184aec54311a0d15271f00af6af6e1a84d2";
const SOURCE_ATLAS_VARIANT_REPORT_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\material-textures-v2-component-uv\source-atlas-variant-report.json";
const SOURCE_ATLAS_VARIANT_REPORT_SHA256: &str =
    "d78dc2360b9c04edee08e9beca7075d5e1fbc25069e07d10bd6ce208f7621cf4";
const TEXTURE_AUTHORING_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\prepared-material-textures-v2-component-uv\texture-authoring.json";
const TEXTURE_AUTHORING_FILE_SHA256: &str =
    "b85f6845c5354049bc61b975ddf0f7590e67e1a191f38dd8e3b5e374de341f43";
const TEXTURE_AUTHORING_SHA256: &str =
    "560ccbd0f3ca7f34a2df1a2b1f50b3f3bc6c9cf2934680c6da8cf850f90a7fb1";
const TEXTURE_DESCRIPTORS_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\prepared-material-textures-v2-component-uv\texture-payload-descriptors.json";
const TEXTURE_DESCRIPTORS_SHA256: &str =
    "6c2092ef83a9e621845582e081dfdf5e5fcf9e7a95c13cd6dab343032a7df245";
const TEXTURE_PAYLOAD_PATH: &str = r"C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\prepared-material-textures-v2-component-uv\texture-payload.bin";
const TEXTURE_PAYLOAD_SHA256: &str =
    "d89843d6817301e9d1716444f57490f2e47e1076624626396b14958a762f2880";
const PRIOR_MODULE_FILE_NAME: &str = "m2a_tlcs1_mod.mod";
const PRIOR_MODULE_SHA256: &str =
    "e02f71e5ba8b5f92ccf76486450ab4bdc78e8b019893fd364d1c109959eb9e83";
const PRIOR_HAK_FILE_NAME: &str = "m2a_tlcs1_hak.hak";
const PRIOR_HAK_SHA256: &str = "80ebb8bca45a671754bc39a6db690da0251963b128efbb25efbbf07cc2f8ff12";
const PRIOR_MODULE_RESREF: &str = "m2a_tlcs1_mod";
const PRIOR_AREA_RESREF: &str = "m2a_tlcs1_ar";
const PRIOR_HAK_RESREF: &str = "m2a_tlcs1_hak";
const PRIOR_MODEL_RESREF: &str = "m2a_tlcs1_mdl";
const PRIOR_TEXTURE_RESREF: &str = "m2a_tlcs1_tex";
const PRIOR_BLUEPRINT_RESREF: &str = "m2a_tlcs1_utp";
const PRIOR_OBJECT_TAG: &str = "m2a_tlcs1_building_ship";
const SHIP_SCALE: f32 = 8.0;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProofAxisV1 {
    model_visibility: String,
    proof_completeness: String,
    owner_observation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PriorCandidateV1 {
    module_file_name: String,
    module_sha256: String,
    hak_file_name: String,
    hak_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MaterialOnlyExceptionV1 {
    authorized: bool,
    owner_statement: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MaterialIterationAdmissionV1 {
    schema_version: u32,
    recorded_at: String,
    prior_candidate: PriorCandidateV1,
    toolset: ProofAxisV1,
    nwn: ProofAxisV1,
    material_only_iteration_exception: Option<MaterialOnlyExceptionV1>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeInstallationV1 {
    source_path: PathBuf,
    destination_path: PathBuf,
    byte_length: usize,
    sha256: String,
    reused_identical_existing: bool,
    byte_identical: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HandoffV1 {
    test_module_file_name: String,
    toolset_module_name: String,
    area_name: String,
    schema_version: u32,
    status: String,
    area_resref: String,
    ordered_hak_files: Vec<String>,
    model_resref: String,
    texture_resrefs: Vec<String>,
    blueprint_resref: String,
    object_tag: String,
    appearance_row: u32,
    placement: PlaceablePlacementV1,
    uniform_scale: f32,
    dimensions_meters: [f32; 3],
    source_sha256: String,
    separation_sha256: String,
    material_uv_projection_sha256: String,
    texture_authoring_sha256: String,
    source_triangle_count: usize,
    output_triangle_count: usize,
    source_vertex_count: u32,
    output_vertex_count: u32,
    geometry_cleanup: bool,
    pwk_sha256: String,
    mdl_sha256: String,
    hak_sha256: String,
    module_sha256: String,
    native_module: NativeInstallationV1,
    native_hak: NativeInstallationV1,
    admission: MaterialIterationAdmissionV1,
    model_visibility: String,
    proof_completeness: String,
    owner_proof_required: bool,
    starts_toolset: bool,
    starts_nwn: bool,
}

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
    let admission_path = PathBuf::from(args.next().ok_or_else(usage)?);
    let identity_path = PathBuf::from(args.next().ok_or_else(usage)?);
    let output = PathBuf::from(args.next().ok_or_else(usage)?);
    let native_module_directory = PathBuf::from(args.next().ok_or_else(usage)?);
    let native_hak_directory = PathBuf::from(args.next().ok_or_else(usage)?);
    if args.next().is_some() {
        return Err(usage());
    }
    if output.exists() {
        return Err(format!("TLC-MS2-OUTPUT-EXISTS: {}", output.display()));
    }

    let admission_bytes = read(&admission_path, "ADMISSION")?;
    let admission: MaterialIterationAdmissionV1 = serde_json::from_slice(&admission_bytes)
        .map_err(|error| {
            format!(
                "TLC-MS2-ADMISSION-PARSE {}: {error}",
                admission_path.display()
            )
        })?;
    validate_admission(&admission)?;
    let identity_bytes = read(&identity_path, "IDENTITY")?;
    let identity: StaticPlaceableIdentityV1 =
        serde_json::from_slice(&identity_bytes).map_err(|error| {
            format!(
                "TLC-MS2-IDENTITY-PARSE {}: {error}",
                identity_path.display()
            )
        })?;
    validate_new_identity(&identity)?;
    let native_module_directory = canonical_directory(&native_module_directory, "NATIVE-MOD-DIR")?;
    let native_hak_directory = canonical_directory(&native_hak_directory, "NATIVE-HAK-DIR")?;
    verify_prior_native_candidate(&native_module_directory, &native_hak_directory)?;

    let source = read_exact(Path::new(SOURCE_PATH), SOURCE_SHA256, "SOURCE")?;
    let base_placeables = read_exact(
        Path::new(BASE_PLACEABLES_PATH),
        BASE_PLACEABLES_SHA256,
        "BASE-PLACEABLES",
    )?;
    let recipe_bytes = read_exact(
        Path::new(RECIPE_PATH),
        RECIPE_FILE_SHA256,
        "MATERIAL-SEPARATION",
    )?;
    let material_separation: ModelMaterialSeparationDocumentV2 =
        serde_json::from_slice(&recipe_bytes)
            .map_err(|error| format!("TLC-MS2-MATERIAL-SEPARATION-PARSE: {error}"))?;
    if model_material_separation_hash_v2(&material_separation)
        .map_err(|error| exact_error("TLC-MS2-MATERIAL-SEPARATION", &error))?
        != SEPARATION_SHA256
    {
        return Err("TLC-MS2-SEPARATION-HASH-MISMATCH".to_owned());
    }
    let material_uv_projection_bytes = read_exact(
        Path::new(MATERIAL_UV_PROJECTION_PATH),
        MATERIAL_UV_PROJECTION_FILE_SHA256,
        "MATERIAL-UV-PROJECTION",
    )?;
    let material_uv_projection: ModelMaterialUvProjectionDocumentV1 =
        serde_json::from_slice(&material_uv_projection_bytes)
            .map_err(|error| format!("TLC-MS2-MATERIAL-UV-PROJECTION-PARSE: {error}"))?;
    if model_material_uv_projection_hash_v1(&material_uv_projection)
        .map_err(|error| exact_error("TLC-MS2-MATERIAL-UV-PROJECTION", &error))?
        != MATERIAL_UV_PROJECTION_SHA256
    {
        return Err("TLC-MS2-MATERIAL-UV-PROJECTION-HASH-MISMATCH".to_owned());
    }
    let source_atlas_variant_report = read_exact(
        Path::new(SOURCE_ATLAS_VARIANT_REPORT_PATH),
        SOURCE_ATLAS_VARIANT_REPORT_SHA256,
        "SOURCE-ATLAS-VARIANT-REPORT",
    )?;
    let texture_authoring_bytes = read_exact(
        Path::new(TEXTURE_AUTHORING_PATH),
        TEXTURE_AUTHORING_FILE_SHA256,
        "TEXTURE-AUTHORING",
    )?;
    let texture_authoring: ModelTextureAuthoringDocumentV1 =
        serde_json::from_slice(&texture_authoring_bytes)
            .map_err(|error| format!("TLC-MS2-TEXTURE-AUTHORING-PARSE: {error}"))?;
    if model_texture_authoring_hash_v1(&texture_authoring)
        .map_err(|error| exact_error("TLC-MS2-TEXTURE-AUTHORING", &error))?
        != TEXTURE_AUTHORING_SHA256
    {
        return Err("TLC-MS2-TEXTURE-AUTHORING-HASH-MISMATCH".to_owned());
    }
    let descriptor_bytes = read_exact(
        Path::new(TEXTURE_DESCRIPTORS_PATH),
        TEXTURE_DESCRIPTORS_SHA256,
        "TEXTURE-DESCRIPTORS",
    )?;
    let descriptors: Vec<ModelTexturePayloadDescriptorV1> =
        serde_json::from_slice(&descriptor_bytes)
            .map_err(|error| format!("TLC-MS2-TEXTURE-DESCRIPTORS-PARSE: {error}"))?;
    let texture_payload = read_exact(
        Path::new(TEXTURE_PAYLOAD_PATH),
        TEXTURE_PAYLOAD_SHA256,
        "TEXTURE-PAYLOAD",
    )?;

    let mut bootstrap = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .map_err(|error| exact_error("TLC-MS2-AUTHORING", &error))?;
    if bootstrap.document.elements.len() != 1 {
        return Err(format!(
            "TLC-MS2-SOURCE-ELEMENTS: expected 1, got {}",
            bootstrap.document.elements.len()
        ));
    }
    let sanitized_min_z = bootstrap
        .inspection
        .nodes
        .iter()
        .flat_map(|node| node.primitives.iter())
        .flat_map(|primitive| primitive.components.iter())
        .map(|component| component.bounds_min[2])
        .fold(f32::INFINITY, f32::min);
    if !sanitized_min_z.is_finite() {
        return Err("TLC-MS2-SOURCE-BOUNDS-NONFINITE".to_owned());
    }
    let element = &mut bootstrap.document.elements[0];
    element.transform.scale = [SHIP_SCALE; 3];
    element.transform.translation = [0.0, 0.0, -sanitized_min_z * SHIP_SCALE];
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
            &material_separation,
            &material_uv_projection,
            &texture_authoring,
            &texture_payload,
            &descriptors,
            &Default::default(),
        )
        .map_err(|error| exact_error("TLC-MS2-BUILD", &error))
    };
    let artifact = build()?;
    let repeated = build()?;
    if artifact.hak_payload != repeated.hak_payload
        || artifact.module_payload != repeated.module_payload
        || serde_json::to_vec(&artifact.report).ok() != serde_json::to_vec(&repeated.report).ok()
    {
        return Err("TLC-MS2-NONDETERMINISTIC-BUILD".to_owned());
    }
    if artifact.report.experimental_aggressive_geometry_cleanup {
        return Err("TLC-MS2-GEOMETRY-CLEANUP-MUST-BE-DISABLED".to_owned());
    }
    let authoring = artifact
        .report
        .authoring
        .as_ref()
        .ok_or_else(|| "TLC-MS2-AUTHORING-REPORT-MISSING".to_owned())?;
    let separation = artifact
        .report
        .material_separation
        .as_ref()
        .ok_or_else(|| "TLC-MS2-SEPARATION-REPORT-MISSING".to_owned())?;
    let uv_projection = artifact
        .report
        .material_uv_projection
        .as_ref()
        .ok_or_else(|| "TLC-MS2-MATERIAL-UV-PROJECTION-REPORT-MISSING".to_owned())?;
    let textures = artifact
        .report
        .model_texture_authoring
        .as_ref()
        .ok_or_else(|| "TLC-MS2-TEXTURE-REPORT-MISSING".to_owned())?;
    if authoring.output_triangle_count as usize != SOURCE_TRIANGLES
        || separation.source_triangle_count as usize != SOURCE_TRIANGLES
        || separation.output_triangle_count as usize != SOURCE_TRIANGLES
        || separation.output_vertex_count != SOURCE_VERTICES
        || separation.duplicated_boundary_vertex_count != 0
        || uv_projection.source_triangle_count as usize != SOURCE_TRIANGLES
        || uv_projection.output_triangle_count as usize != SOURCE_TRIANGLES
        || uv_projection.source_vertex_count < SOURCE_VERTICES
        || uv_projection.output_vertex_count <= uv_projection.source_vertex_count
        || uv_projection.projected_triangle_count != 141_183
        || uv_projection.projected_material_ids != ["wood"]
        || uv_projection.geometry_cleanup
        || textures.resources.len() != 5
    {
        return Err(format!(
            "TLC-MS2-GEOMETRY-OR-MATERIAL-INVARIANT: authoringTriangles={} authoringVertices={} separation={}/{} duplicated={} uv={}/{} projected={} uvVertices={}/{} textures={}",
            authoring.output_triangle_count,
            uv_projection.source_vertex_count,
            separation.source_triangle_count,
            separation.output_triangle_count,
            separation.duplicated_boundary_vertex_count,
            uv_projection.source_triangle_count,
            uv_projection.output_triangle_count,
            uv_projection.projected_triangle_count,
            uv_projection.source_vertex_count,
            uv_projection.output_vertex_count,
            textures.resources.len()
        ));
    }
    let dimensions = [
        authoring.bounds_max[0] - authoring.bounds_min[0],
        authoring.bounds_max[1] - authoring.bounds_min[1],
        authoring.bounds_max[2] - authoring.bounds_min[2],
    ];
    if dimensions[0] < 12.0
        || dimensions[1] < 6.0
        || dimensions[2] < 5.0
        || authoring.bounds_min[2].abs() > 0.001
    {
        return Err(format!(
            "TLC-MS2-SCALE-OR-GROUNDING: dimensions={dimensions:?} minZ={}",
            authoring.bounds_min[2]
        ));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("TLC-MS2-HAK-READBACK: {error}"))?;
    let mdl = hak
        .find(&identity.model_resref, MDL_RESOURCE_TYPE)
        .map_err(|error| format!("TLC-MS2-MDL-FIND: {error}"))?;
    let mdl_inspection =
        inspect_binary_mdl(mdl).map_err(|error| format!("TLC-MS2-MDL-READBACK: {error}"))?;
    let mut mdl_texture_resrefs = BTreeSet::new();
    let mut stream_triangles = Vec::new();
    for root in &mdl_inspection.node_tree.roots {
        collect_mdl_readback(root, &mut mdl_texture_resrefs, &mut stream_triangles);
    }
    let expected_texture_resrefs = textures
        .resources
        .iter()
        .map(|resource| resource.resref.clone())
        .collect::<BTreeSet<_>>();
    if stream_triangles.iter().sum::<usize>() != SOURCE_TRIANGLES
        || stream_triangles
            .iter()
            .any(|triangles| *triangles > NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1)
        || mdl_texture_resrefs != expected_texture_resrefs
    {
        return Err(format!(
            "TLC-MS2-MDL-INVARIANT: streams={stream_triangles:?} textures={mdl_texture_resrefs:?} expected={expected_texture_resrefs:?}"
        ));
    }
    let pwk = hak
        .find(&identity.model_resref, PWK_RESOURCE_TYPE)
        .map_err(|error| format!("TLC-MS2-PWK-FIND: {error}"))?;
    if sha256(pwk) != artifact.report.pwk_sha256 {
        return Err("TLC-MS2-PWK-HASH-MISMATCH".to_owned());
    }
    for resource in &textures.resources {
        let tga = hak
            .find(&resource.resref, TGA_RESOURCE_TYPE)
            .map_err(|error| format!("TLC-MS2-TGA-FIND {}: {error}", resource.resref))?;
        if sha256(tga) != resource.sha256 {
            return Err(format!("TLC-MS2-TGA-HASH-MISMATCH: {}", resource.resref));
        }
    }

    let generated = output.join("generated");
    fs::create_dir_all(&generated)
        .map_err(|error| format!("TLC-MS2-OUTPUT-CREATE {}: {error}", generated.display()))?;
    let authoring_bytes = pretty(&bootstrap.document, "AUTHORING")?;
    let report_bytes = pretty(&artifact.report, "REPORT")?;
    let outputs = [
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
        (
            generated.join("material-separation-v2.json"),
            recipe_bytes.clone(),
        ),
        (
            generated.join("material-uv-projection.json"),
            material_uv_projection_bytes,
        ),
        (
            generated.join("texture-authoring.json"),
            texture_authoring_bytes.clone(),
        ),
        (
            generated.join("texture-payload-descriptors.json"),
            descriptor_bytes.clone(),
        ),
        (
            generated.join("source-atlas-variant-report.json"),
            source_atlas_variant_report,
        ),
        (generated.join("authoring.json"), authoring_bytes),
        (generated.join("placeable-report.json"), report_bytes),
        (generated.join("admission.json"), admission_bytes),
        (generated.join("identity.json"), identity_bytes),
    ];
    for (path, payload) in &outputs {
        write_new(path, payload)?;
        if read(path, "OUTPUT-READBACK")? != *payload {
            return Err(format!("TLC-MS2-OUTPUT-MISMATCH: {}", path.display()));
        }
    }
    for resource in &textures.resources {
        let payload = hak
            .find(&resource.resref, TGA_RESOURCE_TYPE)
            .map_err(|error| format!("TLC-MS2-TGA-FIND {}: {error}", resource.resref))?;
        write_new(&generated.join(format!("{}.tga", resource.resref)), payload)?;
    }

    let native_module = install_exact(
        &generated.join(&identity.module_file_name),
        &native_module_directory.join(&identity.module_file_name),
        "MOD",
    )?;
    let native_hak = install_exact(
        &generated.join(&identity.hak_file_name),
        &native_hak_directory.join(&identity.hak_file_name),
        "HAK",
    )?;
    let handoff = HandoffV1 {
        test_module_file_name: identity.module_file_name.clone(),
        toolset_module_name: identity.module_display_name.clone(),
        area_name: identity.area_name.clone(),
        schema_version: 1,
        status: "ready_for_owner_proof".to_owned(),
        area_resref: identity.area_resref.clone(),
        ordered_hak_files: vec![identity.hak_file_name.clone()],
        model_resref: identity.model_resref.clone(),
        texture_resrefs: expected_texture_resrefs.into_iter().collect(),
        blueprint_resref: identity.blueprint_resref.clone(),
        object_tag: identity.object_tag.clone(),
        appearance_row: artifact.report.appearance_row.value,
        placement,
        uniform_scale: SHIP_SCALE,
        dimensions_meters: dimensions,
        source_sha256: SOURCE_SHA256.to_owned(),
        separation_sha256: SEPARATION_SHA256.to_owned(),
        material_uv_projection_sha256: MATERIAL_UV_PROJECTION_SHA256.to_owned(),
        texture_authoring_sha256: TEXTURE_AUTHORING_SHA256.to_owned(),
        source_triangle_count: SOURCE_TRIANGLES,
        output_triangle_count: stream_triangles.iter().sum(),
        source_vertex_count: SOURCE_VERTICES,
        output_vertex_count: uv_projection.output_vertex_count,
        geometry_cleanup: false,
        pwk_sha256: artifact.report.pwk_sha256.clone(),
        mdl_sha256: artifact.report.mdl_sha256.clone(),
        hak_sha256: artifact.report.hak_sha256.clone(),
        module_sha256: artifact.report.module_sha256.clone(),
        native_module,
        native_hak,
        admission,
        model_visibility: "not_tested".to_owned(),
        proof_completeness: "missing".to_owned(),
        owner_proof_required: true,
        starts_toolset: false,
        starts_nwn: false,
    };
    let handoff_bytes = pretty(&handoff, "HANDOFF")?;
    let handoff_path = output.join("ready-for-owner-proof.json");
    write_new(&handoff_path, &handoff_bytes)?;
    if read(&handoff_path, "HANDOFF-READBACK")? != handoff_bytes {
        return Err("TLC-MS2-HANDOFF-MISMATCH".to_owned());
    }
    serde_json::to_string_pretty(&handoff).map_err(|error| format!("TLC-MS2-SUMMARY: {error}"))
}

fn usage() -> String {
    "usage: materialize_tlc_ship_material_separation_v2 <admission.json> <new-identity.json> <output-directory> <native-module-directory> <native-hak-directory>".to_owned()
}

fn validate_axis(axis: &ProofAxisV1, path: &str) -> Result<(), String> {
    if !matches!(
        axis.model_visibility.as_str(),
        "visible" | "not_visible" | "not_tested"
    ) {
        return Err(format!("TLC-MS2-{path}-VISIBILITY-INVALID"));
    }
    if !matches!(
        axis.proof_completeness.as_str(),
        "verified" | "failed" | "missing"
    ) {
        return Err(format!("TLC-MS2-{path}-COMPLETENESS-INVALID"));
    }
    if axis.owner_observation.trim().is_empty() {
        return Err(format!("TLC-MS2-{path}-OWNER-OBSERVATION-MISSING"));
    }
    Ok(())
}

fn validate_admission(admission: &MaterialIterationAdmissionV1) -> Result<(), String> {
    if admission.schema_version != 1 || admission.recorded_at.trim().is_empty() {
        return Err("TLC-MS2-ADMISSION-SCHEMA-OR-TIME-INVALID".to_owned());
    }
    let prior = &admission.prior_candidate;
    if prior.module_file_name != PRIOR_MODULE_FILE_NAME
        || prior.module_sha256 != PRIOR_MODULE_SHA256
        || prior.hak_file_name != PRIOR_HAK_FILE_NAME
        || prior.hak_sha256 != PRIOR_HAK_SHA256
    {
        return Err("TLC-MS2-ADMISSION-PRIOR-CANDIDATE-MISMATCH".to_owned());
    }
    validate_axis(&admission.toolset, "TOOLSET")?;
    validate_axis(&admission.nwn, "NWN")?;
    let visual_failure = admission.toolset.model_visibility == "not_visible"
        || admission.nwn.model_visibility == "not_visible";
    let exception = admission
        .material_only_iteration_exception
        .as_ref()
        .is_some_and(|exception| {
            exception.authorized && !exception.owner_statement.trim().is_empty()
        });
    if !visual_failure && !exception {
        return Err(
            "TLC-MS2-ITERATION-NOT-ADMITTED: exact prior candidate needs not_visible or a direct owner material-only exception"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_new_identity(identity: &StaticPlaceableIdentityV1) -> Result<(), String> {
    if identity.module_resref == PRIOR_MODULE_RESREF
        || identity.module_file_name == PRIOR_MODULE_FILE_NAME
        || identity.area_resref == PRIOR_AREA_RESREF
        || identity.hak_resref == PRIOR_HAK_RESREF
        || identity.hak_file_name == PRIOR_HAK_FILE_NAME
        || identity.model_resref == PRIOR_MODEL_RESREF
        || identity.texture_resref == PRIOR_TEXTURE_RESREF
        || identity.blueprint_resref == PRIOR_BLUEPRINT_RESREF
        || identity.object_tag == PRIOR_OBJECT_TAG
    {
        return Err("TLC-MS2-IDENTITY-REUSES-FROZEN-PRIOR-LINEAGE".to_owned());
    }
    Ok(())
}

fn verify_prior_native_candidate(
    native_module_directory: &Path,
    native_hak_directory: &Path,
) -> Result<(), String> {
    read_exact(
        &native_module_directory.join(PRIOR_MODULE_FILE_NAME),
        PRIOR_MODULE_SHA256,
        "PRIOR-NATIVE-MOD",
    )?;
    read_exact(
        &native_hak_directory.join(PRIOR_HAK_FILE_NAME),
        PRIOR_HAK_SHA256,
        "PRIOR-NATIVE-HAK",
    )?;
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

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let canonical =
        fs::canonicalize(path).map_err(|error| format!("TLC-MS2-{label}-RESOLVE: {error}"))?;
    if !canonical.is_dir() {
        return Err(format!("TLC-MS2-{label}-NOT-DIRECTORY"));
    }
    Ok(canonical)
}

fn install_exact(
    source: &Path,
    destination: &Path,
    label: &str,
) -> Result<NativeInstallationV1, String> {
    let source = fs::canonicalize(source)
        .map_err(|error| format!("TLC-MS2-{label}-SOURCE-RESOLVE: {error}"))?;
    let source_bytes = read(&source, &format!("{label}-INSTALL-SOURCE"))?;
    let source_hash = sha256(&source_bytes);
    let reused = if destination.exists() {
        let existing = read(destination, &format!("{label}-INSTALL-EXISTING"))?;
        if existing != source_bytes {
            return Err(format!(
                "TLC-MS2-{label}-NATIVE-COLLISION: {} existingSha256={} sourceSha256={source_hash}",
                destination.display(),
                sha256(&existing)
            ));
        }
        true
    } else {
        write_new(destination, &source_bytes)?;
        false
    };
    let installed = read(destination, &format!("{label}-INSTALL-READBACK"))?;
    if installed != source_bytes {
        return Err(format!("TLC-MS2-{label}-INSTALL-HASH-MISMATCH"));
    }
    Ok(NativeInstallationV1 {
        source_path: source,
        destination_path: destination.to_owned(),
        byte_length: source_bytes.len(),
        sha256: source_hash,
        reused_identical_existing: reused,
        byte_identical: true,
    })
}

fn exact_error(label: &str, error: &impl Serialize) -> String {
    serde_json::to_string(error).unwrap_or_else(|_| format!("{label}: serialization failed"))
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("TLC-MS2-{label}-READ {}: {error}", path.display()))
}

fn read_exact(path: &Path, expected_sha256: &str, label: &str) -> Result<Vec<u8>, String> {
    let payload = read(path, label)?;
    let actual = sha256(&payload);
    if actual != expected_sha256 {
        return Err(format!(
            "TLC-MS2-{label}-HASH: expected {expected_sha256}, got {actual}"
        ));
    }
    Ok(payload)
}

fn write_new(path: &Path, payload: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("TLC-MS2-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(payload)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("TLC-MS2-WRITE {}: {error}", path.display()))
}

fn pretty(value: &impl Serialize, label: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value).map_err(|error| format!("TLC-MS2-{label}-SERIALIZE: {error}"))
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(model_visibility: &str) -> ProofAxisV1 {
        ProofAxisV1 {
            model_visibility: model_visibility.to_owned(),
            proof_completeness: if model_visibility == "not_visible" {
                "verified".to_owned()
            } else {
                "missing".to_owned()
            },
            owner_observation: "owner-bound observation".to_owned(),
        }
    }

    fn admission(
        toolset_visibility: &str,
        nwn_visibility: &str,
        exception: Option<MaterialOnlyExceptionV1>,
    ) -> MaterialIterationAdmissionV1 {
        MaterialIterationAdmissionV1 {
            schema_version: 1,
            recorded_at: "2026-08-02T00:00:00+02:00".to_owned(),
            prior_candidate: PriorCandidateV1 {
                module_file_name: PRIOR_MODULE_FILE_NAME.to_owned(),
                module_sha256: PRIOR_MODULE_SHA256.to_owned(),
                hak_file_name: PRIOR_HAK_FILE_NAME.to_owned(),
                hak_sha256: PRIOR_HAK_SHA256.to_owned(),
            },
            toolset: axis(toolset_visibility),
            nwn: axis(nwn_visibility),
            material_only_iteration_exception: exception,
        }
    }

    fn new_identity() -> StaticPlaceableIdentityV1 {
        StaticPlaceableIdentityV1 {
            module_resref: "m2a_tlcsm2_mod".to_owned(),
            module_file_name: "m2a_tlcsm2_mod.mod".to_owned(),
            module_display_name: "The Last City - Material Separation Ship Demo".to_owned(),
            area_resref: "m2a_tlcsm2_ar".to_owned(),
            area_name: "The Last City Material Separation Shipyard".to_owned(),
            hak_resref: "m2a_tlcsm2_hak".to_owned(),
            hak_file_name: "m2a_tlcsm2_hak.hak".to_owned(),
            model_resref: "m2a_tlcsm2_mdl".to_owned(),
            texture_resref: "m2a_tlcsm2_tex".to_owned(),
            blueprint_resref: "m2a_tlcsm2_utp".to_owned(),
            object_tag: "m2a_tlcsm2_ship".to_owned(),
            display_name: "Ship Under Construction - Material Separation".to_owned(),
        }
    }

    #[test]
    fn admission_rejects_unproven_prior_without_exception() {
        let error = validate_admission(&admission("not_tested", "not_tested", None))
            .expect_err("unproven prior candidate must remain gated");
        assert!(error.contains("ITERATION-NOT-ADMITTED"));
    }

    #[test]
    fn admission_accepts_candidate_bound_visual_failure() {
        validate_admission(&admission("visible", "not_visible", None))
            .expect("a candidate-bound NWN not_visible result admits a new iteration");
    }

    #[test]
    fn admission_accepts_direct_material_only_exception() {
        validate_admission(&admission(
            "visible",
            "visible",
            Some(MaterialOnlyExceptionV1 {
                authorized: true,
                owner_statement: "Authorize one material-only iteration.".to_owned(),
            }),
        ))
        .expect("a non-empty direct owner exception admits a material-only iteration");
    }

    #[test]
    fn admission_rejects_wrong_frozen_candidate_hash() {
        let mut document = admission("not_visible", "not_tested", None);
        document.prior_candidate.module_sha256 = "00".repeat(32);
        let error = validate_admission(&document).expect_err("wrong prior hash must fail closed");
        assert!(error.contains("PRIOR-CANDIDATE-MISMATCH"));
    }

    #[test]
    fn identity_rejects_reuse_of_frozen_resref() {
        let mut identity = new_identity();
        identity.area_resref = PRIOR_AREA_RESREF.to_owned();
        let error =
            validate_new_identity(&identity).expect_err("prior resref reuse must fail closed");
        assert!(error.contains("REUSES-FROZEN-PRIOR-LINEAGE"));
    }

    #[test]
    fn identity_accepts_fresh_lineage() {
        validate_new_identity(&new_identity()).expect("fresh identity must pass preflight");
    }
}
