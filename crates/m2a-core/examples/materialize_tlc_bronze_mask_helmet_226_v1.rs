use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use m2a_core::{
    erf::ErfArchive,
    gff::{
        GffDocumentV1, GffFieldV1, GffFileTypeV1, GffLocStringV1, GffLocSubstringV1, GffStructV1,
        GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32,
    },
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakResourceInputV1, HakWriterOptionsV1},
    item::{
        EffectiveResourceNamespaceV1, ITEM_SCHEMA_VERSION, ItemAppearanceRecipeV1,
        ItemBaseRecordV1, ItemColorRecipeV1, ItemCompositionProfileV1, ItemIdentityV1,
        ItemPartRecipeV1, ItemPartSlotV1, ItemPartTransformV1, ItemResourceKeyV1,
        ItemResourceScopeV1, item_recipe_sha256_v1, resolve_item_base_record_v1,
        resolve_item_resource_names_v1,
    },
    item_icon::{
        ITEM_ICON_PLT_RESOURCE_TYPE, ItemPltIconComposeOptionsV1, ItemPltIconLayerInputV1,
        ItemPltIconMaterialMapV1, compose_item_plt_icon_from_material_id_v1,
        write_item_plt_icon_layers_v1,
    },
    item_package::{
        IFO_RESOURCE_TYPE, ITP_RESOURCE_TYPE, ItemCompiledPartInputV1, ItemPackageBuildRequestV1,
        PLT_RESOURCE_TYPE, UTC_RESOURCE_TYPE, UTI_RESOURCE_TYPE, write_item_package_v1,
    },
    item_part::{
        MeshyItemPartCompileRequestV1, compile_meshy_static_item_part_v1,
        static_item_profile_a_options_v1,
    },
    item_uti::{ItemUtiBuildRequestV1, ItemUtiPropertiesV1, read_item_uti_v1},
    mdl::MdlMaterialTextureBindingV1,
    model_pipeline::resolve_base_color_image_index_v1,
    plt::{
        PLT_TRANSPARENT_PIXEL_V1, PltImageV1, PltPixelV1, PltWriterOptionsV1,
        plt_image_pixels_sha256_v1, read_plt_image_v1, write_plt_v1,
    },
    profile_a::{convert_profile_a_exact_v1, derive_meshy_m0_static_rigid_profile_v1},
    tga::{TgaImageV1, TgaPixelFormatV1},
    two_da::TwoDaLimitsV1,
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

const SOURCE_SHA256: &str = "f9a6349da28a4c270d3670ac6d507af33ad58a9e0269d63479a4922acd431062";
const OWNER_SOURCE_SHA256: &str =
    "c2f4370e24fe866bc9303f7b4264a389f7a2d940bb7baffa2c43b00b414444bd";
const MAPPING_SHA256: &str = "4d72413702b57f4294890fc31c067d6cf32362a463a3eea1fb99f8ac99fdc66f";
const ICON_BEAUTY_SHA256: &str = "ee96ac0955380301db87f33a785629be405433eedce2fd0d37e86a9c3e46f551";
const ICON_MATERIAL_ID_SHA256: &str =
    "b26a75d0a2c553e93070b0e6bc8c97551bffa4a320bce0d57dc26f81e908bc1c";
const SOURCE_TRIANGLES: usize = 42_465;
const COMPILED_TRIANGLES: usize = 42_465;
const SOURCE_MODULE_SHA256: &str =
    "389405717b3735acbdae9b29b8b51819526bc35f17795937df177dd7e563d6bd";
const BASEITEMS_HAK_SHA256: &str =
    "c4869ae516a8598f0db5e31b473abba9b15b3b3f04880eb02f4a3a37191a9e8f";
const BASEITEMS_SHA256: &str = "b2ba08aa55c185642c8973859b9d0dbf486bc8411a630a0a10a9f77e84d701c6";
const FROZEN_NAMESPACE_SHA256: &str =
    "7852412c740c9ed48bfbd06cb19042ad9ebc900e83eac4d97bcc40de159b0409";
const VARIANT: u8 = 226;
const MODEL_RESREF: &str = "helm_226";
const ICON_RESREF: &str = "ihelm_226";
const UTI_RESREF: &str = "m2bronmask226";
const HAK_RESREF: &str = "m2bronh226v1";
const MODULE_RESREF: &str = "m2bronpal226";
const MODULE_FILENAME: &str = "m2bronpal226.mod";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora Bronze Mask Palette 226 V1";
const AREA_DISPLAY_NAME: &str = "Bronze Mask Item Palette 226 V1";
const TABLE_HAK_RESREF: &str = "lc_2da";
const TWO_DA_RESOURCE_TYPE: u16 = 2017;

const DONOR_BOUNDS_MIN: [f32; 3] = [-0.126_248_8, -0.159_788_1, -0.111_795_1];
const DONOR_BOUNDS_MAX: [f32; 3] = [0.126_248_8, 0.166_394_5, 0.224_525_4];
const DONOR_WIDTH_OCCUPANCY: f32 = 0.96;
const TOP_CLEARANCE_METERS: f32 = 0.005;
const DEFAULT_METAL1_ROW: u8 = 23;
const DEFAULT_CLOTH1_ROW: u8 = 23;
const DEFAULT_CLOTH2_ROW: u8 = 23;
const DEFAULT_LEATHER1_ROW: u8 = 23;
const DEFAULT_LEATHER2_ROW: u8 = 23;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrozenNamespaceV1 {
    resources: Vec<ItemResourceKeyV1>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        env::var("M2A_REPO_ROOT").unwrap_or_else(|_| r"C:\Projects\meshy2aurora".to_owned()),
    );
    let source_path =
        root.join("sample-3d/tlc-bronze-mask-tripo-20260905-v2/source-palette-atlas-v2.glb");
    let icon_source_path = root.join(
        "artifacts/diagnostics/tlc-bronze-mask-tripo-20260905-v2-offline-qa/icon-source-v3/icon-beauty.png",
    );
    let icon_material_id_path = root.join(
        "artifacts/diagnostics/tlc-bronze-mask-tripo-20260905-v2-offline-qa/icon-source-v3/icon-material-id.png",
    );
    let source_module_path = root.join(
        "proof-output/tlc-sandstorm-mask-helmet-palette-only-221-r2-20260904/m2assmpal221r2.mod",
    );
    let namespace_path = root
        .join("proof-output/tlc-sandstorm-mask-helmet-221-v1-20260904/namespace-inventory.json");
    let output_root = env::var("M2A_BRONZE_MASK_226_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            root.join("proof-output/tlc-bronze-mask-helmet-five-material-226-v1-20260905")
        });
    if output_root.exists() {
        return Err(format!("immutable output already exists: {}", output_root.display()).into());
    }

    let source_glb = read_exact_hash(&source_path, SOURCE_SHA256)?;
    let glb_limits = GlbLimits::default();
    let ingest = ingest_glb(&source_glb, &glb_limits)?;
    if ingest.report.statistics.triangle_count != SOURCE_TRIANGLES {
        return Err(format!(
            "expected exact {SOURCE_TRIANGLES}-triangle owner-selected source, got {}",
            ingest.report.statistics.triangle_count
        )
        .into());
    }
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest)?;
    let conversion =
        convert_profile_a_exact_v1(&ingest, &rig, &static_item_profile_a_options_v1())?;
    if !conversion.report.conversion_eligible {
        return Err(format!(
            "Bronze Mask 226 source failed exact static Item Profile A admission: {}",
            serde_json::to_string_pretty(&conversion.report)?
        )
        .into());
    }
    let model = conversion
        .creature
        .as_ref()
        .ok_or("eligible Item conversion has no common model IR")?;
    let source_bounds = model_bounds(model)?;
    let transform = donor_width_fit_transform(source_bounds)?;
    let fitted_bounds = transformed_bounds(source_bounds, transform);
    if (fitted_bounds.1[2] - (DONOR_BOUNDS_MAX[2] - TOP_CLEARANCE_METERS)).abs() > 0.000_01 {
        return Err(
            "Bronze Mask 226 fitted top differs from the accepted donor-top-fit policy".into(),
        );
    }

    let table_hak_path =
        PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak\lc_2da.hak");
    let table_hak_bytes = read_exact_hash(&table_hak_path, BASEITEMS_HAK_SHA256)?;
    let table_hak = ErfArchive::parse(&table_hak_bytes)?;
    let baseitems_bytes = table_hak.find("baseitems", TWO_DA_RESOURCE_TYPE)?;
    if sha256_hex(baseitems_bytes) != BASEITEMS_SHA256 {
        return Err("lc_2da baseitems.2da differs from the frozen source table".into());
    }
    let base_item = resolve_item_base_record_v1(
        baseitems_bytes,
        17,
        BASEITEMS_SHA256,
        &TwoDaLimitsV1::default(),
    )?;
    validate_helmet_baseitem(&base_item)?;

    let recipe = ItemAppearanceRecipeV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        base_item,
        identity: ItemIdentityV1 {
            uti_resref: UTI_RESREF.to_owned(),
            tag: "M2A_BRONZE_MASK_226_V1".to_owned(),
            display_name: "Brązowa Maska 226".to_owned(),
        },
        gender: None,
        parts: vec![ItemPartRecipeV1 {
            slot: ItemPartSlotV1::Model,
            variant: VARIANT,
            source_part_id: "tlc-bronze-mask-tripo-20260905-v2/source-palette-atlas-v2.glb"
                .to_owned(),
            source_sha256: SOURCE_SHA256.to_owned(),
            transform,
        }],
        colors: Some(ItemColorRecipeV1 {
            leather1: DEFAULT_LEATHER1_ROW,
            leather2: DEFAULT_LEATHER2_ROW,
            cloth1: DEFAULT_CLOTH1_ROW,
            cloth2: DEFAULT_CLOTH2_ROW,
            metal1: DEFAULT_METAL1_ROW,
            metal2: 0,
        }),
    };
    let names = resolve_item_resource_names_v1(&recipe)?;
    if names.len() != 1
        || names[0].model_resref != MODEL_RESREF
        || names[0].icon_resref != ICON_RESREF
    {
        return Err("helmet naming did not resolve to the exact variant-226 namespace".into());
    }
    let recipe_sha256 = item_recipe_sha256_v1(&recipe)?;

    eprintln!("compiling {MODEL_RESREF} from {SOURCE_TRIANGLES} source triangles");
    let compiled = compile_meshy_static_item_part_v1(
        &source_glb,
        &MeshyItemPartCompileRequestV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            recipe_sha256: recipe_sha256.clone(),
            part: recipe.parts[0].clone(),
            model_resref: MODEL_RESREF.to_owned(),
            material_textures: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: MODEL_RESREF.to_owned(),
            }],
        },
    )?;
    if compiled.report.triangle_count != COMPILED_TRIANGLES
        || compiled.inspection.model.classification != 4
        || compiled.report.segmentation.output_segment_count <= 1
    {
        return Err("Bronze Mask 226 MDL did not preserve the sanitized geometry, class and stream partitioning".into());
    }

    let texture_selection = resolve_base_color_image_index_v1(&ingest, model)?;
    let diffuse_image = decode_embedded_image_to_tga_v1(
        &source_glb,
        texture_selection.source_image_index,
        &glb_limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )?;
    let model_plt_image = build_model_plt(&diffuse_image)?;
    let model_plt = write_plt_v1(&model_plt_image, &PltWriterOptionsV1::default())?;
    let model_layer_counts = layer_counts(&model_plt_image);
    require_material_layers(&model_layer_counts, "model")?;

    let icon_plt_image = build_icon_plt(&icon_source_path, &icon_material_id_path)?;
    let transparent_icon_pixels = icon_plt_image
        .pixels
        .iter()
        .filter(|pixel| **pixel == PLT_TRANSPARENT_PIXEL_V1)
        .count();
    if transparent_icon_pixels == 0 || !icon_border_is_transparent(&icon_plt_image) {
        return Err("Bronze Mask 226 icon PLT has no exact transparent background border".into());
    }
    let icon_layer_counts = layer_counts(&icon_plt_image);
    require_material_layers(&icon_layer_counts, "icon")?;
    let icons = write_item_plt_icon_layers_v1(
        &recipe,
        &[ItemPltIconLayerInputV1 {
            slot: ItemPartSlotV1::Model,
            resref: ICON_RESREF.to_owned(),
            source_sha256: plt_image_pixels_sha256_v1(&icon_plt_image),
            image: icon_plt_image.clone(),
        }],
        &PltWriterOptionsV1::default(),
    )?;

    let namespace_bytes = read_exact_hash(&namespace_path, FROZEN_NAMESPACE_SHA256)?;
    let frozen_namespace: FrozenNamespaceV1 = serde_json::from_slice(&namespace_bytes)?;
    let mut effective_resources = frozen_namespace
        .resources
        .into_iter()
        .filter(|resource| resource.scope == ItemResourceScopeV1::Retail)
        .collect::<Vec<_>>();
    effective_resources.extend(
        table_hak
            .resources()
            .iter()
            .map(|resource| ItemResourceKeyV1 {
                resref: resource.resref.clone(),
                resource_type: resource.resource_type,
                scope: ItemResourceScopeV1::Hak,
            }),
    );
    effective_resources.sort_by(|left, right| {
        (&left.resref, left.resource_type).cmp(&(&right.resref, right.resource_type))
    });
    effective_resources.dedup_by(|left, right| {
        left.resref.eq_ignore_ascii_case(&right.resref) && left.resource_type == right.resource_type
    });
    let effective_namespace_json = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 1,
        "policy": "RETAIL_AND_OVERRIDE_FROM_FROZEN_PRE_R1_SCAN_PLUS_EXACT_ATTACHED_LC_2DA; UNATTACHED_LOCAL_HAKS_EXCLUDED",
        "frozenInventorySha256": FROZEN_NAMESPACE_SHA256,
        "attachedTableHakSha256": BASEITEMS_HAK_SHA256,
        "resources": effective_resources,
    }))?;
    let namespace = EffectiveResourceNamespaceV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        complete: true,
        inventories_sha256: vec![sha256_hex(&effective_namespace_json)],
        resources: effective_resources,
    };

    let source_module = read_exact_hash(&source_module_path, SOURCE_MODULE_SHA256)?;
    let module_fixture_resources = build_helmet_226_module_fixture(&source_module)?;
    let package = write_item_package_v1(
        &ItemPackageBuildRequestV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            generator_identity: "materialize_tlc_bronze_mask_helmet_226_v1".to_owned(),
            recipe: recipe.clone(),
            namespace,
            uti: ItemUtiBuildRequestV1 {
                schema_version: ITEM_SCHEMA_VERSION,
                recipe: recipe.clone(),
                properties: ItemUtiPropertiesV1 {
                    description: "Kolorowalna brązowa maska: kaptur Cloth 1, oczy Cloth 2, maska Metal 1 i paski Leather 1/2.".to_owned(),
                    identified_description: "Wariant 226 V1 z pięcioma jawnie przypisanymi regionami jednego atlasu PLT.".to_owned(),
                    cost: 1,
                    comment: "TLC Bronze Mask palette-only candidate 226 V1.".to_owned(),
                    palette_id: 9,
                    ..ItemUtiPropertiesV1::default()
                },
            },
            parts: vec![ItemCompiledPartInputV1 {
                slot: ItemPartSlotV1::Model,
                resref: MODEL_RESREF.to_owned(),
                source_sha256: sha256_hex(&compiled.payload),
                payload: compiled.payload.clone(),
                compile_report: compiled.report.clone(),
            }],
            icons: icons.clone(),
            additional_hak_resources: vec![HakResourceInputV1 {
                resref: MODEL_RESREF.to_owned(),
                resource_type: PLT_RESOURCE_TYPE,
                payload: model_plt.payload.clone(),
            }],
            module_fixture_resources,
        },
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )?;
    if package.manifest.fixture_creature_resref.is_some()
        || package.manifest.fixture_equipment_slot.is_some()
        || package.manifest.ordered_hak_resrefs != [HAK_RESREF, TABLE_HAK_RESREF]
        || package.manifest.semantic_readback_status != "PASS"
    {
        return Err(
            "Bronze Mask 226 package is not the exact palette-only ordered-HAK fixture".into(),
        );
    }
    validate_package_readback(&package.hak_payload, &package.module_payload)?;

    let offline_report = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "lineageDecision": {
            "previousHelm225V1": "owner_visible_with_rejected_inventory_icon",
            "relationship": "distinct_owner_selected_five_material_source_lineage",
        },
        "auroraDecompContract": {
            "modelType": 1,
            "modelTextureResourceType": PLT_RESOURCE_TYPE,
            "iconResourceType": ITEM_ICON_PLT_RESOURCE_TYPE,
            "paletteLayerMap": {"metal1": 2, "metal2": 3, "cloth1": 4, "cloth2": 5, "leather1": 6, "leather2": 7},
            "femaleHumanHelmetScale": 0.85,
            "perVariantAutoNormalization": false,
            "modelTextureContract": "one_model_named_plt",
        },
        "candidate": {
            "moduleFilename": MODULE_FILENAME,
            "moduleResref": MODULE_RESREF,
            "moduleDisplayName": MODULE_DISPLAY_NAME,
            "areaName": AREA_DISPLAY_NAME,
            "orderedHakResrefs": [HAK_RESREF, TABLE_HAK_RESREF],
            "modelResref": MODEL_RESREF,
            "iconResref": ICON_RESREF,
            "utiResref": UTI_RESREF,
        },
        "source": {
            "path": source_path.to_string_lossy(),
            "sha256": SOURCE_SHA256,
            "byteLength": source_glb.len(),
            "triangleCount": SOURCE_TRIANGLES,
            "ownerFiveMaterialSourceSha256": OWNER_SOURCE_SHA256,
            "mappingSha256": MAPPING_SHA256,
            "removedDegenerateTriangleCount": 1,
            "spatialWeldedComponentCount": 1,
            "spatialWeldedBoundaryEdgeCount": 473,
            "spatialWeldedNonManifoldEdgeCount": 596,
        },
        "fit": {
            "donor": "bdhd_items:helm_035",
            "donorBoundsMin": DONOR_BOUNDS_MIN,
            "donorBoundsMax": DONOR_BOUNDS_MAX,
            "sourceBoundsMin": source_bounds.0,
            "sourceBoundsMax": source_bounds.1,
            "uniformScale": transform.scale,
            "translation": transform.translation,
            "fittedBoundsMin": fitted_bounds.0,
            "fittedBoundsMax": fitted_bounds.1,
            "donorWidthOccupancy": DONOR_WIDTH_OCCUPANCY,
            "topClearanceMeters": TOP_CLEARANCE_METERS,
            "verticalPlacementPolicy": "accepted_donor_top_fit",
            "independentAxisScalingApplied": false,
        },
        "paletteAtlas": {
            "dimensions": [2048, 2048],
            "modelTextureResref": MODEL_RESREF,
            "materialSlotCount": 1,
            "materialMappings": [
                {"sourceMaterialId": 0, "semantic": "cloth-cowl", "layer": "Cloth 1", "layerIndex": 4, "pixelRectTopLeft": [0, 0, 1024, 1024]},
                {"sourceMaterialId": 1, "semantic": "viewer-right-mask-strip", "layer": "Leather 2", "layerIndex": 7, "pixelRectTopLeft": [1024, 0, 1024, 1024]},
                {"sourceMaterialId": 2, "semantic": "main-mask-shell", "layer": "Metal 1", "layerIndex": 2, "pixelRectTopLeft": [0, 1024, 512, 512]},
                {"sourceMaterialId": 3, "semantic": "eye-inserts", "layer": "Cloth 2", "layerIndex": 5, "pixelRectTopLeft": [512, 1024, 512, 512]},
                {"sourceMaterialId": 4, "semantic": "viewer-left-mask-strip", "layer": "Leather 1", "layerIndex": 6, "pixelRectTopLeft": [1024, 1024, 512, 512]},
            ],
            "policy": "five_source_material_regions_remapped_to_one_model_named_plt",
        },
        "model": {
            "sha256": sha256_hex(&compiled.payload),
            "triangleCount": compiled.report.triangle_count,
            "outputSegmentCount": compiled.report.segmentation.output_segment_count,
            "semanticReadbackStatus": compiled.report.semantic_readback_status,
        },
        "modelPlt": {
            "sha256": model_plt.report.output_sha256,
            "width": model_plt_image.width,
            "height": model_plt_image.height,
            "numLayers": model_plt_image.num_layers,
            "usedLayerPixelCounts": model_layer_counts,
        },
        "iconPlt": {
            "sha256": icons[0].report.output_sha256,
            "width": icon_plt_image.width,
            "height": icon_plt_image.height,
            "numLayers": icon_plt_image.num_layers,
            "transparentPixelCount": transparent_icon_pixels,
            "usedLayerPixelCounts": icon_layer_counts,
            "transparentBorder": true,
            "beautySourceSha256": ICON_BEAUTY_SHA256,
            "materialIdSourceSha256": ICON_MATERIAL_ID_SHA256,
            "backgroundPolicy": "material_id_alpha_only",
            "paddingPixels": 5,
        },
        "defaultColors": {
            "metal1": DEFAULT_METAL1_ROW,
            "cloth1": DEFAULT_CLOTH1_ROW,
            "cloth2": DEFAULT_CLOTH2_ROW,
            "leather1": DEFAULT_LEATHER1_ROW,
            "leather2": DEFAULT_LEATHER2_ROW,
        },
        "package": package.manifest,
        "nativeInstallation": "not_performed_by_materializer",
        "liveToolsetOrNwn": "not_started_human_owned_final_proof",
    }))?;

    fs::create_dir_all(&output_root)?;
    write_new(
        &output_root.join(format!("{MODEL_RESREF}.mdl")),
        &compiled.payload,
    )?;
    write_new(
        &output_root.join(format!("{MODEL_RESREF}.plt")),
        &model_plt.payload,
    )?;
    write_new(
        &output_root.join(format!("{ICON_RESREF}.plt")),
        &icons[0].payload,
    )?;
    write_new(
        &output_root.join(format!("{UTI_RESREF}.uti")),
        &package.uti.payload,
    )?;
    write_new(
        &output_root.join(format!("{HAK_RESREF}.hak")),
        &package.hak_payload,
    )?;
    write_new(&output_root.join(MODULE_FILENAME), &package.module_payload)?;
    write_new(&output_root.join("manifest.json"), &package.manifest_json)?;
    write_new(
        &output_root.join("recipe.json"),
        &serde_json::to_vec_pretty(&recipe)?,
    )?;
    write_new(
        &output_root.join("effective-namespace.json"),
        &effective_namespace_json,
    )?;
    write_new(&output_root.join("offline-report.json"), &offline_report)?;
    println!("{}", String::from_utf8(offline_report)?);
    Ok(())
}

fn validate_helmet_baseitem(base: &ItemBaseRecordV1) -> Result<(), Box<dyn std::error::Error>> {
    if base.physical_row_index != 17
        || base.profile != ItemCompositionProfileV1::ModelType1
        || base.model_type != 1
        || !base.item_class.eq_ignore_ascii_case("helm")
        || base.gender_specific
        || base.inv_slot_width != 2
        || base.inv_slot_height != 2
    {
        return Err("effective baseitems row 17 is not the audited helmet contract".into());
    }
    Ok(())
}

fn model_bounds(
    model: &m2a_core::model_ir::AuroraModelIrV1,
) -> Result<([f32; 3], [f32; 3]), Box<dyn std::error::Error>> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    let mut count = 0usize;
    for position in model
        .segments
        .iter()
        .flat_map(|segment| segment.positions.iter())
    {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
        count += 1;
    }
    if count == 0 || min.iter().chain(max.iter()).any(|value| !value.is_finite()) {
        return Err("converted Item model has no finite positions".into());
    }
    Ok((min, max))
}

fn donor_width_fit_transform(
    source: ([f32; 3], [f32; 3]),
) -> Result<ItemPartTransformV1, Box<dyn std::error::Error>> {
    let source_width = source.1[0] - source.0[0];
    if !source_width.is_finite() || source_width <= 0.0 {
        return Err("source width is degenerate".into());
    }
    let donor_width = DONOR_BOUNDS_MAX[0] - DONOR_BOUNDS_MIN[0];
    let scale = donor_width * DONOR_WIDTH_OCCUPANCY / source_width;
    let source_center_x = (source.0[0] + source.1[0]) * 0.5;
    let source_center_y = (source.0[1] + source.1[1]) * 0.5;
    let donor_center_x = (DONOR_BOUNDS_MIN[0] + DONOR_BOUNDS_MAX[0]) * 0.5;
    let donor_center_y = (DONOR_BOUNDS_MIN[1] + DONOR_BOUNDS_MAX[1]) * 0.5;
    Ok(ItemPartTransformV1 {
        translation: [
            donor_center_x - source_center_x * scale,
            donor_center_y - source_center_y * scale,
            DONOR_BOUNDS_MAX[2] - TOP_CLEARANCE_METERS - source.1[2] * scale,
        ],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale,
    })
}

fn transformed_bounds(
    source: ([f32; 3], [f32; 3]),
    transform: ItemPartTransformV1,
) -> ([f32; 3], [f32; 3]) {
    (
        std::array::from_fn(|axis| source.0[axis] * transform.scale + transform.translation[axis]),
        std::array::from_fn(|axis| source.1[axis] * transform.scale + transform.translation[axis]),
    )
}

fn build_model_plt(source: &TgaImageV1) -> Result<PltImageV1, Box<dyn std::error::Error>> {
    let channels = match source.pixel_format {
        TgaPixelFormatV1::Rgb8 => 3,
        TgaPixelFormatV1::Rgba8 => 4,
    };
    let expected = usize::try_from(source.width)?
        .checked_mul(usize::try_from(source.height)?)
        .and_then(|value| value.checked_mul(channels))
        .ok_or("source texture dimensions overflow")?;
    if source.pixels.len() != expected {
        return Err("source texture pixel length differs from dimensions".into());
    }
    if source.width != 2048 || source.height != 2048 {
        return Err("palette atlas must be exactly 2048x2048".into());
    }
    let pixels = source
        .pixels
        .chunks_exact(channels)
        .enumerate()
        .map(|(index, rgba)| {
            if channels == 4 && rgba[3] < 64 {
                PLT_TRANSPARENT_PIXEL_V1
            } else {
                let x = index % source.width as usize;
                let y = index / source.width as usize;
                let layer_index = if y < 1024 && x < 1024 {
                    Some(4)
                } else if y < 1024 {
                    Some(7)
                } else if y < 1536 && x < 512 {
                    Some(2)
                } else if y < 1536 && x < 1024 {
                    Some(5)
                } else if y < 1536 && x < 1536 {
                    Some(6)
                } else {
                    None
                };
                layer_index.map_or(PLT_TRANSPARENT_PIXEL_V1, |layer_index| {
                    material_pixel_on_layer(rgba[0], rgba[1], rgba[2], layer_index)
                })
            }
        })
        .collect();
    Ok(PltImageV1 {
        schema_version: 1,
        width: source.width,
        height: source.height,
        num_layers: 8,
        pixels,
    })
}

fn build_icon_plt(
    path: &Path,
    material_id_path: &Path,
) -> Result<PltImageV1, Box<dyn std::error::Error>> {
    let beauty_bytes = read_exact_hash(path, ICON_BEAUTY_SHA256)?;
    let material_id_bytes = read_exact_hash(material_id_path, ICON_MATERIAL_ID_SHA256)?;
    let beauty = image::load_from_memory(&beauty_bytes)?.into_rgba8();
    let material_ids = image::load_from_memory(&material_id_bytes)?.into_rgba8();
    let beauty = TgaImageV1 {
        schema_version: 1,
        width: beauty.width(),
        height: beauty.height(),
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: beauty.into_raw(),
    };
    let material_ids = TgaImageV1 {
        schema_version: 1,
        width: material_ids.width(),
        height: material_ids.height(),
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: material_ids.into_raw(),
    };
    Ok(compose_item_plt_icon_from_material_id_v1(
        &beauty,
        &material_ids,
        &[
            ItemPltIconMaterialMapV1 {
                id_rgb: [255, 0, 0],
                layer_index: 4,
            },
            ItemPltIconMaterialMapV1 {
                id_rgb: [0, 255, 0],
                layer_index: 7,
            },
            ItemPltIconMaterialMapV1 {
                id_rgb: [0, 0, 255],
                layer_index: 2,
            },
            ItemPltIconMaterialMapV1 {
                id_rgb: [255, 255, 0],
                layer_index: 5,
            },
            ItemPltIconMaterialMapV1 {
                id_rgb: [255, 0, 255],
                layer_index: 6,
            },
        ],
        &ItemPltIconComposeOptionsV1::default(),
    )?)
}

fn material_pixel_on_layer(red: u8, green: u8, blue: u8, layer_index: u8) -> PltPixelV1 {
    let luma = (54u32 * u32::from(red) + 183u32 * u32::from(green) + 19u32 * u32::from(blue)) / 256;
    let color_index = ((luma * 174 + 127) / 255) as u8;
    PltPixelV1 {
        color_index,
        layer_index,
    }
}

fn layer_counts(image: &PltImageV1) -> [usize; 10] {
    let mut counts = [0usize; 10];
    for pixel in &image.pixels {
        counts[usize::from(pixel.layer_index)] += 1;
    }
    counts
}

fn require_material_layers(
    counts: &[usize; 10],
    role: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if [2usize, 4, 5, 6, 7]
        .into_iter()
        .any(|layer| counts[layer] == 0)
    {
        return Err(format!(
            "{role} PLT does not contain Metal 1, Cloth 1/2 and Leather 1/2 pixels"
        )
        .into());
    }
    Ok(())
}

fn icon_border_is_transparent(image: &PltImageV1) -> bool {
    let width = image.width as usize;
    let height = image.height as usize;
    (0..width).all(|x| {
        image.pixels[x] == PLT_TRANSPARENT_PIXEL_V1
            && image.pixels[(height - 1) * width + x] == PLT_TRANSPARENT_PIXEL_V1
    }) && (0..height).all(|y| {
        image.pixels[y * width] == PLT_TRANSPARENT_PIXEL_V1
            && image.pixels[y * width + width - 1] == PLT_TRANSPARENT_PIXEL_V1
    })
}

fn build_helmet_226_module_fixture(
    source_module: &[u8],
) -> Result<Vec<HakResourceInputV1>, Box<dyn std::error::Error>> {
    let archive = ErfArchive::parse(source_module)?;
    let mut resources = Vec::new();
    for descriptor in archive.resources() {
        if matches!(
            descriptor.resource_type,
            UTI_RESOURCE_TYPE | ITP_RESOURCE_TYPE
        ) {
            continue;
        }
        if descriptor.resource_type == UTC_RESOURCE_TYPE {
            return Err(
                "palette-only fixture source unexpectedly contains a creature blueprint".into(),
            );
        }
        let mut payload = archive
            .find(&descriptor.resref, descriptor.resource_type)?
            .to_vec();
        match descriptor.resource_type {
            IFO_RESOURCE_TYPE => {
                let mut document = read_expected_gff(&payload, GffFileTypeV1::Ifo)?;
                replace_string(&mut document.root, "Mod_Tag", MODULE_RESREF)?;
                replace_loc_string(&mut document.root, "Mod_Name", MODULE_DISPLAY_NAME)?;
                replace_loc_string(
                    &mut document.root,
                    "Mod_Description",
                    "Palette-only Bronze Mask 226 V1 module: one model-named PLT atlas with Metal 1, Cloth 1/2 and Leather 1/2 regions, corrected native PLT icon and 42466 triangles.",
                )?;
                replace_hak_list(&mut document.root, &[HAK_RESREF, TABLE_HAK_RESREF])?;
                payload = write_gff_v32(&document, &GffWriterOptionsV1::default())?.payload;
            }
            2012 => {
                let mut document = read_expected_gff(&payload, GffFileTypeV1::Are)?;
                replace_loc_string(&mut document.root, "Name", AREA_DISPLAY_NAME)?;
                payload = write_gff_v32(&document, &GffWriterOptionsV1::default())?.payload;
            }
            _ => {}
        }
        resources.push(HakResourceInputV1 {
            resref: descriptor.resref.clone(),
            resource_type: descriptor.resource_type,
            payload,
        });
    }
    Ok(resources)
}

fn validate_package_readback(
    hak_payload: &[u8],
    module_payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let hak = ErfArchive::parse(hak_payload)?;
    let model_plt = read_plt_image_v1(hak.find(MODEL_RESREF, PLT_RESOURCE_TYPE)?)?;
    let icon_plt = read_plt_image_v1(hak.find(ICON_RESREF, ITEM_ICON_PLT_RESOURCE_TYPE)?)?;
    if model_plt.num_layers != 8 || icon_plt.num_layers != 8 {
        return Err("HAK PLT readback lost the exact layer contract".into());
    }
    if hak.find(MODEL_RESREF, 3).is_ok() || hak.find(ICON_RESREF, 3).is_ok() {
        return Err(
            "Bronze Mask 226 HAK contains forbidden fallback TGA for a ModelType-1 helmet".into(),
        );
    }
    let module = ErfArchive::parse(module_payload)?;
    let uti = read_item_uti_v1(
        module.find(UTI_RESREF, UTI_RESOURCE_TYPE)?,
        ItemCompositionProfileV1::ModelType1,
        &Default::default(),
    )?;
    if uti.parts.len() != 1
        || uti.parts[0].variant != VARIANT
        || uti.colors.as_ref().is_none_or(|colors| {
            colors.metal1 != DEFAULT_METAL1_ROW
                || colors.cloth1 != DEFAULT_CLOTH1_ROW
                || colors.cloth2 != DEFAULT_CLOTH2_ROW
                || colors.leather1 != DEFAULT_LEATHER1_ROW
                || colors.leather2 != DEFAULT_LEATHER2_ROW
        })
    {
        return Err(
            "Bronze Mask 226 UTI readback lost variant 226 or the default PLT color rows".into(),
        );
    }
    Ok(())
}

fn read_expected_gff(
    bytes: &[u8],
    expected: GffFileTypeV1,
) -> Result<GffDocumentV1, Box<dyn std::error::Error>> {
    let document = read_gff_v32(bytes, &Default::default())?;
    if document.file_type != expected {
        return Err("module fixture GFF type differs".into());
    }
    Ok(document)
}

fn field_value_mut<'a>(
    structure: &'a mut GffStructV1,
    label: &str,
) -> Result<&'a mut GffValueV1, Box<dyn std::error::Error>> {
    structure
        .fields
        .iter_mut()
        .find(|field| field.label == label)
        .map(|field| &mut field.value)
        .ok_or_else(|| format!("missing required GFF field {label}").into())
}

fn replace_string(
    structure: &mut GffStructV1,
    label: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = field_value_mut(structure, label)?;
    if !matches!(target, GffValueV1::String(_)) {
        return Err(format!("GFF field {label} is not a CExoString").into());
    }
    *target = GffValueV1::String(value.as_bytes().to_vec());
    Ok(())
}

fn replace_loc_string(
    structure: &mut GffStructV1,
    label: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = field_value_mut(structure, label)?;
    if !matches!(target, GffValueV1::LocString(_)) {
        return Err(format!("GFF field {label} is not a CExoLocString").into());
    }
    *target = GffValueV1::LocString(GffLocStringV1 {
        string_ref: u32::MAX,
        substrings: vec![GffLocSubstringV1 {
            string_id: 0,
            bytes: value.as_bytes().to_vec(),
        }],
    });
    Ok(())
}

fn replace_hak_list(
    structure: &mut GffStructV1,
    hak_resrefs: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let target = field_value_mut(structure, "Mod_HakList")?;
    if !matches!(target, GffValueV1::List(_)) {
        return Err("Mod_HakList is not a GFF List".into());
    }
    *target = GffValueV1::List(
        hak_resrefs
            .iter()
            .map(|hak| GffStructV1 {
                struct_id: 8,
                fields: vec![GffFieldV1 {
                    label: "Mod_Hak".to_owned(),
                    value: GffValueV1::String(hak.as_bytes().to_vec()),
                }],
            })
            .collect(),
    );
    Ok(())
}

fn read_exact_hash(path: &Path, expected: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "source hash mismatch for {}: expected {expected}, read {actual}",
            path.display()
        )
        .into());
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
