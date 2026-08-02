use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    animation_library::{
        AnimationContributionV1, AnimationLibraryCatalogSourceV1, AnimationPresetManifestV1,
        AnimationPresetPayloadV1, AnimationRigProfileV1, animation_preset_motion_sha256_v1,
        build_community_animation_catalog_from_root_v1, install_animation_contribution_v1,
        sha256_hex_v1,
    },
    model_pipeline::inspect_editable_animation_source_v1,
};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("rig-profile") => print_rig_profile(&args[1..]),
        Some("catalog") => catalog(&args[1..]),
        Some("hydrate-manifests") => hydrate_manifests(&args[1..]),
        Some("install-contribution") => install_contribution(&args[1..]),
        _ => Err(usage()),
    }
}

fn install_contribution(args: &[String]) -> Result<String, String> {
    let input = required_path(args, "--input")?;
    let root = required_path(args, "--root")?;
    let bytes = fs::read(&input).map_err(|error| {
        format!(
            "ANIMATION-LIBRARY-CONTRIBUTION-READ {}: {error}",
            input.display()
        )
    })?;
    let contribution: AnimationContributionV1 =
        serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "ANIMATION-LIBRARY-CONTRIBUTION-SCHEMA {}: {error}",
                input.display()
            )
        })?;
    let destination = install_animation_contribution_v1(&root, &contribution)
        .map_err(|diagnostics| serde_json::to_string_pretty(&diagnostics).unwrap_or_default())?;
    Ok(format!(
        "animation-library-contribution-installed: {}",
        destination.display()
    ))
}

fn hydrate_manifests(args: &[String]) -> Result<String, String> {
    let root = required_path(args, "--root")?;
    let rig_profile = optional_path(args, "--rig-profile")
        .map(|path| {
            let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            serde_json::from_slice::<AnimationRigProfileV1>(&bytes)
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .transpose()?;
    let presets_root = root.join("presets");
    let mut hydrated = 0usize;
    for entry in fs::read_dir(&presets_root).map_err(|error| {
        format!(
            "ANIMATION-LIBRARY-PRESET-READ {}: {error}",
            presets_root.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("ANIMATION-LIBRARY-PRESET-READ: {error}"))?;
        if !entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        let animation_path = entry.path().join("animation.json");
        let mut manifest: AnimationPresetManifestV1 = serde_json::from_slice(
            &fs::read(&manifest_path)
                .map_err(|error| format!("{}: {error}", manifest_path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
        let animation_bytes = fs::read(&animation_path)
            .map_err(|error| format!("{}: {error}", animation_path.display()))?;
        let animation: AnimationPresetPayloadV1 = serde_json::from_slice(&animation_bytes)
            .map_err(|error| format!("{}: {error}", animation_path.display()))?;
        manifest.duration_seconds = animation.duration_seconds;
        manifest.animation_byte_length = animation_bytes.len() as u64;
        manifest.animation_sha256 = sha256_hex_v1(&animation_bytes);
        manifest.motion_sha256 =
            animation_preset_motion_sha256_v1(&animation).map_err(|diagnostics| {
                serde_json::to_string_pretty(&diagnostics).unwrap_or_default()
            })?;
        if let Some(profile) = &rig_profile {
            manifest.rig_signature_sha256 = profile.signature_sha256.clone();
        }
        let preview_path = entry.path().join("preview.webp");
        if preview_path.is_file() {
            let preview_bytes = fs::read(&preview_path)
                .map_err(|error| format!("{}: {error}", preview_path.display()))?;
            manifest.preview_path = Some("preview.webp".to_owned());
            manifest.preview_byte_length = Some(preview_bytes.len() as u64);
            manifest.preview_sha256 = Some(sha256_hex_v1(&preview_bytes));
        } else {
            manifest.preview_path = None;
            manifest.preview_byte_length = None;
            manifest.preview_sha256 = None;
        }
        manifest.required_bones = animation
            .tracks
            .iter()
            .map(|track| track.target_bone_name.clone())
            .chain(std::iter::once(animation.animation_root_bone_name.clone()))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        fs::write(
            &manifest_path,
            serde_json::to_vec(&manifest)
                .map_err(|error| format!("{}: {error}", manifest_path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
        hydrated += 1;
    }
    Ok(format!("animation-library-manifests-hydrated: {hydrated}"))
}

fn print_rig_profile(args: &[String]) -> Result<String, String> {
    let source = required_path(args, "--source-glb")?;
    let bytes = fs::read(&source).map_err(|error| {
        format!(
            "ANIMATION-LIBRARY-SOURCE-READ {}: {error}",
            source.display()
        )
    })?;
    let inspection = inspect_editable_animation_source_v1(&bytes)
        .map_err(|error| format!("ANIMATION-LIBRARY-RIG-INSPECTION: {error}"))?;
    let profile = AnimationRigProfileV1::from_rig(&inspection.rig)
        .map_err(|diagnostics| serde_json::to_string_pretty(&diagnostics).unwrap_or_default())?;
    let mut bytes = serde_json::to_vec_pretty(&profile)
        .map_err(|error| format!("ANIMATION-LIBRARY-RIG-SERIALIZE: {error}"))?;
    bytes.push(b'\n');
    if let Some(output) = optional_path(args, "--output") {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "ANIMATION-LIBRARY-RIG-DIRECTORY {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&output, bytes).map_err(|error| {
            format!("ANIMATION-LIBRARY-RIG-WRITE {}: {error}", output.display())
        })?;
        Ok(format!(
            "animation-library-rig-profile-written: {}; sha256 {}",
            output.display(),
            profile.signature_sha256
        ))
    } else {
        String::from_utf8(bytes)
            .map_err(|error| format!("ANIMATION-LIBRARY-RIG-SERIALIZE: {error}"))
    }
}

fn catalog(args: &[String]) -> Result<String, String> {
    let root = required_path(args, "--root")?;
    let output = required_path(args, "--output")?;
    let check = args.iter().any(|argument| argument == "--check");
    let write = args.iter().any(|argument| argument == "--write");
    if check == write {
        return Err("catalog requires exactly one of --check or --write".to_owned());
    }
    let (catalog, _) = build_community_animation_catalog_from_root_v1(
        &root,
        AnimationLibraryCatalogSourceV1::EmbeddedRelease,
    )
    .map_err(|diagnostics| serde_json::to_string_pretty(&diagnostics).unwrap_or_default())?;
    let mut bytes = serde_json::to_vec_pretty(&catalog)
        .map_err(|error| format!("ANIMATION-LIBRARY-CATALOG-SERIALIZE: {error}"))?;
    bytes.push(b'\n');
    if check {
        let current = fs::read(&output).map_err(|error| {
            format!(
                "ANIMATION-LIBRARY-CATALOG-READ {}: {error}",
                output.display()
            )
        })?;
        if current != bytes {
            return Err(format!(
                "ANIMATION-LIBRARY-CATALOG-STALE: {} differs from the deterministic Core projection; run --write",
                output.display()
            ));
        }
        Ok(format!(
            "animation-library-catalog-ok: {} presets; sha256 {}",
            catalog.entries.len(),
            catalog.catalog_sha256
        ))
    } else {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "ANIMATION-LIBRARY-CATALOG-DIRECTORY {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&output, bytes).map_err(|error| {
            format!(
                "ANIMATION-LIBRARY-CATALOG-WRITE {}: {error}",
                output.display()
            )
        })?;
        Ok(format!(
            "animation-library-catalog-written: {} presets; sha256 {}",
            catalog.entries.len(),
            catalog.catalog_sha256
        ))
    }
}

fn required_path(args: &[String], name: &str) -> Result<PathBuf, String> {
    args.iter()
        .position(|argument| argument == name)
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .ok_or_else(usage)
}

fn optional_path(args: &[String], name: &str) -> Option<PathBuf> {
    args.iter()
        .position(|argument| argument == name)
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
}

fn usage() -> String {
    "usage: community_animation_library rig-profile --source-glb <path> [--output <profile.json>] | hydrate-manifests --root <animation-library> [--rig-profile <profile.json>] | install-contribution --input <json> --root <animation-library> | catalog --root <animation-library> --output <catalog.json> (--check|--write)".to_owned()
}
