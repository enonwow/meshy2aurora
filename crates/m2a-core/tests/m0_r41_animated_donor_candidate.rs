use std::{collections::BTreeMap, fs, path::PathBuf};

use m2a_core::{
    animated_donor_candidate::{
        M0_R41_ACCEPTED_R40_PROOF_SHA256, M0_R41_APPEARANCE_ROW, M0_R41_AREA_RESREF,
        M0_R41_HAK_RESREF, M0_R41_MODEL_RESREF, M0_R41_MODEL_SHA256, M0_R41_MODULE_RESREF,
        M0_R41_TEXTURE_RESREF, build_m0_r40_animated_donor_candidate_v1,
        build_m0_r41_animated_donor_candidate_v1, verify_m0_r41_animated_donor_candidate_v1,
    },
    erf::ErfArchive,
    mdl::{
        MdlFormatProfileV1, MdlStateProjectionProfileV1, inspect_binary_mdl,
        verify_direct_creature_state_projection_v1,
    },
    proof_module::inspect_binary_creature_multi_fixture_module_v1,
};

#[test]
#[ignore = "requires M2A_REQUIRE_RUNTIME_WITNESSES=1 and the exact local M0/H1 GLBs"]
fn exact_m0_r41_projects_every_rigid_mesh_identity_into_each_type5_state() {
    assert_eq!(
        std::env::var("M2A_REQUIRE_RUNTIME_WITNESSES").as_deref(),
        Ok("1"),
        "set M2A_REQUIRE_RUNTIME_WITNESSES=1 to admit exact local witnesses"
    );
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = fs::read(
        repo.join("proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb"),
    )
    .expect("exact M0 source");
    let donor =
        fs::read(repo.join("sample-3d/h1-humanoid-1500/source.glb")).expect("exact H1 donor");
    let appearance =
        fs::read(repo.join("local-reference-assets/appearance.2da")).expect("exact appearance");

    let r40 = build_m0_r40_animated_donor_candidate_v1(&source, &donor, &appearance)
        .expect("exact r40 baseline");
    let artifact = build_m0_r41_animated_donor_candidate_v1(&source, &donor, &appearance)
        .expect("exact r41 candidate");
    verify_m0_r41_animated_donor_candidate_v1(
        &artifact.contract,
        &source,
        &donor,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .expect("deterministic exact-input replay");

    assert!(artifact.contract.candidate_admissible);
    assert_eq!(
        artifact.contract.admitted_by_r40_proof_sha256,
        M0_R41_ACCEPTED_R40_PROOF_SHA256
    );
    assert_eq!(artifact.contract.model.resref, M0_R41_MODEL_RESREF);
    assert_eq!(artifact.contract.model.sha256, M0_R41_MODEL_SHA256);
    assert_eq!(artifact.contract.rigid_readback.base_node_count, 45);
    assert_eq!(artifact.contract.rigid_readback.rig_node_count, 25);
    assert_eq!(artifact.contract.rigid_readback.rigid_mesh_node_count, 20);
    assert_eq!(artifact.contract.rigid_readback.skin_node_count, 0);
    assert_eq!(artifact.contract.rigid_readback.triangle_count, 1_569);
    assert_eq!(
        artifact.contract.rigid_readback.duplicated_vertex_count,
        4_707
    );
    assert_eq!(
        artifact.contract.rigid_readback.active_parent_bone_count,
        20
    );
    assert_eq!(
        artifact.contract.rigid_readback.aurora_root_name,
        M0_R41_MODEL_RESREF
    );
    assert_eq!(artifact.contract.rigid_readback.skeleton_root_name, "Hips");
    assert_eq!(
        artifact.contract.rigid_readback.animation_node_counts,
        vec![45; 7]
    );
    assert_eq!(
        artifact
            .contract
            .rigid_readback
            .animation_mesh_identity_counts,
        vec![20; 7]
    );
    assert_eq!(
        artifact
            .contract
            .rigid_readback
            .animation_scale_controller_count,
        0
    );
    assert_eq!(
        artifact.contract.rigid_readback.base_root_controller_count,
        0
    );
    assert_eq!(
        artifact.contract.rigid_readback.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
    );
    assert_eq!(
        artifact.contract.rigid_readback.state_projection_profile,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
    );

    let model = inspect_binary_mdl(&artifact.model).expect("r41 MDL readback");
    verify_direct_creature_state_projection_v1(
        &model,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .expect("r41 must match the native retail full-topology state family");
    let base_identity = identity_map(
        &serde_json::to_value(&model.node_tree.roots).expect("serialize base node tree"),
    );
    assert_eq!(base_identity.len(), 45);
    for animation in &model.animations {
        assert_eq!(animation.animation_type, 5);
        let state_identity = identity_map(
            &serde_json::to_value(&animation.node_tree.roots).expect("serialize state node tree"),
        );
        assert_eq!(state_identity, base_identity);
        let mut state_nodes = animation.node_tree.roots.iter().collect::<Vec<_>>();
        let mut flattened = Vec::new();
        while let Some(node) = state_nodes.pop() {
            flattened.push(node);
            state_nodes.extend(&node.children);
        }
        assert!(flattened.iter().all(|node| {
            node.content_flags == 0x01
                && node.mesh.is_none()
                && node.skin.is_none()
                && node
                    .controllers
                    .iter()
                    .all(|controller| controller.controller_type != 36)
        }));
        assert_eq!(
            flattened
                .iter()
                .filter(|node| node.name.starts_with("m2a_seg_"))
                .count(),
            20
        );
    }

    let r40_model = inspect_binary_mdl(&r40.model).expect("r40 baseline readback");
    assert_eq!(
        rigid_mesh_surface(
            &serde_json::to_value(&model.node_tree.roots).expect("serialize r41 base tree")
        ),
        rigid_mesh_surface(
            &serde_json::to_value(&r40_model.node_tree.roots).expect("serialize r40 base tree")
        ),
        "r41 must not change the r40 rigid geometry, UVs, normals or faces"
    );

    let hak = ErfArchive::parse(&artifact.hak).expect("r41 HAK");
    assert_eq!(hak.resources().len(), 3);
    assert_eq!(hak.find(M0_R41_MODEL_RESREF, 2002).unwrap(), artifact.model);
    assert_eq!(
        hak.find(M0_R41_TEXTURE_RESREF, 3).unwrap(),
        artifact.texture
    );
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        artifact.appearance_two_da
    );

    let scene = inspect_binary_creature_multi_fixture_module_v1(&artifact.module).expect("r41 MOD");
    assert_eq!(scene.module_resref, M0_R41_MODULE_RESREF);
    assert_eq!(scene.area_resref, M0_R41_AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [M0_R41_HAK_RESREF]);
    assert_eq!(scene.fixtures.len(), 1);
    assert_eq!(scene.fixtures[0].appearance_row, M0_R41_APPEARANCE_ROW);

    let mut wrong_source = source.clone();
    *wrong_source.last_mut().expect("source byte") ^= 1;
    let error = build_m0_r41_animated_donor_candidate_v1(&wrong_source, &donor, &appearance)
        .expect_err("source identity attack");
    assert_eq!(error.code, "M2A-R41-INPUT-IDENTITY");

    let mut wrong_contract = artifact.contract.clone();
    wrong_contract.model.sha256 = "0".repeat(64);
    let error = verify_m0_r41_animated_donor_candidate_v1(
        &wrong_contract,
        &source,
        &donor,
        &appearance,
        &artifact.module,
        &artifact.hak,
    )
    .expect_err("contract mutation");
    assert_eq!(error.code, "M2A-R41-REPLAY-MISMATCH");
}

fn flattened_nodes(nodes: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut pending = nodes
        .as_array()
        .expect("node array")
        .iter()
        .collect::<Vec<_>>();
    let mut flattened = Vec::new();
    while let Some(node) = pending.pop() {
        flattened.push(node);
        pending.extend(node["children"].as_array().expect("children array").iter());
    }
    flattened
}

fn identity_map(nodes: &serde_json::Value) -> BTreeMap<u64, (String, Option<u64>)> {
    let flattened = flattened_nodes(nodes);
    let offset_to_part = flattened
        .iter()
        .map(|node| {
            (
                node["offset"].as_u64().expect("node offset"),
                node["number"].as_u64().expect("node number"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    flattened
        .into_iter()
        .map(|node| {
            let number = node["number"].as_u64().expect("node number");
            let name = node["name"].as_str().expect("node name").to_owned();
            let parent = node["parentOffset"]
                .as_u64()
                .map(|offset| offset_to_part[&offset]);
            (number, (name, parent))
        })
        .collect()
}

#[derive(Debug, PartialEq)]
struct RigidSurface {
    name: String,
    parent_part: u64,
    vertices: serde_json::Value,
    normals: serde_json::Value,
    uv0: serde_json::Value,
    vertex_indices: serde_json::Value,
}

fn rigid_mesh_surface(nodes: &serde_json::Value) -> Vec<RigidSurface> {
    let flattened = flattened_nodes(nodes);
    let offset_to_part = flattened
        .iter()
        .map(|node| {
            (
                node["offset"].as_u64().expect("node offset"),
                node["number"].as_u64().expect("node number"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    flattened
        .into_iter()
        .filter_map(|node| {
            node["mesh"].as_object().map(|mesh| RigidSurface {
                name: node["name"].as_str().expect("mesh name").to_owned(),
                parent_part: offset_to_part
                    [&node["parentOffset"].as_u64().expect("mesh parent offset")],
                vertices: mesh["vertices"].clone(),
                normals: mesh["normals"].clone(),
                uv0: mesh["uv0"].clone(),
                vertex_indices: serde_json::Value::Array(
                    mesh["faces"]
                        .as_array()
                        .expect("mesh faces")
                        .iter()
                        .map(|face| face["vertexIndices"].clone())
                        .collect(),
                ),
            })
        })
        .collect()
}
