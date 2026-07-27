use std::{env, fs};

use m2a_core::{
    erf::ErfArchive,
    gff::{GffDocumentV1, GffLimitsV1, GffStructV1, GffValueV1, read_gff_v32},
};
use sha2::{Digest, Sha256};

const RETAIL_MOD_ENV: &str = "M2A_RETAIL_PLACEABLE_MOD";
const PLACEABLE_PALETTE_ENV: &str = "M2A_PLACEABLE_PALETTE_ITP";
const RETAIL_BLUEPRINT_RESREF: &str = "flamingbrazier";
const RETAIL_AREA_RESREF: &str = "itemrestrictions";
const RETAIL_MODULE_SHA256: &str =
    "26044acb596a15849cb93f583e76dd8b45eb66fe37a37fd02f0f844d8cc528b2";
const RETAIL_UTP_SHA256: &str = "b599c6062f85d017115e89a92ba9401aee4d6669dca6fc9117e43b8ede2af059";
const RETAIL_GIT_SHA256: &str = "484bb931a5e23a2de344d6c2d00944f05a5999f42480de3b4dc7007115355076";
const UTP_RESOURCE_TYPE: u16 = 2044;
const GIT_RESOURCE_TYPE: u16 = 2023;
const GIC_RESOURCE_TYPE: u16 = 2046;

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn field<'a>(structure: &'a GffStructV1, label: &str) -> Option<&'a GffValueV1> {
    structure
        .fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
}

fn placeables(document: &GffDocumentV1) -> &[GffStructV1] {
    let Some(GffValueV1::List(placeables)) = field(&document.root, "Placeable List") else {
        return &[];
    };
    placeables
}

fn field_kind(value: &GffValueV1) -> &'static str {
    match value {
        GffValueV1::Byte(_) => "BYTE",
        GffValueV1::Char(_) => "CHAR",
        GffValueV1::Word(_) => "WORD",
        GffValueV1::Short(_) => "SHORT",
        GffValueV1::Dword(_) => "DWORD",
        GffValueV1::Int(_) => "INT",
        GffValueV1::Dword64(_) => "DWORD64",
        GffValueV1::Int64(_) => "INT64",
        GffValueV1::Float(_) => "FLOAT",
        GffValueV1::Double(_) => "DOUBLE",
        GffValueV1::String(_) => "CEXOSTRING",
        GffValueV1::ResRef(_) => "RESREF",
        GffValueV1::LocString(_) => "LOCSTRING",
        GffValueV1::Void(_) => "VOID",
        GffValueV1::Struct(_) => "STRUCT",
        GffValueV1::List(_) => "LIST",
    }
}

fn manifest(structure: &GffStructV1) -> Vec<String> {
    structure
        .fields
        .iter()
        .map(|field| format!("{}:{}", field.label, field_kind(&field.value)))
        .collect()
}

fn print_structure(path: &str, structure: &GffStructV1) {
    println!(
        "{path}:StructID={}:{}",
        structure.struct_id,
        manifest(structure).join(",")
    );
    for field in &structure.fields {
        match &field.value {
            GffValueV1::Struct(child) => {
                print_structure(&format!("{path}.{}", field.label), child);
            }
            GffValueV1::List(children) => {
                for (index, child) in children.iter().enumerate() {
                    print_structure(&format!("{path}.{}[{index}]", field.label), child);
                }
            }
            value => println!("{path}.{}={value:?}", field.label),
        }
    }
}

#[test]
#[ignore = "requires M2A_PLACEABLE_PALETTE_ITP pointing at an owned local placeable palette"]
fn inspect_placeable_palette_itp_manifest() {
    let path = env::var(PLACEABLE_PALETTE_ENV).expect("set M2A_PLACEABLE_PALETTE_ITP");
    let bytes = fs::read(&path).expect("read placeable palette ITP in place");
    let document = read_gff_v32(&bytes, &GffLimitsV1::default()).expect("parse placeable ITP");
    println!("palettePath={path}");
    println!("paletteSha256={}", sha256(&bytes));
    println!("paletteFileType={:?}", document.file_type);
    print_structure("root", &document.root);
}

/// Read-only Aurora First support test. It never writes or vendors retail
/// payloads; it prints the exact typed manifest needed to update the
/// clean-room contract when explicitly supplied with a local retail MOD.
#[test]
#[ignore = "requires M2A_RETAIL_PLACEABLE_MOD pointing at an owned local retail module"]
fn inspect_retail_placeable_blueprint_and_instance_manifest() {
    let path = env::var(RETAIL_MOD_ENV).expect("set M2A_RETAIL_PLACEABLE_MOD");
    let bytes = fs::read(&path).expect("read retail MOD in place");
    let archive = ErfArchive::parse(&bytes).expect("parse retail MOD");
    let utp = archive
        .find(RETAIL_BLUEPRINT_RESREF, UTP_RESOURCE_TYPE)
        .expect("find selected retail UTP");
    let utp_document =
        read_gff_v32(utp, &GffLimitsV1::default()).expect("parse selected retail UTP");

    println!("modulePath={path}");
    let module_sha256 = sha256(&bytes);
    assert_eq!(module_sha256, RETAIL_MODULE_SHA256);
    println!("moduleSha256={module_sha256}");
    println!("utpResref={RETAIL_BLUEPRINT_RESREF}");
    let utp_sha256 = sha256(utp);
    assert_eq!(utp_sha256, RETAIL_UTP_SHA256);
    println!("utpSha256={utp_sha256}");
    println!("utpManifest={}", manifest(&utp_document.root).join(","));

    let git = archive
        .find(RETAIL_AREA_RESREF, GIT_RESOURCE_TYPE)
        .expect("find selected retail GIT");
    let git_sha256 = sha256(git);
    assert_eq!(git_sha256, RETAIL_GIT_SHA256);
    let git_document =
        read_gff_v32(git, &GffLimitsV1::default()).expect("parse selected retail GIT");
    let (instance_index, instance) = placeables(&git_document)
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            matches!(
                field(instance, "TemplateResRef"),
                Some(GffValueV1::ResRef(value)) if value.eq_ignore_ascii_case(RETAIL_BLUEPRINT_RESREF)
            )
        })
        .expect("selected retail blueprint must have a placed instance");
    println!("gitResref={RETAIL_AREA_RESREF}");
    println!("gitSha256={git_sha256}");
    println!("gitPlaceableIndex={instance_index}");
    println!("gitManifest={}", manifest(instance).join(","));

    let gic = archive
        .find(RETAIL_AREA_RESREF, GIC_RESOURCE_TYPE)
        .expect("find selected retail GIC");
    println!("gicSha256={}", sha256(gic));
    match read_gff_v32(gic, &GffLimitsV1::default()) {
        Ok(gic_document) => {
            let gic_placeable = placeables(&gic_document)
                .get(instance_index)
                .expect("GIC Placeable List must align with selected GIT index");
            println!("gicManifest={}", manifest(gic_placeable).join(","));
        }
        Err(source) => {
            // This bundled retail GIC contains non-canonical physical
            // FieldIndices records rejected by the project's deliberately
            // strict reader. The BioWare GIC contract remains the source for
            // the generated one-field Comment struct; record the reader
            // limitation without copying or normalizing the retail payload.
            assert_eq!(source.code, "M6-GFF-LAYOUT-INVALID");
            println!("gicManifest=UNREADABLE_BY_STRICT_READER:{}", source.code);
        }
    }
}
