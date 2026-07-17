//! Generated, self-contained creature proof module.
//!
//! This is intentionally a structural Aurora proof artefact, not a claim that
//! a particular Toolset/game session has loaded it.  The resource family and
//! field labels are derived from the local Aurora audit: `module.ifo`, the
//! `ARE`/`GIC`/`GIT` area triplet, a creature `UTC`, `Mod_HakList`, and the
//! exact `Creature List` placement fields.  No retail resource payload is
//! copied into this archive.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLocStringV1, GffLocSubstringV1, GffStructV1,
        GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
    },
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1},
};

/// The sole canonical runtime-test module. Its 16-byte resref identifies
/// both its Codex provenance and its strictly limited animation-proof role.
pub const PROOF_MODULE_RESREF: &str = "m2a_codex_aproof";
pub const PROOF_AREA_RESREF: &str = "m2a_caproof_area";
pub const PROOF_CREATURE_RESREF: &str = "m2a_caproof_h1";
pub const PROOF_HAK_RESREF: &str = "m2a_codex_aproof";

const IFO_RESOURCE_TYPE: u16 = 2014;
const ARE_RESOURCE_TYPE: u16 = 2012;
const GIC_RESOURCE_TYPE: u16 = 2046;
const GIT_RESOURCE_TYPE: u16 = 2023;
const UTC_RESOURCE_TYPE: u16 = 2027;
const FAC_RESOURCE_TYPE: u16 = 2038;

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofModuleReportV1 {
    pub schema_version: u32,
    pub module_resref: String,
    pub area_resref: String,
    pub creature_resref: String,
    pub hak_resref: String,
    pub appearance_row: u16,
    pub resource_count: u32,
    pub byte_length: u64,
    pub sha256: String,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofModuleArtifactV1 {
    pub payload: Vec<u8>,
    pub report: ProofModuleReportV1,
}

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
    let resources = vec![
        resource("module", IFO_RESOURCE_TYPE, module_ifo()?),
        resource("repute", FAC_RESOURCE_TYPE, proof_factions()?),
        resource(PROOF_AREA_RESREF, ARE_RESOURCE_TYPE, proof_area()?),
        resource(PROOF_AREA_RESREF, GIC_RESOURCE_TYPE, proof_gic()?),
        resource(
            PROOF_AREA_RESREF,
            GIT_RESOURCE_TYPE,
            proof_git(appearance_row)?,
        ),
        resource(
            PROOF_CREATURE_RESREF,
            UTC_RESOURCE_TYPE,
            proof_utc(appearance_row)?,
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
    validate_module_readback(&archive.payload, appearance_row)?;
    Ok(ProofModuleArtifactV1 {
        report: ProofModuleReportV1 {
            schema_version: 1,
            module_resref: PROOF_MODULE_RESREF.to_owned(),
            area_resref: PROOF_AREA_RESREF.to_owned(),
            creature_resref: PROOF_CREATURE_RESREF.to_owned(),
            hak_resref: PROOF_HAK_RESREF.to_owned(),
            appearance_row,
            resource_count: resources.len() as u32,
            byte_length: archive.payload.len() as u64,
            sha256: sha256(&archive.payload),
            semantic_readback_status: "PASS".to_owned(),
        },
        payload: archive.payload,
    })
}

fn module_ifo() -> Result<Vec<u8>, ProofModuleErrorV1> {
    // IFO55 is the frozen Aurora manifest.  A sparse IFO can be parsed by our
    // reader yet leaves NWN traversing absent labels during module startup.
    let mut fields = vec![
        field("Mod_ID", GffValueV1::Void(vec![0; 16])),
        field("Mod_MinGameVer", string("1.69")),
        field("Mod_Creator_ID", GffValueV1::Int(0)),
        field("Mod_Version", GffValueV1::Dword(3)),
        field("Expansion_Pack", GffValueV1::Word(0)),
        field(
            "Mod_Name",
            loc("Meshy2Aurora Codex animation proof (single test module)"),
        ),
        field("Mod_Tag", string(PROOF_MODULE_RESREF)),
        field(
            "Mod_Description",
            loc("Generated by Codex for Meshy2Aurora H1 animation proof."),
        ),
        field("Mod_IsSaveGame", GffValueV1::Byte(0)),
        field("Mod_CustomTlk", string("")),
        field("Mod_Entry_Area", resref(PROOF_AREA_RESREF)),
        field("Mod_Entry_X", GffValueV1::Float(5.0)),
        field("Mod_Entry_Y", GffValueV1::Float(5.0)),
        field("Mod_Entry_Z", GffValueV1::Float(0.0)),
        field("Mod_Entry_Dir_X", GffValueV1::Float(1.0)),
        field("Mod_Entry_Dir_Y", GffValueV1::Float(0.0)),
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
                fields: vec![field("Area_Name", resref(PROOF_AREA_RESREF))],
            }]),
        ),
        field(
            "Mod_HakList",
            GffValueV1::List(vec![GffStructV1 {
                struct_id: 8,
                fields: vec![field("Mod_Hak", string(PROOF_HAK_RESREF))],
            }]),
        ),
    ]);
    gff(GffFileTypeV1::Ifo, fields)
}

fn proof_area() -> Result<Vec<u8>, ProofModuleErrorV1> {
    gff(
        GffFileTypeV1::Are,
        vec![
            field("ID", GffValueV1::Int(0)),
            field("Creator_ID", GffValueV1::Int(0)),
            field("Version", GffValueV1::Dword(1)),
            field("Tag", string(PROOF_AREA_RESREF)),
            field("Name", loc("Codex H1 animation proof area")),
            field("ResRef", resref(PROOF_AREA_RESREF)),
            field(
                "Comments",
                string("Generated by Codex for Meshy2Aurora H1 animation proof."),
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
            // Aurora Toolset accepted the native `tdc01` filler tile 5 at
            // orientation 0 as an independent tile.  Unlike the earlier
            // `ttr01` tile 139 experiment, it is not a partial multi-tile
            // group and therefore remains valid in this one-tile proof area.
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
            field("Width", GffValueV1::Int(2)),
            field("Height", GffValueV1::Int(2)),
            field("OnEnter", resref("")),
            field("OnExit", resref("")),
            field("OnHeartbeat", resref("")),
            field("OnUserDefined", resref("")),
            field("TileBrdrDisabled", GffValueV1::Byte(0)),
            field("Tileset", resref("tdc01")),
            field(
                "Tile_List",
                GffValueV1::List(vec![
                    proof_tile(113, 2),
                    proof_tile(113, 3),
                    proof_tile(113, 1),
                    proof_tile(0, 0),
                ]),
            ),
        ],
    )
}

fn proof_tile(tile_id: i32, orientation: i32) -> GffStructV1 {
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
            field("Tile_AnimLoop1", GffValueV1::Byte(0)),
            field("Tile_AnimLoop2", GffValueV1::Byte(0)),
            field("Tile_AnimLoop3", GffValueV1::Byte(0)),
        ],
    }
}

fn proof_gic() -> Result<Vec<u8>, ProofModuleErrorV1> {
    let mut fields = vec![field(
        "Creature List",
        GffValueV1::List(vec![GffStructV1 {
            struct_id: 4,
            fields: vec![field(
                "Comment",
                string("Generated by Codex: canonical H1 animation proof placement"),
            )],
        }]),
    )];
    fields.extend(empty_area_instance_lists());
    gff(GffFileTypeV1::Gic, fields)
}

fn proof_git(appearance_row: u16) -> Result<Vec<u8>, ProofModuleErrorV1> {
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
            GffValueV1::List(vec![proof_git_creature(appearance_row)]),
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

fn proof_git_creature(appearance_row: u16) -> GffStructV1 {
    GffStructV1 {
        struct_id: 4,
        fields: vec![
            field("XPosition", GffValueV1::Float(10.0)),
            field("YPosition", GffValueV1::Float(10.0)),
            field("ZPosition", GffValueV1::Float(0.0)),
            field("XOrientation", GffValueV1::Float(1.0)),
            field("YOrientation", GffValueV1::Float(0.0)),
            field("TemplateResRef", resref(PROOF_CREATURE_RESREF)),
            field("Race", GffValueV1::Byte(0)),
            field("FirstName", loc("Codex Meshy H1 animation proof creature")),
            field("LastName", loc("")),
            field("Appearance_Type", GffValueV1::Word(appearance_row)),
            field("Gender", GffValueV1::Byte(0)),
            field("Phenotype", GffValueV1::Int(0)),
            field("PortraitId", GffValueV1::Word(0)),
            field("Description", loc("")),
            field("Tag", string(PROOF_CREATURE_RESREF)),
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

fn proof_factions() -> Result<Vec<u8>, ProofModuleErrorV1> {
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

fn proof_utc(appearance_row: u16) -> Result<Vec<u8>, ProofModuleErrorV1> {
    // The UTC is an independently-authored creature blueprint, not a GIT
    // instance.  It shares the documented fields/types after the transform,
    // while omitting five placement coordinates and adding blueprint metadata.
    let mut fields = proof_git_creature(appearance_row).fields;
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

fn validate_module_readback(bytes: &[u8], appearance_row: u16) -> Result<(), ProofModuleErrorV1> {
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
        (PROOF_CREATURE_RESREF, UTC_RESOURCE_TYPE, GffFileTypeV1::Utc),
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
                != Some(&GffValueV1::ResRef(PROOF_CREATURE_RESREF.to_owned()))
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
            Some(field) if field.label == "Width" && field.value == GffValueV1::Int(2)
        ));
        let tile_list = are
            .root
            .fields
            .iter()
            .find(|field| field.label == "Tile_List");
        assert!(are.root.fields.iter().any(|field| field.label == "Tileset"
            && field.value == GffValueV1::ResRef("tdc01".to_owned())));
        assert!(matches!(
            tile_list.map(|field| &field.value),
            Some(GffValueV1::List(values))
                if values.len() == 4
                    && [(113, 2), (113, 3), (113, 1), (0, 0)]
                        .iter()
                        .enumerate()
                        .all(|(index, (tile_id, orientation))|
                            values[index].struct_id == 1
                                && values[index].fields.iter().any(|field|
                                    field.label == "Tile_ID"
                                        && field.value == GffValueV1::Int(*tile_id))
                                && values[index].fields.iter().any(|field|
                                    field.label == "Tile_Orientation"
                                        && field.value == GffValueV1::Int(*orientation)))
                    && values[0].fields.iter().any(|field|
                        field.label == "Tile_Height" && field.value == GffValueV1::Int(0))
                    && {
                        let runtime_bytes = values[0].fields.iter().filter(|field|
                            matches!(field.label.as_str(),
                                "Tile_MainLight1" | "Tile_MainLight2" | "Tile_SrcLight1" | "Tile_SrcLight2" |
                                "Tile_AnimLoop1" | "Tile_AnimLoop2" | "Tile_AnimLoop3")
                        ).collect::<Vec<_>>();
                        runtime_bytes.len() == 7
                            && runtime_bytes.iter().all(|field| field.value == GffValueV1::Byte(0))
                    }
        ));
        assert!(
            are.root
                .fields
                .iter()
                .any(|field| field.label == "SunAmbientColor"
                    && field.value == GffValueV1::Dword(0x40_40_40))
        );
        assert!(
            are.root
                .fields
                .iter()
                .any(|field| field.label == "SunDiffuseColor"
                    && field.value == GffValueV1::Dword(0xff_ff_ff))
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
                .any(|field| field.label == "XPosition" && field.value == GffValueV1::Float(10.0))
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
