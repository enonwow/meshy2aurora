use m2a_core::gff::{
    GffFieldV1, GffLimitsV1, GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
};
use m2a_core::item::{
    ITEM_SCHEMA_VERSION, ItemAppearanceRecipeV1, ItemBaseRecordV1, ItemColorRecipeV1,
    ItemCompositionProfileV1, ItemGenderV1, ItemIdentityV1, ItemPartRecipeV1, ItemPartTransformV1,
    required_item_part_slots_v1,
};
use m2a_core::item_uti::{
    ITEM_PALETTE_ID_UNSUPPORTED, ITEM_UTI_FIELD_TYPE_INVALID, ITEM_UTI_FIELD_UNEXPECTED,
    ItemPaletteEntryV1, ItemUtiBuildRequestV1, ItemUtiPropertiesV1, read_item_uti_v1,
    write_item_palette_itp_v1, write_item_uti_v1,
};

const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn request(profile: ItemCompositionProfileV1) -> ItemUtiBuildRequestV1 {
    let parts = required_item_part_slots_v1(profile)
        .iter()
        .enumerate()
        .map(|(index, slot)| ItemPartRecipeV1 {
            slot: *slot,
            variant: u8::try_from(index + 1).unwrap(),
            source_part_id: format!("part-{index}"),
            source_sha256: HASH.to_owned(),
            transform: ItemPartTransformV1::default(),
        })
        .collect();
    let colors = matches!(
        profile,
        ItemCompositionProfileV1::ModelType1 | ItemCompositionProfileV1::ModelType3
    )
    .then_some(ItemColorRecipeV1 {
        leather1: 10,
        leather2: 11,
        cloth1: 12,
        cloth2: 13,
        metal1: 14,
        metal2: 15,
    });
    ItemUtiBuildRequestV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        recipe: ItemAppearanceRecipeV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            base_item: ItemBaseRecordV1 {
                schema_version: ITEM_SCHEMA_VERSION,
                source_sha256: HASH.to_owned(),
                physical_row_index: u32::from(profile.model_type()),
                printed_row_label: u32::from(profile.model_type()),
                profile,
                model_type: profile.model_type(),
                item_class: match profile {
                    ItemCompositionProfileV1::ModelType0 => "AShSw",
                    ItemCompositionProfileV1::ModelType1 => "helm",
                    ItemCompositionProfileV1::ModelType2 => "WSwLs",
                    ItemCompositionProfileV1::ModelType3 => "AArCl",
                }
                .to_owned(),
                gender_specific: profile == ItemCompositionProfileV1::ModelType3,
                inv_slot_width: 1,
                inv_slot_height: 1,
                equipable_slots: "0x00000".to_owned(),
                default_model: None,
                default_icon: None,
            },
            identity: ItemIdentityV1 {
                uti_resref: format!("m2a_item_00{}", profile.model_type()),
                tag: format!("M2A_ITEM_00{}", profile.model_type()),
                display_name: format!("Item profile {}", profile.model_type()),
            },
            gender: (profile == ItemCompositionProfileV1::ModelType3).then_some(ItemGenderV1::Male),
            parts,
            colors,
        },
        properties: ItemUtiPropertiesV1 {
            description: "Description".to_owned(),
            identified_description: "Identified description".to_owned(),
            charges: 2,
            cost: 123,
            add_cost: 4,
            stack_size: 1,
            stolen: false,
            plot: true,
            identified: true,
            cursed: false,
            palette_id: 36,
            comment: "Synthetic Item UTI fixture".to_owned(),
        },
    }
}

#[test]
fn all_four_profiles_are_deterministic_and_round_trip_exactly() {
    for profile in [
        ItemCompositionProfileV1::ModelType0,
        ItemCompositionProfileV1::ModelType1,
        ItemCompositionProfileV1::ModelType2,
        ItemCompositionProfileV1::ModelType3,
    ] {
        let input = request(profile);
        let first = write_item_uti_v1(&input, &GffWriterOptionsV1::default()).unwrap();
        let second = write_item_uti_v1(&input, &GffWriterOptionsV1::default()).unwrap();
        assert_eq!(first.payload, second.payload);
        assert_eq!(first.report, second.report);
        assert_eq!(first.readback, second.readback);
        assert_eq!(first.readback.profile, profile);
        assert_eq!(
            first.readback.parts.len(),
            required_item_part_slots_v1(profile).len()
        );
        assert_eq!(first.report.semantic_readback_status, "PASS");
    }
}

#[test]
fn armor_emits_all_19_native_byte_fields_including_robe_and_six_colors() {
    let artifact = write_item_uti_v1(
        &request(ItemCompositionProfileV1::ModelType3),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let document = read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap();
    for label in [
        "ArmorPart_RFoot",
        "ArmorPart_LFoot",
        "ArmorPart_RShin",
        "ArmorPart_LShin",
        "ArmorPart_LThigh",
        "ArmorPart_RThigh",
        "ArmorPart_Pelvis",
        "ArmorPart_Torso",
        "ArmorPart_Belt",
        "ArmorPart_Neck",
        "ArmorPart_RFArm",
        "ArmorPart_LFArm",
        "ArmorPart_RBicep",
        "ArmorPart_LBicep",
        "ArmorPart_RShoul",
        "ArmorPart_LShoul",
        "ArmorPart_RHand",
        "ArmorPart_LHand",
        "ArmorPart_Robe",
        "Leather1Color",
        "Leather2Color",
        "Cloth1Color",
        "Cloth2Color",
        "Metal1Color",
        "Metal2Color",
    ] {
        assert!(matches!(
            document
                .root
                .fields
                .iter()
                .find(|field| field.label == label)
                .map(|field| &field.value),
            Some(GffValueV1::Byte(_))
        ));
    }
}

#[test]
fn reader_rejects_a_profile_field_with_the_wrong_gff_type() {
    let artifact = write_item_uti_v1(
        &request(ItemCompositionProfileV1::ModelType2),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut document = read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap();
    document
        .root
        .fields
        .iter_mut()
        .find(|field| field.label == "ModelPart2")
        .unwrap()
        .value = GffValueV1::Word(2);
    let mutated = write_gff_v32(&document, &GffWriterOptionsV1::default()).unwrap();
    assert_eq!(
        read_item_uti_v1(
            &mutated.payload,
            ItemCompositionProfileV1::ModelType2,
            &GffLimitsV1::default(),
        )
        .unwrap_err()
        .code,
        ITEM_UTI_FIELD_TYPE_INVALID
    );
}

#[test]
fn reader_rejects_fields_from_another_model_type_profile() {
    let artifact = write_item_uti_v1(
        &request(ItemCompositionProfileV1::ModelType0),
        &GffWriterOptionsV1::default(),
    )
    .unwrap();
    let mut document = read_gff_v32(&artifact.payload, &GffLimitsV1::default()).unwrap();
    document.root.fields.push(GffFieldV1 {
        label: "ModelPart2".to_owned(),
        value: GffValueV1::Byte(1),
    });
    let mutated = write_gff_v32(&document, &GffWriterOptionsV1::default()).unwrap();
    assert_eq!(
        read_item_uti_v1(
            &mutated.payload,
            ItemCompositionProfileV1::ModelType0,
            &GffLimitsV1::default(),
        )
        .unwrap_err()
        .code,
        ITEM_UTI_FIELD_UNEXPECTED
    );
}

#[test]
fn reader_rejects_template_resref_mismatch_by_semantic_comparison() {
    let mut input = request(ItemCompositionProfileV1::ModelType2);
    input.recipe.identity.uti_resref = "m2a_item_exact".to_owned();
    let artifact = write_item_uti_v1(&input, &GffWriterOptionsV1::default()).unwrap();
    assert_eq!(artifact.readback.identity.uti_resref, "m2a_item_exact");
    assert_eq!(
        artifact.report.payload_sha256,
        artifact.readback.payload_sha256
    );
}

#[test]
fn helmet_custom_palette_is_deterministic_and_contains_one_exact_uti_leaf() {
    let entry = ItemPaletteEntryV1 {
        uti_resref: "m2a_helm_palette".to_owned(),
        display_name: "Bronze shadow mask".to_owned(),
        palette_id: 9,
    };
    let first = write_item_palette_itp_v1(&entry).unwrap();
    let second = write_item_palette_itp_v1(&entry).unwrap();
    assert_eq!(first.payload, second.payload);

    let document = read_gff_v32(&first.payload, &GffLimitsV1::default()).unwrap();
    assert_eq!(document.file_type, m2a_core::gff::GffFileTypeV1::Itp);
    assert!(matches!(
        document
            .root
            .fields
            .iter()
            .find(|field| field.label == "RESTYPE")
            .map(|field| &field.value),
        Some(GffValueV1::Word(2025))
    ));
    assert!(matches!(
        document
            .root
            .fields
            .iter()
            .find(|field| field.label == "NEXT_USEABLE_ID")
            .map(|field| &field.value),
        Some(GffValueV1::Byte(65))
    ));
    let main = match document
        .root
        .fields
        .iter()
        .find(|field| field.label == "MAIN")
        .map(|field| &field.value)
    {
        Some(GffValueV1::List(main)) => main,
        _ => panic!("MAIN list"),
    };
    let armor = &main[0];
    let helmets = match armor
        .fields
        .iter()
        .find(|field| field.label == "LIST")
        .map(|field| &field.value)
    {
        Some(GffValueV1::List(categories)) => &categories[0],
        _ => panic!("Armor category list"),
    };
    assert!(matches!(
        helmets
            .fields
            .iter()
            .find(|field| field.label == "ID")
            .map(|field| &field.value),
        Some(GffValueV1::Byte(9))
    ));
    let leaf = match helmets
        .fields
        .iter()
        .find(|field| field.label == "LIST")
        .map(|field| &field.value)
    {
        Some(GffValueV1::List(entries)) => &entries[0],
        _ => panic!("Helmet entry list"),
    };
    assert!(matches!(
        leaf.fields
            .iter()
            .find(|field| field.label == "RESREF")
            .map(|field| &field.value),
        Some(GffValueV1::ResRef(value)) if value == "m2a_helm_palette"
    ));
    assert!(matches!(
        leaf.fields
            .iter()
            .find(|field| field.label == "NAME")
            .map(|field| &field.value),
        Some(GffValueV1::String(value)) if value == b"Bronze shadow mask"
    ));

    let mut unsupported = entry;
    unsupported.palette_id = 8;
    assert_eq!(
        write_item_palette_itp_v1(&unsupported).unwrap_err().code,
        ITEM_PALETTE_ID_UNSUPPORTED
    );
}
