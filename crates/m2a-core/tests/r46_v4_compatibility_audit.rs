use std::{fs, path::Path};

use m2a_core::{
    creature_animation_mapping::{
        AnimationMappingProvenanceV1, AnimationOwnershipV1, AnimationProviderV1,
        AnimationSourceAssignmentV1, AnimationSourceKindV1,
        CREATURE_ANIMATION_AUTHORING_PROFILE_V1, CreatureAnimationAuthoringV1,
        CreatureAnimationMappingStatusV1, DirectCreatureBaseSlotV1, DirectCreatureModelTypeV1,
        validate_creature_animation_authoring_v1,
    },
    direct_creature_animation::FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    model_pipeline::{
        ProceduralCreaturePackageIdentityV1, build_meshy_h1_model_package_v4_with_identity,
    },
    proof_module::BinaryCreatureModuleIdentityV1,
};
use sha2::{Digest, Sha256};

const R46_ROOT: &str =
    r"C:\Projects\meshy2aurora\proof-output\h2-r46-root-first-full-appearance-20260727";
const R45_ROOT: &str =
    r"C:\Projects\meshy2aurora\proof-output\h2-r45-skin-bind-controllers-20260726";

#[test]
#[ignore = "requires the immutable local r45/r46 proof lineages"]
fn exact_r46_runtime_artifacts_are_authored_v4_compatible() {
    let source = fs::read(Path::new(R46_ROOT).join("generated/source.glb")).expect("r46 source");
    let appearance =
        fs::read(Path::new(R45_ROOT).join("generated/appearance.2da")).expect("r45 appearance");
    let source_revision = hex_sha256(&source);
    let authoring = CreatureAnimationAuthoringV1 {
        schema_version: 1,
        profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1.to_owned(),
        model_type: DirectCreatureModelTypeV1::Simple,
        source_revision,
        authoring_revision: 1,
        assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1
            .iter()
            .map(|slot| AnimationSourceAssignmentV1 {
                target_slot: DirectCreatureBaseSlotV1::try_from(*slot).expect("Base 42 slot"),
                source_kind: AnimationSourceKindV1::Procedural,
                source_clip_name: None,
                custom_animation_id: None,
                provenance: AnimationMappingProvenanceV1 {
                    provider: AnimationProviderV1::ProceduralGenerator,
                    asset_id: "m2a:procedural-humanoid-v1".to_owned(),
                    ownership: AnimationOwnershipV1::ProjectGenerated,
                },
            })
            .collect(),
        fallbacks: Vec::new(),
        custom_animations: Vec::new(),
    };
    let validation = validate_creature_animation_authoring_v1(&authoring);
    assert_eq!(validation.status, CreatureAnimationMappingStatusV1::Ready);

    let artifact = build_meshy_h1_model_package_v4_with_identity(
        &source,
        &appearance,
        &authoring,
        &ProceduralCreaturePackageIdentityV1 {
            model_resref: "m2a_h2p46".to_owned(),
            texture_resref: "m2a_h2t46".to_owned(),
            module: BinaryCreatureModuleIdentityV1 {
                module_resref: "m2a_h2r46".to_owned(),
                area_resref: "m2a_h2a46".to_owned(),
                hak_resref: "m2a_h2r46".to_owned(),
            },
            creature_resref: "m2a_h2utc46".to_owned(),
        },
    )
    .expect("exact r46 authored V4 replay");

    let expected = Path::new(R46_ROOT).join("generated");
    assert_eq!(
        artifact.model,
        fs::read(expected.join("m2a_h2p46.mdl")).expect("r46 model")
    );
    assert_eq!(
        artifact.texture,
        fs::read(expected.join("m2a_h2t46.tga")).expect("r46 texture")
    );
    assert_eq!(
        artifact.appearance_two_da,
        fs::read(expected.join("appearance.2da")).expect("r46 appearance")
    );
    assert_eq!(
        artifact.hak,
        fs::read(expected.join("m2a_h2r46.hak")).expect("r46 HAK")
    );
    assert_eq!(
        artifact.proof_module,
        fs::read(expected.join("m2a_h2r46.mod")).expect("r46 MOD")
    );
    println!(
        "r46 authored V4 compatibility: fingerprint={} model={} hak={} module={}",
        validation.authoring_fingerprint_sha256,
        hex_sha256(&artifact.model),
        hex_sha256(&artifact.hak),
        hex_sha256(&artifact.proof_module),
    );
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
