use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::aabb_tree::{AabbEntryV1, AabbTreeV1, validate_aabb_tree_v1};
use super::ascii_common::{
    WalkmeshFaceV1, format_f32, validate_faces, validate_resref, validate_vertices,
};
use super::tile_navigation_ir::{TileNavigationIrV1, TileSurfaceV1, validate_tile_navigation_v1};
use super::{TileWalkmeshErrorV1, error};

const MAX_VERTEX_COUNT: usize = u16::MAX as usize;
const MAX_FACE_COUNT: usize = u16::MAX as usize;
const MAX_AABB_COUNT: usize = MAX_FACE_COUNT * 2 - 1;
const MAX_LINE_BYTES: usize = 255;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileWokInspectionV1 {
    pub schema_version: u32,
    pub format: String,
    pub byte_length: u32,
    pub model_resref: String,
    pub node_name: String,
    pub parent_name: String,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<WalkmeshFaceV1>,
    pub aabb_tree: AabbTreeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileWokReportV1 {
    pub schema_version: u32,
    pub format: String,
    pub model_resref: String,
    pub node_name: String,
    pub vertex_count: u32,
    pub face_count: u32,
    pub aabb_entry_count: u32,
    pub surface_id: i32,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileWokArtifactV1 {
    pub payload: Vec<u8>,
    pub inspection: TileWokInspectionV1,
    pub report: TileWokReportV1,
}

pub fn write_ascii_tile_wok_v1(
    navigation: &TileNavigationIrV1,
) -> Result<TileWokArtifactV1, TileWalkmeshErrorV1> {
    validate_tile_navigation_v1(navigation)?;
    let mut text = String::with_capacity(2048);
    push_line(&mut text, "#MAXWALKMESH ASCII")?;
    push_line(
        &mut text,
        &format!("beginwalkmeshgeom {}", navigation.model_resref),
    )?;
    push_line(&mut text, &format!("node aabb {}", navigation.node_name))?;
    push_line(&mut text, &format!("  parent {}", navigation.model_resref))?;
    push_line(
        &mut text,
        &format!(
            "  position {} {} {}",
            format_f32(navigation.position[0]),
            format_f32(navigation.position[1]),
            format_f32(navigation.position[2])
        ),
    )?;
    push_line(
        &mut text,
        &format!(
            "  orientation {} {} {} {}",
            format_f32(navigation.orientation[0]),
            format_f32(navigation.orientation[1]),
            format_f32(navigation.orientation[2]),
            format_f32(navigation.orientation[3])
        ),
    )?;
    push_line(&mut text, "  wirecolor 0.2 0.8 0.2")?;
    push_line(&mut text, &format!("  verts {}", navigation.vertices.len()))?;
    for vertex in &navigation.vertices {
        push_line(
            &mut text,
            &format!(
                "    {} {} {}",
                format_f32(vertex[0]),
                format_f32(vertex[1]),
                format_f32(vertex[2])
            ),
        )?;
    }
    push_line(&mut text, &format!("  faces {}", navigation.faces.len()))?;
    for face in &navigation.faces {
        push_line(
            &mut text,
            &format!(
                "    {} {} {} {} {} {} {} {}",
                face.vertex_indices[0],
                face.vertex_indices[1],
                face.vertex_indices[2],
                face.smoothing_group,
                face.adjacent_faces[0],
                face.adjacent_faces[1],
                face.adjacent_faces[2],
                face.surface_id
            ),
        )?;
    }
    let preorder = tree_preorder(&navigation.aabb_tree)?;
    push_line(&mut text, &format!("  aabb {}", preorder.len()))?;
    for entry in preorder {
        push_line(
            &mut text,
            &format!(
                "    {} {} {} {} {} {} {}",
                format_f32(entry.bounds_min[0]),
                format_f32(entry.bounds_min[1]),
                format_f32(entry.bounds_min[2]),
                format_f32(entry.bounds_max[0]),
                format_f32(entry.bounds_max[1]),
                format_f32(entry.bounds_max[2]),
                entry
                    .leaf_face
                    .map(|face| face.to_string())
                    .unwrap_or_else(|| "-1".to_owned())
            ),
        )?;
    }
    push_line(&mut text, "endnode")?;
    push_line(
        &mut text,
        &format!("endwalkmeshgeom {}", navigation.model_resref),
    )?;
    let payload = text.into_bytes();
    let inspection = inspect_ascii_tile_wok_v1(&payload)?;
    if inspection.model_resref != navigation.model_resref
        || inspection.node_name != navigation.node_name
        || inspection.vertices != navigation.vertices
        || inspection.faces != navigation.faces
        || inspection.aabb_tree != navigation.aabb_tree
    {
        return Err(error(
            "TILE-WOK-SEMANTIC-DIFF",
            "payload",
            "own WOK readback differs from TileNavigationIrV1",
        ));
    }
    Ok(TileWokArtifactV1 {
        report: TileWokReportV1 {
            schema_version: 1,
            format: inspection.format.clone(),
            model_resref: navigation.model_resref.clone(),
            node_name: navigation.node_name.clone(),
            vertex_count: navigation.vertices.len() as u32,
            face_count: navigation.faces.len() as u32,
            aabb_entry_count: navigation.aabb_tree.entries.len() as u32,
            surface_id: navigation.surface.id(),
            payload_sha256: sha256(&payload),
        },
        payload,
        inspection,
    })
}

pub fn inspect_ascii_tile_wok_v1(bytes: &[u8]) -> Result<TileWokInspectionV1, TileWalkmeshErrorV1> {
    if bytes.starts_with(&[0, 0, 0, 0]) {
        return Err(error(
            "TILE-WOK-BINARY-UNSUPPORTED",
            "wok[0..4]",
            "tile WOK must use the ASCII MAXWALKMESH envelope",
        ));
    }
    if bytes.contains(&0) || !bytes.is_ascii() {
        return Err(error(
            "TILE-WOK-NON-ASCII",
            "wok",
            "tile WOK must contain only non-NUL ASCII bytes",
        ));
    }
    for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.strip_suffix(b"\r").unwrap_or(line).len() > MAX_LINE_BYTES {
            return Err(error(
                "TILE-WOK-LINE-LIMIT",
                format!("wok.lines[{index}]"),
                "runtime line exceeds 255 bytes",
            ));
        }
    }
    let text = std::str::from_utf8(bytes).map_err(|source| {
        error(
            "TILE-WOK-NON-ASCII",
            "wok",
            format!("invalid ASCII/UTF-8: {source}"),
        )
    })?;
    let lines = text.lines().collect::<Vec<_>>();
    let mut cursor = 0_usize;
    let header = next_significant(&lines, &mut cursor)
        .ok_or_else(|| error("TILE-WOK-HEADER-MISSING", "wok", "empty WOK payload"))?;
    if !header.eq_ignore_ascii_case("#MAXWALKMESH ASCII") {
        return Err(error(
            "TILE-WOK-HEADER-INVALID",
            "wok.header",
            "expected #MAXWALKMESH ASCII",
        ));
    }
    let begin = tokens(next_significant(&lines, &mut cursor), "beginwalkmeshgeom")?;
    if begin.len() != 2 || !begin[0].eq_ignore_ascii_case("beginwalkmeshgeom") {
        return Err(error(
            "TILE-WOK-BEGIN-INVALID",
            "wok.beginwalkmeshgeom",
            "expected beginwalkmeshgeom <model_resref>",
        ));
    }
    let model_resref = begin[1].to_owned();
    validate_resref(&model_resref, "wok.modelResref")?;
    let node = tokens(next_significant(&lines, &mut cursor), "node")?;
    if node.len() != 3
        || !node[0].eq_ignore_ascii_case("node")
        || !node[1].eq_ignore_ascii_case("aabb")
    {
        return Err(error(
            "TILE-WOK-NODE-INVALID",
            "wok.node",
            "expected node aabb <node_name>",
        ));
    }
    let node_name = node[2].to_owned();
    let mut parent_name = None;
    let mut position = [0.0, 0.0, 0.0];
    let mut orientation = [1.0, 0.0, 0.0, 0.0];
    let mut vertices = None;
    let mut faces = None;
    let mut raw_aabb = None;
    let mut found_endnode = false;
    while cursor < lines.len() {
        let Some(line) = next_significant(&lines, &mut cursor) else {
            break;
        };
        if line.eq_ignore_ascii_case("endnode") {
            found_endnode = true;
            break;
        }
        let fields = line.split_whitespace().collect::<Vec<_>>();
        match fields
            .first()
            .map(|value| value.to_ascii_lowercase())
            .as_deref()
        {
            Some("parent") => {
                require_len(&fields, 2, "parent")?;
                parent_name = Some(fields[1].to_owned());
            }
            Some("position") => {
                require_len(&fields, 4, "position")?;
                position = parse_vec3(&fields[1..], "position")?;
            }
            Some("orientation") => {
                require_len(&fields, 5, "orientation")?;
                orientation = parse_vec4(&fields[1..], "orientation")?;
            }
            Some("verts") | Some("vertices") => {
                require_len(&fields, 2, "verts")?;
                let count = parse_count(fields[1], MAX_VERTEX_COUNT, "verts")?;
                let mut values = Vec::new();
                values.try_reserve_exact(count).map_err(|_| {
                    error(
                        "TILE-WOK-ALLOCATION",
                        "verts",
                        "vertex allocation failed after count preflight",
                    )
                })?;
                for index in 0..count {
                    let line = next_significant(&lines, &mut cursor).ok_or_else(|| {
                        error(
                            "TILE-WOK-COUNT-MISMATCH",
                            format!("verts[{index}]"),
                            "vertex array ended before declared count",
                        )
                    })?;
                    values.push(parse_vec3(
                        &line.split_whitespace().collect::<Vec<_>>(),
                        &format!("verts[{index}]"),
                    )?);
                }
                vertices = Some(values);
            }
            Some("faces") => {
                require_len(&fields, 2, "faces")?;
                let count = parse_count(fields[1], MAX_FACE_COUNT, "faces")?;
                let mut values = Vec::new();
                values.try_reserve_exact(count).map_err(|_| {
                    error(
                        "TILE-WOK-ALLOCATION",
                        "faces",
                        "face allocation failed after count preflight",
                    )
                })?;
                for index in 0..count {
                    let line = next_significant(&lines, &mut cursor).ok_or_else(|| {
                        error(
                            "TILE-WOK-COUNT-MISMATCH",
                            format!("faces[{index}]"),
                            "face array ended before declared count",
                        )
                    })?;
                    values.push(parse_face(line, index)?);
                }
                faces = Some(values);
            }
            Some("aabb") => {
                require_len(&fields, 2, "aabb")?;
                let count = parse_count(fields[1], MAX_AABB_COUNT, "aabb")?;
                let mut values = Vec::new();
                values.try_reserve_exact(count).map_err(|_| {
                    error(
                        "TILE-WOK-ALLOCATION",
                        "aabb",
                        "AABB allocation failed after count preflight",
                    )
                })?;
                for index in 0..count {
                    let line = next_significant(&lines, &mut cursor).ok_or_else(|| {
                        error(
                            "TILE-WOK-COUNT-MISMATCH",
                            format!("aabb[{index}]"),
                            "AABB array ended before declared count",
                        )
                    })?;
                    values.push(parse_ascii_aabb(line, index)?);
                }
                raw_aabb = Some(values);
            }
            // Retail WOKs carry additional material/bitmap records that do
            // not alter the audited V1 semantic projection.
            _ => {}
        }
    }
    if !found_endnode {
        return Err(error(
            "TILE-WOK-ENDNODE-MISSING",
            "wok.node",
            "WOK node is missing endnode",
        ));
    }
    let end = tokens(next_significant(&lines, &mut cursor), "endwalkmeshgeom")?;
    if end.len() != 2 || !end[0].eq_ignore_ascii_case("endwalkmeshgeom") || end[1] != model_resref {
        return Err(error(
            "TILE-WOK-END-INVALID",
            "wok.endwalkmeshgeom",
            "endwalkmeshgeom must repeat the exact model resref",
        ));
    }
    let parent_name = parent_name.ok_or_else(|| {
        error(
            "TILE-WOK-PARENT-MISSING",
            "wok.node.parent",
            "AABB node must declare parent",
        )
    })?;
    let vertices = vertices.ok_or_else(|| {
        error(
            "TILE-WOK-VERTS-MISSING",
            "wok.node.verts",
            "AABB node must declare verts",
        )
    })?;
    let faces = faces.ok_or_else(|| {
        error(
            "TILE-WOK-FACES-MISSING",
            "wok.node.faces",
            "AABB node must declare faces",
        )
    })?;
    let raw_aabb = raw_aabb.ok_or_else(|| {
        error(
            "TILE-WOK-AABB-MISSING",
            "wok.node.aabb",
            "AABB node must declare its tree",
        )
    })?;
    validate_vertices(&vertices, "wok.verts")?;
    validate_faces(&vertices, &faces, "wok.faces")?;
    if faces.iter().all(|face| face.surface_id == 7)
        || !faces
            .iter()
            .any(|face| matches!(face.surface_id, 1 | 3 | 4 | 5))
    {
        return Err(error(
            "TILE-WOK-NONWALK-FLOOR",
            "wok.faces.surfaceId",
            "tile WOK requires at least one audited Walk=1 surface and cannot use only surface 7",
        ));
    }
    let aabb_tree = reconstruct_preorder_tree(&raw_aabb)?;
    validate_aabb_tree_v1(&aabb_tree, &vertices, &faces)?;
    Ok(TileWokInspectionV1 {
        schema_version: 1,
        format: "MAXWALKMESH_ASCII".to_owned(),
        byte_length: bytes.len() as u32,
        model_resref,
        node_name,
        parent_name,
        position,
        orientation,
        vertices,
        faces,
        aabb_tree,
    })
}

fn tree_preorder(tree: &AabbTreeV1) -> Result<Vec<&AabbEntryV1>, TileWalkmeshErrorV1> {
    let mut output = Vec::with_capacity(tree.entries.len());
    let mut pending = vec![tree.root_index];
    while let Some(index) = pending.pop() {
        let entry = tree.entries.get(index as usize).ok_or_else(|| {
            error(
                "TILE-AABB-CHILD-OOB",
                format!("aabbTree.entries[{index}]"),
                "tree traversal escaped entry array",
            )
        })?;
        output.push(entry);
        if let (Some(left), Some(right)) = (entry.left, entry.right) {
            pending.push(right);
            pending.push(left);
        }
    }
    Ok(output)
}

#[derive(Clone, Copy)]
struct AsciiAabb {
    min: [f32; 3],
    max: [f32; 3],
    leaf: i32,
}

fn reconstruct_preorder_tree(records: &[AsciiAabb]) -> Result<AabbTreeV1, TileWalkmeshErrorV1> {
    let mut entries = Vec::with_capacity(records.len());
    let mut cursor = 0_usize;
    let root_index = reconstruct_node(records, &mut cursor, &mut entries)?;
    if cursor != records.len() {
        return Err(error(
            "TILE-WOK-AABB-COUNT-MISMATCH",
            "wok.aabb",
            "preorder AABB records contain trailing unreachable entries",
        ));
    }
    Ok(AabbTreeV1 {
        root_index,
        entries,
    })
}

fn reconstruct_node(
    records: &[AsciiAabb],
    cursor: &mut usize,
    entries: &mut Vec<AabbEntryV1>,
) -> Result<u32, TileWalkmeshErrorV1> {
    let record = records.get(*cursor).ok_or_else(|| {
        error(
            "TILE-WOK-AABB-COUNT-MISMATCH",
            "wok.aabb",
            "internal AABB record is missing one or both children",
        )
    })?;
    *cursor += 1;
    let index = entries.len() as u32;
    entries.push(AabbEntryV1 {
        bounds_min: record.min,
        bounds_max: record.max,
        left: None,
        right: None,
        leaf_face: (record.leaf >= 0).then_some(record.leaf as u32),
        plane: longest_axis(record.min, record.max),
    });
    if record.leaf < 0 {
        let left = reconstruct_node(records, cursor, entries)?;
        let right = reconstruct_node(records, cursor, entries)?;
        entries[index as usize].left = Some(left);
        entries[index as usize].right = Some(right);
    }
    Ok(index)
}

fn longest_axis(min: [f32; 3], max: [f32; 3]) -> u32 {
    let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    if extent[1] > extent[0] && extent[1] >= extent[2] {
        1
    } else if extent[2] > extent[0] && extent[2] > extent[1] {
        2
    } else {
        0
    }
}

fn push_line(text: &mut String, line: &str) -> Result<(), TileWalkmeshErrorV1> {
    if line.len() > MAX_LINE_BYTES {
        return Err(error(
            "TILE-WOK-LINE-LIMIT",
            "wok",
            "emitted runtime line exceeds 255 bytes",
        ));
    }
    writeln!(text, "{line}").expect("writing to String cannot fail");
    Ok(())
}

fn next_significant<'a>(lines: &'a [&str], cursor: &mut usize) -> Option<&'a str> {
    while *cursor < lines.len() {
        let line = lines[*cursor].trim();
        *cursor += 1;
        if !line.is_empty()
            && (!line.starts_with('#') || line.eq_ignore_ascii_case("#MAXWALKMESH ASCII"))
        {
            return Some(line);
        }
    }
    None
}

fn tokens<'a>(line: Option<&'a str>, path: &str) -> Result<Vec<&'a str>, TileWalkmeshErrorV1> {
    line.map(|value| value.split_whitespace().collect())
        .ok_or_else(|| {
            error(
                "TILE-WOK-UNEXPECTED-EOF",
                path,
                "WOK ended before required record",
            )
        })
}

fn require_len(fields: &[&str], expected: usize, path: &str) -> Result<(), TileWalkmeshErrorV1> {
    if fields.len() != expected {
        return Err(error(
            "TILE-WOK-FIELD-COUNT",
            path,
            format!("expected {expected} fields, got {}", fields.len()),
        ));
    }
    Ok(())
}

fn parse_count(value: &str, maximum: usize, path: &str) -> Result<usize, TileWalkmeshErrorV1> {
    let count = value.parse::<usize>().map_err(|_| {
        error(
            "TILE-WOK-COUNT-INVALID",
            path,
            "count must be a non-negative decimal integer",
        )
    })?;
    if count == 0 || count > maximum {
        return Err(error(
            "TILE-WOK-COUNT-LIMIT",
            path,
            format!("count must be in 1..={maximum}"),
        ));
    }
    Ok(count)
}

fn parse_f32(value: &str, path: &str) -> Result<f32, TileWalkmeshErrorV1> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| error("TILE-WOK-FLOAT-INVALID", path, "expected a finite float"))?;
    if !parsed.is_finite() {
        return Err(error("TILE-WOK-NONFINITE", path, "float must be finite"));
    }
    Ok(parsed)
}

fn parse_i32(value: &str, path: &str) -> Result<i32, TileWalkmeshErrorV1> {
    value.parse::<i32>().map_err(|_| {
        error(
            "TILE-WOK-INTEGER-INVALID",
            path,
            "expected a signed decimal integer",
        )
    })
}

fn parse_vec3(fields: &[&str], path: &str) -> Result<[f32; 3], TileWalkmeshErrorV1> {
    require_len(fields, 3, path)?;
    Ok([
        parse_f32(fields[0], path)?,
        parse_f32(fields[1], path)?,
        parse_f32(fields[2], path)?,
    ])
}

fn parse_vec4(fields: &[&str], path: &str) -> Result<[f32; 4], TileWalkmeshErrorV1> {
    require_len(fields, 4, path)?;
    Ok([
        parse_f32(fields[0], path)?,
        parse_f32(fields[1], path)?,
        parse_f32(fields[2], path)?,
        parse_f32(fields[3], path)?,
    ])
}

fn parse_face(line: &str, index: usize) -> Result<WalkmeshFaceV1, TileWalkmeshErrorV1> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    require_len(&fields, 8, &format!("faces[{index}]"))?;
    let parse_u32 = |field: usize| {
        fields[field].parse::<u32>().map_err(|_| {
            error(
                "TILE-WOK-INTEGER-INVALID",
                format!("faces[{index}][{field}]"),
                "face vertex index must be a non-negative integer",
            )
        })
    };
    Ok(WalkmeshFaceV1 {
        vertex_indices: [parse_u32(0)?, parse_u32(1)?, parse_u32(2)?],
        smoothing_group: parse_i32(fields[3], &format!("faces[{index}].smoothing"))?,
        adjacent_faces: [
            parse_i32(fields[4], &format!("faces[{index}].adjacent0"))?,
            parse_i32(fields[5], &format!("faces[{index}].adjacent1"))?,
            parse_i32(fields[6], &format!("faces[{index}].adjacent2"))?,
        ],
        surface_id: parse_i32(fields[7], &format!("faces[{index}].surfaceId"))?,
    })
}

fn parse_ascii_aabb(line: &str, index: usize) -> Result<AsciiAabb, TileWalkmeshErrorV1> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    require_len(&fields, 7, &format!("aabb[{index}]"))?;
    Ok(AsciiAabb {
        min: parse_vec3(&fields[0..3], &format!("aabb[{index}].min"))?,
        max: parse_vec3(&fields[3..6], &format!("aabb[{index}].max"))?,
        leaf: parse_i32(fields[6], &format!("aabb[{index}].leafFace"))?,
    })
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[allow(dead_code)]
fn surface_from_id(id: i32) -> Option<TileSurfaceV1> {
    match id {
        1 => Some(TileSurfaceV1::Dirt),
        3 => Some(TileSurfaceV1::Grass),
        4 => Some(TileSurfaceV1::Stone),
        5 => Some(TileSurfaceV1::Wood),
        _ => None,
    }
}
