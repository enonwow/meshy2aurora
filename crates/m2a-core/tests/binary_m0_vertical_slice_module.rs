use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::{GffValueV1, read_gff_v32},
    proof_module::{
        BINARY_M0_AREA_RESREF, BINARY_M0_CREATURE_TEMPLATE_RESREF, BINARY_M0_MODULE_RESREF,
        BINARY_M0_PROOF_HAK_RESREF, BinaryM0VerticalSliceIdentityV1, M0_RUNTIME_AREA_HEIGHT,
        M0_RUNTIME_AREA_WIDTH, M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y, M0_RUNTIME_ENTRY_X,
        M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z, M0_RUNTIME_FIXTURE_X, M0_RUNTIME_FIXTURE_Y,
        M0_RUNTIME_FIXTURE_Z, M0_RUNTIME_TILE_ANIMATION_LOOP, M0_RUNTIME_TILES,
        M0_RUNTIME_TILESET_RESREF, build_binary_m0_vertical_slice_module_v1,
        build_binary_m0_vertical_slice_module_with_identity_v1,
        inspect_binary_m0_vertical_slice_module_v1,
    },
};

fn field<'a>(document: &'a m2a_core::gff::GffDocumentV1, label: &str) -> &'a GffValueV1 {
    &document
        .root
        .fields
        .iter()
        .find(|candidate| candidate.label == label)
        .unwrap_or_else(|| panic!("missing field {label}"))
        .value
}

#[test]
fn binary_m0_vertical_slice_can_bind_a_fresh_module_area_and_hak_identity() {
    let identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: "m2a_bm0v10".to_owned(),
        area_resref: "m2a_bm0a10".to_owned(),
        hak_resref: "m2a_m0v10".to_owned(),
    };
    let artifact = build_binary_m0_vertical_slice_module_with_identity_v1(848, &identity).unwrap();
    assert_eq!(artifact.report.module_resref, identity.module_resref);
    assert_eq!(artifact.report.area_resref, identity.area_resref);
    assert_eq!(artifact.report.hak_resref, identity.hak_resref);

    let archive = ErfArchive::parse(&artifact.payload).unwrap();
    let ifo = read_gff_v32(archive.find("module", 2014).unwrap(), &Default::default()).unwrap();
    assert_eq!(
        field(&ifo, "Mod_Entry_Area"),
        &GffValueV1::ResRef(identity.area_resref.clone())
    );
    assert!(matches!(field(&ifo, "Mod_HakList"), GffValueV1::List(items)
        if items.len() == 1
            && items[0].fields.iter().any(|candidate| candidate.label == "Mod_Hak"
                && candidate.value == GffValueV1::String(identity.hak_resref.as_bytes().to_vec()))));
    assert!(archive.find(&identity.area_resref, 2012).is_ok());
    assert!(archive.find(&identity.area_resref, 2023).is_ok());
}

#[test]
fn binary_m0_vertical_slice_module_is_bound_to_the_valid_two_by_two_area_contract() {
    let artifact = build_binary_m0_vertical_slice_module_v1(848).unwrap();

    let runtime_fixture = artifact
        .report
        .binary_m0_runtime_fixture
        .as_ref()
        .expect("binary M0 report must carry the read-back runtime fixture");
    assert_eq!(
        inspect_binary_m0_vertical_slice_module_v1(&artifact.payload).unwrap(),
        *runtime_fixture,
        "the capture binding must be reconstructed from emitted MOD bytes"
    );
    assert_eq!(runtime_fixture.module_resref, BINARY_M0_MODULE_RESREF);
    assert_eq!(runtime_fixture.area_resref, BINARY_M0_AREA_RESREF);
    assert_eq!(
        runtime_fixture.ordered_hak_resrefs,
        [BINARY_M0_PROOF_HAK_RESREF]
    );
    assert_eq!(runtime_fixture.entry_position.x, M0_RUNTIME_ENTRY_X);
    assert_eq!(runtime_fixture.entry_position.y, M0_RUNTIME_ENTRY_Y);
    assert_eq!(runtime_fixture.entry_direction.x, M0_RUNTIME_ENTRY_DIR_X);
    assert_eq!(runtime_fixture.entry_direction.y, M0_RUNTIME_ENTRY_DIR_Y);
    assert_eq!(
        runtime_fixture.fixture.template_resref,
        BINARY_M0_CREATURE_TEMPLATE_RESREF
    );
    assert_eq!(runtime_fixture.fixture.appearance_row, 848);
    assert_eq!(runtime_fixture.fixture.position.x, M0_RUNTIME_FIXTURE_X);
    assert_eq!(runtime_fixture.fixture.position.y, M0_RUNTIME_FIXTURE_Y);

    assert_eq!(artifact.report.module_resref, BINARY_M0_MODULE_RESREF);
    assert_eq!(artifact.report.area_resref, BINARY_M0_AREA_RESREF);
    assert_eq!(artifact.report.hak_resref, BINARY_M0_PROOF_HAK_RESREF);
    assert_eq!(artifact.report.appearance_row, 848);
    assert_eq!(artifact.report.semantic_readback_status, "PASS");

    let archive = ErfArchive::parse(&artifact.payload).unwrap();
    assert_eq!(archive.file_type(), ErfFileType::Module);

    let ifo = read_gff_v32(archive.find("module", 2014).unwrap(), &Default::default()).unwrap();
    assert_eq!(
        field(&ifo, "Mod_Entry_Area"),
        &GffValueV1::ResRef(BINARY_M0_AREA_RESREF.to_owned())
    );
    assert_eq!(
        field(&ifo, "Mod_Entry_X"),
        &GffValueV1::Float(M0_RUNTIME_ENTRY_X)
    );
    assert_eq!(
        field(&ifo, "Mod_Entry_Y"),
        &GffValueV1::Float(M0_RUNTIME_ENTRY_Y)
    );
    assert_eq!(
        field(&ifo, "Mod_Entry_Z"),
        &GffValueV1::Float(M0_RUNTIME_ENTRY_Z)
    );
    assert_eq!(
        field(&ifo, "Mod_Entry_Dir_X"),
        &GffValueV1::Float(M0_RUNTIME_ENTRY_DIR_X)
    );
    assert_eq!(
        field(&ifo, "Mod_Entry_Dir_Y"),
        &GffValueV1::Float(M0_RUNTIME_ENTRY_DIR_Y)
    );
    assert!(matches!(field(&ifo, "Mod_HakList"), GffValueV1::List(items)
        if items.len() == 1
            && items[0].fields.iter().any(|candidate| candidate.label == "Mod_Hak"
                && candidate.value == GffValueV1::String(BINARY_M0_PROOF_HAK_RESREF.as_bytes().to_vec()))));

    let are = read_gff_v32(
        archive.find(BINARY_M0_AREA_RESREF, 2012).unwrap(),
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        field(&are, "Width"),
        &GffValueV1::Int(M0_RUNTIME_AREA_WIDTH)
    );
    assert_eq!(
        field(&are, "Height"),
        &GffValueV1::Int(M0_RUNTIME_AREA_HEIGHT)
    );
    assert_eq!(
        field(&are, "Tileset"),
        &GffValueV1::ResRef(M0_RUNTIME_TILESET_RESREF.to_owned())
    );
    let GffValueV1::List(tiles) = field(&are, "Tile_List") else {
        panic!("Tile_List must be a GFF list");
    };
    assert_eq!(tiles.len(), M0_RUNTIME_TILES.len());
    for (tile, (expected_id, expected_orientation)) in tiles.iter().zip(M0_RUNTIME_TILES) {
        assert!(
            tile.fields
                .iter()
                .any(|candidate| candidate.label == "Tile_ID"
                    && candidate.value == GffValueV1::Int(expected_id))
        );
        assert!(
            tile.fields
                .iter()
                .any(|candidate| candidate.label == "Tile_Orientation"
                    && candidate.value == GffValueV1::Int(expected_orientation))
        );
        for label in ["Tile_AnimLoop1", "Tile_AnimLoop2", "Tile_AnimLoop3"] {
            assert!(tile.fields.iter().any(|candidate| candidate.label == label
                && candidate.value == GffValueV1::Byte(M0_RUNTIME_TILE_ANIMATION_LOOP)));
        }
    }

    let git = read_gff_v32(
        archive.find(BINARY_M0_AREA_RESREF, 2023).unwrap(),
        &Default::default(),
    )
    .unwrap();
    let GffValueV1::List(creatures) = field(&git, "Creature List") else {
        panic!("Creature List must be a GFF list");
    };
    assert_eq!(creatures.len(), 1);
    let creature = &creatures[0];
    assert!(
        creature
            .fields
            .iter()
            .any(|candidate| candidate.label == "TemplateResRef"
                && candidate.value
                    == GffValueV1::ResRef(BINARY_M0_CREATURE_TEMPLATE_RESREF.to_owned()))
    );
    assert!(
        creature
            .fields
            .iter()
            .any(|candidate| candidate.label == "Appearance_Type"
                && candidate.value == GffValueV1::Word(848))
    );
    assert!(
        creature
            .fields
            .iter()
            .any(|candidate| candidate.label == "XPosition"
                && candidate.value == GffValueV1::Float(M0_RUNTIME_FIXTURE_X))
    );
    assert!(
        creature
            .fields
            .iter()
            .any(|candidate| candidate.label == "YPosition"
                && candidate.value == GffValueV1::Float(M0_RUNTIME_FIXTURE_Y))
    );
    assert!(
        creature
            .fields
            .iter()
            .any(|candidate| candidate.label == "ZPosition"
                && candidate.value == GffValueV1::Float(M0_RUNTIME_FIXTURE_Z))
    );
}
