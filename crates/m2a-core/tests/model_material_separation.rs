use m2a_core::{
    glb::{
        AuroraAssetIr, CoordinatePolicy, GlbLimits, IrMaterial, IrMesh, IrNode, IrPrimitive,
        IrScene, IrSource, IrTransform, ingest_glb,
    },
    model_components::{SourceComponentKeyV1, inspect_model_components_v1},
    model_material_capabilities::{
        ModelRenderTargetV1, material_separation_capabilities_v1,
        validate_material_separation_counts_v1,
    },
    model_material_separation::{
        AuthoredMaterialV1, ModelMaterialAssignmentV1, ModelMaterialSeparationDocumentV1,
        default_model_material_separation_v1, model_material_separation_hash_v1,
        resolve_model_materials_v1,
    },
};

#[path = "fixtures/build_synthetic_glb.rs"]
#[allow(dead_code)]
mod fixtures;

#[test]
fn material_capabilities_are_target_supplied_and_share_the_writer_limit() {
    for target in [
        ModelRenderTargetV1::Creature,
        ModelRenderTargetV1::Placeable,
        ModelRenderTargetV1::Tile,
        ModelRenderTargetV1::ModelPart,
    ] {
        let capabilities = material_separation_capabilities_v1(target);
        assert!(capabilities.material_separation_supported);
        assert_eq!(capabilities.max_material_slots, 256);
        assert_eq!(capabilities.selection_granularity, "CONNECTED_COMPONENTS");
        assert!(!capabilities.face_selection_supported);
        assert!(!capabilities.automatic_material_inference);
    }
}

#[test]
fn shared_capability_validator_accepts_exact_limits_and_blocks_overflow() {
    for target in [
        ModelRenderTargetV1::Creature,
        ModelRenderTargetV1::Placeable,
        ModelRenderTargetV1::Tile,
        ModelRenderTargetV1::ModelPart,
    ] {
        let capabilities = material_separation_capabilities_v1(target);
        validate_material_separation_counts_v1(
            target,
            capabilities.max_material_slots as usize,
            capabilities.max_output_sections,
        )
        .expect("exact target capability boundary");
        let slot_error = validate_material_separation_counts_v1(
            target,
            capabilities.max_material_slots as usize + 1,
            0,
        )
        .expect_err("slot overflow");
        assert_eq!(slot_error.code, "MODEL-MATERIAL-SLOT-BUDGET-EXCEEDED");
        let section_error =
            validate_material_separation_counts_v1(target, 0, capabilities.max_output_sections + 1)
                .expect_err("section overflow");
        assert_eq!(section_error.code, "MODEL-MATERIAL-SECTION-BUDGET-EXCEEDED");
    }
}

fn identity_transform() -> IrTransform {
    IrTransform {
        kind: "TRS".to_owned(),
        matrix: None,
        translation: Some([0.0, 0.0, 0.0]),
        rotation: Some([0.0, 0.0, 0.0, 1.0]),
        scale: Some([1.0, 1.0, 1.0]),
    }
}

fn two_component_ir() -> AuroraAssetIr {
    AuroraAssetIr {
        schema_version: 1,
        source: IrSource {
            format: "GLB_2_0".to_owned(),
            byte_length: 1,
            sha256: "31".repeat(32),
            asset_version: "2.0".to_owned(),
            generator: Some("owned material-separation fixture".to_owned()),
        },
        coordinate_space: CoordinatePolicy::default(),
        default_scene_id: Some(0),
        scenes: vec![IrScene {
            id: 0,
            name: Some("Scene".to_owned()),
            root_node_ids: vec![0],
        }],
        nodes: vec![IrNode {
            id: 0,
            name: Some("ship".to_owned()),
            child_ids: Vec::new(),
            parent_ids: Vec::new(),
            transform: IrTransform {
                translation: Some([10.0, 0.0, -2.0]),
                ..identity_transform()
            },
            mesh_id: Some(0),
            skin_id: None,
        }],
        meshes: vec![IrMesh {
            id: 0,
            name: Some("ship-mesh".to_owned()),
            primitive_ids: vec![0],
        }],
        primitives: vec![IrPrimitive {
            id: 0,
            source_mesh_id: 0,
            source_primitive_index: 0,
            topology: "TRIANGLES".to_owned(),
            material_id: Some(0),
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [4.0, 0.0, 0.0],
                [5.0, 0.0, 0.0],
                [4.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 6],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 6],
            uv0: vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [0.0, 1.0],
                [0.0, 0.0],
                [1.0, 0.0],
                [0.0, 1.0],
            ],
            joints0: Vec::new(),
            weights0: Vec::new(),
            indices: vec![0, 1, 2, 3, 4, 5],
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [5.0, 1.0, 0.0],
            source_was_indexed: true,
        }],
        materials: vec![IrMaterial {
            id: 0,
            name: Some("source-atlas".to_owned()),
            base_color_factor: [1.0; 4],
            base_color_texture: None,
            metallic_factor: 0.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: None,
            normal_texture: None,
            emissive_factor: [0.0; 3],
            emissive_texture: None,
            alpha_mode: "OPAQUE".to_owned(),
            alpha_cutoff: None,
            double_sided: false,
        }],
        textures: Vec::new(),
        samplers: Vec::new(),
        images: Vec::new(),
        skins: Vec::new(),
        animations: Vec::new(),
    }
}

fn triangle_budget_ir(triangle_count: usize) -> AuroraAssetIr {
    let mut ir = two_component_ir();
    let primitive = &mut ir.primitives[0];
    primitive.positions.truncate(3);
    primitive.normals.truncate(3);
    primitive.tangents.truncate(3);
    primitive.uv0.truncate(3);
    primitive.indices = [0_u32, 1, 2].repeat(triangle_count);
    primitive.bounds_max = [1.0, 1.0, 0.0];
    ir
}

fn component(component_index: u32) -> SourceComponentKeyV1 {
    SourceComponentKeyV1 {
        scene_id: 0,
        node_id: 0,
        primitive_id: 0,
        component_index,
    }
}

fn material(id: &str, name: &str) -> AuthoredMaterialV1 {
    AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: name.to_owned(),
        preview_color: "#7a4f2a".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: None,
    }
}

#[test]
fn shared_component_inspection_has_stable_world_space_identity() {
    let inventory = inspect_model_components_v1(&two_component_ir()).expect("inspect components");

    assert_eq!(inventory.source_sha256, "31".repeat(32));
    assert_eq!(inventory.scene_id, 0);
    assert_eq!(inventory.components.len(), 2);
    assert_eq!(inventory.components[0].key, component(0));
    assert_eq!(inventory.components[0].triangle_count, 1);
    assert_eq!(inventory.components[0].vertex_count, 3);
    assert_eq!(inventory.components[0].bounds_min, [10.0, 0.0, -2.0]);
    assert_eq!(inventory.components[0].bounds_max, [11.0, 1.0, -2.0]);
    assert_eq!(inventory.components[1].key, component(1));
    assert_eq!(inventory.components[1].bounds_min, [14.0, 0.0, -2.0]);
    assert_eq!(inventory.components[1].bounds_max, [15.0, 1.0, -2.0]);
}

#[test]
fn material_resolution_accepts_exactly_the_shared_300k_budget() {
    let ir = triangle_budget_ir(m2a_core::AURORA_MODEL_TRIANGLE_BUDGET_V1);
    let document = default_model_material_separation_v1(&ir);
    let resolved = resolve_model_materials_v1(&ir, &document).expect("exact 300K budget");
    assert_eq!(
        resolved.report.source_triangle_count as usize,
        m2a_core::AURORA_MODEL_TRIANGLE_BUDGET_V1
    );
    assert_eq!(
        resolved.report.output_triangle_count,
        resolved.report.source_triangle_count
    );
}

#[test]
fn material_resolution_blocks_one_triangle_above_the_shared_budget_before_adjacency_allocation() {
    let ir = triangle_budget_ir(m2a_core::AURORA_MODEL_TRIANGLE_BUDGET_V1 + 1);
    let document = default_model_material_separation_v1(&ir);
    let error = resolve_model_materials_v1(&ir, &document).expect_err("300K + 1 must block");
    assert_eq!(error.code, "MODEL-COMPONENTS-TRIANGLE-BUDGET-EXCEEDED");
}

#[test]
fn inspection_supports_the_four_component_ms0_fixture() {
    let ingest = ingest_glb(
        &fixtures::one_primitive_four_disconnected_triangles(),
        &GlbLimits::default(),
    )
    .expect("four disconnected components");
    let inventory = inspect_model_components_v1(&ingest.ir).expect("component inspection");
    assert_eq!(inventory.components.len(), 4);
    assert_eq!(inventory.triangle_count, 4);
    assert_eq!(
        inventory
            .components
            .iter()
            .map(|component| component.key.component_index)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
}

#[test]
fn empty_recipe_is_an_identity_material_projection() {
    let ir = two_component_ir();
    let document = default_model_material_separation_v1(&ir);
    let resolved = resolve_model_materials_v1(&ir, &document).expect("resolve identity recipe");

    assert_eq!(resolved.report.source_component_count, 2);
    assert_eq!(resolved.report.assigned_component_count, 0);
    assert_eq!(resolved.report.unassigned_component_count, 2);
    assert_eq!(resolved.report.source_triangle_count, 2);
    assert_eq!(resolved.report.output_triangle_count, 2);
    assert_eq!(resolved.report.source_vertex_count, 6);
    assert_eq!(resolved.report.output_vertex_count, 6);
    assert_eq!(resolved.report.duplicated_boundary_vertex_count, 0);
    // A no-op recipe preserves the single source primitive/material bucket.
    // Disconnected components only become separate output sections when their
    // resolved material slots differ.
    assert_eq!(resolved.report.output_section_count, 1);
    assert_eq!(resolved.report.predicted_texture_count, 1);
    assert!(
        resolved
            .report
            .warnings
            .iter()
            .any(|warning| warning == "MATERIAL-SEPARATION-UV0-UNCHANGED")
    );
    assert_eq!(resolved.report.material_slots.len(), 1);
    assert_eq!(resolved.report.material_slots[0].material_slot, 0);
    assert_eq!(
        resolved.report.material_slots[0].source_material_id,
        Some(0)
    );
    assert_eq!(resolved.material_slot_for_triangle(0, 0, 0, 0), Some(0));
    assert_eq!(resolved.material_slot_for_triangle(0, 0, 0, 1), Some(0));
}

#[test]
fn two_components_can_use_distinct_authored_materials_with_one_source_fallback() {
    let ir = two_component_ir();
    let document = ModelMaterialSeparationDocumentV1 {
        schema_version: 1,
        source_sha256: ir.source.sha256.clone(),
        materials: vec![
            material("material:sail", "Sail"),
            material("material:wood", "Wood"),
        ],
        assignments: vec![
            ModelMaterialAssignmentV1 {
                component: component(0),
                authored_material_id: "material:wood".to_owned(),
            },
            ModelMaterialAssignmentV1 {
                component: component(1),
                authored_material_id: "material:sail".to_owned(),
            },
        ],
    };
    let resolved = resolve_model_materials_v1(&ir, &document).expect("resolve two materials");

    assert_eq!(resolved.report.material_slots.len(), 2);
    assert_eq!(resolved.report.predicted_texture_count, 2);
    assert_eq!(
        resolved.report.material_slots[0].authored_material_id,
        "material:sail"
    );
    assert_eq!(
        resolved.report.material_slots[1].authored_material_id,
        "material:wood"
    );
    assert_eq!(
        resolved.report.material_slots[0].source_material_id,
        Some(0)
    );
    assert_eq!(
        resolved.report.material_slots[1].source_material_id,
        Some(0)
    );
    assert_eq!(resolved.report.assigned_component_count, 2);
    assert_eq!(resolved.report.unassigned_component_count, 0);
    assert_eq!(resolved.material_slot_for_triangle(0, 0, 0, 0), Some(1));
    assert_eq!(resolved.material_slot_for_triangle(0, 0, 0, 1), Some(0));
}

#[test]
fn source_and_authored_slots_have_deterministic_order() {
    let ir = two_component_ir();
    let document = ModelMaterialSeparationDocumentV1 {
        schema_version: 1,
        source_sha256: ir.source.sha256.clone(),
        materials: vec![material("material:sail", "Sail")],
        assignments: vec![ModelMaterialAssignmentV1 {
            component: component(1),
            authored_material_id: "material:sail".to_owned(),
        }],
    };
    let resolved = resolve_model_materials_v1(&ir, &document).expect("resolve mixed materials");

    assert_eq!(resolved.report.material_slots.len(), 2);
    assert_eq!(
        resolved.report.material_slots[0].authored_material_id,
        "source:0"
    );
    assert_eq!(
        resolved.report.material_slots[1].authored_material_id,
        "material:sail"
    );
    assert_eq!(resolved.material_slot_for_triangle(0, 0, 0, 0), Some(0));
    assert_eq!(resolved.material_slot_for_triangle(0, 0, 0, 1), Some(1));
}

#[test]
fn canonical_hash_ignores_document_list_order() {
    let ir = two_component_ir();
    let first = ModelMaterialSeparationDocumentV1 {
        schema_version: 1,
        source_sha256: ir.source.sha256.clone(),
        materials: vec![
            material("material:wood", "Wood"),
            material("material:sail", "Sail"),
        ],
        assignments: vec![
            ModelMaterialAssignmentV1 {
                component: component(0),
                authored_material_id: "material:wood".to_owned(),
            },
            ModelMaterialAssignmentV1 {
                component: component(1),
                authored_material_id: "material:sail".to_owned(),
            },
        ],
    };
    let mut second = first.clone();
    second.materials.reverse();
    second.assignments.reverse();

    assert_eq!(
        model_material_separation_hash_v1(&first).expect("first hash"),
        model_material_separation_hash_v1(&second).expect("second hash")
    );
}

#[test]
fn stale_overlap_and_unknown_material_fail_closed() {
    let ir = two_component_ir();

    let mut stale = default_model_material_separation_v1(&ir);
    stale.source_sha256 = "ff".repeat(32);
    let error = resolve_model_materials_v1(&ir, &stale).expect_err("stale source");
    assert_eq!(error.code, "MATERIAL-SEPARATION-SOURCE-MISMATCH");

    let mut overlap = default_model_material_separation_v1(&ir);
    overlap.materials.push(material("material:wood", "Wood"));
    overlap.assignments = vec![
        ModelMaterialAssignmentV1 {
            component: component(0),
            authored_material_id: "material:wood".to_owned(),
        },
        ModelMaterialAssignmentV1 {
            component: component(0),
            authored_material_id: "material:wood".to_owned(),
        },
    ];
    let error = resolve_model_materials_v1(&ir, &overlap).expect_err("overlap");
    assert_eq!(error.code, "MATERIAL-SEPARATION-COMPONENT-OVERLAP");

    let mut unknown = default_model_material_separation_v1(&ir);
    unknown.assignments.push(ModelMaterialAssignmentV1 {
        component: component(0),
        authored_material_id: "material:missing".to_owned(),
    });
    let error = resolve_model_materials_v1(&ir, &unknown).expect_err("unknown material");
    assert_eq!(error.code, "MATERIAL-SEPARATION-MATERIAL-MISSING");
}
