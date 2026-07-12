use std::fmt;

use gltf::mesh::{Mode, Semantic};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const GLB_SCHEMA_VERSION: u32 = 1;

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
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_decoded_geometry_bytes: usize,
    pub max_images: usize,
    pub max_single_image_bytes: usize,
    pub max_total_image_bytes: usize,
    pub max_skins: usize,
    pub max_joints: usize,
    pub max_animations: usize,
    pub max_keyframes: usize,
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
            max_vertices: 1_000_000,
            max_indices: 3_000_000,
            max_decoded_geometry_bytes: 256 * 1024 * 1024,
            max_images: 10_000,
            max_single_image_bytes: 32 * 1024 * 1024,
            max_total_image_bytes: 64 * 1024 * 1024,
            max_skins: 10_000,
            max_joints: 100_000,
            max_animations: 10_000,
            max_keyframes: 1_000_000,
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
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrAnimation {
    pub id: u32,
    pub name: Option<String>,
    pub duration_seconds: f32,
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

pub fn inspect_glb(input: &[u8], limits: &GlbLimits) -> Result<GlbInspectionReport, GlbFatalError> {
    Ok(ingest_glb(input, limits)?.report)
}

pub fn ingest_glb(input: &[u8], limits: &GlbLimits) -> Result<GlbIngestResult, GlbFatalError> {
    validate_header(input, limits)?;
    let (json_bytes, blob) = glb_payloads(input)?;
    let raw = preflight_json(json_bytes, blob, limits)?;
    let gltf = gltf::Gltf::from_slice(input).map_err(map_gltf_error)?;
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
            reserve_decoded_items(
                &mut decoded_bytes,
                primitive
                    .get(&Semantic::Positions)
                    .map_or(0, |accessor| accessor.count()),
                12,
                limits,
            )?;
            let positions = reader
                .read_positions()
                .map(|values| values.collect::<Vec<_>>())
                .unwrap_or_default();
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
            if !normals.is_empty() && normals.len() != positions.len() {
                gates.push(blocking_gate(
                    "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
                    &path,
                    &positions.len().to_string(),
                    &normals.len().to_string(),
                    "NORMAL count differs from POSITION count",
                ));
            }
            if !uv0.is_empty() && uv0.len() != positions.len() {
                gates.push(blocking_gate(
                    "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
                    &path,
                    &positions.len().to_string(),
                    &uv0.len().to_string(),
                    "TEXCOORD_0 count differs from POSITION count",
                ));
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
            if let Some(index) = indices
                .iter()
                .copied()
                .find(|index| *index as usize >= positions.len())
            {
                gates.push(blocking_gate(
                    "M2A-GLB-INDEX-OOB",
                    &path,
                    &format!("index < {}", positions.len()),
                    &index.to_string(),
                    "index is outside the POSITION domain",
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
    let materials = gltf
        .document
        .materials()
        .map(material_from_gltf)
        .collect::<Vec<_>>();
    let textures = gltf
        .document
        .textures()
        .map(|texture| IrTexture {
            id: texture.index() as u32,
            source_image_id: texture.source().index() as u32,
            sampler_index: texture.sampler().index().map(|index| index as u32),
        })
        .collect::<Vec<_>>();
    let coordinate_policy = CoordinatePolicy::default();
    let inventory = GlbInventory {
        scene_count: scenes.len(),
        node_count: nodes.len(),
        mesh_count: meshes.len(),
        primitive_count: primitives.len(),
        material_count: materials.len(),
        texture_count: textures.len(),
        image_count: gltf.document.images().count(),
        skin_count: gltf.document.skins().count(),
        joint_reference_count: gltf
            .document
            .skins()
            .map(|skin| skin.joints().count())
            .sum(),
        animation_count: gltf.document.animations().count(),
        keyframe_count: 0,
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
        diagnostics: Vec::new(),
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
        images: Vec::new(),
        skins: Vec::new(),
        animations: Vec::new(),
    };
    Ok(GlbIngestResult {
        schema_version: GLB_SCHEMA_VERSION,
        ir,
        report,
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

    let nodes = array_or_empty(object.get("nodes"));
    let meshes = array_or_empty(object.get("meshes"));
    let accessors_json = array_or_empty(object.get("accessors"));
    let views_json = array_or_empty(object.get("bufferViews"));
    let images = array_or_empty(object.get("images"));
    let skins = array_or_empty(object.get("skins"));
    let animations = array_or_empty(object.get("animations"));
    check_count(nodes.len(), limits.max_nodes, "nodes")?;
    check_count(meshes.len(), limits.max_meshes, "meshes")?;
    check_count(accessors_json.len(), limits.max_accessors, "accessors")?;
    check_count(views_json.len(), limits.max_buffer_views, "buffer views")?;
    check_count(images.len(), limits.max_images, "images")?;
    check_count(skins.len(), limits.max_skins, "skins")?;
    check_count(animations.len(), limits.max_animations, "animations")?;

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

    let views = parse_buffer_views(views_json, declared_buffer_length)?;
    let accessors = parse_accessors(accessors_json, &views)?;
    validate_image_limits(images, &views, limits)?;

    let mut primitive_count = 0_usize;
    let mut vertex_count = 0_usize;
    let mut index_count = 0_usize;
    let mut triangle_count = 0_usize;
    let mut decoded_geometry_bytes = 0_usize;
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
            if let Some(accessor) = positions {
                if accessor.component_type != 5126 || accessor.element_type != "VEC3" {
                    return Err(GlbFatalError::new(
                        "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                        "POSITION must use FLOAT VEC3",
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
            index_count = checked_add(index_count, primitive_indices)?;
            reserve_decoded_items(&mut decoded_geometry_bytes, primitive_indices, 4, limits)?;
            let mode = primitive.get("mode").and_then(Value::as_u64).unwrap_or(4);
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
    })
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
        let buffer = value.get("buffer").and_then(Value::as_u64).unwrap_or(0);
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
            GlbFatalError::new("M2A-GLB-BUFFER-VIEW-OOB", "buffer view range overflows")
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
        let component_type = value
            .get("componentType")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                GlbFatalError::new(
                    "M2A-GLB-ACCESSOR-LAYOUT-INVALID",
                    "accessor componentType must be an unsigned integer",
                )
                .in_json(format!("{prefix}.componentType"))
            })?;
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
                    "M2A-GLB-ACCESSOR-OOB",
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
                        GlbFatalError::new("M2A-GLB-ACCESSOR-OOB", "accessor byte range overflows")
                            .in_json(prefix.clone())
                    })?
            };
            let end = byte_offset.checked_add(occupied).ok_or_else(|| {
                GlbFatalError::new("M2A-GLB-ACCESSOR-OOB", "accessor byte range overflows")
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
            value
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| {
                    GlbFatalError::new(code, "value must fit an unsigned host index").in_json(path)
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
    let text = error.to_string();
    let code = if text.to_ascii_lowercase().contains("json") {
        "M2A-GLB-JSON-INVALID"
    } else {
        "M2A-GLB-CHUNK-INVALID"
    };
    GlbFatalError::new(code, text)
}
