//! Multi-resource Item HAK/MOD compositor with exact archive readback.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::erf::{ErfArchive, ErfFileType};
use crate::gff::{GffFileTypeV1, GffLimitsV1, GffStructV1, GffValueV1, read_gff_v32};
use crate::hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1, write_hak_v1};
use crate::item::{
    EffectiveResourceNamespaceV1, ITEM_RECIPE_INVALID, ITEM_SCHEMA_INVALID, ITEM_SCHEMA_VERSION,
    ItemAppearanceRecipeV1, ItemAssemblyReportV1, ItemErrorV1, ItemPartGeometryV1, ItemPartSlotV1,
    ItemResourceKeyV1, ItemResourceScopeV1, item_recipe_sha256_v1, preflight_item_namespace_v1,
    resolve_item_resource_names_v1, sha256_hex, validate_item_assembly_v1,
};
use crate::item_icon::{
    ITEM_ICON_PLT_RESOURCE_TYPE, ITEM_ICON_TGA_RESOURCE_TYPE, ItemIconFormatV1,
    ItemIconLayerArtifactV1,
};
use crate::item_part::ItemPartCompileReportV1;
use crate::item_uti::{
    ItemPaletteEntryV1, ItemUtiArtifactV1, ItemUtiBuildRequestV1, write_item_palette_itp_v1,
    write_item_uti_v1,
};
use crate::mdl::{InspectionReport, NodeReport, inspect_binary_mdl};
use crate::plt::read_plt_image_v1;
use crate::tga::read_tga_image_v1;

pub const MDL_RESOURCE_TYPE: u16 = 2002;
pub const TGA_RESOURCE_TYPE: u16 = 3;
pub const PLT_RESOURCE_TYPE: u16 = 6;
pub const IFO_RESOURCE_TYPE: u16 = 2014;
pub const ARE_RESOURCE_TYPE: u16 = 2012;
pub const GIT_RESOURCE_TYPE: u16 = 2023;
pub const GIC_RESOURCE_TYPE: u16 = 2046;
pub const UTI_RESOURCE_TYPE: u16 = 2025;
pub const UTC_RESOURCE_TYPE: u16 = 2027;
pub const ITP_RESOURCE_TYPE: u16 = 2030;

pub const ITEM_PACKAGE_INVALID: &str = "M2A-ITEM-PACKAGE-INVALID";
pub const ITEM_PACKAGE_SOURCE_STALE: &str = "M2A-ITEM-PACKAGE-SOURCE-STALE";
pub const ITEM_PACKAGE_MDL_INVALID: &str = "M2A-ITEM-PACKAGE-MDL-INVALID";
pub const ITEM_PACKAGE_ARCHIVE_INVALID: &str = "M2A-ITEM-PACKAGE-ARCHIVE-INVALID";
pub const ITEM_PACKAGE_SEMANTIC_DIFF: &str = "M2A-ITEM-PACKAGE-SEMANTIC-DIFF";

#[derive(Clone, Debug, PartialEq)]
pub struct ItemCompiledPartInputV1 {
    pub slot: ItemPartSlotV1,
    pub resref: String,
    pub source_sha256: String,
    pub payload: Vec<u8>,
    pub compile_report: ItemPartCompileReportV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemPackageBuildRequestV1 {
    pub schema_version: u32,
    pub generator_identity: String,
    pub recipe: ItemAppearanceRecipeV1,
    pub namespace: EffectiveResourceNamespaceV1,
    pub uti: ItemUtiBuildRequestV1,
    pub parts: Vec<ItemCompiledPartInputV1>,
    pub icons: Vec<ItemIconLayerArtifactV1>,
    pub additional_hak_resources: Vec<HakResourceInputV1>,
    pub module_fixture_resources: Vec<HakResourceInputV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemPackageContainerV1 {
    Standalone,
    Hak,
    Module,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPackageResourceReportV1 {
    pub container: ItemPackageContainerV1,
    pub role: String,
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPackageManifestV1 {
    pub schema_version: u32,
    pub generator_identity: String,
    pub recipe_sha256: String,
    pub baseitems_source_sha256: String,
    pub namespace_inventory_sha256: Vec<String>,
    pub profile: crate::item::ItemCompositionProfileV1,
    pub module_resref: String,
    pub area_resref: String,
    pub ordered_hak_resrefs: Vec<String>,
    pub fixture_creature_resref: Option<String>,
    pub fixture_equipment_slot: Option<u32>,
    pub assembly: ItemAssemblyReportV1,
    pub uti_sha256: String,
    pub hak_sha256: String,
    pub module_sha256: String,
    pub resources: Vec<ItemPackageResourceReportV1>,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemPackageArtifactV1 {
    pub uti: ItemUtiArtifactV1,
    pub hak_payload: Vec<u8>,
    pub module_payload: Vec<u8>,
    pub manifest_json: Vec<u8>,
    pub manifest_sha256: String,
    pub manifest: ItemPackageManifestV1,
}

pub fn write_item_package_v1(
    request: &ItemPackageBuildRequestV1,
    gff_options: &crate::gff::GffWriterOptionsV1,
    archive_options: &HakWriterOptionsV1,
) -> Result<ItemPackageArtifactV1, ItemErrorV1> {
    if request.schema_version != ITEM_SCHEMA_VERSION {
        return Err(ItemErrorV1::fatal(
            ITEM_SCHEMA_INVALID,
            "request.schemaVersion",
            format!("expected schema version {ITEM_SCHEMA_VERSION}"),
        ));
    }
    if request.generator_identity.is_empty() || request.generator_identity.len() > 128 {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_INVALID,
            "request.generatorIdentity",
            "generator identity must contain 1..128 bytes",
        ));
    }
    if request.uti.recipe != request.recipe {
        return Err(ItemErrorV1::fatal(
            ITEM_RECIPE_INVALID,
            "request.uti.recipe",
            "UTI and package must reference the same exact Item recipe",
        ));
    }
    let recipe_sha256 = item_recipe_sha256_v1(&request.recipe)?;
    let names = resolve_item_resource_names_v1(&request.recipe)?;
    let expected_names: HashMap<ItemPartSlotV1, (&str, &str)> = names
        .iter()
        .map(|name| {
            (
                name.slot,
                (name.model_resref.as_str(), name.icon_resref.as_str()),
            )
        })
        .collect();
    if request.parts.len() != names.len() || request.icons.len() != names.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_INVALID,
            "request.parts",
            "package requires exactly one MDL and one icon layer for every resolved slot",
        ));
    }

    let mut part_slots = HashSet::with_capacity(request.parts.len());
    let mut geometry = Vec::with_capacity(request.parts.len());
    let mut hak_resources = Vec::with_capacity(
        request.parts.len() + request.icons.len() + request.additional_hak_resources.len(),
    );
    let mut resource_reports = Vec::new();
    for (index, part) in request.parts.iter().enumerate() {
        if !part_slots.insert(part.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_INVALID,
                format!("request.parts[{index}].slot"),
                "duplicate compiled part slot",
            ));
        }
        let expected_resref = expected_names
            .get(&part.slot)
            .map(|names| names.0)
            .ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PACKAGE_INVALID,
                    format!("request.parts[{index}].slot"),
                    "compiled part slot does not belong to the resolved recipe",
                )
            })?;
        if part.resref != expected_resref {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_INVALID,
                format!("request.parts[{index}].resref"),
                format!("expected exact resolved model ResRef {expected_resref}"),
            ));
        }
        let actual_sha256 = sha256_hex(&part.payload);
        if part.source_sha256 != actual_sha256 {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SOURCE_STALE,
                format!("request.parts[{index}].sourceSha256"),
                format!("expected {}, read {actual_sha256}", part.source_sha256),
            ));
        }
        let recipe_part = request
            .recipe
            .parts
            .iter()
            .find(|recipe_part| recipe_part.slot == part.slot)
            .expect("validated recipe and resolved names contain the slot");
        if part.compile_report.schema_version != ITEM_SCHEMA_VERSION
            || part.compile_report.slot != part.slot
            || part.compile_report.model_resref != part.resref
            || part.compile_report.model_sha256 != actual_sha256
            || part.compile_report.source_sha256 != recipe_part.source_sha256
            || part.compile_report.recipe_sha256 != recipe_sha256
            || part.compile_report.model_classification != 4
            || part.compile_report.semantic_readback_status != "PASS"
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                format!("request.parts[{index}].compileReport"),
                "compiled part report does not bind exact recipe/source/model identities",
            ));
        }
        let inspection = inspect_binary_mdl(&part.payload).map_err(|error| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_MDL_INVALID,
                format!("request.parts[{index}].payload"),
                error.to_string(),
            )
        })?;
        validate_mdl_inspection(&inspection, expected_resref, index)?;
        if request.recipe.base_item.profile == crate::item::ItemCompositionProfileV1::ModelType1 {
            validate_model_type_1_palette_texture(&inspection, expected_resref, index)?;
        }
        let triangle_count = inspection_triangle_count(&inspection, index)?;
        if triangle_count != part.compile_report.triangle_count {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                format!("request.parts[{index}].compileReport.triangleCount"),
                "compiled part triangle count differs from its exact MDL readback",
            ));
        }
        geometry.push(ItemPartGeometryV1 {
            slot: part.slot,
            triangle_count,
        });
        hak_resources.push(HakResourceInputV1 {
            resref: part.resref.clone(),
            resource_type: MDL_RESOURCE_TYPE,
            payload: part.payload.clone(),
        });
        resource_reports.push(resource_report(
            ItemPackageContainerV1::Hak,
            "ITEM_MODEL_PART",
            &part.resref,
            MDL_RESOURCE_TYPE,
            &part.payload,
        ));
    }
    let assembly = validate_item_assembly_v1(request.recipe.base_item.profile, &geometry)?;

    if request.recipe.base_item.profile == crate::item::ItemCompositionProfileV1::ModelType1 {
        let model_resref = expected_names
            .get(&ItemPartSlotV1::Model)
            .map(|names| names.0)
            .expect("ModelType1 recipe resolves exactly one Model part");
        let mut model_plts = request.additional_hak_resources.iter().filter(|resource| {
            resource.resource_type == PLT_RESOURCE_TYPE
                && resource.resref.eq_ignore_ascii_case(model_resref)
        });
        let model_plt = model_plts.next().ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "request.additionalHakResources",
                format!(
                    "ModelType1 requires one model-named PLT resource {model_resref}:{PLT_RESOURCE_TYPE}"
                ),
            )
        })?;
        if model_plts.next().is_some() {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "request.additionalHakResources",
                format!(
                    "ModelType1 contains more than one model-named PLT resource {model_resref}:{PLT_RESOURCE_TYPE}"
                ),
            ));
        }
        read_plt_image_v1(&model_plt.payload).map_err(|source| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "request.additionalHakResources",
                format!("ModelType1 model-named PLT readback failed: {source}"),
            )
        })?;
    }

    let mut icon_slots = HashSet::with_capacity(request.icons.len());
    for (index, icon) in request.icons.iter().enumerate() {
        if !icon_slots.insert(icon.slot) {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_INVALID,
                format!("request.icons[{index}].slot"),
                "duplicate icon layer slot",
            ));
        }
        let expected_resref = expected_names
            .get(&icon.slot)
            .map(|names| names.1)
            .ok_or_else(|| {
                ItemErrorV1::fatal(
                    ITEM_PACKAGE_INVALID,
                    format!("request.icons[{index}].slot"),
                    "icon slot does not belong to the resolved recipe",
                )
            })?;
        if icon.resref != expected_resref
            || icon.report.resref != icon.resref
            || icon.report.slot != icon.slot
            || icon.report.output_sha256 != sha256_hex(&icon.payload)
            || icon.report.semantic_readback_status != "PASS"
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                format!("request.icons[{index}]"),
                "icon artifact identity or report does not match its exact payload",
            ));
        }
        let expected_format = match request.recipe.base_item.profile {
            crate::item::ItemCompositionProfileV1::ModelType1
            | crate::item::ItemCompositionProfileV1::ModelType3 => ItemIconFormatV1::Plt,
            crate::item::ItemCompositionProfileV1::ModelType0
            | crate::item::ItemCompositionProfileV1::ModelType2 => ItemIconFormatV1::Tga,
        };
        let expected_resource_type = match expected_format {
            ItemIconFormatV1::Tga => ITEM_ICON_TGA_RESOURCE_TYPE,
            ItemIconFormatV1::Plt => ITEM_ICON_PLT_RESOURCE_TYPE,
        };
        if icon.report.format != expected_format
            || icon.report.resource_type != expected_resource_type
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                format!("request.icons[{index}].report"),
                format!(
                    "profile requires {:?} resource type {expected_resource_type}",
                    expected_format
                ),
            ));
        }
        let (width, height) = match expected_format {
            ItemIconFormatV1::Tga => {
                let image = read_tga_image_v1(&icon.payload).map_err(|error| {
                    ItemErrorV1::fatal(
                        ITEM_PACKAGE_INVALID,
                        format!("request.icons[{index}].payload"),
                        error.to_string(),
                    )
                })?;
                (image.width, image.height)
            }
            ItemIconFormatV1::Plt => {
                let image = read_plt_image_v1(&icon.payload).map_err(|error| {
                    ItemErrorV1::fatal(
                        ITEM_PACKAGE_INVALID,
                        format!("request.icons[{index}].payload"),
                        error.to_string(),
                    )
                })?;
                (image.width, image.height)
            }
        };
        if width != u32::from(request.recipe.base_item.inv_slot_width) * 32
            || height != u32::from(request.recipe.base_item.inv_slot_height) * 32
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                format!("request.icons[{index}].payload"),
                "icon readback dimensions differ from baseitems.2da",
            ));
        }
        hak_resources.push(HakResourceInputV1 {
            resref: icon.resref.clone(),
            resource_type: expected_resource_type,
            payload: icon.payload.clone(),
        });
        resource_reports.push(resource_report(
            ItemPackageContainerV1::Hak,
            "ITEM_ICON_LAYER",
            &icon.resref,
            expected_resource_type,
            &icon.payload,
        ));
    }
    for resource in &request.additional_hak_resources {
        hak_resources.push(resource.clone());
        resource_reports.push(resource_report(
            ItemPackageContainerV1::Hak,
            "ADDITIONAL_ITEM_RESOURCE",
            &resource.resref,
            resource.resource_type,
            &resource.payload,
        ));
    }

    let uti = write_item_uti_v1(&request.uti, gff_options)?;
    resource_reports.push(resource_report(
        ItemPackageContainerV1::Standalone,
        "ITEM_BLUEPRINT",
        &request.recipe.identity.uti_resref,
        UTI_RESOURCE_TYPE,
        &uti.payload,
    ));
    let fixture = validate_module_fixture_resources(
        &request.module_fixture_resources,
        &request.recipe.identity.uti_resref,
        &gff_options.limits,
    )?;
    let mut module_resources = request.module_fixture_resources.clone();
    module_resources.push(HakResourceInputV1 {
        resref: request.recipe.identity.uti_resref.clone(),
        resource_type: UTI_RESOURCE_TYPE,
        payload: uti.payload.clone(),
    });
    if fixture.creature_resref.is_none() {
        let palette = write_item_palette_itp_v1(&ItemPaletteEntryV1 {
            uti_resref: request.recipe.identity.uti_resref.clone(),
            display_name: request.recipe.identity.display_name.clone(),
            palette_id: request.uti.properties.palette_id,
        })?;
        module_resources.push(HakResourceInputV1 {
            resref: "itempalcus".to_owned(),
            resource_type: ITP_RESOURCE_TYPE,
            payload: palette.payload,
        });
    }
    for resource in &module_resources {
        resource_reports.push(resource_report(
            ItemPackageContainerV1::Module,
            match resource.resource_type {
                UTI_RESOURCE_TYPE => "ITEM_BLUEPRINT",
                ITP_RESOURCE_TYPE => "ITEM_PALETTE",
                _ => "TEST_MODULE_FIXTURE",
            },
            &resource.resref,
            resource.resource_type,
            &resource.payload,
        ));
    }

    let output_keys = hak_resources
        .iter()
        .map(|resource| ItemResourceKeyV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            scope: ItemResourceScopeV1::Output,
        })
        .chain(module_resources.iter().map(|resource| ItemResourceKeyV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            scope: ItemResourceScopeV1::Output,
        }))
        .collect::<Vec<_>>();
    preflight_item_namespace_v1(&output_keys, &request.namespace)?;

    let hak = write_hak_v1(&hak_resources, archive_options).map_err(|error| {
        ItemErrorV1::fatal(ITEM_PACKAGE_ARCHIVE_INVALID, "hak", error.to_string())
    })?;
    let module = write_erf_archive_v1(ErfFileType::Module, &module_resources, archive_options)
        .map_err(|error| {
            ItemErrorV1::fatal(ITEM_PACKAGE_ARCHIVE_INVALID, "module", error.to_string())
        })?;
    verify_archive(&hak.payload, ErfFileType::Hak, &hak_resources, "hak")?;
    verify_archive(
        &module.payload,
        ErfFileType::Module,
        &module_resources,
        "module",
    )?;

    resource_reports.sort_by(|left, right| {
        (left.container as u8, &left.resref, left.resource_type).cmp(&(
            right.container as u8,
            &right.resref,
            right.resource_type,
        ))
    });
    let manifest = ItemPackageManifestV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        generator_identity: request.generator_identity.clone(),
        recipe_sha256,
        baseitems_source_sha256: request.recipe.base_item.source_sha256.clone(),
        namespace_inventory_sha256: request.namespace.inventories_sha256.clone(),
        profile: request.recipe.base_item.profile,
        module_resref: fixture.module_resref,
        area_resref: fixture.area_resref,
        ordered_hak_resrefs: fixture.ordered_hak_resrefs,
        fixture_creature_resref: fixture.creature_resref,
        fixture_equipment_slot: fixture.equipment_slot,
        assembly,
        uti_sha256: uti.report.payload_sha256.clone(),
        hak_sha256: hak.report.archive_sha256,
        module_sha256: module.report.archive_sha256,
        resources: resource_reports,
        semantic_readback_status: "PASS".to_owned(),
    };
    let manifest_json = serde_json::to_vec(&manifest).map_err(|error| {
        ItemErrorV1::fatal(
            ITEM_PACKAGE_INVALID,
            "manifest",
            format!("manifest serialization failed: {error}"),
        )
    })?;
    let manifest_sha256 = sha256_hex(&manifest_json);
    Ok(ItemPackageArtifactV1 {
        uti,
        hak_payload: hak.payload,
        module_payload: module.payload,
        manifest_json,
        manifest_sha256,
        manifest,
    })
}

fn validate_mdl_inspection(
    inspection: &InspectionReport,
    expected_resref: &str,
    index: usize,
) -> Result<(), ItemErrorV1> {
    if !inspection.model.name.eq_ignore_ascii_case(expected_resref) {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_MDL_INVALID,
            format!("request.parts[{index}].payload.model.name"),
            format!(
                "internal MDL model name {} does not match resource ResRef {expected_resref}",
                inspection.model.name
            ),
        ));
    }
    let root = inspection.node_tree.roots.first();
    if inspection.model.classification != 4
        || !inspection.animations.is_empty()
        || inspection.node_tree.roots.len() != 1
        || root.is_none_or(|root| {
            !root.name.eq_ignore_ascii_case(expected_resref)
                || !root.controllers.is_empty()
                || root.controller_keys_header.used != 0
                || root.controller_data_header.used != 0
        })
    {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_MDL_INVALID,
            format!("request.parts[{index}].payload.model.profile"),
            "MDL is not a retail-parity Item classification-4 model with one controllerless model-named root",
        ));
    }
    if !inspection.unsupported.is_empty() || !inspection.diagnostics.is_empty() {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_MDL_INVALID,
            format!("request.parts[{index}].payload"),
            "MDL inspection contains unsupported families or diagnostics",
        ));
    }
    Ok(())
}

fn validate_model_type_1_palette_texture(
    inspection: &InspectionReport,
    expected_resref: &str,
    part_index: usize,
) -> Result<(), ItemErrorV1> {
    fn visit(node: &NodeReport, expected_resref: &str, found: &mut usize) -> bool {
        if let Some(mesh) = &node.mesh
            && mesh.render != 0
            && !mesh.faces.is_empty()
        {
            *found += 1;
            if mesh
                .textures
                .first()
                .is_none_or(|texture| !texture.eq_ignore_ascii_case(expected_resref))
            {
                return false;
            }
        }
        node.children
            .iter()
            .all(|child| visit(child, expected_resref, found))
    }

    let mut found = 0usize;
    let matches = inspection
        .node_tree
        .roots
        .iter()
        .all(|root| visit(root, expected_resref, &mut found));
    if !matches || found == 0 {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            format!("request.parts[{part_index}].payload.mesh.texture0"),
            format!(
                "ModelType1 requires every render mesh to reference the single model-named PLT {expected_resref}"
            ),
        ));
    }
    Ok(())
}

fn inspection_triangle_count(
    inspection: &InspectionReport,
    part_index: usize,
) -> Result<usize, ItemErrorV1> {
    fn visit(node: &NodeReport, total: &mut usize) -> Option<()> {
        if let Some(mesh) = &node.mesh
            && mesh.render != 0
        {
            *total = total.checked_add(mesh.faces.len())?;
        }
        for child in &node.children {
            visit(child, total)?;
        }
        Some(())
    }
    let mut total = 0usize;
    for root in &inspection.node_tree.roots {
        visit(root, &mut total).ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_MDL_INVALID,
                format!("request.parts[{part_index}].payload"),
                "renderable triangle count overflow",
            )
        })?;
    }
    Ok(total)
}

struct ItemModuleFixtureReadbackV1 {
    module_resref: String,
    area_resref: String,
    ordered_hak_resrefs: Vec<String>,
    creature_resref: Option<String>,
    equipment_slot: Option<u32>,
}

fn validate_module_fixture_resources(
    resources: &[HakResourceInputV1],
    uti_resref: &str,
    limits: &GffLimitsV1,
) -> Result<ItemModuleFixtureReadbackV1, ItemErrorV1> {
    if resources
        .iter()
        .any(|resource| resource.resource_type == UTI_RESOURCE_TYPE)
    {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_INVALID,
            "request.moduleFixtureResources",
            "module fixture must not supply UTI; compositor inserts the exact generated blueprint",
        ));
    }
    let required = [
        IFO_RESOURCE_TYPE,
        ARE_RESOURCE_TYPE,
        GIT_RESOURCE_TYPE,
        GIC_RESOURCE_TYPE,
    ];
    for resource_type in required {
        if resources
            .iter()
            .filter(|resource| resource.resource_type == resource_type)
            .count()
            != 1
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_INVALID,
                "request.moduleFixtureResources",
                format!("test module fixture requires exactly one resource type {resource_type}"),
            ));
        }
    }
    let ifo = resources
        .iter()
        .find(|resource| resource.resource_type == IFO_RESOURCE_TYPE)
        .expect("required IFO checked above");
    if ifo.resref != "module" {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_INVALID,
            "request.moduleFixtureResources",
            "module IFO resource must use exact ResRef module",
        ));
    }
    let ifo = read_fixture_gff(ifo, GffFileTypeV1::Ifo, limits, "module.ifo")?;
    let module_resref = field_string(&ifo.root, "Mod_Tag", "module.ifo.Mod_Tag")?;
    let area_list = field_list(&ifo.root, "Mod_Area_list", "module.ifo.Mod_Area_list")?;
    if area_list.len() != 1 {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            "module.ifo.Mod_Area_list",
            "Item fixture requires exactly one Area",
        ));
    }
    let area_resref = field_resref(&area_list[0], "Area_Name", "module.ifo.Mod_Area_list[0]")?;
    let hak_list = field_list(&ifo.root, "Mod_HakList", "module.ifo.Mod_HakList")?;
    if hak_list.is_empty() {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            "module.ifo.Mod_HakList",
            "Item fixture must attach at least one HAK",
        ));
    }
    let ordered_hak_resrefs = hak_list
        .iter()
        .enumerate()
        .map(|(index, item)| {
            field_string(item, "Mod_Hak", &format!("module.ifo.Mod_HakList[{index}]"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if ordered_hak_resrefs
        .iter()
        .any(|resref| validate_fixture_resref(resref).is_err())
    {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            "module.ifo.Mod_HakList",
            "Item fixture HAK names must be valid lower-case ResRefs",
        ));
    }
    let area = resources
        .iter()
        .find(|resource| resource.resource_type == ARE_RESOURCE_TYPE)
        .expect("exact ARE count checked above");
    let git = resources
        .iter()
        .find(|resource| resource.resource_type == GIT_RESOURCE_TYPE)
        .expect("exact GIT count checked above");
    let gic = resources
        .iter()
        .find(|resource| resource.resource_type == GIC_RESOURCE_TYPE)
        .expect("exact GIC count checked above");
    if area.resref != area_resref || git.resref != area_resref || gic.resref != area_resref {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            "request.moduleFixtureResources",
            "IFO Area binding and ARE/GIT/GIC resource keys differ",
        ));
    }
    read_fixture_gff(area, GffFileTypeV1::Are, limits, "area.are")?;
    let git = read_fixture_gff(git, GffFileTypeV1::Git, limits, "area.git")?;
    let gic = read_fixture_gff(gic, GffFileTypeV1::Gic, limits, "area.gic")?;
    let creatures = field_list(&git.root, "Creature List", "area.git.Creature List")?;
    let gic_creatures = field_list(&gic.root, "Creature List", "area.gic.Creature List")?;
    if creatures.is_empty() {
        if !gic_creatures.is_empty() {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "area.gic.Creature List",
                "palette-only Item module requires empty GIT and GIC creature lists",
            ));
        }
        if resources
            .iter()
            .any(|resource| resource.resource_type == UTC_RESOURCE_TYPE)
        {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "module.utc",
                "palette-only Item module must not contain UTC resources",
            ));
        }
        return Ok(ItemModuleFixtureReadbackV1 {
            module_resref,
            area_resref,
            ordered_hak_resrefs,
            creature_resref: None,
            equipment_slot: None,
        });
    }
    if gic_creatures.len() != creatures.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            "area.gic.Creature List",
            "GIC creature count differs from GIT",
        ));
    }
    let (creature_resref, equipment_slot) = creatures
        .iter()
        .enumerate()
        .find_map(|(creature_index, creature)| {
            find_equipped_item(
                creature,
                uti_resref,
                &format!("area.git.Creature List[{creature_index}]"),
            )
            .transpose()
        })
        .transpose()?
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "area.git.Creature List",
                "no GIT creature equips the exact generated UTI ResRef",
            )
        })?;
    let utc_resource = resources
        .iter()
        .find(|resource| {
            resource.resource_type == UTC_RESOURCE_TYPE
                && resource.resref.eq_ignore_ascii_case(&creature_resref)
        })
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "module.utc",
                "GIT equipped creature has no matching module-local UTC",
            )
        })?;
    let utc = read_fixture_gff(utc_resource, GffFileTypeV1::Utc, limits, "creature.utc")?;
    let (utc_template, utc_slot) = find_equipped_item(&utc.root, uti_resref, "creature.utc")?
        .ok_or_else(|| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                "creature.utc.Equip_ItemList",
                "UTC does not equip the exact generated UTI ResRef",
            )
        })?;
    if !utc_template.eq_ignore_ascii_case(&creature_resref) || utc_slot != equipment_slot {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            "creature.utc",
            "GIT and UTC template/equipment slot bindings differ",
        ));
    }
    Ok(ItemModuleFixtureReadbackV1 {
        module_resref,
        area_resref,
        ordered_hak_resrefs,
        creature_resref: Some(creature_resref),
        equipment_slot: Some(equipment_slot),
    })
}

fn read_fixture_gff(
    resource: &HakResourceInputV1,
    expected_type: GffFileTypeV1,
    limits: &GffLimitsV1,
    path: &str,
) -> Result<crate::gff::GffDocumentV1, ItemErrorV1> {
    let document = read_gff_v32(&resource.payload, limits).map_err(|source| {
        ItemErrorV1::fatal(
            ITEM_PACKAGE_INVALID,
            path,
            format!("fixture GFF read failed: {source}"),
        )
    })?;
    if document.file_type != expected_type {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            path,
            "fixture GFF file type differs from its resource role",
        ));
    }
    Ok(document)
}

fn field<'a>(
    root: &'a GffStructV1,
    label: &str,
    path: &str,
) -> Result<&'a GffValueV1, ItemErrorV1> {
    let matches = root
        .fields
        .iter()
        .filter(|field| field.label == label)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            path,
            format!("expected exactly one {label} field"),
        ));
    }
    Ok(&matches[0].value)
}

fn field_list<'a>(
    root: &'a GffStructV1,
    label: &str,
    path: &str,
) -> Result<&'a [GffStructV1], ItemErrorV1> {
    match field(root, label, path)? {
        GffValueV1::List(items) => Ok(items),
        _ => Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            path,
            format!("{label} must be a GFF List"),
        )),
    }
}

fn field_resref(root: &GffStructV1, label: &str, path: &str) -> Result<String, ItemErrorV1> {
    match field(root, label, path)? {
        GffValueV1::ResRef(value) => Ok(value.clone()),
        _ => Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            path,
            format!("{label} must be a GFF ResRef"),
        )),
    }
}

fn field_string(root: &GffStructV1, label: &str, path: &str) -> Result<String, ItemErrorV1> {
    match field(root, label, path)? {
        GffValueV1::String(value) => String::from_utf8(value.clone()).map_err(|_| {
            ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                path,
                format!("{label} is not valid UTF-8"),
            )
        }),
        _ => Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            path,
            format!("{label} must be a GFF String"),
        )),
    }
}

fn find_equipped_item(
    creature: &GffStructV1,
    uti_resref: &str,
    path: &str,
) -> Result<Option<(String, u32)>, ItemErrorV1> {
    let template = field_resref(
        creature,
        "TemplateResRef",
        &format!("{path}.TemplateResRef"),
    )?;
    let equipment = field_list(
        creature,
        "Equip_ItemList",
        &format!("{path}.Equip_ItemList"),
    )?;
    let mut matches = equipment.iter().filter(|entry| {
        matches!(
            entry.fields.iter().find(|field| field.label == "EquippedRes").map(|field| &field.value),
            Some(GffValueV1::ResRef(value)) if value.eq_ignore_ascii_case(uti_resref)
        )
    });
    let Some(entry) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            format!("{path}.Equip_ItemList"),
            "same Item UTI is equipped more than once",
        ));
    }
    Ok(Some((template, entry.struct_id)))
}

fn validate_fixture_resref(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        Err(())
    } else {
        Ok(())
    }
}

fn verify_archive(
    bytes: &[u8],
    expected_file_type: ErfFileType,
    expected: &[HakResourceInputV1],
    path: &str,
) -> Result<(), ItemErrorV1> {
    let archive = ErfArchive::parse(bytes).map_err(|error| {
        ItemErrorV1::fatal(ITEM_PACKAGE_ARCHIVE_INVALID, path, error.to_string())
    })?;
    if archive.file_type() != expected_file_type || archive.resources().len() != expected.len() {
        return Err(ItemErrorV1::fatal(
            ITEM_PACKAGE_SEMANTIC_DIFF,
            path,
            "archive type or resource count differs from the exact request",
        ));
    }
    for resource in expected {
        let readback = archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| {
                ItemErrorV1::fatal(ITEM_PACKAGE_SEMANTIC_DIFF, path, error.to_string())
            })?;
        if readback != resource.payload {
            return Err(ItemErrorV1::fatal(
                ITEM_PACKAGE_SEMANTIC_DIFF,
                path,
                format!(
                    "resource {} type {} differs after archive readback",
                    resource.resref, resource.resource_type
                ),
            ));
        }
    }
    Ok(())
}

fn resource_report(
    container: ItemPackageContainerV1,
    role: &str,
    resref: &str,
    resource_type: u16,
    payload: &[u8],
) -> ItemPackageResourceReportV1 {
    ItemPackageResourceReportV1 {
        container,
        role: role.to_owned(),
        resref: resref.to_owned(),
        resource_type,
        byte_length: payload.len() as u64,
        sha256: sha256_hex(payload),
    }
}
