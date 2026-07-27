use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    inspect_binary_mdl,
    model_pipeline::{
        M6ModelPackageArtifactV1, ProceduralCreaturePackageIdentityV1,
        build_meshy_procedural_humanoid_model_package_with_identity_v1,
    },
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureRuntimeProfileV2,
        inspect_binary_creature_profile_matrix_module_v2,
    },
    two_da::{TwoDaCellValueV1, TwoDaLimitsV1, inspect_two_da_v2, read_two_da_row_v2},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\h2-r46-root-first-full-appearance-20260727";
const CONTRACT_FILE: &str = "h2-r46-root-first-full-appearance-lineage-contract-v1.json";
const MODEL_RESREF: &str = "m2a_h2p46";
const TEXTURE_RESREF: &str = "m2a_h2t46";
const MODULE_RESREF: &str = "m2a_h2r46";
const AREA_RESREF: &str = "m2a_h2a46";
const HAK_RESREF: &str = "m2a_h2r46";
const CREATURE_RESREF: &str = "m2a_h2utc46";
const DONOR_ROW: u32 = 102;
const EXPECTED_APPEARANCE_ROW: u16 = 15_220;

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let command = parse(env::args().skip(1))?;
    if command.output != PathBuf::from(CANONICAL_OUTPUT) {
        return Err(format!(
            "M2A-H2-R46-OUTPUT-IDENTITY: exact output must be {CANONICAL_OUTPUT}"
        ));
    }
    if command.output.exists() {
        return Err(format!(
            "M2A-H2-R46-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }

    let source = read(&command.source_glb, "SOURCE")?;
    let appearance = read(&command.appearance_two_da, "APPEARANCE")?;
    let admission_bytes = read(&command.admission, "ADMISSION")?;
    let admission: Value = serde_json::from_slice(&admission_bytes)
        .map_err(|error| format!("M2A-H2-R46-ADMISSION-JSON: {error}"))?;
    verify_admission(&admission, &source, &appearance)?;

    let identity = runtime_identity();
    let artifact = build_meshy_procedural_humanoid_model_package_with_identity_v1(
        &source,
        &appearance,
        &identity,
    )
    .map_err(|error| error.to_string())?;
    let verification = verify_artifact(&artifact, &identity, &admission)?;

    let contract = serde_json::to_vec_pretty(&serde_json::json!({
        "schemaVersion": 1,
        "profile": "H2_R46_ROOT_FIRST_FULL_APPEARANCE_CANDIDATE_V1",
        "status": "offline_materialized_pending_installation",
        "candidateAdmissible": true,
        "admission": binding("h2-r45-post-failure-r46-admission", &admission_bytes),
        "minimalFunctionalDelta": [
            "renumber the base model and all local animation hierarchies in root-first preorder, with every root at part 0 and all references remapped consistently",
            "clone every one of the 35 cells from direct-S donor physical row 102 and change only LABEL and RACE for appended row 15220",
            "bind fresh r46 resource identities end-to-end without overwriting failed r45"
        ],
        "preservedContracts": [
            "exact source GLB",
            "exact texture pixels",
            "42 local animation states with type 5 and 23 authored callback events",
            "base SkinMesh position and orientation controllers",
            "ActiveMonsterBaseline GIT/UTC profile with Phenotype=INT 0",
            "Area geometry, player entry and creature placement"
        ],
        "identity": identity,
        "source": binding("source", &source),
        "baseAppearance": binding("appearance", &appearance),
        "model": binding(MODEL_RESREF, &artifact.model),
        "texture": binding(TEXTURE_RESREF, &artifact.texture),
        "appearanceTwoDa": binding("appearance", &artifact.appearance_two_da),
        "hak": binding(HAK_RESREF, &artifact.hak),
        "module": binding(MODULE_RESREF, &artifact.proof_module),
        "appearanceRow": artifact.report.appearance.appended_row_index,
        "offlineVerification": verification,
        "animationCompleteness": artifact.report.animation_completeness,
        "animationBehavior": artifact.report.animation_behavior,
        "animationEvents": artifact.report.animation_event_conformance,
        "skinAnimationConformance": artifact.report.skin_animation_conformance,
        "packageManifest": artifact.package_manifest,
        "toolset": {
            "modelVisibility": "not_tested",
            "proofCompleteness": "missing"
        },
        "nwn": {
            "modelVisibility": "not_tested",
            "proofCompleteness": "missing"
        }
    }))
    .map_err(|error| format!("M2A-H2-R46-CONTRACT-SERIALIZE: {error}"))?;

    let generated = command.output.join("generated");
    let reports = command.output.join("reports");
    fs::create_dir_all(&generated)
        .and_then(|_| fs::create_dir_all(&reports))
        .map_err(|error| format!("M2A-H2-R46-OUTPUT-CREATE: {error}"))?;
    let outputs = [
        (generated.join("source.glb"), artifact.source_glb.as_slice()),
        (
            generated.join(format!("{MODEL_RESREF}.mdl")),
            artifact.model.as_slice(),
        ),
        (
            generated.join(format!("{TEXTURE_RESREF}.tga")),
            artifact.texture.as_slice(),
        ),
        (
            generated.join("appearance.2da"),
            artifact.appearance_two_da.as_slice(),
        ),
        (
            generated.join(format!("{HAK_RESREF}.hak")),
            artifact.hak.as_slice(),
        ),
        (
            generated.join(format!("{MODULE_RESREF}.mod")),
            artifact.proof_module.as_slice(),
        ),
        (
            reports.join("materialization-manifest.json"),
            artifact.manifest_json.as_slice(),
        ),
        (
            reports.join("materialization-report.json"),
            artifact.report_json.as_slice(),
        ),
        (
            reports.join("summary.json"),
            artifact.summary_json.as_slice(),
        ),
        (command.output.join(CONTRACT_FILE), contract.as_slice()),
    ];
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        if read(path, "WRITTEN-OUTPUT")? != *expected {
            return Err(format!("M2A-H2-R46-OUTPUT-READBACK: {}", path.display()));
        }
    }

    serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "status": "H2_R46_ROOT_FIRST_FULL_APPEARANCE_CANDIDATE_OFFLINE_MATERIALIZED",
        "outputDirectory": command.output,
        "module": binding(MODULE_RESREF, &artifact.proof_module),
        "hak": binding(HAK_RESREF, &artifact.hak),
        "model": binding(MODEL_RESREF, &artifact.model),
        "texture": binding(TEXTURE_RESREF, &artifact.texture),
        "appearanceRow": artifact.report.appearance.appended_row_index,
        "areaResref": AREA_RESREF,
        "creatureResref": CREATURE_RESREF,
        "modelRootPartNumber": verification["modelRootPartNumber"],
        "animationRootPartNumbersAreZero": verification["animationRootPartNumbersAreZero"],
        "appearanceDonorCloneVerified": verification["appearanceDonorCloneVerified"],
        "phenotypeContract": verification["phenotypeContract"],
        "materializationCount": 1,
        "startsToolset": false,
        "startsNwn": false,
        "proofRunCreated": false
    }))
    .map_err(|error| format!("M2A-H2-R46-SUMMARY: {error}"))
}

fn runtime_identity() -> ProceduralCreaturePackageIdentityV1 {
    ProceduralCreaturePackageIdentityV1 {
        model_resref: MODEL_RESREF.to_owned(),
        texture_resref: TEXTURE_RESREF.to_owned(),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: MODULE_RESREF.to_owned(),
            area_resref: AREA_RESREF.to_owned(),
            hak_resref: HAK_RESREF.to_owned(),
        },
        creature_resref: CREATURE_RESREF.to_owned(),
    }
}

fn verify_admission(admission: &Value, source: &[u8], appearance: &[u8]) -> Result<(), String> {
    if admission["status"] != "r46_materialization_admitted_not_yet_materialized"
        || admission["failedCandidate"]["testModuleFilename"] != "m2a_h2r45.mod"
        || admission["failedCandidate"]["nwn"]["modelVisibility"] != "not_visible"
        || admission["iterationGate"]["newIterationAdmitted"] != true
        || admission["iterationGate"]["diagnosedCauseRecorded"] != true
        || admission["iterationGate"]["minimalIntendedArtifactDeltaRecorded"] != true
        || admission["admittedCandidate"]["testModuleFilename"] != "m2a_h2r46.mod"
        || admission["admittedCandidate"]["modelResref"] != MODEL_RESREF
        || admission["admittedCandidate"]["textureResref"] != TEXTURE_RESREF
        || admission["admittedCandidate"]["areaResref"] != AREA_RESREF
        || admission["admittedCandidate"]["creatureTemplateResref"] != CREATURE_RESREF
        || admission["admittedCandidate"]["inputBindings"]["newAppearanceRowExpectedPhysicalIndex"]
            != u64::from(EXPECTED_APPEARANCE_ROW)
        || admission["admittedCandidate"]["inputBindings"]["directSDonorPhysicalRow"]
            != u64::from(DONOR_ROW)
    {
        return Err("M2A-H2-R46-ADMISSION-DIFF: exact admitted r46 contract required".to_owned());
    }

    require_json_hash(
        admission,
        &["admittedCandidate", "inputBindings", "sourceGlbSha256"],
        source,
        "SOURCE",
    )?;
    require_json_hash(
        admission,
        &["admittedCandidate", "inputBindings", "baseAppearanceSha256"],
        appearance,
        "APPEARANCE",
    )?;
    for (path_keys, hash_keys, label) in [
        (
            &["failedCandidate", "ownerFailureEvidence", "path"][..],
            &["failedCandidate", "ownerFailureEvidence", "sha256"][..],
            "OWNER-FAILURE",
        ),
        (
            &["postFailureAudit", "path"][..],
            &["postFailureAudit", "sha256"][..],
            "POST-FAILURE-AUDIT",
        ),
    ] {
        let path = json_string(admission, path_keys, label)?;
        let bytes = read(Path::new(path), label)?;
        let expected = json_string(admission, hash_keys, label)?;
        if sha256(&bytes) != expected {
            return Err(format!("M2A-H2-R46-{label}-HASH-DIFF: {path}"));
        }
    }
    Ok(())
}

fn verify_artifact(
    artifact: &M6ModelPackageArtifactV1,
    identity: &ProceduralCreaturePackageIdentityV1,
    admission: &Value,
) -> Result<Value, String> {
    let archive = ErfArchive::parse(&artifact.hak)
        .map_err(|error| format!("M2A-H2-R46-HAK-READBACK: {error}"))?;
    for (resref, resource_type, expected) in [
        (MODEL_RESREF, 2002, artifact.model.as_slice()),
        (TEXTURE_RESREF, 3, artifact.texture.as_slice()),
        ("appearance", 2017, artifact.appearance_two_da.as_slice()),
    ] {
        let actual = archive
            .find(resref, resource_type)
            .map_err(|error| format!("M2A-H2-R46-HAK-RESOURCE: {error}"))?;
        if actual != expected {
            return Err(format!("M2A-H2-R46-HAK-RESOURCE-DIFF: {resref}"));
        }
    }

    let model = inspect_binary_mdl(&artifact.model)
        .map_err(|error| format!("M2A-H2-R46-MDL-READBACK: {error}"))?;
    let root = model
        .node_tree
        .roots
        .first()
        .ok_or_else(|| "M2A-H2-R46-MDL-ROOT-MISSING".to_owned())?;
    let skin_node = find_single_skin_node(&model.node_tree.roots)?;
    let animation_roots_are_zero = model.animations.iter().all(|animation| {
        animation.node_tree.roots.len() == 1 && animation.node_tree.roots[0].number == 0
    });
    if root.name != identity.model_resref
        || root.number != 0
        || model.animations.len() != 42
        || model
            .animations
            .iter()
            .any(|animation| animation.animation_type != 5)
        || !animation_roots_are_zero
        || skin_node
            .controllers
            .iter()
            .map(|controller| controller.controller_type)
            .collect::<Vec<_>>()
            != [8, 20]
        || skin_node.controllers[0].times != [0.0]
        || skin_node.controllers[0].values != [[0.0, 0.0, 0.0]]
        || skin_node.controllers[1].times != [0.0]
        || skin_node.controllers[1].values != [[0.0, 0.0, 0.0, 1.0]]
    {
        return Err("M2A-H2-R46-MDL-CONTRACT-DIFF".to_owned());
    }

    let expected_texture_hash = json_string(
        admission,
        &[
            "admittedCandidate",
            "inputBindings",
            "preservedTextureSha256",
        ],
        "TEXTURE",
    )?;
    if sha256(&artifact.texture) != expected_texture_hash {
        return Err("M2A-H2-R46-TEXTURE-HASH-DIFF".to_owned());
    }

    let scene = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module)
        .map_err(|error| format!("M2A-H2-R46-MOD-READBACK: {error}"))?;
    if scene.scene.module_resref != identity.module.module_resref
        || scene.scene.area_resref != identity.module.area_resref
        || scene.scene.ordered_hak_resrefs != [identity.module.hak_resref.clone()]
        || scene.fixtures.len() != 1
        || scene.fixtures[0].fixture.template_resref != identity.creature_resref
        || scene.fixtures[0].runtime_profile
            != BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
    {
        return Err("M2A-H2-R46-MOD-CONTRACT-DIFF".to_owned());
    }

    verify_full_appearance_clone(
        &artifact.appearance_two_da,
        artifact.report.appearance.appended_row_index,
    )?;
    Ok(serde_json::json!({
        "modelRootPartNumber": root.number,
        "animationCount": model.animations.len(),
        "animationRootPartNumbersAreZero": animation_roots_are_zero,
        "skinControllerTypes": skin_node.controllers.iter().map(|value| value.controller_type).collect::<Vec<_>>(),
        "appearanceDonorPhysicalRow": DONOR_ROW,
        "appearanceRow": artifact.report.appearance.appended_row_index,
        "appearanceColumnCount": 35,
        "appearanceDonorCloneVerified": true,
        "appearanceChangedColumns": ["LABEL", "RACE"],
        "phenotypeContract": "explicit_int_zero_in_git_and_utc_validated_by_module_readback",
        "binaryScene": scene
    }))
}

fn verify_full_appearance_clone(bytes: &[u8], appended_row: u16) -> Result<(), String> {
    if appended_row != EXPECTED_APPEARANCE_ROW {
        return Err(format!(
            "M2A-H2-R46-APPEARANCE-ROW-DIFF: expected {EXPECTED_APPEARANCE_ROW}, got {appended_row}"
        ));
    }
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits)
        .map_err(|error| format!("M2A-H2-R46-2DA-INSPECT: {error}"))?;
    if inspection.columns.len() != 35 {
        return Err(format!(
            "M2A-H2-R46-2DA-WIDTH-DIFF: expected 35, got {}",
            inspection.columns.len()
        ));
    }
    let donor = read_two_da_row_v2(bytes, DONOR_ROW, &limits)
        .map_err(|error| format!("M2A-H2-R46-2DA-DONOR: {error}"))?;
    let appended = read_two_da_row_v2(bytes, u32::from(appended_row), &limits)
        .map_err(|error| format!("M2A-H2-R46-2DA-APPENDED: {error}"))?;
    for (index, column) in inspection.columns.iter().enumerate() {
        let actual = appended.cells.get(index);
        match column.to_ascii_uppercase().as_str() {
            "LABEL" => {
                if !matches!(actual, Some(TwoDaCellValueV1::Text { value }) if value == "M2A_M6_PROOF")
                {
                    return Err("M2A-H2-R46-2DA-LABEL-DIFF".to_owned());
                }
            }
            "RACE" => {
                if !matches!(actual, Some(TwoDaCellValueV1::Text { value }) if value == MODEL_RESREF)
                {
                    return Err("M2A-H2-R46-2DA-RACE-DIFF".to_owned());
                }
            }
            _ if actual != donor.cells.get(index) => {
                return Err(format!("M2A-H2-R46-2DA-DONOR-CELL-DIFF: {column}"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn find_single_skin_node(
    roots: &[m2a_core::mdl::NodeReport],
) -> Result<&m2a_core::mdl::NodeReport, String> {
    fn visit<'a>(
        node: &'a m2a_core::mdl::NodeReport,
        output: &mut Vec<&'a m2a_core::mdl::NodeReport>,
    ) {
        if node.skin.is_some() {
            output.push(node);
        }
        for child in &node.children {
            visit(child, output);
        }
    }
    let mut skins = Vec::new();
    for root in roots {
        visit(root, &mut skins);
    }
    if skins.len() != 1 {
        return Err(format!(
            "M2A-H2-R46-SKIN-COUNT: expected 1, got {}",
            skins.len()
        ));
    }
    Ok(skins[0])
}

fn require_json_hash(
    value: &Value,
    keys: &[&str],
    bytes: &[u8],
    label: &str,
) -> Result<(), String> {
    let expected = json_string(value, keys, label)?;
    if sha256(bytes) != expected {
        return Err(format!("M2A-H2-R46-{label}-HASH-DIFF"));
    }
    Ok(())
}

fn json_string<'a>(value: &'a Value, keys: &[&str], label: &str) -> Result<&'a str, String> {
    let mut current = value;
    for key in keys {
        current = &current[*key];
    }
    current
        .as_str()
        .ok_or_else(|| format!("M2A-H2-R46-{label}-ADMISSION-FIELD"))
}

fn binding(resref: &str, bytes: &[u8]) -> Value {
    serde_json::json!({
        "resref": resref,
        "byteLength": bytes.len(),
        "sha256": sha256(bytes)
    })
}

struct Command {
    source_glb: PathBuf,
    appearance_two_da: PathBuf,
    admission: PathBuf,
    output: PathBuf,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut appearance_two_da = None;
    let mut admission = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--appearance-2da" => &mut appearance_two_da,
            "--admission" => &mut admission,
            "--out" => &mut output,
            _ => {
                return Err(format!("M2A-H2-R46-ARGUMENT: {argument}\n{}", usage()));
            }
        };
        if target.is_some() {
            return Err(format!("M2A-H2-R46-ARGUMENT-DUPLICATE: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("M2A-H2-R46-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        admission: PathBuf::from(admission.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("M2A-H2-R46-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("M2A-H2-R46-OUTPUT-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("M2A-H2-R46-OUTPUT-WRITE {}: {error}", path.display()))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn usage() -> String {
    "usage: materialize_h2_r46_root_first_full_appearance_candidate --source-glb <exact-r45-source.glb> --appearance-2da <exact-r45-appearance.2da> --admission <exact-r46-admission.json> --out <exact-r46-dir>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::{CANONICAL_OUTPUT, parse};

    #[test]
    fn requires_exact_inputs_and_has_no_identity_override() {
        let arguments = [
            "--source-glb",
            "source.glb",
            "--appearance-2da",
            "appearance.2da",
            "--admission",
            "admission.json",
            "--out",
            CANONICAL_OUTPUT,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        assert!(parse(arguments.clone()).is_ok());
        for missing in (0..arguments.len()).step_by(2) {
            let mut changed = arguments.clone();
            changed.drain(missing..missing + 2);
            assert!(parse(changed).is_err());
        }
        let mut forbidden = arguments;
        forbidden.extend(["--model-resref".to_owned(), "other".to_owned()]);
        assert!(parse(forbidden).is_err());
    }
}
