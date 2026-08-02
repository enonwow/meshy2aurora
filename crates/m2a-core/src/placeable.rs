//! Static placeable resolver and package pipeline.
//!
//! This module owns only the placeable-specific chain:
//! `placeables.2da -> UTP/GIT/GIC -> HAK/MOD`. Binary model emission remains
//! in the common `mdl` module and consumes the model-kind-neutral
//! `AuroraModelIrV1`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GffArtifactV1, GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLimitsV1, GffLocStringV1,
        GffLocSubstringV1, GffStructV1, GffValueV1, GffWriterOptionsV1, read_gff_v32,
        write_gff_v32,
    },
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{
        HAK_MAX_OUTPUT_BYTES, HakResourceInputV1, HakWriterLimitsV1, HakWriterOptionsV1,
        write_erf_archive_v1, write_hak_v1,
    },
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
        MdlWriterOptionsV1, ParserLimits, write_binary_mdl, write_binary_mdl_exact_face_planes_v1,
        write_binary_mdl_exact_face_planes_with_readback_limits_v1,
    },
    model_ir::{AuroraModelIrV1, AuroraSegmentDeformationV1},
    model_limits::{AURORA_MODEL_TRIANGLE_BUDGET_V1, validate_model_triangle_budget_v1},
    model_material_separation::{
        ModelMaterialSeparationDocumentV1, ModelMaterialSeparationReportV1,
        resolve_model_materials_v1,
    },
    model_pipeline::{
        resolve_base_color_image_indices_v1,
        sanitize_static_model_degenerate_triangles_aggressive_v1,
        sanitize_static_model_degenerate_triangles_v1,
    },
    model_segmentation::segment_model_for_binary_mdl_v1,
    model_texture_authoring::{
        ModelTextureAuthoringDocumentV1, ModelTexturePayloadDescriptorV1,
        ModelTextureResolutionReportV1, resolve_model_texture_authoring_v1,
    },
    placeable_authoring::{
        PLACEABLE_AUTHORING_SCHEMA_VERSION_V2, PLACEABLE_COLLISION_SPEC_SCHEMA_VERSION_V1,
        PlaceableAuthoringApplyReportV1, PlaceableAuthoringDocumentV1,
        PlaceableAuthoringDocumentV2, PlaceableAuthoringProjectionV1,
        PlaceableCollisionCoordinateSpaceV1, PlaceableCollisionModeV1, PlaceableCollisionSpecV1,
        PlaceableElementInspectionV1, SHADOWLESS_MATERIAL_SUFFIX_V1,
        apply_placeable_authoring_to_ingest_v1,
        apply_placeable_authoring_with_material_separation_to_ingest_v1,
        default_placeable_authoring_v1, inspect_placeable_elements_v1, placeable_authoring_hash_v2,
        project_placeable_authoring_v1,
    },
    placeable_collision::{
        PLACEABLE_PWK_NONWALK_SURFACE_ID_V1, PlaceableWalkmeshIrV1,
        custom_placeable_walkmesh_ir_v1, derive_placeable_walkmesh_ir_v1,
        inspect_ascii_placeable_walkmesh_v1, sha256_hex, write_ascii_placeable_walkmesh_v1,
        write_placeable_walkmesh_v1,
    },
    placeable_texture::{
        PlaceableTextureAuthoringBootstrapV1, PlaceableTextureAuthoringDocumentV1,
        PlaceableTextureErrorV1, PlaceableTexturePayloadDescriptorV1,
        PlaceableTextureResolutionReportV1, inspect_placeable_material_textures_v1,
        resolve_placeable_texture_overrides_v1,
    },
    profile_a::{
        ProfileALimitsV1, ProfileAMaterialPolicyV1, ProfileAOptionsV1, ProfileATransformReportV1,
        convert_profile_a, convert_profile_a_owner_approved_oversized_v1,
        derive_meshy_m0_static_rigid_profile_v1,
    },
    proof_module::{
        M0_RUNTIME_FIXTURE_X, M0_RUNTIME_FIXTURE_Y, M0_RUNTIME_FIXTURE_Z,
        binary_creature_multi_fixture_gic, binary_creature_multi_fixture_git,
        binary_m0_area_for_named, binary_m0_module_ifo_for, proof_factions,
    },
    tga::{TgaWriterOptionsV1, write_tga_v1},
    two_da::{
        TwoDaAppendArtifactV1, TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1,
        TwoDaLimitsV1, append_two_da_row_v1, inspect_two_da_v2, read_two_da_row_v2,
    },
};

pub const PLACEABLE_SCHEMA_VERSION: u32 = 1;
pub const STATIC_PLACEABLE_BUILD_OPTIONS_SCHEMA_VERSION: u32 = 1;
pub const OWNER_APPROVED_OVERSIZED_PLACEABLE_SCHEMA_VERSION: u32 = 1;
pub const IFO_RESOURCE_TYPE: u16 = 2014;
pub const ARE_RESOURCE_TYPE: u16 = 2012;
pub const GIT_RESOURCE_TYPE: u16 = 2023;
pub const GIC_RESOURCE_TYPE: u16 = 2046;
pub const FAC_RESOURCE_TYPE: u16 = 2038;
pub const ITP_RESOURCE_TYPE: u16 = 2030;
pub const UTP_RESOURCE_TYPE: u16 = 2044;
pub const MDL_RESOURCE_TYPE: u16 = 2002;
pub const PLACEABLES_2DA_RESOURCE_TYPE: u16 = 2017;
pub const TGA_RESOURCE_TYPE: u16 = 3;
pub const DDS_RESOURCE_TYPE: u16 = 2033;
pub const PWK_RESOURCE_TYPE: u16 = 2053;

/// Placeable-domain name for the shared, model-kind-neutral geometry IR.
///
/// This is intentionally an alias, not a second representation: creature,
/// placeable and future tile profiles must enter the same binary MDL pipeline.
pub type AuroraPlaceableIrV1 = AuroraModelIrV1;

/// User-selectable Placeable build behavior. The default preserves every
/// finite, non-collinear face. The legacy absolute-epsilon cleanup is retained
/// only as an explicit experiment because it can punch holes in dense meshes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticPlaceableBuildOptionsV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub experimental_aggressive_geometry_cleanup: bool,
}

impl Default for StaticPlaceableBuildOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: STATIC_PLACEABLE_BUILD_OPTIONS_SCHEMA_VERSION,
            experimental_aggressive_geometry_cleanup: false,
        }
    }
}

/// Profile A admission for static placeables. Geometry thresholds remain the
/// shared render-model policy; only the material policy differs by target.
pub fn static_placeable_profile_a_options_v1() -> ProfileAOptionsV1 {
    ProfileAOptionsV1 {
        material_policy: ProfileAMaterialPolicyV1::BoundedSourceSlots,
        limits: ProfileALimitsV1 {
            max_unique_materials: 256,
            ..ProfileALimitsV1::default()
        },
        ..ProfileAOptionsV1::default()
    }
}

/// GLB admission for the static-placeable route uses the exact same geometry
/// budget as Creature and every other render-model target.
pub fn static_placeable_glb_limits_v1() -> GlbLimits {
    GlbLimits::default()
}

/// Explicit, source-bound exception for one owner-approved offline Placeable
/// materialization. Product defaults remain unchanged; callers must bind the
/// exception to the immutable source SHA-256 and record the authorization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnerApprovedOversizedPlaceableV1 {
    pub schema_version: u32,
    pub exact_source_sha256: String,
    pub authorization_note: String,
    pub max_input_bytes: usize,
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_decoded_geometry_bytes: usize,
    pub max_triangles: usize,
    pub max_profile_work_bytes: u64,
    pub max_hak_output_bytes: u64,
}

fn oversized_placeable_glb_limits_v1(
    source_glb: &[u8],
    exception: &OwnerApprovedOversizedPlaceableV1,
) -> Result<GlbLimits, PlaceableErrorV1> {
    validate_oversized_placeable_exception_v1(source_glb, exception)?;
    Ok(GlbLimits {
        max_input_bytes: exception.max_input_bytes,
        max_vertices: exception.max_vertices,
        max_indices: exception.max_indices,
        max_decoded_geometry_bytes: exception.max_decoded_geometry_bytes,
        triangle_blocking_above: exception.max_triangles,
        ..GlbLimits::default()
    })
}

fn oversized_placeable_profile_a_options_v1(
    exception: &OwnerApprovedOversizedPlaceableV1,
) -> ProfileAOptionsV1 {
    ProfileAOptionsV1 {
        material_policy: ProfileAMaterialPolicyV1::BoundedSourceSlots,
        limits: ProfileALimitsV1 {
            max_reference_vertices: exception.max_vertices as u64,
            max_reference_triangles: exception.max_triangles as u64,
            max_output_vertices: exception.max_vertices as u64,
            max_output_indices: exception.max_indices as u64,
            max_work_bytes: exception.max_profile_work_bytes,
            max_unique_materials: 256,
            triangle_blocking_above: exception.max_triangles as u64,
            ..ProfileALimitsV1::default()
        },
        ..ProfileAOptionsV1::default()
    }
}

fn validate_oversized_placeable_exception_v1(
    source_glb: &[u8],
    exception: &OwnerApprovedOversizedPlaceableV1,
) -> Result<(), PlaceableErrorV1> {
    if exception.schema_version != OWNER_APPROVED_OVERSIZED_PLACEABLE_SCHEMA_VERSION {
        return Err(error(
            "PLACEABLE-OVERSIZED-EXCEPTION-SCHEMA",
            "exception.schemaVersion",
            "owner-approved oversized Placeable exception schemaVersion must be 1",
        ));
    }
    if exception.authorization_note.trim().is_empty() {
        return Err(error(
            "PLACEABLE-OVERSIZED-EXCEPTION-AUTHORIZATION",
            "exception.authorizationNote",
            "owner-approved oversized Placeable exception requires a durable authorization note",
        ));
    }
    let actual_sha256 = sha256(source_glb);
    if exception.exact_source_sha256 != actual_sha256 {
        return Err(error(
            "PLACEABLE-OVERSIZED-EXCEPTION-SOURCE-MISMATCH",
            "exception.exactSourceSha256",
            format!(
                "exception is bound to {}, but source SHA-256 is {actual_sha256}",
                exception.exact_source_sha256
            ),
        ));
    }
    if source_glb.len() > exception.max_input_bytes
        || exception.max_vertices == 0
        || exception.max_indices == 0
        || exception.max_decoded_geometry_bytes == 0
        || exception.max_triangles <= AURORA_MODEL_TRIANGLE_BUDGET_V1
        || exception.max_profile_work_bytes == 0
        || exception.max_hak_output_bytes == 0
        || exception.max_hak_output_bytes > HAK_MAX_OUTPUT_BYTES
    {
        return Err(error(
            "PLACEABLE-OVERSIZED-EXCEPTION-LIMITS",
            "exception",
            "owner-approved oversized Placeable exception limits are incomplete or do not admit the exact source",
        ));
    }
    Ok(())
}

const REQUIRED_PLACEABLES_COLUMNS: [&str; 13] = [
    "Label",
    "StrRef",
    "ModelName",
    "LightColor",
    "LightOffsetX",
    "LightOffsetY",
    "LightOffsetZ",
    "SoundAppType",
    "ShadowSize",
    "BodyBag",
    "LowGore",
    "Reflection",
    "Static",
];

const UTP_FIELD_MANIFEST: [(&str, &str); 52] = [
    ("Tag", "CEXOSTRING"),
    ("LocName", "LOCSTRING"),
    ("Description", "LOCSTRING"),
    ("TemplateResRef", "RESREF"),
    ("AutoRemoveKey", "BYTE"),
    ("CloseLockDC", "BYTE"),
    ("Conversation", "RESREF"),
    ("Interruptable", "BYTE"),
    ("Faction", "DWORD"),
    ("Plot", "BYTE"),
    ("KeyRequired", "BYTE"),
    ("Lockable", "BYTE"),
    ("Locked", "BYTE"),
    ("OpenLockDC", "BYTE"),
    ("PortraitId", "WORD"),
    ("TrapDetectable", "BYTE"),
    ("TrapDetectDC", "BYTE"),
    ("TrapDisarmable", "BYTE"),
    ("DisarmDC", "BYTE"),
    ("TrapFlag", "BYTE"),
    ("TrapOneShot", "BYTE"),
    ("TrapType", "BYTE"),
    ("KeyName", "CEXOSTRING"),
    ("AnimationState", "BYTE"),
    ("Appearance", "DWORD"),
    ("HP", "SHORT"),
    ("CurrentHP", "SHORT"),
    ("Hardness", "BYTE"),
    ("Fort", "BYTE"),
    ("Ref", "BYTE"),
    ("Will", "BYTE"),
    ("OnClosed", "RESREF"),
    ("OnDamaged", "RESREF"),
    ("OnDeath", "RESREF"),
    ("OnDisarm", "RESREF"),
    ("OnHeartbeat", "RESREF"),
    ("OnLock", "RESREF"),
    ("OnMeleeAttacked", "RESREF"),
    ("OnOpen", "RESREF"),
    ("OnSpellCastAt", "RESREF"),
    ("OnTrapTriggered", "RESREF"),
    ("OnUnlock", "RESREF"),
    ("OnUserDefined", "RESREF"),
    ("HasInventory", "BYTE"),
    ("BodyBag", "BYTE"),
    ("Static", "BYTE"),
    ("Type", "BYTE"),
    ("Useable", "BYTE"),
    ("OnInvDisturbed", "RESREF"),
    ("OnUsed", "RESREF"),
    ("PaletteID", "BYTE"),
    ("Comment", "CEXOSTRING"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableAppearanceRowV1 {
    /// Engine-facing UTP/GIT type is DWORD. The current generic 2DA writer has
    /// a lower explicit product limit, but that limit never changes this
    /// domain type.
    pub value: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceablePlacementV1 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub bearing: f32,
}

impl Default for PlaceablePlacementV1 {
    fn default() -> Self {
        Self {
            x: M0_RUNTIME_FIXTURE_X,
            y: M0_RUNTIME_FIXTURE_Y,
            z: M0_RUNTIME_FIXTURE_Z,
            bearing: 0.0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticPlaceableIdentityV1 {
    pub module_resref: String,
    pub module_file_name: String,
    pub module_display_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_resref: String,
    pub hak_file_name: String,
    pub model_resref: String,
    pub texture_resref: String,
    pub blueprint_resref: String,
    pub object_tag: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticPlaceableBlueprintV1 {
    pub schema_version: u32,
    pub template_resref: String,
    pub object_tag: String,
    pub display_name: String,
    pub appearance_row: PlaceableAppearanceRowV1,
    pub palette_id: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceableTextureInputV1 {
    pub resref: String,
    pub resource_type: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticPlaceableBuildRequestV1 {
    pub schema_version: u32,
    pub identity: StaticPlaceableIdentityV1,
    pub placement: PlaceablePlacementV1,
    pub palette_id: u8,
    pub base_placeables_2da: Vec<u8>,
    pub model: AuroraModelIrV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision_model: Option<AuroraModelIrV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authoring_report: Option<PlaceableAuthoringApplyReportV1>,
    pub material_textures: Vec<MdlMaterialTextureBindingV1>,
    pub textures: Vec<PlaceableTextureInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Placeable2daArtifactV1 {
    pub appearance_row: PlaceableAppearanceRowV1,
    pub append: TwoDaAppendArtifactV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPlaceablePackageReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub profile: String,
    pub component_statuses: StaticPlaceableComponentStatusesV1,
    pub module_file_name: String,
    pub module_display_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_file_name: String,
    pub model_resref: String,
    pub texture_resref: String,
    pub blueprint_resref: String,
    pub object_tag: String,
    pub appearance_row: PlaceableAppearanceRowV1,
    pub placement: PlaceablePlacementV1,
    pub source_model_sha256: String,
    pub mdl_sha256: String,
    pub pwk_sha256: String,
    pub texture_sha256: String,
    #[serde(default)]
    pub texture_resources: Vec<PlaceableTextureBindingReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_authoring: Option<PlaceableTextureResolutionReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_separation: Option<ModelMaterialSeparationReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_texture_authoring: Option<ModelTextureResolutionReportV1>,
    pub placeables_2da_sha256: String,
    pub utp_sha256: String,
    pub itp_sha256: String,
    pub git_sha256: String,
    pub gic_sha256: String,
    pub hak_sha256: String,
    pub module_sha256: String,
    pub hak_resource_count: u32,
    pub module_resource_count: u32,
    pub model_visibility: String,
    pub proof_completeness: String,
    pub palette_completeness: String,
    pub collision_completeness: String,
    #[serde(default)]
    pub experimental_aggressive_geometry_cleanup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authoring: Option<PlaceableAuthoringApplyReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision: Option<ResolvedPlaceableCollisionV1>,
    pub resources: Vec<PlaceableResourceBindingReportV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPlaceableCollisionV1 {
    pub schema_version: u32,
    pub mode: PlaceableCollisionModeV1,
    pub source_coordinate_space: PlaceableCollisionCoordinateSpaceV1,
    pub output_coordinate_space: String,
    pub input_vertices: Vec<[f32; 2]>,
    pub source_vertices: Vec<[f32; 2]>,
    pub vertices: Vec<[f32; 2]>,
    pub triangles: Vec<[u32; 3]>,
    pub bounds_min: [f32; 2],
    pub bounds_max: [f32; 2],
    pub surface_id: i32,
    pub authoring_sha256: String,
    pub collision_sha256: String,
    pub pwk_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableTextureBindingReportV1 {
    pub resref: String,
    pub resource_type: u16,
    pub material_slots: Vec<u32>,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPlaceableAuthoringBootstrapV1 {
    pub schema_version: u32,
    pub inspection: PlaceableElementInspectionV1,
    pub document: PlaceableAuthoringDocumentV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPlaceableAuthoringBootstrapV2 {
    pub schema_version: u32,
    pub inspection: PlaceableElementInspectionV1,
    pub document: PlaceableAuthoringDocumentV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPlaceableComponentStatusesV1 {
    pub mdl: String,
    pub pwk: String,
    pub two_da: String,
    pub utp: String,
    pub git_gic: String,
    pub palette: String,
    pub package: String,
    pub proof: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableResourceBindingReportV1 {
    pub container: String,
    pub role: String,
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticPlaceablePackageArtifactV1 {
    pub hak_payload: Vec<u8>,
    pub module_payload: Vec<u8>,
    pub report: StaticPlaceablePackageReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for PlaceableErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for PlaceableErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> PlaceableErrorV1 {
    PlaceableErrorV1 {
        schema_version: PLACEABLE_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn map_error(code: &str, path: &str, source: impl fmt::Display) -> PlaceableErrorV1 {
    error(code, path, source.to_string())
}

fn text_cell(column_name: &str, value: &str) -> TwoDaCellAssignmentV1 {
    TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Text {
            value: value.to_owned(),
        },
    }
}

fn null_cell(column_name: &str) -> TwoDaCellAssignmentV1 {
    TwoDaCellAssignmentV1 {
        column_name: column_name.to_owned(),
        value: TwoDaCellValueV1::Null,
    }
}

pub fn append_static_placeable_2da_v1(
    base: &[u8],
    label: &str,
    model_resref: &str,
) -> Result<Placeable2daArtifactV1, PlaceableErrorV1> {
    validate_identifier(label, 64, "label")?;
    validate_resref(model_resref, "modelResref")?;
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(base, &limits)
        .map_err(|source| map_error("PLACEABLE-2DA-INVALID", "basePlaceables2da", source))?;
    for required in REQUIRED_PLACEABLES_COLUMNS {
        if !inspection
            .columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(required))
        {
            return Err(error(
                "PLACEABLE-2DA-COLUMN-MISSING",
                "basePlaceables2da.columns",
                format!("required placeables.2da column {required:?} is missing"),
            ));
        }
    }
    if inspection.physical_row_count > u32::from(u16::MAX) {
        return Err(error(
            "PLACEABLE-2DA-WRITER-LIMIT",
            "basePlaceables2da.rows",
            "engine Appearance is DWORD, but the current generic append writer is explicitly limited to u16::MAX physical rows",
        ));
    }

    let request = TwoDaAppendRequestV1 {
        schema_version: 1,
        cells: vec![
            text_cell("Label", label),
            null_cell("StrRef"),
            text_cell("ModelName", model_resref),
            null_cell("LightColor"),
            null_cell("LightOffsetX"),
            null_cell("LightOffsetY"),
            null_cell("LightOffsetZ"),
            null_cell("SoundAppType"),
            text_cell("ShadowSize", "1"),
            text_cell("BodyBag", "0"),
            null_cell("LowGore"),
            null_cell("Reflection"),
            text_cell("Static", "1"),
        ],
    };
    let append = append_two_da_row_v1(base, &request, &limits)
        .map_err(|source| map_error("PLACEABLE-2DA-APPEND-FAILED", "placeables.2da", source))?;
    let appearance_row = PlaceableAppearanceRowV1 {
        value: u32::from(append.report.appended_row_index),
    };
    validate_appended_placeable_row(&append.payload, appearance_row, label, model_resref)?;
    Ok(Placeable2daArtifactV1 {
        appearance_row,
        append,
    })
}

fn validate_appended_placeable_row(
    bytes: &[u8],
    row: PlaceableAppearanceRowV1,
    label: &str,
    model_resref: &str,
) -> Result<(), PlaceableErrorV1> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits)
        .map_err(|source| map_error("PLACEABLE-2DA-READBACK-FAILED", "placeables.2da", source))?;
    let readback = read_two_da_row_v2(bytes, row.value, &limits).map_err(|source| {
        map_error(
            "PLACEABLE-2DA-READBACK-FAILED",
            "placeables.2da.row",
            source,
        )
    })?;
    for (column, expected) in [
        ("Label", Some(label)),
        ("ModelName", Some(model_resref)),
        ("ShadowSize", Some("1")),
        ("BodyBag", Some("0")),
        ("Static", Some("1")),
        ("StrRef", None),
    ] {
        let index = inspection
            .columns
            .iter()
            .position(|candidate| candidate.eq_ignore_ascii_case(column))
            .ok_or_else(|| {
                error(
                    "PLACEABLE-2DA-READBACK-FAILED",
                    "placeables.2da.columns",
                    format!("missing {column} during readback"),
                )
            })?;
        let actual = &readback.cells[index];
        let matches = match (actual, expected) {
            (TwoDaCellValueV1::Null, None) => true,
            (TwoDaCellValueV1::Text { value }, Some(expected)) => value == expected,
            _ => false,
        };
        if !matches {
            return Err(error(
                "PLACEABLE-2DA-SEMANTIC-DIFF",
                format!("placeables.2da.rows[{}].{column}", row.value),
                "appended placeable cell differs from the requested contract",
            ));
        }
    }
    Ok(())
}

pub fn write_static_placeable_utp_v1(
    blueprint: &StaticPlaceableBlueprintV1,
) -> Result<GffArtifactV1, PlaceableErrorV1> {
    validate_blueprint(blueprint)?;
    let document = GffDocumentV1 {
        schema_version: 1,
        file_type: GffFileTypeV1::Utp,
        root: GffStructV1 {
            struct_id: u32::MAX,
            fields: static_placeable_common_fields(blueprint, true),
        },
    };
    let artifact = write_gff_v32(&document, &GffWriterOptionsV1::default())
        .map_err(|source| map_error("PLACEABLE-UTP-WRITE-FAILED", "utp", source))?;
    validate_static_placeable_utp_readback(&artifact.payload, blueprint)?;
    Ok(artifact)
}

/// Emits the conventional module-local `placeablepalcus.itp` custom palette.
///
/// Category IDs and string references follow the audited NWN placeable
/// palette manifest. Only the caller-owned NAME/RESREF leaf is new.
pub fn write_placeable_palette_itp_v1(
    blueprint: &StaticPlaceableBlueprintV1,
) -> Result<GffArtifactV1, PlaceableErrorV1> {
    validate_blueprint(blueprint)?;
    let mut main = standard_placeable_palette_categories();
    let entry = GffStructV1 {
        struct_id: 0,
        fields: vec![
            field(
                "NAME",
                GffValueV1::String(blueprint.display_name.as_bytes().to_vec()),
            ),
            field(
                "RESREF",
                GffValueV1::ResRef(blueprint.template_resref.clone()),
            ),
        ],
    };
    if !insert_palette_entry(&mut main, blueprint.palette_id, entry) {
        return Err(error(
            "PLACEABLE-PALETTE-ID-UNSUPPORTED",
            "blueprint.paletteId",
            format!(
                "PaletteID {} is absent from the audited NWN placeable palette",
                blueprint.palette_id
            ),
        ));
    }
    let document = GffDocumentV1 {
        schema_version: 1,
        file_type: GffFileTypeV1::Itp,
        root: GffStructV1 {
            struct_id: u32::MAX,
            fields: vec![field("MAIN", GffValueV1::List(main))],
        },
    };
    let artifact = write_gff_v32(&document, &GffWriterOptionsV1::default())
        .map_err(|source| map_error("PLACEABLE-ITP-WRITE-FAILED", "placeablepalcus.itp", source))?;
    validate_palette_itp_readback(&artifact.payload, blueprint)?;
    Ok(artifact)
}

fn standard_placeable_palette_categories() -> Vec<GffStructV1> {
    vec![
        palette_category(6783, Some(7), Vec::new()),
        palette_category(111_663, Some(22), Vec::new()),
        palette_category(6782, Some(6), Vec::new()),
        palette_category(6784, Some(8), Vec::new()),
        palette_category(1592, Some(9), Vec::new()),
        palette_category(6785, Some(10), Vec::new()),
        palette_category(6786, Some(11), Vec::new()),
        palette_category(6787, Some(12), Vec::new()),
        palette_category(9122, Some(14), Vec::new()),
        palette_category(9128, Some(15), Vec::new()),
        palette_category(
            6687,
            None,
            vec![
                palette_category(6688, Some(0), Vec::new()),
                palette_category(6689, Some(1), Vec::new()),
                palette_category(6690, Some(2), Vec::new()),
                palette_category(6691, Some(3), Vec::new()),
                palette_category(6692, Some(4), Vec::new()),
            ],
        ),
        palette_category(6788, Some(13), Vec::new()),
        palette_category(
            66_490,
            Some(16),
            vec![
                palette_category(62_485, Some(17), Vec::new()),
                palette_category(5836, Some(19), Vec::new()),
                palette_category(67_585, Some(21), Vec::new()),
                palette_category(53_151, Some(20), Vec::new()),
                palette_category(67_132, Some(18), Vec::new()),
            ],
        ),
        palette_category(6781, Some(5), Vec::new()),
    ]
}

fn palette_category(
    string_ref: u32,
    category_id: Option<u8>,
    children: Vec<GffStructV1>,
) -> GffStructV1 {
    let mut fields = vec![field("STRREF", GffValueV1::Dword(string_ref))];
    if let Some(category_id) = category_id {
        fields.push(field("ID", GffValueV1::Byte(category_id)));
    }
    if !children.is_empty() {
        fields.push(field("LIST", GffValueV1::List(children)));
    }
    GffStructV1 {
        struct_id: 0,
        fields,
    }
}

fn insert_palette_entry(
    structures: &mut [GffStructV1],
    palette_id: u8,
    entry: GffStructV1,
) -> bool {
    for structure in structures {
        if find_value(structure, "ID") == Some(&GffValueV1::Byte(palette_id)) {
            if let Some(list) =
                structure
                    .fields
                    .iter_mut()
                    .find_map(|field| match &mut field.value {
                        GffValueV1::List(list) if field.label == "LIST" => Some(list),
                        _ => None,
                    })
            {
                list.push(entry);
            } else {
                structure
                    .fields
                    .push(field("LIST", GffValueV1::List(vec![entry])));
            }
            return true;
        }
        for field in &mut structure.fields {
            if let GffValueV1::List(children) = &mut field.value
                && insert_palette_entry(children, palette_id, entry.clone())
            {
                return true;
            }
        }
    }
    false
}

fn validate_palette_itp_readback(
    bytes: &[u8],
    blueprint: &StaticPlaceableBlueprintV1,
) -> Result<(), PlaceableErrorV1> {
    let document = read_gff_v32(bytes, &GffLimitsV1::default()).map_err(|source| {
        map_error(
            "PLACEABLE-ITP-READBACK-FAILED",
            "placeablepalcus.itp",
            source,
        )
    })?;
    if document.file_type != GffFileTypeV1::Itp || document.root.struct_id != u32::MAX {
        return Err(error(
            "PLACEABLE-ITP-SEMANTIC-DIFF",
            "placeablepalcus.itp.header",
            "custom palette file type or root StructID differs",
        ));
    }
    let main = require_list(&document.root, "MAIN", "placeablepalcus.itp")?;
    let category = find_palette_category(main, blueprint.palette_id).ok_or_else(|| {
        error(
            "PLACEABLE-ITP-SEMANTIC-DIFF",
            "placeablepalcus.itp.MAIN",
            "requested palette category is absent after readback",
        )
    })?;
    let entries = require_list(category, "LIST", "placeablepalcus.itp.category")?;
    let matching = entries.iter().filter(|entry| {
        find_value(entry, "NAME")
            == Some(&GffValueV1::String(
                blueprint.display_name.as_bytes().to_vec(),
            ))
            && find_value(entry, "RESREF")
                == Some(&GffValueV1::ResRef(blueprint.template_resref.clone()))
    });
    if matching.count() != 1 {
        return Err(error(
            "PLACEABLE-ITP-SEMANTIC-DIFF",
            "placeablepalcus.itp.category.LIST",
            "custom palette must contain exactly one matching NAME/RESREF leaf",
        ));
    }
    Ok(())
}

fn find_palette_category(structures: &[GffStructV1], palette_id: u8) -> Option<&GffStructV1> {
    for structure in structures {
        if find_value(structure, "ID") == Some(&GffValueV1::Byte(palette_id)) {
            return Some(structure);
        }
        for field in &structure.fields {
            if let GffValueV1::List(children) = &field.value
                && let Some(found) = find_palette_category(children, palette_id)
            {
                return Some(found);
            }
        }
    }
    None
}

fn static_placeable_common_fields(
    blueprint: &StaticPlaceableBlueprintV1,
    include_blueprint_fields: bool,
) -> Vec<GffFieldV1> {
    let mut fields = vec![
        field(
            "Tag",
            GffValueV1::String(blueprint.object_tag.as_bytes().to_vec()),
        ),
        field("LocName", loc(&blueprint.display_name)),
        field("Description", empty_loc()),
        field(
            "TemplateResRef",
            GffValueV1::ResRef(blueprint.template_resref.clone()),
        ),
        field("AutoRemoveKey", GffValueV1::Byte(0)),
        field("CloseLockDC", GffValueV1::Byte(0)),
        field("Conversation", GffValueV1::ResRef(String::new())),
        field("Interruptable", GffValueV1::Byte(0)),
        field("Faction", GffValueV1::Dword(1)),
        field("Plot", GffValueV1::Byte(0)),
        field("KeyRequired", GffValueV1::Byte(0)),
        field("Lockable", GffValueV1::Byte(0)),
        field("Locked", GffValueV1::Byte(0)),
        field("OpenLockDC", GffValueV1::Byte(0)),
        field("PortraitId", GffValueV1::Word(0)),
        field("TrapDetectable", GffValueV1::Byte(0)),
        field("TrapDetectDC", GffValueV1::Byte(0)),
        field("TrapDisarmable", GffValueV1::Byte(0)),
        field("DisarmDC", GffValueV1::Byte(0)),
        field("TrapFlag", GffValueV1::Byte(0)),
        field("TrapOneShot", GffValueV1::Byte(0)),
        field("TrapType", GffValueV1::Byte(0)),
        field("KeyName", GffValueV1::String(Vec::new())),
        field("AnimationState", GffValueV1::Byte(0)),
        field(
            "Appearance",
            GffValueV1::Dword(blueprint.appearance_row.value),
        ),
        field("HP", GffValueV1::Short(10)),
        field("CurrentHP", GffValueV1::Short(10)),
        field("Hardness", GffValueV1::Byte(5)),
        field("Fort", GffValueV1::Byte(0)),
        field("Ref", GffValueV1::Byte(0)),
        field("Will", GffValueV1::Byte(0)),
    ];
    fields.extend(
        [
            "OnClosed",
            "OnDamaged",
            "OnDeath",
            "OnDisarm",
            "OnHeartbeat",
            "OnLock",
            "OnMeleeAttacked",
            "OnOpen",
            "OnSpellCastAt",
            "OnTrapTriggered",
            "OnUnlock",
            "OnUserDefined",
        ]
        .into_iter()
        .map(|label| field(label, GffValueV1::ResRef(String::new()))),
    );
    fields.extend([
        field("HasInventory", GffValueV1::Byte(0)),
        field("BodyBag", GffValueV1::Byte(0)),
        field("Static", GffValueV1::Byte(1)),
        field("Type", GffValueV1::Byte(0)),
        field("Useable", GffValueV1::Byte(0)),
        field("OnInvDisturbed", GffValueV1::ResRef(String::new())),
        field("OnUsed", GffValueV1::ResRef(String::new())),
    ]);
    if include_blueprint_fields {
        fields.extend([
            field("PaletteID", GffValueV1::Byte(blueprint.palette_id)),
            field(
                "Comment",
                GffValueV1::String(
                    b"Generated by Meshy2Aurora static placeable pipeline.".to_vec(),
                ),
            ),
        ]);
    }
    fields
}

fn validate_static_placeable_utp_readback(
    bytes: &[u8],
    expected: &StaticPlaceableBlueprintV1,
) -> Result<(), PlaceableErrorV1> {
    let document = read_gff_v32(bytes, &GffLimitsV1::default())
        .map_err(|source| map_error("PLACEABLE-UTP-READBACK-FAILED", "utp", source))?;
    if document.file_type != GffFileTypeV1::Utp || document.root.struct_id != u32::MAX {
        return Err(error(
            "PLACEABLE-UTP-SEMANTIC-DIFF",
            "utp.header",
            "UTP file type or root struct id differs",
        ));
    }
    if document.root.fields.len() != UTP_FIELD_MANIFEST.len() {
        return Err(error(
            "PLACEABLE-UTP-SEMANTIC-DIFF",
            "utp.fields",
            "UTP field count differs from the frozen retail manifest",
        ));
    }
    for (index, ((label, kind), actual)) in UTP_FIELD_MANIFEST
        .iter()
        .zip(&document.root.fields)
        .enumerate()
    {
        if actual.label != *label || gff_kind(&actual.value) != *kind {
            return Err(error(
                "PLACEABLE-UTP-SEMANTIC-DIFF",
                format!("utp.fields[{index}]"),
                format!(
                    "expected {label}:{kind}, got {}:{}",
                    actual.label,
                    gff_kind(&actual.value)
                ),
            ));
        }
    }
    require_value(
        &document.root,
        "TemplateResRef",
        &GffValueV1::ResRef(expected.template_resref.clone()),
        "utp",
    )?;
    require_value(
        &document.root,
        "Appearance",
        &GffValueV1::Dword(expected.appearance_row.value),
        "utp",
    )?;
    require_value(
        &document.root,
        "AnimationState",
        &GffValueV1::Byte(0),
        "utp",
    )?;
    require_value(&document.root, "Static", &GffValueV1::Byte(1), "utp")?;
    require_value(&document.root, "Useable", &GffValueV1::Byte(0), "utp")?;
    if document
        .root
        .fields
        .iter()
        .any(|item| item.label == "ItemList")
    {
        return Err(error(
            "PLACEABLE-UTP-SEMANTIC-DIFF",
            "utp.ItemList",
            "static HasInventory=0 blueprint must not contain ItemList",
        ));
    }
    Ok(())
}

fn build_static_placeable_git(
    blueprint: &StaticPlaceableBlueprintV1,
    placement: PlaceablePlacementV1,
) -> Result<Vec<u8>, PlaceableErrorV1> {
    validate_placement(placement)?;
    let base = binary_creature_multi_fixture_git(&[])
        .map_err(|source| map_error("PLACEABLE-GIT-BASE-FAILED", "git", source))?;
    let mut document = read_gff_v32(&base, &GffLimitsV1::default())
        .map_err(|source| map_error("PLACEABLE-GIT-BASE-FAILED", "git", source))?;
    let mut fields = static_placeable_common_fields(blueprint, false);
    fields.extend([
        field("X", GffValueV1::Float(placement.x)),
        field("Y", GffValueV1::Float(placement.y)),
        field("Z", GffValueV1::Float(placement.z)),
        field("Bearing", GffValueV1::Float(placement.bearing)),
    ]);
    replace_list(
        &mut document.root,
        "Placeable List",
        vec![GffStructV1 {
            struct_id: 9,
            fields,
        }],
    )?;
    let artifact = write_gff_v32(&document, &GffWriterOptionsV1::default())
        .map_err(|source| map_error("PLACEABLE-GIT-WRITE-FAILED", "git", source))?;
    validate_git_readback(&artifact.payload, blueprint, placement)?;
    Ok(artifact.payload)
}

fn build_static_placeable_gic(
    blueprint: &StaticPlaceableBlueprintV1,
    placement: PlaceablePlacementV1,
) -> Result<Vec<u8>, PlaceableErrorV1> {
    let base = binary_creature_multi_fixture_gic(&[])
        .map_err(|source| map_error("PLACEABLE-GIC-BASE-FAILED", "gic", source))?;
    let mut document = read_gff_v32(&base, &GffLimitsV1::default())
        .map_err(|source| map_error("PLACEABLE-GIC-BASE-FAILED", "gic", source))?;
    replace_list(
        &mut document.root,
        "Placeable List",
        vec![GffStructV1 {
            struct_id: 9,
            fields: vec![field(
                "Comment",
                GffValueV1::String(
                    format!(
                        "Placeable {} at ({}, {}, {}), bearing {}",
                        blueprint.object_tag,
                        placement.x,
                        placement.y,
                        placement.z,
                        placement.bearing
                    )
                    .into_bytes(),
                ),
            )],
        }],
    )?;
    let artifact = write_gff_v32(&document, &GffWriterOptionsV1::default())
        .map_err(|source| map_error("PLACEABLE-GIC-WRITE-FAILED", "gic", source))?;
    validate_gic_readback(&artifact.payload)?;
    Ok(artifact.payload)
}

/// Builds one static placeable directly from a Meshy-compatible GLB.
///
/// The GLB ingest, Profile A conversion and binary MDL writer are shared with
/// the creature route. This function adds only placeable resource resolution:
/// `placeables.2da`, UTP, GIT/GIC and deterministic HAK/MOD composition.
pub fn build_meshy_static_placeable_package_v1(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    let options = StaticPlaceableBuildOptionsV1::default();
    build_meshy_static_placeable_package_inner_v1(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        MeshyStaticPlaceableBuildContextV1 {
            authoring: None,
            collision: None,
            textures: None,
            material_separation: None,
            model_textures: None,
            exception: None,
            options: &options,
        },
    )
}

/// Returns the exact editable node/component inventory and its lossless default
/// authoring document after the same sanitizer used by the package build.
pub fn inspect_meshy_static_placeable_authoring_v1(
    source_glb: &[u8],
) -> Result<StaticPlaceableAuthoringBootstrapV1, PlaceableErrorV1> {
    inspect_meshy_static_placeable_authoring_v2(
        source_glb,
        &StaticPlaceableBuildOptionsV1::default(),
    )
}

pub fn inspect_meshy_static_placeable_authoring_v2(
    source_glb: &[u8],
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceableAuthoringBootstrapV1, PlaceableErrorV1> {
    inspect_meshy_static_placeable_authoring_inner_v1(source_glb, None, options)
}

pub fn inspect_meshy_static_placeable_authoring_v3(
    source_glb: &[u8],
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceableAuthoringBootstrapV2, PlaceableErrorV1> {
    let bootstrap = inspect_meshy_static_placeable_authoring_v2(source_glb, options)?;
    Ok(StaticPlaceableAuthoringBootstrapV2 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V2,
        inspection: bootstrap.inspection,
        document: PlaceableAuthoringDocumentV2 {
            schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V2,
            source_sha256: bootstrap.document.source_sha256,
            elements: bootstrap.document.elements,
            collision: PlaceableCollisionSpecV1::default(),
        },
    })
}

/// Returns the exact source material/image identities and the lossless default
/// texture-authoring document used by Studio.
pub fn inspect_meshy_static_placeable_textures_v1(
    source_glb: &[u8],
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<PlaceableTextureAuthoringBootstrapV1, PlaceableErrorV1> {
    validate_static_placeable_build_options_v1(options)?;
    let mut ingest =
        ingest_static_placeable_source_v1(source_glb, &static_placeable_glb_limits_v1())?;
    sanitize_static_placeable_ingest_v1(&mut ingest, options).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let model = convert_static_placeable_model_v1(
        &ingest,
        "m2a_texinspect",
        &static_placeable_profile_a_options_v1(),
        false,
    )?;
    inspect_placeable_material_textures_v1(&ingest, &model).map_err(map_texture_error)
}

/// Resolves preview metadata through the same Core recipe used by V5 build.
pub fn resolve_meshy_static_placeable_textures_v1(
    source_glb: &[u8],
    base_texture_resref: &str,
    geometry_authoring: &PlaceableAuthoringDocumentV2,
    texture_authoring: &PlaceableTextureAuthoringDocumentV1,
    payload_blob: &[u8],
    descriptors: &[PlaceableTexturePayloadDescriptorV1],
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<PlaceableTextureResolutionReportV1, PlaceableErrorV1> {
    validate_static_placeable_build_options_v1(options)?;
    validate_placeable_authoring_v2(geometry_authoring)?;
    let limits = static_placeable_glb_limits_v1();
    let mut ingest = ingest_static_placeable_source_v1(source_glb, &limits)?;
    sanitize_static_placeable_ingest_v1(&mut ingest, options).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let elements = geometry_authoring.elements_document_v1();
    let render_document =
        project_placeable_authoring_v1(&elements, PlaceableAuthoringProjectionV1::Render);
    apply_placeable_authoring_to_ingest_v1(&mut ingest, &render_document)
        .map_err(map_authoring_error)?;
    let model = convert_static_placeable_model_v1(
        &ingest,
        "m2a_texresolve",
        &static_placeable_profile_a_options_v1(),
        false,
    )?;
    resolve_placeable_texture_overrides_v1(
        source_glb,
        &limits,
        &ingest,
        &model,
        base_texture_resref,
        texture_authoring,
        payload_blob,
        descriptors,
    )
    .map(|resolved| resolved.report)
    .map_err(map_texture_error)
}

pub fn resolve_meshy_static_placeable_collision_v1(
    source_glb: &[u8],
    model_resref: &str,
    authoring: &PlaceableAuthoringDocumentV2,
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<ResolvedPlaceableCollisionV1, PlaceableErrorV1> {
    validate_static_placeable_build_options_v1(options)?;
    validate_placeable_authoring_v2(authoring)?;
    let mut render_ingest =
        ingest_static_placeable_source_v1(source_glb, &static_placeable_glb_limits_v1())?;
    sanitize_static_placeable_ingest_v1(&mut render_ingest, options).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let source = render_ingest.clone();
    let elements = authoring.elements_document_v1();
    let render_document =
        project_placeable_authoring_v1(&elements, PlaceableAuthoringProjectionV1::Render);
    let mut authoring_report =
        apply_placeable_authoring_to_ingest_v1(&mut render_ingest, &render_document)
            .map_err(map_authoring_error)?;
    authoring_report.authoring_sha256 =
        placeable_authoring_hash_v2(authoring).map_err(map_authoring_error)?;
    if authoring.collision.mode == PlaceableCollisionModeV1::AutoRectangle
        && authoring_report.collision_element_count == 0
    {
        return Err(error(
            "PLACEABLE-COLLISION-EMPTY",
            "authoring.elements",
            "AUTO_RECTANGLE requires at least one collision element",
        ));
    }
    let profile_options = static_placeable_profile_a_options_v1();
    let (render_model, render_transform) = convert_static_placeable_model_with_transform_v1(
        &render_ingest,
        model_resref,
        &profile_options,
        false,
    )?;
    let collision_model = if authoring.collision.mode == PlaceableCollisionModeV1::AutoRectangle {
        let mut collision_ingest = source;
        let collision_document =
            project_placeable_authoring_v1(&elements, PlaceableAuthoringProjectionV1::Collision);
        apply_placeable_authoring_to_ingest_v1(&mut collision_ingest, &collision_document)
            .map_err(map_authoring_error)?;
        convert_static_placeable_model_v1(&collision_ingest, model_resref, &profile_options, false)?
    } else {
        render_model
    };
    let (walkmesh, mut resolved) = resolve_placeable_collision_v1(
        &authoring.collision,
        model_resref,
        &authoring_report,
        &render_transform,
        &collision_model,
    )?;
    let artifact = write_ascii_placeable_walkmesh_v1(&walkmesh)
        .map_err(|source| map_error("PLACEABLE-PWK-WRITE-FAILED", "walkmesh", source))?;
    resolved.pwk_sha256 = artifact.report.payload_sha256;
    Ok(resolved)
}

/// Inspects one exact oversized source under a durable owner-approved
/// exception without changing the normal browser/product intake limits.
pub fn inspect_meshy_static_placeable_authoring_with_oversized_exception_v1(
    source_glb: &[u8],
    exception: &OwnerApprovedOversizedPlaceableV1,
) -> Result<StaticPlaceableAuthoringBootstrapV1, PlaceableErrorV1> {
    inspect_meshy_static_placeable_authoring_inner_v1(
        source_glb,
        Some(exception),
        &StaticPlaceableBuildOptionsV1::default(),
    )
}

fn inspect_meshy_static_placeable_authoring_inner_v1(
    source_glb: &[u8],
    exception: Option<&OwnerApprovedOversizedPlaceableV1>,
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceableAuthoringBootstrapV1, PlaceableErrorV1> {
    validate_static_placeable_build_options_v1(options)?;
    let limits = exception
        .map(|exception| oversized_placeable_glb_limits_v1(source_glb, exception))
        .transpose()?
        .unwrap_or_else(static_placeable_glb_limits_v1);
    let mut ingest = ingest_static_placeable_source_v1(source_glb, &limits)?;
    sanitize_static_placeable_ingest_v1(&mut ingest, options).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let inspection = inspect_placeable_elements_v1(&ingest.ir).map_err(map_authoring_error)?;
    let document = default_placeable_authoring_v1(&ingest.ir).map_err(map_authoring_error)?;
    Ok(StaticPlaceableAuthoringBootstrapV1 {
        schema_version: PLACEABLE_SCHEMA_VERSION,
        inspection,
        document,
    })
}

/// Builds a static placeable from the same immutable authoring document used by
/// the browser viewport. Render and collision projections share transforms but
/// independently honor their per-element participation flags.
pub fn build_meshy_static_placeable_package_v2(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    authoring: &PlaceableAuthoringDocumentV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    build_meshy_static_placeable_package_v3(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        authoring,
        &StaticPlaceableBuildOptionsV1::default(),
    )
}

pub fn build_meshy_static_placeable_package_v3(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    authoring: &PlaceableAuthoringDocumentV1,
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    build_meshy_static_placeable_package_inner_v1(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        MeshyStaticPlaceableBuildContextV1 {
            authoring: Some(authoring),
            collision: None,
            textures: None,
            material_separation: None,
            model_textures: None,
            exception: None,
            options,
        },
    )
}

pub fn build_meshy_static_placeable_package_v4(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    authoring: &PlaceableAuthoringDocumentV2,
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    validate_placeable_authoring_v2(authoring)?;
    let elements = authoring.elements_document_v1();
    build_meshy_static_placeable_package_inner_v1(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        MeshyStaticPlaceableBuildContextV1 {
            authoring: Some(&elements),
            collision: Some(&authoring.collision),
            textures: None,
            material_separation: None,
            model_textures: None,
            exception: None,
            options,
        },
    )
}

/// Builds an authored static placeable with exact, source-bound per-material
/// texture overrides. V4 and older routes remain byte-identical SOURCE paths.
#[allow(clippy::too_many_arguments)]
pub fn build_meshy_static_placeable_package_v5(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    authoring: &PlaceableAuthoringDocumentV2,
    texture_authoring: &PlaceableTextureAuthoringDocumentV1,
    texture_payload_blob: &[u8],
    texture_payload_descriptors: &[PlaceableTexturePayloadDescriptorV1],
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    validate_placeable_authoring_v2(authoring)?;
    let elements = authoring.elements_document_v1();
    build_meshy_static_placeable_package_inner_v1(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        MeshyStaticPlaceableBuildContextV1 {
            authoring: Some(&elements),
            collision: Some(&authoring.collision),
            textures: Some(MeshyStaticPlaceableTextureContextV1 {
                authoring: texture_authoring,
                payload_blob: texture_payload_blob,
                descriptors: texture_payload_descriptors,
            }),
            material_separation: None,
            model_textures: None,
            exception: None,
            options,
        },
    )
}

#[derive(Clone, Copy)]
struct MeshyStaticModelTextureContextV1<'a> {
    authoring: &'a ModelTextureAuthoringDocumentV1,
    payload_blob: &'a [u8],
    descriptors: &'a [ModelTexturePayloadDescriptorV1],
}

/// Material Separation vertical slice for static Placeables. Render geometry
/// receives authored material slots; collision authoring stays on the exact
/// geometry-only projection and therefore cannot be changed by the recipe.
#[allow(clippy::too_many_arguments)]
pub fn build_meshy_static_placeable_package_v6(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    authoring: &PlaceableAuthoringDocumentV2,
    material_separation: &ModelMaterialSeparationDocumentV1,
    texture_authoring: &ModelTextureAuthoringDocumentV1,
    texture_payload_blob: &[u8],
    texture_payload_descriptors: &[ModelTexturePayloadDescriptorV1],
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    validate_placeable_authoring_v2(authoring)?;
    let elements = authoring.elements_document_v1();
    build_meshy_static_placeable_package_inner_v1(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        MeshyStaticPlaceableBuildContextV1 {
            authoring: Some(&elements),
            collision: Some(&authoring.collision),
            textures: None,
            material_separation: Some(material_separation),
            model_textures: Some(MeshyStaticModelTextureContextV1 {
                authoring: texture_authoring,
                payload_blob: texture_payload_blob,
                descriptors: texture_payload_descriptors,
            }),
            exception: None,
            options,
        },
    )
}

fn validate_placeable_authoring_v2(
    authoring: &PlaceableAuthoringDocumentV2,
) -> Result<(), PlaceableErrorV1> {
    if authoring.schema_version != PLACEABLE_AUTHORING_SCHEMA_VERSION_V2 {
        return Err(error(
            "PLACEABLE-AUTHORING-SCHEMA-UNSUPPORTED",
            "authoring.schemaVersion",
            format!(
                "custom collision authoring requires schema version {}",
                PLACEABLE_AUTHORING_SCHEMA_VERSION_V2
            ),
        ));
    }
    let collision = &authoring.collision;
    if collision.schema_version != PLACEABLE_COLLISION_SPEC_SCHEMA_VERSION_V1 {
        return Err(error(
            "PLACEABLE-COLLISION-SCHEMA-UNSUPPORTED",
            "authoring.collision.schemaVersion",
            format!(
                "expected collision schema version {}",
                PLACEABLE_COLLISION_SPEC_SCHEMA_VERSION_V1
            ),
        ));
    }
    if !collision.padding_meters.is_finite() || collision.padding_meters < 0.0 {
        return Err(error(
            "PLACEABLE-COLLISION-PADDING-INVALID",
            "authoring.collision.paddingMeters",
            "collision padding must be a finite non-negative number of metres",
        ));
    }
    match collision.mode {
        PlaceableCollisionModeV1::AutoRectangle if !collision.vertices.is_empty() => Err(error(
            "PLACEABLE-COLLISION-AUTO-VERTICES-NOT-EMPTY",
            "authoring.collision.vertices",
            "AUTO_RECTANGLE derives its vertices and requires an empty custom vertex list",
        )),
        PlaceableCollisionModeV1::CustomPolygon if collision.padding_meters != 0.0 => Err(error(
            "PLACEABLE-COLLISION-CUSTOM-PADDING-UNSUPPORTED",
            "authoring.collision.paddingMeters",
            "CUSTOM_POLYGON V1 requires exact authored vertices and zero padding",
        )),
        _ => Ok(()),
    }
}

fn resolve_placeable_collision_v1(
    spec: &PlaceableCollisionSpecV1,
    model_resref: &str,
    authoring: &PlaceableAuthoringApplyReportV1,
    render_transform: &ProfileATransformReportV1,
    collision_model: &AuroraModelIrV1,
) -> Result<(PlaceableWalkmeshIrV1, ResolvedPlaceableCollisionV1), PlaceableErrorV1> {
    let input_vertices = match spec.mode {
        PlaceableCollisionModeV1::AutoRectangle => {
            let min = authoring.collision_bounds_min.ok_or_else(|| {
                error(
                    "PLACEABLE-COLLISION-EMPTY",
                    "authoring.elements",
                    "AUTO_RECTANGLE requires finite collision element bounds",
                )
            })?;
            let max = authoring.collision_bounds_max.ok_or_else(|| {
                error(
                    "PLACEABLE-COLLISION-EMPTY",
                    "authoring.elements",
                    "AUTO_RECTANGLE requires finite collision element bounds",
                )
            })?;
            let source_padding = if spec.padding_meters == 0.0 {
                0.0
            } else {
                let scale = render_transform.scale.ok_or_else(|| {
                    error(
                        "PLACEABLE-COLLISION-TRANSFORM-MISSING",
                        "conversion.report.transform.scale",
                        "AUTO_RECTANGLE padding requires the resolved uniform output scale",
                    )
                })?;
                spec.padding_meters / scale.abs()
            };
            vec![
                [min[0] - source_padding, min[2] - source_padding],
                [max[0] + source_padding, min[2] - source_padding],
                [max[0] + source_padding, max[2] + source_padding],
                [min[0] - source_padding, max[2] + source_padding],
            ]
        }
        PlaceableCollisionModeV1::CustomPolygon => spec.vertices.clone(),
    };

    let walkmesh = match spec.mode {
        PlaceableCollisionModeV1::AutoRectangle => {
            let mut walkmesh = derive_placeable_walkmesh_ir_v1(collision_model, model_resref)
                .map_err(|source| map_error("PLACEABLE-PWK-RESOLVE-FAILED", "collision", source))?;
            if spec.padding_meters > 0.0 {
                walkmesh.bounds_min[0] -= spec.padding_meters;
                walkmesh.bounds_min[1] -= spec.padding_meters;
                walkmesh.bounds_max[0] += spec.padding_meters;
                walkmesh.bounds_max[1] += spec.padding_meters;
                walkmesh.vertices = vec![
                    [walkmesh.bounds_min[0], walkmesh.bounds_min[1], 0.0],
                    [walkmesh.bounds_max[0], walkmesh.bounds_min[1], 0.0],
                    [walkmesh.bounds_min[0], walkmesh.bounds_max[1], 0.0],
                    [walkmesh.bounds_max[0], walkmesh.bounds_max[1], 0.0],
                ];
            }
            walkmesh
        }
        PlaceableCollisionModeV1::CustomPolygon => {
            let output_vertices = spec
                .vertices
                .iter()
                .copied()
                .map(|vertex| source_xz_to_aurora_xy_v1(vertex, render_transform))
                .collect::<Result<Vec<_>, _>>()?;
            custom_placeable_walkmesh_ir_v1(model_resref, &output_vertices).map_err(|source| {
                map_error("PLACEABLE-PWK-CUSTOM-POLYGON-INVALID", "collision", source)
            })?
        }
    };
    let vertices = walkmesh
        .vertices
        .iter()
        .map(|vertex| [vertex[0], vertex[1]])
        .collect::<Vec<_>>();
    let source_vertices = vertices
        .iter()
        .copied()
        .map(|vertex| aurora_xy_to_source_xz_v1(vertex, render_transform))
        .collect::<Result<Vec<_>, _>>()?;
    let triangles = walkmesh
        .faces
        .iter()
        .map(|face| face.vertex_indices)
        .collect::<Vec<_>>();
    let collision_bytes = serde_json::to_vec(&(
        spec.mode,
        &vertices,
        &triangles,
        PLACEABLE_PWK_NONWALK_SURFACE_ID_V1,
    ))
    .map_err(|source| {
        error(
            "PLACEABLE-COLLISION-SERIALIZE",
            "collision",
            source.to_string(),
        )
    })?;
    let report = ResolvedPlaceableCollisionV1 {
        schema_version: 1,
        mode: spec.mode,
        source_coordinate_space: spec.coordinate_space,
        output_coordinate_space: "AURORA_XY_METERS".to_owned(),
        input_vertices,
        source_vertices,
        vertices,
        triangles,
        bounds_min: walkmesh.bounds_min,
        bounds_max: walkmesh.bounds_max,
        surface_id: PLACEABLE_PWK_NONWALK_SURFACE_ID_V1,
        authoring_sha256: authoring.authoring_sha256.clone(),
        collision_sha256: sha256_hex(&collision_bytes),
        pwk_sha256: String::new(),
    };
    Ok((walkmesh, report))
}

fn source_xz_to_aurora_xy_v1(
    vertex: [f32; 2],
    transform: &ProfileATransformReportV1,
) -> Result<[f32; 2], PlaceableErrorV1> {
    let scale = transform.scale.ok_or_else(|| {
        error(
            "PLACEABLE-COLLISION-TRANSFORM-MISSING",
            "conversion.report.transform.scale",
            "eligible Placeable conversion must resolve one uniform scale",
        )
    })?;
    let translation = transform.translation.ok_or_else(|| {
        error(
            "PLACEABLE-COLLISION-TRANSFORM-MISSING",
            "conversion.report.transform.translation",
            "eligible Placeable conversion must resolve one translation",
        )
    })?;
    let matrix = transform.basis_matrix;
    let source = [vertex[0], 0.0, vertex[1]];
    let output = [
        scale * (matrix[0] * source[0] + matrix[4] * source[1] + matrix[8] * source[2])
            + translation[0],
        scale * (matrix[1] * source[0] + matrix[5] * source[1] + matrix[9] * source[2])
            + translation[1],
    ];
    if output.iter().any(|coordinate| !coordinate.is_finite()) {
        return Err(error(
            "PLACEABLE-COLLISION-TRANSFORM-INVALID",
            "authoring.collision.vertices",
            "source XZ to Aurora XY transform produced a non-finite coordinate",
        ));
    }
    Ok(output)
}

fn aurora_xy_to_source_xz_v1(
    vertex: [f32; 2],
    transform: &ProfileATransformReportV1,
) -> Result<[f32; 2], PlaceableErrorV1> {
    let scale = transform.scale.ok_or_else(|| {
        error(
            "PLACEABLE-COLLISION-TRANSFORM-MISSING",
            "conversion.report.transform.scale",
            "eligible Placeable conversion must resolve one uniform scale",
        )
    })?;
    let translation = transform.translation.ok_or_else(|| {
        error(
            "PLACEABLE-COLLISION-TRANSFORM-MISSING",
            "conversion.report.transform.translation",
            "eligible Placeable conversion must resolve one translation",
        )
    })?;
    let matrix = transform.basis_matrix;
    let a = matrix[0];
    let b = matrix[8];
    let c = matrix[1];
    let d = matrix[9];
    let determinant = a * d - b * c;
    if !scale.is_finite() || scale.abs() <= f32::EPSILON || determinant.abs() <= f32::EPSILON {
        return Err(error(
            "PLACEABLE-COLLISION-TRANSFORM-INVALID",
            "conversion.report.transform",
            "collision ground-plane transform must be finite and invertible",
        ));
    }
    let output_x = (vertex[0] - translation[0]) / scale;
    let output_y = (vertex[1] - translation[1]) / scale;
    let source = [
        (d * output_x - b * output_y) / determinant,
        (-c * output_x + a * output_y) / determinant,
    ];
    if source.iter().any(|coordinate| !coordinate.is_finite()) {
        return Err(error(
            "PLACEABLE-COLLISION-TRANSFORM-INVALID",
            "collision.vertices",
            "inverse Profile A transform produced a non-finite source coordinate",
        ));
    }
    Ok(source)
}

/// Builds one exact oversized Placeable under an explicit owner-approved,
/// source-bound exception. The default V1/V2 product routes remain unchanged.
pub fn build_meshy_static_placeable_package_with_oversized_exception_v3(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    authoring: &PlaceableAuthoringDocumentV1,
    exception: &OwnerApprovedOversizedPlaceableV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    let options = StaticPlaceableBuildOptionsV1::default();
    build_meshy_static_placeable_package_inner_v1(
        source_glb,
        base_placeables_2da,
        identity,
        placement,
        palette_id,
        MeshyStaticPlaceableBuildContextV1 {
            authoring: Some(authoring),
            collision: None,
            textures: None,
            material_separation: None,
            model_textures: None,
            exception: Some(exception),
            options: &options,
        },
    )
}

fn ingest_static_placeable_source_v1(
    source_glb: &[u8],
    limits: &GlbLimits,
) -> Result<crate::glb::GlbIngestResult, PlaceableErrorV1> {
    ingest_glb(source_glb, limits).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            source.message,
        )
    })
}

fn validate_static_placeable_build_options_v1(
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<(), PlaceableErrorV1> {
    if options.schema_version != STATIC_PLACEABLE_BUILD_OPTIONS_SCHEMA_VERSION {
        return Err(error(
            "PLACEABLE-BUILD-OPTIONS-SCHEMA-INVALID",
            "options.schemaVersion",
            format!(
                "expected Placeable build-options schema {}, got {}",
                STATIC_PLACEABLE_BUILD_OPTIONS_SCHEMA_VERSION, options.schema_version
            ),
        ));
    }
    Ok(())
}

fn sanitize_static_placeable_ingest_v1(
    ingest: &mut crate::glb::GlbIngestResult,
    options: &StaticPlaceableBuildOptionsV1,
) -> Result<(), crate::model_pipeline::M6PipelineErrorV1> {
    if options.experimental_aggressive_geometry_cleanup {
        sanitize_static_model_degenerate_triangles_aggressive_v1(ingest)
    } else {
        sanitize_static_model_degenerate_triangles_v1(ingest)
    }
}

fn map_authoring_error(
    source: crate::placeable_authoring::PlaceableAuthoringErrorV1,
) -> PlaceableErrorV1 {
    error(&source.code, source.path, source.message)
}

fn map_texture_error(source: PlaceableTextureErrorV1) -> PlaceableErrorV1 {
    error(&source.code, source.path, source.message)
}

#[derive(Clone, Copy)]
struct MeshyStaticPlaceableTextureContextV1<'a> {
    authoring: &'a PlaceableTextureAuthoringDocumentV1,
    payload_blob: &'a [u8],
    descriptors: &'a [PlaceableTexturePayloadDescriptorV1],
}

struct MeshyStaticPlaceableBuildContextV1<'a> {
    authoring: Option<&'a PlaceableAuthoringDocumentV1>,
    collision: Option<&'a PlaceableCollisionSpecV1>,
    textures: Option<MeshyStaticPlaceableTextureContextV1<'a>>,
    material_separation: Option<&'a ModelMaterialSeparationDocumentV1>,
    model_textures: Option<MeshyStaticModelTextureContextV1<'a>>,
    exception: Option<&'a OwnerApprovedOversizedPlaceableV1>,
    options: &'a StaticPlaceableBuildOptionsV1,
}

fn build_meshy_static_placeable_package_inner_v1(
    source_glb: &[u8],
    base_placeables_2da: &[u8],
    identity: &StaticPlaceableIdentityV1,
    placement: PlaceablePlacementV1,
    palette_id: u8,
    context: MeshyStaticPlaceableBuildContextV1<'_>,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    let MeshyStaticPlaceableBuildContextV1 {
        authoring,
        collision,
        textures: texture_context,
        material_separation,
        model_textures,
        exception,
        options,
    } = context;
    validate_static_placeable_build_options_v1(options)?;
    validate_resref(&identity.model_resref, "identity.modelResref")?;
    validate_resref(&identity.texture_resref, "identity.textureResref")?;

    let glb_limits = exception
        .map(|exception| oversized_placeable_glb_limits_v1(source_glb, exception))
        .transpose()?
        .unwrap_or_else(static_placeable_glb_limits_v1);
    let mut ingest = ingest_static_placeable_source_v1(source_glb, &glb_limits)?;
    sanitize_static_placeable_ingest_v1(&mut ingest, options).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let source_ingest = ingest.clone();
    let resolved_materials = material_separation
        .map(|document| {
            resolve_model_materials_v1(&source_ingest.ir, document).map_err(|source| {
                error(
                    &source.code.replacen(
                        "MATERIAL-SEPARATION",
                        "PLACEABLE-MATERIAL-SEPARATION",
                        1,
                    ),
                    source.path,
                    source.message,
                )
            })
        })
        .transpose()?;
    let mut authoring_report = None;
    let collision_ingest = if let Some(document) = authoring {
        let source = ingest.clone();
        let render_document =
            project_placeable_authoring_v1(document, PlaceableAuthoringProjectionV1::Render);
        let mut report = if let Some(separation) = material_separation {
            apply_placeable_authoring_with_material_separation_to_ingest_v1(
                &mut ingest,
                &render_document,
                separation,
            )
        } else {
            apply_placeable_authoring_to_ingest_v1(&mut ingest, &render_document)
        }
        .map_err(map_authoring_error)?;
        let custom_polygon =
            collision.is_some_and(|spec| spec.mode == PlaceableCollisionModeV1::CustomPolygon);
        if report.collision_element_count == 0 && !custom_polygon {
            return Err(error(
                "PLACEABLE-COLLISION-EMPTY",
                "authoring.elements",
                "static placeable requires at least one non-deleted source element included in collision",
            ));
        }
        if let Some(spec) = collision {
            let v2 = PlaceableAuthoringDocumentV2 {
                schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V2,
                source_sha256: document.source_sha256.clone(),
                elements: document.elements.clone(),
                collision: spec.clone(),
            };
            report.authoring_sha256 =
                placeable_authoring_hash_v2(&v2).map_err(map_authoring_error)?;
        }
        authoring_report = Some(report);
        let mut collision = source;
        let collision_document =
            project_placeable_authoring_v1(document, PlaceableAuthoringProjectionV1::Collision);
        apply_placeable_authoring_to_ingest_v1(&mut collision, &collision_document)
            .map_err(map_authoring_error)?;
        Some(collision)
    } else {
        None
    };

    let profile_options = exception
        .map(oversized_placeable_profile_a_options_v1)
        .unwrap_or_else(static_placeable_profile_a_options_v1);
    let (model, render_transform) = convert_static_placeable_model_with_transform_v1(
        &ingest,
        &identity.model_resref,
        &profile_options,
        exception.is_some(),
    )?;
    let collision_model = collision_ingest
        .as_ref()
        .map(|source| {
            convert_static_placeable_model_v1(
                source,
                &identity.model_resref,
                &profile_options,
                exception.is_some(),
            )
        })
        .transpose()?;
    let resolved_collision = collision
        .map(|spec| {
            resolve_placeable_collision_v1(
                spec,
                &identity.model_resref,
                authoring_report.as_ref().ok_or_else(|| {
                    error(
                        "PLACEABLE-COLLISION-AUTHORING-MISSING",
                        "authoring",
                        "versioned collision requires an authoring report",
                    )
                })?,
                &render_transform,
                collision_model.as_ref().unwrap_or(&model),
            )
        })
        .transpose()?;

    let (material_textures, textures, texture_authoring_report, model_texture_report) =
        if let Some(texture_context) = model_textures {
            let resolved_materials = resolved_materials.as_ref().ok_or_else(|| {
                error(
                    "PLACEABLE-MATERIAL-SEPARATION-MISSING",
                    "materialSeparation",
                    "neutral model texture authoring requires Material Separation",
                )
            })?;
            let resolved = resolve_model_texture_authoring_v1(
                source_glb,
                &glb_limits,
                &source_ingest,
                resolved_materials,
                &identity.texture_resref,
                texture_context.authoring,
                texture_context.payload_blob,
                texture_context.descriptors,
            )
            .map_err(|source| error(&source.code, source.path, source.message))?;
            let mut resref_by_material_name = BTreeMap::<String, String>::new();
            for binding in &resolved.report.bindings {
                resref_by_material_name.insert(
                    binding.authored_material_id.clone(),
                    binding.output_resref.clone(),
                );
            }
            for slot in &resolved_materials.report.material_slots {
                if slot.system_source_material
                    && let Some(name) = &slot.source_material_name
                    && let Some(resref) = resref_by_material_name
                        .get(&slot.authored_material_id)
                        .cloned()
                {
                    resref_by_material_name.insert(name.clone(), resref);
                }
            }
            let material_textures = model
            .material_source_bindings
            .iter()
            .map(|binding| {
                let name = binding
                    .source_material_id
                    .and_then(|source_id| {
                        ingest
                            .ir
                            .materials
                            .iter()
                            .find(|material| material.id == source_id)
                    })
                    .and_then(|material| material.name.as_deref())
                    .or(binding.source_material_name.as_deref())
                    .ok_or_else(|| {
                        error(
                            "PLACEABLE-MATERIAL-SEPARATION-BINDING-MISSING",
                            "model.materialSourceBindings.sourceMaterialName",
                            format!(
                                "material-separated Placeable output slot {} (source material {:?}) has an unnamed material",
                                binding.slot, binding.source_material_id
                            ),
                        )
                    })?;
                    let base_name = name
                        .strip_suffix(SHADOWLESS_MATERIAL_SUFFIX_V1)
                        .unwrap_or(name);
                    let resref = resref_by_material_name.get(base_name).ok_or_else(|| {
                        error(
                            "PLACEABLE-MATERIAL-SEPARATION-BINDING-MISSING",
                            "model.materialSourceBindings",
                            format!("output material {name} has no authored texture binding"),
                        )
                    })?;
                    Ok(MdlMaterialTextureBindingV1 {
                        material_slot: binding.slot,
                        resref: resref.clone(),
                    })
                })
                .collect::<Result<Vec<_>, PlaceableErrorV1>>()?;
            let report = resolved.report;
            let textures = resolved
                .textures
                .into_iter()
                .map(|texture| PlaceableTextureInputV1 {
                    resref: texture.resref,
                    resource_type: texture.resource_type,
                    payload: texture.payload,
                })
                .collect();
            (material_textures, textures, None, Some(report))
        } else if let Some(texture_context) = texture_context {
            let resolved = resolve_placeable_texture_overrides_v1(
                source_glb,
                &glb_limits,
                &ingest,
                &model,
                &identity.texture_resref,
                texture_context.authoring,
                texture_context.payload_blob,
                texture_context.descriptors,
            )
            .map_err(map_texture_error)?;
            let report = resolved.report;
            let textures = resolved
                .textures
                .into_iter()
                .map(|texture| PlaceableTextureInputV1 {
                    resref: texture.resref,
                    resource_type: texture.resource_type,
                    payload: texture.payload,
                })
                .collect();
            (resolved.material_textures, textures, Some(report), None)
        } else {
            let texture_selections =
                resolve_base_color_image_indices_v1(&ingest, &model).map_err(|source| {
                    error(
                        &format!("PLACEABLE-{}", source.code),
                        source.path,
                        source.message,
                    )
                })?;
            let mut resref_by_image_sha256 = BTreeMap::<String, String>::new();
            let mut material_textures = Vec::with_capacity(texture_selections.len());
            let mut textures = Vec::new();
            for selection in texture_selections {
                let resref = if let Some(resref) =
                    resref_by_image_sha256.get(&selection.source_image_sha256)
                {
                    resref.clone()
                } else {
                    let resref = derive_material_texture_resref_v1(
                        &identity.texture_resref,
                        textures.len(),
                    )?;
                    let texture_image = decode_embedded_image_to_tga_v1(
                        source_glb,
                        selection.source_image_index,
                        &glb_limits,
                        &EmbeddedImageDecodeLimitsV1::default(),
                    )
                    .map_err(|source| {
                        error(
                            &format!("PLACEABLE-{}", source.code),
                            source.json_path.unwrap_or_else(|| "images".to_owned()),
                            source.message,
                        )
                    })?;
                    let tga = write_tga_v1(&texture_image, &TgaWriterOptionsV1::default())
                        .map_err(|source| {
                            error(
                                &format!("PLACEABLE-{}", source.code),
                                source.path,
                                source.message,
                            )
                        })?;
                    textures.push(PlaceableTextureInputV1 {
                        resref: resref.clone(),
                        resource_type: TGA_RESOURCE_TYPE,
                        payload: tga.payload,
                    });
                    resref_by_image_sha256.insert(selection.source_image_sha256, resref.clone());
                    resref
                };
                material_textures.push(MdlMaterialTextureBindingV1 {
                    material_slot: selection.material_slot,
                    resref,
                });
            }
            (material_textures, textures, None, None)
        };

    build_static_placeable_package_inner_v1(
        &StaticPlaceableBuildRequestV1 {
            schema_version: PLACEABLE_SCHEMA_VERSION,
            identity: identity.clone(),
            placement,
            palette_id,
            base_placeables_2da: base_placeables_2da.to_vec(),
            model,
            collision_model,
            authoring_report,
            material_textures,
            textures,
        },
        exception,
        options,
        resolved_collision,
        texture_authoring_report,
        resolved_materials.map(|materials| materials.report),
        model_texture_report,
    )
}

fn derive_material_texture_resref_v1(
    base_resref: &str,
    unique_image_index: usize,
) -> Result<String, PlaceableErrorV1> {
    if unique_image_index == 0 {
        return Ok(base_resref.to_owned());
    }
    let suffix = format!("_m{unique_image_index}");
    let prefix_len = 16usize.checked_sub(suffix.len()).ok_or_else(|| {
        error(
            "PLACEABLE-TEXTURE-RESREF-EXHAUSTED",
            "identity.textureResref",
            "material texture suffix cannot fit the Aurora resref limit",
        )
    })?;
    if prefix_len == 0 {
        return Err(error(
            "PLACEABLE-TEXTURE-RESREF-EXHAUSTED",
            "identity.textureResref",
            "material texture suffix leaves no base resref bytes",
        ));
    }
    Ok(format!(
        "{}{}",
        &base_resref[..base_resref.len().min(prefix_len)],
        suffix
    ))
}

fn convert_static_placeable_model_v1(
    ingest: &crate::glb::GlbIngestResult,
    model_resref: &str,
    profile_options: &ProfileAOptionsV1,
    owner_approved_oversized: bool,
) -> Result<AuroraModelIrV1, PlaceableErrorV1> {
    convert_static_placeable_model_with_transform_v1(
        ingest,
        model_resref,
        profile_options,
        owner_approved_oversized,
    )
    .map(|(model, _)| model)
}

fn convert_static_placeable_model_with_transform_v1(
    ingest: &crate::glb::GlbIngestResult,
    model_resref: &str,
    profile_options: &ProfileAOptionsV1,
    owner_approved_oversized: bool,
) -> Result<(AuroraModelIrV1, ProfileATransformReportV1), PlaceableErrorV1> {
    let rig = derive_meshy_m0_static_rigid_profile_v1(ingest).map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let conversion = if owner_approved_oversized {
        convert_profile_a_owner_approved_oversized_v1(ingest, &rig, profile_options)
    } else {
        convert_profile_a(ingest, &rig, profile_options)
    }
    .map_err(|source| {
        error(
            &format!("PLACEABLE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    if !conversion.report.conversion_eligible {
        let blocking_gates = conversion
            .report
            .gates
            .iter()
            .filter(|gate| gate.severity == "BLOCKING")
            .map(|gate| gate.code.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(error(
            "PLACEABLE-PROFILE-INELIGIBLE",
            "conversion.report",
            format!(
                "static Profile A conversion did not produce an eligible common model IR; blocking gates: {blocking_gates}"
            ),
        ));
    }
    let transform = conversion.report.transform.clone();
    let mut model = conversion.creature.ok_or_else(|| {
        error(
            "PLACEABLE-PROFILE-INELIGIBLE",
            "conversion.model",
            "eligible static Profile A conversion has no common model IR",
        )
    })?;
    let shadowless_slots = model
        .material_source_bindings
        .iter()
        .filter(|binding| {
            binding
                .source_material_name
                .as_deref()
                .is_some_and(|name| name.ends_with(SHADOWLESS_MATERIAL_SUFFIX_V1))
        })
        .map(|binding| binding.slot)
        .collect::<std::collections::BTreeSet<_>>();
    for segment in &mut model.segments {
        segment.cast_shadow = !shadowless_slots.contains(&segment.material_slot);
    }
    normalize_static_model_root(&mut model, model_resref)?;
    Ok((model, transform))
}

fn normalize_static_model_root(
    model: &mut AuroraModelIrV1,
    model_resref: &str,
) -> Result<(), PlaceableErrorV1> {
    let roots = model
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(error(
            "PLACEABLE-HIERARCHY-INVALID",
            "model.nodes",
            "static placeable requires exactly one geometry root",
        ));
    }
    model.nodes[roots[0]].name = model_resref.to_owned();
    Ok(())
}

pub fn build_static_placeable_package_v1(
    request: &StaticPlaceableBuildRequestV1,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    build_static_placeable_package_inner_v1(
        request,
        None,
        &StaticPlaceableBuildOptionsV1::default(),
        None,
        None,
        None,
        None,
    )
}

fn build_static_placeable_package_inner_v1(
    request: &StaticPlaceableBuildRequestV1,
    exception: Option<&OwnerApprovedOversizedPlaceableV1>,
    options: &StaticPlaceableBuildOptionsV1,
    resolved_collision: Option<(PlaceableWalkmeshIrV1, ResolvedPlaceableCollisionV1)>,
    texture_authoring_report: Option<PlaceableTextureResolutionReportV1>,
    material_separation_report: Option<ModelMaterialSeparationReportV1>,
    model_texture_authoring_report: Option<ModelTextureResolutionReportV1>,
) -> Result<StaticPlaceablePackageArtifactV1, PlaceableErrorV1> {
    validate_static_placeable_build_options_v1(options)?;
    validate_request(request, exception)?;
    let mut render_model = request.model.clone();
    segment_model_for_binary_mdl_v1(&mut render_model)
        .map_err(|source| map_error("PLACEABLE-MODEL-SEGMENTATION-FAILED", "model", source))?;
    let identity_texture = request
        .textures
        .iter()
        .find(|texture| {
            texture
                .resref
                .eq_ignore_ascii_case(&request.identity.texture_resref)
        })
        .ok_or_else(|| {
            error(
                "PLACEABLE-IDENTITY-TEXTURE-MISSING",
                "identity.textureResref",
                "validated identity texture payload is missing",
            )
        })?;
    let two_da = append_static_placeable_2da_v1(
        &request.base_placeables_2da,
        &request.identity.object_tag.to_ascii_uppercase(),
        &request.identity.model_resref,
    )?;
    let blueprint = StaticPlaceableBlueprintV1 {
        schema_version: 1,
        template_resref: request.identity.blueprint_resref.clone(),
        object_tag: request.identity.object_tag.clone(),
        display_name: request.identity.display_name.clone(),
        appearance_row: two_da.appearance_row,
        palette_id: request.palette_id,
    };
    let utp = write_static_placeable_utp_v1(&blueprint)?;
    let itp = write_placeable_palette_itp_v1(&blueprint)?;
    let git = build_static_placeable_git(&blueprint, request.placement)?;
    let gic = build_static_placeable_gic(&blueprint, request.placement)?;
    let mdl_options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::PlaceableStaticRigidNativeV1,
        // The profile is inert because static placeables emit no local
        // animations. Keeping the existing enum avoids a second model
        // writer or a fake placeable-only animation pipeline.
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: request.identity.model_resref.clone(),
        diffuse_texture_resref_by_material_slot: request.material_textures.clone(),
    };
    let mdl = if options.experimental_aggressive_geometry_cleanup {
        write_binary_mdl(&render_model, &mdl_options)
    } else if let Some(exception) = exception {
        let max_input_bytes = usize::try_from(exception.max_hak_output_bytes).map_err(|_| {
            error(
                "PLACEABLE-OVERSIZED-MDL-READBACK-LIMIT-INVALID",
                "exception.maxHakOutputBytes",
                "owner-approved HAK byte limit does not fit the host parser",
            )
        })?;
        write_binary_mdl_exact_face_planes_with_readback_limits_v1(
            &render_model,
            &mdl_options,
            &ParserLimits {
                max_input_bytes,
                ..ParserLimits::default()
            },
        )
    } else {
        write_binary_mdl_exact_face_planes_v1(&render_model, &mdl_options)
    }
    .map_err(|source| map_error("PLACEABLE-MDL-WRITE-FAILED", "model", source))?;
    let (pwk, mut resolved_collision_report) = if let Some((walkmesh, report)) = resolved_collision
    {
        (
            write_ascii_placeable_walkmesh_v1(&walkmesh)
                .map_err(|source| map_error("PLACEABLE-PWK-WRITE-FAILED", "walkmesh", source))?,
            Some(report),
        )
    } else {
        let collision_model = request.collision_model.as_ref().unwrap_or(&request.model);
        (
            write_placeable_walkmesh_v1(collision_model, &request.identity.model_resref)
                .map_err(|source| map_error("PLACEABLE-PWK-WRITE-FAILED", "walkmesh", source))?,
            None,
        )
    };
    if let Some(report) = &mut resolved_collision_report {
        report.pwk_sha256 = pwk.report.payload_sha256.clone();
    }

    let mut hak_resources = Vec::with_capacity(3 + request.textures.len());
    hak_resources.push(resource(
        "placeables",
        PLACEABLES_2DA_RESOURCE_TYPE,
        two_da.append.payload.clone(),
    ));
    hak_resources.push(resource(
        &request.identity.model_resref,
        MDL_RESOURCE_TYPE,
        mdl.payload.clone(),
    ));
    hak_resources.push(resource(
        &request.identity.model_resref,
        PWK_RESOURCE_TYPE,
        pwk.payload.clone(),
    ));
    hak_resources.extend(request.textures.iter().map(|texture| {
        resource(
            &texture.resref,
            texture.resource_type,
            texture.payload.clone(),
        )
    }));
    let hak_options = exception.map_or_else(HakWriterOptionsV1::default, |exception| {
        HakWriterOptionsV1 {
            schema_version: 1,
            limits: HakWriterLimitsV1 {
                max_entry_count: HakWriterLimitsV1::default().max_entry_count,
                max_output_bytes: exception.max_hak_output_bytes,
            },
        }
    });
    let hak = write_hak_v1(&hak_resources, &hak_options)
        .map_err(|source| map_error("PLACEABLE-HAK-WRITE-FAILED", "hak", source))?;

    let ifo = binary_m0_module_ifo_for(
        &request.identity.module_resref,
        &request.identity.area_resref,
        &[request.identity.hak_resref.as_str()],
        &request.identity.module_display_name,
        "Generated by Meshy2Aurora for owner-run static placeable proof.",
    )
    .map_err(|source| map_error("PLACEABLE-IFO-WRITE-FAILED", "module.ifo", source))?;
    let are = binary_m0_area_for_named(&request.identity.area_resref, &request.identity.area_name)
        .map_err(|source| map_error("PLACEABLE-ARE-WRITE-FAILED", "area.are", source))?;
    let fac = proof_factions()
        .map_err(|source| map_error("PLACEABLE-FAC-WRITE-FAILED", "repute.fac", source))?;
    let module_resources = vec![
        resource("module", IFO_RESOURCE_TYPE, ifo),
        resource("repute", FAC_RESOURCE_TYPE, fac),
        resource(&request.identity.area_resref, ARE_RESOURCE_TYPE, are),
        resource(
            &request.identity.area_resref,
            GIC_RESOURCE_TYPE,
            gic.clone(),
        ),
        resource(
            &request.identity.area_resref,
            GIT_RESOURCE_TYPE,
            git.clone(),
        ),
        resource(
            &request.identity.blueprint_resref,
            UTP_RESOURCE_TYPE,
            utp.payload.clone(),
        ),
        resource("placeablepalcus", ITP_RESOURCE_TYPE, itp.payload.clone()),
    ];
    let module = write_erf_archive_v1(
        ErfFileType::Module,
        &module_resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| map_error("PLACEABLE-MOD-WRITE-FAILED", "module", source))?;

    validate_package_readback(
        &hak.payload,
        &module.payload,
        request,
        &blueprint,
        &two_da.append.payload,
        &mdl.payload,
        &pwk.payload,
    )?;
    let resources = hak_resources
        .iter()
        .map(|resource| resource_report("HAK", resource))
        .chain(
            module_resources
                .iter()
                .map(|resource| resource_report("MOD", resource)),
        )
        .collect();

    Ok(StaticPlaceablePackageArtifactV1 {
        report: StaticPlaceablePackageReportV1 {
            schema_version: 1,
            status: "OFFLINE_ADMISSION_PASSED".to_owned(),
            profile: "STATIC_PLACEABLE".to_owned(),
            component_statuses: StaticPlaceableComponentStatusesV1 {
                mdl: "passed".to_owned(),
                pwk: "passed".to_owned(),
                two_da: "passed".to_owned(),
                utp: "passed".to_owned(),
                git_gic: "passed".to_owned(),
                palette: "passed".to_owned(),
                package: "passed".to_owned(),
                proof: "not_tested".to_owned(),
            },
            module_file_name: request.identity.module_file_name.clone(),
            module_display_name: request.identity.module_display_name.clone(),
            area_resref: request.identity.area_resref.clone(),
            area_name: request.identity.area_name.clone(),
            hak_file_name: request.identity.hak_file_name.clone(),
            model_resref: request.identity.model_resref.clone(),
            texture_resref: request.identity.texture_resref.clone(),
            blueprint_resref: request.identity.blueprint_resref.clone(),
            object_tag: request.identity.object_tag.clone(),
            appearance_row: blueprint.appearance_row,
            placement: request.placement,
            source_model_sha256: request.model.source_sha256.clone(),
            mdl_sha256: sha256(&mdl.payload),
            pwk_sha256: sha256(&pwk.payload),
            texture_sha256: sha256(&identity_texture.payload),
            texture_resources: texture_binding_reports(request),
            placeables_2da_sha256: sha256(&two_da.append.payload),
            utp_sha256: sha256(&utp.payload),
            itp_sha256: sha256(&itp.payload),
            git_sha256: sha256(&git),
            gic_sha256: sha256(&gic),
            hak_sha256: sha256(&hak.payload),
            module_sha256: sha256(&module.payload),
            hak_resource_count: hak_resources.len() as u32,
            module_resource_count: module_resources.len() as u32,
            model_visibility: "not_tested".to_owned(),
            proof_completeness: "missing".to_owned(),
            palette_completeness: "custom_itp_emitted".to_owned(),
            collision_completeness: "ascii_pwk_emitted_offline_readback_passed".to_owned(),
            experimental_aggressive_geometry_cleanup: options
                .experimental_aggressive_geometry_cleanup,
            authoring: request.authoring_report.clone(),
            collision: resolved_collision_report,
            texture_authoring: texture_authoring_report,
            material_separation: material_separation_report,
            model_texture_authoring: model_texture_authoring_report,
            resources,
        },
        hak_payload: hak.payload,
        module_payload: module.payload,
    })
}

fn validate_request(
    request: &StaticPlaceableBuildRequestV1,
    exception: Option<&OwnerApprovedOversizedPlaceableV1>,
) -> Result<(), PlaceableErrorV1> {
    if request.schema_version != PLACEABLE_SCHEMA_VERSION {
        return Err(error(
            "PLACEABLE-SCHEMA-INVALID",
            "request.schemaVersion",
            "schemaVersion must be 1",
        ));
    }
    validate_resref(&request.identity.module_resref, "identity.moduleResref")?;
    validate_resref(&request.identity.area_resref, "identity.areaResref")?;
    validate_resref(&request.identity.hak_resref, "identity.hakResref")?;
    validate_resref(&request.identity.model_resref, "identity.modelResref")?;
    validate_resref(&request.identity.texture_resref, "identity.textureResref")?;
    validate_resref(
        &request.identity.blueprint_resref,
        "identity.blueprintResref",
    )?;
    validate_identifier(&request.identity.object_tag, 32, "identity.objectTag")?;
    validate_identifier(&request.identity.display_name, 128, "identity.displayName")?;
    validate_file_name(
        &request.identity.module_file_name,
        ".mod",
        "identity.moduleFileName",
    )?;
    validate_file_name(
        &request.identity.hak_file_name,
        ".hak",
        "identity.hakFileName",
    )?;
    validate_placement(request.placement)?;
    validate_static_placeable_model(
        &request.model,
        &request.identity.model_resref,
        "request.model",
    )?;
    if let Some(exception) = exception {
        if request.model.source_sha256 != exception.exact_source_sha256 {
            return Err(error(
                "PLACEABLE-OVERSIZED-EXCEPTION-MODEL-MISMATCH",
                "request.model.sourceSha256",
                "owner-approved oversized exception does not match the converted model source",
            ));
        }
        let triangle_count = request
            .model
            .segments
            .iter()
            .try_fold(0usize, |sum, segment| {
                sum.checked_add(segment.indices.len() / 3).ok_or_else(|| {
                    error(
                        "PLACEABLE-OVERSIZED-TRIANGLE-COUNT-OVERFLOW",
                        "request.model.segments",
                        "oversized Placeable triangle count overflow",
                    )
                })
            })?;
        if triangle_count > exception.max_triangles {
            return Err(error(
                "PLACEABLE-OVERSIZED-TRIANGLE-LIMIT-EXCEEDED",
                "request.model.segments",
                format!(
                    "model has {triangle_count} triangles; owner-approved exact-source limit is {}",
                    exception.max_triangles
                ),
            ));
        }
    } else {
        validate_model_triangle_budget_v1(&request.model).map_err(|source| {
            error(
                &format!("PLACEABLE-{}", source.code),
                source.path,
                source.message,
            )
        })?;
    }
    if let Some(collision_model) = &request.collision_model {
        validate_static_placeable_model(
            collision_model,
            &request.identity.model_resref,
            "request.collisionModel",
        )?;
        if collision_model.source_sha256 != request.model.source_sha256 {
            return Err(error(
                "PLACEABLE-COLLISION-SOURCE-MISMATCH",
                "request.collisionModel.sourceSha256",
                "render and collision models must derive from the same source",
            ));
        }
    }
    match (&request.collision_model, &request.authoring_report) {
        (Some(_), Some(report)) if report.source_sha256 == request.model.source_sha256 => {}
        (Some(_), Some(_)) => {
            return Err(error(
                "PLACEABLE-AUTHORING-SOURCE-MISMATCH",
                "request.authoringReport.sourceSha256",
                "authoring report must derive from the same source as the render model",
            ));
        }
        (None, None) => {}
        _ => {
            return Err(error(
                "PLACEABLE-AUTHORING-CONTRACT-INCOMPLETE",
                "request",
                "collisionModel and authoringReport must either both be present or both be absent",
            ));
        }
    }
    if request.textures.is_empty() || request.material_textures.is_empty() {
        return Err(error(
            "PLACEABLE-TEXTURE-MISSING",
            "request.textures",
            "static placeable requires an explicit material texture binding and payload",
        ));
    }
    for (index, texture) in request.textures.iter().enumerate() {
        validate_resref(
            &texture.resref,
            &format!("request.textures[{index}].resref"),
        )?;
        if !matches!(texture.resource_type, TGA_RESOURCE_TYPE | DDS_RESOURCE_TYPE) {
            return Err(error(
                "PLACEABLE-TEXTURE-TYPE-UNSUPPORTED",
                format!("request.textures[{index}].resourceType"),
                "first static placeable profile accepts TGA type 3 or DDS type 2033",
            ));
        }
        if texture.payload.is_empty() {
            return Err(error(
                "PLACEABLE-TEXTURE-MISSING",
                format!("request.textures[{index}].payload"),
                "texture payload must not be empty",
            ));
        }
    }
    let mut texture_keys = BTreeSet::new();
    for (index, texture) in request.textures.iter().enumerate() {
        if !texture_keys.insert((texture.resref.to_ascii_lowercase(), texture.resource_type)) {
            return Err(error(
                "PLACEABLE-TEXTURE-RESREF-DUPLICATE",
                format!("request.textures[{index}].resref"),
                "texture resref and resource type must be unique inside the HAK",
            ));
        }
    }
    let mut bound_slots = BTreeSet::new();
    for binding in &request.material_textures {
        if !bound_slots.insert(binding.material_slot) {
            return Err(error(
                "PLACEABLE-TEXTURE-BINDING-DUPLICATE",
                "request.materialTextures",
                format!(
                    "material slot {} is bound more than once",
                    binding.material_slot
                ),
            ));
        }
        if !request
            .textures
            .iter()
            .any(|texture| texture.resref.eq_ignore_ascii_case(&binding.resref))
        {
            return Err(error(
                "PLACEABLE-TEXTURE-BINDING-MISSING",
                "request.materialTextures",
                format!("binding {} has no matching texture payload", binding.resref),
            ));
        }
    }
    for material_slot in request
        .model
        .segments
        .iter()
        .map(|segment| segment.material_slot)
    {
        if !bound_slots.contains(&material_slot) {
            return Err(error(
                "PLACEABLE-TEXTURE-BINDING-MISSING",
                "request.materialTextures",
                format!("render material slot {material_slot} has no texture binding"),
            ));
        }
    }
    if !request.textures.iter().any(|texture| {
        texture
            .resref
            .eq_ignore_ascii_case(&request.identity.texture_resref)
    }) || !request.material_textures.iter().any(|binding| {
        binding
            .resref
            .eq_ignore_ascii_case(&request.identity.texture_resref)
    }) {
        return Err(error(
            "PLACEABLE-IDENTITY-TEXTURE-MISSING",
            "identity.textureResref",
            "identity texture must be present both as a model binding and as a HAK payload",
        ));
    }
    Ok(())
}

fn validate_static_placeable_model(
    model: &AuroraModelIrV1,
    expected_root_name: &str,
    path: &str,
) -> Result<(), PlaceableErrorV1> {
    if model.schema_version != 1
        || model.nodes.is_empty()
        || model.segments.is_empty()
        || model
            .segments
            .iter()
            .any(|segment| segment.deformation != AuroraSegmentDeformationV1::Rigid)
    {
        return Err(error(
            "PLACEABLE-MODEL-INVALID",
            path,
            "static placeable requires non-empty schema-v1 rigid common model IR",
        ));
    }
    let roots = model
        .nodes
        .iter()
        .filter(|node| node.parent_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 || roots[0].name != expected_root_name {
        return Err(error(
            "PLACEABLE-MODEL-ROOT-MISMATCH",
            format!("{path}.nodes"),
            "common model IR must have one root named exactly like identity.modelResref",
        ));
    }
    Ok(())
}

fn validate_blueprint(blueprint: &StaticPlaceableBlueprintV1) -> Result<(), PlaceableErrorV1> {
    if blueprint.schema_version != 1 {
        return Err(error(
            "PLACEABLE-SCHEMA-INVALID",
            "blueprint.schemaVersion",
            "schemaVersion must be 1",
        ));
    }
    validate_resref(&blueprint.template_resref, "blueprint.templateResref")?;
    validate_identifier(&blueprint.object_tag, 32, "blueprint.objectTag")?;
    validate_identifier(&blueprint.display_name, 128, "blueprint.displayName")
}

fn validate_resref(value: &str, path: &str) -> Result<(), PlaceableErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "PLACEABLE-RESREF-INVALID",
            path,
            "resref must contain 1..16 canonical lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

fn validate_identifier(value: &str, max_bytes: usize, path: &str) -> Result<(), PlaceableErrorV1> {
    if value.is_empty() || value.len() > max_bytes || !value.is_ascii() {
        return Err(error(
            "PLACEABLE-IDENTITY-INVALID",
            path,
            format!("value must contain 1..{max_bytes} ASCII bytes"),
        ));
    }
    Ok(())
}

fn validate_file_name(value: &str, suffix: &str, path: &str) -> Result<(), PlaceableErrorV1> {
    if !value.ends_with(suffix)
        || value.len() <= suffix.len()
        || value.contains('/')
        || value.contains('\\')
        || !value.is_ascii()
    {
        return Err(error(
            "PLACEABLE-FILE-NAME-INVALID",
            path,
            format!("file name must be a plain ASCII name ending in {suffix}"),
        ));
    }
    Ok(())
}

fn validate_placement(placement: PlaceablePlacementV1) -> Result<(), PlaceableErrorV1> {
    if ![placement.x, placement.y, placement.z, placement.bearing]
        .into_iter()
        .all(f32::is_finite)
    {
        return Err(error(
            "PLACEABLE-PLACEMENT-INVALID",
            "placement",
            "X, Y, Z and Bearing must be finite",
        ));
    }
    if !(0.0..=20.0).contains(&placement.x)
        || !(0.0..=20.0).contains(&placement.y)
        || !(-10.0..=20.0).contains(&placement.z)
    {
        return Err(error(
            "PLACEABLE-PLACEMENT-OUTSIDE-PROOF-AREA",
            "placement",
            "placement must remain inside the generated 2x2 proof Area",
        ));
    }
    Ok(())
}

fn replace_list(
    root: &mut GffStructV1,
    label: &str,
    value: Vec<GffStructV1>,
) -> Result<(), PlaceableErrorV1> {
    let target = root
        .fields
        .iter_mut()
        .find(|field| field.label == label)
        .ok_or_else(|| {
            error(
                "PLACEABLE-GFF-LIST-MISSING",
                label,
                "shared generated Area base is missing the required list",
            )
        })?;
    target.value = GffValueV1::List(value);
    Ok(())
}

fn validate_git_readback(
    bytes: &[u8],
    blueprint: &StaticPlaceableBlueprintV1,
    placement: PlaceablePlacementV1,
) -> Result<(), PlaceableErrorV1> {
    let document = read_gff_v32(bytes, &GffLimitsV1::default())
        .map_err(|source| map_error("PLACEABLE-GIT-READBACK-FAILED", "git", source))?;
    let placeables = require_list(&document.root, "Placeable List", "git")?;
    if placeables.len() != 1 || placeables[0].struct_id != 9 {
        return Err(error(
            "PLACEABLE-GIT-SEMANTIC-DIFF",
            "git.Placeable List",
            "expected exactly one StructID 9 placeable",
        ));
    }
    let instance = &placeables[0];
    if instance.fields.len() != 54 {
        return Err(error(
            "PLACEABLE-GIT-SEMANTIC-DIFF",
            "git.Placeable List[0].fields",
            "static GIT placeable must contain 50 frozen common fields plus X/Y/Z/Bearing",
        ));
    }
    for (index, ((expected_label, expected_kind), actual)) in UTP_FIELD_MANIFEST[..50]
        .iter()
        .zip(&instance.fields[..50])
        .enumerate()
    {
        if actual.label != *expected_label || gff_kind(&actual.value) != *expected_kind {
            return Err(error(
                "PLACEABLE-GIT-SEMANTIC-DIFF",
                format!("git.Placeable List[0].fields[{index}]"),
                format!(
                    "expected {expected_label}:{expected_kind}, got {}:{}",
                    actual.label,
                    gff_kind(&actual.value)
                ),
            ));
        }
    }
    for (index, expected_label) in ["X", "Y", "Z", "Bearing"].iter().enumerate() {
        let actual = &instance.fields[50 + index];
        if actual.label != *expected_label || !matches!(actual.value, GffValueV1::Float(_)) {
            return Err(error(
                "PLACEABLE-GIT-SEMANTIC-DIFF",
                format!("git.Placeable List[0].fields[{}]", 50 + index),
                format!("expected {expected_label}:FLOAT"),
            ));
        }
    }
    require_value(
        instance,
        "TemplateResRef",
        &GffValueV1::ResRef(blueprint.template_resref.clone()),
        "git.Placeable List[0]",
    )?;
    require_value(
        instance,
        "Appearance",
        &GffValueV1::Dword(blueprint.appearance_row.value),
        "git.Placeable List[0]",
    )?;
    for (label, value) in [
        ("X", placement.x),
        ("Y", placement.y),
        ("Z", placement.z),
        ("Bearing", placement.bearing),
    ] {
        require_value(
            instance,
            label,
            &GffValueV1::Float(value),
            "git.Placeable List[0]",
        )?;
    }
    Ok(())
}

fn validate_gic_readback(bytes: &[u8]) -> Result<(), PlaceableErrorV1> {
    let document = read_gff_v32(bytes, &GffLimitsV1::default())
        .map_err(|source| map_error("PLACEABLE-GIC-READBACK-FAILED", "gic", source))?;
    let placeables = require_list(&document.root, "Placeable List", "gic")?;
    if placeables.len() != 1
        || placeables[0].struct_id != 9
        || !matches!(
            find_value(&placeables[0], "Comment"),
            Some(GffValueV1::String(_))
        )
    {
        return Err(error(
            "PLACEABLE-GIC-SEMANTIC-DIFF",
            "gic.Placeable List",
            "expected one StructID 9 comment entry aligned with GIT",
        ));
    }
    Ok(())
}

fn validate_package_readback(
    hak_bytes: &[u8],
    module_bytes: &[u8],
    request: &StaticPlaceableBuildRequestV1,
    blueprint: &StaticPlaceableBlueprintV1,
    expected_two_da: &[u8],
    expected_mdl: &[u8],
    expected_pwk: &[u8],
) -> Result<(), PlaceableErrorV1> {
    let hak = ErfArchive::parse(hak_bytes)
        .map_err(|source| map_error("PLACEABLE-HAK-READBACK-FAILED", "hak", source))?;
    for (resref, resource_type, expected) in [
        ("placeables", PLACEABLES_2DA_RESOURCE_TYPE, expected_two_da),
        (
            request.identity.model_resref.as_str(),
            MDL_RESOURCE_TYPE,
            expected_mdl,
        ),
        (
            request.identity.model_resref.as_str(),
            PWK_RESOURCE_TYPE,
            expected_pwk,
        ),
    ] {
        let actual = hak.find(resref, resource_type).map_err(|source| {
            map_error(
                "PLACEABLE-HAK-READBACK-FAILED",
                &format!("hak.{resref}:{resource_type}"),
                source,
            )
        })?;
        if actual != expected {
            return Err(error(
                "PLACEABLE-HAK-SEMANTIC-DIFF",
                format!("hak.{resref}:{resource_type}"),
                "resource payload differs after archive readback",
            ));
        }
    }
    for texture in &request.textures {
        let actual = hak
            .find(&texture.resref, texture.resource_type)
            .map_err(|source| {
                map_error(
                    "PLACEABLE-HAK-READBACK-FAILED",
                    &format!("hak.{}:{}", texture.resref, texture.resource_type),
                    source,
                )
            })?;
        if actual != texture.payload {
            return Err(error(
                "PLACEABLE-HAK-SEMANTIC-DIFF",
                format!("hak.{}:{}", texture.resref, texture.resource_type),
                "texture payload differs after archive readback",
            ));
        }
    }
    let pwk = hak
        .find(&request.identity.model_resref, PWK_RESOURCE_TYPE)
        .map_err(|source| map_error("PLACEABLE-PWK-MISSING", "hak.pwk", source))?;
    inspect_ascii_placeable_walkmesh_v1(pwk)
        .map_err(|source| map_error("PLACEABLE-PWK-OFFLINE-READBACK-FAILED", "hak.pwk", source))?;

    let module = ErfArchive::parse(module_bytes)
        .map_err(|source| map_error("PLACEABLE-MOD-READBACK-FAILED", "module", source))?;
    let ifo = read_gff_v32(
        module
            .find("module", IFO_RESOURCE_TYPE)
            .map_err(|source| map_error("PLACEABLE-IFO-MISSING", "module.ifo", source))?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| map_error("PLACEABLE-IFO-READBACK-FAILED", "module.ifo", source))?;
    let hak_list = require_list(&ifo.root, "Mod_HakList", "module.ifo")?;
    if hak_list.len() != 1
        || find_value(&hak_list[0], "Mod_Hak")
            != Some(&GffValueV1::String(
                request.identity.hak_resref.as_bytes().to_vec(),
            ))
    {
        return Err(error(
            "PLACEABLE-IFO-SEMANTIC-DIFF",
            "module.ifo.Mod_HakList",
            "module must reference exactly the requested singleton HAK",
        ));
    }
    let are = read_gff_v32(
        module
            .find(&request.identity.area_resref, ARE_RESOURCE_TYPE)
            .map_err(|source| map_error("PLACEABLE-ARE-MISSING", "module.are", source))?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| map_error("PLACEABLE-ARE-READBACK-FAILED", "module.are", source))?;
    require_value(
        &are.root,
        "Name",
        &loc(&request.identity.area_name),
        "module.are",
    )?;
    let utp = module
        .find(&request.identity.blueprint_resref, UTP_RESOURCE_TYPE)
        .map_err(|source| map_error("PLACEABLE-UTP-MISSING", "module.utp", source))?;
    validate_static_placeable_utp_readback(utp, blueprint)?;
    let itp = module
        .find("placeablepalcus", ITP_RESOURCE_TYPE)
        .map_err(|source| {
            map_error(
                "PLACEABLE-ITP-MISSING",
                "module.placeablepalcus.itp",
                source,
            )
        })?;
    validate_palette_itp_readback(itp, blueprint)?;
    let git = module
        .find(&request.identity.area_resref, GIT_RESOURCE_TYPE)
        .map_err(|source| map_error("PLACEABLE-GIT-MISSING", "module.git", source))?;
    validate_git_readback(git, blueprint, request.placement)?;
    let gic = module
        .find(&request.identity.area_resref, GIC_RESOURCE_TYPE)
        .map_err(|source| map_error("PLACEABLE-GIC-MISSING", "module.gic", source))?;
    validate_gic_readback(gic)
}

fn field(label: &str, value: GffValueV1) -> GffFieldV1 {
    GffFieldV1 {
        label: label.to_owned(),
        value,
    }
}

fn loc(value: &str) -> GffValueV1 {
    GffValueV1::LocString(GffLocStringV1 {
        string_ref: u32::MAX,
        substrings: vec![GffLocSubstringV1 {
            string_id: 0,
            bytes: value.as_bytes().to_vec(),
        }],
    })
}

fn empty_loc() -> GffValueV1 {
    GffValueV1::LocString(GffLocStringV1 {
        string_ref: u32::MAX,
        substrings: Vec::new(),
    })
}

fn find_value<'a>(structure: &'a GffStructV1, label: &str) -> Option<&'a GffValueV1> {
    structure
        .fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
}

fn require_value(
    structure: &GffStructV1,
    label: &str,
    expected: &GffValueV1,
    path: &str,
) -> Result<(), PlaceableErrorV1> {
    if find_value(structure, label) != Some(expected) {
        return Err(error(
            "PLACEABLE-GFF-SEMANTIC-DIFF",
            format!("{path}.{label}"),
            "field differs from the placeable contract",
        ));
    }
    Ok(())
}

fn require_list<'a>(
    structure: &'a GffStructV1,
    label: &str,
    path: &str,
) -> Result<&'a [GffStructV1], PlaceableErrorV1> {
    match find_value(structure, label) {
        Some(GffValueV1::List(value)) => Ok(value),
        _ => Err(error(
            "PLACEABLE-GFF-LIST-MISSING",
            format!("{path}.{label}"),
            "required GFF list is missing or has the wrong type",
        )),
    }
}

fn gff_kind(value: &GffValueV1) -> &'static str {
    match value {
        GffValueV1::Byte(_) => "BYTE",
        GffValueV1::Char(_) => "CHAR",
        GffValueV1::Word(_) => "WORD",
        GffValueV1::Short(_) => "SHORT",
        GffValueV1::Dword(_) => "DWORD",
        GffValueV1::Int(_) => "INT",
        GffValueV1::Dword64(_) => "DWORD64",
        GffValueV1::Int64(_) => "INT64",
        GffValueV1::Float(_) => "FLOAT",
        GffValueV1::Double(_) => "DOUBLE",
        GffValueV1::String(_) => "CEXOSTRING",
        GffValueV1::ResRef(_) => "RESREF",
        GffValueV1::LocString(_) => "LOCSTRING",
        GffValueV1::Void(_) => "VOID",
        GffValueV1::Struct(_) => "STRUCT",
        GffValueV1::List(_) => "LIST",
    }
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

fn resource_report(
    container: &str,
    resource: &HakResourceInputV1,
) -> PlaceableResourceBindingReportV1 {
    let role = match resource.resource_type {
        PLACEABLES_2DA_RESOURCE_TYPE => "PLACEABLES_2DA",
        MDL_RESOURCE_TYPE => "MODEL",
        TGA_RESOURCE_TYPE | DDS_RESOURCE_TYPE => "TEXTURE",
        IFO_RESOURCE_TYPE => "MODULE_INFO",
        FAC_RESOURCE_TYPE => "FACTIONS",
        ARE_RESOURCE_TYPE => "AREA",
        GIT_RESOURCE_TYPE => "AREA_INSTANCES",
        GIC_RESOURCE_TYPE => "AREA_COMMENTS",
        UTP_RESOURCE_TYPE => "PLACEABLE_BLUEPRINT",
        ITP_RESOURCE_TYPE => "PLACEABLE_PALETTE",
        PWK_RESOURCE_TYPE => "PLACEABLE_WALKMESH",
        _ => "AUXILIARY",
    };
    PlaceableResourceBindingReportV1 {
        container: container.to_owned(),
        role: role.to_owned(),
        resref: resource.resref.clone(),
        resource_type: resource.resource_type,
        byte_length: resource.payload.len() as u64,
        sha256: sha256(&resource.payload),
    }
}

fn texture_binding_reports(
    request: &StaticPlaceableBuildRequestV1,
) -> Vec<PlaceableTextureBindingReportV1> {
    request
        .textures
        .iter()
        .map(|texture| {
            let mut material_slots = request
                .material_textures
                .iter()
                .filter(|binding| binding.resref.eq_ignore_ascii_case(&texture.resref))
                .map(|binding| binding.material_slot)
                .collect::<Vec<_>>();
            material_slots.sort_unstable();
            PlaceableTextureBindingReportV1 {
                resref: texture.resref.clone(),
                resource_type: texture.resource_type,
                material_slots,
                byte_length: texture.payload.len() as u64,
                sha256: sha256(&texture.payload),
            }
        })
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blueprint(template_resref: &str, appearance: u32) -> StaticPlaceableBlueprintV1 {
        StaticPlaceableBlueprintV1 {
            schema_version: 1,
            template_resref: template_resref.to_owned(),
            object_tag: "m2a_plc_pedestal".to_owned(),
            display_name: "Meshy Ritual Pedestal".to_owned(),
            appearance_row: PlaceableAppearanceRowV1 { value: appearance },
            palette_id: 7,
        }
    }

    fn rewrite_document(document: &GffDocumentV1) -> Vec<u8> {
        write_gff_v32(document, &GffWriterOptionsV1::default())
            .expect("rewrite test GFF")
            .payload
    }

    #[test]
    fn utp_readback_rejects_identity_appearance_state_and_manifest_mutations() {
        let expected = blueprint("m2a_plc_utp", 16_500);
        let artifact = write_static_placeable_utp_v1(&expected).expect("write baseline UTP");

        let wrong_template = blueprint("other_plc_utp", 16_500);
        let error = validate_static_placeable_utp_readback(&artifact.payload, &wrong_template)
            .expect_err("archive key/template mismatch");
        assert_eq!(error.code, "PLACEABLE-GFF-SEMANTIC-DIFF");

        let wrong_appearance = blueprint("m2a_plc_utp", 16_501);
        let error = validate_static_placeable_utp_readback(&artifact.payload, &wrong_appearance)
            .expect_err("appearance mismatch");
        assert_eq!(error.code, "PLACEABLE-GFF-SEMANTIC-DIFF");

        let mut document =
            read_gff_v32(&artifact.payload, &GffLimitsV1::default()).expect("read baseline UTP");
        document
            .root
            .fields
            .iter_mut()
            .find(|field| field.label == "AnimationState")
            .expect("AnimationState")
            .value = GffValueV1::Byte(1);
        let error = validate_static_placeable_utp_readback(&rewrite_document(&document), &expected)
            .expect_err("non-static state");
        assert_eq!(error.code, "PLACEABLE-GFF-SEMANTIC-DIFF");

        let mut document =
            read_gff_v32(&artifact.payload, &GffLimitsV1::default()).expect("read baseline UTP");
        document
            .root
            .fields
            .retain(|field| field.label != "Appearance");
        let error = validate_static_placeable_utp_readback(&rewrite_document(&document), &expected)
            .expect_err("missing required field");
        assert_eq!(error.code, "PLACEABLE-UTP-SEMANTIC-DIFF");

        let mut document =
            read_gff_v32(&artifact.payload, &GffLimitsV1::default()).expect("read baseline UTP");
        document.root.fields.push(field(
            "UnknownField",
            GffValueV1::String(b"forbidden".to_vec()),
        ));
        let error = validate_static_placeable_utp_readback(&rewrite_document(&document), &expected)
            .expect_err("unknown field");
        assert_eq!(error.code, "PLACEABLE-UTP-SEMANTIC-DIFF");
    }

    #[test]
    fn git_readback_rejects_template_and_appearance_mismatch() {
        let expected = blueprint("m2a_plc_utp", 16_500);
        let placement = PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let git = build_static_placeable_git(&expected, placement).expect("write baseline GIT");

        let wrong_template = blueprint("other_plc_utp", 16_500);
        let error = validate_git_readback(&git, &wrong_template, placement)
            .expect_err("GIT template mismatch");
        assert_eq!(error.code, "PLACEABLE-GFF-SEMANTIC-DIFF");

        let wrong_appearance = blueprint("m2a_plc_utp", 16_501);
        let error = validate_git_readback(&git, &wrong_appearance, placement)
            .expect_err("GIT appearance mismatch");
        assert_eq!(error.code, "PLACEABLE-GFF-SEMANTIC-DIFF");
    }
}
