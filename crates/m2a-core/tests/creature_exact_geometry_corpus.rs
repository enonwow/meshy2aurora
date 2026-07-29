use std::{fs, path::PathBuf};

use m2a_core::{
    model_pipeline::{
        ProceduralCreatureBuildOptionsV1, ProceduralCreaturePackageIdentityV1,
        ProceduralCreatureProductIdentityV2,
        build_meshy_procedural_humanoid_p100k_experiment_with_identity_v1,
        build_meshy_procedural_humanoid_p300k_experiment_with_identity_v1,
        build_meshy_procedural_humanoid_product_v2,
        build_meshy_procedural_humanoid_product_with_options_v3,
    },
    proof_module::BinaryCreatureModuleIdentityV1,
    skin_accessory::{
        SkinAccessoryComponentActionV1, SkinAccessoryStabilizationModeV1,
        SkinAccessoryStabilizationOptionsV1,
    },
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("canonical repository root")
        .to_path_buf()
}

fn appearance_fixture() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n".to_vec()
}

fn experiment_identity(prefix: &str) -> ProceduralCreaturePackageIdentityV1 {
    ProceduralCreaturePackageIdentityV1 {
        model_resref: format!("{prefix}mdl"),
        texture_resref: format!("{prefix}tex"),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: format!("{prefix}mod"),
            area_resref: format!("{prefix}area"),
            hak_resref: format!("{prefix}hak"),
        },
        creature_resref: format!("{prefix}utc"),
    }
}

fn assert_exact_triangle_lineage(
    expected: usize,
    ingest: usize,
    runtime_geometry: usize,
    written: usize,
) {
    assert_eq!(ingest, expected, "exact input sanitation changed geometry");
    assert_eq!(
        runtime_geometry, expected,
        "runtime face-plane sanitation changed geometry"
    );
    assert_eq!(
        written, expected,
        "binary MDL face-plane writer changed geometry"
    );
}

#[test]
#[ignore = "requires the local Git-ignored canonical Meshy Creature GLB corpus"]
fn product_20k_replays_through_exact_geometry_policy_without_face_loss() {
    let source =
        fs::read(repo_root().join("sample-3d/tlc-powrotnik-h1-p20k-v1/source-budget20000.glb"))
            .expect("canonical P20K Creature source");
    let artifact = build_meshy_procedural_humanoid_product_v2(
        &source,
        &appearance_fixture(),
        &ProceduralCreatureProductIdentityV2 {
            model_resref: "p0c20mdl".to_owned(),
            texture_resref: "p0c20tex".to_owned(),
            hak_resref: "p0c20hak".to_owned(),
            appearance_label: "P0_CREATURE_20K_EXACT".to_owned(),
        },
    )
    .expect("P20K exact Creature pipeline replay");

    assert_exact_triangle_lineage(
        19_892,
        artifact.report.ingest.statistics.triangle_count,
        artifact.report.geometry.triangle_count,
        artifact.report.model.projection.triangle_count,
    );
}

#[test]
#[ignore = "requires the local Git-ignored canonical Meshy Creature GLB corpus"]
fn p100k_replays_bounded_meshy_overshoot_without_face_loss() {
    let source = fs::read(repo_root().join("sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb"))
        .expect("canonical P100K Creature source");
    let artifact = build_meshy_procedural_humanoid_p100k_experiment_with_identity_v1(
        &source,
        &appearance_fixture(),
        &experiment_identity("p0c100"),
    )
    .expect("P100K exact Creature pipeline replay");

    assert_exact_triangle_lineage(
        102_335,
        artifact.report.ingest.statistics.triangle_count,
        artifact.report.geometry.triangle_count,
        artifact.report.model.projection.triangle_count,
    );
}

#[test]
#[ignore = "requires the local Git-ignored canonical Meshy Creature GLB corpus"]
fn p300k_replays_owner_verified_v5_geometry_without_face_loss() {
    let source = fs::read(repo_root().join("sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb"))
        .expect("canonical P300K Creature source");
    let artifact = build_meshy_procedural_humanoid_p300k_experiment_with_identity_v1(
        &source,
        &appearance_fixture(),
        &experiment_identity("p0c300"),
    )
    .expect("P300K exact Creature pipeline replay");

    assert_exact_triangle_lineage(
        296_276,
        artifact.report.ingest.statistics.triangle_count,
        artifact.report.geometry.triangle_count,
        artifact.report.model.projection.triangle_count,
    );
}

#[test]
#[ignore = "requires the local Git-ignored canonical Meshy Creature GLB corpus"]
fn product_300k_accepts_and_writes_owner_verified_stoneback_geometry_without_face_loss() {
    let source = fs::read(repo_root().join("sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb"))
        .expect("canonical P300K Creature source");
    let artifact = build_meshy_procedural_humanoid_product_v2(
        &source,
        &appearance_fixture(),
        &ProceduralCreatureProductIdentityV2 {
            model_resref: "p0p30mdl".to_owned(),
            texture_resref: "p0p30tex".to_owned(),
            hak_resref: "p0p30hak".to_owned(),
            appearance_label: "P0_PRODUCT_300K_EXACT".to_owned(),
        },
    )
    .expect("shared 300K product pipeline replay");

    assert_exact_triangle_lineage(
        296_276,
        artifact.report.ingest.statistics.triangle_count,
        artifact.report.geometry.triangle_count,
        artifact.report.model.projection.triangle_count,
    );
    assert!(artifact.report.model.projection.mesh_node_count > 1);
}

#[test]
#[ignore = "requires the local Git-ignored canonical Void Crystal Knight GLB"]
fn void_crystal_knight_auto_stabilizes_four_detached_crystals_without_geometry_loss() {
    let source = fs::read(repo_root().join("sample-3d/void-crystal-knight-h1-v1/source.glb"))
        .expect("canonical Void Crystal Knight source");
    let source_before = source.clone();
    let artifact = build_meshy_procedural_humanoid_product_with_options_v3(
        &source,
        &appearance_fixture(),
        &ProceduralCreatureProductIdentityV2 {
            model_resref: "vckregmdl".to_owned(),
            texture_resref: "vckregtex".to_owned(),
            hak_resref: "vckreghak".to_owned(),
            appearance_label: "VCK_REGRESSION".to_owned(),
        },
        &ProceduralCreatureBuildOptionsV1 {
            schema_version: 1,
            texture_artifact_cleanup: false,
            skin_accessory_stabilization: SkinAccessoryStabilizationOptionsV1 {
                schema_version: 1,
                mode: SkinAccessoryStabilizationModeV1::Auto,
                selected_bone_name: None,
            },
        },
    )
    .expect("Void Crystal Knight stabilized product replay");

    assert_eq!(
        source, source_before,
        "pipeline must not mutate source.glb bytes"
    );
    assert_exact_triangle_lineage(
        19_704,
        artifact.report.ingest.statistics.triangle_count,
        artifact.report.geometry.triangle_count,
        artifact.report.model.projection.triangle_count,
    );
    let report = &artifact.report.skin_accessory_stabilization;
    assert_eq!(report.component_count, 5);
    assert_eq!(report.detached_component_count, 4);
    assert_eq!(report.risky_component_count, 4);
    assert_eq!(report.stabilized_component_count, 4);
    assert!(report.changed_vertex_count > 0);
    assert_eq!(
        report
            .components
            .iter()
            .filter(|component| component.action == SkinAccessoryComponentActionV1::PrimaryBody)
            .count(),
        1
    );
    let stabilized = report
        .components
        .iter()
        .filter(|component| component.action == SkinAccessoryComponentActionV1::Stabilized)
        .collect::<Vec<_>>();
    assert_eq!(stabilized.len(), 4);
    assert!(stabilized.iter().all(|component| {
        component.after.max_pair_distance_ratio <= 1.000_01
            && component.after.max_pair_distance_error <= 0.000_01
    }));
    let selected_names = stabilized
        .iter()
        .filter_map(|component| component.selected_bone_name.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(
        selected_names
            .iter()
            .filter(|name| **name == "Spine02")
            .count(),
        2
    );
    assert_eq!(
        selected_names
            .iter()
            .filter(|name| **name == "Spine")
            .count(),
        2
    );
}
