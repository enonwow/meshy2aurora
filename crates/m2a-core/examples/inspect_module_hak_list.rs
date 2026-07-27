use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::{GffValueV1, read_gff_v32},
};
use serde_json::json;
use sha2::{Digest, Sha256};

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: inspect_module_hak_list <module.mod>".to_owned())?;
    let bytes = fs::read(&path).map_err(|error| format!("module read failed: {error}"))?;
    let archive = ErfArchive::parse(&bytes).map_err(|error| error.to_string())?;
    if archive.file_type() != ErfFileType::Module {
        return Err("input is not MOD V1.0".to_owned());
    }
    let ifo = read_gff_v32(
        archive
            .find("module", 2014)
            .map_err(|error| format!("module.ifo missing: {error}"))?,
        &Default::default(),
    )
    .map_err(|error| format!("module.ifo read failed: {error}"))?;
    let hak_list = ifo
        .root
        .fields
        .iter()
        .find(|field| field.label == "Mod_HakList")
        .ok_or_else(|| "module.ifo has no Mod_HakList".to_owned())?;
    let GffValueV1::List(items) = &hak_list.value else {
        return Err("module.ifo Mod_HakList is not a list".to_owned());
    };
    let ordered_haks = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let value = item
                .fields
                .iter()
                .find(|field| field.label == "Mod_Hak")
                .ok_or_else(|| format!("Mod_HakList[{index}] has no Mod_Hak"))?;
            let GffValueV1::String(bytes) = &value.value else {
                return Err(format!("Mod_HakList[{index}].Mod_Hak is not a string"));
            };
            String::from_utf8(bytes.clone())
                .map_err(|_| format!("Mod_HakList[{index}].Mod_Hak is not UTF-8"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    serde_json::to_string_pretty(&json!({
        "modulePath": path,
        "moduleByteLength": bytes.len(),
        "moduleSha256": format!("{:x}", Sha256::digest(&bytes)),
        "orderedHakResrefs": ordered_haks
    }))
    .map_err(|error| error.to_string())
}
