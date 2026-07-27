use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use m2a_core::{
    erf::ErfArchive,
    gff::{GffLimitsV1, GffValueV1, read_gff_v32},
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
        MdlWriterOptionsV1, inspect_binary_mdl, write_binary_tile_mdl_v1,
    },
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
        AuroraSegmentDeformationV1,
    },
    tile::{
        MDL_RESOURCE_TYPE, SET_RESOURCE_TYPE, StaticTileBuildRequestV1, StaticTileIdentityV1,
        TileDescriptorV1, TileSurfaceV1, TileTextureInputV1, build_static_tile_package_v1,
        minimal_static_tileset_v1, parse_tileset_v1, resolve_are_tile_v1, write_tileset_v1,
    },
    walkmesh::{
        build_aabb_tree_v1, flat_tile_navigation_v1, inspect_ascii_tile_wok_v1,
        validate_aabb_tree_v1, write_ascii_tile_wok_v1,
    },
};
use sha2::{Digest, Sha256};

fn identity_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn simple_model(model_resref: &str, triangle_count: usize) -> AuroraModelIrV1 {
    let mut positions = Vec::with_capacity(triangle_count + 2);
    let mut uv0 = Vec::with_capacity(triangle_count + 2);
    positions.push([0.0, 0.0, 0.0]);
    uv0.push([0.5, 0.5]);
    for index in 0..=triangle_count {
        let angle = std::f32::consts::TAU * index as f32 / triangle_count.max(1) as f32;
        positions.push([angle.cos() * 4.0, angle.sin() * 4.0, 0.25]);
        uv0.push([(angle.cos() + 1.0) * 0.5, (angle.sin() + 1.0) * 0.5]);
    }
    let mut indices = Vec::with_capacity(triangle_count * 3);
    for index in 0..triangle_count {
        indices.extend_from_slice(&[0, index as u32 + 1, index as u32 + 2]);
    }
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "TILE_TEST".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "resolved".to_owned(),
        engine_facing_proof: "offline_fixture".to_owned(),
        uv_runtime_proof: "offline_fixture".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 0,
            name: model_resref.to_owned(),
            parent_id: None,
            bind_local_matrix: identity_matrix(),
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: Some(0),
            source_material_name: Some("tile".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 0,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 0,
            cast_shadow: true,
            positions: positions.clone(),
            normals: vec![[0.0, 0.0, 1.0]; positions.len()],
            tangents: None,
            uv0,
            indices,
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    }
}

fn tile_options(model_resref: &str, texture_resref: &str) -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::TileStaticV1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: model_resref.to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: texture_resref.to_owned(),
        }],
    }
}

#[test]
fn aabb_tree_and_ascii_wok_are_deterministic_and_fail_closed() {
    let navigation =
        flat_tile_navigation_v1("m2atilemdl1", TileSurfaceV1::Grass).expect("flat navigation");
    assert_eq!(navigation.vertices.len(), 9);
    assert_eq!(navigation.faces.len(), 8);
    assert_eq!(navigation.aabb_tree.entries.len(), 15);
    let min_x = navigation
        .vertices
        .iter()
        .map(|vertex| vertex[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = navigation
        .vertices
        .iter()
        .map(|vertex| vertex[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = navigation
        .vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = navigation
        .vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(f32::NEG_INFINITY, f32::max);
    assert_eq!([min_x, min_y, max_x, max_y], [-5.0, -5.0, 5.0, 5.0]);
    let mut east_seam = navigation
        .vertices
        .iter()
        .filter(|vertex| vertex[0] == 5.0)
        .map(|vertex| [vertex[1], vertex[2]])
        .collect::<Vec<_>>();
    let mut translated_west_seam = navigation
        .vertices
        .iter()
        .filter(|vertex| vertex[0] == -5.0)
        .map(|vertex| [vertex[1], vertex[2]])
        .collect::<Vec<_>>();
    east_seam.sort_by(|left, right| left[0].total_cmp(&right[0]));
    translated_west_seam.sort_by(|left, right| left[0].total_cmp(&right[0]));
    assert_eq!(east_seam, translated_west_seam);
    validate_aabb_tree_v1(
        &navigation.aabb_tree,
        &navigation.vertices,
        &navigation.faces,
    )
    .expect("valid AABB tree");

    let first = write_ascii_tile_wok_v1(&navigation).expect("first WOK");
    let second = write_ascii_tile_wok_v1(&navigation).expect("second WOK");
    assert_eq!(first.payload, second.payload);
    let text = std::str::from_utf8(&first.payload).expect("ASCII WOK");
    assert!(text.starts_with("#MAXWALKMESH ASCII\n"));
    assert!(text.contains("beginwalkmeshgeom m2atilemdl1"));
    assert!(text.contains("node aabb m2atilemdl1_wg"));
    assert_eq!(first.inspection.model_resref, "m2atilemdl1");
    assert_eq!(first.inspection.faces.len(), 8);
    assert_eq!(first.inspection.aabb_tree.entries.len(), 15);

    let binary = [0_u8, 0, 0, 0, 1, 2, 3, 4];
    assert_eq!(
        inspect_ascii_tile_wok_v1(&binary).unwrap_err().code,
        "TILE-WOK-BINARY-UNSUPPORTED"
    );

    let mut cycle = navigation.aabb_tree.clone();
    cycle.entries[0].left = Some(0);
    assert_eq!(
        validate_aabb_tree_v1(&cycle, &navigation.vertices, &navigation.faces)
            .unwrap_err()
            .code,
        "TILE-AABB-CYCLE"
    );

    let mut child_oob = navigation.aabb_tree.clone();
    child_oob.entries[0].left = Some(child_oob.entries.len() as u32);
    assert_eq!(
        validate_aabb_tree_v1(&child_oob, &navigation.vertices, &navigation.faces)
            .unwrap_err()
            .code,
        "TILE-AABB-CHILD-OOB"
    );

    let leaf_indices = navigation
        .aabb_tree
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| entry.leaf_face.map(|_| index))
        .collect::<Vec<_>>();
    let mut leaf_oob = navigation.aabb_tree.clone();
    leaf_oob.entries[leaf_indices[0]].leaf_face = Some(navigation.faces.len() as u32);
    assert_eq!(
        validate_aabb_tree_v1(&leaf_oob, &navigation.vertices, &navigation.faces)
            .unwrap_err()
            .code,
        "TILE-AABB-LEAF-FACE-OOB"
    );

    let mut duplicate_face = navigation.aabb_tree.clone();
    duplicate_face.entries[leaf_indices[1]].leaf_face =
        duplicate_face.entries[leaf_indices[0]].leaf_face;
    assert_eq!(
        validate_aabb_tree_v1(&duplicate_face, &navigation.vertices, &navigation.faces)
            .unwrap_err()
            .code,
        "TILE-AABB-FACE-COVERAGE"
    );

    let mut child_bounds = navigation.aabb_tree.clone();
    let child_index = child_bounds.entries[0].left.expect("root left") as usize;
    child_bounds.entries[child_index].bounds_max[0] = child_bounds.entries[0].bounds_max[0] + 1.0;
    assert_eq!(
        validate_aabb_tree_v1(&child_bounds, &navigation.vertices, &navigation.faces)
            .unwrap_err()
            .code,
        "TILE-AABB-CHILD-BOUNDS"
    );

    let mut invalid_surface = navigation.clone();
    for face in &mut invalid_surface.faces {
        face.surface_id = 7;
    }
    assert_eq!(
        write_ascii_tile_wok_v1(&invalid_surface).unwrap_err().code,
        "TILE-WOK-NONWALK-FLOOR"
    );

    let count_mismatch = text.replacen("  verts 9", "  verts 99", 1);
    assert_eq!(
        inspect_ascii_tile_wok_v1(count_mismatch.as_bytes())
            .unwrap_err()
            .code,
        "TILE-WOK-FIELD-COUNT"
    );
    let nonfinite = text.replacen("    -5 -5 0", "    nan -5 0", 1);
    assert_eq!(
        inspect_ascii_tile_wok_v1(nonfinite.as_bytes())
            .unwrap_err()
            .code,
        "TILE-WOK-NONFINITE"
    );
    let index_oob = text.replacen("    0 1 4 ", "    0 99 4 ", 1);
    assert_eq!(
        inspect_ascii_tile_wok_v1(index_oob.as_bytes())
            .unwrap_err()
            .code,
        "TILE-WALKMESH-INDEX-OOB"
    );
}

#[test]
fn set_roundtrip_preserves_unknown_fields_and_resolves_model_bound_wok() {
    let mut tileset = minimal_static_tileset_v1(
        "m2atilesetv1",
        TileDescriptorV1::flat_v1("m2atilemdl1", "Grass", TileSurfaceV1::Grass),
        false,
    )
    .expect("minimal SET");
    tileset.document.sections[0]
        .entries
        .push(("FutureField".to_owned(), "preserved".to_owned()));

    let first = write_tileset_v1(&tileset).expect("SET write");
    let parsed = parse_tileset_v1(&first.payload).expect("SET parse");
    let second = write_tileset_v1(&parsed).expect("SET rewrite");
    let reparsed = parse_tileset_v1(&second.payload).expect("SET reparse");
    assert_eq!(parsed, reparsed);
    assert!(
        std::str::from_utf8(&second.payload)
            .unwrap()
            .contains("FutureField=preserved")
    );

    let binding = resolve_are_tile_v1(&reparsed, 0).expect("Tile_ID resolver");
    assert_eq!(binding.model_resref, "m2atilemdl1");
    assert_eq!(binding.wok_resref, "m2atilemdl1");
    assert_eq!(binding.walkmesh_class_token, "msb01");
    assert_ne!(binding.walkmesh_class_token, binding.wok_resref);

    let mut broken = first.payload.clone();
    let count = b"[TILES]\nCount=1";
    let replacement = b"[TILES]\nCount=2";
    let offset = broken
        .windows(count.len())
        .position(|window| window == count)
        .expect("TILES Count");
    broken[offset..offset + replacement.len()].copy_from_slice(replacement);
    assert_eq!(
        parse_tileset_v1(&broken).unwrap_err().code,
        "TILE-SET-TILE-COUNT-MISMATCH"
    );

    let missing_general =
        std::str::from_utf8(&first.payload)
            .unwrap()
            .replacen("[GENERAL]", "[GENERAX]", 1);
    assert_eq!(
        parse_tileset_v1(missing_general.as_bytes())
            .unwrap_err()
            .code,
        "TILE-SET-SECTION-MISSING"
    );
    let noncontiguous_tile = std::str::from_utf8(&first.payload)
        .unwrap()
        .replacen("[TILE0]", "[TILE1]", 1);
    assert_eq!(
        parse_tileset_v1(noncontiguous_tile.as_bytes())
            .unwrap_err()
            .code,
        "TILE-SET-TILE-COUNT-MISMATCH"
    );

    let grouped = format!(
        "{}\n[GROUP0]\nRows=1\nColumns=2\nTile0=0\nTile1=-1\n",
        std::str::from_utf8(&first.payload).unwrap().replacen(
            "[GROUPS]\nCount=0",
            "[GROUPS]\nCount=1",
            1
        )
    );
    let grouped = parse_tileset_v1(grouped.as_bytes()).expect("group with a -1 hole");
    assert_eq!(grouped.groups.len(), 1);
    assert_eq!(grouped.groups[0].tiles, vec![Some(0), None]);
    let grouped_roundtrip = write_tileset_v1(&grouped).expect("group roundtrip");
    assert_eq!(
        parse_tileset_v1(&grouped_roundtrip.payload).unwrap(),
        grouped
    );
    let group_oob = std::str::from_utf8(&grouped_roundtrip.payload)
        .unwrap()
        .replacen("Tile1=-1", "Tile1=1", 1);
    assert_eq!(
        parse_tileset_v1(group_oob.as_bytes()).unwrap_err().code,
        "TILE-SET-GROUP-TILE-OOB"
    );
}

#[test]
fn tile_static_profile_emits_classification_two_and_semantic_aabb() {
    let model = simple_model("m2atilemdl1", 2);
    let navigation =
        flat_tile_navigation_v1("m2atilemdl1", TileSurfaceV1::Stone).expect("navigation");
    let options = tile_options("m2atilemdl1", "m2atiletex1");

    let first = write_binary_tile_mdl_v1(&model, &navigation, &options).expect("tile binary MDL");
    let second =
        write_binary_tile_mdl_v1(&model, &navigation, &options).expect("deterministic MDL");
    assert_eq!(first.payload, second.payload);
    assert_eq!(first.inspection.model.classification, 2);
    assert_eq!(first.inspection.model.fog, 1);

    let mut nodes = Vec::new();
    fn flatten<'a>(
        nodes: &'a [m2a_core::mdl::NodeReport],
        output: &mut Vec<&'a m2a_core::mdl::NodeReport>,
    ) {
        for node in nodes {
            output.push(node);
            flatten(&node.children, output);
        }
    }
    flatten(&first.inspection.node_tree.roots, &mut nodes);
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.content_flags == 0x221)
            .count(),
        1
    );
    let aabb = nodes
        .iter()
        .find_map(|node| node.aabb.as_ref())
        .expect("semantic AABB");
    assert_eq!(aabb.entries.len(), 15);
    assert_eq!(
        aabb.entries
            .iter()
            .filter_map(|entry| entry.leaf_face)
            .collect::<std::collections::BTreeSet<_>>(),
        (0_u32..8).collect()
    );

    let aabb_node = nodes
        .iter()
        .find(|node| node.aabb.is_some())
        .expect("AABB node");
    let mut outside_core = first.payload.clone();
    let pointer_field = 12 + aabb_node.offset as usize + 0x270;
    outside_core[pointer_field..pointer_field + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(inspect_binary_mdl(&outside_core).is_err());

    let root_entry = aabb.root_pointer as usize;
    let mut binary_cycle = first.payload.clone();
    binary_cycle[12 + root_entry + 24..12 + root_entry + 28]
        .copy_from_slice(&(root_entry as u32).to_le_bytes());
    assert!(inspect_binary_mdl(&binary_cycle).is_err());

    let leaf = aabb
        .entries
        .iter()
        .find(|entry| entry.leaf_face.is_some())
        .expect("leaf");
    let mut binary_leaf_oob = first.payload.clone();
    binary_leaf_oob[12 + leaf.offset as usize + 32..12 + leaf.offset as usize + 36]
        .copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(inspect_binary_mdl(&binary_leaf_oob).is_err());

    let mut binary_invalid_bounds = first.payload.clone();
    binary_invalid_bounds[12 + root_entry + 12..12 + root_entry + 16]
        .copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(inspect_binary_mdl(&binary_invalid_bounds).is_err());
}

#[test]
fn tile_profile_accepts_exactly_twenty_thousand_render_triangles() {
    let model = simple_model("m2atilemdl1", 20_000);
    let navigation =
        flat_tile_navigation_v1("m2atilemdl1", TileSurfaceV1::Grass).expect("navigation");
    let artifact = write_binary_tile_mdl_v1(
        &model,
        &navigation,
        &tile_options("m2atilemdl1", "m2atiletex1"),
    )
    .expect("20k tile model");
    assert_eq!(artifact.report.projection.triangle_count, 20_000);
}

#[test]
fn package_builds_one_hak_and_custom_two_by_two_module_with_full_readback() {
    let identity = StaticTileIdentityV1::owner_candidate_v1();
    let request = StaticTileBuildRequestV1 {
        schema_version: 1,
        identity: identity.clone(),
        interior: false,
        terrain_name: "Grass".to_owned(),
        surface: TileSurfaceV1::Grass,
        model: simple_model(&identity.model_resref, 2),
        navigation: flat_tile_navigation_v1(&identity.model_resref, TileSurfaceV1::Grass)
            .expect("navigation"),
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: identity.texture_resref.clone(),
        }],
        textures: vec![TileTextureInputV1 {
            resref: identity.texture_resref.clone(),
            resource_type: 3,
            payload: vec![0; 18],
        }],
    };
    let first = build_static_tile_package_v1(&request).expect("tile package");
    let second = build_static_tile_package_v1(&request).expect("deterministic package");
    assert_eq!(first.hak_payload, second.hak_payload);
    assert_eq!(first.module_payload, second.module_payload);
    assert_eq!(first.report, second.report);
    assert_eq!(first.report.status, "ready_for_owner_proof");
    assert_eq!(first.report.module_file_name, "m2a_tile_static_v1.mod");
    assert_eq!(
        first.report.module_display_name,
        "Meshy2Aurora Tile Static V1"
    );
    assert_eq!(first.report.area_name, "M2A Tile Static 2x2");
    assert_eq!(first.report.tile_id, 0);
    assert_eq!(first.report.area_tile_count, 4);

    let hak = ErfArchive::parse(&first.hak_payload).expect("HAK");
    assert!(
        hak.find(&identity.tileset_resref, SET_RESOURCE_TYPE)
            .is_ok()
    );
    assert!(hak.find(&identity.model_resref, MDL_RESOURCE_TYPE).is_ok());

    let module = ErfArchive::parse(&first.module_payload).expect("MOD");
    let are = read_gff_v32(
        module
            .find(&identity.area_resref, m2a_core::tile::ARE_RESOURCE_TYPE)
            .expect("ARE"),
        &GffLimitsV1::default(),
    )
    .expect("ARE readback");
    let value = |label: &str| {
        &are.root
            .fields
            .iter()
            .find(|field| field.label == label)
            .expect("field")
            .value
    };
    assert_eq!(
        value("Tileset"),
        &GffValueV1::ResRef(identity.tileset_resref.clone())
    );
    assert_eq!(value("Width"), &GffValueV1::Int(2));
    assert_eq!(value("Height"), &GffValueV1::Int(2));
    let GffValueV1::List(tiles) = value("Tile_List") else {
        panic!("Tile_List");
    };
    assert_eq!(tiles.len(), 4);
}

#[test]
fn aabb_builder_rejects_degenerate_faces_before_allocating_a_tree() {
    let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    let faces = vec![m2a_core::walkmesh::WalkmeshFaceV1 {
        vertex_indices: [0, 1, 2],
        smoothing_group: 1,
        adjacent_faces: [-1; 3],
        surface_id: 3,
    }];
    assert_eq!(
        build_aabb_tree_v1(&vertices, &faces).unwrap_err().code,
        "TILE-AABB-DEGENERATE-FACE"
    );
}

#[test]
#[ignore = "requires M2A_TILE_RETAIL_CORPUS=1 and the read-only local NWN retail install"]
fn retail_key_bif_tile_aabb_is_read_in_place_without_copying_payloads() {
    assert_eq!(
        std::env::var("M2A_TILE_RETAIL_CORPUS").as_deref(),
        Ok("1"),
        "set M2A_TILE_RETAIL_CORPUS=1 to opt into the read-only retail test"
    );
    let key_path = std::env::var_os("M2A_REFERENCE_NWN_KEY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                r"C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\nwn_base.key",
            )
        });
    let key = fs::read(&key_path).expect("read nwn_base.key in place");
    assert_eq!(&key[0..8], b"KEY V1  ");
    assert_eq!(
        format!("{:x}", Sha256::digest(&key)),
        "09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935"
    );
    let read_u16 = |bytes: &[u8], offset: usize| {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    };
    let read_u32 = |bytes: &[u8], offset: usize| {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    };
    let bif_count = read_u32(&key, 8) as usize;
    let key_count = read_u32(&key, 12) as usize;
    let file_table_offset = read_u32(&key, 16) as usize;
    let key_table_offset = read_u32(&key, 20) as usize;
    assert_eq!(key_count, 113_489);
    let key_index = 17_910_usize;
    let entry = key_table_offset + key_index * 22;
    let resref = std::str::from_utf8(&key[entry..entry + 16])
        .unwrap()
        .trim_end_matches('\0');
    assert_eq!(resref, "tms01_c01_01");
    assert_eq!(read_u16(&key, entry + 16), MDL_RESOURCE_TYPE);
    let resource_id = read_u32(&key, entry + 18);
    let bif_index = (resource_id >> 20) as usize;
    let resource_index = (resource_id & 0x000f_ffff) as usize;
    assert!(bif_index < bif_count);

    let file_entry = file_table_offset + bif_index * 12;
    let name_offset = read_u32(&key, file_entry + 4) as usize;
    let name_size = read_u16(&key, file_entry + 8) as usize;
    let bif_name = std::str::from_utf8(&key[name_offset..name_offset + name_size])
        .unwrap()
        .trim_end_matches('\0')
        .replace('\\', "/");
    assert_eq!(bif_name, "data/aurora_tms.bif");
    let install_root = key_path
        .parent()
        .and_then(|data| data.parent())
        .expect("NWN install root");
    let bif_path = install_root.join(bif_name);
    let mut bif = File::open(&bif_path).expect("open retail BIF in place");
    let mut bif_header = [0_u8; 20];
    bif.read_exact(&mut bif_header).unwrap();
    assert_eq!(&bif_header[0..8], b"BIFFV1  ");
    let variable_count = read_u32(&bif_header, 8) as usize;
    let variable_table_offset = read_u32(&bif_header, 16) as u64;
    assert!(resource_index < variable_count);
    bif.seek(SeekFrom::Start(
        variable_table_offset + resource_index as u64 * 16,
    ))
    .unwrap();
    let mut variable = [0_u8; 16];
    bif.read_exact(&mut variable).unwrap();
    assert_eq!(read_u32(&variable, 12) as u16, MDL_RESOURCE_TYPE);
    let payload_offset = read_u32(&variable, 4) as u64;
    let payload_size = read_u32(&variable, 8) as usize;
    let mut payload = vec![0_u8; payload_size];
    bif.seek(SeekFrom::Start(payload_offset)).unwrap();
    bif.read_exact(&mut payload).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&payload)),
        "b8736f8f4b06124a8b0619cbfb0265607e16f62b098320cf8518d9aebc096c3e"
    );
    let inspection = inspect_binary_mdl(&payload).expect("retail tile binary MDL");
    assert_eq!(inspection.model.classification, 2);
    let mut stack = inspection.node_tree.roots.iter().collect::<Vec<_>>();
    let mut aabb_nodes = Vec::new();
    while let Some(node) = stack.pop() {
        stack.extend(node.children.iter());
        if node.content_flags == 0x221 {
            aabb_nodes.push(node);
        }
    }
    assert_eq!(aabb_nodes.len(), 1);
    let aabb = aabb_nodes[0].aabb.as_ref().expect("retail semantic AABB");
    assert_eq!(aabb.entries.len(), 15);
    assert_eq!(
        aabb.entries
            .iter()
            .filter_map(|entry| entry.leaf_face)
            .collect::<std::collections::BTreeSet<_>>(),
        (0_u32..8).collect()
    );
}
