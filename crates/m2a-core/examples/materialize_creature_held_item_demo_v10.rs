use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    creature_equipment::{
        AURORA_LEFT_HAND_ANCHOR_V1, AURORA_RIGHT_HAND_ANCHOR_V1, CreatureWeaponEulerOffsetV1,
        CreatureWeaponGripModeV1, CreatureWeaponGripOptionsV1,
    },
    direct_creature_animation::has_preserved_source_combat_attacks_v1,
    inspect_binary_mdl,
    mdl::NodeReport,
    model_pipeline::{
        ProceduralCreatureBuildOptionsV1, ProceduralCreaturePackageIdentityV1,
        build_meshy_procedural_humanoid_p300k_experiment_with_options_v2,
    },
    proof_module::{
        BinaryCreatureEmbeddedStockWeaponV3, BinaryCreatureEquippedFixtureV1,
        BinaryCreatureHandSlotV1, BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1,
        BinaryCreatureProfiledFixtureV2, BinaryCreatureRuntimeProfileV2, M0RuntimeDirectionV1,
        M0RuntimePositionV1, NWN_BASE_BASTARD_SWORD_RESREF_V3,
        NWN_FEAT_WEAPON_PROFICIENCY_EXOTIC_V3,
        build_binary_creature_embedded_stock_weapon_demo_module_named_v4,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3";
const APPEARANCE_SHA256: &str = "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const EXPECTED_TRIANGLE_COUNT: usize = 296_276;
const EXPECTED_ANIMATION_COUNT: usize = 42;
const EXPECTED_SOURCE_CLIP_COUNT: u32 = 10;

const MODULE_RESREF: &str = "m2aweapdemo10";
const AREA_RESREF: &str = "m2aweaparea10";
const HAK_RESREF: &str = "m2aweaphak10";
const MODEL_RESREF: &str = "m2aweapcre10";
const TEXTURE_RESREF: &str = "m2aweaptex10";
const CREATURE_RESREF: &str = "m2awrhand10";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora correct held-item demo V10";
const AREA_DISPLAY_NAME: &str = "Meshy2Aurora correct held-item test area V10";
const CREATURE_DISPLAY_NAME: &str = "Stoneback Brute with owner-corrected held item";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ByteBinding {
    file_name: String,
    byte_length: u64,
    sha256: String,
}

fn main() -> ExitCode {
    match run() {
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

fn run() -> Result<String, String> {
    let (source_path, appearance_path, output_directory) = parse_args(env::args().skip(1))?;
    if output_directory.exists() {
        return Err(format!(
            "CREATURE-HELD-ITEM-V10-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }
    let source = read_exact(&source_path, SOURCE_SHA256, "SOURCE")?;
    let appearance = read_exact(&appearance_path, APPEARANCE_SHA256, "APPEARANCE")?;

    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: MODULE_RESREF.to_owned(),
        area_resref: AREA_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
    };
    let runtime_identity = ProceduralCreaturePackageIdentityV1 {
        model_resref: MODEL_RESREF.to_owned(),
        texture_resref: TEXTURE_RESREF.to_owned(),
        module: module_identity.clone(),
        creature_resref: CREATURE_RESREF.to_owned(),
    };
    let build_options = ProceduralCreatureBuildOptionsV1 {
        texture_artifact_cleanup: true,
        weapon_grip: CreatureWeaponGripOptionsV1 {
            schema_version: 1,
            item_family: m2a_core::creature_product::CreatureItemGripFamilyV1::Sword,
            mode: CreatureWeaponGripModeV1::AutoPlusOffsets,
            right_hand: CreatureWeaponEulerOffsetV1 {
                roll_degrees: 90.0,
                ..CreatureWeaponEulerOffsetV1::default()
            },
            left_hand: CreatureWeaponEulerOffsetV1::default(),
        },
        ..Default::default()
    };
    let product = build_meshy_procedural_humanoid_p300k_experiment_with_options_v2(
        &source,
        &appearance,
        &runtime_identity,
        &build_options,
    )
    .map_err(|error| format!("CREATURE-HELD-ITEM-V10-PRODUCT: {error}"))?;

    if product.report.geometry.triangle_count != EXPECTED_TRIANGLE_COUNT {
        return Err(format!(
            "CREATURE-HELD-ITEM-V10-TRIANGLE-DIFF: expected {EXPECTED_TRIANGLE_COUNT}, got {}",
            product.report.geometry.triangle_count
        ));
    }
    if product.report.conversion.policies.basis_status != "CREATURE_BASIS_V3_RESOLVED"
        || product.report.conversion.policies.asset_forward_mapping
            != "GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y"
        || product.report.conversion.policies.engine_facing_proof != "OWNER_PROOF_REQUIRED"
        || (product.report.conversion.transform.determinant - 1.0).abs() > 1.0e-6
    {
        return Err("CREATURE-HELD-ITEM-V10-FACING-CONTRACT-DIFF".to_owned());
    }
    let completeness = product
        .report
        .animation_completeness
        .as_ref()
        .ok_or_else(|| "CREATURE-HELD-ITEM-V10-ANIMATION-LINEAGE-MISSING".to_owned())?;
    if completeness.input_source_clip_count != EXPECTED_SOURCE_CLIP_COUNT
        || !has_preserved_source_combat_attacks_v1(&completeness.clips)
    {
        return Err("CREATURE-HELD-ITEM-V10-SOURCE-COMBAT-ATTACKS-MISSING".to_owned());
    }

    let model_readback = inspect_binary_mdl(&product.model)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-MDL-READBACK: {error:?}"))?;
    if model_readback.animations.len() != EXPECTED_ANIMATION_COUNT {
        return Err(format!(
            "CREATURE-HELD-ITEM-V10-ANIMATION-DIFF: expected {EXPECTED_ANIMATION_COUNT}, got {}",
            model_readback.animations.len()
        ));
    }
    for attack in ["ca1slashl", "ca1slashr"] {
        if !model_readback
            .animations
            .iter()
            .any(|animation| animation.name.eq_ignore_ascii_case(attack))
        {
            return Err(format!(
                "CREATURE-HELD-ITEM-V10-ATTACK-READBACK-MISSING: {attack}"
            ));
        }
    }
    let right_parent =
        find_parent_name(&model_readback.node_tree.roots, AURORA_RIGHT_HAND_ANCHOR_V1)
            .ok_or_else(|| "CREATURE-HELD-ITEM-V10-RHAND-MISSING".to_owned())?;
    let left_parent = find_parent_name(&model_readback.node_tree.roots, AURORA_LEFT_HAND_ANCHOR_V1)
        .ok_or_else(|| "CREATURE-HELD-ITEM-V10-LHAND-MISSING".to_owned())?;
    if !right_parent.eq_ignore_ascii_case("RightHand")
        || !left_parent.eq_ignore_ascii_case("LeftHand")
    {
        return Err(format!(
            "CREATURE-HELD-ITEM-V10-HOOK-PARENT-DIFF: rhand={right_parent}, lhand={left_parent}"
        ));
    }
    if model_readback.animations.iter().any(|animation| {
        !contains_node(&animation.node_tree.roots, AURORA_RIGHT_HAND_ANCHOR_V1)
            || !contains_node(&animation.node_tree.roots, AURORA_LEFT_HAND_ANCHOR_V1)
    }) {
        return Err("CREATURE-HELD-ITEM-V10-ANIMATION-HOOK-COVERAGE-DIFF".to_owned());
    }

    let appearance_row = product.report.appearance.appended_row_index;
    let model_type = read_model_type(&product.appearance_two_da, appearance_row)?;
    if model_type != "L" {
        return Err(format!(
            "CREATURE-HELD-ITEM-V10-MODELTYPE-DIFF: expected L, got {model_type}"
        ));
    }

    let weapon = BinaryCreatureEmbeddedStockWeaponV3::nwn_base_bastard_sword();
    let fixtures = [BinaryCreatureEquippedFixtureV1 {
        profiled_fixture: BinaryCreatureProfiledFixtureV2 {
            fixture: BinaryCreatureOwnedFixtureV1 {
                id: "m2a_procedural_creature_v10".to_owned(),
                template_resref: CREATURE_RESREF.to_owned(),
                display_name: CREATURE_DISPLAY_NAME.to_owned(),
                appearance_row,
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
    let demo = build_binary_creature_embedded_stock_weapon_demo_module_named_v4(
        &module_identity,
        &fixtures,
        &weapon,
        MODULE_DISPLAY_NAME,
        AREA_DISPLAY_NAME,
        "Stoneback Brute combat demo using the owner-corrected embedded held-item contract.",
    )
    .map_err(|error| format!("CREATURE-HELD-ITEM-V10-MODULE: {error}"))?;
    if demo.readback.schema_version != 3
        || demo.readback.weapon.resref != NWN_BASE_BASTARD_SWORD_RESREF_V3
        || demo.readback.proficiency_feat != NWN_FEAT_WEAPON_PROFICIENCY_EXOTIC_V3
        || demo.readback.fixtures != fixtures
    {
        return Err("CREATURE-HELD-ITEM-V10-EQUIPMENT-READBACK-DIFF".to_owned());
    }

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "CREATURE-HELD-ITEM-V10-OUTPUT-CREATE {}: {error}",
            output_directory.display()
        )
    })?;
    let files = [
        (format!("{MODEL_RESREF}.mdl"), product.model.as_slice()),
        (format!("{TEXTURE_RESREF}.tga"), product.texture.as_slice()),
        (
            "appearance.2da".to_owned(),
            product.appearance_two_da.as_slice(),
        ),
        (format!("{HAK_RESREF}.hak"), product.hak.as_slice()),
        (format!("{MODULE_RESREF}.mod"), demo.payload.as_slice()),
        (
            "materialization-report.json".to_owned(),
            product.report_json.as_slice(),
        ),
        ("summary.json".to_owned(), product.summary_json.as_slice()),
        (
            "conversion-manifest.json".to_owned(),
            product.manifest_json.as_slice(),
        ),
    ];
    let mut generated_files = Vec::with_capacity(files.len());
    for (file_name, bytes) in files {
        write_new(&output_directory.join(&file_name), bytes)?;
        generated_files.push(binding(file_name, bytes));
    }

    let materialization = serde_json::json!({
        "schemaVersion": 1,
        "status": "materialized_offline",
        "iterationAdmission": "V9_OWNER_REPORTED_WRONG_FACING_NO_FUNCTIONAL_ATTACK_AND_MODULE_EQUIPMENT_REGRESSION_2026_08_18",
        "testModuleFileName": format!("{MODULE_RESREF}.mod"),
        "moduleDisplayName": MODULE_DISPLAY_NAME,
        "areaDisplayName": AREA_DISPLAY_NAME,
        "moduleResref": MODULE_RESREF,
        "areaResref": AREA_RESREF,
        "hakResref": HAK_RESREF,
        "modelResref": MODEL_RESREF,
        "textureResref": TEXTURE_RESREF,
        "creatureResref": CREATURE_RESREF,
        "creatureDisplayName": CREATURE_DISPLAY_NAME,
        "heldItemResref": NWN_BASE_BASTARD_SWORD_RESREF_V3,
        "heldItemPlacement": "FULL_EMBEDDED_GIT_ITEM_INSTANCE",
        "heldItemScope": "NWN_BASE_GAME",
        "heldItemBaseItem": weapon.base_item,
        "heldItemModelParts": weapon.model_parts,
        "requiredProficiencyFeat": weapon.required_proficiency_feat,
        "appearanceRow": appearance_row,
        "appearanceModelType": model_type,
        "placement": { "x": 10.0, "y": 14.5, "z": 0.0, "facingX": 0.0, "facingY": -1.0 },
        "source": binding(source_path.display().to_string(), &source),
        "sourceRole": "OWNER_RUNTIME_VERIFIED_STONEBACK_V5_WITH_REAL_MESHY_ATTACKS",
        "priorOwnerProof": {
            "candidate": "tlc-stoneback-brute-p300k-geometry-ab-v5",
            "toolsetModelVisibility": "visible",
            "nwnModelVisibility": "visible",
            "ownerDescription": "appears correctly and looks very good",
            "sourceForwardMapping": "GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y",
            "placementFacing": [0.0, -1.0]
        },
        "inputAppearanceTwoDa": binding(appearance_path.display().to_string(), &appearance),
        "triangleCount": product.report.geometry.triangle_count,
        "animationCount": model_readback.animations.len(),
        "inputSourceClipCount": completeness.input_source_clip_count,
        "sourceCombatClips": ["ca1slashl", "ca1slashr"],
        "sourceCombatGate": "PASS_PRESERVED_SOURCE",
        "creatureBasis": {
            "status": product.report.conversion.policies.basis_status,
            "mapping": product.report.conversion.policies.asset_forward_mapping,
            "determinant": product.report.conversion.transform.determinant,
            "engineFacingProof": product.report.conversion.policies.engine_facing_proof
        },
        "weaponGrip": {
            "mode": "AUTO_PLUS_OFFSETS",
            "rightHand": { "rollDegrees": 90.0, "pitchDegrees": 0.0, "yawDegrees": 0.0 },
            "animationChangedForHeldItem": false
        },
        "hooks": [
            { "name": AURORA_RIGHT_HAND_ANCHOR_V1, "parent": right_parent },
            { "name": AURORA_LEFT_HAND_ANCHOR_V1, "parent": left_parent }
        ],
        "embeddedDemoReadback": demo.readback,
        "generatedFiles": generated_files,
        "startsToolset": false,
        "startsNwn": false
    });
    let materialization_bytes = serde_json::to_vec_pretty(&materialization)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-MATERIALIZATION-JSON: {error}"))?;
    write_new(
        &output_directory.join("materialization.json"),
        &materialization_bytes,
    )?;
    serde_json::to_string_pretty(&materialization)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-STDOUT-JSON: {error}"))
}

fn read_model_type(bytes: &[u8], physical_row: u16) -> Result<String, String> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-APPEARANCE-INSPECT: {error}"))?;
    let row = read_two_da_row_v2(bytes, u32::from(physical_row), &limits)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-APPEARANCE-ROW: {error}"))?;
    let index = inspection
        .columns
        .iter()
        .position(|column| column.eq_ignore_ascii_case("MODELTYPE"))
        .ok_or_else(|| "CREATURE-HELD-ITEM-V10-MODELTYPE-COLUMN-MISSING".to_owned())?;
    row.cells
        .get(index)
        .and_then(|cell| match cell {
            TwoDaCellValueV1::Text { value } => Some(value.clone()),
            TwoDaCellValueV1::Null => None,
        })
        .ok_or_else(|| "CREATURE-HELD-ITEM-V10-MODELTYPE-MISSING".to_owned())
}

fn find_parent_name<'a>(nodes: &'a [NodeReport], target: &str) -> Option<&'a str> {
    for node in nodes {
        if node
            .children
            .iter()
            .any(|child| child.name.eq_ignore_ascii_case(target))
        {
            return Some(&node.name);
        }
        if let Some(parent) = find_parent_name(&node.children, target) {
            return Some(parent);
        }
    }
    None
}

fn contains_node(nodes: &[NodeReport], target: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.name.eq_ignore_ascii_case(target) || contains_node(&node.children, target))
}

fn parse_args(
    arguments: impl IntoIterator<Item = String>,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let mut source = None;
    let mut appearance = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source,
            "--appearance-2da" => &mut appearance,
            "--out" => &mut output,
            _ => {
                return Err(format!(
                    "CREATURE-HELD-ITEM-V10-ARGUMENT-UNKNOWN: {argument}"
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "CREATURE-HELD-ITEM-V10-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!(
                "CREATURE-HELD-ITEM-V10-ARGUMENT-MISSING: {argument}"
            ));
        }
    }
    Ok((
        PathBuf::from(source.ok_or("CREATURE-HELD-ITEM-V10-SOURCE-MISSING")?),
        PathBuf::from(appearance.ok_or("CREATURE-HELD-ITEM-V10-APPEARANCE-MISSING")?),
        PathBuf::from(output.ok_or("CREATURE-HELD-ITEM-V10-OUTPUT-MISSING")?),
    ))
}

fn read_exact(path: &Path, expected_sha256: &str, role: &str) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "CREATURE-HELD-ITEM-V10-{role}-READ {}: {error}",
            path.display()
        )
    })?;
    let actual = sha256(&bytes);
    if actual != expected_sha256 {
        return Err(format!(
            "CREATURE-HELD-ITEM-V10-{role}-HASH-DIFF: expected {expected_sha256}, got {actual}"
        ));
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-WRITE {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-WRITE {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("CREATURE-HELD-ITEM-V10-SYNC {}: {error}", path.display()))
}

fn binding(file_name: String, bytes: &[u8]) -> ByteBinding {
    ByteBinding {
        file_name,
        byte_length: bytes.len() as u64,
        sha256: sha256(bytes),
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
