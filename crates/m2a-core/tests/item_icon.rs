use m2a_core::item::{
    ITEM_SCHEMA_VERSION, ItemAppearanceRecipeV1, ItemBaseRecordV1, ItemColorRecipeV1,
    ItemCompositionProfileV1, ItemIdentityV1, ItemPartRecipeV1, ItemPartTransformV1,
    required_item_part_slots_v1, resolve_item_resource_names_v1,
};
use m2a_core::item_icon::{
    ITEM_ICON_DIMENSIONS_INVALID, ITEM_ICON_INVALID, ITEM_ICON_PLT_RESOURCE_TYPE, ItemIconFormatV1,
    ItemIconLayerInputV1, ItemPltIconComposeOptionsV1, ItemPltIconLayerInputV1,
    ItemPltIconMaterialMapV1, compose_item_plt_icon_from_material_id_v1, write_item_icon_layers_v1,
    write_item_plt_icon_layers_v1,
};
use m2a_core::plt::{
    PltImageV1, PltPixelV1, PltWriterOptionsV1, plt_image_pixels_sha256_v1, read_plt_image_v1,
};
use m2a_core::tga::{
    TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, read_tga_image_v1, write_tga_v1,
};

const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn recipe() -> ItemAppearanceRecipeV1 {
    let profile = ItemCompositionProfileV1::ModelType2;
    ItemAppearanceRecipeV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        base_item: ItemBaseRecordV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            source_sha256: HASH.to_owned(),
            physical_row_index: 1,
            printed_row_label: 1,
            profile,
            model_type: 2,
            item_class: "WSwLs".to_owned(),
            gender_specific: false,
            inv_slot_width: 1,
            inv_slot_height: 4,
            equipable_slots: "0x1C030".to_owned(),
            default_model: Some("it_bag".to_owned()),
            default_icon: Some("iwswls".to_owned()),
        },
        identity: ItemIdentityV1 {
            uti_resref: "m2a_item_001".to_owned(),
            tag: "M2A_ITEM_001".to_owned(),
            display_name: "Item".to_owned(),
        },
        gender: None,
        parts: required_item_part_slots_v1(profile)
            .iter()
            .enumerate()
            .map(|(index, slot)| ItemPartRecipeV1 {
                slot: *slot,
                variant: u8::try_from(index + 1).unwrap(),
                source_part_id: format!("part-{index}"),
                source_sha256: HASH.to_owned(),
                transform: ItemPartTransformV1::default(),
            })
            .collect(),
        colors: None,
    }
}

fn helmet_recipe() -> ItemAppearanceRecipeV1 {
    let profile = ItemCompositionProfileV1::ModelType1;
    ItemAppearanceRecipeV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        base_item: ItemBaseRecordV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            source_sha256: HASH.to_owned(),
            physical_row_index: 17,
            printed_row_label: 17,
            profile,
            model_type: 1,
            item_class: "helm".to_owned(),
            gender_specific: false,
            inv_slot_width: 2,
            inv_slot_height: 2,
            equipable_slots: "0x1".to_owned(),
            default_model: None,
            default_icon: Some("ihelm".to_owned()),
        },
        identity: ItemIdentityV1 {
            uti_resref: "m2a_helm_221".to_owned(),
            tag: "M2A_HELM_221".to_owned(),
            display_name: "Helmet".to_owned(),
        },
        gender: None,
        parts: vec![ItemPartRecipeV1 {
            slot: m2a_core::item::ItemPartSlotV1::Model,
            variant: 221,
            source_part_id: "helmet".to_owned(),
            source_sha256: HASH.to_owned(),
            transform: ItemPartTransformV1::default(),
        }],
        colors: Some(ItemColorRecipeV1 {
            leather1: 0,
            leather2: 0,
            cloth1: 23,
            cloth2: 0,
            metal1: 100,
            metal2: 0,
        }),
    }
}

fn layers(input: &ItemAppearanceRecipeV1) -> Vec<ItemIconLayerInputV1> {
    resolve_item_resource_names_v1(input)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let image = TgaImageV1 {
                schema_version: 1,
                width: 32,
                height: 128,
                pixel_format: TgaPixelFormatV1::Rgba8,
                pixels: vec![u8::try_from(index + 1).unwrap(); 32 * 128 * 4],
            };
            let source_sha256 = write_tga_v1(&image, &TgaWriterOptionsV1::default())
                .unwrap()
                .report
                .input_sha256;
            ItemIconLayerInputV1 {
                slot: name.slot,
                resref: name.icon_resref,
                source_sha256,
                image,
            }
        })
        .collect()
}

#[test]
fn icon_layers_follow_resolved_names_dimensions_alpha_and_exact_pixels() {
    let input = recipe();
    let sources = layers(&input);
    let first =
        write_item_icon_layers_v1(&input, &sources, &TgaWriterOptionsV1::default()).unwrap();
    let second =
        write_item_icon_layers_v1(&input, &sources, &TgaWriterOptionsV1::default()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 3);
    for (artifact, source) in first.iter().zip(sources.iter()) {
        assert_eq!(artifact.resref, source.resref);
        assert_eq!(read_tga_image_v1(&artifact.payload).unwrap(), source.image);
        assert_eq!(artifact.report.semantic_readback_status, "PASS");
    }
}

#[test]
fn icon_build_rejects_missing_wrong_size_rgb_and_stale_inputs() {
    let input = recipe();
    let mut sources = layers(&input);
    sources.pop();
    assert_eq!(
        write_item_icon_layers_v1(&input, &sources, &TgaWriterOptionsV1::default())
            .unwrap_err()
            .code,
        ITEM_ICON_INVALID
    );

    let mut sources = layers(&input);
    sources[0].image.height = 32;
    sources[0].image.pixels.truncate(32 * 32 * 4);
    sources[0].source_sha256 = write_tga_v1(&sources[0].image, &TgaWriterOptionsV1::default())
        .unwrap()
        .report
        .input_sha256;
    assert_eq!(
        write_item_icon_layers_v1(&input, &sources, &TgaWriterOptionsV1::default())
            .unwrap_err()
            .code,
        ITEM_ICON_DIMENSIONS_INVALID
    );

    let mut sources = layers(&input);
    sources[0].image.pixel_format = TgaPixelFormatV1::Rgb8;
    sources[0].image.pixels = vec![1; 32 * 128 * 3];
    sources[0].source_sha256 = write_tga_v1(&sources[0].image, &TgaWriterOptionsV1::default())
        .unwrap()
        .report
        .input_sha256;
    assert_eq!(
        write_item_icon_layers_v1(&input, &sources, &TgaWriterOptionsV1::default())
            .unwrap_err()
            .code,
        ITEM_ICON_INVALID
    );
}

#[test]
fn model_type_1_requires_and_roundtrips_resource_type_6_plt_icons() {
    let recipe = helmet_recipe();
    let name = resolve_item_resource_names_v1(&recipe).unwrap().remove(0);
    let image = PltImageV1 {
        schema_version: 1,
        width: 64,
        height: 64,
        num_layers: 5,
        pixels: vec![
            PltPixelV1 {
                color_index: 90,
                layer_index: 2,
            };
            64 * 64
        ],
    };
    let input = ItemPltIconLayerInputV1 {
        slot: name.slot,
        resref: name.icon_resref,
        source_sha256: plt_image_pixels_sha256_v1(&image),
        image: image.clone(),
    };
    let icons =
        write_item_plt_icon_layers_v1(&recipe, &[input], &PltWriterOptionsV1::default()).unwrap();
    assert_eq!(icons[0].report.format, ItemIconFormatV1::Plt);
    assert_eq!(icons[0].report.resource_type, ITEM_ICON_PLT_RESOURCE_TYPE);
    assert_eq!(read_plt_image_v1(&icons[0].payload).unwrap(), image);

    let tga_image = TgaImageV1 {
        schema_version: 1,
        width: 64,
        height: 64,
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: vec![255; 64 * 64 * 4],
    };
    let tga_input = ItemIconLayerInputV1 {
        slot: name.slot,
        resref: "ihelm_221".to_owned(),
        source_sha256: write_tga_v1(&tga_image, &TgaWriterOptionsV1::default())
            .unwrap()
            .report
            .input_sha256,
        image: tga_image,
    };
    assert_eq!(
        write_item_icon_layers_v1(&recipe, &[tga_input], &TgaWriterOptionsV1::default())
            .unwrap_err()
            .code,
        ITEM_ICON_INVALID
    );
}

#[test]
fn material_id_icon_composition_preserves_dark_foreground_and_all_requested_layers() {
    let width = 5;
    let height = 2;
    let mut beauty_pixels = vec![0u8; width * height * 4];
    let mut id_pixels = vec![0u8; width * height * 4];
    let id_colors = [
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [255, 255, 0],
        [255, 0, 255],
    ];
    for (x, id) in id_colors.into_iter().enumerate() {
        let offset = x * 4;
        let value = if x == 0 {
            1
        } else {
            u8::try_from(40 + x * 30).unwrap()
        };
        beauty_pixels[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
        id_pixels[offset..offset + 4].copy_from_slice(&[id[0], id[1], id[2], 255]);
    }
    // The beauty background is deliberately opaque black. Only the material-ID
    // alpha may decide the silhouette, otherwise the dark Cloth 1 pixel at the
    // source edge is lost exactly like the rejected helmet-225 icon.
    for x in 0..width {
        let offset = (width + x) * 4;
        beauty_pixels[offset..offset + 4].copy_from_slice(&[0, 0, 0, 255]);
    }
    let beauty = TgaImageV1 {
        schema_version: 1,
        width: u32::try_from(width).unwrap(),
        height: u32::try_from(height).unwrap(),
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: beauty_pixels,
    };
    let material_ids = TgaImageV1 {
        schema_version: 1,
        width: u32::try_from(width).unwrap(),
        height: u32::try_from(height).unwrap(),
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: id_pixels,
    };
    let mappings = [
        ItemPltIconMaterialMapV1 {
            id_rgb: id_colors[0],
            layer_index: 4,
        },
        ItemPltIconMaterialMapV1 {
            id_rgb: id_colors[1],
            layer_index: 2,
        },
        ItemPltIconMaterialMapV1 {
            id_rgb: id_colors[2],
            layer_index: 6,
        },
        ItemPltIconMaterialMapV1 {
            id_rgb: id_colors[3],
            layer_index: 7,
        },
        ItemPltIconMaterialMapV1 {
            id_rgb: id_colors[4],
            layer_index: 5,
        },
    ];
    let composed = compose_item_plt_icon_from_material_id_v1(
        &beauty,
        &material_ids,
        &mappings,
        &ItemPltIconComposeOptionsV1 {
            width: 7,
            height: 3,
            padding: 1,
            alpha_threshold: 64,
        },
    )
    .unwrap();

    assert_eq!(composed.width, 7);
    assert_eq!(composed.height, 3);
    assert_eq!(composed.num_layers, 8);
    let used_layers = composed
        .pixels
        .iter()
        .filter(|pixel| pixel.color_index != 255)
        .map(|pixel| pixel.layer_index)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(used_layers, [2, 4, 5, 6, 7].into_iter().collect());
    assert!(
        composed
            .pixels
            .iter()
            .any(|pixel| { pixel.layer_index == 4 && pixel.color_index == 1 })
    );
    assert!((0..7).all(|x| composed.pixels[x].color_index == 255));
    assert!((0..7).all(|x| composed.pixels[14 + x].color_index == 255));
}
