use m2a_core::{
    aurora_material::{AuroraMaterialTargetProfileV1, compile_gltf_material_v1},
    glb::{IrMaterial, IrTextureBinding},
    mtr::{
        MTR_RESOURCE_TYPE_V1, MtrMaterialResourceNamesV1, compile_mtr_document_v1, parse_mtr_v1,
        write_mtr_v1,
    },
    txi::{TXI_RESOURCE_TYPE_V1, TxiDocumentV1, parse_txi_v1, write_txi_v1},
};

fn material() -> IrMaterial {
    IrMaterial {
        id: 3,
        name: Some("hull".to_owned()),
        base_color_factor: [1.0; 4],
        base_color_texture: Some(IrTextureBinding {
            texture_id: 10,
            tex_coord_set: 0,
        }),
        metallic_factor: 0.2,
        roughness_factor: 0.7,
        metallic_roughness_texture: Some(IrTextureBinding {
            texture_id: 11,
            tex_coord_set: 0,
        }),
        normal_texture: Some(IrTextureBinding {
            texture_id: 12,
            tex_coord_set: 0,
        }),
        emissive_factor: [0.0; 3],
        emissive_texture: None,
        alpha_mode: "MASK".to_owned(),
        alpha_cutoff: Some(0.5),
        double_sided: true,
    }
}

#[test]
fn compiler_output_becomes_exact_canonical_mtr_and_round_trips() {
    assert_eq!(MTR_RESOURCE_TYPE_V1, 2072);
    let compiled =
        compile_gltf_material_v1(&material(), AuroraMaterialTargetProfileV1::NwnEeMtr).unwrap();
    let document = compile_mtr_document_v1(
        &compiled.material,
        &MtrMaterialResourceNamesV1 {
            diffuse: Some("m2a_hull_d".to_owned()),
            normal: Some("m2a_hull_n".to_owned()),
            specular: Some("m2a_hull_s".to_owned()),
        },
    )
    .unwrap();
    let bytes = write_mtr_v1(&document).unwrap();
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        "texture0 m2a_hull_d\ntexture1 m2a_hull_n\ntexture2 m2a_hull_s\nrenderhint normalandspecmapped\ntransparency 0\ntwosided 1\nblending punchthrough\n"
    );
    assert_eq!(parse_mtr_v1(&bytes).unwrap(), document);
}

#[test]
fn mtr_parser_rejects_unknown_unsafe_or_noncanonical_input() {
    let unsafe_blend = b"renderhint normal\ntransparency 0\ntwosided 0\nblending additive\n";
    assert_eq!(
        parse_mtr_v1(unsafe_blend).unwrap_err().code,
        "MTR-BLENDING-UNSAFE"
    );
    let reordered = b"transparency 0\nrenderhint normal\ntwosided 0\n";
    assert_eq!(
        parse_mtr_v1(reordered).unwrap_err().code,
        "MTR-TEXT-NON-CANONICAL"
    );
    let invalid_resref = b"texture0 HULL-D\nrenderhint normal\ntransparency 0\ntwosided 0\n";
    assert_eq!(
        parse_mtr_v1(invalid_resref).unwrap_err().code,
        "MTR-TEXTURE-RESREF-INVALID"
    );
}

#[test]
fn normal_map_txi_is_exact_and_round_trips() {
    assert_eq!(TXI_RESOURCE_TYPE_V1, 2022);
    let document = TxiDocumentV1::normal_map_v1();
    let bytes = write_txi_v1(&document).unwrap();
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        "mipmap 1\nfilter 1\nisbumpmap 1\nclamp 0\nbumpmapscaling 1.000000\n"
    );
    assert_eq!(parse_txi_v1(&bytes).unwrap(), document);
}

#[test]
fn txi_parser_rejects_additive_duplicate_and_noncanonical_float() {
    assert_eq!(
        parse_txi_v1(b"blending additive\n").unwrap_err().code,
        "TXI-BLENDING-UNSAFE"
    );
    assert_eq!(
        parse_txi_v1(b"mipmap 1\nmipmap 1\n").unwrap_err().code,
        "TXI-DIRECTIVE-DUPLICATE"
    );
    assert_eq!(
        parse_txi_v1(b"bumpmapscaling 1\n").unwrap_err().code,
        "TXI-TEXT-NON-CANONICAL"
    );
}
