use m2a_core::glb::{GlbLimits, ingest_glb};
use m2a_core::{
    direct_creature_contract::{
        SourceTopologyOriginV1, declare_m0_direct_creature_runtime_profile_v2,
        inspect_m0_source_topology_binding_v1,
    },
    hierarchy_experiment::{
        SOURCE_TOPOLOGY_RIGID_EXPERIMENT_PROFILE_V3, build_source_topology_rigid_experiment_v3,
        declare_source_topology_rigid_experiment_profile_v3,
        verify_source_topology_rigid_experiment_v3,
    },
    owned_fixture::synthetic_owned_m0_hierarchy_experiment_glb_v1,
};
use serde_json::Value;

#[test]
fn hierarchy_experiment_maps_owned_source_and_preserves_raw_mdx() {
    let source = synthetic_owned_m0_hierarchy_experiment_glb_v1().expect("owned hierarchy GLB");
    let ingest = ingest_glb(&source, &GlbLimits::default()).expect("owned source ingest");
    assert_eq!(
        ingest
            .ir
            .nodes
            .iter()
            .map(|node| (node.id, node.parent_ids.clone(), node.child_ids.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0, vec![], vec![1]),
            (1, vec![0], vec![2]),
            (2, vec![1], vec![]),
        ]
    );
    let profile = declare_source_topology_rigid_experiment_profile_v3(
        &source,
        "owned://synthetic/m0-hierarchy-v1.glb",
        SourceTopologyOriginV1::UserDerived,
        "m2a_hexp01",
    )
    .expect("explicit caller-owned hierarchy profile");
    assert_eq!(profile.profile, SOURCE_TOPOLOGY_RIGID_EXPERIMENT_PROFILE_V3);

    let artifact = build_source_topology_rigid_experiment_v3(&source, &profile)
        .expect("controlled hierarchy experiment");
    assert!(!artifact.contract.runtime_admissible);
    assert_eq!(
        profile.source_canonical_identity,
        format!("sha256:{}", profile.source_sha256)
    );
    assert_eq!(artifact.contract.source_geometry_sha256.len(), 64);
    assert_eq!(artifact.contract.source_to_output_nodes.len(), 3);
    assert_eq!(artifact.contract.mesh_attachment.source_node_id, 2);
    assert_eq!(artifact.contract.mesh_attachment.output_parent_part, 2);
    assert_eq!(
        artifact.contract.hierarchy_raw_mdx_sha256, artifact.contract.flat_control_raw_mdx_sha256,
        "hierarchy-only experiment must preserve exact raw MDX bytes"
    );
    assert_eq!(
        artifact.contract.hierarchy_protected_writer_fields_sha256,
        artifact
            .contract
            .flat_control_protected_writer_fields_sha256,
        "the controlled delta must be topology-only"
    );
    verify_source_topology_rigid_experiment_v3(
        &artifact.contract,
        &profile,
        &source,
        &artifact.hierarchy_mdl.payload,
    )
    .expect("independent source/model replay");

    let historical = declare_m0_direct_creature_runtime_profile_v2(
        &source,
        "owned://synthetic/m0-hierarchy-v1.glb",
        SourceTopologyOriginV1::UserDerived,
        "m2a_hexp01",
    )
    .expect("historical profile declaration is only a caller trust root");
    let error = inspect_m0_source_topology_binding_v1(
        &source,
        &artifact.hierarchy_mdl.payload,
        &historical,
    )
    .expect_err("historical flat M0/V2 must reject nested source topology");
    assert_eq!(error.code, "M2A-SOURCE-TOPOLOGY-M0-SHAPE");
}

#[test]
fn hierarchy_experiment_rejects_source_inventory_order_names_and_transforms() {
    let source = synthetic_owned_m0_hierarchy_experiment_glb_v1().expect("owned hierarchy GLB");

    let extra_scene = rewrite_glb_json(&source, |root| {
        root["scenes"]
            .as_array_mut()
            .expect("scenes")
            .push(serde_json::json!({"name": "second", "nodes": [0]}));
    });
    assert_source_rejected(&extra_scene, "M2A-HIERARCHY-EXPERIMENT-SCENE");

    let extra_root = rewrite_glb_json(&source, |root| {
        root["scenes"][0]["nodes"] = serde_json::json!([0, 1]);
    });
    assert_source_rejected(&extra_root, "M2A-HIERARCHY-EXPERIMENT-SCENE");

    let ignored_node = rewrite_glb_json(&source, |root| {
        root["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(serde_json::json!({"name": "ignored_node"}));
    });
    assert_source_rejected(&ignored_node, "M2A-HIERARCHY-EXPERIMENT-ORDER");

    let source_order_drift = rewrite_glb_json(&source, |root| {
        root["nodes"].as_array_mut().expect("nodes").swap(0, 1);
    });
    assert_source_rejected(&source_order_drift, "M2A-GLB-NODE-CYCLE");

    let duplicate_name = rewrite_glb_json(&source, |root| {
        root["nodes"][1]["name"] = serde_json::json!("owned_root");
    });
    assert_source_rejected(&duplicate_name, "M2A-HIERARCHY-EXPERIMENT-NAME");

    let transform_drift = rewrite_glb_json(&source, |root| {
        root["nodes"][1]["translation"] = serde_json::json!([0.25, 0.0, 0.0]);
    });
    assert_source_rejected(&transform_drift, "M2A-HIERARCHY-EXPERIMENT-TRANSFORM");

    let second_mesh_attachment = rewrite_glb_json(&source, |root| {
        root["nodes"][1]["mesh"] = serde_json::json!(0);
    });
    assert_source_rejected(
        &second_mesh_attachment,
        "M2A-HIERARCHY-EXPERIMENT-INVENTORY",
    );

    let second_primitive = rewrite_glb_json(&source, |root| {
        let duplicate = root["meshes"][0]["primitives"][0].clone();
        root["meshes"][0]["primitives"]
            .as_array_mut()
            .expect("primitives")
            .push(duplicate);
    });
    assert_source_rejected(&second_primitive, "M2A-HIERARCHY-EXPERIMENT-INVENTORY");

    let skin = rewrite_glb_json(&source, |root| {
        root["skins"] = serde_json::json!([{"joints": [0]}]);
        root["nodes"][2]["skin"] = serde_json::json!(0);
    });
    assert_source_rejected(&skin, "M2A-HIERARCHY-EXPERIMENT-INVENTORY");

    let source_animation = rewrite_glb_json(&source, |root| {
        root["animations"] = serde_json::json!([{
            "name": "forbidden-source-animation",
            "samplers": [],
            "channels": []
        }]);
    });
    assert_source_rejected(&source_animation, "M2A-HIERARCHY-EXPERIMENT-INVENTORY");

    let missing_uv = rewrite_glb_json(&source, |root| {
        root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .expect("attributes")
            .remove("TEXCOORD_0");
    });
    assert_source_rejected(&missing_uv, "M2A-HIERARCHY-EXPERIMENT-GEOMETRY");
}

#[test]
fn hierarchy_experiment_nullable_names_are_order_derived_without_invented_joints() {
    let source = synthetic_owned_m0_hierarchy_experiment_glb_v1().expect("owned hierarchy GLB");
    let two_null_names = rewrite_glb_json(&source, |root| {
        root["nodes"][2]
            .as_object_mut()
            .expect("mesh node")
            .remove("name");
    });
    let profile = experiment_profile(&two_null_names);
    let artifact = build_source_topology_rigid_experiment_v3(&two_null_names, &profile)
        .expect("two nullable names are resolved by the explicit source-order rule");
    assert_eq!(
        artifact
            .contract
            .source_to_output_nodes
            .iter()
            .map(|mapping| mapping.output_name.as_str())
            .collect::<Vec<_>>(),
        ["m2a_hexp01", "m2a_src_1", "m2a_src_2"]
    );
    assert_eq!(artifact.contract.source_to_output_nodes.len(), 3);
    assert_eq!(artifact.contract.mesh_attachment.output_parent_part, 2);
}

#[test]
fn hierarchy_experiment_rejects_self_consistent_source_drift_and_profile_mixing() {
    let source = synthetic_owned_m0_hierarchy_experiment_glb_v1().expect("owned hierarchy GLB");
    let expected = experiment_profile(&source);
    let original = build_source_topology_rigid_experiment_v3(&source, &expected)
        .expect("original hierarchy experiment");

    let renamed = rewrite_glb_json(&source, |root| {
        root["nodes"][1]["name"] = serde_json::json!("caller_renamed_dummy");
    });
    let renamed_profile = experiment_profile(&renamed);
    let renamed_artifact = build_source_topology_rigid_experiment_v3(&renamed, &renamed_profile)
        .expect("independently self-consistent renamed experiment");
    let error = verify_source_topology_rigid_experiment_v3(
        &renamed_artifact.contract,
        &expected,
        &renamed,
        &renamed_artifact.hierarchy_mdl.payload,
    )
    .expect_err("self-consistent source replacement must not replace expected caller trust root");
    assert_eq!(error.code, "M2A-HIERARCHY-EXPERIMENT-SOURCE-TRUST-ROOT");

    let reparented = rewrite_glb_json(&source, |root| {
        root["nodes"][0]["children"] = serde_json::json!([1, 2]);
        root["nodes"][1]["children"] = serde_json::json!([]);
    });
    let reparented_profile = experiment_profile(&reparented);
    let reparented_artifact =
        build_source_topology_rigid_experiment_v3(&reparented, &reparented_profile)
            .expect("self-consistent alternate mesh parent");
    assert_eq!(
        reparented_artifact.contract.source_to_output_nodes[2].output_parent_part,
        Some(0),
        "source mesh transform itself remains the mesh attachment dummy, while its source-derived parent changes"
    );
    assert_eq!(
        reparented_artifact
            .contract
            .mesh_attachment
            .output_parent_part,
        2
    );
    let error = verify_source_topology_rigid_experiment_v3(
        &original.contract,
        &expected,
        &reparented,
        &reparented_artifact.hierarchy_mdl.payload,
    )
    .expect_err("recomputed alternate attachment cannot satisfy original source binding");
    assert_eq!(error.code, "M2A-HIERARCHY-EXPERIMENT-SOURCE-TRUST-ROOT");

    let mut mixed = expected.clone();
    mixed.state_projection_profile = "CEP_RIGID_PLACEHOLDER_V1".to_owned();
    let error = build_source_topology_rigid_experiment_v3(&source, &mixed)
        .expect_err("mixed state family must be rejected before build");
    assert_eq!(error.code, "M2A-HIERARCHY-EXPERIMENT-PROFILE-VERSION");
}

#[test]
fn hierarchy_experiment_rejects_actual_topology_state_controller_and_payload_corruption() {
    let source = synthetic_owned_m0_hierarchy_experiment_glb_v1().expect("owned hierarchy GLB");
    let profile = experiment_profile(&source);
    let artifact = build_source_topology_rigid_experiment_v3(&source, &profile)
        .expect("controlled hierarchy experiment");
    let core_start = artifact
        .hierarchy_mdl
        .inspection
        .file_header
        .core_range
        .start;

    let mut wrong_dummy_part = artifact.hierarchy_mdl.payload.clone();
    let dummy = &artifact.hierarchy_mdl.report.layout.rig_nodes[1];
    write_u32(
        &mut wrong_dummy_part,
        core_start + dummy.core_offset as usize + 0x1c,
        99,
    );
    assert_candidate_rejected(&artifact.contract, &profile, &source, &wrong_dummy_part);

    let mut omitted_subtree = artifact.hierarchy_mdl.payload.clone();
    let root = &artifact.hierarchy_mdl.inspection.node_tree.roots[0];
    write_u32(
        &mut omitted_subtree,
        core_start + root.offset as usize + 0x4c,
        0,
    );
    write_u32(
        &mut omitted_subtree,
        core_start + root.offset as usize + 0x50,
        0,
    );
    assert_candidate_rejected(&artifact.contract, &profile, &source, &omitted_subtree);

    let node_one = &root.children[0];
    let node_two = &node_one.children[0];
    let mesh_node = &node_two.children[0];
    let mut reordered_and_wrong_mesh_parent = artifact.hierarchy_mdl.payload.clone();
    write_u32(
        &mut reordered_and_wrong_mesh_parent,
        core_start + root.children_header.pointer as usize,
        node_two.offset,
    );
    write_u32(
        &mut reordered_and_wrong_mesh_parent,
        core_start + node_two.children_header.pointer as usize,
        node_one.offset,
    );
    write_u32(
        &mut reordered_and_wrong_mesh_parent,
        core_start + node_one.children_header.pointer as usize,
        mesh_node.offset,
    );
    assert_candidate_rejected(
        &artifact.contract,
        &profile,
        &source,
        &reordered_and_wrong_mesh_parent,
    );

    let animation_report = artifact
        .hierarchy_mdl
        .report
        .animation
        .as_ref()
        .expect("animation writer report");
    let first_clip = &animation_report.clips[0];
    let mut wrong_animation_type = artifact.hierarchy_mdl.payload.clone();
    wrong_animation_type[core_start + first_clip.header_core_offset as usize + 0x6c] = 0;
    assert_candidate_rejected(&artifact.contract, &profile, &source, &wrong_animation_type);

    let state_root = &first_clip.nodes[0];
    let mut cep_state = artifact.hierarchy_mdl.payload.clone();
    write_u32(
        &mut cep_state,
        core_start + state_root.core_offset as usize + 0x6c,
        0x21,
    );
    assert_candidate_rejected(&artifact.contract, &profile, &source, &cep_state);

    let base_root = &artifact.hierarchy_mdl.inspection.node_tree.roots[0];
    let mut controller_injected = artifact.hierarchy_mdl.payload.clone();
    write_array_header(
        &mut controller_injected,
        core_start + state_root.core_offset as usize + 0x54,
        base_root.controller_keys_header.pointer,
        base_root.controller_keys_header.used as u32,
        base_root.controller_keys_header.allocated as u32,
    );
    write_array_header(
        &mut controller_injected,
        core_start + state_root.core_offset as usize + 0x60,
        base_root.controller_data_header.pointer,
        base_root.controller_data_header.used as u32,
        base_root.controller_data_header.allocated as u32,
    );
    assert_candidate_rejected(&artifact.contract, &profile, &source, &controller_injected);

    let mesh = &artifact.hierarchy_mdl.report.layout.mesh_nodes[0];
    let mut protected_render_field = artifact.hierarchy_mdl.payload.clone();
    write_u32(
        &mut protected_render_field,
        core_start + mesh.core_offset as usize + 0xdc,
        0,
    );
    assert_candidate_rejected(
        &artifact.contract,
        &profile,
        &source,
        &protected_render_field,
    );

    let raw_start = artifact
        .hierarchy_mdl
        .inspection
        .file_header
        .raw_range
        .start;
    let mut raw_mdx_mutation = artifact.hierarchy_mdl.payload.clone();
    raw_mdx_mutation[raw_start] ^= 1;
    assert_candidate_rejected(&artifact.contract, &profile, &source, &raw_mdx_mutation);
}

#[test]
fn hierarchy_experiment_raw_partition_rejects_gap_and_unassessed_tail_before_replay() {
    let source = synthetic_owned_m0_hierarchy_experiment_glb_v1().expect("owned hierarchy GLB");
    let profile = experiment_profile(&source);
    let artifact = build_source_topology_rigid_experiment_v3(&source, &profile)
        .expect("controlled hierarchy experiment");

    let prefixed_gap = prefixed_raw_gap_with_rebased_streams(&artifact.hierarchy_mdl, 1);
    let error = verify_source_topology_rigid_experiment_v3(
        &artifact.contract,
        &profile,
        &source,
        &prefixed_gap,
    )
    .expect_err("rebased real streams must not make an unassessed raw prefix admissible");
    assert_eq!(error.code, "M2A-MDL-ENGINE-ENVELOPE-RAW-GAP");
    assert_ne!(error.code, "M2A-HIERARCHY-EXPERIMENT-REPLAY-MISMATCH");

    let nonzero_tail = append_raw_tail(&artifact.hierarchy_mdl.payload, &[0xa5]);
    let error = verify_source_topology_rigid_experiment_v3(
        &artifact.contract,
        &profile,
        &source,
        &nonzero_tail,
    )
    .expect_err("non-zero alignment tail must be rejected before replay");
    assert_eq!(error.code, "M2A-MDL-ENGINE-ENVELOPE-RAW-TAIL-NONZERO");
    assert_ne!(error.code, "M2A-HIERARCHY-EXPERIMENT-REPLAY-MISMATCH");

    let long_zero_tail = append_raw_tail(&artifact.hierarchy_mdl.payload, &[0; 4]);
    let error = verify_source_topology_rigid_experiment_v3(
        &artifact.contract,
        &profile,
        &source,
        &long_zero_tail,
    )
    .expect_err("more than three alignment bytes must be rejected before replay");
    assert_eq!(error.code, "M2A-MDL-ENGINE-ENVELOPE-RAW-TAIL-LENGTH");
    assert_ne!(error.code, "M2A-HIERARCHY-EXPERIMENT-REPLAY-MISMATCH");
}

#[test]
fn hierarchy_experiment_production_module_has_no_witness_locator_or_runtime_materializer() {
    let source = include_str!("../src/hierarchy_experiment.rs");
    for forbidden in [
        "local-reference-assets",
        "models_01.bif",
        "c_horror",
        "c_Direwolf",
        "proof-output",
        "build_erf",
        "build_hak",
        "build_mod",
        "nwtoolset",
        "nwmain",
    ] {
        assert!(
            !source.contains(forbidden),
            "offline hierarchy production module must not contain witness/runtime locator {forbidden}"
        );
    }
}

fn experiment_profile(
    source: &[u8],
) -> m2a_core::hierarchy_experiment::SourceTopologyRigidExperimentProfileV3 {
    declare_source_topology_rigid_experiment_profile_v3(
        source,
        "owned://synthetic/m0-hierarchy-v1.glb",
        SourceTopologyOriginV1::UserDerived,
        "m2a_hexp01",
    )
    .expect("explicit caller-owned hierarchy profile")
}

fn assert_source_rejected(source: &[u8], expected_code: &str) {
    let profile = experiment_profile(source);
    let error = build_source_topology_rigid_experiment_v3(source, &profile)
        .expect_err("invalid hierarchy source must be rejected");
    assert_eq!(error.code, expected_code, "unexpected error: {error}");
}

fn assert_candidate_rejected(
    contract: &m2a_core::hierarchy_experiment::SourceTopologyRigidExperimentContractV3,
    profile: &m2a_core::hierarchy_experiment::SourceTopologyRigidExperimentProfileV3,
    source: &[u8],
    candidate: &[u8],
) {
    verify_source_topology_rigid_experiment_v3(contract, profile, source, candidate)
        .expect_err("actual-byte candidate corruption must be rejected");
}

fn rewrite_glb_json(source: &[u8], edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    assert_eq!(&source[0..4], b"glTF");
    let json_len = u32::from_le_bytes(source[12..16].try_into().expect("JSON length")) as usize;
    let json_start = 20;
    let json_end = json_start + json_len;
    let mut root: Value =
        serde_json::from_slice(&source[json_start..json_end]).expect("owned fixture JSON chunk");
    edit(&mut root);
    let mut json = serde_json::to_vec(&root).expect("rewritten JSON");
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let bin_chunk = &source[json_end..];
    let total = 12 + 8 + json.len() + bin_chunk.len();
    let mut output = Vec::with_capacity(total);
    output.extend_from_slice(b"glTF");
    output.extend_from_slice(&2_u32.to_le_bytes());
    output.extend_from_slice(&(total as u32).to_le_bytes());
    output.extend_from_slice(&(json.len() as u32).to_le_bytes());
    output.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    output.extend_from_slice(&json);
    output.extend_from_slice(bin_chunk);
    output
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 field"))
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn prefixed_raw_gap_with_rebased_streams(
    artifact: &m2a_core::BinaryMdlArtifactV1,
    gap_length: usize,
) -> Vec<u8> {
    assert!(gap_length > 0);
    let core_start = artifact.inspection.file_header.core_range.start;
    let raw_start = artifact.inspection.file_header.raw_range.start;
    let mesh = artifact.report.layout.mesh_nodes[0].core_offset as usize;
    let mut output = artifact.payload.clone();
    output.splice(raw_start..raw_start, std::iter::repeat_n(0, gap_length));
    let raw_length = read_u32(&output, 8);
    write_u32(
        &mut output,
        8,
        raw_length + u32::try_from(gap_length).expect("bounded test gap"),
    );
    for field in [0x22c, 0x234, 0x244, 0x248] {
        let offset = core_start + mesh + field;
        let pointer = i32::from_le_bytes(
            output[offset..offset + 4]
                .try_into()
                .expect("mesh raw pointer"),
        );
        if pointer >= 0 {
            write_i32(
                &mut output,
                offset,
                pointer + i32::try_from(gap_length).expect("bounded test gap"),
            );
        }
    }
    let index_offsets = read_u32(&output, core_start + mesh + 0x210) as usize;
    let index_offset_count = read_u32(&output, core_start + mesh + 0x214) as usize;
    for index in 0..index_offset_count {
        let offset = core_start + index_offsets + index * 4;
        let pointer = i32::from_le_bytes(
            output[offset..offset + 4]
                .try_into()
                .expect("raw index pointer"),
        );
        write_i32(
            &mut output,
            offset,
            pointer + i32::try_from(gap_length).expect("bounded test gap"),
        );
    }
    output
}

fn append_raw_tail(mdl: &[u8], tail: &[u8]) -> Vec<u8> {
    let mut output = mdl.to_vec();
    output.extend_from_slice(tail);
    let raw_length = read_u32(&output, 8);
    write_u32(
        &mut output,
        8,
        raw_length + u32::try_from(tail.len()).expect("bounded test tail"),
    );
    output
}

fn write_array_header(bytes: &mut [u8], offset: usize, pointer: u32, used: u32, allocated: u32) {
    write_u32(bytes, offset, pointer);
    write_u32(bytes, offset + 4, used);
    write_u32(bytes, offset + 8, allocated);
}
