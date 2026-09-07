use std::{
    collections::BTreeMap,
    env, fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use image::{ImageReader, imageops::FilterType};
use m2a_core::{
    erf::ErfArchive,
    gff::{
        GffFileTypeV1, GffLimitsV1, GffStructV1, GffValueV1, GffWriterOptionsV1, read_gff_v32,
        write_gff_v32,
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
    item_icon::{ItemIconLayerInputV1, write_item_icon_layers_v1},
    item_package::{
        GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE, ItemCompiledPartInputV1, ItemPackageBuildRequestV1,
        MDL_RESOURCE_TYPE, TGA_RESOURCE_TYPE, UTC_RESOURCE_TYPE, UTI_RESOURCE_TYPE,
        write_item_package_v1,
    },
    item_part::{
        MeshyItemPartCompileRequestV1, compile_meshy_static_item_part_v1,
        static_item_profile_a_options_v1,
    },
    item_uti::{ItemUtiBuildRequestV1, ItemUtiPropertiesV1},
    mdl::MdlMaterialTextureBindingV1,
    model_pipeline::resolve_base_color_image_index_v1,
    profile_a::{convert_profile_a_exact_v1, derive_meshy_m0_static_rigid_profile_v1},
    proof_module::{
        BinaryCreatureEquippedFixtureV1, BinaryCreatureHandSlotV1, BinaryCreatureModuleIdentityV1,
        BinaryCreatureOwnedFixtureV1, BinaryCreatureProfiledFixtureV2,
        BinaryCreatureRuntimeProfileV2, BinaryCreatureWeaponItemV1, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_owned_weapon_demo_module_named_v2,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1},
    two_da::TwoDaLimitsV1,
};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};

const VARIANT: u8 = 221;
const MODEL_RESREF: &str = "helm_221";
const ICON_RESREF: &str = "ihelm_221";
const UTI_RESREF: &str = "m2assmask221";
const HAK_RESREF: &str = "m2assmh221";
const MODULE_RESREF: &str = "m2assmmod221";
const AREA_RESREF: &str = "m2assmarea221";
const CREATURE_RESREF: &str = "m2assmcre221";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora Sandstorm Mask Helmet 221";
const AREA_DISPLAY_NAME: &str = "Sandstorm Mask Helmet Test 221";
const HELMET_EQUIPMENT_SLOT: u32 = 1;
const HUMAN_APPEARANCE_ROW: u16 = 6;
const TWO_DA_RESOURCE_TYPE: u16 = 2017;

const DONOR_BOUNDS_MIN: [f32; 3] = [-0.126_248_8, -0.159_788_1, -0.111_795_1];
const DONOR_BOUNDS_MAX: [f32; 3] = [0.126_248_8, 0.166_394_5, 0.224_525_4];
const PROVISIONAL_COWL_MIN_Z: f32 = -0.069_743_5;
const FIT_CLEARANCE_METERS: f32 = 0.005;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NamespaceInventorySourceV1 {
    role: &'static str,
    path: String,
    byte_length: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NamespaceInventoryV1<'a> {
    schema_version: u32,
    policy: &'static str,
    sources: &'a [NamespaceInventorySourceV1],
    resources: &'a [ItemResourceKeyV1],
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        env::var("M2A_REPO_ROOT").unwrap_or_else(|_| r"C:\Projects\meshy2aurora".to_owned()),
    );
    let source_path = env::var("M2A_ITEM_SOURCE_GLB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            root.join("sample-3d/tlc-sandstorm-mask-meshy-0904135727-v1/source-p300k.glb")
        });
    let icon_source_path = env::var("M2A_ITEM_ICON_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            root.join(
                "artifacts/diagnostics/tlc-sandstorm-mask-owner-glb-0904135727-offline-qa/product-p300k-renders/front.png",
            )
        });
    let output_dir = env::var("M2A_ITEM_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("proof-output/tlc-sandstorm-mask-helmet-221-v1-20260904"));
    if output_dir.exists() {
        return Err(format!("immutable output already exists: {}", output_dir.display()).into());
    }

    let source_glb = fs::read(&source_path)?;
    let source_sha256 = sha256(&source_glb);
    let glb_limits = GlbLimits::default();
    let ingest = ingest_glb(&source_glb, &glb_limits)?;
    if ingest.report.statistics.triangle_count != 300_000 {
        return Err(format!(
            "expected exact 300000-triangle selected product source, got {}",
            ingest.report.statistics.triangle_count
        )
        .into());
    }
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest)?;
    let conversion =
        convert_profile_a_exact_v1(&ingest, &rig, &static_item_profile_a_options_v1())?;
    if !conversion.report.conversion_eligible {
        return Err("selected GLB failed exact static Item Profile A admission".into());
    }
    let model = conversion
        .creature
        .as_ref()
        .ok_or("eligible Item conversion has no common model IR")?;
    let source_bounds = model_bounds(model)?;
    let transform = donor_fit_transform(source_bounds)?;
    let fitted_bounds = transformed_bounds(source_bounds, transform);
    if fitted_bounds.0[2] <= PROVISIONAL_COWL_MIN_Z {
        return Err(format!(
            "fitted cowl minimum {} does not clear provisional boundary {}",
            fitted_bounds.0[2], PROVISIONAL_COWL_MIN_Z
        )
        .into());
    }

    let baseitems_path =
        PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak\lc_2da.hak");
    let baseitems_hak_bytes = fs::read(&baseitems_path)?;
    let baseitems_hak = ErfArchive::parse(&baseitems_hak_bytes)?;
    let baseitems_bytes = baseitems_hak.find("baseitems", TWO_DA_RESOURCE_TYPE)?;
    let baseitems_sha256 = sha256(baseitems_bytes);
    let base_item = resolve_item_base_record_v1(
        baseitems_bytes,
        17,
        &baseitems_sha256,
        &TwoDaLimitsV1::default(),
    )?;
    validate_helmet_baseitem(&base_item)?;

    let recipe = ItemAppearanceRecipeV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        base_item,
        identity: ItemIdentityV1 {
            uti_resref: UTI_RESREF.to_owned(),
            tag: "M2A_SSMASK_221".to_owned(),
            display_name: "Brązowa Maska Burzy Piaskowej".to_owned(),
        },
        gender: None,
        parts: vec![ItemPartRecipeV1 {
            slot: ItemPartSlotV1::Model,
            variant: VARIANT,
            source_part_id: "tlc-sandstorm-mask-meshy-0904135727-v1/source-p300k.glb".to_owned(),
            source_sha256: source_sha256.clone(),
            transform,
        }],
        colors: Some(ItemColorRecipeV1 {
            leather1: 0,
            leather2: 0,
            cloth1: 0,
            cloth2: 0,
            metal1: 0,
            metal2: 0,
        }),
    };
    let names = resolve_item_resource_names_v1(&recipe)?;
    if names.len() != 1
        || names[0].model_resref != MODEL_RESREF
        || names[0].icon_resref != ICON_RESREF
    {
        return Err("helmet naming did not resolve to the frozen variant-221 namespace".into());
    }
    let recipe_sha256 = item_recipe_sha256_v1(&recipe)?;

    eprintln!("compiling {MODEL_RESREF} from 300000 source triangles");
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
    if compiled.report.triangle_count != 300_000
        || compiled.inspection.model.classification != 4
        || compiled.report.segmentation.output_segment_count <= 1
    {
        return Err(
            "compiled helmet did not preserve budget geometry, retail class and partitioning"
                .into(),
        );
    }

    let texture_selection = resolve_base_color_image_index_v1(&ingest, model)?;
    let source_material = ingest
        .ir
        .materials
        .iter()
        .find(|material| material.id == texture_selection.source_material_id)
        .ok_or("selected base-color material is absent")?;
    if source_material.base_color_factor != [1.0; 4] || source_material.alpha_mode != "OPAQUE" {
        return Err("selected material requires an unsupported base-color or alpha bake".into());
    }
    let diffuse_image = decode_embedded_image_to_tga_v1(
        &source_glb,
        texture_selection.source_image_index,
        &glb_limits,
        &EmbeddedImageDecodeLimitsV1::default(),
    )?;
    let diffuse = write_tga_v1(&diffuse_image, &TgaWriterOptionsV1::default())?;

    let icon_image = build_icon(&icon_source_path)?;
    let icon_source_sha256 = sha256(&icon_image.pixels);
    let icons = write_item_icon_layers_v1(
        &recipe,
        &[ItemIconLayerInputV1 {
            slot: ItemPartSlotV1::Model,
            resref: ICON_RESREF.to_owned(),
            source_sha256: icon_source_sha256.clone(),
            image: icon_image,
        }],
        &TgaWriterOptionsV1::default(),
    )?;

    eprintln!("scanning retail, 300 local HAKs and override for collisions");
    let (namespace_resources, inventory_sources) = build_conservative_namespace()?;
    let inventory_json = serde_json::to_vec_pretty(&NamespaceInventoryV1 {
        schema_version: 1,
        policy: "OUTPUT_COMPATIBLE_KEYS_FROM_NWN_BASE_PLUS_ALL_LOCAL_HAKS_PLUS_OVERRIDE_SUPERSET",
        sources: &inventory_sources,
        resources: &namespace_resources,
    })?;
    let inventory_sha256 = sha256(&inventory_json);
    let namespace_resource_count = namespace_resources.len();
    let namespace = EffectiveResourceNamespaceV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        complete: true,
        inventories_sha256: vec![inventory_sha256.clone()],
        resources: namespace_resources,
    };

    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: MODULE_RESREF.to_owned(),
        area_resref: AREA_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
    };
    let fixture = BinaryCreatureEquippedFixtureV1 {
        profiled_fixture: BinaryCreatureProfiledFixtureV2 {
            fixture: BinaryCreatureOwnedFixtureV1 {
                id: "m2a_ssmask_fixture_221".to_owned(),
                template_resref: CREATURE_RESREF.to_owned(),
                display_name: "Human male - Sandstorm Mask 221".to_owned(),
                appearance_row: HUMAN_APPEARANCE_ROW,
                position: M0RuntimePositionV1 {
                    x: 10.0,
                    y: 14.5,
                    z: 0.0,
                },
                orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
            },
            runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
        },
        hand: BinaryCreatureHandSlotV1::RightHand,
        equipped_item_resref: UTI_RESREF.to_owned(),
    };
    let placeholder = BinaryCreatureWeaponItemV1 {
        resref: UTI_RESREF.to_owned(),
        display_name: recipe.identity.display_name.clone(),
        base_item: 17,
        model_parts: [VARIANT, 1, 1],
    };
    let fixture_module = build_binary_creature_owned_weapon_demo_module_named_v2(
        &module_identity,
        &[fixture],
        &placeholder,
        MODULE_DISPLAY_NAME,
        AREA_DISPLAY_NAME,
        "One Human male fixture equips the generated ModelType-1 helmet in native head slot 1.",
    )?;
    let fixture_archive = ErfArchive::parse(&fixture_module.payload)?;
    let mut module_fixture_resources = Vec::new();
    for resource in fixture_archive.resources() {
        if resource.resource_type == UTI_RESOURCE_TYPE {
            continue;
        }
        let payload = fixture_archive
            .find(&resource.resref, resource.resource_type)?
            .to_vec();
        let payload = match resource.resource_type {
            GIT_RESOURCE_TYPE => {
                patch_equipment_slot(&payload, GffFileTypeV1::Git, HELMET_EQUIPMENT_SLOT)?
            }
            UTC_RESOURCE_TYPE => {
                patch_equipment_slot(&payload, GffFileTypeV1::Utc, HELMET_EQUIPMENT_SLOT)?
            }
            _ => payload,
        };
        module_fixture_resources.push(HakResourceInputV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            payload,
        });
    }

    let package = write_item_package_v1(
        &ItemPackageBuildRequestV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            generator_identity: "materialize_tlc_sandstorm_mask_helmet_v1".to_owned(),
            recipe: recipe.clone(),
            namespace,
            uti: ItemUtiBuildRequestV1 {
                schema_version: ITEM_SCHEMA_VERSION,
                recipe: recipe.clone(),
                properties: ItemUtiPropertiesV1 {
                    description: "Sztywna brązowa maska z krótkim czarnym kapturem.".to_owned(),
                    identified_description:
                        "Maska burzy piaskowej przygotowana przez Meshy2Aurora.".to_owned(),
                    cost: 1,
                    comment: "TLC Sandstorm Mask owner-proof candidate 221.".to_owned(),
                    ..ItemUtiPropertiesV1::default()
                },
            },
            parts: vec![ItemCompiledPartInputV1 {
                slot: ItemPartSlotV1::Model,
                resref: MODEL_RESREF.to_owned(),
                source_sha256: sha256(&compiled.payload),
                payload: compiled.payload.clone(),
                compile_report: compiled.report.clone(),
            }],
            icons: icons.clone(),
            additional_hak_resources: vec![HakResourceInputV1 {
                resref: MODEL_RESREF.to_owned(),
                resource_type: TGA_RESOURCE_TYPE,
                payload: diffuse.payload.clone(),
            }],
            module_fixture_resources,
        },
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )?;
    if package.manifest.fixture_equipment_slot != Some(HELMET_EQUIPMENT_SLOT)
        || package.manifest.semantic_readback_status != "PASS"
    {
        return Err("package readback did not retain the native helmet slot".into());
    }

    let recipe_json = serde_json::to_vec_pretty(&recipe)?;
    let offline_report = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "source": {
            "path": source_path.to_string_lossy(),
            "sha256": source_sha256,
            "byteLength": source_glb.len(),
            "triangleCount": ingest.report.statistics.triangle_count,
        },
        "baseitems": {
            "container": baseitems_path.to_string_lossy(),
            "resource": "baseitems.2da",
            "sha256": baseitems_sha256,
            "physicalRow": 17,
            "modelType": 1,
            "itemClass": "helm",
            "genderSpecific": false,
        },
        "candidate": {
            "variant": VARIANT,
            "modelResref": MODEL_RESREF,
            "iconResref": ICON_RESREF,
            "utiResref": UTI_RESREF,
            "hakResref": HAK_RESREF,
            "moduleResref": MODULE_RESREF,
            "moduleFilename": format!("{MODULE_RESREF}.mod"),
            "moduleDisplayName": MODULE_DISPLAY_NAME,
            "areaResref": AREA_RESREF,
            "areaDisplayName": AREA_DISPLAY_NAME,
            "fixtureCreatureResref": CREATURE_RESREF,
            "fixtureAppearanceRow": HUMAN_APPEARANCE_ROW,
            "fixtureEquipmentSlot": HELMET_EQUIPMENT_SLOT,
            "fixturePlacement": {"x": 10.0, "y": 14.5, "z": 0.0},
        },
        "fit": {
            "effectiveDonor": "bdhd_items:helm_035",
            "donorBoundsMin": DONOR_BOUNDS_MIN,
            "donorBoundsMax": DONOR_BOUNDS_MAX,
            "sourceConvertedBoundsMin": source_bounds.0,
            "sourceConvertedBoundsMax": source_bounds.1,
            "uniformScale": transform.scale,
            "translation": transform.translation,
            "rotationQuaternion": transform.rotation,
            "fittedBoundsMin": fitted_bounds.0,
            "fittedBoundsMax": fitted_bounds.1,
            "provisionalCowlMinZ": PROVISIONAL_COWL_MIN_Z,
            "cowlBoundaryPass": fitted_bounds.0[2] > PROVISIONAL_COWL_MIN_Z,
            "independentAxisScalingApplied": false,
        },
        "model": {
            "sha256": sha256(&compiled.payload),
            "classification": compiled.inspection.model.classification,
            "triangleCount": compiled.report.triangle_count,
            "inputSegmentCount": compiled.report.segmentation.source_segment_count,
            "outputSegmentCount": compiled.report.segmentation.output_segment_count,
            "partitioned": compiled.report.segmentation.output_segment_count > 1,
            "semanticReadbackStatus": compiled.report.semantic_readback_status,
        },
        "diffuse": {
            "resref": MODEL_RESREF,
            "sourceImageIndex": texture_selection.source_image_index,
            "sourceImageSha256": texture_selection.source_image_sha256,
            "width": diffuse_image.width,
            "height": diffuse_image.height,
            "sha256": diffuse.report.output_sha256,
            "alphaMode": source_material.alpha_mode,
        },
        "icon": {
            "resref": ICON_RESREF,
            "sourcePath": icon_source_path.to_string_lossy(),
            "sourcePixelsSha256": icon_source_sha256,
            "sha256": icons[0].report.output_sha256,
            "width": icons[0].report.width,
            "height": icons[0].report.height,
        },
        "namespace": {
            "inventorySha256": inventory_sha256,
            "conservativeResourceCount": namespace_resource_count,
            "variant221AbsentBeforeBuild": true,
        },
        "package": package.manifest,
        "nativeInstallation": "not_performed_owner_instruction_required",
        "liveToolsetOrNwn": "not_started_human_owned_final_proof",
    }))?;

    fs::create_dir_all(&output_dir)?;
    write_new(
        &output_dir.join(format!("{MODEL_RESREF}.mdl")),
        &compiled.payload,
    )?;
    write_new(
        &output_dir.join(format!("{MODEL_RESREF}.tga")),
        &diffuse.payload,
    )?;
    write_new(
        &output_dir.join(format!("{ICON_RESREF}.tga")),
        &icons[0].payload,
    )?;
    write_new(
        &output_dir.join(format!("{UTI_RESREF}.uti")),
        &package.uti.payload,
    )?;
    write_new(
        &output_dir.join(format!("{HAK_RESREF}.hak")),
        &package.hak_payload,
    )?;
    write_new(
        &output_dir.join(format!("{MODULE_RESREF}.mod")),
        &package.module_payload,
    )?;
    write_new(&output_dir.join("manifest.json"), &package.manifest_json)?;
    write_new(&output_dir.join("recipe.json"), &recipe_json)?;
    write_new(
        &output_dir.join("namespace-inventory.json"),
        &inventory_json,
    )?;
    write_new(&output_dir.join("offline-report.json"), &offline_report)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "output": output_dir,
            "moduleFilename": format!("{MODULE_RESREF}.mod"),
            "moduleDisplayName": MODULE_DISPLAY_NAME,
            "areaDisplayName": AREA_DISPLAY_NAME,
            "hakFilename": format!("{HAK_RESREF}.hak"),
            "modelResref": MODEL_RESREF,
            "utiResref": UTI_RESREF,
            "triangleCount": compiled.report.triangle_count,
            "segmentedMeshCount": compiled.report.segmentation.output_segment_count,
            "classification": compiled.inspection.model.classification,
            "fittedBoundsMin": fitted_bounds.0,
            "fittedBoundsMax": fitted_bounds.1,
            "packageSha256": package.manifest.module_sha256,
            "hakSha256": package.manifest.hak_sha256,
            "status": "ready_for_owner_proof",
        }))?
    );
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

fn donor_fit_transform(
    source: ([f32; 3], [f32; 3]),
) -> Result<ItemPartTransformV1, Box<dyn std::error::Error>> {
    let source_span = std::array::from_fn::<_, 3, _>(|axis| source.1[axis] - source.0[axis]);
    if source_span
        .iter()
        .any(|span| !span.is_finite() || *span <= 0.0)
    {
        return Err("source bounds are degenerate".into());
    }
    let donor_span =
        std::array::from_fn::<_, 3, _>(|axis| DONOR_BOUNDS_MAX[axis] - DONOR_BOUNDS_MIN[axis]);
    // Side/depth clearance is stricter than vertical occupancy.  This keeps
    // the rigid cowl inside the donor envelope without non-uniform scaling.
    let allowed_vertical_span =
        DONOR_BOUNDS_MAX[2] - PROVISIONAL_COWL_MIN_Z - 2.0 * FIT_CLEARANCE_METERS;
    let scale = (donor_span[0] * 0.96 / source_span[0])
        .min(donor_span[1] * 0.96 / source_span[1])
        .min(allowed_vertical_span / source_span[2]);
    let source_center =
        std::array::from_fn::<_, 3, _>(|axis| (source.0[axis] + source.1[axis]) * 0.5);
    let mut donor_center = std::array::from_fn::<_, 3, _>(|axis| {
        (DONOR_BOUNDS_MIN[axis] + DONOR_BOUNDS_MAX[axis]) * 0.5
    });
    donor_center[2] = (PROVISIONAL_COWL_MIN_Z + DONOR_BOUNDS_MAX[2]) * 0.5;
    Ok(ItemPartTransformV1 {
        translation: std::array::from_fn(|axis| donor_center[axis] - source_center[axis] * scale),
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

fn build_icon(path: &Path) -> Result<TgaImageV1, Box<dyn std::error::Error>> {
    let image = ImageReader::open(path)?.decode()?.into_rgba8();
    let resized = image::imageops::resize(&image, 64, 64, FilterType::Lanczos3);
    Ok(TgaImageV1 {
        schema_version: 1,
        width: 64,
        height: 64,
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: resized.into_raw(),
    })
}

fn patch_equipment_slot(
    payload: &[u8],
    expected_type: GffFileTypeV1,
    slot: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut document = read_gff_v32(payload, &GffLimitsV1::default())?;
    if document.file_type != expected_type {
        return Err("fixture GFF type mismatch while patching helmet slot".into());
    }
    match expected_type {
        GffFileTypeV1::Git => {
            let creatures = list_field_mut(&mut document.root, "Creature List")?;
            if creatures.len() != 1 {
                return Err("helmet fixture requires exactly one GIT creature".into());
            }
            patch_creature_slot(&mut creatures[0], slot)?;
        }
        GffFileTypeV1::Utc => patch_creature_slot(&mut document.root, slot)?,
        _ => return Err("only GIT and UTC equipment fixtures can be patched".into()),
    }
    Ok(write_gff_v32(&document, &GffWriterOptionsV1::default())?.payload)
}

fn patch_creature_slot(
    creature: &mut GffStructV1,
    slot: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let equipment = list_field_mut(creature, "Equip_ItemList")?;
    if equipment.len() != 1 {
        return Err("helmet fixture requires exactly one equipped item".into());
    }
    equipment[0].struct_id = slot;
    Ok(())
}

fn list_field_mut<'a>(
    structure: &'a mut GffStructV1,
    label: &str,
) -> Result<&'a mut Vec<GffStructV1>, Box<dyn std::error::Error>> {
    let field = structure
        .fields
        .iter_mut()
        .find(|field| field.label == label)
        .ok_or_else(|| format!("fixture field {label} is missing"))?;
    match &mut field.value {
        GffValueV1::List(values) => Ok(values),
        _ => Err(format!("fixture field {label} is not a List").into()),
    }
}

fn build_conservative_namespace()
-> Result<(Vec<ItemResourceKeyV1>, Vec<NamespaceInventorySourceV1>), Box<dyn std::error::Error>> {
    let key_path = PathBuf::from(
        r"C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\nwn_base.key",
    );
    let hak_dir = PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak");
    let override_dir = PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\override");
    let mut resources = BTreeMap::<(String, u16), ItemResourceScopeV1>::new();
    let mut sources = Vec::new();

    let key_bytes = fs::read(&key_path)?;
    for (resref, resource_type) in read_key_resource_keys(&key_bytes)? {
        if !is_output_compatible_namespace_resref(&resref) {
            continue;
        }
        resources.insert(
            (resref.to_ascii_lowercase(), resource_type),
            ItemResourceScopeV1::Retail,
        );
    }
    sources.push(NamespaceInventorySourceV1 {
        role: "NWN_BASE_KEY",
        path: key_path.to_string_lossy().into_owned(),
        byte_length: key_bytes.len() as u64,
    });

    let mut hak_paths = fs::read_dir(&hak_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|value| value.eq_ignore_ascii_case("hak"))
        })
        .collect::<Vec<_>>();
    hak_paths.sort();
    for path in hak_paths {
        for (resref, resource_type) in read_erf_resource_keys(&path)? {
            if !is_output_compatible_namespace_resref(&resref) {
                continue;
            }
            resources
                .entry((resref.to_ascii_lowercase(), resource_type))
                .or_insert(ItemResourceScopeV1::Hak);
        }
        sources.push(NamespaceInventorySourceV1 {
            role: "LOCAL_HAK",
            path: path.to_string_lossy().into_owned(),
            byte_length: fs::metadata(&path)?.len(),
        });
    }

    if override_dir.is_dir() {
        let mut override_paths = fs::read_dir(&override_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        override_paths.sort();
        for path in override_paths {
            if let (Some(stem), Some(resource_type)) = (
                path.file_stem().and_then(|value| value.to_str()),
                resource_type_for_extension(&path),
            ) {
                if !is_output_compatible_namespace_resref(stem) {
                    continue;
                }
                resources.insert(
                    (stem.to_ascii_lowercase(), resource_type),
                    ItemResourceScopeV1::Retail,
                );
            }
        }
        sources.push(NamespaceInventorySourceV1 {
            role: "USER_OVERRIDE_DIRECTORY",
            path: override_dir.to_string_lossy().into_owned(),
            byte_length: 0,
        });
    }

    Ok((
        resources
            .into_iter()
            .map(|((resref, resource_type), scope)| ItemResourceKeyV1 {
                resref,
                resource_type,
                scope,
            })
            .collect(),
        sources,
    ))
}

fn is_output_compatible_namespace_resref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn read_key_resource_keys(bytes: &[u8]) -> Result<Vec<(String, u16)>, Box<dyn std::error::Error>> {
    if bytes.len() < 64 || &bytes[0..4] != b"KEY " || &bytes[4..8] != b"V1  " {
        return Err("expected NWN KEY V1 resource index".into());
    }
    let count = read_u32(bytes, 12)? as usize;
    let offset = read_u32(bytes, 20)? as usize;
    let length = count.checked_mul(22).ok_or("KEY table length overflow")?;
    let table = bytes
        .get(offset..offset.checked_add(length).ok_or("KEY table end overflow")?)
        .ok_or("KEY resource table is out of bounds")?;
    let mut output = Vec::with_capacity(count);
    for entry in table.chunks_exact(22) {
        output.push((
            parse_resref(&entry[..16])?,
            u16::from_le_bytes([entry[16], entry[17]]),
        ));
    }
    Ok(output)
}

fn read_erf_resource_keys(path: &Path) -> Result<Vec<(String, u16)>, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header)?;
    if &header[0..4] != b"HAK " || &header[4..8] != b"V1.0" {
        return Err(format!("expected HAK V1.0: {}", path.display()).into());
    }
    let count = u32::from_le_bytes(header[16..20].try_into()?) as usize;
    let offset = u32::from_le_bytes(header[24..28].try_into()?) as u64;
    let length = count
        .checked_mul(24)
        .ok_or("HAK key table length overflow")?;
    let mut table = vec![0u8; length];
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(&mut table)?;
    let mut output = Vec::with_capacity(count);
    for entry in table.chunks_exact(24) {
        output.push((
            parse_resref(&entry[..16])?,
            u16::from_le_bytes([entry[20], entry[21]]),
        ));
    }
    Ok(output)
}

fn parse_resref(bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let value = std::str::from_utf8(&bytes[..end])?;
    if value.is_empty() {
        return Err("resource inventory contains an empty ResRef".into());
    }
    Ok(value.to_owned())
}

fn resource_type_for_extension(path: &Path) -> Option<u16> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "tga" => Some(TGA_RESOURCE_TYPE),
        "mdl" => Some(MDL_RESOURCE_TYPE),
        "uti" => Some(UTI_RESOURCE_TYPE),
        "utc" => Some(UTC_RESOURCE_TYPE),
        "git" => Some(GIT_RESOURCE_TYPE),
        "gic" => Some(GIC_RESOURCE_TYPE),
        "2da" => Some(TWO_DA_RESOURCE_TYPE),
        _ => None,
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("u32 read is out of bounds")?
            .try_into()?,
    ))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        return Err(format!("refusing to overwrite immutable output {}", path.display()).into());
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
