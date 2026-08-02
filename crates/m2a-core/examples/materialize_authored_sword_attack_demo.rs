use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::{
    animation_studio::{
        AnimationStudioDocumentStatusV1, AnimationStudioDocumentV1, AuthoredAnimationClipInputV1,
        AuthoredAnimationClipStatusV1, CustomAnimationClipReferenceKindV2,
        CustomAnimationClipReferenceV2, CustomAnimationDefinitionV2,
        apply_custom_attack_demo_route_v1, clone_source_clip_for_editing_v1,
        create_humanoid_sword_slash_from_source_clip_v1,
        migrate_creature_animation_authoring_v1_to_v2,
    },
    creature_animation_mapping::{
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CreatureAnimationAuthoringV1,
        CustomAnimationPlaybackV1, DirectCreatureBaseSlotV1, DirectCreatureModelTypeV1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    inspect_binary_mdl,
    model_pipeline::{
        ProceduralCreaturePackageIdentityV1, build_meshy_h1_model_package_v5_with_identity,
        inspect_editable_animation_source_v1,
        write_animation_studio_v5_demo_packet_with_identity_v1,
    },
    proof_module::{
        BinaryCreatureModuleIdentityV1, inspect_binary_creature_profile_matrix_module_v2,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};

const AUTHORED_CLIP_ID: &str = "vck-authored-void-cleave-v2";
const CUSTOM_ANIMATION_ID: &str = "vck-custom-void-cleave-v2";
const OUTPUT_CLIP_NAME: &str = "m2a_voidcleave";
const SOURCE_POSE_CLIP_NAME: &str = "cpause1";
const SOURCE_POSE_TIME_SECONDS: f32 = 0.54;

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let command = parse_command(env::args().skip(1))?;
    if command.output.exists() {
        return Err(format!(
            "ANIMATION-STUDIO-DEMO-DESTINATION-EXISTS: {}",
            command.output.display()
        ));
    }
    let source = fs::read(&command.source_glb).map_err(|error| {
        format!(
            "ANIMATION-STUDIO-DEMO-SOURCE-READ-FAILED {}: {error}",
            command.source_glb.display()
        )
    })?;
    let appearance = fs::read(&command.appearance_two_da).map_err(|error| {
        format!(
            "ANIMATION-STUDIO-DEMO-APPEARANCE-READ-FAILED {}: {error}",
            command.appearance_two_da.display()
        )
    })?;
    let inspection = inspect_editable_animation_source_v1(&source)
        .map_err(|error| format!("ANIMATION-STUDIO-DEMO-INSPECTION-FAILED: {error}"))?;
    let source_clip = clone_source_clip_for_editing_v1(
        &inspection.animations,
        SOURCE_POSE_CLIP_NAME,
        AuthoredAnimationClipInputV1 {
            id: AUTHORED_CLIP_ID.to_owned(),
            name: OUTPUT_CLIP_NAME.to_owned(),
            source_revision: inspection.source_revision.clone(),
            length_seconds: 1.0,
            transition_seconds: 0.1,
            animation_root: inspection.rig.animation_root.clone(),
        },
    )
    .map_err(|diagnostic| {
        format!(
            "ANIMATION-STUDIO-DEMO-SOURCE-CLONE-FAILED {} {}: {}",
            diagnostic.code, diagnostic.path, diagnostic.message
        )
    })?;
    let mut clip = create_humanoid_sword_slash_from_source_clip_v1(
        &source_clip,
        &inspection.rig,
        SOURCE_POSE_TIME_SECONDS,
    )
    .map_err(|diagnostic| {
        format!(
            "ANIMATION-STUDIO-DEMO-PRESET-FAILED {} {}: {}",
            diagnostic.code, diagnostic.path, diagnostic.message
        )
    })?;
    clip.status = AuthoredAnimationClipStatusV1::Valid;
    let authored_keyframe_count = clip
        .tracks
        .iter()
        .map(|track| track.keyframes.len())
        .sum::<usize>();
    let source_clip_fingerprint = clip.source.source_clip_fingerprint.clone();
    let studio = AnimationStudioDocumentV1 {
        schema_version: 1,
        source_revision: inspection.source_revision.clone(),
        authoring_revision: 1,
        status: AnimationStudioDocumentStatusV1::Valid,
        authored_clips: vec![clip],
    };
    let provenance = AnimationMappingProvenanceV1 {
        provider: AnimationProviderV1::ProceduralGenerator,
        asset_id: "m2a:procedural-humanoid-v2".to_owned(),
        ownership: AnimationOwnershipV1::ProjectGenerated,
    };
    let base = CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision: inspection.source_revision.clone(),
        authoring_revision: 1,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot)
                    .expect("canonical native slot"),
                source_kind: AnimationSourceKindV1::Procedural,
                source_clip_name: None,
                custom_animation_id: None,
                provenance: provenance.clone(),
            })
            .collect(),
        fallbacks: Vec::new(),
        custom_animations: Vec::new(),
    };
    let mut authoring = migrate_creature_animation_authoring_v1_to_v2(&base);
    authoring
        .custom_animations
        .push(CustomAnimationDefinitionV2 {
            id: CUSTOM_ANIMATION_ID.to_owned(),
            name: OUTPUT_CLIP_NAME.to_owned(),
            playback: CustomAnimationPlaybackV1::OneShot,
            clip_reference: Some(CustomAnimationClipReferenceV2 {
                source_kind: CustomAnimationClipReferenceKindV2::AuthoredClip,
                source_clip_name: None,
                authored_clip_id: Some(AUTHORED_CLIP_ID.to_owned()),
            }),
            phases: Vec::new(),
            provenance: AnimationMappingProvenanceV1 {
                provider: AnimationProviderV1::UserCustom,
                asset_id: AUTHORED_CLIP_ID.to_owned(),
                ownership: AnimationOwnershipV1::UserOwned,
            },
        });
    let routed = apply_custom_attack_demo_route_v1(&authoring, CUSTOM_ANIMATION_ID).map_err(
        |diagnostic| {
            format!(
                "ANIMATION-STUDIO-DEMO-ROUTE-FAILED {} {}: {}",
                diagnostic.code, diagnostic.path, diagnostic.message
            )
        },
    )?;
    let identity = ProceduralCreaturePackageIdentityV1 {
        model_resref: command.resref.clone(),
        texture_resref: command.resref.clone(),
        module: BinaryCreatureModuleIdentityV1 {
            module_resref: command.resref.clone(),
            area_resref: command.resref.clone(),
            hak_resref: command.resref.clone(),
        },
        creature_resref: command.resref.clone(),
    };
    let artifact = build_meshy_h1_model_package_v5_with_identity(
        &source,
        &appearance,
        &routed.authoring,
        &studio,
        None,
        &identity,
    )
    .map_err(|error| error.to_string())?;
    write_animation_studio_v5_demo_packet_with_identity_v1(&command.output, &artifact, &identity)
        .map_err(|error| error.to_string())?;

    let model = inspect_binary_mdl(&artifact.model).map_err(|error| error.to_string())?;
    let module = inspect_binary_creature_profile_matrix_module_v2(&artifact.proof_module)
        .map_err(|error| error.to_string())?;
    let attack_clips = ["ca1slashl", "ca1slashr", "ca1stab", OUTPUT_CLIP_NAME]
        .iter()
        .map(|name| {
            let clip = artifact
                .report
                .base
                .animation_behavior
                .as_ref()
                .and_then(|behavior| behavior.clips.iter().find(|clip| clip.name == *name))
                .ok_or_else(|| format!("missing attack behavior readback for {name}"))?;
            Ok(json!({
                "name": name,
                "changingControllerCount": clip.changing_controller_count,
                "motionSha256": clip.motion_sha256,
                "events": model.animations.iter()
                    .find(|candidate| candidate.name == *name)
                    .map(|candidate| candidate.events.iter()
                        .map(|event| event.name.as_str())
                        .collect::<Vec<_>>())
                    .unwrap_or_default()
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    serde_json::to_string_pretty(&json!({
        "ok": true,
        "status": "READY_FOR_OWNER_PROOF",
        "pipeline": "ANIMATION_STUDIO_V5_M2A_VOIDCLEAVE_DEMO_V2",
        "outputDirectory": command.output,
        "moduleFile": format!("{}.mod", identity.module.module_resref),
        "hakFile": format!("{}.hak", identity.module.hak_resref),
        "moduleResref": module.scene.module_resref,
        "areaResref": module.scene.area_resref,
        "creatureResref": module.scene.fixtures[0].template_resref,
        "appearanceRow": artifact.report.base.appearance.appended_row_index,
        "runtimeProfile": module.fixtures[0].runtime_profile,
        "productionIdlePreserved": routed.contract.production_idle_slot_preserved,
        "allNativeAttackVariantsRouted": routed.contract.all_native_attack_variants_routed,
        "authoredClip": {
            "id": AUTHORED_CLIP_ID,
            "name": OUTPUT_CLIP_NAME,
            "playback": "ONE_SHOT",
            "sourceKind": "SOURCE_CLIP_COPY",
            "sourceClipName": SOURCE_POSE_CLIP_NAME,
            "sourceClipFingerprint": source_clip_fingerprint,
            "poseTimeSeconds": SOURCE_POSE_TIME_SECONDS,
            "durationSeconds": 1.0,
            "transitionSeconds": 0.1,
            "keyframeCount": authored_keyframe_count,
            "phaseTimesSeconds": [0.0, 0.18, 0.34, 0.5, 0.66, 0.82, 1.0]
        },
        "attackClips": attack_clips,
        "source": binding(&source),
        "model": binding(&artifact.model),
        "texture": binding(&artifact.texture),
        "appearanceTwoDa": binding(&artifact.appearance_two_da),
        "hak": binding(&artifact.hak),
        "module": binding(&artifact.proof_module),
        "triangleCount": artifact.report.base.geometry.triangle_count,
        "skinAccessoryStabilization": artifact.report.base.skin_accessory_stabilization,
        "animationStudioReadback": artifact.animation_studio_readback.status,
        "startsToolset": false,
        "startsNwn": false
    }))
    .map_err(|error| format!("ANIMATION-STUDIO-DEMO-REPORT-FAILED: {error}"))
}

fn binding(bytes: &[u8]) -> serde_json::Value {
    json!({
        "byteLength": bytes.len(),
        "sha256": format!("{:x}", Sha256::digest(bytes))
    })
}

struct Command {
    source_glb: PathBuf,
    appearance_two_da: PathBuf,
    output: PathBuf,
    resref: String,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut source_glb = None;
    let mut appearance_two_da = None;
    let mut output = None;
    let mut resref = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source_glb,
            "--appearance-2da" => &mut appearance_two_da,
            "--out" => &mut output,
            "--resref" => &mut resref,
            "--help" | "-h" => return Err(usage()),
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        };
        if target.is_some() {
            return Err(format!("duplicate argument: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("missing value for {argument}"));
        }
    }
    let resref = resref.ok_or_else(usage)?;
    if resref.is_empty()
        || resref.len() > 16
        || !resref
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(
            "--resref must contain 1..16 lowercase ASCII letters, digits or '_'".to_owned(),
        );
    }
    Ok(Command {
        source_glb: PathBuf::from(source_glb.ok_or_else(usage)?),
        appearance_two_da: PathBuf::from(appearance_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
        resref,
    })
}

fn usage() -> String {
    "usage: materialize_authored_sword_attack_demo --source-glb <path> \
     --appearance-2da <path> --out <new-output-dir> --resref <fresh-resref>"
        .to_owned()
}
