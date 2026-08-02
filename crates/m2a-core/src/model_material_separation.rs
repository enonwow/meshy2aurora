//! Declarative, target-neutral Material Separation authoring.
//!
//! V1 assigns exact source connected components to authored material IDs. It
//! never mutates the source GLB, changes UVs, or infers semantic materials.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    glb::AuroraAssetIr,
    model_components::{
        ModelComponentErrorV1, SourceComponentKeyV1, connected_components_v1,
        inspect_model_components_v1,
    },
};

pub const MODEL_MATERIAL_SEPARATION_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredMaterialV1 {
    pub authored_material_id: String,
    pub display_name: String,
    pub preview_color: String,
    pub source_fallback_material_id: Option<u32>,
    pub source_fallback_image_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialAssignmentV1 {
    pub component: SourceComponentKeyV1,
    pub authored_material_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialSeparationDocumentV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub materials: Vec<AuthoredMaterialV1>,
    pub assignments: Vec<ModelMaterialAssignmentV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedModelMaterialSlotV1 {
    pub material_slot: u32,
    pub authored_material_id: String,
    pub display_name: String,
    pub preview_color: String,
    pub source_material_id: Option<u32>,
    pub source_material_name: Option<String>,
    pub source_image_sha256: Option<String>,
    pub system_source_material: bool,
    pub component_count: u32,
    pub triangle_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialSeparationReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub separation_sha256: String,
    pub source_component_count: u32,
    pub assigned_component_count: u32,
    pub unassigned_component_count: u32,
    pub source_triangle_count: u32,
    pub output_triangle_count: u32,
    pub source_vertex_count: u32,
    pub output_vertex_count: u32,
    pub duplicated_boundary_vertex_count: u32,
    pub output_section_count: u32,
    pub predicted_texture_count: u32,
    pub material_slots: Vec<ResolvedModelMaterialSlotV1>,
    pub unused_authored_material_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SourceTriangleKeyV1 {
    scene_id: u32,
    node_id: u32,
    primitive_id: u32,
    triangle_index: u32,
}

#[derive(Clone, Debug)]
pub struct ResolvedModelMaterialsV1 {
    pub report: ModelMaterialSeparationReportV1,
    triangle_material_slots: BTreeMap<SourceTriangleKeyV1, u32>,
}

impl ResolvedModelMaterialsV1 {
    pub fn material_slot_for_triangle(
        &self,
        scene_id: u32,
        node_id: u32,
        primitive_id: u32,
        triangle_index: u32,
    ) -> Option<u32> {
        self.triangle_material_slots
            .get(&SourceTriangleKeyV1 {
                scene_id,
                node_id,
                primitive_id,
                triangle_index,
            })
            .copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialSeparationErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelMaterialSeparationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelMaterialSeparationErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ModelMaterialSeparationErrorV1 {
    ModelMaterialSeparationErrorV1 {
        schema_version: MODEL_MATERIAL_SEPARATION_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn component_error(source: ModelComponentErrorV1) -> ModelMaterialSeparationErrorV1 {
    error(&source.code, source.path, source.message)
}

pub fn default_model_material_separation_v1(
    ir: &AuroraAssetIr,
) -> ModelMaterialSeparationDocumentV1 {
    ModelMaterialSeparationDocumentV1 {
        schema_version: MODEL_MATERIAL_SEPARATION_SCHEMA_VERSION_V1,
        source_sha256: ir.source.sha256.clone(),
        materials: Vec::new(),
        assignments: Vec::new(),
    }
}

pub fn model_material_separation_hash_v1(
    document: &ModelMaterialSeparationDocumentV1,
) -> Result<String, ModelMaterialSeparationErrorV1> {
    let mut canonical = document.clone();
    canonical
        .materials
        .sort_by(|left, right| left.authored_material_id.cmp(&right.authored_material_id));
    canonical.assignments.sort_by(|left, right| {
        left.component
            .cmp(&right.component)
            .then_with(|| left.authored_material_id.cmp(&right.authored_material_id))
    });
    let payload = serde_json::to_vec(&canonical).map_err(|source| {
        error(
            "MATERIAL-SEPARATION-SERIALIZE",
            "document",
            source.to_string(),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(payload)))
}

fn validate_material_id(value: &str, path: &str) -> Result<(), ModelMaterialSeparationErrorV1> {
    if value.is_empty()
        || value.len() > 96
        || value.starts_with("source:")
        || !value.bytes().all(|value| {
            value.is_ascii_alphanumeric() || matches!(value, b':' | b'_' | b'-' | b'.')
        })
    {
        return Err(error(
            "MATERIAL-SEPARATION-MATERIAL-ID-INVALID",
            path,
            "authored material id must contain 1..=96 ASCII identifier bytes and cannot use the reserved source: prefix",
        ));
    }
    Ok(())
}

fn validate_material(
    ir: &AuroraAssetIr,
    material: &AuthoredMaterialV1,
    path: &str,
) -> Result<(Option<String>, Option<String>), ModelMaterialSeparationErrorV1> {
    validate_material_id(
        &material.authored_material_id,
        &format!("{path}.authoredMaterialId"),
    )?;
    if material.display_name.is_empty() || material.display_name.len() > 120 {
        return Err(error(
            "MATERIAL-SEPARATION-DISPLAY-NAME-INVALID",
            format!("{path}.displayName"),
            "display name must contain 1..=120 bytes",
        ));
    }
    let color = material.preview_color.as_bytes();
    if color.len() != 7
        || color[0] != b'#'
        || !color[1..].iter().all(|value| value.is_ascii_hexdigit())
    {
        return Err(error(
            "MATERIAL-SEPARATION-PREVIEW-COLOR-INVALID",
            format!("{path}.previewColor"),
            "preview color must be exact #RRGGBB",
        ));
    }

    let Some(source_material_id) = material.source_fallback_material_id else {
        if material.source_fallback_image_sha256.is_some() {
            return Err(error(
                "MATERIAL-SEPARATION-SOURCE-IMAGE-MISMATCH",
                format!("{path}.sourceFallbackImageSha256"),
                "a null source material cannot have a source image hash",
            ));
        }
        return Ok((None, None));
    };
    let source_material = ir
        .materials
        .iter()
        .find(|candidate| candidate.id == source_material_id)
        .ok_or_else(|| {
            error(
                "MATERIAL-SEPARATION-SOURCE-MATERIAL-MISSING",
                format!("{path}.sourceFallbackMaterialId"),
                "source fallback material does not exist",
            )
        })?;
    let image_sha256 = match &source_material.base_color_texture {
        None => None,
        Some(binding) => {
            let texture = ir
                .textures
                .iter()
                .find(|texture| texture.id == binding.texture_id)
                .ok_or_else(|| {
                    error(
                        "MATERIAL-SEPARATION-SOURCE-TEXTURE-MISSING",
                        format!("materials[{source_material_id}].baseColorTexture"),
                        "source base-color texture does not exist",
                    )
                })?;
            Some(
                ir.images
                    .iter()
                    .find(|image| image.id == texture.source_image_id)
                    .ok_or_else(|| {
                        error(
                            "MATERIAL-SEPARATION-SOURCE-IMAGE-MISSING",
                            format!("textures[{}].sourceImageId", texture.id),
                            "source base-color image does not exist",
                        )
                    })?
                    .sha256
                    .clone(),
            )
        }
    };
    if material.source_fallback_image_sha256 != image_sha256 {
        return Err(error(
            "MATERIAL-SEPARATION-SOURCE-IMAGE-MISMATCH",
            format!("{path}.sourceFallbackImageSha256"),
            "source fallback image identity no longer matches the GLB",
        ));
    }
    Ok((source_material.name.clone(), image_sha256))
}

fn source_material_key(material_id: Option<u32>) -> String {
    material_id.map_or_else(|| "source:none".to_owned(), |id| format!("source:{id}"))
}

fn source_material_details(
    ir: &AuroraAssetIr,
    source_material_id: Option<u32>,
) -> Result<(Option<String>, Option<String>), ModelMaterialSeparationErrorV1> {
    let material = match source_material_id {
        None => return Ok((None, None)),
        Some(id) => ir
            .materials
            .iter()
            .find(|material| material.id == id)
            .ok_or_else(|| {
                error(
                    "MATERIAL-SEPARATION-SOURCE-MATERIAL-MISSING",
                    "source.ir.primitives.materialId",
                    format!("source material {id} does not exist"),
                )
            })?,
    };
    let image_sha256 = match &material.base_color_texture {
        None => None,
        Some(binding) => {
            let texture = ir
                .textures
                .iter()
                .find(|texture| texture.id == binding.texture_id)
                .ok_or_else(|| {
                    error(
                        "MATERIAL-SEPARATION-SOURCE-TEXTURE-MISSING",
                        format!("materials[{}].baseColorTexture", material.id),
                        "source base-color texture does not exist",
                    )
                })?;
            Some(
                ir.images
                    .iter()
                    .find(|image| image.id == texture.source_image_id)
                    .ok_or_else(|| {
                        error(
                            "MATERIAL-SEPARATION-SOURCE-IMAGE-MISSING",
                            format!("textures[{}].sourceImageId", texture.id),
                            "source base-color image does not exist",
                        )
                    })?
                    .sha256
                    .clone(),
            )
        }
    };
    Ok((material.name.clone(), image_sha256))
}

pub fn resolve_model_materials_v1(
    ir: &AuroraAssetIr,
    document: &ModelMaterialSeparationDocumentV1,
) -> Result<ResolvedModelMaterialsV1, ModelMaterialSeparationErrorV1> {
    if document.schema_version != MODEL_MATERIAL_SEPARATION_SCHEMA_VERSION_V1 {
        return Err(error(
            "MATERIAL-SEPARATION-SCHEMA-UNSUPPORTED",
            "schemaVersion",
            "only Material Separation schema version 1 is supported",
        ));
    }
    if document.source_sha256 != ir.source.sha256 {
        return Err(error(
            "MATERIAL-SEPARATION-SOURCE-MISMATCH",
            "sourceSha256",
            "Material Separation document is bound to a different source GLB",
        ));
    }
    let inventory = inspect_model_components_v1(ir).map_err(component_error)?;
    let component_inventory = inventory
        .components
        .iter()
        .map(|component| (component.key, component))
        .collect::<BTreeMap<_, _>>();

    let mut authored_materials =
        BTreeMap::<String, (&AuthoredMaterialV1, Option<String>, Option<String>)>::new();
    for (index, material) in document.materials.iter().enumerate() {
        let (source_name, source_image_sha256) =
            validate_material(ir, material, &format!("materials[{index}]"))?;
        if authored_materials
            .insert(
                material.authored_material_id.clone(),
                (material, source_name, source_image_sha256),
            )
            .is_some()
        {
            return Err(error(
                "MATERIAL-SEPARATION-MATERIAL-DUPLICATE",
                format!("materials[{index}].authoredMaterialId"),
                "authored material ids must be unique",
            ));
        }
    }

    let mut assignment_by_component = BTreeMap::<SourceComponentKeyV1, &str>::new();
    for (index, assignment) in document.assignments.iter().enumerate() {
        if !component_inventory.contains_key(&assignment.component) {
            return Err(error(
                "MATERIAL-SEPARATION-COMPONENT-MISSING",
                format!("assignments[{index}].component"),
                "source component does not exist in the exact GLB",
            ));
        }
        if !authored_materials.contains_key(&assignment.authored_material_id) {
            return Err(error(
                "MATERIAL-SEPARATION-MATERIAL-MISSING",
                format!("assignments[{index}].authoredMaterialId"),
                "assignment references an unknown authored material",
            ));
        }
        if assignment_by_component
            .insert(assignment.component, &assignment.authored_material_id)
            .is_some()
        {
            return Err(error(
                "MATERIAL-SEPARATION-COMPONENT-OVERLAP",
                format!("assignments[{index}].component"),
                "a source component can be assigned at most once",
            ));
        }
    }

    let used_authored_ids = assignment_by_component
        .values()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    let used_source_materials = inventory
        .components
        .iter()
        .filter(|component| !assignment_by_component.contains_key(&component.key))
        .map(|component| component.source_material_id)
        .collect::<BTreeSet<_>>();

    let mut ordered_keys = Vec::<String>::new();
    if used_source_materials.contains(&None) {
        ordered_keys.push(source_material_key(None));
    }
    for material in &ir.materials {
        if used_source_materials.contains(&Some(material.id)) {
            ordered_keys.push(source_material_key(Some(material.id)));
        }
    }
    ordered_keys.extend(used_authored_ids.iter().cloned());
    let slot_by_key = ordered_keys
        .iter()
        .enumerate()
        .map(|(slot, key)| {
            u32::try_from(slot)
                .map(|slot| (key.clone(), slot))
                .map_err(|_| {
                    error(
                        "MATERIAL-SEPARATION-SLOT-OVERFLOW",
                        "materials",
                        "resolved material slot exceeds u32",
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    let mut component_counts = BTreeMap::<String, u32>::new();
    let mut triangle_counts = BTreeMap::<String, u32>::new();
    let mut triangle_material_slots = BTreeMap::new();
    let mut output_sections = BTreeSet::new();
    for component in &inventory.components {
        let material_key = assignment_by_component.get(&component.key).map_or_else(
            || source_material_key(component.source_material_id),
            |authored_id| (*authored_id).to_owned(),
        );
        let slot = *slot_by_key.get(&material_key).ok_or_else(|| {
            error(
                "MATERIAL-SEPARATION-INTERNAL-CONTRACT",
                "materialSlots",
                "used material has no resolved slot",
            )
        })?;
        *component_counts.entry(material_key.clone()).or_default() += 1;
        *triangle_counts.entry(material_key.clone()).or_default() += component.triangle_count;
        output_sections.insert((
            component.key.scene_id,
            component.key.node_id,
            component.key.primitive_id,
            slot,
        ));
        let primitive = ir
            .primitives
            .iter()
            .find(|primitive| primitive.id == component.key.primitive_id)
            .ok_or_else(|| {
                error(
                    "MATERIAL-SEPARATION-PRIMITIVE-MISSING",
                    "component.primitiveId",
                    "validated component primitive is missing",
                )
            })?;
        let connected = connected_components_v1(primitive, "source.ir.primitives.indices")
            .map_err(component_error)?;
        let selected = connected
            .get(component.key.component_index as usize)
            .ok_or_else(|| {
                error(
                    "MATERIAL-SEPARATION-COMPONENT-MISSING",
                    "component.componentIndex",
                    "validated connected component is missing",
                )
            })?;
        for triangle_index in &selected.triangle_indices {
            let triangle_index = u32::try_from(*triangle_index).map_err(|_| {
                error(
                    "MATERIAL-SEPARATION-TRIANGLE-OVERFLOW",
                    "source.ir.primitives.indices",
                    "triangle index exceeds u32",
                )
            })?;
            if triangle_material_slots
                .insert(
                    SourceTriangleKeyV1 {
                        scene_id: component.key.scene_id,
                        node_id: component.key.node_id,
                        primitive_id: component.key.primitive_id,
                        triangle_index,
                    },
                    slot,
                )
                .is_some()
            {
                return Err(error(
                    "MATERIAL-SEPARATION-TRIANGLE-OVERLAP",
                    "assignments",
                    "a source triangle resolved more than once",
                ));
            }
        }
    }

    let mut material_slots = Vec::new();
    for (slot, material_key) in ordered_keys.iter().enumerate() {
        let slot = u32::try_from(slot).map_err(|_| {
            error(
                "MATERIAL-SEPARATION-SLOT-OVERFLOW",
                "materialSlots",
                "resolved material slot exceeds u32",
            )
        })?;
        let (
            display_name,
            preview_color,
            source_material_id,
            source_material_name,
            source_image_sha256,
            system_source_material,
        ) = if let Some(source_id) = material_key.strip_prefix("source:") {
            let source_material_id = if source_id == "none" {
                None
            } else {
                Some(source_id.parse::<u32>().map_err(|_| {
                    error(
                        "MATERIAL-SEPARATION-INTERNAL-CONTRACT",
                        "materialSlots",
                        "system source material key is invalid",
                    )
                })?)
            };
            let (source_name, image_sha256) = source_material_details(ir, source_material_id)?;
            (
                source_name
                    .clone()
                    .unwrap_or_else(|| "Source material".to_owned()),
                "#808080".to_owned(),
                source_material_id,
                source_name,
                image_sha256,
                true,
            )
        } else {
            let (material, source_name, image_sha256) =
                authored_materials.get(material_key).ok_or_else(|| {
                    error(
                        "MATERIAL-SEPARATION-INTERNAL-CONTRACT",
                        "materialSlots",
                        "authored material key is missing",
                    )
                })?;
            (
                material.display_name.clone(),
                material.preview_color.clone(),
                material.source_fallback_material_id,
                source_name.clone(),
                image_sha256.clone(),
                false,
            )
        };
        material_slots.push(ResolvedModelMaterialSlotV1 {
            material_slot: slot,
            authored_material_id: material_key.clone(),
            display_name,
            preview_color,
            source_material_id,
            source_material_name,
            source_image_sha256,
            system_source_material,
            component_count: *component_counts.get(material_key).unwrap_or(&0),
            triangle_count: *triangle_counts.get(material_key).unwrap_or(&0),
        });
    }

    let unused_authored_material_ids = authored_materials
        .keys()
        .filter(|id| !used_authored_ids.contains(*id))
        .cloned()
        .collect();
    let source_component_count = u32::try_from(inventory.components.len()).map_err(|_| {
        error(
            "MATERIAL-SEPARATION-COMPONENT-OVERFLOW",
            "components",
            "source component count exceeds u32",
        )
    })?;
    let assigned_component_count = u32::try_from(assignment_by_component.len()).map_err(|_| {
        error(
            "MATERIAL-SEPARATION-COMPONENT-OVERFLOW",
            "assignments",
            "assigned component count exceeds u32",
        )
    })?;
    let output_triangle_count = u32::try_from(triangle_material_slots.len()).map_err(|_| {
        error(
            "MATERIAL-SEPARATION-TRIANGLE-OVERFLOW",
            "triangles",
            "output triangle count exceeds u32",
        )
    })?;
    let source_vertex_count = inventory
        .components
        .iter()
        .try_fold(0u32, |sum, component| {
            sum.checked_add(component.vertex_count)
        })
        .ok_or_else(|| {
            error(
                "MATERIAL-SEPARATION-VERTEX-OVERFLOW",
                "components",
                "source vertex count exceeds u32",
            )
        })?;
    let output_section_count = u32::try_from(output_sections.len()).map_err(|_| {
        error(
            "MATERIAL-SEPARATION-SECTION-OVERFLOW",
            "materialSlots",
            "output section count exceeds u32",
        )
    })?;
    let mut warnings = vec!["MATERIAL-SEPARATION-UV0-UNCHANGED".to_owned()];
    if inventory.components.len() == 1 && inventory.triangle_count > 1 {
        warnings.push("MATERIAL-SEPARATION-COMPONENT-GRANULARITY-INSUFFICIENT".to_owned());
    }
    let predicted_texture_count = u32::try_from(material_slots.len()).map_err(|_| {
        error(
            "MATERIAL-SEPARATION-TEXTURE-COUNT-OVERFLOW",
            "materialSlots",
            "predicted texture count exceeds u32",
        )
    })?;
    let report = ModelMaterialSeparationReportV1 {
        schema_version: MODEL_MATERIAL_SEPARATION_SCHEMA_VERSION_V1,
        source_sha256: document.source_sha256.clone(),
        separation_sha256: model_material_separation_hash_v1(document)?,
        source_component_count,
        assigned_component_count,
        unassigned_component_count: source_component_count - assigned_component_count,
        source_triangle_count: inventory.triangle_count,
        output_triangle_count,
        source_vertex_count,
        output_vertex_count: source_vertex_count,
        duplicated_boundary_vertex_count: 0,
        output_section_count,
        predicted_texture_count,
        material_slots,
        unused_authored_material_ids,
        warnings,
    };
    if report.source_triangle_count != report.output_triangle_count {
        return Err(error(
            "MATERIAL-SEPARATION-TRIANGLE-COVERAGE",
            "triangles",
            "every source triangle must resolve exactly once",
        ));
    }
    Ok(ResolvedModelMaterialsV1 {
        report,
        triangle_material_slots,
    })
}
