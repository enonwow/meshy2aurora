use m2a_core::{
    aurora_material::{
        AuroraAlphaModeV1, AuroraMaterialChannelV1, AuroraMaterialCompileStatusV1,
        AuroraMaterialDispositionV1, AuroraMaterialTargetProfileV1, AuroraMtrBlendingV1,
        AuroraRenderHintV1, compile_gltf_material_v1, compile_gltf_materials_v1,
    },
    glb::{IrMaterial, IrTextureBinding},
};

fn texture(texture_id: u32, tex_coord_set: u32) -> IrTextureBinding {
    IrTextureBinding {
        texture_id,
        tex_coord_set,
    }
}

fn simple_material() -> IrMaterial {
    IrMaterial {
        id: 7,
        name: Some("oak_hull".to_owned()),
        base_color_factor: [0.8, 0.7, 0.6, 1.0],
        base_color_texture: Some(texture(10, 0)),
        metallic_factor: 0.0,
        roughness_factor: 1.0,
        metallic_roughness_texture: None,
        normal_texture: None,
        emissive_factor: [0.0; 3],
        emissive_texture: None,
        alpha_mode: "OPAQUE".to_owned(),
        alpha_cutoff: None,
        double_sided: false,
    }
}

#[test]
fn classic_safe_compiles_an_opaque_diffuse_material_without_mtr() {
    let compiled = compile_gltf_material_v1(
        &simple_material(),
        AuroraMaterialTargetProfileV1::AuroraClassicSafe,
    )
    .unwrap();

    assert_eq!(compiled.status, AuroraMaterialCompileStatusV1::Ready);
    assert_eq!(compiled.material.source_material_id, 7);
    assert_eq!(compiled.material.diffuse_color, [0.8, 0.7, 0.6, 1.0]);
    assert_eq!(
        compiled
            .material
            .diffuse_texture
            .as_ref()
            .map(|binding| (binding.source_texture_id, binding.tex_coord_set)),
        Some((10, 0))
    );
    assert_eq!(compiled.material.alpha_mode, AuroraAlphaModeV1::Opaque);
    assert_eq!(compiled.material.render_hint, AuroraRenderHintV1::Normal);
    assert!(!compiled.material.mtr_required);
    assert!(compiled.material.normal_texture.is_none());
    assert!(compiled.material.specular_texture_plan.is_none());
    assert!(compiled.fidelity.entries.iter().all(|entry| {
        !matches!(
            entry.disposition,
            AuroraMaterialDispositionV1::DroppedWithWarning | AuroraMaterialDispositionV1::Blocked
        )
    }));
}

#[test]
fn classic_safe_reports_every_lossy_or_unsupported_pbr_channel() {
    let mut material = simple_material();
    material.metallic_factor = 0.65;
    material.roughness_factor = 0.25;
    material.metallic_roughness_texture = Some(texture(11, 0));
    material.normal_texture = Some(texture(12, 0));
    material.emissive_factor = [0.2, 0.1, 0.05];
    material.emissive_texture = Some(texture(13, 0));
    material.alpha_mode = "MASK".to_owned();
    material.alpha_cutoff = Some(0.4);
    material.double_sided = true;

    let compiled =
        compile_gltf_material_v1(&material, AuroraMaterialTargetProfileV1::AuroraClassicSafe)
            .unwrap();

    assert_eq!(compiled.status, AuroraMaterialCompileStatusV1::Blocked);
    for channel in [
        AuroraMaterialChannelV1::BaseColorFactor,
        AuroraMaterialChannelV1::BaseColorTexture,
        AuroraMaterialChannelV1::MetallicFactor,
        AuroraMaterialChannelV1::RoughnessFactor,
        AuroraMaterialChannelV1::MetallicRoughnessTexture,
        AuroraMaterialChannelV1::NormalTexture,
        AuroraMaterialChannelV1::EmissiveFactor,
        AuroraMaterialChannelV1::EmissiveTexture,
        AuroraMaterialChannelV1::AlphaMode,
        AuroraMaterialChannelV1::AlphaCutoff,
        AuroraMaterialChannelV1::DoubleSided,
    ] {
        assert_eq!(
            compiled
                .fidelity
                .entries
                .iter()
                .filter(|entry| entry.channel == channel)
                .count(),
            1,
            "channel {channel:?} must have exactly one fidelity decision"
        );
    }
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::MetallicFactor),
        Some(AuroraMaterialDispositionV1::Baked)
    );
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::NormalTexture),
        Some(AuroraMaterialDispositionV1::DroppedWithWarning)
    );
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::AlphaMode),
        Some(AuroraMaterialDispositionV1::Blocked)
    );
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::DoubleSided),
        Some(AuroraMaterialDispositionV1::Blocked)
    );
}

#[test]
fn ee_mtr_maps_normal_specular_alpha_and_two_sided_material_state() {
    let mut material = simple_material();
    material.metallic_factor = 0.6;
    material.roughness_factor = 0.3;
    material.metallic_roughness_texture = Some(texture(11, 0));
    material.normal_texture = Some(texture(12, 1));
    material.alpha_mode = "MASK".to_owned();
    material.alpha_cutoff = Some(0.5);
    material.double_sided = true;

    let compiled =
        compile_gltf_material_v1(&material, AuroraMaterialTargetProfileV1::NwnEeMtr).unwrap();

    assert_eq!(
        compiled.status,
        AuroraMaterialCompileStatusV1::ReadyWithWarnings
    );
    assert!(compiled.material.mtr_required);
    assert!(compiled.material.two_sided);
    assert_eq!(compiled.material.alpha_mode, AuroraAlphaModeV1::Mask);
    assert_eq!(
        compiled.material.mtr.as_ref().unwrap().blending,
        Some(AuroraMtrBlendingV1::Punchthrough)
    );
    assert_eq!(
        compiled.material.render_hint,
        AuroraRenderHintV1::NormalAndSpecMapped
    );
    assert_eq!(
        compiled
            .material
            .normal_texture
            .as_ref()
            .map(|binding| (binding.source_texture_id, binding.tex_coord_set)),
        Some((12, 1))
    );
    let specular = compiled.material.specular_texture_plan.as_ref().unwrap();
    assert_eq!(specular.source_texture_ids, vec![11]);
    assert_eq!(specular.tex_coord_set, 0);
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::NormalTexture),
        Some(AuroraMaterialDispositionV1::Preserved)
    );
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::DoubleSided),
        Some(AuroraMaterialDispositionV1::Preserved)
    );
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::AlphaCutoff),
        Some(AuroraMaterialDispositionV1::DroppedWithWarning)
    );
}

#[test]
fn ee_blend_uses_transparency_without_inventing_a_blending_token() {
    let mut material = simple_material();
    material.alpha_mode = "BLEND".to_owned();

    let compiled =
        compile_gltf_material_v1(&material, AuroraMaterialTargetProfileV1::NwnEeMtr).unwrap();
    let mtr = compiled.material.mtr.unwrap();
    assert!(mtr.transparency);
    assert_eq!(mtr.blending, None);
}

#[test]
fn unsupported_uv_sets_and_unknown_alpha_modes_fail_closed_in_the_report_or_parser() {
    let mut material = simple_material();
    material.base_color_texture = Some(texture(10, 4));
    let compiled =
        compile_gltf_material_v1(&material, AuroraMaterialTargetProfileV1::NwnEeMtr).unwrap();
    assert_eq!(compiled.status, AuroraMaterialCompileStatusV1::Blocked);
    assert_eq!(
        compiled
            .fidelity
            .disposition(AuroraMaterialChannelV1::BaseColorTexture),
        Some(AuroraMaterialDispositionV1::Blocked)
    );

    material.alpha_mode = "MAGIC".to_owned();
    let error =
        compile_gltf_material_v1(&material, AuroraMaterialTargetProfileV1::AuroraClassicSafe)
            .unwrap_err();
    assert_eq!(error.code, "AURORA-MATERIAL-ALPHA-MODE-UNSUPPORTED");
    assert_eq!(error.path, "material.alphaMode");
}

#[test]
fn complete_material_table_is_source_bound_and_aggregates_blocking_status() {
    let ready = simple_material();
    let mut blocked = simple_material();
    blocked.id = 8;
    blocked.name = Some("canvas".to_owned());
    blocked.double_sided = true;

    let compiled = compile_gltf_materials_v1(
        &"a".repeat(64),
        &[ready, blocked],
        AuroraMaterialTargetProfileV1::AuroraClassicSafe,
    )
    .unwrap();
    assert_eq!(compiled.source_sha256, "a".repeat(64));
    assert_eq!(compiled.materials.len(), 2);
    assert_eq!(compiled.status, AuroraMaterialCompileStatusV1::Blocked);

    let error = compile_gltf_materials_v1(
        "NOT-A-HASH",
        &[],
        AuroraMaterialTargetProfileV1::AuroraClassicSafe,
    )
    .unwrap_err();
    assert_eq!(error.code, "AURORA-MATERIAL-SOURCE-SHA256-INVALID");
}
