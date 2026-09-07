//! Surface-region guides read in memory from an inspected reference.
//! Only computed carrier labels are returned; reference geometry is never emitted.
use crate::{
    mdl::{InspectionReport, NodeReport},
    reference_supermodel_generic::ReferenceSupermodelGenericErrorV2,
    reference_supermodel_motion::ReferenceSupermodelMotionContractV2,
};
#[derive(Clone)]
struct Triangle {
    part: u32,
    p: [[f32; 3]; 3],
    min: [f32; 3],
    max: [f32; 3],
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn edge_distance(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let v = sub(b, a);
    let t = (dot(sub(p, a), v) / dot(v, v).max(1e-20)).clamp(0., 1.);
    let d = sub(p, std::array::from_fn(|i| a[i] + v[i] * t));
    dot(d, d)
}
fn triangle_distance(p: [f32; 3], v: [[f32; 3]; 3]) -> f32 {
    let u = sub(v[1], v[0]);
    let w = sub(v[2], v[0]);
    let q = sub(p, v[0]);
    let uu = dot(u, u);
    let uw = dot(u, w);
    let ww = dot(w, w);
    let qu = dot(q, u);
    let qw = dot(q, w);
    let det = uu * ww - uw * uw;
    if det > 1e-16 {
        let a = (qu * ww - qw * uw) / det;
        let b = (qw * uu - qu * uw) / det;
        if a >= 0. && b >= 0. && a + b <= 1. {
            let d = std::array::from_fn(|i| q[i] - a * u[i] - b * w[i]);
            return dot(d, d);
        }
    }
    edge_distance(p, v[0], v[1])
        .min(edge_distance(p, v[1], v[2]))
        .min(edge_distance(p, v[2], v[0]))
}
fn world_point(m: [f32; 16], p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| m[i] * p[0] + m[i + 4] * p[1] + m[i + 8] * p[2] + m[i + 12])
}
fn invalid(message: &str) -> ReferenceSupermodelGenericErrorV2 {
    ReferenceSupermodelGenericErrorV2 {
        schema_version: 2,
        code: "M2A-SURFACE-GUIDE-INVALID".into(),
        path: "reference.meshes".into(),
        message: message.into(),
    }
}
fn collect(
    ns: &[NodeReport],
    ws: &[[f32; 16]],
    allowed: &[u32],
    contract: &ReferenceSupermodelMotionContractV2,
    out: &mut Vec<Triangle>,
) -> Result<(), ReferenceSupermodelGenericErrorV2> {
    for n in ns {
        if n.number as usize >= contract.nodes.len() {
            return Err(invalid("Reference node index exceeds contract"));
        }
        if let Some(m) = n.mesh.as_ref().filter(|m| m.render != 0) {
            let mut owner = n.number;
            while !allowed.contains(&owner) {
                if let Some(parent) = contract.nodes[owner as usize].parent_part_number {
                    owner = parent;
                } else {
                    break;
                }
            }
            if allowed.contains(&owner) {
                for face in &m.faces {
                    if face
                        .vertex_indices
                        .iter()
                        .any(|i| *i as usize >= m.vertices.len())
                    {
                        return Err(invalid(
                            "Reference triangle index exceeds its vertex buffer",
                        ));
                    }
                    let p = face.vertex_indices.map(|i| {
                        let v = &m.vertices[i as usize];
                        world_point(ws[n.number as usize], [v.x, v.y, v.z])
                    });
                    if p.iter().flatten().any(|v| !v.is_finite()) {
                        return Err(invalid("Reference guide contains non-finite geometry"));
                    }
                    out.push(Triangle {
                        part: owner,
                        min: std::array::from_fn(|a| {
                            p.iter().map(|p| p[a]).fold(f32::INFINITY, f32::min)
                        }),
                        max: std::array::from_fn(|a| {
                            p.iter().map(|p| p[a]).fold(f32::NEG_INFINITY, f32::max)
                        }),
                        p,
                    });
                }
            }
        }
        collect(&n.children, ws, allowed, contract, out)?;
    }
    Ok(())
}
pub fn reference_surface_labels_v1(
    reference: &InspectionReport,
    contract: &ReferenceSupermodelMotionContractV2,
    points: &[[f32; 3]],
    allowed: &[u32],
) -> Result<Vec<u32>, ReferenceSupermodelGenericErrorV2> {
    if allowed.is_empty()
        || allowed.iter().any(|i| *i as usize >= contract.nodes.len())
        || points.iter().flatten().any(|v| !v.is_finite())
    {
        return Err(invalid(
            "Finite source positions and nonempty valid carrier indices are required",
        ));
    }
    for (i, n) in contract.nodes.iter().enumerate() {
        if n.part_number as usize != i
            || n.parent_part_number.is_some_and(|p| p as usize >= i)
            || n.carrier_bind_local_matrix.iter().any(|v| !v.is_finite())
        {
            return Err(invalid(
                "Reference contract requires ordered acyclic finite bind transforms",
            ));
        }
    }
    let mut ws: Vec<[f32; 16]> = Vec::new();
    for n in &contract.nodes {
        let local = n.carrier_bind_local_matrix;
        let m = if let Some(parent) = n.parent_part_number {
            let a = ws[parent as usize];
            std::array::from_fn(|i| {
                (0..4)
                    .map(|k| a[i % 4 + k * 4] * local[k + i / 4 * 4])
                    .sum()
            })
        } else {
            local
        };
        ws.push(m);
    }
    let mut triangles = Vec::new();
    collect(
        &reference.node_tree.roots,
        &ws,
        allowed,
        contract,
        &mut triangles,
    )?;
    if triangles.is_empty() {
        return Err(ReferenceSupermodelGenericErrorV2 {
            schema_version: 2,
            code: "M2A-SURFACE-GUIDE-EMPTY".into(),
            path: "reference.meshes".into(),
            message: "No visible reference surface belongs to an allowed carrier".into(),
        });
    }
    Ok(points
        .iter()
        .map(|p| {
            let mut best = (f32::INFINITY, allowed[0]);
            for t in &triangles {
                let box_distance = (0..3)
                    .map(|a| (p[a] - p[a].clamp(t.min[a], t.max[a])).powi(2))
                    .sum::<f32>();
                if box_distance > best.0 {
                    continue;
                }
                let d = triangle_distance(*p, t.p);
                if d < best.0 || (d == best.0 && t.part < best.1) {
                    best = (d, t.part);
                }
            }
            best.1
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projected_triangle_distance_handles_inside_edge_and_degenerate() {
        let t = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        assert!((triangle_distance([0.2, 0.2, 2.], t) - 4.).abs() < 1e-6);
        assert!((triangle_distance([1., 1., 0.], t) - 0.5).abs() < 1e-6);
        assert_eq!(triangle_distance([1., 0., 0.], [[0.; 3]; 3]), 1.);
    }
}
