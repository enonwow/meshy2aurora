use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GffDocumentV1, GffFieldV1, GffLimitsV1, GffLocStringV1, GffLocSubstringV1, GffStructV1,
        GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
    },
    glb::{EmbeddedImageDecodeLimitsV1, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1, write_hak_v1},
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
        MdlWriterOptionsV1, inspect_binary_mdl, write_binary_tile_mdl_v1,
    },
    model_ir::{AuroraModelIrV1, AuroraSegmentDeformationV1},
    model_limits::validate_model_triangle_budget_v1,
    model_material_capabilities::{ModelRenderTargetV1, validate_material_separation_counts_v1},
    model_material_separation::{
        ModelMaterialSeparationDocumentV1, ModelMaterialSeparationReportV1,
        resolve_model_materials_v1,
    },
    model_pipeline::{
        resolve_base_color_image_index_v1, sanitize_meshy_h1_degenerate_triangles_v1,
    },
    model_segmentation::segment_model_for_binary_mdl_v1,
    model_texture_authoring::{
        ModelTextureAuthoringDocumentV1, ModelTexturePayloadDescriptorV1,
        ModelTextureResolutionReportV1, resolve_model_texture_authoring_v1,
    },
    placeable::{static_placeable_glb_limits_v1, static_placeable_profile_a_options_v1},
    profile_a::{
        convert_profile_a, convert_profile_a_with_material_separation_v1,
        derive_meshy_m0_static_rigid_profile_v1,
    },
    proof_module::{
        binary_creature_multi_fixture_gic, binary_creature_multi_fixture_git, binary_m0_area_for,
        binary_m0_module_ifo_for, proof_factions,
    },
    tga::{TgaWriterOptionsV1, write_tga_v1},
    walkmesh::{
        TileNavigationIrV1, TileSurfaceV1, flat_tile_navigation_v1, inspect_ascii_tile_wok_v1,
        validate_tile_navigation_v1, write_ascii_tile_wok_v1,
    },
};

use super::set::{
    TileDescriptorV1, minimal_static_tileset_v1, parse_tileset_v1, resolve_are_tile_v1,
    write_tileset_v1,
};

pub const TILE_SCHEMA_VERSION: u32 = 1;
pub const MDL_RESOURCE_TYPE: u16 = 2002;
pub const SET_RESOURCE_TYPE: u16 = 2013;
pub const WOK_RESOURCE_TYPE: u16 = 2016;
pub const TGA_RESOURCE_TYPE: u16 = 3;
pub const DDS_RESOURCE_TYPE: u16 = 2033;
pub const IFO_RESOURCE_TYPE: u16 = 2014;
pub const ARE_RESOURCE_TYPE: u16 = 2012;
pub const GIT_RESOURCE_TYPE: u16 = 2023;
pub const GIC_RESOURCE_TYPE: u16 = 2046;
pub const FAC_RESOURCE_TYPE: u16 = 2038;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticTileIdentityV1 {
    pub module_resref: String,
    pub module_file_name: String,
    pub module_display_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub hak_resref: String,
    pub hak_file_name: String,
    pub tileset_resref: String,
    pub model_resref: String,
    pub texture_resref: String,
    pub image_map_resref: String,
}

impl StaticTileIdentityV1 {
    pub fn owner_candidate_v1() -> Self {
        Self {
            module_resref: "m2atilestv1".to_owned(),
            module_file_name: "m2a_tile_static_v1.mod".to_owned(),
            module_display_name: "Meshy2Aurora Tile Static V1".to_owned(),
            area_resref: "m2atilearea".to_owned(),
            area_name: "M2A Tile Static 2x2".to_owned(),
            hak_resref: "m2atilestv1".to_owned(),
            hak_file_name: "m2a_tile_static_v1.hak".to_owned(),
            tileset_resref: "m2atilesetv1".to_owned(),
            model_resref: "m2atilemdl1".to_owned(),
            texture_resref: "m2atiletex1".to_owned(),
            image_map_resref: "m2atilemap1".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TileTextureInputV1 {
    pub resref: String,
    pub resource_type: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticTileBuildRequestV1 {
    pub schema_version: u32,
    pub identity: StaticTileIdentityV1,
    pub interior: bool,
    pub terrain_name: String,
    pub surface: TileSurfaceV1,
    pub model: AuroraModelIrV1,
    pub navigation: TileNavigationIrV1,
    pub material_textures: Vec<MdlMaterialTextureBindingV1>,
    pub textures: Vec<TileTextureInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileResourceBindingReportV1 {
    pub container: String,
    pub role: String,
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTilePackageReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub profile: String,
    pub module_file_name: String,
    pub module_display_name: String,
    pub area_resref: String,
    pub area_name: String,
    pub area_size: [u32; 2],
    pub area_tile_count: u32,
    pub entry_position: [f32; 3],
    pub hak_file_name: String,
    pub hak_resref: String,
    pub tileset_resref: String,
    pub tile_id: u32,
    pub model_resref: String,
    pub wok_resref: String,
    pub texture_resref: String,
    pub image_map_resref: String,
    pub walkmesh_class_token: String,
    pub surface_id: i32,
    pub model_triangle_count: u32,
    pub wok_triangle_count: u32,
    pub aabb_entry_count: u32,
    pub mdl_sha256: String,
    pub wok_sha256: String,
    pub set_sha256: String,
    pub texture_sha256: String,
    pub image_map_sha256: String,
    pub hak_sha256: String,
    pub module_sha256: String,
    pub hak_resource_count: u32,
    pub module_resource_count: u32,
    pub model_visibility: String,
    pub proof_completeness: String,
    pub navigation_spawn_walkable: String,
    pub navigation_seam_walkable: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_separation: Option<ModelMaterialSeparationReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_texture_authoring: Option<ModelTextureResolutionReportV1>,
    pub resources: Vec<TileResourceBindingReportV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticTilePackageArtifactV1 {
    pub hak_payload: Vec<u8>,
    pub module_payload: Vec<u8>,
    pub mdl_payload: Vec<u8>,
    pub wok_payload: Vec<u8>,
    pub set_payload: Vec<u8>,
    pub texture_payload: Vec<u8>,
    pub image_map_payload: Vec<u8>,
    pub report: StaticTilePackageReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilePackageErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for TilePackageErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TilePackageErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> TilePackageErrorV1 {
    TilePackageErrorV1 {
        schema_version: TILE_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

fn map_error(code: &str, path: &str, source: impl fmt::Display) -> TilePackageErrorV1 {
    error(code, path, source.to_string())
}

/// Full browser/core path from selected GLB bytes to one exact tile HAK/MOD.
pub fn build_meshy_static_tile_package_v1(
    source_glb: &[u8],
    identity: &StaticTileIdentityV1,
    interior: bool,
    terrain_name: &str,
    surface: TileSurfaceV1,
) -> Result<StaticTilePackageArtifactV1, TilePackageErrorV1> {
    validate_identity(identity)?;
    let limits = static_placeable_glb_limits_v1();
    let mut ingest = ingest_glb(source_glb, &limits).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            source.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut ingest).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let conversion = convert_profile_a(&ingest, &rig, &static_placeable_profile_a_options_v1())
        .map_err(|source| {
            error(
                &format!("TILE-{}", source.code),
                source.path,
                source.message,
            )
        })?;
    if !conversion.report.conversion_eligible {
        return Err(error(
            "TILE-PROFILE-INELIGIBLE",
            "conversion.report",
            "GLB did not pass static rigid Profile A admission",
        ));
    }
    let mut model = conversion.creature.ok_or_else(|| {
        error(
            "TILE-PROFILE-INELIGIBLE",
            "conversion.model",
            "eligible conversion produced no common model IR",
        )
    })?;
    normalize_model_root(&mut model, &identity.model_resref)?;
    let selection = resolve_base_color_image_index_v1(&ingest, &model).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        selection.source_image_index,
        &limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.json_path.unwrap_or_else(|| "images".to_owned()),
            source.message,
        )
    })?;
    let tga = write_tga_v1(&image, &TgaWriterOptionsV1::default()).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let navigation = flat_tile_navigation_v1(&identity.model_resref, surface)
        .map_err(|source| map_error("TILE-WOK-DERIVE-FAILED", "navigation", source))?;
    build_static_tile_package_v1(&StaticTileBuildRequestV1 {
        schema_version: TILE_SCHEMA_VERSION,
        identity: identity.clone(),
        interior,
        terrain_name: terrain_name.to_owned(),
        surface,
        model,
        navigation,
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: selection.material_slot,
            resref: identity.texture_resref.clone(),
        }],
        textures: vec![TileTextureInputV1 {
            resref: identity.texture_resref.clone(),
            resource_type: TGA_RESOURCE_TYPE,
            payload: tga.payload,
        }],
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_meshy_static_tile_package_v2(
    source_glb: &[u8],
    identity: &StaticTileIdentityV1,
    interior: bool,
    terrain_name: &str,
    surface: TileSurfaceV1,
    material_separation: &ModelMaterialSeparationDocumentV1,
    texture_authoring: &ModelTextureAuthoringDocumentV1,
    texture_payload_blob: &[u8],
    texture_payload_descriptors: &[ModelTexturePayloadDescriptorV1],
) -> Result<StaticTilePackageArtifactV1, TilePackageErrorV1> {
    validate_identity(identity)?;
    let limits = static_placeable_glb_limits_v1();
    let mut ingest = ingest_glb(source_glb, &limits).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            source.message,
        )
    })?;
    sanitize_meshy_h1_degenerate_triangles_v1(&mut ingest).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let resolved_materials =
        resolve_model_materials_v1(&ingest.ir, material_separation).map_err(|source| {
            error(
                &format!("TILE-{}", source.code),
                source.path,
                source.message,
            )
        })?;
    validate_material_separation_counts_v1(
        ModelRenderTargetV1::Tile,
        resolved_materials.report.material_slots.len(),
        resolved_materials.report.output_section_count,
    )
    .map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let conversion = convert_profile_a_with_material_separation_v1(
        &ingest,
        &rig,
        &static_placeable_profile_a_options_v1(),
        material_separation,
    )
    .map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    if !conversion.report.conversion_eligible {
        return Err(error(
            "TILE-PROFILE-INELIGIBLE",
            "conversion.report",
            "material-separated GLB did not pass static rigid Profile A admission",
        ));
    }
    let mut model = conversion.creature.ok_or_else(|| {
        error(
            "TILE-PROFILE-INELIGIBLE",
            "conversion.model",
            "eligible conversion produced no common model IR",
        )
    })?;
    normalize_model_root(&mut model, &identity.model_resref)?;
    let resolved_textures = resolve_model_texture_authoring_v1(
        source_glb,
        &limits,
        &ingest,
        &resolved_materials,
        &identity.texture_resref,
        texture_authoring,
        texture_payload_blob,
        texture_payload_descriptors,
    )
    .map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let navigation = flat_tile_navigation_v1(&identity.model_resref, surface)
        .map_err(|source| map_error("TILE-WOK-DERIVE-FAILED", "navigation", source))?;
    let texture_report = resolved_textures.report;
    build_static_tile_package_inner_v1(
        &StaticTileBuildRequestV1 {
            schema_version: TILE_SCHEMA_VERSION,
            identity: identity.clone(),
            interior,
            terrain_name: terrain_name.to_owned(),
            surface,
            model,
            navigation,
            material_textures: resolved_textures.material_textures,
            textures: resolved_textures
                .textures
                .into_iter()
                .map(|texture| TileTextureInputV1 {
                    resref: texture.resref,
                    resource_type: texture.resource_type,
                    payload: texture.payload,
                })
                .collect(),
        },
        Some(resolved_materials.report),
        Some(texture_report),
    )
}

pub fn build_static_tile_package_v1(
    request: &StaticTileBuildRequestV1,
) -> Result<StaticTilePackageArtifactV1, TilePackageErrorV1> {
    build_static_tile_package_inner_v1(request, None, None)
}

fn build_static_tile_package_inner_v1(
    request: &StaticTileBuildRequestV1,
    material_separation_report: Option<ModelMaterialSeparationReportV1>,
    model_texture_authoring_report: Option<ModelTextureResolutionReportV1>,
) -> Result<StaticTilePackageArtifactV1, TilePackageErrorV1> {
    validate_request(request)?;
    let mut render_model = request.model.clone();
    segment_model_for_binary_mdl_v1(&mut render_model)
        .map_err(|source| map_error("TILE-MODEL-SEGMENTATION-FAILED", "model", source))?;
    let identity = &request.identity;
    let diffuse = request
        .textures
        .iter()
        .find(|texture| texture.resref == identity.texture_resref)
        .expect("validated identity diffuse");
    let image_map_payload = request
        .textures
        .iter()
        .find(|texture| texture.resref == identity.image_map_resref)
        .map(|texture| texture.payload.clone())
        .unwrap_or_else(|| diffuse.payload.clone());

    let mdl = write_binary_tile_mdl_v1(
        &render_model,
        &request.navigation,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::TileStaticV1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: identity.model_resref.clone(),
            diffuse_texture_resref_by_material_slot: request.material_textures.clone(),
        },
    )
    .map_err(|source| map_error("TILE-MDL-WRITE-FAILED", "model", source))?;
    let wok = write_ascii_tile_wok_v1(&request.navigation)
        .map_err(|source| map_error("TILE-WOK-WRITE-FAILED", "navigation", source))?;
    let mut descriptor = TileDescriptorV1::flat_v1(
        &identity.model_resref,
        &request.terrain_name,
        request.surface,
    );
    descriptor.image_map_2d = Some(identity.image_map_resref.clone());
    let tileset = minimal_static_tileset_v1(&identity.tileset_resref, descriptor, request.interior)
        .map_err(|source| map_error("TILE-SET-BUILD-FAILED", "tileset", source))?;
    let set = write_tileset_v1(&tileset)
        .map_err(|source| map_error("TILE-SET-WRITE-FAILED", "tileset", source))?;

    let mut hak_resources = vec![
        resource(
            &identity.tileset_resref,
            SET_RESOURCE_TYPE,
            set.payload.clone(),
        ),
        resource(
            &identity.model_resref,
            MDL_RESOURCE_TYPE,
            mdl.payload.clone(),
        ),
        resource(
            &identity.model_resref,
            WOK_RESOURCE_TYPE,
            wok.payload.clone(),
        ),
    ];
    for texture in &request.textures {
        hak_resources.push(resource(
            &texture.resref,
            texture.resource_type,
            texture.payload.clone(),
        ));
    }
    if !request
        .textures
        .iter()
        .any(|texture| texture.resref == identity.image_map_resref)
    {
        hak_resources.push(resource(
            &identity.image_map_resref,
            TGA_RESOURCE_TYPE,
            image_map_payload.clone(),
        ));
    }
    let hak = write_hak_v1(&hak_resources, &HakWriterOptionsV1::default())
        .map_err(|source| map_error("TILE-HAK-WRITE-FAILED", "hak", source))?;

    let ifo = build_tile_ifo(identity)?;
    let are = build_tile_are(identity)?;
    let git = binary_creature_multi_fixture_git(&[])
        .map_err(|source| map_error("TILE-GIT-WRITE-FAILED", "area.git", source))?;
    let gic = binary_creature_multi_fixture_gic(&[])
        .map_err(|source| map_error("TILE-GIC-WRITE-FAILED", "area.gic", source))?;
    let fac = proof_factions()
        .map_err(|source| map_error("TILE-FAC-WRITE-FAILED", "repute.fac", source))?;
    let module_resources = vec![
        resource("module", IFO_RESOURCE_TYPE, ifo),
        resource("repute", FAC_RESOURCE_TYPE, fac),
        resource(&identity.area_resref, ARE_RESOURCE_TYPE, are),
        resource(&identity.area_resref, GIT_RESOURCE_TYPE, git),
        resource(&identity.area_resref, GIC_RESOURCE_TYPE, gic),
    ];
    let module = write_erf_archive_v1(
        ErfFileType::Module,
        &module_resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|source| map_error("TILE-MOD-WRITE-FAILED", "module", source))?;

    validate_package_readback(
        request,
        &hak.payload,
        &module.payload,
        &mdl.payload,
        &wok.payload,
        &set.payload,
        &image_map_payload,
    )?;
    validate_repeated_tile_seam(&request.navigation)?;
    let binding = resolve_are_tile_v1(&tileset, 0)
        .map_err(|source| map_error("TILE-CHAIN-READBACK-FAILED", "ARE.Tile_ID", source))?;
    let resources = hak_resources
        .iter()
        .map(|resource| resource_report("HAK", resource))
        .chain(
            module_resources
                .iter()
                .map(|resource| resource_report("MOD", resource)),
        )
        .collect::<Vec<_>>();
    let model_triangle_count = request
        .model
        .segments
        .iter()
        .try_fold(0_u32, |sum, segment| {
            sum.checked_add((segment.indices.len() / 3) as u32)
        })
        .ok_or_else(|| {
            error(
                "TILE-REPORT-OVERFLOW",
                "model.segments",
                "model triangle count overflow",
            )
        })?;
    Ok(StaticTilePackageArtifactV1 {
        hak_payload: hak.payload.clone(),
        module_payload: module.payload.clone(),
        mdl_payload: mdl.payload.clone(),
        wok_payload: wok.payload.clone(),
        set_payload: set.payload.clone(),
        texture_payload: diffuse.payload.clone(),
        image_map_payload: image_map_payload.clone(),
        report: StaticTilePackageReportV1 {
            schema_version: TILE_SCHEMA_VERSION,
            status: "ready_for_owner_proof".to_owned(),
            profile: "TileStaticV1".to_owned(),
            module_file_name: identity.module_file_name.clone(),
            module_display_name: identity.module_display_name.clone(),
            area_resref: identity.area_resref.clone(),
            area_name: identity.area_name.clone(),
            area_size: [2, 2],
            area_tile_count: 4,
            entry_position: [5.0, 5.0, 0.0],
            hak_file_name: identity.hak_file_name.clone(),
            hak_resref: identity.hak_resref.clone(),
            tileset_resref: identity.tileset_resref.clone(),
            tile_id: 0,
            model_resref: binding.model_resref,
            wok_resref: binding.wok_resref,
            texture_resref: identity.texture_resref.clone(),
            image_map_resref: identity.image_map_resref.clone(),
            walkmesh_class_token: binding.walkmesh_class_token,
            surface_id: request.surface.id(),
            model_triangle_count,
            wok_triangle_count: request.navigation.faces.len() as u32,
            aabb_entry_count: request.navigation.aabb_tree.entries.len() as u32,
            mdl_sha256: sha256(&mdl.payload),
            wok_sha256: sha256(&wok.payload),
            set_sha256: sha256(&set.payload),
            texture_sha256: sha256(&diffuse.payload),
            image_map_sha256: sha256(&image_map_payload),
            hak_sha256: sha256(&hak.payload),
            module_sha256: sha256(&module.payload),
            hak_resource_count: hak_resources.len() as u32,
            module_resource_count: module_resources.len() as u32,
            model_visibility: "not_tested".to_owned(),
            proof_completeness: "missing".to_owned(),
            navigation_spawn_walkable: "offline_verified".to_owned(),
            navigation_seam_walkable: "offline_verified".to_owned(),
            material_separation: material_separation_report,
            model_texture_authoring: model_texture_authoring_report,
            resources,
        },
    })
}

fn validate_request(request: &StaticTileBuildRequestV1) -> Result<(), TilePackageErrorV1> {
    if request.schema_version != TILE_SCHEMA_VERSION {
        return Err(error(
            "TILE-SCHEMA-INVALID",
            "request.schemaVersion",
            "StaticTileBuildRequestV1 must use schema version 1",
        ));
    }
    validate_identity(&request.identity)?;
    if request.terrain_name.is_empty()
        || request.terrain_name.len() > 64
        || !request.terrain_name.is_ascii()
    {
        return Err(error(
            "TILE-TERRAIN-INVALID",
            "request.terrainName",
            "terrain name must contain 1..64 ASCII bytes",
        ));
    }
    if request.model.schema_version != 1
        || request.model.nodes.is_empty()
        || request.model.segments.is_empty()
        || request
            .model
            .segments
            .iter()
            .any(|segment| segment.deformation != AuroraSegmentDeformationV1::Rigid)
    {
        return Err(error(
            "TILE-MODEL-INVALID",
            "request.model",
            "TileStaticV1 requires non-empty schema-v1 rigid common model IR",
        ));
    }
    validate_model_triangle_budget_v1(&request.model).map_err(|source| {
        error(
            &format!("TILE-{}", source.code),
            source.path,
            source.message,
        )
    })?;
    let roots = request
        .model
        .nodes
        .iter()
        .filter(|node| node.parent_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 || roots[0].name != request.identity.model_resref {
        return Err(error(
            "TILE-MODEL-ROOT-MISMATCH",
            "request.model.nodes",
            "model must have one root named exactly like identity.modelResref",
        ));
    }
    validate_tile_navigation_v1(&request.navigation)
        .map_err(|source| map_error("TILE-NAVIGATION-INVALID", "request.navigation", source))?;
    if request.navigation.model_resref != request.identity.model_resref
        || request.navigation.surface != request.surface
    {
        return Err(error(
            "TILE-NAVIGATION-BINDING",
            "request.navigation",
            "navigation model resref and surface must equal the tile request",
        ));
    }
    if request.material_textures.is_empty() || request.textures.is_empty() {
        return Err(error(
            "TILE-TEXTURE-MISSING",
            "request.textures",
            "tile requires at least one diffuse binding and payload",
        ));
    }
    for (index, texture) in request.textures.iter().enumerate() {
        validate_resref(
            &texture.resref,
            &format!("request.textures[{index}].resref"),
        )?;
        if !matches!(texture.resource_type, TGA_RESOURCE_TYPE | DDS_RESOURCE_TYPE)
            || texture.payload.is_empty()
        {
            return Err(error(
                "TILE-TEXTURE-INVALID",
                format!("request.textures[{index}]"),
                "tile texture must be non-empty TGA type 3 or DDS type 2033",
            ));
        }
    }
    let diffuse = request
        .textures
        .iter()
        .find(|texture| texture.resref == request.identity.texture_resref)
        .ok_or_else(|| {
            error(
                "TILE-TEXTURE-MISSING",
                "identity.textureResref",
                "identity diffuse texture payload is missing",
            )
        })?;
    if !request
        .material_textures
        .iter()
        .any(|binding| binding.resref == diffuse.resref)
    {
        return Err(error(
            "TILE-TEXTURE-BINDING",
            "request.materialTextures",
            "identity diffuse texture has no material binding",
        ));
    }
    Ok(())
}

fn validate_identity(identity: &StaticTileIdentityV1) -> Result<(), TilePackageErrorV1> {
    for (path, value) in [
        ("identity.moduleResref", identity.module_resref.as_str()),
        ("identity.areaResref", identity.area_resref.as_str()),
        ("identity.hakResref", identity.hak_resref.as_str()),
        ("identity.tilesetResref", identity.tileset_resref.as_str()),
        ("identity.modelResref", identity.model_resref.as_str()),
        ("identity.textureResref", identity.texture_resref.as_str()),
        (
            "identity.imageMapResref",
            identity.image_map_resref.as_str(),
        ),
    ] {
        validate_resref(value, path)?;
    }
    validate_file_name(
        &identity.module_file_name,
        ".mod",
        "identity.moduleFileName",
    )?;
    validate_file_name(&identity.hak_file_name, ".hak", "identity.hakFileName")?;
    for (path, value) in [
        (
            "identity.moduleDisplayName",
            identity.module_display_name.as_str(),
        ),
        ("identity.areaName", identity.area_name.as_str()),
    ] {
        if value.is_empty() || value.len() > 128 || !value.is_ascii() {
            return Err(error(
                "TILE-IDENTITY-INVALID",
                path,
                "display value must contain 1..128 ASCII bytes",
            ));
        }
    }
    Ok(())
}

fn normalize_model_root(
    model: &mut AuroraModelIrV1,
    model_resref: &str,
) -> Result<(), TilePackageErrorV1> {
    let roots = model
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(error(
            "TILE-HIERARCHY-INVALID",
            "model.nodes",
            "tile requires exactly one model root",
        ));
    }
    model.nodes[roots[0]].name = model_resref.to_owned();
    Ok(())
}

fn build_tile_ifo(identity: &StaticTileIdentityV1) -> Result<Vec<u8>, TilePackageErrorV1> {
    let base = binary_m0_module_ifo_for(
        &identity.module_resref,
        &identity.area_resref,
        &[identity.hak_resref.as_str()],
        &identity.module_display_name,
        "Generated by Meshy2Aurora for human-owned static tile proof.",
    )
    .map_err(|source| map_error("TILE-IFO-WRITE-FAILED", "module.ifo", source))?;
    let mut document = read_gff_v32(&base, &GffLimitsV1::default())
        .map_err(|source| map_error("TILE-IFO-READBACK-FAILED", "module.ifo", source))?;
    replace_field(&mut document, "Mod_Entry_X", GffValueV1::Float(5.0))?;
    replace_field(&mut document, "Mod_Entry_Y", GffValueV1::Float(5.0))?;
    replace_field(&mut document, "Mod_Entry_Z", GffValueV1::Float(0.0))?;
    write_gff_v32(&document, &GffWriterOptionsV1::default())
        .map(|artifact| artifact.payload)
        .map_err(|source| map_error("TILE-IFO-WRITE-FAILED", "module.ifo", source))
}

fn build_tile_are(identity: &StaticTileIdentityV1) -> Result<Vec<u8>, TilePackageErrorV1> {
    let base = binary_m0_area_for(&identity.area_resref)
        .map_err(|source| map_error("TILE-ARE-WRITE-FAILED", "area.are", source))?;
    let mut document = read_gff_v32(&base, &GffLimitsV1::default())
        .map_err(|source| map_error("TILE-ARE-READBACK-FAILED", "area.are", source))?;
    replace_field(
        &mut document,
        "Name",
        GffValueV1::LocString(GffLocStringV1 {
            string_ref: u32::MAX,
            substrings: vec![GffLocSubstringV1 {
                string_id: 0,
                bytes: identity.area_name.as_bytes().to_vec(),
            }],
        }),
    )?;
    replace_field(
        &mut document,
        "Comments",
        GffValueV1::String(
            b"Generated by Meshy2Aurora TileStaticV1 owner-proof pipeline.".to_vec(),
        ),
    )?;
    replace_field(&mut document, "Width", GffValueV1::Int(2))?;
    replace_field(&mut document, "Height", GffValueV1::Int(2))?;
    replace_field(
        &mut document,
        "Tileset",
        GffValueV1::ResRef(identity.tileset_resref.clone()),
    )?;
    replace_field(
        &mut document,
        "Tile_List",
        GffValueV1::List((0..4).map(|_| tile_instance()).collect()),
    )?;
    write_gff_v32(&document, &GffWriterOptionsV1::default())
        .map(|artifact| artifact.payload)
        .map_err(|source| map_error("TILE-ARE-WRITE-FAILED", "area.are", source))
}

fn tile_instance() -> GffStructV1 {
    GffStructV1 {
        struct_id: 1,
        fields: vec![
            field("Tile_ID", GffValueV1::Int(0)),
            field("Tile_Orientation", GffValueV1::Int(0)),
            field("Tile_Height", GffValueV1::Int(0)),
            field("Tile_MainLight1", GffValueV1::Byte(0)),
            field("Tile_MainLight2", GffValueV1::Byte(0)),
            field("Tile_SrcLight1", GffValueV1::Byte(0)),
            field("Tile_SrcLight2", GffValueV1::Byte(0)),
            field("Tile_AnimLoop1", GffValueV1::Byte(0)),
            field("Tile_AnimLoop2", GffValueV1::Byte(0)),
            field("Tile_AnimLoop3", GffValueV1::Byte(0)),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_package_readback(
    request: &StaticTileBuildRequestV1,
    hak_bytes: &[u8],
    module_bytes: &[u8],
    mdl_bytes: &[u8],
    wok_bytes: &[u8],
    set_bytes: &[u8],
    image_map_bytes: &[u8],
) -> Result<(), TilePackageErrorV1> {
    let identity = &request.identity;
    let hak = ErfArchive::parse(hak_bytes)
        .map_err(|source| map_error("TILE-HAK-READBACK-FAILED", "hak", source))?;
    for (resref, resource_type, expected) in [
        (
            identity.tileset_resref.as_str(),
            SET_RESOURCE_TYPE,
            set_bytes,
        ),
        (identity.model_resref.as_str(), MDL_RESOURCE_TYPE, mdl_bytes),
        (identity.model_resref.as_str(), WOK_RESOURCE_TYPE, wok_bytes),
        (
            identity.image_map_resref.as_str(),
            TGA_RESOURCE_TYPE,
            image_map_bytes,
        ),
    ] {
        let actual = hak.find(resref, resource_type).map_err(|source| {
            map_error(
                "TILE-HAK-READBACK-FAILED",
                &format!("hak.{resref}:{resource_type}"),
                source,
            )
        })?;
        if actual != expected || sha256(actual) != sha256(expected) {
            return Err(error(
                "TILE-HAK-SEMANTIC-DIFF",
                format!("hak.{resref}:{resource_type}"),
                "HAK payload or SHA-256 differs after readback",
            ));
        }
    }
    for texture in &request.textures {
        let actual = hak
            .find(&texture.resref, texture.resource_type)
            .map_err(|source| {
                map_error(
                    "TILE-HAK-READBACK-FAILED",
                    &format!("hak.{}:{}", texture.resref, texture.resource_type),
                    source,
                )
            })?;
        if actual != texture.payload || sha256(actual) != sha256(&texture.payload) {
            return Err(error(
                "TILE-HAK-SEMANTIC-DIFF",
                format!("hak.{}:{}", texture.resref, texture.resource_type),
                "texture differs after HAK readback",
            ));
        }
    }
    let mdl = inspect_binary_mdl(mdl_bytes)
        .map_err(|source| map_error("TILE-MDL-READBACK-FAILED", "hak.mdl", source))?;
    if mdl.model.classification != 2
        || flatten_nodes(&mdl.node_tree.roots)
            .iter()
            .filter(|node| node.aabb.is_some())
            .count()
            != 1
    {
        return Err(error(
            "TILE-MDL-SEMANTIC-DIFF",
            "hak.mdl",
            "tile MDL must have classification 2 and exactly one semantic AABB node",
        ));
    }
    let wok = inspect_ascii_tile_wok_v1(wok_bytes)
        .map_err(|source| map_error("TILE-WOK-READBACK-FAILED", "hak.wok", source))?;
    let tileset = parse_tileset_v1(set_bytes)
        .map_err(|source| map_error("TILE-SET-READBACK-FAILED", "hak.set", source))?;
    let binding = resolve_are_tile_v1(&tileset, 0)
        .map_err(|source| map_error("TILE-CHAIN-READBACK-FAILED", "ARE.Tile_ID", source))?;
    if binding.model_resref != identity.model_resref
        || binding.wok_resref != identity.model_resref
        || wok.model_resref != identity.model_resref
    {
        return Err(error(
            "TILE-CHAIN-SEMANTIC-DIFF",
            "ARE->SET->MDL/WOK",
            "SET Tile0.Model must bind both same-resref MDL and WOK",
        ));
    }

    let module = ErfArchive::parse(module_bytes)
        .map_err(|source| map_error("TILE-MOD-READBACK-FAILED", "module", source))?;
    let ifo = read_gff_v32(
        module
            .find("module", IFO_RESOURCE_TYPE)
            .map_err(|source| map_error("TILE-IFO-MISSING", "module.ifo", source))?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| map_error("TILE-IFO-READBACK-FAILED", "module.ifo", source))?;
    let hak_list = require_list(&ifo, "Mod_HakList")?;
    if hak_list.len() != 1
        || find_value(&hak_list[0], "Mod_Hak")
            != Some(&GffValueV1::String(identity.hak_resref.as_bytes().to_vec()))
    {
        return Err(error(
            "TILE-IFO-SEMANTIC-DIFF",
            "module.ifo.Mod_HakList",
            "module must contain exactly one ordered HAK entry",
        ));
    }
    for (label, expected) in [
        ("Mod_Entry_X", GffValueV1::Float(5.0)),
        ("Mod_Entry_Y", GffValueV1::Float(5.0)),
        ("Mod_Entry_Z", GffValueV1::Float(0.0)),
    ] {
        if find_root_value(&ifo, label) != Some(&expected) {
            return Err(error(
                "TILE-IFO-SEMANTIC-DIFF",
                format!("module.ifo.{label}"),
                "entry point differs from the walkable Tile0 center",
            ));
        }
    }
    let are = read_gff_v32(
        module
            .find(&identity.area_resref, ARE_RESOURCE_TYPE)
            .map_err(|source| map_error("TILE-ARE-MISSING", "module.are", source))?,
        &GffLimitsV1::default(),
    )
    .map_err(|source| map_error("TILE-ARE-READBACK-FAILED", "module.are", source))?;
    if find_root_value(&are, "Tileset")
        != Some(&GffValueV1::ResRef(identity.tileset_resref.clone()))
        || find_root_value(&are, "Width") != Some(&GffValueV1::Int(2))
        || find_root_value(&are, "Height") != Some(&GffValueV1::Int(2))
    {
        return Err(error(
            "TILE-ARE-SEMANTIC-DIFF",
            "module.are",
            "ARE custom tileset or 2x2 dimensions differ",
        ));
    }
    let tiles = require_list(&are, "Tile_List")?;
    if tiles.len() != 4
        || tiles.iter().any(|tile| {
            find_value(tile, "Tile_ID") != Some(&GffValueV1::Int(0))
                || find_value(tile, "Tile_Orientation") != Some(&GffValueV1::Int(0))
                || find_value(tile, "Tile_Height") != Some(&GffValueV1::Int(0))
        })
    {
        return Err(error(
            "TILE-ARE-TILE-LIST",
            "module.are.Tile_List",
            "2x2 Area requires exactly four Tile_ID=0 orientation=0 height=0 records",
        ));
    }
    Ok(())
}

fn validate_repeated_tile_seam(navigation: &TileNavigationIrV1) -> Result<(), TilePackageErrorV1> {
    let edge = |axis: usize, value: f32, orthogonal: usize| {
        let mut profile = navigation
            .vertices
            .iter()
            .filter(|vertex| vertex[axis].to_bits() == value.to_bits())
            .map(|vertex| (vertex[orthogonal].to_bits(), vertex[2].to_bits()))
            .collect::<Vec<_>>();
        profile.sort_unstable();
        profile
    };
    if edge(0, -5.0, 1) != edge(0, 5.0, 1) || edge(1, -5.0, 0) != edge(1, 5.0, 0) {
        return Err(error(
            "TILE-WOK-SEAM-MISMATCH",
            "navigation.vertices",
            "repeated tile opposite-edge height profiles differ",
        ));
    }
    Ok(())
}

fn replace_field(
    document: &mut GffDocumentV1,
    label: &str,
    value: GffValueV1,
) -> Result<(), TilePackageErrorV1> {
    let field = document
        .root
        .fields
        .iter_mut()
        .find(|field| field.label == label)
        .ok_or_else(|| {
            error(
                "TILE-GFF-FIELD-MISSING",
                label,
                "shared generated GFF base lacks required field",
            )
        })?;
    field.value = value;
    Ok(())
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

fn resource_report(container: &str, resource: &HakResourceInputV1) -> TileResourceBindingReportV1 {
    TileResourceBindingReportV1 {
        container: container.to_owned(),
        role: resource_role(resource.resource_type).to_owned(),
        resref: resource.resref.clone(),
        resource_type: resource.resource_type,
        byte_length: resource.payload.len() as u64,
        sha256: sha256(&resource.payload),
    }
}

fn resource_role(resource_type: u16) -> &'static str {
    match resource_type {
        MDL_RESOURCE_TYPE => "render_and_aabb_mdl",
        SET_RESOURCE_TYPE => "tileset_manifest",
        WOK_RESOURCE_TYPE => "tile_walkmesh",
        TGA_RESOURCE_TYPE => "texture_or_image_map",
        DDS_RESOURCE_TYPE => "texture_dds",
        IFO_RESOURCE_TYPE => "module_ifo",
        ARE_RESOURCE_TYPE => "area_are",
        GIT_RESOURCE_TYPE => "area_git",
        GIC_RESOURCE_TYPE => "area_gic",
        FAC_RESOURCE_TYPE => "factions",
        _ => "other",
    }
}

fn field(label: &str, value: GffValueV1) -> GffFieldV1 {
    GffFieldV1 {
        label: label.to_owned(),
        value,
    }
}

fn find_root_value<'a>(document: &'a GffDocumentV1, label: &str) -> Option<&'a GffValueV1> {
    find_value(&document.root, label)
}

fn find_value<'a>(structure: &'a GffStructV1, label: &str) -> Option<&'a GffValueV1> {
    structure
        .fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
}

fn require_list<'a>(
    document: &'a GffDocumentV1,
    label: &str,
) -> Result<&'a [GffStructV1], TilePackageErrorV1> {
    match find_root_value(document, label) {
        Some(GffValueV1::List(items)) => Ok(items),
        _ => Err(error(
            "TILE-GFF-LIST-MISSING",
            label,
            "required GFF list is missing or has wrong type",
        )),
    }
}

fn flatten_nodes(nodes: &[crate::mdl::NodeReport]) -> Vec<&crate::mdl::NodeReport> {
    let mut output = Vec::new();
    let mut pending = nodes.iter().rev().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        output.push(node);
        pending.extend(node.children.iter().rev());
    }
    output
}

fn validate_resref(value: &str, path: &str) -> Result<(), TilePackageErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "TILE-RESREF-INVALID",
            path,
            "resref must contain 1..16 canonical lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

fn validate_file_name(value: &str, suffix: &str, path: &str) -> Result<(), TilePackageErrorV1> {
    if !value.ends_with(suffix)
        || value.len() <= suffix.len()
        || !value.is_ascii()
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(error(
            "TILE-FILE-NAME-INVALID",
            path,
            format!("expected a plain ASCII file name ending in {suffix}"),
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
