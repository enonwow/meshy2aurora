use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    direct_creature_animation::has_preserved_source_combat_attacks_v1,
    inspect_binary_mdl,
    model_pipeline::{
        ProceduralCreatureBuildOptionsV1, ProceduralCreatureProductIdentityV2,
        build_meshy_procedural_humanoid_product_with_options_v3,
    },
    proof_module::{
        BinaryCreatureGhostlyVisageScriptV1, BinaryCreatureModuleIdentityV1,
        BinaryCreatureOwnedFixtureV1, BinaryCreatureProfiledFixtureV2,
        BinaryCreatureRuntimeProfileV2, GHOSTLY_VISAGE_NO_SOUND_SPAWN_SCRIPT_V1,
        M0RuntimeDirectionV1, M0RuntimePositionV1, VFX_DUR_GHOSTLY_VISAGE_NO_SOUND_V1,
        build_binary_creature_ghostly_visage_demo_module_named_v1,
        inspect_binary_creature_ghostly_visage_demo_module_v1,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af";
const APPEARANCE_SHA256: &str = "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const NSS_SHA256: &str = "80e11e6a90511c2aff65362296968f30c8f5b621ff96bfd393a1a7714dda4578";
const NCS_SHA256: &str = "5a99ae5ed86472c7bbd222a496368758849a65dccfc63c10c01b85f73fa29cca";
const EXPECTED_TRIANGLE_COUNT: usize = 297_190;
const EXPECTED_ANIMATION_COUNT: usize = 42;
const EXPECTED_SOURCE_CLIP_COUNT: u32 = 9;

const MODULE_RESREF: &str = "m2aghostdemo1";
const AREA_RESREF: &str = "m2aghostarea1";
const HAK_RESREF: &str = "m2aghosthak1";
const MODEL_RESREF: &str = "m2aghostcre1";
const TEXTURE_RESREF: &str = "m2aghosttex1";
const CREATURE_RESREF: &str = "m2aghostutc1";
const SCRIPT_RESREF: &str = "m2aghostsp1";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora Fogbound Ghostly Visage V1";
const AREA_DISPLAY_NAME: &str = "Meshy2Aurora Fogbound VFX Test V1";
const CREATURE_DISPLAY_NAME: &str = "Fogbound Claw Guard with Ghostly Visage";

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
    let (source_path, appearance_path, nss_path, ncs_path, output_directory) =
        parse_args(env::args().skip(1))?;
    if output_directory.exists() {
        return Err(format!(
            "CREATURE-GHOSTLY-VISAGE-V1-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }
    let source = read_exact(&source_path, SOURCE_SHA256, "SOURCE")?;
    let appearance = read_exact(&appearance_path, APPEARANCE_SHA256, "APPEARANCE")?;
    let nss = read_exact(&nss_path, NSS_SHA256, "NSS")?;
    let ncs = read_exact(&ncs_path, NCS_SHA256, "NCS")?;
    if nss != GHOSTLY_VISAGE_NO_SOUND_SPAWN_SCRIPT_V1.as_bytes() {
        return Err("CREATURE-GHOSTLY-VISAGE-V1-NSS-SEMANTIC-DIFF".to_owned());
    }
    if ncs.len() <= 8 || !ncs.starts_with(b"NCS V1.0") {
        return Err("CREATURE-GHOSTLY-VISAGE-V1-NCS-HEADER-DIFF".to_owned());
    }

    let product = build_meshy_procedural_humanoid_product_with_options_v3(
        &source,
        &appearance,
        &ProceduralCreatureProductIdentityV2 {
            model_resref: MODEL_RESREF.to_owned(),
            texture_resref: TEXTURE_RESREF.to_owned(),
            hak_resref: HAK_RESREF.to_owned(),
            appearance_label: "M2A_FOGBOUND_GHOST_V1".to_owned(),
        },
        &ProceduralCreatureBuildOptionsV1::default(),
    )
    .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-PRODUCT: {error}"))?;
    if product.report.geometry.triangle_count != EXPECTED_TRIANGLE_COUNT {
        return Err(format!(
            "CREATURE-GHOSTLY-VISAGE-V1-TRIANGLE-DIFF: expected {EXPECTED_TRIANGLE_COUNT}, got {}",
            product.report.geometry.triangle_count
        ));
    }
    if product.report.conversion.policies.basis_status != "CREATURE_BASIS_V3_RESOLVED"
        || product.report.conversion.policies.asset_forward_mapping
            != "GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y"
        || product.report.conversion.policies.engine_facing_proof != "OWNER_PROOF_REQUIRED"
        || (product.report.conversion.transform.determinant - 1.0).abs() > 1.0e-6
    {
        return Err("CREATURE-GHOSTLY-VISAGE-V1-FACING-CONTRACT-DIFF".to_owned());
    }
    let completeness = &product.report.animation_completeness;
    if completeness.input_source_clip_count != EXPECTED_SOURCE_CLIP_COUNT
        || !has_preserved_source_combat_attacks_v1(&completeness.clips)
    {
        return Err("CREATURE-GHOSTLY-VISAGE-V1-SOURCE-COMBAT-ATTACKS-MISSING".to_owned());
    }
    let model_readback = inspect_binary_mdl(&product.model)
        .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-MDL-READBACK: {error:?}"))?;
    if model_readback.animations.len() != EXPECTED_ANIMATION_COUNT {
        return Err(format!(
            "CREATURE-GHOSTLY-VISAGE-V1-ANIMATION-DIFF: expected {EXPECTED_ANIMATION_COUNT}, got {}",
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
                "CREATURE-GHOSTLY-VISAGE-V1-ATTACK-READBACK-MISSING: {attack}"
            ));
        }
    }

    let appearance_row = product.report.appearance.appended_row_index;
    if appearance_row != 15_100 {
        return Err(format!(
            "CREATURE-GHOSTLY-VISAGE-V1-APPEARANCE-ROW-DIFF: expected 15100, got {appearance_row}"
        ));
    }
    let model_type = read_model_type(&product.appearance_two_da, appearance_row)?;
    if model_type != "L" {
        return Err(format!(
            "CREATURE-GHOSTLY-VISAGE-V1-MODELTYPE-DIFF: expected L, got {model_type}"
        ));
    }

    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: MODULE_RESREF.to_owned(),
        area_resref: AREA_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
    };
    let fixtures = [BinaryCreatureProfiledFixtureV2 {
        fixture: BinaryCreatureOwnedFixtureV1 {
            id: "m2a_fogbound_ghost_v1".to_owned(),
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
        runtime_profile: BinaryCreatureRuntimeProfileV2::PassiveMonsterBaseline,
    }];
    let script = BinaryCreatureGhostlyVisageScriptV1 {
        resref: SCRIPT_RESREF.to_owned(),
        visual_effect_id: VFX_DUR_GHOSTLY_VISAGE_NO_SOUND_V1,
        source: nss.clone(),
        compiled: ncs.clone(),
    };
    let demo = build_binary_creature_ghostly_visage_demo_module_named_v1(
        &module_identity,
        &fixtures,
        &script,
        MODULE_DISPLAY_NAME,
        AREA_DISPLAY_NAME,
        "Fogbound Claw Guard with permanent base-game Ghostly Visage No Sound.",
    )
    .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-MODULE: {error}"))?;
    let independent_readback =
        inspect_binary_creature_ghostly_visage_demo_module_v1(&demo.payload, &script)
            .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-MOD-READBACK: {error}"))?;
    if independent_readback != demo.readback
        || demo.readback.visual_effect_id != VFX_DUR_GHOSTLY_VISAGE_NO_SOUND_V1
        || demo.readback.script_resref != SCRIPT_RESREF
        || demo.readback.fixtures != fixtures
    {
        return Err("CREATURE-GHOSTLY-VISAGE-V1-MOD-SEMANTIC-DIFF".to_owned());
    }

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "CREATURE-GHOSTLY-VISAGE-V1-OUTPUT-CREATE {}: {error}",
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
        (format!("{SCRIPT_RESREF}.nss"), nss.as_slice()),
        (format!("{SCRIPT_RESREF}.ncs"), ncs.as_slice()),
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
        "appearanceRow": appearance_row,
        "appearanceModelType": model_type,
        "placement": { "x": 10.0, "y": 14.5, "z": 0.0, "facingX": 0.0, "facingY": -1.0 },
        "source": binding(source_path.display().to_string(), &source),
        "sourceRole": "CANONICAL_MESHY_MERGED_ANIMATED_FOGBOUND_CLAW_GUARD",
        "inputAppearanceTwoDa": binding(appearance_path.display().to_string(), &appearance),
        "triangleCount": product.report.geometry.triangle_count,
        "animationCount": model_readback.animations.len(),
        "inputSourceClipCount": completeness.input_source_clip_count,
        "sourceCombatGate": "PASS_PRESERVED_SOURCE",
        "creatureBasis": {
            "status": product.report.conversion.policies.basis_status,
            "mapping": product.report.conversion.policies.asset_forward_mapping,
            "determinant": product.report.conversion.transform.determinant,
            "engineFacingProof": product.report.conversion.policies.engine_facing_proof
        },
        "visualEffect": {
            "constant": "VFX_DUR_GHOSTLY_VISAGE_NO_SOUND",
            "visualeffects2daRow": VFX_DUR_GHOSTLY_VISAGE_NO_SOUND_V1,
            "progfx2daRow": 402,
            "program": "AlphaLightBlue",
            "duration": "DURATION_TYPE_PERMANENT",
            "target": "OBJECT_SELF",
            "scriptResref": SCRIPT_RESREF,
            "scriptEvent": "ScriptSpawn",
            "sound": null
        },
        "scriptCompiler": {
            "name": "nwnsc",
            "version": "1.1.5",
            "strictMode": true,
            "optimized": true,
            "nss": binding(nss_path.display().to_string(), &nss),
            "ncs": binding(ncs_path.display().to_string(), &ncs)
        },
        "moduleReadback": demo.readback,
        "generatedFiles": generated_files,
        "startsToolset": false,
        "startsNwn": false
    });
    let materialization_bytes = serde_json::to_vec_pretty(&materialization)
        .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-MATERIALIZATION-JSON: {error}"))?;
    write_new(
        &output_directory.join("materialization.json"),
        &materialization_bytes,
    )?;
    serde_json::to_string_pretty(&materialization)
        .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-STDOUT-JSON: {error}"))
}

fn read_model_type(bytes: &[u8], physical_row: u16) -> Result<String, String> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits)
        .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-APPEARANCE-INSPECT: {error}"))?;
    let row = read_two_da_row_v2(bytes, u32::from(physical_row), &limits)
        .map_err(|error| format!("CREATURE-GHOSTLY-VISAGE-V1-APPEARANCE-ROW: {error}"))?;
    let index = inspection
        .columns
        .iter()
        .position(|column| column.eq_ignore_ascii_case("MODELTYPE"))
        .ok_or_else(|| "CREATURE-GHOSTLY-VISAGE-V1-MODELTYPE-COLUMN-MISSING".to_owned())?;
    row.cells
        .get(index)
        .and_then(|cell| match cell {
            TwoDaCellValueV1::Text { value } => Some(value.clone()),
            TwoDaCellValueV1::Null => None,
        })
        .ok_or_else(|| "CREATURE-GHOSTLY-VISAGE-V1-MODELTYPE-MISSING".to_owned())
}

fn parse_args(
    arguments: impl IntoIterator<Item = String>,
) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf, PathBuf), String> {
    let mut source = None;
    let mut appearance = None;
    let mut nss = None;
    let mut ncs = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source,
            "--appearance-2da" => &mut appearance,
            "--spawn-nss" => &mut nss,
            "--spawn-ncs" => &mut ncs,
            "--out" => &mut output,
            _ => {
                return Err(format!(
                    "CREATURE-GHOSTLY-VISAGE-V1-ARGUMENT-UNKNOWN: {argument}"
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "CREATURE-GHOSTLY-VISAGE-V1-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!(
                "CREATURE-GHOSTLY-VISAGE-V1-ARGUMENT-MISSING: {argument}"
            ));
        }
    }
    Ok((
        PathBuf::from(source.ok_or("CREATURE-GHOSTLY-VISAGE-V1-SOURCE-MISSING")?),
        PathBuf::from(appearance.ok_or("CREATURE-GHOSTLY-VISAGE-V1-APPEARANCE-MISSING")?),
        PathBuf::from(nss.ok_or("CREATURE-GHOSTLY-VISAGE-V1-NSS-MISSING")?),
        PathBuf::from(ncs.ok_or("CREATURE-GHOSTLY-VISAGE-V1-NCS-MISSING")?),
        PathBuf::from(output.ok_or("CREATURE-GHOSTLY-VISAGE-V1-OUTPUT-MISSING")?),
    ))
}

fn read_exact(path: &Path, expected_sha256: &str, role: &str) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "CREATURE-GHOSTLY-VISAGE-V1-{role}-READ {}: {error}",
            path.display()
        )
    })?;
    let actual = sha256(&bytes);
    if actual != expected_sha256 {
        return Err(format!(
            "CREATURE-GHOSTLY-VISAGE-V1-{role}-HASH-DIFF: expected {expected_sha256}, got {actual}"
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
        .map_err(|error| {
            format!(
                "CREATURE-GHOSTLY-VISAGE-V1-WRITE {}: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        format!(
            "CREATURE-GHOSTLY-VISAGE-V1-WRITE {}: {error}",
            path.display()
        )
    })?;
    file.sync_all().map_err(|error| {
        format!(
            "CREATURE-GHOSTLY-VISAGE-V1-SYNC {}: {error}",
            path.display()
        )
    })
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
