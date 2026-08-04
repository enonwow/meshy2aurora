#[path = "fixtures/build_synthetic_glb.rs"]
mod build_synthetic_glb;

use m2a_core::item::{
    ItemAttachmentProfileV1, ItemAttachmentRouteV1, ItemFitSourceV1, ItemReferenceMdlInputV1,
    ItemReferenceProfileIdentityV1, ItemReferenceSlotFrameV1, build_item_attachment_profile_v1,
    fit_meshy_item_parts_to_attachment_profile_v1,
    fit_meshy_item_parts_to_attachment_profile_with_axial_scales_v1, validate_item_fit_report_v4,
    validate_item_fit_report_v4_against_profile_v1,
};
use m2a_core::{
    key_bif::{KeyBifFileInputV1, locate_key_bif_resource_v1, resolve_key_bif_resource_sparse_v1},
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
        MdlWriterOptionsV1, write_binary_mdl,
    },
    model_ir::{
        AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
        AuroraSegmentDeformationV1,
    },
};

fn reference_mdl(resref: &str) -> Vec<u8> {
    let identity = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let model = AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "test-reference-item".to_owned(),
        source_sha256: "9".repeat(64),
        basis_status: "test".to_owned(),
        engine_facing_proof: "test".to_owned(),
        uv_runtime_proof: "test".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 0,
            name: resref.to_owned(),
            parent_id: None,
            bind_local_matrix: identity,
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: Some(0),
            source_material_name: Some("reference".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 0,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 0,
            cast_shadow: true,
            positions: vec![
                [-0.1, -0.2, -0.1],
                [0.1, -0.2, -0.1],
                [0.0, 0.3, -0.1],
                [0.0, 0.0, 0.1],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: None,
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.5, 0.5]],
            indices: vec![0, 1, 2, 0, 3, 1, 1, 3, 2, 2, 3, 0],
            face_surface_ids: Vec::new(),
            weights: Vec::new(),
        }],
    };
    write_binary_mdl(
        &model,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::ItemPartStaticRigidAuroraComposerV2,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: resref.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "reference_tex".to_owned(),
            }],
        },
    )
    .unwrap()
    .payload
}

fn retail_wswls_profile() -> ItemAttachmentProfileV1 {
    let mut profile = ItemAttachmentProfileV1 {
        schema_version: 1,
        algorithm: "AURORA_ITEM_REFERENCE_PROFILE_V1".to_owned(),
        status: "PASSED".to_owned(),
        identity: ItemReferenceProfileIdentityV1 {
            schema_version: 1,
            resource_context_sha256: "1".repeat(64),
            baseitems_sha256: "2".repeat(64),
            base_item: 1,
            item_class: "WSwLs".to_owned(),
            model_type: 2,
            reference_kind: "REFERENCE_UTI".to_owned(),
            reference_id: "nw_wswmls002".to_owned(),
        },
        attachment_route: ItemAttachmentRouteV1::Hand,
        equipable_slots: 0x1c030,
        common_origin: [0.0, 0.0, 0.0],
        axial_axis: 1,
        width_axis: 2,
        depth_axis: 0,
        attachment_zone_min: [-0.025740018, -0.2025382, -0.06724753],
        attachment_zone_max: [0.026107183, 0.0841520, 0.067631565],
        attachment_evidence: "ORIGIN_CONTAINING_REFERENCE_PARTS_V1".to_owned(),
        slots: vec![
            ItemReferenceSlotFrameV1 {
                field: "ModelPart1".to_owned(),
                label: "Bottom".to_owned(),
                token: "b".to_owned(),
                model_resref: "wswls_b_023".to_owned(),
                model_sha256: "3".repeat(64),
                controller_node_name: "g_WSwLs_b_023".to_owned(),
                controller_translation: [0.000183583, -0.14571, 0.000191967],
                controller_rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
                bounds_min: [-0.025740017, -0.2025382, -0.06724753],
                bounds_max: [0.026107183, 0.0841520, 0.06763157],
                allow_axial_extension_at_min: false,
                allow_axial_extension_at_max: false,
            },
            ItemReferenceSlotFrameV1 {
                field: "ModelPart2".to_owned(),
                label: "Middle".to_owned(),
                token: "m".to_owned(),
                model_resref: "wswls_m_063".to_owned(),
                model_sha256: "4".repeat(64),
                controller_node_name: "g_WSwLs_m_063".to_owned(),
                controller_translation: [0.000394173, -0.0206093, 0.000168152],
                controller_rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
                bounds_min: [-0.025861627, 0.042125102, -0.09729305],
                bounds_max: [0.025945173, 0.1570407, 0.09743625],
                allow_axial_extension_at_min: false,
                allow_axial_extension_at_max: false,
            },
            ItemReferenceSlotFrameV1 {
                field: "ModelPart3".to_owned(),
                label: "Top".to_owned(),
                token: "t".to_owned(),
                model_resref: "wswls_t_023".to_owned(),
                model_sha256: "5".repeat(64),
                controller_node_name: "g_WSwLs_t_023".to_owned(),
                controller_translation: [0.0000303633, 0.262925, 0.0018867],
                controller_rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
                bounds_min: [-0.010220837, 0.1283190, -0.0460687],
                bounds_max: [0.010273363, 0.9313860, 0.0463833],
                allow_axial_extension_at_min: false,
                allow_axial_extension_at_max: true,
            },
        ],
        profile_sha256: String::new(),
    };
    profile.profile_sha256 = m2a_core::item::item_attachment_profile_sha256_v1(&profile).unwrap();
    profile
}

#[test]
fn reference_slot_fit_preserves_the_retail_wswls_origin_and_slot_ranges() {
    let sources = [
        build_synthetic_glb::rectangular_prism(0.40, 0.42, 1.90),
        build_synthetic_glb::rectangular_prism(1.90, 0.44, 0.48),
        build_synthetic_glb::rectangular_prism(0.30, 0.12, 0.90),
    ];
    let inputs = ["ModelPart1", "ModelPart2", "ModelPart3"]
        .into_iter()
        .zip(["wswls_b_251", "wswls_m_251", "wswls_t_251"])
        .zip(&sources)
        .map(|((field, model_resref), source_glb)| ItemFitSourceV1 {
            field,
            model_resref,
            source_glb,
            source_node: None,
        })
        .collect::<Vec<_>>();
    let profile = retail_wswls_profile();

    let report = fit_meshy_item_parts_to_attachment_profile_v1(&inputs, 0.01, &profile).unwrap();

    assert_eq!(report.schema_version, 4);
    assert_eq!(report.algorithm, "ITEM_REFERENCE_SLOT_FRAME_FIT_V1");
    assert_eq!(report.status, "PASSED", "{report:#?}");
    assert_eq!(report.reference_profile_sha256, profile.profile_sha256);
    for (part, slot) in report.parts.iter().zip(&profile.slots) {
        assert!((part.output_bounds_min[1] - slot.bounds_min[1]).abs() <= 1.0e-5);
        assert!((part.output_bounds_max[1] - slot.bounds_max[1]).abs() <= 1.0e-5);
        assert!(part.output_bounds_min[0] >= slot.bounds_min[0] - 1.0e-5);
        assert!(part.output_bounds_max[0] <= slot.bounds_max[0] + 1.0e-5);
        assert!(part.output_bounds_min[2] >= slot.bounds_min[2] - 1.0e-5);
        assert!(part.output_bounds_max[2] <= slot.bounds_max[2] + 1.0e-5);
        assert_eq!(part.target_space_scale_xyz[1], 1.0);
        assert_eq!(
            part.target_space_scale_xyz[0],
            part.target_space_scale_xyz[2]
        );
    }
    assert!(report.parts[1].target_space_scale_xyz[0] < 1.0);
    assert!(report.parts[0].output_bounds_min[1] < 0.0);
    assert!(report.parts[0].output_bounds_max[1] > 0.0);
    assert!(report.parts[1].output_bounds_min[1] > 0.0);
    assert!(report.parts[0].output_bounds_max[1] > report.parts[1].output_bounds_min[1]);
    assert!(report.parts[1].output_bounds_max[1] > report.parts[2].output_bounds_min[1]);
    validate_item_fit_report_v4(&report).unwrap();
    validate_item_fit_report_v4_against_profile_v1(&report, &profile).unwrap();
}

#[test]
fn reference_slot_fit_validates_a_longer_top_without_moving_its_hand_side_connector() {
    let sources = [
        build_synthetic_glb::rectangular_prism(0.40, 0.42, 1.90),
        build_synthetic_glb::rectangular_prism(1.90, 0.44, 0.48),
        build_synthetic_glb::rectangular_prism(0.30, 0.12, 0.90),
    ];
    let inputs = ["ModelPart1", "ModelPart2", "ModelPart3"]
        .into_iter()
        .zip(["wswls_b_251", "wswls_m_251", "wswls_t_251"])
        .zip(&sources)
        .map(|((field, model_resref), source_glb)| ItemFitSourceV1 {
            field,
            model_resref,
            source_glb,
            source_node: None,
        })
        .collect::<Vec<_>>();
    let profile = retail_wswls_profile();

    let report = fit_meshy_item_parts_to_attachment_profile_with_axial_scales_v1(
        &inputs,
        0.01,
        &profile,
        &[1.0, 1.0, 1.5],
    )
    .unwrap();

    let reference_top = &profile.slots[2];
    let fitted_top = &report.parts[2];
    let reference_length = reference_top.bounds_max[1] - reference_top.bounds_min[1];
    assert!((fitted_top.output_bounds_min[1] - reference_top.bounds_min[1]).abs() <= 1.0e-5);
    assert!(
        (fitted_top.output_bounds_max[1] - (reference_top.bounds_min[1] + reference_length * 1.5))
            .abs()
            <= 1.0e-5
    );
    assert!((fitted_top.target_axial_length - reference_length * 1.5).abs() <= 1.0e-5);
    assert_eq!(report.status, "PASSED", "{report:#?}");
    validate_item_fit_report_v4_against_profile_v1(&report, &profile).unwrap();
}

#[test]
fn reference_slot_fit_rejects_scaling_a_slot_without_an_extension_policy() {
    let sources = [
        build_synthetic_glb::rectangular_prism(0.40, 0.42, 1.90),
        build_synthetic_glb::rectangular_prism(1.90, 0.44, 0.48),
        build_synthetic_glb::rectangular_prism(0.30, 0.12, 0.90),
    ];
    let inputs = ["ModelPart1", "ModelPart2", "ModelPart3"]
        .into_iter()
        .zip(["wswls_b_251", "wswls_m_251", "wswls_t_251"])
        .zip(&sources)
        .map(|((field, model_resref), source_glb)| ItemFitSourceV1 {
            field,
            model_resref,
            source_glb,
            source_node: None,
        })
        .collect::<Vec<_>>();
    let profile = retail_wswls_profile();

    let error = fit_meshy_item_parts_to_attachment_profile_with_axial_scales_v1(
        &inputs,
        0.01,
        &profile,
        &[1.25, 1.0, 1.0],
    )
    .unwrap_err();

    assert_eq!(error.code, "ITEM-FIT-REFERENCE-SCALE-NOT-ALLOWED");
    assert_eq!(error.path, "axialScaleFactors[0]");
}

#[test]
fn reference_profile_rejects_the_old_v7_cursor_frame() {
    let profile = retail_wswls_profile();
    let sources = [
        build_synthetic_glb::rectangular_prism(0.40, 0.42, 1.90),
        build_synthetic_glb::rectangular_prism(1.90, 0.44, 0.48),
        build_synthetic_glb::rectangular_prism(0.30, 0.12, 0.90),
    ];
    let inputs = ["ModelPart1", "ModelPart2", "ModelPart3"]
        .into_iter()
        .zip(["wswls_b_251", "wswls_m_251", "wswls_t_251"])
        .zip(&sources)
        .map(|((field, model_resref), source_glb)| ItemFitSourceV1 {
            field,
            model_resref,
            source_glb,
            source_node: None,
        })
        .collect::<Vec<_>>();
    let old = m2a_core::item::fit_meshy_item_parts_with_target_lengths_aurora_v5(
        &inputs,
        0.01,
        &[0.22, 0.08, 0.90],
    )
    .unwrap();

    let error =
        m2a_core::item::validate_item_fit_report_v3_against_profile_v1(&old, &profile).unwrap_err();
    assert_eq!(error.code, "ITEM-FIT-REFERENCE-SLOT-BOUNDS-MISMATCH");
}

#[test]
fn profile_builder_binds_exact_reference_models_and_hashes() {
    let models = [
        ("ModelPart1", "wswls_b_023"),
        ("ModelPart2", "wswls_m_063"),
        ("ModelPart3", "wswls_t_023"),
    ]
    .map(|(field, resref)| (field, resref, reference_mdl(resref)));
    let inputs = models
        .iter()
        .map(|(field, resref, payload)| ItemReferenceMdlInputV1 {
            field,
            model_resref: resref,
            mdl_payload: payload,
        })
        .collect::<Vec<_>>();
    let baseitems = br#"2DA V2.0

Label ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight
1 longsword WSwLs 2 0 it_bag iwswls 0x1C030 1 4
"#;
    let row = m2a_core::item::resolve_item_baseitem_v1(baseitems, 1).unwrap();

    let profile = build_item_attachment_profile_v1(
        &row,
        &"a".repeat(64),
        &"b".repeat(64),
        "EXPLICIT_VARIANTS",
        "23/63/23",
        &inputs,
    )
    .unwrap();

    assert_eq!(profile.slots.len(), 3);
    assert!(
        profile
            .slots
            .iter()
            .all(|slot| slot.model_sha256.len() == 64)
    );
    assert_eq!(profile.attachment_route, ItemAttachmentRouteV1::Hand);
    assert_eq!(profile.profile_sha256.len(), 64);
    assert_eq!(
        profile.profile_sha256,
        item_profile_roundtrip_hash(&profile),
    );
}

fn item_profile_roundtrip_hash(profile: &ItemAttachmentProfileV1) -> String {
    let json = serde_json::to_string(profile).unwrap();
    let decoded: ItemAttachmentProfileV1 = serde_json::from_str(&json).unwrap();
    m2a_core::item::item_attachment_profile_sha256_v1(&decoded).unwrap()
}

#[test]
fn env_gated_retail_wswls_profile_is_derived_from_the_selected_key_bif_context() {
    let Some(root) = std::env::var_os("M2A_NWN_INSTALL_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let key_path = root.join("data").join("nwn_base.key");
    let key = std::fs::read(&key_path).expect("read-only retail nwn_base.key");

    let base_locator = locate_key_bif_resource_v1("nwn_base.key", &key, "baseitems", 2017)
        .expect("retail baseitems KEY locator");
    let base_bif_path = root.join(base_locator.bif_logical_name.replace('/', "\\"));
    let base_bif = std::fs::read(&base_bif_path).expect("read-only retail baseitems BIF");
    let base_bif_input = KeyBifFileInputV1 {
        logical_name: base_locator.bif_logical_name.clone(),
        bytes: &base_bif,
    };
    let baseitems = resolve_key_bif_resource_sparse_v1(
        "nwn_base.key",
        &key,
        &base_bif_input,
        "baseitems",
        2017,
    )
    .expect("retail baseitems payload");
    let row =
        m2a_core::item::resolve_item_baseitem_v1(baseitems.payload, 1).expect("retail BaseItem 1");

    let model_locator = locate_key_bif_resource_v1("nwn_base.key", &key, "wswls_b_023", 2002)
        .expect("retail WSwLs model KEY locator");
    let model_bif_path = root.join(model_locator.bif_logical_name.replace('/', "\\"));
    let model_bif = std::fs::read(&model_bif_path).expect("read-only retail model BIF");
    let model_bif_input = KeyBifFileInputV1 {
        logical_name: model_locator.bif_logical_name.clone(),
        bytes: &model_bif,
    };
    let resolved = [
        ("ModelPart1", "wswls_b_023"),
        ("ModelPart2", "wswls_m_063"),
        ("ModelPart3", "wswls_t_023"),
    ]
    .map(|(field, resref)| {
        let resource = resolve_key_bif_resource_sparse_v1(
            "nwn_base.key",
            &key,
            &model_bif_input,
            resref,
            2002,
        )
        .expect("retail WSwLs MDL");
        (field, resref, resource)
    });
    let context_sha256 = m2a_core::item::item_payload_sha256_v1(
        format!(
            "{}:{}:{}",
            baseitems.context_sha256, resolved[0].2.context_sha256, model_locator.key_sha256,
        )
        .as_bytes(),
    );
    let inputs = resolved
        .iter()
        .map(|(field, resref, resource)| ItemReferenceMdlInputV1 {
            field,
            model_resref: resref,
            mdl_payload: resource.payload,
        })
        .collect::<Vec<_>>();
    let profile = build_item_attachment_profile_v1(
        &row,
        &context_sha256,
        &baseitems.payload_sha256,
        "RETAIL_P_REF",
        "WSwLs 23/63/23",
        &inputs,
    )
    .expect("retail WSwLs attachment profile");

    let expected_y = [
        [-0.2025382, 0.0841520],
        [0.042125102, 0.1570407],
        [0.1283190, 0.9313860],
    ];
    for (slot, expected) in profile.slots.iter().zip(expected_y) {
        assert!(
            (slot.bounds_min[1] - expected[0]).abs() <= 1.0e-5,
            "{} Y min {} != {} ({profile:#?})",
            slot.field,
            slot.bounds_min[1],
            expected[0],
        );
        assert!(
            (slot.bounds_max[1] - expected[1]).abs() <= 1.0e-5,
            "{} Y max {} != {} ({profile:#?})",
            slot.field,
            slot.bounds_max[1],
            expected[1],
        );
    }
    assert_eq!(profile.attachment_route, ItemAttachmentRouteV1::Hand);
    assert_eq!(profile.common_origin, [0.0, 0.0, 0.0]);
}
