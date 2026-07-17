use std::fs;

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::read_gff_v32,
};

#[test]
fn own_reader_accepts_native_toolset_module_gff_layout() {
    let Some(path) = std::env::var_os("M2A_REFERENCE_TOOLSET_MODULE") else {
        eprintln!("skipped: M2A_REFERENCE_TOOLSET_MODULE is not set");
        return;
    };
    let bytes = fs::read(path).expect("read native Toolset module");
    let archive = ErfArchive::parse(&bytes).expect("parse MOD V1.0");
    assert_eq!(archive.file_type(), ErfFileType::Module);

    for resource in archive.resources() {
        if matches!(resource.resource_type, 2012 | 2014 | 2023 | 2027 | 2046) {
            let payload = archive
                .find(&resource.resref, resource.resource_type)
                .expect("resolve GFF resource");
            read_gff_v32(payload, &Default::default()).unwrap_or_else(|error| {
                panic!(
                    "parse native GFF {} type {}: {error}",
                    resource.resref, resource.resource_type
                )
            });
        }
    }
}
