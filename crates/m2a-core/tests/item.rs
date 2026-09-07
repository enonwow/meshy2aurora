use m2a_core::item::{
    EffectiveResourceNamespaceV1, ITEM_BASEITEMS_INVALID, ITEM_BASEITEMS_STALE,
    ITEM_MODEL_VARIANT_OUT_OF_RANGE, ITEM_NAMESPACE_INCOMPLETE, ITEM_NAMING_UNPROVEN,
    ITEM_RESOURCE_COLLISION, ITEM_TRIANGLE_BUDGET_EXCEEDED, ItemAppearanceRecipeV1,
    ItemAssemblyReportV1, ItemBaseRecordV1, ItemColorRecipeV1, ItemCompositionProfileV1,
    ItemGenderV1, ItemIdentityV1, ItemPartGeometryV1, ItemPartRecipeV1, ItemPartSlotV1,
    ItemPartTransformV1, ItemResourceKeyV1, ItemResourceScopeV1, canonical_item_recipe_json_v1,
    item_recipe_sha256_v1, preflight_item_namespace_v1, required_item_part_slots_v1,
    resolve_item_base_record_v1, resolve_item_resource_names_v1, validate_item_assembly_v1,
    validate_item_model_variants_in_baseitems_v1, validate_item_recipe_v1,
};
use m2a_core::two_da::{TwoDaLimitsV1, inspect_two_da_v2};

const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn baseitems_fixture() -> Vec<u8> {
    b"2DA V2.0\n\nItemClass ModelType GenderSpecific InvSlotWidth InvSlotHeight EquipableSlots DefaultModel DefaultIcon\n0 it_gem 0 0 1 1 0x00000 it_bag iit_gem\n1 helm 1 0 2 2 0x00001 it_bag ihelm\n2 WSwLs 2 0 1 4 0x1C030 it_bag iwswls\n3 AArCl 3 1 2 3 0x00002 gifp iit_chest\n"
        .to_vec()
}

fn resolve(row: u32) -> ItemBaseRecordV1 {
    let bytes = baseitems_fixture();
    let hash = inspect_two_da_v2(&bytes, &TwoDaLimitsV1::default())
        .unwrap()
        .source_sha256;
    resolve_item_base_record_v1(&bytes, row, &hash, &TwoDaLimitsV1::default()).unwrap()
}

fn recipe(profile: ItemCompositionProfileV1) -> ItemAppearanceRecipeV1 {
    let row = u32::from(profile.model_type());
    let mut base_item = resolve(row);
    base_item.gender_specific = profile == ItemCompositionProfileV1::ModelType3;
    let parts = required_item_part_slots_v1(profile)
        .iter()
        .enumerate()
        .map(|(index, slot)| ItemPartRecipeV1 {
            slot: *slot,
            variant: u8::try_from(index + 1).unwrap(),
            source_part_id: format!("part-{index}"),
            source_sha256: HASH.to_owned(),
            transform: ItemPartTransformV1::default(),
        })
        .collect();
    let colors = matches!(
        profile,
        ItemCompositionProfileV1::ModelType1 | ItemCompositionProfileV1::ModelType3
    )
    .then_some(ItemColorRecipeV1 {
        leather1: 1,
        leather2: 2,
        cloth1: 3,
        cloth2: 4,
        metal1: 5,
        metal2: 6,
    });
    ItemAppearanceRecipeV1 {
        schema_version: 1,
        base_item,
        identity: ItemIdentityV1 {
            uti_resref: "m2a_item_001".to_owned(),
            tag: "M2A_ITEM_001".to_owned(),
            display_name: "Meshy2Aurora item".to_owned(),
        },
        gender: (profile == ItemCompositionProfileV1::ModelType3).then_some(ItemGenderV1::Female),
        parts,
        colors,
    }
}

#[test]
fn baseitems_resolver_binds_all_four_profiles_to_the_exact_source() {
    for model_type in 0..=3 {
        let record = resolve(model_type);
        assert_eq!(record.model_type, u8::try_from(model_type).unwrap());
        assert_eq!(record.profile.model_type(), record.model_type);
        assert_eq!(record.physical_row_index, model_type);
        assert_eq!(record.printed_row_label, model_type);
        assert_eq!(record.source_sha256.len(), 64);
    }
    let armor = resolve(3);
    assert!(armor.gender_specific);
    assert_eq!(armor.default_model.as_deref(), Some("gifp"));
    assert_eq!(armor.default_icon.as_deref(), Some("iit_chest"));
}

#[test]
fn baseitems_resolver_rejects_stale_hash_missing_columns_nulls_and_unknown_profile() {
    let bytes = baseitems_fixture();
    assert_eq!(
        resolve_item_base_record_v1(&bytes, 0, HASH, &TwoDaLimitsV1::default())
            .unwrap_err()
            .code,
        ITEM_BASEITEMS_STALE
    );

    let missing = b"2DA V2.0\n\nItemClass ModelType GenderSpecific InvSlotWidth InvSlotHeight EquipableSlots DefaultModel\n0 it_gem 0 0 1 1 0x00000 it_bag\n";
    let missing_hash = inspect_two_da_v2(missing, &TwoDaLimitsV1::default())
        .unwrap()
        .source_sha256;
    assert_eq!(
        resolve_item_base_record_v1(missing, 0, &missing_hash, &TwoDaLimitsV1::default())
            .unwrap_err()
            .code,
        ITEM_BASEITEMS_INVALID
    );

    let invalid = b"2DA V2.0\n\nItemClass ModelType GenderSpecific InvSlotWidth InvSlotHeight EquipableSlots DefaultModel DefaultIcon\n0 it_gem 4 0 1 1 0x00000 **** ****\n";
    let invalid_hash = inspect_two_da_v2(invalid, &TwoDaLimitsV1::default())
        .unwrap()
        .source_sha256;
    assert!(
        resolve_item_base_record_v1(invalid, 0, &invalid_hash, &TwoDaLimitsV1::default()).is_err()
    );
}

#[test]
fn model_variants_must_fit_the_exact_effective_baseitems_range() {
    let bytes = b"2DA V2.0\n\nItemClass ModelType GenderSpecific InvSlotWidth InvSlotHeight EquipableSlots DefaultModel DefaultIcon MinRange MaxRange\n0 helm 1 0 2 2 0x00001 it_bag ihelm 0 50\n";
    let hash = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .unwrap()
        .source_sha256;
    let base_item =
        resolve_item_base_record_v1(bytes, 0, &hash, &TwoDaLimitsV1::default()).unwrap();
    let mut helmet = ItemAppearanceRecipeV1 {
        schema_version: 1,
        base_item,
        identity: ItemIdentityV1 {
            uti_resref: "m2a_helmet_221".to_owned(),
            tag: "M2A_HELMET_221".to_owned(),
            display_name: "Helmet 221".to_owned(),
        },
        gender: None,
        parts: vec![ItemPartRecipeV1 {
            slot: ItemPartSlotV1::Model,
            variant: 50,
            source_part_id: "helmet.glb".to_owned(),
            source_sha256: HASH.to_owned(),
            transform: ItemPartTransformV1::default(),
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

    let range =
        validate_item_model_variants_in_baseitems_v1(bytes, &helmet, &TwoDaLimitsV1::default())
            .unwrap();
    assert_eq!(range.min_range, 0);
    assert_eq!(range.max_range, 50);

    helmet.parts[0].variant = 221;
    let error =
        validate_item_model_variants_in_baseitems_v1(bytes, &helmet, &TwoDaLimitsV1::default())
            .unwrap_err();
    assert_eq!(error.code, ITEM_MODEL_VARIANT_OUT_OF_RANGE);
    assert_eq!(error.path, "recipe.parts[0].variant");

    let expanded = b"2DA V2.0\n\nItemClass ModelType GenderSpecific InvSlotWidth InvSlotHeight EquipableSlots DefaultModel DefaultIcon MinRange MaxRange\n0 helm 1 0 2 2 0x00001 it_bag ihelm 0 255\n";
    let expanded_hash = inspect_two_da_v2(expanded, &TwoDaLimitsV1::default())
        .unwrap()
        .source_sha256;
    helmet.base_item =
        resolve_item_base_record_v1(expanded, 0, &expanded_hash, &TwoDaLimitsV1::default())
            .unwrap();
    assert_eq!(
        validate_item_model_variants_in_baseitems_v1(expanded, &helmet, &TwoDaLimitsV1::default(),)
            .unwrap()
            .max_range,
        255
    );
}

#[test]
fn profiles_require_exact_slot_and_color_sets() {
    assert_eq!(
        required_item_part_slots_v1(ItemCompositionProfileV1::ModelType0).len(),
        1
    );
    assert_eq!(
        required_item_part_slots_v1(ItemCompositionProfileV1::ModelType1).len(),
        1
    );
    assert_eq!(
        required_item_part_slots_v1(ItemCompositionProfileV1::ModelType2).len(),
        3
    );
    assert_eq!(
        required_item_part_slots_v1(ItemCompositionProfileV1::ModelType3).len(),
        19
    );

    for profile in [
        ItemCompositionProfileV1::ModelType0,
        ItemCompositionProfileV1::ModelType1,
        ItemCompositionProfileV1::ModelType2,
        ItemCompositionProfileV1::ModelType3,
    ] {
        validate_item_recipe_v1(&recipe(profile)).unwrap();
    }

    let mut missing = recipe(ItemCompositionProfileV1::ModelType2);
    missing.parts.pop();
    assert!(validate_item_recipe_v1(&missing).is_err());

    let mut wrong_colors = recipe(ItemCompositionProfileV1::ModelType0);
    wrong_colors.colors = Some(ItemColorRecipeV1 {
        leather1: 0,
        leather2: 0,
        cloth1: 0,
        cloth2: 0,
        metal1: 0,
        metal2: 0,
    });
    assert!(validate_item_recipe_v1(&wrong_colors).is_err());
}

#[test]
fn recipe_json_and_hash_are_deterministic_and_identity_sensitive() {
    let first = recipe(ItemCompositionProfileV1::ModelType2);
    let second = first.clone();
    assert_eq!(
        canonical_item_recipe_json_v1(&first).unwrap(),
        canonical_item_recipe_json_v1(&second).unwrap()
    );
    assert_eq!(
        item_recipe_sha256_v1(&first).unwrap(),
        item_recipe_sha256_v1(&second).unwrap()
    );
    let mut changed = second;
    changed.parts[0].variant += 1;
    assert_ne!(
        item_recipe_sha256_v1(&first).unwrap(),
        item_recipe_sha256_v1(&changed).unwrap()
    );
}

#[test]
fn model_type_2_names_match_the_proven_aurora_namespace() {
    let mut input = recipe(ItemCompositionProfileV1::ModelType2);
    input.parts[0].variant = 23;
    input.parts[1].variant = 63;
    input.parts[2].variant = 23;
    let names = resolve_item_resource_names_v1(&input).unwrap();
    assert_eq!(names[0].model_resref, "wswls_b_023");
    assert_eq!(names[1].model_resref, "wswls_m_063");
    assert_eq!(names[2].model_resref, "wswls_t_023");
    assert_eq!(names[0].icon_resref, "iwswls_b_023");
    assert_eq!(names[1].icon_resref, "iwswls_m_063");
    assert_eq!(names[2].icon_resref, "iwswls_t_023");
}

#[test]
fn single_part_gender_naming_is_explicit_and_armor_naming_fails_closed() {
    let mut helmet = recipe(ItemCompositionProfileV1::ModelType1);
    helmet.base_item.gender_specific = true;
    helmet.gender = Some(ItemGenderV1::Male);
    helmet.parts[0].variant = 7;
    let names = resolve_item_resource_names_v1(&helmet).unwrap();
    assert_eq!(names[0].model_resref, "helm_m_007");
    assert_eq!(names[0].icon_resref, "ihelm_m_007");

    assert_eq!(
        resolve_item_resource_names_v1(&recipe(ItemCompositionProfileV1::ModelType3))
            .unwrap_err()
            .code,
        ITEM_NAMING_UNPROVEN
    );
}

fn namespace(complete: bool, resources: Vec<ItemResourceKeyV1>) -> EffectiveResourceNamespaceV1 {
    EffectiveResourceNamespaceV1 {
        schema_version: 1,
        complete,
        inventories_sha256: vec![HASH.to_owned()],
        resources,
    }
}

fn resource(resref: &str, resource_type: u16, scope: ItemResourceScopeV1) -> ItemResourceKeyV1 {
    ItemResourceKeyV1 {
        resref: resref.to_owned(),
        resource_type,
        scope,
    }
}

#[test]
fn namespace_preflight_requires_complete_scope_and_blocks_casefolded_collisions() {
    let output = vec![resource("wswls_b_023", 2002, ItemResourceScopeV1::Output)];
    assert_eq!(
        preflight_item_namespace_v1(&output, &namespace(false, vec![]))
            .unwrap_err()
            .code,
        ITEM_NAMESPACE_INCOMPLETE
    );
    let occupied = namespace(
        true,
        vec![resource("WSWLS_B_023", 2002, ItemResourceScopeV1::Retail)],
    );
    assert_eq!(
        preflight_item_namespace_v1(&output, &occupied)
            .unwrap_err()
            .code,
        ITEM_RESOURCE_COLLISION
    );
    assert!(preflight_item_namespace_v1(&output, &namespace(true, vec![])).is_ok());
}

fn assembly(parts: [usize; 3]) -> Result<ItemAssemblyReportV1, m2a_core::item::ItemErrorV1> {
    validate_item_assembly_v1(
        ItemCompositionProfileV1::ModelType2,
        &[
            ItemPartGeometryV1 {
                slot: ItemPartSlotV1::Bottom,
                triangle_count: parts[0],
            },
            ItemPartGeometryV1 {
                slot: ItemPartSlotV1::Middle,
                triangle_count: parts[1],
            },
            ItemPartGeometryV1 {
                slot: ItemPartSlotV1::Top,
                triangle_count: parts[2],
            },
        ],
    )
}

#[test]
fn assembly_uses_one_shared_300k_budget_across_all_parts() {
    let exact = assembly([100_000, 100_000, 100_000]).unwrap();
    assert_eq!(exact.triangle_count, 300_000);
    assert_eq!(exact.triangle_budget, 300_000);
    assert!(exact.warning);
    assert_eq!(
        assembly([100_000, 100_000, 100_001]).unwrap_err().code,
        ITEM_TRIANGLE_BUDGET_EXCEEDED
    );
}
