#[path = "fixtures/build_synthetic_glb.rs"]
mod fixtures;

use m2a_core::{
    aurora_material::{AuroraMaterialCompileStatusV1, AuroraMaterialTargetProfileV1},
    creature_product::{CreatureAlphaModeV2, CreatureMaterialProfileV2, CreatureMaterialTargetV2},
    erf::ErfArchive,
    glb::{GlbLimits, ingest_glb},
    hak::HakWriterOptionsV1,
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
        MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlFormatProfileV1,
        MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1, MdlWriterOptionsV1, NodeReport,
        inspect_binary_mdl, write_binary_mdl_with_animations,
    },
    model_ir::{
        AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1,
    },
    mtr::{MTR_RESOURCE_TYPE_V1, MtrRenderHintV1, parse_mtr_v1},
    profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        CreatureSourceForwardV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_profile_sha256,
    },
    reference_supermodel_motion::{
        CreatureMaterialSemanticStatusV1, ReferenceSupermodelCarrierClassV3,
        ReferenceSupermodelClassicMaterialOptionsV1, ReferenceSupermodelExactContractOptionsV3,
        ReferenceSupermodelMaterialOptionsV1, ReferenceSupermodelMinimalMtrMaterialOptionsV1,
        ReferenceSupermodelMotionContractV2, ReferenceSupermodelMotionNodeV2,
        ReferenceSupermodelRejectedBaselineV1, ReferenceSupermodelRequiredEventV2,
        VisibleAnchorTrajectorySampleV1,
        bind_static_mesh_for_inherited_supermodel_motion_direct_classic_v1,
        bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_corrected_v2,
        bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_v1,
        bind_static_mesh_for_inherited_supermodel_motion_v1,
        build_creature_material_semantic_report_v1, build_exact_motion_carrier_rig_v3,
        build_exact_reference_supermodel_motion_contract_v3, build_motion_corrected_rig_v1,
        build_reference_supermodel_creature_package_v1, build_retargeted_motion_carrier_rig_v4,
        canonical_reference_supermodel_motion_contract_sha256_v2,
        default_reference_supermodel_motion_tolerances_v2,
        default_reference_supermodel_writer_options_v1,
        evaluate_reference_supermodel_semantic_delta_v1, evaluate_surface_seam_gate_v1,
        evaluate_visible_anchor_trajectory_v1,
        package_reference_supermodel_classic_creature_hak_v1,
        package_reference_supermodel_creature_hak_v1,
        package_reference_supermodel_minimal_twosided_creature_hak_v1,
        retarget_reference_supermodel_animation_set_v5,
        validate_reference_supermodel_motion_oracle_v2,
        visible_controller_surface_semantic_sha256_v1,
    },
};
use sha2::{Digest, Sha256};

#[test]
fn rejected_v8_visible_tail_surface_identity_is_env_gated() {
    let Some(path) = std::env::var_os("M2A_REJECTED_CWOLF_V8_MDL") else {
        eprintln!("skipped: M2A_REJECTED_CWOLF_V8_MDL is not configured");
        return;
    };
    let bytes = std::fs::read(path).expect("read exact rejected V8 MDL");
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f",
    );
    let inspection = inspect_binary_mdl(&bytes).expect("inspect exact rejected V8 MDL");
    let signature = visible_controller_surface_semantic_sha256_v1(
        &inspection,
        &["Wolf_tail".to_owned(), "Wolf_tailend".to_owned()],
    )
    .expect("hash exact visible tail surface semantics");
    eprintln!("rejected-v8-visible-tail-surface-sha256={signature}");
}

#[test]
fn visible_tail_surface_must_follow_controller_amplitude_and_trajectory() {
    let zero_length_clip = evaluate_visible_anchor_trajectory_v1(
        "tail_base",
        "cpause1",
        &[VisibleAnchorTrajectorySampleV1 {
            time_seconds: 0.0,
            surface_centroid: [0.0; 3],
            controller_centroid: [0.0; 3],
        }],
        0.25,
        0.5,
    )
    .unwrap();
    assert_eq!(zero_length_clip.status, "BLOCKED_CONTROLLER_STATIC");

    let stationary = [
        VisibleAnchorTrajectorySampleV1 {
            time_seconds: 0.0,
            surface_centroid: [0.0, 0.0, 0.0],
            controller_centroid: [0.0, 0.0, 0.0],
        },
        VisibleAnchorTrajectorySampleV1 {
            time_seconds: 0.5,
            surface_centroid: [0.0, 0.0, 0.0],
            controller_centroid: [0.0, 0.5, 0.0],
        },
        VisibleAnchorTrajectorySampleV1 {
            time_seconds: 1.0,
            surface_centroid: [0.0, 0.0, 0.0],
            controller_centroid: [0.0, 1.0, 0.0],
        },
    ];
    let blocked =
        evaluate_visible_anchor_trajectory_v1("tail_tip", "cwalk", &stationary, 0.25, 0.5).unwrap();
    assert_eq!(blocked.status, "BLOCKED_AMPLITUDE");
    assert_eq!(blocked.surface_amplitude, 0.0);
    assert_eq!(blocked.controller_amplitude, 1.0);

    let following = stationary.map(|mut sample| {
        sample.surface_centroid = sample.controller_centroid;
        sample
    });
    let passed =
        evaluate_visible_anchor_trajectory_v1("tail_tip", "crun", &following, 0.25, 0.5).unwrap();
    assert_eq!(passed.status, "PASS");
    assert!(passed.amplitude_ratio >= 0.99);
    assert!(passed.trajectory_alignment >= 0.99);
}

#[test]
fn visible_surface_seams_block_even_when_the_legacy_fraction_would_pass() {
    let gate = evaluate_surface_seam_gate_v1(37_056_880, 820_131, 0.025, true).unwrap();
    assert_eq!(gate.legacy_allowed_count, 926_422);
    assert_eq!(gate.status, "BLOCKED_VISIBLE_SEAM");
    assert!(!gate.pass);
}

#[test]
fn renamed_model_is_not_a_semantic_delta_when_tail_surface_is_unchanged() {
    let baseline = ReferenceSupermodelRejectedBaselineV1 {
        model_sha256: "a".repeat(64),
        visible_surface_semantic_sha256: "b".repeat(64),
    };
    let unchanged = evaluate_reference_supermodel_semantic_delta_v1(
        &baseline,
        &"c".repeat(64),
        &"b".repeat(64),
    )
    .unwrap();
    assert!(unchanged.model_bytes_changed);
    assert!(!unchanged.visible_surface_changed);
    assert!(!unchanged.export_delta_proven);
}

fn assert_classic_meshes(nodes: &[NodeReport], count: &mut usize) {
    for node in nodes {
        if let Some(mesh) = node.mesh.as_ref() {
            *count += 1;
            assert_eq!(mesh.render_hint, 0);
            assert!(mesh.tangents.is_empty());
            assert!(mesh.textures.get(1).is_none_or(String::is_empty));
            assert!(mesh.textures.get(2).is_none_or(String::is_empty));
            assert!(mesh.textures.get(3).is_none_or(String::is_empty));
        }
        assert_classic_meshes(&node.children, count);
    }
}

fn assert_minimal_twosided_meshes(nodes: &[NodeReport], material_resref: &str, count: &mut usize) {
    for node in nodes {
        if let Some(mesh) = node.mesh.as_ref() {
            *count += 1;
            assert_eq!(mesh.render_hint, 1);
            assert_eq!(mesh.shadow, 0);
            assert!(mesh.tangents.is_empty());
            assert!(mesh.textures.get(1).is_none_or(String::is_empty));
            assert!(mesh.textures.get(2).is_none_or(String::is_empty));
            assert_eq!(
                mesh.textures.get(3).map(String::as_str),
                Some(material_resref)
            );
        }
        assert_minimal_twosided_meshes(&node.children, material_resref, count);
    }
}

fn minimal_twosided_profile() -> CreatureMaterialProfileV2 {
    CreatureMaterialProfileV2 {
        schema_version: 2,
        target: CreatureMaterialTargetV2::NwnEeMtr,
        normal_maps: false,
        tangent_space_ready: false,
        metallic_roughness_to_specular_gloss: false,
        emissive_to_self_illumination: false,
        alpha_mode: CreatureAlphaModeV2::Opaque,
        double_sided: true,
    }
}

fn identity() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn translated(x: f32, y: f32, z: f32) -> [f32; 16] {
    let mut matrix = identity();
    matrix[12] = x;
    matrix[13] = y;
    matrix[14] = z;
    matrix
}

fn target_rig() -> CreatureRigProfileV1 {
    let mut rig = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: "owned-motion-target-rig".to_owned(),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds: Bounds3V1 {
            min: [-1.0, -1.0, 0.0],
            max: [1.0, 1.0, 2.0],
        },
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes: vec![
            CreatureRigNodeV1 {
                id: 70,
                name: "owned_root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity(),
            },
            CreatureRigNodeV1 {
                id: 71,
                name: "owned_body".to_owned(),
                parent_id: Some(70),
                bind_local_matrix: translated(1.0, 0.0, 0.0),
            },
        ],
        segments: vec![CreatureRigSegmentV1 {
            id: 12,
            name: "owned_motion_skin".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 70,
            surface_positions: vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 2.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: vec![71],
            reference_weights: vec![
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 71,
                    value: 1.0,
                }];
                3
            ],
        }],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    rig
}

fn motion_contract() -> ReferenceSupermodelMotionContractV2 {
    let mut contract = ReferenceSupermodelMotionContractV2 {
        schema_version: 2,
        contract_id: "synthetic-supermodel-motion-v2".to_owned(),
        content_sha256: String::new(),
        supermodel_resref: "c_synth".to_owned(),
        source_model_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .to_owned(),
        inspected_read_only: true,
        no_payload_copied: true,
        classification: 4,
        animation_scale: 1.0,
        clean_room_profile_id: "synthetic-carrier-profile-v1".to_owned(),
        clean_room_profile_sha256:
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_owned(),
        nodes: vec![
            ReferenceSupermodelMotionNodeV2 {
                part_number: 0,
                name: "c_synth".to_owned(),
                parent_part_number: None,
                carrier_bind_local_matrix: identity(),
                position_controller_required: false,
                orientation_controller_required: false,
                scale_controller_required: false,
                anchor_role: Some("root".to_owned()),
                joint_axis: None,
                carrier_class: ReferenceSupermodelCarrierClassV3::PassiveStructural,
                structural_role: "ROOT".to_owned(),
                controlling_clips: Vec::new(),
                dynamic_clips: Vec::new(),
            },
            ReferenceSupermodelMotionNodeV2 {
                part_number: 1,
                name: "owned_body".to_owned(),
                parent_part_number: Some(0),
                // Deliberately differs from target bind [1, 0, 0]. Topology
                // alone cannot detect this incompatibility.
                carrier_bind_local_matrix: translated(0.0, 1.0, 0.0),
                position_controller_required: true,
                orientation_controller_required: false,
                scale_controller_required: false,
                anchor_role: Some("body".to_owned()),
                joint_axis: Some([0.0, 0.0, 1.0]),
                carrier_class: ReferenceSupermodelCarrierClassV3::SkinRelevant,
                structural_role: "ANIMATED_CHAIN_JOINT".to_owned(),
                controlling_clips: vec!["cpause1".to_owned()],
                dynamic_clips: vec!["cpause1".to_owned()],
            },
        ],
        carrier_exclusions: Vec::new(),
        required_clips: vec!["cpause1".to_owned()],
        required_events: Vec::new(),
        tolerances: default_reference_supermodel_motion_tolerances_v2(),
    };
    contract.content_sha256 =
        canonical_reference_supermodel_motion_contract_sha256_v2(&contract).unwrap();
    contract
}

fn reference_supermodel() -> m2a_core::BinaryMdlArtifactV1 {
    let reference = AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "synthetic-supermodel-reference".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
        engine_facing_proof: "SYNTHETIC".to_owned(),
        uv_runtime_proof: "SYNTHETIC".to_owned(),
        nodes: vec![
            AuroraModelNodeV1 {
                id: 70,
                name: "c_synth".to_owned(),
                parent_id: None,
                bind_local_matrix: identity(),
            },
            AuroraModelNodeV1 {
                id: 71,
                name: "owned_body".to_owned(),
                parent_id: Some(70),
                bind_local_matrix: translated(0.0, 1.0, 0.0),
            },
        ],
        material_source_bindings: Vec::new(),
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 10,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 70,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    };
    let animations = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "cpause1".to_owned(),
            animation_root: "c_synth".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.25,
            events: Vec::new(),
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 71,
                path: MdlAnimationTrackPathV1::Translation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 1.0, 0.0], vec![0.5, 1.0, 0.0]],
            }],
        }],
    };
    write_binary_mdl_with_animations(
        &reference,
        &animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "c_synth".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "csyntex".to_owned(),
            }],
        },
    )
    .unwrap()
}

#[test]
fn exact_contract_and_carrier_use_the_reference_bind_pose_instead_of_target_bind() {
    let reference = reference_supermodel();
    let contract = build_exact_reference_supermodel_motion_contract_v3(
        &reference.inspection,
        &ReferenceSupermodelExactContractOptionsV3 {
            contract_id: "synthetic-exact-supermodel-v3".to_owned(),
            supermodel_resref: "c_synth".to_owned(),
            source_model_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_owned(),
            required_clips: vec!["cpause1".to_owned()],
            required_events: Vec::new(),
            semantic_nodes: Vec::new(),
            tolerances: default_reference_supermodel_motion_tolerances_v2(),
        },
    )
    .expect("exact contract from inspected reference");

    assert_eq!(contract.nodes.len(), 2);
    assert_eq!(
        contract.nodes[1].carrier_bind_local_matrix,
        translated(0.0, 1.0, 0.0)
    );
    assert!(contract.nodes[1].position_controller_required);

    let artifact =
        build_exact_motion_carrier_rig_v3(&target_rig(), &contract, &reference.inspection)
            .expect("target mesh rebound to exact inspected carriers");
    assert_eq!(artifact.report.correction_node_count, 0);
    assert!(artifact.report.reference_bind_verified);
    assert!(artifact.report.source_bind_max_abs_error > 0.9);
    assert!(artifact.report.reference_bind_max_abs_error <= 1.0e-6);
    assert_eq!(
        artifact.rig.nodes[1].bind_local_matrix,
        translated(0.0, 1.0, 0.0)
    );
    assert_eq!(
        artifact.rig.segments[0].reference_weights,
        target_rig().segments[0].reference_weights
    );
}

#[test]
fn reference_bind_drift_is_rejected_even_when_name_and_parent_topology_match() {
    let reference = reference_supermodel();
    let mut drift = motion_contract();
    drift.nodes[1].carrier_bind_local_matrix = translated(0.0, 1.25, 0.0);
    drift.content_sha256 =
        canonical_reference_supermodel_motion_contract_sha256_v2(&drift).unwrap();

    let error = build_exact_motion_carrier_rig_v3(&target_rig(), &drift, &reference.inspection)
        .unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH");
}

#[test]
fn topology_match_with_different_bind_pose_gets_explicit_correction_nodes() {
    let target = target_rig();
    let artifact = build_motion_corrected_rig_v1(&target, &motion_contract()).unwrap();

    assert_eq!(artifact.report.carrier_node_count, 2);
    assert_eq!(artifact.report.correction_node_count, 2);
    assert!(artifact.report.neutral_max_abs_error <= 1.0e-6);
    assert_eq!(artifact.rig.nodes.len(), 4);
    assert_eq!(
        artifact.rig.nodes[1].bind_local_matrix,
        translated(0.0, 1.0, 0.0)
    );
    assert_eq!(artifact.rig.nodes[3].parent_id, Some(71));
    assert_eq!(
        artifact.rig.nodes[3].bind_local_matrix,
        translated(1.0, -1.0, 0.0)
    );
    let correction_body = artifact.rig.nodes[3].id;
    assert_eq!(
        artifact.rig.segments[0].allowed_bone_node_ids,
        vec![correction_body]
    );
    assert!(
        artifact.rig.segments[0]
            .reference_weights
            .iter()
            .all(|row| row[0].bone_node_id == correction_body)
    );
}

#[test]
fn motion_contract_hash_and_target_topology_fail_closed() {
    let mut hash_drift = motion_contract();
    hash_drift.content_sha256 = "f".repeat(64);
    let error = build_motion_corrected_rig_v1(&target_rig(), &hash_drift).unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-MOTION-CONTRACT-HASH-MISMATCH");

    let mut parent_drift = target_rig();
    parent_drift.nodes[1].parent_id = None;
    parent_drift.content_sha256.clear();
    parent_drift.content_sha256 = canonical_profile_sha256(&parent_drift).unwrap();
    let error = build_motion_corrected_rig_v1(&parent_drift, &motion_contract()).unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-MOTION-TOPOLOGY-MISMATCH");

    let mut invalid_tolerance = motion_contract();
    invalid_tolerance.tolerances.edge_hard_min_ratio = 0.75;
    invalid_tolerance.content_sha256 =
        canonical_reference_supermodel_motion_contract_sha256_v2(&invalid_tolerance).unwrap();
    let error = build_motion_corrected_rig_v1(&target_rig(), &invalid_tolerance).unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-MOTION-TOLERANCES-INVALID");
}

#[test]
fn required_controller_channels_and_clip_events_fail_closed() {
    let reference = reference_supermodel();
    let mut controller_drift = motion_contract();
    controller_drift.nodes[1].orientation_controller_required = true;
    controller_drift.content_sha256 =
        canonical_reference_supermodel_motion_contract_sha256_v2(&controller_drift).unwrap();
    let error =
        validate_reference_supermodel_motion_oracle_v2(&controller_drift, &reference.inspection)
            .unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-MOTION-CONTROLLER-MISSING");

    let mut event_drift = motion_contract();
    event_drift.required_events = vec![ReferenceSupermodelRequiredEventV2 {
        clip_name: "cpause1".to_owned(),
        event_name: "hit".to_owned(),
        minimum_count: 1,
    }];
    event_drift.content_sha256 =
        canonical_reference_supermodel_motion_contract_sha256_v2(&event_drift).unwrap();
    let error = validate_reference_supermodel_motion_oracle_v2(&event_drift, &reference.inspection)
        .unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-MOTION-EVENT-MISSING");
}

#[test]
fn material_complete_motion_route_packages_mtr_normal_specular_and_twosided() {
    let source = fixtures::material_complete_single_primitive();
    let reference = reference_supermodel();
    let artifact = build_reference_supermodel_creature_package_v1(
        &source,
        &target_rig(),
        &motion_contract(),
        &reference.inspection,
        &default_reference_supermodel_writer_options_v1("m2a_motion"),
        CreatureSourceForwardV1::PositiveZ,
        &ReferenceSupermodelMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: "m2amotex".to_owned(),
        },
    )
    .unwrap();

    assert_eq!(
        artifact.material_compilation.target_profile,
        AuroraMaterialTargetProfileV1::NwnEeMtr
    );
    assert_eq!(
        artifact.material_compilation.status,
        AuroraMaterialCompileStatusV1::Ready
    );
    let mtr_resource = artifact
        .material_package
        .resources
        .iter()
        .find(|resource| resource.resource_type == MTR_RESOURCE_TYPE_V1)
        .expect("MTR resource");
    let mtr = parse_mtr_v1(&mtr_resource.payload).unwrap();
    assert!(mtr.two_sided);
    assert_eq!(mtr.render_hint, MtrRenderHintV1::NormalAndSpecMapped);
    assert_eq!(
        mtr.textures
            .iter()
            .map(|binding| binding.slot)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        artifact
            .material_package
            .resources
            .iter()
            .filter(|resource| resource.resource_type == 3)
            .count(),
        3
    );
    assert!(artifact.model.report.semantic_diff.is_empty());
    assert!(artifact.material_extension.semantic_diff.is_empty());
    assert_eq!(artifact.motion_quality.status, "PASS");
    assert_eq!(
        artifact.material_semantics.status,
        CreatureMaterialSemanticStatusV1::Ready
    );
    assert_eq!(artifact.material_semantics.source_image_count, 3);
    assert_eq!(artifact.material_semantics.source_images.len(), 3);
    assert_eq!(
        artifact
            .material_semantics
            .source_images
            .iter()
            .map(|image| image.disposition.as_str())
            .collect::<Vec<_>>(),
        vec!["PRESERVED", "PRESERVED", "BAKED"]
    );
    assert!(
        artifact
            .material_semantics
            .source_images
            .iter()
            .all(|image| !image.source_channels.is_empty() && !image.output_resrefs.is_empty())
    );
    assert!(
        !artifact.material_semantics.slots[0]
            .diffuse_output_resref
            .is_empty()
    );
    assert!(
        artifact.material_semantics.slots[0]
            .normal_output_resref
            .is_some()
    );
    assert!(
        artifact.material_semantics.slots[0]
            .specular_output_resref
            .is_some()
    );
    assert!(
        artifact.material_semantics.slots[0]
            .material_output_resref
            .is_some()
    );
    assert_eq!(artifact.material_semantics.slots[0].resources.len(), 7);
    assert!(
        artifact.material_semantics.slots[0]
            .resources
            .iter()
            .all(|resource| resource.packaged)
    );
    assert!(artifact.report.motion_compatible);
    assert!(artifact.report.clip_inventory_verified);

    let repeated = bind_static_mesh_for_inherited_supermodel_motion_v1(
        &source,
        &target_rig(),
        &motion_contract(),
        &reference.inspection,
        &default_reference_supermodel_writer_options_v1("m2a_motion"),
        CreatureSourceForwardV1::PositiveZ,
        &ReferenceSupermodelMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: "m2amotex".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(artifact.model.payload, repeated.model.payload);
    assert_eq!(
        artifact.material_package.resources,
        repeated.material_package.resources
    );
    let appearance = b"2DA V2.0\n\nLABEL\n0 M2A_MOTION\n";
    let hak = package_reference_supermodel_creature_hak_v1(
        &artifact,
        appearance,
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    let repeated_hak = package_reference_supermodel_creature_hak_v1(
        &repeated,
        appearance,
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    assert_eq!(hak.resource_count, 9);
    assert_eq!(hak.hak.payload, repeated_hak.hak.payload);

    let mut v3_regression = artifact.material_package.clone();
    v3_regression
        .resources
        .retain(|resource| resource.resource_type != MTR_RESOURCE_TYPE_V1);
    let ingest = ingest_glb(&source, &GlbLimits::default()).unwrap();
    let report = build_creature_material_semantic_report_v1(
        &ingest,
        artifact.conversion.creature.as_ref().unwrap(),
        &artifact.material_compilation,
        &v3_regression,
    )
    .unwrap();
    assert_eq!(report.status, CreatureMaterialSemanticStatusV1::Blocked);
    assert!(
        report.slots[0]
            .resources
            .iter()
            .any(|resource| resource.resource_type == MTR_RESOURCE_TYPE_V1 && !resource.packaged)
    );
}

#[test]
fn direct_classic_motion_route_fails_closed_for_double_sided_source() {
    let source = fixtures::material_complete_single_primitive();
    let reference = reference_supermodel();
    let error = bind_static_mesh_for_inherited_supermodel_motion_direct_classic_v1(
        &source,
        &target_rig(),
        &motion_contract(),
        &reference.inspection,
        &default_reference_supermodel_writer_options_v1("m2a_safe"),
        CreatureSourceForwardV1::PositiveZ,
        &ReferenceSupermodelClassicMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: "m2asafetex".to_owned(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "M2A-SUPERMODEL-CLASSIC-TWO-SIDED-BLOCKED");
}

#[test]
fn direct_classic_motion_route_is_runtime_conservative_for_one_sided_source() {
    let source = fixtures::material_complete_single_primitive_with_double_sided(false);
    let reference = reference_supermodel();
    let artifact = bind_static_mesh_for_inherited_supermodel_motion_direct_classic_v1(
        &source,
        &target_rig(),
        &motion_contract(),
        &reference.inspection,
        &default_reference_supermodel_writer_options_v1("m2a_safe"),
        CreatureSourceForwardV1::PositiveZ,
        &ReferenceSupermodelClassicMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: "m2asafetex".to_owned(),
        },
    )
    .expect("one-sided classic-safe inherited motion");

    assert!(artifact.report.motion_compatible);
    assert_eq!(
        artifact.report.material_profile,
        "CLASSIC_DIFFUSE_TGA_SAFE_V1"
    );
    assert!(!artifact.report.material_extension_applied);
    assert_eq!(artifact.report.tangent_stream_count, 0);
    assert_eq!(artifact.report.runtime_readiness, "RUNTIME_UNPROVEN");

    let mut mesh_count = 0;
    assert_classic_meshes(&artifact.model.inspection.node_tree.roots, &mut mesh_count);
    assert!(mesh_count > 0);

    let appearance = b"2DA V2.0\n\nLABEL\n0 M2A_SAFE\n";
    let texture = b"owned-classic-diffuse-fixture";
    let package = package_reference_supermodel_classic_creature_hak_v1(
        &artifact,
        "m2asafetex",
        texture,
        appearance,
        &HakWriterOptionsV1::default(),
    )
    .expect("classic-safe HAK");
    assert_eq!(package.resource_count, 3);
    let archive = ErfArchive::parse(&package.hak.payload).expect("classic-safe HAK readback");
    assert_eq!(archive.resources().len(), 3);
    assert_eq!(archive.find("m2asafetex", 3).unwrap(), texture);
    assert!(
        archive
            .resources()
            .iter()
            .all(|resource| { !matches!(resource.resource_type, 2022 | MTR_RESOURCE_TYPE_V1) })
    );
}

#[test]
fn minimal_twosided_motion_route_emits_only_diffuse_mtr_and_no_tangents() {
    let source = fixtures::material_complete_single_primitive();
    let reference = reference_supermodel();
    let artifact = bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_v1(
        &source,
        &target_rig(),
        &motion_contract(),
        &reference.inspection,
        &default_reference_supermodel_writer_options_v1("m2a_twosided"),
        CreatureSourceForwardV1::PositiveZ,
        &ReferenceSupermodelMinimalMtrMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: "m2atwotex".to_owned(),
            material_resref: "m2atwomtr".to_owned(),
            material_profile: minimal_twosided_profile(),
        },
    )
    .expect("minimal two-sided inherited motion");

    assert_eq!(
        artifact.report.material_profile,
        "NWN_EE_MTR_TWOSIDED_ONLY_V1"
    );
    assert!(artifact.report.material_extension_applied);
    assert_eq!(artifact.report.tangent_stream_count, 0);
    assert_eq!(artifact.report.material_resource_count, 2);
    assert_eq!(artifact.motion_quality.status, "PASS");
    assert_eq!(artifact.correction.report.correction_node_count, 0);
    assert!(artifact.correction.report.reference_bind_verified);
    assert!(artifact.model.inspection.animations.is_empty());

    let mtr = parse_mtr_v1(&artifact.mtr_resource.payload).unwrap();
    assert_eq!(artifact.mtr_resource.resource_type, MTR_RESOURCE_TYPE_V1);
    assert_eq!(artifact.mtr_resource.resref, "m2atwomtr");
    assert!(mtr.two_sided);
    assert!(!mtr.transparency);
    assert_eq!(mtr.render_hint, MtrRenderHintV1::Normal);
    assert_eq!(
        mtr.textures
            .iter()
            .map(|binding| (binding.slot, binding.resref.as_str()))
            .collect::<Vec<_>>(),
        vec![(0, "m2atwotex")]
    );

    let mut mesh_count = 0;
    assert_minimal_twosided_meshes(
        &artifact.model.inspection.node_tree.roots,
        "m2atwomtr",
        &mut mesh_count,
    );
    assert!(mesh_count > 0);

    let appearance = b"2DA V2.0\n\nLABEL\n0 M2A_TWOSIDED\n";
    let texture = b"owned-minimal-twosided-diffuse-fixture";
    let package = package_reference_supermodel_minimal_twosided_creature_hak_v1(
        &artifact,
        "m2atwotex",
        texture,
        appearance,
        &HakWriterOptionsV1::default(),
    )
    .expect("minimal two-sided HAK");
    assert_eq!(package.resource_count, 4);
    let archive = ErfArchive::parse(&package.hak.payload).expect("minimal two-sided HAK readback");
    assert_eq!(archive.resources().len(), 4);
    assert_eq!(archive.find("m2atwotex", 3).unwrap(), texture);
    assert_eq!(
        archive.find("m2atwomtr", MTR_RESOURCE_TYPE_V1).unwrap(),
        artifact.mtr_resource.payload
    );
    assert_eq!(
        archive
            .resources()
            .iter()
            .filter(|resource| resource.resource_type == 3)
            .count(),
        1
    );
}

#[test]
fn corrected_minimal_twosided_route_keeps_exact_carriers_and_owned_bind_corrections() {
    let source = fixtures::material_complete_single_primitive();
    let reference = reference_supermodel();
    let artifact =
        bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_corrected_v2(
            &source,
            &target_rig(),
            &motion_contract(),
            &reference.inspection,
            &default_reference_supermodel_writer_options_v1("m2a_corrtwo"),
            CreatureSourceForwardV1::PositiveZ,
            &ReferenceSupermodelMinimalMtrMaterialOptionsV1 {
                schema_version: 1,
                texture_resref: "m2acorrtex".to_owned(),
                material_resref: "m2acorrmtr".to_owned(),
                material_profile: minimal_twosided_profile(),
            },
        )
        .expect("corrected minimal two-sided inherited motion");

    assert!(artifact.report.motion_compatible);
    assert!(artifact.report.bind_pose_compatible);
    assert!(artifact.correction.report.reference_bind_verified);
    assert_eq!(artifact.correction.report.carrier_node_count, 2);
    assert_eq!(artifact.correction.report.correction_node_count, 2);
    assert_eq!(artifact.correction.rig.nodes.len(), 4);
    assert_eq!(artifact.motion_quality.anchor_cluster_missing_count, 0);
    assert!(!artifact.report.motion_weight_refinement_applied);
    assert_eq!(artifact.report.motion_weight_refinement_iteration_count, 0);
    assert_eq!(
        artifact
            .report
            .motion_weight_refinement_projected_triangle_count,
        0
    );
}

#[test]
fn verified_retarget_carrier_keeps_owned_pivots_and_exact_reference_names() {
    let target = target_rig();
    let contract = motion_contract();
    let reference = reference_supermodel();
    let artifact =
        build_retargeted_motion_carrier_rig_v4(&target, &contract, &reference.inspection)
            .expect("verified retarget carrier");

    assert_eq!(artifact.report.schema_version, 4);
    assert!(!artifact.report.reference_bind_verified);
    assert_eq!(artifact.report.correction_node_count, 0);
    assert_eq!(artifact.rig.nodes.len(), target.nodes.len());
    for ((output, owned), exact) in artifact
        .rig
        .nodes
        .iter()
        .zip(&target.nodes)
        .zip(&contract.nodes)
    {
        assert_eq!(output.bind_local_matrix, owned.bind_local_matrix);
        assert_eq!(output.name, exact.name);
    }
}

#[test]
fn local_clip_retarget_moves_absolute_reference_translation_to_target_bind() {
    let target = target_rig();
    let contract = motion_contract();
    let reference = reference_supermodel();

    let animations =
        retarget_reference_supermodel_animation_set_v5(&target, &contract, &reference.inspection)
            .expect("retarget exact reference clip");

    assert_eq!(animations.clips.len(), 1);
    let track = animations.clips[0]
        .tracks
        .iter()
        .find(|track| {
            track.target_node_id == 71 && track.path == MdlAnimationTrackPathV1::Translation
        })
        .expect("retargeted body translation");
    assert_eq!(track.times_seconds, [0.0, 1.0]);
    assert_eq!(track.values, [vec![1.0, 0.0, 0.0], vec![1.5, 0.0, 0.0]]);
}
