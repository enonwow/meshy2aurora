use m2a_core::{
    item_material_mapping::{
        ItemMaterialLayerMappingPolicyV1, NwnItemPaletteLayerV1, SourceMaterialLayerMappingV1,
        UnmappedSourceMaterialPolicyV1, build_item_material_plt_v1, lift_item_plt_layer_luma_v1,
        resolve_item_source_material_layer_v1,
    },
    plt::{PLT_TRANSPARENT_PIXEL_V1, PltImageV1, PltPixelV1},
    tga::{TgaImageV1, TgaPixelFormatV1},
};

fn two_color_source() -> TgaImageV1 {
    TgaImageV1 {
        schema_version: 1,
        width: 2,
        height: 1,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: vec![180, 120, 40, 20, 30, 60],
    }
}

#[test]
fn explicit_source_material_mapping_overrides_automatic_pixel_inference() {
    let policy = ItemMaterialLayerMappingPolicyV1 {
        schema_version: 1,
        mappings: vec![
            SourceMaterialLayerMappingV1 {
                source_material_id: 0,
                target_layer: NwnItemPaletteLayerV1::Cloth1,
            },
            SourceMaterialLayerMappingV1 {
                source_material_id: 1,
                target_layer: NwnItemPaletteLayerV1::Metal1,
            },
        ],
        unmapped_policy: UnmappedSourceMaterialPolicyV1::AutomaticClothMetal,
    };

    let cloth = resolve_item_source_material_layer_v1(&policy, 0).unwrap();
    let metal = resolve_item_source_material_layer_v1(&policy, 1).unwrap();
    let fallback = resolve_item_source_material_layer_v1(&policy, 99).unwrap();
    assert_eq!(cloth, Some(NwnItemPaletteLayerV1::Cloth1));
    assert_eq!(metal, Some(NwnItemPaletteLayerV1::Metal1));
    assert_eq!(fallback, None);

    let cloth_plt = build_item_material_plt_v1(&two_color_source(), cloth).unwrap();
    assert!(cloth_plt.pixels.iter().all(|pixel| pixel.layer_index == 4));

    let metal_plt = build_item_material_plt_v1(&two_color_source(), metal).unwrap();
    assert!(metal_plt.pixels.iter().all(|pixel| pixel.layer_index == 2));

    let automatic_plt = build_item_material_plt_v1(&two_color_source(), fallback).unwrap();
    assert_eq!(automatic_plt.pixels[0].layer_index, 2);
    assert_eq!(automatic_plt.pixels[1].layer_index, 4);
}

#[test]
fn duplicate_source_material_mapping_is_rejected() {
    let policy = ItemMaterialLayerMappingPolicyV1 {
        schema_version: 1,
        mappings: vec![
            SourceMaterialLayerMappingV1 {
                source_material_id: 7,
                target_layer: NwnItemPaletteLayerV1::Cloth1,
            },
            SourceMaterialLayerMappingV1 {
                source_material_id: 7,
                target_layer: NwnItemPaletteLayerV1::Metal1,
            },
        ],
        unmapped_policy: UnmappedSourceMaterialPolicyV1::AutomaticClothMetal,
    };

    let error = resolve_item_source_material_layer_v1(&policy, 7).unwrap_err();
    assert_eq!(error.code, "M2A-ITEM-MATERIAL-MAPPING-DUPLICATE");
}

#[test]
fn targeted_palette_luma_lift_changes_only_the_requested_material_layer() {
    let mut image = PltImageV1 {
        schema_version: 1,
        width: 4,
        height: 1,
        num_layers: 8,
        pixels: vec![
            PltPixelV1 {
                color_index: 10,
                layer_index: 5,
            },
            PltPixelV1 {
                color_index: 60,
                layer_index: 5,
            },
            PltPixelV1 {
                color_index: 9,
                layer_index: 2,
            },
            PLT_TRANSPARENT_PIXEL_V1,
        ],
    };

    let changed = lift_item_plt_layer_luma_v1(&mut image, NwnItemPaletteLayerV1::Cloth2, 3, 32)
        .expect("the exact eye-layer lift must be valid");

    assert_eq!(changed, 2);
    assert_eq!(image.pixels[0].color_index, 62);
    assert_eq!(image.pixels[1].color_index, 174);
    assert_eq!(image.pixels[2].color_index, 9);
    assert_eq!(image.pixels[3], PLT_TRANSPARENT_PIXEL_V1);
}
