use std::{env, fs, path::PathBuf};

use image::{ColorType, ImageBuffer, ImageFormat, Rgb};
use m2a_core::glb::{
    EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ColorClusterV1 {
    cluster: usize,
    rgb: [u8; 3],
    pixel_count: usize,
    fraction: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisV1 {
    schema_version: u32,
    source_sha256: String,
    triangle_count: usize,
    vertex_count: usize,
    primitive_count: usize,
    material_count: usize,
    image_count: usize,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    texture_width: u32,
    texture_height: u32,
    color_clusters: Vec<ColorClusterV1>,
}

fn sample_texture(image: &m2a_core::tga::TgaImageV1, uv: [f32; 2]) -> [u8; 3] {
    let channels = match image.pixel_format {
        m2a_core::tga::TgaPixelFormatV1::Rgb8 => 3,
        m2a_core::tga::TgaPixelFormatV1::Rgba8 => 4,
    };
    let wrap = |value: f32| value.rem_euclid(1.0);
    let x = (wrap(uv[0]) * (image.width - 1) as f32).round() as usize;
    let y = ((1.0 - wrap(uv[1])) * (image.height - 1) as f32).round() as usize;
    let offset = (y * image.width as usize + x) * channels;
    [
        image.pixels[offset],
        image.pixels[offset + 1],
        image.pixels[offset + 2],
    ]
}

fn render_projection(
    ingest: &m2a_core::glb::GlbIngestResult,
    texture: &m2a_core::tga::TgaImageV1,
    horizontal_axis: usize,
    vertical_axis: usize,
    depth_axis: usize,
    output: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 1400;
    const HEIGHT: u32 = 900;
    const MARGIN: f32 = 28.0;
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in ingest
        .ir
        .primitives
        .iter()
        .flat_map(|primitive| &primitive.positions)
    {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    let span_h = (max[horizontal_axis] - min[horizontal_axis]).max(1.0e-6);
    let span_v = (max[vertical_axis] - min[vertical_axis]).max(1.0e-6);
    let scale =
        ((WIDTH as f32 - MARGIN * 2.0) / span_h).min((HEIGHT as f32 - MARGIN * 2.0) / span_v);
    let occupied_width = span_h * scale;
    let occupied_height = span_v * scale;
    let offset_x = (WIDTH as f32 - occupied_width) * 0.5;
    let offset_y = (HEIGHT as f32 - occupied_height) * 0.5;
    let project = |point: [f32; 3]| {
        [
            offset_x + (point[horizontal_axis] - min[horizontal_axis]) * scale,
            HEIGHT as f32 - (offset_y + (point[vertical_axis] - min[vertical_axis]) * scale),
            point[depth_axis],
        ]
    };
    let mut pixels = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([26, 29, 31]));
    let mut depths = vec![f32::NEG_INFINITY; (WIDTH * HEIGHT) as usize];
    for primitive in &ingest.ir.primitives {
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
            let uv = if primitive.uv0.len() == primitive.positions.len() {
                triangle
                    .map(|index| primitive.uv0[index as usize])
                    .into_iter()
                    .fold([0.0f32; 2], |sum, uv| [sum[0] + uv[0], sum[1] + uv[1]])
                    .map(|value| value / 3.0)
            } else {
                [0.5, 0.5]
            };
            let source_color = sample_texture(texture, uv);
            let normal = if primitive.normals.len() == primitive.positions.len() {
                triangle
                    .map(|index| primitive.normals[index as usize])
                    .into_iter()
                    .fold([0.0f32; 3], |sum, normal| {
                        [sum[0] + normal[0], sum[1] + normal[1], sum[2] + normal[2]]
                    })
            } else {
                [0.0, 1.0, 0.0]
            };
            let shade = (0.68 + 0.32 * normal[depth_axis].abs()).clamp(0.0, 1.0);
            let color = source_color.map(|value| (value as f32 * shade).round() as u8);
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
                        pixels.put_pixel(x, y, Rgb(color));
                    }
                }
            }
            let _ = triangle_index;
        }
    }
    pixels.save_with_format(output, ImageFormat::Png)?;
    Ok(())
}

fn nearest(color: [f64; 3], centers: &[[f64; 3]]) -> usize {
    centers
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let distance = |center: &&[f64; 3]| {
                (0..3)
                    .map(|axis| (color[axis] - center[axis]).powi(2))
                    .sum::<f64>()
            };
            distance(left).total_cmp(&distance(right))
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source =
        PathBuf::from(args.next().ok_or(
            "usage: analyze_material_separation_source_v2 <source.glb> <output-directory>",
        )?);
    let output =
        PathBuf::from(args.next().ok_or(
            "usage: analyze_material_separation_source_v2 <source.glb> <output-directory>",
        )?);
    if args.next().is_some() {
        return Err(
            "usage: analyze_material_separation_source_v2 <source.glb> <output-directory>".into(),
        );
    }
    let bytes = fs::read(&source)?;
    // Source-quality intake must cover the existing high-detail Creature
    // envelope. Keep every geometry, image and shared 300K triangle gate from
    // the default profile while admitting GLB containers up to the already
    // established P300K input-byte boundary.
    let limits = GlbLimits {
        max_input_bytes: 256 * 1024 * 1024,
        ..GlbLimits::default()
    };
    let ingest = ingest_glb(&bytes, &limits)?;
    let image = decode_embedded_image_to_tga_v1(
        &bytes,
        0,
        &limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )?;
    fs::create_dir_all(&output)?;
    render_projection(&ingest, &image, 0, 1, 2, output.join("source-side-xy.png"))?;
    render_projection(&ingest, &image, 0, 2, 1, output.join("source-top-xz.png"))?;
    render_projection(&ingest, &image, 2, 1, 0, output.join("source-front-zy.png"))?;
    let channels = match image.pixel_format {
        m2a_core::tga::TgaPixelFormatV1::Rgb8 => 3,
        m2a_core::tga::TgaPixelFormatV1::Rgba8 => 4,
    };
    let rgb = image
        .pixels
        .chunks_exact(channels)
        .flat_map(|pixel| pixel[..3].iter().copied())
        .collect::<Vec<_>>();
    image::save_buffer_with_format(
        output.join("source-texture.png"),
        &rgb,
        image.width,
        image.height,
        ColorType::Rgb8,
        ImageFormat::Png,
    )?;

    let pixels = rgb
        .chunks_exact(3)
        .step_by((rgb.len() / 3 / 200_000).max(1))
        .map(|pixel| [pixel[0] as f64, pixel[1] as f64, pixel[2] as f64])
        .collect::<Vec<_>>();
    let seeds = [
        [20.0, 20.0, 20.0],
        [55.0, 40.0, 28.0],
        [85.0, 60.0, 38.0],
        [120.0, 86.0, 56.0],
        [160.0, 125.0, 88.0],
        [200.0, 180.0, 145.0],
        [90.0, 90.0, 85.0],
        [155.0, 155.0, 145.0],
    ];
    let mut centers = seeds;
    for _ in 0..24 {
        let mut sums = [[0.0; 3]; 8];
        let mut counts = [0usize; 8];
        for pixel in &pixels {
            let cluster = nearest(*pixel, &centers);
            counts[cluster] += 1;
            for axis in 0..3 {
                sums[cluster][axis] += pixel[axis];
            }
        }
        for cluster in 0..8 {
            if counts[cluster] == 0 {
                continue;
            }
            for axis in 0..3 {
                centers[cluster][axis] = sums[cluster][axis] / counts[cluster] as f64;
            }
        }
    }
    let mut counts = [0usize; 8];
    for pixel in &pixels {
        counts[nearest(*pixel, &centers)] += 1;
    }
    let mut clusters = centers
        .iter()
        .enumerate()
        .map(|(cluster, center)| ColorClusterV1 {
            cluster,
            rgb: center.map(|value| value.round().clamp(0.0, 255.0) as u8),
            pixel_count: counts[cluster],
            fraction: counts[cluster] as f64 / pixels.len().max(1) as f64,
        })
        .collect::<Vec<_>>();
    clusters.sort_by_key(|cluster| std::cmp::Reverse(cluster.pixel_count));

    let mut bounds_min = [f32::INFINITY; 3];
    let mut bounds_max = [f32::NEG_INFINITY; 3];
    for point in ingest
        .ir
        .primitives
        .iter()
        .flat_map(|primitive| &primitive.positions)
    {
        for axis in 0..3 {
            bounds_min[axis] = bounds_min[axis].min(point[axis]);
            bounds_max[axis] = bounds_max[axis].max(point[axis]);
        }
    }
    let analysis = AnalysisV1 {
        schema_version: 1,
        source_sha256: ingest.ir.source.sha256.clone(),
        triangle_count: ingest.report.statistics.triangle_count,
        vertex_count: ingest.report.statistics.vertex_count,
        primitive_count: ingest.ir.primitives.len(),
        material_count: ingest.ir.materials.len(),
        image_count: ingest.ir.images.len(),
        bounds_min,
        bounds_max,
        texture_width: image.width,
        texture_height: image.height,
        color_clusters: clusters,
    };
    let json = serde_json::to_vec_pretty(&analysis)?;
    fs::write(output.join("analysis.json"), &json)?;
    println!("{}", String::from_utf8(json)?);
    Ok(())
}
