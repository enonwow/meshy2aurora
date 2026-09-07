use std::{env, fs, io::Write, path::Path};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLocStringV1, GffLocSubstringV1, GffStructV1,
        GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
    },
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1},
    item::{ItemAppearanceRecipeV1, validate_item_model_variants_in_baseitems_v1},
    item_package::{
        GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE, IFO_RESOURCE_TYPE, UTC_RESOURCE_TYPE,
        UTI_RESOURCE_TYPE,
    },
    item_uti::read_item_uti_v1,
    two_da::TwoDaLimitsV1,
};
use serde_json::json;
use sha2::{Digest, Sha256};

const SOURCE_MODULE_SHA256: &str =
    "8413f88aebd61507e8206096ffcdd6d3e571a0f66d766346f95f63111d45245a";
const MODEL_HAK_SHA256: &str = "d6f34a55f1d845aa86fe35cdec4df3cd7c0e9494daf26bb8b028307abfbd84ed";
const TABLE_HAK_SHA256: &str = "c4869ae516a8598f0db5e31b473abba9b15b3b3f04880eb02f4a3a37191a9e8f";
const BASEITEMS_SHA256: &str = "b2ba08aa55c185642c8973859b9d0dbf486bc8411a630a0a10a9f77e84d701c6";
const MODEL_SHA256: &str = "578c5fb940469ff31cf4aae9a50c02e497e7e92dca7c4d33328fb0c7e7f535be";
const MODULE_RESREF: &str = "m2assmpal221r2";
const MODULE_FILE_NAME: &str = "m2assmpal221r2.mod";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora Sandstorm Mask Palette 221 R2";
const AREA_DISPLAY_NAME: &str = "Sandstorm Mask Item Palette 221 R2";
const MODEL_HAK_RESREF: &str = "m2assmh221";
const TABLE_HAK_RESREF: &str = "lc_2da";
const MODEL_RESREF: &str = "helm_221";
const UTI_RESREF: &str = "m2assmaskpal221";
const TWO_DA_RESOURCE_TYPE: u16 = 2017;
const MDL_RESOURCE_TYPE: u16 = 2002;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::var("M2A_REPO_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(r"C:\Projects\meshy2aurora"));
    let source_root =
        root.join("proof-output/tlc-sandstorm-mask-helmet-palette-only-221-v1-20260904");
    let model_root = root.join("proof-output/tlc-sandstorm-mask-helmet-221-v1-20260904");
    let output_root = env::var("M2A_ITEM_PALETTE_R2_OUTPUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            root.join("proof-output/tlc-sandstorm-mask-helmet-palette-only-221-r2-20260904")
        });
    if output_root.exists() {
        return Err(format!("immutable output already exists: {}", output_root.display()).into());
    }

    let source_module =
        read_exact_hash(&source_root.join("m2assmpal221.mod"), SOURCE_MODULE_SHA256)?;
    let recipe_bytes = fs::read(model_root.join("recipe.json"))?;
    let recipe: ItemAppearanceRecipeV1 = serde_json::from_slice(&recipe_bytes)?;
    let model_hak_path = model_root.join("m2assmh221.hak");
    let model_hak = read_exact_hash(&model_hak_path, MODEL_HAK_SHA256)?;
    let model_archive = ErfArchive::parse(&model_hak)?;
    let model = model_archive.find(MODEL_RESREF, MDL_RESOURCE_TYPE)?;
    if sha256_hex(model) != MODEL_SHA256 {
        return Err("frozen model HAK no longer contains exact helm_221".into());
    }

    let table_hak_path =
        std::path::PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak\lc_2da.hak");
    let table_hak = read_exact_hash(&table_hak_path, TABLE_HAK_SHA256)?;
    let table_archive = ErfArchive::parse(&table_hak)?;
    let baseitems = table_archive.find("baseitems", TWO_DA_RESOURCE_TYPE)?;
    if sha256_hex(baseitems) != BASEITEMS_SHA256 {
        return Err("lc_2da baseitems.2da differs from the frozen source table".into());
    }
    let range = validate_item_model_variants_in_baseitems_v1(
        baseitems,
        &recipe,
        &TwoDaLimitsV1::default(),
    )?;
    if range.min_range != 0 || range.max_range < 221 {
        return Err("effective helmet model range does not expose variant 221".into());
    }

    let source_archive = ErfArchive::parse(&source_module)?;
    let mut resources = Vec::new();
    for descriptor in source_archive.resources() {
        let mut payload = source_archive
            .find(&descriptor.resref, descriptor.resource_type)?
            .to_vec();
        match descriptor.resource_type {
            IFO_RESOURCE_TYPE => {
                let mut document = read_expected_gff(&payload, GffFileTypeV1::Ifo)?;
                replace_string(&mut document.root, "Mod_Tag", MODULE_RESREF)?;
                replace_loc_string(&mut document.root, "Mod_Name", MODULE_DISPLAY_NAME)?;
                replace_loc_string(
                    &mut document.root,
                    "Mod_Description",
                    "Palette-only helmet module. The model HAK and effective baseitems.2da HAK are both attached so helm_221 is selectable.",
                )?;
                replace_hak_list(&mut document.root, &[MODEL_HAK_RESREF, TABLE_HAK_RESREF])?;
                payload = write_gff_v32(&document, &GffWriterOptionsV1::default())?.payload;
            }
            2012 => {
                let mut document = read_expected_gff(&payload, GffFileTypeV1::Are)?;
                replace_loc_string(&mut document.root, "Name", AREA_DISPLAY_NAME)?;
                payload = write_gff_v32(&document, &GffWriterOptionsV1::default())?.payload;
            }
            _ => {}
        }
        resources.push(HakResourceInputV1 {
            resref: descriptor.resref.clone(),
            resource_type: descriptor.resource_type,
            payload,
        });
    }
    let module = write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )?;
    validate_module(&module.payload)?;

    let manifest = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_palette_check",
        "correction": "attach_exact_effective_baseitems_hak_for_toolset_model_range",
        "sourceFailure": {
            "moduleFilename": "m2assmpal221.mod",
            "moduleSha256": SOURCE_MODULE_SHA256,
            "selectorAvailability": "absent",
            "observedMaximumExistingHelmet": 35,
            "modelVisibility": "not_tested",
            "proofCompleteness": "failed"
        },
        "moduleFilename": MODULE_FILE_NAME,
        "moduleResref": MODULE_RESREF,
        "moduleDisplayName": MODULE_DISPLAY_NAME,
        "areaName": AREA_DISPLAY_NAME,
        "orderedHakResrefs": [MODEL_HAK_RESREF, TABLE_HAK_RESREF],
        "modelHak": {
            "path": model_hak_path.to_string_lossy(),
            "sha256": MODEL_HAK_SHA256
        },
        "tableHak": {
            "path": table_hak_path.to_string_lossy(),
            "sha256": TABLE_HAK_SHA256,
            "baseitemsSha256": BASEITEMS_SHA256,
            "helmetMinRange": range.min_range,
            "helmetMaxRange": range.max_range
        },
        "modelResref": MODEL_RESREF,
        "modelVariant": 221,
        "modelSha256": MODEL_SHA256,
        "utiResref": UTI_RESREF,
        "moduleSha256": sha256_hex(&module.payload),
        "moduleResourceCount": resources.len(),
        "creatureResourceCount": 0,
        "placedCreatureCount": 0,
        "selectorRangeGate": "PASS",
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing"
    }))?;

    fs::create_dir_all(&output_root)?;
    write_new(&output_root.join(MODULE_FILE_NAME), &module.payload)?;
    write_new(&output_root.join("manifest.json"), &manifest)?;
    println!("{}", String::from_utf8(manifest)?);
    Ok(())
}

fn validate_module(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let module = ErfArchive::parse(bytes)?;
    if module.file_type() != ErfFileType::Module || module.resources().len() != 7 {
        return Err("corrected palette module type or exact resource count differs".into());
    }
    if module
        .resources()
        .iter()
        .any(|resource| resource.resource_type == UTC_RESOURCE_TYPE)
    {
        return Err("corrected palette module contains a forbidden creature blueprint".into());
    }
    let uti = read_item_uti_v1(
        module.find(UTI_RESREF, UTI_RESOURCE_TYPE)?,
        m2a_core::item::ItemCompositionProfileV1::ModelType1,
        &Default::default(),
    )?;
    if uti.parts.len() != 1 || uti.parts[0].variant != 221 {
        return Err("corrected module UTI does not select helmet variant 221".into());
    }
    for resource_type in [GIT_RESOURCE_TYPE, GIC_RESOURCE_TYPE] {
        let descriptor = module
            .resources()
            .iter()
            .find(|resource| resource.resource_type == resource_type)
            .ok_or("corrected palette module is missing GIT or GIC")?;
        let document = read_gff_v32(
            module.find(&descriptor.resref, resource_type)?,
            &Default::default(),
        )?;
        if !matches!(
            field_value(&document.root, "Creature List"),
            Some(GffValueV1::List(creatures)) if creatures.is_empty()
        ) {
            return Err("corrected palette module contains a placed creature".into());
        }
    }
    let ifo = read_gff_v32(
        module.find("module", IFO_RESOURCE_TYPE)?,
        &Default::default(),
    )?;
    let expected = [MODEL_HAK_RESREF, TABLE_HAK_RESREF];
    let Some(GffValueV1::List(haks)) = field_value(&ifo.root, "Mod_HakList") else {
        return Err("corrected module has no ordered HAK list".into());
    };
    if haks.len() != expected.len()
        || !haks.iter().zip(expected).all(|(entry, expected)| {
            entry.struct_id == 8
                && matches!(field_value(entry, "Mod_Hak"), Some(GffValueV1::String(value)) if value == expected.as_bytes())
        })
    {
        return Err("corrected module ordered HAK list differs after readback".into());
    }
    Ok(())
}

fn read_expected_gff(
    bytes: &[u8],
    expected: GffFileTypeV1,
) -> Result<GffDocumentV1, Box<dyn std::error::Error>> {
    let document = read_gff_v32(bytes, &Default::default())?;
    if document.file_type != expected {
        return Err("module fixture GFF type differs".into());
    }
    Ok(document)
}

fn field_value<'a>(structure: &'a GffStructV1, label: &str) -> Option<&'a GffValueV1> {
    structure
        .fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
}

fn field_value_mut<'a>(
    structure: &'a mut GffStructV1,
    label: &str,
) -> Result<&'a mut GffValueV1, Box<dyn std::error::Error>> {
    structure
        .fields
        .iter_mut()
        .find(|field| field.label == label)
        .map(|field| &mut field.value)
        .ok_or_else(|| format!("missing required GFF field {label}").into())
}

fn replace_string(
    structure: &mut GffStructV1,
    label: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = field_value_mut(structure, label)?;
    if !matches!(target, GffValueV1::String(_)) {
        return Err(format!("GFF field {label} is not a CExoString").into());
    }
    *target = GffValueV1::String(value.as_bytes().to_vec());
    Ok(())
}

fn replace_loc_string(
    structure: &mut GffStructV1,
    label: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = field_value_mut(structure, label)?;
    if !matches!(target, GffValueV1::LocString(_)) {
        return Err(format!("GFF field {label} is not a CExoLocString").into());
    }
    *target = GffValueV1::LocString(GffLocStringV1 {
        string_ref: u32::MAX,
        substrings: vec![GffLocSubstringV1 {
            string_id: 0,
            bytes: value.as_bytes().to_vec(),
        }],
    });
    Ok(())
}

fn replace_hak_list(
    structure: &mut GffStructV1,
    hak_resrefs: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let target = field_value_mut(structure, "Mod_HakList")?;
    if !matches!(target, GffValueV1::List(_)) {
        return Err("Mod_HakList is not a GFF List".into());
    }
    *target = GffValueV1::List(
        hak_resrefs
            .iter()
            .map(|hak| GffStructV1 {
                struct_id: 8,
                fields: vec![GffFieldV1 {
                    label: "Mod_Hak".to_owned(),
                    value: GffValueV1::String(hak.as_bytes().to_vec()),
                }],
            })
            .collect(),
    );
    Ok(())
}

fn read_exact_hash(path: &Path, expected: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "source hash mismatch for {}: expected {expected}, read {actual}",
            path.display()
        )
        .into());
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
