use m2a_core::key_bif::{
    KeyBifContextInputV1, KeyBifFileInputV1, inspect_key_bif_context_v1,
    locate_key_bif_resource_v1, resolve_key_bif_resource_sparse_v1, resolve_key_bif_resource_v1,
};

fn fixture() -> (Vec<u8>, Vec<u8>) {
    let payload = b"2DA V2.0\n";
    let mut bif = vec![0_u8; 20 + 16];
    bif[0..8].copy_from_slice(b"BIFFV1  ");
    bif[8..12].copy_from_slice(&1_u32.to_le_bytes());
    bif[12..16].copy_from_slice(&0_u32.to_le_bytes());
    bif[16..20].copy_from_slice(&20_u32.to_le_bytes());
    bif[20..24].copy_from_slice(&0_u32.to_le_bytes());
    bif[24..28].copy_from_slice(&36_u32.to_le_bytes());
    bif[28..32].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bif[32..36].copy_from_slice(&2017_u32.to_le_bytes());
    bif.extend_from_slice(payload);

    let bif_name = b"data/test.bif\0";
    let file_table_offset = 24_u32;
    let key_table_offset = file_table_offset + 12;
    let name_offset = key_table_offset + 22;
    let mut key = vec![0_u8; name_offset as usize];
    key[0..8].copy_from_slice(b"KEY V1  ");
    key[8..12].copy_from_slice(&1_u32.to_le_bytes());
    key[12..16].copy_from_slice(&1_u32.to_le_bytes());
    key[16..20].copy_from_slice(&file_table_offset.to_le_bytes());
    key[20..24].copy_from_slice(&key_table_offset.to_le_bytes());
    key[24..28].copy_from_slice(&(bif.len() as u32).to_le_bytes());
    key[28..32].copy_from_slice(&name_offset.to_le_bytes());
    key[32..34].copy_from_slice(&(bif_name.len() as u16).to_le_bytes());
    key[36..45].copy_from_slice(b"baseitems");
    key[52..54].copy_from_slice(&2017_u16.to_le_bytes());
    key[54..58].copy_from_slice(&0_u32.to_le_bytes());
    key.extend_from_slice(bif_name);
    (key, bif)
}

#[test]
fn resolves_exact_key_binding_to_bif_payload_without_filesystem_access() {
    let (key, bif) = fixture();
    let context = KeyBifContextInputV1 {
        schema_version: 1,
        key_file_name: "nwn_base.key".to_owned(),
        key_bytes: &key,
        bif_files: vec![KeyBifFileInputV1 {
            logical_name: "test.bif".to_owned(),
            bytes: &bif,
        }],
    };
    let report = inspect_key_bif_context_v1(&context).unwrap();
    assert_eq!(report.status, "PASSED");
    assert_eq!(report.key_resource_count, 1);
    assert_eq!(report.bif_files[0].logical_name, "data/test.bif");
    assert_eq!(report.context_sha256.len(), 64);

    let resource = resolve_key_bif_resource_v1(&context, "baseitems", 2017).unwrap();
    assert_eq!(resource.payload, b"2DA V2.0\n");
    assert_eq!(resource.bif_index, 0);
    assert_eq!(resource.resource_index, 0);
    assert_eq!(resource.context_sha256, report.context_sha256);
}

#[test]
fn missing_declared_bif_fails_closed() {
    let (key, _) = fixture();
    let context = KeyBifContextInputV1 {
        schema_version: 1,
        key_file_name: "nwn_base.key".to_owned(),
        key_bytes: &key,
        bif_files: Vec::new(),
    };
    let error = inspect_key_bif_context_v1(&context).unwrap_err();
    assert_eq!(error.code, "KEY-BIF-FILE-SET-INCOMPLETE");
}

#[test]
fn sparse_resolution_loads_only_the_key_selected_bif() {
    let (key, bif) = fixture();
    let locator =
        locate_key_bif_resource_v1("nwn_base.key", &key, "baseitems", 2017).expect("KEY locator");
    assert_eq!(locator.bif_logical_name, "data/test.bif");
    assert_eq!(locator.bif_index, 0);
    assert_eq!(locator.resource_index, 0);

    let supplied = KeyBifFileInputV1 {
        logical_name: "test.bif".to_owned(),
        bytes: &bif,
    };
    let resource =
        resolve_key_bif_resource_sparse_v1("nwn_base.key", &key, &supplied, "baseitems", 2017)
            .expect("sparse resource");
    assert_eq!(resource.payload, b"2DA V2.0\n");
    assert_eq!(resource.payload_sha256.len(), 64);
    assert_eq!(resource.context_sha256.len(), 64);
}

#[test]
fn sparse_resolution_rejects_a_bif_not_selected_by_the_key() {
    let (key, bif) = fixture();
    let supplied = KeyBifFileInputV1 {
        logical_name: "wrong.bif".to_owned(),
        bytes: &bif,
    };
    let error =
        resolve_key_bif_resource_sparse_v1("nwn_base.key", &key, &supplied, "baseitems", 2017)
            .unwrap_err();
    assert_eq!(error.code, "KEY-BIF-FILE-MISMATCH");
}
