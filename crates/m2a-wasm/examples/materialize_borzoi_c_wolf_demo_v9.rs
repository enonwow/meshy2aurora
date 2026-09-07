use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1,
        BinaryCreatureProfiledFixtureV2, BinaryCreatureRuntimeProfileV2, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_profile_matrix_module_named_v3,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda";
const SOURCE_BYTE_LENGTH: usize = 29_889_104;
const RETAIL_C_WOLF_SHA256: &str =
    "a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726";
const REJECTED_V8_MODEL_SHA256: &str =
    "a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f";
const REJECTED_V8_SURFACE_SHA256: &str =
    "ff75e44d8c903e68fdded68061c594013374cd4255b9b31b356a9c77e64ab15a";

const MODEL_RESREF: &str = "m2aborzcre9";
const TEXTURE_RESREF: &str = "m2aborztex9";
const MATERIAL_RESREF: &str = "m2aborztex9_m0";
const HAK_RESREF: &str = "m2aborzhak9";
const MODULE_RESREF: &str = "m2aborzmod9";
const AREA_RESREF: &str = "m2aborzarea9";
const CREATURE_RESREF: &str = "m2aborzutc9";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora Borzoi c_wolf Demo V9";
const AREA_DISPLAY_NAME: &str = "Meshy2Aurora Borzoi Test Area V9";

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
    let (source_path, key_path, output_directory, prepared_at_utc) =
        parse_args(env::args().skip(1))?;
    if output_directory.exists() {
        return Err(format!(
            "BORZOI-CWOLF-V9-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }

    let source = read(&source_path, "source GLB")?;
    require_exact(
        &source,
        SOURCE_BYTE_LENGTH,
        SOURCE_SHA256,
        "canonical source GLB",
    )?;
    let c_wolf = read_key_resource(&key_path, "c_wolf", 2002)?;
    require_exact(&c_wolf, 340_812, RETAIL_C_WOLF_SHA256, "retail c_wolf")?;
    let appearance = read_key_resource(&key_path, "appearance", 2017)?;

    let chain_json = serde_json::to_string(&vec![json!({
        "resref": "c_wolf",
        "supermodelResref": "NULL",
        "format": "BINARY",
        "sha256": RETAIL_C_WOLF_SHA256,
        "byteOffset": 0,
        "byteLength": c_wolf.len()
    })])
    .map_err(|error| format!("BORZOI-CWOLF-V9-CHAIN-JSON: {error}"))?;
    let identity_json = serde_json::to_string(&json!({
        "modelResref": MODEL_RESREF,
        "textureResref": TEXTURE_RESREF,
        "materialResref": MATERIAL_RESREF,
        "hakResref": HAK_RESREF,
        "appearanceLabel": "M2A_BORZOI_CWOLF_V9",
        "appearanceDonorResrefs": ["c_dog"],
        "rejectedBaseline": {
            "modelSha256": REJECTED_V8_MODEL_SHA256,
            "visibleSurfaceSemanticSha256": REJECTED_V8_SURFACE_SHA256
        },
        "semanticControllerNames": ["Wolf_tail", "Wolf_tailend"]
    }))
    .map_err(|error| format!("BORZOI-CWOLF-V9-IDENTITY-JSON: {error}"))?;

    let mut product = m2a_wasm::build_reference_supermodel_creature_product_v2_inner(
        "c_wolf",
        &source,
        &appearance,
        &c_wolf,
        &chain_json,
        &identity_json,
        "POSITIVE_Z",
    )?;
    let report_json = product.report_json();
    let report = serde_json::from_str::<Value>(&report_json)
        .map_err(|error| format!("BORZOI-CWOLF-V9-REPORT-JSON: {error}"))?;
    require_product_admission(&report)?;
    let appearance_row = report["appearance"]["appendedRowIndex"]
        .as_u64()
        .ok_or("BORZOI-CWOLF-V9-APPEARANCE-ROW-MISSING")?;
    let appearance_row = u16::try_from(appearance_row)
        .map_err(|_| "BORZOI-CWOLF-V9-APPEARANCE-ROW-OOB".to_owned())?;

    let manifest_json = product.manifest_json();
    let summary_json = product.summary_json();
    let model_readback_json = product.readback_json();
    let model_readback = serde_json::from_str::<Value>(&model_readback_json)
        .map_err(|error| format!("BORZOI-CWOLF-V9-MODEL-READBACK-JSON: {error}"))?;
    if model_readback["model"]["supermodelName"] != "c_wolf"
        || model_readback["animations"].as_array().map(Vec::len) != Some(0)
    {
        return Err(
            "BORZOI-CWOLF-V9-MODEL-READBACK: expected c_wolf and zero local clips".to_owned(),
        );
    }

    let hak = product.take_hak_bytes();
    let model = product.take_model_bytes();
    let texture = product.take_texture_bytes();
    let material = product.take_material_bytes();
    let appearance_two_da = product.take_appearance_two_da_bytes();
    let archive = ErfArchive::parse(&hak)
        .map_err(|error| format!("BORZOI-CWOLF-V9-HAK-READBACK: {error:?}"))?;
    for (resref, resource_type, expected) in [
        (MODEL_RESREF, 2002_u16, model.as_slice()),
        (TEXTURE_RESREF, 3_u16, texture.as_slice()),
        (MATERIAL_RESREF, 2072_u16, material.as_slice()),
        ("appearance", 2017_u16, appearance_two_da.as_slice()),
    ] {
        let actual = archive
            .find(resref, resource_type)
            .map_err(|error| format!("BORZOI-CWOLF-V9-HAK-RESOURCE: {error:?}"))?;
        if actual != expected {
            return Err(format!(
                "BORZOI-CWOLF-V9-HAK-RESOURCE-DIFF: {resref}:{resource_type}"
            ));
        }
    }

    let fixture = BinaryCreatureProfiledFixtureV2 {
        fixture: BinaryCreatureOwnedFixtureV1 {
            id: "owned_borzoi_c_wolf_v9".to_owned(),
            template_resref: CREATURE_RESREF.to_owned(),
            display_name: "Meshy Borzoi - complete c_wolf skeleton".to_owned(),
            appearance_row,
            position: M0RuntimePositionV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
        },
        runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
    };
    let module = build_binary_creature_profile_matrix_module_named_v3(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: MODULE_RESREF.to_owned(),
            area_resref: AREA_RESREF.to_owned(),
            hak_resref: HAK_RESREF.to_owned(),
        },
        std::slice::from_ref(&fixture),
        MODULE_DISPLAY_NAME,
        AREA_DISPLAY_NAME,
        "Owner-proof fixture generated by the generic reference-supermodel product pipeline. The borzoi inherits the complete retail c_wolf controller hierarchy without copying retail payloads.",
    )
    .map_err(|error| format!("BORZOI-CWOLF-V9-MODULE: {error}"))?;
    if module.readback.scene.module_resref != MODULE_RESREF
        || module.readback.scene.area_resref != AREA_RESREF
        || module.readback.scene.ordered_hak_resrefs != [HAK_RESREF.to_owned()]
        || module.readback.fixtures != [fixture]
    {
        return Err("BORZOI-CWOLF-V9-MODULE-READBACK-DIFF".to_owned());
    }

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "BORZOI-CWOLF-V9-OUTPUT-CREATE {}: {error}",
            output_directory.display()
        )
    })?;
    let outputs = [
        (format!("{MODULE_RESREF}.mod"), module.payload.as_slice()),
        (format!("{HAK_RESREF}.hak"), hak.as_slice()),
        (format!("{MODEL_RESREF}.mdl"), model.as_slice()),
        (format!("{TEXTURE_RESREF}.tga"), texture.as_slice()),
        (format!("{MATERIAL_RESREF}.mtr"), material.as_slice()),
        ("appearance.2da".to_owned(), appearance_two_da.as_slice()),
    ];
    for (name, payload) in &outputs {
        write_new(&output_directory.join(name), payload)?;
    }
    write_new(
        &output_directory.join("product-report.json"),
        report_json.as_bytes(),
    )?;
    write_new(
        &output_directory.join("package-manifest.json"),
        manifest_json.as_bytes(),
    )?;
    write_new(
        &output_directory.join("product-summary.json"),
        summary_json.as_bytes(),
    )?;
    write_new(
        &output_directory.join("generated-model-inspection.json"),
        model_readback_json.as_bytes(),
    )?;
    write_json_new(
        &output_directory.join("module-readback.json"),
        &module.readback,
    )?;

    let artifact = |role: &str, name: &str, bytes: &[u8]| {
        json!({
            "role": role,
            "path": output_directory.join(name),
            "byteLength": bytes.len(),
            "sha256": sha256(bytes)
        })
    };
    let ready = json!({
        "schemaVersion": 2,
        "status": "prepared_for_native_install",
        "preparedAtUtc": prepared_at_utc,
        "moduleFileName": format!("{MODULE_RESREF}.mod"),
        "moduleDisplayName": MODULE_DISPLAY_NAME,
        "areaDisplayName": AREA_DISPLAY_NAME,
        "areaResref": AREA_RESREF,
        "orderedHakResrefs": [HAK_RESREF],
        "appearanceRow": appearance_row,
        "creatureTemplateResref": CREATURE_RESREF,
        "modelResref": MODEL_RESREF,
        "fixture": {
            "position": [10.0, 14.5, 0.0],
            "orientation": [0.0, -1.0]
        },
        "canonicalArtifacts": [
            artifact("MOD", &format!("{MODULE_RESREF}.mod"), &module.payload),
            artifact("HAK", &format!("{HAK_RESREF}.hak"), &hak),
            artifact("MDL", &format!("{MODEL_RESREF}.mdl"), &model),
            artifact("DIFFUSE_TGA", &format!("{TEXTURE_RESREF}.tga"), &texture),
            artifact("MINIMAL_MTR", &format!("{MATERIAL_RESREF}.mtr"), &material)
        ],
        "offlineValidation": {
            "sourceSha256": SOURCE_SHA256,
            "referenceSha256": RETAIL_C_WOLF_SHA256,
            "modelSha256": sha256(&model),
            "motionQualityStatus": report["motionQuality"]["status"],
            "carrierNodeCount": report["rigAnalysis"]["carrierNodeCount"],
            "activeWeightedBoneCount": report["rigAnalysis"]["activeWeightedBoneCount"],
            "requiredClipCount": report["motionQuality"]["requiredClipCount"],
            "sampledClipCount": report["motionQuality"]["sampledClipCount"],
            "jointClipRequiredCount": report["motionQuality"]["jointClipRequiredCount"],
            "jointClipPassCount": report["motionQuality"]["jointClipPassCount"],
            "seamViolationCount": report["motionQuality"]["seamPairViolationCount"],
            "pawContactViolationCount": report["motionQuality"]["pawContactViolationCount"],
            "pawSideViolationCount": report["motionQuality"]["pawSideViolationCount"],
            "tailSemanticDeltaProven": report["semanticDelta"]["exportDeltaProven"],
            "modelCastShadowDisabled": true,
            "ownerRuntimeProof": "NOT_RUN_HUMAN_OWNED"
        }
    });
    write_json_new(&output_directory.join("handoff-preinstall.json"), &ready)?;

    Ok(format!(
        "BORZOI_C_WOLF_DEMO_V9_PREPARED={{\"module\":\"{MODULE_RESREF}.mod\",\"moduleSha256\":\"{}\",\"hak\":\"{HAK_RESREF}.hak\",\"hakSha256\":\"{}\",\"appearanceRow\":{appearance_row},\"modelSha256\":\"{}\"}}",
        sha256(&module.payload),
        sha256(&hak),
        sha256(&model)
    ))
}

fn require_product_admission(report: &Value) -> Result<(), String> {
    let required = [
        (
            "status",
            report["status"] == "REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED",
        ),
        ("motionQuality", report["motionQuality"]["status"] == "PASS"),
        (
            "exportAdmission",
            report["exportAdmission"]["status"] == "REFERENCE_SUPERMODEL_EXPORT_ADMITTED",
        ),
        (
            "jointClipCoverage",
            report["motionQuality"]["jointClipRequiredCount"]
                == report["motionQuality"]["jointClipPassCount"],
        ),
        (
            "tailSemanticDelta",
            report["semanticDelta"]["exportDeltaProven"] == true,
        ),
        (
            "seams",
            report["motionQuality"]["seamPairViolationCount"] == 0,
        ),
        (
            "pawContact",
            report["motionQuality"]["pawContactViolationCount"] == 0,
        ),
        (
            "pawSide",
            report["motionQuality"]["pawSideViolationCount"] == 0,
        ),
    ];
    let failed = required
        .into_iter()
        .filter_map(|(name, pass)| (!pass).then_some(name))
        .collect::<Vec<_>>();
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "BORZOI-CWOLF-V9-PRODUCT-ADMISSION: failed {}",
            failed.join(", ")
        ))
    }
}

fn parse_args(
    arguments: impl IntoIterator<Item = String>,
) -> Result<(PathBuf, PathBuf, PathBuf, String), String> {
    let mut source = None;
    let mut key = None;
    let mut output = None;
    let mut timestamp = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("BORZOI-CWOLF-V9-ARGUMENT-VALUE-MISSING: {argument}"))?;
        match argument.as_str() {
            "--source" => source = Some(PathBuf::from(value)),
            "--nwn-base-key" => key = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            "--timestamp-utc" => timestamp = Some(value),
            _ => return Err(format!("BORZOI-CWOLF-V9-ARGUMENT-UNKNOWN: {argument}")),
        }
    }
    Ok((
        source.ok_or("BORZOI-CWOLF-V9-SOURCE-MISSING")?,
        key.ok_or("BORZOI-CWOLF-V9-KEY-MISSING")?,
        output.ok_or("BORZOI-CWOLF-V9-OUTPUT-MISSING")?,
        timestamp.ok_or("BORZOI-CWOLF-V9-TIMESTAMP-MISSING")?,
    ))
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|error| format!("BORZOI-CWOLF-V9-READ {label} {}: {error}", path.display()))
}

fn require_exact(
    payload: &[u8],
    expected_length: usize,
    expected_sha256: &str,
    label: &str,
) -> Result<(), String> {
    let actual_sha256 = sha256(payload);
    if payload.len() != expected_length || actual_sha256 != expected_sha256 {
        return Err(format!(
            "BORZOI-CWOLF-V9-FINGERPRINT {label}: expected {expected_length}/{expected_sha256}, got {}/{}",
            payload.len(),
            actual_sha256
        ));
    }
    Ok(())
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn write_new(path: &Path, payload: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("BORZOI-CWOLF-V9-WRITE-NEW {}: {error}", path.display()))?;
    file.write_all(payload)
        .map_err(|error| format!("BORZOI-CWOLF-V9-WRITE {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("BORZOI-CWOLF-V9-SYNC {}: {error}", path.display()))
}

fn write_json_new(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("BORZOI-CWOLF-V9-SERIALIZE {}: {error}", path.display()))?;
    bytes.push(b'\n');
    write_new(path, &bytes)
}

fn read_key_resource(
    key_path: &Path,
    wanted_resref: &str,
    wanted_type: u16,
) -> Result<Vec<u8>, String> {
    const KEY_HEADER_SIZE: usize = 64;
    const KEY_ENTRY_SIZE: usize = 22;
    let key = fs::read(key_path).map_err(|error| format!("BORZOI-CWOLF-V9-KEY-READ: {error}"))?;
    if key.len() < KEY_HEADER_SIZE || &key[0..4] != b"KEY " || &key[4..8] != b"V1  " {
        return Err("BORZOI-CWOLF-V9-KEY-HEADER: expected KEY V1".to_owned());
    }
    let bif_count = u32_at(&key, 8)? as usize;
    let key_count = u32_at(&key, 12)? as usize;
    let bif_table_offset = u32_at(&key, 16)? as usize;
    let key_table_offset = u32_at(&key, 20)? as usize;
    checked_table(&key, bif_table_offset, bif_count, 12, "BIF table")?;
    checked_table(
        &key,
        key_table_offset,
        key_count,
        KEY_ENTRY_SIZE,
        "KEY table",
    )?;

    let mut selected = None;
    for index in 0..key_count {
        let offset = key_table_offset + index * KEY_ENTRY_SIZE;
        let resref = logical_resref(&key[offset..offset + 16]);
        let resource_type = u16::from_le_bytes([key[offset + 16], key[offset + 17]]);
        if resref.eq_ignore_ascii_case(wanted_resref) && resource_type == wanted_type {
            if selected.is_some() {
                return Err(format!(
                    "BORZOI-CWOLF-V9-KEY-DUPLICATE: {wanted_resref}:{wanted_type}"
                ));
            }
            selected = Some(u32_at(&key, offset + 18)?);
        }
    }
    let resource_id = selected
        .ok_or_else(|| format!("BORZOI-CWOLF-V9-KEY-NOT-FOUND: {wanted_resref}:{wanted_type}"))?;
    let bif_index = (resource_id >> 20) as usize;
    let resource_index = resource_id & 0x000f_ffff;
    if bif_index >= bif_count {
        return Err("BORZOI-CWOLF-V9-KEY-BIF-INDEX-OOB".to_owned());
    }
    let bif_entry = bif_table_offset + bif_index * 12;
    let file_name_offset = u32_at(&key, bif_entry + 4)? as usize;
    let file_name_size = u16::from_le_bytes([key[bif_entry + 8], key[bif_entry + 9]]) as usize;
    let file_name_end = file_name_offset
        .checked_add(file_name_size)
        .filter(|end| *end <= key.len())
        .ok_or("BORZOI-CWOLF-V9-KEY-BIF-NAME-OOB")?;
    let logical_bif_name = String::from_utf8_lossy(&key[file_name_offset..file_name_end])
        .trim_end_matches('\0')
        .replace('\\', "/");
    let native_relative = logical_bif_name.replace('/', std::path::MAIN_SEPARATOR_STR);
    let key_parent = key_path.parent().ok_or("BORZOI-CWOLF-V9-KEY-PARENT")?;
    let installation_root = key_parent.parent().unwrap_or(key_parent);
    let candidates = [
        installation_root.join(&native_relative),
        key_parent.join(&native_relative),
    ];
    let bif_path = candidates
        .iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("BORZOI-CWOLF-V9-BIF-NOT-FOUND: {logical_bif_name}"))?;
    read_bif_resource(bif_path, resource_index, wanted_type)
}

fn read_bif_resource(
    path: &Path,
    resource_index: u32,
    wanted_type: u16,
) -> Result<Vec<u8>, String> {
    let mut bif = File::open(path).map_err(|error| format!("BORZOI-CWOLF-V9-BIF-OPEN: {error}"))?;
    let mut header = [0_u8; 20];
    bif.read_exact(&mut header)
        .map_err(|error| format!("BORZOI-CWOLF-V9-BIF-HEADER-READ: {error}"))?;
    if &header[0..4] != b"BIFF" || &header[4..8] != b"V1  " {
        return Err("BORZOI-CWOLF-V9-BIF-HEADER: expected BIFF V1".to_owned());
    }
    let variable_count = u32::from_le_bytes(header[8..12].try_into().unwrap());
    let variable_table_offset = u32::from_le_bytes(header[16..20].try_into().unwrap());
    if resource_index >= variable_count {
        return Err("BORZOI-CWOLF-V9-BIF-RESOURCE-INDEX-OOB".to_owned());
    }
    bif.seek(SeekFrom::Start(
        u64::from(variable_table_offset) + u64::from(resource_index) * 16,
    ))
    .map_err(|error| format!("BORZOI-CWOLF-V9-BIF-SEEK: {error}"))?;
    let mut entry = [0_u8; 16];
    bif.read_exact(&mut entry)
        .map_err(|error| format!("BORZOI-CWOLF-V9-BIF-ENTRY-READ: {error}"))?;
    let payload_offset = u32::from_le_bytes(entry[4..8].try_into().unwrap());
    let payload_size = u32::from_le_bytes(entry[8..12].try_into().unwrap());
    let resource_type = u32::from_le_bytes(entry[12..16].try_into().unwrap());
    if resource_type != u32::from(wanted_type) {
        return Err(format!(
            "BORZOI-CWOLF-V9-BIF-TYPE-MISMATCH: expected {wanted_type}, got {resource_type}"
        ));
    }
    let mut payload = vec![0_u8; payload_size as usize];
    bif.seek(SeekFrom::Start(u64::from(payload_offset)))
        .map_err(|error| format!("BORZOI-CWOLF-V9-BIF-PAYLOAD-SEEK: {error}"))?;
    bif.read_exact(&mut payload)
        .map_err(|error| format!("BORZOI-CWOLF-V9-BIF-PAYLOAD-READ: {error}"))?;
    Ok(payload)
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or("BORZOI-CWOLF-V9-BINARY-U32-OOB")?;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn checked_table(
    bytes: &[u8],
    offset: usize,
    count: usize,
    stride: usize,
    label: &str,
) -> Result<(), String> {
    let end = count
        .checked_mul(stride)
        .and_then(|length| offset.checked_add(length))
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| format!("BORZOI-CWOLF-V9-{label}-OOB"))?;
    let _ = end;
    Ok(())
}

fn logical_resref(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
