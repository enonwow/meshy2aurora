//! Model-only M6 proof composition. This module intentionally has no GFF,
//! creature-template, gameplay-class or module-generation responsibilities.

use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use crate::direct_creature_animation::{
    COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1, DirectCreatureAnimationBehaviorV2,
    DirectCreatureAnimationClipLineageV2, DirectCreatureAnimationClipOriginV2,
    DirectCreatureAnimationCompletenessV2, DirectCreatureAnimationEventConformanceV1,
    DirectCreatureAnimationEventProfileV1, DirectCreatureAnimationLineageV2,
    DirectCreatureAnimationProfileV1, DirectCreatureClipEventAuthoringV1,
    DirectCreatureEventAuthoringV1, DirectCreatureEventTimingPolicyV2,
    FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1, ProceduralHumanoidKinematicsConformanceV2,
    ProceduralHumanoidRigV1, apply_direct_creature_event_authoring_v1,
    author_procedural_common_native_events_v2, author_procedural_humanoid_full_native_42_v2,
    evaluate_direct_creature_animation_behavior_v2,
    evaluate_direct_creature_event_conformance_from_inspection_v1,
    evaluate_direct_creature_event_conformance_v1, evaluate_procedural_humanoid_kinematics_v2,
};

use crate::{
    animation_studio::{
        AnimationStudioDocumentV1, AnimationStudioReadbackStatusV1, AnimationStudioReadbackV1,
        AnimationStudioRigNodeV1, AnimationStudioRigV1, AuthoredAnimationClipKindV1,
        AuthoredAnimationClipStatusV1, AuthoredAnimationSourceV1, AuthoredAnimationUsageV1,
        CreatureAnimationAuthoringV2, CustomAnimationRuntimeExposureV1,
        MaterializedCreatureAnimationAuthoringV2, evaluate_edited_animation_conformance_v1,
        fingerprint_animation_studio_document_v1, fingerprint_creature_animation_authoring_v2,
        materialize_creature_animation_authoring_v2, validate_creature_animation_authoring_v2,
    },
    creature_animation_mapping::{
        AnimationSourceKindV1, AuthoredAnimationConformanceV1, CreatureAnimationAuthoringV1,
        ResolvedCreatureAnimationMappingV1, evaluate_authored_animation_conformance_v1,
        materialize_authored_direct_creature_clips_v1, resolve_creature_animation_mapping_v1,
        validate_creature_animation_authoring_v1,
    },
    direct_creature_contract::{
        DirectCreatureRuntimeProfileV2, SourceTopologyBindingV1,
        direct_creature_runtime_profile_digest_v2, inspect_m0_source_topology_binding_v1,
        verify_m0_source_topology_binding_v1,
    },
    erf::ErfArchive,
    glb::{
        EmbeddedImageDecodeLimitsV1, GlbIngestResult, GlbLimits, decode_embedded_image_to_tga_v1,
        ingest_glb,
    },
    hak::{HakResourceInputV1, HakWriterOptionsV1, HakWriterReportV1},
    held_weapon::{
        HeldWeaponAttachmentV1, HeldWeaponBakeReportV1, bake_held_weapon_attachment_v1,
        inspect_held_weapon_source_v1,
    },
    mdl::{
        DirectCreatureEngineEnvelopeV1, DirectCreatureStructuralSummaryV1, MdlAnimationClipV1,
        MdlAnimationInterpolationV1, MdlAnimationSetV1, MdlAnimationTrackPathV1,
        MdlAnimationTrackV1, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlStateProjectionProfileV1, MdlStateProjectionProvenanceV1, MdlWriterOptionsV1,
        MdlWriterReportV1, NWN_EE_BINARY_MDL_EPSILON_V1, NodeReport,
        direct_creature_structural_summary_digest_v1, evaluate_skin_deformation_v1,
        inspect_binary_mdl, inspect_direct_creature_engine_envelope_v1,
        verify_direct_creature_state_projection_v1, write_binary_mdl_with_animations,
        write_binary_mdl_with_animations_exact_face_planes_v1,
    },
    model_limits::MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_CEILING_V1,
    model_segmentation::segment_model_for_binary_mdl_v1,
    owned_fixture::{synthetic_owned_m6_animation_mapping_v1, synthetic_owned_m6_rig_v1},
    package::{PackageManifestV1, write_model_package_v1},
    placeable::static_placeable_profile_a_options_v1,
    profile_a::{
        AuroraCreatureIrV1, CreatureRigProfileV1, ProfileAAnimationMappingV1,
        ProfileAConversionReportV1, RigProvenanceV1, RigSegmentDeformationV1,
        canonical_profile_sha256, convert_profile_a, convert_profile_a_with_animations_exact_v1,
        convert_profile_a_with_animations_p100k_experiment_v1,
        convert_profile_a_with_animations_p300k_experiment_v1,
        convert_profile_a_with_animations_v1, derive_meshy_h1_profile_and_mapping_exact_v1,
        derive_meshy_h1_profile_and_mapping_p100k_experiment_v1,
        derive_meshy_h1_profile_and_mapping_p300k_experiment_v1,
        derive_meshy_h1_profile_and_mapping_v1, derive_meshy_m0_static_rigid_profile_v1,
    },
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureRuntimeProfileV2,
        BinaryM0VerticalSliceIdentityV1, BinaryM0VerticalSliceReadbackV1, ProofModuleArtifactV1,
        ProofModuleReportV1, build_binary_m0_vertical_slice_module_v1,
        build_binary_m0_vertical_slice_module_with_identity_v1,
        build_canonical_m0_creature_proof_module_v1, build_creature_proof_module_v1,
        build_single_profiled_creature_proof_module_with_identity_v3,
        inspect_binary_creature_profile_matrix_module_v2,
        inspect_binary_m0_vertical_slice_module_v1,
    },
    skin_accessory::{
        SkinAccessoryStabilizationOptionsV1, SkinAccessoryStabilizationReportV1,
        audit_and_stabilize_skin_accessories_v1,
    },
    tga::{
        TextureArtifactCleanupOptionsV1, TextureArtifactCleanupReportV1, TgaArtifactV1,
        TgaWriterOptionsV1, TgaWriterReportV1, cleanup_texture_artifacts_v1, write_tga_v1,
    },
    two_da::{
        TwoDaAppendReportV1, TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1,
        TwoDaInspectionV1, TwoDaLimitsV1, append_two_da_row_v1, clone_two_da_row_request_v1,
        inspect_two_da_v2, read_two_da_row_v2, retain_two_da_row_prefix_v1,
    },
};

pub const M6_MODEL_RESREF: &str = "m2a_m6p01";
pub const M6_TEXTURE_RESREF: &str = "m2a_m6t01";
pub const M6_APPEARANCE_LABEL: &str = "M2A_M6_PROOF";
/// Companion HAK for the one canonical Codex animation-proof module.
pub const M6_HAK_FILE_NAME: &str = "m2a_codex_aproof.hak";
pub const M6_PROOF_MODULE_FILE_NAME: &str = "m2a_codex_aproof.mod";
pub const M6_MANIFEST_FILE_NAME: &str = "materialization-manifest.json";
pub const MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1: usize = 100_000;
const MESHY_CREATURE_P100K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1: usize =
    MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_CEILING_V1;
pub const MESHY_CREATURE_P300K_EXPERIMENT_TRIANGLE_COUNT_V1: usize = 300_000;
const MESHY_CREATURE_P300K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1: usize = 330_000;
const DIRECT_CREATURE_APPEARANCE_DONOR_ROW_V1: u32 = 102;
const DIRECT_CREATURE_APPEARANCE_DONOR_RACE_V1: &str = "c_horror";
const DIRECT_CREATURE_RUNTIME_APPEARANCE_COLUMNS_V1: [&str; 35] = [
    "LABEL",
    "STRING_REF",
    "NAME",
    "RACE",
    "ENVMAP",
    "BLOODCOLR",
    "MODELTYPE",
    "WEAPONSCALE",
    "WING_TAIL_SCALE",
    "HELMET_SCALE_M",
    "HELMET_SCALE_F",
    "MOVERATE",
    "WALKDIST",
    "RUNDIST",
    "PERSPACE",
    "CREPERSPACE",
    "HEIGHT",
    "HITDIST",
    "PREFATCKDIST",
    "TARGETHEIGHT",
    "ABORTONPARRY",
    "RACIALTYPE",
    "HASLEGS",
    "HASARMS",
    "PORTRAIT",
    "SIZECATEGORY",
    "PERCEPTIONDIST",
    "FOOTSTEPTYPE",
    "SOUNDAPPTYPE",
    "HEADTRACK",
    "HEAD_ARC_H",
    "HEAD_ARC_V",
    "HEAD_NAME",
    "BODY_BAG",
    "TARGETABLE",
];

/// Runtime semantics applied after the structurally eligible direct-model
/// donor row has been cloned.
///
/// The donor proves the Aurora `MODELTYPE=S` shape of the row. It does not
/// define the gameplay identity of a generated humanoid creature.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectCreatureAppearanceSemanticProfileV2 {
    /// Historical compatibility lane: preserves every donor cell except the
    /// generated label and model resref.
    LegacyDirectMonsterDonorV1,
    /// Medium biped values copied from the stock Human runtime row, with
    /// `MODELTYPE=S` and the generated direct-model resref retained.
    HumanoidMediumV1,
}

/// Exact immutable resource identity for a production procedural creature
/// package. Historical M6 entry points retain their fixed compatibility names;
/// fresh proof candidates must supply this identity explicitly so the model,
/// texture, appearance row, HAK, MOD, Area and UTC cannot collide with an
/// earlier payload.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProceduralCreaturePackageIdentityV1 {
    pub model_resref: String,
    pub texture_resref: String,
    pub module: BinaryCreatureModuleIdentityV1,
    pub creature_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectBuildIdentityV1 {
    pub schema_version: u32,
    pub project_id: String,
    pub project_name: String,
    pub project_revision: u64,
}

impl ProjectBuildIdentityV1 {
    pub fn namespace_resref_v1(&self) -> Result<String, M6PipelineErrorV1> {
        self.validate_v1()?;
        let mut digest = Sha256::new();
        digest.update(b"MESHY2AURORA_PROJECT_BUILD_IDENTITY_V1\0");
        digest.update(self.project_id.as_bytes());
        digest.update([0]);
        digest.update(self.project_name.as_bytes());
        digest.update([0]);
        digest.update(self.project_revision.to_le_bytes());
        let token = digest
            .finalize()
            .iter()
            .take(7)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let resref = format!("m2{token}");
        debug_assert_eq!(resref.len(), 16);
        Ok(resref)
    }

    pub fn runtime_identity_v1(
        &self,
    ) -> Result<ProceduralCreaturePackageIdentityV1, M6PipelineErrorV1> {
        let resref = self.namespace_resref_v1()?;
        Ok(ProceduralCreaturePackageIdentityV1 {
            model_resref: resref.clone(),
            texture_resref: resref.clone(),
            module: BinaryCreatureModuleIdentityV1 {
                module_resref: resref.clone(),
                area_resref: resref.clone(),
                hak_resref: resref.clone(),
            },
            creature_resref: resref,
        })
    }

    fn validate_v1(&self) -> Result<(), M6PipelineErrorV1> {
        if self.schema_version != 1 {
            return Err(project_identity_error(
                "projectIdentity.schemaVersion",
                "project build identity schemaVersion must be 1",
            ));
        }
        if self.project_id.len() < 3
            || self.project_id.len() > 128
            || !self.project_id.bytes().enumerate().all(|(index, byte)| {
                byte.is_ascii_alphanumeric()
                    || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'-'))
            })
        {
            return Err(project_identity_error(
                "projectIdentity.projectId",
                "projectId must match the portable 3..128 character project identity contract",
            ));
        }
        if self.project_name.trim().is_empty() || self.project_name.chars().count() > 120 {
            return Err(project_identity_error(
                "projectIdentity.projectName",
                "projectName must contain 1..120 non-whitespace Unicode characters",
            ));
        }
        if self.project_revision == 0 {
            return Err(project_identity_error(
                "projectIdentity.projectRevision",
                "projectRevision must be a positive integer",
            ));
        }
        Ok(())
    }
}

fn project_identity_error(path: &str, message: &str) -> M6PipelineErrorV1 {
    pipeline_error(
        "project",
        "M2A-PROJECT-BUILD-IDENTITY-INVALID",
        path,
        message,
    )
}

/// Collision-free identity of the production resources only. A MOD, Area and
/// UTC belong to a separate demo/proof request and are intentionally absent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProceduralCreatureProductIdentityV2 {
    pub model_resref: String,
    pub texture_resref: String,
    pub hak_resref: String,
    pub appearance_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProceduralCreatureBuildOptionsV1 {
    pub schema_version: u32,
    pub texture_artifact_cleanup: bool,
    #[serde(default)]
    pub skin_accessory_stabilization: SkinAccessoryStabilizationOptionsV1,
}

impl Default for ProceduralCreatureBuildOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            texture_artifact_cleanup: false,
            skin_accessory_stabilization: SkinAccessoryStabilizationOptionsV1::default(),
        }
    }
}

impl ProceduralCreaturePackageIdentityV1 {
    fn historical_default() -> Self {
        Self {
            model_resref: M6_MODEL_RESREF.to_owned(),
            texture_resref: M6_TEXTURE_RESREF.to_owned(),
            module: BinaryCreatureModuleIdentityV1 {
                module_resref: crate::proof_module::PROOF_MODULE_RESREF.to_owned(),
                area_resref: crate::proof_module::PROOF_AREA_RESREF.to_owned(),
                hak_resref: crate::proof_module::PROOF_HAK_RESREF.to_owned(),
            },
            creature_resref: crate::proof_module::PROOF_CREATURE_RESREF.to_owned(),
        }
    }
}

impl ProceduralCreatureProductIdentityV2 {
    fn from_legacy(identity: &ProceduralCreaturePackageIdentityV1) -> Self {
        Self {
            model_resref: identity.model_resref.clone(),
            texture_resref: identity.texture_resref.clone(),
            hak_resref: identity.module.hak_resref.clone(),
            appearance_label: M6_APPEARANCE_LABEL.to_owned(),
        }
    }
}

fn complete_direct_creature_appearance_request_v2(
    appearance_two_da: &[u8],
    inspection: &TwoDaInspectionV1,
    label: &str,
    race: &str,
    error_prefix: &str,
    semantic_profile: DirectCreatureAppearanceSemanticProfileV2,
) -> Result<TwoDaAppendRequestV1, M6PipelineErrorV1> {
    for required in DIRECT_CREATURE_RUNTIME_APPEARANCE_COLUMNS_V1 {
        if !inspection
            .columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(required))
        {
            return Err(pipeline_error(
                "appearance",
                format!("{error_prefix}-APPEARANCE-COLUMN-MISSING"),
                format!("appearance.columns.{required}"),
                format!(
                    "appearance.2da requires the complete direct-creature runtime column {required}"
                ),
            ));
        }
    }

    let donor_row = if inspection.physical_row_count > DIRECT_CREATURE_APPEARANCE_DONOR_ROW_V1 {
        DIRECT_CREATURE_APPEARANCE_DONOR_ROW_V1
    } else {
        0
    };
    let donor = read_two_da_row_v2(appearance_two_da, donor_row, &TwoDaLimitsV1::default())
        .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
    let donor_cell = |column_name: &str| {
        inspection
            .columns
            .iter()
            .position(|column| column.eq_ignore_ascii_case(column_name))
            .and_then(|index| donor.cells.get(index))
    };
    let donor_is_verified_direct_s = matches!(
        donor_cell("MODELTYPE"),
        Some(TwoDaCellValueV1::Text { value }) if value == "S"
    ) && matches!(
        donor_cell("RACE"),
        Some(TwoDaCellValueV1::Text { value })
            if value.eq_ignore_ascii_case(DIRECT_CREATURE_APPEARANCE_DONOR_RACE_V1)
    ) && matches!(
        donor_cell("TARGETABLE"),
        Some(TwoDaCellValueV1::Text { value }) if value == "1"
    );
    if !donor_is_verified_direct_s {
        return Err(pipeline_error(
            "appearance",
            format!("{error_prefix}-APPEARANCE-DONOR-INELIGIBLE"),
            format!("appearance.rows[{donor_row}]"),
            format!(
                "direct-creature output requires a complete stock row {donor_row} donor with MODELTYPE=S, RACE={DIRECT_CREATURE_APPEARANCE_DONOR_RACE_V1} and TARGETABLE=1"
            ),
        ));
    }

    let text_assignment = |column_name: &str, value: &str| TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text {
            value: value.to_owned(),
        },
    };
    let mut assignments = vec![
        text_assignment("LABEL", label),
        text_assignment("RACE", race),
    ];
    if semantic_profile == DirectCreatureAppearanceSemanticProfileV2::HumanoidMediumV1 {
        assignments.extend([
            TwoDaCellAssignmentV1 {
                column_name: "STRING_REF".to_owned(),
                value: TwoDaCellValueV1::Null,
            },
            text_assignment("NAME", label),
            text_assignment("ENVMAP", "default"),
            text_assignment("BLOODCOLR", "R"),
            text_assignment("MODELTYPE", "S"),
            text_assignment("WEAPONSCALE", "1"),
            text_assignment("WING_TAIL_SCALE", "1"),
            text_assignment("HELMET_SCALE_M", "1.05"),
            text_assignment("HELMET_SCALE_F", "0.85"),
            text_assignment("MOVERATE", "NORM"),
            text_assignment("WALKDIST", "1.6"),
            text_assignment("RUNDIST", "3.2"),
            text_assignment("PERSPACE", "0.3"),
            text_assignment("CREPERSPACE", "0.5"),
            text_assignment("HEIGHT", "2"),
            text_assignment("HITDIST", "0.3"),
            text_assignment("PREFATCKDIST", "1.7"),
            text_assignment("TARGETHEIGHT", "H"),
            text_assignment("ABORTONPARRY", "1"),
            text_assignment("RACIALTYPE", "11"),
            text_assignment("HASLEGS", "1"),
            text_assignment("HASARMS", "1"),
            TwoDaCellAssignmentV1 {
                column_name: "PORTRAIT".to_owned(),
                value: TwoDaCellValueV1::Null,
            },
            text_assignment("SIZECATEGORY", "3"),
            text_assignment("PERCEPTIONDIST", "9"),
            text_assignment("FOOTSTEPTYPE", "0"),
            text_assignment("SOUNDAPPTYPE", "0"),
            text_assignment("HEADTRACK", "1"),
            text_assignment("HEAD_ARC_H", "60"),
            text_assignment("HEAD_ARC_V", "30"),
            text_assignment("HEAD_NAME", "head_g"),
            text_assignment("BODY_BAG", "6"),
            text_assignment("TARGETABLE", "1"),
        ]);
    }

    clone_two_da_row_request_v1(
        appearance_two_da,
        donor_row,
        &assignments,
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))
}

/// M0 is a deliberately separate static Meshy control.  Its names do not
/// overlap with H1, so creating a new runtime packet can never replace the H1
/// binary model or its proof module.
pub const M0_MODEL_RESREF: &str = "m2a_m0p01";
pub const M0_TEXTURE_RESREF: &str = "m2a_m0t01";
pub const M0_APPEARANCE_LABEL: &str = "M2A_M0_MESHY_RIGID";
/// Retained for compatibility with previously emitted M0 control packets.
/// New M0 packages intentionally have no external tortoise dependency.
pub const M0_CONTROL_APPEARANCE_LABEL: &str = "M2A_CTRL_TORTOISE";
pub const M0_HAK_FILE_NAME: &str = "m2a_m0_proof.hak";
pub const M0_PROOF_MODULE_FILE_NAME: &str = "m2a_bm0p1.mod";

// The live NWN Toolset exposes appearance rows 0 through 847.  Rows after
// that point in the recovered input are not selectable in Creature
// Properties, so an M0 runtime HAK must derive its own visible-table payload
// before appending the Meshy row.
const AURORA_TOOLSET_VISIBLE_APPEARANCE_ROWS_V1: u32 = 848;

#[derive(Clone, Copy)]
enum M0PackageContractV1 {
    IsolatedControl,
    Canonical,
}

/// Declares whether an emitted M0 `appearance.2da` is only an isolated
/// Toolset vertical slice or a complete runtime table.  This is persisted in
/// the packet instead of being inferred from a free-form policy string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum M0AppearanceTableScopeV1 {
    IsolatedToolsetVerticalSlice,
    FullRuntimeAppendV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0AppearanceTableBindingV1 {
    pub schema_version: u32,
    pub scope: M0AppearanceTableScopeV1,
    pub input_physical_rows: u32,
    pub output_physical_rows: u32,
    pub appended_physical_row: u16,
    pub input_byte_length: u64,
    pub output_byte_length: u64,
    pub input_sha256: String,
    pub output_sha256: String,
    pub source_prefix_preserved: bool,
}

/// Rejects Toolset-only table slices at a general NWN runtime boundary.
pub fn require_m0_full_runtime_appearance_table_v1(
    table: &M0AppearanceTableBindingV1,
) -> Result<(), M6PipelineErrorV1> {
    if table.schema_version != 1 || table.scope != M0AppearanceTableScopeV1::FullRuntimeAppendV1 {
        return Err(pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-SCOPE-NOT-RUNTIME",
            "m0AppearanceTable.scope",
            "general NWN runtime packaging requires FULL_RUNTIME_APPEND",
        ));
    }
    if table.output_physical_rows != table.input_physical_rows.saturating_add(1) {
        return Err(pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-ROW-COUNT-DIFF",
            "m0AppearanceTable.outputPhysicalRows",
            "a full runtime table must preserve every input row and append exactly one row",
        ));
    }
    if !table.source_prefix_preserved
        || table.output_byte_length <= table.input_byte_length
        || table.appended_physical_row as u32 != table.input_physical_rows
        || table.input_sha256.len() != 64
        || table.output_sha256.len() != 64
    {
        return Err(pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-PREFIX-BINDING-INVALID",
            "m0AppearanceTable",
            "full runtime binding must preserve and hash the complete input prefix before one physical append",
        ));
    }
    Ok(())
}

/// Recomputes the full-runtime append binding from the exact emitted table.
/// The original input is represented by its byte length and SHA; the verifier
/// hashes that exact output prefix and never trusts only a free-form policy.
pub fn verify_m0_full_runtime_appearance_table_binding_v1(
    table: &M0AppearanceTableBindingV1,
    output: &[u8],
    fixture_physical_row: u16,
) -> Result<(), M6PipelineErrorV1> {
    require_m0_full_runtime_appearance_table_v1(table)?;
    if output.len() as u64 != table.output_byte_length
        || identity(output).sha256 != table.output_sha256
    {
        return Err(pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-OUTPUT-MISMATCH",
            "appearance.2da",
            "runtime appearance bytes do not match the bound output length/hash",
        ));
    }
    let prefix_length = usize::try_from(table.input_byte_length).map_err(|_| {
        pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-PREFIX-MISMATCH",
            "m0AppearanceTable.inputByteLength",
            "bound appearance prefix length does not fit this platform",
        )
    })?;
    let prefix = output.get(..prefix_length).ok_or_else(|| {
        pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-PREFIX-MISMATCH",
            "appearance.2da",
            "bound input prefix escapes the emitted appearance table",
        )
    })?;
    if identity(prefix).sha256 != table.input_sha256 {
        return Err(pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-PREFIX-MISMATCH",
            "appearance.2da",
            "emitted table does not preserve the exact complete input prefix",
        ));
    }
    let inspection = inspect_two_da_v2(output, &TwoDaLimitsV1::default())
        .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
    if inspection.physical_row_count != table.output_physical_rows
        || fixture_physical_row != table.appended_physical_row
        || u32::from(fixture_physical_row) + 1 != inspection.physical_row_count
    {
        return Err(pipeline_error(
            "appearance",
            "M0-APPEARANCE-TABLE-PHYSICAL-ROW-MISMATCH",
            "binaryScene.fixture.appearanceRow",
            "fixture must bind the one physical row appended after the complete runtime table",
        ));
    }
    Ok(())
}

// Aurora's ASCII-model parser reads `animroot` (decompiled_all.c:886491-886499),
// and the local direct-creature reference `c_squirrel` supplies these states.
// When Meshy supplies one source clip, reuse that owned clip under the minimum
// lifecycle names so NWN can resolve spawn, idle and locomotion.
// Aurora-first direct-creature floor: spawn/idle/movement plus the basic
// combat, damage and death states observed in the local runtime contract.
// Missing source states are clean-room aliases of the user-provided idle clip.
const M6_REQUIRED_DIRECT_CREATURE_CLIPS: [&str; 7] = [
    "cappear",
    "cpause1",
    "cwalk",
    "crun",
    "ca1slashl",
    "cdamagel",
    "cdead",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6PipelineErrorV1 {
    pub schema_version: u32,
    pub stage: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for M6PipelineErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} [{}] at {}: {}",
            self.code, self.stage, self.path, self.message
        )
    }
}

impl std::error::Error for M6PipelineErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6ByteIdentityV1 {
    pub byte_length: u64,
    pub sha256: String,
}

/// A named generated resource bound to the exact bytes that a live proof must
/// install.  File-system paths are intentionally excluded: they are
/// environment state, not generated-package provenance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0RuntimeResourceBindingV1 {
    pub resref: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0RuntimeAppearanceBindingV1 {
    pub physical_row: u16,
    pub label: String,
    pub model_type: String,
    pub race: String,
}

/// Static eligibility facts read from the emitted binary MDL.  `eligible`
/// means only that the model clears the identified native mesh gate; it is
/// never a substitute for an Aurora or NWN visual capture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0RuntimeMeshEligibilityV1 {
    pub eligible: bool,
    pub checked_mesh_count: u32,
    pub eligible_mesh_count: u32,
    pub mesh_types: Vec<u32>,
    pub texture_resrefs: Vec<String>,
    pub rule: String,
}

/// Immutable M0 binary-vertical-slice provenance packet.  The scene portion
/// is parsed from MOD bytes; resource identities are calculated from the
/// HAK/MOD payloads read back from the generated package.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M0RuntimeFixtureContractV2 {
    pub schema_version: u32,
    pub lane: String,
    pub runtime_profile: DirectCreatureRuntimeProfileV2,
    pub runtime_profile_sha256: String,
    pub module: M0RuntimeResourceBindingV1,
    pub hak: M0RuntimeResourceBindingV1,
    pub model: M0RuntimeResourceBindingV1,
    pub texture: M0RuntimeResourceBindingV1,
    pub appearance_two_da: M0RuntimeResourceBindingV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_projection_provenance: Option<MdlStateProjectionProvenanceV1>,
    pub state_projection_summary: DirectCreatureStructuralSummaryV1,
    pub state_projection_summary_sha256: String,
    pub source_topology_binding: SourceTopologyBindingV1,
    pub engine_envelope: DirectCreatureEngineEnvelopeV1,
    pub engine_envelope_sha256: String,
    pub appearance_table: M0AppearanceTableBindingV1,
    pub appearance: M0RuntimeAppearanceBindingV1,
    pub binary_scene: BinaryM0VerticalSliceReadbackV1,
    pub mesh_eligibility: M0RuntimeMeshEligibilityV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6OutputIdentitiesV1 {
    pub model: M6ByteIdentityV1,
    pub texture: M6ByteIdentityV1,
    pub appearance_two_da: M6ByteIdentityV1,
    pub hak: M6ByteIdentityV1,
    pub proof_module: M6ByteIdentityV1,
    pub report: M6ByteIdentityV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6AnimationSummaryV1 {
    pub source_name: String,
    pub output_name: String,
    pub duration_seconds: f32,
    pub has_motion: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6TextureSelectionV1 {
    pub material_slot: u32,
    pub source_material_id: u32,
    pub source_texture_id: u32,
    pub source_image_id: u32,
    pub source_image_index: usize,
    pub source_image_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6GeneratedFileV1 {
    pub relative_path: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6MaterializationManifestV1 {
    pub schema_version: u32,
    pub status: String,
    pub input_glb: M6ByteIdentityV1,
    pub input_appearance_two_da: M6ByteIdentityV1,
    pub texture_selection: M6TextureSelectionV1,
    pub appended_physical_row: u16,
    pub generated_files: Vec<M6GeneratedFileV1>,
    pub package_manifest: PackageManifestV1,
    pub appearance_payload_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_identity: Option<ProjectBuildIdentityV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m0_appearance_table: Option<M0AppearanceTableBindingV1>,
    pub manifest_self_hash_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m0_runtime_fixture_contract: Option<M0RuntimeFixtureContractV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_authoring: Option<ResolvedCreatureAnimationMappingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_animation_conformance: Option<AuthoredAnimationConformanceV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M6MaterializationSummaryV1 {
    pub schema_version: u32,
    pub status: String,
    pub input_glb: M6ByteIdentityV1,
    pub input_appearance_two_da: M6ByteIdentityV1,
    pub outputs: M6OutputIdentitiesV1,
    pub appended_physical_row: u16,
    pub model_resref: String,
    pub texture_resref: String,
    pub animation: M6AnimationSummaryV1,
    pub provenance: RigProvenanceV1,
    pub zero_reference_model_payload_copied: bool,
    pub appearance_payload_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m0_appearance_table: Option<M0AppearanceTableBindingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m0_runtime_fixture_contract: Option<M0RuntimeFixtureContractV2>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeldWeaponProductReadbackV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub attachment_fingerprint_sha256: String,
    pub texture_resref: String,
    pub texture: M6ByteIdentityV1,
    pub bake: HeldWeaponBakeReportV1,
    pub attachment_node_present_in_binary_mdl: bool,
    pub combined_triangle_count_match: bool,
    pub texture_payload_match_in_hak: bool,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6MaterializationReportV1 {
    pub schema_version: u32,
    pub resolved_base_color_image_index: usize,
    pub texture_selection: M6TextureSelectionV1,
    pub geometry: M6GeometryReportV1,
    pub ingest: crate::glb::GlbInspectionReport,
    pub conversion: ProfileAConversionReportV1,
    pub model: MdlWriterReportV1,
    pub texture: TgaWriterReportV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_artifact_cleanup: Option<TextureArtifactCleanupReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin_accessory_stabilization: Option<SkinAccessoryStabilizationReportV1>,
    pub appearance: TwoDaAppendReportV1,
    pub hak: HakWriterReportV1,
    pub proof_module: ProofModuleReportV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_completeness: Option<DirectCreatureAnimationCompletenessV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_behavior: Option<DirectCreatureAnimationBehaviorV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_kinematics_conformance: Option<ProceduralHumanoidKinematicsConformanceV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_event_conformance: Option<DirectCreatureAnimationEventConformanceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_event_timing_policy: Option<DirectCreatureEventTimingPolicyV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_event_authoring_canonical: Option<M6ByteIdentityV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin_animation_conformance: Option<M6SkinAnimationConformanceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m0_runtime_fixture_contract: Option<M0RuntimeFixtureContractV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_authoring: Option<ResolvedCreatureAnimationMappingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_animation_conformance: Option<AuthoredAnimationConformanceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held_weapon: Option<HeldWeaponProductReadbackV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6GeometryReportV1 {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub output_segment_deformation: String,
    pub active_joint_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6ClipSkinAnimationConformanceV1 {
    pub clip_name: String,
    pub sampled_time_count: u32,
    pub max_moved_vertex_count: u32,
    pub max_displacement: f32,
    pub non_rigid_deformation_observed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6SkinAnimationConformanceV1 {
    pub schema_version: u32,
    pub active_joint_count: u32,
    pub required_clip_count: u32,
    pub clips: Vec<M6ClipSkinAnimationConformanceV1>,
    pub complete: bool,
    pub violations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct M6ModelPackageArtifactV1 {
    pub source_glb: Vec<u8>,
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub proof_module: Vec<u8>,
    pub manifest: M6MaterializationManifestV1,
    pub package_manifest: PackageManifestV1,
    pub manifest_json: Vec<u8>,
    pub report: M6MaterializationReportV1,
    pub report_json: Vec<u8>,
    pub summary: M6MaterializationSummaryV1,
    pub summary_json: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableAnimationSourceInspectionV1 {
    pub schema_version: u32,
    pub source_revision: String,
    pub rig: AnimationStudioRigV1,
    pub animations: MdlAnimationSetV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationStudioAuthoredClipAuditV1 {
    pub id: String,
    pub output_name: String,
    pub kind: AuthoredAnimationClipKindV1,
    pub status: AuthoredAnimationClipStatusV1,
    pub revision: u64,
    pub source: AuthoredAnimationSourceV1,
    pub keyframe_count: usize,
    pub event_count: usize,
    pub usages: Vec<AuthoredAnimationUsageV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationStudioBuildAuditV1 {
    pub animation_studio_schema_version: u32,
    pub animation_studio_fingerprint_sha256: String,
    pub animation_studio_revision: u64,
    pub creature_animation_authoring_schema_version: u32,
    pub creature_animation_authoring_fingerprint_sha256: String,
    pub authored_clip_count: usize,
    pub authored_clip_ids: Vec<String>,
    pub authored_clip_output_names: Vec<String>,
    pub authored_event_count: usize,
    pub custom_assignment_count: usize,
    pub custom_runtime_exposures: Vec<CustomAnimationRuntimeExposureV1>,
    pub source_revision: String,
    pub readback_status: AnimationStudioReadbackStatusV1,
    pub animation_studio_readback: AnimationStudioReadbackV1,
    pub source_glb_unchanged: bool,
    pub authored_clips: Vec<AnimationStudioAuthoredClipAuditV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralCreatureProductOutputIdentitiesV2 {
    pub model: M6ByteIdentityV1,
    pub texture: M6ByteIdentityV1,
    pub appearance_two_da: M6ByteIdentityV1,
    pub hak: M6ByteIdentityV1,
    pub report: M6ByteIdentityV1,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6MaterializationReportV5 {
    #[serde(flatten)]
    pub base: M6MaterializationReportV1,
    #[serde(flatten)]
    pub animation_studio: AnimationStudioBuildAuditV1,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralCreatureProductReportV2 {
    pub schema_version: u32,
    pub identity: ProceduralCreatureProductIdentityV2,
    pub appearance_semantic_profile: DirectCreatureAppearanceSemanticProfileV2,
    pub resolved_base_color_image_index: usize,
    pub texture_selection: M6TextureSelectionV1,
    pub geometry: M6GeometryReportV1,
    pub ingest: crate::glb::GlbInspectionReport,
    pub conversion: ProfileAConversionReportV1,
    pub model: MdlWriterReportV1,
    pub texture: TgaWriterReportV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_artifact_cleanup: Option<TextureArtifactCleanupReportV1>,
    pub skin_accessory_stabilization: SkinAccessoryStabilizationReportV1,
    pub appearance: TwoDaAppendReportV1,
    pub hak: HakWriterReportV1,
    pub animation_completeness: DirectCreatureAnimationCompletenessV2,
    pub animation_behavior: DirectCreatureAnimationBehaviorV2,
    pub animation_kinematics_conformance: ProceduralHumanoidKinematicsConformanceV2,
    pub animation_event_conformance: DirectCreatureAnimationEventConformanceV1,
    pub animation_event_timing_policy: DirectCreatureEventTimingPolicyV2,
    pub animation_event_authoring_canonical: M6ByteIdentityV1,
    pub skin_animation_conformance: M6SkinAnimationConformanceV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6MaterializationManifestV5 {
    #[serde(flatten)]
    pub base: M6MaterializationManifestV1,
    #[serde(flatten)]
    pub animation_studio: AnimationStudioBuildAuditV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralCreatureProductSummaryV2 {
    pub schema_version: u32,
    pub status: String,
    pub input_glb: M6ByteIdentityV1,
    pub input_appearance_two_da: M6ByteIdentityV1,
    pub identity: ProceduralCreatureProductIdentityV2,
    pub outputs: ProceduralCreatureProductOutputIdentitiesV2,
    pub appended_physical_row: u16,
    pub animation: M6AnimationSummaryV1,
    pub provenance: RigProvenanceV1,
    pub zero_reference_model_payload_copied: bool,
    pub appearance_payload_policy: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M6MaterializationSummaryV5 {
    #[serde(flatten)]
    pub base: M6MaterializationSummaryV1,
    #[serde(flatten)]
    pub animation_studio: AnimationStudioBuildAuditV1,
}

#[derive(Clone, Debug)]
pub struct M6ModelPackageArtifactV5 {
    pub source_glb: Vec<u8>,
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub proof_module: Vec<u8>,
    pub manifest: M6MaterializationManifestV5,
    pub package_manifest: PackageManifestV1,
    pub manifest_json: Vec<u8>,
    pub report: M6MaterializationReportV5,
    pub report_json: Vec<u8>,
    pub summary: M6MaterializationSummaryV5,
    pub summary_json: Vec<u8>,
    pub animation_studio_readback: AnimationStudioReadbackV1,
    pub materialized_animations: MaterializedCreatureAnimationAuthoringV2,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralCreatureProductManifestV2 {
    pub schema_version: u32,
    pub status: String,
    pub input_glb: M6ByteIdentityV1,
    pub input_appearance_two_da: M6ByteIdentityV1,
    pub identity: ProceduralCreatureProductIdentityV2,
    pub texture_selection: M6TextureSelectionV1,
    pub appended_physical_row: u16,
    pub generated_files: Vec<M6GeneratedFileV1>,
    pub package_manifest: PackageManifestV1,
    pub appearance_payload_policy: String,
    pub manifest_self_hash_policy: String,
}

#[derive(Clone, Debug)]
pub struct ProceduralCreatureProductArtifactV2 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub package_manifest: PackageManifestV1,
    pub manifest: ProceduralCreatureProductManifestV2,
    pub manifest_json: Vec<u8>,
    pub report: ProceduralCreatureProductReportV2,
    pub report_json: Vec<u8>,
    pub summary: ProceduralCreatureProductSummaryV2,
    pub summary_json: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralCreatureProductDemoPacketManifestV2 {
    pub schema_version: u32,
    pub status: String,
    pub input_glb: M6ByteIdentityV1,
    pub input_appearance_two_da: M6ByteIdentityV1,
    pub product_identity: ProceduralCreatureProductIdentityV2,
    pub module_identity: BinaryCreatureModuleIdentityV1,
    pub creature_resref: String,
    pub appearance_row: u16,
    pub geometry: M6GeometryReportV1,
    pub skin_accessory_stabilization: SkinAccessoryStabilizationReportV1,
    pub generated_files: Vec<M6GeneratedFileV1>,
    pub package_manifest: PackageManifestV1,
    pub demo: ProofModuleReportV1,
    pub manifest_self_hash_policy: String,
}

enum M6BuildArtifactV2 {
    LegacyBundle(Box<M6ModelPackageInternalV5>),
    ProceduralProduct(Box<ProceduralCreatureProductArtifactV2>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum M6BuildOutputV2 {
    LegacyBundle,
    ProceduralProduct,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum M6GeometryPolicyV1 {
    ProductBudget,
    P100kSegmentedExperiment,
    P300kSegmentedExperiment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TriangleDegeneracyPolicyV1 {
    LegacyAbsoluteEpsilon,
    ExactFiniteNonCollinear,
}

impl TriangleDegeneracyPolicyV1 {
    fn accepts_cross_length(self, length: f64) -> bool {
        match self {
            Self::LegacyAbsoluteEpsilon => {
                length.is_finite() && length > f64::from(NWN_EE_BINARY_MDL_EPSILON_V1)
            }
            Self::ExactFiniteNonCollinear => length.is_finite() && length > 0.0,
        }
    }
}

fn triangle_degeneracy_policy_for_build_v1(
    output: M6BuildOutputV2,
    geometry_policy: M6GeometryPolicyV1,
) -> TriangleDegeneracyPolicyV1 {
    match (output, geometry_policy) {
        (M6BuildOutputV2::LegacyBundle, M6GeometryPolicyV1::ProductBudget) => {
            TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon
        }
        _ => TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear,
    }
}

fn p100k_experiment_triangle_count_is_eligible_v1(triangle_count: usize) -> bool {
    triangle_count > 20_000
        && triangle_count <= MESHY_CREATURE_P100K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1
}

pub fn build_m6_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let rig = synthetic_owned_m6_rig_v1()
        .map_err(|error| pipeline_error("fixture", error.code, "rig", error.message))?;
    build_m6_model_package_with_profile_v1(
        source_glb,
        appearance_two_da,
        &rig,
        &synthetic_owned_m6_animation_mapping_v1(),
    )
}

/// Runs the Studio's constrained Meshy H1 route.  The source is first
/// inspected into an owned profile/mapping derived solely from the selected
/// GLB; package materialization remains the same audited MDL/TGA/2DA/HAK
/// pipeline as the M6 proof route.
pub fn build_meshy_h1_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut source)?;
    let (rig, mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    build_m6_model_package_with_ingest_v1(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
    )
}

/// Inspects the exact browser-selected GLB into the same editable animation
/// values and output-rig node identities used by the canonical H1 writer.
pub fn inspect_editable_animation_source_v1(
    source_glb: &[u8],
) -> Result<EditableAnimationSourceInspectionV1, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_authored_h1_source_names_v1(&source, &mut mapping, false);
    let animated =
        convert_profile_a_with_animations_v1(&source, &rig, &Default::default(), &mapping)
            .map_err(|error| pipeline_error("animation", error.code, error.path, error.message))?;
    let creature = animated.base.creature.ok_or_else(|| {
        pipeline_error(
            "profile",
            "M6-PROFILE-INELIGIBLE",
            "conversion.creature",
            "editable animation inspection has no creature output",
        )
    })?;
    let animations = animated.animations.ok_or_else(|| {
        pipeline_error(
            "animation",
            "M6-ANIMATION-MISSING",
            "conversion.animations",
            "editable animation inspection has no mapped animations",
        )
    })?;
    let source_revision = identity(source_glb).sha256;
    let animation_root = animations
        .clips
        .first()
        .map(|clip| clip.animation_root.clone())
        .or_else(|| {
            creature
                .nodes
                .iter()
                .find(|node| node.parent_id.is_none())
                .map(|node| node.name.clone())
        })
        .ok_or_else(|| {
            pipeline_error(
                "animation",
                "M2A-ANIMATION-EDIT-BONE-MISSING",
                "rig.animationRoot",
                "editable animation rig has no root node",
            )
        })?;
    let studio_rig =
        animation_studio_rig_from_creature_v1(&creature, &source_revision, &animation_root)?;
    Ok(EditableAnimationSourceInspectionV1 {
        schema_version: 1,
        source_revision,
        rig: studio_rig,
        animations,
    })
}

/// Automatic H1 route with an explicit animation-completeness policy.
///
/// Full mode accepts only source animations whose own names case-insensitively
/// match the confirmed 42-state direct-creature namespace. Unknown names stay
/// outside that namespace, so the completeness gate reports every missing
/// state instead of guessing gameplay semantics from array order.
pub fn build_meshy_h1_model_package_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_profile: DirectCreatureAnimationProfileV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_exact_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_automatic_h1_animation_profile_names_v1(&source, &mut mapping, animation_profile);
    build_m6_model_package_with_ingest_v2(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        animation_profile,
    )
}

/// Builds a complete direct-creature animation set for a skinned Meshy
/// humanoid that owns `cpause1`, any caller-selected native-name source
/// motions, and the required semantic joints.
///
/// This is the production replacement for the legacy seven-state idle-alias
/// route. It preserves every explicit caller-owned native clip, authors only
/// the missing gameplay states plus their event callbacks, emits the
/// controllerless Aurora skin root, and applies the full behavior and
/// non-rigid deformation gates.
pub fn build_meshy_procedural_humanoid_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_procedural_humanoid_model_package_with_identity_v1(
        source_glb,
        appearance_two_da,
        &ProceduralCreaturePackageIdentityV1::historical_default(),
    )
}

/// Builds the complete production procedural creature package under a fresh,
/// caller-owned immutable resource identity.
pub fn build_meshy_procedural_humanoid_model_package_with_identity_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_automatic_h1_animation_profile_names_v1(
        &source,
        &mut mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
    );
    build_m6_model_package_with_ingest_v4(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
        None,
        runtime_identity,
    )
}

/// Builds only the production creature resources. It cannot create a MOD,
/// Area or UTC because their identities are not part of this contract.
pub fn build_meshy_procedural_humanoid_product_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    product_identity: &ProceduralCreatureProductIdentityV2,
) -> Result<ProceduralCreatureProductArtifactV2, M6PipelineErrorV1> {
    build_meshy_procedural_humanoid_product_with_options_v3(
        source_glb,
        appearance_two_da,
        product_identity,
        &ProceduralCreatureBuildOptionsV1::default(),
    )
}

/// Builds the production procedural creature with caller-selected texture
/// processing and detached-accessory skinning stabilization.
pub fn build_meshy_procedural_humanoid_product_with_options_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    product_identity: &ProceduralCreatureProductIdentityV2,
    build_options: &ProceduralCreatureBuildOptionsV1,
) -> Result<ProceduralCreatureProductArtifactV2, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_exact_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_automatic_h1_animation_profile_names_v1(
        &source,
        &mut mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
    );
    let compatibility_identity = ProceduralCreaturePackageIdentityV1 {
        model_resref: product_identity.model_resref.clone(),
        texture_resref: product_identity.texture_resref.clone(),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: String::new(),
            area_resref: String::new(),
            hak_resref: product_identity.hak_resref.clone(),
        },
        creature_resref: String::new(),
    };
    match build_m6_model_package_with_ingest_v5(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
        None,
        &compatibility_identity,
        product_identity,
        M6BuildOutputV2::ProceduralProduct,
        M6GeometryPolicyV1::ProductBudget,
        build_options,
        None,
        None,
        None,
        None,
    )? {
        M6BuildArtifactV2::ProceduralProduct(artifact) => Ok(*artifact),
        M6BuildArtifactV2::LegacyBundle(_) => unreachable!("product output mode is exact"),
    }
}

/// Materializes the historical P100K compatibility profile. The shared product
/// route now accepts up to 300K; this entry point remains only for deterministic
/// replay of the earlier bounded 100K lineage.
///
/// The source must be a Meshy H1 animated GLB with more than 20K triangles.
/// `100K` is the Meshy generation target; a bounded 10% overshoot is accepted
/// explicitly so valid microtriangles are never deleted to force the source
/// below that target. The converted SkinMesh is deterministically partitioned
/// into binary-MDL-safe streams before writing.
pub fn build_meshy_procedural_humanoid_p100k_experiment_with_identity_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_procedural_humanoid_p100k_experiment_with_options_v2(
        source_glb,
        appearance_two_da,
        runtime_identity,
        &ProceduralCreatureBuildOptionsV1::default(),
    )
}

pub fn build_meshy_procedural_humanoid_p100k_experiment_with_options_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
    build_options: &ProceduralCreatureBuildOptionsV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let experiment_limits = p100k_experiment_glb_limits_v1();
    let mut source = ingest_glb(source_glb, &experiment_limits).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut source)?;
    let triangle_count = source.report.statistics.triangle_count;
    if !p100k_experiment_triangle_count_is_eligible_v1(triangle_count) {
        return Err(pipeline_error(
            "ingest",
            "M6-P100K-EXPERIMENT-TRIANGLE-COUNT",
            "sourceGlb",
            format!(
                "the isolated 100K-target experiment requires 20001..={} exact finite non-collinear source triangles (requested target {}, bounded Meshy overshoot {}%), got {triangle_count}",
                MESHY_CREATURE_P100K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1,
                MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1,
                (MESHY_CREATURE_P100K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1
                    - MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1)
                    * 100
                    / MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1
            ),
        ));
    }
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_p100k_experiment_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_automatic_h1_animation_profile_names_v1(
        &source,
        &mut mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
    );
    let product_identity = ProceduralCreatureProductIdentityV2::from_legacy(runtime_identity);
    match build_m6_model_package_with_ingest_v5(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
        None,
        runtime_identity,
        &product_identity,
        M6BuildOutputV2::LegacyBundle,
        M6GeometryPolicyV1::P100kSegmentedExperiment,
        build_options,
        None,
        None,
        None,
        None,
    )? {
        M6BuildArtifactV2::LegacyBundle(built) => Ok(built.artifact),
        M6BuildArtifactV2::ProceduralProduct(_) => {
            unreachable!("the P100K experiment output mode is exact")
        }
    }
}

/// Inspects a source through the historical P100K compatibility envelope.
pub fn inspect_meshy_procedural_humanoid_p100k_experiment_v1(
    source_glb: &[u8],
) -> Result<GlbIngestResult, crate::glb::GlbFatalError> {
    ingest_glb(source_glb, &p100k_experiment_glb_limits_v1())
}

/// Materializes the historical P300K compatibility profile. New conversions
/// use the shared product profile, which now has the same 300K total budget.
pub fn build_meshy_procedural_humanoid_p300k_experiment_with_identity_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_procedural_humanoid_p300k_experiment_with_options_v2(
        source_glb,
        appearance_two_da,
        runtime_identity,
        &ProceduralCreatureBuildOptionsV1::default(),
    )
}

pub fn build_meshy_procedural_humanoid_p300k_experiment_with_options_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
    build_options: &ProceduralCreatureBuildOptionsV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let experiment_limits = p300k_experiment_glb_limits_v1();
    let mut source = ingest_glb(source_glb, &experiment_limits).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut source)?;
    let triangle_count = source.report.statistics.triangle_count;
    if triangle_count <= MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_COUNT_V1
        || triangle_count > MESHY_CREATURE_P300K_EXPERIMENT_TRIANGLE_COUNT_V1
    {
        return Err(pipeline_error(
            "ingest",
            "M6-P300K-EXPERIMENT-TRIANGLE-COUNT",
            "sourceGlb",
            format!(
                "the isolated experiment requires 100001..={} source triangles, got {triangle_count}",
                MESHY_CREATURE_P300K_EXPERIMENT_TRIANGLE_COUNT_V1
            ),
        ));
    }
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_p300k_experiment_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_automatic_h1_animation_profile_names_v1(
        &source,
        &mut mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
    );
    let product_identity = ProceduralCreatureProductIdentityV2::from_legacy(runtime_identity);
    match build_m6_model_package_with_ingest_v5(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
        None,
        runtime_identity,
        &product_identity,
        M6BuildOutputV2::LegacyBundle,
        M6GeometryPolicyV1::P300kSegmentedExperiment,
        build_options,
        None,
        None,
        None,
        None,
    )? {
        M6BuildArtifactV2::LegacyBundle(built) => Ok(built.artifact),
        M6BuildArtifactV2::ProceduralProduct(_) => {
            unreachable!("the P300K experiment output mode is exact")
        }
    }
}

/// Inspects a source through the elevated envelope owned only by the explicit
/// P300K experiment.
pub fn inspect_meshy_procedural_humanoid_p300k_experiment_v1(
    source_glb: &[u8],
) -> Result<GlbIngestResult, crate::glb::GlbFatalError> {
    ingest_glb(source_glb, &p300k_experiment_glb_limits_v1())
}

fn p100k_experiment_glb_limits_v1() -> GlbLimits {
    GlbLimits {
        max_input_bytes: 256 * 1024 * 1024,
        max_decoded_geometry_bytes: 512 * 1024 * 1024,
        max_decoded_skin_animation_bytes: 512 * 1024 * 1024,
        triangle_warning_above: 50_000,
        // Meshy treats 100K as a target and may produce a bounded amount of
        // valid geometry above it. The explicit envelope admits that variance;
        // only non-finite, zero-area or exactly collinear faces are removed.
        triangle_blocking_above: MESHY_CREATURE_P100K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1,
        ..GlbLimits::default()
    }
}

fn p300k_experiment_glb_limits_v1() -> GlbLimits {
    GlbLimits {
        max_input_bytes: 256 * 1024 * 1024,
        max_decoded_geometry_bytes: 512 * 1024 * 1024,
        max_decoded_skin_animation_bytes: 512 * 1024 * 1024,
        triangle_warning_above: 150_000,
        // Meshy may emit degenerate faces above the requested target. They are
        // removed before the exact post-sanitize 300K experiment gate.
        triangle_blocking_above: MESHY_CREATURE_P300K_EXPERIMENT_RAW_TRIANGLE_CEILING_V1,
        ..GlbLimits::default()
    }
}

/// Adds an optional fixture-only MOD/UTC around an already materialized
/// production product. The product bytes are neither rebuilt nor mutated.
pub fn build_procedural_creature_demo_v2(
    product: &ProceduralCreatureProductArtifactV2,
    module_identity: &BinaryCreatureModuleIdentityV1,
    creature_resref: &str,
) -> Result<ProofModuleArtifactV1, M6PipelineErrorV1> {
    if module_identity.hak_resref != product.report.identity.hak_resref {
        return Err(pipeline_error(
            "proof_module",
            "M6-DEMO-HAK-IDENTITY-MISMATCH",
            "moduleIdentity.hakResref",
            format!(
                "demo HAK resref {} does not match the immutable product HAK resref {}",
                module_identity.hak_resref, product.report.identity.hak_resref
            ),
        ));
    }
    build_single_profiled_creature_proof_module_with_identity_v3(
        product.report.appearance.appended_row_index,
        BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
        module_identity,
        creature_resref,
    )
    .map_err(|error| pipeline_error("proof_module", error.code, error.path, error.message))
}

/// Automatic H1 route with caller-owned event authoring and exact binary
/// readback conformance for the selected event profile.
pub fn build_meshy_h1_model_package_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_profile: DirectCreatureAnimationProfileV1,
    event_profile: DirectCreatureAnimationEventProfileV1,
    event_authoring: &DirectCreatureEventAuthoringV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    apply_automatic_h1_animation_profile_names_v1(&source, &mut mapping, animation_profile);
    build_m6_model_package_with_ingest_v3(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        animation_profile,
        Some((event_profile, event_authoring)),
    )
}

/// Additive H1 route driven by the versioned Creature Animation Mapping
/// document. Existing V2/V3 behavior is unchanged.
pub fn build_meshy_h1_model_package_v4(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v4_with_identity(
        source_glb,
        appearance_two_da,
        animation_authoring,
        &ProceduralCreaturePackageIdentityV1::historical_default(),
    )
}

/// Authored H1 route with a caller-owned immutable runtime identity.
///
/// This is the candidate-freezing boundary for an admitted proof lineage. It
/// does not allocate or install an identity and therefore cannot bypass the
/// model iteration gate.
pub fn build_meshy_h1_model_package_v4_with_identity(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV1,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v4_with_events_and_identity(
        source_glb,
        appearance_two_da,
        animation_authoring,
        None,
        runtime_identity,
    )
}

pub fn build_meshy_h1_model_package_v4_with_events(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v4_with_events_and_identity(
        source_glb,
        appearance_two_da,
        animation_authoring,
        event_authoring,
        &ProceduralCreaturePackageIdentityV1::historical_default(),
    )
}

pub fn build_meshy_h1_model_package_v4_with_events_and_identity(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v4_internal(
        source_glb,
        appearance_two_da,
        animation_authoring,
        event_authoring,
        runtime_identity,
        None,
    )
}

pub fn build_meshy_h1_model_package_v4_with_project_identity(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    project_identity: &ProjectBuildIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let runtime_identity = project_identity.runtime_identity_v1()?;
    build_meshy_h1_model_package_v4_internal(
        source_glb,
        appearance_two_da,
        animation_authoring,
        event_authoring,
        &runtime_identity,
        Some(project_identity),
    )
}

fn build_meshy_h1_model_package_v4_internal(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
    project_identity: Option<&ProjectBuildIdentityV1>,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let validation = validate_creature_animation_authoring_v1(animation_authoring);
    if validation.status
        != crate::creature_animation_mapping::CreatureAnimationMappingStatusV1::Ready
    {
        let first = validation.diagnostics.first();
        return Err(pipeline_error(
            "animation",
            first
                .map(|diagnostic| diagnostic.code.as_str())
                .unwrap_or("M2A-ANIMATION-AUTHORING-NOT-READY"),
            first
                .map(|diagnostic| diagnostic.path.as_str())
                .unwrap_or("animationAuthoring"),
            first
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("animation authoring is not ready"),
        ));
    }
    let input_identity = identity(source_glb);
    let source_revision_matches = animation_authoring.source_revision == input_identity.sha256
        || animation_authoring.source_revision == format!("sha256:{}", input_identity.sha256);
    if !source_revision_matches {
        return Err(pipeline_error(
            "animation",
            "M2A-ANIMATION-SOURCE-REVISION-STALE",
            "animationAuthoring.sourceRevision",
            "animation authoring sourceRevision does not match the exact GLB bytes",
        ));
    }
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    let uses_procedural = animation_authoring
        .assignments
        .iter()
        .any(|assignment| assignment.source_kind == AnimationSourceKindV1::Procedural);
    apply_authored_h1_source_names_v1(&source, &mut mapping, uses_procedural);
    let product_identity = ProceduralCreatureProductIdentityV2::from_legacy(runtime_identity);
    match build_m6_model_package_with_ingest_v5(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        if uses_procedural {
            DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        } else {
            DirectCreatureAnimationProfileV1::FullNative42ExplicitV1
        },
        event_authoring.map(|authoring| {
            (
                DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
                authoring,
            )
        }),
        runtime_identity,
        &product_identity,
        M6BuildOutputV2::LegacyBundle,
        M6GeometryPolicyV1::ProductBudget,
        &ProceduralCreatureBuildOptionsV1::default(),
        project_identity,
        Some(animation_authoring),
        None,
        None,
    )? {
        M6BuildArtifactV2::LegacyBundle(built) => Ok(built.artifact),
        M6BuildArtifactV2::ProceduralProduct(_) => {
            unreachable!("authored V4 output mode is exact")
        }
    }
}

pub fn build_meshy_h1_model_package_v5(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV2,
    animation_studio: &AnimationStudioDocumentV1,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v5_with_events(
        source_glb,
        appearance_two_da,
        animation_authoring,
        animation_studio,
        None,
    )
}

pub fn build_meshy_h1_model_package_v5_with_events(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV2,
    animation_studio: &AnimationStudioDocumentV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v5_internal(
        source_glb,
        appearance_two_da,
        animation_authoring,
        animation_studio,
        event_authoring,
        &ProceduralCreaturePackageIdentityV1::historical_default(),
        None,
        None,
    )
}

/// Builds the Animation Studio V5 package with one explicit, caller-owned
/// runtime identity. This is the deterministic demo/proof counterpart of the
/// project-identity entry point: the generated MDL, TGA, HAK, MOD, Area and
/// UTC all use the exact supplied resrefs, while the V5 authored-animation
/// validation and binary readback gates remain unchanged.
pub fn build_meshy_h1_model_package_v5_with_identity(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV2,
    animation_studio: &AnimationStudioDocumentV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    build_meshy_h1_model_package_v5_internal(
        source_glb,
        appearance_two_da,
        animation_authoring,
        animation_studio,
        event_authoring,
        runtime_identity,
        None,
        None,
    )
}

pub fn build_meshy_h1_model_package_v5_with_project_identity(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV2,
    animation_studio: &AnimationStudioDocumentV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    project_identity: &ProjectBuildIdentityV1,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    let runtime_identity = project_identity.runtime_identity_v1()?;
    build_meshy_h1_model_package_v5_internal(
        source_glb,
        appearance_two_da,
        animation_authoring,
        animation_studio,
        event_authoring,
        &runtime_identity,
        Some(project_identity),
        None,
    )
}

/// Final Animation Studio build with one exact rigid held-weapon GLB. The
/// weapon becomes ordinary rigid child geometry in the direct-creature MDL;
/// its single embedded base-color texture is packaged under a deterministic
/// companion resref and verified from HAK readback.
#[allow(clippy::too_many_arguments)]
pub fn build_meshy_h1_model_package_v6_with_held_weapon_project_identity(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV2,
    animation_studio: &AnimationStudioDocumentV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    project_identity: &ProjectBuildIdentityV1,
    weapon_glb: &[u8],
    attachment: &HeldWeaponAttachmentV1,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    let runtime_identity = project_identity.runtime_identity_v1()?;
    build_meshy_h1_model_package_v5_internal(
        source_glb,
        appearance_two_da,
        animation_authoring,
        animation_studio,
        event_authoring,
        &runtime_identity,
        Some(project_identity),
        Some((weapon_glb, attachment)),
    )
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_h1_model_package_v5_internal(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    animation_authoring: &CreatureAnimationAuthoringV2,
    animation_studio: &AnimationStudioDocumentV1,
    event_authoring: Option<&DirectCreatureEventAuthoringV1>,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
    project_identity: Option<&ProjectBuildIdentityV1>,
    held_weapon: Option<(&[u8], &HeldWeaponAttachmentV1)>,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    let source_identity = identity(source_glb);
    for (path, revision) in [
        (
            "animationAuthoring.sourceRevision",
            animation_authoring.source_revision.as_str(),
        ),
        (
            "animationStudio.sourceRevision",
            animation_studio.source_revision.as_str(),
        ),
    ] {
        if revision != source_identity.sha256 {
            return Err(pipeline_error(
                "animation",
                "M2A-ANIMATION-EDIT-SOURCE-STALE",
                path,
                "V5 authoring sourceRevision does not match the exact immutable GLB bytes",
            ));
        }
    }
    let validation =
        validate_creature_animation_authoring_v2(animation_authoring, animation_studio);
    if let Some(first) = validation.first() {
        return Err(pipeline_error(
            "animation",
            &first.code,
            &first.path,
            &first.message,
        ));
    }
    if event_authoring.is_some()
        && animation_studio
            .authored_clips
            .iter()
            .any(|clip| !clip.events.is_empty())
    {
        return Err(pipeline_error(
            "animation",
            "M2A-ANIMATION-EDIT-EVENT",
            "eventAuthoring",
            "external event authoring cannot be combined with integrated authored clip events",
        ));
    }

    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut source)?;
    let (rig, mut mapping) = derive_meshy_h1_profile_and_mapping_exact_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    let uses_procedural = animation_authoring
        .assignments
        .iter()
        .any(|assignment| assignment.source_kind == AnimationSourceKindV1::Procedural);
    apply_authored_h1_source_names_v1(&source, &mut mapping, uses_procedural);
    let product_identity = ProceduralCreatureProductIdentityV2::from_legacy(runtime_identity);
    let built = match build_m6_model_package_with_ingest_v5(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
        if uses_procedural {
            DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        } else {
            DirectCreatureAnimationProfileV1::FullNative42ExplicitV1
        },
        event_authoring.map(|authoring| {
            (
                DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
                authoring,
            )
        }),
        runtime_identity,
        &product_identity,
        M6BuildOutputV2::LegacyBundle,
        M6GeometryPolicyV1::ProductBudget,
        &ProceduralCreatureBuildOptionsV1::default(),
        project_identity,
        None,
        Some((animation_authoring, animation_studio)),
        held_weapon,
    )? {
        M6BuildArtifactV2::LegacyBundle(built) => *built,
        M6BuildArtifactV2::ProceduralProduct(_) => {
            unreachable!("authored V5 output mode is exact")
        }
    };
    let base = built.artifact;
    let expected = built.animation_studio_materialization.ok_or_else(|| {
        pipeline_error(
            "animation",
            "M2A-ANIMATION-EDIT-SCHEMA",
            "animationStudio",
            "V5 internal build did not preserve its exact Animation Studio materialization",
        )
    })?;
    let studio_rig = built.animation_studio_rig.ok_or_else(|| {
        pipeline_error(
            "animation",
            "M2A-ANIMATION-EDIT-SCHEMA",
            "animationStudio.rig",
            "V5 internal build did not preserve its exact Animation Studio rig",
        )
    })?;
    if base.source_glb.as_slice() != source_glb {
        return Err(pipeline_error(
            "animation",
            "M2A-ANIMATION-EDIT-SOURCE-STALE",
            "artifact.sourceGlb",
            "V5 pipeline changed the exact source GLB bytes",
        ));
    }
    let binary_readback = inspect_binary_mdl(&base.model)
        .map_err(|error| pipeline_error("readback", error.code, "model", error.context))?;
    let studio_readback = evaluate_edited_animation_conformance_v1(
        animation_studio,
        &expected,
        &studio_rig,
        &built.writer_readback,
        &binary_readback,
    );
    if studio_readback.status != AnimationStudioReadbackStatusV1::Match {
        let first = studio_readback.diagnostics.first();
        return Err(pipeline_error(
            "readback",
            first
                .map(|diagnostic| diagnostic.code.as_str())
                .unwrap_or("M2A-ANIMATION-EDIT-READBACK-MISMATCH"),
            first
                .map(|diagnostic| diagnostic.path.as_str())
                .unwrap_or("readback.animations"),
            first
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("authored animation binary readback mismatch"),
        ));
    }
    finish_animation_studio_v5_artifact(
        base,
        animation_authoring,
        animation_studio,
        expected,
        studio_readback,
    )
}

fn apply_authored_h1_source_names_v1(
    source: &GlbIngestResult,
    mapping: &mut ProfileAAnimationMappingV1,
    preserve_procedural_idle: bool,
) {
    let procedural_idle_source_id = if preserve_procedural_idle {
        source
            .ir
            .animations
            .iter()
            .find(|animation| {
                animation
                    .name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case("cpause1"))
            })
            .map(|animation| animation.id)
            .or_else(|| {
                mapping
                    .clip_mappings
                    .iter()
                    .find(|clip_mapping| {
                        clip_mapping
                            .output_clip_name
                            .eq_ignore_ascii_case("cpause1")
                    })
                    .map(|clip_mapping| clip_mapping.source_animation_id)
            })
    } else {
        None
    };
    for (index, clip_mapping) in mapping.clip_mappings.iter_mut().enumerate() {
        clip_mapping.output_clip_name =
            if procedural_idle_source_id == Some(clip_mapping.source_animation_id) {
                "cpause1".to_owned()
            } else {
                source
                    .ir
                    .animations
                    .iter()
                    .find(|animation| animation.id == clip_mapping.source_animation_id)
                    .and_then(|animation| animation.name.clone())
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| format!("m2a_source_{index}"))
            };
    }
}

fn first_animation_studio_pipeline_error(
    diagnostics: Vec<crate::animation_studio::AnimationStudioDiagnosticV1>,
) -> M6PipelineErrorV1 {
    diagnostics.first().map_or_else(
        || {
            pipeline_error(
                "animation",
                "M2A-ANIMATION-EDIT-SCHEMA",
                "animationStudio",
                "Animation Studio materialization failed without a diagnostic",
            )
        },
        |diagnostic| {
            pipeline_error(
                "animation",
                &diagnostic.code,
                &diagnostic.path,
                &diagnostic.message,
            )
        },
    )
}

fn animation_studio_rig_from_creature_v1(
    creature: &AuroraCreatureIrV1,
    source_revision: &str,
    animation_root: &str,
) -> Result<AnimationStudioRigV1, M6PipelineErrorV1> {
    let nodes = creature
        .nodes
        .iter()
        .map(|node| {
            let (translation, rotation) =
                decompose_animation_studio_bind_v1(node.bind_local_matrix).map_err(|message| {
                    pipeline_error(
                        "animation",
                        "M2A-ANIMATION-EDIT-QUATERNION",
                        format!("rig.nodes[{}].bindLocalMatrix", node.id),
                        message,
                    )
                })?;
            Ok(AnimationStudioRigNodeV1 {
                node_id: node.id,
                name: node.name.clone(),
                parent_id: node.parent_id,
                translation,
                rotation,
            })
        })
        .collect::<Result<Vec<_>, M6PipelineErrorV1>>()?;
    Ok(AnimationStudioRigV1 {
        schema_version: 1,
        source_revision: source_revision.to_owned(),
        animation_root: animation_root.to_owned(),
        nodes,
    })
}

fn decompose_animation_studio_bind_v1(
    matrix: [f32; 16],
) -> Result<([f32; 3], [f32; 4]), &'static str> {
    if matrix.iter().any(|value| !value.is_finite()) {
        return Err("bind matrix contains a non-finite value");
    }
    let mut columns = [
        [matrix[0], matrix[1], matrix[2]],
        [matrix[4], matrix[5], matrix[6]],
        [matrix[8], matrix[9], matrix[10]],
    ];
    for column in &mut columns {
        let length = (column[0] * column[0] + column[1] * column[1] + column[2] * column[2]).sqrt();
        if !length.is_finite() || length <= 1.0e-8 {
            return Err("bind matrix rotation basis is singular");
        }
        for value in column {
            *value /= length;
        }
    }
    let m00 = columns[0][0];
    let m01 = columns[1][0];
    let m02 = columns[2][0];
    let m10 = columns[0][1];
    let m11 = columns[1][1];
    let m12 = columns[2][1];
    let m20 = columns[0][2];
    let m21 = columns[1][2];
    let m22 = columns[2][2];
    let trace = m00 + m11 + m22;
    let (x, y, z, w) = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        ((m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s)
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        (0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s)
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        ((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s)
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        ((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s)
    };
    let norm = (x * x + y * y + z * z + w * w).sqrt();
    if !norm.is_finite() || norm <= 1.0e-8 {
        return Err("bind matrix produced an invalid rotation quaternion");
    }
    let mut rotation = [x / norm, y / norm, z / norm, w / norm];
    if rotation[3] < 0.0 {
        for component in &mut rotation {
            *component = -*component;
        }
    }
    Ok(([matrix[12], matrix[13], matrix[14]], rotation))
}

fn finish_animation_studio_v5_artifact(
    base: M6ModelPackageArtifactV1,
    animation_authoring: &CreatureAnimationAuthoringV2,
    studio: &AnimationStudioDocumentV1,
    materialized_animations: MaterializedCreatureAnimationAuthoringV2,
    animation_studio_readback: AnimationStudioReadbackV1,
) -> Result<M6ModelPackageArtifactV5, M6PipelineErrorV1> {
    let authored_clips = studio
        .authored_clips
        .iter()
        .map(|clip| AnimationStudioAuthoredClipAuditV1 {
            id: clip.id.clone(),
            output_name: clip.name.clone(),
            kind: clip.kind,
            status: clip.status,
            revision: clip.revision,
            source: clip.source.clone(),
            keyframe_count: clip.tracks.iter().map(|track| track.keyframes.len()).sum(),
            event_count: clip.events.len(),
            usages: materialized_animations
                .authored_usages
                .iter()
                .filter(|usage| usage.authored_clip_id == clip.id)
                .cloned()
                .collect(),
        })
        .collect::<Vec<_>>();
    let audit = AnimationStudioBuildAuditV1 {
        animation_studio_schema_version: studio.schema_version,
        animation_studio_fingerprint_sha256: fingerprint_animation_studio_document_v1(studio),
        animation_studio_revision: studio.authoring_revision,
        creature_animation_authoring_schema_version: animation_authoring.schema_version,
        creature_animation_authoring_fingerprint_sha256:
            fingerprint_creature_animation_authoring_v2(animation_authoring),
        authored_clip_count: studio.authored_clips.len(),
        authored_clip_ids: studio
            .authored_clips
            .iter()
            .map(|clip| clip.id.clone())
            .collect(),
        authored_clip_output_names: studio
            .authored_clips
            .iter()
            .map(|clip| clip.name.clone())
            .collect(),
        authored_event_count: studio
            .authored_clips
            .iter()
            .map(|clip| clip.events.len())
            .sum(),
        custom_assignment_count: animation_authoring
            .assignments
            .iter()
            .filter(|assignment| assignment.source_kind == AnimationSourceKindV1::Custom)
            .count(),
        custom_runtime_exposures: materialized_animations.custom_runtime_exposures.clone(),
        source_revision: studio.source_revision.clone(),
        readback_status: animation_studio_readback.status,
        animation_studio_readback: animation_studio_readback.clone(),
        source_glb_unchanged: true,
        authored_clips,
    };
    if !audit.source_glb_unchanged {
        return Err(pipeline_error(
            "animation",
            "M2A-ANIMATION-EDIT-SOURCE-STALE",
            "artifact.sourceGlb",
            "V5 source GLB changed during artifact finalization",
        ));
    }
    let report = M6MaterializationReportV5 {
        base: base.report.clone(),
        animation_studio: audit.clone(),
    };
    let report_json = json_bytes(&report, "report")?;
    let mut summary_base = base.summary.clone();
    summary_base.outputs.report = identity(&report_json);
    let summary = M6MaterializationSummaryV5 {
        base: summary_base,
        animation_studio: audit.clone(),
    };
    let summary_json = json_bytes(&summary, "summary")?;
    let mut manifest_base = base.manifest.clone();
    for generated in &mut manifest_base.generated_files {
        let replacement = match generated.relative_path.as_str() {
            "reports/materialization-report.json" => Some(identity(&report_json)),
            "reports/summary.json" => Some(identity(&summary_json)),
            _ => None,
        };
        if let Some(replacement) = replacement {
            generated.byte_length = replacement.byte_length;
            generated.sha256 = replacement.sha256;
        }
    }
    let manifest = M6MaterializationManifestV5 {
        base: manifest_base,
        animation_studio: audit,
    };
    let manifest_json = json_bytes(&manifest, "manifest")?;
    Ok(M6ModelPackageArtifactV5 {
        source_glb: base.source_glb,
        model: base.model,
        texture: base.texture,
        appearance_two_da: base.appearance_two_da,
        hak: base.hak,
        proof_module: base.proof_module,
        manifest,
        package_manifest: base.package_manifest,
        manifest_json,
        report,
        report_json,
        summary,
        summary_json,
        animation_studio_readback,
        materialized_animations,
    })
}

fn apply_automatic_h1_animation_profile_names_v1(
    source: &GlbIngestResult,
    mapping: &mut ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
) {
    if !is_full_native_42_profile(animation_profile) {
        return;
    }
    let procedural_idle_only_compatibility = animation_profile
        == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        && mapping.clip_mappings.len() == 1
        && source.ir.animations.len() == 1;
    for (index, clip_mapping) in mapping.clip_mappings.iter_mut().enumerate() {
        let source_name = source
            .ir
            .animations
            .iter()
            .find(|animation| animation.id == clip_mapping.source_animation_id)
            .and_then(|animation| animation.name.as_deref());
        clip_mapping.output_clip_name = if procedural_idle_only_compatibility {
            "cpause1".to_owned()
        } else {
            source_name
                .and_then(|name| {
                    FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
                        .iter()
                        .find(|candidate| candidate.eq_ignore_ascii_case(name))
                        .copied()
                })
                .map(str::to_owned)
                .unwrap_or_else(|| format!("m2a_h1_{index}"))
        };
    }
}

/// Materializes the first independent Meshy runtime control (M0).  Unlike
/// H1, M0 deliberately accepts one static unskinned source primitive and
/// supplies only clean-room, motionless direct-creature lifecycle clips.
///
/// The output HAK carries one appended M0 appearance row. Its paired MOD uses
/// the validated 2x2 vertical-slice layout and has no external control HAK
/// dependency, so a generated package is self-contained.
pub fn build_meshy_m0_static_rigid_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_m0_static_rigid_package_internal(
        source_glb,
        appearance_two_da,
        M0PackageContractV1::IsolatedControl,
        None,
        None,
    )
}

/// Explicit V2 static package construction. Unlike the legacy V1 builder,
/// this path may emit a fixture contract because its trust root is supplied
/// independently by the caller.
pub fn build_meshy_m0_static_rigid_package_with_profile_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_profile: &DirectCreatureRuntimeProfileV2,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_m0_static_rigid_package_internal(
        source_glb,
        appearance_two_da,
        M0PackageContractV1::IsolatedControl,
        None,
        Some(runtime_profile),
    )
}

/// Legacy runtime entry point retained only as an explicit fail-closed API.
/// Runtime materialization requires a caller-owned V2 profile.
pub fn build_meshy_m0_canonical_runtime_package_v1(
    _source_glb: &[u8],
    _appearance_two_da: &[u8],
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    Err(legacy_m0_runtime_admission_forbidden())
}

/// Materializes the production M0 runtime contract with a fresh caller-owned
/// module/Area/HAK identity. Model and texture resrefs remain the stable M0
/// resource identities; only the immutable proof lineage container names are
/// supplied by the caller.
pub fn build_meshy_m0_canonical_runtime_package_with_identity_v1(
    _source_glb: &[u8],
    _appearance_two_da: &[u8],
    _identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    Err(legacy_m0_runtime_admission_forbidden())
}

fn legacy_m0_runtime_admission_forbidden() -> M6PipelineErrorV1 {
    pipeline_error(
        "runtime_fixture_contract",
        "M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN",
        "runtimeProfile",
        "legacy V1 runtime construction is offline-only; use an explicit caller-owned V2 profile and source bytes",
    )
}

/// Explicit V2 admission path. The caller-owned profile is an independent
/// trust root; its source SHA/length cannot be replaced by fields recomputed
/// inside the generated fixture contract.
pub fn build_meshy_m0_canonical_runtime_package_with_profile_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    runtime_profile: &DirectCreatureRuntimeProfileV2,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_m0_static_rigid_package_internal(
        source_glb,
        appearance_two_da,
        M0PackageContractV1::Canonical,
        None,
        Some(runtime_profile),
    )
}

pub fn build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity: &BinaryM0VerticalSliceIdentityV1,
    runtime_profile: &DirectCreatureRuntimeProfileV2,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_meshy_m0_static_rigid_package_internal(
        source_glb,
        appearance_two_da,
        M0PackageContractV1::Canonical,
        Some(identity),
        Some(runtime_profile),
    )
}

fn build_meshy_m0_static_rigid_package_internal(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    contract: M0PackageContractV1,
    runtime_identity: Option<&BinaryM0VerticalSliceIdentityV1>,
    runtime_profile: Option<&DirectCreatureRuntimeProfileV2>,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let mut ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut ingest)?;
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    let conversion = convert_profile_a(&ingest, &rig, &Default::default())
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    if !conversion.report.conversion_eligible {
        return Err(pipeline_error(
            "profile",
            "M0-PROFILE-INELIGIBLE",
            "conversion.report",
            "static M0 Profile A conversion did not produce an eligible model",
        ));
    }
    let mut creature = conversion.creature.as_ref().cloned().ok_or_else(|| {
        pipeline_error(
            "profile",
            "M0-PROFILE-INELIGIBLE",
            "conversion.creature",
            "eligible static M0 conversion has no creature output",
        )
    })?;
    let root_node_id = creature
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .map(|node| node.id)
        .ok_or_else(|| {
            pipeline_error(
                "profile",
                "M0-HIERARCHY-INVALID",
                "conversion.creature.nodes",
                "static M0 direct creature has no root node",
            )
        })?;
    let mut animations = static_direct_creature_runtime_clips(M0_MODEL_RESREF, root_node_id);
    normalize_direct_creature_runtime_root(&mut creature, &mut animations, M0_MODEL_RESREF)?;
    split_creature_segments_for_binary_mdl_v1(&mut creature)?;

    let input_glb_identity = identity(source_glb);
    let input_appearance_identity = identity(appearance_two_da);
    let texture_selection = resolve_base_color_image_index_v1(&ingest, &creature)?;
    let texture_image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        pipeline_error(
            "texture",
            error.code,
            error.json_path.unwrap_or_else(|| "images".to_owned()),
            error.message,
        )
    })?;
    let tga = write_tga_v1(&texture_image, &TgaWriterOptionsV1::default())
        .map_err(|error| pipeline_error("texture", error.code, error.path, error.message))?;
    let mdl = write_binary_mdl_with_animations(
        &creature,
        &animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M0StaticRigidNativeV1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: texture_selection.material_slot,
                resref: M0_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| pipeline_error("model", error.code, error.path, error.message))?;
    let appearance_inspection = inspect_two_da_v2(appearance_two_da, &TwoDaLimitsV1::default())
        .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
    let m0_appearance_cells = |label: &str, race: &str| {
        complete_direct_creature_appearance_request_v2(
            appearance_two_da,
            &appearance_inspection,
            label,
            race,
            "M0",
            DirectCreatureAppearanceSemanticProfileV2::LegacyDirectMonsterDonorV1,
        )
    };
    let (
        appearance,
        proof_module,
        status,
        appearance_payload_policy,
        appearance_table_scope,
        hak_file_name,
        proof_module_file_name,
    ) = match contract {
        M0PackageContractV1::IsolatedControl => {
            let runtime_appearance_two_da = retain_two_da_row_prefix_v1(
                appearance_two_da,
                AURORA_TOOLSET_VISIBLE_APPEARANCE_ROWS_V1,
                &TwoDaLimitsV1::default(),
            )
            .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
            let appearance = append_two_da_row_v1(
                &runtime_appearance_two_da,
                &m0_appearance_cells(M0_APPEARANCE_LABEL, M0_MODEL_RESREF)?,
                &TwoDaLimitsV1::default(),
            )
            .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
            let proof_module =
                build_binary_m0_vertical_slice_module_v1(appearance.report.appended_row_index)
                    .map_err(|error| {
                        pipeline_error("proof_module", error.code, error.path, error.message)
                    })?;
            (
                appearance,
                proof_module,
                "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED",
                "AURORA_VISIBLE_PREFIX_AND_ONE_ROW_APPENDED",
                M0AppearanceTableScopeV1::IsolatedToolsetVerticalSlice,
                M0_HAK_FILE_NAME.to_owned(),
                M0_PROOF_MODULE_FILE_NAME.to_owned(),
            )
        }
        M0PackageContractV1::Canonical => {
            let appearance = append_two_da_row_v1(
                appearance_two_da,
                &m0_appearance_cells(M0_APPEARANCE_LABEL, M0_MODEL_RESREF)?,
                &TwoDaLimitsV1::default(),
            )
            .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
            let (proof_module, hak_file_name, proof_module_file_name) =
                if let Some(identity) = runtime_identity {
                    (
                        build_binary_m0_vertical_slice_module_with_identity_v1(
                            appearance.report.appended_row_index,
                            identity,
                        )
                        .map_err(|error| {
                            pipeline_error("proof_module", error.code, error.path, error.message)
                        })?,
                        format!("{}.hak", identity.hak_resref),
                        format!("{}.mod", identity.module_resref),
                    )
                } else {
                    (
                        build_canonical_m0_creature_proof_module_v1(
                            appearance.report.appended_row_index,
                        )
                        .map_err(|error| {
                            pipeline_error("proof_module", error.code, error.path, error.message)
                        })?,
                        M6_HAK_FILE_NAME.to_owned(),
                        M6_PROOF_MODULE_FILE_NAME.to_owned(),
                    )
                };
            (
                appearance,
                proof_module,
                "M0_MESHY_STATIC_RIGID_CANONICAL_PACKAGE_MATERIALIZED",
                "FULL_RUNTIME_TABLE_AND_ONE_ROW_APPENDED",
                M0AppearanceTableScopeV1::FullRuntimeAppendV1,
                hak_file_name,
                proof_module_file_name,
            )
        }
    };
    let m0_appearance_table = M0AppearanceTableBindingV1 {
        schema_version: 1,
        scope: appearance_table_scope,
        input_physical_rows: appearance.report.physical_rows_before,
        output_physical_rows: appearance.report.physical_rows_after,
        appended_physical_row: appearance.report.appended_row_index,
        input_byte_length: appearance.report.source_byte_length,
        output_byte_length: appearance.report.output_byte_length,
        input_sha256: appearance.report.source_sha256.clone(),
        output_sha256: appearance.report.output_sha256.clone(),
        source_prefix_preserved: appearance.report.source_prefix_preserved,
    };

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: mdl.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: tga.payload.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| pipeline_error("package", error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        pipeline_error(
            "readback",
            error.code,
            format!("hak@{}", error.offset),
            error.context,
        )
    })?;
    let model = archive
        .find(M0_MODEL_RESREF, 2002)
        .map_err(map_erf_readback)?
        .to_vec();
    let texture = archive
        .find(M0_TEXTURE_RESREF, 3)
        .map_err(map_erf_readback)?
        .to_vec();
    let appearance_two_da = archive
        .find("appearance", 2017)
        .map_err(map_erf_readback)?
        .to_vec();
    let hak = package.hak.payload.clone();
    let package_manifest = package.manifest;
    let primitive = ingest.ir.primitives.first().ok_or_else(|| {
        pipeline_error(
            "ingest",
            "M0-GEOMETRY-MISSING",
            "meshes",
            "M0 source GLB has no primitive",
        )
    })?;
    let output_segment = creature.segments.first().ok_or_else(|| {
        pipeline_error(
            "profile",
            "M0-GEOMETRY-MISSING",
            "creature.segments",
            "converted M0 model has no segment",
        )
    })?;
    let m0_runtime_fixture_contract = match runtime_profile {
        Some(runtime_profile) => Some(build_m0_binary_runtime_fixture_contract_v2(
            source_glb,
            runtime_profile,
            &proof_module,
            &hak,
            &model,
            &texture,
            &appearance_two_da,
            &appearance.report,
            &m0_appearance_table,
        )?),
        None => None,
    };
    let report = M6MaterializationReportV1 {
        schema_version: 1,
        resolved_base_color_image_index: texture_selection.source_image_index,
        texture_selection: texture_selection.clone(),
        geometry: M6GeometryReportV1 {
            vertex_count: primitive.positions.len(),
            triangle_count: primitive.indices.len() / 3,
            bounds_min: primitive.bounds_min,
            bounds_max: primitive.bounds_max,
            output_segment_deformation: format!("{:?}", output_segment.deformation)
                .to_ascii_uppercase(),
            active_joint_count: 0,
        },
        ingest: ingest.report,
        conversion: conversion.report,
        model: mdl.report,
        texture: tga.report,
        texture_artifact_cleanup: None,
        skin_accessory_stabilization: None,
        appearance: appearance.report,
        hak: package.hak.report,
        proof_module: proof_module.report.clone(),
        animation_completeness: None,
        animation_behavior: None,
        animation_kinematics_conformance: None,
        animation_event_conformance: None,
        animation_event_timing_policy: None,
        animation_event_authoring_canonical: None,
        skin_animation_conformance: None,
        m0_runtime_fixture_contract: m0_runtime_fixture_contract.clone(),
        animation_authoring: None,
        authored_animation_conformance: None,
        held_weapon: None,
    };
    let report_json = json_bytes(&report, "report")?;
    let summary = M6MaterializationSummaryV1 {
        schema_version: 1,
        status: status.to_owned(),
        input_glb: input_glb_identity.clone(),
        input_appearance_two_da: input_appearance_identity.clone(),
        outputs: M6OutputIdentitiesV1 {
            model: identity(&model),
            texture: identity(&texture),
            appearance_two_da: identity(&appearance_two_da),
            hak: identity(&hak),
            proof_module: identity(&proof_module.payload),
            report: identity(&report_json),
        },
        appended_physical_row: report.appearance.appended_row_index,
        model_resref: M0_MODEL_RESREF.to_owned(),
        texture_resref: M0_TEXTURE_RESREF.to_owned(),
        animation: M6AnimationSummaryV1 {
            source_name: "M2A_STATIC_IDENTITY".to_owned(),
            output_name: "cpause1".to_owned(),
            duration_seconds: 1.0,
            has_motion: false,
        },
        provenance: rig.provenance.clone(),
        zero_reference_model_payload_copied: rig
            .provenance
            .attestations
            .no_reference_payload_copied,
        appearance_payload_policy: appearance_payload_policy.to_owned(),
        m0_appearance_table: Some(m0_appearance_table.clone()),
        m0_runtime_fixture_contract: m0_runtime_fixture_contract.clone(),
    };
    let summary_json = json_bytes(&summary, "summary")?;
    let generated_files = [
        ("generated/source.glb", source_glb),
        ("generated/m2a_m0p01.mdl", model.as_slice()),
        ("generated/m2a_m0t01.tga", texture.as_slice()),
        ("generated/appearance.2da", appearance_two_da.as_slice()),
        (&format!("generated/{hak_file_name}"), hak.as_slice()),
        (
            &format!("generated/{proof_module_file_name}"),
            proof_module.payload.as_slice(),
        ),
        (
            "reports/materialization-report.json",
            report_json.as_slice(),
        ),
        ("reports/summary.json", summary_json.as_slice()),
    ]
    .into_iter()
    .map(|(relative_path, bytes)| {
        let identity = identity(bytes);
        M6GeneratedFileV1 {
            relative_path: relative_path.to_owned(),
            byte_length: identity.byte_length,
            sha256: identity.sha256,
        }
    })
    .collect();
    let manifest = M6MaterializationManifestV1 {
        schema_version: 1,
        status: status.to_owned(),
        input_glb: input_glb_identity,
        input_appearance_two_da: input_appearance_identity,
        texture_selection,
        appended_physical_row: report.appearance.appended_row_index,
        generated_files,
        package_manifest: package_manifest.clone(),
        appearance_payload_policy: appearance_payload_policy.to_owned(),
        project_identity: None,
        m0_appearance_table: Some(m0_appearance_table),
        manifest_self_hash_policy: "EXCLUDED_TO_AVOID_SELF_REFERENCE".to_owned(),
        m0_runtime_fixture_contract,
        animation_authoring: None,
        authored_animation_conformance: None,
    };
    let manifest_json = json_bytes(&manifest, "manifest")?;
    Ok(M6ModelPackageArtifactV1 {
        source_glb: source_glb.to_vec(),
        model,
        texture,
        appearance_two_da,
        hak,
        proof_module: proof_module.payload,
        manifest,
        package_manifest,
        manifest_json,
        report,
        report_json,
        summary,
        summary_json,
    })
}

fn m0_runtime_resource_binding(resref: &str, payload: &[u8]) -> M0RuntimeResourceBindingV1 {
    let byte_identity = identity(payload);
    M0RuntimeResourceBindingV1 {
        resref: resref.to_owned(),
        byte_length: byte_identity.byte_length,
        sha256: byte_identity.sha256,
    }
}

fn m0_runtime_appearance_cell_text(
    appearance_two_da: &[u8],
    physical_row: u16,
    column_name: &str,
) -> Result<String, M6PipelineErrorV1> {
    let inspection =
        inspect_two_da_v2(appearance_two_da, &TwoDaLimitsV1::default()).map_err(|error| {
            pipeline_error(
                "runtime_fixture_contract",
                error.code,
                error.path,
                error.message,
            )
        })?;
    let column_index = inspection
        .columns
        .iter()
        .position(|column| column.eq_ignore_ascii_case(column_name))
        .ok_or_else(|| {
            pipeline_error(
                "runtime_fixture_contract",
                "M0-RUNTIME-CONTRACT-APPEARANCE-COLUMN-MISSING",
                format!("appearance.columns.{column_name}"),
                "runtime appearance.2da is missing a required binding column",
            )
        })?;
    let row = read_two_da_row_v2(
        appearance_two_da,
        u32::from(physical_row),
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    let cell = row.cells.get(column_index).ok_or_else(|| {
        pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-APPEARANCE-CELL-MISSING",
            format!("appearance.rows[{physical_row}].{column_name}"),
            "runtime appearance.2da row is missing a required binding value",
        )
    })?;
    match cell {
        TwoDaCellValueV1::Text { value } => Ok(value.clone()),
        TwoDaCellValueV1::Null => Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-APPEARANCE-CELL-NULL",
            format!("appearance.rows[{physical_row}].{column_name}"),
            "required M0 runtime appearance binding cannot be null",
        )),
    }
}

/// Reads the strict renderability gate for the M0 direct-creature writer
/// profile. This deliberately applies to the emitted M0 profile only; the
/// general binary-MDL reader remains tolerant of other legal mesh modes found
/// in reference content.
pub fn inspect_m0_runtime_mesh_eligibility_v1(
    model: &[u8],
) -> Result<M0RuntimeMeshEligibilityV1, M6PipelineErrorV1> {
    let model_readback = crate::mdl::inspect_binary_mdl(model).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            format!("model@{}", error.offset),
            error.context,
        )
    })?;
    let mut checked_mesh_count = 0u32;
    let mut eligible_mesh_count = 0u32;
    let mut mesh_types = Vec::new();
    let mut texture_resrefs = Vec::new();
    fn visit_meshes(
        node: &crate::mdl::NodeReport,
        checked_mesh_count: &mut u32,
        eligible_mesh_count: &mut u32,
        mesh_types: &mut Vec<u32>,
        texture_resrefs: &mut Vec<String>,
    ) {
        if let Some(mesh) = node.mesh.as_ref() {
            *checked_mesh_count += 1;
            mesh_types.push(mesh.mesh_type);
            texture_resrefs.extend(
                mesh.textures
                    .iter()
                    .filter(|resref| !resref.is_empty())
                    .cloned(),
            );
            let face_indices = mesh
                .faces
                .iter()
                .flat_map(|face| face.vertex_indices)
                .collect::<Vec<_>>();
            let expected_index_count = u32::try_from(face_indices.len()).ok();
            let renderable = expected_index_count.is_some_and(|expected_index_count| {
                mesh.render == 1
                    && mesh.mesh_type == 3
                    && mesh.vertex_count > 0
                    && mesh.vertices.len() == mesh.vertex_count
                    && !mesh.faces.is_empty()
                    && mesh.faces.iter().all(|face| {
                        face.vertex_indices
                            .iter()
                            .all(|index| usize::from(*index) < mesh.vertex_count)
                    })
                    && mesh.index_counts == [expected_index_count]
                    && mesh.raw_index_offsets.len() == 1
                    && mesh.raw_indices.len() == 1
                    && mesh.raw_indices[0] == face_indices
            });
            if renderable {
                *eligible_mesh_count += 1;
            }
        }
        for child in &node.children {
            visit_meshes(
                child,
                checked_mesh_count,
                eligible_mesh_count,
                mesh_types,
                texture_resrefs,
            );
        }
    }
    for root in &model_readback.node_tree.roots {
        visit_meshes(
            root,
            &mut checked_mesh_count,
            &mut eligible_mesh_count,
            &mut mesh_types,
            &mut texture_resrefs,
        );
    }
    if checked_mesh_count == 0 {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-MESH-MISSING",
            "model.nodeTree",
            "binary M0 model has no geometry mesh below its direct-creature root",
        ));
    }
    Ok(M0RuntimeMeshEligibilityV1 {
        eligible: eligible_mesh_count == checked_mesh_count,
        checked_mesh_count,
        eligible_mesh_count,
        mesh_types,
        texture_resrefs,
        rule: "render == 1 && meshType == 3 && vertexCount > 0 && faces.nonEmpty && faceIndicesMatchSingleRawIndexStream && positions.len == vertexCount".to_owned(),
    })
}

fn inspect_m0_runtime_state_projection_v1(
    model: &[u8],
    profile: MdlStateProjectionProfileV1,
    provenance: Option<&MdlStateProjectionProvenanceV1>,
) -> Result<(DirectCreatureStructuralSummaryV1, String), M6PipelineErrorV1> {
    let readback = inspect_binary_mdl(model).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            format!("model@{}", error.offset),
            error.context,
        )
    })?;
    let summary = verify_direct_creature_state_projection_v1(&readback, profile, provenance)
        .map_err(|error| {
            pipeline_error(
                "runtime_fixture_contract",
                error.code,
                error.path,
                error.message,
            )
        })?;
    let digest = direct_creature_structural_summary_digest_v1(&summary).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    Ok((summary, digest))
}

fn require_m0_runtime_mesh_eligibility_v1(
    eligibility: &M0RuntimeMeshEligibilityV1,
) -> Result<(), M6PipelineErrorV1> {
    if eligibility.eligible {
        return Ok(());
    }
    Err(pipeline_error(
        "runtime_fixture_contract",
        "M0-RUNTIME-CONTRACT-MESH-INELIGIBLE",
        "contract.meshEligibility",
        "emitted M0 direct-creature mesh does not satisfy the required renderability gate",
    ))
}

/// Enforces the resolver contract specific to the direct static M0 profile.
/// A packet can be internally self-consistent while still selecting a composite
/// creature or a different model/texture at runtime, so this is deliberately
/// separate from generic HAK hash and binary-mesh checks.
fn require_m0_direct_creature_resolver_binding_v1(
    appearance: &M0RuntimeAppearanceBindingV1,
    mesh_eligibility: &M0RuntimeMeshEligibilityV1,
) -> Result<(), M6PipelineErrorV1> {
    if appearance.model_type != "S" {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-APPEARANCE-MODELTYPE-INVALID",
            format!("appearance.rows[{}].MODELTYPE", appearance.physical_row),
            "direct static M0 requires MODELTYPE S so RACE resolves one model resource",
        ));
    }
    if appearance.race != M0_MODEL_RESREF {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-APPEARANCE-RACE-MISMATCH",
            format!("appearance.rows[{}].RACE", appearance.physical_row),
            "direct static M0 RACE must resolve the generated M0 model resource",
        ));
    }
    if mesh_eligibility.texture_resrefs.as_slice() != [M0_TEXTURE_RESREF] {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-MESH-TEXTURE-RESREF-MISMATCH",
            "model.meshes.textureResrefs",
            "every direct M0 geometry mesh must resolve exactly the generated opaque texture",
        ));
    }
    Ok(())
}

// This validator intentionally receives every independently hashed artifact so
// the cross-artifact contract remains visible at the call boundary.
#[allow(clippy::too_many_arguments)]
fn build_m0_binary_runtime_fixture_contract_v2(
    source_glb: &[u8],
    runtime_profile: &DirectCreatureRuntimeProfileV2,
    proof_module: &ProofModuleArtifactV1,
    hak: &[u8],
    model: &[u8],
    texture: &[u8],
    appearance_two_da: &[u8],
    appearance: &TwoDaAppendReportV1,
    appearance_table: &M0AppearanceTableBindingV1,
) -> Result<M0RuntimeFixtureContractV2, M6PipelineErrorV1> {
    let binary_scene =
        inspect_binary_m0_vertical_slice_module_v1(&proof_module.payload).map_err(|error| {
            pipeline_error(
                "runtime_fixture_contract",
                error.code,
                error.path,
                error.message,
            )
        })?;
    let hak_resref = binary_scene
        .ordered_hak_resrefs
        .first()
        .cloned()
        .filter(|_| binary_scene.ordered_hak_resrefs.len() == 1)
        .ok_or_else(|| {
            pipeline_error(
                "runtime_fixture_contract",
                "M0-RUNTIME-CONTRACT-HAK-LIST-INVALID",
                "binaryScene.orderedHakResrefs",
                "binary M0 contract requires exactly one ordered generated HAK",
            )
        })?;
    let hak_archive = ErfArchive::parse(hak).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            format!("hak@{}", error.offset),
            error.context,
        )
    })?;
    let hak_model = hak_archive
        .find(M0_MODEL_RESREF, 2002)
        .map_err(map_erf_readback)?;
    let hak_texture = hak_archive
        .find(M0_TEXTURE_RESREF, 3)
        .map_err(map_erf_readback)?;
    let hak_appearance = hak_archive
        .find("appearance", 2017)
        .map_err(map_erf_readback)?;
    if hak_model != model || hak_texture != texture || hak_appearance != appearance_two_da {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-HAK-READBACK-DIFF",
            "hak.resources",
            "HAK resources differ from the payloads bound into the runtime contract",
        ));
    }
    if identity(appearance_two_da).sha256 != appearance.output_sha256 {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-APPEARANCE-HASH-DIFF",
            "appearance.2da",
            "HAK appearance.2da does not match the append report output hash",
        ));
    }
    let state_projection_profile = MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1;
    let state_projection_provenance = None;
    let (state_projection_summary, state_projection_summary_sha256) =
        inspect_m0_runtime_state_projection_v1(
            model,
            state_projection_profile,
            state_projection_provenance.as_ref(),
        )?;
    let source_topology_binding =
        inspect_m0_source_topology_binding_v1(source_glb, model, runtime_profile).map_err(
            |error| {
                pipeline_error(
                    "runtime_fixture_contract",
                    error.code,
                    error.path,
                    error.message,
                )
            },
        )?;
    let (engine_envelope, engine_envelope_sha256) =
        inspect_direct_creature_engine_envelope_v1(model).map_err(|error| {
            pipeline_error(
                "runtime_fixture_contract",
                error.code,
                error.path,
                error.message,
            )
        })?;
    let mesh_eligibility = inspect_m0_runtime_mesh_eligibility_v1(model)?;
    require_m0_runtime_mesh_eligibility_v1(&mesh_eligibility)?;
    let appearance_binding = M0RuntimeAppearanceBindingV1 {
        physical_row: binary_scene.fixture.appearance_row,
        label: m0_runtime_appearance_cell_text(
            appearance_two_da,
            binary_scene.fixture.appearance_row,
            "LABEL",
        )?,
        model_type: m0_runtime_appearance_cell_text(
            appearance_two_da,
            binary_scene.fixture.appearance_row,
            "MODELTYPE",
        )?,
        race: m0_runtime_appearance_cell_text(
            appearance_two_da,
            binary_scene.fixture.appearance_row,
            "RACE",
        )?,
    };
    if appearance_binding.physical_row != appearance.appended_row_index {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-APPEARANCE-ROW-DIFF",
            "binaryScene.fixture.appearanceRow",
            "GIT Appearance_Type does not match the appended appearance.2da row",
        ));
    }
    require_m0_direct_creature_resolver_binding_v1(&appearance_binding, &mesh_eligibility)?;
    Ok(M0RuntimeFixtureContractV2 {
        schema_version: 2,
        lane: "M0_BINARY_VERTICAL_SLICE_V2".to_owned(),
        runtime_profile: runtime_profile.clone(),
        runtime_profile_sha256: direct_creature_runtime_profile_digest_v2(runtime_profile)
            .map_err(|error| {
                pipeline_error(
                    "runtime_fixture_contract",
                    error.code,
                    error.path,
                    error.message,
                )
            })?,
        module: m0_runtime_resource_binding(&binary_scene.module_resref, &proof_module.payload),
        hak: m0_runtime_resource_binding(&hak_resref, hak),
        model: m0_runtime_resource_binding(M0_MODEL_RESREF, model),
        texture: m0_runtime_resource_binding(M0_TEXTURE_RESREF, texture),
        appearance_two_da: m0_runtime_resource_binding("appearance", appearance_two_da),
        state_projection_profile,
        state_projection_provenance,
        state_projection_summary,
        state_projection_summary_sha256,
        source_topology_binding,
        engine_envelope,
        engine_envelope_sha256,
        appearance_table: appearance_table.clone(),
        appearance: appearance_binding,
        binary_scene,
        mesh_eligibility,
    })
}

/// Recomputes the complete M0 binary-vertical-slice binding from a candidate
/// MOD and HAK.  A visual capture may be called proof of this build only when
/// this verifier succeeds; it deliberately does not launch Aurora or NWN.
pub fn verify_m0_binary_runtime_fixture_contract_v2(
    contract: &M0RuntimeFixtureContractV2,
    expected_runtime_profile: &DirectCreatureRuntimeProfileV2,
    source_glb: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M6PipelineErrorV1> {
    let mismatch = |path: &str, message: &str| {
        pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-MISMATCH",
            path,
            message,
        )
    };
    if contract.schema_version != 2 || contract.lane != "M0_BINARY_VERTICAL_SLICE_V2" {
        return Err(mismatch(
            "contract.lane",
            "expected exact schema 2 M0_BINARY_VERTICAL_SLICE_V2 contract",
        ));
    }
    let expected_profile_sha256 =
        direct_creature_runtime_profile_digest_v2(expected_runtime_profile).map_err(|error| {
            pipeline_error(
                "runtime_fixture_contract",
                error.code,
                error.path,
                error.message,
            )
        })?;
    if &contract.runtime_profile != expected_runtime_profile
        || contract.runtime_profile_sha256 != expected_profile_sha256
    {
        return Err(mismatch(
            "contract.runtimeProfile",
            "embedded profile differs from the independently supplied caller-owned V2 trust root",
        ));
    }
    let scene = inspect_binary_m0_vertical_slice_module_v1(module).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    if scene != contract.binary_scene {
        return Err(mismatch(
            "contract.binaryScene",
            "MOD IFO/GIT scene does not match the bound entry, HAK list, or fixture",
        ));
    }
    if contract.module != m0_runtime_resource_binding(&scene.module_resref, module) {
        return Err(mismatch(
            "contract.module",
            "MOD identity or resref differs from the runtime packet",
        ));
    }
    let hak_resref = scene
        .ordered_hak_resrefs
        .first()
        .filter(|_| scene.ordered_hak_resrefs.len() == 1)
        .ok_or_else(|| {
            mismatch(
                "module.ifo.Mod_HakList",
                "expected exactly one generated HAK",
            )
        })?;
    if contract.hak != m0_runtime_resource_binding(hak_resref, hak) {
        return Err(mismatch(
            "contract.hak",
            "HAK identity or ordered MOD HAK resref differs from the runtime packet",
        ));
    }
    let archive = ErfArchive::parse(hak).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            format!("hak@{}", error.offset),
            error.context,
        )
    })?;
    let model = archive
        .find(M0_MODEL_RESREF, 2002)
        .map_err(map_erf_readback)?;
    // Assess the exact ordered-HAK resource before deterministic replay. Raw
    // range, alias, marker, trailing-byte, or parser defects are envelope
    // admission failures in their own right and must not be hidden behind a
    // later replay mismatch.
    let (engine_envelope, engine_envelope_sha256) =
        inspect_direct_creature_engine_envelope_v1(model).map_err(|error| {
            pipeline_error(
                "runtime_fixture_contract",
                error.code,
                error.path,
                error.message,
            )
        })?;
    if contract.engine_envelope != engine_envelope
        || contract.engine_envelope_sha256 != engine_envelope_sha256
    {
        return Err(mismatch(
            "contract.engineEnvelope",
            "MDL/MDX engine envelope or canonical digest differs from exact ordered-HAK readback",
        ));
    }
    let replayed_model = replay_m0_direct_creature_model_v2(source_glb, expected_runtime_profile)?;
    if model != replayed_model {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-DETERMINISTIC-REPLAY-MISMATCH",
            "hak.model",
            "MDL extracted from the exact ordered HAK differs from independent source-to-writer replay",
        ));
    }
    let texture = archive
        .find(M0_TEXTURE_RESREF, 3)
        .map_err(map_erf_readback)?;
    let appearance_two_da = archive.find("appearance", 2017).map_err(map_erf_readback)?;
    if contract.model != m0_runtime_resource_binding(M0_MODEL_RESREF, model) {
        return Err(mismatch(
            "contract.model",
            "HAK model resource identity differs from the runtime packet",
        ));
    }
    if contract.state_projection_profile
        != MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1
        || contract.state_projection_provenance.is_some()
    {
        return Err(pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-STATE-PROJECTION-PROFILE",
            "contract.stateProjectionProfile",
            "M0 runtime contracts require RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1 with no CEP provenance",
        ));
    }
    let (state_projection_summary, state_projection_summary_sha256) =
        inspect_m0_runtime_state_projection_v1(
            model,
            contract.state_projection_profile,
            contract.state_projection_provenance.as_ref(),
        )?;
    if contract.state_projection_summary != state_projection_summary
        || contract.state_projection_summary_sha256 != state_projection_summary_sha256
    {
        return Err(mismatch(
            "contract.stateProjectionSummary",
            "MDL state-projection summary or digest differs from the recomputed Retail readback",
        ));
    }
    verify_m0_source_topology_binding_v1(
        &contract.source_topology_binding,
        expected_runtime_profile,
        source_glb,
        model,
    )
    .map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    if contract.texture != m0_runtime_resource_binding(M0_TEXTURE_RESREF, texture) {
        return Err(mismatch(
            "contract.texture",
            "HAK texture resource identity differs from the runtime packet",
        ));
    }
    if contract.appearance_two_da != m0_runtime_resource_binding("appearance", appearance_two_da) {
        return Err(mismatch(
            "contract.appearanceTwoDa",
            "HAK appearance.2da identity differs from the runtime packet",
        ));
    }
    let appearance = M0RuntimeAppearanceBindingV1 {
        physical_row: scene.fixture.appearance_row,
        label: m0_runtime_appearance_cell_text(
            appearance_two_da,
            scene.fixture.appearance_row,
            "LABEL",
        )?,
        model_type: m0_runtime_appearance_cell_text(
            appearance_two_da,
            scene.fixture.appearance_row,
            "MODELTYPE",
        )?,
        race: m0_runtime_appearance_cell_text(
            appearance_two_da,
            scene.fixture.appearance_row,
            "RACE",
        )?,
    };
    if contract.appearance != appearance {
        return Err(mismatch(
            "contract.appearance",
            "GIT Appearance_Type no longer resolves to the bound appearance.2da row",
        ));
    }
    let mesh_eligibility = inspect_m0_runtime_mesh_eligibility_v1(model)?;
    require_m0_runtime_mesh_eligibility_v1(&mesh_eligibility)?;
    require_m0_direct_creature_resolver_binding_v1(&appearance, &mesh_eligibility)?;
    if contract.mesh_eligibility != mesh_eligibility {
        return Err(mismatch(
            "contract.meshEligibility",
            "MDL native-mesh eligibility readback differs from the runtime packet",
        ));
    }
    verify_m0_full_runtime_appearance_table_binding_v1(
        &contract.appearance_table,
        appearance_two_da,
        scene.fixture.appearance_row,
    )?;
    Ok(())
}

/// Builds an A/B runtime diagnostic from the same user-owned Meshy H1 source
/// as the regular route, but deliberately emits a rigid `trimesh` instead of
/// a `skin` node.  It exists solely to isolate the native renderer's skin
/// path from the shared mesh/package path; it is not a product fallback.
///
/// Aurora First evidence: the decompiled loader has a distinct `skin` branch
/// that consumes weights, inverse-bone arrays and constant indices before the
/// common mesh parser (`decompiled_all.c:864063-864098`).  A native A/B proof
/// can therefore determine whether the unresolved failure is inside that
/// branch without changing the user's geometry, texture, HAK or module path.
pub fn build_meshy_h1_rigid_runtime_diagnostic_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let mut source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut source)?;
    let (mut rig, mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    for segment in &mut rig.segments {
        segment.deformation = RigSegmentDeformationV1::Rigid;
        segment.allowed_bone_node_ids.clear();
        segment.reference_weights.clear();
    }
    rig.content_sha256 = canonical_profile_sha256(&rig)
        .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    build_m6_model_package_with_ingest_v1(
        source_glb,
        appearance_two_da,
        source,
        &rig,
        &Default::default(),
        &mapping,
    )
}

pub fn build_m6_model_package_with_profile_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    rig: &CreatureRigProfileV1,
    mapping: &ProfileAAnimationMappingV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_m6_model_package_with_profile_v2(
        source_glb,
        appearance_two_da,
        rig,
        mapping,
        DirectCreatureAnimationProfileV1::GameplayFloor7IdleFallbackV1,
    )
}

/// Versioned M6 construction with an explicit animation-completeness policy.
///
/// `FullNative42ExplicitV1` requires the caller to map all 42 states from
/// owned source animations. It never manufactures attack, damage, death or
/// movement states by renaming `cpause1`.
pub fn build_m6_model_package_with_profile_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    rig: &CreatureRigProfileV1,
    mapping: &ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let glb_limits = GlbLimits::default();
    let ingest = ingest_glb(source_glb, &glb_limits).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    build_m6_model_package_with_ingest_v2(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        &Default::default(),
        mapping,
        animation_profile,
    )
}

/// Versioned M6 construction with explicit caller-owned animation events.
///
/// Event authoring changes no source keyframes and copies no native timing.
/// The selected event profile is evaluated again from the exact binary MDL
/// readback before a package is returned.
pub fn build_m6_model_package_with_profile_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    rig: &CreatureRigProfileV1,
    mapping: &ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
    event_profile: DirectCreatureAnimationEventProfileV1,
    event_authoring: &DirectCreatureEventAuthoringV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let glb_limits = GlbLimits::default();
    let ingest = ingest_glb(source_glb, &glb_limits).map_err(|error| {
        pipeline_error(
            "ingest",
            error.code,
            error.json_path.unwrap_or_else(|| "input".to_owned()),
            error.message,
        )
    })?;
    build_m6_model_package_with_ingest_v3(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        &Default::default(),
        mapping,
        animation_profile,
        Some((event_profile, event_authoring)),
    )
}

fn build_m6_model_package_with_ingest_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    ingest: GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &crate::profile_a::ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_m6_model_package_with_ingest_v2(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        profile_options,
        mapping,
        DirectCreatureAnimationProfileV1::GameplayFloor7IdleFallbackV1,
    )
}

fn build_m6_model_package_with_ingest_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    ingest: GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &crate::profile_a::ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_m6_model_package_with_ingest_v3(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        profile_options,
        mapping,
        animation_profile,
        None,
    )
}

// Preserve the historical v3 compatibility boundary instead of hiding its
// independently versioned inputs in a new aggregate type.
#[allow(clippy::too_many_arguments)]
fn build_m6_model_package_with_ingest_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    ingest: GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &crate::profile_a::ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
    event_configuration: Option<(
        DirectCreatureAnimationEventProfileV1,
        &DirectCreatureEventAuthoringV1,
    )>,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    build_m6_model_package_with_ingest_v4(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        profile_options,
        mapping,
        animation_profile,
        event_configuration,
        &ProceduralCreaturePackageIdentityV1::historical_default(),
    )
}

#[allow(clippy::too_many_arguments)]
fn build_m6_model_package_with_ingest_v4(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    ingest: GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &crate::profile_a::ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
    event_configuration: Option<(
        DirectCreatureAnimationEventProfileV1,
        &DirectCreatureEventAuthoringV1,
    )>,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<M6ModelPackageArtifactV1, M6PipelineErrorV1> {
    let product_identity = ProceduralCreatureProductIdentityV2::from_legacy(runtime_identity);
    match build_m6_model_package_with_ingest_v5(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        profile_options,
        mapping,
        animation_profile,
        event_configuration,
        runtime_identity,
        &product_identity,
        M6BuildOutputV2::LegacyBundle,
        M6GeometryPolicyV1::ProductBudget,
        &ProceduralCreatureBuildOptionsV1::default(),
        None,
        None,
        None,
        None,
    )? {
        M6BuildArtifactV2::LegacyBundle(built) => Ok(built.artifact),
        M6BuildArtifactV2::ProceduralProduct(_) => unreachable!("legacy output mode is exact"),
    }
}

struct M6ModelPackageInternalV5 {
    artifact: M6ModelPackageArtifactV1,
    animation_studio_materialization: Option<MaterializedCreatureAnimationAuthoringV2>,
    animation_studio_rig: Option<AnimationStudioRigV1>,
    writer_readback: crate::mdl::InspectionReport,
}

struct HeldWeaponBuildStateV1 {
    source_sha256: String,
    bake: HeldWeaponBakeReportV1,
    texture_resref: String,
    texture: Option<TgaArtifactV1>,
    material_slots: Vec<u32>,
}

#[allow(clippy::too_many_arguments)]
fn build_m6_model_package_with_ingest_v5(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    ingest: GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &crate::profile_a::ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
    animation_profile: DirectCreatureAnimationProfileV1,
    event_configuration: Option<(
        DirectCreatureAnimationEventProfileV1,
        &DirectCreatureEventAuthoringV1,
    )>,
    runtime_identity: &ProceduralCreaturePackageIdentityV1,
    product_identity: &ProceduralCreatureProductIdentityV2,
    output: M6BuildOutputV2,
    geometry_policy: M6GeometryPolicyV1,
    build_options: &ProceduralCreatureBuildOptionsV1,
    project_identity: Option<&ProjectBuildIdentityV1>,
    animation_authoring: Option<&CreatureAnimationAuthoringV1>,
    animation_studio_authoring: Option<(&CreatureAnimationAuthoringV2, &AnimationStudioDocumentV1)>,
    held_weapon: Option<(&[u8], &HeldWeaponAttachmentV1)>,
) -> Result<M6BuildArtifactV2, M6PipelineErrorV1> {
    validate_procedural_creature_product_identity_v2(product_identity)?;
    if build_options.schema_version != 1 {
        return Err(pipeline_error(
            "profile",
            "M6-PROCEDURAL-BUILD-OPTIONS-SCHEMA",
            "buildOptions.schemaVersion",
            format!(
                "expected procedural creature build-options schema 1, got {}",
                build_options.schema_version
            ),
        ));
    }
    let input_glb_identity = identity(source_glb);
    let input_appearance_identity = identity(appearance_two_da);
    let glb_limits = match geometry_policy {
        M6GeometryPolicyV1::ProductBudget => GlbLimits::default(),
        M6GeometryPolicyV1::P100kSegmentedExperiment => p100k_experiment_glb_limits_v1(),
        M6GeometryPolicyV1::P300kSegmentedExperiment => p300k_experiment_glb_limits_v1(),
    };
    let degeneracy_policy = if animation_studio_authoring.is_some() {
        TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear
    } else {
        triangle_degeneracy_policy_for_build_v1(output, geometry_policy)
    };
    let animated = match (geometry_policy, degeneracy_policy) {
        (
            M6GeometryPolicyV1::ProductBudget,
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear,
        ) => convert_profile_a_with_animations_exact_v1(&ingest, rig, profile_options, mapping),
        (M6GeometryPolicyV1::ProductBudget, TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon) => {
            convert_profile_a_with_animations_v1(&ingest, rig, profile_options, mapping)
        }
        (
            M6GeometryPolicyV1::P100kSegmentedExperiment,
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear,
        ) => convert_profile_a_with_animations_p100k_experiment_v1(&ingest, rig, mapping),
        (
            M6GeometryPolicyV1::P300kSegmentedExperiment,
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear,
        ) => convert_profile_a_with_animations_p300k_experiment_v1(&ingest, rig, mapping),
        (
            M6GeometryPolicyV1::P100kSegmentedExperiment
            | M6GeometryPolicyV1::P300kSegmentedExperiment,
            TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
        ) => unreachable!("segmented Creature experiments always use exact geometry sanitation"),
    }
    .map_err(|error| {
        let stage = if error.code.starts_with("M4A-") {
            "animation"
        } else {
            "profile"
        };
        pipeline_error(stage, error.code, error.path, error.message)
    })?;
    if !animated.base.report.conversion_eligible {
        let blocking = animated
            .base
            .report
            .gates
            .iter()
            .filter(|gate| gate.severity == "BLOCKING")
            .map(|gate| format!("{} at {}: {}", gate.code, gate.path, gate.message))
            .collect::<Vec<_>>();
        return Err(pipeline_error(
            "profile",
            "M6-PROFILE-INELIGIBLE",
            "conversion.report",
            format!(
                "Profile A conversion did not produce an eligible model{}",
                if blocking.is_empty() {
                    String::new()
                } else {
                    format!(": {}", blocking.join("; "))
                }
            ),
        ));
    }
    let mut creature = animated.base.creature.as_ref().cloned().ok_or_else(|| {
        pipeline_error(
            "profile",
            "M6-PROFILE-INELIGIBLE",
            "conversion.creature",
            "eligible conversion has no creature output",
        )
    })?;
    let source_animations = animated.animations.as_ref().ok_or_else(|| {
        pipeline_error(
            "animation",
            "M6-ANIMATION-MISSING",
            "conversion.animations",
            "M6 proof requires one mapped source animation",
        )
    })?;
    let animation_studio_source_animations = if animation_studio_authoring.is_some() {
        let mut source_mapping = mapping.clone();
        apply_authored_h1_source_names_v1(&ingest, &mut source_mapping, false);
        Some(
            convert_profile_a_with_animations_exact_v1(
                &ingest,
                rig,
                profile_options,
                &source_mapping,
            )
            .map_err(|error| pipeline_error("animation", error.code, error.path, error.message))?
            .animations
            .ok_or_else(|| {
                pipeline_error(
                    "animation",
                    "M6-ANIMATION-MISSING",
                    "conversion.animations",
                    "Animation Studio source projection has no mapped animations",
                )
            })?,
        )
    } else {
        None
    };
    let skin_accessory_stabilization = audit_and_stabilize_skin_accessories_v1(
        &mut creature,
        source_animations,
        &build_options.skin_accessory_stabilization,
    )
    .map_err(|error| pipeline_error("profile", error.code, error.path, error.message))?;
    let held_weapon_build = if let Some((weapon_glb, attachment)) = held_weapon {
        let inspected_source = inspect_held_weapon_source_v1(
            &attachment.source.filename,
            weapon_glb,
            &attachment.source.provenance,
        )
        .map_err(|message| {
            pipeline_error(
                "held_weapon",
                "M2A-HELD-WEAPON-SOURCE",
                "heldWeapon.source",
                message,
            )
        })?;
        if inspected_source != attachment.source {
            return Err(pipeline_error(
                "held_weapon",
                "M2A-HELD-WEAPON-IDENTITY",
                "heldWeapon.source",
                "reconnected weapon bytes or metadata do not match the exact authored attachment",
            ));
        }
        if inspected_source.texture_count > 1 {
            return Err(pipeline_error(
                "held_weapon",
                "M2A-HELD-WEAPON-TEXTURE-COUNT",
                "heldWeapon.source.textureCount",
                "the V1 held-weapon product route supports zero or one embedded texture and never drops additional textures",
            ));
        }
        let weapon_ingest = ingest_glb(weapon_glb, &glb_limits).map_err(|error| {
            pipeline_error(
                "held_weapon",
                error.code,
                error
                    .json_path
                    .unwrap_or_else(|| "heldWeapon.glb".to_owned()),
                error.message,
            )
        })?;
        let weapon_rig =
            derive_meshy_m0_static_rigid_profile_v1(&weapon_ingest).map_err(|error| {
                pipeline_error("held_weapon", error.code, error.path, error.message)
            })?;
        let weapon_conversion = convert_profile_a(
            &weapon_ingest,
            &weapon_rig,
            &static_placeable_profile_a_options_v1(),
        )
        .map_err(|error| pipeline_error("held_weapon", error.code, error.path, error.message))?;
        if !weapon_conversion.report.conversion_eligible {
            return Err(pipeline_error(
                "held_weapon",
                "M2A-HELD-WEAPON-CONVERSION",
                "heldWeapon.conversion",
                "held weapon did not pass the shared rigid Profile A conversion gates",
            ));
        }
        let weapon_model = weapon_conversion.creature.ok_or_else(|| {
            pipeline_error(
                "held_weapon",
                "M2A-HELD-WEAPON-CONVERSION",
                "heldWeapon.conversion.creature",
                "eligible held-weapon conversion has no rigid model output",
            )
        })?;
        let weapon_texture = if inspected_source.texture_count == 1 {
            let selection = resolve_base_color_image_index_v1(&weapon_ingest, &weapon_model)?;
            let image = decode_embedded_image_to_tga_v1(
                weapon_glb,
                selection.source_image_index,
                &glb_limits,
                &EmbeddedImageDecodeLimitsV1::default(),
            )
            .map_err(|error| {
                pipeline_error(
                    "held_weapon",
                    error.code,
                    error
                        .json_path
                        .unwrap_or_else(|| "heldWeapon.images".to_owned()),
                    error.message,
                )
            })?;
            let cleanup = cleanup_texture_artifacts_v1(
                &image,
                &TextureArtifactCleanupOptionsV1 {
                    schema_version: 1,
                    enabled: build_options.texture_artifact_cleanup,
                },
            )
            .map_err(|error| {
                pipeline_error("held_weapon", error.code, error.path, error.message)
            })?;
            Some(
                write_tga_v1(&cleanup.image, &TgaWriterOptionsV1::default()).map_err(|error| {
                    pipeline_error("held_weapon", error.code, error.path, error.message)
                })?,
            )
        } else {
            None
        };
        let bake = bake_held_weapon_attachment_v1(&creature, &weapon_model, attachment).map_err(
            |message| {
                pipeline_error(
                    "held_weapon",
                    "M2A-HELD-WEAPON-BAKE",
                    "heldWeapon.attachment",
                    message,
                )
            },
        )?;
        let material_slots = bake.model.segments[bake.segment_count_before..]
            .iter()
            .map(|segment| segment.material_slot)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let texture_resref = if weapon_texture.is_some() {
            held_weapon_texture_resref_v1(&product_identity.texture_resref)?
        } else {
            product_identity.texture_resref.clone()
        };
        creature = bake.model.clone();
        Some(HeldWeaponBuildStateV1 {
            source_sha256: inspected_source.sha256,
            bake,
            texture_resref,
            texture: weapon_texture,
            material_slots,
        })
    } else {
        None
    };
    let procedural_rig = (animation_profile
        == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1)
        .then(|| procedural_humanoid_rig_from_creature_v1(&creature))
        .transpose()?;
    let procedural_authoring = if let Some(procedural_rig) = procedural_rig {
        Some(
            author_procedural_humanoid_full_native_42_v2(source_animations, procedural_rig)
                .map_err(|error| {
                    pipeline_error("animation", error.code, error.path, error.message)
                })?,
        )
    } else {
        None
    };
    let procedural_animations = procedural_authoring
        .as_ref()
        .map(|authored| &authored.animations);
    let mut animation_studio_rig = animation_studio_authoring
        .map(|(_, studio)| {
            animation_studio_rig_from_creature_v1(
                &creature,
                &studio.source_revision,
                animation_studio_source_animations
                    .as_ref()
                    .unwrap_or(source_animations)
                    .clips
                    .first()
                    .map(|clip| clip.animation_root.as_str())
                    .unwrap_or("root"),
            )
        })
        .transpose()?;
    let mut animation_studio_materialization =
        if let Some((authoring, studio)) = animation_studio_authoring {
            Some(
                materialize_creature_animation_authoring_v2(
                    animation_studio_source_animations
                        .as_ref()
                        .unwrap_or(source_animations),
                    procedural_animations,
                    authoring,
                    studio,
                    animation_studio_rig
                        .as_ref()
                        .expect("V2 authoring creates the Studio rig"),
                )
                .map_err(first_animation_studio_pipeline_error)?,
            )
        } else {
            None
        };
    let mut animations = if let Some(materialized) = animation_studio_materialization.as_ref() {
        materialized.animations.clone()
    } else if let Some(authoring) = animation_authoring {
        materialize_authored_direct_creature_clips_v1(
            source_animations,
            procedural_animations,
            authoring,
        )
        .map_err(|error| pipeline_error("animation", error.code, error.path, error.message))?
    } else if let Some(procedural) = procedural_animations {
        procedural.clone()
    } else {
        materialize_direct_creature_runtime_clips_v1(source_animations, animation_profile)?
    };
    let animation_lineage = if let Some(authored) = procedural_authoring.as_ref() {
        Some(authored.lineage.clone())
    } else {
        (animation_profile == DirectCreatureAnimationProfileV1::FullNative42ExplicitV1).then(|| {
            DirectCreatureAnimationLineageV2 {
                schema_version: 2,
                input_source_clip_count: source_animations.clips.len() as u32,
                preserved_source_clip_count: animations.clips.len() as u32,
                source_derived_clip_count: 0,
                procedural_clip_count: 0,
                discarded_source_clips: Vec::new(),
                clips: animations
                    .clips
                    .iter()
                    .map(|clip| DirectCreatureAnimationClipLineageV2 {
                        clip_name: clip.name.clone(),
                        origin: DirectCreatureAnimationClipOriginV2::PreservedSource,
                        source_clip_name: Some(clip.name.clone()),
                    })
                    .collect(),
            }
        })
    };
    let resolved_animation_authoring = animation_authoring
        .map(resolve_creature_animation_mapping_v1)
        .transpose()
        .map_err(|error| pipeline_error("animation", error.code, error.path, error.message))?;
    let authored_animation_conformance = animation_authoring
        .map(|authoring| evaluate_authored_animation_conformance_v1(authoring, &animations));
    if let Some(conformance) = authored_animation_conformance.as_ref()
        && conformance.status
            != crate::creature_animation_mapping::CreatureAnimationMappingStatusV1::Ready
    {
        return Err(pipeline_error(
            "animation",
            "M2A-ANIMATION-AUTHORED-CONFORMANCE",
            "materialized.animations",
            "materialized animations do not conform to the authored mapping",
        ));
    }
    let procedural_event_authoring = if animation_profile
        == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        && event_configuration.is_none()
    {
        let required = author_procedural_common_native_events_v2(
            &animations,
            procedural_rig.expect("the procedural profile established its semantic humanoid rig"),
        )
        .map_err(|error| pipeline_error("animation", error.code, error.path, error.message))?;
        Some(if animation_studio_authoring.is_some() {
            merge_required_events_with_integrated_authored_v1(&animations, &required)
        } else {
            required
        })
    } else {
        None
    };
    let effective_event_configuration = event_configuration.or_else(|| {
        procedural_event_authoring.as_ref().map(|authoring| {
            (
                DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
                authoring,
            )
        })
    });
    if let Some((_, event_authoring)) = effective_event_configuration {
        animations = apply_direct_creature_event_authoring_v1(&animations, event_authoring)
            .map_err(|error| pipeline_error("animation", error.code, error.path, error.message))?;
    }
    if animation_profile == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1 {
        insert_controllerless_aurora_skin_root_v1(&mut creature, &product_identity.model_resref)?;
    }
    match degeneracy_policy {
        TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon => {
            sanitize_runtime_creature_face_planes_v1(&mut creature)?;
        }
        TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear => {
            sanitize_runtime_creature_face_planes_exact_v1(&mut creature)?;
        }
    }
    split_creature_segments_for_binary_mdl_v1(&mut creature)?;
    let runtime_root_id = creature
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .map(|node| node.id);
    normalize_direct_creature_runtime_root(
        &mut creature,
        &mut animations,
        &product_identity.model_resref,
    )?;
    if let Some(rig) = animation_studio_rig.as_mut() {
        rig.animation_root = product_identity.model_resref.clone();
        if let Some(runtime_root_id) = runtime_root_id
            && let Some(root) = rig
                .nodes
                .iter_mut()
                .find(|node| node.node_id == runtime_root_id)
        {
            root.name.clone_from(&product_identity.model_resref);
        }
    }
    if let Some(materialized) = animation_studio_materialization.as_mut() {
        materialized.animations = animations.clone();
    }
    let proof_clip = animations
        .clips
        .iter()
        .find(|clip| clip.name == "cpause1")
        .ok_or_else(|| {
            pipeline_error(
                "animation",
                "M6-ANIMATION-MISSING",
                "conversion.animations.clips",
                "M6 proof requires exact cpause1 mapped clip",
            )
        })?;
    let has_motion = proof_clip
        .tracks
        .iter()
        .any(|track| track.values.windows(2).any(|pair| pair[0] != pair[1]));
    if !has_motion {
        return Err(pipeline_error(
            "animation",
            "M6-ANIMATION-NO-MOTION",
            "conversion.animations.clips[0]",
            "mapped cpause1 clip has no changing values",
        ));
    }

    let texture_selection = resolve_base_color_image_index_v1(&ingest, &creature)?;
    let texture_image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &glb_limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        pipeline_error(
            "texture",
            error.code,
            error.json_path.unwrap_or_else(|| "images".to_owned()),
            error.message,
        )
    })?;
    let texture_cleanup = cleanup_texture_artifacts_v1(
        &texture_image,
        &TextureArtifactCleanupOptionsV1 {
            schema_version: 1,
            enabled: build_options.texture_artifact_cleanup,
        },
    )
    .map_err(|error| pipeline_error("texture", error.code, error.path, error.message))?;
    let texture_artifact_cleanup = build_options
        .texture_artifact_cleanup
        .then_some(texture_cleanup.report);
    let tga = write_tga_v1(&texture_cleanup.image, &TgaWriterOptionsV1::default())
        .map_err(|error| pipeline_error("texture", error.code, error.path, error.message))?;

    let mut texture_bindings = vec![MdlMaterialTextureBindingV1 {
        material_slot: texture_selection.material_slot,
        resref: product_identity.texture_resref.clone(),
    }];
    if let Some(weapon) = held_weapon_build.as_ref() {
        texture_bindings.extend(weapon.material_slots.iter().copied().map(|material_slot| {
            MdlMaterialTextureBindingV1 {
                material_slot,
                resref: weapon.texture_resref.clone(),
            }
        }));
    }
    let writer_options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: if animation_profile
            == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        {
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3
        } else {
            MdlFormatProfileV1::M4DirectCreatureExtended64V1
        },
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: product_identity.model_resref.clone(),
        diffuse_texture_resref_by_material_slot: texture_bindings,
    };
    let mdl = match degeneracy_policy {
        TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon => {
            write_binary_mdl_with_animations(&creature, &animations, &writer_options)
        }
        TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear => {
            write_binary_mdl_with_animations_exact_face_planes_v1(
                &creature,
                &animations,
                &writer_options,
            )
        }
    }
    .map_err(|error| pipeline_error("model", error.code, error.path, error.message))?;
    let writer_readback = mdl.inspection.clone();
    let animation_behavior = is_full_native_42_profile(animation_profile)
        .then(|| evaluate_direct_creature_animation_behavior_v2(&mdl.inspection));
    if let Some(behavior) = animation_behavior.as_ref()
        && !behavior.behavior_candidate_eligible
    {
        return Err(pipeline_error(
            "animation",
            "M6-ANIMATION-BEHAVIOR-INELIGIBLE",
            "model.animations",
            format!(
                "full direct-creature behavior oracle failed: {}",
                behavior.violations.join(", ")
            ),
        ));
    }
    let animation_kinematics_conformance = (animation_profile
        == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        && animation_authoring.is_none()
        && animation_studio_authoring.is_none())
    .then(|| evaluate_procedural_humanoid_kinematics_v2(&mdl.inspection));
    if let Some(kinematics) = animation_kinematics_conformance.as_ref()
        && !kinematics.complete
    {
        return Err(pipeline_error(
            "animation",
            "M6-ANIMATION-KINEMATICS-INELIGIBLE",
            "model.animations",
            format!(
                "procedural humanoid kinematics gate failed: {}",
                kinematics.violations.join(", ")
            ),
        ));
    }
    let animation_event_conformance = effective_event_configuration.map(|(event_profile, _)| {
        evaluate_direct_creature_event_conformance_from_inspection_v1(
            &mdl.inspection,
            event_profile,
        )
    });
    let animation_event_timing_policy = effective_event_configuration.map(|_| {
        if procedural_event_authoring.is_some() {
            DirectCreatureEventTimingPolicyV2::KinematicPeakV2
        } else {
            DirectCreatureEventTimingPolicyV2::CallerOwnedExplicitV1
        }
    });
    let animation_event_authoring_canonical = effective_event_configuration
        .map(|(_, event_authoring)| {
            serde_json::to_vec(event_authoring)
                .map(|bytes| identity(&bytes))
                .map_err(|error| {
                    pipeline_error(
                        "animation",
                        "M6-ANIMATION-EVENT-AUTHORING-SERIALIZE",
                        "eventAuthoring",
                        error.to_string(),
                    )
                })
        })
        .transpose()?;
    if let Some(conformance) = animation_event_conformance.as_ref()
        && !conformance.complete
    {
        return Err(pipeline_error(
            "animation",
            "M6-ANIMATION-EVENTS-INELIGIBLE",
            "model.animations.events",
            format!(
                "direct-creature event conformance failed: {}",
                conformance.missing_pairs.join(", ")
            ),
        ));
    }
    let active_joint_count = creature
        .segments
        .iter()
        .flat_map(|segment| segment.weights.iter())
        .flat_map(|weights| weights.bone_node_ids)
        .flatten()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let skin_animation_conformance = if is_full_native_42_profile(animation_profile) {
        let conformance =
            evaluate_m6_skin_animation_conformance_v1(&mdl.inspection, active_joint_count);
        if !conformance.complete {
            return Err(pipeline_error(
                "animation",
                "M6-SKIN-ANIMATION-INELIGIBLE",
                "model.animations",
                format!(
                    "full direct-creature SkinMesh animation conformance failed: {}",
                    conformance.violations.join(", ")
                ),
            ));
        }
        Some(conformance)
    } else {
        None
    };

    let appearance_inspection = inspect_two_da_v2(appearance_two_da, &TwoDaLimitsV1::default())
        .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
    let appearance_request = complete_direct_creature_appearance_request_v2(
        appearance_two_da,
        &appearance_inspection,
        &product_identity.appearance_label,
        &product_identity.model_resref,
        "M6",
        if is_full_native_42_profile(animation_profile) {
            DirectCreatureAppearanceSemanticProfileV2::HumanoidMediumV1
        } else {
            DirectCreatureAppearanceSemanticProfileV2::LegacyDirectMonsterDonorV1
        },
    )?;
    let appearance = append_two_da_row_v1(
        appearance_two_da,
        &appearance_request,
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;

    let mut resources = vec![
        HakResourceInputV1 {
            resref: product_identity.model_resref.clone(),
            resource_type: 2002,
            payload: mdl.payload.clone(),
        },
        HakResourceInputV1 {
            resref: product_identity.texture_resref.clone(),
            resource_type: 3,
            payload: tga.payload.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    if let Some(weapon) = held_weapon_build.as_ref()
        && let Some(texture) = weapon.texture.as_ref()
    {
        resources.push(HakResourceInputV1 {
            resref: weapon.texture_resref.clone(),
            resource_type: 3,
            payload: texture.payload.clone(),
        });
    }
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| pipeline_error("package", error.code, error.path, error.message))?;

    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        pipeline_error(
            "readback",
            error.code,
            format!("hak@{}", error.offset),
            error.context,
        )
    })?;
    let model = archive
        .find(&product_identity.model_resref, 2002)
        .map_err(map_erf_readback)?
        .to_vec();
    let texture = archive
        .find(&product_identity.texture_resref, 3)
        .map_err(map_erf_readback)?
        .to_vec();
    let appearance_two_da = archive
        .find("appearance", 2017)
        .map_err(map_erf_readback)?
        .to_vec();
    let hak = package.hak.payload.clone();
    let package_manifest = package.manifest;
    let held_weapon_readback = held_weapon_build
        .as_ref()
        .map(|weapon| {
            let attachment_node_name = format!("held_weapon_{}", weapon.bake.attachment_node_id);
            let attachment_node_present_in_binary_mdl = mdl
                .inspection
                .node_tree
                .roots
                .iter()
                .any(|root| node_tree_contains_name_v1(root, &attachment_node_name));
            let combined_triangle_count_match =
                mdl.report.projection.triangle_count == weapon.bake.target_triangle_count_after;
            let expected_texture = weapon
                .texture
                .as_ref()
                .map(|artifact| artifact.payload.as_slice())
                .unwrap_or(tga.payload.as_slice());
            let packaged_texture = archive
                .find(&weapon.texture_resref, 3)
                .map_err(map_erf_readback)?;
            let texture_payload_match_in_hak = packaged_texture == expected_texture;
            if !attachment_node_present_in_binary_mdl
                || !combined_triangle_count_match
                || !texture_payload_match_in_hak
            {
                return Err(pipeline_error(
                    "held_weapon",
                    "M2A-HELD-WEAPON-READBACK",
                    "heldWeapon.readback",
                    format!(
                        "final held-weapon readback mismatch (node={attachment_node_present_in_binary_mdl}, triangles={combined_triangle_count_match}, texture={texture_payload_match_in_hak})"
                    ),
                ));
            }
            Ok(HeldWeaponProductReadbackV1 {
                schema_version: 1,
                source_sha256: weapon.source_sha256.clone(),
                attachment_fingerprint_sha256: weapon
                    .bake
                    .attachment_fingerprint_sha256
                    .clone(),
                texture_resref: weapon.texture_resref.clone(),
                texture: identity(packaged_texture),
                bake: weapon.bake.clone(),
                attachment_node_present_in_binary_mdl,
                combined_triangle_count_match,
                texture_payload_match_in_hak,
                status: "MATCH".to_owned(),
            })
        })
        .transpose()?;

    let output_segment = creature.segments.first().ok_or_else(|| {
        pipeline_error(
            "profile",
            "M6-GEOMETRY-MISSING",
            "creature.segments",
            "converted model has no segment",
        )
    })?;
    let source_animation = ingest.ir.animations.first().ok_or_else(|| {
        pipeline_error(
            "animation",
            "M6-ANIMATION-MISSING",
            "animations",
            "source GLB has no animation",
        )
    })?;
    let animation_summary = M6AnimationSummaryV1 {
        source_name: source_animation.name.clone().unwrap_or_default(),
        output_name: proof_clip.name.clone(),
        duration_seconds: proof_clip.length_seconds,
        has_motion,
    };
    let output_vertex_count = creature
        .segments
        .iter()
        .try_fold(0usize, |sum, segment| {
            sum.checked_add(segment.positions.len())
        })
        .ok_or_else(|| {
            pipeline_error(
                "model",
                "M6-RUNTIME-GEOMETRY-OVERFLOW",
                "creature.segments",
                "runtime vertex count overflow",
            )
        })?;
    let output_triangle_count = creature
        .segments
        .iter()
        .try_fold(0usize, |sum, segment| {
            sum.checked_add(segment.indices.len() / 3)
        })
        .ok_or_else(|| {
            pipeline_error(
                "model",
                "M6-RUNTIME-GEOMETRY-OVERFLOW",
                "creature.segments",
                "runtime triangle count overflow",
            )
        })?;
    let mut output_bounds_min = [f32::INFINITY; 3];
    let mut output_bounds_max = [f32::NEG_INFINITY; 3];
    for position in creature
        .segments
        .iter()
        .flat_map(|segment| segment.positions.iter())
    {
        for (axis, value) in position.iter().copied().enumerate() {
            output_bounds_min[axis] = output_bounds_min[axis].min(value);
            output_bounds_max[axis] = output_bounds_max[axis].max(value);
        }
    }
    let geometry_report = M6GeometryReportV1 {
        vertex_count: output_vertex_count,
        triangle_count: output_triangle_count,
        bounds_min: output_bounds_min,
        bounds_max: output_bounds_max,
        output_segment_deformation: format!("{:?}", output_segment.deformation)
            .to_ascii_uppercase(),
        active_joint_count,
    };

    if output == M6BuildOutputV2::ProceduralProduct {
        let lineage = animation_lineage.clone().ok_or_else(|| {
            pipeline_error(
                "animation",
                "M6-PRODUCT-ANIMATION-LINEAGE-MISSING",
                "animationLineage",
                "the procedural product requires exact V2 clip lineage",
            )
        })?;
        let report = ProceduralCreatureProductReportV2 {
            schema_version: 2,
            identity: product_identity.clone(),
            appearance_semantic_profile:
                DirectCreatureAppearanceSemanticProfileV2::HumanoidMediumV1,
            resolved_base_color_image_index: texture_selection.source_image_index,
            texture_selection: texture_selection.clone(),
            geometry: geometry_report,
            ingest: ingest.report,
            conversion: animated.base.report,
            model: mdl.report,
            texture: tga.report,
            texture_artifact_cleanup: texture_artifact_cleanup.clone(),
            skin_accessory_stabilization: skin_accessory_stabilization.clone(),
            appearance: appearance.report,
            hak: package.hak.report,
            animation_completeness: DirectCreatureAnimationCompletenessV2 {
                schema_version: 2,
                profile: animation_profile,
                required_clip_count: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() as u32,
                input_source_clip_count: lineage.input_source_clip_count,
                preserved_source_clip_count: lineage.preserved_source_clip_count,
                source_derived_clip_count: lineage.source_derived_clip_count,
                procedural_clip_count: lineage.procedural_clip_count,
                discarded_source_clip_count: lineage.discarded_source_clips.len() as u32,
                discarded_source_clips: lineage.discarded_source_clips,
                clips: lineage.clips,
                fallback_alias_count: 0,
                complete: true,
            },
            animation_behavior: animation_behavior.expect("procedural behavior was evaluated"),
            animation_kinematics_conformance: animation_kinematics_conformance
                .expect("procedural kinematics were evaluated"),
            animation_event_conformance: animation_event_conformance
                .expect("procedural events were evaluated"),
            animation_event_timing_policy: animation_event_timing_policy
                .expect("procedural event policy was evaluated"),
            animation_event_authoring_canonical: animation_event_authoring_canonical
                .expect("procedural event authoring was hashed"),
            skin_animation_conformance: skin_animation_conformance
                .expect("procedural skin animation was evaluated"),
        };
        let report_json = json_bytes(&report, "report")?;
        let summary = ProceduralCreatureProductSummaryV2 {
            schema_version: 2,
            status: "PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED".to_owned(),
            input_glb: input_glb_identity.clone(),
            input_appearance_two_da: input_appearance_identity.clone(),
            identity: product_identity.clone(),
            outputs: ProceduralCreatureProductOutputIdentitiesV2 {
                model: identity(&model),
                texture: identity(&texture),
                appearance_two_da: identity(&appearance_two_da),
                hak: identity(&hak),
                report: identity(&report_json),
            },
            appended_physical_row: report.appearance.appended_row_index,
            animation: animation_summary,
            provenance: rig.provenance.clone(),
            zero_reference_model_payload_copied: rig
                .provenance
                .attestations
                .no_reference_payload_copied,
            appearance_payload_policy: "PRESERVED_AND_APPENDED".to_owned(),
        };
        let summary_json = json_bytes(&summary, "summary")?;
        let generated_files = [
            (
                format!("generated/{}.mdl", product_identity.model_resref),
                model.as_slice(),
            ),
            (
                format!("generated/{}.tga", product_identity.texture_resref),
                texture.as_slice(),
            ),
            (
                "generated/appearance.2da".to_owned(),
                appearance_two_da.as_slice(),
            ),
            (
                format!("generated/{}.hak", product_identity.hak_resref),
                hak.as_slice(),
            ),
            (
                "reports/materialization-report.json".to_owned(),
                report_json.as_slice(),
            ),
            ("reports/summary.json".to_owned(), summary_json.as_slice()),
        ]
        .into_iter()
        .map(|(relative_path, bytes)| {
            let identity = identity(bytes);
            M6GeneratedFileV1 {
                relative_path,
                byte_length: identity.byte_length,
                sha256: identity.sha256,
            }
        })
        .collect();
        let manifest = ProceduralCreatureProductManifestV2 {
            schema_version: 2,
            status: "PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED".to_owned(),
            input_glb: input_glb_identity,
            input_appearance_two_da: input_appearance_identity,
            identity: product_identity.clone(),
            texture_selection,
            appended_physical_row: report.appearance.appended_row_index,
            generated_files,
            package_manifest: package_manifest.clone(),
            appearance_payload_policy: "PRESERVED_AND_APPENDED".to_owned(),
            manifest_self_hash_policy: "EXCLUDED_TO_AVOID_SELF_REFERENCE".to_owned(),
        };
        let manifest_json = json_bytes(&manifest, "manifest")?;
        return Ok(M6BuildArtifactV2::ProceduralProduct(Box::new(
            ProceduralCreatureProductArtifactV2 {
                model,
                texture,
                appearance_two_da,
                hak,
                package_manifest,
                manifest,
                manifest_json,
                report,
                report_json,
                summary,
                summary_json,
            },
        )));
    }

    let proof_module = if animation_profile
        == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
        || runtime_identity != &ProceduralCreaturePackageIdentityV1::historical_default()
    {
        build_single_profiled_creature_proof_module_with_identity_v3(
            appearance.report.appended_row_index,
            BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
            &runtime_identity.module,
            &runtime_identity.creature_resref,
        )
    } else {
        build_creature_proof_module_v1(appearance.report.appended_row_index)
    }
    .map_err(|error| pipeline_error("proof_module", error.code, error.path, error.message))?;

    let report = M6MaterializationReportV1 {
        schema_version: 1,
        resolved_base_color_image_index: texture_selection.source_image_index,
        texture_selection: texture_selection.clone(),
        geometry: geometry_report,
        ingest: ingest.report,
        conversion: animated.base.report,
        model: mdl.report,
        texture: tga.report,
        texture_artifact_cleanup,
        skin_accessory_stabilization: Some(skin_accessory_stabilization),
        appearance: appearance.report,
        hak: package.hak.report,
        proof_module: proof_module.report.clone(),
        animation_completeness: animation_lineage.map(|lineage| {
            DirectCreatureAnimationCompletenessV2 {
                schema_version: 2,
                profile: animation_profile,
                required_clip_count: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len() as u32,
                input_source_clip_count: lineage.input_source_clip_count,
                preserved_source_clip_count: lineage.preserved_source_clip_count,
                source_derived_clip_count: lineage.source_derived_clip_count,
                procedural_clip_count: lineage.procedural_clip_count,
                discarded_source_clip_count: lineage.discarded_source_clips.len() as u32,
                discarded_source_clips: lineage.discarded_source_clips,
                clips: lineage.clips,
                fallback_alias_count: 0,
                complete: true,
            }
        }),
        animation_behavior,
        animation_kinematics_conformance,
        animation_event_conformance,
        animation_event_timing_policy,
        animation_event_authoring_canonical,
        skin_animation_conformance,
        m0_runtime_fixture_contract: None,
        animation_authoring: resolved_animation_authoring.clone(),
        authored_animation_conformance: authored_animation_conformance.clone(),
        held_weapon: held_weapon_readback,
    };
    let report_json = json_bytes(&report, "report")?;
    let summary = M6MaterializationSummaryV1 {
        schema_version: 1,
        status: "M6_MODEL_PACKAGE_MATERIALIZED".to_owned(),
        input_glb: input_glb_identity.clone(),
        input_appearance_two_da: input_appearance_identity.clone(),
        outputs: M6OutputIdentitiesV1 {
            model: identity(&model),
            texture: identity(&texture),
            appearance_two_da: identity(&appearance_two_da),
            hak: identity(&hak),
            proof_module: identity(&proof_module.payload),
            report: identity(&report_json),
        },
        appended_physical_row: report.appearance.appended_row_index,
        model_resref: product_identity.model_resref.clone(),
        texture_resref: product_identity.texture_resref.clone(),
        animation: animation_summary,
        provenance: rig.provenance.clone(),
        zero_reference_model_payload_copied: rig
            .provenance
            .attestations
            .no_reference_payload_copied,
        appearance_payload_policy: "PRESERVED_AND_APPENDED".to_owned(),
        m0_appearance_table: None,
        m0_runtime_fixture_contract: None,
    };
    let summary_json = json_bytes(&summary, "summary")?;
    let mut generated_file_inputs = vec![
        ("generated/source.glb".to_owned(), source_glb),
        (
            format!("generated/{}.mdl", product_identity.model_resref),
            model.as_slice(),
        ),
        (
            format!("generated/{}.tga", product_identity.texture_resref),
            texture.as_slice(),
        ),
        (
            "generated/appearance.2da".to_owned(),
            appearance_two_da.as_slice(),
        ),
        (
            format!("generated/{}.hak", product_identity.hak_resref),
            hak.as_slice(),
        ),
        (
            format!("generated/{}.mod", runtime_identity.module.module_resref),
            proof_module.payload.as_slice(),
        ),
        (
            "reports/materialization-report.json".to_owned(),
            report_json.as_slice(),
        ),
        ("reports/summary.json".to_owned(), summary_json.as_slice()),
    ];
    if let Some(weapon) = held_weapon_build.as_ref()
        && let Some(texture) = weapon.texture.as_ref()
    {
        generated_file_inputs.push((
            format!("generated/{}.tga", weapon.texture_resref),
            texture.payload.as_slice(),
        ));
    }
    let generated_files = generated_file_inputs
        .into_iter()
        .map(|(relative_path, bytes)| {
            let identity = identity(bytes);
            M6GeneratedFileV1 {
                relative_path,
                byte_length: identity.byte_length,
                sha256: identity.sha256,
            }
        })
        .collect();
    let manifest = M6MaterializationManifestV1 {
        schema_version: 1,
        status: "M6_MODEL_PACKAGE_MATERIALIZED".to_owned(),
        input_glb: input_glb_identity,
        input_appearance_two_da: input_appearance_identity,
        texture_selection,
        appended_physical_row: report.appearance.appended_row_index,
        generated_files,
        package_manifest: package_manifest.clone(),
        appearance_payload_policy: "PRESERVED_AND_APPENDED".to_owned(),
        project_identity: project_identity.cloned(),
        m0_appearance_table: None,
        manifest_self_hash_policy: "EXCLUDED_TO_AVOID_SELF_REFERENCE".to_owned(),
        m0_runtime_fixture_contract: None,
        animation_authoring: resolved_animation_authoring,
        authored_animation_conformance,
    };
    let manifest_json = json_bytes(&manifest, "manifest")?;
    Ok(M6BuildArtifactV2::LegacyBundle(Box::new(
        M6ModelPackageInternalV5 {
            artifact: M6ModelPackageArtifactV1 {
                source_glb: source_glb.to_vec(),
                model,
                texture,
                appearance_two_da,
                hak,
                proof_module: proof_module.payload,
                manifest,
                package_manifest,
                manifest_json,
                report,
                report_json,
                summary,
                summary_json,
            },
            animation_studio_materialization,
            animation_studio_rig,
            writer_readback,
        },
    )))
}

fn validate_procedural_creature_product_identity_v2(
    identity: &ProceduralCreatureProductIdentityV2,
) -> Result<(), M6PipelineErrorV1> {
    for (path, value) in [
        ("identity.modelResref", identity.model_resref.as_str()),
        ("identity.textureResref", identity.texture_resref.as_str()),
        ("identity.hakResref", identity.hak_resref.as_str()),
    ] {
        if value.is_empty()
            || value.len() > 16
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(pipeline_error(
                "identity",
                "M6-PRODUCT-RESREF-INVALID",
                path,
                "product resrefs must contain 1..=16 lower-case ASCII letters, digits or underscores",
            ));
        }
    }
    if identity.appearance_label.is_empty()
        || identity.appearance_label.len() > 64
        || !identity
            .appearance_label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(pipeline_error(
            "identity",
            "M6-PRODUCT-APPEARANCE-LABEL-INVALID",
            "identity.appearanceLabel",
            "appearance label must contain 1..=64 ASCII letters, digits or underscores",
        ));
    }
    Ok(())
}

fn merge_required_events_with_integrated_authored_v1(
    animations: &MdlAnimationSetV1,
    required: &DirectCreatureEventAuthoringV1,
) -> DirectCreatureEventAuthoringV1 {
    DirectCreatureEventAuthoringV1 {
        schema_version: required.schema_version,
        clips: required
            .clips
            .iter()
            .map(|required_clip| {
                let mut events = animations
                    .clips
                    .iter()
                    .find(|clip| clip.name.eq_ignore_ascii_case(&required_clip.clip_name))
                    .map(|clip| clip.events.clone())
                    .unwrap_or_default();
                for required_event in &required_clip.events {
                    if !events
                        .iter()
                        .any(|event| event.name.eq_ignore_ascii_case(&required_event.name))
                    {
                        events.push(required_event.clone());
                    }
                }
                events.sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
                DirectCreatureClipEventAuthoringV1 {
                    clip_name: required_clip.clip_name.clone(),
                    events,
                }
            })
            .collect(),
    }
}

pub fn materialize_direct_creature_runtime_clips_v1(
    source: &MdlAnimationSetV1,
    profile: DirectCreatureAnimationProfileV1,
) -> Result<MdlAnimationSetV1, M6PipelineErrorV1> {
    if profile == DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1 {
        return Err(pipeline_error(
            "animation",
            "M6-ANIMATION-PROCEDURAL-RIG-REQUIRED",
            "conversion.creature.nodes",
            "procedural humanoid authoring requires the converted creature rig; use the complete model-package pipeline",
        ));
    }
    if profile == DirectCreatureAnimationProfileV1::FullNative42ExplicitV1 {
        let mut folded_names = std::collections::BTreeSet::new();
        for clip in &source.clips {
            if !folded_names.insert(clip.name.to_ascii_lowercase()) {
                return Err(pipeline_error(
                    "animation",
                    "M6-ANIMATION-FULL-PROFILE-DUPLICATE",
                    "conversion.animations.clips",
                    format!(
                        "full direct-creature profile contains a duplicate clip after ASCII case-fold: {}",
                        clip.name
                    ),
                ));
            }
        }
        let missing = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .filter(|name| !folded_names.contains(**name))
            .copied()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(pipeline_error(
                "animation",
                "M6-ANIMATION-FULL-PROFILE-MISSING",
                "conversion.animations.clips",
                format!(
                    "FULL_NATIVE_42_EXPLICIT_V1 requires all 42 explicitly mapped clips and forbids idle fallback; missing: {}",
                    missing.join(", ")
                ),
            ));
        }
        let unknown = source
            .clips
            .iter()
            .filter(|clip| {
                !FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&clip.name))
            })
            .map(|clip| clip.name.as_str())
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            return Err(pipeline_error(
                "animation",
                "M6-ANIMATION-FULL-PROFILE-UNKNOWN",
                "conversion.animations.clips",
                format!(
                    "FULL_NATIVE_42_EXPLICIT_V1 accepts exactly the 42 classified native states; unknown clips: {}",
                    unknown.join(", ")
                ),
            ));
        }
        return Ok(source.clone());
    }

    let idle = source
        .clips
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case("cpause1"))
        .cloned()
        .ok_or_else(|| {
            pipeline_error(
                "animation",
                "M6-ANIMATION-MISSING",
                "conversion.animations.clips",
                "direct-creature runtime profile requires a mapped cpause1 source clip",
            )
        })?;
    let mut output = source.clone();
    for name in M6_REQUIRED_DIRECT_CREATURE_CLIPS {
        if output
            .clips
            .iter()
            .any(|clip| clip.name.eq_ignore_ascii_case(name))
        {
            continue;
        }
        let mut alias = idle.clone();
        alias.name = name.to_owned();
        output.clips.push(alias);
    }
    Ok(output)
}

fn is_full_native_42_profile(profile: DirectCreatureAnimationProfileV1) -> bool {
    matches!(
        profile,
        DirectCreatureAnimationProfileV1::FullNative42ExplicitV1
            | DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1
    )
}

fn procedural_humanoid_rig_from_creature_v1(
    creature: &AuroraCreatureIrV1,
) -> Result<ProceduralHumanoidRigV1, M6PipelineErrorV1> {
    fn exact_node_id(creature: &AuroraCreatureIrV1, name: &str) -> Result<u32, M6PipelineErrorV1> {
        let matches = creature
            .nodes
            .iter()
            .filter(|node| node.name.eq_ignore_ascii_case(name))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(pipeline_error(
                "animation",
                "M6-ANIMATION-PROCEDURAL-RIG",
                format!("conversion.creature.nodes.{name}"),
                format!(
                    "procedural humanoid authoring requires exactly one {name} node, found {}",
                    matches.len()
                ),
            ));
        }
        Ok(matches[0].id)
    }

    Ok(ProceduralHumanoidRigV1 {
        hips: exact_node_id(creature, "Hips")?,
        spine: exact_node_id(creature, "Spine")?,
        head: exact_node_id(creature, "Head")?,
        left_upper_arm: exact_node_id(creature, "LeftArm")?,
        left_forearm: exact_node_id(creature, "LeftForeArm")?,
        right_upper_arm: exact_node_id(creature, "RightArm")?,
        right_forearm: exact_node_id(creature, "RightForeArm")?,
        left_thigh: exact_node_id(creature, "LeftUpLeg")?,
        left_shin: exact_node_id(creature, "LeftLeg")?,
        right_thigh: exact_node_id(creature, "RightUpLeg")?,
        right_shin: exact_node_id(creature, "RightLeg")?,
    })
}

pub(crate) fn materialize_direct_creature_runtime_clips(
    source: &MdlAnimationSetV1,
) -> Result<MdlAnimationSetV1, M6PipelineErrorV1> {
    materialize_direct_creature_runtime_clips_v1(
        source,
        DirectCreatureAnimationProfileV1::GameplayFloor7IdleFallbackV1,
    )
}

const M6_REQUIRED_SKIN_DEFORMATION_CLIPS_V1: [&str; 4] = ["cwalk", "crun", "ca1slashl", "cdamagel"];

fn evaluate_m6_skin_animation_conformance_v1(
    report: &crate::mdl::InspectionReport,
    active_joint_count: usize,
) -> M6SkinAnimationConformanceV1 {
    let mut violations = Vec::new();
    if active_joint_count < 2 {
        violations.push(format!("ACTIVE_JOINT_COUNT_TOO_LOW:{active_joint_count}"));
    }
    let death_clip_name = "ckdbck";
    let required_clip_names = M6_REQUIRED_SKIN_DEFORMATION_CLIPS_V1
        .into_iter()
        .chain(std::iter::once(death_clip_name))
        .collect::<Vec<_>>();
    let mut clips = Vec::with_capacity(required_clip_names.len());
    for clip_name in &required_clip_names {
        let Some(animation) = report
            .animations
            .iter()
            .find(|animation| animation.name.eq_ignore_ascii_case(clip_name))
        else {
            violations.push(format!("DEFORMATION_CLIP_MISSING:{clip_name}"));
            clips.push(M6ClipSkinAnimationConformanceV1 {
                clip_name: (*clip_name).to_owned(),
                sampled_time_count: 0,
                max_moved_vertex_count: 0,
                max_displacement: 0.0,
                non_rigid_deformation_observed: false,
            });
            continue;
        };
        let probe_times = m6_skin_probe_times(animation);
        let mut sampled_time_count = 0u32;
        let mut max_moved_vertex_count = 0u32;
        let mut max_displacement = 0.0f32;
        let mut non_rigid_deformation_observed = false;
        let mut evaluation_failure = None;
        for time_seconds in probe_times {
            match evaluate_skin_deformation_v1(report, clip_name, time_seconds) {
                Ok(sample) => {
                    sampled_time_count = sampled_time_count.saturating_add(1);
                    max_moved_vertex_count = max_moved_vertex_count.max(sample.moved_vertex_count);
                    max_displacement = max_displacement.max(sample.max_displacement);
                    non_rigid_deformation_observed |= m6_has_non_rigid_shape_change_v1(&sample);
                }
                Err(error) => {
                    evaluation_failure = Some(error.code);
                    break;
                }
            }
        }
        if let Some(code) = evaluation_failure {
            violations.push(format!("DEFORMATION_EVALUATION_FAILED:{clip_name}:{code}"));
        } else if !non_rigid_deformation_observed {
            violations.push(format!("NON_RIGID_DEFORMATION_MISSING:{clip_name}"));
        }
        clips.push(M6ClipSkinAnimationConformanceV1 {
            clip_name: (*clip_name).to_owned(),
            sampled_time_count,
            max_moved_vertex_count,
            max_displacement,
            non_rigid_deformation_observed,
        });
    }
    let complete = violations.is_empty();
    M6SkinAnimationConformanceV1 {
        schema_version: 1,
        active_joint_count: active_joint_count as u32,
        required_clip_count: required_clip_names.len() as u32,
        clips,
        complete,
        violations,
    }
}

fn m6_skin_probe_times(animation: &crate::mdl::AnimationReport) -> Vec<f32> {
    let mut times = Vec::new();
    if animation.length.is_finite() && animation.length > 0.0 {
        times.extend((1..=8).map(|step| animation.length * step as f32 / 8.0));
    }
    for root in &animation.node_tree.roots {
        collect_m6_changing_controller_times(root, animation.length, &mut times);
    }
    times.retain(|time| time.is_finite() && *time >= 0.0 && *time <= animation.length);
    times.sort_by(f32::total_cmp);
    times.dedup_by(|left, right| left.to_bits() == right.to_bits());
    if times.len() <= 64 {
        return times;
    }
    (0..64)
        .map(|index| {
            let source_index = index * (times.len() - 1) / 63;
            times[source_index]
        })
        .collect()
}

fn collect_m6_changing_controller_times(node: &NodeReport, length: f32, output: &mut Vec<f32>) {
    for controller in &node.controllers {
        if !controller.decoded || !controller.values.windows(2).any(|rows| rows[0] != rows[1]) {
            continue;
        }
        output.extend(
            controller
                .times
                .iter()
                .copied()
                .filter(|time| time.is_finite() && *time >= 0.0 && *time <= length),
        );
    }
    for child in &node.children {
        collect_m6_changing_controller_times(child, length, output);
    }
}

fn m6_has_non_rigid_shape_change_v1(sample: &crate::mdl::SkinDeformationSampleV1) -> bool {
    let vertices = sample
        .skins
        .iter()
        .flat_map(|skin| skin.vertices.iter())
        .collect::<Vec<_>>();
    if vertices.len() < 2 {
        return false;
    }
    let anchor_count = vertices.len().min(8);
    for anchor_ordinal in 0..anchor_count {
        let anchor_index = anchor_ordinal * (vertices.len() - 1) / (anchor_count - 1);
        for (vertex_index, vertex) in vertices.iter().enumerate() {
            if vertex_index == anchor_index {
                continue;
            }
            let bind_distance =
                m6_distance3_v1(vertices[anchor_index].bind_world, vertex.bind_world);
            let sampled_distance =
                m6_distance3_v1(vertices[anchor_index].sampled_world, vertex.sampled_world);
            let tolerance = 1.0e-5f32.max(bind_distance * 1.0e-5);
            if (sampled_distance - bind_distance).abs() > tolerance {
                return true;
            }
        }
    }
    false
}

fn m6_distance3_v1(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2))
        .sqrt()
}

fn static_direct_creature_runtime_clips(
    model_resref: &str,
    root_node_id: u32,
) -> MdlAnimationSetV1 {
    MdlAnimationSetV1 {
        schema_version: 1,
        clips: M6_REQUIRED_DIRECT_CREATURE_CLIPS
            .into_iter()
            .map(|name| MdlAnimationClipV1 {
                name: name.to_owned(),
                animation_root: model_resref.to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: vec![
                    MdlAnimationTrackV1 {
                        target_node_id: root_node_id,
                        path: MdlAnimationTrackPathV1::Translation,
                        interpolation: MdlAnimationInterpolationV1::Linear,
                        times_seconds: vec![0.0, 1.0],
                        values: vec![vec![0.0, 0.0, 0.0], vec![0.0, 0.0, 0.0]],
                    },
                    MdlAnimationTrackV1 {
                        target_node_id: root_node_id,
                        path: MdlAnimationTrackPathV1::Rotation,
                        interpolation: MdlAnimationInterpolationV1::Linear,
                        times_seconds: vec![0.0, 1.0],
                        values: vec![vec![0.0, 0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0, 1.0]],
                    },
                ],
            })
            .collect(),
    }
}

/// Independent deterministic replay used by the V2 verifier. It starts from
/// exact source GLB bytes and never trusts an external generated MDL/report.
fn replay_m0_direct_creature_model_v2(
    source_glb: &[u8],
    runtime_profile: &DirectCreatureRuntimeProfileV2,
) -> Result<Vec<u8>, M6PipelineErrorV1> {
    let mut ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut ingest)?;
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    let conversion = convert_profile_a(&ingest, &rig, &Default::default()).map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    let mut creature = conversion.creature.ok_or_else(|| {
        pipeline_error(
            "runtime_fixture_contract",
            "M0-RUNTIME-CONTRACT-REPLAY-INELIGIBLE",
            "sourceGlb",
            "deterministic replay did not produce a direct-creature model",
        )
    })?;
    let root_node_id = creature
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .map(|node| node.id)
        .ok_or_else(|| {
            pipeline_error(
                "runtime_fixture_contract",
                "M0-RUNTIME-CONTRACT-REPLAY-ROOT",
                "sourceGlb",
                "deterministic replay has no output root",
            )
        })?;
    let mut animations =
        static_direct_creature_runtime_clips(&runtime_profile.model_resref, root_node_id);
    normalize_direct_creature_runtime_root(
        &mut creature,
        &mut animations,
        &runtime_profile.model_resref,
    )?;
    split_creature_segments_for_binary_mdl_v1(&mut creature)?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, &creature)?;
    let mdl = write_binary_mdl_with_animations(
        &creature,
        &animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M0StaticRigidNativeV1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: runtime_profile.model_resref.clone(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: texture_selection.material_slot,
                resref: M0_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| {
        pipeline_error(
            "runtime_fixture_contract",
            error.code,
            error.path,
            error.message,
        )
    })?;
    Ok(mdl.payload)
}

fn split_creature_segments_for_binary_mdl_v1(
    creature: &mut AuroraCreatureIrV1,
) -> Result<(), M6PipelineErrorV1> {
    segment_model_for_binary_mdl_v1(creature)
        .map(|_| ())
        .map_err(|error| pipeline_error("model", error.code, error.path, error.message))
}

fn sanitize_runtime_creature_face_planes_v1(
    creature: &mut AuroraCreatureIrV1,
) -> Result<usize, M6PipelineErrorV1> {
    sanitize_runtime_creature_face_planes_with_policy_v1(
        creature,
        TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
    )
}

fn sanitize_runtime_creature_face_planes_exact_v1(
    creature: &mut AuroraCreatureIrV1,
) -> Result<usize, M6PipelineErrorV1> {
    sanitize_runtime_creature_face_planes_with_policy_v1(
        creature,
        TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear,
    )
}

fn sanitize_runtime_creature_face_planes_with_policy_v1(
    creature: &mut AuroraCreatureIrV1,
    degeneracy_policy: TriangleDegeneracyPolicyV1,
) -> Result<usize, M6PipelineErrorV1> {
    let source_segments = std::mem::take(&mut creature.segments);
    let mut output_segments = Vec::with_capacity(source_segments.len());
    let mut removed_triangle_count = 0usize;
    for (segment_index, mut segment) in source_segments.into_iter().enumerate() {
        let path = format!("creature.segments[{segment_index}]");
        let triangle_count = segment.indices.len() / 3;
        if segment.indices.is_empty()
            || !segment.indices.len().is_multiple_of(3)
            || segment.positions.len() != segment.normals.len()
            || segment.positions.len() != segment.uv0.len()
            || segment
                .tangents
                .as_ref()
                .is_some_and(|values| values.len() != segment.positions.len())
            || (segment.deformation == RigSegmentDeformationV1::Skin
                && segment.weights.len() != segment.positions.len())
            || (segment.deformation == RigSegmentDeformationV1::Rigid
                && !segment.weights.is_empty())
            || (!segment.face_surface_ids.is_empty()
                && segment.face_surface_ids.len() != triangle_count)
        {
            return Err(pipeline_error(
                "model",
                "M6-RUNTIME-FACE-PLANE-SOURCE-INVALID",
                path,
                "runtime segment arrays do not form complete indexed triangles",
            ));
        }

        let mut retained_triangles = Vec::with_capacity(triangle_count);
        for (triangle_index, triangle) in segment.indices.chunks_exact(3).enumerate() {
            let mut vertices = [0usize; 3];
            for (corner, source_index) in triangle.iter().copied().enumerate() {
                let vertex = usize::try_from(source_index).map_err(|_| {
                    pipeline_error(
                        "model",
                        "M6-RUNTIME-FACE-PLANE-INDEX",
                        &path,
                        "runtime vertex index does not fit this platform",
                    )
                })?;
                if vertex >= segment.positions.len() {
                    return Err(pipeline_error(
                        "model",
                        "M6-RUNTIME-FACE-PLANE-INDEX",
                        &path,
                        "runtime vertex index escapes the segment arrays",
                    ));
                }
                vertices[corner] = vertex;
            }
            let a = segment.positions[vertices[0]];
            let b = segment.positions[vertices[1]];
            let c = segment.positions[vertices[2]];
            let ab = [
                f64::from(b[0]) - f64::from(a[0]),
                f64::from(b[1]) - f64::from(a[1]),
                f64::from(b[2]) - f64::from(a[2]),
            ];
            let ac = [
                f64::from(c[0]) - f64::from(a[0]),
                f64::from(c[1]) - f64::from(a[1]),
                f64::from(c[2]) - f64::from(a[2]),
            ];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let length = (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt();
            if degeneracy_policy.accepts_cross_length(length) {
                retained_triangles.push(triangle_index);
            }
        }

        let removed = triangle_count.saturating_sub(retained_triangles.len());
        removed_triangle_count = removed_triangle_count.checked_add(removed).ok_or_else(|| {
            pipeline_error(
                "model",
                "M6-RUNTIME-FACE-PLANE-OVERFLOW",
                "creature.segments",
                "removed triangle count overflow",
            )
        })?;
        if removed == 0 {
            output_segments.push(segment);
            continue;
        }
        if retained_triangles.is_empty() {
            continue;
        }

        let source_positions = std::mem::take(&mut segment.positions);
        let source_normals = std::mem::take(&mut segment.normals);
        let source_tangents = segment.tangents.take();
        let source_uv0 = std::mem::take(&mut segment.uv0);
        let source_indices = std::mem::take(&mut segment.indices);
        let source_face_surface_ids = std::mem::take(&mut segment.face_surface_ids);
        let source_weights = std::mem::take(&mut segment.weights);
        let mut remap = vec![u32::MAX; source_positions.len()];
        let mut compacted_tangents = source_tangents.as_ref().map(|_| Vec::new());
        for triangle_index in retained_triangles {
            for source_index in &source_indices[triangle_index * 3..(triangle_index + 1) * 3] {
                let source_vertex = usize::try_from(*source_index).map_err(|_| {
                    pipeline_error(
                        "model",
                        "M6-RUNTIME-FACE-PLANE-INDEX",
                        &path,
                        "runtime vertex index does not fit this platform",
                    )
                })?;
                if remap[source_vertex] == u32::MAX {
                    remap[source_vertex] =
                        u32::try_from(segment.positions.len()).map_err(|_| {
                            pipeline_error(
                                "model",
                                "M6-RUNTIME-FACE-PLANE-OVERFLOW",
                                &path,
                                "compacted runtime vertex index exceeds u32",
                            )
                        })?;
                    segment.positions.push(source_positions[source_vertex]);
                    segment.normals.push(source_normals[source_vertex]);
                    segment.uv0.push(source_uv0[source_vertex]);
                    if let (Some(source), Some(output)) =
                        (source_tangents.as_ref(), compacted_tangents.as_mut())
                    {
                        output.push(source[source_vertex]);
                    }
                    if segment.deformation == RigSegmentDeformationV1::Skin {
                        segment.weights.push(source_weights[source_vertex].clone());
                    }
                }
                segment.indices.push(remap[source_vertex]);
            }
            if !source_face_surface_ids.is_empty() {
                segment
                    .face_surface_ids
                    .push(source_face_surface_ids[triangle_index]);
            }
        }
        segment.tangents = compacted_tangents;
        output_segments.push(segment);
    }
    if output_segments.is_empty() {
        return Err(pipeline_error(
            "model",
            "M6-RUNTIME-FACE-PLANE-EMPTY",
            "creature.segments",
            "runtime creature contains no binary-writer-safe triangles",
        ));
    }
    creature.segments = output_segments;
    Ok(removed_triangle_count)
}

fn insert_controllerless_aurora_skin_root_v1(
    creature: &mut AuroraCreatureIrV1,
    model_resref: &str,
) -> Result<(), M6PipelineErrorV1> {
    let roots = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    let [skeleton_root_index] = roots.as_slice() else {
        return Err(pipeline_error(
            "animation",
            "M6-PROCEDURAL-AURORA-ROOT",
            "conversion.creature.nodes",
            "procedural humanoid output requires exactly one source skeleton root",
        ));
    };
    if creature
        .nodes
        .iter()
        .any(|node| node.name.eq_ignore_ascii_case(model_resref))
    {
        return Err(pipeline_error(
            "animation",
            "M6-PROCEDURAL-AURORA-ROOT",
            "modelResourceResref",
            "dedicated Aurora root name collides with a source skeleton node",
        ));
    }
    let aurora_root_id = creature
        .nodes
        .iter()
        .map(|node| node.id)
        .max()
        .unwrap_or_default()
        .checked_add(1)
        .ok_or_else(|| {
            pipeline_error(
                "animation",
                "M6-PROCEDURAL-AURORA-ROOT",
                "conversion.creature.nodes",
                "cannot allocate a distinct Aurora root node id",
            )
        })?;
    let skeleton_root_id = creature.nodes[*skeleton_root_index].id;
    let skeleton_root_bind = creature.nodes[*skeleton_root_index].bind_local_matrix;
    let mut skin_count = 0usize;
    for (segment_index, segment) in creature.segments.iter_mut().enumerate() {
        if segment.deformation != RigSegmentDeformationV1::Skin {
            continue;
        }
        skin_count += 1;
        if segment.parent_node_id != skeleton_root_id {
            return Err(pipeline_error(
                "animation",
                "M6-PROCEDURAL-SKIN-PARENT",
                format!("conversion.creature.segments[{segment_index}].parentNodeId"),
                "procedural SkinMesh must initially be parented to the sole skeleton root",
            ));
        }
        for position in &mut segment.positions {
            *position = transform_affine_point(skeleton_root_bind, *position).ok_or_else(|| {
                pipeline_error(
                    "animation",
                    "M6-PROCEDURAL-SKIN-REPARENT",
                    format!("conversion.creature.segments[{segment_index}].positions"),
                    "baking the skeleton-root bind transform produced a non-finite position",
                )
            })?;
        }
        for normal in &mut segment.normals {
            *normal = transform_affine_direction(skeleton_root_bind, *normal).ok_or_else(|| {
                pipeline_error(
                    "animation",
                    "M6-PROCEDURAL-SKIN-REPARENT",
                    format!("conversion.creature.segments[{segment_index}].normals"),
                    "baking the skeleton-root bind transform produced an invalid normal",
                )
            })?;
        }
        if let Some(tangents) = &mut segment.tangents {
            for tangent in tangents {
                let direction = transform_affine_direction(
                    skeleton_root_bind,
                    [tangent[0], tangent[1], tangent[2]],
                )
                .ok_or_else(|| {
                    pipeline_error(
                        "animation",
                        "M6-PROCEDURAL-SKIN-REPARENT",
                        format!("conversion.creature.segments[{segment_index}].tangents"),
                        "baking the skeleton-root bind transform produced an invalid tangent",
                    )
                })?;
                *tangent = [direction[0], direction[1], direction[2], tangent[3]];
            }
        }
        segment.parent_node_id = aurora_root_id;
    }
    if skin_count == 0 {
        return Err(pipeline_error(
            "animation",
            "M6-PROCEDURAL-SKIN-MISSING",
            "conversion.creature.segments",
            "procedural full creature output requires at least one weighted SkinMesh",
        ));
    }

    creature.nodes[*skeleton_root_index].parent_id = Some(aurora_root_id);
    creature.nodes.push(crate::profile_a::AuroraCreatureNodeV1 {
        id: aurora_root_id,
        name: model_resref.to_owned(),
        parent_id: None,
        bind_local_matrix: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    });
    Ok(())
}

fn transform_affine_point(matrix: [f32; 16], value: [f32; 3]) -> Option<[f32; 3]> {
    let output = [
        matrix[0] * value[0] + matrix[4] * value[1] + matrix[8] * value[2] + matrix[12],
        matrix[1] * value[0] + matrix[5] * value[1] + matrix[9] * value[2] + matrix[13],
        matrix[2] * value[0] + matrix[6] * value[1] + matrix[10] * value[2] + matrix[14],
    ];
    output
        .iter()
        .all(|component| component.is_finite())
        .then_some(output)
}

fn transform_affine_direction(matrix: [f32; 16], value: [f32; 3]) -> Option<[f32; 3]> {
    let output = [
        matrix[0] * value[0] + matrix[4] * value[1] + matrix[8] * value[2],
        matrix[1] * value[0] + matrix[5] * value[1] + matrix[9] * value[2],
        matrix[2] * value[0] + matrix[6] * value[1] + matrix[10] * value[2],
    ];
    let length = (output[0] * output[0] + output[1] * output[1] + output[2] * output[2]).sqrt();
    if !length.is_finite() || length <= f32::EPSILON {
        return None;
    }
    Some([output[0] / length, output[1] / length, output[2] / length])
}

/// Normalizes the generated, self-contained creature to Aurora's direct-model
/// convention: the sole geometry root and every animation root use the model
/// resref. Local `c_squirrel` has this shape; the data below the root remains
/// wholly derived from the user-provided Meshy rig and animation.
fn normalize_direct_creature_runtime_root(
    creature: &mut crate::profile_a::AuroraCreatureIrV1,
    animations: &mut MdlAnimationSetV1,
    model_resref: &str,
) -> Result<(), M6PipelineErrorV1> {
    let roots = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(pipeline_error(
            "animation",
            "M6-ANIMROOT-INVALID",
            "conversion.creature.nodes",
            "direct-creature runtime profile requires exactly one geometry root",
        ));
    }
    creature.nodes[roots[0]].name = model_resref.to_owned();
    for clip in &mut animations.clips {
        clip.animation_root = model_resref.to_owned();
    }
    Ok(())
}

pub(crate) fn sanitize_meshy_h1_degenerate_triangles_v1(
    source: &mut GlbIngestResult,
) -> Result<(), M6PipelineErrorV1> {
    sanitize_meshy_h1_degenerate_triangles_with_policy_v1(
        source,
        TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon,
    )
}

fn sanitize_meshy_h1_degenerate_triangles_exact_v1(
    source: &mut GlbIngestResult,
) -> Result<(), M6PipelineErrorV1> {
    sanitize_meshy_h1_degenerate_triangles_with_policy_v1(
        source,
        TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear,
    )
}

fn sanitize_meshy_h1_degenerate_triangles_with_policy_v1(
    source: &mut GlbIngestResult,
    degeneracy_policy: TriangleDegeneracyPolicyV1,
) -> Result<(), M6PipelineErrorV1> {
    if source.ir.primitives.len() != 1 {
        return Err(pipeline_error(
            "profile",
            "M4A-MESHY-H1-SOURCE-INVALID",
            "source.ir.primitives",
            "Meshy H1 route requires exactly one primitive before sanitation",
        ));
    }
    let primitive = &mut source.ir.primitives[0];
    let before = primitive.indices.len() / 3;
    let mut retained = Vec::with_capacity(primitive.indices.len());
    for triangle in primitive.indices.chunks_exact(3) {
        let a = primitive.positions[triangle[0] as usize];
        let b = primitive.positions[triangle[1] as usize];
        let c = primitive.positions[triangle[2] as usize];
        let ab = [
            f64::from(b[0]) - f64::from(a[0]),
            f64::from(b[1]) - f64::from(a[1]),
            f64::from(b[2]) - f64::from(a[2]),
        ];
        let ac = [
            f64::from(c[0]) - f64::from(a[0]),
            f64::from(c[1]) - f64::from(a[1]),
            f64::from(c[2]) - f64::from(a[2]),
        ];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let length = (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt();
        if degeneracy_policy.accepts_cross_length(length) {
            retained.extend_from_slice(triangle);
        }
    }
    if retained.is_empty() {
        return Err(pipeline_error(
            "profile",
            "M4A-MESHY-H1-SOURCE-INVALID",
            "source.ir.primitives[0].indices",
            "Meshy H1 source contains no Aurora-safe non-degenerate triangles",
        ));
    }
    let source_vertex_count = primitive.positions.len();
    let has_skin_lanes = !primitive.joints0.is_empty() || !primitive.weights0.is_empty();
    if primitive.normals.len() != source_vertex_count
        || primitive.uv0.len() != source_vertex_count
        || (has_skin_lanes
            && (primitive.joints0.len() != source_vertex_count
                || primitive.weights0.len() != source_vertex_count))
        || (!primitive.tangents.is_empty() && primitive.tangents.len() != source_vertex_count)
    {
        return Err(pipeline_error(
            "profile",
            "M4A-MESHY-H1-SOURCE-INVALID",
            "source.ir.primitives[0]",
            "Meshy H1 vertex attributes differ before sanitation",
        ));
    }
    let source_positions = std::mem::take(&mut primitive.positions);
    let source_normals = std::mem::take(&mut primitive.normals);
    let source_tangents = std::mem::take(&mut primitive.tangents);
    let source_uv0 = std::mem::take(&mut primitive.uv0);
    let source_joints0 = std::mem::take(&mut primitive.joints0);
    let source_weights0 = std::mem::take(&mut primitive.weights0);
    let mut remap = vec![u32::MAX; source_vertex_count];
    let mut compacted_indices = Vec::with_capacity(retained.len());
    for source_index in retained {
        let source_vertex = usize::try_from(source_index).map_err(|_| {
            pipeline_error(
                "profile",
                "M4A-MESHY-H1-SOURCE-INVALID",
                "source.ir.primitives[0].indices",
                "Meshy H1 source index does not fit this platform",
            )
        })?;
        if source_vertex >= source_vertex_count {
            return Err(pipeline_error(
                "profile",
                "M4A-MESHY-H1-SOURCE-INVALID",
                "source.ir.primitives[0].indices",
                "Meshy H1 source index escapes the vertex attributes",
            ));
        }
        if remap[source_vertex] == u32::MAX {
            remap[source_vertex] = u32::try_from(primitive.positions.len()).map_err(|_| {
                pipeline_error(
                    "profile",
                    "M4A-MESHY-H1-SOURCE-INVALID",
                    "source.ir.primitives[0].positions",
                    "compacted Meshy H1 vertex index exceeds u32",
                )
            })?;
            primitive.positions.push(source_positions[source_vertex]);
            primitive.normals.push(source_normals[source_vertex]);
            if !source_tangents.is_empty() {
                primitive.tangents.push(source_tangents[source_vertex]);
            }
            primitive.uv0.push(source_uv0[source_vertex]);
            if has_skin_lanes {
                primitive.joints0.push(source_joints0[source_vertex]);
                primitive.weights0.push(source_weights0[source_vertex]);
            }
        }
        compacted_indices.push(remap[source_vertex]);
    }
    primitive.indices = compacted_indices;
    let removed_unreferenced_vertices =
        source_vertex_count.saturating_sub(primitive.positions.len());
    primitive.bounds_min = [f32::INFINITY; 3];
    primitive.bounds_max = [f32::NEG_INFINITY; 3];
    for position in &primitive.positions {
        for (axis, value) in position.iter().copied().enumerate() {
            primitive.bounds_min[axis] = primitive.bounds_min[axis].min(value);
            primitive.bounds_max[axis] = primitive.bounds_max[axis].max(value);
        }
    }
    let after = primitive.indices.len() / 3;
    let removed = before.saturating_sub(after);
    source.report.statistics.vertex_count = source
        .ir
        .primitives
        .iter()
        .map(|item| item.positions.len())
        .sum();
    source.report.statistics.index_count = source
        .ir
        .primitives
        .iter()
        .map(|item| item.indices.len())
        .sum();
    source.report.statistics.triangle_count = source
        .ir
        .primitives
        .iter()
        .map(|item| item.indices.len() / 3)
        .sum();
    if removed > 0 {
        let (code, description) = match degeneracy_policy {
            TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon => (
                "M4A-MESHY-H1-DEGENERATE-TRIANGLES-REMOVED",
                "degenerate source triangles",
            ),
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear => (
                "M4A-MESHY-H1-EXACT-DEGENERATE-TRIANGLES-REMOVED",
                "non-finite, zero-area or exactly collinear source triangles",
            ),
        };
        source.report.diagnostics.push(crate::glb::GlbDiagnostic {
            schema_version: 1,
            severity: "WARNING".to_owned(),
            code: code.to_owned(),
            message: format!("removed {removed} {description} before Aurora materialization"),
            byte_offset: None,
            json_path: Some("meshes[0].primitives[0].indices".to_owned()),
        });
    }
    if removed_unreferenced_vertices > 0 {
        source.report.diagnostics.push(crate::glb::GlbDiagnostic {
            schema_version: 1,
            severity: "WARNING".to_owned(),
            code: "M4A-MESHY-H1-UNREFERENCED-VERTICES-REMOVED".to_owned(),
            message: format!(
                "removed {removed_unreferenced_vertices} unreferenced source vertices after triangle sanitation"
            ),
            byte_offset: None,
            json_path: Some("meshes[0].primitives[0].attributes".to_owned()),
        });
    }
    Ok(())
}

/// Resolves the first used primitive's base color by stable GLB IDs rather
/// than assuming material, texture and image vectors share an index.
pub fn resolve_base_color_image_index_v1(
    ingest: &GlbIngestResult,
    creature: &crate::profile_a::AuroraCreatureIrV1,
) -> Result<M6TextureSelectionV1, M6PipelineErrorV1> {
    let binding = creature.material_source_bindings.first().ok_or_else(|| {
        pipeline_error(
            "texture",
            "M6-BASE-COLOR-MATERIAL-MISSING",
            "creature.materialSourceBindings",
            "converted model has no used material binding",
        )
    })?;
    let material_id = binding.source_material_id.ok_or_else(|| {
        pipeline_error(
            "texture",
            "M6-BASE-COLOR-MATERIAL-MISSING",
            "creature.materialSourceBindings[0].sourceMaterialId",
            "used material binding has no source material id",
        )
    })?;
    let material = ingest
        .ir
        .materials
        .iter()
        .find(|candidate| candidate.id == material_id)
        .ok_or_else(|| {
            pipeline_error(
                "texture",
                "M6-BASE-COLOR-MATERIAL-MISSING",
                "materials",
                format!("material id {material_id} is absent"),
            )
        })?;
    let texture_id = material
        .base_color_texture
        .as_ref()
        .ok_or_else(|| {
            pipeline_error(
                "texture",
                "M6-BASE-COLOR-TEXTURE-MISSING",
                format!("materials[{material_id}].baseColorTexture"),
                "used material has no base-color texture",
            )
        })?
        .texture_id;
    let texture = ingest
        .ir
        .textures
        .iter()
        .find(|candidate| candidate.id == texture_id)
        .ok_or_else(|| {
            pipeline_error(
                "texture",
                "M6-BASE-COLOR-TEXTURE-MISSING",
                "textures",
                format!("texture id {texture_id} is absent"),
            )
        })?;
    let image_index = ingest
        .ir
        .images
        .iter()
        .position(|image| image.id == texture.source_image_id)
        .ok_or_else(|| {
            pipeline_error(
                "texture",
                "M6-BASE-COLOR-IMAGE-MISSING",
                "images",
                format!("image id {} is absent", texture.source_image_id),
            )
        })?;
    let image = &ingest.ir.images[image_index];
    Ok(M6TextureSelectionV1 {
        material_slot: binding.slot,
        source_material_id: material_id,
        source_texture_id: texture_id,
        source_image_id: image.id,
        source_image_index: image_index,
        source_image_sha256: image.sha256.clone(),
    })
}

pub fn write_m6_proof_packet_v1(
    output_dir: &Path,
    artifact: &M6ModelPackageArtifactV1,
) -> Result<(), M6PipelineErrorV1> {
    write_proof_packet_v1(
        output_dir,
        artifact,
        M6_MODEL_RESREF,
        M6_TEXTURE_RESREF,
        M6_HAK_FILE_NAME,
        M6_PROOF_MODULE_FILE_NAME,
    )
}

/// Writes a fresh full-42 procedural creature packet using only the resource
/// identity that was bound into the verified artifact. Unlike the historical
/// M6 writer, this route does not reuse fixed MOD, HAK, model or texture names.
pub fn write_procedural_creature_proof_packet_with_identity_v1(
    output_dir: &Path,
    artifact: &M6ModelPackageArtifactV1,
    identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<(), M6PipelineErrorV1> {
    let archive = ErfArchive::parse(&artifact.hak).map_err(|error| {
        pipeline_error(
            "output",
            error.code,
            format!("artifact.hak@{}", error.offset),
            error.context,
        )
    })?;
    for (resref, resource_type, expected) in [
        (
            identity.model_resref.as_str(),
            2002,
            artifact.model.as_slice(),
        ),
        (
            identity.texture_resref.as_str(),
            3,
            artifact.texture.as_slice(),
        ),
        ("appearance", 2017, artifact.appearance_two_da.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            pipeline_error(
                "output",
                error.code,
                format!("artifact.hak.{resref}"),
                error.context,
            )
        })?;
        if actual != expected {
            return Err(pipeline_error(
                "output",
                "M6-PROCEDURAL-WRITER-HAK-RESOURCE-DIFF",
                format!("artifact.hak.{resref}"),
                "the HAK resource does not equal the independently held artifact payload",
            ));
        }
    }

    let model = inspect_binary_mdl(&artifact.model).map_err(|error| {
        pipeline_error(
            "output",
            error.code,
            format!("artifact.model@{}", error.offset),
            error.context,
        )
    })?;
    let model_root = model.node_tree.roots.first().ok_or_else(|| {
        pipeline_error(
            "output",
            "M6-PROCEDURAL-WRITER-MODEL-ROOT-MISSING",
            "artifact.model.nodeTree.roots",
            "the procedural model must contain one dedicated Aurora root",
        )
    })?;
    let animation_roots_match = model.animations.iter().all(|animation| {
        animation.animation_type == 5
            && animation.node_tree.roots.len() == 1
            && animation.node_tree.roots[0].name == identity.model_resref
            && animation.node_tree.roots[0].number == 0
    });
    if model_root.name != identity.model_resref
        || model_root.number != 0
        || model.animations.len() != FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.len()
        || !animation_roots_match
    {
        return Err(pipeline_error(
            "output",
            "M6-PROCEDURAL-WRITER-MODEL-IDENTITY-DIFF",
            "artifact.model",
            "model root and all 42 type-5 animation roots must match the caller-owned part-0 identity",
        ));
    }

    let module = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module).map_err(
        |error| {
            pipeline_error(
                "output",
                error.code,
                format!("artifact.proofModule.{}", error.path),
                error.message,
            )
        },
    )?;
    if module.scene.module_resref != identity.module.module_resref
        || module.scene.area_resref != identity.module.area_resref
        || module.scene.ordered_hak_resrefs != [identity.module.hak_resref.clone()]
        || module.fixtures.len() != 1
        || module.fixtures[0].runtime_profile
            != BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
        || module.scene.fixtures.len() != 1
        || module.scene.fixtures[0].template_resref != identity.creature_resref
    {
        return Err(pipeline_error(
            "output",
            "M6-PROCEDURAL-WRITER-MODULE-IDENTITY-DIFF",
            "artifact.proofModule",
            "MOD, Area, ordered HAK and creature identities must equal the caller-owned artifact identity",
        ));
    }

    write_proof_packet_v1(
        output_dir,
        artifact,
        &identity.model_resref,
        &identity.texture_resref,
        &format!("{}.hak", identity.module.hak_resref),
        &format!("{}.mod", identity.module.module_resref),
    )
}

/// Writes an immutable Animation Studio V5 demo packet after independently
/// re-reading the HAK, binary MDL and active-monster MOD identities. Authored
/// library clips may extend the native 42-state inventory, but every native
/// state must remain present and the V5 authored readback must be `MATCH`.
pub fn write_animation_studio_v5_demo_packet_with_identity_v1(
    output_dir: &Path,
    artifact: &M6ModelPackageArtifactV5,
    identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<(), M6PipelineErrorV1> {
    let archive = ErfArchive::parse(&artifact.hak).map_err(|error| {
        pipeline_error(
            "output",
            error.code,
            format!("artifact.hak@{}", error.offset),
            error.context,
        )
    })?;
    for (resref, resource_type, expected) in [
        (
            identity.model_resref.as_str(),
            2002,
            artifact.model.as_slice(),
        ),
        (
            identity.texture_resref.as_str(),
            3,
            artifact.texture.as_slice(),
        ),
        ("appearance", 2017, artifact.appearance_two_da.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            pipeline_error(
                "output",
                error.code,
                format!("artifact.hak.{resref}"),
                error.context,
            )
        })?;
        if actual != expected {
            return Err(pipeline_error(
                "output",
                "M2A-ANIMATION-STUDIO-DEMO-HAK-RESOURCE-DIFF",
                format!("artifact.hak.{resref}"),
                "the HAK resource does not equal the independently held V5 payload",
            ));
        }
    }

    if artifact.animation_studio_readback.status != AnimationStudioReadbackStatusV1::Match {
        return Err(pipeline_error(
            "output",
            "M2A-ANIMATION-STUDIO-DEMO-READBACK-MISMATCH",
            "artifact.animationStudioReadback",
            "a V5 demo packet requires an exact authored-animation MATCH readback",
        ));
    }
    let model = inspect_binary_mdl(&artifact.model).map_err(|error| {
        pipeline_error(
            "output",
            error.code,
            format!("artifact.model@{}", error.offset),
            error.context,
        )
    })?;
    let model_root = model.node_tree.roots.first().ok_or_else(|| {
        pipeline_error(
            "output",
            "M2A-ANIMATION-STUDIO-DEMO-MODEL-ROOT-MISSING",
            "artifact.model.nodeTree.roots",
            "the V5 model must contain one dedicated Aurora root",
        )
    })?;
    let animation_roots_match = model.animations.iter().all(|animation| {
        animation.animation_type == 5
            && animation.node_tree.roots.len() == 1
            && animation.node_tree.roots[0].name == identity.model_resref
            && animation.node_tree.roots[0].number == 0
    });
    let native_inventory_complete = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.iter().all(|name| {
        model
            .animations
            .iter()
            .filter(|animation| animation.name.eq_ignore_ascii_case(name))
            .count()
            == 1
    });
    if model_root.name != identity.model_resref
        || model_root.number != 0
        || !native_inventory_complete
        || !animation_roots_match
    {
        return Err(pipeline_error(
            "output",
            "M2A-ANIMATION-STUDIO-DEMO-MODEL-IDENTITY-DIFF",
            "artifact.model",
            "model root, all native states and all type-5 animation roots must match the caller-owned identity",
        ));
    }

    let module = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module).map_err(
        |error| {
            pipeline_error(
                "output",
                error.code,
                format!("artifact.proofModule.{}", error.path),
                error.message,
            )
        },
    )?;
    if module.scene.module_resref != identity.module.module_resref
        || module.scene.area_resref != identity.module.area_resref
        || module.scene.ordered_hak_resrefs != [identity.module.hak_resref.clone()]
        || module.fixtures.len() != 1
        || module.fixtures[0].runtime_profile
            != BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
        || module.scene.fixtures.len() != 1
        || module.scene.fixtures[0].template_resref != identity.creature_resref
        || module.scene.fixtures[0].appearance_row
            != artifact.report.base.appearance.appended_row_index
    {
        return Err(pipeline_error(
            "output",
            "M2A-ANIMATION-STUDIO-DEMO-MODULE-IDENTITY-DIFF",
            "artifact.proofModule",
            "MOD, Area, ordered HAK, hostile runtime profile, creature and appearance identities must match the V5 artifact",
        ));
    }

    if output_dir.exists() {
        return Err(pipeline_error(
            "output",
            "M6-OUTPUT-EXISTS",
            "outputDir",
            "output directory must not already exist",
        ));
    }
    let parent = output_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| io_error("output", "M6-OUTPUT-CREATE-FAILED", parent, error))?;
    let name = output_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("animation-studio-v5-demo");
    let staging = parent.join(format!(".{name}.m2a-stage-{}", std::process::id()));
    if staging.exists() {
        return Err(pipeline_error(
            "output",
            "M6-STAGING-EXISTS",
            logical_path(&staging),
            "pre-existing staging directory is never deleted",
        ));
    }
    let write_result = (|| {
        let generated = staging.join("generated");
        let reports = staging.join("reports");
        for path in [&generated, &reports, &staging.join("live")] {
            fs::create_dir_all(path)
                .map_err(|error| io_error("output", "M6-OUTPUT-CREATE-FAILED", path, error))?;
        }
        for (path, bytes) in [
            (generated.join("source.glb"), artifact.source_glb.as_slice()),
            (
                generated.join(format!("{}.mdl", identity.model_resref)),
                artifact.model.as_slice(),
            ),
            (
                generated.join(format!("{}.tga", identity.texture_resref)),
                artifact.texture.as_slice(),
            ),
            (
                generated.join("appearance.2da"),
                artifact.appearance_two_da.as_slice(),
            ),
            (
                generated.join(format!("{}.hak", identity.module.hak_resref)),
                artifact.hak.as_slice(),
            ),
            (
                generated.join(format!("{}.mod", identity.module.module_resref)),
                artifact.proof_module.as_slice(),
            ),
            (
                reports.join("materialization-report.json"),
                artifact.report_json.as_slice(),
            ),
            (
                reports.join("summary.json"),
                artifact.summary_json.as_slice(),
            ),
            (
                reports.join(M6_MANIFEST_FILE_NAME),
                artifact.manifest_json.as_slice(),
            ),
        ] {
            fs::write(&path, bytes)
                .map_err(|error| io_error("output", "M6-OUTPUT-WRITE-FAILED", &path, error))?;
        }
        Ok::<(), M6PipelineErrorV1>(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    fs::rename(&staging, output_dir).map_err(|error| {
        let _ = fs::remove_dir_all(&staging);
        io_error("output", "M6-OUTPUT-RENAME-FAILED", output_dir, error)
    })?;
    Ok(())
}

/// Writes an immutable owner-proof packet around the active product V2
/// artifact and its independently built demo V2 module. The product bytes are
/// never rebuilt, and the independently supplied source must match the
/// product's recorded input identity.
pub fn write_procedural_creature_product_demo_packet_v2(
    output_dir: &Path,
    product: &ProceduralCreatureProductArtifactV2,
    demo: &ProofModuleArtifactV1,
    source_glb: &[u8],
    module_identity: &BinaryCreatureModuleIdentityV1,
    creature_resref: &str,
) -> Result<(), M6PipelineErrorV1> {
    if output_dir.exists() {
        return Err(pipeline_error(
            "output",
            "M6-OUTPUT-EXISTS",
            "outputDir",
            "output directory must not already exist",
        ));
    }
    let source_identity = identity(source_glb);
    if source_identity != product.summary.input_glb || source_identity != product.manifest.input_glb
    {
        return Err(pipeline_error(
            "output",
            "M6-DEMO-PACKET-SOURCE-DIFF",
            "sourceGlb",
            "the independently supplied source bytes do not match the immutable product input identity",
        ));
    }
    if product.report.identity != product.summary.identity
        || product.report.identity != product.manifest.identity
        || identity(&product.model) != product.summary.outputs.model
        || identity(&product.texture) != product.summary.outputs.texture
        || identity(&product.appearance_two_da) != product.summary.outputs.appearance_two_da
        || identity(&product.hak) != product.summary.outputs.hak
        || identity(&product.report_json) != product.summary.outputs.report
    {
        return Err(pipeline_error(
            "output",
            "M6-DEMO-PACKET-PRODUCT-DIFF",
            "product",
            "product payloads, reports and identities must match the immutable product summary",
        ));
    }
    if module_identity.hak_resref != product.report.identity.hak_resref
        || demo.report.module_resref != module_identity.module_resref
        || demo.report.area_resref != module_identity.area_resref
        || demo.report.hak_resref != module_identity.hak_resref
        || demo.report.creature_resref != creature_resref
        || demo.report.appearance_row != product.report.appearance.appended_row_index
        || identity(&demo.payload).sha256 != demo.report.sha256
        || identity(&demo.payload).byte_length != demo.report.byte_length
    {
        return Err(pipeline_error(
            "output",
            "M6-DEMO-PACKET-MODULE-DIFF",
            "demo",
            "demo payload, module identity, HAK, creature and appearance row must match the immutable product",
        ));
    }

    let archive = ErfArchive::parse(&product.hak).map_err(|error| {
        pipeline_error(
            "output",
            error.code,
            format!("product.hak@{}", error.offset),
            error.context,
        )
    })?;
    for (resref, resource_type, expected) in [
        (
            product.report.identity.model_resref.as_str(),
            2002,
            product.model.as_slice(),
        ),
        (
            product.report.identity.texture_resref.as_str(),
            3,
            product.texture.as_slice(),
        ),
        ("appearance", 2017, product.appearance_two_da.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            pipeline_error(
                "output",
                error.code,
                format!("product.hak.{resref}"),
                error.context,
            )
        })?;
        if actual != expected {
            return Err(pipeline_error(
                "output",
                "M6-DEMO-PACKET-HAK-RESOURCE-DIFF",
                format!("product.hak.{resref}"),
                "the HAK resource differs from the independently held product payload",
            ));
        }
    }

    let module =
        inspect_binary_creature_profile_matrix_module_v2(&demo.payload).map_err(|error| {
            pipeline_error(
                "output",
                error.code,
                format!("demo.{}", error.path),
                error.message,
            )
        })?;
    if module.scene.module_resref != module_identity.module_resref
        || module.scene.area_resref != module_identity.area_resref
        || module.scene.ordered_hak_resrefs != [module_identity.hak_resref.clone()]
        || module.fixtures.len() != 1
        || module.fixtures[0].runtime_profile
            != BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
        || module.scene.fixtures.len() != 1
        || module.scene.fixtures[0].template_resref != creature_resref
        || module.scene.fixtures[0].appearance_row != product.report.appearance.appended_row_index
    {
        return Err(pipeline_error(
            "output",
            "M6-DEMO-PACKET-MODULE-READBACK-DIFF",
            "demo",
            "demo semantic readback differs from the exact module, Area, HAK, creature or appearance identity",
        ));
    }

    let demo_report_json = json_bytes(&demo.report, "demoReport")?;
    let generated_payloads = [
        ("generated/source.glb".to_owned(), source_glb),
        (
            format!("generated/{}.mdl", product.report.identity.model_resref),
            product.model.as_slice(),
        ),
        (
            format!("generated/{}.tga", product.report.identity.texture_resref),
            product.texture.as_slice(),
        ),
        (
            "generated/appearance.2da".to_owned(),
            product.appearance_two_da.as_slice(),
        ),
        (
            format!("generated/{}.hak", product.report.identity.hak_resref),
            product.hak.as_slice(),
        ),
        (
            format!("generated/{}.mod", module_identity.module_resref),
            demo.payload.as_slice(),
        ),
        (
            "reports/materialization-report.json".to_owned(),
            product.report_json.as_slice(),
        ),
        (
            "reports/product-manifest.json".to_owned(),
            product.manifest_json.as_slice(),
        ),
        (
            "reports/summary.json".to_owned(),
            product.summary_json.as_slice(),
        ),
        (
            "reports/demo-report.json".to_owned(),
            demo_report_json.as_slice(),
        ),
    ];
    let generated_files = generated_payloads
        .iter()
        .map(|(relative_path, bytes)| M6GeneratedFileV1 {
            relative_path: relative_path.clone(),
            byte_length: bytes.len() as u64,
            sha256: hex_sha256(bytes),
        })
        .collect::<Vec<_>>();
    let packet_manifest = ProceduralCreatureProductDemoPacketManifestV2 {
        schema_version: 2,
        status: "PROCEDURAL_CREATURE_PRODUCT_DEMO_PACKET_MATERIALIZED".to_owned(),
        input_glb: source_identity,
        input_appearance_two_da: product.summary.input_appearance_two_da.clone(),
        product_identity: product.report.identity.clone(),
        module_identity: module_identity.clone(),
        creature_resref: creature_resref.to_owned(),
        appearance_row: product.report.appearance.appended_row_index,
        geometry: product.report.geometry.clone(),
        skin_accessory_stabilization: product.report.skin_accessory_stabilization.clone(),
        generated_files,
        package_manifest: product.package_manifest.clone(),
        demo: demo.report.clone(),
        manifest_self_hash_policy: "EXCLUDED_TO_AVOID_SELF_REFERENCE".to_owned(),
    };
    let packet_manifest_json = json_bytes(&packet_manifest, "packetManifest")?;

    let parent = output_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| io_error("output", "M6-OUTPUT-CREATE-FAILED", parent, error))?;
    let name = output_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("procedural-product-demo");
    let staging = parent.join(format!(".{name}.m2a-stage-{}", std::process::id()));
    if staging.exists() {
        return Err(pipeline_error(
            "output",
            "M6-STAGING-EXISTS",
            logical_path(&staging),
            "pre-existing staging directory is never deleted",
        ));
    }
    let write_result = (|| {
        for directory in [staging.join("generated"), staging.join("reports")] {
            fs::create_dir_all(&directory).map_err(|error| {
                io_error("output", "M6-OUTPUT-CREATE-FAILED", &directory, error)
            })?;
        }
        fs::create_dir_all(staging.join("live")).map_err(|error| {
            io_error(
                "output",
                "M6-OUTPUT-CREATE-FAILED",
                &staging.join("live"),
                error,
            )
        })?;
        for (relative_path, bytes) in &generated_payloads {
            let path = staging.join(relative_path);
            fs::write(&path, bytes)
                .map_err(|error| io_error("output", "M6-OUTPUT-WRITE-FAILED", &path, error))?;
        }
        let manifest_path = staging.join("reports").join(M6_MANIFEST_FILE_NAME);
        fs::write(&manifest_path, &packet_manifest_json)
            .map_err(|error| io_error("output", "M6-OUTPUT-WRITE-FAILED", &manifest_path, error))?;
        Ok::<(), M6PipelineErrorV1>(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    fs::rename(&staging, output_dir).map_err(|error| {
        let _ = fs::remove_dir_all(&staging);
        io_error("output", "M6-OUTPUT-RENAME-FAILED", output_dir, error)
    })?;
    Ok(())
}

/// Writes the independent static Meshy M0 proof packet without reusing any
/// H1/M6 resource name.  The on-disk layout is deliberately the same audited
/// packet contract as M6 (`generated`, `reports`, and an empty `live` folder),
/// but the artifact names remain isolated so a runtime install cannot replace
/// the H1 proof asset.
pub fn write_m0_proof_packet_v1(
    output_dir: &Path,
    artifact: &M6ModelPackageArtifactV1,
) -> Result<(), M6PipelineErrorV1> {
    require_legacy_m0_offline_artifact(artifact)?;
    write_proof_packet_v1(
        output_dir,
        artifact,
        M0_MODEL_RESREF,
        M0_TEXTURE_RESREF,
        M0_HAK_FILE_NAME,
        M0_PROOF_MODULE_FILE_NAME,
    )
}

/// Writes the recovered M0 packet using the one canonical proof MOD and HAK
/// filenames.  Callers must install only this packet for the live M0 lane.
pub fn write_m0_canonical_proof_packet_v1(
    _output_dir: &Path,
    _artifact: &M6ModelPackageArtifactV1,
) -> Result<(), M6PipelineErrorV1> {
    Err(legacy_m0_runtime_admission_forbidden())
}

fn require_legacy_m0_offline_artifact(
    artifact: &M6ModelPackageArtifactV1,
) -> Result<(), M6PipelineErrorV1> {
    if artifact.summary.m0_runtime_fixture_contract.is_some()
        || artifact.report.m0_runtime_fixture_contract.is_some()
        || artifact.manifest.m0_runtime_fixture_contract.is_some()
    {
        return Err(pipeline_error(
            "output",
            "M0-LEGACY-WRITER-RUNTIME-ADMISSION-FORBIDDEN",
            "artifact.m0RuntimeFixtureContract",
            "V1 packet writers are offline-only and cannot write a V2 runtime-admissible artifact",
        ));
    }
    Ok(())
}

/// Writes a verified canonical M0 runtime package using the caller-owned
/// fresh module and HAK filenames bound into its binary scene. The destination
/// remains absent-only through the shared packet writer.
pub fn write_m0_canonical_runtime_proof_packet_with_profile_v2(
    output_dir: &Path,
    artifact: &M6ModelPackageArtifactV1,
    identity: &BinaryM0VerticalSliceIdentityV1,
    expected_runtime_profile: &DirectCreatureRuntimeProfileV2,
    source_glb: &[u8],
) -> Result<(), M6PipelineErrorV1> {
    let contract = artifact
        .summary
        .m0_runtime_fixture_contract
        .as_ref()
        .ok_or_else(|| {
            pipeline_error(
                "output",
                "M0-RUNTIME-CONTRACT-MISSING",
                "artifact.summary.m0RuntimeFixtureContract",
                "fresh runtime packet requires the production M0 runtime contract",
            )
        })?;
    if contract.binary_scene.module_resref != identity.module_resref
        || contract.binary_scene.area_resref != identity.area_resref
        || contract.binary_scene.ordered_hak_resrefs != [identity.hak_resref.clone()]
    {
        return Err(pipeline_error(
            "output",
            "M0-RUNTIME-IDENTITY-MISMATCH",
            "identity",
            "caller-owned filenames must match the verified MOD/Area/ordered-HAK contract",
        ));
    }
    if artifact.source_glb != source_glb {
        return Err(pipeline_error(
            "output",
            "M0-RUNTIME-SOURCE-MISMATCH",
            "sourceGlb",
            "runtime writer requires the exact source bytes supplied independently from the artifact contract",
        ));
    }
    verify_m0_binary_runtime_fixture_contract_v2(
        contract,
        expected_runtime_profile,
        source_glb,
        &artifact.proof_module,
        &artifact.hak,
    )?;
    write_proof_packet_v1(
        output_dir,
        artifact,
        M0_MODEL_RESREF,
        M0_TEXTURE_RESREF,
        &format!("{}.hak", identity.hak_resref),
        &format!("{}.mod", identity.module_resref),
    )
}

fn write_proof_packet_v1(
    output_dir: &Path,
    artifact: &M6ModelPackageArtifactV1,
    model_resref: &str,
    texture_resref: &str,
    hak_file_name: &str,
    proof_module_file_name: &str,
) -> Result<(), M6PipelineErrorV1> {
    if output_dir.exists() {
        return Err(pipeline_error(
            "output",
            "M6-OUTPUT-EXISTS",
            "outputDir",
            "output directory must not already exist",
        ));
    }
    let parent = output_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| io_error("output", "M6-OUTPUT-CREATE-FAILED", parent, error))?;
    let name = output_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("m6-proof");
    let staging = parent.join(format!(".{name}.m2a-stage-{}", std::process::id()));
    if staging.exists() {
        return Err(pipeline_error(
            "output",
            "M6-STAGING-EXISTS",
            logical_path(&staging),
            "pre-existing staging directory is never deleted",
        ));
    }
    let write_result = write_staging_packet(
        &staging,
        artifact,
        model_resref,
        texture_resref,
        hak_file_name,
        proof_module_file_name,
    );
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    fs::rename(&staging, output_dir).map_err(|error| {
        let _ = fs::remove_dir_all(&staging);
        io_error("output", "M6-OUTPUT-RENAME-FAILED", output_dir, error)
    })?;
    Ok(())
}

fn write_staging_packet(
    staging: &Path,
    artifact: &M6ModelPackageArtifactV1,
    model_resref: &str,
    texture_resref: &str,
    hak_file_name: &str,
    proof_module_file_name: &str,
) -> Result<(), M6PipelineErrorV1> {
    let generated = staging.join("generated");
    let reports = staging.join("reports");
    for path in [&generated, &reports, &staging.join("live")] {
        fs::create_dir_all(path)
            .map_err(|error| io_error("output", "M6-OUTPUT-CREATE-FAILED", path, error))?;
    }
    for (path, bytes) in [
        (generated.join("source.glb"), artifact.source_glb.as_slice()),
        (
            generated.join(format!("{model_resref}.mdl")),
            artifact.model.as_slice(),
        ),
        (
            generated.join(format!("{texture_resref}.tga")),
            artifact.texture.as_slice(),
        ),
        (
            generated.join("appearance.2da"),
            artifact.appearance_two_da.as_slice(),
        ),
        (generated.join(hak_file_name), artifact.hak.as_slice()),
        (
            generated.join(proof_module_file_name),
            artifact.proof_module.as_slice(),
        ),
        (
            reports.join("materialization-report.json"),
            artifact.report_json.as_slice(),
        ),
        (
            reports.join("summary.json"),
            artifact.summary_json.as_slice(),
        ),
    ] {
        fs::write(&path, bytes)
            .map_err(|error| io_error("output", "M6-OUTPUT-WRITE-FAILED", &path, error))?;
    }
    let manifest_path = reports.join(M6_MANIFEST_FILE_NAME);
    fs::write(&manifest_path, &artifact.manifest_json)
        .map_err(|error| io_error("output", "M6-OUTPUT-WRITE-FAILED", &manifest_path, error))?;
    Ok(())
}

fn identity(bytes: &[u8]) -> M6ByteIdentityV1 {
    M6ByteIdentityV1 {
        byte_length: bytes.len() as u64,
        sha256: hex_sha256(bytes),
    }
}

fn held_weapon_texture_resref_v1(base_texture_resref: &str) -> Result<String, M6PipelineErrorV1> {
    let prefix = base_texture_resref.chars().take(13).collect::<String>();
    let resref = format!("{prefix}_w1");
    if resref.len() > 16
        || resref == base_texture_resref
        || !resref
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(pipeline_error(
            "held_weapon",
            "M2A-HELD-WEAPON-TEXTURE-RESREF",
            "productIdentity.textureResref",
            "could not derive a distinct <=16 character weapon texture resref",
        ));
    }
    Ok(resref)
}

fn node_tree_contains_name_v1(node: &NodeReport, expected_name: &str) -> bool {
    node.name == expected_name
        || node
            .children
            .iter()
            .any(|child| node_tree_contains_name_v1(child, expected_name))
}

fn json_bytes<T: Serialize>(value: &T, path: &str) -> Result<Vec<u8>, M6PipelineErrorV1> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        pipeline_error(
            "report",
            "M6-JSON-SERIALIZE-FAILED",
            path,
            error.to_string(),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn map_erf_readback(error: crate::erf::ErfError) -> M6PipelineErrorV1 {
    pipeline_error(
        "readback",
        error.code,
        format!("hak@{}", error.offset),
        error.context,
    )
}

fn pipeline_error(
    stage: &str,
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> M6PipelineErrorV1 {
    M6PipelineErrorV1 {
        schema_version: 1,
        stage: stage.to_ascii_uppercase(),
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

fn io_error(stage: &str, code: &str, path: &Path, error: std::io::Error) -> M6PipelineErrorV1 {
    pipeline_error(stage, code, logical_path(path), error.to_string())
}

fn logical_path(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("output")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        M6_REQUIRED_DIRECT_CREATURE_CLIPS, M6BuildOutputV2, M6GeometryPolicyV1,
        TriangleDegeneracyPolicyV1, apply_authored_h1_source_names_v1,
        materialize_direct_creature_runtime_clips, p100k_experiment_glb_limits_v1,
        p100k_experiment_triangle_count_is_eligible_v1, p300k_experiment_glb_limits_v1,
        sanitize_meshy_h1_degenerate_triangles_exact_v1, sanitize_meshy_h1_degenerate_triangles_v1,
        sanitize_runtime_creature_face_planes_exact_v1, sanitize_runtime_creature_face_planes_v1,
        split_creature_segments_for_binary_mdl_v1, triangle_degeneracy_policy_for_build_v1,
    };
    use crate::{
        glb::{GlbLimits, ingest_glb},
        mdl::{MdlAnimationClipV1, MdlAnimationSetV1, NWN_EE_MAX_MESH_INDEX_COUNT_V1},
        owned_fixture::synthetic_owned_m6_glb_v1,
        profile_a::{
            AuroraCreatureIrV1, AuroraCreatureSegmentV1, AuroraVertexWeightsV1,
            RigSegmentDeformationV1, derive_meshy_h1_profile_and_mapping_p300k_experiment_v1,
            derive_meshy_h1_profile_and_mapping_v1,
        },
    };

    fn clip(name: &str, length_seconds: f32) -> MdlAnimationClipV1 {
        MdlAnimationClipV1 {
            name: name.to_owned(),
            animation_root: "owned_root".to_owned(),
            length_seconds,
            transition_seconds: 0.25,
            events: Vec::new(),
            tracks: Vec::new(),
        }
    }

    #[test]
    fn direct_creature_runtime_profile_aliases_the_owned_idle_clip() {
        let source = MdlAnimationSetV1 {
            schema_version: 1,
            clips: vec![clip("cpause1", 4.0)],
        };

        let output = materialize_direct_creature_runtime_clips(&source).unwrap();

        assert_eq!(output.clips.len(), M6_REQUIRED_DIRECT_CREATURE_CLIPS.len());
        for name in M6_REQUIRED_DIRECT_CREATURE_CLIPS {
            let generated = output.clips.iter().find(|clip| clip.name == name).unwrap();
            assert_eq!(generated.animation_root, "owned_root");
            assert_eq!(generated.length_seconds, 4.0);
        }
    }

    #[test]
    fn direct_creature_runtime_profile_preserves_a_source_specific_state() {
        let source = MdlAnimationSetV1 {
            schema_version: 1,
            clips: vec![clip("cpause1", 4.0), clip("cwalk", 1.5)],
        };

        let output = materialize_direct_creature_runtime_clips(&source).unwrap();

        assert_eq!(
            output
                .clips
                .iter()
                .find(|clip| clip.name == "cwalk")
                .unwrap()
                .length_seconds,
            1.5
        );
        assert_eq!(output.clips.len(), M6_REQUIRED_DIRECT_CREATURE_CLIPS.len());
    }

    #[test]
    fn authored_procedural_route_preserves_the_derived_cpause1_for_a_meshy_name() {
        let glb = synthetic_owned_m6_glb_v1().expect("owned Meshy-style fixture");
        let source = ingest_glb(&glb, &GlbLimits::default()).expect("GLB ingest");
        let (_, mut procedural_mapping) =
            derive_meshy_h1_profile_and_mapping_v1(&source).expect("derived mapping");
        let (_, mut explicit_mapping) =
            derive_meshy_h1_profile_and_mapping_v1(&source).expect("derived mapping");

        apply_authored_h1_source_names_v1(&source, &mut procedural_mapping, true);
        apply_authored_h1_source_names_v1(&source, &mut explicit_mapping, false);

        assert_eq!(
            procedural_mapping.clip_mappings[0].output_clip_name,
            "cpause1"
        );
        assert_eq!(
            explicit_mapping.clip_mappings[0].output_clip_name,
            "owned-linear-pause"
        );
    }

    #[test]
    fn active_creature_routes_use_exact_finite_non_collinear_geometry_policy() {
        assert_eq!(
            triangle_degeneracy_policy_for_build_v1(
                M6BuildOutputV2::ProceduralProduct,
                M6GeometryPolicyV1::ProductBudget,
            ),
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear
        );
        assert_eq!(
            triangle_degeneracy_policy_for_build_v1(
                M6BuildOutputV2::LegacyBundle,
                M6GeometryPolicyV1::P100kSegmentedExperiment,
            ),
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear
        );
        assert_eq!(
            triangle_degeneracy_policy_for_build_v1(
                M6BuildOutputV2::LegacyBundle,
                M6GeometryPolicyV1::P300kSegmentedExperiment,
            ),
            TriangleDegeneracyPolicyV1::ExactFiniteNonCollinear
        );
    }

    #[test]
    fn frozen_legacy_product_bundle_keeps_legacy_geometry_policy() {
        assert_eq!(
            triangle_degeneracy_policy_for_build_v1(
                M6BuildOutputV2::LegacyBundle,
                M6GeometryPolicyV1::ProductBudget,
            ),
            TriangleDegeneracyPolicyV1::LegacyAbsoluteEpsilon
        );
    }

    #[test]
    fn p100k_target_class_accepts_only_its_bounded_meshy_overshoot_envelope() {
        assert!(p100k_experiment_triangle_count_is_eligible_v1(20_001));
        assert!(p100k_experiment_triangle_count_is_eligible_v1(100_000));
        assert!(p100k_experiment_triangle_count_is_eligible_v1(103_290));
        assert!(p100k_experiment_triangle_count_is_eligible_v1(110_000));
        assert!(!p100k_experiment_triangle_count_is_eligible_v1(20_000));
        assert!(!p100k_experiment_triangle_count_is_eligible_v1(110_001));
    }

    #[test]
    #[ignore = "requires the local Git-ignored canonical Meshy Creature GLB corpus"]
    fn canonical_humanoid_creature_corpus_preserves_every_exact_finite_non_collinear_face() {
        struct CorpusCase {
            asset_id: &'static str,
            payload: &'static str,
            expected_raw_triangles: usize,
            limits: GlbLimits,
        }

        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let cases = [
            CorpusCase {
                asset_id: "h1-humanoid-1500",
                payload: "source.glb",
                expected_raw_triangles: 1_556,
                limits: GlbLimits::default(),
            },
            CorpusCase {
                asset_id: "h2-clockwork-sentinel-1500",
                payload: "source.glb",
                expected_raw_triangles: 1_543,
                limits: GlbLimits::default(),
            },
            CorpusCase {
                asset_id: "tlc-powrotnik-h1-p20k-v1",
                payload: "source-budget20000.glb",
                expected_raw_triangles: 19_892,
                limits: GlbLimits::default(),
            },
            CorpusCase {
                asset_id: "tlc-powrotnik-h1-p100k-limit-v1",
                payload: "source.glb",
                expected_raw_triangles: 103_290,
                limits: p100k_experiment_glb_limits_v1(),
            },
            CorpusCase {
                asset_id: "tlc-veiled-humanoid-h1-p100k-v1",
                payload: "source.glb",
                expected_raw_triangles: 102_335,
                limits: p100k_experiment_glb_limits_v1(),
            },
            CorpusCase {
                asset_id: "tlc-stoneback-brute-h1-p300k-v1",
                payload: "source.glb",
                expected_raw_triangles: 296_276,
                limits: p300k_experiment_glb_limits_v1(),
            },
        ];

        for case in cases {
            let asset_root = repo.join("sample-3d").join(case.asset_id);
            assert!(
                asset_root.join("manifest.yaml").is_file(),
                "{} is missing its canonical manifest",
                case.asset_id
            );
            let source_path = asset_root.join(case.payload);
            let bytes = fs::read(&source_path).unwrap_or_else(|error| {
                panic!(
                    "cannot read canonical Creature payload {}: {error}",
                    source_path.display()
                )
            });
            let raw = ingest_glb(&bytes, &case.limits).unwrap_or_else(|error| {
                panic!(
                    "cannot ingest canonical Creature payload {}: {}",
                    source_path.display(),
                    error.message
                )
            });
            assert_eq!(
                raw.report.statistics.triangle_count, case.expected_raw_triangles,
                "{} raw triangle count drifted",
                case.asset_id
            );

            let mut exact = raw.clone();
            sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut exact).unwrap_or_else(|error| {
                panic!("{} exact sanitation failed: {error}", case.asset_id)
            });
            let mut legacy = raw;
            sanitize_meshy_h1_degenerate_triangles_v1(&mut legacy).unwrap_or_else(|error| {
                panic!(
                    "{} legacy comparison sanitation failed: {error}",
                    case.asset_id
                )
            });

            let exact_triangles = exact.report.statistics.triangle_count;
            let legacy_triangles = legacy.report.statistics.triangle_count;
            assert_eq!(
                exact_triangles, case.expected_raw_triangles,
                "{} contains a non-finite, zero-area or exactly collinear source face",
                case.asset_id
            );
            assert!(
                exact_triangles >= legacy_triangles,
                "{} exact sanitation cannot retain fewer faces than the legacy epsilon",
                case.asset_id
            );
            eprintln!(
                "CREATURE_EXACT_CORPUS asset={} raw={} exact={} legacy={} recovered_valid_microtriangles={}",
                case.asset_id,
                case.expected_raw_triangles,
                exact_triangles,
                legacy_triangles,
                exact_triangles - legacy_triangles
            );
        }
    }

    #[test]
    fn p100k_experiment_partitions_skin_geometry_without_losing_triangles() {
        let triangle_count = NWN_EE_MAX_MESH_INDEX_COUNT_V1 / 3 + 1;
        let mut positions = Vec::with_capacity(triangle_count * 3);
        let mut indices = Vec::with_capacity(triangle_count * 3);
        for triangle in 0..triangle_count {
            let base = positions.len() as u32;
            let x = triangle as f32 * 2.0;
            positions.extend([[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]]);
            indices.extend([base, base + 1, base + 2]);
        }
        let vertex_count = positions.len();
        let mut creature = AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "p100k-segmentation-test".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 7,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Skin,
                parent_node_id: 1,
                cast_shadow: true,
                positions,
                normals: vec![[0.0, 0.0, 1.0]; vertex_count],
                tangents: None,
                uv0: vec![[0.0, 0.0]; vertex_count],
                indices,
                face_surface_ids: Vec::new(),
                weights: vec![
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(1), None, None, None],
                        values: [1.0, 0.0, 0.0, 0.0],
                        influence_count: 1,
                    };
                    vertex_count
                ],
            }],
        };

        split_creature_segments_for_binary_mdl_v1(&mut creature).unwrap();

        assert_eq!(creature.segments.len(), 2);
        assert_eq!(
            creature
                .segments
                .iter()
                .map(|segment| segment.indices.len() / 3)
                .sum::<usize>(),
            triangle_count
        );
        assert!(creature.segments.iter().all(|segment| {
            segment.indices.len() <= NWN_EE_MAX_MESH_INDEX_COUNT_V1
                && segment.positions.len() <= usize::from(u16::MAX)
                && segment.weights.len() == segment.positions.len()
        }));
        assert_eq!(creature.segments[0].positions.len(), usize::from(u16::MAX));
        assert_eq!(creature.segments[1].positions.len(), 3);
    }

    #[test]
    fn p300k_experiment_owns_an_explicit_raw_envelope_above_the_exact_gate() {
        let limits = p300k_experiment_glb_limits_v1();

        assert_eq!(limits.triangle_warning_above, 150_000);
        assert_eq!(limits.triangle_blocking_above, 330_000);
        assert!(limits.max_input_bytes >= 256 * 1024 * 1024);
        assert!(limits.max_decoded_geometry_bytes >= 512 * 1024 * 1024);
        assert!(limits.max_decoded_skin_animation_bytes >= 512 * 1024 * 1024);
    }

    #[test]
    fn segmented_experiment_partitions_exactly_300k_triangles_without_loss() {
        let triangle_count = 300_000usize;
        let mut positions = Vec::with_capacity(triangle_count + 2);
        positions.push([0.0, 0.0, 0.0]);
        for vertex in 0..=triangle_count {
            positions.push([vertex as f32, 1.0, 0.0]);
        }
        let mut indices = Vec::with_capacity(triangle_count * 3);
        for triangle in 0..triangle_count {
            indices.extend([0, triangle as u32 + 1, triangle as u32 + 2]);
        }
        let vertex_count = positions.len();
        let mut creature = AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "p300k-segmentation-test".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Skin,
                parent_node_id: 1,
                cast_shadow: true,
                positions,
                normals: vec![[0.0, 0.0, 1.0]; vertex_count],
                tangents: None,
                uv0: vec![[0.0, 0.0]; vertex_count],
                indices,
                face_surface_ids: Vec::new(),
                weights: vec![
                    AuroraVertexWeightsV1 {
                        bone_node_ids: [Some(1), None, None, None],
                        values: [1.0, 0.0, 0.0, 0.0],
                        influence_count: 1,
                    };
                    vertex_count
                ],
            }],
        };

        split_creature_segments_for_binary_mdl_v1(&mut creature).unwrap();

        assert_eq!(
            creature
                .segments
                .iter()
                .map(|segment| segment.indices.len() / 3)
                .sum::<usize>(),
            triangle_count
        );
        assert_eq!(creature.segments.len(), 14);
        assert!(creature.segments.iter().all(|segment| {
            segment.indices.len() <= NWN_EE_MAX_MESH_INDEX_COUNT_V1
                && segment.positions.len() <= usize::from(u16::MAX)
                && segment.weights.len() == segment.positions.len()
        }));
    }

    #[test]
    fn runtime_sanitation_uses_the_binary_writer_face_plane_threshold() {
        let weights = AuroraVertexWeightsV1 {
            bone_node_ids: [Some(1), None, None, None],
            values: [1.0, 0.0, 0.0, 0.0],
            influence_count: 1,
        };
        let mut creature = AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "runtime-face-plane-test".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Skin,
                parent_node_id: 1,
                cast_shadow: true,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [2.0, 0.0, 0.0],
                    [2.01, 0.0, 0.0],
                    [2.0, 0.001, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 6],
                tangents: Some(vec![[1.0, 0.0, 0.0, 1.0]; 6]),
                uv0: vec![[0.0, 0.0]; 6],
                indices: vec![0, 1, 2, 3, 4, 5],
                face_surface_ids: vec![11, 12],
                weights: vec![weights; 6],
            }],
        };

        let removed = sanitize_runtime_creature_face_planes_v1(&mut creature).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(creature.segments.len(), 1);
        assert_eq!(creature.segments[0].indices, vec![0, 1, 2]);
        assert_eq!(creature.segments[0].positions.len(), 3);
        assert_eq!(creature.segments[0].normals.len(), 3);
        assert_eq!(creature.segments[0].tangents.as_ref().unwrap().len(), 3);
        assert_eq!(creature.segments[0].uv0.len(), 3);
        assert_eq!(creature.segments[0].weights.len(), 3);
        assert_eq!(creature.segments[0].face_surface_ids, vec![11]);
    }

    #[test]
    fn exact_runtime_sanitation_preserves_micro_triangles_and_removes_only_collinear_faces() {
        let weights = AuroraVertexWeightsV1 {
            bone_node_ids: [Some(1), None, None, None],
            values: [1.0, 0.0, 0.0, 0.0],
            influence_count: 1,
        };
        let mut creature = AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "runtime-exact-face-plane-test".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "TEST".to_owned(),
            engine_facing_proof: "TEST".to_owned(),
            uv_runtime_proof: "TEST".to_owned(),
            nodes: Vec::new(),
            material_source_bindings: Vec::new(),
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Skin,
                parent_node_id: 1,
                cast_shadow: true,
                positions: vec![
                    [2.0, 0.0, 0.0],
                    [2.01, 0.0, 0.0],
                    [2.0, 0.001, 0.0],
                    [3.0, 0.0, 0.0],
                    [3.01, 0.0, 0.0],
                    [3.02, 0.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 6],
                tangents: Some(vec![[1.0, 0.0, 0.0, 1.0]; 6]),
                uv0: vec![[0.0, 0.0]; 6],
                indices: vec![0, 1, 2, 3, 4, 5],
                face_surface_ids: vec![11, 12],
                weights: vec![weights; 6],
            }],
        };

        let removed = sanitize_runtime_creature_face_planes_exact_v1(&mut creature).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(creature.segments.len(), 1);
        assert_eq!(creature.segments[0].indices, vec![0, 1, 2]);
        assert_eq!(creature.segments[0].positions.len(), 3);
        assert_eq!(creature.segments[0].face_surface_ids, vec![11]);
    }

    #[test]
    fn meshy_source_sanitation_uses_the_binary_writer_face_plane_threshold() {
        let source_glb = synthetic_owned_m6_glb_v1().unwrap();
        let mut source = ingest_glb(&source_glb, &GlbLimits::default()).unwrap();
        let primitive = &mut source.ir.primitives[0];
        let original_triangle_count = primitive.indices.len() / 3;
        let base = primitive.positions.len() as u32;
        primitive
            .positions
            .extend([[2.0, 0.0, 0.0], [2.01, 0.0, 0.0], [2.0, 0.001, 0.0]]);
        primitive.normals.extend([[0.0, 0.0, 1.0]; 3]);
        if !primitive.tangents.is_empty() {
            primitive.tangents.extend([[1.0, 0.0, 0.0, 1.0]; 3]);
        }
        primitive.uv0.extend([[0.0, 0.0]; 3]);
        primitive.joints0.extend([[0, 0, 0, 0]; 3]);
        primitive.weights0.extend([[1.0, 0.0, 0.0, 0.0]; 3]);
        primitive.indices.extend([base, base + 1, base + 2]);
        source.report.statistics.vertex_count += 3;
        source.report.statistics.index_count += 3;
        source.report.statistics.triangle_count += 1;

        sanitize_meshy_h1_degenerate_triangles_v1(&mut source).unwrap();

        assert_eq!(
            source.ir.primitives[0].indices.len() / 3,
            original_triangle_count
        );
        assert_eq!(
            source.report.statistics.triangle_count,
            original_triangle_count
        );
    }

    #[test]
    fn exact_source_sanitation_preserves_micro_triangles_and_removes_only_collinear_faces() {
        let source_glb = synthetic_owned_m6_glb_v1().unwrap();
        let mut source = ingest_glb(&source_glb, &GlbLimits::default()).unwrap();
        let primitive = &mut source.ir.primitives[0];
        let original_triangle_count = primitive.indices.len() / 3;
        let base = primitive.positions.len() as u32;
        primitive.positions.extend([
            [2.0, 0.0, 0.0],
            [2.01, 0.0, 0.0],
            [2.0, 0.001, 0.0],
            [3.0, 0.0, 0.0],
            [3.01, 0.0, 0.0],
            [3.02, 0.0, 0.0],
        ]);
        primitive.normals.extend([[0.0, 0.0, 1.0]; 6]);
        if !primitive.tangents.is_empty() {
            primitive.tangents.extend([[1.0, 0.0, 0.0, 1.0]; 6]);
        }
        primitive.uv0.extend([[0.0, 0.0]; 6]);
        primitive.joints0.extend([[0, 0, 0, 0]; 6]);
        primitive.weights0.extend([[1.0, 0.0, 0.0, 0.0]; 6]);
        primitive
            .indices
            .extend([base, base + 1, base + 2, base + 3, base + 4, base + 5]);
        source.report.statistics.vertex_count += 6;
        source.report.statistics.index_count += 6;
        source.report.statistics.triangle_count += 2;

        sanitize_meshy_h1_degenerate_triangles_exact_v1(&mut source).unwrap();

        assert_eq!(
            source.ir.primitives[0].indices.len() / 3,
            original_triangle_count + 1
        );
        assert_eq!(
            source.report.statistics.triangle_count,
            original_triangle_count + 1
        );
        assert!(source.report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "M4A-MESHY-H1-EXACT-DEGENERATE-TRIANGLES-REMOVED"
                && diagnostic.message.contains("removed 1")
        }));
        let (rig, _) = derive_meshy_h1_profile_and_mapping_p300k_experiment_v1(&source).unwrap();
        assert_eq!(
            rig.segments[0].surface_indices, source.ir.primitives[0].indices,
            "the exact P300K rig surface must preserve every accepted source face"
        );
    }

    #[test]
    fn meshy_sanitation_compacts_every_vertex_attribute_after_triangle_filtering() {
        let source_glb = synthetic_owned_m6_glb_v1().unwrap();
        let mut source = ingest_glb(&source_glb, &GlbLimits::default()).unwrap();
        let primitive = &mut source.ir.primitives[0];
        let original_vertex_count = primitive.positions.len();
        primitive.positions.push([100.0, 100.0, 100.0]);
        primitive.normals.push([0.0, 0.0, 1.0]);
        if !primitive.tangents.is_empty() {
            primitive.tangents.push([1.0, 0.0, 0.0, 1.0]);
        }
        primitive.uv0.push([0.5, 0.5]);
        primitive.joints0.push([0, 0, 0, 0]);
        primitive.weights0.push([1.0, 0.0, 0.0, 0.0]);
        source.report.statistics.vertex_count += 1;

        sanitize_meshy_h1_degenerate_triangles_v1(&mut source).unwrap();

        let primitive = &source.ir.primitives[0];
        assert_eq!(primitive.positions.len(), original_vertex_count);
        assert_eq!(primitive.normals.len(), original_vertex_count);
        assert_eq!(primitive.uv0.len(), original_vertex_count);
        assert_eq!(primitive.joints0.len(), original_vertex_count);
        assert_eq!(primitive.weights0.len(), original_vertex_count);
        assert_eq!(source.report.statistics.vertex_count, original_vertex_count);
        assert!(
            source.report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "M4A-MESHY-H1-UNREFERENCED-VERTICES-REMOVED"
            })
        );
    }
}
