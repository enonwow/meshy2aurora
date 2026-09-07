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
    inspect_binary_mdl,
    mdl::NodeReport,
    model_pipeline::{
        CreatureHeldWeaponModeV1, CreatureHeldWeaponOptionsV1, ProceduralCreatureBuildOptionsV1,
        ProceduralCreatureProductIdentityV2,
        build_meshy_procedural_humanoid_product_with_options_v3,
        build_procedural_creature_demo_with_held_weapon_v3,
    },
    proof_module::{
        BinaryCreatureHandSlotV1, BinaryCreatureModuleIdentityV1, NWN_BASE_SHORTSWORD_RESREF_V2,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7";
const APPEARANCE_SHA256: &str = "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const EXPECTED_TRIANGLE_COUNT: usize = 19_704;
const EXPECTED_ANIMATION_COUNT: usize = 42;

const MODULE_RESREF: &str = "m2aweapdemo9";
const AREA_RESREF: &str = "m2aweaparea9";
const HAK_RESREF: &str = "m2aweaphak9";
const MODEL_RESREF: &str = "m2aweapcre9";
const TEXTURE_RESREF: &str = "m2aweaptex9";
const CREATURE_RESREF: &str = "m2awrhand9";

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
            "CREATURE-HELD-ITEM-V9-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }
    let source = read_exact(&source_path, SOURCE_SHA256, "SOURCE")?;
    let appearance = read_exact(&appearance_path, APPEARANCE_SHA256, "APPEARANCE")?;
    let held_weapon = CreatureHeldWeaponOptionsV1 {
        schema_version: 1,
        mode: CreatureHeldWeaponModeV1::RightHand,
        item_resref: Some(NWN_BASE_SHORTSWORD_RESREF_V2.to_owned()),
    };
    let build_options = ProceduralCreatureBuildOptionsV1 {
        held_weapon: held_weapon.clone(),
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
    let product = build_meshy_procedural_humanoid_product_with_options_v3(
        &source,
        &appearance,
        &ProceduralCreatureProductIdentityV2 {
            model_resref: MODEL_RESREF.to_owned(),
            texture_resref: TEXTURE_RESREF.to_owned(),
            hak_resref: HAK_RESREF.to_owned(),
            appearance_label: "M2A_CREATURE_HELD_ITEM_V9".to_owned(),
        },
        &build_options,
    )
    .map_err(|error| format!("CREATURE-HELD-ITEM-V9-PRODUCT: {error}"))?;
    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: MODULE_RESREF.to_owned(),
        area_resref: AREA_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
    };
    let demo = build_procedural_creature_demo_with_held_weapon_v3(
        &product,
        &module_identity,
        CREATURE_RESREF,
        &held_weapon,
    )
    .map_err(|error| format!("CREATURE-HELD-ITEM-V9-MODULE: {error}"))?;

    if product.report.geometry.triangle_count != EXPECTED_TRIANGLE_COUNT {
        return Err(format!(
            "CREATURE-HELD-ITEM-V9-TRIANGLE-DIFF: expected {EXPECTED_TRIANGLE_COUNT}, got {}",
            product.report.geometry.triangle_count
        ));
    }
    let model_readback = inspect_binary_mdl(&product.model)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V9-MDL-READBACK: {error:?}"))?;
    if model_readback.animations.len() != EXPECTED_ANIMATION_COUNT {
        return Err(format!(
            "CREATURE-HELD-ITEM-V9-ANIMATION-DIFF: expected {EXPECTED_ANIMATION_COUNT}, got {}",
            model_readback.animations.len()
        ));
    }
    let right_parent =
        find_parent_name(&model_readback.node_tree.roots, AURORA_RIGHT_HAND_ANCHOR_V1)
            .ok_or_else(|| "CREATURE-HELD-ITEM-V9-RHAND-MISSING".to_owned())?;
    let left_parent = find_parent_name(&model_readback.node_tree.roots, AURORA_LEFT_HAND_ANCHOR_V1)
        .ok_or_else(|| "CREATURE-HELD-ITEM-V9-LHAND-MISSING".to_owned())?;
    if !right_parent.eq_ignore_ascii_case("RightHand")
        || !left_parent.eq_ignore_ascii_case("LeftHand")
    {
        return Err(format!(
            "CREATURE-HELD-ITEM-V9-HOOK-PARENT-DIFF: rhand={right_parent}, lhand={left_parent}"
        ));
    }
    if model_readback.animations.iter().any(|animation| {
        !contains_node(&animation.node_tree.roots, AURORA_RIGHT_HAND_ANCHOR_V1)
            || !contains_node(&animation.node_tree.roots, AURORA_LEFT_HAND_ANCHOR_V1)
    }) {
        return Err("CREATURE-HELD-ITEM-V9-ANIMATION-HOOK-COVERAGE-DIFF".to_owned());
    }

    let grip = product
        .report
        .weapon_anchor_authoring
        .as_ref()
        .and_then(|report| report.grip_adjustment.as_ref())
        .ok_or_else(|| "CREATURE-HELD-ITEM-V9-GRIP-READBACK-MISSING".to_owned())?;
    if grip.mode != CreatureWeaponGripModeV1::AutoPlusOffsets
        || grip.right_hand.requested.roll_degrees != 90.0
        || grip.right_hand.requested.pitch_degrees != 0.0
        || grip.right_hand.requested.yaw_degrees != 0.0
    {
        return Err("CREATURE-HELD-ITEM-V9-GRIP-READBACK-DIFF".to_owned());
    }

    let appearance_row = product.report.appearance.appended_row_index;
    let appearance_limits = TwoDaLimitsV1::default();
    let appearance_inspection =
        inspect_two_da_v2(&product.appearance_two_da, &appearance_limits)
            .map_err(|error| format!("CREATURE-HELD-ITEM-V9-APPEARANCE-INSPECT: {error}"))?;
    let row = read_two_da_row_v2(
        &product.appearance_two_da,
        u32::from(appearance_row),
        &appearance_limits,
    )
    .map_err(|error| format!("CREATURE-HELD-ITEM-V9-APPEARANCE-ROW: {error}"))?;
    let model_type_index = appearance_inspection
        .columns
        .iter()
        .position(|column| column.eq_ignore_ascii_case("MODELTYPE"))
        .ok_or_else(|| "CREATURE-HELD-ITEM-V9-MODELTYPE-COLUMN-MISSING".to_owned())?;
    let model_type = row
        .cells
        .get(model_type_index)
        .and_then(|cell| match cell {
            TwoDaCellValueV1::Text { value } => Some(value.as_str()),
            TwoDaCellValueV1::Null => None,
        })
        .ok_or_else(|| "CREATURE-HELD-ITEM-V9-MODELTYPE-MISSING".to_owned())?;
    if model_type != "L" {
        return Err(format!(
            "CREATURE-HELD-ITEM-V9-MODELTYPE-DIFF: expected L, got {model_type}"
        ));
    }

    if demo.report.held_weapon_readback.is_some() {
        return Err("CREATURE-HELD-ITEM-V9-OWNED-UTI-READBACK-UNEXPECTED".to_owned());
    }
    let equipment = demo
        .report
        .held_stock_weapon_readback
        .as_ref()
        .ok_or_else(|| "CREATURE-HELD-ITEM-V9-EQUIPMENT-READBACK-MISSING".to_owned())?;
    if equipment.weapon.resref != NWN_BASE_SHORTSWORD_RESREF_V2
        || equipment.weapon.resource_type != 2025
        || equipment.fixtures.len() != 1
        || equipment.fixtures[0].hand != BinaryCreatureHandSlotV1::RightHand
        || equipment.fixtures[0].equipped_item_resref != NWN_BASE_SHORTSWORD_RESREF_V2
    {
        return Err("CREATURE-HELD-ITEM-V9-EQUIPMENT-READBACK-DIFF".to_owned());
    }

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "CREATURE-HELD-ITEM-V9-OUTPUT-CREATE {}: {error}",
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
        "iterationAdmission": "V8_OWNER_BOUND_NWN_FUNCTIONAL_VISUAL_FAILURE_2026_08_17",
        "testModuleFileName": format!("{MODULE_RESREF}.mod"),
        "moduleDisplayName": demo.report.module_display_name,
        "areaDisplayName": demo.report.area_display_name,
        "moduleResref": MODULE_RESREF,
        "areaResref": AREA_RESREF,
        "hakResref": HAK_RESREF,
        "modelResref": MODEL_RESREF,
        "textureResref": TEXTURE_RESREF,
        "creatureResref": CREATURE_RESREF,
        "heldItemResref": NWN_BASE_SHORTSWORD_RESREF_V2,
        "heldItemScope": "NWN_BASE_GAME",
        "appearanceRow": appearance_row,
        "appearanceModelType": model_type,
        "placement": { "x": 10.0, "y": 14.5, "z": 0.0, "facingX": 0.0, "facingY": -1.0 },
        "source": binding(source_path.display().to_string(), &source),
        "sourceRole": "OWNER_PROVED_NON_FACING_MATRIX_HUMANOID",
        "inputAppearanceTwoDa": binding(appearance_path.display().to_string(), &appearance),
        "triangleCount": product.report.geometry.triangle_count,
        "animationCount": model_readback.animations.len(),
        "animationChangedForHeldItem": false,
        "weaponGrip": {
            "mode": "AUTO_PLUS_OFFSETS",
            "rightHand": { "rollDegrees": 90.0, "pitchDegrees": 0.0, "yawDegrees": 0.0 },
            "leftHand": { "rollDegrees": 0.0, "pitchDegrees": 0.0, "yawDegrees": 0.0 }
        },
        "hooks": [
            { "name": AURORA_RIGHT_HAND_ANCHOR_V1, "parent": right_parent },
            { "name": AURORA_LEFT_HAND_ANCHOR_V1, "parent": left_parent }
        ],
        "demoReport": demo.report,
        "generatedFiles": generated_files,
        "startsToolset": false,
        "startsNwn": false
    });
    let materialization_bytes = serde_json::to_vec_pretty(&materialization)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V9-MATERIALIZATION-JSON: {error}"))?;
    write_new(
        &output_directory.join("materialization.json"),
        &materialization_bytes,
    )?;
    serde_json::to_string_pretty(&materialization)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V9-STDOUT-JSON: {error}"))
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
                    "CREATURE-HELD-ITEM-V9-ARGUMENT-UNKNOWN: {argument}"
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "CREATURE-HELD-ITEM-V9-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!(
                "CREATURE-HELD-ITEM-V9-ARGUMENT-MISSING: {argument}"
            ));
        }
    }
    Ok((
        PathBuf::from(source.ok_or("CREATURE-HELD-ITEM-V9-SOURCE-MISSING")?),
        PathBuf::from(appearance.ok_or("CREATURE-HELD-ITEM-V9-APPEARANCE-MISSING")?),
        PathBuf::from(output.ok_or("CREATURE-HELD-ITEM-V9-OUTPUT-MISSING")?),
    ))
}

fn read_exact(path: &Path, expected_sha256: &str, role: &str) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "CREATURE-HELD-ITEM-V9-{role}-READ {}: {error}",
            path.display()
        )
    })?;
    let actual = sha256(&bytes);
    if actual != expected_sha256 {
        return Err(format!(
            "CREATURE-HELD-ITEM-V9-{role}-HASH-DIFF: expected {expected_sha256}, got {actual}"
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
        .map_err(|error| format!("CREATURE-HELD-ITEM-V9-WRITE {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("CREATURE-HELD-ITEM-V9-WRITE {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("CREATURE-HELD-ITEM-V9-SYNC {}: {error}", path.display()))
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
