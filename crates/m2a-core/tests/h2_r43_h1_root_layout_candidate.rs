use std::fs;

use m2a_core::{
    animated_donor_candidate::{
        H2_R43_APPEARANCE_ROW, H2_R43_APPEARANCE_SHA256, H2_R43_AREA_RESREF,
        H2_R43_CONTRACT_SHA256, H2_R43_HAK_RESREF, H2_R43_HAK_SHA256, H2_R43_MODEL_RESREF,
        H2_R43_MODEL_SHA256, H2_R43_MODULE_RESREF, H2_R43_MODULE_SHA256, H2_R43_SOURCE_SHA256,
        H2_R43_STOCK_CONTROL_APPEARANCE_ROW, H2_R43_TEXTURE_RESREF, H2_R43_TEXTURE_SHA256,
        build_h2_r43_h1_root_layout_candidate_v1, verify_h2_r43_h1_root_layout_candidate_v1,
    },
    erf::ErfArchive,
    mdl::{MdlFormatProfileV1, MdlStateProjectionProfileV1, inspect_binary_mdl},
    proof_module::inspect_binary_creature_multi_fixture_module_v1,
};
use sha2::{Digest, Sha256};

#[path = "support/canonical_workspace.rs"]
mod canonical_workspace;

#[test]
fn exact_h2_r43_replays_the_corrupt_draw_h1_v20_root_and_rigid_mesh_shape() {
    let repo = canonical_workspace::canonical_repository_root();
    let source = fs::read(repo.join("sample-3d/h2-clockwork-sentinel-1500/source.glb"))
        .expect("exact generated Meshy H2 source");
    let appearance = fs::read(repo.join("local-reference-assets/appearance.2da"))
        .expect("exact full runtime appearance table");
    assert_eq!(sha256(&source), H2_R43_SOURCE_SHA256);

    let first = build_h2_r43_h1_root_layout_candidate_v1(&source, &appearance).expect("exact r43");
    let replay =
        build_h2_r43_h1_root_layout_candidate_v1(&source, &appearance).expect("r43 replay");
    assert_eq!(first.model, replay.model);
    assert_eq!(first.texture, replay.texture);
    assert_eq!(first.appearance_two_da, replay.appearance_two_da);
    assert_eq!(first.hak, replay.hak);
    assert_eq!(first.module, replay.module);
    assert_eq!(first.contract, replay.contract);
    assert_eq!(first.contract_json, replay.contract_json);
    assert_eq!(sha256(&first.model), H2_R43_MODEL_SHA256);
    assert_eq!(sha256(&first.texture), H2_R43_TEXTURE_SHA256);
    assert_eq!(sha256(&first.appearance_two_da), H2_R43_APPEARANCE_SHA256);
    assert_eq!(sha256(&first.hak), H2_R43_HAK_SHA256);
    assert_eq!(sha256(&first.module), H2_R43_MODULE_SHA256);
    assert_eq!(sha256(&first.contract_json), H2_R43_CONTRACT_SHA256);
    verify_h2_r43_h1_root_layout_candidate_v1(
        &first.contract,
        &source,
        &appearance,
        &first.module,
        &first.hak,
    )
    .expect("exact replay verification");

    assert_eq!(
        first.contract.profile,
        "H2_R43_H1_V20_ROOT_RIGID_TYPE0_CANDIDATE_V1"
    );
    assert_eq!(
        first.contract.format_profile,
        MdlFormatProfileV1::M4DirectCreatureExtended64V1
    );
    assert_eq!(
        first.contract.state_projection_profile,
        MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1
    );
    assert_eq!(first.contract.appearance_row, H2_R43_APPEARANCE_ROW);
    assert_eq!(first.contract.triangle_count, 1_543);
    assert_eq!(first.contract.rig_node_count, 24);
    assert_eq!(first.contract.rigid_mesh_node_count, 1);
    assert_eq!(first.contract.skin_node_count, 0);
    assert_eq!(first.contract.base_root_controller_count, 2);

    let model = inspect_binary_mdl(&first.model).expect("r43 binary MDL readback");
    assert_eq!(model.model.name, H2_R43_MODEL_RESREF);
    assert_eq!(model.node_tree.node_count, 25);
    assert_eq!(model.node_tree.roots.len(), 1);
    let root = &model.node_tree.roots[0];
    assert_eq!(root.name, H2_R43_MODEL_RESREF);
    assert_eq!(root.controllers.len(), 2);

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
            .expect("single rigid H2 mesh")
            .faces
            .len(),
        1_543
    );

    assert_eq!(model.animations.len(), 7);
    assert!(model.animations.iter().all(|animation| {
        animation.animation_type == 0
            && animation.animation_type_padding == [0, 0, 0]
            && animation.node_tree.node_count == 24
            && animation
                .node_tree
                .roots
                .iter()
                .all(|state_root| state_root.name == H2_R43_MODEL_RESREF)
    }));

    let hak = ErfArchive::parse(&first.hak).expect("r43 HAK");
    assert_eq!(hak.find(H2_R43_MODEL_RESREF, 2002).unwrap(), first.model);
    assert_eq!(hak.find(H2_R43_TEXTURE_RESREF, 3).unwrap(), first.texture);
    assert_eq!(
        hak.find("appearance", 2017).unwrap(),
        first.appearance_two_da
    );

    let scene =
        inspect_binary_creature_multi_fixture_module_v1(&first.module).expect("r43 full MOD");
    assert_eq!(scene.module_resref, H2_R43_MODULE_RESREF);
    assert_eq!(scene.area_resref, H2_R43_AREA_RESREF);
    assert_eq!(scene.ordered_hak_resrefs, [H2_R43_HAK_RESREF]);
    assert_eq!(scene.fixtures.len(), 2);
    assert_eq!(scene.fixtures[0].appearance_row, H2_R43_APPEARANCE_ROW);
    assert_eq!(
        scene.fixtures[1].appearance_row,
        H2_R43_STOCK_CONTROL_APPEARANCE_ROW
    );
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
