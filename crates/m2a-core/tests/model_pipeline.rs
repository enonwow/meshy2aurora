use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
};

use m2a_core::{
    erf::ErfArchive,
    inspect_binary_mdl,
    model_pipeline::{
        M6_APPEARANCE_LABEL, M6_MODEL_RESREF, M6_TEXTURE_RESREF, build_m6_model_package_v1,
        build_m6_model_package_with_profile_v1, write_m6_proof_packet_v1,
    },
    owned_fixture::{
        synthetic_owned_m6_animation_mapping_v1, synthetic_owned_m6_glb_v1,
        synthetic_owned_m6_rig_v1,
    },
    profile_a::canonical_profile_sha256,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn appearance_fixture() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY\r\n0 Existing NORM P existing **** **** 0 R 1.0 4\r\n".to_vec()
}

fn appearance_fixture_without_phenotype() -> Vec<u8> {
    b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP BLOODCOLR WEAPONSCALE SIZECATEGORY\r\n0 Existing NORM P existing **** **** R 1.0 4\r\n".to_vec()
}

fn temp_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("m2a-{label}-{}", std::process::id()))
}

#[test]
fn owned_fixture_materializes_deterministically_through_the_complete_model_pipeline() {
    let glb = synthetic_owned_m6_glb_v1().expect("owned GLB fixture");
    let first = build_m6_model_package_v1(&glb, &appearance_fixture()).expect("first package");
    let second = build_m6_model_package_v1(&glb, &appearance_fixture()).expect("second package");

    assert_eq!(first.source_glb, second.source_glb);
    assert_eq!(first.model, second.model);
    assert_eq!(first.texture, second.texture);
    assert_eq!(first.appearance_two_da, second.appearance_two_da);
    assert_eq!(first.hak, second.hak);
    assert_eq!(first.manifest_json, second.manifest_json);
    assert_eq!(first.report_json, second.report_json);
    assert_eq!(first.summary_json, second.summary_json);

    assert_eq!(first.summary.status, "M6_MODEL_PACKAGE_MATERIALIZED");
    assert_eq!(first.summary.model_resref, M6_MODEL_RESREF);
    assert_eq!(first.summary.texture_resref, M6_TEXTURE_RESREF);
    assert_eq!(first.summary.appended_physical_row, 1);
    assert_eq!(first.summary.animation.output_name, "cpause1");
    assert!(first.summary.animation.has_motion);
    assert!(first.summary.provenance.export_allowed);
    assert!(first.summary.zero_reference_model_payload_copied);
    assert_eq!(
        first.summary.appearance_payload_policy,
        "PRESERVED_AND_APPENDED"
    );
    assert_eq!(first.summary.input_glb.sha256, hex_sha256(&glb));
    assert_eq!(
        first.summary.input_appearance_two_da.sha256,
        hex_sha256(&appearance_fixture())
    );
    assert_ne!(
        first.summary.input_appearance_two_da.sha256,
        first.summary.outputs.appearance_two_da.sha256
    );
    assert_eq!(first.report.resolved_base_color_image_index, 1);
    assert!(first.report.geometry.triangle_count > 1);
    assert!(first.report.geometry.bounds_min != first.report.geometry.bounds_max);
    assert_eq!(first.report.geometry.output_segment_deformation, "SKIN");
    assert_eq!(first.report.geometry.active_joint_count, 2);
    assert_eq!(first.report.texture_selection.source_texture_id, 1);
    assert_eq!(first.report.texture_selection.source_image_id, 1);
    assert_eq!(first.report.texture.width, 2);
    assert_eq!(first.report.texture.height, 2);
    assert_eq!(first.report.appearance.changed_cells.len(), 10);
    assert_eq!(
        first.report.appearance.changed_cells[0].column_name,
        "LABEL"
    );
    assert!(
        first
            .appearance_two_da
            .windows(M6_APPEARANCE_LABEL.len())
            .any(|w| w == M6_APPEARANCE_LABEL.as_bytes())
    );
    assert!(
        first
            .appearance_two_da
            .windows(b"m2a_m6p01".len())
            .any(|w| w == b"m2a_m6p01")
    );

    let mdl_readback = inspect_binary_mdl(&first.model).expect("binary MDL own readback");
    let cpause1 = mdl_readback
        .animations
        .iter()
        .find(|animation| animation.name == "cpause1")
        .expect("exact cpause1 readback");
    assert!(cpause1.node_tree.node_count >= 1);
    assert!(
        has_decoded_motion_controller(&serde_json::to_value(cpause1).unwrap()),
        "cpause1 own-readback must contain a decoded changing position controller"
    );

    let archive = ErfArchive::parse(&first.hak).expect("generated HAK readback");
    assert_eq!(archive.resources().len(), 3);
    assert_eq!(archive.find(M6_MODEL_RESREF, 2002).unwrap(), first.model);
    assert_eq!(archive.find(M6_TEXTURE_RESREF, 3).unwrap(), first.texture);
    assert_eq!(
        archive.find("appearance", 2017).unwrap(),
        first.appearance_two_da
    );

    let parsed_manifest: m2a_core::model_pipeline::M6MaterializationManifestV1 =
        serde_json::from_slice(&first.manifest_json).unwrap();
    assert_eq!(parsed_manifest, first.manifest);
    assert_eq!(
        parsed_manifest.package_manifest.package_sha256,
        first.summary.outputs.hak.sha256
    );
    assert_eq!(parsed_manifest.appended_physical_row, 1);
    assert_eq!(parsed_manifest.generated_files.len(), 8);
    assert_eq!(
        first.summary.outputs.proof_module.byte_length,
        first.proof_module.len() as u64
    );
    assert_eq!(
        first.summary.outputs.proof_module.sha256,
        hex_sha256(&first.proof_module)
    );
    assert_eq!(
        parsed_manifest.appearance_payload_policy,
        "PRESERVED_AND_APPENDED"
    );
    let summary_text = String::from_utf8(first.summary_json.clone()).unwrap();
    assert!(summary_text.contains("zeroReferenceModelPayloadCopied"));
    assert!(!summary_text.contains("zeroRetailPayloadCopied"));
    for file in &parsed_manifest.generated_files {
        let bytes = match file.relative_path.as_str() {
            "generated/source.glb" => first.source_glb.as_slice(),
            "generated/m2a_m6p01.mdl" => first.model.as_slice(),
            "generated/m2a_m6t01.tga" => first.texture.as_slice(),
            "generated/appearance.2da" => first.appearance_two_da.as_slice(),
            "generated/m2a_codex_aproof.hak" => first.hak.as_slice(),
            "generated/m2a_codex_aproof.mod" => first.proof_module.as_slice(),
            "reports/materialization-report.json" => first.report_json.as_slice(),
            "reports/summary.json" => first.summary_json.as_slice(),
            path => panic!("unexpected manifest path {path}"),
        };
        assert_eq!(file.byte_length, bytes.len() as u64);
        assert_eq!(file.sha256, hex_sha256(bytes));
    }
}

#[test]
fn appearance_without_any_phenotype_alias_is_legal_and_appends_nine_cells() {
    let source = synthetic_owned_m6_glb_v1().unwrap();
    let base = appearance_fixture_without_phenotype();
    let artifact = build_m6_model_package_v1(&source, &base).unwrap();

    assert_eq!(artifact.report.appearance.appended_row_index, 1);
    assert_eq!(artifact.report.appearance.changed_cells.len(), 9);
    assert!(artifact.appearance_two_da.starts_with(&base));
    assert_eq!(
        artifact.summary.appearance_payload_policy,
        "PRESERVED_AND_APPENDED"
    );
}

#[test]
fn invalid_or_incomplete_appearance_is_a_stable_error_and_never_panics() {
    let glb = synthetic_owned_m6_glb_v1().unwrap();
    for bytes in [
        b"not a 2da".as_slice(),
        b"2DA V2.0\n\nLABEL RACE\n0 Existing existing\n".as_slice(),
    ] {
        let result = catch_unwind(AssertUnwindSafe(|| build_m6_model_package_v1(&glb, bytes)));
        let error = result
            .expect("invalid appearance must not panic")
            .unwrap_err();
        assert_eq!(error.stage, "APPEARANCE");
        assert!(!error.code.is_empty());
    }
}

#[test]
fn missing_animation_base_color_and_ineligible_profile_are_stable_and_panic_free() {
    let appearance = appearance_fixture();
    let missing_animation = mutate_glb(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["animations"] = serde_json::json!([]);
    });
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_m6_model_package_v1(&missing_animation, &appearance)
    }));
    let error = result
        .expect("missing animation must not panic")
        .unwrap_err();
    assert_eq!(error.stage, "ANIMATION");
    assert!(error.code.starts_with("M4A-") || error.code == "M6-ANIMATION-MISSING");

    let missing_texture = mutate_glb(synthetic_owned_m6_glb_v1().unwrap(), |root| {
        root["materials"][0]["pbrMetallicRoughness"]
            .as_object_mut()
            .unwrap()
            .remove("baseColorTexture");
    });
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_m6_model_package_v1(&missing_texture, &appearance)
    }));
    let error = result.expect("missing texture must not panic").unwrap_err();
    assert_eq!(
        (error.stage.as_str(), error.code.as_str()),
        ("TEXTURE", "M6-BASE-COLOR-TEXTURE-MISSING")
    );

    let mut rig = synthetic_owned_m6_rig_v1().unwrap();
    for weights in &mut rig.segments[0].reference_weights {
        for influence in weights {
            influence.value = 0.0;
        }
    }
    rig.content_sha256.clear();
    rig.content_sha256 = canonical_profile_sha256(&rig).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_m6_model_package_with_profile_v1(
            &synthetic_owned_m6_glb_v1().unwrap(),
            &appearance,
            &rig,
            &synthetic_owned_m6_animation_mapping_v1(),
        )
    }));
    let error = result
        .expect("ineligible profile must not panic")
        .unwrap_err();
    assert_eq!(
        (error.stage.as_str(), error.code.as_str()),
        ("PROFILE", "M6-PROFILE-INELIGIBLE")
    );
}

#[test]
fn proof_packet_refuses_nonempty_output_and_never_overwrites_it() {
    let path = temp_path("m6-collision");
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("keep.txt"), b"owned by caller").unwrap();
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &appearance_fixture())
            .unwrap();

    let result = catch_unwind(AssertUnwindSafe(|| {
        write_m6_proof_packet_v1(&path, &artifact)
    }));
    let error = result.expect("collision must not panic").unwrap_err();
    assert_eq!(error.code, "M6-OUTPUT-EXISTS");
    assert_eq!(fs::read(path.join("keep.txt")).unwrap(), b"owned by caller");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn proof_packet_writes_generated_reports_and_empty_live_directories() {
    let path = temp_path("m6-write");
    let _ = fs::remove_dir_all(&path);
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &appearance_fixture())
            .unwrap();
    write_m6_proof_packet_v1(&path, &artifact).unwrap();

    assert_eq!(
        fs::read(path.join("generated/m2a_m6p01.mdl")).unwrap(),
        artifact.model
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_m6t01.tga")).unwrap(),
        artifact.texture
    );
    assert_eq!(
        fs::read(path.join("generated/appearance.2da")).unwrap(),
        artifact.appearance_two_da
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_codex_aproof.hak")).unwrap(),
        artifact.hak
    );
    assert_eq!(
        fs::read(path.join("generated/m2a_codex_aproof.mod")).unwrap(),
        artifact.proof_module
    );
    assert_eq!(
        fs::read(path.join("reports/materialization-manifest.json")).unwrap(),
        artifact.manifest_json
    );
    assert!(path.join("live").is_dir());
    assert_eq!(fs::read_dir(path.join("live")).unwrap().count(), 0);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn proof_packet_never_deletes_a_preexisting_staging_directory() {
    let output = temp_path("m6-stage-owner");
    let staging = output.parent().unwrap().join(format!(
        ".{}.m2a-stage-{}",
        output.file_name().unwrap().to_string_lossy(),
        std::process::id(),
    ));
    let _ = fs::remove_dir_all(&output);
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("keep.txt"), b"caller staging").unwrap();
    let artifact =
        build_m6_model_package_v1(&synthetic_owned_m6_glb_v1().unwrap(), &appearance_fixture())
            .unwrap();
    let error = write_m6_proof_packet_v1(&output, &artifact).unwrap_err();
    assert_eq!(error.code, "M6-STAGING-EXISTS");
    assert_eq!(
        fs::read(staging.join("keep.txt")).unwrap(),
        b"caller staging"
    );
    assert!(!output.exists());
    fs::remove_dir_all(staging).unwrap();
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn mutate_glb(mut glb: Vec<u8>, mutation: impl FnOnce(&mut Value)) -> Vec<u8> {
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let json_end = 20 + json_length;
    let mut root: Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
    let bin = glb.split_off(json_end + 8);
    mutation(&mut root);
    let mut json = serde_json::to_vec(&root).unwrap();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let length = 12 + 8 + json.len() + 8 + bin.len();
    let mut output = Vec::with_capacity(length);
    output.extend_from_slice(b"glTF");
    output.extend_from_slice(&2_u32.to_le_bytes());
    output.extend_from_slice(&(length as u32).to_le_bytes());
    output.extend_from_slice(&(json.len() as u32).to_le_bytes());
    output.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    output.extend_from_slice(&json);
    output.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    output.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
    output.extend_from_slice(&bin);
    output
}

fn has_decoded_motion_controller(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            let is_motion = object.get("decoded") == Some(&Value::Bool(true))
                && object.get("controllerName").and_then(Value::as_str) == Some("position")
                && object
                    .get("times")
                    .and_then(Value::as_array)
                    .is_some_and(|times| times.len() >= 2)
                && object
                    .get("values")
                    .and_then(Value::as_array)
                    .is_some_and(|values| {
                        values.len() >= 2 && values.windows(2).any(|pair| pair[0] != pair[1])
                    });
            is_motion || object.values().any(has_decoded_motion_controller)
        }
        Value::Array(values) => values.iter().any(has_decoded_motion_controller),
        _ => false,
    }
}
