use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{TileWalkmeshErrorV1, error};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalkmeshFaceV1 {
    pub vertex_indices: [u32; 3],
    pub smoothing_group: i32,
    pub adjacent_faces: [i32; 3],
    pub surface_id: i32,
}

pub(crate) fn validate_resref(value: &str, path: &str) -> Result<(), TileWalkmeshErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "TILE-RESREF-INVALID",
            path,
            "resref must contain 1..16 canonical lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

pub(crate) fn validate_vertices(
    vertices: &[[f32; 3]],
    path: &str,
) -> Result<(), TileWalkmeshErrorV1> {
    if vertices.is_empty() || vertices.len() > u16::MAX as usize {
        return Err(error(
            "TILE-WALKMESH-VERTEX-COUNT",
            path,
            "walkmesh must contain 1..65535 vertices",
        ));
    }
    for (index, vertex) in vertices.iter().enumerate() {
        if !vertex.iter().all(|value| value.is_finite()) {
            return Err(error(
                "TILE-WALKMESH-NONFINITE",
                format!("{path}[{index}]"),
                "walkmesh vertices must be finite",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_faces(
    vertices: &[[f32; 3]],
    faces: &[WalkmeshFaceV1],
    path: &str,
) -> Result<(), TileWalkmeshErrorV1> {
    if faces.is_empty() || faces.len() > u16::MAX as usize {
        return Err(error(
            "TILE-WALKMESH-FACE-COUNT",
            path,
            "walkmesh must contain 1..65535 faces",
        ));
    }
    for (face_index, face) in faces.iter().enumerate() {
        if face
            .vertex_indices
            .iter()
            .any(|index| *index as usize >= vertices.len())
        {
            return Err(error(
                "TILE-WALKMESH-INDEX-OOB",
                format!("{path}[{face_index}].vertexIndices"),
                "face vertex index exceeds vertex count",
            ));
        }
        let [ia, ib, ic] = face.vertex_indices;
        if ia == ib || ib == ic || ia == ic {
            return Err(error(
                "TILE-WALKMESH-DEGENERATE-FACE",
                format!("{path}[{face_index}]"),
                "face repeats a vertex index",
            ));
        }
        let a = vertices[ia as usize];
        let b = vertices[ib as usize];
        let c = vertices[ic as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let squared_area = cross.iter().map(|value| value * value).sum::<f32>();
        if !squared_area.is_finite() || squared_area <= 1.0e-12 {
            return Err(error(
                "TILE-WALKMESH-DEGENERATE-FACE",
                format!("{path}[{face_index}]"),
                "face has zero or non-finite area",
            ));
        }
        for (edge, adjacent) in face.adjacent_faces.iter().enumerate() {
            if *adjacent < -1 || (*adjacent >= 0 && *adjacent as usize >= faces.len()) {
                return Err(error(
                    "TILE-WALKMESH-ADJACENCY-OOB",
                    format!("{path}[{face_index}].adjacentFaces[{edge}]"),
                    "adjacent face must be -1 or an existing face index",
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn compute_face_adjacency(
    indices: &[[u32; 3]],
) -> Result<Vec<[i32; 3]>, TileWalkmeshErrorV1> {
    let mut adjacency = vec![[-1_i32; 3]; indices.len()];
    let mut edges: HashMap<(u32, u32), Vec<(usize, usize)>> = HashMap::new();
    for (face_index, triangle) in indices.iter().enumerate() {
        for (edge_index, (a, b)) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ]
        .into_iter()
        .enumerate()
        {
            let key = if a < b { (a, b) } else { (b, a) };
            let uses = edges.entry(key).or_default();
            uses.push((face_index, edge_index));
            if uses.len() > 2 {
                return Err(error(
                    "TILE-WALKMESH-NONMANIFOLD",
                    "faces",
                    "an undirected edge is shared by more than two faces",
                ));
            }
        }
    }
    for uses in edges.values() {
        if let [(left_face, left_edge), (right_face, right_edge)] = uses.as_slice() {
            adjacency[*left_face][*left_edge] = *right_face as i32;
            adjacency[*right_face][*right_edge] = *left_face as i32;
        }
    }
    Ok(adjacency)
}

pub(crate) fn format_f32(value: f32) -> String {
    format_ascii_f32(value, false)
}

pub(crate) fn format_placeable_f32(value: f32) -> String {
    format_ascii_f32(value, true)
}

fn format_ascii_f32(value: f32, force_decimal_for_integral: bool) -> String {
    if value == 0.0 {
        return if force_decimal_for_integral {
            "0.0".to_owned()
        } else {
            "0".to_owned()
        };
    }
    if force_decimal_for_integral {
        let mut output = value.to_string();
        if !output.contains(['.', 'e', 'E']) {
            output.push_str(".0");
        }
        return output;
    }
    let mut text = format!("{value:.6}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::compute_face_adjacency;

    #[test]
    fn adjacency_rejects_an_edge_used_by_three_faces() {
        let error = compute_face_adjacency(&[[0, 1, 2], [1, 0, 3], [0, 1, 4]])
            .expect_err("three uses must be non-manifold");
        assert_eq!(error.code, "TILE-WALKMESH-NONMANIFOLD");
    }

    #[test]
    fn adjacency_links_exactly_two_uses_and_leaves_boundaries_open() {
        let value = compute_face_adjacency(&[[0, 1, 2], [1, 0, 3]]).unwrap();
        assert_eq!(value[0], [1, -1, -1]);
        assert_eq!(value[1], [0, -1, -1]);
    }
}
