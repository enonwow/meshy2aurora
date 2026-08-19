use std::{env, fs, path::PathBuf};

use m2a_core::key_bif::{
    KeyBifFileInputV1, locate_key_bif_resource_v1, resolve_key_bif_resource_sparse_v1,
};

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let key_path = PathBuf::from(args.next().ok_or(
        "usage: extract_key_bif_resource <key-path> <resref> <resource-type> <output-path>",
    )?);
    let resref = args.next().ok_or("missing resref")?;
    let resource_type = args
        .next()
        .ok_or("missing resource type")?
        .parse::<u16>()
        .map_err(|error| format!("invalid resource type: {error}"))?;
    let output_path = PathBuf::from(args.next().ok_or("missing output path")?);
    if args.next().is_some() {
        return Err("too many arguments".to_owned());
    }
    if output_path.exists() {
        return Err(format!(
            "refusing to overwrite existing output {}",
            output_path.display()
        ));
    }

    let key_file_name = key_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("invalid KEY filename {}", key_path.display()))?;
    let install_root = key_path
        .parent()
        .and_then(|value| value.parent())
        .ok_or_else(|| format!("KEY path has no install root: {}", key_path.display()))?;
    let key = fs::read(&key_path)
        .map_err(|error| format!("failed to read {}: {error}", key_path.display()))?;
    let locator = locate_key_bif_resource_v1(key_file_name, &key, &resref, resource_type)
        .map_err(|error| error.to_string())?;
    let bif_path = install_root.join(locator.bif_logical_name.replace('/', "\\"));
    let bif = fs::read(&bif_path)
        .map_err(|error| format!("failed to read {}: {error}", bif_path.display()))?;
    let bif_input = KeyBifFileInputV1 {
        logical_name: locator.bif_logical_name.clone(),
        bytes: &bif,
    };
    let resolved =
        resolve_key_bif_resource_sparse_v1(key_file_name, &key, &bif_input, &resref, resource_type)
            .map_err(|error| error.to_string())?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&output_path, resolved.payload)
        .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
    println!(
        "extracted {}:{} -> {} ({} bytes, sha256={})",
        resolved.resref,
        resolved.resource_type,
        output_path.display(),
        resolved.payload.len(),
        resolved.payload_sha256
    );
    Ok(())
}
