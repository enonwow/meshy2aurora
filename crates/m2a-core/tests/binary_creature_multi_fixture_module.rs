use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::{GffLimitsV1, GffValueV1, GffWriterOptionsV1, read_gff_v32, write_gff_v32},
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_multi_fixture_module_v1,
        inspect_binary_creature_multi_fixture_module_v1,
    },
};

fn identity() -> BinaryCreatureModuleIdentityV1 {
    BinaryCreatureModuleIdentityV1 {
        module_resref: "m2a_diag01".to_owned(),
        area_resref: "m2a_diaga01".to_owned(),
        hak_resref: "m2a_diagh01".to_owned(),
    }
}

fn fixtures() -> Vec<BinaryCreatureOwnedFixtureV1> {
    vec![
        BinaryCreatureOwnedFixtureV1 {
            id: "stock_102".to_owned(),
            template_resref: "m2a_d_stock".to_owned(),
            display_name: "Stock appearance row 102".to_owned(),
            appearance_row: 102,
            position: M0RuntimePositionV1 {
                x: 5.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
        BinaryCreatureOwnedFixtureV1 {
            id: "meshy_m0".to_owned(),
            template_resref: "m2a_d_m0".to_owned(),
            display_name: "Exact Meshy M0 candidate".to_owned(),
            appearance_row: 15_100,
            position: M0RuntimePositionV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
        BinaryCreatureOwnedFixtureV1 {
            id: "meshy_h1".to_owned(),
            template_resref: "m2a_d_h1".to_owned(),
            display_name: "Exact Meshy H1 positive control".to_owned(),
            appearance_row: 15_101,
            position: M0RuntimePositionV1 {
                x: 15.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
    ]
}

fn assert_runtime_complete_creature(fields: &[m2a_core::gff::GffFieldV1]) {
    assert!(
        fields
            .iter()
            .any(|field| { field.label == "MaxHitPoints" && field.value == GffValueV1::Short(13) })
    );
    assert!(matches!(
        fields.iter().find(|field| field.label == "SkillList").map(|field| &field.value),
        Some(GffValueV1::List(skills))
            if skills.len() == 28
                && skills.iter().all(|skill|
                    skill.struct_id == 0
                        && skill.fields.as_slice() == [m2a_core::gff::GffFieldV1 {
                            label: "Rank".to_owned(),
                            value: GffValueV1::Byte(0),
                        }])
    ));
}

fn rewrite_runtime_envelope_field(
    module: &[u8],
    target_resref: &str,
    target_resource_type: u16,
    target_field: &str,
    replacement: GffValueV1,
) -> Vec<u8> {
    let archive = ErfArchive::parse(module).expect("generated MOD archive");
    let resources = archive
        .resources()
        .iter()
        .map(|resource| {
            let mut payload = archive
                .find(&resource.resref, resource.resource_type)
                .expect("resource payload")
                .to_vec();
            if resource.resref == target_resref && resource.resource_type == target_resource_type {
                let mut document = read_gff_v32(&payload, &GffLimitsV1::default())
                    .expect("target creature resource GFF");
                let fields = if target_resource_type == 2023 {
                    match document
                        .root
                        .fields
                        .iter_mut()
                        .find(|field| field.label == "Creature List")
                        .map(|field| &mut field.value)
                    {
                        Some(GffValueV1::List(creatures)) => &mut creatures[0].fields,
                        _ => panic!("GIT creature list"),
                    }
                } else {
                    &mut document.root.fields
                };
                let field = fields
                    .iter_mut()
                    .find(|field| field.label == target_field)
                    .expect("runtime envelope field");
                field.value = replacement.clone();
                payload = write_gff_v32(&document, &GffWriterOptionsV1::default())
                    .expect("rewritten test GFF")
                    .payload;
            }
            HakResourceInputV1 {
                resref: resource.resref.clone(),
                resource_type: resource.resource_type,
                payload,
            }
        })
        .collect::<Vec<_>>();
    write_erf_archive_v1(
        ErfFileType::Module,
        &resources,
        &HakWriterOptionsV1::default(),
    )
    .expect("rebuilt sparse test MOD")
    .payload
}

fn skill_list(count: usize, nonzero_rank: Option<(usize, u8)>) -> GffValueV1 {
    GffValueV1::List(
        (0..count)
            .map(|index| m2a_core::gff::GffStructV1 {
                struct_id: 0,
                fields: vec![m2a_core::gff::GffFieldV1 {
                    label: "Rank".to_owned(),
                    value: GffValueV1::Byte(
                        nonzero_rank
                            .filter(|(target, _)| *target == index)
                            .map(|(_, rank)| rank)
                            .unwrap_or(0),
                    ),
                }],
            })
            .collect(),
    )
}

#[test]
fn caller_owned_multi_fixture_module_round_trips_every_fixture() {
    let identity = identity();
    let fixtures = fixtures();
    let artifact = build_binary_creature_multi_fixture_module_v1(&identity, &fixtures)
        .expect("three-fixture diagnostic MOD");

    assert_eq!(artifact.byte_length, artifact.payload.len() as u64);
    assert_eq!(artifact.readback.module_resref, identity.module_resref);
    assert_eq!(artifact.readback.area_resref, identity.area_resref);
    assert_eq!(artifact.readback.ordered_hak_resrefs, [identity.hak_resref]);
    assert_eq!(artifact.readback.area_width, 2);
    assert_eq!(artifact.readback.area_height, 2);
    assert_eq!(artifact.readback.tileset_resref, "tms01");
    assert_eq!(artifact.readback.tile_count, 4);
    assert_eq!(
        artifact.readback.entry_position,
        M0RuntimePositionV1 {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        }
    );
    assert_eq!(
        artifact.readback.entry_direction,
        M0RuntimeDirectionV1 { x: 0.0, y: 1.0 }
    );
    assert_eq!(artifact.readback.fixtures, fixtures);

    let independent = inspect_binary_creature_multi_fixture_module_v1(&artifact.payload)
        .expect("independent exact readback");
    assert_eq!(independent, artifact.readback);

    let archive = ErfArchive::parse(&artifact.payload).expect("generated MOD archive");
    assert_eq!(archive.file_type(), ErfFileType::Module);
    assert_eq!(archive.resources().len(), 5 + fixtures.len());
    for fixture in &fixtures {
        let utc = archive
            .find(&fixture.template_resref, 2027)
            .expect("one exact UTC per fixture template");
        let utc = read_gff_v32(utc, &GffLimitsV1::default()).expect("runtime-complete UTC");
        assert_runtime_complete_creature(&utc.root.fields);
    }
    let git = read_gff_v32(
        archive.find(&identity.area_resref, 2023).expect("area GIT"),
        &GffLimitsV1::default(),
    )
    .expect("runtime-complete GIT");
    let creatures = match git
        .root
        .fields
        .iter()
        .find(|field| field.label == "Creature List")
        .map(|field| &field.value)
    {
        Some(GffValueV1::List(creatures)) => creatures,
        _ => panic!("GIT creature list"),
    };
    assert_eq!(creatures.len(), fixtures.len());
    for creature in creatures {
        assert_runtime_complete_creature(&creature.fields);
    }
}

#[test]
fn multi_fixture_input_rejects_ambiguous_or_invalid_owned_identity() {
    let identity = identity();

    let error = build_binary_creature_multi_fixture_module_v1(&identity, &[])
        .expect_err("empty fixture list must fail");
    assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-INPUT-INVALID");

    let mut duplicate_id = fixtures();
    duplicate_id[1].id = duplicate_id[0].id.clone();
    let error = build_binary_creature_multi_fixture_module_v1(&identity, &duplicate_id)
        .expect_err("duplicate fixture id must fail");
    assert_eq!(error.path, "fixtures[1].id");

    let mut duplicate_template = fixtures();
    duplicate_template[1].template_resref = duplicate_template[0].template_resref.clone();
    let error = build_binary_creature_multi_fixture_module_v1(&identity, &duplicate_template)
        .expect_err("duplicate UTC template must fail");
    assert_eq!(error.path, "fixtures[1].templateResref");

    let mut invalid_orientation = fixtures();
    invalid_orientation[0].orientation.x = f32::NAN;
    let error = build_binary_creature_multi_fixture_module_v1(&identity, &invalid_orientation)
        .expect_err("non-finite orientation must fail");
    assert_eq!(error.path, "fixtures[0].orientation");
}

#[test]
fn multi_fixture_readback_rejects_internal_utc_template_resref_drift() {
    let artifact = build_binary_creature_multi_fixture_module_v1(&identity(), &fixtures())
        .expect("three-fixture diagnostic MOD");
    let mut mutated = artifact.payload;
    let (utc_offset, utc_size) = {
        let archive = ErfArchive::parse(&mutated).expect("generated MOD archive");
        let resource = archive
            .resources()
            .iter()
            .find(|resource| resource.resref == "m2a_d_m0" && resource.resource_type == 2027)
            .expect("exact M0 UTC resource metadata");
        (resource.offset, resource.size)
    };
    let original = b"m2a_d_m0";
    let replacement = b"m2a_d_x0";
    let utc = &mut mutated[utc_offset..utc_offset + utc_size];
    let matches = utc
        .windows(original.len())
        .enumerate()
        .filter_map(|(offset, bytes)| (bytes == original).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "one internal UTC TemplateResRef value");
    utc[matches[0]..matches[0] + replacement.len()].copy_from_slice(replacement);

    let error = inspect_binary_creature_multi_fixture_module_v1(&mutated)
        .expect_err("internal UTC TemplateResRef must match its GIT/archive-key template");
    assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-READBACK-INVALID");
    assert_eq!(error.path, "fixtures[1].utc.TemplateResRef");
}

#[test]
fn multi_fixture_readback_rejects_sparse_runtime_envelope_in_git_or_utc() {
    let artifact = build_binary_creature_multi_fixture_module_v1(&identity(), &fixtures())
        .expect("three-fixture diagnostic MOD");
    for (resref, resource_type, invalid_field, replacement, expected_path) in [
        (
            "m2a_diaga01",
            2023,
            "MaxHitPoints",
            GffValueV1::Int(13),
            "area.git.Creature List[0].MaxHitPoints",
        ),
        (
            "m2a_diaga01",
            2023,
            "SkillList",
            GffValueV1::List(vec![m2a_core::gff::GffStructV1 {
                struct_id: 0,
                fields: vec![m2a_core::gff::GffFieldV1 {
                    label: "Rank".to_owned(),
                    value: GffValueV1::Word(0),
                }],
            }]),
            "area.git.Creature List[0].SkillList",
        ),
        (
            "m2a_d_stock",
            2027,
            "MaxHitPoints",
            GffValueV1::Int(13),
            "fixtures[0].utc.MaxHitPoints",
        ),
        (
            "m2a_d_stock",
            2027,
            "SkillList",
            GffValueV1::List(vec![m2a_core::gff::GffStructV1 {
                struct_id: 9,
                fields: vec![m2a_core::gff::GffFieldV1 {
                    label: "Rank".to_owned(),
                    value: GffValueV1::Byte(0),
                }],
            }]),
            "fixtures[0].utc.SkillList",
        ),
    ] {
        let sparse = rewrite_runtime_envelope_field(
            &artifact.payload,
            resref,
            resource_type,
            invalid_field,
            replacement,
        );
        let error = inspect_binary_creature_multi_fixture_module_v1(&sparse)
            .expect_err("malformed runtime creature field must fail readback");
        assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-READBACK-INVALID");
        assert_eq!(error.path, expected_path);
    }
}

#[test]
fn multi_fixture_readback_accepts_native_runtime_hp_and_skill_variants() {
    let artifact = build_binary_creature_multi_fixture_module_v1(&identity(), &fixtures())
        .expect("three-fixture diagnostic MOD");
    for (resref, resource_type, field, replacement) in [
        ("m2a_diaga01", 2023, "MaxHitPoints", GffValueV1::Short(58)),
        ("m2a_d_stock", 2027, "MaxHitPoints", GffValueV1::Short(1)),
        (
            "m2a_diaga01",
            2023,
            "SkillList",
            skill_list(23, Some((1, 10))),
        ),
        (
            "m2a_d_stock",
            2027,
            "SkillList",
            GffValueV1::List(Vec::new()),
        ),
    ] {
        let native_variant = rewrite_runtime_envelope_field(
            &artifact.payload,
            resref,
            resource_type,
            field,
            replacement,
        );
        inspect_binary_creature_multi_fixture_module_v1(&native_variant)
            .expect("native-valid HP and SkillList variants must remain admissible");
    }
}

#[test]
fn multi_fixture_readback_rejects_gff_type_and_core_resource_key_drift() {
    let artifact = build_binary_creature_multi_fixture_module_v1(&identity(), &fixtures())
        .expect("three-fixture diagnostic MOD");
    for (resref, resource_type, replacement, expected_path) in [
        ("module", 2014, *b"ARE ", "module.ifo.fileType"),
        ("m2a_diaga01", 2012, *b"GIT ", "area.are.fileType"),
        ("m2a_diaga01", 2023, *b"GIC ", "area.git.fileType"),
        ("m2a_diaga01", 2046, *b"IFO ", "area.gic.fileType"),
    ] {
        let mut mutated = artifact.payload.clone();
        let offset = {
            let archive = ErfArchive::parse(&mutated).expect("generated MOD archive");
            archive
                .resources()
                .iter()
                .find(|resource| {
                    resource.resref == resref && resource.resource_type == resource_type
                })
                .expect("exact core GFF resource metadata")
                .offset
        };
        mutated[offset..offset + replacement.len()].copy_from_slice(&replacement);
        let error = inspect_binary_creature_multi_fixture_module_v1(&mutated)
            .expect_err("resource-key type and GFF file type must agree exactly");
        assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-READBACK-INVALID");
        assert_eq!(error.path, expected_path);
    }

    let mut renamed_core_resource = artifact.payload;
    let (key_table_offset, repute_index) = {
        let archive =
            ErfArchive::parse(&renamed_core_resource).expect("generated MOD archive metadata");
        (
            u32::from_le_bytes(renamed_core_resource[24..28].try_into().unwrap()) as usize,
            archive
                .resources()
                .iter()
                .position(|resource| resource.resref == "repute" && resource.resource_type == 2038)
                .expect("required repute FAC resource"),
        )
    };
    let resref_offset = key_table_offset + repute_index * 24;
    renamed_core_resource[resref_offset..resref_offset + 16].fill(0);
    renamed_core_resource[resref_offset..resref_offset + 7].copy_from_slice(b"unknown");
    let error = inspect_binary_creature_multi_fixture_module_v1(&renamed_core_resource)
        .expect_err("same-count replacement of a required core resource must fail");
    assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-READBACK-INVALID");
    assert_eq!(error.path, "module.resources");
}
