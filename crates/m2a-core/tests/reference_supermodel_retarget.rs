#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod fixtures;

use m2a_core::{
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
        MdlWriterOptionsV1,
    },
    profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1, RigSegmentDeformationV1,
        RigWeightInfluenceV1, canonical_profile_sha256,
    },
    reference_supermodel::{
        ReferenceSupermodelContractV1, ReferenceSupermodelNodeV1,
        canonical_reference_supermodel_contract_sha256_v1,
        retarget_static_mesh_to_reference_supermodel_v1,
    },
};

fn identity() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn owned_skin_profile() -> CreatureRigProfileV1 {
    let mut rig = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: "owned-reference-supermodel-rig".to_owned(),
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
                bind_local_matrix: identity(),
            },
        ],
        segments: vec![CreatureRigSegmentV1 {
            id: 12,
            name: "owned_skin_surface".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 70,
            surface_positions: vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 2.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: vec![71],
            reference_weights: vec![
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 71,
                    value: 1.0,
                }],
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 71,
                    value: 1.0,
                }],
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 71,
                    value: 1.0,
                }],
            ],
        }],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    rig
}

fn contract() -> ReferenceSupermodelContractV1 {
    let mut contract = ReferenceSupermodelContractV1 {
        schema_version: 1,
        contract_id: "synthetic-reference-supermodel".to_owned(),
        content_sha256: String::new(),
        supermodel_resref: "c_synth".to_owned(),
        source_model_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .to_owned(),
        inspected_read_only: true,
        no_payload_copied: true,
        nodes: vec![
            ReferenceSupermodelNodeV1 {
                part_number: 0,
                name: "c_synth".to_owned(),
                parent_part_number: None,
            },
            ReferenceSupermodelNodeV1 {
                part_number: 1,
                name: "owned_body".to_owned(),
                parent_part_number: Some(0),
            },
        ],
    };
    contract.content_sha256 = canonical_reference_supermodel_contract_sha256_v1(&contract).unwrap();
    contract
}

fn writer_options() -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_ref01".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_reftex".to_owned(),
        }],
    }
}

#[test]
fn static_mesh_is_scaled_weighted_and_emitted_against_exact_supermodel_contract() {
    let artifact = retarget_static_mesh_to_reference_supermodel_v1(
        &fixtures::minimal_indexed_triangle(),
        &owned_skin_profile(),
        &contract(),
        &writer_options(),
    )
    .unwrap();

    assert!(artifact.conversion.report.conversion_eligible);
    assert_eq!(artifact.report.supermodel_resref, "c_synth");
    assert_eq!(artifact.report.rig_node_count, 2);
    assert_eq!(artifact.report.skin_segment_count, 1);
    assert_eq!(artifact.report.active_bone_count, 1);
    assert_eq!(artifact.report.local_animation_count, 0);
    assert_eq!(artifact.conversion.report.transform.scale, Some(2.0));

    let creature = artifact.conversion.creature.as_ref().unwrap();
    assert_eq!(creature.nodes[0].name, "m2a_ref01");
    assert_eq!(creature.nodes[1].name, "owned_body");
    assert_eq!(
        creature.segments[0].deformation,
        RigSegmentDeformationV1::Skin
    );
    assert!(creature.segments[0].weights.iter().all(|row| {
        row.influence_count == 1
            && row.bone_node_ids == [Some(71), None, None, None]
            && row.values == [1.0, 0.0, 0.0, 0.0]
    }));

    assert_eq!(artifact.model.inspection.model.supermodel_name, "c_synth");
    assert_eq!(
        artifact
            .model
            .inspection
            .model
            .animation_pointers_header
            .used,
        0
    );
    assert!(artifact.model.report.semantic_diff.is_empty());
    let root = &artifact.model.inspection.node_tree.roots[0];
    assert_eq!(root.name, "m2a_ref01");
    assert_eq!(root.children[0].name, "owned_body");
    assert!(root.children.iter().any(|node| node.skin.is_some()));
}

#[test]
fn child_name_or_parent_drift_is_rejected_before_geometry_conversion() {
    let mut name_drift = contract();
    name_drift.nodes[1].name = "wrong_body".to_owned();
    name_drift.content_sha256.clear();
    name_drift.content_sha256 =
        canonical_reference_supermodel_contract_sha256_v1(&name_drift).unwrap();
    let error = retarget_static_mesh_to_reference_supermodel_v1(
        &fixtures::minimal_indexed_triangle(),
        &owned_skin_profile(),
        &name_drift,
        &writer_options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M7V5-SUPERMODEL-TOPOLOGY-MISMATCH");

    let mut parent_drift = owned_skin_profile();
    parent_drift.nodes[1].parent_id = None;
    parent_drift.content_sha256.clear();
    parent_drift.content_sha256 = canonical_profile_sha256(&parent_drift).unwrap();
    let error = retarget_static_mesh_to_reference_supermodel_v1(
        &fixtures::minimal_indexed_triangle(),
        &parent_drift,
        &contract(),
        &writer_options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M7V5-SUPERMODEL-TOPOLOGY-MISMATCH");
}

#[test]
fn contract_hash_and_skin_requirement_fail_closed() {
    let mut drifted = contract();
    drifted.content_sha256 = "f".repeat(64);
    let error = retarget_static_mesh_to_reference_supermodel_v1(
        &fixtures::minimal_indexed_triangle(),
        &owned_skin_profile(),
        &drifted,
        &writer_options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M7V5-SUPERMODEL-CONTRACT-HASH-MISMATCH");

    let mut rigid = owned_skin_profile();
    rigid.segments[0].deformation = RigSegmentDeformationV1::Rigid;
    rigid.segments[0].allowed_bone_node_ids.clear();
    rigid.segments[0].reference_weights.clear();
    rigid.content_sha256.clear();
    rigid.content_sha256 = canonical_profile_sha256(&rigid).unwrap();
    let error = retarget_static_mesh_to_reference_supermodel_v1(
        &fixtures::minimal_indexed_triangle(),
        &rigid,
        &contract(),
        &writer_options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M7V5-SKIN-REQUIRED");
}

#[test]
fn reference_only_rig_provenance_cannot_be_exported() {
    let mut rig = owned_skin_profile();
    rig.provenance.kind = RigProvenanceKindV1::ReferenceOnly;
    rig.provenance.export_allowed = false;
    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();

    let error = retarget_static_mesh_to_reference_supermodel_v1(
        &fixtures::minimal_indexed_triangle(),
        &rig,
        &contract(),
        &writer_options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "M7V5-SOURCE-INELIGIBLE");
    assert!(error.message.contains("M3A-PROFILE-PROVENANCE-FORBIDDEN"));
}
