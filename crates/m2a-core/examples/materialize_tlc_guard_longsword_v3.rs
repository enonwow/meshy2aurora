#![recursion_limit = "256"]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::ExitCode,
};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::{GffFileTypeV1, GffLimitsV1, GffValueV1, read_gff_v32},
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    item::{
        ItemAttachmentProfileV1, ItemBaseItemV1, ItemBlueprintV1, ItemColorValuesV1,
        ItemComposerMdlInputV2, ItemEquippedProofIdentityV2, ItemEquippedProofProfileV2,
        ItemFitReportV1, ItemFitReportV2, ItemFitReportV3, ItemFitReportV4, ItemFitSourceV1,
        ItemIconLayerInputV3, ItemIconProjectionBoundsV1, ItemPartBuildOptionsV2,
        ItemPartBuildReportV1, ItemPartTextureEncodingV1, ItemPartValueV1,
        ItemProofModuleIdentityV1, ItemProofPlacementV1, ItemReferenceMdlInputV1,
        build_item_and_equipped_proof_module_v3, build_item_attachment_profile_v1,
        build_item_proof_module_v1, build_meshy_item_part_with_options_v2,
        build_meshy_item_part_with_options_v3, encode_item_weapon_part_appearance_v1,
        extend_item_baseitem_model_range_v1, fit_meshy_item_parts_to_attachment_profile_v1,
        fit_meshy_item_parts_with_target_lengths_aurora_v3,
        fit_meshy_item_parts_with_target_lengths_aurora_v4,
        fit_meshy_item_parts_with_target_lengths_aurora_v5,
        fit_meshy_item_parts_with_target_lengths_v2, item_payload_sha256_v1,
        resolve_item_baseitem_v1, resolve_item_equipped_appearance_v1,
        resolve_item_modeltype2_equipment_slot_v1, resolve_item_part_resource_v1,
        validate_item_fit_report_v1, validate_item_fit_report_v2, validate_item_fit_report_v3,
        validate_item_fit_report_v4, validate_item_fit_report_v4_against_profile_v1,
        validate_item_modeltype2_aurora_append_conformance_v2,
        validate_item_modeltype2_aurora_append_conformance_v3,
        validate_item_modeltype2_aurora_append_conformance_v4,
        validate_item_modeltype2_icon_layers_v3, write_item_uti_v1,
    },
    key_bif::{KeyBifFileInputV1, locate_key_bif_resource_v1, resolve_key_bif_resource_sparse_v1},
};
use serde::Serialize;

const CANONICAL_ROOT: &str = r"C:\Projects\meshy2aurora";
const CANDIDATE_ID: &str = "tlc-guard-longsword-item-model-color-v3-20260803";
const BASEITEMS_PATH: &str = r"C:\Projects\New Folder\item-retail-extract\2da\baseitems.2da";
const BASEITEMS_SHA256: &str = "3fbdcd012b55f0b869c6324cc7d4ece44ed63a78f00e7889e7acefac28c8fdf4";
const APPEARANCE_PATH: &str = r"C:\Projects\meshy2aurora\local-reference-assets\appearance.2da";
const APPEARANCE_SHA256: &str = "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const NWN_USER_ROOT: &str = r"C:\Users\enonw\Documents\Neverwinter Nights";
const NWN_INSTALL_ROOT: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights";

const MODULE_FILE_NAME: &str = "m2atgls3.mod";
const MODULE_RESREF: &str = "m2atgls3";
const MODULE_NAME: &str = "Meshy2Aurora TLC Guard Longsword item v3";
const AREA_RESREF: &str = "m2atgla3";
const AREA_NAME: &str = "TLC Guard Longsword Item Model Color V3";
const HAK_FILE_NAME: &str = "m2atglh3.hak";
const HAK_RESREF: &str = "m2atglh3";
const BLUEPRINT_RESREF: &str = "m2atglu3";
const CREATURE_RESREF: &str = "m2atgln3";
const APPEARANCE_ROW: u16 = 6;
const CREATURE_RACE: u8 = 11;
const CREATURE_GENDER: u8 = 0;
const CREATURE_PHENOTYPE: u8 = 0;
const BASE_ITEM: u32 = 1;
const FIT_TOLERANCE: f32 = 0.01;
const FIT_TARGET_AXIAL_LENGTHS: [f32; 3] = [0.22, 0.08, 0.90];
const FIT_SOLUTION_SHA256: &str =
    "ab335afb5b1b6c73611ffd7243201455e07b29e6fe256d0344bfc9aec829e15a";
const AURORA_FIT_SOLUTION_SHA256: &str =
    "3e5aa566baf9e629632c9fc58ca3af5d11459362a0943bc717ef19bcc0ce2da3";
const COMPOSER_FIT_SOLUTION_SHA256: &str =
    "1bc42564509ddf2e5450b3d3ba9458715a2b632123f85acf0c7aad97fcb1a06f";
const FULL_FRAME_FIT_SOLUTION_SHA256: &str =
    "df615ce933a3639aa59880d8ef208e5c8e08ccef442c3012deb411c5d267e479";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateMode {
    FrozenV3,
    ItemPropertiesV4,
    ItemPropertiesV5,
    ItemPropertiesV6,
    ItemPropertiesV7,
    ItemPropertiesV8,
    ItemPropertiesV9,
}

#[derive(Clone, Copy)]
struct CandidateConfig {
    candidate_id: &'static str,
    module_file_name: &'static str,
    module_resref: &'static str,
    module_name: &'static str,
    area_resref: &'static str,
    area_name: &'static str,
    hak_file_name: &'static str,
    hak_resref: &'static str,
    blueprint_resref: &'static str,
    blueprint_tag: &'static str,
    texture_prefixes: [&'static str; 3],
    expected_fit_sha256: &'static str,
    supersedes_candidate: &'static str,
    admission_evidence: &'static str,
    minimal_delta: &'static str,
}

impl CandidateMode {
    fn config(self) -> CandidateConfig {
        match self {
            Self::FrozenV3 => CandidateConfig {
                candidate_id: CANDIDATE_ID,
                module_file_name: MODULE_FILE_NAME,
                module_resref: MODULE_RESREF,
                module_name: MODULE_NAME,
                area_resref: AREA_RESREF,
                area_name: AREA_NAME,
                hak_file_name: HAK_FILE_NAME,
                hak_resref: HAK_RESREF,
                blueprint_resref: BLUEPRINT_RESREF,
                blueprint_tag: "M2ATGLU3",
                texture_prefixes: ["m2atg3b", "m2atg3m", "m2atg3t"],
                expected_fit_sha256: FIT_SOLUTION_SHA256,
                supersedes_candidate: "tlc-guard-longsword-equipped-v2-20260803",
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v2-owner-result-2026-08-03.json",
                minimal_delta: "replace the three invalid legacy selector variants with the product model/color contract; select 23/63/23, emit the complete 3-part x 4-color MDL/TGA/icon matrix, apply the explicit 0.22/0.08/0.90 proportion contract, and place one real Item plus one separate equipped-render witness bound to the same UTI",
            },
            Self::ItemPropertiesV4 => CandidateConfig {
                candidate_id: "tlc-guard-longsword-item-properties-v4-20260803",
                module_file_name: "m2atgls4.mod",
                module_resref: "m2atgls4",
                module_name: "Meshy2Aurora TLC Guard Longsword item v4",
                area_resref: "m2atgla4",
                area_name: "TLC Guard Longsword Item Properties V4",
                hak_file_name: "m2atglh4.hak",
                hak_resref: "m2atglh4",
                blueprint_resref: "m2atglu4",
                blueprint_tag: "M2ATGLU4",
                texture_prefixes: ["m2atg4b", "m2atg4m", "m2atg4t"],
                expected_fit_sha256: AURORA_FIT_SOLUTION_SHA256,
                supersedes_candidate: CANDIDATE_ID,
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v3-owner-toolset-orientation-result-2026-08-03.json",
                minimal_delta: "rotate every Bottom/Middle/Top source from Meshy +Z into the retail WSwLs +Y axial frame, preserve the exact 0.22/0.08/0.90 proportions and all model/color resources, and remove the unrelated creature so the module contains one real identified Item only",
            },
            Self::ItemPropertiesV5 => CandidateConfig {
                candidate_id: "tlc-guard-longsword-item-properties-v5-20260804",
                module_file_name: "m2atgls5.mod",
                module_resref: "m2atgls5",
                module_name: "Meshy2Aurora TLC Guard Longsword item v5",
                area_resref: "m2atgla5",
                area_name: "TLC Guard Longsword Item Properties V5",
                hak_file_name: "m2atglh5.hak",
                hak_resref: "m2atglh5",
                blueprint_resref: "m2atglu5",
                blueprint_tag: "M2ATGLU5",
                texture_prefixes: ["m2atg5b", "m2atg5m", "m2atg5t"],
                expected_fit_sha256: COMPOSER_FIT_SOLUTION_SHA256,
                supersedes_candidate: "tlc-guard-longsword-item-properties-v4-20260803",
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v4-owner-item-properties-result-2026-08-04.json",
                minimal_delta: "preserve the exact Meshy sources, BaseItem 1, model/color selections 23/63/23 and 0.22/0.08/0.90 proportions; replace V4 root-owned transforms with baked axial geometry plus Trimesh-child translation controllers, emit globally unique resref-derived child names, require explicit adjacent connector overlap, and pass node-aware Aurora append conformance for every colorway",
            },
            Self::ItemPropertiesV6 => CandidateConfig {
                candidate_id: "tlc-guard-longsword-item-properties-v6-20260804",
                module_file_name: "m2atgls6.mod",
                module_resref: "m2atgls6",
                module_name: "Meshy2Aurora TLC Guard Longsword item v6",
                area_resref: "m2atgla6",
                area_name: "TLC Guard Longsword Item Properties V6",
                hak_file_name: "m2atglh6.hak",
                hak_resref: "m2atglh6",
                blueprint_resref: "m2atglu6",
                blueprint_tag: "M2ATGLU6",
                texture_prefixes: ["m2atg6b", "m2atg6m", "m2atg6t"],
                expected_fit_sha256: FULL_FRAME_FIT_SOLUTION_SHA256,
                supersedes_candidate: "tlc-guard-longsword-item-properties-v5-20260804",
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v5-owner-item-properties-result-2026-08-04.json",
                minimal_delta: "preserve the exact V5 Meshy sources, BaseItem 1, model/color selections 23/63/23, target lengths and connector-overlap writer; replace axial-only orientation with a complete proper frame mapping axial to +Y, broad width to +Z and depth/front to +X, and require one native three-layer full-canvas vertical icon composite for every colorway",
            },
            Self::ItemPropertiesV7 => CandidateConfig {
                candidate_id: "tlc-guard-longsword-item-properties-v7-20260804",
                module_file_name: "m2atgls7.mod",
                module_resref: "m2atgls7",
                module_name: "Meshy2Aurora TLC Guard Longsword item v7",
                area_resref: "m2atgla7",
                area_name: "TLC Guard Longsword Item Properties V7",
                hak_file_name: "m2atglh7.hak",
                hak_resref: "m2atglh7",
                blueprint_resref: "m2atglu7",
                blueprint_tag: "M2ATGLU7",
                texture_prefixes: ["m2atg6b", "m2atg6m", "m2atg6t"],
                expected_fit_sha256: FULL_FRAME_FIT_SOLUTION_SHA256,
                supersedes_candidate: "tlc-guard-longsword-item-properties-v6-20260804",
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v6-owner-item-properties-result-2026-08-04.json",
                minimal_delta: "preserve every V6 GLB, fit transform, connector, MDL and texture byte-for-byte; rotate the one shared two-dimensional icon frame by 180 degrees for all Bottom/Middle/Top layers together and require decoded TGA part order Top above Middle above Bottom",
            },
            Self::ItemPropertiesV8 => CandidateConfig {
                candidate_id: "tlc-guard-longsword-item-reference-profile-v8-20260804",
                module_file_name: "m2atgls8.mod",
                module_resref: "m2atgls8",
                module_name: "Meshy2Aurora TLC Guard Longsword reference profile v8",
                area_resref: "m2atgla8",
                area_name: "TLC Guard Longsword Reference Profile V8",
                hak_file_name: "m2atglh8.hak",
                hak_resref: "m2atglh8",
                blueprint_resref: "m2atglu8",
                blueprint_tag: "M2ATGLU8",
                texture_prefixes: ["m2atg8b", "m2atg8m", "m2atg8t"],
                expected_fit_sha256: "ac087a33ebc4949e1c19f7db20eb801f72ee5edc85d0efcd58685038fe23d3f4",
                supersedes_candidate: "tlc-guard-longsword-item-properties-v7-20260804",
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v7-owner-slot-frame-result-2026-08-04.json",
                minimal_delta: "replace V7 cursor/target-length placement with an exact retail nwn_base.key/models_02.bif attachment profile; keep the three Meshy GLBs, colors and full-frame orientation, allocate free model 25 selectors instead of shadowing reference 23/63/23, and require real triangle-surface slot bounds with native connector overlap and common hand origin",
            },
            Self::ItemPropertiesV9 => CandidateConfig {
                candidate_id: "tlc-guard-longsword-item-range-envelope-v9-20260804",
                module_file_name: "m2atgls9.mod",
                module_resref: "m2atgls9",
                module_name: "Meshy2Aurora TLC Guard Longsword range envelope v9",
                area_resref: "m2atgla9",
                area_name: "TLC Guard Longsword Range Envelope V9",
                hak_file_name: "m2atglh9.hak",
                hak_resref: "m2atglh9",
                blueprint_resref: "m2atglu9",
                blueprint_tag: "M2ATGLU9",
                texture_prefixes: ["m2atg9b", "m2atg9m", "m2atg9t"],
                expected_fit_sha256: "f213e326161ba57d5c09728e1a44c63e498826d17d74072bcbdd6f748c778907",
                supersedes_candidate: "tlc-guard-longsword-item-reference-profile-v8-20260804",
                admission_evidence: "documentation/evidence/tlc-guard-longsword-v8-owner-reference-profile-result-2026-08-04.json",
                minimal_delta: "preserve BaseItem 1, model 25, the exact V8 Meshy sources, colors, reference orientation and axial slot ranges; add the exact baseitems.2da MaxRange 100->250 override, fit every part inside the native reference X/Z envelope with one aspect-preserving transverse scale, and remove the broken equipped creature witness so the module contains only the identified Item",
            },
        }
    }

    fn is_item_only(self) -> bool {
        self != Self::FrozenV3
    }

    fn uses_aurora_composer_v2(self) -> bool {
        matches!(
            self,
            Self::ItemPropertiesV5
                | Self::ItemPropertiesV6
                | Self::ItemPropertiesV7
                | Self::ItemPropertiesV8
                | Self::ItemPropertiesV9
        )
    }

    fn uses_full_frame_v3(self) -> bool {
        matches!(
            self,
            Self::ItemPropertiesV6
                | Self::ItemPropertiesV7
                | Self::ItemPropertiesV8
                | Self::ItemPropertiesV9
        )
    }

    fn uses_icon_part_order_v2(self) -> bool {
        matches!(
            self,
            Self::ItemPropertiesV7 | Self::ItemPropertiesV8 | Self::ItemPropertiesV9
        )
    }

    fn requires_frozen_v6_delta(self) -> bool {
        self == Self::ItemPropertiesV7
    }

    fn uses_reference_profile_v4(self) -> bool {
        matches!(self, Self::ItemPropertiesV8 | Self::ItemPropertiesV9)
    }

    fn has_equipped_witness(self) -> bool {
        matches!(self, Self::FrozenV3 | Self::ItemPropertiesV8)
    }
}

struct PartSpec {
    field: &'static str,
    role: &'static str,
    token: &'static str,
    model: u8,
    selected_color: u8,
    variant: u8,
    model_resref: &'static str,
    icon_resref: &'static str,
    texture_resref: &'static str,
    texture_prefix: &'static str,
    source_file_name: &'static str,
    source_sha256: &'static str,
    concept_path: &'static str,
    concept_sha256: &'static str,
    task_id: &'static str,
    target_polycount: u32,
    consumed_credits: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceReport {
    field: String,
    role: String,
    path: String,
    byte_length: usize,
    sha256: String,
    concept_path: String,
    concept_sha256: String,
    task_id: String,
    target_polycount: u32,
    consumed_credits: u32,
}

struct BuiltPart {
    field: &'static str,
    role: &'static str,
    color: u8,
    variant: u8,
    model_resref: String,
    texture_resref: String,
    icon_resref: String,
    mdl: Vec<u8>,
    texture: Vec<u8>,
    icon: Vec<u8>,
    report: ItemPartBuildReportV1,
}

struct CandidateFitPart {
    field: String,
    source_sha256: String,
    triangle_count: usize,
    transform: m2a_core::item::ItemPartTransformV1,
    target_space_scale_xyz: [f32; 3],
}

struct CandidateFit {
    algorithm: String,
    status: String,
    solution_sha256: String,
    parts: Vec<CandidateFitPart>,
    report: serde_json::Value,
    composer_report_v2: Option<ItemFitReportV2>,
    composer_report_v3: Option<ItemFitReportV3>,
    composer_report_v4: Option<ItemFitReportV4>,
}

struct RetailReferenceProfileV1 {
    baseitems: Vec<u8>,
    base_item: ItemBaseItemV1,
    profile: ItemAttachmentProfileV1,
    lineage: serde_json::Value,
}

fn main() -> ExitCode {
    let mode = match std::env::args().nth(1).as_deref() {
        None => CandidateMode::FrozenV3,
        Some("--item-properties-v4") => CandidateMode::ItemPropertiesV4,
        Some("--item-properties-v5") => CandidateMode::ItemPropertiesV5,
        Some("--item-properties-v6") => CandidateMode::ItemPropertiesV6,
        Some("--item-properties-v7") => CandidateMode::ItemPropertiesV7,
        Some("--item-properties-v8") => CandidateMode::ItemPropertiesV8,
        Some("--item-properties-v9") => CandidateMode::ItemPropertiesV9,
        Some(argument) => {
            eprintln!("TLC-GUARD-LS-ARGUMENT-INVALID: unsupported argument {argument}");
            return ExitCode::FAILURE;
        }
    };
    match run(mode) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(mode: CandidateMode) -> Result<String, String> {
    let config = mode.config();
    let canonical_root = Path::new(CANONICAL_ROOT)
        .canonicalize()
        .map_err(|error| format!("TLC-GUARD-LS-CANONICAL-ROOT-RESOLVE: {error}"))?;
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("TLC-GUARD-LS-WORKSPACE-RESOLVE: {error}"))?;
    if !workspace_root.starts_with(&canonical_root) {
        return Err(format!(
            "TLC-GUARD-LS-WORKSPACE-INVALID: active workspace {} is outside {}",
            workspace_root.display(),
            canonical_root.display()
        ));
    }
    let output_root = canonical_root
        .join("proof-output")
        .join(config.candidate_id);
    if output_root.exists() {
        return Err(format!(
            "TLC-GUARD-LS-OUTPUT-EXISTS: immutable candidate root already exists: {}",
            output_root.display()
        ));
    }

    let retail_reference = if mode.uses_reference_profile_v4() {
        Some(load_retail_wswls_reference_profile_v1()?)
    } else {
        None
    };
    let (baseitems, base_item) = if let Some(reference) = &retail_reference {
        (reference.baseitems.clone(), reference.base_item.clone())
    } else {
        let baseitems = read_exact(
            Path::new(BASEITEMS_PATH),
            BASEITEMS_SHA256,
            "TLC-GUARD-LS-BASEITEMS",
        )?;
        let base_item = resolve_item_baseitem_v1(&baseitems, BASE_ITEM)
            .map_err(|error| json_error("TLC-GUARD-LS-BASEITEM-RESOLVE", &error))?;
        (baseitems, base_item)
    };
    if base_item.model_type != 2
        || base_item.part_slots.len() != 3
        || base_item.item_class.to_ascii_lowercase() != "wswls"
    {
        return Err(
            "TLC-GUARD-LS-BASEITEM-CONTRACT: BaseItem 1 must resolve to WSwLs ModelType 2 with Bottom/Middle/Top"
                .to_owned(),
        );
    }
    let equipment_slot = resolve_item_modeltype2_equipment_slot_v1(base_item.equipable_slots)
        .map_err(|error| json_error("TLC-GUARD-LS-EQUIP-SLOT", &error))?;
    if equipment_slot != 16 {
        return Err(format!(
            "TLC-GUARD-LS-EQUIP-SLOT: BaseItem 1 must select the exact native Equip_ItemList right-hand bit 16 from mask {}, got {equipment_slot}",
            base_item.equipable_slots,
        ));
    }
    let appearance = read_exact(
        Path::new(APPEARANCE_PATH),
        APPEARANCE_SHA256,
        "TLC-GUARD-LS-APPEARANCE",
    )?;
    let appearance_binding = resolve_item_equipped_appearance_v1(
        &appearance,
        APPEARANCE_ROW,
        CREATURE_RACE,
        CREATURE_GENDER,
        CREATURE_PHENOTYPE,
    )
    .map_err(|error| json_error("TLC-GUARD-LS-APPEARANCE-BINDING", &error))?;

    let specs = [
        PartSpec {
            field: "ModelPart1",
            role: "BOTTOM",
            token: "b",
            model: if mode.uses_reference_profile_v4() {
                25
            } else {
                2
            },
            selected_color: 3,
            variant: if mode.uses_reference_profile_v4() {
                253
            } else {
                23
            },
            model_resref: if mode.uses_reference_profile_v4() {
                "wswls_b_253"
            } else {
                "wswls_b_023"
            },
            icon_resref: if mode.uses_reference_profile_v4() {
                "iwswls_b_253"
            } else {
                "iwswls_b_023"
            },
            texture_resref: match mode {
                CandidateMode::FrozenV3 => "m2atg3b3",
                CandidateMode::ItemPropertiesV4 => "m2atg4b3",
                CandidateMode::ItemPropertiesV5 => "m2atg5b3",
                CandidateMode::ItemPropertiesV6 => "m2atg6b3",
                CandidateMode::ItemPropertiesV7 => "m2atg6b3",
                CandidateMode::ItemPropertiesV8 => "m2atg8b3",
                CandidateMode::ItemPropertiesV9 => "m2atg9b3",
            },
            texture_prefix: config.texture_prefixes[0],
            source_file_name: "bottom.glb",
            source_sha256: "01e554b8a59e934597540f9b1e4a697a2536356c0c60ebb0e5f8ad2067b79766",
            concept_path: "documentation/concepts/item-longsword-demo-v11-last-city-guard-parts/bottom-concept.png",
            concept_sha256: "1ae06ad2819d118f66973d6825973e956126544c73a42258910883cb4e1569b8",
            task_id: "019fc24e-2b01-78b6-8edc-5b3c63758adf",
            target_polycount: 5_000,
            consumed_credits: 30,
        },
        PartSpec {
            field: "ModelPart2",
            role: "MIDDLE",
            token: "m",
            model: if mode.uses_reference_profile_v4() {
                25
            } else {
                6
            },
            selected_color: 3,
            variant: if mode.uses_reference_profile_v4() {
                253
            } else {
                63
            },
            model_resref: if mode.uses_reference_profile_v4() {
                "wswls_m_253"
            } else {
                "wswls_m_063"
            },
            icon_resref: if mode.uses_reference_profile_v4() {
                "iwswls_m_253"
            } else {
                "iwswls_m_063"
            },
            texture_resref: match mode {
                CandidateMode::FrozenV3 => "m2atg3m3",
                CandidateMode::ItemPropertiesV4 => "m2atg4m3",
                CandidateMode::ItemPropertiesV5 => "m2atg5m3",
                CandidateMode::ItemPropertiesV6 => "m2atg6m3",
                CandidateMode::ItemPropertiesV7 => "m2atg6m3",
                CandidateMode::ItemPropertiesV8 => "m2atg8m3",
                CandidateMode::ItemPropertiesV9 => "m2atg9m3",
            },
            texture_prefix: config.texture_prefixes[1],
            source_file_name: "middle.glb",
            source_sha256: "6dae50bf1844d549c3e46c8a570258100ec14d04420bd333736854913af2b071",
            concept_path: "documentation/concepts/item-longsword-demo-v11-last-city-guard-parts/middle-concept.png",
            concept_sha256: "85cc222af76b83a2d7e3716406c8403a893fe7ec650a7a84fa77852d744d679f",
            task_id: "019fc250-9a9f-77f7-bd8d-db021fcc4877",
            target_polycount: 8_000,
            consumed_credits: 30,
        },
        PartSpec {
            field: "ModelPart3",
            role: "TOP",
            token: "t",
            model: if mode.uses_reference_profile_v4() {
                25
            } else {
                2
            },
            selected_color: 3,
            variant: if mode.uses_reference_profile_v4() {
                253
            } else {
                23
            },
            model_resref: if mode.uses_reference_profile_v4() {
                "wswls_t_253"
            } else {
                "wswls_t_023"
            },
            icon_resref: if mode.uses_reference_profile_v4() {
                "iwswls_t_253"
            } else {
                "iwswls_t_023"
            },
            texture_resref: match mode {
                CandidateMode::FrozenV3 => "m2atg3t3",
                CandidateMode::ItemPropertiesV4 => "m2atg4t3",
                CandidateMode::ItemPropertiesV5 => "m2atg5t3",
                CandidateMode::ItemPropertiesV6 => "m2atg6t3",
                CandidateMode::ItemPropertiesV7 => "m2atg6t3",
                CandidateMode::ItemPropertiesV8 => "m2atg8t3",
                CandidateMode::ItemPropertiesV9 => "m2atg9t3",
            },
            texture_prefix: config.texture_prefixes[2],
            source_file_name: "top.glb",
            source_sha256: "be226f1f891dfee517c221121289689cb21fed99bc41ec3b6e09e7611df15143",
            concept_path: "documentation/concepts/item-longsword-demo-v11-last-city-guard-parts/top-concept.png",
            concept_sha256: "3af210423d1b519bb6fd721cd7c4154794152706a183ecd8debaed7113d44f8b",
            task_id: "019fc254-ed2e-79da-b404-ac85c9103a74",
            target_polycount: 3_000,
            consumed_credits: 30,
        },
    ];

    let source_root = canonical_root
        .join("sample-3d")
        .join("tlc-guard-longsword-parts-v1");
    let mut source_payloads = Vec::with_capacity(specs.len());
    let mut source_reports = Vec::with_capacity(specs.len());
    for spec in &specs {
        let resolved =
            resolve_item_part_resource_v1(&base_item, spec.field, spec.variant, None, None)
                .map_err(|error| json_error("TLC-GUARD-LS-RESOURCE-RESOLVE", &error))?;
        if resolved.model_resref != spec.model_resref || resolved.icon_resref != spec.icon_resref {
            return Err(format!(
                "TLC-GUARD-LS-RESOURCE-IDENTITY: {} resolved to {}/{}, expected {}/{}",
                spec.field,
                resolved.model_resref,
                resolved.icon_resref,
                spec.model_resref,
                spec.icon_resref
            ));
        }
        let source_path = source_root.join(spec.source_file_name);
        let source = read_exact(&source_path, spec.source_sha256, "TLC-GUARD-LS-SOURCE")?;
        let concept_path = workspace_root.join(spec.concept_path);
        read_exact(&concept_path, spec.concept_sha256, "TLC-GUARD-LS-CONCEPT")?;
        source_reports.push(SourceReport {
            field: spec.field.to_owned(),
            role: spec.role.to_owned(),
            path: source_path.display().to_string(),
            byte_length: source.len(),
            sha256: item_payload_sha256_v1(&source),
            concept_path: concept_path.display().to_string(),
            concept_sha256: spec.concept_sha256.to_owned(),
            task_id: spec.task_id.to_owned(),
            target_polycount: spec.target_polycount,
            consumed_credits: spec.consumed_credits,
        });
        source_payloads.push(source);
    }

    let fit_sources = specs
        .iter()
        .zip(&source_payloads)
        .map(|(spec, source)| ItemFitSourceV1 {
            field: spec.field,
            model_resref: spec.model_resref,
            source_glb: source,
            source_node: None,
        })
        .collect::<Vec<_>>();
    let fit = match mode {
        CandidateMode::FrozenV3 => {
            let report = fit_meshy_item_parts_with_target_lengths_v2(
                &fit_sources,
                FIT_TOLERANCE,
                &FIT_TARGET_AXIAL_LENGTHS,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-FIT", &error))?;
            validate_item_fit_report_v1(&report)
                .map_err(|error| json_error("TLC-GUARD-LS-FIT-READBACK", &error))?;
            candidate_fit_from_v1(report)?
        }
        CandidateMode::ItemPropertiesV4 => {
            let report = fit_meshy_item_parts_with_target_lengths_aurora_v3(
                &fit_sources,
                FIT_TOLERANCE,
                &FIT_TARGET_AXIAL_LENGTHS,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-FIT", &error))?;
            validate_item_fit_report_v1(&report)
                .map_err(|error| json_error("TLC-GUARD-LS-FIT-READBACK", &error))?;
            candidate_fit_from_v1(report)?
        }
        CandidateMode::ItemPropertiesV5 => {
            let report = fit_meshy_item_parts_with_target_lengths_aurora_v4(
                &fit_sources,
                FIT_TOLERANCE,
                &FIT_TARGET_AXIAL_LENGTHS,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-FIT", &error))?;
            validate_item_fit_report_v2(&report)
                .map_err(|error| json_error("TLC-GUARD-LS-FIT-READBACK", &error))?;
            candidate_fit_from_v2(report)?
        }
        CandidateMode::ItemPropertiesV6 | CandidateMode::ItemPropertiesV7 => {
            let report = fit_meshy_item_parts_with_target_lengths_aurora_v5(
                &fit_sources,
                FIT_TOLERANCE,
                &FIT_TARGET_AXIAL_LENGTHS,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-FIT", &error))?;
            validate_item_fit_report_v3(&report)
                .map_err(|error| json_error("TLC-GUARD-LS-FIT-READBACK", &error))?;
            candidate_fit_from_v3(report)?
        }
        CandidateMode::ItemPropertiesV8 | CandidateMode::ItemPropertiesV9 => {
            let profile = &retail_reference
                .as_ref()
                .ok_or_else(|| {
                    "TLC-GUARD-LS-REFERENCE-PROFILE-MISSING: V8/V9 requires retail KEY/BIF profile"
                        .to_owned()
                })?
                .profile;
            let report =
                fit_meshy_item_parts_to_attachment_profile_v1(&fit_sources, FIT_TOLERANCE, profile)
                    .map_err(|error| json_error("TLC-GUARD-LS-REFERENCE-FIT", &error))?;
            validate_item_fit_report_v4(&report)
                .map_err(|error| json_error("TLC-GUARD-LS-FIT-READBACK", &error))?;
            validate_item_fit_report_v4_against_profile_v1(&report, profile)
                .map_err(|error| json_error("TLC-GUARD-LS-PROFILE-FIT-READBACK", &error))?;
            candidate_fit_from_v4(report)?
        }
    };
    if fit.status != "PASSED" || fit.solution_sha256 != config.expected_fit_sha256 {
        return Err(format!(
            "TLC-GUARD-LS-FIT-IDENTITY: expected PASSED/{}, got {}/{}",
            config.expected_fit_sha256, fit.status, fit.solution_sha256
        ));
    }

    let icon_fit = if mode.uses_reference_profile_v4() {
        let report = fit_meshy_item_parts_with_target_lengths_aurora_v5(
            &fit_sources,
            FIT_TOLERANCE,
            &FIT_TARGET_AXIAL_LENGTHS,
        )
        .map_err(|error| json_error("TLC-GUARD-LS-ICON-FIT", &error))?;
        validate_item_fit_report_v3(&report)
            .map_err(|error| json_error("TLC-GUARD-LS-ICON-FIT-READBACK", &error))?;
        if report.solution_sha256 != FULL_FRAME_FIT_SOLUTION_SHA256 {
            return Err(format!(
                "TLC-GUARD-LS-ICON-FIT-IDENTITY: expected {FULL_FRAME_FIT_SOLUTION_SHA256}, got {}",
                report.solution_sha256
            ));
        }
        Some(candidate_fit_from_v3(report)?)
    } else {
        None
    };

    let mut projection_bounds: Option<ItemIconProjectionBoundsV1> = None;
    let mut preflight_reports = Vec::with_capacity(specs.len());
    for (index, ((spec, source), fit_part)) in specs
        .iter()
        .zip(&source_payloads)
        .zip(&fit.parts)
        .enumerate()
    {
        if fit_part.field != spec.field || fit_part.source_sha256 != spec.source_sha256 {
            return Err(format!(
                "TLC-GUARD-LS-FIT-BINDING: {} is not bound to its exact source",
                spec.field
            ));
        }
        let options = part_options(
            fit_part.transform,
            fit_part.target_space_scale_xyz,
            None,
            None,
            Some(spec.selected_color),
        );
        let artifact = if mode.uses_aurora_composer_v2() {
            build_meshy_item_part_with_options_v3(
                source,
                spec.model_resref,
                spec.texture_resref,
                &options,
            )
        } else {
            build_meshy_item_part_with_options_v2(
                source,
                spec.model_resref,
                spec.texture_resref,
                &options,
            )
        }
        .map_err(|error| json_error("TLC-GUARD-LS-PART-PREFLIGHT", &error))?;
        if artifact.icon_payload.is_some() {
            return Err(format!(
                "TLC-GUARD-LS-ICON-PREFLIGHT: {} emitted an icon without icon_size",
                spec.field
            ));
        }
        let icon_bounds_report = if let Some(icon_fit) = &icon_fit {
            let icon_options = part_options(
                icon_fit.parts[index].transform,
                icon_fit.parts[index].target_space_scale_xyz,
                None,
                None,
                Some(spec.selected_color),
            );
            build_meshy_item_part_with_options_v3(
                source,
                spec.model_resref,
                spec.texture_resref,
                &icon_options,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-ICON-PREFLIGHT", &error))?
            .report
        } else {
            artifact.report.clone()
        };
        let own_bounds = icon_bounds_report.icon_projection_bounds.ok_or_else(|| {
            format!(
                "TLC-GUARD-LS-ICON-BOUNDS: {} emitted no geometry projection bounds",
                spec.field
            )
        })?;
        projection_bounds = Some(union_bounds(projection_bounds, own_bounds));
        preflight_reports.push(artifact.report);
    }
    let projection_bounds = projection_bounds
        .ok_or_else(|| "TLC-GUARD-LS-ICON-BOUNDS: no part bounds were emitted".to_owned())?;
    let icon_size = [
        base_item.inv_slot_width.max(1) * 32,
        base_item.inv_slot_height.max(1) * 32,
    ];

    let mut built_parts = Vec::with_capacity(specs.len() * 4);
    for (index, (((spec, source), fit_part), preflight)) in specs
        .iter()
        .zip(&source_payloads)
        .zip(&fit.parts)
        .zip(&preflight_reports)
        .enumerate()
    {
        for color in 1..=4 {
            let variant = encode_item_weapon_part_appearance_v1(spec.model, color)
                .map_err(|error| json_error("TLC-GUARD-LS-COLORWAY-ENCODE", &error))?;
            let model_resref = format!("wswls_{}_{variant:03}", spec.token);
            let icon_resref = format!("iwswls_{}_{variant:03}", spec.token);
            let texture_resref = format!("{}{color}", spec.texture_prefix);
            let resolved =
                resolve_item_part_resource_v1(&base_item, spec.field, variant, None, None)
                    .map_err(|error| json_error("TLC-GUARD-LS-COLORWAY-RESOLVE", &error))?;
            if resolved.model_resref != model_resref || resolved.icon_resref != icon_resref {
                return Err(format!(
                    "TLC-GUARD-LS-COLORWAY-IDENTITY: {} color {color} resolved to {}/{}, expected {}/{}",
                    spec.field,
                    resolved.model_resref,
                    resolved.icon_resref,
                    model_resref,
                    icon_resref
                ));
            }
            let options = part_options(
                fit_part.transform,
                fit_part.target_space_scale_xyz,
                if icon_fit.is_some() {
                    None
                } else {
                    Some(icon_size)
                },
                if icon_fit.is_some() {
                    None
                } else {
                    Some(projection_bounds)
                },
                Some(color),
            );
            let mut artifact = if mode.uses_aurora_composer_v2() {
                build_meshy_item_part_with_options_v3(
                    source,
                    &model_resref,
                    &texture_resref,
                    &options,
                )
            } else {
                build_meshy_item_part_with_options_v2(
                    source,
                    &model_resref,
                    &texture_resref,
                    &options,
                )
            }
            .map_err(|error| json_error("TLC-GUARD-LS-PART-BUILD", &error))?;
            let icon = if let Some(icon_fit) = &icon_fit {
                let icon_options = part_options(
                    icon_fit.parts[index].transform,
                    icon_fit.parts[index].target_space_scale_xyz,
                    Some(icon_size),
                    Some(projection_bounds),
                    Some(color),
                );
                let icon_artifact = build_meshy_item_part_with_options_v3(
                    source,
                    &model_resref,
                    &texture_resref,
                    &icon_options,
                )
                .map_err(|error| json_error("TLC-GUARD-LS-ICON-BUILD", &error))?;
                let icon = icon_artifact.icon_payload.ok_or_else(|| {
                    format!(
                        "TLC-GUARD-LS-ICON-MISSING: {} color {color} emitted no presentation-fit icon layer",
                        spec.field
                    )
                })?;
                artifact.report.icon_sha256 = icon_artifact.report.icon_sha256;
                artifact.report.icon_projection_bounds =
                    icon_artifact.report.icon_projection_bounds;
                artifact.report.icon_opaque_pixel_count =
                    icon_artifact.report.icon_opaque_pixel_count;
                icon
            } else {
                artifact.icon_payload.take().ok_or_else(|| {
                format!(
                    "TLC-GUARD-LS-ICON-MISSING: {} color {color} emitted no shared-frame icon layer",
                    spec.field
                )
                })?
            };
            let selected_hash_mismatch = color == spec.selected_color
                && (artifact.report.mdl_sha256 != preflight.mdl_sha256
                    || artifact.report.texture_sha256 != preflight.texture_sha256);
            if selected_hash_mismatch
                || artifact.report.degenerate_triangle_count_removed
                    != preflight.degenerate_triangle_count_removed
                || artifact.report.icon_opaque_pixel_count.unwrap_or(0) == 0
                || artifact.report.semantic_readback_status != "PASS"
                || artifact.report.weapon_color != Some(color)
            {
                return Err(format!(
                    "TLC-GUARD-LS-PART-READBACK: {} color {color} failed deterministic build/readback",
                    spec.field
                ));
            }
            built_parts.push(BuiltPart {
                field: spec.field,
                role: spec.role,
                color,
                variant,
                model_resref,
                texture_resref,
                icon_resref,
                mdl: artifact.mdl_payload,
                texture: artifact.texture_payload,
                icon,
                report: artifact.report,
            });
        }
        let texture_hashes = built_parts
            .iter()
            .filter(|part| part.field == spec.field)
            .map(|part| part.report.texture_sha256.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if texture_hashes.len() != 4 {
            return Err(format!(
                "TLC-GUARD-LS-COLORWAY-TEXTURE-DIFF: {} did not emit four distinct concrete textures",
                spec.field
            ));
        }
    }

    let composer_conformance = if mode.uses_aurora_composer_v2() {
        let mut colorways = Vec::with_capacity(4);
        for color in 1..=4 {
            let color_parts = specs
                .iter()
                .map(|spec| {
                    built_parts
                        .iter()
                        .find(|part| part.field == spec.field && part.color == color)
                        .ok_or_else(|| {
                            format!(
                                "TLC-GUARD-LS-COMPOSER-COLORWAY: color {color} has no {}",
                                spec.field
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let inputs = color_parts
                .iter()
                .map(|part| ItemComposerMdlInputV2 {
                    field: part.field,
                    model_resref: &part.model_resref,
                    mdl_payload: &part.mdl,
                })
                .collect::<Vec<_>>();
            let conformance = if mode.uses_reference_profile_v4() {
                let fit_report = fit.composer_report_v4.as_ref().ok_or_else(|| {
                    "TLC-GUARD-LS-COMPOSER-FIT: V8 is missing its typed ItemFitReportV4".to_owned()
                })?;
                validate_item_modeltype2_aurora_append_conformance_v4(&inputs, fit_report)
            } else if mode.uses_full_frame_v3() {
                let fit_report = fit.composer_report_v3.as_ref().ok_or_else(|| {
                    "TLC-GUARD-LS-COMPOSER-FIT: V6 is missing its typed ItemFitReportV3".to_owned()
                })?;
                validate_item_modeltype2_aurora_append_conformance_v3(&inputs, fit_report)
            } else {
                let fit_report = fit.composer_report_v2.as_ref().ok_or_else(|| {
                    "TLC-GUARD-LS-COMPOSER-FIT: V5 is missing its typed ItemFitReportV2".to_owned()
                })?;
                validate_item_modeltype2_aurora_append_conformance_v2(&inputs, fit_report)
            }
            .map_err(|error| json_error("TLC-GUARD-LS-COMPOSER-READBACK", &error))?;
            if conformance.total_triangle_count != 16_356 {
                return Err(format!(
                    "TLC-GUARD-LS-COMPOSER-TRIANGLES: color {color} emitted {}, expected 16356",
                    conformance.total_triangle_count
                ));
            }
            colorways.push(serde_json::json!({
                "color": color,
                "report": conformance,
            }));
        }
        Some(colorways)
    } else {
        None
    };

    let icon_conformance = if mode.uses_full_frame_v3() {
        let layout_profile = if mode.uses_icon_part_order_v2() {
            "LONG_VERTICAL_PART_ORDER_V2"
        } else {
            "LONG_VERTICAL_V1"
        };
        let mut colorways = Vec::with_capacity(4);
        for color in 1..=4 {
            let color_parts = specs
                .iter()
                .map(|spec| {
                    built_parts
                        .iter()
                        .find(|part| part.field == spec.field && part.color == color)
                        .ok_or_else(|| {
                            format!(
                                "TLC-GUARD-LS-ICON-COLORWAY: color {color} has no {}",
                                spec.field
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let inputs = color_parts
                .iter()
                .map(|part| ItemIconLayerInputV3 {
                    field: part.field,
                    icon_resref: &part.icon_resref,
                    payload: &part.icon,
                })
                .collect::<Vec<_>>();
            let conformance = validate_item_modeltype2_icon_layers_v3(
                &inputs,
                icon_size[0],
                icon_size[1],
                layout_profile,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-ICON-COMPOSITE-READBACK", &error))?;
            if conformance.status != "PASSED" || conformance.layer_count != 3 {
                return Err(format!(
                    "TLC-GUARD-LS-ICON-COMPOSITE: color {color} failed native three-layer composition"
                ));
            }
            colorways.push(serde_json::json!({
                "color": color,
                "report": conformance,
            }));
        }
        Some(colorways)
    } else {
        None
    };

    let frozen_v6_delta = if mode.requires_frozen_v6_delta() {
        let v6_root = canonical_root
            .join("proof-output")
            .join("tlc-guard-longsword-item-properties-v6-20260804");
        let mut reports = Vec::with_capacity(built_parts.len());
        for part in &built_parts {
            let v6_mdl = fs::read(v6_root.join(format!("{}.mdl", part.model_resref)))
                .map_err(|error| format!("TLC-GUARD-LS-V7-V6-MDL-READ: {error}"))?;
            let v6_texture = fs::read(v6_root.join(format!("{}.tga", part.texture_resref)))
                .map_err(|error| format!("TLC-GUARD-LS-V7-V6-TEXTURE-READ: {error}"))?;
            let v6_icon = fs::read(v6_root.join(format!("{}.tga", part.icon_resref)))
                .map_err(|error| format!("TLC-GUARD-LS-V7-V6-ICON-READ: {error}"))?;
            if part.mdl != v6_mdl || part.texture != v6_texture {
                return Err(format!(
                    "TLC-GUARD-LS-V7-MODEL-DELTA: {} changed MDL or texture bytes even though V6 model presentation was accepted",
                    part.field
                ));
            }
            if part.icon == v6_icon {
                return Err(format!(
                    "TLC-GUARD-LS-V7-ICON-UNCHANGED: {} color {} did not rotate the frozen V6 icon layer",
                    part.field, part.color
                ));
            }
            reports.push(serde_json::json!({
                "field": part.field,
                "color": part.color,
                "modelByteIdenticalToV6": true,
                "textureByteIdenticalToV6": true,
                "v6IconSha256": item_payload_sha256_v1(&v6_icon),
                "v7IconSha256": item_payload_sha256_v1(&part.icon),
                "iconChanged": true,
            }));
        }
        Some(reports)
    } else {
        None
    };

    let source_triangle_count = fit
        .parts
        .iter()
        .map(|part| part.triangle_count)
        .sum::<usize>();
    let triangle_count = built_parts
        .iter()
        .filter(|part| {
            specs
                .iter()
                .find(|spec| spec.field == part.field)
                .is_some_and(|spec| spec.selected_color == part.color)
        })
        .map(|part| part.report.triangle_count)
        .sum::<usize>();
    let degenerate_triangle_count_removed = built_parts
        .iter()
        .filter(|part| {
            specs
                .iter()
                .find(|spec| spec.field == part.field)
                .is_some_and(|spec| spec.selected_color == part.color)
        })
        .map(|part| part.report.degenerate_triangle_count_removed)
        .sum::<usize>();
    if source_triangle_count != 16_356
        || triangle_count + degenerate_triangle_count_removed != source_triangle_count
        || triangle_count > 300_000
    {
        return Err(format!(
            "TLC-GUARD-LS-TRIANGLE-BUDGET: source={source_triangle_count}, emitted={triangle_count}, removed={degenerate_triangle_count_removed}, budget=300000"
        ));
    }

    let uti = write_item_uti_v1(
        &base_item,
        &ItemBlueprintV1 {
            schema_version: 1,
            template_resref: config.blueprint_resref.to_owned(),
            tag: config.blueprint_tag.to_owned(),
            localized_name: "Last City Bronze Guard Longsword".to_owned(),
            description: "A real three-part longsword Item assembled by the Meshy2Aurora model/color pipeline.".to_owned(),
            identified_description: if mode.uses_reference_profile_v4() {
                "An identified Last City guard longsword using collision-free model 25, concrete bronze color 3 and the exact retail WSwLs attachment profile."
            } else {
                "An identified Last City guard longsword using model selections 2/6/2 and concrete bronze color 3 on all three independent parts."
            }
            .to_owned(),
            comment: match mode {
                CandidateMode::FrozenV3 => "V3 uses the complete ModelType 2 model/color resource matrix; final visual proof is owner-owned.",
                CandidateMode::ItemPropertiesV4 => "V4 uses the Aurora retail WSwLs +Y part frame and contains one real Item only; final visual proof is owner-owned.",
                CandidateMode::ItemPropertiesV5 => "V5 uses controllerless roots, resref-derived Trimesh children, baked axial geometry and connector overlap; final visual proof is owner-owned.",
                CandidateMode::ItemPropertiesV6 => "V6 resolves the complete Aurora Y/Z/X Item frame, preserves the connected three-part model and validates native full-canvas icon layers; final visual proof is owner-owned.",
                CandidateMode::ItemPropertiesV7 => "V7 preserves the visually accepted V6 model byte-for-byte and rotates only the shared three-layer icon frame by 180 degrees; final visual proof is owner-owned.",
                CandidateMode::ItemPropertiesV8 => "V8 derives controller transforms, triangle-surface slot bounds and the hand origin from exact read-only Vanilla KEY/BIF resources, emits collision-free model 25 parts and includes a real Item plus a separate equipped-render witness; final visual proof is owner-owned.",
                CandidateMode::ItemPropertiesV9 => "V9 packages an exact BaseItem 1 MaxRange override, constrains each generated part to the retail X/Z envelope while preserving axial length, and contains only the real identified Item; final visual proof is owner-owned.",
            }
            .to_owned(),
            parts: specs
                .iter()
                .map(|spec| ItemPartValueV1 {
                    field: spec.field.to_owned(),
                    value: spec.variant,
                })
                .collect(),
            properties: Vec::new(),
            colors: ItemColorValuesV1::default(),
            cost: 0,
            add_cost: 0,
            charges: 0,
            stack_size: 1,
            palette_id: 0,
            identified: true,
            stolen: false,
            cursed: false,
            plot: false,
        },
    )
    .map_err(|error| json_error("TLC-GUARD-LS-UTI-BUILD", &error))?;
    let uti_readback = read_gff_v32(&uti.payload, &GffLimitsV1::default())
        .map_err(|error| format!("TLC-GUARD-LS-UTI-GFF-READBACK: {error}"))?;
    let uti_value = |label: &str| {
        uti_readback
            .root
            .fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
    };
    if uti_readback.file_type != GffFileTypeV1::Uti
        || uti.report.base_item != BASE_ITEM
        || uti_value("BaseItem") != Some(&GffValueV1::Int(BASE_ITEM as i32))
        || uti_value("Identified") != Some(&GffValueV1::Byte(1))
        || uti_value("ModelPart1") != Some(&GffValueV1::Byte(specs[0].variant))
        || uti_value("ModelPart2") != Some(&GffValueV1::Byte(specs[1].variant))
        || uti_value("ModelPart3") != Some(&GffValueV1::Byte(specs[2].variant))
        || uti.report.weapon_color_selector_count != 3
    {
        return Err(format!(
            "TLC-GUARD-LS-UTI-READBACK: UTI must read back as identified BaseItem 1 with exact model/color bytes {}/{}/{}",
            specs[0].variant, specs[1].variant, specs[2].variant,
        ));
    }

    let baseitems_range_override = if mode == CandidateMode::ItemPropertiesV9 {
        let artifact = extend_item_baseitem_model_range_v1(&baseitems, BASE_ITEM, 25)
            .map_err(|error| json_error("TLC-GUARD-LS-BASEITEM-RANGE", &error))?;
        if artifact.report.status != "PATCHED"
            || artifact.report.source_min_range != 10
            || artifact.report.source_max_range != 100
            || artifact.report.effective_max_range != 250
        {
            return Err(format!(
                "TLC-GUARD-LS-BASEITEM-RANGE: expected PATCHED 10/100->250, got {}/{}/{}->{}",
                artifact.report.status,
                artifact.report.source_min_range,
                artifact.report.source_max_range,
                artifact.report.effective_max_range,
            ));
        }
        Some(artifact)
    } else {
        None
    };

    let mut hak_resources = Vec::new();
    for part in &built_parts {
        hak_resources.extend([
            resource(&part.model_resref, 2002, part.mdl.clone()),
            resource(&part.texture_resref, 3, part.texture.clone()),
            resource(&part.icon_resref, 3, part.icon.clone()),
        ]);
    }
    hak_resources.push(resource(config.blueprint_resref, 2025, uti.payload.clone()));
    if let Some(override_artifact) = &baseitems_range_override {
        hak_resources.push(resource(
            "baseitems",
            2017,
            override_artifact.payload.clone(),
        ));
    }
    let expected_hak_resource_count = if baseitems_range_override.is_some() {
        38
    } else {
        37
    };
    if built_parts.len() != 12 || hak_resources.len() != expected_hak_resource_count {
        return Err(format!(
            "TLC-GUARD-LS-COLORWAY-COVERAGE: expected 12 concrete colorways and {expected_hak_resource_count} HAK resources, got {}/{}",
            built_parts.len(),
            hak_resources.len()
        ));
    }
    let hak = write_hak_v1(&hak_resources, &HakWriterOptionsV1::default())
        .map_err(|error| format!("TLC-GUARD-LS-HAK-BUILD: {error}"))?;
    let hak_readback = ErfArchive::parse(&hak.payload)
        .map_err(|error| format!("TLC-GUARD-LS-HAK-READBACK: {error}"))?;
    if hak_readback.file_type() != ErfFileType::Hak
        || hak_readback.resources().len() != hak_resources.len()
    {
        return Err("TLC-GUARD-LS-HAK-SEMANTIC-DIFF: archive type/count differs".to_owned());
    }
    for expected in &hak_resources {
        let readback = hak_readback
            .find(&expected.resref, expected.resource_type)
            .map_err(|error| format!("TLC-GUARD-LS-HAK-RESOURCE-READBACK: {error}"))?;
        if readback != expected.payload.as_slice() {
            return Err(format!(
                "TLC-GUARD-LS-HAK-RESOURCE-DIFF: {}/{} differs",
                expected.resref, expected.resource_type
            ));
        }
    }

    let item_placement = if mode.is_item_only() {
        ItemProofPlacementV1::default()
    } else {
        ItemProofPlacementV1 {
            x: 6.0,
            y: 9.0,
            ..ItemProofPlacementV1::default()
        }
    };
    let creature_placement = ItemProofPlacementV1 {
        x: 8.5,
        y: 9.0,
        ..ItemProofPlacementV1::default()
    };
    let (proof_module_payload, proof_module_report) = match mode {
        CandidateMode::FrozenV3 | CandidateMode::ItemPropertiesV8 => {
            let creature_resref = if mode.uses_reference_profile_v4() {
                "m2atgln8"
            } else {
                CREATURE_RESREF
            };
            let artifact = build_item_and_equipped_proof_module_v3(
                &uti.payload,
                &ItemEquippedProofIdentityV2 {
                    schema_version: 2,
                    module_resref: config.module_resref.to_owned(),
                    area_resref: config.area_resref.to_owned(),
                    hak_resref: config.hak_resref.to_owned(),
                    blueprint_resref: config.blueprint_resref.to_owned(),
                    module_name: config.module_name.to_owned(),
                    area_name: config.area_name.to_owned(),
                    creature_resref: creature_resref.to_owned(),
                    creature_display_name: "Equipped Item model render witness".to_owned(),
                    appearance_row: APPEARANCE_ROW,
                    race: CREATURE_RACE,
                    gender: CREATURE_GENDER,
                    phenotype: i32::from(CREATURE_PHENOTYPE),
                    model_prefix: appearance_binding.model_prefix.clone(),
                    appearance_table_sha256: appearance_binding.appearance_table_sha256.clone(),
                    fixture_profile: ItemEquippedProofProfileV2::ModelType2Parts,
                    equipment_slot,
                },
                item_placement,
                creature_placement,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-MODULE-BUILD", &error))?;
            let report = serde_json::to_value(&artifact.report)
                .map_err(|error| format!("TLC-GUARD-LS-MODULE-REPORT-SERIALIZE: {error}"))?;
            (artifact.payload, report)
        }
        CandidateMode::ItemPropertiesV4
        | CandidateMode::ItemPropertiesV5
        | CandidateMode::ItemPropertiesV6
        | CandidateMode::ItemPropertiesV7
        | CandidateMode::ItemPropertiesV9 => {
            let artifact = build_item_proof_module_v1(
                &uti.payload,
                &ItemProofModuleIdentityV1 {
                    schema_version: 1,
                    module_resref: config.module_resref.to_owned(),
                    area_resref: config.area_resref.to_owned(),
                    hak_resref: config.hak_resref.to_owned(),
                    blueprint_resref: config.blueprint_resref.to_owned(),
                    module_name: config.module_name.to_owned(),
                    area_name: config.area_name.to_owned(),
                },
                item_placement,
            )
            .map_err(|error| json_error("TLC-GUARD-LS-MODULE-BUILD", &error))?;
            let report = serde_json::to_value(&artifact.report)
                .map_err(|error| format!("TLC-GUARD-LS-MODULE-REPORT-SERIALIZE: {error}"))?;
            (artifact.payload, report)
        }
    };
    let module_readback = ErfArchive::parse(&proof_module_payload)
        .map_err(|error| format!("TLC-GUARD-LS-MODULE-READBACK: {error}"))?;
    if module_readback.file_type() != ErfFileType::Module
        || module_readback
            .find(config.blueprint_resref, 2025)
            .map_err(|error| format!("TLC-GUARD-LS-MODULE-UTI-READBACK: {error}"))?
            != uti.payload.as_slice()
    {
        return Err(
            "TLC-GUARD-LS-MODULE-SEMANTIC-DIFF: MOD does not contain the exact UTI".to_owned(),
        );
    }

    fs::create_dir_all(&output_root)
        .map_err(|error| format!("TLC-GUARD-LS-OUTPUT-CREATE: {error}"))?;
    for part in &built_parts {
        write_new(
            &output_root.join(format!("{}.mdl", part.model_resref)),
            &part.mdl,
        )?;
        write_new(
            &output_root.join(format!("{}.tga", part.texture_resref)),
            &part.texture,
        )?;
        write_new(
            &output_root.join(format!("{}.tga", part.icon_resref)),
            &part.icon,
        )?;
    }
    write_new(
        &output_root.join(format!("{}.uti", config.blueprint_resref)),
        &uti.payload,
    )?;
    if let Some(override_artifact) = &baseitems_range_override {
        write_new(
            &output_root.join("baseitems.2da"),
            &override_artifact.payload,
        )?;
    }
    write_new(&output_root.join(config.hak_file_name), &hak.payload)?;
    write_new(
        &output_root.join(config.module_file_name),
        &proof_module_payload,
    )?;
    let fit_bytes = serde_json::to_vec_pretty(&fit.report)
        .map_err(|error| format!("TLC-GUARD-LS-FIT-SERIALIZE: {error}"))?;
    write_new(&output_root.join("fit-report.json"), &fit_bytes)?;
    if let Some(reference) = &retail_reference {
        let profile_bytes = serde_json::to_vec_pretty(&reference.profile)
            .map_err(|error| format!("TLC-GUARD-LS-PROFILE-SERIALIZE: {error}"))?;
        write_new(
            &output_root.join("item-attachment-profile.json"),
            &profile_bytes,
        )?;
    }

    let native_module = Path::new(NWN_USER_ROOT)
        .join("modules")
        .join(config.module_file_name);
    let native_hak = Path::new(NWN_USER_ROOT)
        .join("hak")
        .join(config.hak_file_name);
    let module_install =
        install_exact_new_or_identical(&output_root.join(config.module_file_name), &native_module)?;
    let hak_install =
        install_exact_new_or_identical(&output_root.join(config.hak_file_name), &native_hak)?;

    let resource_manifest = hak_resources
        .iter()
        .map(|entry| {
            serde_json::json!({
                "resref": entry.resref,
                "resourceType": entry.resource_type,
                "byteLength": entry.payload.len(),
                "sha256": item_payload_sha256_v1(&entry.payload),
            })
        })
        .collect::<Vec<_>>();
    let report = serde_json::json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "candidateId": config.candidate_id,
        "assetIdentity": "tlc-guard-longsword-v1",
        "supersedesCandidate": config.supersedes_candidate,
        "admissionEvidence": config.admission_evidence,
        "minimalDelta": config.minimal_delta,
        "candidateProfile": if mode.has_equipped_witness() { "ITEM_AND_EQUIPPED_RENDER_WITNESS_V3" } else { "ITEM_ONLY_GROUND_ITEM_V1" },
        "itemPropertiesModelContract": {
            "partOrder": ["Bottom", "Middle", "Top"],
            "axialTargetAxis": if mode.is_item_only() { "+Y" } else { "+Z" },
            "fitAlgorithm": &fit.algorithm,
            "fitSolutionSha256": &fit.solution_sha256,
            "expectedToolsetResult": match mode {
                CandidateMode::FrozenV3 => "frozen V3 result",
                CandidateMode::ItemPropertiesV4 => "historical V4 upright-axis expectation",
                CandidateMode::ItemPropertiesV5 => "one upright, connected Bottom/Middle/Top sword in Item Properties",
                CandidateMode::ItemPropertiesV6 => "one upright, front-facing, connected Bottom/Middle/Top sword in Item Properties",
                CandidateMode::ItemPropertiesV7 => "the exact accepted V6 model plus one upright icon with blade above guard and pommel",
                CandidateMode::ItemPropertiesV8 => "one connected Bottom/Middle/Top longsword in the exact retail slot frame, with the hand origin inside Bottom and native-size connector overlaps",
                CandidateMode::ItemPropertiesV9 => "one connected, front-facing Bottom/Middle/Top longsword inside the exact retail X/Z slot envelopes, with visible model selectors and no creature witness",
            },
            "orientationFrame": fit.composer_report_v4.as_ref().map(|report| &report.orientation_frame).or_else(|| fit.composer_report_v3.as_ref().map(|report| &report.orientation_frame)),
            "composerConformanceAlgorithm": if mode.uses_reference_profile_v4() {
                Some("ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V4")
            } else if mode.uses_full_frame_v3() {
                Some("ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V3")
            } else if mode.uses_aurora_composer_v2() {
                Some("ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2")
            } else {
                None
            },
            "composerConformance": &composer_conformance,
            "iconConformanceAlgorithm": if mode.uses_full_frame_v3() { Some("ITEM_MODELTYPE2_ICON_LAYER_COMPOSITE_V3") } else { None },
            "iconLayoutProfile": if mode.uses_icon_part_order_v2() {
                Some("LONG_VERTICAL_PART_ORDER_V2")
            } else if mode.uses_full_frame_v3() {
                Some("LONG_VERTICAL_V1")
            } else {
                None
            },
            "iconConformance": &icon_conformance,
            "frozenV6Delta": &frozen_v6_delta,
        },
        "testModuleFileName": config.module_file_name,
        "toolsetModuleName": config.module_name,
        "areaName": config.area_name,
        "areaResref": config.area_resref,
        "orderedHakFiles": [config.hak_file_name],
        "orderedHakResrefs": [config.hak_resref],
        "blueprintResref": config.blueprint_resref,
        "blueprintName": "Last City Bronze Guard Longsword",
        "baseItem": BASE_ITEM,
        "baseItemLabel": base_item.label,
        "itemClass": base_item.item_class,
        "modelType": base_item.model_type,
        "identified": true,
        "partFields": specs.iter().map(|spec| serde_json::json!({
            "field": spec.field,
            "role": spec.role,
            "model": spec.model,
            "color": spec.selected_color,
            "variant": spec.variant,
            "modelResref": spec.model_resref,
            "textureResref": spec.texture_resref,
            "iconResref": spec.icon_resref,
        })).collect::<Vec<_>>(),
        "weaponColorways": built_parts.iter().map(|part| serde_json::json!({
            "field": part.field,
            "role": part.role,
            "color": part.color,
            "variant": part.variant,
            "modelResref": part.model_resref,
            "textureResref": part.texture_resref,
            "iconResref": part.icon_resref,
            "mdlSha256": part.report.mdl_sha256,
            "textureSha256": part.report.texture_sha256,
            "iconSha256": part.report.icon_sha256,
        })).collect::<Vec<_>>(),
        "weaponColorwayCoverage": {
            "status": "COMPLETE",
            "partCount": 3,
            "colors": [1, 2, 3, 4],
            "expectedResourceCount": 12,
            "emittedResourceCount": built_parts.len(),
            "geometryReuse": "ONE_MESHY_GLB_PER_PART",
        },
        "sourceGlbs": source_reports,
        "sourceManifest": canonical_root.join("sample-3d/tlc-guard-longsword-parts-v1/manifest.yaml"),
        "baseitems2da": {
            "logicalSource": if mode.uses_reference_profile_v4() { "nwn_base.key:data/base_2da.bif:baseitems.2da" } else { BASEITEMS_PATH },
            "sha256": item_payload_sha256_v1(&baseitems),
            "mutation": if baseitems_range_override.is_some() { "existing-baseitem-1-maxrange-only" } else { "none-existing-baseitem-1" },
            "rangeOverride": baseitems_range_override.as_ref().map(|artifact| &artifact.report),
        },
        "appearance2da": if mode.has_equipped_witness() {
            Some(serde_json::json!({
                "path": APPEARANCE_PATH,
                "sha256": APPEARANCE_SHA256,
                "row": APPEARANCE_ROW,
                "race": CREATURE_RACE,
                "gender": CREATURE_GENDER,
                "phenotype": CREATURE_PHENOTYPE,
                "modelPrefix": appearance_binding.model_prefix,
            }))
        } else {
            None
        },
        "sourceTriangleCount": source_triangle_count,
        "triangleCount": triangle_count,
        "degenerateTriangleCountRemoved": degenerate_triangle_count_removed,
        "triangleBudget": 300_000,
        "fitTargetAxialLengths": if mode.uses_reference_profile_v4() { None } else { Some(FIT_TARGET_AXIAL_LENGTHS) },
        "fit": &fit.report,
        "iconPresentationFit": icon_fit.as_ref().map(|presentation| serde_json::json!({
            "status": presentation.status,
            "algorithm": presentation.algorithm,
            "solutionSha256": presentation.solution_sha256,
            "targetAxialLengths": FIT_TARGET_AXIAL_LENGTHS,
            "worldAttachmentUnaffected": true,
            "fit": presentation.report,
        })),
        "attachmentProfile": retail_reference.as_ref().map(|reference| &reference.profile),
        "referenceResourceContext": retail_reference.as_ref().map(|reference| &reference.lineage),
        "sharedIconProjectionBounds": projection_bounds,
        "iconSize": icon_size,
        "partReports": built_parts.iter().map(|part| &part.report).collect::<Vec<_>>(),
        "utiReport": uti.report,
        "hakResources": resource_manifest,
        "hak": {
            "path": output_root.join(config.hak_file_name),
            "byteLength": hak.payload.len(),
            "sha256": item_payload_sha256_v1(&hak.payload),
            "semanticReadbackStatus": "PASS",
        },
        "module": {
            "path": output_root.join(config.module_file_name),
            "byteLength": proof_module_payload.len(),
            "sha256": item_payload_sha256_v1(&proof_module_payload),
            "report": proof_module_report,
            "embeddedUtiByteIdentical": true,
        },
        "nativeInstallation": {
            "module": module_install,
            "hak": hak_install,
            "byteIdentical": true,
        },
        "placements": {
            "actualItem": item_placement,
            "equippedRenderWitness": if mode.has_equipped_witness() { Some(creature_placement) } else { None },
        },
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "visualAcceptance": "not_tested",
        "ownerProofRequired": true,
        "agentStartedToolset": false,
        "agentStartedNwn": false,
    });
    let report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("TLC-GUARD-LS-REPORT-SERIALIZE: {error}"))?;
    write_new(
        &output_root.join("ready-for-owner-proof.json"),
        &report_bytes,
    )?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| format!("TLC-GUARD-LS-SUMMARY-SERIALIZE: {error}"))
}

fn part_options(
    transform: m2a_core::item::ItemPartTransformV1,
    target_space_scale_xyz: [f32; 3],
    icon_size: Option<[u32; 2]>,
    icon_projection_bounds: Option<ItemIconProjectionBoundsV1>,
    weapon_color: Option<u8>,
) -> ItemPartBuildOptionsV2 {
    ItemPartBuildOptionsV2 {
        schema_version: 1,
        transform,
        source_node: None,
        texture_encoding: ItemPartTextureEncodingV1::DirectColor,
        icon_size,
        icon_projection_bounds,
        weapon_color,
        target_space_scale_xyz,
    }
}

fn candidate_fit_from_v1(report: ItemFitReportV1) -> Result<CandidateFit, String> {
    let serialized = serde_json::to_value(&report)
        .map_err(|error| format!("TLC-GUARD-LS-FIT-SERIALIZE: {error}"))?;
    Ok(CandidateFit {
        algorithm: report.algorithm.clone(),
        status: report.status.clone(),
        solution_sha256: report.solution_sha256.clone(),
        parts: report
            .parts
            .iter()
            .map(|part| CandidateFitPart {
                field: part.field.clone(),
                source_sha256: part.source_sha256.clone(),
                triangle_count: part.triangle_count,
                transform: part.transform,
                target_space_scale_xyz: [1.0; 3],
            })
            .collect(),
        report: serialized,
        composer_report_v2: None,
        composer_report_v3: None,
        composer_report_v4: None,
    })
}

fn candidate_fit_from_v2(report: ItemFitReportV2) -> Result<CandidateFit, String> {
    let serialized = serde_json::to_value(&report)
        .map_err(|error| format!("TLC-GUARD-LS-FIT-SERIALIZE: {error}"))?;
    Ok(CandidateFit {
        algorithm: report.algorithm.clone(),
        status: report.status.clone(),
        solution_sha256: report.solution_sha256.clone(),
        parts: report
            .parts
            .iter()
            .map(|part| CandidateFitPart {
                field: part.field.clone(),
                source_sha256: part.source_sha256.clone(),
                triangle_count: part.triangle_count,
                transform: part.transform,
                target_space_scale_xyz: part.target_space_scale_xyz,
            })
            .collect(),
        report: serialized,
        composer_report_v2: Some(report),
        composer_report_v3: None,
        composer_report_v4: None,
    })
}

fn candidate_fit_from_v3(report: ItemFitReportV3) -> Result<CandidateFit, String> {
    let serialized = serde_json::to_value(&report)
        .map_err(|error| format!("TLC-GUARD-LS-FIT-SERIALIZE: {error}"))?;
    Ok(CandidateFit {
        algorithm: report.algorithm.clone(),
        status: report.status.clone(),
        solution_sha256: report.solution_sha256.clone(),
        parts: report
            .parts
            .iter()
            .map(|part| CandidateFitPart {
                field: part.field.clone(),
                source_sha256: part.source_sha256.clone(),
                triangle_count: part.triangle_count,
                transform: part.transform,
                target_space_scale_xyz: part.target_space_scale_xyz,
            })
            .collect(),
        report: serialized,
        composer_report_v2: None,
        composer_report_v3: Some(report),
        composer_report_v4: None,
    })
}

fn candidate_fit_from_v4(report: ItemFitReportV4) -> Result<CandidateFit, String> {
    let serialized = serde_json::to_value(&report)
        .map_err(|error| format!("TLC-GUARD-LS-FIT-SERIALIZE: {error}"))?;
    Ok(CandidateFit {
        algorithm: report.algorithm.clone(),
        status: report.status.clone(),
        solution_sha256: report.solution_sha256.clone(),
        parts: report
            .parts
            .iter()
            .map(|part| CandidateFitPart {
                field: part.field.clone(),
                source_sha256: part.source_sha256.clone(),
                triangle_count: part.triangle_count,
                transform: part.transform,
                target_space_scale_xyz: part.target_space_scale_xyz,
            })
            .collect(),
        report: serialized,
        composer_report_v2: None,
        composer_report_v3: None,
        composer_report_v4: Some(report),
    })
}

fn union_bounds(
    current: Option<ItemIconProjectionBoundsV1>,
    next: ItemIconProjectionBoundsV1,
) -> ItemIconProjectionBoundsV1 {
    current
        .map(|current| ItemIconProjectionBoundsV1 {
            min: [
                current.min[0].min(next.min[0]),
                current.min[1].min(next.min[1]),
            ],
            max: [
                current.max[0].max(next.max[0]),
                current.max[1].max(next.max[1]),
            ],
        })
        .unwrap_or(next)
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

fn load_retail_wswls_reference_profile_v1() -> Result<RetailReferenceProfileV1, String> {
    let install_root = Path::new(NWN_INSTALL_ROOT);
    let key_file_name = "nwn_base.key";
    let key_path = install_root.join("data").join(key_file_name);
    let key = fs::read(&key_path).map_err(|error| {
        format!(
            "TLC-GUARD-LS-REFERENCE-KEY-READ: {}: {error}",
            key_path.display()
        )
    })?;
    let base_locator = locate_key_bif_resource_v1(key_file_name, &key, "baseitems", 2017)
        .map_err(|error| format!("TLC-GUARD-LS-REFERENCE-BASEITEMS-LOCATOR: {error}"))?;
    let base_bif_path = install_root.join(base_locator.bif_logical_name.replace('/', "\\"));
    let base_bif = fs::read(&base_bif_path).map_err(|error| {
        format!(
            "TLC-GUARD-LS-REFERENCE-BASEITEMS-BIF-READ: {}: {error}",
            base_bif_path.display()
        )
    })?;
    let base_bif_input = KeyBifFileInputV1 {
        logical_name: base_locator.bif_logical_name.clone(),
        bytes: &base_bif,
    };
    let baseitems =
        resolve_key_bif_resource_sparse_v1(key_file_name, &key, &base_bif_input, "baseitems", 2017)
            .map_err(|error| format!("TLC-GUARD-LS-REFERENCE-BASEITEMS-RESOLVE: {error}"))?;
    if baseitems.payload_sha256 != BASEITEMS_SHA256 {
        return Err(format!(
            "TLC-GUARD-LS-REFERENCE-BASEITEMS-HASH: expected {BASEITEMS_SHA256}, got {}",
            baseitems.payload_sha256
        ));
    }
    let base_item = resolve_item_baseitem_v1(baseitems.payload, BASE_ITEM)
        .map_err(|error| json_error("TLC-GUARD-LS-REFERENCE-BASEITEM", &error))?;

    let reference_specs = [
        ("ModelPart1", "wswls_b_023"),
        ("ModelPart2", "wswls_m_063"),
        ("ModelPart3", "wswls_t_023"),
    ];
    let model_locator = locate_key_bif_resource_v1(key_file_name, &key, reference_specs[0].1, 2002)
        .map_err(|error| format!("TLC-GUARD-LS-REFERENCE-MODEL-LOCATOR: {error}"))?;
    let model_bif_path = install_root.join(model_locator.bif_logical_name.replace('/', "\\"));
    let model_bif = fs::read(&model_bif_path).map_err(|error| {
        format!(
            "TLC-GUARD-LS-REFERENCE-MODEL-BIF-READ: {}: {error}",
            model_bif_path.display()
        )
    })?;
    let model_bif_input = KeyBifFileInputV1 {
        logical_name: model_locator.bif_logical_name.clone(),
        bytes: &model_bif,
    };
    let resolved_models = reference_specs.map(|(field, resref)| {
        let resource =
            resolve_key_bif_resource_sparse_v1(key_file_name, &key, &model_bif_input, resref, 2002)
                .map_err(|error| format!("TLC-GUARD-LS-REFERENCE-MODEL-RESOLVE: {error}"))?;
        if resource.bif_logical_name != model_locator.bif_logical_name {
            return Err(format!(
                "TLC-GUARD-LS-REFERENCE-MODEL-BIF-MISMATCH: {resref} resolved from {}",
                resource.bif_logical_name
            ));
        }
        Ok((field, resref, resource))
    });
    let resolved_models = resolved_models
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?;

    for token in ["b", "m", "t"] {
        for color in 1..=4 {
            for (resref, resource_type) in [
                (format!("wswls_{token}_25{color}"), 2002_u16),
                (format!("iwswls_{token}_25{color}"), 3_u16),
            ] {
                match locate_key_bif_resource_v1(key_file_name, &key, &resref, resource_type) {
                    Ok(locator) => {
                        return Err(format!(
                            "TLC-GUARD-LS-V8-RESOURCE-COLLISION: {resref}/{resource_type} already exists in {}",
                            locator.bif_logical_name
                        ));
                    }
                    Err(error) if error.code == "KEY-BIF-RESOURCE-MISSING" => {}
                    Err(error) => {
                        return Err(format!(
                            "TLC-GUARD-LS-V8-RESOURCE-INVENTORY: {resref}/{resource_type}: {error}"
                        ));
                    }
                }
            }
        }
    }

    let key_sha256 = item_payload_sha256_v1(&key);
    let base_bif_sha256 = item_payload_sha256_v1(&base_bif);
    let model_bif_sha256 = item_payload_sha256_v1(&model_bif);
    let context_identity = serde_json::json!({
        "schemaVersion": 1,
        "algorithm": "NWN_KEY_BIF_SELECTED_ITEM_CONTEXT_V1",
        "key": { "logicalName": key_file_name, "sha256": key_sha256 },
        "selectedBifs": [
            { "logicalName": base_locator.bif_logical_name, "sha256": base_bif_sha256 },
            { "logicalName": model_locator.bif_logical_name, "sha256": model_bif_sha256 },
        ],
    });
    let context_sha256 = item_payload_sha256_v1(
        &serde_json::to_vec(&context_identity)
            .map_err(|error| format!("TLC-GUARD-LS-REFERENCE-CONTEXT-SERIALIZE: {error}"))?,
    );
    let inputs = resolved_models
        .iter()
        .map(|(field, resref, resource)| ItemReferenceMdlInputV1 {
            field,
            model_resref: resref,
            mdl_payload: resource.payload,
        })
        .collect::<Vec<_>>();
    let profile = build_item_attachment_profile_v1(
        &base_item,
        &context_sha256,
        &baseitems.payload_sha256,
        "RETAIL_P_REF",
        "WSwLs 23/63/23",
        &inputs,
    )
    .map_err(|error| json_error("TLC-GUARD-LS-REFERENCE-PROFILE", &error))?;
    let expected_y = [
        [-0.2025382_f32, 0.0841520_f32],
        [0.042125102_f32, 0.1570407_f32],
        [0.1283190_f32, 0.9313860_f32],
    ];
    if profile
        .slots
        .iter()
        .zip(expected_y)
        .any(|(slot, expected)| {
            (slot.bounds_min[1] - expected[0]).abs() > 1.0e-5
                || (slot.bounds_max[1] - expected[1]).abs() > 1.0e-5
        })
    {
        return Err(
            "TLC-GUARD-LS-REFERENCE-PROFILE-GOLDEN-DIFF: exact retail surface bounds changed"
                .to_owned(),
        );
    }
    let model_resources = resolved_models
        .iter()
        .map(|(_, resref, resource)| {
            serde_json::json!({
                "resref": resref,
                "resourceType": 2002,
                "bifLogicalName": resource.bif_logical_name,
                "payloadSha256": resource.payload_sha256,
            })
        })
        .collect::<Vec<_>>();
    let lineage = serde_json::json!({
        "schemaVersion": 1,
        "policy": "NWN_KEY_BIF_SELECTED_ITEM_CONTEXT_V1",
        "contextSha256": context_sha256,
        "key": { "logicalName": key_file_name, "sha256": key_sha256 },
        "selectedBifs": [
            { "logicalName": base_locator.bif_logical_name, "sha256": base_bif_sha256 },
            { "logicalName": model_locator.bif_logical_name, "sha256": model_bif_sha256 },
        ],
        "baseitems": {
            "resref": "baseitems",
            "resourceType": 2017,
            "payloadSha256": baseitems.payload_sha256,
        },
        "referenceModels": model_resources,
        "referencePayloadsCopiedToOutput": false,
    });
    Ok(RetailReferenceProfileV1 {
        baseitems: baseitems.payload.to_vec(),
        base_item,
        profile,
        lineage,
    })
}

fn read_exact(path: &Path, expected_sha256: &str, code: &str) -> Result<Vec<u8>, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("{code}-READ: {}: {error}", path.display()))?;
    let actual = item_payload_sha256_v1(&bytes);
    if actual != expected_sha256 {
        return Err(format!(
            "{code}-HASH: {} expected {expected_sha256}, got {actual}",
            path.display()
        ));
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("TLC-GUARD-LS-WRITE-NEW: {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("TLC-GUARD-LS-WRITE: {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("TLC-GUARD-LS-SYNC: {}: {error}", path.display()))?;
    let readback = fs::read(path)
        .map_err(|error| format!("TLC-GUARD-LS-WRITE-READBACK: {}: {error}", path.display()))?;
    if readback != bytes {
        return Err(format!(
            "TLC-GUARD-LS-WRITE-SEMANTIC-DIFF: {}",
            path.display()
        ));
    }
    Ok(())
}

fn install_exact_new_or_identical(
    source: &Path,
    destination: &Path,
) -> Result<serde_json::Value, String> {
    let source_bytes = fs::read(source).map_err(|error| {
        format!(
            "TLC-GUARD-LS-INSTALL-SOURCE-READ: {}: {error}",
            source.display()
        )
    })?;
    let source_hash = item_payload_sha256_v1(&source_bytes);
    let disposition = if destination.exists() {
        let existing = fs::read(destination).map_err(|error| {
            format!(
                "TLC-GUARD-LS-INSTALL-DESTINATION-READ: {}: {error}",
                destination.display()
            )
        })?;
        if existing != source_bytes {
            return Err(format!(
                "TLC-GUARD-LS-INSTALL-COLLISION: {} exists with SHA-256 {}, expected {}; no overwrite performed",
                destination.display(),
                item_payload_sha256_v1(&existing),
                source_hash
            ));
        }
        "reused_identical"
    } else {
        write_new(destination, &source_bytes)?;
        "created_new"
    };
    let installed = fs::read(destination).map_err(|error| {
        format!(
            "TLC-GUARD-LS-INSTALL-VERIFY-READ: {}: {error}",
            destination.display()
        )
    })?;
    let installed_hash = item_payload_sha256_v1(&installed);
    if installed_hash != source_hash {
        return Err(format!(
            "TLC-GUARD-LS-INSTALL-HASH-MISMATCH: {} source {}, destination {}",
            destination.display(),
            source_hash,
            installed_hash
        ));
    }
    Ok(serde_json::json!({
        "source": source,
        "destination": destination,
        "disposition": disposition,
        "byteLength": source_bytes.len(),
        "sha256": source_hash,
    }))
}

fn json_error<T: Serialize>(code: &str, error: &T) -> String {
    serde_json::to_string(error)
        .map(|json| format!("{code}: {json}"))
        .unwrap_or_else(|serialize_error| format!("{code}: {serialize_error}"))
}
