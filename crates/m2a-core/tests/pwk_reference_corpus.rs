use std::{collections::BTreeSet, env, fs};

use m2a_core::{erf::ErfArchive, placeable_collision::inspect_ascii_placeable_walkmesh_v1};
use serde::Serialize;
use sha2::{Digest, Sha256};

const PWK_RESOURCE_TYPE: u16 = 2053;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MeshSummary {
    node_name: String,
    content_flags: u32,
    vertex_count: usize,
    face_count: usize,
    surface_ids: Vec<i32>,
    unsupported_families: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PwkSummary {
    resref: String,
    sha256: String,
    byte_length: usize,
    format: String,
    model_name: Option<String>,
    geometry_type: Option<u32>,
    classification: Option<u8>,
    fog: Option<u8>,
    node_count: Option<usize>,
    meshes: Vec<MeshSummary>,
    ascii_node_kinds: Vec<String>,
    ascii_use_point_count: usize,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn summarize_ascii_pwk(resref: &str, pwk: &[u8]) -> PwkSummary {
    let report = inspect_ascii_placeable_walkmesh_v1(pwk)
        .unwrap_or_else(|error| {
            panic!(
                "retail/reference PWK {resref} must pass the runtime-aligned ASCII reader: {error:?}; prefix={:?}",
                String::from_utf8_lossy(pwk).lines().take(12).collect::<Vec<_>>()
            )
        });
    let meshes = report
        .mesh_nodes
        .into_iter()
        .map(|mesh| MeshSummary {
            node_name: mesh.node_name,
            content_flags: 0,
            vertex_count: mesh.vertices.len(),
            face_count: mesh.faces.len(),
            surface_ids: mesh
                .faces
                .iter()
                .map(|face| face.surface_id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            unsupported_families: Vec::new(),
        })
        .collect();

    PwkSummary {
        resref: resref.to_owned(),
        sha256: sha256(pwk),
        byte_length: pwk.len(),
        format: "ascii".to_owned(),
        model_name: None,
        geometry_type: None,
        classification: None,
        fog: None,
        node_count: None,
        meshes,
        ascii_node_kinds: vec!["trimesh".to_owned()],
        ascii_use_point_count: report.use_points.len(),
    }
}

#[test]
#[ignore = "requires caller-provided M2A_PWK_CORPUS_HAK and M2A_PWK_CORPUS_RESREFS"]
fn inspect_external_pwk_corpus_in_place() {
    let hak_path = env::var("M2A_PWK_CORPUS_HAK").expect("M2A_PWK_CORPUS_HAK");
    let resrefs = env::var("M2A_PWK_CORPUS_RESREFS").expect("M2A_PWK_CORPUS_RESREFS");
    let hak_bytes = fs::read(&hak_path).expect("read-only HAK corpus");
    let archive = ErfArchive::parse(&hak_bytes).expect("HAK corpus");
    let selected_resrefs = if resrefs.trim() == "*" {
        archive
            .resources()
            .iter()
            .filter(|resource| resource.resource_type == PWK_RESOURCE_TYPE)
            .take(8)
            .map(|resource| resource.resref.clone())
            .collect::<Vec<_>>()
    } else {
        resrefs
            .split(',')
            .map(str::trim)
            .filter(|resref| !resref.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert!(
        !selected_resrefs.is_empty(),
        "HAK has no selected PWK resources"
    );
    let summaries = selected_resrefs
        .iter()
        .map(String::as_str)
        .map(|resref| {
            let pwk = archive
                .find(resref, PWK_RESOURCE_TYPE)
                .expect("PWK resource");
            summarize_ascii_pwk(resref, pwk)
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        serde_json::to_string_pretty(&summaries).expect("serialize PWK corpus summary")
    );
}
