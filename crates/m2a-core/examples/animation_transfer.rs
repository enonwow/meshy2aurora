use std::{env, fs, process::ExitCode};

use m2a_core::animation_retarget::{
    AnimationTransferModeV1, AnimationTransferOptionsV1, copy_animation_clip_between_models_v1,
};

fn main() -> ExitCode {
    match run() {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let mut target = None;
    let mut donor = None;
    let mut clip = None;
    let mut clip_id = None;
    let mut output_name = None;
    let mut mode = None;
    let mut output = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {argument}"))?;
        match argument.as_str() {
            "--target" => target = Some(value),
            "--donor" => donor = Some(value),
            "--clip" => clip = Some(value),
            "--clip-id" => clip_id = Some(value),
            "--output-name" => output_name = Some(value),
            "--mode" => {
                mode = Some(match value.as_str() {
                    "EXACT_RIG_COPY_V1" => AnimationTransferModeV1::ExactRigCopyV1,
                    "SAME_HIERARCHY_RETARGET_V1" => {
                        AnimationTransferModeV1::SameHierarchyRetargetV1
                    }
                    _ => return Err(format!("unsupported transfer mode {value:?}")),
                });
            }
            "--output" => output = Some(value),
            _ => return Err(format!("unknown argument {argument:?}")),
        }
    }
    let target_path = target.ok_or("--target is required")?;
    let donor_path = donor.ok_or("--donor is required")?;
    let clip_name = clip.ok_or("--clip is required")?;
    let target_bytes = fs::read(&target_path)
        .map_err(|error| format!("could not read target {target_path:?}: {error}"))?;
    let donor_bytes = fs::read(&donor_path)
        .map_err(|error| format!("could not read donor {donor_path:?}: {error}"))?;
    let result = copy_animation_clip_between_models_v1(
        &target_bytes,
        &donor_bytes,
        &clip_name,
        &AnimationTransferOptionsV1 {
            mode: mode.ok_or("--mode is required")?,
            clip_id: clip_id.ok_or("--clip-id is required")?,
            output_name: output_name.ok_or("--output-name is required")?,
        },
    )
    .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    let json = serde_json::to_string(&result).map_err(|error| error.to_string())?;
    if let Some(output_path) = output {
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("could not create output directory: {error}"))?;
        }
        fs::write(&output_path, json.as_bytes())
            .map_err(|error| format!("could not write output {output_path:?}: {error}"))?;
    }
    Ok(json)
}
