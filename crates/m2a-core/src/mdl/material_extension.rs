use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::model_ir::AuroraModelIrV1;

use super::{
    BinaryMdlArtifactV1, MdlWriteError, MdlWriterOptionsV1, NodeReport, inspect_binary_mdl,
    write_binary_mdl,
};

const FILE_HEADER_SIZE: usize = 0x0c;
const CONTROLLER_KEY_SIZE: usize = 0x0c;
const MESH_DIFFUSE: usize = 0xac;
const MESH_AMBIENT: usize = 0xb8;
const MESH_SPECULAR: usize = 0xc4;
const MESH_SHININESS: usize = 0xd0;
const MESH_TRANSPARENCY: usize = 0xe0;
const MESH_RENDER_HINT: usize = 0xe4;
const MESH_TEXTURE0: usize = 0xe8;
const MESH_TEXTURE_COUNT: usize = 0x232;
const MESH_UV1: usize = 0x238;
const MESH_TANGENT_XYZ: usize = 0x258;
const MESH_TANGENT_SIGN: usize = 0x260;
const NODE_CONTROLLER_KEYS: usize = 0x54;
const NODE_CONTROLLER_DATA: usize = 0x60;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdlRenderHintV1 {
    Automatic,
    Normal,
    NormalAndSpecMapped,
    NormalTangents,
}

impl MdlRenderHintV1 {
    fn binary_value(self) -> u32 {
        match self {
            Self::Automatic => 0,
            Self::Normal => 1,
            Self::NormalAndSpecMapped => 2,
            Self::NormalTangents => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlMaterialStateV1 {
    pub material_slot: u32,
    pub diffuse: [f32; 3],
    pub ambient: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub alpha: f32,
    pub self_illum_color: [f32; 3],
    pub transparency_hint: bool,
    pub render_hint: MdlRenderHintV1,
    pub normal_texture_resref: Option<String>,
    pub specular_texture_resref: Option<String>,
    pub material_resref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlSegmentMaterialStreamsV1 {
    pub segment_id: u32,
    #[serde(default)]
    pub uv1: Vec<[f32; 2]>,
    #[serde(default)]
    pub uv2: Vec<[f32; 2]>,
    #[serde(default)]
    pub uv3: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MdlMaterialExtensionOptionsV1 {
    pub schema_version: u32,
    pub materials: Vec<MdlMaterialStateV1>,
    pub segment_streams: Vec<MdlSegmentMaterialStreamsV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlMaterialSegmentReadbackV1 {
    pub segment_id: u32,
    pub material_slot: u32,
    pub node_core_offset: u32,
    pub uv_set_count: usize,
    pub tangent_count: usize,
    pub alpha_controller_present: bool,
    pub self_illum_controller_present: bool,
    pub semantic_match: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlMaterialExtensionReportV1 {
    pub schema_version: u32,
    pub payload_sha256: String,
    pub segment_readback: Vec<MdlMaterialSegmentReadbackV1>,
    pub semantic_diff: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MdlMaterialExtensionArtifactV1 {
    pub binary: BinaryMdlArtifactV1,
    pub material_report: MdlMaterialExtensionReportV1,
}

/// Writes the established binary MDL profile, extends its mesh material state
/// and raw streams, then parses the final bytes again. The returned artifact
/// never relies on the pre-extension inspection.
pub fn write_binary_mdl_with_materials_v1(
    model: &AuroraModelIrV1,
    writer_options: &MdlWriterOptionsV1,
    options: &MdlMaterialExtensionOptionsV1,
) -> Result<MdlMaterialExtensionArtifactV1, MdlWriteError> {
    let binary = write_binary_mdl(model, writer_options)?;
    extend_binary_mdl_with_materials_v1(model, binary, options)
}

/// Applies the verified material extension to an already-written MDL. This
/// is the shared bridge for animated Creature output: animation authoring is
/// completed first, then mesh material streams are patched and the complete
/// final payload is parsed again.
pub fn extend_binary_mdl_with_materials_v1(
    model: &AuroraModelIrV1,
    mut binary: BinaryMdlArtifactV1,
    options: &MdlMaterialExtensionOptionsV1,
) -> Result<MdlMaterialExtensionArtifactV1, MdlWriteError> {
    validate_options(model, options)?;
    let old_core_len = read_u32(&binary.payload, 4, "payload.coreLength")? as usize;
    let old_raw_len = read_u32(&binary.payload, 8, "payload.rawLength")? as usize;
    let raw_start = FILE_HEADER_SIZE + old_core_len;
    if raw_start.checked_add(old_raw_len) != Some(binary.payload.len()) {
        return Err(error(
            "M4M-INVALID-LAYOUT",
            "payload",
            "writer output core/raw ranges do not cover the payload",
        ));
    }
    let mut core = binary.payload[FILE_HEADER_SIZE..raw_start].to_vec();
    let mut raw = binary.payload[raw_start..].to_vec();
    let materials: HashMap<u32, &MdlMaterialStateV1> = options
        .materials
        .iter()
        .map(|material| (material.material_slot, material))
        .collect();
    let streams: HashMap<u32, &MdlSegmentMaterialStreamsV1> = options
        .segment_streams
        .iter()
        .map(|stream| (stream.segment_id, stream))
        .collect();

    for layout in &binary.report.layout.mesh_nodes {
        let segment = model
            .segments
            .iter()
            .find(|segment| segment.segment_id == layout.segment_id)
            .ok_or_else(|| {
                error(
                    "M4M-SEGMENT-MISSING",
                    "model.segments",
                    format!("layout references missing segment {}", layout.segment_id),
                )
            })?;
        let material = materials[&segment.material_slot];
        let base = layout.core_offset as usize;
        write_vec3(&mut core, base + MESH_DIFFUSE, material.diffuse)?;
        write_vec3(&mut core, base + MESH_AMBIENT, material.ambient)?;
        write_vec3(&mut core, base + MESH_SPECULAR, material.specular)?;
        write_f32(&mut core, base + MESH_SHININESS, material.shininess)?;
        write_u32(
            &mut core,
            base + MESH_TRANSPARENCY,
            u32::from(material.transparency_hint),
        )?;
        write_u32(
            &mut core,
            base + MESH_RENDER_HINT,
            material.render_hint.binary_value(),
        )?;
        write_optional_resref(
            &mut core,
            base + MESH_TEXTURE0 + 64,
            &material.normal_texture_resref,
        )?;
        write_optional_resref(
            &mut core,
            base + MESH_TEXTURE0 + 128,
            &material.specular_texture_resref,
        )?;
        write_optional_resref(
            &mut core,
            base + MESH_TEXTURE0 + 192,
            &material.material_resref,
        )?;

        let stream = streams.get(&segment.segment_id).copied();
        let uv_sets = [
            stream.map_or(&[][..], |value| value.uv1.as_slice()),
            stream.map_or(&[][..], |value| value.uv2.as_slice()),
            stream.map_or(&[][..], |value| value.uv3.as_slice()),
        ];
        let mut texture_count = 1usize;
        for (index, values) in uv_sets.iter().enumerate() {
            if values.is_empty() {
                write_i32(&mut core, base + MESH_UV1 + index * 4, -1)?;
                continue;
            }
            align_vec(&mut raw, 4);
            let pointer = raw.len();
            for value in *values {
                append_f32(&mut raw, value[0]);
                append_f32(&mut raw, value[1]);
            }
            write_i32(
                &mut core,
                base + MESH_UV1 + index * 4,
                as_i32(pointer, "segmentStreams.uv")?,
            )?;
            texture_count = index + 2;
        }
        write_u16(&mut core, base + MESH_TEXTURE_COUNT, texture_count as u16)?;

        if let Some(tangents) = &segment.tangents {
            align_vec(&mut raw, 4);
            let xyz_pointer = raw.len();
            for tangent in tangents {
                append_f32(&mut raw, tangent[0]);
                append_f32(&mut raw, tangent[1]);
                append_f32(&mut raw, tangent[2]);
            }
            align_vec(&mut raw, 4);
            let sign_pointer = raw.len();
            for tangent in tangents {
                append_f32(&mut raw, tangent[3]);
            }
            write_i32(
                &mut core,
                base + MESH_TANGENT_XYZ,
                as_i32(xyz_pointer, "segments.tangents.xyz")?,
            )?;
            write_i32(
                &mut core,
                base + MESH_TANGENT_SIGN,
                as_i32(sign_pointer, "segments.tangents.sign")?,
            )?;
        }
        extend_material_controllers(&mut core, base, material)?;
    }

    let mut payload = Vec::with_capacity(FILE_HEADER_SIZE + core.len() + raw.len());
    payload.extend_from_slice(&binary.payload[..FILE_HEADER_SIZE]);
    write_u32(&mut payload, 4, as_u32(core.len(), "payload.coreLength")?)?;
    write_u32(&mut payload, 8, as_u32(raw.len(), "payload.rawLength")?)?;
    payload.extend_from_slice(&core);
    payload.extend_from_slice(&raw);
    let inspection = inspect_binary_mdl(&payload).map_err(|source| {
        error(
            "M4M-READBACK-FAILED",
            "payload",
            format!("final material MDL failed parser readback: {source}"),
        )
    })?;
    let (segment_readback, semantic_diff) = compare_readback(
        model,
        options,
        &binary.report.layout.mesh_nodes,
        &inspection.node_tree.roots,
    );
    if !semantic_diff.is_empty() {
        return Err(error(
            "M4M-SEMANTIC-MISMATCH",
            "payload",
            semantic_diff.join("; "),
        ));
    }
    let payload_sha256 = format!("{:x}", Sha256::digest(&payload));
    binary.payload = payload;
    binary.inspection = inspection;
    binary.report.payload_sha256 = payload_sha256.clone();
    binary.report.layout.core_length = core.len();
    binary.report.layout.raw_length = raw.len();
    binary.report.layout.file_length = binary.payload.len();
    binary
        .report
        .deviations
        .retain(|entry| entry.code != "M4-TANGENTS-NOT-EMITTED");
    Ok(MdlMaterialExtensionArtifactV1 {
        binary,
        material_report: MdlMaterialExtensionReportV1 {
            schema_version: 1,
            payload_sha256,
            segment_readback,
            semantic_diff,
        },
    })
}

fn validate_options(
    model: &AuroraModelIrV1,
    options: &MdlMaterialExtensionOptionsV1,
) -> Result<(), MdlWriteError> {
    if options.schema_version != 1 {
        return Err(error(
            "M4M-SCHEMA-UNSUPPORTED",
            "materialOptions.schemaVersion",
            "expected schema version 1",
        ));
    }
    let mut slots = HashSet::new();
    for (index, material) in options.materials.iter().enumerate() {
        if !slots.insert(material.material_slot) {
            return Err(error(
                "M4M-MATERIAL-DUPLICATE",
                &format!("materialOptions.materials[{index}].materialSlot"),
                "material slot must be unique",
            ));
        }
        for (name, values) in [
            ("diffuse", material.diffuse.as_slice()),
            ("ambient", material.ambient.as_slice()),
            ("specular", material.specular.as_slice()),
            ("selfIllumColor", material.self_illum_color.as_slice()),
        ] {
            if values
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            {
                return Err(error(
                    "M4M-MATERIAL-RANGE",
                    &format!("materialOptions.materials[{index}].{name}"),
                    "color components must be finite and within 0..=1",
                ));
            }
        }
        if !material.shininess.is_finite() || !(0.0..=128.0).contains(&material.shininess) {
            return Err(error(
                "M4M-MATERIAL-RANGE",
                &format!("materialOptions.materials[{index}].shininess"),
                "shininess must be finite and within 0..=128",
            ));
        }
        if !material.alpha.is_finite() || !(0.0..=1.0).contains(&material.alpha) {
            return Err(error(
                "M4M-MATERIAL-RANGE",
                &format!("materialOptions.materials[{index}].alpha"),
                "alpha must be finite and within 0..=1",
            ));
        }
        for resref in [
            &material.normal_texture_resref,
            &material.specular_texture_resref,
            &material.material_resref,
        ]
        .into_iter()
        .flatten()
        {
            validate_resref(resref, "materialOptions.materials.resref")?;
        }
    }
    for segment in &model.segments {
        if !slots.contains(&segment.material_slot) {
            return Err(error(
                "M4M-MATERIAL-MISSING",
                "materialOptions.materials",
                format!(
                    "segment {} material slot {} has no state",
                    segment.segment_id, segment.material_slot
                ),
            ));
        }
        if segment.tangents.as_ref().is_some_and(|tangents| {
            tangents.len() != segment.positions.len()
                || tangents.iter().flatten().any(|value| !value.is_finite())
                || tangents
                    .iter()
                    .any(|value| value[3] != -1.0 && value[3] != 1.0)
        }) {
            return Err(error(
                "M4M-TANGENT-INVALID",
                "model.segments.tangents",
                "tangents must be finite, one per vertex, with handedness -1 or 1",
            ));
        }
    }
    let mut segment_ids = HashSet::new();
    for (index, stream) in options.segment_streams.iter().enumerate() {
        if !segment_ids.insert(stream.segment_id) {
            return Err(error(
                "M4M-STREAM-DUPLICATE",
                &format!("materialOptions.segmentStreams[{index}].segmentId"),
                "segment stream binding must be unique",
            ));
        }
        let segment = model
            .segments
            .iter()
            .find(|value| value.segment_id == stream.segment_id)
            .ok_or_else(|| {
                error(
                    "M4M-SEGMENT-MISSING",
                    &format!("materialOptions.segmentStreams[{index}].segmentId"),
                    "stream references a missing segment",
                )
            })?;
        let mut gap = false;
        for (name, values) in [
            ("uv1", &stream.uv1),
            ("uv2", &stream.uv2),
            ("uv3", &stream.uv3),
        ] {
            if values.is_empty() {
                gap = true;
            } else {
                if gap {
                    return Err(error(
                        "M4M-UV-GAP",
                        &format!("materialOptions.segmentStreams[{index}].{name}"),
                        "UV sets must be contiguous from UV0",
                    ));
                }
                if values.len() != segment.positions.len()
                    || values.iter().flatten().any(|value| !value.is_finite())
                {
                    return Err(error(
                        "M4M-UV-INVALID",
                        &format!("materialOptions.segmentStreams[{index}].{name}"),
                        "UV set must be finite and contain one value per vertex",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn extend_material_controllers(
    core: &mut Vec<u8>,
    base: usize,
    material: &MdlMaterialStateV1,
) -> Result<(), MdlWriteError> {
    let key_pointer = read_u32(core, base + NODE_CONTROLLER_KEYS, "mesh.controllerKeys")? as usize;
    let key_count = read_u32(
        core,
        base + NODE_CONTROLLER_KEYS + 4,
        "mesh.controllerKeyCount",
    )? as usize;
    let data_pointer = read_u32(core, base + NODE_CONTROLLER_DATA, "mesh.controllerData")? as usize;
    let data_count = read_u32(
        core,
        base + NODE_CONTROLLER_DATA + 4,
        "mesh.controllerDataCount",
    )? as usize;
    let mut keys = if key_count == 0 {
        Vec::new()
    } else {
        slice(
            core,
            key_pointer,
            key_count * CONTROLLER_KEY_SIZE,
            "mesh.controllerKeys",
        )?
        .to_vec()
    };
    let mut data = if data_count == 0 {
        Vec::new()
    } else {
        slice(core, data_pointer, data_count * 4, "mesh.controllerData")?.to_vec()
    };
    let self_time = data.len() / 4;
    append_f32(&mut data, 0.0);
    for value in material.self_illum_color {
        append_f32(&mut data, value);
    }
    append_controller_key(&mut keys, 100, self_time, self_time + 1, 3)?;
    let alpha_time = data.len() / 4;
    append_f32(&mut data, 0.0);
    append_f32(&mut data, material.alpha);
    append_controller_key(&mut keys, 128, alpha_time, alpha_time + 1, 1)?;
    align_vec(core, 4);
    let new_key_pointer = core.len();
    core.extend_from_slice(&keys);
    align_vec(core, 4);
    let new_data_pointer = core.len();
    core.extend_from_slice(&data);
    write_array_header(
        core,
        base + NODE_CONTROLLER_KEYS,
        new_key_pointer,
        keys.len() / CONTROLLER_KEY_SIZE,
    )?;
    write_array_header(
        core,
        base + NODE_CONTROLLER_DATA,
        new_data_pointer,
        data.len() / 4,
    )
}

fn compare_readback(
    model: &AuroraModelIrV1,
    options: &MdlMaterialExtensionOptionsV1,
    layouts: &[super::MdlMeshNodeLayoutV1],
    roots: &[NodeReport],
) -> (Vec<MdlMaterialSegmentReadbackV1>, Vec<String>) {
    let materials: HashMap<u32, &MdlMaterialStateV1> = options
        .materials
        .iter()
        .map(|value| (value.material_slot, value))
        .collect();
    let streams: HashMap<u32, &MdlSegmentMaterialStreamsV1> = options
        .segment_streams
        .iter()
        .map(|value| (value.segment_id, value))
        .collect();
    let mut reports = Vec::new();
    let mut diff = Vec::new();
    for layout in layouts {
        let Some(segment) = model
            .segments
            .iter()
            .find(|value| value.segment_id == layout.segment_id)
        else {
            continue;
        };
        let material = materials[&segment.material_slot];
        let node = find_node(roots, layout.core_offset);
        let mut matches = true;
        let mut uv_set_count = 0;
        let mut tangent_count = 0;
        let mut alpha = false;
        let mut self_illum = false;
        if let Some(node) = node {
            if let Some(mesh) = &node.mesh {
                let expected_stream = streams.get(&segment.segment_id).copied();
                let actual_uv = [&mesh.uv1, &mesh.uv2, &mesh.uv3];
                let expected_uv = [
                    expected_stream.map_or(&[][..], |v| v.uv1.as_slice()),
                    expected_stream.map_or(&[][..], |v| v.uv2.as_slice()),
                    expected_stream.map_or(&[][..], |v| v.uv3.as_slice()),
                ];
                uv_set_count = 1 + actual_uv.iter().filter(|value| !value.is_empty()).count();
                tangent_count = mesh.tangents.len();
                matches &= mesh.diffuse == material.diffuse
                    && mesh.ambient == material.ambient
                    && mesh.specular == material.specular
                    && mesh.shininess == material.shininess;
                matches &= mesh.transparency == u32::from(material.transparency_hint)
                    && mesh.render_hint == material.render_hint.binary_value();
                matches &= mesh.textures[1]
                    == material.normal_texture_resref.as_deref().unwrap_or("")
                    && mesh.textures[2]
                        == material.specular_texture_resref.as_deref().unwrap_or("")
                    && mesh.textures[3] == material.material_resref.as_deref().unwrap_or("");
                for index in 0..3 {
                    matches &= actual_uv[index].len() == expected_uv[index].len()
                        && actual_uv[index]
                            .iter()
                            .zip(expected_uv[index])
                            .all(|(a, e)| [a.x, a.y] == *e);
                }
                matches &= segment.tangents.as_deref().unwrap_or(&[]) == mesh.tangents;
            } else {
                matches = false;
            }
            self_illum = controller_matches(node, 100, &material.self_illum_color);
            alpha = controller_matches(node, 128, &[material.alpha]);
            matches &= self_illum && alpha;
        } else {
            matches = false;
        }
        if !matches {
            diff.push(format!(
                "segment {} material/raw readback differs",
                segment.segment_id
            ));
        }
        reports.push(MdlMaterialSegmentReadbackV1 {
            segment_id: segment.segment_id,
            material_slot: segment.material_slot,
            node_core_offset: layout.core_offset,
            uv_set_count,
            tangent_count,
            alpha_controller_present: alpha,
            self_illum_controller_present: self_illum,
            semantic_match: matches,
        });
    }
    (reports, diff)
}

fn controller_matches(node: &NodeReport, kind: i32, expected: &[f32]) -> bool {
    node.controllers.iter().any(|controller| {
        controller.controller_type == kind
            && controller.times == [0.0]
            && controller.values.as_slice() == [expected]
    })
}

fn find_node(nodes: &[NodeReport], offset: u32) -> Option<&NodeReport> {
    for node in nodes {
        if node.offset == offset {
            return Some(node);
        }
        if let Some(found) = find_node(&node.children, offset) {
            return Some(found);
        }
    }
    None
}

fn validate_resref(value: &str, path: &str) -> Result<(), MdlWriteError> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(error(
            "M4M-RESREF-INVALID",
            path,
            "resref must be 1..=16 ASCII alphanumeric/underscore characters",
        ));
    }
    Ok(())
}

fn write_optional_resref(
    bytes: &mut [u8],
    offset: usize,
    value: &Option<String>,
) -> Result<(), MdlWriteError> {
    let target = bytes.get_mut(offset..offset + 64).ok_or_else(|| {
        error(
            "M4M-LAYOUT-OVERFLOW",
            "payload",
            "texture field escapes core",
        )
    })?;
    target.fill(0);
    if let Some(value) = value {
        target[..value.len()].copy_from_slice(value.as_bytes());
    }
    Ok(())
}

fn append_controller_key(
    bytes: &mut Vec<u8>,
    kind: i32,
    time: usize,
    data: usize,
    columns: u8,
) -> Result<(), MdlWriteError> {
    bytes.extend_from_slice(&kind.to_le_bytes());
    bytes.extend_from_slice(&1i16.to_le_bytes());
    bytes.extend_from_slice(&as_i16(time, "controller.timeIndex")?.to_le_bytes());
    bytes.extend_from_slice(&as_i16(data, "controller.dataIndex")?.to_le_bytes());
    bytes.push(columns);
    bytes.push(0);
    Ok(())
}

fn write_array_header(
    bytes: &mut [u8],
    offset: usize,
    pointer: usize,
    count: usize,
) -> Result<(), MdlWriteError> {
    write_u32(bytes, offset, as_u32(pointer, "array.pointer")?)?;
    write_u32(bytes, offset + 4, as_u32(count, "array.count")?)?;
    write_u32(bytes, offset + 8, as_u32(count, "array.count")?)
}

fn slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    length: usize,
    path: &str,
) -> Result<&'a [u8], MdlWriteError> {
    bytes
        .get(
            offset
                ..offset
                    .checked_add(length)
                    .ok_or_else(|| error("M4M-LAYOUT-OVERFLOW", path, "range overflow"))?,
        )
        .ok_or_else(|| error("M4M-LAYOUT-OVERFLOW", path, "range escapes core"))
}

fn read_u32(bytes: &[u8], offset: usize, path: &str) -> Result<u32, MdlWriteError> {
    let value: [u8; 4] = slice(bytes, offset, 4, path)?
        .try_into()
        .expect("four bytes");
    Ok(u32::from_le_bytes(value))
}

fn write_vec3(bytes: &mut [u8], offset: usize, value: [f32; 3]) -> Result<(), MdlWriteError> {
    for (index, component) in value.into_iter().enumerate() {
        write_f32(bytes, offset + index * 4, component)?;
    }
    Ok(())
}

fn write_f32(bytes: &mut [u8], offset: usize, value: f32) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_i32(bytes: &mut [u8], offset: usize, value: i32) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}
fn write_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<(), MdlWriteError> {
    write_fixed(bytes, offset, value.to_le_bytes())
}

fn write_fixed<const N: usize>(
    bytes: &mut [u8],
    offset: usize,
    value: [u8; N],
) -> Result<(), MdlWriteError> {
    bytes
        .get_mut(offset..offset + N)
        .ok_or_else(|| error("M4M-LAYOUT-OVERFLOW", "payload", "write escapes buffer"))?
        .copy_from_slice(&value);
    Ok(())
}

fn append_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn align_vec(bytes: &mut Vec<u8>, alignment: usize) {
    while !bytes.len().is_multiple_of(alignment) {
        bytes.push(0);
    }
}
fn as_i32(value: usize, path: &str) -> Result<i32, MdlWriteError> {
    i32::try_from(value).map_err(|_| error("M4M-LAYOUT-OVERFLOW", path, "value cannot fit i32"))
}
fn as_u32(value: usize, path: &str) -> Result<u32, MdlWriteError> {
    u32::try_from(value).map_err(|_| error("M4M-LAYOUT-OVERFLOW", path, "value cannot fit u32"))
}
fn as_i16(value: usize, path: &str) -> Result<i16, MdlWriteError> {
    i16::try_from(value).map_err(|_| {
        error(
            "M4M-CONTROLLER-OVERFLOW",
            path,
            "controller index cannot fit i16",
        )
    })
}
fn error(code: &str, path: &str, message: impl Into<String>) -> MdlWriteError {
    MdlWriteError::fatal(code, path, message)
}
