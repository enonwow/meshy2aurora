//! Read-only discovery contracts for local Aurora/NWN supermodel resources.
//!
//! The catalog deliberately separates cheap inventory reads from full binary
//! MDL inspection. KEY/BIF payloads remain in their user-selected containers;
//! callers read only the fixed MDL metadata prefix until a model is previewed.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::erf::{ErfArchive, ErfFileType};

pub const NWN_MDL_RESOURCE_TYPE: u16 = 2002;
pub const BINARY_MDL_CATALOG_PREFIX_BYTES: usize = 12 + 0xe8;

const KEY_HEADER_BYTES: usize = 64;
const KEY_BIF_ENTRY_BYTES: usize = 12;
const KEY_RESOURCE_ENTRY_BYTES: usize = 22;
const BIF_HEADER_BYTES: usize = 20;
const BIF_RESOURCE_ENTRY_BYTES: usize = 16;
const MAX_RESOURCE_COUNT: usize = 1_000_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupermodelCatalogErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub offset: usize,
    pub context: String,
}

impl SupermodelCatalogErrorV1 {
    fn new(code: &str, offset: usize, context: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            code: code.to_owned(),
            offset,
            context: context.into(),
        }
    }
}

impl fmt::Display for SupermodelCatalogErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.offset, self.context
        )
    }
}

impl std::error::Error for SupermodelCatalogErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwnKeyBifV1 {
    pub index: u32,
    pub logical_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwnKeyModelLocatorV1 {
    pub key_index: u32,
    pub resref: String,
    pub bif_index: u32,
    pub resource_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwnKeyModelIndexV1 {
    pub schema_version: u32,
    pub resource_count: usize,
    pub model_resource_count: usize,
    pub bifs: Vec<NwnKeyBifV1>,
    pub models: Vec<NwnKeyModelLocatorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwnBifIndexPlanV1 {
    pub schema_version: u32,
    pub resource_count: usize,
    pub table_offset: usize,
    pub table_byte_length: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwnBifResourceV1 {
    pub resource_index: u32,
    pub resource_id: u32,
    pub payload_offset: u32,
    pub payload_size: u32,
    pub resource_type: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwnBifIndexV1 {
    pub schema_version: u32,
    pub resources: Vec<NwnBifResourceV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HakModelCatalogItemV1 {
    pub resource_index: u32,
    pub resref: String,
    pub payload_offset: usize,
    pub payload_size: usize,
    pub header: Option<MdlCatalogHeaderV1>,
    pub error: Option<SupermodelCatalogErrorV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HakModelIndexV1 {
    pub schema_version: u32,
    pub resource_count: usize,
    pub model_resource_count: usize,
    pub scanned_model_count: usize,
    pub failed_model_count: usize,
    pub models: Vec<HakModelCatalogItemV1>,
}

pub fn index_hak_models_v1(bytes: &[u8]) -> Result<HakModelIndexV1, SupermodelCatalogErrorV1> {
    let archive = ErfArchive::parse(bytes)
        .map_err(|error| SupermodelCatalogErrorV1::new(&error.code, error.offset, error.context))?;
    if archive.file_type() != ErfFileType::Hak {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-HAK-SIGNATURE",
            0,
            "supermodel HAK inventory requires a HAK V1.0 archive",
        ));
    }
    let mut failed_model_count = 0usize;
    let models = archive
        .resources()
        .iter()
        .filter(|resource| resource.resource_type == NWN_MDL_RESOURCE_TYPE)
        .map(|resource| {
            let payload = &bytes[resource.offset..resource.offset + resource.size];
            match inspect_mdl_catalog_header_v1(payload, resource.size) {
                Ok(header) => HakModelCatalogItemV1 {
                    resource_index: resource.resource_id,
                    resref: resource.resref.clone(),
                    payload_offset: resource.offset,
                    payload_size: resource.size,
                    header: Some(header),
                    error: None,
                },
                Err(error) => {
                    failed_model_count = failed_model_count.saturating_add(1);
                    HakModelCatalogItemV1 {
                        resource_index: resource.resource_id,
                        resref: resource.resref.clone(),
                        payload_offset: resource.offset,
                        payload_size: resource.size,
                        header: None,
                        error: Some(error),
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    Ok(HakModelIndexV1 {
        schema_version: 1,
        resource_count: archive.resources().len(),
        model_resource_count: models.len(),
        scanned_model_count: models.len().saturating_sub(failed_model_count),
        failed_model_count,
        models,
    })
}

pub fn index_nwn_key_models_v1(
    bytes: &[u8],
) -> Result<NwnKeyModelIndexV1, SupermodelCatalogErrorV1> {
    require_range(
        bytes,
        0,
        KEY_HEADER_BYTES,
        "M2A-SUPERMODEL-KEY-HEADER",
        "KEY header",
    )?;
    if &bytes[0..4] != b"KEY " || &bytes[4..8] != b"V1  " {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-KEY-SIGNATURE",
            0,
            "expected KEY V1",
        ));
    }
    let bif_count = usize_from_u32(read_u32(bytes, 8)?, 8, "BIF count")?;
    let resource_count = usize_from_u32(read_u32(bytes, 12)?, 12, "resource count")?;
    if bif_count > MAX_RESOURCE_COUNT || resource_count > MAX_RESOURCE_COUNT {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-KEY-LIMIT",
            8,
            "KEY index count exceeds product guardrail",
        ));
    }
    let bif_table_offset = usize_from_u32(read_u32(bytes, 16)?, 16, "BIF table offset")?;
    let resource_table_offset = usize_from_u32(read_u32(bytes, 20)?, 20, "resource table offset")?;
    let bif_table_length = checked_length(bif_count, KEY_BIF_ENTRY_BYTES, 16, "BIF table")?;
    let resource_table_length = checked_length(
        resource_count,
        KEY_RESOURCE_ENTRY_BYTES,
        20,
        "resource table",
    )?;
    require_range(
        bytes,
        bif_table_offset,
        bif_table_length,
        "M2A-SUPERMODEL-KEY-BIF-TABLE",
        "BIF table",
    )?;
    require_range(
        bytes,
        resource_table_offset,
        resource_table_length,
        "M2A-SUPERMODEL-KEY-RESOURCE-TABLE",
        "resource table",
    )?;

    let mut bifs = Vec::with_capacity(bif_count);
    for index in 0..bif_count {
        let entry = bif_table_offset + index * KEY_BIF_ENTRY_BYTES;
        let name_offset =
            usize_from_u32(read_u32(bytes, entry + 4)?, entry + 4, "BIF name offset")?;
        let name_length = usize::from(read_u16(bytes, entry + 8)?);
        let name_bytes = require_range(
            bytes,
            name_offset,
            name_length,
            "M2A-SUPERMODEL-KEY-BIF-NAME",
            "BIF logical name",
        )?;
        let logical_name = parse_path(name_bytes, name_offset)?;
        bifs.push(NwnKeyBifV1 {
            index: index as u32,
            logical_name,
        });
    }

    let mut models = Vec::new();
    for index in 0..resource_count {
        let entry = resource_table_offset + index * KEY_RESOURCE_ENTRY_BYTES;
        let resource_type = read_u16(bytes, entry + 16)?;
        if resource_type != NWN_MDL_RESOURCE_TYPE {
            continue;
        }
        let resource_id = read_u32(bytes, entry + 18)?;
        let bif_index = resource_id >> 20;
        let resource_index = resource_id & 0x000f_ffff;
        if bif_index as usize >= bif_count {
            return Err(SupermodelCatalogErrorV1::new(
                "M2A-SUPERMODEL-KEY-BIF-INDEX",
                entry + 18,
                format!("MDL resource points to missing BIF index {bif_index}"),
            ));
        }
        models.push(NwnKeyModelLocatorV1 {
            key_index: index as u32,
            resref: parse_resref(&bytes[entry..entry + 16], entry)?,
            bif_index,
            resource_index,
        });
    }
    Ok(NwnKeyModelIndexV1 {
        schema_version: 1,
        resource_count,
        model_resource_count: models.len(),
        bifs,
        models,
    })
}

pub fn plan_nwn_bif_index_v1(header: &[u8]) -> Result<NwnBifIndexPlanV1, SupermodelCatalogErrorV1> {
    require_bif_header(header)?;
    let resource_count = usize_from_u32(read_u32(header, 8)?, 8, "BIF resource count")?;
    if resource_count > MAX_RESOURCE_COUNT {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-BIF-LIMIT",
            8,
            "BIF resource count exceeds product guardrail",
        ));
    }
    let table_offset = usize_from_u32(read_u32(header, 16)?, 16, "BIF table offset")?;
    let table_byte_length = checked_length(
        resource_count,
        BIF_RESOURCE_ENTRY_BYTES,
        16,
        "BIF resource table",
    )?;
    Ok(NwnBifIndexPlanV1 {
        schema_version: 1,
        resource_count,
        table_offset,
        table_byte_length,
    })
}

pub fn index_nwn_bif_table_v1(
    header: &[u8],
    table: &[u8],
) -> Result<NwnBifIndexV1, SupermodelCatalogErrorV1> {
    let plan = plan_nwn_bif_index_v1(header)?;
    if table.len() != plan.table_byte_length {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-BIF-TABLE-LENGTH",
            plan.table_offset,
            format!(
                "BIF resource table requires exactly {} bytes, got {}",
                plan.table_byte_length,
                table.len()
            ),
        ));
    }
    let mut resources = Vec::with_capacity(plan.resource_count);
    for index in 0..plan.resource_count {
        let offset = index * BIF_RESOURCE_ENTRY_BYTES;
        resources.push(NwnBifResourceV1 {
            resource_index: index as u32,
            resource_id: read_u32(table, offset)?,
            payload_offset: read_u32(table, offset + 4)?,
            payload_size: read_u32(table, offset + 8)?,
            resource_type: read_u32(table, offset + 12)?,
        });
    }
    Ok(NwnBifIndexV1 {
        schema_version: 1,
        resources,
    })
}

fn require_bif_header(header: &[u8]) -> Result<(), SupermodelCatalogErrorV1> {
    require_range(
        header,
        0,
        BIF_HEADER_BYTES,
        "M2A-SUPERMODEL-BIF-HEADER",
        "BIF header",
    )?;
    if &header[0..4] != b"BIFF" || &header[4..8] != b"V1  " {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-BIF-SIGNATURE",
            0,
            "expected BIFF V1",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MdlCatalogFormatV1 {
    Binary,
    Ascii,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlCatalogHeaderV1 {
    pub schema_version: u32,
    pub format: MdlCatalogFormatV1,
    pub model_name: String,
    pub supermodel_name: String,
    pub classification: Option<u8>,
    pub animation_scale: f32,
    pub local_animation_count: u32,
}

pub fn inspect_mdl_catalog_header_v1(
    bytes: &[u8],
    declared_payload_size: usize,
) -> Result<MdlCatalogHeaderV1, SupermodelCatalogErrorV1> {
    if bytes.len() >= 4 && bytes[..4] == [0, 0, 0, 0] {
        return inspect_binary_catalog_header(bytes, declared_payload_size);
    }
    if bytes.len() != declared_payload_size {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-ASCII-FULL-PAYLOAD-REQUIRED",
            bytes.len(),
            "ASCII MDL catalog inspection requires the complete payload",
        ));
    }
    inspect_ascii_catalog_header(bytes)
}

fn inspect_binary_catalog_header(
    bytes: &[u8],
    declared_payload_size: usize,
) -> Result<MdlCatalogHeaderV1, SupermodelCatalogErrorV1> {
    require_range(
        bytes,
        0,
        BINARY_MDL_CATALOG_PREFIX_BYTES,
        "M2A-SUPERMODEL-MDL-HEADER",
        "binary MDL catalog prefix",
    )?;
    let core_size = usize_from_u32(read_u32(bytes, 4)?, 4, "binary MDL core size")?;
    let raw_size = usize_from_u32(read_u32(bytes, 8)?, 8, "binary MDL raw size")?;
    if core_size < 0xe8 {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-MDL-CORE-SIZE",
            4,
            "binary MDL core is smaller than the model header",
        ));
    }
    let exact_size = 12usize
        .checked_add(core_size)
        .and_then(|value| value.checked_add(raw_size))
        .ok_or_else(|| {
            SupermodelCatalogErrorV1::new(
                "M2A-SUPERMODEL-MDL-SIZE",
                4,
                "binary MDL declared size overflows",
            )
        })?;
    if exact_size != declared_payload_size {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-MDL-SIZE",
            4,
            format!(
                "binary MDL header declares {exact_size} bytes, resource contains {declared_payload_size}"
            ),
        ));
    }
    let animation_scale = read_f32(bytes, 12 + 0xa4)?;
    if !animation_scale.is_finite() {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-MDL-ANIMATION-SCALE",
            12 + 0xa4,
            "animation scale must be finite",
        ));
    }
    Ok(MdlCatalogHeaderV1 {
        schema_version: 1,
        format: MdlCatalogFormatV1::Binary,
        model_name: parse_fixed_string(&bytes[12 + 0x08..12 + 0x48], 12 + 0x08)?,
        supermodel_name: parse_fixed_string(&bytes[12 + 0xa8..12 + 0xe8], 12 + 0xa8)?,
        classification: Some(bytes[12 + 0x72]),
        animation_scale,
        local_animation_count: read_u32(bytes, 12 + 0x78 + 4)?,
    })
}

fn inspect_ascii_catalog_header(
    bytes: &[u8],
) -> Result<MdlCatalogHeaderV1, SupermodelCatalogErrorV1> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-ASCII-UTF8",
            0,
            "ASCII MDL is not valid UTF-8/ASCII text",
        )
    })?;
    let mut model_name = None;
    let mut supermodel_name = None;
    let mut animation_scale = 1.0f32;
    let mut local_animation_count = 0u32;
    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        let Some(keyword) = fields.first() else {
            continue;
        };
        if keyword.eq_ignore_ascii_case("newmodel") && fields.len() >= 2 {
            model_name.get_or_insert_with(|| fields[1].to_owned());
        } else if keyword.eq_ignore_ascii_case("setsupermodel") && fields.len() >= 3 {
            supermodel_name = Some(fields[2].to_owned());
        } else if keyword.eq_ignore_ascii_case("setanimationscale") && fields.len() >= 2 {
            animation_scale = fields[1].parse::<f32>().map_err(|_| {
                SupermodelCatalogErrorV1::new(
                    "M2A-SUPERMODEL-ASCII-ANIMATION-SCALE",
                    0,
                    "ASCII MDL animation scale is invalid",
                )
            })?;
        } else if keyword.eq_ignore_ascii_case("newanim") {
            local_animation_count = local_animation_count.saturating_add(1);
        }
    }
    let model_name = model_name.ok_or_else(|| {
        SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-ASCII-MODEL-NAME",
            0,
            "ASCII MDL has no newmodel declaration",
        )
    })?;
    let supermodel_name = supermodel_name.unwrap_or_else(|| "NULL".to_owned());
    if !animation_scale.is_finite() {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-ASCII-ANIMATION-SCALE",
            0,
            "ASCII MDL animation scale must be finite",
        ));
    }
    validate_catalog_resref(&model_name, "ASCII model name")?;
    validate_catalog_resref_or_null(&supermodel_name, "ASCII supermodel name")?;
    Ok(MdlCatalogHeaderV1 {
        schema_version: 1,
        format: MdlCatalogFormatV1::Ascii,
        model_name,
        supermodel_name,
        classification: None,
        animation_scale,
        local_animation_count,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SupermodelSourceKindV1 {
    BaseKeyBif,
    Hak,
    Override,
    LooseMdl,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupermodelModelV1 {
    pub resref: String,
    pub source_id: String,
    pub source_kind: SupermodelSourceKindV1,
    pub container_name: String,
    pub source_priority: u32,
    pub resource_index: u32,
    pub payload_offset: u32,
    pub payload_size: u32,
    pub header: MdlCatalogHeaderV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupermodelCatalogBuildInputV1 {
    pub declared_model_count: usize,
    pub failed_model_count: usize,
    pub models: Vec<SupermodelModelV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SupermodelCatalogCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SupermodelCatalogEntryStatusV1 {
    Resolved,
    Missing,
    Cyclic,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupermodelCatalogEntryV1 {
    pub resref: String,
    pub status: SupermodelCatalogEntryStatusV1,
    pub resource: Option<SupermodelModelV1>,
    pub definitions: Vec<SupermodelModelV1>,
    pub children: Vec<String>,
    pub chain: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupermodelCatalogV1 {
    pub schema_version: u32,
    pub completeness: SupermodelCatalogCompletenessV1,
    pub declared_model_count: usize,
    pub scanned_model_count: usize,
    pub failed_model_count: usize,
    pub supermodel_count: usize,
    pub entries: Vec<SupermodelCatalogEntryV1>,
}

pub fn build_supermodel_catalog_v1(input: &SupermodelCatalogBuildInputV1) -> SupermodelCatalogV1 {
    let mut definitions: HashMap<String, Vec<(usize, &SupermodelModelV1)>> = HashMap::new();
    for (index, model) in input.models.iter().enumerate() {
        definitions
            .entry(normalize_name(&model.resref))
            .or_default()
            .push((index, model));
    }
    for values in definitions.values_mut() {
        values.sort_by(|left, right| {
            left.1
                .source_priority
                .cmp(&right.1.source_priority)
                .then_with(|| left.0.cmp(&right.0))
        });
    }
    let effective = definitions
        .iter()
        .filter_map(|(name, values)| values.first().map(|(_, model)| (name.clone(), *model)))
        .collect::<HashMap<_, _>>();

    let mut targets = BTreeMap::<String, String>::new();
    let mut children = HashMap::<String, BTreeSet<String>>::new();
    for model in effective.values() {
        let parent = model.header.supermodel_name.trim();
        if is_null_supermodel(parent) {
            continue;
        }
        let normalized_parent = normalize_name(parent);
        targets
            .entry(normalized_parent.clone())
            .or_insert_with(|| parent.to_owned());
        children
            .entry(normalized_parent)
            .or_default()
            .insert(model.resref.clone());
    }

    let mut entries = Vec::with_capacity(targets.len());
    for (normalized_target, referenced_spelling) in targets {
        let resource = effective.get(&normalized_target).copied();
        let resref = resource
            .map(|model| model.resref.clone())
            .unwrap_or(referenced_spelling);
        let (status, chain) = resolve_chain(&normalized_target, &effective);
        let entry_definitions = definitions
            .get(&normalized_target)
            .map(|values| values.iter().map(|(_, model)| (*model).clone()).collect())
            .unwrap_or_default();
        entries.push(SupermodelCatalogEntryV1 {
            resref,
            status,
            resource: resource.cloned(),
            definitions: entry_definitions,
            children: children
                .remove(&normalized_target)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            chain,
        });
    }
    entries.sort_by(|left, right| {
        left.resref
            .to_ascii_lowercase()
            .cmp(&right.resref.to_ascii_lowercase())
            .then_with(|| left.resref.cmp(&right.resref))
    });
    let complete =
        input.failed_model_count == 0 && input.models.len() == input.declared_model_count;
    SupermodelCatalogV1 {
        schema_version: 1,
        completeness: if complete {
            SupermodelCatalogCompletenessV1::Complete
        } else {
            SupermodelCatalogCompletenessV1::Partial
        },
        declared_model_count: input.declared_model_count,
        scanned_model_count: input.models.len(),
        failed_model_count: input.failed_model_count,
        supermodel_count: entries.len(),
        entries,
    }
}

fn resolve_chain(
    start: &str,
    effective: &HashMap<String, &SupermodelModelV1>,
) -> (SupermodelCatalogEntryStatusV1, Vec<String>) {
    let mut current = start.to_owned();
    let mut visited = HashSet::new();
    let mut chain = Vec::new();
    loop {
        if !visited.insert(current.clone()) {
            if let Some(model) = effective.get(&current) {
                chain.push(model.resref.clone());
            }
            return (SupermodelCatalogEntryStatusV1::Cyclic, chain);
        }
        let Some(model) = effective.get(&current) else {
            if chain.is_empty() {
                chain.push(current);
            }
            return (SupermodelCatalogEntryStatusV1::Missing, chain);
        };
        chain.push(model.resref.clone());
        let parent = model.header.supermodel_name.trim();
        if is_null_supermodel(parent) {
            return (SupermodelCatalogEntryStatusV1::Resolved, chain);
        }
        current = normalize_name(parent);
    }
}

fn normalize_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn is_null_supermodel(value: &str) -> bool {
    value.is_empty() || value.eq_ignore_ascii_case("NULL")
}

fn validate_catalog_resref(value: &str, context: &str) -> Result<(), SupermodelCatalogErrorV1> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-MDL-RESREF",
            0,
            format!("{context} is not a valid Aurora resource name"),
        ));
    }
    Ok(())
}

fn validate_catalog_resref_or_null(
    value: &str,
    context: &str,
) -> Result<(), SupermodelCatalogErrorV1> {
    if is_null_supermodel(value) {
        Ok(())
    } else {
        validate_catalog_resref(value, context)
    }
}

fn parse_resref(bytes: &[u8], offset: usize) -> Result<String, SupermodelCatalogErrorV1> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if bytes[end..].iter().any(|byte| *byte != 0) {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-KEY-RESREF-PADDING",
            offset + end,
            "KEY resref has non-NUL bytes after its terminator",
        ));
    }
    let value = parse_fixed_string(bytes, offset)?;
    if value.len() > 16 {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-KEY-RESREF",
            offset,
            "KEY resref exceeds 16 bytes",
        ));
    }
    validate_catalog_resref(&value, "KEY resref")?;
    Ok(value)
}

fn parse_fixed_string(bytes: &[u8], offset: usize) -> Result<String, SupermodelCatalogErrorV1> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if end == 0 {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-STRING-EMPTY",
            offset,
            "fixed string must not be empty",
        ));
    }
    if !bytes[..end].iter().all(u8::is_ascii) {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-STRING-ASCII",
            offset,
            "fixed string must be ASCII",
        ));
    }
    Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
}

fn parse_path(bytes: &[u8], offset: usize) -> Result<String, SupermodelCatalogErrorV1> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if end == 0 || !bytes[..end].iter().all(u8::is_ascii) {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-KEY-BIF-NAME",
            offset,
            "BIF logical name must be non-empty ASCII",
        ));
    }
    let path = String::from_utf8_lossy(&bytes[..end]).replace('\\', "/");
    if path.starts_with('/')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-KEY-BIF-PATH",
            offset,
            "BIF logical name must be a safe relative path",
        ));
    }
    Ok(path)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, SupermodelCatalogErrorV1> {
    let value = require_range(bytes, offset, 2, "M2A-SUPERMODEL-BINARY-OOB", "u16 field")?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, SupermodelCatalogErrorV1> {
    let value = require_range(bytes, offset, 4, "M2A-SUPERMODEL-BINARY-OOB", "u32 field")?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32, SupermodelCatalogErrorV1> {
    let value = read_u32(bytes, offset)?;
    Ok(f32::from_bits(value))
}

fn usize_from_u32(
    value: u32,
    offset: usize,
    context: &str,
) -> Result<usize, SupermodelCatalogErrorV1> {
    usize::try_from(value).map_err(|_| {
        SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-INTEGER-RANGE",
            offset,
            format!("{context} does not fit this platform"),
        )
    })
}

fn checked_length(
    count: usize,
    stride: usize,
    offset: usize,
    context: &str,
) -> Result<usize, SupermodelCatalogErrorV1> {
    count.checked_mul(stride).ok_or_else(|| {
        SupermodelCatalogErrorV1::new(
            "M2A-SUPERMODEL-BINARY-RANGE",
            offset,
            format!("{context} length overflows"),
        )
    })
}

fn require_range<'a>(
    bytes: &'a [u8],
    offset: usize,
    length: usize,
    code: &str,
    context: &str,
) -> Result<&'a [u8], SupermodelCatalogErrorV1> {
    let end = offset.checked_add(length).ok_or_else(|| {
        SupermodelCatalogErrorV1::new(code, offset, format!("{context} range overflows"))
    })?;
    bytes.get(offset..end).ok_or_else(|| {
        SupermodelCatalogErrorV1::new(
            code,
            offset,
            format!(
                "{context} requires bytes {offset}..{end}, input has {}",
                bytes.len()
            ),
        )
    })
}
