//! Exact item inventory-icon layer validation and TGA authoring.

use std::collections::{HashMap, HashSet};

use image::{RgbaImage, imageops::FilterType};
use serde::{Deserialize, Serialize};

use crate::item::{
    ITEM_RECIPE_INVALID, ITEM_SCHEMA_INVALID, ITEM_SCHEMA_VERSION, ItemAppearanceRecipeV1,
    ItemCompositionProfileV1, ItemErrorV1, ItemPartSlotV1, resolve_item_resource_names_v1,
    sha256_hex, validate_resref,
};
use crate::plt::{
    PLT_TRANSPARENT_PIXEL_V1, PltImageV1, PltPixelV1, PltWriterOptionsV1,
    plt_image_pixels_sha256_v1, read_plt_image_v1, write_plt_v1,
};
use crate::tga::{
    TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, read_tga_image_v1, write_tga_v1,
};

pub const ITEM_ICON_INVALID: &str = "M2A-ITEM-ICON-INVALID";
pub const ITEM_ICON_SOURCE_STALE: &str = "M2A-ITEM-ICON-SOURCE-STALE";
pub const ITEM_ICON_DIMENSIONS_INVALID: &str = "M2A-ITEM-ICON-DIMENSIONS-INVALID";
pub const ITEM_ICON_SEMANTIC_DIFF: &str = "M2A-ITEM-ICON-SEMANTIC-DIFF";
pub const ITEM_ICON_TGA_RESOURCE_TYPE: u16 = 3;
pub const ITEM_ICON_PLT_RESOURCE_TYPE: u16 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemIconFormatV1 {
    Tga,
    Plt,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemIconLayerInputV1 {
    pub slot: ItemPartSlotV1,
    pub resref: String,
    pub source_sha256: String,
    pub image: TgaImageV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemIconLayerReportV1 {
    pub slot: ItemPartSlotV1,
    pub resref: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub width: u32,
    pub height: u32,
    pub byte_length: u64,
    pub format: ItemIconFormatV1,
    pub resource_type: u16,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemIconLayerArtifactV1 {
    pub slot: ItemPartSlotV1,
    pub resref: String,
    pub payload: Vec<u8>,
    pub report: ItemIconLayerReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPltIconLayerInputV1 {
    pub slot: ItemPartSlotV1,
    pub resref: String,
    pub source_sha256: String,
    pub image: PltImageV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPltIconMaterialMapV1 {
    pub id_rgb: [u8; 3],
    pub layer_index: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPltIconComposeOptionsV1 {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub alpha_threshold: u8,
}

impl Default for ItemPltIconComposeOptionsV1 {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            padding: 5,
            alpha_threshold: 64,
        }
    }
}

/// Composes a palette icon from a beauty render and a material-ID render.
///
/// Foreground identity comes exclusively from the material-ID alpha channel;
/// dark beauty pixels are never treated as background. This keeps black cloth
/// connected to the image edge while still producing an exact transparent
/// PLT border. The detected silhouette is aspect-fitted inside the requested
/// padding before every material ID is mapped to its Aurora palette layer.
pub fn compose_item_plt_icon_from_material_id_v1(
    beauty: &TgaImageV1,
    material_ids: &TgaImageV1,
    mappings: &[ItemPltIconMaterialMapV1],
    options: &ItemPltIconComposeOptionsV1,
) -> Result<PltImageV1, ItemErrorV1> {
    if beauty.schema_version != 1 || material_ids.schema_version != 1 {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "iconComposition.inputs.schemaVersion",
            "beauty and material-ID inputs require schema version 1",
        ));
    }
    if beauty.pixel_format != TgaPixelFormatV1::Rgba8
        || material_ids.pixel_format != TgaPixelFormatV1::Rgba8
    {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "iconComposition.inputs.pixelFormat",
            "beauty and material-ID inputs must both be RGBA8",
        ));
    }
    if beauty.width == 0
        || beauty.height == 0
        || beauty.width != material_ids.width
        || beauty.height != material_ids.height
    {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_DIMENSIONS_INVALID,
            "iconComposition.inputs",
            "beauty and material-ID dimensions must be equal and non-zero",
        ));
    }
    let expected_bytes = usize::try_from(beauty.width)
        .ok()
        .and_then(|width| {
            usize::try_from(beauty.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                "iconComposition.inputs",
                "input dimensions overflow the RGBA8 payload length",
            )
        })?;
    if beauty.pixels.len() != expected_bytes || material_ids.pixels.len() != expected_bytes {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_DIMENSIONS_INVALID,
            "iconComposition.inputs.pixels",
            "input pixel lengths differ from their RGBA8 dimensions",
        ));
    }
    if options.width == 0
        || options.height == 0
        || options.padding.checked_mul(2).is_none()
        || options.padding * 2 >= options.width
        || options.padding * 2 >= options.height
    {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_DIMENSIONS_INVALID,
            "iconComposition.options",
            "output dimensions must leave a non-empty interior after padding",
        ));
    }
    if mappings.is_empty() {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "iconComposition.mappings",
            "at least one material-ID mapping is required",
        ));
    }
    let mut seen_ids = HashSet::with_capacity(mappings.len());
    for (index, mapping) in mappings.iter().enumerate() {
        if !seen_ids.insert(mapping.id_rgb) {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconComposition.mappings[{index}].idRgb"),
                "material-ID colors must be unique",
            ));
        }
        if mapping.layer_index > 9 {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconComposition.mappings[{index}].layerIndex"),
                "Aurora PLT layer index must be between 0 and 9",
            ));
        }
    }

    let source_width = usize::try_from(beauty.width).expect("validated width fits usize");
    let mut minimum = [usize::MAX, usize::MAX];
    let mut maximum = [0usize, 0usize];
    let mut foreground_count = 0usize;
    for (index, rgba) in material_ids.pixels.chunks_exact(4).enumerate() {
        if rgba[3] < options.alpha_threshold {
            continue;
        }
        let x = index % source_width;
        let y = index / source_width;
        minimum[0] = minimum[0].min(x);
        minimum[1] = minimum[1].min(y);
        maximum[0] = maximum[0].max(x);
        maximum[1] = maximum[1].max(y);
        foreground_count += 1;
    }
    if foreground_count == 0 {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "iconComposition.materialIds",
            "material-ID alpha contains no foreground pixels",
        ));
    }

    let crop_width = u32::try_from(maximum[0] - minimum[0] + 1).expect("crop width fits u32");
    let crop_height = u32::try_from(maximum[1] - minimum[1] + 1).expect("crop height fits u32");
    let available_width = options.width - options.padding * 2;
    let available_height = options.height - options.padding * 2;
    let scale = (f64::from(available_width) / f64::from(crop_width))
        .min(f64::from(available_height) / f64::from(crop_height));
    let target_width = ((f64::from(crop_width) * scale).round() as u32)
        .max(1)
        .min(available_width);
    let target_height = ((f64::from(crop_height) * scale).round() as u32)
        .max(1)
        .min(available_height);
    let offset_x = (options.width - target_width) / 2;
    let offset_y = (options.height - target_height) / 2;

    let beauty_image = RgbaImage::from_raw(beauty.width, beauty.height, beauty.pixels.clone())
        .expect("validated RGBA8 beauty dimensions match payload");
    let id_image = RgbaImage::from_raw(
        material_ids.width,
        material_ids.height,
        material_ids.pixels.clone(),
    )
    .expect("validated RGBA8 material-ID dimensions match payload");
    let crop_x = u32::try_from(minimum[0]).expect("crop x fits u32");
    let crop_y = u32::try_from(minimum[1]).expect("crop y fits u32");
    let beauty_crop =
        image::imageops::crop_imm(&beauty_image, crop_x, crop_y, crop_width, crop_height)
            .to_image();
    let id_crop =
        image::imageops::crop_imm(&id_image, crop_x, crop_y, crop_width, crop_height).to_image();
    let resized_beauty = image::imageops::resize(
        &beauty_crop,
        target_width,
        target_height,
        FilterType::Lanczos3,
    );
    let resized_ids =
        image::imageops::resize(&id_crop, target_width, target_height, FilterType::Nearest);
    let output_len = usize::try_from(options.width)
        .ok()
        .and_then(|width| {
            usize::try_from(options.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                "iconComposition.options",
                "output pixel count overflows this platform",
            )
        })?;
    let mut pixels = vec![PLT_TRANSPARENT_PIXEL_V1; output_len];
    let output_width = usize::try_from(options.width).expect("validated output width fits usize");
    for y in 0..target_height {
        for x in 0..target_width {
            let id = resized_ids.get_pixel(x, y).0;
            if id[3] < options.alpha_threshold {
                continue;
            }
            let mapping = mappings
                .iter()
                .min_by_key(|mapping| {
                    mapping
                        .id_rgb
                        .iter()
                        .zip(id[..3].iter())
                        .map(|(expected, actual)| {
                            let delta = i32::from(*expected) - i32::from(*actual);
                            u32::try_from(delta * delta).expect("squared u8 delta fits u32")
                        })
                        .sum::<u32>()
                })
                .expect("non-empty mapping was validated");
            let beauty = resized_beauty.get_pixel(x, y).0;
            let luma = (54u32 * u32::from(beauty[0])
                + 183u32 * u32::from(beauty[1])
                + 19u32 * u32::from(beauty[2]))
                / 256;
            let color_index = ((luma * 174 + 127) / 255) as u8;
            let destination = usize::try_from(offset_y + y).expect("destination y fits usize")
                * output_width
                + usize::try_from(offset_x + x).expect("destination x fits usize");
            pixels[destination] = PltPixelV1 {
                color_index,
                layer_index: mapping.layer_index,
            };
        }
    }
    let num_layers = mappings
        .iter()
        .map(|mapping| mapping.layer_index)
        .max()
        .expect("non-empty mapping was validated")
        .saturating_add(1);
    Ok(PltImageV1 {
        schema_version: 1,
        width: options.width,
        height: options.height,
        num_layers: u32::from(num_layers),
        pixels,
    })
}

pub fn write_item_icon_layers_v1(
    recipe: &ItemAppearanceRecipeV1,
    layers: &[ItemIconLayerInputV1],
    options: &TgaWriterOptionsV1,
) -> Result<Vec<ItemIconLayerArtifactV1>, ItemErrorV1> {
    if matches!(
        recipe.base_item.profile,
        ItemCompositionProfileV1::ModelType1 | ItemCompositionProfileV1::ModelType3
    ) {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "recipe.baseItem.profile",
            "ModelType 1 and 3 inventory icons require Aurora PLT resources",
        ));
    }
    if recipe.schema_version != ITEM_SCHEMA_VERSION {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            "recipe.schemaVersion",
            format!("expected schema version {ITEM_SCHEMA_VERSION}"),
        ));
    }
    let names = resolve_item_resource_names_v1(recipe)?;
    if layers.len() != names.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "iconLayers",
            format!("expected {} icon layers, got {}", names.len(), layers.len()),
        ));
    }
    let width = u32::from(recipe.base_item.inv_slot_width)
        .checked_mul(32)
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                "recipe.baseItem.invSlotWidth",
                "icon width overflows u32",
            )
        })?;
    let height = u32::from(recipe.base_item.inv_slot_height)
        .checked_mul(32)
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                "recipe.baseItem.invSlotHeight",
                "icon height overflows u32",
            )
        })?;
    let expected: HashMap<ItemPartSlotV1, &str> = names
        .iter()
        .map(|name| (name.slot, name.icon_resref.as_str()))
        .collect();
    let mut seen = HashSet::with_capacity(layers.len());
    let mut artifacts = Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        if !seen.insert(layer.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].slot"),
                "duplicate icon layer slot",
            ));
        }
        let expected_resref = expected.get(&layer.slot).ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].slot"),
                "icon slot does not belong to the selected profile",
            )
        })?;
        validate_resref(&layer.resref, &format!("iconLayers[{index}].resref"))?;
        if layer.resref != *expected_resref {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].resref"),
                format!("expected exact resolved icon ResRef {expected_resref}"),
            ));
        }
        let actual_source_sha256 = sha256_hex(&layer.image.pixels);
        if layer.source_sha256 != actual_source_sha256 {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_SOURCE_STALE,
                format!("iconLayers[{index}].sourceSha256"),
                format!(
                    "expected {}, read {} from canonical pixels",
                    layer.source_sha256, actual_source_sha256
                ),
            ));
        }
        if layer.image.width != width || layer.image.height != height {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                format!("iconLayers[{index}].image"),
                format!(
                    "expected {}x{} from baseitems inventory slots, got {}x{}",
                    width, height, layer.image.width, layer.image.height
                ),
            ));
        }
        if layer.image.pixel_format != TgaPixelFormatV1::Rgba8 {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].image.pixelFormat"),
                "item icon layers require RGBA8 so alpha is explicit",
            ));
        }
        let artifact = write_tga_v1(&layer.image, options).map_err(|error| {
            ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}]"),
                format!("{}: {}", error.code, error.message),
            )
        })?;
        let readback = read_tga_image_v1(&artifact.payload).map_err(|error| {
            ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].readback"),
                format!("{}: {}", error.code, error.message),
            )
        })?;
        if readback != layer.image {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_SEMANTIC_DIFF,
                format!("iconLayers[{index}].readback"),
                "TGA readback differs from the exact canonical input pixels",
            ));
        }
        artifacts.push(ItemIconLayerArtifactV1 {
            slot: layer.slot,
            resref: layer.resref.clone(),
            payload: artifact.payload,
            report: ItemIconLayerReportV1 {
                slot: layer.slot,
                resref: layer.resref.clone(),
                source_sha256: layer.source_sha256.clone(),
                output_sha256: artifact.report.output_sha256,
                width,
                height,
                byte_length: artifact.report.byte_length,
                format: ItemIconFormatV1::Tga,
                resource_type: ITEM_ICON_TGA_RESOURCE_TYPE,
                semantic_readback_status: "PASS".to_owned(),
            },
        });
    }
    for name in &names {
        if !seen.contains(&name.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                "iconLayers",
                format!("required icon layer {:?} is missing", name.slot),
            ));
        }
    }
    artifacts.sort_by_key(|artifact| {
        names
            .iter()
            .position(|name| name.slot == artifact.slot)
            .expect("every artifact belongs to resolved names")
    });
    Ok(artifacts)
}

pub fn write_item_plt_icon_layers_v1(
    recipe: &ItemAppearanceRecipeV1,
    layers: &[ItemPltIconLayerInputV1],
    options: &PltWriterOptionsV1,
) -> Result<Vec<ItemIconLayerArtifactV1>, ItemErrorV1> {
    if recipe.schema_version != ITEM_SCHEMA_VERSION {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            "recipe.schemaVersion",
            format!("expected schema version {ITEM_SCHEMA_VERSION}"),
        ));
    }
    if !matches!(
        recipe.base_item.profile,
        ItemCompositionProfileV1::ModelType1 | ItemCompositionProfileV1::ModelType3
    ) {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "recipe.baseItem.profile",
            "Aurora PLT inventory icons are required only for ModelType 1 and 3",
        ));
    }
    let names = resolve_item_resource_names_v1(recipe)?;
    if layers.len() != names.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_ICON_INVALID,
            "iconLayers",
            format!("expected {} icon layers, got {}", names.len(), layers.len()),
        ));
    }
    let width = u32::from(recipe.base_item.inv_slot_width)
        .checked_mul(32)
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                "recipe.baseItem.invSlotWidth",
                "icon width overflows u32",
            )
        })?;
    let height = u32::from(recipe.base_item.inv_slot_height)
        .checked_mul(32)
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                "recipe.baseItem.invSlotHeight",
                "icon height overflows u32",
            )
        })?;
    let expected: HashMap<ItemPartSlotV1, &str> = names
        .iter()
        .map(|name| (name.slot, name.icon_resref.as_str()))
        .collect();
    let mut seen = HashSet::with_capacity(layers.len());
    let mut artifacts = Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        if !seen.insert(layer.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].slot"),
                "duplicate icon layer slot",
            ));
        }
        let expected_resref = expected.get(&layer.slot).ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].slot"),
                "icon slot does not belong to the selected profile",
            )
        })?;
        validate_resref(&layer.resref, &format!("iconLayers[{index}].resref"))?;
        if layer.resref != *expected_resref {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].resref"),
                format!("expected exact resolved icon ResRef {expected_resref}"),
            ));
        }
        let actual_source_sha256 = plt_image_pixels_sha256_v1(&layer.image);
        if layer.source_sha256 != actual_source_sha256 {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_SOURCE_STALE,
                format!("iconLayers[{index}].sourceSha256"),
                format!(
                    "expected {}, read {} from canonical PLT pixels",
                    layer.source_sha256, actual_source_sha256
                ),
            ));
        }
        if layer.image.width != width || layer.image.height != height {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_DIMENSIONS_INVALID,
                format!("iconLayers[{index}].image"),
                format!(
                    "expected {}x{} from baseitems inventory slots, got {}x{}",
                    width, height, layer.image.width, layer.image.height
                ),
            ));
        }
        let artifact = write_plt_v1(&layer.image, options).map_err(|error| {
            ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}]"),
                format!("{}: {}", error.code, error.message),
            )
        })?;
        let readback = read_plt_image_v1(&artifact.payload).map_err(|error| {
            ItemErrorV1::fatal(
                ITEM_ICON_INVALID,
                format!("iconLayers[{index}].readback"),
                format!("{}: {}", error.code, error.message),
            )
        })?;
        if readback != layer.image {
            return Err(ItemErrorV1::fatal(
                ITEM_ICON_SEMANTIC_DIFF,
                format!("iconLayers[{index}].readback"),
                "PLT readback differs from the exact canonical input pixels",
            ));
        }
        artifacts.push(ItemIconLayerArtifactV1 {
            slot: layer.slot,
            resref: layer.resref.clone(),
            payload: artifact.payload,
            report: ItemIconLayerReportV1 {
                slot: layer.slot,
                resref: layer.resref.clone(),
                source_sha256: layer.source_sha256.clone(),
                output_sha256: artifact.report.output_sha256,
                width,
                height,
                byte_length: artifact.report.byte_length,
                format: ItemIconFormatV1::Plt,
                resource_type: ITEM_ICON_PLT_RESOURCE_TYPE,
                semantic_readback_status: "PASS".to_owned(),
            },
        });
    }
    for name in &names {
        if !seen.contains(&name.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                "iconLayers",
                format!("required icon layer {:?} is missing", name.slot),
            ));
        }
    }
    artifacts.sort_by_key(|artifact| {
        names
            .iter()
            .position(|name| name.slot == artifact.slot)
            .expect("every artifact belongs to resolved names")
    });
    Ok(artifacts)
}
