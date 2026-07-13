use std::panic::{AssertUnwindSafe, catch_unwind};

use m2a_core::hak::{HakResourceInputV1, HakWriterOptionsV1, RESREF_INVALID, write_hak_v1};
use m2a_core::package::{
    PACKAGE_RESOURCE_INVALID, PACKAGE_ROLE_DUPLICATE, PACKAGE_ROLE_MISSING, PackageManifestV1,
    PackageResourceRoleV1, write_package_manifest_v1,
};

fn resource(resref: &str, resource_type: u16, payload: &[u8]) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload: payload.to_vec(),
    }
}

fn profile_resources() -> Vec<HakResourceInputV1> {
    vec![
        resource("texture", 3, b"tga"),
        resource("appearance", 2017, b"2da"),
        resource("model", 2002, b"mdl"),
    ]
}

#[test]
fn happy_manifest_matches_hak_report_and_has_frozen_compact_json() {
    let resources = profile_resources();
    let before = resources.clone();
    let options = HakWriterOptionsV1::default();
    let manifest = write_package_manifest_v1(&resources, &options).unwrap();
    let hak = write_hak_v1(&resources, &options).unwrap();
    assert_eq!(resources, before);
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.package_sha256, hak.report.archive_sha256);
    assert_eq!(manifest.resources.len(), 3);
    assert_eq!(
        manifest
            .resources
            .iter()
            .map(|resource| (resource.resref.as_str(), resource.role))
            .collect::<Vec<_>>(),
        [
            ("appearance", PackageResourceRoleV1::AppearanceTable),
            ("model", PackageResourceRoleV1::Model),
            ("texture", PackageResourceRoleV1::Texture),
        ]
    );
    for (manifest_resource, report) in manifest.resources.iter().zip(&hak.report.resources) {
        assert_eq!(manifest_resource.resref, report.resref);
        assert_eq!(manifest_resource.resource_type, report.resource_type);
        assert_eq!(
            manifest_resource.byte_length,
            u64::from(report.payload_size)
        );
        assert_eq!(manifest_resource.sha256, report.payload_sha256);
        assert_eq!(manifest_resource.hak_resource_id, report.resource_id);
        assert_eq!(manifest_resource.hak_payload_offset, report.payload_offset);
    }
    assert_eq!(
        serde_json::to_string(&manifest).unwrap(),
        "{\"schemaVersion\":1,\"packageSha256\":\"494862f6a12f91d5a269519d0579a05ace5bb50fd8f72b5711fcae7445444477\",\"resources\":[{\"role\":\"APPEARANCE_TABLE\",\"resref\":\"appearance\",\"type\":2017,\"byteLength\":3,\"sha256\":\"ddf81e9e4f364c6f086fd730b8f6d2bc4b46068045a085e1be8fc7470a615c6f\",\"hakResourceId\":0,\"hakPayloadOffset\":256},{\"role\":\"MODEL\",\"resref\":\"model\",\"type\":2002,\"byteLength\":3,\"sha256\":\"d3c3c54797643905c5cc97f7da4717058dbe6ad183ef1586104cadd197ca47c6\",\"hakResourceId\":1,\"hakPayloadOffset\":259},{\"role\":\"TEXTURE\",\"resref\":\"texture\",\"type\":3,\"byteLength\":3,\"sha256\":\"9dedca90fc9c44caeb39e0a6b8d28a157105bfba113872846ce0b2f5eff923d3\",\"hakResourceId\":2,\"hakPayloadOffset\":262}]}"
    );
}

#[test]
fn shuffled_profile_inputs_produce_identical_manifest() {
    let items = profile_resources();
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let expected = write_package_manifest_v1(&items, &HakWriterOptionsV1::default()).unwrap();
    for permutation in permutations {
        let input = permutation.map(|index| items[index].clone());
        assert_eq!(
            write_package_manifest_v1(&input, &HakWriterOptionsV1::default()).unwrap(),
            expected
        );
    }
}

#[test]
fn missing_duplicate_wrong_appearance_and_extra_profiles_have_stable_errors_without_panics() {
    let cases = vec![
        (
            "missing",
            vec![
                resource("appearance", 2017, b"2da"),
                resource("model", 2002, b"mdl"),
            ],
            PACKAGE_ROLE_MISSING,
            "resources",
        ),
        (
            "duplicate model",
            vec![
                resource("appearance", 2017, b"2da"),
                resource("model_a", 2002, b"a"),
                resource("model_b", 2002, b"b"),
            ],
            PACKAGE_ROLE_DUPLICATE,
            "resources[2]",
        ),
        (
            "wrong type",
            vec![
                resource("appearance", 2017, b"2da"),
                resource("model", 2002, b"mdl"),
                resource("texture", 4, b"bad"),
            ],
            PACKAGE_RESOURCE_INVALID,
            "resources[2].resourceType",
        ),
        (
            "appearance resref mismatch",
            vec![
                resource("appearance_alt", 2017, b"2da"),
                resource("model", 2002, b"mdl"),
                resource("texture", 3, b"tga"),
            ],
            PACKAGE_RESOURCE_INVALID,
            "resources[0].resref",
        ),
        (
            "appearance type mismatch",
            vec![
                resource("appearance", 4, b"2da"),
                resource("model", 2002, b"mdl"),
                resource("texture", 3, b"tga"),
            ],
            PACKAGE_RESOURCE_INVALID,
            "resources[0].resourceType",
        ),
        (
            "extra",
            vec![
                resource("appearance", 2017, b"2da"),
                resource("model", 2002, b"mdl"),
                resource("texture", 3, b"tga"),
                resource("zzz", 4, b"extra"),
            ],
            PACKAGE_RESOURCE_INVALID,
            "resources[3].resourceType",
        ),
    ];
    for (label, resources, code, path) in cases {
        let result = catch_unwind(AssertUnwindSafe(|| {
            write_package_manifest_v1(&resources, &HakWriterOptionsV1::default())
        }));
        let error = result
            .expect("invalid package profile must not panic")
            .unwrap_err();
        assert_eq!(error.code, code, "{label}");
        assert_eq!(error.path, path, "{label}");
    }
}

#[test]
fn manifest_is_strict_and_hak_validation_precedes_package_profile_validation() {
    let manifest =
        write_package_manifest_v1(&profile_resources(), &HakWriterOptionsV1::default()).unwrap();
    let json = serde_json::to_value(&manifest).unwrap();
    assert_eq!(json["resources"][0]["role"], "APPEARANCE_TABLE");
    assert_eq!(json["resources"][1]["role"], "MODEL");
    assert_eq!(json["resources"][2]["role"], "TEXTURE");
    assert_eq!(json["resources"][0]["type"], 2017);
    assert!(json["resources"][0].get("resourceType").is_none());
    assert!(serde_json::from_value::<PackageManifestV1>(json.clone()).is_ok());

    let mut top_unknown = json.clone();
    top_unknown["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<PackageManifestV1>(top_unknown).is_err());
    let mut resource_unknown = json;
    resource_unknown["resources"][0]["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<PackageManifestV1>(resource_unknown).is_err());

    let error = write_package_manifest_v1(
        &[resource("BAD", 2002, b"mdl")],
        &HakWriterOptionsV1::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, RESREF_INVALID);
    assert_eq!(error.path, "resources[0].resref");
}
