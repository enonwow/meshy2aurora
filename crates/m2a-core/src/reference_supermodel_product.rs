//! Common exact-chain contract for applying reference supermodels.
//!
//! Family-specific code owns rig construction and semantic annotations. This
//! module owns only selection identity, chain provenance and export admission;
//! it never names or special-cases a retail supermodel family.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(feature = "legacy-c-wolf-demo")]
use crate::two_da::retain_two_da_row_prefix_v1;
use crate::two_da::{
    TwoDaAppendArtifactV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
    append_two_da_row_v1, clone_two_da_row_request_v1, inspect_two_da_v2, read_two_da_row_v2,
};
use crate::{
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1},
    hak::{HakArtifactV1, HakWriterOptionsV1},
    package::PackageManifestV1,
    reference_supermodel_admission::{
        ReferenceSupermodelAdmissionInputV3, ReferenceSupermodelAdmissionModeV3,
        ReferenceSupermodelAdmissionV3, evaluate_reference_supermodel_admission_v3,
    },
    reference_supermodel_generic::{
        GenericReferenceRigAnalysisV2, ReferenceSupermodelChainAnalysisReportV2,
    },
    reference_supermodel_motion::{
        ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelSemanticDeltaReportV1,
        package_reference_supermodel_minimal_twosided_creature_hak_v1,
    },
    tga::{TgaArtifactV1, TgaWriterOptionsV1, write_tga_v1},
};

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelSurfaceContinuityPolicyV1 {
    pub fail_on_any_seam_violation: bool,
}

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelRigProfileV1 {
    pub schema_version: u32,
    pub profile_id: String,
    pub supermodel_resref: String,
    pub motion_provider_resref: String,
    pub visible_tail_roles: Vec<String>,
    pub required_tail_clips: Vec<String>,
    pub surface_continuity: ReferenceSupermodelSurfaceContinuityPolicyV1,
    pub appearance: ReferenceSupermodelAppearanceProfileV1,
}

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelAppearanceProfileV1 {
    pub retained_physical_rows: u32,
    pub donor_physical_row: u32,
    pub expected_donor_model_type: String,
    pub expected_donor_race: String,
}

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelExactChainResourceV1 {
    pub resref: String,
    pub supermodel_resref: String,
    pub sha256: String,
    pub byte_length: usize,
}

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelProfileValidationReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub profile_id: String,
    pub selected_supermodel_resref: String,
    pub motion_provider_resref: String,
    pub exact_chain_resource_count: usize,
    pub exact_chain_sha256: Vec<String>,
    pub retail_payload_copied: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelProductErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelExportAdmissionInputV1 {
    pub schema_version: u32,
    pub motion_compatible: bool,
    pub motion_quality_status: String,
    pub runtime_readiness: String,
    pub exact_chain_validated: bool,
    pub full_carrier_coverage: bool,
    pub required_joint_coverage: bool,
    pub skin_influence_coverage: bool,
    pub inherited_clip_coverage: bool,
    pub visible_motion_coverage: bool,
    pub seam_violation_count: u64,
    pub semantic_delta_required: bool,
    pub semantic_delta_proven: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSupermodelExportAdmissionReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub runtime_readiness: String,
    pub diagnostic_preview: bool,
    pub full_carrier_coverage: bool,
    pub required_joint_coverage: bool,
    pub skin_influence_coverage: bool,
    pub inherited_clip_coverage: bool,
    pub visible_motion_coverage: bool,
    pub seam_violation_count: u64,
}

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelProductIdentityV1 {
    pub model_resref: String,
    pub texture_resref: String,
    pub material_resref: String,
    pub hak_resref: String,
    pub appearance_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSupermodelProductIdentityV2 {
    pub model_resref: String,
    pub texture_resref: String,
    pub material_resref: String,
    pub hak_resref: String,
    pub appearance_label: String,
    pub appearance_donor_resrefs: Vec<String>,
    #[serde(default)]
    pub rejected_baseline:
        Option<crate::reference_supermodel_motion::ReferenceSupermodelRejectedBaselineV1>,
    #[serde(default)]
    pub semantic_controller_names: Vec<String>,
}

#[cfg(feature = "legacy-c-wolf-demo")]
#[derive(Debug)]
pub struct ReferenceSupermodelCreatureProductV1 {
    pub profile_validation: ReferenceSupermodelProfileValidationReportV1,
    pub admission: ReferenceSupermodelExportAdmissionReportV1,
    pub identity: ReferenceSupermodelProductIdentityV1,
    pub texture: TgaArtifactV1,
    pub appearance: TwoDaAppendArtifactV1,
    pub hak: HakArtifactV1,
    pub package_manifest: PackageManifestV1,
}

#[derive(Debug)]
pub struct ReferenceSupermodelCreatureProductV2 {
    pub selection_analysis: ReferenceSupermodelChainAnalysisReportV2,
    pub admission: ReferenceSupermodelExportAdmissionReportV1,
    pub admission_v3: ReferenceSupermodelAdmissionV3,
    pub identity: ReferenceSupermodelProductIdentityV2,
    pub appearance_donor_resref: String,
    pub appearance_donor_physical_row: u32,
    pub texture: TgaArtifactV1,
    pub appearance: TwoDaAppendArtifactV1,
    pub hak: HakArtifactV1,
    pub package_manifest: PackageManifestV1,
}

impl fmt::Display for ReferenceSupermodelProductErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ReferenceSupermodelProductErrorV1 {}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReferenceSupermodelProductErrorV1 {
    ReferenceSupermodelProductErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

fn is_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(feature = "legacy-c-wolf-demo")]
fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn reference_supermodel_resource_sha256_v1(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(feature = "legacy-c-wolf-demo")]
pub fn build_reference_supermodel_appearance_v1(
    base_appearance_two_da: &[u8],
    profile: &ReferenceSupermodelRigProfileV1,
    model_resref: &str,
    appearance_label: &str,
) -> Result<TwoDaAppendArtifactV1, ReferenceSupermodelProductErrorV1> {
    if !is_resref(model_resref)
        || appearance_label.trim().is_empty()
        || profile.appearance.retained_physical_rows == 0
        || profile.appearance.donor_physical_row >= profile.appearance.retained_physical_rows
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-PROFILE-INVALID",
            "profile.appearance",
            "appearance adapter requires a retained prefix, donor row, model resref and label",
        ));
    }
    let limits = TwoDaLimitsV1::default();
    let retained = retain_two_da_row_prefix_v1(
        base_appearance_two_da,
        profile.appearance.retained_physical_rows,
        &limits,
    )
    .map_err(|source| error(source.code, source.path, source.message))?;
    let inspection = inspect_two_da_v2(&retained, &limits)
        .map_err(|source| error(source.code, source.path, source.message))?;
    let donor = read_two_da_row_v2(&retained, profile.appearance.donor_physical_row, &limits)
        .map_err(|source| error(source.code, source.path, source.message))?;
    let donor_text = |column_name: &str| {
        inspection
            .columns
            .iter()
            .position(|name| name.eq_ignore_ascii_case(column_name))
            .and_then(|index| donor.cells.get(index))
            .and_then(|value| match value {
                TwoDaCellValueV1::Text { value } => Some(value.as_str()),
                TwoDaCellValueV1::Null => None,
            })
    };
    if donor_text("MODELTYPE") != Some(profile.appearance.expected_donor_model_type.as_str())
        || donor_text("RACE") != Some(profile.appearance.expected_donor_race.as_str())
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-DONOR-MISMATCH",
            "appearance.donor",
            "the exact donor row does not match the family adapter's model type and race",
        ));
    }
    let text = |column_name: &str, value: &str| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text {
            value: value.to_owned(),
        },
    };
    let request = clone_two_da_row_request_v1(
        &retained,
        profile.appearance.donor_physical_row,
        &[text("LABEL", appearance_label), text("RACE", model_resref)],
        &limits,
    )
    .map_err(|source| error(source.code, source.path, source.message))?;
    append_two_da_row_v1(&retained, &request, &limits)
        .map_err(|source| error(source.code, source.path, source.message))
}

/// Finds a structural single-part Creature appearance donor from caller-bound
/// catalog children. Selection is based on actual 2DA data, never on a family
/// or species name embedded in product code.
pub fn build_reference_supermodel_appearance_v2(
    base_appearance_two_da: &[u8],
    appearance_donor_resrefs: &[String],
    model_resref: &str,
    appearance_label: &str,
) -> Result<(TwoDaAppendArtifactV1, String, u32), ReferenceSupermodelProductErrorV1> {
    if !is_resref(model_resref)
        || appearance_label.trim().is_empty()
        || appearance_donor_resrefs.is_empty()
        || appearance_donor_resrefs
            .iter()
            .any(|value| !is_resref(value))
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-SELECTION-INVALID",
            "appearance",
            "appearance selection requires output identity and caller-bound donor resrefs",
        ));
    }
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(base_appearance_two_da, &limits)
        .map_err(|source| error(source.code, source.path, source.message))?;
    let column = |name: &str| {
        inspection
            .columns
            .iter()
            .position(|candidate| candidate.eq_ignore_ascii_case(name))
    };
    let model_type_column = column("MODELTYPE").ok_or_else(|| {
        error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-COLUMN-MISSING",
            "appearance.columns.MODELTYPE",
            "appearance.2da has no MODELTYPE column",
        )
    })?;
    let race_column = column("RACE").ok_or_else(|| {
        error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-COLUMN-MISSING",
            "appearance.columns.RACE",
            "appearance.2da has no RACE column",
        )
    })?;
    let donor_set = appearance_donor_resrefs
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut selected = None;
    for row_index in 0..inspection.physical_row_count {
        let row = read_two_da_row_v2(base_appearance_two_da, row_index, &limits)
            .map_err(|source| error(source.code, source.path, source.message))?;
        let text = |column_index: usize| match row.cells.get(column_index) {
            Some(TwoDaCellValueV1::Text { value }) => Some(value.as_str()),
            _ => None,
        };
        let model_type = text(model_type_column);
        let race = text(race_column);
        if model_type.is_some_and(|value| value.eq_ignore_ascii_case("S"))
            && race.is_some_and(|value| donor_set.contains(&value.to_ascii_lowercase()))
        {
            selected = Some((row_index, race.unwrap().to_ascii_lowercase()));
            break;
        }
    }
    let (donor_row, donor_resref) = selected.ok_or_else(|| {
        error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-DONOR-MISSING",
            "appearance.rows",
            format!(
                "no single-part Creature row matches catalog donor candidates: {}",
                appearance_donor_resrefs.join(", ")
            ),
        )
    })?;
    let text = |column_name: &str, value: &str| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text {
            value: value.to_owned(),
        },
    };
    let request = clone_two_da_row_request_v1(
        base_appearance_two_da,
        donor_row,
        &[text("LABEL", appearance_label), text("RACE", model_resref)],
        &limits,
    )
    .map_err(|source| error(source.code, source.path, source.message))?;
    let artifact = append_two_da_row_v1(base_appearance_two_da, &request, &limits)
        .map_err(|source| error(source.code, source.path, source.message))?;
    Ok((artifact, donor_resref, donor_row))
}

#[allow(clippy::too_many_arguments)]
pub fn package_reference_supermodel_creature_product_v2(
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
    selection_analysis: &ReferenceSupermodelChainAnalysisReportV2,
    rig_analysis: &GenericReferenceRigAnalysisV2,
    identity: &ReferenceSupermodelProductIdentityV2,
    motion: &ReferenceSupermodelMinimalMtrMotionArtifactV1,
    semantic_delta_required: bool,
    semantic_delta: Option<&ReferenceSupermodelSemanticDeltaReportV1>,
) -> Result<ReferenceSupermodelCreatureProductV2, ReferenceSupermodelProductErrorV1> {
    for (path, value) in [
        ("identity.modelResref", identity.model_resref.as_str()),
        ("identity.textureResref", identity.texture_resref.as_str()),
        ("identity.materialResref", identity.material_resref.as_str()),
        ("identity.hakResref", identity.hak_resref.as_str()),
    ] {
        if !is_resref(value) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-PRODUCT-IDENTITY-INVALID",
                path,
                "product resource identities must be lowercase Aurora resrefs",
            ));
        }
    }
    if selection_analysis.status != "REFERENCE_SUPERMODEL_STRUCTURALLY_READY"
        || !selection_analysis.structural_errors.is_empty()
        || selection_analysis.exact_chain.is_empty()
        || selection_analysis.retail_payload_copied
    {
        return Err(error(
            "BLOCKED_REFERENCE_SUPERMODEL_STRUCTURE",
            "selectionAnalysis",
            "only an exact, structurally ready, read-only chain may enter Creature export",
        ));
    }
    if motion.report.model_resource_resref != identity.model_resref
        || motion.mtr_resource.resref != identity.material_resref
        || !motion
            .report
            .supermodel_resref
            .eq_ignore_ascii_case(&selection_analysis.selected_supermodel_resref)
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-PRODUCT-BINDING-MISMATCH",
            "motion.report",
            "model, material and selected supermodel differ from the exact product request",
        ));
    }
    let admission_v3 =
        evaluate_reference_supermodel_admission_v3(&ReferenceSupermodelAdmissionInputV3 {
            schema_version: 3,
            mode: ReferenceSupermodelAdmissionModeV3::Product,
            structure_status: selection_analysis.status.clone(),
            surface_anatomy_status: rig_analysis.surface_anatomy.status.clone(),
            joint_fit_status: rig_analysis.joint_fit.status.clone(),
            skinning_status: rig_analysis.skinning.status.clone(),
            bind_pose_status: rig_analysis.bind_pose.status.clone(),
            motion_quality_status: motion.motion_quality.status.clone(),
            exact_chain_validated: true,
            full_carrier_coverage: motion.report.carrier_coverage.full_carrier_coverage,
            required_joint_coverage: motion.report.carrier_coverage.required_joint_coverage,
            skin_influence_coverage: motion.report.skin_influence_coverage,
            inherited_clip_coverage: motion.motion_quality.inherited_clip_coverage,
            visible_motion_coverage: motion.motion_quality.visible_motion_coverage,
            seam_violation_count: motion.motion_quality.seam_pair_violation_count,
            motion_compatible: motion.report.motion_compatible,
            runtime_readiness: motion.report.runtime_readiness.clone(),
            semantic_delta_required,
            semantic_delta_proven: semantic_delta.is_some_and(|delta| delta.export_delta_proven),
        })
        .map_err(|source| error(source.code, source.path, source.message))?;
    if !admission_v3.export_allowed {
        return Err(error(
            admission_v3
                .blocking_codes
                .first()
                .cloned()
                .unwrap_or_else(|| "BLOCKED_REFERENCE_SUPERMODEL_ADMISSION".to_owned()),
            admission_v3.first_blocking_stage.as_deref().map_or_else(
                || "admission".to_owned(),
                |stage| format!("admission.{stage}"),
            ),
            format!(
                "reference-supermodel product admission blocked: {}",
                admission_v3.blocking_codes.join(", ")
            ),
        ));
    }
    let admission =
        admit_reference_supermodel_export_v1(&ReferenceSupermodelExportAdmissionInputV1 {
            schema_version: 2,
            motion_compatible: motion.report.motion_compatible,
            motion_quality_status: motion.motion_quality.status.clone(),
            runtime_readiness: motion.report.runtime_readiness.clone(),
            exact_chain_validated: true,
            full_carrier_coverage: motion.report.carrier_coverage.full_carrier_coverage,
            required_joint_coverage: motion.report.carrier_coverage.required_joint_coverage,
            skin_influence_coverage: motion.report.skin_influence_coverage,
            inherited_clip_coverage: motion.motion_quality.inherited_clip_coverage,
            visible_motion_coverage: motion.motion_quality.visible_motion_coverage,
            seam_violation_count: motion.motion_quality.seam_pair_violation_count,
            semantic_delta_required,
            semantic_delta_proven: semantic_delta.is_some_and(|delta| delta.export_delta_proven),
        })?;
    let decoded = decode_embedded_image_to_tga_v1(
        source_glb,
        0,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            source.code,
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            source.message,
        )
    })?;
    let texture = write_tga_v1(&decoded, &TgaWriterOptionsV1::default())
        .map_err(|source| error(source.code, source.path, source.message))?;
    let (appearance, appearance_donor_resref, appearance_donor_physical_row) =
        build_reference_supermodel_appearance_v2(
            base_appearance_two_da,
            &identity.appearance_donor_resrefs,
            &identity.model_resref,
            &identity.appearance_label,
        )?;
    let package = package_reference_supermodel_minimal_twosided_creature_hak_v1(
        motion,
        &identity.texture_resref,
        &texture.payload,
        &appearance.payload,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| error(source.code, source.path, source.message))?;
    let resources = vec![
        crate::hak::HakResourceInputV1 {
            resref: identity.model_resref.clone(),
            resource_type: 2002,
            payload: motion.model.payload.clone(),
        },
        crate::hak::HakResourceInputV1 {
            resref: identity.texture_resref.clone(),
            resource_type: 3,
            payload: texture.payload.clone(),
        },
        motion.mtr_resource.clone(),
        crate::hak::HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package_manifest =
        crate::package::write_package_manifest_v1(&resources, &HakWriterOptionsV1::default())
            .map_err(|source| error(source.code, source.path, source.message))?;
    if package_manifest.package_sha256 != package.hak.report.archive_sha256 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-PRODUCT-MANIFEST-DIFF",
            "packageManifest.packageSha256",
            "independent package manifest differs from the admitted HAK",
        ));
    }
    Ok(ReferenceSupermodelCreatureProductV2 {
        selection_analysis: selection_analysis.clone(),
        admission,
        admission_v3,
        identity: identity.clone(),
        appearance_donor_resref,
        appearance_donor_physical_row,
        texture,
        appearance,
        hak: package.hak,
        package_manifest,
    })
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "legacy-c-wolf-demo")]
pub fn package_reference_supermodel_creature_product_v1(
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
    profile: &ReferenceSupermodelRigProfileV1,
    exact_chain: &[ReferenceSupermodelExactChainResourceV1],
    identity: &ReferenceSupermodelProductIdentityV1,
    motion: &ReferenceSupermodelMinimalMtrMotionArtifactV1,
    semantic_delta_required: bool,
    semantic_delta: Option<&ReferenceSupermodelSemanticDeltaReportV1>,
) -> Result<ReferenceSupermodelCreatureProductV1, ReferenceSupermodelProductErrorV1> {
    for (path, value) in [
        ("identity.modelResref", identity.model_resref.as_str()),
        ("identity.textureResref", identity.texture_resref.as_str()),
        ("identity.materialResref", identity.material_resref.as_str()),
        ("identity.hakResref", identity.hak_resref.as_str()),
    ] {
        if !is_resref(value) {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-PRODUCT-IDENTITY-INVALID",
                path,
                "product resource identities must be lowercase Aurora resrefs",
            ));
        }
    }
    if motion.report.model_resource_resref != identity.model_resref
        || motion.mtr_resource.resref != identity.material_resref
        || !motion
            .report
            .supermodel_resref
            .eq_ignore_ascii_case(&profile.supermodel_resref)
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-PRODUCT-BINDING-MISMATCH",
            "motion.report",
            "model, material and supermodel identities differ from the exact product request",
        ));
    }
    let profile_validation = validate_reference_supermodel_rig_profile_v1(profile, exact_chain)?;
    let admission =
        admit_reference_supermodel_export_v1(&ReferenceSupermodelExportAdmissionInputV1 {
            schema_version: 2,
            motion_compatible: motion.report.motion_compatible,
            motion_quality_status: motion.motion_quality.status.clone(),
            runtime_readiness: motion.report.runtime_readiness.clone(),
            exact_chain_validated: profile_validation.status
                == "REFERENCE_SUPERMODEL_PROFILE_READY",
            full_carrier_coverage: motion.report.carrier_coverage.full_carrier_coverage,
            required_joint_coverage: motion.report.carrier_coverage.required_joint_coverage,
            skin_influence_coverage: motion.report.skin_influence_coverage,
            inherited_clip_coverage: motion.motion_quality.inherited_clip_coverage,
            visible_motion_coverage: motion.motion_quality.visible_motion_coverage,
            seam_violation_count: motion.motion_quality.seam_pair_violation_count,
            semantic_delta_required,
            semantic_delta_proven: semantic_delta.is_some_and(|delta| delta.export_delta_proven),
        })?;
    let decoded = decode_embedded_image_to_tga_v1(
        source_glb,
        0,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            source.code,
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            source.message,
        )
    })?;
    let texture = write_tga_v1(&decoded, &TgaWriterOptionsV1::default())
        .map_err(|source| error(source.code, source.path, source.message))?;
    let appearance = build_reference_supermodel_appearance_v1(
        base_appearance_two_da,
        profile,
        &identity.model_resref,
        &identity.appearance_label,
    )?;
    let package = package_reference_supermodel_minimal_twosided_creature_hak_v1(
        motion,
        &identity.texture_resref,
        &texture.payload,
        &appearance.payload,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| error(source.code, source.path, source.message))?;
    let resources = vec![
        crate::hak::HakResourceInputV1 {
            resref: identity.model_resref.clone(),
            resource_type: 2002,
            payload: motion.model.payload.clone(),
        },
        crate::hak::HakResourceInputV1 {
            resref: identity.texture_resref.clone(),
            resource_type: 3,
            payload: texture.payload.clone(),
        },
        motion.mtr_resource.clone(),
        crate::hak::HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package_manifest =
        crate::package::write_package_manifest_v1(&resources, &HakWriterOptionsV1::default())
            .map_err(|source| error(source.code, source.path, source.message))?;
    if package_manifest.package_sha256 != package.hak.report.archive_sha256 {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-PRODUCT-MANIFEST-DIFF",
            "packageManifest.packageSha256",
            "independent package manifest differs from the admitted HAK",
        ));
    }
    Ok(ReferenceSupermodelCreatureProductV1 {
        profile_validation,
        admission,
        identity: identity.clone(),
        texture,
        appearance,
        hak: package.hak,
        package_manifest,
    })
}

pub fn admit_reference_supermodel_export_v1(
    input: &ReferenceSupermodelExportAdmissionInputV1,
) -> Result<ReferenceSupermodelExportAdmissionReportV1, ReferenceSupermodelProductErrorV1> {
    if input.schema_version != 2 || input.motion_quality_status.trim().is_empty() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-EXPORT-ADMISSION-INVALID",
            "admission",
            "export admission requires the exact V2 full-skeleton and motion-quality result",
        ));
    }
    if !input.exact_chain_validated {
        return Err(error(
            "BLOCKED_EXACT_CHAIN_UNVERIFIED",
            "admission.exactChainValidated",
            "the selected-to-root MDL/resref/SHA-256 chain was not validated",
        ));
    }
    for (passed, code, path, message) in [
        (
            input.full_carrier_coverage,
            "BLOCKED_FULL_CARRIER_COVERAGE",
            "admission.fullCarrierCoverage",
            "output carrier readback is missing, adding or reparenting required supermodel carriers",
        ),
        (
            input.required_joint_coverage,
            "BLOCKED_REQUIRED_JOINT_COVERAGE",
            "admission.requiredJointCoverage",
            "the complete inherited carrier inventory is not present in the child skeleton",
        ),
        (
            input.skin_influence_coverage,
            "BLOCKED_SKIN_INFLUENCE_COVERAGE",
            "admission.skinInfluenceCoverage",
            "one or more skin-relevant animated joints has no active visible surface influence",
        ),
        (
            input.inherited_clip_coverage,
            "BLOCKED_INHERITED_CLIP_COVERAGE",
            "admission.inheritedClipCoverage",
            "the complete inherited clip corpus was not sampled",
        ),
        (
            input.visible_motion_coverage,
            "BLOCKED_VISIBLE_MOTION_COVERAGE",
            "admission.visibleMotionCoverage",
            "one or more dynamic joint×clip cells has no visible surface response",
        ),
    ] {
        if !passed {
            return Err(error(code, path, message));
        }
    }
    if input.seam_violation_count != 0 {
        return Err(error(
            "BLOCKED_SURFACE_SEAM_CONTINUITY",
            "admission.seamViolationCount",
            "surface seam continuity is fail-on-any and requires exactly zero violations",
        ));
    }
    if !input.motion_compatible || input.motion_quality_status != "PASS" {
        return Err(error(
            "BLOCKED_MOTION_INCOMPATIBLE",
            "admission.motionCompatible",
            "diagnostic or motion-incompatible output cannot enter Creature export",
        ));
    }
    if input.runtime_readiness != "RUNTIME_UNPROVEN" {
        return Err(error(
            "BLOCKED_DIAGNOSTIC_RUNTIME_READINESS",
            "admission.runtimeReadiness",
            "only a non-diagnostic, offline-qualified result can enter packaging",
        ));
    }
    if input.semantic_delta_required && !input.semantic_delta_proven {
        return Err(error(
            "BLOCKED_SEMANTIC_DELTA_MISSING",
            "admission.semanticDeltaProven",
            "the candidate does not differ in the rejected repair-area surface semantics",
        ));
    }
    Ok(ReferenceSupermodelExportAdmissionReportV1 {
        schema_version: 2,
        status: "REFERENCE_SUPERMODEL_EXPORT_ADMITTED".to_owned(),
        runtime_readiness: input.runtime_readiness.clone(),
        diagnostic_preview: false,
        full_carrier_coverage: input.full_carrier_coverage,
        required_joint_coverage: input.required_joint_coverage,
        skin_influence_coverage: input.skin_influence_coverage,
        inherited_clip_coverage: input.inherited_clip_coverage,
        visible_motion_coverage: input.visible_motion_coverage,
        seam_violation_count: input.seam_violation_count,
    })
}

#[cfg(feature = "legacy-c-wolf-demo")]
pub fn validate_reference_supermodel_rig_profile_v1(
    profile: &ReferenceSupermodelRigProfileV1,
    chain: &[ReferenceSupermodelExactChainResourceV1],
) -> Result<ReferenceSupermodelProfileValidationReportV1, ReferenceSupermodelProductErrorV1> {
    if profile.profile_id.trim().is_empty() {
        return Err(error(
            "BLOCKED_PROFILE_MISSING",
            "profile.profileId",
            "the selected supermodel family has no registered rig adapter",
        ));
    }
    if profile.schema_version != 1
        || !is_resref(&profile.supermodel_resref)
        || !is_resref(&profile.motion_provider_resref)
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-PROFILE-INVALID",
            "profile",
            "profile schema and exact lowercase resrefs must satisfy V1",
        ));
    }
    if profile.appearance.retained_physical_rows == 0
        || profile.appearance.donor_physical_row >= profile.appearance.retained_physical_rows
        || profile
            .appearance
            .expected_donor_model_type
            .trim()
            .is_empty()
        || profile.appearance.expected_donor_race.trim().is_empty()
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-APPEARANCE-PROFILE-INVALID",
            "profile.appearance",
            "family adapters require an explicit appearance donor contract",
        ));
    }
    if chain.is_empty() {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-CHAIN-EMPTY",
            "chain",
            "the exact selected-to-root supermodel chain is required",
        ));
    }
    if !chain[0]
        .resref
        .eq_ignore_ascii_case(&profile.supermodel_resref)
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-SELECTION-MISMATCH",
            "chain[0].resref",
            "the exact chain does not begin with the profile supermodel",
        ));
    }
    let mut seen = BTreeSet::new();
    for (index, resource) in chain.iter().enumerate() {
        if !is_resref(&resource.resref)
            || !is_sha256(&resource.sha256)
            || resource.byte_length == 0
            || !seen.insert(resource.resref.to_ascii_lowercase())
        {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-CHAIN-RESOURCE-INVALID",
                format!("chain[{index}]"),
                "every exact chain resource requires a unique resref, byte length and lowercase SHA-256",
            ));
        }
        let expected_parent = chain
            .get(index + 1)
            .map(|parent| parent.resref.as_str())
            .unwrap_or("NULL");
        if !resource
            .supermodel_resref
            .eq_ignore_ascii_case(expected_parent)
        {
            return Err(error(
                "M2A-REFERENCE-SUPERMODEL-CHAIN-MISMATCH",
                format!("chain[{index}].supermodelResref"),
                format!(
                    "declared parent {:?} does not match exact next chain resource {:?}",
                    resource.supermodel_resref, expected_parent
                ),
            ));
        }
    }
    if !chain.iter().any(|resource| {
        resource
            .resref
            .eq_ignore_ascii_case(&profile.motion_provider_resref)
    }) {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-MOTION-PROVIDER-MISSING",
            "profile.motionProviderResref",
            "the declared motion provider is absent from the exact chain",
        ));
    }
    if !profile.visible_tail_roles.is_empty()
        && (profile.required_tail_clips.is_empty()
            || profile
                .visible_tail_roles
                .iter()
                .any(|role| role.trim().is_empty()))
    {
        return Err(error(
            "M2A-REFERENCE-SUPERMODEL-TAIL-PROFILE-INVALID",
            "profile.visibleTailRoles",
            "visible tail roles require explicit blocking clips",
        ));
    }
    Ok(ReferenceSupermodelProfileValidationReportV1 {
        schema_version: 1,
        status: "REFERENCE_SUPERMODEL_PROFILE_READY".to_owned(),
        profile_id: profile.profile_id.clone(),
        selected_supermodel_resref: profile.supermodel_resref.clone(),
        motion_provider_resref: profile.motion_provider_resref.clone(),
        exact_chain_resource_count: chain.len(),
        exact_chain_sha256: chain
            .iter()
            .map(|resource| resource.sha256.clone())
            .collect(),
        retail_payload_copied: false,
    })
}
