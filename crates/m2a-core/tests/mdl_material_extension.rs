use m2a_core::mdl::{
    MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1, MdlAnimationTrackPathV1,
    MdlAnimationTrackV1, MdlFormatProfileV1, MdlMaterialExtensionOptionsV1, MdlMaterialStateV1,
    MdlMaterialTextureBindingV1, MdlRenderHintV1, MdlSegmentMaterialStreamsV1,
    MdlStateProjectionProfileV1, MdlWriterOptionsV1, extend_binary_mdl_with_materials_v1,
    write_binary_mdl_with_animations, write_binary_mdl_with_materials_v1,
};
use m2a_core::model_ir::{
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1,
};

fn model() -> AuroraModelIrV1 {
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "material-extension-fixture".to_owned(),
        source_sha256: "0".repeat(64),
        basis_status: "LOCKED".to_owned(),
        engine_facing_proof: "OFFLINE".to_owned(),
        uv_runtime_proof: "OFFLINE".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 1,
            name: "root".to_owned(),
            parent_id: None,
            bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 7,
            source_material_id: Some(12),
            source_material_name: Some("hull".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 9,
            material_slot: 7,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 1,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: Some(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    }
}

fn writer_options() -> MdlWriterOptionsV1 {
    MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::PlaceableStaticRigidNativeV1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: "m2a_matext".to_owned(),
        diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
            material_slot: 7,
            resref: "hull_d".to_owned(),
        }],
    }
}

fn material_options() -> MdlMaterialExtensionOptionsV1 {
    MdlMaterialExtensionOptionsV1 {
        schema_version: 1,
        materials: vec![MdlMaterialStateV1 {
            material_slot: 7,
            diffuse: [0.75, 0.5, 0.25],
            ambient: [0.2, 0.15, 0.1],
            specular: [0.08, 0.07, 0.06],
            shininess: 18.0,
            alpha: 0.85,
            self_illum_color: [0.03, 0.02, 0.01],
            transparency_hint: true,
            render_hint: MdlRenderHintV1::NormalAndSpecMapped,
            normal_texture_resref: Some("hull_n".to_owned()),
            specular_texture_resref: Some("hull_s".to_owned()),
            material_resref: Some("hull_mtr".to_owned()),
        }],
        segment_streams: vec![MdlSegmentMaterialStreamsV1 {
            segment_id: 9,
            uv1: vec![[0.1, 0.2], [0.3, 0.4], [0.5, 0.6]],
            uv2: vec![[0.6, 0.5], [0.4, 0.3], [0.2, 0.1]],
            uv3: vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
        }],
    }
}

#[test]
fn extended_material_writer_roundtrips_all_supported_state_and_streams() {
    let artifact =
        write_binary_mdl_with_materials_v1(&model(), &writer_options(), &material_options())
            .expect("extended material writer");
    assert!(artifact.material_report.semantic_diff.is_empty());
    assert_eq!(artifact.material_report.segment_readback.len(), 1);
    let segment = &artifact.material_report.segment_readback[0];
    assert_eq!(segment.uv_set_count, 4);
    assert_eq!(segment.tangent_count, 3);
    assert!(segment.alpha_controller_present);
    assert!(segment.self_illum_controller_present);
    assert!(segment.semantic_match);
    assert_eq!(
        artifact.material_report.payload_sha256,
        artifact.binary.report.payload_sha256
    );
    assert!(
        artifact
            .binary
            .report
            .deviations
            .iter()
            .all(|entry| entry.code != "M4-TANGENTS-NOT-EMITTED")
    );
}

#[test]
fn extended_material_writer_rejects_non_contiguous_uv_sets() {
    let mut options = material_options();
    options.segment_streams[0].uv1.clear();
    let error = write_binary_mdl_with_materials_v1(&model(), &writer_options(), &options)
        .expect_err("UV2 without UV1 must fail");
    assert_eq!(error.code, "M4M-UV-GAP");
}

#[test]
fn material_extension_preserves_an_already_written_animation_table() {
    let mut options = writer_options();
    options.format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64V1;
    let animations = MdlAnimationSetV1 {
        schema_version: 1,
        clips: vec![MdlAnimationClipV1 {
            name: "idle".to_owned(),
            animation_root: "root".to_owned(),
            length_seconds: 1.0,
            transition_seconds: 0.25,
            events: Vec::new(),
            tracks: vec![MdlAnimationTrackV1 {
                target_node_id: 1,
                path: MdlAnimationTrackPathV1::Translation,
                interpolation: MdlAnimationInterpolationV1::Linear,
                times_seconds: vec![0.0, 1.0],
                values: vec![vec![0.0, 0.0, 0.0], vec![0.1, 0.0, 0.0]],
            }],
        }],
    };
    let animated = write_binary_mdl_with_animations(&model(), &animations, &options)
        .expect("animated base writer");
    let extended = extend_binary_mdl_with_materials_v1(&model(), animated, &material_options())
        .expect("animated material extension");
    assert_eq!(extended.binary.inspection.animations.len(), 1);
    assert_eq!(extended.binary.inspection.animations[0].name, "idle");
    assert!(extended.material_report.semantic_diff.is_empty());
}
