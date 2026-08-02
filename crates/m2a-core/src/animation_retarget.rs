//! Deterministic animation transfer between two independently inspected H1
//! models. Core owns compatibility, semantic bone mapping and rest-pose delta
//! math so the browser cannot silently use a different retarget algorithm.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    animation_studio::{
        ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION, ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3,
        ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP, AnimationKeyframeV1,
        AnimationRetargetProvenanceV1, AnimationStudioRigNodeV1, AnimationStudioRigV1,
        AuthoredAnimationClipKindV1, AuthoredAnimationClipStatusV1, AuthoredAnimationClipV1,
        AuthoredAnimationEventV1, AuthoredAnimationSourceKindV1, AuthoredAnimationSourceV1,
        AuthoredAnimationTrackPathV1, AuthoredAnimationTrackV1,
        validate_authored_animation_clip_v1,
    },
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationTrackPathV1,
        MdlAnimationTrackV1,
    },
    model_pipeline::inspect_editable_animation_source_v1,
};

pub const ANIMATION_RETARGET_ALGORITHM_V1: &str = "M2A_SAME_HIERARCHY_REST_DELTA_V1";
pub const ANIMATION_RETARGET_ALGORITHM_V2: &str = "M2A_HUMANOID_SEMANTIC_CHAIN_V2";
pub const ANIMATION_RETARGET_LIMITS_V1: &str =
    "UNIQUE_NAMES|SAME_PARENT_GRAPH|TR_ONLY|LINEAR|NO_SCALE_OR_SHEAR";
pub const ANIMATION_TRANSFER_RIG_INCOMPATIBLE: &str = "M2A-ANIMATION-TRANSFER-RIG-INCOMPATIBLE";
pub const ANIMATION_TRANSFER_MODE_BLOCKED: &str = "M2A-ANIMATION-TRANSFER-MODE-BLOCKED";
pub const ANIMATION_TRANSFER_CLIP_MISSING: &str = "M2A-ANIMATION-TRANSFER-CLIP-MISSING";
pub const ANIMATION_TRANSFER_TRACK_UNSUPPORTED: &str = "M2A-ANIMATION-TRANSFER-TRACK-UNSUPPORTED";
pub const ANIMATION_TRANSFER_OUTPUT_INVALID: &str = "M2A-ANIMATION-TRANSFER-OUTPUT-INVALID";
pub const ANIMATION_TRANSFER_LIMIT: &str = "M2A-ANIMATION-RETARGET-LIMIT";

const REST_EPSILON: f32 = 1.0e-5;
const MOTION_EPSILON: f32 = 1.0e-6;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationTransferCompatibilityStatusV1 {
    ExactCopy,
    RetargetableSameHierarchy,
    Incompatible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationTransferModeV1 {
    ExactRigCopyV1,
    SameHierarchyRetargetV1,
    HumanoidSemanticRetargetV2,
}

impl AnimationTransferModeV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactRigCopyV1 => "EXACT_RIG_COPY_V1",
            Self::SameHierarchyRetargetV1 => "SAME_HIERARCHY_RETARGET_V1",
            Self::HumanoidSemanticRetargetV2 => "HUMANOID_SEMANTIC_RETARGET_V2",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferDiagnosticV1 {
    pub code: String,
    pub path: String,
    pub message: String,
    pub action: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticBoneMappingEntryV1 {
    pub bone_name: String,
    pub parent_name: Option<String>,
    pub donor_node_id: u32,
    pub target_node_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticRigMappingV1 {
    pub schema_version: u32,
    pub root_name: String,
    pub entries: Vec<SemanticBoneMappingEntryV1>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HumanoidBoneSemanticV2 {
    Root,
    Hips,
    Spine,
    Chest,
    Neck,
    Head,
    LeftClavicle,
    LeftUpperArm,
    LeftLowerArm,
    LeftHand,
    RightClavicle,
    RightUpperArm,
    RightLowerArm,
    RightHand,
    LeftUpperLeg,
    LeftLowerLeg,
    LeftFoot,
    LeftToe,
    RightUpperLeg,
    RightLowerLeg,
    RightFoot,
    RightToe,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanoidSemanticOverrideV2 {
    pub semantic: HumanoidBoneSemanticV2,
    pub donor_node_id: u32,
    pub target_node_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanoidSemanticBoneEntryV2 {
    pub semantic: HumanoidBoneSemanticV2,
    pub required: bool,
    pub donor_node_id: u32,
    pub donor_node_name: String,
    pub target_node_id: u32,
    pub target_node_name: String,
    pub mapping_source: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HumanoidRetargetCompatibilityStatusV2 {
    Compatible,
    ManualConfirmationRequired,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanoidSemanticBoneMapV2 {
    pub schema_version: u32,
    pub alias_dictionary_version: String,
    pub status: HumanoidRetargetCompatibilityStatusV2,
    pub donor_source_revision: String,
    pub target_source_revision: String,
    pub manual_mapping_confirmed: bool,
    pub entries: Vec<HumanoidSemanticBoneEntryV2>,
    pub diagnostics: Vec<AnimationTransferDiagnosticV1>,
    pub fingerprint_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferCompatibilityV1 {
    pub schema_version: u32,
    pub status: AnimationTransferCompatibilityStatusV1,
    pub donor_source_revision: String,
    pub target_source_revision: String,
    pub donor_rig_signature_sha256: String,
    pub target_rig_signature_sha256: String,
    pub compatibility_fingerprint_sha256: String,
    pub allowed_modes: Vec<AnimationTransferModeV1>,
    pub mapping: Option<SemanticRigMappingV1>,
    pub diagnostics: Vec<AnimationTransferDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferOptionsV1 {
    pub mode: AnimationTransferModeV1,
    pub clip_id: String,
    pub output_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferResultV1 {
    pub schema_version: u32,
    pub required_document_schema_version: u32,
    pub compatibility: AnimationTransferCompatibilityV1,
    pub clip: AuthoredAnimationClipV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnimationTransferBatchCommitV1 {
    AllOrNothing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferBatchClipRequestV1 {
    pub donor_clip_name: String,
    pub mode: AnimationTransferModeV1,
    pub clip_id: String,
    pub output_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferBatchRequestV1 {
    pub schema_version: u32,
    pub commit: AnimationTransferBatchCommitV1,
    pub donor_source_revision: String,
    pub target_source_revision: String,
    pub clips: Vec<AnimationTransferBatchClipRequestV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferBatchResultV1 {
    pub schema_version: u32,
    pub commit: AnimationTransferBatchCommitV1,
    pub compatibility: AnimationTransferCompatibilityV1,
    pub clips: Vec<AuthoredAnimationClipV1>,
    pub batch_fingerprint_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferBatchRequestV2 {
    pub schema_version: u32,
    pub commit: AnimationTransferBatchCommitV1,
    pub donor_source_revision: String,
    pub target_source_revision: String,
    pub clips: Vec<AnimationTransferBatchClipRequestV1>,
    pub semantic_map: Option<HumanoidSemanticBoneMapV2>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferBatchClipResultV2 {
    pub donor_clip_name: String,
    pub mode: AnimationTransferModeV1,
    pub status: String,
    pub clip: AuthoredAnimationClipV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferBatchResultV2 {
    pub schema_version: u32,
    pub commit: AnimationTransferBatchCommitV1,
    pub donor_source_revision: String,
    pub target_source_revision: String,
    pub semantic_map_fingerprint_sha256: Option<String>,
    pub clips: Vec<AnimationTransferBatchClipResultV2>,
    pub batch_fingerprint_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTransferErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for AnimationTransferErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for AnimationTransferErrorV1 {}

pub fn rig_signature_sha256_v1(rig: &AnimationStudioRigV1) -> String {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SignatureNode<'a> {
        name: &'a str,
        parent_name: Option<&'a str>,
        translation: [f32; 3],
        rotation: [f32; 4],
    }

    let by_id = rig
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = rig
        .nodes
        .iter()
        .map(|node| SignatureNode {
            name: &node.name,
            parent_name: node
                .parent_id
                .and_then(|parent_id| by_id.get(&parent_id).map(|parent| parent.name.as_str())),
            translation: canonical_vec3(node.translation),
            rotation: canonical_quaternion(node.rotation).unwrap_or(node.rotation),
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.name.cmp(right.name));
    fingerprint_json(&(rig.animation_root.as_str(), nodes))
}

pub fn build_semantic_rig_mapping_v1(
    target: &AnimationStudioRigV1,
    donor: &AnimationStudioRigV1,
) -> Result<SemanticRigMappingV1, Vec<AnimationTransferDiagnosticV1>> {
    let mut diagnostics = Vec::new();
    validate_rig_shape("targetRig", target, &mut diagnostics);
    validate_rig_shape("donorRig", donor, &mut diagnostics);

    let target_by_name = unique_nodes_by_name("targetRig", target, &mut diagnostics);
    let donor_by_name = unique_nodes_by_name("donorRig", donor, &mut diagnostics);
    let target_by_id = nodes_by_id(target);
    let donor_by_id = nodes_by_id(donor);

    let target_names = target_by_name.keys().collect::<BTreeSet<_>>();
    let donor_names = donor_by_name.keys().collect::<BTreeSet<_>>();
    for name in target_names.difference(&donor_names) {
        diagnostics.push(transfer_diagnostic(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            format!("donorRig.nodes[{name}]"),
            format!("donor rig is missing target bone {name:?}"),
        ));
    }
    for name in donor_names.difference(&target_names) {
        diagnostics.push(transfer_diagnostic(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            format!("donorRig.nodes[{name}]"),
            format!("donor-only bone {name:?} has no target mapping"),
        ));
    }

    let target_root = root_name(target);
    let donor_root = root_name(donor);
    if target_root != donor_root || target.animation_root != donor.animation_root {
        diagnostics.push(transfer_diagnostic(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rig.animationRoot",
            format!(
                "root differs: target {:?}/{:?}, donor {:?}/{:?}",
                target_root, target.animation_root, donor_root, donor.animation_root
            ),
        ));
    }

    let mut entries = Vec::new();
    for (name, target_node) in &target_by_name {
        let Some(donor_node) = donor_by_name.get(name) else {
            continue;
        };
        let target_parent = parent_name(target_node, &target_by_id);
        let donor_parent = parent_name(donor_node, &donor_by_id);
        if target_parent != donor_parent {
            diagnostics.push(transfer_diagnostic(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("rig.nodes[{name}].parent"),
                format!(
                    "parent differs for {name:?}: target {target_parent:?}, donor {donor_parent:?}"
                ),
            ));
            continue;
        }
        entries.push(SemanticBoneMappingEntryV1 {
            bone_name: (*name).clone(),
            parent_name: target_parent.map(str::to_owned),
            donor_node_id: donor_node.node_id,
            target_node_id: target_node.node_id,
        });
    }
    entries.sort_by(|left, right| left.bone_name.cmp(&right.bone_name));

    if diagnostics.is_empty() {
        Ok(SemanticRigMappingV1 {
            schema_version: 1,
            root_name: target_root.unwrap_or_default().to_owned(),
            entries,
        })
    } else {
        Err(diagnostics)
    }
}

pub fn inspect_humanoid_retarget_compatibility_v2(
    target: &AnimationStudioRigV1,
    donor: &AnimationStudioRigV1,
    overrides: &[HumanoidSemanticOverrideV2],
    manual_mapping_confirmed: bool,
) -> HumanoidSemanticBoneMapV2 {
    const ALIAS_VERSION: &str = "M2A_HUMANOID_ALIASES_2026_07_V1";
    let mut diagnostics = Vec::new();
    let mut entries = Vec::new();
    let mut incompatible = false;
    let mut manual_required = false;
    let mut override_semantics = BTreeSet::new();
    for (index, mapping) in overrides.iter().enumerate() {
        if !override_semantics.insert(mapping.semantic) {
            diagnostics.push(transfer_diagnostic(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("overrides[{index}].semantic"),
                "manual semantic overrides must be unique",
            ));
            incompatible = true;
        }
    }
    for semantic in humanoid_semantics_v2() {
        let required = semantic_required_v2(semantic);
        if let Some(mapping) = overrides
            .iter()
            .find(|mapping| mapping.semantic == semantic)
        {
            let donor_node = donor
                .nodes
                .iter()
                .find(|node| node.node_id == mapping.donor_node_id);
            let target_node = target
                .nodes
                .iter()
                .find(|node| node.node_id == mapping.target_node_id);
            match donor_node.zip(target_node) {
                Some((donor_node, target_node)) => {
                    entries.push(HumanoidSemanticBoneEntryV2 {
                        semantic,
                        required,
                        donor_node_id: donor_node.node_id,
                        donor_node_name: donor_node.name.clone(),
                        target_node_id: target_node.node_id,
                        target_node_name: target_node.name.clone(),
                        mapping_source: "MANUAL".to_owned(),
                    });
                    if !manual_mapping_confirmed {
                        manual_required = true;
                    }
                }
                None => {
                    diagnostics.push(transfer_diagnostic(
                        ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                        format!("overrides[{semantic:?}]"),
                        "manual semantic override targets a node absent from donor or target rig",
                    ));
                    incompatible = true;
                }
            }
            continue;
        }
        let donor_candidates = semantic_candidates_v2(donor, semantic);
        let target_candidates = semantic_candidates_v2(target, semantic);
        match (donor_candidates.as_slice(), target_candidates.as_slice()) {
            ([donor_node], [target_node]) => entries.push(HumanoidSemanticBoneEntryV2 {
                semantic,
                required,
                donor_node_id: donor_node.node_id,
                donor_node_name: donor_node.name.clone(),
                target_node_id: target_node.node_id,
                target_node_name: target_node.name.clone(),
                mapping_source: "VERSIONED_ALIAS".to_owned(),
            }),
            ([], _) | (_, []) if required => {
                diagnostics.push(transfer_diagnostic(
                    ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                    format!("semantics.{semantic:?}"),
                    "required humanoid semantic is missing from donor or target rig",
                ));
                incompatible = true;
            }
            ([], _) | (_, []) => {}
            _ => {
                diagnostics.push(transfer_diagnostic(
                    ANIMATION_TRANSFER_MODE_BLOCKED,
                    format!("semantics.{semantic:?}"),
                    "semantic alias is ambiguous and requires an explicit node mapping",
                ));
                manual_required = true;
            }
        }
    }
    let by_semantic = entries
        .iter()
        .map(|entry| (entry.semantic, entry))
        .collect::<BTreeMap<_, _>>();
    for (child, parent) in humanoid_chain_edges_v2() {
        let Some((child, parent)) = by_semantic.get(&child).zip(by_semantic.get(&parent)) else {
            continue;
        };
        if !is_descendant_v2(donor, child.donor_node_id, parent.donor_node_id)
            || !is_descendant_v2(target, child.target_node_id, parent.target_node_id)
        {
            diagnostics.push(transfer_diagnostic(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("semantics.{:?}.chain", child.semantic),
                format!(
                    "semantic {:?} is not below {:?} in both rigs",
                    child.semantic, parent.semantic
                ),
            ));
            incompatible = true;
        }
    }
    entries.sort_by_key(|entry| entry.semantic);
    let status = if incompatible {
        HumanoidRetargetCompatibilityStatusV2::Incompatible
    } else if manual_required {
        HumanoidRetargetCompatibilityStatusV2::ManualConfirmationRequired
    } else {
        HumanoidRetargetCompatibilityStatusV2::Compatible
    };
    let fingerprint_sha256 = fingerprint_json(&(
        ALIAS_VERSION,
        status,
        donor.source_revision.as_str(),
        target.source_revision.as_str(),
        manual_mapping_confirmed,
        &entries,
        &diagnostics,
    ));
    HumanoidSemanticBoneMapV2 {
        schema_version: 2,
        alias_dictionary_version: ALIAS_VERSION.to_owned(),
        status,
        donor_source_revision: donor.source_revision.clone(),
        target_source_revision: target.source_revision.clone(),
        manual_mapping_confirmed,
        entries,
        diagnostics,
        fingerprint_sha256,
    }
}

pub fn retarget_animation_clip_humanoid_v2(
    donor_clip: &MdlAnimationClipV1,
    donor_rig: &AnimationStudioRigV1,
    target_rig: &AnimationStudioRigV1,
    semantic_map: &HumanoidSemanticBoneMapV2,
    clip_id: &str,
    output_name: &str,
) -> Result<AnimationTransferResultV1, AnimationTransferErrorV1> {
    if semantic_map.schema_version != 2
        || semantic_map.status != HumanoidRetargetCompatibilityStatusV2::Compatible
        || semantic_map.donor_source_revision != donor_rig.source_revision
        || semantic_map.target_source_revision != target_rig.source_revision
    {
        return Err(transfer_error(
            ANIMATION_TRANSFER_MODE_BLOCKED,
            "semanticMap",
            "semantic V2 map is incompatible, unconfirmed or stale for these exact rigs",
        ));
    }
    let donor_by_id = nodes_by_id(donor_rig);
    let target_by_id = nodes_by_id(target_rig);
    let mut seen_donor = BTreeSet::new();
    let mut mapping_entries = Vec::new();
    for entry in &semantic_map.entries {
        if !seen_donor.insert(entry.donor_node_id) {
            continue;
        }
        let donor_node = donor_by_id.get(&entry.donor_node_id).ok_or_else(|| {
            transfer_error(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                "semanticMap.entries.donorNodeId",
                "semantic donor node is absent from the exact donor rig",
            )
        })?;
        let target_node = target_by_id.get(&entry.target_node_id).ok_or_else(|| {
            transfer_error(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                "semanticMap.entries.targetNodeId",
                "semantic target node is absent from the exact target rig",
            )
        })?;
        mapping_entries.push(SemanticBoneMappingEntryV1 {
            bone_name: format!("{:?}", entry.semantic),
            parent_name: target_node.parent_id.and_then(|parent_id| {
                target_by_id
                    .get(&parent_id)
                    .map(|parent| parent.name.clone())
            }),
            donor_node_id: donor_node.node_id,
            target_node_id: target_node.node_id,
        });
    }
    mapping_entries.sort_by_key(|entry| entry.donor_node_id);
    let mapping = SemanticRigMappingV1 {
        schema_version: 1,
        root_name: target_rig.animation_root.clone(),
        entries: mapping_entries,
    };
    let compatibility = AnimationTransferCompatibilityV1 {
        schema_version: 1,
        status: AnimationTransferCompatibilityStatusV1::RetargetableSameHierarchy,
        donor_source_revision: donor_rig.source_revision.clone(),
        target_source_revision: target_rig.source_revision.clone(),
        donor_rig_signature_sha256: rig_signature_sha256_v1(donor_rig),
        target_rig_signature_sha256: rig_signature_sha256_v1(target_rig),
        compatibility_fingerprint_sha256: semantic_map.fingerprint_sha256.clone(),
        allowed_modes: vec![AnimationTransferModeV1::HumanoidSemanticRetargetV2],
        mapping: Some(mapping.clone()),
        diagnostics: semantic_map.diagnostics.clone(),
    };
    let options = AnimationTransferOptionsV1 {
        mode: AnimationTransferModeV1::HumanoidSemanticRetargetV2,
        clip_id: clip_id.to_owned(),
        output_name: output_name.to_owned(),
    };
    let (clip, _) = transfer_clip(
        donor_clip,
        donor_rig,
        target_rig,
        &mapping,
        &compatibility,
        &options,
    )?;
    if let Some(diagnostic) = validate_authored_animation_clip_v1(&clip, target_rig).first() {
        return Err(transfer_error(
            ANIMATION_TRANSFER_OUTPUT_INVALID,
            diagnostic.path.clone(),
            diagnostic.message.clone(),
        ));
    }
    Ok(AnimationTransferResultV1 {
        schema_version: 1,
        required_document_schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3,
        compatibility,
        clip,
    })
}

pub fn animation_retarget_compatibility_fingerprint_v1(
    status: AnimationTransferCompatibilityStatusV1,
    donor_rig_signature_sha256: &str,
    target_rig_signature_sha256: &str,
    mapping: Option<&SemanticRigMappingV1>,
    diagnostics: &[AnimationTransferDiagnosticV1],
) -> String {
    fingerprint_json(&(
        status,
        donor_rig_signature_sha256,
        target_rig_signature_sha256,
        mapping,
        diagnostics,
    ))
}

pub fn inspect_animation_transfer_compatibility_v1(
    target: &AnimationStudioRigV1,
    donor: &AnimationStudioRigV1,
) -> AnimationTransferCompatibilityV1 {
    let donor_signature = rig_signature_sha256_v1(donor);
    let target_signature = rig_signature_sha256_v1(target);
    let (status, mapping, diagnostics) = match build_semantic_rig_mapping_v1(target, donor) {
        Ok(mapping) => {
            let mut differences = Vec::new();
            let exact = mapping.entries.iter().fold(true, |all_exact, entry| {
                let donor_node = donor
                    .nodes
                    .iter()
                    .find(|node| node.node_id == entry.donor_node_id)
                    .expect("mapping binds donor node");
                let target_node = target
                    .nodes
                    .iter()
                    .find(|node| node.node_id == entry.target_node_id)
                    .expect("mapping binds target node");
                let translation_matches = vec3_near(
                    donor_node.translation,
                    target_node.translation,
                    REST_EPSILON,
                );
                let rotation_matches =
                    quaternion_near(donor_node.rotation, target_node.rotation, REST_EPSILON);
                let translation_differs = donor_node
                    .translation
                    .iter()
                    .zip(target_node.translation)
                    .any(|(donor, target)| donor.to_bits() != target.to_bits());
                let rotation_differs = !rotation_matches;
                if donor_node.node_id != target_node.node_id {
                    differences.push(transfer_diagnostic(
                        "M2A-ANIMATION-RETARGET-NODE-ID",
                        format!("rig.nodes[{}].nodeId", entry.bone_name),
                        format!(
                            "node ID differs for {}: donor {}, target {}",
                            entry.bone_name, donor_node.node_id, target_node.node_id
                        ),
                    ));
                }
                if translation_differs {
                    differences.push(transfer_diagnostic(
                        "M2A-ANIMATION-RETARGET-REST-TRANSLATION",
                        format!("rig.nodes[{}].translation", entry.bone_name),
                        format!("rest translation differs for {}", entry.bone_name),
                    ));
                }
                if rotation_differs {
                    differences.push(transfer_diagnostic(
                        "M2A-ANIMATION-RETARGET-REST-ROTATION",
                        format!("rig.nodes[{}].rotation", entry.bone_name),
                        format!("rest rotation differs for {}", entry.bone_name),
                    ));
                }
                all_exact && translation_matches && rotation_matches
            });
            (
                if exact {
                    AnimationTransferCompatibilityStatusV1::ExactCopy
                } else {
                    AnimationTransferCompatibilityStatusV1::RetargetableSameHierarchy
                },
                Some(mapping),
                differences,
            )
        }
        Err(diagnostics) => (
            AnimationTransferCompatibilityStatusV1::Incompatible,
            None,
            diagnostics,
        ),
    };
    let allowed_modes = match status {
        AnimationTransferCompatibilityStatusV1::ExactCopy => vec![
            AnimationTransferModeV1::ExactRigCopyV1,
            AnimationTransferModeV1::SameHierarchyRetargetV1,
        ],
        AnimationTransferCompatibilityStatusV1::RetargetableSameHierarchy => {
            vec![AnimationTransferModeV1::SameHierarchyRetargetV1]
        }
        AnimationTransferCompatibilityStatusV1::Incompatible => Vec::new(),
    };
    let fingerprint = animation_retarget_compatibility_fingerprint_v1(
        status,
        &donor_signature,
        &target_signature,
        mapping.as_ref(),
        &diagnostics,
    );
    AnimationTransferCompatibilityV1 {
        schema_version: 1,
        status,
        donor_source_revision: donor.source_revision.clone(),
        target_source_revision: target.source_revision.clone(),
        donor_rig_signature_sha256: donor_signature,
        target_rig_signature_sha256: target_signature,
        compatibility_fingerprint_sha256: fingerprint,
        allowed_modes,
        mapping,
        diagnostics,
    }
}

pub fn inspect_animation_transfer_compatibility_between_models_v1(
    target_glb: &[u8],
    donor_glb: &[u8],
) -> Result<AnimationTransferCompatibilityV1, AnimationTransferErrorV1> {
    let target = inspect_editable_animation_source_v1(target_glb)
        .map_err(|error| pipeline_transfer_error("targetGlb", error.code, error.message))?;
    let donor = inspect_editable_animation_source_v1(donor_glb)
        .map_err(|error| pipeline_transfer_error("donorGlb", error.code, error.message))?;
    Ok(inspect_animation_transfer_compatibility_v1(
        &target.rig,
        &donor.rig,
    ))
}

pub fn inspect_humanoid_retarget_compatibility_between_models_v2(
    target_glb: &[u8],
    donor_glb: &[u8],
    overrides: &[HumanoidSemanticOverrideV2],
    manual_mapping_confirmed: bool,
) -> Result<HumanoidSemanticBoneMapV2, AnimationTransferErrorV1> {
    let target = inspect_editable_animation_source_v1(target_glb)
        .map_err(|error| pipeline_transfer_error("targetGlb", error.code, error.message))?;
    let donor = inspect_editable_animation_source_v1(donor_glb)
        .map_err(|error| pipeline_transfer_error("donorGlb", error.code, error.message))?;
    Ok(inspect_humanoid_retarget_compatibility_v2(
        &target.rig,
        &donor.rig,
        overrides,
        manual_mapping_confirmed,
    ))
}

pub fn retarget_animation_clip_between_models_humanoid_v2(
    target_glb: &[u8],
    donor_glb: &[u8],
    clip_name: &str,
    semantic_map: &HumanoidSemanticBoneMapV2,
    clip_id: &str,
    output_name: &str,
) -> Result<AnimationTransferResultV1, AnimationTransferErrorV1> {
    let target = inspect_editable_animation_source_v1(target_glb)
        .map_err(|error| pipeline_transfer_error("targetGlb", error.code, error.message))?;
    let donor = inspect_editable_animation_source_v1(donor_glb)
        .map_err(|error| pipeline_transfer_error("donorGlb", error.code, error.message))?;
    let donor_clip = donor
        .animations
        .clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            transfer_error(
                ANIMATION_TRANSFER_CLIP_MISSING,
                "clipName",
                format!("donor clip {clip_name:?} does not exist"),
            )
        })?;
    retarget_animation_clip_humanoid_v2(
        donor_clip,
        &donor.rig,
        &target.rig,
        semantic_map,
        clip_id,
        output_name,
    )
}

pub fn copy_animation_clip_between_models_v1(
    target_glb: &[u8],
    donor_glb: &[u8],
    clip_name: &str,
    options: &AnimationTransferOptionsV1,
) -> Result<AnimationTransferResultV1, AnimationTransferErrorV1> {
    let target = inspect_editable_animation_source_v1(target_glb)
        .map_err(|error| pipeline_transfer_error("targetGlb", error.code, error.message))?;
    let donor = inspect_editable_animation_source_v1(donor_glb)
        .map_err(|error| pipeline_transfer_error("donorGlb", error.code, error.message))?;
    let compatibility = inspect_animation_transfer_compatibility_v1(&target.rig, &donor.rig);
    if !compatibility.allowed_modes.contains(&options.mode) {
        return Err(transfer_error(
            ANIMATION_TRANSFER_MODE_BLOCKED,
            "options.mode",
            format!(
                "mode {} is not allowed for compatibility status {:?}",
                options.mode.as_str(),
                compatibility.status
            ),
        ));
    }
    let donor_clip = donor
        .animations
        .clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case(clip_name))
        .ok_or_else(|| {
            transfer_error(
                ANIMATION_TRANSFER_CLIP_MISSING,
                "clipName",
                format!("donor clip {clip_name:?} does not exist"),
            )
        })?;
    let mapping = compatibility.mapping.as_ref().ok_or_else(|| {
        transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "compatibility.mapping",
            "incompatible rigs have no semantic mapping",
        )
    })?;
    let (mut clip, root_motion_scale) = transfer_clip(
        donor_clip,
        &donor.rig,
        &target.rig,
        mapping,
        &compatibility,
        options,
    )?;
    let diagnostics = validate_authored_animation_clip_v1(&clip, &target.rig);
    if let Some(diagnostic) = diagnostics.first() {
        return Err(transfer_error(
            ANIMATION_TRANSFER_OUTPUT_INVALID,
            &diagnostic.path,
            &diagnostic.message,
        ));
    }
    clip.status = AuthoredAnimationClipStatusV1::Draft;
    Ok(AnimationTransferResultV1 {
        schema_version: 1,
        required_document_schema_version: if options.mode != AnimationTransferModeV1::ExactRigCopyV1
        {
            ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3
        } else {
            ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION
        },
        compatibility,
        clip: {
            debug_assert!(root_motion_scale.is_finite());
            clip
        },
    })
}

pub fn prepare_animation_transfer_batch_v1(
    target_glb: &[u8],
    donor_glb: &[u8],
    request: &AnimationTransferBatchRequestV1,
) -> Result<AnimationTransferBatchResultV1, AnimationTransferErrorV1> {
    if request.schema_version != 1 || request.clips.is_empty() || request.clips.len() > 256 {
        return Err(transfer_error(
            ANIMATION_TRANSFER_LIMIT,
            "batch.clips",
            "batch schema must be V1 and contain 1..=256 clips",
        ));
    }
    let target = inspect_editable_animation_source_v1(target_glb)
        .map_err(|error| pipeline_transfer_error("targetGlb", error.code, error.message))?;
    let donor = inspect_editable_animation_source_v1(donor_glb)
        .map_err(|error| pipeline_transfer_error("donorGlb", error.code, error.message))?;
    if request.target_source_revision != target.source_revision
        || request.donor_source_revision != donor.source_revision
    {
        return Err(transfer_error(
            ANIMATION_TRANSFER_MODE_BLOCKED,
            "batch.sourceRevision",
            "batch source revisions do not match the exact donor and target GLBs",
        ));
    }
    let mut clip_ids = BTreeSet::new();
    let mut output_names = BTreeSet::new();
    for (index, clip) in request.clips.iter().enumerate() {
        if !clip_ids.insert(clip.clip_id.as_str()) {
            return Err(transfer_error(
                ANIMATION_TRANSFER_OUTPUT_INVALID,
                format!("batch.clips[{index}].clipId"),
                "batch clip IDs must be unique",
            ));
        }
        if !output_names.insert(clip.output_name.to_ascii_lowercase()) {
            return Err(transfer_error(
                ANIMATION_TRANSFER_OUTPUT_INVALID,
                format!("batch.clips[{index}].outputName"),
                "batch output names must be unique case-insensitively",
            ));
        }
    }
    let compatibility = inspect_animation_transfer_compatibility_v1(&target.rig, &donor.rig);
    let mapping = compatibility.mapping.as_ref().ok_or_else(|| {
        transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "compatibility.mapping",
            "incompatible rigs have no semantic mapping",
        )
    })?;
    let mut clips = Vec::with_capacity(request.clips.len());
    for (index, requested) in request.clips.iter().enumerate() {
        if !compatibility.allowed_modes.contains(&requested.mode) {
            return Err(transfer_error(
                ANIMATION_TRANSFER_MODE_BLOCKED,
                format!("batch.clips[{index}].mode"),
                format!("mode {} is not allowed", requested.mode.as_str()),
            ));
        }
        let donor_clip = donor
            .animations
            .clips
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(&requested.donor_clip_name))
            .ok_or_else(|| {
                transfer_error(
                    ANIMATION_TRANSFER_CLIP_MISSING,
                    format!("batch.clips[{index}].donorClipName"),
                    format!("donor clip {:?} does not exist", requested.donor_clip_name),
                )
            })?;
        let options = AnimationTransferOptionsV1 {
            mode: requested.mode,
            clip_id: requested.clip_id.clone(),
            output_name: requested.output_name.clone(),
        };
        let (mut clip, _) = transfer_clip(
            donor_clip,
            &donor.rig,
            &target.rig,
            mapping,
            &compatibility,
            &options,
        )?;
        if let Some(diagnostic) = validate_authored_animation_clip_v1(&clip, &target.rig).first() {
            return Err(transfer_error(
                ANIMATION_TRANSFER_OUTPUT_INVALID,
                format!("batch.clips[{index}].{}", diagnostic.path),
                diagnostic.message.clone(),
            ));
        }
        clip.status = AuthoredAnimationClipStatusV1::Draft;
        clips.push(clip);
    }
    let batch_fingerprint_sha256 = fingerprint_json(&(request, &compatibility, &clips));
    Ok(AnimationTransferBatchResultV1 {
        schema_version: 1,
        commit: request.commit,
        compatibility,
        clips,
        batch_fingerprint_sha256,
    })
}

/// Transactional batch seam shared by exact, same-hierarchy and semantic V2
/// transfers. Results are only returned after every requested clip succeeds;
/// callers therefore have one atomic commit point and never receive a partial
/// document mutation.
pub fn prepare_animation_transfer_batch_v2(
    target_glb: &[u8],
    donor_glb: &[u8],
    request: &AnimationTransferBatchRequestV2,
) -> Result<AnimationTransferBatchResultV2, AnimationTransferErrorV1> {
    if request.schema_version != 2 || request.clips.is_empty() || request.clips.len() > 256 {
        return Err(transfer_error(
            ANIMATION_TRANSFER_LIMIT,
            "batch.clips",
            "batch schema must be V2 and contain 1..=256 clips",
        ));
    }
    let target = inspect_editable_animation_source_v1(target_glb)
        .map_err(|error| pipeline_transfer_error("targetGlb", error.code, error.message))?;
    let donor = inspect_editable_animation_source_v1(donor_glb)
        .map_err(|error| pipeline_transfer_error("donorGlb", error.code, error.message))?;
    if request.target_source_revision != target.source_revision
        || request.donor_source_revision != donor.source_revision
    {
        return Err(transfer_error(
            ANIMATION_TRANSFER_MODE_BLOCKED,
            "batch.sourceRevision",
            "batch V2 source revisions do not match the exact donor and target GLBs",
        ));
    }
    let semantic_requested = request
        .clips
        .iter()
        .any(|clip| clip.mode == AnimationTransferModeV1::HumanoidSemanticRetargetV2);
    if semantic_requested {
        let semantic_map = request.semantic_map.as_ref().ok_or_else(|| {
            transfer_error(
                ANIMATION_TRANSFER_MODE_BLOCKED,
                "batch.semanticMap",
                "semantic V2 batch clips require one confirmed map",
            )
        })?;
        if semantic_map.status != HumanoidRetargetCompatibilityStatusV2::Compatible
            || !semantic_map.manual_mapping_confirmed
            || semantic_map.donor_source_revision != donor.source_revision
            || semantic_map.target_source_revision != target.source_revision
        {
            return Err(transfer_error(
                ANIMATION_TRANSFER_MODE_BLOCKED,
                "batch.semanticMap",
                "semantic map is incomplete, unconfirmed or stale for the exact rig pair",
            ));
        }
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, clip) in request.clips.iter().enumerate() {
        if clip.donor_clip_name.trim().is_empty()
            || clip.clip_id.trim().is_empty()
            || clip.output_name.trim().is_empty()
            || !ids.insert(clip.clip_id.as_str())
            || !names.insert(clip.output_name.to_ascii_lowercase())
        {
            return Err(transfer_error(
                ANIMATION_TRANSFER_OUTPUT_INVALID,
                format!("batch.clips[{index}]"),
                "batch clip source/name/id must be non-empty and output identities unique",
            ));
        }
    }
    let mut clips = Vec::with_capacity(request.clips.len());
    for (index, requested) in request.clips.iter().enumerate() {
        let result = if requested.mode == AnimationTransferModeV1::HumanoidSemanticRetargetV2 {
            retarget_animation_clip_between_models_humanoid_v2(
                target_glb,
                donor_glb,
                &requested.donor_clip_name,
                request
                    .semantic_map
                    .as_ref()
                    .expect("semantic request validated"),
                &requested.clip_id,
                &requested.output_name,
            )
        } else {
            copy_animation_clip_between_models_v1(
                target_glb,
                donor_glb,
                &requested.donor_clip_name,
                &AnimationTransferOptionsV1 {
                    mode: requested.mode,
                    clip_id: requested.clip_id.clone(),
                    output_name: requested.output_name.clone(),
                },
            )
        }
        .map_err(|error| AnimationTransferErrorV1 {
            path: format!("batch.clips[{index}].{}", error.path),
            ..error
        })?;
        clips.push(AnimationTransferBatchClipResultV2 {
            donor_clip_name: requested.donor_clip_name.clone(),
            mode: requested.mode,
            status: "READY".into(),
            clip: result.clip,
        });
    }
    let semantic_map_fingerprint_sha256 = request
        .semantic_map
        .as_ref()
        .map(|map| map.fingerprint_sha256.clone());
    let batch_fingerprint_sha256 =
        fingerprint_json(&(request, &clips, semantic_map_fingerprint_sha256.as_deref()));
    Ok(AnimationTransferBatchResultV2 {
        schema_version: 2,
        commit: request.commit,
        donor_source_revision: donor.source_revision,
        target_source_revision: target.source_revision,
        semantic_map_fingerprint_sha256,
        clips,
        batch_fingerprint_sha256,
    })
}

/// Retargets an already inspected donor clip onto a same-name/same-parent
/// target hierarchy. This is the deterministic Core seam used by unit tests,
/// native callers and the model-to-model high-level boundary.
pub fn retarget_animation_clip_same_hierarchy_v1(
    donor_clip: &MdlAnimationClipV1,
    donor_rig: &AnimationStudioRigV1,
    target_rig: &AnimationStudioRigV1,
    clip_id: &str,
    output_name: &str,
) -> Result<AnimationTransferResultV1, AnimationTransferErrorV1> {
    let compatibility = inspect_animation_transfer_compatibility_v1(target_rig, donor_rig);
    let mode = AnimationTransferModeV1::SameHierarchyRetargetV1;
    if !compatibility.allowed_modes.contains(&mode) {
        return Err(transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "compatibility",
            "rigs are not compatible with same-hierarchy retargeting",
        ));
    }
    let mapping = compatibility
        .mapping
        .as_ref()
        .expect("allowed mode has mapping");
    let options = AnimationTransferOptionsV1 {
        mode,
        clip_id: clip_id.to_owned(),
        output_name: output_name.to_owned(),
    };
    let (clip, _) = transfer_clip(
        donor_clip,
        donor_rig,
        target_rig,
        mapping,
        &compatibility,
        &options,
    )?;
    let diagnostics = validate_authored_animation_clip_v1(&clip, target_rig);
    if let Some(diagnostic) = diagnostics.first() {
        return Err(transfer_error(
            ANIMATION_TRANSFER_OUTPUT_INVALID,
            &diagnostic.path,
            &diagnostic.message,
        ));
    }
    Ok(AnimationTransferResultV1 {
        schema_version: 1,
        required_document_schema_version: ANIMATION_STUDIO_DOCUMENT_SCHEMA_VERSION_V3,
        compatibility,
        clip,
    })
}

fn transfer_clip(
    donor_clip: &MdlAnimationClipV1,
    donor_rig: &AnimationStudioRigV1,
    target_rig: &AnimationStudioRigV1,
    mapping: &SemanticRigMappingV1,
    compatibility: &AnimationTransferCompatibilityV1,
    options: &AnimationTransferOptionsV1,
) -> Result<(AuthoredAnimationClipV1, f32), AnimationTransferErrorV1> {
    let source_keyframe_count = donor_clip
        .tracks
        .iter()
        .try_fold(0_usize, |count, track| {
            count.checked_add(track.times_seconds.len())
        })
        .ok_or_else(|| {
            transfer_error(
                ANIMATION_TRANSFER_LIMIT,
                "donorClip.tracks",
                "source keyframe count overflowed the platform counter",
            )
        })?;
    if source_keyframe_count > ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP {
        return Err(transfer_error(
            ANIMATION_TRANSFER_LIMIT,
            "donorClip.tracks",
            format!(
                "source clip has {source_keyframe_count} keyframes; product limit is {ANIMATION_STUDIO_MAX_KEYFRAMES_PER_CLIP}"
            ),
        ));
    }
    let donor_by_id = nodes_by_id(donor_rig);
    let target_by_id = nodes_by_id(target_rig);
    let mapping_by_donor_id = mapping
        .entries
        .iter()
        .map(|entry| (entry.donor_node_id, entry))
        .collect::<BTreeMap<_, _>>();
    let root_motion_scale = if options.mode != AnimationTransferModeV1::ExactRigCopyV1 {
        skeleton_height_v1(target_rig)? / skeleton_height_v1(donor_rig)?
    } else {
        1.0
    };
    if !root_motion_scale.is_finite() || root_motion_scale <= 0.0 {
        return Err(transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rig.skeletonHeight",
            "root-motion height ratio is not a finite positive value",
        ));
    }

    let mut tracks = Vec::new();
    for donor_track in &donor_clip.tracks {
        if donor_track.times_seconds.len() != donor_track.values.len() {
            return Err(transfer_error(
                ANIMATION_TRANSFER_TRACK_UNSUPPORTED,
                "donorClip.tracks",
                "track time/value row counts differ",
            ));
        }
        if donor_track.interpolation != MdlAnimationInterpolationV1::Linear {
            return Err(transfer_error(
                ANIMATION_TRANSFER_TRACK_UNSUPPORTED,
                "donorClip.tracks.interpolation",
                "retargeting currently accepts LINEAR tracks only",
            ));
        }
        let Some(entry) = mapping_by_donor_id.get(&donor_track.target_node_id) else {
            if options.mode == AnimationTransferModeV1::HumanoidSemanticRetargetV2 {
                continue;
            }
            return Err(transfer_error(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                "donorClip.tracks.targetNodeId",
                format!(
                    "donor track node {} has no semantic target",
                    donor_track.target_node_id
                ),
            ));
        };
        let donor_node = donor_by_id[&entry.donor_node_id];
        let target_node = target_by_id[&entry.target_node_id];
        let path = match donor_track.path {
            MdlAnimationTrackPathV1::Translation => AuthoredAnimationTrackPathV1::Translation,
            MdlAnimationTrackPathV1::Rotation => AuthoredAnimationTrackPathV1::Rotation,
            MdlAnimationTrackPathV1::Scale if constant_scale_track(donor_track) => continue,
            MdlAnimationTrackPathV1::Scale | MdlAnimationTrackPathV1::Weights => {
                return Err(transfer_error(
                    ANIMATION_TRANSFER_TRACK_UNSUPPORTED,
                    "donorClip.tracks.path",
                    "changing SCALE and WEIGHTS tracks cannot be transferred",
                ));
            }
        };
        let values = match (options.mode, path) {
            (AnimationTransferModeV1::ExactRigCopyV1, _) => donor_track.values.clone(),
            (
                AnimationTransferModeV1::SameHierarchyRetargetV1
                | AnimationTransferModeV1::HumanoidSemanticRetargetV2,
                AuthoredAnimationTrackPathV1::Translation,
            ) => retarget_translation_values(
                donor_track,
                donor_node,
                target_node,
                entry.parent_name.is_none(),
                root_motion_scale,
            )?,
            (
                AnimationTransferModeV1::SameHierarchyRetargetV1
                | AnimationTransferModeV1::HumanoidSemanticRetargetV2,
                AuthoredAnimationTrackPathV1::Rotation,
            ) => retarget_rotation_values(donor_track, donor_node, target_node)?,
        };
        let suffix = match path {
            AuthoredAnimationTrackPathV1::Translation => "translation",
            AuthoredAnimationTrackPathV1::Rotation => "rotation",
        };
        tracks.push(AuthoredAnimationTrackV1 {
            id: format!("track-{:04}-{suffix}", target_node.node_id),
            target_node_id: target_node.node_id,
            path,
            interpolation: donor_track.interpolation,
            keyframes: donor_track
                .times_seconds
                .iter()
                .copied()
                .zip(values)
                .enumerate()
                .map(|(index, (time_seconds, value))| AnimationKeyframeV1 {
                    id: format!("key-{:04}-{suffix}-{index:04}", target_node.node_id),
                    time_seconds,
                    value,
                })
                .collect(),
        });
    }
    tracks.sort_by_key(|track| (track.target_node_id, track.path));
    let events = donor_clip
        .events
        .iter()
        .enumerate()
        .map(|(index, event)| AuthoredAnimationEventV1 {
            id: format!("event-{index:04}"),
            time_seconds: event.time_seconds,
            name: event.name.clone(),
        })
        .collect::<Vec<_>>();
    let donor_clip_fingerprint = fingerprint_json(donor_clip);
    let output_motion_fingerprint = fingerprint_json(&(
        donor_clip.length_seconds,
        donor_clip.transition_seconds,
        target_rig.animation_root.as_str(),
        &tracks,
        &events,
    ));
    let is_retarget = options.mode != AnimationTransferModeV1::ExactRigCopyV1;
    let source = AuthoredAnimationSourceV1 {
        kind: if is_retarget {
            AuthoredAnimationSourceKindV1::RetargetedModelCopy
        } else {
            AuthoredAnimationSourceKindV1::ImportedModelCopy
        },
        source_revision: donor_rig.source_revision.clone(),
        source_clip_name: Some(donor_clip.name.clone()),
        source_clip_fingerprint: Some(donor_clip_fingerprint.clone()),
        procedural_template: None,
        library_preset: None,
        retarget: is_retarget.then(|| AnimationRetargetProvenanceV1 {
            donor_source_revision: donor_rig.source_revision.clone(),
            target_source_revision: target_rig.source_revision.clone(),
            donor_clip_name: donor_clip.name.clone(),
            donor_clip_fingerprint,
            donor_rig_signature_sha256: compatibility.donor_rig_signature_sha256.clone(),
            target_rig_signature_sha256: compatibility.target_rig_signature_sha256.clone(),
            compatibility_fingerprint_sha256: compatibility
                .compatibility_fingerprint_sha256
                .clone(),
            mode: options.mode.as_str().to_owned(),
            root_motion_scale,
            output_motion_fingerprint_sha256: output_motion_fingerprint,
            algorithm_version: match options.mode {
                AnimationTransferModeV1::HumanoidSemanticRetargetV2 => {
                    ANIMATION_RETARGET_ALGORITHM_V2
                }
                _ => ANIMATION_RETARGET_ALGORITHM_V1,
            }
            .to_owned(),
            algorithm_limits: match options.mode {
                AnimationTransferModeV1::HumanoidSemanticRetargetV2 => {
                    "VERSIONED_ALIASES|REQUIRED_SEMANTICS|MANUAL_CONFIRMATION|TR_ONLY|LINEAR"
                }
                _ => ANIMATION_RETARGET_LIMITS_V1,
            }
            .to_owned(),
        }),
    };
    Ok((
        AuthoredAnimationClipV1 {
            id: options.clip_id.clone(),
            name: options.output_name.clone(),
            kind: AuthoredAnimationClipKindV1::Motion,
            status: AuthoredAnimationClipStatusV1::Draft,
            source,
            length_seconds: donor_clip.length_seconds,
            transition_seconds: donor_clip.transition_seconds,
            animation_root: target_rig.animation_root.clone(),
            tracks,
            events,
            revision: 1,
        },
        root_motion_scale,
    ))
}

fn retarget_translation_values(
    track: &MdlAnimationTrackV1,
    donor_node: &AnimationStudioRigNodeV1,
    target_node: &AnimationStudioRigNodeV1,
    is_root: bool,
    root_motion_scale: f32,
) -> Result<Vec<Vec<f32>>, AnimationTransferErrorV1> {
    if track
        .values
        .iter()
        .any(|row| row.len() != 3 || row.iter().any(|value| !value.is_finite()))
    {
        return Err(transfer_error(
            ANIMATION_TRANSFER_TRACK_UNSUPPORTED,
            "donorClip.tracks.translation",
            "translation rows must contain three finite values",
        ));
    }
    if track_is_constant(&track.values) {
        return Ok(vec![target_node.translation.to_vec(); track.values.len()]);
    }
    let scale = if is_root {
        root_motion_scale
    } else {
        let donor_length = vector_length(donor_node.translation);
        let target_length = vector_length(target_node.translation);
        if donor_length <= MOTION_EPSILON || !donor_length.is_finite() || !target_length.is_finite()
        {
            return Err(transfer_error(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("donorRig.nodes[{}].translation", donor_node.name),
                "a varying child translation requires a non-zero finite donor bone length",
            ));
        }
        target_length / donor_length
    };
    Ok(track
        .values
        .iter()
        .map(|row| {
            (0..3)
                .map(|axis| {
                    target_node.translation[axis]
                        + (row[axis] - donor_node.translation[axis]) * scale
                })
                .collect()
        })
        .collect())
}

fn retarget_rotation_values(
    track: &MdlAnimationTrackV1,
    donor_node: &AnimationStudioRigNodeV1,
    target_node: &AnimationStudioRigNodeV1,
) -> Result<Vec<Vec<f32>>, AnimationTransferErrorV1> {
    let donor_rest = canonical_quaternion(donor_node.rotation)?;
    let target_rest = canonical_quaternion(target_node.rotation)?;
    let inverse_donor_rest = quaternion_inverse(donor_rest);
    track
        .values
        .iter()
        .map(|row| {
            let source_animation = canonical_quaternion_slice(row)?;
            let delta = quaternion_multiply(inverse_donor_rest, source_animation);
            Ok(canonical_quaternion(quaternion_multiply(target_rest, delta))?.to_vec())
        })
        .collect()
}

fn validate_rig_shape(
    path: &str,
    rig: &AnimationStudioRigV1,
    diagnostics: &mut Vec<AnimationTransferDiagnosticV1>,
) {
    if rig.nodes.is_empty() {
        diagnostics.push(transfer_diagnostic(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            format!("{path}.nodes"),
            "rig contains no nodes",
        ));
    }
    let by_id = nodes_by_id(rig);
    if by_id.len() != rig.nodes.len() {
        diagnostics.push(transfer_diagnostic(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            format!("{path}.nodes"),
            "node IDs must be unique inside each rig",
        ));
    }
    let roots = rig
        .nodes
        .iter()
        .filter(|node| node.parent_id.is_none())
        .count();
    if roots != 1 {
        diagnostics.push(transfer_diagnostic(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            format!("{path}.nodes"),
            format!("rig must have exactly one root, got {roots}"),
        ));
    }
    for node in &rig.nodes {
        if node.name.trim().is_empty()
            || node.translation.iter().any(|value| !value.is_finite())
            || canonical_quaternion(node.rotation).is_err()
        {
            diagnostics.push(transfer_diagnostic(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("{path}.nodes[{}]", node.node_id),
                "node name and rest TR values must be finite and valid",
            ));
        }
        if node
            .parent_id
            .is_some_and(|parent_id| !by_id.contains_key(&parent_id))
        {
            diagnostics.push(transfer_diagnostic(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("{path}.nodes[{}].parentId", node.node_id),
                "parent ID does not exist in the same rig",
            ));
        }
        let mut seen = BTreeSet::new();
        let mut cursor = Some(node.node_id);
        while let Some(node_id) = cursor {
            if !seen.insert(node_id) {
                diagnostics.push(transfer_diagnostic(
                    ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                    format!("{path}.nodes[{}].parentId", node.node_id),
                    "parent graph contains a cycle",
                ));
                break;
            }
            cursor = by_id
                .get(&node_id)
                .and_then(|candidate| candidate.parent_id);
        }
    }
}

fn unique_nodes_by_name<'a>(
    path: &str,
    rig: &'a AnimationStudioRigV1,
    diagnostics: &mut Vec<AnimationTransferDiagnosticV1>,
) -> BTreeMap<String, &'a AnimationStudioRigNodeV1> {
    let mut output = BTreeMap::new();
    for node in &rig.nodes {
        if output.insert(node.name.clone(), node).is_some() {
            diagnostics.push(transfer_diagnostic(
                ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
                format!("{path}.nodes[{}].name", node.node_id),
                format!("bone name {:?} is not unique", node.name),
            ));
        }
    }
    output
}

fn nodes_by_id(rig: &AnimationStudioRigV1) -> BTreeMap<u32, &AnimationStudioRigNodeV1> {
    rig.nodes.iter().map(|node| (node.node_id, node)).collect()
}

fn root_name(rig: &AnimationStudioRigV1) -> Option<&str> {
    let mut roots = rig.nodes.iter().filter(|node| node.parent_id.is_none());
    let first = roots.next()?;
    roots.next().is_none().then_some(first.name.as_str())
}

fn parent_name<'a>(
    node: &AnimationStudioRigNodeV1,
    by_id: &BTreeMap<u32, &'a AnimationStudioRigNodeV1>,
) -> Option<&'a str> {
    node.parent_id
        .and_then(|parent_id| by_id.get(&parent_id).map(|parent| parent.name.as_str()))
}

fn skeleton_height_v1(rig: &AnimationStudioRigV1) -> Result<f32, AnimationTransferErrorV1> {
    let by_id = nodes_by_id(rig);
    let mut memo = BTreeMap::new();
    let mut world_positions = Vec::with_capacity(rig.nodes.len());
    for node in &rig.nodes {
        let (position, _) = world_transform(node.node_id, &by_id, &mut memo, &mut BTreeSet::new())?;
        world_positions.push(position);
    }
    let min_y = world_positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = world_positions
        .iter()
        .map(|position| position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let vertical = max_y - min_y;
    if vertical.is_finite() && vertical > MOTION_EPSILON {
        return Ok(vertical);
    }
    let mut extent = 0.0_f32;
    for left in &world_positions {
        for right in &world_positions {
            extent = extent.max(vector_length([
                left[0] - right[0],
                left[1] - right[1],
                left[2] - right[2],
            ]));
        }
    }
    if extent.is_finite() && extent > MOTION_EPSILON {
        Ok(extent)
    } else {
        Err(transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rig.skeletonHeight",
            "rig has no finite non-zero rest-pose extent",
        ))
    }
}

type WorldTransform = ([f32; 3], [f32; 4]);

fn world_transform(
    node_id: u32,
    by_id: &BTreeMap<u32, &AnimationStudioRigNodeV1>,
    memo: &mut BTreeMap<u32, WorldTransform>,
    active: &mut BTreeSet<u32>,
) -> Result<WorldTransform, AnimationTransferErrorV1> {
    if let Some(value) = memo.get(&node_id) {
        return Ok(*value);
    }
    if !active.insert(node_id) {
        return Err(transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rig.nodes.parentId",
            "parent graph contains a cycle",
        ));
    }
    let node = by_id.get(&node_id).ok_or_else(|| {
        transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rig.nodes",
            format!("node ID {node_id} is missing"),
        )
    })?;
    let local_rotation = canonical_quaternion(node.rotation)?;
    let result = if let Some(parent_id) = node.parent_id {
        let (parent_position, parent_rotation) = world_transform(parent_id, by_id, memo, active)?;
        let rotated = quaternion_rotate(parent_rotation, node.translation);
        (
            [
                parent_position[0] + rotated[0],
                parent_position[1] + rotated[1],
                parent_position[2] + rotated[2],
            ],
            canonical_quaternion(quaternion_multiply(parent_rotation, local_rotation))?,
        )
    } else {
        (node.translation, local_rotation)
    };
    active.remove(&node_id);
    memo.insert(node_id, result);
    Ok(result)
}

fn constant_scale_track(track: &MdlAnimationTrackV1) -> bool {
    let Some(first) = track.values.first() else {
        return false;
    };
    (first.len() == 1 || first.len() == 3)
        && first.iter().all(|value| value.is_finite())
        && (first.len() == 1
            || first
                .iter()
                .all(|value| (*value - first[0]).abs() <= REST_EPSILON))
        && track.values.iter().all(|row| {
            row.len() == first.len()
                && row.iter().all(|value| value.is_finite())
                && row
                    .iter()
                    .zip(first)
                    .all(|(actual, expected)| (*actual - *expected).abs() <= REST_EPSILON)
        })
}

fn track_is_constant(values: &[Vec<f32>]) -> bool {
    let Some(first) = values.first() else {
        return true;
    };
    values.iter().all(|row| {
        row.len() == first.len()
            && row
                .iter()
                .zip(first)
                .all(|(actual, expected)| (*actual - *expected).abs() <= MOTION_EPSILON)
    })
}

fn canonical_vec3(mut value: [f32; 3]) -> [f32; 3] {
    for component in &mut value {
        if *component == 0.0 {
            *component = 0.0;
        }
    }
    value
}

fn humanoid_semantics_v2() -> [HumanoidBoneSemanticV2; 22] {
    use HumanoidBoneSemanticV2::*;
    [
        Root,
        Hips,
        Spine,
        Chest,
        Neck,
        Head,
        LeftClavicle,
        LeftUpperArm,
        LeftLowerArm,
        LeftHand,
        RightClavicle,
        RightUpperArm,
        RightLowerArm,
        RightHand,
        LeftUpperLeg,
        LeftLowerLeg,
        LeftFoot,
        LeftToe,
        RightUpperLeg,
        RightLowerLeg,
        RightFoot,
        RightToe,
    ]
}

fn semantic_required_v2(semantic: HumanoidBoneSemanticV2) -> bool {
    !matches!(
        semantic,
        HumanoidBoneSemanticV2::Root
            | HumanoidBoneSemanticV2::Chest
            | HumanoidBoneSemanticV2::Neck
            | HumanoidBoneSemanticV2::LeftClavicle
            | HumanoidBoneSemanticV2::RightClavicle
            | HumanoidBoneSemanticV2::LeftToe
            | HumanoidBoneSemanticV2::RightToe
    )
}

fn semantic_aliases_v2(semantic: HumanoidBoneSemanticV2) -> &'static [&'static str] {
    use HumanoidBoneSemanticV2::*;
    match semantic {
        Root => &["root", "armature", "scene"],
        Hips => &["hips", "pelvis", "bip01pelvis", "mixamorighips"],
        Spine => &["spine", "spine01", "bip01spine", "mixamorigspine"],
        Chest => &[
            "chest",
            "spine02",
            "spine2",
            "upperchest",
            "mixamorigspine2",
        ],
        Neck => &["neck", "neck01", "mixamorigneck"],
        Head => &["head", "head01", "bip01head", "mixamorighead"],
        LeftClavicle => &[
            "leftshoulder",
            "lclavicle",
            "claviclel",
            "mixamorigleftshoulder",
        ],
        LeftUpperArm => &["leftarm", "lupperarm", "upperarml", "mixamorigleftarm"],
        LeftLowerArm => &[
            "leftforearm",
            "llowerarm",
            "lowerarml",
            "mixamorigleftforearm",
        ],
        LeftHand => &["lefthand", "lhand", "handl", "mixamoriglefthand"],
        RightClavicle => &[
            "rightshoulder",
            "rclavicle",
            "clavicler",
            "mixamorigrightshoulder",
        ],
        RightUpperArm => &["rightarm", "rupperarm", "upperarmr", "mixamorigrightarm"],
        RightLowerArm => &[
            "rightforearm",
            "rlowerarm",
            "lowerarmr",
            "mixamorigrightforearm",
        ],
        RightHand => &["righthand", "rhand", "handr", "mixamorigrighthand"],
        LeftUpperLeg => &["leftupleg", "lthigh", "thighl", "mixamorigleftupleg"],
        LeftLowerLeg => &["leftleg", "lcalf", "calfl", "mixamorigleftleg"],
        LeftFoot => &["leftfoot", "lfoot", "footl", "mixamorigleftfoot"],
        LeftToe => &["lefttoebase", "ltoe", "toel", "mixamoriglefttoebase"],
        RightUpperLeg => &["rightupleg", "rthigh", "thighr", "mixamorigrightupleg"],
        RightLowerLeg => &["rightleg", "rcalf", "calfr", "mixamorigrightleg"],
        RightFoot => &["rightfoot", "rfoot", "footr", "mixamorigrightfoot"],
        RightToe => &["righttoebase", "rtoe", "toer", "mixamorigrighttoebase"],
    }
}

fn semantic_candidates_v2(
    rig: &AnimationStudioRigV1,
    semantic: HumanoidBoneSemanticV2,
) -> Vec<&AnimationStudioRigNodeV1> {
    let aliases = semantic_aliases_v2(semantic);
    rig.nodes
        .iter()
        .filter(|node| aliases.contains(&normalize_bone_name_v2(&node.name).as_str()))
        .collect()
}

fn normalize_bone_name_v2(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn humanoid_chain_edges_v2() -> [(HumanoidBoneSemanticV2, HumanoidBoneSemanticV2); 17] {
    use HumanoidBoneSemanticV2::*;
    [
        (Spine, Hips),
        (Chest, Spine),
        (Neck, Spine),
        (Head, Neck),
        (LeftClavicle, Chest),
        (LeftUpperArm, Spine),
        (LeftLowerArm, LeftUpperArm),
        (LeftHand, LeftLowerArm),
        (RightClavicle, Chest),
        (RightUpperArm, Spine),
        (RightLowerArm, RightUpperArm),
        (RightHand, RightLowerArm),
        (LeftUpperLeg, Hips),
        (LeftLowerLeg, LeftUpperLeg),
        (LeftFoot, LeftLowerLeg),
        (RightUpperLeg, Hips),
        (RightLowerLeg, RightUpperLeg),
    ]
}

fn is_descendant_v2(rig: &AnimationStudioRigV1, child_id: u32, ancestor_id: u32) -> bool {
    if child_id == ancestor_id {
        return false;
    }
    let by_id = nodes_by_id(rig);
    let mut cursor = by_id.get(&child_id).and_then(|node| node.parent_id);
    let mut visited = BTreeSet::new();
    while let Some(node_id) = cursor {
        if node_id == ancestor_id {
            return true;
        }
        if !visited.insert(node_id) {
            return false;
        }
        cursor = by_id.get(&node_id).and_then(|node| node.parent_id);
    }
    false
}

fn canonical_quaternion_slice(value: &[f32]) -> Result<[f32; 4], AnimationTransferErrorV1> {
    let value: [f32; 4] = value.try_into().map_err(|_| {
        transfer_error(
            ANIMATION_TRANSFER_TRACK_UNSUPPORTED,
            "rotation",
            "rotation row must contain four values",
        )
    })?;
    canonical_quaternion(value)
}

fn canonical_quaternion(mut value: [f32; 4]) -> Result<[f32; 4], AnimationTransferErrorV1> {
    if value.iter().any(|component| !component.is_finite()) {
        return Err(transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rotation",
            "quaternion contains a non-finite value",
        ));
    }
    let norm = value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= MOTION_EPSILON {
        return Err(transfer_error(
            ANIMATION_TRANSFER_RIG_INCOMPATIBLE,
            "rotation",
            "quaternion has a zero or non-finite norm",
        ));
    }
    for component in &mut value {
        *component /= norm;
    }
    let flip = value[3] < 0.0
        || (value[3] == 0.0
            && value[..3]
                .iter()
                .find(|component| **component != 0.0)
                .is_some_and(|component| *component < 0.0));
    if flip {
        for component in &mut value {
            *component = -*component;
        }
    }
    for component in &mut value {
        if *component == 0.0 {
            *component = 0.0;
        }
    }
    Ok(value)
}

fn quaternion_near(left: [f32; 4], right: [f32; 4], epsilon: f32) -> bool {
    let direct = left
        .iter()
        .zip(right)
        .all(|(left, right)| (*left - right).abs() <= epsilon);
    let opposite = left
        .iter()
        .zip(right)
        .all(|(left, right)| (*left + right).abs() <= epsilon);
    direct || opposite
}

fn vec3_near(left: [f32; 3], right: [f32; 3], epsilon: f32) -> bool {
    left.iter()
        .zip(right)
        .all(|(left, right)| (*left - right).abs() <= epsilon)
}

fn quaternion_inverse(value: [f32; 4]) -> [f32; 4] {
    [-value[0], -value[1], -value[2], value[3]]
}

fn quaternion_multiply(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    let [lx, ly, lz, lw] = left;
    let [rx, ry, rz, rw] = right;
    [
        lw * rx + lx * rw + ly * rz - lz * ry,
        lw * ry - lx * rz + ly * rw + lz * rx,
        lw * rz + lx * ry - ly * rx + lz * rw,
        lw * rw - lx * rx - ly * ry - lz * rz,
    ]
}

fn quaternion_rotate(rotation: [f32; 4], vector: [f32; 3]) -> [f32; 3] {
    let vector_quaternion = [vector[0], vector[1], vector[2], 0.0];
    let rotated = quaternion_multiply(
        quaternion_multiply(rotation, vector_quaternion),
        quaternion_inverse(rotation),
    );
    [rotated[0], rotated[1], rotated[2]]
}

fn vector_length(value: [f32; 3]) -> f32 {
    value
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt()
}

fn fingerprint_json(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("retarget contract is serializable");
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn transfer_diagnostic(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> AnimationTransferDiagnosticV1 {
    AnimationTransferDiagnosticV1 {
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
        action: if code.contains("REST") || code.contains("NODE-ID") {
            "Review the reported rig difference, then use one of Core's explicitly allowed transfer modes."
        } else {
            "Choose a donor with the same unique bone names, animation root, and parent-name hierarchy."
        }
        .to_owned(),
    }
}

fn transfer_error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> AnimationTransferErrorV1 {
    AnimationTransferErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn pipeline_transfer_error(
    path: &str,
    code: impl Into<String>,
    message: impl Into<String>,
) -> AnimationTransferErrorV1 {
    AnimationTransferErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.to_owned(),
        message: message.into(),
    }
}
