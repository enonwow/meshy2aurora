use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use image::{ImageBuffer, ImageFormat, Rgb, RgbImage};
use m2a_core::{
    glb::{GlbLimits, IrPrimitive, ingest_glb},
    model_components::inspect_model_components_v1,
    model_material_separation::{ModelMaterialSeparationDocumentV2, resolve_model_materials_v2},
    model_material_uv_projection::{
        MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1, ModelMaterialUvProjectionDocumentV1,
        ModelMaterialUvProjectionModeV1, model_material_uv_projection_hash_v1,
        project_primitive_uv0_v1,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewReport {
    schema_version: u32,
    status: String,
    source_sha256: String,
    separation_sha256: String,
    source_triangle_count: u32,
    output_triangle_count: u32,
    geometry_cleanup: bool,
    source_uv0_preserved: bool,
    source_uv0_preserved_for_unprojected_materials: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    material_uv_projection_sha256: Option<String>,
    projected_triangle_count: u32,
    preview_sha256: BTreeMap<String, String>,
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn sample_texture(image: &RgbImage, uv: [f32; 2]) -> [u8; 3] {
    let wrap = |value: f32| value.rem_euclid(1.0);
    let x = wrap(uv[0]) * (image.width() - 1) as f32;
    // glTF defines (0, 0) at the upper-left image pixel. `image` stores PNG
    // scanlines in that same logical order, so V must not be flipped here.
    let y = wrap(uv[1]) * (image.height() - 1) as f32;
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let samples = [
        (image.get_pixel(x0, y0).0, (1.0 - tx) * (1.0 - ty)),
        (image.get_pixel(x1, y0).0, tx * (1.0 - ty)),
        (image.get_pixel(x0, y1).0, (1.0 - tx) * ty),
        (image.get_pixel(x1, y1).0, tx * ty),
    ];
    std::array::from_fn(|channel| {
        samples
            .iter()
            .map(|(color, weight)| color[channel] as f32 * weight)
            .sum::<f32>()
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

#[allow(clippy::too_many_arguments)]
fn render_projection(
    primitive: &IrPrimitive,
    material_slot_by_triangle: &[u32],
    projected_uv0_by_triangle: &[Option<[[f32; 2]; 3]>],
    texture_by_slot: &BTreeMap<u32, RgbImage>,
    horizontal_axis: usize,
    vertical_axis: usize,
    depth_axis: usize,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 1400;
    const HEIGHT: u32 = 900;
    const MARGIN: f32 = 28.0;
    let min = primitive.bounds_min;
    let max = primitive.bounds_max;
    let span_h = (max[horizontal_axis] - min[horizontal_axis]).max(1.0e-6);
    let span_v = (max[vertical_axis] - min[vertical_axis]).max(1.0e-6);
    let scale =
        ((WIDTH as f32 - MARGIN * 2.0) / span_h).min((HEIGHT as f32 - MARGIN * 2.0) / span_v);
    let offset_x = (WIDTH as f32 - span_h * scale) * 0.5;
    let offset_y = (HEIGHT as f32 - span_v * scale) * 0.5;
    let project = |point: [f32; 3]| {
        [
            offset_x + (point[horizontal_axis] - min[horizontal_axis]) * scale,
            HEIGHT as f32 - (offset_y + (point[vertical_axis] - min[vertical_axis]) * scale),
            point[depth_axis],
        ]
    };
    let mut pixels = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([26, 29, 31]));
    let mut depths = vec![f32::NEG_INFINITY; (WIDTH * HEIGHT) as usize];
    for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
        let triangle = [triangle[0], triangle[1], triangle[2]];
        let points = triangle.map(|index| primitive.positions[index as usize]);
        let projected = points.map(project);
        let min_x = projected
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .clamp(0.0, (WIDTH - 1) as f32) as u32;
        let max_x = projected
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .clamp(0.0, (WIDTH - 1) as f32) as u32;
        let min_y = projected
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .clamp(0.0, (HEIGHT - 1) as f32) as u32;
        let max_y = projected
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .clamp(0.0, (HEIGHT - 1) as f32) as u32;
        let edge = |a: [f32; 3], b: [f32; 3], x: f32, y: f32| {
            (x - a[0]) * (b[1] - a[1]) - (y - a[1]) * (b[0] - a[0])
        };
        let area = edge(projected[0], projected[1], projected[2][0], projected[2][1]);
        if area.abs() < 1.0e-8 {
            continue;
        }
        let uvs = if let Some(projected) = projected_uv0_by_triangle[triangle_index] {
            projected
        } else if primitive.uv0.len() == primitive.positions.len() {
            triangle.map(|index| primitive.uv0[index as usize])
        } else {
            [[0.5, 0.5]; 3]
        };
        let texture = &texture_by_slot[&material_slot_by_triangle[triangle_index]];
        let edge_a = [
            points[1][0] - points[0][0],
            points[1][1] - points[0][1],
            points[1][2] - points[0][2],
        ];
        let edge_b = [
            points[2][0] - points[0][0],
            points[2][1] - points[0][1],
            points[2][2] - points[0][2],
        ];
        let normal = [
            edge_a[1] * edge_b[2] - edge_a[2] * edge_b[1],
            edge_a[2] * edge_b[0] - edge_a[0] * edge_b[2],
            edge_a[0] * edge_b[1] - edge_a[1] * edge_b[0],
        ];
        let normal_length =
            (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let face_normal = if normal_length > 1.0e-12 {
            normal.map(|value| value / normal_length)
        } else {
            [0.0, 0.0, 1.0]
        };
        let vertex_normals = if primitive.normals.len() == primitive.positions.len() {
            triangle.map(|index| primitive.normals[index as usize])
        } else {
            [face_normal; 3]
        };
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let sample_x = x as f32 + 0.5;
                let sample_y = y as f32 + 0.5;
                let w0 = edge(projected[1], projected[2], sample_x, sample_y) / area;
                let w1 = edge(projected[2], projected[0], sample_x, sample_y) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1.0e-5 || w1 < -1.0e-5 || w2 < -1.0e-5 {
                    continue;
                }
                let depth = w0 * projected[0][2] + w1 * projected[1][2] + w2 * projected[2][2];
                let offset = (y * WIDTH + x) as usize;
                if depth >= depths[offset] {
                    depths[offset] = depth;
                    let uv = [
                        w0 * uvs[0][0] + w1 * uvs[1][0] + w2 * uvs[2][0],
                        w0 * uvs[0][1] + w1 * uvs[1][1] + w2 * uvs[2][1],
                    ];
                    let source_color = sample_texture(texture, uv);
                    let interpolated_normal: [f32; 3] = std::array::from_fn(|axis| {
                        w0 * vertex_normals[0][axis]
                            + w1 * vertex_normals[1][axis]
                            + w2 * vertex_normals[2][axis]
                    });
                    let interpolated_length = interpolated_normal
                        .iter()
                        .map(|value| value * value)
                        .sum::<f32>()
                        .sqrt();
                    let facing = if interpolated_length > 1.0e-12 {
                        (interpolated_normal[depth_axis] / interpolated_length).abs()
                    } else {
                        face_normal[depth_axis].abs()
                    };
                    let shade = 0.78 + facing * 0.22;
                    let color = Rgb(source_color.map(|value| (value as f32 * shade).round() as u8));
                    pixels.put_pixel(x, y, color);
                }
            }
        }
    }
    pixels.save_with_format(output, ImageFormat::Png)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source_path = PathBuf::from(args.next().ok_or(
        "usage: render_tlc_ship_material_preview_v2 <source.glb> <recipe.json> <normalized-texture-directory> <output-directory>",
    )?);
    let recipe_path = PathBuf::from(args.next().ok_or(
        "usage: render_tlc_ship_material_preview_v2 <source.glb> <recipe.json> <normalized-texture-directory> <output-directory>",
    )?);
    let texture_directory = PathBuf::from(args.next().ok_or(
        "usage: render_tlc_ship_material_preview_v2 <source.glb> <recipe.json> <normalized-texture-directory> <output-directory>",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: render_tlc_ship_material_preview_v2 <source.glb> <recipe.json> <normalized-texture-directory> <output-directory> [material-uv-projection.json]",
    )?);
    let material_uv_projection_path = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err(
            "usage: render_tlc_ship_material_preview_v2 <source.glb> <recipe.json> <normalized-texture-directory> <output-directory> [material-uv-projection.json]".into(),
        );
    }
    if output.exists() {
        return Err(format!("output already exists: {}", output.display()).into());
    }
    fs::create_dir_all(&output)?;
    let source = fs::read(&source_path)?;
    let limits = GlbLimits::default();
    let ingest = ingest_glb(&source, &limits)?;
    if ingest.ir.source.sha256 != SOURCE_SHA256 || ingest.ir.primitives.len() != 1 {
        return Err("exact TLC ship source identity mismatch".into());
    }
    let recipe: ModelMaterialSeparationDocumentV2 =
        serde_json::from_slice(&fs::read(&recipe_path)?)?;
    let materials = resolve_model_materials_v2(&ingest.ir, &recipe)?;
    let inventory = inspect_model_components_v1(&ingest.ir)?;
    let key = inventory
        .components
        .first()
        .ok_or("source has no components")?
        .key;
    let primitive = &ingest.ir.primitives[0];
    let material_slot_by_triangle = (0..primitive.indices.len() / 3)
        .map(|triangle| {
            materials
                .material_slot_for_triangle(
                    key.scene_id,
                    key.node_id,
                    key.primitive_id,
                    triangle as u32,
                )
                .ok_or_else(|| format!("unassigned triangle: {triangle}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut projected_uv0_by_triangle = vec![None; primitive.indices.len() / 3];
    let mut projected_triangle_count = 0u32;
    let material_uv_projection_sha256 = material_uv_projection_path
        .as_ref()
        .map(|path| -> Result<String, Box<dyn std::error::Error>> {
            let document: ModelMaterialUvProjectionDocumentV1 =
                serde_json::from_slice(&fs::read(path)?)?;
            if document.schema_version != MODEL_MATERIAL_UV_PROJECTION_SCHEMA_VERSION_V1
                || document.source_sha256 != ingest.ir.source.sha256
                || document.separation_sha256 != materials.report.separation_sha256
            {
                return Err("material UV projection identity mismatch".into());
            }
            for rule in &document.rules {
                let slot = materials
                    .report
                    .material_slots
                    .iter()
                    .find(|slot| slot.authored_material_id == rule.authored_material_id)
                    .ok_or_else(|| {
                        format!(
                            "projected material is absent from separation: {}",
                            rule.authored_material_id
                        )
                    })?
                    .material_slot;
                if rule.mode != ModelMaterialUvProjectionModeV1::Source {
                    let projected = project_primitive_uv0_v1(primitive, rule)?;
                    for (triangle, material_slot) in material_slot_by_triangle.iter().enumerate() {
                        if *material_slot == slot {
                            projected_uv0_by_triangle[triangle] = Some(projected[triangle]);
                            projected_triangle_count += 1;
                        }
                    }
                }
            }
            Ok(model_material_uv_projection_hash_v1(&document)?)
        })
        .transpose()?;
    let mut texture_by_slot = BTreeMap::new();
    for slot in &materials.report.material_slots {
        let texture =
            image::open(texture_directory.join(format!("{}.png", slot.authored_material_id)))?
                .into_rgb8();
        if texture.width() != 1024 || texture.height() != 1024 {
            return Err(format!(
                "normalized texture dimensions mismatch: {}",
                slot.authored_material_id
            )
            .into());
        }
        texture_by_slot.insert(slot.material_slot, texture);
    }
    let files = [
        ("retextured-side-xy.png", 0, 1, 2),
        ("retextured-top-xz.png", 0, 2, 1),
        ("retextured-front-zy.png", 2, 1, 0),
    ];
    for (file, horizontal, vertical, depth) in files {
        render_projection(
            primitive,
            &material_slot_by_triangle,
            &projected_uv0_by_triangle,
            &texture_by_slot,
            horizontal,
            vertical,
            depth,
            &output.join(file),
        )?;
    }
    let preview_sha256 = files
        .into_iter()
        .map(|(file, _, _, _)| {
            let payload = fs::read(output.join(file))?;
            Ok((file.to_owned(), sha256(&payload)))
        })
        .collect::<Result<BTreeMap<_, _>, std::io::Error>>()?;
    let report = PreviewReport {
        schema_version: 1,
        status: "offline_retextured_preview_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        separation_sha256: materials.report.separation_sha256,
        source_triangle_count: materials.report.source_triangle_count,
        output_triangle_count: materials.report.output_triangle_count,
        geometry_cleanup: false,
        source_uv0_preserved: projected_triangle_count == 0,
        source_uv0_preserved_for_unprojected_materials: true,
        material_uv_projection_sha256,
        projected_triangle_count,
        preview_sha256,
    };
    fs::write(
        output.join("preview-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gltf_zero_v_samples_the_top_image_scanline() {
        let image = RgbImage::from_raw(1, 2, vec![255, 0, 0, 0, 0, 255]).unwrap();
        assert_eq!(sample_texture(&image, [0.0, 0.0]), [255, 0, 0]);
        assert_eq!(sample_texture(&image, [0.0, 0.999]), [0, 0, 255]);
    }
}
