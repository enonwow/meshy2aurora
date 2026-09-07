use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
use m2a_core::{
    AURORA_MODEL_TRIANGLE_BUDGET_V1, AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1,
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1,
    aurora_material::AuroraMaterialTargetProfileV1,
    erf::ErfArchive,
    gff::{GffLimitsV1, GffValueV1, read_gff_v32},
    glb::{GlbLimits, ingest_glb},
    mdl::MdlMaterialTextureBindingV1,
    model_components::SourceComponentKeyV1,
    model_material_separation::{
        AuthoredMaterialV1, ModelMaterialAssignmentV1, ModelMaterialFaceAssignmentV2,
        ModelMaterialSeparationDocumentV1, ModelMaterialSeparationDocumentV2,
        SourceFaceSelectionV2, SourceTriangleRangeV2, resolve_model_materials_v1,
        resolve_model_materials_v2,
    },
    model_material_uv_projection::{
        ModelMaterialUvProjectionDocumentV1, ModelMaterialUvProjectionModeV1,
        ModelMaterialUvProjectionRuleV1,
    },
    model_texture_authoring::{
        ModelTextureBindingModeV1, ModelTexturePayloadDescriptorV1,
        default_model_texture_authoring_v1,
    },
    mtr::{MTR_RESOURCE_TYPE_V1, parse_mtr_v1},
    owned_fixture::synthetic_owned_m6_glb_v1,
    placeable::{
        ARE_RESOURCE_TYPE, GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE, IFO_RESOURCE_TYPE,
        ITP_RESOURCE_TYPE, MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE, PWK_RESOURCE_TYPE,
        PlaceablePlacementV1, PlaceableTextureInputV1, StaticPlaceableBlueprintV1,
        StaticPlaceableBuildRequestV1, StaticPlaceableIdentityV1, UTP_RESOURCE_TYPE,
        append_static_placeable_2da_v1, build_meshy_static_placeable_package_v1,
        build_meshy_static_placeable_package_v2, build_meshy_static_placeable_package_v4,
        build_meshy_static_placeable_package_v5, build_meshy_static_placeable_package_v6,
        build_meshy_static_placeable_package_v7, build_meshy_static_placeable_package_v8,
        build_meshy_static_placeable_package_v9, build_static_placeable_package_v1,
        inspect_meshy_static_placeable_authoring_v1, inspect_meshy_static_placeable_authoring_v3,
        inspect_meshy_static_placeable_textures_v1, resolve_meshy_static_placeable_collision_v1,
        resolve_meshy_static_placeable_textures_v1, static_placeable_glb_limits_v1,
        static_placeable_profile_a_options_v1, write_placeable_palette_itp_v1,
        write_static_placeable_utp_v1,
    },
    placeable_authoring::{
        PlaceableAuthoringDocumentV2, PlaceableCollisionCoordinateSpaceV1,
        PlaceableCollisionModeV1, PlaceableCollisionSpecV1, PlaceableElementKindV1,
    },
    placeable_texture::{
        PlaceableTextureAlphaPolicyV1, PlaceableTextureBindingModeV1,
        PlaceableTexturePayloadDescriptorV1,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, read_two_da_row_v2},
    txi::{TXI_RESOURCE_TYPE_V1, parse_txi_v1},
};
use std::{env, fs};

#[path = "fixtures/build_synthetic_glb.rs"]
mod build_synthetic_glb;

fn base_placeables_2da() -> Vec<u8> {
    let columns = [
        "Label",
        "StrRef",
        "ModelName",
        "LightColor",
        "LightOffsetX",
        "LightOffsetY",
        "LightOffsetZ",
        "SoundAppType",
        "ShadowSize",
        "BodyBag",
        "LowGore",
        "Reflection",
        "Static",
    ];
    let row = |label: &str, model: &str| {
        [
            label, "****", model, "****", "****", "****", "****", "****", "1", "0", "****", "****",
            "1",
        ]
        .join(" ")
    };
    format!(
        "2DA V2.0\n\n{}\n0 {}\n1 {}\n2 {}\n",
        columns.join(" "),
        row("ARMOIRE", "plc_a01"),
        row("OS_RESERVED", "****"),
        row("ACTIVE_AFTER_RESERVED", "plc_b08"),
    )
    .into_bytes()
}

#[test]
fn placeable_v9_packages_ee_materials_with_semantic_readback() {
    let source = build_synthetic_glb::mutate_json(static_source_glb(), |root| {
        root["materials"][0]["pbrMetallicRoughness"]
            .as_object_mut()
            .expect("PBR material")
            .remove("baseColorTexture");
    });
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .expect("V9 authoring")
        .document;
    let separation = m2a_core::model_material_separation::default_model_material_separation_v2(
        &ingest_glb(&source, &GlbLimits::default())
            .expect("ingest")
            .ir,
    );
    let artifact = build_meshy_static_placeable_package_v9(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
        &separation,
        None,
        None,
        &[],
        &[],
        AuroraMaterialTargetProfileV1::NwnEeMtr,
        &Default::default(),
    )
    .expect("complete Placeable V9");

    assert_eq!(artifact.report.profile, "STATIC_PLACEABLE_V9_NWN_EE_MTR");
    assert!(artifact.report.material_compilation.is_some());
    assert!(artifact.report.source_quality.is_some());
    assert_eq!(
        artifact.report.material_semantic_readback_status.as_deref(),
        Some("PASS")
    );
    let hak = ErfArchive::parse(&artifact.hak_payload).expect("HAK");
    let mtr = artifact
        .report
        .resources
        .iter()
        .find(|item| item.resource_type == MTR_RESOURCE_TYPE_V1)
        .expect("MTR resource");
    parse_mtr_v1(
        hak.find(&mtr.resref, mtr.resource_type)
            .expect("MTR payload"),
    )
    .expect("MTR readback");
    let txi = artifact
        .report
        .resources
        .iter()
        .find(|item| item.resource_type == TXI_RESOURCE_TYPE_V1)
        .expect("TXI resource");
    parse_txi_v1(
        hak.find(&txi.resref, txi.resource_type)
            .expect("TXI payload"),
    )
    .expect("TXI readback");
}

#[test]
fn placeable_v9_classic_profile_fails_closed_for_two_sided_material() {
    let source = build_synthetic_glb::mutate_json(static_source_glb(), |root| {
        root["materials"][0]["doubleSided"] = serde_json::json!(true);
    });
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .expect("V9 authoring")
        .document;
    let separation = m2a_core::model_material_separation::default_model_material_separation_v2(
        &ingest_glb(&source, &GlbLimits::default())
            .expect("ingest")
            .ir,
    );
    let error = build_meshy_static_placeable_package_v9(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
        &separation,
        None,
        None,
        &[],
        &[],
        AuroraMaterialTargetProfileV1::AuroraClassicSafe,
        &Default::default(),
    )
    .expect_err("classic profile must reject unsupported two-sided output");

    assert_eq!(error.code, "AURORA-MATERIAL-PACKAGE-COMPILER-BLOCKED");
}

#[test]
fn placeable_v9_source_quality_gate_blocks_unreadable_texture_density() {
    let source = static_source_glb();
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .expect("V9 authoring")
        .document;
    let separation = m2a_core::model_material_separation::default_model_material_separation_v2(
        &ingest_glb(&source, &GlbLimits::default())
            .expect("ingest")
            .ir,
    );
    let error = build_meshy_static_placeable_package_v9(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
        &separation,
        None,
        None,
        &[],
        &[],
        AuroraMaterialTargetProfileV1::NwnEeMtr,
        &Default::default(),
    )
    .expect_err("unreadable source texture density must block V9");

    assert_eq!(error.code, "AURORA-MATERIAL-PACKAGE-SOURCE-QUALITY-BLOCKED");
}

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
            name: "m2a_plc_ped".to_owned(),
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

fn identity() -> StaticPlaceableIdentityV1 {
    StaticPlaceableIdentityV1 {
        module_resref: "m2a_plc_mod".to_owned(),
        module_file_name: "m2a_plc_mod.mod".to_owned(),
        module_display_name: "Meshy2Aurora placeable proof".to_owned(),
        area_resref: "m2a_plc_area".to_owned(),
        area_name: "Meshy2Aurora placeable area".to_owned(),
        hak_resref: "m2a_plc_hak".to_owned(),
        hak_file_name: "m2a_plc_hak.hak".to_owned(),
        model_resref: "m2a_plc_ped".to_owned(),
        texture_resref: "m2a_plc_tex".to_owned(),
        blueprint_resref: "m2a_plc_utp".to_owned(),
        object_tag: "m2a_plc_pedestal".to_owned(),
        display_name: "Meshy Ritual Pedestal".to_owned(),
    }
}

fn static_request() -> StaticPlaceableBuildRequestV1 {
    StaticPlaceableBuildRequestV1 {
        schema_version: 1,
        identity: identity(),
        placement: PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        palette_id: 7,
        base_placeables_2da: base_placeables_2da(),
        model: static_model(),
        collision_model: None,
        authoring_report: None,
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_plc_tex".to_owned(),
        }],
        textures: vec![PlaceableTextureInputV1 {
            resref: "m2a_plc_tex".to_owned(),
            resource_type: 3,
            payload: b"synthetic-owned-tga-fixture".to_vec(),
        }],
    }
}

fn static_source_glb() -> Vec<u8> {
    build_synthetic_glb::mutate_json(
        synthetic_owned_m6_glb_v1().expect("owned GLB fixture"),
        |root| {
            root["skins"] = serde_json::json!([]);
            root["animations"] = serde_json::json!([]);
            root["scenes"][0]["nodes"] = serde_json::json!([0]);
            root["nodes"] = serde_json::json!([{
                "name": "placeable-source-root",
                "mesh": 0
            }]);
            let attributes = root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .expect("synthetic primitive attributes");
            attributes.remove("JOINTS_0");
            attributes.remove("WEIGHTS_0");
        },
    )
}

fn multi_material_static_source_glb() -> Vec<u8> {
    build_synthetic_glb::mutate_json(
        build_synthetic_glb::material_image_two_distinct_base_colors(),
        |root| {
            for material in root["materials"].as_array_mut().expect("fixture materials") {
                material["alphaMode"] = serde_json::json!("OPAQUE");
                material.as_object_mut().unwrap().remove("alphaCutoff");
            }
        },
    )
}

fn multi_material_shared_image_static_source_glb() -> Vec<u8> {
    build_synthetic_glb::mutate_json(multi_material_static_source_glb(), |root| {
        root["materials"][1]["pbrMetallicRoughness"]["baseColorTexture"]["index"] =
            serde_json::json!(0);
    })
}

fn owned_rgb_png(rgb: [u8; 3]) -> Vec<u8> {
    let mut payload = Vec::new();
    PngEncoder::new(&mut payload)
        .write_image(&rgb, 1, 1, ColorType::Rgb8.into())
        .expect("encode owned RGB PNG");
    payload
}

fn owned_rgba_png(rgba: [u8; 4]) -> Vec<u8> {
    let mut payload = Vec::new();
    PngEncoder::new(&mut payload)
        .write_image(&rgba, 1, 1, ColorType::Rgba8.into())
        .expect("encode owned RGBA PNG");
    payload
}

#[test]
fn meshy_glb_uses_the_shared_ingest_profile_and_model_pipeline() {
    let source = static_source_glb();
    let artifact = build_meshy_static_placeable_package_v1(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
    )
    .expect("build placeable directly from GLB");

    assert_eq!(artifact.report.source_model_sha256, hex_sha256(&source));
    let hak = ErfArchive::parse(&artifact.hak_payload).expect("read HAK");
    assert!(hak.find("m2a_plc_ped", MDL_RESOURCE_TYPE).is_ok());
    assert!(hak.find("m2a_plc_tex", 3).is_ok());
    let mdl = m2a_core::mdl::inspect_binary_mdl(
        hak.find("m2a_plc_ped", MDL_RESOURCE_TYPE)
            .expect("read model"),
    )
    .expect("inspect model");
    assert_eq!(mdl.node_tree.roots[0].name, "m2a_plc_ped");
    assert!(mdl.node_tree.node_count >= 1);
    assert_eq!(mdl.animations.len(), 0);
}

#[test]
fn meshy_glb_preserves_distinct_base_color_textures_per_material_slot() {
    let source = multi_material_static_source_glb();
    let artifact = build_meshy_static_placeable_package_v1(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
    )
    .expect("build two-material placeable");

    let hak = ErfArchive::parse(&artifact.hak_payload).expect("read HAK");
    assert!(hak.find("m2a_plc_tex", 3).is_ok());
    assert!(hak.find("m2a_plc_tex_m1", 3).is_ok());
    assert_eq!(artifact.report.texture_resources.len(), 2);
    assert_eq!(artifact.report.texture_resources[0].material_slots, vec![0]);
    assert_eq!(artifact.report.texture_resources[1].material_slots, vec![1]);
    assert_ne!(
        artifact.report.texture_resources[0].sha256,
        artifact.report.texture_resources[1].sha256
    );

    let mdl = m2a_core::mdl::inspect_binary_mdl(
        hak.find("m2a_plc_ped", MDL_RESOURCE_TYPE)
            .expect("read model"),
    )
    .expect("inspect model");
    let mut diffuse = mdl.node_tree.roots[0]
        .children
        .iter()
        .filter_map(|node| node.mesh.as_ref())
        .map(|mesh| mesh.textures[0].clone())
        .collect::<Vec<_>>();
    diffuse.sort();
    diffuse.dedup();
    assert_eq!(diffuse, vec!["m2a_plc_tex", "m2a_plc_tex_m1"]);
}

#[test]
fn meshy_glb_deduplicates_a_shared_base_color_image_across_material_slots() {
    let source = multi_material_shared_image_static_source_glb();
    let artifact = build_meshy_static_placeable_package_v1(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
    )
    .expect("build shared-image placeable");

    assert_eq!(artifact.report.texture_resources.len(), 1);
    assert_eq!(
        artifact.report.texture_resources[0].material_slots,
        vec![0, 1]
    );
    assert_eq!(artifact.report.hak_resource_count, 4);
    let hak = ErfArchive::parse(&artifact.hak_payload).expect("read HAK");
    assert!(hak.find("m2a_plc_tex", 3).is_ok());
    assert!(hak.find("m2a_plc_tex_m1", 3).is_err());
}

#[test]
fn placeable_texture_override_resolves_per_material_and_builds_the_exact_hak_tga() {
    let source = multi_material_static_source_glb();
    let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1::default();
    let bootstrap = inspect_meshy_static_placeable_textures_v1(&source, &options)
        .expect("inspect placeable materials");
    assert_eq!(bootstrap.inspection.materials.len(), 2);
    assert_eq!(bootstrap.document.bindings.len(), 2);
    assert_eq!(bootstrap.inspection.materials[0].material_slot, 0);
    assert_eq!(bootstrap.inspection.materials[1].material_slot, 1);
    assert!(
        bootstrap
            .inspection
            .materials
            .iter()
            .all(|material| material.has_uv0)
    );

    let override_png = owned_rgb_png([17, 91, 203]);
    let override_sha256 = hex_sha256(&override_png);
    let mut texture_authoring = bootstrap.document;
    let binding = &mut texture_authoring.bindings[1];
    binding.mode = PlaceableTextureBindingModeV1::Override;
    binding.override_asset_id = Some("slot-1-blue".to_owned());
    binding.override_sha256 = Some(override_sha256.clone());
    binding.override_mime_type = Some("image/png".to_owned());
    binding.override_byte_length = Some(override_png.len() as u64);
    binding.alpha_policy = PlaceableTextureAlphaPolicyV1::OpaqueOnly;
    let descriptors = vec![PlaceableTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "slot-1-blue".to_owned(),
        sha256: override_sha256,
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: override_png.len() as u64,
    }];
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &options)
        .expect("inspect placeable authoring")
        .document;

    let resolved = resolve_meshy_static_placeable_textures_v1(
        &source,
        &identity().texture_resref,
        &authoring,
        &texture_authoring,
        &override_png,
        &descriptors,
        &options,
    )
    .expect("resolve texture override");
    assert_eq!(resolved.bindings.len(), 2);
    assert_eq!(
        resolved.bindings[0].mode,
        PlaceableTextureBindingModeV1::Source
    );
    assert_eq!(
        resolved.bindings[1].mode,
        PlaceableTextureBindingModeV1::Override
    );
    assert_eq!(resolved.resources.len(), 2);
    assert_eq!(resolved.bindings[1].source_alpha_mode, "OPAQUE");
    assert!(
        resolved.bindings[1]
            .ignored_source_pbr_maps
            .contains(&"normalTexture".to_owned())
    );

    let artifact = build_meshy_static_placeable_package_v5(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
        &texture_authoring,
        &override_png,
        &descriptors,
        &options,
    )
    .expect("build authored placeable with texture override");
    assert_eq!(artifact.report.texture_authoring.as_ref(), Some(&resolved));
    let hak = ErfArchive::parse(&artifact.hak_payload).expect("read HAK");
    for resource in &resolved.resources {
        let payload = hak
            .find(&resource.resref, resource.resource_type)
            .expect("resolved texture in HAK");
        assert_eq!(hex_sha256(payload), resource.sha256);
    }
}

#[test]
fn placeable_texture_override_rejects_nonopaque_alpha_malformed_image_and_stale_binding() {
    let source = multi_material_static_source_glb();
    let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1::default();
    let mut authoring = inspect_meshy_static_placeable_textures_v1(&source, &options)
        .expect("inspect placeable materials")
        .document;
    let geometry_authoring = inspect_meshy_static_placeable_authoring_v3(&source, &options)
        .expect("inspect placeable authoring")
        .document;
    let override_png = owned_rgba_png([255, 0, 0, 96]);
    let descriptor = PlaceableTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "slot-0-alpha".to_owned(),
        sha256: hex_sha256(&override_png),
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: override_png.len() as u64,
    };
    let binding = &mut authoring.bindings[0];
    binding.mode = PlaceableTextureBindingModeV1::Override;
    binding.override_asset_id = Some(descriptor.asset_id.clone());
    binding.override_sha256 = Some(descriptor.sha256.clone());
    binding.override_mime_type = Some(descriptor.mime_type.clone());
    binding.override_byte_length = Some(descriptor.byte_length);
    let error = resolve_meshy_static_placeable_textures_v1(
        &source,
        &identity().texture_resref,
        &geometry_authoring,
        &authoring,
        &override_png,
        std::slice::from_ref(&descriptor),
        &options,
    )
    .expect_err("nonopaque alpha must fail closed");
    assert_eq!(error.code, "PLACEABLE-TEXTURE-ALPHA-UNSUPPORTED");

    let malformed_png = b"\x89PNG\r\n\x1a\nnot-a-valid-png".to_vec();
    let malformed_descriptor = PlaceableTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "slot-0-malformed".to_owned(),
        sha256: hex_sha256(&malformed_png),
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: malformed_png.len() as u64,
    };
    let binding = &mut authoring.bindings[0];
    binding.override_asset_id = Some(malformed_descriptor.asset_id.clone());
    binding.override_sha256 = Some(malformed_descriptor.sha256.clone());
    binding.override_mime_type = Some(malformed_descriptor.mime_type.clone());
    binding.override_byte_length = Some(malformed_descriptor.byte_length);
    let error = resolve_meshy_static_placeable_textures_v1(
        &source,
        &identity().texture_resref,
        &geometry_authoring,
        &authoring,
        &malformed_png,
        std::slice::from_ref(&malformed_descriptor),
        &options,
    )
    .expect_err("malformed PNG must fail closed");
    assert_eq!(error.code, "PLACEABLE-TEXTURE-DECODE-FAILED");

    authoring.bindings[0].source_image_sha256 = "0".repeat(64);
    let error = resolve_meshy_static_placeable_textures_v1(
        &source,
        &identity().texture_resref,
        &geometry_authoring,
        &authoring,
        &malformed_png,
        &[malformed_descriptor],
        &options,
    )
    .expect_err("stale source image binding must fail closed");
    assert_eq!(error.code, "PLACEABLE-TEXTURE-SOURCE-BINDING-STALE");
}

#[test]
fn placeable_texture_resolution_tracks_only_materials_retained_by_geometry_authoring() {
    let source = multi_material_static_source_glb();
    let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1::default();
    let texture_authoring = inspect_meshy_static_placeable_textures_v1(&source, &options)
        .expect("inspect placeable materials")
        .document;
    let mut geometry_authoring = inspect_meshy_static_placeable_authoring_v3(&source, &options)
        .expect("inspect placeable authoring")
        .document;
    let inspection = inspect_meshy_static_placeable_authoring_v3(&source, &options)
        .expect("inspect placeable authoring components")
        .inspection;
    let template = geometry_authoring.elements[0].clone();
    geometry_authoring.elements = inspection.nodes[0]
        .primitives
        .iter()
        .flat_map(|primitive| {
            let template = template.clone();
            primitive.components.iter().map(move |component| {
                let mut element = template.clone();
                element.id = format!(
                    "node-0-p{}-c{}",
                    primitive.primitive_id, component.component_index
                );
                element.kind = PlaceableElementKindV1::SourceComponent;
                let selector = element.source.as_mut().expect("source element");
                selector.primitive_id = Some(primitive.primitive_id);
                selector.component_index = Some(component.component_index);
                element.deleted = primitive.primitive_id != 0;
                element
            })
        })
        .collect();

    let resolved = resolve_meshy_static_placeable_textures_v1(
        &source,
        &identity().texture_resref,
        &geometry_authoring,
        &texture_authoring,
        &[],
        &[],
        &options,
    )
    .expect("resolve only retained material");
    assert_eq!(resolved.bindings.len(), 1);
    assert_eq!(resolved.bindings[0].material_slot, 0);
    assert_eq!(resolved.resources.len(), 1);
}

#[test]
fn placeable_texture_override_rejects_source_cutout_without_txi_contract() {
    let source = build_synthetic_glb::mutate_json(multi_material_static_source_glb(), |root| {
        root["materials"][0]["alphaMode"] = serde_json::json!("MASK");
        root["materials"][0]["alphaCutoff"] = serde_json::json!(0.5);
    });
    let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1::default();
    let mut texture_authoring = inspect_meshy_static_placeable_textures_v1(&source, &options)
        .expect("inspect cutout material")
        .document;
    let geometry_authoring = inspect_meshy_static_placeable_authoring_v3(&source, &options)
        .expect("inspect geometry authoring")
        .document;
    let override_png = owned_rgb_png([9, 19, 29]);
    let descriptor = PlaceableTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "cutout-override".to_owned(),
        sha256: hex_sha256(&override_png),
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: override_png.len() as u64,
    };
    let binding = &mut texture_authoring.bindings[0];
    binding.mode = PlaceableTextureBindingModeV1::Override;
    binding.override_asset_id = Some(descriptor.asset_id.clone());
    binding.override_sha256 = Some(descriptor.sha256.clone());
    binding.override_mime_type = Some(descriptor.mime_type.clone());
    binding.override_byte_length = Some(descriptor.byte_length);

    let error = resolve_meshy_static_placeable_textures_v1(
        &source,
        &identity().texture_resref,
        &geometry_authoring,
        &texture_authoring,
        &override_png,
        &[descriptor],
        &options,
    )
    .expect_err("cutout override requires a future TXI contract");
    assert_eq!(error.code, "PLACEABLE-TEXTURE-ALPHA-MODE-UNSUPPORTED");
}

#[test]
fn authored_placeable_emits_independent_mesh_shadow_flags_and_collision_projection() {
    let source = static_source_glb();
    let mut authoring = inspect_meshy_static_placeable_authoring_v1(&source)
        .expect("inspect authoring")
        .document;
    let mut copy = authoring.elements[0].clone();
    copy.id = "shadowless-copy".to_owned();
    copy.name = "Shadowless copy".to_owned();
    copy.kind = PlaceableElementKindV1::Copy;
    copy.transform.translation = [2.0, 0.0, 0.0];
    copy.flags.cast_shadow = false;
    copy.flags.include_in_collision = false;
    authoring.elements.push(copy);

    let artifact = build_meshy_static_placeable_package_v2(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
    )
    .expect("authored placeable");
    let report = artifact
        .report
        .authoring
        .as_ref()
        .expect("authoring report");
    assert_eq!(report.renderable_element_count, 2);
    assert_eq!(report.collision_element_count, 1);
    assert_eq!(report.shadow_element_count, 1);

    let hak = ErfArchive::parse(&artifact.hak_payload).expect("read HAK");
    let mdl = m2a_core::mdl::inspect_binary_mdl(
        hak.find("m2a_plc_ped", MDL_RESOURCE_TYPE)
            .expect("read model"),
    )
    .expect("inspect model");
    let mut shadows = mdl.node_tree.roots[0]
        .children
        .iter()
        .filter_map(|node| node.mesh.as_ref().map(|mesh| mesh.shadow))
        .collect::<Vec<_>>();
    shadows.sort_unstable();
    assert_eq!(shadows, vec![0, 1]);
}

#[test]
fn authored_placeable_rejects_an_empty_collision_projection() {
    let source = static_source_glb();
    let mut authoring = inspect_meshy_static_placeable_authoring_v1(&source)
        .expect("inspect authoring")
        .document;
    for element in &mut authoring.elements {
        element.flags.include_in_collision = false;
    }

    let error = build_meshy_static_placeable_package_v2(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
    )
    .expect_err("empty collision projection must fail closed");
    assert_eq!(error.code, "PLACEABLE-COLLISION-EMPTY");
}

#[test]
fn authored_placeable_v2_custom_polygon_is_written_to_the_exact_hak_pwk() {
    let source = static_source_glb();
    let v1 = inspect_meshy_static_placeable_authoring_v1(&source)
        .expect("inspect authoring")
        .document;
    let authoring = PlaceableAuthoringDocumentV2 {
        schema_version: 2,
        source_sha256: v1.source_sha256,
        elements: v1.elements,
        collision: PlaceableCollisionSpecV1 {
            schema_version: 1,
            mode: PlaceableCollisionModeV1::CustomPolygon,
            coordinate_space: PlaceableCollisionCoordinateSpaceV1::GltfSourceXzMeters,
            padding_meters: 0.0,
            vertices: vec![
                [-0.5, -0.5],
                [0.5, -0.5],
                [0.5, 0.5],
                [0.0, 0.0],
                [-0.5, 0.5],
            ],
        },
    };
    let artifact = build_meshy_static_placeable_package_v4(
        &source,
        &base_placeables_2da(),
        &identity(),
        PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        },
        7,
        &authoring,
        &Default::default(),
    )
    .expect("build custom collision placeable");

    let collision = artifact
        .report
        .collision
        .as_ref()
        .expect("collision report");
    let preview = resolve_meshy_static_placeable_collision_v1(
        &source,
        &identity().model_resref,
        &authoring,
        &Default::default(),
    )
    .expect("resolve the same custom collision for Studio preview");
    assert_eq!(&preview, collision);
    assert_eq!(collision.mode, PlaceableCollisionModeV1::CustomPolygon);
    assert_eq!(collision.vertices.len(), 5);
    assert_eq!(collision.triangles.len(), 3);
    assert_eq!(collision.surface_id, 7);
    assert_eq!(
        collision.authoring_sha256,
        artifact.report.authoring.as_ref().unwrap().authoring_sha256
    );

    let hak = ErfArchive::parse(&artifact.hak_payload).expect("read HAK");
    let pwk = hak
        .find("m2a_plc_ped", PWK_RESOURCE_TYPE)
        .expect("custom PWK");
    assert_eq!(
        m2a_core::placeable_collision::sha256_hex(pwk),
        collision.pwk_sha256
    );
    let readback = m2a_core::placeable_collision::inspect_ascii_placeable_walkmesh_v1(pwk)
        .expect("read custom PWK");
    assert_eq!(readback.mesh_nodes[0].vertices.len(), 5);
    assert_eq!(readback.mesh_nodes[0].faces.len(), 3);
}

#[test]
fn authored_placeable_v2_auto_rectangle_preserves_the_legacy_pwk_bytes() {
    let source = static_source_glb();
    let identity = identity();
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let legacy = build_meshy_static_placeable_package_v1(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
    )
    .expect("legacy auto rectangle");
    let authoring = m2a_core::placeable::inspect_meshy_static_placeable_authoring_v3(
        &source,
        &Default::default(),
    )
    .expect("V2 authoring")
    .document;
    let v2 = build_meshy_static_placeable_package_v4(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &Default::default(),
    )
    .expect("V2 auto rectangle");
    let legacy_hak = ErfArchive::parse(&legacy.hak_payload).expect("legacy HAK");
    let v2_hak = ErfArchive::parse(&v2.hak_payload).expect("V2 HAK");
    assert_eq!(
        legacy_hak
            .find(&identity.model_resref, PWK_RESOURCE_TYPE)
            .expect("legacy PWK"),
        v2_hak
            .find(&identity.model_resref, PWK_RESOURCE_TYPE)
            .expect("V2 PWK"),
    );
    assert_eq!(legacy.report.pwk_sha256, v2.report.pwk_sha256);
    assert_eq!(
        v2.report.collision.as_ref().expect("collision").mode,
        PlaceableCollisionModeV1::AutoRectangle,
    );
}

#[test]
fn material_separated_placeable_writes_two_textures_and_preserves_pwk_bytes() {
    let source = multi_material_static_source_glb();
    let identity = identity();
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .expect("Placeable authoring")
        .document;
    let ingest = ingest_glb(&source, &GlbLimits::default()).expect("material source ingest");
    let fallback_sha = ingest.ir.images[0].sha256.clone();
    let material = |id: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: id.to_owned(),
        preview_color: "#806040".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: Some(fallback_sha.clone()),
    };
    let separation = ModelMaterialSeparationDocumentV1 {
        schema_version: 1,
        source_sha256: ingest.ir.source.sha256.clone(),
        materials: vec![material("material:sail"), material("material:wood")],
        assignments: vec![
            ModelMaterialAssignmentV1 {
                component: SourceComponentKeyV1 {
                    scene_id: 0,
                    node_id: 0,
                    primitive_id: 0,
                    component_index: 0,
                },
                authored_material_id: "material:wood".to_owned(),
            },
            ModelMaterialAssignmentV1 {
                component: SourceComponentKeyV1 {
                    scene_id: 0,
                    node_id: 0,
                    primitive_id: 1,
                    component_index: 0,
                },
                authored_material_id: "material:sail".to_owned(),
            },
        ],
    };
    let materials =
        resolve_model_materials_v1(&ingest.ir, &separation).expect("material separation");
    let mut texture_authoring =
        default_model_texture_authoring_v1(&ingest, &materials).expect("texture authoring");
    let override_png = owned_rgb_png([30, 50, 80]);
    let override_sha = hex_sha256(&override_png);
    let sail = texture_authoring
        .bindings
        .iter_mut()
        .find(|binding| binding.authored_material_id == "material:sail")
        .expect("sail binding");
    sail.mode = ModelTextureBindingModeV1::Override;
    sail.override_asset_id = Some("override:sail".to_owned());
    sail.override_sha256 = Some(override_sha.clone());
    sail.override_mime_type = Some("image/png".to_owned());
    sail.override_byte_length = Some(override_png.len() as u64);
    let descriptors = [ModelTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "override:sail".to_owned(),
        sha256: override_sha,
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: override_png.len() as u64,
    }];

    let baseline = build_meshy_static_placeable_package_v4(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &Default::default(),
    )
    .expect("baseline Placeable");
    let separated = build_meshy_static_placeable_package_v6(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &separation,
        &texture_authoring,
        &override_png,
        &descriptors,
        &Default::default(),
    )
    .expect("material-separated Placeable");
    let repeated = build_meshy_static_placeable_package_v6(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &separation,
        &texture_authoring,
        &override_png,
        &descriptors,
        &Default::default(),
    )
    .expect("repeated material-separated Placeable");

    assert_eq!(baseline.report.pwk_sha256, separated.report.pwk_sha256);
    assert_eq!(separated.hak_payload, repeated.hak_payload);
    assert_eq!(separated.module_payload, repeated.module_payload);
    assert_eq!(
        serde_json::to_vec(&separated.report).expect("first report"),
        serde_json::to_vec(&repeated.report).expect("repeated report")
    );
    let baseline_hak = ErfArchive::parse(&baseline.hak_payload).expect("baseline HAK");
    let separated_hak = ErfArchive::parse(&separated.hak_payload).expect("separated HAK");
    assert_eq!(
        baseline_hak
            .find(&identity.model_resref, PWK_RESOURCE_TYPE)
            .expect("baseline PWK"),
        separated_hak
            .find(&identity.model_resref, PWK_RESOURCE_TYPE)
            .expect("separated PWK")
    );
    assert_eq!(
        separated
            .report
            .material_separation
            .as_ref()
            .expect("separation report")
            .material_slots
            .len(),
        2
    );
    let texture_report = separated
        .report
        .model_texture_authoring
        .as_ref()
        .expect("model texture report");
    assert_eq!(texture_report.bindings.len(), 2);
    assert_eq!(texture_report.resources.len(), 2);
    assert!(
        texture_report
            .resources
            .iter()
            .all(|resource| separated_hak.find(&resource.resref, 3).is_ok())
    );
}

#[test]
fn face_mode_v2_placeable_preserves_every_triangle_and_pwk_bytes() {
    let source = multi_material_static_source_glb();
    let identity = identity();
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .expect("Placeable authoring")
        .document;
    let ingest = ingest_glb(&source, &GlbLimits::default()).expect("material source ingest");
    let fallback_sha = ingest.ir.images[0].sha256.clone();
    let material = |id: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: id.to_owned(),
        preview_color: "#806040".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: Some(fallback_sha.clone()),
    };
    let selection = |primitive_id, material_id: &str| ModelMaterialFaceAssignmentV2 {
        selection: SourceFaceSelectionV2 {
            scene_id: 0,
            node_id: 0,
            primitive_id,
            triangle_ranges: vec![SourceTriangleRangeV2 {
                start_triangle: 0,
                triangle_count: 1,
            }],
        },
        authored_material_id: material_id.to_owned(),
    };
    let separation = ModelMaterialSeparationDocumentV2 {
        schema_version: 2,
        source_sha256: ingest.ir.source.sha256.clone(),
        materials: vec![material("material:sail"), material("material:wood")],
        component_assignments: Vec::new(),
        face_assignments: vec![selection(0, "material:wood"), selection(1, "material:sail")],
    };
    let materials = resolve_model_materials_v2(&ingest.ir, &separation)
        .expect("Face Mode V2 material separation");
    let texture_authoring = default_model_texture_authoring_v1(&ingest, materials.projection_v1())
        .expect("texture authoring");
    let baseline = build_meshy_static_placeable_package_v4(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &Default::default(),
    )
    .expect("baseline Placeable");
    let separated = build_meshy_static_placeable_package_v7(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &separation,
        &texture_authoring,
        &[],
        &[],
        &Default::default(),
    )
    .expect("Face Mode V2 Placeable");

    assert_eq!(baseline.report.pwk_sha256, separated.report.pwk_sha256);
    assert_eq!(
        separated
            .report
            .material_separation
            .as_ref()
            .expect("V2 projection report")
            .schema_version,
        2
    );
    assert_eq!(
        separated
            .report
            .material_separation
            .as_ref()
            .expect("V2 projection report")
            .source_triangle_count,
        separated
            .report
            .material_separation
            .as_ref()
            .expect("V2 projection report")
            .output_triangle_count,
    );
}

#[test]
fn material_box_uv_projection_changes_only_the_separated_render_material() {
    let source = multi_material_static_source_glb();
    let identity = identity();
    let placement = PlaceablePlacementV1 {
        x: 10.0,
        y: 14.5,
        z: 0.0,
        bearing: 0.0,
    };
    let authoring = inspect_meshy_static_placeable_authoring_v3(&source, &Default::default())
        .expect("Placeable authoring")
        .document;
    let ingest = ingest_glb(&source, &GlbLimits::default()).expect("material source ingest");
    let fallback_sha = ingest.ir.images[0].sha256.clone();
    let material = |id: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: id.to_owned(),
        preview_color: "#806040".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: Some(fallback_sha.clone()),
    };
    let selection = |primitive_id, material_id: &str| ModelMaterialFaceAssignmentV2 {
        selection: SourceFaceSelectionV2 {
            scene_id: 0,
            node_id: 0,
            primitive_id,
            triangle_ranges: vec![SourceTriangleRangeV2 {
                start_triangle: 0,
                triangle_count: 1,
            }],
        },
        authored_material_id: material_id.to_owned(),
    };
    let separation = ModelMaterialSeparationDocumentV2 {
        schema_version: 2,
        source_sha256: ingest.ir.source.sha256.clone(),
        materials: vec![material("material:sail"), material("material:wood")],
        component_assignments: Vec::new(),
        face_assignments: vec![selection(0, "material:wood"), selection(1, "material:sail")],
    };
    let materials = resolve_model_materials_v2(&ingest.ir, &separation)
        .expect("Face Mode V2 material separation");
    let texture_authoring = default_model_texture_authoring_v1(&ingest, materials.projection_v1())
        .expect("texture authoring");
    let projection = ModelMaterialUvProjectionDocumentV1 {
        schema_version: 1,
        source_sha256: ingest.ir.source.sha256.clone(),
        separation_sha256: materials.report.separation_sha256.clone(),
        rules: vec![ModelMaterialUvProjectionRuleV1 {
            authored_material_id: "material:wood".to_owned(),
            mode: ModelMaterialUvProjectionModeV1::MaterialBoxWorld,
            u_repeats: 0.5,
            v_min: 0.0,
            v_max: 1.0,
            deterministic_u_phase: false,
        }],
    };
    let baseline = build_meshy_static_placeable_package_v7(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &separation,
        &texture_authoring,
        &[],
        &[],
        &Default::default(),
    )
    .expect("Face Mode V2 Placeable");
    let projected = build_meshy_static_placeable_package_v8(
        &source,
        &base_placeables_2da(),
        &identity,
        placement,
        7,
        &authoring,
        &separation,
        &projection,
        &texture_authoring,
        &[],
        &[],
        &Default::default(),
    )
    .expect("material-local UV Placeable");

    assert_eq!(baseline.report.pwk_sha256, projected.report.pwk_sha256);
    assert_ne!(baseline.report.mdl_sha256, projected.report.mdl_sha256);
    let report = projected
        .report
        .material_uv_projection
        .as_ref()
        .expect("UV projection report");
    assert_eq!(report.source_triangle_count, report.output_triangle_count);
    assert_eq!(report.projected_triangle_count, 1);
    assert_eq!(report.projected_material_ids, ["material:wood"]);
    assert!(report.source_uv0_preserved_for_unprojected_materials);
    assert!(!report.geometry_cleanup);
    assert_eq!(
        projected
            .report
            .model_texture_authoring
            .as_ref()
            .expect("model texture report")
            .uv_policy,
        "MATERIAL_UV_PROJECTION_V1"
    );
}

fn hex_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn field<'a>(document: &'a m2a_core::gff::GffStructV1, label: &str) -> &'a GffValueV1 {
    &document
        .fields
        .iter()
        .find(|field| field.label == label)
        .unwrap_or_else(|| panic!("missing field {label}"))
        .value
}

#[test]
fn placeables_2da_appends_at_the_physical_end_with_dword_domain_identity() {
    let artifact =
        append_static_placeable_2da_v1(&base_placeables_2da(), "M2A_PEDESTAL", "m2a_plc_ped")
            .expect("append placeable row");

    assert_eq!(artifact.appearance_row.value, 3);
    assert_eq!(artifact.append.report.physical_rows_before, 3);
    let inspection =
        m2a_core::two_da::inspect_two_da_v2(&artifact.append.payload, &TwoDaLimitsV1::default())
            .expect("inspect appended 2DA");
    let row = read_two_da_row_v2(
        &artifact.append.payload,
        artifact.appearance_row.value,
        &TwoDaLimitsV1::default(),
    )
    .expect("read appended row");
    let value = |column: &str| {
        let index = inspection
            .columns
            .iter()
            .position(|candidate| candidate == column)
            .expect("column");
        &row.cells[index]
    };
    assert_eq!(
        value("Label"),
        &TwoDaCellValueV1::Text {
            value: "M2A_PEDESTAL".to_owned()
        }
    );
    assert_eq!(
        value("ModelName"),
        &TwoDaCellValueV1::Text {
            value: "m2a_plc_ped".to_owned()
        }
    );
    assert_eq!(
        value("Static"),
        &TwoDaCellValueV1::Text {
            value: "1".to_owned()
        }
    );
}

#[test]
fn static_utp_uses_the_frozen_retail_manifest_and_roundtrips() {
    let blueprint = StaticPlaceableBlueprintV1 {
        schema_version: 1,
        template_resref: "m2a_plc_utp".to_owned(),
        object_tag: "m2a_plc_pedestal".to_owned(),
        display_name: "Meshy Ritual Pedestal".to_owned(),
        appearance_row: m2a_core::placeable::PlaceableAppearanceRowV1 { value: 16_500 },
        palette_id: 0,
    };
    let artifact = write_static_placeable_utp_v1(&blueprint).expect("write UTP");
    let document = read_gff_v32(&artifact.payload, &GffLimitsV1::default()).expect("read UTP");

    assert_eq!(document.file_type, m2a_core::gff::GffFileTypeV1::Utp);
    assert_eq!(document.root.struct_id, u32::MAX);
    assert_eq!(document.root.fields.len(), 52);
    assert_eq!(
        field(&document.root, "TemplateResRef"),
        &GffValueV1::ResRef("m2a_plc_utp".to_owned())
    );
    assert_eq!(
        field(&document.root, "Appearance"),
        &GffValueV1::Dword(16_500)
    );
    assert_eq!(
        field(&document.root, "AnimationState"),
        &GffValueV1::Byte(0)
    );
    assert_eq!(field(&document.root, "Static"), &GffValueV1::Byte(1));
    assert_eq!(field(&document.root, "Useable"), &GffValueV1::Byte(0));
    assert!(
        document
            .root
            .fields
            .iter()
            .all(|candidate| candidate.label != "ItemList")
    );
}

#[test]
fn custom_placeable_palette_itp_binds_the_blueprint_to_its_category() {
    let blueprint = StaticPlaceableBlueprintV1 {
        schema_version: 1,
        template_resref: "m2a_plc_utp".to_owned(),
        object_tag: "m2a_plc_pedestal".to_owned(),
        display_name: "Meshy Ritual Pedestal".to_owned(),
        appearance_row: m2a_core::placeable::PlaceableAppearanceRowV1 { value: 16_500 },
        palette_id: 7,
    };
    let artifact = write_placeable_palette_itp_v1(&blueprint).expect("write custom palette");
    let document =
        read_gff_v32(&artifact.payload, &GffLimitsV1::default()).expect("read custom palette");
    assert_eq!(document.file_type, m2a_core::gff::GffFileTypeV1::Itp);
    let GffValueV1::List(main) = field(&document.root, "MAIN") else {
        panic!("MAIN must be a list");
    };
    let category = main
        .iter()
        .find(|item| {
            item.fields
                .iter()
                .any(|field| field.label == "ID" && field.value == GffValueV1::Byte(7))
        })
        .expect("palette category 7");
    let GffValueV1::List(entries) = field(category, "LIST") else {
        panic!("category LIST must be a list");
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(
        field(&entries[0], "NAME"),
        &GffValueV1::String(b"Meshy Ritual Pedestal".to_vec())
    );
    assert_eq!(
        field(&entries[0], "RESREF"),
        &GffValueV1::ResRef("m2a_plc_utp".to_owned())
    );
}

#[test]
fn placeable_contract_rejects_missing_columns_invalid_resrefs_and_unsupported_palette_ids() {
    let missing_static = b"2DA V2.0\n\nLabel StrRef ModelName LightColor LightOffsetX LightOffsetY LightOffsetZ SoundAppType ShadowSize BodyBag LowGore Reflection\n0 ARMOIRE **** plc_a01 **** **** **** **** **** 1 0 **** ****\n";
    let error = append_static_placeable_2da_v1(missing_static, "M2A_PEDESTAL", "m2a_plc_ped")
        .expect_err("missing required column");
    assert_eq!(error.code, "PLACEABLE-2DA-COLUMN-MISSING");

    let error = append_static_placeable_2da_v1(
        &base_placeables_2da(),
        "M2A_PEDESTAL",
        "model_resref_is_too_long",
    )
    .expect_err("long model resref");
    assert_eq!(error.code, "PLACEABLE-RESREF-INVALID");

    let error = append_static_placeable_2da_v1(&base_placeables_2da(), "M2A_PEDESTAL", "UpperCase")
        .expect_err("non-canonical resref case");
    assert_eq!(error.code, "PLACEABLE-RESREF-INVALID");

    let blueprint = StaticPlaceableBlueprintV1 {
        schema_version: 1,
        template_resref: "m2a_plc_utp".to_owned(),
        object_tag: "m2a_plc_pedestal".to_owned(),
        display_name: "Meshy Ritual Pedestal".to_owned(),
        appearance_row: m2a_core::placeable::PlaceableAppearanceRowV1 { value: 3 },
        palette_id: 255,
    };
    let error = write_placeable_palette_itp_v1(&blueprint).expect_err("unsupported palette id");
    assert_eq!(error.code, "PLACEABLE-PALETTE-ID-UNSUPPORTED");
}

#[test]
fn placeable_package_rejects_nonfinite_placement_and_unbound_identity_texture() {
    let mut request = static_request();
    request.placement.x = f32::NAN;
    let error = build_static_placeable_package_v1(&request).expect_err("non-finite placement");
    assert_eq!(error.code, "PLACEABLE-PLACEMENT-INVALID");

    let mut request = static_request();
    request.placement.bearing = f32::INFINITY;
    let error = build_static_placeable_package_v1(&request).expect_err("infinite placement");
    assert_eq!(error.code, "PLACEABLE-PLACEMENT-INVALID");

    let mut request = static_request();
    request.placement.y = 20.5;
    let error = build_static_placeable_package_v1(&request).expect_err("outside proof Area");
    assert_eq!(error.code, "PLACEABLE-PLACEMENT-OUTSIDE-PROOF-AREA");

    let mut request = static_request();
    request.identity.texture_resref = "missing_tex".to_owned();
    let error =
        build_static_placeable_package_v1(&request).expect_err("identity texture not bound");
    assert_eq!(error.code, "PLACEABLE-IDENTITY-TEXTURE-MISSING");

    let mut request = static_request();
    request
        .material_textures
        .push(request.material_textures[0].clone());
    let error =
        build_static_placeable_package_v1(&request).expect_err("duplicate material slot binding");
    assert_eq!(error.code, "PLACEABLE-TEXTURE-BINDING-DUPLICATE");

    let mut request = static_request();
    request.textures.push(request.textures[0].clone());
    let error = build_static_placeable_package_v1(&request)
        .expect_err("duplicate texture resource identity");
    assert_eq!(error.code, "PLACEABLE-TEXTURE-RESREF-DUPLICATE");

    let mut request = static_request();
    request.model.segments[0].indices = [0_u32, 1, 2].repeat(300_001);
    let error = build_static_placeable_package_v1(&request).expect_err("shared triangle budget");
    assert_eq!(error.code, "PLACEABLE-M2A-MODEL-TRIANGLE-BUDGET-EXCEEDED");
}

#[test]
fn full_static_placeable_package_is_deterministic_and_cross_resource_consistent() {
    let request = static_request();

    let first = build_static_placeable_package_v1(&request).expect("build package");
    let second = build_static_placeable_package_v1(&request).expect("rebuild package");
    assert_eq!(first.hak_payload, second.hak_payload);
    assert_eq!(first.module_payload, second.module_payload);
    assert_eq!(first.report, second.report);
    let mut one_byte_mutation = first.module_payload.clone();
    let last = one_byte_mutation
        .last_mut()
        .expect("generated module must not be empty");
    *last ^= 1;
    assert_ne!(
        hex_sha256(&one_byte_mutation),
        hex_sha256(&first.module_payload)
    );
    assert_eq!(first.report.status, "OFFLINE_ADMISSION_PASSED");
    assert_eq!(first.report.appearance_row.value, 3);
    assert_eq!(first.report.model_resref, "m2a_plc_ped");
    assert_eq!(first.report.blueprint_resref, "m2a_plc_utp");
    assert_eq!(first.report.model_visibility, "not_tested");
    assert_eq!(first.report.proof_completeness, "missing");
    assert_eq!(first.report.palette_completeness, "custom_itp_emitted");
    assert_eq!(
        first.report.collision_completeness,
        "ascii_pwk_emitted_offline_readback_passed"
    );
    assert!(!first.report.experimental_aggressive_geometry_cleanup);
    assert_eq!(first.report.hak_resource_count, 4);
    assert_eq!(first.report.profile, "STATIC_PLACEABLE");
    assert_eq!(first.report.component_statuses.mdl, "passed");
    assert_eq!(first.report.component_statuses.pwk, "passed");
    assert_eq!(first.report.component_statuses.two_da, "passed");
    assert_eq!(first.report.component_statuses.utp, "passed");
    assert_eq!(first.report.component_statuses.git_gic, "passed");
    assert_eq!(first.report.component_statuses.package, "passed");
    assert_eq!(first.report.component_statuses.proof, "not_tested");
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|resource| resource.resref == "m2a_plc_ped"
                && resource.resource_type == MDL_RESOURCE_TYPE)
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|resource| resource.resref == "m2a_plc_ped"
                && resource.resource_type == PWK_RESOURCE_TYPE
                && resource.sha256 == first.report.pwk_sha256)
    );
    assert!(
        first
            .report
            .resources
            .iter()
            .any(|resource| resource.resref == "m2a_plc_utp"
                && resource.resource_type == UTP_RESOURCE_TYPE)
    );

    let hak = ErfArchive::parse(&first.hak_payload).expect("read HAK");
    assert!(hak.find("placeables", PLACEABLES_2DA_RESOURCE_TYPE).is_ok());
    assert!(hak.find("m2a_plc_ped", MDL_RESOURCE_TYPE).is_ok());
    let pwk = hak
        .find("m2a_plc_ped", PWK_RESOURCE_TYPE)
        .expect("placeable PWK");
    assert!(pwk.is_ascii());
    assert!(!pwk.starts_with(&[0, 0, 0, 0]));
    let pwk = m2a_core::placeable_collision::inspect_ascii_placeable_walkmesh_v1(pwk)
        .expect("inspect ASCII PWK");
    assert_eq!(pwk.format, "nwn1-ascii-pwk");
    assert_eq!(pwk.mesh_nodes.len(), 1);
    let pwk_mesh = &pwk.mesh_nodes[0];
    assert_eq!(pwk_mesh.vertices.len(), 4);
    assert_eq!(pwk_mesh.faces.len(), 2);
    assert!(pwk_mesh.faces.iter().all(|face| face.surface_id == 7));
    assert!(hak.find("m2a_plc_tex", 3).is_ok());

    let module = ErfArchive::parse(&first.module_payload).expect("read MOD");
    assert!(module.find("module", IFO_RESOURCE_TYPE).is_ok());
    let are = read_gff_v32(
        module.find("m2a_plc_area", ARE_RESOURCE_TYPE).expect("ARE"),
        &GffLimitsV1::default(),
    )
    .expect("read ARE");
    let GffValueV1::LocString(area_name) = field(&are.root, "Name") else {
        panic!("ARE Name must be a localized string");
    };
    assert_eq!(area_name.substrings.len(), 1);
    assert_eq!(
        area_name.substrings[0].bytes,
        b"Meshy2Aurora placeable area"
    );
    assert!(module.find("m2a_plc_area", GIT_RESOURCE_TYPE).is_ok());
    assert!(module.find("m2a_plc_area", GIC_RESOURCE_TYPE).is_ok());
    assert!(module.find("m2a_plc_utp", UTP_RESOURCE_TYPE).is_ok());
    assert!(module.find("placeablepalcus", ITP_RESOURCE_TYPE).is_ok());

    let git = read_gff_v32(
        module.find("m2a_plc_area", GIT_RESOURCE_TYPE).expect("GIT"),
        &GffLimitsV1::default(),
    )
    .expect("read GIT");
    let GffValueV1::List(placeables) = field(&git.root, "Placeable List") else {
        panic!("Placeable List must be a list");
    };
    assert_eq!(placeables.len(), 1);
    assert_eq!(placeables[0].struct_id, 9);
    assert_eq!(
        field(&placeables[0], "TemplateResRef"),
        &GffValueV1::ResRef("m2a_plc_utp".to_owned())
    );
    assert_eq!(field(&placeables[0], "Appearance"), &GffValueV1::Dword(3));
    assert_eq!(field(&placeables[0], "X"), &GffValueV1::Float(10.0));
    assert_eq!(field(&placeables[0], "Y"), &GffValueV1::Float(14.5));

    let gic = read_gff_v32(
        module.find("m2a_plc_area", GIC_RESOURCE_TYPE).expect("GIC"),
        &GffLimitsV1::default(),
    )
    .expect("read GIC");
    let GffValueV1::List(comments) = field(&gic.root, "Placeable List") else {
        panic!("GIC Placeable List must be a list");
    };
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].struct_id, 9);
}

#[test]
#[ignore = "requires M2A_S1_PLACEABLE_GLB pointing at the owner-approved Meshy GLB"]
fn inspect_owner_placeable_glb_without_materializing_a_candidate() {
    let path = env::var("M2A_S1_PLACEABLE_GLB").expect("set M2A_S1_PLACEABLE_GLB");
    let bytes = fs::read(path).expect("read owner GLB in place");
    let result = m2a_core::glb::ingest_glb(&bytes, &m2a_core::glb::GlbLimits::default())
        .expect("ingest owner GLB");
    println!(
        "sourceSha256={} meshes={} primitives={} materials={} images={} skins={} animations={} triangles={}",
        result.ir.source.sha256,
        result.report.inventory.mesh_count,
        result.report.inventory.primitive_count,
        result.report.inventory.material_count,
        result.report.inventory.image_count,
        result.report.inventory.skin_count,
        result.report.inventory.animation_count,
        result.report.statistics.triangle_count,
    );
}

#[test]
fn placeable_profile_uses_the_shared_three_hundred_thousand_triangle_budget() {
    let options = static_placeable_profile_a_options_v1();
    let glb_limits = static_placeable_glb_limits_v1();
    assert_eq!(
        options.limits.triangle_warning_above,
        AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1 as u64
    );
    assert_eq!(
        options.limits.triangle_blocking_above,
        AURORA_MODEL_TRIANGLE_BUDGET_V1 as u64
    );
    assert_eq!(
        glb_limits.triangle_warning_above,
        AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1
    );
    assert_eq!(
        glb_limits.triangle_blocking_above,
        AURORA_MODEL_TRIANGLE_BUDGET_V1
    );
    assert_eq!(AURORA_MODEL_TRIANGLE_BUDGET_V1, 300_000);
    assert_eq!(AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1, 150_000);
    const {
        assert!(AURORA_MODEL_TRIANGLE_BUDGET_V1 > m2a_core::mdl::NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1);
    }
    assert_eq!(m2a_core::mdl::NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, 21_845);
    assert_eq!(m2a_core::mdl::NWN_EE_MAX_MESH_INDEX_COUNT_V1, 65_535);
}
