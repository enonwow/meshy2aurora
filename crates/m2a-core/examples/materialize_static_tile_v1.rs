use std::{fs, path::PathBuf};

use m2a_core::{
    mdl::MdlMaterialTextureBindingV1,
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
        AuroraSegmentDeformationV1,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1},
    tile::{
        StaticTileBuildRequestV1, StaticTileIdentityV1, TileSurfaceV1, TileTextureInputV1,
        build_static_tile_package_v1,
    },
    walkmesh::flat_tile_navigation_v1,
};

fn model(identity: &StaticTileIdentityV1) -> AuroraModelIrV1 {
    let positions = vec![
        [-5.0, -5.0, 0.0],
        [0.0, -5.0, 0.08],
        [5.0, -5.0, 0.0],
        [-5.0, 0.0, 0.08],
        [0.0, 0.0, 0.18],
        [5.0, 0.0, 0.08],
        [-5.0, 5.0, 0.0],
        [0.0, 5.0, 0.08],
        [5.0, 5.0, 0.0],
    ];
    let uv0 = positions
        .iter()
        .map(|position| [(position[0] + 5.0) / 10.0, (position[1] + 5.0) / 10.0])
        .collect::<Vec<_>>();
    let indices = vec![
        0, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4, 3, 4, 7, 3, 7, 6, 4, 5, 8, 4, 8, 7,
    ];
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "TILE_STATIC_V1_OWNER_CANDIDATE".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "PROJECT_OWNED_PROCEDURAL_FIXTURE".to_owned(),
        engine_facing_proof: "READY_FOR_OWNER_PROOF".to_owned(),
        uv_runtime_proof: "READY_FOR_OWNER_PROOF".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 0,
            name: identity.model_resref.clone(),
            parent_id: None,
            bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0,
            ],
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: Some(0),
            source_material_name: Some("tile_grass_checker".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 0,
            material_slot: 0,
            cast_shadow: true,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 0,
            normals: vec![[0.0, 0.0, 1.0]; positions.len()],
            tangents: None,
            positions,
            uv0,
            indices,
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    }
}

fn checker_tga() -> Vec<u8> {
    let mut pixels = Vec::with_capacity(4 * 4 * 3);
    for y in 0..4 {
        for x in 0..4 {
            let rgb = if (x + y) % 2 == 0 {
                [54, 112, 46]
            } else {
                [112, 88, 52]
            };
            pixels.extend_from_slice(&rgb);
        }
    }
    write_tga_v1(
        &TgaImageV1 {
            schema_version: 1,
            width: 4,
            height: 4,
            pixel_format: TgaPixelFormatV1::Rgb8,
            pixels,
        },
        &TgaWriterOptionsV1::default(),
    )
    .expect("valid project-owned checker TGA")
    .payload
}

fn main() {
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|crates| crates.parent())
        .expect("repository root")
        .to_path_buf();
    assert_eq!(
        repository_root,
        PathBuf::from(r"C:\Projects\meshy2aurora"),
        "candidate materialization is canonical-workspace-only"
    );
    let output = repository_root.join("artifacts").join("tile-static-v1");
    fs::create_dir_all(&output).expect("create canonical artifact directory");

    let identity = StaticTileIdentityV1::owner_candidate_v1();
    let texture = checker_tga();
    let artifact = build_static_tile_package_v1(&StaticTileBuildRequestV1 {
        schema_version: 1,
        identity: identity.clone(),
        interior: false,
        terrain_name: "Grass".to_owned(),
        surface: TileSurfaceV1::Grass,
        model: model(&identity),
        navigation: flat_tile_navigation_v1(&identity.model_resref, TileSurfaceV1::Grass)
            .expect("flat tile navigation"),
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: identity.texture_resref.clone(),
        }],
        textures: vec![TileTextureInputV1 {
            resref: identity.texture_resref.clone(),
            resource_type: 3,
            payload: texture,
        }],
    })
    .expect("materialize exact tile package");

    for (name, payload) in [
        (
            identity.module_file_name.clone(),
            artifact.module_payload.as_slice(),
        ),
        (
            identity.hak_file_name.clone(),
            artifact.hak_payload.as_slice(),
        ),
        (
            format!("{}.mdl", identity.model_resref),
            artifact.mdl_payload.as_slice(),
        ),
        (
            format!("{}.wok", identity.model_resref),
            artifact.wok_payload.as_slice(),
        ),
        (
            format!("{}.set", identity.tileset_resref),
            artifact.set_payload.as_slice(),
        ),
        (
            format!("{}.tga", identity.texture_resref),
            artifact.texture_payload.as_slice(),
        ),
        (
            format!("{}.tga", identity.image_map_resref),
            artifact.image_map_payload.as_slice(),
        ),
    ] {
        fs::write(output.join(name), payload).expect("write exact candidate artifact");
    }
    fs::write(
        output.join("tile-materialization-report.json"),
        serde_json::to_vec_pretty(&artifact.report).expect("serialize report"),
    )
    .expect("write candidate report");
    println!("{}", serde_json::to_string(&artifact.report).unwrap());
}
