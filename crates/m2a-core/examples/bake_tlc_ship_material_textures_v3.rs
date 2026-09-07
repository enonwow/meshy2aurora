use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use image::{ImageFormat, Rgb, RgbImage, imageops};
use m2a_core::{
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    model_components::inspect_model_components_v1,
    model_material_separation::{
        ModelMaterialSeparationDocumentV2, model_material_separation_hash_v2,
        resolve_model_materials_v2,
    },
    tga::TgaPixelFormatV1,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const WOOD_SOURCE_SHA256: &str = "3b0669f683e4bf10f5a55a381cfa9669a7b8dfd921901829daa3b35acc2bbdec";
const WOOD_SOURCE_URL: &str = "https://polyhaven.com/a/wood_planks";
const MATERIALS: [&str; 5] = ["wood", "rope", "sail", "cloth", "metal"];

#[derive(Clone, Debug)]
struct ComponentProjection {
    min: [f32; 3],
    max: [f32; 3],
    longitudinal_axis: usize,
    transverse_axis: usize,
    strip_index: usize,
    phase: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MaterialReport {
    authored_material_id: String,
    strategy: String,
    output_path: PathBuf,
    output_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BakeReport {
    schema_version: u32,
    status: String,
    source_sha256: String,
    separation_sha256: String,
    source_triangle_count: u32,
    output_triangle_count: u32,
    source_uv0_preserved: bool,
    geometry_cleanup: bool,
    wood_source_url: String,
    wood_source_license: String,
    wood_source_sha256: String,
    connected_component_count: u32,
    wood_triangle_count: u32,
    projected_pixel_count: u32,
    overlapping_projected_pixel_writes: u64,
    fallback_pixel_count: u32,
    materials: Vec<MaterialReport>,
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

fn clamp_u8(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn find(parent: &mut [usize], mut value: usize) -> usize {
    let mut root = value;
    while parent[root] != root {
        root = parent[root];
    }
    while parent[value] != value {
        let next = parent[value];
        parent[value] = root;
        value = next;
    }
    root
}

fn union(parent: &mut [usize], left: usize, right: usize) {
    let left = find(parent, left);
    let right = find(parent, right);
    if left != right {
        let (root, child) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        parent[child] = root;
    }
}

fn projection_axes(min: [f32; 3], max: [f32; 3]) -> (usize, usize) {
    let mut axes = [0usize, 1, 2];
    axes.sort_by(|left, right| {
        let left_span = max[*left] - min[*left];
        let right_span = max[*right] - min[*right];
        right_span
            .partial_cmp(&left_span)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.cmp(right))
    });
    (axes[0], axes[1])
}

fn connected_component_projection_by_triangle(
    primitive: &m2a_core::glb::IrPrimitive,
) -> Result<(Vec<usize>, Vec<ComponentProjection>), Box<dyn std::error::Error>> {
    let triangle_count = primitive.indices.len() / 3;
    let mut parent = (0..triangle_count).collect::<Vec<_>>();
    let mut first_triangle_by_vertex = BTreeMap::<u32, usize>::new();
    for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
        for vertex in triangle {
            if let Some(first_triangle) = first_triangle_by_vertex.get(vertex) {
                union(&mut parent, triangle_index, *first_triangle);
            } else {
                first_triangle_by_vertex.insert(*vertex, triangle_index);
            }
        }
    }

    let mut component_index_by_root = BTreeMap::<usize, usize>::new();
    let mut component_by_triangle = Vec::with_capacity(triangle_count);
    for triangle_index in 0..triangle_count {
        let root = find(&mut parent, triangle_index);
        let next = component_index_by_root.len();
        let component = *component_index_by_root.entry(root).or_insert(next);
        component_by_triangle.push(component);
    }

    let component_count = component_index_by_root.len();
    let mut mins = vec![[f32::INFINITY; 3]; component_count];
    let mut maxs = vec![[f32::NEG_INFINITY; 3]; component_count];
    for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
        let component = component_by_triangle[triangle_index];
        for vertex in triangle {
            let point = primitive.positions[*vertex as usize];
            for axis in 0..3 {
                mins[component][axis] = mins[component][axis].min(point[axis]);
                maxs[component][axis] = maxs[component][axis].max(point[axis]);
            }
        }
    }

    let projections = mins
        .into_iter()
        .zip(maxs)
        .enumerate()
        .map(|(component, (min, max))| {
            let (longitudinal_axis, transverse_axis) = projection_axes(min, max);
            let hash = (component as u32).wrapping_mul(0x9e37_79b9);
            ComponentProjection {
                min,
                max,
                longitudinal_axis,
                transverse_axis,
                strip_index: (hash as usize) % WOOD_PLANK_STRIPS.len(),
                phase: ((hash >> 8) & 0xffff) as f32 / 65_535.0,
            }
        })
        .collect();
    Ok((component_by_triangle, projections))
}

const WOOD_PLANK_STRIPS: [(f32, f32); 9] = [
    (0.015, 0.125),
    (0.150, 0.235),
    (0.270, 0.355),
    (0.390, 0.465),
    (0.495, 0.565),
    (0.600, 0.685),
    (0.720, 0.800),
    (0.835, 0.905),
    (0.935, 0.985),
];

fn projected_wood_uv(point: [f32; 3], projection: &ComponentProjection) -> [f32; 2] {
    let longitudinal_span = (projection.max[projection.longitudinal_axis]
        - projection.min[projection.longitudinal_axis])
        .max(1.0e-6);
    let transverse_span = (projection.max[projection.transverse_axis]
        - projection.min[projection.transverse_axis])
        .max(1.0e-6);
    let longitudinal = (point[projection.longitudinal_axis]
        - projection.min[projection.longitudinal_axis])
        / longitudinal_span;
    let transverse = (point[projection.transverse_axis]
        - projection.min[projection.transverse_axis])
        / transverse_span;
    let (strip_start, strip_end) = WOOD_PLANK_STRIPS[projection.strip_index];
    [
        (longitudinal + projection.phase).rem_euclid(1.0),
        strip_start + transverse.clamp(0.0, 1.0) * (strip_end - strip_start),
    ]
}

fn sample_bilinear(image: &RgbImage, uv: [f32; 2]) -> Rgb<u8> {
    let x = uv[0].rem_euclid(1.0) * (image.width() - 1) as f32;
    let y = uv[1].clamp(0.0, 1.0) * (image.height() - 1) as f32;
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let samples = [
        (*image.get_pixel(x0, y0), (1.0 - tx) * (1.0 - ty)),
        (*image.get_pixel(x1, y0), tx * (1.0 - ty)),
        (*image.get_pixel(x0, y1), (1.0 - tx) * ty),
        (*image.get_pixel(x1, y1), tx * ty),
    ];
    Rgb(std::array::from_fn(|channel| {
        clamp_u8(
            samples
                .iter()
                .map(|(color, weight)| color[channel] as f32 * weight)
                .sum(),
        )
    }))
}

fn shade_wood(reference: Rgb<u8>, source_luma: f32) -> Rgb<u8> {
    let factor = 0.48 + 0.30 * (source_luma / 255.0).clamp(0.0, 1.0);
    Rgb(reference.0.map(|value| clamp_u8(value as f32 * factor)))
}

fn edge(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> f32 {
    (point[0] - a[0]) * (b[1] - a[1]) - (point[1] - a[1]) * (b[0] - a[0])
}

fn dilate_projected_pixels(image: &mut RgbImage, mask: &mut [bool], passes: usize) {
    let width = image.width();
    let height = image.height();
    for _ in 0..passes {
        let previous = image.clone();
        let previous_mask = mask.to_vec();
        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                if previous_mask[index] {
                    continue;
                }
                let mut sum = [0u32; 3];
                let mut count = 0u32;
                for offset_y in -1i32..=1 {
                    for offset_x in -1i32..=1 {
                        if offset_x == 0 && offset_y == 0 {
                            continue;
                        }
                        let nx = x as i32 + offset_x;
                        let ny = y as i32 + offset_y;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let neighbor = (ny as u32 * width + nx as u32) as usize;
                        if previous_mask[neighbor] {
                            let color = previous.get_pixel(nx as u32, ny as u32).0;
                            for channel in 0..3 {
                                sum[channel] += u32::from(color[channel]);
                            }
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    image.put_pixel(x, y, Rgb(sum.map(|value| (value / count) as u8)));
                    mask[index] = true;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bake_wood(
    primitive: &m2a_core::glb::IrPrimitive,
    material_slot_by_triangle: &[u32],
    wood_slot: u32,
    component_by_triangle: &[usize],
    projections: &[ComponentProjection],
    reference: &RgbImage,
    source_shading: &RgbImage,
    fallback: &RgbImage,
) -> Result<(RgbImage, u32, u32, u64), Box<dyn std::error::Error>> {
    let width = fallback.width();
    let height = fallback.height();
    let pixel_count = (width * height) as usize;
    let mut sums = vec![[0u64; 3]; pixel_count];
    let mut counts = vec![0u32; pixel_count];
    let mut wood_triangle_count = 0u32;
    let mut overlap_writes = 0u64;

    for (triangle_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
        if material_slot_by_triangle[triangle_index] != wood_slot {
            continue;
        }
        wood_triangle_count += 1;
        let indices = [triangle[0], triangle[1], triangle[2]];
        let points = indices.map(|index| primitive.positions[index as usize]);
        let uvs = indices.map(|index| primitive.uv0[index as usize]);
        let texture_points =
            uvs.map(|uv| [uv[0] * (width - 1) as f32, uv[1] * (height - 1) as f32]);
        let area = edge(texture_points[0], texture_points[1], texture_points[2]);
        let projection = &projections[component_by_triangle[triangle_index]];
        let mut wrote_triangle = false;

        if area.abs() >= 1.0e-8 {
            let min_x = texture_points
                .iter()
                .map(|point| point[0])
                .fold(f32::INFINITY, f32::min)
                .floor()
                .clamp(0.0, (width - 1) as f32) as u32;
            let max_x = texture_points
                .iter()
                .map(|point| point[0])
                .fold(f32::NEG_INFINITY, f32::max)
                .ceil()
                .clamp(0.0, (width - 1) as f32) as u32;
            let min_y = texture_points
                .iter()
                .map(|point| point[1])
                .fold(f32::INFINITY, f32::min)
                .floor()
                .clamp(0.0, (height - 1) as f32) as u32;
            let max_y = texture_points
                .iter()
                .map(|point| point[1])
                .fold(f32::NEG_INFINITY, f32::max)
                .ceil()
                .clamp(0.0, (height - 1) as f32) as u32;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let sample = [x as f32 + 0.5, y as f32 + 0.5];
                    let w0 = edge(texture_points[1], texture_points[2], sample) / area;
                    let w1 = edge(texture_points[2], texture_points[0], sample) / area;
                    let w2 = 1.0 - w0 - w1;
                    if w0 < -1.0e-5 || w1 < -1.0e-5 || w2 < -1.0e-5 {
                        continue;
                    }
                    let point = std::array::from_fn(|axis| {
                        w0 * points[0][axis] + w1 * points[1][axis] + w2 * points[2][axis]
                    });
                    let wood = sample_bilinear(reference, projected_wood_uv(point, projection));
                    let source = source_shading.get_pixel(x, y).0;
                    let luma = 0.2126 * source[0] as f32
                        + 0.7152 * source[1] as f32
                        + 0.0722 * source[2] as f32;
                    let color = shade_wood(wood, luma);
                    let pixel = (y * width + x) as usize;
                    if counts[pixel] > 0 {
                        overlap_writes += 1;
                    }
                    for channel in 0..3 {
                        sums[pixel][channel] += u64::from(color[channel]);
                    }
                    counts[pixel] += 1;
                    wrote_triangle = true;
                }
            }
        }

        // Many Meshy UV islands are sub-pixel even at 2048. Seed the nearest
        // centroid texel so dilation can still provide a deterministic sample.
        if !wrote_triangle {
            let uv = [
                (uvs[0][0] + uvs[1][0] + uvs[2][0]) / 3.0,
                (uvs[0][1] + uvs[1][1] + uvs[2][1]) / 3.0,
            ];
            let x = (uv[0].clamp(0.0, 1.0) * (width - 1) as f32).round() as u32;
            let y = (uv[1].clamp(0.0, 1.0) * (height - 1) as f32).round() as u32;
            let point = std::array::from_fn(|axis| {
                (points[0][axis] + points[1][axis] + points[2][axis]) / 3.0
            });
            let wood = sample_bilinear(reference, projected_wood_uv(point, projection));
            let source = source_shading.get_pixel(x, y).0;
            let luma =
                0.2126 * source[0] as f32 + 0.7152 * source[1] as f32 + 0.0722 * source[2] as f32;
            let color = shade_wood(wood, luma);
            let pixel = (y * width + x) as usize;
            if counts[pixel] > 0 {
                overlap_writes += 1;
            }
            for channel in 0..3 {
                sums[pixel][channel] += u64::from(color[channel]);
            }
            counts[pixel] += 1;
        }
    }

    let mut output = fallback.clone();
    let mut mask = vec![false; pixel_count];
    let mut projected_pixels = 0u32;
    for y in 0..height {
        for x in 0..width {
            let pixel = (y * width + x) as usize;
            if counts[pixel] == 0 {
                continue;
            }
            output.put_pixel(
                x,
                y,
                Rgb(std::array::from_fn(|channel| {
                    (sums[pixel][channel] / u64::from(counts[pixel])) as u8
                })),
            );
            mask[pixel] = true;
            projected_pixels += 1;
        }
    }
    dilate_projected_pixels(&mut output, &mut mask, 2);
    Ok((
        output,
        wood_triangle_count,
        projected_pixels,
        overlap_writes,
    ))
}

fn load_rgb(path: &Path) -> Result<RgbImage, Box<dyn std::error::Error>> {
    Ok(image::open(path)?.into_rgb8())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source_path = PathBuf::from(args.next().ok_or(
        "usage: bake_tlc_ship_material_textures_v3 <source.glb> <recipe.json> <wood-diffuse> <fallback-material-directory> <output-directory>",
    )?);
    let recipe_path = PathBuf::from(args.next().ok_or(
        "usage: bake_tlc_ship_material_textures_v3 <source.glb> <recipe.json> <wood-diffuse> <fallback-material-directory> <output-directory>",
    )?);
    let wood_source_path = PathBuf::from(args.next().ok_or(
        "usage: bake_tlc_ship_material_textures_v3 <source.glb> <recipe.json> <wood-diffuse> <fallback-material-directory> <output-directory>",
    )?);
    let fallback_directory = PathBuf::from(args.next().ok_or(
        "usage: bake_tlc_ship_material_textures_v3 <source.glb> <recipe.json> <wood-diffuse> <fallback-material-directory> <output-directory>",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: bake_tlc_ship_material_textures_v3 <source.glb> <recipe.json> <wood-diffuse> <fallback-material-directory> <output-directory>",
    )?);
    if args.next().is_some() {
        return Err("usage: bake_tlc_ship_material_textures_v3 <source.glb> <recipe.json> <wood-diffuse> <fallback-material-directory> <output-directory>".into());
    }
    if output.exists() {
        return Err(format!("output already exists: {}", output.display()).into());
    }

    let source = fs::read(&source_path)?;
    let limits = GlbLimits::default();
    let ingest = ingest_glb(&source, &limits)?;
    if ingest.ir.source.sha256 != SOURCE_SHA256 || ingest.ir.primitives.len() != 1 {
        return Err("exact TLC ship source identity mismatch".into());
    }
    let primitive = &ingest.ir.primitives[0];
    if primitive.uv0.len() != primitive.positions.len() {
        return Err("TLC ship requires exact source UV0 for texture baking".into());
    }

    let recipe: ModelMaterialSeparationDocumentV2 =
        serde_json::from_slice(&fs::read(&recipe_path)?)?;
    let resolved = resolve_model_materials_v2(&ingest.ir, &recipe)?;
    let inventory = inspect_model_components_v1(&ingest.ir)?;
    let key = inventory
        .components
        .first()
        .ok_or("source has no components")?
        .key;
    let material_slot_by_triangle = (0..primitive.indices.len() / 3)
        .map(|triangle| {
            resolved
                .material_slot_for_triangle(
                    key.scene_id,
                    key.node_id,
                    key.primitive_id,
                    triangle as u32,
                )
                .ok_or_else(|| format!("unassigned source triangle: {triangle}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let wood_slot = resolved
        .report
        .material_slots
        .iter()
        .find(|slot| slot.authored_material_id == "wood")
        .ok_or("recipe has no wood material")?
        .material_slot;

    let wood_source_payload = fs::read(&wood_source_path)?;
    if sha256(&wood_source_payload) != WOOD_SOURCE_SHA256 {
        return Err("exact Poly Haven wood diffuse identity mismatch".into());
    }
    let wood_reference = load_rgb(&wood_source_path)?;
    let fallback_wood = load_rgb(&fallback_directory.join("wood.png"))?;
    if fallback_wood.width() != 2048 || fallback_wood.height() != 2048 {
        return Err("fallback material atlas must be 2048x2048".into());
    }

    let decoded = decode_embedded_image_to_tga_v1(
        &source,
        0,
        &limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )?;
    let channels = match decoded.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3,
        TgaPixelFormatV1::Rgba8 => 4,
    };
    let source_image = RgbImage::from_fn(decoded.width, decoded.height, |x, y| {
        let offset = ((y * decoded.width + x) as usize) * channels;
        Rgb([
            decoded.pixels[offset],
            decoded.pixels[offset + 1],
            decoded.pixels[offset + 2],
        ])
    });
    let source_shading = imageops::blur(&source_image, 8.0);
    let (component_by_triangle, projections) =
        connected_component_projection_by_triangle(primitive)?;
    let (wood, wood_triangle_count, projected_pixel_count, overlap_writes) = bake_wood(
        primitive,
        &material_slot_by_triangle,
        wood_slot,
        &component_by_triangle,
        &projections,
        &wood_reference,
        &source_shading,
        &fallback_wood,
    )?;

    fs::create_dir_all(&output)?;
    let mut materials = Vec::new();
    for material in MATERIALS {
        let path = output.join(format!("{material}.png"));
        if material == "wood" {
            wood.save_with_format(&path, ImageFormat::Png)?;
        } else {
            fs::copy(fallback_directory.join(format!("{material}.png")), &path)?;
        }
        materials.push(MaterialReport {
            authored_material_id: material.to_owned(),
            strategy: if material == "wood" {
                "CC0 directional wood baked through exact separated Wood faces into source UV0 with source low-frequency shading"
                    .to_owned()
            } else {
                "existing independent material-specific source-atlas grade retained"
                    .to_owned()
            },
            output_path: path.clone(),
            output_sha256: sha256(&fs::read(path)?),
        });
    }

    let report = BakeReport {
        schema_version: 1,
        status: "offline_material_specific_texture_bake_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        separation_sha256: model_material_separation_hash_v2(&recipe)?,
        source_triangle_count: resolved.report.source_triangle_count,
        output_triangle_count: resolved.report.output_triangle_count,
        source_uv0_preserved: true,
        geometry_cleanup: false,
        wood_source_url: WOOD_SOURCE_URL.to_owned(),
        wood_source_license: "CC0".to_owned(),
        wood_source_sha256: WOOD_SOURCE_SHA256.to_owned(),
        connected_component_count: projections.len() as u32,
        wood_triangle_count,
        projected_pixel_count,
        overlapping_projected_pixel_writes: overlap_writes,
        fallback_pixel_count: fallback_wood.width() * fallback_wood.height()
            - projected_pixel_count,
        materials,
    };
    fs::write(
        output.join("material-texture-bake-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_axes_follow_longest_component_extent() {
        assert_eq!(projection_axes([0.0; 3], [10.0, 2.0, 1.0]), (0, 1));
        assert_eq!(projection_axes([0.0; 3], [1.0, 8.0, 3.0]), (1, 2));
    }

    #[test]
    fn projected_wood_uv_stays_inside_selected_plank_strip() {
        let projection = ComponentProjection {
            min: [0.0; 3],
            max: [10.0, 2.0, 1.0],
            longitudinal_axis: 0,
            transverse_axis: 1,
            strip_index: 3,
            phase: 0.25,
        };
        let uv = projected_wood_uv([5.0, 1.0, 0.0], &projection);
        let strip = WOOD_PLANK_STRIPS[3];
        assert!((0.0..1.0).contains(&uv[0]));
        assert!(uv[1] >= strip.0 && uv[1] <= strip.1);
    }

    #[test]
    fn source_shading_darkens_cc0_wood_without_destroying_warm_order() {
        let dark = shade_wood(Rgb([150, 100, 60]), 40.0);
        let light = shade_wood(Rgb([150, 100, 60]), 200.0);
        assert!(dark[0] > dark[1] && dark[1] > dark[2]);
        assert!(light[0] > light[1] && light[1] > light[2]);
        assert!(light[0] > dark[0]);
    }
}
