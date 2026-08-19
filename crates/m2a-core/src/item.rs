//! Aurora-first item authoring contracts.
//!
//! Item assembly is selected by `UTI.BaseItem -> baseitems.2da.ModelType`.
//! It is deliberately independent from user-facing equipment categories:
//! the resolved model type determines one, three, or nineteen resource slots.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GFF_SCHEMA_VERSION, GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLimitsV1, GffLocStringV1,
        GffLocSubstringV1, GffStructV1, GffValueV1, GffWriterOptionsV1, read_gff_v32,
        write_gff_v32,
    },
    glb::{EmbeddedImageDecodeLimitsV1, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1},
    mdl::{
        InspectionReport, MdlAnimationSetV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlStateProjectionProfileV1, MdlWriterOptionsV1, NodeReport, inspect_binary_mdl,
        write_binary_mdl_with_animations_exact_face_planes_v1,
    },
    model_limits::{
        AURORA_MODEL_TRIANGLE_BUDGET_V1, AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
        validate_model_triangle_budget_v1,
    },
    model_pipeline::{
        resolve_base_color_image_index_v1, sanitize_meshy_h1_degenerate_triangles_exact_v1,
    },
    model_segmentation::segment_model_for_binary_mdl_v1,
    placeable::{static_placeable_glb_limits_v1, static_placeable_profile_a_options_v1},
    plt::{ItemPltLayerV1, PltWriterOptionsV1, write_item_plt_v1},
    profile_a::{convert_profile_a, derive_meshy_m0_static_rigid_profile_v1},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1, M0_RUNTIME_ENTRY_DIR_X,
        M0_RUNTIME_ENTRY_DIR_Y, M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z,
        M0RuntimeDirectionV1, M0RuntimePositionV1, binary_creature_multi_fixture_gic,
        binary_creature_multi_fixture_git, binary_m0_area_for, binary_m0_module_ifo_for,
        build_binary_creature_multi_fixture_module_v1, proof_factions,
    },
    tga::{
        TGA_SCHEMA_VERSION, TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, read_tga_image_v1,
        write_tga_v1,
    },
    two_da::{
        TwoDaAppendReportV1, TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellPatchV1,
        TwoDaCellValueV1, TwoDaInspectionV1, TwoDaLimitsV1, TwoDaRowPatchReportV1,
        TwoDaRowPatchRequestV1, append_two_da_row_v1, clone_two_da_row_request_v1,
        inspect_two_da_v2, patch_two_da_row_v1, read_two_da_row_v2,
    },
};

pub const ITEM_SCHEMA_VERSION_V1: u32 = 1;
pub const ITEM_WEAPON_COLOR_MIN_V1: u8 = 1;
pub const ITEM_WEAPON_COLOR_MAX_V1: u8 = 4;
pub const ITEM_WEAPON_MODEL_MAX_V1: u8 = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemWeaponPartAppearanceV1 {
    pub schema_version: u32,
    pub model: u8,
    pub color: u8,
    pub encoded_value: u8,
}

pub const ITEM_COLOR_FIELDS_V1: [&str; 6] = [
    "Leather1Color",
    "Leather2Color",
    "Cloth1Color",
    "Cloth2Color",
    "Metal1Color",
    "Metal2Color",
];

pub const ARMOR_PART_FIELDS_V1: [&str; 19] = [
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
];

pub const ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1: &str =
    "09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935";

pub const CAPART_REQUIRED_REFERENCE_TABLES_V1: [&str; 14] = [
    "CAPART",
    "PARTS_FOOT",
    "PARTS_SHIN",
    "PARTS_LEGS",
    "PARTS_PELVIS",
    "PARTS_CHEST",
    "PARTS_BELT",
    "PARTS_NECK",
    "PARTS_FOREARM",
    "PARTS_BICEP",
    "PARTS_SHOULDER",
    "PARTS_HAND",
    "PARTS_ROBE",
    "APPEARANCE",
];

const CAPART_PART_PROFILES_V1: [(&str, u32, &str, &str, &str); 19] = [
    ("ArmorPart_RFoot", 0, "FOOTR", "rfoot_g", "PARTS_FOOT"),
    ("ArmorPart_LFoot", 1, "FOOTL", "lfoot_g", "PARTS_FOOT"),
    ("ArmorPart_RShin", 2, "SHINR", "rshin_g", "PARTS_SHIN"),
    ("ArmorPart_LShin", 3, "SHINL", "lshin_g", "PARTS_SHIN"),
    ("ArmorPart_LThigh", 4, "LEGL", "lthigh_g", "PARTS_LEGS"),
    ("ArmorPart_RThigh", 5, "LEGR", "rthigh_g", "PARTS_LEGS"),
    ("ArmorPart_Pelvis", 6, "PELVIS", "pelvis_g", "PARTS_PELVIS"),
    ("ArmorPart_Torso", 7, "CHEST", "torso_g", "PARTS_CHEST"),
    ("ArmorPart_Belt", 8, "BELT", "belt_g", "PARTS_BELT"),
    ("ArmorPart_Neck", 9, "NECK", "neck_g", "PARTS_NECK"),
    (
        "ArmorPart_RFArm",
        10,
        "FORER",
        "rforearm_g",
        "PARTS_FOREARM",
    ),
    (
        "ArmorPart_LFArm",
        11,
        "FOREL",
        "lforearm_g",
        "PARTS_FOREARM",
    ),
    ("ArmorPart_RBicep", 12, "BICEPR", "rbicep_g", "PARTS_BICEP"),
    ("ArmorPart_LBicep", 13, "BICEPL", "lbicep_g", "PARTS_BICEP"),
    (
        "ArmorPart_RShoul",
        14,
        "SHOR",
        "rshoulder_g",
        "PARTS_SHOULDER",
    ),
    (
        "ArmorPart_LShoul",
        15,
        "SHOL",
        "lshoulder_g",
        "PARTS_SHOULDER",
    ),
    ("ArmorPart_RHand", 16, "HANDR", "rhand_g", "PARTS_HAND"),
    ("ArmorPart_LHand", 17, "HANDL", "lhand_g", "PARTS_HAND"),
    ("ArmorPart_Robe", 18, "ROBE", "root", "PARTS_ROBE"),
];

const CAPART_ROBE_HIDE_COLUMNS_V1: [(&str, &str); 19] = [
    ("HIDEFOOTR", "FOOTR"),
    ("HIDEFOOTL", "FOOTL"),
    ("HIDESHINR", "SHINR"),
    ("HIDESHINL", "SHINL"),
    ("HIDELEGR", "LEGR"),
    ("HIDELEGL", "LEGL"),
    ("HIDEPELVIS", "PELVIS"),
    ("HIDECHEST", "CHEST"),
    ("HIDEBELT", "BELT"),
    ("HIDENECK", "NECK"),
    ("HIDEFORER", "FORER"),
    ("HIDEFOREL", "FOREL"),
    ("HIDEBICEPR", "BICEPR"),
    ("HIDEBICEPL", "BICEPL"),
    ("HIDESHOR", "SHOR"),
    ("HIDESHOL", "SHOL"),
    ("HIDEHANDR", "HANDR"),
    ("HIDEHANDL", "HANDL"),
    ("HIDEHEAD", "HEAD"),
];

pub const ITEM_PROPERTY_CAST_SPELL_V1: u16 = 15;
pub const ITEM_PROPERTY_ON_HIT_CAST_SPELL_V1: u16 = 48;
pub const ITEM_BASEITEM_SPELL_SCROLL_DELETED_V1: u32 = 54;
pub const ITEM_BASEITEM_SPELL_SCROLL_V1: u32 = 75;
pub const ITEM_BASEITEM_CLOAK_V1: u32 = 80;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemCompositionProfileV1 {
    SinglePart,
    BottomMiddleTop,
    CapartArmor,
    CloakModel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemTextureProfileV1 {
    DirectColor,
    PaletteLayers,
    CapartPaletteLayers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemIconProfileV1 {
    Standard,
    Layered,
    IprpSpell,
    CapartComposite,
    CloakModel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemPartSourceKindV1 {
    MeshyGlb,
    CapartSelection,
    CloakModelSelection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemCapabilityV1 {
    pub schema_version: u32,
    pub composition_profile: ItemCompositionProfileV1,
    pub texture_profile: ItemTextureProfileV1,
    pub icon_profile: ItemIconProfileV1,
    /// Number of ordinary, independently built Meshy GLBs required by this
    /// composer. CAPART armor selects contextual body-part resources instead.
    pub meshy_source_count: u8,
    pub required_reference_tables: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartSlotV1 {
    pub index: u8,
    pub field: String,
    pub label: String,
    pub token: Option<String>,
    pub source_kind: ItemPartSourceKindV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_table: Option<String>,
    pub requires_explicit_resource_resrefs: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemBaseItemV1 {
    pub schema_version: u32,
    pub base_item: u32,
    pub label: String,
    pub item_class: String,
    pub model_type: u8,
    pub min_range: Option<u32>,
    pub max_range: Option<u32>,
    pub gender_specific: bool,
    pub default_model: Option<String>,
    pub default_icon: Option<String>,
    pub equipable_slots: u32,
    pub inv_slot_width: u32,
    pub inv_slot_height: u32,
    /// Exact runtime-routing columns carried by the selected baseitems.2da.
    /// Reduced fixtures and static item rows may omit them.
    pub weapon_wield: Option<u32>,
    pub weapon_type: Option<u32>,
    pub ranged_weapon: Option<u32>,
    pub ammunition_type: Option<u32>,
    pub capability: ItemCapabilityV1,
    pub part_slots: Vec<ItemPartSlotV1>,
    pub color_fields: Vec<String>,
}

/// The three native ammunition inventory channels used by ranged launchers.
/// Additional rows in ammunitiontypes.2da add visual/damage variants; they do
/// not create a fourth inventory channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemAmmunitionChannelV1 {
    Arrow,
    Bolt,
    Bullet,
}

impl ItemAmmunitionChannelV1 {
    fn ammo_base_item(self) -> u32 {
        match self {
            Self::Arrow => 20,
            Self::Bolt => 25,
            Self::Bullet => 27,
        }
    }

    fn ammunition_type(self) -> u32 {
        match self {
            Self::Arrow => 1,
            Self::Bolt => 2,
            Self::Bullet => 3,
        }
    }

    fn ammunitiontypes_offset(self) -> u32 {
        self.ammunition_type() - 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemWielderClipV1 {
    Bowshot,
    Xbowshot,
}

impl ItemWielderClipV1 {
    fn runtime_name(self) -> &'static str {
        match self {
            Self::Bowshot => "bowshot",
            Self::Xbowshot => "xbowshot",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemRangedWeaponProfileV1 {
    pub schema_version: u32,
    pub ammunition_channel: ItemAmmunitionChannelV1,
    /// Native projectile damage-visual selector. Values 0..5 belong to the
    /// retail table; authoring a new block starts at 6.
    pub damage_ranged_projectile: u8,
    pub projectile_model_resref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_sound_resref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_sound_resref: Option<String>,
    pub wielder_clip: ItemWielderClipV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRangedWeaponBindingV1 {
    pub schema_version: u32,
    pub weapon_base_item: u32,
    pub ammo_base_item: u32,
    pub ammunition_type: u32,
    pub weapon_wield: u32,
    pub weapon_type: u32,
    pub ranged_weapon: u32,
    pub damage_ranged_projectile: u8,
    pub ammunitiontypes_row: u32,
    pub projectile_model_resref: String,
    pub shot_sound_resref: Option<String>,
    pub impact_sound_resref: Option<String>,
    pub runtime_clip: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemAmmunitionVariantEntryV1 {
    pub label: String,
    pub model_resref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_sound_resref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_sound_resref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemAmmunitionVariantBlockRequestV1 {
    pub schema_version: u32,
    pub damage_ranged_projectile: u8,
    /// Native order: Arrow, Bolt, Bullet, Dart, Shuriken, Throwing Axe.
    pub entries: Vec<ItemAmmunitionVariantEntryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemAmmunitionVariantRowReportV1 {
    pub row: u32,
    pub label: String,
    pub model_resref: String,
    pub shot_sound_resref: Option<String>,
    pub impact_sound_resref: Option<String>,
    pub ammunition_type: u8,
    pub damage_ranged_projectile: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemAmmunitionVariantBlockReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub damage_ranged_projectile: u8,
    pub first_row: u32,
    pub last_row: u32,
    pub source_sha256: String,
    pub output_sha256: String,
    pub rows: Vec<ItemAmmunitionVariantRowReportV1>,
    pub append_reports: Vec<TwoDaAppendReportV1>,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemAmmunitionVariantBlockArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ItemAmmunitionVariantBlockReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemDamageRangedProjectileRequestV1 {
    pub schema_version: u32,
    pub damage_type_row: u32,
    pub expected_label: String,
    pub damage_ranged_projectile: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDamageRangedProjectileReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub damage_type_row: u32,
    pub expected_label: String,
    pub source_damage_ranged_projectile: u8,
    pub damage_ranged_projectile: u8,
    pub source_sha256: String,
    pub output_sha256: String,
    pub patch_report: TwoDaRowPatchReportV1,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemDamageRangedProjectileArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ItemDamageRangedProjectileReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemWeaponRuntimeRouteV1 {
    pub schema_version: u32,
    /// Audited retail BaseItem that owns these combat/animation semantics.
    pub base_item: u32,
    pub weapon_wield: u32,
    pub weapon_type: u32,
    pub ranged_weapon: u32,
    pub runtime_clip: String,
    pub animated_part_field: String,
    pub animated_part_label: String,
    pub reference_family: String,
}

/// Resolves only exact, audited retail routes. A custom output BaseItem never
/// becomes its own runtime donor merely because it copied the same numeric
/// columns; its lineage is recorded when the donor row is cloned.
pub fn resolve_item_weapon_runtime_route_v1(
    base_item: &ItemBaseItemV1,
) -> Result<Option<ItemWeaponRuntimeRouteV1>, ItemErrorV1> {
    let (Some(weapon_wield), Some(weapon_type), Some(ranged_weapon)) = (
        base_item.weapon_wield,
        base_item.weapon_type,
        base_item.ranged_weapon,
    ) else {
        return Ok(None);
    };
    let (runtime_clip, animated_part_field, animated_part_label, reference_family) = match (
        base_item.base_item,
        weapon_wield,
        weapon_type,
        ranged_weapon,
    ) {
        (6 | 7, 6, 1, 25) => ("xbowshot", "ModelPart3", "Top", "RETAIL_CROSSBOW"),
        (8 | 11, 5, 1, 20) => ("bowshot", "ModelPart2", "Middle", "RETAIL_BOW"),
        _ => return Ok(None),
    };
    Ok(Some(ItemWeaponRuntimeRouteV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        base_item: base_item.base_item,
        weapon_wield,
        weapon_type,
        ranged_weapon,
        runtime_clip: runtime_clip.to_owned(),
        animated_part_field: animated_part_field.to_owned(),
        animated_part_label: animated_part_label.to_owned(),
        reference_family: reference_family.to_owned(),
    }))
}

/// Resolves a custom ranged weapon from its authored semantic columns. Unlike
/// the retail route report above, this is not donor lineage: all values are
/// checked atomically against the selected standalone BaseItem.
pub fn resolve_item_ranged_weapon_profile_v1(
    base_item: &ItemBaseItemV1,
    profile: &ItemRangedWeaponProfileV1,
) -> Result<ItemRangedWeaponBindingV1, ItemErrorV1> {
    if profile.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-RANGED-WEAPON-PROFILE-SCHEMA-INVALID",
            "profile.schemaVersion",
            "ranged weapon profile schemaVersion must be 1",
        ));
    }
    if profile.damage_ranged_projectile < 6 {
        return Err(error(
            "ITEM-RANGED-WEAPON-PROJECTILE-VARIANT-RETAIL",
            "profile.damageRangedProjectile",
            "new projectile authoring must use a non-retail DamageRangedProjectile value in 6..255",
        ));
    }
    let projectile_model_resref = validate_resref(
        &profile.projectile_model_resref,
        "profile.projectileModelResref",
    )?;
    let shot_sound_resref = profile
        .shot_sound_resref
        .as_deref()
        .map(|value| validate_resref(value, "profile.shotSoundResref"))
        .transpose()?;
    let impact_sound_resref = profile
        .impact_sound_resref
        .as_deref()
        .map(|value| validate_resref(value, "profile.impactSoundResref"))
        .transpose()?;

    let expected_ammo_base_item = profile.ammunition_channel.ammo_base_item();
    let expected_ammunition_type = profile.ammunition_channel.ammunition_type();
    let expected_clip = match base_item.weapon_wield {
        Some(5) => ItemWielderClipV1::Bowshot,
        Some(6) => ItemWielderClipV1::Xbowshot,
        _ => {
            return Err(error(
                "ITEM-RANGED-WEAPON-WIELD-UNSUPPORTED",
                "baseitems.2da.WeaponWield",
                "V1 ranged weapons require the audited bow (5) or crossbow (6) wielder route",
            ));
        }
    };
    if base_item.weapon_type != Some(1)
        || base_item.ranged_weapon != Some(expected_ammo_base_item)
        || base_item.ammunition_type != Some(expected_ammunition_type)
        || profile.wielder_clip != expected_clip
    {
        return Err(error(
            "ITEM-RANGED-WEAPON-CHANNEL-MISMATCH",
            "profile.ammunitionChannel",
            format!(
                "{} requires WeaponType 1, RangedWeapon {}, AmmunitionType {} and clip {}",
                match profile.ammunition_channel {
                    ItemAmmunitionChannelV1::Arrow => "Arrow",
                    ItemAmmunitionChannelV1::Bolt => "Bolt",
                    ItemAmmunitionChannelV1::Bullet => "Bullet",
                },
                expected_ammo_base_item,
                expected_ammunition_type,
                expected_clip.runtime_name(),
            ),
        ));
    }

    let damage = u32::from(profile.damage_ranged_projectile);
    Ok(ItemRangedWeaponBindingV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        weapon_base_item: base_item.base_item,
        ammo_base_item: expected_ammo_base_item,
        ammunition_type: expected_ammunition_type,
        weapon_wield: base_item.weapon_wield.unwrap_or_default(),
        weapon_type: base_item.weapon_type.unwrap_or_default(),
        ranged_weapon: base_item.ranged_weapon.unwrap_or_default(),
        damage_ranged_projectile: profile.damage_ranged_projectile,
        ammunitiontypes_row: damage * 6 + profile.ammunition_channel.ammunitiontypes_offset(),
        projectile_model_resref,
        shot_sound_resref,
        impact_sound_resref,
        runtime_clip: expected_clip.runtime_name().to_owned(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemBaseItemsCatalogV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub physical_row_count: u32,
    pub inactive_row_count: u32,
    pub rows: Vec<ItemBaseItemV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemBaseItemModelRangeReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub base_item: u32,
    pub model: u8,
    pub model_bucket: u32,
    pub source_min_range: u32,
    pub source_max_range: u32,
    pub effective_max_range: u32,
    pub source_sha256: String,
    pub output_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_report: Option<TwoDaRowPatchReportV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemBaseItemModelRangeArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ItemBaseItemModelRangeReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemCustomWeaponBaseItemRequestV2 {
    pub schema_version: u32,
    pub donor_base_item: u32,
    /// Required physical append index and resulting printed BaseItem label.
    pub output_base_item: u32,
    pub label: String,
    pub item_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_strref: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inv_slot_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inv_slot_height: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemCustomWeaponBaseItemReportV2 {
    pub schema_version: u32,
    pub status: String,
    pub donor_base_item: u32,
    pub output_base_item: u32,
    pub label: String,
    pub item_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_strref: Option<u32>,
    pub inv_slot_width: u32,
    pub inv_slot_height: u32,
    pub source_sha256: String,
    pub output_sha256: String,
    pub runtime_route: ItemWeaponRuntimeRouteV1,
    pub append_report: TwoDaAppendReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemCustomWeaponBaseItemArtifactV2 {
    pub payload: Vec<u8>,
    pub selected: ItemBaseItemV1,
    pub report: ItemCustomWeaponBaseItemReportV2,
}

/// Exact standalone BaseItem definition. Every behavior-bearing cell is
/// supplied by the authoring contract; no existing BaseItem row is read as a
/// template or donor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemStandaloneWeaponBaseItemRequestV3 {
    pub schema_version: u32,
    /// Required physical append index and resulting printed BaseItem label.
    pub output_base_item: u32,
    pub label: String,
    pub item_class: String,
    pub cells: Vec<TwoDaCellAssignmentV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemStandaloneWeaponBaseItemReportV3 {
    pub schema_version: u32,
    pub status: String,
    pub output_base_item: u32,
    pub label: String,
    pub item_class: String,
    pub inv_slot_width: u32,
    pub inv_slot_height: u32,
    pub definition_source: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub append_report: TwoDaAppendReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemStandaloneWeaponBaseItemArtifactV3 {
    pub payload: Vec<u8>,
    pub selected: ItemBaseItemV1,
    pub report: ItemStandaloneWeaponBaseItemReportV3,
}

/// A deterministic, read-only source participating in Item resource
/// resolution. `order` is identity-bearing evidence, not an implicit guess at
/// Aurora's priority between multiple custom containers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemResourceProviderKindV1 {
    Vanilla,
    Hak,
    Module,
}

#[derive(Clone, Debug)]
pub struct ItemResourceProviderInputV1<'a> {
    pub provider_id: String,
    pub kind: ItemResourceProviderKindV1,
    pub order: u32,
    pub file_name: String,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct ItemResourceContextInputV1<'a> {
    pub schema_version: u32,
    pub resolution_policy: String,
    pub providers: Vec<ItemResourceProviderInputV1<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResourceProviderReportV1 {
    pub provider_id: String,
    pub kind: ItemResourceProviderKindV1,
    pub order: u32,
    pub file_name: String,
    pub container_file_type: String,
    pub byte_length: usize,
    pub sha256: String,
    pub resource_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResourceContextReportV1 {
    pub schema_version: u32,
    pub resolution_policy: String,
    pub status: String,
    pub providers: Vec<ItemResourceProviderReportV1>,
    pub context_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemEffectiveBaseItemsSelectionV1 {
    pub schema_version: u32,
    pub status: String,
    pub resource_context_sha256: String,
    pub baseitems_sha256: String,
    pub winning_provider_id: String,
    pub shadowed_provider_ids: Vec<String>,
    pub selected: ItemBaseItemV1,
    pub selection_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemResolvedContextResourceV1<'a> {
    pub schema_version: u32,
    pub resref: String,
    pub resource_type: u16,
    pub winning_provider_id: String,
    pub shadowed_provider_ids: Vec<String>,
    pub payload: &'a [u8],
    pub payload_sha256: String,
    pub context_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResolvedResourceV1 {
    pub schema_version: u32,
    pub base_item: u32,
    pub field: String,
    pub variant: u8,
    pub model_resref: String,
    pub icon_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartValueV1 {
    pub field: String,
    pub value: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPropertyV1 {
    pub property_name: u16,
    pub subtype: u16,
    pub cost_table: u8,
    pub cost_value: u16,
    pub param1: u8,
    pub param1_value: u8,
    pub chance_appear: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCastSpellIconV1 {
    pub schema_version: u32,
    pub base_item: u32,
    pub cast_spell_subtype: u16,
    pub icon_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCloakResolutionV1 {
    pub schema_version: u32,
    pub cloak_model_row: u16,
    pub model: u16,
    pub texture: u16,
    pub icon: u16,
    pub model_resref: String,
    pub texture_resref: String,
    pub icon_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCloakResolutionV2 {
    pub schema_version: u32,
    pub cloak_model_row: u16,
    pub model: u16,
    pub texture: u16,
    pub icon: u16,
    pub model_resref: String,
    pub texture_resref: String,
    pub icon_resref: String,
    pub resource_verification: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemReferenceResourceFormatV1 {
    BinaryMdl,
    AsciiMdl,
    PltV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResourceProvenanceV1 {
    pub schema_version: u32,
    pub source_kind: String,
    pub source_container_file_name: String,
    pub source_container_sha256: String,
    pub resource_locator: String,
    pub expected_sha256: String,
    pub manifest_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResourceInventoryEntryV2 {
    pub schema_version: u32,
    pub resource_type: u16,
    pub resref: String,
    pub byte_length: u64,
    pub sha256: String,
    pub format: ItemReferenceResourceFormatV1,
    pub triangle_count: usize,
    pub provenance: ItemResourceProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCloakResolutionV3 {
    pub schema_version: u32,
    pub cloak_model_row: u16,
    pub model: u16,
    pub texture: u16,
    pub icon: u16,
    pub model_resref: String,
    pub texture_resref: String,
    pub icon_resref: String,
    pub model_resource: ItemResourceInventoryEntryV2,
    pub texture_resource: ItemResourceInventoryEntryV2,
    pub icon_resource: ItemResourceInventoryEntryV2,
    pub inventory_sha256: String,
    pub resource_verification: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEquippedAppearanceBindingV1 {
    pub schema_version: u32,
    pub appearance_row: u16,
    pub model_type: String,
    pub race_token: String,
    pub racial_type: u8,
    pub gender: u8,
    pub phenotype: u8,
    pub model_prefix: String,
    pub appearance_table_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCapartPartResolutionV1 {
    pub schema_version: u32,
    pub field: String,
    pub selector: u8,
    pub capart_row: u32,
    pub mdl_name: String,
    pub node_name: String,
    pub parts_table: String,
    pub available_part_count: u32,
    pub robe_hidden_mdl_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemCapartContextV1 {
    pub schema_version: u32,
    pub model_prefix: String,
    pub gender_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCapartPartResolutionV2 {
    pub schema_version: u32,
    pub field: String,
    pub selector: u8,
    pub capart_row: u32,
    pub mdl_name: String,
    pub node_name: String,
    pub parts_table: String,
    pub available_part_count: u32,
    pub robe_hidden_mdl_names: Vec<String>,
    pub hidden_by_robe: bool,
    pub model_candidates: Vec<String>,
    pub resolved_model_resref: Option<String>,
    pub model_resource: Option<ItemResourceInventoryEntryV2>,
    pub palette_resource: Option<ItemResourceInventoryEntryV2>,
    pub context_sha256: String,
    pub resource_verification: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemReferenceTableInspectionV1 {
    pub schema_version: u32,
    pub table_name: String,
    pub source_sha256: String,
    pub source_byte_length: u64,
    pub normalized_byte_length: u64,
    pub stripped_trailing_blank_row_count: u32,
    pub inspection: TwoDaInspectionV1,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemColorValuesV1 {
    pub leather1_color: Option<u8>,
    pub leather2_color: Option<u8>,
    pub cloth1_color: Option<u8>,
    pub cloth2_color: Option<u8>,
    pub metal1_color: Option<u8>,
    pub metal2_color: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemBlueprintV1 {
    pub schema_version: u32,
    pub template_resref: String,
    pub tag: String,
    pub localized_name: String,
    pub description: String,
    pub identified_description: String,
    pub comment: String,
    pub parts: Vec<ItemPartValueV1>,
    #[serde(default)]
    pub properties: Vec<ItemPropertyV1>,
    pub colors: ItemColorValuesV1,
    pub cost: u32,
    pub add_cost: u32,
    pub charges: u8,
    pub stack_size: u16,
    pub palette_id: u8,
    pub identified: bool,
    pub stolen: bool,
    pub cursed: bool,
    pub plot: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemUtiReportV1 {
    pub schema_version: u32,
    pub template_resref: String,
    pub base_item: u32,
    pub model_type: u8,
    pub part_count: u32,
    pub color_field_count: u32,
    pub weapon_color_selector_count: u32,
    pub byte_length: u64,
    pub output_sha256: String,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemUtiArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ItemUtiReportV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartTransformV1 {
    pub translation: [f32; 3],
    pub rotation_xyzw: [f32; 4],
    pub uniform_scale: f32,
    pub pivot: [f32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemProjectileAxisV1 {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl ItemProjectileAxisV1 {
    fn vector(self) -> [f32; 3] {
        match self {
            Self::PositiveX => [1.0, 0.0, 0.0],
            Self::NegativeX => [-1.0, 0.0, 0.0],
            Self::PositiveY => [0.0, 1.0, 0.0],
            Self::NegativeY => [0.0, -1.0, 0.0],
            Self::PositiveZ => [0.0, 0.0, 1.0],
            Self::NegativeZ => [0.0, 0.0, -1.0],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemProjectileBuildOptionsV1 {
    pub schema_version: u32,
    pub source_forward_axis: ItemProjectileAxisV1,
    pub transform: ItemPartTransformV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemProjectileBuildReportV1 {
    pub schema_version: u32,
    pub profile: String,
    pub model_resref: String,
    pub texture_resref: String,
    pub source_forward_axis: ItemProjectileAxisV1,
    pub aurora_forward_axis: ItemProjectileAxisV1,
    pub transformed_forward: [f32; 3],
    pub orientation_status: String,
    pub source_sha256: String,
    pub triangle_count: usize,
    pub degenerate_triangle_count_removed: usize,
    pub stream_count: usize,
    pub transform: ItemPartTransformV1,
    pub mdl_sha256: String,
    pub texture_sha256: String,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemProjectileArtifactV1 {
    pub mdl_payload: Vec<u8>,
    pub texture_payload: Vec<u8>,
    pub report: ItemProjectileBuildReportV1,
    pub readback: InspectionReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemPartTextureEncodingV1 {
    DirectColor,
    PltMetal1,
    PltMetal2,
    PltCloth1,
    PltCloth2,
    PltLeather1,
    PltLeather2,
}

impl ItemPartTextureEncodingV1 {
    fn plt_layer(self) -> Option<ItemPltLayerV1> {
        match self {
            Self::DirectColor => None,
            Self::PltMetal1 => Some(ItemPltLayerV1::Metal1),
            Self::PltMetal2 => Some(ItemPltLayerV1::Metal2),
            Self::PltCloth1 => Some(ItemPltLayerV1::Cloth1),
            Self::PltCloth2 => Some(ItemPltLayerV1::Cloth2),
            Self::PltLeather1 => Some(ItemPltLayerV1::Leather1),
            Self::PltLeather2 => Some(ItemPltLayerV1::Leather2),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartBuildOptionsV1 {
    pub schema_version: u32,
    pub transform: ItemPartTransformV1,
    pub source_node: Option<String>,
    pub texture_encoding: ItemPartTextureEncodingV1,
    pub icon_size: Option<[u32; 2]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemIconProjectionBoundsV1 {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartBuildOptionsV2 {
    pub schema_version: u32,
    pub transform: ItemPartTransformV1,
    pub source_node: Option<String>,
    pub texture_encoding: ItemPartTextureEncodingV1,
    pub icon_size: Option<[u32; 2]>,
    pub icon_projection_bounds: Option<ItemIconProjectionBoundsV1>,
    /// Native ModelType 2 weapon color slot. Geometry stays byte-equivalent at
    /// the IR boundary while this slot deterministically authors a concrete
    /// direct-color texture resource.
    #[serde(default)]
    pub weapon_color: Option<u8>,
    /// Additional target-frame scale baked after the authored rotation. The
    /// reference fitter uses `[s, 1, s]` to fit transverse width/depth without
    /// shortening a weapon along Aurora's Y assembly axis.
    #[serde(default = "item_unit_scale_xyz_v1")]
    pub target_space_scale_xyz: [f32; 3],
}

fn item_unit_scale_xyz_v1() -> [f32; 3] {
    [1.0; 3]
}

fn item_is_unit_scale_xyz_v1(value: &[f32; 3]) -> bool {
    *value == item_unit_scale_xyz_v1()
}

impl Default for ItemPartBuildOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            transform: ItemPartTransformV1::default(),
            source_node: None,
            texture_encoding: ItemPartTextureEncodingV1::DirectColor,
            icon_size: None,
        }
    }
}

impl Default for ItemPartBuildOptionsV2 {
    fn default() -> Self {
        Self {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            transform: ItemPartTransformV1::default(),
            source_node: None,
            texture_encoding: ItemPartTextureEncodingV1::DirectColor,
            icon_size: None,
            icon_projection_bounds: None,
            weapon_color: None,
            target_space_scale_xyz: item_unit_scale_xyz_v1(),
        }
    }
}

impl Default for ItemPartTransformV1 {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
            uniform_scale: 1.0,
            pivot: [0.0; 3],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPartBuildReportV1 {
    pub schema_version: u32,
    pub profile: String,
    pub model_resref: String,
    pub texture_resref: String,
    pub texture_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_color: Option<u8>,
    pub source_node: Option<String>,
    pub source_sha256: String,
    pub triangle_count: usize,
    pub degenerate_triangle_count_removed: usize,
    pub stream_count: usize,
    pub transform: ItemPartTransformV1,
    pub target_space_scale_xyz: [f32; 3],
    pub mdl_sha256: String,
    pub texture_sha256: String,
    pub icon_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_projection_bounds: Option<ItemIconProjectionBoundsV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_opaque_pixel_count: Option<u64>,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemPartArtifactV1 {
    pub mdl_payload: Vec<u8>,
    pub texture_payload: Vec<u8>,
    pub icon_payload: Option<Vec<u8>>,
    pub report: ItemPartBuildReportV1,
    pub readback: InspectionReport,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemIconLayerInputV3<'a> {
    pub field: &'a str,
    pub icon_resref: &'a str,
    pub payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemIconLayerSummaryV3 {
    pub field: String,
    pub icon_resref: String,
    pub opaque_pixel_count: u64,
    pub bounds_min: [u32; 2],
    pub bounds_max_exclusive: [u32; 2],
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemIconCompositeReportV3 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub layout_profile: String,
    pub width: u32,
    pub height: u32,
    pub layer_count: usize,
    pub layers: Vec<ItemIconLayerSummaryV3>,
    pub opaque_pixel_count: u64,
    pub bounds_min: [u32; 2],
    pub bounds_max_exclusive: [u32; 2],
    pub axial_fill_ratio: f32,
    pub occupied_fill_ratio: f32,
    pub part_order_status: String,
    pub composite_rgba_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemSeamMeasurementV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub first_field: String,
    pub second_field: String,
    pub first_source_sha256: String,
    pub second_source_sha256: String,
    pub first_transform_sha256: String,
    pub second_transform_sha256: String,
    pub first_triangle_count: usize,
    pub second_triangle_count: usize,
    pub tolerance: f32,
    pub status: String,
    pub gap: f32,
    pub overlap: bool,
    pub measurement_sha256: String,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemFitSourceV1<'a> {
    pub field: &'a str,
    pub model_resref: &'a str,
    pub source_glb: &'a [u8],
    pub source_node: Option<&'a str>,
}

/// One exact editor transform submitted for geometry validation. This route
/// recomputes readback from the authored Q/T/S and never runs auto-fit again.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemManualFitPartV2 {
    pub field: String,
    pub transform: ItemPartTransformV1,
    pub target_space_scale_xyz: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemFitPartV1 {
    pub field: String,
    pub source_sha256: String,
    pub source_node: Option<String>,
    pub triangle_count: usize,
    pub input_bounds_min: [f32; 3],
    pub input_bounds_max: [f32; 3],
    pub axial_source_axis: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axial_target_axis: Option<u8>,
    pub target_axial_length: f32,
    pub transform: ItemPartTransformV1,
    pub transform_sha256: String,
    pub output_bounds_min: [f32; 3],
    pub output_bounds_max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemFitReportV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub tolerance: f32,
    pub iterations: u32,
    pub parts: Vec<ItemFitPartV1>,
    pub adjacent_seams: Vec<ItemSeamMeasurementV1>,
    pub non_adjacent_measurements: Vec<ItemSeamMeasurementV1>,
    pub solution_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemConnectorAnchorV2 {
    pub kind: String,
    pub axial_axis: u8,
    pub position: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemFitPartV2 {
    pub field: String,
    pub source_sha256: String,
    pub source_node: Option<String>,
    pub triangle_count: usize,
    pub input_bounds_min: [f32; 3],
    pub input_bounds_max: [f32; 3],
    pub axial_source_axis: u8,
    pub axial_target_axis: u8,
    pub target_axial_length: f32,
    pub transform: ItemPartTransformV1,
    #[serde(default = "item_unit_scale_xyz_v1")]
    pub target_space_scale_xyz: [f32; 3],
    pub transform_sha256: String,
    pub output_bounds_min: [f32; 3],
    pub output_bounds_max: [f32; 3],
    pub bottom_connector: Option<ItemConnectorAnchorV2>,
    pub top_connector: Option<ItemConnectorAnchorV2>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemAdjacentConnectorV2 {
    pub first_field: String,
    pub first_connector: String,
    pub second_field: String,
    pub second_connector: String,
    pub axial_axis: u8,
    pub axial_overlap: f32,
    pub required_min_overlap: f32,
    pub required_max_overlap: f32,
    pub surface_status: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemFitReportV2 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub tolerance: f32,
    pub iterations: u32,
    pub parts: Vec<ItemFitPartV2>,
    pub adjacent_seams: Vec<ItemSeamMeasurementV1>,
    pub adjacent_connectors: Vec<ItemAdjacentConnectorV2>,
    pub non_adjacent_measurements: Vec<ItemSeamMeasurementV1>,
    pub solution_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemOrientationFrameV3 {
    pub source_axial_axis: u8,
    pub source_width_axis: u8,
    pub source_depth_axis: u8,
    pub target_axial_axis: u8,
    pub target_width_axis: u8,
    pub target_depth_axis: u8,
    pub width_to_depth_ratio: f32,
    pub handedness_determinant: f32,
    pub evidence: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemFitReportV3 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub tolerance: f32,
    pub iterations: u32,
    pub orientation_frame: ItemOrientationFrameV3,
    pub parts: Vec<ItemFitPartV2>,
    pub adjacent_seams: Vec<ItemSeamMeasurementV1>,
    pub adjacent_connectors: Vec<ItemAdjacentConnectorV2>,
    pub non_adjacent_measurements: Vec<ItemSeamMeasurementV1>,
    pub solution_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemAttachmentRouteV1 {
    None,
    Hand,
    Capart,
    Cloak,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemReferenceProfileIdentityV1 {
    pub schema_version: u32,
    pub resource_context_sha256: String,
    pub baseitems_sha256: String,
    pub base_item: u32,
    pub item_class: String,
    pub model_type: u8,
    pub reference_kind: String,
    pub reference_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemReferenceSlotFrameV1 {
    pub field: String,
    pub label: String,
    pub token: String,
    pub model_resref: String,
    pub model_sha256: String,
    pub controller_node_name: String,
    pub controller_translation: [f32; 3],
    pub controller_rotation_xyzw: [f32; 4],
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub allow_axial_extension_at_min: bool,
    pub allow_axial_extension_at_max: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemAttachmentProfileV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub identity: ItemReferenceProfileIdentityV1,
    pub attachment_route: ItemAttachmentRouteV1,
    pub equipable_slots: u32,
    pub common_origin: [f32; 3],
    pub axial_axis: u8,
    pub width_axis: u8,
    pub depth_axis: u8,
    pub attachment_zone_min: [f32; 3],
    pub attachment_zone_max: [f32; 3],
    pub attachment_evidence: String,
    pub slots: Vec<ItemReferenceSlotFrameV1>,
    pub profile_sha256: String,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemReferenceMdlInputV1<'a> {
    pub field: &'a str,
    pub model_resref: &'a str,
    pub mdl_payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemFitReportV4 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub tolerance: f32,
    pub iterations: u32,
    pub reference_profile_sha256: String,
    pub common_origin: [f32; 3],
    pub orientation_frame: ItemOrientationFrameV3,
    pub parts: Vec<ItemFitPartV2>,
    pub adjacent_seams: Vec<ItemSeamMeasurementV1>,
    pub adjacent_connectors: Vec<ItemAdjacentConnectorV2>,
    pub non_adjacent_measurements: Vec<ItemSeamMeasurementV1>,
    pub solution_sha256: String,
}

// V2/V3 reports predate target-space transverse scaling. Their public JSON now
// exposes the explicit unit default, while their frozen solution hashes retain
// the original canonical payload. V4 hashes the complete scale-aware report.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemFitPartV2LegacyHashView<'a> {
    field: &'a str,
    source_sha256: &'a str,
    source_node: &'a Option<String>,
    triangle_count: usize,
    input_bounds_min: [f32; 3],
    input_bounds_max: [f32; 3],
    axial_source_axis: u8,
    axial_target_axis: u8,
    target_axial_length: f32,
    transform: ItemPartTransformV1,
    transform_sha256: &'a str,
    output_bounds_min: [f32; 3],
    output_bounds_max: [f32; 3],
    bottom_connector: &'a Option<ItemConnectorAnchorV2>,
    top_connector: &'a Option<ItemConnectorAnchorV2>,
}

impl<'a> From<&'a ItemFitPartV2> for ItemFitPartV2LegacyHashView<'a> {
    fn from(part: &'a ItemFitPartV2) -> Self {
        Self {
            field: &part.field,
            source_sha256: &part.source_sha256,
            source_node: &part.source_node,
            triangle_count: part.triangle_count,
            input_bounds_min: part.input_bounds_min,
            input_bounds_max: part.input_bounds_max,
            axial_source_axis: part.axial_source_axis,
            axial_target_axis: part.axial_target_axis,
            target_axial_length: part.target_axial_length,
            transform: part.transform,
            transform_sha256: &part.transform_sha256,
            output_bounds_min: part.output_bounds_min,
            output_bounds_max: part.output_bounds_max,
            bottom_connector: &part.bottom_connector,
            top_connector: &part.top_connector,
        }
    }
}

fn item_fit_report_v2_legacy_hash_bytes_v1(
    report: &ItemFitReportV2,
) -> Result<Vec<u8>, ItemErrorV1> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Report<'a> {
        schema_version: u32,
        algorithm: &'a str,
        status: &'a str,
        tolerance: f32,
        iterations: u32,
        parts: Vec<ItemFitPartV2LegacyHashView<'a>>,
        adjacent_seams: &'a [ItemSeamMeasurementV1],
        adjacent_connectors: &'a [ItemAdjacentConnectorV2],
        non_adjacent_measurements: &'a [ItemSeamMeasurementV1],
        solution_sha256: &'a str,
    }
    serde_json::to_vec(&Report {
        schema_version: report.schema_version,
        algorithm: &report.algorithm,
        status: &report.status,
        tolerance: report.tolerance,
        iterations: report.iterations,
        parts: report.parts.iter().map(Into::into).collect(),
        adjacent_seams: &report.adjacent_seams,
        adjacent_connectors: &report.adjacent_connectors,
        non_adjacent_measurements: &report.non_adjacent_measurements,
        solution_sha256: &report.solution_sha256,
    })
    .map_err(|_| {
        error(
            "ITEM-FIT-LEGACY-HASH-SERIALIZE-FAILED",
            "report",
            "legacy Item fit hash payload could not be serialized",
        )
    })
}

fn item_fit_report_v3_legacy_hash_bytes_v1(
    report: &ItemFitReportV3,
) -> Result<Vec<u8>, ItemErrorV1> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Report<'a> {
        schema_version: u32,
        algorithm: &'a str,
        status: &'a str,
        tolerance: f32,
        iterations: u32,
        orientation_frame: &'a ItemOrientationFrameV3,
        parts: Vec<ItemFitPartV2LegacyHashView<'a>>,
        adjacent_seams: &'a [ItemSeamMeasurementV1],
        adjacent_connectors: &'a [ItemAdjacentConnectorV2],
        non_adjacent_measurements: &'a [ItemSeamMeasurementV1],
        solution_sha256: &'a str,
    }
    serde_json::to_vec(&Report {
        schema_version: report.schema_version,
        algorithm: &report.algorithm,
        status: &report.status,
        tolerance: report.tolerance,
        iterations: report.iterations,
        orientation_frame: &report.orientation_frame,
        parts: report.parts.iter().map(Into::into).collect(),
        adjacent_seams: &report.adjacent_seams,
        adjacent_connectors: &report.adjacent_connectors,
        non_adjacent_measurements: &report.non_adjacent_measurements,
        solution_sha256: &report.solution_sha256,
    })
    .map_err(|_| {
        error(
            "ITEM-FIT-LEGACY-HASH-SERIALIZE-FAILED",
            "report",
            "legacy full-frame Item fit hash payload could not be serialized",
        )
    })
}

#[derive(Clone, Copy, Debug)]
pub struct ItemComposerMdlInputV2<'a> {
    pub field: &'a str,
    pub model_resref: &'a str,
    pub mdl_payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemComposerPartConformanceV2 {
    pub field: String,
    pub model_resref: String,
    pub root_node_name: String,
    pub root_controller_owner: String,
    pub transform_controller_owner: String,
    pub mesh_node_names: Vec<String>,
    pub triangle_count: usize,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemComposerConformanceReportV2 {
    pub schema_version: u32,
    pub algorithm: String,
    pub status: String,
    pub append_order: Vec<String>,
    pub parts: Vec<ItemComposerPartConformanceV2>,
    pub composite_bounds_min: [f32; 3],
    pub composite_bounds_max: [f32; 3],
    pub total_triangle_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemProofModuleIdentityV1 {
    pub schema_version: u32,
    pub module_resref: String,
    pub area_resref: String,
    pub hak_resref: String,
    pub blueprint_resref: String,
    pub module_name: String,
    pub area_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemRangedWeaponProofIdentityV1 {
    pub schema_version: u32,
    pub module_resref: String,
    pub area_resref: String,
    pub hak_resref: String,
    pub weapon_blueprint_resref: String,
    pub ammunition_blueprint_resref: String,
    pub module_name: String,
    pub area_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemProofPlacementV1 {
    pub schema_version: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation_x: f32,
    pub orientation_y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemEquippedProofProfileV2 {
    CapartArmor,
    Cloak,
    ModelType2Parts,
}

impl ItemEquippedProofProfileV2 {
    fn equipment_slot(self, selected_slot: u32) -> Result<u32, ItemErrorV1> {
        let expected = match self {
            Self::CapartArmor => Some(2),
            Self::Cloak => Some(8_192),
            Self::ModelType2Parts => None,
        };
        if let Some(expected) = expected {
            if selected_slot != expected {
                return Err(error(
                    "ITEM-EQUIPPED-PROOF-SLOT-MISMATCH",
                    "identity.equipmentSlot",
                    format!(
                        "{} requires native Equip_ItemList slot {expected}, got {selected_slot}",
                        self.report_name()
                    ),
                ));
            }
        } else if !matches!(selected_slot, 0x10 | 0x20) {
            return Err(error(
                "ITEM-EQUIPPED-PROOF-SLOT-INVALID",
                "identity.equipmentSlot",
                "ModelType 2 proof requires the data-selected native right-hand (16) or left-hand (32) Equip_ItemList slot",
            ));
        }
        Ok(selected_slot)
    }

    fn report_name(self) -> &'static str {
        match self {
            Self::CapartArmor => "EQUIPPED_CAPART_ARMOR_V2",
            Self::Cloak => "EQUIPPED_CLOAK_V2",
            Self::ModelType2Parts => "EQUIPPED_MODELTYPE2_PARTS_V2",
        }
    }
}

/// Selects a deterministic humanoid hand slot from the exact BaseItem
/// `EquipableSlots` mask. This is deliberately data-driven: no weapon/shield
/// category is inferred. Right hand is preferred when the BaseItem permits
/// both hands, which matches the proof wearer's intended presentation.
pub fn resolve_item_modeltype2_equipment_slot_v1(equipable_slots: u32) -> Result<u32, ItemErrorV1> {
    for slot in [0x10, 0x20] {
        if equipable_slots & slot != 0 {
            return Ok(slot);
        }
    }
    Err(error(
        "ITEM-MODELTYPE2-PROOF-SLOT-UNSUPPORTED",
        "baseitems.2da.EquipableSlots",
        format!(
            "ModelType 2 equipped proof requires a humanoid right-hand (16) or left-hand (32) bit; mask is {equipable_slots}"
        ),
    ))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemEquippedProofIdentityV2 {
    pub schema_version: u32,
    pub module_resref: String,
    pub area_resref: String,
    pub hak_resref: String,
    pub blueprint_resref: String,
    pub module_name: String,
    pub area_name: String,
    pub creature_resref: String,
    pub creature_display_name: String,
    pub appearance_row: u16,
    pub race: u8,
    pub gender: u8,
    pub phenotype: i32,
    pub model_prefix: String,
    pub appearance_table_sha256: String,
    pub fixture_profile: ItemEquippedProofProfileV2,
    pub equipment_slot: u32,
}

impl Default for ItemProofPlacementV1 {
    fn default() -> Self {
        Self {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            x: M0_RUNTIME_ENTRY_X,
            y: M0_RUNTIME_ENTRY_Y + 4.5,
            z: M0_RUNTIME_ENTRY_Z,
            orientation_x: 0.0,
            orientation_y: -1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemProofModuleReportV1 {
    pub schema_version: u32,
    pub fixture_profile: String,
    pub module_resref: String,
    pub module_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_resref: String,
    pub blueprint_resref: String,
    pub ground_item_count: u32,
    pub creature_count: u32,
    pub item_position: [f32; 3],
    pub entry_position: [f32; 3],
    pub entry_direction: [f32; 2],
    pub resource_count: u32,
    pub byte_length: u64,
    pub output_sha256: String,
    pub semantic_readback_status: String,
    pub model_visibility: String,
    pub proof_completeness: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRangedWeaponProofModuleReportV1 {
    pub schema_version: u32,
    pub fixture_profile: String,
    pub module_resref: String,
    pub module_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_resref: String,
    pub weapon_blueprint_resref: String,
    pub ammunition_blueprint_resref: String,
    pub ground_item_count: u32,
    pub creature_count: u32,
    pub weapon_position: [f32; 3],
    pub ammunition_position: [f32; 3],
    pub target_blueprint_resref: String,
    pub target_appearance_row: u16,
    pub target_position: [f32; 3],
    pub target_walk_rate: i32,
    pub target_scripts_empty: bool,
    pub entry_position: [f32; 3],
    pub entry_direction: [f32; 2],
    pub resource_count: u32,
    pub byte_length: u64,
    pub output_sha256: String,
    pub semantic_readback_status: String,
    pub model_visibility: String,
    pub proof_completeness: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEquippedProofModuleReportV2 {
    pub schema_version: u32,
    pub module_resref: String,
    pub module_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_resref: String,
    pub blueprint_resref: String,
    pub creature_resref: String,
    pub appearance_row: u16,
    pub race: u8,
    pub gender: u8,
    pub phenotype: i32,
    pub model_prefix: String,
    pub appearance_table_sha256: String,
    pub fixture_profile: String,
    pub equipment_slot: u32,
    pub creature_position: [f32; 3],
    pub entry_position: [f32; 3],
    pub entry_direction: [f32; 2],
    pub resource_count: u32,
    pub byte_length: u64,
    pub output_sha256: String,
    pub semantic_readback_status: String,
    pub model_visibility: String,
    pub proof_completeness: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemAndEquippedProofModuleReportV3 {
    pub schema_version: u32,
    pub module_resref: String,
    pub module_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_resref: String,
    pub blueprint_resref: String,
    pub creature_resref: String,
    pub fixture_profile: String,
    pub equipment_slot: u32,
    pub item_position: [f32; 3],
    pub creature_position: [f32; 3],
    pub ground_item_count: u32,
    pub equipped_item_count: u32,
    pub resource_count: u32,
    pub byte_length: u64,
    pub output_sha256: String,
    pub semantic_readback_status: String,
    pub model_visibility: String,
    pub proof_completeness: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemProofModuleArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ItemProofModuleReportV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemRangedWeaponProofModuleArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ItemRangedWeaponProofModuleReportV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemEquippedProofModuleArtifactV2 {
    pub payload: Vec<u8>,
    pub report: ItemEquippedProofModuleReportV2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemAndEquippedProofModuleArtifactV3 {
    pub payload: Vec<u8>,
    pub report: ItemAndEquippedProofModuleReportV3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemTriangleBudgetReportV1 {
    pub schema_version: u32,
    pub triangle_count: usize,
    pub triangle_budget: usize,
    pub warning_above: usize,
    pub warning: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
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

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> ItemErrorV1 {
    ItemErrorV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn required_column(columns: &[String], name: &str) -> Result<usize, ItemErrorV1> {
    let matches = columns
        .iter()
        .enumerate()
        .filter(|(_, value)| value.eq_ignore_ascii_case(name))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        [] => Err(error(
            "ITEM-BASEITEMS-COLUMN-MISSING",
            format!("baseitems.2da.columns.{name}"),
            format!("required baseitems.2da column {name} is missing"),
        )),
        _ => Err(error(
            "ITEM-BASEITEMS-COLUMN-DUPLICATE",
            format!("baseitems.2da.columns.{name}"),
            format!("baseitems.2da column {name} is ambiguous"),
        )),
    }
}

fn optional_column(columns: &[String], name: &str) -> Result<Option<usize>, ItemErrorV1> {
    let matches = columns
        .iter()
        .enumerate()
        .filter(|(_, value)| value.eq_ignore_ascii_case(name))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(Some(*index)),
        [] => Ok(None),
        _ => Err(error(
            "ITEM-BASEITEMS-COLUMN-DUPLICATE",
            format!("baseitems.2da.columns.{name}"),
            format!("baseitems.2da column {name} is ambiguous"),
        )),
    }
}

fn cell_text<'a>(
    cells: &'a [TwoDaCellValueV1],
    index: usize,
    row: u32,
    name: &str,
) -> Result<&'a str, ItemErrorV1> {
    match cells.get(index) {
        Some(TwoDaCellValueV1::Text { value }) if !value.trim().is_empty() => Ok(value.trim()),
        _ => Err(error(
            "ITEM-BASEITEMS-VALUE-MISSING",
            format!("baseitems.2da.rows.{row}.{name}"),
            format!("selected BaseItem row has no {name} value"),
        )),
    }
}

fn parse_u32(value: &str, row: u32, name: &str) -> Result<u32, ItemErrorV1> {
    let parsed = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map(|digits| u32::from_str_radix(digits, 16))
        .unwrap_or_else(|| value.parse::<u32>());
    parsed.map_err(|_| {
        error(
            "ITEM-BASEITEMS-NUMBER-INVALID",
            format!("baseitems.2da.rows.{row}.{name}"),
            format!("{name} must be an unsigned integer"),
        )
    })
}

fn parse_optional_text(
    cells: &[TwoDaCellValueV1],
    index: usize,
) -> Result<Option<String>, ItemErrorV1> {
    match cells.get(index) {
        Some(TwoDaCellValueV1::Null) | None => Ok(None),
        Some(TwoDaCellValueV1::Text { value }) if value.trim().is_empty() => Ok(None),
        Some(TwoDaCellValueV1::Text { value }) => {
            let value = value.trim();
            if value == "*" || value == "****" {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
    }
}

fn parse_optional_u32(
    cells: &[TwoDaCellValueV1],
    index: Option<usize>,
    row: u32,
    name: &str,
) -> Result<Option<u32>, ItemErrorV1> {
    let Some(index) = index else {
        return Ok(None);
    };
    let Some(value) = parse_optional_text(cells, index)? else {
        return Ok(None);
    };
    parse_u32(&value, row, name).map(Some)
}

fn inactive_row(
    cells: &[TwoDaCellValueV1],
    item_class_index: usize,
    model_type_index: usize,
) -> bool {
    let active_text = |index: usize| match cells.get(index) {
        Some(TwoDaCellValueV1::Text { value }) => {
            let value = value.trim();
            !value.is_empty() && value != "*" && value != "****"
        }
        Some(TwoDaCellValueV1::Null) | None => false,
    };
    !active_text(item_class_index) || !active_text(model_type_index)
}

fn part_slots(base_item: u32, model_type: u8, explicit: bool) -> Vec<ItemPartSlotV1> {
    let single_source_kind = if base_item == ITEM_BASEITEM_CLOAK_V1 {
        ItemPartSourceKindV1::CloakModelSelection
    } else {
        ItemPartSourceKindV1::MeshyGlb
    };
    match model_type {
        0 | 1 => vec![ItemPartSlotV1 {
            index: 0,
            field: "ModelPart1".to_owned(),
            label: "Model".to_owned(),
            token: None,
            source_kind: single_source_kind,
            reference_table: (base_item == ITEM_BASEITEM_CLOAK_V1).then(|| "CloakModel".to_owned()),
            requires_explicit_resource_resrefs: explicit,
        }],
        2 => ["Bottom", "Middle", "Top"]
            .into_iter()
            .zip(["b", "m", "t"])
            .enumerate()
            .map(|(index, (label, token))| ItemPartSlotV1 {
                index: index as u8,
                field: format!("ModelPart{}", index + 1),
                label: label.to_owned(),
                token: Some(token.to_owned()),
                source_kind: ItemPartSourceKindV1::MeshyGlb,
                reference_table: None,
                requires_explicit_resource_resrefs: explicit,
            })
            .collect(),
        3 => ARMOR_PART_FIELDS_V1
            .into_iter()
            .enumerate()
            .map(|(index, field)| ItemPartSlotV1 {
                index: index as u8,
                field: field.to_owned(),
                label: field.trim_start_matches("ArmorPart_").to_owned(),
                token: None,
                source_kind: ItemPartSourceKindV1::CapartSelection,
                reference_table: Some(CAPART_PART_PROFILES_V1[index].4.to_owned()),
                requires_explicit_resource_resrefs: false,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn item_capability_v1(
    base_item: u32,
    model_type: u8,
    part_count: usize,
    equipable_slots: u32,
) -> ItemCapabilityV1 {
    let (composition_profile, texture_profile, icon_profile, meshy_source_count, tables) =
        if base_item == ITEM_BASEITEM_CLOAK_V1 {
            (
                ItemCompositionProfileV1::CloakModel,
                ItemTextureProfileV1::PaletteLayers,
                ItemIconProfileV1::CloakModel,
                0,
                vec!["CloakModel".to_owned(), "Appearance".to_owned()],
            )
        } else if model_type == 3 {
            (
                ItemCompositionProfileV1::CapartArmor,
                ItemTextureProfileV1::CapartPaletteLayers,
                ItemIconProfileV1::CapartComposite,
                0,
                CAPART_REQUIRED_REFERENCE_TABLES_V1
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )
        } else {
            let composition = if model_type == 2 {
                ItemCompositionProfileV1::BottomMiddleTop
            } else {
                ItemCompositionProfileV1::SinglePart
            };
            let texture = if model_type == 1 {
                ItemTextureProfileV1::PaletteLayers
            } else {
                ItemTextureProfileV1::DirectColor
            };
            let icon = if matches!(
                base_item,
                ITEM_BASEITEM_SPELL_SCROLL_DELETED_V1 | ITEM_BASEITEM_SPELL_SCROLL_V1
            ) {
                ItemIconProfileV1::IprpSpell
            } else if model_type == 1 {
                ItemIconProfileV1::Layered
            } else {
                ItemIconProfileV1::Standard
            };
            let tables = if icon == ItemIconProfileV1::IprpSpell {
                vec!["IPRP_SPELLS".to_owned()]
            } else if model_type == 2 && equipable_slots != 0 {
                vec!["Appearance".to_owned()]
            } else {
                Vec::new()
            };
            (composition, texture, icon, part_count as u8, tables)
        };
    ItemCapabilityV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        composition_profile,
        texture_profile,
        icon_profile,
        meshy_source_count,
        required_reference_tables: tables,
    }
}

pub fn resolve_item_capability_v1(base_item: &ItemBaseItemV1) -> ItemCapabilityV1 {
    item_capability_v1(
        base_item.base_item,
        base_item.model_type,
        base_item.part_slots.len(),
        base_item.equipable_slots,
    )
}

fn parse_row(
    columns: &[String],
    cells: &[TwoDaCellValueV1],
    base_item: u32,
) -> Result<ItemBaseItemV1, ItemErrorV1> {
    let label_index = required_column(columns, "Label")?;
    let item_class_index = required_column(columns, "ItemClass")?;
    let model_type_index = required_column(columns, "ModelType")?;
    let min_range_index = optional_column(columns, "MinRange")?;
    let max_range_index = optional_column(columns, "MaxRange")?;
    let gender_index = required_column(columns, "GenderSpecific")?;
    let default_model_index = required_column(columns, "DefaultModel")?;
    let default_icon_index = required_column(columns, "DefaultIcon")?;
    let equipable_slots_index = required_column(columns, "EquipableSlots")?;
    let width_index = required_column(columns, "InvSlotWidth")?;
    let height_index = required_column(columns, "InvSlotHeight")?;
    let weapon_wield_index = optional_column(columns, "WeaponWield")?;
    let weapon_type_index = optional_column(columns, "WeaponType")?;
    let ranged_weapon_index = optional_column(columns, "RangedWeapon")?;
    let ammunition_type_index = optional_column(columns, "AmmunitionType")?;

    let label = cell_text(cells, label_index, base_item, "Label")?.to_owned();
    let item_class = cell_text(cells, item_class_index, base_item, "ItemClass")?.to_owned();
    let model_type_value = parse_u32(
        cell_text(cells, model_type_index, base_item, "ModelType")?,
        base_item,
        "ModelType",
    )?;
    let model_type = u8::try_from(model_type_value).map_err(|_| {
        error(
            "ITEM-MODEL-TYPE-UNSUPPORTED",
            format!("baseitems.2da.rows.{base_item}.ModelType"),
            "ModelType must be one of 0, 1, 2, or 3",
        )
    })?;
    if model_type > 3 {
        return Err(error(
            "ITEM-MODEL-TYPE-UNSUPPORTED",
            format!("baseitems.2da.rows.{base_item}.ModelType"),
            "ModelType must be one of 0, 1, 2, or 3",
        ));
    }
    let gender_specific = match parse_u32(
        cell_text(cells, gender_index, base_item, "GenderSpecific")?,
        base_item,
        "GenderSpecific",
    )? {
        0 => false,
        1 => true,
        _ => {
            return Err(error(
                "ITEM-BASEITEMS-BOOLEAN-INVALID",
                format!("baseitems.2da.rows.{base_item}.GenderSpecific"),
                "GenderSpecific must be 0 or 1",
            ));
        }
    };
    let explicit = gender_specific || model_type == 3 || item_class.eq_ignore_ascii_case("cloak");
    let equipable_slots = parse_u32(
        cell_text(cells, equipable_slots_index, base_item, "EquipableSlots")?,
        base_item,
        "EquipableSlots",
    )?;

    let slots = part_slots(base_item, model_type, explicit);
    Ok(ItemBaseItemV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        base_item,
        label,
        item_class,
        model_type,
        min_range: parse_optional_u32(cells, min_range_index, base_item, "MinRange")?,
        max_range: parse_optional_u32(cells, max_range_index, base_item, "MaxRange")?,
        gender_specific,
        default_model: parse_optional_text(cells, default_model_index)?,
        default_icon: parse_optional_text(cells, default_icon_index)?,
        equipable_slots,
        inv_slot_width: parse_u32(
            cell_text(cells, width_index, base_item, "InvSlotWidth")?,
            base_item,
            "InvSlotWidth",
        )?,
        inv_slot_height: parse_u32(
            cell_text(cells, height_index, base_item, "InvSlotHeight")?,
            base_item,
            "InvSlotHeight",
        )?,
        weapon_wield: parse_optional_u32(cells, weapon_wield_index, base_item, "WeaponWield")?,
        weapon_type: parse_optional_u32(cells, weapon_type_index, base_item, "WeaponType")?,
        ranged_weapon: parse_optional_u32(cells, ranged_weapon_index, base_item, "RangedWeapon")?,
        ammunition_type: parse_optional_u32(
            cells,
            ammunition_type_index,
            base_item,
            "AmmunitionType",
        )?,
        capability: item_capability_v1(base_item, model_type, slots.len(), equipable_slots),
        part_slots: slots,
        color_fields: if matches!(model_type, 1 | 3) {
            ITEM_COLOR_FIELDS_V1
                .into_iter()
                .map(str::to_owned)
                .collect()
        } else {
            Vec::new()
        },
    })
}

pub fn inspect_item_baseitems_v1(bytes: &[u8]) -> Result<ItemBaseItemsCatalogV1, ItemErrorV1> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits).map_err(|source| {
        error(
            "ITEM-BASEITEMS-INVALID",
            source.path,
            format!("baseitems.2da inspection failed: {}", source.message),
        )
    })?;
    let item_class_index = required_column(&inspection.columns, "ItemClass")?;
    let model_type_index = required_column(&inspection.columns, "ModelType")?;
    let mut rows = Vec::with_capacity(inspection.physical_row_count as usize);
    let mut labels = BTreeSet::new();
    let mut inactive_row_count = 0_u32;
    for physical_row in 0..inspection.physical_row_count {
        let readback = read_two_da_row_v2(bytes, physical_row, &limits).map_err(|source| {
            error(
                "ITEM-BASEITEMS-INVALID",
                source.path,
                format!("baseitems.2da row readback failed: {}", source.message),
            )
        })?;
        if !labels.insert(readback.printed_row_label) {
            return Err(error(
                "ITEM-BASEITEMS-ROW-DUPLICATE",
                format!("baseitems.2da.rows.{}", readback.printed_row_label),
                "BaseItem printed row labels must be unique",
            ));
        }
        if inactive_row(&readback.cells, item_class_index, model_type_index) {
            inactive_row_count += 1;
            continue;
        }
        rows.push(parse_row(
            &inspection.columns,
            &readback.cells,
            readback.printed_row_label,
        )?);
    }
    Ok(ItemBaseItemsCatalogV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        source_sha256: inspection.source_sha256,
        physical_row_count: inspection.physical_row_count,
        inactive_row_count,
        rows,
    })
}

pub fn resolve_item_baseitem_v1(
    bytes: &[u8],
    base_item: u32,
) -> Result<ItemBaseItemV1, ItemErrorV1> {
    let catalog = inspect_item_baseitems_v1(bytes)?;
    let matches = catalog
        .rows
        .into_iter()
        .filter(|row| row.base_item == base_item)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [row] => Ok(row.clone()),
        [] => Err(error(
            "ITEM-BASEITEM-NOT-FOUND",
            "baseItem",
            format!("BaseItem {base_item} is not present in baseitems.2da"),
        )),
        _ => Err(error(
            "ITEM-BASEITEM-AMBIGUOUS",
            "baseItem",
            format!("BaseItem {base_item} appears more than once in baseitems.2da"),
        )),
    }
}

/// Appends one exact custom ModelType 2 weapon BaseItem by cloning an audited
/// ranged donor. The operation fails before writing if the source table's next
/// physical append index is not exactly the requested output BaseItem.
pub fn append_item_custom_weapon_baseitem_v2(
    bytes: &[u8],
    request: &ItemCustomWeaponBaseItemRequestV2,
) -> Result<ItemCustomWeaponBaseItemArtifactV2, ItemErrorV1> {
    if request.schema_version != 2 {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-SCHEMA-INVALID",
            "request.schemaVersion",
            "custom weapon BaseItem V2 request schemaVersion must be 2",
        ));
    }
    if request.output_base_item == 113
        || request.label.eq_ignore_ascii_case("hextech_shotgun")
        || request.item_class.eq_ignore_ascii_case("WHxSh")
    {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-STANDALONE-RESERVED",
            "request",
            "BaseItem 113 / hextech_shotgun / WHxSh is reserved for standalone V3 authoring",
        ));
    }
    if request.label.is_empty()
        || request.label.len() > 32
        || !request
            .label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-LABEL-INVALID",
            "request.label",
            "custom weapon BaseItem label must contain 1..32 ASCII letters, digits or underscores",
        ));
    }
    // `<ItemClass>_<part>_<nnn>` must remain within Aurora's 16-byte resref.
    if request.item_class.is_empty()
        || request.item_class.len() > 10
        || !request
            .item_class
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-ITEMCLASS-INVALID",
            "request.itemClass",
            "custom weapon ItemClass must contain 1..10 ASCII letters, digits or underscores",
        ));
    }
    if request
        .inv_slot_width
        .into_iter()
        .chain(request.inv_slot_height)
        .any(|value| !(1..=8).contains(&value))
    {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-ICON-SLOTS-INVALID",
            "request.invSlotWidth",
            "custom weapon inventory slot dimensions must each be in 1..8",
        ));
    }

    let catalog = inspect_item_baseitems_v1(bytes)?;
    if catalog.physical_row_count != request.output_base_item {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-OUTPUT-INDEX-MISMATCH",
            "request.outputBaseItem",
            format!(
                "requested output BaseItem {} but the exact next physical append index is {}",
                request.output_base_item, catalog.physical_row_count
            ),
        ));
    }
    if catalog.rows.iter().any(|row| {
        row.base_item == request.output_base_item
            || row.label.eq_ignore_ascii_case(&request.label)
            || row.item_class.eq_ignore_ascii_case(&request.item_class)
    }) {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-IDENTITY-COLLISION",
            "request",
            "output BaseItem, label and ItemClass must be absent from the complete input baseitems.2da",
        ));
    }

    let donor = resolve_item_baseitem_v1(bytes, request.donor_base_item)?;
    let runtime_route = resolve_item_weapon_runtime_route_v1(&donor)?.ok_or_else(|| {
        error(
            "ITEM-CUSTOM-BASEITEM-DONOR-ROUTE-MISSING",
            "request.donorBaseItem",
            "custom weapon donor must expose one exact audited ranged runtime route",
        )
    })?;
    if donor.model_type != 2
        || donor.capability.composition_profile != ItemCompositionProfileV1::BottomMiddleTop
        || donor.part_slots.len() != 3
    {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-DONOR-UNSUPPORTED",
            "request.donorBaseItem",
            "custom weapon donor must be a three-part ModelType 2 weapon",
        ));
    }

    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits).map_err(|source| {
        error(
            "ITEM-CUSTOM-BASEITEM-INSPECTION-FAILED",
            source.path,
            source.message,
        )
    })?;
    let mut donor_physical_row = None;
    for physical_row in 0..inspection.physical_row_count {
        let row = read_two_da_row_v2(bytes, physical_row, &limits).map_err(|source| {
            error(
                "ITEM-CUSTOM-BASEITEM-DONOR-READBACK-FAILED",
                source.path,
                source.message,
            )
        })?;
        if row.printed_row_label == request.donor_base_item
            && donor_physical_row.replace(physical_row).is_some()
        {
            return Err(error(
                "ITEM-CUSTOM-BASEITEM-DONOR-AMBIGUOUS",
                "request.donorBaseItem",
                "custom weapon donor BaseItem appears more than once",
            ));
        }
    }
    let donor_physical_row = donor_physical_row.ok_or_else(|| {
        error(
            "ITEM-CUSTOM-BASEITEM-DONOR-MISSING",
            "request.donorBaseItem",
            "custom weapon donor BaseItem is not present in baseitems.2da",
        )
    })?;

    let text = |column_name: &str, value: String| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text { value },
    };
    let mut overrides = vec![
        text("Label", request.label.clone()),
        text("ItemClass", request.item_class.clone()),
    ];
    if let Some(name_strref) = request.name_strref {
        overrides.push(text("Name", name_strref.to_string()));
    }
    if let Some(inv_slot_width) = request.inv_slot_width {
        overrides.push(text("InvSlotWidth", inv_slot_width.to_string()));
    }
    if let Some(inv_slot_height) = request.inv_slot_height {
        overrides.push(text("InvSlotHeight", inv_slot_height.to_string()));
    }
    let append_request =
        clone_two_da_row_request_v1(bytes, donor_physical_row, &overrides, &limits).map_err(
            |source| {
                error(
                    "ITEM-CUSTOM-BASEITEM-CLONE-FAILED",
                    source.path,
                    source.message,
                )
            },
        )?;
    let append = append_two_da_row_v1(bytes, &append_request, &limits).map_err(|source| {
        error(
            "ITEM-CUSTOM-BASEITEM-APPEND-FAILED",
            source.path,
            source.message,
        )
    })?;
    let appended_base_item = u32::from(append.report.appended_row_index);
    if appended_base_item != request.output_base_item {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-OUTPUT-READBACK-MISMATCH",
            "baseitems.2da.appendedRow",
            "2DA append report did not preserve the requested exact output BaseItem",
        ));
    }
    let selected = resolve_item_baseitem_v1(&append.payload, appended_base_item)?;
    let preserved_semantics = selected.base_item == request.output_base_item
        && selected.label == request.label
        && selected.item_class == request.item_class
        && selected.model_type == donor.model_type
        && selected.min_range == donor.min_range
        && selected.max_range == donor.max_range
        && selected.gender_specific == donor.gender_specific
        && selected.default_model == donor.default_model
        && selected.default_icon == donor.default_icon
        && selected.equipable_slots == donor.equipable_slots
        && selected.weapon_wield == donor.weapon_wield
        && selected.weapon_type == donor.weapon_type
        && selected.ranged_weapon == donor.ranged_weapon
        && selected.inv_slot_width == request.inv_slot_width.unwrap_or(donor.inv_slot_width)
        && selected.inv_slot_height == request.inv_slot_height.unwrap_or(donor.inv_slot_height)
        && selected.capability == donor.capability
        && selected.part_slots == donor.part_slots
        && selected.color_fields == donor.color_fields;
    if !preserved_semantics {
        return Err(error(
            "ITEM-CUSTOM-BASEITEM-SEMANTIC-DIFF",
            "baseitems.2da.appendedRow",
            "appended weapon BaseItem did not preserve every donor runtime semantic outside the explicit identity and inventory-dimension overrides",
        ));
    }

    let report = ItemCustomWeaponBaseItemReportV2 {
        schema_version: 2,
        status: "APPENDED_EXACT".to_owned(),
        donor_base_item: request.donor_base_item,
        output_base_item: request.output_base_item,
        label: request.label.clone(),
        item_class: request.item_class.clone(),
        name_strref: request.name_strref,
        inv_slot_width: selected.inv_slot_width,
        inv_slot_height: selected.inv_slot_height,
        source_sha256: item_payload_sha256_v1(bytes),
        output_sha256: item_payload_sha256_v1(&append.payload),
        runtime_route,
        append_report: append.report,
    };
    Ok(ItemCustomWeaponBaseItemArtifactV2 {
        payload: append.payload,
        selected,
        report,
    })
}

/// Appends one standalone ModelType 2 weapon BaseItem from explicit column
/// assignments. Unlike V2, this path never resolves, reads or clones a donor
/// row. Columns omitted by the caller are written as the native 2DA null cell.
pub fn append_item_standalone_weapon_baseitem_v3(
    bytes: &[u8],
    request: &ItemStandaloneWeaponBaseItemRequestV3,
) -> Result<ItemStandaloneWeaponBaseItemArtifactV3, ItemErrorV1> {
    if request.schema_version != 3 {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-SCHEMA-INVALID",
            "request.schemaVersion",
            "standalone weapon BaseItem request schemaVersion must be 3",
        ));
    }
    if request.label.is_empty()
        || request.label.len() > 32
        || !request
            .label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-LABEL-INVALID",
            "request.label",
            "standalone weapon BaseItem label must contain 1..32 ASCII letters, digits or underscores",
        ));
    }
    if request.item_class.is_empty()
        || request.item_class.len() > 10
        || !request
            .item_class
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-ITEMCLASS-INVALID",
            "request.itemClass",
            "standalone weapon ItemClass must contain 1..10 ASCII letters, digits or underscores",
        ));
    }

    let catalog = inspect_item_baseitems_v1(bytes)?;
    if catalog.physical_row_count != request.output_base_item {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-OUTPUT-INDEX-MISMATCH",
            "request.outputBaseItem",
            format!(
                "requested output BaseItem {} but the exact next physical append index is {}",
                request.output_base_item, catalog.physical_row_count
            ),
        ));
    }
    if catalog.rows.iter().any(|row| {
        row.base_item == request.output_base_item
            || row.label.eq_ignore_ascii_case(&request.label)
            || row.item_class.eq_ignore_ascii_case(&request.item_class)
    }) {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-IDENTITY-COLLISION",
            "request",
            "output BaseItem, label and ItemClass must be absent from the complete input baseitems.2da",
        ));
    }

    let mut assigned_columns = BTreeSet::new();
    for (index, assignment) in request.cells.iter().enumerate() {
        if assignment.column_name.eq_ignore_ascii_case("Label")
            || assignment.column_name.eq_ignore_ascii_case("ItemClass")
        {
            return Err(error(
                "ITEM-STANDALONE-BASEITEM-IDENTITY-CELL-FORBIDDEN",
                format!("request.cells[{index}].columnName"),
                "Label and ItemClass are identity fields and must use the typed request fields",
            ));
        }
        let normalized = assignment.column_name.to_ascii_lowercase();
        if !assigned_columns.insert(normalized) {
            return Err(error(
                "ITEM-STANDALONE-BASEITEM-CELL-DUPLICATE",
                format!("request.cells[{index}].columnName"),
                "standalone BaseItem column assignments must be unique",
            ));
        }
    }

    let text = |column_name: &str, value: String| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text { value },
    };
    let mut cells = request.cells.clone();
    cells.push(text("Label", request.label.clone()));
    cells.push(text("ItemClass", request.item_class.clone()));
    let limits = TwoDaLimitsV1::default();
    let append = append_two_da_row_v1(
        bytes,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &limits,
    )
    .map_err(|source| {
        error(
            "ITEM-STANDALONE-BASEITEM-APPEND-FAILED",
            source.path,
            source.message,
        )
    })?;
    let appended_base_item = u32::from(append.report.appended_row_index);
    if appended_base_item != request.output_base_item {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-OUTPUT-READBACK-MISMATCH",
            "baseitems.2da.appendedRow",
            "2DA append report did not preserve the requested standalone BaseItem",
        ));
    }
    let selected = resolve_item_baseitem_v1(&append.payload, appended_base_item)?;
    if selected.base_item != request.output_base_item
        || selected.label != request.label
        || selected.item_class != request.item_class
        || selected.model_type != 2
        || selected.capability.composition_profile != ItemCompositionProfileV1::BottomMiddleTop
        || selected.part_slots.len() != 3
    {
        return Err(error(
            "ITEM-STANDALONE-BASEITEM-SEMANTIC-READBACK-FAILED",
            "baseitems.2da.appendedRow",
            "standalone weapon must read back as the exact three-part ModelType 2 identity",
        ));
    }
    let report = ItemStandaloneWeaponBaseItemReportV3 {
        schema_version: 3,
        status: "APPENDED_STANDALONE_EXACT".to_owned(),
        output_base_item: request.output_base_item,
        label: request.label.clone(),
        item_class: request.item_class.clone(),
        inv_slot_width: selected.inv_slot_width,
        inv_slot_height: selected.inv_slot_height,
        definition_source: "EXPLICIT_COLUMN_ASSIGNMENTS".to_owned(),
        source_sha256: item_payload_sha256_v1(bytes),
        output_sha256: item_payload_sha256_v1(&append.payload),
        append_report: append.report,
    };
    Ok(ItemStandaloneWeaponBaseItemArtifactV3 {
        payload: append.payload,
        selected,
        report,
    })
}

/// Appends one complete native ammunition visual block. Current EE tables
/// identify every row explicitly with `AmmunitionType` and
/// `DamageRangedProjectile`; retail ordering additionally keeps the block at
/// `DamageRangedProjectile * 6`. Partial blocks are forbidden even when the
/// product currently exposes only the Arrow, Bolt and Bullet inventory
/// channels.
pub fn append_item_ammunition_variant_block_v1(
    bytes: &[u8],
    request: &ItemAmmunitionVariantBlockRequestV1,
) -> Result<ItemAmmunitionVariantBlockArtifactV1, ItemErrorV1> {
    if request.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-AMMUNITION-VARIANT-SCHEMA-INVALID",
            "request.schemaVersion",
            "ammunition variant request schemaVersion must be 1",
        ));
    }
    if request.damage_ranged_projectile < 6 {
        return Err(error(
            "ITEM-AMMUNITION-VARIANT-RETAIL-RESERVED",
            "request.damageRangedProjectile",
            "DamageRangedProjectile values 0..5 are the immutable retail blocks",
        ));
    }
    if request.entries.len() != 6 {
        return Err(error(
            "ITEM-AMMUNITION-VARIANT-BLOCK-INCOMPLETE",
            "request.entries",
            "one damage variant must define exactly six native projectile rows in Arrow/Bolt/Bullet/Dart/Shuriken/ThrowingAxe order",
        ));
    }

    let limits = TwoDaLimitsV1::default();
    let source = inspect_two_da_v2(bytes, &limits).map_err(|source| {
        error(
            "ITEM-AMMUNITIONTYPES-INVALID",
            source.path,
            format!("ammunitiontypes.2da inspection failed: {}", source.message),
        )
    })?;
    for required in [
        "label",
        "Model",
        "ShotSound",
        "ImpactSound",
        "AmmunitionType",
        "DamageRangedProjectile",
    ] {
        required_column(&source.columns, required)?;
    }
    let first_row = u32::from(request.damage_ranged_projectile) * 6;
    if source.physical_row_count > first_row {
        return Err(error(
            "ITEM-AMMUNITION-VARIANT-ROW-COLLISION",
            "request.damageRangedProjectile",
            format!(
                "variant block starts at row {first_row}, but ammunitiontypes.2da already has {} rows",
                source.physical_row_count
            ),
        ));
    }
    if source.physical_row_count < first_row {
        return Err(error(
            "ITEM-AMMUNITION-VARIANT-ROW-GAP",
            "request.damageRangedProjectile",
            format!(
                "variant block starts at row {first_row}, but the next append row is {}; missing earlier blocks cannot be synthesized",
                source.physical_row_count
            ),
        ));
    }

    let mut labels = BTreeSet::new();
    let mut normalized_entries = Vec::with_capacity(6);
    for (index, entry) in request.entries.iter().enumerate() {
        if entry.label.is_empty()
            || entry.label.len() > 32
            || !entry
                .label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            || !labels.insert(entry.label.to_ascii_lowercase())
        {
            return Err(error(
                "ITEM-AMMUNITION-VARIANT-LABEL-INVALID",
                format!("request.entries[{index}].label"),
                "each ammunition label must be unique and contain 1..32 ASCII letters, digits or underscores",
            ));
        }
        normalized_entries.push(ItemAmmunitionVariantEntryV1 {
            label: entry.label.clone(),
            model_resref: validate_resref(
                &entry.model_resref,
                &format!("request.entries[{index}].modelResref"),
            )?,
            shot_sound_resref: entry
                .shot_sound_resref
                .as_deref()
                .map(|value| {
                    validate_resref(value, &format!("request.entries[{index}].shotSoundResref"))
                })
                .transpose()?,
            impact_sound_resref: entry
                .impact_sound_resref
                .as_deref()
                .map(|value| {
                    validate_resref(
                        value,
                        &format!("request.entries[{index}].impactSoundResref"),
                    )
                })
                .transpose()?,
        });
    }

    let text = |column_name: &str, value: String| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text { value },
    };
    let optional = |column_name: &str, value: &Option<String>| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: value
            .as_ref()
            .map(|value| TwoDaCellValueV1::Text {
                value: value.clone(),
            })
            .unwrap_or(TwoDaCellValueV1::Null),
    };
    let mut payload = bytes.to_vec();
    let mut append_reports = Vec::with_capacity(6);
    for (offset, entry) in normalized_entries.iter().enumerate() {
        let ammunition_type = u8::try_from(offset + 1).map_err(|_| {
            error(
                "ITEM-AMMUNITION-VARIANT-TYPE-INVALID",
                "request.entries",
                "native ammunition type does not fit BYTE",
            )
        })?;
        let append = append_two_da_row_v1(
            &payload,
            &TwoDaAppendRequestV1 {
                schema_version: 1,
                cells: vec![
                    text("label", entry.label.clone()),
                    text("Model", entry.model_resref.clone()),
                    optional("ShotSound", &entry.shot_sound_resref),
                    optional("ImpactSound", &entry.impact_sound_resref),
                    text("AmmunitionType", ammunition_type.to_string()),
                    text(
                        "DamageRangedProjectile",
                        request.damage_ranged_projectile.to_string(),
                    ),
                ],
            },
            &limits,
        )
        .map_err(|source| {
            error(
                "ITEM-AMMUNITION-VARIANT-APPEND-FAILED",
                source.path,
                source.message,
            )
        })?;
        append_reports.push(append.report);
        payload = append.payload;
    }

    let output = inspect_two_da_v2(&payload, &limits).map_err(|source| {
        error(
            "ITEM-AMMUNITION-VARIANT-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    if output.physical_row_count != first_row + 6 {
        return Err(error(
            "ITEM-AMMUNITION-VARIANT-READBACK-FAILED",
            "ammunitiontypes.2da.rows",
            "appended ammunition block does not contain exactly six new rows",
        ));
    }
    let label_index = required_column(&output.columns, "label")?;
    let model_index = required_column(&output.columns, "Model")?;
    let shot_index = required_column(&output.columns, "ShotSound")?;
    let impact_index = required_column(&output.columns, "ImpactSound")?;
    let ammunition_type_index = required_column(&output.columns, "AmmunitionType")?;
    let damage_index = required_column(&output.columns, "DamageRangedProjectile")?;
    let expected_cell = |value: &Option<String>| {
        value
            .as_ref()
            .map(|value| TwoDaCellValueV1::Text {
                value: value.clone(),
            })
            .unwrap_or(TwoDaCellValueV1::Null)
    };
    let mut rows = Vec::with_capacity(6);
    for (offset, entry) in normalized_entries.into_iter().enumerate() {
        let row = first_row + offset as u32;
        let ammunition_type = u8::try_from(offset + 1).map_err(|_| {
            error(
                "ITEM-AMMUNITION-VARIANT-TYPE-INVALID",
                "request.entries",
                "native ammunition type does not fit BYTE",
            )
        })?;
        let readback = read_two_da_row_v2(&payload, row, &limits).map_err(|source| {
            error(
                "ITEM-AMMUNITION-VARIANT-READBACK-FAILED",
                source.path,
                source.message,
            )
        })?;
        if readback.printed_row_label != row
            || readback.cells[label_index]
                != (TwoDaCellValueV1::Text {
                    value: entry.label.clone(),
                })
            || readback.cells[model_index]
                != (TwoDaCellValueV1::Text {
                    value: entry.model_resref.clone(),
                })
            || readback.cells[shot_index] != expected_cell(&entry.shot_sound_resref)
            || readback.cells[impact_index] != expected_cell(&entry.impact_sound_resref)
            || readback.cells[ammunition_type_index]
                != (TwoDaCellValueV1::Text {
                    value: ammunition_type.to_string(),
                })
            || readback.cells[damage_index]
                != (TwoDaCellValueV1::Text {
                    value: request.damage_ranged_projectile.to_string(),
                })
        {
            return Err(error(
                "ITEM-AMMUNITION-VARIANT-SEMANTIC-DIFF",
                format!("ammunitiontypes.2da.rows[{row}]"),
                "appended ammunition row differs from the exact request",
            ));
        }
        rows.push(ItemAmmunitionVariantRowReportV1 {
            row,
            label: entry.label,
            model_resref: entry.model_resref,
            shot_sound_resref: entry.shot_sound_resref,
            impact_sound_resref: entry.impact_sound_resref,
            ammunition_type,
            damage_ranged_projectile: request.damage_ranged_projectile,
        });
    }

    Ok(ItemAmmunitionVariantBlockArtifactV1 {
        report: ItemAmmunitionVariantBlockReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            status: "APPENDED_COMPLETE_DAMAGE_VARIANT_BLOCK".to_owned(),
            damage_ranged_projectile: request.damage_ranged_projectile,
            first_row,
            last_row: first_row + 5,
            source_sha256: item_payload_sha256_v1(bytes),
            output_sha256: item_payload_sha256_v1(&payload),
            rows,
            append_reports,
            semantic_readback_status: "PASS".to_owned(),
        },
        payload,
    })
}

/// Binds an existing EE damage type to one custom ranged-projectile visual
/// block. The source row must be neutral (`0`) or already bound to the exact
/// requested block; an existing different binding is a collision.
pub fn patch_item_damage_ranged_projectile_v1(
    bytes: &[u8],
    request: &ItemDamageRangedProjectileRequestV1,
) -> Result<ItemDamageRangedProjectileArtifactV1, ItemErrorV1> {
    if request.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-DAMAGE-RANGED-PROJECTILE-SCHEMA-INVALID",
            "request.schemaVersion",
            "damage projectile request schemaVersion must be 1",
        ));
    }
    if request.damage_ranged_projectile < 6 {
        return Err(error(
            "ITEM-DAMAGE-RANGED-PROJECTILE-RETAIL-RESERVED",
            "request.damageRangedProjectile",
            "custom damage projectile bindings must use a value in 6..255",
        ));
    }
    if request.expected_label.is_empty()
        || request.expected_label.len() > 64
        || !request
            .expected_label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(error(
            "ITEM-DAMAGE-TYPE-LABEL-INVALID",
            "request.expectedLabel",
            "damage type label must contain 1..64 ASCII letters, digits, underscores or hyphens",
        ));
    }

    let limits = TwoDaLimitsV1::default();
    let source = inspect_two_da_v2(bytes, &limits).map_err(|source| {
        error(
            "ITEM-DAMAGETYPES-INVALID",
            source.path,
            format!("damagetypes.2da inspection failed: {}", source.message),
        )
    })?;
    let label_index = required_column(&source.columns, "Label")?;
    let damage_index = required_column(&source.columns, "DamageRangedProjectile")?;
    let (physical_row_index, row) = (0..source.physical_row_count)
        .find_map(|physical_row_index| {
            let row = read_two_da_row_v2(bytes, physical_row_index, &limits).ok()?;
            (row.printed_row_label == request.damage_type_row).then_some((physical_row_index, row))
        })
        .ok_or_else(|| {
            error(
                "ITEM-DAMAGE-TYPE-ROW-NOT-FOUND",
                "request.damageTypeRow",
                format!(
                    "DamageTypes.2DA row {} was not found",
                    request.damage_type_row
                ),
            )
        })?;
    let source_label = match &row.cells[label_index] {
        TwoDaCellValueV1::Text { value } => value,
        TwoDaCellValueV1::Null => "",
    };
    if source_label != request.expected_label {
        return Err(error(
            "ITEM-DAMAGE-TYPE-LABEL-MISMATCH",
            "request.expectedLabel",
            format!(
                "DamageTypes.2DA row {} has label {:?}, expected {:?}",
                request.damage_type_row, source_label, request.expected_label
            ),
        ));
    }
    let source_damage = match &row.cells[damage_index] {
        TwoDaCellValueV1::Text { value } => value.parse::<u8>().map_err(|_| {
            error(
                "ITEM-DAMAGE-RANGED-PROJECTILE-INVALID",
                format!(
                    "DamageTypes.2DA.rows[{}].DamageRangedProjectile",
                    request.damage_type_row
                ),
                "DamageRangedProjectile must be an integer in 0..255",
            )
        })?,
        TwoDaCellValueV1::Null => 0,
    };
    if source_damage != 0 && source_damage != request.damage_ranged_projectile {
        return Err(error(
            "ITEM-DAMAGE-RANGED-PROJECTILE-COLLISION",
            "request.damageRangedProjectile",
            format!(
                "DamageTypes.2DA row {} already selects projectile variant {}",
                request.damage_type_row, source_damage
            ),
        ));
    }

    let patched = patch_two_da_row_v1(
        bytes,
        &TwoDaRowPatchRequestV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            physical_row_index,
            expected_printed_row_label: request.damage_type_row,
            cells: vec![TwoDaCellPatchV1 {
                column_name: "DamageRangedProjectile".to_owned(),
                expected_value: TwoDaCellValueV1::Text {
                    value: source_damage.to_string(),
                },
                value: TwoDaCellValueV1::Text {
                    value: request.damage_ranged_projectile.to_string(),
                },
            }],
        },
        &limits,
    )
    .map_err(|source| {
        error(
            "ITEM-DAMAGE-RANGED-PROJECTILE-PATCH-FAILED",
            source.path,
            source.message,
        )
    })?;
    let output = inspect_two_da_v2(&patched.payload, &limits).map_err(|source| {
        error(
            "ITEM-DAMAGE-RANGED-PROJECTILE-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let output_label_index = required_column(&output.columns, "Label")?;
    let output_damage_index = required_column(&output.columns, "DamageRangedProjectile")?;
    let output_row =
        read_two_da_row_v2(&patched.payload, physical_row_index, &limits).map_err(|source| {
            error(
                "ITEM-DAMAGE-RANGED-PROJECTILE-READBACK-FAILED",
                source.path,
                source.message,
            )
        })?;
    if output_row.printed_row_label != request.damage_type_row
        || output_row.cells[output_label_index]
            != (TwoDaCellValueV1::Text {
                value: request.expected_label.clone(),
            })
        || output_row.cells[output_damage_index]
            != (TwoDaCellValueV1::Text {
                value: request.damage_ranged_projectile.to_string(),
            })
    {
        return Err(error(
            "ITEM-DAMAGE-RANGED-PROJECTILE-SEMANTIC-DIFF",
            format!("DamageTypes.2DA.rows[{}]", request.damage_type_row),
            "patched damage type row differs from the exact request",
        ));
    }

    let output_sha256 = item_payload_sha256_v1(&patched.payload);
    Ok(ItemDamageRangedProjectileArtifactV1 {
        payload: patched.payload,
        report: ItemDamageRangedProjectileReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            status: if source_damage == request.damage_ranged_projectile {
                "ALREADY_BOUND_DAMAGE_RANGED_PROJECTILE".to_owned()
            } else {
                "PATCHED_DAMAGE_RANGED_PROJECTILE".to_owned()
            },
            damage_type_row: request.damage_type_row,
            expected_label: request.expected_label.clone(),
            source_damage_ranged_projectile: source_damage,
            damage_ranged_projectile: request.damage_ranged_projectile,
            source_sha256: item_payload_sha256_v1(bytes),
            output_sha256,
            patch_report: patched.report,
            semantic_readback_status: "PASS".to_owned(),
        },
    })
}

/// Ensures Aurora's ModelType 2 selector scan reaches the selected model
/// bucket. This remains an existing BaseItem: only that row's `MaxRange` is
/// raised, and only when the selected model lies above the source range.
pub fn extend_item_baseitem_model_range_v1(
    bytes: &[u8],
    base_item: u32,
    model: u8,
) -> Result<ItemBaseItemModelRangeArtifactV1, ItemErrorV1> {
    if model > ITEM_WEAPON_MODEL_MAX_V1 {
        return Err(error(
            "ITEM-WEAPON-MODEL-OUT-OF-RANGE",
            "model",
            format!("weapon model must be in 0..={ITEM_WEAPON_MODEL_MAX_V1}, got {model}"),
        ));
    }
    let selected = resolve_item_baseitem_v1(bytes, base_item)?;
    if selected.model_type != 2 {
        return Err(error(
            "ITEM-BASEITEM-RANGE-UNSUPPORTED",
            "baseItem",
            format!(
                "BaseItem {base_item} has ModelType {}, expected ModelType 2",
                selected.model_type
            ),
        ));
    }
    let source_min_range = selected.min_range.ok_or_else(|| {
        error(
            "ITEM-BASEITEM-RANGE-MISSING",
            format!("baseitems.2da.rows.{base_item}.MinRange"),
            "ModelType 2 BaseItem requires MinRange",
        )
    })?;
    let source_max_range = selected.max_range.ok_or_else(|| {
        error(
            "ITEM-BASEITEM-RANGE-MISSING",
            format!("baseitems.2da.rows.{base_item}.MaxRange"),
            "ModelType 2 BaseItem requires MaxRange",
        )
    })?;
    if source_min_range > source_max_range {
        return Err(error(
            "ITEM-BASEITEM-RANGE-INVALID",
            format!("baseitems.2da.rows.{base_item}"),
            format!("MinRange {source_min_range} exceeds MaxRange {source_max_range}"),
        ));
    }
    let model_bucket = u32::from(model) * 10;
    if model_bucket < source_min_range {
        return Err(error(
            "ITEM-BASEITEM-MODEL-BELOW-MIN-RANGE",
            "model",
            format!(
                "model bucket {model_bucket} is below BaseItem {base_item} MinRange {source_min_range}"
            ),
        ));
    }

    let source_sha256 = item_payload_sha256_v1(bytes);
    if model_bucket <= source_max_range {
        return Ok(ItemBaseItemModelRangeArtifactV1 {
            payload: bytes.to_vec(),
            report: ItemBaseItemModelRangeReportV1 {
                schema_version: ITEM_SCHEMA_VERSION_V1,
                status: "NOT_REQUIRED".to_owned(),
                base_item,
                model,
                model_bucket,
                source_min_range,
                source_max_range,
                effective_max_range: source_max_range,
                source_sha256: source_sha256.clone(),
                output_sha256: source_sha256,
                patch_report: None,
            },
        });
    }

    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits).map_err(|source| {
        error(
            "ITEM-BASEITEMS-INVALID",
            source.path,
            format!("baseitems.2da inspection failed: {}", source.message),
        )
    })?;
    let physical_row_index = (0..inspection.physical_row_count)
        .find(|physical_row| {
            read_two_da_row_v2(bytes, *physical_row, &limits)
                .map(|row| row.printed_row_label == base_item)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            error(
                "ITEM-BASEITEM-NOT-FOUND",
                "baseItem",
                format!("BaseItem {base_item} is not present in baseitems.2da"),
            )
        })?;
    let patched = patch_two_da_row_v1(
        bytes,
        &TwoDaRowPatchRequestV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            physical_row_index,
            expected_printed_row_label: base_item,
            cells: vec![TwoDaCellPatchV1 {
                column_name: "MaxRange".to_owned(),
                expected_value: TwoDaCellValueV1::Text {
                    value: source_max_range.to_string(),
                },
                value: TwoDaCellValueV1::Text {
                    value: model_bucket.to_string(),
                },
            }],
        },
        &limits,
    )
    .map_err(|source| {
        error(
            "ITEM-BASEITEM-RANGE-PATCH-FAILED",
            source.path,
            format!("baseitems.2da row patch failed: {}", source.message),
        )
    })?;
    let output_row = resolve_item_baseitem_v1(&patched.payload, base_item)?;
    if output_row.min_range != Some(source_min_range) || output_row.max_range != Some(model_bucket)
    {
        return Err(error(
            "ITEM-BASEITEM-RANGE-READBACK-FAILED",
            "baseitems.2da",
            "patched baseitems.2da did not preserve MinRange and raise only MaxRange",
        ));
    }
    Ok(ItemBaseItemModelRangeArtifactV1 {
        report: ItemBaseItemModelRangeReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            status: "PATCHED".to_owned(),
            base_item,
            model,
            model_bucket,
            source_min_range,
            source_max_range,
            effective_max_range: model_bucket,
            source_sha256,
            output_sha256: item_payload_sha256_v1(&patched.payload),
            patch_report: Some(patched.report),
        },
        payload: patched.payload,
    })
}

fn two_da_cell_by_column_v1<'a>(
    table_name: &str,
    columns: &[String],
    cells: &'a [TwoDaCellValueV1],
    row: u32,
    column: &str,
) -> Result<&'a str, ItemErrorV1> {
    let matches = columns
        .iter()
        .enumerate()
        .filter(|(_, value)| value.eq_ignore_ascii_case(column))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let index = match matches.as_slice() {
        [index] => *index,
        [] => {
            return Err(error(
                "ITEM-REFERENCE-COLUMN-MISSING",
                format!("{table_name}.columns.{column}"),
                format!("{table_name} has no required {column} column"),
            ));
        }
        _ => {
            return Err(error(
                "ITEM-REFERENCE-COLUMN-DUPLICATE",
                format!("{table_name}.columns.{column}"),
                format!("{table_name} has more than one {column} column"),
            ));
        }
    };
    match cells.get(index) {
        Some(TwoDaCellValueV1::Text { value })
            if !value.trim().is_empty() && value.trim() != "*" && value.trim() != "****" =>
        {
            Ok(value.trim())
        }
        _ => Err(error(
            "ITEM-REFERENCE-VALUE-MISSING",
            format!("{table_name}.rows.{row}.{column}"),
            format!("{table_name} row {row} has no {column} value"),
        )),
    }
}

fn terminal_newline_v1(bytes: &[u8], end: usize) -> Option<(usize, usize)> {
    if end >= 2 && bytes.get(end - 2..end) == Some(b"\r\n") {
        Some((end - 2, 2))
    } else if end >= 1 && bytes.get(end - 1) == Some(&b'\n') {
        Some((end - 1, 1))
    } else {
        None
    }
}

/// Retail item reference tables may contain one or more empty physical records
/// after their final data row. Keep the generic 2DA parser strict and remove
/// only those terminal, same-newline-style, empty-or-space-only records at the
/// Item boundary.
fn normalize_item_reference_two_da_v1(bytes: &[u8]) -> (&[u8], u32) {
    let mut end = bytes.len();
    let mut stripped = 0_u32;
    while let Some((terminal_start, terminal_width)) = terminal_newline_v1(bytes, end) {
        let Some(previous_lf) = bytes[..terminal_start]
            .iter()
            .rposition(|byte| *byte == b'\n')
        else {
            break;
        };
        let previous_end = previous_lf + 1;
        let previous_width = if previous_lf > 0 && bytes[previous_lf - 1] == b'\r' {
            2
        } else {
            1
        };
        if previous_width != terminal_width
            || !bytes[previous_end..terminal_start]
                .iter()
                .all(|byte| *byte == b' ')
        {
            break;
        }
        end = previous_end;
        stripped = stripped.saturating_add(1);
    }
    (&bytes[..end], stripped)
}

pub fn inspect_item_reference_two_da_v1(
    table_name: &str,
    bytes: &[u8],
) -> Result<ItemReferenceTableInspectionV1, ItemErrorV1> {
    let (normalized, stripped_trailing_blank_row_count) = normalize_item_reference_two_da_v1(bytes);
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(normalized, &limits).map_err(|source| {
        error(
            "ITEM-REFERENCE-2DA-INVALID",
            source.path,
            format!("{table_name} inspection failed: {}", source.message),
        )
    })?;
    Ok(ItemReferenceTableInspectionV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        table_name: table_name.to_owned(),
        source_sha256: item_payload_sha256_v1(bytes),
        source_byte_length: bytes.len() as u64,
        normalized_byte_length: normalized.len() as u64,
        stripped_trailing_blank_row_count,
        inspection,
    })
}

fn read_reference_row_v1(
    table_name: &str,
    bytes: &[u8],
    physical_row: u32,
) -> Result<(Vec<String>, Vec<TwoDaCellValueV1>), ItemErrorV1> {
    let (normalized, _) = normalize_item_reference_two_da_v1(bytes);
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_item_reference_two_da_v1(table_name, bytes)?.inspection;
    if physical_row >= inspection.physical_row_count {
        return Err(error(
            "ITEM-REFERENCE-ROW-MISSING",
            format!("{table_name}.rows.{physical_row}"),
            format!(
                "{table_name} has {} physical rows; row {physical_row} is unavailable",
                inspection.physical_row_count
            ),
        ));
    }
    let row = read_two_da_row_v2(normalized, physical_row, &limits).map_err(|source| {
        error(
            "ITEM-REFERENCE-2DA-INVALID",
            source.path,
            format!("{table_name} row readback failed: {}", source.message),
        )
    })?;
    Ok((inspection.columns, row.cells))
}

fn columns_equal_v1(columns: &[String], expected: &[&str]) -> bool {
    columns.len() == expected.len()
        && columns
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.eq_ignore_ascii_case(expected))
}

pub fn resolve_item_capart_part_v1(
    field: &str,
    selector: u8,
    capart_2da: &[u8],
    parts_table_name: &str,
    parts_2da: &[u8],
) -> Result<ItemCapartPartResolutionV1, ItemErrorV1> {
    let &(canonical_field, capart_row, expected_mdl_name, expected_node_name, expected_parts_table) =
        CAPART_PART_PROFILES_V1
            .iter()
            .find(|(candidate, ..)| candidate.eq_ignore_ascii_case(field))
            .ok_or_else(|| {
                error(
                    "ITEM-CAPART-FIELD-INVALID",
                    "field",
                    format!("{field} is not one of the 19 native ArmorPart_* fields"),
                )
            })?;
    let normalized_parts_table = parts_table_name.trim().to_ascii_uppercase();
    if normalized_parts_table != expected_parts_table {
        return Err(error(
            "ITEM-CAPART-PARTS-TABLE-MISMATCH",
            format!("{canonical_field}.partsTable"),
            format!(
                "{canonical_field} is resolved by {expected_parts_table}.2da, not {parts_table_name}"
            ),
        ));
    }

    let capart_inspection = inspect_item_reference_two_da_v1("CAPART", capart_2da)?.inspection;
    let expected_capart_columns = ["NAME", "MDLNAME", "NODENAME"];
    if capart_inspection.physical_row_count != CAPART_PART_PROFILES_V1.len() as u32
        || !columns_equal_v1(&capart_inspection.columns, &expected_capart_columns)
    {
        return Err(error(
            "ITEM-CAPART-TABLE-SCHEMA-MISMATCH",
            "CAPART",
            "CAPART.2da must contain exactly 19 physical rows and the exact NAME/MDLNAME/NODENAME columns",
        ));
    }
    let (capart_columns, capart_cells) = read_reference_row_v1("CAPART", capart_2da, capart_row)?;
    let mdl_name = two_da_cell_by_column_v1(
        "CAPART",
        &capart_columns,
        &capart_cells,
        capart_row,
        "MDLNAME",
    )?;
    let node_name = two_da_cell_by_column_v1(
        "CAPART",
        &capart_columns,
        &capart_cells,
        capart_row,
        "NODENAME",
    )?;
    if !mdl_name.eq_ignore_ascii_case(expected_mdl_name) {
        return Err(error(
            "ITEM-CAPART-MAPPING-MISMATCH",
            format!("CAPART.rows.{capart_row}.MDLNAME"),
            format!(
                "{canonical_field} requires CAPART MDLNAME {expected_mdl_name}, found {mdl_name}"
            ),
        ));
    }
    if !node_name.eq_ignore_ascii_case(expected_node_name) {
        return Err(error(
            "ITEM-CAPART-MAPPING-MISMATCH",
            format!("CAPART.rows.{capart_row}.NODENAME"),
            format!(
                "{canonical_field} requires CAPART NODENAME {expected_node_name}, found {node_name}"
            ),
        ));
    }

    let parts_inspection =
        inspect_item_reference_two_da_v1(expected_parts_table, parts_2da)?.inspection;
    let mut expected_parts_columns = vec!["COSTMODIFIER", "ACBONUS"];
    if expected_parts_table == "PARTS_ROBE" {
        expected_parts_columns.extend(
            CAPART_ROBE_HIDE_COLUMNS_V1
                .iter()
                .map(|(column, _)| *column),
        );
    }
    if !columns_equal_v1(&parts_inspection.columns, &expected_parts_columns) {
        return Err(error(
            "ITEM-CAPART-PARTS-TABLE-SCHEMA-MISMATCH",
            expected_parts_table,
            format!(
                "{expected_parts_table}.2da columns do not match its exact CAPART selector contract"
            ),
        ));
    }
    let available_part_count = parts_inspection.physical_row_count;
    if u32::from(selector) >= available_part_count {
        return Err(error(
            "ITEM-CAPART-SELECTOR-OUT-OF-RANGE",
            format!("{canonical_field}.selector"),
            format!(
                "{canonical_field} selector {selector} exceeds the {available_part_count} rows available in {expected_parts_table}.2da"
            ),
        ));
    }
    let (parts_columns, parts_cells) =
        read_reference_row_v1(expected_parts_table, parts_2da, u32::from(selector))?;

    let mut robe_hidden_mdl_names = Vec::new();
    if expected_parts_table == "PARTS_ROBE" {
        for &(column, hidden_mdl_name) in &CAPART_ROBE_HIDE_COLUMNS_V1 {
            let value = two_da_cell_by_column_v1(
                expected_parts_table,
                &parts_columns,
                &parts_cells,
                u32::from(selector),
                column,
            )?;
            match value {
                "0" => {}
                "1" => robe_hidden_mdl_names.push(hidden_mdl_name.to_owned()),
                _ => {
                    return Err(error(
                        "ITEM-CAPART-ROBE-MASK-INVALID",
                        format!("{expected_parts_table}.rows.{selector}.{column}"),
                        format!("{column} must be the native binary flag 0 or 1"),
                    ));
                }
            }
        }
    }

    Ok(ItemCapartPartResolutionV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        field: canonical_field.to_owned(),
        selector,
        capart_row,
        mdl_name: expected_mdl_name.to_owned(),
        node_name: expected_node_name.to_owned(),
        parts_table: expected_parts_table.to_owned(),
        available_part_count,
        robe_hidden_mdl_names,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_item_capart_part_v2(
    field: &str,
    selector: u8,
    capart_2da: &[u8],
    parts_table_name: &str,
    parts_2da: &[u8],
    context: &ItemCapartContextV1,
    resource_inventory: &[ItemResourceInventoryEntryV2],
    hidden_by_robe: bool,
) -> Result<ItemCapartPartResolutionV2, ItemErrorV1> {
    if context.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-CAPART-CONTEXT-SCHEMA-INVALID",
            "context.schemaVersion",
            "CAPART context schemaVersion must be 1",
        ));
    }
    let model_prefix = validate_item_player_model_prefix_v1(&context.model_prefix)?;
    let gender_code = context
        .gender_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.len() != 1 || !value.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
                Err(error(
                    "ITEM-CAPART-GENDER-CODE-INVALID",
                    "context.genderCode",
                    "optional genderCode must be exactly one ASCII letter or digit",
                ))
            } else {
                Ok(value.to_ascii_lowercase())
            }
        })
        .transpose()?;
    let base =
        resolve_item_capart_part_v1(field, selector, capart_2da, parts_table_name, parts_2da)?;
    let model_token = base.mdl_name.to_ascii_lowercase();
    let mut model_candidates = Vec::new();
    if let Some(gender_code) = &gender_code {
        model_candidates.push(validate_resref(
            &format!("{model_prefix}{gender_code}_{model_token}{selector:03}"),
            "context.genderSpecificModelResref",
        )?);
    }
    model_candidates.push(validate_resref(
        &format!("{model_prefix}_{model_token}{selector:03}"),
        "context.fallbackModelResref",
    )?);
    let inventory = validate_item_resource_inventory_v1(resource_inventory)?;
    let (resolved_model_resref, model_resource, palette_resource, resource_verification) =
        if hidden_by_robe {
            (None, None, None, "SKIPPED_BY_ROBE_MASK".to_owned())
        } else {
            let resolved = model_candidates
                .iter()
                .find_map(|candidate| {
                    let model = inventory.get(&(2002, candidate.clone())).cloned()?;
                    let palette = inventory.get(&(6, candidate.clone())).cloned()?;
                    Some((candidate.clone(), model, palette))
                })
                .ok_or_else(|| {
                    error(
                        "ITEM-CAPART-CONTEXT-RESOURCE-MISSING",
                        format!("{}.resourceInventory", base.field),
                        format!(
                            "{} selector {} requires matching contextual MDL and PLT resources: {}",
                            base.field,
                            selector,
                            model_candidates.join(" or ")
                        ),
                    )
                })?;
            (
                Some(resolved.0),
                Some(resolved.1),
                Some(resolved.2),
                "PINNED_MANIFEST_PAYLOAD_VALIDATED".to_owned(),
            )
        };
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ContextBinding<'a> {
        context: &'a ItemCapartContextV1,
        field: &'a str,
        selector: u8,
        model_candidates: &'a [String],
        resolved_model_resref: &'a Option<String>,
        model_resource: &'a Option<ItemResourceInventoryEntryV2>,
        palette_resource: &'a Option<ItemResourceInventoryEntryV2>,
        hidden_by_robe: bool,
    }
    let context_bytes = serde_json::to_vec(&ContextBinding {
        context,
        field: &base.field,
        selector,
        model_candidates: &model_candidates,
        resolved_model_resref: &resolved_model_resref,
        model_resource: &model_resource,
        palette_resource: &palette_resource,
        hidden_by_robe,
    })
    .map_err(|_| {
        error(
            "ITEM-CAPART-CONTEXT-SERIALIZE-FAILED",
            "context",
            "CAPART context binding could not be serialized",
        )
    })?;
    Ok(ItemCapartPartResolutionV2 {
        schema_version: 2,
        field: base.field,
        selector: base.selector,
        capart_row: base.capart_row,
        mdl_name: base.mdl_name,
        node_name: base.node_name,
        parts_table: base.parts_table,
        available_part_count: base.available_part_count,
        robe_hidden_mdl_names: base.robe_hidden_mdl_names,
        hidden_by_robe,
        model_candidates,
        resolved_model_resref,
        model_resource,
        palette_resource,
        context_sha256: item_payload_sha256_v1(&context_bytes),
        resource_verification,
    })
}

pub fn resolve_item_cast_spell_icon_v1(
    base_item: u32,
    properties: &[ItemPropertyV1],
    iprp_spells_2da: &[u8],
) -> Result<ItemCastSpellIconV1, ItemErrorV1> {
    if !matches!(
        base_item,
        ITEM_BASEITEM_SPELL_SCROLL_DELETED_V1 | ITEM_BASEITEM_SPELL_SCROLL_V1
    ) {
        return Err(error(
            "ITEM-IPRP-SPELL-BASEITEM-INVALID",
            "baseItem",
            "IPRP_SPELLS icon override is defined only for BaseItem 54 and 75",
        ));
    }
    let matches = properties
        .iter()
        .filter(|property| property.property_name == ITEM_PROPERTY_CAST_SPELL_V1)
        .collect::<Vec<_>>();
    let property = match matches.as_slice() {
        [property] => *property,
        [] => {
            return Err(error(
                "ITEM-IPRP-SPELL-PROPERTY-MISSING",
                "properties",
                "this BaseItem needs exactly one Cast Spell property (PropertyName 15)",
            ));
        }
        _ => {
            return Err(error(
                "ITEM-IPRP-SPELL-PROPERTY-AMBIGUOUS",
                "properties",
                "this BaseItem has more than one Cast Spell property",
            ));
        }
    };
    let row = u32::from(property.subtype);
    let (columns, cells) = read_reference_row_v1("IPRP_SPELLS", iprp_spells_2da, row)?;
    let icon = two_da_cell_by_column_v1("IPRP_SPELLS", &columns, &cells, row, "Icon")?;
    Ok(ItemCastSpellIconV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        base_item,
        cast_spell_subtype: property.subtype,
        icon_resref: validate_resref(icon, "IPRP_SPELLS.Icon")?,
    })
}

pub fn resolve_item_cloak_v1(
    cloak_model_row: u16,
    cloak_model_2da: &[u8],
) -> Result<ItemCloakResolutionV1, ItemErrorV1> {
    let row = u32::from(cloak_model_row);
    let (columns, cells) = read_reference_row_v1("CloakModel", cloak_model_2da, row)?;
    let model_text = two_da_cell_by_column_v1("CloakModel", &columns, &cells, row, "MODEL")?;
    let texture_text = two_da_cell_by_column_v1("CloakModel", &columns, &cells, row, "TEXTURE")?;
    let icon_text = two_da_cell_by_column_v1("CloakModel", &columns, &cells, row, "ICON")?;
    let model = u16::try_from(parse_u32(model_text, row, "MODEL")?).map_err(|_| {
        error(
            "ITEM-CLOAK-MODEL-OVERFLOW",
            format!("CloakModel.rows.{row}.MODEL"),
            "CloakModel MODEL must fit an unsigned 16-bit value",
        )
    })?;
    let texture = u16::try_from(parse_u32(texture_text, row, "TEXTURE")?).map_err(|_| {
        error(
            "ITEM-CLOAK-TEXTURE-OVERFLOW",
            format!("CloakModel.rows.{row}.TEXTURE"),
            "CloakModel TEXTURE must fit an unsigned 16-bit value",
        )
    })?;
    let icon = u16::try_from(parse_u32(icon_text, row, "ICON")?).map_err(|_| {
        error(
            "ITEM-CLOAK-ICON-OVERFLOW",
            format!("CloakModel.rows.{row}.ICON"),
            "CloakModel ICON must fit an unsigned 16-bit value",
        )
    })?;
    Ok(ItemCloakResolutionV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        cloak_model_row,
        model,
        texture,
        icon,
        model_resref: validate_resref(&format!("pmh0_cloak_{model:03}"), "CloakModel.modelResref")?,
        texture_resref: validate_resref(
            &format!("cloak_{texture:03}"),
            "CloakModel.textureResref",
        )?,
        icon_resref: validate_resref(&format!("icloak_m_{icon:03}"), "CloakModel.iconResref")?,
    })
}

pub fn resolve_item_cloak_v2(
    cloak_model_row: u16,
    cloak_model_2da: &[u8],
    available_resource_keys: &[impl AsRef<str>],
) -> Result<ItemCloakResolutionV2, ItemErrorV1> {
    resolve_item_cloak_for_model_prefix_v2(
        cloak_model_row,
        "pmh0",
        cloak_model_2da,
        available_resource_keys,
    )
}

fn resolve_item_cloak_for_model_prefix_v2(
    cloak_model_row: u16,
    model_prefix: &str,
    cloak_model_2da: &[u8],
    available_resource_keys: &[impl AsRef<str>],
) -> Result<ItemCloakResolutionV2, ItemErrorV1> {
    let model_prefix = validate_item_player_model_prefix_v1(model_prefix)?;
    let icon_gender = model_prefix
        .as_bytes()
        .get(1)
        .copied()
        .map(char::from)
        .expect("validated player model prefix has a gender token");
    let row = u32::from(cloak_model_row);
    let (columns, cells) = read_reference_row_v1("CloakModel", cloak_model_2da, row)?;
    let model_text = two_da_cell_by_column_v1("CloakModel", &columns, &cells, row, "MODEL")?;
    let texture_text = two_da_cell_by_column_v1("CloakModel", &columns, &cells, row, "TEXTURE")?;
    let icon_text = two_da_cell_by_column_v1("CloakModel", &columns, &cells, row, "ICON")?;
    let model = u16::try_from(parse_u32(model_text, row, "MODEL")?).map_err(|_| {
        error(
            "ITEM-CLOAK-MODEL-OVERFLOW",
            format!("CloakModel.rows.{row}.MODEL"),
            "CloakModel MODEL must fit an unsigned 16-bit value",
        )
    })?;
    let texture = u16::try_from(parse_u32(texture_text, row, "TEXTURE")?).map_err(|_| {
        error(
            "ITEM-CLOAK-TEXTURE-OVERFLOW",
            format!("CloakModel.rows.{row}.TEXTURE"),
            "CloakModel TEXTURE must fit an unsigned 16-bit value",
        )
    })?;
    let icon = u16::try_from(parse_u32(icon_text, row, "ICON")?).map_err(|_| {
        error(
            "ITEM-CLOAK-ICON-OVERFLOW",
            format!("CloakModel.rows.{row}.ICON"),
            "CloakModel ICON must fit an unsigned 16-bit value",
        )
    })?;
    let model_resref = validate_resref(
        &format!("{model_prefix}_cloak_{model:03}"),
        "CloakModel.modelResref",
    )?;
    let texture_resref =
        validate_resref(&format!("cloak_{texture:03}"), "CloakModel.textureResref")?;
    let icon_resref = validate_resref(
        &format!("icloak_{icon_gender}_{icon:03}"),
        "CloakModel.iconResref",
    )?;
    let available = available_resource_keys
        .iter()
        .map(|value| value.as_ref().trim().to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let required = [
        format!("2002:{model_resref}"),
        format!("6:{texture_resref}"),
        format!("6:{icon_resref}"),
    ];
    let missing = required
        .iter()
        .filter(|key| !available.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(error(
            "ITEM-CLOAK-RESOURCE-MISSING",
            "availableResourceKeys",
            format!(
                "CloakModel row {cloak_model_row} resolves to resources that are absent from the caller-bound inventory: {}",
                missing.join(", ")
            ),
        ));
    }
    Ok(ItemCloakResolutionV2 {
        schema_version: 2,
        cloak_model_row,
        model,
        texture,
        icon,
        model_resref,
        texture_resref,
        icon_resref,
        resource_verification: "RESOURCE_KEYS_VERIFIED".to_owned(),
    })
}

fn validate_item_resource_inventory_v1(
    inventory: &[ItemResourceInventoryEntryV2],
) -> Result<BTreeMap<(u16, String), ItemResourceInventoryEntryV2>, ItemErrorV1> {
    let mut validated = BTreeMap::new();
    for (index, entry) in inventory.iter().enumerate() {
        let resref = validate_resref(&entry.resref, &format!("resourceInventory[{index}].resref"))?;
        if entry.schema_version != 2
            || !matches!(entry.resource_type, 6 | 2002)
            || entry.byte_length == 0
            || entry.sha256.len() != 64
            || !entry
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || entry.provenance.schema_version != ITEM_SCHEMA_VERSION_V1
            || entry.provenance.source_kind != "NWN_BASE_KEY"
            || !entry
                .provenance
                .source_container_file_name
                .eq_ignore_ascii_case("nwn_base.key")
            || entry.provenance.source_container_sha256 != ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1
            || entry.provenance.expected_sha256 != entry.sha256
            || !valid_lowercase_sha256_v1(&entry.provenance.manifest_sha256)
            || !valid_item_resource_locator_v1(&entry.provenance.resource_locator)
            || !matches!(
                (entry.resource_type, entry.format),
                (2002, ItemReferenceResourceFormatV1::BinaryMdl)
                    | (2002, ItemReferenceResourceFormatV1::AsciiMdl)
                    | (6, ItemReferenceResourceFormatV1::PltV1)
            )
            || (entry.resource_type == 6 && entry.triangle_count != 0)
        {
            return Err(error(
                "ITEM-RESOURCE-INVENTORY-ENTRY-INVALID",
                format!("resourceInventory[{index}]"),
                "inventory entries require schemaVersion 2, a validated MDL/PLT format, pinned nwn_base.key provenance, exact payload hash and locator",
            ));
        }
        let normalized = ItemResourceInventoryEntryV2 {
            resref: resref.clone(),
            ..entry.clone()
        };
        if validated
            .insert((entry.resource_type, resref), normalized)
            .is_some()
        {
            return Err(error(
                "ITEM-RESOURCE-INVENTORY-DUPLICATE",
                format!("resourceInventory[{index}]"),
                "inventory contains the same resource type/resref more than once",
            ));
        }
    }
    Ok(validated)
}

pub fn resolve_item_cloak_v3(
    cloak_model_row: u16,
    cloak_model_2da: &[u8],
    resource_inventory: &[ItemResourceInventoryEntryV2],
) -> Result<ItemCloakResolutionV3, ItemErrorV1> {
    resolve_item_cloak_v4(cloak_model_row, "pmh0", cloak_model_2da, resource_inventory)
}

pub fn resolve_item_cloak_v4(
    cloak_model_row: u16,
    model_prefix: &str,
    cloak_model_2da: &[u8],
    resource_inventory: &[ItemResourceInventoryEntryV2],
) -> Result<ItemCloakResolutionV3, ItemErrorV1> {
    let inventory = validate_item_resource_inventory_v1(resource_inventory)?;
    let resource_keys = inventory
        .keys()
        .map(|(resource_type, resref)| format!("{resource_type}:{resref}"))
        .collect::<Vec<_>>();
    let resolved = resolve_item_cloak_for_model_prefix_v2(
        cloak_model_row,
        model_prefix,
        cloak_model_2da,
        &resource_keys,
    )?;
    let model_resource = inventory
        .get(&(2002, resolved.model_resref.clone()))
        .cloned()
        .ok_or_else(|| {
            error(
                "ITEM-CLOAK-RESOURCE-MISSING",
                "resourceInventory",
                format!("missing exact MDL bytes for {}", resolved.model_resref),
            )
        })?;
    let texture_resource = inventory
        .get(&(6, resolved.texture_resref.clone()))
        .cloned()
        .ok_or_else(|| {
            error(
                "ITEM-CLOAK-RESOURCE-MISSING",
                "resourceInventory",
                format!("missing exact PLT bytes for {}", resolved.texture_resref),
            )
        })?;
    let icon_resource = inventory
        .get(&(6, resolved.icon_resref.clone()))
        .cloned()
        .ok_or_else(|| {
            error(
                "ITEM-CLOAK-RESOURCE-MISSING",
                "resourceInventory",
                format!("missing exact PLT bytes for {}", resolved.icon_resref),
            )
        })?;
    let inventory_bytes = serde_json::to_vec(&[&model_resource, &texture_resource, &icon_resource])
        .map_err(|_| {
            error(
                "ITEM-RESOURCE-INVENTORY-SERIALIZE-FAILED",
                "resourceInventory",
                "verified cloak inventory could not be serialized",
            )
        })?;
    Ok(ItemCloakResolutionV3 {
        schema_version: 4,
        cloak_model_row,
        model: resolved.model,
        texture: resolved.texture,
        icon: resolved.icon,
        model_resref: resolved.model_resref,
        texture_resref: resolved.texture_resref,
        icon_resref: resolved.icon_resref,
        model_resource,
        texture_resource,
        icon_resource,
        inventory_sha256: item_payload_sha256_v1(&inventory_bytes),
        resource_verification: "PINNED_MANIFEST_PAYLOAD_VALIDATED".to_owned(),
    })
}

fn valid_lowercase_sha256_v1(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_item_resource_locator_v1(value: &str) -> bool {
    let mut segments = value.split(':');
    matches!(
        (
            segments.next(),
            segments.next().and_then(|value| value.parse::<u32>().ok()),
            segments.next(),
            segments.next().and_then(|value| value.parse::<u32>().ok()),
            segments.next(),
        ),
        (Some("bif"), Some(_), Some("resource"), Some(_), None)
    )
}

fn validate_item_player_model_prefix_v1(value: &str) -> Result<String, ItemErrorV1> {
    let normalized = value.trim().to_ascii_lowercase();
    let bytes = normalized.as_bytes();
    if bytes.len() != 4
        || bytes[0] != b'p'
        || !matches!(bytes[1], b'm' | b'f')
        || !bytes[2].is_ascii_alphanumeric()
        || !bytes[3].is_ascii_digit()
    {
        return Err(error(
            "ITEM-PLAYER-MODEL-PREFIX-INVALID",
            "modelPrefix",
            "player model prefix must be p + m/f + one appearance RACE token + one phenotype digit",
        ));
    }
    Ok(normalized)
}

fn reference_mdl_triangle_count_v1(report: &InspectionReport) -> Result<usize, ItemErrorV1> {
    fn visit(node: &crate::mdl::NodeReport, total: &mut usize) -> Result<(), ItemErrorV1> {
        if let Some(mesh) = &node.mesh {
            if mesh
                .index_counts
                .iter()
                .any(|count| *count > u16::MAX as u32)
            {
                return Err(error(
                    "ITEM-RESOURCE-MDL-STREAM-BOUNDARY-EXCEEDED",
                    format!("model.nodes.{}.mesh.indexCounts", node.name),
                    "one retail MDL mesh stream exceeds the 65,535 index-entry binary boundary",
                ));
            }
            if mesh.render != 0 {
                *total = total.checked_add(mesh.faces.len()).ok_or_else(|| {
                    error(
                        "ITEM-TRIANGLE-COUNT-OVERFLOW",
                        "referenceResource.model",
                        "reference MDL render triangle count overflowed this platform",
                    )
                })?;
            }
        }
        for child in &node.children {
            visit(child, total)?;
        }
        Ok(())
    }

    let mut total = 0usize;
    for root in &report.node_tree.roots {
        visit(root, &mut total)?;
    }
    Ok(total)
}

fn inspect_reference_plt_v1(bytes: &[u8]) -> Result<(), ItemErrorV1> {
    const HEADER_LENGTH: usize = 24;
    if bytes.len() < HEADER_LENGTH || &bytes[..8] != b"PLT V1  " {
        return Err(error(
            "ITEM-RESOURCE-PLT-INVALID",
            "referenceResource.bytes",
            "retail PLT payload must start with the 24-byte PLT V1 header",
        ));
    }
    let read_u32 = |offset: usize| {
        u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("validated PLT header"),
        )
    };
    let palette_count = read_u32(8);
    let width = read_u32(16);
    let height = read_u32(20);
    let expected_length = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(2))
        .and_then(|payload| payload.checked_add(HEADER_LENGTH as u64));
    if !(1..=10).contains(&palette_count)
        || read_u32(12) != 0
        || width == 0
        || height == 0
        || expected_length != Some(bytes.len() as u64)
        || bytes[HEADER_LENGTH..]
            .chunks_exact(2)
            .any(|pixel| u32::from(pixel[1]) >= palette_count)
    {
        return Err(error(
            "ITEM-RESOURCE-PLT-INVALID",
            "referenceResource.bytes",
            "retail PLT payload has invalid dimensions, palette count, reserved field, palette index or byte length",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct ItemAsciiMdlNodeV1 {
    render: bool,
    vertex_count: Option<usize>,
    face_count: Option<usize>,
    face_indices: Vec<[usize; 3]>,
}

fn reference_ascii_mdl_error_v1(message: impl Into<String>) -> ItemErrorV1 {
    error(
        "ITEM-RESOURCE-MDL-INVALID",
        "referenceResource.bytes",
        message,
    )
}

fn inspect_reference_ascii_mdl_v1(
    bytes: &[u8],
    expected_resref: &str,
) -> Result<usize, ItemErrorV1> {
    if bytes.iter().any(|byte| *byte == 0 || !byte.is_ascii()) {
        return Err(reference_ascii_mdl_error_v1(
            "ASCII MDL must contain only non-NUL ASCII bytes",
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|source| {
        reference_ascii_mdl_error_v1(format!("ASCII MDL is not valid UTF-8/ASCII: {source}"))
    })?;
    let lines = text.lines().collect::<Vec<_>>();
    let mut saw_header = false;
    let mut declared_model: Option<String> = None;
    let mut saw_begin_geometry = false;
    let mut saw_end_geometry = false;
    let mut node: Option<ItemAsciiMdlNodeV1> = None;
    let mut node_count = 0usize;
    let mut geometry_node_count = 0usize;
    let mut triangle_count = 0usize;
    let mut line_index = 0usize;

    while line_index < lines.len() {
        let line = lines[line_index].trim();
        line_index += 1;
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            if line.eq_ignore_ascii_case("#NWmax MODEL ASCII")
                || line.eq_ignore_ascii_case("#MAXMODEL ASCII")
            {
                saw_header = true;
            }
            continue;
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let keyword = tokens[0].to_ascii_lowercase();
        match keyword.as_str() {
            "newmodel" => {
                if node.is_some() || tokens.len() != 2 || declared_model.is_some() {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: expected exactly one `newmodel <resref>` declaration"
                    )));
                }
                declared_model = Some(tokens[1].to_ascii_lowercase());
            }
            "beginmodelgeom" => {
                if node.is_some()
                    || tokens.len() != 2
                    || saw_begin_geometry
                    || !tokens[1].eq_ignore_ascii_case(expected_resref)
                {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: invalid or duplicate beginmodelgeom identity"
                    )));
                }
                saw_begin_geometry = true;
            }
            "endmodelgeom" => {
                if node.is_some()
                    || !saw_begin_geometry
                    || saw_end_geometry
                    || !matches!(tokens.len(), 1 | 2)
                    || (tokens.len() == 2 && !tokens[1].eq_ignore_ascii_case(expected_resref))
                {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: invalid endmodelgeom boundary"
                    )));
                }
                saw_end_geometry = true;
            }
            "node" => {
                if node.is_some() || !saw_begin_geometry || saw_end_geometry || tokens.len() != 3 {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: invalid or nested node declaration"
                    )));
                }
                node_count = node_count.checked_add(1).ok_or_else(|| {
                    reference_ascii_mdl_error_v1("ASCII MDL node count overflowed")
                })?;
                node = Some(ItemAsciiMdlNodeV1 {
                    render: true,
                    ..ItemAsciiMdlNodeV1::default()
                });
            }
            "endnode" => {
                if tokens.len() != 1 {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: endnode must not have arguments"
                    )));
                }
                let finished = node.take().ok_or_else(|| {
                    reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: endnode has no matching node"
                    ))
                })?;
                if let Some(face_count) = finished.face_count {
                    let vertex_count = finished.vertex_count.ok_or_else(|| {
                        reference_ascii_mdl_error_v1(format!(
                            "line {line_index}: a node with faces has no verts section"
                        ))
                    })?;
                    if finished
                        .face_indices
                        .iter()
                        .flatten()
                        .any(|index| *index >= vertex_count)
                    {
                        return Err(reference_ascii_mdl_error_v1(format!(
                            "line {line_index}: face index exceeds the node vertex count"
                        )));
                    }
                    geometry_node_count = geometry_node_count.checked_add(1).ok_or_else(|| {
                        reference_ascii_mdl_error_v1("ASCII MDL geometry node count overflowed")
                    })?;
                    if finished.render {
                        triangle_count =
                            triangle_count.checked_add(face_count).ok_or_else(|| {
                                reference_ascii_mdl_error_v1(
                                    "ASCII MDL render triangle count overflowed",
                                )
                            })?;
                    }
                }
            }
            "render" => {
                let current = node.as_mut().ok_or_else(|| {
                    reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: render appears outside a node"
                    ))
                })?;
                if tokens.len() != 2 || !matches!(tokens[1], "0" | "1") {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: render must be 0 or 1"
                    )));
                }
                current.render = tokens[1] == "1";
            }
            "verts" => {
                let current = node.as_mut().ok_or_else(|| {
                    reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: verts appears outside a node"
                    ))
                })?;
                if tokens.len() != 2 || current.vertex_count.is_some() {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: invalid or duplicate verts section"
                    )));
                }
                let count = tokens[1].parse::<usize>().map_err(|_| {
                    reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: verts count is not an unsigned integer"
                    ))
                })?;
                if count == 0 || count > lines.len().saturating_sub(line_index) {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: verts count exceeds the remaining payload"
                    )));
                }
                for vertex_offset in 0..count {
                    let vertex_line = lines[line_index + vertex_offset].trim();
                    let values = vertex_line.split_whitespace().collect::<Vec<_>>();
                    if values.len() < 3
                        || values[..3].iter().any(|value| {
                            value
                                .parse::<f32>()
                                .map(|value| !value.is_finite())
                                .unwrap_or(true)
                        })
                    {
                        return Err(reference_ascii_mdl_error_v1(format!(
                            "line {}: vertex must start with three finite numbers",
                            line_index + vertex_offset + 1
                        )));
                    }
                }
                line_index += count;
                current.vertex_count = Some(count);
            }
            "faces" => {
                let current = node.as_mut().ok_or_else(|| {
                    reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: faces appears outside a node"
                    ))
                })?;
                if tokens.len() != 2 || current.face_count.is_some() {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: invalid or duplicate faces section"
                    )));
                }
                let count = tokens[1].parse::<usize>().map_err(|_| {
                    reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: faces count is not an unsigned integer"
                    ))
                })?;
                if count == 0 || count > lines.len().saturating_sub(line_index) {
                    return Err(reference_ascii_mdl_error_v1(format!(
                        "line {line_index}: faces count exceeds the remaining payload"
                    )));
                }
                let mut indices = Vec::with_capacity(count);
                for face_offset in 0..count {
                    let face_line = lines[line_index + face_offset].trim();
                    let values = face_line.split_whitespace().collect::<Vec<_>>();
                    if values.len() < 3 {
                        return Err(reference_ascii_mdl_error_v1(format!(
                            "line {}: face must start with three vertex indices",
                            line_index + face_offset + 1
                        )));
                    }
                    let mut face = [0usize; 3];
                    for (target, value) in face.iter_mut().zip(&values[..3]) {
                        *target = value.parse::<usize>().map_err(|_| {
                            reference_ascii_mdl_error_v1(format!(
                                "line {}: face vertex index is not an unsigned integer",
                                line_index + face_offset + 1
                            ))
                        })?;
                    }
                    indices.push(face);
                }
                line_index += count;
                current.face_count = Some(count);
                current.face_indices = indices;
            }
            _ => {}
        }
    }

    if node.is_some() {
        return Err(reference_ascii_mdl_error_v1(
            "ASCII MDL ended before the current node's endnode",
        ));
    }
    let declared_model = declared_model.ok_or_else(|| {
        reference_ascii_mdl_error_v1("ASCII MDL is missing its newmodel declaration")
    })?;
    if !declared_model.eq_ignore_ascii_case(expected_resref) {
        return Err(error(
            "ITEM-RESOURCE-MDL-IDENTITY-MISMATCH",
            "referenceResource.resref",
            format!(
                "ASCII MDL declares model {declared_model:?}, not requested resref {expected_resref:?}"
            ),
        ));
    }
    if !saw_header
        || !saw_begin_geometry
        || !saw_end_geometry
        || node_count == 0
        || geometry_node_count == 0
        || triangle_count == 0
    {
        return Err(reference_ascii_mdl_error_v1(
            "ASCII MDL requires its format header, geometry envelope, nodes and non-empty render geometry",
        ));
    }
    Ok(triangle_count)
}

pub fn inspect_item_reference_resource_v1(
    resource_type: u16,
    resref: &str,
    bytes: &[u8],
    provenance: &ItemResourceProvenanceV1,
) -> Result<ItemResourceInventoryEntryV2, ItemErrorV1> {
    let resref = validate_resref(resref, "referenceResource.resref")?;
    if provenance.schema_version != ITEM_SCHEMA_VERSION_V1
        || provenance.source_kind != "NWN_BASE_KEY"
        || !provenance
            .source_container_file_name
            .eq_ignore_ascii_case("nwn_base.key")
        || provenance.source_container_sha256 != ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1
        || !valid_lowercase_sha256_v1(&provenance.expected_sha256)
        || !valid_lowercase_sha256_v1(&provenance.manifest_sha256)
        || !valid_item_resource_locator_v1(&provenance.resource_locator)
    {
        return Err(error(
            "ITEM-RESOURCE-PROVENANCE-INVALID",
            "referenceResource.provenance",
            "reference resources require a pinned nwn_base.key hash, exact payload hash and bif:<index>:resource:<index> locator",
        ));
    }
    if bytes.is_empty() {
        return Err(error(
            "ITEM-RESOURCE-PAYLOAD-EMPTY",
            "referenceResource.bytes",
            "reference resource payload must not be empty",
        ));
    }
    let actual_sha256 = item_payload_sha256_v1(bytes);
    if actual_sha256 != provenance.expected_sha256 {
        return Err(error(
            "ITEM-RESOURCE-PAYLOAD-HASH-MISMATCH",
            "referenceResource.bytes",
            format!(
                "payload SHA-256 {actual_sha256} differs from the authoritative manifest hash {}",
                provenance.expected_sha256
            ),
        ));
    }
    let (format, triangle_count) = match resource_type {
        2002 => {
            if bytes.get(..4) == Some(&[0, 0, 0, 0]) {
                let report = inspect_binary_mdl(bytes).map_err(|source| {
                    error(
                        "ITEM-RESOURCE-MDL-INVALID",
                        "referenceResource.bytes",
                        source.to_string(),
                    )
                })?;
                if !report.model.name.eq_ignore_ascii_case(&resref) {
                    return Err(error(
                        "ITEM-RESOURCE-MDL-IDENTITY-MISMATCH",
                        "referenceResource.resref",
                        format!(
                            "binary MDL declares model {:?}, not requested resref {resref:?}",
                            report.model.name
                        ),
                    ));
                }
                (
                    ItemReferenceResourceFormatV1::BinaryMdl,
                    reference_mdl_triangle_count_v1(&report)?,
                )
            } else {
                (
                    ItemReferenceResourceFormatV1::AsciiMdl,
                    inspect_reference_ascii_mdl_v1(bytes, &resref)?,
                )
            }
        }
        6 => {
            inspect_reference_plt_v1(bytes)?;
            (ItemReferenceResourceFormatV1::PltV1, 0)
        }
        _ => {
            return Err(error(
                "ITEM-RESOURCE-TYPE-UNSUPPORTED",
                "referenceResource.resourceType",
                "retail Item resource inventory accepts binary/ASCII MDL type 2002 and PLT type 6",
            ));
        }
    };
    Ok(ItemResourceInventoryEntryV2 {
        schema_version: 2,
        resource_type,
        resref,
        byte_length: bytes.len() as u64,
        sha256: actual_sha256,
        format,
        triangle_count,
        provenance: provenance.clone(),
    })
}

pub fn resolve_item_equipped_appearance_v1(
    appearance_2da: &[u8],
    appearance_row: u16,
    racial_type: u8,
    gender: u8,
    phenotype: u8,
) -> Result<ItemEquippedAppearanceBindingV1, ItemErrorV1> {
    let row = u32::from(appearance_row);
    let (columns, cells) = read_reference_row_v1("APPEARANCE", appearance_2da, row)?;
    let model_type = two_da_cell_by_column_v1("APPEARANCE", &columns, &cells, row, "MODELTYPE")?;
    if model_type != "P" {
        return Err(error(
            "ITEM-EQUIPPED-APPEARANCE-MODELTYPE-INVALID",
            format!("APPEARANCE.rows.{row}.MODELTYPE"),
            "equipped CAPART/Cloak fixture requires a part-based MODELTYPE=P appearance",
        ));
    }
    let race_token = two_da_cell_by_column_v1("APPEARANCE", &columns, &cells, row, "RACE")?;
    if race_token.len() != 1 || !race_token.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err(error(
            "ITEM-EQUIPPED-APPEARANCE-RACE-INVALID",
            format!("APPEARANCE.rows.{row}.RACE"),
            "part-based appearance RACE must be one ASCII model-prefix token",
        ));
    }
    let table_racial_type = u8::try_from(parse_u32(
        two_da_cell_by_column_v1("APPEARANCE", &columns, &cells, row, "RACIALTYPE")?,
        row,
        "RACIALTYPE",
    )?)
    .map_err(|_| {
        error(
            "ITEM-EQUIPPED-APPEARANCE-RACIALTYPE-INVALID",
            format!("APPEARANCE.rows.{row}.RACIALTYPE"),
            "appearance RACIALTYPE must fit a GFF BYTE",
        )
    })?;
    if table_racial_type != racial_type {
        return Err(error(
            "ITEM-EQUIPPED-APPEARANCE-RACIALTYPE-MISMATCH",
            format!("APPEARANCE.rows.{row}.RACIALTYPE"),
            format!(
                "fixture Race {racial_type} differs from appearance RACIALTYPE {table_racial_type}"
            ),
        ));
    }
    let gender_token = match gender {
        0 => 'm',
        1 => 'f',
        _ => {
            return Err(error(
                "ITEM-EQUIPPED-APPEARANCE-GENDER-INVALID",
                "equippedProofContext.gender",
                "player-model fixture gender must be 0 (male) or 1 (female)",
            ));
        }
    };
    if phenotype > 9 {
        return Err(error(
            "ITEM-EQUIPPED-APPEARANCE-PHENOTYPE-INVALID",
            "equippedProofContext.phenotype",
            "player-model fixture phenotype must be a single decimal digit",
        ));
    }
    let model_prefix = validate_item_player_model_prefix_v1(&format!(
        "p{gender_token}{}{phenotype}",
        race_token.to_ascii_lowercase()
    ))?;
    Ok(ItemEquippedAppearanceBindingV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        appearance_row,
        model_type: model_type.to_owned(),
        race_token: race_token.to_ascii_lowercase(),
        racial_type,
        gender,
        phenotype,
        model_prefix,
        appearance_table_sha256: item_payload_sha256_v1(appearance_2da),
    })
}

fn validate_resref(value: &str, path: &str) -> Result<String, ItemErrorV1> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.len() > 16
        || !normalized
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "ITEM-RESREF-INVALID",
            path,
            "resref must contain 1..16 lowercase ASCII letters, digits, or underscores",
        ));
    }
    Ok(normalized)
}

/// Encodes Aurora's ModelType 2 part selector. The decimal tens are the model
/// number and the ones digit is one of the four native weapon color variants.
pub fn encode_item_weapon_part_appearance_v1(model: u8, color: u8) -> Result<u8, ItemErrorV1> {
    if !(ITEM_WEAPON_COLOR_MIN_V1..=ITEM_WEAPON_COLOR_MAX_V1).contains(&color) {
        return Err(error(
            "ITEM-WEAPON-COLOR-INVALID",
            "color",
            format!(
                "ModelType 2 weapon color must be {}..{}",
                ITEM_WEAPON_COLOR_MIN_V1, ITEM_WEAPON_COLOR_MAX_V1
            ),
        ));
    }
    let encoded = u16::from(model) * 10 + u16::from(color);
    u8::try_from(encoded).map_err(|_| {
        error(
            "ITEM-WEAPON-MODEL-OVERFLOW",
            "model",
            format!(
                "ModelType 2 weapon model must encode with color into one BYTE; maximum model is {}",
                ITEM_WEAPON_MODEL_MAX_V1
            ),
        )
    })
}

/// Decodes and validates one Aurora ModelType 2 part selector. Values whose
/// ones digit is 0 or 5..9 name no native weapon color and are rejected.
pub fn decode_item_weapon_part_appearance_v1(
    encoded_value: u8,
) -> Result<ItemWeaponPartAppearanceV1, ItemErrorV1> {
    let model = encoded_value / 10;
    let color = encoded_value % 10;
    let canonical = encode_item_weapon_part_appearance_v1(model, color)?;
    Ok(ItemWeaponPartAppearanceV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        model,
        color,
        encoded_value: canonical,
    })
}

pub fn resolve_item_part_resource_v1(
    base_item: &ItemBaseItemV1,
    field: &str,
    variant: u8,
    explicit_model_resref: Option<&str>,
    explicit_icon_resref: Option<&str>,
) -> Result<ItemResolvedResourceV1, ItemErrorV1> {
    if base_item.model_type == 2 {
        decode_item_weapon_part_appearance_v1(variant)?;
    }
    let slot = base_item
        .part_slots
        .iter()
        .find(|slot| slot.field.eq_ignore_ascii_case(field))
        .ok_or_else(|| {
            error(
                "ITEM-PART-SLOT-INVALID",
                "field",
                format!("{field} is not part of ModelType {}", base_item.model_type),
            )
        })?;
    if slot.source_kind != ItemPartSourceKindV1::MeshyGlb {
        return Err(error(
            "ITEM-REFERENCE-COMPOSER-REQUIRED",
            format!("parts.{}", slot.field),
            match slot.source_kind {
                ItemPartSourceKindV1::CapartSelection => {
                    "CAPART armor fields are numeric UTI selectors resolved through CAPART and their exact parts_* tables; they are not independent MDL/icon resrefs"
                }
                ItemPartSourceKindV1::CloakModelSelection => {
                    "cloak ModelPart1 is a numeric CloakModel row selector; it is not an independent MDL/icon resref"
                }
                ItemPartSourceKindV1::MeshyGlb => unreachable!(),
            },
        ));
    }
    let (model_resref, icon_resref) = if slot.requires_explicit_resource_resrefs {
        let model = explicit_model_resref.ok_or_else(|| {
            error(
                "ITEM-RESOURCE-CONTEXT-REQUIRED",
                format!("parts.{}.modelResref", slot.field),
                "this item profile needs a caller-provided, Aurora-validated model resref",
            )
        })?;
        let icon = explicit_icon_resref.ok_or_else(|| {
            error(
                "ITEM-RESOURCE-CONTEXT-REQUIRED",
                format!("parts.{}.iconResref", slot.field),
                "this item profile needs a caller-provided, Aurora-validated icon resref",
            )
        })?;
        (
            validate_resref(model, "modelResref")?,
            validate_resref(icon, "iconResref")?,
        )
    } else {
        let item_class = base_item.item_class.to_ascii_lowercase();
        let suffix = slot
            .token
            .as_deref()
            .map(|token| format!("_{token}_{variant:03}"))
            .unwrap_or_else(|| format!("_{variant:03}"));
        (
            validate_resref(&format!("{item_class}{suffix}"), "modelResref")?,
            validate_resref(&format!("i{item_class}{suffix}"), "iconResref")?,
        )
    };
    Ok(ItemResolvedResourceV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        base_item: base_item.base_item,
        field: slot.field.clone(),
        variant,
        model_resref,
        icon_resref,
    })
}

fn field(label: &str, value: GffValueV1) -> GffFieldV1 {
    GffFieldV1 {
        label: label.to_owned(),
        value,
    }
}

fn loc_string(value: &str) -> GffValueV1 {
    GffValueV1::LocString(GffLocStringV1 {
        string_ref: u32::MAX,
        substrings: if value.is_empty() {
            Vec::new()
        } else {
            vec![GffLocSubstringV1 {
                string_id: 0,
                bytes: value.as_bytes().to_vec(),
            }]
        },
    })
}

fn color_values(colors: &ItemColorValuesV1) -> [u8; 6] {
    [
        colors.leather1_color.unwrap_or(0),
        colors.leather2_color.unwrap_or(0),
        colors.cloth1_color.unwrap_or(0),
        colors.cloth2_color.unwrap_or(0),
        colors.metal1_color.unwrap_or(0),
        colors.metal2_color.unwrap_or(0),
    ]
}

fn item_property_struct_v1(
    index: usize,
    property: &ItemPropertyV1,
) -> Result<GffStructV1, ItemErrorV1> {
    if property.chance_appear > 100 {
        return Err(error(
            "ITEM-UTI-PROPERTY-CHANCE-INVALID",
            format!("blueprint.properties[{index}].chanceAppear"),
            "ChanceAppear must be in the range 0..100",
        ));
    }
    let cost_table = if property.property_name == ITEM_PROPERTY_ON_HIT_CAST_SPELL_V1 {
        // Aurora normalizes PropertyName 0x30 to CostTable 0x18 while reading
        // the UTI. Emitting that canonical value prevents a write/read semantic
        // drift and matches the native property contract.
        0x18
    } else {
        property.cost_table
    };
    Ok(GffStructV1 {
        struct_id: index as u32,
        fields: vec![
            field("PropertyName", GffValueV1::Word(property.property_name)),
            field("Subtype", GffValueV1::Word(property.subtype)),
            field("CostTable", GffValueV1::Byte(cost_table)),
            field("CostValue", GffValueV1::Word(property.cost_value)),
            field("Param1", GffValueV1::Byte(property.param1)),
            field("Param1Value", GffValueV1::Byte(property.param1_value)),
            field("ChanceAppear", GffValueV1::Byte(property.chance_appear)),
        ],
    })
}

pub fn write_item_uti_v1(
    base_item: &ItemBaseItemV1,
    blueprint: &ItemBlueprintV1,
) -> Result<ItemUtiArtifactV1, ItemErrorV1> {
    if blueprint.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-UTI-SCHEMA-INVALID",
            "blueprint.schemaVersion",
            "item blueprint schemaVersion must be 1",
        ));
    }
    let template_resref = validate_resref(&blueprint.template_resref, "templateResref")?;
    let expected = base_item
        .part_slots
        .iter()
        .map(|slot| slot.field.as_str())
        .collect::<BTreeSet<_>>();
    let mut values = BTreeMap::new();
    for part in &blueprint.parts {
        let expected_field = expected
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(&part.field))
            .copied()
            .ok_or_else(|| {
                error(
                    "ITEM-UTI-PART-UNEXPECTED",
                    format!("blueprint.parts.{}", part.field),
                    format!(
                        "{} is not a part field for ModelType {}",
                        part.field, base_item.model_type
                    ),
                )
            })?;
        if values.insert(expected_field, part.value).is_some() {
            return Err(error(
                "ITEM-UTI-PART-DUPLICATE",
                format!("blueprint.parts.{}", part.field),
                "each resolved part field must appear exactly once",
            ));
        }
    }
    if values.len() != expected.len() {
        let missing = expected
            .iter()
            .filter(|field| !values.contains_key(**field))
            .copied()
            .collect::<Vec<_>>();
        return Err(error(
            "ITEM-UTI-PARTS-INCOMPLETE",
            "blueprint.parts",
            format!("missing required part fields: {}", missing.join(", ")),
        ));
    }
    if base_item.model_type == 2 {
        for (field, value) in &values {
            decode_item_weapon_part_appearance_v1(*value).map_err(|source| {
                error(
                    &source.code,
                    format!("blueprint.parts.{field}"),
                    source.message,
                )
            })?;
        }
    }
    let colors = [
        blueprint.colors.leather1_color,
        blueprint.colors.leather2_color,
        blueprint.colors.cloth1_color,
        blueprint.colors.cloth2_color,
        blueprint.colors.metal1_color,
        blueprint.colors.metal2_color,
    ];
    if base_item.color_fields.is_empty() && colors.iter().any(Option::is_some) {
        return Err(error(
            "ITEM-UTI-COLORS-UNEXPECTED",
            "blueprint.colors",
            format!(
                "ModelType {} does not serialize item color fields",
                base_item.model_type
            ),
        ));
    }
    if !base_item.color_fields.is_empty() && colors.iter().any(Option::is_none) {
        return Err(error(
            "ITEM-UTI-COLORS-INCOMPLETE",
            "blueprint.colors",
            format!(
                "ModelType {} requires all six numeric item color fields",
                base_item.model_type
            ),
        ));
    }

    let mut fields = vec![
        field(
            "TemplateResRef",
            GffValueV1::ResRef(template_resref.clone()),
        ),
        field("Tag", GffValueV1::String(blueprint.tag.as_bytes().to_vec())),
        field("LocalizedName", loc_string(&blueprint.localized_name)),
        field("Description", loc_string(&blueprint.description)),
        field(
            "DescIdentified",
            loc_string(&blueprint.identified_description),
        ),
        field(
            "Comment",
            GffValueV1::String(blueprint.comment.as_bytes().to_vec()),
        ),
        field(
            "BaseItem",
            GffValueV1::Int(i32::try_from(base_item.base_item).map_err(|_| {
                error(
                    "ITEM-UTI-BASEITEM-OVERFLOW",
                    "baseItem",
                    "UTI BaseItem must fit the native signed INT field",
                )
            })?),
        ),
    ];
    for slot in &base_item.part_slots {
        fields.push(field(
            &slot.field,
            GffValueV1::Byte(values[slot.field.as_str()]),
        ));
    }
    if !base_item.color_fields.is_empty() {
        for (label, value) in ITEM_COLOR_FIELDS_V1
            .into_iter()
            .zip(color_values(&blueprint.colors))
        {
            fields.push(field(label, GffValueV1::Byte(value)));
        }
    }
    let properties = blueprint
        .properties
        .iter()
        .enumerate()
        .map(|(index, property)| item_property_struct_v1(index, property))
        .collect::<Result<Vec<_>, _>>()?;
    fields.extend([
        field("Cost", GffValueV1::Dword(blueprint.cost)),
        field("AddCost", GffValueV1::Dword(blueprint.add_cost)),
        field("Charges", GffValueV1::Byte(blueprint.charges)),
        field("StackSize", GffValueV1::Word(blueprint.stack_size)),
        field("PaletteID", GffValueV1::Byte(blueprint.palette_id)),
        field(
            "Identified",
            GffValueV1::Byte(u8::from(blueprint.identified)),
        ),
        field("Stolen", GffValueV1::Byte(u8::from(blueprint.stolen))),
        field("Cursed", GffValueV1::Byte(u8::from(blueprint.cursed))),
        field("Plot", GffValueV1::Byte(u8::from(blueprint.plot))),
        field("PropertiesList", GffValueV1::List(properties)),
    ]);

    let artifact = write_gff_v32(
        &GffDocumentV1 {
            schema_version: GFF_SCHEMA_VERSION,
            file_type: GffFileTypeV1::Uti,
            root: GffStructV1 {
                struct_id: u32::MAX,
                fields,
            },
        },
        &GffWriterOptionsV1::default(),
    )
    .map_err(|source| error("ITEM-UTI-WRITE-FAILED", source.path, source.message))?;

    Ok(ItemUtiArtifactV1 {
        payload: artifact.payload,
        report: ItemUtiReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            template_resref,
            base_item: base_item.base_item,
            model_type: base_item.model_type,
            part_count: base_item.part_slots.len() as u32,
            color_field_count: base_item.color_fields.len() as u32,
            weapon_color_selector_count: if base_item.model_type == 2 {
                base_item.part_slots.len() as u32
            } else {
                0
            },
            byte_length: artifact.report.byte_length,
            output_sha256: artifact.report.output_sha256,
            semantic_readback_status: artifact.report.semantic_readback_status,
        },
    })
}

pub fn item_payload_sha256_v1(bytes: &[u8]) -> String {
    hex_lower(&Sha256::digest(bytes))
}

fn item_resource_provider_kind_name_v1(kind: ItemResourceProviderKindV1) -> &'static str {
    match kind {
        ItemResourceProviderKindV1::Vanilla => "VANILLA",
        ItemResourceProviderKindV1::Hak => "HAK",
        ItemResourceProviderKindV1::Module => "MODULE",
    }
}

fn item_erf_file_type_name_v1(file_type: ErfFileType) -> &'static str {
    match file_type {
        ErfFileType::Erf => "ERF",
        ErfFileType::Hak => "HAK",
        ErfFileType::Module => "MOD",
    }
}

/// Validates and hashes an ordered Item resource context without writing to
/// any source container. A context is accepted only when every provider is an
/// independently readable ERF-family archive with a stable identity.
pub fn inspect_item_resource_context_v1(
    context: &ItemResourceContextInputV1<'_>,
) -> Result<ItemResourceContextReportV1, ItemErrorV1> {
    if context.schema_version != ITEM_SCHEMA_VERSION_V1
        || context.resolution_policy != "SINGLE_CUSTOM_PROVIDER_OVER_VANILLA_V1"
        || context.providers.is_empty()
    {
        return Err(error(
            "ITEM-RESOURCE-CONTEXT-INVALID",
            "context",
            "Item resource context requires schema 1, the fail-closed single-custom policy and at least one provider",
        ));
    }
    let mut provider_ids = BTreeSet::new();
    let mut order_keys = BTreeSet::new();
    let mut providers = Vec::with_capacity(context.providers.len());
    for (index, provider) in context.providers.iter().enumerate() {
        if provider.provider_id.trim().is_empty()
            || provider.file_name.trim().is_empty()
            || !provider_ids.insert(provider.provider_id.clone())
            || !order_keys.insert((
                item_resource_provider_kind_name_v1(provider.kind),
                provider.order,
            ))
        {
            return Err(error(
                "ITEM-RESOURCE-CONTEXT-PROVIDER-INVALID",
                format!("context.providers[{index}]"),
                "provider id, file name and kind-local order must be non-empty and unique",
            ));
        }
        let archive = ErfArchive::parse(provider.bytes).map_err(|source| {
            error(
                "ITEM-RESOURCE-CONTEXT-CONTAINER-INVALID",
                format!("context.providers[{index}].bytes"),
                format!(
                    "{} at byte {}: {}",
                    source.code, source.offset, source.context
                ),
            )
        })?;
        let declared_container_matches = match provider.kind {
            // Tests and pre-indexed retail inventories may expose Vanilla as
            // an immutable ERF-family fixture. Its provenance, not its ERF
            // signature, is what makes it Vanilla.
            ItemResourceProviderKindV1::Vanilla => true,
            ItemResourceProviderKindV1::Hak => archive.file_type() == ErfFileType::Hak,
            ItemResourceProviderKindV1::Module => archive.file_type() == ErfFileType::Module,
        };
        if !declared_container_matches {
            return Err(error(
                "ITEM-RESOURCE-CONTEXT-KIND-MISMATCH",
                format!("context.providers[{index}].kind"),
                "declared custom provider kind does not match the container signature",
            ));
        }
        providers.push(ItemResourceProviderReportV1 {
            provider_id: provider.provider_id.clone(),
            kind: provider.kind,
            order: provider.order,
            file_name: provider.file_name.clone(),
            container_file_type: item_erf_file_type_name_v1(archive.file_type()).to_owned(),
            byte_length: provider.bytes.len(),
            sha256: item_payload_sha256_v1(provider.bytes),
            resource_count: archive.resources().len(),
        });
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ContextIdentity<'a> {
        schema_version: u32,
        resolution_policy: &'a str,
        providers: &'a [ItemResourceProviderReportV1],
    }
    let identity = serde_json::to_vec(&ContextIdentity {
        schema_version: context.schema_version,
        resolution_policy: &context.resolution_policy,
        providers: &providers,
    })
    .map_err(|_| {
        error(
            "ITEM-RESOURCE-CONTEXT-SERIALIZE-FAILED",
            "context",
            "resource context identity could not be serialized",
        )
    })?;
    Ok(ItemResourceContextReportV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        resolution_policy: context.resolution_policy.clone(),
        status: "PASSED".to_owned(),
        providers,
        context_sha256: item_payload_sha256_v1(&identity),
    })
}

/// Resolves one exact resource. One custom HAK or MOD may shadow Vanilla; two
/// custom matches are rejected because Aurora's multi-container collision
/// priority has not been accepted as a product invariant.
pub fn resolve_item_resource_from_context_v1<'a>(
    context: &'a ItemResourceContextInputV1<'a>,
    resref: &str,
    resource_type: u16,
) -> Result<ItemResolvedContextResourceV1<'a>, ItemErrorV1> {
    validate_resref(resref, "resref")?;
    let report = inspect_item_resource_context_v1(context)?;
    let mut vanilla_matches = Vec::new();
    let mut custom_matches = Vec::new();
    for (index, provider) in context.providers.iter().enumerate() {
        let archive = ErfArchive::parse(provider.bytes).map_err(|source| {
            error(
                "ITEM-RESOURCE-CONTEXT-CONTAINER-INVALID",
                format!("context.providers[{index}].bytes"),
                format!(
                    "{} at byte {}: {}",
                    source.code, source.offset, source.context
                ),
            )
        })?;
        match archive.find(resref, resource_type) {
            Ok(payload) => {
                let binding = (index, provider, payload);
                if provider.kind == ItemResourceProviderKindV1::Vanilla {
                    vanilla_matches.push(binding);
                } else {
                    custom_matches.push(binding);
                }
            }
            Err(source) if source.code == crate::erf::RESOURCE_MISSING => {}
            Err(source) => {
                return Err(error(
                    "ITEM-RESOURCE-CONTEXT-LOOKUP-FAILED",
                    format!("context.providers[{index}]"),
                    format!(
                        "{} at byte {}: {}",
                        source.code, source.offset, source.context
                    ),
                ));
            }
        }
    }
    if custom_matches.len() > 1 {
        return Err(error(
            "ITEM-RESOURCE-CONTEXT-AMBIGUOUS-CUSTOM-RESOURCE",
            format!("resources.{resref}.{resource_type}"),
            format!(
                "resource occurs in multiple custom providers: {}",
                custom_matches
                    .iter()
                    .map(|(_, provider, _)| provider.provider_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    if custom_matches.is_empty() && vanilla_matches.len() > 1 {
        return Err(error(
            "ITEM-RESOURCE-CONTEXT-AMBIGUOUS-VANILLA-RESOURCE",
            format!("resources.{resref}.{resource_type}"),
            "resource occurs in multiple Vanilla providers",
        ));
    }
    let (winning_index, winning_provider, payload) = custom_matches
        .first()
        .copied()
        .or_else(|| vanilla_matches.first().copied())
        .ok_or_else(|| {
            error(
                "ITEM-RESOURCE-CONTEXT-RESOURCE-MISSING",
                format!("resources.{resref}.{resource_type}"),
                "resource was not found in the selected Item resource context",
            )
        })?;
    let shadowed_provider_ids = vanilla_matches
        .iter()
        .filter(|(index, _, _)| *index != winning_index)
        .map(|(_, provider, _)| provider.provider_id.clone())
        .collect();
    Ok(ItemResolvedContextResourceV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        resref: resref.to_ascii_lowercase(),
        resource_type,
        winning_provider_id: winning_provider.provider_id.clone(),
        shadowed_provider_ids,
        payload,
        payload_sha256: item_payload_sha256_v1(payload),
        context_sha256: report.context_sha256,
    })
}

pub fn select_effective_item_baseitem_v1(
    context: &ItemResourceContextInputV1<'_>,
    base_item: u32,
) -> Result<ItemEffectiveBaseItemsSelectionV1, ItemErrorV1> {
    let resolved = resolve_item_resource_from_context_v1(context, "baseitems", 2017)?;
    let selected = resolve_item_baseitem_v1(resolved.payload, base_item)?;
    let mut selection = ItemEffectiveBaseItemsSelectionV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        status: "PASSED".to_owned(),
        resource_context_sha256: resolved.context_sha256,
        baseitems_sha256: resolved.payload_sha256,
        winning_provider_id: resolved.winning_provider_id,
        shadowed_provider_ids: resolved.shadowed_provider_ids,
        selected,
        selection_sha256: String::new(),
    };
    let bytes = serde_json::to_vec(&selection).map_err(|_| {
        error(
            "ITEM-BASEITEM-SELECTION-SERIALIZE-FAILED",
            "selection",
            "effective BaseItem selection could not be serialized",
        )
    })?;
    selection.selection_sha256 = item_payload_sha256_v1(&bytes);
    Ok(selection)
}

fn item_proof_map_error_v1(
    code: &str,
    path: &str,
    source: crate::proof_module::ProofModuleErrorV1,
) -> ItemErrorV1 {
    error(code, format!("{path}.{}", source.path), source.message)
}

fn item_proof_resource_v1(
    resref: &str,
    resource_type: u16,
    payload: Vec<u8>,
) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

fn replace_gff_list_v1(
    document: &mut GffDocumentV1,
    label: &str,
    values: Vec<GffStructV1>,
) -> Result<(), ItemErrorV1> {
    let field = document
        .root
        .fields
        .iter_mut()
        .find(|field| field.label == label)
        .ok_or_else(|| {
            error(
                "ITEM-PROOF-MODULE-BASE-INVALID",
                format!("git.{label}"),
                "shared proof Area base is missing the required instance list",
            )
        })?;
    field.value = GffValueV1::List(values);
    Ok(())
}

fn write_item_proof_gff_v1(document: &GffDocumentV1, path: &str) -> Result<Vec<u8>, ItemErrorV1> {
    write_gff_v32(document, &GffWriterOptionsV1::default())
        .map(|artifact| artifact.payload)
        .map_err(|source| {
            error(
                "ITEM-PROOF-MODULE-GFF-WRITE-FAILED",
                format!("{path}.{}", source.path),
                source.message,
            )
        })
}

pub fn build_item_proof_module_v1(
    uti_payload: &[u8],
    identity: &ItemProofModuleIdentityV1,
    placement: ItemProofPlacementV1,
) -> Result<ItemProofModuleArtifactV1, ItemErrorV1> {
    if identity.schema_version != ITEM_SCHEMA_VERSION_V1
        || placement.schema_version != ITEM_SCHEMA_VERSION_V1
    {
        return Err(error(
            "ITEM-PROOF-MODULE-SCHEMA-INVALID",
            "identity.schemaVersion",
            "item proof identity and placement schemaVersion must both be 1",
        ));
    }
    let module_resref = validate_resref(&identity.module_resref, "identity.moduleResref")?;
    let area_resref = validate_resref(&identity.area_resref, "identity.areaResref")?;
    let hak_resref = validate_resref(&identity.hak_resref, "identity.hakResref")?;
    let blueprint_resref = validate_resref(&identity.blueprint_resref, "identity.blueprintResref")?;
    if identity.module_name.trim().is_empty() || identity.area_name.trim().is_empty() {
        return Err(error(
            "ITEM-PROOF-MODULE-NAME-INVALID",
            "identity.moduleName",
            "moduleName and areaName must be non-empty",
        ));
    }
    if ![
        placement.x,
        placement.y,
        placement.z,
        placement.orientation_x,
        placement.orientation_y,
    ]
    .into_iter()
    .all(f32::is_finite)
    {
        return Err(error(
            "ITEM-PROOF-PLACEMENT-NONFINITE",
            "placement",
            "item proof placement must contain only finite values",
        ));
    }

    let uti = read_gff_v32(uti_payload, &GffLimitsV1::default())
        .map_err(|source| error("ITEM-PROOF-UTI-READ-FAILED", source.path, source.message))?;
    if uti.file_type != GffFileTypeV1::Uti {
        return Err(error(
            "ITEM-PROOF-UTI-TYPE-INVALID",
            "uti.header",
            "candidate item proof input must be a UTI V3.2 payload",
        ));
    }
    let template_resref = uti
        .root
        .fields
        .iter()
        .find(|field| field.label == "TemplateResRef")
        .and_then(|field| match &field.value {
            GffValueV1::ResRef(value) => Some(value.as_str()),
            _ => None,
        });
    if template_resref != Some(blueprint_resref.as_str()) {
        return Err(error(
            "ITEM-PROOF-UTI-IDENTITY-MISMATCH",
            "uti.TemplateResRef",
            "UTI TemplateResRef differs from the candidate proof blueprint resref",
        ));
    }

    let ifo = binary_m0_module_ifo_for(
        &module_resref,
        &area_resref,
        &[hak_resref.as_str()],
        identity.module_name.trim(),
        "Candidate-bound Meshy2Aurora Item proof module. Visual proof is owner-owned.",
    )
    .map_err(|source| item_proof_map_error_v1("ITEM-PROOF-IFO-BUILD-FAILED", "ifo", source))?;
    let mut are = read_gff_v32(
        &binary_m0_area_for(&area_resref).map_err(|source| {
            item_proof_map_error_v1("ITEM-PROOF-ARE-BUILD-FAILED", "are", source)
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| error("ITEM-PROOF-ARE-READ-FAILED", source.path, source.message))?;
    if let Some(name) = are
        .root
        .fields
        .iter_mut()
        .find(|field| field.label == "Name")
    {
        name.value = loc_string(identity.area_name.trim());
    }
    let are = write_item_proof_gff_v1(&are, "are")?;

    let mut gic = read_gff_v32(
        &binary_creature_multi_fixture_gic(&[]).map_err(|source| {
            item_proof_map_error_v1("ITEM-PROOF-GIC-BUILD-FAILED", "gic", source)
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| error("ITEM-PROOF-GIC-READ-FAILED", source.path, source.message))?;
    replace_gff_list_v1(
        &mut gic,
        "List",
        vec![GffStructV1 {
            struct_id: 0,
            fields: vec![field(
                "Comment",
                GffValueV1::String(
                    format!(
                        "Candidate-bound item {} at ({}, {}, {})",
                        blueprint_resref, placement.x, placement.y, placement.z
                    )
                    .into_bytes(),
                ),
            )],
        }],
    )?;
    let gic = write_item_proof_gff_v1(&gic, "gic")?;

    let mut git = read_gff_v32(
        &binary_creature_multi_fixture_git(&[]).map_err(|source| {
            item_proof_map_error_v1("ITEM-PROOF-GIT-BUILD-FAILED", "git", source)
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| error("ITEM-PROOF-GIT-READ-FAILED", source.path, source.message))?;
    let mut instance_fields = uti.root.fields.clone();
    instance_fields.extend([
        field("XPosition", GffValueV1::Float(placement.x)),
        field("YPosition", GffValueV1::Float(placement.y)),
        field("ZPosition", GffValueV1::Float(placement.z)),
        field("XOrientation", GffValueV1::Float(placement.orientation_x)),
        field("YOrientation", GffValueV1::Float(placement.orientation_y)),
    ]);
    replace_gff_list_v1(
        &mut git,
        "List",
        vec![GffStructV1 {
            struct_id: 0,
            fields: instance_fields,
        }],
    )?;
    let git = write_item_proof_gff_v1(&git, "git")?;
    let fac = proof_factions()
        .map_err(|source| item_proof_map_error_v1("ITEM-PROOF-FAC-BUILD-FAILED", "fac", source))?;
    let resources = vec![
        item_proof_resource_v1("module", 2014, ifo),
        item_proof_resource_v1("repute", 2038, fac),
        item_proof_resource_v1(&area_resref, 2012, are),
        item_proof_resource_v1(&area_resref, 2046, gic),
        item_proof_resource_v1(&area_resref, 2023, git),
        item_proof_resource_v1(&blueprint_resref, 2025, uti_payload.to_vec()),
    ];
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PROOF-MODULE-WRITE-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    let readback = ErfArchive::parse(&archive.payload).map_err(|source| {
        error(
            "ITEM-PROOF-MODULE-READBACK-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    if readback.file_type() != ErfFileType::Module
        || readback.resources().len() != resources.len()
        || readback.find(&blueprint_resref, 2025).ok() != Some(uti_payload)
    {
        return Err(error(
            "ITEM-PROOF-MODULE-SEMANTIC-DIFF",
            "module.resources",
            "MOD readback does not contain the exact candidate UTI and expected resource count",
        ));
    }
    let git_readback = read_gff_v32(
        readback.find(&area_resref, 2023).map_err(|source| {
            error(
                "ITEM-PROOF-MODULE-SEMANTIC-DIFF",
                "module.git",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PROOF-GIT-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let item_list = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        });
    if item_list.is_none_or(|items| items.len() != 1) {
        return Err(error(
            "ITEM-PROOF-MODULE-SEMANTIC-DIFF",
            "git.List",
            "candidate MOD must contain exactly one placed item instance",
        ));
    }
    let creature_list = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        });
    if creature_list.is_none_or(|creatures| !creatures.is_empty())
        || readback
            .resources()
            .iter()
            .any(|resource| resource.resource_type == 2027)
    {
        return Err(error(
            "ITEM-PROOF-MODULE-CREATURE-UNEXPECTED",
            "module.git.Creature List",
            "item-only proof MOD must contain zero creatures and zero UTC resources",
        ));
    }

    Ok(ItemProofModuleArtifactV1 {
        report: ItemProofModuleReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            fixture_profile: "ITEM_ONLY_GROUND_ITEM_V1".to_owned(),
            module_resref,
            module_name: identity.module_name.trim().to_owned(),
            area_resref,
            area_name: identity.area_name.trim().to_owned(),
            hak_resref,
            blueprint_resref,
            ground_item_count: 1,
            creature_count: 0,
            item_position: [placement.x, placement.y, placement.z],
            entry_position: [M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z],
            entry_direction: [M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y],
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            output_sha256: item_payload_sha256_v1(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            model_visibility: "not_tested".to_owned(),
            proof_completeness: "missing".to_owned(),
        },
        payload: archive.payload,
    })
}

/// Builds the owner-facing ranged fixture with both exact UTI payloads placed
/// on the ground. It prepares acquisition of the weapon and known stack but
/// deliberately does not claim that firing, consumption or projectile
/// visibility has been proven in NWN runtime.
pub fn build_item_ranged_weapon_proof_module_v1(
    weapon_uti_payload: &[u8],
    ammunition_uti_payload: &[u8],
    identity: &ItemRangedWeaponProofIdentityV1,
    weapon_placement: ItemProofPlacementV1,
    ammunition_placement: ItemProofPlacementV1,
) -> Result<ItemRangedWeaponProofModuleArtifactV1, ItemErrorV1> {
    if identity.schema_version != ITEM_SCHEMA_VERSION_V1
        || weapon_placement.schema_version != ITEM_SCHEMA_VERSION_V1
        || ammunition_placement.schema_version != ITEM_SCHEMA_VERSION_V1
    {
        return Err(error(
            "ITEM-RANGED-PROOF-SCHEMA-INVALID",
            "identity.schemaVersion",
            "ranged proof identity and both placements must use schemaVersion 1",
        ));
    }
    let module_resref = validate_resref(&identity.module_resref, "identity.moduleResref")?;
    let area_resref = validate_resref(&identity.area_resref, "identity.areaResref")?;
    let hak_resref = validate_resref(&identity.hak_resref, "identity.hakResref")?;
    let weapon_blueprint_resref = validate_resref(
        &identity.weapon_blueprint_resref,
        "identity.weaponBlueprintResref",
    )?;
    let ammunition_blueprint_resref = validate_resref(
        &identity.ammunition_blueprint_resref,
        "identity.ammunitionBlueprintResref",
    )?;
    if weapon_blueprint_resref == ammunition_blueprint_resref {
        return Err(error(
            "ITEM-RANGED-PROOF-UTI-IDENTITY-COLLISION",
            "identity.ammunitionBlueprintResref",
            "weapon and ammunition proof UTI resrefs must differ",
        ));
    }
    if ![
        weapon_placement.x,
        weapon_placement.y,
        weapon_placement.z,
        weapon_placement.orientation_x,
        weapon_placement.orientation_y,
        ammunition_placement.x,
        ammunition_placement.y,
        ammunition_placement.z,
        ammunition_placement.orientation_x,
        ammunition_placement.orientation_y,
    ]
    .into_iter()
    .all(f32::is_finite)
    {
        return Err(error(
            "ITEM-RANGED-PROOF-PLACEMENT-NONFINITE",
            "placement",
            "ranged proof placements must contain only finite values",
        ));
    }
    let ammunition_uti =
        read_gff_v32(ammunition_uti_payload, &GffLimitsV1::default()).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-AMMUNITION-UTI-READ-FAILED",
                source.path,
                source.message,
            )
        })?;
    let ammunition_template = ammunition_uti
        .root
        .fields
        .iter()
        .find(|field| field.label == "TemplateResRef")
        .and_then(|field| match &field.value {
            GffValueV1::ResRef(value) => Some(value.as_str()),
            _ => None,
        });
    if ammunition_uti.file_type != GffFileTypeV1::Uti
        || ammunition_template != Some(ammunition_blueprint_resref.as_str())
    {
        return Err(error(
            "ITEM-RANGED-PROOF-AMMUNITION-UTI-IDENTITY-MISMATCH",
            "ammunitionUti.TemplateResRef",
            "ammunition proof input must be the exact requested UTI",
        ));
    }

    const TARGET_BLUEPRINT_RESREF: &str = "m2arngtarget";
    const TARGET_APPEARANCE_ROW: u16 = 102;
    const TARGET_POSITION: M0RuntimePositionV1 = M0RuntimePositionV1 {
        x: 10.0,
        y: 18.0,
        z: 0.0,
    };
    const TARGET_SCRIPT_FIELDS: [&str; 13] = [
        "ScriptHeartbeat",
        "ScriptOnNotice",
        "ScriptSpellAt",
        "ScriptAttacked",
        "ScriptDamaged",
        "ScriptDisturbed",
        "ScriptEndRound",
        "ScriptDialogue",
        "ScriptSpawn",
        "ScriptRested",
        "ScriptDeath",
        "ScriptUserDefine",
        "ScriptOnBlocked",
    ];
    let target_fixture = BinaryCreatureOwnedFixtureV1 {
        id: "m2a_ranged_target".to_owned(),
        template_resref: TARGET_BLUEPRINT_RESREF.to_owned(),
        display_name: "Nieruchomy cel treningowy".to_owned(),
        appearance_row: TARGET_APPEARANCE_ROW,
        position: TARGET_POSITION,
        orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
    };
    let target_module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: module_resref.clone(),
            area_resref: area_resref.clone(),
            hak_resref: hak_resref.clone(),
        },
        std::slice::from_ref(&target_fixture),
    )
    .map_err(|source| {
        item_proof_map_error_v1("ITEM-RANGED-PROOF-TARGET-BUILD-FAILED", "target", source)
    })?;
    let target_archive = ErfArchive::parse(&target_module.payload).map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-TARGET-MODULE-READ-FAILED",
            "target.module",
            source.to_string(),
        )
    })?;
    let target_git = read_gff_v32(
        target_archive.find(&area_resref, 2023).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-TARGET-GIT-READ-FAILED",
                "target.module.git",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-TARGET-GIT-READ-FAILED",
            source.path,
            source.message,
        )
    })?;
    let target_creatures = target_git
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values.clone()),
            _ => None,
        })
        .filter(|values| values.len() == 1)
        .ok_or_else(|| {
            error(
                "ITEM-RANGED-PROOF-TARGET-GIT-INVALID",
                "target.module.git.Creature List",
                "target fixture must contain exactly one generated creature instance",
            )
        })?;
    let target_gic = read_gff_v32(
        target_archive.find(&area_resref, 2046).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-TARGET-GIC-READ-FAILED",
                "target.module.gic",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-TARGET-GIC-READ-FAILED",
            source.path,
            source.message,
        )
    })?;
    let target_creature_comments = target_gic
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values.clone()),
            _ => None,
        })
        .filter(|values| values.len() == 1)
        .ok_or_else(|| {
            error(
                "ITEM-RANGED-PROOF-TARGET-GIC-INVALID",
                "target.module.gic.Creature List",
                "target fixture GIC must contain exactly one creature comment",
            )
        })?;
    let target_utc_payload = target_archive
        .find(TARGET_BLUEPRINT_RESREF, 2027)
        .map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-TARGET-UTC-READ-FAILED",
                "target.module.utc",
                source.to_string(),
            )
        })?
        .to_vec();

    let base = build_item_proof_module_v1(
        weapon_uti_payload,
        &ItemProofModuleIdentityV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            module_resref: module_resref.clone(),
            area_resref: area_resref.clone(),
            hak_resref: hak_resref.clone(),
            blueprint_resref: weapon_blueprint_resref.clone(),
            module_name: identity.module_name.clone(),
            area_name: identity.area_name.clone(),
        },
        weapon_placement,
    )?;
    let base_archive = ErfArchive::parse(&base.payload).map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-MODULE-READBACK-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    let mut git = read_gff_v32(
        base_archive.find(&area_resref, 2023).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-GIT-READ-FAILED",
                "module.git",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-GIT-READ-FAILED",
            source.path,
            source.message,
        )
    })?;
    let mut item_list = git
        .root
        .fields
        .iter()
        .find(|field| field.label == "List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                "ITEM-RANGED-PROOF-GIT-LIST-MISSING",
                "git.List",
                "base proof module is missing its item instance list",
            )
        })?;
    if item_list.len() != 1 {
        return Err(error(
            "ITEM-RANGED-PROOF-GIT-LIST-INVALID",
            "git.List",
            "base proof module must contain exactly one weapon instance",
        ));
    }
    let mut ammunition_fields = ammunition_uti.root.fields.clone();
    ammunition_fields.extend([
        field("XPosition", GffValueV1::Float(ammunition_placement.x)),
        field("YPosition", GffValueV1::Float(ammunition_placement.y)),
        field("ZPosition", GffValueV1::Float(ammunition_placement.z)),
        field(
            "XOrientation",
            GffValueV1::Float(ammunition_placement.orientation_x),
        ),
        field(
            "YOrientation",
            GffValueV1::Float(ammunition_placement.orientation_y),
        ),
    ]);
    item_list.push(GffStructV1 {
        struct_id: 1,
        fields: ammunition_fields,
    });
    replace_gff_list_v1(&mut git, "List", item_list)?;
    replace_gff_list_v1(&mut git, "Creature List", target_creatures)?;
    let ranged_git = write_item_proof_gff_v1(&git, "git")?;

    let mut gic = read_gff_v32(
        base_archive.find(&area_resref, 2046).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-GIC-READ-FAILED",
                "module.gic",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-GIC-READ-FAILED",
            source.path,
            source.message,
        )
    })?;
    replace_gff_list_v1(&mut gic, "Creature List", target_creature_comments)?;
    let ranged_gic = write_item_proof_gff_v1(&gic, "gic")?;

    let mut resources = Vec::with_capacity(base_archive.resources().len() + 2);
    for resource in base_archive.resources() {
        let payload = if resource.resref.eq_ignore_ascii_case(&area_resref)
            && resource.resource_type == 2023
        {
            ranged_git.clone()
        } else if resource.resref.eq_ignore_ascii_case(&area_resref)
            && resource.resource_type == 2046
        {
            ranged_gic.clone()
        } else {
            base_archive
                .find(&resource.resref, resource.resource_type)
                .map_err(|source| {
                    error(
                        "ITEM-RANGED-PROOF-RESOURCE-READ-FAILED",
                        "module.resources",
                        source.to_string(),
                    )
                })?
                .to_vec()
        };
        resources.push(item_proof_resource_v1(
            &resource.resref,
            resource.resource_type,
            payload,
        ));
    }
    resources.push(item_proof_resource_v1(
        &ammunition_blueprint_resref,
        2025,
        ammunition_uti_payload.to_vec(),
    ));
    resources.push(item_proof_resource_v1(
        TARGET_BLUEPRINT_RESREF,
        2027,
        target_utc_payload.clone(),
    ));
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-MODULE-WRITE-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    let readback = ErfArchive::parse(&archive.payload).map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-MODULE-READBACK-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    if readback.file_type() != ErfFileType::Module
        || readback.resources().len() != resources.len()
        || readback.find(&weapon_blueprint_resref, 2025).ok() != Some(weapon_uti_payload)
        || readback.find(&ammunition_blueprint_resref, 2025).ok() != Some(ammunition_uti_payload)
        || readback.find(TARGET_BLUEPRINT_RESREF, 2027).ok() != Some(target_utc_payload.as_slice())
    {
        return Err(error(
            "ITEM-RANGED-PROOF-MODULE-SEMANTIC-DIFF",
            "module.resources",
            "ranged MOD does not contain both exact UTI payloads and expected resources",
        ));
    }
    let git_readback = read_gff_v32(
        readback.find(&area_resref, 2023).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-MODULE-SEMANTIC-DIFF",
                "module.git",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-GIT-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let items = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        });
    let template_resrefs = items
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.fields
                .iter()
                .find(|field| field.label == "TemplateResRef")
                .and_then(|field| match &field.value {
                    GffValueV1::ResRef(value) => Some(value.clone()),
                    _ => None,
                })
        })
        .collect::<BTreeSet<_>>();
    if items.is_none_or(|items| items.len() != 2)
        || template_resrefs
            != BTreeSet::from([
                weapon_blueprint_resref.clone(),
                ammunition_blueprint_resref.clone(),
            ])
    {
        return Err(error(
            "ITEM-RANGED-PROOF-GIT-SEMANTIC-DIFF",
            "git.List",
            "ranged proof Area must place exactly the requested weapon and ammunition stack",
        ));
    }

    let creatures = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        });
    let target = creatures.and_then(|values| values.first());
    let target_field = |label: &str| {
        target.and_then(|creature| {
            creature
                .fields
                .iter()
                .find(|field| field.label == label)
                .map(|field| &field.value)
        })
    };
    let target_scripts_empty = TARGET_SCRIPT_FIELDS
        .iter()
        .all(|label| target_field(label) == Some(&GffValueV1::ResRef(String::new())));
    if creatures.is_none_or(|values| values.len() != 1)
        || target_field("TemplateResRef")
            != Some(&GffValueV1::ResRef(TARGET_BLUEPRINT_RESREF.to_owned()))
        || target_field("Appearance_Type") != Some(&GffValueV1::Word(TARGET_APPEARANCE_ROW))
        || target_field("XPosition") != Some(&GffValueV1::Float(TARGET_POSITION.x))
        || target_field("YPosition") != Some(&GffValueV1::Float(TARGET_POSITION.y))
        || target_field("ZPosition") != Some(&GffValueV1::Float(TARGET_POSITION.z))
        || target_field("WalkRate") != Some(&GffValueV1::Int(0))
        || target_field("PerceptionRange") != Some(&GffValueV1::Byte(0))
        || !target_scripts_empty
    {
        return Err(error(
            "ITEM-RANGED-PROOF-TARGET-SEMANTIC-DIFF",
            "git.Creature List",
            "ranged demo must contain one exact immobile, perception-free target with empty AI scripts",
        ));
    }
    let target_utc_readback = read_gff_v32(
        readback
            .find(TARGET_BLUEPRINT_RESREF, 2027)
            .map_err(|source| {
                error(
                    "ITEM-RANGED-PROOF-TARGET-UTC-READBACK-FAILED",
                    "module.utc",
                    source.to_string(),
                )
            })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-TARGET-UTC-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let utc_field = |label: &str| {
        target_utc_readback
            .root
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    if target_utc_readback.file_type != GffFileTypeV1::Utc
        || utc_field("TemplateResRef")
            != Some(&GffValueV1::ResRef(TARGET_BLUEPRINT_RESREF.to_owned()))
        || utc_field("WalkRate") != Some(&GffValueV1::Int(0))
        || utc_field("PerceptionRange") != Some(&GffValueV1::Byte(0))
        || !TARGET_SCRIPT_FIELDS
            .iter()
            .all(|label| utc_field(label) == Some(&GffValueV1::ResRef(String::new())))
    {
        return Err(error(
            "ITEM-RANGED-PROOF-TARGET-UTC-SEMANTIC-DIFF",
            "module.utc",
            "target UTC must preserve the exact immobile, perception-free and script-free runtime envelope",
        ));
    }
    let gic_readback = read_gff_v32(
        readback.find(&area_resref, 2046).map_err(|source| {
            error(
                "ITEM-RANGED-PROOF-GIC-READBACK-FAILED",
                "module.gic",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-RANGED-PROOF-GIC-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let gic_creatures = gic_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        });
    if gic_creatures.is_none_or(|values| values.len() != 1) {
        return Err(error(
            "ITEM-RANGED-PROOF-GIC-SEMANTIC-DIFF",
            "module.gic.Creature List",
            "ranged demo GIC must describe exactly one target creature",
        ));
    }

    Ok(ItemRangedWeaponProofModuleArtifactV1 {
        report: ItemRangedWeaponProofModuleReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            fixture_profile: "RANGED_WEAPON_AMMUNITION_AND_IMMOBILE_TARGET_V1".to_owned(),
            module_resref,
            module_name: identity.module_name.trim().to_owned(),
            area_resref,
            area_name: identity.area_name.trim().to_owned(),
            hak_resref,
            weapon_blueprint_resref,
            ammunition_blueprint_resref,
            ground_item_count: 2,
            creature_count: 1,
            weapon_position: [weapon_placement.x, weapon_placement.y, weapon_placement.z],
            ammunition_position: [
                ammunition_placement.x,
                ammunition_placement.y,
                ammunition_placement.z,
            ],
            target_blueprint_resref: TARGET_BLUEPRINT_RESREF.to_owned(),
            target_appearance_row: TARGET_APPEARANCE_ROW,
            target_position: [TARGET_POSITION.x, TARGET_POSITION.y, TARGET_POSITION.z],
            target_walk_rate: 0,
            target_scripts_empty,
            entry_position: [M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z],
            entry_direction: [M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y],
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            output_sha256: item_payload_sha256_v1(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            model_visibility: "not_tested".to_owned(),
            proof_completeness: "missing".to_owned(),
        },
        payload: archive.payload,
    })
}

fn set_equipped_item_reference_v2(
    fields: &mut [GffFieldV1],
    equipment_slot: u32,
    blueprint_resref: &str,
    path: &str,
) -> Result<(), ItemErrorV1> {
    let equipment = fields
        .iter_mut()
        .find(|field| field.label == "Equip_ItemList")
        .ok_or_else(|| {
            error(
                "ITEM-EQUIPPED-PROOF-CREATURE-INVALID",
                format!("{path}.Equip_ItemList"),
                "generated proof creature is missing Equip_ItemList",
            )
        })?;
    equipment.value = GffValueV1::List(vec![GffStructV1 {
        struct_id: equipment_slot,
        fields: vec![field(
            "EquippedRes",
            GffValueV1::ResRef(blueprint_resref.to_owned()),
        )],
    }]);
    Ok(())
}

fn verify_equipped_item_reference_v2(
    fields: &[GffFieldV1],
    equipment_slot: u32,
    blueprint_resref: &str,
    path: &str,
) -> Result<(), ItemErrorV1> {
    let values = fields
        .iter()
        .find(|field| field.label == "Equip_ItemList")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
                format!("{path}.Equip_ItemList"),
                "proof creature Equip_ItemList is missing or has the wrong field type",
            )
        })?;
    let exact = matches!(
        values.as_slice(),
        [equipped]
            if equipped.struct_id == equipment_slot
                && matches!(
                    equipped.fields.as_slice(),
                    [GffFieldV1 {
                        label,
                        value: GffValueV1::ResRef(value),
                    }] if label == "EquippedRes" && value == blueprint_resref
                )
    );
    if !exact {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
            format!("{path}.Equip_ItemList"),
            "proof creature must contain exactly one EquippedRes bound to the expected native equipment slot",
        ));
    }
    Ok(())
}

fn set_equipped_creature_context_v2(
    fields: &mut [GffFieldV1],
    identity: &ItemEquippedProofIdentityV2,
    path: &str,
) -> Result<(), ItemErrorV1> {
    for (label, value) in [
        ("Race", GffValueV1::Byte(identity.race)),
        ("Gender", GffValueV1::Byte(identity.gender)),
        ("Phenotype", GffValueV1::Int(identity.phenotype)),
    ] {
        let field = fields
            .iter_mut()
            .find(|field| field.label == label)
            .ok_or_else(|| {
                error(
                    "ITEM-EQUIPPED-PROOF-CREATURE-INVALID",
                    format!("{path}.{label}"),
                    format!("generated proof creature is missing {label}"),
                )
            })?;
        field.value = value;
    }
    Ok(())
}

fn verify_equipped_creature_context_v2(
    fields: &[GffFieldV1],
    identity: &ItemEquippedProofIdentityV2,
    path: &str,
) -> Result<(), ItemErrorV1> {
    let exact = [
        ("Appearance_Type", GffValueV1::Word(identity.appearance_row)),
        ("Race", GffValueV1::Byte(identity.race)),
        ("Gender", GffValueV1::Byte(identity.gender)),
        ("Phenotype", GffValueV1::Int(identity.phenotype)),
    ]
    .into_iter()
    .all(|(label, expected)| {
        fields
            .iter()
            .find(|field| field.label == label)
            .is_some_and(|field| field.value == expected)
    });
    if !exact {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-CONTEXT-DIFF",
            path,
            "proof creature Appearance_Type/Race/Gender/Phenotype differ from the exact requested model context",
        ));
    }
    Ok(())
}

pub fn build_item_equipped_proof_module_v2(
    uti_payload: &[u8],
    identity: &ItemEquippedProofIdentityV2,
    placement: ItemProofPlacementV1,
) -> Result<ItemEquippedProofModuleArtifactV2, ItemErrorV1> {
    if identity.schema_version != 2 || placement.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-SCHEMA-INVALID",
            "identity.schemaVersion",
            "equipped proof identity schemaVersion must be 2 and placement schemaVersion must be 1",
        ));
    }
    let module_resref = validate_resref(&identity.module_resref, "identity.moduleResref")?;
    let area_resref = validate_resref(&identity.area_resref, "identity.areaResref")?;
    let hak_resref = validate_resref(&identity.hak_resref, "identity.hakResref")?;
    let blueprint_resref = validate_resref(&identity.blueprint_resref, "identity.blueprintResref")?;
    let creature_resref = validate_resref(&identity.creature_resref, "identity.creatureResref")?;
    let model_prefix = validate_item_player_model_prefix_v1(&identity.model_prefix)?;
    if !valid_lowercase_sha256_v1(&identity.appearance_table_sha256) {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-APPEARANCE-BINDING-INVALID",
            "identity.appearanceTableSha256",
            "equipped proof identity requires the exact lowercase SHA-256 of its validated appearance.2da",
        ));
    }
    if identity.module_name.trim().is_empty()
        || identity.area_name.trim().is_empty()
        || identity.creature_display_name.trim().is_empty()
    {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-NAME-INVALID",
            "identity",
            "module, Area and creature display names must be non-empty",
        ));
    }
    if !(0..=255).contains(&identity.phenotype) {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-PHENOTYPE-INVALID",
            "identity.phenotype",
            "proof creature phenotype must be an integer in 0..255",
        ));
    }
    let expected_gender_token = match identity.gender {
        0 => b'm',
        1 => b'f',
        _ => {
            return Err(error(
                "ITEM-EQUIPPED-PROOF-GENDER-INVALID",
                "identity.gender",
                "part-based proof creature gender must be 0 (male) or 1 (female)",
            ));
        }
    };
    if model_prefix.as_bytes()[1] != expected_gender_token
        || usize::from(model_prefix.as_bytes()[3] - b'0') != identity.phenotype as usize
    {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-MODEL-PREFIX-MISMATCH",
            "identity.modelPrefix",
            "equipped proof model prefix must encode the exact requested gender and phenotype",
        ));
    }
    if ![
        placement.x,
        placement.y,
        placement.z,
        placement.orientation_x,
        placement.orientation_y,
    ]
    .into_iter()
    .all(f32::is_finite)
    {
        return Err(error(
            "ITEM-PROOF-PLACEMENT-NONFINITE",
            "placement",
            "equipped proof placement must contain only finite values",
        ));
    }
    let uti = read_gff_v32(uti_payload, &GffLimitsV1::default())
        .map_err(|source| error("ITEM-PROOF-UTI-READ-FAILED", source.path, source.message))?;
    if uti.file_type != GffFileTypeV1::Uti
        || uti
            .root
            .fields
            .iter()
            .find(|field| field.label == "TemplateResRef")
            .and_then(|field| match &field.value {
                GffValueV1::ResRef(value) => Some(value.as_str()),
                _ => None,
            })
            != Some(blueprint_resref.as_str())
    {
        return Err(error(
            "ITEM-PROOF-UTI-IDENTITY-MISMATCH",
            "uti.TemplateResRef",
            "UTI type or TemplateResRef differs from the equipped proof identity",
        ));
    }
    if identity.fixture_profile == ItemEquippedProofProfileV2::ModelType2Parts {
        let exact_model_parts =
            ["ModelPart1", "ModelPart2", "ModelPart3"]
                .into_iter()
                .all(|label| {
                    uti.root.fields.iter().any(|field| {
                        field.label == label && matches!(field.value, GffValueV1::Byte(_))
                    })
                });
        if !exact_model_parts {
            return Err(error(
                "ITEM-EQUIPPED-PROOF-MODELTYPE2-UTI-INVALID",
                "uti",
                "ModelType 2 equipped proof requires BYTE fields ModelPart1, ModelPart2 and ModelPart3",
            ));
        }
    }
    let equipment_slot = identity
        .fixture_profile
        .equipment_slot(identity.equipment_slot)?;

    let ifo = binary_m0_module_ifo_for(
        &module_resref,
        &area_resref,
        &[hak_resref.as_str()],
        identity.module_name.trim(),
        "Candidate-bound equipped Item proof module. Visual proof is owner-owned.",
    )
    .map_err(|source| item_proof_map_error_v1("ITEM-PROOF-IFO-BUILD-FAILED", "ifo", source))?;
    let mut are = read_gff_v32(
        &binary_m0_area_for(&area_resref).map_err(|source| {
            item_proof_map_error_v1("ITEM-PROOF-ARE-BUILD-FAILED", "are", source)
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| error("ITEM-PROOF-ARE-READ-FAILED", source.path, source.message))?;
    if let Some(name) = are
        .root
        .fields
        .iter_mut()
        .find(|field| field.label == "Name")
    {
        name.value = loc_string(identity.area_name.trim());
    }
    let are = write_item_proof_gff_v1(&are, "are")?;
    let fixture = BinaryCreatureOwnedFixtureV1 {
        id: creature_resref.clone(),
        template_resref: creature_resref.clone(),
        display_name: identity.creature_display_name.trim().to_owned(),
        appearance_row: identity.appearance_row,
        position: M0RuntimePositionV1 {
            x: placement.x,
            y: placement.y,
            z: placement.z,
        },
        orientation: M0RuntimeDirectionV1 {
            x: placement.orientation_x,
            y: placement.orientation_y,
        },
    };
    let gic = binary_creature_multi_fixture_gic(std::slice::from_ref(&fixture))
        .map_err(|source| item_proof_map_error_v1("ITEM-PROOF-GIC-BUILD-FAILED", "gic", source))?;
    let mut git = read_gff_v32(
        &binary_creature_multi_fixture_git(std::slice::from_ref(&fixture)).map_err(|source| {
            item_proof_map_error_v1("ITEM-PROOF-GIT-BUILD-FAILED", "git", source)
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| error("ITEM-PROOF-GIT-READ-FAILED", source.path, source.message))?;
    let creatures = git
        .root
        .fields
        .iter_mut()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &mut field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                "ITEM-EQUIPPED-PROOF-CREATURE-INVALID",
                "git.Creature List",
                "generated proof GIT is missing Creature List",
            )
        })?;
    let [creature] = creatures.as_mut_slice() else {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-CREATURE-INVALID",
            "git.Creature List",
            "equipped proof GIT must contain exactly one creature",
        ));
    };
    set_equipped_item_reference_v2(
        &mut creature.fields,
        equipment_slot,
        &blueprint_resref,
        "git.Creature List[0]",
    )?;
    set_equipped_creature_context_v2(&mut creature.fields, identity, "git.Creature List[0]")?;
    let mut utc_fields = creature.fields.clone();
    utc_fields.retain(|field| {
        !matches!(
            field.label.as_str(),
            "XPosition" | "YPosition" | "ZPosition" | "XOrientation" | "YOrientation"
        )
    });
    utc_fields.insert(0, field("PaletteID", GffValueV1::Byte(0)));
    utc_fields.insert(
        1,
        field(
            "Comment",
            GffValueV1::String(
                format!(
                    "Equipped {} proof wearer generated by Meshy2Aurora.",
                    identity.fixture_profile.report_name()
                )
                .into_bytes(),
            ),
        ),
    );
    let utc = write_item_proof_gff_v1(
        &GffDocumentV1 {
            schema_version: GFF_SCHEMA_VERSION,
            file_type: GffFileTypeV1::Utc,
            root: GffStructV1 {
                struct_id: u32::MAX,
                fields: utc_fields,
            },
        },
        "utc",
    )?;
    let git = write_item_proof_gff_v1(&git, "git")?;
    let fac = proof_factions()
        .map_err(|source| item_proof_map_error_v1("ITEM-PROOF-FAC-BUILD-FAILED", "fac", source))?;
    let resources = vec![
        item_proof_resource_v1("module", 2014, ifo),
        item_proof_resource_v1("repute", 2038, fac),
        item_proof_resource_v1(&area_resref, 2012, are),
        item_proof_resource_v1(&area_resref, 2046, gic),
        item_proof_resource_v1(&area_resref, 2023, git),
        item_proof_resource_v1(&blueprint_resref, 2025, uti_payload.to_vec()),
        item_proof_resource_v1(&creature_resref, 2027, utc),
    ];
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PROOF-MODULE-WRITE-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    let readback = ErfArchive::parse(&archive.payload).map_err(|source| {
        error(
            "ITEM-PROOF-MODULE-READBACK-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    if readback.file_type() != ErfFileType::Module
        || readback.resources().len() != resources.len()
        || readback.find(&blueprint_resref, 2025).ok() != Some(uti_payload)
    {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
            "module.resources",
            "equipped proof MOD resource readback differs from the exact input set",
        ));
    }
    let git_readback = read_gff_v32(
        readback.find(&area_resref, 2023).map_err(|source| {
            error(
                "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
                "module.git",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PROOF-GIT-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let git_creatures = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
                "git.Creature List",
                "equipped proof GIT has no Creature List",
            )
        })?;
    let [git_creature] = git_creatures.as_slice() else {
        return Err(error(
            "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
            "git.Creature List",
            "equipped proof GIT must contain exactly one creature",
        ));
    };
    verify_equipped_item_reference_v2(
        &git_creature.fields,
        equipment_slot,
        &blueprint_resref,
        "git.Creature List[0]",
    )?;
    verify_equipped_creature_context_v2(&git_creature.fields, identity, "git.Creature List[0]")?;
    let utc_readback = read_gff_v32(
        readback.find(&creature_resref, 2027).map_err(|source| {
            error(
                "ITEM-EQUIPPED-PROOF-SEMANTIC-DIFF",
                "module.utc",
                source.to_string(),
            )
        })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PROOF-UTC-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    verify_equipped_item_reference_v2(
        &utc_readback.root.fields,
        equipment_slot,
        &blueprint_resref,
        "utc",
    )?;
    verify_equipped_creature_context_v2(&utc_readback.root.fields, identity, "utc")?;

    Ok(ItemEquippedProofModuleArtifactV2 {
        report: ItemEquippedProofModuleReportV2 {
            schema_version: 2,
            module_resref,
            module_name: identity.module_name.trim().to_owned(),
            area_resref,
            area_name: identity.area_name.trim().to_owned(),
            hak_resref,
            blueprint_resref,
            creature_resref,
            appearance_row: identity.appearance_row,
            race: identity.race,
            gender: identity.gender,
            phenotype: identity.phenotype,
            model_prefix,
            appearance_table_sha256: identity.appearance_table_sha256.clone(),
            fixture_profile: identity.fixture_profile.report_name().to_owned(),
            equipment_slot,
            creature_position: [placement.x, placement.y, placement.z],
            entry_position: [M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z],
            entry_direction: [M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y],
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            output_sha256: item_payload_sha256_v1(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            model_visibility: "not_tested".to_owned(),
            proof_completeness: "missing".to_owned(),
        },
        payload: archive.payload,
    })
}

/// Builds one candidate-bound demo Area with two deliberately separate views
/// of the same byte-identical UTI: a real placed Item instance and a humanoid
/// render witness carrying that UTI in the native equipment slot. The placed
/// Item proves the object type while the equipped witness exercises Aurora's
/// ModelType 2 part assembler.
pub fn build_item_and_equipped_proof_module_v3(
    uti_payload: &[u8],
    identity: &ItemEquippedProofIdentityV2,
    item_placement: ItemProofPlacementV1,
    creature_placement: ItemProofPlacementV1,
) -> Result<ItemAndEquippedProofModuleArtifactV3, ItemErrorV1> {
    if item_placement.schema_version != ITEM_SCHEMA_VERSION_V1
        || ![
            item_placement.x,
            item_placement.y,
            item_placement.z,
            item_placement.orientation_x,
            item_placement.orientation_y,
        ]
        .into_iter()
        .all(f32::is_finite)
    {
        return Err(error(
            "ITEM-PAIRED-PROOF-PLACEMENT-INVALID",
            "itemPlacement",
            "paired proof Item placement must use schemaVersion 1 and contain only finite values",
        ));
    }

    let equipped = build_item_equipped_proof_module_v2(uti_payload, identity, creature_placement)?;
    let equipped_archive = ErfArchive::parse(&equipped.payload).map_err(|source| {
        error(
            "ITEM-PAIRED-PROOF-MODULE-READBACK-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    let uti = read_gff_v32(uti_payload, &GffLimitsV1::default())
        .map_err(|source| error("ITEM-PROOF-UTI-READ-FAILED", source.path, source.message))?;
    let mut git = read_gff_v32(
        equipped_archive
            .find(&equipped.report.area_resref, 2023)
            .map_err(|source| {
                error(
                    "ITEM-PAIRED-PROOF-GIT-MISSING",
                    "module.git",
                    source.to_string(),
                )
            })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| error("ITEM-PROOF-GIT-READ-FAILED", source.path, source.message))?;
    let mut item_fields = uti.root.fields.clone();
    item_fields.extend([
        field("XPosition", GffValueV1::Float(item_placement.x)),
        field("YPosition", GffValueV1::Float(item_placement.y)),
        field("ZPosition", GffValueV1::Float(item_placement.z)),
        field(
            "XOrientation",
            GffValueV1::Float(item_placement.orientation_x),
        ),
        field(
            "YOrientation",
            GffValueV1::Float(item_placement.orientation_y),
        ),
    ]);
    replace_gff_list_v1(
        &mut git,
        "List",
        vec![GffStructV1 {
            struct_id: 0,
            fields: item_fields,
        }],
    )?;
    let paired_git = write_item_proof_gff_v1(&git, "git")?;

    let resources = equipped_archive
        .resources()
        .iter()
        .map(|resource| {
            let payload = if resource
                .resref
                .eq_ignore_ascii_case(&equipped.report.area_resref)
                && resource.resource_type == 2023
            {
                paired_git.clone()
            } else {
                equipped_archive
                    .find(&resource.resref, resource.resource_type)
                    .map_err(|source| {
                        error(
                            "ITEM-PAIRED-PROOF-RESOURCE-READBACK-FAILED",
                            format!(
                                "module.resources.{}/{}",
                                resource.resref, resource.resource_type
                            ),
                            source.to_string(),
                        )
                    })?
                    .to_vec()
            };
            Ok(item_proof_resource_v1(
                &resource.resref,
                resource.resource_type,
                payload,
            ))
        })
        .collect::<Result<Vec<_>, ItemErrorV1>>()?;
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PAIRED-PROOF-MODULE-WRITE-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    let readback = ErfArchive::parse(&archive.payload).map_err(|source| {
        error(
            "ITEM-PAIRED-PROOF-MODULE-READBACK-FAILED",
            "module",
            source.to_string(),
        )
    })?;
    if readback.file_type() != ErfFileType::Module
        || readback.resources().len() != resources.len()
        || readback.find(&equipped.report.blueprint_resref, 2025).ok() != Some(uti_payload)
    {
        return Err(error(
            "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
            "module.resources",
            "paired proof MOD does not contain the exact UTI and expected resources",
        ));
    }
    let git_readback = read_gff_v32(
        readback
            .find(&equipped.report.area_resref, 2023)
            .map_err(|source| {
                error(
                    "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
                    "module.git",
                    source.to_string(),
                )
            })?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            "ITEM-PROOF-GIT-READBACK-FAILED",
            source.path,
            source.message,
        )
    })?;
    let items = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
                "git.List",
                "paired proof GIT is missing the Item instance list",
            )
        })?;
    let [placed_item] = items.as_slice() else {
        return Err(error(
            "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
            "git.List",
            "paired proof GIT must contain exactly one placed Item",
        ));
    };
    let placed_template = placed_item
        .fields
        .iter()
        .find(|field| field.label == "TemplateResRef")
        .and_then(|field| match &field.value {
            GffValueV1::ResRef(value) => Some(value.as_str()),
            _ => None,
        });
    if placed_template != Some(equipped.report.blueprint_resref.as_str()) {
        return Err(error(
            "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
            "git.List[0].TemplateResRef",
            "placed Item is not bound to the exact candidate UTI",
        ));
    }
    let creatures = git_readback
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .and_then(|field| match &field.value {
            GffValueV1::List(values) => Some(values),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
                "git.Creature List",
                "paired proof GIT is missing the equipped render witness",
            )
        })?;
    let [creature] = creatures.as_slice() else {
        return Err(error(
            "ITEM-PAIRED-PROOF-SEMANTIC-DIFF",
            "git.Creature List",
            "paired proof GIT must contain exactly one equipped render witness",
        ));
    };
    verify_equipped_item_reference_v2(
        &creature.fields,
        equipped.report.equipment_slot,
        &equipped.report.blueprint_resref,
        "git.Creature List[0]",
    )?;
    verify_equipped_creature_context_v2(&creature.fields, identity, "git.Creature List[0]")?;

    Ok(ItemAndEquippedProofModuleArtifactV3 {
        report: ItemAndEquippedProofModuleReportV3 {
            schema_version: 3,
            module_resref: equipped.report.module_resref,
            module_name: equipped.report.module_name,
            area_resref: equipped.report.area_resref,
            area_name: equipped.report.area_name,
            hak_resref: equipped.report.hak_resref,
            blueprint_resref: equipped.report.blueprint_resref,
            creature_resref: equipped.report.creature_resref,
            fixture_profile: "ITEM_AND_EQUIPPED_MODELTYPE2_PARTS_V3".to_owned(),
            equipment_slot: equipped.report.equipment_slot,
            item_position: [item_placement.x, item_placement.y, item_placement.z],
            creature_position: equipped.report.creature_position,
            ground_item_count: 1,
            equipped_item_count: 1,
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            output_sha256: item_payload_sha256_v1(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            model_visibility: "not_tested".to_owned(),
            proof_completeness: "missing".to_owned(),
        },
        payload: archive.payload,
    })
}

fn write_item_icon_layer_v1(
    source: &TgaImageV1,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ItemErrorV1> {
    if width == 0 || height == 0 {
        return Err(error(
            "ITEM-ICON-DIMENSIONS-INVALID",
            "options.iconSize",
            "item icon dimensions must both be greater than zero",
        ));
    }
    if source.width == 0 || source.height == 0 {
        return Err(error(
            "ITEM-ICON-SOURCE-DIMENSIONS-INVALID",
            "source.image",
            "embedded base-color image has zero width or height",
        ));
    }

    let destination_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|value| value.checked_mul(4))
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            error(
                "ITEM-ICON-SIZE-OVERFLOW",
                "options.iconSize",
                "item icon RGBA payload length exceeds the host address space",
            )
        })?;
    let source_channels = match source.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3_usize,
        TgaPixelFormatV1::Rgba8 => 4_usize,
    };
    let expected_source_len = u64::from(source.width)
        .checked_mul(u64::from(source.height))
        .and_then(|value| value.checked_mul(source_channels as u64))
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            error(
                "ITEM-ICON-SOURCE-SIZE-OVERFLOW",
                "source.image.pixels",
                "embedded base-color image dimensions overflow the host address space",
            )
        })?;
    if source.pixels.len() != expected_source_len {
        return Err(error(
            "ITEM-ICON-SOURCE-PIXELS-INVALID",
            "source.image.pixels",
            format!(
                "embedded base-color image has {} bytes; expected {expected_source_len}",
                source.pixels.len()
            ),
        ));
    }

    let (destination_width, destination_height) = if u64::from(width) * u64::from(source.height)
        <= u64::from(height) * u64::from(source.width)
    {
        (
            width,
            ((u64::from(source.height) * u64::from(width)) / u64::from(source.width)).max(1) as u32,
        )
    } else {
        (
            ((u64::from(source.width) * u64::from(height)) / u64::from(source.height)).max(1)
                as u32,
            height,
        )
    };
    let left = (width - destination_width) / 2;
    let top = (height - destination_height) / 2;
    let mut pixels = vec![0_u8; destination_len];

    for destination_y in 0..destination_height {
        let source_y = (u64::from(destination_y) * u64::from(source.height)
            / u64::from(destination_height)) as u32;
        for destination_x in 0..destination_width {
            let source_x = (u64::from(destination_x) * u64::from(source.width)
                / u64::from(destination_width)) as u32;
            let source_offset =
                ((source_y as usize * source.width as usize) + source_x as usize) * source_channels;
            let destination_offset = (((top + destination_y) as usize * width as usize)
                + (left + destination_x) as usize)
                * 4;
            pixels[destination_offset..destination_offset + 3]
                .copy_from_slice(&source.pixels[source_offset..source_offset + 3]);
            pixels[destination_offset + 3] = if source_channels == 4 {
                source.pixels[source_offset + 3]
            } else {
                u8::MAX
            };
        }
    }

    write_tga_v1(
        &TgaImageV1 {
            schema_version: TGA_SCHEMA_VERSION,
            width,
            height,
            pixel_format: TgaPixelFormatV1::Rgba8,
            pixels,
        },
        &TgaWriterOptionsV1::default(),
    )
    .map(|artifact| artifact.payload)
    .map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.path,
            source.message,
        )
    })
}

#[derive(Clone, Copy)]
struct ItemIconVertexV1 {
    projected: [f32; 3],
    uv: [f32; 2],
}

fn item_node_world_matrices_v1(
    model: &crate::model_ir::AuroraModelIrV1,
) -> Result<BTreeMap<u32, [f32; 16]>, ItemErrorV1> {
    let mut worlds = BTreeMap::new();
    while worlds.len() < model.nodes.len() {
        let mut progressed = false;
        for node in &model.nodes {
            if worlds.contains_key(&node.id) {
                continue;
            }
            let world = match node.parent_id {
                None => node.bind_local_matrix,
                Some(parent_id) => {
                    let Some(parent) = worlds.get(&parent_id).copied() else {
                        continue;
                    };
                    multiply_matrix_v1(parent, node.bind_local_matrix)
                }
            };
            worlds.insert(node.id, world);
            progressed = true;
        }
        if !progressed {
            return Err(error(
                "ITEM-ICON-HIERARCHY-INVALID",
                "model.nodes",
                "geometry-derived icon cannot resolve the model node hierarchy",
            ));
        }
    }
    Ok(worlds)
}

fn project_item_icon_point_aurora_v2(point: [f32; 3]) -> [f32; 3] {
    // Aurora ModelType 2 icons are authored as independent 2D layers on one
    // shared canvas. For the vertical Item profile, view the complete Aurora
    // frame along front/depth X. Aurora presents the decoded TGA with the
    // opposite 2D axial direction to the authoring raster, so rotate the whole
    // shared icon frame by 180 degrees: broad width Z remains horizontal while
    // Top is above Middle and Bottom in Toolset. Never rotate a part alone.
    [point[2], -point[1], point[0]]
}

fn item_icon_geometry_v1(
    model: &crate::model_ir::AuroraModelIrV1,
) -> Result<Vec<Vec<ItemIconVertexV1>>, ItemErrorV1> {
    let worlds = item_node_world_matrices_v1(model)?;
    model
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            if segment.positions.len() != segment.uv0.len()
                || !segment.indices.len().is_multiple_of(3)
            {
                return Err(error(
                    "ITEM-ICON-GEOMETRY-INVALID",
                    format!("model.segments[{segment_index}]"),
                    "geometry-derived icon requires aligned positions/UVs and triangle indices",
                ));
            }
            let world = worlds
                .get(&segment.parent_node_id)
                .copied()
                .ok_or_else(|| {
                    error(
                        "ITEM-ICON-HIERARCHY-INVALID",
                        format!("model.segments[{segment_index}].parentNodeId"),
                        "icon segment parent node is absent from the resolved hierarchy",
                    )
                })?;
            Ok(segment
                .positions
                .iter()
                .zip(&segment.uv0)
                .map(|(&position, &uv)| ItemIconVertexV1 {
                    projected: project_item_icon_point_aurora_v2(transform_point_v1(
                        world, position,
                    )),
                    uv,
                })
                .collect())
        })
        .collect()
}

fn item_icon_projection_bounds_v1(
    geometry: &[Vec<ItemIconVertexV1>],
) -> Result<ItemIconProjectionBoundsV1, ItemErrorV1> {
    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for vertex in geometry.iter().flatten() {
        for axis in 0..2 {
            min[axis] = min[axis].min(vertex.projected[axis]);
            max[axis] = max[axis].max(vertex.projected[axis]);
        }
    }
    let bounds = ItemIconProjectionBoundsV1 { min, max };
    validate_item_icon_projection_bounds_v1(bounds)?;
    Ok(bounds)
}

fn validate_item_icon_projection_bounds_v1(
    bounds: ItemIconProjectionBoundsV1,
) -> Result<(), ItemErrorV1> {
    if bounds
        .min
        .into_iter()
        .chain(bounds.max)
        .any(|value| !value.is_finite())
        || bounds.max[0] <= bounds.min[0]
        || bounds.max[1] <= bounds.min[1]
    {
        return Err(error(
            "ITEM-ICON-PROJECTION-BOUNDS-INVALID",
            "options.iconProjectionBounds",
            "icon projection bounds must be finite and have positive width and height",
        ));
    }
    Ok(())
}

fn item_icon_edge_v1(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> f32 {
    (point[0] - a[0]) * (b[1] - a[1]) - (point[1] - a[1]) * (b[0] - a[0])
}

fn sample_item_icon_texture_v1(source: &TgaImageV1, uv: [f32; 2]) -> [u8; 4] {
    let channels = match source.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3,
        TgaPixelFormatV1::Rgba8 => 4,
    };
    let wrap = |value: f32| {
        let wrapped = value.rem_euclid(1.0);
        if wrapped >= 1.0 { 0.0 } else { wrapped }
    };
    let x = ((wrap(uv[0]) * source.width as f32).floor() as u32).min(source.width - 1);
    let y = (((1.0 - wrap(uv[1])) * source.height as f32).floor() as u32).min(source.height - 1);
    let offset = ((y * source.width + x) as usize) * channels;
    [
        source.pixels[offset],
        source.pixels[offset + 1],
        source.pixels[offset + 2],
        if channels == 4 {
            source.pixels[offset + 3]
        } else {
            u8::MAX
        },
    ]
}

fn write_item_icon_layer_v2(
    source: &TgaImageV1,
    model: &crate::model_ir::AuroraModelIrV1,
    width: u32,
    height: u32,
    requested_bounds: Option<ItemIconProjectionBoundsV1>,
) -> Result<(Vec<u8>, ItemIconProjectionBoundsV1, u64), ItemErrorV1> {
    if width == 0 || height == 0 {
        return Err(error(
            "ITEM-ICON-DIMENSIONS-INVALID",
            "options.iconSize",
            "item icon dimensions must both be greater than zero",
        ));
    }
    let geometry = item_icon_geometry_v1(model)?;
    let own_bounds = item_icon_projection_bounds_v1(&geometry)?;
    let bounds = requested_bounds.unwrap_or(own_bounds);
    validate_item_icon_projection_bounds_v1(bounds)?;
    let pixel_count = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            error(
                "ITEM-ICON-SIZE-OVERFLOW",
                "options.iconSize",
                "item icon pixel count exceeds the host address space",
            )
        })?;
    let mut pixels = vec![0_u8; pixel_count * 4];
    let mut depth = vec![f32::NEG_INFINITY; pixel_count];
    let margin = (width.min(height) as f32 * 0.06).max(1.0);
    let drawable_width = (width as f32 - 2.0 * margin).max(1.0);
    let drawable_height = (height as f32 - 2.0 * margin).max(1.0);
    let center = [
        (bounds.min[0] + bounds.max[0]) * 0.5,
        (bounds.min[1] + bounds.max[1]) * 0.5,
    ];
    let scale = (drawable_width / (bounds.max[0] - bounds.min[0]))
        .min(drawable_height / (bounds.max[1] - bounds.min[1]));
    let to_screen = |vertex: ItemIconVertexV1| {
        [
            width as f32 * 0.5 + (vertex.projected[0] - center[0]) * scale,
            height as f32 * 0.5 - (vertex.projected[1] - center[1]) * scale,
            vertex.projected[2],
        ]
    };

    for (segment_index, segment) in model.segments.iter().enumerate() {
        let vertices = &geometry[segment_index];
        for triangle in segment.indices.chunks_exact(3) {
            let indices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            if indices.iter().any(|&index| index >= vertices.len()) {
                return Err(error(
                    "ITEM-ICON-GEOMETRY-INVALID",
                    format!("model.segments[{segment_index}].indices"),
                    "icon triangle index exceeds the segment vertex array",
                ));
            }
            let source_vertices = [
                vertices[indices[0]],
                vertices[indices[1]],
                vertices[indices[2]],
            ];
            let screen = source_vertices.map(to_screen);
            let screen_2d = [
                [screen[0][0], screen[0][1]],
                [screen[1][0], screen[1][1]],
                [screen[2][0], screen[2][1]],
            ];
            let area = item_icon_edge_v1(screen_2d[0], screen_2d[1], screen_2d[2]);
            if !area.is_finite() || area.abs() <= 1.0e-8 {
                continue;
            }
            let min_x = screen
                .iter()
                .map(|vertex| vertex[0])
                .fold(f32::INFINITY, f32::min)
                .floor()
                .max(0.0) as u32;
            let max_x = screen
                .iter()
                .map(|vertex| vertex[0])
                .fold(f32::NEG_INFINITY, f32::max)
                .ceil()
                .min(width.saturating_sub(1) as f32) as u32;
            let min_y = screen
                .iter()
                .map(|vertex| vertex[1])
                .fold(f32::INFINITY, f32::min)
                .floor()
                .max(0.0) as u32;
            let max_y = screen
                .iter()
                .map(|vertex| vertex[1])
                .fold(f32::NEG_INFINITY, f32::max)
                .ceil()
                .min(height.saturating_sub(1) as f32) as u32;
            if min_x > max_x || min_y > max_y {
                continue;
            }
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let point = [x as f32 + 0.5, y as f32 + 0.5];
                    let weights = [
                        item_icon_edge_v1(screen_2d[1], screen_2d[2], point) / area,
                        item_icon_edge_v1(screen_2d[2], screen_2d[0], point) / area,
                        item_icon_edge_v1(screen_2d[0], screen_2d[1], point) / area,
                    ];
                    if weights.iter().any(|weight| *weight < -1.0e-5) {
                        continue;
                    }
                    let candidate_depth = weights[0] * screen[0][2]
                        + weights[1] * screen[1][2]
                        + weights[2] * screen[2][2];
                    let pixel_index = (y * width + x) as usize;
                    if candidate_depth <= depth[pixel_index] {
                        continue;
                    }
                    let uv = [
                        weights[0] * source_vertices[0].uv[0]
                            + weights[1] * source_vertices[1].uv[0]
                            + weights[2] * source_vertices[2].uv[0],
                        weights[0] * source_vertices[0].uv[1]
                            + weights[1] * source_vertices[1].uv[1]
                            + weights[2] * source_vertices[2].uv[1],
                    ];
                    let color = sample_item_icon_texture_v1(source, uv);
                    if color[3] == 0 {
                        continue;
                    }
                    depth[pixel_index] = candidate_depth;
                    // The shared TGA writer emits descriptor 8 (bottom-left
                    // origin), while raster-space y=0 is the top scanline.
                    // Store rows bottom-up so native TGA readers recover the
                    // same orientation shown by the authoring projection.
                    let output_pixel_index = ((height - y - 1) * width + x) as usize;
                    pixels[output_pixel_index * 4..output_pixel_index * 4 + 4]
                        .copy_from_slice(&color);
                }
            }
        }
    }
    let opaque_pixel_count = pixels.chunks_exact(4).filter(|pixel| pixel[3] != 0).count() as u64;
    if opaque_pixel_count == 0 {
        return Err(error(
            "ITEM-ICON-SILHOUETTE-EMPTY",
            "model.segments",
            "geometry-derived icon rasterization produced no visible pixels",
        ));
    }
    let payload = write_tga_v1(
        &TgaImageV1 {
            schema_version: TGA_SCHEMA_VERSION,
            width,
            height,
            pixel_format: TgaPixelFormatV1::Rgba8,
            pixels,
        },
        &TgaWriterOptionsV1::default(),
    )
    .map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.path,
            source.message,
        )
    })?
    .payload;
    Ok((payload, bounds, opaque_pixel_count))
}

fn item_icon_alpha_bounds_v3(
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Option<(u64, [u32; 2], [u32; 2])> {
    let mut count = 0_u64;
    let mut min = [u32::MAX; 2];
    let mut max = [0_u32; 2];
    for (index, pixel) in pixels.chunks_exact(4).enumerate() {
        if pixel[3] == 0 {
            continue;
        }
        let index = index as u32;
        let x = index % width;
        let y = index / width;
        if y >= height {
            continue;
        }
        count += 1;
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        max[0] = max[0].max(x + 1);
        max[1] = max[1].max(y + 1);
    }
    (count > 0).then_some((count, min, max))
}

/// Reproduces the native Bottom/Middle/Top icon resource contract offline.
/// Every layer must already use the same full canvas; this function never
/// recenters or rescales an individual part.
pub fn validate_item_modeltype2_icon_layers_v3(
    inputs: &[ItemIconLayerInputV3<'_>],
    expected_width: u32,
    expected_height: u32,
    layout_profile: &str,
) -> Result<ItemIconCompositeReportV3, ItemErrorV1> {
    if inputs.len() != 3 || expected_width == 0 || expected_height == 0 {
        return Err(error(
            "ITEM-ICON-V3-LAYER-SET-INVALID",
            "inputs",
            "ModelType 2 icon conformance requires exactly three non-zero full-canvas layers",
        ));
    }
    if !matches!(
        layout_profile,
        "LONG_VERTICAL_V1" | "LONG_VERTICAL_PART_ORDER_V2"
    ) {
        return Err(error(
            "ITEM-ICON-V3-LAYOUT-UNSUPPORTED",
            "layoutProfile",
            "the native icon conformance route supports LONG_VERTICAL_V1 and LONG_VERTICAL_PART_ORDER_V2",
        ));
    }
    let expected_fields = ["ModelPart1", "ModelPart2", "ModelPart3"];
    let pixel_count = usize::try_from(u64::from(expected_width) * u64::from(expected_height))
        .map_err(|_| {
            error(
                "ITEM-ICON-V3-DIMENSIONS-INVALID",
                "expectedDimensions",
                "icon canvas does not fit the host address space",
            )
        })?;
    let mut composite = vec![0_u8; pixel_count * 4];
    let mut layers = Vec::with_capacity(3);
    for (index, input) in inputs.iter().enumerate() {
        if input.field != expected_fields[index] {
            return Err(error(
                "ITEM-ICON-V3-LAYER-ORDER-INVALID",
                format!("inputs[{index}].field"),
                "icon layers must be supplied in Bottom, Middle, Top order",
            ));
        }
        validate_resref(input.icon_resref, &format!("inputs[{index}].iconResref"))?;
        let image = read_tga_image_v1(input.payload).map_err(|message| {
            error(
                "ITEM-ICON-V3-TGA-INVALID",
                format!("inputs[{index}].payload"),
                message,
            )
        })?;
        if image.width != expected_width
            || image.height != expected_height
            || image.pixel_format != TgaPixelFormatV1::Rgba8
        {
            return Err(error(
                "ITEM-ICON-V3-CANVAS-MISMATCH",
                format!("inputs[{index}].payload"),
                "every native icon layer must be RGBA8 and use the exact shared canvas dimensions",
            ));
        }
        let (opaque_pixel_count, bounds_min, bounds_max_exclusive) =
            item_icon_alpha_bounds_v3(&image.pixels, image.width, image.height).ok_or_else(
                || {
                    error(
                        "ITEM-ICON-V3-LAYER-EMPTY",
                        format!("inputs[{index}].payload"),
                        "a native icon layer cannot be completely transparent",
                    )
                },
            )?;
        layers.push(ItemIconLayerSummaryV3 {
            field: input.field.to_owned(),
            icon_resref: input.icon_resref.to_owned(),
            opaque_pixel_count,
            bounds_min,
            bounds_max_exclusive,
            sha256: item_payload_sha256_v1(input.payload),
        });
        for (target, source) in composite
            .chunks_exact_mut(4)
            .zip(image.pixels.chunks_exact(4))
        {
            let source_alpha = u32::from(source[3]);
            if source_alpha == 0 {
                continue;
            }
            let target_alpha = u32::from(target[3]);
            let inverse = 255 - source_alpha;
            let output_alpha = source_alpha + (target_alpha * inverse + 127) / 255;
            for channel in 0..3 {
                let source_premultiplied = u32::from(source[channel]) * source_alpha;
                let target_premultiplied =
                    u32::from(target[channel]) * target_alpha * inverse / 255;
                target[channel] = if output_alpha == 0 {
                    0
                } else {
                    ((source_premultiplied + target_premultiplied) / output_alpha).min(255) as u8
                };
            }
            target[3] = output_alpha.min(255) as u8;
        }
    }
    let (opaque_pixel_count, bounds_min, bounds_max_exclusive) =
        item_icon_alpha_bounds_v3(&composite, expected_width, expected_height).ok_or_else(
            || {
                error(
                    "ITEM-ICON-V3-COMPOSITE-EMPTY",
                    "inputs",
                    "Bottom/Middle/Top icon composition produced no visible pixels",
                )
            },
        )?;
    let bounds_width = bounds_max_exclusive[0] - bounds_min[0];
    let bounds_height = bounds_max_exclusive[1] - bounds_min[1];
    let axial_fill_ratio = bounds_height as f32 / expected_height as f32;
    let occupied_fill_ratio = opaque_pixel_count as f32 / (bounds_width * bounds_height) as f32;
    if axial_fill_ratio < 0.65 || opaque_pixel_count < u64::from(expected_height) {
        return Err(error(
            "ITEM-ICON-V3-INCOMPLETE-SILHOUETTE",
            "composite",
            format!(
                "LONG_VERTICAL_V1 requires at least 65% axial canvas coverage and {} visible pixels; got {:.3} and {opaque_pixel_count}",
                expected_height, axial_fill_ratio,
            ),
        ));
    }
    if bounds_min[1] == 0 || bounds_max_exclusive[1] == expected_height {
        return Err(error(
            "ITEM-ICON-V3-CLIPPED",
            "composite",
            "LONG_VERTICAL_V1 icon touches the top or bottom canvas boundary",
        ));
    }
    let part_order_status = if layout_profile == "LONG_VERTICAL_PART_ORDER_V2" {
        let center_y = |layer: &ItemIconLayerSummaryV3| {
            (layer.bounds_min[1] as f32 + layer.bounds_max_exclusive[1] as f32) * 0.5
        };
        let bottom_center = center_y(&layers[0]);
        let middle_center = center_y(&layers[1]);
        let top_center = center_y(&layers[2]);
        if !(top_center < middle_center && middle_center < bottom_center) {
            return Err(error(
                "ITEM-ICON-V3-PART-ORDER-INVERTED",
                "layers",
                format!(
                    "LONG_VERTICAL_PART_ORDER_V2 requires Top above Middle above Bottom in decoded TGA coordinates; centers are Top={top_center:.3}, Middle={middle_center:.3}, Bottom={bottom_center:.3}"
                ),
            ));
        }
        "PASSED"
    } else {
        "NOT_ENFORCED"
    };
    Ok(ItemIconCompositeReportV3 {
        schema_version: 3,
        algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3".to_owned(),
        status: "PASSED".to_owned(),
        layout_profile: layout_profile.to_owned(),
        width: expected_width,
        height: expected_height,
        layer_count: layers.len(),
        layers,
        opaque_pixel_count,
        bounds_min,
        bounds_max_exclusive,
        axial_fill_ratio,
        occupied_fill_ratio,
        part_order_status: part_order_status.to_owned(),
        composite_rgba_sha256: item_payload_sha256_v1(&composite),
    })
}

pub fn build_meshy_item_part_v1(
    source_glb: &[u8],
    model_resref: &str,
    texture_resref: &str,
    transform: ItemPartTransformV1,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    build_meshy_item_part_with_options_v1(
        source_glb,
        model_resref,
        texture_resref,
        &ItemPartBuildOptionsV1 {
            transform,
            ..ItemPartBuildOptionsV1::default()
        },
    )
}

pub fn build_meshy_item_part_with_options_v1(
    source_glb: &[u8],
    model_resref: &str,
    texture_resref: &str,
    options: &ItemPartBuildOptionsV1,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    build_meshy_item_part_with_internal_options_v1(
        source_glb,
        model_resref,
        texture_resref,
        &ItemPartBuildInternalOptionsV1 {
            schema_version: options.schema_version,
            transform: options.transform,
            source_node: options.source_node.clone(),
            texture_encoding: options.texture_encoding,
            icon_size: options.icon_size,
            icon_projection_bounds: None,
            weapon_color: None,
            target_space_scale_xyz: item_unit_scale_xyz_v1(),
            geometry_derived_icon: false,
            aurora_composer_v2: false,
        },
    )
}

pub fn build_meshy_item_part_with_options_v2(
    source_glb: &[u8],
    model_resref: &str,
    texture_resref: &str,
    options: &ItemPartBuildOptionsV2,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    build_meshy_item_part_with_internal_options_v1(
        source_glb,
        model_resref,
        texture_resref,
        &ItemPartBuildInternalOptionsV1 {
            schema_version: options.schema_version,
            transform: options.transform,
            source_node: options.source_node.clone(),
            texture_encoding: options.texture_encoding,
            icon_size: options.icon_size,
            icon_projection_bounds: options.icon_projection_bounds,
            weapon_color: options.weapon_color,
            target_space_scale_xyz: options.target_space_scale_xyz,
            geometry_derived_icon: true,
            aurora_composer_v2: false,
        },
    )
}

/// Builds one ModelType 2 part using the node/controller ownership observed in
/// retail Aurora weapon parts. Geometry normalization is baked into mesh
/// streams; the controllerless model root never carries the assembly transform.
pub fn build_meshy_item_part_with_options_v3(
    source_glb: &[u8],
    model_resref: &str,
    texture_resref: &str,
    options: &ItemPartBuildOptionsV2,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    build_meshy_item_part_with_internal_options_v1(
        source_glb,
        model_resref,
        texture_resref,
        &ItemPartBuildInternalOptionsV1 {
            schema_version: options.schema_version,
            transform: options.transform,
            source_node: options.source_node.clone(),
            texture_encoding: options.texture_encoding,
            icon_size: options.icon_size,
            icon_projection_bounds: options.icon_projection_bounds,
            weapon_color: options.weapon_color,
            target_space_scale_xyz: options.target_space_scale_xyz,
            geometry_derived_icon: true,
            aurora_composer_v2: true,
        },
    )
}

/// Builds a static flying projectile and rejects an authored transform whose
/// declared nose direction does not become Aurora +Y. This makes orientation
/// an explicit authoring fact instead of a bounds-based guess.
pub fn build_meshy_ranged_projectile_v1(
    source_glb: &[u8],
    model_resref: &str,
    texture_resref: &str,
    options: &ItemProjectileBuildOptionsV1,
) -> Result<ItemProjectileArtifactV1, ItemErrorV1> {
    if options.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-PROJECTILE-SCHEMA-INVALID",
            "options.schemaVersion",
            "projectile build options schemaVersion must be 1",
        ));
    }
    validate_item_part_transform_v1(options.transform)?;
    let matrix = item_part_rigid_matrix_v1(options.transform)?;
    let source = options.source_forward_axis.vector();
    let mut transformed = [
        matrix[0] * source[0] + matrix[4] * source[1] + matrix[8] * source[2],
        matrix[1] * source[0] + matrix[5] * source[1] + matrix[9] * source[2],
        matrix[2] * source[0] + matrix[6] * source[1] + matrix[10] * source[2],
    ];
    let length = transformed
        .into_iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if !length.is_finite() || length <= 1.0e-6 {
        return Err(error(
            "ITEM-PROJECTILE-FORWARD-AXIS-INVALID",
            "options.sourceForwardAxis",
            "projectile forward direction cannot be normalized",
        ));
    }
    for value in &mut transformed {
        *value /= length;
    }
    if transformed[0].abs() > 1.0e-4
        || (transformed[1] - 1.0).abs() > 1.0e-4
        || transformed[2].abs() > 1.0e-4
    {
        return Err(error(
            "ITEM-PROJECTILE-FORWARD-AXIS-MISMATCH",
            "options.transform.rotationXyzw",
            format!(
                "declared source forward axis resolves to [{:.6}, {:.6}, {:.6}], expected Aurora +Y",
                transformed[0], transformed[1], transformed[2]
            ),
        ));
    }

    let part = build_meshy_item_part_with_internal_options_v1(
        source_glb,
        model_resref,
        texture_resref,
        &ItemPartBuildInternalOptionsV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            transform: options.transform,
            source_node: options.source_node.clone(),
            texture_encoding: ItemPartTextureEncodingV1::DirectColor,
            icon_size: None,
            icon_projection_bounds: None,
            weapon_color: None,
            target_space_scale_xyz: item_unit_scale_xyz_v1(),
            geometry_derived_icon: false,
            aurora_composer_v2: false,
        },
    )?;
    if part.icon_payload.is_some() || part.report.texture_format != "TGA_V1" {
        return Err(error(
            "ITEM-PROJECTILE-OUTPUT-PROFILE-MISMATCH",
            "projectile",
            "ranged projectile must emit one binary MDL and one direct-color TGA without an inventory icon",
        ));
    }

    Ok(ItemProjectileArtifactV1 {
        mdl_payload: part.mdl_payload,
        texture_payload: part.texture_payload,
        report: ItemProjectileBuildReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            profile: "RANGED_PROJECTILE_STATIC_V1".to_owned(),
            model_resref: part.report.model_resref,
            texture_resref: part.report.texture_resref,
            source_forward_axis: options.source_forward_axis,
            aurora_forward_axis: ItemProjectileAxisV1::PositiveY,
            transformed_forward: transformed,
            orientation_status: "PASS".to_owned(),
            source_sha256: part.report.source_sha256,
            triangle_count: part.report.triangle_count,
            degenerate_triangle_count_removed: part.report.degenerate_triangle_count_removed,
            stream_count: part.report.stream_count,
            transform: part.report.transform,
            mdl_sha256: part.report.mdl_sha256,
            texture_sha256: part.report.texture_sha256,
            semantic_readback_status: part.report.semantic_readback_status,
        },
        readback: part.readback,
    })
}

#[derive(Clone, Debug)]
struct ItemPartBuildInternalOptionsV1 {
    schema_version: u32,
    transform: ItemPartTransformV1,
    source_node: Option<String>,
    texture_encoding: ItemPartTextureEncodingV1,
    icon_size: Option<[u32; 2]>,
    icon_projection_bounds: Option<ItemIconProjectionBoundsV1>,
    weapon_color: Option<u8>,
    target_space_scale_xyz: [f32; 3],
    geometry_derived_icon: bool,
    aurora_composer_v2: bool,
}

struct PreparedItemPartV1 {
    ingest: crate::glb::GlbIngestResult,
    model: crate::model_ir::AuroraModelIrV1,
    source_node: Option<String>,
    triangle_count: usize,
    degenerate_triangle_count_removed: usize,
}

/// Removes only faces that become non-finite or exactly degenerate after a
/// user-authored uniform scale is baked. The Item writer uses exact finite
/// face-plane semantics so valid small Meshy faces survive scale normalization.
/// Any unavoidable removal is deterministic and reported by the Item build.
fn sanitize_item_model_face_planes_v1(
    model: &mut crate::model_ir::AuroraModelIrV1,
) -> Result<usize, ItemErrorV1> {
    let mut removed_total = 0_usize;
    for (segment_index, segment) in model.segments.iter_mut().enumerate() {
        if !segment.indices.len().is_multiple_of(3)
            || (!segment.face_surface_ids.is_empty()
                && segment.face_surface_ids.len() != segment.indices.len() / 3)
        {
            return Err(error(
                "ITEM-PART-FACE-PLANE-SOURCE-INVALID",
                format!("model.segments[{segment_index}].indices"),
                "item segment indices or face surface metadata are not triangle-aligned",
            ));
        }
        let has_surface_ids = !segment.face_surface_ids.is_empty();
        let mut retained_indices = Vec::with_capacity(segment.indices.len());
        let mut retained_surface_ids = has_surface_ids
            .then(|| Vec::with_capacity(segment.face_surface_ids.len()))
            .unwrap_or_default();
        for (triangle_index, triangle) in segment.indices.chunks_exact(3).enumerate() {
            let mut vertices = [[0.0_f32; 3]; 3];
            for (corner, source_index) in triangle.iter().copied().enumerate() {
                let index = usize::try_from(source_index).map_err(|_| {
                    error(
                        "ITEM-PART-FACE-PLANE-INDEX",
                        format!("model.segments[{segment_index}].indices"),
                        "item triangle index does not fit this platform",
                    )
                })?;
                vertices[corner] = *segment.positions.get(index).ok_or_else(|| {
                    error(
                        "ITEM-PART-FACE-PLANE-INDEX",
                        format!("model.segments[{segment_index}].indices[{triangle_index}]"),
                        "item triangle index escapes the position stream",
                    )
                })?;
            }
            let edge_ab = [
                f64::from(vertices[1][0]) - f64::from(vertices[0][0]),
                f64::from(vertices[1][1]) - f64::from(vertices[0][1]),
                f64::from(vertices[1][2]) - f64::from(vertices[0][2]),
            ];
            let edge_ac = [
                f64::from(vertices[2][0]) - f64::from(vertices[0][0]),
                f64::from(vertices[2][1]) - f64::from(vertices[0][1]),
                f64::from(vertices[2][2]) - f64::from(vertices[0][2]),
            ];
            let cross = [
                edge_ab[1] * edge_ac[2] - edge_ab[2] * edge_ac[1],
                edge_ab[2] * edge_ac[0] - edge_ab[0] * edge_ac[2],
                edge_ab[0] * edge_ac[1] - edge_ab[1] * edge_ac[0],
            ];
            let length = (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt();
            if length.is_finite() && length > 0.0 {
                retained_indices.extend_from_slice(triangle);
                if has_surface_ids {
                    retained_surface_ids.push(segment.face_surface_ids[triangle_index]);
                }
            } else {
                removed_total += 1;
            }
        }
        if retained_indices.is_empty() {
            return Err(error(
                "ITEM-PART-FACE-PLANE-EMPTY",
                format!("model.segments[{segment_index}].indices"),
                "item segment contains no Aurora-safe face after post-transform sanitation",
            ));
        }
        segment.indices = retained_indices;
        if has_surface_ids {
            segment.face_surface_ids = retained_surface_ids;
        }
    }
    Ok(removed_total)
}

fn normalize_item_geometry_for_aurora_composer_v2(
    model: &mut crate::model_ir::AuroraModelIrV1,
    root_index: usize,
    composed_root: [f32; 16],
    target_space_scale_xyz: [f32; 3],
) -> Result<(), ItemErrorV1> {
    if model.nodes.len() != 1 {
        return Err(error(
            "ITEM-PART-COMPOSER-HIERARCHY-INVALID",
            "model.nodes",
            "ModelType 2 output requires one controllerless model root and direct Trimesh children",
        ));
    }
    let root_id = model.nodes[root_index].id;
    if let Some((segment_index, _)) = model
        .segments
        .iter()
        .enumerate()
        .find(|(_, segment)| segment.parent_node_id != root_id)
    {
        return Err(error(
            "ITEM-PART-COMPOSER-HIERARCHY-INVALID",
            format!("model.segments[{segment_index}].parentNodeId"),
            "every generated Item Trimesh must be a direct child of the model root",
        ));
    }
    if target_space_scale_xyz
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(error(
            "ITEM-PART-TARGET-SPACE-SCALE-INVALID",
            "options.targetSpaceScaleXyz",
            "target-space scale must contain three positive finite components",
        ));
    }
    let columns = [
        [composed_root[0], composed_root[1], composed_root[2]],
        [composed_root[4], composed_root[5], composed_root[6]],
        [composed_root[8], composed_root[9], composed_root[10]],
    ];
    let dot = |first: [f32; 3], second: [f32; 3]| {
        first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
    };
    let lengths = columns.map(|column| dot(column, column).sqrt());
    let determinant = columns[0][0]
        * (columns[1][1] * columns[2][2] - columns[2][1] * columns[1][2])
        - columns[1][0] * (columns[0][1] * columns[2][2] - columns[2][1] * columns[0][2])
        + columns[2][0] * (columns[0][1] * columns[1][2] - columns[1][1] * columns[0][2]);
    if composed_root.iter().any(|value| !value.is_finite())
        || lengths.iter().any(|length| (*length - 1.0).abs() > 1.0e-4)
        || dot(columns[0], columns[1]).abs() > 1.0e-4
        || dot(columns[0], columns[2]).abs() > 1.0e-4
        || dot(columns[1], columns[2]).abs() > 1.0e-4
        || (determinant - 1.0).abs() > 1.0e-4
    {
        return Err(error(
            "ITEM-PART-COMPOSER-NORMALIZATION-NONRIGID",
            "model.nodes[0].bindLocalMatrix",
            "Item composer geometry normalization accepts only a finite proper rigid basis after uniform scale is baked",
        ));
    }
    let rotate = |value: [f32; 3]| {
        [
            composed_root[0] * value[0] + composed_root[4] * value[1] + composed_root[8] * value[2],
            composed_root[1] * value[0] + composed_root[5] * value[1] + composed_root[9] * value[2],
            composed_root[2] * value[0]
                + composed_root[6] * value[1]
                + composed_root[10] * value[2],
        ]
    };
    let normalize = |value: [f32; 3]| -> Result<[f32; 3], ItemErrorV1> {
        let length = dot(value, value).sqrt();
        if !length.is_finite() || length <= 1.0e-8 {
            return Err(error(
                "ITEM-PART-COMPOSER-DIRECTION-INVALID",
                "model.segments",
                "rotated normal or tangent has no finite direction",
            ));
        }
        Ok(value.map(|component| component / length))
    };
    for segment in &mut model.segments {
        for position in &mut segment.positions {
            let rotated = rotate(*position);
            *position = std::array::from_fn(|axis| rotated[axis] * target_space_scale_xyz[axis]);
        }
        for normal in &mut segment.normals {
            let rotated = rotate(*normal);
            *normal = normalize(std::array::from_fn(|axis| {
                rotated[axis] / target_space_scale_xyz[axis]
            }))?;
        }
        if let Some(tangents) = &mut segment.tangents {
            for (tangent, normal) in tangents.iter_mut().zip(&segment.normals) {
                let rotated = rotate([tangent[0], tangent[1], tangent[2]]);
                let scaled =
                    std::array::from_fn(|axis| rotated[axis] * target_space_scale_xyz[axis]);
                let projection = dot(scaled, *normal);
                let orthogonal =
                    std::array::from_fn(|axis| scaled[axis] - projection * normal[axis]);
                let normalized = normalize(orthogonal)?;
                tangent[0] = normalized[0];
                tangent[1] = normalized[1];
                tangent[2] = normalized[2];
            }
        }
    }
    model.nodes[root_index].bind_local_matrix = [
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        composed_root[12],
        composed_root[13],
        composed_root[14],
        1.0,
    ];
    model.profile_id = "ITEM_PART_AURORA_COMPOSER_V2".to_owned();
    Ok(())
}

fn selected_item_scene_material_ids_v1(
    ingest: &crate::glb::GlbIngestResult,
) -> Result<BTreeSet<Option<u32>>, ItemErrorV1> {
    let scene_id = ingest.ir.default_scene_id.ok_or_else(|| {
        error(
            "ITEM-SOURCE-NODE-SCENE-MISSING",
            "source.ir.defaultSceneId",
            "Item material validation requires a default GLB scene",
        )
    })?;
    let scene = ingest
        .ir
        .scenes
        .iter()
        .find(|scene| scene.id == scene_id)
        .ok_or_else(|| {
            error(
                "ITEM-SOURCE-NODE-SCENE-MISSING",
                "source.ir.defaultSceneId",
                "default GLB scene does not exist",
            )
        })?;
    let mut pending = scene.root_node_ids.clone();
    let mut visited = BTreeSet::new();
    let mut mesh_ids = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id) {
            continue;
        }
        let node = ingest
            .ir
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| {
                error(
                    "ITEM-SOURCE-NODE-HIERARCHY-INVALID",
                    "source.ir.nodes",
                    format!("selected scene references missing node {node_id}"),
                )
            })?;
        pending.extend(node.child_ids.iter().copied());
        if let Some(mesh_id) = node.mesh_id {
            mesh_ids.insert(mesh_id);
        }
    }
    Ok(ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| mesh_ids.contains(&primitive.source_mesh_id))
        .map(|primitive| primitive.material_id)
        .collect())
}

fn prepare_meshy_item_part_v1(
    source_glb: &[u8],
    model_resref: &str,
    options: &ItemPartBuildInternalOptionsV1,
) -> Result<PreparedItemPartV1, ItemErrorV1> {
    let limits = static_placeable_glb_limits_v1();
    let mut ingest = ingest_glb(source_glb, &limits).map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            source.message,
        )
    })?;
    let source_node = select_item_source_node_v1(&mut ingest, options.source_node.as_deref())?;
    let used_material_ids = selected_item_scene_material_ids_v1(&ingest)?;
    if used_material_ids.len() > 1 {
        return Err(error(
            "ITEM-PART-MULTI-MATERIAL-UNSUPPORTED",
            "model.materialSourceBindings",
            format!(
                "this Item part uses {} source materials, but the current artifact contract emits exactly one texture resource; split the source into one material per part",
                used_material_ids.len()
            ),
        ));
    }
    sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut ingest).map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest).map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let conversion = convert_profile_a(&ingest, &rig, &static_placeable_profile_a_options_v1())
        .map_err(|source| {
            error(
                &format!("ITEM-{}", source.code),
                source.path,
                source.message,
            )
        })?;
    if !conversion.report.conversion_eligible {
        return Err(error(
            "ITEM-PART-PROFILE-INELIGIBLE",
            "conversion.report",
            "static item part did not pass the shared Profile A gates",
        ));
    }
    let mut model = conversion.creature.ok_or_else(|| {
        error(
            "ITEM-PART-PROFILE-INELIGIBLE",
            "conversion.model",
            "eligible static item part has no common model IR",
        )
    })?;
    let root_indices = model
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    if root_indices.len() != 1 {
        return Err(error(
            "ITEM-PART-HIERARCHY-INVALID",
            "model.nodes",
            "an item part requires exactly one model root",
        ));
    }
    let root_index = root_indices[0];
    model.nodes[root_index]
        .name
        .clone_from(&model_resref.to_owned());
    if options.geometry_derived_icon {
        bake_uniform_scale_v2(
            &mut model,
            options.transform.uniform_scale,
            options.transform.pivot,
        );
    } else {
        bake_uniform_scale_v1(
            &mut model,
            options.transform.uniform_scale,
            options.transform.pivot,
        );
    }
    let authored = item_part_rigid_matrix_v1(options.transform)?;
    let composed_root = multiply_matrix_v1(authored, model.nodes[root_index].bind_local_matrix);
    if options.aurora_composer_v2 {
        normalize_item_geometry_for_aurora_composer_v2(
            &mut model,
            root_index,
            composed_root,
            options.target_space_scale_xyz,
        )?;
    } else {
        model.nodes[root_index].bind_local_matrix = composed_root;
    }

    let degenerate_triangle_count_removed = sanitize_item_model_face_planes_v1(&mut model)?;
    let triangle_count = validate_model_triangle_budget_v1(&model).map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    Ok(PreparedItemPartV1 {
        ingest,
        model,
        source_node,
        triangle_count,
        degenerate_triangle_count_removed,
    })
}

#[derive(Clone, Copy)]
struct ItemSeamTriangleV1 {
    vertices: [[f64; 3]; 3],
    bounds: ItemSeamBoundsV1,
    centroid: [f64; 3],
}

#[derive(Clone, Copy)]
struct ItemSeamBoundsV1 {
    min: [f64; 3],
    max: [f64; 3],
}

enum ItemSeamBvhV1 {
    Leaf {
        bounds: ItemSeamBoundsV1,
        triangles: Vec<usize>,
    },
    Branch {
        bounds: ItemSeamBoundsV1,
        left: Box<ItemSeamBvhV1>,
        right: Box<ItemSeamBvhV1>,
    },
}

impl ItemSeamBvhV1 {
    fn bounds(&self) -> ItemSeamBoundsV1 {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => *bounds,
        }
    }
}

fn item_seam_bounds_union_v1(
    first: ItemSeamBoundsV1,
    second: ItemSeamBoundsV1,
) -> ItemSeamBoundsV1 {
    let mut output = first;
    for axis in 0..3 {
        output.min[axis] = output.min[axis].min(second.min[axis]);
        output.max[axis] = output.max[axis].max(second.max[axis]);
    }
    output
}

fn item_seam_bounds_distance_squared_v1(first: ItemSeamBoundsV1, second: ItemSeamBoundsV1) -> f64 {
    (0..3)
        .map(|axis| {
            let separation = (first.min[axis] - second.max[axis])
                .max(second.min[axis] - first.max[axis])
                .max(0.0);
            separation * separation
        })
        .sum()
}

fn build_item_seam_bvh_v1(
    triangles: &[ItemSeamTriangleV1],
    mut indices: Vec<usize>,
) -> ItemSeamBvhV1 {
    let bounds = indices
        .iter()
        .map(|&index| triangles[index].bounds)
        .reduce(item_seam_bounds_union_v1)
        .expect("item seam BVH requires at least one triangle");
    if indices.len() <= 8 {
        return ItemSeamBvhV1::Leaf {
            bounds,
            triangles: indices,
        };
    }
    let mut centroid_min = [f64::INFINITY; 3];
    let mut centroid_max = [f64::NEG_INFINITY; 3];
    for &index in &indices {
        for axis in 0..3 {
            centroid_min[axis] = centroid_min[axis].min(triangles[index].centroid[axis]);
            centroid_max[axis] = centroid_max[axis].max(triangles[index].centroid[axis]);
        }
    }
    let axis = (0..3)
        .max_by(|&first, &second| {
            (centroid_max[first] - centroid_min[first])
                .total_cmp(&(centroid_max[second] - centroid_min[second]))
        })
        .unwrap_or(0);
    indices.sort_by(|&first, &second| {
        triangles[first].centroid[axis].total_cmp(&triangles[second].centroid[axis])
    });
    let right_indices = indices.split_off(indices.len() / 2);
    let left = build_item_seam_bvh_v1(triangles, indices);
    let right = build_item_seam_bvh_v1(triangles, right_indices);
    ItemSeamBvhV1::Branch {
        bounds,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn item_seam_sub3_v1(first: [f64; 3], second: [f64; 3]) -> [f64; 3] {
    [
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    ]
}

fn item_seam_add_scaled3_v1(origin: [f64; 3], delta: [f64; 3], scale: f64) -> [f64; 3] {
    [
        origin[0] + delta[0] * scale,
        origin[1] + delta[1] * scale,
        origin[2] + delta[2] * scale,
    ]
}

fn item_seam_dot3_v1(first: [f64; 3], second: [f64; 3]) -> f64 {
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}

fn item_seam_cross3_v1(first: [f64; 3], second: [f64; 3]) -> [f64; 3] {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

fn item_seam_length_squared_v1(value: [f64; 3]) -> f64 {
    item_seam_dot3_v1(value, value)
}

fn item_seam_point_triangle_distance_squared_v1(point: [f64; 3], triangle: [[f64; 3]; 3]) -> f64 {
    let a = triangle[0];
    let b = triangle[1];
    let c = triangle[2];
    let ab = item_seam_sub3_v1(b, a);
    let ac = item_seam_sub3_v1(c, a);
    let ap = item_seam_sub3_v1(point, a);
    let d1 = item_seam_dot3_v1(ab, ap);
    let d2 = item_seam_dot3_v1(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return item_seam_length_squared_v1(ap);
    }
    let bp = item_seam_sub3_v1(point, b);
    let d3 = item_seam_dot3_v1(ab, bp);
    let d4 = item_seam_dot3_v1(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return item_seam_length_squared_v1(bp);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let projection = item_seam_add_scaled3_v1(a, ab, d1 / (d1 - d3));
        return item_seam_length_squared_v1(item_seam_sub3_v1(point, projection));
    }
    let cp = item_seam_sub3_v1(point, c);
    let d5 = item_seam_dot3_v1(ab, cp);
    let d6 = item_seam_dot3_v1(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return item_seam_length_squared_v1(cp);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let projection = item_seam_add_scaled3_v1(a, ac, d2 / (d2 - d6));
        return item_seam_length_squared_v1(item_seam_sub3_v1(point, projection));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let edge = item_seam_sub3_v1(c, b);
        let projection = item_seam_add_scaled3_v1(b, edge, (d4 - d3) / ((d4 - d3) + (d5 - d6)));
        return item_seam_length_squared_v1(item_seam_sub3_v1(point, projection));
    }
    let denominator = 1.0 / (va + vb + vc);
    let v = vb * denominator;
    let w = vc * denominator;
    let projection = item_seam_add_scaled3_v1(item_seam_add_scaled3_v1(a, ab, v), ac, w);
    item_seam_length_squared_v1(item_seam_sub3_v1(point, projection))
}

fn item_seam_segment_distance_squared_v1(
    first_start: [f64; 3],
    first_end: [f64; 3],
    second_start: [f64; 3],
    second_end: [f64; 3],
) -> f64 {
    const EPSILON: f64 = 1.0e-15;
    let first_delta = item_seam_sub3_v1(first_end, first_start);
    let second_delta = item_seam_sub3_v1(second_end, second_start);
    let offset = item_seam_sub3_v1(first_start, second_start);
    let a = item_seam_dot3_v1(first_delta, first_delta);
    let e = item_seam_dot3_v1(second_delta, second_delta);
    let f = item_seam_dot3_v1(second_delta, offset);
    let (mut first_parameter, mut second_parameter);
    if a <= EPSILON && e <= EPSILON {
        return item_seam_length_squared_v1(offset);
    }
    if a <= EPSILON {
        first_parameter = 0.0;
        second_parameter = (f / e).clamp(0.0, 1.0);
    } else {
        let c = item_seam_dot3_v1(first_delta, offset);
        if e <= EPSILON {
            second_parameter = 0.0;
            first_parameter = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = item_seam_dot3_v1(first_delta, second_delta);
            let denominator = a * e - b * b;
            first_parameter = if denominator.abs() > EPSILON {
                ((b * f - c * e) / denominator).clamp(0.0, 1.0)
            } else {
                0.0
            };
            second_parameter = (b * first_parameter + f) / e;
            if second_parameter < 0.0 {
                second_parameter = 0.0;
                first_parameter = (-c / a).clamp(0.0, 1.0);
            } else if second_parameter > 1.0 {
                second_parameter = 1.0;
                first_parameter = ((b - c) / a).clamp(0.0, 1.0);
            }
        }
    }
    let first_point = item_seam_add_scaled3_v1(first_start, first_delta, first_parameter);
    let second_point = item_seam_add_scaled3_v1(second_start, second_delta, second_parameter);
    item_seam_length_squared_v1(item_seam_sub3_v1(first_point, second_point))
}

fn item_seam_segment_triangle_strict_intersection_v1(
    start: [f64; 3],
    end: [f64; 3],
    triangle: [[f64; 3]; 3],
    epsilon: f64,
) -> bool {
    let direction = item_seam_sub3_v1(end, start);
    let edge1 = item_seam_sub3_v1(triangle[1], triangle[0]);
    let edge2 = item_seam_sub3_v1(triangle[2], triangle[0]);
    let p = item_seam_cross3_v1(direction, edge2);
    let determinant = item_seam_dot3_v1(edge1, p);
    if determinant.abs() <= epsilon {
        return false;
    }
    let inverse = 1.0 / determinant;
    let offset = item_seam_sub3_v1(start, triangle[0]);
    let u = item_seam_dot3_v1(offset, p) * inverse;
    let q = item_seam_cross3_v1(offset, edge1);
    let v = item_seam_dot3_v1(direction, q) * inverse;
    let t = item_seam_dot3_v1(edge2, q) * inverse;
    t > epsilon && t < 1.0 - epsilon && u > epsilon && v > epsilon && u + v < 1.0 - epsilon
}

fn item_seam_triangles_strictly_intersect_v1(
    first: [[f64; 3]; 3],
    second: [[f64; 3]; 3],
    epsilon: f64,
) -> bool {
    let edges = [(0, 1), (1, 2), (2, 0)];
    edges.iter().any(|&(start, end)| {
        item_seam_segment_triangle_strict_intersection_v1(first[start], first[end], second, epsilon)
            || item_seam_segment_triangle_strict_intersection_v1(
                second[start],
                second[end],
                first,
                epsilon,
            )
    })
}

fn item_seam_triangle_distance_squared_v1(first: [[f64; 3]; 3], second: [[f64; 3]; 3]) -> f64 {
    let mut best = f64::INFINITY;
    for vertex in first {
        best = best.min(item_seam_point_triangle_distance_squared_v1(vertex, second));
    }
    for vertex in second {
        best = best.min(item_seam_point_triangle_distance_squared_v1(vertex, first));
    }
    let edges = [(0, 1), (1, 2), (2, 0)];
    for &(first_start, first_end) in &edges {
        for &(second_start, second_end) in &edges {
            best = best.min(item_seam_segment_distance_squared_v1(
                first[first_start],
                first[first_end],
                second[second_start],
                second[second_end],
            ));
        }
    }
    best
}

fn item_seam_point_bvh_distance_squared_v1(
    point: [f64; 3],
    bvh: &ItemSeamBvhV1,
    triangles: &[ItemSeamTriangleV1],
    best_distance_squared: &mut f64,
) {
    let point_bounds = ItemSeamBoundsV1 {
        min: point,
        max: point,
    };
    if item_seam_bounds_distance_squared_v1(point_bounds, bvh.bounds()) > *best_distance_squared {
        return;
    }
    match bvh {
        ItemSeamBvhV1::Leaf {
            triangles: indices, ..
        } => {
            for &index in indices {
                *best_distance_squared = (*best_distance_squared).min(
                    item_seam_point_triangle_distance_squared_v1(point, triangles[index].vertices),
                );
            }
        }
        ItemSeamBvhV1::Branch { left, right, .. } => {
            let mut children = [left.as_ref(), right.as_ref()];
            children.sort_by(|first, second| {
                item_seam_bounds_distance_squared_v1(point_bounds, first.bounds()).total_cmp(
                    &item_seam_bounds_distance_squared_v1(point_bounds, second.bounds()),
                )
            });
            for child in children {
                item_seam_point_bvh_distance_squared_v1(
                    point,
                    child,
                    triangles,
                    best_distance_squared,
                );
            }
        }
    }
}

fn item_seam_ray_bounds_intersects_v1(
    origin: [f64; 3],
    direction: [f64; 3],
    bounds: ItemSeamBoundsV1,
    epsilon: f64,
) -> bool {
    let mut near = 0.0_f64;
    let mut far = f64::INFINITY;
    for axis in 0..3 {
        if direction[axis].abs() <= epsilon {
            if origin[axis] < bounds.min[axis] - epsilon
                || origin[axis] > bounds.max[axis] + epsilon
            {
                return false;
            }
            continue;
        }
        let inverse = 1.0 / direction[axis];
        let mut axis_near = (bounds.min[axis] - origin[axis]) * inverse;
        let mut axis_far = (bounds.max[axis] - origin[axis]) * inverse;
        if axis_near > axis_far {
            std::mem::swap(&mut axis_near, &mut axis_far);
        }
        near = near.max(axis_near);
        far = far.min(axis_far);
        if far < near {
            return false;
        }
    }
    far > epsilon
}

fn item_seam_ray_triangle_hit_v1(
    origin: [f64; 3],
    direction: [f64; 3],
    triangle: [[f64; 3]; 3],
    epsilon: f64,
) -> bool {
    let edge1 = item_seam_sub3_v1(triangle[1], triangle[0]);
    let edge2 = item_seam_sub3_v1(triangle[2], triangle[0]);
    let p = item_seam_cross3_v1(direction, edge2);
    let determinant = item_seam_dot3_v1(edge1, p);
    if determinant.abs() <= epsilon {
        return false;
    }
    let inverse = 1.0 / determinant;
    let offset = item_seam_sub3_v1(origin, triangle[0]);
    let u = item_seam_dot3_v1(offset, p) * inverse;
    if u <= epsilon || u >= 1.0 - epsilon {
        return false;
    }
    let q = item_seam_cross3_v1(offset, edge1);
    let v = item_seam_dot3_v1(direction, q) * inverse;
    if v <= epsilon || u + v >= 1.0 - epsilon {
        return false;
    }
    item_seam_dot3_v1(edge2, q) * inverse > epsilon
}

fn item_seam_ray_bvh_hit_count_v1(
    origin: [f64; 3],
    direction: [f64; 3],
    bvh: &ItemSeamBvhV1,
    triangles: &[ItemSeamTriangleV1],
    epsilon: f64,
) -> usize {
    if !item_seam_ray_bounds_intersects_v1(origin, direction, bvh.bounds(), epsilon) {
        return 0;
    }
    match bvh {
        ItemSeamBvhV1::Leaf {
            triangles: indices, ..
        } => indices
            .iter()
            .filter(|&&index| {
                item_seam_ray_triangle_hit_v1(origin, direction, triangles[index].vertices, epsilon)
            })
            .count(),
        ItemSeamBvhV1::Branch { left, right, .. } => {
            item_seam_ray_bvh_hit_count_v1(origin, direction, left, triangles, epsilon)
                + item_seam_ray_bvh_hit_count_v1(origin, direction, right, triangles, epsilon)
        }
    }
}

fn item_seam_vertex_key_v1(vertex: [f64; 3]) -> [u64; 3] {
    vertex.map(|value| {
        let canonical = if value == 0.0 { 0.0 } else { value };
        canonical.to_bits()
    })
}

fn item_seam_is_watertight_v1(triangles: &[ItemSeamTriangleV1]) -> bool {
    let mut edges = BTreeMap::<([u64; 3], [u64; 3]), usize>::new();
    for triangle in triangles {
        let vertices = triangle.vertices.map(item_seam_vertex_key_v1);
        for (first, second) in [(0, 1), (1, 2), (2, 0)] {
            let edge = if vertices[first] <= vertices[second] {
                (vertices[first], vertices[second])
            } else {
                (vertices[second], vertices[first])
            };
            *edges.entry(edge).or_default() += 1;
        }
    }
    !edges.is_empty() && edges.values().all(|&count| count == 2)
}

fn item_seam_bounds_strictly_contains_point_v1(
    bounds: ItemSeamBoundsV1,
    point: [f64; 3],
    epsilon: f64,
) -> bool {
    (0..3).all(|axis| {
        point[axis] > bounds.min[axis] + epsilon && point[axis] < bounds.max[axis] - epsilon
    })
}

fn item_seam_closed_mesh_contains_point_v1(
    point: [f64; 3],
    target_triangles: &[ItemSeamTriangleV1],
    target_bvh: &ItemSeamBvhV1,
    epsilon: f64,
) -> bool {
    if !item_seam_bounds_strictly_contains_point_v1(target_bvh.bounds(), point, epsilon) {
        return false;
    }
    let mut surface_distance_squared = f64::INFINITY;
    item_seam_point_bvh_distance_squared_v1(
        point,
        target_bvh,
        target_triangles,
        &mut surface_distance_squared,
    );
    if surface_distance_squared <= epsilon * epsilon {
        return false;
    }
    const DIRECTIONS: [[f64; 3]; 3] = [
        [1.0, 0.371_390_676_354_103_7, 0.218_217_890_235_992_4],
        [0.312_347_523_777_212_1, 1.0, 0.517_638_090_205_041_5],
        [0.618_033_988_749_894_8, 0.271_828_182_845_904_5, 1.0],
    ];
    DIRECTIONS
        .iter()
        .filter(|&&direction| {
            item_seam_ray_bvh_hit_count_v1(point, direction, target_bvh, target_triangles, epsilon)
                % 2
                == 1
        })
        .count()
        >= 2
}

fn item_seam_closed_mesh_contains_surface_point_v1(
    source_triangles: &[ItemSeamTriangleV1],
    target_triangles: &[ItemSeamTriangleV1],
    target_bvh: &ItemSeamBvhV1,
    epsilon: f64,
) -> bool {
    if !item_seam_is_watertight_v1(target_triangles) {
        return false;
    }
    source_triangles.iter().any(|triangle| {
        item_seam_closed_mesh_contains_point_v1(
            triangle.centroid,
            target_triangles,
            target_bvh,
            epsilon,
        )
    })
}

fn item_seam_closed_meshes_share_interior_v1(
    first_triangles: &[ItemSeamTriangleV1],
    first_bvh: &ItemSeamBvhV1,
    second_triangles: &[ItemSeamTriangleV1],
    second_bvh: &ItemSeamBvhV1,
    epsilon: f64,
) -> bool {
    if !item_seam_is_watertight_v1(first_triangles) || !item_seam_is_watertight_v1(second_triangles)
    {
        return false;
    }
    let probe_offset = epsilon * 16.0;
    first_triangles
        .iter()
        .chain(second_triangles)
        .any(|triangle| {
            let edge1 = item_seam_sub3_v1(triangle.vertices[1], triangle.vertices[0]);
            let edge2 = item_seam_sub3_v1(triangle.vertices[2], triangle.vertices[0]);
            let normal = item_seam_cross3_v1(edge1, edge2);
            let length = item_seam_length_squared_v1(normal).sqrt();
            if length <= epsilon {
                return false;
            }
            let direction = normal.map(|value| value / length);
            [probe_offset, -probe_offset].into_iter().any(|offset| {
                let point = item_seam_add_scaled3_v1(triangle.centroid, direction, offset);
                item_seam_closed_mesh_contains_point_v1(point, first_triangles, first_bvh, epsilon)
                    && item_seam_closed_mesh_contains_point_v1(
                        point,
                        second_triangles,
                        second_bvh,
                        epsilon,
                    )
            })
        })
}

fn visit_item_seam_bvh_pair_v1(
    first_bvh: &ItemSeamBvhV1,
    first_triangles: &[ItemSeamTriangleV1],
    second_bvh: &ItemSeamBvhV1,
    second_triangles: &[ItemSeamTriangleV1],
    intersection_epsilon: f64,
    best_distance_squared: &mut f64,
    strict_intersection: &mut bool,
) {
    if *strict_intersection
        || item_seam_bounds_distance_squared_v1(first_bvh.bounds(), second_bvh.bounds())
            > *best_distance_squared
    {
        return;
    }
    match (first_bvh, second_bvh) {
        (
            ItemSeamBvhV1::Leaf {
                triangles: first_indices,
                ..
            },
            ItemSeamBvhV1::Leaf {
                triangles: second_indices,
                ..
            },
        ) => {
            for &first_index in first_indices {
                for &second_index in second_indices {
                    let first = first_triangles[first_index];
                    let second = second_triangles[second_index];
                    if item_seam_bounds_distance_squared_v1(first.bounds, second.bounds)
                        > *best_distance_squared
                    {
                        continue;
                    }
                    if item_seam_triangles_strictly_intersect_v1(
                        first.vertices,
                        second.vertices,
                        intersection_epsilon,
                    ) {
                        *strict_intersection = true;
                        *best_distance_squared = 0.0;
                        return;
                    }
                    *best_distance_squared = (*best_distance_squared).min(
                        item_seam_triangle_distance_squared_v1(first.vertices, second.vertices),
                    );
                }
            }
        }
        (ItemSeamBvhV1::Branch { left, right, .. }, ItemSeamBvhV1::Leaf { .. }) => {
            let mut children = [left.as_ref(), right.as_ref()];
            children.sort_by(|first, second| {
                item_seam_bounds_distance_squared_v1(first.bounds(), second_bvh.bounds()).total_cmp(
                    &item_seam_bounds_distance_squared_v1(second.bounds(), second_bvh.bounds()),
                )
            });
            for child in children {
                visit_item_seam_bvh_pair_v1(
                    child,
                    first_triangles,
                    second_bvh,
                    second_triangles,
                    intersection_epsilon,
                    best_distance_squared,
                    strict_intersection,
                );
            }
        }
        (ItemSeamBvhV1::Leaf { .. }, ItemSeamBvhV1::Branch { left, right, .. }) => {
            let mut children = [left.as_ref(), right.as_ref()];
            children.sort_by(|first, second| {
                item_seam_bounds_distance_squared_v1(first_bvh.bounds(), first.bounds()).total_cmp(
                    &item_seam_bounds_distance_squared_v1(first_bvh.bounds(), second.bounds()),
                )
            });
            for child in children {
                visit_item_seam_bvh_pair_v1(
                    first_bvh,
                    first_triangles,
                    child,
                    second_triangles,
                    intersection_epsilon,
                    best_distance_squared,
                    strict_intersection,
                );
            }
        }
        (
            ItemSeamBvhV1::Branch {
                left: first_left,
                right: first_right,
                ..
            },
            ItemSeamBvhV1::Branch {
                left: second_left,
                right: second_right,
                ..
            },
        ) => {
            let mut pairs = [
                (first_left.as_ref(), second_left.as_ref()),
                (first_left.as_ref(), second_right.as_ref()),
                (first_right.as_ref(), second_left.as_ref()),
                (first_right.as_ref(), second_right.as_ref()),
            ];
            pairs.sort_by(|(first_a, second_a), (first_b, second_b)| {
                item_seam_bounds_distance_squared_v1(first_a.bounds(), second_a.bounds()).total_cmp(
                    &item_seam_bounds_distance_squared_v1(first_b.bounds(), second_b.bounds()),
                )
            });
            for (first, second) in pairs {
                visit_item_seam_bvh_pair_v1(
                    first,
                    first_triangles,
                    second,
                    second_triangles,
                    intersection_epsilon,
                    best_distance_squared,
                    strict_intersection,
                );
            }
        }
    }
}

fn item_seam_model_triangles_v1(
    model: &crate::model_ir::AuroraModelIrV1,
) -> Result<Vec<ItemSeamTriangleV1>, ItemErrorV1> {
    let worlds = item_node_world_matrices_v1(model)?;
    let mut triangles = Vec::new();
    for (segment_index, segment) in model.segments.iter().enumerate() {
        let world = worlds
            .get(&segment.parent_node_id)
            .copied()
            .ok_or_else(|| {
                error(
                    "ITEM-SEAM-HIERARCHY-INVALID",
                    format!("model.segments[{segment_index}].parentNodeId"),
                    "seam segment parent node is absent from the resolved hierarchy",
                )
            })?;
        for (triangle_index, indices) in segment.indices.chunks_exact(3).enumerate() {
            let mut vertices = [[0.0_f64; 3]; 3];
            for corner in 0..3 {
                let position = segment
                    .positions
                    .get(indices[corner] as usize)
                    .copied()
                    .ok_or_else(|| {
                        error(
                            "ITEM-SEAM-GEOMETRY-INVALID",
                            format!(
                                "model.segments[{segment_index}].triangles[{triangle_index}].indices[{corner}]"
                            ),
                            "seam triangle index exceeds the segment position array",
                        )
                    })?;
                let transformed = transform_point_v1(world, position);
                vertices[corner] = [
                    f64::from(transformed[0]),
                    f64::from(transformed[1]),
                    f64::from(transformed[2]),
                ];
            }
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            let mut centroid = [0.0_f64; 3];
            for vertex in vertices {
                for axis in 0..3 {
                    min[axis] = min[axis].min(vertex[axis]);
                    max[axis] = max[axis].max(vertex[axis]);
                    centroid[axis] += vertex[axis] / 3.0;
                }
            }
            triangles.push(ItemSeamTriangleV1 {
                vertices,
                bounds: ItemSeamBoundsV1 { min, max },
                centroid,
            });
        }
    }
    if triangles.is_empty() {
        return Err(error(
            "ITEM-SEAM-GEOMETRY-EMPTY",
            "model.segments",
            "authoritative seam measurement requires at least one triangle per part",
        ));
    }
    Ok(triangles)
}

fn item_seam_transform_sha256_v1(options: &ItemPartBuildOptionsV2) -> Result<String, ItemErrorV1> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Binding<'a> {
        transform: ItemPartTransformV1,
        source_node: &'a Option<String>,
        #[serde(skip_serializing_if = "item_is_unit_scale_xyz_v1")]
        target_space_scale_xyz: [f32; 3],
    }
    let bytes = serde_json::to_vec(&Binding {
        transform: options.transform,
        source_node: &options.source_node,
        target_space_scale_xyz: options.target_space_scale_xyz,
    })
    .map_err(|_| {
        error(
            "ITEM-SEAM-BINDING-SERIALIZE-FAILED",
            "options",
            "seam source-node and transform binding could not be serialized",
        )
    })?;
    Ok(item_payload_sha256_v1(&bytes))
}

fn item_fit_bounds_v2(
    source: ItemFitSourceV1<'_>,
    transform: ItemPartTransformV1,
    target_space_scale_xyz: [f32; 3],
) -> Result<(ItemSeamBoundsV1, usize), ItemErrorV1> {
    let options = ItemPartBuildInternalOptionsV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        transform,
        source_node: source.source_node.map(str::to_owned),
        texture_encoding: ItemPartTextureEncodingV1::DirectColor,
        icon_size: None,
        icon_projection_bounds: None,
        weapon_color: None,
        target_space_scale_xyz,
        geometry_derived_icon: true,
        aurora_composer_v2: true,
    };
    let prepared = prepare_meshy_item_part_v1(source.source_glb, source.model_resref, &options)?;
    let triangles = item_seam_model_triangles_v1(&prepared.model)?;
    let bounds = triangles
        .iter()
        .map(|triangle| triangle.bounds)
        .reduce(item_seam_bounds_union_v1)
        .ok_or_else(|| {
            error(
                "ITEM-FIT-GEOMETRY-EMPTY",
                source.field,
                "Item auto-fit requires at least one triangle per source",
            )
        })?;
    Ok((bounds, prepared.triangle_count))
}

fn item_fit_bounds_v1(
    source: ItemFitSourceV1<'_>,
    transform: ItemPartTransformV1,
) -> Result<(ItemSeamBoundsV1, usize), ItemErrorV1> {
    item_fit_bounds_v2(source, transform, item_unit_scale_xyz_v1())
}

fn item_fit_long_axis_v1(bounds: ItemSeamBoundsV1) -> usize {
    (0..3)
        .max_by(|&first, &second| {
            let first_extent = bounds.max[first] - bounds.min[first];
            let second_extent = bounds.max[second] - bounds.min[second];
            first_extent
                .total_cmp(&second_extent)
                .then_with(|| second.cmp(&first))
        })
        .unwrap_or(0)
}

fn item_fit_axis_rotation_v1(axis: usize) -> [f32; 4] {
    match axis {
        0 => [
            0.0,
            -std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ],
        1 => [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ],
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}

fn item_fit_axis_rotation_to_aurora_y_v2(axis: usize) -> [f32; 4] {
    match axis {
        0 => [
            0.0,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        ],
        1 => [0.0, 0.0, 0.0, 1.0],
        _ => [
            -std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ],
    }
}

fn item_fit_options_v1(
    source_node: Option<&str>,
    transform: ItemPartTransformV1,
) -> ItemPartBuildOptionsV2 {
    item_fit_options_v2(source_node, transform, item_unit_scale_xyz_v1())
}

fn item_fit_options_v2(
    source_node: Option<&str>,
    transform: ItemPartTransformV1,
    target_space_scale_xyz: [f32; 3],
) -> ItemPartBuildOptionsV2 {
    ItemPartBuildOptionsV2 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        transform,
        source_node: source_node.map(str::to_owned),
        texture_encoding: ItemPartTextureEncodingV1::DirectColor,
        icon_size: None,
        icon_projection_bounds: None,
        weapon_color: None,
        target_space_scale_xyz,
    }
}

fn item_reference_controller_value_v1<const N: usize>(
    node: &NodeReport,
    name: &str,
    fallback: [f32; N],
) -> Result<[f32; N], ItemErrorV1> {
    let Some(controller) = node.controllers.iter().find(|controller| {
        controller
            .controller_name
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(name))
    }) else {
        return Ok(fallback);
    };
    let values = controller.values.first().ok_or_else(|| {
        error(
            "ITEM-REFERENCE-CONTROLLER-EMPTY",
            format!("nodes.{}.controllers.{name}", node.name),
            "reference controller has no decoded value row",
        )
    })?;
    if values.len() < N || values[..N].iter().any(|value| !value.is_finite()) {
        return Err(error(
            "ITEM-REFERENCE-CONTROLLER-INVALID",
            format!("nodes.{}.controllers.{name}", node.name),
            "reference controller does not contain the required finite values",
        ));
    }
    Ok(std::array::from_fn(|index| values[index]))
}

#[derive(Clone, Debug)]
struct ItemReferenceMdlGeometryV1 {
    controller_node_name: String,
    controller_translation: [f32; 3],
    controller_rotation_xyzw: [f32; 4],
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
}

fn inspect_item_reference_mdl_geometry_v1(
    inspection: &InspectionReport,
) -> Result<ItemReferenceMdlGeometryV1, ItemErrorV1> {
    fn walk(
        node: &NodeReport,
        parent_matrix: [f32; 16],
        bounds_min: &mut [f32; 3],
        bounds_max: &mut [f32; 3],
        evidence: &mut Option<(String, [f32; 3], [f32; 4])>,
        vertex_count: &mut usize,
    ) -> Result<(), ItemErrorV1> {
        let translation = item_reference_controller_value_v1(node, "position", [0.0; 3])?;
        let rotation =
            item_reference_controller_value_v1(node, "orientation", [0.0, 0.0, 0.0, 1.0])?;
        let local_matrix = item_part_rigid_matrix_v1(ItemPartTransformV1 {
            translation,
            rotation_xyzw: rotation,
            uniform_scale: 1.0,
            pivot: [0.0; 3],
        })?;
        let world_matrix = multiply_matrix_v1(parent_matrix, local_matrix);
        if let Some(mesh) = &node.mesh {
            if evidence.is_none() {
                *evidence = Some((node.name.clone(), translation, rotation));
            }
            for vertex in &mesh.vertices {
                let point = transform_point_v1(world_matrix, [vertex.x, vertex.y, vertex.z]);
                if point.iter().any(|value| !value.is_finite()) {
                    return Err(error(
                        "ITEM-REFERENCE-MDL-BOUNDS-INVALID",
                        format!("nodes.{}.mesh.vertices", node.name),
                        "reference model contains a non-finite transformed vertex",
                    ));
                }
                for axis in 0..3 {
                    bounds_min[axis] = bounds_min[axis].min(point[axis]);
                    bounds_max[axis] = bounds_max[axis].max(point[axis]);
                }
                *vertex_count += 1;
            }
        }
        for child in &node.children {
            walk(
                child,
                world_matrix,
                bounds_min,
                bounds_max,
                evidence,
                vertex_count,
            )?;
        }
        Ok(())
    }

    let identity = [
        1.0, 0.0, 0.0, 0.0, // X column
        0.0, 1.0, 0.0, 0.0, // Y column
        0.0, 0.0, 1.0, 0.0, // Z column
        0.0, 0.0, 0.0, 1.0, // translation/homogeneous column
    ];
    let mut bounds_min = [f32::INFINITY; 3];
    let mut bounds_max = [f32::NEG_INFINITY; 3];
    let mut evidence = None;
    let mut vertex_count = 0_usize;
    for root in &inspection.node_tree.roots {
        walk(
            root,
            identity,
            &mut bounds_min,
            &mut bounds_max,
            &mut evidence,
            &mut vertex_count,
        )?;
    }
    let (controller_node_name, controller_translation, controller_rotation_xyzw) = evidence
        .ok_or_else(|| {
            error(
                "ITEM-REFERENCE-MDL-GEOMETRY-EMPTY",
                "mdl.nodeTree",
                "reference Item model has no mesh node",
            )
        })?;
    if vertex_count == 0
        || (0..3).any(|axis| {
            !bounds_min[axis].is_finite()
                || !bounds_max[axis].is_finite()
                || bounds_max[axis] <= bounds_min[axis]
        })
    {
        return Err(error(
            "ITEM-REFERENCE-MDL-BOUNDS-INVALID",
            "mdl.nodeTree",
            "reference Item model must have finite, positive transformed mesh bounds",
        ));
    }
    Ok(ItemReferenceMdlGeometryV1 {
        controller_node_name,
        controller_translation,
        controller_rotation_xyzw,
        bounds_min,
        bounds_max,
    })
}

fn item_attachment_route_v1(base_item: &ItemBaseItemV1) -> ItemAttachmentRouteV1 {
    if base_item.model_type == 2 && base_item.equipable_slots & (0x10 | 0x20) != 0 {
        ItemAttachmentRouteV1::Hand
    } else if base_item.model_type == 3 {
        ItemAttachmentRouteV1::Capart
    } else if base_item.base_item == ITEM_BASEITEM_CLOAK_V1
        || base_item.capability.composition_profile == ItemCompositionProfileV1::CloakModel
    {
        ItemAttachmentRouteV1::Cloak
    } else {
        ItemAttachmentRouteV1::None
    }
}

pub fn item_attachment_profile_sha256_v1(
    profile: &ItemAttachmentProfileV1,
) -> Result<String, ItemErrorV1> {
    let mut identity = profile.clone();
    identity.profile_sha256.clear();
    let bytes = serde_json::to_vec(&identity).map_err(|_| {
        error(
            "ITEM-REFERENCE-PROFILE-SERIALIZE-FAILED",
            "profile",
            "Item attachment profile could not be serialized",
        )
    })?;
    Ok(item_payload_sha256_v1(&bytes))
}

/// Finalizes an author-supplied attachment frame without consulting a retail
/// model family. The source hashes, slot bounds and HAND origin remain part of
/// the immutable profile hash. Concept images are never runtime inputs.
pub fn finalize_item_authored_attachment_profile_v1(
    mut profile: ItemAttachmentProfileV1,
) -> Result<ItemAttachmentProfileV1, ItemErrorV1> {
    if !profile.profile_sha256.is_empty() {
        return Err(error(
            "ITEM-AUTHORED-PROFILE-PREHASHED",
            "profile.profileSha256",
            "authored attachment profile must be submitted with an empty profileSha256",
        ));
    }
    if profile.identity.reference_kind != "AUTHOR_DIRECTED_SOURCE_FRAME"
        || profile.attachment_evidence != "AUTHOR_MANUAL_ALIGNMENT_V1"
    {
        return Err(error(
            "ITEM-AUTHORED-PROFILE-EVIDENCE-INVALID",
            "profile.identity.referenceKind",
            "authored attachment profile must use the exact source-frame and manual-alignment evidence contract",
        ));
    }
    profile.profile_sha256 = item_attachment_profile_sha256_v1(&profile)?;
    validate_item_attachment_profile_v1(&profile)?;
    Ok(profile)
}

fn validate_item_attachment_profile_v1(
    profile: &ItemAttachmentProfileV1,
) -> Result<(), ItemErrorV1> {
    let mut axes = [profile.axial_axis, profile.width_axis, profile.depth_axis];
    axes.sort_unstable();
    if profile.schema_version != ITEM_SCHEMA_VERSION_V1
        || profile.algorithm != "AURORA_ITEM_REFERENCE_PROFILE_V1"
        || profile.status != "PASSED"
        || axes != [0, 1, 2]
        || !valid_lowercase_sha256_v1(&profile.identity.resource_context_sha256)
        || !valid_lowercase_sha256_v1(&profile.identity.baseitems_sha256)
        || !valid_lowercase_sha256_v1(&profile.profile_sha256)
        || profile.identity.schema_version != ITEM_SCHEMA_VERSION_V1
        || profile.identity.item_class.trim().is_empty()
        || profile.identity.reference_kind.trim().is_empty()
        || profile.identity.reference_id.trim().is_empty()
        || profile.slots.is_empty()
    {
        return Err(error(
            "ITEM-REFERENCE-PROFILE-INVALID",
            "profile",
            "Item attachment profile identity, axes, slots or hashes are invalid",
        ));
    }
    for (index, slot) in profile.slots.iter().enumerate() {
        if slot.field.trim().is_empty()
            || slot.label.trim().is_empty()
            || slot.token.trim().is_empty()
            || slot.controller_node_name.trim().is_empty()
            || !valid_lowercase_sha256_v1(&slot.model_sha256)
            || (0..3).any(|axis| {
                !slot.bounds_min[axis].is_finite()
                    || !slot.bounds_max[axis].is_finite()
                    || slot.bounds_max[axis] <= slot.bounds_min[axis]
            })
        {
            return Err(error(
                "ITEM-REFERENCE-PROFILE-SLOT-INVALID",
                format!("profile.slots[{index}]"),
                "reference slot binding or transformed bounds are invalid",
            ));
        }
        validate_resref(
            &slot.model_resref,
            &format!("profile.slots[{index}].modelResref"),
        )?;
    }
    if item_attachment_profile_sha256_v1(profile)? != profile.profile_sha256 {
        return Err(error(
            "ITEM-REFERENCE-PROFILE-HASH-MISMATCH",
            "profile.profileSha256",
            "Item attachment profile hash does not match its semantic payload",
        ));
    }
    Ok(())
}

/// Extracts the real model-space slot frames from exact reference MDLs. The
/// resulting profile binds BaseItem semantics, resource-context identity and
/// byte-level model provenance into one immutable fitting contract.
pub fn build_item_attachment_profile_v1(
    base_item: &ItemBaseItemV1,
    resource_context_sha256: &str,
    baseitems_sha256: &str,
    reference_kind: &str,
    reference_id: &str,
    inputs: &[ItemReferenceMdlInputV1<'_>],
) -> Result<ItemAttachmentProfileV1, ItemErrorV1> {
    if !valid_lowercase_sha256_v1(resource_context_sha256)
        || !valid_lowercase_sha256_v1(baseitems_sha256)
        || reference_kind.trim().is_empty()
        || reference_id.trim().is_empty()
        || inputs.len() != base_item.part_slots.len()
        || inputs.is_empty()
    {
        return Err(error(
            "ITEM-REFERENCE-PROFILE-INPUT-INVALID",
            "inputs",
            "profile construction requires exact context/baseitems hashes, reference identity and one MDL per BaseItem slot",
        ));
    }
    let mut slots = Vec::with_capacity(inputs.len());
    for (index, (input, expected)) in inputs.iter().zip(&base_item.part_slots).enumerate() {
        if !input.field.eq_ignore_ascii_case(&expected.field) {
            return Err(error(
                "ITEM-REFERENCE-PROFILE-SLOT-ORDER",
                format!("inputs[{index}].field"),
                format!("expected ordered BaseItem field {}", expected.field),
            ));
        }
        validate_resref(input.model_resref, &format!("inputs[{index}].modelResref"))?;
        let inspection = inspect_binary_mdl(input.mdl_payload).map_err(|source| {
            error(
                "ITEM-REFERENCE-MDL-READBACK",
                format!("inputs[{index}].mdlPayload"),
                format!("{} at byte {}", source.code, source.offset),
            )
        })?;
        if !inspection
            .model
            .name
            .eq_ignore_ascii_case(input.model_resref)
            || !inspection.diagnostics.is_empty()
            || !inspection.unsupported.is_empty()
        {
            return Err(error(
                "ITEM-REFERENCE-MDL-IDENTITY",
                format!("inputs[{index}]"),
                "reference MDL name or semantic readback does not match its declared resref",
            ));
        }
        let geometry = inspect_item_reference_mdl_geometry_v1(&inspection)?;
        slots.push(ItemReferenceSlotFrameV1 {
            field: expected.field.clone(),
            label: expected.label.clone(),
            token: expected.token.clone().ok_or_else(|| {
                error(
                    "ITEM-REFERENCE-PROFILE-TOKEN-MISSING",
                    format!("baseItem.partSlots[{index}].token"),
                    "reference-based model slots require a filename token",
                )
            })?,
            model_resref: input.model_resref.to_ascii_lowercase(),
            model_sha256: item_payload_sha256_v1(input.mdl_payload),
            controller_node_name: geometry.controller_node_name,
            controller_translation: geometry.controller_translation,
            controller_rotation_xyzw: geometry.controller_rotation_xyzw,
            bounds_min: geometry.bounds_min,
            bounds_max: geometry.bounds_max,
            allow_axial_extension_at_min: false,
            allow_axial_extension_at_max: base_item.model_type == 2 && index + 1 == inputs.len(),
        });
    }
    let common_origin = [0.0_f32; 3];
    let origin_slots = slots
        .iter()
        .filter(|slot| {
            (0..3).all(|axis| {
                slot.bounds_min[axis] <= common_origin[axis] + 1.0e-5
                    && slot.bounds_max[axis] >= common_origin[axis] - 1.0e-5
            })
        })
        .collect::<Vec<_>>();
    if origin_slots.is_empty() {
        return Err(error(
            "ITEM-REFERENCE-ORIGIN-OUTSIDE-PARTS",
            "profile.commonOrigin",
            "no exact reference slot contains the Aurora model origin",
        ));
    }
    let attachment_zone_min = std::array::from_fn(|axis| {
        origin_slots
            .iter()
            .map(|slot| slot.bounds_min[axis])
            .fold(f32::INFINITY, f32::min)
    });
    let attachment_zone_max = std::array::from_fn(|axis| {
        origin_slots
            .iter()
            .map(|slot| slot.bounds_max[axis])
            .fold(f32::NEG_INFINITY, f32::max)
    });
    let mut profile = ItemAttachmentProfileV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        algorithm: "AURORA_ITEM_REFERENCE_PROFILE_V1".to_owned(),
        status: "PASSED".to_owned(),
        identity: ItemReferenceProfileIdentityV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            resource_context_sha256: resource_context_sha256.to_owned(),
            baseitems_sha256: baseitems_sha256.to_owned(),
            base_item: base_item.base_item,
            item_class: base_item.item_class.clone(),
            model_type: base_item.model_type,
            reference_kind: reference_kind.trim().to_owned(),
            reference_id: reference_id.trim().to_owned(),
        },
        attachment_route: item_attachment_route_v1(base_item),
        equipable_slots: base_item.equipable_slots,
        common_origin,
        axial_axis: 1,
        width_axis: 2,
        depth_axis: 0,
        attachment_zone_min,
        attachment_zone_max,
        attachment_evidence: "ORIGIN_CONTAINING_REFERENCE_PARTS_V1".to_owned(),
        slots,
        profile_sha256: String::new(),
    };
    profile.profile_sha256 = item_attachment_profile_sha256_v1(&profile)?;
    validate_item_attachment_profile_v1(&profile)?;
    Ok(profile)
}

/// Proposes one deterministic rigid transform per ordered Meshy Item slot.
/// The three-part envelope is derived from the Aurora Bottom/Middle/Top
/// composer itself, never from a Weapon/Shield usage category. The proposal is
/// accepted only when the existing triangle-surface seam measurement reports
/// every adjacent pair as TOUCHING and no non-adjacent pair overlaps.
pub fn fit_meshy_item_parts_v1(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
) -> Result<ItemFitReportV1, ItemErrorV1> {
    let target_lengths = if sources.len() == 1 {
        vec![1.0]
    } else {
        vec![0.30, 0.10, 0.60]
    };
    fit_meshy_item_parts_with_target_lengths_v2(sources, tolerance, &target_lengths)
}

/// Fits the ordered Aurora slots against an explicit axial-length contract.
/// Values are absolute composer-space lengths, so the caller controls both
/// the total item length and the Bottom/Middle/Top proportions. This legacy
/// +Z route is retained only so the immutable V3 lineage remains reproducible.
pub fn fit_meshy_item_parts_with_target_lengths_v2(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    target_lengths: &[f32],
) -> Result<ItemFitReportV1, ItemErrorV1> {
    fit_meshy_item_parts_on_target_axis_v3(
        sources,
        tolerance,
        target_lengths,
        2,
        "ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V1",
        false,
        None,
    )
}

/// Proposes deterministic Bottom/Middle/Top transforms in the +Y axial
/// convention used by Aurora's ModelType 2 weapon-part composer.
pub fn fit_meshy_item_parts_aurora_v3(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
) -> Result<ItemFitReportV1, ItemErrorV1> {
    let target_lengths = if sources.len() == 1 {
        vec![1.0]
    } else {
        vec![0.30, 0.10, 0.60]
    };
    fit_meshy_item_parts_with_target_lengths_aurora_v3(sources, tolerance, &target_lengths)
}

/// Fits ordered Aurora Item slots against explicit composer-space lengths and
/// binds the resulting report to Aurora's +Y axial convention.
pub fn fit_meshy_item_parts_with_target_lengths_aurora_v3(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    target_lengths: &[f32],
) -> Result<ItemFitReportV1, ItemErrorV1> {
    fit_meshy_item_parts_on_target_axis_v3(
        sources,
        tolerance,
        target_lengths,
        1,
        "ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V2_AURORA_Y",
        true,
        None,
    )
}

fn item_fit_matrix_determinant_v3(columns: [[f32; 3]; 3]) -> f32 {
    columns[0][0] * (columns[1][1] * columns[2][2] - columns[2][1] * columns[1][2])
        - columns[1][0] * (columns[0][1] * columns[2][2] - columns[2][1] * columns[0][2])
        + columns[2][0] * (columns[0][1] * columns[1][2] - columns[1][1] * columns[0][2])
}

fn item_fit_quaternion_from_columns_v3(columns: [[f32; 3]; 3]) -> [f32; 4] {
    let r00 = columns[0][0];
    let r01 = columns[1][0];
    let r02 = columns[2][0];
    let r10 = columns[0][1];
    let r11 = columns[1][1];
    let r12 = columns[2][1];
    let r20 = columns[0][2];
    let r21 = columns[1][2];
    let r22 = columns[2][2];
    let trace = r00 + r11 + r22;
    let [x, y, z, w] = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(r21 - r12) / s, (r02 - r20) / s, (r10 - r01) / s, s * 0.25]
    } else if r00 > r11 && r00 > r22 {
        let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0;
        [s * 0.25, (r01 + r10) / s, (r02 + r20) / s, (r21 - r12) / s]
    } else if r11 > r22 {
        let s = (1.0 + r11 - r00 - r22).sqrt() * 2.0;
        [(r01 + r10) / s, s * 0.25, (r12 + r21) / s, (r02 - r20) / s]
    } else {
        let s = (1.0 + r22 - r00 - r11).sqrt() * 2.0;
        [(r02 + r20) / s, (r12 + r21) / s, s * 0.25, (r10 - r01) / s]
    };
    let norm = (x * x + y * y + z * z + w * w).sqrt();
    [x / norm, y / norm, z / norm, w / norm]
}

fn resolve_item_orientation_frame_v3(
    sources: &[ItemFitSourceV1<'_>],
) -> Result<(ItemOrientationFrameV3, [f32; 4]), ItemErrorV1> {
    let identities = sources
        .iter()
        .copied()
        .map(|source| item_fit_bounds_v1(source, ItemPartTransformV1::default()))
        .collect::<Result<Vec<_>, _>>()?;
    let long_axes = identities
        .iter()
        .map(|(bounds, _)| item_fit_long_axis_v1(*bounds))
        .collect::<Vec<_>>();
    let source_axial_axis = if sources.len() == 1 || long_axes[0] == long_axes[sources.len() - 1] {
        long_axes[0]
    } else {
        let mut counts = [0_u8; 3];
        for axis in &long_axes {
            counts[*axis] += 1;
        }
        (0..3)
            .max_by_key(|&axis| (counts[axis], std::cmp::Reverse(axis)))
            .unwrap_or(0)
    };
    let transverse_axes = (0..3)
        .filter(|axis| *axis != source_axial_axis)
        .collect::<Vec<_>>();
    let mut scores = [0.0_f64; 3];
    for (bounds, _) in &identities {
        let axial_extent = bounds.max[source_axial_axis] - bounds.min[source_axial_axis];
        if !axial_extent.is_finite() || axial_extent <= 1.0e-9 {
            return Err(error(
                "ITEM-FRAME-AXIS-DEGENERATE",
                "sources",
                "full Item orientation frame requires a positive shared axial extent",
            ));
        }
        for axis in &transverse_axes {
            scores[*axis] += (bounds.max[*axis] - bounds.min[*axis]) / axial_extent;
        }
    }
    let source_width_axis = transverse_axes
        .iter()
        .copied()
        .max_by(|first, second| scores[*first].total_cmp(&scores[*second]))
        .unwrap_or(0);
    let source_depth_axis = transverse_axes
        .iter()
        .copied()
        .find(|axis| *axis != source_width_axis)
        .unwrap_or(0);
    let width_to_depth_ratio = (scores[source_width_axis] / scores[source_depth_axis]) as f32;
    if !width_to_depth_ratio.is_finite() || width_to_depth_ratio <= 0.0 {
        return Err(error(
            "ITEM-FRAME-TRANSVERSE-DEGENERATE",
            "sources",
            "full Item orientation frame requires two finite transverse extents",
        ));
    }

    let mut columns = [[0.0_f32; 3]; 3];
    columns[source_axial_axis] = [0.0, 1.0, 0.0];
    columns[source_width_axis] = [0.0, 0.0, 1.0];
    columns[source_depth_axis] = [1.0, 0.0, 0.0];
    if item_fit_matrix_determinant_v3(columns) < 0.0 {
        columns[source_depth_axis] = [-1.0, 0.0, 0.0];
    }
    let determinant = item_fit_matrix_determinant_v3(columns);
    let status = if width_to_depth_ratio >= 1.10 {
        "PASSED"
    } else {
        "MANUAL_REQUIRED"
    };
    Ok((
        ItemOrientationFrameV3 {
            source_axial_axis: source_axial_axis as u8,
            source_width_axis: source_width_axis as u8,
            source_depth_axis: source_depth_axis as u8,
            target_axial_axis: 1,
            target_width_axis: 2,
            target_depth_axis: 0,
            width_to_depth_ratio,
            handedness_determinant: determinant,
            evidence: "GROUP_NORMALIZED_TRANSVERSE_EXTENTS_WITH_PROPER_HANDEDNESS_V1".to_owned(),
            status: status.to_owned(),
        },
        item_fit_quaternion_from_columns_v3(columns),
    ))
}

/// Fits ModelType 2 parts in Aurora +Y space and gives every adjacent pair a
/// small, explicit axial connector overlap. Retail weapon families use such
/// overlap to hide seams; a tolerance-only positive gap is not accepted.
pub fn fit_meshy_item_parts_with_target_lengths_aurora_v4(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    target_lengths: &[f32],
) -> Result<ItemFitReportV2, ItemErrorV1> {
    let legacy =
        fit_meshy_item_parts_with_target_lengths_aurora_v3(sources, tolerance, target_lengths)?;
    finish_item_fit_with_connector_overlap_v2(
        sources,
        tolerance,
        target_lengths,
        legacy,
        "ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y",
    )
}

fn finish_item_fit_with_connector_overlap_v2(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    target_lengths: &[f32],
    legacy: ItemFitReportV1,
    algorithm: &str,
) -> Result<ItemFitReportV2, ItemErrorV1> {
    let mut parts = legacy
        .parts
        .into_iter()
        .map(|part| ItemFitPartV2 {
            field: part.field,
            source_sha256: part.source_sha256,
            source_node: part.source_node,
            triangle_count: part.triangle_count,
            input_bounds_min: part.input_bounds_min,
            input_bounds_max: part.input_bounds_max,
            axial_source_axis: part.axial_source_axis,
            axial_target_axis: 1,
            target_axial_length: part.target_axial_length,
            transform: part.transform,
            target_space_scale_xyz: item_unit_scale_xyz_v1(),
            transform_sha256: part.transform_sha256,
            output_bounds_min: part.output_bounds_min,
            output_bounds_max: part.output_bounds_max,
            bottom_connector: None,
            top_connector: None,
        })
        .collect::<Vec<_>>();
    let mut connector_overlaps = Vec::with_capacity(parts.len().saturating_sub(1));
    let mut cursor = parts
        .first()
        .map(|part| part.output_bounds_min[1])
        .unwrap_or(0.0);
    for index in 0..parts.len() {
        let overlap = if index == 0 {
            0.0
        } else {
            (target_lengths[index - 1].min(target_lengths[index]) * 0.10).min(0.01)
        };
        let desired_min = if index == 0 { cursor } else { cursor - overlap };
        let shift = desired_min - parts[index].output_bounds_min[1];
        parts[index].transform.translation[1] += shift;
        parts[index].output_bounds_min[1] += shift;
        parts[index].output_bounds_max[1] += shift;
        let options = item_fit_options_v1(sources[index].source_node, parts[index].transform);
        parts[index].transform_sha256 = item_seam_transform_sha256_v1(&options)?;
        cursor = parts[index].output_bounds_max[1];
        if index > 0 {
            connector_overlaps.push(overlap);
        }
    }
    for index in 0..parts.len() {
        let transverse_center = [
            (parts[index].output_bounds_min[0] + parts[index].output_bounds_max[0]) * 0.5,
            0.0,
            (parts[index].output_bounds_min[2] + parts[index].output_bounds_max[2]) * 0.5,
        ];
        if index > 0 {
            parts[index].bottom_connector = Some(ItemConnectorAnchorV2 {
                kind: "BOTTOM".to_owned(),
                axial_axis: 1,
                position: [
                    transverse_center[0],
                    parts[index].output_bounds_min[1],
                    transverse_center[2],
                ],
            });
        }
        if index + 1 < parts.len() {
            parts[index].top_connector = Some(ItemConnectorAnchorV2 {
                kind: "TOP".to_owned(),
                axial_axis: 1,
                position: [
                    transverse_center[0],
                    parts[index].output_bounds_max[1],
                    transverse_center[2],
                ],
            });
        }
    }
    let options = parts
        .iter()
        .zip(sources)
        .map(|(part, source)| item_fit_options_v1(source.source_node, part.transform))
        .collect::<Vec<_>>();
    let mut adjacent_seams = Vec::with_capacity(parts.len().saturating_sub(1));
    let mut adjacent_connectors = Vec::with_capacity(parts.len().saturating_sub(1));
    for index in 0..parts.len().saturating_sub(1) {
        let seam = measure_meshy_item_seam_v1(
            sources[index].field,
            sources[index].source_glb,
            sources[index].model_resref,
            &options[index],
            sources[index + 1].field,
            sources[index + 1].source_glb,
            sources[index + 1].model_resref,
            &options[index + 1],
            tolerance,
        )?;
        let axial_overlap =
            parts[index].output_bounds_max[1] - parts[index + 1].output_bounds_min[1];
        let designed = connector_overlaps[index];
        let required_min_overlap = designed * 0.75;
        let required_max_overlap = designed * 1.25;
        let overlapping = axial_overlap >= required_min_overlap
            && axial_overlap <= required_max_overlap
            && seam.status != "GAP";
        adjacent_connectors.push(ItemAdjacentConnectorV2 {
            first_field: parts[index].field.clone(),
            first_connector: "TOP".to_owned(),
            second_field: parts[index + 1].field.clone(),
            second_connector: "BOTTOM".to_owned(),
            axial_axis: 1,
            axial_overlap,
            required_min_overlap,
            required_max_overlap,
            surface_status: seam.status.clone(),
            status: if overlapping { "OVERLAPPING" } else { "FAILED" }.to_owned(),
        });
        adjacent_seams.push(seam);
    }
    let mut non_adjacent_measurements = Vec::new();
    for first in 0..sources.len() {
        for second in first + 2..sources.len() {
            non_adjacent_measurements.push(measure_meshy_item_seam_v1(
                sources[first].field,
                sources[first].source_glb,
                sources[first].model_resref,
                &options[first],
                sources[second].field,
                sources[second].source_glb,
                sources[second].model_resref,
                &options[second],
                tolerance,
            )?);
        }
    }
    let passed = adjacent_connectors
        .iter()
        .all(|connector| connector.status == "OVERLAPPING")
        && non_adjacent_measurements
            .iter()
            .all(|measurement| !measurement.overlap);
    let mut report = ItemFitReportV2 {
        schema_version: 2,
        algorithm: algorithm.to_owned(),
        status: if passed { "PASSED" } else { "MANUAL_REQUIRED" }.to_owned(),
        tolerance,
        iterations: 1,
        parts,
        adjacent_seams,
        adjacent_connectors,
        non_adjacent_measurements,
        solution_sha256: String::new(),
    };
    let bytes = item_fit_report_v2_legacy_hash_bytes_v1(&report)?;
    report.solution_sha256 = item_payload_sha256_v1(&bytes);
    Ok(report)
}

/// Resolves a complete proper Item basis. Aurora's modular Item convention is
/// axial +Y, broad width Z and front/depth X. Unlike V4 this function does not
/// leave the transverse roll unconstrained after mapping the longest axis.
pub fn fit_meshy_item_parts_with_target_lengths_aurora_v5(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    target_lengths: &[f32],
) -> Result<ItemFitReportV3, ItemErrorV1> {
    let (orientation_frame, rotation) = resolve_item_orientation_frame_v3(sources)?;
    let legacy = fit_meshy_item_parts_on_target_axis_v3(
        sources,
        tolerance,
        target_lengths,
        1,
        "ORDERED_SLOT_FULL_FRAME_SEAM_GATE_V3_AURORA_YZX",
        true,
        Some(rotation),
    )?;
    let fitted = finish_item_fit_with_connector_overlap_v2(
        sources,
        tolerance,
        target_lengths,
        legacy,
        "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX",
    )?;
    let passed = fitted.status == "PASSED" && orientation_frame.status == "PASSED";
    let mut report = ItemFitReportV3 {
        schema_version: 3,
        algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX".to_owned(),
        status: if passed { "PASSED" } else { "MANUAL_REQUIRED" }.to_owned(),
        tolerance: fitted.tolerance,
        iterations: fitted.iterations,
        orientation_frame,
        parts: fitted.parts,
        adjacent_seams: fitted.adjacent_seams,
        adjacent_connectors: fitted.adjacent_connectors,
        non_adjacent_measurements: fitted.non_adjacent_measurements,
        solution_sha256: String::new(),
    };
    let bytes = item_fit_report_v3_legacy_hash_bytes_v1(&report)?;
    report.solution_sha256 = item_payload_sha256_v1(&bytes);
    validate_item_fit_report_v3(&report)?;
    Ok(report)
}

/// Fits generated parts into exact model-space frames extracted from a
/// selected Aurora reference Item. This replaces the origin-zero cursor used
/// by V1-V3 and therefore preserves the reference grip/attachment zone.
pub fn fit_meshy_item_parts_to_attachment_profile_v1(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    profile: &ItemAttachmentProfileV1,
) -> Result<ItemFitReportV4, ItemErrorV1> {
    fit_meshy_item_parts_to_attachment_profile_with_axial_scales_v1(
        sources,
        tolerance,
        profile,
        &vec![1.0; sources.len()],
    )
}

/// Fits generated parts into an Aurora reference Item while allowing explicit,
/// reference-relative axial scaling only on slot ends that the extracted
/// profile marks as extensible. The non-extensible connector end remains
/// anchored, so a longer blade cannot move the grip or break the adjacent slot.
pub fn fit_meshy_item_parts_to_attachment_profile_with_axial_scales_v1(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    profile: &ItemAttachmentProfileV1,
    axial_scale_factors: &[f32],
) -> Result<ItemFitReportV4, ItemErrorV1> {
    validate_item_attachment_profile_v1(profile)?;
    if profile.axial_axis != 1 || profile.width_axis != 2 || profile.depth_axis != 0 {
        return Err(error(
            "ITEM-FIT-REFERENCE-FRAME-UNSUPPORTED",
            "profile",
            "the current modular Item fitter requires the Aurora Y axial, Z width, X depth frame",
        ));
    }
    if sources.len() != profile.slots.len() || !matches!(sources.len(), 1 | 3) {
        return Err(error(
            "ITEM-FIT-REFERENCE-SLOT-COUNT",
            "sources",
            "generated sources must match the complete ordered reference slot set",
        ));
    }
    if axial_scale_factors.len() != sources.len()
        || axial_scale_factors
            .iter()
            .any(|value| !value.is_finite() || !(0.5..=2.0).contains(value))
    {
        return Err(error(
            "ITEM-FIT-REFERENCE-SCALE-INVALID",
            "axialScaleFactors",
            "reference-relative axial scales must contain one finite value from 0.5 through 2.0 per ordered slot",
        ));
    }
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(error(
            "ITEM-FIT-TOLERANCE-INVALID",
            "tolerance",
            "Item fit tolerance must be finite and non-negative",
        ));
    }
    for (index, (source, slot)) in sources.iter().zip(&profile.slots).enumerate() {
        if !source.field.eq_ignore_ascii_case(&slot.field) {
            return Err(error(
                "ITEM-FIT-REFERENCE-SLOT-ORDER",
                format!("sources[{index}].field"),
                format!("expected reference field {}", slot.field),
            ));
        }
        validate_resref(
            source.model_resref,
            &format!("sources[{index}].modelResref"),
        )?;
        if (axial_scale_factors[index] - 1.0).abs() > 1.0e-6
            && !slot.allow_axial_extension_at_min
            && !slot.allow_axial_extension_at_max
        {
            return Err(error(
                "ITEM-FIT-REFERENCE-SCALE-NOT-ALLOWED",
                format!("axialScaleFactors[{index}]"),
                format!(
                    "{} is locked by the selected Aurora reference slot and cannot be scaled independently",
                    slot.label,
                ),
            ));
        }
    }

    let (orientation_frame, rotation) = resolve_item_orientation_frame_v3(sources)?;
    let identities = sources
        .iter()
        .copied()
        .map(|source| item_fit_bounds_v1(source, ItemPartTransformV1::default()))
        .collect::<Result<Vec<_>, _>>()?;
    let mut parts = Vec::with_capacity(sources.len());
    let mut options = Vec::with_capacity(sources.len());
    for (index, ((source, slot), (identity_bounds, triangle_count))) in sources
        .iter()
        .copied()
        .zip(&profile.slots)
        .zip(&identities)
        .enumerate()
    {
        let rotation_only = ItemPartTransformV1 {
            translation: [0.0; 3],
            rotation_xyzw: rotation,
            uniform_scale: 1.0,
            pivot: [0.0; 3],
        };
        let rotated_unscaled = item_fit_bounds_v1(source, rotation_only)?.0;
        let reference_axial_length = slot.bounds_max[1] - slot.bounds_min[1];
        let target_axial_length = reference_axial_length * axial_scale_factors[index];
        let source_axial_length = rotated_unscaled.max[1] - rotated_unscaled.min[1];
        if !source_axial_length.is_finite() || source_axial_length <= 1.0e-9 {
            return Err(error(
                "ITEM-FIT-REFERENCE-AXIS-DEGENERATE",
                format!("sources[{index}]"),
                "generated source has no positive extent on the resolved axial frame",
            ));
        }
        let base_transform = ItemPartTransformV1 {
            uniform_scale: (f64::from(target_axial_length) / source_axial_length) as f32,
            ..rotation_only
        };
        let axial_scaled_bounds = item_fit_bounds_v1(source, base_transform)?.0;
        let mut transverse_scale = 1.0_f64;
        for axis in [0_usize, 2_usize] {
            let source_span = axial_scaled_bounds.max[axis] - axial_scaled_bounds.min[axis];
            let reference_span = f64::from(slot.bounds_max[axis] - slot.bounds_min[axis]);
            if source_span.is_finite() && source_span > 1.0e-9 {
                transverse_scale = transverse_scale.min(reference_span / source_span);
            }
        }
        if !transverse_scale.is_finite() || transverse_scale <= 0.0 {
            return Err(error(
                "ITEM-FIT-REFERENCE-TRANSVERSE-DEGENERATE",
                format!("profile.slots[{index}]"),
                "reference slot does not provide a positive finite transverse envelope",
            ));
        }
        transverse_scale = transverse_scale.min(1.0);
        let target_space_scale_xyz = [transverse_scale as f32, 1.0, transverse_scale as f32];
        let scaled_bounds = item_fit_bounds_v2(source, base_transform, target_space_scale_xyz)?.0;
        let mut translation = [0.0_f32; 3];
        for axis in [0_usize, 2_usize] {
            let target_center = (slot.bounds_min[axis] + slot.bounds_max[axis]) * 0.5;
            let source_center = (scaled_bounds.min[axis] + scaled_bounds.max[axis]) * 0.5;
            translation[axis] = target_center - source_center as f32;
        }
        translation[1] = if slot.allow_axial_extension_at_min && !slot.allow_axial_extension_at_max
        {
            slot.bounds_max[1] - scaled_bounds.max[1] as f32
        } else if slot.allow_axial_extension_at_min && slot.allow_axial_extension_at_max {
            let target_center = (slot.bounds_min[1] + slot.bounds_max[1]) * 0.5;
            let source_center = (scaled_bounds.min[1] + scaled_bounds.max[1]) * 0.5;
            target_center - source_center as f32
        } else {
            slot.bounds_min[1] - scaled_bounds.min[1] as f32
        };
        let transform = ItemPartTransformV1 {
            translation,
            ..base_transform
        };
        let output_bounds = ItemSeamBoundsV1 {
            min: std::array::from_fn(|axis| scaled_bounds.min[axis] + f64::from(translation[axis])),
            max: std::array::from_fn(|axis| scaled_bounds.max[axis] + f64::from(translation[axis])),
        };
        let part_options =
            item_fit_options_v2(source.source_node, transform, target_space_scale_xyz);
        let transform_sha256 = item_seam_transform_sha256_v1(&part_options)?;
        let transverse_center = [
            (output_bounds.min[0] + output_bounds.max[0]) as f32 * 0.5,
            0.0,
            (output_bounds.min[2] + output_bounds.max[2]) as f32 * 0.5,
        ];
        let bottom_connector = (index > 0).then(|| ItemConnectorAnchorV2 {
            kind: "BOTTOM".to_owned(),
            axial_axis: 1,
            position: [
                transverse_center[0],
                output_bounds.min[1] as f32,
                transverse_center[2],
            ],
        });
        let top_connector = (index + 1 < sources.len()).then(|| ItemConnectorAnchorV2 {
            kind: "TOP".to_owned(),
            axial_axis: 1,
            position: [
                transverse_center[0],
                output_bounds.max[1] as f32,
                transverse_center[2],
            ],
        });
        options.push(part_options);
        parts.push(ItemFitPartV2 {
            field: slot.field.clone(),
            source_sha256: item_payload_sha256_v1(source.source_glb),
            source_node: source.source_node.map(str::to_owned),
            triangle_count: *triangle_count,
            input_bounds_min: identity_bounds.min.map(|value| value as f32),
            input_bounds_max: identity_bounds.max.map(|value| value as f32),
            axial_source_axis: orientation_frame.source_axial_axis,
            axial_target_axis: 1,
            target_axial_length,
            transform,
            target_space_scale_xyz,
            transform_sha256,
            output_bounds_min: output_bounds.min.map(|value| value as f32),
            output_bounds_max: output_bounds.max.map(|value| value as f32),
            bottom_connector,
            top_connector,
        });
    }

    let mut adjacent_seams = Vec::with_capacity(parts.len().saturating_sub(1));
    let mut adjacent_connectors = Vec::with_capacity(parts.len().saturating_sub(1));
    for index in 0..parts.len().saturating_sub(1) {
        let seam = measure_meshy_item_seam_v1(
            sources[index].field,
            sources[index].source_glb,
            sources[index].model_resref,
            &options[index],
            sources[index + 1].field,
            sources[index + 1].source_glb,
            sources[index + 1].model_resref,
            &options[index + 1],
            tolerance,
        )?;
        let expected_overlap =
            profile.slots[index].bounds_max[1] - profile.slots[index + 1].bounds_min[1];
        if !expected_overlap.is_finite() || expected_overlap <= 0.0 {
            return Err(error(
                "ITEM-FIT-REFERENCE-CONNECTOR-INVALID",
                format!("profile.slots[{index}]"),
                "adjacent reference slots must carry a positive axial overlap",
            ));
        }
        let actual_overlap =
            parts[index].output_bounds_max[1] - parts[index + 1].output_bounds_min[1];
        let margin = (tolerance * 0.25).max(1.0e-5);
        let required_min_overlap = (expected_overlap - margin).max(expected_overlap * 0.5);
        let required_max_overlap = expected_overlap + margin;
        let overlap_epsilon = 1.0e-6;
        let overlapping = actual_overlap + overlap_epsilon >= required_min_overlap
            && actual_overlap <= required_max_overlap + overlap_epsilon
            && seam.status != "GAP";
        adjacent_connectors.push(ItemAdjacentConnectorV2 {
            first_field: parts[index].field.clone(),
            first_connector: "TOP".to_owned(),
            second_field: parts[index + 1].field.clone(),
            second_connector: "BOTTOM".to_owned(),
            axial_axis: 1,
            axial_overlap: actual_overlap,
            required_min_overlap,
            required_max_overlap,
            surface_status: seam.status.clone(),
            status: if overlapping { "OVERLAPPING" } else { "FAILED" }.to_owned(),
        });
        adjacent_seams.push(seam);
    }
    let mut non_adjacent_measurements = Vec::new();
    for first in 0..sources.len() {
        for second in first + 2..sources.len() {
            non_adjacent_measurements.push(measure_meshy_item_seam_v1(
                sources[first].field,
                sources[first].source_glb,
                sources[first].model_resref,
                &options[first],
                sources[second].field,
                sources[second].source_glb,
                sources[second].model_resref,
                &options[second],
                tolerance,
            )?);
        }
    }
    let passed = orientation_frame.status == "PASSED"
        && adjacent_connectors
            .iter()
            .all(|connector| connector.status == "OVERLAPPING")
        && non_adjacent_measurements
            .iter()
            .all(|measurement| !measurement.overlap);
    let mut report = ItemFitReportV4 {
        schema_version: 4,
        algorithm: "ITEM_REFERENCE_SLOT_FRAME_FIT_V1".to_owned(),
        status: if passed { "PASSED" } else { "MANUAL_REQUIRED" }.to_owned(),
        tolerance,
        iterations: 1,
        reference_profile_sha256: profile.profile_sha256.clone(),
        common_origin: profile.common_origin,
        orientation_frame,
        parts,
        adjacent_seams,
        adjacent_connectors,
        non_adjacent_measurements,
        solution_sha256: String::new(),
    };
    let bytes = serde_json::to_vec(&report).map_err(|_| {
        error(
            "ITEM-FIT-V4-REPORT-SERIALIZE-FAILED",
            "report",
            "reference-frame Item fit report could not be serialized",
        )
    })?;
    report.solution_sha256 = item_payload_sha256_v1(&bytes);
    validate_item_fit_report_v4(&report)?;
    validate_item_fit_report_v4_against_profile_v1(&report, profile)?;
    Ok(report)
}

/// Validates the exact visible three-part editor transforms. It recomputes
/// bounds, surfaces, connector overlap and hashes from the submitted Q/T/S;
/// no automatic fit solution is generated or substituted.
pub fn validate_meshy_item_parts_manual_fit_v2(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    profile: &ItemAttachmentProfileV1,
    baseline: &ItemFitReportV4,
    authored_parts: &[ItemManualFitPartV2],
) -> Result<ItemFitReportV4, ItemErrorV1> {
    validate_item_fit_report_v4(baseline)?;
    validate_item_fit_report_v4_against_profile_v1(baseline, profile)?;
    if !matches!(
        baseline.algorithm.as_str(),
        "ITEM_REFERENCE_SLOT_FRAME_FIT_V1" | "ITEM_REFERENCE_MANUAL_FIT_V2"
    ) || sources.len() != 3
        || authored_parts.len() != 3
        || baseline.parts.len() != 3
        || !tolerance.is_finite()
        || tolerance < 0.0
    {
        return Err(error(
            "ITEM-MANUAL-FIT-INPUT-INVALID",
            "authoredParts",
            "manual validation requires one valid reference baseline and three exact ordered source transforms",
        ));
    }

    let mut parts = Vec::with_capacity(3);
    for (index, ((source, authored), fitted)) in sources
        .iter()
        .zip(authored_parts)
        .zip(&baseline.parts)
        .enumerate()
    {
        if source.field != authored.field
            || source.field != fitted.field
            || item_payload_sha256_v1(source.source_glb) != fitted.source_sha256
            || source.source_node != fitted.source_node.as_deref()
            || authored
                .target_space_scale_xyz
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0 || *value > 4.0)
        {
            return Err(error(
                "ITEM-MANUAL-FIT-PART-IDENTITY-MISMATCH",
                format!("authoredParts[{index}]"),
                "manual transform must preserve exact source hashes, field order, sourceNode and finite target-space scale",
            ));
        }
        validate_item_part_transform_v1(authored.transform)?;
        let (bounds, triangle_count) =
            item_fit_bounds_v2(*source, authored.transform, authored.target_space_scale_xyz)?;
        let options = item_fit_options_v2(
            source.source_node,
            authored.transform,
            authored.target_space_scale_xyz,
        );
        parts.push(ItemFitPartV2 {
            field: fitted.field.clone(),
            source_sha256: fitted.source_sha256.clone(),
            source_node: fitted.source_node.clone(),
            triangle_count,
            input_bounds_min: fitted.input_bounds_min,
            input_bounds_max: fitted.input_bounds_max,
            axial_source_axis: fitted.axial_source_axis,
            axial_target_axis: fitted.axial_target_axis,
            target_axial_length: fitted.target_axial_length,
            transform: authored.transform,
            target_space_scale_xyz: authored.target_space_scale_xyz,
            transform_sha256: item_seam_transform_sha256_v1(&options)?,
            output_bounds_min: bounds.min.map(|value| value as f32),
            output_bounds_max: bounds.max.map(|value| value as f32),
            bottom_connector: None,
            top_connector: None,
        });
    }

    let ordered_axis = (0..3)
        .filter_map(|axis| {
            let centers = parts
                .iter()
                .map(|part| (part.output_bounds_min[axis] + part.output_bounds_max[axis]) * 0.5)
                .collect::<Vec<_>>();
            let direction = if centers[0] < centers[1] && centers[1] < centers[2] {
                1_i8
            } else if centers[0] > centers[1] && centers[1] > centers[2] {
                -1_i8
            } else {
                return None;
            };
            let span = (centers[2] - centers[0]).abs();
            (span > 1.0e-5).then_some((axis, direction, span))
        })
        .max_by(|first, second| first.2.total_cmp(&second.2));
    let Some((target_axis, axial_direction, _)) = ordered_axis else {
        return Err(error(
            "ITEM-MANUAL-FIT-ROLE-ORDER-INVALID",
            "authoredParts",
            "manual Bottom, Middle and Top centers must remain strictly ordered on one exact assembly axis",
        ));
    };
    let (target_width_axis, target_depth_axis) = if target_axis == profile.axial_axis as usize {
        (profile.width_axis, profile.depth_axis)
    } else {
        let width_axis = profile.axial_axis;
        let depth_axis = (0_u8..3)
            .find(|axis| *axis as usize != target_axis && *axis != width_axis)
            .ok_or_else(|| {
                error(
                    "ITEM-MANUAL-FIT-FRAME-INVALID",
                    "profile",
                    "manual assembly could not derive a complete target frame",
                )
            })?;
        (width_axis, depth_axis)
    };
    for (index, part) in parts.iter_mut().enumerate() {
        let target_axial_length =
            part.output_bounds_max[target_axis] - part.output_bounds_min[target_axis];
        if !target_axial_length.is_finite() || target_axial_length <= 1.0e-9 {
            return Err(error(
                "ITEM-MANUAL-FIT-AXIS-DEGENERATE",
                format!("authoredParts[{index}]"),
                "manual transform collapses the selected Aurora chain axis",
            ));
        }
        part.axial_target_axis = target_axis as u8;
        part.target_axial_length = target_axial_length;
    }
    let part_count = parts.len();
    for (index, part) in parts.iter_mut().enumerate() {
        let center = std::array::from_fn(|axis| {
            (part.output_bounds_min[axis] + part.output_bounds_max[axis]) * 0.5
        });
        if index > 0 {
            let mut position = center;
            position[target_axis] = if axial_direction > 0 {
                part.output_bounds_min[target_axis]
            } else {
                part.output_bounds_max[target_axis]
            };
            part.bottom_connector = Some(ItemConnectorAnchorV2 {
                kind: "BOTTOM".to_owned(),
                axial_axis: target_axis as u8,
                position,
            });
        }
        if index + 1 < part_count {
            let mut position = center;
            position[target_axis] = if axial_direction > 0 {
                part.output_bounds_max[target_axis]
            } else {
                part.output_bounds_min[target_axis]
            };
            part.top_connector = Some(ItemConnectorAnchorV2 {
                kind: "TOP".to_owned(),
                axial_axis: target_axis as u8,
                position,
            });
        }
    }

    let options = authored_parts
        .iter()
        .zip(sources)
        .map(|(authored, source)| {
            item_fit_options_v2(
                source.source_node,
                authored.transform,
                authored.target_space_scale_xyz,
            )
        })
        .collect::<Vec<_>>();
    let mut adjacent_seams = Vec::with_capacity(2);
    let mut adjacent_connectors = Vec::with_capacity(2);
    for index in 0..2 {
        let seam = measure_meshy_item_seam_v1(
            sources[index].field,
            sources[index].source_glb,
            sources[index].model_resref,
            &options[index],
            sources[index + 1].field,
            sources[index + 1].source_glb,
            sources[index + 1].model_resref,
            &options[index + 1],
            tolerance,
        )?;
        let actual_overlap = parts[index].output_bounds_max[target_axis]
            .min(parts[index + 1].output_bounds_max[target_axis])
            - parts[index].output_bounds_min[target_axis]
                .max(parts[index + 1].output_bounds_min[target_axis]);
        let required_min_overlap = tolerance.max(1.0e-6);
        let reference_span = profile.slots[index].bounds_max[target_axis]
            .min(profile.slots[index + 1].bounds_max[target_axis])
            - profile.slots[index].bounds_min[target_axis]
                .max(profile.slots[index + 1].bounds_min[target_axis]);
        let baseline_max = baseline.adjacent_connectors[index].required_max_overlap;
        let required_max_overlap = reference_span
            .max(required_min_overlap * 2.0)
            .max(baseline_max);
        let overlapping = actual_overlap + 1.0e-6 >= required_min_overlap
            && actual_overlap <= required_max_overlap + 1.0e-6
            && seam.status != "GAP";
        adjacent_connectors.push(ItemAdjacentConnectorV2 {
            first_field: parts[index].field.clone(),
            first_connector: "TOP".to_owned(),
            second_field: parts[index + 1].field.clone(),
            second_connector: "BOTTOM".to_owned(),
            axial_axis: target_axis as u8,
            axial_overlap: actual_overlap,
            required_min_overlap,
            required_max_overlap,
            surface_status: seam.status.clone(),
            status: if overlapping { "OVERLAPPING" } else { "FAILED" }.to_owned(),
        });
        adjacent_seams.push(seam);
    }
    let non_adjacent_measurements = vec![measure_meshy_item_seam_v1(
        sources[0].field,
        sources[0].source_glb,
        sources[0].model_resref,
        &options[0],
        sources[2].field,
        sources[2].source_glb,
        sources[2].model_resref,
        &options[2],
        tolerance,
    )?];
    if adjacent_connectors
        .iter()
        .any(|connector| connector.status != "OVERLAPPING")
        || non_adjacent_measurements
            .iter()
            .any(|measurement| measurement.overlap)
    {
        let adjacent_diagnostics = adjacent_connectors
            .iter()
            .map(|connector| {
                format!(
                    "{}->{} overlap={} required=[{}, {}] surface={} status={}",
                    connector.first_field,
                    connector.second_field,
                    connector.axial_overlap,
                    connector.required_min_overlap,
                    connector.required_max_overlap,
                    connector.surface_status,
                    connector.status,
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(error(
            "ITEM-MANUAL-FIT-CONNECTIONS-FAILED",
            "authoredParts",
            format!(
                "manual transforms must preserve bounded adjacent surface overlap and Bottom/Top separation: {adjacent_diagnostics}"
            ),
        ));
    }

    let mut report = ItemFitReportV4 {
        schema_version: 4,
        algorithm: "ITEM_REFERENCE_MANUAL_FIT_V2".to_owned(),
        status: "PASSED".to_owned(),
        tolerance,
        iterations: baseline.iterations.saturating_add(1),
        reference_profile_sha256: baseline.reference_profile_sha256.clone(),
        common_origin: baseline.common_origin,
        orientation_frame: ItemOrientationFrameV3 {
            target_axial_axis: target_axis as u8,
            target_width_axis,
            target_depth_axis,
            ..baseline.orientation_frame.clone()
        },
        parts,
        adjacent_seams,
        adjacent_connectors,
        non_adjacent_measurements,
        solution_sha256: String::new(),
    };
    let bytes = serde_json::to_vec(&report).map_err(|_| {
        error(
            "ITEM-MANUAL-FIT-REPORT-SERIALIZE-FAILED",
            "report",
            "manual Item fit report could not be serialized",
        )
    })?;
    report.solution_sha256 = item_payload_sha256_v1(&bytes);
    validate_item_fit_report_v4(&report)?;
    validate_item_fit_report_v4_against_profile_v1(&report, profile)?;
    Ok(report)
}

fn fit_meshy_item_parts_on_target_axis_v3(
    sources: &[ItemFitSourceV1<'_>],
    tolerance: f32,
    target_lengths: &[f32],
    axial_target_axis: usize,
    algorithm: &str,
    emit_axial_target_axis: bool,
    rotation_override: Option<[f32; 4]>,
) -> Result<ItemFitReportV1, ItemErrorV1> {
    if !matches!(sources.len(), 1 | 3) {
        return Err(error(
            "ITEM-FIT-SLOT-COUNT-UNSUPPORTED",
            "sources",
            "Item auto-fit supports the one-part and ordered three-part Meshy composer schemas",
        ));
    }
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(error(
            "ITEM-FIT-TOLERANCE-INVALID",
            "tolerance",
            "Item auto-fit tolerance must be finite and non-negative",
        ));
    }
    if target_lengths.len() != sources.len()
        || target_lengths
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        || target_lengths.iter().sum::<f32>() > 10.0
    {
        return Err(error(
            "ITEM-FIT-PROPORTION-CONTRACT-INVALID",
            "targetAxialLengths",
            "target axial lengths must contain one positive finite value per ordered slot and total at most 10 composer units",
        ));
    }
    let mut fields = BTreeSet::new();
    for (index, source) in sources.iter().enumerate() {
        if source.field.trim().is_empty() || !fields.insert(source.field.to_ascii_lowercase()) {
            return Err(error(
                "ITEM-FIT-FIELDS-INVALID",
                format!("sources[{index}].field"),
                "Item auto-fit requires distinct non-empty ordered fields",
            ));
        }
        validate_resref(
            source.model_resref,
            &format!("sources[{index}].modelResref"),
        )?;
    }

    let identities = sources
        .iter()
        .copied()
        .map(|source| item_fit_bounds_v1(source, ItemPartTransformV1::default()))
        .collect::<Result<Vec<_>, _>>()?;
    let long_axes = identities
        .iter()
        .map(|(bounds, _)| item_fit_long_axis_v1(*bounds))
        .collect::<Vec<_>>();
    let axial_source_axis = if sources.len() == 1 || long_axes[0] == long_axes[sources.len() - 1] {
        long_axes[0]
    } else {
        let mut counts = [0_u8; 3];
        for axis in &long_axes {
            counts[*axis] += 1;
        }
        (0..3)
            .max_by_key(|&axis| (counts[axis], std::cmp::Reverse(axis)))
            .unwrap_or(0)
    };
    let rotation = rotation_override.unwrap_or_else(|| match axial_target_axis {
        1 => item_fit_axis_rotation_to_aurora_y_v2(axial_source_axis),
        _ => item_fit_axis_rotation_v1(axial_source_axis),
    });
    let transverse_axes = match axial_target_axis {
        1 => [0, 2],
        _ => [0, 1],
    };
    let mut cursor = 0.0_f64;
    let mut parts = Vec::with_capacity(sources.len());
    let mut options = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().copied().enumerate() {
        let identity_bounds = identities[index].0;
        let axial_extent =
            identity_bounds.max[axial_source_axis] - identity_bounds.min[axial_source_axis];
        if !axial_extent.is_finite() || axial_extent <= 1.0e-9 {
            return Err(error(
                "ITEM-FIT-AXIS-DEGENERATE",
                format!("sources[{index}]"),
                "Item auto-fit could not resolve a positive axial extent",
            ));
        }
        let uniform_scale = f64::from(target_lengths[index]) / axial_extent;
        let base_transform = ItemPartTransformV1 {
            translation: [0.0; 3],
            rotation_xyzw: rotation,
            uniform_scale: uniform_scale as f32,
            pivot: [0.0; 3],
        };
        let rotated_bounds = item_fit_bounds_v1(source, base_transform)?.0;
        let first_transverse_center =
            (rotated_bounds.min[transverse_axes[0]] + rotated_bounds.max[transverse_axes[0]]) * 0.5;
        let second_transverse_center =
            (rotated_bounds.min[transverse_axes[1]] + rotated_bounds.max[transverse_axes[1]]) * 0.5;
        let desired_axial_min = cursor
            + if index == 0 {
                0.0
            } else {
                f64::from(tolerance) * 0.25
            };
        let mut translation = [0.0_f32; 3];
        translation[transverse_axes[0]] = -first_transverse_center as f32;
        translation[transverse_axes[1]] = -second_transverse_center as f32;
        translation[axial_target_axis] =
            (desired_axial_min - rotated_bounds.min[axial_target_axis]) as f32;
        let transform = ItemPartTransformV1 {
            translation,
            ..base_transform
        };
        let output_bounds = ItemSeamBoundsV1 {
            min: [
                rotated_bounds.min[0] + f64::from(translation[0]),
                rotated_bounds.min[1] + f64::from(translation[1]),
                rotated_bounds.min[2] + f64::from(translation[2]),
            ],
            max: [
                rotated_bounds.max[0] + f64::from(translation[0]),
                rotated_bounds.max[1] + f64::from(translation[1]),
                rotated_bounds.max[2] + f64::from(translation[2]),
            ],
        };
        cursor = output_bounds.max[axial_target_axis];
        let part_options = item_fit_options_v1(source.source_node, transform);
        let transform_sha256 = item_seam_transform_sha256_v1(&part_options)?;
        options.push(part_options);
        parts.push(ItemFitPartV1 {
            field: source.field.trim().to_owned(),
            source_sha256: item_payload_sha256_v1(source.source_glb),
            source_node: source.source_node.map(str::to_owned),
            triangle_count: identities[index].1,
            input_bounds_min: identity_bounds.min.map(|value| value as f32),
            input_bounds_max: identity_bounds.max.map(|value| value as f32),
            axial_source_axis: axial_source_axis as u8,
            axial_target_axis: emit_axial_target_axis.then_some(axial_target_axis as u8),
            target_axial_length: target_lengths[index],
            transform,
            transform_sha256,
            output_bounds_min: output_bounds.min.map(|value| value as f32),
            output_bounds_max: output_bounds.max.map(|value| value as f32),
        });
    }

    let mut adjacent_seams = Vec::new();
    for index in 0..sources.len().saturating_sub(1) {
        adjacent_seams.push(measure_meshy_item_seam_v1(
            sources[index].field,
            sources[index].source_glb,
            sources[index].model_resref,
            &options[index],
            sources[index + 1].field,
            sources[index + 1].source_glb,
            sources[index + 1].model_resref,
            &options[index + 1],
            tolerance,
        )?);
    }
    let mut non_adjacent_measurements = Vec::new();
    for first in 0..sources.len() {
        for second in first + 2..sources.len() {
            non_adjacent_measurements.push(measure_meshy_item_seam_v1(
                sources[first].field,
                sources[first].source_glb,
                sources[first].model_resref,
                &options[first],
                sources[second].field,
                sources[second].source_glb,
                sources[second].model_resref,
                &options[second],
                tolerance,
            )?);
        }
    }
    let passed = adjacent_seams
        .iter()
        .all(|measurement| measurement.status == "TOUCHING" && !measurement.overlap)
        && non_adjacent_measurements
            .iter()
            .all(|measurement| !measurement.overlap);
    let mut report = ItemFitReportV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        algorithm: algorithm.to_owned(),
        status: if passed { "PASSED" } else { "MANUAL_REQUIRED" }.to_owned(),
        tolerance,
        iterations: 1,
        parts,
        adjacent_seams,
        non_adjacent_measurements,
        solution_sha256: String::new(),
    };
    let bytes = serde_json::to_vec(&report).map_err(|_| {
        error(
            "ITEM-FIT-REPORT-SERIALIZE-FAILED",
            "report",
            "Item auto-fit report could not be serialized",
        )
    })?;
    report.solution_sha256 = item_payload_sha256_v1(&bytes);
    Ok(report)
}

pub fn validate_item_fit_report_v1(report: &ItemFitReportV1) -> Result<(), ItemErrorV1> {
    let expected_target_axis = match report.algorithm.as_str() {
        "ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V1" => None,
        "ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V2_AURORA_Y" => Some(1),
        _ => {
            return Err(error(
                "ITEM-FIT-REPORT-INVALID",
                "report.algorithm",
                "Item fit report algorithm is unsupported",
            ));
        }
    };
    if report.schema_version != ITEM_SCHEMA_VERSION_V1
        || !matches!(report.status.as_str(), "PASSED" | "MANUAL_REQUIRED")
        || !matches!(report.parts.len(), 1 | 3)
        || !report.tolerance.is_finite()
        || report.tolerance < 0.0
    {
        return Err(error(
            "ITEM-FIT-REPORT-INVALID",
            "report",
            "Item fit report schema, algorithm, status, slot count or tolerance is invalid",
        ));
    }
    for part in &report.parts {
        validate_item_part_transform_v1(part.transform)?;
        if part.field.trim().is_empty()
            || !valid_lowercase_sha256_v1(&part.source_sha256)
            || !valid_lowercase_sha256_v1(&part.transform_sha256)
            || part.axial_target_axis != expected_target_axis
            || !part.target_axial_length.is_finite()
            || part.target_axial_length <= 0.0
        {
            return Err(error(
                "ITEM-FIT-REPORT-PART-INVALID",
                "report.parts",
                "Item fit part binding has an invalid field or SHA-256",
            ));
        }
    }
    if report
        .parts
        .iter()
        .map(|part| part.target_axial_length)
        .sum::<f32>()
        > 10.0
    {
        return Err(error(
            "ITEM-FIT-REPORT-PROPORTION-CONTRACT-INVALID",
            "report.parts.targetAxialLength",
            "Item fit report target axial lengths exceed the 10-unit composer limit",
        ));
    }
    if report.status == "PASSED"
        && (report
            .adjacent_seams
            .iter()
            .any(|seam| seam.status != "TOUCHING" || seam.overlap)
            || report
                .non_adjacent_measurements
                .iter()
                .any(|measurement| measurement.overlap))
    {
        return Err(error(
            "ITEM-FIT-REPORT-SEAM-INVALID",
            "report",
            "A PASSED Item fit report must preserve touching adjacent seams and non-overlapping non-adjacent parts",
        ));
    }
    let mut unhashed = report.clone();
    unhashed.solution_sha256.clear();
    let bytes = serde_json::to_vec(&unhashed).map_err(|_| {
        error(
            "ITEM-FIT-REPORT-SERIALIZE-FAILED",
            "report",
            "Item fit report could not be serialized for hash validation",
        )
    })?;
    if report.solution_sha256 != item_payload_sha256_v1(&bytes) {
        return Err(error(
            "ITEM-FIT-REPORT-HASH-MISMATCH",
            "report.solutionSha256",
            "Item fit report solution hash does not match its exact semantic payload",
        ));
    }
    Ok(())
}

pub fn validate_item_fit_report_v2(report: &ItemFitReportV2) -> Result<(), ItemErrorV1> {
    if report.schema_version != 2
        || report.algorithm != "ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y"
        || !matches!(report.status.as_str(), "PASSED" | "MANUAL_REQUIRED")
        || !matches!(report.parts.len(), 1 | 3)
        || !report.tolerance.is_finite()
        || report.tolerance < 0.0
        || report.adjacent_connectors.len() != report.parts.len().saturating_sub(1)
        || report.adjacent_seams.len() != report.parts.len().saturating_sub(1)
    {
        return Err(error(
            "ITEM-FIT-V2-REPORT-INVALID",
            "report",
            "Item composer fit must use schema 2, the connector-overlap algorithm and a complete ordered slot set",
        ));
    }
    for (index, part) in report.parts.iter().enumerate() {
        validate_item_part_transform_v1(part.transform)?;
        let expected_bottom = index > 0;
        let expected_top = index + 1 < report.parts.len();
        if part.field.trim().is_empty()
            || !valid_lowercase_sha256_v1(&part.source_sha256)
            || !valid_lowercase_sha256_v1(&part.transform_sha256)
            || part
                .target_space_scale_xyz
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
            || part.axial_target_axis != 1
            || !part.target_axial_length.is_finite()
            || part.target_axial_length <= 0.0
            || part.bottom_connector.is_some() != expected_bottom
            || part.top_connector.is_some() != expected_top
        {
            return Err(error(
                "ITEM-FIT-V2-PART-INVALID",
                format!("report.parts[{index}]"),
                "Item composer fit part has an invalid binding, axis or connector set",
            ));
        }
        for (connector, kind, axial) in [
            (
                part.bottom_connector.as_ref(),
                "BOTTOM",
                part.output_bounds_min[1],
            ),
            (
                part.top_connector.as_ref(),
                "TOP",
                part.output_bounds_max[1],
            ),
        ] {
            if let Some(connector) = connector
                && (connector.kind != kind
                    || connector.axial_axis != 1
                    || connector.position.iter().any(|value| !value.is_finite())
                    || (connector.position[1] - axial).abs() > 1.0e-5)
            {
                return Err(error(
                    "ITEM-FIT-V2-CONNECTOR-INVALID",
                    format!("report.parts[{index}].{kind}"),
                    "connector kind, +Y axis or axial anchor does not match the fitted envelope",
                ));
            }
        }
    }
    for (index, connector) in report.adjacent_connectors.iter().enumerate() {
        let first = &report.parts[index];
        let second = &report.parts[index + 1];
        let actual_overlap = first.output_bounds_max[1] - second.output_bounds_min[1];
        let identity_valid = connector.first_field == first.field
            && connector.first_connector == "TOP"
            && connector.second_field == second.field
            && connector.second_connector == "BOTTOM"
            && connector.axial_axis == 1;
        let measurement_valid = connector.axial_overlap.is_finite()
            && connector.required_min_overlap.is_finite()
            && connector.required_max_overlap.is_finite()
            && connector.required_min_overlap > 0.0
            && connector.required_max_overlap >= connector.required_min_overlap
            && (connector.axial_overlap - actual_overlap).abs() <= 1.0e-5;
        let gate_passed = connector.status == "OVERLAPPING"
            && connector.surface_status != "GAP"
            && connector.axial_overlap >= connector.required_min_overlap
            && connector.axial_overlap <= connector.required_max_overlap;
        let status_valid = if report.status == "PASSED" {
            gate_passed
        } else {
            gate_passed || connector.status == "FAILED"
        };
        if !identity_valid || !measurement_valid || !status_valid {
            return Err(error(
                "ITEM-FIT-V2-CONNECTOR-OVERLAP-INVALID",
                format!("report.adjacentConnectors[{index}]"),
                "PASSED adjacent connectors require a positive bounded overlap and no surface gap; MANUAL_REQUIRED reports must retain an explicit FAILED measurement",
            ));
        }
    }
    if report.status == "PASSED"
        && report
            .non_adjacent_measurements
            .iter()
            .any(|measurement| measurement.overlap)
    {
        return Err(error(
            "ITEM-FIT-V2-NONADJACENT-OVERLAP",
            "report.nonAdjacentMeasurements",
            "a PASSED composer fit cannot overlap non-adjacent parts",
        ));
    }
    let mut unhashed = report.clone();
    unhashed.solution_sha256.clear();
    let bytes = item_fit_report_v2_legacy_hash_bytes_v1(&unhashed)?;
    if report.solution_sha256 != item_payload_sha256_v1(&bytes) {
        return Err(error(
            "ITEM-FIT-V2-REPORT-HASH-MISMATCH",
            "report.solutionSha256",
            "Item composer fit solution hash does not match its exact semantic payload",
        ));
    }
    Ok(())
}

pub fn validate_item_fit_report_v3(report: &ItemFitReportV3) -> Result<(), ItemErrorV1> {
    let frame = &report.orientation_frame;
    let distinct_source_axes = {
        let mut axes = [
            frame.source_axial_axis,
            frame.source_width_axis,
            frame.source_depth_axis,
        ];
        axes.sort_unstable();
        axes == [0, 1, 2]
    };
    if report.schema_version != 3
        || report.algorithm != "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX"
        || !matches!(report.status.as_str(), "PASSED" | "MANUAL_REQUIRED")
        || !matches!(report.parts.len(), 1 | 3)
        || !distinct_source_axes
        || frame.target_axial_axis != 1
        || frame.target_width_axis != 2
        || frame.target_depth_axis != 0
        || !frame.width_to_depth_ratio.is_finite()
        || frame.width_to_depth_ratio <= 0.0
        || !frame.handedness_determinant.is_finite()
        || (frame.handedness_determinant - 1.0).abs() > 1.0e-5
        || frame.evidence != "GROUP_NORMALIZED_TRANSVERSE_EXTENTS_WITH_PROPER_HANDEDNESS_V1"
        || !matches!(frame.status.as_str(), "PASSED" | "MANUAL_REQUIRED")
    {
        return Err(error(
            "ITEM-FIT-V3-FRAME-INVALID",
            "report.orientationFrame",
            "full-frame Item fit requires distinct source axes mapped to Aurora depth X, axial Y and width Z with proper handedness",
        ));
    }

    let mut legacy = ItemFitReportV2 {
        schema_version: 2,
        algorithm: "ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y".to_owned(),
        status: report.status.clone(),
        tolerance: report.tolerance,
        iterations: report.iterations,
        parts: report.parts.clone(),
        adjacent_seams: report.adjacent_seams.clone(),
        adjacent_connectors: report.adjacent_connectors.clone(),
        non_adjacent_measurements: report.non_adjacent_measurements.clone(),
        solution_sha256: String::new(),
    };
    let legacy_bytes = item_fit_report_v2_legacy_hash_bytes_v1(&legacy)?;
    legacy.solution_sha256 = item_payload_sha256_v1(&legacy_bytes);
    validate_item_fit_report_v2(&legacy)?;

    let mut composite_min = [f32::INFINITY; 3];
    let mut composite_max = [f32::NEG_INFINITY; 3];
    for (index, part) in report.parts.iter().enumerate() {
        let matrix = item_part_rigid_matrix_v1(part.transform)?;
        let source_axes = [
            (
                frame.source_depth_axis as usize,
                frame.target_depth_axis as usize,
                false,
            ),
            (
                frame.source_axial_axis as usize,
                frame.target_axial_axis as usize,
                true,
            ),
            (
                frame.source_width_axis as usize,
                frame.target_width_axis as usize,
                true,
            ),
        ];
        for (source_axis, target_axis, positive) in source_axes {
            for row in 0..3 {
                let value = matrix[source_axis * 4 + row];
                let expected = if row == target_axis { 1.0 } else { 0.0 };
                let matches = if positive {
                    (value - expected).abs() <= 1.0e-4
                } else {
                    (value.abs() - expected).abs() <= 1.0e-4
                };
                if !matches {
                    return Err(error(
                        "ITEM-FIT-V3-EDGE-ON-FRAME",
                        format!("report.parts[{index}].transform.rotationXyzw"),
                        "part rotation does not map the resolved broad width to Aurora Z and therefore can present the item edge-on",
                    ));
                }
            }
        }
        for axis in 0..3 {
            composite_min[axis] = composite_min[axis].min(part.output_bounds_min[axis]);
            composite_max[axis] = composite_max[axis].max(part.output_bounds_max[axis]);
        }
    }
    let projected_width = composite_max[2] - composite_min[2];
    let projected_depth = composite_max[0] - composite_min[0];
    if report.status == "PASSED"
        && (!projected_width.is_finite()
            || !projected_depth.is_finite()
            || projected_depth <= 0.0
            || projected_width / projected_depth < 1.10)
    {
        return Err(error(
            "ITEM-FIT-V3-EDGE-ON-SILHOUETTE",
            "report.parts.outputBounds",
            "a PASSED full-frame Item fit must expose a wider Z silhouette than its X depth",
        ));
    }
    let expected_status = if frame.status == "PASSED" && legacy.status == "PASSED" {
        "PASSED"
    } else {
        "MANUAL_REQUIRED"
    };
    if report.status != expected_status {
        return Err(error(
            "ITEM-FIT-V3-STATUS-MISMATCH",
            "report.status",
            "full-frame status must combine the orientation and connector gates",
        ));
    }
    let mut unhashed = report.clone();
    unhashed.solution_sha256.clear();
    let bytes = item_fit_report_v3_legacy_hash_bytes_v1(&unhashed)?;
    if report.solution_sha256 != item_payload_sha256_v1(&bytes) {
        return Err(error(
            "ITEM-FIT-V3-REPORT-HASH-MISMATCH",
            "report.solutionSha256",
            "full-frame Item fit solution hash does not match its exact semantic payload",
        ));
    }
    Ok(())
}

pub fn validate_item_fit_report_v4(report: &ItemFitReportV4) -> Result<(), ItemErrorV1> {
    let frame = &report.orientation_frame;
    let mut source_axes = [
        frame.source_axial_axis,
        frame.source_width_axis,
        frame.source_depth_axis,
    ];
    source_axes.sort_unstable();
    let mut target_axes = [
        frame.target_axial_axis,
        frame.target_width_axis,
        frame.target_depth_axis,
    ];
    target_axes.sort_unstable();
    if report.schema_version != 4
        || !matches!(
            report.algorithm.as_str(),
            "ITEM_REFERENCE_SLOT_FRAME_FIT_V1" | "ITEM_REFERENCE_MANUAL_FIT_V2"
        )
        || !matches!(report.status.as_str(), "PASSED" | "MANUAL_REQUIRED")
        || !matches!(report.parts.len(), 1 | 3)
        || !valid_lowercase_sha256_v1(&report.reference_profile_sha256)
        || report.common_origin.iter().any(|value| !value.is_finite())
        || source_axes != [0, 1, 2]
        || target_axes != [0, 1, 2]
        || frame.evidence != "GROUP_NORMALIZED_TRANSVERSE_EXTENTS_WITH_PROPER_HANDEDNESS_V1"
        || !matches!(frame.status.as_str(), "PASSED" | "MANUAL_REQUIRED")
    {
        return Err(error(
            "ITEM-FIT-V4-REPORT-INVALID",
            "report",
            "reference-frame Item fit identity, frame or slot set is invalid",
        ));
    }
    if report.algorithm == "ITEM_REFERENCE_MANUAL_FIT_V2" {
        return validate_item_manual_fit_report_v2(report);
    }
    if frame.target_axial_axis != 1 || frame.target_width_axis != 2 || frame.target_depth_axis != 0
    {
        return Err(error(
            "ITEM-FIT-V4-TARGET-FRAME-INVALID",
            "report.orientationFrame",
            "automatic reference-slot fitting requires Aurora target axes Y/Z/X",
        ));
    }
    let mut connector_view = ItemFitReportV2 {
        schema_version: 2,
        algorithm: "ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y".to_owned(),
        status: report.status.clone(),
        tolerance: report.tolerance,
        iterations: report.iterations,
        parts: report.parts.clone(),
        adjacent_seams: report.adjacent_seams.clone(),
        adjacent_connectors: report.adjacent_connectors.clone(),
        non_adjacent_measurements: report.non_adjacent_measurements.clone(),
        solution_sha256: String::new(),
    };
    let connector_bytes = item_fit_report_v2_legacy_hash_bytes_v1(&connector_view)?;
    connector_view.solution_sha256 = item_payload_sha256_v1(&connector_bytes);
    validate_item_fit_report_v2(&connector_view)?;
    let expected_status = if frame.status == "PASSED" && connector_view.status == "PASSED" {
        "PASSED"
    } else {
        "MANUAL_REQUIRED"
    };
    if report.status != expected_status {
        return Err(error(
            "ITEM-FIT-V4-STATUS-MISMATCH",
            "report.status",
            "reference-frame status must combine orientation and connector gates",
        ));
    }
    let mut unhashed = report.clone();
    unhashed.solution_sha256.clear();
    let bytes = serde_json::to_vec(&unhashed).map_err(|_| {
        error(
            "ITEM-FIT-V4-REPORT-SERIALIZE-FAILED",
            "report",
            "reference-frame Item fit report could not be serialized for hash validation",
        )
    })?;
    if report.solution_sha256 != item_payload_sha256_v1(&bytes) {
        return Err(error(
            "ITEM-FIT-V4-REPORT-HASH-MISMATCH",
            "report.solutionSha256",
            "reference-frame Item fit hash does not match its semantic payload",
        ));
    }
    Ok(())
}

fn validate_item_manual_fit_report_v2(report: &ItemFitReportV4) -> Result<(), ItemErrorV1> {
    let target_axis = report.orientation_frame.target_axial_axis as usize;
    if report.status != "PASSED"
        || report.orientation_frame.status != "PASSED"
        || report.parts.len() != 3
        || report.adjacent_seams.len() != 2
        || report.adjacent_connectors.len() != 2
        || report.non_adjacent_measurements.len() != 1
    {
        return Err(error(
            "ITEM-MANUAL-FIT-REPORT-INVALID",
            "report",
            "manual fit requires three exact parts, two passed adjacent interfaces and one separated non-adjacent pair",
        ));
    }
    let expected_fields = ["ModelPart1", "ModelPart2", "ModelPart3"];
    let centers = report
        .parts
        .iter()
        .map(|part| {
            (part.output_bounds_min[target_axis] + part.output_bounds_max[target_axis]) * 0.5
        })
        .collect::<Vec<_>>();
    let axial_direction = if centers[0] < centers[1] && centers[1] < centers[2] {
        1_i8
    } else if centers[0] > centers[1] && centers[1] > centers[2] {
        -1_i8
    } else {
        return Err(error(
            "ITEM-MANUAL-FIT-ROLE-ORDER-INVALID",
            "report.parts",
            "manual Bottom, Middle and Top centers are not ordered on the declared assembly axis",
        ));
    };
    for (index, (part, expected_field)) in report.parts.iter().zip(expected_fields).enumerate() {
        validate_item_part_transform_v1(part.transform)?;
        if part.field != expected_field
            || !valid_lowercase_sha256_v1(&part.source_sha256)
            || !valid_lowercase_sha256_v1(&part.transform_sha256)
            || part.axial_target_axis as usize != target_axis
            || !part.target_axial_length.is_finite()
            || part.target_axial_length <= 0.0
            || part
                .target_space_scale_xyz
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0 || *value > 4.0)
            || part.bottom_connector.is_some() != (index > 0)
            || part.top_connector.is_some() != (index + 1 < report.parts.len())
        {
            return Err(error(
                "ITEM-MANUAL-FIT-PART-INVALID",
                format!("report.parts[{index}]"),
                "manual fit part identity, transform, target axis or connector set is invalid",
            ));
        }
        for (connector, kind, expected_axial) in [
            (
                part.bottom_connector.as_ref(),
                "BOTTOM",
                if axial_direction > 0 {
                    part.output_bounds_min[target_axis]
                } else {
                    part.output_bounds_max[target_axis]
                },
            ),
            (
                part.top_connector.as_ref(),
                "TOP",
                if axial_direction > 0 {
                    part.output_bounds_max[target_axis]
                } else {
                    part.output_bounds_min[target_axis]
                },
            ),
        ] {
            if let Some(connector) = connector
                && (connector.kind != kind
                    || connector.axial_axis as usize != target_axis
                    || connector.position.iter().any(|value| !value.is_finite())
                    || (connector.position[target_axis] - expected_axial).abs() > 1.0e-5)
            {
                return Err(error(
                    "ITEM-MANUAL-FIT-CONNECTOR-INVALID",
                    format!("report.parts[{index}].{kind}"),
                    "manual connector does not match the declared assembly direction and endpoint",
                ));
            }
        }
    }
    for (index, connector) in report.adjacent_connectors.iter().enumerate() {
        let first = &report.parts[index];
        let second = &report.parts[index + 1];
        let actual_overlap = first.output_bounds_max[target_axis]
            .min(second.output_bounds_max[target_axis])
            - first.output_bounds_min[target_axis].max(second.output_bounds_min[target_axis]);
        if connector.first_field != first.field
            || connector.first_connector != "TOP"
            || connector.second_field != second.field
            || connector.second_connector != "BOTTOM"
            || connector.axial_axis as usize != target_axis
            || connector.status != "OVERLAPPING"
            || connector.surface_status == "GAP"
            || !connector.axial_overlap.is_finite()
            || !connector.required_min_overlap.is_finite()
            || !connector.required_max_overlap.is_finite()
            || connector.required_min_overlap <= 0.0
            || connector.required_max_overlap < connector.required_min_overlap
            || connector.axial_overlap + 1.0e-6 < connector.required_min_overlap
            || connector.axial_overlap > connector.required_max_overlap + 1.0e-6
            || (connector.axial_overlap - actual_overlap).abs() > 1.0e-5
        {
            return Err(error(
                "ITEM-MANUAL-FIT-CONNECTIONS-FAILED",
                format!("report.adjacentConnectors[{index}]"),
                "manual adjacent parts require bounded axial and real surface overlap",
            ));
        }
    }
    if report.non_adjacent_measurements[0].overlap {
        return Err(error(
            "ITEM-MANUAL-FIT-NONADJACENT-OVERLAP",
            "report.nonAdjacentMeasurements[0]",
            "manual Bottom and Top parts must remain separated",
        ));
    }
    let mut unhashed = report.clone();
    unhashed.solution_sha256.clear();
    let bytes = serde_json::to_vec(&unhashed).map_err(|_| {
        error(
            "ITEM-MANUAL-FIT-REPORT-SERIALIZE-FAILED",
            "report",
            "manual Item fit report could not be serialized for hash validation",
        )
    })?;
    if report.solution_sha256 != item_payload_sha256_v1(&bytes) {
        return Err(error(
            "ITEM-MANUAL-FIT-REPORT-HASH-MISMATCH",
            "report.solutionSha256",
            "manual Item fit hash does not match its semantic payload",
        ));
    }
    Ok(())
}

pub fn validate_item_fit_report_v4_against_profile_v1(
    report: &ItemFitReportV4,
    profile: &ItemAttachmentProfileV1,
) -> Result<(), ItemErrorV1> {
    validate_item_fit_report_v4(report)?;
    validate_item_attachment_profile_v1(profile)?;
    if report.reference_profile_sha256 != profile.profile_sha256
        || report.common_origin != profile.common_origin
        || report.parts.len() != profile.slots.len()
    {
        return Err(error(
            "ITEM-FIT-REFERENCE-IDENTITY-MISMATCH",
            "report.referenceProfileSha256",
            "fit report is not bound to the selected reference attachment profile",
        ));
    }
    if report.algorithm == "ITEM_REFERENCE_MANUAL_FIT_V2" {
        return validate_item_reference_manual_fit_against_profile_v2(report, profile);
    }
    for (index, (part, slot)) in report.parts.iter().zip(&profile.slots).enumerate() {
        let target_center_x = (slot.bounds_min[0] + slot.bounds_max[0]) * 0.5;
        let target_center_z = (slot.bounds_min[2] + slot.bounds_max[2]) * 0.5;
        let output_center_x = (part.output_bounds_min[0] + part.output_bounds_max[0]) * 0.5;
        let output_center_z = (part.output_bounds_min[2] + part.output_bounds_max[2]) * 0.5;
        let anchored_axial_min = !slot.allow_axial_extension_at_min;
        let anchored_axial_max = !slot.allow_axial_extension_at_max;
        if part.field != slot.field
            || (part.target_space_scale_xyz[0] - part.target_space_scale_xyz[2]).abs() > 1.0e-6
            || (part.target_space_scale_xyz[1] - 1.0).abs() > 1.0e-6
            || part.target_space_scale_xyz[0] > 1.0 + 1.0e-6
            || (anchored_axial_min
                && (part.output_bounds_min[1] - slot.bounds_min[1]).abs() > 1.0e-5)
            || (anchored_axial_max
                && (part.output_bounds_max[1] - slot.bounds_max[1]).abs() > 1.0e-5)
            || (output_center_x - target_center_x).abs() > 1.0e-5
            || (output_center_z - target_center_z).abs() > 1.0e-5
            || part.output_bounds_min[0] < slot.bounds_min[0] - 1.0e-5
            || part.output_bounds_max[0] > slot.bounds_max[0] + 1.0e-5
            || part.output_bounds_min[2] < slot.bounds_min[2] - 1.0e-5
            || part.output_bounds_max[2] > slot.bounds_max[2] + 1.0e-5
        {
            return Err(error(
                "ITEM-FIT-REFERENCE-SLOT-BOUNDS-MISMATCH",
                format!("report.parts[{index}].outputBounds"),
                "generated part does not preserve the selected reference slot axial range, transverse center and X/Z envelope",
            ));
        }
    }
    Ok(())
}

fn validate_item_reference_manual_fit_against_profile_v2(
    report: &ItemFitReportV4,
    profile: &ItemAttachmentProfileV1,
) -> Result<(), ItemErrorV1> {
    let target_axis = report.orientation_frame.target_axial_axis as usize;
    let width_axis = report.orientation_frame.target_width_axis as usize;
    let depth_axis = report.orientation_frame.target_depth_axis as usize;
    let mut reference_min = [f32::INFINITY; 3];
    let mut reference_max = [f32::NEG_INFINITY; 3];
    for slot in &profile.slots {
        for axis in 0..3 {
            reference_min[axis] = reference_min[axis].min(slot.bounds_min[axis]);
            reference_max[axis] = reference_max[axis].max(slot.bounds_max[axis]);
        }
    }
    let reference_span = reference_max[target_axis] - reference_min[target_axis];
    let hand_anchor = profile.slots.get(1).ok_or_else(|| {
        error(
            "ITEM-MANUAL-FIT-HAND-SLOT-MISSING",
            "profile.slots",
            "manual three-part validation requires the Middle HAND slot",
        )
    })?;
    let mut composite_min = [f32::INFINITY; 3];
    let mut composite_max = [f32::NEG_INFINITY; 3];
    for (index, (part, slot)) in report.parts.iter().zip(&profile.slots).enumerate() {
        if part.field != slot.field {
            return Err(error(
                "ITEM-MANUAL-FIT-REFERENCE-SLOT-MISMATCH",
                format!("report.parts[{index}].field"),
                "manual fit must preserve the exact reference slot order",
            ));
        }
        for axis in 0..3 {
            composite_min[axis] = composite_min[axis].min(part.output_bounds_min[axis]);
            composite_max[axis] = composite_max[axis].max(part.output_bounds_max[axis]);
        }
        for axis in [width_axis, depth_axis] {
            let center = (part.output_bounds_min[axis] + part.output_bounds_max[axis]) * 0.5;
            let reference_extent = reference_max[axis] - reference_min[axis];
            let within_guard = center >= reference_min[axis] - reference_extent * 0.25
                && center <= reference_max[axis] + reference_extent * 0.25
                && part.output_bounds_max[axis] - part.output_bounds_min[axis]
                    <= reference_extent * 2.0 + 1.0e-5;
            let hand_preserved = index != 1
                || ((center - hand_anchor.controller_translation[axis]).abs() <= 1.0e-5
                    && part.output_bounds_min[axis] >= reference_min[axis] - 1.0e-5
                    && part.output_bounds_max[axis] <= reference_max[axis] + 1.0e-5);
            if !within_guard || !hand_preserved {
                return Err(error(
                    "ITEM-MANUAL-FIT-TRANSVERSE-GUARD-FAILED",
                    format!("report.parts[{index}].outputBounds"),
                    "manual Bottom/Top must stay in the guarded reference envelope and Middle must preserve the HAND pivot",
                ));
            }
        }
    }
    let composite_span = composite_max[target_axis] - composite_min[target_axis];
    if !composite_span.is_finite()
        || composite_span < reference_span * 0.80
        || composite_span > reference_span + 1.0e-5
    {
        return Err(error(
            "ITEM-MANUAL-FIT-LENGTH-GUARD-FAILED",
            "report.parts.outputBounds",
            "manual composite must retain 80%..100% of the reference longitudinal span",
        ));
    }
    let hand_pair_contains_origin = report.parts[..2].iter().any(|part| {
        profile.common_origin[target_axis] >= part.output_bounds_min[target_axis] - 1.0e-5
            && profile.common_origin[target_axis] <= part.output_bounds_max[target_axis] + 1.0e-5
    });
    if !hand_pair_contains_origin {
        return Err(error(
            "ITEM-MANUAL-FIT-HAND-ANCHOR-FAILED",
            "report.commonOrigin",
            "manual Bottom/Middle pair must keep the exact reference Item origin",
        ));
    }
    Ok(())
}

pub fn validate_item_fit_report_v3_against_profile_v1(
    report: &ItemFitReportV3,
    profile: &ItemAttachmentProfileV1,
) -> Result<(), ItemErrorV1> {
    validate_item_fit_report_v3(report)?;
    validate_item_attachment_profile_v1(profile)?;
    if report.parts.len() != profile.slots.len() {
        return Err(error(
            "ITEM-FIT-REFERENCE-SLOT-COUNT",
            "report.parts",
            "legacy fit report does not contain the selected reference slot set",
        ));
    }
    for (index, (part, slot)) in report.parts.iter().zip(&profile.slots).enumerate() {
        if part.field != slot.field
            || (part.output_bounds_min[1] - slot.bounds_min[1]).abs() > 1.0e-5
            || (part.output_bounds_max[1] - slot.bounds_max[1]).abs() > 1.0e-5
        {
            return Err(error(
                "ITEM-FIT-REFERENCE-SLOT-BOUNDS-MISMATCH",
                format!("report.parts[{index}].outputBounds"),
                "legacy cursor-based fit does not preserve the selected reference slot frame",
            ));
        }
    }
    Ok(())
}

pub fn validate_item_modeltype2_aurora_append_conformance_v2(
    inputs: &[ItemComposerMdlInputV2<'_>],
    fit: &ItemFitReportV2,
) -> Result<ItemComposerConformanceReportV2, ItemErrorV1> {
    validate_item_fit_report_v2(fit)?;
    validate_item_modeltype2_aurora_append_payloads_v1(inputs, &fit.parts)
}

fn validate_item_modeltype2_aurora_append_payloads_v1(
    inputs: &[ItemComposerMdlInputV2<'_>],
    fitted_parts: &[ItemFitPartV2],
) -> Result<ItemComposerConformanceReportV2, ItemErrorV1> {
    if inputs.len() != 3 || fitted_parts.len() != 3 {
        return Err(error(
            "ITEM-MODELTYPE2-COMPOSER-SLOT-COUNT",
            "inputs",
            "Aurora ModelType 2 append emulation requires Bottom, Middle and Top",
        ));
    }
    let expected_fields = ["ModelPart1", "ModelPart2", "ModelPart3"];
    let mut global_names = BTreeSet::new();
    let mut parts = Vec::with_capacity(3);
    let mut composite_min = [f32::INFINITY; 3];
    let mut composite_max = [f32::NEG_INFINITY; 3];
    let mut total_triangle_count = 0_usize;
    for (index, ((input, fitted), expected_field)) in inputs
        .iter()
        .zip(fitted_parts)
        .zip(expected_fields)
        .enumerate()
    {
        if !input.field.eq_ignore_ascii_case(expected_field)
            || fitted.field != expected_field
            || input.model_resref.is_empty()
        {
            return Err(error(
                "ITEM-MODELTYPE2-APPEND-ORDER",
                format!("inputs[{index}]"),
                "composer inputs must be ordered ModelPart1, ModelPart2, ModelPart3",
            ));
        }
        let inspection = inspect_binary_mdl(input.mdl_payload).map_err(|source| {
            error(
                "ITEM-MODELTYPE2-MDL-READBACK",
                format!("inputs[{index}].mdlPayload"),
                format!("{} at byte {}", source.code, source.offset),
            )
        })?;
        if inspection.model.name != input.model_resref
            || inspection.node_tree.roots.len() != 1
            || !inspection.diagnostics.is_empty()
            || !inspection.unsupported.is_empty()
        {
            return Err(error(
                "ITEM-MODELTYPE2-MDL-IDENTITY",
                format!("inputs[{index}]"),
                "binary MDL identity, root count or parser diagnostics do not match the declared part",
            ));
        }
        let root = &inspection.node_tree.roots[0];
        if root.name != input.model_resref || root.content_flags != 0x01 {
            return Err(error(
                "ITEM-MODELTYPE2-ROOT-INVALID",
                format!("inputs[{index}].root"),
                "Item part root must be the controllerless model-named generic node",
            ));
        }
        if !root.controllers.is_empty()
            || root.controller_keys_header.pointer != 0
            || root.controller_keys_header.used != 0
            || root.controller_data_header.pointer != 0
            || root.controller_data_header.used != 0
        {
            return Err(error(
                "ITEM-MODELTYPE2-ROOT-CONTROLLERS",
                format!("inputs[{index}].root.controllers"),
                "Aurora appends Middle and Top without applying their root controllers; ModelType 2 roots must be controllerless",
            ));
        }
        if root.children.is_empty()
            || root.children.len() + 1 != inspection.node_tree.node_count
            || root
                .children
                .iter()
                .any(|child| child.mesh.is_none() || !child.children.is_empty())
        {
            return Err(error(
                "ITEM-MODELTYPE2-HIERARCHY",
                format!("inputs[{index}].root.children"),
                "every output node below the root must be a direct Trimesh child",
            ));
        }
        let expected_header_min = [-5.0, -5.0, -1.0];
        let expected_header_max = [5.0, 5.0, 10.0];
        let header_min = [
            inspection.model.bounds_min.x,
            inspection.model.bounds_min.y,
            inspection.model.bounds_min.z,
        ];
        let header_max = [
            inspection.model.bounds_max.x,
            inspection.model.bounds_max.y,
            inspection.model.bounds_max.z,
        ];
        if (0..3).any(|axis| {
            (header_min[axis] - expected_header_min[axis]).abs() > 1.0e-5
                || (header_max[axis] - expected_header_max[axis]).abs() > 1.0e-5
        }) {
            return Err(error(
                "ITEM-MODELTYPE2-MODEL-HEADER-PROFILE",
                format!("inputs[{index}].model.bounds"),
                "ModelType 2 output must use the retail weapon-part model-header envelope",
            ));
        }
        let mut part_min = [f32::INFINITY; 3];
        let mut part_max = [f32::NEG_INFINITY; 3];
        let mut mesh_node_names = Vec::with_capacity(root.children.len());
        let expected_single_name = format!("g_{}", input.model_resref);
        for (mesh_index, child) in root.children.iter().enumerate() {
            let expected_name = if root.children.len() == 1 {
                expected_single_name.clone()
            } else {
                format!("g_{}_{:02}", input.model_resref, mesh_index + 1)
            };
            if child.name != expected_name
                || child.content_flags != 0x21
                || !global_names.insert(child.name.to_ascii_lowercase())
            {
                return Err(error(
                    "ITEM-MODELTYPE2-MESH-NAME",
                    format!("inputs[{index}].root.children[{mesh_index}].name"),
                    "Trimesh names must be resref-derived and globally unique across appended parts",
                ));
            }
            let position = child
                .controllers
                .iter()
                .find(|controller| controller.controller_type == 8)
                .and_then(|controller| controller.values.first())
                .filter(|value| value.len() == 3)
                .ok_or_else(|| {
                    error(
                        "ITEM-MODELTYPE2-MESH-CONTROLLERS",
                        format!("inputs[{index}].root.children[{mesh_index}].controllers"),
                        "each Trimesh child requires one position controller",
                    )
                })?;
            let orientation = child
                .controllers
                .iter()
                .find(|controller| controller.controller_type == 20)
                .and_then(|controller| controller.values.first())
                .filter(|value| value.len() == 4)
                .ok_or_else(|| {
                    error(
                        "ITEM-MODELTYPE2-MESH-CONTROLLERS",
                        format!("inputs[{index}].root.children[{mesh_index}].controllers"),
                        "each Trimesh child requires one orientation controller",
                    )
                })?;
            if child.controllers.len() != 2
                || (0..3).any(|axis| {
                    (position[axis] - fitted.transform.translation[axis]).abs() > 1.0e-4
                })
                || (0..4)
                    .any(|axis| (orientation[axis] - [0.0, 0.0, 0.0, 1.0][axis]).abs() > 1.0e-5)
            {
                return Err(error(
                    "ITEM-MODELTYPE2-MESH-CONTROLLERS",
                    format!("inputs[{index}].root.children[{mesh_index}].controllers"),
                    "assembly translation must belong to Trimesh and geometry normalization must leave an identity Trimesh orientation",
                ));
            }
            let mesh = child.mesh.as_ref().expect("validated Trimesh child");
            total_triangle_count += mesh.faces.len();
            for vertex in &mesh.vertices {
                let point = [
                    vertex.x + position[0],
                    vertex.y + position[1],
                    vertex.z + position[2],
                ];
                for axis in 0..3 {
                    part_min[axis] = part_min[axis].min(point[axis]);
                    part_max[axis] = part_max[axis].max(point[axis]);
                }
            }
            mesh_node_names.push(child.name.clone());
        }
        if (0..3).any(|axis| {
            (part_min[axis] - fitted.output_bounds_min[axis]).abs() > 1.0e-4
                || (part_max[axis] - fitted.output_bounds_max[axis]).abs() > 1.0e-4
        }) || part_max[1] <= part_min[1]
        {
            return Err(error(
                "ITEM-MODELTYPE2-COMPOSED-BOUNDS",
                format!("inputs[{index}]"),
                "node-aware composed geometry bounds differ from the connector-qualified +Y fit",
            ));
        }
        for axis in 0..3 {
            composite_min[axis] = composite_min[axis].min(part_min[axis]);
            composite_max[axis] = composite_max[axis].max(part_max[axis]);
        }
        let triangle_count = root
            .children
            .iter()
            .filter_map(|child| child.mesh.as_ref())
            .map(|mesh| mesh.faces.len())
            .sum::<usize>();
        if triangle_count != fitted.triangle_count {
            return Err(error(
                "ITEM-MODELTYPE2-TRIANGLE-COUNT",
                format!("inputs[{index}]"),
                "composer validation must preserve every fitted source triangle",
            ));
        }
        parts.push(ItemComposerPartConformanceV2 {
            field: expected_field.to_owned(),
            model_resref: input.model_resref.to_owned(),
            root_node_name: root.name.clone(),
            root_controller_owner: "NONE".to_owned(),
            transform_controller_owner: "TRIMESH_CHILD".to_owned(),
            mesh_node_names,
            triangle_count,
            bounds_min: part_min,
            bounds_max: part_max,
        });
    }
    Ok(ItemComposerConformanceReportV2 {
        schema_version: 2,
        algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2".to_owned(),
        status: "PASSED".to_owned(),
        append_order: expected_fields.map(str::to_owned).to_vec(),
        parts,
        composite_bounds_min: composite_min,
        composite_bounds_max: composite_max,
        total_triangle_count,
    })
}

pub fn validate_item_modeltype2_aurora_append_conformance_v3(
    inputs: &[ItemComposerMdlInputV2<'_>],
    fit: &ItemFitReportV3,
) -> Result<ItemComposerConformanceReportV2, ItemErrorV1> {
    validate_item_fit_report_v3(fit)?;
    let mut legacy = ItemFitReportV2 {
        schema_version: 2,
        algorithm: "ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y".to_owned(),
        status: fit.status.clone(),
        tolerance: fit.tolerance,
        iterations: fit.iterations,
        parts: fit.parts.clone(),
        adjacent_seams: fit.adjacent_seams.clone(),
        adjacent_connectors: fit.adjacent_connectors.clone(),
        non_adjacent_measurements: fit.non_adjacent_measurements.clone(),
        solution_sha256: String::new(),
    };
    let bytes = item_fit_report_v2_legacy_hash_bytes_v1(&legacy)?;
    legacy.solution_sha256 = item_payload_sha256_v1(&bytes);
    validate_item_modeltype2_aurora_append_conformance_v2(inputs, &legacy)
}

pub fn validate_item_modeltype2_aurora_append_conformance_v4(
    inputs: &[ItemComposerMdlInputV2<'_>],
    fit: &ItemFitReportV4,
) -> Result<ItemComposerConformanceReportV2, ItemErrorV1> {
    validate_item_fit_report_v4(fit)?;
    if fit.algorithm == "ITEM_REFERENCE_MANUAL_FIT_V2" {
        return validate_item_modeltype2_aurora_append_payloads_v1(inputs, &fit.parts);
    }
    let mut legacy = ItemFitReportV2 {
        schema_version: 2,
        algorithm: "ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y".to_owned(),
        status: fit.status.clone(),
        tolerance: fit.tolerance,
        iterations: fit.iterations,
        parts: fit.parts.clone(),
        adjacent_seams: fit.adjacent_seams.clone(),
        adjacent_connectors: fit.adjacent_connectors.clone(),
        non_adjacent_measurements: fit.non_adjacent_measurements.clone(),
        solution_sha256: String::new(),
    };
    let bytes = item_fit_report_v2_legacy_hash_bytes_v1(&legacy)?;
    legacy.solution_sha256 = item_payload_sha256_v1(&bytes);
    validate_item_modeltype2_aurora_append_conformance_v2(inputs, &legacy)
}

#[allow(clippy::too_many_arguments)]
pub fn measure_meshy_item_seam_v1(
    first_field: &str,
    first_source_glb: &[u8],
    first_model_resref: &str,
    first_options: &ItemPartBuildOptionsV2,
    second_field: &str,
    second_source_glb: &[u8],
    second_model_resref: &str,
    second_options: &ItemPartBuildOptionsV2,
    tolerance: f32,
) -> Result<ItemSeamMeasurementV1, ItemErrorV1> {
    if first_field.trim().is_empty()
        || second_field.trim().is_empty()
        || first_field.eq_ignore_ascii_case(second_field)
    {
        return Err(error(
            "ITEM-SEAM-FIELDS-INVALID",
            "fields",
            "authoritative seam measurement requires two distinct non-empty part fields",
        ));
    }
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(error(
            "ITEM-SEAM-TOLERANCE-INVALID",
            "tolerance",
            "seam tolerance must be finite and non-negative",
        ));
    }
    if first_options.schema_version != ITEM_SCHEMA_VERSION_V1
        || second_options.schema_version != ITEM_SCHEMA_VERSION_V1
    {
        return Err(error(
            "ITEM-SEAM-OPTIONS-SCHEMA-INVALID",
            "options.schemaVersion",
            "seam part options schemaVersion must be 1",
        ));
    }
    let first_model_resref = validate_resref(first_model_resref, "first.modelResref")?;
    let second_model_resref = validate_resref(second_model_resref, "second.modelResref")?;
    validate_item_part_transform_v1(first_options.transform)?;
    validate_item_part_transform_v1(second_options.transform)?;
    let internal = |options: &ItemPartBuildOptionsV2| ItemPartBuildInternalOptionsV1 {
        schema_version: options.schema_version,
        transform: options.transform,
        source_node: options.source_node.clone(),
        texture_encoding: options.texture_encoding,
        icon_size: None,
        icon_projection_bounds: None,
        weapon_color: None,
        target_space_scale_xyz: options.target_space_scale_xyz,
        geometry_derived_icon: true,
        aurora_composer_v2: true,
    };
    let first = prepare_meshy_item_part_v1(
        first_source_glb,
        &first_model_resref,
        &internal(first_options),
    )?;
    let second = prepare_meshy_item_part_v1(
        second_source_glb,
        &second_model_resref,
        &internal(second_options),
    )?;
    let first_triangles = item_seam_model_triangles_v1(&first.model)?;
    let second_triangles = item_seam_model_triangles_v1(&second.model)?;
    let first_bvh = build_item_seam_bvh_v1(&first_triangles, (0..first_triangles.len()).collect());
    let second_bvh =
        build_item_seam_bvh_v1(&second_triangles, (0..second_triangles.len()).collect());
    let extent = item_seam_bounds_union_v1(first_bvh.bounds(), second_bvh.bounds());
    let scale = (0..3)
        .map(|axis| extent.max[axis] - extent.min[axis])
        .fold(1.0_f64, f64::max);
    let intersection_epsilon = (scale * 1.0e-8).max(1.0e-9);
    let mut best_distance_squared = f64::INFINITY;
    let mut overlap = false;
    visit_item_seam_bvh_pair_v1(
        &first_bvh,
        &first_triangles,
        &second_bvh,
        &second_triangles,
        intersection_epsilon,
        &mut best_distance_squared,
        &mut overlap,
    );
    if !best_distance_squared.is_finite() {
        return Err(error(
            "ITEM-SEAM-MEASUREMENT-NONFINITE",
            "geometry",
            "authoritative triangle-surface seam measurement produced no finite distance",
        ));
    }
    if !overlap {
        overlap = item_seam_closed_mesh_contains_surface_point_v1(
            &first_triangles,
            &second_triangles,
            &second_bvh,
            intersection_epsilon,
        ) || item_seam_closed_mesh_contains_surface_point_v1(
            &second_triangles,
            &first_triangles,
            &first_bvh,
            intersection_epsilon,
        ) || item_seam_closed_meshes_share_interior_v1(
            &first_triangles,
            &first_bvh,
            &second_triangles,
            &second_bvh,
            intersection_epsilon,
        );
    }
    let gap = best_distance_squared.sqrt() as f32;
    let status = if overlap {
        "OVERLAP"
    } else if gap <= tolerance {
        "TOUCHING"
    } else {
        "GAP"
    };
    let mut report = ItemSeamMeasurementV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        algorithm: "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1".to_owned(),
        first_field: first_field.trim().to_owned(),
        second_field: second_field.trim().to_owned(),
        first_source_sha256: item_payload_sha256_v1(first_source_glb),
        second_source_sha256: item_payload_sha256_v1(second_source_glb),
        first_transform_sha256: item_seam_transform_sha256_v1(first_options)?,
        second_transform_sha256: item_seam_transform_sha256_v1(second_options)?,
        first_triangle_count: first.triangle_count,
        second_triangle_count: second.triangle_count,
        tolerance,
        status: status.to_owned(),
        gap,
        overlap,
        measurement_sha256: String::new(),
    };
    let measurement_bytes = serde_json::to_vec(&report).map_err(|_| {
        error(
            "ITEM-SEAM-REPORT-SERIALIZE-FAILED",
            "report",
            "authoritative seam report could not be serialized",
        )
    })?;
    report.measurement_sha256 = item_payload_sha256_v1(&measurement_bytes);
    Ok(report)
}

fn apply_item_weapon_colorway_v1(image: &mut TgaImageV1, color: u8) -> Result<(), ItemErrorV1> {
    // Reuse the authoritative native color-domain validation. Model zero is
    // sufficient because this operation authors only the concrete texture.
    encode_item_weapon_part_appearance_v1(0, color)?;
    if color == 1 {
        return Ok(());
    }
    let channels = match image.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3,
        TgaPixelFormatV1::Rgba8 => 4,
    };
    let channel = |value: u16| value.min(255) as u8;
    for pixel in image.pixels.chunks_exact_mut(channels) {
        let [red, green, blue] = [
            u16::from(pixel[0]),
            u16::from(pixel[1]),
            u16::from(pixel[2]),
        ];
        let resolved = match color {
            // Cool steel: slightly desaturated red/green and a lifted blue.
            2 => [red * 3 / 4, green * 7 / 8, blue * 9 / 8],
            // Bronze: warm highlights without replacing texture detail.
            3 => [red * 9 / 8, green * 3 / 4, blue / 2],
            // Blackened steel: retain source detail at half intensity.
            4 => [red / 2, green / 2, blue / 2],
            _ => unreachable!("weapon color domain was validated above"),
        };
        pixel[0] = channel(resolved[0]);
        pixel[1] = channel(resolved[1]);
        pixel[2] = channel(resolved[2]);
    }
    Ok(())
}

fn build_meshy_item_part_with_internal_options_v1(
    source_glb: &[u8],
    model_resref: &str,
    texture_resref: &str,
    options: &ItemPartBuildInternalOptionsV1,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    if options.schema_version != ITEM_SCHEMA_VERSION_V1 {
        return Err(error(
            "ITEM-PART-OPTIONS-SCHEMA-INVALID",
            "options.schemaVersion",
            "item part build options schemaVersion must be 1",
        ));
    }
    let model_resref = validate_resref(model_resref, "modelResref")?;
    let texture_resref = validate_resref(texture_resref, "textureResref")?;
    validate_item_part_transform_v1(options.transform)?;
    if options
        .target_space_scale_xyz
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
        || (!options.aurora_composer_v2
            && options.target_space_scale_xyz != item_unit_scale_xyz_v1())
    {
        return Err(error(
            "ITEM-PART-TARGET-SPACE-SCALE-INVALID",
            "options.targetSpaceScaleXyz",
            "target-space scale must be positive and is supported only by the Aurora composer build route",
        ));
    }

    let limits = static_placeable_glb_limits_v1();
    let PreparedItemPartV1 {
        ingest,
        mut model,
        source_node,
        triangle_count,
        degenerate_triangle_count_removed,
    } = prepare_meshy_item_part_v1(source_glb, &model_resref, options)?;
    if model.material_source_bindings.len() > 1 {
        return Err(error(
            "ITEM-PART-MULTI-MATERIAL-UNSUPPORTED",
            "model.materialSourceBindings",
            "this Item part resolved more than one material binding, but the current artifact contract emits exactly one texture resource",
        ));
    }
    let texture_selection =
        resolve_base_color_image_index_v1(&ingest, &model).map_err(|source| {
            error(
                &format!("ITEM-{}", source.code),
                source.path,
                source.message,
            )
        })?;
    let mut texture_image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            &format!("ITEM-{}", source.code),
            source.json_path.unwrap_or_else(|| "images".to_owned()),
            source.message,
        )
    })?;
    if let Some(color) = options.weapon_color {
        if options.texture_encoding != ItemPartTextureEncodingV1::DirectColor {
            return Err(error(
                "ITEM-WEAPON-COLORWAY-PLT-UNSUPPORTED",
                "options.textureEncoding",
                "ModelType 2 weapon colorways must emit concrete direct-color TGA resources",
            ));
        }
        apply_item_weapon_colorway_v1(&mut texture_image, color)?;
    }
    let mut icon_projection_bounds = None;
    let mut icon_opaque_pixel_count = None;
    let icon_payload = if options.geometry_derived_icon {
        match options.icon_size {
            Some([width, height]) => {
                let (payload, bounds, opaque_pixels) = write_item_icon_layer_v2(
                    &texture_image,
                    &model,
                    width,
                    height,
                    options.icon_projection_bounds,
                )?;
                icon_projection_bounds = Some(bounds);
                icon_opaque_pixel_count = Some(opaque_pixels);
                Some(payload)
            }
            None => {
                let geometry = item_icon_geometry_v1(&model)?;
                icon_projection_bounds = Some(item_icon_projection_bounds_v1(&geometry)?);
                None
            }
        }
    } else {
        options
            .icon_size
            .map(|[width, height]| write_item_icon_layer_v1(&texture_image, width, height))
            .transpose()?
    };
    let icon_sha256 = icon_payload.as_deref().map(item_payload_sha256_v1);
    let (texture_payload, texture_sha256, texture_format) =
        if let Some(layer) = options.texture_encoding.plt_layer() {
            let texture = write_item_plt_v1(
                &texture_image,
                &PltWriterOptionsV1 {
                    layer,
                    ..PltWriterOptionsV1::default()
                },
            )
            .map_err(|source| error(&source.code, source.path, source.message))?;
            (
                texture.payload,
                texture.report.output_sha256,
                "PLT_V1".to_owned(),
            )
        } else {
            let texture =
                write_tga_v1(&texture_image, &TgaWriterOptionsV1::default()).map_err(|source| {
                    error(
                        &format!("ITEM-{}", source.code),
                        source.path,
                        source.message,
                    )
                })?;
            (
                texture.payload,
                texture.report.output_sha256,
                "TGA_V1".to_owned(),
            )
        };
    let bindings = model
        .material_source_bindings
        .iter()
        .map(|binding| MdlMaterialTextureBindingV1 {
            material_slot: binding.slot,
            resref: texture_resref.clone(),
        })
        .collect();
    segment_model_for_binary_mdl_v1(&mut model)
        .map_err(|source| error("ITEM-PART-SEGMENTATION-FAILED", source.path, source.message))?;
    let stream_count = model.segments.len();
    let mdl = write_binary_mdl_with_animations_exact_face_planes_v1(
        &model,
        &MdlAnimationSetV1::empty(),
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: if options.aurora_composer_v2 {
                MdlFormatProfileV1::ItemPartStaticRigidAuroraComposerV2
            } else {
                MdlFormatProfileV1::ItemPartStaticRigidNativeV1
            },
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: model_resref.clone(),
            diffuse_texture_resref_by_material_slot: bindings,
        },
    )
    .map_err(|source| error("ITEM-PART-MDL-WRITE-FAILED", source.path, source.message))?;
    let readback = inspect_binary_mdl(&mdl.payload).map_err(|source| {
        error(
            "ITEM-PART-MDL-READBACK-FAILED",
            source.context,
            format!("{} at byte {}", source.code, source.offset),
        )
    })?;
    if readback.model.name != model_resref {
        return Err(error(
            "ITEM-PART-MDL-SEMANTIC-DIFF",
            "readback.model.name",
            "binary MDL readback model identity differs from the requested item part resref",
        ));
    }

    Ok(ItemPartArtifactV1 {
        mdl_payload: mdl.payload.clone(),
        texture_payload,
        icon_payload,
        report: ItemPartBuildReportV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            profile: if options.aurora_composer_v2 {
                "ITEM_PART_AURORA_COMPOSER_V2".to_owned()
            } else {
                "ITEM_PART_STATIC_RIGID".to_owned()
            },
            model_resref,
            texture_resref,
            texture_format,
            weapon_color: options.weapon_color,
            source_node,
            source_sha256: item_payload_sha256_v1(source_glb),
            triangle_count,
            degenerate_triangle_count_removed,
            stream_count,
            transform: options.transform,
            target_space_scale_xyz: options.target_space_scale_xyz,
            mdl_sha256: item_payload_sha256_v1(&mdl.payload),
            texture_sha256,
            icon_sha256,
            icon_projection_bounds,
            icon_opaque_pixel_count,
            semantic_readback_status: "PASS".to_owned(),
        },
        readback,
    })
}

fn select_item_source_node_v1(
    ingest: &mut crate::glb::GlbIngestResult,
    requested: Option<&str>,
) -> Result<Option<String>, ItemErrorV1> {
    let Some(requested) = requested.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let matches = ingest
        .ir
        .nodes
        .iter()
        .filter(|node| node.name.as_deref() == Some(requested))
        .collect::<Vec<_>>();
    let selected = match matches.as_slice() {
        [node] => *node,
        [] => {
            return Err(error(
                "ITEM-SOURCE-NODE-NOT-FOUND",
                "options.sourceNode",
                format!("GLB has no node named {requested:?}"),
            ));
        }
        _ => {
            return Err(error(
                "ITEM-SOURCE-NODE-AMBIGUOUS",
                "options.sourceNode",
                format!("GLB has more than one node named {requested:?}"),
            ));
        }
    };
    let scene_id = ingest.ir.default_scene_id.ok_or_else(|| {
        error(
            "ITEM-SOURCE-NODE-SCENE-MISSING",
            "source.ir.defaultSceneId",
            "source-node selection requires a default GLB scene",
        )
    })?;
    let scene = ingest
        .ir
        .scenes
        .get_mut(scene_id as usize)
        .filter(|scene| scene.id == scene_id)
        .ok_or_else(|| {
            error(
                "ITEM-SOURCE-NODE-SCENE-MISSING",
                "source.ir.defaultSceneId",
                "default GLB scene does not exist",
            )
        })?;
    if !scene.root_node_ids.contains(&selected.id) {
        return Err(error(
            "ITEM-SOURCE-NODE-NOT-ROOT",
            "options.sourceNode",
            "sourceNode must be a top-level node of the default scene so its transform remains lossless",
        ));
    }
    scene.root_node_ids = vec![selected.id];
    let selected_root_id = selected.id;
    let mut pending = vec![selected_root_id];
    let mut selected_node_ids = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !selected_node_ids.insert(node_id) {
            continue;
        }
        let node = ingest
            .ir
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| {
                error(
                    "ITEM-SOURCE-NODE-HIERARCHY-INVALID",
                    "source.ir.nodes",
                    format!("selected source hierarchy references missing node {node_id}"),
                )
            })?;
        pending.extend(node.child_ids.iter().copied());
    }
    let selected_mesh_ids = ingest
        .ir
        .nodes
        .iter()
        .filter(|node| selected_node_ids.contains(&node.id))
        .filter_map(|node| node.mesh_id)
        .collect::<BTreeSet<_>>();
    let node_id_map = ingest
        .ir
        .nodes
        .iter()
        .filter(|node| selected_node_ids.contains(&node.id))
        .enumerate()
        .map(|(index, node)| (node.id, index as u32))
        .collect::<BTreeMap<_, _>>();
    let mesh_id_map = ingest
        .ir
        .meshes
        .iter()
        .filter(|mesh| selected_mesh_ids.contains(&mesh.id))
        .enumerate()
        .map(|(index, mesh)| (mesh.id, index as u32))
        .collect::<BTreeMap<_, _>>();
    let selected_primitive_ids = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| selected_mesh_ids.contains(&primitive.source_mesh_id))
        .map(|primitive| primitive.id)
        .collect::<BTreeSet<_>>();
    let primitive_id_map = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| selected_primitive_ids.contains(&primitive.id))
        .enumerate()
        .map(|(index, primitive)| (primitive.id, index as u32))
        .collect::<BTreeMap<_, _>>();
    ingest.ir.nodes.retain_mut(|node| {
        if !selected_node_ids.contains(&node.id) {
            return false;
        }
        node.id = node_id_map[&node.id];
        node.child_ids = node
            .child_ids
            .iter()
            .filter_map(|id| node_id_map.get(id).copied())
            .collect();
        node.parent_ids = node
            .parent_ids
            .iter()
            .filter_map(|id| node_id_map.get(id).copied())
            .collect();
        node.mesh_id = node.mesh_id.and_then(|id| mesh_id_map.get(&id).copied());
        true
    });
    ingest.ir.meshes.retain_mut(|mesh| {
        if !selected_mesh_ids.contains(&mesh.id) {
            return false;
        }
        mesh.id = mesh_id_map[&mesh.id];
        mesh.primitive_ids = mesh
            .primitive_ids
            .iter()
            .filter_map(|id| primitive_id_map.get(id).copied())
            .collect();
        true
    });
    ingest.ir.primitives.retain_mut(|primitive| {
        if !selected_primitive_ids.contains(&primitive.id) {
            return false;
        }
        primitive.id = primitive_id_map[&primitive.id];
        primitive.source_mesh_id = mesh_id_map[&primitive.source_mesh_id];
        true
    });
    scene.root_node_ids = vec![node_id_map[&selected_root_id]];
    ingest.report.inventory.node_count = ingest.ir.nodes.len();
    ingest.report.inventory.mesh_count = ingest.ir.meshes.len();
    ingest.report.inventory.primitive_count = ingest.ir.primitives.len();
    ingest.report.statistics.vertex_count = ingest
        .ir
        .primitives
        .iter()
        .map(|primitive| primitive.positions.len())
        .sum();
    ingest.report.statistics.index_count = ingest
        .ir
        .primitives
        .iter()
        .map(|primitive| primitive.indices.len())
        .sum();
    ingest.report.statistics.triangle_count = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.topology == "TRIANGLES")
        .map(|primitive| primitive.indices.len() / 3)
        .sum();
    ingest.report.statistics.primitives_missing_normals = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.normals.is_empty())
        .count();
    ingest.report.statistics.primitives_missing_uv0 = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.uv0.is_empty())
        .count();
    ingest.report.statistics.non_triangle_primitives = ingest
        .ir
        .primitives
        .iter()
        .filter(|primitive| primitive.topology != "TRIANGLES")
        .count();
    let mut positions = ingest
        .ir
        .primitives
        .iter()
        .flat_map(|primitive| primitive.positions.iter().copied());
    if let Some(first) = positions.next() {
        let (min, max) = positions.fold((first, first), |(mut min, mut max), position| {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
            (min, max)
        });
        ingest.report.statistics.bounds_min = Some(min);
        ingest.report.statistics.bounds_max = Some(max);
    } else {
        ingest.report.statistics.bounds_min = None;
        ingest.report.statistics.bounds_max = None;
    }
    Ok(Some(requested.to_owned()))
}

pub fn validate_item_triangle_budget_v1(
    part_triangle_counts: &[usize],
) -> Result<ItemTriangleBudgetReportV1, ItemErrorV1> {
    let triangle_count =
        part_triangle_counts
            .iter()
            .enumerate()
            .try_fold(0usize, |total, (index, count)| {
                total.checked_add(*count).ok_or_else(|| {
                    error(
                        "ITEM-TRIANGLE-COUNT-OVERFLOW",
                        format!("partTriangleCounts[{index}]"),
                        "combined item triangle count overflowed this platform",
                    )
                })
            })?;
    if triangle_count > AURORA_MODEL_TRIANGLE_BUDGET_V1 {
        return Err(ItemErrorV1 {
            schema_version: ITEM_SCHEMA_VERSION_V1,
            code: "ITEM-TRIANGLE-BUDGET-EXCEEDED".to_owned(),
            path: "partTriangleCounts".to_owned(),
            message: format!(
                "item parts contain {triangle_count} triangles in total; shared product budget is {AURORA_MODEL_TRIANGLE_BUDGET_V1}"
            ),
        });
    }
    Ok(ItemTriangleBudgetReportV1 {
        schema_version: ITEM_SCHEMA_VERSION_V1,
        triangle_count,
        triangle_budget: AURORA_MODEL_TRIANGLE_BUDGET_V1,
        warning_above: AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
        warning: triangle_count > AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
    })
}

fn validate_item_part_transform_v1(transform: ItemPartTransformV1) -> Result<(), ItemErrorV1> {
    if transform
        .translation
        .into_iter()
        .chain(transform.rotation_xyzw)
        .chain(transform.pivot)
        .chain([transform.uniform_scale])
        .any(|value| !value.is_finite())
        || transform.uniform_scale <= 0.0
    {
        return Err(error(
            "ITEM-PART-TRANSFORM-INVALID",
            "transform",
            "translation, quaternion, pivot, and positive uniform scale must be finite",
        ));
    }
    let norm_squared = transform
        .rotation_xyzw
        .into_iter()
        .map(|value| value * value)
        .sum::<f32>();
    if norm_squared <= 1.0e-12 {
        return Err(error(
            "ITEM-PART-TRANSFORM-INVALID",
            "transform.rotationXyzw",
            "rotation quaternion must have a non-zero norm",
        ));
    }
    Ok(())
}

fn bake_uniform_scale_v1(
    model: &mut crate::model_ir::AuroraModelIrV1,
    scale: f32,
    pivot: [f32; 3],
) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    for segment in &mut model.segments {
        for position in &mut segment.positions {
            for axis in 0..3 {
                position[axis] = pivot[axis] + (position[axis] - pivot[axis]) * scale;
            }
        }
    }
    for node in &mut model.nodes {
        for axis in 0..3 {
            let matrix_index = 12 + axis;
            node.bind_local_matrix[matrix_index] *= scale;
        }
    }
}

fn bake_uniform_scale_v2(
    model: &mut crate::model_ir::AuroraModelIrV1,
    scale: f32,
    pivot: [f32; 3],
) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    // Uniform scale commutes with every rigid node rotation, so preserving the
    // hierarchy losslessly requires scaling mesh-local positions around their
    // own origins and scaling node-to-parent translations. Only root
    // translations are moved around the caller's model-space pivot.
    for segment in &mut model.segments {
        for position in &mut segment.positions {
            for value in position {
                *value *= scale;
            }
        }
    }
    for node in &mut model.nodes {
        for (axis, pivot_axis) in pivot.iter().copied().enumerate() {
            let matrix_index = 12 + axis;
            node.bind_local_matrix[matrix_index] = if node.parent_id.is_none() {
                pivot_axis + (node.bind_local_matrix[matrix_index] - pivot_axis) * scale
            } else {
                node.bind_local_matrix[matrix_index] * scale
            };
        }
    }
}

fn item_part_rigid_matrix_v1(transform: ItemPartTransformV1) -> Result<[f32; 16], ItemErrorV1> {
    let norm = transform
        .rotation_xyzw
        .into_iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= 1.0e-6 {
        return Err(error(
            "ITEM-PART-TRANSFORM-INVALID",
            "transform.rotationXyzw",
            "rotation quaternion cannot be normalized",
        ));
    }
    let [x, y, z, w] = transform.rotation_xyzw.map(|value| value / norm);
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;
    let rotation = [
        1.0 - 2.0 * (yy + zz),
        2.0 * (xy + wz),
        2.0 * (xz - wy),
        0.0,
        2.0 * (xy - wz),
        1.0 - 2.0 * (xx + zz),
        2.0 * (yz + wx),
        0.0,
        2.0 * (xz + wy),
        2.0 * (yz - wx),
        1.0 - 2.0 * (xx + yy),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    let rotated_pivot = transform_point_v1(rotation, transform.pivot);
    let mut output = rotation;
    for axis in 0..3 {
        output[12 + axis] =
            transform.translation[axis] + transform.pivot[axis] - rotated_pivot[axis];
    }
    Ok(output)
}

fn transform_point_v1(matrix: [f32; 16], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn multiply_matrix_v1(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::bake_uniform_scale_v2;
    use crate::model_ir::{
        AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1,
    };

    #[test]
    fn v2_scale_preserves_rotated_hierarchy_and_applies_pivot_only_at_roots() {
        let root_matrix = [
            0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
        ];
        let child_matrix = [
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 2.0, 3.0, 1.0,
        ];
        let mut model = AuroraModelIrV1 {
            schema_version: 1,
            profile_id: "test".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "test".to_owned(),
            engine_facing_proof: "test".to_owned(),
            uv_runtime_proof: "test".to_owned(),
            nodes: vec![
                AuroraModelNodeV1 {
                    id: 0,
                    name: "root".to_owned(),
                    parent_id: None,
                    bind_local_matrix: root_matrix,
                },
                AuroraModelNodeV1 {
                    id: 1,
                    name: "child".to_owned(),
                    parent_id: Some(0),
                    bind_local_matrix: child_matrix,
                },
            ],
            material_source_bindings: Vec::new(),
            segments: vec![AuroraModelSegmentV1 {
                segment_id: 0,
                material_slot: 0,
                deformation: AuroraSegmentDeformationV1::Rigid,
                parent_node_id: 1,
                cast_shadow: true,
                positions: vec![[4.0, 5.0, 6.0]],
                normals: vec![[0.0, 0.0, 1.0]],
                tangents: None,
                uv0: vec![[0.0, 0.0]],
                indices: Vec::new(),
                face_surface_ids: Vec::new(),
                weights: Vec::new(),
            }],
        };

        bake_uniform_scale_v2(&mut model, 2.0, [10.0, 20.0, 30.0]);

        assert_eq!(&model.nodes[0].bind_local_matrix[..12], &root_matrix[..12]);
        assert_eq!(&model.nodes[1].bind_local_matrix[..12], &child_matrix[..12]);
        assert_eq!(
            &model.nodes[0].bind_local_matrix[12..15],
            &[-6.0, -14.0, -22.0]
        );
        assert_eq!(&model.nodes[1].bind_local_matrix[12..15], &[2.0, 4.0, 6.0]);
        assert_eq!(model.segments[0].positions, [[8.0, 10.0, 12.0]]);
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}
