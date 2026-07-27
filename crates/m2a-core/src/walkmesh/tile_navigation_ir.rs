use std::collections::{HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::aabb_tree::{AabbTreeV1, build_aabb_tree_v1, validate_aabb_tree_v1};
use super::ascii_common::{
    WalkmeshFaceV1, compute_face_adjacency, validate_faces, validate_resref, validate_vertices,
};
use super::{TileWalkmeshErrorV1, error};

pub const TILE_FOOTPRINT_SIZE_V1: f32 = 10.0;
pub const TILE_FOOTPRINT_HALF_EXTENT_V1: f32 = 5.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TileSurfaceV1 {
    Dirt,
    Grass,
    Stone,
    Wood,
}

impl TileSurfaceV1 {
    pub const fn id(self) -> i32 {
        match self {
            Self::Dirt => 1,
            Self::Grass => 3,
            Self::Stone => 4,
            Self::Wood => 5,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Dirt => "Dirt",
            Self::Grass => "Grass",
            Self::Stone => "Stone",
            Self::Wood => "Wood",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TileNavigationIrV1 {
    pub schema_version: u32,
    pub model_resref: String,
    pub node_name: String,
    pub surface: TileSurfaceV1,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<WalkmeshFaceV1>,
    pub aabb_tree: AabbTreeV1,
}

pub fn flat_tile_navigation_v1(
    model_resref: &str,
    surface: TileSurfaceV1,
) -> Result<TileNavigationIrV1, TileWalkmeshErrorV1> {
    validate_resref(model_resref, "modelResref")?;
    let coordinates = [-5.0_f32, 0.0, 5.0];
    let mut vertices = Vec::with_capacity(9);
    for y in coordinates {
        for x in coordinates {
            vertices.push([x, y, 0.0]);
        }
    }
    let mut triangles = Vec::with_capacity(8);
    for y in 0..2_u32 {
        for x in 0..2_u32 {
            let bottom_left = y * 3 + x;
            let bottom_right = bottom_left + 1;
            let top_left = bottom_left + 3;
            let top_right = top_left + 1;
            triangles.push([bottom_left, bottom_right, top_right]);
            triangles.push([bottom_left, top_right, top_left]);
        }
    }
    let adjacency = compute_face_adjacency(&triangles)?;
    let faces = triangles
        .into_iter()
        .zip(adjacency)
        .map(|(vertex_indices, adjacent_faces)| WalkmeshFaceV1 {
            vertex_indices,
            smoothing_group: 1,
            adjacent_faces,
            surface_id: surface.id(),
        })
        .collect::<Vec<_>>();
    let aabb_tree = build_aabb_tree_v1(&vertices, &faces)?;
    let navigation = TileNavigationIrV1 {
        schema_version: 1,
        model_resref: model_resref.to_owned(),
        node_name: format!("{model_resref}_wg"),
        surface,
        position: [0.0, 0.0, 0.0],
        orientation: [1.0, 0.0, 0.0, 0.0],
        vertices,
        faces,
        aabb_tree,
    };
    validate_tile_navigation_v1(&navigation)?;
    Ok(navigation)
}

pub fn validate_tile_navigation_v1(
    navigation: &TileNavigationIrV1,
) -> Result<(), TileWalkmeshErrorV1> {
    if navigation.schema_version != 1 {
        return Err(error(
            "TILE-NAVIGATION-SCHEMA",
            "navigation.schemaVersion",
            "TileNavigationIrV1 must use schema version 1",
        ));
    }
    validate_resref(&navigation.model_resref, "navigation.modelResref")?;
    if navigation.node_name.is_empty()
        || navigation.node_name.len() > 31
        || !navigation.node_name.is_ascii()
    {
        return Err(error(
            "TILE-NAVIGATION-NODE-NAME",
            "navigation.nodeName",
            "node name must contain 1..31 ASCII bytes",
        ));
    }
    if !navigation.position.iter().all(|value| value.is_finite())
        || !navigation.orientation.iter().all(|value| value.is_finite())
    {
        return Err(error(
            "TILE-NAVIGATION-TRANSFORM",
            "navigation.transform",
            "position and orientation must be finite",
        ));
    }
    validate_vertices(&navigation.vertices, "navigation.vertices")?;
    validate_faces(&navigation.vertices, &navigation.faces, "navigation.faces")?;
    if navigation.faces.iter().all(|face| face.surface_id == 7) {
        return Err(error(
            "TILE-WOK-NONWALK-FLOOR",
            "navigation.faces.surfaceId",
            "placeable Nonwalk surface 7 cannot be the only V1 tile floor",
        ));
    }
    if navigation
        .faces
        .iter()
        .any(|face| face.surface_id != navigation.surface.id())
    {
        return Err(error(
            "TILE-WOK-SURFACE-MISMATCH",
            "navigation.faces.surfaceId",
            "every V1 floor face must use the explicitly selected walkable surface",
        ));
    }
    validate_exact_footprint(&navigation.vertices)?;
    validate_connected_walkable_component(&navigation.faces)?;
    validate_aabb_tree_v1(
        &navigation.aabb_tree,
        &navigation.vertices,
        &navigation.faces,
    )
}

fn validate_exact_footprint(vertices: &[[f32; 3]]) -> Result<(), TileWalkmeshErrorV1> {
    let min_x = vertices
        .iter()
        .map(|vertex| vertex[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = vertices
        .iter()
        .map(|vertex| vertex[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(f32::NEG_INFINITY, f32::max);
    if [min_x, min_y] != [-5.0, -5.0] || [max_x, max_y] != [5.0, 5.0] {
        return Err(error(
            "TILE-WOK-FOOTPRINT",
            "navigation.vertices",
            "V1 tile navigation bounds must be exactly -5..5 on X and Y",
        ));
    }
    for (axis, boundary) in [(0, -5.0), (0, 5.0), (1, -5.0), (1, 5.0)] {
        if !vertices
            .iter()
            .any(|vertex| vertex[axis].to_bits() == f32::to_bits(boundary))
        {
            return Err(error(
                "TILE-WOK-SEAM-VERTEX-MISSING",
                "navigation.vertices",
                "every V1 footprint edge must contain an exact boundary vertex",
            ));
        }
    }
    Ok(())
}

fn validate_connected_walkable_component(
    faces: &[WalkmeshFaceV1],
) -> Result<(), TileWalkmeshErrorV1> {
    let mut queue = VecDeque::from([0_usize]);
    let mut visited = HashSet::new();
    while let Some(face) = queue.pop_front() {
        if !visited.insert(face) {
            continue;
        }
        for adjacent in faces[face].adjacent_faces {
            if adjacent >= 0 {
                queue.push_back(adjacent as usize);
            }
        }
    }
    if visited.len() != faces.len() {
        return Err(error(
            "TILE-WOK-WALKABLE-DISCONNECTED",
            "navigation.faces",
            "V1 floor faces must form one connected walkable component",
        ));
    }
    Ok(())
}
