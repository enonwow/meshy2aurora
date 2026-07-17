//! Deterministic, repository-owned source material for native proof packets.
//!
//! No retail, CEP, decompiled or third-party payload is embedded here.

use std::fmt;

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use serde::Serialize;
use serde_json::{Value, json};

use crate::profile_a::{
    Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
    ProfileAAnimationClipMappingV1, ProfileAAnimationMappingV1, ProfileAAnimationNodeMappingV1,
    RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1, RigSegmentDeformationV1,
    RigWeightInfluenceV1, canonical_profile_sha256,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedFixtureErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub message: String,
}

impl fmt::Display for OwnedFixtureErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for OwnedFixtureErrorV1 {}

/// Builds an asymmetric twelve-triangle skinned 3D wedge with two embedded PNGs.
///
/// Image zero is an intentional decoy. The only used base-color texture points
/// at image one, a 2x2 RGBA checker. The source root has one LINEAR translation
/// clip which the M6 mapping renames to `cpause1`.
pub fn synthetic_owned_m6_glb_v1() -> Result<Vec<u8>, OwnedFixtureErrorV1> {
    let a = [-0.8_f32, 0.0, -0.35];
    let b = [0.6, 0.0, -0.35];
    let c = [0.7, 1.5, -0.25];
    let d = [-0.5, 1.8, -0.25];
    let a2 = [-0.8, 0.0, 0.35];
    let b2 = [0.6, 0.0, 0.35];
    let c2 = [0.7, 1.5, 0.2];
    let d2 = [-0.5, 1.8, 0.2];
    let positions = [
        a, d, c, b, a2, b2, c2, d2, a, a2, d2, d, b, c, c2, b2, a, b, b2, a2, d, d2, c2, c,
    ];
    let normals: [[f32; 3]; 24] = std::array::from_fn(|index| {
        let face = index / 4;
        face_normal(
            positions[face * 4],
            positions[face * 4 + 1],
            positions[face * 4 + 2],
        )
    });
    let uv0 = [
        [0.0_f32, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
    ];
    let indices: [u16; 36] = std::array::from_fn(|index| {
        let face = index / 6;
        let local = [0_u16, 1, 2, 0, 2, 3][index % 6];
        u16::try_from(face * 4).unwrap_or(0) + local
    });
    let joints = [[0_u8, 1, 0, 0]; 24];
    let weights: [[f32; 4]; 24] = std::array::from_fn(|index| {
        let height = positions[index][1];
        let tip = (height / 1.8).clamp(0.0, 1.0) * 0.8;
        [1.0 - tip, tip, 0.0, 0.0]
    });
    let inverse_bind_matrices = [
        identity(),
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0,
        ],
    ];
    let times = [0.0_f32, 0.5, 1.25];
    let translations = [[0.0_f32, 0.0, 0.0], [0.25, 0.0, 0.15], [-0.1, 0.2, 0.0]];

    let mut bin = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    let position_accessor = push_f32x3_accessor(
        &mut bin,
        &mut views,
        &mut accessors,
        &positions,
        Some(([-0.8, 0.0, -0.35], [0.7, 1.8, 0.35])),
    );
    let normal_accessor = push_f32x3_accessor(&mut bin, &mut views, &mut accessors, &normals, None);
    let uv_accessor = push_f32x2_accessor(&mut bin, &mut views, &mut accessors, &uv0);
    let index_accessor = push_u16_accessor(&mut bin, &mut views, &mut accessors, &indices);
    let joint_accessor = push_u8x4_accessor(&mut bin, &mut views, &mut accessors, &joints);
    let weight_accessor = push_f32x4_accessor(&mut bin, &mut views, &mut accessors, &weights);
    let inverse_bind_accessor =
        push_f32x16_accessor(&mut bin, &mut views, &mut accessors, &inverse_bind_matrices);
    let time_accessor = push_f32_accessor(
        &mut bin,
        &mut views,
        &mut accessors,
        &times,
        Some((0.0, 1.25)),
    );
    let translation_accessor =
        push_f32x3_accessor(&mut bin, &mut views, &mut accessors, &translations, None);

    let decoy_png = encode_png(&[
        32, 32, 32, 255, 64, 64, 64, 255, 96, 96, 96, 255, 128, 128, 128, 255,
    ])?;
    let checker_png = encode_png(&[
        255, 32, 32, 255, 32, 255, 64, 255, 32, 64, 255, 255, 255, 224, 32, 255,
    ])?;
    let decoy_view = push_blob(&mut bin, &mut views, &decoy_png);
    let checker_view = push_blob(&mut bin, &mut views, &checker_png);

    let root = json!({
        "asset": {
            "version": "2.0",
            "generator": "meshy2aurora-owned-synthetic-m6-v1",
            "extras": {
                "provenance": "SYNTHETIC",
                "exportAllowed": true,
                "noReferencePayloadCopied": true
            }
        },
        "scene": 0,
        "scenes": [{"name": "m6-proof-scene", "nodes": [0]}],
        "nodes": [
            {"name": "m6-root", "children": [1, 2], "translation": [0.0, 0.0, 0.0]},
            {"name": "m6-tip", "translation": [0.0, 1.0, 0.0]},
            {"name": "m6-asymmetric-mesh", "mesh": 0, "skin": 0}
        ],
        "meshes": [{
            "name": "m6-asymmetric-low-poly",
            "primitives": [{
                "attributes": {
                    "POSITION": position_accessor,
                    "NORMAL": normal_accessor,
                    "TEXCOORD_0": uv_accessor,
                    "JOINTS_0": joint_accessor,
                    "WEIGHTS_0": weight_accessor
                },
                "indices": index_accessor,
                "material": 0,
                "mode": 4
            }]
        }],
        "skins": [{
            "name": "m6-owned-rig",
            "joints": [0, 1],
            "skeleton": 0,
            "inverseBindMatrices": inverse_bind_accessor
        }],
        "animations": [{
            "name": "owned-linear-pause",
            "samplers": [{
                "input": time_accessor,
                "output": translation_accessor,
                "interpolation": "LINEAR"
            }],
            "channels": [{"sampler": 0, "target": {"node": 0, "path": "translation"}}]
        }],
        "materials": [{
            "name": "m6-checker-material",
            "pbrMetallicRoughness": {
                "baseColorFactor": [1.0, 1.0, 1.0, 1.0],
                "baseColorTexture": {"index": 1, "texCoord": 0},
                "metallicFactor": 0.0,
                "roughnessFactor": 1.0
            },
            "doubleSided": true
        }],
        "samplers": [{"magFilter": 9728, "minFilter": 9728, "wrapS": 10497, "wrapT": 10497}],
        "textures": [
            {"name": "m6-decoy-texture-zero", "sampler": 0, "source": 0},
            {"name": "m6-used-checker", "sampler": 0, "source": 1}
        ],
        "images": [
            {"name": "m6-decoy-image-zero", "bufferView": decoy_view, "mimeType": "image/png"},
            {"name": "m6-used-checker-image-one", "bufferView": checker_view, "mimeType": "image/png"}
        ],
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": views,
        "accessors": accessors
    });
    make_glb(root, bin)
}

pub fn synthetic_owned_m6_rig_v1() -> Result<CreatureRigProfileV1, OwnedFixtureErrorV1> {
    let mut rig = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: "m2a-m6-owned-proof-rig-v1".to_owned(),
        content_sha256: String::new(),
        provenance: provenance(),
        target_bounds: Bounds3V1 {
            min: [-1.5, -0.5, 0.0],
            max: [1.5, 1.0, 3.0],
        },
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes: vec![
            CreatureRigNodeV1 {
                id: 100,
                name: "m6-root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity(),
            },
            CreatureRigNodeV1 {
                id: 101,
                name: "m6-tip".to_owned(),
                parent_id: Some(100),
                bind_local_matrix: translation(0.0, 1.0, 0.0),
            },
        ],
        segments: vec![CreatureRigSegmentV1 {
            id: 200,
            name: "m6-asymmetric-body".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 100,
            surface_positions: vec![
                [-1.5, -0.5, 0.0],
                [1.5, -0.5, 0.0],
                [1.5, 1.0, 0.0],
                [-1.5, 1.0, 0.0],
                [-1.5, -0.5, 3.0],
                [1.5, -0.5, 3.0],
                [1.5, 1.0, 3.0],
                [-1.5, 1.0, 3.0],
            ],
            surface_indices: vec![
                0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 4, 7, 0, 7, 3, 1, 2, 6, 1, 6, 5, 0, 1, 5, 0,
                5, 4, 3, 7, 6, 3, 6, 2,
            ],
            allowed_bone_node_ids: vec![100, 101],
            reference_weights: (0..8)
                .map(|index| {
                    let tip = if index < 4 { 0.15 } else { 0.85 };
                    vec![
                        RigWeightInfluenceV1 {
                            bone_node_id: 100,
                            value: 1.0 - tip,
                        },
                        RigWeightInfluenceV1 {
                            bone_node_id: 101,
                            value: tip,
                        },
                    ]
                })
                .collect(),
        }],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|error| OwnedFixtureErrorV1 {
        schema_version: 1,
        code: error.code,
        message: error.message,
    })?;
    Ok(rig)
}

pub fn synthetic_owned_m6_animation_mapping_v1() -> ProfileAAnimationMappingV1 {
    ProfileAAnimationMappingV1 {
        schema_version: 1,
        source_skin_id: 0,
        provenance: provenance(),
        source_translation_scale: 1.0,
        node_mappings: vec![
            ProfileAAnimationNodeMappingV1 {
                source_node_id: 0,
                output_rig_node_id: 100,
            },
            ProfileAAnimationNodeMappingV1 {
                source_node_id: 1,
                output_rig_node_id: 101,
            },
        ],
        clip_mappings: vec![ProfileAAnimationClipMappingV1 {
            source_animation_id: 0,
            output_clip_name: "cpause1".to_owned(),
            transition_seconds: 0.25,
        }],
    }
}

pub fn synthetic_owned_m6_provenance_v1() -> RigProvenanceV1 {
    provenance()
}

fn provenance() -> RigProvenanceV1 {
    RigProvenanceV1 {
        kind: RigProvenanceKindV1::Synthetic,
        export_allowed: true,
        attestations: RigProvenanceAttestationsV1 {
            controlled_construction: true,
            no_reference_payload_copied: true,
            rights_confirmed: true,
        },
    }
}

fn identity() -> [f32; 16] {
    translation(0.0, 0.0, 0.0)
}

fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let length = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    [cross[0] / length, cross[1] / length, cross[2] / length]
}

fn encode_png(pixels: &[u8]) -> Result<Vec<u8>, OwnedFixtureErrorV1> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(pixels, 2, 2, ExtendedColorType::Rgba8)
        .map_err(|error| OwnedFixtureErrorV1 {
            schema_version: 1,
            code: "M6-FIXTURE-PNG-ENCODE-FAILED".to_owned(),
            message: error.to_string(),
        })?;
    Ok(bytes)
}

fn push_blob(bin: &mut Vec<u8>, views: &mut Vec<Value>, bytes: &[u8]) -> usize {
    align4(bin);
    let offset = bin.len();
    bin.extend_from_slice(bytes);
    let index = views.len();
    views.push(view(offset, bytes.len()));
    index
}

fn push_f32x3_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[[f32; 3]],
    bounds: Option<([f32; 3], [f32; 3])>,
) -> usize {
    let (offset, length) = append_f32_rows(bin, values);
    let view_index = push_view(views, offset, length);
    let mut accessor = json!({"bufferView": view_index, "componentType": 5126, "count": values.len(), "type": "VEC3"});
    if let Some((min, max)) = bounds {
        accessor["min"] = json!(min);
        accessor["max"] = json!(max);
    }
    push_accessor(accessors, accessor)
}

fn push_f32x2_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[[f32; 2]],
) -> usize {
    let (offset, length) = append_f32_rows(bin, values);
    let view_index = push_view(views, offset, length);
    push_accessor(
        accessors,
        json!({"bufferView": view_index, "componentType": 5126, "count": values.len(), "type": "VEC2"}),
    )
}

fn push_f32x4_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[[f32; 4]],
) -> usize {
    let (offset, length) = append_f32_rows(bin, values);
    let view_index = push_view(views, offset, length);
    push_accessor(
        accessors,
        json!({"bufferView": view_index, "componentType": 5126, "count": values.len(), "type": "VEC4"}),
    )
}

fn push_f32x16_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[[f32; 16]],
) -> usize {
    let (offset, length) = append_f32_rows(bin, values);
    let view_index = push_view(views, offset, length);
    push_accessor(
        accessors,
        json!({"bufferView": view_index, "componentType": 5126, "count": values.len(), "type": "MAT4"}),
    )
}

fn push_f32_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[f32],
    bounds: Option<(f32, f32)>,
) -> usize {
    align4(bin);
    let offset = bin.len();
    for value in values {
        bin.extend_from_slice(&value.to_le_bytes());
    }
    let view_index = push_view(views, offset, bin.len() - offset);
    let mut accessor = json!({"bufferView": view_index, "componentType": 5126, "count": values.len(), "type": "SCALAR"});
    if let Some((min, max)) = bounds {
        accessor["min"] = json!([min]);
        accessor["max"] = json!([max]);
    }
    push_accessor(accessors, accessor)
}

fn push_u16_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[u16],
) -> usize {
    align4(bin);
    let offset = bin.len();
    for value in values {
        bin.extend_from_slice(&value.to_le_bytes());
    }
    let view_index = push_view(views, offset, bin.len() - offset);
    push_accessor(
        accessors,
        json!({"bufferView": view_index, "componentType": 5123, "count": values.len(), "type": "SCALAR"}),
    )
}

fn push_u8x4_accessor(
    bin: &mut Vec<u8>,
    views: &mut Vec<Value>,
    accessors: &mut Vec<Value>,
    values: &[[u8; 4]],
) -> usize {
    align4(bin);
    let offset = bin.len();
    for value in values {
        bin.extend_from_slice(value);
    }
    let view_index = push_view(views, offset, bin.len() - offset);
    push_accessor(
        accessors,
        json!({"bufferView": view_index, "componentType": 5121, "count": values.len(), "type": "VEC4"}),
    )
}

fn append_f32_rows<const N: usize>(bin: &mut Vec<u8>, values: &[[f32; N]]) -> (usize, usize) {
    align4(bin);
    let offset = bin.len();
    for row in values {
        for value in row {
            bin.extend_from_slice(&value.to_le_bytes());
        }
    }
    (offset, bin.len() - offset)
}

fn push_view(views: &mut Vec<Value>, offset: usize, length: usize) -> usize {
    let index = views.len();
    views.push(view(offset, length));
    index
}

fn push_accessor(accessors: &mut Vec<Value>, value: Value) -> usize {
    let index = accessors.len();
    accessors.push(value);
    index
}

fn view(offset: usize, length: usize) -> Value {
    json!({"buffer": 0, "byteOffset": offset, "byteLength": length})
}

fn align4(bytes: &mut Vec<u8>) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
}

fn make_glb(root: Value, mut bin: Vec<u8>) -> Result<Vec<u8>, OwnedFixtureErrorV1> {
    let mut json_bytes = serde_json::to_vec(&root).map_err(|error| OwnedFixtureErrorV1 {
        schema_version: 1,
        code: "M6-FIXTURE-JSON-FAILED".to_owned(),
        message: error.to_string(),
    })?;
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    align4(&mut bin);
    let length = 12_usize
        .checked_add(8)
        .and_then(|v| v.checked_add(json_bytes.len()))
        .and_then(|v| v.checked_add(8))
        .and_then(|v| v.checked_add(bin.len()))
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| OwnedFixtureErrorV1 {
            schema_version: 1,
            code: "M6-FIXTURE-LENGTH-OVERFLOW".to_owned(),
            message: "owned GLB length exceeds u32".to_owned(),
        })?;
    let json_length = u32::try_from(json_bytes.len()).map_err(|_| OwnedFixtureErrorV1 {
        schema_version: 1,
        code: "M6-FIXTURE-LENGTH-OVERFLOW".to_owned(),
        message: "owned JSON length exceeds u32".to_owned(),
    })?;
    let bin_length = u32::try_from(bin.len()).map_err(|_| OwnedFixtureErrorV1 {
        schema_version: 1,
        code: "M6-FIXTURE-LENGTH-OVERFLOW".to_owned(),
        message: "owned BIN length exceeds u32".to_owned(),
    })?;
    let mut glb = Vec::with_capacity(length as usize);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&length.to_le_bytes());
    glb.extend_from_slice(&json_length.to_le_bytes());
    glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&bin_length.to_le_bytes());
    glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
    glb.extend_from_slice(&bin);
    Ok(glb)
}
