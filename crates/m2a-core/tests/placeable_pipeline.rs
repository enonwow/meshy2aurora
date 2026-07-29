use m2a_core::{
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1,
    erf::ErfArchive,
    gff::{GffLimitsV1, GffValueV1, read_gff_v32},
    mdl::MdlMaterialTextureBindingV1,
    model_pipeline::ProjectBuildIdentityV1,
    owned_fixture::synthetic_owned_m6_glb_v1,
    placeable::{
        GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE, IFO_RESOURCE_TYPE, ITP_RESOURCE_TYPE,
        MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE, PWK_RESOURCE_TYPE, PlaceablePlacementV1,
        PlaceableTextureInputV1, StaticPlaceableBlueprintV1, StaticPlaceableBuildRequestV1,
        StaticPlaceableIdentityV1, UTP_RESOURCE_TYPE, append_static_placeable_2da_v1,
        build_meshy_static_placeable_package_v1, build_meshy_static_placeable_package_v2,
        build_meshy_static_placeable_package_v3_with_project_identity,
        build_static_placeable_package_v1, inspect_meshy_static_placeable_authoring_v1,
        static_placeable_glb_limits_v1, static_placeable_profile_a_options_v1,
        write_placeable_palette_itp_v1, write_static_placeable_utp_v1,
    },
    placeable_authoring::PlaceableElementKindV1,
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, read_two_da_row_v2},
};
use std::{env, fs};

#[allow(dead_code)]
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
        "ascii_pwk_emitted_runtime_readback_passed"
    );
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
fn placeable_profile_uses_the_shared_product_budget_below_the_binary_boundary() {
    let options = static_placeable_profile_a_options_v1();
    let glb_limits = static_placeable_glb_limits_v1();
    assert_eq!(
        options.limits.triangle_blocking_above,
        m2a_core::AURORA_MODEL_TRIANGLE_BUDGET_V1 as u64
    );
    assert_eq!(
        glb_limits.triangle_blocking_above,
        m2a_core::AURORA_MODEL_TRIANGLE_BUDGET_V1
    );
    assert_eq!(
        options.limits.triangle_warning_above,
        m2a_core::AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1 as u64
    );
    assert_eq!(
        glb_limits.triangle_warning_above,
        m2a_core::AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1
    );
    assert_eq!(m2a_core::mdl::NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, 21_845);
    assert_eq!(m2a_core::mdl::NWN_EE_MAX_MESH_INDEX_COUNT_V1, 65_535);
}

#[test]
fn project_identity_owns_placeable_names_and_report_lineage() {
    let source = static_source_glb();
    let project = ProjectBuildIdentityV1 {
        schema_version: 1,
        project_id: "owner-placeable-project".to_owned(),
        project_name: "Owner Ritual Pedestal".to_owned(),
        project_revision: 7,
    };
    let first = build_meshy_static_placeable_package_v3_with_project_identity(
        &source,
        &base_placeables_2da(),
        &project,
        PlaceablePlacementV1::default(),
        7,
        None,
    )
    .expect("project-owned placeable");
    let second = build_meshy_static_placeable_package_v3_with_project_identity(
        &source,
        &base_placeables_2da(),
        &project,
        PlaceablePlacementV1::default(),
        7,
        None,
    )
    .expect("same revision");

    assert_eq!(first.report.project_identity, Some(project.clone()));
    assert_eq!(
        first.report.module_file_name,
        second.report.module_file_name
    );
    assert_eq!(first.report.hak_file_name, second.report.hak_file_name);
    assert_eq!(first.report.module_sha256, second.report.module_sha256);
    assert_eq!(first.report.hak_sha256, second.report.hak_sha256);
    assert_eq!(first.report.model_resref.len(), 16);
    assert_eq!(
        first.report.module_file_name,
        format!("{}.mod", first.report.model_resref)
    );
    assert_eq!(
        first.report.hak_file_name,
        format!("{}.hak", first.report.model_resref)
    );

    let changed = build_meshy_static_placeable_package_v3_with_project_identity(
        &source,
        &base_placeables_2da(),
        &ProjectBuildIdentityV1 {
            project_revision: 8,
            ..project
        },
        PlaceablePlacementV1::default(),
        7,
        None,
    )
    .expect("next revision");
    assert_ne!(first.report.model_resref, changed.report.model_resref);
    assert_ne!(first.report.module_sha256, changed.report.module_sha256);
}
