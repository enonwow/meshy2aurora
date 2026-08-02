//! Generated, self-contained creature proof module.
//!
//! This is intentionally a structural Aurora proof artefact, not a claim that
//! a particular Toolset/game session has loaded it.  The resource family and
//! field labels are derived from the local Aurora audit: `module.ifo`, the
//! `ARE`/`GIC`/`GIT` area triplet, a creature `UTC`, `Mod_HakList`, and the
//! exact `Creature List` placement fields.  No retail resource payload is
//! copied into this archive.

use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLimitsV1, GffLocStringV1, GffLocSubstringV1,
        GffStructV1, GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
    },
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1},
};

/// The sole canonical runtime-test module. Its 16-byte resref identifies
/// both its Codex provenance and its strictly limited animation-proof role.
pub const PROOF_MODULE_RESREF: &str = "m2a_codex_aproof";
pub const PROOF_AREA_RESREF: &str = "m2a_caproof_area";
pub const PROOF_CREATURE_RESREF: &str = "m2a_caproof_h1";
pub const PROOF_HAK_RESREF: &str = "m2a_codex_aproof";
/// Canonical M0 fixture placed in the single project runtime-proof module.
/// The model resref stays `m2a_m0p01`; this is the module-local UTC resref.
pub const M0_CANONICAL_PROOF_CREATURE_RESREF: &str = "m2a_caproof_m0";

/// M0 keeps a separate module, area and HAK so it cannot overwrite the H1
/// proof.  The tortoise remains an external positive control: this package
/// references its installed HAK but never embeds its model or texture bytes.
pub const M0_PROOF_MODULE_RESREF: &str = "m2a_m0_proof";
pub const M0_PROOF_AREA_RESREF: &str = "m2a_m0proof_area";
pub const M0_PROOF_CREATURE_RESREF: &str = "m2a_m0p01";
pub const M0_CONTROL_CREATURE_RESREF: &str = "m2a_m0_tort";
pub const M0_PROOF_HAK_RESREF: &str = "m2a_m0_proof";
pub const M0_TORTOISE_REFERENCE_HAK_RESREF: &str = "znd_tortoise";

/// A fresh, caller-owned binary vertical-slice target. It intentionally does
/// not reuse the stalled Toolset-created module or the invalid M0 control
/// Area. Its Area layout follows the fresh Aurora-created `tms01` 2x2
/// precedent recorded for M0 r21.
pub const BINARY_M0_MODULE_RESREF: &str = "m2a_bm0p1";
pub const BINARY_M0_AREA_RESREF: &str = "m2a_bm0a1";
pub const BINARY_M0_CREATURE_TEMPLATE_RESREF: &str = "nw_dwarfmerc001";
pub const BINARY_M0_PROOF_HAK_RESREF: &str = M0_PROOF_HAK_RESREF;

/// Immutable scene geometry for every generated M0 runtime check.  A tile is
/// 10 by 10 world units, so this 2 by 2 Area spans `[0, 20]` on X and Y.
/// Keep the module entry at the centre and the sole fixture directly north of
/// it: this makes the first NWN camera view a deterministic model check rather
/// than a search for an object outside the initial view.
pub const M0_RUNTIME_TILESET_RESREF: &str = "tms01";
/// Exact tile sequence emitted by Aurora for the renderable r21 MicroSet
/// 2x2 Area, in serialized `Tile_List` order.
pub const M0_RUNTIME_TILES: [(i32, i32); 4] = [(12, 2), (12, 1), (12, 3), (12, 3)];
pub const M0_RUNTIME_TILE_ANIMATION_LOOP: u8 = 1;
pub const M0_RUNTIME_AREA_WIDTH: i32 = 2;
pub const M0_RUNTIME_AREA_HEIGHT: i32 = 2;
pub const M0_RUNTIME_ENTRY_X: f32 = 10.0;
pub const M0_RUNTIME_ENTRY_Y: f32 = 10.0;
pub const M0_RUNTIME_ENTRY_Z: f32 = 0.0;
pub const M0_RUNTIME_ENTRY_DIR_X: f32 = 0.0;
pub const M0_RUNTIME_ENTRY_DIR_Y: f32 = 1.0;
pub const M0_RUNTIME_FIXTURE_X: f32 = 10.0;
pub const M0_RUNTIME_FIXTURE_Y: f32 = 14.5;
pub const M0_RUNTIME_FIXTURE_Z: f32 = 0.0;

/// Caller-owned identities for an isolated binary M0 vertical slice.  The
/// default constants above remain the historical profile; a new runtime lane
/// must use fresh module/Area/HAK resrefs instead of overwriting it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryM0VerticalSliceIdentityV1 {
    pub module_resref: String,
    pub area_resref: String,
    pub hak_resref: String,
}

/// Position read back from a generated M0 runtime MOD.  It deliberately keeps
/// the IFO/GIT coordinates separate: a proof capture must bind both rather
/// than infer one from the other.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0RuntimePositionV1 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0RuntimeDirectionV1 {
    pub x: f32,
    pub y: f32,
}

/// The exact creature instance resolved from the binary M0 fixture's GIT.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryM0FixtureReadbackV1 {
    pub template_resref: String,
    pub appearance_row: u16,
    pub position: M0RuntimePositionV1,
    pub orientation: M0RuntimeDirectionV1,
}

/// Read-only, portable scene binding extracted from the generated MOD bytes.
/// It records no claim about a running Toolset or NWN client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryM0VerticalSliceReadbackV1 {
    pub schema_version: u32,
    pub module_resref: String,
    pub area_resref: String,
    pub ordered_hak_resrefs: Vec<String>,
    pub entry_position: M0RuntimePositionV1,
    pub entry_direction: M0RuntimeDirectionV1,
    pub fixture: BinaryM0FixtureReadbackV1,
}

/// Caller-owned MOD/Area/singleton-HAK identity for a generated creature
/// comparison scene.  This is deliberately independent from the historical
/// single-M0 identity so one diagnostic Area can contain several owned
/// creature fixtures without inventing another GFF writer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureModuleIdentityV1 {
    pub module_resref: String,
    pub area_resref: String,
    pub hak_resref: String,
}

/// One complete, caller-owned creature instance and its module-local UTC
/// blueprint identity. `id` is serialized as the instance/blueprint Tag;
/// `template_resref` is the exact UTC resource key.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureOwnedFixtureV1 {
    pub id: String,
    pub template_resref: String,
    pub display_name: String,
    pub appearance_row: u16,
    pub position: M0RuntimePositionV1,
    pub orientation: M0RuntimeDirectionV1,
}

/// Independent readback of the complete generated comparison scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureMultiFixtureModuleReadbackV1 {
    pub schema_version: u32,
    pub module_resref: String,
    pub area_resref: String,
    pub ordered_hak_resrefs: Vec<String>,
    pub entry_position: M0RuntimePositionV1,
    pub entry_direction: M0RuntimeDirectionV1,
    pub tileset_resref: String,
    pub area_width: i32,
    pub area_height: i32,
    pub tile_count: u32,
    pub fixtures: Vec<BinaryCreatureOwnedFixtureV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryCreatureMultiFixtureModuleArtifactV1 {
    pub payload: Vec<u8>,
    pub byte_length: u64,
    pub sha256: String,
    pub readback: BinaryCreatureMultiFixtureModuleReadbackV1,
}

/// Versioned semantic profiles used by the generated creature comparison
/// matrix.  `LegacyMinimal` preserves the frozen V1 output.  The two monster
/// baselines are independently authored from the Aurora load contract and
/// native-resource observations; they do not embed or copy a retail UTC.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryCreatureRuntimeProfileV2 {
    LegacyMinimal,
    PassiveMonsterBaseline,
    ActiveMonsterBaseline,
}

/// One V1 fixture identity paired with the exact runtime semantics that must
/// be emitted into both its GIT instance and its module-local UTC.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureProfiledFixtureV2 {
    pub fixture: BinaryCreatureOwnedFixtureV1,
    pub runtime_profile: BinaryCreatureRuntimeProfileV2,
}

/// V2 keeps the already-independent V1 scene readback and adds a separately
/// inferred profile for every fixture.  The profile list is not trusted build
/// input echoed into the result: the inspector classifies it from GIT and UTC
/// bytes and requires those two resources to agree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureProfileMatrixModuleReadbackV2 {
    pub schema_version: u32,
    pub scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub fixtures: Vec<BinaryCreatureProfiledFixtureV2>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryCreatureProfileMatrixModuleArtifactV2 {
    pub payload: Vec<u8>,
    pub byte_length: u64,
    pub sha256: String,
    pub readback: BinaryCreatureProfileMatrixModuleReadbackV2,
}

/// Native UTC equipment slots are bit-mask struct identifiers, not script
/// inventory indexes.  The values below were independently read back from
/// Aurora-authored UTC blueprints (`16` right hand, `32` left hand).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryCreatureHandSlotV1 {
    RightHand,
    LeftHand,
}

impl BinaryCreatureHandSlotV1 {
    pub const fn native_struct_id(self) -> u32 {
        match self {
            Self::RightHand => 16,
            Self::LeftHand => 32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureEquippedFixtureV1 {
    pub profiled_fixture: BinaryCreatureProfiledFixtureV2,
    pub hand: BinaryCreatureHandSlotV1,
    pub equipped_item_resref: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureWeaponItemV1 {
    pub resref: String,
    pub display_name: String,
    pub base_item: i32,
    pub model_parts: [u8; 3],
}

impl BinaryCreatureWeaponItemV1 {
    /// Clean-room longsword fixture based on the public UTI field contract.
    pub fn owned_longsword(resref: impl Into<String>) -> Self {
        Self {
            resref: resref.into(),
            display_name: "Meshy2Aurora attachment test longsword".to_owned(),
            base_item: 1,
            model_parts: [11, 11, 11],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureWeaponDemoReadbackV1 {
    pub schema_version: u32,
    pub scene: BinaryCreatureProfileMatrixModuleReadbackV2,
    pub weapon: BinaryCreatureWeaponItemV1,
    pub fixtures: Vec<BinaryCreatureEquippedFixtureV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryCreatureWeaponDemoArtifactV1 {
    pub payload: Vec<u8>,
    pub byte_length: u64,
    pub sha256: String,
    pub readback: BinaryCreatureWeaponDemoReadbackV1,
}

/// Stock short-sword blueprint present in the NWN:EE base resource index.
///
/// The V1 attachment demo generated its own syntactically valid UTI, but the
/// owner proof showed that Aurora did not resolve that resource into the hand
/// slot. V2 deliberately uses the same stock blueprint resref observed in a
/// native equipped UTC and present as resource type 2025 in `nwn_base.key`.
pub const NWN_BASE_SHORTSWORD_RESREF_V2: &str = "nw_wswss001";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinaryCreatureWeaponResourceScopeV2 {
    NwnBaseGame,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureStockWeaponV2 {
    pub resref: String,
    pub resource_type: u16,
    pub resource_scope: BinaryCreatureWeaponResourceScopeV2,
}

impl BinaryCreatureStockWeaponV2 {
    pub fn nwn_base_shortsword() -> Self {
        Self {
            resref: NWN_BASE_SHORTSWORD_RESREF_V2.to_owned(),
            resource_type: UTI_RESOURCE_TYPE,
            resource_scope: BinaryCreatureWeaponResourceScopeV2::NwnBaseGame,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryCreatureWeaponDemoReadbackV2 {
    pub schema_version: u32,
    pub scene: BinaryCreatureProfileMatrixModuleReadbackV2,
    pub weapon: BinaryCreatureStockWeaponV2,
    pub fixtures: Vec<BinaryCreatureEquippedFixtureV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryCreatureWeaponDemoArtifactV2 {
    pub payload: Vec<u8>,
    pub byte_length: u64,
    pub sha256: String,
    pub readback: BinaryCreatureWeaponDemoReadbackV2,
}

impl BinaryM0VerticalSliceIdentityV1 {
    pub fn historical_default() -> Self {
        Self {
            module_resref: BINARY_M0_MODULE_RESREF.to_owned(),
            area_resref: BINARY_M0_AREA_RESREF.to_owned(),
            hak_resref: BINARY_M0_PROOF_HAK_RESREF.to_owned(),
        }
    }
}

const IFO_RESOURCE_TYPE: u16 = 2014;
const ARE_RESOURCE_TYPE: u16 = 2012;
const GIC_RESOURCE_TYPE: u16 = 2046;
const GIT_RESOURCE_TYPE: u16 = 2023;
const UTC_RESOURCE_TYPE: u16 = 2027;
const UTI_RESOURCE_TYPE: u16 = 2025;
const FAC_RESOURCE_TYPE: u16 = 2038;
const BINARY_CREATURE_RUNTIME_MAX_HIT_POINTS: i16 = 13;
const BINARY_CREATURE_RUNTIME_SKILL_COUNT: usize = 28;

const AREA_INSTANCE_LISTS: [&str; 8] = [
    "Door List",
    "Encounter List",
    "List",
    "SoundList",
    "StoreList",
    "TriggerList",
    "WaypointList",
    "Placeable List",
];

const GIC_ROOT_LISTS: [&str; 9] = [
    "Creature List",
    "Door List",
    "Encounter List",
    "List",
    "SoundList",
    "StoreList",
    "TriggerList",
    "WaypointList",
    "Placeable List",
];

const GIT_ROOT_FIELDS: [&str; 10] = [
    "AreaProperties",
    "Creature List",
    "Door List",
    "Encounter List",
    "List",
    "SoundList",
    "StoreList",
    "TriggerList",
    "WaypointList",
    "Placeable List",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofModuleReportV2 {
    pub schema_version: u32,
    pub module_resref: String,
    pub module_display_name: String,
    pub area_resref: String,
    pub area_display_name: String,
    pub creature_resref: String,
    pub hak_resref: String,
    pub appearance_row: u16,
    pub resource_count: u32,
    pub byte_length: u64,
    pub sha256: String,
    pub semantic_readback_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_m0_runtime_fixture: Option<BinaryM0VerticalSliceReadbackV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProofModuleArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ProofModuleReportV2,
}

#[deprecated(note = "use ProofModuleReportV2; the serialized report schema is version 2")]
pub type ProofModuleReportV1 = ProofModuleReportV2;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofModuleErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ProofModuleErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ProofModuleErrorV1 {}

/// Builds a clean-room `.mod` containing a proof creature and an in-module
/// placement.  The HAK itself remains a separately downloadable package; the
/// IFO records its lower-case HAK resref exactly as Aurora's `Mod_HakList`
/// contract requires.
pub fn build_creature_proof_module_v1(
    appearance_row: u16,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    build_canonical_creature_proof_module_v1(
        appearance_row,
        PROOF_CREATURE_RESREF,
        "Codex Meshy H1 animation proof creature",
        "Codex H1 animation proof area",
        "Generated by Codex for Meshy2Aurora H1 animation proof.",
    )
}

/// Builds a single-fixture proof module with one explicit runtime-complete
/// creature profile applied identically to the GIT instance and module-local
/// UTC. The fixture is the only creature in the Area and is placed directly in
/// front of the player entry position.
pub fn build_single_profiled_creature_proof_module_v2(
    appearance_row: u16,
    runtime_profile: BinaryCreatureRuntimeProfileV2,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    let identity = BinaryCreatureModuleIdentityV1 {
        module_resref: PROOF_MODULE_RESREF.to_owned(),
        area_resref: PROOF_AREA_RESREF.to_owned(),
        hak_resref: PROOF_HAK_RESREF.to_owned(),
    };
    build_single_profiled_creature_proof_module_with_identity_v3(
        appearance_row,
        runtime_profile,
        &identity,
        PROOF_CREATURE_RESREF,
    )
}

/// Builds the production single-creature proof scene under one exact
/// caller-owned MOD/Area/HAK/UTC identity. This is the collision-free
/// counterpart of the historical V2 helper: no runtime or proof claim is
/// made, but every resource name is read back from the emitted MOD bytes.
pub fn build_single_profiled_creature_proof_module_with_identity_v3(
    appearance_row: u16,
    runtime_profile: BinaryCreatureRuntimeProfileV2,
    identity: &BinaryCreatureModuleIdentityV1,
    creature_resref: &str,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora procedural humanoid proof";
    const AREA_DISPLAY_NAME: &str = "Meshy2Aurora procedural humanoid proof area";
    let fixture = BinaryCreatureProfiledFixtureV2 {
        fixture: BinaryCreatureOwnedFixtureV1 {
            id: "m2a_procedural_creature".to_owned(),
            template_resref: creature_resref.to_owned(),
            display_name: "Meshy procedural humanoid".to_owned(),
            appearance_row,
            position: M0RuntimePositionV1 {
                x: M0_RUNTIME_FIXTURE_X,
                y: M0_RUNTIME_FIXTURE_Y,
                z: M0_RUNTIME_FIXTURE_Z,
            },
            orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
        },
        runtime_profile,
    };
    let fixtures = [fixture];
    let artifact = build_binary_creature_profile_matrix_module_named_v2(
        identity,
        &fixtures,
        MODULE_DISPLAY_NAME,
        AREA_DISPLAY_NAME,
        "One owned procedural humanoid fixture with an explicit active monster runtime profile.",
    )?;
    if artifact.readback.fixtures != fixtures {
        return Err(error(
            "M6-PROFILED-PROOF-MODULE-SEMANTIC-DIFF",
            "module",
            "single-fixture proof module differs from its exact runtime profile",
        ));
    }
    Ok(ProofModuleArtifactV1 {
        report: ProofModuleReportV2 {
            schema_version: 2,
            module_resref: identity.module_resref.clone(),
            module_display_name: MODULE_DISPLAY_NAME.to_owned(),
            area_resref: identity.area_resref.clone(),
            area_display_name: AREA_DISPLAY_NAME.to_owned(),
            creature_resref: creature_resref.to_owned(),
            hak_resref: identity.hak_resref.clone(),
            appearance_row,
            resource_count: (5 + fixtures.len()) as u32,
            byte_length: artifact.byte_length,
            sha256: artifact.sha256,
            semantic_readback_status: "PASS".to_owned(),
            binary_m0_runtime_fixture: None,
        },
        payload: artifact.payload,
    })
}

/// Builds the one canonical proof module with the recovered static Meshy M0
/// fixture.  It deliberately keeps the canonical module, Area and single HAK
/// names, so M0 cannot create a parallel Toolset/NWN workflow.
pub fn build_canonical_m0_creature_proof_module_v1(
    appearance_row: u16,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    build_canonical_creature_proof_module_v1(
        appearance_row,
        M0_CANONICAL_PROOF_CREATURE_RESREF,
        "Meshy M0 static rigid proof creature",
        "Meshy M0 static proof area",
        "Generated by Meshy2Aurora from the recovered Meshy M0 asset.",
    )
}

/// Materializes the new single-HAK M0 vertical-slice module as an own binary
/// ERF/GFF artifact. It uses the fixed M0 runtime fixture: a `tms01` 2x2 Area
/// with the exact ordered native tile sequence `(12,2)`, `(12,1)`, `(12,3)`,
/// `(12,3)` and animation loops set to `1`, entry `[10, 10, 0]` facing north,
/// and exactly one Dwarf Mercenary fixture at `[10, 14.5, 0]`. The sequence is
/// taken from the fresh Aurora-created r21 Area that visibly rendered M0; the
/// caller still must pass the central binary-bootstrap structural gate and
/// later native Toolset geometry gates.
pub fn build_binary_m0_vertical_slice_module_v1(
    appearance_row: u16,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    build_binary_m0_vertical_slice_module_with_identity_v1(
        appearance_row,
        &BinaryM0VerticalSliceIdentityV1::historical_default(),
    )
}

/// Materializes the same owned `tms01` geometry under caller-supplied fresh
/// resrefs. This prevents a newer Meshy model from replacing an earlier proof
/// MOD or HAK solely to obtain an independent runtime profile.
pub fn build_binary_m0_vertical_slice_module_with_identity_v1(
    appearance_row: u16,
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    validate_binary_m0_vertical_slice_identity(identity)?;
    let creature_display_name = "Meshy M0 binary vertical-slice fixture";
    let resources = vec![
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            binary_m0_module_ifo_for(
                &identity.module_resref,
                &identity.area_resref,
                &[identity.hak_resref.as_str()],
                "Meshy2Aurora M0 binary vertical slice",
                "Generated by Meshy2Aurora for the M0 binary vertical-slice proof.",
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            &identity.area_resref,
            ARE_RESOURCE_TYPE,
            binary_m0_area_for(&identity.area_resref)?,
        ),
        resource(
            &identity.area_resref,
            GIC_RESOURCE_TYPE,
            proof_gic_for("Generated by Meshy2Aurora: M0 binary vertical-slice fixture.")?,
        ),
        resource(
            &identity.area_resref,
            GIT_RESOURCE_TYPE,
            binary_m0_git_for(appearance_row, creature_display_name)?,
        ),
        resource(
            BINARY_M0_CREATURE_TEMPLATE_RESREF,
            UTC_RESOURCE_TYPE,
            proof_utc_for(
                appearance_row,
                BINARY_M0_CREATURE_TEMPLATE_RESREF,
                creature_display_name,
            )?,
        ),
    ];
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M0-BINARY-VERTICAL-SLICE-MODULE-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    validate_binary_m0_vertical_slice_module_readback(&archive.payload, appearance_row, identity)?;
    let binary_m0_runtime_fixture = inspect_binary_m0_vertical_slice_module_v1(&archive.payload)?;
    Ok(ProofModuleArtifactV1 {
        report: ProofModuleReportV2 {
            schema_version: 2,
            module_resref: identity.module_resref.clone(),
            module_display_name: "Meshy2Aurora M0 binary vertical slice".to_owned(),
            area_resref: identity.area_resref.clone(),
            area_display_name: "Meshy2Aurora M0 binary vertical-slice area".to_owned(),
            creature_resref: BINARY_M0_CREATURE_TEMPLATE_RESREF.to_owned(),
            hak_resref: identity.hak_resref.clone(),
            appearance_row,
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            sha256: sha256(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            binary_m0_runtime_fixture: Some(binary_m0_runtime_fixture),
        },
        payload: archive.payload,
    })
}

/// Builds one deterministic 2x2 creature comparison Area with a fixed player
/// entry at `[10, 10, 0]` facing +Y, one singleton HAK binding, and an ordered
/// list of caller-owned creature fixtures.  Every fixture receives its own UTC
/// resource; no retail blueprint payload is copied into the MOD.
pub fn build_binary_creature_multi_fixture_module_v1(
    identity: &BinaryCreatureModuleIdentityV1,
    fixtures: &[BinaryCreatureOwnedFixtureV1],
) -> Result<BinaryCreatureMultiFixtureModuleArtifactV1, ProofModuleErrorV1> {
    validate_binary_creature_multi_fixture_input(identity, fixtures)?;
    let mut resources = Vec::with_capacity(5 + fixtures.len());
    resources.extend([
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            binary_m0_module_ifo_for(
                &identity.module_resref,
                &identity.area_resref,
                &[identity.hak_resref.as_str()],
                "Meshy2Aurora creature comparison",
                "Generated by Meshy2Aurora as an owned multi-fixture diagnostic scene.",
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            &identity.area_resref,
            ARE_RESOURCE_TYPE,
            binary_m0_area_for(&identity.area_resref)?,
        ),
        resource(
            &identity.area_resref,
            GIC_RESOURCE_TYPE,
            binary_creature_multi_fixture_gic(fixtures)?,
        ),
        resource(
            &identity.area_resref,
            GIT_RESOURCE_TYPE,
            binary_creature_multi_fixture_git(fixtures)?,
        ),
    ]);
    for fixture in fixtures {
        resources.push(resource(
            &fixture.template_resref,
            UTC_RESOURCE_TYPE,
            binary_creature_owned_fixture_utc(fixture)?,
        ));
    }
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M0-BINARY-MULTI-FIXTURE-MODULE-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    let readback = inspect_binary_creature_multi_fixture_module_v1(&archive.payload)?;
    if readback.module_resref != identity.module_resref
        || readback.area_resref != identity.area_resref
        || readback.ordered_hak_resrefs != [identity.hak_resref.clone()]
        || readback.fixtures != fixtures
    {
        return Err(error(
            "M0-BINARY-MULTI-FIXTURE-SEMANTIC-DIFF",
            "module",
            "generated module differs from caller-owned identity or fixture order",
        ));
    }
    Ok(BinaryCreatureMultiFixtureModuleArtifactV1 {
        byte_length: archive.payload.len() as u64,
        sha256: sha256(&archive.payload),
        payload: archive.payload,
        readback,
    })
}

/// Builds a comparison scene whose runtime-semantic profile is explicit for
/// every fixture.  This is an offline builder only: it neither materializes a
/// proof candidate nor reads or mutates a live Toolset/NWN session.
pub fn build_binary_creature_profile_matrix_module_v2(
    identity: &BinaryCreatureModuleIdentityV1,
    fixtures: &[BinaryCreatureProfiledFixtureV2],
) -> Result<BinaryCreatureProfileMatrixModuleArtifactV2, ProofModuleErrorV1> {
    build_binary_creature_profile_matrix_module_named_v2(
        identity,
        fixtures,
        "Meshy2Aurora creature runtime-profile comparison",
        "Meshy2Aurora creature comparison area",
        "Generated by Meshy2Aurora as an owned profile-matrix diagnostic scene.",
    )
}

fn build_binary_creature_profile_matrix_module_named_v2(
    identity: &BinaryCreatureModuleIdentityV1,
    fixtures: &[BinaryCreatureProfiledFixtureV2],
    module_display_name: &str,
    area_display_name: &str,
    module_description: &str,
) -> Result<BinaryCreatureProfileMatrixModuleArtifactV2, ProofModuleErrorV1> {
    let owned_fixtures = fixtures
        .iter()
        .map(|fixture| fixture.fixture.clone())
        .collect::<Vec<_>>();
    validate_binary_creature_multi_fixture_input(identity, &owned_fixtures)?;

    let mut resources = Vec::with_capacity(5 + fixtures.len());
    resources.extend([
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            binary_m0_module_ifo_for(
                &identity.module_resref,
                &identity.area_resref,
                &[identity.hak_resref.as_str()],
                module_display_name,
                module_description,
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            &identity.area_resref,
            ARE_RESOURCE_TYPE,
            binary_m0_area_for_named(&identity.area_resref, area_display_name)?,
        ),
        resource(
            &identity.area_resref,
            GIC_RESOURCE_TYPE,
            binary_creature_multi_fixture_gic(&owned_fixtures)?,
        ),
        resource(
            &identity.area_resref,
            GIT_RESOURCE_TYPE,
            binary_creature_profile_matrix_git(fixtures)?,
        ),
    ]);
    for fixture in fixtures {
        resources.push(resource(
            &fixture.fixture.template_resref,
            UTC_RESOURCE_TYPE,
            binary_creature_profiled_fixture_utc(fixture)?,
        ));
    }

    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M0-BINARY-PROFILE-MATRIX-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    let readback = inspect_binary_creature_profile_matrix_module_v2(&archive.payload)?;
    if readback.fixtures != fixtures {
        return Err(error(
            "M0-BINARY-PROFILE-MATRIX-SEMANTIC-DIFF",
            "module",
            "generated module differs from caller-owned fixture/profile order",
        ));
    }
    Ok(BinaryCreatureProfileMatrixModuleArtifactV2 {
        byte_length: archive.payload.len() as u64,
        sha256: sha256(&archive.payload),
        payload: archive.payload,
        readback,
    })
}

/// Builds a clean-room right/left-hand attachment demo.  Both the placed GIT
/// instances and their module-local UTC blueprints own the same non-empty
/// `Equip_ItemList`, while one generated UTI supplies the visible weapon.
pub fn build_binary_creature_weapon_demo_module_v1(
    identity: &BinaryCreatureModuleIdentityV1,
    fixtures: &[BinaryCreatureEquippedFixtureV1],
    weapon: &BinaryCreatureWeaponItemV1,
) -> Result<BinaryCreatureWeaponDemoArtifactV1, ProofModuleErrorV1> {
    let profiled = fixtures
        .iter()
        .map(|fixture| fixture.profiled_fixture.clone())
        .collect::<Vec<_>>();
    let owned = profiled
        .iter()
        .map(|fixture| fixture.fixture.clone())
        .collect::<Vec<_>>();
    validate_binary_creature_multi_fixture_input(identity, &owned)?;
    validate_binary_creature_weapon_demo_input(fixtures, weapon)?;

    let mut resources = Vec::with_capacity(6 + fixtures.len());
    resources.extend([
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            binary_m0_module_ifo_for(
                &identity.module_resref,
                &identity.area_resref,
                &[identity.hak_resref.as_str()],
                "Meshy2Aurora Creature Weapon Anchors V1",
                "Generated by Meshy2Aurora to test right- and left-hand item attachment anchors.",
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            &identity.area_resref,
            ARE_RESOURCE_TYPE,
            binary_m0_area_for_named(&identity.area_resref, "Meshy2Aurora Creature Weapon Test")?,
        ),
        resource(
            &identity.area_resref,
            GIC_RESOURCE_TYPE,
            binary_creature_multi_fixture_gic(&owned)?,
        ),
        resource(
            &identity.area_resref,
            GIT_RESOURCE_TYPE,
            binary_creature_weapon_demo_git(fixtures)?,
        ),
        resource(
            &weapon.resref,
            UTI_RESOURCE_TYPE,
            binary_creature_weapon_uti(weapon)?,
        ),
    ]);
    for fixture in fixtures {
        resources.push(resource(
            &fixture.profiled_fixture.fixture.template_resref,
            UTC_RESOURCE_TYPE,
            binary_creature_equipped_fixture_utc(fixture)?,
        ));
    }
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M0-BINARY-WEAPON-DEMO-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    let readback = inspect_binary_creature_weapon_demo_module_v1(&archive.payload)?;
    if readback.weapon != *weapon || readback.fixtures != fixtures {
        return Err(error(
            "M0-BINARY-WEAPON-DEMO-SEMANTIC-DIFF",
            "module",
            "weapon or equipped fixture readback differs from authored input",
        ));
    }
    Ok(BinaryCreatureWeaponDemoArtifactV1 {
        byte_length: archive.payload.len() as u64,
        sha256: sha256(&archive.payload),
        payload: archive.payload,
        readback,
    })
}

/// Builds the corrected weapon-attachment demo around one verified NWN base
/// item. Unlike V1, this module does not pretend that a module-local generated
/// UTI is native-resolved merely because our own GFF parser can read it.
pub fn build_binary_creature_stock_weapon_demo_module_v2(
    identity: &BinaryCreatureModuleIdentityV1,
    fixtures: &[BinaryCreatureEquippedFixtureV1],
    weapon: &BinaryCreatureStockWeaponV2,
) -> Result<BinaryCreatureWeaponDemoArtifactV2, ProofModuleErrorV1> {
    let profiled = fixtures
        .iter()
        .map(|fixture| fixture.profiled_fixture.clone())
        .collect::<Vec<_>>();
    let owned = profiled
        .iter()
        .map(|fixture| fixture.fixture.clone())
        .collect::<Vec<_>>();
    validate_binary_creature_multi_fixture_input(identity, &owned)?;
    validate_binary_creature_stock_weapon_demo_input_v2(fixtures, weapon)?;

    let mut resources = Vec::with_capacity(5 + fixtures.len());
    resources.extend([
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            binary_m0_module_ifo_for(
                &identity.module_resref,
                &identity.area_resref,
                &[identity.hak_resref.as_str()],
                "Meshy2Aurora Creature Weapon Attachment V2",
                "Generated by Meshy2Aurora to test a native-resolved stock weapon on corrected Creature attachment hooks.",
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            &identity.area_resref,
            ARE_RESOURCE_TYPE,
            binary_m0_area_for_named(
                &identity.area_resref,
                "Meshy2Aurora Creature Weapon Test V2",
            )?,
        ),
        resource(
            &identity.area_resref,
            GIC_RESOURCE_TYPE,
            binary_creature_multi_fixture_gic(&owned)?,
        ),
        resource(
            &identity.area_resref,
            GIT_RESOURCE_TYPE,
            binary_creature_weapon_demo_git(fixtures)?,
        ),
    ]);
    for fixture in fixtures {
        resources.push(resource(
            &fixture.profiled_fixture.fixture.template_resref,
            UTC_RESOURCE_TYPE,
            binary_creature_equipped_fixture_utc(fixture)?,
        ));
    }
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M0-BINARY-WEAPON-DEMO-V2-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    let readback = inspect_binary_creature_stock_weapon_demo_module_v2(&archive.payload, weapon)?;
    if readback.weapon != *weapon || readback.fixtures != fixtures {
        return Err(error(
            "M0-BINARY-WEAPON-DEMO-V2-SEMANTIC-DIFF",
            "module",
            "stock weapon or equipped fixture readback differs from authored input",
        ));
    }
    Ok(BinaryCreatureWeaponDemoArtifactV2 {
        byte_length: archive.payload.len() as u64,
        sha256: sha256(&archive.payload),
        payload: archive.payload,
        readback,
    })
}

fn validate_binary_creature_stock_weapon_demo_input_v2(
    fixtures: &[BinaryCreatureEquippedFixtureV1],
    weapon: &BinaryCreatureStockWeaponV2,
) -> Result<(), ProofModuleErrorV1> {
    validate_binary_creature_stock_weapon_identity_v2(weapon)?;
    if fixtures.is_empty()
        || fixtures
            .iter()
            .any(|fixture| fixture.equipped_item_resref != weapon.resref)
    {
        return Err(binary_creature_input_error(
            "fixtures.equippedItemResref",
            "every V2 fixture must reference the exact verified NWN base weapon",
        ));
    }
    Ok(())
}

fn validate_binary_creature_stock_weapon_identity_v2(
    weapon: &BinaryCreatureStockWeaponV2,
) -> Result<(), ProofModuleErrorV1> {
    if weapon != &BinaryCreatureStockWeaponV2::nwn_base_shortsword() {
        return Err(binary_creature_input_error(
            "weapon",
            "V2 proof accepts only the exact NWN base short-sword resource nw_wswss001 (UTI 2025)",
        ));
    }
    Ok(())
}

fn validate_binary_creature_weapon_demo_input(
    fixtures: &[BinaryCreatureEquippedFixtureV1],
    weapon: &BinaryCreatureWeaponItemV1,
) -> Result<(), ProofModuleErrorV1> {
    if !is_owned_resref(&weapon.resref) {
        return Err(binary_creature_input_error(
            "weapon.resref",
            "weapon resref must contain 1..16 lowercase ASCII letters, digits, or underscores",
        ));
    }
    if weapon.display_name.is_empty()
        || weapon.display_name.len() > 128
        || weapon.display_name.chars().any(char::is_control)
    {
        return Err(binary_creature_input_error(
            "weapon.displayName",
            "weapon display name must contain 1..128 bytes without control characters",
        ));
    }
    if weapon.base_item < 0 || weapon.model_parts.contains(&0) {
        return Err(binary_creature_input_error(
            "weapon",
            "weapon base item must be non-negative and all model parts must be non-zero",
        ));
    }
    if fixtures.is_empty()
        || fixtures
            .iter()
            .any(|fixture| fixture.equipped_item_resref != weapon.resref)
    {
        return Err(binary_creature_input_error(
            "fixtures.equippedItemResref",
            "every demo fixture must reference the generated weapon UTI",
        ));
    }
    Ok(())
}

fn validate_binary_creature_multi_fixture_input(
    identity: &BinaryCreatureModuleIdentityV1,
    fixtures: &[BinaryCreatureOwnedFixtureV1],
) -> Result<(), ProofModuleErrorV1> {
    for (path, value) in [
        ("identity.moduleResref", identity.module_resref.as_str()),
        ("identity.areaResref", identity.area_resref.as_str()),
        ("identity.hakResref", identity.hak_resref.as_str()),
    ] {
        if !is_owned_resref(value) {
            return Err(binary_creature_input_error(
                path,
                "resref must contain 1..16 lowercase ASCII letters, digits, or underscores",
            ));
        }
    }
    if fixtures.is_empty() || fixtures.len() > 64 {
        return Err(binary_creature_input_error(
            "fixtures",
            "fixture list must contain 1..64 entries",
        ));
    }
    let mut ids = HashSet::with_capacity(fixtures.len());
    let mut templates = HashSet::with_capacity(fixtures.len());
    for (index, fixture) in fixtures.iter().enumerate() {
        if fixture.id.is_empty()
            || fixture.id.len() > 32
            || !fixture
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].id"),
                "fixture id must contain 1..32 lowercase ASCII letters, digits, or underscores",
            ));
        }
        if !ids.insert(fixture.id.as_str()) {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].id"),
                "fixture id must be unique",
            ));
        }
        if !is_owned_resref(&fixture.template_resref) {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].templateResref"),
                "template resref must contain 1..16 lowercase ASCII letters, digits, or underscores",
            ));
        }
        if !templates.insert(fixture.template_resref.as_str()) {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].templateResref"),
                "template resref must be unique",
            ));
        }
        if fixture.display_name.is_empty()
            || fixture.display_name.len() > 128
            || fixture.display_name.chars().any(char::is_control)
        {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].displayName"),
                "display name must contain 1..128 UTF-8 bytes without control characters",
            ));
        }
        if !fixture.position.x.is_finite()
            || !fixture.position.y.is_finite()
            || !fixture.position.z.is_finite()
            || !(0.0..=20.0).contains(&fixture.position.x)
            || !(0.0..=20.0).contains(&fixture.position.y)
        {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].position"),
                "fixture position must be finite and X/Y must remain inside the 2x2 Area",
            ));
        }
        if !fixture.orientation.x.is_finite()
            || !fixture.orientation.y.is_finite()
            || (fixture.orientation.x == 0.0 && fixture.orientation.y == 0.0)
        {
            return Err(binary_creature_input_error(
                format!("fixtures[{index}].orientation"),
                "fixture orientation must be a finite non-zero 2D vector",
            ));
        }
    }
    Ok(())
}

fn is_owned_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn binary_creature_input_error(
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProofModuleErrorV1 {
    error("M0-BINARY-MULTI-FIXTURE-INPUT-INVALID", path, message)
}

fn binary_creature_readback_error(
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProofModuleErrorV1 {
    error("M0-BINARY-MULTI-FIXTURE-READBACK-INVALID", path, message)
}

fn binary_creature_profile_readback_error(
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProofModuleErrorV1 {
    error("M0-BINARY-PROFILE-MATRIX-READBACK-INVALID", path, message)
}

fn binary_creature_field<'a>(
    fields: &'a [GffFieldV1],
    label: &str,
    path: &str,
) -> Result<&'a GffValueV1, ProofModuleErrorV1> {
    fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
        .ok_or_else(|| binary_creature_readback_error(path, format!("missing {label}")))
}

fn validate_binary_creature_runtime_complete_envelope(
    fields: &[GffFieldV1],
    path: &str,
) -> Result<(), ProofModuleErrorV1> {
    let phenotype_path = format!("{path}.Phenotype");
    if !matches!(
        binary_creature_field(fields, "Phenotype", &phenotype_path)?,
        GffValueV1::Int(0)
    ) {
        return Err(binary_creature_readback_error(
            phenotype_path,
            "direct whole-model creature Phenotype must be an explicit INT 0",
        ));
    }

    let max_hit_points_path = format!("{path}.MaxHitPoints");
    if !matches!(
        binary_creature_field(fields, "MaxHitPoints", &max_hit_points_path)?,
        GffValueV1::Short(value) if *value > 0
    ) {
        return Err(binary_creature_readback_error(
            max_hit_points_path,
            "runtime creature MaxHitPoints must be a positive SHORT",
        ));
    }

    let skill_list_path = format!("{path}.SkillList");
    let valid_skill_list = matches!(
        binary_creature_field(fields, "SkillList", &skill_list_path)?,
        GffValueV1::List(skills)
            if skills.iter().all(|skill|
                skill.struct_id == 0
                    && matches!(
                        skill.fields.as_slice(),
                        [GffFieldV1 {
                            label,
                            value: GffValueV1::Byte(_),
                        }] if label == "Rank"
                    )
            )
    );
    if !valid_skill_list {
        return Err(binary_creature_readback_error(
            skill_list_path,
            "runtime creature SkillList entries must be struct 0 with one BYTE Rank field",
        ));
    }
    Ok(())
}

fn binary_creature_string(value: &GffValueV1, path: &str) -> Result<String, ProofModuleErrorV1> {
    match value {
        GffValueV1::String(value) => String::from_utf8(value.clone())
            .map_err(|_| binary_creature_readback_error(path, "expected a valid UTF-8 string")),
        _ => Err(binary_creature_readback_error(path, "expected string")),
    }
}

fn binary_creature_resref(value: &GffValueV1, path: &str) -> Result<String, ProofModuleErrorV1> {
    match value {
        GffValueV1::ResRef(value) => Ok(value.clone()),
        _ => Err(binary_creature_readback_error(path, "expected resref")),
    }
}

fn binary_creature_loc_string(
    value: &GffValueV1,
    path: &str,
) -> Result<String, ProofModuleErrorV1> {
    match value {
        GffValueV1::LocString(value)
            if value.string_ref == u32::MAX
                && value.substrings.len() == 1
                && value.substrings[0].string_id == 0 =>
        {
            String::from_utf8(value.substrings[0].bytes.clone()).map_err(|_| {
                binary_creature_readback_error(path, "expected a valid UTF-8 locstring")
            })
        }
        _ => Err(binary_creature_readback_error(
            path,
            "expected one owned inline locstring",
        )),
    }
}

fn binary_creature_float(value: &GffValueV1, path: &str) -> Result<f32, ProofModuleErrorV1> {
    match value {
        GffValueV1::Float(value) if value.is_finite() => Ok(*value),
        _ => Err(binary_creature_readback_error(
            path,
            "expected finite float",
        )),
    }
}

fn binary_creature_word(value: &GffValueV1, path: &str) -> Result<u16, ProofModuleErrorV1> {
    match value {
        GffValueV1::Word(value) => Ok(*value),
        _ => Err(binary_creature_readback_error(path, "expected word")),
    }
}

fn binary_creature_int(value: &GffValueV1, path: &str) -> Result<i32, ProofModuleErrorV1> {
    match value {
        GffValueV1::Int(value) => Ok(*value),
        _ => Err(binary_creature_readback_error(path, "expected int")),
    }
}

/// Reads the scene binding from a binary M0 vertical-slice MOD.  This is
/// intentionally independent of the caller's requested identity and
/// appearance row: consumers can bind a future Toolset or NWN capture to the
/// bytes that were actually emitted.
fn binary_m0_readback_field<'a>(
    fields: &'a [GffFieldV1],
    label: &str,
    path: &str,
) -> Result<&'a GffValueV1, ProofModuleErrorV1> {
    fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
        .ok_or_else(|| {
            error(
                "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
                path,
                format!("missing {label}"),
            )
        })
}

pub fn inspect_binary_m0_vertical_slice_module_v1(
    bytes: &[u8],
) -> Result<BinaryM0VerticalSliceReadbackV1, ProofModuleErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    if archive.file_type() != ErfFileType::Module {
        return Err(error(
            "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
            "module.signature",
            "expected MOD V1.0",
        ));
    }
    let ifo = read_gff_v32(
        archive
            .find("module", IFO_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "module.ifo", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "module.ifo", value.message))?;
    let field_value = binary_m0_readback_field;
    let string_value = |value: &GffValueV1, path: &str| match value {
        GffValueV1::String(value) => String::from_utf8(value.clone()).map_err(|_| {
            error(
                "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
                path,
                "expected UTF-8 string",
            )
        }),
        _ => Err(error(
            "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
            path,
            "expected string",
        )),
    };
    let resref_value = |value: &GffValueV1, path: &str| match value {
        GffValueV1::ResRef(value) => Ok(value.clone()),
        _ => Err(error(
            "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
            path,
            "expected resref",
        )),
    };
    let float_value = |value: &GffValueV1, path: &str| match value {
        GffValueV1::Float(value) if value.is_finite() => Ok(*value),
        _ => Err(error(
            "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
            path,
            "expected finite float",
        )),
    };
    let word_value = |value: &GffValueV1, path: &str| match value {
        GffValueV1::Word(value) => Ok(*value),
        _ => Err(error(
            "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
            path,
            "expected word",
        )),
    };

    let module_resref = string_value(
        field_value(&ifo.root.fields, "Mod_Tag", "module.ifo.Mod_Tag")?,
        "module.ifo.Mod_Tag",
    )?;
    let area_resref = resref_value(
        field_value(
            &ifo.root.fields,
            "Mod_Entry_Area",
            "module.ifo.Mod_Entry_Area",
        )?,
        "module.ifo.Mod_Entry_Area",
    )?;
    let entry_position = M0RuntimePositionV1 {
        x: float_value(
            field_value(&ifo.root.fields, "Mod_Entry_X", "module.ifo.Mod_Entry_X")?,
            "module.ifo.Mod_Entry_X",
        )?,
        y: float_value(
            field_value(&ifo.root.fields, "Mod_Entry_Y", "module.ifo.Mod_Entry_Y")?,
            "module.ifo.Mod_Entry_Y",
        )?,
        z: float_value(
            field_value(&ifo.root.fields, "Mod_Entry_Z", "module.ifo.Mod_Entry_Z")?,
            "module.ifo.Mod_Entry_Z",
        )?,
    };
    let entry_direction = M0RuntimeDirectionV1 {
        x: float_value(
            field_value(
                &ifo.root.fields,
                "Mod_Entry_Dir_X",
                "module.ifo.Mod_Entry_Dir_X",
            )?,
            "module.ifo.Mod_Entry_Dir_X",
        )?,
        y: float_value(
            field_value(
                &ifo.root.fields,
                "Mod_Entry_Dir_Y",
                "module.ifo.Mod_Entry_Dir_Y",
            )?,
            "module.ifo.Mod_Entry_Dir_Y",
        )?,
    };
    let ordered_hak_resrefs =
        match field_value(&ifo.root.fields, "Mod_HakList", "module.ifo.Mod_HakList")? {
            GffValueV1::List(entries) => entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let path = format!("module.ifo.Mod_HakList[{index}].Mod_Hak");
                    string_value(field_value(&entry.fields, "Mod_Hak", &path)?, &path)
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => {
                return Err(error(
                    "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
                    "module.ifo.Mod_HakList",
                    "expected HAK list",
                ));
            }
        };

    let git = read_gff_v32(
        archive
            .find(&area_resref, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.git", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.git", value.message))?;
    let creatures = match field_value(&git.root.fields, "Creature List", "area.git.Creature List")?
    {
        GffValueV1::List(values) if values.len() == 1 => values,
        _ => {
            return Err(error(
                "M0-BINARY-VERTICAL-SLICE-READBACK-INVALID",
                "area.git.Creature List",
                "expected exactly one creature fixture",
            ));
        }
    };
    let creature = &creatures[0];
    let fixture = BinaryM0FixtureReadbackV1 {
        template_resref: resref_value(
            field_value(
                &creature.fields,
                "TemplateResRef",
                "area.git.Creature List[0].TemplateResRef",
            )?,
            "area.git.Creature List[0].TemplateResRef",
        )?,
        appearance_row: word_value(
            field_value(
                &creature.fields,
                "Appearance_Type",
                "area.git.Creature List[0].Appearance_Type",
            )?,
            "area.git.Creature List[0].Appearance_Type",
        )?,
        position: M0RuntimePositionV1 {
            x: float_value(
                field_value(
                    &creature.fields,
                    "XPosition",
                    "area.git.Creature List[0].XPosition",
                )?,
                "area.git.Creature List[0].XPosition",
            )?,
            y: float_value(
                field_value(
                    &creature.fields,
                    "YPosition",
                    "area.git.Creature List[0].YPosition",
                )?,
                "area.git.Creature List[0].YPosition",
            )?,
            z: float_value(
                field_value(
                    &creature.fields,
                    "ZPosition",
                    "area.git.Creature List[0].ZPosition",
                )?,
                "area.git.Creature List[0].ZPosition",
            )?,
        },
        orientation: M0RuntimeDirectionV1 {
            x: float_value(
                field_value(
                    &creature.fields,
                    "XOrientation",
                    "area.git.Creature List[0].XOrientation",
                )?,
                "area.git.Creature List[0].XOrientation",
            )?,
            y: float_value(
                field_value(
                    &creature.fields,
                    "YOrientation",
                    "area.git.Creature List[0].YOrientation",
                )?,
                "area.git.Creature List[0].YOrientation",
            )?,
        },
    };
    Ok(BinaryM0VerticalSliceReadbackV1 {
        schema_version: 1,
        module_resref,
        area_resref,
        ordered_hak_resrefs,
        entry_position,
        entry_direction,
        fixture,
    })
}

/// Re-reads every binding from a generated multi-fixture MOD.  The readback is
/// independent from the caller's input and rejects extra resources, a
/// non-singleton HAK list, non-canonical Area geometry, ambiguous fixture
/// identities, or a UTC that disagrees with its GIT instance.
pub fn inspect_binary_creature_multi_fixture_module_v1(
    bytes: &[u8],
) -> Result<BinaryCreatureMultiFixtureModuleReadbackV1, ProofModuleErrorV1> {
    inspect_binary_creature_multi_fixture_module_with_extras_v1(bytes, &[])
}

fn inspect_binary_creature_multi_fixture_module_with_extras_v1(
    bytes: &[u8],
    allowed_extra_resources: &[(String, u16)],
) -> Result<BinaryCreatureMultiFixtureModuleReadbackV1, ProofModuleErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    if archive.file_type() != ErfFileType::Module {
        return Err(binary_creature_readback_error(
            "module.signature",
            "expected MOD V1.0",
        ));
    }
    let ifo = read_gff_v32(
        archive
            .find("module", IFO_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "module.ifo", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "module.ifo", value.message))?;
    if ifo.file_type != GffFileTypeV1::Ifo {
        return Err(binary_creature_readback_error(
            "module.ifo.fileType",
            "module IFO resource must contain an exact IFO GFF",
        ));
    }
    let module_resref = binary_creature_string(
        binary_creature_field(&ifo.root.fields, "Mod_Tag", "module.ifo.Mod_Tag")?,
        "module.ifo.Mod_Tag",
    )?;
    let area_resref = binary_creature_resref(
        binary_creature_field(
            &ifo.root.fields,
            "Mod_Entry_Area",
            "module.ifo.Mod_Entry_Area",
        )?,
        "module.ifo.Mod_Entry_Area",
    )?;
    let entry_position = M0RuntimePositionV1 {
        x: binary_creature_float(
            binary_creature_field(&ifo.root.fields, "Mod_Entry_X", "module.ifo.Mod_Entry_X")?,
            "module.ifo.Mod_Entry_X",
        )?,
        y: binary_creature_float(
            binary_creature_field(&ifo.root.fields, "Mod_Entry_Y", "module.ifo.Mod_Entry_Y")?,
            "module.ifo.Mod_Entry_Y",
        )?,
        z: binary_creature_float(
            binary_creature_field(&ifo.root.fields, "Mod_Entry_Z", "module.ifo.Mod_Entry_Z")?,
            "module.ifo.Mod_Entry_Z",
        )?,
    };
    let entry_direction = M0RuntimeDirectionV1 {
        x: binary_creature_float(
            binary_creature_field(
                &ifo.root.fields,
                "Mod_Entry_Dir_X",
                "module.ifo.Mod_Entry_Dir_X",
            )?,
            "module.ifo.Mod_Entry_Dir_X",
        )?,
        y: binary_creature_float(
            binary_creature_field(
                &ifo.root.fields,
                "Mod_Entry_Dir_Y",
                "module.ifo.Mod_Entry_Dir_Y",
            )?,
            "module.ifo.Mod_Entry_Dir_Y",
        )?,
    };
    if entry_position
        != (M0RuntimePositionV1 {
            x: M0_RUNTIME_ENTRY_X,
            y: M0_RUNTIME_ENTRY_Y,
            z: M0_RUNTIME_ENTRY_Z,
        })
        || entry_direction
            != (M0RuntimeDirectionV1 {
                x: M0_RUNTIME_ENTRY_DIR_X,
                y: M0_RUNTIME_ENTRY_DIR_Y,
            })
    {
        return Err(binary_creature_readback_error(
            "module.ifo.entry",
            "entry must remain [10,10,0] facing +Y",
        ));
    }
    let ordered_hak_resrefs =
        match binary_creature_field(&ifo.root.fields, "Mod_HakList", "module.ifo.Mod_HakList")? {
            GffValueV1::List(entries) if entries.len() == 1 => entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let path = format!("module.ifo.Mod_HakList[{index}].Mod_Hak");
                    binary_creature_string(
                        binary_creature_field(&entry.fields, "Mod_Hak", &path)?,
                        &path,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => {
                return Err(binary_creature_readback_error(
                    "module.ifo.Mod_HakList",
                    "expected exactly one HAK",
                ));
            }
        };

    let are = read_gff_v32(
        archive
            .find(&area_resref, ARE_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.are", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.are", value.message))?;
    if are.file_type != GffFileTypeV1::Are {
        return Err(binary_creature_readback_error(
            "area.are.fileType",
            "Area ARE resource must contain an exact ARE GFF",
        ));
    }
    let area_width = binary_creature_int(
        binary_creature_field(&are.root.fields, "Width", "area.are.Width")?,
        "area.are.Width",
    )?;
    let area_height = binary_creature_int(
        binary_creature_field(&are.root.fields, "Height", "area.are.Height")?,
        "area.are.Height",
    )?;
    let tileset_resref = binary_creature_resref(
        binary_creature_field(&are.root.fields, "Tileset", "area.are.Tileset")?,
        "area.are.Tileset",
    )?;
    let tiles = match binary_creature_field(&are.root.fields, "Tile_List", "area.are.Tile_List")? {
        GffValueV1::List(values) => values,
        _ => {
            return Err(binary_creature_readback_error(
                "area.are.Tile_List",
                "expected tile list",
            ));
        }
    };
    let tiles_are_exact = tiles.len() == M0_RUNTIME_TILES.len()
        && tiles
            .iter()
            .zip(M0_RUNTIME_TILES)
            .all(|(tile, (expected_id, expected_orientation))| {
                tile.fields.iter().any(|field| {
                    field.label == "Tile_ID" && field.value == GffValueV1::Int(expected_id)
                }) && tile.fields.iter().any(|field| {
                    field.label == "Tile_Orientation"
                        && field.value == GffValueV1::Int(expected_orientation)
                }) && ["Tile_AnimLoop1", "Tile_AnimLoop2", "Tile_AnimLoop3"]
                    .iter()
                    .all(|label| {
                        tile.fields.iter().any(|field| {
                            field.label == *label
                                && field.value == GffValueV1::Byte(M0_RUNTIME_TILE_ANIMATION_LOOP)
                        })
                    })
            });
    if area_width != M0_RUNTIME_AREA_WIDTH
        || area_height != M0_RUNTIME_AREA_HEIGHT
        || tileset_resref != M0_RUNTIME_TILESET_RESREF
        || !tiles_are_exact
    {
        return Err(binary_creature_readback_error(
            "area.are",
            "Area differs from the exact owned 2x2 tms01 tile contract",
        ));
    }

    let git = read_gff_v32(
        archive
            .find(&area_resref, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.git", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.git", value.message))?;
    if git.file_type != GffFileTypeV1::Git {
        return Err(binary_creature_readback_error(
            "area.git.fileType",
            "Area GIT resource must contain an exact GIT GFF",
        ));
    }
    let creatures =
        match binary_creature_field(&git.root.fields, "Creature List", "area.git.Creature List")? {
            GffValueV1::List(values) if !values.is_empty() => values,
            _ => {
                return Err(binary_creature_readback_error(
                    "area.git.Creature List",
                    "expected at least one creature fixture",
                ));
            }
        };
    let mut fixtures = Vec::with_capacity(creatures.len());
    for (index, creature) in creatures.iter().enumerate() {
        validate_binary_creature_runtime_complete_envelope(
            &creature.fields,
            &format!("area.git.Creature List[{index}]"),
        )?;
        let path = |field: &str| format!("area.git.Creature List[{index}].{field}");
        let id_path = path("Tag");
        let template_path = path("TemplateResRef");
        let display_path = path("FirstName");
        let appearance_path = path("Appearance_Type");
        let x_path = path("XPosition");
        let y_path = path("YPosition");
        let z_path = path("ZPosition");
        let orientation_x_path = path("XOrientation");
        let orientation_y_path = path("YOrientation");
        fixtures.push(BinaryCreatureOwnedFixtureV1 {
            id: binary_creature_string(
                binary_creature_field(&creature.fields, "Tag", &id_path)?,
                &id_path,
            )?,
            template_resref: binary_creature_resref(
                binary_creature_field(&creature.fields, "TemplateResRef", &template_path)?,
                &template_path,
            )?,
            display_name: binary_creature_loc_string(
                binary_creature_field(&creature.fields, "FirstName", &display_path)?,
                &display_path,
            )?,
            appearance_row: binary_creature_word(
                binary_creature_field(&creature.fields, "Appearance_Type", &appearance_path)?,
                &appearance_path,
            )?,
            position: M0RuntimePositionV1 {
                x: binary_creature_float(
                    binary_creature_field(&creature.fields, "XPosition", &x_path)?,
                    &x_path,
                )?,
                y: binary_creature_float(
                    binary_creature_field(&creature.fields, "YPosition", &y_path)?,
                    &y_path,
                )?,
                z: binary_creature_float(
                    binary_creature_field(&creature.fields, "ZPosition", &z_path)?,
                    &z_path,
                )?,
            },
            orientation: M0RuntimeDirectionV1 {
                x: binary_creature_float(
                    binary_creature_field(&creature.fields, "XOrientation", &orientation_x_path)?,
                    &orientation_x_path,
                )?,
                y: binary_creature_float(
                    binary_creature_field(&creature.fields, "YOrientation", &orientation_y_path)?,
                    &orientation_y_path,
                )?,
            },
        });
    }
    validate_binary_creature_multi_fixture_input(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: module_resref.clone(),
            area_resref: area_resref.clone(),
            hak_resref: ordered_hak_resrefs[0].clone(),
        },
        &fixtures,
    )?;

    let gic = read_gff_v32(
        archive
            .find(&area_resref, GIC_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.gic", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.gic", value.message))?;
    if gic.file_type != GffFileTypeV1::Gic {
        return Err(binary_creature_readback_error(
            "area.gic.fileType",
            "Area GIC resource must contain an exact GIC GFF",
        ));
    }
    if !matches!(
        binary_creature_field(&gic.root.fields, "Creature List", "area.gic.Creature List")?,
        GffValueV1::List(values) if values.len() == fixtures.len()
    ) {
        return Err(binary_creature_readback_error(
            "area.gic.Creature List",
            "GIC fixture count differs from GIT",
        ));
    }
    let expected_resources = [
        ("module".to_owned(), IFO_RESOURCE_TYPE),
        ("repute".to_owned(), FAC_RESOURCE_TYPE),
        (area_resref.clone(), ARE_RESOURCE_TYPE),
        (area_resref.clone(), GIC_RESOURCE_TYPE),
        (area_resref.clone(), GIT_RESOURCE_TYPE),
    ]
    .into_iter()
    .chain(
        fixtures
            .iter()
            .map(|fixture| (fixture.template_resref.clone(), UTC_RESOURCE_TYPE)),
    )
    .chain(allowed_extra_resources.iter().cloned())
    .collect::<HashSet<_>>();
    let actual_resources = archive
        .resources()
        .iter()
        .map(|resource| (resource.resref.clone(), resource.resource_type))
        .collect::<HashSet<_>>();
    if archive.resources().len() != expected_resources.len()
        || actual_resources != expected_resources
    {
        return Err(binary_creature_readback_error(
            "module.resources",
            "expected the exact module core resource set plus one UTC per fixture",
        ));
    }
    for (index, fixture) in fixtures.iter().enumerate() {
        let utc_path = format!("fixtures[{index}].utc");
        let utc = read_gff_v32(
            archive
                .find(&fixture.template_resref, UTC_RESOURCE_TYPE)
                .map_err(|value| error(value.code, &utc_path, value.context))?,
            &GffLimitsV1::default(),
        )
        .map_err(|value| error(value.code, &utc_path, value.message))?;
        if utc.file_type != GffFileTypeV1::Utc {
            return Err(binary_creature_readback_error(
                format!("{utc_path}.fileType"),
                "fixture archive resource must contain an exact UTC GFF",
            ));
        }
        if utc.root.struct_id != u32::MAX {
            return Err(binary_creature_readback_error(
                format!("{utc_path}.structId"),
                "UTC root struct ID must be 0xffffffff",
            ));
        }
        validate_binary_creature_runtime_complete_envelope(&utc.root.fields, &utc_path)?;
        let utc_template_path = format!("{utc_path}.TemplateResRef");
        let utc_template = binary_creature_resref(
            binary_creature_field(&utc.root.fields, "TemplateResRef", &utc_template_path)?,
            &utc_template_path,
        )?;
        if utc_template != fixture.template_resref {
            return Err(binary_creature_readback_error(
                utc_template_path,
                "UTC TemplateResRef differs from its GIT and archive-key template",
            ));
        }
        let utc_id = binary_creature_string(
            binary_creature_field(&utc.root.fields, "Tag", &format!("{utc_path}.Tag"))?,
            &format!("{utc_path}.Tag"),
        )?;
        let utc_display = binary_creature_loc_string(
            binary_creature_field(
                &utc.root.fields,
                "FirstName",
                &format!("{utc_path}.FirstName"),
            )?,
            &format!("{utc_path}.FirstName"),
        )?;
        let utc_appearance = binary_creature_word(
            binary_creature_field(
                &utc.root.fields,
                "Appearance_Type",
                &format!("{utc_path}.Appearance_Type"),
            )?,
            &format!("{utc_path}.Appearance_Type"),
        )?;
        if utc_id != fixture.id
            || utc_display != fixture.display_name
            || utc_appearance != fixture.appearance_row
        {
            return Err(binary_creature_readback_error(
                utc_path,
                "UTC id, display name, or appearance differs from GIT",
            ));
        }
    }

    Ok(BinaryCreatureMultiFixtureModuleReadbackV1 {
        schema_version: 1,
        module_resref,
        area_resref,
        ordered_hak_resrefs,
        entry_position,
        entry_direction,
        tileset_resref,
        area_width,
        area_height,
        tile_count: tiles.len() as u32,
        fixtures,
    })
}

/// Independently classifies every emitted runtime profile from the exact MOD
/// bytes.  A partial or previously unknown mixture fails closed instead of
/// being silently labelled as one of the supported profiles.
pub fn inspect_binary_creature_profile_matrix_module_v2(
    bytes: &[u8],
) -> Result<BinaryCreatureProfileMatrixModuleReadbackV2, ProofModuleErrorV1> {
    inspect_binary_creature_profile_matrix_module_with_extras_v2(bytes, &[], false)
}

fn inspect_binary_creature_profile_matrix_module_with_extras_v2(
    bytes: &[u8],
    allowed_extra_resources: &[(String, u16)],
    ignore_hand_equipment_for_profile_classification: bool,
) -> Result<BinaryCreatureProfileMatrixModuleReadbackV2, ProofModuleErrorV1> {
    let scene = inspect_binary_creature_multi_fixture_module_with_extras_v1(
        bytes,
        allowed_extra_resources,
    )?;
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    let git = read_gff_v32(
        archive
            .find(&scene.area_resref, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.git", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.git", value.message))?;
    let creatures =
        match binary_creature_field(&git.root.fields, "Creature List", "area.git.Creature List")? {
            GffValueV1::List(values) if values.len() == scene.fixtures.len() => values,
            _ => {
                return Err(binary_creature_profile_readback_error(
                    "area.git.Creature List",
                    "profile matrix creature count differs from V1 scene readback",
                ));
            }
        };

    let mut fixtures = Vec::with_capacity(scene.fixtures.len());
    for (index, (fixture, creature)) in scene.fixtures.iter().zip(creatures).enumerate() {
        let git_path = format!("area.git.Creature List[{index}].runtimeProfile");
        let git_fields = runtime_profile_classification_fields(
            &creature.fields,
            ignore_hand_equipment_for_profile_classification,
        );
        let git_profile =
            classify_binary_creature_runtime_profile(&git_fields, fixture, &git_path)?;
        let utc_path = format!("fixtures[{index}].utc.runtimeProfile");
        let utc = read_gff_v32(
            archive
                .find(&fixture.template_resref, UTC_RESOURCE_TYPE)
                .map_err(|value| error(value.code, &utc_path, value.context))?,
            &GffLimitsV1::default(),
        )
        .map_err(|value| error(value.code, &utc_path, value.message))?;
        let utc_fields = runtime_profile_classification_fields(
            &utc.root.fields,
            ignore_hand_equipment_for_profile_classification,
        );
        let utc_profile =
            classify_binary_creature_runtime_profile(&utc_fields, fixture, &utc_path)?;
        if git_profile != utc_profile {
            return Err(binary_creature_profile_readback_error(
                utc_path,
                "GIT instance and module-local UTC classify as different runtime profiles",
            ));
        }
        fixtures.push(BinaryCreatureProfiledFixtureV2 {
            fixture: fixture.clone(),
            runtime_profile: git_profile,
        });
    }

    Ok(BinaryCreatureProfileMatrixModuleReadbackV2 {
        schema_version: 2,
        scene,
        fixtures,
    })
}

fn runtime_profile_classification_fields(
    fields: &[GffFieldV1],
    ignore_hand_equipment: bool,
) -> Vec<GffFieldV1> {
    let mut normalized = fields.to_vec();
    if ignore_hand_equipment
        && let Some(equipment) = normalized
            .iter_mut()
            .find(|field| field.label == "Equip_ItemList")
    {
        equipment.value = GffValueV1::List(Vec::new());
    }
    normalized
}

/// Reads the final MOD bytes and independently binds both GIT and UTC
/// equipment to the owned UTI.  Extra archive resources remain rejected; the
/// single UTI is the only extension of the profile-matrix resource contract.
pub fn inspect_binary_creature_weapon_demo_module_v1(
    bytes: &[u8],
) -> Result<BinaryCreatureWeaponDemoReadbackV1, ProofModuleErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    let uti_resources = archive
        .resources()
        .iter()
        .filter(|resource| resource.resource_type == UTI_RESOURCE_TYPE)
        .collect::<Vec<_>>();
    if uti_resources.len() != 1 {
        return Err(binary_creature_readback_error(
            "module.resources.uti",
            "weapon demo must contain exactly one owned UTI",
        ));
    }
    let weapon_resref = uti_resources[0].resref.clone();
    let scene = inspect_binary_creature_profile_matrix_module_with_extras_v2(
        bytes,
        &[(weapon_resref.clone(), UTI_RESOURCE_TYPE)],
        true,
    )?;
    let weapon_document = read_gff_v32(
        archive
            .find(&weapon_resref, UTI_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "weapon.uti", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "weapon.uti", value.message))?;
    if weapon_document.file_type != GffFileTypeV1::Uti {
        return Err(binary_creature_readback_error(
            "weapon.uti.fileType",
            "weapon resource must contain an exact UTI GFF",
        ));
    }
    let byte = |label: &str| -> Result<u8, ProofModuleErrorV1> {
        match binary_creature_field(
            &weapon_document.root.fields,
            label,
            &format!("weapon.uti.{label}"),
        )? {
            GffValueV1::Byte(value) => Ok(*value),
            _ => Err(binary_creature_readback_error(
                format!("weapon.uti.{label}"),
                "expected byte",
            )),
        }
    };
    let weapon = BinaryCreatureWeaponItemV1 {
        resref: binary_creature_resref(
            binary_creature_field(
                &weapon_document.root.fields,
                "TemplateResRef",
                "weapon.uti.TemplateResRef",
            )?,
            "weapon.uti.TemplateResRef",
        )?,
        display_name: binary_creature_loc_string(
            binary_creature_field(
                &weapon_document.root.fields,
                "LocalizedName",
                "weapon.uti.LocalizedName",
            )?,
            "weapon.uti.LocalizedName",
        )?,
        base_item: binary_creature_int(
            binary_creature_field(
                &weapon_document.root.fields,
                "BaseItem",
                "weapon.uti.BaseItem",
            )?,
            "weapon.uti.BaseItem",
        )?,
        model_parts: [
            byte("ModelPart1")?,
            byte("ModelPart2")?,
            byte("ModelPart3")?,
        ],
    };
    if weapon.resref != weapon_resref {
        return Err(binary_creature_readback_error(
            "weapon.uti.TemplateResRef",
            "UTI TemplateResRef differs from its archive resource key",
        ));
    }

    let git = read_gff_v32(
        archive
            .find(&scene.scene.area_resref, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.git", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.git", value.message))?;
    let creatures =
        match binary_creature_field(&git.root.fields, "Creature List", "area.git.Creature List")? {
            GffValueV1::List(values) if values.len() == scene.fixtures.len() => values,
            _ => {
                return Err(binary_creature_readback_error(
                    "area.git.Creature List",
                    "weapon demo fixture count differs from profile readback",
                ));
            }
        };
    let mut fixtures = Vec::with_capacity(scene.fixtures.len());
    for (index, (profiled_fixture, git_creature)) in
        scene.fixtures.iter().zip(creatures).enumerate()
    {
        let git_equipment = read_binary_creature_hand_equipment(
            &git_creature.fields,
            &format!("area.git.Creature List[{index}].Equip_ItemList"),
        )?;
        let utc_path = format!("fixtures[{index}].utc");
        let utc = read_gff_v32(
            archive
                .find(&profiled_fixture.fixture.template_resref, UTC_RESOURCE_TYPE)
                .map_err(|value| error(value.code, &utc_path, value.context))?,
            &GffLimitsV1::default(),
        )
        .map_err(|value| error(value.code, &utc_path, value.message))?;
        let utc_equipment = read_binary_creature_hand_equipment(
            &utc.root.fields,
            &format!("{utc_path}.Equip_ItemList"),
        )?;
        if git_equipment != utc_equipment || git_equipment.1 != weapon.resref {
            return Err(binary_creature_readback_error(
                format!("fixtures[{index}].Equip_ItemList"),
                "GIT and UTC equipment must agree and reference the owned UTI",
            ));
        }
        fixtures.push(BinaryCreatureEquippedFixtureV1 {
            profiled_fixture: profiled_fixture.clone(),
            hand: git_equipment.0,
            equipped_item_resref: git_equipment.1,
        });
    }
    Ok(BinaryCreatureWeaponDemoReadbackV1 {
        schema_version: 1,
        scene,
        weapon,
        fixtures,
    })
}

/// Reads a V2 stock-weapon demo and proves the GIT/UTC equipment references.
/// The external base-game UTI is intentionally not copied into the MOD; its
/// resolution scope remains explicit in `weapon` rather than being faked by a
/// module-local parser-only resource.
pub fn inspect_binary_creature_stock_weapon_demo_module_v2(
    bytes: &[u8],
    weapon: &BinaryCreatureStockWeaponV2,
) -> Result<BinaryCreatureWeaponDemoReadbackV2, ProofModuleErrorV1> {
    validate_binary_creature_stock_weapon_identity_v2(weapon)?;
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    if archive
        .resources()
        .iter()
        .any(|resource| resource.resource_type == UTI_RESOURCE_TYPE)
    {
        return Err(binary_creature_readback_error(
            "module.resources.uti",
            "V2 stock-weapon demo must resolve the base UTI and must not embed a module-local UTI",
        ));
    }
    let scene = inspect_binary_creature_profile_matrix_module_with_extras_v2(bytes, &[], true)?;
    let git = read_gff_v32(
        archive
            .find(&scene.scene.area_resref, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "area.git", value.context))?,
        &GffLimitsV1::default(),
    )
    .map_err(|value| error(value.code, "area.git", value.message))?;
    let creatures =
        match binary_creature_field(&git.root.fields, "Creature List", "area.git.Creature List")? {
            GffValueV1::List(values) if values.len() == scene.fixtures.len() => values,
            _ => {
                return Err(binary_creature_readback_error(
                    "area.git.Creature List",
                    "V2 weapon demo fixture count differs from profile readback",
                ));
            }
        };
    let mut fixtures = Vec::with_capacity(scene.fixtures.len());
    for (index, (profiled_fixture, git_creature)) in
        scene.fixtures.iter().zip(creatures).enumerate()
    {
        let git_equipment = read_binary_creature_hand_equipment(
            &git_creature.fields,
            &format!("area.git.Creature List[{index}].Equip_ItemList"),
        )?;
        let utc_path = format!("fixtures[{index}].utc");
        let utc = read_gff_v32(
            archive
                .find(&profiled_fixture.fixture.template_resref, UTC_RESOURCE_TYPE)
                .map_err(|value| error(value.code, &utc_path, value.context))?,
            &GffLimitsV1::default(),
        )
        .map_err(|value| error(value.code, &utc_path, value.message))?;
        let utc_equipment = read_binary_creature_hand_equipment(
            &utc.root.fields,
            &format!("{utc_path}.Equip_ItemList"),
        )?;
        if git_equipment != utc_equipment || git_equipment.1 != weapon.resref {
            return Err(binary_creature_readback_error(
                format!("fixtures[{index}].Equip_ItemList"),
                "GIT and UTC equipment must agree and reference the exact NWN base weapon",
            ));
        }
        fixtures.push(BinaryCreatureEquippedFixtureV1 {
            profiled_fixture: profiled_fixture.clone(),
            hand: git_equipment.0,
            equipped_item_resref: git_equipment.1,
        });
    }
    Ok(BinaryCreatureWeaponDemoReadbackV2 {
        schema_version: 2,
        scene,
        weapon: weapon.clone(),
        fixtures,
    })
}

fn read_binary_creature_hand_equipment(
    fields: &[GffFieldV1],
    path: &str,
) -> Result<(BinaryCreatureHandSlotV1, String), ProofModuleErrorV1> {
    let list = match binary_creature_field(fields, "Equip_ItemList", path)? {
        GffValueV1::List(values) if values.len() == 1 => values,
        _ => {
            return Err(binary_creature_readback_error(
                path,
                "expected exactly one equipped hand item",
            ));
        }
    };
    let hand = match list[0].struct_id {
        16 => BinaryCreatureHandSlotV1::RightHand,
        32 => BinaryCreatureHandSlotV1::LeftHand,
        other => {
            return Err(binary_creature_readback_error(
                path,
                format!("unsupported native hand slot struct id {other}"),
            ));
        }
    };
    let equipped = binary_creature_resref(
        binary_creature_field(
            &list[0].fields,
            "EquippedRes",
            &format!("{path}[0].EquippedRes"),
        )?,
        &format!("{path}[0].EquippedRes"),
    )?;
    Ok((hand, equipped))
}

fn build_canonical_creature_proof_module_v1(
    appearance_row: u16,
    creature_resref: &str,
    creature_display_name: &str,
    area_display_name: &str,
    description: &str,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    let resources = vec![
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            module_ifo_for(
                PROOF_MODULE_RESREF,
                PROOF_AREA_RESREF,
                &[PROOF_HAK_RESREF],
                "Meshy2Aurora canonical runtime proof",
                description,
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            PROOF_AREA_RESREF,
            ARE_RESOURCE_TYPE,
            proof_area_for(PROOF_AREA_RESREF, area_display_name, description)?,
        ),
        resource(
            PROOF_AREA_RESREF,
            GIC_RESOURCE_TYPE,
            proof_gic_for(&format!(
                "Generated by Codex: canonical proof placement for {creature_resref}"
            ))?,
        ),
        resource(
            PROOF_AREA_RESREF,
            GIT_RESOURCE_TYPE,
            proof_git_for(appearance_row, creature_resref, creature_display_name)?,
        ),
        resource(
            creature_resref,
            UTC_RESOURCE_TYPE,
            proof_utc_for(appearance_row, creature_resref, creature_display_name)?,
        ),
    ];
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M6-PROOF-MODULE-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    validate_module_readback(&archive.payload, appearance_row, creature_resref)?;
    Ok(ProofModuleArtifactV1 {
        report: ProofModuleReportV2 {
            schema_version: 2,
            module_resref: PROOF_MODULE_RESREF.to_owned(),
            module_display_name: "Meshy2Aurora canonical runtime proof".to_owned(),
            area_resref: PROOF_AREA_RESREF.to_owned(),
            area_display_name: area_display_name.to_owned(),
            creature_resref: creature_resref.to_owned(),
            hak_resref: PROOF_HAK_RESREF.to_owned(),
            appearance_row,
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            sha256: sha256(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            binary_m0_runtime_fixture: None,
        },
        payload: archive.payload,
    })
}

/// Builds the two-fixture M0 runtime module.  The generated M0 and the
/// reference tortoise occupy recorded, separate positions in one area.  The
/// tortoise's payload stays in its owner-provided HAK, below the generated M0
/// HAK in `Mod_HakList` so our combined `appearance.2da` is authoritative.
pub fn build_m0_control_proof_module_v1(
    tortoise_appearance_row: u16,
    m0_appearance_row: u16,
) -> Result<ProofModuleArtifactV1, ProofModuleErrorV1> {
    let fixtures = [
        M0FixtureV1 {
            creature_resref: M0_CONTROL_CREATURE_RESREF,
            display_name: "Reference tortoise control",
            appearance_row: tortoise_appearance_row,
            x: 12.0,
            y: 10.0,
        },
        M0FixtureV1 {
            creature_resref: M0_PROOF_CREATURE_RESREF,
            display_name: "Meshy M0 static rigid control",
            appearance_row: m0_appearance_row,
            x: 16.0,
            y: 10.0,
        },
    ];
    let hak_resrefs = [M0_TORTOISE_REFERENCE_HAK_RESREF, M0_PROOF_HAK_RESREF];
    let resources = vec![
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            module_ifo_for(
                M0_PROOF_MODULE_RESREF,
                M0_PROOF_AREA_RESREF,
                &hak_resrefs,
                "Meshy2Aurora M0 static runtime control",
                "Generated M0 static Meshy proof with a separate tortoise positive control.",
            )?,
        ),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(
            M0_PROOF_AREA_RESREF,
            ARE_RESOURCE_TYPE,
            proof_area_for(
                M0_PROOF_AREA_RESREF,
                "Meshy2Aurora M0 static runtime proof area",
                "Generated by Codex for Meshy M0 static rigid runtime proof.",
            )?,
        ),
        resource(
            M0_PROOF_AREA_RESREF,
            GIC_RESOURCE_TYPE,
            m0_proof_gic(&fixtures)?,
        ),
        resource(
            M0_PROOF_AREA_RESREF,
            GIT_RESOURCE_TYPE,
            m0_proof_git(&fixtures)?,
        ),
        resource(
            M0_CONTROL_CREATURE_RESREF,
            UTC_RESOURCE_TYPE,
            m0_proof_utc(&fixtures[0])?,
        ),
        resource(
            M0_PROOF_CREATURE_RESREF,
            UTC_RESOURCE_TYPE,
            m0_proof_utc(&fixtures[1])?,
        ),
    ];
    let archive = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|write_error| {
        error(
            "M0-PROOF-MODULE-WRITE-FAILED",
            "module",
            write_error.to_string(),
        )
    })?;
    validate_m0_control_module_readback(&archive.payload, &fixtures, &hak_resrefs)?;
    Ok(ProofModuleArtifactV1 {
        report: ProofModuleReportV2 {
            schema_version: 2,
            module_resref: M0_PROOF_MODULE_RESREF.to_owned(),
            module_display_name: "Meshy2Aurora M0 static runtime control".to_owned(),
            area_resref: M0_PROOF_AREA_RESREF.to_owned(),
            area_display_name: "Meshy2Aurora M0 static runtime proof area".to_owned(),
            creature_resref: M0_PROOF_CREATURE_RESREF.to_owned(),
            hak_resref: M0_PROOF_HAK_RESREF.to_owned(),
            appearance_row: m0_appearance_row,
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            sha256: sha256(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
            binary_m0_runtime_fixture: None,
        },
        payload: archive.payload,
    })
}

fn module_ifo_for(
    module_resref: &str,
    area_resref: &str,
    hak_resrefs: &[&str],
    display_name: &str,
    description: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    module_ifo_for_with_entry(
        module_resref,
        area_resref,
        hak_resrefs,
        display_name,
        description,
        (5.0, 5.0, 0.0),
        (1.0, 0.0),
    )
}

pub(crate) fn binary_m0_module_ifo_for(
    module_resref: &str,
    area_resref: &str,
    hak_resrefs: &[&str],
    display_name: &str,
    description: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    module_ifo_for_with_entry(
        module_resref,
        area_resref,
        hak_resrefs,
        display_name,
        description,
        (M0_RUNTIME_ENTRY_X, M0_RUNTIME_ENTRY_Y, M0_RUNTIME_ENTRY_Z),
        (M0_RUNTIME_ENTRY_DIR_X, M0_RUNTIME_ENTRY_DIR_Y),
    )
}

fn module_ifo_for_with_entry(
    module_resref: &str,
    area_resref: &str,
    hak_resrefs: &[&str],
    display_name: &str,
    description: &str,
    entry_position: (f32, f32, f32),
    entry_direction: (f32, f32),
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    // IFO55 is the frozen Aurora manifest.  A sparse IFO can be parsed by our
    // reader yet leaves NWN traversing absent labels during module startup.
    let mut fields = vec![
        field("Mod_ID", GffValueV1::Void(vec![0; 16])),
        field("Mod_MinGameVer", string("1.69")),
        field("Mod_Creator_ID", GffValueV1::Int(0)),
        field("Mod_Version", GffValueV1::Dword(3)),
        field("Expansion_Pack", GffValueV1::Word(0)),
        field("Mod_Name", loc(display_name)),
        field("Mod_Tag", string(module_resref)),
        field("Mod_Description", loc(description)),
        field("Mod_IsSaveGame", GffValueV1::Byte(0)),
        field("Mod_CustomTlk", string("")),
        field("Mod_Entry_Area", resref(area_resref)),
        field("Mod_Entry_X", GffValueV1::Float(entry_position.0)),
        field("Mod_Entry_Y", GffValueV1::Float(entry_position.1)),
        field("Mod_Entry_Z", GffValueV1::Float(entry_position.2)),
        field("Mod_Entry_Dir_X", GffValueV1::Float(entry_direction.0)),
        field("Mod_Entry_Dir_Y", GffValueV1::Float(entry_direction.1)),
        field("Mod_Expan_List", GffValueV1::List(Vec::new())),
        field("Mod_DawnHour", GffValueV1::Byte(6)),
        field("Mod_DuskHour", GffValueV1::Byte(18)),
        field("Mod_MinPerHour", GffValueV1::Byte(60)),
        field("Mod_StartMonth", GffValueV1::Byte(0)),
        field("Mod_StartDay", GffValueV1::Byte(1)),
        field("Mod_StartHour", GffValueV1::Byte(12)),
        field("Mod_StartYear", GffValueV1::Dword(1372)),
        field("Mod_XPScale", GffValueV1::Byte(10)),
    ];
    fields.extend(
        [
            "Mod_OnHeartbeat",
            "Mod_OnModLoad",
            "Mod_OnModStart",
            "Mod_OnClientEntr",
            "Mod_OnClientLeav",
            "Mod_OnActvtItem",
            "Mod_OnAcquirItem",
            "Mod_OnUsrDefined",
            "Mod_OnUnAqreItem",
            "Mod_OnPlrDeath",
            "Mod_OnPlrDying",
            "Mod_OnPlrEqItm",
            "Mod_OnPlrLvlUp",
            "Mod_OnSpawnBtnDn",
            "Mod_OnPlrRest",
            "Mod_OnPlrUnEqItm",
            "Mod_OnCutsnAbort",
            "Mod_OnPlrChat",
            "Mod_OnPlrTarget",
            "Mod_OnPlrGuiEvt",
            "Mod_OnPlrTileAct",
            "Mod_OnNuiEvent",
            "Mod_StartMovie",
            "Mod_DefaultBic",
        ]
        .into_iter()
        .map(|label| field(label, resref(""))),
    );
    fields.extend([
        field("Mod_UUID", string("")),
        field("Mod_PartyControl", GffValueV1::Int(0)),
        field("Mod_CutSceneList", GffValueV1::List(Vec::new())),
        field("Mod_GVar_List", GffValueV1::List(Vec::new())),
        field(
            "Mod_Area_list",
            GffValueV1::List(vec![GffStructV1 {
                struct_id: 6,
                fields: vec![field("Area_Name", resref(area_resref))],
            }]),
        ),
        field(
            "Mod_HakList",
            GffValueV1::List(
                hak_resrefs
                    .iter()
                    .map(|hak_resref| GffStructV1 {
                        struct_id: 8,
                        fields: vec![field("Mod_Hak", string(hak_resref))],
                    })
                    .collect(),
            ),
        ),
    ]);
    gff(GffFileTypeV1::Ifo, fields)
}

fn proof_area_for(
    area_resref: &str,
    display_name: &str,
    comments: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    gff(
        GffFileTypeV1::Are,
        vec![
            field("ID", GffValueV1::Int(0)),
            // The prior `tdc01` profile proved that a field of one repeated
            // interior tile is not a valid Toolset render base: every GFF
            // field read back, but TfrmViewerArea remained black.  Use the
            // independently observed `tin01` environment invariants from the
            // local, renderable proof precedent instead.  The tile field
            // below remains generated by this writer.
            field("Creator_ID", GffValueV1::Int(-1)),
            field("Version", GffValueV1::Dword(2)),
            field("Tag", string(area_resref)),
            field("Name", loc(display_name)),
            field("ResRef", resref(area_resref)),
            field("Comments", string(comments)),
            field("Expansion_List", GffValueV1::List(Vec::new())),
            field("Flags", GffValueV1::Dword(1)),
            field("ModSpotCheck", GffValueV1::Int(0)),
            field("ModListenCheck", GffValueV1::Int(0)),
            field("MoonAmbientColor", GffValueV1::Dword(3_947_580)),
            field("MoonDiffuseColor", GffValueV1::Dword(11_184_810)),
            field("MoonFogAmount", GffValueV1::Byte(5)),
            field("MoonFogColor", GffValueV1::Dword(0)),
            field("MoonShadows", GffValueV1::Byte(0)),
            // `tin01` tile 94 is the observed interior class in the renderable
            // local precedent.  We generate a deterministic full field rather
            // than reusing any reference Area payload.
            field("SunAmbientColor", GffValueV1::Dword(0)),
            field("SunDiffuseColor", GffValueV1::Dword(0)),
            field("SunFogAmount", GffValueV1::Byte(0)),
            field("SunFogColor", GffValueV1::Dword(0)),
            field("SunShadows", GffValueV1::Byte(0)),
            field("IsNight", GffValueV1::Byte(1)),
            field("LightingScheme", GffValueV1::Byte(12)),
            field("ShadowOpacity", GffValueV1::Byte(60)),
            field("FogClipDist", GffValueV1::Float(45.0)),
            field("SkyBox", GffValueV1::Byte(0)),
            field("DayNightCycle", GffValueV1::Byte(0)),
            field("ChanceRain", GffValueV1::Int(0)),
            field("ChanceSnow", GffValueV1::Int(0)),
            field("ChanceLightning", GffValueV1::Int(0)),
            field("WindPower", GffValueV1::Int(0)),
            field("LoadScreenID", GffValueV1::Word(0)),
            field("PlayerVsPlayer", GffValueV1::Byte(3)),
            field("NoRest", GffValueV1::Byte(0)),
            field("Width", GffValueV1::Int(8)),
            field("Height", GffValueV1::Int(8)),
            field("OnEnter", resref("")),
            field("OnExit", resref("")),
            field("OnHeartbeat", resref("")),
            field("OnUserDefined", resref("")),
            field("TileBrdrDisabled", GffValueV1::Byte(0)),
            field("Tileset", resref("tin01")),
            field(
                "Tile_List",
                GffValueV1::List((0..64).map(proof_tile).collect()),
            ),
        ],
    )
}

pub(crate) fn binary_m0_area_for(area_resref: &str) -> Result<Vec<u8>, ProofModuleErrorV1> {
    binary_m0_area_for_named(area_resref, "Meshy2Aurora M0 binary vertical-slice area")
}

pub(crate) fn binary_m0_area_for_named(
    area_resref: &str,
    area_display_name: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    // This is the one project-owned M0 runtime fixture. The exact `tms01`
    // sequence comes from the fresh Aurora-created r21 Area whose validated
    // TScrollBox visibly rendered both tiles and M0. Do not fall back to the
    // historical synthetic `tdc01` sequence, which lacked that live proof.
    gff(
        GffFileTypeV1::Are,
        vec![
            field("ID", GffValueV1::Int(0)),
            field("Creator_ID", GffValueV1::Int(0)),
            field("Version", GffValueV1::Dword(1)),
            field("Tag", string(area_resref)),
            field("Name", loc(area_display_name)),
            field("ResRef", resref(area_resref)),
            field(
                "Comments",
                string("Generated by Meshy2Aurora from the valid owned H1 area contract."),
            ),
            field("Expansion_List", GffValueV1::List(Vec::new())),
            field("Flags", GffValueV1::Dword(0)),
            field("ModSpotCheck", GffValueV1::Int(0)),
            field("ModListenCheck", GffValueV1::Int(0)),
            field("MoonAmbientColor", GffValueV1::Dword(0)),
            field("MoonDiffuseColor", GffValueV1::Dword(0x20_20_20)),
            field("MoonFogAmount", GffValueV1::Byte(0)),
            field("MoonFogColor", GffValueV1::Dword(0)),
            field("MoonShadows", GffValueV1::Byte(0)),
            field("SunAmbientColor", GffValueV1::Dword(0x40_40_40)),
            field("SunDiffuseColor", GffValueV1::Dword(0xff_ff_ff)),
            field("SunFogAmount", GffValueV1::Byte(0)),
            field("SunFogColor", GffValueV1::Dword(0x80_80_80)),
            field("SunShadows", GffValueV1::Byte(1)),
            field("IsNight", GffValueV1::Byte(0)),
            field("LightingScheme", GffValueV1::Byte(0)),
            field("ShadowOpacity", GffValueV1::Byte(50)),
            field("FogClipDist", GffValueV1::Float(45.0)),
            field("SkyBox", GffValueV1::Byte(0)),
            field("DayNightCycle", GffValueV1::Byte(1)),
            field("ChanceRain", GffValueV1::Int(0)),
            field("ChanceSnow", GffValueV1::Int(0)),
            field("ChanceLightning", GffValueV1::Int(0)),
            field("WindPower", GffValueV1::Int(0)),
            field("LoadScreenID", GffValueV1::Word(0)),
            field("PlayerVsPlayer", GffValueV1::Byte(0)),
            field("NoRest", GffValueV1::Byte(0)),
            field("Width", GffValueV1::Int(M0_RUNTIME_AREA_WIDTH)),
            field("Height", GffValueV1::Int(M0_RUNTIME_AREA_HEIGHT)),
            field("OnEnter", resref("")),
            field("OnExit", resref("")),
            field("OnHeartbeat", resref("")),
            field("OnUserDefined", resref("")),
            field("TileBrdrDisabled", GffValueV1::Byte(0)),
            field("Tileset", resref(M0_RUNTIME_TILESET_RESREF)),
            field(
                "Tile_List",
                GffValueV1::List(
                    M0_RUNTIME_TILES
                        .iter()
                        .map(|&(tile_id, orientation)| binary_m0_tile(tile_id, orientation))
                        .collect(),
                ),
            ),
        ],
    )
}

fn binary_m0_tile(tile_id: i32, orientation: i32) -> GffStructV1 {
    GffStructV1 {
        struct_id: 1,
        fields: vec![
            field("Tile_ID", GffValueV1::Int(tile_id)),
            field("Tile_Orientation", GffValueV1::Int(orientation)),
            field("Tile_Height", GffValueV1::Int(0)),
            field("Tile_MainLight1", GffValueV1::Byte(0)),
            field("Tile_MainLight2", GffValueV1::Byte(0)),
            field("Tile_SrcLight1", GffValueV1::Byte(0)),
            field("Tile_SrcLight2", GffValueV1::Byte(0)),
            field(
                "Tile_AnimLoop1",
                GffValueV1::Byte(M0_RUNTIME_TILE_ANIMATION_LOOP),
            ),
            field(
                "Tile_AnimLoop2",
                GffValueV1::Byte(M0_RUNTIME_TILE_ANIMATION_LOOP),
            ),
            field(
                "Tile_AnimLoop3",
                GffValueV1::Byte(M0_RUNTIME_TILE_ANIMATION_LOOP),
            ),
        ],
    }
}

fn proof_tile(index: usize) -> GffStructV1 {
    // The layout deliberately contains no copied reference map.  Its four
    // locally generated light/orientation states cover the valid runtime
    // channel ranges observed in the renderable `tin01` precedent.
    let (orientation, main_light_1, main_light_2, source_light_1, source_light_2) = match index % 4
    {
        0 => (2, 0, 0, 0, 0),
        1 => (0, 30, 0, 3, 3),
        2 => (3, 0, 14, 3, 3),
        _ => (1, 4, 14, 2, 2),
    };
    GffStructV1 {
        struct_id: 1,
        fields: vec![
            field("Tile_ID", GffValueV1::Int(94)),
            field("Tile_Orientation", GffValueV1::Int(orientation)),
            field("Tile_Height", GffValueV1::Int(0)),
            field("Tile_MainLight1", GffValueV1::Byte(main_light_1)),
            field("Tile_MainLight2", GffValueV1::Byte(main_light_2)),
            field("Tile_SrcLight1", GffValueV1::Byte(source_light_1)),
            field("Tile_SrcLight2", GffValueV1::Byte(source_light_2)),
            field("Tile_AnimLoop1", GffValueV1::Byte(1)),
            field("Tile_AnimLoop2", GffValueV1::Byte(1)),
            field("Tile_AnimLoop3", GffValueV1::Byte(1)),
        ],
    }
}

pub(crate) fn binary_creature_multi_fixture_gic(
    fixtures: &[BinaryCreatureOwnedFixtureV1],
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![field(
        "Creature List",
        GffValueV1::List(
            fixtures
                .iter()
                .map(|fixture| GffStructV1 {
                    struct_id: 4,
                    fields: vec![field(
                        "Comment",
                        string(&format!(
                            "Owned fixture {} at ({}, {}, {})",
                            fixture.id, fixture.position.x, fixture.position.y, fixture.position.z
                        )),
                    )],
                })
                .collect(),
        ),
    )];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Gic, fields)
}

pub(crate) fn binary_creature_multi_fixture_git(
    fixtures: &[BinaryCreatureOwnedFixtureV1],
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![
        field(
            "AreaProperties",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 100,
                fields: [
                    "AmbientSndDay",
                    "AmbientSndNight",
                    "AmbientSndDayVol",
                    "AmbientSndNitVol",
                    "EnvAudio",
                    "MusicBattle",
                    "MusicDay",
                    "MusicNight",
                    "MusicDelay",
                ]
                .into_iter()
                .map(|label| field(label, GffValueV1::Int(0)))
                .collect(),
            }),
        ),
        field(
            "Creature List",
            GffValueV1::List(
                fixtures
                    .iter()
                    .map(binary_creature_owned_fixture_git_creature)
                    .collect(),
            ),
        ),
    ];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Git, fields)
}

fn binary_creature_profile_matrix_git(
    fixtures: &[BinaryCreatureProfiledFixtureV2],
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![
        field(
            "AreaProperties",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 100,
                fields: [
                    "AmbientSndDay",
                    "AmbientSndNight",
                    "AmbientSndDayVol",
                    "AmbientSndNitVol",
                    "EnvAudio",
                    "MusicBattle",
                    "MusicDay",
                    "MusicNight",
                    "MusicDelay",
                ]
                .into_iter()
                .map(|label| field(label, GffValueV1::Int(0)))
                .collect(),
            }),
        ),
        field(
            "Creature List",
            GffValueV1::List(
                fixtures
                    .iter()
                    .map(binary_creature_profiled_fixture_git_creature)
                    .collect(),
            ),
        ),
    ];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Git, fields)
}

fn binary_creature_weapon_demo_git(
    fixtures: &[BinaryCreatureEquippedFixtureV1],
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![
        field(
            "AreaProperties",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 100,
                fields: [
                    "AmbientSndDay",
                    "AmbientSndNight",
                    "AmbientSndDayVol",
                    "AmbientSndNitVol",
                    "EnvAudio",
                    "MusicBattle",
                    "MusicDay",
                    "MusicNight",
                    "MusicDelay",
                ]
                .into_iter()
                .map(|label| field(label, GffValueV1::Int(0)))
                .collect(),
            }),
        ),
        field(
            "Creature List",
            GffValueV1::List(
                fixtures
                    .iter()
                    .map(binary_creature_equipped_fixture_git_creature)
                    .collect(),
            ),
        ),
    ];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Git, fields)
}

fn binary_creature_owned_fixture_git_creature(
    fixture: &BinaryCreatureOwnedFixtureV1,
) -> GffStructV1 {
    let mut creature = proof_git_creature(
        fixture.appearance_row,
        &fixture.template_resref,
        &fixture.display_name,
    );
    for item in &mut creature.fields {
        match item.label.as_str() {
            "XPosition" => item.value = GffValueV1::Float(fixture.position.x),
            "YPosition" => item.value = GffValueV1::Float(fixture.position.y),
            "ZPosition" => item.value = GffValueV1::Float(fixture.position.z),
            "XOrientation" => item.value = GffValueV1::Float(fixture.orientation.x),
            "YOrientation" => item.value = GffValueV1::Float(fixture.orientation.y),
            "Tag" => item.value = string(&fixture.id),
            _ => {}
        }
    }
    apply_binary_creature_runtime_complete_envelope(&mut creature.fields);
    creature
}

fn binary_creature_profiled_fixture_git_creature(
    fixture: &BinaryCreatureProfiledFixtureV2,
) -> GffStructV1 {
    let mut creature = binary_creature_owned_fixture_git_creature(&fixture.fixture);
    apply_binary_creature_runtime_profile(&mut creature.fields, fixture.runtime_profile);
    creature
}

fn binary_creature_equipped_fixture_git_creature(
    fixture: &BinaryCreatureEquippedFixtureV1,
) -> GffStructV1 {
    let mut creature = binary_creature_profiled_fixture_git_creature(&fixture.profiled_fixture);
    apply_binary_creature_equipment(
        &mut creature.fields,
        fixture.hand,
        &fixture.equipped_item_resref,
    );
    creature
}

fn apply_binary_creature_equipment(
    fields: &mut [GffFieldV1],
    hand: BinaryCreatureHandSlotV1,
    item_resref: &str,
) {
    if let Some(item) = fields
        .iter_mut()
        .find(|field| field.label == "Equip_ItemList")
    {
        item.value = GffValueV1::List(vec![GffStructV1 {
            struct_id: hand.native_struct_id(),
            fields: vec![field("EquippedRes", resref(item_resref))],
        }]);
    }
}

/// Applies the exact two-field normalization observed when Aurora saved the
/// generated diagnostic GIT. UTC materialization reuses this same creature
/// builder, so the instance and its module-local blueprint cannot diverge.
fn apply_binary_creature_runtime_complete_envelope(fields: &mut [GffFieldV1]) {
    for item in fields {
        match item.label.as_str() {
            "MaxHitPoints" => {
                item.value = GffValueV1::Short(BINARY_CREATURE_RUNTIME_MAX_HIT_POINTS)
            }
            "SkillList" => {
                item.value = GffValueV1::List(
                    (0..BINARY_CREATURE_RUNTIME_SKILL_COUNT)
                        .map(|_| GffStructV1 {
                            struct_id: 0,
                            fields: vec![field("Rank", GffValueV1::Byte(0))],
                        })
                        .collect(),
                )
            }
            _ => {}
        }
    }
}

fn binary_creature_owned_fixture_utc(
    fixture: &BinaryCreatureOwnedFixtureV1,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = binary_creature_owned_fixture_git_creature(fixture).fields;
    fields.drain(..5);
    fields.insert(0, field("PaletteID", GffValueV1::Byte(0)));
    fields.insert(
        1,
        field(
            "Comment",
            string(&format!(
                "Generated by Meshy2Aurora for owned fixture {}.",
                fixture.id
            )),
        ),
    );
    gff(GffFileTypeV1::Utc, fields)
}

fn binary_creature_profiled_fixture_utc(
    fixture: &BinaryCreatureProfiledFixtureV2,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = binary_creature_profiled_fixture_git_creature(fixture).fields;
    fields.drain(..5);
    fields.insert(0, field("PaletteID", GffValueV1::Byte(0)));
    fields.insert(
        1,
        field(
            "Comment",
            string(&format!(
                "Generated by Meshy2Aurora for owned profiled fixture {}.",
                fixture.fixture.id
            )),
        ),
    );
    gff(GffFileTypeV1::Utc, fields)
}

fn binary_creature_equipped_fixture_utc(
    fixture: &BinaryCreatureEquippedFixtureV1,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = binary_creature_equipped_fixture_git_creature(fixture).fields;
    fields.drain(..5);
    fields.insert(0, field("PaletteID", GffValueV1::Byte(0)));
    fields.insert(
        1,
        field(
            "Comment",
            string(&format!(
                "Generated by Meshy2Aurora for equipped fixture {}.",
                fixture.profiled_fixture.fixture.id
            )),
        ),
    );
    gff(GffFileTypeV1::Utc, fields)
}

fn binary_creature_weapon_uti(
    weapon: &BinaryCreatureWeaponItemV1,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let [part1, part2, part3] = weapon.model_parts;
    gff(
        GffFileTypeV1::Uti,
        vec![
            field("Description", loc("")),
            field("xModelPart3", GffValueV1::Word(part3.into())),
            field("ModelPart2", GffValueV1::Byte(part2)),
            field("xModelPart2", GffValueV1::Word(part2.into())),
            field("xModelPart1", GffValueV1::Word(part1.into())),
            field("ModelPart3", GffValueV1::Byte(part3)),
            field("PropertiesList", GffValueV1::List(Vec::new())),
            field(
                "Comment",
                string("Generated by Meshy2Aurora for the hand-anchor demo."),
            ),
            field("Charges", GffValueV1::Byte(0)),
            field("Plot", GffValueV1::Byte(0)),
            field("Cost", GffValueV1::Dword(30)),
            field("DescIdentified", loc("")),
            field("TemplateResRef", resref(&weapon.resref)),
            field("BaseItem", GffValueV1::Int(weapon.base_item)),
            field("Identified", GffValueV1::Byte(1)),
            field("PaletteID", GffValueV1::Byte(53)),
            field("Cursed", GffValueV1::Byte(0)),
            field("LocalizedName", loc(&weapon.display_name)),
            field("AddCost", GffValueV1::Dword(0)),
            field("StackSize", GffValueV1::Word(1)),
            field("Tag", string(&weapon.resref.to_ascii_uppercase())),
            field("Stolen", GffValueV1::Byte(0)),
            field("ModelPart1", GffValueV1::Byte(part1)),
        ],
    )
}

fn apply_binary_creature_runtime_profile(
    fields: &mut [GffFieldV1],
    profile: BinaryCreatureRuntimeProfileV2,
) {
    if profile == BinaryCreatureRuntimeProfileV2::LegacyMinimal {
        return;
    }

    for item in fields {
        item.value = match item.label.as_str() {
            "Race" => GffValueV1::Byte(7),
            "Gender" => GffValueV1::Byte(2),
            "PortraitId" => GffValueV1::Word(236),
            "FactionID" => GffValueV1::Word(
                if profile == BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline {
                    1
                } else {
                    2
                },
            ),
            "SoundSetFile" => GffValueV1::Word(53),
            "Interruptable" => GffValueV1::Byte(1),
            "Str" => GffValueV1::Byte(19),
            "Dex" => GffValueV1::Byte(15),
            "Con" => GffValueV1::Byte(17),
            "Int" => GffValueV1::Byte(6),
            "Wis" => GffValueV1::Byte(12),
            "Cha" => GffValueV1::Byte(6),
            "WalkRate" => GffValueV1::Int(7),
            "NaturalAC" => GffValueV1::Byte(6),
            "HitPoints" | "CurrentHitPoints" => GffValueV1::Short(22),
            "MaxHitPoints" => GffValueV1::Short(37),
            "LawfulChaotic" => GffValueV1::Byte(0),
            "ChallengeRating" => GffValueV1::Float(5.0),
            "PerceptionRange" => GffValueV1::Byte(11),
            "ClassList" => GffValueV1::List(vec![GffStructV1 {
                struct_id: 2,
                fields: vec![
                    field("Class", GffValueV1::Int(11)),
                    field("ClassLevel", GffValueV1::Short(5)),
                ],
            }]),
            label
                if profile == BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
                    && binary_creature_default_script(label).is_some() =>
            {
                resref(binary_creature_default_script(label).expect("guarded script label"))
            }
            _ => item.value.clone(),
        };
    }
}

fn binary_creature_default_script(label: &str) -> Option<&'static str> {
    match label {
        "ScriptHeartbeat" => Some("nw_c2_default1"),
        "ScriptOnNotice" => Some("nw_c2_default2"),
        "ScriptSpellAt" => Some("nw_c2_defaultb"),
        "ScriptAttacked" => Some("nw_c2_default5"),
        "ScriptDamaged" => Some("nw_c2_default6"),
        "ScriptDisturbed" => Some("nw_c2_default8"),
        "ScriptEndRound" => Some("nw_c2_default3"),
        "ScriptDialogue" => Some("nw_c2_default4"),
        "ScriptSpawn" => Some("nw_c2_default9"),
        "ScriptRested" => Some("nw_c2_defaulta"),
        "ScriptDeath" => Some("nw_c2_default7"),
        "ScriptUserDefine" => Some("nw_c2_defaultd"),
        "ScriptOnBlocked" => Some("nw_c2_defaulte"),
        _ => None,
    }
}

fn classify_binary_creature_runtime_profile(
    fields: &[GffFieldV1],
    fixture: &BinaryCreatureOwnedFixtureV1,
    path: &str,
) -> Result<BinaryCreatureRuntimeProfileV2, ProofModuleErrorV1> {
    let matches = [
        BinaryCreatureRuntimeProfileV2::LegacyMinimal,
        BinaryCreatureRuntimeProfileV2::PassiveMonsterBaseline,
        BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
    ]
    .into_iter()
    .filter(|profile| binary_creature_runtime_profile_matches(fields, fixture, *profile))
    .collect::<Vec<_>>();
    match matches.as_slice() {
        [profile] => Ok(*profile),
        [] => Err(binary_creature_profile_readback_error(
            path,
            "runtime fields do not exactly match any supported profile",
        )),
        _ => Err(binary_creature_profile_readback_error(
            path,
            "runtime fields ambiguously match more than one supported profile",
        )),
    }
}

fn binary_creature_runtime_profile_matches(
    fields: &[GffFieldV1],
    fixture: &BinaryCreatureOwnedFixtureV1,
    profile: BinaryCreatureRuntimeProfileV2,
) -> bool {
    let expected =
        binary_creature_profiled_fixture_git_creature(&BinaryCreatureProfiledFixtureV2 {
            fixture: fixture.clone(),
            runtime_profile: profile,
        });
    expected
        .fields
        .iter()
        .filter(|field| {
            !matches!(
                field.label.as_str(),
                "XPosition"
                    | "YPosition"
                    | "ZPosition"
                    | "XOrientation"
                    | "YOrientation"
                    | "TemplateResRef"
                    | "FirstName"
                    | "Appearance_Type"
                    | "Tag"
            )
        })
        .all(|expected_field| {
            fields
                .iter()
                .find(|field| field.label == expected_field.label)
                .is_some_and(|actual_field| actual_field.value == expected_field.value)
        })
}

#[derive(Clone, Copy)]
struct M0FixtureV1 {
    creature_resref: &'static str,
    display_name: &'static str,
    appearance_row: u16,
    x: f32,
    y: f32,
}

fn m0_proof_gic(fixtures: &[M0FixtureV1]) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![field(
        "Creature List",
        GffValueV1::List(
            fixtures
                .iter()
                .map(|fixture| GffStructV1 {
                    struct_id: 4,
                    fields: vec![field(
                        "Comment",
                        string(&format!(
                            "Generated by Codex: M0 fixture {} at ({}, {})",
                            fixture.creature_resref, fixture.x, fixture.y
                        )),
                    )],
                })
                .collect(),
        ),
    )];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Gic, fields)
}

fn m0_proof_git(fixtures: &[M0FixtureV1]) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![
        field(
            "AreaProperties",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 100,
                fields: [
                    "AmbientSndDay",
                    "AmbientSndNight",
                    "AmbientSndDayVol",
                    "AmbientSndNitVol",
                    "EnvAudio",
                    "MusicBattle",
                    "MusicDay",
                    "MusicNight",
                    "MusicDelay",
                ]
                .into_iter()
                .map(|label| field(label, GffValueV1::Int(0)))
                .collect(),
            }),
        ),
        field(
            "Creature List",
            GffValueV1::List(fixtures.iter().map(m0_proof_git_creature).collect()),
        ),
    ];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Git, fields)
}

fn m0_proof_git_creature(fixture: &M0FixtureV1) -> GffStructV1 {
    let mut creature = proof_git_creature(
        fixture.appearance_row,
        PROOF_CREATURE_RESREF,
        "canonical proof placeholder",
    );
    for item in &mut creature.fields {
        match item.label.as_str() {
            "XPosition" => item.value = GffValueV1::Float(fixture.x),
            "YPosition" => item.value = GffValueV1::Float(fixture.y),
            "TemplateResRef" => item.value = resref(fixture.creature_resref),
            "FirstName" => item.value = loc(fixture.display_name),
            "Tag" => item.value = string(fixture.creature_resref),
            _ => {}
        }
    }
    creature
}

fn m0_proof_utc(fixture: &M0FixtureV1) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = m0_proof_git_creature(fixture).fields;
    fields.drain(..5);
    fields.insert(0, field("PaletteID", GffValueV1::Byte(0)));
    fields.insert(
        1,
        field(
            "Comment",
            string(&format!(
                "Generated by Codex for M0 runtime fixture {}.",
                fixture.creature_resref
            )),
        ),
    );
    gff(GffFileTypeV1::Utc, fields)
}

fn binary_m0_git_for(
    appearance_row: u16,
    creature_display_name: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut creature = proof_git_creature(
        appearance_row,
        BINARY_M0_CREATURE_TEMPLATE_RESREF,
        creature_display_name,
    );
    for item in &mut creature.fields {
        match item.label.as_str() {
            "XPosition" => item.value = GffValueV1::Float(M0_RUNTIME_FIXTURE_X),
            "YPosition" => item.value = GffValueV1::Float(M0_RUNTIME_FIXTURE_Y),
            "ZPosition" => item.value = GffValueV1::Float(M0_RUNTIME_FIXTURE_Z),
            _ => {}
        }
    }
    let mut fields = vec![
        field(
            "AreaProperties",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 100,
                fields: [
                    "AmbientSndDay",
                    "AmbientSndNight",
                    "AmbientSndDayVol",
                    "AmbientSndNitVol",
                    "EnvAudio",
                    "MusicBattle",
                    "MusicDay",
                    "MusicNight",
                    "MusicDelay",
                ]
                .into_iter()
                .map(|label| field(label, GffValueV1::Int(0)))
                .collect(),
            }),
        ),
        field("Creature List", GffValueV1::List(vec![creature])),
    ];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Git, fields)
}

fn proof_gic_for(comment: &str) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![field(
        "Creature List",
        GffValueV1::List(vec![GffStructV1 {
            struct_id: 4,
            fields: vec![field("Comment", string(comment))],
        }]),
    )];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Gic, fields)
}

fn proof_git_for(
    appearance_row: u16,
    creature_resref: &str,
    creature_display_name: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![
        field(
            "AreaProperties",
            GffValueV1::Struct(GffStructV1 {
                struct_id: 100,
                fields: [
                    "AmbientSndDay",
                    "AmbientSndNight",
                    "AmbientSndDayVol",
                    "AmbientSndNitVol",
                    "EnvAudio",
                    "MusicBattle",
                    "MusicDay",
                    "MusicNight",
                    "MusicDelay",
                ]
                .into_iter()
                .map(|label| field(label, GffValueV1::Int(0)))
                .collect(),
            }),
        ),
        field(
            "Creature List",
            GffValueV1::List(vec![proof_git_creature(
                appearance_row,
                creature_resref,
                creature_display_name,
            )]),
        ),
    ];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Git, fields)
}

fn empty_area_instance_lists() -> Vec<GffFieldV1> {
    AREA_INSTANCE_LISTS
        .into_iter()
        .map(|label| field(label, GffValueV1::List(Vec::new())))
        .collect()
}

fn proof_git_creature(
    appearance_row: u16,
    creature_resref: &str,
    creature_display_name: &str,
) -> GffStructV1 {
    GffStructV1 {
        struct_id: 4,
        fields: vec![
            // Keep the generated creature clear of the Test Module player start
            // at (10, 10), so the runtime frame can show it independently.
            field("XPosition", GffValueV1::Float(14.0)),
            field("YPosition", GffValueV1::Float(10.0)),
            field("ZPosition", GffValueV1::Float(0.0)),
            field("XOrientation", GffValueV1::Float(1.0)),
            field("YOrientation", GffValueV1::Float(0.0)),
            field("TemplateResRef", resref(creature_resref)),
            field("Race", GffValueV1::Byte(0)),
            field("FirstName", loc(creature_display_name)),
            field("LastName", loc("")),
            field("Appearance_Type", GffValueV1::Word(appearance_row)),
            field("Gender", GffValueV1::Byte(0)),
            field("Phenotype", GffValueV1::Int(0)),
            field("PortraitId", GffValueV1::Word(0)),
            field("Description", loc("")),
            field("Tag", string(creature_resref)),
            field("Conversation", resref("")),
            field("IsPC", GffValueV1::Byte(0)),
            // Commoner is neutral to the player in the Toolset-authored
            // standard faction table emitted by `proof_factions`.
            field("FactionID", GffValueV1::Word(2)),
            field("Disarmable", GffValueV1::Byte(0)),
            field("Subrace", string("")),
            field("Deity", string("")),
            field("Wings_New", GffValueV1::Dword(0)),
            field("Tail_New", GffValueV1::Dword(0)),
            field("SoundSetFile", GffValueV1::Word(0)),
            field("Plot", GffValueV1::Byte(0)),
            field("IsImmortal", GffValueV1::Byte(0)),
            field("Interruptable", GffValueV1::Byte(0)),
            field("Lootable", GffValueV1::Byte(0)),
            field("NoPermDeath", GffValueV1::Byte(0)),
            field("BodyBag", GffValueV1::Byte(0)),
            field("StartingPackage", GffValueV1::Byte(0)),
            field("DecayTime", GffValueV1::Dword(0)),
            field("Str", GffValueV1::Byte(10)),
            field("Dex", GffValueV1::Byte(10)),
            field("Con", GffValueV1::Byte(10)),
            field("Int", GffValueV1::Byte(10)),
            field("Wis", GffValueV1::Byte(10)),
            field("Cha", GffValueV1::Byte(10)),
            field("WalkRate", GffValueV1::Int(0)),
            field("NaturalAC", GffValueV1::Byte(0)),
            field("HitPoints", GffValueV1::Short(1)),
            field("CurrentHitPoints", GffValueV1::Short(1)),
            field("MaxHitPoints", GffValueV1::Short(1)),
            field("refbonus", GffValueV1::Short(0)),
            field("willbonus", GffValueV1::Short(0)),
            field("fortbonus", GffValueV1::Short(0)),
            field("GoodEvil", GffValueV1::Byte(50)),
            field("LawfulChaotic", GffValueV1::Byte(50)),
            field("ChallengeRating", GffValueV1::Float(0.0)),
            field("CRAdjust", GffValueV1::Int(0)),
            field("PerceptionRange", GffValueV1::Byte(0)),
        ]
        .into_iter()
        .chain(
            [
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
            ]
            .into_iter()
            .map(|label| field(label, resref(""))),
        )
        .chain([
            field("SkillList", GffValueV1::List(Vec::new())),
            field("FeatList", GffValueV1::List(Vec::new())),
            field("TemplateList", GffValueV1::List(Vec::new())),
            field("SpecAbilityList", GffValueV1::List(Vec::new())),
            // The native runtime refuses a gameplay creature with an empty
            // ClassList. This simple non-caster child is the exact schema and
            // value read back from the Toolset-created proof creature.
            field(
                "ClassList",
                GffValueV1::List(vec![GffStructV1 {
                    struct_id: 2,
                    fields: vec![
                        field("Class", GffValueV1::Int(12)),
                        field("ClassLevel", GffValueV1::Short(12)),
                    ],
                }]),
            ),
            field("Equip_ItemList", GffValueV1::List(Vec::new())),
        ])
        .collect(),
    }
}

pub(crate) fn proof_factions() -> Result<Vec<u8>, ProofModuleErrorV1> {
    let faction_list = ["PC", "Hostile", "Commoner", "Merchant", "Defender"]
        .into_iter()
        .enumerate()
        .map(|(id, name)| GffStructV1 {
            struct_id: id as u32,
            fields: vec![
                field("FactionParentID", GffValueV1::Dword(u32::MAX)),
                field("FactionName", string(name)),
                field("FactionGlobal", GffValueV1::Word(1)),
            ],
        })
        .collect();
    let reputations = [
        (0, 1, 0),
        (0, 2, 50),
        (0, 3, 50),
        (0, 4, 50),
        (1, 1, 100),
        (1, 2, 0),
        (1, 3, 0),
        (1, 4, 0),
        (2, 1, 0),
        (2, 2, 100),
        (2, 3, 50),
        (2, 4, 100),
        (3, 1, 0),
        (3, 2, 50),
        (3, 3, 100),
        (3, 4, 100),
        (4, 1, 0),
        (4, 2, 50),
        (4, 3, 100),
        (4, 4, 100),
    ]
    .into_iter()
    .enumerate()
    .map(|(id, (first, second, reputation))| GffStructV1 {
        struct_id: id as u32,
        fields: vec![
            field("FactionID1", GffValueV1::Dword(first)),
            field("FactionID2", GffValueV1::Dword(second)),
            field("FactionRep", GffValueV1::Dword(reputation)),
        ],
    })
    .collect();
    gff(
        GffFileTypeV1::Fac,
        vec![
            field("FactionList", GffValueV1::List(faction_list)),
            field("RepList", GffValueV1::List(reputations)),
        ],
    )
}

fn proof_utc_for(
    appearance_row: u16,
    creature_resref: &str,
    creature_display_name: &str,
) -> Result<Vec<u8>, ProofModuleErrorV1> {
    // The UTC is an independently-authored creature blueprint, not a GIT
    // instance.  It shares the documented fields/types after the transform,
    // while omitting five placement coordinates and adding blueprint metadata.
    let mut fields =
        proof_git_creature(appearance_row, creature_resref, creature_display_name).fields;
    fields.drain(..5);
    fields.insert(0, field("PaletteID", GffValueV1::Byte(0)));
    fields.insert(
        1,
        field(
            "Comment",
            string("Generated by Codex for Meshy2Aurora H1 animation proof."),
        ),
    );
    gff(GffFileTypeV1::Utc, fields)
}

fn validate_binary_m0_vertical_slice_module_readback(
    bytes: &[u8],
    appearance_row: u16,
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<(), ProofModuleErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "binary_m0.archive", value.context))?;
    if archive.file_type() != ErfFileType::Module || archive.resources().len() != 6 {
        return Err(error(
            "M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF",
            "binary_m0.resources",
            "expected exactly one module, area triplet, faction table, and fixture UTC",
        ));
    }
    let ifo = read_gff_v32(
        archive
            .find("module", IFO_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "binary_m0.module", value.context))?,
        &Default::default(),
    )
    .map_err(|value| error(value.code, "binary_m0.module", value.message))?;
    let ifo_field = |label: &str| {
        ifo.root
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    if ifo_field("Mod_Entry_Area") != Some(&GffValueV1::ResRef(identity.area_resref.clone()))
        || ifo_field("Mod_Entry_X") != Some(&GffValueV1::Float(M0_RUNTIME_ENTRY_X))
        || ifo_field("Mod_Entry_Y") != Some(&GffValueV1::Float(M0_RUNTIME_ENTRY_Y))
        || ifo_field("Mod_Entry_Z") != Some(&GffValueV1::Float(M0_RUNTIME_ENTRY_Z))
        || ifo_field("Mod_Entry_Dir_X") != Some(&GffValueV1::Float(M0_RUNTIME_ENTRY_DIR_X))
        || ifo_field("Mod_Entry_Dir_Y") != Some(&GffValueV1::Float(M0_RUNTIME_ENTRY_DIR_Y))
        || !matches!(ifo_field("Mod_HakList"), Some(GffValueV1::List(values))
            if values.len() == 1
                && values[0].fields.iter().any(|field| field.label == "Mod_Hak"
                    && field.value == GffValueV1::String(identity.hak_resref.as_bytes().to_vec())))
    {
        return Err(error(
            "M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF",
            "binary_m0.module.ifo",
            "entry point or ordered HAK list differs",
        ));
    }

    let are = read_gff_v32(
        archive
            .find(&identity.area_resref, ARE_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "binary_m0.are", value.context))?,
        &Default::default(),
    )
    .map_err(|value| error(value.code, "binary_m0.are", value.message))?;
    let are_field = |label: &str| {
        are.root
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    let valid_tiles = matches!(are_field("Tile_List"), Some(GffValueV1::List(values))
        if values.len() == M0_RUNTIME_TILES.len()
            && values.iter().zip(M0_RUNTIME_TILES).all(|(tile, (expected_id, expected_orientation))|
                tile.fields.iter().any(|field| field.label == "Tile_ID" && field.value == GffValueV1::Int(expected_id))
                && tile.fields.iter().any(|field| field.label == "Tile_Orientation" && field.value == GffValueV1::Int(expected_orientation))
                && ["Tile_AnimLoop1", "Tile_AnimLoop2", "Tile_AnimLoop3"].iter().all(|label|
                    tile.fields.iter().any(|field| field.label == *label && field.value == GffValueV1::Byte(M0_RUNTIME_TILE_ANIMATION_LOOP)))));
    if are_field("Width") != Some(&GffValueV1::Int(M0_RUNTIME_AREA_WIDTH))
        || are_field("Height") != Some(&GffValueV1::Int(M0_RUNTIME_AREA_HEIGHT))
        || are_field("Tileset") != Some(&GffValueV1::ResRef(M0_RUNTIME_TILESET_RESREF.to_owned()))
        || !valid_tiles
    {
        return Err(error(
            "M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF",
            "binary_m0.are",
            "generated area does not match the owned valid 2x2 tile contract",
        ));
    }

    let git = read_gff_v32(
        archive
            .find(&identity.area_resref, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "binary_m0.git", value.context))?,
        &Default::default(),
    )
    .map_err(|value| error(value.code, "binary_m0.git", value.message))?;
    let creature = match git
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
    {
        Some(GffFieldV1 {
            value: GffValueV1::List(values),
            ..
        }) if values.len() == 1 => &values[0],
        _ => {
            return Err(error(
                "M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF",
                "binary_m0.git.Creature List",
                "expected one fixture",
            ));
        }
    };
    let creature_field = |label: &str| {
        creature
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    if creature_field("TemplateResRef")
        != Some(&GffValueV1::ResRef(
            BINARY_M0_CREATURE_TEMPLATE_RESREF.to_owned(),
        ))
        || creature_field("Appearance_Type") != Some(&GffValueV1::Word(appearance_row))
        || creature_field("XPosition") != Some(&GffValueV1::Float(M0_RUNTIME_FIXTURE_X))
        || creature_field("YPosition") != Some(&GffValueV1::Float(M0_RUNTIME_FIXTURE_Y))
        || creature_field("ZPosition") != Some(&GffValueV1::Float(M0_RUNTIME_FIXTURE_Z))
    {
        return Err(error(
            "M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF",
            "binary_m0.git.Creature List",
            "fixture template, appearance, or position differs",
        ));
    }
    Ok(())
}

fn validate_binary_m0_vertical_slice_identity(
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<(), ProofModuleErrorV1> {
    for (field_name, value) in [
        ("module_resref", identity.module_resref.as_str()),
        ("area_resref", identity.area_resref.as_str()),
        ("hak_resref", identity.hak_resref.as_str()),
    ] {
        if value.is_empty()
            || value.len() > 16
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(error(
                "M0-BINARY-VERTICAL-SLICE-IDENTITY-INVALID",
                format!("binary_m0.{field_name}"),
                "resref must contain 1..16 lowercase ASCII letters, digits, or underscores",
            ));
        }
    }
    Ok(())
}

fn validate_module_readback(
    bytes: &[u8],
    appearance_row: u16,
    creature_resref: &str,
) -> Result<(), ProofModuleErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    if archive.file_type() != ErfFileType::Module {
        return Err(error(
            "M6-PROOF-MODULE-SEMANTIC-DIFF",
            "module.signature",
            "expected MOD V1.0",
        ));
    }
    let expected = [
        ("module", IFO_RESOURCE_TYPE, GffFileTypeV1::Ifo),
        ("repute", FAC_RESOURCE_TYPE, GffFileTypeV1::Fac),
        (PROOF_AREA_RESREF, ARE_RESOURCE_TYPE, GffFileTypeV1::Are),
        (PROOF_AREA_RESREF, GIC_RESOURCE_TYPE, GffFileTypeV1::Gic),
        (PROOF_AREA_RESREF, GIT_RESOURCE_TYPE, GffFileTypeV1::Git),
        (creature_resref, UTC_RESOURCE_TYPE, GffFileTypeV1::Utc),
    ];
    if archive.resources().len() != expected.len() {
        return Err(error(
            "M6-PROOF-MODULE-SEMANTIC-DIFF",
            "module.resources",
            "unexpected resource count",
        ));
    }
    for (resref_value, resource_type, file_type) in expected {
        let payload = archive
            .find(resref_value, resource_type)
            .map_err(|value| error(value.code, format!("module.{resref_value}"), value.context))?;
        let document = read_gff_v32(payload, &Default::default())
            .map_err(|value| error(value.code, format!("module.{resref_value}"), value.message))?;
        if document.file_type != file_type {
            return Err(error(
                "M6-PROOF-MODULE-SEMANTIC-DIFF",
                format!("module.{resref_value}"),
                "GFF file type differs",
            ));
        }
        if resource_type == IFO_RESOURCE_TYPE {
            let hak_list = document
                .root
                .fields
                .iter()
                .find(|item| item.label == "Mod_HakList")
                .map(|item| &item.value);
            let expected_hak = GffValueV1::String(PROOF_HAK_RESREF.as_bytes().to_vec());
            if !matches!(
                hak_list,
                Some(GffValueV1::List(values))
                    if values.len() == 1
                        && values[0].struct_id == 8
                        && values[0].fields.len() == 1
                        && matches!(values[0].fields.first(), Some(field)
                            if field.label == "Mod_Hak" && field.value == expected_hak)
            ) {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "module.ifo.Mod_HakList",
                    "MOD must reference exactly the canonical Codex animation-proof HAK",
                ));
            }
            if document
                .root
                .fields
                .iter()
                .any(|item| item.label == "Mod_Hak")
            {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "module.ifo.Mod_Hak",
                    "MOD must not carry the legacy root Mod_Hak field",
                ));
            }
            let area_list = document
                .root
                .fields
                .iter()
                .find(|item| item.label == "Mod_Area_list")
                .map(|item| &item.value);
            if !matches!(
                area_list,
                Some(GffValueV1::List(values))
                    if values.len() == 1
                        && values[0].struct_id == 6
                        && values[0].fields.len() == 1
                        && matches!(values[0].fields.first(), Some(field)
                            if field.label == "Area_Name"
                                && field.value == GffValueV1::ResRef(PROOF_AREA_RESREF.to_owned()))
            ) {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "module.ifo.Mod_Area_list",
                    "MOD must contain the canonical area as struct id 6 / CResRef",
                ));
            }
        }
        if resource_type == GIC_RESOURCE_TYPE {
            if field_labels(&document.root) != GIC_ROOT_LISTS {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "gic.root",
                    "GIC must contain the exact ordered nine instance lists",
                ));
            }
            let Some(GffValueV1::List(creatures)) =
                document.root.fields.first().map(|field| &field.value)
            else {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "gic.Creature List",
                    "GIC Creature List is missing",
                ));
            };
            if creatures.len() != 1
                || creatures[0].struct_id != 4
                || creatures[0].fields.len() != 1
                || !matches!(creatures[0].fields.first(), Some(field)
                    if field.label == "Comment" && matches!(field.value, GffValueV1::String(_)))
            {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "gic.Creature List",
                    "GIC must align a struct id 4 creature comment with GIT",
                ));
            }
            if document
                .root
                .fields
                .iter()
                .skip(1)
                .any(|field| !matches!(&field.value, GffValueV1::List(values) if values.is_empty()))
            {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "gic.root",
                    "GIC non-creature instance lists must be empty in this proof module",
                ));
            }
        }
        if resource_type == GIT_RESOURCE_TYPE {
            if field_labels(&document.root) != GIT_ROOT_FIELDS {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "git.root",
                    "GIT must contain exact ordered AreaProperties and instance lists",
                ));
            }
            let Some(GffValueV1::Struct(area_properties)) =
                document.root.fields.first().map(|field| &field.value)
            else {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "git.AreaProperties",
                    "GIT AreaProperties is missing",
                ));
            };
            let expected_area_properties = [
                "AmbientSndDay",
                "AmbientSndNight",
                "AmbientSndDayVol",
                "AmbientSndNitVol",
                "EnvAudio",
                "MusicBattle",
                "MusicDay",
                "MusicNight",
                "MusicDelay",
            ];
            if area_properties.struct_id != 100
                || field_labels(area_properties) != expected_area_properties
                || area_properties
                    .fields
                    .iter()
                    .any(|field| !matches!(field.value, GffValueV1::Int(_)))
            {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "git.AreaProperties",
                    "GIT must retain the typed AreaProperties struct id 100",
                ));
            }
            let Some(GffValueV1::List(creatures)) =
                document.root.fields.get(1).map(|field| &field.value)
            else {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "git.Creature List",
                    "GIT Creature List is missing",
                ));
            };
            if creatures.len() != 1 || creatures[0].struct_id != 4 {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "git.Creature List",
                    "GIT must contain one struct id 4 proof creature",
                ));
            }
            let creature = &creatures[0];
            if creature
                .fields
                .iter()
                .find(|field| field.label == "TemplateResRef")
                .map(|field| &field.value)
                != Some(&GffValueV1::ResRef(creature_resref.to_owned()))
                || creature
                    .fields
                    .iter()
                    .find(|field| field.label == "Appearance_Type")
                    .map(|field| &field.value)
                    != Some(&GffValueV1::Word(appearance_row))
            {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "git.Creature List",
                    "GIT proof creature must reference the canonical UTC and appearance row",
                ));
            }
        }
        if resource_type == UTC_RESOURCE_TYPE {
            let value = document
                .root
                .fields
                .iter()
                .find(|item| item.label == "Appearance_Type")
                .map(|item| &item.value);
            if value != Some(&GffValueV1::Word(appearance_row)) {
                return Err(error(
                    "M6-PROOF-MODULE-SEMANTIC-DIFF",
                    "utc.Appearance_Type",
                    "UTC does not reference appended appearance row",
                ));
            }
        }
    }
    Ok(())
}

fn validate_m0_control_module_readback(
    bytes: &[u8],
    fixtures: &[M0FixtureV1],
    hak_resrefs: &[&str],
) -> Result<(), ProofModuleErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|value| error(value.code, "module.archive", value.context))?;
    if archive.file_type() != ErfFileType::Module {
        return Err(error(
            "M0-PROOF-MODULE-SEMANTIC-DIFF",
            "module.signature",
            "expected MOD V1.0",
        ));
    }
    let expected = [
        ("module", IFO_RESOURCE_TYPE, GffFileTypeV1::Ifo),
        ("repute", FAC_RESOURCE_TYPE, GffFileTypeV1::Fac),
        (M0_PROOF_AREA_RESREF, ARE_RESOURCE_TYPE, GffFileTypeV1::Are),
        (M0_PROOF_AREA_RESREF, GIC_RESOURCE_TYPE, GffFileTypeV1::Gic),
        (M0_PROOF_AREA_RESREF, GIT_RESOURCE_TYPE, GffFileTypeV1::Git),
        (
            M0_CONTROL_CREATURE_RESREF,
            UTC_RESOURCE_TYPE,
            GffFileTypeV1::Utc,
        ),
        (
            M0_PROOF_CREATURE_RESREF,
            UTC_RESOURCE_TYPE,
            GffFileTypeV1::Utc,
        ),
    ];
    if archive.resources().len() != expected.len() {
        return Err(error(
            "M0-PROOF-MODULE-SEMANTIC-DIFF",
            "module.resources",
            "M0 MOD must contain exactly the two fixtures and core area resources",
        ));
    }
    for (resref_value, resource_type, file_type) in expected {
        let payload = archive
            .find(resref_value, resource_type)
            .map_err(|value| error(value.code, format!("module.{resref_value}"), value.context))?;
        let document = read_gff_v32(payload, &Default::default())
            .map_err(|value| error(value.code, format!("module.{resref_value}"), value.message))?;
        if document.file_type != file_type {
            return Err(error(
                "M0-PROOF-MODULE-SEMANTIC-DIFF",
                format!("module.{resref_value}"),
                "GFF file type differs",
            ));
        }
    }
    let ifo = read_gff_v32(
        archive
            .find("module", IFO_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "module.ifo", value.context))?,
        &Default::default(),
    )
    .map_err(|value| error(value.code, "module.ifo", value.message))?;
    let Some(GffValueV1::List(haks)) = ifo
        .root
        .fields
        .iter()
        .find(|field| field.label == "Mod_HakList")
        .map(|field| &field.value)
    else {
        return Err(error(
            "M0-PROOF-MODULE-SEMANTIC-DIFF",
            "module.ifo.Mod_HakList",
            "M0 MOD must have an ordered HAK list",
        ));
    };
    if haks.len() != hak_resrefs.len()
        || !haks.iter().zip(hak_resrefs).all(|(entry, expected_hak)| {
            entry.struct_id == 8
                && entry.fields.len() == 1
                && matches!(entry.fields.first(), Some(field)
                    if field.label == "Mod_Hak"
                        && field.value == GffValueV1::String(expected_hak.as_bytes().to_vec()))
        })
    {
        return Err(error(
            "M0-PROOF-MODULE-SEMANTIC-DIFF",
            "module.ifo.Mod_HakList",
            "M0 MOD must list the external tortoise HAK before the generated M0 HAK",
        ));
    }
    let git = read_gff_v32(
        archive
            .find(M0_PROOF_AREA_RESREF, GIT_RESOURCE_TYPE)
            .map_err(|value| error(value.code, "module.git", value.context))?,
        &Default::default(),
    )
    .map_err(|value| error(value.code, "module.git", value.message))?;
    let Some(GffValueV1::List(creatures)) = git
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .map(|field| &field.value)
    else {
        return Err(error(
            "M0-PROOF-MODULE-SEMANTIC-DIFF",
            "git.Creature List",
            "M0 GIT must contain both fixtures",
        ));
    };
    if creatures.len() != fixtures.len() {
        return Err(error(
            "M0-PROOF-MODULE-SEMANTIC-DIFF",
            "git.Creature List",
            "M0 GIT fixture count differs",
        ));
    }
    for (creature, fixture) in creatures.iter().zip(fixtures) {
        let expected_resref = GffValueV1::ResRef(fixture.creature_resref.to_owned());
        if creature.struct_id != 4
            || creature
                .fields
                .iter()
                .find(|field| field.label == "TemplateResRef")
                .map(|field| &field.value)
                != Some(&expected_resref)
            || creature
                .fields
                .iter()
                .find(|field| field.label == "Appearance_Type")
                .map(|field| &field.value)
                != Some(&GffValueV1::Word(fixture.appearance_row))
            || creature
                .fields
                .iter()
                .find(|field| field.label == "XPosition")
                .map(|field| &field.value)
                != Some(&GffValueV1::Float(fixture.x))
            || creature
                .fields
                .iter()
                .find(|field| field.label == "YPosition")
                .map(|field| &field.value)
                != Some(&GffValueV1::Float(fixture.y))
        {
            return Err(error(
                "M0-PROOF-MODULE-SEMANTIC-DIFF",
                "git.Creature List",
                "M0 fixture identity, appearance row or position differs",
            ));
        }
    }
    for fixture in fixtures {
        let utc = read_gff_v32(
            archive
                .find(fixture.creature_resref, UTC_RESOURCE_TYPE)
                .map_err(|value| error(value.code, "module.utc", value.context))?,
            &Default::default(),
        )
        .map_err(|value| error(value.code, "module.utc", value.message))?;
        if utc
            .root
            .fields
            .iter()
            .find(|field| field.label == "Appearance_Type")
            .map(|field| &field.value)
            != Some(&GffValueV1::Word(fixture.appearance_row))
        {
            return Err(error(
                "M0-PROOF-MODULE-SEMANTIC-DIFF",
                "utc.Appearance_Type",
                "M0 fixture UTC does not reference its expected appearance row",
            ));
        }
    }
    Ok(())
}

fn resource(resref_value: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref_value.to_owned(),
        resource_type,
        payload,
    }
}

fn gff(file_type: GffFileTypeV1, fields: Vec<GffFieldV1>) -> Result<Vec<u8>, ProofModuleErrorV1> {
    write_gff_v32(
        &GffDocumentV1 {
            schema_version: 1,
            file_type,
            root: GffStructV1 {
                struct_id: u32::MAX,
                fields,
            },
        },
        &GffWriterOptionsV1::default(),
    )
    .map(|artifact| artifact.payload)
    .map_err(|value| error(value.code, "module.gff", value.message))
}

fn field(label: &str, value: GffValueV1) -> GffFieldV1 {
    GffFieldV1 {
        label: label.to_owned(),
        value,
    }
}

fn field_labels(structure: &GffStructV1) -> Vec<&str> {
    structure
        .fields
        .iter()
        .map(|field| field.label.as_str())
        .collect()
}

fn resref(value: &str) -> GffValueV1 {
    GffValueV1::ResRef(value.to_owned())
}
fn string(value: &str) -> GffValueV1 {
    GffValueV1::String(value.as_bytes().to_vec())
}
fn loc(value: &str) -> GffValueV1 {
    GffValueV1::LocString(GffLocStringV1 {
        string_ref: u32::MAX,
        substrings: vec![GffLocSubstringV1 {
            string_id: 0,
            bytes: value.as_bytes().to_vec(),
        }],
    })
}
fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProofModuleErrorV1 {
    ProofModuleErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_demo_owns_uti_and_matches_git_to_utc_hand_slots() {
        let identity = BinaryCreatureModuleIdentityV1 {
            module_resref: "m2aweapdemo1".to_owned(),
            area_resref: "m2aweaparea1".to_owned(),
            hak_resref: "m2aweaphak1".to_owned(),
        };
        let weapon = BinaryCreatureWeaponItemV1::owned_longsword("m2aweapitem1");
        let make_fixture = |id: &str, template: &str, hand: BinaryCreatureHandSlotV1, x: f32| {
            BinaryCreatureEquippedFixtureV1 {
                profiled_fixture: BinaryCreatureProfiledFixtureV2 {
                    fixture: BinaryCreatureOwnedFixtureV1 {
                        id: id.to_owned(),
                        template_resref: template.to_owned(),
                        display_name: format!("{hand:?} attachment"),
                        appearance_row: 15_100,
                        position: M0RuntimePositionV1 { x, y: 14.5, z: 0.0 },
                        orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
                    },
                    runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
                },
                hand,
                equipped_item_resref: weapon.resref.clone(),
            }
        };
        let fixtures = vec![
            make_fixture(
                "m2a_right_hand",
                "m2awrhand1",
                BinaryCreatureHandSlotV1::RightHand,
                7.5,
            ),
            make_fixture(
                "m2a_left_hand",
                "m2awlhand1",
                BinaryCreatureHandSlotV1::LeftHand,
                12.5,
            ),
        ];
        let artifact =
            build_binary_creature_weapon_demo_module_v1(&identity, &fixtures, &weapon).unwrap();
        assert_eq!(artifact.readback.weapon, weapon);
        assert_eq!(artifact.readback.fixtures, fixtures);
        let archive = ErfArchive::parse(&artifact.payload).unwrap();
        assert_eq!(archive.resources().len(), 8);
        assert_eq!(
            read_gff_v32(
                archive.find("m2aweapitem1", UTI_RESOURCE_TYPE).unwrap(),
                &Default::default(),
            )
            .unwrap()
            .file_type,
            GffFileTypeV1::Uti
        );
        assert!(inspect_binary_creature_multi_fixture_module_v1(&artifact.payload).is_err());
    }

    #[test]
    fn stock_weapon_demo_v2_has_real_equipment_reference_and_no_synthetic_uti() {
        let identity = BinaryCreatureModuleIdentityV1 {
            module_resref: "m2aweapdemo2".to_owned(),
            area_resref: "m2aweaparea2".to_owned(),
            hak_resref: "m2aweaphak2".to_owned(),
        };
        let weapon = BinaryCreatureStockWeaponV2::nwn_base_shortsword();
        let fixtures = vec![BinaryCreatureEquippedFixtureV1 {
            profiled_fixture: BinaryCreatureProfiledFixtureV2 {
                fixture: BinaryCreatureOwnedFixtureV1 {
                    id: "m2a_weapon_right_v2".to_owned(),
                    template_resref: "m2awrhand2".to_owned(),
                    display_name: "RIGHT HAND - native stock sword - rhand".to_owned(),
                    appearance_row: 15_100,
                    position: M0RuntimePositionV1 {
                        x: 10.0,
                        y: 14.5,
                        z: 0.0,
                    },
                    orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
                },
                runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
            },
            hand: BinaryCreatureHandSlotV1::RightHand,
            equipped_item_resref: weapon.resref.clone(),
        }];

        let artifact =
            build_binary_creature_stock_weapon_demo_module_v2(&identity, &fixtures, &weapon)
                .unwrap();
        assert_eq!(artifact.readback.schema_version, 2);
        assert_eq!(artifact.readback.weapon, weapon);
        assert_eq!(artifact.readback.fixtures, fixtures);
        assert_eq!(artifact.readback.fixtures[0].hand.native_struct_id(), 16);
        assert_eq!(
            artifact.readback.fixtures[0].equipped_item_resref,
            NWN_BASE_SHORTSWORD_RESREF_V2
        );
        let archive = ErfArchive::parse(&artifact.payload).unwrap();
        assert_eq!(archive.resources().len(), 6);
        assert!(
            archive
                .resources()
                .iter()
                .all(|resource| resource.resource_type != UTI_RESOURCE_TYPE)
        );
        assert_eq!(
            inspect_binary_creature_stock_weapon_demo_module_v2(&artifact.payload, &weapon)
                .unwrap()
                .fixtures,
            fixtures
        );

        let mut unverified = weapon;
        unverified.resref = "not_in_base_key".to_owned();
        let error = build_binary_creature_stock_weapon_demo_module_v2(
            &identity,
            &artifact.readback.fixtures,
            &unverified,
        )
        .unwrap_err();
        assert_eq!(error.path, "weapon");
    }

    #[test]
    fn generated_module_contains_self_owned_utc_git_and_hak_reference() {
        let artifact = build_creature_proof_module_v1(15_100).unwrap();
        assert_eq!(PROOF_MODULE_RESREF, "m2a_codex_aproof");
        assert_eq!(PROOF_HAK_RESREF, "m2a_codex_aproof");
        assert_eq!(artifact.report.module_resref, PROOF_MODULE_RESREF);
        assert_eq!(artifact.report.hak_resref, PROOF_HAK_RESREF);
        assert_eq!(artifact.report.semantic_readback_status, "PASS");
        assert_eq!(artifact.report.resource_count, 6);
        let archive = ErfArchive::parse(&artifact.payload).unwrap();
        assert_eq!(archive.file_type(), ErfFileType::Module);
        let ifo = read_gff_v32(
            archive.find("module", IFO_RESOURCE_TYPE).unwrap(),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(ifo.root.fields.len(), 55, "IFO55 manifest must be complete");
        assert!(matches!(
            ifo.root.fields.first(),
            Some(field) if field.label == "Mod_ID" && field.value == GffValueV1::Void(vec![0; 16])
        ));
        assert!(matches!(
            ifo.root.fields.get(10),
            Some(field) if field.label == "Mod_Entry_Area"
                && field.value == GffValueV1::ResRef(PROOF_AREA_RESREF.to_owned())
        ));
        let hak_list = ifo
            .root
            .fields
            .iter()
            .find(|field| field.label == "Mod_HakList");
        assert!(
            matches!(hak_list.map(|field| &field.value), Some(GffValueV1::List(values)) if values.len() == 1)
        );
        assert!(matches!(
            hak_list.map(|field| &field.value),
            Some(GffValueV1::List(values))
                if values[0].struct_id == 8
                    && values[0].fields.len() == 1
                    && matches!(values[0].fields.first(), Some(field)
                        if field.label == "Mod_Hak"
                            && field.value == GffValueV1::String(PROOF_HAK_RESREF.as_bytes().to_vec()))
        ));
        assert!(ifo.root.fields.iter().all(|field| field.label != "Mod_Hak"));
        let utc = read_gff_v32(
            archive
                .find(PROOF_CREATURE_RESREF, UTC_RESOURCE_TYPE)
                .unwrap(),
            &Default::default(),
        )
        .unwrap();
        assert!(utc.root.fields.iter().any(
            |field| field.label == "Appearance_Type" && field.value == GffValueV1::Word(15_100)
        ));
        assert!(
            utc.root
                .fields
                .iter()
                .any(|field| field.label == "Race" && field.value == GffValueV1::Byte(0))
        );
        assert!(
            utc.root
                .fields
                .iter()
                .any(|field| field.label == "refbonus" && field.value == GffValueV1::Short(0))
        );
        assert!(matches!(
            utc.root.fields.iter().find(|field| field.label == "ClassList").map(|field| &field.value),
            Some(GffValueV1::List(values))
                if values.len() == 1
                    && values[0].struct_id == 2
                    && values[0].fields == vec![
                        field("Class", GffValueV1::Int(12)),
                        field("ClassLevel", GffValueV1::Short(12)),
                    ]
        ));
        assert!(
            utc.root
                .fields
                .iter()
                .any(|field| field.label == "FactionID" && field.value == GffValueV1::Word(2))
        );
        let factions = read_gff_v32(
            archive.find("repute", FAC_RESOURCE_TYPE).unwrap(),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(factions.file_type, GffFileTypeV1::Fac);
        assert!(matches!(
            factions.root.fields.iter().find(|field| field.label == "FactionList").map(|field| &field.value),
            Some(GffValueV1::List(values))
                if values.len() == 5
                    && values[2].struct_id == 2
                    && values[2].fields.iter().any(|field|
                        field.label == "FactionName"
                            && field.value == GffValueV1::String(b"Commoner".to_vec()))
        ));
        assert!(matches!(
            factions.root.fields.iter().find(|field| field.label == "RepList").map(|field| &field.value),
            Some(GffValueV1::List(values)) if values.len() == 20
        ));
        let are = read_gff_v32(
            archive.find(PROOF_AREA_RESREF, ARE_RESOURCE_TYPE).unwrap(),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(are.root.fields.len(), 43, "ARE43 manifest must be complete");
        assert!(matches!(
            are.root.fields.first(),
            Some(field) if field.label == "ID" && field.value == GffValueV1::Int(0)
        ));
        assert!(matches!(
            are.root.fields.get(34),
            Some(field) if field.label == "Width" && field.value == GffValueV1::Int(8)
        ));
        let tile_list = are
            .root
            .fields
            .iter()
            .find(|field| field.label == "Tile_List");
        assert!(are.root.fields.iter().any(|field| field.label == "Tileset"
            && field.value == GffValueV1::ResRef("tin01".to_owned())));
        assert!(matches!(
            tile_list.map(|field| &field.value),
            Some(GffValueV1::List(values))
                if values.len() == 64
                    && values.iter().all(|tile|
                        tile.struct_id == 1
                            && tile.fields.iter().any(|field|
                                field.label == "Tile_ID" && field.value == GffValueV1::Int(94))
                            && tile.fields.iter().any(|field|
                                field.label == "Tile_Orientation" && matches!(field.value, GffValueV1::Int(0..=3))))
                    && values[0].fields.iter().any(|field|
                        field.label == "Tile_Height" && field.value == GffValueV1::Int(0))
                    && {
                        let runtime_bytes = values[0].fields.iter().filter(|field|
                            matches!(field.label.as_str(),
                                "Tile_MainLight1" | "Tile_MainLight2" | "Tile_SrcLight1" | "Tile_SrcLight2" |
                                "Tile_AnimLoop1" | "Tile_AnimLoop2" | "Tile_AnimLoop3")
                        ).collect::<Vec<_>>();
                        runtime_bytes.len() == 7
                            && runtime_bytes.iter().any(|field| field.value != GffValueV1::Byte(0))
                    }
        ));
        assert!(
            are.root.fields.iter().any(
                |field| field.label == "SunAmbientColor" && field.value == GffValueV1::Dword(0)
            )
        );
        assert!(
            are.root.fields.iter().any(
                |field| field.label == "SunDiffuseColor" && field.value == GffValueV1::Dword(0)
            )
        );
        let git = read_gff_v32(
            archive.find(PROOF_AREA_RESREF, GIT_RESOURCE_TYPE).unwrap(),
            &Default::default(),
        )
        .unwrap();
        assert!(
            git.root
                .fields
                .iter()
                .any(|field| field.label == "Creature List")
        );
        let creature = git
            .root
            .fields
            .iter()
            .find(|field| field.label == "Creature List")
            .and_then(|field| match &field.value {
                GffValueV1::List(values) => values.first(),
                _ => None,
            })
            .expect("one proof creature");
        assert!(
            creature
                .fields
                .iter()
                .any(|field| field.label == "XPosition" && field.value == GffValueV1::Float(14.0))
        );
        assert!(
            creature
                .fields
                .iter()
                .any(|field| field.label == "YPosition" && field.value == GffValueV1::Float(10.0))
        );
    }

    /// Environment-gated Aurora-first readback.  It reads a user-selected
    /// reference ARE in place through the own parser and prints only the
    /// environment/tile contract needed to author an independent proof area.
    /// It never packages or copies reference payload into this repository.
    #[test]
    fn selected_reference_are_reports_environment_profile_without_copying_payload() {
        let Some(path) = std::env::var_os("M2A_REFERENCE_ARE_FILE") else {
            eprintln!("skipped: M2A_REFERENCE_ARE_FILE is not set");
            return;
        };
        let bytes = std::fs::read(path).expect("env-selected reference ARE must be readable");
        let document = read_gff_v32(&bytes, &Default::default())
            .expect("env-selected reference ARE must parse with the own reader");
        assert_eq!(document.file_type, GffFileTypeV1::Are);
        for label in [
            "Tileset",
            "MoonAmbientColor",
            "MoonDiffuseColor",
            "MoonFogAmount",
            "MoonFogColor",
            "MoonShadows",
            "SunAmbientColor",
            "SunDiffuseColor",
            "SunFogAmount",
            "SunFogColor",
            "SunShadows",
            "IsNight",
            "LightingScheme",
            "ShadowOpacity",
            "FogClipDist",
            "SkyBox",
            "DayNightCycle",
            "LoadScreenID",
            "Width",
            "Height",
            "Tile_List",
        ] {
            let value = document
                .root
                .fields
                .iter()
                .find(|field| field.label == label)
                .map(|field| &field.value);
            eprintln!("{label}={value:?}");
        }
        let tiles = document
            .root
            .fields
            .iter()
            .find(|field| field.label == "Tile_List")
            .and_then(|field| match &field.value {
                GffValueV1::List(values) => Some(values),
                _ => None,
            })
            .expect("reference ARE Tile_List must be a list");
        let first = tiles.first().expect("reference ARE must contain a tile");
        eprintln!("first_tile_struct_id={}", first.struct_id);
        for field in &first.fields {
            eprintln!("first_tile.{}={:?}", field.label, field.value);
        }
    }
}
