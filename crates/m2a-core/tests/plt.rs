use m2a_core::plt::{
    PLT_MAX_OUTPUT_BYTES, PltImageV1, PltPixelV1, PltWriterLimitsV1, PltWriterOptionsV1,
    read_plt_image_v1, write_plt_v1,
};

fn image() -> PltImageV1 {
    PltImageV1 {
        schema_version: 1,
        width: 2,
        height: 2,
        num_layers: 5,
        pixels: vec![
            PltPixelV1 {
                color_index: 10,
                layer_index: 2,
            },
            PltPixelV1 {
                color_index: 20,
                layer_index: 4,
            },
            PltPixelV1 {
                color_index: 255,
                layer_index: 0,
            },
            PltPixelV1 {
                color_index: 30,
                layer_index: 3,
            },
        ],
    }
}

#[test]
fn exact_plt_v1_is_bottom_up_with_locked_header_and_roundtrip() {
    let source = image();
    let artifact = write_plt_v1(&source, &PltWriterOptionsV1::default()).unwrap();
    assert_eq!(&artifact.payload[..8], b"PLT V1  ");
    assert_eq!(
        &artifact.payload[8..24],
        &[5, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0]
    );
    assert_eq!(
        &artifact.payload[24..],
        &[255, 0, 30, 3, 10, 2, 20, 4],
        "PLT pixels are stored bottom row first as (color, layer) pairs"
    );
    assert_eq!(read_plt_image_v1(&artifact.payload).unwrap(), source);
    assert_eq!(artifact.report.used_layers, vec![0, 2, 3, 4]);
    assert_eq!(artifact.report.byte_length, 32);
}

#[test]
fn writer_is_deterministic_and_rejects_invalid_shape_layers_and_trailing_bytes() {
    let source = image();
    assert_eq!(
        write_plt_v1(&source, &PltWriterOptionsV1::default()).unwrap(),
        write_plt_v1(&source, &PltWriterOptionsV1::default()).unwrap()
    );

    let mut invalid = source.clone();
    invalid.pixels.pop();
    assert_eq!(
        write_plt_v1(&invalid, &PltWriterOptionsV1::default())
            .unwrap_err()
            .code,
        "M2A-PLT-PIXEL-LENGTH-INVALID"
    );

    let mut invalid = source.clone();
    invalid.pixels[0].layer_index = invalid.num_layers as u8;
    assert_eq!(
        write_plt_v1(&invalid, &PltWriterOptionsV1::default())
            .unwrap_err()
            .code,
        "M2A-PLT-LAYER-INVALID"
    );

    let mut payload = write_plt_v1(&source, &PltWriterOptionsV1::default())
        .unwrap()
        .payload;
    payload.push(0);
    assert_eq!(
        read_plt_image_v1(&payload).unwrap_err().code,
        "M2A-PLT-LENGTH-INVALID"
    );
}

#[test]
fn exact_output_limit_is_inclusive() {
    let source = image();
    let options = PltWriterOptionsV1 {
        schema_version: 1,
        limits: PltWriterLimitsV1 {
            max_output_bytes: 32,
        },
    };
    assert_eq!(write_plt_v1(&source, &options).unwrap().payload.len(), 32);
    let options = PltWriterOptionsV1 {
        schema_version: 1,
        limits: PltWriterLimitsV1 {
            max_output_bytes: 31,
        },
    };
    assert_eq!(
        write_plt_v1(&source, &options).unwrap_err().code,
        "M2A-PLT-OUTPUT-LIMIT-EXCEEDED"
    );
    assert!(PLT_MAX_OUTPUT_BYTES >= 32);
}
