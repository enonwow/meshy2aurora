#[path = "../examples/materialize_tri_control_diagnostic.rs"]
mod materializer;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use sha2::{Digest, Sha256};

const BASE_SHA256: &str = "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
const LAST_CITY_BASE_SHA256: &str =
    "ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2";
const R31_MDL_SHA256: &str = "fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6";
const R31_TGA_SHA256: &str = "079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b";
const H1_APPEARANCE_SHA256: &str =
    "e50e2f8c5fc42771f5c433dd3984b0f8e740938d6f722adf838942f1f6342f8e";
const H1_MDL_SHA256: &str = "6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd";
const H1_TGA_SHA256: &str = "ab8f11c7f448801d905594802a223a1ed5969bc5d09ce06e6588cbe83b6a86b4";

#[test]
fn explicit_last_city_v2_selects_only_the_canonical_copy_and_caller_fresh_resrefs() {
    let workspace = TestWorkspace::new("last-city-parse-only");
    let arguments = last_city_arguments(&workspace.root.join("never-materialized"));
    let summary = materializer::test_parse_command_summary(arguments.clone())
        .expect("parse explicit Last City V2 without reading or materializing");
    assert_eq!(summary.profile, "last-city-v2");
    assert_eq!(summary.module_resref, "m2a_diaglc01");
    assert_eq!(summary.area_resref, "m2a_diagla01");
    assert_eq!(summary.hak_resref, "m2a_diaglh01");
    assert_eq!(summary.base_byte_length, 7_655_336);
    assert_eq!(summary.base_sha256, LAST_CITY_BASE_SHA256);
    assert_eq!(summary.declared_base_sha256, LAST_CITY_BASE_SHA256);
    assert_eq!(
        summary.base_relative_path,
        "proof-output/lc-hd-animals-c-squirrel-reference-audit-2026-07-18/source-copies/appearance.2da"
    );
    assert_eq!(
        summary.supplied_base_path,
        repo_root().join(&summary.base_relative_path)
    );
    assert!(!workspace.root.join("never-materialized").exists());

    let mut unsupported = arguments;
    replace_value(&mut unsupported, "--profile", "last-city-v3");
    let error = materializer::test_parse_command_summary(unsupported)
        .expect_err("unknown explicit profile must fail before any file read or output");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-PROFILE"), "{error}");
    assert!(!workspace.root.join("never-materialized").exists());
}

#[test]
fn cli_materializes_only_deterministic_offline_diagnostic_outputs_and_never_overwrites() {
    let workspace = TestWorkspace::new("deterministic");
    let first = workspace.root.join("first");

    let first_report = materializer::run_with_arguments(arguments(&first))
        .expect("first exact offline diagnostic materialization");

    assert!(first_report.diagnostic_only);
    assert!(!first_report.runtime_admissible);
    assert!(!first_report.starts_toolset);
    assert!(!first_report.starts_nwn);
    assert!(!first_report.installs_artifacts);
    assert_eq!(first_report.output_file_count, 4);
    assert_eq!(
        file_map(&first).keys().cloned().collect::<Vec<_>>(),
        vec![
            "m2a_diag01.mod",
            "m2a_diagh01.hak",
            "tri-control-diagnostic-contract-v1.json",
            "tri-control-diagnostic-profile-v1.json",
        ]
    );

    let contract: Value = read_json(&first.join("tri-control-diagnostic-contract-v1.json"));
    let profile: Value = read_json(&first.join("tri-control-diagnostic-profile-v1.json"));
    assert_eq!(contract["diagnosticOnly"], true);
    assert_eq!(contract["runtimeAdmissible"], false);
    assert_eq!(contract["resolverVerified"], false);
    assert_eq!(contract["rendererVerified"], false);
    assert_eq!(contract["fixtures"].as_array().unwrap().len(), 3);
    assert_eq!(
        contract["hakContract"]["resources"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert_eq!(profile["diagnosticOnly"], true);
    assert_eq!(profile["runtimeAdmissible"], false);
    assert_eq!(profile["fixtures"], contract["fixtures"]);
    assert_eq!(profile["resources"], contract["hakContract"]["resources"]);
    assert_eq!(profile["identity"]["moduleResref"], "m2a_diag01");
    assert_eq!(profile["identity"]["areaResref"], "m2a_diaga01");
    assert_eq!(profile["identity"]["hakResref"], "m2a_diagh01");
    assert_eq!(
        contract["fixtures"]
            .as_array()
            .unwrap()
            .iter()
            .map(|fixture| (
                fixture["role"].as_str().unwrap(),
                fixture["id"].as_str().unwrap(),
                fixture["templateResref"].as_str().unwrap(),
                fixture["appearanceRow"].as_u64().unwrap(),
                fixture["modelResref"].as_str().unwrap(),
                fixture["position"]["x"].as_f64().unwrap(),
                fixture["position"]["y"].as_f64().unwrap(),
                fixture["position"]["z"].as_f64().unwrap(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "stock_renderer_control",
                "stock_control",
                "m2a_d_stock",
                102,
                "c_horror",
                5.0,
                18.0,
                0.0,
            ),
            (
                "exact_r31_candidate",
                "candidate_m0",
                "m2a_d_m0",
                15_100,
                "m2a_m0p01",
                10.0,
                14.5,
                0.0,
            ),
            (
                "project_owned_positive_control",
                "custom_control_h1",
                "m2a_d_h1",
                15_101,
                "m2a_m6p01",
                15.0,
                18.0,
                0.0,
            ),
        ]
    );
    assert!(
        contract["fixtures"]
            .as_array()
            .unwrap()
            .iter()
            .all(|fixture| {
                fixture["orientation"]["x"] == 1.0 && fixture["orientation"]["y"] == 0.0
            })
    );
    assert_eq!(
        profile["appearanceRows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (
                row["physicalRow"].as_u64().unwrap(),
                row["modelType"].as_str().unwrap(),
                row["race"].as_str().unwrap(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (102, "S", "c_horror"),
            (15_100, "S", "m2a_m0p01"),
            (15_101, "S", "m2a_m6p01")
        ]
    );
    let resources = profile["resources"].as_array().unwrap();
    assert_eq!(
        resources
            .iter()
            .map(|resource| (
                resource["resref"].as_str().unwrap(),
                resource["resourceType"].as_u64().unwrap(),
            ))
            .collect::<Vec<_>>(),
        vec![
            ("appearance", 2017),
            ("m2a_m0p01", 2002),
            ("m2a_m0t01", 3),
            ("m2a_m6p01", 2002),
            ("m2a_m6t01", 3),
        ]
    );
    assert_eq!(resources[0]["sha256"], profile["appearance"]["sha256"]);
    assert_eq!(resources[1]["sha256"], R31_MDL_SHA256);
    assert_eq!(resources[2]["sha256"], R31_TGA_SHA256);
    assert_eq!(resources[3]["sha256"], H1_MDL_SHA256);
    assert_eq!(resources[4]["sha256"], H1_TGA_SHA256);
    let hashes = file_map(&first);
    assert_eq!(profile["module"]["sha256"], hashes["m2a_diag01.mod"]);
    assert_eq!(profile["hak"]["sha256"], hashes["m2a_diagh01.hak"]);
    assert_eq!(
        profile["contract"]["sha256"],
        hashes["tri-control-diagnostic-contract-v1.json"]
    );

    let before = file_map(&first);
    let error = materializer::run_with_arguments(arguments(&first))
        .expect_err("pre-existing output must never be reused or overwritten");
    assert!(
        error.contains("M2A-TRI-CONTROL-CLI-OUTPUT-EXISTS"),
        "{error}"
    );
    assert_eq!(file_map(&first), before);
}

#[test]
fn cli_rejects_wrong_declared_hash_exact_path_and_resref_before_output() {
    let workspace = TestWorkspace::new("negative");

    let wrong_hash_out = workspace.root.join("wrong-hash");
    let mut wrong_hash = arguments(&wrong_hash_out);
    replace_value(&mut wrong_hash, "--h1-model-sha256", &"0".repeat(64));
    let error = materializer::run_with_arguments(wrong_hash)
        .expect_err("wrong caller-declared exact hash must fail");
    assert!(
        error.contains("M2A-TRI-CONTROL-CLI-INPUT-DECLARED-HASH"),
        "{error}"
    );
    assert!(!wrong_hash_out.exists());

    let wrong_path_out = workspace.root.join("wrong-path");
    let mut wrong_path = arguments(&wrong_path_out);
    let r31_tga = argument_value(&wrong_path, "--r31-texture").to_owned();
    replace_value(&mut wrong_path, "--r31-model", &r31_tga);
    let error = materializer::run_with_arguments(wrong_path)
        .expect_err("alternate input path must fail even before payload use");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-INPUT-PATH"), "{error}");
    assert!(!wrong_path_out.exists());

    let wrong_resref_out = workspace.root.join("wrong-resref");
    let mut wrong_resref = arguments(&wrong_resref_out);
    replace_value(&mut wrong_resref, "--module-resref", "M2A_DIAG01");
    let error = materializer::run_with_arguments(wrong_resref)
        .expect_err("non-canonical resref must fail module builder");
    assert!(error.contains("M2A-BINARY-CREATURE"), "{error}");
    assert!(!wrong_resref_out.exists());
}

#[test]
fn publisher_is_no_clobber_under_destination_race_and_cleans_every_owned_failure_stage() {
    use materializer::PublicationTestFaultV1 as Fault;

    let workspace = TestWorkspace::new("publisher-faults");

    let partial = workspace.root.join("partial");
    let error = materializer::test_publish_tiny_outputs(&partial, Fault::PartialWrite)
        .expect_err("partial staging write must fail");
    assert!(
        error.contains("M2A-TRI-CONTROL-CLI-TEST-PARTIAL-WRITE"),
        "{error}"
    );
    assert!(!partial.exists());
    assert_no_publisher_siblings(&workspace.root, "partial");

    let raced = workspace.root.join("raced");
    let error = materializer::test_publish_tiny_outputs(&raced, Fault::DestinationRace)
        .expect_err("atomic no-replace publish must lose safely to an injected destination");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-PUBLISH"), "{error}");
    assert_eq!(
        fs::read(raced.join("attacker-sentinel.txt")).unwrap(),
        b"preserve me"
    );
    assert_no_publisher_siblings(&workspace.root, "raced");

    let verify = workspace.root.join("verify");
    let error = materializer::test_publish_tiny_outputs(&verify, Fault::PostPublishVerify)
        .expect_err("post-publish verifier failure must clean only the owned publication");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-TEST-VERIFY"), "{error}");
    assert!(!verify.exists());
    assert_no_publisher_siblings(&workspace.root, "verify");

    let staging_foreign = workspace.root.join("staging-foreign");
    let error = materializer::test_publish_tiny_outputs(
        &staging_foreign,
        Fault::ForeignAfterStagingValidation,
    )
    .expect_err("foreign staging child must block cleanup and survive");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-CLEANUP"), "{error}");
    let staging_leftover = only_publisher_sibling(&workspace.root, "staging-foreign", "stage");
    assert_eq!(
        fs::read(staging_leftover.join("foreign-sentinel.txt")).unwrap(),
        b"user bytes"
    );
    remove_test_directory_nonrecursive(&staging_leftover);

    let quarantine_foreign = workspace.root.join("quarantine-foreign");
    let error = materializer::test_publish_tiny_outputs(
        &quarantine_foreign,
        Fault::ForeignAfterQuarantineValidation,
    )
    .expect_err("foreign quarantine child must block cleanup and survive");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-CLEANUP"), "{error}");
    assert!(!quarantine_foreign.exists());
    let quarantine_leftover =
        only_publisher_sibling(&workspace.root, "quarantine-foreign", "quarantine");
    assert_eq!(
        fs::read(quarantine_leftover.join("foreign-sentinel.txt")).unwrap(),
        b"user bytes"
    );
    remove_test_directory_nonrecursive(&quarantine_leftover);

    let replacement = workspace.root.join("replacement");
    let error = materializer::test_publish_tiny_outputs(
        &replacement,
        Fault::ReplaceExpectedChildDuringCleanup,
    )
    .expect_err("replacement expected child must block cleanup and survive");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-CLEANUP"), "{error}");
    assert!(!replacement.exists());
    let replacement_leftover = only_publisher_sibling(&workspace.root, "replacement", "quarantine");
    assert_eq!(
        fs::read(replacement_leftover.join("one.bin")).unwrap(),
        b"replacement user bytes"
    );
    remove_test_directory_nonrecursive(&replacement_leftover);
}

#[cfg(windows)]
#[test]
fn cli_rejects_reparse_ancestors_and_dangling_reparse_destinations_before_build() {
    use std::os::windows::fs::symlink_dir;

    let workspace = TestWorkspace::new("reparse");
    let real_parent = workspace.root.join("real-parent");
    fs::create_dir(&real_parent).unwrap();
    let alias_parent = workspace.root.join("alias-parent");
    if let Err(error) = symlink_dir(&real_parent, &alias_parent) {
        if error.raw_os_error() == Some(1314) {
            return;
        }
        panic!("create test directory symlink: {error}");
    }
    let via_reparse = alias_parent.join("output");
    let error = materializer::run_with_arguments(arguments(&via_reparse))
        .expect_err("output ancestor reparse boundary must fail before build");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-REPARSE"), "{error}");
    assert!(!via_reparse.exists());

    let missing_target = workspace.root.join("missing-target");
    let dangling = workspace.root.join("dangling-output");
    symlink_dir(&missing_target, &dangling).unwrap();
    let error = materializer::run_with_arguments(arguments(&dangling))
        .expect_err("dangling destination reparse point must count as occupied");
    assert!(
        error.contains("M2A-TRI-CONTROL-CLI-OUTPUT-EXISTS")
            || error.contains("M2A-TRI-CONTROL-CLI-REPARSE"),
        "{error}"
    );
    assert!(fs::symlink_metadata(&dangling).is_ok());

    let repo_alias = workspace.root.join("repo-alias");
    symlink_dir(repo_root(), &repo_alias).unwrap();
    let pinned_reparse_out = workspace.root.join("pinned-reparse-output");
    let mut pinned_reparse = arguments(&pinned_reparse_out);
    replace_value(
        &mut pinned_reparse,
        "--base-appearance",
        &repo_alias
            .join("local-reference-assets/appearance.2da")
            .display()
            .to_string(),
    );
    let error = materializer::run_with_arguments(pinned_reparse)
        .expect_err("canonical payload through a reparse ancestor must still fail");
    assert!(error.contains("M2A-TRI-CONTROL-CLI-REPARSE"), "{error}");
    assert!(!pinned_reparse_out.exists());
}

#[cfg(windows)]
#[test]
fn publisher_contract_uses_win32_zero_flag_move_instead_of_std_replacing_rename() {
    let wrapper = include_str!("../../m2a-win32-noreplace/src/lib.rs");
    let materializer = include_str!("../examples/materialize_tri_control_diagnostic.rs");
    assert!(wrapper.contains("MoveFileExW(source, destination, 0)"));
    assert!(!materializer.contains("fs::rename("));
}

fn arguments(output: &Path) -> Vec<String> {
    let repo = repo_root();
    let r31 = repo.join("proof-output/m0-r31-hierarchy-only-20260722/generated");
    let h1 = repo.join("proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/generated");
    pairs([
        ("--outDir", output.display().to_string()),
        ("--module-resref", "m2a_diag01".to_owned()),
        ("--area-resref", "m2a_diaga01".to_owned()),
        ("--hak-resref", "m2a_diagh01".to_owned()),
        (
            "--base-appearance",
            repo.join("local-reference-assets/appearance.2da")
                .display()
                .to_string(),
        ),
        ("--base-appearance-sha256", BASE_SHA256.to_owned()),
        (
            "--r31-model",
            r31.join("m2a_m0p01.mdl").display().to_string(),
        ),
        ("--r31-model-sha256", R31_MDL_SHA256.to_owned()),
        (
            "--r31-texture",
            r31.join("m2a_m0t01.tga").display().to_string(),
        ),
        ("--r31-texture-sha256", R31_TGA_SHA256.to_owned()),
        (
            "--h1-appearance",
            h1.join("appearance.2da").display().to_string(),
        ),
        ("--h1-appearance-sha256", H1_APPEARANCE_SHA256.to_owned()),
        ("--h1-model", h1.join("m2a_m6p01.mdl").display().to_string()),
        ("--h1-model-sha256", H1_MDL_SHA256.to_owned()),
        (
            "--h1-texture",
            h1.join("m2a_m6t01.tga").display().to_string(),
        ),
        ("--h1-texture-sha256", H1_TGA_SHA256.to_owned()),
    ])
}

fn last_city_arguments(output: &Path) -> Vec<String> {
    let repo = repo_root();
    let mut values = arguments(output);
    values.splice(0..0, ["--profile".to_owned(), "last-city-v2".to_owned()]);
    replace_value(&mut values, "--module-resref", "m2a_diaglc01");
    replace_value(&mut values, "--area-resref", "m2a_diagla01");
    replace_value(&mut values, "--hak-resref", "m2a_diaglh01");
    replace_value(
        &mut values,
        "--base-appearance",
        &repo
            .join(
                "proof-output/lc-hd-animals-c-squirrel-reference-audit-2026-07-18/source-copies/appearance.2da",
            )
            .display()
            .to_string(),
    );
    replace_value(
        &mut values,
        "--base-appearance-sha256",
        LAST_CITY_BASE_SHA256,
    );
    values
}

fn pairs<const N: usize>(values: [(&str, String); N]) -> Vec<String> {
    values
        .into_iter()
        .flat_map(|(key, value)| [key.to_owned(), value])
        .collect()
}

fn replace_value(arguments: &mut [String], key: &str, value: &str) {
    let index = arguments
        .iter()
        .position(|argument| argument == key)
        .unwrap();
    arguments[index + 1] = value.to_owned();
}

fn argument_value<'a>(arguments: &'a [String], key: &str) -> &'a str {
    let index = arguments
        .iter()
        .position(|argument| argument == key)
        .unwrap();
    &arguments[index + 1]
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn file_map(root: &Path) -> BTreeMap<String, String> {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            (
                entry.file_name().to_string_lossy().into_owned(),
                sha256(&fs::read(entry.path()).unwrap()),
            )
        })
        .collect()
}

fn assert_no_publisher_siblings(parent: &Path, output_name: &str) {
    let prefixes = [
        format!(".{output_name}.m2a-tri-control-stage-"),
        format!(".{output_name}.m2a-tri-control-quarantine-"),
    ];
    let leftovers = fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| prefixes.iter().any(|prefix| name.starts_with(prefix)))
        .collect::<Vec<_>>();
    assert!(leftovers.is_empty(), "publisher leftovers: {leftovers:?}");
}

fn only_publisher_sibling(parent: &Path, output_name: &str, kind: &str) -> PathBuf {
    let prefix = format!(".{output_name}.m2a-tri-control-{kind}-");
    let matches = fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "expected one {kind} leftover: {matches:?}"
    );
    matches[0].clone()
}

fn remove_test_directory_nonrecursive(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::remove_file(entry.path()).unwrap();
    }
    fs::remove_dir(path).unwrap();
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = repo_root().join(format!(
            "target/tri-control-cli-tests/{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
