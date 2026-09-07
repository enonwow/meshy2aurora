use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    c_wolf_rig::{
        build_c_wolf_compatible_rig_v1, build_c_wolf_compatible_rig_v2,
        build_c_wolf_compatible_rig_v3, build_c_wolf_compatible_rig_v4,
        build_c_wolf_compatible_rig_v5, build_c_wolf_motion_contract_exact_v3,
        c_wolf_compatibility_topology_v1,
    },
    creature_product::{CreatureAlphaModeV2, CreatureMaterialProfileV2, CreatureMaterialTargetV2},
    erf::ErfArchive,
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakResourceInputV1, HakWriterOptionsV1},
    mdl::{
        InspectionReport, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
        MdlStateProjectionProfileV1, MdlWriterOptionsV1, NodeReport,
        evaluate_reference_supermodel_skin_deformation_v1, inspect_binary_mdl,
    },
    package::write_model_package_v1,
    profile_a::CreatureSourceForwardV1,
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1,
        BinaryCreatureProfiledFixtureV2, BinaryCreatureRuntimeProfileV2, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_profile_matrix_module_named_v3,
    },
    reference_proof::{
        CapabilityResult, CapabilityStatus, ExecutionMetadata, InputFingerprint, InvariantResult,
        InvariantStatus, ReferenceCapability, ReferenceIdentity, ReferenceManifest,
        ReferenceManifestEntry, ReferenceSource, build_reference_proof_packet,
    },
    reference_supermodel::{
        ReferenceSupermodelContractV1, ReferenceSupermodelNodeV1,
        ReferenceSupermodelRetargetArtifactV1, canonical_reference_supermodel_contract_sha256_v1,
        retarget_static_mesh_to_reference_supermodel_v2,
    },
    reference_supermodel_motion::{
        ReferenceSupermodelClassicMaterialOptionsV1, ReferenceSupermodelClassicMotionArtifactV1,
        ReferenceSupermodelMaterialOptionsV1, ReferenceSupermodelMinimalMtrMaterialOptionsV1,
        ReferenceSupermodelMinimalMtrMotionArtifactV1, ReferenceSupermodelMotionArtifactV2,
        bind_static_mesh_for_inherited_supermodel_motion_direct_classic_v1,
        bind_static_mesh_for_inherited_supermodel_motion_direct_v2,
        bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_corrected_v2,
        bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_retargeted_v4,
        bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_v1,
        default_reference_supermodel_writer_options_v1,
        inspect_inherited_motion_quality_with_tolerances_v2,
        package_reference_supermodel_classic_creature_hak_v1,
        package_reference_supermodel_creature_hak_v1,
        package_reference_supermodel_minimal_twosided_creature_hak_v1,
        validate_reference_supermodel_motion_oracle_v2,
    },
    tga::{TgaWriterOptionsV1, write_tga_v1},
    two_da::{
        TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1, append_two_da_row_v1,
        clone_two_da_row_request_v1, inspect_two_da_v2, read_two_da_row_v2,
        retain_two_da_row_prefix_v1,
    },
};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};

const RETAIL_C_WOLF_SHA256: &str =
    "a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726";
const RETAIL_C_WOLF_BYTE_LENGTH: usize = 340_812;

const DOG_DONOR_PHYSICAL_ROW: u32 = 176;
const AURORA_VISIBLE_APPEARANCE_ROWS: u32 = 848;
type NumberedTopologyV1 = Vec<(u32, &'static str, Option<u32>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateRigRecipe {
    V1,
    V2,
    V3,
    V4,
    V5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateModelRecipe {
    LegacyDiffuse,
    FullMtrMotion,
    ClassicDiffuseMotion,
    MinimalTwoSidedMtrMotion,
    MinimalTwoSidedMtrCorrectedMotion,
    MinimalTwoSidedMtrRetargetedMotion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateQualityRecipe {
    None,
    LegacyObserved,
    LegacyStrict,
    MotionOracle,
}

struct CandidateIdentity {
    version: u32,
    model_resref: &'static str,
    texture_resref: &'static str,
    hak_resref: &'static str,
    module_resref: &'static str,
    area_resref: &'static str,
    creature_resref: &'static str,
    module_display_name: &'static str,
    area_display_name: &'static str,
    appearance_label: &'static str,
    fixture_id: &'static str,
    purpose: &'static str,
    source_sha256: &'static str,
    source_byte_length: usize,
    source_triangle_count: usize,
    source_image_count: usize,
    source_task_id: Option<&'static str>,
    rig_recipe: CandidateRigRecipe,
    model_recipe: CandidateModelRecipe,
    quality_recipe: CandidateQualityRecipe,
}

enum CandidateModelBuild {
    Legacy(Box<ReferenceSupermodelRetargetArtifactV1>),
    MotionV2(Box<ReferenceSupermodelMotionArtifactV2>),
    ClassicMotion(Box<ReferenceSupermodelClassicMotionArtifactV1>),
    MinimalMtrMotion(Box<ReferenceSupermodelMinimalMtrMotionArtifactV1>),
}

fn candidate_identity(version: u32) -> Result<CandidateIdentity, String> {
    match version {
        1 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre1",
            texture_resref: "m2aborztex1",
            hak_resref: "m2aborzhak1",
            module_resref: "m2aborzmod1",
            area_resref: "m2aborzarea1",
            creature_resref: "m2aborzutc1",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V1",
            area_display_name: "Meshy2Aurora Borzoi Test Area V1",
            appearance_label: "M2A_BORZOI_CWOLF_V1",
            fixture_id: "owned_borzoi_c_wolf_v1",
            purpose: "BORZOI_C_WOLF_INHERITED_ANIMATION_DEMO_V1",
            source_sha256: "44b5d3c63387ae7587b4de8dc9e4a6866948cd034cd1a1e3a0c0ce69af3e6678",
            source_byte_length: 6_847_588,
            source_triangle_count: 60_780,
            source_image_count: 1,
            source_task_id: Some("01a01fbf-fdb8-77b8-b381-c6480727cd3d"),
            rig_recipe: CandidateRigRecipe::V1,
            model_recipe: CandidateModelRecipe::LegacyDiffuse,
            quality_recipe: CandidateQualityRecipe::None,
        }),
        2 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre2",
            texture_resref: "m2aborztex2",
            hak_resref: "m2aborzhak2",
            module_resref: "m2aborzmod2",
            area_resref: "m2aborzarea2",
            creature_resref: "m2aborzutc2",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V2",
            area_display_name: "Meshy2Aurora Borzoi Test Area V2",
            appearance_label: "M2A_BORZOI_CWOLF_V2",
            fixture_id: "owned_borzoi_c_wolf_v2",
            purpose: "BORZOI_C_WOLF_INHERITED_ANIMATION_DEMO_V2",
            source_sha256: "44b5d3c63387ae7587b4de8dc9e4a6866948cd034cd1a1e3a0c0ce69af3e6678",
            source_byte_length: 6_847_588,
            source_triangle_count: 60_780,
            source_image_count: 1,
            source_task_id: Some("01a01fbf-fdb8-77b8-b381-c6480727cd3d"),
            rig_recipe: CandidateRigRecipe::V2,
            model_recipe: CandidateModelRecipe::LegacyDiffuse,
            quality_recipe: CandidateQualityRecipe::None,
        }),
        3 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre3",
            texture_resref: "m2aborztex3",
            hak_resref: "m2aborzhak3",
            module_resref: "m2aborzmod3",
            area_resref: "m2aborzarea3",
            creature_resref: "m2aborzutc3",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V3",
            area_display_name: "Meshy2Aurora Borzoi Test Area V3",
            appearance_label: "M2A_BORZOI_CWOLF_V3",
            fixture_id: "owned_borzoi_c_wolf_v3",
            purpose: "OWNER_SUPPLIED_BORZOI_C_WOLF_DEMO_V3",
            source_sha256: "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
            source_byte_length: 29_889_104,
            source_triangle_count: 300_000,
            source_image_count: 3,
            source_task_id: None,
            rig_recipe: CandidateRigRecipe::V3,
            model_recipe: CandidateModelRecipe::LegacyDiffuse,
            quality_recipe: CandidateQualityRecipe::LegacyObserved,
        }),
        4 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre4",
            texture_resref: "m2aborztex4",
            hak_resref: "m2aborzhak4",
            module_resref: "m2aborzmod4",
            area_resref: "m2aborzarea4",
            creature_resref: "m2aborzutc4",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V4",
            area_display_name: "Meshy2Aurora Borzoi Test Area V4",
            appearance_label: "M2A_BORZOI_CWOLF_V4",
            fixture_id: "owned_borzoi_c_wolf_v4",
            purpose: "OWNER_AUTHORIZED_BORZOI_QUALITY_REMEDIATION_V4",
            source_sha256: "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
            source_byte_length: 29_889_104,
            source_triangle_count: 300_000,
            source_image_count: 3,
            source_task_id: None,
            rig_recipe: CandidateRigRecipe::V4,
            model_recipe: CandidateModelRecipe::LegacyDiffuse,
            quality_recipe: CandidateQualityRecipe::LegacyStrict,
        }),
        5 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre5",
            texture_resref: "m2aborztex5",
            hak_resref: "m2aborzhak5",
            module_resref: "m2aborzmod5",
            area_resref: "m2aborzarea5",
            creature_resref: "m2aborzutc5",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V5",
            area_display_name: "Meshy2Aurora Borzoi Test Area V5",
            appearance_label: "M2A_BORZOI_CWOLF_V5",
            fixture_id: "owned_borzoi_c_wolf_v5",
            purpose: "OWNER_AUTHORIZED_MATERIAL_AND_C_WOLF_MOTION_PIPELINE_V5",
            source_sha256: "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
            source_byte_length: 29_889_104,
            source_triangle_count: 300_000,
            source_image_count: 3,
            source_task_id: None,
            rig_recipe: CandidateRigRecipe::V5,
            model_recipe: CandidateModelRecipe::FullMtrMotion,
            quality_recipe: CandidateQualityRecipe::MotionOracle,
        }),
        6 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre6",
            texture_resref: "m2aborztex6",
            hak_resref: "m2aborzhak6",
            module_resref: "m2aborzmod6",
            area_resref: "m2aborzarea6",
            creature_resref: "m2aborzutc6",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V6",
            area_display_name: "Meshy2Aurora Borzoi Test Area V6",
            appearance_label: "M2A_BORZOI_CWOLF_V6",
            fixture_id: "owned_borzoi_c_wolf_v6",
            purpose: "OWNER_AUTHORIZED_V5_RIG_CLASSIC_DIFFUSE_ISOLATION_V6",
            source_sha256: "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
            source_byte_length: 29_889_104,
            source_triangle_count: 300_000,
            source_image_count: 3,
            source_task_id: None,
            rig_recipe: CandidateRigRecipe::V5,
            model_recipe: CandidateModelRecipe::ClassicDiffuseMotion,
            quality_recipe: CandidateQualityRecipe::MotionOracle,
        }),
        7 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre7",
            texture_resref: "m2aborztex7",
            hak_resref: "m2aborzhak7",
            module_resref: "m2aborzmod7",
            area_resref: "m2aborzarea7",
            creature_resref: "m2aborzutc7",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V7",
            area_display_name: "Meshy2Aurora Borzoi Test Area V7",
            appearance_label: "M2A_BORZOI_CWOLF_V7",
            fixture_id: "owned_borzoi_c_wolf_v7",
            purpose: "OWNER_AUTHORIZED_V6_TWOSIDED_MATERIAL_REMEDIATION_V7",
            source_sha256: "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
            source_byte_length: 29_889_104,
            source_triangle_count: 300_000,
            source_image_count: 3,
            source_task_id: None,
            rig_recipe: CandidateRigRecipe::V5,
            model_recipe: CandidateModelRecipe::MinimalTwoSidedMtrMotion,
            quality_recipe: CandidateQualityRecipe::MotionOracle,
        }),
        8 => Ok(CandidateIdentity {
            version,
            model_resref: "m2aborzcre8",
            texture_resref: "m2aborztex8",
            hak_resref: "m2aborzhak8",
            module_resref: "m2aborzmod8",
            area_resref: "m2aborzarea8",
            creature_resref: "m2aborzutc8",
            module_display_name: "Meshy2Aurora Borzoi c_wolf Demo V8",
            area_display_name: "Meshy2Aurora Borzoi Test Area V8",
            appearance_label: "M2A_BORZOI_CWOLF_V8",
            fixture_id: "owned_borzoi_c_wolf_v8",
            purpose: "OWNER_AUTHORIZED_EXACT_C_WOLF_BIND_RETEST_V8",
            source_sha256: "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
            source_byte_length: 29_889_104,
            source_triangle_count: 300_000,
            source_image_count: 3,
            source_task_id: None,
            rig_recipe: CandidateRigRecipe::V5,
            model_recipe: CandidateModelRecipe::MinimalTwoSidedMtrRetargetedMotion,
            quality_recipe: CandidateQualityRecipe::MotionOracle,
        }),
        _ => Err(format!(
            "BORZOI-CWOLF-DEMO-CANDIDATE-VERSION: expected 1, 2, 3, 4, 5, 6, 7, or 8, got {version}"
        )),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ByteBinding {
    file_name: String,
    byte_length: usize,
    sha256: String,
}

struct KeyResource {
    payload: Vec<u8>,
    logical_bif_name: String,
    key_index: u32,
    resource_index: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InheritedDeformationClipQualityV4 {
    clip_name: String,
    sampled_times: Vec<f32>,
    edge_sample_count: u64,
    edge_stretch_over_two_count: u64,
    edge_compression_under_half_count: u64,
    triangle_sample_count: u64,
    world_normal_opposition_count: u64,
    triangle_area_collapse_count: u64,
    triangle_area_expansion_count: u64,
    max_edge_stretch: f32,
    min_edge_compression: f32,
    max_displacement: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InheritedDeformationQualityV4 {
    schema_version: u32,
    profile: String,
    clip_count: usize,
    clips: Vec<InheritedDeformationClipQualityV4>,
    edge_sample_count: u64,
    edge_stretch_over_two_count: u64,
    edge_compression_under_half_count: u64,
    triangle_sample_count: u64,
    world_normal_opposition_count: u64,
    triangle_area_collapse_count: u64,
    triangle_area_expansion_count: u64,
}

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let (
        source_path,
        key_path,
        output_directory,
        timestamp_utc,
        candidate_version,
        audit_motion_contract_only,
    ) = parse_args(env::args().skip(1))?;
    let candidate = candidate_identity(candidate_version)?;
    if output_directory.exists() {
        return Err(format!(
            "BORZOI-CWOLF-DEMO-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }

    let source_glb = read(&source_path, "source GLB")?;
    require_fingerprint(
        &source_glb,
        candidate.source_byte_length,
        candidate.source_sha256,
        "source GLB",
    )?;
    let source = ingest_glb(&source_glb, &GlbLimits::default())
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-GLB: {error}"))?;
    if source.report.statistics.triangle_count != candidate.source_triangle_count
        || source.ir.materials.len() != 1
        || source.ir.images.len() != candidate.source_image_count
        || source.ir.materials[0].base_color_factor != [1.0, 1.0, 1.0, 1.0]
    {
        return Err(format!(
            "BORZOI-CWOLF-DEMO-SOURCE-CONTRACT: expected {} triangles, {} images, one material and a white base-color factor",
            candidate.source_triangle_count, candidate.source_image_count
        ));
    }

    let retail_c_wolf = read_key_resource(&key_path, "c_wolf", 2002)?;
    require_fingerprint(
        &retail_c_wolf.payload,
        RETAIL_C_WOLF_BYTE_LENGTH,
        RETAIL_C_WOLF_SHA256,
        "base NWN c_wolf",
    )?;
    let retail_inspection = inspect_binary_mdl(&retail_c_wolf.payload)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-RETAIL-MDL: {error}"))?;
    let expected_topology = c_wolf_compatibility_topology_v1();
    let actual_topology = flatten_numbered_topology(&retail_inspection.node_tree.roots)?;
    if retail_inspection.model.name != "c_Wolf"
        || !retail_inspection
            .model
            .supermodel_name
            .eq_ignore_ascii_case("null")
        || retail_inspection.model.classification != 4
        || retail_inspection.model.animation_scale != 1.0
        || retail_inspection.node_tree.node_count != 30
        || retail_inspection.animations.len() != 42
        || actual_topology != expected_topology
    {
        return Err(
            "BORZOI-CWOLF-DEMO-RETAIL-CONTRACT: exact c_wolf name/root/classification/scale/30-node topology/42-animation contract drifted"
                .to_owned(),
        );
    }
    let motion_contract =
        build_c_wolf_motion_contract_exact_v3(RETAIL_C_WOLF_SHA256, &retail_inspection)
            .map_err(|error| format!("BORZOI-CWOLF-MOTION-CONTRACT: {error}"))?;
    validate_reference_supermodel_motion_oracle_v2(&motion_contract, &retail_inspection)
        .map_err(|error| format!("BORZOI-CWOLF-MOTION-ORACLE: {error}"))?;
    if motion_contract.clean_room_profile_id != "exact-inspected-supermodel-bind-v3"
        || motion_contract.nodes.len() != expected_topology.len()
    {
        return Err(
            "BORZOI-CWOLF-MOTION-EXACT-BIND-CONTRACT: materializer requires the exact inspected V3 carrier inventory"
                .to_owned(),
        );
    }
    if audit_motion_contract_only {
        return Ok(format!(
            "BORZOI_C_WOLF_MOTION_CONTRACT_AUDIT={{\"status\":\"PASS\",\"contractSha256\":\"{}\",\"requiredClips\":{},\"requiredEvents\":{}}}",
            motion_contract.content_sha256,
            motion_contract.required_clips.len(),
            motion_contract.required_events.len(),
        ));
    }

    // `c_wolf` is an animation-only supermodel. The retail `c_dog` mesh is a
    // read-only compatible consumer and therefore provides the meaningful
    // deformation baseline for distinguishing ordinary clip motion from
    // candidate-specific tearing. No retail payload is written or copied.
    let retail_c_dog = read_key_resource(&key_path, "c_dog", 2002)?;
    let retail_c_dog_inspection = inspect_binary_mdl(&retail_c_dog.payload)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-RETAIL-CDOG-MDL: {error}"))?;
    if !retail_c_dog_inspection
        .model
        .supermodel_name
        .eq_ignore_ascii_case("c_wolf")
        || !retail_c_dog_inspection
            .node_tree
            .roots
            .iter()
            .any(node_or_descendant_has_mesh)
    {
        return Err(
            "BORZOI-CWOLF-DEMO-RETAIL-CDOG-CONTRACT: c_dog is not a mesh-bearing c_wolf consumer"
                .to_owned(),
        );
    }
    let retail_c_dog_quality = inspect_inherited_motion_quality_with_tolerances_v2(
        &retail_c_dog_inspection,
        &retail_inspection,
        &motion_contract.required_clips,
        &motion_contract.tolerances,
    )
    .map_err(|error| format!("BORZOI-CWOLF-DEMO-RETAIL-CDOG-QUALITY: {error}"))?;
    eprintln!(
        "BORZOI_C_WOLF_RETAIL_CDOG_BASELINE={{\"sha256\":\"{}\",\"hardEdges\":{},\"softEdges\":{},\"collapsedTriangles\":{},\"expandedTriangles\":{},\"opposedNormals\":{},\"seamViolations\":{},\"edgeSamples\":{},\"triangleSamples\":{},\"seamSamples\":{}}}",
        sha256(&retail_c_dog.payload),
        retail_c_dog_quality.edge_outside_hard_limit_count,
        retail_c_dog_quality.edge_outside_soft_limit_count,
        retail_c_dog_quality.triangle_area_collapse_count,
        retail_c_dog_quality.triangle_area_expansion_count,
        retail_c_dog_quality.world_normal_opposition_count,
        retail_c_dog_quality.seam_pair_violation_count,
        retail_c_dog_quality.edge_sample_count,
        retail_c_dog_quality.triangle_sample_count,
        retail_c_dog_quality.seam_pair_sample_count,
    );

    let reference_manifest = c_wolf_reference_manifest(&retail_c_wolf.payload);
    let mesh_present = retail_inspection
        .node_tree
        .roots
        .iter()
        .any(node_or_descendant_has_mesh);
    let reference_packet = build_reference_proof_packet(
        &reference_manifest,
        "c-wolf-base-v1",
        &retail_c_wolf.payload,
        ExecutionMetadata {
            command_label: format!("materialize-borzoi-c-wolf-demo-v{}", candidate.version),
            timestamp_utc,
        },
        vec![
            capability(ReferenceCapability::Header, CapabilityStatus::Pass),
            capability(ReferenceCapability::CoreRanges, CapabilityStatus::Pass),
            capability(ReferenceCapability::NodeTree, CapabilityStatus::Pass),
            capability(
                ReferenceCapability::Mesh,
                if mesh_present {
                    CapabilityStatus::Pass
                } else {
                    CapabilityStatus::NotPresent
                },
            ),
            capability(ReferenceCapability::Controllers, CapabilityStatus::Pass),
            capability(ReferenceCapability::Animations, CapabilityStatus::Pass),
        ],
        vec![
            invariant("model-name-c-wolf", "c_Wolf", &retail_inspection.model.name),
            invariant(
                "supermodel-null",
                "NULL",
                &retail_inspection.model.supermodel_name,
            ),
            invariant(
                "classification-four",
                "4",
                &retail_inspection.model.classification.to_string(),
            ),
            invariant(
                "node-count-thirty",
                "30",
                &retail_inspection.node_tree.node_count.to_string(),
            ),
            invariant(
                "animation-count-forty-two",
                "42",
                &retail_inspection.animations.len().to_string(),
            ),
            invariant("exact-topology-match", "true", "true"),
        ],
    )
    .map_err(|error| format!("BORZOI-CWOLF-DEMO-PREF: {error}"))?;

    let (rig, fit_report) = match candidate.rig_recipe {
        CandidateRigRecipe::V1 => (
            build_c_wolf_compatible_rig_v1(&source)
                .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG: {error}"))?,
            None,
        ),
        CandidateRigRecipe::V2 => {
            let artifact = build_c_wolf_compatible_rig_v2(&source)
                .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG: {error}"))?;
            (
                artifact.rig,
                Some(
                    serde_json::to_value(artifact.fit_report)
                        .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG-REPORT-JSON: {error}"))?,
                ),
            )
        }
        CandidateRigRecipe::V3 => {
            let artifact = build_c_wolf_compatible_rig_v3(&source)
                .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG: {error}"))?;
            (
                artifact.rig,
                Some(
                    serde_json::to_value(artifact.fit_report)
                        .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG-REPORT-JSON: {error}"))?,
                ),
            )
        }
        CandidateRigRecipe::V4 => {
            let artifact = build_c_wolf_compatible_rig_v4(&source)
                .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG: {error}"))?;
            (
                artifact.rig,
                Some(
                    serde_json::to_value(artifact.fit_report)
                        .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG-REPORT-JSON: {error}"))?,
                ),
            )
        }
        CandidateRigRecipe::V5 => {
            let artifact = build_c_wolf_compatible_rig_v5(&source)
                .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG: {error}"))?;
            (
                artifact.rig,
                Some(
                    serde_json::to_value(artifact.fit_report)
                        .map_err(|error| format!("BORZOI-CWOLF-DEMO-RIG-REPORT-JSON: {error}"))?,
                ),
            )
        }
    };
    let mut contract = ReferenceSupermodelContractV1 {
        schema_version: 1,
        contract_id: "base-nwn-c-wolf-compatibility-v1".to_owned(),
        content_sha256: String::new(),
        supermodel_resref: "c_wolf".to_owned(),
        source_model_sha256: RETAIL_C_WOLF_SHA256.to_owned(),
        inspected_read_only: true,
        no_payload_copied: true,
        nodes: expected_topology
            .iter()
            .map(
                |(part_number, name, parent_part_number)| ReferenceSupermodelNodeV1 {
                    part_number: *part_number,
                    name: (*name).to_owned(),
                    parent_part_number: *parent_part_number,
                },
            )
            .collect(),
    };
    contract.content_sha256 = canonical_reference_supermodel_contract_sha256_v1(&contract)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-CONTRACT: {error}"))?;

    let model_build = match candidate.model_recipe {
        CandidateModelRecipe::FullMtrMotion => {
            let artifact = bind_static_mesh_for_inherited_supermodel_motion_direct_v2(
                &source_glb,
                &rig,
                &motion_contract,
                &retail_inspection,
                &default_reference_supermodel_writer_options_v1(candidate.model_resref),
                CreatureSourceForwardV1::PositiveZ,
                &ReferenceSupermodelMaterialOptionsV1 {
                    schema_version: 1,
                    texture_resref: candidate.texture_resref.to_owned(),
                },
            )
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-MOTION-MTR: {error}"))?;
            if !artifact.report.motion_compatible
                || !artifact.report.bind_pose_compatible
                || !artifact.report.skin_bind_compatible
                || !artifact.correction.report.reference_bind_verified
                || artifact.correction.report.reference_bind_max_abs_error
                    > motion_contract.tolerances.bind_max_abs_error
                || artifact.motion_quality.anchor_cluster_missing_count != 0
                || artifact.report.motion_correction_applied
                || artifact.correction.report.carrier_node_count != 30
                || artifact.correction.report.correction_node_count != 0
                || count_rig_nodes(&artifact.model.inspection) != 30
            {
                return Err(
                    "BORZOI-CWOLF-DEMO-MOTION-MTR-CONTRACT: direct-carrier/material/motion report is incomplete"
                        .to_owned(),
                );
            }
            CandidateModelBuild::MotionV2(Box::new(artifact))
        }
        CandidateModelRecipe::MinimalTwoSidedMtrMotion
        | CandidateModelRecipe::MinimalTwoSidedMtrCorrectedMotion
        | CandidateModelRecipe::MinimalTwoSidedMtrRetargetedMotion => {
            let corrected =
                candidate.model_recipe == CandidateModelRecipe::MinimalTwoSidedMtrCorrectedMotion;
            let retargeted =
                candidate.model_recipe == CandidateModelRecipe::MinimalTwoSidedMtrRetargetedMotion;
            let material_options = ReferenceSupermodelMinimalMtrMaterialOptionsV1 {
                schema_version: 1,
                texture_resref: candidate.texture_resref.to_owned(),
                material_resref: format!("{}_m0", candidate.texture_resref),
                material_profile: CreatureMaterialProfileV2 {
                    schema_version: 2,
                    target: CreatureMaterialTargetV2::NwnEeMtr,
                    normal_maps: false,
                    tangent_space_ready: false,
                    metallic_roughness_to_specular_gloss: false,
                    emissive_to_self_illumination: false,
                    alpha_mode: CreatureAlphaModeV2::Opaque,
                    double_sided: true,
                },
            };
            let artifact = if corrected {
                bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_corrected_v2(
                    &source_glb,
                    &rig,
                    &motion_contract,
                    &retail_inspection,
                    &default_reference_supermodel_writer_options_v1(candidate.model_resref),
                    CreatureSourceForwardV1::PositiveZ,
                    &material_options,
                )
            } else if retargeted {
                bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_retargeted_v4(
                    &source_glb,
                    &rig,
                    &motion_contract,
                    &retail_inspection,
                    &default_reference_supermodel_writer_options_v1(candidate.model_resref),
                    CreatureSourceForwardV1::PositiveZ,
                    &material_options,
                )
            } else {
                bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_v1(
                    &source_glb,
                    &rig,
                    &motion_contract,
                    &retail_inspection,
                    &default_reference_supermodel_writer_options_v1(candidate.model_resref),
                    CreatureSourceForwardV1::PositiveZ,
                    &material_options,
                )
            }
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-MOTION-MINIMAL-MTR: {error}"))?;
            let expected_correction_count = if corrected { 30 } else { 0 };
            let expected_rig_node_count = if corrected { 60 } else { 30 };
            if !artifact.report.motion_compatible
                || !artifact.report.bind_pose_compatible
                || !artifact.report.skin_bind_compatible
                || (retargeted
                    && (artifact.correction.report.schema_version != 4
                        || artifact.correction.report.reference_bind_verified))
                || (!retargeted
                    && (!artifact.correction.report.reference_bind_verified
                        || artifact.correction.report.reference_bind_max_abs_error
                            > motion_contract.tolerances.bind_max_abs_error))
                || artifact.motion_quality.anchor_cluster_missing_count != 0
                || !artifact.report.material_extension_applied
                || artifact.report.tangent_stream_count != 0
                || artifact.report.material_resource_count != 2
                || artifact.report.material_profile != "NWN_EE_MTR_TWOSIDED_ONLY_V1"
                || artifact.correction.report.carrier_node_count != 30
                || artifact.correction.report.correction_node_count != expected_correction_count
                || count_rig_nodes(&artifact.model.inspection) != expected_rig_node_count
            {
                return Err(
                    "BORZOI-CWOLF-DEMO-MOTION-MINIMAL-MTR-CONTRACT: carrier/correction motion and minimal twosided report is incomplete"
                        .to_owned(),
                );
            }
            CandidateModelBuild::MinimalMtrMotion(Box::new(artifact))
        }
        CandidateModelRecipe::ClassicDiffuseMotion => {
            let artifact = bind_static_mesh_for_inherited_supermodel_motion_direct_classic_v1(
                &source_glb,
                &rig,
                &motion_contract,
                &retail_inspection,
                &default_reference_supermodel_writer_options_v1(candidate.model_resref),
                CreatureSourceForwardV1::PositiveZ,
                &ReferenceSupermodelClassicMaterialOptionsV1 {
                    schema_version: 1,
                    texture_resref: candidate.texture_resref.to_owned(),
                },
            )
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-MOTION-CLASSIC: {error}"))?;
            if !artifact.report.motion_compatible
                || !artifact.report.bind_pose_compatible
                || !artifact.report.skin_bind_compatible
                || !artifact.correction.report.reference_bind_verified
                || artifact.correction.report.reference_bind_max_abs_error
                    > motion_contract.tolerances.bind_max_abs_error
                || artifact.motion_quality.anchor_cluster_missing_count != 0
                || artifact.report.material_extension_applied
                || artifact.report.tangent_stream_count != 0
                || artifact.report.material_profile != "CLASSIC_DIFFUSE_TGA_SAFE_V1"
                || artifact.correction.report.carrier_node_count != 30
                || artifact.correction.report.correction_node_count != 0
                || count_rig_nodes(&artifact.model.inspection) != 30
            {
                return Err(
                    "BORZOI-CWOLF-DEMO-MOTION-CLASSIC-CONTRACT: direct-carrier motion/classic diffuse report is incomplete"
                        .to_owned(),
                );
            }
            CandidateModelBuild::ClassicMotion(Box::new(artifact))
        }
        CandidateModelRecipe::LegacyDiffuse => {
            let retarget = retarget_static_mesh_to_reference_supermodel_v2(
                &source_glb,
                &rig,
                &contract,
                &MdlWriterOptionsV1 {
                    schema_version: 1,
                    format_profile:
                        MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
                    state_projection_profile:
                        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
                    state_projection_provenance: None,
                    model_resource_resref: candidate.model_resref.to_owned(),
                    diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                        material_slot: 0,
                        resref: candidate.texture_resref.to_owned(),
                    }],
                },
                CreatureSourceForwardV1::PositiveZ,
            )
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-RETARGET: {error}"))?;
            CandidateModelBuild::Legacy(Box::new(retarget))
        }
    };
    let model = match &model_build {
        CandidateModelBuild::Legacy(artifact) => &artifact.model,
        CandidateModelBuild::MotionV2(artifact) => &artifact.model,
        CandidateModelBuild::ClassicMotion(artifact) => &artifact.model,
        CandidateModelBuild::MinimalMtrMotion(artifact) => &artifact.model,
    };
    validate_generated_topology(
        &model.inspection.node_tree.roots,
        &expected_topology,
        candidate.model_resref,
    )?;
    let (basis_status, forward_mapping, basis_determinant, active_bone_count) = match &model_build {
        CandidateModelBuild::Legacy(artifact) => (
            artifact.conversion.report.policies.basis_status.as_str(),
            artifact
                .conversion
                .report
                .policies
                .asset_forward_mapping
                .as_str(),
            artifact.conversion.report.transform.determinant,
            artifact.report.active_bone_count,
        ),
        CandidateModelBuild::MotionV2(artifact) => (
            artifact.conversion.report.policies.basis_status.as_str(),
            artifact
                .conversion
                .report
                .policies
                .asset_forward_mapping
                .as_str(),
            artifact.conversion.report.transform.determinant,
            count_active_skin_bones(&artifact.model.inspection),
        ),
        CandidateModelBuild::ClassicMotion(artifact) => (
            artifact.conversion.report.policies.basis_status.as_str(),
            artifact
                .conversion
                .report
                .policies
                .asset_forward_mapping
                .as_str(),
            artifact.conversion.report.transform.determinant,
            count_active_skin_bones(&artifact.model.inspection),
        ),
        CandidateModelBuild::MinimalMtrMotion(artifact) => (
            artifact.conversion.report.policies.basis_status.as_str(),
            artifact
                .conversion
                .report
                .policies
                .asset_forward_mapping
                .as_str(),
            artifact.conversion.report.transform.determinant,
            count_active_skin_bones(&artifact.model.inspection),
        ),
    };
    if basis_status != "CREATURE_BASIS_V3_RESOLVED"
        || forward_mapping != "GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y"
        || basis_determinant != 1.0
        || model.inspection.model.name != candidate.model_resref
        || !model
            .inspection
            .model
            .supermodel_name
            .eq_ignore_ascii_case("c_wolf")
        || !model.inspection.animations.is_empty()
        || active_bone_count < 12
        || !model.report.semantic_diff.is_empty()
    {
        return Err(
            "BORZOI-CWOLF-DEMO-MODEL-CONTRACT: generated MDL failed basis/supermodel/static-skin/readback invariants"
                .to_owned(),
        );
    }
    let triangle_count = count_model_triangles(&model.inspection.node_tree.roots);
    if triangle_count != candidate.source_triangle_count {
        return Err(format!(
            "BORZOI-CWOLF-DEMO-TRIANGLE-DIFF: expected {}, got {triangle_count}",
            candidate.source_triangle_count
        ));
    }
    let inherited_deformation_quality = match (candidate.quality_recipe, &model_build) {
        (
            CandidateQualityRecipe::LegacyObserved | CandidateQualityRecipe::LegacyStrict,
            CandidateModelBuild::Legacy(_),
        ) => {
            let quality =
                inspect_inherited_deformation_quality_v4(&model.inspection, &retail_inspection)?;
            if candidate.quality_recipe == CandidateQualityRecipe::LegacyStrict
                && (quality.edge_stretch_over_two_count != 0
                    || quality.edge_compression_under_half_count != 0
                    || quality.triangle_area_collapse_count != 0
                    || quality.triangle_area_expansion_count != 0
                    || quality.clips.iter().any(|clip| {
                        !clip.max_displacement.is_finite() || clip.max_displacement <= 0.05
                    }))
            {
                return Err(format!(
                    "BORZOI-CWOLF-DEFORMATION-GATE: stretch={}, compression={}, collapse={}, expansion={} and every inherited clip must move the skin",
                    quality.edge_stretch_over_two_count,
                    quality.edge_compression_under_half_count,
                    quality.triangle_area_collapse_count,
                    quality.triangle_area_expansion_count,
                ));
            }
            Some(
                serde_json::to_value(quality)
                    .map_err(|error| format!("BORZOI-CWOLF-DEFORMATION-QUALITY-JSON: {error}"))?,
            )
        }
        (CandidateQualityRecipe::None, CandidateModelBuild::Legacy(_)) => None,
        (CandidateQualityRecipe::MotionOracle, CandidateModelBuild::MotionV2(artifact)) => Some(
            serde_json::to_value(&artifact.motion_quality)
                .map_err(|error| format!("BORZOI-CWOLF-MOTION-QUALITY-JSON: {error}"))?,
        ),
        (CandidateQualityRecipe::MotionOracle, CandidateModelBuild::ClassicMotion(artifact)) => {
            Some(
                serde_json::to_value(&artifact.motion_quality)
                    .map_err(|error| format!("BORZOI-CWOLF-MOTION-QUALITY-JSON: {error}"))?,
            )
        }
        (CandidateQualityRecipe::MotionOracle, CandidateModelBuild::MinimalMtrMotion(artifact)) => {
            Some(
                serde_json::to_value(&artifact.motion_quality)
                    .map_err(|error| format!("BORZOI-CWOLF-MOTION-QUALITY-JSON: {error}"))?,
            )
        }
        _ => {
            return Err(
                "BORZOI-CWOLF-DEMO-QUALITY-RECIPE: candidate quality and model recipes are inconsistent"
                    .to_owned(),
            );
        }
    };
    if candidate.quality_recipe == CandidateQualityRecipe::LegacyStrict
        && inherited_deformation_quality.is_none()
    {
        return Err(
            "BORZOI-CWOLF-DEFORMATION-GATE: V4 requires the resolved c_wolf deformation oracle"
                .to_owned(),
        );
    }

    let diffuse_texture = match &model_build {
        CandidateModelBuild::Legacy(_)
        | CandidateModelBuild::ClassicMotion(_)
        | CandidateModelBuild::MinimalMtrMotion(_) => {
            let decoded = decode_embedded_image_to_tga_v1(
                &source_glb,
                0,
                &GlbLimits::default(),
                &EmbeddedImageDecodeLimitsV1::default(),
            )
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-TEXTURE-DECODE: {error}"))?;
            Some(
                write_tga_v1(&decoded, &TgaWriterOptionsV1::default())
                    .map_err(|error| format!("BORZOI-CWOLF-DEMO-TEXTURE-WRITE: {error}"))?,
            )
        }
        CandidateModelBuild::MotionV2(_) => None,
    };

    let retail_appearance = read_key_resource(&key_path, "appearance", 2017)?;
    let appearance_limits = TwoDaLimitsV1::default();
    let retained_appearance = retain_two_da_row_prefix_v1(
        &retail_appearance.payload,
        AURORA_VISIBLE_APPEARANCE_ROWS,
        &appearance_limits,
    )
    .map_err(|error| format!("BORZOI-CWOLF-DEMO-APPEARANCE-PREFIX: {error}"))?;
    validate_dog_donor(&retained_appearance, &appearance_limits)?;
    let append_request = clone_two_da_row_request_v1(
        &retained_appearance,
        DOG_DONOR_PHYSICAL_ROW,
        &[
            text_assignment("LABEL", candidate.appearance_label),
            text_assignment("RACE", candidate.model_resref),
        ],
        &appearance_limits,
    )
    .map_err(|error| format!("BORZOI-CWOLF-DEMO-APPEARANCE-CLONE: {error}"))?;
    let appearance =
        append_two_da_row_v1(&retained_appearance, &append_request, &appearance_limits)
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-APPEARANCE-APPEND: {error}"))?;
    if u32::from(appearance.report.appended_row_index) != AURORA_VISIBLE_APPEARANCE_ROWS {
        return Err(format!(
            "BORZOI-CWOLF-DEMO-APPEARANCE-ROW: expected {}, got {}",
            AURORA_VISIBLE_APPEARANCE_ROWS, appearance.report.appended_row_index
        ));
    }

    let mut resources = vec![HakResourceInputV1 {
        resref: candidate.model_resref.to_owned(),
        resource_type: 2002,
        payload: model.payload.clone(),
    }];
    match (&model_build, &diffuse_texture) {
        (CandidateModelBuild::Legacy(_) | CandidateModelBuild::ClassicMotion(_), Some(texture)) => {
            resources.push(HakResourceInputV1 {
                resref: candidate.texture_resref.to_owned(),
                resource_type: 3,
                payload: texture.payload.clone(),
            });
        }
        (CandidateModelBuild::MinimalMtrMotion(artifact), Some(texture)) => {
            resources.push(HakResourceInputV1 {
                resref: candidate.texture_resref.to_owned(),
                resource_type: 3,
                payload: texture.payload.clone(),
            });
            resources.push(artifact.mtr_resource.clone());
        }
        (CandidateModelBuild::MotionV2(artifact), None) => {
            resources.extend(artifact.material_package.resources.iter().cloned());
        }
        _ => {
            return Err(
                "BORZOI-CWOLF-DEMO-MATERIAL-ROUTE: legacy and motion material states were mixed"
                    .to_owned(),
            );
        }
    }
    resources.push(HakResourceInputV1 {
        resref: "appearance".to_owned(),
        resource_type: 2017,
        payload: appearance.payload.clone(),
    });
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-HAK: {error}"))?;
    if let CandidateModelBuild::MotionV2(artifact) = &model_build {
        let motion_package = package_reference_supermodel_creature_hak_v1(
            artifact,
            &appearance.payload,
            &HakWriterOptionsV1::default(),
        )
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-MOTION-V2-HAK: {error}"))?;
        if motion_package.hak.payload != package.hak.payload
            || motion_package.resource_count != resources.len()
        {
            return Err(
                "BORZOI-CWOLF-DEMO-MOTION-V2-HAK-DIFF: common package and motion package differ"
                    .to_owned(),
            );
        }
    }
    if let (CandidateModelBuild::ClassicMotion(artifact), Some(texture)) =
        (&model_build, &diffuse_texture)
    {
        let motion_package = package_reference_supermodel_classic_creature_hak_v1(
            artifact,
            candidate.texture_resref,
            &texture.payload,
            &appearance.payload,
            &HakWriterOptionsV1::default(),
        )
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-MOTION-CLASSIC-HAK: {error}"))?;
        if motion_package.hak.payload != package.hak.payload
            || motion_package.resource_count != resources.len()
        {
            return Err(
                "BORZOI-CWOLF-DEMO-MOTION-CLASSIC-HAK-DIFF: common package and classic motion package differ"
                    .to_owned(),
            );
        }
    }
    if let (CandidateModelBuild::MinimalMtrMotion(artifact), Some(texture)) =
        (&model_build, &diffuse_texture)
    {
        let motion_package = package_reference_supermodel_minimal_twosided_creature_hak_v1(
            artifact,
            candidate.texture_resref,
            &texture.payload,
            &appearance.payload,
            &HakWriterOptionsV1::default(),
        )
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-MOTION-MINIMAL-MTR-HAK: {error}"))?;
        if motion_package.hak.payload != package.hak.payload
            || motion_package.resource_count != resources.len()
        {
            return Err(
                "BORZOI-CWOLF-DEMO-MOTION-MINIMAL-MTR-HAK-DIFF: common package and minimal MTR motion package differ"
                    .to_owned(),
            );
        }
    }
    let hak_readback = ErfArchive::parse(&package.hak.payload)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-HAK-READBACK: {error:?}"))?;
    for resource in &resources {
        let readback = hak_readback
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| format!("BORZOI-CWOLF-DEMO-HAK-RESOURCE: {error:?}"))?;
        if readback != resource.payload {
            return Err(format!(
                "BORZOI-CWOLF-DEMO-HAK-DIFF: {}:{}",
                resource.resref, resource.resource_type
            ));
        }
    }

    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: candidate.module_resref.to_owned(),
        area_resref: candidate.area_resref.to_owned(),
        hak_resref: candidate.hak_resref.to_owned(),
    };
    let fixture = BinaryCreatureProfiledFixtureV2 {
        fixture: BinaryCreatureOwnedFixtureV1 {
            id: candidate.fixture_id.to_owned(),
            template_resref: candidate.creature_resref.to_owned(),
            display_name: "Meshy Borzoi - c_wolf inherited animations".to_owned(),
            appearance_row: appearance.report.appended_row_index,
            position: M0RuntimePositionV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
        },
        runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
    };
    let module = build_binary_creature_profile_matrix_module_named_v3(
        &module_identity,
        std::slice::from_ref(&fixture),
        candidate.module_display_name,
        candidate.area_display_name,
        "Owned Meshy borzoi mesh, clean-room c_wolf-compatible skin and base-game c_wolf supermodel animations.",
    )
    .map_err(|error| format!("BORZOI-CWOLF-DEMO-MODULE: {error}"))?;
    if module.readback.scene.module_resref != candidate.module_resref
        || module.readback.scene.area_resref != candidate.area_resref
        || module.readback.scene.ordered_hak_resrefs != [candidate.hak_resref.to_owned()]
        || module.readback.fixtures != [fixture]
    {
        return Err("BORZOI-CWOLF-DEMO-MODULE-DIFF: offline MOD readback drifted".to_owned());
    }

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "BORZOI-CWOLF-DEMO-OUTPUT-CREATE {}: {error}",
            output_directory.display()
        )
    })?;
    let model_name = format!("{}.mdl", candidate.model_resref);
    let texture_name = format!("{}.tga", candidate.texture_resref);
    let hak_name = format!("{}.hak", candidate.hak_resref);
    let module_name = format!("{}.mod", candidate.module_resref);
    write_new(&output_directory.join(&model_name), &model.payload)?;
    let mut material_outputs = Vec::new();
    for resource in &resources {
        if resource.resource_type == 2002 || resource.resource_type == 2017 {
            continue;
        }
        let file_name = aurora_resource_file_name(resource)?;
        write_new(&output_directory.join(&file_name), &resource.payload)?;
        material_outputs.push(json!({
            "resref": resource.resref,
            "resourceType": resource.resource_type,
            "fileName": file_name,
            "byteLength": resource.payload.len(),
            "sha256": sha256(&resource.payload),
        }));
    }
    write_new(
        &output_directory.join("appearance.2da"),
        &appearance.payload,
    )?;
    write_new(&output_directory.join(&hak_name), &package.hak.payload)?;
    write_new(&output_directory.join(&module_name), &module.payload)?;
    write_json_new(&output_directory.join("rig-profile.json"), &rig)?;
    if let Some(report) = &fit_report {
        write_json_new(&output_directory.join("source-fit-report.json"), report)?;
    }
    if let Some(report) = &inherited_deformation_quality {
        write_json_new(
            &output_directory.join("inherited-deformation-quality.json"),
            report,
        )?;
    }
    if candidate.model_recipe != CandidateModelRecipe::LegacyDiffuse {
        write_json_new(
            &output_directory.join("supermodel-contract.json"),
            &motion_contract,
        )?;
    } else {
        write_json_new(
            &output_directory.join("supermodel-contract.json"),
            &contract,
        )?;
    }
    write_json_new(
        &output_directory.join("reference-c-wolf-proof-packet.json"),
        &reference_packet,
    )?;
    write_json_new(
        &output_directory.join("generated-model-inspection.json"),
        &model.inspection,
    )?;
    match &model_build {
        CandidateModelBuild::Legacy(artifact) => {
            write_json_new(
                &output_directory.join("retarget-report.json"),
                &artifact.report,
            )?;
            write_json_new(
                &output_directory.join("conversion-report.json"),
                &artifact.conversion.report,
            )?;
        }
        CandidateModelBuild::MotionV2(artifact) => {
            write_json_new(
                &output_directory.join("motion-build-report.json"),
                &artifact.report,
            )?;
            write_json_new(
                &output_directory.join("conversion-report.json"),
                &artifact.conversion.report,
            )?;
            write_json_new(
                &output_directory.join("correction-report.json"),
                &artifact.correction.report,
            )?;
            write_json_new(
                &output_directory.join("corrected-rig-profile.json"),
                &artifact.correction.rig,
            )?;
            write_json_new(
                &output_directory.join("material-semantics.json"),
                &artifact.material_semantics,
            )?;
            write_json_new(
                &output_directory.join("material-compilation.json"),
                &artifact.material_compilation,
            )?;
        }
        CandidateModelBuild::ClassicMotion(artifact) => {
            write_json_new(
                &output_directory.join("motion-build-report.json"),
                &artifact.report,
            )?;
            write_json_new(
                &output_directory.join("conversion-report.json"),
                &artifact.conversion.report,
            )?;
            write_json_new(
                &output_directory.join("correction-report.json"),
                &artifact.correction.report,
            )?;
            write_json_new(
                &output_directory.join("corrected-rig-profile.json"),
                &artifact.correction.rig,
            )?;
        }
        CandidateModelBuild::MinimalMtrMotion(artifact) => {
            write_json_new(
                &output_directory.join("motion-build-report.json"),
                &artifact.report,
            )?;
            write_json_new(
                &output_directory.join("conversion-report.json"),
                &artifact.conversion.report,
            )?;
            write_json_new(
                &output_directory.join("correction-report.json"),
                &artifact.correction.report,
            )?;
            write_json_new(
                &output_directory.join("corrected-rig-profile.json"),
                &artifact.correction.rig,
            )?;
            write_json_new(
                &output_directory.join("material-compilation.json"),
                &artifact.material_compilation,
            )?;
            write_json_new(
                &output_directory.join("material-extension-readback.json"),
                &artifact.material_extension,
            )?;
        }
    }
    write_json_new(
        &output_directory.join("package-manifest.json"),
        &package.manifest,
    )?;
    write_json_new(
        &output_directory.join("module-readback.json"),
        &module.readback,
    )?;

    let material_semantics = match &model_build {
        CandidateModelBuild::Legacy(_) | CandidateModelBuild::ClassicMotion(_) => {
            serde_json::Value::Null
        }
        CandidateModelBuild::MinimalMtrMotion(artifact) => json!({
            "profile": artifact.report.material_profile,
            "sourceCompilerTarget": artifact.material_compilation.target_profile,
            "sourceCompilerStatus": artifact.material_compilation.status,
            "minimalMtrResref": artifact.mtr_resource.resref,
            "minimalMtrSha256": sha256(&artifact.mtr_resource.payload),
            "twosided": true,
            "normalMapPackaged": false,
            "specularMapPackaged": false,
            "txiPackaged": false,
            "tangentStreamCount": artifact.report.tangent_stream_count,
        }),
        CandidateModelBuild::MotionV2(artifact) => {
            serde_json::to_value(&artifact.material_semantics)
                .map_err(|error| format!("BORZOI-CWOLF-MATERIAL-SEMANTICS-JSON: {error}"))?
        }
    };
    let primary_texture = resources
        .iter()
        .find(|resource| resource.resource_type == 3 && resource.resref == candidate.texture_resref)
        .ok_or("BORZOI-CWOLF-DEMO-PRIMARY-TEXTURE-MISSING")?;
    let handoff = json!({
        "schemaVersion": 1,
        "status": "awaiting_verified_native_install",
        "purpose": candidate.purpose,
        "candidateVersion": candidate.version,
        "testModuleFileName": module_name,
        "moduleDisplayName": candidate.module_display_name,
        "areaDisplayName": candidate.area_display_name,
        "moduleResref": candidate.module_resref,
        "areaResref": candidate.area_resref,
        "hakResref": candidate.hak_resref,
        "appearanceRow": appearance.report.appended_row_index,
        "appearanceDonorPhysicalRow": DOG_DONOR_PHYSICAL_ROW,
        "creatureTemplateResref": candidate.creature_resref,
        "modelResref": candidate.model_resref,
        "textureResref": candidate.texture_resref,
        "source": {
            "sha256": candidate.source_sha256,
            "byteLength": candidate.source_byte_length,
            "meshyTaskId": candidate.source_task_id,
            "triangleCount": source.report.statistics.triangle_count,
        },
        "retailReference": {
            "source": "BASE_NWN",
            "resref": "c_wolf",
            "resourceType": 2002,
            "sha256": RETAIL_C_WOLF_SHA256,
            "byteLength": RETAIL_C_WOLF_BYTE_LENGTH,
            "logicalBifName": retail_c_wolf.logical_bif_name,
            "keyIndex": retail_c_wolf.key_index,
            "resourceIndex": retail_c_wolf.resource_index,
            "nodeCount": 30,
            "animationCount": 42,
            "payloadPersisted": false,
        },
        "modelContract": {
            "supermodel": "c_wolf",
            "localAnimationCount": 0,
            "inheritedAnimationCount": 42,
            "targetRigNodeCount": rig.nodes.len(),
            "outputRigNodeCount": model.inspection.node_tree.node_count,
            "activeBoneCount": active_bone_count,
            "sourceForward": "GLTF_POSITIVE_Z",
            "auroraForward": "AURORA_POSITIVE_Y",
            "basisStatus": basis_status,
            "basisDeterminant": basis_determinant,
            "triangleCount": triangle_count,
            "sourceFitReport": fit_report,
            "inheritedDeformationQuality": inherited_deformation_quality,
            "materialSemantics": material_semantics,
            "retailCWolfDeformationCalibration": "NOT_APPLICABLE_RETAIL_C_WOLF_USES_RIGID_MESH_NODES_NOT_SKINMESH",
        },
        "fixture": {
            "position": [10.0, 14.5, 0.0],
            "orientation": [0.0, -1.0],
            "runtimeProfile": "active_monster_baseline",
            "expectedBehavior": "The borzoi faces the player and obtains idle/movement/combat animation clips from the base-game c_wolf supermodel.",
        },
        "outputs": {
            "module": binding(format!("{}.mod", candidate.module_resref), &module.payload),
            "hak": binding(format!("{}.hak", candidate.hak_resref), &package.hak.payload),
            "model": binding(model_name, &model.payload),
            "texture": binding(texture_name, &primary_texture.payload),
            "materialResources": material_outputs,
            "appearance": binding("appearance.2da", &appearance.payload),
        },
        "offlineReadback": {
            "model": "PASS",
            "hak": "PASS",
            "module": "PASS",
            "referenceProof": "PASS",
        },
        "visualProofOwnership": "OWNER",
    });
    write_json_new(&output_directory.join("handoff-preinstall.json"), &handoff)?;

    Ok(format!(
        "BORZOI_C_WOLF_DEMO_PREPARED={{\"module\":\"{}.mod\",\"moduleSha256\":\"{}\",\"hak\":\"{}.hak\",\"hakSha256\":\"{}\",\"appearanceRow\":{},\"modelSha256\":\"{}\"}}",
        candidate.module_resref,
        sha256(&module.payload),
        candidate.hak_resref,
        sha256(&package.hak.payload),
        appearance.report.appended_row_index,
        sha256(&model.payload),
    ))
}

fn inspect_inherited_deformation_quality_v4(
    target: &InspectionReport,
    supermodel: &InspectionReport,
) -> Result<InheritedDeformationQualityV4, String> {
    const CLIPS: [&str; 7] = [
        "cpause1",
        "cwalk",
        "crun",
        "ca1slashl",
        "ca1slashr",
        "cdamagel",
        "ckdbckdie",
    ];
    let mut clips = Vec::with_capacity(CLIPS.len());
    for clip_name in CLIPS {
        let clip = supermodel
            .animations
            .iter()
            .find(|clip| clip.name.eq_ignore_ascii_case(clip_name))
            .ok_or_else(|| format!("BORZOI-CWOLF-DEFORMATION-CLIP-MISSING: {clip_name}"))?;
        let sampled_times = [0.0, 0.25, 0.5, 0.75, 1.0]
            .into_iter()
            .map(|fraction| clip.length * fraction)
            .collect::<Vec<_>>();
        let mut quality = InheritedDeformationClipQualityV4 {
            clip_name: clip.name.clone(),
            sampled_times: sampled_times.clone(),
            edge_sample_count: 0,
            edge_stretch_over_two_count: 0,
            edge_compression_under_half_count: 0,
            triangle_sample_count: 0,
            world_normal_opposition_count: 0,
            triangle_area_collapse_count: 0,
            triangle_area_expansion_count: 0,
            max_edge_stretch: 0.0,
            min_edge_compression: f32::INFINITY,
            max_displacement: 0.0,
        };
        for time_seconds in sampled_times {
            let sample = evaluate_reference_supermodel_skin_deformation_v1(
                target,
                supermodel,
                clip_name,
                time_seconds,
            )
            .map_err(|error| format!("BORZOI-CWOLF-DEFORMATION: {error}"))?;
            quality.max_displacement = quality.max_displacement.max(sample.max_displacement);
            for skin in &sample.skins {
                let node = find_node_by_part_v4(&target.node_tree.roots, skin.node_part)
                    .ok_or_else(|| {
                        format!(
                            "BORZOI-CWOLF-DEFORMATION-SKIN-NODE-MISSING: {}",
                            skin.node_part
                        )
                    })?;
                let mesh = node.mesh.as_ref().ok_or_else(|| {
                    format!(
                        "BORZOI-CWOLF-DEFORMATION-SKIN-MESH-MISSING: {}",
                        skin.node_name
                    )
                })?;
                for face in &mesh.faces {
                    let indices = face.vertex_indices.map(usize::from);
                    if indices.iter().any(|index| *index >= skin.vertices.len()) {
                        return Err(format!(
                            "BORZOI-CWOLF-DEFORMATION-FACE-INDEX: {}",
                            skin.node_name
                        ));
                    }
                    let bind = indices.map(|index| skin.vertices[index].bind_world);
                    let moved = indices.map(|index| skin.vertices[index].sampled_world);
                    quality.triangle_sample_count += 1;
                    let bind_normal = triangle_cross_v4(bind);
                    let moved_normal = triangle_cross_v4(moved);
                    let bind_area_twice = vector_length_v4(bind_normal);
                    let moved_area_twice = vector_length_v4(moved_normal);
                    if bind_area_twice > 1.0e-8 {
                        let area_ratio = moved_area_twice / bind_area_twice;
                        if area_ratio < 0.2 {
                            quality.triangle_area_collapse_count += 1;
                        }
                        if area_ratio > 5.0 {
                            quality.triangle_area_expansion_count += 1;
                        }
                        if moved_area_twice > 1.0e-8 && dot_v4(bind_normal, moved_normal) < 0.0 {
                            quality.world_normal_opposition_count += 1;
                        }
                    }
                    for (left, right) in [(0, 1), (1, 2), (2, 0)] {
                        let bind_length = distance_v4(bind[left], bind[right]);
                        if bind_length <= 1.0e-8 {
                            continue;
                        }
                        let ratio = distance_v4(moved[left], moved[right]) / bind_length;
                        quality.edge_sample_count += 1;
                        quality.max_edge_stretch = quality.max_edge_stretch.max(ratio);
                        quality.min_edge_compression = quality.min_edge_compression.min(ratio);
                        if ratio > 2.0 {
                            quality.edge_stretch_over_two_count += 1;
                        }
                        if ratio < 0.5 {
                            quality.edge_compression_under_half_count += 1;
                        }
                    }
                }
            }
        }
        if !quality.min_edge_compression.is_finite() {
            quality.min_edge_compression = 1.0;
        }
        clips.push(quality);
    }
    Ok(InheritedDeformationQualityV4 {
        schema_version: 5,
        profile: "RESOLVED_C_WOLF_EDGE_AREA_AND_WORLD_NORMAL_ORACLE_V5".to_owned(),
        clip_count: clips.len(),
        edge_sample_count: clips.iter().map(|clip| clip.edge_sample_count).sum(),
        edge_stretch_over_two_count: clips
            .iter()
            .map(|clip| clip.edge_stretch_over_two_count)
            .sum(),
        edge_compression_under_half_count: clips
            .iter()
            .map(|clip| clip.edge_compression_under_half_count)
            .sum(),
        triangle_sample_count: clips.iter().map(|clip| clip.triangle_sample_count).sum(),
        world_normal_opposition_count: clips
            .iter()
            .map(|clip| clip.world_normal_opposition_count)
            .sum(),
        triangle_area_collapse_count: clips
            .iter()
            .map(|clip| clip.triangle_area_collapse_count)
            .sum(),
        triangle_area_expansion_count: clips
            .iter()
            .map(|clip| clip.triangle_area_expansion_count)
            .sum(),
        clips,
    })
}

fn find_node_by_part_v4(nodes: &[NodeReport], part: u32) -> Option<&NodeReport> {
    for node in nodes {
        if node.number == part {
            return Some(node);
        }
        if let Some(found) = find_node_by_part_v4(&node.children, part) {
            return Some(found);
        }
    }
    None
}

fn distance_v4(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2))
        .sqrt()
}

fn triangle_cross_v4(points: [[f32; 3]; 3]) -> [f32; 3] {
    let left = [
        points[1][0] - points[0][0],
        points[1][1] - points[0][1],
        points[1][2] - points[0][2],
    ];
    let right = [
        points[2][0] - points[0][0],
        points[2][1] - points[0][1],
        points[2][2] - points[0][2],
    ];
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn vector_length_v4(value: [f32; 3]) -> f32 {
    (value[0].powi(2) + value[1].powi(2) + value[2].powi(2)).sqrt()
}

fn dot_v4(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateModelRecipe, CandidateQualityRecipe, CandidateRigRecipe, candidate_identity,
    };

    #[test]
    fn v6_is_v5_rig_with_classic_diffuse_motion_recipe() {
        let v5 = candidate_identity(5).expect("V5 recipe");
        let v6 = candidate_identity(6).expect("V6 recipe");

        assert_eq!(v5.rig_recipe, CandidateRigRecipe::V5);
        assert_eq!(v5.model_recipe, CandidateModelRecipe::FullMtrMotion);
        assert_eq!(v6.rig_recipe, CandidateRigRecipe::V5);
        assert_eq!(v6.model_recipe, CandidateModelRecipe::ClassicDiffuseMotion);
        assert_eq!(v6.quality_recipe, CandidateQualityRecipe::MotionOracle);
        assert_eq!(v6.model_resref, "m2aborzcre6");
        assert_eq!(v6.hak_resref, "m2aborzhak6");
        assert_eq!(v6.module_resref, "m2aborzmod6");
    }

    #[test]
    fn v7_is_v5_rig_with_minimal_twosided_mtr_motion_recipe() {
        let v6 = candidate_identity(6).expect("V6 recipe");
        let v7 = candidate_identity(7).expect("V7 recipe");

        assert_eq!(v6.rig_recipe, CandidateRigRecipe::V5);
        assert_eq!(v7.rig_recipe, CandidateRigRecipe::V5);
        assert_eq!(
            v7.model_recipe,
            CandidateModelRecipe::MinimalTwoSidedMtrMotion
        );
        assert_eq!(v7.quality_recipe, CandidateQualityRecipe::MotionOracle);
        assert_eq!(v7.model_resref, "m2aborzcre7");
        assert_eq!(v7.texture_resref, "m2aborztex7");
        assert_eq!(v7.hak_resref, "m2aborzhak7");
        assert_eq!(v7.module_resref, "m2aborzmod7");
    }

    #[test]
    fn v8_preserves_v7_surface_semantics_and_uses_verified_target_bind_retargeting() {
        let v7 = candidate_identity(7).expect("V7 recipe");
        let v8 = candidate_identity(8).expect("V8 recipe");

        assert_eq!(v7.rig_recipe, CandidateRigRecipe::V5);
        assert_eq!(v8.rig_recipe, CandidateRigRecipe::V5);
        assert_eq!(
            v8.model_recipe,
            CandidateModelRecipe::MinimalTwoSidedMtrRetargetedMotion
        );
        assert_ne!(v8.model_recipe, v7.model_recipe);
        assert_eq!(v8.quality_recipe, CandidateQualityRecipe::MotionOracle);
        assert_eq!(v8.source_sha256, v7.source_sha256);
        assert_eq!(v8.source_triangle_count, 300_000);
        assert_eq!(v8.model_resref, "m2aborzcre8");
        assert_eq!(v8.texture_resref, "m2aborztex8");
        assert_eq!(v8.hak_resref, "m2aborzhak8");
        assert_eq!(v8.module_resref, "m2aborzmod8");
        assert_eq!(v8.purpose, "OWNER_AUTHORIZED_EXACT_C_WOLF_BIND_RETEST_V8");
    }
}

fn parse_args(
    arguments: impl IntoIterator<Item = String>,
) -> Result<(PathBuf, PathBuf, PathBuf, String, u32, bool), String> {
    let mut source = None;
    let mut key = None;
    let mut output = None;
    let mut timestamp = None;
    let mut candidate_version = None;
    let mut audit_motion_contract_only = false;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = |arguments: &mut dyn Iterator<Item = String>| {
            arguments
                .next()
                .ok_or_else(|| format!("BORZOI-CWOLF-DEMO-ARGUMENT-VALUE-MISSING: {argument}"))
        };
        match argument.as_str() {
            "--source" => source = Some(PathBuf::from(value(&mut arguments)?)),
            "--nwn-base-key" => key = Some(PathBuf::from(value(&mut arguments)?)),
            "--output" => output = Some(PathBuf::from(value(&mut arguments)?)),
            "--timestamp-utc" => timestamp = Some(value(&mut arguments)?),
            "--candidate-version" => {
                candidate_version =
                    Some(value(&mut arguments)?.parse::<u32>().map_err(|error| {
                        format!("BORZOI-CWOLF-DEMO-CANDIDATE-VERSION-PARSE: {error}")
                    })?)
            }
            "--audit-motion-contract-only" => audit_motion_contract_only = true,
            _ => return Err(format!("BORZOI-CWOLF-DEMO-ARGUMENT-UNKNOWN: {argument}")),
        }
    }
    Ok((
        source.ok_or("BORZOI-CWOLF-DEMO-SOURCE-MISSING")?,
        key.ok_or("BORZOI-CWOLF-DEMO-KEY-MISSING")?,
        output.ok_or("BORZOI-CWOLF-DEMO-OUTPUT-MISSING")?,
        timestamp.ok_or("BORZOI-CWOLF-DEMO-TIMESTAMP-MISSING")?,
        candidate_version.unwrap_or(1),
        audit_motion_contract_only,
    ))
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("BORZOI-CWOLF-DEMO-READ-{label}: {error}"))
}

fn write_new(path: &Path, payload: &[u8]) -> Result<(), String> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-WRITE-NEW {}: {error}", path.display()))?;
    output
        .write_all(payload)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-WRITE {}: {error}", path.display()))
}

fn write_json_new(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut payload = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-JSON: {error}"))?;
    payload.push(b'\n');
    write_new(path, &payload)
}

fn binding(file_name: impl Into<String>, payload: &[u8]) -> ByteBinding {
    ByteBinding {
        file_name: file_name.into(),
        byte_length: payload.len(),
        sha256: sha256(payload),
    }
}

fn sha256(payload: &[u8]) -> String {
    Sha256::digest(payload)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn require_fingerprint(
    payload: &[u8],
    expected_length: usize,
    expected_sha256: &str,
    label: &str,
) -> Result<(), String> {
    let actual_sha256 = sha256(payload);
    if payload.len() != expected_length || actual_sha256 != expected_sha256 {
        return Err(format!(
            "BORZOI-CWOLF-DEMO-FINGERPRINT-MISMATCH-{label}: expected {expected_length}/{expected_sha256}, got {}/{}",
            payload.len(),
            actual_sha256
        ));
    }
    Ok(())
}

fn text_assignment(column_name: &str, value: &str) -> TwoDaCellAssignmentV1 {
    TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text {
            value: value.to_owned(),
        },
    }
}

fn validate_dog_donor(bytes: &[u8], limits: &TwoDaLimitsV1) -> Result<(), String> {
    let inspection = inspect_two_da_v2(bytes, limits)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-DONOR-INSPECTION: {error}"))?;
    let donor = read_two_da_row_v2(bytes, DOG_DONOR_PHYSICAL_ROW, limits)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-DONOR-ROW: {error}"))?;
    let cells = inspection
        .columns
        .iter()
        .cloned()
        .zip(donor.cells)
        .collect::<BTreeMap<_, _>>();
    let text = |column: &str| {
        cells
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(column))
            .and_then(|(_, value)| match value {
                TwoDaCellValueV1::Text { value } => Some(value.as_str()),
                TwoDaCellValueV1::Null => None,
            })
    };
    if text("MODELTYPE") != Some("S") || text("RACE") != Some("c_dog") {
        return Err(
            "BORZOI-CWOLF-DEMO-DONOR-CONTRACT: physical row 176 is not the expected c_dog MODELTYPE S donor"
                .to_owned(),
        );
    }
    Ok(())
}

fn capability(capability: ReferenceCapability, status: CapabilityStatus) -> CapabilityResult {
    CapabilityResult {
        capability,
        status,
        diagnostics: Vec::new(),
    }
}

fn invariant(name: &str, expected: &str, actual: &str) -> InvariantResult {
    InvariantResult {
        invariant: name.to_owned(),
        status: if expected.eq_ignore_ascii_case(actual) {
            InvariantStatus::Pass
        } else {
            InvariantStatus::Fail
        },
        expected: Some(expected.to_owned()),
        actual: Some(actual.to_owned()),
        diagnostics: Vec::new(),
    }
}

fn c_wolf_reference_manifest(payload: &[u8]) -> ReferenceManifest {
    let expected_invariants = [
        "model-name-c-wolf",
        "supermodel-null",
        "classification-four",
        "node-count-thirty",
        "animation-count-forty-two",
        "exact-topology-match",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    ReferenceManifest {
        schema_version: 1,
        entries: vec![ReferenceManifestEntry {
            identity: ReferenceIdentity {
                reference_id: "c-wolf-base-v1".to_owned(),
                source: ReferenceSource::BaseNwn,
                resref: "c_wolf".to_owned(),
                resource_type: 2002,
            },
            expected_input: InputFingerprint::from_bytes(payload),
            expected_capabilities: vec![
                ReferenceCapability::Header,
                ReferenceCapability::CoreRanges,
                ReferenceCapability::NodeTree,
                ReferenceCapability::Mesh,
                ReferenceCapability::Controllers,
                ReferenceCapability::Animations,
            ],
            expected_invariants,
        }],
    }
}

fn flatten_numbered_topology(roots: &[NodeReport]) -> Result<NumberedTopologyV1, String> {
    fn visit(
        node: &NodeReport,
        parent: Option<u32>,
        values: &mut BTreeMap<u32, (String, Option<u32>)>,
    ) -> Result<(), String> {
        if values
            .insert(node.number, (node.name.clone(), parent))
            .is_some()
        {
            return Err(format!(
                "BORZOI-CWOLF-DEMO-RETAIL-NODE-DUPLICATE: {}",
                node.number
            ));
        }
        for child in &node.children {
            visit(child, Some(node.number), values)?;
        }
        Ok(())
    }

    let mut values = BTreeMap::new();
    for root in roots {
        visit(root, None, &mut values)?;
    }
    let expected = c_wolf_compatibility_topology_v1();
    let mut result = Vec::with_capacity(values.len());
    for (part, expected_name, _) in expected {
        let (actual_name, parent) = values
            .get(&part)
            .ok_or_else(|| format!("BORZOI-CWOLF-DEMO-RETAIL-NODE-MISSING: part {part}"))?;
        if actual_name != expected_name {
            return Err(format!(
                "BORZOI-CWOLF-DEMO-RETAIL-NODE-NAME: part {part} expected {expected_name}, got {actual_name}"
            ));
        }
        result.push((part, expected_name, *parent));
    }
    Ok(result)
}

fn validate_generated_topology(
    roots: &[NodeReport],
    expected: &[(u32, &str, Option<u32>)],
    model_resref: &str,
) -> Result<(), String> {
    fn visit(
        node: &NodeReport,
        parent: Option<u32>,
        values: &mut BTreeMap<u32, (String, Option<u32>)>,
    ) {
        values.insert(node.number, (node.name.clone(), parent));
        for child in &node.children {
            visit(child, Some(node.number), values);
        }
    }
    let mut values = BTreeMap::new();
    for root in roots {
        visit(root, None, &mut values);
    }
    for (part, name, parent) in expected {
        let actual = values
            .get(part)
            .ok_or_else(|| format!("BORZOI-CWOLF-DEMO-GENERATED-NODE-MISSING: part {part}"))?;
        let expected_name = if *part == 0 { model_resref } else { *name };
        if actual.0 != expected_name || actual.1 != *parent {
            return Err(format!(
                "BORZOI-CWOLF-DEMO-GENERATED-TOPOLOGY-DIFF: part {part} expected {expected_name}/{parent:?}, got {}/{:?}",
                actual.0, actual.1
            ));
        }
    }
    Ok(())
}

fn node_or_descendant_has_mesh(node: &NodeReport) -> bool {
    node.mesh.is_some() || node.children.iter().any(node_or_descendant_has_mesh)
}

fn count_model_triangles(nodes: &[NodeReport]) -> usize {
    fn count(node: &NodeReport) -> usize {
        node.mesh.as_ref().map_or(0, |mesh| mesh.faces.len())
            + node.children.iter().map(count).sum::<usize>()
    }
    nodes.iter().map(count).sum()
}

fn count_rig_nodes(report: &InspectionReport) -> usize {
    fn count(node: &NodeReport) -> usize {
        usize::from(node.mesh.is_none()) + node.children.iter().map(count).sum::<usize>()
    }
    report.node_tree.roots.iter().map(count).sum()
}

fn count_active_skin_bones(report: &InspectionReport) -> usize {
    fn visit(node: &NodeReport, active: &mut BTreeSet<u16>) {
        if let Some(skin) = &node.skin {
            for (weights, references) in skin.vertex_weights.iter().zip(&skin.bone_references) {
                for lane in 0..4 {
                    if weights[lane] > 0.0 && references[lane] != u16::MAX {
                        active.insert(references[lane]);
                    }
                }
            }
        }
        for child in &node.children {
            visit(child, active);
        }
    }
    let mut active = BTreeSet::new();
    for root in &report.node_tree.roots {
        visit(root, &mut active);
    }
    active.len()
}

fn aurora_resource_file_name(resource: &HakResourceInputV1) -> Result<String, String> {
    let extension = match resource.resource_type {
        3 => "tga",
        2022 => "txi",
        2072 => "mtr",
        other => {
            return Err(format!(
                "BORZOI-CWOLF-DEMO-MATERIAL-RESOURCE-TYPE: unsupported {other} for {}",
                resource.resref
            ));
        }
    };
    Ok(format!("{}.{}", resource.resref, extension))
}

fn read_key_resource(
    key_path: &Path,
    wanted_resref: &str,
    wanted_type: u16,
) -> Result<KeyResource, String> {
    const KEY_HEADER_SIZE: usize = 64;
    const KEY_ENTRY_SIZE: usize = 22;
    let key = fs::read(key_path).map_err(|error| format!("BORZOI-CWOLF-DEMO-KEY-READ: {error}"))?;
    if key.len() < KEY_HEADER_SIZE || &key[0..4] != b"KEY " || &key[4..8] != b"V1  " {
        return Err("BORZOI-CWOLF-DEMO-KEY-HEADER: expected KEY V1".to_owned());
    }
    let bif_count = u32_at(&key, 8)? as usize;
    let key_count = u32_at(&key, 12)? as usize;
    let bif_table_offset = u32_at(&key, 16)? as usize;
    let key_table_offset = u32_at(&key, 20)? as usize;
    checked_table(&key, bif_table_offset, bif_count, 12, "BIF table")?;
    checked_table(
        &key,
        key_table_offset,
        key_count,
        KEY_ENTRY_SIZE,
        "KEY table",
    )?;

    let mut selected = None;
    for index in 0..key_count {
        let offset = key_table_offset + index * KEY_ENTRY_SIZE;
        let resref = logical_resref(&key[offset..offset + 16]);
        let resource_type = u16::from_le_bytes([key[offset + 16], key[offset + 17]]);
        if resref.eq_ignore_ascii_case(wanted_resref) && resource_type == wanted_type {
            if selected.is_some() {
                return Err(format!(
                    "BORZOI-CWOLF-DEMO-KEY-DUPLICATE: {wanted_resref}:{wanted_type}"
                ));
            }
            selected = Some((index as u32, u32_at(&key, offset + 18)?));
        }
    }
    let (key_index, resource_id) = selected
        .ok_or_else(|| format!("BORZOI-CWOLF-DEMO-KEY-NOT-FOUND: {wanted_resref}:{wanted_type}"))?;
    let bif_index = (resource_id >> 20) as usize;
    let resource_index = resource_id & 0x000f_ffff;
    if bif_index >= bif_count {
        return Err("BORZOI-CWOLF-DEMO-KEY-BIF-INDEX-OOB".to_owned());
    }
    let bif_entry = bif_table_offset + bif_index * 12;
    let file_name_offset = u32_at(&key, bif_entry + 4)? as usize;
    let file_name_size = u16::from_le_bytes([key[bif_entry + 8], key[bif_entry + 9]]) as usize;
    let file_name_end = file_name_offset
        .checked_add(file_name_size)
        .filter(|end| *end <= key.len())
        .ok_or("BORZOI-CWOLF-DEMO-KEY-BIF-NAME-OOB")?;
    let logical_bif_name = String::from_utf8_lossy(&key[file_name_offset..file_name_end])
        .trim_end_matches('\0')
        .replace('\\', "/");
    let native_relative = logical_bif_name.replace('/', std::path::MAIN_SEPARATOR_STR);
    let key_parent = key_path.parent().ok_or("BORZOI-CWOLF-DEMO-KEY-PARENT")?;
    let installation_root = key_parent.parent().unwrap_or(key_parent);
    let candidates = [
        installation_root.join(&native_relative),
        key_parent.join(&native_relative),
    ];
    let bif_path = candidates
        .iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("BORZOI-CWOLF-DEMO-BIF-NOT-FOUND: {logical_bif_name}"))?;
    let payload = read_bif_resource(bif_path, resource_index, wanted_type)?;
    Ok(KeyResource {
        payload,
        logical_bif_name,
        key_index,
        resource_index,
    })
}

fn read_bif_resource(
    path: &Path,
    resource_index: u32,
    wanted_type: u16,
) -> Result<Vec<u8>, String> {
    let mut bif =
        File::open(path).map_err(|error| format!("BORZOI-CWOLF-DEMO-BIF-OPEN: {error}"))?;
    let mut header = [0u8; 20];
    bif.read_exact(&mut header)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-BIF-HEADER-READ: {error}"))?;
    if &header[0..4] != b"BIFF" || &header[4..8] != b"V1  " {
        return Err("BORZOI-CWOLF-DEMO-BIF-HEADER: expected BIFF V1".to_owned());
    }
    let variable_count = u32::from_le_bytes(header[8..12].try_into().unwrap());
    let variable_table_offset = u32::from_le_bytes(header[16..20].try_into().unwrap());
    if resource_index >= variable_count {
        return Err("BORZOI-CWOLF-DEMO-BIF-RESOURCE-INDEX-OOB".to_owned());
    }
    let entry_offset = u64::from(variable_table_offset) + u64::from(resource_index) * 16;
    bif.seek(SeekFrom::Start(entry_offset))
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-BIF-SEEK: {error}"))?;
    let mut entry = [0u8; 16];
    bif.read_exact(&mut entry)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-BIF-ENTRY-READ: {error}"))?;
    let payload_offset = u32::from_le_bytes(entry[4..8].try_into().unwrap());
    let payload_size = u32::from_le_bytes(entry[8..12].try_into().unwrap());
    let resource_type = u32::from_le_bytes(entry[12..16].try_into().unwrap());
    if resource_type != u32::from(wanted_type) {
        return Err(format!(
            "BORZOI-CWOLF-DEMO-BIF-TYPE-MISMATCH: expected {wanted_type}, got {resource_type}"
        ));
    }
    let payload_size = usize::try_from(payload_size)
        .map_err(|_| "BORZOI-CWOLF-DEMO-BIF-PAYLOAD-SIZE".to_owned())?;
    let mut payload = vec![0; payload_size];
    bif.seek(SeekFrom::Start(u64::from(payload_offset)))
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-BIF-PAYLOAD-SEEK: {error}"))?;
    bif.read_exact(&mut payload)
        .map_err(|error| format!("BORZOI-CWOLF-DEMO-BIF-PAYLOAD-READ: {error}"))?;
    Ok(payload)
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or("BORZOI-CWOLF-DEMO-BINARY-U32-OOB")?;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn checked_table(
    bytes: &[u8],
    offset: usize,
    count: usize,
    stride: usize,
    label: &str,
) -> Result<(), String> {
    let end = count
        .checked_mul(stride)
        .and_then(|length| offset.checked_add(length))
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| format!("BORZOI-CWOLF-DEMO-{label}-OOB"))?;
    let _ = end;
    Ok(())
}

fn logical_resref(bytes: &[u8]) -> String {
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..length]).into_owned()
}
