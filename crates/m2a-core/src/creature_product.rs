//! Versioned product contracts shared by every Creature frontend.
//!
//! These contracts deliberately separate render-model authoring from runtime
//! Appearance/UTC authoring.  A gameplay-only edit therefore has its own
//! canonical identity and never implies rebuilding the binary MDL.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::model_limits::AURORA_MODEL_TRIANGLE_BUDGET_V1;

pub const CREATURE_PRODUCT_CONTRACT_SCHEMA_VERSION_V1: u32 = 1;
const SHARED_TRIANGLE_LIMIT_U32: u32 = AURORA_MODEL_TRIANGLE_BUDGET_V1 as u32;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureProductContractErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for CreatureProductContractErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for CreatureProductContractErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> CreatureProductContractErrorV1 {
    CreatureProductContractErrorV1 {
        schema_version: CREATURE_PRODUCT_CONTRACT_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

pub fn creature_contract_digest_v1<T: Serialize>(
    value: &T,
) -> Result<String, CreatureProductContractErrorV1> {
    let bytes = serde_json::to_vec(value).map_err(|source| {
        error(
            "CREATURE-CONTRACT-SERIALIZATION",
            "contract",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

// -------------------------------------------------------------------------
// Equipment and item-family grip contracts

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureItemGripFamilyV1 {
    #[default]
    Sword,
    AxeMace,
    SpearPolearm,
    BowCrossbow,
    Shield,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureEquipmentPlacementV2 {
    RightHand,
    LeftHand,
    BothHands,
}

impl CreatureEquipmentPlacementV2 {
    /// Slot identifier used by the embedded item struct. Two-handed weapons
    /// are stored in the right-hand slot while reserving both hands.
    pub const fn primary_native_struct_id(self) -> u32 {
        match self {
            Self::RightHand | Self::BothHands => 16,
            Self::LeftHand => 32,
        }
    }

    fn occupied_native_slots(self) -> &'static [u32] {
        match self {
            Self::RightHand => &[16],
            Self::LeftHand => &[32],
            Self::BothHands => &[16, 32],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureItemResourceScopeV1 {
    NwnBaseGame,
    ModuleOwnedRecipe,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureEquipmentItemV2 {
    pub resref: String,
    pub display_name: String,
    pub resource_scope: CreatureItemResourceScopeV1,
    pub base_item: i32,
    pub model_parts: [u8; 3],
    pub placement: CreatureEquipmentPlacementV2,
    pub grip_family: CreatureItemGripFamilyV1,
    #[serde(default)]
    pub required_proficiency_feats: Vec<u16>,
    #[serde(default = "default_localized_name_strref")]
    pub localized_name_strref: u32,
    #[serde(default = "default_item_cost")]
    pub cost: u32,
}

const fn default_localized_name_strref() -> u32 {
    u32::MAX
}

const fn default_item_cost() -> u32 {
    1
}

impl CreatureEquipmentItemV2 {
    pub fn owned(
        resref: impl Into<String>,
        display_name: impl Into<String>,
        base_item: i32,
        model_parts: [u8; 3],
        placement: CreatureEquipmentPlacementV2,
        grip_family: CreatureItemGripFamilyV1,
    ) -> Self {
        Self {
            resref: resref.into(),
            display_name: display_name.into(),
            resource_scope: CreatureItemResourceScopeV1::ModuleOwnedRecipe,
            base_item,
            model_parts,
            placement,
            grip_family,
            required_proficiency_feats: Vec::new(),
            localized_name_strref: u32::MAX,
            cost: 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureEquipmentLoadoutV2 {
    pub schema_version: u32,
    #[serde(default)]
    pub items: Vec<CreatureEquipmentItemV2>,
}

impl Default for CreatureEquipmentLoadoutV2 {
    fn default() -> Self {
        Self {
            schema_version: 2,
            items: Vec::new(),
        }
    }
}

impl CreatureEquipmentLoadoutV2 {
    /// The exact complete item recipe proven by the owner-corrected V10
    /// module. It is embedded in GIT/UTC and requires no module-local UTI.
    pub fn v10_bastard_sword_right_hand() -> Self {
        Self {
            schema_version: 2,
            items: vec![CreatureEquipmentItemV2 {
                resref: "nw_wswbs001".to_owned(),
                display_name: "Bastard Sword".to_owned(),
                resource_scope: CreatureItemResourceScopeV1::NwnBaseGame,
                base_item: 3,
                model_parts: [41, 11, 11],
                placement: CreatureEquipmentPlacementV2::RightHand,
                grip_family: CreatureItemGripFamilyV1::Sword,
                required_proficiency_feats: vec![44],
                localized_name_strref: 168,
                cost: 70,
            }],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureEquipmentLoadoutReportV2 {
    pub schema_version: u32,
    pub embedded_item_count: usize,
    pub native_slots: Vec<u32>,
    pub required_proficiency_feats: Vec<u16>,
    pub no_equipped_res_shortcuts: bool,
    pub canonical_sha256: String,
}

pub fn validate_creature_equipment_loadout_v2(
    loadout: &CreatureEquipmentLoadoutV2,
) -> Result<CreatureEquipmentLoadoutReportV2, CreatureProductContractErrorV1> {
    if loadout.schema_version != 2 {
        return Err(error(
            "CREATURE-EQUIPMENT-SCHEMA",
            "equipmentLoadout.schemaVersion",
            "Creature equipment loadout requires schemaVersion 2",
        ));
    }
    if loadout.items.len() > 2 {
        return Err(error(
            "CREATURE-EQUIPMENT-ITEM-LIMIT",
            "equipmentLoadout.items",
            "at most two native hand items are supported",
        ));
    }
    let mut slots = BTreeSet::new();
    let mut feats = BTreeSet::new();
    let mut resrefs = BTreeSet::new();
    let mut has_both_hands = false;
    for (index, item) in loadout.items.iter().enumerate() {
        let path = format!("equipmentLoadout.items[{index}]");
        if !valid_resref(&item.resref) {
            return Err(error(
                "CREATURE-EQUIPMENT-RESREF",
                format!("{path}.resref"),
                "resref must contain 1..16 lowercase ASCII letters, digits, or underscores",
            ));
        }
        if !resrefs.insert(item.resref.to_ascii_lowercase()) {
            return Err(error(
                "CREATURE-EQUIPMENT-DUPLICATE-ITEM",
                format!("{path}.resref"),
                "one loadout cannot contain the same embedded item identity twice",
            ));
        }
        if item.display_name.trim().is_empty()
            || item.display_name.len() > 128
            || item.display_name.chars().any(char::is_control)
        {
            return Err(error(
                "CREATURE-EQUIPMENT-DISPLAY-NAME",
                format!("{path}.displayName"),
                "display name must contain 1..128 bytes without control characters",
            ));
        }
        if item.base_item < 0 || item.model_parts.contains(&0) {
            return Err(error(
                "CREATURE-EQUIPMENT-ITEM-RECIPE",
                path,
                "base item must be non-negative and every model part must be non-zero",
            ));
        }
        has_both_hands |= item.placement == CreatureEquipmentPlacementV2::BothHands;
        for slot in item.placement.occupied_native_slots() {
            if !slots.insert(*slot) {
                let code = if has_both_hands {
                    "CREATURE-EQUIPMENT-TWO-HANDED-COLLISION"
                } else {
                    "CREATURE-EQUIPMENT-SLOT-COLLISION"
                };
                return Err(error(
                    code,
                    format!("{path}.placement"),
                    "two embedded items reserve the same native hand slot",
                ));
            }
        }
        let mut previous = None;
        for feat in &item.required_proficiency_feats {
            if previous.is_some_and(|value| value >= *feat) {
                return Err(error(
                    "CREATURE-EQUIPMENT-FEAT-ORDER",
                    format!("{path}.requiredProficiencyFeats"),
                    "required feats must be strictly increasing and unique",
                ));
            }
            previous = Some(*feat);
            feats.insert(*feat);
        }
    }
    Ok(CreatureEquipmentLoadoutReportV2 {
        schema_version: 2,
        embedded_item_count: loadout.items.len(),
        native_slots: slots.into_iter().collect(),
        required_proficiency_feats: feats.into_iter().collect(),
        no_equipped_res_shortcuts: true,
        canonical_sha256: creature_contract_digest_v1(loadout)?,
    })
}

fn valid_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureItemGripProfileV1 {
    pub schema_version: u32,
    pub family: CreatureItemGripFamilyV1,
    /// Proper row-major rotation in the local NWN item basis.
    pub primary_item_basis_rotation: [[f32; 3]; 3],
    pub secondary_hand_target: bool,
    pub calibration_status: String,
}

pub fn creature_item_grip_profile_v1(
    family: CreatureItemGripFamilyV1,
) -> CreatureItemGripProfileV1 {
    let (rotation, secondary_hand_target) = match family {
        CreatureItemGripFamilyV1::Sword => {
            ([[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]], false)
        }
        CreatureItemGripFamilyV1::AxeMace => {
            ([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], false)
        }
        CreatureItemGripFamilyV1::SpearPolearm => {
            ([[-1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, -1.0, 0.0]], true)
        }
        CreatureItemGripFamilyV1::BowCrossbow => {
            ([[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]], true)
        }
        CreatureItemGripFamilyV1::Shield => {
            ([[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]], false)
        }
    };
    CreatureItemGripProfileV1 {
        schema_version: 1,
        family,
        primary_item_basis_rotation: rotation,
        secondary_hand_target,
        calibration_status: "OFFLINE_INITIAL_CALIBRATION_OWNER_PROOF_REQUIRED".to_owned(),
    }
}

// -------------------------------------------------------------------------
// Appearance runtime envelope and UTC authoring

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureModelBoundsV1 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureTargetHeightV1 {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureRuntimeEnvelopeV1 {
    pub schema_version: u32,
    pub height: f32,
    pub hit_distance: f32,
    pub personal_space: f32,
    pub creature_personal_space: f32,
    pub preferred_attack_distance: f32,
    pub target_height: CreatureTargetHeightV1,
    pub perception_distance: f32,
    pub walk_distance: f32,
    pub run_distance: f32,
    pub size_category: u8,
    pub footstep_type: u8,
}

impl CreatureRuntimeEnvelopeV1 {
    pub fn medium_humanoid() -> Self {
        Self {
            schema_version: 1,
            height: 2.0,
            hit_distance: 0.3,
            personal_space: 0.3,
            creature_personal_space: 0.5,
            preferred_attack_distance: 1.7,
            target_height: CreatureTargetHeightV1::High,
            perception_distance: 9.0,
            walk_distance: 1.6,
            run_distance: 3.2,
            size_category: 3,
            footstep_type: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureRuntimeEnvelopePolicyV1 {
    #[default]
    MediumHumanoid,
    BoundsDerived,
    Explicit(CreatureRuntimeEnvelopeV1),
}

pub fn derive_creature_runtime_envelope_v1(
    policy: &CreatureRuntimeEnvelopePolicyV1,
    bounds: CreatureModelBoundsV1,
) -> Result<CreatureRuntimeEnvelopeV1, CreatureProductContractErrorV1> {
    if bounds
        .min
        .iter()
        .chain(&bounds.max)
        .any(|value| !value.is_finite())
        || (0..3).any(|axis| bounds.min[axis] > bounds.max[axis])
    {
        return Err(error(
            "CREATURE-RUNTIME-BOUNDS",
            "modelBounds",
            "bounds must be finite and ordered on every axis",
        ));
    }
    let envelope = match policy {
        CreatureRuntimeEnvelopePolicyV1::MediumHumanoid => {
            CreatureRuntimeEnvelopeV1::medium_humanoid()
        }
        CreatureRuntimeEnvelopePolicyV1::Explicit(value) => value.clone(),
        CreatureRuntimeEnvelopePolicyV1::BoundsDerived => {
            let height = bounds.max[2] - bounds.min[2];
            let horizontal_radius = [
                bounds.min[0].abs(),
                bounds.max[0].abs(),
                bounds.min[1].abs(),
                bounds.max[1].abs(),
            ]
            .into_iter()
            .fold(0.0_f32, f32::max)
            .max(0.2);
            let personal_space = (horizontal_radius + 0.15).max(0.3);
            CreatureRuntimeEnvelopeV1 {
                schema_version: 1,
                height,
                hit_distance: horizontal_radius,
                personal_space,
                creature_personal_space: personal_space + 0.2,
                preferred_attack_distance: (horizontal_radius + 1.0).max(1.2),
                target_height: if height < 1.0 {
                    CreatureTargetHeightV1::Low
                } else if height < 1.8 {
                    CreatureTargetHeightV1::Medium
                } else {
                    CreatureTargetHeightV1::High
                },
                perception_distance: 9.0,
                walk_distance: (height * 0.8).max(0.5),
                run_distance: (height * 1.6).max(1.0),
                size_category: if height < 1.0 {
                    2
                } else if height > 3.0 {
                    4
                } else {
                    3
                },
                footstep_type: 0,
            }
        }
    };
    validate_runtime_envelope(&envelope)?;
    Ok(envelope)
}

fn validate_runtime_envelope(
    envelope: &CreatureRuntimeEnvelopeV1,
) -> Result<(), CreatureProductContractErrorV1> {
    if envelope.schema_version != 1 {
        return Err(error(
            "CREATURE-RUNTIME-SCHEMA",
            "runtimeEnvelope.schemaVersion",
            "runtime envelope requires schemaVersion 1",
        ));
    }
    let distances = [
        envelope.height,
        envelope.hit_distance,
        envelope.personal_space,
        envelope.creature_personal_space,
        envelope.preferred_attack_distance,
        envelope.perception_distance,
        envelope.walk_distance,
        envelope.run_distance,
    ];
    if distances
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
        || envelope.personal_space < envelope.hit_distance
        || envelope.creature_personal_space < envelope.personal_space
        || envelope.preferred_attack_distance <= envelope.hit_distance
    {
        return Err(error(
            "CREATURE-RUNTIME-ENVELOPE",
            "runtimeEnvelope",
            "runtime distances must be finite, positive and monotonically safe",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureClassAuthoringV1 {
    pub class_id: i32,
    pub level: i16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureScriptHooksV1 {
    pub heartbeat: String,
    pub notice: String,
    pub spell_at: String,
    pub attacked: String,
    pub damaged: String,
    pub disturbed: String,
    pub end_round: String,
    pub dialogue: String,
    pub spawn: String,
    pub rested: String,
    pub death: String,
    pub user_defined: String,
    pub blocked: String,
}

impl CreatureScriptHooksV1 {
    pub fn active_monster_default() -> Self {
        Self {
            heartbeat: "nw_c2_default1".to_owned(),
            notice: "nw_c2_default2".to_owned(),
            spell_at: "nw_c2_defaultb".to_owned(),
            attacked: "nw_c2_default5".to_owned(),
            damaged: "nw_c2_default6".to_owned(),
            disturbed: "nw_c2_default8".to_owned(),
            end_round: "nw_c2_default3".to_owned(),
            dialogue: "nw_c2_default4".to_owned(),
            spawn: "nw_c2_default9".to_owned(),
            rested: "nw_c2_defaulta".to_owned(),
            death: "nw_c2_default7".to_owned(),
            user_defined: "nw_c2_defaultd".to_owned(),
            blocked: "nw_c2_defaulte".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureBlueprintAuthoringV1 {
    pub schema_version: u32,
    pub first_name: String,
    pub last_name: String,
    pub description: String,
    pub tag: String,
    pub conversation: String,
    pub faction_id: u16,
    pub portrait_id: u16,
    pub sound_set_file: u16,
    pub race: u8,
    pub gender: u8,
    pub hit_points: i16,
    pub current_hit_points: i16,
    pub max_hit_points: i16,
    pub challenge_rating: f32,
    pub abilities: [u8; 6],
    pub natural_ac: u8,
    pub perception_range: u8,
    pub class: CreatureClassAuthoringV1,
    #[serde(default)]
    pub feats: Vec<u16>,
    pub scripts: CreatureScriptHooksV1,
    pub plot: bool,
    pub immortal: bool,
    pub interruptable: bool,
    pub lootable: bool,
}

impl CreatureBlueprintAuthoringV1 {
    pub fn active_monster_default() -> Self {
        Self {
            schema_version: 1,
            first_name: "Meshy procedural creature".to_owned(),
            last_name: String::new(),
            description: String::new(),
            tag: "m2a_creature".to_owned(),
            conversation: String::new(),
            faction_id: 1,
            portrait_id: 236,
            sound_set_file: 53,
            race: 7,
            gender: 2,
            hit_points: 22,
            current_hit_points: 22,
            max_hit_points: 37,
            challenge_rating: 5.0,
            abilities: [19, 15, 17, 6, 12, 6],
            natural_ac: 6,
            perception_range: 11,
            class: CreatureClassAuthoringV1 {
                class_id: 11,
                level: 5,
            },
            feats: Vec::new(),
            scripts: CreatureScriptHooksV1::active_monster_default(),
            plot: false,
            immortal: false,
            interruptable: true,
            lootable: false,
        }
    }
}

pub fn validate_creature_blueprint_authoring_v1(
    value: &CreatureBlueprintAuthoringV1,
) -> Result<String, CreatureProductContractErrorV1> {
    if value.schema_version != 1 {
        return Err(error(
            "CREATURE-BLUEPRINT-SCHEMA",
            "blueprint.schemaVersion",
            "Creature blueprint authoring requires schemaVersion 1",
        ));
    }
    if value.first_name.trim().is_empty()
        || value.first_name.len() > 128
        || value.first_name.chars().any(char::is_control)
        || value.last_name.len() > 128
        || value.description.chars().any(char::is_control)
    {
        return Err(error(
            "CREATURE-BLUEPRINT-TEXT",
            "blueprint",
            "blueprint names must be bounded and free of control characters",
        ));
    }
    if !valid_resref(&value.tag.to_ascii_lowercase())
        || (!value.conversation.is_empty() && !valid_resref(&value.conversation))
    {
        return Err(error(
            "CREATURE-BLUEPRINT-RESREF",
            "blueprint.tag",
            "tag and non-empty conversation must satisfy the 16-byte resref envelope",
        ));
    }
    if value.hit_points <= 0
        || value.current_hit_points <= 0
        || value.max_hit_points <= 0
        || value.hit_points > value.max_hit_points
        || value.current_hit_points > value.max_hit_points
        || !value.challenge_rating.is_finite()
        || value.challenge_rating < 0.0
        || value.class.class_id < 0
        || value.class.level <= 0
        || value
            .abilities
            .iter()
            .any(|ability| !(1..=255).contains(ability))
    {
        return Err(error(
            "CREATURE-BLUEPRINT-GAMEPLAY",
            "blueprint",
            "gameplay values must be finite, positive and internally consistent",
        ));
    }
    let mut previous = None;
    for feat in &value.feats {
        if previous.is_some_and(|entry| entry >= *feat) {
            return Err(error(
                "CREATURE-BLUEPRINT-FEAT-ORDER",
                "blueprint.feats",
                "feats must be strictly increasing and unique",
            ));
        }
        previous = Some(*feat);
    }
    for (path, script) in [
        ("heartbeat", &value.scripts.heartbeat),
        ("notice", &value.scripts.notice),
        ("spellAt", &value.scripts.spell_at),
        ("attacked", &value.scripts.attacked),
        ("damaged", &value.scripts.damaged),
        ("disturbed", &value.scripts.disturbed),
        ("endRound", &value.scripts.end_round),
        ("dialogue", &value.scripts.dialogue),
        ("spawn", &value.scripts.spawn),
        ("rested", &value.scripts.rested),
        ("death", &value.scripts.death),
        ("userDefined", &value.scripts.user_defined),
        ("blocked", &value.scripts.blocked),
    ] {
        if !script.is_empty() && !valid_resref(script) {
            return Err(error(
                "CREATURE-BLUEPRINT-SCRIPT-RESREF",
                format!("blueprint.scripts.{path}"),
                "script resref must be empty or contain 1..16 lowercase ASCII characters",
            ));
        }
    }
    creature_contract_digest_v1(value)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureDemoAuthoringV1 {
    pub schema_version: u32,
    pub blueprint: CreatureBlueprintAuthoringV1,
    pub equipment_loadout: CreatureEquipmentLoadoutV2,
}

impl CreatureDemoAuthoringV1 {
    pub fn active_monster_with_v10_bastard_sword() -> Self {
        Self {
            schema_version: 1,
            blueprint: CreatureBlueprintAuthoringV1::active_monster_default(),
            equipment_loadout: CreatureEquipmentLoadoutV2::v10_bastard_sword_right_hand(),
        }
    }
}

pub fn validate_creature_demo_authoring_v1(
    value: &CreatureDemoAuthoringV1,
) -> Result<String, CreatureProductContractErrorV1> {
    if value.schema_version != 1 {
        return Err(error(
            "CREATURE-DEMO-AUTHORING-SCHEMA",
            "demoAuthoring.schemaVersion",
            "Creature demo authoring requires schemaVersion 1",
        ));
    }
    validate_creature_blueprint_authoring_v1(&value.blueprint)?;
    validate_creature_equipment_loadout_v2(&value.equipment_loadout)?;
    creature_contract_digest_v1(value)
}

// -------------------------------------------------------------------------
// Motion packs and skeleton profiles

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureMotionSkeletonProfileV1 {
    Humanoid,
    Quadruped,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureMotionClipV1 {
    pub source_clip_name: String,
    pub output_clip_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_family: Option<CreatureItemGripFamilyV1>,
}

impl CreatureMotionClipV1 {
    pub fn source(
        source_clip_name: impl Into<String>,
        output_clip_name: impl Into<String>,
        weapon_family: Option<CreatureItemGripFamilyV1>,
    ) -> Self {
        Self {
            source_clip_name: source_clip_name.into(),
            output_clip_name: output_clip_name.into(),
            weapon_family,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureMotionPackV1 {
    pub schema_version: u32,
    pub source_identity: String,
    pub skeleton_profile: CreatureMotionSkeletonProfileV1,
    pub semantic_joints: Vec<String>,
    pub clips: Vec<CreatureMotionClipV1>,
}

impl CreatureMotionPackV1 {
    pub fn humanoid_weapon_pack(
        source_identity: impl Into<String>,
        clips: Vec<CreatureMotionClipV1>,
    ) -> Self {
        Self {
            schema_version: 1,
            source_identity: source_identity.into(),
            skeleton_profile: CreatureMotionSkeletonProfileV1::Humanoid,
            semantic_joints: [
                "Hips",
                "Spine",
                "Head",
                "RightHand",
                "LeftHand",
                "RightFoot",
                "LeftFoot",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            clips,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureMotionPackReportV1 {
    pub schema_version: u32,
    pub skeleton_profile: CreatureMotionSkeletonProfileV1,
    pub clip_count: usize,
    pub weapon_attack_families: Vec<CreatureItemGripFamilyV1>,
    pub four_point_contact_ready: bool,
    pub canonical_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureMotionSourceReportV1 {
    pub schema_version: u32,
    pub skeleton_profile: CreatureMotionSkeletonProfileV1,
    pub source_identity: String,
    pub resolved_joint_count: usize,
    pub resolved_clip_count: usize,
    pub selected_weapon_family: Option<CreatureItemGripFamilyV1>,
    pub selected_clip_count: usize,
    pub four_point_contact_ready: bool,
    pub motion_pack_sha256: String,
}

pub fn validate_creature_motion_pack_v1(
    pack: &CreatureMotionPackV1,
) -> Result<CreatureMotionPackReportV1, CreatureProductContractErrorV1> {
    if pack.schema_version != 1 {
        return Err(error(
            "CREATURE-MOTION-SCHEMA",
            "motionPack.schemaVersion",
            "motion pack requires schemaVersion 1",
        ));
    }
    if !pack.source_identity.starts_with("sha256:") || pack.source_identity.len() != 71 {
        return Err(error(
            "CREATURE-MOTION-SOURCE-IDENTITY",
            "motionPack.sourceIdentity",
            "motion source identity must be an exact sha256:<64 lowercase hex> value",
        ));
    }
    let semantic = pack
        .semantic_joints
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let required = match pack.skeleton_profile {
        CreatureMotionSkeletonProfileV1::Humanoid => [
            "hips",
            "spine",
            "head",
            "righthand",
            "lefthand",
            "rightfoot",
            "leftfoot",
        ]
        .as_slice(),
        CreatureMotionSkeletonProfileV1::Quadruped => [
            "root",
            "spine",
            "head",
            "frontleftfoot",
            "frontrightfoot",
            "hindleftfoot",
            "hindrightfoot",
        ]
        .as_slice(),
    };
    let missing = required
        .iter()
        .filter(|name| !semantic.contains(**name))
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        let code = if pack.skeleton_profile == CreatureMotionSkeletonProfileV1::Quadruped {
            "CREATURE-MOTION-QUADRUPED-CONTACTS-MISSING"
        } else {
            "CREATURE-MOTION-HUMANOID-SEMANTICS-MISSING"
        };
        return Err(error(
            code,
            "motionPack.semanticJoints",
            format!(
                "required semantic joints are missing: {}",
                missing.join(", ")
            ),
        ));
    }
    if pack.clips.is_empty() {
        return Err(error(
            "CREATURE-MOTION-CLIPS-MISSING",
            "motionPack.clips",
            "motion pack must contain at least one source clip",
        ));
    }
    let mut outputs = BTreeSet::new();
    let mut weapon_families = BTreeSet::new();
    for (index, clip) in pack.clips.iter().enumerate() {
        if clip.source_clip_name.trim().is_empty() || clip.output_clip_name.trim().is_empty() {
            return Err(error(
                "CREATURE-MOTION-CLIP-NAME",
                format!("motionPack.clips[{index}]"),
                "source and output clip names must be non-empty",
            ));
        }
        if !outputs.insert(clip.output_clip_name.to_ascii_lowercase()) {
            return Err(error(
                "CREATURE-MOTION-OUTPUT-DUPLICATE",
                format!("motionPack.clips[{index}].outputClipName"),
                "one motion pack cannot map two sources to the same output state",
            ));
        }
        if let Some(family) = clip.weapon_family {
            weapon_families.insert(family);
        }
    }
    let four_point_contact_ready = pack.skeleton_profile
        == CreatureMotionSkeletonProfileV1::Quadruped
        && required.iter().all(|name| semantic.contains(*name));
    Ok(CreatureMotionPackReportV1 {
        schema_version: 1,
        skeleton_profile: pack.skeleton_profile,
        clip_count: pack.clips.len(),
        weapon_attack_families: weapon_families.into_iter().collect(),
        four_point_contact_ready,
        canonical_sha256: creature_contract_digest_v1(pack)?,
    })
}

/// Binds a declarative MotionPack to the exact joint and animation inventory
/// of one immutable source. This is shared by the humanoid build and the
/// dedicated quadruped intake lane so semantic labels cannot merely be
/// asserted without corresponding source nodes.
pub fn validate_creature_motion_pack_source_v1(
    pack: &CreatureMotionPackV1,
    source_sha256: &str,
    source_joint_names: &[String],
    source_clip_names: &[String],
    selected_weapon_family: Option<CreatureItemGripFamilyV1>,
) -> Result<CreatureMotionSourceReportV1, CreatureProductContractErrorV1> {
    let pack_report = validate_creature_motion_pack_v1(pack)?;
    let expected_identity = format!("sha256:{source_sha256}");
    if pack.source_identity != expected_identity {
        return Err(error(
            "CREATURE-MOTION-SOURCE-MISMATCH",
            "motionPack.sourceIdentity",
            format!(
                "motion pack belongs to {}, but the inspected source is {}",
                pack.source_identity, expected_identity
            ),
        ));
    }
    let joints = source_joint_names
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for (index, semantic) in pack.semantic_joints.iter().enumerate() {
        if !joints.contains(&semantic.to_ascii_lowercase()) {
            return Err(error(
                "CREATURE-MOTION-SOURCE-JOINT-MISSING",
                format!("motionPack.semanticJoints[{index}]"),
                format!("semantic joint {semantic} is absent from the selected source"),
            ));
        }
    }
    let clip_names = source_clip_names
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    for (index, clip) in pack.clips.iter().enumerate() {
        if clip_names
            .iter()
            .filter(|name| name.as_str() == clip.source_clip_name.to_ascii_lowercase())
            .count()
            != 1
        {
            return Err(error(
                "CREATURE-MOTION-SOURCE-CLIP-MISSING",
                format!("motionPack.clips[{index}].sourceClipName"),
                "source clip must resolve to exactly one inspected animation",
            ));
        }
    }
    let selected_clip_count = pack
        .clips
        .iter()
        .filter(|clip| {
            clip.weapon_family.is_none()
                || selected_weapon_family.is_some_and(|family| clip.weapon_family == Some(family))
        })
        .count();
    Ok(CreatureMotionSourceReportV1 {
        schema_version: 1,
        skeleton_profile: pack.skeleton_profile,
        source_identity: expected_identity,
        resolved_joint_count: pack.semantic_joints.len(),
        resolved_clip_count: pack.clips.len(),
        selected_weapon_family,
        selected_clip_count,
        four_point_contact_ready: pack_report.four_point_contact_ready,
        motion_pack_sha256: pack_report.canonical_sha256,
    })
}

// -------------------------------------------------------------------------
// Material and performance profiles

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureMaterialTargetV2 {
    AuroraClassicSafe,
    NwnEeMtr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureAlphaModeV2 {
    Opaque,
    Mask,
    Blend,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureMaterialProfileV2 {
    pub schema_version: u32,
    pub target: CreatureMaterialTargetV2,
    pub normal_maps: bool,
    pub tangent_space_ready: bool,
    pub metallic_roughness_to_specular_gloss: bool,
    pub emissive_to_self_illumination: bool,
    pub alpha_mode: CreatureAlphaModeV2,
    pub double_sided: bool,
}

impl CreatureMaterialProfileV2 {
    pub fn classic_diffuse() -> Self {
        Self {
            schema_version: 2,
            target: CreatureMaterialTargetV2::AuroraClassicSafe,
            normal_maps: false,
            tangent_space_ready: false,
            metallic_roughness_to_specular_gloss: false,
            emissive_to_self_illumination: false,
            alpha_mode: CreatureAlphaModeV2::Opaque,
            double_sided: false,
        }
    }

    pub fn nwn_ee_mtr() -> Self {
        Self {
            schema_version: 2,
            target: CreatureMaterialTargetV2::NwnEeMtr,
            normal_maps: false,
            tangent_space_ready: true,
            metallic_roughness_to_specular_gloss: true,
            emissive_to_self_illumination: true,
            alpha_mode: CreatureAlphaModeV2::Opaque,
            double_sided: false,
        }
    }
}

impl Default for CreatureMaterialProfileV2 {
    fn default() -> Self {
        Self::classic_diffuse()
    }
}

pub fn validate_creature_material_profile_v2(
    profile: &CreatureMaterialProfileV2,
) -> Result<String, CreatureProductContractErrorV1> {
    if profile.schema_version != 2 {
        return Err(error(
            "CREATURE-MATERIAL-SCHEMA",
            "materialProfile.schemaVersion",
            "Creature material profile requires schemaVersion 2",
        ));
    }
    if profile.normal_maps && !profile.tangent_space_ready {
        return Err(error(
            "CREATURE-MATERIAL-TANGENTS-REQUIRED",
            "materialProfile.tangentSpaceReady",
            "normal maps require a finite tangent frame for every mapped segment",
        ));
    }
    if profile.target == CreatureMaterialTargetV2::AuroraClassicSafe
        && (profile.normal_maps
            || profile.metallic_roughness_to_specular_gloss
            || profile.emissive_to_self_illumination
            || profile.alpha_mode != CreatureAlphaModeV2::Opaque
            || profile.double_sided)
    {
        return Err(error(
            "CREATURE-MATERIAL-CLASSIC-UNSUPPORTED",
            "materialProfile",
            "classic-safe output supports diffuse opaque one-sided material semantics only",
        ));
    }
    if profile.alpha_mode == CreatureAlphaModeV2::Blend {
        return Err(error(
            "CREATURE-MATERIAL-ALPHA-BLEND-UNPROVED",
            "materialProfile.alphaMode",
            "alpha blend remains fail-closed until an owner-bound runtime proof exists",
        ));
    }
    creature_contract_digest_v1(profile)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreaturePerformancePresetV1 {
    Compact,
    #[default]
    Standard,
    High,
    Maximum,
}

impl CreaturePerformancePresetV1 {
    pub const fn suggested_triangle_target(self) -> u32 {
        match self {
            Self::Compact => 50_000,
            Self::Standard => 100_000,
            Self::High => 200_000,
            Self::Maximum => SHARED_TRIANGLE_LIMIT_U32,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreaturePerformanceInputV1 {
    pub preset: CreaturePerformancePresetV1,
    pub triangle_count: u32,
    pub segment_count: u32,
    pub material_count: u32,
    pub draw_call_count: u32,
    pub joint_count: u32,
    pub controller_count: u32,
    pub model_bytes: u64,
    pub texture_bytes: u64,
    pub hak_bytes: u64,
    pub instance_count: u32,
}

impl Default for CreaturePerformanceInputV1 {
    fn default() -> Self {
        Self {
            preset: CreaturePerformancePresetV1::Standard,
            triangle_count: 0,
            segment_count: 0,
            material_count: 0,
            draw_call_count: 0,
            joint_count: 0,
            controller_count: 0,
            model_bytes: 0,
            texture_bytes: 0,
            hak_bytes: 0,
            instance_count: 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreaturePerformanceReportV1 {
    pub schema_version: u32,
    pub preset: CreaturePerformancePresetV1,
    pub hard_triangle_limit: u32,
    pub suggested_triangle_target: u32,
    pub suggested_target_met: bool,
    pub triangle_count: u32,
    pub segment_count: u32,
    pub material_count: u32,
    pub draw_call_count: u32,
    pub joint_count: u32,
    pub controller_count: u32,
    pub model_bytes: u64,
    pub texture_bytes: u64,
    pub hak_bytes: u64,
    pub instance_count: u32,
    pub aggregate_instance_triangles: u64,
    pub warnings: Vec<String>,
}

pub fn evaluate_creature_performance_v1(
    input: &CreaturePerformanceInputV1,
) -> Result<CreaturePerformanceReportV1, CreatureProductContractErrorV1> {
    if input.triangle_count > SHARED_TRIANGLE_LIMIT_U32 {
        return Err(error(
            "CREATURE-PERFORMANCE-TRIANGLE-BUDGET",
            "performance.triangleCount",
            format!(
                "triangle count {} exceeds the shared hard limit {}",
                input.triangle_count, SHARED_TRIANGLE_LIMIT_U32
            ),
        ));
    }
    let suggested = input.preset.suggested_triangle_target();
    let mut warnings = Vec::new();
    if input.triangle_count > suggested {
        warnings.push(format!(
            "triangle count exceeds the {:?} suggested target; the shared hard limit is unchanged",
            input.preset
        ));
    }
    if input.draw_call_count > 32 {
        warnings.push("draw-call count exceeds the 32-call review threshold".to_owned());
    }
    if input.hak_bytes > 64 * 1024 * 1024 {
        warnings.push("HAK payload exceeds the 64 MiB review threshold".to_owned());
    }
    Ok(CreaturePerformanceReportV1 {
        schema_version: 1,
        preset: input.preset,
        hard_triangle_limit: SHARED_TRIANGLE_LIMIT_U32,
        suggested_triangle_target: suggested,
        suggested_target_met: input.triangle_count <= suggested,
        triangle_count: input.triangle_count,
        segment_count: input.segment_count,
        material_count: input.material_count,
        draw_call_count: input.draw_call_count,
        joint_count: input.joint_count,
        controller_count: input.controller_count,
        model_bytes: input.model_bytes,
        texture_bytes: input.texture_bytes,
        hak_bytes: input.hak_bytes,
        instance_count: input.instance_count,
        aggregate_instance_triangles: u64::from(input.triangle_count)
            * u64::from(input.instance_count),
        warnings,
    })
}
