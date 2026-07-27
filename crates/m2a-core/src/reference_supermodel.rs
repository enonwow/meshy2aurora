//! Clean-room static-mesh retargeting for an existing Aurora supermodel.
//!
//! The route deliberately separates compatibility facts (resref plus ordered
//! name/part/parent topology) from exportable rig payload. A reference model
//! may be inspected read-only to build the compatibility contract, but bind
//! transforms, skin surfaces and weights must come from an owned, synthetic or
//! user-provided `CreatureRigProfileV1`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    glb::{GlbLimits, ingest_glb},
    mdl::{BinaryMdlArtifactV1, MdlWriterOptionsV1, write_binary_mdl_with_supermodel},
    profile_a::{
        CreatureRigProfileV1, ProfileAConversionOutcomeV1, ProfileAOptionsV1,
        RigSegmentDeformationV1, convert_profile_a,
    },
};

pub const REFERENCE_SUPERMODEL_RETARGET_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelNodeV1 {
    pub part_number: u32,
    pub name: String,
    pub parent_part_number: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelContractV1 {
    pub schema_version: u32,
    pub contract_id: String,
    pub content_sha256: String,
    pub supermodel_resref: String,
    pub source_model_sha256: String,
    pub inspected_read_only: bool,
    pub no_payload_copied: bool,
    pub nodes: Vec<ReferenceSupermodelNodeV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelRetargetReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub rig_profile_sha256: String,
    pub compatibility_contract_sha256: String,
    pub supermodel_resref: String,
    pub model_resource_resref: String,
    pub uniform_scale: f32,
    pub rig_node_count: usize,
    pub skin_segment_count: usize,
    pub rigid_segment_count: usize,
    pub active_bone_count: usize,
    pub local_animation_count: usize,
    pub model_sha256: String,
}

#[derive(Debug)]
pub struct ReferenceSupermodelRetargetArtifactV1 {
    pub conversion: ProfileAConversionOutcomeV1,
    pub model: BinaryMdlArtifactV1,
    pub report: ReferenceSupermodelRetargetReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelRetargetErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ReferenceSupermodelRetargetErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ReferenceSupermodelRetargetErrorV1 {}

pub fn canonical_reference_supermodel_contract_sha256_v1(
    contract: &ReferenceSupermodelContractV1,
) -> Result<String, ReferenceSupermodelRetargetErrorV1> {
    let mut canonical = contract.clone();
    canonical.content_sha256.clear();
    let bytes = serde_json::to_vec(&canonical).map_err(|error| {
        retarget_error(
            "M7V5-SUPERMODEL-CONTRACT-SERIALIZE-FAILED",
            "contract",
            error.to_string(),
        )
    })?;
    Ok(hex_sha256(&bytes))
}

pub fn retarget_static_mesh_to_reference_supermodel_v1(
    source_glb: &[u8],
    rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelContractV1,
    writer_options: &MdlWriterOptionsV1,
) -> Result<ReferenceSupermodelRetargetArtifactV1, ReferenceSupermodelRetargetErrorV1> {
    validate_contract(contract)?;
    validate_rig_topology(rig, contract)?;

    let source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        retarget_error(
            "M7V5-SOURCE-GLB-INVALID",
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            format!("{}: {}", error.code, error.message),
        )
    })?;
    let mut conversion = convert_profile_a(&source, rig, &ProfileAOptionsV1::default())
        .map_err(|error| retarget_error(error.code, error.path, error.message))?;
    if !conversion.report.conversion_eligible {
        let blocking = conversion
            .report
            .gates
            .iter()
            .filter(|gate| gate.severity == "BLOCKING")
            .map(|gate| gate.code.as_str())
            .collect::<Vec<_>>();
        return Err(retarget_error(
            "M7V5-SOURCE-INELIGIBLE",
            "conversion.report.gates",
            if blocking.is_empty() {
                "Profile A rejected the static source without a blocking gate".to_owned()
            } else {
                format!("Profile A blocking gates: {}", blocking.join(", "))
            },
        ));
    }

    let creature = conversion.creature.as_mut().ok_or_else(|| {
        retarget_error(
            "M7V5-SOURCE-INELIGIBLE",
            "conversion.creature",
            "eligible Profile A conversion did not produce creature IR",
        )
    })?;
    let roots = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    if roots.as_slice() != [0] {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-TOPOLOGY-MISMATCH",
            "conversion.creature.nodes",
            "reference-supermodel output requires the sole rig root at part 0",
        ));
    }
    creature.nodes[0].name = writer_options.model_resource_resref.clone();

    let skin_segment_count = creature
        .segments
        .iter()
        .filter(|segment| segment.deformation == RigSegmentDeformationV1::Skin)
        .count();
    let rigid_segment_count = creature.segments.len() - skin_segment_count;
    if skin_segment_count == 0 {
        return Err(retarget_error(
            "M7V5-SKIN-REQUIRED",
            "conversion.creature.segments",
            "reference-supermodel animation requires at least one weighted Skin segment",
        ));
    }
    let active_bones = creature
        .segments
        .iter()
        .filter(|segment| segment.deformation == RigSegmentDeformationV1::Skin)
        .flat_map(|segment| &segment.weights)
        .flat_map(|row| row.bone_node_ids)
        .flatten()
        .collect::<BTreeSet<_>>();
    if active_bones.is_empty() {
        return Err(retarget_error(
            "M7V5-SKIN-REQUIRED",
            "conversion.creature.segments.weights",
            "reference-supermodel Skin segments must contain active bone weights",
        ));
    }

    let model =
        write_binary_mdl_with_supermodel(creature, &contract.supermodel_resref, writer_options)
            .map_err(|error| retarget_error(error.code, error.path, error.message))?;
    let uniform_scale = conversion.report.transform.scale.ok_or_else(|| {
        retarget_error(
            "M7V5-SOURCE-INELIGIBLE",
            "conversion.report.transform.scale",
            "eligible Profile A conversion did not report its uniform scale",
        )
    })?;
    let report = ReferenceSupermodelRetargetReportV1 {
        schema_version: REFERENCE_SUPERMODEL_RETARGET_SCHEMA_VERSION,
        source_sha256: conversion.source_sha256.clone(),
        rig_profile_sha256: rig.content_sha256.clone(),
        compatibility_contract_sha256: contract.content_sha256.clone(),
        supermodel_resref: contract.supermodel_resref.clone(),
        model_resource_resref: writer_options.model_resource_resref.clone(),
        uniform_scale,
        rig_node_count: creature.nodes.len(),
        skin_segment_count,
        rigid_segment_count,
        active_bone_count: active_bones.len(),
        local_animation_count: 0,
        model_sha256: model.report.payload_sha256.clone(),
    };
    Ok(ReferenceSupermodelRetargetArtifactV1 {
        conversion,
        model,
        report,
    })
}

fn validate_contract(
    contract: &ReferenceSupermodelContractV1,
) -> Result<(), ReferenceSupermodelRetargetErrorV1> {
    if contract.schema_version != REFERENCE_SUPERMODEL_RETARGET_SCHEMA_VERSION {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-CONTRACT-SCHEMA-UNSUPPORTED",
            "contract.schemaVersion",
            "only ReferenceSupermodelContractV1 schema version 1 is supported",
        ));
    }
    if !logical_id(&contract.contract_id) {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-CONTRACT-ID-INVALID",
            "contract.contractId",
            "contract id must be a 1..64 lowercase logical identifier",
        ));
    }
    if !is_resref(&contract.supermodel_resref)
        || contract.supermodel_resref.eq_ignore_ascii_case("NULL")
    {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-RESREF-INVALID",
            "contract.supermodelResref",
            "supermodel must be a non-NULL 1..16 character ASCII resref",
        ));
    }
    if !is_lower_sha256(&contract.source_model_sha256)
        || !contract.inspected_read_only
        || !contract.no_payload_copied
    {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-PROVENANCE-INVALID",
            "contract",
            "contract requires a source hash, read-only inspection and no-payload-copy attestation",
        ));
    }
    let expected = canonical_reference_supermodel_contract_sha256_v1(contract)?;
    if !is_lower_sha256(&contract.content_sha256) || contract.content_sha256 != expected {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-CONTRACT-HASH-MISMATCH",
            "contract.contentSha256",
            "contract hash does not match canonical compatibility content",
        ));
    }
    if contract.nodes.is_empty() {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-TOPOLOGY-INVALID",
            "contract.nodes",
            "compatibility topology must contain at least one node",
        ));
    }
    let mut folded_names = BTreeSet::new();
    for (index, node) in contract.nodes.iter().enumerate() {
        if node.part_number != index as u32
            || !is_node_name(&node.name)
            || !folded_names.insert(node.name.to_ascii_lowercase())
        {
            return Err(retarget_error(
                "M7V5-SUPERMODEL-TOPOLOGY-INVALID",
                format!("contract.nodes[{index}]"),
                "parts must be contiguous and node names valid and case-fold unique",
            ));
        }
        if index == 0 {
            if node.parent_part_number.is_some() {
                return Err(retarget_error(
                    "M7V5-SUPERMODEL-TOPOLOGY-INVALID",
                    "contract.nodes[0].parentPartNumber",
                    "part 0 must be the sole root",
                ));
            }
        } else if node
            .parent_part_number
            .is_none_or(|parent| parent >= node.part_number)
        {
            return Err(retarget_error(
                "M7V5-SUPERMODEL-TOPOLOGY-INVALID",
                format!("contract.nodes[{index}].parentPartNumber"),
                "every non-root parent must reference an earlier part",
            ));
        }
    }
    Ok(())
}

fn validate_rig_topology(
    rig: &CreatureRigProfileV1,
    contract: &ReferenceSupermodelContractV1,
) -> Result<(), ReferenceSupermodelRetargetErrorV1> {
    if rig.nodes.len() != contract.nodes.len() {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-TOPOLOGY-MISMATCH",
            "rig.nodes",
            "owned target rig node count differs from the compatibility contract",
        ));
    }
    let part_by_id = rig
        .nodes
        .iter()
        .enumerate()
        .map(|(part, node)| (node.id, part as u32))
        .collect::<BTreeMap<_, _>>();
    if part_by_id.len() != rig.nodes.len() {
        return Err(retarget_error(
            "M7V5-SUPERMODEL-TOPOLOGY-MISMATCH",
            "rig.nodes",
            "owned target rig node ids must be unique",
        ));
    }
    for (part, (rig_node, expected)) in rig.nodes.iter().zip(&contract.nodes).enumerate() {
        let actual_parent = rig_node
            .parent_id
            .and_then(|parent_id| part_by_id.get(&parent_id).copied());
        let name_matches = part == 0 || rig_node.name == expected.name;
        if expected.part_number != part as u32
            || actual_parent != expected.parent_part_number
            || !name_matches
        {
            return Err(retarget_error(
                "M7V5-SUPERMODEL-TOPOLOGY-MISMATCH",
                format!("rig.nodes[{part}]"),
                "owned target rig name/part/parent topology differs from the compatibility contract",
            ));
        }
    }
    Ok(())
}

fn retarget_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReferenceSupermodelRetargetErrorV1 {
    ReferenceSupermodelRetargetErrorV1 {
        schema_version: REFERENCE_SUPERMODEL_RETARGET_SCHEMA_VERSION,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

fn logical_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

fn is_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_node_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 31
        && value.is_ascii()
        && !value.bytes().any(|byte| byte == 0)
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
