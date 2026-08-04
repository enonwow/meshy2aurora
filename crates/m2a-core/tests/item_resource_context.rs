use m2a_core::{
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    item::{
        ItemResourceContextInputV1, ItemResourceProviderInputV1, ItemResourceProviderKindV1,
        inspect_item_resource_context_v1, resolve_item_resource_from_context_v1,
        select_effective_item_baseitem_v1,
    },
};

fn hak(resources: &[(&str, u16, &[u8])]) -> Vec<u8> {
    write_hak_v1(
        &resources
            .iter()
            .map(|(resref, resource_type, payload)| HakResourceInputV1 {
                resref: (*resref).to_owned(),
                resource_type: *resource_type,
                payload: (*payload).to_vec(),
            })
            .collect::<Vec<_>>(),
        &HakWriterOptionsV1::default(),
    )
    .unwrap()
    .payload
}

#[test]
fn effective_baseitem_selection_binds_the_winning_table_and_row_snapshot() {
    let baseitems = br#"2DA V2.0

Label ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight
1 longsword WSwLs 2 0 it_bag iwswls 0x1C030 1 4
"#;
    let custom = hak(&[("baseitems", 2017, baseitems)]);
    let context = ItemResourceContextInputV1 {
        schema_version: 1,
        resolution_policy: "SINGLE_CUSTOM_PROVIDER_OVER_VANILLA_V1".to_owned(),
        providers: vec![ItemResourceProviderInputV1 {
            provider_id: "hak:last_city".to_owned(),
            kind: ItemResourceProviderKindV1::Hak,
            order: 0,
            file_name: "last_city.hak".to_owned(),
            bytes: &custom,
        }],
    };

    let selection = select_effective_item_baseitem_v1(&context, 1).unwrap();
    assert_eq!(selection.winning_provider_id, "hak:last_city");
    assert_eq!(selection.selected.base_item, 1);
    assert_eq!(selection.selected.item_class, "WSwLs");
    assert_eq!(selection.selection_sha256.len(), 64);
}

#[test]
fn one_custom_hak_overrides_the_vanilla_inventory_with_full_provenance() {
    let vanilla = hak(&[
        ("baseitems", 2017, b"vanilla"),
        ("wswls_b_023", 2002, b"retail"),
    ]);
    let custom = hak(&[
        ("baseitems", 2017, b"custom"),
        ("wswls_b_251", 2002, b"generated"),
    ]);
    let context = ItemResourceContextInputV1 {
        schema_version: 1,
        resolution_policy: "SINGLE_CUSTOM_PROVIDER_OVER_VANILLA_V1".to_owned(),
        providers: vec![
            ItemResourceProviderInputV1 {
                provider_id: "vanilla:nwn_base".to_owned(),
                kind: ItemResourceProviderKindV1::Vanilla,
                order: 0,
                file_name: "nwn_base.hak-fixture".to_owned(),
                bytes: &vanilla,
            },
            ItemResourceProviderInputV1 {
                provider_id: "hak:last_city".to_owned(),
                kind: ItemResourceProviderKindV1::Hak,
                order: 0,
                file_name: "last_city.hak".to_owned(),
                bytes: &custom,
            },
        ],
    };

    let report = inspect_item_resource_context_v1(&context).unwrap();
    assert_eq!(report.status, "PASSED");
    assert_eq!(report.providers.len(), 2);
    assert_eq!(report.context_sha256.len(), 64);

    let resolved = resolve_item_resource_from_context_v1(&context, "baseitems", 2017).unwrap();
    assert_eq!(resolved.winning_provider_id, "hak:last_city");
    assert_eq!(resolved.payload, b"custom");
    assert_eq!(resolved.shadowed_provider_ids, ["vanilla:nwn_base"]);
}

#[test]
fn two_custom_providers_with_the_same_resource_fail_closed() {
    let first = hak(&[("baseitems", 2017, b"first")]);
    let second = hak(&[("baseitems", 2017, b"second")]);
    let context = ItemResourceContextInputV1 {
        schema_version: 1,
        resolution_policy: "SINGLE_CUSTOM_PROVIDER_OVER_VANILLA_V1".to_owned(),
        providers: vec![
            ItemResourceProviderInputV1 {
                provider_id: "hak:first".to_owned(),
                kind: ItemResourceProviderKindV1::Hak,
                order: 0,
                file_name: "first.hak".to_owned(),
                bytes: &first,
            },
            ItemResourceProviderInputV1 {
                provider_id: "hak:second".to_owned(),
                kind: ItemResourceProviderKindV1::Hak,
                order: 1,
                file_name: "second.hak".to_owned(),
                bytes: &second,
            },
        ],
    };

    let error = resolve_item_resource_from_context_v1(&context, "baseitems", 2017).unwrap_err();
    assert_eq!(
        error.code,
        "ITEM-RESOURCE-CONTEXT-AMBIGUOUS-CUSTOM-RESOURCE"
    );
}

#[test]
fn provider_order_and_payload_hashes_are_bound_into_context_identity() {
    let first = hak(&[("one", 2002, b"one")]);
    let second = hak(&[("two", 2002, b"two")]);
    let build = |first_order, second_order| ItemResourceContextInputV1 {
        schema_version: 1,
        resolution_policy: "SINGLE_CUSTOM_PROVIDER_OVER_VANILLA_V1".to_owned(),
        providers: vec![
            ItemResourceProviderInputV1 {
                provider_id: "hak:first".to_owned(),
                kind: ItemResourceProviderKindV1::Hak,
                order: first_order,
                file_name: "first.hak".to_owned(),
                bytes: &first,
            },
            ItemResourceProviderInputV1 {
                provider_id: "hak:second".to_owned(),
                kind: ItemResourceProviderKindV1::Hak,
                order: second_order,
                file_name: "second.hak".to_owned(),
                bytes: &second,
            },
        ],
    };

    let forward = inspect_item_resource_context_v1(&build(0, 1)).unwrap();
    let reverse = inspect_item_resource_context_v1(&build(1, 0)).unwrap();
    assert_ne!(forward.context_sha256, reverse.context_sha256);
}
