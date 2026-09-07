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
    Cloth,
    Metal,
    Rope,
    Sail,
    WoodBow,
    WoodDeck,
    WoodFrame,
    WoodHull,
    WoodMasts,
    WoodRails,
    WoodStern,
    WoodSupports,
}

impl Category {
    const ALL: [Self; 12] = [
        Self::Cloth,
        Self::Metal,
        Self::Rope,
        Self::Sail,
        Self::WoodBow,
        Self::WoodDeck,
        Self::WoodFrame,
        Self::WoodHull,
        Self::WoodMasts,
        Self::WoodRails,
        Self::WoodStern,
        Self::WoodSupports,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::Cloth => "cloth",
            Self::Metal => "metal",
            Self::Rope => "rope",
            Self::Sail => "sail",
            Self::WoodBow => "wood_bow",
            Self::WoodDeck => "wood_deck",
            Self::WoodFrame => "wood_frame",
            Self::WoodHull => "wood_hull",
            Self::WoodMasts => "wood_masts",
            Self::WoodRails => "wood_rails",
            Self::WoodStern => "wood_stern",
            Self::WoodSupports => "wood_supports",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Cloth => "Secondary working canvas",
            Self::Metal => "Dark forged fittings",
            Self::Rope => "Hemp rope and rigging",
            Self::Sail => "Primary furled sail canvas",
            Self::WoodBow => "Bow framing and stem timber",
            Self::WoodDeck => "Deck planking",
            Self::WoodFrame => "Internal ribs and structural beams",
            Self::WoodHull => "Outer hull planking",
            Self::WoodMasts => "Masts and spars",
            Self::WoodRails => "Rails and upper trim",
            Self::WoodStern => "Stern and raised superstructure",
            Self::WoodSupports => "Construction supports and lower framing",
        }
    }

    fn preview_color(self) -> &'static str {
        match self {
            Self::Cloth => "#9E7356",
            Self::Metal => "#48545E",
            Self::Rope => "#C8994B",
            Self::Sail => "#D7CFB7",
            Self::WoodBow => "#8A653B",
            Self::WoodDeck => "#C08A4A",
            Self::WoodFrame => "#59412F",
            Self::WoodHull => "#8A4F2D",
            Self::WoodMasts => "#4F3526",
            Self::WoodRails => "#A2703D",
            Self::WoodStern => "#725A3F",
            Self::WoodSupports => "#3F332B",
        }
    }

    fn rgb(self) -> Rgb<u8> {
        let value = self.preview_color().trim_start_matches('#');
        Rgb([
            u8::from_str_radix(&value[0..2], 16).expect("constant color"),
            u8::from_str_radix(&value[2..4], 16).expect("constant color"),
            u8::from_str_radix(&value[4..6], 16).expect("constant color"),
        ])
    }
}

#[derive(Clone, Copy, Debug)]
struct TriangleFeatures {
    centroid: [f32; 3],
    normal: [f32; 3],
    double_area: f32,
    luma: f32,
    chroma: u8,
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
    classifier_kind: String,
    classifier_semantic_guarantee: bool,
    notes: Vec<String>,
}

fn sample_texture(image: &TgaImageV1, uv: [f32; 2]) -> [u8; 3] {
    let channels = match image.pixel_format {
        m2a_core::tga::TgaPixelFormatV1::Rgb8 => 3,
        m2a_core::tga::TgaPixelFormatV1::Rgba8 => 4,
    };
    let x = (uv[0].rem_euclid(1.0) * (image.width - 1) as f32).round() as usize;
    let y = (uv[1].rem_euclid(1.0) * (image.height - 1) as f32).round() as usize;
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

fn triangle_features(
    primitive: &IrPrimitive,
    texture: &TgaImageV1,
    triangle_index: usize,
) -> TriangleFeatures {
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
    TriangleFeatures {
        centroid,
        normal,
        double_area,
        luma: 0.2126 * color[0] as f32 + 0.7152 * color[1] as f32 + 0.0722 * color[2] as f32,
        chroma: color.iter().max().unwrap() - color.iter().min().unwrap(),
    }
}

fn classify_wood(features: TriangleFeatures) -> Category {
    let [x, y, z] = features.centroid;
    let [_, normal_y, normal_z] = features.normal;
    let fore_mast = (-0.46..=-0.16).contains(&x);
    let aft_mast = (0.18..=0.44).contains(&x);

    if y > 0.10 && (fore_mast || aft_mast) {
        Category::WoodMasts
    } else if x > 0.32 && y > -0.18 {
        Category::WoodStern
    } else if x < -0.55 && y > -0.20 {
        Category::WoodBow
    } else if normal_y.abs() > 0.55 && y > -0.22 {
        Category::WoodDeck
    } else if y < -0.43 || (y < -0.32 && z.abs() > 0.33) {
        Category::WoodSupports
    } else if (-0.08..=0.10).contains(&y) && normal_y.abs() <= 0.75 {
        Category::WoodRails
    } else if normal_z.abs() > 0.42 && y < 0.02 {
        Category::WoodHull
    } else {
        Category::WoodFrame
    }
}

fn classify(primitive: &IrPrimitive, texture: &TgaImageV1, triangle_index: usize) -> Category {
    let features = triangle_features(primitive, texture, triangle_index);
    let [x, y, z] = features.centroid;
    let fore_rig = (-0.52..=-0.10).contains(&x) && y > -0.02;
    let aft_rig = (0.16..=0.62).contains(&x) && y > 0.00;
    let rig = fore_rig || aft_rig;
    let cloth_surface = rig
        && z.abs() > 0.035
        && features.normal[0].abs() > 0.18
        && y > 0.04
        && features.double_area > 1.0e-7;

    if rig
        && y > 0.075
        && features.double_area < 0.000_045
        && features.luma < 92.0
        && (!cloth_surface || features.normal[0].abs() < 0.42)
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
    if y < 0.12
        && features.chroma <= 24
        && (54.0..=138.0).contains(&features.luma)
        && features.double_area < 0.000_45
    {
        return Category::Metal;
    }
    classify_wood(features)
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
        let points = [triangle[0], triangle[1], triangle[2]]
            .map(|index| primitive.positions[index as usize]);
        let projected = points.map(project);
        let min_x = projected
            .iter()
            .map(|p| p[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .clamp(0.0, (WIDTH - 1) as f32) as u32;
        let max_x = projected
            .iter()
            .map(|p| p[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .clamp(0.0, (WIDTH - 1) as f32) as u32;
        let min_y = projected
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .clamp(0.0, (HEIGHT - 1) as f32) as u32;
        let max_y = projected
            .iter()
            .map(|p| p[1])
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
    let source = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_detailed_material_separation_v3 <source.glb> <output-directory>",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: prepare_tlc_ship_detailed_material_separation_v3 <source.glb> <output-directory>",
    )?);
    if args.next().is_some() {
        return Err("usage: prepare_tlc_ship_detailed_material_separation_v3 <source.glb> <output-directory>".into());
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
    let ranges = Category::ALL
        .into_iter()
        .map(|category| (category, compact_ranges(&categories, category)))
        .collect::<BTreeMap<_, _>>();
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
        .collect::<Vec<_>>();
    if category_reports
        .iter()
        .any(|category| category.triangle_count == 0)
    {
        return Err("detailed material classifier produced an empty authored material".into());
    }
    let report = PreparationReport {
        schema_version: 1,
        status: "offline_detailed_recipe_validated".to_owned(),
        source_sha256: SOURCE_SHA256.to_owned(),
        source_triangle_count: SOURCE_TRIANGLES,
        output_triangle_count: resolved.report.output_triangle_count,
        output_vertex_count: resolved.report.output_vertex_count,
        duplicated_boundary_vertex_count: resolved.report.duplicated_boundary_vertex_count,
        separation_sha256,
        category_reports,
        geometry_cleanup: false,
        source_uv0_preserved: true,
        classifier_kind: "exact-source geometric and source-color heuristic with semantic wood zones".to_owned(),
        classifier_semantic_guarantee: false,
        notes: vec![
            "The four non-wood groups preserve the prior source-bound classifier.".to_owned(),
            "The former Wood complement is split into eight position/normal-based authored materials.".to_owned(),
            "No triangle, index, source UV, scale, placement or collision mutation is permitted.".to_owned(),
            "Final semantic correctness remains owner-reviewed in the Material Separation overlay and Toolset.".to_owned(),
        ],
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

    fn features(centroid: [f32; 3], normal: [f32; 3]) -> TriangleFeatures {
        TriangleFeatures {
            centroid,
            normal,
            double_area: 0.01,
            luma: 100.0,
            chroma: 30,
        }
    }

    #[test]
    fn wood_zones_have_stable_precedence() {
        assert_eq!(
            classify_wood(features([-0.30, 0.20, 0.0], [0.0, 0.0, 1.0])),
            Category::WoodMasts
        );
        assert_eq!(
            classify_wood(features([0.60, 0.00, 0.0], [0.0, 0.0, 1.0])),
            Category::WoodStern
        );
        assert_eq!(
            classify_wood(features([-0.70, 0.00, 0.0], [0.0, 0.0, 1.0])),
            Category::WoodBow
        );
        assert_eq!(
            classify_wood(features([0.00, -0.10, 0.0], [0.0, 1.0, 0.0])),
            Category::WoodDeck
        );
        assert_eq!(
            classify_wood(features([0.00, -0.48, 0.0], [0.0, 0.0, 1.0])),
            Category::WoodSupports
        );
        assert_eq!(
            classify_wood(features([0.00, 0.00, 0.0], [0.0, 0.0, 1.0])),
            Category::WoodRails
        );
        assert_eq!(
            classify_wood(features([0.00, -0.20, 0.0], [0.0, 0.0, 1.0])),
            Category::WoodHull
        );
        assert_eq!(
            classify_wood(features([0.00, -0.20, 0.0], [1.0, 0.0, 0.0])),
            Category::WoodFrame
        );
    }
}
