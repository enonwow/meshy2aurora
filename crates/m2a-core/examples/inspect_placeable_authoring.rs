use std::{env, fs, process};

use m2a_core::placeable::inspect_meshy_static_placeable_authoring_v1;

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: inspect_placeable_authoring <source.glb>");
        process::exit(2);
    };
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        eprintln!("cannot read {path}: {error}");
        process::exit(2);
    });
    let bootstrap = inspect_meshy_static_placeable_authoring_v1(&bytes).unwrap_or_else(|error| {
        eprintln!(
            "{}",
            serde_json::to_string(&error).expect("serialize authoring error")
        );
        process::exit(1);
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&bootstrap).expect("serialize authoring bootstrap")
    );
}
