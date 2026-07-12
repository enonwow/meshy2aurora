use serde_json::{Value, json};

pub const MINIMAL_PNG: [u8; 68] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xb5, 0x1c, 0x0c,
    0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xfc, 0xff, 0x1f, 0x00,
    0x03, 0x03, 0x02, 0x00, 0xef, 0xa3, 0xe1, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44,
    0xae, 0x42, 0x60, 0x82,
];

pub fn minimal_indexed_triangle() -> Vec<u8> {
    geometry_glb(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        Some(&[[0.0, 0.0, 1.0]; 3]),
        Some(&[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        Some(&[0, 1, 2]),
        4,
        default_nodes(),
    )
}

pub fn axis_hierarchy_asymmetric() -> Vec<u8> {
    geometry_glb(
        &[[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]],
        Some(&[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
        Some(&[[0.25, 0.75], [0.5, 0.125], [1.0, 0.0]]),
        Some(&[2, 0, 1]),
        4,
        json!([
            {
                "name": "axis-root",
                "children": [1],
                "translation": [10.0, 20.0, 30.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [1.0, 1.0, 1.0]
            },
            {
                "name": "asymmetric-child",
                "mesh": 0,
                "translation": [1.0, 2.0, 3.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [2.0, 3.0, 4.0]
            }
        ]),
    )
}

pub fn uv_corners_and_out_of_range() -> Vec<u8> {
    geometry_glb(
        &[
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [-1.0, 2.0, 0.0],
        ],
        Some(&[[0.0, 0.0, 1.0]; 6]),
        Some(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [-0.25, 1.25],
            [2.0, -1.0],
        ]),
        Some(&[0, 1, 2, 0, 2, 3, 3, 4, 5]),
        4,
        default_nodes(),
    )
}

pub fn material_image_two_primitives() -> Vec<u8> {
    let (mut root, mut bin) = split_glb(minimal_indexed_triangle());
    align4(&mut bin);
    let image_offset = bin.len();
    bin.extend_from_slice(&MINIMAL_PNG);
    let image_view_index = root["bufferViews"]
        .as_array()
        .expect("synthetic buffer views")
        .len();
    root["bufferViews"]
        .as_array_mut()
        .expect("synthetic buffer views")
        .push(view(image_offset, MINIMAL_PNG.len()));
    root["buffers"][0]["byteLength"] = json!(bin.len());

    let primitives = root["meshes"][0]["primitives"]
        .as_array_mut()
        .expect("synthetic primitives");
    let mut second = primitives[0].clone();
    second["material"] = json!(1);
    primitives.push(second);

    root["samplers"] = json!([
        {
            "name": "nearest-clamped-mirrored",
            "magFilter": 9728,
            "minFilter": 9987,
            "wrapS": 33071,
            "wrapT": 33648
        },
        {
            "name": "linear-repeat-clamped",
            "magFilter": 9729,
            "minFilter": 9984,
            "wrapS": 10497,
            "wrapT": 33071
        }
    ]);
    root["images"] = json!([{
        "name": "embedded-one-pixel",
        "bufferView": image_view_index,
        "mimeType": "image/png"
    }]);
    root["textures"] = json!([
        {"name": "base-color", "sampler": 0, "source": 0},
        {"name": "normal-map", "sampler": 1, "source": 0},
        {"name": "metallic-roughness", "sampler": 0, "source": 0},
        {"name": "emissive", "sampler": 1, "source": 0}
    ]);
    root["materials"] = json!([
        {
            "name": "painted-mask",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.8, 0.7, 0.6, 0.5],
                "baseColorTexture": {"index": 0, "texCoord": 0},
                "metallicFactor": 0.35,
                "roughnessFactor": 0.65,
                "metallicRoughnessTexture": {"index": 2, "texCoord": 2}
            },
            "normalTexture": {"index": 1, "texCoord": 1},
            "emissiveFactor": [0.03, 0.02, 0.01],
            "emissiveTexture": {"index": 3, "texCoord": 3},
            "alphaMode": "MASK",
            "alphaCutoff": 0.33,
            "doubleSided": true
        },
        {
            "name": "translucent-detail",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.1, 0.2, 0.3, 0.4],
                "baseColorTexture": {"index": 0, "texCoord": 0},
                "metallicFactor": 0.05,
                "roughnessFactor": 0.15
            },
            "normalTexture": {"index": 1, "texCoord": 0},
            "alphaMode": "BLEND",
            "doubleSided": false
        }
    ]);
    make_glb(root, bin)
}

pub fn material_image_duplicate_image_reference() -> Vec<u8> {
    mutate_json(material_image_two_primitives(), |root| {
        let images = root["images"].as_array_mut().expect("synthetic images");
        let mut duplicate = images[0].clone();
        duplicate["name"] = json!("embedded-one-pixel-duplicate-reference");
        images.push(duplicate);
    })
}

pub fn material_image_invalid_buffer_view() -> Vec<u8> {
    mutate_json(material_image_two_primitives(), |root| {
        root["images"][0]["bufferView"] = json!(usize::MAX);
    })
}

pub fn material_image_with_declared_mime(mime_type: Value) -> Vec<u8> {
    mutate_json(material_image_two_primitives(), |root| {
        root["images"][0]["mimeType"] = mime_type;
    })
}

pub fn material_image_without_declared_mime() -> Vec<u8> {
    mutate_json(material_image_two_primitives(), |root| {
        root["images"][0]
            .as_object_mut()
            .expect("synthetic image")
            .remove("mimeType");
    })
}

pub fn nonfinite_material_factor() -> Vec<u8> {
    mutate_json(material_image_two_primitives(), |root| {
        root["materials"][0]["pbrMetallicRoughness"]["metallicFactor"] = json!(1.0e39_f64);
    })
}

pub fn missing_uv() -> Vec<u8> {
    geometry_glb(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        Some(&[[0.0, 0.0, 1.0]; 3]),
        None,
        Some(&[0, 1, 2]),
        4,
        default_nodes(),
    )
}

pub fn missing_position() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .expect("synthetic attributes")
            .remove("POSITION");
    })
}

pub fn missing_position_without_accessors_or_views() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["meshes"][0]["primitives"][0]["attributes"] = json!({});
        root["meshes"][0]["primitives"][0]
            .as_object_mut()
            .expect("synthetic primitive")
            .remove("indices");
        root["accessors"] = json!([]);
        root["bufferViews"] = json!([]);
    })
}

pub fn mismatched_attributes() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["accessors"][1]["count"] = json!(2);
    })
}

pub fn morph_target() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["meshes"][0]["primitives"][0]["targets"] = json!([{"POSITION": 0}]);
    })
}

pub fn non_indexed_triangle() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["meshes"][0]["primitives"][0]
            .as_object_mut()
            .expect("synthetic primitive")
            .remove("indices");
    })
}

pub fn indexed_triangle_with_component(component_type: u32) -> Vec<u8> {
    let (mut root, mut bin) = split_glb(minimal_indexed_triangle());
    let accessor_index = root["meshes"][0]["primitives"][0]["indices"]
        .as_u64()
        .expect("synthetic indices") as usize;
    let view_index = root["accessors"][accessor_index]["bufferView"]
        .as_u64()
        .expect("synthetic index view") as usize;
    let (offset, length) = match component_type {
        5121 => {
            let offset = root["bufferViews"][view_index]["byteOffset"]
                .as_u64()
                .expect("synthetic view offset") as usize;
            bin[offset..offset + 3].copy_from_slice(&[0, 1, 2]);
            (offset, 3)
        }
        5123 => {
            let offset = root["bufferViews"][view_index]["byteOffset"]
                .as_u64()
                .expect("synthetic view offset") as usize;
            (offset, 6)
        }
        5125 => {
            align4(&mut bin);
            let offset = bin.len();
            for index in [0_u32, 1, 2] {
                bin.extend_from_slice(&index.to_le_bytes());
            }
            (offset, 12)
        }
        _ => panic!("unsupported synthetic index component {component_type}"),
    };
    root["accessors"][accessor_index]["componentType"] = json!(component_type);
    root["bufferViews"][view_index]["byteOffset"] = json!(offset);
    root["bufferViews"][view_index]["byteLength"] = json!(length);
    root["buffers"][0]["byteLength"] = json!(bin.len());
    make_glb(root, bin)
}

pub fn incomplete_triangle() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["accessors"][3]["count"] = json!(2);
    })
}

pub fn degenerate_triangle() -> Vec<u8> {
    let (root, mut bin) = split_glb(minimal_indexed_triangle());
    let view_offset = root["bufferViews"][3]["byteOffset"]
        .as_u64()
        .expect("synthetic index view") as usize;
    for (slot, index) in [0_u16, 0, 1].into_iter().enumerate() {
        let offset = view_offset + slot * 2;
        bin[offset..offset + 2].copy_from_slice(&index.to_le_bytes());
    }
    make_glb(root, bin)
}

pub fn index_out_of_bounds() -> Vec<u8> {
    let (root, mut bin) = split_glb(minimal_indexed_triangle());
    let view_offset = root["bufferViews"][3]["byteOffset"]
        .as_u64()
        .expect("synthetic index view") as usize;
    bin[view_offset + 4..view_offset + 6].copy_from_slice(&3_u16.to_le_bytes());
    make_glb(root, bin)
}

pub fn missing_normals() -> Vec<u8> {
    geometry_glb(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        None,
        Some(&[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        Some(&[0, 1, 2]),
        4,
        default_nodes(),
    )
}

pub fn non_triangle_lines() -> Vec<u8> {
    geometry_glb(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        Some(&[[0.0, 0.0, 1.0]; 2]),
        Some(&[[0.0, 0.0], [1.0, 0.0]]),
        Some(&[0, 1]),
        1,
        default_nodes(),
    )
}

pub fn triangle_budget(triangle_count: usize) -> Vec<u8> {
    let mut indices = Vec::with_capacity(triangle_count.saturating_mul(3));
    for _ in 0..triangle_count {
        indices.extend_from_slice(&[0, 1, 2]);
    }
    geometry_glb(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        Some(&[[0.0, 0.0, 1.0]; 3]),
        Some(&[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        Some(&indices),
        4,
        default_nodes(),
    )
}

pub fn two_primitive_triangle_budget(triangle_count_each: usize) -> Vec<u8> {
    mutate_json(triangle_budget(triangle_count_each), |root| {
        let primitives = root["meshes"][0]["primitives"]
            .as_array_mut()
            .expect("synthetic primitives");
        primitives.push(primitives[0].clone());
    })
}

pub fn tiny_positions_with_large_normals_and_uv(attribute_count: usize) -> Vec<u8> {
    let normals = vec![[0.0, 0.0, 1.0]; attribute_count];
    let uv0 = vec![[0.5, 0.5]; attribute_count];
    geometry_glb(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        Some(&normals),
        Some(&uv0),
        Some(&[0, 1, 2]),
        4,
        default_nodes(),
    )
}

pub fn source_metadata_and_matrix() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["asset"]["generator"] = json!("m2a-matrix-probe");
        let node = root["nodes"][0].as_object_mut().expect("synthetic node");
        node.remove("translation");
        node.remove("rotation");
        node.remove("scale");
        node.insert(
            "matrix".to_owned(),
            json!([
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0
            ]),
        );
    })
}

pub fn external_image_uri() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["images"] = json!([{"uri": "outside.png"}]);
    })
}

pub fn external_buffer_uri() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["buffers"][0]["uri"] = json!("outside.bin");
    })
}

pub fn primitive_compression_extension(name: &str) -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["meshes"][0]["primitives"][0]["extensions"][name] = json!({});
    })
}

pub fn buffer_view_compression_extension(name: &str) -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["bufferViews"][0]["extensions"][name] = json!({});
    })
}

pub fn sparse_position_accessor() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["accessors"][0]["sparse"] = json!({
            "count": 1,
            "indices": {"bufferView": 0, "componentType": 5123},
            "values": {"bufferView": 0}
        });
    })
}

pub fn required_extension(name: &str) -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["extensionsRequired"] = json!([name]);
        root["extensionsUsed"] = json!([name]);
    })
}

pub fn used_extension(name: &str) -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["extensionsUsed"] = json!([name]);
    })
}

pub fn root_value(root: Value) -> Vec<u8> {
    make_glb(root, Vec::new())
}

pub fn buffer_view_oob() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["bufferViews"][0]["byteOffset"] = json!(usize::MAX);
    })
}

pub fn accessor_oob() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["accessors"][0]["byteOffset"] = json!(usize::MAX);
    })
}

pub fn invalid_accessor_layout() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["accessors"][0]["componentType"] = json!(5123);
        root["accessors"][0]["type"] = json!("VEC3");
    })
}

pub fn nonfinite_node_transform() -> Vec<u8> {
    mutate_json(minimal_indexed_triangle(), |root| {
        root["nodes"][0]["translation"] = json!([1.0e39_f64, 0.0, 0.0]);
    })
}

pub fn skin_animation_without_inverse_bind_matrices() -> Vec<u8> {
    skin_animation(false)
}

pub fn skin_animation_with_inverse_bind_matrices() -> Vec<u8> {
    skin_animation(true)
}

pub fn skin_animation_with_extra_inverse_bind_matrix() -> Vec<u8> {
    let (mut root, mut bin) = split_glb(skin_animation_with_inverse_bind_matrices());
    let accessor_index = root["skins"][0]["inverseBindMatrices"]
        .as_u64()
        .expect("synthetic IBM accessor") as usize;
    let view_index = root["accessors"][accessor_index]["bufferView"]
        .as_u64()
        .expect("synthetic IBM view") as usize;
    let source_offset = root["bufferViews"][view_index]["byteOffset"]
        .as_u64()
        .expect("synthetic IBM offset") as usize;
    let source_length = root["bufferViews"][view_index]["byteLength"]
        .as_u64()
        .expect("synthetic IBM length") as usize;
    let matrices = bin[source_offset..source_offset + source_length].to_vec();
    align4(&mut bin);
    let new_offset = bin.len();
    bin.extend_from_slice(&matrices);
    for value in [
        1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 7.0, 8.0, 9.0, 1.0,
    ] {
        bin.extend_from_slice(&value.to_le_bytes());
    }
    root["bufferViews"][view_index]["byteOffset"] = json!(new_offset);
    root["bufferViews"][view_index]["byteLength"] = json!(source_length + 64);
    root["accessors"][accessor_index]["count"] = json!(3);
    root["buffers"][0]["byteLength"] = json!(bin.len());
    make_glb(root, bin)
}

pub fn secondary_skin_influence_set() -> Vec<u8> {
    mutate_json(skin_animation_with_inverse_bind_matrices(), |root| {
        let joints = root["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"].clone();
        let weights = root["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"].clone();
        root["meshes"][0]["primitives"][0]["attributes"]["JOINTS_1"] = joints;
        root["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_1"] = weights;
    })
}

fn skin_animation(include_inverse_bind_matrices: bool) -> Vec<u8> {
    let (mut root, mut bin) = split_glb(minimal_indexed_triangle());

    let joints = [[0_u8, 1, 0, 0], [1, 0, 0, 0], [0, 1, 0, 0]];
    align4(&mut bin);
    let joints_offset = bin.len();
    for row in joints {
        bin.extend_from_slice(&row);
    }
    let joints_length = bin.len() - joints_offset;
    let joints_view = push_view(&mut root, joints_offset, joints_length);
    let joints_accessor = push_accessor(
        &mut root,
        json!({
            "bufferView": joints_view,
            "componentType": 5121,
            "count": 3,
            "type": "VEC4"
        }),
    );

    let weights = [
        [0.75_f32, 0.25, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.4, 0.6, 0.0, 0.0],
    ];
    let weights_range = append_f32x4(&mut bin, &weights);
    let weights_view = push_view(&mut root, weights_range.0, weights_range.1);
    let weights_accessor = push_accessor(
        &mut root,
        json!({
            "bufferView": weights_view,
            "componentType": 5126,
            "count": 3,
            "type": "VEC4"
        }),
    );
    root["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"] = json!(joints_accessor);
    root["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"] = json!(weights_accessor);

    root["nodes"] = json!([
        {
            "name": "rig-root",
            "children": [1, 2],
            "translation": [0.0, 0.0, 0.0]
        },
        {
            "name": "joint-one",
            "translation": [0.0, 1.0, 0.0]
        },
        {
            "name": "skinned-mesh",
            "mesh": 0,
            "skin": 0
        }
    ]);
    root["scenes"][0]["nodes"] = json!([0]);

    let mut skin = json!({
        "name": "two-joint-skin",
        "joints": [0, 1],
        "skeleton": 0
    });
    if include_inverse_bind_matrices {
        let matrices = [
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
            ],
        ];
        let range = append_f32x16(&mut bin, &matrices);
        let view_index = push_view(&mut root, range.0, range.1);
        let accessor_index = push_accessor(
            &mut root,
            json!({
                "bufferView": view_index,
                "componentType": 5126,
                "count": 2,
                "type": "MAT4"
            }),
        );
        skin["inverseBindMatrices"] = json!(accessor_index);
    }
    root["skins"] = json!([skin]);

    let times = [0.0_f32, 0.5, 1.25];
    let time_range = append_f32(&mut bin, &times);
    let time_view = push_view(&mut root, time_range.0, time_range.1);
    let time_accessor = push_accessor(
        &mut root,
        json!({
            "bufferView": time_view,
            "componentType": 5126,
            "count": 3,
            "type": "SCALAR",
            "min": [0.0],
            "max": [1.25]
        }),
    );
    let translations = [[0.0_f32, 0.0, 0.0], [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
    let translation_range = append_f32x3(&mut bin, &translations);
    let translation_view = push_view(&mut root, translation_range.0, translation_range.1);
    let translation_accessor = push_accessor(
        &mut root,
        json!({
            "bufferView": translation_view,
            "componentType": 5126,
            "count": 3,
            "type": "VEC3"
        }),
    );
    let rotations = [
        [0.0_f32, 0.0, 0.0, 1.0],
        [0.0, 0.70710677, 0.0, 0.70710677],
        [0.0, 1.0, 0.0, 0.0],
    ];
    let rotation_range = append_f32x4(&mut bin, &rotations);
    let rotation_view = push_view(&mut root, rotation_range.0, rotation_range.1);
    let rotation_accessor = push_accessor(
        &mut root,
        json!({
            "bufferView": rotation_view,
            "componentType": 5126,
            "count": 3,
            "type": "VEC4"
        }),
    );
    let scales = [
        [-0.1_f32, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.1, 0.0, 0.0],
        [-0.2, 0.0, 0.0],
        [1.5, 2.0, 2.5],
        [0.2, 0.0, 0.0],
        [-0.3, 0.0, 0.0],
        [2.0, 3.0, 4.0],
        [0.3, 0.0, 0.0],
    ];
    let scale_range = append_f32x3(&mut bin, &scales);
    let scale_view = push_view(&mut root, scale_range.0, scale_range.1);
    let scale_accessor = push_accessor(
        &mut root,
        json!({
            "bufferView": scale_view,
            "componentType": 5126,
            "count": 9,
            "type": "VEC3"
        }),
    );
    root["animations"] = json!([{
        "name": "source-trs",
        "samplers": [
            {"input": time_accessor, "output": translation_accessor, "interpolation": "LINEAR"},
            {"input": time_accessor, "output": rotation_accessor, "interpolation": "STEP"},
            {"input": time_accessor, "output": scale_accessor, "interpolation": "CUBICSPLINE"}
        ],
        "channels": [
            {"sampler": 0, "target": {"node": 1, "path": "translation"}},
            {"sampler": 1, "target": {"node": 1, "path": "rotation"}},
            {"sampler": 2, "target": {"node": 1, "path": "scale"}}
        ]
    }]);
    root["buffers"][0]["byteLength"] = json!(bin.len());
    make_glb(root, bin)
}

pub fn with_weights_animation_channel() -> Vec<u8> {
    mutate_json(skin_animation_with_inverse_bind_matrices(), |root| {
        root["animations"][0]["channels"]
            .as_array_mut()
            .expect("synthetic channels")
            .push(json!({"sampler": 0, "target": {"node": 2, "path": "weights"}}));
    })
}

pub fn mutate_accessor_f32(
    glb: Vec<u8>,
    accessor_index: usize,
    scalar_index: usize,
    value: f32,
) -> Vec<u8> {
    let (root, mut bin) = split_glb(glb);
    let accessor = &root["accessors"][accessor_index];
    let view_index = accessor["bufferView"].as_u64().unwrap() as usize;
    let view = &root["bufferViews"][view_index];
    let view_offset = view["byteOffset"].as_u64().unwrap_or(0) as usize;
    let accessor_offset = accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
    let byte_offset = view_offset + accessor_offset + scalar_index * 4;
    bin[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
    make_glb(root, bin)
}

pub fn mutate_accessor_u8(
    glb: Vec<u8>,
    accessor_index: usize,
    scalar_index: usize,
    value: u8,
) -> Vec<u8> {
    let (root, mut bin) = split_glb(glb);
    let accessor = &root["accessors"][accessor_index];
    let view_index = accessor["bufferView"].as_u64().unwrap() as usize;
    let view = &root["bufferViews"][view_index];
    let view_offset = view["byteOffset"].as_u64().unwrap_or(0) as usize;
    let accessor_offset = accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
    bin[view_offset + accessor_offset + scalar_index] = value;
    make_glb(root, bin)
}

fn default_nodes() -> Value {
    json!([{"name": "source-root", "mesh": 0}])
}

fn geometry_glb(
    positions: &[[f32; 3]],
    normals: Option<&[[f32; 3]]>,
    uv0: Option<&[[f32; 2]]>,
    indices: Option<&[u16]>,
    mode: u32,
    nodes: Value,
) -> Vec<u8> {
    let mut bin = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    let position_range = append_f32x3(&mut bin, positions);
    views.push(view(position_range.0, position_range.1));
    let (min, max) = bounds(positions);
    accessors.push(json!({
        "bufferView": 0,
        "componentType": 5126,
        "count": positions.len(),
        "type": "VEC3",
        "min": min,
        "max": max
    }));
    let mut attributes = serde_json::Map::new();
    attributes.insert("POSITION".to_owned(), json!(0));

    if let Some(normals) = normals {
        let range = append_f32x3(&mut bin, normals);
        let view_index = views.len();
        views.push(view(range.0, range.1));
        let accessor_index = accessors.len();
        accessors.push(json!({
            "bufferView": view_index,
            "componentType": 5126,
            "count": normals.len(),
            "type": "VEC3"
        }));
        attributes.insert("NORMAL".to_owned(), json!(accessor_index));
    }

    if let Some(uv0) = uv0 {
        let range = append_f32x2(&mut bin, uv0);
        let view_index = views.len();
        views.push(view(range.0, range.1));
        let accessor_index = accessors.len();
        accessors.push(json!({
            "bufferView": view_index,
            "componentType": 5126,
            "count": uv0.len(),
            "type": "VEC2"
        }));
        attributes.insert("TEXCOORD_0".to_owned(), json!(accessor_index));
    }

    let index_accessor = indices.map(|indices| {
        align4(&mut bin);
        let offset = bin.len();
        for index in indices {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        let length = bin.len() - offset;
        let view_index = views.len();
        views.push(view(offset, length));
        let accessor_index = accessors.len();
        accessors.push(json!({
            "bufferView": view_index,
            "componentType": 5123,
            "count": indices.len(),
            "type": "SCALAR"
        }));
        accessor_index
    });

    let mut primitive = serde_json::Map::new();
    primitive.insert("attributes".to_owned(), Value::Object(attributes));
    primitive.insert("mode".to_owned(), json!(mode));
    primitive.insert("material".to_owned(), json!(0));
    if let Some(index_accessor) = index_accessor {
        primitive.insert("indices".to_owned(), json!(index_accessor));
    }

    let root = json!({
        "asset": {"version": "2.0", "generator": "m2a-synthetic"},
        "scene": 0,
        "scenes": [{"name": "source-scene", "nodes": [0]}],
        "nodes": nodes,
        "meshes": [{"name": "source-mesh", "primitives": [Value::Object(primitive)]}],
        "materials": [{
            "name": "source-material",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.1, 0.2, 0.3, 0.4],
                "metallicFactor": 0.25,
                "roughnessFactor": 0.75
            },
            "emissiveFactor": [0.01, 0.02, 0.03],
            "alphaMode": "BLEND",
            "doubleSided": true
        }],
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": views,
        "accessors": accessors
    });
    make_glb(root, bin)
}

fn append_f32x3(bin: &mut Vec<u8>, values: &[[f32; 3]]) -> (usize, usize) {
    align4(bin);
    let offset = bin.len();
    for row in values {
        for value in row {
            bin.extend_from_slice(&value.to_le_bytes());
        }
    }
    (offset, bin.len() - offset)
}

fn append_f32x2(bin: &mut Vec<u8>, values: &[[f32; 2]]) -> (usize, usize) {
    align4(bin);
    let offset = bin.len();
    for row in values {
        for value in row {
            bin.extend_from_slice(&value.to_le_bytes());
        }
    }
    (offset, bin.len() - offset)
}

fn append_f32x4(bin: &mut Vec<u8>, values: &[[f32; 4]]) -> (usize, usize) {
    align4(bin);
    let offset = bin.len();
    for row in values {
        for value in row {
            bin.extend_from_slice(&value.to_le_bytes());
        }
    }
    (offset, bin.len() - offset)
}

fn append_f32x16(bin: &mut Vec<u8>, values: &[[f32; 16]]) -> (usize, usize) {
    align4(bin);
    let offset = bin.len();
    for row in values {
        for value in row {
            bin.extend_from_slice(&value.to_le_bytes());
        }
    }
    (offset, bin.len() - offset)
}

fn append_f32(bin: &mut Vec<u8>, values: &[f32]) -> (usize, usize) {
    align4(bin);
    let offset = bin.len();
    for value in values {
        bin.extend_from_slice(&value.to_le_bytes());
    }
    (offset, bin.len() - offset)
}

fn push_view(root: &mut Value, offset: usize, length: usize) -> usize {
    let views = root["bufferViews"]
        .as_array_mut()
        .expect("synthetic buffer views");
    let index = views.len();
    views.push(view(offset, length));
    index
}

fn push_accessor(root: &mut Value, accessor: Value) -> usize {
    let accessors = root["accessors"]
        .as_array_mut()
        .expect("synthetic accessors");
    let index = accessors.len();
    accessors.push(accessor);
    index
}

fn view(offset: usize, length: usize) -> Value {
    json!({"buffer": 0, "byteOffset": offset, "byteLength": length})
}

fn bounds(values: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for row in values {
        for axis in 0..3 {
            min[axis] = min[axis].min(row[axis]);
            max[axis] = max[axis].max(row[axis]);
        }
    }
    (min, max)
}

fn align4(bytes: &mut Vec<u8>) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
}

fn make_glb(root: Value, mut bin: Vec<u8>) -> Vec<u8> {
    let mut json = serde_json::to_vec(&root).expect("synthetic JSON must serialize");
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    align4(&mut bin);
    let length = 12 + 8 + json.len() + 8 + bin.len();
    let length = u32::try_from(length).expect("synthetic GLB length fits u32");
    let json_length = u32::try_from(json.len()).expect("synthetic JSON length fits u32");
    let bin_length = u32::try_from(bin.len()).expect("synthetic BIN length fits u32");

    let mut glb = Vec::with_capacity(length as usize);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&length.to_le_bytes());
    glb.extend_from_slice(&json_length.to_le_bytes());
    glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    glb.extend_from_slice(&json);
    glb.extend_from_slice(&bin_length.to_le_bytes());
    glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
    glb.extend_from_slice(&bin);
    glb
}

pub fn mutate_json(glb: Vec<u8>, mutation: impl FnOnce(&mut Value)) -> Vec<u8> {
    let (mut root, bin) = split_glb(glb);
    mutation(&mut root);
    make_glb(root, bin)
}

fn split_glb(mut glb: Vec<u8>) -> (Value, Vec<u8>) {
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let json_end = 20 + json_length;
    let root: Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
    let bin_start = json_end + 8;
    let bin = glb.split_off(bin_start);
    (root, bin)
}
