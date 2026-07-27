use std::{fs, path::PathBuf};

use m2a_core::{
    animated_donor_candidate::{
        H2_R42_APPEARANCE_ROW, H2_R42_APPEARANCE_SHA256, H2_R42_AREA_RESREF,
        H2_R42_CONTRACT_SHA256, H2_R42_HAK_RESREF, H2_R42_HAK_SHA256, H2_R42_MODEL_RESREF,
        H2_R42_MODEL_SHA256, H2_R42_MODULE_RESREF, H2_R42_MODULE_SHA256, H2_R42_SOURCE_SHA256,
        H2_R42_STOCK_CONTROL_APPEARANCE_ROW, H2_R42_TEXTURE_RESREF, H2_R42_TEXTURE_SHA256,
        build_h2_r42_visibility_candidate_v1, verify_h2_r42_visibility_candidate_v1,
    },
    erf::ErfArchive,
    mdl::inspect_binary_mdl,
    proof_module::inspect_binary_creature_multi_fixture_module_v1,
};
use sha2::{Digest, Sha256};

#[test]
fn exact_h2_r42_uses_the_owned_runtime_positive_type0_root_rigid_family() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf();
    let source = fs::read(repo.join("sample-3d/h2-clockwork-sentinel-1500/source.glb"))
        .expect("exact generated Meshy H2 source");
    let appearance = fs::read(repo.join("local-reference-assets/appearance.2da"))
        .expect("exact full runtime appearance table");
    assert_eq!(sha256(&source), H2_R42_SOURCE_SHA256);

    let first =
        build_h2_r42_visibility_candidate_v1(&source, &appearance).expect("exact r42 candidate");
    let repeated =
        build_h2_r42_visibility_candidate_v1(&source, &appearance).expect("deterministic replay");
    assert_eq!(first.model, repeated.model);
    assert_eq!(first.texture, repeated.texture);
    assert_eq!(first.appearance_two_da, repeated.appearance_two_da);
    assert_eq!(first.hak, repeated.hak);
    assert_eq!(first.module, repeated.module);
    assert_eq!(first.contract, repeated.contract);
    assert_eq!(first.contract_json, repeated.contract_json);
    verify_h2_r42_visibility_candidate_v1(
        &first.contract,
        &source,
        &appearance,
        &first.module,
        &first.hak,
    )
    .expect("exact replay verification");

    assert_eq!(first.contract.source.sha256, H2_R42_SOURCE_SHA256);
    assert_eq!(sha256(&first.model), H2_R42_MODEL_SHA256);
    assert_eq!(sha256(&first.texture), H2_R42_TEXTURE_SHA256);
    assert_eq!(sha256(&first.appearance_two_da), H2_R42_APPEARANCE_SHA256);
    assert_eq!(sha256(&first.hak), H2_R42_HAK_SHA256);
    assert_eq!(sha256(&first.module), H2_R42_MODULE_SHA256);
    assert_eq!(sha256(&first.contract_json), H2_R42_CONTRACT_SHA256);
    assert_eq!(
        first.contract.profile,
        "H2_R42_OWNED_TYPE0_ROOT_RIGID_VISIBILITY_CANDIDATE_V1"
    );
    assert_eq!(first.contract.model.resref, H2_R42_MODEL_RESREF);
    assert_eq!(first.contract.texture.resref, H2_R42_TEXTURE_RESREF);
    assert_eq!(first.contract.appearance_row, H2_R42_APPEARANCE_ROW);
    assert_eq!(first.contract.triangle_count, 1_543);
    assert_eq!(first.contract.rigid_mesh_node_count, 1);
    assert_eq!(first.contract.skin_node_count, 0);

    let model = inspect_binary_mdl(&first.model).expect("r42 binary MDL readback");
    assert_eq!(model.model.name, H2_R42_MODEL_RESREF);
    assert_eq!(model.node_tree.roots.len(), 1);
    let root = &model.node_tree.roots[0];
    assert_eq!(root.name, H2_R42_MODEL_RESREF);
    assert!(root.controllers.is_empty());

    let mut mesh_nodes = Vec::new();
    let mut skin_count = 0;
    let mut pending = model.node_tree.roots.iter().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        if node.mesh.is_some() {
            mesh_nodes.push(node);
        }
        skin_count += usize::from(node.skin.is_some());
        pending.extend(&node.children);
    }
    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(skin_count, 0);
    assert_eq!(mesh_nodes[0].parent_offset, Some(root.offset));
    assert_eq!(
        mesh_nodes[0]
            .mesh
            .as_ref()
            .expect("root rigid mesh")
            .faces
            .len(),
        1_543
    );
    assert_eq!(model.animations.len(), 7);
    assert!(model.animations.iter().all(|animation| {
        animation.animation_type == 0
            && animation.animation_type_padding == [0, 0, 0]
            && animation.node_tree.node_count + 1 == model.node_tree.node_count
            && animation
                .node_tree
                .roots
                .iter()
                .all(|state_root| state_root.name == H2_R42_MODEL_RESREF)
    }));

    let hak = ErfArchive::parse(&first.hak).expect("r42 HAK");
    assert_eq!(hak.find(H2_R42_MODEL_RESREF, 2002).unwrap(), first.model);
    assert_eq!(hak.find(H2_R42_TEXTURE_RESREF, 3).unwrap(), first.texture);
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        first.appearance_two_da
    );

    let scene =
        inspect_binary_creature_multi_fixture_module_v1(&first.module).expect("r42 complete MOD");
    assert_eq!(scene.module_resref, H2_R42_MODULE_RESREF);
    assert_eq!(scene.area_resref, H2_R42_AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [H2_R42_HAK_RESREF]);
    assert_eq!(scene.fixtures.len(), 2);
    assert_eq!(scene.fixtures[0].appearance_row, H2_R42_APPEARANCE_ROW);
    assert_eq!(
        scene.fixtures[1].appearance_row,
        H2_R42_STOCK_CONTROL_APPEARANCE_ROW
    );
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
