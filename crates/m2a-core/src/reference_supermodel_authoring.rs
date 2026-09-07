//! Owned, hash-bound rig authoring layered on top of a reference-supermodel target rig.
//!
//! The document intentionally cannot rename, add, delete or reparent carrier nodes. It may only
//! override an existing carrier's local bind matrix and sparse rows of owned mesh weights.

use std::{
    collections::{BTreeSet, HashSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::profile_a::{
    CreatureRigProfileV1, CreatureSourceForwardV1, RigWeightInfluenceV1, canonical_profile_sha256,
};
use crate::reference_supermodel_generic::ReferenceSupermodelChainResourceAnalysisV2;

pub const REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1: u32 = 1;
pub const REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V2: u32 = 2;

/// Stable digest of the exact selected reference chain. Payload bytes are represented by each
/// resource's already verified SHA-256 and byte length; no retail/reference payload is copied.
pub fn reference_supermodel_exact_chain_sha256_v1(
    selected_supermodel_resref: &str,
    exact_chain: &[ReferenceSupermodelChainResourceAnalysisV2],
) -> Result<String, ReferenceSupermodelRigAuthoringErrorV1> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashView<'a> {
        schema_version: u32,
        selected_supermodel_resref: &'a str,
        exact_chain: &'a [ReferenceSupermodelChainResourceAnalysisV2],
    }
    let bytes = serde_json::to_vec(&HashView {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1,
        selected_supermodel_resref,
        exact_chain,
    })
    .map_err(|error| {
        authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-CHAIN-HASH-FAILED",
            "$.exactChain",
            &format!("the exact chain could not be canonicalized: {error}"),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelJointOverrideV1 {
    /// Stable carrier part number. In the generic carrier route this is the target rig node id.
    pub carrier_part_number: u32,
    /// Column-major affine local bind matrix for the owned target rig.
    pub bind_local_matrix: [f32; 16],
    /// Optional authoring metadata. It never changes the inherited controller name.
    pub semantic_role: Option<String>,
    /// Optional normalized local joint axis used by authoring tools.
    pub joint_axis: Option<[f32; 3]>,
    /// UI-level edit lock persisted with the authored override.
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelVertexWeightOverrideV1 {
    pub segment_id: u32,
    pub vertex_index: usize,
    pub influences: Vec<RigWeightInfluenceV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringDocumentV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub source_forward: CreatureSourceForwardV1,
    pub selected_supermodel_resref: String,
    pub exact_chain_sha256: String,
    pub motion_contract_sha256: String,
    pub base_rig_sha256: String,
    pub joint_overrides: Vec<ReferenceSupermodelJointOverrideV1>,
    pub weight_overrides: Vec<ReferenceSupermodelVertexWeightOverrideV1>,
    pub content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub authoring_sha256: String,
    pub base_rig_sha256: String,
    pub output_rig_sha256: String,
    pub joint_override_count: usize,
    pub weight_override_count: usize,
    pub semantic_override_count: usize,
    pub carrier_topology_preserved: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringArtifactV1 {
    pub rig: CreatureRigProfileV1,
    pub report: ReferenceSupermodelRigAuthoringReportV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelLandmarkOverrideV2 {
    pub landmark_id: String,
    pub carrier_part_number: u32,
    pub target_world_position: [f32; 3],
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelComponentBindingV2 {
    pub segment_id: u32,
    pub component_index: usize,
    pub region_id: String,
    pub allowed_bone_node_ids: Vec<u32>,
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRegionWeightConstraintV2 {
    pub segment_id: u32,
    pub region_id: String,
    pub vertex_indices: Vec<usize>,
    pub allowed_bone_node_ids: Vec<u32>,
    pub forbidden_bone_node_ids: Vec<u32>,
    pub maximum_influence_count: usize,
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringDocumentV2 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub source_forward: CreatureSourceForwardV1,
    pub selected_supermodel_resref: String,
    pub exact_chain_sha256: String,
    pub motion_contract_sha256: String,
    pub structural_profile_sha256: String,
    pub surface_anatomy_sha256: String,
    pub fitter_algorithm: String,
    pub base_rig_sha256: String,
    pub landmark_overrides: Vec<ReferenceSupermodelLandmarkOverrideV2>,
    pub joint_overrides: Vec<ReferenceSupermodelJointOverrideV1>,
    pub component_bindings: Vec<ReferenceSupermodelComponentBindingV2>,
    pub region_weight_constraints: Vec<ReferenceSupermodelRegionWeightConstraintV2>,
    pub weight_overrides: Vec<ReferenceSupermodelVertexWeightOverrideV1>,
    pub content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringReportV2 {
    pub schema_version: u32,
    pub status: String,
    pub authoring_sha256: String,
    pub base_rig_sha256: String,
    pub output_rig_sha256: String,
    pub landmark_override_count: usize,
    pub joint_override_count: usize,
    pub component_binding_count: usize,
    pub region_weight_constraint_count: usize,
    pub weight_override_count: usize,
    pub carrier_topology_preserved: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringArtifactV2 {
    pub rig: CreatureRigProfileV1,
    pub report: ReferenceSupermodelRigAuthoringReportV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigAuthoringErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ReferenceSupermodelRigAuthoringErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl Error for ReferenceSupermodelRigAuthoringErrorV1 {}

pub fn new_reference_supermodel_rig_authoring_v1(
    source_sha256: &str,
    source_forward: CreatureSourceForwardV1,
    selected_supermodel_resref: &str,
    exact_chain_sha256: &str,
    motion_contract_sha256: &str,
    base_rig: &CreatureRigProfileV1,
) -> Result<ReferenceSupermodelRigAuthoringDocumentV1, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_base_rig(base_rig)?;
    seal_reference_supermodel_rig_authoring_v1(ReferenceSupermodelRigAuthoringDocumentV1 {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: source_sha256.to_owned(),
        source_forward,
        selected_supermodel_resref: selected_supermodel_resref.to_owned(),
        exact_chain_sha256: exact_chain_sha256.to_owned(),
        motion_contract_sha256: motion_contract_sha256.to_owned(),
        base_rig_sha256: base_rig.content_sha256.clone(),
        joint_overrides: Vec::new(),
        weight_overrides: Vec::new(),
        content_sha256: String::new(),
    })
}

pub fn seal_reference_supermodel_rig_authoring_v1(
    mut document: ReferenceSupermodelRigAuthoringDocumentV1,
) -> Result<ReferenceSupermodelRigAuthoringDocumentV1, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_document_shape(&document)?;
    normalize_document_zeroes(&mut document);
    document
        .joint_overrides
        .sort_by_key(|override_row| override_row.carrier_part_number);
    document
        .weight_overrides
        .sort_by_key(|override_row| (override_row.segment_id, override_row.vertex_index));
    document.content_sha256 = canonical_authoring_sha256(&document)?;
    Ok(document)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_reference_supermodel_rig_authoring_v1(
    document: &ReferenceSupermodelRigAuthoringDocumentV1,
    expected_source_sha256: &str,
    expected_source_forward: CreatureSourceForwardV1,
    expected_selected_supermodel_resref: &str,
    expected_exact_chain_sha256: &str,
    expected_motion_contract_sha256: &str,
    base_rig: &CreatureRigProfileV1,
) -> Result<ReferenceSupermodelRigAuthoringArtifactV1, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_document_shape(document)?;
    let expected_document_hash = canonical_authoring_sha256(document)?;
    if document.content_sha256 != expected_document_hash {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-HASH-MISMATCH",
            "$.contentSha256",
            "the authored rig document was modified after it was sealed",
        ));
    }
    validate_base_rig(base_rig)?;
    if document.source_sha256 != expected_source_sha256
        || document.source_forward != expected_source_forward
        || document.selected_supermodel_resref != expected_selected_supermodel_resref
        || document.exact_chain_sha256 != expected_exact_chain_sha256
        || document.motion_contract_sha256 != expected_motion_contract_sha256
        || document.base_rig_sha256 != base_rig.content_sha256
    {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-MISMATCH",
            "$",
            "the authored rig belongs to a different source, direction, supermodel chain, motion contract or base rig",
        ));
    }

    let topology_before = base_rig
        .nodes
        .iter()
        .map(|node| (node.id, node.name.clone(), node.parent_id))
        .collect::<Vec<_>>();
    let mut rig = base_rig.clone();

    for override_row in &document.joint_overrides {
        let node = rig
            .nodes
            .iter_mut()
            .find(|node| node.id == override_row.carrier_part_number)
            .ok_or_else(|| {
                authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-JOINT-NOT-FOUND",
                    "$.jointOverrides[].carrierPartNumber",
                    "the override does not identify an existing target carrier node",
                )
            })?;
        node.bind_local_matrix = override_row.bind_local_matrix;
    }

    for override_row in &document.weight_overrides {
        let segment = rig
            .segments
            .iter_mut()
            .find(|segment| segment.id == override_row.segment_id)
            .ok_or_else(|| {
                authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-SEGMENT-NOT-FOUND",
                    "$.weightOverrides[].segmentId",
                    "the override does not identify an existing owned surface segment",
                )
            })?;
        if override_row.vertex_index >= segment.reference_weights.len() {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-VERTEX-NOT-FOUND",
                "$.weightOverrides[].vertexIndex",
                "the override vertex index is outside the selected segment",
            ));
        }
        for influence in &override_row.influences {
            if !segment
                .allowed_bone_node_ids
                .contains(&influence.bone_node_id)
            {
                return Err(authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-BONE-NOT-ALLOWED",
                    "$.weightOverrides[].influences[].boneNodeId",
                    "the selected carrier is not allowed by the segment skin contract",
                ));
            }
        }
        segment.reference_weights[override_row.vertex_index] = override_row.influences.clone();
    }

    let topology_after = rig
        .nodes
        .iter()
        .map(|node| (node.id, node.name.clone(), node.parent_id))
        .collect::<Vec<_>>();
    if topology_before != topology_after {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-TOPOLOGY-MUTATED",
            "$.nodes",
            "carrier ids, controller names or parent links changed during authoring",
        ));
    }

    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|error| {
        authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-RIG-HASH-FAILED",
            "$.rig",
            &format!("the authored target rig could not be canonicalized: {error}"),
        )
    })?;
    Ok(ReferenceSupermodelRigAuthoringArtifactV1 {
        report: ReferenceSupermodelRigAuthoringReportV1 {
            schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1,
            status: "ready".to_owned(),
            authoring_sha256: document.content_sha256.clone(),
            base_rig_sha256: base_rig.content_sha256.clone(),
            output_rig_sha256: rig.content_sha256.clone(),
            joint_override_count: document.joint_overrides.len(),
            weight_override_count: document.weight_overrides.len(),
            semantic_override_count: document
                .joint_overrides
                .iter()
                .filter(|override_row| {
                    override_row.semantic_role.is_some() || override_row.joint_axis.is_some()
                })
                .count(),
            carrier_topology_preserved: true,
        },
        rig,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn new_reference_supermodel_rig_authoring_v2(
    source_sha256: &str,
    source_forward: CreatureSourceForwardV1,
    selected_supermodel_resref: &str,
    exact_chain_sha256: &str,
    motion_contract_sha256: &str,
    structural_profile_sha256: &str,
    surface_anatomy_sha256: &str,
    fitter_algorithm: &str,
    base_rig: &CreatureRigProfileV1,
) -> Result<ReferenceSupermodelRigAuthoringDocumentV2, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_base_rig(base_rig)?;
    seal_reference_supermodel_rig_authoring_v2(ReferenceSupermodelRigAuthoringDocumentV2 {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V2,
        source_sha256: source_sha256.to_owned(),
        source_forward,
        selected_supermodel_resref: selected_supermodel_resref.to_owned(),
        exact_chain_sha256: exact_chain_sha256.to_owned(),
        motion_contract_sha256: motion_contract_sha256.to_owned(),
        structural_profile_sha256: structural_profile_sha256.to_owned(),
        surface_anatomy_sha256: surface_anatomy_sha256.to_owned(),
        fitter_algorithm: fitter_algorithm.to_owned(),
        base_rig_sha256: base_rig.content_sha256.clone(),
        landmark_overrides: Vec::new(),
        joint_overrides: Vec::new(),
        component_bindings: Vec::new(),
        region_weight_constraints: Vec::new(),
        weight_overrides: Vec::new(),
        content_sha256: String::new(),
    })
}

pub fn migrate_reference_supermodel_rig_authoring_v1_to_v2(
    document: &ReferenceSupermodelRigAuthoringDocumentV1,
    structural_profile_sha256: &str,
    surface_anatomy_sha256: &str,
    fitter_algorithm: &str,
) -> Result<ReferenceSupermodelRigAuthoringDocumentV2, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_document_shape(document)?;
    if canonical_authoring_sha256(document)? != document.content_sha256 {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-HASH-MISMATCH",
            "$.contentSha256",
            "the V1 authoring document was modified after it was sealed",
        ));
    }
    seal_reference_supermodel_rig_authoring_v2(ReferenceSupermodelRigAuthoringDocumentV2 {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V2,
        source_sha256: document.source_sha256.clone(),
        source_forward: document.source_forward,
        selected_supermodel_resref: document.selected_supermodel_resref.clone(),
        exact_chain_sha256: document.exact_chain_sha256.clone(),
        motion_contract_sha256: document.motion_contract_sha256.clone(),
        structural_profile_sha256: structural_profile_sha256.to_owned(),
        surface_anatomy_sha256: surface_anatomy_sha256.to_owned(),
        fitter_algorithm: fitter_algorithm.to_owned(),
        base_rig_sha256: document.base_rig_sha256.clone(),
        landmark_overrides: Vec::new(),
        joint_overrides: document.joint_overrides.clone(),
        component_bindings: Vec::new(),
        region_weight_constraints: Vec::new(),
        weight_overrides: document.weight_overrides.clone(),
        content_sha256: String::new(),
    })
}

pub fn seal_reference_supermodel_rig_authoring_v2(
    mut document: ReferenceSupermodelRigAuthoringDocumentV2,
) -> Result<ReferenceSupermodelRigAuthoringDocumentV2, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_document_shape_v2(&document)?;
    normalize_document_zeroes_v2(&mut document);
    document
        .landmark_overrides
        .sort_by_key(|row| (row.carrier_part_number, row.landmark_id.clone()));
    document
        .joint_overrides
        .sort_by_key(|row| row.carrier_part_number);
    document
        .component_bindings
        .sort_by_key(|row| (row.segment_id, row.component_index));
    document
        .region_weight_constraints
        .sort_by_key(|row| (row.segment_id, row.region_id.clone()));
    for row in &mut document.component_bindings {
        row.allowed_bone_node_ids.sort_unstable();
        row.allowed_bone_node_ids.dedup();
    }
    for row in &mut document.region_weight_constraints {
        row.vertex_indices.sort_unstable();
        row.vertex_indices.dedup();
        row.allowed_bone_node_ids.sort_unstable();
        row.allowed_bone_node_ids.dedup();
        row.forbidden_bone_node_ids.sort_unstable();
        row.forbidden_bone_node_ids.dedup();
    }
    document
        .weight_overrides
        .sort_by_key(|row| (row.segment_id, row.vertex_index));
    document.content_sha256 = canonical_authoring_sha256_v2(&document)?;
    Ok(document)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_reference_supermodel_rig_authoring_v2(
    document: &ReferenceSupermodelRigAuthoringDocumentV2,
    expected_source_sha256: &str,
    expected_source_forward: CreatureSourceForwardV1,
    expected_selected_supermodel_resref: &str,
    expected_exact_chain_sha256: &str,
    expected_motion_contract_sha256: &str,
    expected_structural_profile_sha256: &str,
    expected_surface_anatomy_sha256: &str,
    expected_fitter_algorithm: &str,
    base_rig: &CreatureRigProfileV1,
) -> Result<ReferenceSupermodelRigAuthoringArtifactV2, ReferenceSupermodelRigAuthoringErrorV1> {
    validate_document_shape_v2(document)?;
    if document.content_sha256 != canonical_authoring_sha256_v2(document)? {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-HASH-MISMATCH",
            "$.contentSha256",
            "the V2 authoring document was modified after it was sealed",
        ));
    }
    validate_base_rig(base_rig)?;
    if !document.joint_overrides.is_empty() || !document.landmark_overrides.is_empty() {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-IMMUTABLE-BIND-OVERRIDE-FORBIDDEN",
            "$.jointOverrides|$.landmarkOverrides",
            "selected supermodel carrier transforms are immutable; author mesh registration, component binding or skin weights instead",
        ));
    }
    if document.source_sha256 != expected_source_sha256
        || document.source_forward != expected_source_forward
        || document.selected_supermodel_resref != expected_selected_supermodel_resref
        || document.exact_chain_sha256 != expected_exact_chain_sha256
        || document.motion_contract_sha256 != expected_motion_contract_sha256
        || document.structural_profile_sha256 != expected_structural_profile_sha256
        || document.surface_anatomy_sha256 != expected_surface_anatomy_sha256
        || document.fitter_algorithm != expected_fitter_algorithm
        || document.base_rig_sha256 != base_rig.content_sha256
    {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-MISMATCH",
            "$",
            "the V2 authoring belongs to a different source, exact chain, structural profile, anatomy analysis, fitter or base rig",
        ));
    }
    let topology_before = topology_identity(base_rig);
    let mut rig = base_rig.clone();
    apply_component_bindings_v2(&mut rig, &document.component_bindings)?;
    apply_region_weight_constraints_v2(&mut rig, &document.region_weight_constraints)?;
    apply_weight_overrides(&mut rig, &document.weight_overrides)?;
    if topology_before != topology_identity(&rig) {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-TOPOLOGY-MUTATED",
            "$.nodes",
            "carrier ids, controller names or parent links changed during V2 authoring",
        ));
    }
    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).map_err(|error| {
        authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-RIG-HASH-FAILED",
            "$.rig",
            &format!("the V2 authored target rig could not be canonicalized: {error}"),
        )
    })?;
    Ok(ReferenceSupermodelRigAuthoringArtifactV2 {
        report: ReferenceSupermodelRigAuthoringReportV2 {
            schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V2,
            status: "ready".to_owned(),
            authoring_sha256: document.content_sha256.clone(),
            base_rig_sha256: base_rig.content_sha256.clone(),
            output_rig_sha256: rig.content_sha256.clone(),
            landmark_override_count: document.landmark_overrides.len(),
            joint_override_count: document.joint_overrides.len(),
            component_binding_count: document.component_bindings.len(),
            region_weight_constraint_count: document.region_weight_constraints.len(),
            weight_override_count: document.weight_overrides.len(),
            carrier_topology_preserved: true,
        },
        rig,
    })
}

fn validate_document_shape_v2(
    document: &ReferenceSupermodelRigAuthoringDocumentV2,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    if document.schema_version != REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V2 {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-SCHEMA-UNSUPPORTED",
            "$.schemaVersion",
            "expected reference-supermodel rig authoring schema version 2",
        ));
    }
    for (path, hash) in [
        ("$.sourceSha256", document.source_sha256.as_str()),
        ("$.exactChainSha256", document.exact_chain_sha256.as_str()),
        (
            "$.motionContractSha256",
            document.motion_contract_sha256.as_str(),
        ),
        (
            "$.structuralProfileSha256",
            document.structural_profile_sha256.as_str(),
        ),
        (
            "$.surfaceAnatomySha256",
            document.surface_anatomy_sha256.as_str(),
        ),
        ("$.baseRigSha256", document.base_rig_sha256.as_str()),
    ] {
        if !is_lower_sha256(hash) {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-INVALID",
                path,
                "expected a lowercase SHA-256 digest",
            ));
        }
    }
    if !is_resref(&document.selected_supermodel_resref)
        || !is_identifier(&document.fitter_algorithm, 128)
    {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-INVALID",
            "$",
            "selected supermodel and fitter algorithm identifiers are invalid",
        ));
    }
    let mut landmark_keys = HashSet::new();
    for row in &document.landmark_overrides {
        if !landmark_keys.insert((row.landmark_id.clone(), row.carrier_part_number))
            || !is_identifier(&row.landmark_id, 128)
            || row
                .target_world_position
                .iter()
                .any(|value| !value.is_finite())
        {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-LANDMARK-INVALID",
                "$.landmarkOverrides",
                "landmark overrides must be unique, finite and use stable identifiers",
            ));
        }
    }
    let legacy_shape = ReferenceSupermodelRigAuthoringDocumentV1 {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: document.source_sha256.clone(),
        source_forward: document.source_forward,
        selected_supermodel_resref: document.selected_supermodel_resref.clone(),
        exact_chain_sha256: document.exact_chain_sha256.clone(),
        motion_contract_sha256: document.motion_contract_sha256.clone(),
        base_rig_sha256: document.base_rig_sha256.clone(),
        joint_overrides: document.joint_overrides.clone(),
        weight_overrides: document.weight_overrides.clone(),
        content_sha256: String::new(),
    };
    validate_document_shape(&legacy_shape)?;
    let mut component_keys = HashSet::new();
    for row in &document.component_bindings {
        if !component_keys.insert((row.segment_id, row.component_index))
            || !is_identifier(&row.region_id, 128)
            || row.allowed_bone_node_ids.is_empty()
            || contains_duplicates(&row.allowed_bone_node_ids)
        {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-COMPONENT-BINDING-INVALID",
                "$.componentBindings",
                "component bindings must be unique and contain a non-empty allowed carrier set",
            ));
        }
    }
    let mut region_keys = HashSet::new();
    for row in &document.region_weight_constraints {
        let allowed = row
            .allowed_bone_node_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let forbidden = row
            .forbidden_bone_node_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if !region_keys.insert((row.segment_id, row.region_id.clone()))
            || !is_identifier(&row.region_id, 128)
            || row.vertex_indices.is_empty()
            || contains_duplicates(&row.vertex_indices)
            || contains_duplicates(&row.allowed_bone_node_ids)
            || contains_duplicates(&row.forbidden_bone_node_ids)
            || !allowed.is_disjoint(&forbidden)
            || !(1..=4).contains(&row.maximum_influence_count)
        {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-REGION-CONSTRAINT-INVALID",
                "$.regionWeightConstraints",
                "region constraints require unique vertices, disjoint carrier sets and a one-to-four influence limit",
            ));
        }
    }
    Ok(())
}

fn apply_component_bindings_v2(
    rig: &mut CreatureRigProfileV1,
    bindings: &[ReferenceSupermodelComponentBindingV2],
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    for row in bindings {
        let segment = rig
            .segments
            .iter_mut()
            .find(|segment| segment.id == row.segment_id)
            .ok_or_else(|| {
                authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-SEGMENT-NOT-FOUND",
                    "$.componentBindings[].segmentId",
                    "component binding references a missing segment",
                )
            })?;
        ensure_allowed_bones(segment, &row.allowed_bone_node_ids, "$.componentBindings")?;
        let components =
            segment_components(segment.surface_positions.len(), &segment.surface_indices)?;
        let vertices = components.get(row.component_index).ok_or_else(|| {
            authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-COMPONENT-NOT-FOUND",
                "$.componentBindings[].componentIndex",
                "component binding references a missing surface component",
            )
        })?;
        constrain_weight_rows(
            &mut segment.reference_weights,
            vertices,
            &row.allowed_bone_node_ids,
            &[],
            4,
            "$.componentBindings",
        )?;
    }
    Ok(())
}

fn apply_region_weight_constraints_v2(
    rig: &mut CreatureRigProfileV1,
    constraints: &[ReferenceSupermodelRegionWeightConstraintV2],
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    for row in constraints {
        let segment = rig
            .segments
            .iter_mut()
            .find(|segment| segment.id == row.segment_id)
            .ok_or_else(|| {
                authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-SEGMENT-NOT-FOUND",
                    "$.regionWeightConstraints[].segmentId",
                    "region constraint references a missing segment",
                )
            })?;
        ensure_allowed_bones(
            segment,
            &row.allowed_bone_node_ids,
            "$.regionWeightConstraints",
        )?;
        ensure_allowed_bones(
            segment,
            &row.forbidden_bone_node_ids,
            "$.regionWeightConstraints",
        )?;
        constrain_weight_rows(
            &mut segment.reference_weights,
            &row.vertex_indices,
            &row.allowed_bone_node_ids,
            &row.forbidden_bone_node_ids,
            row.maximum_influence_count,
            "$.regionWeightConstraints",
        )?;
    }
    Ok(())
}

fn apply_weight_overrides(
    rig: &mut CreatureRigProfileV1,
    overrides: &[ReferenceSupermodelVertexWeightOverrideV1],
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    for row in overrides {
        let segment = rig
            .segments
            .iter_mut()
            .find(|segment| segment.id == row.segment_id)
            .ok_or_else(|| {
                authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-SEGMENT-NOT-FOUND",
                    "$.weightOverrides[].segmentId",
                    "weight override references a missing segment",
                )
            })?;
        if row.vertex_index >= segment.reference_weights.len() {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-VERTEX-NOT-FOUND",
                "$.weightOverrides[].vertexIndex",
                "weight override references a missing vertex",
            ));
        }
        ensure_allowed_bones(
            segment,
            &row.influences
                .iter()
                .map(|influence| influence.bone_node_id)
                .collect::<Vec<_>>(),
            "$.weightOverrides",
        )?;
        segment.reference_weights[row.vertex_index] = row.influences.clone();
    }
    Ok(())
}

fn constrain_weight_rows(
    weights: &mut [Vec<RigWeightInfluenceV1>],
    vertices: &[usize],
    allowed: &[u32],
    forbidden: &[u32],
    maximum: usize,
    path: &str,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    for vertex in vertices {
        let row = weights.get_mut(*vertex).ok_or_else(|| {
            authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-VERTEX-NOT-FOUND",
                path,
                "constraint references a missing vertex",
            )
        })?;
        row.retain(|influence| {
            (allowed.is_empty() || allowed.contains(&influence.bone_node_id))
                && !forbidden.contains(&influence.bone_node_id)
        });
        row.sort_by(|left, right| {
            right
                .value
                .total_cmp(&left.value)
                .then(left.bone_node_id.cmp(&right.bone_node_id))
        });
        row.truncate(maximum);
        let total = row.iter().map(|influence| influence.value).sum::<f32>();
        if row.is_empty() || !total.is_finite() || total <= 0.0 {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONSTRAINT-EMPTY",
                path,
                "the authored carrier constraint removes every existing influence; add an explicit weight override",
            ));
        }
        for influence in row {
            influence.value /= total;
        }
    }
    Ok(())
}

fn ensure_allowed_bones(
    segment: &crate::profile_a::CreatureRigSegmentV1,
    bones: &[u32],
    path: &str,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    if bones
        .iter()
        .any(|bone| !segment.allowed_bone_node_ids.contains(bone))
    {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-BONE-NOT-ALLOWED",
            path,
            "the authored region references a carrier outside the segment skin contract",
        ));
    }
    Ok(())
}

fn segment_components(
    vertex_count: usize,
    indices: &[u32],
) -> Result<Vec<Vec<usize>>, ReferenceSupermodelRigAuthoringErrorV1> {
    let mut adjacency = vec![Vec::new(); vertex_count];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= vertex_count) {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-SURFACE-INVALID",
                "$.baseRig.segments[].surfaceIndices",
                "surface component analysis encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            adjacency[vertices[left]].push(vertices[right]);
            adjacency[vertices[right]].push(vertices[left]);
        }
    }
    let mut visited = vec![false; vertex_count];
    let mut output = Vec::new();
    for start in 0..vertex_count {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut pending = vec![start];
        let mut component = Vec::new();
        while let Some(vertex) = pending.pop() {
            component.push(vertex);
            for neighbor in &adjacency[vertex] {
                if !visited[*neighbor] {
                    visited[*neighbor] = true;
                    pending.push(*neighbor);
                }
            }
        }
        component.sort_unstable();
        output.push(component);
    }
    Ok(output)
}

fn topology_identity(rig: &CreatureRigProfileV1) -> Vec<(u32, String, Option<u32>)> {
    rig.nodes
        .iter()
        .map(|node| (node.id, node.name.clone(), node.parent_id))
        .collect()
}

fn contains_duplicates<T: Eq + std::hash::Hash>(values: &[T]) -> bool {
    let mut seen = HashSet::new();
    values.iter().any(|value| !seen.insert(value))
}

fn is_identifier(value: &str, maximum_length: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
}

fn validate_document_shape(
    document: &ReferenceSupermodelRigAuthoringDocumentV1,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    if document.schema_version != REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1 {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-SCHEMA-UNSUPPORTED",
            "$.schemaVersion",
            "only reference-supermodel rig authoring schema version 1 is supported",
        ));
    }
    for (path, hash) in [
        ("$.sourceSha256", document.source_sha256.as_str()),
        ("$.exactChainSha256", document.exact_chain_sha256.as_str()),
        (
            "$.motionContractSha256",
            document.motion_contract_sha256.as_str(),
        ),
        ("$.baseRigSha256", document.base_rig_sha256.as_str()),
    ] {
        if !is_lower_sha256(hash) {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-INVALID",
                path,
                "expected a lowercase SHA-256 digest",
            ));
        }
    }
    if !is_resref(&document.selected_supermodel_resref) {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-CONTEXT-INVALID",
            "$.selectedSupermodelResref",
            "expected a lowercase Aurora resref with at most 16 characters",
        ));
    }

    let mut carrier_parts = HashSet::new();
    for (index, override_row) in document.joint_overrides.iter().enumerate() {
        if !carrier_parts.insert(override_row.carrier_part_number) {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-JOINT-DUPLICATE",
                &format!("$.jointOverrides[{index}].carrierPartNumber"),
                "a carrier can have at most one bind override",
            ));
        }
        validate_bind_matrix(&override_row.bind_local_matrix, index)?;
        if let Some(role) = &override_row.semantic_role {
            if role.is_empty()
                || role.len() > 64
                || !role
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            {
                return Err(authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-SEMANTIC-INVALID",
                    &format!("$.jointOverrides[{index}].semanticRole"),
                    "semantic roles must be lowercase identifiers with at most 64 characters",
                ));
            }
        }
        if let Some(axis) = override_row.joint_axis {
            let length_squared = axis
                .iter()
                .map(|component| component * component)
                .sum::<f32>();
            if !axis.iter().all(|component| component.is_finite())
                || (length_squared - 1.0).abs() > 1.0e-4
            {
                return Err(authoring_error(
                    "M2A-REFERENCE-SUPERMODEL-AUTHORING-JOINT-AXIS-INVALID",
                    &format!("$.jointOverrides[{index}].jointAxis"),
                    "joint axes must be finite unit vectors",
                ));
            }
        }
    }

    let mut weight_rows = HashSet::new();
    for (index, override_row) in document.weight_overrides.iter().enumerate() {
        if !weight_rows.insert((override_row.segment_id, override_row.vertex_index)) {
            return Err(authoring_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORING-WEIGHT-DUPLICATE",
                &format!("$.weightOverrides[{index}]"),
                "a segment vertex can have at most one weight override",
            ));
        }
        validate_weight_row(&override_row.influences, index)?;
    }
    Ok(())
}

fn validate_bind_matrix(
    matrix: &[f32; 16],
    index: usize,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    let affine = matrix.iter().all(|value| value.is_finite())
        && matrix[3].abs() <= 1.0e-6
        && matrix[7].abs() <= 1.0e-6
        && matrix[11].abs() <= 1.0e-6
        && (matrix[15] - 1.0).abs() <= 1.0e-6;
    let determinant = matrix[0] * (matrix[5] * matrix[10] - matrix[9] * matrix[6])
        - matrix[4] * (matrix[1] * matrix[10] - matrix[9] * matrix[2])
        + matrix[8] * (matrix[1] * matrix[6] - matrix[5] * matrix[2]);
    if !affine || !determinant.is_finite() || determinant.abs() <= 1.0e-8 {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-JOINT-MATRIX-INVALID",
            &format!("$.jointOverrides[{index}].bindLocalMatrix"),
            "bind matrices must be finite, affine and invertible",
        ));
    }
    Ok(())
}

fn validate_weight_row(
    influences: &[RigWeightInfluenceV1],
    index: usize,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    let mut ids = HashSet::new();
    let sum = influences
        .iter()
        .map(|influence| influence.value)
        .sum::<f32>();
    if influences.is_empty()
        || influences.len() > 4
        || influences.iter().any(|influence| {
            !ids.insert(influence.bone_node_id)
                || !influence.value.is_finite()
                || influence.value <= 0.0
        })
        || !sum.is_finite()
        || (sum - 1.0).abs() > 1.0e-4
    {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-WEIGHT-INVALID",
            &format!("$.weightOverrides[{index}].influences"),
            "weights must contain one to four unique, positive, finite influences summing to one",
        ));
    }
    Ok(())
}

fn validate_base_rig(
    base_rig: &CreatureRigProfileV1,
) -> Result<(), ReferenceSupermodelRigAuthoringErrorV1> {
    let canonical = canonical_profile_sha256(base_rig).map_err(|error| {
        authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-BASE-RIG-INVALID",
            "$.baseRig",
            &format!("the base target rig could not be canonicalized: {error}"),
        )
    })?;
    if base_rig.content_sha256 != canonical {
        return Err(authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-BASE-RIG-HASH-MISMATCH",
            "$.baseRig.contentSha256",
            "the base target rig content does not match its digest",
        ));
    }
    Ok(())
}

fn canonical_authoring_sha256(
    document: &ReferenceSupermodelRigAuthoringDocumentV1,
) -> Result<String, ReferenceSupermodelRigAuthoringErrorV1> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashView<'a> {
        schema_version: u32,
        source_sha256: &'a str,
        source_forward: CreatureSourceForwardV1,
        selected_supermodel_resref: &'a str,
        exact_chain_sha256: &'a str,
        motion_contract_sha256: &'a str,
        base_rig_sha256: &'a str,
        joint_overrides: &'a [ReferenceSupermodelJointOverrideV1],
        weight_overrides: &'a [ReferenceSupermodelVertexWeightOverrideV1],
    }
    let bytes = serde_json::to_vec(&HashView {
        schema_version: document.schema_version,
        source_sha256: &document.source_sha256,
        source_forward: document.source_forward,
        selected_supermodel_resref: &document.selected_supermodel_resref,
        exact_chain_sha256: &document.exact_chain_sha256,
        motion_contract_sha256: &document.motion_contract_sha256,
        base_rig_sha256: &document.base_rig_sha256,
        joint_overrides: &document.joint_overrides,
        weight_overrides: &document.weight_overrides,
    })
    .map_err(|error| {
        authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-HASH-FAILED",
            "$",
            &format!("the authoring document could not be canonicalized: {error}"),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn canonical_authoring_sha256_v2(
    document: &ReferenceSupermodelRigAuthoringDocumentV2,
) -> Result<String, ReferenceSupermodelRigAuthoringErrorV1> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashView<'a> {
        schema_version: u32,
        source_sha256: &'a str,
        source_forward: CreatureSourceForwardV1,
        selected_supermodel_resref: &'a str,
        exact_chain_sha256: &'a str,
        motion_contract_sha256: &'a str,
        structural_profile_sha256: &'a str,
        surface_anatomy_sha256: &'a str,
        fitter_algorithm: &'a str,
        base_rig_sha256: &'a str,
        landmark_overrides: &'a [ReferenceSupermodelLandmarkOverrideV2],
        joint_overrides: &'a [ReferenceSupermodelJointOverrideV1],
        component_bindings: &'a [ReferenceSupermodelComponentBindingV2],
        region_weight_constraints: &'a [ReferenceSupermodelRegionWeightConstraintV2],
        weight_overrides: &'a [ReferenceSupermodelVertexWeightOverrideV1],
    }
    let bytes = serde_json::to_vec(&HashView {
        schema_version: document.schema_version,
        source_sha256: &document.source_sha256,
        source_forward: document.source_forward,
        selected_supermodel_resref: &document.selected_supermodel_resref,
        exact_chain_sha256: &document.exact_chain_sha256,
        motion_contract_sha256: &document.motion_contract_sha256,
        structural_profile_sha256: &document.structural_profile_sha256,
        surface_anatomy_sha256: &document.surface_anatomy_sha256,
        fitter_algorithm: &document.fitter_algorithm,
        base_rig_sha256: &document.base_rig_sha256,
        landmark_overrides: &document.landmark_overrides,
        joint_overrides: &document.joint_overrides,
        component_bindings: &document.component_bindings,
        region_weight_constraints: &document.region_weight_constraints,
        weight_overrides: &document.weight_overrides,
    })
    .map_err(|error| {
        authoring_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORING-HASH-FAILED",
            "$",
            &format!("the V2 authoring document could not be canonicalized: {error}"),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn normalize_document_zeroes(document: &mut ReferenceSupermodelRigAuthoringDocumentV1) {
    for override_row in &mut document.joint_overrides {
        for value in &mut override_row.bind_local_matrix {
            if *value == 0.0 {
                *value = 0.0;
            }
        }
        if let Some(axis) = &mut override_row.joint_axis {
            for value in axis {
                if *value == 0.0 {
                    *value = 0.0;
                }
            }
        }
    }
    for override_row in &mut document.weight_overrides {
        for influence in &mut override_row.influences {
            if influence.value == 0.0 {
                influence.value = 0.0;
            }
        }
    }
}

fn normalize_document_zeroes_v2(document: &mut ReferenceSupermodelRigAuthoringDocumentV2) {
    for row in &mut document.landmark_overrides {
        for value in &mut row.target_world_position {
            if *value == 0.0 {
                *value = 0.0;
            }
        }
    }
    let mut legacy = ReferenceSupermodelRigAuthoringDocumentV1 {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: document.source_sha256.clone(),
        source_forward: document.source_forward,
        selected_supermodel_resref: document.selected_supermodel_resref.clone(),
        exact_chain_sha256: document.exact_chain_sha256.clone(),
        motion_contract_sha256: document.motion_contract_sha256.clone(),
        base_rig_sha256: document.base_rig_sha256.clone(),
        joint_overrides: std::mem::take(&mut document.joint_overrides),
        weight_overrides: std::mem::take(&mut document.weight_overrides),
        content_sha256: String::new(),
    };
    normalize_document_zeroes(&mut legacy);
    document.joint_overrides = legacy.joint_overrides;
    document.weight_overrides = legacy.weight_overrides;
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn authoring_error(
    code: &str,
    path: &str,
    message: &str,
) -> ReferenceSupermodelRigAuthoringErrorV1 {
    ReferenceSupermodelRigAuthoringErrorV1 {
        schema_version: REFERENCE_SUPERMODEL_RIG_AUTHORING_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.to_owned(),
    }
}
