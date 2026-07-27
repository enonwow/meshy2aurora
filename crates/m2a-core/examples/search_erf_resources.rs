use std::{env, fs, path::PathBuf, process::ExitCode};

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
    let mut arguments = env::args().skip(1);
    let path = arguments.next().map(PathBuf::from).ok_or_else(|| {
        "usage: search_erf_resources <archive> [--resref-only] <keyword>...".to_owned()
    })?;
    let arguments = arguments.collect::<Vec<_>>();
    let resref_only = arguments.iter().any(|value| value == "--resref-only");
    let keywords = arguments
        .into_iter()
        .filter(|value| value != "--resref-only")
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if keywords.is_empty() {
        return Err("at least one keyword is required".to_owned());
    }
    let bytes = fs::read(&path).map_err(|error| format!("archive read failed: {error}"))?;
    let archive = ErfArchive::parse(&bytes).map_err(|error| error.to_string())?;
    let matches = archive
        .resources()
        .iter()
        .filter_map(|resource| {
            let payload = archive
                .find(&resource.resref, resource.resource_type)
                .ok()?;
            let resref = resource.resref.to_ascii_lowercase();
            let matched_keywords = keywords
                .iter()
                .filter(|keyword| {
                    resref.contains(keyword.as_str())
                        || (!resref_only
                            && contains_ascii_case_insensitive(payload, keyword.as_bytes()))
                })
                .cloned()
                .collect::<Vec<_>>();
            (!matched_keywords.is_empty()).then(|| {
                json!({
                    "resref": resource.resref,
                    "resourceId": resource.resource_id,
                    "resourceType": resource.resource_type,
                    "offset": resource.offset,
                    "byteLength": payload.len(),
                    "sha256": format!("{:x}", Sha256::digest(payload)),
                    "matchedKeywords": matched_keywords
                })
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&json!({
        "archivePath": path,
        "archiveByteLength": bytes.len(),
        "archiveSha256": format!("{:x}", Sha256::digest(&bytes)),
        "keywords": keywords,
        "resrefOnly": resref_only,
        "matchCount": matches.len(),
        "matches": matches
    }))
    .map_err(|error| error.to_string())
}

fn contains_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.windows(needle.len()).any(|window| {
            window
                .iter()
                .zip(needle)
                .all(|(left, right)| left.eq_ignore_ascii_case(right))
        })
}
