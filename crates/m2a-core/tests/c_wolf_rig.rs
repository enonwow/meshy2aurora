#![cfg(feature = "legacy-c-wolf-demo")]

#[path = "fixtures/build_synthetic_glb.rs"]
mod fixtures;

use m2a_core::{
    c_wolf_rig::{
        build_c_wolf_compatible_rig_v1, build_c_wolf_motion_contract_v2,
        c_wolf_compatibility_topology_v1,
    },
    glb::{GlbLimits, ingest_glb},
    profile_a::{RigProvenanceKindV1, RigSegmentDeformationV1, canonical_profile_sha256},
    reference_supermodel_motion::{
        build_motion_corrected_rig_v1, canonical_reference_supermodel_motion_contract_sha256_v2,
    },
};

const SYNTHETIC_REFERENCE_SHA256: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn owned_surface_produces_exact_c_wolf_topology_and_normalized_weights() {
    let source = ingest_glb(
        &fixtures::one_primitive_two_disconnected_boxes_with_embedded_texture(),
        &GlbLimits::default(),
    )
    .unwrap();
    let rig = build_c_wolf_compatible_rig_v1(&source).unwrap();

    let actual_topology = rig
        .nodes
        .iter()
        .map(|node| (node.id, node.name.as_str(), node.parent_id))
        .collect::<Vec<_>>();
    assert_eq!(actual_topology, c_wolf_compatibility_topology_v1());
    assert_eq!(rig.nodes.len(), 30);
    assert_eq!(rig.provenance.kind, RigProvenanceKindV1::Owned);
    assert!(rig.provenance.export_allowed);
    assert!(rig.provenance.attestations.controlled_construction);
    assert!(rig.provenance.attestations.no_reference_payload_copied);
    assert!(rig.provenance.attestations.rights_confirmed);
    assert_eq!(rig.content_sha256, canonical_profile_sha256(&rig).unwrap());

    assert_eq!(rig.segments.len(), 1);
    let skin = &rig.segments[0];
    assert_eq!(skin.deformation, RigSegmentDeformationV1::Skin);
    assert_eq!(skin.reference_weights.len(), skin.surface_positions.len());
    assert_eq!(skin.allowed_bone_node_ids, (2..=27).collect::<Vec<_>>());
    for row in &skin.reference_weights {
        assert!(!row.is_empty() && row.len() <= 4);
        let sum = row.iter().map(|influence| influence.value).sum::<f32>();
        assert!((sum - 1.0).abs() <= 1.0e-6);
        assert!(row.iter().all(|influence| {
            influence.value > 0.0 && skin.allowed_bone_node_ids.contains(&influence.bone_node_id)
        }));
    }
}

#[test]
fn transformed_source_is_rejected_instead_of_guessing_world_space() {
    let source = ingest_glb(
        &fixtures::axis_hierarchy_asymmetric(),
        &GlbLimits::default(),
    )
    .unwrap();
    let error = build_c_wolf_compatible_rig_v1(&source).unwrap_err();
    assert_eq!(error.code, "M2A-CWOLF-RIG-SOURCE-TRANSFORM-UNSUPPORTED");
}

#[test]
fn c_wolf_motion_contract_is_clean_room_fixed_and_builds_correction_layer() {
    let source = ingest_glb(
        &fixtures::one_primitive_two_disconnected_boxes_with_embedded_texture(),
        &GlbLimits::default(),
    )
    .unwrap();
    let target = build_c_wolf_compatible_rig_v1(&source).unwrap();
    let contract = build_c_wolf_motion_contract_v2(SYNTHETIC_REFERENCE_SHA256).unwrap();

    assert_eq!(contract.nodes.len(), 30);
    assert_eq!(contract.required_clips.len(), 8);
    assert_eq!(contract.required_events.len(), 6);
    assert_eq!(contract.classification, 4);
    assert_eq!(contract.animation_scale, 1.0);
    assert_eq!(
        contract.content_sha256,
        canonical_reference_supermodel_motion_contract_sha256_v2(&contract).unwrap()
    );
    assert_eq!(
        contract.nodes[1].carrier_bind_local_matrix[13], 0.0,
        "carrier profile must not inherit source-fitted root placement"
    );

    let corrected = build_motion_corrected_rig_v1(&target, &contract).unwrap();
    assert_eq!(corrected.report.carrier_node_count, 30);
    assert_eq!(corrected.report.correction_node_count, 30);
    assert_eq!(corrected.rig.nodes.len(), 60);
    assert!(corrected.report.neutral_max_abs_error <= 1.0e-4);
    assert!(
        corrected.rig.segments[0]
            .allowed_bone_node_ids
            .iter()
            .all(|bone| *bone >= 30)
    );
}
