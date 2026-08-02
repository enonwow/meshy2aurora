use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    item::{
        ItemBlueprintV1, ItemColorValuesV1, ItemPartBuildOptionsV1, ItemPartTextureEncodingV1,
        ItemPartTransformV1, ItemPartValueV1, ItemProofModuleIdentityV1, ItemProofPlacementV1,
        build_item_proof_module_v1, build_meshy_item_part_with_options_v1, item_payload_sha256_v1,
        resolve_item_baseitem_v1, resolve_item_part_resource_v1, write_item_uti_v1,
    },
};
use serde::Serialize;

const OUTPUT_ROOT: &str = r"C:\Projects\meshy2aurora\proof-output\item-composed-r01-20260730";
const BASEITEMS_PATH: &str = r"C:\Projects\New Folder\item-retail-extract\2da\baseitems.2da";
const BASEITEMS_SHA256: &str = "3fbdcd012b55f0b869c6324cc7d4ece44ed63a78f00e7889e7acefac28c8fdf4";
const NWN_USER_ROOT: &str = r"C:\Users\enonw\Documents\Neverwinter Nights";
const MODULE_FILE_NAME: &str = "m2ait_r01.mod";
const MODULE_RESREF: &str = "m2ait_r01";
const MODULE_NAME: &str = "Meshy2Aurora Item composed r01";
const AREA_RESREF: &str = "m2ait_a01";
const AREA_NAME: &str = "Meshy2Aurora Item Assembly Proof";
const HAK_FILE_NAME: &str = "m2ait_h01.hak";
const HAK_RESREF: &str = "m2ait_h01";
const BLUEPRINT_RESREF: &str = "m2ait_u01";
const BASE_ITEM: u32 = 1;

struct PartSpec {
    field: &'static str,
    variant: u8,
    model_resref: &'static str,
    texture_resref: &'static str,
    source_path: &'static str,
    source_sha256: &'static str,
    transform: ItemPartTransformV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceReport {
    field: String,
    path: String,
    byte_length: usize,
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
    let output_root = PathBuf::from(OUTPUT_ROOT);
    if output_root.exists() {
        return Err(format!(
            "ITEM-R01-OUTPUT-EXISTS: immutable candidate root already exists: {}",
            output_root.display()
        ));
    }
    let baseitems = read_exact(
        Path::new(BASEITEMS_PATH),
        BASEITEMS_SHA256,
        "ITEM-R01-BASEITEMS",
    )?;
    let base_item = resolve_item_baseitem_v1(&baseitems, BASE_ITEM)
        .map_err(|error| json_error("ITEM-R01-BASEITEM-RESOLVE", &error))?;
    if base_item.model_type != 2 || base_item.part_slots.len() != 3 {
        return Err(
            "ITEM-R01-BASEITEM-CONTRACT: BaseItem 1 must be ModelType 2 with three parts"
                .to_owned(),
        );
    }

    let specs = [
        PartSpec {
            field: "ModelPart1",
            variant: 251,
            model_resref: "wswls_b_251",
            texture_resref: "m2aitxb01",
            source_path: r"C:\Projects\meshy2aurora\sample-3d\s1-placeable-ritual-pedestal-1500\source.glb",
            source_sha256: "dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372",
            transform: transform(0.0, 0.0, 0.0, 1.0),
        },
        PartSpec {
            field: "ModelPart2",
            variant: 251,
            model_resref: "wswls_m_251",
            texture_resref: "m2aitxm01",
            source_path: r"C:\Projects\meshy2aurora\sample-3d\s1-static-prop-1500\source.glb",
            source_sha256: "e476bfa426701bb73f75b7555bd6576a7923064ecab0f7e7afae277f373ca191",
            transform: transform(0.0, 0.0, 2.0, 1.0),
        },
        PartSpec {
            field: "ModelPart3",
            variant: 251,
            model_resref: "wswls_t_251",
            texture_resref: "m2aitxt01",
            source_path: r"C:\Projects\meshy2aurora\sample-3d\tlc-aether-lamp-p20k-v1\final.glb",
            source_sha256: "310a6597f92bfcd391f44d293585417fa53abe3f1a576d93a7e0dc2c586b1ac7",
            transform: transform(0.0, 0.0, 4.0, 1.0),
        },
    ];
    let mut sources = Vec::new();
    let mut parts = Vec::new();
    let mut hak_resources = Vec::new();
    let mut generated_files = Vec::<(String, Vec<u8>)>::new();
    for spec in &specs {
        let resolved =
            resolve_item_part_resource_v1(&base_item, spec.field, spec.variant, None, None)
                .map_err(|error| json_error("ITEM-R01-RESOURCE-RESOLVE", &error))?;
        if resolved.model_resref != spec.model_resref {
            return Err(format!(
                "ITEM-R01-MODEL-IDENTITY: {} resolved to {}, expected {}",
                spec.field, resolved.model_resref, spec.model_resref
            ));
        }
        let source = read_exact(
            Path::new(spec.source_path),
            spec.source_sha256,
            "ITEM-R01-SOURCE",
        )?;
        sources.push(SourceReport {
            field: spec.field.to_owned(),
            path: spec.source_path.to_owned(),
            byte_length: source.len(),
            sha256: item_payload_sha256_v1(&source),
        });
        let artifact = build_meshy_item_part_with_options_v1(
            &source,
            spec.model_resref,
            spec.texture_resref,
            &ItemPartBuildOptionsV1 {
                schema_version: 1,
                transform: spec.transform,
                source_node: None,
                texture_encoding: ItemPartTextureEncodingV1::DirectColor,
                icon_size: Some([32, 96]),
            },
        )
        .map_err(|error| json_error("ITEM-R01-PART-BUILD", &error))?;
        let icon = artifact.icon_payload.ok_or_else(|| {
            format!(
                "ITEM-R01-ICON-MISSING: {} emitted no icon layer",
                spec.field
            )
        })?;
        hak_resources.extend([
            resource(spec.model_resref, 2002, artifact.mdl_payload.clone()),
            resource(spec.texture_resref, 3, artifact.texture_payload.clone()),
            resource(&resolved.icon_resref, 3, icon.clone()),
        ]);
        generated_files.extend([
            (format!("{}.mdl", spec.model_resref), artifact.mdl_payload),
            (
                format!("{}.tga", spec.texture_resref),
                artifact.texture_payload,
            ),
            (format!("{}.tga", resolved.icon_resref), icon),
        ]);
        parts.push((spec, artifact.report));
    }
    let triangle_count = parts
        .iter()
        .map(|(_, report)| report.triangle_count)
        .sum::<usize>();
    if triangle_count > 300_000 {
        return Err(format!(
            "ITEM-R01-TRIANGLE-BUDGET: {triangle_count} exceeds 300000"
        ));
    }

    let uti = write_item_uti_v1(
        &base_item,
        &ItemBlueprintV1 {
            schema_version: 1,
            template_resref: BLUEPRINT_RESREF.to_owned(),
            tag: "M2AIT_R01".to_owned(),
            localized_name: "Meshy composed three-part item r01".to_owned(),
            description: "Candidate-bound Meshy2Aurora three-part item.".to_owned(),
            identified_description: "Candidate-bound Meshy2Aurora three-part item.".to_owned(),
            comment: "Generated offline; final visual proof is owner-owned.".to_owned(),
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
    .map_err(|error| json_error("ITEM-R01-UTI-BUILD", &error))?;
    hak_resources.push(resource(BLUEPRINT_RESREF, 2025, uti.payload.clone()));
    let hak = write_hak_v1(&hak_resources, &HakWriterOptionsV1::default())
        .map_err(|error| format!("ITEM-R01-HAK-BUILD: {error}"))?;
    let hak_readback = ErfArchive::parse(&hak.payload)
        .map_err(|error| format!("ITEM-R01-HAK-READBACK: {error}"))?;
    if hak_readback.resources().len() != hak_resources.len() {
        return Err("ITEM-R01-HAK-SEMANTIC-DIFF: resource count differs".to_owned());
    }
    let proof_module = build_item_proof_module_v1(
        &uti.payload,
        &ItemProofModuleIdentityV1 {
            schema_version: 1,
            module_resref: MODULE_RESREF.to_owned(),
            area_resref: AREA_RESREF.to_owned(),
            hak_resref: HAK_RESREF.to_owned(),
            blueprint_resref: BLUEPRINT_RESREF.to_owned(),
            module_name: MODULE_NAME.to_owned(),
            area_name: AREA_NAME.to_owned(),
        },
        ItemProofPlacementV1::default(),
    )
    .map_err(|error| json_error("ITEM-R01-MODULE-BUILD", &error))?;

    fs::create_dir_all(&output_root).map_err(|error| format!("ITEM-R01-OUTPUT-CREATE: {error}"))?;
    generated_files.extend([
        (format!("{BLUEPRINT_RESREF}.uti"), uti.payload.clone()),
        (HAK_FILE_NAME.to_owned(), hak.payload.clone()),
        (MODULE_FILE_NAME.to_owned(), proof_module.payload.clone()),
    ]);
    for (name, payload) in &generated_files {
        write_new(&output_root.join(name), payload)?;
    }

    let native_module = Path::new(NWN_USER_ROOT)
        .join("modules")
        .join(MODULE_FILE_NAME);
    let native_hak = Path::new(NWN_USER_ROOT).join("hak").join(HAK_FILE_NAME);
    let module_install =
        install_exact_new_or_identical(&output_root.join(MODULE_FILE_NAME), &native_module)?;
    let hak_install =
        install_exact_new_or_identical(&output_root.join(HAK_FILE_NAME), &native_hak)?;

    let report = serde_json::json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "candidateId": "item-composed-r01-20260730",
        "testModuleFileName": MODULE_FILE_NAME,
        "toolsetModuleName": MODULE_NAME,
        "areaName": AREA_NAME,
        "areaResref": AREA_RESREF,
        "orderedHakFiles": [HAK_FILE_NAME],
        "orderedHakResrefs": [HAK_RESREF],
        "blueprintResref": BLUEPRINT_RESREF,
        "baseItem": BASE_ITEM,
        "modelType": base_item.model_type,
        "partFields": specs.iter().map(|spec| serde_json::json!({
            "field": spec.field,
            "variant": spec.variant,
            "modelResref": spec.model_resref,
            "textureResref": spec.texture_resref,
        })).collect::<Vec<_>>(),
        "sourceGlbs": sources,
        "baseitems2da": {
            "path": BASEITEMS_PATH,
            "sha256": BASEITEMS_SHA256,
        },
        "triangleCount": triangle_count,
        "partReports": parts.iter().map(|(_, report)| report).collect::<Vec<_>>(),
        "utiReport": uti.report,
        "hak": {
            "path": output_root.join(HAK_FILE_NAME),
            "byteLength": hak.payload.len(),
            "sha256": item_payload_sha256_v1(&hak.payload),
        },
        "module": {
            "path": output_root.join(MODULE_FILE_NAME),
            "byteLength": proof_module.payload.len(),
            "sha256": item_payload_sha256_v1(&proof_module.payload),
            "report": proof_module.report,
        },
        "nativeInstallation": {
            "module": module_install,
            "hak": hak_install,
            "byteIdentical": true,
        },
        "placement": ItemProofPlacementV1::default(),
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "ownerProofRequired": true,
        "agentStartedToolset": false,
        "agentStartedNwn": false,
    });
    let report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("ITEM-R01-REPORT-SERIALIZE: {error}"))?;
    write_new(
        &output_root.join("ready-for-owner-proof.json"),
        &report_bytes,
    )?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| format!("ITEM-R01-SUMMARY-SERIALIZE: {error}"))
}

fn transform(x: f32, y: f32, z: f32, uniform_scale: f32) -> ItemPartTransformV1 {
    ItemPartTransformV1 {
        translation: [x, y, z],
        rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
        uniform_scale,
        pivot: [0.0, 0.0, 0.0],
    }
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
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
        .map_err(|error| format!("ITEM-R01-WRITE-NEW: {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("ITEM-R01-WRITE: {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("ITEM-R01-SYNC: {}: {error}", path.display()))?;
    let readback = fs::read(path)
        .map_err(|error| format!("ITEM-R01-WRITE-READBACK: {}: {error}", path.display()))?;
    if readback != bytes {
        return Err(format!("ITEM-R01-WRITE-SEMANTIC-DIFF: {}", path.display()));
    }
    Ok(())
}

fn install_exact_new_or_identical(
    source: &Path,
    destination: &Path,
) -> Result<serde_json::Value, String> {
    let source_bytes = fs::read(source).map_err(|error| {
        format!(
            "ITEM-R01-INSTALL-SOURCE-READ: {}: {error}",
            source.display()
        )
    })?;
    let source_hash = item_payload_sha256_v1(&source_bytes);
    let disposition = if destination.exists() {
        let existing = fs::read(destination).map_err(|error| {
            format!(
                "ITEM-R01-INSTALL-DESTINATION-READ: {}: {error}",
                destination.display()
            )
        })?;
        if existing != source_bytes {
            return Err(format!(
                "ITEM-R01-INSTALL-COLLISION: {} exists with SHA-256 {}, expected {}; no overwrite performed",
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
            "ITEM-R01-INSTALL-VERIFY-READ: {}: {error}",
            destination.display()
        )
    })?;
    let installed_hash = item_payload_sha256_v1(&installed);
    if installed_hash != source_hash {
        return Err(format!(
            "ITEM-R01-INSTALL-HASH-MISMATCH: {} source {}, destination {}",
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
