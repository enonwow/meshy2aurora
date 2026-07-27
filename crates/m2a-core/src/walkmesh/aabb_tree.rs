use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::ascii_common::{WalkmeshFaceV1, validate_faces, validate_vertices};
use super::{TileWalkmeshErrorV1, error};

const EPSILON: f32 = 1.0e-5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AabbEntryV1 {
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub left: Option<u32>,
    pub right: Option<u32>,
    pub leaf_face: Option<u32>,
    pub plane: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AabbTreeV1 {
    pub root_index: u32,
    pub entries: Vec<AabbEntryV1>,
}

#[derive(Clone, Copy)]
struct FaceBounds {
    face: u32,
    min: [f32; 3],
    max: [f32; 3],
    centroid: [f32; 3],
}

pub fn build_aabb_tree_v1(
    vertices: &[[f32; 3]],
    faces: &[WalkmeshFaceV1],
) -> Result<AabbTreeV1, TileWalkmeshErrorV1> {
    validate_vertices(vertices, "vertices")?;
    validate_faces(vertices, faces, "faces").map_err(|source| {
        let code = if source.code == "TILE-WALKMESH-DEGENERATE-FACE" {
            "TILE-AABB-DEGENERATE-FACE"
        } else {
            source.code.as_str()
        };
        error(code, source.path, source.message)
    })?;
    let mut bounded = Vec::with_capacity(faces.len());
    for (face_index, face) in faces.iter().enumerate() {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for vertex_index in face.vertex_indices {
            let vertex = vertices[vertex_index as usize];
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex[axis]);
                max[axis] = max[axis].max(vertex[axis]);
            }
        }
        bounded.push(FaceBounds {
            face: face_index as u32,
            min,
            max,
            centroid: [
                (min[0] + max[0]) * 0.5,
                (min[1] + max[1]) * 0.5,
                (min[2] + max[2]) * 0.5,
            ],
        });
    }
    let expected_entries = faces
        .len()
        .checked_mul(2)
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(|| {
            error(
                "TILE-AABB-LAYOUT-OVERFLOW",
                "faces",
                "AABB entry count overflow",
            )
        })?;
    let mut entries = Vec::new();
    entries.try_reserve_exact(expected_entries).map_err(|_| {
        error(
            "TILE-AABB-ALLOCATION",
            "entries",
            "AABB entry allocation failed after validated size preflight",
        )
    })?;
    let root_index = build_node(&mut bounded, &mut entries)?;
    let tree = AabbTreeV1 {
        root_index,
        entries,
    };
    validate_aabb_tree_v1(&tree, vertices, faces)?;
    Ok(tree)
}

fn build_node(
    faces: &mut [FaceBounds],
    entries: &mut Vec<AabbEntryV1>,
) -> Result<u32, TileWalkmeshErrorV1> {
    let (min, max) = union_bounds(faces);
    let index = u32::try_from(entries.len()).map_err(|_| {
        error(
            "TILE-AABB-LAYOUT-OVERFLOW",
            "entries",
            "AABB entry index exceeds u32",
        )
    })?;
    entries.push(AabbEntryV1 {
        bounds_min: min,
        bounds_max: max,
        left: None,
        right: None,
        leaf_face: None,
        plane: 0,
    });
    if faces.len() == 1 {
        entries[index as usize].leaf_face = Some(faces[0].face);
        return Ok(index);
    }
    let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let axis = if extent[1] > extent[0] && extent[1] >= extent[2] {
        1
    } else if extent[2] > extent[0] && extent[2] > extent[1] {
        2
    } else {
        0
    };
    faces.sort_by(|left, right| {
        left.centroid[axis]
            .total_cmp(&right.centroid[axis])
            .then_with(|| left.face.cmp(&right.face))
    });
    let middle = faces.len() / 2;
    let (left_faces, right_faces) = faces.split_at_mut(middle);
    let left = build_node(left_faces, entries)?;
    let right = build_node(right_faces, entries)?;
    entries[index as usize].left = Some(left);
    entries[index as usize].right = Some(right);
    entries[index as usize].plane = axis as u32;
    Ok(index)
}

fn union_bounds(faces: &[FaceBounds]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for face in faces {
        for axis in 0..3 {
            min[axis] = min[axis].min(face.min[axis]);
            max[axis] = max[axis].max(face.max[axis]);
        }
    }
    (min, max)
}

pub fn validate_aabb_tree_v1(
    tree: &AabbTreeV1,
    vertices: &[[f32; 3]],
    faces: &[WalkmeshFaceV1],
) -> Result<(), TileWalkmeshErrorV1> {
    validate_vertices(vertices, "vertices")?;
    validate_faces(vertices, faces, "faces")?;
    if tree.entries.is_empty() || tree.root_index as usize >= tree.entries.len() {
        return Err(error(
            "TILE-AABB-ROOT-OOB",
            "aabbTree.rootIndex",
            "AABB root must reference an existing entry",
        ));
    }
    let expected_entries = faces
        .len()
        .checked_mul(2)
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(|| {
            error(
                "TILE-AABB-LAYOUT-OVERFLOW",
                "faces",
                "AABB entry count overflow",
            )
        })?;
    if tree.entries.len() != expected_entries {
        return Err(error(
            "TILE-AABB-ENTRY-COUNT",
            "aabbTree.entries",
            format!(
                "binary V1 tree requires exactly 2*faces-1 entries; expected {expected_entries}, got {}",
                tree.entries.len()
            ),
        ));
    }
    let mut visited = HashSet::new();
    let mut active = HashSet::new();
    let mut covered = vec![0_u8; faces.len()];
    visit(
        tree.root_index,
        tree,
        None,
        &mut visited,
        &mut active,
        &mut covered,
    )?;
    if visited.len() != tree.entries.len() {
        return Err(error(
            "TILE-AABB-UNREACHABLE",
            "aabbTree.entries",
            "every AABB entry must be reachable from the root",
        ));
    }
    if covered.iter().any(|count| *count != 1) {
        return Err(error(
            "TILE-AABB-FACE-COVERAGE",
            "aabbTree.entries",
            "AABB leaves must cover every face exactly once",
        ));
    }
    let (geometry_min, geometry_max) = geometry_bounds(vertices);
    let root = &tree.entries[tree.root_index as usize];
    if !contains(root.bounds_min, root.bounds_max, geometry_min, geometry_max) {
        return Err(error(
            "TILE-AABB-ROOT-BOUNDS",
            "aabbTree.root",
            "root bounds do not contain all AABB geometry",
        ));
    }
    Ok(())
}

fn visit(
    index: u32,
    tree: &AabbTreeV1,
    parent_bounds: Option<([f32; 3], [f32; 3])>,
    visited: &mut HashSet<u32>,
    active: &mut HashSet<u32>,
    covered: &mut [u8],
) -> Result<(), TileWalkmeshErrorV1> {
    if !active.insert(index) {
        return Err(error(
            "TILE-AABB-CYCLE",
            format!("aabbTree.entries[{index}]"),
            "AABB child pointers form a cycle",
        ));
    }
    if !visited.insert(index) {
        active.remove(&index);
        return Err(error(
            "TILE-AABB-ALIAS",
            format!("aabbTree.entries[{index}]"),
            "AABB entry is referenced by more than one parent",
        ));
    }
    let entry = tree.entries.get(index as usize).ok_or_else(|| {
        error(
            "TILE-AABB-CHILD-OOB",
            format!("aabbTree.entries[{index}]"),
            "AABB child references an entry outside the tree",
        )
    })?;
    validate_bounds(entry, index)?;
    if let Some((parent_min, parent_max)) = parent_bounds
        && !contains(parent_min, parent_max, entry.bounds_min, entry.bounds_max)
    {
        return Err(error(
            "TILE-AABB-CHILD-BOUNDS",
            format!("aabbTree.entries[{index}]"),
            "child bounds escape parent bounds",
        ));
    }
    match (entry.left, entry.right, entry.leaf_face) {
        (Some(left), Some(right), None) => {
            if entry.plane > 2 {
                return Err(error(
                    "TILE-AABB-PLANE-INVALID",
                    format!("aabbTree.entries[{index}].plane"),
                    "internal plane axis must be 0, 1 or 2",
                ));
            }
            visit(
                left,
                tree,
                Some((entry.bounds_min, entry.bounds_max)),
                visited,
                active,
                covered,
            )?;
            visit(
                right,
                tree,
                Some((entry.bounds_min, entry.bounds_max)),
                visited,
                active,
                covered,
            )?;
        }
        (None, None, Some(face)) => {
            let slot = covered.get_mut(face as usize).ok_or_else(|| {
                error(
                    "TILE-AABB-LEAF-FACE-OOB",
                    format!("aabbTree.entries[{index}].leafFace"),
                    "leaf face index exceeds walkmesh face count",
                )
            })?;
            *slot = slot.checked_add(1).ok_or_else(|| {
                error(
                    "TILE-AABB-FACE-COVERAGE",
                    format!("aabbTree.entries[{index}].leafFace"),
                    "leaf coverage counter overflow",
                )
            })?;
        }
        _ => {
            return Err(error(
                "TILE-AABB-NODE-SHAPE",
                format!("aabbTree.entries[{index}]"),
                "internal entries require two children; leaves require one face and no children",
            ));
        }
    }
    active.remove(&index);
    Ok(())
}

fn validate_bounds(entry: &AabbEntryV1, index: u32) -> Result<(), TileWalkmeshErrorV1> {
    for axis in 0..3 {
        if !entry.bounds_min[axis].is_finite()
            || !entry.bounds_max[axis].is_finite()
            || entry.bounds_min[axis] > entry.bounds_max[axis]
        {
            return Err(error(
                "TILE-AABB-BOUNDS-INVALID",
                format!("aabbTree.entries[{index}].bounds"),
                "AABB bounds must be finite and ordered",
            ));
        }
    }
    Ok(())
}

fn contains(
    outer_min: [f32; 3],
    outer_max: [f32; 3],
    inner_min: [f32; 3],
    inner_max: [f32; 3],
) -> bool {
    (0..3).all(|axis| {
        inner_min[axis] + EPSILON >= outer_min[axis] && inner_max[axis] - EPSILON <= outer_max[axis]
    })
}

fn geometry_bounds(vertices: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(vertex[axis]);
            max[axis] = max[axis].max(vertex[axis]);
        }
    }
    (min, max)
}
