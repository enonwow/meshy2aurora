//! Aurora-first contracts shared by item authoring, UTI generation and packaging.
//!
//! `baseitems.2da.ModelType` is the only composition discriminator.  Product
//! categories such as weapon or shield deliberately do not appear here.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::model_limits::{
    AURORA_MODEL_TRIANGLE_BUDGET_V1, AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
};
use crate::two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2};

pub const ITEM_SCHEMA_VERSION: u32 = 1;

pub const ITEM_SCHEMA_INVALID: &str = "M2A-ITEM-SCHEMA-INVALID";
pub const ITEM_BASEITEMS_INVALID: &str = "M2A-ITEM-BASEITEMS-INVALID";
pub const ITEM_BASEITEMS_STALE: &str = "M2A-ITEM-BASEITEMS-STALE";
pub const ITEM_PROFILE_INVALID: &str = "M2A-ITEM-PROFILE-INVALID";
pub const ITEM_RECIPE_INVALID: &str = "M2A-ITEM-RECIPE-INVALID";
pub const ITEM_RESREF_INVALID: &str = "M2A-ITEM-RESREF-INVALID";
pub const ITEM_NAMING_UNPROVEN: &str = "M2A-ITEM-NAMING-UNPROVEN";
pub const ITEM_NAMESPACE_INCOMPLETE: &str = "M2A-ITEM-NAMESPACE-INCOMPLETE";
pub const ITEM_RESOURCE_COLLISION: &str = "M2A-ITEM-RESOURCE-COLLISION";
pub const ITEM_TRIANGLE_COUNT_OVERFLOW: &str = "M2A-ITEM-TRIANGLE-COUNT-OVERFLOW";
pub const ITEM_TRIANGLE_BUDGET_EXCEEDED: &str = "M2A-ITEM-TRIANGLE-BUDGET-EXCEEDED";
pub const ITEM_MODEL_VARIANT_OUT_OF_RANGE: &str = "M2A-ITEM-MODEL-VARIANT-OUT-OF-RANGE";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

impl ItemErrorV1 {
    pub(crate) fn fatal(code: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            schema_version: ITEM_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ItemErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ItemErrorV1 {}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemCompositionProfileV1 {
    ModelType0,
    ModelType1,
    ModelType2,
    ModelType3,
}

impl ItemCompositionProfileV1 {
    pub const fn model_type(self) -> u8 {
        match self {
            Self::ModelType0 => 0,
            Self::ModelType1 => 1,
            Self::ModelType2 => 2,
            Self::ModelType3 => 3,
        }
    }

    pub fn from_model_type(model_type: u8) -> Result<Self, ItemErrorV1> {
        match model_type {
            0 => Ok(Self::ModelType0),
            1 => Ok(Self::ModelType1),
            2 => Ok(Self::ModelType2),
            3 => Ok(Self::ModelType3),
            _ => Err(ItemErrorV1::fatal(
                ITEM_PROFILE_INVALID,
                "baseItem.modelType",
                format!("unsupported baseitems.2da ModelType {model_type}"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemPartSlotV1 {
    Model,
    Bottom,
    Middle,
    Top,
    ArmorRFoot,
    ArmorLFoot,
    ArmorRShin,
    ArmorLShin,
    ArmorLThigh,
    ArmorRThigh,
    ArmorPelvis,
    ArmorTorso,
    ArmorBelt,
    ArmorNeck,
    ArmorRForearm,
    ArmorLForearm,
    ArmorRBicep,
    ArmorLBicep,
    ArmorRShoulder,
    ArmorLShoulder,
    ArmorRHand,
    ArmorLHand,
    ArmorRobe,
}

const SINGLE_PART_SLOTS: &[ItemPartSlotV1] = &[ItemPartSlotV1::Model];
const THREE_PART_SLOTS: &[ItemPartSlotV1] = &[
    ItemPartSlotV1::Bottom,
    ItemPartSlotV1::Middle,
    ItemPartSlotV1::Top,
];
pub const ARMOR_PART_SLOTS_V1: &[ItemPartSlotV1] = &[
    ItemPartSlotV1::ArmorRFoot,
    ItemPartSlotV1::ArmorLFoot,
    ItemPartSlotV1::ArmorRShin,
    ItemPartSlotV1::ArmorLShin,
    ItemPartSlotV1::ArmorLThigh,
    ItemPartSlotV1::ArmorRThigh,
    ItemPartSlotV1::ArmorPelvis,
    ItemPartSlotV1::ArmorTorso,
    ItemPartSlotV1::ArmorBelt,
    ItemPartSlotV1::ArmorNeck,
    ItemPartSlotV1::ArmorRForearm,
    ItemPartSlotV1::ArmorLForearm,
    ItemPartSlotV1::ArmorRBicep,
    ItemPartSlotV1::ArmorLBicep,
    ItemPartSlotV1::ArmorRShoulder,
    ItemPartSlotV1::ArmorLShoulder,
    ItemPartSlotV1::ArmorRHand,
    ItemPartSlotV1::ArmorLHand,
    ItemPartSlotV1::ArmorRobe,
];

pub const fn required_item_part_slots_v1(
    profile: ItemCompositionProfileV1,
) -> &'static [ItemPartSlotV1] {
    match profile {
        ItemCompositionProfileV1::ModelType0 | ItemCompositionProfileV1::ModelType1 => {
            SINGLE_PART_SLOTS
        }
        ItemCompositionProfileV1::ModelType2 => THREE_PART_SLOTS,
        ItemCompositionProfileV1::ModelType3 => ARMOR_PART_SLOTS_V1,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemBaseRecordV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub physical_row_index: u32,
    pub printed_row_label: u32,
    pub profile: ItemCompositionProfileV1,
    pub model_type: u8,
    pub item_class: String,
    pub gender_specific: bool,
    pub inv_slot_width: u16,
    pub inv_slot_height: u16,
    pub equipable_slots: String,
    pub default_model: Option<String>,
    pub default_icon: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemBaseModelRangeV1 {
    pub min_range: u8,
    pub max_range: u8,
}

/// Resolves one exact physical `baseitems.2da` row and binds it to the expected
/// source hash. No defaults are invented for missing or null required cells.
pub fn resolve_item_base_record_v1(
    bytes: &[u8],
    physical_row_index: u32,
    expected_source_sha256: &str,
    limits: &TwoDaLimitsV1,
) -> Result<ItemBaseRecordV1, ItemErrorV1> {
    validate_sha256(expected_source_sha256, "expectedSourceSha256")?;
    let inspection = inspect_two_da_v2(bytes, limits).map_err(|error| {
        ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            "baseitems2da",
            format!("{}: {}", error.code, error.message),
        )
    })?;
    if inspection.source_sha256 != expected_source_sha256 {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_STALE,
            "expectedSourceSha256",
            format!(
                "expected {}, read {}",
                expected_source_sha256, inspection.source_sha256
            ),
        ));
    }
    let row = read_two_da_row_v2(bytes, physical_row_index, limits).map_err(|error| {
        ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("rows[{physical_row_index}]"),
            format!("{}: {}", error.code, error.message),
        )
    })?;
    if row.printed_row_label != physical_row_index {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("rows[{physical_row_index}].label"),
            format!(
                "printed row label {} does not equal BaseItem index {physical_row_index}",
                row.printed_row_label
            ),
        ));
    }
    if inspection.columns.len() != row.cells.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("rows[{physical_row_index}]"),
            "validated row width differs from the declared schema",
        ));
    }
    let cells: HashMap<&str, &TwoDaCellValueV1> = inspection
        .columns
        .iter()
        .map(String::as_str)
        .zip(row.cells.iter())
        .collect();

    let item_class = required_cell(&cells, "ItemClass", physical_row_index)?.to_owned();
    validate_item_class(&item_class, "baseItem.itemClass")?;
    let model_type = parse_u8(
        required_cell(&cells, "ModelType", physical_row_index)?,
        "baseItem.modelType",
    )?;
    let profile = ItemCompositionProfileV1::from_model_type(model_type)?;
    let gender_specific = match required_cell(&cells, "GenderSpecific", physical_row_index)? {
        "0" => false,
        "1" => true,
        value => {
            return Err(ItemErrorV1::fatal(
                ITEM_BASEITEMS_INVALID,
                "baseItem.genderSpecific",
                format!("expected 0 or 1, got {value}"),
            ));
        }
    };
    let inv_slot_width = parse_positive_u16(
        required_cell(&cells, "InvSlotWidth", physical_row_index)?,
        "baseItem.invSlotWidth",
    )?;
    let inv_slot_height = parse_positive_u16(
        required_cell(&cells, "InvSlotHeight", physical_row_index)?,
        "baseItem.invSlotHeight",
    )?;
    let equipable_slots = required_cell(&cells, "EquipableSlots", physical_row_index)?.to_owned();
    let default_model = optional_cell(&cells, "DefaultModel", physical_row_index)?;
    let default_icon = optional_cell(&cells, "DefaultIcon", physical_row_index)?;

    Ok(ItemBaseRecordV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        source_sha256: inspection.source_sha256,
        physical_row_index,
        printed_row_label: row.printed_row_label,
        profile,
        model_type,
        item_class,
        gender_specific,
        inv_slot_width,
        inv_slot_height,
        equipable_slots,
        default_model,
        default_icon,
    })
}

/// Confirms that every numeric model part in a recipe is discoverable by the
/// Toolset's exact `baseitems.2da` MinRange/MaxRange scan. A model resource can
/// be present in a HAK and still be absent from the Appearance combo when its
/// variant lies outside this range.
pub fn validate_item_model_variants_in_baseitems_v1(
    bytes: &[u8],
    recipe: &ItemAppearanceRecipeV1,
    limits: &TwoDaLimitsV1,
) -> Result<ItemBaseModelRangeV1, ItemErrorV1> {
    validate_item_recipe_v1(recipe)?;
    let resolved = resolve_item_base_record_v1(
        bytes,
        recipe.base_item.physical_row_index,
        &recipe.base_item.source_sha256,
        limits,
    )?;
    if resolved != recipe.base_item {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_STALE,
            "recipe.baseItem",
            "recipe base-item record differs from the exact effective baseitems.2da row",
        ));
    }

    let inspection = inspect_two_da_v2(bytes, limits).map_err(|error| {
        ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            "baseitems2da",
            format!("{}: {}", error.code, error.message),
        )
    })?;
    let row = read_two_da_row_v2(bytes, recipe.base_item.physical_row_index, limits).map_err(
        |error| {
            ItemErrorV1::fatal(
                ITEM_BASEITEMS_INVALID,
                format!("rows[{}]", recipe.base_item.physical_row_index),
                format!("{}: {}", error.code, error.message),
            )
        },
    )?;
    if inspection.columns.len() != row.cells.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("rows[{}]", recipe.base_item.physical_row_index),
            "validated row width differs from the declared schema",
        ));
    }
    let cells: HashMap<&str, &TwoDaCellValueV1> = inspection
        .columns
        .iter()
        .map(String::as_str)
        .zip(row.cells.iter())
        .collect();
    let min_range = parse_u8(
        required_cell(&cells, "MinRange", recipe.base_item.physical_row_index)?,
        "baseItem.minRange",
    )?;
    let max_range = parse_u8(
        required_cell(&cells, "MaxRange", recipe.base_item.physical_row_index)?,
        "baseItem.maxRange",
    )?;
    if min_range > max_range {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            "baseItem.modelRange",
            format!("MinRange {min_range} exceeds MaxRange {max_range}"),
        ));
    }
    for (index, part) in recipe.parts.iter().enumerate() {
        if !(min_range..=max_range).contains(&part.variant) {
            return Err(ItemErrorV1::fatal(
                ITEM_MODEL_VARIANT_OUT_OF_RANGE,
                format!("recipe.parts[{index}].variant"),
                format!(
                    "variant {} is outside effective baseitems.2da range {min_range}..={max_range}",
                    part.variant
                ),
            ));
        }
    }

    Ok(ItemBaseModelRangeV1 {
        min_range,
        max_range,
    })
}

fn required_cell<'a>(
    cells: &HashMap<&str, &'a TwoDaCellValueV1>,
    column: &str,
    row: u32,
) -> Result<&'a str, ItemErrorV1> {
    match cells.get(column) {
        Some(TwoDaCellValueV1::Text { value }) => Ok(value),
        Some(TwoDaCellValueV1::Null) => Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("rows[{row}].{column}"),
            "required baseitems.2da cell is null",
        )),
        None => Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("columns.{column}"),
            "required baseitems.2da column is missing",
        )),
    }
}

fn optional_cell(
    cells: &HashMap<&str, &TwoDaCellValueV1>,
    column: &str,
    row: u32,
) -> Result<Option<String>, ItemErrorV1> {
    match cells.get(column) {
        Some(TwoDaCellValueV1::Text { value }) => Ok(Some(value.clone())),
        Some(TwoDaCellValueV1::Null) => Ok(None),
        None => Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            format!("rows[{row}].{column}"),
            "required baseitems.2da schema column is missing",
        )),
    }
}

fn parse_u8(value: &str, path: &str) -> Result<u8, ItemErrorV1> {
    value.parse::<u8>().map_err(|_| {
        ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            path,
            format!("expected an unsigned byte, got {value}"),
        )
    })
}

fn parse_positive_u16(value: &str, path: &str) -> Result<u16, ItemErrorV1> {
    let parsed = value.parse::<u16>().map_err(|_| {
        ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            path,
            format!("expected a positive u16, got {value}"),
        )
    })?;
    if parsed == 0 {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            path,
            "value must be greater than zero",
        ));
    }
    Ok(parsed)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemGenderV1 {
    Male,
    Female,
}

impl ItemGenderV1 {
    const fn suffix(self) -> char {
        match self {
            Self::Male => 'm',
            Self::Female => 'f',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartTransformV1 {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
}

impl Default for ItemPartTransformV1 {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartRecipeV1 {
    pub slot: ItemPartSlotV1,
    pub variant: u8,
    pub source_part_id: String,
    pub source_sha256: String,
    pub transform: ItemPartTransformV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemColorRecipeV1 {
    pub leather1: u8,
    pub leather2: u8,
    pub cloth1: u8,
    pub cloth2: u8,
    pub metal1: u8,
    pub metal2: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemIdentityV1 {
    pub uti_resref: String,
    pub tag: String,
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemAppearanceRecipeV1 {
    pub schema_version: u32,
    pub base_item: ItemBaseRecordV1,
    pub identity: ItemIdentityV1,
    pub gender: Option<ItemGenderV1>,
    pub parts: Vec<ItemPartRecipeV1>,
    pub colors: Option<ItemColorRecipeV1>,
}

pub fn validate_item_recipe_v1(recipe: &ItemAppearanceRecipeV1) -> Result<(), ItemErrorV1> {
    if recipe.schema_version != ITEM_SCHEMA_VERSION
        || recipe.base_item.schema_version != ITEM_SCHEMA_VERSION
    {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            "recipe.schemaVersion",
            format!("expected schema version {ITEM_SCHEMA_VERSION}"),
        ));
    }
    if recipe.base_item.model_type != recipe.base_item.profile.model_type() {
        return Err(ItemErrorV1::fatal(
            ITEM_PROFILE_INVALID,
            "recipe.baseItem.profile",
            "resolved profile does not match ModelType",
        ));
    }
    validate_sha256(
        &recipe.base_item.source_sha256,
        "recipe.baseItem.sourceSha256",
    )?;
    validate_resref(&recipe.identity.uti_resref, "recipe.identity.utiResref")?;
    if recipe.identity.tag.is_empty()
        || recipe.identity.tag.len() > 32
        || !recipe
            .identity
            .tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "recipe.identity.tag",
            "tag must contain 1..32 ASCII alphanumeric/underscore characters",
        ));
    }
    if recipe.identity.display_name.is_empty() || recipe.identity.display_name.len() > 256 {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "recipe.identity.displayName",
            "display name must contain 1..256 UTF-8 bytes",
        ));
    }
    if recipe.base_item.gender_specific != recipe.gender.is_some() {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "recipe.gender",
            "gender must be present exactly when baseitems.GenderSpecific is 1",
        ));
    }

    let required = required_item_part_slots_v1(recipe.base_item.profile);
    if recipe.parts.len() != required.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "recipe.parts",
            format!(
                "profile {:?} requires exactly {} parts, got {}",
                recipe.base_item.profile,
                required.len(),
                recipe.parts.len()
            ),
        ));
    }
    let mut slots = HashSet::with_capacity(recipe.parts.len());
    for (index, part) in recipe.parts.iter().enumerate() {
        if !slots.insert(part.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                format!("recipe.parts[{index}].slot"),
                "duplicate item part slot",
            ));
        }
        if !required.contains(&part.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                format!("recipe.parts[{index}].slot"),
                "slot does not belong to the resolved ModelType profile",
            ));
        }
        if part.source_part_id.is_empty() || part.source_part_id.len() > 128 {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                format!("recipe.parts[{index}].sourcePartId"),
                "source part id must contain 1..128 bytes",
            ));
        }
        validate_sha256(
            &part.source_sha256,
            &format!("recipe.parts[{index}].sourceSha256"),
        )?;
        validate_transform(&part.transform, index)?;
    }
    for slot in required {
        if !slots.contains(slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                "recipe.parts",
                format!("required slot {slot:?} is missing"),
            ));
        }
    }
    let colors_required = matches!(
        recipe.base_item.profile,
        ItemCompositionProfileV1::ModelType1 | ItemCompositionProfileV1::ModelType3
    );
    if colors_required != recipe.colors.is_some() {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "recipe.colors",
            "six colors must be present exactly for ModelType 1 and 3",
        ));
    }
    Ok(())
}

fn validate_transform(transform: &ItemPartTransformV1, index: usize) -> Result<(), ItemErrorV1> {
    if transform
        .translation
        .iter()
        .chain(transform.rotation.iter())
        .any(|value| !value.is_finite())
        || !transform.scale.is_finite()
        || transform.scale <= 0.0
    {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            format!("recipe.parts[{index}].transform"),
            "transform values must be finite and uniform scale must be greater than zero",
        ));
    }
    let norm_squared: f32 = transform.rotation.iter().map(|value| value * value).sum();
    if (norm_squared - 1.0).abs() > 0.0001 {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            format!("recipe.parts[{index}].transform.rotation"),
            "rotation quaternion must be normalized",
        ));
    }
    Ok(())
}

pub fn canonical_item_recipe_json_v1(
    recipe: &ItemAppearanceRecipeV1,
) -> Result<Vec<u8>, ItemErrorV1> {
    validate_item_recipe_v1(recipe)?;
    serde_json::to_vec(recipe).map_err(|error| {
        ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "recipe",
            format!("canonical JSON serialization failed: {error}"),
        )
    })
}

pub fn item_recipe_sha256_v1(recipe: &ItemAppearanceRecipeV1) -> Result<String, ItemErrorV1> {
    Ok(sha256_hex(&canonical_item_recipe_json_v1(recipe)?))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResourceNameV1 {
    pub slot: ItemPartSlotV1,
    pub variant: u8,
    pub model_resref: String,
    pub icon_resref: String,
}

/// Resolves the generic item naming function proven in the Toolset
/// decompilation. Armor body-part naming is intentionally blocked until its
/// race/phenotype/gender namespace is represented by a separate proven input.
pub fn resolve_item_resource_names_v1(
    recipe: &ItemAppearanceRecipeV1,
) -> Result<Vec<ItemResourceNameV1>, ItemErrorV1> {
    validate_item_recipe_v1(recipe)?;
    if recipe.base_item.profile == ItemCompositionProfileV1::ModelType3 {
        return Err(ItemErrorV1::fatal(
            ITEM_NAMING_UNPROVEN,
            "recipe.baseItem.profile",
            "armor body-part naming requires a proven race/phenotype/gender namespace",
        ));
    }
    let item_class = recipe.base_item.item_class.to_ascii_lowercase();
    let mut names = Vec::with_capacity(recipe.parts.len());
    for part in &recipe.parts {
        let suffix = match recipe.base_item.profile {
            ItemCompositionProfileV1::ModelType2 => match part.slot {
                ItemPartSlotV1::Bottom => format!("b_{:03}", part.variant),
                ItemPartSlotV1::Middle => format!("m_{:03}", part.variant),
                ItemPartSlotV1::Top => format!("t_{:03}", part.variant),
                _ => unreachable!("recipe validation restricts ModelType2 slots"),
            },
            ItemCompositionProfileV1::ModelType0 | ItemCompositionProfileV1::ModelType1 => {
                if let Some(gender) = recipe.gender {
                    format!("{}_{:03}", gender.suffix(), part.variant)
                } else {
                    format!("{:03}", part.variant)
                }
            }
            ItemCompositionProfileV1::ModelType3 => unreachable!(),
        };
        let model_resref = format!("{item_class}_{suffix}");
        let icon_resref = format!("i{model_resref}");
        validate_resref(&model_resref, "resolvedNames.modelResref")?;
        validate_resref(&icon_resref, "resolvedNames.iconResref")?;
        names.push(ItemResourceNameV1 {
            slot: part.slot,
            variant: part.variant,
            model_resref,
            icon_resref,
        });
    }
    Ok(names)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemResourceScopeV1 {
    Retail,
    Hak,
    Module,
    Output,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResourceKeyV1 {
    pub resref: String,
    pub resource_type: u16,
    pub scope: ItemResourceScopeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveResourceNamespaceV1 {
    pub schema_version: u32,
    pub complete: bool,
    pub inventories_sha256: Vec<String>,
    pub resources: Vec<ItemResourceKeyV1>,
}

pub fn preflight_item_namespace_v1(
    output: &[ItemResourceKeyV1],
    namespace: &EffectiveResourceNamespaceV1,
) -> Result<(), ItemErrorV1> {
    if namespace.schema_version != ITEM_SCHEMA_VERSION {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            "namespace.schemaVersion",
            format!("expected schema version {ITEM_SCHEMA_VERSION}"),
        ));
    }
    if !namespace.complete {
        return Err(ItemErrorV1::fatal(
            ITEM_NAMESPACE_INCOMPLETE,
            "namespace.complete",
            "REQUIRE_ABSENT needs a complete effective resource namespace",
        ));
    }
    if namespace.inventories_sha256.is_empty() {
        return Err(ItemErrorV1::fatal(
            ITEM_NAMESPACE_INCOMPLETE,
            "namespace.inventoriesSha256",
            "at least one hashed namespace inventory is required",
        ));
    }
    for (index, hash) in namespace.inventories_sha256.iter().enumerate() {
        validate_sha256(hash, &format!("namespace.inventoriesSha256[{index}]"))?;
    }

    let mut existing = HashSet::with_capacity(namespace.resources.len());
    for (index, key) in namespace.resources.iter().enumerate() {
        validate_namespace_resref(&key.resref, &format!("namespace.resources[{index}].resref"))?;
        existing.insert((key.resref.to_ascii_lowercase(), key.resource_type));
    }
    let mut generated = HashSet::with_capacity(output.len());
    for (index, key) in output.iter().enumerate() {
        validate_resref(&key.resref, &format!("output[{index}].resref"))?;
        let folded = (key.resref.to_ascii_lowercase(), key.resource_type);
        if !generated.insert(folded.clone()) {
            return Err(ItemErrorV1::fatal(
                ITEM_RESOURCE_COLLISION,
                format!("output[{index}]"),
                "output contains a case-insensitive duplicate resource key",
            ));
        }
        if existing.contains(&folded) {
            return Err(ItemErrorV1::fatal(
                ITEM_RESOURCE_COLLISION,
                format!("output[{index}]"),
                format!(
                    "resource {} type {} already exists in the effective namespace",
                    key.resref, key.resource_type
                ),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartGeometryV1 {
    pub slot: ItemPartSlotV1,
    pub triangle_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemAssemblyReportV1 {
    pub schema_version: u32,
    pub profile: ItemCompositionProfileV1,
    pub part_count: usize,
    pub triangle_count: usize,
    pub triangle_budget: usize,
    pub warning_threshold: usize,
    pub warning: bool,
}

pub fn validate_item_assembly_v1(
    profile: ItemCompositionProfileV1,
    parts: &[ItemPartGeometryV1],
) -> Result<ItemAssemblyReportV1, ItemErrorV1> {
    let required = required_item_part_slots_v1(profile);
    if parts.len() != required.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "assembly.parts",
            format!("expected {} parts, got {}", required.len(), parts.len()),
        ));
    }
    let mut slots = HashSet::with_capacity(parts.len());
    let mut triangle_count = 0usize;
    for (index, part) in parts.iter().enumerate() {
        if !required.contains(&part.slot) || !slots.insert(part.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_RECIPE_INVALID,
                format!("assembly.parts[{index}].slot"),
                "assembly contains an unexpected or duplicate slot",
            ));
        }
        triangle_count = triangle_count
            .checked_add(part.triangle_count)
            .ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_TRIANGLE_COUNT_OVERFLOW,
                    "assembly.parts",
                    "whole-item triangle count overflow",
                )
            })?;
    }
    if triangle_count > AURORA_MODEL_TRIANGLE_BUDGET_V1 {
        return Err(ItemErrorV1::fatal(
            ITEM_TRIANGLE_BUDGET_EXCEEDED,
            "assembly.parts",
            format!(
                "item has {triangle_count} triangles; shared product budget is {AURORA_MODEL_TRIANGLE_BUDGET_V1}"
            ),
        ));
    }
    Ok(ItemAssemblyReportV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        profile,
        part_count: parts.len(),
        triangle_count,
        triangle_budget: AURORA_MODEL_TRIANGLE_BUDGET_V1,
        warning_threshold: AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
        warning: triangle_count > AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
    })
}

fn validate_namespace_resref(value: &str, path: &str) -> Result<(), ItemErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(ItemErrorV1::fatal(
            ITEM_RESREF_INVALID,
            path,
            "namespace ResRef must contain 1..16 ASCII alphanumeric/underscore characters",
        ));
    }
    Ok(())
}

pub(crate) fn validate_resref(value: &str, path: &str) -> Result<(), ItemErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ItemErrorV1::fatal(
            ITEM_RESREF_INVALID,
            path,
            "ResRef must contain 1..16 lowercase ASCII alphanumeric/underscore characters",
        ));
    }
    Ok(())
}

fn validate_item_class(value: &str, path: &str) -> Result<(), ItemErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(ItemErrorV1::fatal(
            ITEM_BASEITEMS_INVALID,
            path,
            "ItemClass must contain 1..16 ASCII alphanumeric/underscore characters",
        ));
    }
    Ok(())
}

pub(crate) fn validate_sha256(value: &str, path: &str) -> Result<(), ItemErrorV1> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            path,
            "SHA-256 must be 64 lowercase hexadecimal characters",
        ));
    }
    Ok(())
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
