use m2a_core::reference_supermodel_product::{
    ReferenceSupermodelExportAdmissionInputV1, admit_reference_supermodel_export_v1,
};

fn complete_admission() -> ReferenceSupermodelExportAdmissionInputV1 {
    ReferenceSupermodelExportAdmissionInputV1 {
        schema_version: 2,
        motion_compatible: true,
        motion_quality_status: "PASS".to_owned(),
        runtime_readiness: "RUNTIME_UNPROVEN".to_owned(),
        exact_chain_validated: true,
        full_carrier_coverage: true,
        required_joint_coverage: true,
        skin_influence_coverage: true,
        inherited_clip_coverage: true,
        visible_motion_coverage: true,
        seam_violation_count: 0,
        semantic_delta_required: false,
        semantic_delta_proven: false,
    }
}

#[test]
fn diagnostic_or_motion_incompatible_results_never_enter_export() {
    for (motion_compatible, runtime_readiness, expected) in [
        (
            false,
            "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED",
            "BLOCKED_MOTION_INCOMPATIBLE",
        ),
        (
            true,
            "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED",
            "BLOCKED_DIAGNOSTIC_RUNTIME_READINESS",
        ),
    ] {
        let mut input = complete_admission();
        input.motion_compatible = motion_compatible;
        input.runtime_readiness = runtime_readiness.to_owned();
        let error = admit_reference_supermodel_export_v1(&input).unwrap_err();
        assert_eq!(error.code, expected);
    }
}

#[test]
fn repaired_baseline_requires_a_semantic_delta_before_export() {
    let mut input = complete_admission();
    input.semantic_delta_required = true;
    assert_eq!(
        admit_reference_supermodel_export_v1(&input)
            .unwrap_err()
            .code,
        "BLOCKED_SEMANTIC_DELTA_MISSING",
    );
}

#[test]
fn every_full_skeleton_gate_is_required_before_product_admission() {
    for (field, expected) in [
        ("fullCarrierCoverage", "BLOCKED_FULL_CARRIER_COVERAGE"),
        ("requiredJointCoverage", "BLOCKED_REQUIRED_JOINT_COVERAGE"),
        ("skinInfluenceCoverage", "BLOCKED_SKIN_INFLUENCE_COVERAGE"),
        ("inheritedClipCoverage", "BLOCKED_INHERITED_CLIP_COVERAGE"),
        ("visibleMotionCoverage", "BLOCKED_VISIBLE_MOTION_COVERAGE"),
        ("seamViolationCount", "BLOCKED_SURFACE_SEAM_CONTINUITY"),
    ] {
        let mut input = complete_admission();
        match field {
            "fullCarrierCoverage" => input.full_carrier_coverage = false,
            "requiredJointCoverage" => input.required_joint_coverage = false,
            "skinInfluenceCoverage" => input.skin_influence_coverage = false,
            "inheritedClipCoverage" => input.inherited_clip_coverage = false,
            "visibleMotionCoverage" => input.visible_motion_coverage = false,
            "seamViolationCount" => input.seam_violation_count = 1,
            _ => unreachable!(),
        }
        assert_eq!(
            admit_reference_supermodel_export_v1(&input)
                .unwrap_err()
                .code,
            expected
        );
    }
    assert_eq!(
        admit_reference_supermodel_export_v1(&complete_admission())
            .expect("complete full-skeleton contract is admissible")
            .status,
        "REFERENCE_SUPERMODEL_EXPORT_ADMITTED"
    );
}
