use m2a_core::placeable::AuroraPlaceableIrV1;
use m2a_core::{
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
    MdlStateProjectionProfileV1, MdlWriterOptionsV1, write_binary_mdl,
};

fn static_model() -> AuroraModelIrV1 {
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "placeable-static-test".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "AURORA_Z_UP".to_owned(),
        engine_facing_proof: "OFFLINE_ONLY".to_owned(),
        uv_runtime_proof: "OFFLINE_ONLY".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 0,
            name: "m2a_plc_test".to_owned(),
            parent_id: None,
            bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0,
            ],
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: Some(0),
            source_material_name: Some("pedestal".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 0,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 0,
            cast_shadow: true,
            positions: vec![[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 1.5]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    }
}

#[test]
fn placeable_domain_ir_is_the_shared_model_ir_without_a_copy() {
    let placeable: AuroraPlaceableIrV1 = static_model();
    let common: AuroraModelIrV1 = placeable;
    assert_eq!(common.profile_id, "placeable-static-test");
}

#[test]
fn placeable_profile_uses_the_common_binary_mdl_writer_and_geometry_bounds() {
    let artifact = write_binary_mdl(
        &static_model(),
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::PlaceableStaticRigidNativeV1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "m2a_plc_test".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "m2a_plc_tex".to_owned(),
            }],
        },
    )
    .expect("shared writer should emit a static placeable model");

    assert_eq!(
        artifact.report.format_profile,
        MdlFormatProfileV1::PlaceableStaticRigidNativeV1
    );
    assert_eq!(artifact.report.projection.mesh_node_count, 1);
    assert_eq!(artifact.report.projection.triangle_count, 1);
    assert_eq!(
        [
            artifact.inspection.model.bounds_min.x,
            artifact.inspection.model.bounds_min.y,
            artifact.inspection.model.bounds_min.z,
        ],
        [-0.5, -0.5, 0.0]
    );
    assert_eq!(
        [
            artifact.inspection.model.bounds_max.x,
            artifact.inspection.model.bounds_max.y,
            artifact.inspection.model.bounds_max.z,
        ],
        [0.5, 0.5, 1.5]
    );
    assert!(artifact.report.semantic_diff.is_empty());
}
