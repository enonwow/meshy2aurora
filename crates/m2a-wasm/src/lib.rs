use wasm_bindgen::prelude::*;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StudioRuntimeCapabilitiesV1<'a> {
    schema_version: u32,
    runtime_contract: &'a str,
    creature_source_forward: &'a str,
    creature_triangle_budget: usize,
    creature_equipment: &'a str,
    creature_motion_pack: &'a str,
    creature_materials: &'a str,
    reference_supermodel_motion: &'a str,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod item_boundary_tests {
    use super::{
        compile_item_part_v1_inner, read_item_uti_v1_json_inner,
        resolve_item_base_record_v1_json_inner, write_item_icon_layers_v1_inner,
        write_item_uti_v1_inner,
    };
    use m2a_core::item::{
        ItemAppearanceRecipeV1, ItemBaseRecordV1, ItemCompositionProfileV1, ItemIdentityV1,
        ItemPartRecipeV1, ItemPartSlotV1, ItemPartTransformV1,
    };
    use m2a_core::item_icon::ItemIconLayerInputV1;
    use m2a_core::item_uti::{ItemUtiBuildRequestV1, ItemUtiPropertiesV1};
    use m2a_core::{
        item_part::ItemPartCompileRequestV1,
        mdl::MdlMaterialTextureBindingV1,
        model_ir::{
            AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1,
            AuroraModelSegmentV1, AuroraSegmentDeformationV1,
        },
    };
    use sha2::{Digest, Sha256};

    const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn request() -> ItemUtiBuildRequestV1 {
        ItemUtiBuildRequestV1 {
            schema_version: 1,
            recipe: ItemAppearanceRecipeV1 {
                schema_version: 1,
                base_item: ItemBaseRecordV1 {
                    schema_version: 1,
                    source_sha256: HASH.to_owned(),
                    physical_row_index: 0,
                    printed_row_label: 0,
                    profile: ItemCompositionProfileV1::ModelType0,
                    model_type: 0,
                    item_class: "AShSw".to_owned(),
                    gender_specific: false,
                    inv_slot_width: 2,
                    inv_slot_height: 2,
                    equipable_slots: "0x00020".to_owned(),
                    default_model: None,
                    default_icon: None,
                },
                identity: ItemIdentityV1 {
                    uti_resref: "m2a_item_wasm".to_owned(),
                    tag: "M2A_ITEM_WASM".to_owned(),
                    display_name: "WASM parity item".to_owned(),
                },
                gender: None,
                parts: vec![ItemPartRecipeV1 {
                    slot: ItemPartSlotV1::Model,
                    variant: 7,
                    source_part_id: "model".to_owned(),
                    source_sha256: HASH.to_owned(),
                    transform: ItemPartTransformV1::default(),
                }],
                colors: None,
            },
            properties: ItemUtiPropertiesV1::default(),
        }
    }

    #[test]
    fn baseitems_boundary_matches_core_json_exactly() {
        let bytes = b"2DA V2.0\n\nItemClass ModelType GenderSpecific InvSlotWidth InvSlotHeight EquipableSlots DefaultModel DefaultIcon\n0 AShSw 0 0 2 2 0x00020 **** ****\n";
        let limits = m2a_core::two_da::TwoDaLimitsV1::default();
        let limits_json = serde_json::to_string(&limits).unwrap();
        let hash = m2a_core::two_da::inspect_two_da_v2(bytes, &limits)
            .unwrap()
            .source_sha256;
        let core = m2a_core::item::resolve_item_base_record_v1(bytes, 0, &hash, &limits).unwrap();
        let boundary =
            resolve_item_base_record_v1_json_inner(bytes, 0, &hash, &limits_json).unwrap();
        assert_eq!(boundary, serde_json::to_string(&core).unwrap());
    }

    #[test]
    fn item_uti_boundary_is_byte_and_readback_identical_to_core() {
        let request = request();
        let options = m2a_core::gff::GffWriterOptionsV1::default();
        let request_json = serde_json::to_string(&request).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let core = m2a_core::item_uti::write_item_uti_v1(&request, &options).unwrap();
        let boundary = write_item_uti_v1_inner(&request_json, &options_json).unwrap();
        assert_eq!(boundary.payload, core.payload);
        assert_eq!(boundary.report, core.report);
        assert_eq!(boundary.readback, core.readback);
        let limits_json = serde_json::to_string(&options.limits).unwrap();
        assert_eq!(
            read_item_uti_v1_json_inner(&boundary.payload, 0, &limits_json).unwrap(),
            serde_json::to_string(&core.readback).unwrap()
        );
    }

    #[test]
    fn item_part_boundary_is_byte_and_readback_identical_to_core() {
        let identity = [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ];
        let request = ItemPartCompileRequestV1 {
            schema_version: 1,
            recipe_sha256: HASH.to_owned(),
            part: ItemPartRecipeV1 {
                slot: ItemPartSlotV1::Model,
                variant: 7,
                source_part_id: "model".to_owned(),
                source_sha256: HASH.to_owned(),
                transform: ItemPartTransformV1 {
                    translation: [0.25, 0.5, 0.75],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: 1.5,
                },
            },
            model_resref: "ashsw_007".to_owned(),
            model: AuroraModelIrV1 {
                schema_version: 1,
                profile_id: "wasm-item-part-test".to_owned(),
                source_sha256: HASH.to_owned(),
                basis_status: "TEST".to_owned(),
                engine_facing_proof: "TEST".to_owned(),
                uv_runtime_proof: "TEST".to_owned(),
                nodes: vec![AuroraModelNodeV1 {
                    id: 1,
                    name: "root".to_owned(),
                    parent_id: None,
                    bind_local_matrix: identity,
                }],
                material_source_bindings: vec![AuroraMaterialSourceBindingV1 {
                    slot: 0,
                    source_material_id: None,
                    source_material_name: None,
                }],
                segments: vec![AuroraModelSegmentV1 {
                    segment_id: 1,
                    material_slot: 0,
                    deformation: AuroraSegmentDeformationV1::Rigid,
                    parent_node_id: 1,
                    cast_shadow: true,
                    positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    normals: vec![[0.0, 0.0, 1.0]; 3],
                    tangents: None,
                    uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                    indices: vec![0, 1, 2],
                    face_surface_ids: vec![],
                    weights: vec![],
                }],
            },
            material_textures: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "item_tex".to_owned(),
            }],
        };
        let core = m2a_core::item_part::compile_item_part_v1(&request).unwrap();
        let boundary =
            compile_item_part_v1_inner(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(boundary.payload, core.payload);
        assert_eq!(boundary.report, core.report);
        assert_eq!(boundary.inspection, core.inspection);
    }

    #[test]
    fn item_icon_boundary_is_byte_and_report_identical_to_core() {
        let request = request();
        let recipe = request.recipe;
        let pixels = vec![0x7f; 64 * 64 * 4];
        let layers = vec![ItemIconLayerInputV1 {
            slot: ItemPartSlotV1::Model,
            resref: "iashsw_007".to_owned(),
            source_sha256: format!("{:x}", Sha256::digest(&pixels)),
            image: m2a_core::tga::TgaImageV1 {
                schema_version: 1,
                width: 64,
                height: 64,
                pixel_format: m2a_core::tga::TgaPixelFormatV1::Rgba8,
                pixels,
            },
        }];
        let options = m2a_core::tga::TgaWriterOptionsV1::default();
        let core =
            m2a_core::item_icon::write_item_icon_layers_v1(&recipe, &layers, &options).unwrap();
        let (payload_blob, descriptors_json, reports_json) = write_item_icon_layers_v1_inner(
            &serde_json::to_string(&recipe).unwrap(),
            &serde_json::to_string(&layers).unwrap(),
            &serde_json::to_string(&options).unwrap(),
        )
        .unwrap();
        assert_eq!(payload_blob, core[0].payload);
        assert_eq!(
            reports_json,
            serde_json::to_string(&vec![&core[0].report]).unwrap()
        );
        let descriptors: serde_json::Value = serde_json::from_str(&descriptors_json).unwrap();
        assert_eq!(descriptors[0]["payloadOffset"], 0);
        assert_eq!(
            descriptors[0]["payloadSize"],
            serde_json::json!(core[0].payload.len())
        );
        assert_eq!(descriptors[0]["resref"], "iashsw_007");
    }
}

#[wasm_bindgen(js_name = studioRuntimeCapabilitiesV1Json)]
pub fn studio_runtime_capabilities_v1_json() -> String {
    serde_json::to_string(&StudioRuntimeCapabilitiesV1 {
        schema_version: 1,
        runtime_contract: "M2A_STUDIO_WASM_2026_08_19_V3",
        creature_source_forward: "CARDINAL_XZ_TO_AURORA_POSITIVE_Y_V2",
        creature_triangle_budget: m2a_core::AURORA_MODEL_TRIANGLE_BUDGET_V1,
        creature_equipment: "COMPLETE_EMBEDDED_GIT_UTC_V2",
        creature_motion_pack: "SOURCE_BOUND_HUMANOID_QUADRUPED_V1",
        creature_materials: "ANIMATED_CLASSIC_OR_NWN_EE_MTR_V2",
        reference_supermodel_motion:
            "EXACT_REFERENCE_BIND_AND_WEIGHTED_ANCHORS_V3_WITH_MATERIAL_LEDGER",
    })
    .expect("static Studio runtime capability contract serializes")
}

fn build_exact_reference_supermodel_motion_contract_v3_json_inner(
    reference_mdl: &[u8],
    options_json: &str,
) -> Result<String, String> {
    let options = serde_json::from_str::<
        m2a_core::reference_supermodel_motion::ReferenceSupermodelExactContractOptionsV3,
    >(options_json)
    .map_err(|_| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 3,
            "code": "M2A-WASM-SUPERMODEL-EXACT-OPTIONS-JSON-INVALID",
            "path": "optionsJson",
            "message": "options JSON must satisfy ReferenceSupermodelExactContractOptionsV3"
        }))
    })?;
    let reference = m2a_core::mdl::inspect_binary_mdl(reference_mdl).map_err(|error| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 3,
            "code": "M2A-WASM-SUPERMODEL-REFERENCE-MDL-INVALID",
            "path": "referenceMdl",
            "message": error.to_string()
        }))
    })?;
    let contract =
        m2a_core::reference_supermodel_motion::build_exact_reference_supermodel_motion_contract_v3(
            &reference, &options,
        )
        .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&contract))
}

/// Builds a supermodel-independent inherited-motion contract from an exact,
/// read-only binary MDL inspection. The same boundary is used for every
/// selected Creature supermodel.
#[wasm_bindgen(js_name = buildExactReferenceSupermodelMotionContractV3Json)]
pub fn build_exact_reference_supermodel_motion_contract_v3_json(
    reference_mdl: &[u8],
    options_json: &str,
) -> Result<String, JsValue> {
    build_exact_reference_supermodel_motion_contract_v3_json_inner(reference_mdl, options_json)
        .map_err(|error| JsValue::from_str(&error))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MotionCorrectionBoundaryOutputV1<'a> {
    schema_version: u32,
    compatibility_level:
        m2a_core::reference_supermodel_motion::ReferenceSupermodelCompatibilityLevelV2,
    rig: &'a m2a_core::profile_a::CreatureRigProfileV1,
    report: &'a m2a_core::reference_supermodel_motion::ReferenceSupermodelCorrectionReportV1,
}

fn build_motion_corrected_rig_v1_json_inner(
    target_rig_json: &str,
    motion_contract_json: &str,
) -> Result<String, String> {
    let target = serde_json::from_str::<m2a_core::profile_a::CreatureRigProfileV1>(target_rig_json)
        .map_err(|_| {
            serialize_json(&serde_json::json!({
                "schemaVersion": 2,
                "code": "M2A-WASM-MOTION-TARGET-RIG-JSON-INVALID",
                "path": "targetRigJson",
                "message": "target rig JSON must satisfy the strict CreatureRigProfileV1 schema"
            }))
        })?;
    let contract = serde_json::from_str::<
        m2a_core::reference_supermodel_motion::ReferenceSupermodelMotionContractV2,
    >(motion_contract_json)
    .map_err(|_| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 2,
            "code": "M2A-WASM-MOTION-CONTRACT-JSON-INVALID",
            "path": "motionContractJson",
            "message": "motion contract JSON must satisfy the strict ReferenceSupermodelMotionContractV2 schema"
        }))
    })?;
    let artifact =
        m2a_core::reference_supermodel_motion::build_motion_corrected_rig_v1(&target, &contract)
            .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&MotionCorrectionBoundaryOutputV1 {
        schema_version: 1,
        compatibility_level:
            m2a_core::reference_supermodel_motion::ReferenceSupermodelCompatibilityLevelV2::TopologyOnly,
        rig: &artifact.rig,
        report: &artifact.report,
    }))
}

/// Builds the legacy owned correction layer below a carrier hierarchy. Without
/// an immutable reference MDL this boundary can prove topology only.
#[wasm_bindgen(js_name = buildMotionCorrectedRigV1Json)]
pub fn build_motion_corrected_rig_v1_json(
    target_rig_json: &str,
    motion_contract_json: &str,
) -> Result<String, JsValue> {
    build_motion_corrected_rig_v1_json_inner(target_rig_json, motion_contract_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_exact_motion_carrier_rig_v3_json_inner(
    target_rig_json: &str,
    motion_contract_json: &str,
    reference_mdl: &[u8],
) -> Result<String, String> {
    let target = serde_json::from_str::<m2a_core::profile_a::CreatureRigProfileV1>(target_rig_json)
        .map_err(|_| {
            serialize_json(&serde_json::json!({
                "schemaVersion": 3,
                "code": "M2A-WASM-MOTION-TARGET-RIG-JSON-INVALID",
                "path": "targetRigJson",
                "message": "target rig JSON must satisfy CreatureRigProfileV1"
            }))
        })?;
    let contract = serde_json::from_str::<
        m2a_core::reference_supermodel_motion::ReferenceSupermodelMotionContractV2,
    >(motion_contract_json)
    .map_err(|_| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 3,
            "code": "M2A-WASM-MOTION-CONTRACT-JSON-INVALID",
            "path": "motionContractJson",
            "message": "motion contract JSON must satisfy ReferenceSupermodelMotionContractV2"
        }))
    })?;
    let reference = m2a_core::mdl::inspect_binary_mdl(reference_mdl).map_err(|error| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 3,
            "code": "M2A-WASM-SUPERMODEL-REFERENCE-MDL-INVALID",
            "path": "referenceMdl",
            "message": error.to_string()
        }))
    })?;
    let artifact = m2a_core::reference_supermodel_motion::build_exact_motion_carrier_rig_v3(
        &target, &contract, &reference,
    )
    .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&MotionCorrectionBoundaryOutputV1 {
        schema_version: 3,
        compatibility_level:
            m2a_core::reference_supermodel_motion::ReferenceSupermodelCompatibilityLevelV2::BindPoseCompatible,
        rig: &artifact.rig,
        report: &artifact.report,
    }))
}

/// Rebinds an owned target rig to exact inspected carrier matrices and refuses
/// bind drift before returning `BIND_POSE_COMPATIBLE`.
#[wasm_bindgen(js_name = buildExactMotionCarrierRigV3Json)]
pub fn build_exact_motion_carrier_rig_v3_json(
    target_rig_json: &str,
    motion_contract_json: &str,
    reference_mdl: &[u8],
) -> Result<String, JsValue> {
    build_exact_motion_carrier_rig_v3_json_inner(
        target_rig_json,
        motion_contract_json,
        reference_mdl,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct ReferenceSupermodelAppliedPreviewWasmArtifactV2 {
    model_bytes: Vec<u8>,
    readback_json: String,
    apply_report_json: String,
    authoring_json: String,
    target_rig_json: String,
}

#[wasm_bindgen]
impl ReferenceSupermodelAppliedPreviewWasmArtifactV2 {
    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = applyReportJson)]
    pub fn apply_report_json(&self) -> String {
        self.apply_report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = authoringJson)]
    pub fn authoring_json(&self) -> String {
        self.authoring_json.clone()
    }

    #[wasm_bindgen(getter, js_name = targetRigJson)]
    pub fn target_rig_json(&self) -> String {
        self.target_rig_json.clone()
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferenceSupermodelChainBlobDescriptorV2 {
    resref: String,
    supermodel_resref: String,
    format: m2a_core::reference_supermodel_generic::ReferenceSupermodelFormatV2,
    sha256: String,
    byte_offset: usize,
    byte_length: usize,
}

fn parse_reference_supermodel_chain_blob_v2(
    blob: &[u8],
    descriptors_json: &str,
) -> Result<Vec<m2a_core::reference_supermodel_generic::ReferenceSupermodelChainPayloadV2>, String>
{
    let descriptors =
        serde_json::from_str::<Vec<ReferenceSupermodelChainBlobDescriptorV2>>(descriptors_json)
            .map_err(|source| {
                serialize_json(&serde_json::json!({
                    "schemaVersion": 2,
                    "code": "M2A-REFERENCE-SUPERMODEL-CHAIN-DESCRIPTORS-JSON",
                    "path": "referenceChainJson",
                    "message": source.to_string()
                }))
            })?;
    let mut payloads = Vec::with_capacity(descriptors.len());
    for (index, descriptor) in descriptors.into_iter().enumerate() {
        let end = descriptor
            .byte_offset
            .checked_add(descriptor.byte_length)
            .filter(|end| *end <= blob.len())
            .ok_or_else(|| {
                serialize_json(&serde_json::json!({
                    "schemaVersion": 2,
                    "code": "M2A-REFERENCE-SUPERMODEL-CHAIN-RANGE",
                    "path": format!("referenceChain[{index}]"),
                    "message": "descriptor byte range is outside the exact chain blob"
                }))
            })?;
        payloads.push(
            m2a_core::reference_supermodel_generic::ReferenceSupermodelChainPayloadV2 {
                resource:
                    m2a_core::reference_supermodel_generic::ReferenceSupermodelChainResourceV2 {
                        resref: descriptor.resref,
                        supermodel_resref: descriptor.supermodel_resref,
                        format: descriptor.format,
                        sha256: descriptor.sha256,
                        byte_length: descriptor.byte_length,
                    },
                payload: blob[descriptor.byte_offset..end].to_vec(),
            },
        );
    }
    Ok(payloads)
}

fn parse_reference_source_forward_v2(
    source_forward: &str,
) -> Result<m2a_core::profile_a::CreatureSourceForwardV1, String> {
    match source_forward {
        "POSITIVE_Z" => Ok(m2a_core::profile_a::CreatureSourceForwardV1::PositiveZ),
        "NEGATIVE_Z" => Ok(m2a_core::profile_a::CreatureSourceForwardV1::NegativeZ),
        "POSITIVE_X" => Ok(m2a_core::profile_a::CreatureSourceForwardV1::PositiveX),
        "NEGATIVE_X" => Ok(m2a_core::profile_a::CreatureSourceForwardV1::NegativeX),
        _ => Err(serialize_json(&serde_json::json!({
            "schemaVersion": 2,
            "code": "M2A-REFERENCE-SUPERMODEL-SOURCE-FORWARD-INVALID",
            "path": "sourceForward",
            "message": "sourceForward must be one of the four supported cardinal glTF axes"
        }))),
    }
}

#[derive(Clone, Copy)]
enum ReferenceSupermodelAuthoringInputModeV1<'a> {
    Automatic,
    Draft(&'a str),
    Sealed(&'a str),
}

#[derive(Clone, Copy)]
enum ReferenceSupermodelAuthoringInputModeV2<'a> {
    Automatic,
    Draft(&'a str),
    Sealed(&'a str),
}

struct PreparedReferenceSupermodelRigV1 {
    source_forward: m2a_core::profile_a::CreatureSourceForwardV1,
    source: m2a_core::glb::GlbIngestResult,
    analysis: m2a_core::reference_supermodel_generic::ReferenceSupermodelChainAnalysisArtifactV2,
    base_rig: m2a_core::reference_supermodel_generic::GenericReferenceRigArtifactV2,
    exact_chain_sha256: String,
    authoring: m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringDocumentV1,
    authored: m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringArtifactV1,
    authored_validation: m2a_core::reference_supermodel_generic::GenericReferenceRigAnalysisV2,
}

struct PreparedReferenceSupermodelRigV2 {
    source_forward: m2a_core::profile_a::CreatureSourceForwardV1,
    source: m2a_core::glb::GlbIngestResult,
    analysis: m2a_core::reference_supermodel_generic::ReferenceSupermodelChainAnalysisArtifactV2,
    base_rig: m2a_core::reference_supermodel_generic::GenericReferenceRigArtifactV2,
    exact_chain_sha256: String,
    authoring: m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringDocumentV2,
    authored: m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringArtifactV2,
    authored_validation: m2a_core::reference_supermodel_generic::GenericReferenceRigAnalysisV2,
}

fn parse_reference_supermodel_authoring_v1(
    authoring_json: &str,
) -> Result<
    m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringDocumentV1,
    String,
> {
    serde_json::from_str::<
        m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringDocumentV1,
    >(authoring_json)
    .map_err(|source| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "code": "M2A-REFERENCE-SUPERMODEL-AUTHORING-JSON",
            "path": "authoringJson",
            "message": source.to_string()
        }))
    })
}

fn parse_reference_supermodel_authoring_v2(
    authoring_json: &str,
) -> Result<
    m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringDocumentV2,
    String,
> {
    serde_json::from_str::<
        m2a_core::reference_supermodel_authoring::ReferenceSupermodelRigAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|source| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 2,
            "code": "M2A-REFERENCE-SUPERMODEL-AUTHORING-V2-JSON",
            "path": "authoringJson",
            "message": source.to_string()
        }))
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReferenceSupermodelPreparationModeV3 {
    Diagnostic,
    Product,
}

fn reference_supermodel_skinning_options_v1(
    allow_excessive_branch_boundary_repair: bool,
) -> m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1 {
    m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1 {
        allow_excessive_branch_boundary_repair,
        retain_editable_draft_on_quality_failure: true,
    }
}

fn prepare_reference_supermodel_rig_v1_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_mode: ReferenceSupermodelAuthoringInputModeV1<'_>,
    mode: ReferenceSupermodelPreparationModeV3,
    skinning_options: m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1,
) -> Result<PreparedReferenceSupermodelRigV1, String> {
    let source_forward_text = source_forward;
    let source_forward = parse_reference_source_forward_v2(source_forward)?;
    let chain_payloads =
        parse_reference_supermodel_chain_blob_v2(reference_chain_blob, reference_chain_json)?;
    let analysis = m2a_core::reference_supermodel_generic::analyze_reference_supermodel_chain_v2(
        selected_supermodel_resref,
        &chain_payloads,
    )
    .map_err(|error| serialize_json(&error))?;
    let authoring_supermodel_resref = selected_supermodel_resref.to_ascii_lowercase();
    let source = m2a_core::glb::ingest_glb(source_glb, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let registered_source = m2a_core::reference_source_frame::source_uses_reference_bind_frame_v1(
        source_glb,
        selected_supermodel_resref,
        &analysis.report.selected_sha256,
        source_forward_text,
    )
    .map_err(|error| serialize_json(&error))?;
    let derive_rig = if registered_source {
        m2a_core::reference_supermodel_generic::derive_registered_reference_supermodel_rig_from_glb_v1
    } else {
        m2a_core::reference_supermodel_generic::derive_immutable_reference_supermodel_rig_from_glb_v4
    };
    let base_rig = derive_rig(
        &source,
        &analysis.motion_contract,
        &analysis.combined_reference,
        source_forward,
        skinning_options,
    )
    .map_err(|error| serialize_json(&error))?;
    let exact_chain_sha256 =
        m2a_core::reference_supermodel_authoring::reference_supermodel_exact_chain_sha256_v1(
            &authoring_supermodel_resref,
            &analysis.report.exact_chain,
        )
        .map_err(|error| serialize_json(&error))?;
    let authoring = match authoring_mode {
        ReferenceSupermodelAuthoringInputModeV1::Automatic => {
            m2a_core::reference_supermodel_authoring::new_reference_supermodel_rig_authoring_v1(
                &source.report.input.sha256,
                source_forward,
                &authoring_supermodel_resref,
                &exact_chain_sha256,
                &analysis.motion_contract.content_sha256,
                &base_rig.rig,
            )
            .map_err(|error| serialize_json(&error))?
        }
        ReferenceSupermodelAuthoringInputModeV1::Draft(authoring_json) => {
            m2a_core::reference_supermodel_authoring::seal_reference_supermodel_rig_authoring_v1(
                parse_reference_supermodel_authoring_v1(authoring_json)?,
            )
            .map_err(|error| serialize_json(&error))?
        }
        ReferenceSupermodelAuthoringInputModeV1::Sealed(authoring_json) => {
            parse_reference_supermodel_authoring_v1(authoring_json)?
        }
    };
    let authored =
        m2a_core::reference_supermodel_authoring::apply_reference_supermodel_rig_authoring_v1(
            &authoring,
            &source.report.input.sha256,
            source_forward,
            &authoring_supermodel_resref,
            &exact_chain_sha256,
            &analysis.motion_contract.content_sha256,
            &base_rig.rig,
        )
        .map_err(|error| serialize_json(&error))?;
    let authored_parts = authoring
        .joint_overrides
        .iter()
        .map(|row| row.carrier_part_number)
        .collect::<std::collections::BTreeSet<_>>();
    let authored_validation =
        m2a_core::reference_supermodel_generic::validate_authored_generic_reference_rig_v3(
            &analysis.motion_contract,
            &base_rig.report,
            &authored.rig,
            &authored_parts,
        )
        .map_err(|error| serialize_json(&error))?;
    if mode == ReferenceSupermodelPreparationModeV3::Product {
        require_reference_supermodel_preproduct_rig_v3(&authored_validation)?;
    }
    Ok(PreparedReferenceSupermodelRigV1 {
        source_forward,
        source,
        analysis,
        base_rig,
        exact_chain_sha256,
        authoring,
        authored,
        authored_validation,
    })
}

fn prepare_reference_supermodel_rig_v2_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_mode: ReferenceSupermodelAuthoringInputModeV2<'_>,
    mode: ReferenceSupermodelPreparationModeV3,
    skinning_options: m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1,
) -> Result<PreparedReferenceSupermodelRigV2, String> {
    let prepared_v1 = prepare_reference_supermodel_rig_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV1::Automatic,
        ReferenceSupermodelPreparationModeV3::Diagnostic,
        skinning_options,
    )?;
    let supermodel_resref = selected_supermodel_resref.to_ascii_lowercase();
    let structural_profile_sha256 = &prepared_v1
        .base_rig
        .report
        .structural_profile
        .content_sha256;
    let surface_anatomy_sha256 = &prepared_v1.base_rig.report.surface_anatomy.content_sha256;
    let fitter_algorithm = &prepared_v1.base_rig.report.algorithm;
    let authoring = match authoring_mode {
        ReferenceSupermodelAuthoringInputModeV2::Automatic => {
            m2a_core::reference_supermodel_authoring::new_reference_supermodel_rig_authoring_v2(
                &prepared_v1.source.report.input.sha256,
                prepared_v1.source_forward,
                &supermodel_resref,
                &prepared_v1.exact_chain_sha256,
                &prepared_v1.analysis.motion_contract.content_sha256,
                structural_profile_sha256,
                surface_anatomy_sha256,
                fitter_algorithm,
                &prepared_v1.base_rig.rig,
            )
            .map_err(|error| serialize_json(&error))?
        }
        ReferenceSupermodelAuthoringInputModeV2::Draft(json) => {
            m2a_core::reference_supermodel_authoring::seal_reference_supermodel_rig_authoring_v2(
                parse_reference_supermodel_authoring_v2(json)?,
            )
            .map_err(|error| serialize_json(&error))?
        }
        ReferenceSupermodelAuthoringInputModeV2::Sealed(json) => {
            parse_reference_supermodel_authoring_v2(json)?
        }
    };
    let authored =
        m2a_core::reference_supermodel_authoring::apply_reference_supermodel_rig_authoring_v2(
            &authoring,
            &prepared_v1.source.report.input.sha256,
            prepared_v1.source_forward,
            &supermodel_resref,
            &prepared_v1.exact_chain_sha256,
            &prepared_v1.analysis.motion_contract.content_sha256,
            structural_profile_sha256,
            surface_anatomy_sha256,
            fitter_algorithm,
            &prepared_v1.base_rig.rig,
        )
        .map_err(|error| serialize_json(&error))?;
    let authored_parts = authoring
        .joint_overrides
        .iter()
        .map(|row| row.carrier_part_number)
        .chain(
            authoring
                .landmark_overrides
                .iter()
                .map(|row| row.carrier_part_number),
        )
        .collect::<std::collections::BTreeSet<_>>();
    let authored_validation =
        m2a_core::reference_supermodel_generic::validate_authored_generic_reference_rig_v3(
            &prepared_v1.analysis.motion_contract,
            &prepared_v1.base_rig.report,
            &authored.rig,
            &authored_parts,
        )
        .map_err(|error| serialize_json(&error))?;
    if mode == ReferenceSupermodelPreparationModeV3::Product {
        require_reference_supermodel_preproduct_rig_v3(&authored_validation)?;
    }
    Ok(PreparedReferenceSupermodelRigV2 {
        source_forward: prepared_v1.source_forward,
        source: prepared_v1.source,
        analysis: prepared_v1.analysis,
        base_rig: prepared_v1.base_rig,
        exact_chain_sha256: prepared_v1.exact_chain_sha256,
        authoring,
        authored,
        authored_validation,
    })
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct ReferenceSupermodelPreparedRigWasmArtifactV1 {
    report_json: String,
    authoring_json: String,
    target_rig_json: String,
}

#[wasm_bindgen]
impl ReferenceSupermodelPreparedRigWasmArtifactV1 {
    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = authoringJson)]
    pub fn authoring_json(&self) -> String {
        self.authoring_json.clone()
    }

    #[wasm_bindgen(getter, js_name = targetRigJson)]
    pub fn target_rig_json(&self) -> String {
        self.target_rig_json.clone()
    }
}

fn prepare_reference_supermodel_rig_artifact_v1_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV1, String> {
    let prepared = prepare_reference_supermodel_rig_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV1::Automatic,
        ReferenceSupermodelPreparationModeV3::Diagnostic,
        reference_supermodel_skinning_options_v1(false),
    )?;
    let status = if prepared.authored_validation.joint_fit.status == "READY"
        && prepared.authored_validation.skinning.status == "READY"
        && prepared.authored_validation.bind_pose.status == "PASS"
    {
        "REFERENCE_SUPERMODEL_FITTED_RIG_READY"
    } else {
        "REFERENCE_SUPERMODEL_FITTED_RIG_NEEDS_AUTHORING"
    };
    let report_json = serialize_json(&serde_json::json!({
        "schemaVersion": 1,
        "status": status,
        "selectedSupermodelResref": prepared.analysis.report.selected_supermodel_resref,
        "source": prepared.source.report,
        "exactChainSha256": prepared.exact_chain_sha256,
        "motionContractSha256": prepared.analysis.motion_contract.content_sha256,
        "baseRigSha256": prepared.authoring.base_rig_sha256,
        "outputRigSha256": prepared.authored.report.output_rig_sha256,
        "exactChain": prepared.analysis.report.exact_chain,
        "structuralAnalysis": prepared.analysis.report,
        "rigAnalysis": prepared.base_rig.report,
        "authoredRigAnalysis": prepared.authored_validation,
        "rigAuthoring": prepared.authored.report,
        "motionQuality": "NOT_EVALUATED",
        "retailPayloadCopied": false
    }));
    Ok(ReferenceSupermodelPreparedRigWasmArtifactV1 {
        report_json,
        authoring_json: serialize_json(&prepared.authoring),
        target_rig_json: serialize_json(&prepared.authored.rig),
    })
}

/// Performs only exact-chain analysis, anatomy fitting and local skinning.
/// It intentionally does not build MDL bytes or evaluate all inherited clips,
/// so Studio can inspect and author the fitted rig before the expensive preview.
#[wasm_bindgen(js_name = prepareReferenceSupermodelRigV1)]
pub fn prepare_reference_supermodel_rig_v1(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV1, JsValue> {
    prepare_reference_supermodel_rig_artifact_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct ReferenceSupermodelPreparedRigWasmArtifactV2 {
    report_json: String,
    authoring_json: String,
    target_rig_json: String,
}

#[wasm_bindgen]
impl ReferenceSupermodelPreparedRigWasmArtifactV2 {
    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = authoringJson)]
    pub fn authoring_json(&self) -> String {
        self.authoring_json.clone()
    }

    #[wasm_bindgen(getter, js_name = targetRigJson)]
    pub fn target_rig_json(&self) -> String {
        self.target_rig_json.clone()
    }
}

fn prepare_reference_supermodel_rig_artifact_v2_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_mode: ReferenceSupermodelAuthoringInputModeV2<'_>,
    skinning_options: m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, String> {
    let prepared = prepare_reference_supermodel_rig_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        authoring_mode,
        ReferenceSupermodelPreparationModeV3::Diagnostic,
        skinning_options,
    )?;
    let status = if prepared.authored_validation.joint_fit.status == "READY"
        && prepared.authored_validation.skinning.status == "READY"
        && prepared.authored_validation.bind_pose.status == "PASS"
    {
        "REFERENCE_SUPERMODEL_FITTED_RIG_V2_READY"
    } else {
        "REFERENCE_SUPERMODEL_FITTED_RIG_V2_NEEDS_AUTHORING"
    };
    Ok(ReferenceSupermodelPreparedRigWasmArtifactV2 {
        report_json: serialize_json(&serde_json::json!({
            "schemaVersion": 2,
            "status": status,
            "selectedSupermodelResref": prepared.analysis.report.selected_supermodel_resref,
            "sourceForward": prepared.source_forward,
            "source": prepared.source.report,
            "exactChainSha256": prepared.exact_chain_sha256,
            "motionContractSha256": prepared.analysis.motion_contract.content_sha256,
            "structuralProfileSha256": prepared.base_rig.report.structural_profile.content_sha256,
            "surfaceAnatomySha256": prepared.base_rig.report.surface_anatomy.content_sha256,
            "baseRigSha256": prepared.authoring.base_rig_sha256,
            "outputRigSha256": prepared.authored.report.output_rig_sha256,
            "exactChain": prepared.analysis.report.exact_chain,
            "structuralAnalysis": prepared.analysis.report,
            "rigAnalysis": prepared.base_rig.report,
            "authoredRigAnalysis": prepared.authored_validation,
            "rigAuthoring": prepared.authored.report,
            "motionQuality": "NOT_EVALUATED",
            "retailPayloadCopied": false
        })),
        authoring_json: serialize_json(&prepared.authoring),
        target_rig_json: serialize_json(&prepared.authored.rig),
    })
}

/// Native release-corpus entrypoint. It runs the exact same diagnostic
/// preparation as Studio and intentionally produces no MDL/HAK/MOD payload.
pub fn prepare_reference_supermodel_rig_v2_native(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, String> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Automatic,
        reference_supermodel_skinning_options_v1(false),
    )
}

/// V2 preparation binds editable landmarks, component bindings and region
/// constraints to the exact structural profile and surface-anatomy hashes.
#[wasm_bindgen(js_name = prepareReferenceSupermodelRigV2)]
pub fn prepare_reference_supermodel_rig_v2(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, JsValue> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Automatic,
        reference_supermodel_skinning_options_v1(false),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Seals an editable V2 draft and reapplies it to the exact current base rig
/// without running the inherited-motion oracle.
#[wasm_bindgen(js_name = prepareReferenceSupermodelAuthoredRigV2)]
pub fn prepare_reference_supermodel_authored_rig_v2(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, JsValue> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Draft(authoring_json),
        reference_supermodel_skinning_options_v1(false),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Validates and applies an already sealed V2 document. This is the same
/// fail-closed authoring admission used by future preview/product V2 calls.
#[wasm_bindgen(js_name = validateReferenceSupermodelSealedRigV2)]
pub fn validate_reference_supermodel_sealed_rig_v2(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, JsValue> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Sealed(authoring_json),
        reference_supermodel_skinning_options_v1(false),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// V3 exposes the explicit experimental branch-repair policy. The selected
/// supermodel carrier hierarchy and bind matrices remain immutable.
#[wasm_bindgen(js_name = prepareReferenceSupermodelRigV3)]
pub fn prepare_reference_supermodel_rig_v3(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    allow_excessive_branch_boundary_repair: bool,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, JsValue> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Automatic,
        reference_supermodel_skinning_options_v1(allow_excessive_branch_boundary_repair),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = prepareReferenceSupermodelAuthoredRigV3)]
pub fn prepare_reference_supermodel_authored_rig_v3(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
    allow_excessive_branch_boundary_repair: bool,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, JsValue> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Draft(authoring_json),
        reference_supermodel_skinning_options_v1(allow_excessive_branch_boundary_repair),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = validateReferenceSupermodelSealedRigV3)]
pub fn validate_reference_supermodel_sealed_rig_v3(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
    allow_excessive_branch_boundary_repair: bool,
) -> Result<ReferenceSupermodelPreparedRigWasmArtifactV2, JsValue> {
    prepare_reference_supermodel_rig_artifact_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Sealed(authoring_json),
        reference_supermodel_skinning_options_v1(allow_excessive_branch_boundary_repair),
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_reference_supermodel_applied_preview_v2_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, String> {
    build_reference_supermodel_applied_preview_with_authoring_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        None,
    )
}

fn build_reference_supermodel_applied_preview_with_authoring_v1_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: Option<&str>,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, String> {
    let prepared = prepare_reference_supermodel_rig_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        authoring_json.map_or(
            ReferenceSupermodelAuthoringInputModeV1::Automatic,
            ReferenceSupermodelAuthoringInputModeV1::Draft,
        ),
        ReferenceSupermodelPreparationModeV3::Diagnostic,
        reference_supermodel_skinning_options_v1(false),
    )?;
    build_reference_supermodel_applied_preview_from_prepared_v3(
        source_glb,
        prepared.source_forward,
        &prepared.analysis,
        &prepared.authored_validation,
        &prepared.authored.rig,
        serde_json::to_value(&prepared.authored.report).map_err(|error| error.to_string())?,
        serialize_json(&prepared.authoring),
    )
}

fn build_reference_supermodel_applied_preview_from_prepared_v3(
    source_glb: &[u8],
    source_forward: m2a_core::profile_a::CreatureSourceForwardV1,
    analysis: &m2a_core::reference_supermodel_generic::ReferenceSupermodelChainAnalysisArtifactV2,
    validated_rig_analysis: &m2a_core::reference_supermodel_generic::GenericReferenceRigAnalysisV2,
    authored_rig: &m2a_core::profile_a::CreatureRigProfileV1,
    authored_report: serde_json::Value,
    authoring_json: String,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, String> {
    let artifact = m2a_core::reference_supermodel_motion::build_inherited_supermodel_motion_diagnostic_preview_v1(
        source_glb,
        authored_rig,
        &analysis.motion_contract,
        &analysis.combined_reference,
        &m2a_core::reference_supermodel_motion::default_reference_supermodel_writer_options_v1(
            "m2a_refpreview",
        ),
        source_forward,
        &m2a_core::reference_supermodel_motion::ReferenceSupermodelMinimalMtrMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: "m2arefprevtex".to_owned(),
            material_resref: "m2arefprevmtr".to_owned(),
            material_profile: m2a_core::creature_product::CreatureMaterialProfileV2 {
                schema_version: 2,
                target: m2a_core::creature_product::CreatureMaterialTargetV2::NwnEeMtr,
                normal_maps: false,
                tangent_space_ready: false,
                metallic_roughness_to_specular_gloss: false,
                emissive_to_self_illumination: false,
                alpha_mode: m2a_core::creature_product::CreatureAlphaModeV2::Opaque,
                double_sided: true,
            },
        },
    )
    .map_err(|error| serialize_json(&error))?;
    let full_product_motion_gate = artifact.report.motion_compatible
        && artifact.report.carrier_coverage.full_carrier_coverage
        && artifact.report.carrier_coverage.required_joint_coverage
        && artifact.report.skin_influence_coverage
        && artifact.motion_quality.inherited_clip_coverage
        && artifact.motion_quality.visible_motion_coverage
        && artifact.motion_quality.seam_pair_violation_count == 0
        && artifact.motion_quality.status == "PASS";
    let admission_v3 = m2a_core::reference_supermodel_admission::evaluate_reference_supermodel_admission_v3(
        &m2a_core::reference_supermodel_admission::ReferenceSupermodelAdmissionInputV3 {
            schema_version: 3,
            mode: m2a_core::reference_supermodel_admission::ReferenceSupermodelAdmissionModeV3::Diagnostic,
            structure_status: analysis.report.status.clone(),
            surface_anatomy_status: validated_rig_analysis.surface_anatomy.status.clone(),
            joint_fit_status: validated_rig_analysis.joint_fit.status.clone(),
            skinning_status: validated_rig_analysis.skinning.status.clone(),
            bind_pose_status: validated_rig_analysis.bind_pose.status.clone(),
            motion_quality_status: artifact.motion_quality.status.clone(),
            exact_chain_validated: analysis.report.structural_errors.is_empty(),
            full_carrier_coverage: artifact.report.carrier_coverage.full_carrier_coverage,
            required_joint_coverage: artifact.report.carrier_coverage.required_joint_coverage,
            skin_influence_coverage: artifact.report.skin_influence_coverage,
            inherited_clip_coverage: artifact.motion_quality.inherited_clip_coverage,
            visible_motion_coverage: artifact.motion_quality.visible_motion_coverage,
            seam_violation_count: artifact.motion_quality.seam_pair_violation_count,
            motion_compatible: artifact.report.motion_compatible,
            runtime_readiness: artifact.report.runtime_readiness.clone(),
            semantic_delta_required: false,
            semantic_delta_proven: false,
        },
    )
    .map_err(|error| serialize_json(&error))?;
    let preview_status = if full_product_motion_gate
        && validated_rig_analysis.surface_anatomy.status == "READY"
        && validated_rig_analysis.joint_fit.status == "READY"
        && validated_rig_analysis.skinning.status == "READY"
        && validated_rig_analysis.bind_pose.status == "PASS"
    {
        "APPLIED_PREVIEW_OFFLINE_PASS"
    } else {
        "DIAGNOSTIC_BLOCKED"
    };
    let apply_report_json = serialize_json(&serde_json::json!({
        "schemaVersion": 2,
        "status": preview_status,
        "supermodelResref": analysis.report.selected_supermodel_resref,
        "referenceFormat": analysis.report.selected_format,
        "referenceSha256": analysis.report.selected_sha256,
        "modelSha256": artifact.model.report.payload_sha256,
        "localAnimationCount": artifact.model.inspection.animations.len(),
        "inheritedAnimationCount": analysis.report.inherited_animation_names.len(),
        "requiredClipCount": analysis.motion_contract.required_clips.len(),
        "motionCompatible": artifact.report.motion_compatible,
        "bindPoseCompatible": artifact.report.bind_pose_compatible,
        "skinBindCompatible": artifact.report.skin_bind_compatible,
        "fullCarrierCoverage": artifact.report.carrier_coverage.full_carrier_coverage,
        "requiredJointCoverage": artifact.report.carrier_coverage.required_joint_coverage,
        "skinInfluenceCoverage": artifact.report.skin_influence_coverage,
        "inheritedClipCoverage": artifact.motion_quality.inherited_clip_coverage,
        "visibleMotionCoverage": artifact.motion_quality.visible_motion_coverage,
        "seamViolationCount": artifact.motion_quality.seam_pair_violation_count,
        "motionQualityStatus": artifact.motion_quality.status,
        "runtimeReadiness": artifact.report.runtime_readiness,
        "exactChain": analysis.report.exact_chain,
        "structuralAnalysis": analysis.report,
        "rigAnalysis": validated_rig_analysis,
        "experimentalAllowExcessiveSkinBranchRepair": validated_rig_analysis.skinning.branch_boundary_repair_limit_bypass_enabled,
        "admissionV3": admission_v3,
        "rigAuthoring": authored_report,
        "retailPayloadCopied": false,
        "motionBuild": artifact.report,
        "motionQuality": artifact.motion_quality,
    }));
    Ok(ReferenceSupermodelAppliedPreviewWasmArtifactV2 {
        model_bytes: artifact.model.payload,
        readback_json: serialize_json(&artifact.model.inspection),
        apply_report_json,
        authoring_json,
        target_rig_json: serialize_json(authored_rig),
    })
}

fn build_reference_supermodel_applied_preview_with_authoring_v2_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: Option<&str>,
    skinning_options: m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, String> {
    let prepared = prepare_reference_supermodel_rig_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        authoring_json.map_or(
            ReferenceSupermodelAuthoringInputModeV2::Automatic,
            ReferenceSupermodelAuthoringInputModeV2::Draft,
        ),
        ReferenceSupermodelPreparationModeV3::Diagnostic,
        skinning_options,
    )?;
    build_reference_supermodel_applied_preview_from_prepared_v3(
        source_glb,
        prepared.source_forward,
        &prepared.analysis,
        &prepared.authored_validation,
        &prepared.authored.rig,
        serde_json::to_value(&prepared.authored.report).map_err(|error| error.to_string())?,
        serialize_json(&prepared.authoring),
    )
}

fn require_reference_supermodel_preproduct_rig_v3(
    analysis: &m2a_core::reference_supermodel_generic::GenericReferenceRigAnalysisV2,
) -> Result<(), String> {
    let blocking = [
        (analysis.surface_anatomy.status != "READY").then_some((
            "BLOCKED_SURFACE_ANATOMY",
            "rigAnalysis.surfaceAnatomy",
            analysis.surface_anatomy.status.as_str(),
        )),
        (analysis.joint_fit.status != "READY").then_some((
            "BLOCKED_JOINT_FIT",
            "rigAnalysis.jointFit",
            analysis.joint_fit.status.as_str(),
        )),
        (analysis.skinning.status != "READY").then_some((
            "BLOCKED_SKINNING",
            "rigAnalysis.skinning",
            analysis.skinning.status.as_str(),
        )),
        (analysis.bind_pose.status != "PASS").then_some((
            "BLOCKED_BIND_POSE",
            "rigAnalysis.bindPose",
            analysis.bind_pose.status.as_str(),
        )),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if blocking.is_empty() {
        return Ok(());
    }
    let affected_parts = analysis
        .joint_fit
        .constraint_violations
        .iter()
        .flat_map(|constraint| constraint.part_numbers.iter().copied())
        .chain(
            analysis
                .bind_pose
                .violations
                .iter()
                .flat_map(|violation| violation.part_numbers.iter().copied()),
        )
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let (code, path, source_status) = blocking[0];
    Err(serialize_json(&serde_json::json!({
        "schemaVersion": 3,
        "code": code,
        "path": path,
        "message": "product preparation requires PASS for surface anatomy, joint fit, skinning and bind pose",
        "sourceStatus": source_status,
        "blockingCodes": blocking.iter().map(|row| row.0).collect::<Vec<_>>(),
        "affectedCarrierPartNumbers": affected_parts,
    })))
}

/// Native diagnostic entry point for repository examples and offline validation.
///
/// The public WASM boundary must translate errors to `JsValue`, but constructing a
/// `JsValue` is unsupported on non-wasm targets. Native callers therefore use this
/// string-error adapter so a rejected rig or motion gate is reported instead of
/// aborting the host process.
#[cfg(not(target_arch = "wasm32"))]
pub fn build_reference_supermodel_applied_preview_v2_native(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, String> {
    build_reference_supermodel_applied_preview_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
    )
}

/// Structure-driven applied-preview boundary for any exact catalog selection.
#[wasm_bindgen(js_name = buildReferenceSupermodelAppliedPreviewV2)]
pub fn build_reference_supermodel_applied_preview_v2(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, JsValue> {
    build_reference_supermodel_applied_preview_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Rebuilds the diagnostic preview from an editable V1 authoring draft. The returned artifact
/// contains the canonical sealed authoring document that must be passed to the product boundary.
#[wasm_bindgen(js_name = buildReferenceSupermodelAuthoredPreviewV1)]
pub fn build_reference_supermodel_authored_preview_v1(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, JsValue> {
    build_reference_supermodel_applied_preview_with_authoring_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        Some(authoring_json),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Builds preview from the structural-profile/anatomy bound Authoring V2
/// artifact. This is the preview half of the same V2 rig boundary consumed by
/// the product endpoint below.
#[cfg(not(target_arch = "wasm32"))]
pub fn build_reference_supermodel_authored_preview_v2_native(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, String> {
    build_reference_supermodel_applied_preview_with_authoring_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        Some(authoring_json),
        reference_supermodel_skinning_options_v1(false),
    )
}

#[wasm_bindgen(js_name = buildReferenceSupermodelAppliedPreviewV3)]
pub fn build_reference_supermodel_applied_preview_v3(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, JsValue> {
    build_reference_supermodel_applied_preview_with_authoring_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        None,
        reference_supermodel_skinning_options_v1(false),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = buildReferenceSupermodelAuthoredPreviewV2)]
pub fn build_reference_supermodel_authored_preview_v2(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, JsValue> {
    build_reference_supermodel_applied_preview_with_authoring_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        Some(authoring_json),
        reference_supermodel_skinning_options_v1(false),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// V4 preview accepts the explicit experimental branch-repair bypass while
/// preserving every later structural and inherited-motion validation.
#[wasm_bindgen(js_name = buildReferenceSupermodelAppliedPreviewV4)]
pub fn build_reference_supermodel_applied_preview_v4(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    allow_excessive_branch_boundary_repair: bool,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, JsValue> {
    build_reference_supermodel_applied_preview_with_authoring_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        None,
        reference_supermodel_skinning_options_v1(allow_excessive_branch_boundary_repair),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = buildReferenceSupermodelAuthoredPreviewV3)]
pub fn build_reference_supermodel_authored_preview_v3(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    source_forward: &str,
    authoring_json: &str,
    allow_excessive_branch_boundary_repair: bool,
) -> Result<ReferenceSupermodelAppliedPreviewWasmArtifactV2, JsValue> {
    build_reference_supermodel_applied_preview_with_authoring_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        Some(authoring_json),
        reference_supermodel_skinning_options_v1(allow_excessive_branch_boundary_repair),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct ReferenceSupermodelProductWasmArtifactV1 {
    hak_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    texture_bytes: Vec<u8>,
    material_bytes: Vec<u8>,
    appearance_two_da_bytes: Vec<u8>,
    report_json: String,
    manifest_json: String,
    summary_json: String,
    readback_json: String,
}

#[wasm_bindgen]
impl ReferenceSupermodelProductWasmArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }
    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }
    #[wasm_bindgen(js_name = takeTextureBytes)]
    pub fn take_texture_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_bytes)
    }
    #[wasm_bindgen(js_name = takeMaterialBytes)]
    pub fn take_material_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.material_bytes)
    }
    #[wasm_bindgen(js_name = takeAppearanceTwoDaBytes)]
    pub fn take_appearance_two_da_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.appearance_two_da_bytes)
    }
    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }
    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }
    #[wasm_bindgen(getter, js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.summary_json.clone()
    }
    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

#[allow(clippy::too_many_arguments)]
#[doc(hidden)]
pub fn build_reference_supermodel_creature_product_v2_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, String> {
    build_reference_supermodel_creature_product_with_authoring_v3_inner(
        selected_supermodel_resref,
        source_glb,
        appearance_two_da,
        reference_chain_blob,
        reference_chain_json,
        identity_json,
        source_forward,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_reference_supermodel_creature_product_with_authoring_v3_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
    authoring_json: Option<&str>,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::reference_supermodel_product::ReferenceSupermodelProductIdentityV2,
    >(identity_json)
    .map_err(|source| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 1,
            "code": "M2A-REFERENCE-SUPERMODEL-PRODUCT-IDENTITY-JSON",
            "path": "identityJson",
            "message": source.to_string()
        }))
    })?;
    let prepared = prepare_reference_supermodel_rig_v1_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        authoring_json.map_or(
            ReferenceSupermodelAuthoringInputModeV1::Automatic,
            ReferenceSupermodelAuthoringInputModeV1::Sealed,
        ),
        ReferenceSupermodelPreparationModeV3::Product,
        reference_supermodel_skinning_options_v1(false),
    )?;
    build_reference_supermodel_creature_product_from_prepared_v4(
        source_glb,
        appearance_two_da,
        &identity,
        prepared.source_forward,
        &prepared.analysis,
        &prepared.source,
        &prepared.authored_validation,
        &prepared.authored.rig,
        serde_json::to_value(&prepared.authored.report).map_err(|error| error.to_string())?,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_reference_supermodel_creature_product_from_prepared_v4(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity: &m2a_core::reference_supermodel_product::ReferenceSupermodelProductIdentityV2,
    source_forward: m2a_core::profile_a::CreatureSourceForwardV1,
    analysis: &m2a_core::reference_supermodel_generic::ReferenceSupermodelChainAnalysisArtifactV2,
    source: &m2a_core::glb::GlbIngestResult,
    validated_rig_analysis: &m2a_core::reference_supermodel_generic::GenericReferenceRigAnalysisV2,
    authored_rig: &m2a_core::profile_a::CreatureRigProfileV1,
    authored_report: serde_json::Value,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, String> {
    let motion = m2a_core::reference_supermodel_motion::bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_v1(
        source_glb,
        authored_rig,
        &analysis.motion_contract,
        &analysis.combined_reference,
        &m2a_core::reference_supermodel_motion::default_reference_supermodel_writer_options_v1(
            &identity.model_resref,
        ),
        source_forward,
        &m2a_core::reference_supermodel_motion::ReferenceSupermodelMinimalMtrMaterialOptionsV1 {
            schema_version: 1,
            texture_resref: identity.texture_resref.clone(),
            material_resref: identity.material_resref.clone(),
            material_profile: m2a_core::creature_product::CreatureMaterialProfileV2 {
                schema_version: 2,
                target: m2a_core::creature_product::CreatureMaterialTargetV2::NwnEeMtr,
                normal_maps: false,
                tangent_space_ready: false,
                metallic_roughness_to_specular_gloss: false,
                emissive_to_self_illumination: false,
                alpha_mode: m2a_core::creature_product::CreatureAlphaModeV2::Opaque,
                double_sided: true,
            },
        },
    )
    .map_err(|error| serialize_json(&error))?;
    if identity.rejected_baseline.is_some() && identity.semantic_controller_names.is_empty() {
        return Err(serialize_json(&serde_json::json!({
            "schemaVersion": 2,
            "code": "M2A-REFERENCE-SUPERMODEL-SEMANTIC-CONTROLLERS-MISSING",
            "path": "identity.semanticControllerNames",
            "message": "a rejected baseline requires explicit repair-area controller names"
        })));
    }
    let visible_surface_sha256 = if identity.semantic_controller_names.is_empty() {
        None
    } else {
        Some(
            m2a_core::reference_supermodel_motion::visible_controller_surface_semantic_sha256_v1(
                &motion.model.inspection,
                &identity.semantic_controller_names,
            )
            .map_err(|error| serialize_json(&error))?,
        )
    };
    let semantic_delta = match (&identity.rejected_baseline, &visible_surface_sha256) {
        (Some(baseline), Some(surface_sha256)) => Some(
            m2a_core::reference_supermodel_motion::evaluate_reference_supermodel_semantic_delta_v1(
                baseline,
                &motion.model.report.payload_sha256,
                surface_sha256,
            )
            .map_err(|error| serialize_json(&error))?,
        ),
        _ => None,
    };
    let product =
        m2a_core::reference_supermodel_product::package_reference_supermodel_creature_product_v2(
            source_glb,
            appearance_two_da,
            &analysis.report,
            validated_rig_analysis,
            &identity,
            &motion,
            identity.rejected_baseline.is_some(),
            semantic_delta.as_ref(),
        )
        .map_err(|error| serialize_json(&error))?;
    let report_json = serialize_json(&serde_json::json!({
        "schemaVersion": 2,
        "status": "REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED",
        "selectedSupermodelResref": analysis.report.selected_supermodel_resref,
        "selectedFormat": analysis.report.selected_format,
        "identity": product.identity,
        "source": source.report,
        "model": motion.model.report,
        "texture": product.texture.report,
        "appearance": product.appearance.report,
        "hak": product.hak.report,
        "packageManifest": product.package_manifest,
        "motionBuild": motion.report,
        "motionQuality": motion.motion_quality,
        "semanticDelta": semantic_delta,
        "visibleSurfaceSemanticSha256": visible_surface_sha256,
        "exactChain": analysis.report.exact_chain,
        "structuralAnalysis": product.selection_analysis,
        "rigAnalysis": validated_rig_analysis,
        "rigAuthoring": authored_report,
        "appearanceDonorResref": product.appearance_donor_resref,
        "appearanceDonorPhysicalRow": product.appearance_donor_physical_row,
        "exportAdmission": product.admission,
        "admissionV3": product.admission_v3,
        "retailPayloadCopied": false,
        "ownerRuntimeProof": "NOT_RUN_HUMAN_OWNED"
    }));
    let summary_json = serialize_json(&serde_json::json!({
        "schemaVersion": 2,
        "status": "REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED",
        "selectedSupermodelResref": analysis.report.selected_supermodel_resref,
        "selectedFormat": analysis.report.selected_format,
        "identity": identity,
        "motionCompatible": true,
        "runtimeReadiness": "RUNTIME_UNPROVEN",
        "fullCarrierCoverage": product.admission.full_carrier_coverage,
        "requiredJointCoverage": product.admission.required_joint_coverage,
        "skinInfluenceCoverage": product.admission.skin_influence_coverage,
        "inheritedClipCoverage": product.admission.inherited_clip_coverage,
        "visibleMotionCoverage": product.admission.visible_motion_coverage,
        "seamViolationCount": product.admission.seam_violation_count,
        "ownerRuntimeProof": "NOT_RUN_HUMAN_OWNED"
    }));
    let manifest_json = serialize_json(&serde_json::json!({
        "schemaVersion": 2,
        "status": "REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED",
        "packageManifest": product.package_manifest,
        "exactChain": analysis.report.exact_chain,
        "retailPayloadCopied": false
    }));
    Ok(ReferenceSupermodelProductWasmArtifactV1 {
        hak_bytes: product.hak.payload,
        model_bytes: motion.model.payload,
        texture_bytes: product.texture.payload,
        material_bytes: motion.mtr_resource.payload,
        appearance_two_da_bytes: product.appearance.payload,
        report_json,
        manifest_json,
        summary_json,
        readback_json: serialize_json(&motion.model.inspection),
    })
}

#[allow(clippy::too_many_arguments)]
fn build_reference_supermodel_creature_product_with_authoring_v4_inner(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
    authoring_json: &str,
    skinning_options: m2a_core::reference_supermodel_skinning::ReferenceSupermodelSkinningOptionsV1,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::reference_supermodel_product::ReferenceSupermodelProductIdentityV2,
    >(identity_json)
    .map_err(|source| {
        serialize_json(&serde_json::json!({
            "schemaVersion": 2,
            "code": "M2A-REFERENCE-SUPERMODEL-PRODUCT-IDENTITY-JSON",
            "path": "identityJson",
            "message": source.to_string()
        }))
    })?;
    let prepared = prepare_reference_supermodel_rig_v2_inner(
        selected_supermodel_resref,
        source_glb,
        reference_chain_blob,
        reference_chain_json,
        source_forward,
        ReferenceSupermodelAuthoringInputModeV2::Sealed(authoring_json),
        ReferenceSupermodelPreparationModeV3::Product,
        skinning_options,
    )?;
    build_reference_supermodel_creature_product_from_prepared_v4(
        source_glb,
        appearance_two_da,
        &identity,
        prepared.source_forward,
        &prepared.analysis,
        &prepared.source,
        &prepared.authored_validation,
        &prepared.authored.rig,
        serde_json::to_value(&prepared.authored.report).map_err(|error| error.to_string())?,
    )
}

/// Native counterpart of the sealed-authoring product boundary. It preserves
/// all admission gates and returns structured string errors instead of JsValue.
#[allow(clippy::too_many_arguments)]
pub fn build_reference_supermodel_creature_product_v4_native(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, String> {
    build_reference_supermodel_creature_product_with_authoring_v4_inner(
        selected_supermodel_resref,
        source_glb,
        appearance_two_da,
        reference_chain_blob,
        reference_chain_json,
        identity_json,
        source_forward,
        authoring_json,
        reference_supermodel_skinning_options_v1(false),
    )
}

/// Produces the product-only Creature MDL/TGA/MTR/appearance.2da/HAK set.
/// Diagnostic preview output can never reach this boundary.
#[wasm_bindgen(js_name = buildReferenceSupermodelCreatureProductV2)]
pub fn build_reference_supermodel_creature_product_v2(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, JsValue> {
    build_reference_supermodel_creature_product_v2_inner(
        selected_supermodel_resref,
        source_glb,
        appearance_two_da,
        reference_chain_blob,
        reference_chain_json,
        identity_json,
        source_forward,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Product boundary for a sealed V1 authored target rig. Unlike preview, this boundary never
/// reseals a draft: stale or modified authoring fails closed before model generation.
#[wasm_bindgen(js_name = buildReferenceSupermodelCreatureProductV3)]
pub fn build_reference_supermodel_creature_product_v3(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, JsValue> {
    build_reference_supermodel_creature_product_with_authoring_v3_inner(
        selected_supermodel_resref,
        source_glb,
        appearance_two_da,
        reference_chain_blob,
        reference_chain_json,
        identity_json,
        source_forward,
        Some(authoring_json),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Product boundary for a sealed structural-profile/anatomy-bound Authoring
/// V2 document. It never reseals drafts and consumes the same authored rig as
/// `buildReferenceSupermodelAuthoredPreviewV2`.
#[wasm_bindgen(js_name = buildReferenceSupermodelCreatureProductV4)]
pub fn build_reference_supermodel_creature_product_v4(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
    authoring_json: &str,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, JsValue> {
    build_reference_supermodel_creature_product_with_authoring_v4_inner(
        selected_supermodel_resref,
        source_glb,
        appearance_two_da,
        reference_chain_blob,
        reference_chain_json,
        identity_json,
        source_forward,
        authoring_json,
        reference_supermodel_skinning_options_v1(false),
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// V5 product boundary carries the same explicit branch-repair policy used by
/// the accepted V4/V3 preview. All pre-product quality gates remain active.
#[wasm_bindgen(js_name = buildReferenceSupermodelCreatureProductV5)]
pub fn build_reference_supermodel_creature_product_v5(
    selected_supermodel_resref: &str,
    source_glb: &[u8],
    appearance_two_da: &[u8],
    reference_chain_blob: &[u8],
    reference_chain_json: &str,
    identity_json: &str,
    source_forward: &str,
    authoring_json: &str,
    allow_excessive_branch_boundary_repair: bool,
) -> Result<ReferenceSupermodelProductWasmArtifactV1, JsValue> {
    build_reference_supermodel_creature_product_with_authoring_v4_inner(
        selected_supermodel_resref,
        source_glb,
        appearance_two_da,
        reference_chain_blob,
        reference_chain_json,
        identity_json,
        source_forward,
        authoring_json,
        reference_supermodel_skinning_options_v1(allow_excessive_branch_boundary_repair),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod reference_supermodel_motion_boundary_tests {
    use super::{
        build_exact_reference_supermodel_motion_contract_v3_json_inner,
        build_reference_supermodel_applied_preview_v2_inner,
        build_reference_supermodel_applied_preview_with_authoring_v1_inner,
        build_reference_supermodel_creature_product_v2_inner,
        build_reference_supermodel_creature_product_with_authoring_v3_inner,
    };
    use m2a_core::{
        mdl::{
            MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
            MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlFormatProfileV1,
            MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1, MdlWriterOptionsV1,
            write_binary_mdl_with_animations,
        },
        model_ir::{
            AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1,
        },
        reference_supermodel_motion::{
            ReferenceSupermodelExactContractOptionsV3, ReferenceSupermodelSemanticNodeV3,
            build_exact_reference_supermodel_motion_contract_v3,
            default_reference_supermodel_motion_tolerances_v2,
        },
    };
    use sha2::{Digest, Sha256};
    use std::{
        env, fs,
        fs::File,
        io::{Read, Seek, SeekFrom},
        path::Path,
    };

    fn identity() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]
    }

    #[test]
    fn reference_supermodel_preview_rejects_an_unknown_source_axis_before_ingest() {
        let error = build_reference_supermodel_applied_preview_v2_inner(
            "c_any",
            b"not-a-glb",
            b"not-a-chain",
            "[]",
            "UP",
        )
        .unwrap_err();
        assert!(error.contains("M2A-REFERENCE-SUPERMODEL-SOURCE-FORWARD-INVALID"));
    }

    #[test]
    fn any_family_reaches_structural_analysis_instead_of_a_profile_whitelist() {
        let error = build_reference_supermodel_applied_preview_v2_inner(
            "c_unregistered",
            b"not-a-glb",
            b"",
            "[]",
            "POSITIVE_Z",
        )
        .unwrap_err();
        assert!(error.contains("M2A-REFERENCE-SUPERMODEL-SELECTION-INVALID"));
        assert!(!error.contains("PROFILE_MISSING"));
    }

    fn read_key_model_resource(key_path: &Path, resref: &str) -> Vec<u8> {
        let key_bytes = fs::read(key_path).expect("read the selected NWN KEY");
        let index = m2a_core::supermodel_catalog::index_nwn_key_models_v1(&key_bytes)
            .expect("index the selected NWN KEY");
        let locator = index
            .models
            .iter()
            .find(|model| model.resref.eq_ignore_ascii_case(resref))
            .expect("find the exact model resref in the selected KEY");
        let logical_bif = index
            .bifs
            .iter()
            .find(|bif| bif.index == locator.bif_index)
            .expect("resolve the model BIF from the selected KEY");
        let key_parent = key_path.parent().expect("KEY has a parent directory");
        let installation_root = key_parent.parent().unwrap_or(key_parent);
        let native_relative = logical_bif
            .logical_name
            .replace(['\\', '/'], std::path::MAIN_SEPARATOR_STR);
        let bif_path = [
            installation_root.join(&native_relative),
            key_parent.join(&native_relative),
        ]
        .into_iter()
        .find(|candidate| candidate.is_file())
        .expect("resolve the exact BIF referenced by the selected KEY");
        let mut bif = File::open(&bif_path).expect("open the exact BIF referenced by the KEY");
        let mut header = [0_u8; 20];
        bif.read_exact(&mut header).expect("read the BIF header");
        let plan = m2a_core::supermodel_catalog::plan_nwn_bif_index_v1(&header)
            .expect("plan the exact BIF resource-table read");
        bif.seek(SeekFrom::Start(plan.table_offset as u64))
            .expect("seek to the BIF resource table");
        let mut table = vec![0_u8; plan.table_byte_length];
        bif.read_exact(&mut table)
            .expect("read the BIF resource table");
        let bif_index = m2a_core::supermodel_catalog::index_nwn_bif_table_v1(&header, &table)
            .expect("index the exact BIF resource table");
        let resource = bif_index
            .resources
            .iter()
            .find(|resource| resource.resource_index == locator.resource_index)
            .expect("resolve the exact BIF model resource");
        assert_eq!(
            resource.resource_type as u16,
            m2a_core::supermodel_catalog::NWN_MDL_RESOURCE_TYPE
        );
        bif.seek(SeekFrom::Start(resource.payload_offset as u64))
            .expect("seek to the exact BIF model payload");
        let mut payload = vec![0_u8; resource.payload_size as usize];
        bif.read_exact(&mut payload)
            .expect("read the exact BIF model payload");
        payload
    }

    fn read_exact_key_supermodel_chain(
        key_path: &Path,
        selected_resref: &str,
    ) -> (Vec<u8>, String) {
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        let mut current = selected_resref.to_ascii_lowercase();
        let mut seen = std::collections::BTreeSet::new();
        while !current.eq_ignore_ascii_case("NULL") {
            assert!(
                seen.insert(current.clone()),
                "retail chain cycle at {current}"
            );
            let payload = read_key_model_resource(key_path, &current);
            let inspection =
                m2a_core::reference_supermodel_generic::inspect_reference_supermodel_mdl_v2(
                    &payload,
                    m2a_core::reference_supermodel_generic::ReferenceSupermodelFormatV2::Auto,
                )
                .expect("inspect exact retail chain resource");
            let offset = blob.len();
            let sha256 = format!("{:x}", Sha256::digest(&payload));
            blob.extend_from_slice(&payload);
            descriptors.push(serde_json::json!({
                "resref": current,
                "supermodelResref": inspection.model.supermodel_name,
                "format": if inspection.format == "NWN_ASCII_MDL" { "ASCII" } else { "BINARY" },
                "sha256": sha256,
                "byteOffset": offset,
                "byteLength": payload.len()
            }));
            current = inspection.model.supermodel_name.to_ascii_lowercase();
        }
        (blob, serde_json::to_string(&descriptors).unwrap())
    }

    fn static_textured_owned_glb() -> Vec<u8> {
        let glb = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1()
            .expect("build the owned textured GLB fixture");
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_length;
        let mut root: serde_json::Value =
            serde_json::from_slice(&glb[20..json_end]).expect("parse owned GLB JSON");
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);
        root["nodes"] = serde_json::json!([{
            "name": "reference-supermodel-e2e-source",
            "mesh": 0
        }]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .expect("primitive attributes");
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");
        let mut json = serde_json::to_vec(&root).expect("serialize static owned GLB JSON");
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let mut result = Vec::new();
        result.extend_from_slice(b"glTF");
        result.extend_from_slice(&2_u32.to_le_bytes());
        result.extend_from_slice(&0_u32.to_le_bytes());
        result.extend_from_slice(&(json.len() as u32).to_le_bytes());
        result.extend_from_slice(b"JSON");
        result.extend_from_slice(&json);
        result.extend_from_slice(&glb[json_end..]);
        let total_length = result.len() as u32;
        result[8..12].copy_from_slice(&total_length.to_le_bytes());
        result
    }

    fn disconnected_static_textured_owned_glb() -> Vec<u8> {
        let mut glb = static_textured_owned_glb();
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_length;
        let mut root: serde_json::Value =
            serde_json::from_slice(&glb[20..json_end]).expect("parse static GLB JSON");
        let centers = [
            [-1.0_f32, 0.0, 0.5],
            [1.0, 0.0, 0.5],
            [0.0, -1.0, 0.5],
            [0.0, 1.0, 0.5],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 2.0],
        ];
        let mut target_positions = Vec::with_capacity(24);
        for center in centers {
            for [dx, dy] in [
                [-0.01_f32, -0.01],
                [-0.01, 0.01],
                [0.01, 0.01],
                [0.01, -0.01],
            ] {
                target_positions.push([center[0] + dx, center[1] + dy, center[2]]);
            }
        }
        // Inverse of the POSITIVE_Z Creature basis used by the generic
        // retargeter: target = [-source.x, source.z, source.y].
        let source_positions = target_positions
            .iter()
            .map(|point| [-point[0], point[2], point[1]])
            .collect::<Vec<_>>();
        root["accessors"][0]["min"] = serde_json::json!([-1.01, 0.0, -1.01]);
        root["accessors"][0]["max"] = serde_json::json!([1.01, 2.0, 1.01]);
        let position_view = root["accessors"][0]["bufferView"]
            .as_u64()
            .expect("position buffer view") as usize;
        let position_offset = root["bufferViews"][position_view]
            .get("byteOffset")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize
            + root["accessors"][0]
                .get("byteOffset")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize;
        let mut json = serde_json::to_vec(&root).expect("serialize disconnected GLB JSON");
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let old_bin = glb[(json_end + 8)..].to_vec();
        let mut bin = old_bin;
        for (vertex, point) in source_positions.iter().enumerate() {
            for (axis, value) in point.iter().enumerate() {
                let offset = position_offset + (vertex * 3 + axis) * 4;
                bin[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        glb.clear();
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&0_u32.to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"BIN\0");
        glb.extend_from_slice(&bin);
        let total_length = glb.len() as u32;
        glb[8..12].copy_from_slice(&total_length.to_le_bytes());
        glb
    }

    #[test]
    fn c_wolf_structural_ground_terminals_are_explicit_when_configured() {
        let Some(key_path) = env::var_os("M2A_REFERENCE_NWN_KEY") else {
            eprintln!("skipped: M2A_REFERENCE_NWN_KEY is not configured");
            return;
        };
        let (reference_chain, reference_chain_json) =
            read_exact_key_supermodel_chain(Path::new(&key_path), "c_wolf");
        let payloads = super::parse_reference_supermodel_chain_blob_v2(
            &reference_chain,
            &reference_chain_json,
        )
        .expect("parse exact c_wolf chain");
        let analysis =
            m2a_core::reference_supermodel_generic::analyze_reference_supermodel_chain_v2(
                "c_wolf", &payloads,
            )
            .expect("analyze exact c_wolf chain");
        let terminals = analysis
            .motion_contract
            .nodes
            .iter()
            .filter(|node| node.structural_role == "LIMB_GROUND_CONTACT_TERMINAL")
            .map(|node| {
                serde_json::json!({
                    "partNumber": node.part_number,
                    "name": node.name,
                    "parentPartNumber": node.parent_part_number,
                    "anchorRole": node.anchor_role,
                    "localTranslation": [
                        node.carrier_bind_local_matrix[12],
                        node.carrier_bind_local_matrix[13],
                        node.carrier_bind_local_matrix[14]
                    ],
                    "dynamicClips": node.dynamic_clips
                })
            })
            .collect::<Vec<_>>();
        eprintln!("{}", serde_json::json!({ "groundTerminals": terminals }));
        assert_eq!(terminals.len(), 4);
    }

    #[test]
    fn c_wolf_applied_preview_builds_from_real_selected_key_and_source_when_configured() {
        let Some(key_path) = env::var_os("M2A_REFERENCE_NWN_KEY") else {
            eprintln!("skipped: M2A_REFERENCE_NWN_KEY is not configured");
            return;
        };
        let Some(source_path) = env::var_os("M2A_CWOLF_PREVIEW_SOURCE_GLB") else {
            eprintln!("skipped: M2A_CWOLF_PREVIEW_SOURCE_GLB is not configured");
            return;
        };
        let (reference_chain, reference_chain_json) =
            read_exact_key_supermodel_chain(Path::new(&key_path), "c_wolf");
        let source = fs::read(&source_path).expect("read the selected source GLB");
        assert_eq!(source.len(), 29_889_104);
        assert_eq!(
            format!("{:x}", Sha256::digest(&source)),
            "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda"
        );

        let artifact = build_reference_supermodel_applied_preview_v2_inner(
            "c_wolf",
            &source,
            &reference_chain,
            &reference_chain_json,
            "POSITIVE_Z",
        )
        .expect("build a structure-derived c_wolf applied preview");
        let readback: serde_json::Value =
            serde_json::from_str(&artifact.readback_json).expect("parse applied-model readback");
        let report: serde_json::Value =
            serde_json::from_str(&artifact.apply_report_json).expect("parse application report");
        let failed_cells = report["motionQuality"]["jointClipCoverageMatrix"]
            .as_array()
            .expect("joint×clip coverage matrix")
            .iter()
            .filter(|cell| cell["pass"] == false)
            .cloned()
            .collect::<Vec<_>>();
        let problem_clips = report["motionQuality"]["clips"]
            .as_array()
            .expect("motion quality clips")
            .iter()
            .filter(|clip| {
                clip["triangleAreaCollapseCount"].as_u64().unwrap_or(0) > 0
                    || clip["triangleAreaExpansionCount"].as_u64().unwrap_or(0) > 0
                    || clip["pawContactViolationCount"].as_u64().unwrap_or(0) > 0
                    || clip["pawSideViolationCount"].as_u64().unwrap_or(0) > 0
            })
            .map(|clip| {
                serde_json::json!({
                    "clipName": clip["clipName"],
                    "triangleAreaCollapseCount": clip["triangleAreaCollapseCount"],
                    "triangleAreaExpansionCount": clip["triangleAreaExpansionCount"],
                    "worstTriangleAreaCollapse": clip["worstTriangleAreaCollapse"],
                    "worstTriangleAreaExpansion": clip["worstTriangleAreaExpansion"],
                    "pawContactViolationCount": clip["pawContactViolationCount"],
                    "pawSideViolationCount": clip["pawSideViolationCount"],
                    "maxEdgeRatio": clip["maxEdgeRatio"],
                    "minEdgeRatio": clip["minEdgeRatio"]
                })
            })
            .collect::<Vec<_>>();
        let worst_expansion_clip = report["motionQuality"]["clips"]
            .as_array()
            .and_then(|clips| {
                clips
                    .iter()
                    .max_by_key(|clip| clip["triangleAreaExpansionCount"].as_u64().unwrap_or(0))
            });
        let worst_collapse_clip = report["motionQuality"]["clips"]
            .as_array()
            .and_then(|clips| {
                clips
                    .iter()
                    .max_by_key(|clip| clip["triangleAreaCollapseCount"].as_u64().unwrap_or(0))
            });
        eprintln!(
            "M2A_WORST_AREA_DIAGNOSTICS {}",
            serde_json::json!({
                "worstExpansionClip": worst_expansion_clip.map(|clip| serde_json::json!({
                    "clipName": clip["clipName"],
                    "count": clip["triangleAreaExpansionCount"],
                    "triangle": clip["worstTriangleAreaExpansion"]
                })),
                "worstCollapseClip": worst_collapse_clip.map(|clip| serde_json::json!({
                    "clipName": clip["clipName"],
                    "count": clip["triangleAreaCollapseCount"],
                    "triangle": clip["worstTriangleAreaCollapse"]
                }))
            })
        );
        eprintln!(
            "{}",
            serde_json::json!({
                "sourceSha256": format!("{:x}", Sha256::digest(&source)),
                "modelSha256": report["modelSha256"],
                "carrierNodeCount": report["rigAnalysis"]["carrierNodeCount"],
                "activeWeightedBoneCount": report["rigAnalysis"]["activeWeightedBoneCount"],
                "initialUnweightedRequiredJointNames": report["rigAnalysis"]["initialUnweightedRequiredJointNames"],
                "surfaceComponentCount": report["rigAnalysis"]["surfaceComponentCount"],
                "stabilizedSmallComponentCount": report["rigAnalysis"]["stabilizedSmallComponentCount"],
                "stabilizedSmallComponentVertexCount": report["rigAnalysis"]["stabilizedSmallComponentVertexCount"],
                "fittedGroundContactChainCount": report["rigAnalysis"]["fittedGroundContactChainCount"],
                "requiredClipCount": report["motionQuality"]["requiredClipCount"],
                "sampledClipCount": report["motionQuality"]["sampledClipCount"],
                "jointClipRequiredCount": report["motionQuality"]["jointClipRequiredCount"],
                "jointClipPassCount": report["motionQuality"]["jointClipPassCount"],
                "failedJointClipCells": failed_cells,
                "seamPairSampleCount": report["motionQuality"]["seamPairSampleCount"],
                "seamPairViolationCount": report["motionQuality"]["seamPairViolationCount"],
                "pawContactViolationCount": report["motionQuality"]["pawContactViolationCount"],
                "pawSideViolationCount": report["motionQuality"]["pawSideViolationCount"],
                "pawClusterCount": report["motionQuality"]["clips"][0]["pawClusterCount"],
                "triangleAreaCollapseCount": report["motionQuality"]["triangleAreaCollapseCount"],
                "triangleAreaCollapseAllowedCount": report["motionQuality"]["triangleAreaCollapseAllowedCount"],
                "triangleAreaExpansionCount": report["motionQuality"]["triangleAreaExpansionCount"],
                "triangleAreaExpansionAllowedCount": report["motionQuality"]["triangleAreaExpansionAllowedCount"],
                "edgeOutsideSoftLimitCount": report["motionQuality"]["edgeOutsideSoftLimitCount"],
                "edgeSoftSampleCount": report["motionQuality"]["edgeSoftSampleCount"],
                "edgeOutsideSoftAllowedCount": report["motionQuality"]["edgeOutsideSoftAllowedCount"],
                "edgeOutsideHardLimitCount": report["motionQuality"]["edgeOutsideHardLimitCount"],
                "edgeOutsideHardAllowedCount": report["motionQuality"]["edgeOutsideHardAllowedCount"],
                "problemClipCount": problem_clips.len(),
                "status": report["motionQuality"]["status"]
            })
        );
        assert_eq!(readback["model"]["supermodelName"], "c_wolf");
        assert_eq!(readback["animations"].as_array().map(Vec::len), Some(0));
        assert_eq!(report["inheritedAnimationCount"], 42);
        assert_eq!(report["requiredClipCount"], 42);
        assert_eq!(report["bindPoseCompatible"], true);
        assert_eq!(report["skinBindCompatible"], true);
        assert_eq!(report["fullCarrierCoverage"], true);
        assert_eq!(report["requiredJointCoverage"], true);
        assert_eq!(report["skinInfluenceCoverage"], true);
        assert_eq!(report["inheritedClipCoverage"], true);
        assert_eq!(report["visibleMotionCoverage"], true);
        assert_eq!(report["seamViolationCount"], 0);
        assert_eq!(report["motionQualityStatus"], "PASS");
        assert_eq!(report["rigAnalysis"]["carrierNodeCount"], 30);
        assert_eq!(
            report["rigAnalysis"]["activeWeightedBoneCount"],
            report["rigAnalysis"]["allowedBoneCount"]
        );
        assert_eq!(
            report["structuralAnalysis"]["exactChain"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(report["structuralAnalysis"]["retailPayloadCopied"], false);
        assert!(report["rigAnalysis"]["weightedBoneCount"].as_u64().unwrap() >= 2);
        assert!(!artifact.model_bytes.is_empty());
        eprintln!("{}", artifact.apply_report_json);
    }

    #[test]
    fn exact_c_wolf_full_skeleton_reaches_product_packaging_when_configured() {
        let Some(key_path) = env::var_os("M2A_REFERENCE_NWN_KEY") else {
            eprintln!("skipped: M2A_REFERENCE_NWN_KEY is not configured");
            return;
        };
        let Some(source_path) = env::var_os("M2A_CWOLF_PREVIEW_SOURCE_GLB") else {
            eprintln!("skipped: M2A_CWOLF_PREVIEW_SOURCE_GLB is not configured");
            return;
        };
        let (reference_chain, reference_chain_json) =
            read_exact_key_supermodel_chain(Path::new(&key_path), "c_wolf");
        let source = fs::read(source_path).expect("read exact configured source GLB");
        let product = build_reference_supermodel_creature_product_v2_inner(
            "c_wolf",
            &source,
            b"2DA V2.0\n\nLABEL MODELTYPE RACE\n0 dummy S c_dog\n",
            &reference_chain,
            &reference_chain_json,
            r#"{"modelResref":"m2acwprod","textureResref":"m2acwtex","materialResref":"m2acwmtr","hakResref":"m2acwhak","appearanceLabel":"M2A_CWOLF_PRODUCT","appearanceDonorResrefs":["c_dog"],"rejectedBaseline":{"modelSha256":"a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f","visibleSurfaceSemanticSha256":"ff75e44d8c903e68fdded68061c594013374cd4255b9b31b356a9c77e64ab15a"},"semanticControllerNames":["Wolf_tail","Wolf_tailend"]}"#,
            "POSITIVE_Z",
        )
        .expect("exact c_wolf full-skeleton product admission");
        let report: serde_json::Value = serde_json::from_str(&product.report_json).unwrap();
        assert_eq!(report["selectedSupermodelResref"], "c_wolf");
        assert_eq!(report["semanticDelta"]["exportDeltaProven"], true);
        assert_eq!(report["exportAdmission"]["fullCarrierCoverage"], true);
        assert_eq!(report["exportAdmission"]["requiredJointCoverage"], true);
        assert_eq!(report["exportAdmission"]["skinInfluenceCoverage"], true);
        assert_eq!(report["exportAdmission"]["inheritedClipCoverage"], true);
        assert_eq!(report["exportAdmission"]["visibleMotionCoverage"], true);
        assert_eq!(report["exportAdmission"]["seamViolationCount"], 0);
        assert!(!product.hak_bytes.is_empty());
    }

    #[test]
    fn real_non_c_wolf_supermodel_completes_analysis_preview_and_product_when_configured() {
        let Some(key_path) = env::var_os("M2A_REFERENCE_NWN_KEY") else {
            eprintln!("skipped: M2A_REFERENCE_NWN_KEY is not configured");
            return;
        };
        let selected =
            env::var("M2A_REFERENCE_NON_CWOLF_RESREF").unwrap_or_else(|_| "c_bat".to_owned());
        assert_ne!(selected.to_ascii_lowercase(), "c_wolf");
        let (reference_chain, reference_chain_json) =
            read_exact_key_supermodel_chain(Path::new(&key_path), &selected);
        let source = disconnected_static_textured_owned_glb();
        let preview = build_reference_supermodel_applied_preview_v2_inner(
            &selected,
            &source,
            &reference_chain,
            &reference_chain_json,
            "POSITIVE_Z",
        )
        .expect("apply a real non-c_wolf supermodel through structural analysis");
        let preview_report: serde_json::Value =
            serde_json::from_str(&preview.apply_report_json).unwrap();
        assert_eq!(preview_report["supermodelResref"], selected);
        assert_eq!(
            preview_report["structuralAnalysis"]["status"],
            "REFERENCE_SUPERMODEL_STRUCTURALLY_READY"
        );
        assert_eq!(
            preview_report["motionQualityStatus"], "PASS",
            "non-wolf preview report: {}",
            preview.apply_report_json
        );
        let identity_json = serde_json::to_string(&serde_json::json!({
            "modelResref": "m2arefprod",
            "textureResref": "m2areftex",
            "materialResref": "m2arefmtr",
            "hakResref": "m2arefhak",
            "appearanceLabel": "M2A_REFERENCE_PRODUCT",
            "appearanceDonorResrefs": [selected],
            "semanticControllerNames": []
        }))
        .unwrap();
        let appearance = format!("2DA V2.0\n\nLABEL MODELTYPE RACE\n0 donor S {}\n", selected);
        let product = build_reference_supermodel_creature_product_v2_inner(
            &selected,
            &source,
            appearance.as_bytes(),
            &reference_chain,
            &reference_chain_json,
            &identity_json,
            "POSITIVE_Z",
        )
        .expect("complete real non-c_wolf offline Creature packaging");
        let report: serde_json::Value = serde_json::from_str(&product.report_json).unwrap();
        assert_eq!(report["selectedSupermodelResref"], selected);
        assert_eq!(
            report["exportAdmission"]["status"],
            "REFERENCE_SUPERMODEL_EXPORT_ADMITTED"
        );
        assert_eq!(report["retailPayloadCopied"], false);
        assert!(!product.model_bytes.is_empty());
        assert!(!product.hak_bytes.is_empty());
        eprintln!(
            "{}",
            serde_json::json!({
                "selectedSupermodelResref": selected,
                "referenceSha256": preview_report["referenceSha256"],
                "modelSha256": preview_report["modelSha256"],
                "carrierNodeCount": preview_report["rigAnalysis"]["carrierNodeCount"],
                "activeWeightedBoneCount": preview_report["rigAnalysis"]["activeWeightedBoneCount"],
                "requiredClipCount": preview_report["requiredClipCount"],
                "jointClipRequiredCount": preview_report["motionQuality"]["jointClipRequiredCount"],
                "jointClipPassCount": preview_report["motionQuality"]["jointClipPassCount"],
                "seamViolationCount": preview_report["seamViolationCount"],
                "motionQualityStatus": preview_report["motionQualityStatus"],
                "exportAdmission": report["exportAdmission"]["status"],
            })
        );
    }

    #[test]
    fn ascii_supermodel_completes_analysis_preview_and_product_offline() {
        let selected = "c_ascii_e2e";
        let ascii = br#"newmodel c_ascii_e2e
setsupermodel c_ascii_e2e NULL
classification CHARACTER
setanimationscale 1
beginmodelgeom c_ascii_e2e
node dummy c_ascii_e2e
parent NULL
position 0 0 0
endnode
node dummy branch_l
parent c_ascii_e2e
position -0.5 0 0.7
endnode
node dummy branch_r
parent c_ascii_e2e
position 0.5 0 0.7
endnode
endmodelgeom c_ascii_e2e
newanim cpause1 c_ascii_e2e
length 1
animroot c_ascii_e2e
node dummy c_ascii_e2e
parent NULL
positionkey 2
0 0 0 0
1 0 0.05 0
endnode
node dummy branch_l
parent c_ascii_e2e
positionkey 2
0 -0.5 0 0.7
1 -0.495 0 0.7
endnode
node dummy branch_r
parent c_ascii_e2e
positionkey 2
0 0.5 0 0.7
1 0.505 0 0.7
endnode
doneanim cpause1 c_ascii_e2e
newanim cwalk c_ascii_e2e
length 1
animroot c_ascii_e2e
node dummy c_ascii_e2e
parent NULL
positionkey 2
0 0 0 0
1 0 0.2 0
endnode
node dummy branch_l
parent c_ascii_e2e
positionkey 2
0 -0.5 0 0.7
1 -0.49 0 0.7
endnode
node dummy branch_r
parent c_ascii_e2e
positionkey 2
0 0.5 0 0.7
1 0.51 0 0.7
endnode
doneanim cwalk c_ascii_e2e
newanim crun c_ascii_e2e
length 0.8
animroot c_ascii_e2e
node dummy c_ascii_e2e
parent NULL
positionkey 2
0 0 0 0
0.8 0 0.35 0
endnode
node dummy branch_l
parent c_ascii_e2e
positionkey 2
0 -0.5 0 0.7
0.8 -0.485 0 0.7
endnode
node dummy branch_r
parent c_ascii_e2e
positionkey 2
0 0.5 0 0.7
0.8 0.515 0 0.7
endnode
doneanim crun c_ascii_e2e
donemodel c_ascii_e2e
"#;
        let sha256 = format!("{:x}", Sha256::digest(ascii));
        let chain_json = serde_json::to_string(&vec![serde_json::json!({
            "resref": selected,
            "supermodelResref": "NULL",
            "format": "ASCII",
            "sha256": sha256,
            "byteOffset": 0,
            "byteLength": ascii.len()
        })])
        .unwrap();
        let source = disconnected_static_textured_owned_glb();
        let preview = build_reference_supermodel_applied_preview_v2_inner(
            selected,
            &source,
            ascii,
            &chain_json,
            "POSITIVE_Z",
        )
        .expect("apply structural ASCII supermodel");
        let preview_report: serde_json::Value =
            serde_json::from_str(&preview.apply_report_json).unwrap();
        assert_eq!(preview_report["referenceFormat"], "ASCII");
        assert_eq!(
            preview_report["motionQualityStatus"], "PASS",
            "ASCII full-skeleton report: {}",
            preview.apply_report_json
        );
        assert_eq!(
            preview_report["structuralAnalysis"]["inheritedAnimationNames"],
            serde_json::json!(["cpause1", "cwalk", "crun"])
        );

        let target_rig: serde_json::Value = serde_json::from_str(&preview.target_rig_json).unwrap();
        let mut authoring: serde_json::Value =
            serde_json::from_str(&preview.authoring_json).unwrap();
        let authored_node = &target_rig["nodes"][0];
        authoring["jointOverrides"] = serde_json::json!([{
            "carrierPartNumber": authored_node["id"],
            "bindLocalMatrix": authored_node["bindLocalMatrix"],
            "semanticRole": "rig_root",
            "jointAxis": [0.0, 1.0, 0.0],
            "locked": false
        }]);
        let authored_preview = build_reference_supermodel_applied_preview_with_authoring_v1_inner(
            selected,
            &source,
            ascii,
            &chain_json,
            "POSITIVE_Z",
            Some(&serde_json::to_string(&authoring).unwrap()),
        )
        .expect("seal and rebuild an authored target rig preview");
        let authored_report: serde_json::Value =
            serde_json::from_str(&authored_preview.apply_report_json).unwrap();
        assert_eq!(authored_report["rigAuthoring"]["jointOverrideCount"], 1);
        assert_eq!(
            authored_report["rigAuthoring"]["carrierTopologyPreserved"],
            true
        );

        let identity_json = serde_json::to_string(&serde_json::json!({
            "modelResref": "m2aasciiprod",
            "textureResref": "m2aasciitex",
            "materialResref": "m2aasciimtr",
            "hakResref": "m2aasciihak",
            "appearanceLabel": "M2A_ASCII_REFERENCE_PRODUCT",
            "appearanceDonorResrefs": [selected],
            "semanticControllerNames": []
        }))
        .unwrap();
        let appearance = format!("2DA V2.0\n\nLABEL MODELTYPE RACE\n0 donor S {selected}\n");
        let product = build_reference_supermodel_creature_product_with_authoring_v3_inner(
            selected,
            &source,
            appearance.as_bytes(),
            ascii,
            &chain_json,
            &identity_json,
            "POSITIVE_Z",
            Some(&authored_preview.authoring_json),
        )
        .expect("package the sealed authored ASCII supermodel Creature product");
        let report: serde_json::Value = serde_json::from_str(&product.report_json).unwrap();
        assert_eq!(report["selectedFormat"], "ASCII");
        assert_eq!(report["rigAuthoring"]["jointOverrideCount"], 1);
        assert_eq!(
            report["rigAuthoring"]["authoringSha256"],
            serde_json::from_str::<serde_json::Value>(&authored_preview.authoring_json).unwrap()["contentSha256"]
        );
        assert_eq!(
            report["exportAdmission"]["status"],
            "REFERENCE_SUPERMODEL_EXPORT_ADMITTED"
        );
        assert!(!product.model_bytes.is_empty());
        assert!(!product.hak_bytes.is_empty());
    }

    fn translated_y(y: f32) -> [f32; 16] {
        let mut matrix = identity();
        matrix[13] = y;
        matrix
    }

    fn exact_reference_fixture() -> m2a_core::BinaryMdlArtifactV1 {
        write_binary_mdl_with_animations(
            &AuroraModelIrV1 {
                schema_version: 1,
                profile_id: "wasm-exact-supermodel-reference".to_owned(),
                source_sha256: "0".repeat(64),
                basis_status: "SYNTHETIC".to_owned(),
                engine_facing_proof: "SYNTHETIC".to_owned(),
                uv_runtime_proof: "SYNTHETIC".to_owned(),
                nodes: vec![
                    AuroraModelNodeV1 {
                        id: 1,
                        name: "c_exact".to_owned(),
                        parent_id: None,
                        bind_local_matrix: identity(),
                    },
                    AuroraModelNodeV1 {
                        id: 2,
                        name: "tail".to_owned(),
                        parent_id: Some(1),
                        bind_local_matrix: translated_y(1.25),
                    },
                ],
                material_source_bindings: Vec::new(),
                segments: vec![AuroraModelSegmentV1 {
                    segment_id: 1,
                    material_slot: 0,
                    deformation: AuroraSegmentDeformationV1::Rigid,
                    parent_node_id: 1,
                    cast_shadow: true,
                    positions: vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]],
                    normals: vec![[0.0, 0.0, 1.0]; 3],
                    tangents: None,
                    uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                    indices: vec![0, 1, 2],
                    face_surface_ids: Vec::new(),
                    weights: Vec::new(),
                }],
            },
            &MdlAnimationSetV1 {
                schema_version: 1,
                clips: vec![MdlAnimationClipV1 {
                    name: "cpause1".to_owned(),
                    animation_root: "c_exact".to_owned(),
                    length_seconds: 1.0,
                    transition_seconds: 0.25,
                    events: Vec::new(),
                    tracks: vec![MdlAnimationTrackV1 {
                        target_node_id: 2,
                        path: MdlAnimationTrackPathV1::Translation,
                        interpolation: MdlAnimationInterpolationV1::Linear,
                        times_seconds: vec![0.0, 1.0],
                        values: vec![vec![0.0, 1.25, 0.0], vec![0.25, 1.25, 0.0]],
                    }],
                }],
            },
            &MdlWriterOptionsV1 {
                schema_version: 1,
                format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
                state_projection_profile:
                    MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
                state_projection_provenance: None,
                model_resource_resref: "c_exact".to_owned(),
                diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                    material_slot: 0,
                    resref: "cexacttex".to_owned(),
                }],
            },
        )
        .unwrap()
    }

    #[test]
    fn exact_contract_boundary_is_generic_and_matches_core_json() {
        let reference = exact_reference_fixture();
        let options = ReferenceSupermodelExactContractOptionsV3 {
            contract_id: "wasm-exact-supermodel-v3".to_owned(),
            supermodel_resref: "c_exact".to_owned(),
            source_model_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_owned(),
            required_clips: vec!["cpause1".to_owned()],
            required_events: Vec::new(),
            semantic_nodes: vec![ReferenceSupermodelSemanticNodeV3 {
                node_name: "tail".to_owned(),
                anchor_role: Some("tail_tip".to_owned()),
                joint_axis: Some([0.0, 0.0, 1.0]),
            }],
            tolerances: default_reference_supermodel_motion_tolerances_v2(),
        };
        let expected =
            build_exact_reference_supermodel_motion_contract_v3(&reference.inspection, &options)
                .unwrap();
        let actual = build_exact_reference_supermodel_motion_contract_v3_json_inner(
            &reference.payload,
            &serde_json::to_string(&options).unwrap(),
        )
        .unwrap();

        assert_eq!(actual, serde_json::to_string(&expected).unwrap());
        assert_eq!(expected.nodes.len(), 2);
        assert_eq!(
            expected.nodes[1].carrier_bind_local_matrix,
            translated_y(1.25)
        );
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileAJsonInputError<'a> {
    schema_version: u32,
    code: &'a str,
    severity: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MdlWriterJsonInputError<'a> {
    schema_version: u32,
    code: &'a str,
    severity: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct M5BoundaryJsonError<'a> {
    schema_version: u32,
    code: &'a str,
    severity: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HakResourceDescriptorV1 {
    resref: String,
    resource_type: u16,
    payload_offset: u32,
    payload_size: u32,
}

impl m2a_core::hak::HakResourceMetadataV1 for HakResourceDescriptorV1 {
    fn hak_resref(&self) -> &str {
        &self.resref
    }

    fn hak_resource_type(&self) -> u16 {
        self.resource_type
    }

    fn hak_payload_size(&self) -> Option<u64> {
        Some(u64::from(self.payload_size))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HakResourceDescriptorsV1 {
    schema_version: u32,
    resources: Vec<HakResourceDescriptorV1>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(
    tag = "role",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum M7PayloadDescriptorV1 {
    Source {
        relative_path: String,
        payload_offset: u32,
        payload_size: u32,
    },
    #[serde(rename = "RIGGED_HUMANOID_APPEARANCE_2DA")]
    RiggedHumanoidAppearance2da {
        sample_id: String,
        payload_offset: u32,
        payload_size: u32,
    },
}

impl M7PayloadDescriptorV1 {
    fn offset(&self) -> u32 {
        match self {
            Self::Source { payload_offset, .. }
            | Self::RiggedHumanoidAppearance2da { payload_offset, .. } => *payload_offset,
        }
    }

    fn size(&self) -> u32 {
        match self {
            Self::Source { payload_size, .. }
            | Self::RiggedHumanoidAppearance2da { payload_size, .. } => *payload_size,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct M7PayloadDescriptorsV1 {
    schema_version: u32,
    payloads: Vec<M7PayloadDescriptorV1>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct M7BoundaryErrorV1<'a> {
    schema_version: u32,
    code: &'a str,
    path: String,
    message: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaceableBoundaryErrorV1<'a> {
    schema_version: u32,
    code: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaceableTextureArtifactDescriptorV1 {
    schema_version: u32,
    resref: String,
    resource_type: u16,
    byte_offset: u64,
    byte_length: u64,
    sha256: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StaticTileBoundaryOptionsV1 {
    schema_version: u32,
    identity: m2a_core::tile::StaticTileIdentityV1,
    interior: bool,
    terrain_name: String,
    surface: m2a_core::tile::TileSurfaceV1,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TileBoundaryErrorV1<'a> {
    schema_version: u32,
    code: &'a str,
    path: &'a str,
    message: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelMaterialBoundaryErrorV1 {
    schema_version: u32,
    code: String,
    path: String,
    message: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelComponentBoundaryOutputV1 {
    schema_version: u32,
    capabilities: m2a_core::model_material_capabilities::ModelMaterialCapabilitiesV1,
    inventory: m2a_core::model_components::ModelComponentInventoryV1,
    document: m2a_core::model_material_separation::ModelMaterialSeparationDocumentV1,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelMaterialResolutionBoundaryOutputV1 {
    schema_version: u32,
    capabilities: m2a_core::model_material_capabilities::ModelMaterialCapabilitiesV1,
    report: m2a_core::model_material_separation::ModelMaterialSeparationReportV1,
    texture_authoring: m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelFaceBoundaryOutputV2 {
    schema_version: u32,
    capabilities: m2a_core::model_material_capabilities::ModelMaterialCapabilitiesV1,
    inventory: m2a_core::model_components::ModelComponentInventoryV1,
    document: m2a_core::model_material_separation::ModelMaterialSeparationDocumentV2,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelMaterialResolutionBoundaryOutputV2 {
    schema_version: u32,
    capabilities: m2a_core::model_material_capabilities::ModelMaterialCapabilitiesV1,
    report: m2a_core::model_material_separation::ModelMaterialSeparationReportV2,
    texture_authoring: m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
}

fn model_material_boundary_error(
    code: &str,
    path: &str,
    message: impl Into<String>,
) -> ModelMaterialBoundaryErrorV1 {
    ModelMaterialBoundaryErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
    }
}

fn model_material_boundary_error_v2(
    code: &str,
    path: &str,
    message: impl Into<String>,
) -> ModelMaterialBoundaryErrorV1 {
    ModelMaterialBoundaryErrorV1 {
        schema_version: 2,
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
    }
}

fn parse_model_render_target_v1(
    target: &str,
) -> Result<m2a_core::model_material_capabilities::ModelRenderTargetV1, ModelMaterialBoundaryErrorV1>
{
    use m2a_core::model_material_capabilities::ModelRenderTargetV1;
    match target {
        "CREATURE" => Ok(ModelRenderTargetV1::Creature),
        "PLACEABLE" => Ok(ModelRenderTargetV1::Placeable),
        "TILE" => Ok(ModelRenderTargetV1::Tile),
        "MODEL_PART" => Ok(ModelRenderTargetV1::ModelPart),
        _ => Err(model_material_boundary_error(
            "MODEL-MATERIAL-TARGET-INVALID",
            "target",
            "target must be CREATURE, PLACEABLE, TILE or MODEL_PART",
        )),
    }
}

fn parse_aurora_material_profile_v1(
    profile: &str,
) -> Result<m2a_core::aurora_material::AuroraMaterialTargetProfileV1, ModelMaterialBoundaryErrorV1>
{
    use m2a_core::aurora_material::AuroraMaterialTargetProfileV1;
    match profile {
        "AURORA_CLASSIC_SAFE" => Ok(AuroraMaterialTargetProfileV1::AuroraClassicSafe),
        "NWN_EE_MTR" => Ok(AuroraMaterialTargetProfileV1::NwnEeMtr),
        _ => Err(model_material_boundary_error(
            "AURORA-MATERIAL-PROFILE-INVALID",
            "profile",
            "profile must be AURORA_CLASSIC_SAFE or NWN_EE_MTR",
        )),
    }
}

fn compile_aurora_materials_v1_json_inner(bytes: &[u8], profile: &str) -> Result<String, String> {
    let profile =
        parse_aurora_material_profile_v1(profile).map_err(|error| serialize_json(&error))?;
    let ingest = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let compiled = m2a_core::aurora_material::compile_gltf_materials_v1(
        &ingest.ir.source.sha256,
        &ingest.ir.materials,
        profile,
    )
    .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&compiled))
}

/// Returns the source-bound Aurora material plan and complete fidelity ledger.
#[wasm_bindgen(js_name = compileAuroraMaterialsV1Json)]
pub fn compile_aurora_materials_v1_json(bytes: &[u8], profile: &str) -> Result<String, JsValue> {
    compile_aurora_materials_v1_json_inner(bytes, profile)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod aurora_material_boundary_tests {
    use super::compile_aurora_materials_v1_json_inner;

    #[test]
    fn material_compiler_boundary_is_source_bound_and_rejects_unknown_profiles() {
        let glb = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let json = compile_aurora_materials_v1_json_inner(&glb, "AURORA_CLASSIC_SAFE").unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["sourceSha256"].as_str().unwrap().len(), 64);
        assert_eq!(value["targetProfile"], "AURORA_CLASSIC_SAFE");
        assert!(
            value["materials"]
                .as_array()
                .is_some_and(|items| !items.is_empty())
        );

        let error = compile_aurora_materials_v1_json_inner(&glb, "MAGIC").unwrap_err();
        let value: serde_json::Value = serde_json::from_str(&error).unwrap();
        assert_eq!(value["code"], "AURORA-MATERIAL-PROFILE-INVALID");
    }
}

fn inspect_model_components_v1_json_inner(bytes: &[u8], target: &str) -> Result<String, String> {
    let target = parse_model_render_target_v1(target).map_err(|error| serialize_json(&error))?;
    let ingest = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let inventory = m2a_core::model_components::inspect_model_components_v1(&ingest.ir)
        .map_err(|error| serialize_json(&error))?;
    let document =
        m2a_core::model_material_separation::default_model_material_separation_v1(&ingest.ir);
    Ok(serialize_json(&ModelComponentBoundaryOutputV1 {
        schema_version: 1,
        capabilities: m2a_core::model_material_capabilities::material_separation_capabilities_v1(
            target,
        ),
        inventory,
        document,
    }))
}

/// Returns compact connected-component inspection and target capabilities.
/// The per-triangle material map intentionally never crosses the WASM boundary.
#[wasm_bindgen(js_name = inspectModelComponentsV1Json)]
pub fn inspect_model_components_v1_json(bytes: &[u8], target: &str) -> Result<String, JsValue> {
    inspect_model_components_v1_json_inner(bytes, target).map_err(|error| JsValue::from_str(&error))
}

fn resolve_model_materials_v1_json_inner(
    bytes: &[u8],
    target: &str,
    document_json: &str,
) -> Result<String, String> {
    let target = parse_model_render_target_v1(target).map_err(|error| serialize_json(&error))?;
    let ingest = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let document = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV1,
    >(document_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "documentJson",
            "material separation document must satisfy the strict V1 schema",
        ))
    })?;
    let resolved =
        m2a_core::model_material_separation::resolve_model_materials_v1(&ingest.ir, &document)
            .map_err(|error| serialize_json(&error))?;
    let capabilities =
        m2a_core::model_material_capabilities::validate_material_separation_counts_v1(
            target,
            resolved.report.material_slots.len(),
            resolved.report.output_section_count,
        )
        .map_err(|error| serialize_json(&error))?;
    let texture_authoring =
        m2a_core::model_texture_authoring::default_model_texture_authoring_v1(&ingest, &resolved)
            .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&ModelMaterialResolutionBoundaryOutputV1 {
        schema_version: 1,
        capabilities,
        report: resolved.report,
        texture_authoring,
    }))
}

/// Validates and resolves a target-neutral Material Separation recipe.
#[wasm_bindgen(js_name = resolveModelMaterialsV1Json)]
pub fn resolve_model_materials_v1_json(
    bytes: &[u8],
    target: &str,
    document_json: &str,
) -> Result<String, JsValue> {
    resolve_model_materials_v1_json_inner(bytes, target, document_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn inspect_model_faces_v2_json_inner(bytes: &[u8], target: &str) -> Result<String, String> {
    let target = parse_model_render_target_v1(target).map_err(|error| serialize_json(&error))?;
    let ingest = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let inventory = m2a_core::model_components::inspect_model_components_v1(&ingest.ir)
        .map_err(|error| serialize_json(&error))?;
    let document =
        m2a_core::model_material_separation::default_model_material_separation_v2(&ingest.ir);
    Ok(serialize_json(&ModelFaceBoundaryOutputV2 {
        schema_version: 2,
        capabilities: m2a_core::model_material_capabilities::material_separation_capabilities_v2(
            target,
        ),
        inventory,
        document,
    }))
}

/// Returns Face Mode V2 bootstrap data. Exact face ordinals stay attached to
/// the source GLB and are selected in the viewport; no per-triangle payload is
/// copied over the WASM boundary.
#[wasm_bindgen(js_name = inspectModelFacesV2Json)]
pub fn inspect_model_faces_v2_json(bytes: &[u8], target: &str) -> Result<String, JsValue> {
    inspect_model_faces_v2_json_inner(bytes, target).map_err(|error| JsValue::from_str(&error))
}

fn resolve_model_materials_v2_json_inner(
    bytes: &[u8],
    target: &str,
    document_json: &str,
) -> Result<String, String> {
    let target = parse_model_render_target_v1(target).map_err(|error| serialize_json(&error))?;
    let ingest = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let document = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV2,
    >(document_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "documentJson",
            "material separation document must satisfy the strict Face Mode V2 schema",
        ))
    })?;
    let resolved =
        m2a_core::model_material_separation::resolve_model_materials_v2(&ingest.ir, &document)
            .map_err(|error| serialize_json(&error))?;
    let capabilities =
        m2a_core::model_material_capabilities::validate_material_separation_counts_v2(
            target,
            resolved.report.material_slots.len(),
            resolved.report.output_section_count,
        )
        .map_err(|error| serialize_json(&error))?;
    let texture_authoring = m2a_core::model_texture_authoring::default_model_texture_authoring_v1(
        &ingest,
        resolved.projection_v1(),
    )
    .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&ModelMaterialResolutionBoundaryOutputV2 {
        schema_version: 2,
        capabilities,
        report: resolved.report,
        texture_authoring,
    }))
}

/// Validates and resolves an exact Face Mode V2 recipe.
#[wasm_bindgen(js_name = resolveModelMaterialsV2Json)]
pub fn resolve_model_materials_v2_json(
    bytes: &[u8],
    target: &str,
    document_json: &str,
) -> Result<String, JsValue> {
    resolve_model_materials_v2_json_inner(bytes, target, document_json)
        .map_err(|error| JsValue::from_str(&error))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct M7BatchBoundaryOutputV1 {
    schema_version: u32,
    report: m2a_core::m7_corpus::M7CorpusBatchReportV1,
    packets: Vec<m2a_core::m7_corpus::M7PerProfileProofPacketV1>,
}

const SERIALIZATION_ERROR_JSON: &str = concat!(
    r#"{"schemaVersion":1,"code":"M2A-JSON-SERIALIZATION","severity":"error","offset":0,"context":""#,
    "WASM adapter JSON serialization",
    r#""}"#,
);

/// Inspects an Aurora/NWN binary MDL selected by JavaScript.
///
/// `wasm-bindgen` maps the borrowed byte slice to a JavaScript `Uint8Array`.
/// The adapter deliberately owns only the JS/WASM boundary and JSON encoding;
/// all format parsing remains in `m2a-core`.
#[wasm_bindgen(js_name = inspectBinaryMdl)]
pub fn inspect_binary_mdl(bytes: &[u8]) -> String {
    let result = m2a_core::inspect_binary_mdl(bytes);

    match result {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Builds the read-only MDL locator inventory from one user-selected KEY V1 file.
#[wasm_bindgen(js_name = indexNwnKeyModelsV1Json)]
pub fn index_nwn_key_models_v1_json(bytes: &[u8]) -> String {
    match m2a_core::supermodel_catalog::index_nwn_key_models_v1(bytes) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Catalogues MDL entries inside one user-selected HAK without extracting them.
#[wasm_bindgen(js_name = indexHakModelsV1Json)]
pub fn index_hak_models_v1_json(bytes: &[u8]) -> String {
    match m2a_core::supermodel_catalog::index_hak_models_v1(bytes) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Returns the exact variable-resource table range required from a BIFF V1 file.
#[wasm_bindgen(js_name = planNwnBifIndexV1Json)]
pub fn plan_nwn_bif_index_v1_json(header: &[u8]) -> String {
    match m2a_core::supermodel_catalog::plan_nwn_bif_index_v1(header) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Parses a BIFF V1 header and the exact table slice requested by the plan adapter.
#[wasm_bindgen(js_name = indexNwnBifTableV1Json)]
pub fn index_nwn_bif_table_v1_json(header: &[u8], table: &[u8]) -> String {
    match m2a_core::supermodel_catalog::index_nwn_bif_table_v1(header, table) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MdlCatalogHeaderBatchDescriptorV1 {
    item_id: String,
    byte_offset: usize,
    byte_length: usize,
    declared_payload_size: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MdlCatalogHeaderBatchItemV1 {
    item_id: String,
    header: Option<m2a_core::supermodel_catalog::MdlCatalogHeaderV1>,
    error: Option<m2a_core::supermodel_catalog::SupermodelCatalogErrorV1>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MdlCatalogHeaderBatchV1 {
    schema_version: u32,
    items: Vec<MdlCatalogHeaderBatchItemV1>,
}

/// Inspects a bounded batch of independently sliced MDL metadata prefixes.
#[wasm_bindgen(js_name = inspectMdlCatalogHeadersV1Json)]
pub fn inspect_mdl_catalog_headers_v1_json(blob: &[u8], descriptors_json: &str) -> String {
    let descriptors =
        match serde_json::from_str::<Vec<MdlCatalogHeaderBatchDescriptorV1>>(descriptors_json) {
            Ok(value) if value.len() <= 1_024 => value,
            _ => {
                return serialize_json(&serde_json::json!({
                    "schemaVersion": 1,
                    "code": "M2A-SUPERMODEL-BATCH-DESCRIPTORS",
                    "offset": 0,
                    "context": "descriptor JSON must be a strict array with at most 1024 items"
                }));
            }
        };
    let items = descriptors
        .into_iter()
        .map(|descriptor| {
            let bytes = descriptor
                .byte_offset
                .checked_add(descriptor.byte_length)
                .and_then(|end| blob.get(descriptor.byte_offset..end));
            let result =
                bytes.ok_or_else(|| m2a_core::supermodel_catalog::SupermodelCatalogErrorV1 {
                    schema_version: 1,
                    code: "M2A-SUPERMODEL-BATCH-RANGE".to_owned(),
                    offset: descriptor.byte_offset,
                    context: "descriptor escapes the supplied batch blob".to_owned(),
                });
            match result.and_then(|bytes| {
                m2a_core::supermodel_catalog::inspect_mdl_catalog_header_v1(
                    bytes,
                    descriptor.declared_payload_size,
                )
            }) {
                Ok(header) => MdlCatalogHeaderBatchItemV1 {
                    item_id: descriptor.item_id,
                    header: Some(header),
                    error: None,
                },
                Err(error) => MdlCatalogHeaderBatchItemV1 {
                    item_id: descriptor.item_id,
                    header: None,
                    error: Some(error),
                },
            }
        })
        .collect();
    serialize_json(&MdlCatalogHeaderBatchV1 {
        schema_version: 1,
        items,
    })
}

/// Builds the deterministic case-insensitive supermodel graph from scanned metadata.
#[wasm_bindgen(js_name = buildSupermodelCatalogV1Json)]
pub fn build_supermodel_catalog_v1_json(input_json: &str) -> String {
    let input = match serde_json::from_str::<
        m2a_core::supermodel_catalog::SupermodelCatalogBuildInputV1,
    >(input_json)
    {
        Ok(value) => value,
        Err(_) => {
            return serialize_json(&serde_json::json!({
                "schemaVersion": 1,
                "code": "M2A-SUPERMODEL-CATALOG-INPUT",
                "offset": 0,
                "context": "catalog input JSON does not match the strict V1 schema"
            }));
        }
    };
    serialize_json(&m2a_core::supermodel_catalog::build_supermodel_catalog_v1(
        &input,
    ))
}

#[cfg(test)]
mod supermodel_catalog_wasm_boundary_tests {
    use super::{build_supermodel_catalog_v1_json, inspect_mdl_catalog_headers_v1_json};

    #[test]
    fn invalid_catalog_json_and_batch_ranges_return_stable_json_errors() {
        let invalid = build_supermodel_catalog_v1_json("{}");
        assert!(invalid.contains("M2A-SUPERMODEL-CATALOG-INPUT"));

        let report = inspect_mdl_catalog_headers_v1_json(
            &[0; 8],
            r#"[{"itemId":"one","byteOffset":7,"byteLength":2,"declaredPayloadSize":2}]"#,
        );
        assert!(report.contains("M2A-SUPERMODEL-BATCH-RANGE"));
    }
}

/// Inspects a GLB selected by JavaScript with the default project guardrails.
///
/// This adapter owns only the JS/WASM boundary and JSON encoding. GLB parsing,
/// validation, gates and diagnostics remain exclusively in `m2a-core`.
#[wasm_bindgen(js_name = inspectGlbJson)]
pub fn inspect_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::inspect_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(report) => serialize_json(&report),
        Err(error) => serialize_json(&error),
    }
}

/// Backward-compatible alias for the original M2 WASM export.
#[wasm_bindgen(js_name = inspectGlb)]
pub fn inspect_glb(bytes: &[u8]) -> String {
    inspect_glb_json(bytes)
}

/// Ingests a GLB selected by JavaScript into source-preserving AuroraAssetIR.
///
/// The adapter deliberately delegates the complete operation to `m2a-core`.
#[wasm_bindgen(js_name = ingestGlbJson)]
pub fn ingest_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

/// Produces a compact, diagnostic-only inspection for unusually large GLBs.
/// The core result is always conversion-ineligible and cannot be used by any
/// build or package export route.
#[wasm_bindgen(js_name = inspectHighPolyGlbJson)]
pub fn inspect_high_poly_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::inspect_high_poly_glb_v1(bytes) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

/// Ingests a static placeable or tile GLB with the shared native single-mesh
/// triangle envelope. The parser and AuroraAssetIR stay identical to the
/// creature route; only target-specific admission diagnostics differ.
#[wasm_bindgen(js_name = ingestStaticRigidGlbJson)]
pub fn ingest_static_rigid_glb_json(bytes: &[u8]) -> String {
    match m2a_core::glb::ingest_glb(
        bytes,
        &m2a_core::placeable::static_placeable_glb_limits_v1(),
    ) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

/// Inspects the historical P100K Creature compatibility profile with its exact
/// bounded envelope owned by `m2a-core`.
#[wasm_bindgen(js_name = ingestMeshyP100kExperimentJson)]
pub fn ingest_meshy_p100k_experiment_json(bytes: &[u8]) -> String {
    match m2a_core::model_pipeline::inspect_meshy_procedural_humanoid_p100k_experiment_v1(bytes) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

/// Inspects the historical P300K Creature compatibility profile. Default
/// product inspection now uses the shared 300K budget.
#[wasm_bindgen(js_name = ingestMeshyP300kExperimentJson)]
pub fn ingest_meshy_p300k_experiment_json(bytes: &[u8]) -> String {
    match m2a_core::model_pipeline::inspect_meshy_procedural_humanoid_p300k_experiment_v1(bytes) {
        Ok(result) => serialize_json(&result),
        Err(error) => serialize_json(&error),
    }
}

/// Backward-compatible alias for the original M2 WASM export.
#[wasm_bindgen(js_name = ingestGlb)]
pub fn ingest_glb(bytes: &[u8]) -> String {
    ingest_glb_json(bytes)
}

/// Converts a Meshy-style GLB with an explicit clean-room Profile A rig.
///
/// The JSON boundary is strict (`deny_unknown_fields` is defined by the core
/// input types). GLB ingestion and every transformation remain in `m2a-core`;
/// this adapter only parses public JSON and serializes the deterministic core
/// outcome or fatal error.
#[wasm_bindgen(js_name = convertProfileAGlbJson)]
pub fn convert_profile_a_glb_json(bytes: &[u8], rig_json: &str, options_json: &str) -> String {
    let (rig, options) = match parse_profile_a_json(rig_json, options_json) {
        Ok(values) => values,
        Err(error) => return error,
    };
    let source = match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(value) => value,
        Err(error) => return serialize_json(&error),
    };
    match m2a_core::profile_a::convert_profile_a(&source, &rig, &options) {
        Ok(outcome) => serialize_json(&outcome),
        Err(error) => serialize_json(&error),
    }
}

/// Converts a Meshy-style animated GLB with explicit clean-room rig and
/// source-to-output animation mappings.
///
/// This boundary only performs strict JSON decoding and GLB ingestion. Mapping
/// validation, retargeting and all deterministic output generation are owned by
/// the public `m2a-core` M4A2 route.
#[wasm_bindgen(js_name = convertProfileAWithAnimationsGlbJson)]
pub fn convert_profile_a_with_animations_glb_json(
    bytes: &[u8],
    rig_json: &str,
    options_json: &str,
    mapping_json: &str,
) -> String {
    let (rig, options) = match parse_profile_a_json(rig_json, options_json) {
        Ok(values) => values,
        Err(error) => return error,
    };
    let mapping =
        match serde_json::from_str::<m2a_core::profile_a::ProfileAAnimationMappingV1>(mapping_json)
        {
            Ok(value) => value,
            Err(_) => {
                return serialize_json(&ProfileAJsonInputError {
                    schema_version: 1,
                    code: "M4A-MAPPING-JSON-INVALID",
                    severity: "FATAL",
                    path: "mappingJson",
                    message: "animation mapping JSON does not match the public schema",
                });
            }
        };
    let source = match m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default()) {
        Ok(value) => value,
        Err(error) => return serialize_json(&error),
    };
    match m2a_core::profile_a::convert_profile_a_with_animations_v1(
        &source, &rig, &options, &mapping,
    ) {
        Ok(outcome) => serialize_json(&outcome),
        Err(error) => serialize_json(&error),
    }
}

/// Concise alias retained for callers that adopted the initial M3 adapter name.
#[wasm_bindgen(js_name = convertProfileAJson)]
pub fn convert_profile_a_json(bytes: &[u8], rig_json: &str, options_json: &str) -> String {
    convert_profile_a_glb_json(bytes, rig_json, options_json)
}

fn parse_profile_a_json(
    rig_json: &str,
    options_json: &str,
) -> Result<
    (
        m2a_core::profile_a::CreatureRigProfileV1,
        m2a_core::profile_a::ProfileAOptionsV1,
    ),
    String,
> {
    let rig = serde_json::from_str(rig_json).map_err(|_| {
        serialize_json(&ProfileAJsonInputError {
            schema_version: 1,
            code: "M3A-PROFILE-JSON-INVALID",
            severity: "FATAL",
            path: "rigJson",
            message: "rig profile JSON does not match the public schema",
        })
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        serialize_json(&ProfileAJsonInputError {
            schema_version: 1,
            code: "M3A-OPTIONS-INVALID",
            severity: "FATAL",
            path: "optionsJson",
            message: "options JSON does not match the public schema",
        })
    })?;
    Ok((rig, options))
}

/// Writes an Aurora binary MDL from strict public JSON contracts.
///
/// The returned vector is mapped to a JavaScript `Uint8Array`. Failures are a
/// stable serialized JSON object carried by `JsValue`; no binary encoding or
/// validation is duplicated at the WASM boundary.
#[wasm_bindgen(js_name = writeBinaryMdlWithAnimations)]
pub fn write_binary_mdl_with_animations(
    creature_json: &str,
    animations_json: &str,
    options_json: &str,
) -> Result<Vec<u8>, JsValue> {
    let (creature, animations, options) =
        parse_mdl_writer_json(creature_json, animations_json, options_json)
            .map_err(|error| JsValue::from_str(&error))?;
    m2a_core::mdl::write_binary_mdl_with_animations(&creature, &animations, &options)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

/// Returns the deterministic core writer report or the same stable JSON error
/// used by `writeBinaryMdlWithAnimations`.
#[wasm_bindgen(js_name = writeBinaryMdlWithAnimationsReportJson)]
pub fn write_binary_mdl_with_animations_report_json(
    creature_json: &str,
    animations_json: &str,
    options_json: &str,
) -> String {
    let (creature, animations, options) =
        match parse_mdl_writer_json(creature_json, animations_json, options_json) {
            Ok(values) => values,
            Err(error) => return error,
        };
    match m2a_core::mdl::write_binary_mdl_with_animations(&creature, &animations, &options) {
        Ok(artifact) => serialize_json(&artifact.report),
        Err(error) => serialize_json(&error),
    }
}

fn parse_mdl_writer_json(
    creature_json: &str,
    animations_json: &str,
    options_json: &str,
) -> Result<
    (
        m2a_core::profile_a::AuroraCreatureIrV1,
        m2a_core::mdl::MdlAnimationSetV1,
        m2a_core::mdl::MdlWriterOptionsV1,
    ),
    String,
> {
    let creature = serde_json::from_str(creature_json).map_err(|_| {
        mdl_writer_json_error(
            "M4A-CREATURE-JSON-INVALID",
            "creatureJson",
            "creature JSON does not match the strict public schema",
        )
    })?;
    let animations = serde_json::from_str(animations_json).map_err(|_| {
        mdl_writer_json_error(
            "M4A-ANIMATION-JSON-INVALID",
            "animationsJson",
            "animation set JSON does not match the strict public schema",
        )
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        mdl_writer_json_error(
            "M4A-OPTIONS-JSON-INVALID",
            "optionsJson",
            "writer options JSON does not match the strict public schema",
        )
    })?;
    Ok((creature, animations, options))
}

fn mdl_writer_json_error(code: &str, path: &str, message: &str) -> String {
    serialize_json(&MdlWriterJsonInputError {
        schema_version: 1,
        code,
        severity: "FATAL",
        path,
        message,
    })
}

fn serialize_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| SERIALIZATION_ERROR_JSON.to_owned())
}

fn m5_boundary_error(code: &str, path: &str, message: &str) -> String {
    m5_error_json(code, "FATAL", path, message)
}

fn m5_error_json(code: &str, severity: &str, path: &str, message: &str) -> String {
    serialize_json(&M5BoundaryJsonError {
        schema_version: 1,
        code,
        severity,
        path,
        message,
    })
}

fn two_da_core_error_json(error: &m2a_core::two_da::TwoDaError) -> String {
    m5_error_json(&error.code, &error.severity, &error.path, &error.message)
}

fn parse_tga_json(
    image_json: &str,
    options_json: &str,
) -> Result<(m2a_core::tga::TgaImageV1, m2a_core::tga::TgaWriterOptionsV1), String> {
    let image = serde_json::from_str(image_json).map_err(|_| {
        m5_boundary_error(
            "M5-TGA-IMAGE-JSON-INVALID",
            "imageJson",
            "image JSON does not match the strict public schema",
        )
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        m5_boundary_error(
            "M5-TGA-OPTIONS-JSON-INVALID",
            "optionsJson",
            "TGA options JSON does not match the strict public schema",
        )
    })?;
    Ok((image, options))
}

fn write_tga_artifact_json(
    image_json: &str,
    options_json: &str,
) -> Result<m2a_core::tga::TgaArtifactV1, String> {
    let (image, options) = parse_tga_json(image_json, options_json)?;
    m2a_core::tga::write_tga_v1(&image, &options).map_err(|error| serialize_json(&error))
}

/// Writes a deterministic TGA from strict image/options JSON.
#[wasm_bindgen(js_name = writeTgaV1)]
pub fn write_tga_v1(image_json: &str, options_json: &str) -> Result<Vec<u8>, JsValue> {
    write_tga_artifact_json(image_json, options_json)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the report for the same deterministic TGA core operation.
#[wasm_bindgen(js_name = writeTgaV1ReportJson)]
pub fn write_tga_v1_report_json(image_json: &str, options_json: &str) -> Result<String, JsValue> {
    write_tga_artifact_json(image_json, options_json)
        .map(|artifact| serialize_json(&artifact.report))
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_two_da_limits_json(limits_json: &str) -> Result<m2a_core::two_da::TwoDaLimitsV1, String> {
    serde_json::from_str(limits_json).map_err(|_| {
        m5_boundary_error(
            "M5-2DA-LIMITS-JSON-INVALID",
            "limitsJson",
            "2DA limits JSON does not match the strict public schema",
        )
    })
}

fn inspect_two_da_v2_json_inner(bytes: &[u8], limits_json: &str) -> Result<String, String> {
    let limits = parse_two_da_limits_json(limits_json)?;
    m2a_core::two_da::inspect_two_da_v2(bytes, &limits)
        .map(|inspection| serialize_json(&inspection))
        .map_err(|error| two_da_core_error_json(&error))
}

/// Inspects strict 2DA V2.0 bytes with caller-supplied strict limits JSON.
#[wasm_bindgen(js_name = inspectTwoDaV2Json)]
pub fn inspect_two_da_v2_json(bytes: &[u8], limits_json: &str) -> Result<String, JsValue> {
    inspect_two_da_v2_json_inner(bytes, limits_json).map_err(|error| JsValue::from_str(&error))
}

fn resolve_item_base_record_v1_json_inner(
    bytes: &[u8],
    physical_row_index: u32,
    expected_source_sha256: &str,
    limits_json: &str,
) -> Result<String, String> {
    let limits = parse_two_da_limits_json(limits_json)?;
    m2a_core::item::resolve_item_base_record_v1(
        bytes,
        physical_row_index,
        expected_source_sha256,
        &limits,
    )
    .map(|record| serialize_json(&record))
    .map_err(|error| serialize_json(&error))
}

/// Resolves one exact BaseItem row from a hash-bound `baseitems.2da` input.
#[wasm_bindgen(js_name = resolveItemBaseRecordV1Json)]
pub fn resolve_item_base_record_v1_json(
    bytes: &[u8],
    physical_row_index: u32,
    expected_source_sha256: &str,
    limits_json: &str,
) -> Result<String, JsValue> {
    resolve_item_base_record_v1_json_inner(
        bytes,
        physical_row_index,
        expected_source_sha256,
        limits_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemRecipeResolutionBoundaryV1 {
    schema_version: u32,
    recipe_sha256: String,
    required_slots: Vec<m2a_core::item::ItemPartSlotV1>,
    resource_names: Vec<m2a_core::item::ItemResourceNameV1>,
}

fn resolve_item_recipe_v1_json_inner(recipe_json: &str) -> Result<String, String> {
    let recipe: m2a_core::item::ItemAppearanceRecipeV1 = serde_json::from_str(recipe_json)
        .map_err(|_| {
            m5_boundary_error(
                "M2A-ITEM-RECIPE-JSON-INVALID",
                "recipeJson",
                "Item recipe JSON does not match the strict public schema",
            )
        })?;
    m2a_core::item::validate_item_recipe_v1(&recipe).map_err(|error| serialize_json(&error))?;
    let recipe_sha256 =
        m2a_core::item::item_recipe_sha256_v1(&recipe).map_err(|error| serialize_json(&error))?;
    let resource_names = m2a_core::item::resolve_item_resource_names_v1(&recipe)
        .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&ItemRecipeResolutionBoundaryV1 {
        schema_version: 1,
        recipe_sha256,
        required_slots: m2a_core::item::required_item_part_slots_v1(recipe.base_item.profile)
            .to_vec(),
        resource_names,
    }))
}

/// Validates and hashes one strict Item recipe, then resolves its proven names.
#[wasm_bindgen(js_name = resolveItemRecipeV1Json)]
pub fn resolve_item_recipe_v1_json(recipe_json: &str) -> Result<String, JsValue> {
    resolve_item_recipe_v1_json_inner(recipe_json).map_err(|error| JsValue::from_str(&error))
}

fn compile_item_part_v1_inner(
    request_json: &str,
) -> Result<m2a_core::item_part::ItemPartArtifactV1, String> {
    let request: m2a_core::item_part::ItemPartCompileRequestV1 = serde_json::from_str(request_json)
        .map_err(|_| {
            m5_boundary_error(
                "M2A-ITEM-PART-REQUEST-JSON-INVALID",
                "requestJson",
                "Item part request JSON does not match the strict public schema",
            )
        })?;
    m2a_core::item_part::compile_item_part_v1(&request).map_err(|error| serialize_json(&error))
}

/// Item-part result returned as binary MDL plus typed compile/readback reports.
#[wasm_bindgen]
pub struct ItemPartWasmArtifactV1 {
    mdl_bytes: Vec<u8>,
    report_json: String,
    readback_json: String,
}

#[wasm_bindgen]
impl ItemPartWasmArtifactV1 {
    #[wasm_bindgen(js_name = takeMdlBytes)]
    pub fn take_mdl_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.mdl_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

/// Compiles one hash-bound common model IR as a classification-0 Item part.
#[wasm_bindgen(js_name = compileItemPartV1)]
pub fn compile_item_part_v1(request_json: &str) -> Result<ItemPartWasmArtifactV1, JsValue> {
    compile_item_part_v1_inner(request_json)
        .map(|artifact| ItemPartWasmArtifactV1 {
            report_json: serialize_json(&artifact.report),
            readback_json: serialize_json(&artifact.inspection),
            mdl_bytes: artifact.payload,
        })
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_item_uti_boundary(
    request_json: &str,
    options_json: &str,
) -> Result<
    (
        m2a_core::item_uti::ItemUtiBuildRequestV1,
        m2a_core::gff::GffWriterOptionsV1,
    ),
    String,
> {
    let request = serde_json::from_str(request_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-UTI-REQUEST-JSON-INVALID",
            "requestJson",
            "Item UTI request JSON does not match the strict public schema",
        )
    })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-UTI-OPTIONS-JSON-INVALID",
            "optionsJson",
            "GFF writer options JSON does not match the strict public schema",
        )
    })?;
    Ok((request, options))
}

fn write_item_uti_v1_inner(
    request_json: &str,
    options_json: &str,
) -> Result<m2a_core::item_uti::ItemUtiArtifactV1, String> {
    let (request, options) = parse_item_uti_boundary(request_json, options_json)?;
    m2a_core::item_uti::write_item_uti_v1(&request, &options)
        .map_err(|error| serialize_json(&error))
}

/// Item UTI result returned as binary bytes plus separate typed reports.
#[wasm_bindgen]
pub struct ItemUtiWasmArtifactV1 {
    uti_bytes: Vec<u8>,
    report_json: String,
    readback_json: String,
}

#[wasm_bindgen]
impl ItemUtiWasmArtifactV1 {
    #[wasm_bindgen(js_name = takeUtiBytes)]
    pub fn take_uti_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.uti_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

/// Writes a deterministic production UTI for ModelType 0/1/2/3.
#[wasm_bindgen(js_name = writeItemUtiV1)]
pub fn write_item_uti_v1(
    request_json: &str,
    options_json: &str,
) -> Result<ItemUtiWasmArtifactV1, JsValue> {
    write_item_uti_v1_inner(request_json, options_json)
        .map(|artifact| ItemUtiWasmArtifactV1 {
            report_json: serialize_json(&artifact.report),
            readback_json: serialize_json(&artifact.readback),
            uti_bytes: artifact.payload,
        })
        .map_err(|error| JsValue::from_str(&error))
}

fn read_item_uti_v1_json_inner(
    bytes: &[u8],
    model_type: u8,
    limits_json: &str,
) -> Result<String, String> {
    let profile = m2a_core::item::ItemCompositionProfileV1::from_model_type(model_type)
        .map_err(|error| serialize_json(&error))?;
    let limits = serde_json::from_str(limits_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-UTI-LIMITS-JSON-INVALID",
            "limitsJson",
            "GFF limits JSON does not match the strict public schema",
        )
    })?;
    m2a_core::item_uti::read_item_uti_v1(bytes, profile, &limits)
        .map(|readback| serialize_json(&readback))
        .map_err(|error| serialize_json(&error))
}

/// Reads exact typed Item UTI semantics for one caller-selected ModelType.
#[wasm_bindgen(js_name = readItemUtiV1Json)]
pub fn read_item_uti_v1_json(
    bytes: &[u8],
    model_type: u8,
    limits_json: &str,
) -> Result<String, JsValue> {
    read_item_uti_v1_json_inner(bytes, model_type, limits_json)
        .map_err(|error| JsValue::from_str(&error))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemIconOutputDescriptorV1 {
    slot: m2a_core::item::ItemPartSlotV1,
    resref: String,
    report: m2a_core::item_icon::ItemIconLayerReportV1,
    payload_offset: u32,
    payload_size: u32,
}

fn write_item_icon_layers_v1_inner(
    recipe_json: &str,
    layers_json: &str,
    options_json: &str,
) -> Result<(Vec<u8>, String, String), String> {
    let recipe = serde_json::from_str(recipe_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-ICON-RECIPE-JSON-INVALID",
            "recipeJson",
            "Item recipe JSON does not match the strict public schema",
        )
    })?;
    let layers: Vec<m2a_core::item_icon::ItemIconLayerInputV1> = serde_json::from_str(layers_json)
        .map_err(|_| {
            m5_boundary_error(
                "M2A-ITEM-ICON-LAYERS-JSON-INVALID",
                "layersJson",
                "Item icon layers JSON does not match the strict public schema",
            )
        })?;
    let options = serde_json::from_str(options_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-ICON-OPTIONS-JSON-INVALID",
            "optionsJson",
            "TGA writer options JSON does not match the strict public schema",
        )
    })?;
    let artifacts = m2a_core::item_icon::write_item_icon_layers_v1(&recipe, &layers, &options)
        .map_err(|error| serialize_json(&error))?;
    let mut payload_blob = Vec::new();
    let mut descriptors = Vec::with_capacity(artifacts.len());
    let mut reports = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let payload_offset = u32::try_from(payload_blob.len()).map_err(|_| {
            m5_boundary_error(
                "M2A-ITEM-ICON-PAYLOAD-OVERFLOW",
                "payloadBlob",
                "Item icon payload offset exceeds u32",
            )
        })?;
        let payload_size = u32::try_from(artifact.payload.len()).map_err(|_| {
            m5_boundary_error(
                "M2A-ITEM-ICON-PAYLOAD-OVERFLOW",
                "payloadBlob",
                "Item icon payload size exceeds u32",
            )
        })?;
        payload_blob.extend_from_slice(&artifact.payload);
        reports.push(artifact.report.clone());
        descriptors.push(ItemIconOutputDescriptorV1 {
            slot: artifact.slot,
            resref: artifact.resref,
            report: artifact.report,
            payload_offset,
            payload_size,
        });
    }
    Ok((
        payload_blob,
        serialize_json(&descriptors),
        serialize_json(&reports),
    ))
}

#[wasm_bindgen]
pub struct ItemIconLayersWasmArtifactV1 {
    payload_blob: Vec<u8>,
    descriptors_json: String,
    reports_json: String,
}

#[wasm_bindgen]
impl ItemIconLayersWasmArtifactV1 {
    #[wasm_bindgen(js_name = takePayloadBlob)]
    pub fn take_payload_blob(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.payload_blob)
    }

    #[wasm_bindgen(getter, js_name = descriptorsJson)]
    pub fn descriptors_json(&self) -> String {
        self.descriptors_json.clone()
    }

    #[wasm_bindgen(getter, js_name = reportsJson)]
    pub fn reports_json(&self) -> String {
        self.reports_json.clone()
    }
}

/// Writes all exact profile-required RGBA8 Item icon layers as TGA payloads.
#[wasm_bindgen(js_name = writeItemIconLayersV1)]
pub fn write_item_icon_layers_v1(
    recipe_json: &str,
    layers_json: &str,
    options_json: &str,
) -> Result<ItemIconLayersWasmArtifactV1, JsValue> {
    write_item_icon_layers_v1_inner(recipe_json, layers_json, options_json)
        .map(
            |(payload_blob, descriptors_json, reports_json)| ItemIconLayersWasmArtifactV1 {
                payload_blob,
                descriptors_json,
                reports_json,
            },
        )
        .map_err(|error| JsValue::from_str(&error))
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ItemPartPayloadDescriptorV1 {
    slot: m2a_core::item::ItemPartSlotV1,
    resref: String,
    source_sha256: String,
    compile_report: m2a_core::item_part::ItemPartCompileReportV1,
    payload_offset: u32,
    payload_size: u32,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ItemIconPayloadDescriptorV1 {
    slot: m2a_core::item::ItemPartSlotV1,
    resref: String,
    report: m2a_core::item_icon::ItemIconLayerReportV1,
    payload_offset: u32,
    payload_size: u32,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ItemPackageBoundaryRequestV1 {
    schema_version: u32,
    generator_identity: String,
    recipe: m2a_core::item::ItemAppearanceRecipeV1,
    namespace: m2a_core::item::EffectiveResourceNamespaceV1,
    uti: m2a_core::item_uti::ItemUtiBuildRequestV1,
    parts: Vec<ItemPartPayloadDescriptorV1>,
    icons: Vec<ItemIconPayloadDescriptorV1>,
    additional_hak_resources: Vec<HakResourceDescriptorV1>,
    module_fixture_resources: Vec<HakResourceDescriptorV1>,
}

fn item_package_range_error(path: &str, message: &str) -> String {
    m5_boundary_error("M2A-ITEM-PACKAGE-PAYLOAD-RANGE-INVALID", path, message)
}

fn item_payload_slice<'a>(
    payload_blob: &'a [u8],
    offset: u32,
    size: u32,
    path: &str,
) -> Result<&'a [u8], String> {
    let start = usize::try_from(offset)
        .map_err(|_| item_package_range_error(path, "payload offset does not fit this platform"))?;
    let length = usize::try_from(size)
        .map_err(|_| item_package_range_error(path, "payload size does not fit this platform"))?;
    let end = start
        .checked_add(length)
        .ok_or_else(|| item_package_range_error(path, "payload range overflows this platform"))?;
    payload_blob
        .get(start..end)
        .ok_or_else(|| item_package_range_error(path, "payload range is outside payloadBlob"))
}

fn validate_item_package_payload_ranges(
    payload_blob: &[u8],
    request: &ItemPackageBoundaryRequestV1,
) -> Result<(), String> {
    let mut ranges = Vec::new();
    for (index, descriptor) in request.parts.iter().enumerate() {
        ranges.push((
            descriptor.payload_offset,
            descriptor.payload_size,
            format!("parts[{index}]"),
        ));
    }
    for (index, descriptor) in request.icons.iter().enumerate() {
        ranges.push((
            descriptor.payload_offset,
            descriptor.payload_size,
            format!("icons[{index}]"),
        ));
    }
    for (index, descriptor) in request.additional_hak_resources.iter().enumerate() {
        ranges.push((
            descriptor.payload_offset,
            descriptor.payload_size,
            format!("additionalHakResources[{index}]"),
        ));
    }
    for (index, descriptor) in request.module_fixture_resources.iter().enumerate() {
        ranges.push((
            descriptor.payload_offset,
            descriptor.payload_size,
            format!("moduleFixtureResources[{index}]"),
        ));
    }
    let mut normalized = Vec::with_capacity(ranges.len());
    for (offset, size, path) in ranges {
        let slice = item_payload_slice(payload_blob, offset, size, &path)?;
        if !slice.is_empty() {
            normalized.push((offset as usize, offset as usize + slice.len(), path));
        }
    }
    normalized.sort_by_key(|range| (range.0, range.1));
    let mut cursor = 0usize;
    for (start, end, path) in normalized {
        if start != cursor {
            return Err(item_package_range_error(
                &path,
                if start < cursor {
                    "non-empty payload ranges overlap"
                } else {
                    "payload ranges leave an unowned gap"
                },
            ));
        }
        cursor = end;
    }
    if cursor != payload_blob.len() {
        return Err(item_package_range_error(
            "payloadBlob",
            "payload ranges do not consume exact payloadBlob",
        ));
    }
    Ok(())
}

fn materialize_item_hak_resource(
    payload_blob: &[u8],
    descriptor: &HakResourceDescriptorV1,
    path: &str,
) -> Result<m2a_core::hak::HakResourceInputV1, String> {
    Ok(m2a_core::hak::HakResourceInputV1 {
        resref: descriptor.resref.clone(),
        resource_type: descriptor.resource_type,
        payload: item_payload_slice(
            payload_blob,
            descriptor.payload_offset,
            descriptor.payload_size,
            path,
        )?
        .to_vec(),
    })
}

fn write_item_package_v1_inner(
    payload_blob: &[u8],
    request_json: &str,
    gff_options_json: &str,
    archive_options_json: &str,
) -> Result<m2a_core::item_package::ItemPackageArtifactV1, String> {
    let boundary: ItemPackageBoundaryRequestV1 =
        serde_json::from_str(request_json).map_err(|_| {
            m5_boundary_error(
                "M2A-ITEM-PACKAGE-REQUEST-JSON-INVALID",
                "requestJson",
                "Item package request JSON does not match the strict public schema",
            )
        })?;
    validate_item_package_payload_ranges(payload_blob, &boundary)?;
    let gff_options = serde_json::from_str(gff_options_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-PACKAGE-GFF-OPTIONS-INVALID",
            "gffOptionsJson",
            "GFF options JSON does not match the strict public schema",
        )
    })?;
    let archive_options = serde_json::from_str(archive_options_json).map_err(|_| {
        m5_boundary_error(
            "M2A-ITEM-PACKAGE-ARCHIVE-OPTIONS-INVALID",
            "archiveOptionsJson",
            "archive options JSON does not match the strict public schema",
        )
    })?;
    let parts = boundary
        .parts
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            Ok(m2a_core::item_package::ItemCompiledPartInputV1 {
                slot: descriptor.slot,
                resref: descriptor.resref.clone(),
                source_sha256: descriptor.source_sha256.clone(),
                compile_report: descriptor.compile_report.clone(),
                payload: item_payload_slice(
                    payload_blob,
                    descriptor.payload_offset,
                    descriptor.payload_size,
                    &format!("parts[{index}]"),
                )?
                .to_vec(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let icons = boundary
        .icons
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            Ok(m2a_core::item_icon::ItemIconLayerArtifactV1 {
                slot: descriptor.slot,
                resref: descriptor.resref.clone(),
                payload: item_payload_slice(
                    payload_blob,
                    descriptor.payload_offset,
                    descriptor.payload_size,
                    &format!("icons[{index}]"),
                )?
                .to_vec(),
                report: descriptor.report.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let additional_hak_resources = boundary
        .additional_hak_resources
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            materialize_item_hak_resource(
                payload_blob,
                descriptor,
                &format!("additionalHakResources[{index}]"),
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    let module_fixture_resources = boundary
        .module_fixture_resources
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            materialize_item_hak_resource(
                payload_blob,
                descriptor,
                &format!("moduleFixtureResources[{index}]"),
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    m2a_core::item_package::write_item_package_v1(
        &m2a_core::item_package::ItemPackageBuildRequestV1 {
            schema_version: boundary.schema_version,
            generator_identity: boundary.generator_identity,
            recipe: boundary.recipe,
            namespace: boundary.namespace,
            uti: boundary.uti,
            parts,
            icons,
            additional_hak_resources,
            module_fixture_resources,
        },
        &gff_options,
        &archive_options,
    )
    .map_err(|error| serialize_json(&error))
}

#[wasm_bindgen]
pub struct ItemPackageWasmArtifactV1 {
    uti_bytes: Vec<u8>,
    hak_bytes: Vec<u8>,
    module_bytes: Vec<u8>,
    manifest_json: Vec<u8>,
    manifest_sha256: String,
    uti_report_json: String,
    uti_readback_json: String,
}

#[wasm_bindgen]
impl ItemPackageWasmArtifactV1 {
    #[wasm_bindgen(js_name = takeUtiBytes)]
    pub fn take_uti_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.uti_bytes)
    }

    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModuleBytes)]
    pub fn take_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.module_bytes)
    }

    #[wasm_bindgen(js_name = takeManifestBytes)]
    pub fn take_manifest_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.manifest_json)
    }

    #[wasm_bindgen(getter, js_name = manifestSha256)]
    pub fn manifest_sha256(&self) -> String {
        self.manifest_sha256.clone()
    }

    #[wasm_bindgen(getter, js_name = utiReportJson)]
    pub fn uti_report_json(&self) -> String {
        self.uti_report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = utiReadbackJson)]
    pub fn uti_readback_json(&self) -> String {
        self.uti_readback_json.clone()
    }
}

/// Builds one deterministic Item UTI/HAK/MOD/manifest graph off the UI thread.
#[wasm_bindgen(js_name = writeItemPackageV1)]
pub fn write_item_package_v1(
    payload_blob: &[u8],
    request_json: &str,
    gff_options_json: &str,
    archive_options_json: &str,
) -> Result<ItemPackageWasmArtifactV1, JsValue> {
    write_item_package_v1_inner(
        payload_blob,
        request_json,
        gff_options_json,
        archive_options_json,
    )
    .map(|artifact| ItemPackageWasmArtifactV1 {
        uti_report_json: serialize_json(&artifact.uti.report),
        uti_readback_json: serialize_json(&artifact.uti.readback),
        uti_bytes: artifact.uti.payload,
        hak_bytes: artifact.hak_payload,
        module_bytes: artifact.module_payload,
        manifest_json: artifact.manifest_json,
        manifest_sha256: artifact.manifest_sha256,
    })
    .map_err(|error| JsValue::from_str(&error))
}

fn parse_two_da_append_json(
    request_json: &str,
    limits_json: &str,
) -> Result<
    (
        m2a_core::two_da::TwoDaAppendRequestV1,
        m2a_core::two_da::TwoDaLimitsV1,
    ),
    String,
> {
    let request = serde_json::from_str(request_json).map_err(|_| {
        m5_boundary_error(
            "M5-2DA-REQUEST-JSON-INVALID",
            "requestJson",
            "2DA append request JSON does not match the strict public schema",
        )
    })?;
    let limits = parse_two_da_limits_json(limits_json)?;
    Ok((request, limits))
}

fn append_two_da_row_artifact_json(
    bytes: &[u8],
    request_json: &str,
    limits_json: &str,
) -> Result<m2a_core::two_da::TwoDaAppendArtifactV1, String> {
    let (request, limits) = parse_two_da_append_json(request_json, limits_json)?;
    m2a_core::two_da::append_two_da_row_v1(bytes, &request, &limits)
        .map_err(|error| two_da_core_error_json(&error))
}

/// Appends one full-width row while preserving every source byte.
#[wasm_bindgen(js_name = appendTwoDaRowV1)]
pub fn append_two_da_row_v1(
    bytes: &[u8],
    request_json: &str,
    limits_json: &str,
) -> Result<Vec<u8>, JsValue> {
    append_two_da_row_artifact_json(bytes, request_json, limits_json)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the report for the same deterministic append core operation.
#[wasm_bindgen(js_name = appendTwoDaRowV1ReportJson)]
pub fn append_two_da_row_v1_report_json(
    bytes: &[u8],
    request_json: &str,
    limits_json: &str,
) -> Result<String, JsValue> {
    append_two_da_row_artifact_json(bytes, request_json, limits_json)
        .map(|artifact| serialize_json(&artifact.report))
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_hak_resources_json(resources_json: &str) -> Result<HakResourceDescriptorsV1, String> {
    let descriptors: HakResourceDescriptorsV1 =
        serde_json::from_str(resources_json).map_err(|_| {
            m5_boundary_error(
                "M5-HAK-RESOURCES-JSON-INVALID",
                "resourcesJson",
                "HAK resources JSON does not match the strict public schema",
            )
        })?;
    if descriptors.schema_version != 1 {
        return Err(m5_boundary_error(
            "M5-HAK-RESOURCES-JSON-INVALID",
            "resourcesJson",
            "HAK resources schemaVersion must be 1",
        ));
    }
    Ok(descriptors)
}

fn parse_hak_options_json(options_json: &str) -> Result<m2a_core::hak::HakWriterOptionsV1, String> {
    serde_json::from_str(options_json).map_err(|_| {
        m5_boundary_error(
            "M5-HAK-OPTIONS-JSON-INVALID",
            "optionsJson",
            "HAK options JSON does not match the strict public schema",
        )
    })
}

fn hak_range_error(path: &str, message: &str) -> String {
    m5_boundary_error("M5-HAK-PAYLOAD-RANGE-INVALID", path, message)
}

fn hak_allocation_error(message: &str) -> String {
    m5_boundary_error("M5-HAK-ALLOCATION-FAILED", "output", message)
}

fn clone_hak_string(value: &str) -> Result<String, String> {
    let mut output = String::new();
    output
        .try_reserve_exact(value.len())
        .map_err(|_| hak_allocation_error("could not reserve HAK resource resref"))?;
    output.push_str(value);
    Ok(output)
}

fn validate_hak_payload_ranges(
    payload_blob: &[u8],
    descriptors: &HakResourceDescriptorsV1,
) -> Result<(), String> {
    let mut ranges = Vec::new();
    ranges
        .try_reserve_exact(descriptors.resources.len())
        .map_err(|_| hak_allocation_error("could not reserve HAK payload range plan"))?;
    for (index, descriptor) in descriptors.resources.iter().enumerate() {
        let start = usize::try_from(descriptor.payload_offset).map_err(|_| {
            hak_range_error(
                &format!("resources[{index}].payloadOffset"),
                "payloadOffset does not fit this platform",
            )
        })?;
        if start > payload_blob.len() {
            return Err(hak_range_error(
                &format!("resources[{index}].payloadOffset"),
                "payloadOffset is outside payloadBlob",
            ));
        }
        let size = usize::try_from(descriptor.payload_size).map_err(|_| {
            hak_range_error(
                &format!("resources[{index}].payloadSize"),
                "payloadSize does not fit this platform",
            )
        })?;
        let end = start.checked_add(size).ok_or_else(|| {
            hak_range_error(
                &format!("resources[{index}].payloadSize"),
                "payload range overflows this platform",
            )
        })?;
        if end > payload_blob.len() {
            return Err(hak_range_error(
                &format!("resources[{index}].payloadSize"),
                "payload range extends past payloadBlob",
            ));
        }
        if size != 0 {
            ranges.push((start, end));
        }
    }
    ranges.sort_unstable();
    let mut cursor = 0usize;
    for (start, end) in ranges {
        if start < cursor {
            return Err(hak_range_error(
                "payloadBlob",
                "non-empty resource payload ranges overlap",
            ));
        }
        if start > cursor {
            return Err(hak_range_error(
                "payloadBlob",
                "resource payload ranges leave a gap",
            ));
        }
        cursor = end;
    }
    if cursor != payload_blob.len() {
        return Err(hak_range_error(
            "payloadBlob",
            "resource payload ranges do not consume exact payloadBlob",
        ));
    }

    Ok(())
}

fn materialize_hak_resources(
    payload_blob: &[u8],
    descriptors: &HakResourceDescriptorsV1,
) -> Result<Vec<m2a_core::hak::HakResourceInputV1>, String> {
    let mut resources = Vec::new();
    resources
        .try_reserve_exact(descriptors.resources.len())
        .map_err(|_| hak_allocation_error("could not reserve HAK resources"))?;
    for descriptor in &descriptors.resources {
        let start = descriptor.payload_offset as usize;
        let end = start + descriptor.payload_size as usize;
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(descriptor.payload_size as usize)
            .map_err(|_| hak_allocation_error("could not reserve HAK resource payload"))?;
        payload.extend_from_slice(&payload_blob[start..end]);
        resources.push(m2a_core::hak::HakResourceInputV1 {
            resref: clone_hak_string(&descriptor.resref)?,
            resource_type: descriptor.resource_type,
            payload,
        });
    }
    Ok(resources)
}

fn parse_hak_boundary(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<
    (
        Vec<m2a_core::hak::HakResourceInputV1>,
        m2a_core::hak::HakWriterOptionsV1,
    ),
    String,
> {
    let descriptors = parse_hak_resources_json(resources_json)?;
    let options = parse_hak_options_json(options_json)?;
    validate_hak_payload_ranges(payload_blob, &descriptors)?;
    m2a_core::hak::preflight_hak_v1(&descriptors.resources, &options)
        .map_err(|error| serialize_json(&error))?;
    let resources = materialize_hak_resources(payload_blob, &descriptors)?;
    Ok((resources, options))
}

fn write_hak_artifact_json(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<m2a_core::hak::HakArtifactV1, String> {
    let (resources, options) = parse_hak_boundary(payload_blob, resources_json, options_json)?;
    m2a_core::hak::write_hak_v1(&resources, &options).map_err(|error| serialize_json(&error))
}

/// Writes deterministic HAK bytes from one blob and strict resource descriptors.
#[wasm_bindgen(js_name = writeHakV1)]
pub fn write_hak_v1(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<Vec<u8>, JsValue> {
    write_hak_artifact_json(payload_blob, resources_json, options_json)
        .map(|artifact| artifact.payload)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the report for the same deterministic HAK core operation.
#[wasm_bindgen(js_name = writeHakV1ReportJson)]
pub fn write_hak_v1_report_json(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    write_hak_artifact_json(payload_blob, resources_json, options_json)
        .map(|artifact| serialize_json(&artifact.report))
        .map_err(|error| JsValue::from_str(&error))
}

fn write_package_manifest_v1_json_inner(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<String, String> {
    let (resources, options) = parse_hak_boundary(payload_blob, resources_json, options_json)?;
    m2a_core::package::write_package_manifest_v1(&resources, &options)
        .map(|manifest| serialize_json(&manifest))
        .map_err(|error| serialize_json(&error))
}

/// Model-only package result returned from one core composition pass.
///
/// JavaScript receives HAK bytes separately from JSON metadata, so binary
/// payloads never cross the boundary as base64.
#[wasm_bindgen]
pub struct ModelPackageArtifactV1 {
    hak_bytes: Vec<u8>,
    report_json: String,
    manifest_json: String,
}

#[wasm_bindgen]
impl ModelPackageArtifactV1 {
    /// Transfers ownership of the HAK buffer to JavaScript exactly once.
    /// Later calls deterministically return an empty `Uint8Array`.
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }
}

fn write_model_package_v1_inner(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<ModelPackageArtifactV1, String> {
    let (resources, options) = parse_hak_boundary(payload_blob, resources_json, options_json)?;
    let artifact = m2a_core::package::write_model_package_v1(&resources, &options)
        .map_err(|error| serialize_json(&error))?;
    Ok(ModelPackageArtifactV1 {
        report_json: serialize_json(&artifact.hak.report),
        manifest_json: serialize_json(&artifact.manifest),
        hak_bytes: artifact.hak.payload,
    })
}

/// Composes ready binary MDL+MDX, TGA and appended appearance.2da payloads.
///
/// One call performs one HAK write and returns its bytes, report and exact
/// manifest. Call `takeHakBytes()` once to transfer the binary buffer without
/// cloning it.
#[wasm_bindgen(js_name = writeModelPackageV1)]
pub fn write_model_package_v1(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<ModelPackageArtifactV1, JsValue> {
    write_model_package_v1_inner(payload_blob, resources_json, options_json)
        .map_err(|error| JsValue::from_str(&error))
}

/// Returns the deterministic manifest sidecar after successful HAK own-readback.
#[wasm_bindgen(js_name = writePackageManifestV1Json)]
pub fn write_package_manifest_v1_json(
    payload_blob: &[u8],
    resources_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    write_package_manifest_v1_json_inner(payload_blob, resources_json, options_json)
        .map_err(|error| JsValue::from_str(&error))
}

/// Browser-transferable result of the static-mesh plus animated-donor route.
/// The binary model buffer can be taken once; the deterministic JSON sidecars
/// remain available for audit and UI presentation.
#[wasm_bindgen]
pub struct AnimatedDonorModelArtifactV1 {
    model_bytes: Vec<u8>,
    report_json: String,
    readback_json: String,
    animations_json: String,
    conversion_json: String,
}

#[wasm_bindgen]
impl AnimatedDonorModelArtifactV1 {
    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = animationsJson)]
    pub fn animations_json(&self) -> String {
        self.animations_json.clone()
    }

    #[wasm_bindgen(getter, js_name = conversionJson)]
    pub fn conversion_json(&self) -> String {
        self.conversion_json.clone()
    }
}

/// Retargets a browser-selected static GLB onto a separate browser-selected
/// Meshy H1-style animated donor and emits one self-validated binary MDL.
#[wasm_bindgen(js_name = retargetStaticMeshToAnimatedDonorV1)]
pub fn retarget_static_mesh_to_animated_donor_v1(
    source_glb: &[u8],
    animated_donor_glb: &[u8],
    writer_options_json: &str,
) -> Result<AnimatedDonorModelArtifactV1, JsValue> {
    let options = serde_json::from_str::<m2a_core::mdl::MdlWriterOptionsV1>(writer_options_json)
        .map_err(|_| {
            JsValue::from_str(&mdl_writer_json_error(
                "M7-DONOR-WRITER-OPTIONS-JSON-INVALID",
                "writerOptionsJson",
                "writer options JSON does not match the strict public schema",
            ))
        })?;
    let artifact = m2a_core::animated_donor::retarget_static_mesh_to_animated_donor_v1(
        source_glb,
        animated_donor_glb,
        &options,
    )
    .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(AnimatedDonorModelArtifactV1 {
        model_bytes: artifact.model.payload,
        report_json: serialize_json(&artifact.report),
        readback_json: serialize_json(&artifact.model.inspection),
        animations_json: serialize_json(&artifact.animations),
        conversion_json: serialize_json(&artifact.conversion),
    })
}

/// Browser Studio result from the existing canonical model-only pipeline.
/// Binary buffers cross the boundary separately and can each be taken once.
#[wasm_bindgen]
pub struct StudioModelPackageArtifactV1 {
    hak_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    proof_module_bytes: Vec<u8>,
    pwk_bytes: Vec<u8>,
    texture_payload_blob: Vec<u8>,
    texture_descriptors_json: String,
    report_json: String,
    manifest_json: String,
    summary_json: String,
    readback_json: String,
}

#[wasm_bindgen]
impl StudioModelPackageArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    /// Transfers the self-contained generated proof module exactly once.
    #[wasm_bindgen(js_name = takeProofModuleBytes)]
    pub fn take_proof_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.proof_module_bytes)
    }

    #[wasm_bindgen(js_name = takePwkBytes)]
    pub fn take_pwk_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pwk_bytes)
    }

    #[wasm_bindgen(js_name = takeTexturePayloadBlob)]
    pub fn take_texture_payload_blob(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_payload_blob)
    }

    #[wasm_bindgen(getter, js_name = textureDescriptorsJson)]
    pub fn texture_descriptors_json(&self) -> String {
        self.texture_descriptors_json.clone()
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }

    #[wasm_bindgen(getter, js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.summary_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

/// Browser result for production creature resources only. A proof MOD is a
/// separate operation and therefore has no byte getter on this boundary.
#[wasm_bindgen]
pub struct StudioCreatureProductArtifactV2 {
    hak_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    texture_bytes: Vec<u8>,
    texture_payload_blob: Vec<u8>,
    texture_descriptors_json: String,
    report_json: String,
    manifest_json: String,
    summary_json: String,
    readback_json: String,
}

/// Browser result for one immutable production creature plus its separately
/// authored demo MOD. The product report/manifest remain product-only; the
/// module has its own canonical report and does not mutate product bytes.
#[wasm_bindgen]
pub struct StudioCreatureProductDemoArtifactV1 {
    hak_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    texture_bytes: Vec<u8>,
    texture_payload_blob: Vec<u8>,
    texture_descriptors_json: String,
    proof_module_bytes: Vec<u8>,
    report_json: String,
    manifest_json: String,
    summary_json: String,
    readback_json: String,
    demo_report_json: String,
}

/// Browser-transferable complete output of the isolated P100K Creature
/// experiment. Unlike the production-only V2 artifact, this boundary includes
/// the caller-identified demo MOD and exposes every generated resource for
/// deterministic boundary verification.
#[wasm_bindgen]
pub struct StudioP100kCreaturePackageArtifactV1 {
    hak_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    texture_bytes: Vec<u8>,
    appearance_two_da_bytes: Vec<u8>,
    proof_module_bytes: Vec<u8>,
    report_json: String,
    manifest_json: String,
    summary_json: String,
    readback_json: String,
}

#[wasm_bindgen]
impl StudioP100kCreaturePackageArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(js_name = takeTextureBytes)]
    pub fn take_texture_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_bytes)
    }

    #[wasm_bindgen(js_name = takeAppearanceTwoDaBytes)]
    pub fn take_appearance_two_da_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.appearance_two_da_bytes)
    }

    #[wasm_bindgen(js_name = takeProofModuleBytes)]
    pub fn take_proof_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.proof_module_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }

    #[wasm_bindgen(getter, js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.summary_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

#[wasm_bindgen]
impl StudioCreatureProductArtifactV2 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(js_name = takeTextureBytes)]
    pub fn take_texture_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_bytes)
    }

    #[wasm_bindgen(js_name = takeTexturePayloadBlob)]
    pub fn take_texture_payload_blob(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_payload_blob)
    }

    #[wasm_bindgen(getter, js_name = textureDescriptorsJson)]
    pub fn texture_descriptors_json(&self) -> String {
        self.texture_descriptors_json.clone()
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }

    #[wasm_bindgen(getter, js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.summary_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }
}

#[wasm_bindgen]
impl StudioCreatureProductDemoArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(js_name = takeTextureBytes)]
    pub fn take_texture_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_bytes)
    }

    #[wasm_bindgen(js_name = takeTexturePayloadBlob)]
    pub fn take_texture_payload_blob(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_payload_blob)
    }

    #[wasm_bindgen(getter, js_name = textureDescriptorsJson)]
    pub fn texture_descriptors_json(&self) -> String {
        self.texture_descriptors_json.clone()
    }

    #[wasm_bindgen(js_name = takeProofModuleBytes)]
    pub fn take_proof_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.proof_module_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = manifestJson)]
    pub fn manifest_json(&self) -> String {
        self.manifest_json.clone()
    }

    #[wasm_bindgen(getter, js_name = summaryJson)]
    pub fn summary_json(&self) -> String {
        self.summary_json.clone()
    }

    #[wasm_bindgen(getter, js_name = readbackJson)]
    pub fn readback_json(&self) -> String {
        self.readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = demoReportJson)]
    pub fn demo_report_json(&self) -> String {
        self.demo_report_json.clone()
    }
}

/// Executes the canonical Rust model-only GLB -> MDL/TGA/2DA/HAK pipeline for
/// browser-selected bytes. No filesystem, DOM or alternate conversion path is
/// involved at this boundary.
#[wasm_bindgen(js_name = buildM6ModelPackageV1)]
pub fn build_m6_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    let artifact =
        m2a_core::model_pipeline::build_m6_model_package_v1(source_glb, appearance_two_da)
            .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;
    let readback = m2a_core::inspect_binary_mdl(&artifact.model)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        pwk_bytes: Vec::new(),
        texture_payload_blob: Vec::new(),
        texture_descriptors_json: "[]".to_owned(),
        report_json: String::from_utf8(artifact.report_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        readback_json: serialize_json(&readback),
    })
}

/// Executes the constrained real-Meshy H1 Studio route for browser-selected
/// bytes.  It derives its clean-room rig from the selected GLB and returns the
/// same one-shot transferable artifact surface as the synthetic M6 proof.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV1)]
pub fn build_meshy_h1_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    let artifact =
        m2a_core::model_pipeline::build_meshy_h1_model_package_v1(source_glb, appearance_two_da)
            .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;
    let readback = m2a_core::inspect_binary_mdl(&artifact.model)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        pwk_bytes: Vec::new(),
        texture_payload_blob: Vec::new(),
        texture_descriptors_json: "[]".to_owned(),
        report_json: String::from_utf8(artifact.report_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds the self-contained H1 profile only when all 42 direct-creature
/// states are present as explicitly named source animations. Unlike V1, this
/// boundary never manufactures gameplay clips by renaming the idle motion.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV2)]
pub fn build_meshy_h1_model_package_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_h1_model_package_v2_inner(source_glb, appearance_two_da)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_h1_model_package_v2_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, String> {
    let artifact = m2a_core::model_pipeline::build_meshy_h1_model_package_v2(
        source_glb,
        appearance_two_da,
        m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        pwk_bytes: Vec::new(),
        texture_payload_blob: Vec::new(),
        texture_descriptors_json: "[]".to_owned(),
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds a skinned Meshy humanoid with one preserved idle and 41 distinct
/// clean-room gameplay clips. This boundary never falls back to the historical
/// seven aliases of the same idle motion.
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidModelPackageV1)]
pub fn build_meshy_procedural_humanoid_model_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_model_package_v1_inner(source_glb, appearance_two_da)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_model_package_v1_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, String> {
    let artifact = m2a_core::model_pipeline::build_meshy_procedural_humanoid_model_package_v1(
        source_glb,
        appearance_two_da,
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        pwk_bytes: Vec::new(),
        texture_payload_blob: Vec::new(),
        texture_descriptors_json: "[]".to_owned(),
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds the production-only procedural creature under an exact caller-owned
/// identity. This V2 boundary cannot emit a MOD or UTC.
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidProductV2)]
pub fn build_meshy_procedural_humanoid_product_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
) -> Result<StudioCreatureProductArtifactV2, JsValue> {
    build_meshy_procedural_humanoid_product_v2_inner(source_glb, appearance_two_da, identity_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn pack_model_texture_artifacts_v1(
    textures: &[m2a_core::model_texture_authoring::ResolvedModelTexturePayloadV1],
) -> (Vec<u8>, String) {
    let mut payload_blob = Vec::new();
    let mut descriptors = Vec::with_capacity(textures.len());
    for texture in textures {
        let byte_offset = payload_blob.len() as u64;
        payload_blob.extend_from_slice(&texture.payload);
        descriptors.push(PlaceableTextureArtifactDescriptorV1 {
            schema_version: 1,
            resref: texture.resref.clone(),
            resource_type: texture.resource_type,
            byte_offset,
            byte_length: texture.payload.len() as u64,
            sha256: m2a_core::placeable_collision::sha256_hex(&texture.payload),
        });
    }
    (payload_blob, serialize_json(&descriptors))
}

enum ParsedCreatureMaterialSeparationV2 {
    ComponentV1(m2a_core::model_material_separation::ModelMaterialSeparationDocumentV1),
    FaceV2(m2a_core::model_material_separation::ModelMaterialSeparationDocumentV2),
}

impl ParsedCreatureMaterialSeparationV2 {
    fn as_core_input(
        &self,
    ) -> m2a_core::model_pipeline::CreatureModelMaterialSeparationInputV2<'_> {
        match self {
            Self::ComponentV1(document) => {
                m2a_core::model_pipeline::CreatureModelMaterialSeparationInputV2::ComponentV1(
                    document,
                )
            }
            Self::FaceV2(document) => {
                m2a_core::model_pipeline::CreatureModelMaterialSeparationInputV2::FaceV2(document)
            }
        }
    }
}

fn parse_creature_model_material_authoring_v2(
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_descriptors_json: &str,
) -> Result<
    (
        ParsedCreatureMaterialSeparationV2,
        m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
        Vec<m2a_core::model_texture_authoring::ModelTexturePayloadDescriptorV1>,
    ),
    String,
> {
    let schema_version = serde_json::from_str::<serde_json::Value>(material_separation_json)
        .ok()
        .and_then(|value| {
            value
                .get("schemaVersion")
                .and_then(serde_json::Value::as_u64)
        });
    let separation = match schema_version {
        Some(1) => serde_json::from_str(material_separation_json)
            .map(ParsedCreatureMaterialSeparationV2::ComponentV1),
        Some(2) => serde_json::from_str(material_separation_json)
            .map(ParsedCreatureMaterialSeparationV2::FaceV2),
        _ => Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported Creature Material Separation schema",
        ))),
    }
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "materialSeparationJson",
            "material separation JSON does not match the strict V1 or Face Mode V2 schema",
        ))
    })?;
    let texture_authoring = serde_json::from_str(model_texture_authoring_json).map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-TEXTURE-DOCUMENT-JSON-INVALID",
            "modelTextureAuthoringJson",
            "model texture JSON does not match the strict V1 schema",
        ))
    })?;
    let descriptors = serde_json::from_str(texture_payload_descriptors_json).map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-TEXTURE-DESCRIPTORS-JSON-INVALID",
            "modelTexturePayloadDescriptorsJson",
            "model texture payload descriptors do not match the strict V1 schema",
        ))
    })?;
    Ok((separation, texture_authoring, descriptors))
}

fn build_meshy_procedural_humanoid_product_v2_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
) -> Result<StudioCreatureProductArtifactV2, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreatureProductIdentityV2,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-PRODUCT-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "product identity JSON does not match the strict V2 schema".to_owned(),
        })
    })?;
    let artifact = m2a_core::model_pipeline::build_meshy_procedural_humanoid_product_v2(
        source_glb,
        appearance_two_da,
        &identity,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_procedural_creature_product_v2(artifact)
}

#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidProductWithOptionsV3)]
pub fn build_meshy_procedural_humanoid_product_with_options_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
) -> Result<StudioCreatureProductArtifactV2, JsValue> {
    build_meshy_procedural_humanoid_product_with_options_v3_inner(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_product_with_options_v3_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
) -> Result<StudioCreatureProductArtifactV2, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreatureProductIdentityV2,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-PRODUCT-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "product identity JSON does not match the strict V2 schema".to_owned(),
        })
    })?;
    let build_options = parse_procedural_creature_build_options_v1(build_options_json)?;
    let artifact =
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_product_with_options_v3(
            source_glb,
            appearance_two_da,
            &identity,
            &build_options,
        )
        .map_err(|error| serialize_json(&error))?;
    finish_procedural_creature_product_v2(artifact)
}

/// Builds an immutable production creature and then wraps that exact product
/// in a separately identified demo MOD containing its module-local UTC.
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidProductDemoWithOptionsV1)]
pub fn build_meshy_procedural_humanoid_product_demo_with_options_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
) -> Result<StudioCreatureProductDemoArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_product_demo_internal_v2(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
        module_identity_json,
        creature_resref,
        None,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidProductDemoWithMaterialsV2)]
pub fn build_meshy_procedural_humanoid_product_demo_with_materials_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
) -> Result<StudioCreatureProductDemoArtifactV1, JsValue> {
    let (separation, textures, descriptors) = parse_creature_model_material_authoring_v2(
        material_separation_json,
        model_texture_authoring_json,
        texture_payload_descriptors_json,
    )
    .map_err(|error| JsValue::from_str(&error))?;
    build_meshy_procedural_humanoid_product_demo_internal_v2(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
        module_identity_json,
        creature_resref,
        Some(
            m2a_core::model_pipeline::CreatureModelMaterialAuthoringInputV2 {
                separation: separation.as_core_input(),
                textures: &textures,
                texture_payload_blob,
                texture_payload_descriptors: &descriptors,
            },
        ),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_procedural_humanoid_product_demo_internal_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
    material_authoring: Option<m2a_core::model_pipeline::CreatureModelMaterialAuthoringInputV2<'_>>,
) -> Result<StudioCreatureProductDemoArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreatureProductIdentityV2,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-PRODUCT-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "product identity JSON does not match the strict V2 schema".to_owned(),
        })
    })?;
    let module_identity = serde_json::from_str::<
        m2a_core::proof_module::BinaryCreatureModuleIdentityV1,
    >(module_identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-DEMO-IDENTITY-JSON".to_owned(),
            path: "moduleIdentityJson".to_owned(),
            message: "demo module identity JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let build_options = parse_procedural_creature_build_options_v1(build_options_json)?;
    let product = if let Some(material_authoring) = material_authoring {
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_product_with_materials_v5(
            source_glb,
            appearance_two_da,
            &identity,
            &build_options,
            material_authoring,
        )
    } else {
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_product_with_options_v3(
            source_glb,
            appearance_two_da,
            &identity,
            &build_options,
        )
    }
    .map_err(|error| serialize_json(&error))?;
    let demo = m2a_core::model_pipeline::build_procedural_creature_demo_with_authoring_v4(
        &product,
        &module_identity,
        creature_resref,
        &build_options.held_weapon,
        build_options.demo_authoring.as_ref(),
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&product.model).map_err(|error| serialize_json(&error))?;
    let (texture_payload_blob, texture_descriptors_json) =
        pack_model_texture_artifacts_v1(&product.material_textures);

    Ok(StudioCreatureProductDemoArtifactV1 {
        hak_bytes: product.hak,
        model_bytes: product.model,
        texture_bytes: product.texture,
        texture_payload_blob,
        texture_descriptors_json,
        proof_module_bytes: demo.payload,
        report_json: String::from_utf8(product.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(product.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(product.summary_json).map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
        demo_report_json: serialize_json(&demo.report),
    })
}

#[cfg(test)]
fn build_meshy_procedural_humanoid_product_demo_with_options_v1_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
) -> Result<StudioCreatureProductDemoArtifactV1, String> {
    build_meshy_procedural_humanoid_product_demo_internal_v2(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
        module_identity_json,
        creature_resref,
        None,
    )
}

/// Builds an exact full-native H1 package under caller-owned product/demo
/// identities. An empty event JSON preserves the source event policy; a
/// non-empty strict sidecar replaces only the event table.
#[wasm_bindgen(js_name = buildMeshyFullNativeH1PackageWithOptionsV4)]
pub fn build_meshy_full_native_h1_package_with_options_v4(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    event_authoring_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
) -> Result<StudioCreatureProductDemoArtifactV1, JsValue> {
    build_meshy_full_native_h1_package_internal_v5(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
        event_authoring_json,
        module_identity_json,
        creature_resref,
        None,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyFullNativeH1PackageWithMaterialsV5)]
pub fn build_meshy_full_native_h1_package_with_materials_v5(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    event_authoring_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
) -> Result<StudioCreatureProductDemoArtifactV1, JsValue> {
    let (separation, textures, descriptors) = parse_creature_model_material_authoring_v2(
        material_separation_json,
        model_texture_authoring_json,
        texture_payload_descriptors_json,
    )
    .map_err(|error| JsValue::from_str(&error))?;
    build_meshy_full_native_h1_package_internal_v5(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
        event_authoring_json,
        module_identity_json,
        creature_resref,
        Some(
            m2a_core::model_pipeline::CreatureModelMaterialAuthoringInputV2 {
                separation: separation.as_core_input(),
                textures: &textures,
                texture_payload_blob,
                texture_payload_descriptors: &descriptors,
            },
        ),
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_full_native_h1_package_internal_v5(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    event_authoring_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
    material_authoring: Option<m2a_core::model_pipeline::CreatureModelMaterialAuthoringInputV2<'_>>,
) -> Result<StudioCreatureProductDemoArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreatureProductIdentityV2,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-PRODUCT-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "product identity JSON does not match the strict V2 schema".to_owned(),
        })
    })?;
    let module_identity = serde_json::from_str::<
        m2a_core::proof_module::BinaryCreatureModuleIdentityV1,
    >(module_identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-DEMO-IDENTITY-JSON".to_owned(),
            path: "moduleIdentityJson".to_owned(),
            message: "demo module identity JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let build_options = parse_procedural_creature_build_options_v1(build_options_json)?;
    if module_identity.hak_resref != identity.hak_resref {
        return Err(serialize_json(
            &m2a_core::model_pipeline::M6PipelineErrorV1 {
                schema_version: 1,
                stage: "IDENTITY".to_owned(),
                code: "M6-DEMO-HAK-IDENTITY-MISMATCH".to_owned(),
                path: "moduleIdentityJson.hakResref".to_owned(),
                message: "demo HAK resref does not match the product HAK resref".to_owned(),
            },
        ));
    }
    let runtime_identity = m2a_core::model_pipeline::ProceduralCreaturePackageIdentityV1 {
        model_resref: identity.model_resref.clone(),
        texture_resref: identity.texture_resref.clone(),
        module: module_identity,
        creature_resref: creature_resref.to_owned(),
    };
    let event_authoring = if event_authoring_json.is_empty() {
        None
    } else {
        Some(
            serde_json::from_str::<m2a_core::model_pipeline::DirectCreatureEventAuthoringV1>(
                event_authoring_json,
            )
            .map_err(|_| {
                serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
                    schema_version: 1,
                    stage: "ANIMATION".to_owned(),
                    code: "M6-ANIMATION-EVENT-AUTHORING-JSON".to_owned(),
                    path: "eventAuthoringJson".to_owned(),
                    message: "event authoring JSON does not match the strict V1 schema".to_owned(),
                })
            })?,
        )
    };
    let event_configuration = event_authoring.as_ref().map(|authoring| {
        (
            m2a_core::model_pipeline::DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
            authoring,
        )
    });
    let package = if let Some(material_authoring) = material_authoring {
        m2a_core::model_pipeline::build_meshy_full_native_h1_package_with_materials_v6(
            source_glb,
            appearance_two_da,
            &runtime_identity,
            &identity,
            &build_options,
            event_configuration,
            material_authoring,
        )
    } else {
        m2a_core::model_pipeline::build_meshy_full_native_h1_package_with_options_v4(
            source_glb,
            appearance_two_da,
            &runtime_identity,
            &identity,
            &build_options,
            event_configuration,
        )
    }
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&package.model).map_err(|error| serialize_json(&error))?;
    let demo_report_json = serialize_json(&package.report.proof_module);
    let (texture_payload_blob, texture_descriptors_json) =
        pack_model_texture_artifacts_v1(&package.material_textures);

    Ok(StudioCreatureProductDemoArtifactV1 {
        hak_bytes: package.hak,
        model_bytes: package.model,
        texture_bytes: package.texture,
        texture_payload_blob,
        texture_descriptors_json,
        proof_module_bytes: package.proof_module,
        report_json: String::from_utf8(package.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(package.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(package.summary_json).map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
        demo_report_json,
    })
}

#[cfg(test)]
fn build_meshy_full_native_h1_package_with_options_v4_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
    event_authoring_json: &str,
    module_identity_json: &str,
    creature_resref: &str,
) -> Result<StudioCreatureProductDemoArtifactV1, String> {
    build_meshy_full_native_h1_package_internal_v5(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
        event_authoring_json,
        module_identity_json,
        creature_resref,
        None,
    )
}

fn finish_procedural_creature_product_v2(
    artifact: m2a_core::model_pipeline::ProceduralCreatureProductArtifactV3,
) -> Result<StudioCreatureProductArtifactV2, String> {
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;
    let (texture_payload_blob, texture_descriptors_json) =
        pack_model_texture_artifacts_v1(&artifact.material_textures);

    Ok(StudioCreatureProductArtifactV2 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        texture_bytes: artifact.texture,
        texture_payload_blob,
        texture_descriptors_json,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

fn parse_procedural_creature_build_options_v1(
    build_options_json: &str,
) -> Result<m2a_core::model_pipeline::ProceduralCreatureBuildOptionsV1, String> {
    serde_json::from_str(build_options_json).map_err(|_| {
        procedural_creature_json_error(
            "M6-PROCEDURAL-BUILD-OPTIONS-JSON",
            "buildOptionsJson",
            "procedural creature build options JSON does not match the strict V1 schema",
        )
    })
}

/// Validates a humanoid or quadruped MotionPack against the exact immutable
/// GLB node and clip inventory without building or mutating a model.
#[wasm_bindgen(js_name = inspectCreatureMotionPackSourceV1Json)]
pub fn inspect_creature_motion_pack_source_v1_json(
    source_glb: &[u8],
    motion_pack_json: &str,
) -> Result<String, String> {
    let pack: m2a_core::creature_product::CreatureMotionPackV1 =
        serde_json::from_str(motion_pack_json).map_err(|_| {
            procedural_creature_json_error(
                "CREATURE-MOTION-PACK-JSON",
                "motionPackJson",
                "motion pack JSON does not match the strict V1 schema",
            )
        })?;
    let ingest = m2a_core::glb::ingest_glb(source_glb, &m2a_core::glb::GlbLimits::default())
        .map_err(|error| serialize_json(&error))?;
    let joints = ingest
        .ir
        .nodes
        .iter()
        .filter_map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let clips = ingest
        .ir
        .animations
        .iter()
        .filter_map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    let report = m2a_core::creature_product::validate_creature_motion_pack_source_v1(
        &pack,
        &ingest.ir.source.sha256,
        &joints,
        &clips,
        None,
    )
    .map_err(|error| serialize_json(&error))?;
    Ok(serialize_json(&report))
}

fn procedural_creature_json_error(code: &str, path: &str, message: &str) -> String {
    serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
        schema_version: 1,
        stage: "TEXTURE".to_owned(),
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.to_owned(),
    })
}

/// Builds the caller-identified historical P100K Creature compatibility
/// profile through the browser boundary.
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidP100kExperimentV1)]
pub fn build_meshy_procedural_humanoid_p100k_experiment_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_p100k_experiment_v1_inner(
        source_glb,
        appearance_two_da,
        identity_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_p100k_experiment_v1_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreaturePackageIdentityV1,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-P100K-PACKAGE-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "package identity JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let artifact =
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_p100k_experiment_with_identity_v1(
            source_glb,
            appearance_two_da,
            &identity,
        )
        .map_err(|error| serialize_json(&error))?;
    finish_procedural_creature_package_v1(artifact)
}

#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2)]
pub fn build_meshy_procedural_humanoid_p100k_experiment_with_options_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_p100k_experiment_with_options_v2_inner(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_p100k_experiment_with_options_v2_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreaturePackageIdentityV1,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-P100K-PACKAGE-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "package identity JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let build_options = parse_procedural_creature_build_options_v1(build_options_json)?;
    let artifact =
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_p100k_experiment_with_options_v2(
            source_glb,
            appearance_two_da,
            &identity,
            &build_options,
        )
        .map_err(|error| serialize_json(&error))?;
    finish_procedural_creature_package_v1(artifact)
}

/// Builds the caller-identified historical P300K Creature compatibility
/// profile through the same browser boundary as P100K.
#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidP300kExperimentV1)]
pub fn build_meshy_procedural_humanoid_p300k_experiment_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_p300k_experiment_v1_inner(
        source_glb,
        appearance_two_da,
        identity_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_p300k_experiment_v1_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreaturePackageIdentityV1,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-P300K-PACKAGE-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "package identity JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let artifact =
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_p300k_experiment_with_identity_v1(
            source_glb,
            appearance_two_da,
            &identity,
        )
        .map_err(|error| serialize_json(&error))?;
    finish_procedural_creature_package_v1(artifact)
}

#[wasm_bindgen(js_name = buildMeshyProceduralHumanoidP300kExperimentWithOptionsV2)]
pub fn build_meshy_procedural_humanoid_p300k_experiment_with_options_v2(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, JsValue> {
    build_meshy_procedural_humanoid_p300k_experiment_with_options_v2_inner(
        source_glb,
        appearance_two_da,
        identity_json,
        build_options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_procedural_humanoid_p300k_experiment_with_options_v2_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    identity_json: &str,
    build_options_json: &str,
) -> Result<StudioP100kCreaturePackageArtifactV1, String> {
    let identity = serde_json::from_str::<
        m2a_core::model_pipeline::ProceduralCreaturePackageIdentityV1,
    >(identity_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "IDENTITY".to_owned(),
            code: "M6-P300K-PACKAGE-IDENTITY-JSON".to_owned(),
            path: "identityJson".to_owned(),
            message: "package identity JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let build_options = parse_procedural_creature_build_options_v1(build_options_json)?;
    let artifact =
        m2a_core::model_pipeline::build_meshy_procedural_humanoid_p300k_experiment_with_options_v2(
            source_glb,
            appearance_two_da,
            &identity,
            &build_options,
        )
        .map_err(|error| serialize_json(&error))?;
    finish_procedural_creature_package_v1(artifact)
}

fn finish_procedural_creature_package_v1(
    artifact: m2a_core::model_pipeline::M6ModelPackageArtifactV1,
) -> Result<StudioP100kCreaturePackageArtifactV1, String> {
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioP100kCreaturePackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        texture_bytes: artifact.texture,
        appearance_two_da_bytes: artifact.appearance_two_da,
        proof_module_bytes: artifact.proof_module,
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Builds the self-contained 42-state H1 profile with exact caller-owned
/// animation event tables. Event timings are parsed from the strict V1 JSON
/// sidecar and are never inferred from native witness assets.
#[wasm_bindgen(js_name = buildMeshyH1ModelPackageV3)]
pub fn build_meshy_h1_model_package_v3(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    event_authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_h1_model_package_v3_inner(source_glb, appearance_two_da, event_authoring_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn build_meshy_h1_model_package_v3_inner(
    source_glb: &[u8],
    appearance_two_da: &[u8],
    event_authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let event_authoring = serde_json::from_str::<
        m2a_core::model_pipeline::DirectCreatureEventAuthoringV1,
    >(event_authoring_json)
    .map_err(|_| {
        serialize_json(&m2a_core::model_pipeline::M6PipelineErrorV1 {
            schema_version: 1,
            stage: "ANIMATION".to_owned(),
            code: "M6-ANIMATION-EVENT-AUTHORING-JSON".to_owned(),
            path: "eventAuthoringJson".to_owned(),
            message: "event authoring JSON does not match the strict V1 schema".to_owned(),
        })
    })?;
    let artifact = m2a_core::model_pipeline::build_meshy_h1_model_package_v3(
        source_glb,
        appearance_two_da,
        m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        m2a_core::model_pipeline::DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
        &event_authoring,
    )
    .map_err(|error| serialize_json(&error))?;
    let readback =
        m2a_core::inspect_binary_mdl(&artifact.model).map_err(|error| serialize_json(&error))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        pwk_bytes: Vec::new(),
        texture_payload_blob: Vec::new(),
        texture_descriptors_json: "[]".to_owned(),
        report_json: String::from_utf8(artifact.report_json).map_err(|error| error.to_string())?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| error.to_string())?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| error.to_string())?,
        readback_json: serialize_json(&readback),
    })
}

/// Executes the separate M0 control lane for one static, unskinned Meshy GLB.
/// The returned HAK/MOD names are distinct from H1 and the MOD uses the
/// validated, self-contained M0 vertical-slice area.
#[wasm_bindgen(js_name = buildMeshyM0StaticRigidPackageV1)]
pub fn build_meshy_m0_static_rigid_package_v1(
    source_glb: &[u8],
    appearance_two_da: &[u8],
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    let artifact = m2a_core::model_pipeline::build_meshy_m0_static_rigid_package_v1(
        source_glb,
        appearance_two_da,
    )
    .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;
    let readback = m2a_core::inspect_binary_mdl(&artifact.model)
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))?;

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak,
        model_bytes: artifact.model,
        proof_module_bytes: artifact.proof_module,
        pwk_bytes: Vec::new(),
        texture_payload_blob: Vec::new(),
        texture_descriptors_json: "[]".to_owned(),
        report_json: String::from_utf8(artifact.report_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        manifest_json: String::from_utf8(artifact.manifest_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        summary_json: String::from_utf8(artifact.summary_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        readback_json: serialize_json(&readback),
    })
}

fn build_meshy_static_placeable_package_v1_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v1(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

fn build_meshy_static_placeable_package_v2_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV1,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v2(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

fn build_meshy_static_placeable_package_v3_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV1,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring schema",
        })
    })?;
    let options = serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(
        options_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict Placeable build-options schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v3(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

fn build_meshy_static_placeable_package_v4_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring V2 schema",
        })
    })?;
    let options = serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(
        options_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict Placeable build-options schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v4(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_static_placeable_package_v5_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&PlaceableBoundaryErrorV1 {
                    schema_version: 1,
                    code: "PLACEABLE-IDENTITY-JSON-INVALID",
                    path: "identityJson",
                    message: "identity JSON does not match the strict static placeable schema",
                })
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-PLACEMENT-JSON-INVALID",
            path: "placementJson",
            message: "placement JSON does not match the strict static placeable schema",
        })
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring V2 schema",
        })
    })?;
    let texture_authoring = serde_json::from_str::<
        m2a_core::placeable_texture::PlaceableTextureAuthoringDocumentV1,
    >(texture_authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-TEXTURE-AUTHORING-JSON-INVALID",
            path: "textureAuthoringJson",
            message: "texture authoring JSON does not match the strict V1 schema",
        })
    })?;
    let descriptors = serde_json::from_str::<
        Vec<m2a_core::placeable_texture::PlaceableTexturePayloadDescriptorV1>,
    >(texture_payload_descriptors_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-TEXTURE-PAYLOAD-DESCRIPTORS-JSON-INVALID",
            path: "texturePayloadDescriptorsJson",
            message: "texture payload descriptors JSON does not match the strict V1 schema",
        })
    })?;
    let options = serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(
        options_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict Placeable build-options schema",
        })
    })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v5(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &texture_authoring,
        texture_payload_blob,
        &descriptors,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_static_placeable_package_v6_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error(
                    "PLACEABLE-IDENTITY-JSON-INVALID",
                    "identityJson",
                    "identity JSON does not match the strict static Placeable schema",
                ))
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "PLACEABLE-PLACEMENT-JSON-INVALID",
            "placementJson",
            "placement JSON does not match the strict static Placeable schema",
        ))
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "PLACEABLE-AUTHORING-JSON-INVALID",
            "authoringJson",
            "authoring JSON does not match the strict Placeable V2 schema",
        ))
    })?;
    let material_separation = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV1,
    >(material_separation_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "materialSeparationJson",
            "material separation JSON does not match the strict V1 schema",
        ))
    })?;
    let texture_authoring = serde_json::from_str::<
        m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
    >(model_texture_authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-TEXTURE-DOCUMENT-JSON-INVALID",
            "modelTextureAuthoringJson",
            "model texture JSON does not match the strict V1 schema",
        ))
    })?;
    let descriptors = serde_json::from_str::<
        Vec<m2a_core::model_texture_authoring::ModelTexturePayloadDescriptorV1>,
    >(texture_payload_descriptors_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-TEXTURE-DESCRIPTORS-JSON-INVALID",
            "modelTexturePayloadDescriptorsJson",
            "model texture payload descriptors do not match the strict V1 schema",
        ))
    })?;
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error(
                    "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
                    "optionsJson",
                    "build options JSON does not match the strict Placeable schema",
                ))
            })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v6(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &material_separation,
        &texture_authoring,
        texture_payload_blob,
        &descriptors,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_static_placeable_package_v7_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "PLACEABLE-IDENTITY-JSON-INVALID",
                    "identityJson",
                    "identity JSON does not match the strict static Placeable schema",
                ))
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "PLACEABLE-PLACEMENT-JSON-INVALID",
            "placementJson",
            "placement JSON does not match the strict static Placeable schema",
        ))
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "PLACEABLE-AUTHORING-JSON-INVALID",
            "authoringJson",
            "authoring JSON does not match the strict Placeable V2 schema",
        ))
    })?;
    let material_separation = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV2,
    >(material_separation_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "materialSeparationJson",
            "material separation JSON does not match the strict Face Mode V2 schema",
        ))
    })?;
    let texture_authoring = serde_json::from_str::<
        m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
    >(model_texture_authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-TEXTURE-DOCUMENT-JSON-INVALID",
            "modelTextureAuthoringJson",
            "model texture JSON does not match the strict V1 schema",
        ))
    })?;
    let descriptors = serde_json::from_str::<
        Vec<m2a_core::model_texture_authoring::ModelTexturePayloadDescriptorV1>,
    >(texture_payload_descriptors_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-TEXTURE-DESCRIPTORS-JSON-INVALID",
            "modelTexturePayloadDescriptorsJson",
            "model texture payload descriptors do not match the strict V1 schema",
        ))
    })?;
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
                    "optionsJson",
                    "build options JSON does not match the strict Placeable schema",
                ))
            })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v7(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &material_separation,
        &texture_authoring,
        texture_payload_blob,
        &descriptors,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_static_placeable_package_v8_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    material_uv_projection_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "PLACEABLE-IDENTITY-JSON-INVALID",
                    "identityJson",
                    "identity JSON does not match the strict static Placeable schema",
                ))
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "PLACEABLE-PLACEMENT-JSON-INVALID",
            "placementJson",
            "placement JSON does not match the strict static Placeable schema",
        ))
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "PLACEABLE-AUTHORING-JSON-INVALID",
            "authoringJson",
            "authoring JSON does not match the strict Placeable V2 schema",
        ))
    })?;
    let material_separation = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV2,
    >(material_separation_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "materialSeparationJson",
            "material separation JSON does not match the strict Face Mode V2 schema",
        ))
    })?;
    let material_uv_projection = serde_json::from_str::<
        m2a_core::model_material_uv_projection::ModelMaterialUvProjectionDocumentV1,
    >(material_uv_projection_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-MATERIAL-UV-PROJECTION-JSON-INVALID",
            "materialUvProjectionJson",
            "material UV projection JSON does not match the strict V1 schema",
        ))
    })?;
    let texture_authoring = serde_json::from_str::<
        m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
    >(model_texture_authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-TEXTURE-DOCUMENT-JSON-INVALID",
            "modelTextureAuthoringJson",
            "model texture JSON does not match the strict V1 schema",
        ))
    })?;
    let descriptors = serde_json::from_str::<
        Vec<m2a_core::model_texture_authoring::ModelTexturePayloadDescriptorV1>,
    >(texture_payload_descriptors_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-TEXTURE-DESCRIPTORS-JSON-INVALID",
            "modelTexturePayloadDescriptorsJson",
            "model texture payload descriptors do not match the strict V1 schema",
        ))
    })?;
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
                    "optionsJson",
                    "build options JSON does not match the strict Placeable schema",
                ))
            })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v8(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &material_separation,
        &material_uv_projection,
        &texture_authoring,
        texture_payload_blob,
        &descriptors,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

#[allow(clippy::too_many_arguments)]
fn build_meshy_static_placeable_package_v9_inner(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    material_uv_projection_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    material_profile_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, String> {
    let identity =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableIdentityV1>(identity_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "PLACEABLE-IDENTITY-JSON-INVALID",
                    "identityJson",
                    "identity JSON does not match the strict static Placeable schema",
                ))
            })?;
    let placement = serde_json::from_str::<m2a_core::placeable::PlaceablePlacementV1>(
        placement_json,
    )
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "PLACEABLE-PLACEMENT-JSON-INVALID",
            "placementJson",
            "placement JSON does not match the strict static Placeable schema",
        ))
    })?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "PLACEABLE-AUTHORING-JSON-INVALID",
            "authoringJson",
            "authoring JSON does not match the strict Placeable V2 schema",
        ))
    })?;
    let material_separation = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV2,
    >(material_separation_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "materialSeparationJson",
            "material separation JSON does not match the strict Face Mode V2 schema",
        ))
    })?;
    let material_uv_projection = if material_uv_projection_json.trim().is_empty() {
        None
    } else {
        Some(
            serde_json::from_str::<
                m2a_core::model_material_uv_projection::ModelMaterialUvProjectionDocumentV1,
            >(material_uv_projection_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "MODEL-MATERIAL-UV-PROJECTION-JSON-INVALID",
                    "materialUvProjectionJson",
                    "material UV projection JSON does not match the strict V1 schema",
                ))
            })?,
        )
    };
    let material_profile = serde_json::from_str::<
        m2a_core::aurora_material::AuroraMaterialTargetProfileV1,
    >(material_profile_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error_v2(
            "AURORA-MATERIAL-PROFILE-JSON-INVALID",
            "materialProfileJson",
            "material profile must be AURORA_CLASSIC_SAFE or NWN_EE_MTR",
        ))
    })?;
    let texture_authoring = if model_texture_authoring_json.trim().is_empty() {
        None
    } else {
        Some(
            serde_json::from_str::<
                m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
            >(model_texture_authoring_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "MODEL-TEXTURE-DOCUMENT-JSON-INVALID",
                    "modelTextureAuthoringJson",
                    "model texture JSON does not match the strict V1 schema",
                ))
            })?,
        )
    };
    let descriptors = if texture_authoring.is_some() {
        serde_json::from_str::<
            Vec<m2a_core::model_texture_authoring::ModelTexturePayloadDescriptorV1>,
        >(texture_payload_descriptors_json)
        .map_err(|_| {
            serialize_json(&model_material_boundary_error_v2(
                "MODEL-TEXTURE-DESCRIPTORS-JSON-INVALID",
                "modelTexturePayloadDescriptorsJson",
                "model texture payload descriptors do not match the strict V1 schema",
            ))
        })?
    } else {
        Vec::new()
    };
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| {
                serialize_json(&model_material_boundary_error_v2(
                    "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
                    "optionsJson",
                    "build options JSON does not match the strict Placeable schema",
                ))
            })?;
    let artifact = m2a_core::placeable::build_meshy_static_placeable_package_v9(
        source_glb,
        placeables_two_da,
        &identity,
        placement,
        palette_id,
        &authoring,
        &material_separation,
        material_uv_projection.as_ref(),
        texture_authoring.as_ref(),
        texture_payload_blob,
        &descriptors,
        material_profile,
        &options,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_placeable_artifact_v1(artifact, &identity)
}

fn resolve_meshy_static_placeable_collision_v1_inner(
    source_glb: &[u8],
    model_resref: &str,
    authoring_json: &str,
    options_json: &str,
) -> Result<String, String> {
    let authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(authoring_json)
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-AUTHORING-JSON-INVALID",
            path: "authoringJson",
            message: "authoring JSON does not match the strict placeable authoring V2 schema",
        })
    })?;
    let options = serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(
        options_json,
    )
    .map_err(|_| {
        serialize_json(&PlaceableBoundaryErrorV1 {
            schema_version: 1,
            code: "PLACEABLE-BUILD-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict Placeable build-options schema",
        })
    })?;
    m2a_core::placeable::resolve_meshy_static_placeable_collision_v1(
        source_glb,
        model_resref,
        &authoring,
        &options,
    )
    .map(|resolved| serialize_json(&resolved))
    .map_err(|error| serialize_json(&error))
}

fn finish_static_placeable_artifact_v1(
    artifact: m2a_core::placeable::StaticPlaceablePackageArtifactV1,
    identity: &m2a_core::placeable::StaticPlaceableIdentityV1,
) -> Result<StudioModelPackageArtifactV1, String> {
    let hak = m2a_core::erf::ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| serialize_json(&error))?;
    let model = hak
        .find(
            &identity.model_resref,
            m2a_core::placeable::MDL_RESOURCE_TYPE,
        )
        .map_err(|error| serialize_json(&error))?
        .to_vec();
    let pwk = hak
        .find(
            &identity.model_resref,
            m2a_core::placeable::PWK_RESOURCE_TYPE,
        )
        .map_err(|error| serialize_json(&error))?
        .to_vec();
    let material_resource_reports = artifact
        .report
        .resources
        .iter()
        .filter(|resource| {
            resource.container == "HAK"
                && matches!(
                    resource.resource_type,
                    m2a_core::placeable::TGA_RESOURCE_TYPE
                        | m2a_core::placeable::DDS_RESOURCE_TYPE
                        | m2a_core::mtr::MTR_RESOURCE_TYPE_V1
                        | m2a_core::txi::TXI_RESOURCE_TYPE_V1
                )
        })
        .collect::<Vec<_>>();
    let mut texture_payload_blob = Vec::new();
    let mut texture_descriptors = Vec::with_capacity(material_resource_reports.len());
    for resource in material_resource_reports {
        let payload = hak
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| serialize_json(&error))?;
        let byte_offset = texture_payload_blob.len() as u64;
        texture_payload_blob.extend_from_slice(payload);
        texture_descriptors.push(PlaceableTextureArtifactDescriptorV1 {
            schema_version: 1,
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            byte_offset,
            byte_length: payload.len() as u64,
            sha256: resource.sha256.clone(),
        });
    }
    let readback = m2a_core::inspect_binary_mdl(&model).map_err(|error| serialize_json(&error))?;
    let report_json = serialize_json(&artifact.report);

    Ok(StudioModelPackageArtifactV1 {
        hak_bytes: artifact.hak_payload,
        model_bytes: model,
        proof_module_bytes: artifact.module_payload,
        pwk_bytes: pwk,
        texture_payload_blob,
        texture_descriptors_json: serialize_json(&texture_descriptors),
        report_json: report_json.clone(),
        manifest_json: report_json.clone(),
        summary_json: report_json,
        readback_json: serialize_json(&readback),
    })
}

/// Inspects one placeable GLB and returns the default immutable element
/// authoring document plus stable connected-component identities.
#[wasm_bindgen(js_name = inspectMeshyStaticPlaceableAuthoringV1)]
pub fn inspect_meshy_static_placeable_authoring_v1(source_glb: &[u8]) -> Result<String, JsValue> {
    m2a_core::placeable::inspect_meshy_static_placeable_authoring_v1(source_glb)
        .map(|bootstrap| serialize_json(&bootstrap))
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

#[wasm_bindgen(js_name = inspectMeshyStaticPlaceableAuthoringV2)]
pub fn inspect_meshy_static_placeable_authoring_v2(
    source_glb: &[u8],
    options_json: &str,
) -> Result<String, JsValue> {
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| JsValue::from_str("PLACEABLE-BUILD-OPTIONS-JSON-INVALID"))?;
    m2a_core::placeable::inspect_meshy_static_placeable_authoring_v2(source_glb, &options)
        .map(|bootstrap| serialize_json(&bootstrap))
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

#[wasm_bindgen(js_name = inspectMeshyStaticPlaceableAuthoringV3)]
pub fn inspect_meshy_static_placeable_authoring_v3(
    source_glb: &[u8],
    options_json: &str,
) -> Result<String, JsValue> {
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| JsValue::from_str("PLACEABLE-BUILD-OPTIONS-JSON-INVALID"))?;
    m2a_core::placeable::inspect_meshy_static_placeable_authoring_v3(source_glb, &options)
        .map(|bootstrap| serialize_json(&bootstrap))
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

#[wasm_bindgen(js_name = inspectMeshyStaticPlaceableTexturesV1)]
pub fn inspect_meshy_static_placeable_textures_v1(
    source_glb: &[u8],
    options_json: &str,
) -> Result<String, JsValue> {
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| JsValue::from_str("PLACEABLE-BUILD-OPTIONS-JSON-INVALID"))?;
    m2a_core::placeable::inspect_meshy_static_placeable_textures_v1(source_glb, &options)
        .map(|bootstrap| serialize_json(&bootstrap))
        .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

#[wasm_bindgen(js_name = resolveMeshyStaticPlaceableTexturesV1)]
pub fn resolve_meshy_static_placeable_textures_v1(
    source_glb: &[u8],
    base_texture_resref: &str,
    geometry_authoring_json: &str,
    texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    let geometry_authoring = serde_json::from_str::<
        m2a_core::placeable_authoring::PlaceableAuthoringDocumentV2,
    >(geometry_authoring_json)
    .map_err(|_| JsValue::from_str("PLACEABLE-AUTHORING-JSON-INVALID"))?;
    let authoring = serde_json::from_str::<
        m2a_core::placeable_texture::PlaceableTextureAuthoringDocumentV1,
    >(texture_authoring_json)
    .map_err(|_| JsValue::from_str("PLACEABLE-TEXTURE-AUTHORING-JSON-INVALID"))?;
    let descriptors = serde_json::from_str::<
        Vec<m2a_core::placeable_texture::PlaceableTexturePayloadDescriptorV1>,
    >(texture_payload_descriptors_json)
    .map_err(|_| JsValue::from_str("PLACEABLE-TEXTURE-PAYLOAD-DESCRIPTORS-JSON-INVALID"))?;
    let options =
        serde_json::from_str::<m2a_core::placeable::StaticPlaceableBuildOptionsV1>(options_json)
            .map_err(|_| JsValue::from_str("PLACEABLE-BUILD-OPTIONS-JSON-INVALID"))?;
    m2a_core::placeable::resolve_meshy_static_placeable_textures_v1(
        source_glb,
        base_texture_resref,
        &geometry_authoring,
        &authoring,
        texture_payload_blob,
        &descriptors,
        &options,
    )
    .map(|report| serialize_json(&report))
    .map_err(|error| JsValue::from_str(&serialize_json(&error)))
}

#[wasm_bindgen(js_name = resolveMeshyStaticPlaceableCollisionV1)]
pub fn resolve_meshy_static_placeable_collision_v1(
    source_glb: &[u8],
    model_resref: &str,
    authoring_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    resolve_meshy_static_placeable_collision_v1_inner(
        source_glb,
        model_resref,
        authoring_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Executes the static Meshy GLB -> common model IR -> placeable resource
/// resolver pipeline. HAK, MOD and model bytes are transferred separately;
/// the three JSON getters expose the same immutable machine-readable report.
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV1)]
pub fn build_meshy_static_placeable_package_v1(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v1_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Executes the authored placeable lane. The immutable authoring document is
/// applied before the shared Profile A conversion, while PWK uses the collision
/// projection of the same document.
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV2)]
pub fn build_meshy_static_placeable_package_v2(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v2_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Executes the authored Placeable lane with explicit experimental geometry
/// cleanup options. The option defaults to disabled in Studio.
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV3)]
pub fn build_meshy_static_placeable_package_v3(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v3_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV4)]
pub fn build_meshy_static_placeable_package_v4(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v4_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV5)]
pub fn build_meshy_static_placeable_package_v5(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v5_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        texture_authoring_json,
        texture_payload_blob,
        texture_payload_descriptors_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV6)]
pub fn build_meshy_static_placeable_package_v6(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v6_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        material_separation_json,
        model_texture_authoring_json,
        texture_payload_blob,
        texture_payload_descriptors_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV7)]
pub fn build_meshy_static_placeable_package_v7(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v7_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        material_separation_json,
        model_texture_authoring_json,
        texture_payload_blob,
        texture_payload_descriptors_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Executes Face Mode V2 with optional per-material render UV projection.
/// Collision authoring remains source-derived and is not modified.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV8)]
pub fn build_meshy_static_placeable_package_v8(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    material_uv_projection_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v8_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        material_separation_json,
        material_uv_projection_json,
        model_texture_authoring_json,
        texture_payload_blob,
        texture_payload_descriptors_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Executes the production Placeable material pipeline. The optional UV
/// projection argument is an empty string when no projection document is
/// selected. Material resources (TGA/MTR/TXI) are returned through the common
/// descriptor/blob channel.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = buildMeshyStaticPlaceablePackageV9)]
pub fn build_meshy_static_placeable_package_v9(
    source_glb: &[u8],
    placeables_two_da: &[u8],
    identity_json: &str,
    placement_json: &str,
    palette_id: u8,
    authoring_json: &str,
    material_separation_json: &str,
    material_uv_projection_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
    material_profile_json: &str,
    options_json: &str,
) -> Result<StudioModelPackageArtifactV1, JsValue> {
    build_meshy_static_placeable_package_v9_inner(
        source_glb,
        placeables_two_da,
        identity_json,
        placement_json,
        palette_id,
        authoring_json,
        material_separation_json,
        material_uv_projection_json,
        model_texture_authoring_json,
        texture_payload_blob,
        texture_payload_descriptors_json,
        material_profile_json,
        options_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Browser-transferable result for the complete static tile lane. Every
/// binary payload remains an owned byte buffer; JSON getters contain only
/// reports/readbacks and never filesystem paths.
#[wasm_bindgen]
pub struct StudioTilePackageArtifactV1 {
    hak_bytes: Vec<u8>,
    module_bytes: Vec<u8>,
    model_bytes: Vec<u8>,
    wok_bytes: Vec<u8>,
    set_bytes: Vec<u8>,
    texture_bytes: Vec<u8>,
    texture_payload_blob: Vec<u8>,
    texture_descriptors_json: String,
    image_map_bytes: Vec<u8>,
    report_json: String,
    model_readback_json: String,
    wok_readback_json: String,
    set_readback_json: String,
}

#[wasm_bindgen]
impl StudioTilePackageArtifactV1 {
    #[wasm_bindgen(js_name = takeHakBytes)]
    pub fn take_hak_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.hak_bytes)
    }

    #[wasm_bindgen(js_name = takeModuleBytes)]
    pub fn take_module_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.module_bytes)
    }

    #[wasm_bindgen(js_name = takeModelBytes)]
    pub fn take_model_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.model_bytes)
    }

    #[wasm_bindgen(js_name = takeWokBytes)]
    pub fn take_wok_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.wok_bytes)
    }

    #[wasm_bindgen(js_name = takeSetBytes)]
    pub fn take_set_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.set_bytes)
    }

    #[wasm_bindgen(js_name = takeTextureBytes)]
    pub fn take_texture_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_bytes)
    }

    #[wasm_bindgen(js_name = takeTexturePayloadBlob)]
    pub fn take_texture_payload_blob(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.texture_payload_blob)
    }

    #[wasm_bindgen(getter, js_name = textureDescriptorsJson)]
    pub fn texture_descriptors_json(&self) -> String {
        self.texture_descriptors_json.clone()
    }

    #[wasm_bindgen(js_name = takeImageMapBytes)]
    pub fn take_image_map_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.image_map_bytes)
    }

    #[wasm_bindgen(getter, js_name = reportJson)]
    pub fn report_json(&self) -> String {
        self.report_json.clone()
    }

    #[wasm_bindgen(getter, js_name = modelReadbackJson)]
    pub fn model_readback_json(&self) -> String {
        self.model_readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = wokReadbackJson)]
    pub fn wok_readback_json(&self) -> String {
        self.wok_readback_json.clone()
    }

    #[wasm_bindgen(getter, js_name = setReadbackJson)]
    pub fn set_readback_json(&self) -> String {
        self.set_readback_json.clone()
    }
}

fn build_meshy_static_tile_package_v1_inner(
    source_glb: &[u8],
    options_json: &str,
) -> Result<StudioTilePackageArtifactV1, String> {
    let options = serde_json::from_str::<StaticTileBoundaryOptionsV1>(options_json).map_err(|_| {
        serialize_json(&TileBoundaryErrorV1 {
            schema_version: 1,
            code: "TILE-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict StaticTileBoundaryOptionsV1 schema",
        })
    })?;
    if options.schema_version != 1 {
        return Err(serialize_json(&TileBoundaryErrorV1 {
            schema_version: 1,
            code: "TILE-OPTIONS-SCHEMA-INVALID",
            path: "optionsJson.schemaVersion",
            message: "tile boundary options must use schema version 1",
        }));
    }
    let artifact = m2a_core::tile::build_meshy_static_tile_package_v1(
        source_glb,
        &options.identity,
        options.interior,
        &options.terrain_name,
        options.surface,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_tile_artifact_v1(artifact)
}

fn finish_static_tile_artifact_v1(
    artifact: m2a_core::tile::StaticTilePackageArtifactV1,
) -> Result<StudioTilePackageArtifactV1, String> {
    let model_readback = m2a_core::inspect_binary_mdl(&artifact.mdl_payload)
        .map_err(|error| serialize_json(&error))?;
    let wok_readback = m2a_core::walkmesh::inspect_ascii_tile_wok_v1(&artifact.wok_payload)
        .map_err(|error| serialize_json(&error))?;
    let set_readback = m2a_core::tile::parse_tileset_v1(&artifact.set_payload)
        .map_err(|error| serialize_json(&error))?;
    let hak = m2a_core::erf::ErfArchive::parse(&artifact.hak_payload)
        .map_err(|error| serialize_json(&error))?;
    let mut texture_payload_blob = Vec::new();
    let mut texture_descriptors = Vec::new();
    for resource in artifact.report.resources.iter().filter(|resource| {
        resource.container == "HAK"
            && resource.resource_type == 3
            && resource.resref != artifact.report.image_map_resref
    }) {
        let payload = hak
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| serialize_json(&error))?;
        let byte_offset = texture_payload_blob.len() as u64;
        texture_payload_blob.extend_from_slice(payload);
        texture_descriptors.push(PlaceableTextureArtifactDescriptorV1 {
            schema_version: 1,
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            byte_offset,
            byte_length: payload.len() as u64,
            sha256: resource.sha256.clone(),
        });
    }
    Ok(StudioTilePackageArtifactV1 {
        hak_bytes: artifact.hak_payload,
        module_bytes: artifact.module_payload,
        model_bytes: artifact.mdl_payload,
        wok_bytes: artifact.wok_payload,
        set_bytes: artifact.set_payload,
        texture_bytes: artifact.texture_payload,
        texture_payload_blob,
        texture_descriptors_json: serialize_json(&texture_descriptors),
        image_map_bytes: artifact.image_map_payload,
        report_json: serialize_json(&artifact.report),
        model_readback_json: serialize_json(&model_readback),
        wok_readback_json: serialize_json(&wok_readback),
        set_readback_json: serialize_json(&set_readback),
    })
}

fn build_meshy_static_tile_package_v2_inner(
    source_glb: &[u8],
    options_json: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
) -> Result<StudioTilePackageArtifactV1, String> {
    let options = serde_json::from_str::<StaticTileBoundaryOptionsV1>(options_json).map_err(|_| {
        serialize_json(&TileBoundaryErrorV1 {
            schema_version: 1,
            code: "TILE-OPTIONS-JSON-INVALID",
            path: "optionsJson",
            message: "options JSON does not match the strict StaticTileBoundaryOptionsV1 schema",
        })
    })?;
    if options.schema_version != 1 {
        return Err(serialize_json(&TileBoundaryErrorV1 {
            schema_version: 1,
            code: "TILE-OPTIONS-SCHEMA-INVALID",
            path: "optionsJson.schemaVersion",
            message: "tile boundary options must use schema version 1",
        }));
    }
    let material_separation = serde_json::from_str::<
        m2a_core::model_material_separation::ModelMaterialSeparationDocumentV1,
    >(material_separation_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-MATERIAL-DOCUMENT-JSON-INVALID",
            "materialSeparationJson",
            "material separation JSON does not match the strict V1 schema",
        ))
    })?;
    let texture_authoring = serde_json::from_str::<
        m2a_core::model_texture_authoring::ModelTextureAuthoringDocumentV1,
    >(model_texture_authoring_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-TEXTURE-DOCUMENT-JSON-INVALID",
            "modelTextureAuthoringJson",
            "model texture JSON does not match the strict V1 schema",
        ))
    })?;
    let descriptors = serde_json::from_str::<
        Vec<m2a_core::model_texture_authoring::ModelTexturePayloadDescriptorV1>,
    >(texture_payload_descriptors_json)
    .map_err(|_| {
        serialize_json(&model_material_boundary_error(
            "MODEL-TEXTURE-DESCRIPTORS-JSON-INVALID",
            "modelTexturePayloadDescriptorsJson",
            "model texture payload descriptors do not match the strict V1 schema",
        ))
    })?;
    let artifact = m2a_core::tile::build_meshy_static_tile_package_v2(
        source_glb,
        &options.identity,
        options.interior,
        &options.terrain_name,
        options.surface,
        &material_separation,
        &texture_authoring,
        texture_payload_blob,
        &descriptors,
    )
    .map_err(|error| serialize_json(&error))?;
    finish_static_tile_artifact_v1(artifact)
}

/// Executes `GLB bytes + strict JSON options -> HAK/MOD/report/readbacks`
/// entirely inside Rust/WASM. No DOM, backend or filesystem is consulted.
#[wasm_bindgen(js_name = buildMeshyStaticTilePackageV1)]
pub fn build_meshy_static_tile_package_v1(
    source_glb: &[u8],
    options_json: &str,
) -> Result<StudioTilePackageArtifactV1, JsValue> {
    build_meshy_static_tile_package_v1_inner(source_glb, options_json)
        .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = buildMeshyStaticTilePackageV2)]
pub fn build_meshy_static_tile_package_v2(
    source_glb: &[u8],
    options_json: &str,
    material_separation_json: &str,
    model_texture_authoring_json: &str,
    texture_payload_blob: &[u8],
    texture_payload_descriptors_json: &str,
) -> Result<StudioTilePackageArtifactV1, JsValue> {
    build_meshy_static_tile_package_v2_inner(
        source_glb,
        options_json,
        material_separation_json,
        model_texture_authoring_json,
        texture_payload_blob,
        texture_payload_descriptors_json,
    )
    .map_err(|error| JsValue::from_str(&error))
}

/// Validates the strict, versioned M7 corpus manifest through `m2a-core`.
#[wasm_bindgen(js_name = validateM7CorpusManifestV1Json)]
pub fn validate_m7_corpus_manifest_v1_json(manifest_json: &str) -> String {
    match m2a_core::m7_corpus::parse_m7_corpus_manifest_v1(manifest_json.as_bytes()) {
        Ok(manifest) => serialize_json(&manifest),
        Err(error) => serialize_json(&error),
    }
}

/// Inspects M7 intake from borrowed slices of one browser-owned binary blob.
#[wasm_bindgen(js_name = inspectM7CorpusIntakeV1Json)]
pub fn inspect_m7_corpus_intake_v1_json(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> String {
    match inspect_m7_corpus_intake_v1_inner(manifest_json, payload_blob, descriptors_json) {
        Ok(report) => serialize_json(&report),
        Err(error) => error,
    }
}

/// Runs the existing canonical M7 batch and returns its report plus ordered
/// proof packets. Ready humanoids are materialized only by the canonical M6
/// constructor; the other routes remain explicit M7-V5 deferrals in core.
#[wasm_bindgen(js_name = buildM7CorpusBatchV1)]
pub fn build_m7_corpus_batch_v1(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> Result<String, JsValue> {
    build_m7_corpus_batch_v1_inner(manifest_json, payload_blob, descriptors_json)
        .map_err(|error| JsValue::from_str(&error))
}

fn parse_m7_boundary<'a>(
    manifest_json: &str,
    payload_blob: &'a [u8],
    descriptors_json: &'a str,
) -> Result<
    (
        m2a_core::m7_corpus::M7CorpusManifestV1,
        M7PayloadDescriptorsV1,
    ),
    String,
> {
    let manifest = m2a_core::m7_corpus::parse_m7_corpus_manifest_v1(manifest_json.as_bytes())
        .map_err(|error| serialize_json(&error))?;
    let descriptors: M7PayloadDescriptorsV1 =
        serde_json::from_str(descriptors_json).map_err(|_| {
            m7_boundary_error(
                "M7-WASM-DESCRIPTORS-JSON-INVALID",
                "descriptorsJson",
                "payload descriptors JSON does not match the public schema",
            )
        })?;
    if descriptors.schema_version != 1 {
        return Err(m7_boundary_error(
            "M7-WASM-DESCRIPTORS-SCHEMA-UNSUPPORTED",
            "descriptorsJson.schemaVersion",
            format!("expected schema 1, got {}", descriptors.schema_version),
        ));
    }
    validate_m7_blob_layout(payload_blob, &descriptors.payloads)?;
    validate_m7_descriptor_semantics(&manifest, &descriptors.payloads)?;
    Ok((manifest, descriptors))
}

fn inspect_m7_corpus_intake_v1_inner(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> Result<m2a_core::m7_corpus::M7CorpusIntakeReportV1, String> {
    let (manifest, descriptors) = parse_m7_boundary(manifest_json, payload_blob, descriptors_json)?;
    let payloads = m7_source_payloads(payload_blob, &descriptors.payloads)?;
    m2a_core::m7_corpus::inspect_m7_corpus_intake_v1(&manifest, &payloads)
        .map_err(|error| serialize_json(&error))
}

fn build_m7_corpus_batch_v1_inner(
    manifest_json: &str,
    payload_blob: &[u8],
    descriptors_json: &str,
) -> Result<String, String> {
    use m2a_core::m7_corpus::{M7CorpusEntryV1, M7IntakeStatusV1};

    let (manifest, descriptors) = parse_m7_boundary(manifest_json, payload_blob, descriptors_json)?;
    let payloads = m7_source_payloads(payload_blob, &descriptors.payloads)?;
    let intake = m2a_core::m7_corpus::inspect_m7_corpus_intake_v1(&manifest, &payloads)
        .map_err(|error| serialize_json(&error))?;
    let mut canonical_artifacts = Vec::new();
    if intake.status == M7IntakeStatusV1::ReadyForM7V5 {
        for entry in &manifest.samples {
            let M7CorpusEntryV1::RiggedHumanoidSourceClips {
                sample_id, source, ..
            } = entry
            else {
                continue;
            };
            if intake
                .samples
                .iter()
                .find(|sample| sample.sample_id == *sample_id)
                .is_none_or(|sample| sample.status != M7IntakeStatusV1::ReadyForM7V5)
            {
                continue;
            }
            let Some(source) = source else { continue };
            let source_bytes = payloads
                .iter()
                .find(|payload| {
                    payload
                        .relative_path
                        .eq_ignore_ascii_case(&source.relative_path)
                })
                .map(|payload| payload.bytes)
                .ok_or_else(|| {
                    m7_boundary_error(
                        "M7-WASM-SOURCE-PAYLOAD-MISSING",
                        "descriptorsJson.payloads",
                        format!("missing SOURCE descriptor for sample {sample_id:?}"),
                    )
                })?;
            let appearance_bytes =
                m7_appearance_payload(payload_blob, &descriptors.payloads, sample_id)?;
            canonical_artifacts.push(
                m2a_core::m7_corpus::M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6(
                    sample_id,
                    source_bytes,
                    appearance_bytes,
                )
                .map_err(|error| serialize_json(&error))?,
            );
        }
    }
    let artifact =
        m2a_core::m7_corpus::build_m7_corpus_batch_v1(&manifest, &payloads, &canonical_artifacts)
            .map_err(|error| serialize_json(&error))?;
    serialize_json_result(&M7BatchBoundaryOutputV1 {
        schema_version: 1,
        report: artifact.report,
        packets: artifact
            .packets
            .into_iter()
            .map(|packet| packet.packet)
            .collect(),
    })
}

fn validate_m7_blob_layout(
    payload_blob: &[u8],
    descriptors: &[M7PayloadDescriptorV1],
) -> Result<(), String> {
    let mut ranges = descriptors
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            let start = usize::try_from(descriptor.offset()).map_err(|_| {
                m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-INVALID",
                    format!("descriptorsJson.payloads[{index}].payloadOffset"),
                    "payload offset does not fit this platform",
                )
            })?;
            let size = usize::try_from(descriptor.size()).map_err(|_| {
                m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-INVALID",
                    format!("descriptorsJson.payloads[{index}].payloadSize"),
                    "payload size does not fit this platform",
                )
            })?;
            if size == 0 {
                return Err(m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-EMPTY",
                    format!("descriptorsJson.payloads[{index}].payloadSize"),
                    "payload descriptors must cover at least one byte",
                ));
            }
            let end = start.checked_add(size).ok_or_else(|| {
                m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-OVERFLOW",
                    format!("descriptorsJson.payloads[{index}]"),
                    "payload range overflows address space",
                )
            })?;
            if end > payload_blob.len() {
                return Err(m7_boundary_error(
                    "M7-WASM-PAYLOAD-RANGE-OOB",
                    format!("descriptorsJson.payloads[{index}]"),
                    format!(
                        "payload range ends at {end}, blob length is {}",
                        payload_blob.len()
                    ),
                ));
            }
            Ok((start, end, index))
        })
        .collect::<Result<Vec<_>, String>>()?;
    ranges.sort_unstable();
    let mut cursor = 0_usize;
    for (start, end, index) in ranges {
        if start != cursor {
            let code = if start < cursor {
                "M7-WASM-PAYLOAD-RANGE-OVERLAP"
            } else {
                "M7-WASM-PAYLOAD-RANGE-GAP"
            };
            return Err(m7_boundary_error(
                code,
                format!("descriptorsJson.payloads[{index}]"),
                format!("expected next payload offset {cursor}, got {start}"),
            ));
        }
        cursor = end;
    }
    if cursor != payload_blob.len() {
        return Err(m7_boundary_error(
            "M7-WASM-PAYLOAD-RANGE-GAP",
            "descriptorsJson.payloads",
            format!(
                "descriptors cover {cursor} of {} blob bytes",
                payload_blob.len()
            ),
        ));
    }
    Ok(())
}

fn validate_m7_descriptor_semantics(
    manifest: &m2a_core::m7_corpus::M7CorpusManifestV1,
    descriptors: &[M7PayloadDescriptorV1],
) -> Result<(), String> {
    use m2a_core::m7_corpus::M7CorpusEntryV1;

    let humanoid_sample_ids = manifest
        .samples
        .iter()
        .filter_map(|entry| match entry {
            M7CorpusEntryV1::RiggedHumanoidSourceClips { sample_id, .. } => Some(sample_id),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let all_sample_ids = manifest
        .samples
        .iter()
        .map(M7CorpusEntryV1::sample_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut appearances = std::collections::BTreeSet::new();
    for (index, descriptor) in descriptors.iter().enumerate() {
        let M7PayloadDescriptorV1::RiggedHumanoidAppearance2da { sample_id, .. } = descriptor
        else {
            continue;
        };
        if !all_sample_ids.contains(sample_id.as_str()) {
            return Err(m7_boundary_error(
                "M7-WASM-APPEARANCE-2DA-SAMPLE-UNKNOWN",
                format!("descriptorsJson.payloads[{index}].sampleId"),
                format!("appearance descriptor names unknown sample {sample_id:?}"),
            ));
        }
        if !humanoid_sample_ids.contains(sample_id) {
            return Err(m7_boundary_error(
                "M7-WASM-APPEARANCE-2DA-ROLE-MISMATCH",
                format!("descriptorsJson.payloads[{index}].sampleId"),
                format!("appearance descriptor targets non-humanoid sample {sample_id:?}"),
            ));
        }
        if !appearances.insert(sample_id.to_ascii_lowercase()) {
            return Err(m7_boundary_error(
                "M7-WASM-APPEARANCE-2DA-DUPLICATE",
                format!("descriptorsJson.payloads[{index}].sampleId"),
                format!("duplicate appearance descriptor for sample {sample_id:?}"),
            ));
        }
    }
    Ok(())
}

fn m7_source_payloads<'a>(
    payload_blob: &'a [u8],
    descriptors: &'a [M7PayloadDescriptorV1],
) -> Result<Vec<m2a_core::m7_corpus::M7SourcePayloadV1<'a>>, String> {
    let mut paths = std::collections::BTreeSet::new();
    descriptors
        .iter()
        .enumerate()
        .filter_map(|(index, descriptor)| match descriptor {
            M7PayloadDescriptorV1::Source {
                relative_path,
                payload_offset,
                payload_size,
            } => Some((index, relative_path, *payload_offset, *payload_size)),
            M7PayloadDescriptorV1::RiggedHumanoidAppearance2da { .. } => None,
        })
        .map(|(index, relative_path, offset, size)| {
            if !paths.insert(relative_path.to_ascii_lowercase()) {
                return Err(m7_boundary_error(
                    "M7-WASM-SOURCE-DESCRIPTOR-DUPLICATE",
                    format!("descriptorsJson.payloads[{index}].relativePath"),
                    "SOURCE relative paths must be unique case-insensitively",
                ));
            }
            let start = offset as usize;
            let end = start + size as usize;
            Ok(m2a_core::m7_corpus::M7SourcePayloadV1 {
                relative_path,
                bytes: &payload_blob[start..end],
            })
        })
        .collect()
}

fn m7_appearance_payload<'a>(
    payload_blob: &'a [u8],
    descriptors: &'a [M7PayloadDescriptorV1],
    sample_id: &str,
) -> Result<&'a [u8], String> {
    let matches = descriptors
        .iter()
        .filter_map(|descriptor| match descriptor {
            M7PayloadDescriptorV1::RiggedHumanoidAppearance2da {
                sample_id: candidate,
                payload_offset,
                payload_size,
            } if candidate == sample_id => Some((*payload_offset, *payload_size)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(m7_boundary_error(
            "M7-WASM-APPEARANCE-2DA-CARDINALITY",
            "descriptorsJson.payloads",
            format!(
                "ready humanoid sample {sample_id:?} requires exactly one RIGGED_HUMANOID_APPEARANCE_2DA descriptor, got {}",
                matches.len()
            ),
        ));
    }
    let (offset, size) = matches[0];
    let start = offset as usize;
    Ok(&payload_blob[start..start + size as usize])
}

fn m7_boundary_error(
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> String {
    serialize_json(&M7BoundaryErrorV1 {
        schema_version: 1,
        code,
        path: path.into(),
        message: message.into(),
    })
}

fn serialize_json_result<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|_| SERIALIZATION_ERROR_JSON.to_owned())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod m7_native_tests {
    use super::{
        M7BatchBoundaryOutputV1, M7PayloadDescriptorV1, M7PayloadDescriptorsV1,
        build_m7_corpus_batch_v1_inner, inspect_m7_corpus_intake_v1_inner, serialize_json,
        serialize_json_result, validate_m7_corpus_manifest_v1_json,
    };
    use m2a_core::m7_corpus::{
        M7ByteIdentityV1, M7CanonicalPipelineArtifactV1, M7CorpusEntryV1, M7CorpusManifestV1,
        M7OriginalSourceProvenanceV1, M7SourceDescriptorV1, M7SourcePayloadV1, M7SourceProviderV1,
        M7StaticResourceKindV1,
    };
    use sha2::{Digest, Sha256};

    const DEFERRED_MANIFEST: &str = r#"{
      "schemaVersion":1,
      "corpusId":"browser_corpus",
      "artDirectionApprovalId":null,
      "samples":[
        {"role":"RIGGED_HUMANOID_SOURCE_CLIPS","sampleId":"humanoid","source":null,"requiredSourceClipNames":["walk"]},
        {"role":"NON_HUMANOID_REFERENCE_SUPERMODEL","sampleId":"creature","source":null,"referenceSupermodel":"c_dog"},
        {"role":"STATIC_PLACEABLE_OR_ITEM","sampleId":"placeable","source":null,"resourceKind":"PLACEABLE"}
      ]
    }"#;
    const EMPTY_DESCRIPTORS: &str = r#"{"schemaVersion":1,"payloads":[]}"#;
    const READY_BATCH_JSON_SHA256: &str =
        "dbe8d7254dc6fedc0a2a3cd0f1f82f18aee8848de00b528accaf286d7f22988d";
    const APPEARANCE: &[u8] =
        include_bytes!("../../../apps/studio-web/tests/fixtures/appearance.2da");

    fn source_descriptor(path: &str, bytes: &[u8], task: &str) -> M7SourceDescriptorV1 {
        M7SourceDescriptorV1 {
            relative_path: path.to_owned(),
            identity: M7ByteIdentityV1 {
                byte_length: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(bytes)),
            },
            provenance: M7OriginalSourceProvenanceV1 {
                provider: M7SourceProviderV1::Meshy,
                provider_task_id: task.to_owned(),
                original_export_attested: true,
                rights_confirmed: true,
                not_synthetic_fixture_attested: true,
            },
        }
    }

    fn remove_rig_and_animations(glb: &[u8]) -> Vec<u8> {
        fn normalize_js_integer_numbers(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Number(number) => {
                    if number.is_f64()
                        && let Some(float) = number.as_f64()
                        && float.fract() == 0.0
                    {
                        if float >= 0.0 && float <= u64::MAX as f64 {
                            *number = serde_json::Number::from(float as u64);
                        } else if float >= i64::MIN as f64 && float <= i64::MAX as f64 {
                            *number = serde_json::Number::from(float as i64);
                        }
                    }
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        normalize_js_integer_numbers(value);
                    }
                }
                serde_json::Value::Object(values) => {
                    for value in values.values_mut() {
                        normalize_js_integer_numbers(value);
                    }
                }
                _ => {}
            }
        }

        let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_len;
        let mut json: serde_json::Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
        let root = json.as_object_mut().unwrap();
        root.remove("skins");
        root.remove("animations");
        for node in root["nodes"].as_array_mut().unwrap() {
            node.as_object_mut().unwrap().remove("skin");
        }
        normalize_js_integer_numbers(&mut json);
        let mut json_bytes = serde_json::to_vec(&json).unwrap();
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }
        let mut result = Vec::new();
        result.extend_from_slice(b"glTF");
        result.extend_from_slice(&2_u32.to_le_bytes());
        result.extend_from_slice(&0_u32.to_le_bytes());
        result.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(b"JSON");
        result.extend_from_slice(&json_bytes);
        result.extend_from_slice(&glb[json_end..]);
        let total_len = result.len() as u32;
        result[8..12].copy_from_slice(&total_len.to_le_bytes());
        result
    }

    fn ready_corpus() -> (M7CorpusManifestV1, Vec<u8>, Vec<u8>) {
        let humanoid = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let static_glb = remove_rig_and_animations(&humanoid);
        let manifest = M7CorpusManifestV1 {
            schema_version: 1,
            corpus_id: "m7-wasm-ready-fixture".to_owned(),
            art_direction_approval_id: Some("owned-test-approval".to_owned()),
            samples: vec![
                M7CorpusEntryV1::RiggedHumanoidSourceClips {
                    sample_id: "humanoid".to_owned(),
                    source: Some(source_descriptor(
                        "models/humanoid.glb",
                        &humanoid,
                        "task-h",
                    )),
                    required_source_clip_names: vec!["owned-linear-pause".to_owned()],
                },
                M7CorpusEntryV1::NonHumanoidReferenceSupermodel {
                    sample_id: "creature".to_owned(),
                    source: Some(source_descriptor(
                        "models/creature.glb",
                        &static_glb,
                        "task-c",
                    )),
                    reference_supermodel: "c_horror".to_owned(),
                },
                M7CorpusEntryV1::StaticPlaceableOrItem {
                    sample_id: "static-prop".to_owned(),
                    source: Some(source_descriptor(
                        "models/static.glb",
                        &static_glb,
                        "task-s",
                    )),
                    resource_kind: M7StaticResourceKindV1::Placeable,
                },
            ],
        };
        (manifest, humanoid, static_glb)
    }

    fn append_payload(
        blob: &mut Vec<u8>,
        payloads: &mut Vec<M7PayloadDescriptorV1>,
        role: impl FnOnce(u32, u32) -> M7PayloadDescriptorV1,
        bytes: &[u8],
    ) {
        let offset = blob.len() as u32;
        blob.extend_from_slice(bytes);
        payloads.push(role(offset, bytes.len() as u32));
    }

    fn ready_boundary(
        include_appearance: bool,
    ) -> (M7CorpusManifestV1, Vec<u8>, Vec<u8>, Vec<u8>, String) {
        let (manifest, humanoid, static_glb) = ready_corpus();
        let mut blob = Vec::new();
        let mut payloads = Vec::new();
        for (path, bytes) in [
            ("models/humanoid.glb", humanoid.as_slice()),
            ("models/creature.glb", static_glb.as_slice()),
            ("models/static.glb", static_glb.as_slice()),
        ] {
            append_payload(
                &mut blob,
                &mut payloads,
                |payload_offset, payload_size| M7PayloadDescriptorV1::Source {
                    relative_path: path.to_owned(),
                    payload_offset,
                    payload_size,
                },
                bytes,
            );
        }
        if include_appearance {
            append_payload(
                &mut blob,
                &mut payloads,
                |payload_offset, payload_size| M7PayloadDescriptorV1::RiggedHumanoidAppearance2da {
                    sample_id: "humanoid".to_owned(),
                    payload_offset,
                    payload_size,
                },
                APPEARANCE,
            );
        }
        let descriptors = serde_json::to_string(&M7PayloadDescriptorsV1 {
            schema_version: 1,
            payloads,
        })
        .unwrap();
        (manifest, humanoid, static_glb, blob, descriptors)
    }

    #[test]
    fn m7_deferred_boundary_is_exact_core_json_and_deterministic() {
        let manifest =
            m2a_core::m7_corpus::parse_m7_corpus_manifest_v1(DEFERRED_MANIFEST.as_bytes()).unwrap();
        assert_eq!(
            validate_m7_corpus_manifest_v1_json(DEFERRED_MANIFEST),
            serialize_json(&manifest)
        );
        let core = m2a_core::m7_corpus::inspect_m7_corpus_intake_v1(&manifest, &[]).unwrap();
        let adapter =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], EMPTY_DESCRIPTORS).unwrap();
        assert_eq!(serialize_json(&adapter), serialize_json(&core));

        let first =
            build_m7_corpus_batch_v1_inner(DEFERRED_MANIFEST, &[], EMPTY_DESCRIPTORS).unwrap();
        let second =
            build_m7_corpus_batch_v1_inner(DEFERRED_MANIFEST, &[], EMPTY_DESCRIPTORS).unwrap();
        assert_eq!(first, second);
        assert!(!first.to_ascii_lowercase().contains("base64"));
        let value: serde_json::Value = serde_json::from_str(&first).unwrap();
        assert_eq!(value["report"]["packetCount"], 3);
        assert_eq!(value["packets"].as_array().unwrap().len(), 3);
        assert_eq!(value["report"]["m7DoneClaimAllowed"], false);
    }

    #[test]
    fn m7_blob_descriptors_are_strict_checked_and_exact_covering() {
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], "{").unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-DESCRIPTORS-JSON-INVALID"
        );

        let unknown = r#"{"schemaVersion":1,"payloads":[],"extra":true}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], unknown).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-DESCRIPTORS-JSON-INVALID"
        );

        let oob = r#"{"schemaVersion":1,"payloads":[{"role":"SOURCE","relativePath":"source.glb","payloadOffset":0,"payloadSize":2}]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0], oob).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-OOB"
        );

        let gap = r#"{"schemaVersion":1,"payloads":[{"role":"SOURCE","relativePath":"source.glb","payloadOffset":1,"payloadSize":1}]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], gap).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-GAP"
        );

        let overlap = r#"{"schemaVersion":1,"payloads":[
          {"role":"SOURCE","relativePath":"a.glb","payloadOffset":0,"payloadSize":2},
          {"role":"SOURCE","relativePath":"b.glb","payloadOffset":1,"payloadSize":1}
        ]}"#;
        let error =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], overlap).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-OVERLAP"
        );

        let unsupported = r#"{"schemaVersion":2,"payloads":[]}"#;
        let error =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], unsupported).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-DESCRIPTORS-SCHEMA-UNSUPPORTED"
        );

        let zero = r#"{"schemaVersion":1,"payloads":[{"role":"SOURCE","relativePath":"source.glb","payloadOffset":0,"payloadSize":0}]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[], zero).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-PAYLOAD-RANGE-EMPTY"
        );

        let duplicate_source = r#"{"schemaVersion":1,"payloads":[
          {"role":"SOURCE","relativePath":"Models/Source.glb","payloadOffset":0,"payloadSize":1},
          {"role":"SOURCE","relativePath":"models/source.GLB","payloadOffset":1,"payloadSize":1}
        ]}"#;
        let error = inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], duplicate_source)
            .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-SOURCE-DESCRIPTOR-DUPLICATE"
        );
    }

    #[test]
    fn appearance_descriptors_are_semantic_and_never_ignored() {
        let descriptor = |sample_id: &str| {
            format!(
                r#"{{"schemaVersion":1,"payloads":[{{"role":"RIGGED_HUMANOID_APPEARANCE_2DA","sampleId":"{sample_id}","payloadOffset":0,"payloadSize":1}}]}}"#
            )
        };
        for (sample_id, code) in [
            ("missing", "M7-WASM-APPEARANCE-2DA-SAMPLE-UNKNOWN"),
            ("creature", "M7-WASM-APPEARANCE-2DA-ROLE-MISMATCH"),
        ] {
            let error =
                inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0], &descriptor(sample_id))
                    .unwrap_err();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
                code
            );
        }

        let duplicate = r#"{"schemaVersion":1,"payloads":[
          {"role":"RIGGED_HUMANOID_APPEARANCE_2DA","sampleId":"humanoid","payloadOffset":0,"payloadSize":1},
          {"role":"RIGGED_HUMANOID_APPEARANCE_2DA","sampleId":"humanoid","payloadOffset":1,"payloadSize":1}
        ]}"#;
        let error =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0, 1], duplicate).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-APPEARANCE-2DA-DUPLICATE"
        );

        let intake =
            inspect_m7_corpus_intake_v1_inner(DEFERRED_MANIFEST, &[0], &descriptor("humanoid"))
                .unwrap();
        assert_eq!(
            intake.status,
            m2a_core::m7_corpus::M7IntakeStatusV1::InputDeferred
        );
    }

    #[test]
    fn ready_owned_boundary_is_exact_native_batch_oracle_and_immutable() {
        let (manifest, humanoid, static_glb, blob, descriptors) = ready_boundary(true);
        let manifest_json = serialize_json(&manifest);
        let blob_before = blob.clone();
        let intake =
            inspect_m7_corpus_intake_v1_inner(&manifest_json, &blob, &descriptors).unwrap();
        assert_eq!(
            intake.status,
            m2a_core::m7_corpus::M7IntakeStatusV1::ReadyForM7V5
        );
        assert_eq!(blob, blob_before);

        let adapter = build_m7_corpus_batch_v1_inner(&manifest_json, &blob, &descriptors).unwrap();
        assert_eq!(blob, blob_before);
        let payloads = [
            M7SourcePayloadV1 {
                relative_path: "models/humanoid.glb",
                bytes: &humanoid,
            },
            M7SourcePayloadV1 {
                relative_path: "models/creature.glb",
                bytes: &static_glb,
            },
            M7SourcePayloadV1 {
                relative_path: "models/static.glb",
                bytes: &static_glb,
            },
        ];
        let canonical = [M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6(
            "humanoid", &humanoid, APPEARANCE,
        )
        .unwrap()];
        let native =
            m2a_core::m7_corpus::build_m7_corpus_batch_v1(&manifest, &payloads, &canonical)
                .unwrap();
        assert_eq!(native.report.materialized_packet_count, 1);
        assert_eq!(native.report.deferred_packet_count, 2);
        let expected = serialize_json_result(&M7BatchBoundaryOutputV1 {
            schema_version: 1,
            report: native.report,
            packets: native
                .packets
                .into_iter()
                .map(|packet| packet.packet)
                .collect(),
        })
        .unwrap();
        assert_eq!(adapter, expected);
        assert_eq!(
            format!("{:x}", Sha256::digest(adapter.as_bytes())),
            READY_BATCH_JSON_SHA256
        );
        assert_eq!(
            adapter,
            build_m7_corpus_batch_v1_inner(&manifest_json, &blob, &descriptors).unwrap()
        );
    }

    #[test]
    fn global_intake_gate_prevents_mixed_readiness_materialization() {
        let (manifest, humanoid, static_glb) = ready_corpus();
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        for (path, bytes) in [
            ("models/humanoid.glb", humanoid.as_slice()),
            ("models/creature.glb", static_glb.as_slice()),
        ] {
            append_payload(
                &mut blob,
                &mut descriptors,
                |payload_offset, payload_size| M7PayloadDescriptorV1::Source {
                    relative_path: path.to_owned(),
                    payload_offset,
                    payload_size,
                },
                bytes,
            );
        }
        append_payload(
            &mut blob,
            &mut descriptors,
            |payload_offset, payload_size| M7PayloadDescriptorV1::RiggedHumanoidAppearance2da {
                sample_id: "humanoid".to_owned(),
                payload_offset,
                payload_size,
            },
            APPEARANCE,
        );
        let descriptors = serde_json::to_string(&M7PayloadDescriptorsV1 {
            schema_version: 1,
            payloads: descriptors,
        })
        .unwrap();
        let output =
            build_m7_corpus_batch_v1_inner(&serialize_json(&manifest), &blob, &descriptors)
                .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["report"]["materializedPacketCount"], 0);
        assert_eq!(value["report"]["status"], "INPUT_DEFERRED");
    }

    #[test]
    fn invalid_intake_with_valid_appearance_preserves_core_diagnostics() {
        let (manifest, _, _, mut blob, descriptors) = ready_boundary(true);
        blob[0] ^= 0xff;
        let output =
            build_m7_corpus_batch_v1_inner(&serialize_json(&manifest), &blob, &descriptors)
                .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["report"]["status"], "INPUT_INVALID");
        assert_eq!(value["report"]["materializedPacketCount"], 0);
        let humanoid = value["packets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|packet| packet["sampleId"] == "humanoid")
            .unwrap();
        assert_eq!(humanoid["status"], "INPUT_INVALID");
        assert!(
            humanoid["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["code"] == "M7-SOURCE-IDENTITY-MISMATCH")
        );
    }

    #[test]
    fn ready_humanoid_requires_exactly_one_appearance_descriptor() {
        let (manifest, _, _, blob, descriptors) = ready_boundary(false);
        let error = build_m7_corpus_batch_v1_inner(&serialize_json(&manifest), &blob, &descriptors)
            .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M7-WASM-APPEARANCE-2DA-CARDINALITY"
        );
    }

    #[test]
    fn public_manifest_api_reports_malformed_unknown_and_unsupported_json() {
        for (manifest_json, code) in [
            ("{", "M7-MANIFEST-JSON-INVALID"),
            (
                r#"{"schemaVersion":2,"corpusId":"x","artDirectionApprovalId":null,"samples":[]}"#,
                "M7-MANIFEST-SCHEMA-UNSUPPORTED",
            ),
            (
                r#"{"schemaVersion":1,"corpusId":"x","artDirectionApprovalId":null,"samples":[],"unknown":true}"#,
                "M7-MANIFEST-JSON-INVALID",
            ),
        ] {
            let value: serde_json::Value =
                serde_json::from_str(&validate_m7_corpus_manifest_v1_json(manifest_json)).unwrap();
            assert_eq!(value["code"], code);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod m5_native_tests {
    use super::{
        HakResourceDescriptorV1, HakResourceDescriptorsV1, PlaceableTextureArtifactDescriptorV1,
        append_two_da_row_artifact_json, append_two_da_row_v1, append_two_da_row_v1_report_json,
        build_m6_model_package_v1, build_meshy_full_native_h1_package_with_options_v4_inner,
        build_meshy_h1_model_package_v2_inner, build_meshy_h1_model_package_v3_inner,
        build_meshy_procedural_humanoid_model_package_v1_inner,
        build_meshy_procedural_humanoid_p100k_experiment_v1_inner,
        build_meshy_procedural_humanoid_product_demo_with_options_v1_inner,
        build_meshy_procedural_humanoid_product_v2_inner,
        build_meshy_procedural_humanoid_product_with_options_v3_inner,
        build_meshy_static_placeable_package_v1_inner,
        build_meshy_static_placeable_package_v2_inner,
        build_meshy_static_placeable_package_v3_inner,
        build_meshy_static_placeable_package_v4_inner,
        build_meshy_static_placeable_package_v5_inner, build_meshy_static_tile_package_v1_inner,
        ingest_glb_json, ingest_meshy_p100k_experiment_json, inspect_two_da_v2_json,
        inspect_two_da_v2_json_inner, materialize_hak_resources,
        resolve_meshy_static_placeable_collision_v1_inner, serialize_json, write_hak_artifact_json,
        write_hak_v1, write_hak_v1_report_json, write_model_package_v1,
        write_model_package_v1_inner, write_package_manifest_v1_json,
        write_package_manifest_v1_json_inner, write_tga_artifact_json, write_tga_v1,
        write_tga_v1_report_json,
    };
    use m2a_core::hak::{HakResourceInputV1, HakWriterOptionsV1};
    use m2a_core::tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1};
    use m2a_core::two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
    };

    const DIRECT_CREATURE_APPEARANCE: &[u8] = b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY STRING_REF NAME WING_TAIL_SCALE HELMET_SCALE_M HELMET_SCALE_F WALKDIST RUNDIST PERSPACE CREPERSPACE HEIGHT HITDIST PREFATCKDIST TARGETHEIGHT ABORTONPARRY RACIALTYPE HASLEGS HASARMS PERCEPTIONDIST FOOTSTEPTYPE SOUNDAPPTYPE HEADTRACK HEAD_ARC_H HEAD_ARC_V HEAD_NAME BODY_BAG TARGETABLE\r\n0 Existing NORM S c_horror po_Horror **** 0 G **** 4 **** Hook_Horror 1 1 1 2.33 3.5 0.6 1 1 0.4 2.1 H 1 1 1 1 9 4 6 1 60 30 head 0 1\r\n";

    #[test]
    fn procedural_options_boundary_accepts_and_preserves_manual_weapon_grip_offsets() {
        let options = super::parse_procedural_creature_build_options_v1(
            r#"{"schemaVersion":1,"sourceForward":"POSITIVE_Z","textureArtifactCleanup":false,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"AUTO"},"weaponGrip":{"schemaVersion":1,"mode":"AUTO_PLUS_OFFSETS","rightHand":{"rollDegrees":12.5,"pitchDegrees":-3.0,"yawDegrees":7.0},"leftHand":{"rollDegrees":0.0,"pitchDegrees":0.0,"yawDegrees":0.0}}}"#,
        )
        .expect("strict RPY build options");
        assert_eq!(
            options.weapon_grip.mode,
            m2a_core::creature_equipment::CreatureWeaponGripModeV1::AutoPlusOffsets
        );
        assert_eq!(options.weapon_grip.right_hand.roll_degrees, 12.5);
        assert_eq!(options.weapon_grip.right_hand.pitch_degrees, -3.0);
        assert_eq!(options.weapon_grip.right_hand.yaw_degrees, 7.0);
    }

    #[test]
    fn procedural_options_boundary_carries_complete_creature_product_contracts() {
        use m2a_core::creature_product::{
            CreatureDemoAuthoringV1, CreatureMaterialProfileV2, CreatureMotionClipV1,
            CreatureMotionPackV1, CreaturePerformancePresetV1, CreatureRuntimeEnvelopePolicyV1,
        };
        let mut requested = m2a_core::model_pipeline::ProceduralCreatureBuildOptionsV1 {
            demo_authoring: Some(CreatureDemoAuthoringV1::active_monster_with_v10_bastard_sword()),
            runtime_envelope: CreatureRuntimeEnvelopePolicyV1::BoundsDerived,
            material_profile: CreatureMaterialProfileV2::nwn_ee_mtr(),
            performance_preset: CreaturePerformancePresetV1::High,
            ..Default::default()
        };
        requested.motion_pack = Some(CreatureMotionPackV1::humanoid_weapon_pack(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            vec![CreatureMotionClipV1::source("Attack", "c2a1", None)],
        ));
        let json = serde_json::to_string(&requested).unwrap();
        let parsed = super::parse_procedural_creature_build_options_v1(&json)
            .expect("complete Creature product options");
        assert_eq!(parsed, requested);
        assert_eq!(
            parsed.demo_authoring.unwrap().equipment_loadout.items[0].model_parts,
            [41, 11, 11]
        );
    }

    fn tga_image() -> TgaImageV1 {
        TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgb8,
            pixels: vec![255, 0, 128],
        }
    }

    fn static_owned_glb() -> Vec<u8> {
        let glb = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().expect("owned GLB fixture");
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_length;
        let mut root: serde_json::Value =
            serde_json::from_slice(&glb[20..json_end]).expect("GLB JSON");
        root["skins"] = serde_json::json!([]);
        root["animations"] = serde_json::json!([]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);
        root["nodes"] = serde_json::json!([{
            "name": "placeable-source-root",
            "mesh": 0
        }]);
        let attributes = root["meshes"][0]["primitives"][0]["attributes"]
            .as_object_mut()
            .expect("primitive attributes");
        attributes.remove("JOINTS_0");
        attributes.remove("WEIGHTS_0");

        let mut json = serde_json::to_vec(&root).expect("serialize GLB JSON");
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let mut result = Vec::new();
        result.extend_from_slice(b"glTF");
        result.extend_from_slice(&2_u32.to_le_bytes());
        result.extend_from_slice(&0_u32.to_le_bytes());
        result.extend_from_slice(&(json.len() as u32).to_le_bytes());
        result.extend_from_slice(b"JSON");
        result.extend_from_slice(&json);
        result.extend_from_slice(&glb[json_end..]);
        let total_length = result.len() as u32;
        result[8..12].copy_from_slice(&total_length.to_le_bytes());
        result
    }

    fn full_native_42_owned_glb() -> Vec<u8> {
        m2a_core::owned_fixture::synthetic_owned_m6_full_native_42_glb_v1()
            .expect("owned full-native-42 GLB fixture")
    }

    fn common_native_event_authoring() -> m2a_core::model_pipeline::DirectCreatureEventAuthoringV1 {
        use std::collections::BTreeMap;

        let mut clips = BTreeMap::<String, Vec<m2a_core::mdl::MdlAnimationEventV1>>::new();
        for (clip_name, event_name) in
            m2a_core::model_pipeline::COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1
        {
            clips.entry((*clip_name).to_owned()).or_default().push(
                m2a_core::mdl::MdlAnimationEventV1 {
                    time_seconds: 0.5,
                    name: (*event_name).to_owned(),
                },
            );
        }
        m2a_core::model_pipeline::DirectCreatureEventAuthoringV1 {
            schema_version: 1,
            clips: clips
                .into_iter()
                .map(|(clip_name, events)| {
                    m2a_core::model_pipeline::DirectCreatureClipEventAuthoringV1 {
                        clip_name,
                        events,
                    }
                })
                .collect(),
        }
    }

    fn placeables_two_da() -> Vec<u8> {
        b"2DA V2.0\n\nLabel StrRef ModelName LightColor LightOffsetX LightOffsetY LightOffsetZ SoundAppType ShadowSize BodyBag LowGore Reflection Static\n0 EXISTING **** plc_a01 **** **** **** **** **** 1 0 **** **** 1\n".to_vec()
    }

    fn placeable_identity() -> m2a_core::placeable::StaticPlaceableIdentityV1 {
        m2a_core::placeable::StaticPlaceableIdentityV1 {
            module_resref: "m2a_plc_mod".to_owned(),
            module_file_name: "m2a_plc_mod.mod".to_owned(),
            module_display_name: "Meshy2Aurora placeable proof".to_owned(),
            area_resref: "m2a_plc_area".to_owned(),
            area_name: "Meshy2Aurora placeable area".to_owned(),
            hak_resref: "m2a_plc_hak".to_owned(),
            hak_file_name: "m2a_plc_hak.hak".to_owned(),
            model_resref: "m2a_plc_ped".to_owned(),
            texture_resref: "m2a_plc_tex".to_owned(),
            blueprint_resref: "m2a_plc_utp".to_owned(),
            object_tag: "m2a_plc_pedestal".to_owned(),
            display_name: "Meshy Ritual Pedestal".to_owned(),
        }
    }

    #[test]
    fn placeable_boundary_rejects_non_schema_identity_before_materialization() {
        let error = match build_meshy_static_placeable_package_v1_inner(
            &[],
            &[],
            r#"{"schemaVersion":1,"unknown":true}"#,
            r#"{"x":10.0,"y":14.5,"z":0.0,"bearing":0.0}"#,
            7,
        ) {
            Ok(_) => panic!("strict identity JSON must fail"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("JSON error");
        assert_eq!(value["code"], "PLACEABLE-IDENTITY-JSON-INVALID");
        assert_eq!(value["path"], "identityJson");
    }

    #[test]
    fn placeable_boundary_is_byte_identical_to_the_core_package() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v1(
            &source, &table, &identity, placement, 7,
        )
        .expect("core placeable");
        let mut boundary = build_meshy_static_placeable_package_v1_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
        )
        .expect("WASM boundary placeable");

        assert_eq!(boundary.take_hak_bytes(), core.hak_payload);
        assert_eq!(boundary.take_proof_module_bytes(), core.module_payload);
        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(boundary.take_hak_bytes().is_empty());
        assert!(boundary.take_proof_module_bytes().is_empty());
        assert!(!boundary.take_model_bytes().is_empty());
        assert!(boundary.take_model_bytes().is_empty());
    }

    #[test]
    fn authored_placeable_boundary_is_byte_identical_to_the_core_package() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let bootstrap = m2a_core::placeable::inspect_meshy_static_placeable_authoring_v1(&source)
            .expect("authoring bootstrap");
        let mut document = bootstrap.document;
        document.elements[0].transform.scale = [2.0, 2.0, 2.0];
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v2(
            &source, &table, &identity, placement, 7, &document,
        )
        .expect("authored core placeable");
        let mut boundary = build_meshy_static_placeable_package_v2_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
            &serde_json::to_string(&document).unwrap(),
        )
        .expect("authored WASM boundary placeable");

        assert_eq!(boundary.take_hak_bytes(), core.hak_payload);
        assert_eq!(boundary.take_proof_module_bytes(), core.module_payload);
        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(core.report.authoring.is_some());
    }

    #[test]
    fn authored_placeable_v3_routes_the_explicit_aggressive_cleanup_option() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let document = m2a_core::placeable::inspect_meshy_static_placeable_authoring_v1(&source)
            .expect("authoring bootstrap")
            .document;
        let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1 {
            schema_version: 1,
            experimental_aggressive_geometry_cleanup: true,
        };
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v3(
            &source, &table, &identity, placement, 7, &document, &options,
        )
        .expect("aggressive opt-in core placeable");
        let boundary = build_meshy_static_placeable_package_v3_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
            &serde_json::to_string(&document).unwrap(),
            &serde_json::to_string(&options).unwrap(),
        )
        .expect("aggressive opt-in WASM boundary placeable");

        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(core.report.experimental_aggressive_geometry_cleanup);
    }

    #[test]
    fn authored_placeable_v4_custom_polygon_has_exact_resolver_and_pwk_bytes() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1::default();
        let mut document =
            m2a_core::placeable::inspect_meshy_static_placeable_authoring_v3(&source, &options)
                .expect("V2 authoring bootstrap")
                .document;
        document.collision.mode =
            m2a_core::placeable_authoring::PlaceableCollisionModeV1::CustomPolygon;
        document.collision.vertices = vec![
            [-0.5, -0.5],
            [0.5, -0.5],
            [0.5, 0.5],
            [0.0, 0.0],
            [-0.5, 0.5],
        ];
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v4(
            &source, &table, &identity, placement, 7, &document, &options,
        )
        .expect("custom polygon core placeable");
        let authoring_json = serde_json::to_string(&document).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let mut boundary = build_meshy_static_placeable_package_v4_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
            &authoring_json,
            &options_json,
        )
        .expect("custom polygon WASM placeable");
        let hak = m2a_core::erf::ErfArchive::parse(&core.hak_payload).expect("read HAK");
        let expected_pwk = hak
            .find(
                &identity.model_resref,
                m2a_core::placeable::PWK_RESOURCE_TYPE,
            )
            .expect("PWK");
        assert_eq!(boundary.take_pwk_bytes(), expected_pwk);
        assert!(boundary.take_pwk_bytes().is_empty());

        let resolved_json = resolve_meshy_static_placeable_collision_v1_inner(
            &source,
            &identity.model_resref,
            &authoring_json,
            &options_json,
        )
        .expect("WASM collision resolver");
        assert_eq!(
            resolved_json,
            serialize_json(core.report.collision.as_ref().expect("collision report"))
        );
    }

    #[test]
    fn authored_placeable_v5_exposes_exact_texture_payload_artifacts() {
        let source = static_owned_glb();
        let table = placeables_two_da();
        let identity = placeable_identity();
        let placement = m2a_core::placeable::PlaceablePlacementV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
            bearing: 0.0,
        };
        let options = m2a_core::placeable::StaticPlaceableBuildOptionsV1::default();
        let geometry =
            m2a_core::placeable::inspect_meshy_static_placeable_authoring_v3(&source, &options)
                .expect("geometry authoring")
                .document;
        let textures =
            m2a_core::placeable::inspect_meshy_static_placeable_textures_v1(&source, &options)
                .expect("texture authoring")
                .document;
        let core = m2a_core::placeable::build_meshy_static_placeable_package_v5(
            &source,
            &table,
            &identity,
            placement,
            7,
            &geometry,
            &textures,
            &[],
            &[],
            &options,
        )
        .expect("V5 core package");
        let mut boundary = build_meshy_static_placeable_package_v5_inner(
            &source,
            &table,
            &serde_json::to_string(&identity).unwrap(),
            &serde_json::to_string(&placement).unwrap(),
            7,
            &serde_json::to_string(&geometry).unwrap(),
            &serde_json::to_string(&textures).unwrap(),
            &[],
            "[]",
            &serde_json::to_string(&options).unwrap(),
        )
        .expect("V5 WASM boundary package");

        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        let blob = boundary.take_texture_payload_blob();
        assert!(!blob.is_empty());
        let descriptors: Vec<PlaceableTextureArtifactDescriptorV1> =
            serde_json::from_str(&boundary.texture_descriptors_json()).unwrap();
        assert_eq!(descriptors.len(), core.report.texture_resources.len());
        assert_eq!(descriptors[0].byte_length as usize, blob.len());
        assert!(boundary.take_texture_payload_blob().is_empty());
    }

    #[test]
    fn authored_placeable_boundary_rejects_non_schema_document() {
        let error = match build_meshy_static_placeable_package_v2_inner(
            &[],
            &[],
            &serde_json::to_string(&placeable_identity()).unwrap(),
            r#"{"x":10.0,"y":14.5,"z":0.0,"bearing":0.0}"#,
            7,
            r#"{"schemaVersion":1,"unknown":true}"#,
        ) {
            Ok(_) => panic!("strict authoring JSON must fail"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("JSON error");
        assert_eq!(value["code"], "PLACEABLE-AUTHORING-JSON-INVALID");
        assert_eq!(value["path"], "authoringJson");
    }

    #[test]
    fn tile_boundary_is_byte_identical_to_the_common_core_pipeline() {
        let source = static_owned_glb();
        let identity = m2a_core::tile::StaticTileIdentityV1::owner_candidate_v1();
        let core = m2a_core::tile::build_meshy_static_tile_package_v1(
            &source,
            &identity,
            false,
            "Grass",
            m2a_core::walkmesh::TileSurfaceV1::Grass,
        )
        .expect("core tile");
        let options = serde_json::json!({
            "schemaVersion": 1,
            "identity": identity,
            "interior": false,
            "terrainName": "Grass",
            "surface": "GRASS"
        })
        .to_string();
        let mut boundary =
            build_meshy_static_tile_package_v1_inner(&source, &options).expect("WASM tile");

        assert_eq!(boundary.take_hak_bytes(), core.hak_payload);
        assert_eq!(boundary.take_module_bytes(), core.module_payload);
        assert_eq!(boundary.take_model_bytes(), core.mdl_payload);
        assert_eq!(boundary.take_wok_bytes(), core.wok_payload);
        assert_eq!(boundary.take_set_bytes(), core.set_payload);
        assert_eq!(boundary.take_texture_bytes(), core.texture_payload);
        assert_eq!(boundary.take_image_map_bytes(), core.image_map_payload);
        assert_eq!(boundary.report_json(), serialize_json(&core.report));
        assert!(boundary.take_hak_bytes().is_empty());
        assert!(boundary.take_module_bytes().is_empty());
        assert!(boundary.take_model_bytes().is_empty());
        assert!(boundary.take_wok_bytes().is_empty());
        assert!(boundary.take_set_bytes().is_empty());
    }

    #[test]
    fn tile_boundary_rejects_unknown_options_before_reading_source() {
        let error = match build_meshy_static_tile_package_v1_inner(
            &[],
            r#"{"schemaVersion":1,"identity":{},"interior":false,"terrainName":"Grass","surface":"Grass","unknown":true}"#,
        ) {
            Ok(_) => panic!("strict tile options must fail"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("JSON error");
        assert_eq!(value["code"], "TILE-OPTIONS-JSON-INVALID");
        assert_eq!(value["path"], "optionsJson");
    }

    fn two_da_request() -> TwoDaAppendRequestV1 {
        TwoDaAppendRequestV1 {
            schema_version: 1,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "new".to_owned(),
                },
            }],
        }
    }

    fn hak_boundary(
        entries: &[(&str, u16, &[u8])],
    ) -> (Vec<u8>, HakResourceDescriptorsV1, Vec<HakResourceInputV1>) {
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        let mut resources = Vec::new();
        for &(resref, resource_type, payload) in entries {
            let payload_offset = blob.len() as u32;
            blob.extend_from_slice(payload);
            descriptors.push(HakResourceDescriptorV1 {
                resref: resref.to_owned(),
                resource_type,
                payload_offset,
                payload_size: payload.len() as u32,
            });
            resources.push(HakResourceInputV1 {
                resref: resref.to_owned(),
                resource_type,
                payload: payload.to_vec(),
            });
        }
        (
            blob,
            HakResourceDescriptorsV1 {
                schema_version: 1,
                resources: descriptors,
            },
            resources,
        )
    }

    fn package_entries() -> [(&'static str, u16, &'static [u8]); 3] {
        [
            ("texture", 3, b"tga"),
            ("appearance", 2017, b"2da"),
            ("model", 2002, b"mdl"),
        ]
    }

    #[test]
    fn tga_public_bytes_and_report_are_exact_core_and_inputs_are_immutable() {
        let image = tga_image();
        let options = TgaWriterOptionsV1::default();
        let image_json = serde_json::to_string(&image).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (image_json.clone(), options_json.clone());
        let core = m2a_core::tga::write_tga_v1(&image, &options).unwrap();

        assert_eq!(
            write_tga_v1(&image_json, &options_json).unwrap(),
            core.payload
        );
        assert_eq!(
            write_tga_v1_report_json(&image_json, &options_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((image_json, options_json), before);
    }

    #[test]
    fn two_da_public_inspect_append_and_report_are_exact_core_and_immutable() {
        let source = b"2DA V2.0\n\nA B\n0 old ****\n".to_vec();
        let request = two_da_request();
        let limits = TwoDaLimitsV1::default();
        let request_json = serde_json::to_string(&request).unwrap();
        let limits_json = serde_json::to_string(&limits).unwrap();
        let before = (source.clone(), request_json.clone(), limits_json.clone());

        let inspection = m2a_core::two_da::inspect_two_da_v2(&source, &limits).unwrap();
        assert_eq!(
            inspect_two_da_v2_json(&source, &limits_json).unwrap(),
            serde_json::to_string(&inspection).unwrap()
        );
        let core = m2a_core::two_da::append_two_da_row_v1(&source, &request, &limits).unwrap();
        assert_eq!(
            append_two_da_row_v1(&source, &request_json, &limits_json).unwrap(),
            core.payload
        );
        assert_eq!(
            append_two_da_row_v1_report_json(&source, &request_json, &limits_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((source, request_json, limits_json), before);
    }

    #[test]
    fn boundary_json_errors_are_frozen_strict_and_follow_argument_precedence() {
        let options_json = serde_json::to_string(&TgaWriterOptionsV1::default()).unwrap();
        let image_error = write_tga_artifact_json("{", &options_json).unwrap_err();
        assert_eq!(
            image_error,
            r#"{"schemaVersion":1,"code":"M5-TGA-IMAGE-JSON-INVALID","severity":"FATAL","path":"imageJson","message":"image JSON does not match the strict public schema"}"#
        );
        let mut image = serde_json::to_value(tga_image()).unwrap();
        image["unknown"] = serde_json::json!(true);
        assert_eq!(
            write_tga_artifact_json(&image.to_string(), &options_json).unwrap_err(),
            image_error
        );
        let options_error =
            write_tga_artifact_json(&serialize_json(&tga_image()), "{").unwrap_err();
        assert_eq!(
            options_error,
            r#"{"schemaVersion":1,"code":"M5-TGA-OPTIONS-JSON-INVALID","severity":"FATAL","path":"optionsJson","message":"TGA options JSON does not match the strict public schema"}"#
        );
        for path in ["unknown", "limits.unknown"] {
            let mut options = serde_json::to_value(TgaWriterOptionsV1::default()).unwrap();
            if path == "unknown" {
                options["unknown"] = serde_json::json!(true);
            } else {
                options["limits"]["unknown"] = serde_json::json!(true);
            }
            assert_eq!(
                write_tga_artifact_json(&serialize_json(&tga_image()), &options.to_string())
                    .unwrap_err(),
                options_error,
                "{path}"
            );
        }

        let source = b"2DA V2.0\n\nA\n";
        let request_error = append_two_da_row_artifact_json(source, "{", "{").unwrap_err();
        assert_eq!(
            request_error,
            r#"{"schemaVersion":1,"code":"M5-2DA-REQUEST-JSON-INVALID","severity":"FATAL","path":"requestJson","message":"2DA append request JSON does not match the strict public schema"}"#
        );
        let limits_error = inspect_two_da_v2_json_inner(source, "{").unwrap_err();
        assert_eq!(
            limits_error,
            r#"{"schemaVersion":1,"code":"M5-2DA-LIMITS-JSON-INVALID","severity":"FATAL","path":"limitsJson","message":"2DA limits JSON does not match the strict public schema"}"#
        );
        let mut limits = serde_json::to_value(TwoDaLimitsV1::default()).unwrap();
        limits["unknown"] = serde_json::json!(true);
        assert_eq!(
            inspect_two_da_v2_json_inner(source, &limits.to_string()).unwrap_err(),
            limits_error
        );
        for path in ["request", "cell", "value"] {
            let mut request = serde_json::to_value(two_da_request()).unwrap();
            match path {
                "request" => request["unknown"] = serde_json::json!(true),
                "cell" => request["cells"][0]["unknown"] = serde_json::json!(true),
                "value" => request["cells"][0]["value"]["unknown"] = serde_json::json!(true),
                _ => unreachable!(),
            }
            assert_eq!(
                append_two_da_row_artifact_json(
                    source,
                    &request.to_string(),
                    &serde_json::to_string(&TwoDaLimitsV1::default()).unwrap()
                )
                .unwrap_err(),
                request_error,
                "{path}"
            );
        }
    }

    #[test]
    fn core_errors_remain_exact_json_and_boundary_never_uses_base64() {
        let invalid_image = TgaImageV1 {
            width: 0,
            ..tga_image()
        };
        let options = TgaWriterOptionsV1::default();
        let direct = m2a_core::tga::write_tga_v1(&invalid_image, &options).unwrap_err();
        assert_eq!(
            write_tga_artifact_json(
                &serde_json::to_string(&invalid_image).unwrap(),
                &serde_json::to_string(&options).unwrap()
            )
            .unwrap_err(),
            serde_json::to_string(&direct).unwrap()
        );

        let limits = TwoDaLimitsV1::default();
        let direct = m2a_core::two_da::inspect_two_da_v2(b"bad\n", &limits).unwrap_err();
        let boundary =
            inspect_two_da_v2_json_inner(b"bad\n", &serde_json::to_string(&limits).unwrap())
                .unwrap_err();
        assert_eq!(boundary, super::two_da_core_error_json(&direct));
        assert_eq!(
            boundary,
            r#"{"schemaVersion":1,"code":"M5-2DA-HEADER-INVALID","severity":"FATAL","path":"header","message":"line 1 must be exactly 2DA V2.0"}"#
        );
        assert!(!boundary.contains("byteOffset"));

        let image_json = serde_json::to_string(&tga_image()).unwrap();
        let report =
            write_tga_v1_report_json(&image_json, &serde_json::to_string(&options).unwrap())
                .unwrap();
        assert!(!image_json.to_ascii_lowercase().contains("base64"));
        assert!(!report.to_ascii_lowercase().contains("base64"));
    }

    #[test]
    fn hak_and_package_match_core_are_deterministic_immutable_and_frozen() {
        let base_entries = package_entries();
        let (blob, descriptors, resources) = hak_boundary(&base_entries);
        let options = HakWriterOptionsV1::default();
        let resources_json = serde_json::to_string(&descriptors).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (blob.clone(), resources_json.clone(), options_json.clone());
        let core_hak = m2a_core::hak::write_hak_v1(&resources, &options).unwrap();
        let core_manifest =
            m2a_core::package::write_package_manifest_v1(&resources, &options).unwrap();

        assert_eq!(
            write_hak_v1(&blob, &resources_json, &options_json).unwrap(),
            core_hak.payload
        );
        assert_eq!(
            write_hak_v1_report_json(&blob, &resources_json, &options_json).unwrap(),
            serde_json::to_string(&core_hak.report).unwrap()
        );
        let manifest =
            write_package_manifest_v1_json(&blob, &resources_json, &options_json).unwrap();
        assert_eq!(manifest, serde_json::to_string(&core_manifest).unwrap());
        assert_eq!(
            manifest,
            "{\"schemaVersion\":1,\"packageSha256\":\"494862f6a12f91d5a269519d0579a05ace5bb50fd8f72b5711fcae7445444477\",\"resources\":[{\"role\":\"APPEARANCE_TABLE\",\"resref\":\"appearance\",\"type\":2017,\"byteLength\":3,\"sha256\":\"ddf81e9e4f364c6f086fd730b8f6d2bc4b46068045a085e1be8fc7470a615c6f\",\"hakResourceId\":0,\"hakPayloadOffset\":256},{\"role\":\"MODEL\",\"resref\":\"model\",\"type\":2002,\"byteLength\":3,\"sha256\":\"d3c3c54797643905c5cc97f7da4717058dbe6ad183ef1586104cadd197ca47c6\",\"hakResourceId\":1,\"hakPayloadOffset\":259},{\"role\":\"TEXTURE\",\"resref\":\"texture\",\"type\":3,\"byteLength\":3,\"sha256\":\"9dedca90fc9c44caeb39e0a6b8d28a157105bfba113872846ce0b2f5eff923d3\",\"hakResourceId\":2,\"hakPayloadOffset\":262}]}"
        );
        assert_eq!((blob, resources_json, options_json.clone()), before);
        assert!(!manifest.to_ascii_lowercase().contains("base64"));

        let expected = (core_hak, manifest);
        for order in [[2, 0, 1], [1, 2, 0]] {
            let entries = order.map(|index| base_entries[index]);
            let (blob, descriptors, _) = hak_boundary(&entries);
            let resources_json = serde_json::to_string(&descriptors).unwrap();
            assert_eq!(
                write_hak_artifact_json(&blob, &resources_json, &options_json).unwrap(),
                expected.0
            );
            assert_eq!(
                write_package_manifest_v1_json_inner(&blob, &resources_json, &options_json)
                    .unwrap(),
                expected.1
            );
        }
    }

    #[test]
    fn model_package_adapter_returns_one_core_artifact_with_native_parity() {
        let base_entries = package_entries();
        let (blob, descriptors, resources) = hak_boundary(&base_entries);
        let options = HakWriterOptionsV1::default();
        let resources_json = serde_json::to_string(&descriptors).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (blob.clone(), resources_json.clone(), options_json.clone());
        let core = m2a_core::package::write_model_package_v1(&resources, &options).unwrap();

        let mut wasm = write_model_package_v1_inner(&blob, &resources_json, &options_json).unwrap();
        assert_eq!(
            wasm.report_json(),
            serde_json::to_string(&core.hak.report).unwrap()
        );
        assert_eq!(
            wasm.manifest_json(),
            serde_json::to_string(&core.manifest).unwrap()
        );
        assert_eq!((blob, resources_json, options_json), before);
        assert!(!wasm.report_json().to_ascii_lowercase().contains("base64"));
        assert!(!wasm.manifest_json().to_ascii_lowercase().contains("base64"));
        assert_eq!(wasm.take_hak_bytes(), core.hak.payload);
        assert!(wasm.take_hak_bytes().is_empty());

        for order in [[2, 0, 1], [1, 2, 0]] {
            let entries = order.map(|index| base_entries[index]);
            let (blob, descriptors, _) = hak_boundary(&entries);
            let resources_json = serde_json::to_string(&descriptors).unwrap();
            let mut candidate =
                write_model_package_v1_inner(&blob, &resources_json, &before.2).unwrap();
            assert_eq!(candidate.manifest_json(), wasm.manifest_json());
            assert_eq!(candidate.take_hak_bytes(), core.hak.payload);
            assert!(candidate.take_hak_bytes().is_empty());
        }

        let mut public = write_model_package_v1(&before.0, &before.1, &before.2).unwrap();
        assert_eq!(public.manifest_json(), wasm.manifest_json());
        assert_eq!(public.take_hak_bytes(), core.hak.payload);
        assert!(public.take_hak_bytes().is_empty());
    }

    #[test]
    fn studio_model_package_adapter_is_exact_core_and_transfers_each_binary_once() {
        let source = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let core =
            m2a_core::model_pipeline::build_m6_model_package_v1(&source, appearance).unwrap();
        let mut studio = build_m6_model_package_v1(&source, appearance).unwrap();

        assert_eq!(
            studio.report_json(),
            String::from_utf8(core.report_json.clone()).unwrap()
        );
        assert_eq!(
            studio.manifest_json(),
            String::from_utf8(core.manifest_json.clone()).unwrap()
        );
        assert_eq!(
            studio.summary_json(),
            String::from_utf8(core.summary_json.clone()).unwrap()
        );
        assert_eq!(
            studio.readback_json(),
            serialize_json(&m2a_core::inspect_binary_mdl(&core.model).unwrap())
        );
        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert!(studio.take_hak_bytes().is_empty());
        assert_eq!(studio.take_model_bytes(), core.model);
        assert!(studio.take_model_bytes().is_empty());
    }

    #[test]
    fn studio_full_native_h1_v2_is_exact_core_and_fails_closed_without_42_source_clips() {
        let source = full_native_42_owned_glb();
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v2(
            &source,
            appearance,
            m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
        )
        .expect("core full H1");
        let mut studio =
            build_meshy_h1_model_package_v2_inner(&source, appearance).expect("Studio full H1");
        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        assert_eq!(
            studio.readback_json(),
            serialize_json(&m2a_core::inspect_binary_mdl(&core.model).unwrap())
        );

        let idle_only = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let error = match build_meshy_h1_model_package_v2_inner(&idle_only, appearance) {
            Ok(_) => panic!("full Studio path must reject one idle source"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).expect("structured error");
        assert_eq!(value["code"], "M6-ANIMATION-FULL-PROFILE-MISSING");
    }

    #[test]
    fn studio_h2_procedural_lane_materializes_the_owned_single_idle_source_as_full_42() {
        let source = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../sample-3d/h2-clockwork-sentinel-1500/source.glb"),
        )
        .expect("owned H2 GLB");
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v2(
            &source,
            appearance,
            m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1,
        )
        .expect("core procedural H2");
        let mut studio =
            build_meshy_procedural_humanoid_model_package_v1_inner(&source, appearance)
                .expect("Studio procedural H2");

        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        let report: serde_json::Value =
            serde_json::from_str(&studio.report_json()).expect("report JSON");
        assert_eq!(
            report["animationCompleteness"]["profile"],
            "FULL_NATIVE42_PROCEDURAL_HUMANOID_V1"
        );
        assert_eq!(report["animationCompleteness"]["schemaVersion"], 2);
        assert_eq!(report["animationCompleteness"]["requiredClipCount"], 42);
        assert_eq!(report["animationCompleteness"]["inputSourceClipCount"], 1);
        assert_eq!(
            report["animationCompleteness"]["preservedSourceClipCount"],
            1
        );
        assert_eq!(report["animationCompleteness"]["sourceDerivedClipCount"], 0);
        assert_eq!(report["animationCompleteness"]["proceduralClipCount"], 41);
        assert_eq!(
            report["animationCompleteness"]["discardedSourceClipCount"],
            0
        );
        assert_eq!(report["animationCompleteness"]["fallbackAliasCount"], 0);
        assert_eq!(report["animationBehavior"]["schemaVersion"], 2);
        assert_eq!(
            report["animationBehavior"]["behaviorCandidateEligible"],
            true
        );
        assert_eq!(report["animationKinematicsConformance"]["schemaVersion"], 2);
        assert_eq!(
            report["animationKinematicsConformance"]["allTracksStartAtZero"],
            true
        );
        assert_eq!(
            report["animationKinematicsConformance"]["requiredTransitionBoundariesContinuous"],
            true
        );
        assert_eq!(
            report["animationKinematicsConformance"]["locomotionRootMotionInPlace"],
            true
        );
        assert_eq!(report["animationKinematicsConformance"]["complete"], true);
        assert_eq!(report["animationEventConformance"]["complete"], true);
        assert_eq!(report["animationEventTimingPolicy"], "KINEMATIC_PEAK_V2");
        assert_eq!(report["skinAnimationConformance"]["complete"], true);
    }

    #[test]
    fn studio_procedural_v3_uses_fresh_product_identity_and_has_no_module_surface() {
        let source = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../sample-3d/h2-clockwork-sentinel-1500/source.glb"),
        )
        .expect("owned H2 GLB");
        let identity_json = r#"{
            "modelResref":"m2a_stmdl2",
            "textureResref":"m2a_sttex2",
            "hakResref":"m2a_sthak2",
            "appearanceLabel":"M2A_STUDIO_CREATURE_V2"
        }"#;
        let identity = serde_json::from_str::<
            m2a_core::model_pipeline::ProceduralCreatureProductIdentityV2,
        >(identity_json)
        .unwrap();
        let core = m2a_core::model_pipeline::build_meshy_procedural_humanoid_product_v2(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            &identity,
        )
        .expect("core product");
        let mut studio = build_meshy_procedural_humanoid_product_v2_inner(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            identity_json,
        )
        .expect("Studio product");

        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        assert_eq!(studio.take_texture_bytes(), core.texture);
        assert!(studio.take_texture_bytes().is_empty());
        let summary: serde_json::Value =
            serde_json::from_str(&studio.summary_json()).expect("summary JSON");
        assert_eq!(summary["schemaVersion"], 3);
        assert_eq!(
            summary["status"],
            "PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED"
        );
        assert_eq!(summary["identity"]["modelResref"], "m2a_stmdl2");
        assert!(summary["outputs"].get("proofModule").is_none());
        let manifest: serde_json::Value =
            serde_json::from_str(&studio.manifest_json()).expect("manifest JSON");
        assert!(
            manifest["generatedFiles"]
                .as_array()
                .unwrap()
                .iter()
                .all(|file| !file["relativePath"].as_str().unwrap().ends_with(".mod"))
        );

        let cleaned = build_meshy_procedural_humanoid_product_with_options_v3_inner(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            identity_json,
            r#"{"schemaVersion":1,"textureArtifactCleanup":true,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"KEEP_SOURCE_WEIGHTS"}}"#,
        )
        .expect("Studio product with texture cleanup");
        let cleaned_report: serde_json::Value =
            serde_json::from_str(&cleaned.report_json()).expect("cleaned report JSON");
        assert_eq!(cleaned_report["textureArtifactCleanup"]["enabled"], true);
        assert!(
            cleaned_report["textureArtifactCleanup"]["inspectedPixelCount"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(
            cleaned_report["skinAccessoryStabilization"]["mode"],
            "KEEP_SOURCE_WEIGHTS"
        );

        let mut studio_demo =
            build_meshy_procedural_humanoid_product_demo_with_options_v1_inner(
                &source,
                DIRECT_CREATURE_APPEARANCE,
                identity_json,
                r#"{"schemaVersion":1,"textureArtifactCleanup":false,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"AUTO"}}"#,
                r#"{"moduleResref":"m2a_stdemo2","areaResref":"m2a_starea2","hakResref":"m2a_sthak2"}"#,
                "m2a_stutc2",
            )
            .expect("Studio product plus optional demo");
        assert_eq!(studio_demo.take_hak_bytes(), core.hak);
        assert_eq!(studio_demo.take_model_bytes(), core.model);
        assert_eq!(studio_demo.take_texture_bytes(), core.texture);
        let demo_report: serde_json::Value =
            serde_json::from_str(&studio_demo.demo_report_json()).expect("demo report JSON");
        assert_eq!(demo_report["moduleResref"], "m2a_stdemo2");
        assert_eq!(demo_report["areaResref"], "m2a_starea2");
        assert_eq!(demo_report["creatureResref"], "m2a_stutc2");
        assert_eq!(demo_report["hakResref"], "m2a_sthak2");
        assert_eq!(demo_report["semanticReadbackStatus"], "PASS");
        assert!(!studio_demo.take_proof_module_bytes().is_empty());
        assert!(studio_demo.take_proof_module_bytes().is_empty());

        let mut held_demo =
            build_meshy_procedural_humanoid_product_demo_with_options_v1_inner(
                &source,
                DIRECT_CREATURE_APPEARANCE,
                identity_json,
                r#"{"schemaVersion":1,"textureArtifactCleanup":false,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"AUTO"},"heldWeapon":{"schemaVersion":1,"mode":"LEFT_HAND","itemResref":"nw_wswss001"}}"#,
                r#"{"moduleResref":"m2a_stdemo2","areaResref":"m2a_starea2","hakResref":"m2a_sthak2"}"#,
                "m2a_stutc2",
            )
            .expect("Studio product plus held-item demo");
        let held_report: serde_json::Value =
            serde_json::from_str(&held_demo.demo_report_json()).expect("held demo report JSON");
        assert_eq!(
            held_report["heldStockWeaponReadback"]["weapon"]["resref"],
            "nw_wswss001"
        );
        assert_eq!(
            held_report["heldStockWeaponReadback"]["fixtures"][0]["hand"],
            "left_hand"
        );
        assert_eq!(
            held_report["heldStockWeaponReadback"]["fixtures"][0]["equippedItemResref"],
            "nw_wswss001"
        );
        assert!(!held_demo.take_proof_module_bytes().is_empty());

        let options_error = match build_meshy_procedural_humanoid_product_with_options_v3_inner(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            identity_json,
            r#"{"schemaVersion":1,"textureArtifactCleanup":true,"unknown":1}"#,
        ) {
            Ok(_) => panic!("unknown texture options must fail closed"),
            Err(error) => error,
        };
        let options_value: serde_json::Value = serde_json::from_str(&options_error).unwrap();
        assert_eq!(options_value["code"], "M6-PROCEDURAL-BUILD-OPTIONS-JSON");

        let error = match build_meshy_procedural_humanoid_product_v2_inner(
            &source,
            DIRECT_CREATURE_APPEARANCE,
            r#"{"modelResref":"m2a_stmdl2"}"#,
        ) {
            Ok(_) => panic!("partial identity must fail closed"),
            Err(error) => error,
        };
        let value: serde_json::Value = serde_json::from_str(&error).unwrap();
        assert_eq!(value["code"], "M6-PRODUCT-IDENTITY-JSON");
    }

    #[test]
    fn studio_p100k_boundary_replays_the_frozen_candidate_byte_for_byte_when_requested() {
        if std::env::var_os("M2A_REQUIRE_P100K_STUDIO_REPLAY").is_none() {
            return;
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let source =
            std::fs::read(root.join("sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb"))
                .expect("canonical P100K Meshy source");
        let appearance = std::fs::read(root.join("local-reference-assets/appearance.2da"))
            .expect("canonical full appearance table");
        let identity_json = r#"{
            "modelResref":"tlcveil100_m1",
            "textureResref":"tlcveil100_t1",
            "module":{
                "moduleResref":"tlcv100demo1",
                "areaResref":"tlcv100area1",
                "hakResref":"tlcv100hak1"
            },
            "creatureResref":"tlcv100utc1"
        }"#;

        let mut studio = build_meshy_procedural_humanoid_p100k_experiment_v1_inner(
            &source,
            &appearance,
            identity_json,
        )
        .expect("Studio P100K replay");
        let frozen = root.join("proof-output/tlc-veiled-humanoid-p100k-v1/generated");

        assert_eq!(
            studio.take_proof_module_bytes(),
            std::fs::read(frozen.join("tlcv100demo1.mod")).unwrap()
        );
        assert_eq!(
            studio.take_hak_bytes(),
            std::fs::read(frozen.join("tlcv100hak1.hak")).unwrap()
        );
        assert_eq!(
            studio.take_model_bytes(),
            std::fs::read(frozen.join("tlcveil100_m1.mdl")).unwrap()
        );
        assert_eq!(
            studio.take_texture_bytes(),
            std::fs::read(frozen.join("tlcveil100_t1.tga")).unwrap()
        );
        assert_eq!(
            studio.take_appearance_two_da_bytes(),
            std::fs::read(frozen.join("appearance.2da")).unwrap()
        );
    }

    #[test]
    fn studio_product_inspection_uses_the_shared_300k_budget() {
        if std::env::var_os("M2A_REQUIRE_P100K_STUDIO_REPLAY").is_none() {
            return;
        }
        let source = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb"),
        )
        .expect("canonical P100K Meshy source");
        let default: serde_json::Value =
            serde_json::from_str(&ingest_glb_json(&source)).expect("default inspection JSON");
        let experiment: serde_json::Value =
            serde_json::from_str(&ingest_meshy_p100k_experiment_json(&source))
                .expect("P100K inspection JSON");

        assert_eq!(default["report"]["conversionEligible"], true);
        assert_eq!(experiment["report"]["conversionEligible"], true);
        assert_eq!(experiment["report"]["statistics"]["triangleCount"], 102_335);
    }

    #[test]
    fn studio_full_native_h1_v3_compatibility_and_v4_product_boundaries_are_exact() {
        let source = full_native_42_owned_glb();
        let appearance = DIRECT_CREATURE_APPEARANCE;
        let event_authoring = common_native_event_authoring();
        let event_authoring_json =
            serde_json::to_string(&event_authoring).expect("event authoring JSON");
        let core = m2a_core::model_pipeline::build_meshy_h1_model_package_v3(
            &source,
            appearance,
            m2a_core::model_pipeline::DirectCreatureAnimationProfileV1::FullNative42ExplicitV1,
            m2a_core::model_pipeline::DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1,
            &event_authoring,
        )
        .expect("core eventful full H1");
        let mut studio =
            build_meshy_h1_model_package_v3_inner(&source, appearance, &event_authoring_json)
                .expect("Studio eventful full H1");
        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert_eq!(studio.take_model_bytes(), core.model);
        assert_eq!(
            studio.readback_json(),
            serialize_json(&m2a_core::inspect_binary_mdl(&core.model).unwrap())
        );
        let report: serde_json::Value =
            serde_json::from_slice(&core.report_json).expect("Core report");
        assert_eq!(report["animationEventConformance"]["requiredPairCount"], 23);
        assert_eq!(
            report["animationEventConformance"]["satisfiedPairCount"],
            23
        );
        assert_eq!(report["animationEventConformance"]["complete"], true);

        let product_identity_json = r#"{
            "modelResref":"m2a_evtmdl_v2",
            "textureResref":"m2a_evttex_v2",
            "hakResref":"m2a_evthak_v2",
            "appearanceLabel":"M2A_EVENT_CREATURE_V2"
        }"#;
        let mut product_demo = build_meshy_full_native_h1_package_with_options_v4_inner(
                &source,
                appearance,
                product_identity_json,
                r#"{"schemaVersion":1,"textureArtifactCleanup":false,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"AUTO"},"heldWeapon":{"schemaVersion":1,"mode":"LEFT_HAND","itemResref":"nw_wswss001"}}"#,
                &event_authoring_json,
                r#"{"moduleResref":"m2a_evtmod_v2","areaResref":"m2a_evtarea_v2","hakResref":"m2a_evthak_v2"}"#,
                "m2a_evtutc_v2",
            )
            .expect("collision-free full-native package with caller-owned events");
        let product_report: serde_json::Value =
            serde_json::from_str(&product_demo.report_json()).expect("product report JSON");
        let product_summary: serde_json::Value =
            serde_json::from_str(&product_demo.summary_json()).expect("product summary JSON");
        assert_eq!(product_summary["modelResref"], "m2a_evtmdl_v2");
        assert_eq!(
            product_report["animationEventConformance"]["requiredPairCount"],
            23
        );
        assert_eq!(
            product_report["animationEventConformance"]["satisfiedPairCount"],
            23
        );
        let product_demo_report: serde_json::Value =
            serde_json::from_str(&product_demo.demo_report_json()).expect("demo report JSON");
        assert_eq!(product_demo_report["moduleResref"], "m2a_evtmod_v2");
        assert_eq!(product_demo_report["hakResref"], "m2a_evthak_v2");
        assert_eq!(
            product_demo_report["heldStockWeaponReadback"]["fixtures"][0]["hand"],
            "left_hand"
        );
        assert_eq!(
            product_demo_report["heldStockWeaponReadback"]["weapon"]["resref"],
            "nw_wswss001"
        );
        assert_eq!(
            product_demo_report["heldStockWeaponReadback"]["weapon"]["resourceScope"],
            "NWN_BASE_GAME"
        );
        assert_eq!(
            product_report["proofModule"]["sha256"],
            product_demo_report["sha256"]
        );
        assert!(!product_demo.take_hak_bytes().is_empty());
        assert!(!product_demo.take_model_bytes().is_empty());
        assert!(!product_demo.take_texture_bytes().is_empty());
        assert!(!product_demo.take_proof_module_bytes().is_empty());

        let plain = build_meshy_full_native_h1_package_with_options_v4_inner(
            &source,
            appearance,
            product_identity_json,
            r#"{"schemaVersion":1,"textureArtifactCleanup":false,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"KEEP_SOURCE_WEIGHTS"}}"#,
            "",
            r#"{"moduleResref":"m2a_evtmod_v2","areaResref":"m2a_evtarea_v2","hakResref":"m2a_evthak_v2"}"#,
            "m2a_evtutc_v2",
        )
        .expect("collision-free full-native package without an event sidecar");
        let plain_report: serde_json::Value =
            serde_json::from_str(&plain.report_json()).expect("plain report JSON");
        assert!(plain_report["animationEventConformance"].is_null());

        let boundary_malformed = match build_meshy_full_native_h1_package_with_options_v4_inner(
            &source,
            appearance,
            product_identity_json,
            r#"{"schemaVersion":1,"textureArtifactCleanup":false,"skinAccessoryStabilization":{"schemaVersion":2,"mode":"AUTO"}}"#,
            "{",
            r#"{"moduleResref":"m2a_evtmod_v2","areaResref":"m2a_evtarea_v2","hakResref":"m2a_evthak_v2"}"#,
            "m2a_evtutc_v2",
        ) {
            Ok(_) => panic!("malformed full-native sidecar must fail"),
            Err(error) => error,
        };
        let boundary_malformed: serde_json::Value =
            serde_json::from_str(&boundary_malformed).expect("structured boundary error");
        assert_eq!(
            boundary_malformed["code"],
            "M6-ANIMATION-EVENT-AUTHORING-JSON"
        );

        let malformed = match build_meshy_h1_model_package_v3_inner(&source, appearance, "{") {
            Ok(_) => panic!("malformed event authoring JSON must fail"),
            Err(error) => error,
        };
        let malformed: serde_json::Value =
            serde_json::from_str(&malformed).expect("structured JSON error");
        assert_eq!(malformed["code"], "M6-ANIMATION-EVENT-AUTHORING-JSON");
        assert_eq!(malformed["path"], "eventAuthoringJson");
        let unknown = match build_meshy_h1_model_package_v3_inner(
            &source,
            appearance,
            r#"{"schemaVersion":1,"clips":[],"unknown":true}"#,
        ) {
            Ok(_) => panic!("unknown event authoring JSON fields must fail"),
            Err(error) => error,
        };
        let unknown: serde_json::Value =
            serde_json::from_str(&unknown).expect("structured unknown-field error");
        assert_eq!(unknown["code"], "M6-ANIMATION-EVENT-AUTHORING-JSON");

        let mut incomplete = event_authoring;
        incomplete
            .clips
            .iter_mut()
            .find(|clip| clip.clip_name == "ccastout")
            .expect("ccastout authoring")
            .events
            .retain(|event| event.name != "cast");
        let error = match build_meshy_h1_model_package_v3_inner(
            &source,
            appearance,
            &serde_json::to_string(&incomplete).unwrap(),
        ) {
            Ok(_) => panic!("incomplete event authoring must fail"),
            Err(error) => error,
        };
        let error: serde_json::Value = serde_json::from_str(&error).expect("structured error");
        assert_eq!(error["code"], "M6-ANIMATION-EVENTS-INELIGIBLE");
        assert!(error["message"].as_str().unwrap().contains("ccastout:cast"));
    }

    #[test]
    fn hak_boundary_precedence_ranges_zero_size_and_no_panic_are_stable() {
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let options_json = serde_json::to_string(&HakWriterOptionsV1::default()).unwrap();
        assert_eq!(
            write_hak_artifact_json(&[], "{", "{").unwrap_err(),
            r#"{"schemaVersion":1,"code":"M5-HAK-RESOURCES-JSON-INVALID","severity":"FATAL","path":"resourcesJson","message":"HAK resources JSON does not match the strict public schema"}"#
        );
        let (_, valid, _) = hak_boundary(&[("a", 1, b"x")]);
        let valid_json = serde_json::to_string(&valid).unwrap();
        assert_eq!(
            write_hak_artifact_json(&[], &valid_json, "{").unwrap_err(),
            r#"{"schemaVersion":1,"code":"M5-HAK-OPTIONS-JSON-INVALID","severity":"FATAL","path":"optionsJson","message":"HAK options JSON does not match the strict public schema"}"#
        );
        for nested in [false, true] {
            let mut options = serde_json::to_value(HakWriterOptionsV1::default()).unwrap();
            if nested {
                options["limits"]["unknown"] = serde_json::json!(true);
            } else {
                options["unknown"] = serde_json::json!(true);
            }
            let error =
                write_hak_artifact_json(&[], &valid_json, &options.to_string()).unwrap_err();
            assert!(error.contains("M5-HAK-OPTIONS-JSON-INVALID"));
        }
        for nested in [false, true] {
            let mut resources = serde_json::to_value(&valid).unwrap();
            if nested {
                resources["resources"][0]["unknown"] = serde_json::json!(true);
            } else {
                resources["unknown"] = serde_json::json!(true);
            }
            let error =
                write_hak_artifact_json(&[], &resources.to_string(), &options_json).unwrap_err();
            assert!(error.contains("M5-HAK-RESOURCES-JSON-INVALID"));
        }

        let cases = [
            (vec![1], vec![("a", 1, 2, 0)], "resources[0].payloadOffset"),
            (vec![1], vec![("a", 1, 0, 2)], "resources[0].payloadSize"),
            (
                vec![1, 2],
                vec![("a", 1, 0, 2), ("b", 1, 1, 1)],
                "payloadBlob",
            ),
            (vec![1, 2], vec![("a", 1, 1, 1)], "payloadBlob"),
            (vec![1, 2], vec![("a", 1, 0, 1)], "payloadBlob"),
        ];
        for (blob, raw, expected_path) in cases {
            let descriptors = HakResourceDescriptorsV1 {
                schema_version: 1,
                resources: raw
                    .into_iter()
                    .map(|(resref, resource_type, payload_offset, payload_size)| {
                        HakResourceDescriptorV1 {
                            resref: resref.to_owned(),
                            resource_type,
                            payload_offset,
                            payload_size,
                        }
                    })
                    .collect(),
            };
            let resources_json = serde_json::to_string(&descriptors).unwrap();
            let result = catch_unwind(AssertUnwindSafe(|| {
                write_hak_artifact_json(&blob, &resources_json, &options_json)
            }));
            let error = result
                .expect("range validation must not panic")
                .unwrap_err();
            let value: serde_json::Value = serde_json::from_str(&error).unwrap();
            assert_eq!(value["code"], "M5-HAK-PAYLOAD-RANGE-INVALID");
            assert_eq!(value["path"], expected_path);
        }

        let empty = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![HakResourceDescriptorV1 {
                resref: "empty".to_owned(),
                resource_type: 1,
                payload_offset: 0,
                payload_size: 0,
            }],
        };
        assert!(
            materialize_hak_resources(&[], &empty).unwrap()[0]
                .payload
                .is_empty()
        );
    }

    #[test]
    fn hak_range_precedes_borrowed_core_preflight_and_both_precede_payload_copy() {
        let descriptor = |resref: &str, payload_size: u32| HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![HakResourceDescriptorV1 {
                resref: resref.to_owned(),
                resource_type: 3,
                payload_offset: 99,
                payload_size,
            }],
        };

        let invalid_resref = serde_json::to_string(&descriptor("BAD", 1)).unwrap();
        let default_options = serde_json::to_string(&HakWriterOptionsV1::default()).unwrap();
        let error = write_hak_artifact_json(&[], &invalid_resref, &default_options).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let valid_descriptor = serde_json::to_string(&descriptor("texture", 1)).unwrap();
        let invalid_options = serde_json::to_string(&HakWriterOptionsV1 {
            schema_version: 2,
            ..HakWriterOptionsV1::default()
        })
        .unwrap();
        let error = write_hak_artifact_json(&[], &valid_descriptor, &invalid_options).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let entry_limited = serde_json::to_string(&HakWriterOptionsV1 {
            schema_version: 1,
            limits: m2a_core::hak::HakWriterLimitsV1 {
                max_entry_count: 0,
                max_output_bytes: m2a_core::hak::HAK_MAX_OUTPUT_BYTES,
            },
        })
        .unwrap();
        let error = write_hak_artifact_json(&[], &valid_descriptor, &entry_limited).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let output_limited = serde_json::to_string(&HakWriterOptionsV1 {
            schema_version: 1,
            limits: m2a_core::hak::HakWriterLimitsV1 {
                max_entry_count: 1,
                max_output_bytes: 160,
            },
        })
        .unwrap();
        let error = write_hak_artifact_json(&[], &valid_descriptor, &output_limited).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let duplicate = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![
                HakResourceDescriptorV1 {
                    resref: "same".to_owned(),
                    resource_type: 3,
                    payload_offset: 99,
                    payload_size: 1,
                },
                HakResourceDescriptorV1 {
                    resref: "same".to_owned(),
                    resource_type: 3,
                    payload_offset: 100,
                    payload_size: 1,
                },
            ],
        };
        let error = write_hak_artifact_json(
            &[],
            &serde_json::to_string(&duplicate).unwrap(),
            &default_options,
        )
        .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let range_error =
            write_hak_artifact_json(&[], &valid_descriptor, &default_options).unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&range_error).unwrap()["code"],
            "M5-HAK-PAYLOAD-RANGE-INVALID"
        );

        let valid_range_invalid_resref = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: vec![HakResourceDescriptorV1 {
                resref: "BAD".to_owned(),
                resource_type: 3,
                payload_offset: 0,
                payload_size: 1,
            }],
        };
        let error = write_hak_artifact_json(
            &[0],
            &serde_json::to_string(&valid_range_invalid_resref).unwrap(),
            &default_options,
        )
        .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "M5-HAK-RESREF-INVALID"
        );
    }
}

#[cfg(test)]
#[path = "../../m2a-core/tests/fixtures/build_synthetic_glb.rs"]
mod profile_a_animation_fixtures;

#[cfg(test)]
mod profile_a_test_support {
    use m2a_core::profile_a::{
        Bounds3V1, CreatureRigNodeV1, CreatureRigProfileV1, CreatureRigSegmentV1,
        ProfileAAnimationClipMappingV1, ProfileAAnimationMappingV1, ProfileAAnimationNodeMappingV1,
        ProfileAOptionsV1, RigProvenanceAttestationsV1, RigProvenanceKindV1, RigProvenanceV1,
        RigSegmentDeformationV1, RigWeightInfluenceV1, canonical_profile_sha256,
    };
    use sha2::{Digest, Sha256};

    pub const RIGID_JSON_SHA256: &str =
        "d62b2444df8005b6bef0affb7f753767488ad33568096a47523cabbe5edefa06";
    pub const RIGID_JSON_LENGTH: usize = 3187;
    pub const SKIN_JSON_SHA256: &str =
        "8017ea957de0e7a47426fa004063996762772589847604c628d4cfbd1b27f79b";
    pub const SKIN_JSON_LENGTH: usize = 3563;
    pub const LIMIT_FATAL_JSON_SHA256: &str =
        "3bfb45cf36af0d4af174cea656ab669714a55c4c88a9661c7ea573be75bec4a2";
    pub const LIMIT_FATAL_JSON_LENGTH: usize = 162;
    pub const MALFORMED_JSON_SHA256: &str =
        "03bd6ebd5cdb45f738de87363d3ad6a95de9bbb5aa119a2fce3199255b8efa55";
    pub const MALFORMED_JSON_LENGTH: usize = 151;
    pub const ANIMATED_JSON_SHA256: &str =
        "65c3d6d5e3764ec4a256996f84b2631f665fa14bc56f3dd75de5091b31c673ee";
    pub const ANIMATED_JSON_LENGTH: usize = 3886;

    pub fn identity() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn base_profile() -> CreatureRigProfileV1 {
        CreatureRigProfileV1 {
            schema_version: 1,
            profile_id: "synthetic-rigid-axis-profile".to_owned(),
            content_sha256: String::new(),
            provenance: RigProvenanceV1 {
                kind: RigProvenanceKindV1::Synthetic,
                export_allowed: true,
                attestations: RigProvenanceAttestationsV1 {
                    controlled_construction: true,
                    no_reference_payload_copied: true,
                    rights_confirmed: true,
                },
            },
            target_bounds: Bounds3V1 {
                min: [-10.0, -10.0, 0.0],
                max: [10.0, 10.0, 1.0],
            },
            alignment_anchor: [0.0, 0.0, 0.0],
            nodes: vec![CreatureRigNodeV1 {
                id: 70,
                name: "synthetic-rigid-root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity(),
            }],
            segments: Vec::new(),
        }
    }

    fn finish(mut profile: CreatureRigProfileV1) -> CreatureRigProfileV1 {
        profile.content_sha256 = canonical_profile_sha256(&profile).unwrap();
        profile
    }

    pub fn rigid_profile() -> CreatureRigProfileV1 {
        let mut profile = base_profile();
        profile.segments.push(CreatureRigSegmentV1 {
            id: 9,
            name: "synthetic-rigid-segment".to_owned(),
            deformation: RigSegmentDeformationV1::Rigid,
            parent_node_id: 70,
            surface_positions: vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 1.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: Vec::new(),
            reference_weights: Vec::new(),
        });
        finish(profile)
    }

    pub fn skin_multi_profile() -> CreatureRigProfileV1 {
        let mut profile = base_profile();
        profile.profile_id = "wasm-controlled-skin-multi".to_owned();
        profile.nodes.push(CreatureRigNodeV1 {
            id: 1,
            name: "wasm-controlled-bone".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: identity(),
        });
        profile.segments.push(CreatureRigSegmentV1 {
            id: 5,
            name: "wasm-controlled-rigid-far".to_owned(),
            deformation: RigSegmentDeformationV1::Rigid,
            parent_node_id: 70,
            surface_positions: vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 0.0, 1.0]],
            surface_indices: vec![0, 1, 2],
            allowed_bone_node_ids: Vec::new(),
            reference_weights: Vec::new(),
        });
        profile.segments.push(CreatureRigSegmentV1 {
            id: 10,
            name: "wasm-controlled-skin-near".to_owned(),
            deformation: RigSegmentDeformationV1::Skin,
            parent_node_id: 70,
            surface_positions: vec![
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [-1.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            surface_indices: vec![0, 1, 2, 1, 3, 2],
            allowed_bone_node_ids: vec![1],
            reference_weights: vec![
                vec![RigWeightInfluenceV1 {
                    bone_node_id: 1,
                    value: 1.0,
                }];
                4
            ],
        });
        finish(profile)
    }

    pub fn options() -> ProfileAOptionsV1 {
        ProfileAOptionsV1::default()
    }

    pub fn profile_json(profile: &CreatureRigProfileV1) -> String {
        serde_json::to_string(profile).unwrap()
    }

    pub fn options_json(options: &ProfileAOptionsV1) -> String {
        serde_json::to_string(options).unwrap()
    }

    pub fn linear_animated_glb() -> Vec<u8> {
        super::profile_a_animation_fixtures::mutate_json(
            super::profile_a_animation_fixtures::skin_animation_with_inverse_bind_matrices(),
            |root| {
                root["animations"][0]["samplers"][1]["interpolation"] = serde_json::json!("LINEAR");
                root["animations"][0]["samplers"]
                    .as_array_mut()
                    .expect("synthetic animation samplers")
                    .truncate(2);
                root["animations"][0]["channels"]
                    .as_array_mut()
                    .expect("synthetic animation channels")
                    .truncate(2);
            },
        )
    }

    pub fn linear_nonplanar_animated_glb() -> Vec<u8> {
        super::profile_a_animation_fixtures::mutate_accessor_f32(linear_animated_glb(), 0, 8, 0.5)
    }

    pub fn animated_profile() -> CreatureRigProfileV1 {
        let mut profile = rigid_profile();
        profile.target_bounds.max[2] = 2.0;
        let mut child_bind = identity();
        child_bind[0] = 0.0;
        child_bind[1] = 1.0;
        child_bind[4] = -1.0;
        child_bind[5] = 0.0;
        child_bind[12] = 10.0;
        child_bind[13] = 20.0;
        child_bind[14] = 30.0;
        profile.nodes.push(CreatureRigNodeV1 {
            id: 71,
            name: "synthetic-animated-child".to_owned(),
            parent_id: Some(70),
            bind_local_matrix: child_bind,
        });
        finish(profile)
    }

    pub fn animation_mapping() -> ProfileAAnimationMappingV1 {
        ProfileAAnimationMappingV1 {
            schema_version: 1,
            source_skin_id: 0,
            provenance: RigProvenanceV1 {
                kind: RigProvenanceKindV1::Synthetic,
                export_allowed: true,
                attestations: RigProvenanceAttestationsV1 {
                    controlled_construction: true,
                    no_reference_payload_copied: true,
                    rights_confirmed: true,
                },
            },
            source_translation_scale: 1.0,
            node_mappings: vec![
                ProfileAAnimationNodeMappingV1 {
                    source_node_id: 0,
                    output_rig_node_id: 70,
                },
                ProfileAAnimationNodeMappingV1 {
                    source_node_id: 1,
                    output_rig_node_id: 71,
                },
            ],
            clip_mappings: vec![ProfileAAnimationClipMappingV1 {
                source_animation_id: 0,
                output_clip_name: "cpause1".to_owned(),
                transition_seconds: 0.25,
            }],
        }
    }

    pub fn sha256(value: &str) -> String {
        let digest = Sha256::digest(value.as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn minimal_glb() -> Vec<u8> {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = [[0.0_f32, 0.0, 1.0]; 3];
        let uv0 = [[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let mut bin = Vec::new();
        let positions_offset = bin.len();
        append_rows(&mut bin, &positions);
        let normals_offset = bin.len();
        append_rows(&mut bin, &normals);
        let uv_offset = bin.len();
        append_rows(&mut bin, &uv0);
        let indices_offset = bin.len();
        for index in [0_u16, 1, 2] {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        align4(&mut bin, 0);
        let root = serde_json::json!({
            "asset": {"version": "2.0", "generator": "m2a-profile-a-byte-proof"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"name": "proof-root", "mesh": 0}],
            "meshes": [{"primitives": [{
                "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                "indices": 3,
                "mode": 4
            }]}],
            "buffers": [{"byteLength": bin.len()}],
            "bufferViews": [
                {"buffer": 0, "byteOffset": positions_offset, "byteLength": normals_offset - positions_offset},
                {"buffer": 0, "byteOffset": normals_offset, "byteLength": uv_offset - normals_offset},
                {"buffer": 0, "byteOffset": uv_offset, "byteLength": indices_offset - uv_offset},
                {"buffer": 0, "byteOffset": indices_offset, "byteLength": 6}
            ],
            "accessors": [
                {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0]},
                {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3"},
                {"bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2"},
                {"bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR"}
            ]
        });
        make_glb(root, bin)
    }

    fn append_rows<const N: usize>(output: &mut Vec<u8>, rows: &[[f32; N]]) {
        for row in rows {
            for value in row {
                output.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    fn align4(bytes: &mut Vec<u8>, fill: u8) {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(fill);
        }
    }

    fn make_glb(root: serde_json::Value, mut bin: Vec<u8>) -> Vec<u8> {
        let mut json = serde_json::to_vec(&root).unwrap();
        align4(&mut json, b' ');
        align4(&mut bin, 0);
        let total = 12 + 8 + json.len() + 8 + bin.len();
        let mut glb = Vec::with_capacity(total);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin);
        glb
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod profile_a_native_tests {
    use super::{
        convert_profile_a_glb_json, convert_profile_a_json,
        convert_profile_a_with_animations_glb_json, profile_a_test_support as support,
        retarget_static_mesh_to_animated_donor_v1,
    };

    fn direct_core_json(
        bytes: &[u8],
        rig: &m2a_core::profile_a::CreatureRigProfileV1,
        options: &m2a_core::profile_a::ProfileAOptionsV1,
    ) -> String {
        let source = m2a_core::glb::ingest_glb(bytes, &m2a_core::glb::GlbLimits::default())
            .expect("controlled source ingest");
        match m2a_core::profile_a::convert_profile_a(&source, rig, options) {
            Ok(value) => serde_json::to_string(&value).unwrap(),
            Err(error) => serde_json::to_string(&error).unwrap(),
        }
    }

    #[test]
    fn native_profile_a_adapter_is_exact_core_json_and_records_byte_proof() {
        let glb = support::minimal_glb();
        for (rig, expected_sha, expected_length) in [
            (
                support::rigid_profile(),
                support::RIGID_JSON_SHA256,
                support::RIGID_JSON_LENGTH,
            ),
            (
                support::skin_multi_profile(),
                support::SKIN_JSON_SHA256,
                support::SKIN_JSON_LENGTH,
            ),
        ] {
            let options = support::options();
            let actual = convert_profile_a_json(
                &glb,
                &support::profile_json(&rig),
                &support::options_json(&options),
            );
            assert_eq!(
                actual,
                convert_profile_a_glb_json(
                    &glb,
                    &support::profile_json(&rig),
                    &support::options_json(&options),
                )
            );
            assert_eq!(actual, direct_core_json(&glb, &rig, &options));
            assert_eq!(
                actual.len(),
                expected_length,
                "sha={}",
                support::sha256(&actual)
            );
            assert_eq!(support::sha256(&actual), expected_sha);
        }
    }

    #[test]
    fn native_profile_a_json_errors_are_stable_and_strict() {
        let glb = support::minimal_glb();
        let options = support::options_json(&support::options());
        let malformed = convert_profile_a_glb_json(&glb, "{", &options);
        assert_eq!(malformed, convert_profile_a_json(&glb, "{", &options));
        assert_eq!(
            malformed,
            r#"{"schemaVersion":1,"code":"M3A-PROFILE-JSON-INVALID","severity":"FATAL","path":"rigJson","message":"rig profile JSON does not match the public schema"}"#
        );
        assert_eq!(
            malformed.len(),
            support::MALFORMED_JSON_LENGTH,
            "sha={}",
            support::sha256(&malformed)
        );
        assert_eq!(support::sha256(&malformed), support::MALFORMED_JSON_SHA256);
        let mut unknown = serde_json::to_value(support::rigid_profile()).unwrap();
        unknown["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_json(&glb, &unknown.to_string(), &options),
            convert_profile_a_json(&glb, "{", &options)
        );

        let rig = support::skin_multi_profile();
        let mut limited = support::options();
        limited.limits.max_distance_evaluations = 2;
        let fatal = convert_profile_a_json(
            &glb,
            &support::profile_json(&rig),
            &support::options_json(&limited),
        );
        assert_eq!(fatal, direct_core_json(&glb, &rig, &limited));
        assert_eq!(
            fatal.len(),
            support::LIMIT_FATAL_JSON_LENGTH,
            "sha={}",
            support::sha256(&fatal)
        );
        assert_eq!(support::sha256(&fatal), support::LIMIT_FATAL_JSON_SHA256);
    }

    #[test]
    fn native_animated_profile_a_adapter_matches_core_and_rejects_invalid_mapping_json() {
        let glb = support::linear_animated_glb();
        let rig = support::animated_profile();
        let options = support::options();
        let mapping = support::animation_mapping();
        let rig_json = serde_json::to_string(&rig).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let mapping_json = serde_json::to_string(&mapping).unwrap();

        let source = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default())
            .expect("controlled animated source ingest");
        let core = m2a_core::profile_a::convert_profile_a_with_animations_v1(
            &source, &rig, &options, &mapping,
        )
        .expect("controlled animated core conversion");
        let expected = serde_json::to_string(&core).unwrap();
        assert_eq!(expected.len(), support::ANIMATED_JSON_LENGTH);
        assert_eq!(support::sha256(&expected), support::ANIMATED_JSON_SHA256);
        assert_eq!(
            convert_profile_a_with_animations_glb_json(
                &glb,
                &rig_json,
                &options_json,
                &mapping_json,
            ),
            expected
        );

        let malformed =
            convert_profile_a_with_animations_glb_json(&glb, &rig_json, &options_json, "{");
        assert_eq!(
            malformed,
            r#"{"schemaVersion":1,"code":"M4A-MAPPING-JSON-INVALID","severity":"FATAL","path":"mappingJson","message":"animation mapping JSON does not match the public schema"}"#
        );
        let mut unknown = serde_json::to_value(&mapping).unwrap();
        unknown["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_with_animations_glb_json(
                &glb,
                &rig_json,
                &options_json,
                &unknown.to_string(),
            ),
            malformed
        );
    }

    #[test]
    fn native_animated_donor_boundary_is_byte_and_report_identical_to_core() {
        use m2a_core::mdl::{
            MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
            MdlWriterOptionsV1,
        };

        let source = support::minimal_glb();
        let donor = support::linear_nonplanar_animated_glb();
        let options = MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "wasm_donor".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "wasm_donortex".to_owned(),
            }],
        };
        let core = m2a_core::animated_donor::retarget_static_mesh_to_animated_donor_v1(
            &source, &donor, &options,
        )
        .unwrap();
        let mut boundary = retarget_static_mesh_to_animated_donor_v1(
            &source,
            &donor,
            &serde_json::to_string(&options).unwrap(),
        )
        .unwrap();

        assert_eq!(boundary.take_model_bytes(), core.model.payload);
        assert!(boundary.take_model_bytes().is_empty());
        assert_eq!(
            boundary.report_json(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!(
            boundary.readback_json(),
            serde_json::to_string(&core.model.inspection).unwrap()
        );
        assert_eq!(
            boundary.animations_json(),
            serde_json::to_string(&core.animations).unwrap()
        );
        assert_eq!(
            boundary.conversion_json(),
            serde_json::to_string(&core.conversion).unwrap()
        );
    }
}

#[cfg(test)]
mod model_material_boundary_tests {
    use super::*;

    #[test]
    fn inspection_is_compact_target_neutral_and_deterministic() {
        let glb = profile_a_animation_fixtures::one_primitive_two_disconnected_triangles();
        let first = inspect_model_components_v1_json_inner(&glb, "CREATURE").unwrap();
        let second = inspect_model_components_v1_json_inner(&glb, "CREATURE").unwrap();
        assert_eq!(first, second);
        let value: serde_json::Value = serde_json::from_str(&first).unwrap();
        assert_eq!(value["capabilities"]["target"], "CREATURE");
        assert_eq!(value["capabilities"]["maxMaterialSlots"], 256);
        assert_eq!(
            value["inventory"]["components"].as_array().unwrap().len(),
            2
        );
        assert!(value.get("triangleMaterialMap").is_none());
        assert!(!first.contains("triangleMaterialSlots"));
    }

    #[test]
    fn resolver_returns_report_only_and_rejects_stale_or_unknown_targets() {
        let glb = profile_a_animation_fixtures::one_primitive_two_disconnected_triangles();
        let inspection: serde_json::Value = serde_json::from_str(
            &inspect_model_components_v1_json_inner(&glb, "PLACEABLE").unwrap(),
        )
        .unwrap();
        let source_sha = inspection["inventory"]["sourceSha256"].as_str().unwrap();
        let component0 = inspection["inventory"]["components"][0]["key"].clone();
        let component1 = inspection["inventory"]["components"][1]["key"].clone();
        let document = serde_json::json!({
            "schemaVersion": 1,
            "sourceSha256": source_sha,
            "materials": [
                {
                    "authoredMaterialId": "material:sail",
                    "displayName": "Sail",
                    "previewColor": "#d0c8b0",
                    "sourceFallbackMaterialId": 0,
                    "sourceFallbackImageSha256": null
                },
                {
                    "authoredMaterialId": "material:wood",
                    "displayName": "Wood",
                    "previewColor": "#704020",
                    "sourceFallbackMaterialId": 0,
                    "sourceFallbackImageSha256": null
                }
            ],
            "assignments": [
                {"component": component0, "authoredMaterialId": "material:wood"},
                {"component": component1, "authoredMaterialId": "material:sail"}
            ]
        });
        let output = resolve_model_materials_v1_json_inner(
            &glb,
            "PLACEABLE",
            &serde_json::to_string(&document).unwrap(),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            value["report"]["materialSlots"].as_array().unwrap().len(),
            2
        );
        assert!(value.get("triangleMaterialMap").is_none());

        let mut stale = document;
        stale["sourceSha256"] = serde_json::json!("0".repeat(64));
        let error = resolve_model_materials_v1_json_inner(
            &glb,
            "PLACEABLE",
            &serde_json::to_string(&stale).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&error).unwrap()["code"],
            "MATERIAL-SEPARATION-SOURCE-MISMATCH"
        );

        let target_error = inspect_model_components_v1_json_inner(&glb, "ITEM").unwrap_err();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&target_error).unwrap()["code"],
            "MODEL-MATERIAL-TARGET-INVALID"
        );
    }

    #[test]
    fn face_mode_v2_boundary_exposes_face_capability_and_exact_ranges() {
        let glb = profile_a_animation_fixtures::one_primitive_two_disconnected_triangles();
        let inspection: serde_json::Value =
            serde_json::from_str(&inspect_model_faces_v2_json_inner(&glb, "PLACEABLE").unwrap())
                .unwrap();
        assert_eq!(inspection["schemaVersion"], 2);
        assert_eq!(inspection["capabilities"]["faceSelectionSupported"], true);
        assert_eq!(
            inspection["capabilities"]["selectionGranularity"],
            "CONNECTED_COMPONENTS_AND_FACES"
        );
        let source_sha = inspection["inventory"]["sourceSha256"].as_str().unwrap();
        let document = serde_json::json!({
            "schemaVersion": 2,
            "sourceSha256": source_sha,
            "materials": [{
                "authoredMaterialId": "material:wood",
                "displayName": "Wood",
                "previewColor": "#704020",
                "sourceFallbackMaterialId": 0,
                "sourceFallbackImageSha256": null
            }],
            "componentAssignments": [],
            "faceAssignments": [{
                "selection": {
                    "sceneId": 0,
                    "nodeId": 0,
                    "primitiveId": 0,
                    "triangleRanges": [{"startTriangle": 1, "triangleCount": 1}]
                },
                "authoredMaterialId": "material:wood"
            }]
        });
        let output = resolve_model_materials_v2_json_inner(
            &glb,
            "PLACEABLE",
            &serde_json::to_string(&document).unwrap(),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["schemaVersion"], 2);
        assert_eq!(value["report"]["assignedFaceCount"], 1);
        assert_eq!(value["report"]["outputTriangleCount"], 2);
        assert_eq!(value["report"]["duplicatedBoundaryVertexCount"], 0);
        assert!(value.get("triangleMaterialMap").is_none());
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::{
        HakResourceDescriptorV1, HakResourceDescriptorsV1, append_two_da_row_v1,
        append_two_da_row_v1_report_json, build_m6_model_package_v1, convert_profile_a_glb_json,
        convert_profile_a_json, convert_profile_a_with_animations_glb_json, ingest_glb,
        ingest_glb_json, inspect_binary_mdl, inspect_glb, inspect_glb_json, inspect_two_da_v2_json,
        profile_a_test_support as profile_support,
        write_binary_mdl_with_animations as write_mdl_bytes,
        write_binary_mdl_with_animations_report_json as write_mdl_report_json, write_hak_v1,
        write_hak_v1_report_json, write_package_manifest_v1_json, write_tga_v1,
        write_tga_v1_report_json,
    };
    use wasm_bindgen_test::*;

    #[allow(dead_code)]
    mod fixtures {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../m2a-core/tests/fixtures/build_minimal_binary_mdl.rs"
        ));
    }

    #[wasm_bindgen_test]
    fn m5_tga_boundary_matches_core_is_immutable_and_has_frozen_error() {
        use m2a_core::tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1};

        let image = TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgb8,
            pixels: vec![255, 0, 128],
        };
        let options = TgaWriterOptionsV1::default();
        let image_json = serde_json::to_string(&image).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (image_json.clone(), options_json.clone());
        let core = m2a_core::tga::write_tga_v1(&image, &options).unwrap();

        assert_eq!(
            write_tga_v1(&image_json, &options_json).unwrap(),
            core.payload
        );
        assert_eq!(
            write_tga_v1_report_json(&image_json, &options_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((image_json, options_json), before);

        let error = write_tga_v1("{", &before.1)
            .unwrap_err()
            .as_string()
            .unwrap();
        assert_eq!(
            error,
            r#"{"schemaVersion":1,"code":"M5-TGA-IMAGE-JSON-INVALID","severity":"FATAL","path":"imageJson","message":"image JSON does not match the strict public schema"}"#
        );
        assert!(!error.to_ascii_lowercase().contains("base64"));
        for nested in [false, true] {
            let mut invalid = serde_json::to_value(options).unwrap();
            if nested {
                invalid["limits"]["unknown"] = serde_json::json!(true);
            } else {
                invalid["unknown"] = serde_json::json!(true);
            }
            let error = write_tga_v1(&before.0, &invalid.to_string())
                .unwrap_err()
                .as_string()
                .unwrap();
            assert!(error.contains("M5-TGA-OPTIONS-JSON-INVALID"));
        }
    }

    #[wasm_bindgen_test]
    fn m5_two_da_boundary_matches_core_is_immutable_and_strict() {
        use m2a_core::two_da::{
            TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        };

        let source = b"2DA V2.0\n\nA B\n0 old ****\n".to_vec();
        let request = TwoDaAppendRequestV1 {
            schema_version: 1,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "new".to_owned(),
                },
            }],
        };
        let limits = TwoDaLimitsV1::default();
        let request_json = serde_json::to_string(&request).unwrap();
        let limits_json = serde_json::to_string(&limits).unwrap();
        let before = (source.clone(), request_json.clone(), limits_json.clone());

        let inspection = m2a_core::two_da::inspect_two_da_v2(&source, &limits).unwrap();
        assert_eq!(
            inspect_two_da_v2_json(&source, &limits_json).unwrap(),
            serde_json::to_string(&inspection).unwrap()
        );
        let core = m2a_core::two_da::append_two_da_row_v1(&source, &request, &limits).unwrap();
        assert_eq!(
            append_two_da_row_v1(&source, &request_json, &limits_json).unwrap(),
            core.payload
        );
        assert_eq!(
            append_two_da_row_v1_report_json(&source, &request_json, &limits_json).unwrap(),
            serde_json::to_string(&core.report).unwrap()
        );
        assert_eq!((source, request_json, limits_json), before);

        let error = append_two_da_row_v1(&before.0, "{", "{")
            .unwrap_err()
            .as_string()
            .unwrap();
        assert_eq!(
            error,
            r#"{"schemaVersion":1,"code":"M5-2DA-REQUEST-JSON-INVALID","severity":"FATAL","path":"requestJson","message":"2DA append request JSON does not match the strict public schema"}"#
        );
        assert!(!error.to_ascii_lowercase().contains("base64"));
        let mut invalid_limits = serde_json::to_value(limits).unwrap();
        invalid_limits["unknown"] = serde_json::json!(true);
        let error = inspect_two_da_v2_json(&before.0, &invalid_limits.to_string())
            .unwrap_err()
            .as_string()
            .unwrap();
        assert!(error.contains("M5-2DA-LIMITS-JSON-INVALID"));
        for path in ["request", "cell", "value"] {
            let mut invalid = serde_json::to_value(&request).unwrap();
            match path {
                "request" => invalid["unknown"] = serde_json::json!(true),
                "cell" => invalid["cells"][0]["unknown"] = serde_json::json!(true),
                "value" => invalid["cells"][0]["value"]["unknown"] = serde_json::json!(true),
                _ => unreachable!(),
            }
            let error = append_two_da_row_v1(&before.0, &invalid.to_string(), &before.2)
                .unwrap_err()
                .as_string()
                .unwrap();
            assert!(error.contains("M5-2DA-REQUEST-JSON-INVALID"), "{path}");
        }
    }

    #[wasm_bindgen_test]
    fn m5_hak_package_boundary_matches_core_is_immutable_and_strict() {
        use m2a_core::hak::{HakResourceInputV1, HakWriterOptionsV1};

        let entries: [(&str, u16, &[u8]); 3] = [
            ("texture", 3, b"tga"),
            ("appearance", 2017, b"2da"),
            ("model", 2002, b"mdl"),
        ];
        let mut blob = Vec::new();
        let mut descriptors = Vec::new();
        let mut resources = Vec::new();
        for (resref, resource_type, payload) in entries {
            let payload_offset = blob.len() as u32;
            blob.extend_from_slice(payload);
            descriptors.push(HakResourceDescriptorV1 {
                resref: resref.to_owned(),
                resource_type,
                payload_offset,
                payload_size: payload.len() as u32,
            });
            resources.push(HakResourceInputV1 {
                resref: resref.to_owned(),
                resource_type,
                payload: payload.to_vec(),
            });
        }
        let descriptors = HakResourceDescriptorsV1 {
            schema_version: 1,
            resources: descriptors,
        };
        let options = HakWriterOptionsV1::default();
        let resources_json = serde_json::to_string(&descriptors).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let before = (blob.clone(), resources_json.clone(), options_json.clone());
        let hak = m2a_core::hak::write_hak_v1(&resources, &options).unwrap();
        let manifest = m2a_core::package::write_package_manifest_v1(&resources, &options).unwrap();

        assert_eq!(
            write_hak_v1(&blob, &resources_json, &options_json).unwrap(),
            hak.payload
        );
        assert_eq!(
            write_hak_v1_report_json(&blob, &resources_json, &options_json).unwrap(),
            serde_json::to_string(&hak.report).unwrap()
        );
        assert_eq!(
            write_package_manifest_v1_json(&blob, &resources_json, &options_json).unwrap(),
            serde_json::to_string(&manifest).unwrap()
        );
        assert_eq!((blob, resources_json, options_json), before);

        let error = write_hak_v1(&before.0, "{", "{")
            .unwrap_err()
            .as_string()
            .unwrap();
        assert_eq!(
            error,
            r#"{"schemaVersion":1,"code":"M5-HAK-RESOURCES-JSON-INVALID","severity":"FATAL","path":"resourcesJson","message":"HAK resources JSON does not match the strict public schema"}"#
        );
        assert!(!error.to_ascii_lowercase().contains("base64"));
    }

    #[wasm_bindgen_test]
    fn studio_model_package_public_wasm_boundary_matches_core() {
        let source = m2a_core::owned_fixture::synthetic_owned_m6_glb_v1().unwrap();
        let appearance = b"2DA V2.0\r\n\r\nLABEL MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY\r\n0 Existing NORM P existing **** **** 0 R 1.0 4\r\n";
        let core =
            m2a_core::model_pipeline::build_m6_model_package_v1(&source, appearance).unwrap();
        let mut studio = build_m6_model_package_v1(&source, appearance).unwrap();

        assert_eq!(studio.take_hak_bytes(), core.hak);
        assert!(studio.take_hak_bytes().is_empty());
        assert_eq!(studio.take_model_bytes(), core.model);
        assert!(studio.take_model_bytes().is_empty());
        assert_eq!(
            studio.report_json(),
            String::from_utf8(core.report_json).unwrap()
        );
        assert_eq!(
            studio.manifest_json(),
            String::from_utf8(core.manifest_json).unwrap()
        );
        assert_eq!(
            studio.summary_json(),
            String::from_utf8(core.summary_json).unwrap()
        );
    }

    #[wasm_bindgen_test]
    fn public_animation_writer_is_native_wasm_byte_and_report_identical() {
        use m2a_core::mdl::{
            MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
            MdlAnimationTrackPathV1, MdlAnimationTrackV1, MdlFormatProfileV1,
            MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1, MdlWriterOptionsV1,
        };
        use m2a_core::profile_a::{
            AuroraCreatureIrV1, AuroraCreatureNodeV1, AuroraCreatureSegmentV1,
            MaterialSourceBindingV1, RigSegmentDeformationV1,
        };

        let identity = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let creature = AuroraCreatureIrV1 {
            schema_version: 1,
            profile_id: "wasm-owned-animation".to_owned(),
            source_sha256: "0".repeat(64),
            basis_status: "PROFILE_A_LOCKED_M3".to_owned(),
            engine_facing_proof: "OPEN_M6".to_owned(),
            uv_runtime_proof: "OPEN_M6".to_owned(),
            nodes: vec![AuroraCreatureNodeV1 {
                id: 70,
                name: "root".to_owned(),
                parent_id: None,
                bind_local_matrix: identity,
            }],
            material_source_bindings: vec![MaterialSourceBindingV1 {
                slot: 0,
                source_material_id: None,
                source_material_name: None,
            }],
            segments: vec![AuroraCreatureSegmentV1 {
                segment_id: 1,
                material_slot: 0,
                deformation: RigSegmentDeformationV1::Rigid,
                parent_node_id: 70,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                tangents: None,
                uv0: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                indices: vec![0, 1, 2],
                face_surface_ids: Vec::new(),
                weights: Vec::new(),
            }],
        };
        let animations = MdlAnimationSetV1 {
            schema_version: 1,
            clips: vec![MdlAnimationClipV1 {
                name: "cpause1".to_owned(),
                animation_root: "owned_root".to_owned(),
                length_seconds: 1.0,
                transition_seconds: 0.25,
                events: Vec::new(),
                tracks: vec![MdlAnimationTrackV1 {
                    target_node_id: 70,
                    path: MdlAnimationTrackPathV1::Translation,
                    interpolation: MdlAnimationInterpolationV1::Linear,
                    times_seconds: vec![0.0, 1.0],
                    values: vec![vec![0.0, 0.0, 0.0], vec![0.25, 0.0, 0.0]],
                }],
            }],
        };
        let options = MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: "wasm_mdl".to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: "wasm_tex".to_owned(),
            }],
        };
        let native =
            m2a_core::mdl::write_binary_mdl_with_animations(&creature, &animations, &options)
                .expect("native writer");
        let creature_json = serde_json::to_string(&creature).unwrap();
        let animations_json = serde_json::to_string(&animations).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();

        let wasm_bytes =
            write_mdl_bytes(&creature_json, &animations_json, &options_json).expect("WASM writer");
        let wasm_report = write_mdl_report_json(&creature_json, &animations_json, &options_json);
        assert_eq!(wasm_bytes, native.payload);
        assert_eq!(wasm_report, serde_json::to_string(&native.report).unwrap());

        let invalid = write_mdl_report_json(
            &creature_json.replacen("{", "{\"unknown\":true,", 1),
            &animations_json,
            &options_json,
        );
        let error: serde_json::Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(error["code"], "M4A-CREATURE-JSON-INVALID");
        assert_eq!(error["path"], "creatureJson");

        let invalid = write_mdl_report_json(
            &creature_json,
            &animations_json.replacen("{", "{\"unknown\":true,", 1),
            &options_json,
        );
        let error: serde_json::Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(error["code"], "M4A-ANIMATION-JSON-INVALID");
        assert_eq!(error["path"], "animationsJson");

        let invalid = write_mdl_report_json(
            &creature_json,
            &animations_json,
            &options_json.replacen("{", "{\"unknown\":true,", 1),
        );
        let error: serde_json::Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(error["code"], "M4A-OPTIONS-JSON-INVALID");
        assert_eq!(error["path"], "optionsJson");
    }

    #[wasm_bindgen_test]
    fn public_adapter_returns_stable_json_error_for_empty_input() {
        let first = inspect_binary_mdl(&[]);
        let second = inspect_binary_mdl(&[]);

        assert_eq!(first, second);
        let error: serde_json::Value =
            serde_json::from_str(&first).expect("adapter must return JSON");
        assert_eq!(error["schemaVersion"], 1);
        assert_eq!(error["severity"], "error");
        assert_eq!(error["code"], "M2A-MDL-HEADER-INVALID");
        assert!(error["offset"].is_number());
        assert!(error["context"].is_string());
    }

    #[wasm_bindgen_test]
    fn profile_a_rigid_and_skin_are_native_wasm_byte_identical() {
        let glb = profile_support::minimal_glb();
        for (rig, expected_sha, expected_length, expected_deformation) in [
            (
                profile_support::rigid_profile(),
                profile_support::RIGID_JSON_SHA256,
                profile_support::RIGID_JSON_LENGTH,
                "RIGID",
            ),
            (
                profile_support::skin_multi_profile(),
                profile_support::SKIN_JSON_SHA256,
                profile_support::SKIN_JSON_LENGTH,
                "SKIN",
            ),
        ] {
            let options = profile_support::options_json(&profile_support::options());
            let rig_json = profile_support::profile_json(&rig);
            let first = convert_profile_a_json(&glb, &rig_json, &options);
            let second = convert_profile_a_json(&glb, &rig_json, &options);
            assert_eq!(first, second);
            assert_eq!(first, convert_profile_a_glb_json(&glb, &rig_json, &options));
            assert_eq!(first.len(), expected_length);
            assert_eq!(profile_support::sha256(&first), expected_sha);
            let outcome: serde_json::Value = serde_json::from_str(&first).unwrap();
            assert_eq!(outcome["report"]["conversionEligible"], true);
            assert_eq!(
                outcome["creature"]["segments"][0]["deformation"],
                expected_deformation
            );
        }
    }

    #[wasm_bindgen_test]
    fn animated_profile_a_adapter_is_native_wasm_json_identical_and_strict() {
        let glb = profile_support::linear_animated_glb();
        let rig = profile_support::animated_profile();
        let options = profile_support::options();
        let mapping = profile_support::animation_mapping();
        let source = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default())
            .expect("controlled animated source ingest");
        let core = m2a_core::profile_a::convert_profile_a_with_animations_v1(
            &source, &rig, &options, &mapping,
        )
        .expect("controlled animated core conversion");
        let rig_json = serde_json::to_string(&rig).unwrap();
        let options_json = serde_json::to_string(&options).unwrap();
        let mapping_json = serde_json::to_string(&mapping).unwrap();
        let actual = convert_profile_a_with_animations_glb_json(
            &glb,
            &rig_json,
            &options_json,
            &mapping_json,
        );
        assert_eq!(actual, serde_json::to_string(&core).unwrap());
        assert_eq!(actual.len(), profile_support::ANIMATED_JSON_LENGTH);
        assert_eq!(
            profile_support::sha256(&actual),
            profile_support::ANIMATED_JSON_SHA256
        );

        let malformed =
            convert_profile_a_with_animations_glb_json(&glb, &rig_json, &options_json, "{");
        let error: serde_json::Value = serde_json::from_str(&malformed).unwrap();
        assert_eq!(error["code"], "M4A-MAPPING-JSON-INVALID");
        assert_eq!(error["severity"], "FATAL");
        assert_eq!(error["path"], "mappingJson");

        let mut unknown = serde_json::to_value(&mapping).unwrap();
        unknown["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_with_animations_glb_json(
                &glb,
                &rig_json,
                &options_json,
                &unknown.to_string(),
            ),
            malformed
        );
    }

    #[wasm_bindgen_test]
    fn profile_a_json_boundary_and_fatal_paths_match_core_exactly() {
        let glb = profile_support::minimal_glb();
        let rig = profile_support::rigid_profile();
        let rig_json = profile_support::profile_json(&rig);
        let options = profile_support::options();
        let options_json = profile_support::options_json(&options);

        let malformed = convert_profile_a_json(&glb, "{", &options_json);
        assert_eq!(malformed, convert_profile_a_json(&glb, "[]", &options_json));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&malformed).unwrap()["code"],
            "M3A-PROFILE-JSON-INVALID"
        );
        assert_eq!(malformed.len(), profile_support::MALFORMED_JSON_LENGTH);
        assert_eq!(
            profile_support::sha256(&malformed),
            profile_support::MALFORMED_JSON_SHA256
        );

        let mut unknown_rig = serde_json::to_value(&rig).unwrap();
        unknown_rig["unknownField"] = serde_json::json!(true);
        assert_eq!(
            convert_profile_a_json(&glb, &unknown_rig.to_string(), &options_json),
            malformed
        );
        let mut unknown_options = serde_json::to_value(&options).unwrap();
        unknown_options["unknownField"] = serde_json::json!(true);
        let options_error = convert_profile_a_json(&glb, &rig_json, &unknown_options.to_string());
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&options_error).unwrap()["code"],
            "M3A-OPTIONS-INVALID"
        );

        let truncated = &glb[..glb.len() - 1];
        let expected_ingest =
            m2a_core::glb::ingest_glb(truncated, &m2a_core::glb::GlbLimits::default()).unwrap_err();
        assert_eq!(
            convert_profile_a_json(truncated, &rig_json, &options_json),
            serde_json::to_string(&expected_ingest).unwrap()
        );

        let mut limited = options.clone();
        limited.limits.max_distance_evaluations = 2;
        let limited_json =
            convert_profile_a_json(&glb, &rig_json, &profile_support::options_json(&limited));
        let source = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default()).unwrap();
        let expected_limit =
            m2a_core::profile_a::convert_profile_a(&source, &rig, &limited).unwrap_err();
        assert_eq!(
            limited_json,
            serde_json::to_string(&expected_limit).unwrap()
        );
        assert_eq!(expected_limit.code, "M3A-LIMIT-EXCEEDED");
        assert_eq!(limited_json.len(), profile_support::LIMIT_FATAL_JSON_LENGTH);
        assert_eq!(
            profile_support::sha256(&limited_json),
            profile_support::LIMIT_FATAL_JSON_SHA256
        );

        let mut provenance = rig.clone();
        provenance.provenance.kind = m2a_core::profile_a::RigProvenanceKindV1::ReferenceOnly;
        provenance.provenance.export_allowed = false;
        provenance.content_sha256.clear();
        provenance.content_sha256 =
            m2a_core::profile_a::canonical_profile_sha256(&provenance).unwrap();
        let provenance_json = convert_profile_a_json(
            &glb,
            &profile_support::profile_json(&provenance),
            &options_json,
        );
        let expected_provenance =
            m2a_core::profile_a::convert_profile_a(&source, &provenance, &options).unwrap();
        assert_eq!(
            provenance_json,
            serde_json::to_string(&expected_provenance).unwrap()
        );
        let blocked: serde_json::Value = serde_json::from_str(&provenance_json).unwrap();
        assert!(blocked["creature"].is_null());
        assert!(
            blocked["report"]["gates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|gate| gate["code"] == "M3A-PROFILE-PROVENANCE-FORBIDDEN")
        );
    }

    #[wasm_bindgen_test]
    fn public_adapter_returns_deterministic_json_for_synthetic_mdl() {
        let mdl = minimal_binary_mdl();
        let original = mdl.clone();
        let first = inspect_binary_mdl(&mdl);
        let second = inspect_binary_mdl(&mdl);

        assert_eq!(mdl, original, "adapter must not mutate selected-file bytes");
        assert_eq!(first, second);

        let report: serde_json::Value =
            serde_json::from_str(&first).expect("adapter must return JSON");
        assert_eq!(report["schemaVersion"], 1);
        assert_eq!(report["format"], "nwn1-binary-mdl");
        assert_eq!(report["byteLength"].as_u64(), Some(mdl.len() as u64));
        assert_eq!(report["nodeTree"]["nodeCount"], 1);
    }

    #[wasm_bindgen_test]
    fn public_adapter_exposes_deep_m1b_report_deterministically() {
        let mdl = fixtures::build_deep_binary_mdl();
        let first = inspect_binary_mdl(&mdl);
        let second = inspect_binary_mdl(&mdl);

        assert_eq!(first, second);

        let report: serde_json::Value =
            serde_json::from_str(&first).expect("adapter must return JSON");
        let root = &report["nodeTree"]["roots"][0];
        let controllers = root["controllers"]
            .as_array()
            .expect("deep fixture controllers");
        assert_eq!(controllers.len(), 5);
        assert_eq!(controllers[0]["controllerName"], "position");
        assert_eq!(controllers[0]["packedByte"], 3);
        assert_eq!(controllers[0]["interpolationFlags"], 0);
        assert_eq!(controllers[0]["decoded"], true);
        assert_eq!(controllers[1]["controllerName"], "orientation");
        assert_eq!(controllers[1]["packedByte"], 4);
        assert_eq!(controllers[1]["interpolationFlags"], 0);
        assert_eq!(controllers[1]["decoded"], true);
        assert_eq!(controllers[4]["controllerName"], "alpha");

        let mesh = root["mesh"].as_object().expect("deep fixture mesh");
        assert_eq!(mesh["vertexCount"], 3);
        assert_eq!(mesh["faces"].as_array().map(Vec::len), Some(1));
        assert_eq!(mesh["textures"][0], "m2a_diffuse");

        let animations = report["animations"]
            .as_array()
            .expect("deep fixture animations");
        assert_eq!(animations.len(), 2);
        assert_eq!(animations[0]["name"], "walk");
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["controllerName"],
            "position"
        );
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["packedByte"],
            3
        );
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["interpolationFlags"],
            0
        );
        assert_eq!(
            animations[0]["nodeTree"]["roots"][0]["controllers"][0]["decoded"],
            true
        );
        assert_eq!(animations[1]["name"], "idle");
    }

    #[wasm_bindgen_test]
    fn public_adapter_exposes_both_m1b_skin_variants_deterministically() {
        for (extended64, expected_variant, expected_inline_count) in
            [(false, "legacy17", 17), (true, "extended64", 64)]
        {
            let mdl = fixtures::build_skin_binary_mdl(extended64);
            let first = inspect_binary_mdl(&mdl);
            let second = inspect_binary_mdl(&mdl);

            assert_eq!(first, second);

            let report: serde_json::Value =
                serde_json::from_str(&first).expect("adapter must return JSON");
            let root = &report["nodeTree"]["roots"][0];
            assert!(root["mesh"].is_object());
            assert_eq!(root["skin"]["variant"], expected_variant);
            assert_eq!(
                root["skin"]["inlineMapping"].as_array().map(Vec::len),
                Some(expected_inline_count)
            );
            assert_eq!(
                root["skin"]["nodeToBoneMap"].as_array().map(Vec::len),
                Some(3)
            );
            assert_eq!(
                root["skin"]["vertexWeights"].as_array().map(Vec::len),
                Some(3)
            );
            assert_eq!(
                root["skin"]["boneReferences"].as_array().map(Vec::len),
                Some(3)
            );
        }
    }

    #[wasm_bindgen_test]
    fn public_glb_adapters_match_core_json_and_are_deterministic() {
        let glb = minimal_synthetic_glb();
        let original = glb.clone();

        let inspect_first = inspect_glb(&glb);
        let inspect_second = inspect_glb(&glb);
        let expected_report =
            m2a_core::glb::inspect_glb(&glb, &m2a_core::glb::GlbLimits::default())
                .expect("synthetic GLB report");
        assert_eq!(inspect_first, inspect_second);
        assert_eq!(
            inspect_first,
            serde_json::to_string(&expected_report).unwrap()
        );

        let ingest_first = ingest_glb(&glb);
        let ingest_second = ingest_glb(&glb);
        let expected_ingest = m2a_core::glb::ingest_glb(&glb, &m2a_core::glb::GlbLimits::default())
            .expect("synthetic GLB ingest");
        assert_eq!(ingest_first, ingest_second);
        assert_eq!(
            ingest_first,
            serde_json::to_string(&expected_ingest).unwrap()
        );
        assert_eq!(glb, original, "WASM adapters must not mutate source bytes");

        let report: serde_json::Value = serde_json::from_str(&inspect_first).unwrap();
        assert_eq!(report["schemaVersion"], 1);
        assert_eq!(report["format"], "GLB_2_0");
        assert_eq!(report["inventory"]["sceneCount"], 1);
        assert_eq!(report["statistics"]["triangleCount"], 1);
        assert_eq!(report["coordinatePolicy"]["storedSpace"], "GLTF_SOURCE");

        let result: serde_json::Value = serde_json::from_str(&ingest_first).unwrap();
        assert_eq!(result["schemaVersion"], 1);
        assert_eq!(result["ir"]["schemaVersion"], 1);
        assert_eq!(
            result["ir"]["primitives"][0]["indices"],
            serde_json::json!([0, 1, 2])
        );
        assert_eq!(result["report"], report);
    }

    #[wasm_bindgen_test]
    fn public_glb_adapters_return_the_same_stable_empty_error_as_core() {
        let expected =
            m2a_core::glb::inspect_glb(&[], &m2a_core::glb::GlbLimits::default()).unwrap_err();
        let expected_json = serde_json::to_string(&expected).unwrap();

        assert_eq!(inspect_glb(&[]), expected_json);
        assert_eq!(ingest_glb(&[]), expected_json);
        assert_eq!(inspect_glb(&[]), inspect_glb(&[]));
        assert_eq!(ingest_glb(&[]), ingest_glb(&[]));

        let error: serde_json::Value = serde_json::from_str(&expected_json).unwrap();
        assert_eq!(error["schemaVersion"], 1);
        assert_eq!(error["code"], "M2A-GLB-INPUT-EMPTY");
        assert!(error["message"].is_string());
    }

    #[wasm_bindgen_test]
    fn public_contract_names_and_legacy_aliases_are_byte_identical() {
        let glb = minimal_synthetic_glb();

        assert_eq!(inspect_glb_json(&glb), inspect_glb(&glb));
        assert_eq!(ingest_glb_json(&glb), ingest_glb(&glb));
    }

    #[wasm_bindgen_test]
    fn public_fixture_d_preserves_material_image_identity_without_payload() {
        let glb = material_image_synthetic_glb();
        let (report, result) = assert_public_glb_parity(&glb);

        assert_eq!(report["inventory"]["primitiveCount"], 2);
        assert_eq!(report["inventory"]["materialCount"], 2);
        assert_eq!(report["inventory"]["textureCount"], 1);
        assert_eq!(report["inventory"]["samplerCount"], 1);
        assert_eq!(report["inventory"]["imageCount"], 1);

        let material = &result["ir"]["materials"][0];
        assert_eq!(
            material["baseColorFactor"],
            serde_json::json!([0.8, 0.7, 0.6, 0.5])
        );
        assert_eq!(material["baseColorTexture"]["textureId"], 0);
        assert_eq!(material["metallicFactor"], 0.35);
        assert_eq!(material["roughnessFactor"], 0.65);
        assert_eq!(material["alphaMode"], "MASK");

        let image = &result["ir"]["images"][0];
        assert_eq!(image["name"], "wasm-one-pixel");
        assert_eq!(image["mimeType"], "image/png");
        assert_eq!(image["byteLength"], MINIMAL_PNG.len());
        assert_eq!(image["payloadEmbeddedInJson"], false);
        assert_eq!(image["sha256"].as_str().map(str::len), Some(64));

        let json = ingest_glb_json(&glb);
        assert!(!json.contains("data:image"));
        assert!(!json.contains("iVBORw0KGgo"));
        assert!(!json.contains("imageBytes"));
        assert!(!json.contains("\"payload\":"));
        assert!(!json.contains("C:\\\\"));
    }

    #[wasm_bindgen_test]
    fn public_fixture_e_preserves_skin_and_animation_schema_in_source_basis() {
        let glb = skin_animation_synthetic_glb(false);
        let (report, result) = assert_public_glb_parity(&glb);

        assert_eq!(report["inventory"]["skinCount"], 1);
        assert_eq!(report["inventory"]["jointReferenceCount"], 2);
        assert_eq!(report["inventory"]["animationCount"], 1);
        assert_eq!(report["inventory"]["keyframeCount"], 9);

        let skin = &result["ir"]["skins"][0];
        assert_eq!(skin["skeletonRootNodeId"], 0);
        assert_eq!(skin["jointNodeIds"], serde_json::json!([0, 1]));
        assert_eq!(
            skin["inverseBindMatrices"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(skin["inverseBindMatrices"][1][12], 2.0);
        assert_eq!(skin["inverseBindMatrices"][1][13], 3.0);
        assert_eq!(skin["inverseBindMatrices"][1][14], 4.0);

        let primitive = &result["ir"]["primitives"][0];
        assert_eq!(primitive["joints0"][0], serde_json::json!([0, 1, 0, 0]));
        assert_eq!(
            primitive["weights0"][0],
            serde_json::json!([0.75, 0.25, 0.0, 0.0])
        );

        let animation = &result["ir"]["animations"][0];
        assert_eq!(animation["durationSeconds"], 1.25);
        assert_eq!(animation["samplers"][0]["interpolation"], "LINEAR");
        assert_eq!(animation["samplers"][1]["interpolation"], "STEP");
        assert_eq!(animation["samplers"][2]["interpolation"], "CUBICSPLINE");
        assert_eq!(
            animation["samplers"][2]["outputValues"]
                .as_array()
                .map(Vec::len),
            Some(27)
        );
        assert_eq!(animation["channels"][0]["targetPath"], "TRANSLATION");
        assert_eq!(animation["channels"][1]["targetPath"], "ROTATION");
        assert_eq!(animation["channels"][2]["targetPath"], "SCALE");

        let without_inverse_bind = mutate_synthetic_glb(glb, |root| {
            root["skins"][0]
                .as_object_mut()
                .unwrap()
                .remove("inverseBindMatrices");
        });
        let (_, without_inverse_bind) = assert_public_glb_parity(&without_inverse_bind);
        assert_eq!(
            without_inverse_bind["ir"]["skins"][0]["inverseBindMatrices"],
            serde_json::json!([])
        );
    }

    #[wasm_bindgen_test]
    fn public_fixture_e_weights_channel_is_explicitly_blocking() {
        let glb = skin_animation_synthetic_glb(true);
        let (report, result) = assert_public_glb_parity(&glb);

        assert_eq!(
            result["ir"]["animations"][0]["channels"][3]["targetPath"],
            "WEIGHTS"
        );
        assert_eq!(report["conversionEligible"], false);
        assert_gate(&report, "M2A-GLB-ANIMATION-WEIGHTS-DEFERRED", "BLOCKING");
    }

    #[wasm_bindgen_test]
    fn public_fixture_f_exposes_stable_blocking_gates() {
        for (glb, code) in [
            (missing_uv_synthetic_glb(), "M2A-GLB-UV0-MISSING"),
            (missing_position_synthetic_glb(), "M2A-GLB-POSITION-MISSING"),
            (
                morph_target_synthetic_glb(),
                "M2A-GLB-MORPH-TARGETS-DEFERRED",
            ),
            (
                mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
                    root["meshes"][0]["primitives"][0]["mode"] = serde_json::json!(1);
                }),
                "M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED",
            ),
            (
                mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
                    root["accessors"][2]["count"] = serde_json::json!(2);
                }),
                "M2A-GLB-ATTRIBUTE-COUNT-MISMATCH",
            ),
        ] {
            let (report, result) = assert_public_glb_parity(&glb);
            assert_eq!(report["conversionEligible"], false);
            assert_gate(&report, code, "BLOCKING");
            assert_eq!(result["report"], report);
        }
    }

    #[wasm_bindgen_test]
    fn public_glb_contract_returns_stable_fatal_errors_and_never_accepts_truncation() {
        let complete = material_image_synthetic_glb();
        let mut bad_magic = complete.clone();
        bad_magic[0] = b'X';
        let mut bad_version = complete.clone();
        bad_version[4..8].copy_from_slice(&1_u32.to_le_bytes());
        let mut bad_length = complete.clone();
        bad_length[8..12].copy_from_slice(&0_u32.to_le_bytes());

        for (bytes, code) in [
            (Vec::new(), "M2A-GLB-INPUT-EMPTY"),
            (bad_magic, "M2A-GLB-HEADER-INVALID"),
            (bad_version, "M2A-GLB-VERSION-UNSUPPORTED"),
            (bad_length, "M2A-GLB-LENGTH-MISMATCH"),
            (oversized_json_chunk_glb(), "M2A-GLB-LIMIT-EXCEEDED"),
        ] {
            assert_stable_fatal_parity(&bytes, code);
        }

        let fixture = minimal_synthetic_glb();
        for length in 0..fixture.len() {
            let prefix = &fixture[..length];
            let inspect = inspect_glb_json(prefix);
            let ingest = ingest_glb_json(prefix);
            assert_eq!(inspect, inspect_glb_json(prefix));
            assert_eq!(ingest, ingest_glb_json(prefix));
            let inspect_error: serde_json::Value = serde_json::from_str(&inspect).unwrap();
            let ingest_error: serde_json::Value = serde_json::from_str(&ingest).unwrap();
            assert_eq!(inspect_error["schemaVersion"], 1);
            assert_eq!(ingest_error["schemaVersion"], 1);
            assert!(
                inspect_error["code"]
                    .as_str()
                    .is_some_and(|code| code.starts_with("M2A-GLB-"))
            );
            assert!(
                ingest_error["code"]
                    .as_str()
                    .is_some_and(|code| code.starts_with("M2A-GLB-"))
            );
        }
    }

    fn assert_public_glb_parity(glb: &[u8]) -> (serde_json::Value, serde_json::Value) {
        let original = glb.to_vec();
        let inspect_first = inspect_glb_json(glb);
        let inspect_second = inspect_glb_json(glb);
        let ingest_first = ingest_glb_json(glb);
        let ingest_second = ingest_glb_json(glb);

        assert_eq!(
            glb,
            original.as_slice(),
            "public adapters must not mutate input"
        );
        assert_eq!(inspect_first, inspect_second);
        assert_eq!(ingest_first, ingest_second);
        assert_eq!(inspect_first, inspect_glb(glb));
        assert_eq!(ingest_first, ingest_glb(glb));

        let core_report =
            m2a_core::glb::inspect_glb(glb, &m2a_core::glb::GlbLimits::default()).unwrap();
        let core_result =
            m2a_core::glb::ingest_glb(glb, &m2a_core::glb::GlbLimits::default()).unwrap();
        assert_eq!(inspect_first, serde_json::to_string(&core_report).unwrap());
        assert_eq!(ingest_first, serde_json::to_string(&core_result).unwrap());

        (
            serde_json::from_str(&inspect_first).unwrap(),
            serde_json::from_str(&ingest_first).unwrap(),
        )
    }

    fn assert_stable_fatal_parity(bytes: &[u8], expected_code: &str) {
        let expected =
            m2a_core::glb::inspect_glb(bytes, &m2a_core::glb::GlbLimits::default()).unwrap_err();
        let expected_json = serde_json::to_string(&expected).unwrap();
        assert_eq!(expected.code, expected_code);
        assert_eq!(inspect_glb_json(bytes), expected_json);
        assert_eq!(ingest_glb_json(bytes), expected_json);
        assert_eq!(inspect_glb(bytes), expected_json);
        assert_eq!(ingest_glb(bytes), expected_json);
    }

    fn assert_gate(report: &serde_json::Value, code: &str, severity: &str) {
        assert!(
            report["gates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|gate| { gate["code"] == code && gate["severity"] == severity })
        );
    }

    fn minimal_binary_mdl() -> Vec<u8> {
        const FILE_HEADER_SIZE: usize = 0x0c;
        const MODEL_HEADER_SIZE: usize = 0xe8;
        const NODE_HEADER_SIZE: usize = 0x70;
        const ROOT_NODE_OFFSET: u32 = MODEL_HEADER_SIZE as u32;
        const MODEL_DATA_SIZE: usize = MODEL_HEADER_SIZE + NODE_HEADER_SIZE;

        let mut bytes = vec![0_u8; FILE_HEADER_SIZE + MODEL_DATA_SIZE];

        write_u32(&mut bytes, 0x00, 0);
        write_u32(&mut bytes, 0x04, MODEL_DATA_SIZE as u32);
        write_u32(&mut bytes, 0x08, 0);

        let model = FILE_HEADER_SIZE;
        write_c_string(&mut bytes, model + 0x08, 64, "m2a_minimal");
        write_u32(&mut bytes, model + 0x48, ROOT_NODE_OFFSET);
        write_u32(&mut bytes, model + 0x4c, 1);

        let root = FILE_HEADER_SIZE + ROOT_NODE_OFFSET as usize;
        write_u32(&mut bytes, root + 0x1c, 0);
        write_c_string(&mut bytes, root + 0x20, 32, "root");
        write_u32(&mut bytes, root + 0x6c, 0x001);

        bytes
    }

    fn minimal_synthetic_glb() -> Vec<u8> {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = [[0.0_f32, 0.0, 1.0]; 3];
        let uv0 = [[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = [0_u16, 1, 2];
        let mut bin = Vec::new();

        let positions_offset = bin.len();
        append_f32_rows(&mut bin, &positions);
        let positions_length = bin.len() - positions_offset;
        let normals_offset = bin.len();
        append_f32_rows(&mut bin, &normals);
        let normals_length = bin.len() - normals_offset;
        let uv_offset = bin.len();
        append_f32_rows(&mut bin, &uv0);
        let uv_length = bin.len() - uv_offset;
        let indices_offset = bin.len();
        for index in indices {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        let indices_length = bin.len() - indices_offset;
        align4(&mut bin, 0);

        let root = serde_json::json!({
            "asset": {"version": "2.0", "generator": "m2a-wasm-synthetic"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"name": "root", "mesh": 0}],
            "meshes": [{"primitives": [{
                "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                "indices": 3,
                "mode": 4
            }]}],
            "buffers": [{"byteLength": bin.len()}],
            "bufferViews": [
                {"buffer": 0, "byteOffset": positions_offset, "byteLength": positions_length},
                {"buffer": 0, "byteOffset": normals_offset, "byteLength": normals_length},
                {"buffer": 0, "byteOffset": uv_offset, "byteLength": uv_length},
                {"buffer": 0, "byteOffset": indices_offset, "byteLength": indices_length}
            ],
            "accessors": [
                {
                    "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
                    "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]
                },
                {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3"},
                {"bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2"},
                {"bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR"}
            ]
        });
        let mut json = serde_json::to_vec(&root).unwrap();
        align4(&mut json, b' ');

        let total_length = 12 + 8 + json.len() + 8 + bin.len();
        let mut glb = Vec::with_capacity(total_length);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin);
        glb
    }

    const MINIMAL_PNG: [u8; 68] = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xb5,
        0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xfc,
        0xff, 0x1f, 0x00, 0x03, 0x03, 0x02, 0x00, 0xef, 0xa3, 0xe1, 0x1d, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    fn material_image_synthetic_glb() -> Vec<u8> {
        let (mut root, mut bin) = split_synthetic_glb(&minimal_synthetic_glb());
        align4(&mut bin, 0);
        let image_offset = bin.len();
        bin.extend_from_slice(&MINIMAL_PNG);
        let image_view = push_view(&mut root, image_offset, MINIMAL_PNG.len());
        root["buffers"][0]["byteLength"] = serde_json::json!(bin.len());

        let first = root["meshes"][0]["primitives"][0].clone();
        root["meshes"][0]["primitives"] = serde_json::json!([first.clone(), first]);
        root["meshes"][0]["primitives"][0]["material"] = serde_json::json!(0);
        root["meshes"][0]["primitives"][1]["material"] = serde_json::json!(1);
        root["samplers"] = serde_json::json!([{
            "name": "wasm-sampler",
            "magFilter": 9728,
            "minFilter": 9987,
            "wrapS": 33071,
            "wrapT": 33648
        }]);
        root["images"] = serde_json::json!([{
            "name": "wasm-one-pixel",
            "bufferView": image_view,
            "mimeType": "image/png"
        }]);
        root["textures"] = serde_json::json!([{"sampler": 0, "source": 0}]);
        root["materials"] = serde_json::json!([
            {
                "name": "wasm-painted-mask",
                "pbrMetallicRoughness": {
                    "baseColorFactor": [0.8, 0.7, 0.6, 0.5],
                    "baseColorTexture": {"index": 0, "texCoord": 0},
                    "metallicFactor": 0.35,
                    "roughnessFactor": 0.65
                },
                "alphaMode": "MASK",
                "alphaCutoff": 0.33,
                "doubleSided": true
            },
            {
                "name": "wasm-detail",
                "pbrMetallicRoughness": {
                    "baseColorTexture": {"index": 0},
                    "metallicFactor": 0.05,
                    "roughnessFactor": 0.15
                }
            }
        ]);
        make_synthetic_glb(root, bin)
    }

    fn skin_animation_synthetic_glb(include_weights_channel: bool) -> Vec<u8> {
        let (mut root, mut bin) = split_synthetic_glb(&minimal_synthetic_glb());

        align4(&mut bin, 0);
        let joints_offset = bin.len();
        for joints in [[0_u8, 1, 0, 0], [1, 0, 0, 0], [0, 1, 0, 0]] {
            bin.extend_from_slice(&joints);
        }
        let joints_view = push_view(&mut root, joints_offset, 12);
        let joints_accessor = push_accessor(
            &mut root,
            serde_json::json!({
                "bufferView": joints_view,
                "componentType": 5121,
                "count": 3,
                "type": "VEC4"
            }),
        );

        let weights_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[0.75, 0.25, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.4, 0.6, 0.0, 0.0],
            3,
            "VEC4",
            None,
            None,
        );
        root["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"] =
            serde_json::json!(joints_accessor);
        root["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"] =
            serde_json::json!(weights_accessor);

        root["nodes"] = serde_json::json!([
            {"name": "rig-root", "children": [1, 2]},
            {"name": "joint-one", "translation": [0.0, 1.0, 0.0]},
            {"name": "skinned-mesh", "mesh": 0, "skin": 0}
        ]);
        root["scenes"][0]["nodes"] = serde_json::json!([0]);

        let inverse_bind_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
            ],
            2,
            "MAT4",
            None,
            None,
        );
        root["skins"] = serde_json::json!([{
            "name": "wasm-two-joint-skin",
            "joints": [0, 1],
            "skeleton": 0,
            "inverseBindMatrices": inverse_bind_accessor
        }]);

        let times_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[0.0, 0.5, 1.25],
            3,
            "SCALAR",
            Some(serde_json::json!([0.0])),
            Some(serde_json::json!([1.25])),
        );
        let translation_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            3,
            "VEC3",
            None,
            None,
        );
        let rotation_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[
                0.0, 0.0, 0.0, 1.0, 0.0, 0.70710677, 0.0, 0.70710677, 0.0, 1.0, 0.0, 0.0,
            ],
            3,
            "VEC4",
            None,
            None,
        );
        let scale_accessor = push_f32_accessor(
            &mut root,
            &mut bin,
            &[
                -0.1, 0.0, 0.0, 1.0, 1.0, 1.0, 0.1, 0.0, 0.0, -0.2, 0.0, 0.0, 1.5, 2.0, 2.5, 0.2,
                0.0, 0.0, -0.3, 0.0, 0.0, 2.0, 3.0, 4.0, 0.3, 0.0, 0.0,
            ],
            9,
            "VEC3",
            None,
            None,
        );
        let mut channels = vec![
            serde_json::json!({"sampler": 0, "target": {"node": 1, "path": "translation"}}),
            serde_json::json!({"sampler": 1, "target": {"node": 1, "path": "rotation"}}),
            serde_json::json!({"sampler": 2, "target": {"node": 1, "path": "scale"}}),
        ];
        if include_weights_channel {
            channels
                .push(serde_json::json!({"sampler": 0, "target": {"node": 2, "path": "weights"}}));
        }
        root["animations"] = serde_json::json!([{
            "name": "wasm-source-trs",
            "samplers": [
                {"input": times_accessor, "output": translation_accessor, "interpolation": "LINEAR"},
                {"input": times_accessor, "output": rotation_accessor, "interpolation": "STEP"},
                {"input": times_accessor, "output": scale_accessor, "interpolation": "CUBICSPLINE"}
            ],
            "channels": channels
        }]);
        root["buffers"][0]["byteLength"] = serde_json::json!(bin.len());
        make_synthetic_glb(root, bin)
    }

    fn missing_uv_synthetic_glb() -> Vec<u8> {
        mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("TEXCOORD_0");
        })
    }

    fn missing_position_synthetic_glb() -> Vec<u8> {
        mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
            root["meshes"][0]["primitives"][0]["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("POSITION");
        })
    }

    fn morph_target_synthetic_glb() -> Vec<u8> {
        mutate_synthetic_glb(minimal_synthetic_glb(), |root| {
            root["meshes"][0]["primitives"][0]["targets"] = serde_json::json!([{"POSITION": 0}]);
        })
    }

    fn oversized_json_chunk_glb() -> Vec<u8> {
        const OVERSIZED_JSON_LENGTH: usize = 16 * 1024 * 1024 + 4;
        let total_length = 12 + 8 + OVERSIZED_JSON_LENGTH;
        let mut glb = Vec::with_capacity(total_length);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());
        glb.extend_from_slice(&(OVERSIZED_JSON_LENGTH as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.resize(total_length, b' ');
        glb
    }

    fn push_f32_accessor(
        root: &mut serde_json::Value,
        bin: &mut Vec<u8>,
        values: &[f32],
        count: usize,
        element_type: &str,
        min: Option<serde_json::Value>,
        max: Option<serde_json::Value>,
    ) -> usize {
        align4(bin, 0);
        let offset = bin.len();
        for value in values {
            bin.extend_from_slice(&value.to_le_bytes());
        }
        let view = push_view(root, offset, bin.len() - offset);
        let mut accessor = serde_json::json!({
            "bufferView": view,
            "componentType": 5126,
            "count": count,
            "type": element_type
        });
        if let Some(min) = min {
            accessor["min"] = min;
        }
        if let Some(max) = max {
            accessor["max"] = max;
        }
        push_accessor(root, accessor)
    }

    fn push_view(root: &mut serde_json::Value, offset: usize, length: usize) -> usize {
        let views = root["bufferViews"].as_array_mut().unwrap();
        let index = views.len();
        views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": length
        }));
        index
    }

    fn push_accessor(root: &mut serde_json::Value, accessor: serde_json::Value) -> usize {
        let accessors = root["accessors"].as_array_mut().unwrap();
        let index = accessors.len();
        accessors.push(accessor);
        index
    }

    fn mutate_synthetic_glb(
        glb: Vec<u8>,
        mutation: impl FnOnce(&mut serde_json::Value),
    ) -> Vec<u8> {
        let (mut root, bin) = split_synthetic_glb(&glb);
        mutation(&mut root);
        make_synthetic_glb(root, bin)
    }

    fn split_synthetic_glb(glb: &[u8]) -> (serde_json::Value, Vec<u8>) {
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json_end = 20 + json_length;
        let root = serde_json::from_slice(&glb[20..json_end]).unwrap();
        let bin_start = json_end + 8;
        (root, glb[bin_start..].to_vec())
    }

    fn make_synthetic_glb(root: serde_json::Value, mut bin: Vec<u8>) -> Vec<u8> {
        let mut json = serde_json::to_vec(&root).unwrap();
        align4(&mut json, b' ');
        align4(&mut bin, 0);
        let total_length = 12 + 8 + json.len() + 8 + bin.len();
        let mut glb = Vec::with_capacity(total_length);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
        glb.extend_from_slice(&json);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin);
        glb
    }

    fn append_f32_rows<const N: usize>(bytes: &mut Vec<u8>, rows: &[[f32; N]]) {
        for row in rows {
            for value in row {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    fn align4(bytes: &mut Vec<u8>, padding: u8) {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(padding);
        }
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_c_string(bytes: &mut [u8], offset: usize, capacity: usize, value: &str) {
        assert!(value.len() < capacity);
        bytes[offset..offset + value.len()].copy_from_slice(value.as_bytes());
    }
}
