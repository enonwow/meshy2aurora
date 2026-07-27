use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::erf::ErfArchive;
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
    let command = parse(env::args().skip(1))?;
    if command.output.exists() {
        return Err(format!(
            "output already exists: {}",
            command.output.display()
        ));
    }
    let bytes =
        fs::read(&command.archive).map_err(|error| format!("archive read failed: {error}"))?;
    fs::create_dir_all(&command.output)
        .map_err(|error| format!("output create failed: {error}"))?;
    let mut outputs = Vec::with_capacity(command.resources.len());
    if command.lenient_unrelated_resrefs {
        for identity in &command.resources {
            let payload = find_resource_lenient(&bytes, identity)?;
            outputs.push(extract_one(&command.output, identity, payload)?);
        }
    } else {
        let archive = ErfArchive::parse(&bytes).map_err(|error| error.to_string())?;
        for identity in &command.resources {
            let payload = archive
                .find(&identity.resref, identity.resource_type)
                .map_err(|error| {
                    format!(
                        "resource missing {}:{}: {error}",
                        identity.resref, identity.resource_type
                    )
                })?;
            outputs.push(extract_one(&command.output, identity, payload)?);
        }
    }
    let manifest = json!({
        "schemaVersion": 1,
        "sourceArchive": command.archive,
        "sourceArchiveByteLength": bytes.len(),
        "sourceArchiveSha256": format!("{:x}", Sha256::digest(&bytes)),
        "lenientUnrelatedResrefs": command.lenient_unrelated_resrefs,
        "outputs": outputs
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    write_new(
        &command.output.join("extraction-manifest.json"),
        &manifest_bytes,
    )?;
    serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())
}

struct Command {
    archive: PathBuf,
    output: PathBuf,
    resources: Vec<ResourceIdentity>,
    lenient_unrelated_resrefs: bool,
}

struct ResourceIdentity {
    resref: String,
    resource_type: u16,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut values = arguments.into_iter();
    let archive = values.next().map(PathBuf::from).ok_or_else(usage)?;
    let mut output = None;
    let mut resources = Vec::new();
    let mut lenient_unrelated_resrefs = false;
    while let Some(argument) = values.next() {
        match argument.as_str() {
            "--lenient-unrelated-resrefs" => {
                lenient_unrelated_resrefs = true;
            }
            "--out" => {
                if output.is_some() {
                    return Err("--out supplied more than once".to_owned());
                }
                output = values.next().map(PathBuf::from);
                if output.is_none() {
                    return Err("--out requires a path".to_owned());
                }
            }
            "--resource" => {
                let value = values
                    .next()
                    .ok_or_else(|| "--resource requires resref:type".to_owned())?;
                let (resref, resource_type) = value
                    .rsplit_once(':')
                    .ok_or_else(|| format!("invalid resource identity: {value}"))?;
                resources.push(ResourceIdentity {
                    resref: resref.to_owned(),
                    resource_type: resource_type
                        .parse()
                        .map_err(|_| format!("invalid resource type: {resource_type}"))?,
                });
            }
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }
    if resources.is_empty() {
        return Err("at least one --resource is required".to_owned());
    }
    Ok(Command {
        archive,
        output: output.ok_or_else(usage)?,
        resources,
        lenient_unrelated_resrefs,
    })
}

fn usage() -> String {
    "usage: extract_erf_resources <archive> [--lenient-unrelated-resrefs] --out \
     <new-directory> --resource <resref:type>..."
        .to_owned()
}

fn extract_one(
    output: &Path,
    identity: &ResourceIdentity,
    payload: &[u8],
) -> Result<serde_json::Value, String> {
    let file_name = format!("{}.{}", identity.resref, extension(identity.resource_type));
    let path = output.join(&file_name);
    write_new(&path, payload)?;
    Ok(json!({
        "resref": identity.resref,
        "resourceType": identity.resource_type,
        "fileName": file_name,
        "byteLength": payload.len(),
        "sha256": format!("{:x}", Sha256::digest(payload))
    }))
}

fn find_resource_lenient<'a>(
    bytes: &'a [u8],
    identity: &ResourceIdentity,
) -> Result<&'a [u8], String> {
    const HEADER_SIZE: usize = 160;
    const KEY_SIZE: usize = 24;
    const RESOURCE_SIZE: usize = 8;
    const MAX_ENTRIES: usize = 262_144;

    if bytes.len() < HEADER_SIZE
        || !matches!(&bytes[0..4], b"ERF " | b"HAK " | b"MOD ")
        || &bytes[4..8] != b"V1.0"
    {
        return Err("lenient extraction requires an ERF/HAK/MOD V1.0 archive".to_owned());
    }
    let entry_count = read_u32(bytes, 16)? as usize;
    if entry_count > MAX_ENTRIES {
        return Err(format!(
            "entry count {entry_count} exceeds safety limit {MAX_ENTRIES}"
        ));
    }
    let key_offset = read_u32(bytes, 24)? as usize;
    let resource_offset = read_u32(bytes, 28)? as usize;
    checked_range(bytes, key_offset, entry_count, KEY_SIZE, "key table")?;
    checked_range(
        bytes,
        resource_offset,
        entry_count,
        RESOURCE_SIZE,
        "resource table",
    )?;

    let mut found = None;
    for index in 0..entry_count {
        let key = key_offset + index * KEY_SIZE;
        let raw_resref = &bytes[key..key + 16];
        let resref_end = raw_resref
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(raw_resref.len());
        let resource_type = read_u16(bytes, key + 20)?;
        if resource_type != identity.resource_type
            || !raw_resref[..resref_end].eq_ignore_ascii_case(identity.resref.as_bytes())
        {
            continue;
        }
        if found.is_some() {
            return Err(format!(
                "duplicate resource {}:{}",
                identity.resref, identity.resource_type
            ));
        }
        let resource_id = read_u32(bytes, key + 16)? as usize;
        if resource_id >= entry_count {
            return Err(format!(
                "resource id {resource_id} is outside entry count {entry_count}"
            ));
        }
        let descriptor = resource_offset + resource_id * RESOURCE_SIZE;
        let payload_offset = read_u32(bytes, descriptor)? as usize;
        let payload_size = read_u32(bytes, descriptor + 4)? as usize;
        let end = payload_offset
            .checked_add(payload_size)
            .ok_or_else(|| "resource payload range overflow".to_owned())?;
        let payload = bytes
            .get(payload_offset..end)
            .ok_or_else(|| "resource payload range is outside archive".to_owned())?;
        found = Some(payload);
    }
    found.ok_or_else(|| {
        format!(
            "resource missing {}:{}",
            identity.resref, identity.resource_type
        )
    })
}

fn checked_range(
    bytes: &[u8],
    offset: usize,
    count: usize,
    stride: usize,
    label: &str,
) -> Result<(), String> {
    let size = count
        .checked_mul(stride)
        .ok_or_else(|| format!("{label} size overflow"))?;
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("{label} range overflow"))?;
    if offset < 160 || end > bytes.len() {
        return Err(format!("{label} range is outside archive"));
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 read at {offset} is outside archive"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 read at {offset} is outside archive"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn extension(resource_type: u16) -> &'static str {
    match resource_type {
        3 => "tga",
        2002 => "mdl",
        2009 => "nss",
        2017 => "2da",
        2025 => "uti",
        2033 => "dds",
        2044 => "utp",
        2051 => "mtr",
        2053 => "pwk",
        2080 => "png",
        _ => "bin",
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create failed {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("write failed {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::extension;

    #[test]
    fn maps_nui_png_and_common_aurora_resource_extensions() {
        assert_eq!(extension(2009), "nss");
        assert_eq!(extension(2025), "uti");
        assert_eq!(extension(2053), "pwk");
        assert_eq!(extension(2080), "png");
    }
}
