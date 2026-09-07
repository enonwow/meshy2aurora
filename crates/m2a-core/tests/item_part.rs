use m2a_core::item::{ItemPartRecipeV1, ItemPartSlotV1, ItemPartTransformV1};
use m2a_core::item_part::{
    ITEM_PART_SOURCE_STALE, ITEM_PART_TRIANGLE_BUDGET_EXCEEDED, ItemPartCompileRequestV1,
    MeshyItemPartCompileRequestV1, compile_item_part_v1, compile_meshy_static_item_part_v1,
    compile_meshy_static_item_part_v2,
};
use m2a_core::mdl::MdlMaterialTextureBindingV1;
use m2a_core::mdl::NodeReport;
use m2a_core::model_ir::{
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1,
};
use sha2::{Digest, Sha256};

const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[path = "fixtures/build_synthetic_glb.rs"]
mod synthetic_glb;

fn model() -> AuroraModelIrV1 {
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "item-part-test".to_owned(),
        source_sha256: HASH.to_owned(),
        basis_status: "TEST".to_owned(),
        engine_facing_proof: "TEST".to_owned(),
        uv_runtime_proof: "TEST".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 10,
            name: "source_root".to_owned(),
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
            source_material_name: Some("synthetic".to_owned()),
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 1,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 10,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            face_surface_ids: vec![],
            weights: vec![],
        }],
    }
}

fn request() -> ItemPartCompileRequestV1 {
    let half = std::f32::consts::FRAC_1_SQRT_2;
    ItemPartCompileRequestV1 {
        schema_version: 1,
        recipe_sha256: HASH.to_owned(),
        part: ItemPartRecipeV1 {
            slot: ItemPartSlotV1::Bottom,
            variant: 23,
            source_part_id: "bottom".to_owned(),
            source_sha256: HASH.to_owned(),
            transform: ItemPartTransformV1 {
                translation: [0.25, -0.5, 1.0],
                rotation: [0.0, 0.0, half, half],
                scale: 2.0,
            },
        },
        model_resref: "wswls_b_023".to_owned(),
        model: model(),
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "m2a_item_tex".to_owned(),
        }],
    }
}

#[test]
fn item_part_writer_emits_retail_character_classification_and_exact_trs_controllers() {
    let artifact = compile_item_part_v1(&request()).unwrap();
    assert_eq!(artifact.inspection.model.classification, 4);
    assert_eq!(artifact.inspection.model.name, "wswls_b_023");
    assert_eq!(artifact.inspection.model.bounds_min.x, -5.0);
    assert_eq!(artifact.inspection.model.bounds_max.z, 10.0);
    assert_eq!(artifact.report.triangle_count, 1);
    assert_eq!(artifact.report.transform_controller_types, [8, 20, 36]);
    assert_eq!(artifact.report.semantic_readback_status, "PASS");

    let root = &artifact.inspection.node_tree.roots[0];
    assert_eq!(root.name, "wswls_b_023");
    assert!(root.controllers.is_empty());
    let transform = &root.children[0];
    assert_eq!(transform.name, "m2a_item_xform");
    assert_eq!(
        transform
            .controllers
            .iter()
            .map(|controller| controller.controller_type)
            .collect::<Vec<_>>(),
        [8, 20, 36]
    );
    let scale = transform
        .controllers
        .iter()
        .find(|controller| controller.controller_type == 36)
        .unwrap();
    assert_eq!(scale.values, [vec![2.0]]);
}

#[test]
fn item_part_output_is_deterministic_and_source_bound() {
    let first = compile_item_part_v1(&request()).unwrap();
    let second = compile_item_part_v1(&request()).unwrap();
    assert_eq!(first.payload, second.payload);
    assert_eq!(first.report.model_sha256, second.report.model_sha256);

    let mut stale = request();
    stale.model.source_sha256 = "1".repeat(64);
    assert_eq!(
        compile_item_part_v1(&stale).unwrap_err().code,
        ITEM_PART_SOURCE_STALE
    );
}

#[test]
fn identity_scale_omits_redundant_scale_controller() {
    let mut request = request();
    request.part.transform = ItemPartTransformV1::default();
    let artifact = compile_item_part_v1(&request).unwrap();
    assert_eq!(artifact.report.transform_controller_types, [8, 20]);
}

#[test]
fn item_part_preserves_finite_non_collinear_microtriangles() {
    let mut request = request();
    request.model.segments[0].positions = vec![
        [0.0, 0.0, 0.0],
        [0.000_001, 0.0, 0.0],
        [0.0, 0.000_001, 0.0],
    ];

    let artifact = compile_item_part_v1(&request).unwrap();
    assert_eq!(artifact.report.triangle_count, 1);
    assert_eq!(artifact.inspection.model.classification, 4);
}

#[test]
fn static_meshy_glb_compiles_directly_to_a_retail_class_item_part() {
    let source =
        synthetic_glb::one_primitive_two_disconnected_triangles_with_embedded_texture_and_sidedness(
            false,
        );
    let source_sha256 = format!("{:x}", Sha256::digest(&source));
    let part = ItemPartRecipeV1 {
        slot: ItemPartSlotV1::Model,
        variant: 201,
        source_part_id: "static-glb".to_owned(),
        source_sha256,
        transform: ItemPartTransformV1::default(),
    };
    let artifact = compile_meshy_static_item_part_v2(
        &source,
        &MeshyItemPartCompileRequestV1 {
            schema_version: 1,
            recipe_sha256: HASH.to_owned(),
            part,
            model_resref: "helm_201".to_owned(),
            material_textures: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "helm_201".to_owned(),
            }],
        },
    )
    .unwrap();

    assert_eq!(artifact.inspection.model.name, "helm_201");
    assert_eq!(artifact.inspection.model.classification, 4);
    assert_eq!(artifact.report.source_triangle_count, 2);
    assert_eq!(artifact.report.double_sided_backface_triangle_count, 0);
    assert_eq!(
        artifact.report.double_sided_policy,
        "SOURCE_SINGLE_SIDED_UNCHANGED_V1"
    );
    assert_eq!(artifact.report.triangle_count, 2);
    assert_eq!(artifact.report.semantic_readback_status, "PASS");
}

fn collect_mesh_nodes<'a>(node: &'a NodeReport, output: &mut Vec<&'a NodeReport>) {
    if node.mesh.is_some() {
        output.push(node);
    }
    for child in &node.children {
        collect_mesh_nodes(child, output);
    }
}

#[test]
fn static_item_preserves_source_double_sided_as_explicit_backface_geometry() {
    let source = synthetic_glb::
        one_primitive_two_disconnected_triangles_with_embedded_texture_and_double_sided();
    let source_sha256 = format!("{:x}", Sha256::digest(&source));
    let artifact = compile_meshy_static_item_part_v2(
        &source,
        &MeshyItemPartCompileRequestV1 {
            schema_version: 1,
            recipe_sha256: HASH.to_owned(),
            part: ItemPartRecipeV1 {
                slot: ItemPartSlotV1::Model,
                variant: 202,
                source_part_id: "double-sided-static-glb".to_owned(),
                source_sha256,
                transform: ItemPartTransformV1::default(),
            },
            model_resref: "helm_202".to_owned(),
            material_textures: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "helm_202".to_owned(),
            }],
        },
    )
    .unwrap();

    assert_eq!(artifact.report.source_triangle_count, 2);
    assert_eq!(artifact.report.double_sided_backface_triangle_count, 2);
    assert_eq!(artifact.report.triangle_count, 4);
    assert_eq!(
        artifact.report.double_sided_policy,
        "SOURCE_DOUBLE_SIDED_EXPLICIT_BACKFACES_V1"
    );

    let mut mesh_nodes = Vec::new();
    for root in &artifact.inspection.node_tree.roots {
        collect_mesh_nodes(root, &mut mesh_nodes);
    }
    assert_eq!(mesh_nodes.len(), 1);
    let mesh = mesh_nodes[0].mesh.as_ref().unwrap();
    assert_eq!(mesh.faces.len(), 4);
    assert_eq!(mesh.vertex_count, 12);
    assert_eq!(&mesh.uv0[..6], &mesh.uv0[6..]);
    for index in 0..6 {
        assert_eq!(mesh.normals[index].x, -mesh.normals[index + 6].x);
        assert_eq!(mesh.normals[index].y, -mesh.normals[index + 6].y);
        assert_eq!(mesh.normals[index].z, -mesh.normals[index + 6].z);
    }
    let source_face = mesh.faces[0].vertex_indices;
    assert_eq!(
        mesh.faces[2].vertex_indices,
        [source_face[0] + 6, source_face[2] + 6, source_face[1] + 6,]
    );
}

#[test]
fn static_item_v1_keeps_legacy_double_sided_behavior_for_frozen_lineages() {
    let source = synthetic_glb::
        one_primitive_two_disconnected_triangles_with_embedded_texture_and_double_sided();
    let source_sha256 = format!("{:x}", Sha256::digest(&source));
    let artifact = compile_meshy_static_item_part_v1(
        &source,
        &MeshyItemPartCompileRequestV1 {
            schema_version: 1,
            recipe_sha256: HASH.to_owned(),
            part: ItemPartRecipeV1 {
                slot: ItemPartSlotV1::Model,
                variant: 203,
                source_part_id: "legacy-double-sided-static-glb".to_owned(),
                source_sha256,
                transform: ItemPartTransformV1::default(),
            },
            model_resref: "helm_203".to_owned(),
            material_textures: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "helm_203".to_owned(),
            }],
        },
    )
    .unwrap();

    assert_eq!(artifact.report.source_triangle_count, 2);
    assert_eq!(artifact.report.double_sided_backface_triangle_count, 0);
    assert_eq!(artifact.report.triangle_count, 2);
    assert_eq!(
        artifact.report.double_sided_policy,
        "SOURCE_DOUBLE_SIDED_LEGACY_UNMAPPED_V1"
    );
}

#[test]
fn static_item_v2_applies_the_shared_triangle_budget_after_double_sided_expansion() {
    let source = synthetic_glb::triangle_budget_with_double_sided_and_positive_extents(150_001);
    let source_sha256 = format!("{:x}", Sha256::digest(&source));
    let error = compile_meshy_static_item_part_v2(
        &source,
        &MeshyItemPartCompileRequestV1 {
            schema_version: 1,
            recipe_sha256: HASH.to_owned(),
            part: ItemPartRecipeV1 {
                slot: ItemPartSlotV1::Model,
                variant: 204,
                source_part_id: "double-sided-over-budget-static-glb".to_owned(),
                source_sha256,
                transform: ItemPartTransformV1::default(),
            },
            model_resref: "helm_204".to_owned(),
            material_textures: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "helm_204".to_owned(),
            }],
        },
    )
    .unwrap_err();

    assert_eq!(error.code, ITEM_PART_TRIANGLE_BUDGET_EXCEEDED);
    assert!(error.message.contains("300002 triangles"));
}
