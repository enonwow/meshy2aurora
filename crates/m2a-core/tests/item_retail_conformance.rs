use std::{
    env, fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use m2a_core::{
    gff::{
        GffFileTypeV1, GffLimitsV1, GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
    },
    item::{
        ARMOR_PART_FIELDS_V1, ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1, ItemPropertyV1,
        ItemResourceProvenanceV1, inspect_item_baseitems_v1, inspect_item_reference_resource_v1,
        resolve_item_capart_part_v1, resolve_item_cast_spell_icon_v1, resolve_item_cloak_v2,
    },
};
use sha2::{Digest, Sha256};

const RETAIL_BASEITEMS_ENV: &str = "M2A_RETAIL_BASEITEMS_2DA";
const RETAIL_UTI_ROOT_ENV: &str = "M2A_RETAIL_ITEM_UTI_ROOT";
const RETAIL_2DA_ROOT_ENV: &str = "M2A_RETAIL_ITEM_2DA_ROOT";
const RETAIL_BASEITEMS_SHA256: &str =
    "3fbdcd012b55f0b869c6324cc7d4ece44ed63a78f00e7889e7acefac28c8fdf4";
const RETAIL_KEY_ENV: &str = "M2A_REFERENCE_NWN_KEY";

fn field<'a>(document: &'a m2a_core::gff::GffDocumentV1, label: &str) -> Option<&'a GffValueV1> {
    document
        .root
        .fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_retail_resource(
    key_path: &Path,
    key: &[u8],
    expected_resref: &str,
    expected_type: u16,
) -> (Vec<u8>, u32, u32) {
    let key_count = read_u32(key, 12) as usize;
    let file_table_offset = read_u32(key, 16) as usize;
    let key_table_offset = read_u32(key, 20) as usize;
    let (_, resource_id) = (0..key_count)
        .find_map(|index| {
            let offset = key_table_offset + index * 22;
            let resref = std::str::from_utf8(&key[offset..offset + 16])
                .ok()?
                .trim_end_matches('\0');
            (resref == expected_resref && read_u16(key, offset + 16) == expected_type)
                .then(|| (index, read_u32(key, offset + 18)))
        })
        .unwrap_or_else(|| panic!("retail KEY has no {expected_type}:{expected_resref}"));
    let bif_index = resource_id >> 20;
    let resource_index = resource_id & 0x000f_ffff;
    let file_offset = file_table_offset + bif_index as usize * 12;
    let name_offset = read_u32(key, file_offset + 4) as usize;
    let name_size = read_u16(key, file_offset + 8) as usize;
    let bif_name = std::str::from_utf8(&key[name_offset..name_offset + name_size])
        .unwrap()
        .trim_end_matches('\0')
        .replace('\\', "/");
    let install_root = key_path
        .parent()
        .and_then(Path::parent)
        .expect("NWN install root");
    let mut bif = fs::File::open(install_root.join(bif_name)).expect("open retail BIF in place");
    let mut header = [0_u8; 20];
    bif.read_exact(&mut header).unwrap();
    assert_eq!(&header[..8], b"BIFFV1  ");
    let variable_count = read_u32(&header, 8);
    assert!(resource_index < variable_count);
    let variable_table_offset = read_u32(&header, 16) as u64;
    bif.seek(SeekFrom::Start(
        variable_table_offset + u64::from(resource_index) * 16,
    ))
    .unwrap();
    let mut variable = [0_u8; 16];
    bif.read_exact(&mut variable).unwrap();
    assert_eq!(read_u32(&variable, 12) as u16, expected_type);
    let mut payload = vec![0_u8; read_u32(&variable, 8) as usize];
    bif.seek(SeekFrom::Start(read_u32(&variable, 4) as u64))
        .unwrap();
    bif.read_exact(&mut payload).unwrap();
    (payload, bif_index, resource_index)
}

#[test]
#[ignore = "requires M2A_RETAIL_ITEM_2DA_ROOT pointing at the audited exact retail extraction"]
fn exact_retail_special_tables_resolve_spell_cloak_and_all_capart_slots() {
    let root = PathBuf::from(
        env::var(RETAIL_2DA_ROOT_ENV)
            .expect("set M2A_RETAIL_ITEM_2DA_ROOT and run this ignored retail test explicitly"),
    );
    let spells = fs::read(root.join("iprp_spells.2da")).expect("read retail iprp_spells.2da");
    let cloak = fs::read(root.join("cloakmodel.2da")).expect("read retail cloakmodel.2da");
    let capart = fs::read(root.join("capart.2da")).expect("read retail capart.2da");

    let property = ItemPropertyV1 {
        property_name: 15,
        subtype: 1,
        cost_table: 0,
        cost_value: 0,
        param1: 255,
        param1_value: 0,
        chance_appear: 100,
    };
    let spell = resolve_item_cast_spell_icon_v1(54, &[property], &spells).unwrap();
    assert_eq!(spell.cast_spell_subtype, 1);
    assert_eq!(spell.icon_resref, "iss_aid");

    let cloak = resolve_item_cloak_v2(
        1,
        &cloak,
        &["2002:pmh0_cloak_001", "6:cloak_001", "6:icloak_m_001"],
    )
    .unwrap();
    assert_eq!(cloak.model, 1);
    assert_eq!(cloak.icon, 1);
    assert_eq!(cloak.model_resref, "pmh0_cloak_001");
    assert_eq!(cloak.texture_resref, "cloak_001");
    assert_eq!(cloak.icon_resref, "icloak_m_001");
    assert_eq!(cloak.resource_verification, "RESOURCE_KEYS_VERIFIED");

    let part_tables = [
        "PARTS_FOOT",
        "PARTS_FOOT",
        "PARTS_SHIN",
        "PARTS_SHIN",
        "PARTS_LEGS",
        "PARTS_LEGS",
        "PARTS_PELVIS",
        "PARTS_CHEST",
        "PARTS_BELT",
        "PARTS_NECK",
        "PARTS_FOREARM",
        "PARTS_FOREARM",
        "PARTS_BICEP",
        "PARTS_BICEP",
        "PARTS_SHOULDER",
        "PARTS_SHOULDER",
        "PARTS_HAND",
        "PARTS_HAND",
        "PARTS_ROBE",
    ];
    for (&field, &table_name) in ARMOR_PART_FIELDS_V1.iter().zip(&part_tables) {
        let table = fs::read(root.join(format!("{}.2da", table_name.to_ascii_lowercase())))
            .unwrap_or_else(|error| panic!("read exact retail {table_name}.2da: {error}"));
        let resolved = resolve_item_capart_part_v1(field, 0, &capart, table_name, &table)
            .unwrap_or_else(|error| panic!("resolve exact retail {field}: {error:?}"));
        assert_eq!(resolved.field, field);
        assert_eq!(resolved.selector, 0);
        assert_eq!(resolved.parts_table, table_name);
        assert!(resolved.available_part_count > 0);
    }
}

#[test]
#[ignore = "requires the read-only local NWN retail KEY/BIF installation"]
fn exact_retail_cloak_and_capart_payloads_pass_own_format_and_provenance_validation() {
    let key_path = env::var_os(RETAIL_KEY_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                r"C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\nwn_base.key",
            )
        });
    let key = fs::read(&key_path).expect("read nwn_base.key in place");
    assert_eq!(&key[..8], b"KEY V1  ");
    assert_eq!(
        format!("{:x}", Sha256::digest(&key)),
        ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1
    );
    let manifest_sha256 = format!("{:x}", Sha256::digest(b"exact-retail-cloak-row-1"));
    let resources = [
        (2002, "pmh0_cloak_001"),
        (6, "cloak_001"),
        (6, "icloak_m_001"),
        (2002, "pmh0_footr001"),
        (6, "pmh0_footr001"),
    ];
    let inspected = resources.map(|(resource_type, resref)| {
        let (payload, bif_index, resource_index) =
            read_retail_resource(&key_path, &key, resref, resource_type);
        let provenance = ItemResourceProvenanceV1 {
            schema_version: 1,
            source_kind: "NWN_BASE_KEY".to_owned(),
            source_container_file_name: "nwn_base.key".to_owned(),
            source_container_sha256: ITEM_RETAIL_NWN_BASE_KEY_SHA256_V1.to_owned(),
            resource_locator: format!("bif:{bif_index}:resource:{resource_index}"),
            expected_sha256: format!("{:x}", Sha256::digest(&payload)),
            manifest_sha256: manifest_sha256.clone(),
        };
        inspect_item_reference_resource_v1(resource_type, resref, &payload, &provenance)
            .unwrap_or_else(|error| {
                panic!("inspect exact retail {resource_type}:{resref}: {error:?}")
            })
    });
    assert_eq!(
        inspected[0].format,
        m2a_core::item::ItemReferenceResourceFormatV1::AsciiMdl
    );
    assert!(inspected[0].triangle_count > 0);
    assert_eq!(inspected[1].triangle_count, 0);
    assert_eq!(inspected[2].triangle_count, 0);
    assert!(inspected[3].triangle_count > 0);
    assert_eq!(inspected[4].triangle_count, 0);
}

#[test]
#[ignore = "requires M2A_RETAIL_BASEITEMS_2DA pointing at the audited exact retail table"]
fn exact_retail_baseitems_resolves_active_rows_and_resref_fallbacks() {
    let path = env::var(RETAIL_BASEITEMS_ENV)
        .expect("set M2A_RETAIL_BASEITEMS_2DA and run this ignored retail test explicitly");
    let bytes = fs::read(path).expect("read exact retail baseitems.2da in place");
    let catalog = inspect_item_baseitems_v1(&bytes).expect("inspect retail baseitems.2da");

    assert_eq!(catalog.source_sha256, RETAIL_BASEITEMS_SHA256);
    assert_eq!(catalog.physical_row_count, 113);
    assert_eq!(catalog.inactive_row_count, 17);
    assert_eq!(catalog.rows.len(), 96);

    let longsword = catalog
        .rows
        .iter()
        .find(|row| row.base_item == 1)
        .expect("BaseItem 1");
    assert_eq!(longsword.model_type, 2);
    assert_eq!(longsword.item_class, "WSwLs");
    assert_eq!(longsword.default_model.as_deref(), Some("it_bag"));
    assert_eq!(longsword.default_icon.as_deref(), Some("iwswls"));

    let armor = catalog
        .rows
        .iter()
        .find(|row| row.base_item == 16)
        .expect("BaseItem 16");
    assert_eq!(armor.model_type, 3);
    assert_eq!(armor.default_model.as_deref(), Some("gifp"));
    assert_eq!(armor.default_icon.as_deref(), Some("iit_chest"));
}

#[test]
#[ignore = "requires M2A_RETAIL_ITEM_UTI_ROOT pointing at the audited exact retail corpus"]
fn exact_retail_uti_corpus_has_native_field_types_and_semantic_roundtrip() {
    let root = env::var(RETAIL_UTI_ROOT_ENV)
        .expect("set M2A_RETAIL_ITEM_UTI_ROOT and run this ignored retail test explicitly");
    let mut paths = fs::read_dir(PathBuf::from(root))
        .expect("read retail UTI root")
        .map(|entry| entry.expect("retail UTI entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "uti"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(
        paths.len(),
        7,
        "the audited retail corpus contains seven UTI files"
    );

    for path in paths {
        let bytes = fs::read(&path).expect("read retail UTI in place");
        let document =
            read_gff_v32(&bytes, &GffLimitsV1::default()).expect("parse retail UTI V3.2");
        assert_eq!(document.file_type, GffFileTypeV1::Uti, "{}", path.display());
        assert!(
            matches!(field(&document, "BaseItem"), Some(GffValueV1::Int(_))),
            "{} BaseItem must be INT",
            path.display()
        );
        assert!(
            matches!(field(&document, "Charges"), Some(GffValueV1::Byte(_))),
            "{} Charges must be BYTE",
            path.display()
        );
        for label in ["ModelPart1", "ModelPart2", "ModelPart3"] {
            assert!(
                field(&document, label).is_none()
                    || matches!(field(&document, label), Some(GffValueV1::Byte(_))),
                "{} {label} must be BYTE when present",
                path.display()
            );
        }

        let rewritten = write_gff_v32(&document, &GffWriterOptionsV1::default())
            .expect("rewrite retail UTI through own GFF writer");
        let readback = read_gff_v32(&rewritten.payload, &GffLimitsV1::default())
            .expect("read rewritten retail UTI");
        assert_eq!(
            readback,
            document,
            "{} must survive a semantic GFF roundtrip",
            path.display()
        );
    }
}
