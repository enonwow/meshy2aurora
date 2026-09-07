use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use image::{ImageBuffer, ImageFormat, Rgb};
use m2a_core::{
    glb::{
        EmbeddedImageDecodeLimitsV1, GlbLimits, IrPrimitive, decode_embedded_image_to_tga_v1,
        ingest_glb,
    },
    model_components::inspect_model_components_v1,
    model_material_separation::{
        AuthoredMaterialV1, ModelMaterialFaceAssignmentV2, ModelMaterialSeparationDocumentV2,
        SourceFaceSelectionV2, SourceTriangleRangeV2, model_material_separation_hash_v2,
        resolve_model_materials_v2,
    },
    tga::TgaImageV1,
};
use serde::Serialize;

const SOURCE_SHA256: &str = "61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278";
const SOURCE_TRIANGLES: usize = 152_574;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
enum Category {
    Wood,
    Rope,
    Sail,
    Cloth,
    Metal,
}

impl Category {
    const ALL: [Self; 5] = [Self::Wood, Self::Rope, Self::Sail, Self::Cloth, Self::Metal];

    fn id(self) -> &'static str {
        match self {
            Self::Wood => "wood",
            Self::Rope => "rope",
            Self::Sail => "sail",
            Self::Cloth => "cloth",
            Self::Metal => "metal",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Wood => "Aged ship timber",
            Self::Rope => "Hemp rope and rigging",
            Self::Sail => "Primary furled sail canvas",
            Self::Cloth => "Secondary working canvas",
            Self::Metal => "Dark forged fittings",
        }
    }

    fn preview_color(self) -> &'static str {
        match self {
            Self::Wood => "#70472A",
            Self::Rope => "#C8994B",
            Self::Sail => "#D7CFB7",
            Self::Cloth => "#9E7356",
            Self::Metal => "#48545E",
        }
    }

    fn rgb(self) -> Rgb<u8> {
        match self {
            Self::Wood => Rgb([112, 71, 42]),
            Self::Rope => Rgb([200, 153, 75]),
            Self::Sail => Rgb([215, 207, 183]),
            Self::Cloth => Rgb([158, 115, 86]),
            Self::Metal => Rgb([72, 84, 94]),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryReport {
    authored_material_id: String,
    triangle_count: usize,
    fraction: f64,
    range_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparationReport {
    schema_version: u32,
    status: String,
    source_sha256: String,
    source_triangle_count: usize,
    output_triangle_count: u32,
    output_vertex_count: u32,
    duplicated_boundary_vertex_count: u32,
    separation_sha256: String,
    category_reports: Vec<CategoryReport>,
    geometry_cleanup: bool,
    source_uv0_preserved: bool,
    classifier: ClassifierReport,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassifierReport {
    kind: String,
    scope: String,
    semantic_guarantee: bool,
    notes: Vec<String>,
}

fn sample_texture(image: &TgaImageV1, uv: [f32; 2]) -> [u8; 3] {
    let channels = match image.pixel_format {
        m2a_core::tga::TgaPixelFormatV1::Rgb8 => 3,
        m2a_core::tga::TgaPixelFormatV1::Rgba8 => 4,
    };
    let wrap = |value: f32| value.rem_euclid(1.0);
    let x = (wrap(uv[0]) * (image.width - 1) as f32).round() as usize;
    // glTF texture coordinates use an upper-left origin, matching the decoded
    // embedded image buffer. Flipping V would classify unrelated atlas texels.
    let y = (wrap(uv[1]) * (image.height - 1) as f32).round() as usize;
    let offset = (y * image.width as usize + x) * channels;
    [
        image.pixels[offset],
        image.pixels[offset + 1],
        image.pixels[offset + 2],
    ]
}

fn sub(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn length(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn classify(primitive: &IrPrimitive, texture: &TgaImageV1, triangle_index: usize) -> Category {
    let raw = &primitive.indices[triangle_index * 3..triangle_index * 3 + 3];
    let indices = [raw[0], raw[1], raw[2]];
    let points = indices.map(|index| primitive.positions[index as usize]);
    let centroid = [
        (points[0][0] + points[1][0] + points[2][0]) / 3.0,
        (points[0][1] + points[1][1] + points[2][1]) / 3.0,
        (points[0][2] + points[1][2] + points[2][2]) / 3.0,
    ];
    let face = cross(sub(points[1], points[0]), sub(points[2], points[0]));
    let double_area = length(face);
    let normal = if double_area > 1.0e-12 {
        face.map(|value| value / double_area)
    } else {
        [0.0, 1.0, 0.0]
    };
    let uv = if primitive.uv0.len() == primitive.positions.len() {
        let values = indices.map(|index| primitive.uv0[index as usize]);
        [
            (values[0][0] + values[1][0] + values[2][0]) / 3.0,
            (values[0][1] + values[1][1] + values[2][1]) / 3.0,
        ]
    } else {
        [0.5, 0.5]
    };
    let color = sample_texture(texture, uv);
    let luma = 0.2126 * color[0] as f32 + 0.7152 * color[1] as f32 + 0.0722 * color[2] as f32;
    let chroma = color.iter().max().unwrap() - color.iter().min().unwrap();

    let fore_rig = (-0.52..=-0.10).contains(&centroid[0]) && centroid[1] > -0.02;
    let aft_rig = (0.16..=0.62).contains(&centroid[0]) && centroid[1] > 0.00;
    let rig = fore_rig || aft_rig;
    let away_from_mast = centroid[2].abs() > 0.035;
    let cloth_surface = rig
        && away_from_mast
        && normal[0].abs() > 0.18
        && centroid[1] > 0.04
        && double_area > 1.0e-7;

    if rig
        && centroid[1] > 0.075
        && double_area < 0.000_045
        && luma < 92.0
        && (!cloth_surface || normal[0].abs() < 0.42)
    {
        return Category::Rope;
    }
    if cloth_surface {
        return if aft_rig {
            Category::Sail
        } else {
            Category::Cloth
        };
    }
    if centroid[1] < 0.12
        && chroma <= 24
        && (54.0..=138.0).contains(&luma)
        && double_area < 0.000_45
    {
        return Category::Metal;
    }
    Category::Wood
}

fn compact_ranges(categories: &[Category], category: Category) -> Vec<SourceTriangleRangeV2> {
    let mut ranges = Vec::new();
    let mut start = None;
    for (index, candidate) in categories.iter().copied().enumerate() {
        if candidate == category {
            start.get_or_insert(index);
        } else if let Some(start_index) = start.take() {
            ranges.push(SourceTriangleRangeV2 {
                start_triangle: start_index as u32,
                triangle_count: (index - start_index) as u32,
            });
        }
    }
    if let Some(start_index) = start {
        ranges.push(SourceTriangleRangeV2 {
            start_triangle: start_index as u32,
            triangle_count: (categories.len() - start_index) as u32,
        });
    }
    ranges
}

fn render_projection(
    primitive: &IrPrimitive,
    categories: &[Category],
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
        let color = categories[triangle_index].rgb();
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
    let source =
        PathBuf::from(args.next().ok_or(
            "usage: prepare_tlc_ship_material_separation_v2 <source.glb> <output-directory>",
        )?);
    let output =
        PathBuf::from(args.next().ok_or(
            "usage: prepare_tlc_ship_material_separation_v2 <source.glb> <output-directory>",
        )?);
    if args.next().is_some() {
        return Err(
            "usage: prepare_tlc_ship_material_separation_v2 <source.glb> <output-directory>".into(),
        );
    }
    if output.exists() {
        return Err(format!("output already exists: {}", output.display()).into());
    }
    let source_bytes = fs::read(&source)?;
    let limits = GlbLimits::default();
    let ingest = ingest_glb(&source_bytes, &limits)?;
    if ingest.ir.source.sha256 != SOURCE_SHA256 {
        return Err(format!(
            "source hash mismatch: expected {SOURCE_SHA256}, got {}",
            ingest.ir.source.sha256
        )
        .into());
    }
    if ingest.ir.primitives.len() != 1
        || ingest.ir.primitives[0].indices.len() / 3 != SOURCE_TRIANGLES
    {
        return Err("exact source topology mismatch".into());
    }
    let inventory = inspect_model_components_v1(&ingest.ir)?;
    let first = inventory
        .components
        .first()
        .ok_or("source has no connected components")?
        .key;
    if inventory.components.iter().any(|component| {
        component.key.scene_id != first.scene_id
            || component.key.node_id != first.node_id
            || component.key.primitive_id != first.primitive_id
    }) {
        return Err("source must contain one exact primitive instance".into());
    }
    let source_material_id = ingest.ir.primitives[0]
        .material_id
        .ok_or("source primitive has no fallback material")?;
    let source_image_sha256 = ingest
        .ir
        .images
        .first()
        .ok_or("source has no fallback image")?
        .sha256
        .clone();
    let texture = decode_embedded_image_to_tga_v1(
        &source_bytes,
        0,
        &limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )?;
    let primitive = &ingest.ir.primitives[0];
    let categories = (0..SOURCE_TRIANGLES)
        .map(|triangle| classify(primitive, &texture, triangle))
        .collect::<Vec<_>>();

    let materials = Category::ALL
        .into_iter()
        .map(|category| AuthoredMaterialV1 {
            authored_material_id: category.id().to_owned(),
            display_name: category.display_name().to_owned(),
            preview_color: category.preview_color().to_owned(),
            source_fallback_material_id: Some(source_material_id),
            source_fallback_image_sha256: Some(source_image_sha256.clone()),
        })
        .collect::<Vec<_>>();
    let mut ranges = BTreeMap::new();
    for category in Category::ALL {
        ranges.insert(category, compact_ranges(&categories, category));
    }
    let face_assignments = Category::ALL
        .into_iter()
        .map(|category| ModelMaterialFaceAssignmentV2 {
            selection: SourceFaceSelectionV2 {
                scene_id: first.scene_id,
                node_id: first.node_id,
                primitive_id: first.primitive_id,
                triangle_ranges: ranges[&category].clone(),
            },
            authored_material_id: category.id().to_owned(),
        })
        .collect::<Vec<_>>();
    let document = ModelMaterialSeparationDocumentV2 {
        schema_version: 2,
        source_sha256: SOURCE_SHA256.to_owned(),
        materials,
        component_assignments: Vec::new(),
        face_assignments,
    };
    let resolved = resolve_model_materials_v2(&ingest.ir, &document)?;
    if resolved.report.source_triangle_count as usize != SOURCE_TRIANGLES
        || resolved.report.output_triangle_count as usize != SOURCE_TRIANGLES
        || resolved.report.unassigned_face_count != 0
        || resolved.report.duplicated_boundary_vertex_count != 0
    {
        return Err("resolved material recipe changed or failed to cover geometry".into());
    }
    let separation_sha256 = model_material_separation_hash_v2(&document)?;
    if separation_sha256 != resolved.report.separation_sha256 {
        return Err("separation hash readback mismatch".into());
    }
    let category_reports = Category::ALL
        .into_iter()
        .map(|category| {
            let triangle_count = categories
                .iter()
                .filter(|candidate| **candidate == category)
                .count();
            CategoryReport {
                authored_material_id: category.id().to_owned(),
                triangle_count,
                fraction: triangle_count as f64 / SOURCE_TRIANGLES as f64,
                range_count: ranges[&category].len(),
            }
        })
        .collect();
    let report = PreparationReport {
        schema_version: 1,
        status: "offline_recipe_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        source_triangle_count: SOURCE_TRIANGLES,
        output_triangle_count: resolved.report.output_triangle_count,
        output_vertex_count: resolved.report.output_vertex_count,
        duplicated_boundary_vertex_count: resolved.report.duplicated_boundary_vertex_count,
        separation_sha256,
        category_reports,
        geometry_cleanup: false,
        source_uv0_preserved: true,
        classifier: ClassifierReport {
            kind: "source-bound geometric and source-color heuristic".to_owned(),
            scope: "tlc-ship-under-construction-s1-p150k-v1 only".to_owned(),
            semantic_guarantee: false,
            notes: vec![
                "Wood is the conservative complement.".to_owned(),
                "Sail and Cloth split the two furled-canvas rig regions.".to_owned(),
                "Rope and Metal selections are conservative test regions and require visual review."
                    .to_owned(),
                "No triangle, vertex, UV or index-buffer mutation is permitted.".to_owned(),
            ],
        },
    };

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("material-separation-v2.json"),
        serde_json::to_vec_pretty(&document)?,
    )?;
    fs::write(
        output.join("preparation-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    render_projection(
        primitive,
        &categories,
        0,
        1,
        2,
        &output.join("assignment-side-xy.png"),
    )?;
    render_projection(
        primitive,
        &categories,
        0,
        2,
        1,
        &output.join("assignment-top-xz.png"),
    )?;
    render_projection(
        primitive,
        &categories,
        2,
        1,
        0,
        &output.join("assignment-front-zy.png"),
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_samples_gltf_zero_v_from_the_top_scanline() {
        let image = TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 2,
            pixel_format: m2a_core::tga::TgaPixelFormatV1::Rgb8,
            pixels: vec![255, 0, 0, 0, 0, 255],
        };
        assert_eq!(sample_texture(&image, [0.0, 0.0]), [255, 0, 0]);
        assert_eq!(sample_texture(&image, [0.0, 0.999]), [0, 0, 255]);
    }
}
