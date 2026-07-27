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
use sha2::{Digest, Sha256};

const CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\h2-r45-skin-bind-controllers-20260726";
const CONTRACT_FILE: &str = "h2-r45-skin-bind-controllers-lineage-contract-v1.json";
const MODEL_RESREF: &str = "m2a_h2p45";
const TEXTURE_RESREF: &str = "m2a_h2t45";
const MODULE_RESREF: &str = "m2a_h2r45";
const AREA_RESREF: &str = "m2a_h2a45";
const HAK_RESREF: &str = "m2a_h2r45";
const CREATURE_RESREF: &str = "m2a_h2utc45";

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
    if command.output.as_path() != Path::new(CANONICAL_OUTPUT) {
        return Err(format!(
            "M2A-H2-R45-OUTPUT-IDENTITY: exact output must be {CANONICAL_OUTPUT}"
        ));
    }
    if command.output.exists() {
        return Err(format!(
            "M2A-H2-R45-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }

    let source = read(&command.source_glb, "SOURCE")?;
    let appearance = read(&command.appearance_two_da, "APPEARANCE")?;
    let failure_evidence = read(&command.failure_evidence, "FAILURE-EVIDENCE")?;
    let failure_json: serde_json::Value = serde_json::from_slice(&failure_evidence)
        .map_err(|error| format!("M2A-H2-R45-FAILURE-EVIDENCE-JSON: {error}"))?;
    if failure_json["candidate"]["testModuleFilename"] != "m2a_h2r44.mod"
        || failure_json["runtimeVerdict"]["nwn"]["modelVisibility"] != "not_visible"
        || failure_json["postFailureAudit"]["confirmedConformanceDefect"]["code"]
            != "M2A-NWN-SKIN-BASE-CONTROLLERS-MISSING"
    {
        return Err(
            "M2A-H2-R45-FAILURE-EVIDENCE-DIFF: exact r44 failure and confirmed conformance defect required"
                .to_owned(),
        );
    }

    let identity = runtime_identity();
    let artifact = build_meshy_procedural_humanoid_model_package_with_identity_v1(
        &source,
        &appearance,
        &identity,
    )
    .map_err(|error| error.to_string())?;
    let model = verify_artifact(&artifact, &identity)?;
    let scene = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module)
        .map_err(|error| format!("M2A-H2-R45-MOD-READBACK: {error}"))?;
    let skin_node = find_single_skin_node(&model.node_tree.roots)?;

    let contract = serde_json::to_vec_pretty(&serde_json::json!({
        "schemaVersion": 1,
        "profile": "H2_R45_SKIN_BIND_CONTROLLERS_CANDIDATE_V1",
        "status": "offline_materialized_pending_installation",
        "candidateAdmissible": true,
        "admittedByOwnerFailure": {
            "path": command.failure_evidence,
            "byteLength": failure_evidence.len(),
            "sha256": sha256(&failure_evidence),
            "failedModule": "m2a_h2r44.mod",
            "modelVisibility": "not_visible"
        },
        "correctedConformanceDefect": {
            "code": "M2A-NWN-SKIN-BASE-CONTROLLERS-MISSING",
            "classification": "confirmed emitter conformance defect; not proven as the sole NWN invisibility root cause",
            "r44SkinControllerCount": 0,
            "nativeCepSkinNodeCount": 1437,
            "nativeCepZeroControllerSkinNodeCount": 0,
            "minimalNativeControllerTypes": [8, 20]
        },
        "minimalFunctionalDelta": [
            "preserve r44 source geometry, skin weights, bone references, inverse binds, texture, 42 type-5 states, 23 callbacks, appearance semantics, runtime creature profile and placement",
            "add one constant identity position controller and one constant identity orientation controller to the base SkinMesh node",
            "bind fresh r45 resource identities end-to-end without overwriting the failed r44 lineage"
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
        "modelReadback": {
            "rootName": model.node_tree.roots[0].name,
            "rootControllerCount": model.node_tree.roots[0].controllers.len(),
            "skinNodeName": skin_node.name,
            "skinControllerTypes": skin_node.controllers.iter().map(|controller| controller.controller_type).collect::<Vec<_>>(),
            "skinControllerValues": skin_node.controllers.iter().map(|controller| controller.values.clone()).collect::<Vec<_>>(),
            "animationCount": model.animations.len(),
            "animationTypes": model.animations.iter().map(|animation| animation.animation_type).collect::<Vec<_>>(),
            "skinNodeCount": count_skin_nodes(&model.node_tree.roots),
            "formatProfile": artifact.report.model.format_profile
        },
        "animationCompleteness": artifact.report.animation_completeness,
        "animationBehavior": artifact.report.animation_behavior,
        "animationEvents": artifact.report.animation_event_conformance,
        "skinAnimationConformance": artifact.report.skin_animation_conformance,
        "binaryScene": scene,
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
    .map_err(|error| format!("M2A-H2-R45-CONTRACT-SERIALIZE: {error}"))?;

    let generated = command.output.join("generated");
    let reports = command.output.join("reports");
    fs::create_dir_all(&generated)
        .and_then(|_| fs::create_dir_all(&reports))
        .map_err(|error| format!("M2A-H2-R45-OUTPUT-CREATE: {error}"))?;
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
            return Err(format!("M2A-H2-R45-OUTPUT-READBACK: {}", path.display()));
        }
    }

    serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "status": "H2_R45_SKIN_BIND_CONTROLLERS_CANDIDATE_OFFLINE_MATERIALIZED",
        "outputDirectory": command.output,
        "module": binding(MODULE_RESREF, &artifact.proof_module),
        "hak": binding(HAK_RESREF, &artifact.hak),
        "model": binding(MODEL_RESREF, &artifact.model),
        "texture": binding(TEXTURE_RESREF, &artifact.texture),
        "appearanceRow": artifact.report.appearance.appended_row_index,
        "areaResref": AREA_RESREF,
        "creatureResref": CREATURE_RESREF,
        "animationCount": model.animations.len(),
        "skinNodeCount": count_skin_nodes(&model.node_tree.roots),
        "skinControllerTypes": skin_node.controllers.iter().map(|controller| controller.controller_type).collect::<Vec<_>>(),
        "materializationCount": 1,
        "startsToolset": false,
        "startsNwn": false,
        "proofRunCreated": false
    }))
    .map_err(|error| format!("M2A-H2-R45-SUMMARY: {error}"))
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

fn verify_artifact(
    artifact: &M6ModelPackageArtifactV1,
    identity: &ProceduralCreaturePackageIdentityV1,
) -> Result<m2a_core::InspectionReport, String> {
    let archive = ErfArchive::parse(&artifact.hak)
        .map_err(|error| format!("M2A-H2-R45-HAK-READBACK: {error}"))?;
    for (resref, resource_type, expected) in [
        (MODEL_RESREF, 2002, artifact.model.as_slice()),
        (TEXTURE_RESREF, 3, artifact.texture.as_slice()),
        ("appearance", 2017, artifact.appearance_two_da.as_slice()),
    ] {
        let actual = archive
            .find(resref, resource_type)
            .map_err(|error| format!("M2A-H2-R45-HAK-RESOURCE: {error}"))?;
        if actual != expected {
            return Err(format!("M2A-H2-R45-HAK-RESOURCE-DIFF: {resref}"));
        }
    }

    let model = inspect_binary_mdl(&artifact.model)
        .map_err(|error| format!("M2A-H2-R45-MDL-READBACK: {error}"))?;
    let root = model
        .node_tree
        .roots
        .first()
        .ok_or_else(|| "M2A-H2-R45-MDL-ROOT-MISSING".to_owned())?;
    let skin_node = find_single_skin_node(&model.node_tree.roots)?;
    if root.name != identity.model_resref
        || !root.controllers.is_empty()
        || model.animations.len() != 42
        || model
            .animations
            .iter()
            .any(|animation| animation.animation_type != 5)
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
        return Err("M2A-H2-R45-MDL-CONTRACT-DIFF".to_owned());
    }

    let scene = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module)
        .map_err(|error| format!("M2A-H2-R45-MOD-READBACK: {error}"))?;
    if scene.scene.module_resref != identity.module.module_resref
        || scene.scene.area_resref != identity.module.area_resref
        || scene.scene.ordered_hak_resrefs != [identity.module.hak_resref.clone()]
        || scene.fixtures.len() != 1
        || scene.fixtures[0].fixture.template_resref != identity.creature_resref
        || scene.fixtures[0].runtime_profile
            != BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline
    {
        return Err("M2A-H2-R45-MOD-CONTRACT-DIFF".to_owned());
    }
    require_appearance_cell(
        &artifact.appearance_two_da,
        artifact.report.appearance.appended_row_index,
        "MODELTYPE",
        "S",
    )?;
    require_appearance_cell(
        &artifact.appearance_two_da,
        artifact.report.appearance.appended_row_index,
        "RACE",
        MODEL_RESREF,
    )?;
    Ok(model)
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
            "M2A-H2-R45-SKIN-COUNT: expected 1, got {}",
            skins.len()
        ));
    }
    Ok(skins[0])
}

fn require_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), String> {
    let limits = TwoDaLimitsV1::default();
    let inspection = inspect_two_da_v2(bytes, &limits)
        .map_err(|error| format!("M2A-H2-R45-2DA-INSPECT: {error}"))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| format!("M2A-H2-R45-2DA-COLUMN-MISSING: {column}"))?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &limits)
        .map_err(|error| format!("M2A-H2-R45-2DA-ROW: {error}"))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(format!(
            "M2A-H2-R45-2DA-CELL-DIFF: {column} expected {expected}"
        )),
    }
}

fn count_skin_nodes(roots: &[m2a_core::mdl::NodeReport]) -> usize {
    fn visit(node: &m2a_core::mdl::NodeReport) -> usize {
        usize::from(node.skin.is_some()) + node.children.iter().map(visit).sum::<usize>()
    }
    roots.iter().map(visit).sum()
}

fn binding(resref: &str, bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "resref": resref,
        "byteLength": bytes.len(),
        "sha256": sha256(bytes)
    })
}

struct Command {
    source_glb: PathBuf,
    appearance_two_da: PathBuf,
    failure_evidence: PathBuf,
    output: PathBuf,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut appearance_two_da = None;
    let mut failure_evidence = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--appearance-2da" => &mut appearance_two_da,
            "--failure-evidence" => &mut failure_evidence,
            "--out" => &mut output,
            _ => {
                return Err(format!("M2A-H2-R45-ARGUMENT: {argument}\n{}", usage()));
            }
        };
        if target.is_some() {
            return Err(format!("M2A-H2-R45-ARGUMENT-DUPLICATE: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("M2A-H2-R45-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        failure_evidence: PathBuf::from(failure_evidence.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("M2A-H2-R45-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("M2A-H2-R45-OUTPUT-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("M2A-H2-R45-OUTPUT-WRITE {}: {error}", path.display()))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn usage() -> String {
    "usage: materialize_h2_r45_skin_bind_candidate --source-glb <exact-H2.glb> --appearance-2da <exact-full-table> --failure-evidence <exact-r44-result.json> --out <exact-r45-dir>".to_owned()
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
            "--failure-evidence",
            "r44.json",
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
