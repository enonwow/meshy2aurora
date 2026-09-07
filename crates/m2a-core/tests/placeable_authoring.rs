use m2a_core::{
    glb::{
        AuroraAssetIr, CoordinatePolicy, IrMaterial, IrMesh, IrNode, IrPrimitive, IrScene,
        IrSource, IrTransform,
    },
    model_components::SourceComponentKeyV1,
    model_material_separation::{
        AuthoredMaterialV1, ModelMaterialAssignmentV1, ModelMaterialFaceAssignmentV2,
        ModelMaterialSeparationDocumentV1, ModelMaterialSeparationDocumentV2,
        SourceFaceSelectionV2, SourceTriangleRangeV2,
    },
    placeable_authoring::{
        PlaceableAuthoringElementV1, PlaceableElementFlagsV1, PlaceableElementKindV1,
        PlaceableElementTransformV1, apply_placeable_authoring_v1,
        apply_placeable_authoring_with_material_separation_v1,
        apply_placeable_authoring_with_material_separation_v2, default_placeable_authoring_v1,
        inspect_placeable_elements_v1, split_placeable_node_components_v1,
    },
};

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
            sha256: "11".repeat(32),
            asset_version: "2.0".to_owned(),
            generator: Some("owned test fixture".to_owned()),
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
            name: Some("reactor".to_owned()),
            child_ids: Vec::new(),
            parent_ids: Vec::new(),
            transform: identity_transform(),
            mesh_id: Some(0),
            skin_id: None,
        }],
        meshes: vec![IrMesh {
            id: 0,
            name: Some("reactor-mesh".to_owned()),
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
            name: Some("reactor-metal".to_owned()),
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

#[test]
fn inspection_and_split_have_stable_component_identity() {
    let ir = two_component_ir();
    let inspection = inspect_placeable_elements_v1(&ir).expect("inspect editable elements");

    assert_eq!(inspection.render_node_count, 1);
    assert_eq!(inspection.primitive_count, 1);
    assert_eq!(inspection.connected_component_count, 2);
    assert_eq!(inspection.nodes[0].element_id, "node:0");
    assert_eq!(
        inspection.nodes[0].primitives[0].components[0].element_id,
        "node:0:primitive:0:component:0"
    );
    assert_eq!(
        inspection.nodes[0].primitives[0].components[1].element_id,
        "node:0:primitive:0:component:1"
    );

    let mut document = default_placeable_authoring_v1(&ir).expect("default authoring");
    assert_eq!(document.elements.len(), 1);
    assert_eq!(
        document.elements[0].kind,
        PlaceableElementKindV1::SourceNode
    );

    split_placeable_node_components_v1(&ir, &mut document, 0).expect("split node");
    assert_eq!(document.elements.len(), 2);
    assert!(document.elements.iter().all(|element| {
        element.kind == PlaceableElementKindV1::SourceComponent
            && element
                .source
                .as_ref()
                .and_then(|source| source.component_index)
                .is_some()
    }));
}

#[test]
fn component_transform_and_duplicate_are_baked_without_moving_the_other_component() {
    let mut ir = two_component_ir();
    let mut document = default_placeable_authoring_v1(&ir).expect("default authoring");
    split_placeable_node_components_v1(&ir, &mut document, 0).expect("split node");

    document.elements[0].transform.translation = [0.0, 0.0, 2.0];
    document.elements[0].transform.scale = [2.0, 2.0, 2.0];
    let mut duplicate = document.elements[0].clone();
    duplicate.id = "copy:lava-stream".to_owned();
    duplicate.name = "Lava stream copy".to_owned();
    duplicate.kind = PlaceableElementKindV1::Copy;
    duplicate.transform.translation = [10.0, 0.0, 0.0];
    duplicate.transform.scale = [1.0, 1.0, 1.0];
    duplicate.flags.include_in_collision = false;
    duplicate.flags.cast_shadow = false;
    document.elements.push(duplicate);

    let report = apply_placeable_authoring_v1(&mut ir, &document).expect("apply authoring");
    assert_eq!(report.source_triangle_count, 2);
    assert_eq!(report.output_triangle_count, 3);
    assert_eq!(report.renderable_element_count, 3);
    assert_eq!(report.collision_element_count, 2);
    assert_eq!(report.shadow_element_count, 2);
    assert_eq!(report.bounds_min, [0.0, 0.0, 0.0]);
    assert_eq!(report.bounds_max, [11.0, 2.0, 2.0]);

    assert_eq!(ir.primitives.len(), 2);
    assert_eq!(ir.materials.len(), 2);
    assert!(
        ir.materials[1]
            .name
            .as_deref()
            .is_some_and(|name| name.ends_with("__m2a_no_shadow"))
    );
    assert_eq!(
        ir.primitives
            .iter()
            .map(|primitive| primitive.indices.len())
            .sum::<usize>(),
        9
    );
    assert!(
        ir.primitives
            .iter()
            .flat_map(|primitive| primitive.positions.iter())
            .any(|position| position[2] == 2.0)
    );
    assert!(
        ir.primitives
            .iter()
            .flat_map(|primitive| primitive.positions.iter())
            .any(|position| position[0] == 10.0)
    );
    assert!(
        ir.primitives
            .iter()
            .flat_map(|primitive| primitive.positions.iter())
            .any(|position| *position == [4.0, 0.0, 0.0])
    );
}

#[test]
fn material_separation_splits_render_buckets_and_copies_inherit_material() {
    let mut ir = two_component_ir();
    let mut document = default_placeable_authoring_v1(&ir).expect("default authoring");
    split_placeable_node_components_v1(&ir, &mut document, 0).expect("split node");
    document.elements[0].name = "Renamed wood".to_owned();
    document.elements[0].transform.translation = [2.0, 0.0, 0.0];
    document.elements[0].flags.hidden = true;
    document.elements[0].flags.locked = true;
    let mut duplicate = document.elements[0].clone();
    duplicate.id = "copy:wood".to_owned();
    duplicate.name = "Renamed wood copy".to_owned();
    duplicate.kind = PlaceableElementKindV1::Copy;
    duplicate.transform.translation = [10.0, 0.0, 0.0];
    duplicate.flags.include_in_collision = false;
    duplicate.flags.cast_shadow = false;
    document.elements.push(duplicate);
    let material = |id: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: id.to_owned(),
        preview_color: "#806040".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: None,
    };
    let recipe = ModelMaterialSeparationDocumentV1 {
        schema_version: 1,
        source_sha256: ir.source.sha256.clone(),
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
                    primitive_id: 0,
                    component_index: 1,
                },
                authored_material_id: "material:sail".to_owned(),
            },
        ],
    };

    let report = apply_placeable_authoring_with_material_separation_v1(&mut ir, &document, &recipe)
        .expect("material separated Placeable authoring");

    assert_eq!(report.source_triangle_count, 2);
    assert_eq!(report.output_triangle_count, 3);
    assert_eq!(report.renderable_element_count, 3);
    assert_eq!(report.collision_element_count, 2);
    assert_eq!(report.shadow_element_count, 2);
    assert_eq!(ir.primitives.len(), 3);
    assert_eq!(
        ir.primitives
            .iter()
            .map(|primitive| primitive.material_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    assert_eq!(
        ir.primitives
            .iter()
            .map(|primitive| primitive.indices.len())
            .collect::<std::collections::BTreeSet<_>>(),
        [3].into_iter().collect()
    );
    assert!(
        ir.materials
            .iter()
            .any(|material| material.name.as_deref() == Some("material:wood"))
    );
    assert!(
        ir.materials
            .iter()
            .any(|material| material.name.as_deref() == Some("material:sail"))
    );
    let emitted_material_names = ir
        .primitives
        .iter()
        .map(|primitive| {
            ir.materials[primitive.material_id.expect("authored material") as usize]
                .name
                .as_deref()
                .expect("authored material name")
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert!(emitted_material_names.contains("material:wood"));
    assert!(emitted_material_names.contains("material:sail"));
    assert!(
        emitted_material_names
            .iter()
            .any(|name| name.starts_with("material:wood") && name.ends_with("__m2a_no_shadow")),
        "the copied wood remains logically wood while cast-shadow is bucketed independently"
    );
    assert!(ir.primitives.iter().any(|primitive| {
        let name = ir.materials[primitive.material_id.expect("material") as usize]
            .name
            .as_deref()
            .unwrap_or_default();
        name.ends_with("__m2a_no_shadow")
            && primitive
                .positions
                .iter()
                .any(|position| position[0] == 10.0)
    }));
}

#[test]
fn face_mode_splits_two_materials_inside_one_connected_component() {
    let mut ir = two_component_ir();
    let primitive = &mut ir.primitives[0];
    primitive.positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    primitive.normals = vec![[0.0, 0.0, 1.0]; 4];
    primitive.tangents = vec![[1.0, 0.0, 0.0, 1.0]; 4];
    primitive.uv0 = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    primitive.indices = vec![0, 1, 2, 1, 3, 2];
    primitive.bounds_max = [1.0, 1.0, 0.0];
    let document = default_placeable_authoring_v1(&ir).expect("default authoring");
    let material = |id: &str| AuthoredMaterialV1 {
        authored_material_id: id.to_owned(),
        display_name: id.to_owned(),
        preview_color: "#806040".to_owned(),
        source_fallback_material_id: Some(0),
        source_fallback_image_sha256: None,
    };
    let face = |triangle, material_id: &str| ModelMaterialFaceAssignmentV2 {
        selection: SourceFaceSelectionV2 {
            scene_id: 0,
            node_id: 0,
            primitive_id: 0,
            triangle_ranges: vec![SourceTriangleRangeV2 {
                start_triangle: triangle,
                triangle_count: 1,
            }],
        },
        authored_material_id: material_id.to_owned(),
    };
    let recipe = ModelMaterialSeparationDocumentV2 {
        schema_version: 2,
        source_sha256: ir.source.sha256.clone(),
        materials: vec![material("material:sail"), material("material:wood")],
        component_assignments: Vec::new(),
        face_assignments: vec![face(0, "material:wood"), face(1, "material:sail")],
    };

    let report = apply_placeable_authoring_with_material_separation_v2(&mut ir, &document, &recipe)
        .expect("Face Mode authoring");

    assert_eq!(report.source_triangle_count, 2);
    assert_eq!(report.output_triangle_count, 2);
    assert_eq!(ir.primitives.len(), 2);
    assert!(
        ir.primitives
            .iter()
            .all(|primitive| primitive.indices.len() == 3)
    );
    let emitted_material_names = ir
        .primitives
        .iter()
        .map(|primitive| {
            ir.materials[primitive.material_id.expect("material") as usize]
                .name
                .as_deref()
                .expect("material name")
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        emitted_material_names,
        ["material:sail", "material:wood"].into()
    );
}

#[test]
fn deleting_one_component_removes_only_its_output_geometry() {
    let mut ir = two_component_ir();
    let mut document = default_placeable_authoring_v1(&ir).expect("default authoring");
    split_placeable_node_components_v1(&ir, &mut document, 0).expect("split node");
    document.elements[0].deleted = true;

    let report = apply_placeable_authoring_v1(&mut ir, &document).expect("apply deletion");

    assert_eq!(report.source_triangle_count, 2);
    assert_eq!(report.output_triangle_count, 1);
    assert_eq!(ir.primitives.len(), 1);
    assert_eq!(ir.primitives[0].indices.len(), 3);
    assert!(
        ir.primitives[0]
            .positions
            .iter()
            .all(|position| position[0] >= 4.0),
        "the untouched connected component must remain byte-for-byte geometrically identifiable"
    );
}

#[test]
fn group_translation_moves_only_its_component_children() {
    let mut ir = two_component_ir();
    let mut document = default_placeable_authoring_v1(&ir).expect("default authoring");
    split_placeable_node_components_v1(&ir, &mut document, 0).expect("split node");

    let group_id = "group:lower-basin".to_owned();
    document.elements[0].parent_id = Some(group_id.clone());
    document.elements.push(PlaceableAuthoringElementV1 {
        id: group_id,
        name: "Lower basin".to_owned(),
        kind: PlaceableElementKindV1::Group,
        source: None,
        parent_id: None,
        transform: PlaceableElementTransformV1 {
            translation: [0.0, 0.0, 0.1],
            ..PlaceableElementTransformV1::default()
        },
        flags: PlaceableElementFlagsV1::default(),
        deleted: false,
    });

    let report = apply_placeable_authoring_v1(&mut ir, &document).expect("apply grouped move");
    assert_eq!(report.source_triangle_count, 2);
    assert_eq!(report.output_triangle_count, 2);
    assert_eq!(report.bounds_min, [0.0, 0.0, 0.0]);
    assert_eq!(report.bounds_max, [5.0, 1.0, 0.1]);

    let positions = ir.primitives[0].positions.as_slice();
    assert!(
        positions
            .iter()
            .take(3)
            .all(|position| (position[2] - 0.1).abs() < 1.0e-6),
        "the grouped component must receive the group translation"
    );
    assert!(
        positions
            .iter()
            .skip(3)
            .all(|position| position[2].abs() < 1.0e-6),
        "the ungrouped component must remain at its source location"
    );
}

#[test]
fn hierarchy_cycle_and_overlapping_source_coverage_fail_closed() {
    let ir = two_component_ir();
    let mut cycle = default_placeable_authoring_v1(&ir).expect("default authoring");
    cycle.elements[0].parent_id = Some(cycle.elements[0].id.clone());
    let error = apply_placeable_authoring_v1(&mut ir.clone(), &cycle).expect_err("self cycle");
    assert_eq!(error.code, "PLACEABLE-AUTHORING-HIERARCHY-CYCLE");

    let mut overlap = default_placeable_authoring_v1(&ir).expect("default authoring");
    let mut component = overlap.elements[0].clone();
    component.id = "node:0:primitive:0:component:0".to_owned();
    component.kind = PlaceableElementKindV1::SourceComponent;
    component.source.as_mut().expect("source").primitive_id = Some(0);
    component.source.as_mut().expect("source").component_index = Some(0);
    overlap.elements.push(component);
    let error =
        apply_placeable_authoring_v1(&mut ir.clone(), &overlap).expect_err("overlap must fail");
    assert_eq!(error.code, "PLACEABLE-AUTHORING-SOURCE-OVERLAP");
}

#[test]
fn non_uniform_scale_rebuilds_an_orthonormal_normal_tangent_frame() {
    let mut ir = two_component_ir();
    let diagonal = std::f32::consts::FRAC_1_SQRT_2;
    ir.primitives[0].normals = vec![[diagonal, diagonal, 0.0]; 6];
    ir.primitives[0].tangents = vec![[diagonal, -diagonal, 0.0, 1.0]; 6];

    let mut document = default_placeable_authoring_v1(&ir).expect("default authoring");
    document.elements[0].transform.scale = [2.0, 1.0, 1.0];

    apply_placeable_authoring_v1(&mut ir, &document).expect("apply non-uniform scale");
    let normal = ir.primitives[0].normals[0];
    let tangent = ir.primitives[0].tangents[0];
    let dot = normal[0] * tangent[0] + normal[1] * tangent[1] + normal[2] * tangent[2];

    assert!(dot.abs() < 1.0e-6, "normal/tangent dot={dot}");
    assert!((normal[0] - 0.447_213_6).abs() < 1.0e-6);
    assert!((normal[1] - 0.894_427_2).abs() < 1.0e-6);
    assert!((tangent[0] - 0.894_427_2).abs() < 1.0e-6);
    assert!((tangent[1] + 0.447_213_6).abs() < 1.0e-6);
    assert_eq!(tangent[3], 1.0);
}
