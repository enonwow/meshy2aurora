use m2a_core::{
    erf::ErfArchive,
    gff::{GffFileTypeV1, GffLimitsV1, GffValueV1, read_gff_v32},
    item::{
        ARMOR_PART_FIELDS_V1, CAPART_REQUIRED_REFERENCE_TABLES_V1, ITEM_COLOR_FIELDS_V1,
        ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1, ItemBlueprintV1, ItemCapartContextV1,
        ItemColorValuesV1, ItemCompositionProfileV1, ItemEquippedProofIdentityV2,
        ItemEquippedProofProfileV2, ItemIconProfileV1, ItemPartBuildOptionsV2,
        ItemPartTextureEncodingV1, ItemPartTransformV1, ItemPartValueV1, ItemProofModuleIdentityV1,
        ItemProofPlacementV1, ItemPropertyV1, ItemResourceInventoryEntryV2,
        ItemResourceProvenanceV1, ItemTextureProfileV1, build_item_equipped_proof_module_v2,
        build_item_proof_module_v1, build_meshy_item_part_v1,
        build_meshy_item_part_with_options_v2, inspect_item_baseitems_v1,
        inspect_item_reference_resource_v1, inspect_item_reference_two_da_v1,
        measure_meshy_item_seam_v1, resolve_item_baseitem_v1, resolve_item_capability_v1,
        resolve_item_capart_part_v1, resolve_item_capart_part_v2, resolve_item_cast_spell_icon_v1,
        resolve_item_cloak_v2, resolve_item_cloak_v3, resolve_item_equipped_appearance_v1,
        resolve_item_part_resource_v1, validate_item_triangle_budget_v1, write_item_uti_v1,
    },
    owned_fixture::synthetic_owned_m6_glb_v1,
};
use sha2::{Digest, Sha256};

#[path = "fixtures/build_synthetic_glb.rs"]
mod build_synthetic_glb;

const BASEITEMS: &[u8] = br#"2DA V2.0

Label Name ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight
0 single **** Ring 0 0 it_bag iring 8 1 1
1 colored **** Helm 1 0 it_bag ihelm 1 2 2
2 segmented **** WSwLs 2 0 it_bag iwswls 16 1 4
3 armor **** Armor 3 1 gifp iit_chest 2 2 3
43 DELETED **** **** * **** it_bag **** 0 1 1
83 padding **** **** **** **** **** **** **** **** ****
"#;

fn resource_provenance(bytes: &[u8], locator: &str) -> ItemResourceProvenanceV1 {
    ItemResourceProvenanceV1 {
        schema_version: 1,
        source_kind: "NWN_BASE_KEY".to_owned(),
        source_container_file_name: "nwn_base.key".to_owned(),
        source_container_sha256: ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1.to_owned(),
        resource_locator: locator.to_owned(),
        expected_sha256: format!("{:x}", Sha256::digest(bytes)),
        manifest_sha256: "b".repeat(64),
    }
}

fn validated_reference_resource(
    resource_type: u16,
    resref: &str,
    bytes: &[u8],
    locator: &str,
) -> ItemResourceInventoryEntryV2 {
    inspect_item_reference_resource_v1(
        resource_type,
        resref,
        bytes,
        &resource_provenance(bytes, locator),
    )
    .unwrap()
}

fn one_pixel_retail_plt() -> Vec<u8> {
    let mut bytes = b"PLT V1  ".to_vec();
    bytes.extend_from_slice(&10_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

fn reference_mdl(resref: &str) -> Vec<u8> {
    let source = build_synthetic_glb::mutate_json(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);
        root["nodes"] = serde_json::json!([{
            "name": "retail-reference-root",
            "mesh": 0
        }]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .unwrap();
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");
    });
    build_meshy_item_part_v1(
        &source,
        resref,
        "reference_tex",
        ItemPartTransformV1::default(),
    )
    .unwrap()
    .mdl_payload
}

#[test]
fn baseitem_catalog_derives_four_model_types_without_usage_categories() {
    let catalog = inspect_item_baseitems_v1(BASEITEMS).unwrap();

    assert_eq!(catalog.rows.len(), 4);
    assert_eq!(catalog.physical_row_count, 6);
    assert_eq!(catalog.inactive_row_count, 2);
    assert_eq!(
        catalog
            .rows
            .iter()
            .map(|row| row.part_slots.len())
            .collect::<Vec<_>>(),
        [1, 1, 3, 19]
    );
    assert_eq!(
        catalog
            .rows
            .iter()
            .map(|row| row.color_fields.len())
            .collect::<Vec<_>>(),
        [0, 6, 0, 6]
    );
    assert_eq!(catalog.rows[2].part_slots[0].field, "ModelPart1");
    assert_eq!(catalog.rows[2].part_slots[0].token.as_deref(), Some("b"));
    assert_eq!(catalog.rows[2].part_slots[1].field, "ModelPart2");
    assert_eq!(catalog.rows[2].part_slots[1].token.as_deref(), Some("m"));
    assert_eq!(catalog.rows[2].part_slots[2].field, "ModelPart3");
    assert_eq!(catalog.rows[2].part_slots[2].token.as_deref(), Some("t"));
    assert_eq!(catalog.rows[2].default_model.as_deref(), Some("it_bag"));
    assert_eq!(catalog.rows[2].default_icon.as_deref(), Some("iwswls"));
    assert_eq!(
        catalog.rows[3]
            .part_slots
            .iter()
            .map(|slot| slot.field.as_str())
            .collect::<Vec<_>>(),
        ARMOR_PART_FIELDS_V1
    );
    assert_eq!(
        catalog.rows[3]
            .color_fields
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ITEM_COLOR_FIELDS_V1
    );
}

#[test]
fn selected_baseitem_is_resolved_by_printed_row_label_not_physical_position() {
    let source = br#"2DA V2.0

Label ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight
77 chosen WSwLs 2 0 it_bag iwswls 16 1 4
"#;
    let row = resolve_item_baseitem_v1(source, 77).unwrap();

    assert_eq!(row.base_item, 77);
    assert_eq!(row.label, "chosen");
    assert_eq!(row.model_type, 2);
    assert_eq!(row.part_slots.len(), 3);
    assert!(resolve_item_baseitem_v1(source, 0).is_err());
}

#[test]
fn three_part_resource_names_follow_aurora_model_and_icon_namespaces() {
    let row = resolve_item_baseitem_v1(BASEITEMS, 2).unwrap();
    let bottom = resolve_item_part_resource_v1(&row, "ModelPart1", 23, None, None).unwrap();

    assert_eq!(bottom.model_resref, "wswls_b_023");
    assert_eq!(bottom.icon_resref, "iwswls_b_023");
    assert_eq!(bottom.variant, 23);
}

#[test]
fn capart_armor_is_a_reference_composer_while_gender_specific_meshes_need_resrefs() {
    let armor = resolve_item_baseitem_v1(BASEITEMS, 3).unwrap();
    let error =
        resolve_item_part_resource_v1(&armor, "ArmorPart_Torso", 43, None, None).unwrap_err();
    assert_eq!(error.code, "ITEM-REFERENCE-COMPOSER-REQUIRED");

    let mut gender_specific = resolve_item_baseitem_v1(BASEITEMS, 0).unwrap();
    gender_specific.gender_specific = true;
    gender_specific.part_slots[0].requires_explicit_resource_resrefs = true;
    let resolved = resolve_item_part_resource_v1(
        &gender_specific,
        "ModelPart1",
        43,
        Some("p_hhm_chest043"),
        Some("ip_hhm_chest043"),
    )
    .unwrap();
    assert_eq!(resolved.model_resref, "p_hhm_chest043");
    assert_eq!(resolved.icon_resref, "ip_hhm_chest043");
}

#[test]
fn uti_is_numeric_for_parts_colors_and_item_fields_and_roundtrips() {
    let row = resolve_item_baseitem_v1(BASEITEMS, 2).unwrap();
    let artifact = write_item_uti_v1(
        &row,
        &ItemBlueprintV1 {
            schema_version: 1,
            template_resref: "m2a_lsword01".to_owned(),
            tag: "M2A_LSWORD01".to_owned(),
            localized_name: "Meshy Longsword".to_owned(),
            description: "Generated item".to_owned(),
            identified_description: "Generated item".to_owned(),
            comment: "Meshy2Aurora Item V1".to_owned(),
            parts: vec![
                ItemPartValueV1 {
                    field: "ModelPart1".to_owned(),
                    value: 251,
                },
                ItemPartValueV1 {
                    field: "ModelPart2".to_owned(),
                    value: 252,
                },
                ItemPartValueV1 {
                    field: "ModelPart3".to_owned(),
                    value: 253,
                },
            ],
            properties: vec![ItemPropertyV1 {
                property_name: 15,
                subtype: 37,
                cost_table: 3,
                cost_value: 9,
                param1: 255,
                param1_value: 0,
                chance_appear: 100,
            }],
            colors: ItemColorValuesV1::default(),
            cost: 100,
            add_cost: 0,
            charges: 0,
            stack_size: 1,
            palette_id: 0,
            identified: true,
            stolen: false,
            cursed: false,
            plot: false,
        },
    )
    .unwrap();
    let document = read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap();

    assert_eq!(document.file_type, GffFileTypeV1::Uti);
    let value = |label: &str| {
        document
            .root
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    assert_eq!(value("BaseItem"), Some(&GffValueV1::Int(2)));
    assert_eq!(value("ModelPart1"), Some(&GffValueV1::Byte(251)));
    assert_eq!(value("ModelPart2"), Some(&GffValueV1::Byte(252)));
    assert_eq!(value("ModelPart3"), Some(&GffValueV1::Byte(253)));
    assert_eq!(value("Cost"), Some(&GffValueV1::Dword(100)));
    assert_eq!(value("Charges"), Some(&GffValueV1::Byte(0)));
    assert_eq!(value("StackSize"), Some(&GffValueV1::Word(1)));
    assert_eq!(value("Identified"), Some(&GffValueV1::Byte(1)));
    let properties = match value("PropertiesList") {
        Some(GffValueV1::List(values)) => values,
        other => panic!("expected PropertiesList, got {other:?}"),
    };
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0].struct_id, 0);
    let property_value = |label: &str| {
        properties[0]
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    assert_eq!(property_value("PropertyName"), Some(&GffValueV1::Word(15)));
    assert_eq!(property_value("Subtype"), Some(&GffValueV1::Word(37)));
    assert_eq!(property_value("CostTable"), Some(&GffValueV1::Byte(3)));
    assert_eq!(property_value("CostValue"), Some(&GffValueV1::Word(9)));
    assert_eq!(property_value("Param1"), Some(&GffValueV1::Byte(255)));
    assert_eq!(property_value("Param1Value"), Some(&GffValueV1::Byte(0)));
    assert_eq!(property_value("ChanceAppear"), Some(&GffValueV1::Byte(100)));
    assert_eq!(artifact.report.base_item, 2);
    assert_eq!(artifact.report.part_count, 3);
    assert_eq!(artifact.report.semantic_readback_status, "PASS");
}

#[test]
fn uti_requires_exact_profile_fields_once_each() {
    let row = resolve_item_baseitem_v1(BASEITEMS, 2).unwrap();
    let blueprint = ItemBlueprintV1 {
        schema_version: 1,
        template_resref: "m2a_bad".to_owned(),
        tag: "M2A_BAD".to_owned(),
        localized_name: "Bad".to_owned(),
        description: String::new(),
        identified_description: String::new(),
        comment: String::new(),
        parts: vec![ItemPartValueV1 {
            field: "ModelPart1".to_owned(),
            value: 1,
        }],
        properties: Vec::new(),
        colors: ItemColorValuesV1::default(),
        cost: 0,
        add_cost: 0,
        charges: 0,
        stack_size: 1,
        palette_id: 0,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
    };

    let error = write_item_uti_v1(&row, &blueprint).unwrap_err();
    assert_eq!(error.code, "ITEM-UTI-PARTS-INCOMPLETE");
}

#[test]
fn uti_requires_exact_color_profile_instead_of_silent_defaults() {
    let base_blueprint = ItemBlueprintV1 {
        schema_version: 1,
        template_resref: "m2a_colors".to_owned(),
        tag: "M2A_COLORS".to_owned(),
        localized_name: "Colors".to_owned(),
        description: String::new(),
        identified_description: String::new(),
        comment: String::new(),
        parts: vec![ItemPartValueV1 {
            field: "ModelPart1".to_owned(),
            value: 7,
        }],
        properties: Vec::new(),
        colors: ItemColorValuesV1::default(),
        cost: 0,
        add_cost: 0,
        charges: 0,
        stack_size: 1,
        palette_id: 0,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
    };
    let colored = resolve_item_baseitem_v1(BASEITEMS, 1).unwrap();
    let error = write_item_uti_v1(&colored, &base_blueprint).unwrap_err();
    assert_eq!(error.code, "ITEM-UTI-COLORS-INCOMPLETE");

    let single = resolve_item_baseitem_v1(BASEITEMS, 0).unwrap();
    let mut unexpected = base_blueprint.clone();
    unexpected.colors.leather1_color = Some(1);
    let error = write_item_uti_v1(&single, &unexpected).unwrap_err();
    assert_eq!(error.code, "ITEM-UTI-COLORS-UNEXPECTED");

    let mut complete = base_blueprint;
    complete.colors = ItemColorValuesV1 {
        leather1_color: Some(1),
        leather2_color: Some(2),
        cloth1_color: Some(3),
        cloth2_color: Some(4),
        metal1_color: Some(5),
        metal2_color: Some(6),
    };
    let artifact = write_item_uti_v1(&colored, &complete).unwrap();
    let document = read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap();
    for (label, value) in ITEM_COLOR_FIELDS_V1.into_iter().zip(1u8..=6) {
        assert!(
            document
                .root
                .fields
                .iter()
                .any(|field| field.label == label && field.value == GffValueV1::Byte(value))
        );
    }
}

#[test]
fn capability_matrix_distinguishes_static_plt_modular_and_capart_composers() {
    let plain = resolve_item_baseitem_v1(BASEITEMS, 0).unwrap();
    let colored = resolve_item_baseitem_v1(BASEITEMS, 1).unwrap();
    let modular = resolve_item_baseitem_v1(BASEITEMS, 2).unwrap();
    let armor = resolve_item_baseitem_v1(BASEITEMS, 3).unwrap();

    let plain_capability = resolve_item_capability_v1(&plain);
    assert_eq!(
        plain_capability.composition_profile,
        ItemCompositionProfileV1::SinglePart
    );
    assert_eq!(
        plain_capability.texture_profile,
        ItemTextureProfileV1::DirectColor
    );

    let colored_capability = resolve_item_capability_v1(&colored);
    assert_eq!(
        colored_capability.composition_profile,
        ItemCompositionProfileV1::SinglePart
    );
    assert_eq!(
        colored_capability.texture_profile,
        ItemTextureProfileV1::PaletteLayers
    );

    let modular_capability = resolve_item_capability_v1(&modular);
    assert_eq!(
        modular_capability.composition_profile,
        ItemCompositionProfileV1::BottomMiddleTop
    );
    assert_eq!(
        modular_capability.texture_profile,
        ItemTextureProfileV1::DirectColor
    );

    let armor_capability = resolve_item_capability_v1(&armor);
    assert_eq!(
        armor_capability.composition_profile,
        ItemCompositionProfileV1::CapartArmor
    );
    assert_eq!(
        armor_capability.texture_profile,
        ItemTextureProfileV1::CapartPaletteLayers
    );
    assert_eq!(armor_capability.meshy_source_count, 0);
    assert_eq!(
        armor_capability.required_reference_tables,
        CAPART_REQUIRED_REFERENCE_TABLES_V1
    );
}

#[test]
fn capart_composer_requires_native_mapping_part_ranges_and_robe_masks() {
    let capart = br#"2DA V2.0

NAME MDLNAME NODENAME
0 0 FOOTR rfoot_g
1 0 FOOTL lfoot_g
2 0 SHINR rshin_g
3 0 SHINL lshin_g
4 0 LEGL lthigh_g
5 0 LEGR rthigh_g
6 0 PELVIS pelvis_g
7 0 CHEST torso_g
8 0 BELT belt_g
9 0 NECK neck_g
10 0 FORER rforearm_g
11 0 FOREL lforearm_g
12 0 BICEPR rbicep_g
13 0 BICEPL lbicep_g
14 0 SHOR rshoulder_g
15 0 SHOL lshoulder_g
16 0 HANDR rhand_g
17 0 HANDL lhand_g
18 0 ROBE root
"#;
    let foot = br#"2DA V2.0

COSTMODIFIER ACBONUS
0 0 0.00
1 0 0.10
"#;
    let resolved =
        resolve_item_capart_part_v1("ArmorPart_RFoot", 1, capart, "PARTS_FOOT", foot).unwrap();
    assert_eq!(resolved.capart_row, 0);
    assert_eq!(resolved.mdl_name, "FOOTR");
    assert_eq!(resolved.node_name, "rfoot_g");
    assert_eq!(resolved.parts_table, "PARTS_FOOT");
    assert_eq!(resolved.available_part_count, 2);
    assert!(resolved.robe_hidden_mdl_names.is_empty());

    let out_of_range =
        resolve_item_capart_part_v1("ArmorPart_RFoot", 2, capart, "PARTS_FOOT", foot).unwrap_err();
    assert_eq!(out_of_range.code, "ITEM-CAPART-SELECTOR-OUT-OF-RANGE");

    let mut wrong_capart = capart.to_vec();
    let footr_offset = wrong_capart
        .windows(b"FOOTR".len())
        .position(|window| window == b"FOOTR")
        .unwrap();
    wrong_capart[footr_offset..footr_offset + 5].copy_from_slice(b"CHEST");
    let mapping_error =
        resolve_item_capart_part_v1("ArmorPart_RFoot", 1, &wrong_capart, "PARTS_FOOT", foot)
            .unwrap_err();
    assert_eq!(mapping_error.code, "ITEM-CAPART-MAPPING-MISMATCH");

    let mut wrong_node = capart.to_vec();
    let node_offset = wrong_node
        .windows(b"rfoot_g".len())
        .position(|window| window == b"rfoot_g")
        .unwrap();
    wrong_node[node_offset..node_offset + 7].copy_from_slice(b"torso_g");
    let node_error =
        resolve_item_capart_part_v1("ArmorPart_RFoot", 1, &wrong_node, "PARTS_FOOT", foot)
            .unwrap_err();
    assert_eq!(node_error.code, "ITEM-CAPART-MAPPING-MISMATCH");
    assert!(node_error.path.ends_with("NODENAME"));

    let robe = br#"2DA V2.0

COSTMODIFIER ACBONUS HIDEFOOTR HIDEFOOTL HIDESHINR HIDESHINL HIDELEGR HIDELEGL HIDEPELVIS HIDECHEST HIDEBELT HIDENECK HIDEFORER HIDEFOREL HIDEBICEPR HIDEBICEPL HIDESHOR HIDESHOL HIDEHANDR HIDEHANDL HIDEHEAD
0 0 0.00 0 0 1 1 1 1 1 1 1 0 1 1 1 1 1 1 0 0 0
"#;
    let robe =
        resolve_item_capart_part_v1("ArmorPart_Robe", 0, capart, "PARTS_ROBE", robe).unwrap();
    assert_eq!(
        robe.robe_hidden_mdl_names,
        [
            "SHINR", "SHINL", "LEGR", "LEGL", "PELVIS", "CHEST", "BELT", "FORER", "FOREL",
            "BICEPR", "BICEPL", "SHOR", "SHOL"
        ]
    );
}

#[test]
fn capart_v2_resolves_contextual_model_bytes_and_skips_robe_hidden_parts() {
    let capart = br#"2DA V2.0

NAME MDLNAME NODENAME
0 0 FOOTR rfoot_g
1 0 FOOTL lfoot_g
2 0 SHINR rshin_g
3 0 SHINL lshin_g
4 0 LEGL lthigh_g
5 0 LEGR rthigh_g
6 0 PELVIS pelvis_g
7 0 CHEST torso_g
8 0 BELT belt_g
9 0 NECK neck_g
10 0 FORER rforearm_g
11 0 FOREL lforearm_g
12 0 BICEPR rbicep_g
13 0 BICEPL lbicep_g
14 0 SHOR rshoulder_g
15 0 SHOL lshoulder_g
16 0 HANDR rhand_g
17 0 HANDL lhand_g
18 0 ROBE root
"#;
    let foot = b"2DA V2.0\n\nCOSTMODIFIER ACBONUS\n0 0 0.00\n1 0 0.10\n";
    let model = reference_mdl("pmh0_footr001");
    let palette = one_pixel_retail_plt();
    let inventory = [
        validated_reference_resource(2002, "pmh0_footr001", &model, "bif:1:resource:2"),
        validated_reference_resource(6, "pmh0_footr001", &palette, "bif:1:resource:3"),
    ];
    let context = ItemCapartContextV1 {
        schema_version: 1,
        model_prefix: "pmh0".to_owned(),
        gender_code: None,
    };
    let resolved = resolve_item_capart_part_v2(
        "ArmorPart_RFoot",
        1,
        capart,
        "PARTS_FOOT",
        foot,
        &context,
        &inventory,
        false,
    )
    .unwrap();
    assert_eq!(resolved.model_candidates, ["pmh0_footr001"]);
    assert_eq!(
        resolved.resolved_model_resref.as_deref(),
        Some("pmh0_footr001")
    );
    assert_eq!(resolved.model_resource.as_ref(), Some(&inventory[0]));
    assert_eq!(resolved.palette_resource.as_ref(), Some(&inventory[1]));
    assert_eq!(
        resolved.resource_verification,
        "PINNED_MANIFEST_PAYLOAD_VALIDATED"
    );

    let hidden = resolve_item_capart_part_v2(
        "ArmorPart_RFoot",
        1,
        capart,
        "PARTS_FOOT",
        foot,
        &context,
        &[],
        true,
    )
    .unwrap();
    assert!(hidden.resolved_model_resref.is_none());
    assert_eq!(hidden.resource_verification, "SKIPPED_BY_ROBE_MASK");
}

#[test]
fn cloak_v2_binds_the_selected_row_to_exact_existing_model_and_icon_resources() {
    let cloak_model = br#"2DA V2.0

LABEL MODEL TEXTURE ICON
0 **** 0 0 0
1 Plain 1 1 1
2 Arcane 1 10 2
"#;
    let resources = ["2002:pmh0_cloak_001", "6:cloak_010", "6:icloak_m_002"];
    let resolved = resolve_item_cloak_v2(2, cloak_model, &resources).unwrap();
    assert_eq!(resolved.model_resref, "pmh0_cloak_001");
    assert_eq!(resolved.texture_resref, "cloak_010");
    assert_eq!(resolved.icon_resref, "icloak_m_002");
    assert_eq!(resolved.resource_verification, "RESOURCE_KEYS_VERIFIED");

    let missing = resolve_item_cloak_v2(2, cloak_model, &["2002:pmh0_cloak_001"]).unwrap_err();
    assert_eq!(missing.code, "ITEM-CLOAK-RESOURCE-MISSING");
}

#[test]
fn cloak_v3_requires_validated_mdl_and_both_retail_plt_resources() {
    let cloak_model = br#"2DA V2.0

LABEL MODEL TEXTURE ICON
0 **** 0 0 0
1 Plain 1 1 1
2 Arcane 1 10 2
"#;
    let model = reference_mdl("pmh0_cloak_001");
    let texture = one_pixel_retail_plt();
    let icon = one_pixel_retail_plt();
    let inventory = [
        validated_reference_resource(2002, "pmh0_cloak_001", &model, "bif:1:resource:2"),
        validated_reference_resource(6, "cloak_010", &texture, "bif:1:resource:3"),
        validated_reference_resource(6, "icloak_m_002", &icon, "bif:1:resource:4"),
    ];
    let resolved = resolve_item_cloak_v3(2, cloak_model, &inventory).unwrap();
    assert_eq!(resolved.model_resource, inventory[0]);
    assert_eq!(resolved.texture_resource, inventory[1]);
    assert_eq!(resolved.icon_resource, inventory[2]);
    assert_eq!(resolved.inventory_sha256.len(), 64);
    assert_eq!(
        resolved.resource_verification,
        "PINNED_MANIFEST_PAYLOAD_VALIDATED"
    );

    let mut invalid = inventory.clone();
    invalid[1].provenance.expected_sha256 = "0".repeat(64);
    let error = resolve_item_cloak_v3(2, cloak_model, &invalid).unwrap_err();
    assert_eq!(error.code, "ITEM-RESOURCE-INVENTORY-ENTRY-INVALID");
}

#[test]
fn reference_resource_inspection_rejects_forged_payloads_and_manifest_hashes() {
    let fake_mdl = [0x4d, 0x44, 0x4c, 1];
    let provenance = resource_provenance(&fake_mdl, "bif:1:resource:2");
    let error = inspect_item_reference_resource_v1(2002, "pmh0_footr001", &fake_mdl, &provenance)
        .unwrap_err();
    assert_eq!(error.code, "ITEM-RESOURCE-MDL-INVALID");

    let plt = one_pixel_retail_plt();
    let mut wrong_hash = resource_provenance(&plt, "bif:1:resource:3");
    wrong_hash.expected_sha256 = "0".repeat(64);
    let error =
        inspect_item_reference_resource_v1(6, "pmh0_footr001", &plt, &wrong_hash).unwrap_err();
    assert_eq!(error.code, "ITEM-RESOURCE-PAYLOAD-HASH-MISMATCH");
}

#[test]
fn equipped_appearance_derives_the_exact_player_model_prefix_from_the_selected_row() {
    let appearance = br#"2DA V2.0

LABEL RACE MODELTYPE RACIALTYPE
0 Dwarf D P 0
1 Elf E P 1
2 Gnome G P 2
3 Halfling A P 3
4 Half_Elf H P 4
5 Half_Orc O P 5
6 Human H P 6
"#;
    let binding = resolve_item_equipped_appearance_v1(appearance, 6, 6, 0, 0).unwrap();
    assert_eq!(binding.model_prefix, "pmh0");
    assert_eq!(binding.model_type, "P");
    assert_eq!(binding.race_token, "h");
    assert_eq!(binding.appearance_table_sha256.len(), 64);

    let mismatch = resolve_item_equipped_appearance_v1(appearance, 6, 5, 0, 0).unwrap_err();
    assert_eq!(
        mismatch.code,
        "ITEM-EQUIPPED-APPEARANCE-RACIALTYPE-MISMATCH"
    );
}

#[test]
fn capability_matrix_marks_spell_icons_and_cloak_table_resolution() {
    let mut spell = resolve_item_baseitem_v1(BASEITEMS, 0).unwrap();
    spell.base_item = 54;
    assert_eq!(
        resolve_item_capability_v1(&spell).icon_profile,
        ItemIconProfileV1::IprpSpell
    );

    let mut cloak = resolve_item_baseitem_v1(BASEITEMS, 1).unwrap();
    cloak.base_item = 80;
    cloak.item_class = "cloak".to_owned();
    let capability = resolve_item_capability_v1(&cloak);
    assert_eq!(
        capability.composition_profile,
        ItemCompositionProfileV1::CloakModel
    );
    assert_eq!(capability.icon_profile, ItemIconProfileV1::CloakModel);
    assert_eq!(
        capability.required_reference_tables,
        ["CloakModel", "Appearance"]
    );
}

#[test]
fn item_reference_reader_only_strips_terminal_blank_physical_rows() {
    let source = br#"2DA V2.0

Label Name CasterLvl InnateLvl Cost SpellIndex PotionUse WandUse GeneralUse Icon
0 None 0 0 0 0 0 0 0 0 iss_none
1 Aid 1098 3 2 4500 1 1 1 1 iss_Aid

"#;
    let inspection = inspect_item_reference_two_da_v1("IPRP_SPELLS", source).unwrap();
    assert_eq!(inspection.source_byte_length, source.len() as u64);
    assert_eq!(inspection.stripped_trailing_blank_row_count, 1);
    assert_eq!(inspection.inspection.physical_row_count, 2);
    assert_ne!(
        inspection.source_sha256, inspection.inspection.source_sha256,
        "the report must distinguish exact source bytes from normalized parse bytes"
    );

    let property = ItemPropertyV1 {
        property_name: 15,
        subtype: 1,
        cost_table: 0,
        cost_value: 0,
        param1: 255,
        param1_value: 0,
        chance_appear: 100,
    };
    let resolution = resolve_item_cast_spell_icon_v1(54, &[property], source).unwrap();
    assert_eq!(resolution.icon_resref, "iss_aid");
}

#[test]
fn item_part_uses_its_own_static_profile_and_bakes_transform_to_root_controllers() {
    let source = build_synthetic_glb::mutate_json(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);
        root["nodes"] = serde_json::json!([{
            "name": "item-part-source-root",
            "mesh": 0
        }]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .unwrap();
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");
    });
    let artifact = build_meshy_item_part_v1(
        &source,
        "wswls_b_251",
        "m2alsbt1",
        ItemPartTransformV1 {
            translation: [1.0, 2.0, 3.0],
            rotation_xyzw: [
                0.0,
                0.0,
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ],
            uniform_scale: 1.0,
            pivot: [0.0; 3],
        },
    )
    .unwrap();

    assert_eq!(artifact.report.profile, "ITEM_PART_STATIC_RIGID");
    assert!(artifact.report.triangle_count > 0);
    assert_eq!(artifact.readback.model.name, "wswls_b_251");
    let root = &artifact.readback.node_tree.roots[0];
    let position = root
        .controllers
        .iter()
        .find(|controller| controller.controller_type == 8)
        .unwrap();
    let orientation = root
        .controllers
        .iter()
        .find(|controller| controller.controller_type == 20)
        .unwrap();
    assert_eq!(position.values[0], [1.0, 2.0, 3.0]);
    assert!((orientation.values[0][2].abs() - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
    assert!((orientation.values[0][3].abs() - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
    assert!(!artifact.texture_payload.is_empty());
}

#[test]
fn item_part_can_select_one_top_level_source_node_and_emit_real_plt() {
    let source = build_synthetic_glb::mutate_json(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        root["scenes"][0]["nodes"] = serde_json::json!([0, 1]);
        root["nodes"] = serde_json::json!([
            { "name": "blade", "mesh": 0 },
            { "name": "unused" }
        ]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .unwrap();
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");
    });
    let artifact = build_meshy_item_part_with_options_v2(
        &source,
        "wswls_b_251",
        "m2alsbt1",
        &ItemPartBuildOptionsV2 {
            schema_version: 1,
            transform: ItemPartTransformV1::default(),
            source_node: Some("blade".to_owned()),
            texture_encoding: ItemPartTextureEncodingV1::PltMetal1,
            icon_size: Some([64, 96]),
            icon_projection_bounds: None,
        },
    )
    .unwrap();

    assert_eq!(artifact.report.source_node.as_deref(), Some("blade"));
    assert_eq!(artifact.report.texture_format, "PLT_V1");
    assert_eq!(&artifact.texture_payload[..8], b"PLT V1  ");
    assert!(
        artifact.texture_payload[25..]
            .iter()
            .step_by(2)
            .all(|value| *value == 2)
    );
    assert!(artifact.report.icon_sha256.is_some());
    let icon = artifact.icon_payload.as_deref().unwrap();
    assert_eq!(icon[2], 2);
    assert_eq!(u16::from_le_bytes([icon[12], icon[13]]), 64);
    assert_eq!(u16::from_le_bytes([icon[14], icon[15]]), 96);
    let opaque_pixels = icon[18..]
        .chunks_exact(4)
        .filter(|pixel| pixel[3] == u8::MAX)
        .count();
    assert!(
        opaque_pixels > 0 && opaque_pixels < 64 * 64,
        "the icon must be a geometry-derived silhouette, not an opaque resized texture rectangle"
    );
}

#[test]
fn item_geometry_budget_is_the_sum_of_all_parts() {
    let admitted = validate_item_triangle_budget_v1(&[100_000, 100_000, 100_000]).unwrap();
    assert_eq!(admitted.triangle_count, 300_000);
    assert_eq!(admitted.triangle_budget, 300_000);
    assert!(admitted.warning);

    let error = validate_item_triangle_budget_v1(&[150_000, 150_001]).unwrap_err();
    assert_eq!(error.code, "ITEM-TRIANGLE-BUDGET-EXCEEDED");
    assert!(error.message.contains("300001"));
}

#[test]
fn item_part_rejects_multiple_used_materials_instead_of_rebinding_the_first_texture() {
    let error = build_meshy_item_part_with_options_v2(
        &build_synthetic_glb::material_image_two_primitives_3d(),
        "multi_mat",
        "multi_tex",
        &ItemPartBuildOptionsV2::default(),
    )
    .unwrap_err();

    assert_eq!(error.code, "ITEM-PART-MULTI-MATERIAL-UNSUPPORTED");
    assert_eq!(error.path, "model.materialSourceBindings");
}

#[test]
fn item_part_material_gate_ignores_materials_outside_the_selected_source_root() {
    let source = build_synthetic_glb::mutate_json(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .unwrap();
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");
        let mut unused_mesh = root["meshes"][0].clone();
        unused_mesh["name"] = serde_json::json!("unused-mesh");
        unused_mesh["primitives"][0]["material"] = serde_json::json!(1);
        root["meshes"].as_array_mut().unwrap().push(unused_mesh);
        let mut unused_material = root["materials"][0].clone();
        unused_material["name"] = serde_json::json!("unused-material");
        root["materials"]
            .as_array_mut()
            .unwrap()
            .push(unused_material);
        root["nodes"] = serde_json::json!([
            { "name": "selected", "mesh": 0 },
            { "name": "unused", "mesh": 1 }
        ]);
        root["scenes"][0]["nodes"] = serde_json::json!([0, 1]);
    });
    let artifact = build_meshy_item_part_with_options_v2(
        &source,
        "selected_part",
        "selected_tex",
        &ItemPartBuildOptionsV2 {
            source_node: Some("selected".to_owned()),
            ..ItemPartBuildOptionsV2::default()
        },
    )
    .unwrap();

    assert_eq!(artifact.report.source_node.as_deref(), Some("selected"));
    assert_eq!(artifact.report.triangle_count, 12);
}

#[test]
fn authoritative_seam_measurement_uses_transformed_triangle_surfaces_and_binds_inputs() {
    let source = build_synthetic_glb::unit_cube();
    let options = |translation| ItemPartBuildOptionsV2 {
        transform: ItemPartTransformV1 {
            translation,
            ..ItemPartTransformV1::default()
        },
        ..ItemPartBuildOptionsV2::default()
    };

    let touching = measure_meshy_item_seam_v1(
        "ModelPart1",
        &source,
        "seam_a",
        &options([0.0, 0.0, 0.0]),
        "ModelPart2",
        &source,
        "seam_b",
        &options([1.0, 0.0, 0.0]),
        0.01,
    )
    .unwrap();
    assert_eq!(touching.status, "TOUCHING");
    assert!(!touching.overlap);
    assert!(touching.gap <= 1.0e-6);
    assert_eq!(touching.algorithm, "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1");
    assert_eq!(touching.first_source_sha256, touching.second_source_sha256);
    assert_ne!(
        touching.first_transform_sha256,
        touching.second_transform_sha256
    );
    assert_eq!(touching.measurement_sha256.len(), 64);

    let tolerated_gap = measure_meshy_item_seam_v1(
        "ModelPart1",
        &source,
        "seam_a",
        &options([0.0, 0.0, 0.0]),
        "ModelPart2",
        &source,
        "seam_b",
        &options([1.005, 0.0, 0.0]),
        0.01,
    )
    .unwrap();
    assert_eq!(tolerated_gap.status, "TOUCHING");
    assert!((tolerated_gap.gap - 0.005).abs() <= 1.0e-5);

    let gap = measure_meshy_item_seam_v1(
        "ModelPart1",
        &source,
        "seam_a",
        &options([0.0, 0.0, 0.0]),
        "ModelPart2",
        &source,
        "seam_b",
        &options([1.1, 0.0, 0.0]),
        0.01,
    )
    .unwrap();
    assert_eq!(gap.status, "GAP");
    assert!((gap.gap - 0.1).abs() <= 1.0e-5);

    let overlap = measure_meshy_item_seam_v1(
        "ModelPart1",
        &source,
        "seam_a",
        &options([0.0, 0.0, 0.0]),
        "ModelPart2",
        &source,
        "seam_b",
        &options([0.25, 0.0, 0.0]),
        0.01,
    )
    .unwrap();
    assert_eq!(overlap.status, "OVERLAP");
    assert!(overlap.overlap);
}

#[test]
fn item_proof_module_embeds_the_exact_uti_and_places_one_candidate_in_front_of_entry() {
    let row = resolve_item_baseitem_v1(BASEITEMS, 0).unwrap();
    let uti = write_item_uti_v1(
        &row,
        &ItemBlueprintV1 {
            schema_version: 1,
            template_resref: "m2aitestuti".to_owned(),
            tag: "M2AITESTUTI".to_owned(),
            localized_name: "Item proof fixture".to_owned(),
            description: "Item proof fixture".to_owned(),
            identified_description: "Item proof fixture".to_owned(),
            comment: "Item proof fixture".to_owned(),
            parts: vec![ItemPartValueV1 {
                field: "ModelPart1".to_owned(),
                value: 1,
            }],
            properties: Vec::new(),
            colors: ItemColorValuesV1::default(),
            cost: 0,
            add_cost: 0,
            charges: 0,
            stack_size: 1,
            palette_id: 0,
            identified: true,
            stolen: false,
            cursed: false,
            plot: false,
        },
    )
    .unwrap();
    let module = build_item_proof_module_v1(
        &uti.payload,
        &ItemProofModuleIdentityV1 {
            schema_version: 1,
            module_resref: "m2aitestmod".to_owned(),
            area_resref: "m2aitestarea".to_owned(),
            hak_resref: "m2aitesthak".to_owned(),
            blueprint_resref: "m2aitestuti".to_owned(),
            module_name: "Meshy2Aurora Item candidate".to_owned(),
            area_name: "Item Assembly Proof".to_owned(),
        },
        ItemProofPlacementV1::default(),
    )
    .unwrap();

    assert_eq!(module.report.semantic_readback_status, "PASS");
    assert_eq!(module.report.item_position, [10.0, 14.5, 0.0]);
    assert_eq!(module.report.entry_direction, [0.0, 1.0]);
    assert_eq!(module.report.model_visibility, "not_tested");
    assert_eq!(module.report.proof_completeness, "missing");
    assert!(!module.payload.is_empty());
}

#[test]
fn capart_proof_module_equips_the_uti_on_one_creature_instead_of_placing_it_on_the_ground() {
    let row = resolve_item_baseitem_v1(BASEITEMS, 3).unwrap();
    let uti = write_item_uti_v1(
        &row,
        &ItemBlueprintV1 {
            schema_version: 1,
            template_resref: "m2aarmoruti".to_owned(),
            tag: "M2AARMORUTI".to_owned(),
            localized_name: "Equipped armor proof".to_owned(),
            description: "Equipped armor proof".to_owned(),
            identified_description: "Equipped armor proof".to_owned(),
            comment: "Equipped armor proof".to_owned(),
            parts: ARMOR_PART_FIELDS_V1
                .into_iter()
                .map(|field| ItemPartValueV1 {
                    field: field.to_owned(),
                    value: 1,
                })
                .collect(),
            properties: Vec::new(),
            colors: ItemColorValuesV1 {
                leather1_color: Some(0),
                leather2_color: Some(0),
                cloth1_color: Some(0),
                cloth2_color: Some(0),
                metal1_color: Some(0),
                metal2_color: Some(0),
            },
            cost: 0,
            add_cost: 0,
            charges: 0,
            stack_size: 1,
            palette_id: 0,
            identified: true,
            stolen: false,
            cursed: false,
            plot: false,
        },
    )
    .unwrap();
    let module = build_item_equipped_proof_module_v2(
        &uti.payload,
        &ItemEquippedProofIdentityV2 {
            schema_version: 2,
            module_resref: "m2aarmormod".to_owned(),
            area_resref: "m2aarmorarea".to_owned(),
            hak_resref: "m2aarmorhak".to_owned(),
            blueprint_resref: "m2aarmoruti".to_owned(),
            module_name: "Equipped CAPART proof".to_owned(),
            area_name: "Equipped CAPART proof".to_owned(),
            creature_resref: "m2aarmornpc".to_owned(),
            creature_display_name: "CAPART proof wearer".to_owned(),
            appearance_row: 6,
            race: 6,
            gender: 0,
            phenotype: 0,
            model_prefix: "pmh0".to_owned(),
            appearance_table_sha256: "a".repeat(64),
            fixture_profile: ItemEquippedProofProfileV2::CapartArmor,
        },
        ItemProofPlacementV1::default(),
    )
    .unwrap();
    assert_eq!(module.report.fixture_profile, "EQUIPPED_CAPART_ARMOR_V2");
    assert_eq!(module.report.equipment_slot, 2);
    assert_eq!(module.report.model_prefix, "pmh0");
    assert_eq!(module.report.appearance_table_sha256, "a".repeat(64));
    assert_eq!(
        (
            module.report.race,
            module.report.gender,
            module.report.phenotype
        ),
        (6, 0, 0)
    );
    assert_eq!(module.report.semantic_readback_status, "PASS");

    let archive = ErfArchive::parse(&module.payload).unwrap();
    let git = read_gff_v32(
        archive.find("m2aarmorarea", 2023).unwrap(),
        &GffLimitsV1::default(),
    )
    .unwrap();
    let ground_items = git
        .root
        .fields
        .iter()
        .find(|field| field.label == "List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .unwrap();
    assert!(ground_items.is_empty());
    let creatures = git
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .unwrap();
    assert_eq!(creatures.len(), 1);
    let equipment = creatures[0]
        .fields
        .iter()
        .find(|field| field.label == "Equip_ItemList")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .unwrap();
    assert_eq!(equipment.len(), 1);
    assert_eq!(equipment[0].struct_id, 2);
    assert_eq!(
        equipment[0].fields[0].value,
        GffValueV1::ResRef("m2aarmoruti".to_owned())
    );
    assert!(archive.find("m2aarmornpc", 2027).is_ok());
}

#[test]
fn cloak_proof_module_equips_the_uti_in_the_native_cloak_slot() {
    let baseitems = br#"2DA V2.0

Label ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight
80 Cloak cloak 1 1 it_bag icloak 8192 2 2
"#;
    let row = resolve_item_baseitem_v1(baseitems, 80).unwrap();
    let uti = write_item_uti_v1(
        &row,
        &ItemBlueprintV1 {
            schema_version: 1,
            template_resref: "m2acloakuti".to_owned(),
            tag: "M2ACLOAKUTI".to_owned(),
            localized_name: "Equipped cloak proof".to_owned(),
            description: "Equipped cloak proof".to_owned(),
            identified_description: "Equipped cloak proof".to_owned(),
            comment: "Equipped cloak proof".to_owned(),
            parts: vec![ItemPartValueV1 {
                field: "ModelPart1".to_owned(),
                value: 1,
            }],
            properties: Vec::new(),
            colors: ItemColorValuesV1 {
                leather1_color: Some(0),
                leather2_color: Some(0),
                cloth1_color: Some(0),
                cloth2_color: Some(0),
                metal1_color: Some(0),
                metal2_color: Some(0),
            },
            cost: 0,
            add_cost: 0,
            charges: 0,
            stack_size: 1,
            palette_id: 0,
            identified: true,
            stolen: false,
            cursed: false,
            plot: false,
        },
    )
    .unwrap();
    let module = build_item_equipped_proof_module_v2(
        &uti.payload,
        &ItemEquippedProofIdentityV2 {
            schema_version: 2,
            module_resref: "m2acloakmod".to_owned(),
            area_resref: "m2acloakarea".to_owned(),
            hak_resref: "m2acloakhak".to_owned(),
            blueprint_resref: "m2acloakuti".to_owned(),
            module_name: "Equipped Cloak proof".to_owned(),
            area_name: "Equipped Cloak proof".to_owned(),
            creature_resref: "m2acloaknpc".to_owned(),
            creature_display_name: "Cloak proof wearer".to_owned(),
            appearance_row: 6,
            race: 6,
            gender: 0,
            phenotype: 0,
            model_prefix: "pmh0".to_owned(),
            appearance_table_sha256: "a".repeat(64),
            fixture_profile: ItemEquippedProofProfileV2::Cloak,
        },
        ItemProofPlacementV1::default(),
    )
    .unwrap();

    assert_eq!(module.report.fixture_profile, "EQUIPPED_CLOAK_V2");
    assert_eq!(module.report.equipment_slot, 8_192);
    assert_eq!(module.report.model_prefix, "pmh0");
    let archive = ErfArchive::parse(&module.payload).unwrap();
    let git = read_gff_v32(
        archive.find("m2acloakarea", 2023).unwrap(),
        &GffLimitsV1::default(),
    )
    .unwrap();
    let ground_items = git
        .root
        .fields
        .iter()
        .find(|field| field.label == "List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .unwrap();
    assert!(ground_items.is_empty());
    let equipment = git
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => values.first(),
            _ => None,
        })
        .and_then(|creature| {
            creature
                .fields
                .iter()
                .find(|field| field.label == "Equip_ItemList")
        })
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .unwrap();
    assert_eq!(equipment.len(), 1);
    assert_eq!(equipment[0].struct_id, 8_192);
    assert_eq!(
        equipment[0].fields[0].value,
        GffValueV1::ResRef("m2acloakuti".to_owned())
    );
    assert!(archive.find("m2acloaknpc", 2027).is_ok());
}
