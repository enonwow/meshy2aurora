//! One fail-closed admission decision for every reference-supermodel route.
//!
//! Diagnostics may retain a rejected rig so an author can inspect it.  Product
//! packaging and export may not reinterpret the same rejected stage as ready.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceSupermodelAdmissionModeV3 {
    Diagnostic,
    Product,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelAdmissionInputV3 {
    pub schema_version: u32,
    pub mode: ReferenceSupermodelAdmissionModeV3,
    pub structure_status: String,
    pub surface_anatomy_status: String,
    pub joint_fit_status: String,
    pub skinning_status: String,
    pub bind_pose_status: String,
    pub motion_quality_status: String,
    pub exact_chain_validated: bool,
    pub full_carrier_coverage: bool,
    pub required_joint_coverage: bool,
    pub skin_influence_coverage: bool,
    pub inherited_clip_coverage: bool,
    pub visible_motion_coverage: bool,
    pub seam_violation_count: u64,
    pub motion_compatible: bool,
    pub runtime_readiness: String,
    pub semantic_delta_required: bool,
    pub semantic_delta_proven: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelAdmissionStageV3 {
    pub stage: String,
    pub source_status: String,
    pub status: String,
    pub blocking_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelAdmissionV3 {
    pub schema_version: u32,
    pub mode: ReferenceSupermodelAdmissionModeV3,
    pub status: String,
    pub export_allowed: bool,
    pub first_blocking_stage: Option<String>,
    pub blocking_codes: Vec<String>,
    pub stages: Vec<ReferenceSupermodelAdmissionStageV3>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelAdmissionErrorV3 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

pub fn evaluate_reference_supermodel_admission_v3(
    input: &ReferenceSupermodelAdmissionInputV3,
) -> Result<ReferenceSupermodelAdmissionV3, ReferenceSupermodelAdmissionErrorV3> {
    if input.schema_version != 3 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-ADMISSION-VERSION",
            "admission.schemaVersion",
            "reference-supermodel admission requires schema version 3",
        ));
    }
    for (path, value) in [
        ("admission.structureStatus", input.structure_status.as_str()),
        (
            "admission.surfaceAnatomyStatus",
            input.surface_anatomy_status.as_str(),
        ),
        ("admission.jointFitStatus", input.joint_fit_status.as_str()),
        ("admission.skinningStatus", input.skinning_status.as_str()),
        ("admission.bindPoseStatus", input.bind_pose_status.as_str()),
        (
            "admission.motionQualityStatus",
            input.motion_quality_status.as_str(),
        ),
        (
            "admission.runtimeReadiness",
            input.runtime_readiness.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-ADMISSION-STATUS-MISSING",
                path,
                "every admission stage requires an explicit source status",
            ));
        }
    }

    let mut stages = Vec::new();
    stages.push(stage(
        "structure",
        &input.structure_status,
        input.structure_status == "REFERENCE_SUPERMODEL_STRUCTURALLY_READY"
            && input.exact_chain_validated
            && input.full_carrier_coverage
            && input.required_joint_coverage,
        [
            (!input.exact_chain_validated).then_some("BLOCKED_EXACT_CHAIN_UNVERIFIED"),
            (!input.full_carrier_coverage).then_some("BLOCKED_FULL_CARRIER_COVERAGE"),
            (!input.required_joint_coverage).then_some("BLOCKED_REQUIRED_JOINT_COVERAGE"),
            (input.structure_status != "REFERENCE_SUPERMODEL_STRUCTURALLY_READY")
                .then_some("BLOCKED_REFERENCE_SUPERMODEL_STRUCTURE"),
        ],
    ));
    stages.push(stage(
        "surfaceAnatomy",
        &input.surface_anatomy_status,
        input.surface_anatomy_status == "READY",
        [(input.surface_anatomy_status != "READY").then_some("BLOCKED_SURFACE_ANATOMY")],
    ));
    stages.push(stage(
        "jointFit",
        &input.joint_fit_status,
        input.joint_fit_status == "READY",
        [(input.joint_fit_status != "READY").then_some("BLOCKED_JOINT_FIT")],
    ));
    stages.push(stage(
        "skinning",
        &input.skinning_status,
        input.skinning_status == "READY" && input.skin_influence_coverage,
        [
            (input.skinning_status != "READY").then_some("BLOCKED_SKINNING"),
            (!input.skin_influence_coverage).then_some("BLOCKED_SKIN_INFLUENCE_COVERAGE"),
        ],
    ));
    stages.push(stage(
        "bindPose",
        &input.bind_pose_status,
        input.bind_pose_status == "PASS",
        [(input.bind_pose_status != "PASS").then_some("BLOCKED_BIND_POSE")],
    ));
    stages.push(stage(
        "motionQuality",
        &input.motion_quality_status,
        input.motion_quality_status == "PASS"
            && input.motion_compatible
            && input.inherited_clip_coverage
            && input.visible_motion_coverage
            && input.seam_violation_count == 0,
        [
            (input.motion_quality_status != "PASS" || !input.motion_compatible)
                .then_some("BLOCKED_MOTION_INCOMPATIBLE"),
            (!input.inherited_clip_coverage).then_some("BLOCKED_INHERITED_CLIP_COVERAGE"),
            (!input.visible_motion_coverage).then_some("BLOCKED_VISIBLE_MOTION_COVERAGE"),
            (input.seam_violation_count != 0).then_some("BLOCKED_SURFACE_SEAM_CONTINUITY"),
        ],
    ));
    stages.push(stage(
        "export",
        &input.runtime_readiness,
        input.runtime_readiness == "RUNTIME_UNPROVEN"
            && (!input.semantic_delta_required || input.semantic_delta_proven),
        [
            (input.runtime_readiness != "RUNTIME_UNPROVEN")
                .then_some("BLOCKED_DIAGNOSTIC_RUNTIME_READINESS"),
            (input.semantic_delta_required && !input.semantic_delta_proven)
                .then_some("BLOCKED_SEMANTIC_DELTA_MISSING"),
        ],
    ));

    let blocking_codes = stages
        .iter()
        .flat_map(|stage| stage.blocking_codes.iter().cloned())
        .collect::<Vec<_>>();
    let first_blocking_stage = stages
        .iter()
        .find(|stage| stage.status == "BLOCKED")
        .map(|stage| stage.stage.clone());
    let all_pass = blocking_codes.is_empty();
    let export_allowed = input.mode == ReferenceSupermodelAdmissionModeV3::Product && all_pass;
    let status = if all_pass {
        "PASS"
    } else if input.mode == ReferenceSupermodelAdmissionModeV3::Diagnostic {
        "DIAGNOSTIC_BLOCKED"
    } else {
        "BLOCKED"
    };
    Ok(ReferenceSupermodelAdmissionV3 {
        schema_version: 3,
        mode: input.mode,
        status: status.to_owned(),
        export_allowed,
        first_blocking_stage,
        blocking_codes,
        stages,
    })
}

fn stage<const N: usize>(
    name: &str,
    source_status: &str,
    passed: bool,
    codes: [Option<&str>; N],
) -> ReferenceSupermodelAdmissionStageV3 {
    let blocking_codes = codes
        .into_iter()
        .flatten()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    ReferenceSupermodelAdmissionStageV3 {
        stage: name.to_owned(),
        source_status: source_status.to_owned(),
        status: if passed && blocking_codes.is_empty() {
            "PASS"
        } else {
            "BLOCKED"
        }
        .to_owned(),
        blocking_codes,
    }
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReferenceSupermodelAdmissionErrorV3 {
    ReferenceSupermodelAdmissionErrorV3 {
        schema_version: 3,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing(mode: ReferenceSupermodelAdmissionModeV3) -> ReferenceSupermodelAdmissionInputV3 {
        ReferenceSupermodelAdmissionInputV3 {
            schema_version: 3,
            mode,
            structure_status: "REFERENCE_SUPERMODEL_STRUCTURALLY_READY".to_owned(),
            surface_anatomy_status: "READY".to_owned(),
            joint_fit_status: "READY".to_owned(),
            skinning_status: "READY".to_owned(),
            bind_pose_status: "PASS".to_owned(),
            motion_quality_status: "PASS".to_owned(),
            exact_chain_validated: true,
            full_carrier_coverage: true,
            required_joint_coverage: true,
            skin_influence_coverage: true,
            inherited_clip_coverage: true,
            visible_motion_coverage: true,
            seam_violation_count: 0,
            motion_compatible: true,
            runtime_readiness: "RUNTIME_UNPROVEN".to_owned(),
            semantic_delta_required: false,
            semantic_delta_proven: false,
        }
    }

    #[test]
    fn green_coverage_cannot_hide_a_rejected_joint_fit() {
        let mut input = passing(ReferenceSupermodelAdmissionModeV3::Product);
        input.joint_fit_status = "NEEDS_AUTHORING".to_owned();

        let report = evaluate_reference_supermodel_admission_v3(&input).unwrap();

        assert_eq!(report.status, "BLOCKED");
        assert!(!report.export_allowed);
        assert_eq!(report.first_blocking_stage.as_deref(), Some("jointFit"));
        assert!(
            report
                .blocking_codes
                .contains(&"BLOCKED_JOINT_FIT".to_owned())
        );
    }

    #[test]
    fn diagnostic_retains_but_never_exports_a_rejected_rig() {
        let mut input = passing(ReferenceSupermodelAdmissionModeV3::Diagnostic);
        input.skinning_status = "BLOCKED".to_owned();

        let report = evaluate_reference_supermodel_admission_v3(&input).unwrap();

        assert_eq!(report.status, "DIAGNOSTIC_BLOCKED");
        assert!(!report.export_allowed);
        assert_eq!(report.first_blocking_stage.as_deref(), Some("skinning"));
    }

    #[test]
    fn pass_has_no_blocked_substage() {
        let report = evaluate_reference_supermodel_admission_v3(&passing(
            ReferenceSupermodelAdmissionModeV3::Product,
        ))
        .unwrap();

        assert_eq!(report.status, "PASS");
        assert!(report.export_allowed);
        assert!(report.blocking_codes.is_empty());
        assert!(report.stages.iter().all(|stage| stage.status == "PASS"));
    }
}
