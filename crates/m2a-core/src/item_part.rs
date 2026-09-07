//! Compilation of one resolved Item part from the shared model IR.
//!
//! This module owns only Item-specific model identity and authoring transform.
//! GLB ingestion and material authoring remain shared upstream stages.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    glb::{GlbIngestResult, GlbLimits, ingest_glb},
    item::{
        ITEM_RECIPE_INVALID, ITEM_SCHEMA_INVALID, ITEM_SCHEMA_VERSION, ItemErrorV1,
        ItemPartRecipeV1, ItemPartSlotV1, ItemPartTransformV1, sha256_hex, validate_resref,
        validate_sha256,
    },
    mdl::{
        BinaryMdlArtifactV1, InspectionReport, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlStateProjectionProfileV1, MdlWriterOptionsV1, NodeReport,
        write_binary_mdl_exact_face_planes_v1,
    },
    model_ir::{AuroraModelIrV1, AuroraModelNodeV1, AuroraSegmentDeformationV1},
    model_limits::AURORA_MODEL_TRIANGLE_BUDGET_V1,
    model_segmentation::{ModelSegmentationReportV1, segment_model_for_binary_mdl_v1},
    profile_a::{
        ProfileALimitsV1, ProfileAMaterialPolicyV1, ProfileAOptionsV1, convert_profile_a_exact_v1,
        derive_meshy_m0_static_rigid_profile_v1,
    },
};

pub const ITEM_PART_INVALID: &str = "M2A-ITEM-PART-INVALID";
pub const ITEM_PART_SOURCE_STALE: &str = "M2A-ITEM-PART-SOURCE-STALE";
pub const ITEM_PART_TRIANGLE_BUDGET_EXCEEDED: &str = "M2A-ITEM-PART-TRIANGLE-BUDGET-EXCEEDED";
pub const ITEM_PART_WRITE_FAILED: &str = "M2A-ITEM-PART-WRITE-FAILED";
pub const ITEM_PART_SEMANTIC_DIFF: &str = "M2A-ITEM-PART-SEMANTIC-DIFF";
pub const ITEM_PART_GLB_INVALID: &str = "M2A-ITEM-PART-GLB-INVALID";
pub const ITEM_PART_PROFILE_INVALID: &str = "M2A-ITEM-PART-PROFILE-INVALID";
pub const ITEM_PART_DOUBLE_SIDED_INVALID: &str = "M2A-ITEM-PART-DOUBLE-SIDED-INVALID";

const ITEM_TRANSFORM_NODE_NAME: &str = "m2a_item_xform";
const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, 1.0, 0.0, //
    0.0, 0.0, 0.0, 1.0,
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPartCompileRequestV1 {
    pub schema_version: u32,
    pub recipe_sha256: String,
    pub part: ItemPartRecipeV1,
    pub model_resref: String,
    pub model: AuroraModelIrV1,
    pub material_textures: Vec<MdlMaterialTextureBindingV1>,
}

/// Direct static-GLB entry point for Item parts.  This keeps the same narrow
/// Meshy M0 intake used by other rigid product routes while binding the source
/// bytes, recipe, resolved Item ResRef and texture slots before MDL emission.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeshyItemPartCompileRequestV1 {
    pub schema_version: u32,
    pub recipe_sha256: String,
    pub part: ItemPartRecipeV1,
    pub model_resref: String,
    pub material_textures: Vec<MdlMaterialTextureBindingV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPartCompileReportV1 {
    pub schema_version: u32,
    pub slot: ItemPartSlotV1,
    pub recipe_sha256: String,
    pub source_sha256: String,
    pub model_resref: String,
    pub model_sha256: String,
    pub source_triangle_count: usize,
    pub double_sided_backface_triangle_count: usize,
    pub double_sided_policy: String,
    pub triangle_count: usize,
    pub segmentation: ModelSegmentationReportV1,
    pub transform: ItemPartTransformV1,
    pub transform_node_name: String,
    pub transform_controller_types: Vec<i32>,
    pub model_classification: u8,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemPartArtifactV1 {
    pub payload: Vec<u8>,
    pub inspection: InspectionReport,
    pub report: ItemPartCompileReportV1,
}

/// Item-specific Profile A options.  Geometry uses the one shared product
/// budget; rigid Item models may retain the bounded material slots already
/// present in the source GLB.
pub fn static_item_profile_a_options_v1() -> ProfileAOptionsV1 {
    ProfileAOptionsV1 {
        material_policy: ProfileAMaterialPolicyV1::BoundedSourceSlots,
        limits: ProfileALimitsV1 {
            max_unique_materials: 256,
            ..ProfileALimitsV1::default()
        },
        ..ProfileAOptionsV1::default()
    }
}

/// Compiles one unskinned, animation-free Meshy GLB directly into an Item
/// part.  No geometry is simplified here: inputs over the 300k product budget
/// fail closed, while legal large meshes are partitioned by the existing MDL
/// segmenter without changing triangles or material slots.
pub fn compile_meshy_static_item_part_v1(
    source_glb: &[u8],
    request: &MeshyItemPartCompileRequestV1,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    compile_meshy_static_item_part_internal_v2(source_glb, request, false)
}

/// Compiles a static Item while preserving glTF `doubleSided` in classic
/// Aurora MDL. Aurora has no equivalent mesh flag in this profile, so V2 emits
/// explicit reverse-wound faces with inverted normals and unchanged UVs.
/// V1 remains frozen for reproducibility of existing Item proof lineages.
pub fn compile_meshy_static_item_part_v2(
    source_glb: &[u8],
    request: &MeshyItemPartCompileRequestV1,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    compile_meshy_static_item_part_internal_v2(source_glb, request, true)
}

fn compile_meshy_static_item_part_internal_v2(
    source_glb: &[u8],
    request: &MeshyItemPartCompileRequestV1,
    preserve_source_double_sided: bool,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|source| {
        ItemErrorV1::fatal(
            ITEM_PART_GLB_INVALID,
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            format!("{}: {}", source.code, source.message),
        )
    })?;
    if ingest.report.input.sha256 != request.part.source_sha256 {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_SOURCE_STALE,
            "request.part.sourceSha256",
            format!(
                "recipe binds {}, source GLB is {}",
                request.part.source_sha256, ingest.report.input.sha256
            ),
        ));
    }
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest).map_err(|source| {
        ItemErrorV1::fatal(
            ITEM_PART_PROFILE_INVALID,
            source.path,
            format!("{}: {}", source.code, source.message),
        )
    })?;
    let conversion = convert_profile_a_exact_v1(&ingest, &rig, &static_item_profile_a_options_v1())
        .map_err(|source| {
            ItemErrorV1::fatal(
                ITEM_PART_PROFILE_INVALID,
                source.path,
                format!("{}: {}", source.code, source.message),
            )
        })?;
    if !conversion.report.conversion_eligible {
        let blocking = conversion
            .report
            .gates
            .iter()
            .filter(|gate| gate.severity == "BLOCKING")
            .map(|gate| gate.code.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ItemErrorV1::fatal(
            ITEM_PART_PROFILE_INVALID,
            "conversion.report",
            format!("static Item conversion is not eligible; blocking gates: {blocking}"),
        ));
    }
    let mut model = conversion.creature.ok_or_else(|| {
        ItemErrorV1::fatal(
            ITEM_PART_PROFILE_INVALID,
            "conversion.model",
            "eligible static Item conversion has no common model IR",
        )
    })?;
    let source_triangle_count = model
        .segments
        .iter()
        .try_fold(0usize, |sum, segment| {
            sum.checked_add(segment.indices.len() / 3)
        })
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PART_DOUBLE_SIDED_INVALID,
                "conversion.model.segments",
                "source triangle count overflow",
            )
        })?;
    let double_sided_slots = bound_source_double_sided_slots_v1(&ingest, &model)?;
    let double_sided_backface_triangle_count = if preserve_source_double_sided {
        expand_source_double_sided_geometry_v1(&double_sided_slots, &mut model)?
    } else {
        0
    };
    let mut artifact = compile_item_part_v1(&ItemPartCompileRequestV1 {
        schema_version: request.schema_version,
        recipe_sha256: request.recipe_sha256.clone(),
        part: request.part.clone(),
        model_resref: request.model_resref.clone(),
        model,
        material_textures: request.material_textures.clone(),
    })?;
    artifact.report.source_triangle_count = source_triangle_count;
    artifact.report.double_sided_backface_triangle_count = double_sided_backface_triangle_count;
    artifact.report.double_sided_policy = if double_sided_slots.is_empty() {
        "SOURCE_SINGLE_SIDED_UNCHANGED_V1"
    } else if preserve_source_double_sided {
        "SOURCE_DOUBLE_SIDED_EXPLICIT_BACKFACES_V1"
    } else {
        "SOURCE_DOUBLE_SIDED_LEGACY_UNMAPPED_V1"
    }
    .to_owned();
    Ok(artifact)
}

fn bound_source_double_sided_slots_v1(
    ingest: &GlbIngestResult,
    model: &AuroraModelIrV1,
) -> Result<BTreeSet<u32>, ItemErrorV1> {
    let mut double_sided_slots = BTreeSet::new();
    for binding in &model.material_source_bindings {
        let Some(source_material_id) = binding.source_material_id else {
            continue;
        };
        let source_material = ingest
            .ir
            .materials
            .iter()
            .find(|material| material.id == source_material_id)
            .ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PART_DOUBLE_SIDED_INVALID,
                    "conversion.model.materialSourceBindings",
                    format!("source material {source_material_id} is missing"),
                )
            })?;
        if source_material.double_sided {
            double_sided_slots.insert(binding.slot);
        }
    }
    Ok(double_sided_slots)
}

fn expand_source_double_sided_geometry_v1(
    double_sided_slots: &BTreeSet<u32>,
    model: &mut AuroraModelIrV1,
) -> Result<usize, ItemErrorV1> {
    let mut generated_triangle_count = 0usize;
    for segment in &mut model.segments {
        if !double_sided_slots.contains(&segment.material_slot) {
            continue;
        }
        if !segment.indices.len().is_multiple_of(3)
            || segment.positions.len() != segment.normals.len()
            || segment.positions.len() != segment.uv0.len()
            || segment
                .tangents
                .as_ref()
                .is_some_and(|tangents| tangents.len() != segment.positions.len())
            || (!segment.weights.is_empty() && segment.weights.len() != segment.positions.len())
            || (!segment.face_surface_ids.is_empty()
                && segment.face_surface_ids.len() != segment.indices.len() / 3)
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PART_DOUBLE_SIDED_INVALID,
                "conversion.model.segments",
                "double-sided expansion requires complete triangle, normal, UV, tangent, weight and surface streams",
            ));
        }

        let original_vertex_count = segment.positions.len();
        let vertex_offset = u32::try_from(original_vertex_count).map_err(|_| {
            ItemErrorV1::fatal(
                ITEM_PART_DOUBLE_SIDED_INVALID,
                "conversion.model.segments.positions",
                "double-sided vertex offset does not fit u32",
            )
        })?;
        let original_indices = segment.indices.clone();
        let original_triangle_count = original_indices.len() / 3;
        generated_triangle_count = generated_triangle_count
            .checked_add(original_triangle_count)
            .ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PART_DOUBLE_SIDED_INVALID,
                    "conversion.model.segments.indices",
                    "double-sided triangle count overflow",
                )
            })?;

        let backface_positions = segment.positions.clone();
        let backface_normals = segment
            .normals
            .iter()
            .map(|normal| [-normal[0], -normal[1], -normal[2]])
            .collect::<Vec<_>>();
        let backface_uv0 = segment.uv0.clone();
        let backface_tangents = segment.tangents.as_ref().map(|tangents| {
            tangents
                .iter()
                .map(|tangent| [tangent[0], tangent[1], tangent[2], -tangent[3]])
                .collect::<Vec<_>>()
        });
        let backface_weights = segment.weights.clone();
        let backface_surface_ids = segment.face_surface_ids.clone();

        segment.positions.extend(backface_positions);
        segment.normals.extend(backface_normals);
        segment.uv0.extend(backface_uv0);
        if let (Some(tangents), Some(backfaces)) = (segment.tangents.as_mut(), backface_tangents) {
            tangents.extend(backfaces);
        }
        if !segment.weights.is_empty() {
            segment.weights.extend(backface_weights);
        }
        if !segment.face_surface_ids.is_empty() {
            segment.face_surface_ids.extend(backface_surface_ids);
        }
        segment
            .indices
            .try_reserve(original_indices.len())
            .map_err(|_| {
                ItemErrorV1::fatal(
                    ITEM_PART_DOUBLE_SIDED_INVALID,
                    "conversion.model.segments.indices",
                    "double-sided index allocation failed",
                )
            })?;
        for triangle in original_indices.chunks_exact(3) {
            let a = vertex_offset.checked_add(triangle[0]).ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PART_DOUBLE_SIDED_INVALID,
                    "conversion.model.segments.indices",
                    "double-sided index overflow",
                )
            })?;
            let b = vertex_offset.checked_add(triangle[1]).ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PART_DOUBLE_SIDED_INVALID,
                    "conversion.model.segments.indices",
                    "double-sided index overflow",
                )
            })?;
            let c = vertex_offset.checked_add(triangle[2]).ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PART_DOUBLE_SIDED_INVALID,
                    "conversion.model.segments.indices",
                    "double-sided index overflow",
                )
            })?;
            segment.indices.extend_from_slice(&[a, c, b]);
        }
    }
    Ok(generated_triangle_count)
}

/// Compiles one Item part from the common, already material-resolved model IR.
///
/// The function inserts a model-named controllerless identity root and bakes
/// the authoring TRS into the former source root. Translation/orientation use
/// native bind controllers 8/20; non-unit uniform scale uses controller 36.
pub fn compile_item_part_v1(
    request: &ItemPartCompileRequestV1,
) -> Result<ItemPartArtifactV1, ItemErrorV1> {
    validate_request(request)?;
    let mut model = request.model.clone();
    let triangle_count = model
        .segments
        .iter()
        .try_fold(0usize, |sum, segment| {
            sum.checked_add(segment.indices.len() / 3)
        })
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PART_INVALID,
                "request.model.segments",
                "part triangle count overflow",
            )
        })?;
    if triangle_count > AURORA_MODEL_TRIANGLE_BUDGET_V1 {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_TRIANGLE_BUDGET_EXCEEDED,
            "request.model.segments",
            format!(
                "part has {triangle_count} triangles; whole-item budget is {AURORA_MODEL_TRIANGLE_BUDGET_V1}"
            ),
        ));
    }
    insert_item_transform_root(&mut model, &request.model_resref, request.part.transform)?;
    let segmentation = segment_model_for_binary_mdl_v1(&mut model).map_err(|source| {
        ItemErrorV1::fatal(
            &source.code,
            source.path,
            format!("item-part segmentation failed: {}", source.message),
        )
    })?;
    if segmentation.triangle_count != triangle_count {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_SEMANTIC_DIFF,
            "segmentation.triangleCount",
            "segmentation changed the item-part triangle count",
        ));
    }
    let BinaryMdlArtifactV1 {
        payload,
        inspection,
        report: writer_report,
    } = write_binary_mdl_exact_face_planes_v1(
        &model,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::ItemPartStaticRigidNativeV1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: request.model_resref.clone(),
            diffuse_texture_resref_by_material_slot: request.material_textures.clone(),
        },
    )
    .map_err(|source| {
        ItemErrorV1::fatal(
            ITEM_PART_WRITE_FAILED,
            source.path,
            format!("{}: {}", source.code, source.message),
        )
    })?;
    if !writer_report.semantic_diff.is_empty()
        || inspection.model.name != request.model_resref
        || inspection.model.classification != 4
        || !inspection.animations.is_empty()
    {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_SEMANTIC_DIFF,
            "mdl.readback",
            "item-part MDL differs from its exact semantic projection",
        ));
    }
    let root = inspection.node_tree.roots.first().ok_or_else(|| {
        ItemErrorV1::fatal(
            ITEM_PART_SEMANTIC_DIFF,
            "mdl.nodeTree.roots",
            "item-part MDL has no root",
        )
    })?;
    if root.name != request.model_resref
        || !root.controllers.is_empty()
        || root.controller_keys_header.used != 0
        || root.controller_data_header.used != 0
    {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_SEMANTIC_DIFF,
            "mdl.nodeTree.root",
            "item-part root is not the exact controllerless model identity",
        ));
    }
    let transform_node = find_node(root, ITEM_TRANSFORM_NODE_NAME).ok_or_else(|| {
        ItemErrorV1::fatal(
            ITEM_PART_SEMANTIC_DIFF,
            "mdl.nodeTree.transform",
            "item-part transform node is missing after readback",
        )
    })?;
    let transform_controller_types = transform_node
        .controllers
        .iter()
        .map(|controller| controller.controller_type)
        .collect::<Vec<_>>();
    let expected_controller_types = if (request.part.transform.scale - 1.0).abs() > 1.0e-5 {
        vec![8, 20, 36]
    } else {
        vec![8, 20]
    };
    if transform_controller_types != expected_controller_types {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_SEMANTIC_DIFF,
            "mdl.nodeTree.transform.controllers",
            "item-part authoring transform controller set differs after readback",
        ));
    }
    let model_sha256 = sha256_hex(&payload);
    Ok(ItemPartArtifactV1 {
        payload,
        inspection,
        report: ItemPartCompileReportV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            slot: request.part.slot,
            recipe_sha256: request.recipe_sha256.clone(),
            source_sha256: request.part.source_sha256.clone(),
            model_resref: request.model_resref.clone(),
            model_sha256,
            source_triangle_count: triangle_count,
            double_sided_backface_triangle_count: 0,
            double_sided_policy: "NO_SOURCE_DOUBLE_SIDED_EXPANSION_V1".to_owned(),
            triangle_count,
            segmentation,
            transform: request.part.transform,
            transform_node_name: ITEM_TRANSFORM_NODE_NAME.to_owned(),
            transform_controller_types,
            model_classification: 4,
            semantic_readback_status: "PASS".to_owned(),
        },
    })
}

fn validate_request(request: &ItemPartCompileRequestV1) -> Result<(), ItemErrorV1> {
    if request.schema_version != ITEM_SCHEMA_VERSION || request.model.schema_version != 1 {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            "request.schemaVersion",
            "item part and common model IR must use schema version 1",
        ));
    }
    validate_sha256(&request.recipe_sha256, "request.recipeSha256")?;
    validate_sha256(&request.part.source_sha256, "request.part.sourceSha256")?;
    validate_resref(&request.model_resref, "request.modelResref")?;
    if request.model.source_sha256 != request.part.source_sha256 {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_SOURCE_STALE,
            "request.model.sourceSha256",
            format!(
                "recipe binds {}, common model IR binds {}",
                request.part.source_sha256, request.model.source_sha256
            ),
        ));
    }
    if request.model.segments.is_empty()
        || request
            .model
            .segments
            .iter()
            .any(|segment| segment.deformation != AuroraSegmentDeformationV1::Rigid)
    {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_INVALID,
            "request.model.segments",
            "item parts require non-empty rigid render geometry",
        ));
    }
    let transform = request.part.transform;
    if transform
        .translation
        .iter()
        .chain(transform.rotation.iter())
        .any(|value| !value.is_finite())
        || !transform.scale.is_finite()
        || transform.scale <= 0.0
    {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "request.part.transform",
            "item-part transform must be finite with positive uniform scale",
        ));
    }
    let norm_squared = transform
        .rotation
        .iter()
        .map(|value| value * value)
        .sum::<f32>();
    if (norm_squared - 1.0).abs() > 0.0001 {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "request.part.transform.rotation",
            "item-part rotation quaternion must be normalized",
        ));
    }
    Ok(())
}

fn insert_item_transform_root(
    model: &mut AuroraModelIrV1,
    model_resref: &str,
    transform: ItemPartTransformV1,
) -> Result<(), ItemErrorV1> {
    let roots = model
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.parent_id.is_none())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_INVALID,
            "request.model.nodes",
            "item part requires exactly one source root",
        ));
    }
    if model.nodes.iter().enumerate().any(|(index, node)| {
        index != roots[0]
            && (node.name.eq_ignore_ascii_case(model_resref)
                || node.name.eq_ignore_ascii_case(ITEM_TRANSFORM_NODE_NAME))
    }) {
        return Err(ItemErrorV1::fatal(
            ITEM_PART_INVALID,
            "request.model.nodes.name",
            "source node name collides with the reserved Item root namespace",
        ));
    }
    let new_root_id = model
        .nodes
        .iter()
        .map(|node| node.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PART_INVALID,
                "request.model.nodes.id",
                "cannot allocate the Item identity root id",
            )
        })?;
    let old_root = &mut model.nodes[roots[0]];
    old_root.name = ITEM_TRANSFORM_NODE_NAME.to_owned();
    old_root.parent_id = Some(new_root_id);
    old_root.bind_local_matrix =
        multiply_matrix(transform_matrix(transform), old_root.bind_local_matrix);
    model.nodes.push(AuroraModelNodeV1 {
        id: new_root_id,
        name: model_resref.to_owned(),
        parent_id: None,
        bind_local_matrix: IDENTITY,
    });
    Ok(())
}

fn transform_matrix(transform: ItemPartTransformV1) -> [f32; 16] {
    let [x, y, z, w] = transform.rotation;
    let scale = transform.scale;
    [
        (1.0 - 2.0 * (y * y + z * z)) * scale,
        (2.0 * (x * y + z * w)) * scale,
        (2.0 * (x * z - y * w)) * scale,
        0.0,
        (2.0 * (x * y - z * w)) * scale,
        (1.0 - 2.0 * (x * x + z * z)) * scale,
        (2.0 * (y * z + x * w)) * scale,
        0.0,
        (2.0 * (x * z + y * w)) * scale,
        (2.0 * (y * z - x * w)) * scale,
        (1.0 - 2.0 * (x * x + y * y)) * scale,
        0.0,
        transform.translation[0],
        transform.translation[1],
        transform.translation[2],
        1.0,
    ]
}

fn multiply_matrix(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

fn find_node<'a>(node: &'a NodeReport, name: &str) -> Option<&'a NodeReport> {
    if node.name == name {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, name))
}
