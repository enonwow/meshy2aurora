//! Complete offline GLB-to-Aurora material/package vertical slice.
//!
//! The result proves offline package integrity. It deliberately does not mark
//! the in-memory artifact as `ready_for_owner_proof`: that status requires an
//! exact frozen candidate plus verified native MOD/HAK installation.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    aurora_material::{
        AuroraMaterialCompileStatusV1, AuroraMaterialTargetProfileV1, AuroraRenderHintV1,
        compile_gltf_materials_v1,
    },
    aurora_material_bake::bake_specular_gloss_v1,
    erf::ErfArchive,
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakArtifactV1, HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    mdl::{
        MdlFormatProfileV1, MdlMaterialExtensionOptionsV1, MdlMaterialStateV1,
        MdlMaterialTextureBindingV1, MdlRenderHintV1, MdlSegmentMaterialStreamsV1,
        MdlStateProjectionProfileV1, MdlWriterOptionsV1, inspect_binary_mdl,
        write_binary_mdl_with_materials_v1,
    },
    model_ir::AuroraModelIrV1,
    model_pipeline::append_direct_creature_humanoid_appearance_row_v1,
    mtr::{
        MTR_RESOURCE_TYPE_V1, MtrMaterialResourceNamesV1, compile_mtr_document_v1, parse_mtr_v1,
        write_mtr_v1,
    },
    placeable::static_placeable_profile_a_options_v1,
    profile_a::{convert_profile_a, derive_meshy_m0_static_rigid_profile_v1},
    proof_module::{
        BinaryM0VerticalSliceIdentityV1, ProofModuleArtifactV1,
        build_binary_m0_vertical_slice_module_with_identity_v1,
        inspect_binary_m0_vertical_slice_module_v1,
    },
    tga::{TgaImageV1, TgaPixelFormatV1, TgaWriterOptionsV1, read_tga_image_v1, write_tga_v1},
    txi::{TXI_RESOURCE_TYPE_V1, TxiDocumentV1, parse_txi_v1, write_txi_v1},
};

pub const MODEL_MATERIAL_E2E_SCHEMA_VERSION_V1: u32 = 1;
const MDL_RESOURCE_TYPE_V1: u16 = 2002;
const TGA_RESOURCE_TYPE_V1: u16 = 3;
const APPEARANCE_2DA_RESOURCE_TYPE_V1: u16 = 2017;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterialE2eRequestV1 {
    pub schema_version: u32,
    pub model_resref: String,
    pub hak_resref: String,
    pub module_resref: String,
    pub area_resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialE2eResourceV1 {
    pub resref: String,
    pub resource_type: u16,
    pub byte_length: u64,
    pub sha256: String,
    pub readback_equal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialE2eReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub material_profile: String,
    pub material_count: usize,
    pub model_resref: String,
    pub model_sha256: String,
    pub appearance_row: u16,
    pub hak_resref: String,
    pub hak_sha256: String,
    pub module_resref: String,
    pub module_sha256: String,
    pub area_resref: String,
    pub resources: Vec<ModelMaterialE2eResourceV1>,
    pub semantic_readback_status: String,
    pub ready_for_owner_proof: bool,
    pub owner_proof_blocker: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelMaterialE2eArtifactV1 {
    pub hak: HakArtifactV1,
    pub module: ProofModuleArtifactV1,
    pub report: ModelMaterialE2eReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMaterialE2eErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ModelMaterialE2eErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelMaterialE2eErrorV1 {}

struct MaterialResources {
    slot: u32,
    state: MdlMaterialStateV1,
    diffuse_binding: MdlMaterialTextureBindingV1,
    resources: Vec<HakResourceInputV1>,
}

pub fn build_model_material_e2e_v1(
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
    request: &ModelMaterialE2eRequestV1,
) -> Result<ModelMaterialE2eArtifactV1, ModelMaterialE2eErrorV1> {
    validate_request(request)?;
    let ingest = ingest_glb(source_glb, &GlbLimits::default())
        .map_err(|source| error(&source.code, "sourceGlb", source.message))?;
    let compiled = compile_gltf_materials_v1(
        &ingest.ir.source.sha256,
        &ingest.ir.materials,
        AuroraMaterialTargetProfileV1::NwnEeMtr,
    )
    .map_err(|source| error(&source.code, source.path, source.message))?;
    if compiled.status == AuroraMaterialCompileStatusV1::Blocked {
        return Err(error(
            "MATERIAL-E2E-COMPILER-BLOCKED",
            "materials",
            "NWN EE material compilation contains a blocked source channel",
        ));
    }
    let rig = derive_meshy_m0_static_rigid_profile_v1(&ingest)
        .map_err(|source| error(&source.code, source.path, source.message))?;
    let conversion = convert_profile_a(&ingest, &rig, &static_placeable_profile_a_options_v1())
        .map_err(|source| error(&source.code, source.path, source.message))?;
    let mut model = conversion.creature.ok_or_else(|| {
        error(
            "MATERIAL-E2E-MODEL-MISSING",
            "conversion.creature",
            "eligible conversion returned no common model IR",
        )
    })?;
    normalize_root_name(&mut model, &request.model_resref)?;
    let compiled_by_id = compiled
        .materials
        .iter()
        .map(|entry| (entry.material.source_material_id, &entry.material))
        .collect::<BTreeMap<_, _>>();
    let mut material_resources = Vec::new();
    for binding in &model.material_source_bindings {
        let source_id = binding.source_material_id.ok_or_else(|| {
            error(
                "MATERIAL-E2E-SOURCE-MATERIAL-MISSING",
                "model.materialSourceBindings",
                format!("material slot {} has no source material", binding.slot),
            )
        })?;
        let material = compiled_by_id.get(&source_id).copied().ok_or_else(|| {
            error(
                "MATERIAL-E2E-COMPILED-MATERIAL-MISSING",
                "materials",
                format!("source material {source_id} has no compiler output"),
            )
        })?;
        material_resources.push(material_resources_v1(
            source_glb,
            &ingest,
            binding.slot,
            material,
        )?);
    }
    material_resources.sort_by_key(|entry| entry.slot);
    let writer_options = MdlWriterOptionsV1 {
        schema_version: 1,
        format_profile: MdlFormatProfileV1::PlaceableStaticRigidNativeV1,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        state_projection_provenance: None,
        model_resource_resref: request.model_resref.clone(),
        diffuse_texture_resref_by_material_slot: material_resources
            .iter()
            .map(|entry| entry.diffuse_binding.clone())
            .collect(),
    };
    let extension_options = MdlMaterialExtensionOptionsV1 {
        schema_version: 1,
        materials: material_resources
            .iter()
            .map(|entry| entry.state.clone())
            .collect(),
        segment_streams: model
            .segments
            .iter()
            .map(|segment| MdlSegmentMaterialStreamsV1 {
                segment_id: segment.segment_id,
                uv1: Vec::new(),
                uv2: Vec::new(),
                uv3: Vec::new(),
            })
            .collect(),
    };
    let model_artifact =
        write_binary_mdl_with_materials_v1(&model, &writer_options, &extension_options)
            .map_err(|source| error(&source.code, source.path, source.message))?;
    let appearance = append_direct_creature_humanoid_appearance_row_v1(
        base_appearance_two_da,
        "M2A_MATERIAL_E2E_V1",
        &request.model_resref,
    )
    .map_err(|source| error(&source.code, source.path, source.message))?;
    let mut hak_resources = vec![
        HakResourceInputV1 {
            resref: request.model_resref.clone(),
            resource_type: MDL_RESOURCE_TYPE_V1,
            payload: model_artifact.binary.payload.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: APPEARANCE_2DA_RESOURCE_TYPE_V1,
            payload: appearance.payload.clone(),
        },
    ];
    for material in material_resources {
        hak_resources.extend(material.resources);
    }
    let hak = write_hak_v1(&hak_resources, &HakWriterOptionsV1::default())
        .map_err(|source| error(&source.code, source.path, source.message))?;
    let hak_archive = ErfArchive::parse(&hak.payload)
        .map_err(|source| error(&source.code, "hak", source.context))?;
    let mut resource_report = Vec::new();
    for resource in &hak_resources {
        let readback = hak_archive
            .find(&resource.resref, resource.resource_type)
            .map_err(|source| error(&source.code, "hak.resources", source.context))?;
        let readback_equal = readback == resource.payload;
        if !readback_equal {
            return Err(error(
                "MATERIAL-E2E-HAK-SEMANTIC-DIFF",
                "hak.resources",
                format!(
                    "{}:{} differs after archive readback",
                    resource.resref, resource.resource_type
                ),
            ));
        }
        validate_resource_readback(resource, readback)?;
        resource_report.push(ModelMaterialE2eResourceV1 {
            resref: resource.resref.clone(),
            resource_type: resource.resource_type,
            byte_length: resource.payload.len() as u64,
            sha256: sha256(&resource.payload),
            readback_equal,
        });
    }
    resource_report.sort_by(|left, right| {
        left.resref
            .cmp(&right.resref)
            .then(left.resource_type.cmp(&right.resource_type))
    });
    let module_identity = BinaryM0VerticalSliceIdentityV1 {
        module_resref: request.module_resref.clone(),
        area_resref: request.area_resref.clone(),
        hak_resref: request.hak_resref.clone(),
    };
    let module = build_binary_m0_vertical_slice_module_with_identity_v1(
        appearance.report.appended_row_index,
        &module_identity,
    )
    .map_err(|source| error(&source.code, source.path, source.message))?;
    let module_readback = inspect_binary_m0_vertical_slice_module_v1(&module.payload)
        .map_err(|source| error(&source.code, source.path, source.message))?;
    if module_readback.module_resref != request.module_resref
        || module_readback.area_resref != request.area_resref
        || module_readback.ordered_hak_resrefs != [request.hak_resref.clone()]
        || module_readback.fixture.appearance_row != appearance.report.appended_row_index
    {
        return Err(error(
            "MATERIAL-E2E-MODULE-SEMANTIC-DIFF",
            "module",
            "module identity, HAK binding, Area, or appearance row differs after readback",
        ));
    }
    Ok(ModelMaterialE2eArtifactV1 {
        report: ModelMaterialE2eReportV1 {
            schema_version: MODEL_MATERIAL_E2E_SCHEMA_VERSION_V1,
            source_sha256: ingest.ir.source.sha256,
            material_profile: "NWN_EE_MTR".to_owned(),
            material_count: compiled.materials.len(),
            model_resref: request.model_resref.clone(),
            model_sha256: model_artifact.binary.report.payload_sha256,
            appearance_row: appearance.report.appended_row_index,
            hak_resref: request.hak_resref.clone(),
            hak_sha256: hak.report.archive_sha256.clone(),
            module_resref: request.module_resref.clone(),
            module_sha256: module.report.sha256.clone(),
            area_resref: request.area_resref.clone(),
            resources: resource_report,
            semantic_readback_status: "PASS".to_owned(),
            ready_for_owner_proof: false,
            owner_proof_blocker: "ARTIFACTS_NOT_FROZEN_OR_INSTALLED".to_owned(),
        },
        hak,
        module,
    })
}

fn material_resources_v1(
    source_glb: &[u8],
    ingest: &crate::glb::GlbIngestResult,
    slot: u32,
    material: &crate::aurora_material::AuroraMaterialIrV1,
) -> Result<MaterialResources, ModelMaterialE2eErrorV1> {
    let diffuse_resref = resource_resref("m2ad", slot)?;
    let normal_resref = material
        .normal_texture
        .as_ref()
        .map(|_| resource_resref("m2an", slot))
        .transpose()?;
    let specular_resref = material
        .specular_texture_plan
        .as_ref()
        .map(|_| resource_resref("m2as", slot))
        .transpose()?;
    let material_resref = resource_resref("m2am", slot)?;
    let source = ingest
        .ir
        .materials
        .iter()
        .find(|value| value.id == material.source_material_id)
        .ok_or_else(|| {
            error(
                "MATERIAL-E2E-SOURCE-MATERIAL-MISSING",
                "materials",
                "compiled material source is missing",
            )
        })?;
    let base = if let Some(binding) = &source.base_color_texture {
        decode_texture(source_glb, ingest, binding.texture_id)?
    } else {
        TgaImageV1 {
            schema_version: 1,
            width: 1,
            height: 1,
            pixel_format: TgaPixelFormatV1::Rgba8,
            pixels: source
                .base_color_factor
                .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
                .to_vec(),
        }
    };
    let diffuse = write_tga_v1(&base, &TgaWriterOptionsV1::default())
        .map_err(|source| error(&source.code, source.path, source.message))?;
    let mut resources = vec![HakResourceInputV1 {
        resref: diffuse_resref.clone(),
        resource_type: TGA_RESOURCE_TYPE_V1,
        payload: diffuse.payload,
    }];
    resources.push(txi_resource(
        &diffuse_resref,
        TxiDocumentV1 {
            schema_version: 1,
            mipmap: Some(true),
            filter: Some(true),
            gamma: Some(2.2),
            is_bump_map: None,
            clamp: Some(false),
            alpha_mean: None,
            is_diffuse_bump_map: None,
            is_specular_bump_map: None,
            bump_map_scaling: None,
            specular_color: None,
            blending: None,
        },
    )?);
    if let (Some(binding), Some(resref)) = (&source.normal_texture, &normal_resref) {
        let image = decode_texture(source_glb, ingest, binding.texture_id)?;
        let tga = write_tga_v1(&image, &TgaWriterOptionsV1::default())
            .map_err(|source| error(&source.code, source.path, source.message))?;
        resources.push(HakResourceInputV1 {
            resref: resref.clone(),
            resource_type: TGA_RESOURCE_TYPE_V1,
            payload: tga.payload,
        });
        resources.push(txi_resource(resref, TxiDocumentV1::normal_map_v1())?);
    }
    if let (Some(plan), Some(resref)) = (&material.specular_texture_plan, &specular_resref) {
        let metallic_roughness = source
            .metallic_roughness_texture
            .as_ref()
            .map(|binding| decode_texture(source_glb, ingest, binding.texture_id))
            .transpose()?;
        let bake = bake_specular_gloss_v1(plan, &base, metallic_roughness.as_ref())
            .map_err(|source| error(&source.code, source.path, source.message))?;
        resources.push(HakResourceInputV1 {
            resref: resref.clone(),
            resource_type: TGA_RESOURCE_TYPE_V1,
            payload: bake.tga.payload,
        });
        resources.push(txi_resource(
            resref,
            TxiDocumentV1 {
                schema_version: 1,
                mipmap: Some(true),
                filter: Some(true),
                gamma: None,
                is_bump_map: None,
                clamp: Some(false),
                alpha_mean: None,
                is_diffuse_bump_map: None,
                is_specular_bump_map: Some(true),
                bump_map_scaling: None,
                specular_color: Some(material.specular_color),
                blending: None,
            },
        )?);
    }
    let mtr_document = compile_mtr_document_v1(
        material,
        &MtrMaterialResourceNamesV1 {
            diffuse: Some(diffuse_resref.clone()),
            normal: normal_resref.clone(),
            specular: specular_resref.clone(),
        },
    )
    .map_err(|source| error(&source.code, "mtr", source.message))?;
    let mtr =
        write_mtr_v1(&mtr_document).map_err(|source| error(&source.code, "mtr", source.message))?;
    if parse_mtr_v1(&mtr).map_err(|source| error(&source.code, "mtr", source.message))?
        != mtr_document
    {
        return Err(error(
            "MATERIAL-E2E-MTR-SEMANTIC-DIFF",
            "mtr",
            "MTR differs after readback",
        ));
    }
    resources.push(HakResourceInputV1 {
        resref: material_resref.clone(),
        resource_type: MTR_RESOURCE_TYPE_V1,
        payload: mtr,
    });
    Ok(MaterialResources {
        slot,
        state: MdlMaterialStateV1 {
            material_slot: slot,
            diffuse: material.diffuse_color[..3]
                .try_into()
                .expect("three components"),
            ambient: material.ambient_color,
            specular: material.specular_color,
            shininess: material.shininess,
            alpha: material.diffuse_color[3],
            self_illum_color: material.self_illum_color,
            transparency_hint: material
                .mtr
                .as_ref()
                .is_some_and(|state| state.transparency),
            render_hint: match material.render_hint {
                AuroraRenderHintV1::Normal => MdlRenderHintV1::Normal,
                AuroraRenderHintV1::NormalAndSpecMapped => MdlRenderHintV1::NormalAndSpecMapped,
            },
            normal_texture_resref: normal_resref,
            specular_texture_resref: specular_resref,
            material_resref: Some(material_resref),
        },
        diffuse_binding: MdlMaterialTextureBindingV1 {
            material_slot: slot,
            resref: diffuse_resref,
        },
        resources,
    })
}

fn decode_texture(
    source_glb: &[u8],
    ingest: &crate::glb::GlbIngestResult,
    texture_id: u32,
) -> Result<TgaImageV1, ModelMaterialE2eErrorV1> {
    let texture = ingest
        .ir
        .textures
        .iter()
        .find(|value| value.id == texture_id)
        .ok_or_else(|| {
            error(
                "MATERIAL-E2E-TEXTURE-MISSING",
                "textures",
                format!("texture {texture_id} is missing"),
            )
        })?;
    let image_index = ingest
        .ir
        .images
        .iter()
        .position(|image| image.id == texture.source_image_id)
        .ok_or_else(|| {
            error(
                "MATERIAL-E2E-IMAGE-MISSING",
                "images",
                format!("image {} is missing", texture.source_image_id),
            )
        })?;
    decode_embedded_image_to_tga_v1(
        source_glb,
        image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|source| error(&source.code, "images", source.message))
}

fn txi_resource(
    resref: &str,
    document: TxiDocumentV1,
) -> Result<HakResourceInputV1, ModelMaterialE2eErrorV1> {
    let payload =
        write_txi_v1(&document).map_err(|source| error(&source.code, "txi", source.message))?;
    if parse_txi_v1(&payload).map_err(|source| error(&source.code, "txi", source.message))?
        != document
    {
        return Err(error(
            "MATERIAL-E2E-TXI-SEMANTIC-DIFF",
            "txi",
            "TXI differs after readback",
        ));
    }
    Ok(HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type: TXI_RESOURCE_TYPE_V1,
        payload,
    })
}

fn validate_resource_readback(
    resource: &HakResourceInputV1,
    readback: &[u8],
) -> Result<(), ModelMaterialE2eErrorV1> {
    match resource.resource_type {
        MDL_RESOURCE_TYPE_V1 => {
            inspect_binary_mdl(readback)
                .map_err(|source| error(&source.code, "hak.model", source.context))?;
        }
        TGA_RESOURCE_TYPE_V1 => {
            read_tga_image_v1(readback)
                .map_err(|source| error(&source.code, "hak.texture", source.message))?;
        }
        MTR_RESOURCE_TYPE_V1 => {
            parse_mtr_v1(readback)
                .map_err(|source| error(&source.code, "hak.mtr", source.message))?;
        }
        TXI_RESOURCE_TYPE_V1 => {
            parse_txi_v1(readback)
                .map_err(|source| error(&source.code, "hak.txi", source.message))?;
        }
        APPEARANCE_2DA_RESOURCE_TYPE_V1 => {}
        _ => {
            return Err(error(
                "MATERIAL-E2E-RESOURCE-TYPE-UNEXPECTED",
                "hak.resources",
                format!("unexpected resource type {}", resource.resource_type),
            ));
        }
    }
    Ok(())
}

fn normalize_root_name(
    model: &mut AuroraModelIrV1,
    model_resref: &str,
) -> Result<(), ModelMaterialE2eErrorV1> {
    let roots = model
        .nodes
        .iter_mut()
        .filter(|node| node.parent_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(error(
            "MATERIAL-E2E-HIERARCHY-INVALID",
            "model.nodes",
            "model must have exactly one root",
        ));
    }
    roots.into_iter().next().expect("one root").name = model_resref.to_owned();
    Ok(())
}

fn validate_request(request: &ModelMaterialE2eRequestV1) -> Result<(), ModelMaterialE2eErrorV1> {
    if request.schema_version != MODEL_MATERIAL_E2E_SCHEMA_VERSION_V1 {
        return Err(error(
            "MATERIAL-E2E-SCHEMA-UNSUPPORTED",
            "schemaVersion",
            "expected schema version 1",
        ));
    }
    for (path, value) in [
        ("modelResref", &request.model_resref),
        ("hakResref", &request.hak_resref),
        ("moduleResref", &request.module_resref),
        ("areaResref", &request.area_resref),
    ] {
        if value.is_empty()
            || value.len() > 16
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(error(
                "MATERIAL-E2E-RESREF-INVALID",
                path,
                "resref must be lowercase ASCII, 1..=16 characters",
            ));
        }
    }
    Ok(())
}

fn resource_resref(prefix: &str, slot: u32) -> Result<String, ModelMaterialE2eErrorV1> {
    let value = format!("{prefix}{slot:x}");
    if value.len() > 16 {
        return Err(error(
            "MATERIAL-E2E-RESREF-EXHAUSTED",
            "materials",
            "material resource resref exceeds 16 characters",
        ));
    }
    Ok(value)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ModelMaterialE2eErrorV1 {
    ModelMaterialE2eErrorV1 {
        schema_version: MODEL_MATERIAL_E2E_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}
