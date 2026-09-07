use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
type P = [f32; 3];
fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: P, b: P) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(a: P) -> f32 {
    dot(a, a).sqrt()
}
fn tp(m: [f32; 16], p: P) -> P {
    std::array::from_fn(|i| m[i] * p[0] + m[i + 4] * p[1] + m[i + 8] * p[2] + m[i + 12])
}
fn inverse(m: [f32; 16]) -> Result<[f32; 16], String> {
    let mut a = [[0.0f64; 8]; 4];
    for i in 0..4 {
        for j in 0..4 {
            a[i][j] = m[i + j * 4] as f64;
        }
        a[i][i + 4] = 1.0;
    }
    for c in 0..4 {
        let pivot = (c..4)
            .max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))
            .unwrap();
        if a[pivot][c].abs() < 1e-12 {
            return Err("singular bind".into());
        }
        a.swap(c, pivot);
        let div = a[c][c];
        for j in 0..8 {
            a[c][j] /= div;
        }
        for i in 0..4 {
            if i == c {
                continue;
            }
            let v = a[i][c];
            for j in 0..8 {
                a[i][j] -= v * a[c][j];
            }
        }
    }
    Ok(std::array::from_fn(|i| a[i % 4][i / 4 + 4] as f32))
}
fn flat<'a>(ns: &'a [m2a_core::mdl::NodeReport], out: &mut Vec<&'a m2a_core::mdl::NodeReport>) {
    for n in ns {
        out.push(n);
        flat(&n.children, out)
    }
}
fn project(w: &mut [f32; 4], n: usize) {
    let mut u = w[..n].to_vec();
    u.sort_by(|a, b| b.total_cmp(a));
    let mut sum = 0.0;
    let mut theta = 0.0;
    for i in 0..n {
        sum += u[i];
        let t = (sum - 1.0) / (i + 1) as f32;
        if u[i] > t {
            theta = t;
        }
    }
    for k in 0..n {
        w[k] = (w[k] - theta).max(0.0);
    }
    for k in n..4 {
        w[k] = 0.0;
    }
}
struct Group {
    p: P,
    bones: Vec<u32>,
    w: [f32; 4],
    original: [f32; 4],
}
struct Pose {
    clip: String,
    time: f32,
    bases: Vec<[P; 4]>,
}
struct Edge {
    a: usize,
    b: usize,
    len: f32,
}
fn pos(g: &Group, b: &[P; 4]) -> P {
    let mut p = [0.0; 3];
    for k in 0..g.bones.len() {
        for d in 0..3 {
            p[d] += b[k][d] * g.w[k];
        }
    }
    p
}
pub fn run(
    target_bytes: &[u8],
    reference_bytes: &[u8],
    rig_path: &str,
    out: &Path,
    minimax: bool,
    local: bool,
) -> Result<(), String> {
    let rig: Value = serde_json::from_slice(&fs::read(rig_path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let target = m2a_core::mdl::inspect_binary_mdl(target_bytes).map_err(|e| e.to_string())?;
    let reference =
        m2a_core::mdl::inspect_binary_mdl(reference_bytes).map_err(|e| e.to_string())?;
    let seg = &rig["segments"][0];
    let pts: Vec<P> =
        serde_json::from_value(seg["surfacePositions"].clone()).map_err(|e| e.to_string())?;
    let indices: Vec<usize> =
        serde_json::from_value(seg["surfaceIndices"].clone()).map_err(|e| e.to_string())?;
    let mut ns = Vec::new();
    flat(&target.node_tree.roots, &mut ns);
    let mut own_to_part = BTreeMap::new();
    for n in rig["nodes"].as_array().ok_or("rig nodes")? {
        if let Some(found) = ns
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(n["name"].as_str().unwrap()))
        {
            own_to_part.insert(n["id"].as_u64().unwrap() as u32, found.number);
        }
    }
    let mut by_point = BTreeMap::new();
    let mut vertex_group = Vec::new();
    let mut groups: Vec<Group> = Vec::new();
    for (i, p) in pts.iter().enumerate() {
        let key = p.map(f32::to_bits);
        let id = if let Some(&id) = by_point.get(&key) {
            id
        } else {
            let row = seg["referenceWeights"][i].as_array().ok_or("weights")?;
            if row.is_empty() || row.len() > 4 {
                return Err("invalid support".into());
            }
            let mut w = [0.0; 4];
            let mut bones = Vec::new();
            for (k, r) in row.iter().enumerate() {
                w[k] = r["value"].as_f64().unwrap() as f32;
                bones.push(r["boneNodeId"].as_u64().unwrap() as u32);
            }
            if bones.iter().any(|b| !own_to_part.contains_key(b)) {
                return Err("bone not mapped".into());
            }
            let id = groups.len();
            groups.push(Group {
                p: *p,
                bones,
                w,
                original: w,
            });
            by_point.insert(key, id);
            id
        };
        vertex_group.push(id);
    }
    let mut es = BTreeSet::new();
    for t in indices.chunks_exact(3) {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let a = vertex_group[a];
            let b = vertex_group[b];
            if a != b {
                es.insert((a.min(b), a.max(b)));
            }
        }
    }
    let edges: Vec<Edge> = es
        .into_iter()
        .filter_map(|(a, b)| {
            let len = norm(sub(groups[a].p, groups[b].p));
            (len > 1e-5).then_some(Edge { a, b, len })
        })
        .collect();
    let parts: Vec<u32> = own_to_part.values().copied().collect();
    let mut poses = Vec::new();
    for c in &reference.animations {
        let count = ((c.length * 30.0).ceil() as usize).max(2);
        for s in 0..=count {
            let time = c.length * s as f32 / count as f32;
            let rows = m2a_core::mdl::evaluate_reference_supermodel_node_world_matrices_v2(
                &target, &reference, &c.name, time, &parts,
            )
            .map_err(|e| e.to_string())?;
            let mut mats = BTreeMap::new();
            for r in rows {
                mats.insert(
                    r.node_part,
                    super::matrix_mul(r.sampled_world_matrix, inverse(r.bind_world_matrix)?),
                );
            }
            let bases = groups
                .iter()
                .map(|g| {
                    let mut b = [[0.0; 3]; 4];
                    for (k, bone) in g.bones.iter().enumerate() {
                        b[k] = tp(mats[&own_to_part[bone]], g.p);
                    }
                    b
                })
                .collect();
            poses.push(Pose {
                clip: c.name.clone(),
                time,
                bases,
            });
        }
    }
    // Check solver basis against the actual MDL evaluator before any authoring output.
    let check = &poses
        .iter()
        .find(|p| p.clip.eq_ignore_ascii_case("cwalk") && p.time > 0.2)
        .ok_or("walk missing")?;
    let samples = m2a_core::mdl::evaluate_reference_supermodel_render_deformation_samples_v3(
        &target,
        &reference,
        &check.clip,
        &[check.time],
    )
    .map_err(|e| e.to_string())?;
    let mut parity = 0.0f32;
    let mut checked = 0usize;
    for skin in &samples[0].skins {
        for v in &skin.vertices {
            let (g, dist) = groups
                .iter()
                .enumerate()
                .map(|(i, g)| (i, norm(sub(g.p, v.bind_world))))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .ok_or("no group")?;
            if dist > 1e-5 {
                return Err(format!("source/MDL position mismatch {dist}"));
            }
            parity = parity.max(norm(sub(pos(&groups[g], &check.bases[g]), v.sampled_world)));
            checked += 1;
        }
    }
    if parity > 2e-5 {
        return Err(format!("basis parity failure {parity}"));
    }
    if local {
        let info = repair_local(&mut groups, &poses, &edges, &indices, &vertex_group);
        let weights: Vec<Value> = vertex_group
            .iter()
            .map(|&i| {
                let g = &groups[i];
                json!(
                    g.bones
                        .iter()
                        .enumerate()
                        .filter(|(k, _)| g.w[*k] > 1e-8)
                        .map(|(k, b)| json!({"boneNodeId":b,"value":g.w[k]}))
                        .collect::<Vec<_>>()
                )
            })
            .collect();
        let output = json!({"algorithm":"MULTIPOSE_LOCAL_COORDINATE_REPAIR_V1","baseRigSha256":rig["contentSha256"],"referenceWeights":weights,"admission":"NOT_VALIDATED","basisParityMaxError":parity,"basisParityCheckedVertices":checked,"samplingHz":30,"poseCount":poses.len(),"groupCount":groups.len(),"repair":info,"geometryChanged":false});
        fs::write(
            out.join("strain-proposal.json"),
            serde_json::to_vec(&output).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        return Ok(());
    }
    let mut history = Vec::new();
    let mut best_loss = f64::INFINITY;
    let mut best_weights = Vec::new();
    for round in 0..100 {
        let mut grad = vec![[0.0f64; 4]; groups.len()];
        let mut hess = vec![[0.0f64; 4]; groups.len()];
        let mut loss = 0.0f64;
        let mut hard = 0usize;
        let mut min = 1.0f32;
        let mut max = 1.0f32;
        if minimax {
            let mut worst = vec![(0.0f32, 0usize); edges.len()];
            for (pi, p) in poses.iter().enumerate() {
                let q: Vec<P> = groups
                    .iter()
                    .enumerate()
                    .map(|(i, g)| pos(g, &p.bases[i]))
                    .collect();
                for (ei, e) in edges.iter().enumerate() {
                    let r = norm(sub(q[e.a], q[e.b])) / e.len;
                    min = min.min(r);
                    max = max.max(r);
                    if r < 0.25 || r > 4.0 {
                        hard += 1
                    }
                    let error = r - r.clamp(0.65, 1.7);
                    if error * error > worst[ei].0 {
                        worst[ei] = (error * error, pi);
                    }
                }
            }
            for (ei, e) in edges.iter().enumerate() {
                if worst[ei].0 < 1e-10 {
                    continue;
                }
                loss += worst[ei].0 as f64;
                let p = &poses[worst[ei].1];
                let d = sub(
                    pos(&groups[e.a], &p.bases[e.a]),
                    pos(&groups[e.b], &p.bases[e.b]),
                );
                let l = norm(d).max(1e-9);
                let r = l / e.len;
                let error = (r - r.clamp(0.65, 1.7)) as f64;
                for (i, sign) in [(e.a, 1.0f64), (e.b, -1.0f64)] {
                    let b = &p.bases[i];
                    for k in 1..groups[i].bones.len() {
                        let derivative = (dot(d, sub(b[k], b[0])) / (l * e.len)) as f64 * sign;
                        grad[i][k] += error * derivative;
                        hess[i][k] += derivative * derivative;
                    }
                }
            }
        } else {
            for p in &poses {
                let q: Vec<P> = groups
                    .iter()
                    .enumerate()
                    .map(|(i, g)| pos(g, &p.bases[i]))
                    .collect();
                for e in &edges {
                    let d = sub(q[e.a], q[e.b]);
                    let l = norm(d).max(1e-9);
                    let r = l / e.len;
                    min = min.min(r);
                    max = max.max(r);
                    if r < 0.25 || r > 4.0 {
                        hard += 1
                    }
                    let goal = r.clamp(0.65, 1.7);
                    if (r - goal).abs() < 1e-6 {
                        continue;
                    }
                    let error = (r - goal) as f64;
                    loss += error * error;
                    for (i, sign) in [(e.a, 1.0f64), (e.b, -1.0f64)] {
                        let b = &p.bases[i];
                        for k in 1..groups[i].bones.len() {
                            let derivative = (dot(d, sub(b[k], b[0])) / (l * e.len)) as f64 * sign;
                            grad[i][k] += error * derivative;
                            hess[i][k] += derivative * derivative;
                        }
                    }
                }
            }
        }
        if loss < best_loss {
            best_loss = loss;
            best_weights = groups.iter().map(|g| g.w).collect();
        }
        if round % 10 == 0 || round == 99 {
            eprintln!("STRAIN_SOLVER round={round} loss={loss} hard={hard} min={min} max={max}");
            history.push(
                json!({"round":round,"loss":loss,"hardEdgeTimeSamples":hard,"min":min,"max":max}),
            );
        }
        if round == 99 {
            break;
        }
        for (i, g) in groups.iter_mut().enumerate() {
            if g.bones.len() < 2 {
                continue;
            }
            let mut nw = g.w;
            for k in 1..g.bones.len() {
                let prior_scale = if minimax { 0.01 } else { 2.0 };
                let prior = prior_scale * (g.w[k] - g.original[k]) as f64;
                let step = (-0.18 * (grad[i][k] + prior) / (hess[i][k] + prior_scale))
                    .clamp(-0.015, 0.015) as f32;
                nw[k] += step;
                nw[0] -= step;
            }
            project(&mut nw, g.bones.len());
            g.w = nw;
        }
    }
    for (g, w) in groups.iter_mut().zip(best_weights) {
        g.w = w;
    }
    let weights: Vec<Value> = vertex_group
        .iter()
        .map(|&i| {
            let g = &groups[i];
            json!(
                g.bones
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| g.w[*k] > 1e-8)
                    .map(|(k, b)| json!({"boneNodeId":b,"value":g.w[k]}))
                    .collect::<Vec<_>>()
            )
        })
        .collect();
    let output = json!({"algorithm":"MULTIPOSE_SURFACE_STRAIN_PROPOSAL_V1","baseRigSha256":rig["contentSha256"],"referenceWeights":weights,"admission":"NOT_VALIDATED","basisParityMaxError":parity,"basisParityCheckedVertices":checked,"samplingHz":30,"poseCount":poses.len(),"groupCount":groups.len(),"history":history,"aggregation":if minimax{"WORST_POSE_PER_EDGE"}else{"SUM_ALL_POSES"},"bestLoss":best_loss,"geometryChanged":false});
    fs::write(
        out.join("strain-proposal.json"),
        serde_json::to_vec(&output).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simplex_preserves_mass_and_nonnegative() {
        let mut w = [-0.2, 0.4, 1.2, 0.0];
        project(&mut w, 3);
        assert!((w.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(w.iter().all(|v| *v >= 0.0));
    }
    #[test]
    fn inverse_roundtrip() {
        let m = [
            0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
        ];
        let p = [0.2, 0.3, 0.4];
        assert!(norm(sub(tp(inverse(m).unwrap(), tp(m, p)), p)) < 1e-6);
    }
}

fn cross(a: P, b: P) -> P {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
struct Tri {
    ids: [usize; 3],
    area: f32,
}
fn area(a: P, b: P, c: P) -> f32 {
    norm(cross(sub(b, a), sub(c, a)))
}
fn edge_penalty(r: f32) -> f64 {
    let r = r.max(1e-6) as f64;
    (0.20 / r - 1.0).max(0.0).powi(2)
        + (r / 3.0 - 1.0).max(0.0).powi(2)
        + 0.0001 * (r - r.clamp(0.65, 1.7)).powi(2)
}
fn tri_penalty(r: f32) -> f64 {
    let r = r.max(1e-7) as f64;
    (0.03 / r - 1.0).max(0.0).powi(2) + 0.001 * (r / 30.0 - 1.0).max(0.0).powi(2)
}
fn repair_local(
    groups: &mut [Group],
    poses: &[Pose],
    edges: &[Edge],
    indices: &[usize],
    vg: &[usize],
) -> Value {
    let tris: Vec<Tri> = indices
        .chunks_exact(3)
        .filter_map(|t| {
            let ids = [vg[t[0]], vg[t[1]], vg[t[2]]];
            let a = area(groups[ids[0]].p, groups[ids[1]].p, groups[ids[2]].p);
            (a > 1e-11).then_some(Tri { ids, area: a })
        })
        .collect();
    let mut ea = vec![Vec::new(); groups.len()];
    let mut ta = vec![Vec::new(); groups.len()];
    for (i, e) in edges.iter().enumerate() {
        ea[e.a].push(i);
        ea[e.b].push(i);
    }
    for (i, t) in tris.iter().enumerate() {
        for &g in &t.ids {
            ta[g].push(i);
        }
    }
    let mut q: Vec<Vec<P>> = poses
        .iter()
        .map(|p| {
            groups
                .iter()
                .enumerate()
                .map(|(i, g)| pos(g, &p.bases[i]))
                .collect()
        })
        .collect();
    let mut active = BTreeSet::new();
    for row in &q {
        for e in edges {
            let r = norm(sub(row[e.a], row[e.b])) / e.len;
            if r < 0.20 || r > 3.5 {
                active.insert(e.a);
                active.insert(e.b);
            }
        }
        for t in &tris {
            let r = area(row[t.ids[0]], row[t.ids[1]], row[t.ids[2]]) / t.area;
            if r < 0.03 {
                active.extend(t.ids);
            }
        }
    }
    let seed_count = active.len();
    for g in active.clone() {
        for &ei in &ea[g] {
            active.insert(edges[ei].a);
            active.insert(edges[ei].b);
        }
    }
    active.retain(|&i| groups[i].bones.len() > 1);
    let cost = |g: usize, w: [f32; 4], q: &Vec<Vec<P>>, groups: &[Group]| -> f64 {
        let mut result = 0.0;
        for (pi, p) in poses.iter().enumerate() {
            let mut point = [0.0; 3];
            for k in 0..groups[g].bones.len() {
                for d in 0..3 {
                    point[d] += p.bases[g][k][d] * w[k];
                }
            }
            for &ei in &ea[g] {
                let e = &edges[ei];
                let other = if e.a == g { e.b } else { e.a };
                result += edge_penalty(norm(sub(point, q[pi][other])) / e.len);
            }
            for &ti in &ta[g] {
                let t = &tris[ti];
                let a = t.ids.map(|id| if id == g { point } else { q[pi][id] });
                result += tri_penalty(area(a[0], a[1], a[2]) / t.area);
            }
        }
        for k in 0..groups[g].bones.len() {
            result += 0.01 * ((w[k] - groups[g].original[k]) as f64).powi(2);
        }
        result
    };
    let mut history = Vec::new();
    for round in 0..24 {
        let step = if round < 8 {
            0.005
        } else if round < 16 {
            0.001
        } else {
            0.0002
        };
        let mut changes = 0;
        let mut reduction = 0.0;
        for &g in &active {
            let old = groups[g].w;
            let initial = cost(g, old, &q, groups);
            let mut best = initial;
            let mut selected = old;
            for a in 0..groups[g].bones.len() {
                for b in 0..groups[g].bones.len() {
                    if a == b || old[b] < step {
                        continue;
                    }
                    let mut nw = old;
                    nw[a] += step;
                    nw[b] -= step;
                    let c = cost(g, nw, &q, groups);
                    if c < best - 1e-9 {
                        best = c;
                        selected = nw;
                    }
                }
            }
            if selected != old {
                groups[g].w = selected;
                for (pi, p) in poses.iter().enumerate() {
                    q[pi][g] = pos(&groups[g], &p.bases[g]);
                }
                changes += 1;
                reduction += initial - best;
            }
        }
        eprintln!(
            "LOCAL_REPAIR round={round} active={} changes={changes} reduction={reduction}",
            active.len()
        );
        history.push(
            json!({"round":round,"step":step,"changedGroups":changes,"lossReduction":reduction}),
        );
    }
    json!({"seedGroups":seed_count,"activeGroups":active.len(),"history":history,"triangleCount":tris.len(),"allPosesCheckedPerMove":poses.len()})
}

#[cfg(test)]
mod local_tests {
    use super::*;
    #[test]
    fn collapse_penalties_escalate() {
        assert!(edge_penalty(0.04) > edge_penalty(0.1));
        assert!(edge_penalty(0.1) > edge_penalty(0.2));
        assert!(tri_penalty(0.002) > tri_penalty(0.02));
        assert_eq!(tri_penalty(1.0), 0.0);
    }
    #[test]
    fn rigid_transform_preserves_area() {
        let points = [[0.1, 0.2, 0.3], [0.7, 0.3, 0.1], [0.1, 0.9, 0.5]];
        let m = [
            0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
        ];
        let moved = points.map(|p| tp(m, p));
        assert!(
            (area(points[0], points[1], points[2]) - area(moved[0], moved[1], moved[2])).abs()
                < 1e-6
        );
    }
}
