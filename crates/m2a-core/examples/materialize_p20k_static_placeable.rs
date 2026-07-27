use std::{
    env,
    f32::consts::TAU,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    mdl::{
        MdlMaterialTextureBindingV1, NWN_EE_MAX_MESH_INDEX_COUNT_V1,
        NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, inspect_binary_mdl,
    },
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
        AuroraSegmentDeformationV1,
    },
    placeable::{
        ARE_RESOURCE_TYPE, FAC_RESOURCE_TYPE, GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE,
        IFO_RESOURCE_TYPE, ITP_RESOURCE_TYPE, MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE,
        PlaceablePlacementV1, PlaceableTextureInputV1, StaticPlaceableBuildRequestV1,
        StaticPlaceableIdentityV1, TGA_RESOURCE_TYPE, UTP_RESOURCE_TYPE,
        build_static_placeable_package_v1,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1},
};
use serde_json::json;
use sha2::{Digest, Sha256};

const CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\p20k-placeable-stress-v1-20260725";
const BASE_PLACEABLES_SHA256: &str =
    "e8bce48f354c9e76597ccb95cd643a8b5c7a08f6ce211ec22a29d5507fab32a4";
const TRIANGLE_COUNT: usize = 20_000;
const RADIAL_SECTORS: usize = 100;
const PROFILE_RINGS: usize = 100;
const RING_STRIDE: usize = RADIAL_SECTORS + 1;
const VERTEX_COUNT: usize = 2 + PROFILE_RINGS * RING_STRIDE;
const INDEX_COUNT: usize = TRIANGLE_COUNT * 3;
const ACTUAL_AREA_NAME: &str = "Meshy2Aurora M0 binary vertical-slice area";

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
    if command.output.as_path() != Path::new(CANONICAL_OUTPUT) {
        return Err(format!(
            "PLACEABLE-P20K-OUTPUT-IDENTITY: exact output must be {CANONICAL_OUTPUT}"
        ));
    }
    if command.output.exists() {
        return Err(format!(
            "PLACEABLE-P20K-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }

    let base_placeables = read(&command.placeables_two_da, "PLACEABLES-2DA")?;
    require_hash(&base_placeables, BASE_PLACEABLES_SHA256, "PLACEABLES-2DA")?;

    let identity = StaticPlaceableIdentityV1 {
        module_resref: "m2a_p20k_mod".to_owned(),
        module_file_name: "m2a_p20k_mod.mod".to_owned(),
        module_display_name: "Meshy2Aurora P20K Placeable Stress Test".to_owned(),
        area_resref: "m2a_p20k_ar".to_owned(),
        // The current shared owned Area fixture emits this exact localized
        // name. Keep the owner handoff truthful until the Area builder takes
        // a domain-specific display name.
        area_name: ACTUAL_AREA_NAME.to_owned(),
        hak_resref: "m2a_p20k_hak".to_owned(),
        hak_file_name: "m2a_p20k_hak.hak".to_owned(),
        model_resref: "m2a_p20k_rel".to_owned(),
        texture_resref: "m2a_p20k_tex".to_owned(),
        blueprint_resref: "m2a_p20k_utp".to_owned(),
        object_tag: "m2a_p20k_stress_reliquary".to_owned(),
        display_name: "M2A 20K Stress Reliquary".to_owned(),
    };
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let model = build_stress_reliquary(&identity.model_resref)?;
    verify_source_geometry(&model)?;
    let texture = build_stress_texture()?;
    let texture_sha256 = sha256(&texture);

    let artifact = build_static_placeable_package_v1(&StaticPlaceableBuildRequestV1 {
        schema_version: 1,
        identity: identity.clone(),
        placement,
        palette_id: 7,
        base_placeables_2da: base_placeables.clone(),
        model: model.clone(),
        collision_model: None,
        authoring_report: None,
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: identity.texture_resref.clone(),
        }],
        textures: vec![PlaceableTextureInputV1 {
            resref: identity.texture_resref.clone(),
            resource_type: TGA_RESOURCE_TYPE,
            payload: texture.clone(),
        }],
    })
    .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;

    if artifact.report.appearance_row.value != 16_501 {
        return Err(format!(
            "PLACEABLE-P20K-APPEARANCE-ROW: expected 16501, got {}",
            artifact.report.appearance_row.value
        ));
    }

    let hak = ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| format!("PLACEABLE-P20K-HAK-READBACK: {error}"))?;
    let module = ErfArchive::parse(&artifact.module_payload)
        .map_err(|error| format!("PLACEABLE-P20K-MOD-READBACK: {error}"))?;
    let mdl = hak
        .find(&identity.model_resref, MDL_RESOURCE_TYPE)
        .map_err(|error| format!("PLACEABLE-P20K-MDL-FIND: {error}"))?;
    let mdl_inspection =
        inspect_binary_mdl(mdl).map_err(|error| format!("PLACEABLE-P20K-MDL-READBACK: {error}"))?;
    let mesh = mdl_inspection
        .node_tree
        .roots
        .first()
        .and_then(|root| root.children.first())
        .and_then(|node| node.mesh.as_ref())
        .ok_or_else(|| "PLACEABLE-P20K-MDL-MESH-MISSING".to_owned())?;
    if mesh.vertex_count != VERTEX_COUNT
        || mesh.faces.len() != TRIANGLE_COUNT
        || mesh.index_counts.as_slice() != [INDEX_COUNT as u32]
        || mesh.raw_indices.len() != 1
        || mesh.raw_indices[0].len() != INDEX_COUNT
    {
        return Err(format!(
            "PLACEABLE-P20K-MDL-GEOMETRY-MISMATCH: vertices={} faces={} indexCounts={:?} rawIndexStreams={}",
            mesh.vertex_count,
            mesh.faces.len(),
            mesh.index_counts,
            mesh.raw_indices.len()
        ));
    }
    if mdl_inspection.diagnostics.iter().any(|item| {
        item.code.contains("DEGENERATE") || item.context.to_ascii_lowercase().contains("degenerate")
    }) {
        return Err("PLACEABLE-P20K-MDL-DEGENERATE-READBACK".to_owned());
    }

    let source_spec = json!({
        "schemaVersion": 1,
        "kind": "SYNTHETIC_AURORA_NATIVE_PLACEABLE_STRESS_FIXTURE",
        "name": "Stress Reliquary",
        "generator": "materialize_p20k_static_placeable",
        "radialSectors": RADIAL_SECTORS,
        "profileRings": PROFILE_RINGS,
        "vertexCount": VERTEX_COUNT,
        "triangleCount": TRIANGLE_COUNT,
        "indexCount": INDEX_COUNT,
        "meshCount": 1,
        "materialCount": 1,
        "sourceModelSha256": model.source_sha256,
        "provenance": {
            "controlledConstruction": true,
            "noReferencePayloadCopied": true,
            "rightsConfirmed": true
        }
    });
    let source_spec_json = pretty(&source_spec, "SOURCE-SPEC")?;
    let source_model_json = pretty(&model, "SOURCE-MODEL-IR")?;
    let report_json = pretty(&artifact.report, "REPORT")?;
    let geometry_readback = json!({
        "schemaVersion": 1,
        "mdlSha256": sha256(mdl),
        "modelName": mdl_inspection.model.name,
        "meshCount": 1,
        "vertexCount": mesh.vertex_count,
        "triangleCount": mesh.faces.len(),
        "indexCount": mesh.index_counts[0],
        "rawIndexCount": mesh.raw_indices[0].len(),
        "meshType": mesh.mesh_type,
        "render": mesh.render,
        "textureResrefs": mesh.textures,
        "diagnosticCount": mdl_inspection.diagnostics.len(),
        "semanticContract": {
            "oneMesh": true,
            "exactTwentyThousandTriangles": true,
            "belowNwnEePerMeshTriangleBoundary": TRIANGLE_COUNT <= NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1,
            "belowNwnEePerMeshIndexBoundary": INDEX_COUNT <= NWN_EE_MAX_MESH_INDEX_COUNT_V1
        }
    });
    let geometry_readback_json = pretty(&geometry_readback, "GEOMETRY-READBACK")?;

    let generated = command.output.join("generated");
    fs::create_dir_all(&generated)
        .map_err(|error| format!("PLACEABLE-P20K-OUTPUT-CREATE: {error}"))?;
    let outputs = vec![
        (generated.join("source-spec.json"), source_spec_json.clone()),
        (
            generated.join("source-model-ir.json"),
            source_model_json.clone(),
        ),
        (
            generated.join("base-placeables.2da"),
            base_placeables.clone(),
        ),
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
        let actual = read(path, "WRITTEN-OUTPUT")?;
        if actual != *expected {
            return Err(format!(
                "PLACEABLE-P20K-OUTPUT-READBACK: {}",
                path.display()
            ));
        }
    }

    let handoff = json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "lane": "P20K_SINGLE_MESH_STRESS_V1",
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
        "geometry": {
            "meshCount": 1,
            "vertexCount": VERTEX_COUNT,
            "triangleCount": TRIANGLE_COUNT,
            "indexCount": INDEX_COUNT,
            "perMeshTriangleLimit": NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1,
            "perMeshIndexLimit": NWN_EE_MAX_MESH_INDEX_COUNT_V1,
            "triangleHeadroom": NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1 - TRIANGLE_COUNT
        },
        "source": {
            "kind": "synthetic-controlled-construction",
            "sourceSpecSha256": sha256(&source_spec_json),
            "sourceModelIrSha256": sha256(&source_model_json),
            "sourceModelSha256": model.source_sha256
        },
        "basePlaceables2da": {
            "path": command.placeables_two_da,
            "byteLength": base_placeables.len(),
            "sha256": sha256(&base_placeables)
        },
        "outputs": {
            "mdlSha256": artifact.report.mdl_sha256,
            "textureSha256": texture_sha256,
            "placeables2daSha256": artifact.report.placeables_2da_sha256,
            "utpSha256": artifact.report.utp_sha256,
            "itpSha256": artifact.report.itp_sha256,
            "gitSha256": artifact.report.git_sha256,
            "gicSha256": artifact.report.gic_sha256,
            "hakSha256": artifact.report.hak_sha256,
            "moduleSha256": artifact.report.module_sha256,
            "reportSha256": sha256(&report_json),
            "geometryReadbackSha256": sha256(&geometry_readback_json)
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
        "materializationCount": 1
    });
    let handoff_json = pretty(&handoff, "HANDOFF")?;
    let handoff_path = command.output.join("ready-for-owner-proof.json");
    write_new(&handoff_path, &handoff_json)?;
    if read(&handoff_path, "HANDOFF-READBACK")? != handoff_json {
        return Err("PLACEABLE-P20K-HANDOFF-READBACK".to_owned());
    }

    serde_json::to_string_pretty(&handoff)
        .map_err(|error| format!("PLACEABLE-P20K-SUMMARY: {error}"))
}

fn build_stress_reliquary(model_resref: &str) -> Result<AuroraModelIrV1, String> {
    if TRIANGLE_COUNT != 2 * RADIAL_SECTORS * PROFILE_RINGS
        || INDEX_COUNT > NWN_EE_MAX_MESH_INDEX_COUNT_V1
    {
        return Err("PLACEABLE-P20K-GENERATOR-CONSTANTS-INVALID".to_owned());
    }

    let mut positions = Vec::with_capacity(VERTEX_COUNT);
    let mut uv0 = Vec::with_capacity(VERTEX_COUNT);
    positions.push([0.0, 0.0, 0.0]);
    uv0.push([0.5, 0.0]);
    for ring in 0..PROFILE_RINGS {
        let t = (ring + 1) as f32 / (PROFILE_RINGS + 1) as f32;
        let z = 3.2 * t;
        for sector in 0..=RADIAL_SECTORS {
            let u = sector as f32 / RADIAL_SECTORS as f32;
            let theta = TAU * u;
            let radius = reliquary_radius(t, theta);
            positions.push([radius * theta.cos(), radius * theta.sin(), z]);
            uv0.push([u, t]);
        }
    }
    let top = positions.len() as u32;
    positions.push([0.0, 0.0, 3.2]);
    uv0.push([0.5, 1.0]);

    let mut indices = Vec::with_capacity(INDEX_COUNT);
    let first_ring = 1_u32;
    for sector in 0..RADIAL_SECTORS {
        let current = first_ring + sector as u32;
        let next = current + 1;
        indices.extend_from_slice(&[0, next, current]);
    }
    for ring in 0..(PROFILE_RINGS - 1) {
        let lower = 1 + ring * RING_STRIDE;
        let upper = lower + RING_STRIDE;
        for sector in 0..RADIAL_SECTORS {
            let a = (lower + sector) as u32;
            let b = a + 1;
            let c = (upper + sector) as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    let last_ring = 1 + (PROFILE_RINGS - 1) * RING_STRIDE;
    for sector in 0..RADIAL_SECTORS {
        let current = (last_ring + sector) as u32;
        let next = current + 1;
        indices.extend_from_slice(&[current, next, top]);
    }

    let mut normals = derive_smooth_normals(&positions, &indices)?;
    normals[0] = [0.0, 0.0, -1.0];
    normals[top as usize] = [0.0, 0.0, 1.0];
    for ring in 0..PROFILE_RINGS {
        let first = 1 + ring * RING_STRIDE;
        let seam = first + RADIAL_SECTORS;
        let combined = normalize(add3(normals[first], normals[seam]))?;
        normals[first] = combined;
        normals[seam] = combined;
    }

    let source_sha256 = geometry_sha256(&positions, &normals, &uv0, &indices);
    Ok(AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "synthetic-placeable-20k-stress-v1".to_owned(),
        source_sha256,
        basis_status: "AURORA_NATIVE_SYNTHETIC_V1".to_owned(),
        engine_facing_proof: "OWNER_PROOF_REQUIRED".to_owned(),
        uv_runtime_proof: "OWNER_PROOF_REQUIRED".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 1,
            name: model_resref.to_owned(),
            parent_id: None,
            bind_local_matrix: identity(),
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: None,
            source_material_name: Some("synthetic-stress-bronze".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 1,
            material_slot: 0,
            cast_shadow: true,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 1,
            positions,
            normals,
            tangents: None,
            uv0,
            indices,
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    })
}

fn reliquary_radius(t: f32, theta: f32) -> f32 {
    let base = if t < 0.08 {
        lerp(0.92, 0.82, t / 0.08)
    } else if t < 0.18 {
        lerp(0.82, 0.48, (t - 0.08) / 0.10)
    } else if t < 0.66 {
        0.43 + 0.035 * (t * 18.0 * TAU).sin()
    } else if t < 0.76 {
        lerp(0.48, 0.72, (t - 0.66) / 0.10)
    } else if t < 0.90 {
        0.72 + 0.10 * ((t - 0.76) / 0.14 * std::f32::consts::PI).sin()
    } else {
        lerp(0.66, 0.42, (t - 0.90) / 0.10)
    };
    let lobe_strength = if (0.20..0.64).contains(&t) {
        0.055
    } else {
        0.025
    };
    base * (1.0 + lobe_strength * (theta * 8.0).cos())
}

fn derive_smooth_normals(positions: &[[f32; 3]], indices: &[u32]) -> Result<Vec<[f32; 3]>, String> {
    let mut normals = vec![[0.0_f32; 3]; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let a = positions[triangle[0] as usize];
        let b = positions[triangle[1] as usize];
        let c = positions[triangle[2] as usize];
        let face = cross(sub3(b, a), sub3(c, a));
        if length_sq(face) <= f32::EPSILON {
            return Err("PLACEABLE-P20K-DEGENERATE-TRIANGLE".to_owned());
        }
        for &vertex in triangle {
            normals[vertex as usize] = add3(normals[vertex as usize], face);
        }
    }
    normals.into_iter().map(normalize).collect()
}

fn verify_source_geometry(model: &AuroraModelIrV1) -> Result<(), String> {
    if model.segments.len() != 1 {
        return Err("PLACEABLE-P20K-SOURCE-MESH-COUNT".to_owned());
    }
    let segment = &model.segments[0];
    if segment.positions.len() != VERTEX_COUNT
        || segment.normals.len() != VERTEX_COUNT
        || segment.uv0.len() != VERTEX_COUNT
        || segment.indices.len() != INDEX_COUNT
        || segment.indices.len() / 3 != TRIANGLE_COUNT
        || segment
            .indices
            .iter()
            .any(|&value| value as usize >= VERTEX_COUNT)
    {
        return Err("PLACEABLE-P20K-SOURCE-GEOMETRY-MISMATCH".to_owned());
    }
    Ok(())
}

fn build_stress_texture() -> Result<Vec<u8>, String> {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 256;
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let rune = (x + 2 * y) % 37 < 3 || (2 * x + HEIGHT - y) % 53 < 3;
            let band = y % 32 < 5;
            let border = x % 64 < 4;
            let (red, green, blue) = if rune {
                (55, 220, 235)
            } else if band || border {
                (188, 124, 42)
            } else {
                let noise = ((x * 17 + y * 29) % 23) as u8;
                (52 + noise, 45 + noise / 2, 38 + noise / 3)
            };
            pixels.extend_from_slice(&[red, green, blue]);
        }
    }
    write_tga_v1(
        &TgaImageV1 {
            schema_version: 1,
            width: WIDTH,
            height: HEIGHT,
            pixel_format: TgaPixelFormatV1::Rgb8,
            pixels,
        },
        &TgaWriterOptionsV1::default(),
    )
    .map(|artifact| artifact.payload)
    .map_err(|error| format!("PLACEABLE-P20K-TGA-WRITE: {error}"))
}

fn geometry_sha256(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uv0: &[[f32; 2]],
    indices: &[u32],
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"meshy2aurora:p20k-stress-reliquary:v1");
    for row in positions {
        for value in row {
            digest.update(value.to_le_bytes());
        }
    }
    for row in normals {
        for value in row {
            digest.update(value.to_le_bytes());
        }
    }
    for row in uv0 {
        for value in row {
            digest.update(value.to_le_bytes());
        }
    }
    for value in indices {
        digest.update(value.to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn identity() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length_sq(value: [f32; 3]) -> f32 {
    value[0] * value[0] + value[1] * value[1] + value[2] * value[2]
}

fn normalize(value: [f32; 3]) -> Result<[f32; 3], String> {
    let length = length_sq(value).sqrt();
    if !length.is_finite() || length <= f32::EPSILON {
        return Err("PLACEABLE-P20K-NORMAL-INVALID".to_owned());
    }
    Ok([value[0] / length, value[1] / length, value[2] / length])
}

struct Command {
    placeables_two_da: PathBuf,
    output: PathBuf,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut placeables_two_da = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--placeables-2da" => &mut placeables_two_da,
            "--out" => &mut output,
            _ => {
                return Err(format!("PLACEABLE-P20K-ARGUMENT: {argument}\n{}", usage()));
            }
        };
        if target.is_some() {
            return Err(format!("PLACEABLE-P20K-ARGUMENT-DUPLICATE: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("PLACEABLE-P20K-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        placeables_two_da: PathBuf::from(placeables_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn usage() -> String {
    format!(
        "usage: materialize_p20k_static_placeable --placeables-2da <exact-full-table> --out {CANONICAL_OUTPUT}"
    )
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|error| format!("PLACEABLE-P20K-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "PLACEABLE-P20K-OUTPUT-CREATE-NEW {}: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("PLACEABLE-P20K-OUTPUT-WRITE {}: {error}", path.display()))
}

fn require_hash(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256(bytes);
    if actual != expected {
        return Err(format!(
            "PLACEABLE-P20K-{label}-HASH: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn pretty(value: &impl serde::Serialize, label: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value)
        .map_err(|error| format!("PLACEABLE-P20K-{label}-SERIALIZE: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
