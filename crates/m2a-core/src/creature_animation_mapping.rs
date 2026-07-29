use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Deserializer, Serialize, de};
use sha2::{Digest, Sha256};

use crate::{
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    mdl::{MdlAnimationClipV1, MdlAnimationSetV1},
};

pub const CREATURE_ANIMATION_AUTHORING_PROFILE_V1: &str =
    "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DirectCreatureBaseSlotV1(String);

impl DirectCreatureBaseSlotV1 {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DirectCreatureBaseSlotV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<&str> for DirectCreatureBaseSlotV1 {
    type Error = CreatureAnimationMappingErrorV1;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.contains(&value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(CreatureAnimationMappingErrorV1 {
                code: "M2A-ANIMATION-BASE-SLOT-UNKNOWN",
                path: "baseSlot".to_owned(),
                message: format!("unknown direct-creature S/L base slot {value:?}"),
            })
        }
    }
}

impl<'de> Deserialize<'de> for DirectCreatureBaseSlotV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value.as_str()).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DirectCreatureModelTypeV1 {
    #[serde(rename = "S")]
    Simple,
    #[serde(rename = "L")]
    Limited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaybackPolicyV1 {
    /// Standard base-state playback is routed by the engine. This deliberately
    /// does not invent a serialized MDL `loop` field.
    EngineManaged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuroraAnimationStateCatalogEntryV1 {
    pub state_id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub slot: &'static str,
    pub playback_policy: PlaybackPolicyV1,
}

macro_rules! state {
    ($state_id:literal, $label:literal, $description:literal, $slot:literal) => {
        AuroraAnimationStateCatalogEntryV1 {
            state_id: $state_id,
            label: $label,
            description: $description,
            slot: $slot,
            playback_policy: PlaybackPolicyV1::EngineManaged,
        }
    };
}

/// Versioned semantic projection of the exact S/L base namespace.
///
/// The state identifiers and labels are product vocabulary over public format
/// slot names. Playback remains `ENGINE_MANAGED` until the separate runtime
/// loop/state-routing proof closes; P/F model families are intentionally not
/// represented by this catalog.
pub const AURORA_ANIMATION_STATE_CATALOG_V1: [AuroraAnimationStateCatalogEntryV1; 42] = [
    state!(
        "aurora.direct-creature.attack.slash-left",
        "Attack — slash left",
        "Primary left slash attack state.",
        "ca1slashl"
    ),
    state!(
        "aurora.direct-creature.attack.slash-right",
        "Attack — slash right",
        "Primary right slash attack state.",
        "ca1slashr"
    ),
    state!(
        "aurora.direct-creature.attack.stab",
        "Attack — stab",
        "Primary stab attack state.",
        "ca1stab"
    ),
    state!(
        "aurora.direct-creature.attack.reach",
        "Attack — reach",
        "Reach attack state.",
        "creach"
    ),
    state!(
        "aurora.direct-creature.spell.conjure",
        "Spell — conjure",
        "Spell preparation state.",
        "cconjure1"
    ),
    state!(
        "aurora.direct-creature.spell.cast-out",
        "Spell — cast out",
        "Spell release state.",
        "ccastout"
    ),
    state!(
        "aurora.direct-creature.defense.parry-left",
        "Defense — parry left",
        "Left parry state.",
        "cparryl"
    ),
    state!(
        "aurora.direct-creature.defense.parry-right",
        "Defense — parry right",
        "Right parry state.",
        "cparryr"
    ),
    state!(
        "aurora.direct-creature.defense.dodge-lateral",
        "Defense — lateral dodge",
        "Lateral dodge state.",
        "cdodgelr"
    ),
    state!(
        "aurora.direct-creature.defense.dodge-short",
        "Defense — short dodge",
        "Short dodge state.",
        "cdodges"
    ),
    state!(
        "aurora.direct-creature.combat.ready-right",
        "Combat ready — right",
        "Right-side combat-ready state.",
        "creadyr"
    ),
    state!(
        "aurora.direct-creature.combat.ready-left",
        "Combat ready — left",
        "Left-side combat-ready state.",
        "creadyl"
    ),
    state!(
        "aurora.direct-creature.damage.left",
        "Damage — left",
        "Left damage-response state.",
        "cdamagel"
    ),
    state!(
        "aurora.direct-creature.damage.right",
        "Damage — right",
        "Right damage-response state.",
        "cdamager"
    ),
    state!(
        "aurora.direct-creature.damage.short",
        "Damage — short",
        "Short damage-response state.",
        "cdamages"
    ),
    state!(
        "aurora.direct-creature.knockdown.fall",
        "Knockdown — fall",
        "Knockdown transition state.",
        "ckdbck"
    ),
    state!(
        "aurora.direct-creature.knockdown.prone",
        "Knockdown — prone",
        "Knocked-down pose state.",
        "ckdbckps"
    ),
    state!(
        "aurora.direct-creature.death.knockdown",
        "Death — knockdown",
        "Terminal knockdown death transition.",
        "ckdbckdie"
    ),
    state!(
        "aurora.direct-creature.recovery.from-knockdown",
        "Recovery — from knockdown",
        "Get-up transition after knockdown.",
        "cguptokdb"
    ),
    state!(
        "aurora.direct-creature.recovery.stand-back",
        "Recovery — stand back",
        "Back-facing stand-up state.",
        "cgustandb"
    ),
    state!(
        "aurora.direct-creature.locomotion.walk",
        "Locomotion — walk",
        "Standard walk state.",
        "cwalk"
    ),
    state!(
        "aurora.direct-creature.locomotion.run",
        "Locomotion — run",
        "Standard run state.",
        "crun"
    ),
    state!(
        "aurora.direct-creature.locomotion.combat-walk-forward",
        "Combat walk — forward",
        "Forward combat-walk state.",
        "ccwalkf"
    ),
    state!(
        "aurora.direct-creature.locomotion.combat-walk-back",
        "Combat walk — back",
        "Backward combat-walk state.",
        "ccwalkb"
    ),
    state!(
        "aurora.direct-creature.locomotion.combat-walk-left",
        "Combat walk — left",
        "Left combat-walk state.",
        "ccwalkl"
    ),
    state!(
        "aurora.direct-creature.locomotion.combat-walk-right",
        "Combat walk — right",
        "Right combat-walk state.",
        "ccwalkr"
    ),
    state!(
        "aurora.direct-creature.idle.pause",
        "Idle — pause",
        "Standard idle/pause state.",
        "cpause1"
    ),
    state!(
        "aurora.direct-creature.head.turn-left",
        "Head turn — left",
        "Left head-turn state.",
        "chturnl"
    ),
    state!(
        "aurora.direct-creature.head.turn-right",
        "Head turn — right",
        "Right head-turn state.",
        "chturnr"
    ),
    state!(
        "aurora.direct-creature.action.taunt",
        "Action — taunt",
        "Taunt state.",
        "ctaunt"
    ),
    state!(
        "aurora.direct-creature.attack.close-low",
        "Close attack — low",
        "Low close-range attack state.",
        "cclosel"
    ),
    state!(
        "aurora.direct-creature.attack.close-high",
        "Close attack — high",
        "High close-range attack state.",
        "ccloseh"
    ),
    state!(
        "aurora.direct-creature.interaction.get-middle",
        "Interaction — get middle",
        "Get-middle transition state.",
        "cgetmid"
    ),
    state!(
        "aurora.direct-creature.damage.knockdown",
        "Damage — knockdown",
        "Knockdown damage-response state.",
        "ckdbckdmg"
    ),
    state!(
        "aurora.direct-creature.spell.cast-out-loop",
        "Spell — cast hold",
        "Cast-out hold state; exact runtime routing remains engine-managed.",
        "ccastoutlp"
    ),
    state!(
        "aurora.direct-creature.condition.spasm",
        "Condition — spasm",
        "Spasm state.",
        "cspasm"
    ),
    state!(
        "aurora.direct-creature.visibility.appear",
        "Visibility — appear",
        "Appearance transition state.",
        "cappear"
    ),
    state!(
        "aurora.direct-creature.visibility.disappear",
        "Visibility — disappear",
        "Disappearance transition state.",
        "cdisappear"
    ),
    state!(
        "aurora.direct-creature.interaction.get-middle-loop",
        "Interaction — middle hold",
        "Get-middle hold state; exact runtime routing remains engine-managed.",
        "cgetmidlp"
    ),
    state!(
        "aurora.direct-creature.death.dead",
        "Death — dead",
        "Dead-state pose; family motion semantics remain engine-managed.",
        "cdead"
    ),
    state!(
        "aurora.direct-creature.visibility.disappear-loop",
        "Visibility — disappeared hold",
        "Disappearance hold state; exact runtime routing remains engine-managed.",
        "cdisappearlp"
    ),
    state!(
        "aurora.direct-creature.locomotion.combat-turn-right",
        "Combat turn — right",
        "Right combat-turn state.",
        "ccturnr"
    ),
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectCreatureBaseSlotDefinitionV1 {
    pub state_id: String,
    pub label: String,
    pub description: String,
    pub slot: DirectCreatureBaseSlotV1,
    pub gameplay_floor: bool,
    pub supported_model_types: Vec<DirectCreatureModelTypeV1>,
    pub playback_policy: PlaybackPolicyV1,
}

pub type AuroraAnimationStateDefinitionV1 = DirectCreatureBaseSlotDefinitionV1;

pub fn is_gameplay_floor_slot_v1(slot: &str) -> bool {
    matches!(
        slot,
        "cappear" | "cpause1" | "cwalk" | "crun" | "ca1slashl" | "cdamagel" | "cdead"
    )
}

pub fn direct_creature_base_catalog_v1() -> Vec<DirectCreatureBaseSlotDefinitionV1> {
    AURORA_ANIMATION_STATE_CATALOG_V1
        .iter()
        .map(|entry| DirectCreatureBaseSlotDefinitionV1 {
            state_id: entry.state_id.to_owned(),
            label: entry.label.to_owned(),
            description: entry.description.to_owned(),
            slot: DirectCreatureBaseSlotV1(entry.slot.to_owned()),
            gameplay_floor: is_gameplay_floor_slot_v1(entry.slot),
            supported_model_types: vec![
                DirectCreatureModelTypeV1::Simple,
                DirectCreatureModelTypeV1::Limited,
            ],
            playback_policy: entry.playback_policy,
        })
        .collect()
}

pub fn resolve_base_slot_for_state_v1(
    state_id: &str,
    _model_type: DirectCreatureModelTypeV1,
) -> Result<DirectCreatureBaseSlotV1, CreatureAnimationMappingErrorV1> {
    AURORA_ANIMATION_STATE_CATALOG_V1
        .iter()
        .find(|entry| entry.state_id == state_id)
        .map(|entry| DirectCreatureBaseSlotV1(entry.slot.to_owned()))
        .ok_or_else(|| CreatureAnimationMappingErrorV1 {
            code: "M2A-ANIMATION-STATE-UNKNOWN",
            path: "stateId".to_owned(),
            message: format!("unknown Aurora direct-creature animation state {state_id:?}"),
        })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceAnimationClipV1 {
    pub clip_id: String,
    pub name: String,
    pub duration_seconds: f32,
    pub track_count: u32,
    pub target_node_ids: Vec<u32>,
    #[serde(default)]
    pub target_paths: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationProviderV1 {
    SourceGlb,
    CompatibleSupermodel,
    ProceduralGenerator,
    UserCustom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationOwnershipV1 {
    UserOwned,
    EngineInherited,
    ProjectGenerated,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationMappingProvenanceV1 {
    pub provider: AnimationProviderV1,
    /// Portable asset identity, never a filesystem path.
    pub asset_id: String,
    pub ownership: AnimationOwnershipV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationSourceKindV1 {
    SourceClip,
    InheritedSupermodel,
    Procedural,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationSourceAssignmentV1 {
    pub target_slot: DirectCreatureBaseSlotV1,
    pub source_kind: AnimationSourceKindV1,
    pub source_clip_name: Option<String>,
    pub custom_animation_id: Option<String>,
    pub provenance: AnimationMappingProvenanceV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationFallbackReviewV1 {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationFallbackDecisionV1 {
    pub id: String,
    pub target_slot: DirectCreatureBaseSlotV1,
    pub source_slot: DirectCreatureBaseSlotV1,
    pub reason: String,
    pub review: AnimationFallbackReviewV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomAnimationPlaybackV1 {
    OneShot,
    LoopingPhased,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomAnimationPhaseKindV1 {
    Start,
    Loop,
    End,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomAnimationPhaseV1 {
    pub phase: CustomAnimationPhaseKindV1,
    pub source_clip_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomAnimationDefinitionV1 {
    pub id: String,
    pub name: String,
    pub playback: CustomAnimationPlaybackV1,
    /// Used only by `ONE_SHOT`; phased loops use `phases`.
    pub source_clip_name: Option<String>,
    pub phases: Vec<CustomAnimationPhaseV1>,
    pub provenance: AnimationMappingProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureAnimationAuthoringV1 {
    pub schema_version: u32,
    pub profile: String,
    pub model_type: DirectCreatureModelTypeV1,
    pub source_revision: String,
    pub authoring_revision: u64,
    pub assignments: Vec<AnimationSourceAssignmentV1>,
    pub fallbacks: Vec<AnimationFallbackDecisionV1>,
    pub custom_animations: Vec<CustomAnimationDefinitionV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationMappingDiagnosticLevelV1 {
    Info,
    Warning,
    Blocking,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureAnimationMappingDiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub level: AnimationMappingDiagnosticLevelV1,
    pub message: String,
    pub action: String,
}

pub type AnimationMappingDiagnosticV1 = CreatureAnimationMappingDiagnosticV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreatureAnimationMappingStatusV1 {
    Ready,
    NeedsReview,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedCreatureBaseAnimationV1 {
    pub target_slot: DirectCreatureBaseSlotV1,
    pub resolved_source_slot: DirectCreatureBaseSlotV1,
    pub assignment: AnimationSourceAssignmentV1,
    pub via_fallback_slots: Vec<DirectCreatureBaseSlotV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedCreatureCustomAnimationV1 {
    pub id: String,
    pub playback: CustomAnimationPlaybackV1,
    pub phases: Vec<CustomAnimationPhaseKindV1>,
    pub output_clip_names: Vec<String>,
    pub source_clip_names: Vec<String>,
    pub provenance: AnimationMappingProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedCreatureAnimationMappingV1 {
    pub schema_version: u32,
    pub profile: String,
    pub source_revision: String,
    pub authoring_revision: u64,
    pub authoring_fingerprint_sha256: String,
    pub base_animations: Vec<ResolvedCreatureBaseAnimationV1>,
    pub custom_animations: Vec<ResolvedCreatureCustomAnimationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatureAnimationAuthoringValidationV1 {
    pub schema_version: u32,
    pub status: CreatureAnimationMappingStatusV1,
    pub mapped_base_slot_count: u32,
    pub review_count: u32,
    pub blocking_count: u32,
    pub custom_animation_count: u32,
    pub authoring_fingerprint_sha256: String,
    pub diagnostics: Vec<CreatureAnimationMappingDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAnimationConformanceV1 {
    pub schema_version: u32,
    pub status: CreatureAnimationMappingStatusV1,
    pub expected_base_slot_count: u32,
    pub materialized_base_slot_count: u32,
    pub expected_custom_clip_names: Vec<String>,
    pub materialized_custom_clip_names: Vec<String>,
    pub diagnostics: Vec<CreatureAnimationMappingDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatureAnimationMappingErrorV1 {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl fmt::Display for CreatureAnimationMappingErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for CreatureAnimationMappingErrorV1 {}

pub fn validate_creature_animation_authoring_contract_v1(
    authoring: &CreatureAnimationAuthoringV1,
) -> Vec<CreatureAnimationMappingDiagnosticV1> {
    let mut diagnostics = Vec::new();
    if authoring.schema_version != 1 {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-SCHEMA-VERSION",
            "schemaVersion",
            AnimationMappingDiagnosticLevelV1::Blocking,
            "creature animation authoring schemaVersion must be 1",
            "Open the project with a compatible Studio version.",
        ));
    }
    if authoring.profile != CREATURE_ANIMATION_AUTHORING_PROFILE_V1 {
        diagnostics.push(diagnostic(
            "M2A-ANIMATION-PROFILE",
            "profile",
            AnimationMappingDiagnosticLevelV1::Blocking,
            "creature animation authoring profile is unsupported",
            "Recreate or migrate the animation mapping document.",
        ));
    }

    let mut assigned_slots = BTreeSet::new();
    for (index, assignment) in authoring.assignments.iter().enumerate() {
        if !assigned_slots.insert(assignment.target_slot.as_str()) {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-ASSIGNMENT-DUPLICATE",
                format!("assignments[{index}].targetSlot"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "each base slot may have at most one direct assignment",
                "Keep one source assignment for this target slot.",
            ));
        }
        if assignment.source_kind == AnimationSourceKindV1::InheritedSupermodel
            || assignment.provenance.provider == AnimationProviderV1::CompatibleSupermodel
        {
            if looks_like_filesystem_path(&assignment.provenance.asset_id) {
                diagnostics.push(diagnostic(
                    "M2A-ANIMATION-SUPERMODEL-PATH-FORBIDDEN",
                    format!("assignments[{index}].provenance.assetId"),
                    AnimationMappingDiagnosticLevelV1::Blocking,
                    "supermodel provenance must use a portable asset identity, not a filesystem path",
                    "Select a compatible built-in provider identity.",
                ));
            } else {
                diagnostics.push(diagnostic(
                    "M2A-ANIMATION-SUPERMODEL-PROVIDER-UNAVAILABLE",
                    format!("assignments[{index}].provenance.assetId"),
                    AnimationMappingDiagnosticLevelV1::Blocking,
                    "no compatible packaged supermodel provider is available for the authored V4 build lane",
                    "Use a local or generated source until a versioned compatible provider is installed.",
                ));
            }
        }
    }

    for (index, fallback) in authoring.fallbacks.iter().enumerate() {
        if fallback.review == AnimationFallbackReviewV1::Pending {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-FALLBACK-REVIEW-REQUIRED",
                format!("fallbacks[{index}].review"),
                AnimationMappingDiagnosticLevelV1::Warning,
                "fallback requires an explicit accept or reject decision",
                "Review the fallback target, source and reason.",
            ));
        }
    }

    for (index, custom) in authoring.custom_animations.iter().enumerate() {
        match custom.playback {
            CustomAnimationPlaybackV1::OneShot => {
                if custom.source_clip_name.as_deref().is_none_or(str::is_empty)
                    || !custom.phases.is_empty()
                {
                    diagnostics.push(diagnostic(
                        "M2A-ANIMATION-CUSTOM-ONE-SHOT-PHASES",
                        format!("customAnimations[{index}]"),
                        AnimationMappingDiagnosticLevelV1::Blocking,
                        "one-shot custom animation requires one source clip and no START/LOOP/END phases",
                        "Remove phased clips or switch to LOOPING_PHASED.",
                    ));
                }
            }
            CustomAnimationPlaybackV1::LoopingPhased => {
                let loop_count = custom
                    .phases
                    .iter()
                    .filter(|phase| phase.phase == CustomAnimationPhaseKindV1::Loop)
                    .count();
                let unique_phase_count = custom
                    .phases
                    .iter()
                    .map(|phase| phase.phase as u8)
                    .collect::<BTreeSet<_>>()
                    .len();
                if custom.source_clip_name.is_some()
                    || loop_count != 1
                    || unique_phase_count != custom.phases.len()
                {
                    diagnostics.push(diagnostic(
                        "M2A-ANIMATION-CUSTOM-LOOP-PHASES",
                        format!("customAnimations[{index}]"),
                        AnimationMappingDiagnosticLevelV1::Blocking,
                        "looping custom animation requires exactly one LOOP and at most one START and END phase",
                        "Correct the phased source clips.",
                    ));
                }
            }
        }
    }
    validate_custom_output_names(authoring, &mut diagnostics);

    diagnostics
}

/// Canonical readiness validation used by WASM and the Studio worker.
///
/// A pending fallback remains `NEEDS_REVIEW`; it is never silently accepted.
/// Missing coverage, cycles and malformed custom definitions are blocking.
pub fn validate_creature_animation_authoring_v1(
    authoring: &CreatureAnimationAuthoringV1,
) -> CreatureAnimationAuthoringValidationV1 {
    let mut diagnostics = validate_creature_animation_authoring_contract_v1(authoring);
    validate_assignment_shapes(authoring, &mut diagnostics);
    validate_fallback_graph(authoring, &mut diagnostics);
    validate_custom_definitions(authoring, &mut diagnostics);

    let mut mapped_base_slot_count = 0_u32;
    for (index, slot) in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().enumerate() {
        match resolve_effective_assignment(authoring, slot, true) {
            Ok(Some(_)) => mapped_base_slot_count += 1,
            Ok(None) => diagnostics.push(diagnostic(
                "M2A-ANIMATION-BASE-SLOT-MISSING",
                format!("baseSlots[{index}]"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                format!("required direct-creature base slot {slot} has no effective source"),
                "Assign a compatible source or review an explicit fallback.",
            )),
            Err(entry) => diagnostics.push(entry),
        }
    }
    sort_diagnostics(&mut diagnostics);
    let blocking_count = diagnostics
        .iter()
        .filter(|entry| entry.level == AnimationMappingDiagnosticLevelV1::Blocking)
        .count() as u32;
    let review_count = diagnostics
        .iter()
        .filter(|entry| entry.level == AnimationMappingDiagnosticLevelV1::Warning)
        .count() as u32;
    let status = if blocking_count > 0 {
        CreatureAnimationMappingStatusV1::Blocked
    } else if review_count > 0 {
        CreatureAnimationMappingStatusV1::NeedsReview
    } else {
        CreatureAnimationMappingStatusV1::Ready
    };
    CreatureAnimationAuthoringValidationV1 {
        schema_version: 1,
        status,
        mapped_base_slot_count,
        review_count,
        blocking_count,
        custom_animation_count: authoring.custom_animations.len() as u32,
        authoring_fingerprint_sha256: creature_animation_authoring_fingerprint_v1(authoring),
        diagnostics,
    }
}

pub fn resolve_creature_animation_mapping_v1(
    authoring: &CreatureAnimationAuthoringV1,
) -> Result<ResolvedCreatureAnimationMappingV1, CreatureAnimationMappingErrorV1> {
    let validation = validate_creature_animation_authoring_v1(authoring);
    if validation.status != CreatureAnimationMappingStatusV1::Ready {
        return Err(CreatureAnimationMappingErrorV1 {
            code: "M2A-ANIMATION-AUTHORING-NOT-READY",
            path: "animationAuthoring".to_owned(),
            message: format!(
                "animation authoring is {:?}: {} blocking and {} review diagnostic(s)",
                validation.status, validation.blocking_count, validation.review_count
            ),
        });
    }
    let mut base_animations = Vec::with_capacity(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len());
    for slot in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 {
        let (assignment, resolved_source_slot, via_fallback_slots) =
            resolve_effective_assignment(authoring, slot, false)
                .map_err(|entry| CreatureAnimationMappingErrorV1 {
                    code: "M2A-ANIMATION-RESOLUTION",
                    path: entry.path,
                    message: entry.message,
                })?
                .ok_or_else(|| CreatureAnimationMappingErrorV1 {
                    code: "M2A-ANIMATION-BASE-SLOT-MISSING",
                    path: format!("baseSlots.{slot}"),
                    message: format!("required base slot {slot} has no effective source"),
                })?;
        base_animations.push(ResolvedCreatureBaseAnimationV1 {
            target_slot: DirectCreatureBaseSlotV1(slot.to_owned()),
            resolved_source_slot,
            assignment,
            via_fallback_slots,
        });
    }
    let custom_animations = authoring
        .custom_animations
        .iter()
        .map(resolve_custom_definition)
        .collect();
    Ok(ResolvedCreatureAnimationMappingV1 {
        schema_version: 1,
        profile: authoring.profile.clone(),
        source_revision: authoring.source_revision.clone(),
        authoring_revision: authoring.authoring_revision,
        authoring_fingerprint_sha256: validation.authoring_fingerprint_sha256,
        base_animations,
        custom_animations,
    })
}

/// Materializes only sources explicitly selected by the authoring document.
/// `procedural` is a separately generated full-42 set and is consulted solely
/// for assignments whose source kind is `PROCEDURAL`.
pub fn materialize_authored_direct_creature_clips_v1(
    source: &MdlAnimationSetV1,
    procedural: Option<&MdlAnimationSetV1>,
    authoring: &CreatureAnimationAuthoringV1,
) -> Result<MdlAnimationSetV1, CreatureAnimationMappingErrorV1> {
    let resolved = resolve_creature_animation_mapping_v1(authoring)?;
    let mut clips =
        Vec::with_capacity(resolved.base_animations.len() + resolved.custom_animations.len() * 3);
    for base in &resolved.base_animations {
        let source_clip = match base.assignment.source_kind {
            AnimationSourceKindV1::SourceClip => find_source_clip(
                source,
                base.assignment.source_clip_name.as_deref(),
                "M2A-ANIMATION-SOURCE-CLIP-MISSING",
            )?,
            AnimationSourceKindV1::Procedural => find_named_clip(
                procedural.ok_or_else(|| CreatureAnimationMappingErrorV1 {
                    code: "M2A-ANIMATION-PROCEDURAL-SET-MISSING",
                    path: format!("baseSlots.{}", base.target_slot),
                    message: "procedural assignment requires the generated full-42 clip set"
                        .to_owned(),
                })?,
                base.target_slot.as_str(),
                "M2A-ANIMATION-PROCEDURAL-CLIP-MISSING",
            )?,
            AnimationSourceKindV1::Custom => {
                let custom_id =
                    base.assignment
                        .custom_animation_id
                        .as_deref()
                        .ok_or_else(|| CreatureAnimationMappingErrorV1 {
                            code: "M2A-ANIMATION-CUSTOM-ID-MISSING",
                            path: format!("baseSlots.{}", base.target_slot),
                            message: "custom source assignment requires customAnimationId"
                                .to_owned(),
                        })?;
                let custom = authoring
                    .custom_animations
                    .iter()
                    .find(|candidate| candidate.id == custom_id)
                    .ok_or_else(|| CreatureAnimationMappingErrorV1 {
                        code: "M2A-ANIMATION-CUSTOM-UNKNOWN",
                        path: format!("customAnimations.{custom_id}"),
                        message: "custom source assignment references an unknown definition"
                            .to_owned(),
                    })?;
                find_source_clip(
                    source,
                    custom_primary_source_clip(custom),
                    "M2A-ANIMATION-CUSTOM-SOURCE-CLIP-MISSING",
                )?
            }
            AnimationSourceKindV1::InheritedSupermodel => {
                return Err(CreatureAnimationMappingErrorV1 {
                    code: "M2A-ANIMATION-SUPERMODEL-BUILD-UNSUPPORTED",
                    path: format!("baseSlots.{}", base.target_slot),
                    message: "the authored local-clip build lane cannot materialize an external supermodel source"
                        .to_owned(),
                });
            }
        };
        clips.push(clone_as(source_clip, base.target_slot.as_str()));
    }
    for custom in &authoring.custom_animations {
        match custom.playback {
            CustomAnimationPlaybackV1::OneShot => {
                let source_clip = find_source_clip(
                    source,
                    custom.source_clip_name.as_deref(),
                    "M2A-ANIMATION-CUSTOM-SOURCE-CLIP-MISSING",
                )?;
                clips.push(clone_as(source_clip, &custom.name));
            }
            CustomAnimationPlaybackV1::LoopingPhased => {
                for phase in &custom.phases {
                    let output_name = custom_phase_output_name(&custom.name, phase.phase);
                    let source_clip = find_named_clip(
                        source,
                        &phase.source_clip_name,
                        "M2A-ANIMATION-CUSTOM-SOURCE-CLIP-MISSING",
                    )?;
                    clips.push(clone_as(source_clip, &output_name));
                }
            }
        }
    }
    Ok(MdlAnimationSetV1 {
        schema_version: source.schema_version,
        clips,
    })
}

pub fn evaluate_authored_animation_conformance_v1(
    authoring: &CreatureAnimationAuthoringV1,
    materialized: &MdlAnimationSetV1,
) -> AuthoredAnimationConformanceV1 {
    let expected_custom_clip_names = authoring
        .custom_animations
        .iter()
        .flat_map(|custom| match custom.playback {
            CustomAnimationPlaybackV1::OneShot => vec![custom.name.clone()],
            CustomAnimationPlaybackV1::LoopingPhased => custom
                .phases
                .iter()
                .map(|phase| custom_phase_output_name(&custom.name, phase.phase))
                .collect(),
        })
        .collect::<Vec<_>>();
    let folded = materialized
        .clips
        .iter()
        .map(|clip| clip.name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    for slot in FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 {
        if !folded.contains(slot) {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-BUILT-BASE-SLOT-MISSING",
                format!("materialized.clips.{slot}"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                format!("materialized animation set is missing base slot {slot}"),
                "Rebuild from the current ready authoring document.",
            ));
        }
    }
    for name in &expected_custom_clip_names {
        if !folded.contains(&name.to_ascii_lowercase()) {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-BUILT-CUSTOM-MISSING",
                format!("materialized.clips.{name}"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                format!("materialized animation set is missing custom clip {name}"),
                "Repair the custom source mapping and rebuild.",
            ));
        }
    }
    sort_diagnostics(&mut diagnostics);
    let materialized_custom_clip_names = expected_custom_clip_names
        .iter()
        .filter(|name| folded.contains(&name.to_ascii_lowercase()))
        .cloned()
        .collect::<Vec<_>>();
    let materialized_base_slot_count = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .filter(|slot| folded.contains(**slot))
        .count() as u32;
    AuthoredAnimationConformanceV1 {
        schema_version: 1,
        status: if diagnostics.is_empty() {
            CreatureAnimationMappingStatusV1::Ready
        } else {
            CreatureAnimationMappingStatusV1::Blocked
        },
        expected_base_slot_count: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() as u32,
        materialized_base_slot_count,
        expected_custom_clip_names,
        materialized_custom_clip_names,
        diagnostics,
    }
}

pub fn creature_animation_authoring_fingerprint_v1(
    authoring: &CreatureAnimationAuthoringV1,
) -> String {
    let canonical = serde_json::to_vec(authoring)
        .expect("CreatureAnimationAuthoringV1 contains only JSON-serializable fields");
    Sha256::digest(canonical)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_assignment_shapes(
    authoring: &CreatureAnimationAuthoringV1,
    diagnostics: &mut Vec<CreatureAnimationMappingDiagnosticV1>,
) {
    for (index, assignment) in authoring.assignments.iter().enumerate() {
        let valid = match assignment.source_kind {
            AnimationSourceKindV1::SourceClip => {
                assignment
                    .source_clip_name
                    .as_deref()
                    .is_some_and(|name| !name.trim().is_empty())
                    && assignment.custom_animation_id.is_none()
            }
            AnimationSourceKindV1::Custom => {
                assignment.source_clip_name.is_none()
                    && assignment
                        .custom_animation_id
                        .as_deref()
                        .is_some_and(|id| !id.trim().is_empty())
            }
            AnimationSourceKindV1::Procedural | AnimationSourceKindV1::InheritedSupermodel => {
                assignment.source_clip_name.is_none() && assignment.custom_animation_id.is_none()
            }
        };
        if !valid {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-ASSIGNMENT-SHAPE",
                format!("assignments[{index}]"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "animation assignment fields do not match its sourceKind",
                "Select the source again from the compatible source picker.",
            ));
        }
        if assignment.source_kind == AnimationSourceKindV1::Custom
            && !authoring
                .custom_animations
                .iter()
                .any(|custom| Some(custom.id.as_str()) == assignment.custom_animation_id.as_deref())
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-CUSTOM-UNKNOWN",
                format!("assignments[{index}].customAnimationId"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "assignment references an unknown custom animation",
                "Select an existing custom animation or remove this assignment.",
            ));
        }
    }
}

fn validate_fallback_graph(
    authoring: &CreatureAnimationAuthoringV1,
    diagnostics: &mut Vec<CreatureAnimationMappingDiagnosticV1>,
) {
    let mut targets = BTreeSet::new();
    for (index, fallback) in authoring.fallbacks.iter().enumerate() {
        if fallback.target_slot == fallback.source_slot {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-FALLBACK-SELF",
                format!("fallbacks[{index}]"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "fallback target and source must differ",
                "Choose another source slot.",
            ));
        }
        if fallback.review != AnimationFallbackReviewV1::Rejected
            && !targets.insert(fallback.target_slot.clone())
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-FALLBACK-DUPLICATE",
                format!("fallbacks[{index}].targetSlot"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "a base slot may have at most one active fallback",
                "Keep one fallback decision for this target.",
            ));
        }
    }
    let active = authoring
        .fallbacks
        .iter()
        .filter(|fallback| fallback.review != AnimationFallbackReviewV1::Rejected)
        .map(|fallback| (fallback.target_slot.as_str(), fallback.source_slot.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut reported = BTreeSet::new();
    for start in active.keys().copied() {
        let mut order = Vec::new();
        let mut positions = BTreeMap::new();
        let mut current = start;
        while let Some(next) = active.get(current).copied() {
            if let Some(cycle_start) = positions.get(current).copied() {
                let cycle: Vec<&str> = order[cycle_start..].to_vec();
                let mut identities = (0..cycle.len())
                    .map(|index| {
                        cycle[index..]
                            .iter()
                            .chain(cycle[..index].iter())
                            .copied()
                            .collect::<Vec<_>>()
                            .join(">")
                    })
                    .collect::<Vec<_>>();
                identities.sort();
                if let Some(identity) = identities.first()
                    && reported.insert(identity.clone())
                {
                    diagnostics.push(diagnostic(
                        "M2A-ANIMATION-FALLBACK-CYCLE",
                        format!("fallbacks.{identity}"),
                        AnimationMappingDiagnosticLevelV1::Blocking,
                        format!("fallback cycle detected: {identity}"),
                        "Remove one fallback edge from the cycle.",
                    ));
                }
                break;
            }
            positions.insert(current, order.len());
            order.push(current);
            current = next;
        }
    }
}

fn validate_custom_definitions(
    authoring: &CreatureAnimationAuthoringV1,
    diagnostics: &mut Vec<CreatureAnimationMappingDiagnosticV1>,
) {
    let base = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, custom) in authoring.custom_animations.iter().enumerate() {
        let folded = custom.name.to_ascii_lowercase();
        if !ids.insert(custom.id.as_str()) {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-CUSTOM-ID-DUPLICATE",
                format!("customAnimations[{index}].id"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "custom animation IDs must be unique",
                "Create a new stable custom animation ID.",
            ));
        }
        let mut name_bytes = custom.name.bytes();
        if custom.name.is_empty()
            || custom.name.len() > 16
            || !name_bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic())
            || !name_bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-CUSTOM-NAME",
                format!("customAnimations[{index}].name"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "custom animation name must be a 1..16 character ASCII resref",
                "Use only ASCII letters, digits and underscore.",
            ));
        }
        if base.contains(&folded) || !names.insert(folded) {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-CUSTOM-NAME-CONFLICT",
                format!("customAnimations[{index}].name"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "custom animation name conflicts with a base slot or another custom name",
                "Choose a unique custom animation name.",
            ));
        }
        if custom.playback == CustomAnimationPlaybackV1::LoopingPhased
            && custom
                .phases
                .iter()
                .any(|phase| custom_phase_output_name(&custom.name, phase.phase).len() > 16)
        {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-CUSTOM-PHASE-NAME",
                format!("customAnimations[{index}].name"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "phased custom animation output names must fit the 16-character resref boundary",
                "Shorten the custom name to at most 14 characters.",
            ));
        }
    }
}

fn validate_custom_output_names(
    authoring: &CreatureAnimationAuthoringV1,
    diagnostics: &mut Vec<CreatureAnimationMappingDiagnosticV1>,
) {
    let mut output_names = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for (index, custom) in authoring.custom_animations.iter().enumerate() {
        let materialized_names = match custom.playback {
            CustomAnimationPlaybackV1::OneShot => vec![custom.name.clone()],
            CustomAnimationPlaybackV1::LoopingPhased => custom
                .phases
                .iter()
                .map(|phase| custom_phase_output_name(&custom.name, phase.phase))
                .collect(),
        };
        let mut output_conflict = false;
        for name in materialized_names {
            output_conflict |= !output_names.insert(name.to_ascii_lowercase());
        }
        if output_conflict {
            diagnostics.push(diagnostic(
                "M2A-ANIMATION-CUSTOM-OUTPUT-NAME-CONFLICT",
                format!("customAnimations[{index}].name"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                "materialized custom animation clip names must be unique across Base 42 and custom definitions",
                "Rename one custom animation so every materialized clip name is unique.",
            ));
        }
    }
}

type EffectiveAssignment = (
    AnimationSourceAssignmentV1,
    DirectCreatureBaseSlotV1,
    Vec<DirectCreatureBaseSlotV1>,
);

fn resolve_effective_assignment(
    authoring: &CreatureAnimationAuthoringV1,
    slot: &str,
    allow_pending: bool,
) -> Result<Option<EffectiveAssignment>, CreatureAnimationMappingDiagnosticV1> {
    let assignments = authoring
        .assignments
        .iter()
        .map(|assignment| (assignment.target_slot.as_str(), assignment))
        .collect::<BTreeMap<_, _>>();
    let fallbacks = authoring
        .fallbacks
        .iter()
        .filter(|fallback| fallback.review != AnimationFallbackReviewV1::Rejected)
        .map(|fallback| (fallback.target_slot.as_str(), fallback))
        .collect::<BTreeMap<_, _>>();
    let mut visited = BTreeSet::new();
    let mut via = Vec::new();
    let mut current = slot;
    for _depth in 0..=FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() {
        if let Some(assignment) = assignments.get(current) {
            return Ok(Some((
                (*assignment).clone(),
                DirectCreatureBaseSlotV1(current.to_owned()),
                via,
            )));
        }
        if !visited.insert(current) {
            return Err(diagnostic(
                "M2A-ANIMATION-FALLBACK-CYCLE",
                format!("baseSlots.{slot}"),
                AnimationMappingDiagnosticLevelV1::Blocking,
                format!("fallback cycle encountered while resolving {slot}"),
                "Remove one fallback edge from the cycle.",
            ));
        }
        let Some(fallback) = fallbacks.get(current) else {
            return Ok(None);
        };
        if fallback.review == AnimationFallbackReviewV1::Pending && !allow_pending {
            return Ok(None);
        }
        via.push(DirectCreatureBaseSlotV1(current.to_owned()));
        current = fallback.source_slot.as_str();
    }
    Err(diagnostic(
        "M2A-ANIMATION-FALLBACK-DEPTH",
        format!("baseSlots.{slot}"),
        AnimationMappingDiagnosticLevelV1::Blocking,
        "fallback chain exceeds the 42-slot resolution bound",
        "Shorten the fallback chain.",
    ))
}

fn resolve_custom_definition(
    custom: &CustomAnimationDefinitionV1,
) -> ResolvedCreatureCustomAnimationV1 {
    let (output_clip_names, source_clip_names) = match custom.playback {
        CustomAnimationPlaybackV1::OneShot => (
            vec![custom.name.clone()],
            custom.source_clip_name.clone().into_iter().collect(),
        ),
        CustomAnimationPlaybackV1::LoopingPhased => (
            custom
                .phases
                .iter()
                .map(|phase| custom_phase_output_name(&custom.name, phase.phase))
                .collect(),
            custom
                .phases
                .iter()
                .map(|phase| phase.source_clip_name.clone())
                .collect(),
        ),
    };
    ResolvedCreatureCustomAnimationV1 {
        id: custom.id.clone(),
        playback: custom.playback,
        phases: custom.phases.iter().map(|phase| phase.phase).collect(),
        output_clip_names,
        source_clip_names,
        provenance: custom.provenance.clone(),
    }
}

fn custom_primary_source_clip(custom: &CustomAnimationDefinitionV1) -> Option<&str> {
    match custom.playback {
        CustomAnimationPlaybackV1::OneShot => custom.source_clip_name.as_deref(),
        CustomAnimationPlaybackV1::LoopingPhased => custom
            .phases
            .iter()
            .find(|phase| phase.phase == CustomAnimationPhaseKindV1::Loop)
            .map(|phase| phase.source_clip_name.as_str()),
    }
}

fn custom_phase_output_name(name: &str, phase: CustomAnimationPhaseKindV1) -> String {
    match phase {
        CustomAnimationPhaseKindV1::Start => format!("{name}_s"),
        CustomAnimationPhaseKindV1::Loop => name.to_owned(),
        CustomAnimationPhaseKindV1::End => format!("{name}_e"),
    }
}

fn find_source_clip<'a>(
    source: &'a MdlAnimationSetV1,
    name: Option<&str>,
    code: &'static str,
) -> Result<&'a MdlAnimationClipV1, CreatureAnimationMappingErrorV1> {
    let name = name.ok_or_else(|| CreatureAnimationMappingErrorV1 {
        code,
        path: "animationSource.sourceClipName".to_owned(),
        message: "animation source clip name is missing".to_owned(),
    })?;
    find_named_clip(source, name, code)
}

fn find_named_clip<'a>(
    source: &'a MdlAnimationSetV1,
    name: &str,
    code: &'static str,
) -> Result<&'a MdlAnimationClipV1, CreatureAnimationMappingErrorV1> {
    source
        .clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| CreatureAnimationMappingErrorV1 {
            code,
            path: format!("sourceAnimations.{name}"),
            message: format!("source animation clip {name:?} is unavailable"),
        })
}

fn clone_as(source: &MdlAnimationClipV1, output_name: &str) -> MdlAnimationClipV1 {
    let mut output = source.clone();
    output.name = output_name.to_owned();
    output
}

fn sort_diagnostics(diagnostics: &mut [CreatureAnimationMappingDiagnosticV1]) {
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.code.cmp(&right.code))
            .then(left.message.cmp(&right.message))
    });
}

fn looks_like_filesystem_path(value: &str) -> bool {
    value.contains('/')
        || value.contains('\\')
        || value.as_bytes().get(1) == Some(&b':')
        || value.starts_with('.')
}

fn diagnostic(
    code: impl Into<String>,
    path: impl Into<String>,
    level: AnimationMappingDiagnosticLevelV1,
    message: impl Into<String>,
    action: impl Into<String>,
) -> CreatureAnimationMappingDiagnosticV1 {
    CreatureAnimationMappingDiagnosticV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        level,
        message: message.into(),
        action: action.into(),
    }
}
