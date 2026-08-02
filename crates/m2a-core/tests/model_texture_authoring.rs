use m2a_core::{
    glb::{GlbLimits, ingest_glb},
    model_components::SourceComponentKeyV1,
    model_material_separation::{
        AuthoredMaterialV1, ModelMaterialAssignmentV1, ModelMaterialSeparationDocumentV1,
        resolve_model_materials_v1,
    },
    model_texture_authoring::{
        ModelTextureBindingModeV1, ModelTexturePayloadDescriptorV1,
        default_model_texture_authoring_v1, resolve_model_texture_authoring_v1,
    },
};
use sha2::{Digest, Sha256};

#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod fixtures;

fn source_and_materials() -> (
    Vec<u8>,
    m2a_core::glb::GlbIngestResult,
    m2a_core::model_material_separation::ResolvedModelMaterialsV1,
) {
    let bytes = fixtures::mutate_json(
        fixtures::material_image_two_distinct_base_colors(),
        |root| {
            for material in root["materials"].as_array_mut().expect("materials") {
                material["alphaMode"] = serde_json::json!("OPAQUE");
                material
                    .as_object_mut()
                    .expect("material")
                    .remove("normalTexture");
                material
                    .as_object_mut()
                    .expect("material")
                    .remove("emissiveTexture");
                material["pbrMetallicRoughness"]
                    .as_object_mut()
                    .expect("pbr")
                    .remove("metallicRoughnessTexture");
            }
        },
    );
    let ingest = ingest_glb(&bytes, &GlbLimits::default()).expect("two-image source");
    let source_image_sha256 = ingest.ir.images[0].sha256.clone();
    let material = |id: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: id.to_owned(),
        preview_color: "#806040".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: Some(source_image_sha256.clone()),
    };
    let recipe = ModelMaterialSeparationDocumentV1 {
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
    let resolved = resolve_model_materials_v1(&ingest.ir, &recipe).expect("material resolution");
    (bytes, ingest, resolved)
}

#[test]
fn two_authored_materials_from_one_source_can_emit_distinct_exact_tgas() {
    let (bytes, ingest, separated) = source_and_materials();
    let mut authoring =
        default_model_texture_authoring_v1(&ingest, &separated).expect("texture bootstrap");
    let blue_payload = fixtures::OWNED_BLUE_RGBA_PNG.to_vec();
    let blue_sha = format!("{:x}", Sha256::digest(&blue_payload));
    let sail = authoring
        .bindings
        .iter_mut()
        .find(|binding| binding.authored_material_id == "material:sail")
        .expect("sail binding");
    sail.mode = ModelTextureBindingModeV1::Override;
    sail.override_asset_id = Some("override:sail".to_owned());
    sail.override_sha256 = Some(blue_sha.clone());
    sail.override_mime_type = Some("image/png".to_owned());
    sail.override_byte_length = Some(blue_payload.len() as u64);
    let descriptors = [ModelTexturePayloadDescriptorV1 {
        schema_version: 1,
        asset_id: "override:sail".to_owned(),
        sha256: blue_sha,
        mime_type: "image/png".to_owned(),
        byte_offset: 0,
        byte_length: blue_payload.len() as u64,
    }];

    let resolved = resolve_model_texture_authoring_v1(
        &bytes,
        &GlbLimits::default(),
        &ingest,
        &separated,
        "m2a_shiptex",
        &authoring,
        &blue_payload,
        &descriptors,
    )
    .expect("neutral texture resolution");

    assert_eq!(resolved.material_textures.len(), 2);
    assert_eq!(resolved.textures.len(), 2);
    assert_ne!(resolved.textures[0].payload, resolved.textures[1].payload);
    assert_eq!(
        resolved.report.separation_sha256,
        separated.report.separation_sha256
    );
    assert_eq!(
        resolved.report.bindings[0].authored_material_id,
        "material:sail"
    );
    assert_eq!(
        resolved.report.bindings[1].authored_material_id,
        "material:wood"
    );
}

#[test]
fn identical_resulting_tga_bytes_are_deduplicated_by_exact_sha() {
    let (bytes, ingest, separated) = source_and_materials();
    let authoring =
        default_model_texture_authoring_v1(&ingest, &separated).expect("texture bootstrap");
    let resolved = resolve_model_texture_authoring_v1(
        &bytes,
        &GlbLimits::default(),
        &ingest,
        &separated,
        "m2a_shiptex",
        &authoring,
        &[],
        &[],
    )
    .expect("source texture resolution");
    assert_eq!(resolved.material_textures.len(), 2);
    assert_eq!(resolved.textures.len(), 1);
    assert_eq!(resolved.report.resources[0].material_slots, [0, 1]);
}

#[test]
fn unsupported_alpha_blocks_and_ignored_source_pbr_maps_are_never_silent() {
    let build_case = |bytes: Vec<u8>| {
        let ingest = ingest_glb(&bytes, &GlbLimits::default()).expect("PBR source ingest");
        let fallback_sha = ingest.ir.images[0].sha256.clone();
        let authored = |id: &str| AuthoredMaterialV1 {
            authored_material_id: id.to_owned(),
            display_name: id.to_owned(),
            preview_color: "#806040".to_owned(),
            source_fallback_material_id: Some(0),
            source_fallback_image_sha256: Some(fallback_sha.clone()),
        };
        let recipe = ModelMaterialSeparationDocumentV1 {
            schema_version: 1,
            source_sha256: ingest.ir.source.sha256.clone(),
            materials: vec![authored("material:sail"), authored("material:wood")],
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
        let materials = resolve_model_materials_v1(&ingest.ir, &recipe).expect("materials");
        let mut authoring =
            default_model_texture_authoring_v1(&ingest, &materials).expect("texture bootstrap");
        let payload = fixtures::OWNED_BLUE_RGBA_PNG.to_vec();
        let payload_sha = format!("{:x}", Sha256::digest(&payload));
        let sail = authoring
            .bindings
            .iter_mut()
            .find(|binding| binding.authored_material_id == "material:sail")
            .expect("sail binding");
        sail.mode = ModelTextureBindingModeV1::Override;
        sail.override_asset_id = Some("override:sail".to_owned());
        sail.override_sha256 = Some(payload_sha.clone());
        sail.override_mime_type = Some("image/png".to_owned());
        sail.override_byte_length = Some(payload.len() as u64);
        let descriptors = vec![ModelTexturePayloadDescriptorV1 {
            schema_version: 1,
            asset_id: "override:sail".to_owned(),
            sha256: payload_sha,
            mime_type: "image/png".to_owned(),
            byte_offset: 0,
            byte_length: payload.len() as u64,
        }];
        (bytes, ingest, materials, authoring, payload, descriptors)
    };

    let (bytes, ingest, materials, authoring, payload, descriptors) =
        build_case(fixtures::material_image_two_distinct_base_colors());
    let error = resolve_model_texture_authoring_v1(
        &bytes,
        &GlbLimits::default(),
        &ingest,
        &materials,
        "m2a_shiptex",
        &authoring,
        &payload,
        &descriptors,
    )
    .expect_err("MASK source fallback must block an opaque-only override");
    assert_eq!(error.code, "MODEL-TEXTURE-ALPHA-MODE-UNSUPPORTED");

    let opaque_pbr = fixtures::mutate_json(
        fixtures::material_image_two_distinct_base_colors(),
        |root| root["materials"][0]["alphaMode"] = serde_json::json!("OPAQUE"),
    );
    let (bytes, ingest, materials, authoring, payload, descriptors) = build_case(opaque_pbr);
    let resolved = resolve_model_texture_authoring_v1(
        &bytes,
        &GlbLimits::default(),
        &ingest,
        &materials,
        "m2a_shiptex",
        &authoring,
        &payload,
        &descriptors,
    )
    .expect("opaque base-color override with explicit PBR-loss report");
    let sail = resolved
        .report
        .bindings
        .iter()
        .find(|binding| binding.authored_material_id == "material:sail")
        .expect("sail report");
    assert_eq!(
        sail.ignored_source_pbr_maps,
        [
            "normalTexture",
            "metallicRoughnessTexture",
            "emissiveTexture"
        ]
    );
}
