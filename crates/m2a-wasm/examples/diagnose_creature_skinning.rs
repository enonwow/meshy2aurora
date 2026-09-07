mod creature_strain_solver;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let source = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let reference = read_key_resource(Path::new(&args[2]), "c_wolf", 2002)?;
    let sha = |b: &[u8]| format!("{:x}", Sha256::digest(b));
    let expected_source_sha = if let Some(i) = args.iter().position(|s| s == "--source-sha256") {
        args.get(i + 1)
            .ok_or("--source-sha256 requires a value")?
            .as_str()
    } else if args
        .get(4)
        .is_some_and(|s| s == "pipeline" || s == "preview" || s == "product")
    {
        "a2d74adb6dd36c8577c59a331ba98d0b2f41c4c544d1782a6968997b96a876ed"
    } else {
        "dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689"
    };
    if sha(&source) != expected_source_sha {
        return Err("CREATURE-DIAG-SOURCE-SHA-MISMATCH".to_owned());
    }
    if args
        .get(4)
        .is_some_and(|s| s == "strain-optimize" || s == "strain-minimax" || s == "strain-local")
    {
        let model =
            fs::read(args.get(5).ok_or("own model required")?).map_err(|e| e.to_string())?;
        creature_strain_solver::run(
            &model,
            &reference,
            args.get(6).ok_or("rig required")?,
            Path::new(&args[3]),
            args[4] == "strain-minimax",
            args[4] == "strain-local",
        )?;
        println!("STRAIN_PROPOSAL_WRITTEN");
        return Ok(());
    }
    if args.get(4).is_some_and(|s| s == "region-audit") {
        let model = fs::read(args.get(5).ok_or("Region audit requires own model")?)
            .map_err(|e| e.to_string())?;
        let mut report = audit_motion_regions(
            &model,
            &reference,
            args.get(6).ok_or("Region audit requires regions JSON")?,
        )?;
        if args.iter().any(|s| s == "--all-triangles") {
            report["fullTriangleAudit"] = audit_all_surface_triangles(&model, &reference)?;
        }
        fs::write(
            Path::new(&args[3]).join("region-audit.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        println!("REGION_AUDIT_WRITTEN");
        return Ok(());
    }
    if args.get(4).is_some_and(|s| s == "joint-audit") {
        let report = audit_reference_joint_lengths(&reference)?;
        fs::write(
            Path::new(&args[3]).join("joint-length-audit.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        println!("JOINT_AUDIT_WRITTEN");
        return Ok(());
    }
    let chain = json!([{"resref":"c_wolf","supermodelResref":"NULL","format":"BINARY","sha256":sha(&reference),"byteOffset":0,"byteLength":reference.len()}]).to_string();
    if args.get(4).is_some_and(|s| s == "product") {
        let appearance = read_key_resource(Path::new(&args[2]), "appearance", 2017)?;
        let authoring = fs::read_to_string(args.get(5).ok_or("Product requires sealed authoring")?)
            .map_err(|e| e.to_string())?;
        let identity=json!({"modelResref":"c_tlcborzoi","textureResref":"tlcborzoitex","materialResref":"tlcborzoimtr","hakResref":"tlc_brz_260906","appearanceLabel":"TLC_BORZOI_TRIPO","appearanceDonorResrefs":["c_wolf"]}).to_string();
        let mut product = m2a_wasm::build_reference_supermodel_creature_product_v4_native(
            "c_wolf",
            &source,
            &appearance,
            &reference,
            &chain,
            &identity,
            "POSITIVE_Z",
            &authoring,
        )?;
        for (name, bytes) in [
            ("c_tlcborzoi.mdl", product.take_model_bytes()),
            ("tlcborzoitex.tga", product.take_texture_bytes()),
            ("tlcborzoimtr.mtr", product.take_material_bytes()),
            ("tlc_brz_260906.hak", product.take_hak_bytes()),
            ("appearance.2da", product.take_appearance_two_da_bytes()),
        ] {
            fs::write(Path::new(&args[3]).join(name), bytes).map_err(|e| e.to_string())?;
        }
        for (name, text) in [
            ("product-report.json", product.report_json()),
            ("manifest.json", product.manifest_json()),
            ("summary.json", product.summary_json()),
            ("readback.json", product.readback_json()),
        ] {
            fs::write(Path::new(&args[3]).join(name), text).map_err(|e| e.to_string())?;
        }
        println!("CREATURE_PRODUCT_MATERIALIZED");
        return Ok(());
    }
    if args
        .get(4)
        .is_some_and(|s| s == "pipeline" || s == "preview")
    {
        if args.get(4).is_some_and(|s| s == "preview") {
            let mut p = if let Some(authoring_path) = args.get(5) {
                let authoring = fs::read_to_string(authoring_path).map_err(|e| e.to_string())?;
                m2a_wasm::build_reference_supermodel_authored_preview_v2_native(
                    "c_wolf",
                    &source,
                    &reference,
                    &chain,
                    "POSITIVE_Z",
                    &authoring,
                )?
            } else {
                m2a_wasm::build_reference_supermodel_applied_preview_v2_native(
                    "c_wolf",
                    &source,
                    &reference,
                    &chain,
                    "POSITIVE_Z",
                )?
            };
            fs::write(format!("{}/preview.mdl", args[3]), p.take_model_bytes())
                .map_err(|e| e.to_string())?;
            fs::write(format!("{}/readback.json", args[3]), p.readback_json())
                .map_err(|e| e.to_string())?;
            fs::write(
                format!("{}/preview-report.json", args[3]),
                p.apply_report_json(),
            )
            .map_err(|e| e.to_string())?;
            fs::write(format!("{}/rig.json", args[3]), p.target_rig_json())
                .map_err(|e| e.to_string())?;
            fs::write(format!("{}/authoring.json", args[3]), p.authoring_json())
                .map_err(|e| e.to_string())?;
            println!("DIAGNOSTIC_PREVIEW_WRITTEN");
            return Ok(());
        }
        let prepared = m2a_wasm::prepare_reference_supermodel_rig_v2_native(
            "c_wolf",
            &source,
            &reference,
            &chain,
            "POSITIVE_Z",
        );
        match prepared {
            Ok(p) => {
                fs::write(format!("{}/authoring.json", args[3]), p.authoring_json())
                    .map_err(|e| e.to_string())?;
                fs::write(format!("{}/rig.json", args[3]), p.target_rig_json())
                    .map_err(|e| e.to_string())?;
                fs::write(format!("{}/report.json", args[3]), p.report_json())
                    .map_err(|e| e.to_string())?;
                println!("PREPARED");
            }
            Err(e) => {
                fs::write(format!("{}/report.json", args[3]), &e).map_err(|e| e.to_string())?;
                println!("{e}");
            }
        }
        return Ok(());
    }
    let _ = chain;
    let analysis = m2a_core::reference_supermodel_generic::analyze_reference_supermodel_chain_v2("c_wolf", &[m2a_core::reference_supermodel_generic::ReferenceSupermodelChainPayloadV2 {
 resource: m2a_core::reference_supermodel_generic::ReferenceSupermodelChainResourceV2 {resref:"c_wolf".into(), supermodel_resref:"NULL".into(), format:m2a_core::reference_supermodel_generic::ReferenceSupermodelFormatV2::Binary, sha256:sha(&reference), byte_length:reference.len()},payload:reference
 }]).map_err(|e|e.to_string())?;
    let source = m2a_core::glb::ingest_glb(&source, &m2a_core::glb::GlbLimits::default())
        .map_err(|e| e.to_string())?;
    let contract = &analysis.motion_contract;
    let mut worlds: Vec<[f32; 16]> = Vec::new();
    for n in &contract.nodes {
        let local = n.carrier_bind_local_matrix;
        worlds.push(if let Some(p) = n.parent_part_number {
            matrix_mul(worlds[p as usize], local)
        } else {
            local
        });
    }
    let mut render = Vec::new();
    collect_render(
        &analysis.combined_reference.node_tree.roots,
        &worlds,
        &mut render,
    );
    dump_meshes(&analysis.combined_reference.node_tree.roots);
    let rb = point_bounds(&render);
    let points = source.ir.primitives[0]
        .positions
        .iter()
        .map(|p| [-p[0], p[2], p[1]])
        .collect::<Vec<_>>();
    let sb = point_bounds(&points);
    let scale = (rb[1][2] - rb[0][2]) / (sb[1][2] - sb[0][2]);
    let sa = [
        (sb[0][0] + sb[1][0]) * 0.5,
        (sb[0][1] + sb[1][1]) * 0.5,
        sb[0][2],
    ];
    let ra = [
        (rb[0][0] + rb[1][0]) * 0.5,
        (rb[0][1] + rb[1][1]) * 0.5,
        rb[0][2],
    ];
    let registered = points
        .iter()
        .map(|p| std::array::from_fn::<_, 3, _>(|a| (p[a] - sa[a]) * scale + ra[a]))
        .collect::<Vec<_>>();
    fs::write(format!("{}/registration.json",args[3]),serde_json::to_vec(&json!({"bounds":rb,"scale":scale,"positions":registered,"indices":source.ir.primitives[0].indices,"joints":contract.nodes.iter().enumerate().map(|(i,n)|json!({"id":i,"name":n.name,"parent":n.parent_part_number,"position":[worlds[i][12],worlds[i][13],worlds[i][14]],"class":n.carrier_class})).collect::<Vec<_>>() })).unwrap()).map_err(|e|e.to_string())?;
    let guide_allowed=contract.nodes.iter().filter(|n| n.carrier_class==m2a_core::reference_supermodel_motion::ReferenceSupermodelCarrierClassV3::SkinRelevant).map(|n|n.part_number).collect::<Vec<_>>();
    let guide = m2a_core::reference_surface_guides::reference_surface_labels_v1(
        &analysis.combined_reference,
        contract,
        &registered,
        &guide_allowed,
    )
    .map_err(|e| e.to_string())?;
    let mut counts = std::collections::BTreeMap::new();
    for label in &guide {
        *counts
            .entry(contract.nodes[*label as usize].name.clone())
            .or_insert(0) += 1;
    }
    eprintln!("GUIDE:{}", json!(counts));
    fs::write(
        format!("{}/guide-labels.json", args[3]),
        serde_json::to_vec(&guide).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    let result = if args.get(5).is_some_and(|s| s == "fit") {
        let fitted = registered
            .iter()
            .map(|p| [p[0] * 0.85, p[1] * 0.72 - 0.16, p[2] * 0.83 + 0.037])
            .collect::<Vec<_>>();
        let fb = point_bounds(&fitted);
        fs::write(
            format!("{}/fit-points.json", args[3]),
            serde_json::to_vec(&fitted).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        {
            let _ = fb;
            let mut source_fit = source.clone();
            source_fit.ir.primitives[0].positions =
                fitted.iter().map(|p| [-p[0], p[2], p[1]]).collect();
            m2a_core::reference_supermodel_generic::derive_registered_reference_supermodel_rig_from_glb_v1(&source_fit,contract,&analysis.combined_reference,m2a_core::profile_a::CreatureSourceForwardV1::PositiveZ,m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1 {allow_excessive_branch_boundary_repair:args.get(4).is_some_and(|s|s=="diagnose"), ..Default::default()})
        }
    } else {
        m2a_core::reference_supermodel_generic::derive_immutable_reference_supermodel_rig_from_glb_v4(&source,contract,&analysis.combined_reference,m2a_core::profile_a::CreatureSourceForwardV1::PositiveZ,m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1 {allow_excessive_branch_boundary_repair:args.get(4).is_some_and(|s|s=="diagnose"), ..Default::default()})
    };
    let report = match result {
        Ok(p) => {
            fs::write(
                format!("{}/rig.json", args[3]),
                serde_json::to_string(&p.rig).unwrap(),
            )
            .map_err(|e| e.to_string())?;
            serde_json::to_value(&p.report).unwrap()
        }
        Err(e) => json!({"error":e}),
    };
    fs::write(
        format!("{}/report.json", args[3]),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    println!("{}", report.get("error").unwrap_or(&json!("PREPARED")));
    Ok(())
}
fn read_key_resource(
    key_path: &Path,
    wanted_resref: &str,
    wanted_type: u16,
) -> Result<Vec<u8>, String> {
    const KEY_HEADER_SIZE: usize = 64;
    const KEY_ENTRY_SIZE: usize = 22;
    let key = fs::read(key_path).map_err(|error| format!("CREATURE-DIAG-KEY-READ: {error}"))?;
    if key.len() < KEY_HEADER_SIZE || &key[0..4] != b"KEY " || &key[4..8] != b"V1  " {
        return Err("CREATURE-DIAG-KEY-HEADER: expected KEY V1".to_owned());
    }
    let bif_count = u32_at(&key, 8)? as usize;
    let key_count = u32_at(&key, 12)? as usize;
    let bif_table_offset = u32_at(&key, 16)? as usize;
    let key_table_offset = u32_at(&key, 20)? as usize;
    checked_table(&key, bif_table_offset, bif_count, 12, "BIF table")?;
    checked_table(
        &key,
        key_table_offset,
        key_count,
        KEY_ENTRY_SIZE,
        "KEY table",
    )?;

    let mut selected = None;
    for index in 0..key_count {
        let offset = key_table_offset + index * KEY_ENTRY_SIZE;
        let resref = logical_resref(&key[offset..offset + 16]);
        let resource_type = u16::from_le_bytes([key[offset + 16], key[offset + 17]]);
        if resref.eq_ignore_ascii_case(wanted_resref) && resource_type == wanted_type {
            if selected.is_some() {
                return Err(format!(
                    "CREATURE-DIAG-KEY-DUPLICATE: {wanted_resref}:{wanted_type}"
                ));
            }
            selected = Some(u32_at(&key, offset + 18)?);
        }
    }
    let resource_id = selected
        .ok_or_else(|| format!("CREATURE-DIAG-KEY-NOT-FOUND: {wanted_resref}:{wanted_type}"))?;
    let bif_index = (resource_id >> 20) as usize;
    let resource_index = resource_id & 0x000f_ffff;
    if bif_index >= bif_count {
        return Err("CREATURE-DIAG-KEY-BIF-INDEX-OOB".to_owned());
    }
    let bif_entry = bif_table_offset + bif_index * 12;
    let file_name_offset = u32_at(&key, bif_entry + 4)? as usize;
    let file_name_size = u16::from_le_bytes([key[bif_entry + 8], key[bif_entry + 9]]) as usize;
    let file_name_end = file_name_offset
        .checked_add(file_name_size)
        .filter(|end| *end <= key.len())
        .ok_or("CREATURE-DIAG-KEY-BIF-NAME-OOB")?;
    let logical_bif_name = String::from_utf8_lossy(&key[file_name_offset..file_name_end])
        .trim_end_matches('\0')
        .replace('\\', "/");
    let native_relative = logical_bif_name.replace('/', std::path::MAIN_SEPARATOR_STR);
    let key_parent = key_path.parent().ok_or("CREATURE-DIAG-KEY-PARENT")?;
    let installation_root = key_parent.parent().unwrap_or(key_parent);
    let candidates = [
        installation_root.join(&native_relative),
        key_parent.join(&native_relative),
    ];
    let bif_path = candidates
        .iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("CREATURE-DIAG-BIF-NOT-FOUND: {logical_bif_name}"))?;
    read_bif_resource(bif_path, resource_index, wanted_type)
}

fn read_bif_resource(
    path: &Path,
    resource_index: u32,
    wanted_type: u16,
) -> Result<Vec<u8>, String> {
    let mut bif = File::open(path).map_err(|error| format!("CREATURE-DIAG-BIF-OPEN: {error}"))?;
    let mut header = [0_u8; 20];
    bif.read_exact(&mut header)
        .map_err(|error| format!("CREATURE-DIAG-BIF-HEADER-READ: {error}"))?;
    if &header[0..4] != b"BIFF" || &header[4..8] != b"V1  " {
        return Err("CREATURE-DIAG-BIF-HEADER: expected BIFF V1".to_owned());
    }
    let variable_count = u32::from_le_bytes(header[8..12].try_into().unwrap());
    let variable_table_offset = u32::from_le_bytes(header[16..20].try_into().unwrap());
    if resource_index >= variable_count {
        return Err("CREATURE-DIAG-BIF-RESOURCE-INDEX-OOB".to_owned());
    }
    bif.seek(SeekFrom::Start(
        u64::from(variable_table_offset) + u64::from(resource_index) * 16,
    ))
    .map_err(|error| format!("CREATURE-DIAG-BIF-SEEK: {error}"))?;
    let mut entry = [0_u8; 16];
    bif.read_exact(&mut entry)
        .map_err(|error| format!("CREATURE-DIAG-BIF-ENTRY-READ: {error}"))?;
    let payload_offset = u32::from_le_bytes(entry[4..8].try_into().unwrap());
    let payload_size = u32::from_le_bytes(entry[8..12].try_into().unwrap());
    let resource_type = u32::from_le_bytes(entry[12..16].try_into().unwrap());
    if resource_type != u32::from(wanted_type) {
        return Err(format!(
            "CREATURE-DIAG-BIF-TYPE-MISMATCH: expected {wanted_type}, got {resource_type}"
        ));
    }
    let mut payload = vec![0_u8; payload_size as usize];
    bif.seek(SeekFrom::Start(u64::from(payload_offset)))
        .map_err(|error| format!("CREATURE-DIAG-BIF-PAYLOAD-SEEK: {error}"))?;
    bif.read_exact(&mut payload)
        .map_err(|error| format!("CREATURE-DIAG-BIF-PAYLOAD-READ: {error}"))?;
    Ok(payload)
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or("CREATURE-DIAG-BINARY-U32-OOB")?;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn checked_table(
    bytes: &[u8],
    offset: usize,
    count: usize,
    stride: usize,
    label: &str,
) -> Result<(), String> {
    let end = count
        .checked_mul(stride)
        .and_then(|length| offset.checked_add(length))
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| format!("CREATURE-DIAG-{label}-OOB"))?;
    let _ = end;
    Ok(())
}

fn logical_resref(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
fn matrix_mul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    std::array::from_fn(|i| (0..4).map(|k| a[i % 4 + k * 4] * b[k + i / 4 * 4]).sum())
}
fn point(m: [f32; 16], p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|a| m[a] * p[0] + m[a + 4] * p[1] + m[a + 8] * p[2] + m[a + 12])
}
fn point_bounds(ps: &[[f32; 3]]) -> [[f32; 3]; 2] {
    [
        std::array::from_fn(|a| ps.iter().map(|p| p[a]).fold(f32::INFINITY, f32::min)),
        std::array::from_fn(|a| ps.iter().map(|p| p[a]).fold(f32::NEG_INFINITY, f32::max)),
    ]
}
fn collect_render(ns: &[m2a_core::mdl::NodeReport], ws: &[[f32; 16]], ps: &mut Vec<[f32; 3]>) {
    for n in ns {
        if let Some(m) = n.mesh.as_ref().filter(|m| m.render != 0) {
            ps.extend(
                m.vertices
                    .iter()
                    .map(|v| point(ws[n.number as usize], [v.x, v.y, v.z])),
            );
        }
        collect_render(&n.children, ws, ps);
    }
}

fn dump_meshes(ns: &[m2a_core::mdl::NodeReport]) {
    for n in ns {
        eprintln!(
            "MESH:{}:{}:{:?}",
            n.number,
            n.name,
            n.mesh.as_ref().map(|m| (m.render, m.vertices.len()))
        );
        dump_meshes(&n.children);
    }
}

fn audit_motion_regions(
    target_bytes: &[u8],
    reference_bytes: &[u8],
    regions_path: &str,
) -> Result<serde_json::Value, String> {
    let target = m2a_core::mdl::inspect_binary_mdl(target_bytes).map_err(|e| e.to_string())?;
    let reference =
        m2a_core::mdl::inspect_binary_mdl(reference_bytes).map_err(|e| e.to_string())?;
    #[derive(serde::Deserialize)]
    struct Region {
        name: String,
        min: [f32; 3],
        max: [f32; 3],
    }
    let regions: Vec<Region> =
        serde_json::from_str(&fs::read_to_string(regions_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    fn nodes<'a>(
        ns: &'a [m2a_core::mdl::NodeReport],
        out: &mut Vec<&'a m2a_core::mdl::NodeReport>,
    ) {
        for n in ns {
            out.push(n);
            nodes(&n.children, out)
        }
    }
    let mut flat = Vec::new();
    nodes(&target.node_tree.roots, &mut flat);
    let mut edges = std::collections::BTreeMap::new();
    for n in flat {
        if let Some(m) = &n.mesh {
            let mut unique = std::collections::BTreeSet::new();
            for f in &m.faces {
                let [a, b, c] = f.vertex_indices;
                for (x, y) in [(a, b), (b, c), (c, a)] {
                    unique.insert((x.min(y) as usize, x.max(y) as usize));
                }
            }
            edges.insert(n.number, unique);
        }
    }
    let mut clips = Vec::new();
    for clip in &reference.animations {
        let count = ((clip.length * 30.0).ceil() as usize).max(2);
        let times = (0..=count)
            .map(|i| clip.length * i as f32 / count as f32)
            .collect::<Vec<_>>();
        let samples = m2a_core::mdl::evaluate_reference_supermodel_render_deformation_samples_v3(
            &target, &reference, &clip.name, &times,
        )
        .map_err(|e| e.to_string())?;
        let mut stats=regions.iter().map(|r|json!({"name":r.name,"edgeSamples":0,"belowQuarter":0,"aboveFour":0,"minRatio":1.0,"maxRatio":1.0})).collect::<Vec<_>>();
        for sample in samples {
            for skin in sample.skins {
                if let Some(es) = edges.get(&skin.node_part) {
                    for &(a, b) in es {
                        let va = &skin.vertices[a];
                        let vb = &skin.vertices[b];
                        let center = std::array::from_fn::<_, 3, _>(|i| {
                            (va.bind_world[i] + vb.bind_world[i]) * 0.5
                        });
                        let Some(ri) = regions.iter().position(|r| {
                            (0..3).all(|i| center[i] >= r.min[i] && center[i] <= r.max[i])
                        }) else {
                            continue;
                        };
                        let distance = |p: [f32; 3], q: [f32; 3]| {
                            (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt()
                        };
                        let bind = distance(va.bind_world, vb.bind_world);
                        if bind < 1e-5 {
                            continue;
                        }
                        let ratio = distance(va.sampled_world, vb.sampled_world) / bind;
                        let s = &mut stats[ri];
                        s["edgeSamples"] = json!(s["edgeSamples"].as_u64().unwrap() + 1);
                        if ratio < 0.25 {
                            s["belowQuarter"] = json!(s["belowQuarter"].as_u64().unwrap() + 1)
                        }
                        if ratio > 4.0 {
                            s["aboveFour"] = json!(s["aboveFour"].as_u64().unwrap() + 1)
                        }
                        if ratio < (s["minRatio"].as_f64().unwrap() as f32) {
                            s["minRatio"] = json!(ratio);
                            s["worstCollapse"] = json!({"time":sample.time_seconds,"bind":[va.bind_world,vb.bind_world],"moved":[va.sampled_world,vb.sampled_world],"indices":[a,b]});
                        }
                        if ratio > (s["maxRatio"].as_f64().unwrap() as f32) {
                            s["maxRatio"] = json!(ratio);
                            s["worstStretch"] = json!({"time":sample.time_seconds,"bind":[va.bind_world,vb.bind_world],"moved":[va.sampled_world,vb.sampled_world],"indices":[a,b]});
                        }
                    }
                }
            }
        }
        clips.push(json!({"clip":clip.name,"sampleCount":times.len(),"regions":stats}));
    }
    Ok(
        json!({"algorithm":"REGION_SURFACE_STRAIN_V1","samplingHz":30,"modelSha256":format!("{:x}",Sha256::digest(target_bytes)),"referenceSha256":format!("{:x}",Sha256::digest(reference_bytes)),"nativeRuntime":"NOT_TESTED","clips":clips}),
    )
}

fn audit_reference_joint_lengths(reference_bytes: &[u8]) -> Result<serde_json::Value, String> {
    let reference =
        m2a_core::mdl::inspect_binary_mdl(reference_bytes).map_err(|e| e.to_string())?;
    fn walk(
        ns: &[m2a_core::mdl::NodeReport],
        parent: Option<u32>,
        rows: &mut Vec<(u32, Option<u32>, String)>,
    ) {
        for n in ns {
            rows.push((n.number, parent, n.name.clone()));
            walk(&n.children, Some(n.number), rows)
        }
    }
    let mut nodes = Vec::new();
    walk(&reference.node_tree.roots, None, &mut nodes);
    let ids = nodes.iter().map(|r| r.0).collect::<Vec<_>>();
    let mut clips = Vec::new();
    for clip in &reference.animations {
        let mut ratios = nodes
            .iter()
            .map(|n| json!({"name":n.2,"min":1.0,"max":1.0}))
            .collect::<Vec<_>>();
        for step in 0..=30 {
            let matrices = m2a_core::mdl::evaluate_reference_supermodel_node_world_matrices_v2(
                &reference,
                &reference,
                &clip.name,
                clip.length * step as f32 / 30.0,
                &ids,
            )
            .map_err(|e| e.to_string())?;
            for (i, n) in nodes.iter().enumerate() {
                let Some(parent) = n.1 else { continue };
                let j = ids.iter().position(|id| *id == parent).unwrap();
                let dist = |a: &[f32; 16], b: &[f32; 16]| {
                    (12..15).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()
                };
                let bind = dist(
                    &matrices[i].bind_world_matrix,
                    &matrices[j].bind_world_matrix,
                );
                if bind < 1e-5 {
                    continue;
                }
                let ratio = dist(
                    &matrices[i].sampled_world_matrix,
                    &matrices[j].sampled_world_matrix,
                ) / bind;
                let r = &mut ratios[i];
                r["min"] = json!(ratio.min(r["min"].as_f64().unwrap() as f32));
                r["max"] = json!(ratio.max(r["max"].as_f64().unwrap() as f32));
            }
        }
        clips.push(json!({"clip":clip.name,"jointLengthRatios":ratios}));
    }
    Ok(json!({"referenceSha256":format!("{:x}",Sha256::digest(reference_bytes)),"clips":clips}))
}

fn audit_all_surface_triangles(
    target_bytes: &[u8],
    reference_bytes: &[u8],
) -> Result<serde_json::Value, String> {
    let target = m2a_core::mdl::inspect_binary_mdl(target_bytes).map_err(|e| e.to_string())?;
    let reference =
        m2a_core::mdl::inspect_binary_mdl(reference_bytes).map_err(|e| e.to_string())?;
    fn collect(
        ns: &[m2a_core::mdl::NodeReport],
        out: &mut std::collections::BTreeMap<u32, Vec<[usize; 3]>>,
    ) {
        for n in ns {
            if let Some(m) = &n.mesh {
                out.insert(
                    n.number,
                    m.faces
                        .iter()
                        .map(|f| f.vertex_indices.map(usize::from))
                        .collect(),
                );
            }
            collect(&n.children, out)
        }
    }
    fn area(p: [[f32; 3]; 3]) -> f32 {
        let a = std::array::from_fn::<_, 3, _>(|i| p[1][i] - p[0][i]);
        let b = std::array::from_fn::<_, 3, _>(|i| p[2][i] - p[0][i]);
        let c = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        (c.iter().map(|v| v * v).sum::<f32>()).sqrt()
    }
    let mut faces = std::collections::BTreeMap::new();
    collect(&target.node_tree.roots, &mut faces);
    let mut clips = Vec::new();
    for c in &reference.animations {
        let count = ((c.length * 30.0).ceil() as usize).max(2);
        let times: Vec<f32> = (0..=count)
            .map(|i| c.length * i as f32 / count as f32)
            .collect();
        let samples = m2a_core::mdl::evaluate_reference_supermodel_render_deformation_samples_v3(
            &target, &reference, &c.name, &times,
        )
        .map_err(|e| e.to_string())?;
        let mut min = 1.0f32;
        let mut max = 1.0f32;
        let mut n = 0usize;
        let mut collapse = 0usize;
        let mut expand = 0usize;
        let mut catastrophic = 0usize;
        let mut witness = json!(null);
        for s in samples {
            for skin in &s.skins {
                if let Some(fs) = faces.get(&skin.node_part) {
                    for f in fs {
                        let bind = f.map(|i| skin.vertices[i].bind_world);
                        let rest = area(bind);
                        if rest < 1e-11 {
                            continue;
                        }
                        let ratio = area(f.map(|i| skin.vertices[i].sampled_world)) / rest;
                        n += 1;
                        if ratio < 0.05 {
                            collapse += 1
                        }
                        if ratio > 20.0 {
                            expand += 1
                        }
                        if ratio < 0.0025 || ratio > 400.0 {
                            catastrophic += 1
                        }
                        if ratio < min {
                            min = ratio;
                            witness = json!({"time":s.time_seconds,"indices":f,"bind":bind,"areaRatio":ratio});
                        }
                        max = max.max(ratio);
                    }
                }
            }
        }
        clips.push(json!({"clip":c.name,"sampleCount":times.len(),"triangleSamples":n,"minAreaRatio":min,"maxAreaRatio":max,"collapseSamples":collapse,"expansionSamples":expand,"catastrophicSamples":catastrophic,"worstCollapse":witness}));
    }
    Ok(
        json!({"samplingHz":30,"geometryDomain":"ALL_INDEXED_TRIANGLES","clips":clips,"nativeRuntime":"NOT_TESTED"}),
    )
}
