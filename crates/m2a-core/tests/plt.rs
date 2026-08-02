use m2a_core::{
    plt::{
        ItemPltLayerV1, PLT_HEADER_BYTE_LENGTH_V1, PltWriterOptionsV1, inspect_item_plt_v1,
        write_item_plt_v1,
    },
    tga::{TGA_SCHEMA_VERSION, TgaImageV1, TgaPixelFormatV1},
};

#[test]
fn item_plt_writes_bottom_up_palette_indices_and_roundtrips() {
    let image = TgaImageV1 {
        schema_version: TGA_SCHEMA_VERSION,
        width: 2,
        height: 2,
        pixel_format: TgaPixelFormatV1::Rgb8,
        pixels: vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255],
    };
    let artifact = write_item_plt_v1(
        &image,
        &PltWriterOptionsV1 {
            layer: ItemPltLayerV1::Metal1,
            ..PltWriterOptionsV1::default()
        },
    )
    .unwrap();

    assert_eq!(&artifact.payload[..8], b"PLT V1  ");
    assert_eq!(
        &artifact.payload[PLT_HEADER_BYTE_LENGTH_V1..],
        &[29, 2, 255, 2, 77, 2, 149, 2]
    );
    let readback = inspect_item_plt_v1(&artifact.payload).unwrap();
    assert_eq!(readback.width, 2);
    assert_eq!(readback.height, 2);
    assert_eq!(readback.layer, ItemPltLayerV1::Metal1);
    assert_eq!(readback.color_indices, [77, 149, 29, 255]);
    assert_eq!(artifact.report.semantic_readback_status, "PASS");
}

#[test]
fn item_plt_rejects_transparency_instead_of_silently_dropping_alpha() {
    let image = TgaImageV1 {
        schema_version: TGA_SCHEMA_VERSION,
        width: 1,
        height: 1,
        pixel_format: TgaPixelFormatV1::Rgba8,
        pixels: vec![1, 2, 3, 254],
    };
    let error = write_item_plt_v1(&image, &PltWriterOptionsV1::default()).unwrap_err();
    assert_eq!(error.code, "ITEM-PLT-ALPHA-UNSUPPORTED");
}
