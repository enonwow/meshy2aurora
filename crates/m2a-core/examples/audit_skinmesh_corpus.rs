use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    mdl::{InspectionReport, NodeReport, inspect_binary_mdl},
};
use serde_json::{Value, json};

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
        .ok_or_else(|| "usage: audit_skinmesh_corpus <archive.hak>".to_owned())?;
    let bytes = fs::read(&path).map_err(|error| format!("archive read failed: {error}"))?;
    let archive = ErfArchive::parse(&bytes).map_err(|error| error.to_string())?;

    let mut binary_model_count = 0usize;
    let mut parsed_model_count = 0usize;
    let mut skin_model_count = 0usize;
    let mut skin_node_count = 0usize;
    let mut controller_histogram = BTreeMap::<String, usize>::new();
    let mut max_skin = None::<Value>;
    let mut max_active_mapped_nodes = None::<Value>;
    let mut active_mapped_node_histogram = BTreeMap::<usize, usize>::new();
    let mut self_contained = Vec::new();
    let mut self_contained_controller_histogram = BTreeMap::<String, usize>::new();
    let mut skin_only_models = Vec::new();
    let mut skin_only_controller_histogram = BTreeMap::<String, usize>::new();

    for resource in archive
        .resources()
        .iter()
        .filter(|resource| resource.resource_type == 2002)
    {
        let payload = archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| error.to_string())?;
        if !payload.starts_with(&[0, 0, 0, 0]) {
            continue;
        }
        binary_model_count += 1;
        let Ok(report) = inspect_binary_mdl(payload) else {
            continue;
        };
        parsed_model_count += 1;

        let mut base_nodes = Vec::new();
        collect_nodes(&report.node_tree.roots, &mut base_nodes);
        let skins = base_nodes
            .iter()
            .copied()
            .filter(|node| node.skin.is_some())
            .collect::<Vec<_>>();
        if skins.is_empty() {
            continue;
        }
        skin_model_count += 1;
        skin_node_count += skins.len();
        let rigid_mesh_count = base_nodes
            .iter()
            .filter(|node| node.mesh.is_some() && node.skin.is_none())
            .count();

        for node in &skins {
            let controller_profile = controller_profile(node);
            *controller_histogram
                .entry(controller_profile.clone())
                .or_default() += 1;
            let mesh = node.mesh.as_ref().expect("skin node has mesh");
            let skin = node.skin.as_ref().expect("skin payload");
            let active_mapped_node_count = skin
                .node_to_bone_map
                .iter()
                .filter(|value| **value >= 0)
                .count();
            *active_mapped_node_histogram
                .entry(active_mapped_node_count)
                .or_default() += 1;
            let replace_max = max_skin
                .as_ref()
                .and_then(|value| value["vertexCount"].as_u64())
                .is_none_or(|current| mesh.vertex_count as u64 > current);
            if replace_max {
                max_skin = Some(json!({
                    "resref": resource.resref,
                    "modelName": report.model.name,
                    "nodeName": node.name,
                    "vertexCount": mesh.vertex_count,
                    "faceCount": mesh.faces.len(),
                    "controllerProfile": controller_profile,
                    "supermodel": report.model.supermodel_name,
                    "animationCount": report.animations.len()
                }));
            }
            let replace_max_active = max_active_mapped_nodes
                .as_ref()
                .and_then(|value| value["activeMappedNodeCount"].as_u64())
                .is_none_or(|current| active_mapped_node_count as u64 > current);
            if replace_max_active {
                max_active_mapped_nodes = Some(json!({
                    "resref": resource.resref,
                    "modelName": report.model.name,
                    "nodeName": node.name,
                    "activeMappedNodeCount": active_mapped_node_count,
                    "mapCount": skin.node_to_bone_map.len(),
                    "vertexCount": mesh.vertex_count,
                    "controllerProfile": controller_profile,
                    "supermodel": report.model.supermodel_name,
                    "animationCount": report.animations.len()
                }));
            }
        }

        if rigid_mesh_count == 0 {
            for node in &skins {
                *skin_only_controller_histogram
                    .entry(controller_profile(node))
                    .or_default() += 1;
            }
            skin_only_models.push(json!({
                "resref": resource.resref,
                "modelName": report.model.name,
                "classification": report.model.classification,
                "supermodel": report.model.supermodel_name,
                "animationCount": report.animations.len(),
                "baseNodeCount": base_nodes.len(),
                "rootPart": report.node_tree.roots.first().expect("root").number,
                "skinCount": skins.len(),
                "skins": skins.iter().map(|node| json!({
                    "name": node.name,
                    "vertexCount": node.mesh.as_ref().expect("skin mesh").vertex_count,
                    "controllerProfile": controller_profile(node),
                    "activeMappedNodeCount": node.skin.as_ref().expect("skin payload")
                        .node_to_bone_map.iter().filter(|value| **value >= 0).count()
                })).collect::<Vec<_>>()
            }));
        }

        if report.model.supermodel_name.eq_ignore_ascii_case("NULL")
            && !report.animations.is_empty()
            && report.model.classification == 4
        {
            for node in &skins {
                *self_contained_controller_histogram
                    .entry(controller_profile(node))
                    .or_default() += 1;
            }
            self_contained.push(summarize_model(
                &resource.resref,
                &report,
                &base_nodes,
                &skins,
            ));
        }
    }

    serde_json::to_string_pretty(&json!({
        "schemaVersion": 1,
        "archivePath": path,
        "archiveByteLength": bytes.len(),
        "binaryModelCount": binary_model_count,
        "parsedModelCount": parsed_model_count,
        "skinModelCount": skin_model_count,
        "skinNodeCount": skin_node_count,
        "skinControllerProfileHistogram": controller_histogram,
        "activeMappedNodeHistogram": active_mapped_node_histogram,
        "maxSkinByActiveMappedNodeCount": max_active_mapped_nodes,
        "maxSkinByVertexCount": max_skin,
        "skinOnlyModels": {
            "definition": "at least one base SkinMesh and zero non-skin mesh nodes",
            "modelCount": skin_only_models.len(),
            "skinControllerProfileHistogram": skin_only_controller_histogram,
            "models": skin_only_models
        },
        "selfContainedAnimatedCharacterSkinFamily": {
            "definition": "classification=4, supermodel=NULL, local animation count > 0, at least one base SkinMesh",
            "modelCount": self_contained.len(),
            "skinControllerProfileHistogram": self_contained_controller_histogram,
            "models": self_contained
        }
    }))
    .map_err(|error| error.to_string())
}

fn summarize_model(
    resref: &str,
    report: &InspectionReport,
    base_nodes: &[&NodeReport],
    skins: &[&NodeReport],
) -> Value {
    let base_skin_names = skins
        .iter()
        .map(|node| node.name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut matching_generic = 0usize;
    let mut matching_skin = 0usize;
    let mut states_without_matching_skin_name = 0usize;
    for animation in &report.animations {
        let mut state_nodes = Vec::new();
        collect_nodes(&animation.node_tree.roots, &mut state_nodes);
        let mut state_has_matching_skin_name = false;
        for node in state_nodes {
            if !base_skin_names.contains(&node.name.to_ascii_lowercase()) {
                continue;
            }
            state_has_matching_skin_name = true;
            matching_generic += usize::from(node.content_flags == 0x01);
            matching_skin += usize::from(node.content_flags == 0x61);
        }
        states_without_matching_skin_name += usize::from(!state_has_matching_skin_name);
    }

    json!({
        "resref": resref,
        "modelName": report.model.name,
        "animationCount": report.animations.len(),
        "animationScale": report.model.animation_scale,
        "baseNodeCount": base_nodes.len(),
        "rootPart": report.node_tree.roots.first().expect("root").number,
        "rootControllerProfile": controller_profile(report.node_tree.roots.first().expect("root")),
        "skinCount": skins.len(),
        "rigidMeshCount": base_nodes
            .iter()
            .filter(|node| node.mesh.is_some() && node.skin.is_none())
            .count(),
        "partOrderMismatchCount": base_nodes
            .iter()
            .enumerate()
            .filter(|(order, node)| node.number as usize != *order)
            .count(),
        "maxSkinVertexCount": skins
            .iter()
            .map(|node| node.mesh.as_ref().expect("skin mesh").vertex_count)
            .max()
            .unwrap_or(0),
        "maxSkinFaceCount": skins
            .iter()
            .map(|node| node.mesh.as_ref().expect("skin mesh").faces.len())
            .max()
            .unwrap_or(0),
        "stateProjection": {
            "matchingGenericNodeCount": matching_generic,
            "matchingSkinNodeCount": matching_skin,
            "statesWithoutMatchingSkinName": states_without_matching_skin_name
        },
        "skins": skins.iter().map(|node| {
            let skin = node.skin.as_ref().expect("skin payload");
            let mesh = node.mesh.as_ref().expect("skin mesh");
            json!({
                "name": node.name,
                "number": node.number,
                "parentOffset": node.parent_offset,
                "vertexCount": mesh.vertex_count,
                "faceCount": mesh.faces.len(),
                "textures": mesh.textures,
                "shadow": mesh.shadow,
                "beaming": mesh.beaming,
                "meshType": mesh.mesh_type,
                "render": mesh.render,
                "transparency": mesh.transparency,
                "renderHint": mesh.render_hint,
                "tileFade": mesh.tile_fade,
                "controllerProfile": controller_profile(node),
                "variant": skin.variant,
                "mapCount": skin.node_to_bone_map.len(),
                "activeMappedNodeCount": skin.node_to_bone_map.iter().filter(|value| **value >= 0).count(),
                "weightsHeader": skin.weights_header,
                "qHeader": skin.q_header,
                "tHeader": skin.t_header,
                "constantsHeader": skin.constants_header,
                "rawWeightsPointer": skin.raw_weights_pointer,
                "rawRefsPointer": skin.raw_refs_pointer,
                "boneConstantValues": skin.bone_constants.iter().copied().collect::<BTreeSet<_>>()
            })
        }).collect::<Vec<_>>()
    })
}

fn controller_profile(node: &NodeReport) -> String {
    node.controllers
        .iter()
        .map(|controller| controller.controller_type.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn collect_nodes<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
    for node in nodes {
        output.push(node);
        collect_nodes(&node.children, output);
    }
}
