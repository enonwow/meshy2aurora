use m2a_core::supermodel_catalog::{
    MdlCatalogFormatV1, SupermodelCatalogBuildInputV1, SupermodelCatalogCompletenessV1,
    SupermodelCatalogEntryStatusV1, SupermodelModelV1, SupermodelSourceKindV1,
    build_supermodel_catalog_v1, index_hak_models_v1, index_nwn_bif_table_v1,
    index_nwn_key_models_v1, inspect_mdl_catalog_header_v1, plan_nwn_bif_index_v1,
};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn fixed(bytes: &mut [u8], offset: usize, width: usize, value: &str) {
    bytes[offset..offset + value.len()].copy_from_slice(value.as_bytes());
    bytes[offset + value.len()..offset + width].fill(0);
}

fn synthetic_key() -> Vec<u8> {
    let bif_table_offset = 64usize;
    let bif_name_offset = bif_table_offset + 12;
    let bif_name = b"data\\models_01.bif\0";
    let key_table_offset = bif_name_offset + bif_name.len();
    let mut bytes = vec![0u8; key_table_offset + 3 * 22];
    bytes[0..8].copy_from_slice(b"KEY V1  ");
    write_u32(&mut bytes, 8, 1);
    write_u32(&mut bytes, 12, 3);
    write_u32(&mut bytes, 16, bif_table_offset as u32);
    write_u32(&mut bytes, 20, key_table_offset as u32);
    write_u32(&mut bytes, bif_table_offset + 4, bif_name_offset as u32);
    bytes[bif_table_offset + 8..bif_table_offset + 10]
        .copy_from_slice(&(bif_name.len() as u16).to_le_bytes());
    bytes[bif_name_offset..bif_name_offset + bif_name.len()].copy_from_slice(bif_name);

    for (index, (resref, resource_type)) in [
        ("c_child", 2002u16),
        ("ignored", 2017u16),
        ("c_parent", 2002u16),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = key_table_offset + index * 22;
        fixed(&mut bytes, offset, 16, resref);
        bytes[offset + 16..offset + 18].copy_from_slice(&resource_type.to_le_bytes());
        write_u32(&mut bytes, offset + 18, index as u32);
    }
    bytes
}

fn synthetic_bif_header(count: u32, table_offset: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 20];
    bytes[0..8].copy_from_slice(b"BIFFV1  ");
    write_u32(&mut bytes, 8, count);
    write_u32(&mut bytes, 16, table_offset);
    bytes
}

fn synthetic_binary_mdl_header(name: &str, supermodel: &str, animations: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 12 + 0xe8];
    write_u32(&mut bytes, 0, 0);
    write_u32(&mut bytes, 4, 0xe8);
    write_u32(&mut bytes, 8, 0);
    fixed(&mut bytes, 12 + 0x08, 64, name);
    write_u32(&mut bytes, 12 + 0x48, 0xe8);
    write_u32(&mut bytes, 12 + 0x4c, 1);
    write_u32(&mut bytes, 12 + 0x6c, 2);
    bytes[12 + 0x72] = 4;
    write_u32(&mut bytes, 12 + 0x78 + 4, animations);
    write_u32(&mut bytes, 12 + 0x78 + 8, animations);
    bytes[12 + 0xa4..12 + 0xa8].copy_from_slice(&0.75f32.to_le_bytes());
    fixed(&mut bytes, 12 + 0xa8, 64, supermodel);
    bytes
}

fn synthetic_hak_with_model(resref: &str, mdl: &[u8]) -> Vec<u8> {
    let key_offset = 160usize;
    let resource_offset = key_offset + 24;
    let payload_offset = resource_offset + 8;
    let mut bytes = vec![0u8; payload_offset + mdl.len()];
    bytes[0..8].copy_from_slice(b"HAK V1.0");
    write_u32(&mut bytes, 16, 1);
    write_u32(&mut bytes, 24, key_offset as u32);
    write_u32(&mut bytes, 28, resource_offset as u32);
    fixed(&mut bytes, key_offset, 16, resref);
    write_u32(&mut bytes, key_offset + 16, 0);
    bytes[key_offset + 20..key_offset + 22].copy_from_slice(&2002u16.to_le_bytes());
    write_u32(&mut bytes, resource_offset, payload_offset as u32);
    write_u32(&mut bytes, resource_offset + 4, mdl.len() as u32);
    bytes[payload_offset..].copy_from_slice(mdl);
    bytes
}

#[test]
fn key_index_preserves_all_mdl_locators_and_bif_names() {
    let report = index_nwn_key_models_v1(&synthetic_key()).expect("valid KEY index");
    assert_eq!(report.resource_count, 3);
    assert_eq!(report.model_resource_count, 2);
    assert_eq!(report.bifs[0].logical_name, "data/models_01.bif");
    assert_eq!(report.models[0].resref, "c_child");
    assert_eq!(report.models[0].bif_index, 0);
    assert_eq!(report.models[0].resource_index, 0);
    assert_eq!(report.models[1].resref, "c_parent");
    assert_eq!(report.models[1].resource_index, 2);
}

#[test]
fn bif_index_is_two_phase_and_validates_the_exact_variable_table() {
    let header = synthetic_bif_header(2, 32);
    let plan = plan_nwn_bif_index_v1(&header).expect("valid BIF plan");
    assert_eq!(plan.table_offset, 32);
    assert_eq!(plan.table_byte_length, 32);

    let mut table = vec![0u8; 32];
    write_u32(&mut table, 0, 0);
    write_u32(&mut table, 4, 100);
    write_u32(&mut table, 8, 244);
    write_u32(&mut table, 12, 2002);
    write_u32(&mut table, 16, 1);
    write_u32(&mut table, 20, 400);
    write_u32(&mut table, 24, 64);
    write_u32(&mut table, 28, 2017);
    let index = index_nwn_bif_table_v1(&header, &table).expect("valid BIF table");
    assert_eq!(index.resources[0].payload_offset, 100);
    assert_eq!(index.resources[0].payload_size, 244);
    assert_eq!(index.resources[0].resource_type, 2002);
    assert!(index_nwn_bif_table_v1(&header, &table[..31]).is_err());
}

#[test]
fn catalog_header_supports_binary_and_ascii_mdl_metadata() {
    let binary = synthetic_binary_mdl_header("c_child", "c_parent", 42);
    let report =
        inspect_mdl_catalog_header_v1(&binary, binary.len()).expect("binary catalog header");
    assert_eq!(report.format, MdlCatalogFormatV1::Binary);
    assert_eq!(report.model_name, "c_child");
    assert_eq!(report.supermodel_name, "c_parent");
    assert_eq!(report.local_animation_count, 42);
    assert_eq!(report.classification, Some(4));
    assert_eq!(report.animation_scale, 0.75);

    let ascii = b"newmodel a_child\nsetsupermodel a_child a_parent\nsetanimationscale 0.5\nnewanim walk a_child\ndoneanim walk a_child\n";
    let report = inspect_mdl_catalog_header_v1(ascii, ascii.len()).expect("ASCII catalog header");
    assert_eq!(report.format, MdlCatalogFormatV1::Ascii);
    assert_eq!(report.model_name, "a_child");
    assert_eq!(report.supermodel_name, "a_parent");
    assert_eq!(report.local_animation_count, 1);
    assert_eq!(report.animation_scale, 0.5);
}

fn model(resref: &str, supermodel: &str, priority: u32) -> SupermodelModelV1 {
    SupermodelModelV1 {
        resref: resref.to_owned(),
        source_id: "base".to_owned(),
        source_kind: SupermodelSourceKindV1::BaseKeyBif,
        container_name: "data/models_01.bif".to_owned(),
        source_priority: priority,
        resource_index: priority,
        payload_offset: priority * 100,
        payload_size: 244,
        header: inspect_mdl_catalog_header_v1(
            &synthetic_binary_mdl_header(
                resref,
                supermodel,
                if resref == "c_parent" { 42 } else { 0 },
            ),
            244,
        )
        .expect("fixture header"),
    }
}

#[test]
fn graph_discovers_supermodels_case_insensitively_and_reports_completeness() {
    let report = build_supermodel_catalog_v1(&SupermodelCatalogBuildInputV1 {
        declared_model_count: 4,
        failed_model_count: 0,
        models: vec![
            model("c_child", "C_PARENT", 0),
            model("c_parent", "NULL", 1),
            model("cycle_a", "cycle_b", 2),
            model("cycle_b", "cycle_a", 3),
        ],
    });
    assert_eq!(
        report.completeness,
        SupermodelCatalogCompletenessV1::Complete
    );
    assert_eq!(report.entries.len(), 3);
    let parent = report
        .entries
        .iter()
        .find(|entry| entry.resref == "c_parent")
        .unwrap();
    assert_eq!(parent.status, SupermodelCatalogEntryStatusV1::Resolved);
    assert_eq!(parent.children, vec!["c_child"]);
    assert_eq!(
        parent
            .resource
            .as_ref()
            .unwrap()
            .header
            .local_animation_count,
        42
    );
    let cycle = report
        .entries
        .iter()
        .find(|entry| entry.resref == "cycle_a")
        .unwrap();
    assert_eq!(cycle.status, SupermodelCatalogEntryStatusV1::Cyclic);
}

#[test]
fn unresolved_reference_and_failed_scan_are_never_reported_as_complete() {
    let report = build_supermodel_catalog_v1(&SupermodelCatalogBuildInputV1 {
        declared_model_count: 2,
        failed_model_count: 1,
        models: vec![model("c_child", "missing_parent", 0)],
    });
    assert_eq!(
        report.completeness,
        SupermodelCatalogCompletenessV1::Partial
    );
    assert_eq!(
        report.entries[0].status,
        SupermodelCatalogEntryStatusV1::Missing
    );
    assert!(report.entries[0].resource.is_none());
}

#[test]
fn hak_index_catalogues_mdl_payloads_without_extracting_them() {
    let mdl = synthetic_binary_mdl_header("c_hak_child", "c_horror", 0);
    let hak = synthetic_hak_with_model("c_hak_child", &mdl);
    let report = index_hak_models_v1(&hak).expect("valid HAK model inventory");
    assert_eq!(report.resource_count, 1);
    assert_eq!(report.model_resource_count, 1);
    assert_eq!(report.failed_model_count, 0);
    assert_eq!(report.models[0].resref, "c_hak_child");
    assert_eq!(report.models[0].payload_size, mdl.len());
    assert_eq!(
        report.models[0].header.as_ref().unwrap().supermodel_name,
        "c_horror"
    );
}

fn retail_bif_path(key_path: &Path, logical_name: &str) -> Option<PathBuf> {
    let key_parent = key_path.parent()?;
    let installation_root = key_parent.parent().unwrap_or(key_parent);
    let native = logical_name.replace('/', std::path::MAIN_SEPARATOR_STR);
    [installation_root.join(&native), key_parent.join(&native)]
        .into_iter()
        .find(|candidate| candidate.is_file())
}

#[test]
fn env_gated_retail_catalog_scans_every_declared_mdl_header() {
    let Ok(key_path) = std::env::var("M2A_REFERENCE_NWN_KEY") else {
        return;
    };
    let key_path = PathBuf::from(key_path);
    let key = std::fs::read(&key_path).expect("read M2A_REFERENCE_NWN_KEY");
    let index = index_nwn_key_models_v1(&key).expect("index retail KEY");
    let mut by_bif = std::collections::BTreeMap::<u32, Vec<_>>::new();
    for model in &index.models {
        by_bif.entry(model.bif_index).or_default().push(model);
    }
    let mut scanned = 0usize;
    let mut failures = Vec::new();
    let mut catalog_models = Vec::with_capacity(index.model_resource_count);
    for (bif_index, models) in by_bif {
        let bif = &index.bifs[bif_index as usize];
        let path = retail_bif_path(&key_path, &bif.logical_name)
            .unwrap_or_else(|| panic!("missing retail BIF {}", bif.logical_name));
        let mut file = std::fs::File::open(&path).expect("open retail BIF");
        let mut header = [0u8; 20];
        file.read_exact(&mut header).expect("read BIF header");
        let plan = plan_nwn_bif_index_v1(&header).expect("plan BIF table");
        let mut table = vec![0u8; plan.table_byte_length];
        file.seek(SeekFrom::Start(plan.table_offset as u64))
            .expect("seek BIF table");
        file.read_exact(&mut table).expect("read BIF table");
        let resources = index_nwn_bif_table_v1(&header, &table)
            .expect("index BIF table")
            .resources;
        for model in models {
            let resource = &resources[model.resource_index as usize];
            let prefix_length = (resource.payload_size as usize)
                .min(m2a_core::supermodel_catalog::BINARY_MDL_CATALOG_PREFIX_BYTES);
            let mut payload = vec![0u8; prefix_length];
            file.seek(SeekFrom::Start(resource.payload_offset as u64))
                .expect("seek MDL payload");
            file.read_exact(&mut payload).expect("read MDL prefix");
            let mut result =
                inspect_mdl_catalog_header_v1(&payload, resource.payload_size as usize);
            if result
                .as_ref()
                .is_err_and(|error| error.code == "M2A-SUPERMODEL-ASCII-FULL-PAYLOAD-REQUIRED")
            {
                payload.resize(resource.payload_size as usize, 0);
                file.seek(SeekFrom::Start(resource.payload_offset as u64))
                    .expect("seek ASCII MDL payload");
                file.read_exact(&mut payload)
                    .expect("read ASCII MDL payload");
                result = inspect_mdl_catalog_header_v1(&payload, resource.payload_size as usize);
            }
            match result {
                Ok(catalog_header) => {
                    scanned += 1;
                    catalog_models.push(SupermodelModelV1 {
                        resref: model.resref.clone(),
                        source_id: "retail-base".to_owned(),
                        source_kind: SupermodelSourceKindV1::BaseKeyBif,
                        container_name: bif.logical_name.clone(),
                        source_priority: 1_000,
                        resource_index: model.resource_index,
                        payload_offset: resource.payload_offset,
                        payload_size: resource.payload_size,
                        header: catalog_header,
                    });
                }
                Err(error) => failures.push(format!("{}:{}", model.resref, error.code)),
            }
        }
    }
    assert_eq!(
        scanned,
        index.model_resource_count,
        "retail catalog left {} unread headers: {}",
        failures.len(),
        failures
            .iter()
            .take(24)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    let catalog = build_supermodel_catalog_v1(&SupermodelCatalogBuildInputV1 {
        declared_model_count: index.model_resource_count,
        failed_model_count: 0,
        models: catalog_models,
    });
    assert_eq!(
        catalog.completeness,
        SupermodelCatalogCompletenessV1::Complete
    );
    assert!(
        catalog
            .entries
            .iter()
            .any(|entry| entry.resref.eq_ignore_ascii_case("c_wolf"))
    );
    assert!(
        catalog
            .entries
            .iter()
            .any(|entry| entry.resref.eq_ignore_ascii_case("c_horror"))
    );
    eprintln!(
        "retail supermodel catalog: {} MDL -> {} discovered supermodels",
        catalog.scanned_model_count, catalog.supermodel_count
    );
    let mut binary_preview_pass = 0usize;
    let mut binary_preview_failed = 0usize;
    let mut ascii_preview_deferred = 0usize;
    let mut ascii_with_animations = 0usize;
    let mut failed_binary_with_animations = 0usize;
    for entry in &catalog.entries {
        let Some(resource) = &entry.resource else {
            continue;
        };
        if resource.header.format == MdlCatalogFormatV1::Ascii {
            ascii_preview_deferred += 1;
            if resource.header.local_animation_count > 0 {
                ascii_with_animations += 1;
            }
            continue;
        }
        let path = retail_bif_path(&key_path, &resource.container_name)
            .unwrap_or_else(|| panic!("missing retail BIF {}", resource.container_name));
        let mut file = std::fs::File::open(path).expect("open supermodel BIF");
        file.seek(SeekFrom::Start(resource.payload_offset as u64))
            .expect("seek supermodel MDL");
        let mut payload = vec![0u8; resource.payload_size as usize];
        file.read_exact(&mut payload).expect("read supermodel MDL");
        match m2a_core::inspect_binary_mdl(&payload) {
            Ok(_) => binary_preview_pass += 1,
            Err(error) => {
                binary_preview_failed += 1;
                if resource.header.local_animation_count > 0 {
                    failed_binary_with_animations += 1;
                }
                assert!(
                    !entry.resref.eq_ignore_ascii_case("c_wolf")
                        && !entry.resref.eq_ignore_ascii_case("c_horror"),
                    "required reference supermodel {} failed full preview readback: {}",
                    entry.resref,
                    error
                );
            }
        }
    }
    eprintln!(
        "retail supermodel full preview: {binary_preview_pass} binary PASS, {binary_preview_failed} binary unsupported ({failed_binary_with_animations} with animations), {ascii_preview_deferred} ASCII metadata-only ({ascii_with_animations} with animations)"
    );
}
