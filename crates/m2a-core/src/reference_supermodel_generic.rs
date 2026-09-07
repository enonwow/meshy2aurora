//! Structure-driven reference-supermodel analysis and rig derivation.
//!
//! This module has no family/resref registry. Exact input bytes determine the
//! format, hierarchy, neutral controllers and animation inventory. Generated
//! weights are derived continuously from the inspected carrier structure; no
//! retail geometry, weights or animation payload is copied to output.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    glb::GlbIngestResult,
    mdl::{
        AnimationEventReport, AnimationReport, ArrayReport, ByteRangeReport, ControllerReport,
        FileHeaderReport, InspectionReport, ModelReport, NodeReport, NodeTreeReport, Vec3,
        inspect_binary_mdl,
    },
    profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        CreatureSourceForwardV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_profile_sha256,
    },
    reference_supermodel_bind_pose::{
        ReferenceSupermodelBindPoseReportV1, validate_reference_supermodel_bind_pose_v1,
    },
    reference_supermodel_motion::{
        ReferenceSupermodelCarrierClassV3, ReferenceSupermodelExactContractOptionsV3,
        ReferenceSupermodelMotionContractV2, ReferenceSupermodelSemanticNodeV3,
        build_exact_reference_supermodel_motion_contract_v3,
        default_reference_supermodel_motion_tolerances_v2,
    },
    reference_supermodel_skinning::{
        ReferenceSupermodelSkinningOptionsV1, ReferenceSupermodelSkinningReportV1,
        derive_local_reference_skinning_v1, validate_authored_reference_skinning_v2,
    },
    reference_supermodel_structure::{
        ReferenceSupermodelJointFitReportV1, ReferenceSupermodelStructuralProfileV1,
        build_immutable_reference_supermodel_joint_report_v1,
        build_reference_supermodel_structural_profile_v1,
        fit_reference_supermodel_joints_with_anatomy_v1,
        reference_supermodel_rig_world_positions_v1,
        revalidate_authored_reference_supermodel_joints_v2,
    },
    reference_supermodel_surface_anatomy::{
        TargetSurfaceAnatomyV1, analyze_target_surface_anatomy_v1,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReferenceSupermodelFormatV2 {
    Auto,
    Binary,
    Ascii,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelChainResourceV2 {
    pub resref: String,
    pub supermodel_resref: String,
    pub format: ReferenceSupermodelFormatV2,
    pub sha256: String,
    pub byte_length: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceSupermodelChainPayloadV2 {
    pub resource: ReferenceSupermodelChainResourceV2,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelChainResourceAnalysisV2 {
    pub resref: String,
    pub supermodel_resref: String,
    pub format: ReferenceSupermodelFormatV2,
    pub sha256: String,
    pub byte_length: usize,
    pub node_count: usize,
    pub controller_count: usize,
    pub local_animation_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelStructuralErrorV2 {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelChainAnalysisReportV2 {
    pub schema_version: u32,
    pub status: String,
    pub selected_supermodel_resref: String,
    pub selected_format: ReferenceSupermodelFormatV2,
    pub selected_sha256: String,
    pub motion_carrier_resref: String,
    pub exact_chain: Vec<ReferenceSupermodelChainResourceAnalysisV2>,
    pub inherited_animation_names: Vec<String>,
    pub carrier_node_count: usize,
    pub carrier_controller_count: usize,
    pub structural_errors: Vec<ReferenceSupermodelStructuralErrorV2>,
    pub retail_payload_copied: bool,
}

#[derive(Clone, Debug)]
pub struct ReferenceSupermodelChainAnalysisArtifactV2 {
    pub report: ReferenceSupermodelChainAnalysisReportV2,
    pub combined_reference: InspectionReport,
    pub motion_contract: ReferenceSupermodelMotionContractV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelGenericErrorV2 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ReferenceSupermodelGenericErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ReferenceSupermodelGenericErrorV2 {}

fn generic_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReferenceSupermodelGenericErrorV2 {
    ReferenceSupermodelGenericErrorV2 {
        schema_version: 2,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericReferenceRigAnalysisV2 {
    pub schema_version: u32,
    pub algorithm: String,
    pub carrier_node_count: usize,
    pub full_carrier_coverage: bool,
    pub required_joint_coverage: bool,
    pub allowed_bone_count: usize,
    pub weighted_bone_count: usize,
    pub active_weighted_bone_count: usize,
    pub initial_unweighted_required_joint_names: Vec<String>,
    pub unweighted_required_joint_names: Vec<String>,
    pub passive_unweighted_joint_names: Vec<String>,
    pub skin_influence_coverage: bool,
    pub joint_influences: Vec<GenericReferenceJointInfluenceV3>,
    pub surface_vertex_count: usize,
    pub surface_triangle_count: usize,
    pub surface_component_count: usize,
    pub stabilized_small_component_count: usize,
    pub stabilized_small_component_vertex_count: usize,
    pub fitted_ground_contact_chain_count: usize,
    pub duplicate_position_group_count: usize,
    pub no_reference_payload_copied: bool,
    pub structural_profile: ReferenceSupermodelStructuralProfileV1,
    pub surface_anatomy: TargetSurfaceAnatomyV1,
    pub joint_fit: ReferenceSupermodelJointFitReportV1,
    pub skinning: ReferenceSupermodelSkinningReportV1,
    pub bind_pose: ReferenceSupermodelBindPoseReportV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericReferenceJointInfluenceV3 {
    pub joint_name: String,
    pub carrier_class: ReferenceSupermodelCarrierClassV3,
    pub positive_influence_vertex_count: usize,
    pub visible_cluster_vertex_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericReferenceSkinInfluenceAuditV3 {
    pub schema_version: u32,
    pub allowed_bone_count: usize,
    /// Compatibility alias retained for V2 consumers. It now has the correct
    /// meaning: distinct bone references present in positive normalized rows.
    pub weighted_bone_count: usize,
    pub active_weighted_bone_count: usize,
    pub unweighted_required_joint_names: Vec<String>,
    pub passive_unweighted_joint_names: Vec<String>,
    pub skin_influence_coverage: bool,
    pub joint_influences: Vec<GenericReferenceJointInfluenceV3>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenericReferenceRigArtifactV2 {
    pub rig: CreatureRigProfileV1,
    pub report: GenericReferenceRigAnalysisV2,
}

pub fn inspect_reference_supermodel_mdl_v2(
    bytes: &[u8],
    format: ReferenceSupermodelFormatV2,
) -> Result<InspectionReport, ReferenceSupermodelGenericErrorV2> {
    let detected = match format {
        ReferenceSupermodelFormatV2::Auto => {
            if bytes.starts_with(&[0, 0, 0, 0]) {
                ReferenceSupermodelFormatV2::Binary
            } else {
                ReferenceSupermodelFormatV2::Ascii
            }
        }
        explicit => explicit,
    };
    match detected {
        ReferenceSupermodelFormatV2::Binary => inspect_binary_mdl(bytes).map_err(|source| {
            generic_error(
                source.code,
                format!("referenceMdl@{}", source.offset),
                source.context,
            )
        }),
        ReferenceSupermodelFormatV2::Ascii => inspect_ascii_reference_supermodel_v2(bytes),
        ReferenceSupermodelFormatV2::Auto => unreachable!("auto format resolved above"),
    }
}

/// Validates and analyzes the entire exact selected-to-root chain. Selection
/// identity comes from the caller and exact bytes, never from a family table.
pub fn analyze_reference_supermodel_chain_v2(
    selected_supermodel_resref: &str,
    chain: &[ReferenceSupermodelChainPayloadV2],
) -> Result<ReferenceSupermodelChainAnalysisArtifactV2, ReferenceSupermodelGenericErrorV2> {
    if !is_resref_v2(selected_supermodel_resref) || chain.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SELECTION-INVALID",
            "selection",
            "selection requires a lowercase resref and a non-empty exact chain",
        ));
    }
    if !chain[0]
        .resource
        .resref
        .eq_ignore_ascii_case(selected_supermodel_resref)
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SELECTION-MISMATCH",
            "chain[0].resref",
            "the exact chain must begin with the selected supermodel",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut parsed = Vec::with_capacity(chain.len());
    let mut exact_chain = Vec::with_capacity(chain.len());
    for (index, item) in chain.iter().enumerate() {
        let resource = &item.resource;
        if !is_resref_v2(&resource.resref)
            || !seen.insert(resource.resref.to_ascii_lowercase())
            || !is_sha256_v2(&resource.sha256)
            || resource.byte_length == 0
            || resource.byte_length != item.payload.len()
            || resource.format == ReferenceSupermodelFormatV2::Auto
        {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-CHAIN-RESOURCE-INVALID",
                format!("chain[{index}]"),
                "each chain item requires unique identity, explicit format, byte length and SHA-256",
            ));
        }
        let expected_parent = chain
            .get(index + 1)
            .map(|parent| parent.resource.resref.as_str())
            .unwrap_or("NULL");
        if !resource
            .supermodel_resref
            .eq_ignore_ascii_case(expected_parent)
        {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-CHAIN-MISMATCH",
                format!("chain[{index}].supermodelResref"),
                format!(
                    "declared parent {:?} differs from exact next resource {:?}",
                    resource.supermodel_resref, expected_parent
                ),
            ));
        }
        let actual_sha = format!("{:x}", Sha256::digest(&item.payload));
        if actual_sha != resource.sha256 {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-SHA256-MISMATCH",
                format!("chain[{index}].payload"),
                "exact MDL bytes differ from the caller-bound SHA-256",
            ));
        }
        let inspection = inspect_reference_supermodel_mdl_v2(&item.payload, resource.format)?;
        if !inspection.model.name.eq_ignore_ascii_case(&resource.resref)
            || !inspection
                .model
                .supermodel_name
                .eq_ignore_ascii_case(&resource.supermodel_resref)
        {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-MDL-IDENTITY-MISMATCH",
                format!("chain[{index}].model"),
                "parsed newmodel/setsupermodel identity differs from the exact descriptor",
            ));
        }
        let controller_count = count_report_controllers_v2(&inspection);
        exact_chain.push(ReferenceSupermodelChainResourceAnalysisV2 {
            resref: resource.resref.clone(),
            supermodel_resref: resource.supermodel_resref.clone(),
            format: resource.format,
            sha256: resource.sha256.clone(),
            byte_length: resource.byte_length,
            node_count: inspection.node_tree.node_count,
            controller_count,
            local_animation_count: inspection.animations.len(),
        });
        parsed.push(inspection);
    }
    let provider_index = parsed
        .iter()
        .position(|report| !report.animations.is_empty() && report.node_tree.node_count > 0)
        .ok_or_else(|| {
            generic_error(
                "M2A-REFERENCE-SUPERMODEL-MOTION-PROVIDER-MISSING",
                "chain",
                "no exact chain resource contains both a carrier hierarchy and local animations",
            )
        })?;
    if parsed
        .iter()
        .any(|report| report.node_tree.node_count > 0 && report.model.classification != 4)
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-STRUCTURE-NOT-CREATURE",
            "chain.classification",
            "Creature application requires classification 4 throughout the resolved structural chain",
        ));
    }
    let mut inherited_animation_names = Vec::new();
    let mut inherited_seen = BTreeSet::new();
    for report in &parsed {
        for animation in &report.animations {
            if inherited_seen.insert(animation.name.to_ascii_lowercase()) {
                inherited_animation_names.push(animation.name.clone());
            }
        }
    }
    let combined_reference = merge_reference_supermodel_chain_v3(
        selected_supermodel_resref,
        &chain[0].resource.supermodel_resref,
        &parsed,
    )?;
    let required_clips = inherited_animation_names.clone();
    if required_clips.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-INHERITED-CLIPS-EMPTY",
            "chain.animations",
            "the complete selected-to-root chain exposes no inherited clip corpus",
        ));
    }
    let mut tolerances = default_reference_supermodel_motion_tolerances_v2();
    // The generic route must never re-introduce the density-normalized seam
    // allowance which admitted the owner-visible split-surface failure.
    tolerances.fail_on_any_seam_violation = true;
    let semantic_nodes = discover_structural_semantic_nodes_v2(&combined_reference);
    let motion_contract = build_exact_reference_supermodel_motion_contract_v3(
        &combined_reference,
        &ReferenceSupermodelExactContractOptionsV3 {
            contract_id: "exact-selected-supermodel-structural-v2".to_owned(),
            supermodel_resref: selected_supermodel_resref.to_owned(),
            source_model_sha256: chain[0].resource.sha256.clone(),
            required_clips,
            required_events: Vec::new(),
            semantic_nodes,
            tolerances,
        },
    )
    .map_err(|source| generic_error(source.code, source.path, source.message))?;
    let report = ReferenceSupermodelChainAnalysisReportV2 {
        schema_version: 2,
        status: "REFERENCE_SUPERMODEL_STRUCTURALLY_READY".to_owned(),
        selected_supermodel_resref: selected_supermodel_resref.to_owned(),
        selected_format: chain[0].resource.format,
        selected_sha256: chain[0].resource.sha256.clone(),
        motion_carrier_resref: chain[provider_index].resource.resref.clone(),
        exact_chain,
        inherited_animation_names,
        carrier_node_count: motion_contract.nodes.len(),
        carrier_controller_count: count_report_controllers_v2(&combined_reference),
        structural_errors: Vec::new(),
        retail_payload_copied: false,
    };
    Ok(ReferenceSupermodelChainAnalysisArtifactV2 {
        report,
        combined_reference,
        motion_contract,
    })
}

#[derive(Clone, Debug)]
struct MergedReferenceNodeV3 {
    source: NodeReport,
    parent_key: Option<String>,
    order: usize,
}

/// Creates one structural inspection used only by the clean-room contract
/// builder. Retail payloads are not serialized or copied to product output.
/// Model roots from every chain level are canonical aliases of the selected
/// child root; all other nodes are merged by their full parent path.
fn merge_reference_supermodel_chain_v3(
    selected_resref: &str,
    selected_parent_resref: &str,
    reports: &[InspectionReport],
) -> Result<InspectionReport, ReferenceSupermodelGenericErrorV2> {
    fn collect(
        nodes: &[NodeReport],
        parent_key: Option<&str>,
        selected_resref: &str,
        merged: &mut BTreeMap<String, MergedReferenceNodeV3>,
        order: &mut usize,
        report_index: usize,
    ) -> Result<(), ReferenceSupermodelGenericErrorV2> {
        for node in nodes {
            let is_root = parent_key.is_none();
            let name = if is_root {
                selected_resref.to_owned()
            } else {
                node.name.clone()
            };
            let key = if let Some(parent) = parent_key {
                format!("{parent}/{}", name.to_ascii_lowercase())
            } else {
                "<selected-root>".to_owned()
            };
            if let Some(existing) = merged.get(&key) {
                if !existing.source.name.eq_ignore_ascii_case(&name)
                    || existing.parent_key.as_deref() != parent_key
                {
                    return Err(generic_error(
                        "M2A-REFERENCE-SUPERMODEL-CHAIN-HIERARCHY-INCOMPATIBLE",
                        format!("chain[{report_index}].nodeTree.{key}"),
                        "chain resources define incompatible carrier identity or parentage",
                    ));
                }
            } else {
                let mut source = node.clone();
                source.name = name;
                source.children.clear();
                merged.insert(
                    key.clone(),
                    MergedReferenceNodeV3 {
                        source,
                        parent_key: parent_key.map(str::to_owned),
                        order: *order,
                    },
                );
                *order += 1;
            }
            collect(
                &node.children,
                Some(&key),
                selected_resref,
                merged,
                order,
                report_index,
            )?;
        }
        Ok(())
    }

    fn build(
        key: &str,
        parent_offset: Option<u32>,
        merged: &BTreeMap<String, MergedReferenceNodeV3>,
        next: &mut u32,
    ) -> Result<NodeReport, ReferenceSupermodelGenericErrorV2> {
        let entry = merged.get(key).ok_or_else(|| {
            generic_error(
                "M2A-REFERENCE-SUPERMODEL-CHAIN-HIERARCHY-INCOMPATIBLE",
                "chain.nodeTree",
                "merged carrier node is absent",
            )
        })?;
        let number = *next;
        *next = next.saturating_add(1);
        let offset = number.saturating_add(1);
        let mut child_keys = merged
            .iter()
            .filter(|(_, candidate)| candidate.parent_key.as_deref() == Some(key))
            .map(|(key, candidate)| (key.clone(), candidate.order))
            .collect::<Vec<_>>();
        child_keys.sort_by_key(|(_, order)| *order);
        let children = child_keys
            .iter()
            .map(|(child, _)| build(child, Some(offset), merged, next))
            .collect::<Result<Vec<_>, _>>()?;
        let mut node = entry.source.clone();
        node.offset = offset;
        node.number = number;
        node.parent_offset = parent_offset;
        node.children_header.used = children.len();
        node.children_header.allocated = children.len();
        node.children = children;
        Ok(node)
    }

    let selected = reports.first().ok_or_else(|| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-CHAIN-EMPTY",
            "chain",
            "cannot merge an empty reference-supermodel chain",
        )
    })?;
    let mut merged = BTreeMap::new();
    let mut order = 0;
    for (report_index, report) in reports.iter().enumerate() {
        collect(
            &report.node_tree.roots,
            None,
            selected_resref,
            &mut merged,
            &mut order,
            report_index,
        )?;
    }
    if !merged.contains_key("<selected-root>") {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-CHAIN-ROOT-MISSING",
            "chain.nodeTree",
            "resolved chain contains no structural model root",
        ));
    }
    let mut next = 0;
    let root = build("<selected-root>", None, &merged, &mut next)?;
    let mut animations = Vec::new();
    let mut animation_names = BTreeSet::new();
    for report in reports {
        for animation in &report.animations {
            if animation_names.insert(animation.name.to_ascii_lowercase()) {
                animations.push(animation.clone());
            }
        }
    }
    let mut combined = selected.clone();
    combined.model.name = selected_resref.to_owned();
    combined.model.supermodel_name = selected_parent_resref.to_owned();
    combined.node_tree = NodeTreeReport {
        declared_node_count: next as usize,
        node_count: next as usize,
        max_depth: node_depth_v2(&root),
        roots: vec![root],
    };
    combined.animations = animations;
    Ok(combined)
}

fn is_resref_v2(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn is_sha256_v2(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn count_report_controllers_v2(report: &InspectionReport) -> usize {
    fn count_nodes(nodes: &[NodeReport]) -> usize {
        nodes
            .iter()
            .map(|node| node.controllers.len() + count_nodes(&node.children))
            .sum()
    }
    count_nodes(&report.node_tree.roots)
        + report
            .animations
            .iter()
            .map(|animation| count_nodes(&animation.node_tree.roots))
            .sum::<usize>()
}

fn discover_structural_semantic_nodes_v2(
    report: &InspectionReport,
) -> Vec<ReferenceSupermodelSemanticNodeV3> {
    fn collect<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
        for node in nodes {
            output.push(node);
            collect(&node.children, output);
        }
    }
    let mut nodes = Vec::new();
    collect(&report.node_tree.roots, &mut nodes);
    let mut counts = BTreeMap::<String, usize>::new();
    for node in &nodes {
        *counts.entry(node.name.to_ascii_lowercase()).or_default() += 1;
    }
    nodes
        .into_iter()
        .filter(|node| {
            let name = node.name.to_ascii_lowercase();
            let has_tail_token = name
                .split('_')
                .any(|token| token == "tail" || token == "tailend")
                || name.contains("tail_end");
            has_tail_token && counts.get(&name) == Some(&1)
        })
        .enumerate()
        .map(|(index, node)| ReferenceSupermodelSemanticNodeV3 {
            node_name: node.name.clone(),
            anchor_role: Some(format!("tail_{index}")),
            joint_axis: None,
        })
        .collect()
}

#[derive(Clone, Debug)]
struct AsciiNodeV2 {
    name: String,
    parent_name: Option<String>,
    controllers: Vec<ControllerReport>,
}

#[derive(Clone, Debug)]
struct AsciiAnimationV2 {
    name: String,
    length: f32,
    transition: f32,
    animation_root: String,
    nodes: Vec<AsciiNodeV2>,
}

fn inspect_ascii_reference_supermodel_v2(
    bytes: &[u8],
) -> Result<InspectionReport, ReferenceSupermodelGenericErrorV2> {
    let text = std::str::from_utf8(bytes).map_err(|source| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-UTF8",
            "referenceMdl",
            source.to_string(),
        )
    })?;
    if text.as_bytes().iter().any(|byte| !byte.is_ascii()) {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-NONASCII",
            "referenceMdl",
            "ASCII MDL contains non-ASCII bytes",
        ));
    }
    let lines = text
        .lines()
        .map(|raw| raw.split('#').next().unwrap_or("").trim().to_owned())
        .collect::<Vec<_>>();
    let mut model_name = None;
    let mut supermodel_name = "NULL".to_owned();
    let mut classification = None;
    let mut animation_scale = 1.0f32;
    let mut base_nodes = Vec::<AsciiNodeV2>::new();
    let mut animations = Vec::<AsciiAnimationV2>::new();
    let mut current_animation: Option<AsciiAnimationV2> = None;
    let mut current_node: Option<AsciiNodeV2> = None;
    let mut line_index = 0usize;
    while line_index < lines.len() {
        let line = &lines[line_index];
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        line_index += 1;
        let Some(keyword) = fields.first().copied() else {
            continue;
        };
        if keyword.eq_ignore_ascii_case("newmodel") && fields.len() >= 2 {
            model_name.get_or_insert_with(|| fields[1].to_owned());
        } else if keyword.eq_ignore_ascii_case("setsupermodel") && fields.len() >= 3 {
            supermodel_name = fields[2].to_owned();
        } else if keyword.eq_ignore_ascii_case("classification") && fields.len() >= 2 {
            classification = Some(parse_ascii_classification_v2(fields[1])?);
        } else if keyword.eq_ignore_ascii_case("setanimationscale") && fields.len() >= 2 {
            animation_scale = parse_f32_v2(fields[1], "setanimationscale")?;
        } else if keyword.eq_ignore_ascii_case("newanim") && fields.len() >= 2 {
            if current_animation.is_some() || current_node.is_some() {
                return Err(generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-NESTING",
                    format!("lines[{line_index}]"),
                    "newanim cannot begin inside another animation or node",
                ));
            }
            current_animation = Some(AsciiAnimationV2 {
                name: fields[1].to_owned(),
                length: 0.0,
                transition: 0.25,
                animation_root: model_name.clone().unwrap_or_default(),
                nodes: Vec::new(),
            });
        } else if keyword.eq_ignore_ascii_case("doneanim") {
            let animation = current_animation.take().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-NESTING",
                    format!("lines[{line_index}]"),
                    "doneanim has no matching newanim",
                )
            })?;
            animations.push(animation);
        } else if keyword.eq_ignore_ascii_case("length") && fields.len() >= 2 {
            if let Some(animation) = &mut current_animation {
                animation.length = parse_f32_v2(fields[1], "animation.length")?;
            }
        } else if keyword.eq_ignore_ascii_case("transtime") && fields.len() >= 2 {
            if let Some(animation) = &mut current_animation {
                animation.transition = parse_f32_v2(fields[1], "animation.transtime")?;
            }
        } else if keyword.eq_ignore_ascii_case("animroot") && fields.len() >= 2 {
            if let Some(animation) = &mut current_animation {
                animation.animation_root = fields[1].to_owned();
            }
        } else if keyword.eq_ignore_ascii_case("node") && fields.len() >= 3 {
            if current_node.is_some() {
                return Err(generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-NESTING",
                    format!("lines[{line_index}]"),
                    "nested node declarations are invalid",
                ));
            }
            current_node = Some(AsciiNodeV2 {
                name: fields[2].to_owned(),
                parent_name: None,
                controllers: Vec::new(),
            });
        } else if keyword.eq_ignore_ascii_case("endnode") {
            let node = current_node.take().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-NESTING",
                    format!("lines[{line_index}]"),
                    "endnode has no matching node",
                )
            })?;
            if let Some(animation) = &mut current_animation {
                animation.nodes.push(node);
            } else {
                base_nodes.push(node);
            }
        } else if keyword.eq_ignore_ascii_case("parent") && fields.len() >= 2 {
            let node = current_node.as_mut().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-PARENT-OUTSIDE-NODE",
                    format!("lines[{line_index}]"),
                    "parent declaration requires a node block",
                )
            })?;
            node.parent_name =
                (!fields[1].eq_ignore_ascii_case("NULL")).then(|| fields[1].to_owned());
        } else if ["position", "orientation", "scale"]
            .iter()
            .any(|candidate| keyword.eq_ignore_ascii_case(candidate))
        {
            let node = current_node.as_mut().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-CONTROLLER-OUTSIDE-NODE",
                    format!("lines[{line_index}]"),
                    "controller declaration requires a node block",
                )
            })?;
            node.controllers
                .push(parse_ascii_static_controller_v2(keyword, &fields[1..])?);
        } else if keyword.to_ascii_lowercase().ends_with("key")
            && ["positionkey", "orientationkey", "scalekey"]
                .iter()
                .any(|candidate| keyword.eq_ignore_ascii_case(candidate))
        {
            let row_count = fields
                .get(1)
                .ok_or_else(|| {
                    generic_error(
                        "M2A-REFERENCE-SUPERMODEL-ASCII-KEY-COUNT",
                        format!("lines[{line_index}]"),
                        "key controller requires a row count",
                    )
                })?
                .parse::<usize>()
                .map_err(|source| {
                    generic_error(
                        "M2A-REFERENCE-SUPERMODEL-ASCII-KEY-COUNT",
                        format!("lines[{line_index}]"),
                        source.to_string(),
                    )
                })?;
            let node = current_node.as_mut().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-CONTROLLER-OUTSIDE-NODE",
                    format!("lines[{line_index}]"),
                    "key controller requires a node block",
                )
            })?;
            let mut rows = Vec::with_capacity(row_count);
            for _ in 0..row_count {
                let row_line = lines.get(line_index).ok_or_else(|| {
                    generic_error(
                        "M2A-REFERENCE-SUPERMODEL-ASCII-KEY-TRUNCATED",
                        format!("lines[{line_index}]"),
                        "key controller rows are truncated",
                    )
                })?;
                line_index += 1;
                rows.push(row_line.split_ascii_whitespace().collect::<Vec<_>>());
            }
            node.controllers
                .push(parse_ascii_key_controller_v2(keyword, &rows)?);
        }
    }
    if current_node.is_some() || current_animation.is_some() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-TRUNCATED",
            "referenceMdl",
            "ASCII MDL ended inside a node or animation",
        ));
    }
    let model_name = model_name.ok_or_else(|| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-MODEL-NAME",
            "referenceMdl",
            "ASCII MDL has no newmodel declaration",
        )
    })?;
    let classification = classification.ok_or_else(|| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-CLASSIFICATION",
            "referenceMdl.classification",
            "ASCII MDL requires an explicit classification",
        )
    })?;
    if !animation_scale.is_finite() || animation_scale <= 0.0 {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-ANIMATION-SCALE",
            "referenceMdl.animationScale",
            "animation scale must be finite and positive",
        ));
    }
    let base_tree = build_ascii_node_tree_v2(&base_nodes, None)?;
    let animation_reports = animations
        .into_iter()
        .enumerate()
        .map(|(index, animation)| {
            let tree = build_ascii_node_tree_v2(&animation.nodes, Some(&base_nodes))?;
            Ok(AnimationReport {
                offset: u32::try_from(index + 1).unwrap_or(u32::MAX),
                name: animation.name,
                geometry_array_50: empty_array_v2(),
                geometry_array_5c: empty_array_v2(),
                runtime_68: 0,
                animation_type: 5,
                animation_type_padding: [0; 3],
                length: animation.length,
                transition: animation.transition,
                animation_root: animation.animation_root,
                events_header: empty_array_v2(),
                events: Vec::<AnimationEventReport>::new(),
                node_tree: tree,
            })
        })
        .collect::<Result<Vec<_>, ReferenceSupermodelGenericErrorV2>>()?;
    Ok(InspectionReport {
        schema_version: 2,
        format: "NWN_ASCII_MDL".to_owned(),
        byte_length: bytes.len(),
        file_header: FileHeaderReport {
            binary_mdl_id: 0,
            mdx_start: 0,
            mdx_size: 0,
            mdx_range_in_bounds: true,
            core_range: ByteRangeReport {
                start: 0,
                length: bytes.len(),
                end: bytes.len(),
            },
            raw_range: ByteRangeReport {
                start: bytes.len(),
                length: 0,
                end: bytes.len(),
            },
        },
        model: ModelReport {
            name: model_name,
            root_node_offset: base_tree.roots.first().map_or(0, |node| node.offset),
            geometry_type: 2,
            classification,
            fog: 0,
            child_model_count: 0,
            bounds_min: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            bounds_max: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 0.0,
            animation_scale,
            supermodel_name,
            animation_pointers_header: ArrayReport {
                pointer: 0,
                used: animation_reports.len(),
                allocated: animation_reports.len(),
            },
        },
        node_tree: base_tree,
        animations: animation_reports,
        unsupported: Vec::new(),
        diagnostics: Vec::new(),
    })
}

fn parse_ascii_classification_v2(value: &str) -> Result<u8, ReferenceSupermodelGenericErrorV2> {
    match value.to_ascii_uppercase().as_str() {
        "EFFECT" => Ok(1),
        "TILE" => Ok(2),
        "CHARACTER" | "CREATURE" => Ok(4),
        "DOOR" => Ok(8),
        _ => value.parse::<u8>().map_err(|_| {
            generic_error(
                "M2A-REFERENCE-SUPERMODEL-ASCII-CLASSIFICATION",
                "referenceMdl.classification",
                format!("unsupported ASCII classification {value:?}"),
            )
        }),
    }
}

fn parse_f32_v2(value: &str, path: &str) -> Result<f32, ReferenceSupermodelGenericErrorV2> {
    let parsed = value.parse::<f32>().map_err(|source| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-NUMBER",
            path,
            source.to_string(),
        )
    })?;
    parsed.is_finite().then_some(parsed).ok_or_else(|| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-NUMBER",
            path,
            "number must be finite",
        )
    })
}

fn parse_ascii_static_controller_v2(
    keyword: &str,
    values: &[&str],
) -> Result<ControllerReport, ReferenceSupermodelGenericErrorV2> {
    let name = keyword.to_ascii_lowercase();
    let expected = match name.as_str() {
        "position" => 3,
        "orientation" => 4,
        "scale" => 1,
        _ => 0,
    };
    if values.len() < expected {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-CONTROLLER",
            keyword,
            format!("{keyword} requires {expected} values"),
        ));
    }
    let mut parsed = values[..expected]
        .iter()
        .map(|value| parse_f32_v2(value, keyword))
        .collect::<Result<Vec<_>, _>>()?;
    if name == "orientation" {
        parsed = axis_angle_to_quaternion_v2(&parsed)?;
    }
    Ok(controller_report_v2(&name, vec![0.0], vec![parsed]))
}

fn parse_ascii_key_controller_v2(
    keyword: &str,
    rows: &[Vec<&str>],
) -> Result<ControllerReport, ReferenceSupermodelGenericErrorV2> {
    let lower_keyword = keyword.to_ascii_lowercase();
    let name = lower_keyword
        .strip_suffix("key")
        .expect("caller accepts only *key controllers")
        .to_owned();
    let expected = match name.as_str() {
        "position" => 3,
        "orientation" => 4,
        "scale" => 1,
        _ => 0,
    };
    let mut times = Vec::with_capacity(rows.len());
    let mut values = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        if row.len() < expected + 1 {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-ASCII-KEY",
                format!("{keyword}[{index}]"),
                format!("key row requires time plus {expected} values"),
            ));
        }
        times.push(parse_f32_v2(row[0], keyword)?);
        let mut parsed = row[1..=expected]
            .iter()
            .map(|value| parse_f32_v2(value, keyword))
            .collect::<Result<Vec<_>, _>>()?;
        if name == "orientation" {
            parsed = axis_angle_to_quaternion_v2(&parsed)?;
        }
        values.push(parsed);
    }
    Ok(controller_report_v2(&name, times, values))
}

fn axis_angle_to_quaternion_v2(
    values: &[f32],
) -> Result<Vec<f32>, ReferenceSupermodelGenericErrorV2> {
    let axis_length =
        (values[0] * values[0] + values[1] * values[1] + values[2] * values[2]).sqrt();
    if axis_length <= f32::EPSILON {
        if values[3].abs() <= f32::EPSILON {
            return Ok(vec![0.0, 0.0, 0.0, 1.0]);
        }
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-ORIENTATION",
            "orientation",
            "non-zero angle requires a non-zero axis",
        ));
    }
    let sine = (values[3] * 0.5).sin() / axis_length;
    Ok(vec![
        values[0] * sine,
        values[1] * sine,
        values[2] * sine,
        (values[3] * 0.5).cos(),
    ])
}

fn controller_report_v2(name: &str, times: Vec<f32>, values: Vec<Vec<f32>>) -> ControllerReport {
    let (controller_type, column_count) = match name {
        "position" => (8, 3),
        "orientation" => (20, 4),
        "scale" => (36, 1),
        _ => (0, 0),
    };
    ControllerReport {
        key_offset: 0,
        controller_type,
        controller_name: Some(name.to_owned()),
        packed_byte: 0,
        interpolation_flags: 0,
        decoded: true,
        padding_byte: 0,
        row_count: values.len(),
        time_index: 0,
        data_index: 0,
        column_count,
        times,
        values,
    }
}

fn build_ascii_node_tree_v2(
    nodes: &[AsciiNodeV2],
    base_nodes: Option<&[AsciiNodeV2]>,
) -> Result<NodeTreeReport, ReferenceSupermodelGenericErrorV2> {
    let mut selected = nodes
        .iter()
        .map(|node| (node.name.to_ascii_lowercase(), node.clone()))
        .collect::<BTreeMap<_, _>>();
    if selected.len() != nodes.len() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-NODE-DUPLICATE",
            "referenceMdl.nodes",
            "node names must be unique after ASCII case-fold",
        ));
    }
    if let Some(base) = base_nodes {
        let base_by_name = base
            .iter()
            .map(|node| (node.name.to_ascii_lowercase(), node))
            .collect::<BTreeMap<_, _>>();
        let mut pending = selected
            .values()
            .filter_map(|node| node.parent_name.clone())
            .collect::<Vec<_>>();
        while let Some(parent_name) = pending.pop() {
            let key = parent_name.to_ascii_lowercase();
            if selected.contains_key(&key) {
                continue;
            }
            let parent = base_by_name.get(&key).ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-ASCII-PARENT-MISSING",
                    "referenceMdl.nodes.parent",
                    format!("parent node {parent_name:?} is absent"),
                )
            })?;
            selected.insert(key, (*parent).clone());
            if let Some(grandparent) = &parent.parent_name {
                pending.push(grandparent.clone());
            }
        }
    }
    let ordered_names = if let Some(base) = base_nodes {
        base.iter()
            .filter(|node| selected.contains_key(&node.name.to_ascii_lowercase()))
            .map(|node| node.name.to_ascii_lowercase())
            .collect::<Vec<_>>()
    } else {
        nodes
            .iter()
            .map(|node| node.name.to_ascii_lowercase())
            .collect::<Vec<_>>()
    };
    let ordinal = ordered_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let roots = ordered_names
        .iter()
        .filter(|name| selected[*name].parent_name.is_none())
        .cloned()
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-HIERARCHY",
            "referenceMdl.nodes",
            format!(
                "node hierarchy requires exactly one root, found {}",
                roots.len()
            ),
        ));
    }
    let mut visiting = BTreeSet::new();
    let root = build_ascii_node_report_v2(&roots[0], &selected, &ordinal, &mut visiting)?;
    let node_count = selected.len();
    let reachable = count_node_reports_v2(&root);
    if reachable != node_count {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-HIERARCHY",
            "referenceMdl.nodes",
            "node hierarchy contains an unreachable node or cycle",
        ));
    }
    Ok(NodeTreeReport {
        declared_node_count: node_count,
        node_count,
        max_depth: node_depth_v2(&root),
        roots: vec![root],
    })
}

fn build_ascii_node_report_v2(
    name: &str,
    nodes: &BTreeMap<String, AsciiNodeV2>,
    ordinal: &BTreeMap<String, usize>,
    visiting: &mut BTreeSet<String>,
) -> Result<NodeReport, ReferenceSupermodelGenericErrorV2> {
    if !visiting.insert(name.to_owned()) {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ASCII-HIERARCHY",
            "referenceMdl.nodes",
            "node hierarchy contains a cycle",
        ));
    }
    let node = &nodes[name];
    let mut child_names = nodes
        .iter()
        .filter(|(_, candidate)| {
            candidate
                .parent_name
                .as_ref()
                .is_some_and(|parent| parent.eq_ignore_ascii_case(&node.name))
        })
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    child_names.sort_by_key(|child| ordinal.get(child).copied().unwrap_or(usize::MAX));
    let children = child_names
        .iter()
        .map(|child| build_ascii_node_report_v2(child, nodes, ordinal, visiting))
        .collect::<Result<Vec<_>, _>>()?;
    visiting.remove(name);
    let index = ordinal[name];
    let offset = u32::try_from(index + 1).unwrap_or(u32::MAX);
    let parent_offset = node
        .parent_name
        .as_ref()
        .map(|parent| {
            ordinal
                .get(&parent.to_ascii_lowercase())
                .copied()
                .ok_or_else(|| {
                    generic_error(
                        "M2A-REFERENCE-SUPERMODEL-ASCII-PARENT-MISSING",
                        format!("referenceMdl.nodes[{index}].parent"),
                        format!("parent {parent:?} is absent"),
                    )
                })
        })
        .transpose()?
        .map(|parent| u32::try_from(parent + 1).unwrap_or(u32::MAX));
    Ok(NodeReport {
        offset,
        number: u32::try_from(index).unwrap_or(u32::MAX),
        name: node.name.clone(),
        parent_offset,
        inherit_color: 0,
        content_flags: 1,
        unsupported_families: Vec::new(),
        children_header: ArrayReport {
            pointer: 0,
            used: children.len(),
            allocated: children.len(),
        },
        controller_keys_header: ArrayReport {
            pointer: 0,
            used: node.controllers.len(),
            allocated: node.controllers.len(),
        },
        controller_data_header: empty_array_v2(),
        controllers: node.controllers.clone(),
        mesh: None,
        skin: None,
        aabb: None,
        children,
    })
}

fn empty_array_v2() -> ArrayReport {
    ArrayReport {
        pointer: 0,
        used: 0,
        allocated: 0,
    }
}
fn count_node_reports_v2(node: &NodeReport) -> usize {
    1 + node
        .children
        .iter()
        .map(count_node_reports_v2)
        .sum::<usize>()
}
fn node_depth_v2(node: &NodeReport) -> usize {
    1 + node.children.iter().map(node_depth_v2).max().unwrap_or(0)
}

/// Audits the weights actually authored on the visible surface. `allowed` is
/// only a writer permission set; it is deliberately not used as a proxy for
/// active weights.
pub fn audit_generic_reference_skin_influences_v3(
    contract: &ReferenceSupermodelMotionContractV2,
    allowed: &[u32],
    weights: &[Vec<RigWeightInfluenceV1>],
) -> Result<GenericReferenceSkinInfluenceAuditV3, ReferenceSupermodelGenericErrorV2> {
    let allowed_input_len = allowed.len();
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    if allowed.len() != allowed_input_len
        || allowed
            .iter()
            .any(|part| (*part as usize) >= contract.nodes.len())
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-ALLOWED-BONES-INVALID",
            "rig.allowedBoneNodeIds",
            "allowed bone references must be unique carrier parts",
        ));
    }
    let mut positive_vertices = BTreeMap::<u32, BTreeSet<usize>>::new();
    let mut visible_vertices = BTreeMap::<u32, BTreeSet<usize>>::new();
    for (vertex, row) in weights.iter().enumerate() {
        let mut sum = 0.0_f32;
        let mut dominant = None::<(f32, u32)>;
        for influence in row {
            if !allowed.contains(&influence.bone_node_id)
                || !influence.value.is_finite()
                || influence.value < 0.0
            {
                return Err(generic_error(
                    "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-INVALID",
                    format!("rig.referenceWeights[{vertex}]"),
                    "every influence must reference an allowed carrier and have a finite non-negative value",
                ));
            }
            sum += influence.value;
            if influence.value > 0.0 {
                positive_vertices
                    .entry(influence.bone_node_id)
                    .or_default()
                    .insert(vertex);
            }
            if dominant.is_none_or(|(value, bone)| {
                influence.value > value
                    || (influence.value == value && influence.bone_node_id < bone)
            }) {
                dominant = Some((influence.value, influence.bone_node_id));
            }
        }
        if row.is_empty() || !sum.is_finite() || (sum - 1.0).abs() > 1.0e-4 {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-SKIN-WEIGHT-NOT-NORMALIZED",
                format!("rig.referenceWeights[{vertex}]"),
                "every visible vertex requires a non-empty normalized weight row",
            ));
        }
        if let Some((_, bone)) = dominant.filter(|(value, _)| *value >= 0.1) {
            visible_vertices.entry(bone).or_default().insert(vertex);
        }
    }
    let active = positive_vertices.keys().copied().collect::<BTreeSet<_>>();
    let mut unweighted_required_joint_names = Vec::new();
    let mut passive_unweighted_joint_names = Vec::new();
    let mut joint_influences = Vec::with_capacity(contract.nodes.len());
    for node in &contract.nodes {
        let positive_influence_vertex_count = positive_vertices
            .get(&node.part_number)
            .map_or(0, BTreeSet::len);
        let visible_cluster_vertex_count = visible_vertices
            .get(&node.part_number)
            .map_or(0, BTreeSet::len);
        if node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant {
            if positive_influence_vertex_count == 0 || visible_cluster_vertex_count == 0 {
                unweighted_required_joint_names.push(node.name.clone());
            }
        } else if positive_influence_vertex_count == 0 {
            passive_unweighted_joint_names.push(node.name.clone());
        }
        joint_influences.push(GenericReferenceJointInfluenceV3 {
            joint_name: node.name.clone(),
            carrier_class: node.carrier_class,
            positive_influence_vertex_count,
            visible_cluster_vertex_count,
        });
    }
    Ok(GenericReferenceSkinInfluenceAuditV3 {
        schema_version: 3,
        allowed_bone_count: allowed.len(),
        weighted_bone_count: active.len(),
        active_weighted_bone_count: active.len(),
        skin_influence_coverage: unweighted_required_joint_names.is_empty(),
        unweighted_required_joint_names,
        passive_unweighted_joint_names,
        joint_influences,
    })
}

pub fn derive_generic_reference_rig_from_surface_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
    surface_indices: &[u32],
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    if contract.nodes.is_empty() || contract.nodes[0].parent_part_number.is_some() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
            "contract.nodes",
            "generic retargeting requires one ordered carrier root",
        ));
    }
    if surface_positions.is_empty() || surface_indices.is_empty() || surface_indices.len() % 3 != 0
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INVALID",
            "surface",
            "retargeting requires a non-empty indexed triangle surface",
        ));
    }
    if surface_positions
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
        || surface_indices
            .iter()
            .any(|index| *index as usize >= surface_positions.len())
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INVALID",
            "surface",
            "surface positions and indices must be finite and in bounds",
        ));
    }
    let target_bounds = bounds_v2(surface_positions)?;
    let structural_profile = build_reference_supermodel_structural_profile_v1(contract)?;
    let surface_anatomy = analyze_target_surface_anatomy_v1(surface_positions, surface_indices)?;
    let joint_fit = fit_reference_supermodel_joints_with_anatomy_v1(
        contract,
        surface_positions,
        &structural_profile,
        &surface_anatomy,
    )?;
    let fitted_worlds = &joint_fit.fitted_world_positions;
    let fitted_ground_contact_chain_count = joint_fit.report.fitted_ground_contact_chain_count;
    let nodes = build_fitted_bind_nodes_preserving_reference_frames_v2(contract, fitted_worlds)?;
    let mut allowed = contract
        .nodes
        .iter()
        .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    if allowed.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-JOINTS-MISSING",
            "contract.nodes",
            "the full inherited controller corpus exposes no skin-relevant carrier joint",
        ));
    }
    allowed.sort_unstable();
    allowed.dedup();
    let skinning = derive_local_reference_skinning_v1(
        contract,
        surface_positions,
        surface_indices,
        &allowed,
        fitted_worlds,
        &surface_anatomy,
    )?;
    let reference_weights = skinning.weights;
    let initial_influence_audit =
        audit_generic_reference_skin_influences_v3(contract, &allowed, &reference_weights)?;
    let influence_audit =
        audit_generic_reference_skin_influences_v3(contract, &allowed, &reference_weights)?;
    if !influence_audit.skin_influence_coverage {
        let missing_details = influence_audit
            .joint_influences
            .iter()
            .filter(|joint| {
                influence_audit
                    .unweighted_required_joint_names
                    .contains(&joint.joint_name)
            })
            .map(|joint| {
                let feasibility = skinning
                    .report
                    .skin_region_feasibility
                    .iter()
                    .find(|region| region.joint_name == joint.joint_name);
                match feasibility {
                    Some(region) => format!(
                        "{}(positive={}, visible={}, availableDepth={:.6}, requiredWidth={:.6}, ratio={:.6}, childBoundaryDepth={:?})",
                        joint.joint_name,
                        joint.positive_influence_vertex_count,
                        joint.visible_cluster_vertex_count,
                        region.available_geodesic_depth_fraction,
                        region.required_blend_width_fraction,
                        region.available_to_required_ratio,
                        region.minimum_child_boundary_depth_fraction,
                    ),
                    None => format!(
                        "{}(positive={}, visible={}, feasibility=missing)",
                        joint.joint_name,
                        joint.positive_influence_vertex_count,
                        joint.visible_cluster_vertex_count,
                    ),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-LOCAL-COVERAGE-BLOCKED",
            "rig.referenceWeights",
            format!(
                "local skinning left required joints without a semantic surface cluster: {missing_details}; boundaryRepairReassigned={}, boundaryRigidGroups={}",
                skinning.report.branch_boundary_repair_vertex_count,
                skinning.report.local_boundary_rigid_group_count,
            ),
        ));
    }
    let surface_component_count =
        connected_surface_components_v3(surface_positions.len(), surface_indices)?.len();
    let duplicate_position_group_count = duplicate_position_group_count_v2(surface_positions);
    let bind_pose = validate_reference_supermodel_bind_pose_v1(
        contract,
        &structural_profile,
        &surface_anatomy.report,
        &joint_fit.report,
        &skinning.report,
        surface_positions,
        &reference_weights,
    )?;
    let segment = CreatureRigSegmentV1 {
        id: 1,
        name: "structure_derived_reference_surface_v2".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: contract.nodes[0].part_number,
        surface_positions: surface_positions.to_vec(),
        surface_indices: surface_indices.to_vec(),
        allowed_bone_node_ids: allowed.clone(),
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: format!("structure-derived-{}-v2", contract.supermodel_resref),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor: [0.0, 0.0, 0.0],
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig)
        .map_err(|source| generic_error(source.code, source.path, source.message))?;
    Ok(GenericReferenceRigArtifactV2 {
        report: GenericReferenceRigAnalysisV2 {
            schema_version: 3,
            algorithm: "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V48".to_owned(),
            carrier_node_count: contract.nodes.len(),
            full_carrier_coverage: rig.nodes.len() == contract.nodes.len(),
            required_joint_coverage: rig.nodes.len() == contract.nodes.len(),
            allowed_bone_count: influence_audit.allowed_bone_count,
            weighted_bone_count: influence_audit.weighted_bone_count,
            active_weighted_bone_count: influence_audit.active_weighted_bone_count,
            initial_unweighted_required_joint_names: initial_influence_audit
                .unweighted_required_joint_names,
            unweighted_required_joint_names: influence_audit.unweighted_required_joint_names,
            passive_unweighted_joint_names: influence_audit.passive_unweighted_joint_names,
            skin_influence_coverage: influence_audit.skin_influence_coverage,
            joint_influences: influence_audit.joint_influences,
            surface_vertex_count: surface_positions.len(),
            surface_triangle_count: surface_indices.len() / 3,
            surface_component_count,
            stabilized_small_component_count: skinning.report.component_projection_count,
            stabilized_small_component_vertex_count: skinning
                .report
                .component_projection_vertex_count,
            fitted_ground_contact_chain_count,
            duplicate_position_group_count,
            no_reference_payload_copied: true,
            structural_profile,
            surface_anatomy: surface_anatomy.report,
            joint_fit: joint_fit.report,
            skinning: skinning.report,
            bind_pose,
        },
        rig,
    })
}

/// Applies an inspected supermodel without retargeting its named carrier
/// hierarchy. The caller-owned mesh is uniformly registered to the reference
/// render envelope; carrier names, parents and local bind matrices are copied
/// from the exact clean-room contract unchanged.
pub fn derive_immutable_reference_supermodel_rig_from_surface_v3(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
    surface_indices: &[u32],
    reference_render_bounds: Bounds3V1,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    derive_immutable_reference_supermodel_rig_from_surface_v4(
        contract,
        surface_positions,
        surface_indices,
        reference_render_bounds,
        ReferenceSupermodelSkinningOptionsV1::default(),
    )
}

/// V4 keeps the selected supermodel hierarchy and bind immutable while
/// allowing callers to explicitly choose the skinning safety policy.
pub fn derive_immutable_reference_supermodel_rig_from_surface_v4(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
    surface_indices: &[u32],
    reference_render_bounds: Bounds3V1,
    skinning_options: ReferenceSupermodelSkinningOptionsV1,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    derive_immutable_reference_supermodel_rig_from_surface_guided_v5(
        contract,
        surface_positions,
        surface_indices,
        reference_render_bounds,
        skinning_options,
        None,
    )
}

fn derive_immutable_reference_supermodel_rig_from_surface_guided_v5(
    contract: &ReferenceSupermodelMotionContractV2,
    surface_positions: &[[f32; 3]],
    surface_indices: &[u32],
    reference_render_bounds: Bounds3V1,
    skinning_options: ReferenceSupermodelSkinningOptionsV1,
    reference_guide: Option<&InspectionReport>,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    if contract.nodes.is_empty() || contract.nodes[0].parent_part_number.is_some() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
            "contract.nodes",
            "immutable supermodel application requires one ordered carrier root",
        ));
    }
    if surface_positions.is_empty() || surface_indices.is_empty() || surface_indices.len() % 3 != 0
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INVALID",
            "surface",
            "immutable supermodel application requires a non-empty indexed triangle surface",
        ));
    }
    if surface_positions
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
        || surface_indices
            .iter()
            .any(|index| *index as usize >= surface_positions.len())
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SURFACE-INVALID",
            "surface",
            "surface positions and indices must be finite and in bounds",
        ));
    }
    let reference_height = reference_render_bounds.max[2] - reference_render_bounds.min[2];
    let source_bounds = bounds_v2(surface_positions)?;
    let source_height = source_bounds.max[2] - source_bounds.min[2];
    if !reference_height.is_finite()
        || reference_height <= 1.0e-6
        || !source_height.is_finite()
        || source_height <= 1.0e-6
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-REGISTRATION-BOUNDS-INVALID",
            "referenceRenderBounds",
            "source and selected-supermodel render envelopes require positive finite height",
        ));
    }
    let source_anchor = bottom_center_v3(source_bounds);
    let alignment_anchor = bottom_center_v3(reference_render_bounds);
    let scale = reference_height / source_height;
    let registered_world_positions = surface_positions
        .iter()
        .map(|point| {
            std::array::from_fn(|axis| {
                (point[axis] - source_anchor[axis]) * scale + alignment_anchor[axis]
            })
        })
        .collect::<Vec<_>>();
    let target_bounds = bounds_v2(&registered_world_positions)?;
    let structural_profile = build_reference_supermodel_structural_profile_v1(contract)?;
    let surface_anatomy =
        analyze_target_surface_anatomy_v1(&registered_world_positions, surface_indices)?;
    let joint_fit = build_immutable_reference_supermodel_joint_report_v1(
        contract,
        &structural_profile,
        &surface_anatomy,
        target_bounds,
    )?;
    let exact_world_matrices = exact_contract_world_matrices_v3(contract)?;
    let exact_world_positions = exact_world_matrices
        .iter()
        .map(|world| [world[12], world[13], world[14]])
        .collect::<Vec<_>>();
    let nodes = contract
        .nodes
        .iter()
        .map(|node| CreatureRigNodeV1 {
            id: node.part_number,
            name: node.name.clone(),
            parent_id: node.parent_part_number,
            bind_local_matrix: node.carrier_bind_local_matrix,
        })
        .collect::<Vec<_>>();
    let mut allowed = contract
        .nodes
        .iter()
        .filter(|node| node.carrier_class == ReferenceSupermodelCarrierClassV3::SkinRelevant)
        .map(|node| node.part_number)
        .collect::<Vec<_>>();
    allowed.sort_unstable();
    allowed.dedup();
    if allowed.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-JOINTS-MISSING",
            "contract.nodes",
            "the selected supermodel exposes no skin-relevant carrier joint",
        ));
    }
    let guide_labels = reference_guide
        .map(|reference| {
            crate::reference_surface_guides::reference_surface_labels_v1(
                reference,
                contract,
                &registered_world_positions,
                &allowed,
            )
        })
        .transpose()?;
    let skinning =
        crate::reference_supermodel_skinning::derive_local_reference_skinning_with_seed_labels_v2(
            contract,
            &registered_world_positions,
            surface_indices,
            &allowed,
            &exact_world_positions,
            &surface_anatomy,
            skinning_options,
            guide_labels.as_deref(),
        )?;
    let reference_weights = skinning.weights;
    let influence_audit =
        audit_generic_reference_skin_influences_v3(contract, &allowed, &reference_weights)?;
    if !influence_audit.skin_influence_coverage
        && !skinning_options.retain_editable_draft_on_quality_failure
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SKIN-LOCAL-COVERAGE-BLOCKED",
            "rig.referenceWeights",
            format!(
                "immutable-bind skinning left required joints without a visible cluster: {}",
                influence_audit.unweighted_required_joint_names.join(", ")
            ),
        ));
    }
    let bind_pose = validate_reference_supermodel_bind_pose_v1(
        contract,
        &structural_profile,
        &surface_anatomy.report,
        &joint_fit,
        &skinning.report,
        &registered_world_positions,
        &reference_weights,
    )?;
    let root_inverse = inverse_affine_v2(exact_world_matrices[0]).ok_or_else(|| {
        generic_error(
            "M2A-REFERENCE-SUPERMODEL-ROOT-NONINVERTIBLE",
            "contract.nodes[0].carrierBindLocalMatrix",
            "the immutable selected-supermodel root must be an invertible affine transform",
        )
    })?;
    let local_surface_positions = registered_world_positions
        .iter()
        .map(|point| transform_point_v2(root_inverse, *point))
        .collect::<Vec<_>>();
    let segment = CreatureRigSegmentV1 {
        id: 1,
        name: "immutable_reference_bind_surface_v1".to_owned(),
        deformation: RigSegmentDeformationV1::Skin,
        parent_node_id: contract.nodes[0].part_number,
        surface_positions: local_surface_positions,
        surface_indices: surface_indices.to_vec(),
        allowed_bone_node_ids: allowed,
        reference_weights,
    };
    let mut rig = CreatureRigProfileV1 {
        schema_version: 1,
        profile_id: format!("immutable-reference-bind-{}-v1", contract.supermodel_resref),
        content_sha256: String::new(),
        provenance: RigProvenanceV1 {
            kind: RigProvenanceKindV1::Owned,
            export_allowed: true,
            attestations: RigProvenanceAttestationsV1 {
                controlled_construction: true,
                no_reference_payload_copied: true,
                rights_confirmed: true,
            },
        },
        target_bounds,
        alignment_anchor,
        nodes,
        segments: vec![segment],
    };
    rig.content_sha256 = canonical_profile_sha256(&rig)
        .map_err(|source| generic_error(source.code, source.path, source.message))?;
    let surface_component_count =
        connected_surface_components_v3(surface_positions.len(), surface_indices)?.len();
    let duplicate_position_group_count =
        duplicate_position_group_count_v2(&registered_world_positions);
    Ok(GenericReferenceRigArtifactV2 {
        report: GenericReferenceRigAnalysisV2 {
            schema_version: 5,
            algorithm: "IMMUTABLE_REFERENCE_BIND_MESH_REGISTRATION_V1".to_owned(),
            carrier_node_count: contract.nodes.len(),
            full_carrier_coverage: rig.nodes.len() == contract.nodes.len(),
            required_joint_coverage: rig.nodes.len() == contract.nodes.len(),
            allowed_bone_count: influence_audit.allowed_bone_count,
            weighted_bone_count: influence_audit.weighted_bone_count,
            active_weighted_bone_count: influence_audit.active_weighted_bone_count,
            initial_unweighted_required_joint_names: influence_audit
                .unweighted_required_joint_names
                .clone(),
            unweighted_required_joint_names: influence_audit.unweighted_required_joint_names,
            passive_unweighted_joint_names: influence_audit.passive_unweighted_joint_names,
            skin_influence_coverage: influence_audit.skin_influence_coverage,
            joint_influences: influence_audit.joint_influences,
            surface_vertex_count: registered_world_positions.len(),
            surface_triangle_count: surface_indices.len() / 3,
            surface_component_count,
            stabilized_small_component_count: skinning.report.component_projection_count,
            stabilized_small_component_vertex_count: skinning
                .report
                .component_projection_vertex_count,
            fitted_ground_contact_chain_count: joint_fit.fitted_ground_contact_chain_count,
            duplicate_position_group_count,
            no_reference_payload_copied: true,
            structural_profile,
            surface_anatomy: surface_anatomy.report,
            joint_fit,
            skinning: skinning.report,
            bind_pose,
        },
        rig,
    })
}

pub fn validate_authored_generic_reference_rig_v3(
    contract: &ReferenceSupermodelMotionContractV2,
    base_analysis: &GenericReferenceRigAnalysisV2,
    authored_rig: &CreatureRigProfileV1,
    authored_joint_part_numbers: &BTreeSet<u32>,
) -> Result<GenericReferenceRigAnalysisV2, ReferenceSupermodelGenericErrorV2> {
    if authored_rig.nodes.len() != contract.nodes.len() || authored_rig.segments.len() != 1 {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORED-RIG-SHAPE",
            "authoredRig",
            "authored validation requires the exact carrier inventory and one owned render segment",
        ));
    }
    for (node, carrier) in authored_rig.nodes.iter().zip(&contract.nodes) {
        if node.id != carrier.part_number
            || node.name != carrier.name
            || node.parent_id != carrier.parent_part_number
        {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-AUTHORED-RIG-TOPOLOGY",
                format!("authoredRig.nodes[{}]", node.id),
                "authored validation may not rename, add, remove or reparent exact carriers",
            ));
        }
    }
    let segment = &authored_rig.segments[0];
    let anatomy =
        analyze_target_surface_anatomy_v1(&segment.surface_positions, &segment.surface_indices)?;
    if anatomy.report.content_sha256 != base_analysis.surface_anatomy.content_sha256
        || base_analysis.structural_profile.content_sha256
            != base_analysis.joint_fit.structural_profile_sha256
    {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-AUTHORED-RIG-ANALYSIS-CONTEXT",
            "authoredRig",
            "authored validation must reuse the exact hashed surface analysis and structural profile",
        ));
    }
    let fitted_worlds = reference_supermodel_rig_world_positions_v1(authored_rig)?;
    let immutable_bind = base_analysis.joint_fit.algorithm == "IMMUTABLE_REFERENCE_BIND_V1";
    let joint_fit = if immutable_bind {
        if !authored_joint_part_numbers.is_empty() {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-IMMUTABLE-BIND-OVERRIDE-FORBIDDEN",
                "authoredRig.nodes",
                "selected supermodel carrier transforms are immutable; author mesh registration or skin weights instead",
            ));
        }
        for (index, (node, carrier)) in authored_rig.nodes.iter().zip(&contract.nodes).enumerate() {
            let maximum_error = node
                .bind_local_matrix
                .iter()
                .zip(carrier.carrier_bind_local_matrix)
                .map(|(actual, expected)| (actual - expected).abs())
                .fold(0.0_f32, f32::max);
            if maximum_error > contract.tolerances.bind_max_abs_error {
                return Err(generic_error(
                    "M2A-REFERENCE-SUPERMODEL-IMMUTABLE-BIND-DRIFT",
                    format!("authoredRig.nodes[{index}].bindLocalMatrix"),
                    format!("exact selected-supermodel bind drifted by {maximum_error}"),
                ));
            }
        }
        build_immutable_reference_supermodel_joint_report_v1(
            contract,
            &base_analysis.structural_profile,
            &anatomy,
            bounds_v2(&segment.surface_positions)?,
        )?
    } else {
        revalidate_authored_reference_supermodel_joints_v2(
            contract,
            &base_analysis.structural_profile,
            &anatomy,
            &base_analysis.joint_fit,
            &fitted_worlds,
            authored_joint_part_numbers,
        )?
    };
    let skinning = validate_authored_reference_skinning_v2(
        contract,
        &segment.surface_positions,
        &segment.surface_indices,
        &segment.allowed_bone_node_ids,
        &fitted_worlds,
        &segment.reference_weights,
        &base_analysis.skinning,
    )?;
    let bind_pose = validate_reference_supermodel_bind_pose_v1(
        contract,
        &base_analysis.structural_profile,
        &anatomy.report,
        &joint_fit,
        &skinning,
        &segment.surface_positions,
        &segment.reference_weights,
    )?;
    let influence_audit = audit_generic_reference_skin_influences_v3(
        contract,
        &segment.allowed_bone_node_ids,
        &segment.reference_weights,
    )?;
    let mut report = base_analysis.clone();
    report.schema_version = 4;
    report.algorithm = "STRUCTURAL_ANATOMY_AUTHORED_VALIDATION_V49".to_owned();
    report.full_carrier_coverage = authored_rig.nodes.len() == contract.nodes.len();
    report.required_joint_coverage = report.full_carrier_coverage;
    report.weighted_bone_count = influence_audit.weighted_bone_count;
    report.active_weighted_bone_count = influence_audit.active_weighted_bone_count;
    report.unweighted_required_joint_names = influence_audit.unweighted_required_joint_names;
    report.passive_unweighted_joint_names = influence_audit.passive_unweighted_joint_names;
    report.skin_influence_coverage = influence_audit.skin_influence_coverage;
    report.joint_influences = influence_audit.joint_influences;
    report.surface_anatomy = anatomy.report;
    report.joint_fit = joint_fit;
    report.skinning = skinning;
    report.bind_pose = bind_pose;
    Ok(report)
}

/// Extracts the exact static render surface from the selected GLB and derives
/// a structure-driven target rig. The basis is the same locked Creature V3
/// basis later used by Profile A, so weight lookup and emitted geometry share
/// one coordinate space.
pub fn derive_generic_reference_rig_from_glb_v2(
    source: &GlbIngestResult,
    contract: &ReferenceSupermodelMotionContractV2,
    source_forward: CreatureSourceForwardV1,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    if source.ir.primitives.len() != 1 {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SOURCE-PRIMITIVE-COUNT",
            "source.ir.primitives",
            format!(
                "structure-driven retargeting currently requires exactly one render primitive, found {}",
                source.ir.primitives.len()
            ),
        ));
    }
    if !source.ir.skins.is_empty() || !source.ir.animations.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SOURCE-NOT-STATIC",
            "source.ir",
            "reference-supermodel retargeting requires an unskinned, unanimated source surface",
        ));
    }
    let primitive = &source.ir.primitives[0];
    let mut positions = primitive
        .positions
        .iter()
        .copied()
        .map(|point| creature_basis_point_v2(point, source_forward))
        .collect::<Vec<_>>();
    let raw_bounds = bounds_v2(&positions)?;
    let center_x = (raw_bounds.min[0] + raw_bounds.max[0]) * 0.5;
    let center_y = (raw_bounds.min[1] + raw_bounds.max[1]) * 0.5;
    let ground_z = raw_bounds.min[2];
    for point in &mut positions {
        point[0] -= center_x;
        point[1] -= center_y;
        point[2] -= ground_z;
    }
    derive_generic_reference_rig_from_surface_v2(contract, &positions, &primitive.indices)
}

/// Production selected-supermodel route. It measures the exact reference
/// render envelope, registers the owned source mesh into that coordinate
/// space and leaves the complete named carrier bind untouched.
pub fn derive_immutable_reference_supermodel_rig_from_glb_v3(
    source: &GlbIngestResult,
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
    source_forward: CreatureSourceForwardV1,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    derive_immutable_reference_supermodel_rig_from_glb_v4(
        source,
        contract,
        reference,
        source_forward,
        ReferenceSupermodelSkinningOptionsV1::default(),
    )
}

/// V4 exposes the explicit skinning policy without changing the selected
/// supermodel topology, names, parents or bind matrices.
pub fn derive_immutable_reference_supermodel_rig_from_glb_v4(
    source: &GlbIngestResult,
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
    source_forward: CreatureSourceForwardV1,
    skinning_options: ReferenceSupermodelSkinningOptionsV1,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    if source.ir.primitives.len() != 1 {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SOURCE-PRIMITIVE-COUNT",
            "source.ir.primitives",
            format!(
                "immutable supermodel application currently requires exactly one render primitive, found {}",
                source.ir.primitives.len()
            ),
        ));
    }
    if !source.ir.skins.is_empty() || !source.ir.animations.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SOURCE-NOT-STATIC",
            "source.ir",
            "reference-supermodel application requires an unskinned, unanimated source surface",
        ));
    }
    let primitive = &source.ir.primitives[0];
    let positions = primitive
        .positions
        .iter()
        .copied()
        .map(|point| creature_basis_point_v2(point, source_forward))
        .collect::<Vec<_>>();
    let reference_render_bounds = exact_reference_render_bounds_v3(contract, reference)?;
    derive_immutable_reference_supermodel_rig_from_surface_guided_v5(
        contract,
        &positions,
        &primitive.indices,
        reference_render_bounds,
        skinning_options,
        Some(reference),
    )
}

/// The caller must verify a source declaration bound to the selected reference SHA-256.
/// Geometry is already in the reference bind frame; no bounding-box rescaling is applied.
pub fn derive_registered_reference_supermodel_rig_from_glb_v1(
    source: &GlbIngestResult,
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
    source_forward: CreatureSourceForwardV1,
    skinning_options: ReferenceSupermodelSkinningOptionsV1,
) -> Result<GenericReferenceRigArtifactV2, ReferenceSupermodelGenericErrorV2> {
    if source.ir.primitives.len() != 1 {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SOURCE-PRIMITIVE-COUNT",
            "source.ir.primitives",
            format!(
                "immutable supermodel application currently requires exactly one render primitive, found {}",
                source.ir.primitives.len()
            ),
        ));
    }
    if !source.ir.skins.is_empty() || !source.ir.animations.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-SOURCE-NOT-STATIC",
            "source.ir",
            "reference-supermodel application requires an unskinned, unanimated source surface",
        ));
    }
    let primitive = &source.ir.primitives[0];
    let positions = primitive
        .positions
        .iter()
        .copied()
        .map(|point| creature_basis_point_v2(point, source_forward))
        .collect::<Vec<_>>();
    let reference_render_bounds = bounds_v2(&positions)?;
    derive_immutable_reference_supermodel_rig_from_surface_guided_v5(
        contract,
        &positions,
        &primitive.indices,
        reference_render_bounds,
        skinning_options,
        Some(reference),
    )
}

fn exact_reference_render_bounds_v3(
    contract: &ReferenceSupermodelMotionContractV2,
    reference: &InspectionReport,
) -> Result<Bounds3V1, ReferenceSupermodelGenericErrorV2> {
    fn collect<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
        for node in nodes {
            output.push(node);
            collect(&node.children, output);
        }
    }
    let mut nodes = Vec::new();
    collect(&reference.node_tree.roots, &mut nodes);
    if nodes.len() != contract.nodes.len() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-REFERENCE-NODE-COUNT-MISMATCH",
            "reference.nodeTree",
            "reference render-envelope extraction requires the exact contract carrier inventory",
        ));
    }
    let worlds = exact_contract_world_matrices_v3(contract)?;
    let mut points = Vec::<[f32; 3]>::new();
    for (part, (node, carrier)) in nodes.iter().zip(&contract.nodes).enumerate() {
        if node.number as usize != part || !node.name.eq_ignore_ascii_case(&carrier.name) {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-REFERENCE-TOPOLOGY-MISMATCH",
                format!("reference.nodeTree.nodes[{part}]"),
                "reference render-envelope carrier identity differs from the exact contract",
            ));
        }
        if let Some(mesh) = node.mesh.as_ref().filter(|mesh| mesh.render != 0) {
            points.extend(
                mesh.vertices
                    .iter()
                    .map(|vertex| transform_point_v2(worlds[part], [vertex.x, vertex.y, vertex.z])),
            );
        }
    }
    if points.is_empty() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-REFERENCE-RENDER-ENVELOPE-MISSING",
            "reference.nodeTree.meshes",
            "automatic immutable-bind application requires visible reference geometry; choose a mesh-bearing compatible child when the selected animation supermodel is abstract",
        ));
    }
    bounds_v2(&points)
}

fn exact_contract_world_matrices_v3(
    contract: &ReferenceSupermodelMotionContractV2,
) -> Result<Vec<[f32; 16]>, ReferenceSupermodelGenericErrorV2> {
    let mut worlds = Vec::with_capacity(contract.nodes.len());
    for (part, node) in contract.nodes.iter().enumerate() {
        let world = if let Some(parent) = node.parent_part_number {
            let parent_world = worlds.get(parent as usize).copied().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
                    format!("contract.nodes[{part}].parentPartNumber"),
                    "an exact carrier parent must precede its child",
                )
            })?;
            multiply_matrix_v2(parent_world, node.carrier_bind_local_matrix)
        } else {
            if part != 0 {
                return Err(generic_error(
                    "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
                    format!("contract.nodes[{part}].parentPartNumber"),
                    "only the first exact carrier may be a root",
                ));
            }
            node.carrier_bind_local_matrix
        };
        if world.iter().any(|value| !value.is_finite()) {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-REFERENCE-BIND-INVALID",
                format!("contract.nodes[{part}].carrierBindLocalMatrix"),
                "exact carrier world bind must be finite",
            ));
        }
        worlds.push(world);
    }
    Ok(worlds)
}

fn bottom_center_v3(bounds: Bounds3V1) -> [f32; 3] {
    [
        (bounds.min[0] + bounds.max[0]) * 0.5,
        (bounds.min[1] + bounds.max[1]) * 0.5,
        bounds.min[2],
    ]
}

fn creature_basis_point_v2(point: [f32; 3], source_forward: CreatureSourceForwardV1) -> [f32; 3] {
    match source_forward {
        CreatureSourceForwardV1::PositiveZ => [-point[0], point[2], point[1]],
        CreatureSourceForwardV1::NegativeZ => [point[0], -point[2], point[1]],
        CreatureSourceForwardV1::PositiveX => [point[1], point[2], point[0]],
        CreatureSourceForwardV1::NegativeX => [-point[1], point[2], -point[0]],
    }
}

fn connected_surface_components_v3(
    vertex_count: usize,
    indices: &[u32],
) -> Result<Vec<Vec<usize>>, ReferenceSupermodelGenericErrorV2> {
    let mut adjacency = vec![Vec::<usize>::new(); vertex_count];
    for triangle in indices.chunks_exact(3) {
        let vertices = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if vertices.iter().any(|vertex| *vertex >= vertex_count) {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-SURFACE-INDEX-INVALID",
                "surface.indices",
                "surface component analysis encountered an out-of-range index",
            ));
        }
        for (left, right) in [(0, 1), (1, 2), (2, 0)] {
            adjacency[vertices[left]].push(vertices[right]);
            adjacency[vertices[right]].push(vertices[left]);
        }
    }
    let mut visited = vec![false; vertex_count];
    let mut components = Vec::new();
    for start in 0..vertex_count {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut pending = vec![start];
        let mut component = Vec::new();
        while let Some(vertex) = pending.pop() {
            component.push(vertex);
            for &neighbor in &adjacency[vertex] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    pending.push(neighbor);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    Ok(components)
}

fn bounds_v2(points: &[[f32; 3]]) -> Result<Bounds3V1, ReferenceSupermodelGenericErrorV2> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    if min.iter().chain(&max).any(|value| !value.is_finite()) {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-BOUNDS-INVALID",
            "surface",
            "bounds are non-finite",
        ));
    }
    Ok(Bounds3V1 { min, max })
}

/// Retargets carrier pivots without changing the exact local coordinate frame
/// in which inherited transform controllers were authored.  Replacing a
/// reference bind matrix with a translation-only matrix is invalid whenever
/// the reference node has a non-identity bind orientation (for example a tail,
/// ear, wing or paw).  Parent rotation also means that a world-space pivot
/// delta is not a valid local translation.
fn build_fitted_bind_nodes_preserving_reference_frames_v2(
    contract: &ReferenceSupermodelMotionContractV2,
    fitted_world_positions: &[[f32; 3]],
) -> Result<Vec<CreatureRigNodeV1>, ReferenceSupermodelGenericErrorV2> {
    if fitted_world_positions.len() != contract.nodes.len() {
        return Err(generic_error(
            "M2A-REFERENCE-SUPERMODEL-JOINT-FIT-COUNT-MISMATCH",
            "jointFit.fittedWorldPositions",
            "the fitted pivot inventory must exactly match the carrier inventory",
        ));
    }
    let mut nodes = Vec::with_capacity(contract.nodes.len());
    let mut target_worlds = Vec::<[f32; 16]>::with_capacity(contract.nodes.len());
    for (index, node) in contract.nodes.iter().enumerate() {
        let mut local = node.carrier_bind_local_matrix;
        let local_translation = if let Some(parent) = node.parent_part_number {
            let parent_world = target_worlds.get(parent as usize).copied().ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-STRUCTURE-INVALID",
                    format!("contract.nodes[{index}].parentPartNumber"),
                    "a carrier parent must precede its child",
                )
            })?;
            let parent_inverse = inverse_affine_v2(parent_world).ok_or_else(|| {
                generic_error(
                    "M2A-REFERENCE-SUPERMODEL-PARENT-FRAME-NONINVERTIBLE",
                    format!("contract.nodes[{index}].carrierBindLocalMatrix"),
                    format!(
                        "the fitted parent frame must be finite and invertible: {parent_world:?}"
                    ),
                )
            })?;
            transform_point_v2(parent_inverse, fitted_world_positions[index])
        } else {
            fitted_world_positions[index]
        };
        local[12] = local_translation[0];
        local[13] = local_translation[1];
        local[14] = local_translation[2];
        if local.iter().any(|value| !value.is_finite()) {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-TARGET-BIND-INVALID",
                format!("targetRig.nodes[{index}].bindLocalMatrix"),
                "the retargeted local bind matrix must be finite",
            ));
        }
        let world = node
            .parent_part_number
            .map(|parent| multiply_matrix_v2(target_worlds[parent as usize], local))
            .unwrap_or(local);
        let world_position = [world[12], world[13], world[14]];
        if distance_v2(world_position, fitted_world_positions[index]) > 1.0e-4 {
            return Err(generic_error(
                "M2A-REFERENCE-SUPERMODEL-TARGET-BIND-PIVOT-MISMATCH",
                format!("targetRig.nodes[{index}].bindLocalMatrix"),
                "the frame-preserving local bind does not reconstruct the fitted world pivot",
            ));
        }
        target_worlds.push(world);
        nodes.push(CreatureRigNodeV1 {
            id: node.part_number,
            name: node.name.clone(),
            parent_id: node.parent_part_number,
            bind_local_matrix: local,
        });
    }
    Ok(nodes)
}

fn multiply_matrix_v2(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0_f32; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

fn transform_point_v2(matrix: [f32; 16], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ]
}

fn inverse_affine_v2(matrix: [f32; 16]) -> Option<[f32; 16]> {
    if matrix.iter().any(|value| !value.is_finite())
        || matrix[3].abs() > 1.0e-6
        || matrix[7].abs() > 1.0e-6
        || matrix[11].abs() > 1.0e-6
        || (matrix[15] - 1.0).abs() > 1.0e-6
    {
        return None;
    }
    let linear = [
        [matrix[0], matrix[4], matrix[8]],
        [matrix[1], matrix[5], matrix[9]],
        [matrix[2], matrix[6], matrix[10]],
    ];
    let inverse = inverse_matrix3_v2(linear)?;
    let translation = [matrix[12], matrix[13], matrix[14]];
    let inverse_translation = multiply_matrix3_vector_v2(inverse, translation.map(|value| -value));
    Some([
        inverse[0][0],
        inverse[1][0],
        inverse[2][0],
        0.0,
        inverse[0][1],
        inverse[1][1],
        inverse[2][1],
        0.0,
        inverse[0][2],
        inverse[1][2],
        inverse[2][2],
        0.0,
        inverse_translation[0],
        inverse_translation[1],
        inverse_translation[2],
        1.0,
    ])
}

fn inverse_matrix3_v2(matrix: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    let determinant = matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]);
    if !determinant.is_finite() || determinant.abs() <= 1.0e-12 {
        return None;
    }
    let inverse = determinant.recip();
    Some([
        [
            (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1]) * inverse,
            (matrix[0][2] * matrix[2][1] - matrix[0][1] * matrix[2][2]) * inverse,
            (matrix[0][1] * matrix[1][2] - matrix[0][2] * matrix[1][1]) * inverse,
        ],
        [
            (matrix[1][2] * matrix[2][0] - matrix[1][0] * matrix[2][2]) * inverse,
            (matrix[0][0] * matrix[2][2] - matrix[0][2] * matrix[2][0]) * inverse,
            (matrix[0][2] * matrix[1][0] - matrix[0][0] * matrix[1][2]) * inverse,
        ],
        [
            (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]) * inverse,
            (matrix[0][1] * matrix[2][0] - matrix[0][0] * matrix[2][1]) * inverse,
            (matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]) * inverse,
        ],
    ])
}

fn multiply_matrix3_vector_v2(matrix: [[f32; 3]; 3], value: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * value[0] + matrix[0][1] * value[1] + matrix[0][2] * value[2],
        matrix[1][0] * value[0] + matrix[1][1] * value[1] + matrix[1][2] * value[2],
        matrix[2][0] * value[0] + matrix[2][1] * value[1] + matrix[2][2] * value[2],
    ]
}

fn distance_v2(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn duplicate_position_group_count_v2(points: &[[f32; 3]]) -> usize {
    let mut counts = BTreeMap::<[u32; 3], usize>::new();
    for point in points {
        *counts.entry(point.map(f32::to_bits)).or_default() += 1;
    }
    counts.values().filter(|count| **count > 1).count()
}
