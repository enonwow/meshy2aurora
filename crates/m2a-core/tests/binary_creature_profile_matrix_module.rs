use m2a_core::{
    erf::ErfArchive,
    gff::{GffLimitsV1, GffStructV1, GffValueV1, read_gff_v32},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1,
        BinaryCreatureProfiledFixtureV2, BinaryCreatureRuntimeProfileV2, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_profile_matrix_module_v2,
        inspect_binary_creature_multi_fixture_module_v1,
        inspect_binary_creature_profile_matrix_module_v2,
    },
};

fn identity() -> BinaryCreatureModuleIdentityV1 {
    BinaryCreatureModuleIdentityV1 {
        module_resref: "m2a_prof01".to_owned(),
        area_resref: "m2a_profa01".to_owned(),
        hak_resref: "m2a_profh01".to_owned(),
    }
}

fn fixture(
    id: &str,
    template_resref: &str,
    appearance_row: u16,
    x: f32,
    profile: BinaryCreatureRuntimeProfileV2,
) -> BinaryCreatureProfiledFixtureV2 {
    BinaryCreatureProfiledFixtureV2 {
        fixture: BinaryCreatureOwnedFixtureV1 {
            id: id.to_owned(),
            template_resref: template_resref.to_owned(),
            display_name: format!("{id} profile comparison"),
            appearance_row,
            position: M0RuntimePositionV1 { x, y: 14.5, z: 0.0 },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
        runtime_profile: profile,
    }
}

fn fixtures() -> Vec<BinaryCreatureProfiledFixtureV2> {
    vec![
        fixture(
            "legacy_102",
            "m2a_p_l102",
            102,
            5.0,
            BinaryCreatureRuntimeProfileV2::LegacyMinimal,
        ),
        fixture(
            "passive_102",
            "m2a_p_p102",
            102,
            10.0,
            BinaryCreatureRuntimeProfileV2::PassiveMonsterBaseline,
        ),
        fixture(
            "active_15100",
            "m2a_p_a15100",
            15_100,
            15.0,
            BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
        ),
    ]
}

fn field<'a>(creature: &'a GffStructV1, label: &str) -> &'a GffValueV1 {
    &creature
        .fields
        .iter()
        .find(|field| field.label == label)
        .unwrap_or_else(|| panic!("missing {label}"))
        .value
}

fn assert_monster_baseline(creature: &GffStructV1, active: bool) {
    assert_eq!(field(creature, "Race"), &GffValueV1::Byte(7));
    assert_eq!(field(creature, "Gender"), &GffValueV1::Byte(2));
    assert_eq!(field(creature, "Interruptable"), &GffValueV1::Byte(1));
    assert_eq!(field(creature, "WalkRate"), &GffValueV1::Int(7));
    assert_eq!(field(creature, "HitPoints"), &GffValueV1::Short(22));
    assert_eq!(field(creature, "CurrentHitPoints"), &GffValueV1::Short(22));
    assert_eq!(field(creature, "MaxHitPoints"), &GffValueV1::Short(37));
    assert_eq!(field(creature, "PerceptionRange"), &GffValueV1::Byte(11));
    assert!(matches!(
        field(creature, "ClassList"),
        GffValueV1::List(classes)
            if classes.as_slice() == [GffStructV1 {
                struct_id: 2,
                fields: vec![
                    m2a_core::gff::GffFieldV1 {
                        label: "Class".to_owned(),
                        value: GffValueV1::Int(11),
                    },
                    m2a_core::gff::GffFieldV1 {
                        label: "ClassLevel".to_owned(),
                        value: GffValueV1::Short(5),
                    },
                ],
            }]
    ));
    assert_eq!(
        field(creature, "FactionID"),
        &GffValueV1::Word(if active { 1 } else { 2 })
    );
    assert_eq!(
        field(creature, "ScriptHeartbeat"),
        &GffValueV1::ResRef(if active {
            "nw_c2_default1".to_owned()
        } else {
            String::new()
        })
    );
    assert_eq!(
        field(creature, "ScriptSpawn"),
        &GffValueV1::ResRef(if active {
            "nw_c2_default9".to_owned()
        } else {
            String::new()
        })
    );
}

#[test]
fn profile_matrix_round_trips_exact_profile_for_git_and_every_utc() {
    let fixtures = fixtures();
    let artifact = build_binary_creature_profile_matrix_module_v2(&identity(), &fixtures)
        .expect("profile comparison module");

    assert_eq!(artifact.readback.schema_version, 2);
    assert_eq!(artifact.readback.fixtures, fixtures);
    assert_eq!(artifact.readback.scene.fixtures.len(), fixtures.len());
    assert_eq!(
        inspect_binary_creature_profile_matrix_module_v2(&artifact.payload)
            .expect("independent V2 profile readback"),
        artifact.readback
    );

    let archive = ErfArchive::parse(&artifact.payload).expect("generated MOD archive");
    let git = read_gff_v32(
        archive.find("m2a_profa01", 2023).expect("profile GIT"),
        &GffLimitsV1::default(),
    )
    .expect("profile GIT GFF");
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
    assert_eq!(field(&creatures[0], "Race"), &GffValueV1::Byte(0));
    assert_eq!(
        field(&creatures[0], "ClassList"),
        &GffValueV1::List(vec![GffStructV1 {
            struct_id: 2,
            fields: vec![
                m2a_core::gff::GffFieldV1 {
                    label: "Class".to_owned(),
                    value: GffValueV1::Int(12),
                },
                m2a_core::gff::GffFieldV1 {
                    label: "ClassLevel".to_owned(),
                    value: GffValueV1::Short(12),
                },
            ],
        }])
    );
    assert_monster_baseline(&creatures[1], false);
    assert_monster_baseline(&creatures[2], true);

    for (index, fixture) in fixtures.iter().enumerate() {
        let utc = read_gff_v32(
            archive
                .find(&fixture.fixture.template_resref, 2027)
                .expect("one local UTC per profiled fixture"),
            &GffLimitsV1::default(),
        )
        .expect("profile UTC GFF");
        for label in [
            "Race",
            "Gender",
            "Phenotype",
            "FactionID",
            "Interruptable",
            "WalkRate",
            "HitPoints",
            "CurrentHitPoints",
            "MaxHitPoints",
            "PerceptionRange",
            "ScriptHeartbeat",
            "ScriptSpawn",
            "ClassList",
        ] {
            assert_eq!(
                field(&utc.root, label),
                field(&creatures[index], label),
                "{label} differs between GIT and UTC for fixture {index}"
            );
        }
    }
}

#[test]
fn profile_readback_rejects_unclassified_semantic_mix() {
    let artifact = build_binary_creature_profile_matrix_module_v2(&identity(), &fixtures())
        .expect("profile comparison module");
    let archive = ErfArchive::parse(&artifact.payload).expect("generated MOD archive");
    let resources = archive
        .resources()
        .iter()
        .map(|resource| {
            let payload = archive
                .find(&resource.resref, resource.resource_type)
                .expect("resource payload");
            m2a_core::hak::HakResourceInputV1 {
                resref: resource.resref.clone(),
                resource_type: resource.resource_type,
                payload: if resource.resref == "m2a_profa01" && resource.resource_type == 2023 {
                    let mut document =
                        read_gff_v32(payload, &GffLimitsV1::default()).expect("profile GIT GFF");
                    let creatures = match document
                        .root
                        .fields
                        .iter_mut()
                        .find(|field| field.label == "Creature List")
                        .map(|field| &mut field.value)
                    {
                        Some(GffValueV1::List(creatures)) => creatures,
                        _ => panic!("GIT creature list"),
                    };
                    creatures[1]
                        .fields
                        .iter_mut()
                        .find(|field| field.label == "WalkRate")
                        .expect("WalkRate")
                        .value = GffValueV1::Int(6);
                    m2a_core::gff::write_gff_v32(
                        &document,
                        &m2a_core::gff::GffWriterOptionsV1::default(),
                    )
                    .expect("mutated GIT")
                    .payload
                } else {
                    payload.to_vec()
                },
            }
        })
        .collect::<Vec<_>>();
    let mutated = m2a_core::hak::write_erf_archive_v1(
        m2a_core::erf::ErfFileType::Module,
        &resources,
        &m2a_core::hak::HakWriterOptionsV1::default(),
    )
    .expect("mutated profile MOD")
    .payload;

    let error = inspect_binary_creature_profile_matrix_module_v2(&mutated)
        .expect_err("unknown partial profile must fail closed");
    assert_eq!(error.code, "M0-BINARY-PROFILE-MATRIX-READBACK-INVALID");
    assert_eq!(error.path, "area.git.Creature List[1].runtimeProfile");
}

#[test]
fn base_scene_readback_rejects_non_default_creature_phenotype() {
    let artifact = build_binary_creature_profile_matrix_module_v2(&identity(), &fixtures())
        .expect("profile comparison module");
    let archive = ErfArchive::parse(&artifact.payload).expect("generated MOD archive");
    let resources = archive
        .resources()
        .iter()
        .map(|resource| {
            let payload = archive
                .find(&resource.resref, resource.resource_type)
                .expect("resource payload");
            m2a_core::hak::HakResourceInputV1 {
                resref: resource.resref.clone(),
                resource_type: resource.resource_type,
                payload: if resource.resref == "m2a_profa01" && resource.resource_type == 2023 {
                    let mut document =
                        read_gff_v32(payload, &GffLimitsV1::default()).expect("profile GIT GFF");
                    let creatures = match document
                        .root
                        .fields
                        .iter_mut()
                        .find(|field| field.label == "Creature List")
                        .map(|field| &mut field.value)
                    {
                        Some(GffValueV1::List(creatures)) => creatures,
                        _ => panic!("GIT creature list"),
                    };
                    creatures[0]
                        .fields
                        .iter_mut()
                        .find(|field| field.label == "Phenotype")
                        .expect("Phenotype")
                        .value = GffValueV1::Int(2);
                    m2a_core::gff::write_gff_v32(
                        &document,
                        &m2a_core::gff::GffWriterOptionsV1::default(),
                    )
                    .expect("mutated GIT")
                    .payload
                } else {
                    payload.to_vec()
                },
            }
        })
        .collect::<Vec<_>>();
    let mutated = m2a_core::hak::write_erf_archive_v1(
        m2a_core::erf::ErfFileType::Module,
        &resources,
        &m2a_core::hak::HakWriterOptionsV1::default(),
    )
    .expect("mutated profile MOD")
    .payload;

    let error = inspect_binary_creature_multi_fixture_module_v1(&mutated)
        .expect_err("base scene readback must bind the explicit default phenotype");
    assert_eq!(error.code, "M0-BINARY-MULTI-FIXTURE-READBACK-INVALID");
    assert_eq!(error.path, "area.git.Creature List[0].Phenotype");
}
