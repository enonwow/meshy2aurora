//! Production, fail-closed package route for the one admitted Meshy hierarchy candidate.
//! The route is intentionally separate from the non-admitting V3 experiment and
//! from legacy flat M0/V2 admission.

use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::ErfArchive,
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakResourceInputV1, HakWriterOptionsV1},
    hierarchy_experiment::{
        MeshyHierarchyCandidateModelContractV4, MeshyHierarchyCandidateProfileV4,
        build_meshy_hierarchy_candidate_model_v4, verify_meshy_hierarchy_candidate_model_v4,
    },
    model_pipeline::{
        M0AppearanceTableBindingV1, M0AppearanceTableScopeV1, M0RuntimeAppearanceBindingV1,
        M0RuntimeMeshEligibilityV1, M0RuntimeResourceBindingV1, M6ByteIdentityV1,
        M6TextureSelectionV1, inspect_m0_runtime_mesh_eligibility_v1,
        resolve_base_color_image_index_v1, verify_m0_full_runtime_appearance_table_binding_v1,
    },
    package::{PackageManifestV1, write_model_package_v1},
    profile_a::{convert_profile_a, derive_meshy_m0_static_rigid_profile_v1},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureMultiFixtureModuleArtifactV1,
        BinaryCreatureOwnedFixtureV1, BinaryM0VerticalSliceIdentityV1,
        BinaryM0VerticalSliceReadbackV1, M0RuntimeDirectionV1, M0RuntimePositionV1,
        build_binary_creature_multi_fixture_module_v1,
        build_binary_m0_vertical_slice_module_with_identity_v1,
        inspect_binary_creature_multi_fixture_module_v1,
        inspect_binary_m0_vertical_slice_module_v1,
    },
    runtime_evidence::{KnownRuntimeNegativeV1, known_r30_runtime_negative_v1},
    tga::{TgaWriterOptionsV1, write_tga_v1},
    two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        append_two_da_row_v1, inspect_two_da_v2, read_two_da_row_v2,
    },
};

pub const MESHY_HIERARCHY_PACKAGE_PROFILE_V4: &str =
    "MESHY_HIERARCHY_RUNTIME_CANDIDATE_PACKAGE_PROFILE_V4";
pub const MESHY_HIERARCHY_PACKAGE_CONTRACT_V4: &str =
    "MESHY_HIERARCHY_RUNTIME_CANDIDATE_PACKAGE_CONTRACT_V4";
const FULL_APPEARANCE_BYTE_LENGTH: u64 = 6_901_169;
const FULL_APPEARANCE_SHA256: &str =
    "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const FULL_APPEARANCE_ROWS: u32 = 15_100;
const MODEL_RESREF: &str = "m2a_m0p01";
const TEXTURE_RESREF: &str = "m2a_m0t01";
pub const M0_R31_HAK_SHA256: &str =
    "ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12";
pub const M0_R31_HAK_RESREF: &str = "m2a_m0r31";
pub const M0_R32_MODULE_RESREF: &str = "m2a_m0r32";
pub const M0_R32_AREA_RESREF: &str = "m2a_m0a32";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeshyHierarchyPackageProfileV4 {
    pub schema_version: u32,
    pub profile: String,
    pub model_profile: MeshyHierarchyCandidateProfileV4,
    pub model_profile_sha256: String,
    pub identity: BinaryM0VerticalSliceIdentityV1,
    pub full_appearance_byte_length: u64,
    pub full_appearance_sha256: String,
    pub full_appearance_physical_rows: u32,
    pub admission: KnownRuntimeNegativeV1,
    pub materialization_count: u32,
    pub runtime_profile_materialized: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeshyHierarchyPackageContractV4 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub structural_verdict: String,
    pub runtime_model_visibility: String,
    pub runtime_proof_completeness: String,
    pub package_profile_sha256: String,
    pub admission: KnownRuntimeNegativeV1,
    pub original_source: M6ByteIdentityV1,
    pub derived_source: M6ByteIdentityV1,
    pub model_contract: MeshyHierarchyCandidateModelContractV4,
    pub module: M0RuntimeResourceBindingV1,
    pub hak: M0RuntimeResourceBindingV1,
    pub model: M0RuntimeResourceBindingV1,
    pub texture: M0RuntimeResourceBindingV1,
    pub appearance_two_da: M0RuntimeResourceBindingV1,
    pub appearance_table: M0AppearanceTableBindingV1,
    pub appearance: M0RuntimeAppearanceBindingV1,
    pub binary_scene: BinaryM0VerticalSliceReadbackV1,
    pub mesh_eligibility: M0RuntimeMeshEligibilityV1,
    pub package_manifest: PackageManifestV1,
    pub materialization_count: u32,
    pub runtime_profile_materialized: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshyHierarchyPackageSummaryV4 {
    pub schema_version: u32,
    pub status: String,
    pub contract: MeshyHierarchyPackageContractV4,
    pub texture_selection: M6TextureSelectionV1,
    pub profile_sha256: String,
    pub contract_sha256: String,
    pub starts_toolset: bool,
    pub starts_nwn: bool,
    pub no_local_toolset_adapter: bool,
}

#[derive(Clone, Debug)]
pub struct MeshyHierarchyPackageArtifactV4 {
    pub original_source_glb: Vec<u8>,
    pub derived_source_glb: Vec<u8>,
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub profile: MeshyHierarchyPackageProfileV4,
    pub contract: MeshyHierarchyPackageContractV4,
    pub profile_json: Vec<u8>,
    pub contract_json: Vec<u8>,
    pub summary_json: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshyHierarchyPackageErrorV4 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for MeshyHierarchyPackageErrorV4 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for MeshyHierarchyPackageErrorV4 {}

/// Repackages the exact r31 model lineage in a fresh MOD whose sole creature
/// uses the generic runtime-complete GIT/UTC envelope. The function never
/// reads, writes, rebuilds, or copies the r31 HAK; callers must bind and verify
/// that immutable external archive separately.
pub fn build_meshy_r31_corrected_container_module_v1(
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<BinaryCreatureMultiFixtureModuleArtifactV1, MeshyHierarchyPackageErrorV4> {
    if identity.module_resref != M0_R32_MODULE_RESREF
        || identity.area_resref != M0_R32_AREA_RESREF
        || identity.hak_resref != M0_R31_HAK_RESREF
    {
        return Err(package_error(
            "M2A-HIERARCHY-CORRECTED-CONTAINER-IDENTITY",
            "identity",
            "corrected container must be exact m2a_m0r32/m2a_m0a32 over singleton m2a_m0r31",
        ));
    }
    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: identity.module_resref.clone(),
        area_resref: identity.area_resref.clone(),
        hak_resref: identity.hak_resref.clone(),
    };
    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: "m0_fixture".to_owned(),
        template_resref: "nw_dwarfmerc001".to_owned(),
        display_name: "Meshy M0 binary vertical-slice fixture".to_owned(),
        appearance_row: 15_100,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(&module_identity, &fixtures)
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let independent = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    if independent != module.readback
        || independent.module_resref != M0_R32_MODULE_RESREF
        || independent.area_resref != M0_R32_AREA_RESREF
        || independent.ordered_hak_resrefs != [M0_R31_HAK_RESREF]
        || independent.fixtures != fixtures
    {
        return Err(package_error(
            "M2A-HIERARCHY-CORRECTED-CONTAINER-READBACK",
            "module",
            "corrected container differs from exact identity, singleton r31 HAK, or fixture",
        ));
    }
    Ok(module)
}

pub fn declare_meshy_hierarchy_package_profile_v4(
    model_profile: &MeshyHierarchyCandidateProfileV4,
    identity: &BinaryM0VerticalSliceIdentityV1,
    full_appearance_two_da: &[u8],
) -> Result<MeshyHierarchyPackageProfileV4, MeshyHierarchyPackageErrorV4> {
    require_r31_identity(identity)?;
    if full_appearance_two_da.len() as u64 != FULL_APPEARANCE_BYTE_LENGTH
        || sha256_bytes(full_appearance_two_da) != FULL_APPEARANCE_SHA256
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-APPEARANCE-TRUST-ROOT",
            "fullAppearanceTwoDa",
            "r31 requires the exact full runtime appearance table and 15100-row prefix",
        ));
    }
    let appearance = inspect_two_da_v2(full_appearance_two_da, &TwoDaLimitsV1::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    if appearance.physical_row_count != FULL_APPEARANCE_ROWS {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-APPEARANCE-TRUST-ROOT",
            "fullAppearanceTwoDa",
            "r31 requires the exact full runtime appearance table and 15100-row prefix",
        ));
    }
    Ok(MeshyHierarchyPackageProfileV4 {
        schema_version: 4,
        profile: MESHY_HIERARCHY_PACKAGE_PROFILE_V4.to_owned(),
        model_profile: model_profile.clone(),
        model_profile_sha256: canonical_digest(model_profile, "modelProfile")?,
        identity: identity.clone(),
        full_appearance_byte_length: full_appearance_two_da.len() as u64,
        full_appearance_sha256: sha256_bytes(full_appearance_two_da),
        full_appearance_physical_rows: appearance.physical_row_count,
        admission: known_r30_runtime_negative_v1(),
        materialization_count: 1,
        runtime_profile_materialized: false,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_meshy_hierarchy_package_v4(
    original_glb: &[u8],
    derived_glb: &[u8],
    frozen_r30_model: &[u8],
    full_appearance_two_da: &[u8],
    profile: &MeshyHierarchyPackageProfileV4,
) -> Result<MeshyHierarchyPackageArtifactV4, MeshyHierarchyPackageErrorV4> {
    require_package_profile(profile, full_appearance_two_da)?;
    let model_artifact = build_meshy_hierarchy_candidate_model_v4(
        original_glb,
        derived_glb,
        frozen_r30_model,
        &profile.model_profile,
    )
    .map_err(|error| package_error(error.code, error.path, error.message))?;
    let ingest = ingest_glb(derived_glb, &GlbLimits::default()).map_err(|error| {
        package_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "derivedSource".to_owned()),
            error.message,
        )
    })?;
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest)
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let conversion = convert_profile_a(&ingest, &rig, &Default::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let creature = conversion.creature.ok_or_else(|| {
        package_error(
            "M2A-HIERARCHY-PACKAGE-CONVERSION",
            "derivedSource",
            "derived source did not produce the expected rigid creature IR",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, &creature)
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        derived_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        package_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "derivedSource.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let appearance = append_m0_runtime_appearance(full_appearance_two_da)?;
    if appearance.report.appended_row_index != 15_100 {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r31 fixture must remain bound to exact physical row 15100",
        ));
    }
    let module = build_binary_m0_vertical_slice_module_with_identity_v1(
        appearance.report.appended_row_index,
        &profile.identity,
    )
    .map_err(|error| package_error(error.code, error.path, error.message))?;
    let resources = vec![
        HakResourceInputV1 {
            resref: MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: model_artifact.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.payload.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        package_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    let model = archive.find(MODEL_RESREF, 2002).map_err(map_erf)?.to_vec();
    let texture_bytes = archive.find(TEXTURE_RESREF, 3).map_err(map_erf)?.to_vec();
    let appearance_bytes = archive.find("appearance", 2017).map_err(map_erf)?.to_vec();
    if model != model_artifact.model.payload
        || texture_bytes != texture.payload
        || appearance_bytes != appearance.payload
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-HAK-READBACK",
            "hak.resources",
            "exact generated resources differ after HAK readback",
        ));
    }
    let scene = inspect_binary_m0_vertical_slice_module_v1(&module.payload)
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    if scene.module_resref != profile.identity.module_resref
        || scene.area_resref != profile.identity.area_resref
        || scene.ordered_hak_resrefs != [profile.identity.hak_resref.clone()]
        || scene.fixture.appearance_row != 15_100
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-SCENE-IDENTITY",
            "module",
            "MOD readback differs from exact r31 Area/single-HAK/fixture identity",
        ));
    }
    let appearance_table = M0AppearanceTableBindingV1 {
        schema_version: 1,
        scope: M0AppearanceTableScopeV1::FullRuntimeAppendV1,
        input_physical_rows: appearance.report.physical_rows_before,
        output_physical_rows: appearance.report.physical_rows_after,
        appended_physical_row: appearance.report.appended_row_index,
        input_byte_length: appearance.report.source_byte_length,
        output_byte_length: appearance.report.output_byte_length,
        input_sha256: appearance.report.source_sha256.clone(),
        output_sha256: appearance.report.output_sha256.clone(),
        source_prefix_preserved: appearance.report.source_prefix_preserved,
    };
    verify_m0_full_runtime_appearance_table_binding_v1(
        &appearance_table,
        &appearance_bytes,
        scene.fixture.appearance_row,
    )
    .map_err(|error| package_error(error.code, error.path, error.message))?;
    let appearance_binding = M0RuntimeAppearanceBindingV1 {
        physical_row: scene.fixture.appearance_row,
        label: appearance_cell(&appearance_bytes, scene.fixture.appearance_row, "LABEL")?,
        model_type: appearance_cell(&appearance_bytes, scene.fixture.appearance_row, "MODELTYPE")?,
        race: appearance_cell(&appearance_bytes, scene.fixture.appearance_row, "RACE")?,
    };
    if appearance_binding.label != "M2A_M0_MESHY_RIGID"
        || appearance_binding.model_type != "S"
        || appearance_binding.race != MODEL_RESREF
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-APPEARANCE-RESOLVER",
            "appearance.row15100",
            "fixture row must resolve the exact direct Meshy model resource",
        ));
    }
    let mesh_eligibility = inspect_m0_runtime_mesh_eligibility_v1(&model)
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    if !mesh_eligibility.eligible || mesh_eligibility.texture_resrefs != [TEXTURE_RESREF] {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-MESH-INELIGIBLE",
            "model",
            "candidate model must preserve one eligible native rigid Meshy mesh and texture",
        ));
    }
    let contract = MeshyHierarchyPackageContractV4 {
        schema_version: 4,
        profile: MESHY_HIERARCHY_PACKAGE_CONTRACT_V4.to_owned(),
        candidate_admissible: true,
        structural_verdict: "STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED".to_owned(),
        runtime_model_visibility: "not_tested".to_owned(),
        runtime_proof_completeness: "missing".to_owned(),
        package_profile_sha256: canonical_digest(profile, "packageProfile")?,
        admission: profile.admission.clone(),
        original_source: identity(original_glb),
        derived_source: identity(derived_glb),
        model_contract: model_artifact.contract,
        module: resource_binding(&profile.identity.module_resref, &module.payload),
        hak: resource_binding(&profile.identity.hak_resref, &package.hak.payload),
        model: resource_binding(MODEL_RESREF, &model),
        texture: resource_binding(TEXTURE_RESREF, &texture_bytes),
        appearance_two_da: resource_binding("appearance", &appearance_bytes),
        appearance_table,
        appearance: appearance_binding,
        binary_scene: scene,
        mesh_eligibility,
        package_manifest: package.manifest,
        materialization_count: 1,
        runtime_profile_materialized: false,
    };
    let profile_json = json_bytes(profile, "profile")?;
    let contract_json = json_bytes(&contract, "contract")?;
    let summary = MeshyHierarchyPackageSummaryV4 {
        schema_version: 4,
        status: "R31_HIERARCHY_CANDIDATE_OFFLINE_MATERIALIZED_GATE_B_PENDING".to_owned(),
        contract: contract.clone(),
        texture_selection,
        profile_sha256: sha256_bytes(&profile_json),
        contract_sha256: sha256_bytes(&contract_json),
        starts_toolset: false,
        starts_nwn: false,
        no_local_toolset_adapter: true,
    };
    let summary_json = json_bytes(&summary, "summary")?;
    Ok(MeshyHierarchyPackageArtifactV4 {
        original_source_glb: original_glb.to_vec(),
        derived_source_glb: derived_glb.to_vec(),
        model,
        texture: texture_bytes,
        appearance_two_da: appearance_bytes,
        hak: package.hak.payload,
        module: module.payload,
        profile: profile.clone(),
        contract,
        profile_json,
        contract_json,
        summary_json,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_meshy_hierarchy_package_v4(
    contract: &MeshyHierarchyPackageContractV4,
    expected_profile: &MeshyHierarchyPackageProfileV4,
    original_glb: &[u8],
    derived_glb: &[u8],
    frozen_r30_model: &[u8],
    full_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), MeshyHierarchyPackageErrorV4> {
    if contract.schema_version != 4
        || contract.profile != MESHY_HIERARCHY_PACKAGE_CONTRACT_V4
        || !contract.candidate_admissible
        || contract.structural_verdict != "STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED"
        || contract.runtime_model_visibility != "not_tested"
        || contract.runtime_proof_completeness != "missing"
        || contract.materialization_count != 1
        || contract.runtime_profile_materialized
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-CONTRACT-VERSION",
            "contract",
            "only exact V4 Gate-B-pending candidate admission is accepted",
        ));
    }
    let replay = build_meshy_hierarchy_package_v4(
        original_glb,
        derived_glb,
        frozen_r30_model,
        full_appearance_two_da,
        expected_profile,
    )?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-REPLAY-MISMATCH",
            "contract",
            "contract/MOD/HAK differ from independent exact-input package replay",
        ));
    }
    verify_meshy_hierarchy_candidate_model_v4(
        &contract.model_contract,
        &expected_profile.model_profile,
        original_glb,
        derived_glb,
        frozen_r30_model,
        &replay.model,
    )
    .map_err(|error| package_error(error.code, error.path, error.message))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn write_meshy_hierarchy_package_v4(
    output_dir: &Path,
    artifact: &MeshyHierarchyPackageArtifactV4,
    frozen_r30_model: &[u8],
    full_appearance_two_da: &[u8],
) -> Result<(), MeshyHierarchyPackageErrorV4> {
    verify_meshy_hierarchy_package_v4(
        &artifact.contract,
        &artifact.profile,
        &artifact.original_source_glb,
        &artifact.derived_source_glb,
        frozen_r30_model,
        full_appearance_two_da,
        &artifact.module,
        &artifact.hak,
    )?;
    if output_dir.exists() {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-OUTPUT-EXISTS",
            output_dir.display().to_string(),
            "candidate packet destination must be absent",
        ));
    }
    let parent = output_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    let name = output_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("r31-hierarchy-candidate");
    let staging = parent.join(format!(".{name}.m2a-stage-{}", std::process::id()));
    if staging.exists() {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-STAGING-EXISTS",
            staging.display().to_string(),
            "pre-existing staging directory is never reused or deleted",
        ));
    }
    let generated = staging.join("generated");
    let reports = staging.join("reports");
    fs::create_dir_all(&generated).map_err(|error| io_error(&generated, error))?;
    fs::create_dir_all(&reports).map_err(|error| io_error(&reports, error))?;
    fs::create_dir_all(staging.join("live")).map_err(|error| io_error(&staging, error))?;
    for (path, bytes) in [
        (
            generated.join("original-source.glb"),
            artifact.original_source_glb.as_slice(),
        ),
        (
            generated.join("derived-source.glb"),
            artifact.derived_source_glb.as_slice(),
        ),
        (generated.join("m2a_m0p01.mdl"), artifact.model.as_slice()),
        (generated.join("m2a_m0t01.tga"), artifact.texture.as_slice()),
        (
            generated.join("appearance.2da"),
            artifact.appearance_two_da.as_slice(),
        ),
        (generated.join("m2a_m0r31.hak"), artifact.hak.as_slice()),
        (generated.join("m2a_m0r31.mod"), artifact.module.as_slice()),
        (
            reports.join("r31-hierarchy-package-profile-v4.json"),
            artifact.profile_json.as_slice(),
        ),
        (
            reports.join("r31-hierarchy-candidate-contract-v4.json"),
            artifact.contract_json.as_slice(),
        ),
        (
            reports.join("materialization-summary.json"),
            artifact.summary_json.as_slice(),
        ),
    ] {
        fs::write(&path, bytes).map_err(|error| io_error(&path, error))?;
    }
    fs::rename(&staging, output_dir).map_err(|error| io_error(output_dir, error))?;
    Ok(())
}

fn require_package_profile(
    profile: &MeshyHierarchyPackageProfileV4,
    full_appearance_two_da: &[u8],
) -> Result<(), MeshyHierarchyPackageErrorV4> {
    require_r31_identity(&profile.identity)?;
    if profile.schema_version != 4
        || profile.profile != MESHY_HIERARCHY_PACKAGE_PROFILE_V4
        || profile.model_profile_sha256 != canonical_digest(&profile.model_profile, "modelProfile")?
        || full_appearance_two_da.len() as u64 != FULL_APPEARANCE_BYTE_LENGTH
        || sha256_bytes(full_appearance_two_da) != FULL_APPEARANCE_SHA256
        || profile.full_appearance_byte_length != FULL_APPEARANCE_BYTE_LENGTH
        || profile.full_appearance_sha256 != FULL_APPEARANCE_SHA256
        || profile.full_appearance_byte_length != full_appearance_two_da.len() as u64
        || profile.full_appearance_sha256 != sha256_bytes(full_appearance_two_da)
        || profile.full_appearance_physical_rows != FULL_APPEARANCE_ROWS
        || profile.admission != known_r30_runtime_negative_v1()
        || profile.materialization_count != 1
        || profile.runtime_profile_materialized
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-PROFILE",
            "profile",
            "unknown, mixed, self-minted, non-r31, or runtime-promoted package profile is inadmissible",
        ));
    }
    Ok(())
}

fn require_r31_identity(
    identity: &BinaryM0VerticalSliceIdentityV1,
) -> Result<(), MeshyHierarchyPackageErrorV4> {
    if identity.module_resref != "m2a_m0r31"
        || identity.area_resref != "m2a_m0a31"
        || identity.hak_resref != "m2a_m0r31"
    {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-LINEAGE-IDENTITY",
            "identity",
            "the one authorized lineage is exact MOD m2a_m0r31, Area m2a_m0a31, ordered HAK [m2a_m0r31]",
        ));
    }
    Ok(())
}

fn append_m0_runtime_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, MeshyHierarchyPackageErrorV4> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", "M2A_M0_MESHY_RIGID"),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", MODEL_RESREF),
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
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(package_error(
            "M2A-HIERARCHY-PACKAGE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| package_error(error.code, error.path, error.message))
}

fn appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
) -> Result<String, MeshyHierarchyPackageErrorV4> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    let index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            package_error(
                "M2A-HIERARCHY-PACKAGE-APPEARANCE-COLUMN",
                column,
                "required column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| package_error(error.code, error.path, error.message))?;
    match row.cells.get(index) {
        Some(TwoDaCellValueV1::Text { value }) => Ok(value.clone()),
        _ => Err(package_error(
            "M2A-HIERARCHY-PACKAGE-APPEARANCE-CELL",
            column,
            "required fixture binding cell is absent or null",
        )),
    }
}

fn resource_binding(resref: &str, bytes: &[u8]) -> M0RuntimeResourceBindingV1 {
    let identity = identity(bytes);
    M0RuntimeResourceBindingV1 {
        resref: resref.to_owned(),
        byte_length: identity.byte_length,
        sha256: identity.sha256,
    }
}

fn identity(bytes: &[u8]) -> M6ByteIdentityV1 {
    M6ByteIdentityV1 {
        byte_length: bytes.len() as u64,
        sha256: sha256_bytes(bytes),
    }
}

fn canonical_digest<T: Serialize>(
    value: &T,
    path: &str,
) -> Result<String, MeshyHierarchyPackageErrorV4> {
    serde_json::to_vec(value)
        .map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| {
            package_error(
                "M2A-HIERARCHY-PACKAGE-SERIALIZATION",
                path,
                error.to_string(),
            )
        })
}

fn json_bytes<T: Serialize>(
    value: &T,
    path: &str,
) -> Result<Vec<u8>, MeshyHierarchyPackageErrorV4> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        package_error(
            "M2A-HIERARCHY-PACKAGE-SERIALIZATION",
            path,
            error.to_string(),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn map_erf(error: crate::erf::ErfError) -> MeshyHierarchyPackageErrorV4 {
    package_error(error.code, format!("hak@{}", error.offset), error.context)
}

fn io_error(path: &Path, error: std::io::Error) -> MeshyHierarchyPackageErrorV4 {
    package_error(
        "M2A-HIERARCHY-PACKAGE-IO",
        path.display().to_string(),
        error.to_string(),
    )
}

fn package_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> MeshyHierarchyPackageErrorV4 {
    MeshyHierarchyPackageErrorV4 {
        schema_version: 4,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
