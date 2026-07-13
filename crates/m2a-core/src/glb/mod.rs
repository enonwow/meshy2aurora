use std::{collections::HashSet, fmt, io::Cursor};

use gltf::{
    mesh::{Mode, Semantic},
    texture::{MagFilter, MinFilter, WrappingMode},
};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits as ImageLimits};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::tga::{TGA_MAX_OUTPUT_BYTES, TGA_SCHEMA_VERSION, TgaImageV1, TgaPixelFormatV1};

pub const GLB_SCHEMA_VERSION: u32 = 1;
pub const MAX_DECODED_IMAGE_DIMENSION_V1: u32 = 16_384;
pub const MAX_DECODED_IMAGE_PIXELS_V1: u64 = 16 * 1024 * 1024;
const TGA_CONTAINER_OVERHEAD_V1: u64 = 18 + 26;
pub const MAX_DECODED_IMAGE_BYTES_V1: u64 = TGA_MAX_OUTPUT_BYTES - TGA_CONTAINER_OVERHEAD_V1;
const MAX_IMAGE_DECODER_ALLOCATION_BYTES_V1: u64 = MAX_DECODED_IMAGE_PIXELS_V1 * 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddedImageDecodeLimitsV1 {
    pub max_width: u32,
    pub max_height: u32,
    pub max_pixels: u64,
    pub max_decoded_bytes: u64,
}

impl Default for EmbeddedImageDecodeLimitsV1 {
    fn default() -> Self {
        Self {
            max_width: MAX_DECODED_IMAGE_DIMENSION_V1,
            max_height: MAX_DECODED_IMAGE_DIMENSION_V1,
            max_pixels: MAX_DECODED_IMAGE_PIXELS_V1,
            max_decoded_bytes: MAX_DECODED_IMAGE_BYTES_V1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlbLimits {
    pub max_input_bytes: usize,
    pub max_json_chunk_bytes: usize,
    pub max_nodes: usize,
    pub max_node_depth: usize,
    pub max_meshes: usize,
    pub max_primitives: usize,
    pub max_accessors: usize,
    pub max_buffer_views: usize,
    pub max_materials: usize,
    pub max_textures: usize,
    pub max_samplers: usize,
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_decoded_geometry_bytes: usize,
    pub max_images: usize,
    pub max_single_image_bytes: usize,
    pub max_total_image_bytes: usize,
    pub max_skins: usize,
    pub max_joints: usize,
    pub max_animations: usize,
    pub max_animation_samplers: usize,
    pub max_animation_channels: usize,
    pub max_keyframes: usize,
    pub max_decoded_skin_animation_bytes: usize,
    pub max_diagnostics: usize,
    pub triangle_warning_above: usize,
    pub triangle_blocking_above: usize,
}

impl Default for GlbLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_json_chunk_bytes: 16 * 1024 * 1024,
            max_nodes: 100_000,
            max_node_depth: 512,
            max_meshes: 100_000,
            max_primitives: 100_000,
            max_accessors: 100_000,
            max_buffer_views: 100_000,
            max_materials: 10_000,
            max_textures: 10_000,
            max_samplers: 10_000,
            max_vertices: 1_000_000,
            max_indices: 3_000_000,
            max_decoded_geometry_bytes: 256 * 1024 * 1024,
            max_images: 10_000,
            max_single_image_bytes: 32 * 1024 * 1024,
            max_total_image_bytes: 64 * 1024 * 1024,
            max_skins: 10_000,
            max_joints: 100_000,
            max_animations: 10_000,
            max_animation_samplers: 100_000,
            max_animation_channels: 100_000,
            max_keyframes: 1_000_000,
            max_decoded_skin_animation_bytes: 64 * 1024 * 1024,
            max_diagnostics: 2_048,
            triangle_warning_above: 5_000,
            triangle_blocking_above: 10_000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbFatalError {
    pub schema_version: u32,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_path: Option<String>,
}

impl GlbFatalError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            schema_version: GLB_SCHEMA_VERSION,
            code: code.to_owned(),
            message: message.into(),
            byte_offset: None,
            json_path: None,
        }
    }

    fn at(mut self, byte_offset: usize) -> Self {
        self.byte_offset = Some(byte_offset);
        self
    }

    fn in_json(mut self, json_path: impl Into<String>) -> Self {
        self.json_path = Some(json_path.into());
        self
    }
}

impl fmt::Display for GlbFatalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for GlbFatalError {}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbIngestResult {
    pub schema_version: u32,
    pub ir: AuroraAssetIr,
    pub report: GlbInspectionReport,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuroraAssetIr {
    pub schema_version: u32,
    pub source: IrSource,
    pub coordinate_space: CoordinatePolicy,
    pub default_scene_id: Option<u32>,
    pub scenes: Vec<IrScene>,
    pub nodes: Vec<IrNode>,
    pub meshes: Vec<IrMesh>,
    pub primitives: Vec<IrPrimitive>,
    pub materials: Vec<IrMaterial>,
    pub textures: Vec<IrTexture>,
    pub samplers: Vec<IrSampler>,
    pub images: Vec<IrImageRef>,
    pub skins: Vec<IrSkin>,
    pub animations: Vec<IrAnimation>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrSource {
    pub format: String,
    pub byte_length: usize,
    pub sha256: String,
    pub asset_version: String,
    pub generator: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatePolicy {
    pub stored_space: String,
    pub up: String,
    pub forward_convention: String,
    pub handedness: String,
    pub units: String,
    pub positions_policy: String,
    pub uv_policy: String,
    pub winding_policy: String,
    pub target_transform_status: String,
}

impl Default for CoordinatePolicy {
    fn default() -> Self {
        Self {
            stored_space: "GLTF_SOURCE".to_owned(),
            up: "POSITIVE_Y".to_owned(),
            forward_convention: "POSITIVE_Z".to_owned(),
            handedness: "RIGHT_HANDED".to_owned(),
            units: "METERS_DECLARED_BY_MESHY".to_owned(),
            positions_policy: "PRESERVED".to_owned(),
            uv_policy: "PRESERVED".to_owned(),
            winding_policy: "PRESERVED".to_owned(),
            target_transform_status: "UNRESOLVED_M3".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrScene {
    pub id: u32,
    pub name: Option<String>,
    pub root_node_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrNode {
    pub id: u32,
    pub name: Option<String>,
    pub child_ids: Vec<u32>,
    pub parent_ids: Vec<u32>,
    pub transform: IrTransform,
    pub mesh_id: Option<u32>,
    pub skin_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrTransform {
    pub kind: String,
    pub matrix: Option<[f32; 16]>,
    pub translation: Option<[f32; 3]>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrMesh {
    pub id: u32,
    pub name: Option<String>,
    pub primitive_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrPrimitive {
    pub id: u32,
    pub source_mesh_id: u32,
    pub source_primitive_index: u32,
    pub topology: String,
    pub material_id: Option<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uv0: Vec<[f32; 2]>,
    pub joints0: Vec<[u16; 4]>,
    pub weights0: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub source_was_indexed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrMaterial {
    pub id: u32,
    pub name: Option<String>,
    pub base_color_factor: [f32; 4],
    pub base_color_texture: Option<IrTextureBinding>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<IrTextureBinding>,
    pub normal_texture: Option<IrTextureBinding>,
    pub emissive_factor: [f32; 3],
    pub emissive_texture: Option<IrTextureBinding>,
    pub alpha_mode: String,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrTextureBinding {
    pub texture_id: u32,
    pub tex_coord_set: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrTexture {
    pub id: u32,
    pub source_image_id: u32,
    pub sampler_index: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrSampler {
    pub id: u32,
    pub name: Option<String>,
    pub mag_filter: Option<String>,
    pub min_filter: Option<String>,
    pub wrap_s: String,
    pub wrap_t: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrImageRef {
    pub id: u32,
    pub name: Option<String>,
    pub mime_type: String,
    pub byte_offset: usize,
    pub byte_length: usize,
    pub sha256: String,
    pub payload_embedded_in_json: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrSkin {
    pub id: u32,
    pub name: Option<String>,
    pub skeleton_root_node_id: Option<u32>,
    pub joint_node_ids: Vec<u32>,
    pub inverse_bind_matrices: Vec<[f32; 16]>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrAnimation {
    pub id: u32,
    pub name: Option<String>,
    pub duration_seconds: f32,
    pub samplers: Vec<IrAnimationSampler>,
    pub channels: Vec<IrAnimationChannel>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrAnimationSampler {
    pub id: u32,
    pub interpolation: String,
    pub input_times_seconds: Vec<f32>,
    pub output_accessor_type: String,
    pub output_values: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrAnimationChannel {
    pub sampler_id: u32,
    pub target_node_id: u32,
    pub target_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbInspectionReport {
    pub schema_version: u32,
    pub format: String,
    pub input: GlbInputIdentity,
    pub coordinate_policy: CoordinatePolicy,
    pub inventory: GlbInventory,
    pub statistics: GlbStatistics,
    pub gates: Vec<GlbGate>,
    pub diagnostics: Vec<GlbDiagnostic>,
    pub conversion_eligible: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbInputIdentity {
    pub byte_length: usize,
    pub sha256: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbInventory {
    pub scene_count: usize,
    pub node_count: usize,
    pub mesh_count: usize,
    pub primitive_count: usize,
    pub material_count: usize,
    pub texture_count: usize,
    pub sampler_count: usize,
    pub image_count: usize,
    pub skin_count: usize,
    pub joint_reference_count: usize,
    pub animation_count: usize,
    pub keyframe_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbStatistics {
    pub vertex_count: usize,
    pub index_count: usize,
    pub triangle_count: usize,
    pub bounds_min: Option<[f32; 3]>,
    pub bounds_max: Option<[f32; 3]>,
    pub primitives_missing_normals: usize,
    pub primitives_missing_uv0: usize,
    pub non_triangle_primitives: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbGate {
    pub code: String,
    pub severity: String,
    pub path: String,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlbDiagnostic {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub byte_offset: Option<usize>,
    pub json_path: Option<String>,
    pub message: String,
}

pub fn decode_embedded_image_to_tga_v1(
    input: &[u8],
    image_index: usize,
    glb_limits: &GlbLimits,
    decode_limits: &EmbeddedImageDecodeLimitsV1,
) -> Result<TgaImageV1, GlbFatalError> {
    validate_embedded_image_decode_limits(decode_limits)?;

    let ingest = ingest_glb(input, glb_limits)?;
    let image_ref = ingest.ir.images.get(image_index).ok_or_else(|| {
        GlbFatalError::new(
            "M2A-GLB-IMAGE-INDEX-INVALID",
            format!(
                "image index {image_index} is outside the {} embedded images",
                ingest.ir.images.len()
            ),
        )
        .in_json(format!("images[{image_index}]"))
    })?;
    let (_, blob) = glb_payloads(input)?;
    let byte_end = image_ref
        .byte_offset
        .checked_add(image_ref.byte_length)
        .ok_or_else(|| {
            GlbFatalError::new(
                "M2A-GLB-INTEGER-OVERFLOW",
                "embedded image byte range overflow",
            )
            .in_json(format!("images[{image_index}].bufferView"))
        })?;
    let bytes = blob.get(image_ref.byte_offset..byte_end).ok_or_else(|| {
        GlbFatalError::new(
            "M2A-GLB-BUFFER-VIEW-OOB",
            "embedded image byte range exceeds the BIN chunk",
        )
        .in_json(format!("images[{image_index}].bufferView"))
    })?;
    let declared_format = match image_ref.mime_type.as_str() {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        _ => {
            return Err(GlbFatalError::new(
                "M2A-GLB-IMAGE-MIME-UNSUPPORTED",
                "embedded image mimeType must be image/png or image/jpeg",
            )
            .in_json(format!("images[{image_index}].mimeType")));
        }
    };
    if let Some(detected_format) = detect_supported_image_format(bytes)
        && detected_format != declared_format
    {
        return Err(GlbFatalError::new(
            "M2A-GLB-IMAGE-MIME-MISMATCH",
            format!(
                "declared {} does not match embedded {} bytes",
                image_ref.mime_type,
                image_format_name(detected_format)
            ),
        )
        .in_json(format!("images[{image_index}].mimeType")));
    }

    let mut decoder_limits = ImageLimits::default();
    decoder_limits.max_image_width = Some(decode_limits.max_width);
    decoder_limits.max_image_height = Some(decode_limits.max_height);
    decoder_limits.max_alloc = Some(MAX_IMAGE_DECODER_ALLOCATION_BYTES_V1);
    let mut decode_reader = ImageReader::with_format(Cursor::new(bytes), declared_format);
    decode_reader.limits(decoder_limits);
    let decoder = decode_reader.into_decoder().map_err(|error| {
        map_embedded_image_decode_error(error, image_index, "could not initialize image decoder")
    })?;
    let (width, height) = decoder.dimensions();
    let pixel_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| embedded_image_limit_error(image_index, "pixel count overflows u64"))?;
    if pixel_count > decode_limits.max_pixels {
        return Err(embedded_image_limit_error(
            image_index,
            format!(
                "decoded image has {pixel_count} pixels but the configured limit is {}",
                decode_limits.max_pixels
            ),
        ));
    }

    let source_decoded_bytes = decoder.total_bytes();
    if source_decoded_bytes > MAX_IMAGE_DECODER_ALLOCATION_BYTES_V1 {
        return Err(embedded_image_limit_error(
            image_index,
            format!(
                "source image decoder requires {source_decoded_bytes} bytes but the internal v1 limit is {MAX_IMAGE_DECODER_ALLOCATION_BYTES_V1}"
            ),
        ));
    }
    let has_alpha = decoder.color_type().has_alpha();
    let channel_count = if has_alpha { 4_u64 } else { 3_u64 };
    let decoded_bytes = pixel_count.checked_mul(channel_count).ok_or_else(|| {
        embedded_image_limit_error(image_index, "decoded byte count overflows u64")
    })?;
    if decoded_bytes > decode_limits.max_decoded_bytes {
        return Err(embedded_image_limit_error(
            image_index,
            format!(
                "decoded image requires {decoded_bytes} bytes but the configured limit is {}",
                decode_limits.max_decoded_bytes
            ),
        ));
    }
    let decoded = DynamicImage::from_decoder(decoder).map_err(|error| {
        map_embedded_image_decode_error(error, image_index, "could not decode embedded image")
    })?;

    let (pixel_format, pixels) = if has_alpha {
        (TgaPixelFormatV1::Rgba8, decoded.into_rgba8().into_raw())
    } else {
        (TgaPixelFormatV1::Rgb8, decoded.into_rgb8().into_raw())
    };
    if u64::try_from(pixels.len()).ok() != Some(decoded_bytes) {
        return Err(GlbFatalError::new(
            "M2A-GLB-IMAGE-DECODE-INVALID",
            "decoded pixel buffer length does not match image dimensions",
        )
        .in_json(format!("images[{image_index}].bufferView")));
    }

    Ok(TgaImageV1 {
        schema_version: TGA_SCHEMA_VERSION,
        width,
        height,
        pixel_format,
        pixels,
    })
}

fn validate_embedded_image_decode_limits(
    limits: &EmbeddedImageDecodeLimitsV1,
) -> Result<(), GlbFatalError> {
    if limits.max_width == 0
        || limits.max_height == 0
        || limits.max_pixels == 0
        || limits.max_decoded_bytes == 0
        || limits.max_width > MAX_DECODED_IMAGE_DIMENSION_V1
        || limits.max_height > MAX_DECODED_IMAGE_DIMENSION_V1
        || limits.max_pixels > MAX_DECODED_IMAGE_PIXELS_V1
        || limits.max_decoded_bytes > MAX_DECODED_IMAGE_BYTES_V1
    {
        return Err(GlbFatalError::new(
            "M2A-GLB-IMAGE-DECODE-LIMIT-EXCEEDED",
            "image decode limits must be non-zero and no greater than their v1 maxima",
        ));
    }
    Ok(())
}

fn detect_supported_image_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(ImageFormat::Png)
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(ImageFormat::Jpeg)
    } else {
        None
    }
}

fn image_format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        _ => "unsupported image",
    }
}

fn map_embedded_image_decode_error(
    error: image::ImageError,
    image_index: usize,
    context: &str,
) -> GlbFatalError {
    if matches!(error, image::ImageError::Limits(_)) {
        embedded_image_limit_error(image_index, format!("{context}: {error}"))
    } else {
        GlbFatalError::new(
            "M2A-GLB-IMAGE-DECODE-INVALID",
            format!("{context}: {error}"),
        )
        .in_json(format!("images[{image_index}].bufferView"))
    }
}

fn embedded_image_limit_error(image_index: usize, message: impl Into<String>) -> GlbFatalError {
    GlbFatalError::new("M2A-GLB-IMAGE-DECODE-LIMIT-EXCEEDED", message)
        .in_json(format!("images[{image_index}].bufferView"))
}

pub fn inspect_glb(input: &[u8], limits: &GlbLimits) -> Result<GlbInspectionReport, GlbFatalError> {
    Ok(ingest_glb(input, limits)?.report)
}

pub fn ingest_glb(input: &[u8], limits: &GlbLimits) -> Result<GlbIngestResult, GlbFatalError> {
    validate_header(input, limits)?;
    let (json_bytes, blob) = glb_payloads(input)?;
    let raw = preflight_json(json_bytes, blob, limits)?;
    let patched_input = if raw.missing_position_primitives.is_empty() {
        None
    } else {
        Some(patch_missing_positions_for_validation(
            json_bytes,
            blob,
            &raw.missing_position_primitives,
        )?)
    };
    let document_input = patched_input.as_deref().unwrap_or(input);
    let gltf = gltf::Gltf::from_slice(document_input).map_err(map_gltf_error)?;
    let gltf_blob = gltf.blob.as_deref().ok_or_else(|| {
        GlbFatalError::new("M2A-GLB-BIN-MISSING", "embedded GLB BIN chunk is required")
    })?;
    validate_document_limits(&gltf.document, limits)?;
    for buffer in gltf.document.buffers() {
        if !matches!(buffer.source(), gltf::buffer::Source::Bin) {
            return Err(GlbFatalError::new(
                "M2A-GLB-EXTERNAL-URI-UNSUPPORTED",
                "external buffer URI is unsupported",
            ));
        }
        if buffer.length() > gltf_blob.len() {
            return Err(GlbFatalError::new(
                "M2A-GLB-BUFFER-VIEW-OOB",
                "declared buffer length exceeds the embedded BIN chunk",
            ));
        }
    }

    let sha256 = sha256_hex(input);
    let scenes = gltf
        .document
        .scenes()
        .map(|scene| IrScene {
            id: scene.index() as u32,
            name: scene.name().map(str::to_owned),
            root_node_ids: scene.nodes().map(|node| node.index() as u32).collect(),
        })
        .collect::<Vec<_>>();

    let node_count = gltf.document.nodes().count();
    let mut parent_ids = vec![Vec::<u32>::new(); node_count];
    for node in gltf.document.nodes() {
        for child in node.children() {
            parent_ids[child.index()].push(node.index() as u32);
        }
    }
    validate_node_graph(&gltf.document, limits)?;
    let nodes = gltf
        .document
        .nodes()
        .map(|node| {
            let transform = match node.transform() {
                gltf::scene::Transform::Matrix { matrix } => IrTransform {
                    kind: "MATRIX".to_owned(),
                    matrix: Some(flatten_matrix(matrix)),
                    translation: None,
                    rotation: None,
                    scale: None,
                },
                gltf::scene::Transform::Decomposed {
                    translation,
                    rotation,
                    scale,
                } => IrTransform {
                    kind: "TRS".to_owned(),
                    matrix: None,
                    translation: Some(translation),
                    rotation: Some(rotation),
                    scale: Some(scale),
                },
            };
            IrNode {
                id: node.index() as u32,
                name: node.name().map(str::to_owned),
                child_ids: node.children().map(|child| child.index() as u32).collect(),
                parent_ids: parent_ids[node.index()].clone(),
                transform,
                mesh_id: node.mesh().map(|mesh| mesh.index() as u32),
                skin_id: node.skin().map(|skin| skin.index() as u32),
            }
        })
        .collect::<Vec<_>>();

    let mut primitives = Vec::new();
    let mut meshes = Vec::new();
    let mut gates = Vec::new();
    let mut statistics = GlbStatistics {
        vertex_count: 0,
        index_count: 0,
        triangle_count: 0,
        bounds_min: None,
        bounds_max: None,
        primitives_missing_normals: 0,
        primitives_missing_uv0: 0,
        non_triangle_primitives: 0,
    };
    let mut decoded_bytes = 0_usize;

    for mesh in gltf.document.meshes() {
        let mut primitive_ids = Vec::new();
        for (source_primitive_index, primitive) in mesh.primitives().enumerate() {
            let primitive_id = primitives.len() as u32;
            primitive_ids.push(primitive_id);
            let path = format!(
                "meshes[{}].primitives[{source_primitive_index}]",
                mesh.index()
            );
            let reader = primitive.reader(|_| Some(gltf_blob));
            let source_missing_position = raw
                .missing_position_primitives
                .contains(&(mesh.index(), source_primitive_index));
            reserve_decoded_items(
                &mut decoded_bytes,
                if source_missing_position {
                    0
                } else {
                    primitive
                        .get(&Semantic::Positions)
                        .map_or(0, |accessor| accessor.count())
                },
                12,
                limits,
            )?;
            let positions = if source_missing_position {
                Vec::new()
            } else {
                reader
                    .read_positions()
                    .map(|values| values.collect::<Vec<_>>())
                    .unwrap_or_default()
            };
            if positions.is_empty() {
                gates.push(blocking_gate(
                    "M2A-GLB-POSITION-MISSING",
                    &path,
                    "POSITION VEC3",
                    "missing",
                    "primitive has no usable POSITION attribute",
                ));
            }
            reserve_decoded_items(
                &mut decoded_bytes,
                primitive
                    .get(&Semantic::Normals)
                    .map_or(0, |accessor| accessor.count()),
                12,
                limits,
            )?;
            let normals = reader
                .read_normals()
                .map(|values| values.collect::<Vec<_>>())
                .unwrap_or_default();
            reserve_decoded_items(
                &mut decoded_bytes,
                primitive
                    .get(&Semantic::Tangents)
                    .map_or(0, |accessor| accessor.count()),
                16,
                limits,
            )?;
            let tangents = reader
                .read_tangents()
                .map(|values| values.collect::<Vec<_>>())
                .unwrap_or_default();
            reserve_decoded_items(
                &mut decoded_bytes,
                primitive
                    .get(&Semantic::TexCoords(0))
                    .map_or(0, |accessor| accessor.count()),
                8,
                limits,
            )?;
            let uv0 = reader
                .read_tex_coords(0)
                .map(|values| values.into_f32().collect::<Vec<_>>())
                .unwrap_or_default();
            reserve_decoded_items(
                &mut decoded_bytes,
                primitive
                    .get(&Semantic::Joints(0))
                    .map_or(0, |accessor| accessor.count()),
                8,
                limits,
            )?;
            let joints0 = reader
                .read_joints(0)
                .map(|values| values.into_u16().collect::<Vec<_>>())
                .unwrap_or_default();
            reserve_decoded_items(
                &mut decoded_bytes,
                primitive
                    .get(&Semantic::Weights(0))
                    .map_or(0, |accessor| accessor.count()),
                16,
                limits,
            )?;
            let weights0 = reader
                .read_weights(0)
                .map(|values| values.into_f32().collect::<Vec<_>>())
                .unwrap_or_default();
            let source_was_indexed = primitive.indices().is_some();
            let indices = if let Some(values) = reader.read_indices() {
                reserve_decoded_items(
                    &mut decoded_bytes,
                    primitive.indices().map_or(0, |accessor| accessor.count()),
                    4,
                    limits,
                )?;
                values.into_u32().collect::<Vec<_>>()
            } else {
                reserve_decoded_items(&mut decoded_bytes, positions.len(), 4, limits)?;
                (0..positions.len())
                    .map(u32::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| {
                        GlbFatalError::new(
                            "M2A-GLB-INTEGER-OVERFLOW",
                            "vertex index does not fit u32",
                        )
                    })?
            };

            reject_nonfinite(&positions, &normals, &tangents, &uv0, &weights0)?;
            if !source_missing_position && !normals.is_empty() && normals.len() != positions.len() {
                gates.push(blocking_gate(
                    "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
                    &path,
                    &positions.len().to_string(),
                    &normals.len().to_string(),
                    "NORMAL count differs from POSITION count",
                ));
            }
            if !source_missing_position && !uv0.is_empty() && uv0.len() != positions.len() {
                gates.push(blocking_gate(
                    "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
                    &path,
                    &positions.len().to_string(),
                    &uv0.len().to_string(),
                    "TEXCOORD_0 count differs from POSITION count",
                ));
            }
            for (semantic, count) in [
                ("TANGENT", tangents.len()),
                ("JOINTS_0", joints0.len()),
                ("WEIGHTS_0", weights0.len()),
            ] {
                if !source_missing_position && count != 0 && count != positions.len() {
                    gates.push(blocking_gate(
                        "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
                        &path,
                        &positions.len().to_string(),
                        &count.to_string(),
                        &format!("{semantic} count differs from POSITION count"),
                    ));
                }
            }
            if normals.is_empty() {
                statistics.primitives_missing_normals += 1;
                gates.push(warning_gate(
                    "M2A-GLB-NORMALS-MISSING",
                    &path,
                    "NORMAL VEC3",
                    "missing",
                    "primitive has no normals",
                ));
            }
            if uv0.is_empty() {
                statistics.primitives_missing_uv0 += 1;
                gates.push(blocking_gate(
                    "M2A-GLB-UV0-MISSING",
                    &path,
                    "TEXCOORD_0 VEC2",
                    "missing",
                    "primitive has no UV0 data",
                ));
            }
            let is_triangles = primitive.mode() == Mode::Triangles;
            if !is_triangles {
                statistics.non_triangle_primitives += 1;
                gates.push(blocking_gate(
                    "M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED",
                    &path,
                    "TRIANGLES",
                    &format!("{:?}", primitive.mode()),
                    "only triangle primitives are conversion-eligible",
                ));
            }
            if primitive.morph_targets().len() != 0 {
                gates.push(blocking_gate(
                    "M2A-GLB-MORPH-TARGETS-DEFERRED",
                    &path,
                    "no morph targets in M2 conversion subset",
                    &primitive.morph_targets().len().to_string(),
                    "morph-target conversion semantics are deferred",
                ));
            }
            if primitive.get(&Semantic::Joints(1)).is_some()
                || primitive.get(&Semantic::Weights(1)).is_some()
            {
                gates.push(warning_gate(
                    "M2A-GLB-SKIN-INFLUENCE-COUNT",
                    &path,
                    "at most JOINTS_0/WEIGHTS_0 (four lanes)",
                    "secondary skin influence set present",
                    "source skinning exposes more than four influence lanes",
                ));
            }
            if let Some(index) = (!source_missing_position)
                .then(|| {
                    indices
                        .iter()
                        .copied()
                        .find(|index| *index as usize >= positions.len())
                })
                .flatten()
            {
                gates.push(blocking_gate(
                    "M2A-GLB-INDEX-OOB",
                    &path,
                    &format!("index < {}", positions.len()),
                    &index.to_string(),
                    "index is outside the POSITION domain",
                ));
            }
            if is_triangles && !indices.len().is_multiple_of(3) {
                gates.push(blocking_gate(
                    "M2A-GLB-INCOMPLETE-TRIANGLES",
                    &path,
                    "index count divisible by 3",
                    &indices.len().to_string(),
                    "triangle primitive contains an incomplete final triangle",
                ));
            }
            if is_triangles
                && !source_missing_position
                && has_degenerate_triangle(&positions, &indices)
            {
                gates.push(blocking_gate(
                    "M2A-GLB-DEGENERATE-TRIANGLES",
                    &path,
                    "non-degenerate triangles",
                    "one or more repeated-index or exact zero-area triangles",
                    "degenerate triangle repair is deferred to an explicit conversion policy",
                ));
            }
            let triangle_count = if is_triangles { indices.len() / 3 } else { 0 };
            let (bounds_min, bounds_max) = bounds(&positions);
            merge_bounds(
                &mut statistics.bounds_min,
                &mut statistics.bounds_max,
                bounds_min,
                bounds_max,
            );
            statistics.vertex_count = checked_add(statistics.vertex_count, positions.len())?;
            statistics.index_count = checked_add(statistics.index_count, indices.len())?;
            statistics.triangle_count = checked_add(statistics.triangle_count, triangle_count)?;
            primitives.push(IrPrimitive {
                id: primitive_id,
                source_mesh_id: mesh.index() as u32,
                source_primitive_index: source_primitive_index as u32,
                topology: if is_triangles { "TRIANGLES" } else { "OTHER" }.to_owned(),
                material_id: primitive.material().index().map(|index| index as u32),
                positions,
                normals,
                tangents,
                uv0,
                joints0,
                weights0,
                indices,
                bounds_min,
                bounds_max,
                source_was_indexed,
            });
        }
        meshes.push(IrMesh {
            id: mesh.index() as u32,
            name: mesh.name().map(str::to_owned),
            primitive_ids,
        });
    }

    debug_assert_eq!(statistics.vertex_count, raw.vertex_count);
    debug_assert_eq!(statistics.index_count, raw.index_count);
    debug_assert_eq!(statistics.triangle_count, raw.triangle_count);
    debug_assert_eq!(decoded_bytes, raw.decoded_geometry_bytes);
    if statistics.triangle_count > limits.triangle_blocking_above {
        gates.push(blocking_gate(
            "M2A-GLB-GEOMETRY-OVER-BUDGET",
            "statistics.triangleCount",
            &format!("<= {} triangles", limits.triangle_blocking_above),
            &statistics.triangle_count.to_string(),
            "asset exceeds the conversion triangle budget",
        ));
    } else if statistics.triangle_count > limits.triangle_warning_above {
        gates.push(warning_gate(
            "M2A-GLB-GEOMETRY-WARNING",
            "statistics.triangleCount",
            &format!("<= {} triangles", limits.triangle_warning_above),
            &statistics.triangle_count.to_string(),
            "asset exceeds the preferred triangle budget",
        ));
    }

    gates.push(warning_gate(
        "M2A-GLB-TARGET-TRANSFORM-UNRESOLVED",
        "coordinateSpace",
        "resolved by M3",
        "UNRESOLVED_M3",
        "M2 preserves glTF source coordinates, UV values and winding",
    ));
    let skins = decode_skins(&gltf.document, gltf_blob)?;
    validate_skin_joint_lanes(&nodes, &primitives, &skins)?;
    let animations = decode_animations(&gltf.document, gltf_blob, &mut gates)?;
    let materials = gltf
        .document
        .materials()
        .map(material_from_gltf)
        .collect::<Vec<_>>();
    reject_nonfinite_materials(&materials)?;
    for material in &materials {
        if material.base_color_texture.is_none() {
            gates.push(warning_gate(
                "M2A-GLB-BASECOLOR-TEXTURE-MISSING",
                &format!(
                    "materials[{}].pbrMetallicRoughness.baseColorTexture",
                    material.id
                ),
                "embedded baseColorTexture",
                "missing",
                "material has no base-color texture and requires downstream policy",
            ));
        }
    }
    let textures = gltf
        .document
        .textures()
        .map(|texture| IrTexture {
            id: texture.index() as u32,
            source_image_id: texture.source().index() as u32,
            sampler_index: texture.sampler().index().map(|index| index as u32),
        })
        .collect::<Vec<_>>();
    let samplers = gltf
        .document
        .samplers()
        .enumerate()
        .map(|(index, sampler)| IrSampler {
            id: index as u32,
            name: sampler.name().map(str::to_owned),
            mag_filter: sampler.mag_filter().map(mag_filter_name).map(str::to_owned),
            min_filter: sampler.min_filter().map(min_filter_name).map(str::to_owned),
            wrap_s: wrapping_mode_name(sampler.wrap_s()).to_owned(),
            wrap_t: wrapping_mode_name(sampler.wrap_t()).to_owned(),
        })
        .collect::<Vec<_>>();
    let images = gltf
        .document
        .images()
        .map(|image| match image.source() {
            gltf::image::Source::View { view, mime_type } => {
                let byte_offset = view.offset();
                let byte_length = view.length();
                let byte_end = byte_offset.checked_add(byte_length).ok_or_else(|| {
                    GlbFatalError::new(
                        "M2A-GLB-INTEGER-OVERFLOW",
                        "embedded image byte range overflow",
                    )
                    .in_json(format!("images[{}].bufferView", image.index()))
                })?;
                let bytes = gltf_blob.get(byte_offset..byte_end).ok_or_else(|| {
                    GlbFatalError::new(
                        "M2A-GLB-BUFFER-VIEW-OOB",
                        "embedded image byte range exceeds the BIN chunk",
                    )
                    .in_json(format!("images[{}].bufferView", image.index()))
                })?;
                Ok(IrImageRef {
                    id: image.index() as u32,
                    name: image.name().map(str::to_owned),
                    mime_type: mime_type.to_owned(),
                    byte_offset,
                    byte_length,
                    sha256: sha256_hex(bytes),
                    payload_embedded_in_json: false,
                })
            }
            gltf::image::Source::Uri { .. } => Err(GlbFatalError::new(
                "M2A-GLB-EXTERNAL-URI-UNSUPPORTED",
                "external image URI is unsupported",
            )
            .in_json(format!("images[{}].uri", image.index()))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let coordinate_policy = CoordinatePolicy::default();
    let inventory = GlbInventory {
        scene_count: scenes.len(),
        node_count: nodes.len(),
        mesh_count: meshes.len(),
        primitive_count: primitives.len(),
        material_count: materials.len(),
        texture_count: textures.len(),
        sampler_count: samplers.len(),
        image_count: images.len(),
        skin_count: skins.len(),
        joint_reference_count: skins.iter().map(|skin| skin.joint_node_ids.len()).sum(),
        animation_count: animations.len(),
        keyframe_count: animations
            .iter()
            .flat_map(|animation| animation.samplers.iter())
            .map(|sampler| sampler.input_times_seconds.len())
            .sum(),
    };
    let conversion_eligible = !gates.iter().any(|gate| gate.severity == "BLOCKING");
    let report = GlbInspectionReport {
        schema_version: GLB_SCHEMA_VERSION,
        format: "GLB_2_0".to_owned(),
        input: GlbInputIdentity {
            byte_length: input.len(),
            sha256: sha256.clone(),
        },
        coordinate_policy: coordinate_policy.clone(),
        inventory,
        statistics,
        gates,
        diagnostics: raw.diagnostics,
        conversion_eligible,
    };
    let ir = AuroraAssetIr {
        schema_version: GLB_SCHEMA_VERSION,
        source: IrSource {
            format: "GLB_2_0".to_owned(),
            byte_length: input.len(),
            sha256,
            asset_version: raw.asset_version,
            generator: raw.generator,
        },
        coordinate_space: coordinate_policy,
        default_scene_id: gltf
            .document
            .default_scene()
            .map(|scene| scene.index() as u32),
        scenes,
        nodes,
        meshes,
        primitives,
        materials,
        textures,
        samplers,
        images,
        skins,
        animations,
    };
    Ok(GlbIngestResult {
        schema_version: GLB_SCHEMA_VERSION,
        ir,
        report,
    })
}

fn has_degenerate_triangle(positions: &[[f32; 3]], indices: &[u32]) -> bool {
    indices.chunks_exact(3).any(|triangle| {
        if triangle[0] == triangle[1] || triangle[0] == triangle[2] || triangle[1] == triangle[2] {
            return true;
        }
        let Some(a) = positions.get(triangle[0] as usize) else {
            return false;
        };
        let Some(b) = positions.get(triangle[1] as usize) else {
            return false;
        };
        let Some(c) = positions.get(triangle[2] as usize) else {
            return false;
        };
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        cross == [0.0; 3]
    })
}

fn validate_header(input: &[u8], limits: &GlbLimits) -> Result<(), GlbFatalError> {
    if input.is_empty() {
        return Err(GlbFatalError::new(
            "M2A-GLB-INPUT-EMPTY",
            "GLB input is empty",
        ));
    }
    if input.len() > limits.max_input_bytes {
        return Err(GlbFatalError::new(
            "M2A-GLB-INPUT-LIMIT-EXCEEDED",
            format!(
                "input length {} exceeds {}",
                input.len(),
                limits.max_input_bytes
            ),
        ));
    }
    if input.len() < 12 || &input[..4] != b"glTF" {
        return Err(GlbFatalError::new("M2A-GLB-HEADER-INVALID", "invalid GLB header").at(0));
    }
    let version = read_u32(input, 4)?;
    if version != 2 {
        return Err(GlbFatalError::new(
            "M2A-GLB-VERSION-UNSUPPORTED",
            format!("GLB version {version} is unsupported"),
        )
        .at(4));
    }
    let declared = read_u32(input, 8)? as usize;
    if declared != input.len() {
        return Err(GlbFatalError::new(
            "M2A-GLB-LENGTH-MISMATCH",
            format!(
                "declared length {declared} differs from input length {}",
                input.len()
            ),
        )
        .at(8));
    }
    if input.len() < 20 {
        return Err(
            GlbFatalError::new("M2A-GLB-CHUNK-INVALID", "JSON chunk header is truncated").at(12),
        );
    }
    let json_len = read_u32(input, 12)? as usize;
    if json_len > limits.max_json_chunk_bytes {
        return Err(GlbFatalError::new(
            "M2A-GLB-LIMIT-EXCEEDED",
            "JSON chunk exceeds the configured limit",
        )
        .at(12));
    }
    if read_u32(input, 16)? != 0x4e4f_534a {
        return Err(
            GlbFatalError::new("M2A-GLB-CHUNK-INVALID", "first GLB chunk is not JSON").at(16),
        );
    }
    let json_end = 20_usize.checked_add(json_len).ok_or_else(|| {
        GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "JSON chunk range overflow")
    })?;
    if json_end > input.len() {
        return Err(GlbFatalError::new("M2A-GLB-CHUNK-INVALID", "JSON chunk is truncated").at(12));
    }
    Ok(())
}

#[derive(Debug)]
struct RawPreflight {
    asset_version: String,
    generator: Option<String>,
    vertex_count: usize,
    index_count: usize,
    triangle_count: usize,
    decoded_geometry_bytes: usize,
    diagnostics: Vec<GlbDiagnostic>,
    missing_position_primitives: HashSet<(usize, usize)>,
}

#[derive(Clone, Debug)]
struct RawBufferView {
    byte_offset: usize,
    byte_length: usize,
    byte_stride: Option<usize>,
}

#[derive(Clone, Debug)]
struct RawAccessor {
    count: usize,
    component_type: u64,
    element_type: String,
    normalized: bool,
    component_count: usize,
}

fn glb_payloads(input: &[u8]) -> Result<(&[u8], &[u8]), GlbFatalError> {
    let json_length = read_u32(input, 12)? as usize;
    if !json_length.is_multiple_of(4) {
        return Err(GlbFatalError::new(
            "M2A-GLB-CHUNK-INVALID",
            "JSON chunk is not 4-byte aligned",
        )
        .at(12));
    }
    let json_end = 20_usize.checked_add(json_length).ok_or_else(|| {
        GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "JSON chunk range overflow")
    })?;
    let bin_header_end = json_end.checked_add(8).ok_or_else(|| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            "BIN chunk header range overflow",
        )
    })?;
    if bin_header_end > input.len() {
        return Err(GlbFatalError::new(
            "M2A-GLB-BIN-MISSING",
            "embedded GLB BIN chunk is required",
        )
        .at(json_end));
    }
    let bin_length = read_u32(input, json_end)? as usize;
    if read_u32(input, json_end + 4)? != 0x004e_4942 {
        return Err(
            GlbFatalError::new("M2A-GLB-CHUNK-INVALID", "second GLB chunk is not BIN")
                .at(json_end + 4),
        );
    }
    if !bin_length.is_multiple_of(4) {
        return Err(
            GlbFatalError::new("M2A-GLB-CHUNK-INVALID", "BIN chunk is not 4-byte aligned")
                .at(json_end),
        );
    }
    let bin_end = bin_header_end.checked_add(bin_length).ok_or_else(|| {
        GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "BIN chunk range overflow")
    })?;
    if bin_end != input.len() {
        return Err(GlbFatalError::new(
            "M2A-GLB-CHUNK-INVALID",
            "BIN chunk length does not consume the declared GLB payload",
        )
        .at(json_end));
    }
    Ok((&input[20..json_end], &input[bin_header_end..bin_end]))
}

fn patch_missing_positions_for_validation(
    json_bytes: &[u8],
    blob: &[u8],
    missing: &HashSet<(usize, usize)>,
) -> Result<Vec<u8>, GlbFatalError> {
    let mut root: Value = serde_json::from_slice(json_bytes)
        .map_err(|error| GlbFatalError::new("M2A-GLB-JSON-INVALID", error.to_string()))?;
    let mut validation_blob = blob.to_vec();
    let candidate = array_or_empty(root.get("accessors"))
        .iter()
        .position(|accessor| {
            accessor.get("componentType").and_then(Value::as_u64) == Some(5126)
                && accessor.get("type").and_then(Value::as_str) == Some("VEC3")
                && !accessor
                    .get("normalized")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
        });
    let candidate = match candidate {
        Some(index) => index,
        None => {
            while !validation_blob.len().is_multiple_of(4) {
                validation_blob.push(0);
            }
            let position_offset = validation_blob.len();
            for value in [0.0_f32, 0.0, 0.0] {
                validation_blob.extend_from_slice(&value.to_le_bytes());
            }
            if root.get("bufferViews").is_none() {
                root["bufferViews"] = Value::Array(Vec::new());
            }
            let views = root["bufferViews"].as_array_mut().ok_or_else(|| {
                GlbFatalError::new("M2A-GLB-JSON-INVALID", "bufferViews must be an array")
                    .in_json("bufferViews")
            })?;
            let view_index = views.len();
            views.push(serde_json::json!({
                "buffer": 0,
                "byteOffset": position_offset,
                "byteLength": 12
            }));
            if root.get("accessors").is_none() {
                root["accessors"] = Value::Array(Vec::new());
            }
            let accessors = root["accessors"].as_array_mut().ok_or_else(|| {
                GlbFatalError::new("M2A-GLB-JSON-INVALID", "accessors must be an array")
                    .in_json("accessors")
            })?;
            let index = accessors.len();
            accessors.push(serde_json::json!({
                "bufferView": view_index,
                "componentType": 5126,
                "count": 1,
                "type": "VEC3",
                "min": [0.0, 0.0, 0.0],
                "max": [0.0, 0.0, 0.0]
            }));
            root["buffers"][0]["byteLength"] = Value::from(validation_blob.len() as u64);
            index
        }
    };
    for &(mesh_index, primitive_index) in missing {
        let attributes = root
            .get_mut("meshes")
            .and_then(Value::as_array_mut)
            .and_then(|meshes| meshes.get_mut(mesh_index))
            .and_then(|mesh| mesh.get_mut("primitives"))
            .and_then(Value::as_array_mut)
            .and_then(|primitives| primitives.get_mut(primitive_index))
            .and_then(|primitive| primitive.get_mut("attributes"))
            .and_then(Value::as_object_mut)
            .ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-JSON-INVALID",
                    "primitive attributes must be an object",
                )
                .in_json(format!(
                    "meshes[{mesh_index}].primitives[{primitive_index}].attributes"
                ))
            })?;
        attributes.insert("POSITION".to_owned(), Value::from(candidate as u64));
    }

    let mut json = serde_json::to_vec(&root)
        .map_err(|error| GlbFatalError::new("M2A-GLB-JSON-INVALID", error.to_string()))?;
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let total = 12_usize
        .checked_add(8)
        .and_then(|value| value.checked_add(json.len()))
        .and_then(|value| value.checked_add(8))
        .and_then(|value| value.checked_add(validation_blob.len()))
        .ok_or_else(|| {
            GlbFatalError::new(
                "M2A-GLB-INTEGER-OVERFLOW",
                "patched validation GLB length overflow",
            )
        })?;
    let total_u32 = u32::try_from(total).map_err(|_| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            "patched validation GLB length exceeds u32",
        )
    })?;
    let json_u32 = u32::try_from(json.len()).map_err(|_| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            "patched validation JSON length exceeds u32",
        )
    })?;
    let blob_u32 = u32::try_from(validation_blob.len()).map_err(|_| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            "patched validation BIN length exceeds u32",
        )
    })?;
    let mut output = Vec::with_capacity(total);
    output.extend_from_slice(b"glTF");
    output.extend_from_slice(&2_u32.to_le_bytes());
    output.extend_from_slice(&total_u32.to_le_bytes());
    output.extend_from_slice(&json_u32.to_le_bytes());
    output.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    output.extend_from_slice(&json);
    output.extend_from_slice(&blob_u32.to_le_bytes());
    output.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
    output.extend_from_slice(&validation_blob);
    Ok(output)
}

fn preflight_json(
    json_bytes: &[u8],
    blob: &[u8],
    limits: &GlbLimits,
) -> Result<RawPreflight, GlbFatalError> {
    let root: Value = serde_json::from_slice(json_bytes)
        .map_err(|error| GlbFatalError::new("M2A-GLB-JSON-INVALID", error.to_string()).at(20))?;
    let object = root.as_object().ok_or_else(|| {
        GlbFatalError::new("M2A-GLB-JSON-INVALID", "glTF JSON root must be an object").in_json("$")
    })?;

    reject_unsupported_encodings(&root)?;
    reject_nonfinite_node_transforms(&root)?;
    let diagnostics = optional_extension_diagnostics(&root, limits.max_diagnostics)?;

    let asset = object
        .get("asset")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            GlbFatalError::new("M2A-GLB-JSON-INVALID", "asset object is required").in_json("asset")
        })?;
    let asset_version = asset
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            GlbFatalError::new("M2A-GLB-JSON-INVALID", "asset.version must be a string")
                .in_json("asset.version")
        })?
        .to_owned();
    if asset_version != "2.0" {
        return Err(GlbFatalError::new(
            "M2A-GLB-JSON-INVALID",
            "asset.version must be exactly 2.0 for GLB 2.0",
        )
        .in_json("asset.version"));
    }
    let generator = match asset.get("generator") {
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => {
            return Err(GlbFatalError::new(
                "M2A-GLB-JSON-INVALID",
                "asset.generator must be a string",
            )
            .in_json("asset.generator"));
        }
        None => None,
    };

    for field in [
        "scenes",
        "nodes",
        "meshes",
        "accessors",
        "bufferViews",
        "materials",
        "textures",
        "samplers",
        "images",
        "skins",
        "animations",
        "buffers",
        "extensionsUsed",
        "extensionsRequired",
    ] {
        if object.get(field).is_some_and(|value| !value.is_array()) {
            return Err(GlbFatalError::new(
                "M2A-GLB-JSON-INVALID",
                format!("{field} must be an array"),
            )
            .in_json(field));
        }
    }
    let scenes = array_or_empty(object.get("scenes"));
    let nodes = array_or_empty(object.get("nodes"));
    let meshes = array_or_empty(object.get("meshes"));
    let accessors_json = array_or_empty(object.get("accessors"));
    let views_json = array_or_empty(object.get("bufferViews"));
    let materials = array_or_empty(object.get("materials"));
    let textures = array_or_empty(object.get("textures"));
    let samplers = array_or_empty(object.get("samplers"));
    let images = array_or_empty(object.get("images"));
    let skins = array_or_empty(object.get("skins"));
    let animations = array_or_empty(object.get("animations"));
    check_count(nodes.len(), limits.max_nodes, "nodes")?;
    check_count(meshes.len(), limits.max_meshes, "meshes")?;
    check_count(accessors_json.len(), limits.max_accessors, "accessors")?;
    check_count(views_json.len(), limits.max_buffer_views, "buffer views")?;
    check_count(materials.len(), limits.max_materials, "materials")?;
    check_count(textures.len(), limits.max_textures, "textures")?;
    check_count(samplers.len(), limits.max_samplers, "samplers")?;
    check_count(images.len(), limits.max_images, "images")?;
    check_count(skins.len(), limits.max_skins, "skins")?;
    check_count(animations.len(), limits.max_animations, "animations")?;

    validate_scene_and_node_references(object, scenes, nodes, meshes, skins)?;
    validate_material_texture_sampler_references(materials, textures, samplers, images)?;

    let buffers = array_or_empty(object.get("buffers"));
    if buffers.len() != 1 {
        return Err(GlbFatalError::new(
            "M2A-GLB-BUFFER-VIEW-OOB",
            "GLB v1 subset requires exactly one embedded buffer",
        )
        .in_json("buffers"));
    }
    let declared_buffer_length = json_usize(
        buffers[0].get("byteLength"),
        "buffers[0].byteLength",
        "M2A-GLB-BUFFER-VIEW-OOB",
    )?;
    if declared_buffer_length > blob.len() {
        return Err(GlbFatalError::new(
            "M2A-GLB-BUFFER-VIEW-OOB",
            "declared buffer length exceeds embedded BIN bytes",
        )
        .in_json("buffers[0].byteLength"));
    }
    if blob.len().saturating_sub(declared_buffer_length) > 3 {
        return Err(GlbFatalError::new(
            "M2A-GLB-BUFFER-VIEW-OOB",
            "embedded BIN length differs from buffers[0].byteLength by more than GLB padding",
        )
        .in_json("buffers[0].byteLength"));
    }

    let views = parse_buffer_views(views_json, declared_buffer_length)?;
    let accessors = parse_accessors(accessors_json, &views)?;
    validate_image_limits(images, &views, limits)?;
    validate_skin_animation_preflight(nodes, meshes, skins, animations, &accessors, limits)?;

    let mut primitive_count = 0_usize;
    let mut vertex_count = 0_usize;
    let mut index_count = 0_usize;
    let mut triangle_count = 0_usize;
    let mut decoded_geometry_bytes = 0_usize;
    let mut missing_position_primitives = HashSet::new();
    for (mesh_index, mesh) in meshes.iter().enumerate() {
        let primitives = array_or_empty(mesh.get("primitives"));
        primitive_count = checked_add(primitive_count, primitives.len())?;
        if primitive_count > limits.max_primitives {
            return Err(limit_error(format!(
                "primitives count {primitive_count} exceeds {}",
                limits.max_primitives
            )));
        }
        for (primitive_index, primitive) in primitives.iter().enumerate() {
            let path = format!("meshes[{mesh_index}].primitives[{primitive_index}]");
            let position_accessor = primitive
                .get("attributes")
                .and_then(|value| value.get("POSITION"))
                .map(|value| {
                    json_usize(
                        Some(value),
                        &format!("{path}.attributes.POSITION"),
                        "M2A-GLB-ACCESSOR-OOB",
                    )
                })
                .transpose()?;
            let positions = position_accessor
                .map(|index| {
                    accessors.get(index).ok_or_else(|| {
                        GlbFatalError::new(
                            "M2A-GLB-ACCESSOR-OOB",
                            "POSITION accessor index is out of range",
                        )
                        .in_json(format!("{path}.attributes.POSITION"))
                    })
                })
                .transpose()?;
            if positions.is_none() {
                missing_position_primitives.insert((mesh_index, primitive_index));
            }
            if let Some(accessor) = positions {
                if accessor.component_type != 5126
                    || accessor.element_type != "VEC3"
                    || accessor.normalized
                {
                    return Err(GlbFatalError::new(
                        "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                        "POSITION must use non-normalized FLOAT VEC3",
                    )
                    .in_json(format!("{path}.attributes.POSITION")));
                }
                vertex_count = checked_add(vertex_count, accessor.count)?;
                reserve_decoded_items(&mut decoded_geometry_bytes, accessor.count, 12, limits)?;
            }
            let attributes = primitive.get("attributes").and_then(Value::as_object);
            for (semantic, item_bytes) in [
                ("NORMAL", 12),
                ("TANGENT", 16),
                ("TEXCOORD_0", 8),
                ("JOINTS_0", 8),
                ("WEIGHTS_0", 16),
            ] {
                let Some(value) = attributes.and_then(|attributes| attributes.get(semantic)) else {
                    continue;
                };
                let accessor_index = json_usize(
                    Some(value),
                    &format!("{path}.attributes.{semantic}"),
                    "M2A-GLB-ACCESSOR-OOB",
                )?;
                let accessor = accessors.get(accessor_index).ok_or_else(|| {
                    GlbFatalError::new(
                        "M2A-GLB-ACCESSOR-OOB",
                        format!("{semantic} accessor index is out of range"),
                    )
                    .in_json(format!("{path}.attributes.{semantic}"))
                })?;
                validate_geometry_accessor_layout(semantic, accessor, &path)?;
                reserve_decoded_items(
                    &mut decoded_geometry_bytes,
                    accessor.count,
                    item_bytes,
                    limits,
                )?;
            }
            let primitive_indices = match primitive.get("indices") {
                Some(value) => {
                    let accessor_index = json_usize(
                        Some(value),
                        &format!("{path}.indices"),
                        "M2A-GLB-ACCESSOR-OOB",
                    )?;
                    let accessor = accessors.get(accessor_index).ok_or_else(|| {
                        GlbFatalError::new(
                            "M2A-GLB-ACCESSOR-OOB",
                            "index accessor index is out of range",
                        )
                        .in_json(format!("{path}.indices"))
                    })?;
                    if !matches!(accessor.component_type, 5121 | 5123 | 5125)
                        || accessor.element_type != "SCALAR"
                    {
                        return Err(GlbFatalError::new(
                            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                            "indices must use unsigned integer SCALAR",
                        )
                        .in_json(format!("{path}.indices")));
                    }
                    accessor.count
                }
                None => positions.map_or(0, |accessor| accessor.count),
            };
            if let Some(material) = optional_json_usize(
                primitive.get("material"),
                &format!("{path}.material"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )? && material >= materials.len()
            {
                return Err(layout_error("primitive material reference is out of range")
                    .in_json(format!("{path}.material")));
            }
            index_count = checked_add(index_count, primitive_indices)?;
            reserve_decoded_items(&mut decoded_geometry_bytes, primitive_indices, 4, limits)?;
            let mode = optional_json_usize(
                primitive.get("mode"),
                &format!("{path}.mode"),
                "M2A-GLB-JSON-INVALID",
            )?
            .unwrap_or(4);
            if mode > 6 {
                return Err(GlbFatalError::new(
                    "M2A-GLB-JSON-INVALID",
                    "primitive mode must be a glTF topology value in 0..=6",
                )
                .in_json(format!("{path}.mode")));
            }
            if mode == 4 {
                triangle_count = checked_add(triangle_count, primitive_indices / 3)?;
            }
        }
    }
    if vertex_count > limits.max_vertices {
        return Err(limit_error(format!(
            "asset vertex count {vertex_count} exceeds {}",
            limits.max_vertices
        )));
    }
    if index_count > limits.max_indices {
        return Err(limit_error(format!(
            "asset index count {index_count} exceeds {}",
            limits.max_indices
        )));
    }
    Ok(RawPreflight {
        asset_version,
        generator,
        vertex_count,
        index_count,
        triangle_count,
        decoded_geometry_bytes,
        diagnostics,
        missing_position_primitives,
    })
}

fn validate_geometry_accessor_layout(
    semantic: &str,
    accessor: &RawAccessor,
    primitive_path: &str,
) -> Result<(), GlbFatalError> {
    let valid = match semantic {
        "NORMAL" => {
            accessor.component_type == 5126
                && accessor.element_type == "VEC3"
                && !accessor.normalized
        }
        "TANGENT" => {
            accessor.component_type == 5126
                && accessor.element_type == "VEC4"
                && !accessor.normalized
        }
        "TEXCOORD_0" => {
            accessor.element_type == "VEC2"
                && ((accessor.component_type == 5126 && !accessor.normalized)
                    || (matches!(accessor.component_type, 5121 | 5123) && accessor.normalized))
        }
        "JOINTS_0" => {
            accessor.element_type == "VEC4"
                && matches!(accessor.component_type, 5121 | 5123)
                && !accessor.normalized
        }
        "WEIGHTS_0" => {
            accessor.element_type == "VEC4"
                && ((accessor.component_type == 5126 && !accessor.normalized)
                    || (matches!(accessor.component_type, 5121 | 5123) && accessor.normalized))
        }
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(GlbFatalError::new(
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            format!("{semantic} accessor has an invalid component/type/normalized layout"),
        )
        .in_json(format!("{primitive_path}.attributes.{semantic}")))
    }
}

fn optional_extension_diagnostics(
    root: &Value,
    max_diagnostics: usize,
) -> Result<Vec<GlbDiagnostic>, GlbFatalError> {
    let used = array_or_empty(root.get("extensionsUsed"));
    let required = array_or_empty(root.get("extensionsRequired"));
    let required = required
        .iter()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let mut diagnostics = Vec::with_capacity(used.len().min(max_diagnostics));
    for (index, value) in used.iter().enumerate() {
        let name = value.as_str().ok_or_else(|| {
            GlbFatalError::new(
                "M2A-GLB-JSON-INVALID",
                "extensionsUsed entries must be strings",
            )
            .in_json(format!("extensionsUsed[{index}]"))
        })?;
        if required.contains(name) || is_compression_extension(name) {
            continue;
        }
        if diagnostics.len() < max_diagnostics {
            diagnostics.push(GlbDiagnostic {
                schema_version: GLB_SCHEMA_VERSION,
                code: "M2A-GLB-OPTIONAL-EXTENSION-IGNORED".to_owned(),
                severity: "WARNING".to_owned(),
                byte_offset: None,
                json_path: Some(format!("extensionsUsed[{index}]")),
                message: format!("optional extension {name} is preserved as inventory only"),
            });
        }
    }
    Ok(diagnostics)
}

fn validate_scene_and_node_references(
    root: &serde_json::Map<String, Value>,
    scenes: &[Value],
    nodes: &[Value],
    meshes: &[Value],
    skins: &[Value],
) -> Result<(), GlbFatalError> {
    if let Some(scene) = optional_json_usize(
        root.get("scene"),
        "scene",
        "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
    )? && scene >= scenes.len()
    {
        return Err(layout_error("default scene reference is out of range").in_json("scene"));
    }
    for (scene_index, scene) in scenes.iter().enumerate() {
        let roots = match scene.get("nodes") {
            Some(Value::Array(values)) => values.as_slice(),
            Some(_) => {
                return Err(layout_error("scene nodes must be an array")
                    .in_json(format!("scenes[{scene_index}].nodes")));
            }
            None => &[],
        };
        for (root_index, root_node) in roots.iter().enumerate() {
            let node = json_usize(
                Some(root_node),
                &format!("scenes[{scene_index}].nodes[{root_index}]"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            if node >= nodes.len() {
                return Err(layout_error("scene root node reference is out of range")
                    .in_json(format!("scenes[{scene_index}].nodes[{root_index}]")));
            }
        }
    }
    for (node_index, node) in nodes.iter().enumerate() {
        if !node.is_object() {
            return Err(
                layout_error("node must be an object").in_json(format!("nodes[{node_index}]"))
            );
        }
        if let Some(children) = node.get("children") {
            let children = children.as_array().ok_or_else(|| {
                layout_error("node children must be an array")
                    .in_json(format!("nodes[{node_index}].children"))
            })?;
            for (child_index, child) in children.iter().enumerate() {
                let referenced = json_usize(
                    Some(child),
                    &format!("nodes[{node_index}].children[{child_index}]"),
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                )?;
                if referenced >= nodes.len() {
                    return Err(layout_error("node child reference is out of range")
                        .in_json(format!("nodes[{node_index}].children[{child_index}]")));
                }
            }
        }
        if let Some(mesh) = optional_json_usize(
            node.get("mesh"),
            &format!("nodes[{node_index}].mesh"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? && mesh >= meshes.len()
        {
            return Err(layout_error("node mesh reference is out of range")
                .in_json(format!("nodes[{node_index}].mesh")));
        }
        if let Some(skin) = optional_json_usize(
            node.get("skin"),
            &format!("nodes[{node_index}].skin"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? && skin >= skins.len()
        {
            return Err(layout_error("node skin reference is out of range")
                .in_json(format!("nodes[{node_index}].skin")));
        }
    }
    Ok(())
}

fn validate_material_texture_sampler_references(
    materials: &[Value],
    textures: &[Value],
    samplers: &[Value],
    images: &[Value],
) -> Result<(), GlbFatalError> {
    for (index, sampler) in samplers.iter().enumerate() {
        for (field, allowed) in [
            ("magFilter", &[9728_usize, 9729][..]),
            ("minFilter", &[9728_usize, 9729, 9984, 9985, 9986, 9987][..]),
            ("wrapS", &[33071_usize, 33648, 10497][..]),
            ("wrapT", &[33071_usize, 33648, 10497][..]),
        ] {
            let Some(value) = optional_json_usize(
                sampler.get(field),
                &format!("samplers[{index}].{field}"),
                "M2A-GLB-JSON-INVALID",
            )?
            else {
                continue;
            };
            if !allowed.contains(&value) {
                return Err(GlbFatalError::new(
                    "M2A-GLB-JSON-INVALID",
                    format!("sampler {field} value {value} is invalid"),
                )
                .in_json(format!("samplers[{index}].{field}")));
            }
        }
    }
    for (index, texture) in textures.iter().enumerate() {
        let source = json_usize(
            texture.get("source"),
            &format!("textures[{index}].source"),
            "M2A-GLB-JSON-INVALID",
        )?;
        if source >= images.len() {
            return Err(GlbFatalError::new(
                "M2A-GLB-JSON-INVALID",
                "texture image source reference is out of range",
            )
            .in_json(format!("textures[{index}].source")));
        }
        if let Some(sampler) = optional_json_usize(
            texture.get("sampler"),
            &format!("textures[{index}].sampler"),
            "M2A-GLB-JSON-INVALID",
        )? && sampler >= samplers.len()
        {
            return Err(GlbFatalError::new(
                "M2A-GLB-JSON-INVALID",
                "texture sampler reference is out of range",
            )
            .in_json(format!("textures[{index}].sampler")));
        }
    }
    for (material_index, material) in materials.iter().enumerate() {
        let pbr = material.get("pbrMetallicRoughness");
        for (prefix, binding) in [
            (
                "pbrMetallicRoughness.baseColorTexture",
                pbr.and_then(|value| value.get("baseColorTexture")),
            ),
            (
                "pbrMetallicRoughness.metallicRoughnessTexture",
                pbr.and_then(|value| value.get("metallicRoughnessTexture")),
            ),
            ("normalTexture", material.get("normalTexture")),
            ("occlusionTexture", material.get("occlusionTexture")),
            ("emissiveTexture", material.get("emissiveTexture")),
        ] {
            let Some(binding) = binding else {
                continue;
            };
            let path = format!("materials[{material_index}].{prefix}.index");
            let texture = json_usize(binding.get("index"), &path, "M2A-GLB-JSON-INVALID")?;
            if texture >= textures.len() {
                return Err(GlbFatalError::new(
                    "M2A-GLB-JSON-INVALID",
                    "material texture reference is out of range",
                )
                .in_json(path));
            }
        }
    }
    Ok(())
}

fn reject_unsupported_encodings(root: &Value) -> Result<(), GlbFatalError> {
    for (index, buffer) in array_or_empty(root.get("buffers")).iter().enumerate() {
        if buffer.get("uri").is_some() {
            return Err(GlbFatalError::new(
                "M2A-GLB-EXTERNAL-URI-UNSUPPORTED",
                "external buffer URI is unsupported",
            )
            .in_json(format!("buffers[{index}].uri")));
        }
    }
    for (index, image) in array_or_empty(root.get("images")).iter().enumerate() {
        if image.get("uri").is_some() {
            return Err(GlbFatalError::new(
                "M2A-GLB-EXTERNAL-URI-UNSUPPORTED",
                "external image URI is unsupported",
            )
            .in_json(format!("images[{index}].uri")));
        }
    }
    for (index, accessor) in array_or_empty(root.get("accessors")).iter().enumerate() {
        if accessor.get("sparse").is_some() {
            return Err(GlbFatalError::new(
                "M2A-GLB-SPARSE-ACCESSOR-UNSUPPORTED",
                "sparse accessors are unsupported",
            )
            .in_json(format!("accessors[{index}].sparse")));
        }
    }
    let required_extensions = array_or_empty(root.get("extensionsRequired"));
    let used_extensions = array_or_empty(root.get("extensionsUsed"));
    for (path, extensions) in [
        ("extensionsRequired", required_extensions),
        ("extensionsUsed", used_extensions),
    ] {
        if extensions.iter().any(|extension| !extension.is_string()) {
            return Err(GlbFatalError::new(
                "M2A-GLB-JSON-INVALID",
                format!("{path} entries must be strings"),
            )
            .in_json(path));
        }
    }
    if let Some(name) = required_extensions
        .iter()
        .chain(used_extensions)
        .filter_map(Value::as_str)
        .find(|name| is_compression_extension(name))
    {
        return Err(GlbFatalError::new(
            "M2A-GLB-COMPRESSION-UNSUPPORTED",
            format!("geometry compression extension {name} is unsupported"),
        )
        .in_json("extensionsUsed"));
    }
    if let Some(name) = required_extensions.first().and_then(Value::as_str) {
        return Err(GlbFatalError::new(
            "M2A-GLB-REQUIRED-EXTENSION-UNSUPPORTED",
            format!("required extension {name} is unsupported"),
        )
        .in_json("extensionsRequired"));
    }
    for (mesh_index, mesh) in array_or_empty(root.get("meshes")).iter().enumerate() {
        for (primitive_index, primitive) in
            array_or_empty(mesh.get("primitives")).iter().enumerate()
        {
            if let Some(extensions) = primitive.get("extensions").and_then(Value::as_object)
                && let Some(name) = extensions
                    .keys()
                    .find(|name| is_compression_extension(name))
            {
                return Err(GlbFatalError::new(
                    "M2A-GLB-COMPRESSION-UNSUPPORTED",
                    format!("geometry compression extension {name} is unsupported"),
                )
                .in_json(format!(
                    "meshes[{mesh_index}].primitives[{primitive_index}].extensions.{name}"
                )));
            }
        }
    }
    for (view_index, view) in array_or_empty(root.get("bufferViews")).iter().enumerate() {
        if let Some(extensions) = view.get("extensions").and_then(Value::as_object)
            && let Some(name) = extensions
                .keys()
                .find(|name| is_compression_extension(name))
        {
            return Err(GlbFatalError::new(
                "M2A-GLB-COMPRESSION-UNSUPPORTED",
                format!("geometry compression extension {name} is unsupported"),
            )
            .in_json(format!("bufferViews[{view_index}].extensions.{name}")));
        }
    }
    Ok(())
}

fn is_compression_extension(name: &str) -> bool {
    matches!(
        name,
        "KHR_draco_mesh_compression" | "EXT_meshopt_compression"
    )
}

fn reject_nonfinite_node_transforms(root: &Value) -> Result<(), GlbFatalError> {
    for (node_index, node) in array_or_empty(root.get("nodes")).iter().enumerate() {
        for field in ["matrix", "translation", "rotation", "scale"] {
            let Some(values) = node.get(field).and_then(Value::as_array) else {
                continue;
            };
            for value in values {
                let Some(value) = value.as_f64() else {
                    continue;
                };
                if !value.is_finite() || !(value as f32).is_finite() {
                    return Err(GlbFatalError::new(
                        "M2A-GLB-NONFINITE-FLOAT",
                        "node transform contains a non-finite f32 value",
                    )
                    .in_json(format!("nodes[{node_index}].{field}")));
                }
            }
        }
    }
    Ok(())
}

fn parse_buffer_views(
    values: &[Value],
    buffer_length: usize,
) -> Result<Vec<RawBufferView>, GlbFatalError> {
    let mut output = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let prefix = format!("bufferViews[{index}]");
        let buffer = json_usize(
            value.get("buffer"),
            &format!("{prefix}.buffer"),
            "M2A-GLB-BUFFER-VIEW-OOB",
        )?;
        if buffer != 0 {
            return Err(GlbFatalError::new(
                "M2A-GLB-BUFFER-VIEW-OOB",
                "buffer view references a non-embedded buffer",
            )
            .in_json(format!("{prefix}.buffer")));
        }
        let byte_offset = optional_json_usize(
            value.get("byteOffset"),
            &format!("{prefix}.byteOffset"),
            "M2A-GLB-BUFFER-VIEW-OOB",
        )?
        .unwrap_or(0);
        let byte_length = json_usize(
            value.get("byteLength"),
            &format!("{prefix}.byteLength"),
            "M2A-GLB-BUFFER-VIEW-OOB",
        )?;
        let end = byte_offset.checked_add(byte_length).ok_or_else(|| {
            GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "buffer view range overflows")
                .in_json(prefix.clone())
        })?;
        if end > buffer_length {
            return Err(GlbFatalError::new(
                "M2A-GLB-BUFFER-VIEW-OOB",
                "buffer view exceeds embedded buffer bounds",
            )
            .in_json(prefix));
        }
        let byte_stride = optional_json_usize(
            value.get("byteStride"),
            &format!("bufferViews[{index}].byteStride"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )?;
        if let Some(stride) = byte_stride
            && (!(4..=252).contains(&stride) || !stride.is_multiple_of(4))
        {
            return Err(GlbFatalError::new(
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                "buffer view stride must be a 4-byte multiple in 4..=252",
            )
            .in_json(format!("bufferViews[{index}].byteStride")));
        }
        output.push(RawBufferView {
            byte_offset,
            byte_length,
            byte_stride,
        });
    }
    Ok(output)
}

fn parse_accessors(
    values: &[Value],
    views: &[RawBufferView],
) -> Result<Vec<RawAccessor>, GlbFatalError> {
    let mut output = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let prefix = format!("accessors[{index}]");
        let component_type = json_usize(
            value.get("componentType"),
            &format!("{prefix}.componentType"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? as u64;
        let component_size: usize = match component_type {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            5125 | 5126 => 4,
            _ => {
                return Err(GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                    "accessor componentType is unsupported by glTF 2.0",
                )
                .in_json(format!("{prefix}.componentType")));
            }
        };
        let element_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                    "accessor type must be a string",
                )
                .in_json(format!("{prefix}.type"))
            })?
            .to_owned();
        let component_count: usize = match element_type.as_str() {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" | "MAT2" => 4,
            "MAT3" => 9,
            "MAT4" => 16,
            _ => {
                return Err(GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                    "accessor type is unsupported by glTF 2.0",
                )
                .in_json(format!("{prefix}.type")));
            }
        };
        let element_size = component_size.checked_mul(component_count).ok_or_else(|| {
            GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "accessor element size overflow")
        })?;
        let count = json_usize(
            value.get("count"),
            &format!("{prefix}.count"),
            "M2A-GLB-ACCESSOR-OOB",
        )?;
        let byte_offset = optional_json_usize(
            value.get("byteOffset"),
            &format!("{prefix}.byteOffset"),
            "M2A-GLB-ACCESSOR-OOB",
        )?
        .unwrap_or(0);
        let buffer_view = optional_json_usize(
            value.get("bufferView"),
            &format!("{prefix}.bufferView"),
            "M2A-GLB-ACCESSOR-OOB",
        )?;
        if let Some(view_index) = buffer_view {
            let view = views.get(view_index).ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-OOB",
                    "accessor bufferView index is out of range",
                )
                .in_json(format!("{prefix}.bufferView"))
            })?;
            let absolute_offset = view.byte_offset.checked_add(byte_offset).ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-INTEGER-OVERFLOW",
                    "accessor absolute byte offset overflows",
                )
                .in_json(prefix.clone())
            })?;
            let stride = view.byte_stride.unwrap_or(element_size);
            if stride < element_size || !stride.is_multiple_of(component_size) {
                return Err(GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                    "accessor stride cannot contain its element layout",
                )
                .in_json(prefix.clone()));
            }
            let occupied = if count == 0 {
                0
            } else {
                count
                    .checked_sub(1)
                    .and_then(|value| value.checked_mul(stride))
                    .and_then(|value| value.checked_add(element_size))
                    .ok_or_else(|| {
                        GlbFatalError::new(
                            "M2A-GLB-INTEGER-OVERFLOW",
                            "accessor byte range overflows",
                        )
                        .in_json(prefix.clone())
                    })?
            };
            let end = byte_offset.checked_add(occupied).ok_or_else(|| {
                GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "accessor byte range overflows")
                    .in_json(prefix.clone())
            })?;
            if end > view.byte_length {
                return Err(GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-OOB",
                    "accessor exceeds its buffer view",
                )
                .in_json(prefix.clone()));
            }
            if !byte_offset.is_multiple_of(component_size)
                || !absolute_offset.is_multiple_of(component_size)
            {
                return Err(GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                    "accessor byte offset is not component-aligned",
                )
                .in_json(prefix.clone()));
            }
        } else if count != 0 {
            return Err(GlbFatalError::new(
                "M2A-GLB-ACCESSOR-OOB",
                "non-empty accessor has no bufferView",
            )
            .in_json(format!("{prefix}.bufferView")));
        }
        output.push(RawAccessor {
            count,
            component_type,
            element_type,
            normalized: value
                .get("normalized")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            component_count,
        });
    }
    Ok(output)
}

fn validate_image_limits(
    images: &[Value],
    views: &[RawBufferView],
    limits: &GlbLimits,
) -> Result<(), GlbFatalError> {
    let mut total = 0_usize;
    for (index, image) in images.iter().enumerate() {
        let mime_path = format!("images[{index}].mimeType");
        let mime_type = image
            .get("mimeType")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-IMAGE-MIME-UNSUPPORTED",
                    "embedded image mimeType must be image/png or image/jpeg",
                )
                .in_json(mime_path.clone())
            })?;
        if !matches!(mime_type, "image/png" | "image/jpeg") {
            return Err(GlbFatalError::new(
                "M2A-GLB-IMAGE-MIME-UNSUPPORTED",
                format!("embedded image mimeType {mime_type:?} is unsupported"),
            )
            .in_json(mime_path));
        }
        let view_index = json_usize(
            image.get("bufferView"),
            &format!("images[{index}].bufferView"),
            "M2A-GLB-BUFFER-VIEW-OOB",
        )?;
        let view = views.get(view_index).ok_or_else(|| {
            GlbFatalError::new(
                "M2A-GLB-BUFFER-VIEW-OOB",
                "image bufferView index is out of range",
            )
            .in_json(format!("images[{index}].bufferView"))
        })?;
        if view.byte_length > limits.max_single_image_bytes {
            return Err(limit_error(format!(
                "image {index} length {} exceeds {}",
                view.byte_length, limits.max_single_image_bytes
            )));
        }
        total = checked_add(total, view.byte_length)?;
        if total > limits.max_total_image_bytes {
            return Err(limit_error(format!(
                "total image bytes {total} exceeds {}",
                limits.max_total_image_bytes
            )));
        }
    }
    Ok(())
}

fn validate_skin_animation_preflight(
    nodes: &[Value],
    meshes: &[Value],
    skins: &[Value],
    animations: &[Value],
    accessors: &[RawAccessor],
    limits: &GlbLimits,
) -> Result<(), GlbFatalError> {
    let mut decoded_bytes = 0_usize;
    let mut total_joints = 0_usize;

    for (node_index, node) in nodes.iter().enumerate() {
        if let Some(mesh) = optional_json_usize(
            node.get("mesh"),
            &format!("nodes[{node_index}].mesh"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? && mesh >= meshes.len()
        {
            return Err(layout_error("node mesh reference is out of range")
                .in_json(format!("nodes[{node_index}].mesh")));
        }
        if let Some(skin) = optional_json_usize(
            node.get("skin"),
            &format!("nodes[{node_index}].skin"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? && skin >= skins.len()
        {
            return Err(layout_error("node skin reference is out of range")
                .in_json(format!("nodes[{node_index}].skin")));
        }
    }

    for (skin_index, skin) in skins.iter().enumerate() {
        let path = format!("skins[{skin_index}]");
        let joints = skin
            .get("joints")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                layout_error("skin joints must be a non-empty array")
                    .in_json(format!("{path}.joints"))
            })?;
        if joints.is_empty() {
            return Err(layout_error("skin joints must be a non-empty array")
                .in_json(format!("{path}.joints")));
        }
        total_joints = checked_add(total_joints, joints.len())?;
        check_count(total_joints, limits.max_joints, "joints")?;
        reserve_skin_animation_items(
            &mut decoded_bytes,
            joints.len(),
            4,
            limits,
            "skin joint IDs",
        )?;
        for (joint_index, joint) in joints.iter().enumerate() {
            let node = json_usize(
                Some(joint),
                &format!("{path}.joints[{joint_index}]"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            if node >= nodes.len() {
                return Err(layout_error("skin joint node reference is out of range")
                    .in_json(format!("{path}.joints[{joint_index}]")));
            }
        }
        if let Some(skeleton) = optional_json_usize(
            skin.get("skeleton"),
            &format!("{path}.skeleton"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? && skeleton >= nodes.len()
        {
            return Err(layout_error("skin skeleton node reference is out of range")
                .in_json(format!("{path}.skeleton")));
        }
        if let Some(accessor_index) = optional_json_usize(
            skin.get("inverseBindMatrices"),
            &format!("{path}.inverseBindMatrices"),
            "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
        )? {
            let accessor = accessors.get(accessor_index).ok_or_else(|| {
                layout_error("inverse bind matrix accessor is out of range")
                    .in_json(format!("{path}.inverseBindMatrices"))
            })?;
            if accessor.component_type != 5126
                || accessor.element_type != "MAT4"
                || accessor.normalized
            {
                return Err(layout_error(
                    "inverse bind matrices must use non-normalized FLOAT MAT4",
                )
                .in_json(format!("{path}.inverseBindMatrices")));
            }
            if accessor.count < joints.len() {
                return Err(
                    layout_error("inverse bind matrix count must cover every skin joint")
                        .in_json(format!("{path}.inverseBindMatrices")),
                );
            }
            reserve_skin_animation_items(
                &mut decoded_bytes,
                accessor.count,
                64,
                limits,
                "inverse bind matrices",
            )?;
        }
    }

    for (mesh_index, mesh) in meshes.iter().enumerate() {
        for (primitive_index, primitive) in
            array_or_empty(mesh.get("primitives")).iter().enumerate()
        {
            let path = format!("meshes[{mesh_index}].primitives[{primitive_index}].attributes");
            let attributes = primitive.get("attributes").and_then(Value::as_object);
            let joints = attributes.and_then(|value| value.get("JOINTS_0"));
            let weights = attributes.and_then(|value| value.get("WEIGHTS_0"));
            if joints.is_some() != weights.is_some() {
                return Err(
                    layout_error("JOINTS_0 and WEIGHTS_0 must be provided as a pair").in_json(path),
                );
            }
            let (Some(joints), Some(weights)) = (joints, weights) else {
                continue;
            };
            let joints_index = json_usize(
                Some(joints),
                &format!("{path}.JOINTS_0"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            let weights_index = json_usize(
                Some(weights),
                &format!("{path}.WEIGHTS_0"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            let joints_accessor = accessors.get(joints_index).ok_or_else(|| {
                layout_error("JOINTS_0 accessor is out of range")
                    .in_json(format!("{path}.JOINTS_0"))
            })?;
            let weights_accessor = accessors.get(weights_index).ok_or_else(|| {
                layout_error("WEIGHTS_0 accessor is out of range")
                    .in_json(format!("{path}.WEIGHTS_0"))
            })?;
            if !matches!(joints_accessor.component_type, 5121 | 5123)
                || joints_accessor.element_type != "VEC4"
                || joints_accessor.normalized
            {
                return Err(layout_error(
                    "JOINTS_0 must use non-normalized UNSIGNED_BYTE/UNSIGNED_SHORT VEC4",
                )
                .in_json(format!("{path}.JOINTS_0")));
            }
            let valid_weights = (weights_accessor.component_type == 5126
                && !weights_accessor.normalized)
                || (matches!(weights_accessor.component_type, 5121 | 5123)
                    && weights_accessor.normalized);
            if !valid_weights || weights_accessor.element_type != "VEC4" {
                return Err(layout_error(
                    "WEIGHTS_0 must use FLOAT VEC4 or normalized unsigned integer VEC4",
                )
                .in_json(format!("{path}.WEIGHTS_0")));
            }
            if joints_accessor.count != weights_accessor.count {
                return Err(
                    layout_error("JOINTS_0 and WEIGHTS_0 counts must match").in_json(path.clone())
                );
            }
            if let Some(position) = attributes.and_then(|value| value.get("POSITION")) {
                let position_index = json_usize(
                    Some(position),
                    &format!("{path}.POSITION"),
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                )?;
                let position_accessor = accessors.get(position_index).ok_or_else(|| {
                    layout_error("POSITION accessor is out of range")
                        .in_json(format!("{path}.POSITION"))
                })?;
                if joints_accessor.count != position_accessor.count {
                    return Err(
                        layout_error("skinning attribute count must match POSITION count")
                            .in_json(path.clone()),
                    );
                }
            }
            reserve_skin_animation_items(
                &mut decoded_bytes,
                joints_accessor.count,
                8,
                limits,
                "decoded JOINTS_0",
            )?;
            reserve_skin_animation_items(
                &mut decoded_bytes,
                weights_accessor.count,
                16,
                limits,
                "decoded WEIGHTS_0",
            )?;
        }
    }

    let mut total_samplers = 0_usize;
    let mut total_channels = 0_usize;
    let mut total_keyframes = 0_usize;
    for (animation_index, animation) in animations.iter().enumerate() {
        let path = format!("animations[{animation_index}]");
        let samplers = animation
            .get("samplers")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                layout_error("animation samplers must be an array")
                    .in_json(format!("{path}.samplers"))
            })?;
        let channels = animation
            .get("channels")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                layout_error("animation channels must be an array")
                    .in_json(format!("{path}.channels"))
            })?;
        total_samplers = checked_add(total_samplers, samplers.len())?;
        total_channels = checked_add(total_channels, channels.len())?;
        check_count(
            total_samplers,
            limits.max_animation_samplers,
            "animation samplers",
        )?;
        check_count(
            total_channels,
            limits.max_animation_channels,
            "animation channels",
        )?;

        let mut sampler_accessors = Vec::with_capacity(samplers.len());
        for (sampler_index, sampler) in samplers.iter().enumerate() {
            let sampler_path = format!("{path}.samplers[{sampler_index}]");
            let input_index = json_usize(
                sampler.get("input"),
                &format!("{sampler_path}.input"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            let output_index = json_usize(
                sampler.get("output"),
                &format!("{sampler_path}.output"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            let input = accessors.get(input_index).ok_or_else(|| {
                layout_error("animation input accessor is out of range")
                    .in_json(format!("{sampler_path}.input"))
            })?;
            let output = accessors.get(output_index).ok_or_else(|| {
                layout_error("animation output accessor is out of range")
                    .in_json(format!("{sampler_path}.output"))
            })?;
            if input.component_type != 5126
                || input.element_type != "SCALAR"
                || input.normalized
                || input.count == 0
            {
                return Err(layout_error(
                    "animation input must use non-empty non-normalized FLOAT SCALAR",
                )
                .in_json(format!("{sampler_path}.input")));
            }
            if output.component_type != 5126 || output.normalized {
                return Err(
                    layout_error("animation output must use non-normalized FLOAT values")
                        .in_json(format!("{sampler_path}.output")),
                );
            }
            let interpolation = sampler
                .get("interpolation")
                .map(|value| value.as_str())
                .unwrap_or(Some("LINEAR"));
            if !matches!(interpolation, Some("LINEAR" | "STEP" | "CUBICSPLINE")) {
                return Err(layout_error("animation interpolation is unsupported")
                    .in_json(format!("{sampler_path}.interpolation")));
            }
            total_keyframes = checked_add(total_keyframes, input.count)?;
            check_count(total_keyframes, limits.max_keyframes, "animation keyframes")?;
            reserve_skin_animation_items(
                &mut decoded_bytes,
                input.count,
                4,
                limits,
                "animation input times",
            )?;
            let output_item_bytes = output.component_count.checked_mul(4).ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-INTEGER-OVERFLOW",
                    "animation output element size overflow",
                )
            })?;
            reserve_skin_animation_items(
                &mut decoded_bytes,
                output.count,
                output_item_bytes,
                limits,
                "animation output values",
            )?;
            sampler_accessors.push((input, output, interpolation.unwrap_or("LINEAR")));
        }

        let mut targeted_paths = HashSet::new();
        for (channel_index, channel) in channels.iter().enumerate() {
            let channel_path = format!("{path}.channels[{channel_index}]");
            let sampler_index = json_usize(
                channel.get("sampler"),
                &format!("{channel_path}.sampler"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            let (input, output, interpolation) = sampler_accessors
                .get(sampler_index)
                .copied()
                .ok_or_else(|| {
                    layout_error("animation channel sampler reference is out of range")
                        .in_json(format!("{channel_path}.sampler"))
                })?;
            let target = channel
                .get("target")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    layout_error("animation channel target must be an object")
                        .in_json(format!("{channel_path}.target"))
                })?;
            let node_index = json_usize(
                target.get("node"),
                &format!("{channel_path}.target.node"),
                "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
            )?;
            if node_index >= nodes.len() {
                return Err(
                    layout_error("animation target node reference is out of range")
                        .in_json(format!("{channel_path}.target.node")),
                );
            }
            let target_path = target.get("path").and_then(Value::as_str).ok_or_else(|| {
                layout_error("animation target path is missing")
                    .in_json(format!("{channel_path}.target.path"))
            })?;
            if !matches!(
                target_path,
                "translation" | "rotation" | "scale" | "weights"
            ) {
                return Err(layout_error("animation target path is unsupported")
                    .in_json(format!("{channel_path}.target.path")));
            }
            if !targeted_paths.insert((node_index, target_path.to_owned())) {
                return Err(
                    layout_error("animation has duplicate target node/path channels")
                        .in_json(format!("{channel_path}.target")),
                );
            }
            if target_path != "weights" {
                let expected_type = if target_path == "rotation" {
                    "VEC4"
                } else {
                    "VEC3"
                };
                if output.element_type != expected_type {
                    return Err(layout_error(format!(
                        "{target_path} animation output must use FLOAT {expected_type}"
                    ))
                    .in_json(format!("{channel_path}.sampler")));
                }
                let expected_count = if interpolation == "CUBICSPLINE" {
                    input.count.checked_mul(3).ok_or_else(|| {
                        GlbFatalError::new(
                            "M2A-GLB-INTEGER-OVERFLOW",
                            "CUBICSPLINE output count overflow",
                        )
                        .in_json(format!("{channel_path}.sampler"))
                    })?
                } else {
                    input.count
                };
                if output.count != expected_count {
                    return Err(layout_error(
                        "animation output count does not match input/interpolation",
                    )
                    .in_json(format!("{channel_path}.sampler")));
                }
            }
        }
    }
    Ok(())
}

fn reserve_skin_animation_items(
    decoded_bytes: &mut usize,
    item_count: usize,
    item_bytes: usize,
    limits: &GlbLimits,
    label: &str,
) -> Result<(), GlbFatalError> {
    let bytes = item_count.checked_mul(item_bytes).ok_or_else(|| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            format!("{label} decoded size overflow"),
        )
    })?;
    *decoded_bytes = decoded_bytes.checked_add(bytes).ok_or_else(|| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            "cumulative decoded skin/animation size overflow",
        )
    })?;
    if *decoded_bytes > limits.max_decoded_skin_animation_bytes {
        return Err(limit_error(format!(
            "decoded skin/animation materialization {} exceeds {}",
            *decoded_bytes, limits.max_decoded_skin_animation_bytes
        )));
    }
    Ok(())
}

fn layout_error(message: impl Into<String>) -> GlbFatalError {
    GlbFatalError::new("M2A-GLB-ACCESSOR-LAYOUT-INVALID", message)
}

fn array_or_empty(value: Option<&Value>) -> &[Value] {
    value.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn json_usize(value: Option<&Value>, path: &str, code: &str) -> Result<usize, GlbFatalError> {
    optional_json_usize(value, path, code)?.ok_or_else(|| {
        GlbFatalError::new(code, "required unsigned integer is missing").in_json(path)
    })
}

fn optional_json_usize(
    value: Option<&Value>,
    path: &str,
    code: &str,
) -> Result<Option<usize>, GlbFatalError> {
    value
        .map(|value| {
            let value = value.as_u64().ok_or_else(|| {
                GlbFatalError::new(code, "value must be an unsigned integer").in_json(path)
            })?;
            let canonical = u32::try_from(value).map_err(|_| {
                GlbFatalError::new(
                    "M2A-GLB-INTEGER-OVERFLOW",
                    "unsigned GLB integer exceeds the canonical u32 domain",
                )
                .in_json(path)
            })?;
            usize::try_from(canonical).map_err(|_| {
                GlbFatalError::new(
                    "M2A-GLB-INTEGER-OVERFLOW",
                    "canonical u32 GLB integer does not fit the host index domain",
                )
                .in_json(path)
            })
        })
        .transpose()
}

fn check_count(count: usize, limit: usize, label: &str) -> Result<(), GlbFatalError> {
    if count > limit {
        return Err(limit_error(format!(
            "{label} count {count} exceeds {limit}"
        )));
    }
    Ok(())
}

fn limit_error(message: impl Into<String>) -> GlbFatalError {
    GlbFatalError::new("M2A-GLB-LIMIT-EXCEEDED", message)
}

fn flatten_matrix(matrix: [[f32; 4]; 4]) -> [f32; 16] {
    [
        matrix[0][0],
        matrix[0][1],
        matrix[0][2],
        matrix[0][3],
        matrix[1][0],
        matrix[1][1],
        matrix[1][2],
        matrix[1][3],
        matrix[2][0],
        matrix[2][1],
        matrix[2][2],
        matrix[2][3],
        matrix[3][0],
        matrix[3][1],
        matrix[3][2],
        matrix[3][3],
    ]
}

fn validate_document_limits(
    document: &gltf::Document,
    limits: &GlbLimits,
) -> Result<(), GlbFatalError> {
    for (count, limit, label) in [
        (document.nodes().count(), limits.max_nodes, "nodes"),
        (document.meshes().count(), limits.max_meshes, "meshes"),
        (
            document
                .meshes()
                .map(|mesh| mesh.primitives().count())
                .sum(),
            limits.max_primitives,
            "primitives",
        ),
        (
            document.accessors().count(),
            limits.max_accessors,
            "accessors",
        ),
        (
            document.views().count(),
            limits.max_buffer_views,
            "buffer views",
        ),
        (
            document.materials().count(),
            limits.max_materials,
            "materials",
        ),
        (document.textures().count(), limits.max_textures, "textures"),
        (document.samplers().count(), limits.max_samplers, "samplers"),
        (document.images().count(), limits.max_images, "images"),
        (document.skins().count(), limits.max_skins, "skins"),
        (
            document.animations().count(),
            limits.max_animations,
            "animations",
        ),
    ] {
        if count > limit {
            return Err(GlbFatalError::new(
                "M2A-GLB-LIMIT-EXCEEDED",
                format!("{label} count {count} exceeds {limit}"),
            ));
        }
    }
    Ok(())
}

fn validate_node_graph(document: &gltf::Document, limits: &GlbLimits) -> Result<(), GlbFatalError> {
    let children = document
        .nodes()
        .map(|node| {
            node.children()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    fn visit(
        node: usize,
        depth: usize,
        children: &[Vec<usize>],
        visiting: &mut [bool],
        limits: &GlbLimits,
    ) -> Result<(), GlbFatalError> {
        if depth > limits.max_node_depth {
            return Err(GlbFatalError::new(
                "M2A-GLB-LIMIT-EXCEEDED",
                "node depth exceeds limit",
            ));
        }
        if visiting[node] {
            return Err(GlbFatalError::new(
                "M2A-GLB-NODE-CYCLE",
                "node graph contains a cycle",
            ));
        }
        visiting[node] = true;
        for &child in &children[node] {
            visit(child, depth + 1, children, visiting, limits)?;
        }
        visiting[node] = false;
        Ok(())
    }
    let mut visiting = vec![false; children.len()];
    for node in 0..children.len() {
        visit(node, 0, &children, &mut visiting, limits)?;
    }
    Ok(())
}

fn decode_skins(document: &gltf::Document, blob: &[u8]) -> Result<Vec<IrSkin>, GlbFatalError> {
    document
        .skins()
        .map(|skin| {
            let inverse_bind_matrices = skin
                .reader(|_| Some(blob))
                .read_inverse_bind_matrices()
                .map(|matrices| {
                    matrices
                        .map(|matrix| {
                            let flat = flatten_matrix(matrix);
                            if flat.iter().any(|value| !value.is_finite()) {
                                return Err(GlbFatalError::new(
                                    "M2A-GLB-NONFINITE-FLOAT",
                                    "inverse bind matrix contains a non-finite value",
                                )
                                .in_json(format!("skins[{}].inverseBindMatrices", skin.index())));
                            }
                            Ok(flat)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();
            Ok(IrSkin {
                id: skin.index() as u32,
                name: skin.name().map(str::to_owned),
                skeleton_root_node_id: skin.skeleton().map(|node| node.index() as u32),
                joint_node_ids: skin.joints().map(|node| node.index() as u32).collect(),
                inverse_bind_matrices,
            })
        })
        .collect()
}

fn validate_skin_joint_lanes(
    nodes: &[IrNode],
    primitives: &[IrPrimitive],
    skins: &[IrSkin],
) -> Result<(), GlbFatalError> {
    for node in nodes {
        let (Some(mesh_id), Some(skin_id)) = (node.mesh_id, node.skin_id) else {
            continue;
        };
        let skin = skins.get(skin_id as usize).ok_or_else(|| {
            layout_error("node skin reference is out of range")
                .in_json(format!("nodes[{}].skin", node.id))
        })?;
        for primitive in primitives
            .iter()
            .filter(|primitive| primitive.source_mesh_id == mesh_id)
        {
            for (vertex_index, lanes) in primitive.joints0.iter().enumerate() {
                if let Some(lane) = lanes
                    .iter()
                    .copied()
                    .find(|lane| *lane as usize >= skin.joint_node_ids.len())
                {
                    return Err(layout_error(format!(
                        "JOINTS_0 lane {lane} is outside skin {} joint domain",
                        skin.id
                    ))
                    .in_json(format!(
                        "meshes[{mesh_id}].primitives[{}].attributes.JOINTS_0[{vertex_index}]",
                        primitive.source_primitive_index
                    )));
                }
            }
        }
    }
    Ok(())
}

fn decode_animations(
    document: &gltf::Document,
    blob: &[u8],
    gates: &mut Vec<GlbGate>,
) -> Result<Vec<IrAnimation>, GlbFatalError> {
    document
        .animations()
        .map(|animation| {
            let samplers = animation
                .samplers()
                .map(|sampler| {
                    let input_accessor = sampler.input();
                    let input_times_seconds = decode_scalar_f32(input_accessor.clone(), blob)
                        .ok_or_else(|| {
                            layout_error("animation input accessor could not be decoded").in_json(
                                format!(
                                    "animations[{}].samplers[{}].input",
                                    animation.index(),
                                    sampler.index()
                                ),
                            )
                        })?;
                    validate_animation_times(
                        &input_times_seconds,
                        input_accessor.min(),
                        input_accessor.max(),
                        animation.index(),
                        sampler.index(),
                    )?;
                    let output = sampler.output();
                    let output_accessor_type = accessor_type_name(output.dimensions()).to_owned();
                    let output_values = decode_float_accessor(output, blob).ok_or_else(|| {
                        layout_error("animation output accessor could not be decoded").in_json(
                            format!(
                                "animations[{}].samplers[{}].output",
                                animation.index(),
                                sampler.index()
                            ),
                        )
                    })?;
                    if output_values.iter().any(|value| !value.is_finite()) {
                        return Err(GlbFatalError::new(
                            "M2A-GLB-NONFINITE-FLOAT",
                            "animation output contains a non-finite value",
                        )
                        .in_json(format!(
                            "animations[{}].samplers[{}].output",
                            animation.index(),
                            sampler.index()
                        )));
                    }
                    Ok(IrAnimationSampler {
                        id: sampler.index() as u32,
                        interpolation: interpolation_name(sampler.interpolation()).to_owned(),
                        input_times_seconds,
                        output_accessor_type,
                        output_values,
                    })
                })
                .collect::<Result<Vec<_>, GlbFatalError>>()?;

            let channels = animation
                .channels()
                .map(|channel| {
                    let target_path = animation_property_name(channel.target().property());
                    if target_path == "WEIGHTS" {
                        gates.push(blocking_gate(
                            "M2A-GLB-ANIMATION-WEIGHTS-DEFERRED",
                            &format!(
                                "animations[{}].channels[{}]",
                                animation.index(),
                                channel.index()
                            ),
                            "translation, rotation or scale target",
                            "WEIGHTS",
                            "morph-target weight conversion semantics are deferred",
                        ));
                    }
                    IrAnimationChannel {
                        sampler_id: channel.sampler().index() as u32,
                        target_node_id: channel.target().node().index() as u32,
                        target_path: target_path.to_owned(),
                    }
                })
                .collect::<Vec<_>>();
            let duration_seconds = samplers
                .iter()
                .filter_map(|sampler| sampler.input_times_seconds.last().copied())
                .fold(0.0_f32, f32::max);
            Ok(IrAnimation {
                id: animation.index() as u32,
                name: animation.name().map(str::to_owned),
                duration_seconds,
                samplers,
                channels,
            })
        })
        .collect()
}

fn decode_scalar_f32(accessor: gltf::Accessor<'_>, blob: &[u8]) -> Option<Vec<f32>> {
    gltf::accessor::Iter::<f32>::new(accessor, |_| Some(blob)).map(Iterator::collect)
}

fn decode_float_accessor(accessor: gltf::Accessor<'_>, blob: &[u8]) -> Option<Vec<f32>> {
    use gltf::accessor::{Dimensions, Iter};

    match accessor.dimensions() {
        Dimensions::Scalar => Iter::<f32>::new(accessor, |_| Some(blob)).map(Iterator::collect),
        Dimensions::Vec2 => {
            Iter::<[f32; 2]>::new(accessor, |_| Some(blob)).map(|values| values.flatten().collect())
        }
        Dimensions::Vec3 => {
            Iter::<[f32; 3]>::new(accessor, |_| Some(blob)).map(|values| values.flatten().collect())
        }
        Dimensions::Vec4 => {
            Iter::<[f32; 4]>::new(accessor, |_| Some(blob)).map(|values| values.flatten().collect())
        }
        Dimensions::Mat2 => Iter::<[[f32; 2]; 2]>::new(accessor, |_| Some(blob))
            .map(|values| values.flatten().flatten().collect()),
        Dimensions::Mat3 => Iter::<[[f32; 3]; 3]>::new(accessor, |_| Some(blob))
            .map(|values| values.flatten().flatten().collect()),
        Dimensions::Mat4 => Iter::<[[f32; 4]; 4]>::new(accessor, |_| Some(blob))
            .map(|values| values.flatten().flatten().collect()),
    }
}

fn validate_animation_times(
    times: &[f32],
    declared_min: Option<Value>,
    declared_max: Option<Value>,
    animation_index: usize,
    sampler_index: usize,
) -> Result<(), GlbFatalError> {
    let path = format!("animations[{animation_index}].samplers[{sampler_index}].input");
    if times.iter().any(|time| !time.is_finite()) {
        return Err(GlbFatalError::new(
            "M2A-GLB-NONFINITE-FLOAT",
            "animation input time contains a non-finite value",
        )
        .in_json(path));
    }
    if times.first().is_some_and(|time| *time < 0.0)
        || times.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(layout_error(
            "animation input times must be non-negative and strictly increasing",
        )
        .in_json(path));
    }
    let min = scalar_bound(declared_min.as_ref()).ok_or_else(|| {
        layout_error("animation input accessor requires finite scalar min").in_json(path.clone())
    })?;
    let max = scalar_bound(declared_max.as_ref()).ok_or_else(|| {
        layout_error("animation input accessor requires finite scalar max").in_json(path.clone())
    })?;
    if times.first().copied() != Some(min) || times.last().copied() != Some(max) {
        return Err(
            layout_error("animation input min/max must match first/last keyframe time")
                .in_json(path),
        );
    }
    Ok(())
}

fn scalar_bound(value: Option<&Value>) -> Option<f32> {
    let values = value?.as_array()?;
    if values.len() != 1 {
        return None;
    }
    let value = values[0].as_f64()? as f32;
    value.is_finite().then_some(value)
}

fn accessor_type_name(dimensions: gltf::accessor::Dimensions) -> &'static str {
    use gltf::accessor::Dimensions;
    match dimensions {
        Dimensions::Scalar => "SCALAR",
        Dimensions::Vec2 => "VEC2",
        Dimensions::Vec3 => "VEC3",
        Dimensions::Vec4 => "VEC4",
        Dimensions::Mat2 => "MAT2",
        Dimensions::Mat3 => "MAT3",
        Dimensions::Mat4 => "MAT4",
    }
}

fn interpolation_name(interpolation: gltf::animation::Interpolation) -> &'static str {
    use gltf::animation::Interpolation;
    match interpolation {
        Interpolation::Linear => "LINEAR",
        Interpolation::Step => "STEP",
        Interpolation::CubicSpline => "CUBICSPLINE",
    }
}

fn animation_property_name(property: gltf::animation::Property) -> &'static str {
    use gltf::animation::Property;
    match property {
        Property::Translation => "TRANSLATION",
        Property::Rotation => "ROTATION",
        Property::Scale => "SCALE",
        Property::MorphTargetWeights => "WEIGHTS",
    }
}

fn material_from_gltf(material: gltf::Material<'_>) -> IrMaterial {
    let pbr = material.pbr_metallic_roughness();
    IrMaterial {
        id: material.index().unwrap_or(0) as u32,
        name: material.name().map(str::to_owned),
        base_color_factor: pbr.base_color_factor(),
        base_color_texture: pbr.base_color_texture().map(|info| IrTextureBinding {
            texture_id: info.texture().index() as u32,
            tex_coord_set: info.tex_coord(),
        }),
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        metallic_roughness_texture: pbr
            .metallic_roughness_texture()
            .map(|info| IrTextureBinding {
                texture_id: info.texture().index() as u32,
                tex_coord_set: info.tex_coord(),
            }),
        normal_texture: material.normal_texture().map(|info| IrTextureBinding {
            texture_id: info.texture().index() as u32,
            tex_coord_set: info.tex_coord(),
        }),
        emissive_factor: material.emissive_factor(),
        emissive_texture: material.emissive_texture().map(|info| IrTextureBinding {
            texture_id: info.texture().index() as u32,
            tex_coord_set: info.tex_coord(),
        }),
        alpha_mode: format!("{:?}", material.alpha_mode()).to_ascii_uppercase(),
        alpha_cutoff: material.alpha_cutoff(),
        double_sided: material.double_sided(),
    }
}

fn reject_nonfinite_materials(materials: &[IrMaterial]) -> Result<(), GlbFatalError> {
    for (index, material) in materials.iter().enumerate() {
        let finite = material
            .base_color_factor
            .iter()
            .chain([material.metallic_factor, material.roughness_factor].iter())
            .chain(material.emissive_factor.iter())
            .chain(material.alpha_cutoff.iter())
            .all(|value| value.is_finite());
        if !finite {
            return Err(GlbFatalError::new(
                "M2A-GLB-NONFINITE-FLOAT",
                "material contains a non-finite float",
            )
            .in_json(format!("materials[{index}]")));
        }
    }
    Ok(())
}

fn mag_filter_name(filter: MagFilter) -> &'static str {
    match filter {
        MagFilter::Nearest => "NEAREST",
        MagFilter::Linear => "LINEAR",
    }
}

fn min_filter_name(filter: MinFilter) -> &'static str {
    match filter {
        MinFilter::Nearest => "NEAREST",
        MinFilter::Linear => "LINEAR",
        MinFilter::NearestMipmapNearest => "NEAREST_MIPMAP_NEAREST",
        MinFilter::LinearMipmapNearest => "LINEAR_MIPMAP_NEAREST",
        MinFilter::NearestMipmapLinear => "NEAREST_MIPMAP_LINEAR",
        MinFilter::LinearMipmapLinear => "LINEAR_MIPMAP_LINEAR",
    }
}

fn wrapping_mode_name(mode: WrappingMode) -> &'static str {
    match mode {
        WrappingMode::ClampToEdge => "CLAMP_TO_EDGE",
        WrappingMode::MirroredRepeat => "MIRRORED_REPEAT",
        WrappingMode::Repeat => "REPEAT",
    }
}

fn reserve_decoded_items(
    decoded_bytes: &mut usize,
    item_count: usize,
    item_bytes: usize,
    limits: &GlbLimits,
) -> Result<(), GlbFatalError> {
    let materialized_bytes = item_count.checked_mul(item_bytes).ok_or_else(|| {
        GlbFatalError::new(
            "M2A-GLB-INTEGER-OVERFLOW",
            "decoded geometry materialization size overflow",
        )
    })?;
    *decoded_bytes = decoded_bytes
        .checked_add(materialized_bytes)
        .ok_or_else(|| {
            GlbFatalError::new(
                "M2A-GLB-INTEGER-OVERFLOW",
                "cumulative decoded geometry size overflow",
            )
        })?;
    if *decoded_bytes > limits.max_decoded_geometry_bytes {
        return Err(limit_error(format!(
            "decoded geometry materialization {} exceeds {}",
            *decoded_bytes, limits.max_decoded_geometry_bytes
        )));
    }
    Ok(())
}

fn reject_nonfinite(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    tangents: &[[f32; 4]],
    uv0: &[[f32; 2]],
    weights: &[[f32; 4]],
) -> Result<(), GlbFatalError> {
    let finite = positions
        .iter()
        .flatten()
        .chain(normals.iter().flatten())
        .chain(tangents.iter().flatten())
        .chain(uv0.iter().flatten())
        .chain(weights.iter().flatten())
        .all(|value| value.is_finite());
    if !finite {
        return Err(GlbFatalError::new(
            "M2A-GLB-NONFINITE-FLOAT",
            "decoded attribute contains a non-finite float",
        ));
    }
    if weights.iter().flatten().any(|value| *value < 0.0) {
        return Err(layout_error("decoded WEIGHTS_0 contains a negative value"));
    }
    Ok(())
}

fn bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    (min, max)
}

fn merge_bounds(
    aggregate_min: &mut Option<[f32; 3]>,
    aggregate_max: &mut Option<[f32; 3]>,
    min: [f32; 3],
    max: [f32; 3],
) {
    match (aggregate_min.as_mut(), aggregate_max.as_mut()) {
        (Some(current_min), Some(current_max)) => {
            for axis in 0..3 {
                current_min[axis] = current_min[axis].min(min[axis]);
                current_max[axis] = current_max[axis].max(max[axis]);
            }
        }
        _ => {
            *aggregate_min = Some(min);
            *aggregate_max = Some(max);
        }
    }
}

fn blocking_gate(code: &str, path: &str, expected: &str, actual: &str, message: &str) -> GlbGate {
    gate(code, "BLOCKING", path, expected, actual, message)
}

fn warning_gate(code: &str, path: &str, expected: &str, actual: &str, message: &str) -> GlbGate {
    gate(code, "WARNING", path, expected, actual, message)
}

fn gate(
    code: &str,
    severity: &str,
    path: &str,
    expected: &str,
    actual: &str,
    message: &str,
) -> GlbGate {
    GlbGate {
        code: code.to_owned(),
        severity: severity.to_owned(),
        path: path.to_owned(),
        expected: expected.to_owned(),
        actual: actual.to_owned(),
        message: message.to_owned(),
    }
}

fn read_u32(input: &[u8], offset: usize) -> Result<u32, GlbFatalError> {
    let bytes = input.get(offset..offset + 4).ok_or_else(|| {
        GlbFatalError::new("M2A-GLB-CHUNK-INVALID", "truncated u32 field").at(offset)
    })?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("four-byte slice"),
    ))
}

fn checked_add(left: usize, right: usize) -> Result<usize, GlbFatalError> {
    left.checked_add(right)
        .ok_or_else(|| GlbFatalError::new("M2A-GLB-INTEGER-OVERFLOW", "counter addition overflow"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use fmt::Write;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn map_gltf_error(error: gltf::Error) -> GlbFatalError {
    GlbFatalError::new("M2A-GLB-JSON-INVALID", error.to_string())
}
