//! Placeable PWK generation on top of the shared model IR.
//!
//! Render models remain native binary MDL resources. Placeable walkmeshes use
//! a separate ASCII resource grammar consumed by
//! `CNWPlaceableSurfaceMesh::LoadWalkMesh` for resource type `2053`.

use std::fmt::Write as _;
use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    model_ir::{AuroraModelIrV1, AuroraSegmentDeformationV1},
    walkmesh::{WalkmeshFaceV1, format_placeable_f32},
};

pub const PLACEABLE_PWK_SCHEMA_VERSION: u32 = 1;
pub const PLACEABLE_PWK_NONWALK_SURFACE_ID_V1: i32 = 7;
pub const PLACEABLE_PWK_RUNTIME_LINE_BUFFER_BYTES_V1: usize = 256;
pub const PLACEABLE_PWK_MAX_LINE_BYTES_V1: usize = PLACEABLE_PWK_RUNTIME_LINE_BUFFER_BYTES_V1 - 1;

const MIN_FOOTPRINT_EXTENT: f32 = 1.0e-4;
const MAX_PWK_VERTEX_COUNT: usize = 65_535;
const MAX_PWK_FACE_COUNT: usize = 65_535;
pub const CUSTOM_PLACEABLE_PWK_MAX_VERTEX_COUNT_V1: usize = 64;
const POLYGON_EPSILON_V1: f64 = 1.0e-8;

/// Backward-compatible placeable-domain name for the shared ASCII walkmesh
/// face grammar. PWK and WOK retain separate envelopes and validators.
pub type PlaceableWalkmeshFaceV1 = WalkmeshFaceV1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableWalkmeshIrV1 {
    pub schema_version: u32,
    pub model_resref: String,
    pub root_node_name: String,
    pub mesh_node_name: String,
    pub bounds_min: [f32; 2],
    pub bounds_max: [f32; 2],
    pub position: [f32; 3],
    pub orientation: [f32; 4],
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<PlaceableWalkmeshFaceV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableWalkmeshMeshInspectionV1 {
    pub node_name: String,
    pub parent_name: String,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<PlaceableWalkmeshFaceV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableWalkmeshUsePointV1 {
    pub node_name: String,
    pub parent_name: String,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableWalkmeshInspectionV1 {
    pub schema_version: u32,
    pub format: String,
    pub byte_length: u32,
    pub mesh_nodes: Vec<PlaceableWalkmeshMeshInspectionV1>,
    pub use_points: Vec<PlaceableWalkmeshUsePointV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableWalkmeshReportV1 {
    pub schema_version: u32,
    pub format: String,
    pub runtime_loader: String,
    pub model_resref: String,
    pub root_node_name: String,
    pub bounds_min: [f32; 2],
    pub bounds_max: [f32; 2],
    pub vertex_count: u32,
    pub face_count: u32,
    pub surface_id: i32,
    pub use_point_count: u32,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaceableWalkmeshArtifactV1 {
    pub payload: Vec<u8>,
    pub inspection: PlaceableWalkmeshInspectionV1,
    pub report: PlaceableWalkmeshReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableWalkmeshErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for PlaceableWalkmeshErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for PlaceableWalkmeshErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> PlaceableWalkmeshErrorV1 {
    PlaceableWalkmeshErrorV1 {
        schema_version: PLACEABLE_PWK_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

/// Derives the static placeable walkmesh domain IR from the same normalized
/// model IR used by the render-model pipeline.
pub fn derive_placeable_walkmesh_ir_v1(
    model: &AuroraModelIrV1,
    model_resref: &str,
) -> Result<PlaceableWalkmeshIrV1, PlaceableWalkmeshErrorV1> {
    validate_resref(model_resref)?;
    let (bounds_min, bounds_max) = world_xy_bounds(model)?;
    if bounds_max[0] - bounds_min[0] <= MIN_FOOTPRINT_EXTENT
        || bounds_max[1] - bounds_min[1] <= MIN_FOOTPRINT_EXTENT
    {
        return Err(error(
            "PLACEABLE-PWK-FOOTPRINT-DEGENERATE",
            "model.segments",
            "world-space XY collision footprint must have positive width and depth",
        ));
    }

    Ok(PlaceableWalkmeshIrV1 {
        schema_version: PLACEABLE_PWK_SCHEMA_VERSION,
        model_resref: model_resref.to_owned(),
        root_node_name: format!("{model_resref}_pwk"),
        mesh_node_name: format!("{model_resref}_wg"),
        bounds_min,
        bounds_max,
        position: [0.0, 0.0, 0.0],
        orientation: [1.0, 0.0, 0.0, 0.0],
        vertices: vec![
            [bounds_min[0], bounds_min[1], 0.0],
            [bounds_max[0], bounds_min[1], 0.0],
            [bounds_min[0], bounds_max[1], 0.0],
            [bounds_max[0], bounds_max[1], 0.0],
        ],
        faces: vec![
            PlaceableWalkmeshFaceV1 {
                vertex_indices: [2, 0, 3],
                smoothing_group: 1,
                // The exact runtime parser consumes eight integer fields but
                // stores only v0/v1/v2 and surface. Retail static PWKs use
                // zeroes in these three compatibility slots.
                adjacent_faces: [0, 0, 0],
                surface_id: PLACEABLE_PWK_NONWALK_SURFACE_ID_V1,
            },
            PlaceableWalkmeshFaceV1 {
                vertex_indices: [1, 3, 0],
                smoothing_group: 1,
                adjacent_faces: [0, 0, 0],
                surface_id: PLACEABLE_PWK_NONWALK_SURFACE_ID_V1,
            },
        ],
    })
}

/// Builds one deterministic, flat Nonwalk PWK from a caller-authored simple
/// polygon in final Aurora XY metres. The input may use either winding; the
/// returned IR is canonical and therefore produces byte-identical ASCII for
/// geometrically identical ordered rings.
pub fn custom_placeable_walkmesh_ir_v1(
    model_resref: &str,
    vertices_xy: &[[f32; 2]],
) -> Result<PlaceableWalkmeshIrV1, PlaceableWalkmeshErrorV1> {
    validate_resref(model_resref)?;
    let vertices_xy = canonical_polygon_v1(vertices_xy)?;
    let triangles = triangulate_simple_polygon_v1(&vertices_xy)?;
    let mut bounds_min = [f32::INFINITY; 2];
    let mut bounds_max = [f32::NEG_INFINITY; 2];
    for vertex in &vertices_xy {
        for axis in 0..2 {
            bounds_min[axis] = bounds_min[axis].min(vertex[axis]);
            bounds_max[axis] = bounds_max[axis].max(vertex[axis]);
        }
    }
    let walkmesh = PlaceableWalkmeshIrV1 {
        schema_version: PLACEABLE_PWK_SCHEMA_VERSION,
        model_resref: model_resref.to_owned(),
        root_node_name: format!("{model_resref}_pwk"),
        mesh_node_name: format!("{model_resref}_wg"),
        bounds_min,
        bounds_max,
        position: [0.0, 0.0, 0.0],
        orientation: [1.0, 0.0, 0.0, 0.0],
        vertices: vertices_xy
            .into_iter()
            .map(|vertex| [vertex[0], vertex[1], 0.0])
            .collect(),
        faces: triangles
            .into_iter()
            .map(|vertex_indices| PlaceableWalkmeshFaceV1 {
                vertex_indices,
                smoothing_group: 1,
                adjacent_faces: [0, 0, 0],
                surface_id: PLACEABLE_PWK_NONWALK_SURFACE_ID_V1,
            })
            .collect(),
    };
    validate_walkmesh_ir(&walkmesh)?;
    Ok(walkmesh)
}

fn canonical_polygon_v1(input: &[[f32; 2]]) -> Result<Vec<[f32; 2]>, PlaceableWalkmeshErrorV1> {
    let mut vertices = input.to_vec();
    if vertices.len() >= 4 && same_point_v1(vertices[0], *vertices.last().unwrap()) {
        vertices.pop();
    }
    if !(3..=CUSTOM_PLACEABLE_PWK_MAX_VERTEX_COUNT_V1).contains(&vertices.len()) {
        return Err(error(
            "PLACEABLE-PWK-POLYGON-VERTEX-COUNT",
            "collision.vertices",
            format!(
                "custom polygon requires 3..={} vertices after optional closure removal",
                CUSTOM_PLACEABLE_PWK_MAX_VERTEX_COUNT_V1
            ),
        ));
    }
    for (index, vertex) in vertices.iter().enumerate() {
        if vertex.iter().any(|coordinate| !coordinate.is_finite()) {
            return Err(error(
                "PLACEABLE-PWK-POLYGON-NUMERIC-INVALID",
                format!("collision.vertices[{index}]"),
                "custom polygon coordinates must be finite",
            ));
        }
        let next = vertices[(index + 1) % vertices.len()];
        if same_point_v1(*vertex, next) {
            return Err(error(
                "PLACEABLE-PWK-POLYGON-VERTEX-DUPLICATE",
                format!("collision.vertices[{index}]"),
                "consecutive custom polygon vertices must be distinct",
            ));
        }
    }
    for left in 0..vertices.len() {
        for right in left + 1..vertices.len() {
            if same_point_v1(vertices[left], vertices[right]) {
                return Err(error(
                    "PLACEABLE-PWK-POLYGON-VERTEX-DUPLICATE",
                    format!("collision.vertices[{right}]"),
                    format!("custom polygon vertex duplicates vertex {left}"),
                ));
            }
        }
    }
    if polygon_self_intersects_v1(&vertices) {
        return Err(error(
            "PLACEABLE-PWK-POLYGON-SELF-INTERSECTION",
            "collision.vertices",
            "custom polygon edges must not self-intersect",
        ));
    }
    let area = signed_area_v1(&vertices);
    if !area.is_finite() || area.abs() <= POLYGON_EPSILON_V1 {
        return Err(error(
            "PLACEABLE-PWK-POLYGON-AREA-DEGENERATE",
            "collision.vertices",
            "custom polygon must have a finite non-zero area",
        ));
    }
    if area < 0.0 {
        vertices.reverse();
    }
    let first = vertices
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            left[0]
                .total_cmp(&right[0])
                .then_with(|| left[1].total_cmp(&right[1]))
        })
        .map(|(index, _)| index)
        .expect("validated polygon is non-empty");
    vertices.rotate_left(first);
    Ok(vertices)
}

fn triangulate_simple_polygon_v1(
    vertices: &[[f32; 2]],
) -> Result<Vec<[u32; 3]>, PlaceableWalkmeshErrorV1> {
    let mut ring = (0..vertices.len()).collect::<Vec<_>>();
    let mut triangles = Vec::with_capacity(vertices.len() - 2);
    while ring.len() > 3 {
        let mut ear = None;
        for cursor in 0..ring.len() {
            let previous = ring[(cursor + ring.len() - 1) % ring.len()];
            let current = ring[cursor];
            let next = ring[(cursor + 1) % ring.len()];
            if cross2_v1(vertices[previous], vertices[current], vertices[next])
                <= POLYGON_EPSILON_V1
            {
                continue;
            }
            if ring.iter().copied().any(|candidate| {
                candidate != previous
                    && candidate != current
                    && candidate != next
                    && point_in_triangle_v1(
                        vertices[candidate],
                        vertices[previous],
                        vertices[current],
                        vertices[next],
                    )
            }) {
                continue;
            }
            ear = Some((cursor, [previous as u32, current as u32, next as u32]));
            break;
        }
        let Some((cursor, triangle)) = ear else {
            return Err(error(
                "PLACEABLE-PWK-POLYGON-TRIANGULATION-FAILED",
                "collision.vertices",
                "custom polygon could not be triangulated deterministically",
            ));
        };
        triangles.push(triangle);
        ring.remove(cursor);
    }
    triangles.push([ring[0] as u32, ring[1] as u32, ring[2] as u32]);
    Ok(triangles)
}

fn same_point_v1(left: [f32; 2], right: [f32; 2]) -> bool {
    left[0].to_bits() == right[0].to_bits() && left[1].to_bits() == right[1].to_bits()
}

fn signed_area_v1(vertices: &[[f32; 2]]) -> f64 {
    vertices
        .iter()
        .enumerate()
        .map(|(index, left)| {
            let right = vertices[(index + 1) % vertices.len()];
            f64::from(left[0]) * f64::from(right[1]) - f64::from(right[0]) * f64::from(left[1])
        })
        .sum::<f64>()
        * 0.5
}

fn cross2_v1(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f64 {
    (f64::from(b[0]) - f64::from(a[0])) * (f64::from(c[1]) - f64::from(a[1]))
        - (f64::from(b[1]) - f64::from(a[1])) * (f64::from(c[0]) - f64::from(a[0]))
}

fn orientation_v1(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> i8 {
    let value = cross2_v1(a, b, c);
    if value > POLYGON_EPSILON_V1 {
        1
    } else if value < -POLYGON_EPSILON_V1 {
        -1
    } else {
        0
    }
}

fn point_on_segment_v1(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> bool {
    orientation_v1(a, b, point) == 0
        && f64::from(point[0]) >= f64::from(a[0].min(b[0])) - POLYGON_EPSILON_V1
        && f64::from(point[0]) <= f64::from(a[0].max(b[0])) + POLYGON_EPSILON_V1
        && f64::from(point[1]) >= f64::from(a[1].min(b[1])) - POLYGON_EPSILON_V1
        && f64::from(point[1]) <= f64::from(a[1].max(b[1])) + POLYGON_EPSILON_V1
}

fn segments_intersect_v1(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    let ab_c = orientation_v1(a, b, c);
    let ab_d = orientation_v1(a, b, d);
    let cd_a = orientation_v1(c, d, a);
    let cd_b = orientation_v1(c, d, b);
    (ab_c != ab_d && cd_a != cd_b)
        || (ab_c == 0 && point_on_segment_v1(c, a, b))
        || (ab_d == 0 && point_on_segment_v1(d, a, b))
        || (cd_a == 0 && point_on_segment_v1(a, c, d))
        || (cd_b == 0 && point_on_segment_v1(b, c, d))
}

fn polygon_self_intersects_v1(vertices: &[[f32; 2]]) -> bool {
    for left in 0..vertices.len() {
        let left_next = (left + 1) % vertices.len();
        for right in left + 1..vertices.len() {
            let right_next = (right + 1) % vertices.len();
            if left == right
                || left_next == right
                || right_next == left
                || (left == 0 && right_next == 0)
            {
                continue;
            }
            if segments_intersect_v1(
                vertices[left],
                vertices[left_next],
                vertices[right],
                vertices[right_next],
            ) {
                return true;
            }
        }
    }
    false
}

fn point_in_triangle_v1(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    cross2_v1(a, b, point) >= -POLYGON_EPSILON_V1
        && cross2_v1(b, c, point) >= -POLYGON_EPSILON_V1
        && cross2_v1(c, a, point) >= -POLYGON_EPSILON_V1
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Serializes a validated placeable walkmesh IR using the newline-delimited
/// grammar consumed by `CNWPlaceableSurfaceMesh::LoadWalkMesh`.
pub fn write_ascii_placeable_walkmesh_v1(
    walkmesh: &PlaceableWalkmeshIrV1,
) -> Result<PlaceableWalkmeshArtifactV1, PlaceableWalkmeshErrorV1> {
    validate_walkmesh_ir(walkmesh)?;

    let mut text = String::with_capacity(512);
    push_runtime_line(&mut text, "# Generated by Meshy2Aurora")?;
    push_runtime_line(&mut text, "# pwk file")?;
    push_runtime_line(&mut text, "# Meshy2Aurora PWKMESH ASCII")?;
    push_runtime_line(
        &mut text,
        &format!("node trimesh {}", walkmesh.mesh_node_name),
    )?;
    push_runtime_line(&mut text, &format!("  parent {}", walkmesh.root_node_name))?;
    push_runtime_line(
        &mut text,
        &format!(
            "  position {} {} {}",
            format_placeable_f32(walkmesh.position[0]),
            format_placeable_f32(walkmesh.position[1]),
            format_placeable_f32(walkmesh.position[2])
        ),
    )?;
    push_runtime_line(
        &mut text,
        &format!(
            "  orientation {} {} {} {}",
            format_placeable_f32(walkmesh.orientation[0]),
            format_placeable_f32(walkmesh.orientation[1]),
            format_placeable_f32(walkmesh.orientation[2]),
            format_placeable_f32(walkmesh.orientation[3])
        ),
    )?;
    push_runtime_line(&mut text, "  wirecolor 0.690196 0.101961 0.101961")?;
    push_runtime_line(&mut text, "  bitmap NULL")?;
    push_runtime_line(&mut text, &format!("  verts {}", walkmesh.vertices.len()))?;
    for vertex in &walkmesh.vertices {
        push_runtime_line(
            &mut text,
            &format!(
                "    {} {} {}",
                format_placeable_f32(vertex[0]),
                format_placeable_f32(vertex[1]),
                format_placeable_f32(vertex[2])
            ),
        )?;
    }
    push_runtime_line(&mut text, &format!("  faces {}", walkmesh.faces.len()))?;
    for face in &walkmesh.faces {
        let mut line = String::with_capacity(64);
        write!(
            line,
            "    {} {} {} {} {} {} {} {}",
            face.vertex_indices[0],
            face.vertex_indices[1],
            face.vertex_indices[2],
            face.smoothing_group,
            face.adjacent_faces[0],
            face.adjacent_faces[1],
            face.adjacent_faces[2],
            face.surface_id
        )
        .expect("writing to String cannot fail");
        push_runtime_line(&mut text, &line)?;
    }
    push_runtime_line(&mut text, "endnode")?;

    let payload = text.into_bytes();
    let inspection = inspect_ascii_placeable_walkmesh_v1(&payload)?;
    validate_writer_readback(walkmesh, &inspection)?;

    Ok(PlaceableWalkmeshArtifactV1 {
        report: PlaceableWalkmeshReportV1 {
            schema_version: PLACEABLE_PWK_SCHEMA_VERSION,
            format: inspection.format.clone(),
            runtime_loader: "CNWPlaceableSurfaceMesh::LoadWalkMesh".to_owned(),
            model_resref: walkmesh.model_resref.clone(),
            root_node_name: walkmesh.root_node_name.clone(),
            bounds_min: walkmesh.bounds_min,
            bounds_max: walkmesh.bounds_max,
            vertex_count: walkmesh.vertices.len() as u32,
            face_count: walkmesh.faces.len() as u32,
            surface_id: PLACEABLE_PWK_NONWALK_SURFACE_ID_V1,
            use_point_count: inspection.use_points.len() as u32,
            payload_sha256: sha256(&payload),
        },
        payload,
        inspection,
    })
}

/// Convenience entry point used by the placeable package builder.
pub fn write_placeable_walkmesh_v1(
    model: &AuroraModelIrV1,
    model_resref: &str,
) -> Result<PlaceableWalkmeshArtifactV1, PlaceableWalkmeshErrorV1> {
    let walkmesh = derive_placeable_walkmesh_ir_v1(model, model_resref)?;
    write_ascii_placeable_walkmesh_v1(&walkmesh)
}

/// Parses the ASCII PWK subset used by the exact NWN placeable surface-mesh
/// loader and validates counts, numeric fields and face references.
pub fn inspect_ascii_placeable_walkmesh_v1(
    bytes: &[u8],
) -> Result<PlaceableWalkmeshInspectionV1, PlaceableWalkmeshErrorV1> {
    if bytes.starts_with(&[0, 0, 0, 0]) {
        return Err(error(
            "PLACEABLE-PWK-BINARY-UNSUPPORTED",
            "pwk[0..4]",
            "runtime-ready placeable PWK must be ASCII, not binary MDL",
        ));
    }
    if bytes.contains(&0) {
        return Err(error(
            "PLACEABLE-PWK-NUL-BYTE",
            "pwk",
            "ASCII PWK must not contain NUL bytes",
        ));
    }
    if !bytes.is_ascii() {
        return Err(error(
            "PLACEABLE-PWK-NON-ASCII",
            "pwk",
            "placeable PWK must contain only ASCII bytes",
        ));
    }
    validate_runtime_line_lengths(bytes)?;

    let text = std::str::from_utf8(bytes).map_err(|source| {
        error(
            "PLACEABLE-PWK-NON-ASCII",
            "pwk",
            format!("placeable PWK is not valid UTF-8/ASCII: {source}"),
        )
    })?;
    let lines = text.lines().collect::<Vec<_>>();
    let mut cursor = 0_usize;
    let mut mesh_nodes = Vec::new();
    let mut use_points = Vec::new();

    while cursor < lines.len() {
        let line = lines[cursor].trim();
        if line.is_empty() || line.starts_with('#') {
            cursor += 1;
            continue;
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        if tokens.first().copied() != Some("node") {
            // The runtime scans for node records and ignores unrelated
            // top-level metadata such as retail `filedependancy` lines.
            cursor += 1;
            continue;
        }
        if tokens.len() != 3 {
            return Err(error(
                "PLACEABLE-PWK-NODE-INVALID",
                format!("pwk.lines[{cursor}]"),
                "node record must contain exactly node, kind and name",
            ));
        }
        let node_kind = tokens[1];
        let node_name = tokens[2].to_owned();
        if !matches!(node_kind, "trimesh" | "dummy") {
            return Err(error(
                "PLACEABLE-PWK-NODE-KIND-UNSUPPORTED",
                format!("pwk.lines[{cursor}]"),
                format!("unsupported PWK node kind {node_kind}"),
            ));
        }

        cursor += 1;
        let mut parent_name = None;
        let mut position = None;
        let mut orientation = None;
        let mut vertices = None;
        let mut faces = None;
        let mut found_endnode = false;

        while cursor < lines.len() {
            let node_line = lines[cursor].trim();
            if node_line.is_empty() || node_line.starts_with('#') {
                cursor += 1;
                continue;
            }
            if node_line == "endnode" {
                found_endnode = true;
                cursor += 1;
                break;
            }
            let fields = node_line.split_whitespace().collect::<Vec<_>>();
            match fields.first().copied().unwrap_or_default() {
                "parent" => {
                    require_field_count(&fields, 2, cursor, "parent")?;
                    set_once(&mut parent_name, fields[1].to_owned(), cursor, "parent")?;
                    cursor += 1;
                }
                "position" => {
                    require_field_count(&fields, 4, cursor, "position")?;
                    let value = parse_f32_triplet(&fields[1..], cursor, "position")?;
                    set_once(&mut position, value, cursor, "position")?;
                    cursor += 1;
                }
                "orientation" => {
                    require_field_count(&fields, 5, cursor, "orientation")?;
                    let value = parse_f32_quad(&fields[1..], cursor, "orientation")?;
                    set_once(&mut orientation, value, cursor, "orientation")?;
                    cursor += 1;
                }
                "verts" | "vertices" => {
                    require_field_count(&fields, 2, cursor, "verts")?;
                    let count = parse_count(fields[1], MAX_PWK_VERTEX_COUNT, cursor, "verts")?;
                    if vertices.is_some() {
                        return Err(duplicate_field(cursor, "verts"));
                    }
                    cursor += 1;
                    let mut parsed = Vec::with_capacity(count);
                    for index in 0..count {
                        let value =
                            parse_data_triplet(&lines, &mut cursor, &format!("verts[{index}]"))?;
                        parsed.push(value);
                    }
                    vertices = Some(parsed);
                }
                "faces" => {
                    require_field_count(&fields, 2, cursor, "faces")?;
                    let count = parse_count(fields[1], MAX_PWK_FACE_COUNT, cursor, "faces")?;
                    if faces.is_some() {
                        return Err(duplicate_field(cursor, "faces"));
                    }
                    cursor += 1;
                    let mut parsed = Vec::with_capacity(count);
                    for index in 0..count {
                        parsed.push(parse_face_line(&lines, &mut cursor, index)?);
                    }
                    faces = Some(parsed);
                }
                // These records are accepted by retail PWKs and ignored by
                // the exact placeable collision loader for geometry.
                _ => cursor += 1,
            }
        }

        if !found_endnode {
            return Err(error(
                "PLACEABLE-PWK-ENDNODE-MISSING",
                format!("pwk.node[{node_name}]"),
                "PWK node is missing endnode",
            ));
        }
        let parent_name = parent_name.ok_or_else(|| {
            error(
                "PLACEABLE-PWK-PARENT-MISSING",
                format!("pwk.node[{node_name}].parent"),
                "PWK node must declare parent",
            )
        })?;
        let position = position.unwrap_or([0.0, 0.0, 0.0]);
        let orientation = orientation.unwrap_or([1.0, 0.0, 0.0, 0.0]);

        if node_kind == "trimesh" {
            let vertices = vertices.ok_or_else(|| {
                error(
                    "PLACEABLE-PWK-VERTS-MISSING",
                    format!("pwk.node[{node_name}].verts"),
                    "trimesh node must declare verts",
                )
            })?;
            let faces = faces.ok_or_else(|| {
                error(
                    "PLACEABLE-PWK-FACES-MISSING",
                    format!("pwk.node[{node_name}].faces"),
                    "trimesh node must declare faces",
                )
            })?;
            validate_face_indices(&faces, vertices.len(), &node_name)?;
            mesh_nodes.push(PlaceableWalkmeshMeshInspectionV1 {
                node_name,
                parent_name,
                position,
                orientation,
                vertices,
                faces,
            });
        } else if node_name.contains("_pwk_use") || node_name.contains("_pwk_dp_use_") {
            use_points.push(PlaceableWalkmeshUsePointV1 {
                node_name,
                parent_name,
                position,
                orientation,
            });
        }
    }

    if mesh_nodes.is_empty() {
        return Err(error(
            "PLACEABLE-PWK-MESH-MISSING",
            "pwk",
            "ASCII PWK must contain at least one trimesh node",
        ));
    }

    Ok(PlaceableWalkmeshInspectionV1 {
        schema_version: PLACEABLE_PWK_SCHEMA_VERSION,
        format: "nwn1-ascii-pwk".to_owned(),
        byte_length: bytes.len().try_into().map_err(|_| {
            error(
                "PLACEABLE-PWK-SIZE-OVERFLOW",
                "pwk",
                "PWK byte length does not fit u32",
            )
        })?,
        mesh_nodes,
        use_points,
    })
}

fn validate_walkmesh_ir(walkmesh: &PlaceableWalkmeshIrV1) -> Result<(), PlaceableWalkmeshErrorV1> {
    if walkmesh.schema_version != PLACEABLE_PWK_SCHEMA_VERSION {
        return Err(error(
            "PLACEABLE-PWK-SCHEMA-UNSUPPORTED",
            "walkmesh.schemaVersion",
            format!(
                "expected schema version {}, got {}",
                PLACEABLE_PWK_SCHEMA_VERSION, walkmesh.schema_version
            ),
        ));
    }
    validate_resref(&walkmesh.model_resref)?;
    if walkmesh.root_node_name != format!("{}_pwk", walkmesh.model_resref) {
        return Err(error(
            "PLACEABLE-PWK-ROOT-NAME-INVALID",
            "walkmesh.rootNodeName",
            "static PWK parent must be <modelResref>_pwk",
        ));
    }
    if walkmesh.mesh_node_name != format!("{}_wg", walkmesh.model_resref) {
        return Err(error(
            "PLACEABLE-PWK-MESH-NAME-INVALID",
            "walkmesh.meshNodeName",
            "static PWK mesh must be <modelResref>_wg",
        ));
    }
    if walkmesh.vertices.is_empty() || walkmesh.faces.is_empty() {
        return Err(error(
            "PLACEABLE-PWK-MESH-EMPTY",
            "walkmesh",
            "static PWK requires non-empty vertices and faces",
        ));
    }
    if walkmesh
        .position
        .iter()
        .chain(walkmesh.orientation.iter())
        .chain(walkmesh.vertices.iter().flatten())
        .any(|value| !value.is_finite())
    {
        return Err(error(
            "PLACEABLE-PWK-NUMERIC-INVALID",
            "walkmesh",
            "PWK transforms and vertices must be finite",
        ));
    }
    validate_face_indices(
        &walkmesh.faces,
        walkmesh.vertices.len(),
        &walkmesh.mesh_node_name,
    )?;
    if walkmesh
        .faces
        .iter()
        .any(|face| face.surface_id != PLACEABLE_PWK_NONWALK_SURFACE_ID_V1)
    {
        return Err(error(
            "PLACEABLE-PWK-SURFACE-INVALID",
            "walkmesh.faces",
            "static collision PWK requires surface 7 (Nonwalk) on every face",
        ));
    }
    Ok(())
}

fn validate_writer_readback(
    expected: &PlaceableWalkmeshIrV1,
    actual: &PlaceableWalkmeshInspectionV1,
) -> Result<(), PlaceableWalkmeshErrorV1> {
    if actual.mesh_nodes.len() != 1 || !actual.use_points.is_empty() {
        return Err(error(
            "PLACEABLE-PWK-READBACK-DIFF",
            "walkmesh.nodes",
            "static PWK writer must round-trip one trimesh and zero use points",
        ));
    }
    let mesh = &actual.mesh_nodes[0];
    if mesh.node_name != expected.mesh_node_name
        || mesh.parent_name != expected.root_node_name
        || mesh.position != expected.position
        || mesh.orientation != expected.orientation
        || mesh.vertices != expected.vertices
        || mesh.faces != expected.faces
    {
        return Err(error(
            "PLACEABLE-PWK-READBACK-DIFF",
            "walkmesh",
            "ASCII writer/readback did not preserve the placeable walkmesh IR",
        ));
    }
    Ok(())
}

fn validate_runtime_line_lengths(bytes: &[u8]) -> Result<(), PlaceableWalkmeshErrorV1> {
    for (line_index, line) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let content_length = line.len() - usize::from(line.ends_with(b"\n"));
        if content_length > PLACEABLE_PWK_MAX_LINE_BYTES_V1 {
            return Err(error(
                "PLACEABLE-PWK-LINE-TOO-LONG",
                format!("pwk.lines[{line_index}]"),
                format!(
                    "line has {content_length} bytes; runtime-safe maximum is {}",
                    PLACEABLE_PWK_MAX_LINE_BYTES_V1
                ),
            ));
        }
    }
    Ok(())
}

fn push_runtime_line(output: &mut String, line: &str) -> Result<(), PlaceableWalkmeshErrorV1> {
    if !line.is_ascii() {
        return Err(error(
            "PLACEABLE-PWK-NON-ASCII",
            "walkmesh",
            "writer line contains non-ASCII characters",
        ));
    }
    if line.len() > PLACEABLE_PWK_MAX_LINE_BYTES_V1 {
        return Err(error(
            "PLACEABLE-PWK-LINE-TOO-LONG",
            "walkmesh",
            format!(
                "writer line has {} bytes; runtime-safe maximum is {}",
                line.len(),
                PLACEABLE_PWK_MAX_LINE_BYTES_V1
            ),
        ));
    }
    output.push_str(line);
    output.push_str("\r\n");
    Ok(())
}

fn require_field_count(
    fields: &[&str],
    expected: usize,
    line: usize,
    label: &str,
) -> Result<(), PlaceableWalkmeshErrorV1> {
    if fields.len() != expected {
        return Err(error(
            "PLACEABLE-PWK-FIELD-INVALID",
            format!("pwk.lines[{line}]"),
            format!("{label} requires {} value(s)", expected - 1),
        ));
    }
    Ok(())
}

fn set_once<T>(
    target: &mut Option<T>,
    value: T,
    line: usize,
    label: &str,
) -> Result<(), PlaceableWalkmeshErrorV1> {
    if target.replace(value).is_some() {
        return Err(duplicate_field(line, label));
    }
    Ok(())
}

fn duplicate_field(line: usize, label: &str) -> PlaceableWalkmeshErrorV1 {
    error(
        "PLACEABLE-PWK-FIELD-DUPLICATE",
        format!("pwk.lines[{line}]"),
        format!("{label} may appear only once per node"),
    )
}

fn parse_count(
    value: &str,
    maximum: usize,
    line: usize,
    label: &str,
) -> Result<usize, PlaceableWalkmeshErrorV1> {
    let count = value.parse::<usize>().map_err(|_| {
        error(
            "PLACEABLE-PWK-COUNT-INVALID",
            format!("pwk.lines[{line}]"),
            format!("{label} count must be an unsigned integer"),
        )
    })?;
    if count == 0 || count > maximum {
        return Err(error(
            "PLACEABLE-PWK-COUNT-INVALID",
            format!("pwk.lines[{line}]"),
            format!("{label} count must be in 1..={maximum}"),
        ));
    }
    Ok(count)
}

fn parse_data_triplet(
    lines: &[&str],
    cursor: &mut usize,
    label: &str,
) -> Result<[f32; 3], PlaceableWalkmeshErrorV1> {
    let line_index = *cursor;
    let line = lines.get(*cursor).ok_or_else(|| {
        error(
            "PLACEABLE-PWK-DATA-TRUNCATED",
            format!("pwk.{label}"),
            "declared data rows exceed available lines",
        )
    })?;
    let fields = line.split_whitespace().collect::<Vec<_>>();
    require_field_count(&fields, 3, line_index, label)?;
    let value = parse_f32_triplet(&fields, line_index, label)?;
    *cursor += 1;
    Ok(value)
}

fn parse_face_line(
    lines: &[&str],
    cursor: &mut usize,
    face_index: usize,
) -> Result<PlaceableWalkmeshFaceV1, PlaceableWalkmeshErrorV1> {
    let line_index = *cursor;
    let line = lines.get(*cursor).ok_or_else(|| {
        error(
            "PLACEABLE-PWK-DATA-TRUNCATED",
            format!("pwk.faces[{face_index}]"),
            "declared face rows exceed available lines",
        )
    })?;
    let fields = line.split_whitespace().collect::<Vec<_>>();
    require_field_count(&fields, 8, line_index, "face")?;
    let mut values = [0_i32; 8];
    for (field_index, field) in fields.iter().enumerate() {
        values[field_index] = field.parse::<i32>().map_err(|_| {
            error(
                "PLACEABLE-PWK-FACE-INVALID",
                format!("pwk.lines[{line_index}].fields[{field_index}]"),
                "face fields must be signed integers",
            )
        })?;
    }
    if values[..3].iter().any(|value| *value < 0) {
        return Err(error(
            "PLACEABLE-PWK-FACE-INDEX-INVALID",
            format!("pwk.faces[{face_index}].vertexIndices"),
            "face vertex indices must be non-negative",
        ));
    }
    *cursor += 1;
    Ok(PlaceableWalkmeshFaceV1 {
        vertex_indices: [values[0] as u32, values[1] as u32, values[2] as u32],
        smoothing_group: values[3],
        adjacent_faces: [values[4], values[5], values[6]],
        surface_id: values[7],
    })
}

fn parse_f32_triplet(
    fields: &[&str],
    line: usize,
    label: &str,
) -> Result<[f32; 3], PlaceableWalkmeshErrorV1> {
    Ok([
        parse_f32(fields[0], line, label, 0)?,
        parse_f32(fields[1], line, label, 1)?,
        parse_f32(fields[2], line, label, 2)?,
    ])
}

fn parse_f32_quad(
    fields: &[&str],
    line: usize,
    label: &str,
) -> Result<[f32; 4], PlaceableWalkmeshErrorV1> {
    Ok([
        parse_f32(fields[0], line, label, 0)?,
        parse_f32(fields[1], line, label, 1)?,
        parse_f32(fields[2], line, label, 2)?,
        parse_f32(fields[3], line, label, 3)?,
    ])
}

fn parse_f32(
    field: &str,
    line: usize,
    label: &str,
    field_index: usize,
) -> Result<f32, PlaceableWalkmeshErrorV1> {
    let value = field.parse::<f32>().map_err(|_| {
        error(
            "PLACEABLE-PWK-FLOAT-INVALID",
            format!("pwk.lines[{line}].{label}[{field_index}]"),
            "numeric field must be a finite float",
        )
    })?;
    if !value.is_finite() {
        return Err(error(
            "PLACEABLE-PWK-FLOAT-INVALID",
            format!("pwk.lines[{line}].{label}[{field_index}]"),
            "numeric field must be a finite float",
        ));
    }
    Ok(value)
}

fn validate_face_indices(
    faces: &[PlaceableWalkmeshFaceV1],
    vertex_count: usize,
    node_name: &str,
) -> Result<(), PlaceableWalkmeshErrorV1> {
    for (face_index, face) in faces.iter().enumerate() {
        if face
            .vertex_indices
            .iter()
            .any(|index| *index as usize >= vertex_count)
        {
            return Err(error(
                "PLACEABLE-PWK-FACE-INDEX-INVALID",
                format!("pwk.node[{node_name}].faces[{face_index}].vertexIndices"),
                format!("face references a vertex outside 0..{vertex_count}"),
            ));
        }
    }
    Ok(())
}

fn world_xy_bounds(
    model: &AuroraModelIrV1,
) -> Result<([f32; 2], [f32; 2]), PlaceableWalkmeshErrorV1> {
    if model.schema_version != 1 || model.nodes.is_empty() || model.segments.is_empty() {
        return Err(error(
            "PLACEABLE-PWK-MODEL-INVALID",
            "model",
            "schemaVersion 1 with non-empty nodes and segments is required",
        ));
    }
    let mut id_to_index = HashMap::with_capacity(model.nodes.len());
    for (index, node) in model.nodes.iter().enumerate() {
        if id_to_index.insert(node.id, index).is_some() {
            return Err(error(
                "PLACEABLE-PWK-HIERARCHY-INVALID",
                format!("model.nodes[{index}].id"),
                "node ids must be unique",
            ));
        }
        if node
            .bind_local_matrix
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(error(
                "PLACEABLE-PWK-TRANSFORM-INVALID",
                format!("model.nodes[{index}].bindLocalMatrix"),
                "bind transform must contain only finite values",
            ));
        }
    }
    let mut worlds = vec![None; model.nodes.len()];
    let mut visiting = vec![false; model.nodes.len()];
    for index in 0..model.nodes.len() {
        resolve_world(index, model, &id_to_index, &mut worlds, &mut visiting)?;
    }

    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for (segment_index, segment) in model.segments.iter().enumerate() {
        if segment.deformation != AuroraSegmentDeformationV1::Rigid {
            return Err(error(
                "PLACEABLE-PWK-MODEL-INVALID",
                format!("model.segments[{segment_index}].deformation"),
                "static placeable collision accepts only rigid segments",
            ));
        }
        let parent_index = id_to_index
            .get(&segment.parent_node_id)
            .copied()
            .ok_or_else(|| {
                error(
                    "PLACEABLE-PWK-HIERARCHY-INVALID",
                    format!("model.segments[{segment_index}].parentNodeId"),
                    "segment parent node does not exist",
                )
            })?;
        let world = worlds[parent_index].expect("world matrix resolved");
        for (vertex_index, position) in segment.positions.iter().copied().enumerate() {
            let transformed = transform_point(world, position).ok_or_else(|| {
                error(
                    "PLACEABLE-PWK-TRANSFORM-INVALID",
                    format!("model.segments[{segment_index}].positions[{vertex_index}]"),
                    "world-space position must be finite affine geometry",
                )
            })?;
            for axis in 0..2 {
                min[axis] = min[axis].min(transformed[axis]);
                max[axis] = max[axis].max(transformed[axis]);
            }
        }
    }
    if min
        .into_iter()
        .chain(max)
        .any(|coordinate| !coordinate.is_finite())
    {
        return Err(error(
            "PLACEABLE-PWK-FOOTPRINT-DEGENERATE",
            "model.segments",
            "collision footprint has no finite world-space geometry",
        ));
    }
    Ok((min, max))
}

fn resolve_world(
    index: usize,
    model: &AuroraModelIrV1,
    id_to_index: &HashMap<u32, usize>,
    worlds: &mut [Option<[f32; 16]>],
    visiting: &mut [bool],
) -> Result<[f32; 16], PlaceableWalkmeshErrorV1> {
    if let Some(world) = worlds[index] {
        return Ok(world);
    }
    if visiting[index] {
        return Err(error(
            "PLACEABLE-PWK-HIERARCHY-INVALID",
            format!("model.nodes[{index}].parentId"),
            "node hierarchy contains a cycle",
        ));
    }
    visiting[index] = true;
    let node = &model.nodes[index];
    let world = match node.parent_id {
        None => node.bind_local_matrix,
        Some(parent_id) => {
            let parent_index = id_to_index.get(&parent_id).copied().ok_or_else(|| {
                error(
                    "PLACEABLE-PWK-HIERARCHY-INVALID",
                    format!("model.nodes[{index}].parentId"),
                    "parent node does not exist",
                )
            })?;
            mul_mat4(
                resolve_world(parent_index, model, id_to_index, worlds, visiting)?,
                node.bind_local_matrix,
            )
        }
    };
    visiting[index] = false;
    worlds[index] = Some(world);
    Ok(world)
}

fn mul_mat4(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|lane| left[lane * 4 + row] * right[column * 4 + lane])
                .sum();
        }
    }
    output
}

fn transform_point(matrix: [f32; 16], point: [f32; 3]) -> Option<[f32; 3]> {
    let output = [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ];
    output
        .iter()
        .all(|value| value.is_finite())
        .then_some(output)
}

fn validate_resref(value: &str) -> Result<(), PlaceableWalkmeshErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "PLACEABLE-PWK-RESREF-INVALID",
            "modelResref",
            "resref must contain 1..=16 lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
