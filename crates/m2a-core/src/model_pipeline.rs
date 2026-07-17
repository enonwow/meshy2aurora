//! Model-only M6 proof composition. This module intentionally has no GFF,
//! creature-template, gameplay-class or module-generation responsibilities.

use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::ErfArchive,
    glb::{
        EmbeddedImageDecodeLimitsV1, GlbIngestResult, GlbLimits, decode_embedded_image_to_tga_v1,
        ingest_glb,
    },
    hak::{HakResourceInputV1, HakWriterOptionsV1, HakWriterReportV1},
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlWriterOptionsV1, MdlWriterReportV1,
        write_binary_mdl_with_animations,
    },
    owned_fixture::{synthetic_owned_m6_animation_mapping_v1, synthetic_owned_m6_rig_v1},
    package::{PackageManifestV1, write_model_package_v1},
    profile_a::{
        CreatureRigProfileV1, ProfileAAnimationMappingV1, ProfileAConversionReportV1,
        RigProvenanceV1, convert_profile_a_with_animations_v1,
        derive_meshy_h1_profile_and_mapping_v1,
    },
    proof_module::{ProofModuleReportV1, build_creature_proof_module_v1},
    tga::{TgaWriterOptionsV1, TgaWriterReportV1, write_tga_v1},
    two_da::{
        TwoDaAppendReportV1, TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1,
        TwoDaLimitsV1, append_two_da_row_v1, inspect_two_da_v2,
    },
};

pub const M6_MODEL_RESREF: &str = "m2a_m6p01";
pub const M6_TEXTURE_RESREF: &str = "m2a_m6t01";
pub const M6_APPEARANCE_LABEL: &str = "M2A_M6_PROOF";
/// Companion HAK for the one canonical Codex animation-proof module.
pub const M6_HAK_FILE_NAME: &str = "m2a_codex_aproof.hak";
pub const M6_PROOF_MODULE_FILE_NAME: &str = "m2a_codex_aproof.mod";
pub const M6_MANIFEST_FILE_NAME: &str = "materialization-manifest.json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub manifest_self_hash_policy: String,
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
    pub appearance: TwoDaAppendReportV1,
    pub hak: HakWriterReportV1,
    pub proof_module: ProofModuleReportV1,
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

pub fn build_m6_model_package_with_profile_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    rig: &CreatureRigProfileV1,
    mapping: &ProfileAAnimationMappingV1,
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
    build_m6_model_package_with_ingest_v1(
        source_glb,
        appearance_two_da,
        ingest,
        rig,
        &Default::default(),
        mapping,
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
    let input_glb_identity = identity(source_glb);
    let input_appearance_identity = identity(appearance_two_da);
    let glb_limits = GlbLimits::default();
    let animated = convert_profile_a_with_animations_v1(&ingest, rig, profile_options, mapping)
        .map_err(|error| {
            let stage = if error.code.starts_with("M4A-") {
                "animation"
            } else {
                "profile"
            };
            pipeline_error(stage, error.code, error.path, error.message)
        })?;
    if !animated.base.report.conversion_eligible {
        return Err(pipeline_error(
            "profile",
            "M6-PROFILE-INELIGIBLE",
            "conversion.report",
            "Profile A conversion did not produce an eligible model",
        ));
    }
    let creature = animated.base.creature.as_ref().ok_or_else(|| {
        pipeline_error(
            "profile",
            "M6-PROFILE-INELIGIBLE",
            "conversion.creature",
            "eligible conversion has no creature output",
        )
    })?;
    let animations = animated.animations.as_ref().ok_or_else(|| {
        pipeline_error(
            "animation",
            "M6-ANIMATION-MISSING",
            "conversion.animations",
            "M6 proof requires one mapped source animation",
        )
    })?;
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

    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)?;
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
    let tga = write_tga_v1(&texture_image, &TgaWriterOptionsV1::default())
        .map_err(|error| pipeline_error("texture", error.code, error.path, error.message))?;

    let mdl = write_binary_mdl_with_animations(
        creature,
        animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            model_resource_resref: M6_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: texture_selection.material_slot,
                resref: M6_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| pipeline_error("model", error.code, error.path, error.message))?;

    let appearance_inspection = inspect_two_da_v2(appearance_two_da, &TwoDaLimitsV1::default())
        .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;
    let required_columns = [
        "LABEL",
        "MOVERATE",
        "MODELTYPE",
        "RACE",
        "PORTRAIT",
        "ENVMAP",
        "BLOODCOLR",
        "WEAPONSCALE",
        "SIZECATEGORY",
    ];
    for required in required_columns {
        if !appearance_inspection
            .columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(required))
        {
            return Err(pipeline_error(
                "appearance",
                "M6-APPEARANCE-COLUMN-MISSING",
                format!("appearance.columns.{required}"),
                format!("appearance.2da requires column {required}"),
            ));
        }
    }
    let mut cells = [
        ("LABEL", M6_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M6_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let phenotype_aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present_phenotype_aliases = appearance_inspection
        .columns
        .iter()
        .filter(|column| {
            phenotype_aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present_phenotype_aliases.len() > 1 {
        return Err(pipeline_error(
            "appearance",
            "M6-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns.phenotype",
            "appearance.2da contains more than one supported phenotype alias",
        ));
    }
    if let Some(column) = present_phenotype_aliases.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    let appearance = append_two_da_row_v1(
        appearance_two_da,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| pipeline_error("appearance", error.code, error.path, error.message))?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M6_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: mdl.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M6_TEXTURE_RESREF.to_owned(),
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
        .find(M6_MODEL_RESREF, 2002)
        .map_err(map_erf_readback)?
        .to_vec();
    let texture = archive
        .find(M6_TEXTURE_RESREF, 3)
        .map_err(map_erf_readback)?
        .to_vec();
    let appearance_two_da = archive
        .find("appearance", 2017)
        .map_err(map_erf_readback)?
        .to_vec();
    let hak = package.hak.payload.clone();
    let package_manifest = package.manifest;
    let proof_module = build_creature_proof_module_v1(appearance.report.appended_row_index)
        .map_err(|error| pipeline_error("proof_module", error.code, error.path, error.message))?;

    let primitive = ingest.ir.primitives.first().ok_or_else(|| {
        pipeline_error(
            "ingest",
            "M6-GEOMETRY-MISSING",
            "meshes",
            "source GLB has no primitive",
        )
    })?;
    let output_segment = creature.segments.first().ok_or_else(|| {
        pipeline_error(
            "profile",
            "M6-GEOMETRY-MISSING",
            "creature.segments",
            "converted model has no segment",
        )
    })?;
    let active_joint_count = output_segment
        .weights
        .iter()
        .flat_map(|weights| weights.bone_node_ids)
        .flatten()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
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
            active_joint_count,
        },
        ingest: ingest.report,
        conversion: animated.base.report,
        model: mdl.report,
        texture: tga.report,
        appearance: appearance.report,
        hak: package.hak.report,
        proof_module: proof_module.report.clone(),
    };
    let report_json = json_bytes(&report, "report")?;
    let source_animation = ingest.ir.animations.first().ok_or_else(|| {
        pipeline_error(
            "animation",
            "M6-ANIMATION-MISSING",
            "animations",
            "source GLB has no animation",
        )
    })?;
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
        model_resref: M6_MODEL_RESREF.to_owned(),
        texture_resref: M6_TEXTURE_RESREF.to_owned(),
        animation: M6AnimationSummaryV1 {
            source_name: source_animation.name.clone().unwrap_or_default(),
            output_name: proof_clip.name.clone(),
            duration_seconds: proof_clip.length_seconds,
            has_motion,
        },
        provenance: rig.provenance.clone(),
        zero_reference_model_payload_copied: rig
            .provenance
            .attestations
            .no_reference_payload_copied,
        appearance_payload_policy: "PRESERVED_AND_APPENDED".to_owned(),
    };
    let summary_json = json_bytes(&summary, "summary")?;
    let generated_files = [
        ("generated/source.glb", source_glb),
        ("generated/m2a_m6p01.mdl", model.as_slice()),
        ("generated/m2a_m6t01.tga", texture.as_slice()),
        ("generated/appearance.2da", appearance_two_da.as_slice()),
        ("generated/m2a_codex_aproof.hak", hak.as_slice()),
        (
            "generated/m2a_codex_aproof.mod",
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
        status: "M6_MODEL_PACKAGE_MATERIALIZED".to_owned(),
        input_glb: input_glb_identity,
        input_appearance_two_da: input_appearance_identity,
        texture_selection,
        appended_physical_row: report.appearance.appended_row_index,
        generated_files,
        package_manifest: package_manifest.clone(),
        appearance_payload_policy: "PRESERVED_AND_APPENDED".to_owned(),
        manifest_self_hash_policy: "EXCLUDED_TO_AVOID_SELF_REFERENCE".to_owned(),
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

fn sanitize_meshy_h1_degenerate_triangles_v1(
    source: &mut GlbIngestResult,
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
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let length_squared = cross.iter().map(|value| value * value).sum::<f32>();
        if length_squared.is_finite() && length_squared > 1.0e-10 {
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
    primitive.indices = retained;
    let after = primitive.indices.len() / 3;
    let removed = before.saturating_sub(after);
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
        source.report.diagnostics.push(crate::glb::GlbDiagnostic {
            schema_version: 1,
            severity: "WARNING".to_owned(),
            code: "M4A-MESHY-H1-DEGENERATE-TRIANGLES-REMOVED".to_owned(),
            message: format!(
                "removed {removed} degenerate source triangles before Aurora materialization"
            ),
            byte_offset: None,
            json_path: Some("meshes[0].primitives[0].indices".to_owned()),
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
    let write_result = write_staging_packet(&staging, artifact);
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
            generated.join(format!("{M6_MODEL_RESREF}.mdl")),
            artifact.model.as_slice(),
        ),
        (
            generated.join(format!("{M6_TEXTURE_RESREF}.tga")),
            artifact.texture.as_slice(),
        ),
        (
            generated.join("appearance.2da"),
            artifact.appearance_two_da.as_slice(),
        ),
        (generated.join(M6_HAK_FILE_NAME), artifact.hak.as_slice()),
        (
            generated.join(M6_PROOF_MODULE_FILE_NAME),
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
