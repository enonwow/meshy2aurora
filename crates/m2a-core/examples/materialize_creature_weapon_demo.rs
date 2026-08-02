use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    creature_equipment::{AURORA_LEFT_HAND_ANCHOR_V1, AURORA_RIGHT_HAND_ANCHOR_V1},
    inspect_binary_mdl,
    mdl::NodeReport,
    model_pipeline::{
        ProceduralCreatureBuildOptionsV1, ProceduralCreatureProductIdentityV2,
        build_meshy_procedural_humanoid_product_with_options_v3,
    },
    proof_module::{
        BinaryCreatureEquippedFixtureV1, BinaryCreatureHandSlotV1, BinaryCreatureModuleIdentityV1,
        BinaryCreatureOwnedFixtureV1, BinaryCreatureProfiledFixtureV2,
        BinaryCreatureRuntimeProfileV2, BinaryCreatureStockWeaponV2, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_stock_weapon_demo_module_v2,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MODULE_RESREF: &str = "m2aweapdemo2";
const AREA_RESREF: &str = "m2aweaparea2";
const HAK_RESREF: &str = "m2aweaphak2";
const MODEL_RESREF: &str = "m2aweapcre2";
const TEXTURE_RESREF: &str = "m2aweaptex2";
const RIGHT_CREATURE_RESREF: &str = "m2awrhand2";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora Creature Weapon Attachment V2";
const AREA_DISPLAY_NAME: &str = "Meshy2Aurora Creature Weapon Test V2";

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
            "CREATURE-WEAPON-DEMO-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }
    let source = read(&source_path, "SOURCE")?;
    let appearance = read(&appearance_path, "APPEARANCE")?;
    let identity = ProceduralCreatureProductIdentityV2 {
        model_resref: MODEL_RESREF.to_owned(),
        texture_resref: TEXTURE_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
        appearance_label: "M2A_CREATURE_WEAPON_ANCHOR_V2".to_owned(),
    };
    let product = build_meshy_procedural_humanoid_product_with_options_v3(
        &source,
        &appearance,
        &identity,
        // Use the canonical source's declared/default +Z forward contract.
        // This demo does not select any of the facing-matrix experiment variants.
        &ProceduralCreatureBuildOptionsV1::default(),
    )
    .map_err(|error| format!("CREATURE-WEAPON-DEMO-PIPELINE: {error}"))?;

    let model_readback = inspect_binary_mdl(&product.model)
        .map_err(|error| format!("CREATURE-WEAPON-DEMO-MDL-READBACK: {error:?}"))?;
    let right_parent =
        find_parent_name(&model_readback.node_tree.roots, AURORA_RIGHT_HAND_ANCHOR_V1)
            .ok_or_else(|| "CREATURE-WEAPON-DEMO-RHAND-ANCHOR-MISSING".to_owned())?;
    let left_parent = find_parent_name(&model_readback.node_tree.roots, AURORA_LEFT_HAND_ANCHOR_V1)
        .ok_or_else(|| "CREATURE-WEAPON-DEMO-LHAND-ANCHOR-MISSING".to_owned())?;
    if !right_parent.eq_ignore_ascii_case("RightHand")
        || !left_parent.eq_ignore_ascii_case("LeftHand")
    {
        return Err(format!(
            "CREATURE-WEAPON-DEMO-ANCHOR-PARENT-DIFF: rhand_g={right_parent}, lhand_g={left_parent}"
        ));
    }
    let animation_anchor_coverage = model_readback
        .animations
        .iter()
        .filter(|animation| {
            contains_node(&animation.node_tree.roots, AURORA_RIGHT_HAND_ANCHOR_V1)
                && contains_node(&animation.node_tree.roots, AURORA_LEFT_HAND_ANCHOR_V1)
        })
        .count();
    if animation_anchor_coverage != model_readback.animations.len() {
        return Err(format!(
            "CREATURE-WEAPON-DEMO-ANIMATION-ANCHOR-DIFF: {animation_anchor_coverage}/{}",
            model_readback.animations.len()
        ));
    }
    let appearance_limits = TwoDaLimitsV1::default();
    let appearance_inspection =
        inspect_two_da_v2(&product.appearance_two_da, &appearance_limits)
            .map_err(|error| format!("CREATURE-WEAPON-DEMO-APPEARANCE-INSPECTION: {error}"))?;
    let appearance_row = read_two_da_row_v2(
        &product.appearance_two_da,
        u32::from(product.report.appearance.appended_row_index),
        &appearance_limits,
    )
    .map_err(|error| format!("CREATURE-WEAPON-DEMO-APPEARANCE-READBACK: {error}"))?;
    let model_type_index = appearance_inspection
        .columns
        .iter()
        .position(|column| column.eq_ignore_ascii_case("MODELTYPE"))
        .ok_or_else(|| "CREATURE-WEAPON-DEMO-MODELTYPE-COLUMN-MISSING".to_owned())?;
    let model_type = appearance_row
        .cells
        .get(model_type_index)
        .and_then(|cell| match cell {
            TwoDaCellValueV1::Text { value } => Some(value.as_str()),
            TwoDaCellValueV1::Null => None,
        })
        .ok_or_else(|| "CREATURE-WEAPON-DEMO-MODELTYPE-MISSING".to_owned())?;
    if model_type != "L" {
        return Err(format!(
            "CREATURE-WEAPON-DEMO-MODELTYPE-DIFF: expected L, got {model_type}"
        ));
    }

    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: MODULE_RESREF.to_owned(),
        area_resref: AREA_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
    };
    let weapon = BinaryCreatureStockWeaponV2::nwn_base_shortsword();
    let make_fixture = |id: &str,
                        template_resref: &str,
                        display_name: &str,
                        hand: BinaryCreatureHandSlotV1,
                        x: f32| BinaryCreatureEquippedFixtureV1 {
        profiled_fixture: BinaryCreatureProfiledFixtureV2 {
            fixture: BinaryCreatureOwnedFixtureV1 {
                id: id.to_owned(),
                template_resref: template_resref.to_owned(),
                display_name: display_name.to_owned(),
                appearance_row: product.report.appearance.appended_row_index,
                position: M0RuntimePositionV1 { x, y: 14.5, z: 0.0 },
                orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
            },
            runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
        },
        hand,
        equipped_item_resref: weapon.resref.clone(),
    };
    let fixtures = vec![make_fixture(
        "m2a_weapon_right_v2",
        RIGHT_CREATURE_RESREF,
        "RIGHT HAND - native stock sword - rhand",
        BinaryCreatureHandSlotV1::RightHand,
        10.0,
    )];
    let module =
        build_binary_creature_stock_weapon_demo_module_v2(&module_identity, &fixtures, &weapon)
            .map_err(|error| format!("CREATURE-WEAPON-DEMO-MODULE: {error}"))?;

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "CREATURE-WEAPON-DEMO-OUTPUT-CREATE {}: {error}",
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
        (format!("{MODULE_RESREF}.mod"), module.payload.as_slice()),
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
    let mut generated = Vec::with_capacity(files.len());
    for (name, bytes) in files {
        write_new(&output_directory.join(&name), bytes)?;
        generated.push(binding(name, bytes));
    }

    let handoff = serde_json::json!({
        "schemaVersion": 2,
        "status": "ready_for_owner_proof",
        "purpose": "CREATURE_NATIVE_HAND_ATTACHMENT_DEMO_V2",
        "testModuleFileName": format!("{MODULE_RESREF}.mod"),
        "moduleDisplayName": MODULE_DISPLAY_NAME,
        "areaDisplayName": AREA_DISPLAY_NAME,
        "moduleResref": MODULE_RESREF,
        "areaResref": AREA_RESREF,
        "hakResref": HAK_RESREF,
        "modelResref": MODEL_RESREF,
        "textureResref": TEXTURE_RESREF,
        "appearanceRow": product.report.appearance.appended_row_index,
        "source": binding(source_path.display().to_string(), &source),
        "inputAppearanceTwoDa": binding(appearance_path.display().to_string(), &appearance),
        "triangleCount": product.report.geometry.triangle_count,
        "animationCount": model_readback.animations.len(),
        "animationAnchorCoverage": animation_anchor_coverage,
        "appearanceModelType": model_type,
        "orientationPolicy": "CANONICAL_SOURCE_POSITIVE_Z_NO_FACING_MATRIX_VARIANT",
        "anchors": [
            { "name": AURORA_RIGHT_HAND_ANCHOR_V1, "parent": right_parent },
            { "name": AURORA_LEFT_HAND_ANCHOR_V1, "parent": left_parent }
        ],
        "equipment": module.readback.fixtures,
        "weapon": module.readback.weapon,
        "moduleReadback": module.readback,
        "generatedFiles": generated,
        "ownerProof": {
            "toolset": { "modelVisibility": "not_tested", "proofCompleteness": "missing" },
            "nwn": { "modelVisibility": "not_tested", "proofCompleteness": "missing" }
        },
        "startsToolset": false,
        "startsNwn": false
    });
    let handoff_bytes = serde_json::to_vec_pretty(&handoff)
        .map_err(|error| format!("CREATURE-WEAPON-DEMO-HANDOFF-JSON: {error}"))?;
    write_new(&output_directory.join("handoff.json"), &handoff_bytes)?;
    serde_json::to_string_pretty(&handoff)
        .map_err(|error| format!("CREATURE-WEAPON-DEMO-STDOUT-JSON: {error}"))
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
            _ => return Err(format!("CREATURE-WEAPON-DEMO-ARGUMENT-UNKNOWN: {argument}")),
        };
        if target.is_some() {
            return Err(format!(
                "CREATURE-WEAPON-DEMO-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("CREATURE-WEAPON-DEMO-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok((
        PathBuf::from(source.ok_or("CREATURE-WEAPON-DEMO-SOURCE-MISSING")?),
        PathBuf::from(appearance.ok_or("CREATURE-WEAPON-DEMO-APPEARANCE-MISSING")?),
        PathBuf::from(output.ok_or("CREATURE-WEAPON-DEMO-OUTPUT-MISSING")?),
    ))
}

fn read(path: &Path, role: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| {
        format!(
            "CREATURE-WEAPON-DEMO-{role}-READ {}: {error}",
            path.display()
        )
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("CREATURE-WEAPON-DEMO-WRITE {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("CREATURE-WEAPON-DEMO-WRITE {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("CREATURE-WEAPON-DEMO-SYNC {}: {error}", path.display()))
}

fn binding(file_name: String, bytes: &[u8]) -> ByteBinding {
    ByteBinding {
        file_name,
        byte_length: bytes.len() as u64,
        sha256: Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    }
}
