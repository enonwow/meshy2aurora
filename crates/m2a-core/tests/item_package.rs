use m2a_core::erf::{ErfArchive, ErfFileType};
use m2a_core::gff::{GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32};
use m2a_core::hak::{HakResourceInputV1, HakWriterOptionsV1};
use m2a_core::item::{
    EffectiveResourceNamespaceV1, ITEM_SCHEMA_VERSION, ItemAppearanceRecipeV1, ItemBaseRecordV1,
    ItemColorRecipeV1, ItemCompositionProfileV1, ItemIdentityV1, ItemPartRecipeV1, ItemPartSlotV1,
    ItemPartTransformV1, ItemResourceScopeV1, required_item_part_slots_v1,
    resolve_item_resource_names_v1,
};
use m2a_core::item_icon::{
    ITEM_ICON_PLT_RESOURCE_TYPE, ItemIconLayerInputV1, ItemPltIconLayerInputV1,
    write_item_icon_layers_v1, write_item_plt_icon_layers_v1,
};
use m2a_core::item_package::{
    GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE, ITEM_PACKAGE_SEMANTIC_DIFF, ITEM_PACKAGE_SOURCE_STALE,
    ITP_RESOURCE_TYPE, ItemCompiledPartInputV1, ItemPackageBuildRequestV1, MDL_RESOURCE_TYPE,
    PLT_RESOURCE_TYPE, UTC_RESOURCE_TYPE, UTI_RESOURCE_TYPE, write_item_package_v1,
};
use m2a_core::item_part::{ItemPartCompileRequestV1, compile_item_part_v1};
use m2a_core::item_uti::{ItemUtiBuildRequestV1, ItemUtiPropertiesV1};
use m2a_core::mdl::MdlMaterialTextureBindingV1;
use m2a_core::model_ir::{
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1,
};
use m2a_core::plt::{
    PltImageV1, PltPixelV1, PltWriterOptionsV1, plt_image_pixels_sha256_v1, write_plt_v1,
};
use m2a_core::proof_module::{
    BinaryCreatureEquippedFixtureV1, BinaryCreatureHandSlotV1, BinaryCreatureModuleIdentityV1,
    BinaryCreatureOwnedFixtureV1, BinaryCreatureProfiledFixtureV2, BinaryCreatureRuntimeProfileV2,
    BinaryCreatureWeaponItemV1, M0RuntimeDirectionV1, M0RuntimePositionV1,
    build_binary_creature_owned_weapon_demo_module_named_v2,
};
use m2a_core::tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, write_tga_v1};
use sha2::{Digest, Sha256};

const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn recipe() -> ItemAppearanceRecipeV1 {
    let profile = ItemCompositionProfileV1::ModelType2;
    ItemAppearanceRecipeV1 {
        schema_version: 1,
        base_item: ItemBaseRecordV1 {
            schema_version: 1,
            source_sha256: HASH.to_owned(),
            physical_row_index: 1,
            printed_row_label: 1,
            profile,
            model_type: 2,
            item_class: "WSwLs".to_owned(),
            gender_specific: false,
            inv_slot_width: 1,
            inv_slot_height: 4,
            equipable_slots: "0x1C030".to_owned(),
            default_model: Some("it_bag".to_owned()),
            default_icon: Some("iwswls".to_owned()),
        },
        identity: ItemIdentityV1 {
            uti_resref: "m2a_item_001".to_owned(),
            tag: "M2A_ITEM_001".to_owned(),
            display_name: "Synthetic package item".to_owned(),
        },
        gender: None,
        parts: required_item_part_slots_v1(profile)
            .iter()
            .enumerate()
            .map(|(index, slot)| ItemPartRecipeV1 {
                slot: *slot,
                variant: u8::try_from(index + 1).unwrap(),
                source_part_id: format!("part-{index}"),
                source_sha256: HASH.to_owned(),
                transform: ItemPartTransformV1::default(),
            })
            .collect(),
        colors: None,
    }
}

fn model() -> AuroraModelIrV1 {
    let identity = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    AuroraModelIrV1 {
        schema_version: 1,
        profile_id: "item-part-test".to_owned(),
        source_sha256: HASH.to_owned(),
        basis_status: "TEST".to_owned(),
        engine_facing_proof: "OPEN".to_owned(),
        uv_runtime_proof: "OPEN".to_owned(),
        nodes: vec![AuroraModelNodeV1 {
            id: 1,
            name: "source_root".to_owned(),
            parent_id: None,
            bind_local_matrix: identity,
        }],
        material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
            slot: 0,
            source_material_id: None,
            source_material_name: None,
        }],
        segments: vec![AuroraModelSegmentV1 {
            segment_id: 1,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Rigid,
            parent_node_id: 1,
            cast_shadow: true,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: None,
            uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            face_surface_ids: vec![],
            weights: vec![],
        }],
    }
}

fn request() -> ItemPackageBuildRequestV1 {
    let recipe = recipe();
    let recipe_sha256 = m2a_core::item::item_recipe_sha256_v1(&recipe).unwrap();
    let names = resolve_item_resource_names_v1(&recipe).unwrap();
    let parts = names
        .iter()
        .map(|name| {
            let recipe_part = recipe
                .parts
                .iter()
                .find(|part| part.slot == name.slot)
                .unwrap();
            let compiled = compile_item_part_v1(&ItemPartCompileRequestV1 {
                schema_version: 1,
                recipe_sha256: recipe_sha256.clone(),
                part: recipe_part.clone(),
                model_resref: name.model_resref.clone(),
                model: model(),
                material_textures: vec![MdlMaterialTextureBindingV1 {
                    material_slot: 0,
                    resref: "m2a_tex".to_owned(),
                }],
            })
            .unwrap();
            ItemCompiledPartInputV1 {
                slot: name.slot,
                resref: name.model_resref.clone(),
                source_sha256: sha256(&compiled.payload),
                payload: compiled.payload,
                compile_report: compiled.report,
            }
        })
        .collect();
    let icon_inputs = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let image = TgaImageV1 {
                schema_version: 1,
                width: 32,
                height: 128,
                pixel_format: TgaPixelFormatV1::Rgba8,
                pixels: vec![u8::try_from(index + 1).unwrap(); 32 * 128 * 4],
            };
            let source_sha256 = write_tga_v1(&image, &TgaWriterOptionsV1::default())
                .unwrap()
                .report
                .input_sha256;
            ItemIconLayerInputV1 {
                slot: name.slot,
                resref: name.icon_resref.clone(),
                source_sha256,
                image,
            }
        })
        .collect::<Vec<_>>();
    let icons =
        write_item_icon_layers_v1(&recipe, &icon_inputs, &TgaWriterOptionsV1::default()).unwrap();
    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: "m2aitemmod1".to_owned(),
        area_resref: "m2aitemarea1".to_owned(),
        hak_resref: "m2aitemhak1".to_owned(),
    };
    let weapon = BinaryCreatureWeaponItemV1 {
        resref: recipe.identity.uti_resref.clone(),
        display_name: recipe.identity.display_name.clone(),
        base_item: recipe.base_item.physical_row_index as i32,
        model_parts: [1, 2, 3],
    };
    let fixture = BinaryCreatureEquippedFixtureV1 {
        profiled_fixture: BinaryCreatureProfiledFixtureV2 {
            fixture: BinaryCreatureOwnedFixtureV1 {
                id: "m2a_item_fixture".to_owned(),
                template_resref: "m2aitemcre1".to_owned(),
                display_name: "Item fixture".to_owned(),
                appearance_row: 102,
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
        equipped_item_resref: recipe.identity.uti_resref.clone(),
    };
    let fixture_module = build_binary_creature_owned_weapon_demo_module_named_v2(
        &module_identity,
        &[fixture],
        &weapon,
        "Meshy2Aurora Item package test",
        "Meshy2Aurora Item fixture area",
        "Generated Item fixture.",
    )
    .unwrap();
    let fixture_archive = ErfArchive::parse(&fixture_module.payload).unwrap();
    let module_fixture_resources = fixture_archive
        .resources()
        .iter()
        .filter(|resource| resource.resource_type != UTI_RESOURCE_TYPE)
        .map(|resource| HakResourceInputV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            payload: fixture_archive
                .find(&resource.resref, resource.resource_type)
                .unwrap()
                .to_vec(),
        })
        .collect();
    ItemPackageBuildRequestV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        generator_identity: "m2a-core-item-package-test-v1".to_owned(),
        namespace: EffectiveResourceNamespaceV1 {
            schema_version: 1,
            complete: true,
            inventories_sha256: vec![HASH.to_owned()],
            resources: vec![],
        },
        uti: ItemUtiBuildRequestV1 {
            schema_version: 1,
            recipe: recipe.clone(),
            properties: ItemUtiPropertiesV1::default(),
        },
        recipe,
        parts,
        icons,
        additional_hak_resources: vec![],
        module_fixture_resources,
    }
}

fn without_creatures(mut input: ItemPackageBuildRequestV1) -> ItemPackageBuildRequestV1 {
    input.uti.properties.palette_id = 9;
    input
        .module_fixture_resources
        .retain(|resource| resource.resource_type != UTC_RESOURCE_TYPE);
    for resource in &mut input.module_fixture_resources {
        if !matches!(
            resource.resource_type,
            GIT_RESOURCE_TYPE | GIC_RESOURCE_TYPE
        ) {
            continue;
        }
        let mut document = read_gff_v32(&resource.payload, &Default::default()).unwrap();
        let creatures = document
            .root
            .fields
            .iter_mut()
            .find(|field| field.label == "Creature List")
            .expect("fixture creature list");
        creatures.value = GffValueV1::List(vec![]);
        resource.payload = write_gff_v32(&document, &GffWriterOptionsV1::default())
            .unwrap()
            .payload;
    }
    input
}

fn helmet_request() -> ItemPackageBuildRequestV1 {
    let mut input = without_creatures(request());
    let profile = ItemCompositionProfileV1::ModelType1;
    let recipe = ItemAppearanceRecipeV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        base_item: ItemBaseRecordV1 {
            schema_version: ITEM_SCHEMA_VERSION,
            source_sha256: HASH.to_owned(),
            physical_row_index: 17,
            printed_row_label: 17,
            profile,
            model_type: 1,
            item_class: "helm".to_owned(),
            gender_specific: false,
            inv_slot_width: 2,
            inv_slot_height: 2,
            equipable_slots: "0x1".to_owned(),
            default_model: None,
            default_icon: Some("ihelm".to_owned()),
        },
        identity: ItemIdentityV1 {
            uti_resref: "m2a_helm_221".to_owned(),
            tag: "M2A_HELM_221".to_owned(),
            display_name: "PLT helmet".to_owned(),
        },
        gender: None,
        parts: vec![ItemPartRecipeV1 {
            slot: ItemPartSlotV1::Model,
            variant: 221,
            source_part_id: "helmet".to_owned(),
            source_sha256: HASH.to_owned(),
            transform: ItemPartTransformV1::default(),
        }],
        colors: Some(ItemColorRecipeV1 {
            leather1: 0,
            leather2: 0,
            cloth1: 23,
            cloth2: 0,
            metal1: 23,
            metal2: 0,
        }),
    };
    let recipe_sha256 = m2a_core::item::item_recipe_sha256_v1(&recipe).unwrap();
    let name = resolve_item_resource_names_v1(&recipe).unwrap().remove(0);
    let compiled = compile_item_part_v1(&ItemPartCompileRequestV1 {
        schema_version: 1,
        recipe_sha256,
        part: recipe.parts[0].clone(),
        model_resref: name.model_resref.clone(),
        model: model(),
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: name.model_resref.clone(),
        }],
    })
    .unwrap();
    let plt = PltImageV1 {
        schema_version: 1,
        width: 64,
        height: 64,
        num_layers: 5,
        pixels: vec![
            PltPixelV1 {
                color_index: 80,
                layer_index: 2,
            };
            64 * 64
        ],
    };
    let model_plt = write_plt_v1(&plt, &PltWriterOptionsV1::default())
        .unwrap()
        .payload;
    let model_resref = name.model_resref.clone();
    let icons = write_item_plt_icon_layers_v1(
        &recipe,
        &[ItemPltIconLayerInputV1 {
            slot: name.slot,
            resref: name.icon_resref,
            source_sha256: plt_image_pixels_sha256_v1(&plt),
            image: plt,
        }],
        &PltWriterOptionsV1::default(),
    )
    .unwrap();
    input.recipe = recipe.clone();
    input.uti.recipe = recipe;
    input.parts = vec![ItemCompiledPartInputV1 {
        slot: name.slot,
        resref: name.model_resref,
        source_sha256: sha256(&compiled.payload),
        payload: compiled.payload,
        compile_report: compiled.report,
    }];
    input.icons = icons;
    input.additional_hak_resources = vec![HakResourceInputV1 {
        resref: model_resref,
        resource_type: PLT_RESOURCE_TYPE,
        payload: model_plt,
    }];
    input
}

#[test]
fn item_package_is_deterministic_and_reads_every_resource_back_exactly() {
    let input = request();
    let first = write_item_package_v1(
        &input,
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    let second = write_item_package_v1(
        &input,
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.manifest.semantic_readback_status, "PASS");
    assert_eq!(first.manifest.module_resref, "m2aitemmod1");
    assert_eq!(first.manifest.area_resref, "m2aitemarea1");
    assert_eq!(first.manifest.ordered_hak_resrefs, ["m2aitemhak1"]);
    assert_eq!(
        first.manifest.fixture_creature_resref.as_deref(),
        Some("m2aitemcre1")
    );
    assert_eq!(first.manifest.fixture_equipment_slot, Some(16));
    assert_eq!(first.manifest.assembly.triangle_count, 3);
    assert_eq!(
        ErfArchive::parse(&first.hak_payload).unwrap().file_type(),
        ErfFileType::Hak
    );
    let module = ErfArchive::parse(&first.module_payload).unwrap();
    assert_eq!(module.file_type(), ErfFileType::Module);
    assert_eq!(
        module
            .find(&input.recipe.identity.uti_resref, UTI_RESOURCE_TYPE)
            .unwrap(),
        first.uti.payload
    );
    let hak = ErfArchive::parse(&first.hak_payload).unwrap();
    for name in resolve_item_resource_names_v1(&input.recipe).unwrap() {
        assert!(hak.find(&name.model_resref, MDL_RESOURCE_TYPE).is_ok());
    }
}

#[test]
fn palette_only_package_keeps_the_uti_and_rejects_all_creatures() {
    let input = without_creatures(request());
    let artifact = write_item_package_v1(
        &input,
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )
    .unwrap();

    assert_eq!(artifact.manifest.fixture_creature_resref, None);
    assert_eq!(artifact.manifest.fixture_equipment_slot, None);
    let module = ErfArchive::parse(&artifact.module_payload).unwrap();
    assert_eq!(module.resources().len(), 7);
    assert!(
        module
            .resources()
            .iter()
            .all(|resource| resource.resource_type != UTC_RESOURCE_TYPE)
    );
    assert_eq!(
        module
            .find(&input.recipe.identity.uti_resref, UTI_RESOURCE_TYPE)
            .unwrap(),
        artifact.uti.payload
    );
    assert!(module.find("itempalcus", ITP_RESOURCE_TYPE).is_ok());
    for resource_type in [GIT_RESOURCE_TYPE, GIC_RESOURCE_TYPE] {
        let resource = module
            .resources()
            .iter()
            .find(|resource| resource.resource_type == resource_type)
            .unwrap();
        let document = read_gff_v32(
            module
                .find(&resource.resref, resource.resource_type)
                .unwrap(),
            &Default::default(),
        )
        .unwrap();
        assert!(matches!(
            document
                .root
                .fields
                .iter()
                .find(|field| field.label == "Creature List")
                .map(|field| &field.value),
            Some(GffValueV1::List(creatures)) if creatures.is_empty()
        ));
    }
}

#[test]
fn model_type_1_package_uses_resource_type_6_for_the_inventory_icon() {
    let input = helmet_request();
    let artifact = write_item_package_v1(
        &input,
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )
    .unwrap();
    let hak = ErfArchive::parse(&artifact.hak_payload).unwrap();
    assert!(hak.find("ihelm_221", ITEM_ICON_PLT_RESOURCE_TYPE).is_ok());
    assert!(hak.find("ihelm_221", 3).is_err());
}

#[test]
fn model_type_1_package_rejects_non_model_named_mesh_texture() {
    let mut input = helmet_request();
    let recipe_sha256 = m2a_core::item::item_recipe_sha256_v1(&input.recipe).unwrap();
    let name = resolve_item_resource_names_v1(&input.recipe)
        .unwrap()
        .remove(0);
    let compiled = compile_item_part_v1(&ItemPartCompileRequestV1 {
        schema_version: ITEM_SCHEMA_VERSION,
        recipe_sha256,
        part: input.recipe.parts[0].clone(),
        model_resref: name.model_resref.clone(),
        model: model(),
        material_textures: vec![MdlMaterialTextureBindingV1 {
            material_slot: 0,
            resref: "h224cloth".to_owned(),
        }],
    })
    .unwrap();
    input.parts = vec![ItemCompiledPartInputV1 {
        slot: name.slot,
        resref: name.model_resref,
        source_sha256: sha256(&compiled.payload),
        payload: compiled.payload,
        compile_report: compiled.report,
    }];

    let error = write_item_package_v1(
        &input,
        &GffWriterOptionsV1::default(),
        &HakWriterOptionsV1::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ITEM_PACKAGE_SEMANTIC_DIFF);
    assert_eq!(error.path, "request.parts[0].payload.mesh.texture0");
}

#[test]
fn item_package_rejects_a_stale_compiled_part_before_archiving() {
    let mut input = request();
    input.parts[0].source_sha256 = HASH.to_owned();
    assert_eq!(
        write_item_package_v1(
            &input,
            &GffWriterOptionsV1::default(),
            &HakWriterOptionsV1::default(),
        )
        .unwrap_err()
        .code,
        ITEM_PACKAGE_SOURCE_STALE
    );
}

#[test]
fn item_package_rejects_a_compile_report_from_another_recipe() {
    let mut input = request();
    input.parts[0].compile_report.recipe_sha256 = HASH.to_owned();
    assert_eq!(
        write_item_package_v1(
            &input,
            &GffWriterOptionsV1::default(),
            &HakWriterOptionsV1::default(),
        )
        .unwrap_err()
        .code,
        ITEM_PACKAGE_SEMANTIC_DIFF
    );
}

#[test]
fn package_namespace_remains_require_absent() {
    let mut input = request();
    input
        .namespace
        .resources
        .push(m2a_core::item::ItemResourceKeyV1 {
            resref: input.parts[0].resref.to_ascii_uppercase(),
            resource_type: MDL_RESOURCE_TYPE,
            scope: ItemResourceScopeV1::Retail,
        });
    assert!(
        write_item_package_v1(
            &input,
            &GffWriterOptionsV1::default(),
            &HakWriterOptionsV1::default(),
        )
        .is_err()
    );
}

#[test]
fn package_rejects_fixture_not_bound_to_the_exact_generated_uti() {
    let mut input = request();
    input.recipe.identity.uti_resref = "m2a_item_002".to_owned();
    input.uti.recipe = input.recipe.clone();
    assert_eq!(
        write_item_package_v1(
            &input,
            &GffWriterOptionsV1::default(),
            &HakWriterOptionsV1::default(),
        )
        .unwrap_err()
        .code,
        ITEM_PACKAGE_SEMANTIC_DIFF
    );
}
