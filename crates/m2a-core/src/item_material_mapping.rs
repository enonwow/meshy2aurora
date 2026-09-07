//! Explicit source-material to NWN Item PLT-layer mapping.
//!
//! An authored mapping always wins. Only an unmapped source material may use
//! the compatibility classifier that historically inferred Metal 1 versus
//! Cloth 1 per texture pixel.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    plt::{PLT_SCHEMA_VERSION, PltImageV1, PltPixelV1},
    tga::{TGA_SCHEMA_VERSION, TgaImageV1, TgaPixelFormatV1},
};

pub const ITEM_MATERIAL_LAYER_MAPPING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum NwnItemPaletteLayerV1 {
    Skin = 0,
    Hair = 1,
    Metal1 = 2,
    Metal2 = 3,
    Cloth1 = 4,
    Cloth2 = 5,
    Leather1 = 6,
    Leather2 = 7,
}

impl NwnItemPaletteLayerV1 {
    pub const fn layer_index(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceMaterialLayerMappingV1 {
    pub source_material_id: u32,
    pub target_layer: NwnItemPaletteLayerV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnmappedSourceMaterialPolicyV1 {
    AutomaticClothMetal,
    Reject,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemMaterialLayerMappingPolicyV1 {
    pub schema_version: u32,
    pub mappings: Vec<SourceMaterialLayerMappingV1>,
    pub unmapped_policy: UnmappedSourceMaterialPolicyV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemMaterialMappingErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl ItemMaterialMappingErrorV1 {
    fn new(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: ITEM_MATERIAL_LAYER_MAPPING_SCHEMA_VERSION,
            code: code.to_owned(),
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ItemMaterialMappingErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ItemMaterialMappingErrorV1 {}

pub fn resolve_item_source_material_layer_v1(
    policy: &ItemMaterialLayerMappingPolicyV1,
    source_material_id: u32,
) -> Result<Option<NwnItemPaletteLayerV1>, ItemMaterialMappingErrorV1> {
    if policy.schema_version != ITEM_MATERIAL_LAYER_MAPPING_SCHEMA_VERSION {
        return Err(ItemMaterialMappingErrorV1::new(
            "M2A-ITEM-MATERIAL-MAPPING-SCHEMA-INVALID",
            "policy.schemaVersion",
            format!(
                "expected schema version {ITEM_MATERIAL_LAYER_MAPPING_SCHEMA_VERSION}, got {}",
                policy.schema_version
            ),
        ));
    }

    let mut seen = BTreeSet::new();
    for (index, mapping) in policy.mappings.iter().enumerate() {
        if !seen.insert(mapping.source_material_id) {
            return Err(ItemMaterialMappingErrorV1::new(
                "M2A-ITEM-MATERIAL-MAPPING-DUPLICATE",
                format!("policy.mappings[{index}].sourceMaterialId").as_str(),
                format!(
                    "source material {} has more than one target layer",
                    mapping.source_material_id
                ),
            ));
        }
    }

    if let Some(mapping) = policy
        .mappings
        .iter()
        .find(|mapping| mapping.source_material_id == source_material_id)
    {
        return Ok(Some(mapping.target_layer));
    }

    match policy.unmapped_policy {
        UnmappedSourceMaterialPolicyV1::AutomaticClothMetal => Ok(None),
        UnmappedSourceMaterialPolicyV1::Reject => Err(ItemMaterialMappingErrorV1::new(
            "M2A-ITEM-MATERIAL-MAPPING-MISSING",
            "sourceMaterialId",
            format!("source material {source_material_id} has no explicit NWN layer mapping"),
        )),
    }
}

pub fn build_item_material_plt_v1(
    source: &TgaImageV1,
    explicit_layer: Option<NwnItemPaletteLayerV1>,
) -> Result<PltImageV1, ItemMaterialMappingErrorV1> {
    if source.schema_version != TGA_SCHEMA_VERSION {
        return Err(ItemMaterialMappingErrorV1::new(
            "M2A-ITEM-MATERIAL-TEXTURE-SCHEMA-INVALID",
            "source.schemaVersion",
            format!("expected TGA schema version {TGA_SCHEMA_VERSION}"),
        ));
    }
    let channels = match source.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3usize,
        TgaPixelFormatV1::Rgba8 => 4usize,
    };
    let pixel_count = usize::try_from(source.width)
        .ok()
        .and_then(|width| {
            usize::try_from(source.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or_else(|| {
            ItemMaterialMappingErrorV1::new(
                "M2A-ITEM-MATERIAL-TEXTURE-DIMENSIONS-INVALID",
                "source",
                "texture dimensions overflow usize",
            )
        })?;
    let expected_length = pixel_count.checked_mul(channels).ok_or_else(|| {
        ItemMaterialMappingErrorV1::new(
            "M2A-ITEM-MATERIAL-TEXTURE-DIMENSIONS-INVALID",
            "source.pixels",
            "texture byte length overflows usize",
        )
    })?;
    if source.width == 0 || source.height == 0 || source.pixels.len() != expected_length {
        return Err(ItemMaterialMappingErrorV1::new(
            "M2A-ITEM-MATERIAL-TEXTURE-DIMENSIONS-INVALID",
            "source.pixels",
            format!(
                "expected {expected_length} bytes for {}x{}, got {}",
                source.width,
                source.height,
                source.pixels.len()
            ),
        ));
    }

    let pixels = source
        .pixels
        .chunks_exact(channels)
        .map(|rgba| {
            let inferred_layer = if is_warm_metal_pixel(rgba[0], rgba[1], rgba[2]) {
                NwnItemPaletteLayerV1::Metal1
            } else {
                NwnItemPaletteLayerV1::Cloth1
            };
            PltPixelV1 {
                color_index: plt_color_index(rgba[0], rgba[1], rgba[2]),
                layer_index: explicit_layer.unwrap_or(inferred_layer).layer_index(),
            }
        })
        .collect::<Vec<_>>();
    let highest_layer = explicit_layer
        .map(NwnItemPaletteLayerV1::layer_index)
        .unwrap_or(NwnItemPaletteLayerV1::Cloth1.layer_index());

    Ok(PltImageV1 {
        schema_version: PLT_SCHEMA_VERSION,
        width: source.width,
        height: source.height,
        num_layers: u32::from(highest_layer).saturating_add(1).max(5),
        pixels,
    })
}

/// Raises the usable palette-ramp range for one explicitly mapped PLT layer.
///
/// Mesh generators can emit materially distinct parts whose baked albedo is
/// so dark that even a vivid Aurora color row remains visually black. This
/// transform keeps every other layer byte-exact and preserves relative detail
/// within the selected layer while clamping to Aurora's authored 0..=174 ramp.
pub fn lift_item_plt_layer_luma_v1(
    image: &mut PltImageV1,
    target_layer: NwnItemPaletteLayerV1,
    multiplier: u16,
    offset: u8,
) -> Result<usize, ItemMaterialMappingErrorV1> {
    let expected_pixels = usize::try_from(image.width)
        .ok()
        .and_then(|width| {
            usize::try_from(image.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or_else(|| {
            ItemMaterialMappingErrorV1::new(
                "M2A-ITEM-MATERIAL-PLT-DIMENSIONS-INVALID",
                "image",
                "PLT dimensions overflow usize",
            )
        })?;
    if image.schema_version != PLT_SCHEMA_VERSION
        || image.width == 0
        || image.height == 0
        || image.pixels.len() != expected_pixels
        || image.num_layers <= u32::from(target_layer.layer_index())
        || multiplier == 0
    {
        return Err(ItemMaterialMappingErrorV1::new(
            "M2A-ITEM-MATERIAL-PLT-LUMA-LIFT-INVALID",
            "image",
            "valid PLT dimensions, target layer and a non-zero multiplier are required",
        ));
    }

    let mut changed = 0usize;
    for pixel in &mut image.pixels {
        if pixel.layer_index != target_layer.layer_index() {
            continue;
        }
        let lifted = u32::from(pixel.color_index)
            .saturating_mul(u32::from(multiplier))
            .saturating_add(u32::from(offset))
            .min(174);
        pixel.color_index = u8::try_from(lifted).expect("0..=174 fits u8");
        changed += 1;
    }
    Ok(changed)
}

fn plt_color_index(red: u8, green: u8, blue: u8) -> u8 {
    let luma = (54u32 * u32::from(red) + 183u32 * u32::from(green) + 19u32 * u32::from(blue)) / 256;
    ((luma * 174 + 127) / 255) as u8
}

fn is_warm_metal_pixel(red: u8, green: u8, blue: u8) -> bool {
    i16::from(red) - i16::from(blue) >= 10 && i16::from(green) - i16::from(blue) >= 2
}
