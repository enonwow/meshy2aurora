use serde_json::{Value, json};

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

fn mutate_json(mut glb: Vec<u8>, mutation: impl FnOnce(&mut Value)) -> Vec<u8> {
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let json_end = 20 + json_length;
    let mut root: Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
    mutation(&mut root);
    let bin_start = json_end + 8;
    let bin = glb.split_off(bin_start);
    make_glb(root, bin)
}
