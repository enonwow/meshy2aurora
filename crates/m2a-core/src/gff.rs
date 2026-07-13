use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(test)]
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[cfg(test)]
static FORCE_NEXT_TRACKED_ALLOCATION_FAILURE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static ALLOCATION_TEST_LOCK: Mutex<()> = Mutex::new(());
#[cfg(test)]
static TRACKED_INPUT_MATERIALIZED_BYTES: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static FORCE_NEXT_LOC_UNIQUENESS_ALLOCATION_FAILURE: AtomicBool = AtomicBool::new(false);

pub const GFF_SCHEMA_VERSION: u32 = 1;
pub const GFF_MAX_BYTES: u64 = 67_108_864;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GffFileTypeV1 {
    #[serde(rename = "UTC ")]
    Utc,
    #[serde(rename = "IFO ")]
    Ifo,
    #[serde(rename = "ARE ")]
    Are,
    #[serde(rename = "GIT ")]
    Git,
    #[serde(rename = "GIC ")]
    Gic,
}

impl GffFileTypeV1 {
    const fn bytes(self) -> [u8; 4] {
        match self {
            Self::Utc => *b"UTC ",
            Self::Ifo => *b"IFO ",
            Self::Are => *b"ARE ",
            Self::Git => *b"GIT ",
            Self::Gic => *b"GIC ",
        }
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"UTC " => Some(Self::Utc),
            b"IFO " => Some(Self::Ifo),
            b"ARE " => Some(Self::Are),
            b"GIT " => Some(Self::Git),
            b"GIC " => Some(Self::Gic),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffDocumentV1 {
    pub schema_version: u32,
    pub file_type: GffFileTypeV1,
    pub root: GffStructV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffStructV1 {
    pub struct_id: u32,
    pub fields: Vec<GffFieldV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffFieldV1 {
    pub label: String,
    pub value: GffValueV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffLocStringV1 {
    pub string_ref: u32,
    pub substrings: Vec<GffLocSubstringV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffLocSubstringV1 {
    pub string_id: i32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum GffValueV1 {
    Byte(u8),
    Char(i8),
    Word(u16),
    Short(i16),
    Dword(u32),
    Int(i32),
    Dword64(u64),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(Vec<u8>),
    ResRef(String),
    LocString(GffLocStringV1),
    Void(Vec<u8>),
    Struct(GffStructV1),
    List(Vec<GffStructV1>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffLimitsV1 {
    pub max_gff_bytes: u64,
    pub max_structs: u32,
    pub max_fields: u32,
    pub max_labels: u32,
    pub max_fields_per_struct: u32,
    pub max_list_elements: u32,
    pub max_depth: u32,
    pub max_string_bytes: u32,
    pub max_loc_string_bytes: u32,
    pub max_void_bytes: u32,
    pub max_diagnostics: u32,
}

impl Default for GffLimitsV1 {
    fn default() -> Self {
        Self {
            max_gff_bytes: GFF_MAX_BYTES,
            max_structs: 65_536,
            max_fields: 262_144,
            max_labels: 65_536,
            max_fields_per_struct: 65_536,
            max_list_elements: 65_536,
            max_depth: 64,
            max_string_bytes: 1_024,
            max_loc_string_bytes: 1_048_576,
            max_void_bytes: 16_777_216,
            max_diagnostics: 2_048,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GffWriterOptionsV1 {
    pub schema_version: u32,
    pub limits: GffLimitsV1,
}
impl Default for GffWriterOptionsV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            limits: GffLimitsV1::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GffWriterReportV1 {
    pub schema_version: u32,
    pub file_type: GffFileTypeV1,
    pub struct_offset: u32,
    pub struct_count: u32,
    pub field_offset: u32,
    pub field_count: u32,
    pub label_offset: u32,
    pub label_count: u32,
    pub field_data_offset: u32,
    pub field_data_count: u32,
    pub field_indices_offset: u32,
    pub field_indices_count: u32,
    pub list_indices_offset: u32,
    pub list_indices_count: u32,
    pub byte_length: u64,
    pub output_sha256: String,
    pub max_depth: u32,
    pub semantic_readback_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GffArtifactV1 {
    pub payload: Vec<u8>,
    pub report: GffWriterReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GffErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}
impl GffErrorV1 {
    fn fatal(code: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            path: path.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for GffErrorV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.path, self.message)
    }
}
impl std::error::Error for GffErrorV1 {}

struct Preflight<'a> {
    labels: HashSet<&'a str>,
    structs: u32,
    fields: u32,
    field_data_bytes: u32,
    field_indices_bytes: u32,
    list_indices_bytes: u32,
    max_depth: u32,
}

#[derive(Clone, Copy)]
struct OutputLayout {
    struct_offset: u32,
    struct_count: u32,
    field_offset: u32,
    field_count: u32,
    label_offset: u32,
    label_count: u32,
    field_data_offset: u32,
    field_data_count: u32,
    field_indices_offset: u32,
    field_indices_count: u32,
    list_indices_offset: u32,
    list_indices_count: u32,
    total: u32,
}

pub fn write_gff_v32(
    document: &GffDocumentV1,
    options: &GffWriterOptionsV1,
) -> Result<GffArtifactV1, GffErrorV1> {
    validate_limits(&options.limits)?;
    if options.schema_version != 1 {
        return Err(err(
            "M6-GFF-OPTIONS-INVALID",
            "options.schemaVersion",
            "schemaVersion must be 1",
        ));
    }
    if document.schema_version != 1 {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            "document.schemaVersion",
            "schemaVersion must be 1",
        ));
    }
    if document.root.struct_id != u32::MAX {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            "document.root.structId",
            "root struct ID must be 0xffffffff",
        ));
    }
    let mut preflight = Preflight {
        labels: HashSet::new(),
        structs: 0,
        fields: 0,
        field_data_bytes: 0,
        field_indices_bytes: 0,
        list_indices_bytes: 0,
        max_depth: 0,
    };
    scan_struct(
        &document.root,
        0,
        "document.root",
        &options.limits,
        &mut preflight,
    )?;
    let layout = plan_output_layout(&preflight, &options.limits)?;
    emit_document(document, options, &preflight, layout)
}

fn scan_struct<'a>(
    input: &'a GffStructV1,
    depth: u32,
    path: &str,
    limits: &GffLimitsV1,
    preflight: &mut Preflight<'a>,
) -> Result<(), GffErrorV1> {
    if depth > limits.max_depth {
        return Err(err(
            "M6-GFF-DEPTH-LIMIT-EXCEEDED",
            path,
            "tree depth exceeds maxDepth",
        ));
    }
    preflight.max_depth = preflight.max_depth.max(depth);
    preflight.structs = preflight
        .structs
        .checked_add(1)
        .ok_or_else(|| err("M6-GFF-LIMIT-EXCEEDED", path, "struct count overflow"))?;
    if preflight.structs > limits.max_structs {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            "struct count exceeds maxStructs",
        ));
    }
    if input.fields.len() > limits.max_fields_per_struct as usize {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            "field count exceeds maxFieldsPerStruct",
        ));
    }
    let add = u32::try_from(input.fields.len()).map_err(|_| {
        err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            "field count does not fit u32",
        )
    })?;
    preflight.fields = preflight
        .fields
        .checked_add(add)
        .ok_or_else(|| err("M6-GFF-LIMIT-EXCEEDED", path, "field count overflow"))?;
    if preflight.fields > limits.max_fields {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            "field count exceeds maxFields",
        ));
    }
    if input.fields.len() > 1 {
        preflight.field_indices_bytes = add_u32(
            preflight.field_indices_bytes,
            mul_u32(add, 4, "fieldIndices")?,
            "fieldIndices",
        )?;
    }
    let mut labels = HashSet::new();
    labels
        .try_reserve(input.fields.len())
        .map_err(|_| alloc(path))?;
    for (field_index, field) in input.fields.iter().enumerate() {
        let field_path = format!("{path}.fields[{field_index}]");
        validate_label(&field.label, &field_path)?;
        if !labels.insert(field.label.as_str()) {
            return Err(err(
                "M6-GFF-DUPLICATE-LABEL",
                &field_path,
                "labels in one struct must be unique",
            ));
        }
        if !preflight.labels.contains(field.label.as_str()) {
            preflight
                .labels
                .try_reserve(1)
                .map_err(|_| alloc(&field_path))?;
            preflight.labels.insert(field.label.as_str());
            if preflight.labels.len() > limits.max_labels as usize {
                return Err(err(
                    "M6-GFF-LIMIT-EXCEEDED",
                    &field_path,
                    "label count exceeds maxLabels",
                ));
            }
        }
        match &field.value {
            GffValueV1::Byte(_)
            | GffValueV1::Char(_)
            | GffValueV1::Word(_)
            | GffValueV1::Short(_)
            | GffValueV1::Dword(_)
            | GffValueV1::Int(_) => {}
            GffValueV1::Dword64(_) | GffValueV1::Int64(_) => {
                preflight.field_data_bytes = add_u32(preflight.field_data_bytes, 8, "fieldData")?;
            }
            GffValueV1::Float(v) if v.is_finite() => {}
            GffValueV1::Double(v) if v.is_finite() => {
                preflight.field_data_bytes = add_u32(preflight.field_data_bytes, 8, "fieldData")?;
            }
            GffValueV1::Float(_) | GffValueV1::Double(_) => {
                return Err(err(
                    "M6-GFF-VALUE-INVALID",
                    &field_path,
                    "floating-point values must be finite",
                ));
            }
            GffValueV1::String(v) => {
                check_len(
                    v.len(),
                    limits.max_string_bytes,
                    &field_path,
                    "maxStringBytes",
                )?;
                preflight.field_data_bytes = add_u32(
                    preflight.field_data_bytes,
                    add_u32(4, to_u32(v.len(), &field_path)?, "fieldData")?,
                    "fieldData",
                )?;
            }
            GffValueV1::ResRef(v) => {
                validate_resref(v, &field_path)?;
                preflight.field_data_bytes = add_u32(
                    preflight.field_data_bytes,
                    add_u32(1, to_u32(v.len(), &field_path)?, "fieldData")?,
                    "fieldData",
                )?;
            }
            GffValueV1::Void(v) => {
                check_len(v.len(), limits.max_void_bytes, &field_path, "maxVoidBytes")?;
                preflight.field_data_bytes = add_u32(
                    preflight.field_data_bytes,
                    add_u32(4, to_u32(v.len(), &field_path)?, "fieldData")?,
                    "fieldData",
                )?;
            }
            GffValueV1::LocString(v) => {
                validate_loc(v, limits, &field_path)?;
                preflight.field_data_bytes = add_u32(
                    preflight.field_data_bytes,
                    add_u32(4, loc_encoded_len(v)?, "fieldData")?,
                    "fieldData",
                )?;
            }
            GffValueV1::Struct(child) => {
                scan_struct(child, depth + 1, &field_path, limits, preflight)?;
            }
            GffValueV1::List(children) => {
                check_len(
                    children.len(),
                    limits.max_list_elements,
                    &field_path,
                    "maxListElements",
                )?;
                preflight.list_indices_bytes = add_u32(
                    preflight.list_indices_bytes,
                    add_u32(
                        4,
                        mul_u32(to_u32(children.len(), &field_path)?, 4, "listIndices")?,
                        "listIndices",
                    )?,
                    "listIndices",
                )?;
                for (i, child) in children.iter().enumerate() {
                    scan_struct(
                        child,
                        depth + 1,
                        &format!("{field_path}.list[{i}]"),
                        limits,
                        preflight,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn plan_output_layout(
    preflight: &Preflight<'_>,
    limits: &GffLimitsV1,
) -> Result<OutputLayout, GffErrorV1> {
    let struct_offset = 56u32;
    let field_offset = add_u32(
        struct_offset,
        mul_u32(preflight.structs, 12, "structArray")?,
        "fieldOffset",
    )?;
    let label_offset = add_u32(
        field_offset,
        mul_u32(preflight.fields, 12, "fieldArray")?,
        "labelOffset",
    )?;
    let label_count = to_u32(preflight.labels.len(), "labelCount")?;
    let field_data_offset = add_u32(
        label_offset,
        mul_u32(label_count, 16, "labelArray")?,
        "fieldDataOffset",
    )?;
    let field_indices_offset = add_u32(
        field_data_offset,
        preflight.field_data_bytes,
        "fieldIndicesOffset",
    )?;
    let list_indices_offset = add_u32(
        field_indices_offset,
        preflight.field_indices_bytes,
        "listIndicesOffset",
    )?;
    let total = add_u32(list_indices_offset, preflight.list_indices_bytes, "EOF")?;
    if u64::from(total) > limits.max_gff_bytes {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            "output",
            "output exceeds maxGffBytes",
        ));
    }
    Ok(OutputLayout {
        struct_offset,
        struct_count: preflight.structs,
        field_offset,
        field_count: preflight.fields,
        label_offset,
        label_count,
        field_data_offset,
        field_data_count: preflight.field_data_bytes,
        field_indices_offset,
        field_indices_count: preflight.field_indices_bytes,
        list_indices_offset,
        list_indices_count: preflight.list_indices_bytes,
        total,
    })
}

fn collect_struct_refs<'a>(
    structure: &'a GffStructV1,
    structs: &mut Vec<&'a GffStructV1>,
    indices: &mut HashMap<usize, u32>,
) -> Result<(), GffErrorV1> {
    let index = to_u32(structs.len(), "structs")?;
    structs.try_reserve(1).map_err(|_| alloc("structs"))?;
    indices.try_reserve(1).map_err(|_| alloc("structs"))?;
    structs.push(structure);
    indices.insert(structure as *const GffStructV1 as usize, index);
    for field in &structure.fields {
        match &field.value {
            GffValueV1::Struct(child) => collect_struct_refs(child, structs, indices)?,
            GffValueV1::List(children) => {
                for child in children {
                    collect_struct_refs(child, structs, indices)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn emit_document<'a>(
    document: &GffDocumentV1,
    options: &GffWriterOptionsV1,
    preflight: &Preflight<'a>,
    output_layout: OutputLayout,
) -> Result<GffArtifactV1, GffErrorV1> {
    fail_tracked_allocation("output.materialization")?;
    let mut planned_structs = Vec::new();
    let mut struct_indices = HashMap::new();
    planned_structs
        .try_reserve_exact(output_layout.struct_count as usize)
        .map_err(|_| alloc("structs"))?;
    struct_indices
        .try_reserve(output_layout.struct_count as usize)
        .map_err(|_| alloc("structs"))?;
    collect_struct_refs(&document.root, &mut planned_structs, &mut struct_indices)?;
    let mut fields: Vec<[u32; 3]> = Vec::new();
    fields
        .try_reserve_exact(output_layout.field_count as usize)
        .map_err(|_| alloc("output.fields"))?;
    let mut labels: Vec<&str> = Vec::new();
    let mut label_map: HashMap<&str, u32> = HashMap::new();
    let mut field_data = Vec::new();
    field_data
        .try_reserve_exact(output_layout.field_data_count as usize)
        .map_err(|_| alloc("fieldData"))?;
    let mut field_indices = Vec::new();
    field_indices
        .try_reserve_exact(output_layout.field_indices_count as usize)
        .map_err(|_| alloc("fieldIndices"))?;
    let mut list_indices = Vec::new();
    list_indices
        .try_reserve_exact(output_layout.list_indices_count as usize)
        .map_err(|_| alloc("listIndices"))?;
    let mut struct_entries = Vec::new();
    struct_entries
        .try_reserve_exact(output_layout.struct_count as usize)
        .map_err(|_| alloc("output.structs"))?;
    for structure in &planned_structs {
        let field_start = to_u32(fields.len(), "fields")?;
        let field_count = to_u32(structure.fields.len(), "struct.fieldCount")?;
        let struct_data = match field_count {
            0 => 0,
            1 => field_start,
            _ => {
                let offset = to_u32(field_indices.len(), "fieldIndices")?;
                for index in field_start..add_u32(field_start, field_count, "fields")? {
                    field_indices.extend_from_slice(&index.to_le_bytes());
                }
                offset
            }
        };
        struct_entries.push([structure.struct_id, struct_data, field_count]);
        for field in &structure.fields {
            let label_index = if let Some(index) = label_map.get(field.label.as_str()) {
                *index
            } else {
                let index = to_u32(labels.len(), "labels")?;
                label_map.try_reserve(1).map_err(|_| alloc("labels"))?;
                labels.try_reserve(1).map_err(|_| alloc("labels"))?;
                label_map.insert(field.label.as_str(), index);
                labels.push(field.label.as_str());
                index
            };
            let (type_id, data) = encode_value(
                &field.value,
                &struct_indices,
                &mut field_data,
                &mut list_indices,
                &options.limits,
            )?;
            fields.push([type_id, label_index, data]);
        }
    }
    if fields.len() != output_layout.field_count as usize
        || labels.len() != output_layout.label_count as usize
        || field_data.len() != output_layout.field_data_count as usize
        || field_indices.len() != output_layout.field_indices_count as usize
        || list_indices.len() != output_layout.list_indices_count as usize
    {
        return Err(layout(
            "output",
            "emission differs from borrowed preflight layout",
        ));
    }
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(output_layout.total as usize)
        .map_err(|_| alloc("output"))?;
    payload.extend_from_slice(&document.file_type.bytes());
    payload.extend_from_slice(b"V3.2");
    for value in [
        output_layout.struct_offset,
        output_layout.struct_count,
        output_layout.field_offset,
        output_layout.field_count,
        output_layout.label_offset,
        output_layout.label_count,
        output_layout.field_data_offset,
        output_layout.field_data_count,
        output_layout.field_indices_offset,
        output_layout.field_indices_count,
        output_layout.list_indices_offset,
        output_layout.list_indices_count,
    ] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    for entry in struct_entries {
        for value in entry {
            payload.extend_from_slice(&value.to_le_bytes());
        }
    }
    for entry in fields {
        for value in entry {
            payload.extend_from_slice(&value.to_le_bytes());
        }
    }
    for label in labels {
        let bytes = label.as_bytes();
        payload.extend_from_slice(bytes);
        payload.resize(payload.len() + 16 - bytes.len(), 0);
    }
    payload.extend_from_slice(&field_data);
    payload.extend_from_slice(&field_indices);
    payload.extend_from_slice(&list_indices);
    if payload.len() != output_layout.total as usize {
        return Err(layout(
            "output",
            "emitted byte length differs from planned EOF",
        ));
    }
    let readback = read_gff_v32(&payload, &options.limits)
        .map_err(|e| err("M6-GFF-READBACK-FAILED", "output", e.to_string()))?;
    if &readback != document {
        return Err(err(
            "M6-GFF-SEMANTIC-DIFF",
            "output",
            "public own-reader tree differs from input",
        ));
    }
    let report = GffWriterReportV1 {
        schema_version: 1,
        file_type: document.file_type,
        struct_offset: output_layout.struct_offset,
        struct_count: output_layout.struct_count,
        field_offset: output_layout.field_offset,
        field_count: output_layout.field_count,
        label_offset: output_layout.label_offset,
        label_count: output_layout.label_count,
        field_data_offset: output_layout.field_data_offset,
        field_data_count: output_layout.field_data_count,
        field_indices_offset: output_layout.field_indices_offset,
        field_indices_count: output_layout.field_indices_count,
        list_indices_offset: output_layout.list_indices_offset,
        list_indices_count: output_layout.list_indices_count,
        byte_length: u64::from(output_layout.total),
        output_sha256: sha256_hex(&payload)?,
        max_depth: preflight.max_depth,
        semantic_readback_status: copy_string("PASS", "report.semanticReadbackStatus")?,
    };
    Ok(GffArtifactV1 { payload, report })
}

fn encode_value(
    value: &GffValueV1,
    struct_indices: &HashMap<usize, u32>,
    data: &mut Vec<u8>,
    list_indices: &mut Vec<u8>,
    limits: &GffLimitsV1,
) -> Result<(u32, u32), GffErrorV1> {
    let inline = |type_id, value| Ok((type_id, value));
    match value {
        GffValueV1::Byte(v) => inline(0, u32::from(*v)),
        GffValueV1::Char(v) => inline(1, u32::from(*v as u8)),
        GffValueV1::Word(v) => inline(2, u32::from(*v)),
        GffValueV1::Short(v) => inline(3, u32::from(*v as u16)),
        GffValueV1::Dword(v) => inline(4, *v),
        GffValueV1::Int(v) => inline(5, *v as u32),
        GffValueV1::Float(v) => inline(8, v.to_bits()),
        GffValueV1::Struct(v) => inline(
            14,
            *struct_indices
                .get(&(v as *const GffStructV1 as usize))
                .ok_or_else(|| layout("structs", "missing planned struct index"))?,
        ),
        GffValueV1::Dword64(v) => {
            let o = append_data(data, &v.to_le_bytes(), limits)?;
            Ok((6, o))
        }
        GffValueV1::Int64(v) => {
            let o = append_data(data, &v.to_le_bytes(), limits)?;
            Ok((7, o))
        }
        GffValueV1::Double(v) => {
            let o = append_data(data, &v.to_bits().to_le_bytes(), limits)?;
            Ok((9, o))
        }
        GffValueV1::String(v) => {
            let o = to_u32(data.len(), "fieldData")?;
            append_data(
                data,
                &to_u32(v.len(), "string.length")?.to_le_bytes(),
                limits,
            )?;
            append_data(data, v, limits)?;
            Ok((10, o))
        }
        GffValueV1::ResRef(v) => {
            let o = to_u32(data.len(), "fieldData")?;
            append_data(data, &[v.len() as u8], limits)?;
            append_data(data, v.as_bytes(), limits)?;
            Ok((11, o))
        }
        GffValueV1::LocString(v) => {
            let o = to_u32(data.len(), "fieldData")?;
            let total = loc_encoded_len(v)?;
            append_data(data, &total.to_le_bytes(), limits)?;
            append_data(data, &v.string_ref.to_le_bytes(), limits)?;
            append_data(
                data,
                &to_u32(v.substrings.len(), "locString.count")?.to_le_bytes(),
                limits,
            )?;
            for s in &v.substrings {
                append_data(data, &s.string_id.to_le_bytes(), limits)?;
                append_data(data, &(s.bytes.len() as i32).to_le_bytes(), limits)?;
                append_data(data, &s.bytes, limits)?;
            }
            Ok((12, o))
        }
        GffValueV1::Void(v) => {
            let o = to_u32(data.len(), "fieldData")?;
            append_data(data, &to_u32(v.len(), "void.length")?.to_le_bytes(), limits)?;
            append_data(data, v, limits)?;
            Ok((13, o))
        }
        GffValueV1::List(v) => {
            let offset = to_u32(list_indices.len(), "listIndices")?;
            list_indices.extend_from_slice(&to_u32(v.len(), "list.count")?.to_le_bytes());
            for child in v {
                let index = struct_indices
                    .get(&(child as *const GffStructV1 as usize))
                    .ok_or_else(|| layout("structs", "missing planned list struct index"))?;
                list_indices.extend_from_slice(&index.to_le_bytes());
            }
            Ok((15, offset))
        }
    }
}

#[derive(Clone, Copy)]
struct Header {
    file_type: GffFileTypeV1,
    so: u32,
    sc: u32,
    fo: u32,
    fc: u32,
    lo: u32,
    lc: u32,
    doff: u32,
    dc: u32,
    fio: u32,
    fic: u32,
    lio: u32,
    lic: u32,
}
#[derive(Clone, Copy)]
struct PStruct {
    id: u32,
    data: u32,
    count: u32,
}
#[derive(Clone, Copy)]
struct PField {
    ty: u32,
    label: u32,
    data: u32,
}

pub fn read_gff_v32(bytes: &[u8], limits: &GffLimitsV1) -> Result<GffDocumentV1, GffErrorV1> {
    validate_limits(limits)?;
    if bytes.len() as u64 > limits.max_gff_bytes {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            "input",
            "input exceeds maxGffBytes",
        ));
    }
    if bytes.len() < 56 {
        return Err(err(
            "M6-GFF-HEADER-INVALID",
            "input.header",
            "GFF header requires 56 bytes",
        ));
    }
    if &bytes[4..8] != b"V3.2" {
        return Err(err(
            "M6-GFF-VERSION-UNSUPPORTED",
            "input.header.fileVersion",
            "expected V3.2",
        ));
    }
    let file_type = GffFileTypeV1::from_bytes(&bytes[..4]).ok_or_else(|| {
        err(
            "M6-GFF-FILE-TYPE-UNSUPPORTED",
            "input.header.fileType",
            "unsupported GFF file type",
        )
    })?;
    let h = Header {
        file_type,
        so: u32at(bytes, 8)?,
        sc: u32at(bytes, 12)?,
        fo: u32at(bytes, 16)?,
        fc: u32at(bytes, 20)?,
        lo: u32at(bytes, 24)?,
        lc: u32at(bytes, 28)?,
        doff: u32at(bytes, 32)?,
        dc: u32at(bytes, 36)?,
        fio: u32at(bytes, 40)?,
        fic: u32at(bytes, 44)?,
        lio: u32at(bytes, 48)?,
        lic: u32at(bytes, 52)?,
    };
    validate_layout(bytes.len(), &h)?;
    if h.sc == 0
        || h.sc > limits.max_structs
        || h.fc > limits.max_fields
        || h.lc > limits.max_labels
    {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            "input.header.counts",
            "GFF count exceeds configured limits",
        ));
    }
    let mut structs = Vec::new();
    structs
        .try_reserve_exact(h.sc as usize)
        .map_err(|_| alloc("input.structs"))?;
    for i in 0..h.sc {
        let o = h.so as usize + i as usize * 12;
        structs.push(PStruct {
            id: u32at(bytes, o)?,
            data: u32at(bytes, o + 4)?,
            count: u32at(bytes, o + 8)?,
        });
    }
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(h.fc as usize)
        .map_err(|_| alloc("input.fields"))?;
    for i in 0..h.fc {
        let o = h.fo as usize + i as usize * 12;
        let ty = u32at(bytes, o)?;
        if ty > 15 {
            return Err(err(
                "M6-GFF-TYPE-UNSUPPORTED",
                format!("input.fields[{i}].type"),
                "field type ID exceeds 15",
            ));
        }
        fields.push(PField {
            ty,
            label: u32at(bytes, o + 4)?,
            data: u32at(bytes, o + 8)?,
        });
    }
    let mut label_slices = Vec::new();
    label_slices
        .try_reserve_exact(h.lc as usize)
        .map_err(|_| alloc("input.labels"))?;
    for i in 0..h.lc {
        let o = h.lo as usize + i as usize * 16;
        label_slices.push(&bytes[o..o + 16]);
    }
    validate_raw_references(bytes, &h, &structs, &fields, label_slices.len())?;
    validate_raw_payload_bounds(bytes, &h, &structs, &fields)?;
    validate_label_array(&mut label_slices)?;
    validate_value_encodings(bytes, &h, &fields)?;
    let owners = validate_indices_and_ownership(bytes, &h, &structs, &fields, &label_slices)?;
    validate_loc_limits_and_uniqueness(bytes, &h, &fields, limits)?;
    let mut used = filled_vec(false, structs.len(), "input.structs")?;
    let context = ReadContext {
        bytes,
        header: &h,
        structs: &structs,
        fields: &fields,
        labels: &label_slices,
        limits,
    };
    let root = context.build_struct(0, 0, &mut used)?;
    if used.iter().any(|v| !*v) || owners.first() != Some(&0) {
        return Err(layout(
            "input.structs",
            "canonical tree is not fully reachable",
        ));
    }
    Ok(GffDocumentV1 {
        schema_version: 1,
        file_type: h.file_type,
        root,
    })
}

fn validate_indices_and_ownership(
    bytes: &[u8],
    h: &Header,
    structs: &[PStruct],
    fields: &[PField],
    labels: &[&[u8]],
) -> Result<Vec<u32>, GffErrorV1> {
    if structs[0].id != u32::MAX {
        return Err(layout(
            "input.structs[0].structId",
            "root struct ID must be 0xffffffff",
        ));
    }
    let mut unique_labels = HashSet::new();
    unique_labels
        .try_reserve(labels.len())
        .map_err(|_| alloc("input.labels"))?;
    for (index, label) in labels.iter().enumerate() {
        if !unique_labels.insert(*label) {
            return Err(err(
                "M6-GFF-LAYOUT-INVALID",
                format!("input.labels[{index}]"),
                "LabelArray values must be globally unique",
            ));
        }
    }
    let mut label_references = filled_vec(0u32, labels.len(), "input.labels")?;
    let mut label_last_struct = filled_vec(None::<usize>, labels.len(), "input.labels")?;
    let mut field_owned = filled_vec(false, fields.len(), "input.fields")?;
    let mut fi_cursor = 0u32;
    let mut expected_field_index = 0u32;
    for (si, s) in structs.iter().enumerate() {
        if s.count == 0 {
            if s.data != 0 {
                return Err(layout(
                    format!("input.structs[{si}].data"),
                    "zero-field struct data must be zero",
                ));
            }
        } else if s.count > 1 {
            if s.data != fi_cursor {
                return Err(layout(
                    format!("input.structs[{si}].data"),
                    "FieldIndices records are not canonical",
                ));
            }
            let bytes_len = mul_u32(s.count, 4, "fieldIndices")?;
            let end = add_u32(s.data, bytes_len, "fieldIndices")?;
            if end > h.fic {
                return Err(err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.structs[{si}]"),
                    "FieldIndices range is out of bounds",
                ));
            }
            fi_cursor = end;
        }
        for position in 0..s.count {
            let u = field_index_at(bytes, h, *s, position)? as usize;
            if u >= fields.len() {
                return Err(err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.structs[{si}]"),
                    "field index is out of bounds",
                ));
            }
            if u as u32 != expected_field_index {
                return Err(layout(
                    format!("input.structs[{si}].fields[{position}]"),
                    "FieldArray encounter order must be the exact sequence 0..FieldCount",
                ));
            }
            expected_field_index = expected_field_index
                .checked_add(1)
                .ok_or_else(|| layout("input.fields", "field encounter index overflow"))?;
            if field_owned[u] {
                return Err(layout(
                    format!("input.structs[{si}]"),
                    "FieldArray index must have exact one owner",
                ));
            }
            field_owned[u] = true;
            if fields[u].label as usize >= labels.len() {
                return Err(err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.fields[{u}].label"),
                    "label index is out of bounds",
                ));
            }
            let label_index = fields[u].label as usize;
            if label_last_struct[label_index] == Some(si) {
                return Err(err(
                    "M6-GFF-DUPLICATE-LABEL",
                    format!("input.structs[{si}]"),
                    "labels in one struct must be unique",
                ));
            }
            label_last_struct[label_index] = Some(si);
            label_references[label_index] = label_references[label_index]
                .checked_add(1)
                .ok_or_else(|| layout("input.labels", "label reference count overflow"))?;
        }
    }
    if fi_cursor != h.fic || field_owned.iter().any(|v| !*v) || label_references.contains(&0) {
        return Err(layout(
            "input.fieldIndices",
            "field/label ownership is incomplete or FieldIndices has trailing bytes",
        ));
    }
    validate_complex_coverage(bytes, h, fields)?;
    let mut owners = filled_vec(0u32, structs.len(), "input.ownership")?;
    for s in structs {
        for position in 0..s.count {
            let index = field_index_at(bytes, h, *s, position)? as usize;
            let f = fields[index];
            match f.ty {
                14 => add_owner(f.data, &mut owners)?,
                15 => {
                    let (start, count) = list_record(bytes, h, f.data)?;
                    for item in 0..count {
                        add_owner(u32at(bytes, start + 4 + item as usize * 4)?, &mut owners)?;
                    }
                }
                _ => {}
            }
        }
    }
    if owners[0] != 0 || owners.iter().skip(1).any(|v| *v != 1) {
        return Err(layout(
            "input.ownership",
            "root must have no parent and every non-root exact one owner",
        ));
    }
    let mut visited = filled_vec(false, structs.len(), "input.ownership")?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(structs.len())
        .map_err(|_| alloc("input.ownership"))?;
    stack.push(0usize);
    while let Some(i) = stack.pop() {
        if visited[i] {
            return Err(layout("input.ownership", "cycle or reuse detected"));
        }
        visited[i] = true;
        let s = structs[i];
        for position in 0..s.count {
            let field = fields[field_index_at(bytes, h, s, position)? as usize];
            match field.ty {
                14 => stack.push(field.data as usize),
                15 => {
                    let (start, count) = list_record(bytes, h, field.data)?;
                    for item in 0..count {
                        stack.push(u32at(bytes, start + 4 + item as usize * 4)? as usize);
                    }
                }
                _ => {}
            }
        }
    }
    if visited.iter().any(|v| !*v) {
        return Err(layout("input.ownership", "unreachable struct detected"));
    }
    Ok(owners)
}

fn validate_raw_references(
    bytes: &[u8],
    h: &Header,
    structs: &[PStruct],
    fields: &[PField],
    label_count: usize,
) -> Result<(), GffErrorV1> {
    for (struct_index, structure) in structs.iter().enumerate() {
        if structure.count > 1 {
            if !structure.data.is_multiple_of(4) {
                return Err(err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.structs[{struct_index}].data"),
                    "FieldIndices offset must be u32 aligned",
                ));
            }
            let end = add_u32(
                structure.data,
                mul_u32(structure.count, 4, "fieldIndices")?,
                "fieldIndices",
            )?;
            if end > h.fic {
                return Err(err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.structs[{struct_index}]"),
                    "FieldIndices range is out of bounds",
                ));
            }
        }
        for position in 0..structure.count {
            let field_index = field_index_at(bytes, h, *structure, position)? as usize;
            let field = fields.get(field_index).ok_or_else(|| {
                err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.structs[{struct_index}]"),
                    "field index is out of bounds",
                )
            })?;
            if field.label as usize >= label_count {
                return Err(err(
                    "M6-GFF-INDEX-OOB",
                    format!("input.fields[{field_index}].label"),
                    "label index is out of bounds",
                ));
            }
        }
    }
    Ok(())
}

fn validate_raw_payload_bounds(
    bytes: &[u8],
    h: &Header,
    structs: &[PStruct],
    fields: &[PField],
) -> Result<(), GffErrorV1> {
    for field in fields {
        let path = "input.fields";
        match field.ty {
            6 | 7 | 9 => require_field_data_range(field.data, 8, h, path)?,
            10 | 13 => {
                require_field_data_range(field.data, 4, h, path)?;
                let length = u32at(bytes, h.doff as usize + field.data as usize)?;
                require_field_data_range(field.data, add_u32(4, length, path)?, h, path)?;
            }
            11 => {
                require_field_data_range(field.data, 1, h, path)?;
                let length = bytes[h.doff as usize + field.data as usize] as u32;
                require_field_data_range(field.data, add_u32(1, length, path)?, h, path)?;
            }
            12 => {
                require_field_data_range(field.data, 4, h, path)?;
                let total = u32at(bytes, h.doff as usize + field.data as usize)?;
                require_field_data_range(field.data, add_u32(4, total, path)?, h, path)?;
            }
            14 => {
                if field.data as usize >= structs.len() {
                    return Err(err(
                        "M6-GFF-INDEX-OOB",
                        path,
                        "struct index is out of bounds",
                    ));
                }
            }
            15 => {
                let (start, count) = list_record(bytes, h, field.data)?;
                for item in 0..count {
                    let child = u32at(bytes, start + 4 + item as usize * 4)?;
                    if child as usize >= structs.len() {
                        return Err(err(
                            "M6-GFF-INDEX-OOB",
                            path,
                            "list struct index is out of bounds",
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn require_field_data_range(
    offset: u32,
    length: u32,
    h: &Header,
    path: &str,
) -> Result<(), GffErrorV1> {
    let end = add_u32(offset, length, path)?;
    if end > h.dc {
        return Err(err(
            "M6-GFF-INDEX-OOB",
            path,
            "FieldData range is out of bounds",
        ));
    }
    Ok(())
}

fn validate_label_array(labels: &mut [&[u8]]) -> Result<(), GffErrorV1> {
    for (index, label) in labels.iter_mut().enumerate() {
        let raw = *label;
        let length = raw.iter().position(|byte| *byte == 0).unwrap_or(16);
        if length == 0
            || !raw[..length].iter().all(u8::is_ascii)
            || !raw[length..].iter().all(|byte| *byte == 0)
        {
            return Err(err(
                "M6-GFF-LABEL-INVALID",
                format!("input.labels[{index}]"),
                "label encoding is not canonical ASCII/NUL padding",
            ));
        }
        *label = &raw[..length];
    }
    Ok(())
}

fn add_owner(child: u32, owners: &mut [u32]) -> Result<(), GffErrorV1> {
    let c = child as usize;
    if c >= owners.len() {
        return Err(err(
            "M6-GFF-INDEX-OOB",
            "input.ownership",
            "struct index is out of bounds",
        ));
    }
    owners[c] = owners[c]
        .checked_add(1)
        .ok_or_else(|| layout("input.ownership", "owner count overflow"))?;
    Ok(())
}

fn field_index_at(
    bytes: &[u8],
    h: &Header,
    structure: PStruct,
    position: u32,
) -> Result<u32, GffErrorV1> {
    if structure.count == 1 {
        Ok(structure.data)
    } else {
        u32at(
            bytes,
            h.fio as usize + structure.data as usize + position as usize * 4,
        )
    }
}

fn list_record(bytes: &[u8], h: &Header, offset: u32) -> Result<(usize, u32), GffErrorV1> {
    if offset >= h.lic {
        return Err(err(
            "M6-GFF-INDEX-OOB",
            "input.listIndices",
            "list offset out of bounds",
        ));
    }
    let start = h.lio as usize + offset as usize;
    let count = u32at(bytes, start)?;
    let end = add_u32(
        offset,
        add_u32(4, mul_u32(count, 4, "list")?, "list")?,
        "list",
    )?;
    if end > h.lic {
        return Err(err(
            "M6-GFF-INDEX-OOB",
            "input.listIndices",
            "list record out of bounds",
        ));
    }
    Ok((start, count))
}

fn validate_value_encodings(bytes: &[u8], h: &Header, fields: &[PField]) -> Result<(), GffErrorV1> {
    let data = &bytes[h.doff as usize..(h.doff + h.dc) as usize];
    for field in fields {
        let path = "input.fields";
        match field.ty {
            0 | 1 if field.data > 255 => {
                return Err(err(
                    "M6-GFF-VALUE-INVALID",
                    path,
                    "inline byte/char has nonzero high bits",
                ));
            }
            2 | 3 if field.data > 65_535 => {
                return Err(err(
                    "M6-GFF-VALUE-INVALID",
                    path,
                    "inline word/short has nonzero high bits",
                ));
            }
            8 if !f32::from_bits(field.data).is_finite() => {
                return Err(err("M6-GFF-VALUE-INVALID", path, "FLOAT is non-finite"));
            }
            9 => {
                let value =
                    f64::from_bits(u64::from_le_bytes(slice8(data, field.data as usize, path)?));
                if !value.is_finite() {
                    return Err(err("M6-GFF-VALUE-INVALID", path, "DOUBLE is non-finite"));
                }
            }
            11 => {
                let at = field.data as usize;
                let length = data[at] as usize;
                if length > 16 {
                    return Err(err(
                        "M6-GFF-VALUE-INVALID",
                        path,
                        "CResRef exceeds 16 bytes",
                    ));
                }
                let value = std::str::from_utf8(&data[at + 1..at + 1 + length])
                    .map_err(|_| err("M6-GFF-VALUE-INVALID", path, "CResRef is not ASCII"))?;
                validate_resref(value, path)?;
            }
            12 => validate_encoded_loc_structure(data, field.data as usize, path)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_encoded_loc_structure(data: &[u8], at: usize, path: &str) -> Result<(), GffErrorV1> {
    let total = u32at(data, at)? as usize;
    if total < 8 {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "LocString total_size is shorter than metadata",
        ));
    }
    let end = at
        .checked_add(4)
        .and_then(|v| v.checked_add(total))
        .ok_or_else(|| layout(path, "LocString range overflow"))?;
    let count = u32at(data, at + 8)? as usize;
    if count > (total - 8) / 8 {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "LocString substring count cannot fit total_size",
        ));
    }
    let mut cursor = at + 12;
    for _ in 0..count {
        let id = i32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
        let signed = i32::from_le_bytes(data[cursor + 4..cursor + 8].try_into().unwrap());
        if id < 0 {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString string_id must be nonnegative",
            ));
        }
        if signed < 0 {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString byte length is negative",
            ));
        }
        cursor = cursor
            .checked_add(8 + signed as usize)
            .ok_or_else(|| layout(path, "LocString length overflow"))?;
        if cursor > end {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString substring exceeds total_size",
            ));
        }
    }
    if cursor != end {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "LocString total_size has trailing bytes",
        ));
    }
    Ok(())
}

fn validate_loc_limits_and_uniqueness(
    bytes: &[u8],
    h: &Header,
    fields: &[PField],
    limits: &GffLimitsV1,
) -> Result<(), GffErrorV1> {
    let data = &bytes[h.doff as usize..(h.doff + h.dc) as usize];
    for (index, field) in fields.iter().enumerate() {
        if field.ty != 12 {
            continue;
        }
        let path = format!("input.fields[{index}]");
        let at = field.data as usize;
        let total = u32at(data, at)? as usize;
        let physical_length = total
            .checked_add(4)
            .ok_or_else(|| layout(&path, "LocString physical length overflow"))?;
        if physical_length > limits.max_loc_string_bytes as usize {
            return Err(err(
                "M6-GFF-LIMIT-EXCEEDED",
                &path,
                "LocString exceeds maxLocStringBytes",
            ));
        }
        let count = u32at(data, at + 8)? as usize;
        let mut ids = HashSet::new();
        if count > 0 {
            fail_loc_uniqueness_allocation(&path, count)?;
            ids.try_reserve(count).map_err(|_| alloc(&path))?;
        }
        let mut cursor = at + 12;
        for _ in 0..count {
            let id = i32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
            let length =
                i32::from_le_bytes(data[cursor + 4..cursor + 8].try_into().unwrap()) as usize;
            if !ids.insert(id) {
                return Err(err(
                    "M6-GFF-VALUE-INVALID",
                    &path,
                    "LocString string_id must be unique",
                ));
            }
            cursor += 8 + length;
        }
    }
    Ok(())
}

fn fail_loc_uniqueness_allocation(_path: &str, _count: usize) -> Result<(), GffErrorV1> {
    #[cfg(test)]
    {
        if FORCE_NEXT_LOC_UNIQUENESS_ALLOCATION_FAILURE.swap(false, Ordering::SeqCst) {
            return Err(alloc(_path));
        }
        TRACKED_INPUT_MATERIALIZED_BYTES.fetch_add(
            _count.saturating_mul(std::mem::size_of::<i32>()),
            Ordering::SeqCst,
        );
    }
    Ok(())
}

fn validate_complex_coverage(
    bytes: &[u8],
    h: &Header,
    fields: &[PField],
) -> Result<(), GffErrorV1> {
    let mut data_cursor = 0u32;
    let mut list_cursor = 0u32;
    for (i, f) in fields.iter().enumerate() {
        match f.ty {
            0..=5 | 8 | 14 => {}
            6 | 7 | 9 => consume_exact(
                f.data,
                8,
                &mut data_cursor,
                h.dc,
                format!("input.fields[{i}]"),
            )?,
            10 | 13 => {
                if f.data != data_cursor {
                    return Err(layout(
                        format!("input.fields[{i}]"),
                        "FieldData records are not canonical",
                    ));
                }
                let n = u32at(bytes, h.doff as usize + f.data as usize)?;
                consume_exact(
                    f.data,
                    add_u32(4, n, "payload")?,
                    &mut data_cursor,
                    h.dc,
                    format!("input.fields[{i}]"),
                )?;
            }
            11 => {
                if f.data != data_cursor {
                    return Err(layout(
                        format!("input.fields[{i}]"),
                        "FieldData records are not canonical",
                    ));
                }
                let n = *bytes
                    .get(h.doff as usize + f.data as usize)
                    .ok_or_else(|| {
                        err(
                            "M6-GFF-INDEX-OOB",
                            format!("input.fields[{i}]"),
                            "ResRef offset out of bounds",
                        )
                    })? as u32;
                consume_exact(
                    f.data,
                    1 + n,
                    &mut data_cursor,
                    h.dc,
                    format!("input.fields[{i}]"),
                )?;
            }
            12 => {
                if f.data != data_cursor {
                    return Err(layout(
                        format!("input.fields[{i}]"),
                        "FieldData records are not canonical",
                    ));
                }
                let n = u32at(bytes, h.doff as usize + f.data as usize)?;
                consume_exact(
                    f.data,
                    add_u32(4, n, "locString")?,
                    &mut data_cursor,
                    h.dc,
                    format!("input.fields[{i}]"),
                )?;
            }
            15 => {
                if f.data != list_cursor {
                    return Err(layout(
                        format!("input.fields[{i}]"),
                        "ListIndices records are not canonical",
                    ));
                }
                let count = u32at(bytes, h.lio as usize + f.data as usize)?;
                list_cursor = add_u32(
                    f.data,
                    add_u32(4, mul_u32(count, 4, "list")?, "list")?,
                    "list",
                )?;
                if list_cursor > h.lic {
                    return Err(err(
                        "M6-GFF-INDEX-OOB",
                        format!("input.fields[{i}]"),
                        "list range out of bounds",
                    ));
                }
            }
            _ => {}
        }
    }
    if data_cursor != h.dc || list_cursor != h.lic {
        return Err(layout(
            "input.sections",
            "complex sections have gaps or trailing bytes",
        ));
    }
    Ok(())
}

struct ReadContext<'a> {
    bytes: &'a [u8],
    header: &'a Header,
    structs: &'a [PStruct],
    fields: &'a [PField],
    labels: &'a [&'a [u8]],
    limits: &'a GffLimitsV1,
}

impl ReadContext<'_> {
    fn build_struct(
        &self,
        index: u32,
        depth: u32,
        used: &mut [bool],
    ) -> Result<GffStructV1, GffErrorV1> {
        if depth > self.limits.max_depth {
            return Err(err(
                "M6-GFF-DEPTH-LIMIT-EXCEEDED",
                format!("input.structs[{index}]"),
                "tree depth exceeds maxDepth",
            ));
        }
        let i = index as usize;
        if used[i] {
            return Err(layout("input.ownership", "struct reuse detected"));
        }
        used[i] = true;
        let s = self.structs[i];
        if s.count > self.limits.max_fields_per_struct {
            return Err(err(
                "M6-GFF-LIMIT-EXCEEDED",
                format!("input.structs[{i}]"),
                "field count exceeds maxFieldsPerStruct",
            ));
        }
        let indices = struct_field_indices(self.bytes, self.header, s)?;
        let mut out = Vec::new();
        out.try_reserve_exact(indices.len())
            .map_err(|_| alloc(format!("input.structs[{i}].fields")))?;
        for idx in indices {
            let f = self.fields[idx as usize];
            let source = std::str::from_utf8(self.labels[f.label as usize]).map_err(|_| {
                err(
                    "M6-GFF-LABEL-INVALID",
                    format!("input.fields[{idx}].label"),
                    "label is not ASCII",
                )
            })?;
            let label = copy_string(source, "input.label")?;
            let value = self.decode_value(f, depth, used, idx)?;
            out.push(GffFieldV1 { label, value });
        }
        Ok(GffStructV1 {
            struct_id: s.id,
            fields: out,
        })
    }

    fn decode_value(
        &self,
        f: PField,
        depth: u32,
        used: &mut [bool],
        field_index: u32,
    ) -> Result<GffValueV1, GffErrorV1> {
        let d =
            &self.bytes[self.header.doff as usize..(self.header.doff + self.header.dc) as usize];
        let at = f.data as usize;
        let path = format!("input.fields[{field_index}]");
        Ok(match f.ty {
            0 => GffValueV1::Byte(f.data as u8),
            1 => GffValueV1::Char(f.data as u8 as i8),
            2 => GffValueV1::Word(f.data as u16),
            3 => GffValueV1::Short(f.data as u16 as i16),
            4 => GffValueV1::Dword(f.data),
            5 => GffValueV1::Int(f.data as i32),
            6 => GffValueV1::Dword64(u64::from_le_bytes(slice8(d, at, &path)?)),
            7 => GffValueV1::Int64(i64::from_le_bytes(slice8(d, at, &path)?)),
            8 => {
                let v = f32::from_bits(f.data);
                if !v.is_finite() {
                    return Err(err("M6-GFF-VALUE-INVALID", path, "FLOAT is non-finite"));
                }
                GffValueV1::Float(v)
            }
            9 => {
                let v = f64::from_bits(u64::from_le_bytes(slice8(d, at, &path)?));
                if !v.is_finite() {
                    return Err(err("M6-GFF-VALUE-INVALID", path, "DOUBLE is non-finite"));
                }
                GffValueV1::Double(v)
            }
            10 => {
                let n = u32at(d, at)? as usize;
                check_len(n, self.limits.max_string_bytes, &path, "maxStringBytes")?;
                GffValueV1::String(copy_bytes(&d[at + 4..at + 4 + n], &path)?)
            }
            11 => {
                let n = d[at] as usize;
                if n > 16 {
                    return Err(err(
                        "M6-GFF-VALUE-INVALID",
                        path,
                        "CResRef exceeds 16 bytes",
                    ));
                }
                let s = std::str::from_utf8(&d[at + 1..at + 1 + n])
                    .map_err(|_| err("M6-GFF-VALUE-INVALID", &path, "CResRef is not ASCII"))?;
                validate_resref(s, &path)?;
                GffValueV1::ResRef(copy_string(s, &path)?)
            }
            12 => GffValueV1::LocString(decode_loc(d, at, self.limits, &path)?),
            13 => {
                let n = u32at(d, at)? as usize;
                check_len(n, self.limits.max_void_bytes, &path, "maxVoidBytes")?;
                GffValueV1::Void(copy_bytes(&d[at + 4..at + 4 + n], &path)?)
            }
            14 => GffValueV1::Struct(self.build_struct(f.data, depth + 1, used)?),
            15 => {
                let (start, count) = list_record(self.bytes, self.header, f.data)?;
                check_len(
                    count as usize,
                    self.limits.max_list_elements,
                    &path,
                    "maxListElements",
                )?;
                let mut values = Vec::new();
                values
                    .try_reserve_exact(count as usize)
                    .map_err(|_| alloc(&path))?;
                for item in 0..count {
                    let idx = u32at(self.bytes, start + 4 + item as usize * 4)?;
                    values.push(self.build_struct(idx, depth + 1, used)?);
                }
                GffValueV1::List(values)
            }
            _ => return Err(err("M6-GFF-TYPE-UNSUPPORTED", path, "unsupported type")),
        })
    }
}

fn decode_loc(
    d: &[u8],
    at: usize,
    limits: &GffLimitsV1,
    path: &str,
) -> Result<GffLocStringV1, GffErrorV1> {
    let total = u32at(d, at)? as usize;
    let physical_length = total
        .checked_add(4)
        .ok_or_else(|| layout(path, "LocString physical length overflow"))?;
    if physical_length > limits.max_loc_string_bytes as usize {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            "LocString exceeds maxLocStringBytes",
        ));
    }
    if total < 8 {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "LocString total_size is shorter than metadata",
        ));
    }
    let end = at
        .checked_add(4)
        .and_then(|value| value.checked_add(total))
        .ok_or_else(|| layout(path, "LocString range overflow"))?;
    let string_ref = u32at(d, at + 4)?;
    let count = u32at(d, at + 8)? as usize;
    if count > (total - 8) / 8 {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "LocString substring count cannot fit total_size",
        ));
    }
    let mut cursor = at + 12;
    let mut subs = Vec::new();
    subs.try_reserve_exact(count).map_err(|_| alloc(path))?;
    let mut ids = HashSet::new();
    ids.try_reserve(count).map_err(|_| alloc(path))?;
    for _ in 0..count {
        if cursor + 8 > end {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString substring header exceeds total_size",
            ));
        }
        let id = i32::from_le_bytes(d[cursor..cursor + 4].try_into().unwrap());
        let signed = i32::from_le_bytes(d[cursor + 4..cursor + 8].try_into().unwrap());
        if signed < 0 {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString byte length is negative",
            ));
        }
        let n = signed as usize;
        check_len(n, limits.max_string_bytes, path, "maxStringBytes")?;
        cursor = cursor
            .checked_add(8 + n)
            .ok_or_else(|| layout(path, "LocString length overflow"))?;
        if cursor > end {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString substring exceeds total_size",
            ));
        }
        if id < 0 || !ids.insert(id) {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString string_id must be nonnegative and unique",
            ));
        }
        subs.push(GffLocSubstringV1 {
            string_id: id,
            bytes: copy_bytes(&d[cursor - n..cursor], path)?,
        });
    }
    if cursor != end {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "LocString total_size has trailing bytes",
        ));
    }
    Ok(GffLocStringV1 {
        string_ref,
        substrings: subs,
    })
}

fn validate_layout(len: usize, h: &Header) -> Result<(), GffErrorV1> {
    if h.so != 56 {
        return Err(layout(
            "input.header.structOffset",
            "StructOffset must be 56",
        ));
    }
    let fo = add_u32(h.so, mul_u32(h.sc, 12, "structArray")?, "fieldOffset")?;
    let lo = add_u32(fo, mul_u32(h.fc, 12, "fieldArray")?, "labelOffset")?;
    let doff = add_u32(lo, mul_u32(h.lc, 16, "labelArray")?, "fieldDataOffset")?;
    let fio = add_u32(doff, h.dc, "fieldIndicesOffset")?;
    let lio = add_u32(fio, h.fic, "listIndicesOffset")?;
    let eof = add_u32(lio, h.lic, "EOF")?;
    if h.fo != fo
        || h.lo != lo
        || h.doff != doff
        || h.fio != fio
        || h.lio != lio
        || eof as usize != len
        || !h.fic.is_multiple_of(4)
        || !h.lic.is_multiple_of(4)
    {
        return Err(layout(
            "input.header.sections",
            "sections are not an exact contiguous chain to EOF",
        ));
    }
    Ok(())
}

fn validate_limits(l: &GffLimitsV1) -> Result<(), GffErrorV1> {
    let d = GffLimitsV1::default();
    if l.max_gff_bytes == 0
        || l.max_gff_bytes > d.max_gff_bytes
        || l.max_structs == 0
        || l.max_structs > d.max_structs
        || l.max_fields == 0
        || l.max_fields > d.max_fields
        || l.max_labels == 0
        || l.max_labels > d.max_labels
        || l.max_fields_per_struct == 0
        || l.max_fields_per_struct > d.max_fields_per_struct
        || l.max_list_elements == 0
        || l.max_list_elements > d.max_list_elements
        || l.max_depth == 0
        || l.max_depth > d.max_depth
        || l.max_string_bytes == 0
        || l.max_string_bytes > d.max_string_bytes
        || l.max_loc_string_bytes == 0
        || l.max_loc_string_bytes > d.max_loc_string_bytes
        || l.max_void_bytes == 0
        || l.max_void_bytes > d.max_void_bytes
        || l.max_diagnostics == 0
        || l.max_diagnostics > d.max_diagnostics
    {
        return Err(err(
            "M6-GFF-OPTIONS-INVALID",
            "limits",
            "all limits must be nonzero and at most their hard defaults",
        ));
    }
    Ok(())
}
fn validate_label(s: &str, path: &str) -> Result<(), GffErrorV1> {
    if s.is_empty() || s.len() > 16 || !s.is_ascii() || s.as_bytes().contains(&0) {
        return Err(err(
            "M6-GFF-LABEL-INVALID",
            path,
            "label must be 1..=16 ASCII bytes without NUL",
        ));
    }
    Ok(())
}
fn validate_resref(s: &str, path: &str) -> Result<(), GffErrorV1> {
    if s.len() > 16
        || (!s.is_empty()
            && !s
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'))
    {
        return Err(err(
            "M6-GFF-VALUE-INVALID",
            path,
            "CResRef must be empty or lowercase [a-z0-9_]+ and at most 16 bytes",
        ));
    }
    Ok(())
}
fn validate_loc(v: &GffLocStringV1, l: &GffLimitsV1, path: &str) -> Result<(), GffErrorV1> {
    let mut encoded_length = 8u32;
    for substring in &v.substrings {
        if substring.string_id < 0 {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString string_id must be nonnegative and unique",
            ));
        }
        check_len(
            substring.bytes.len(),
            l.max_string_bytes,
            path,
            "maxStringBytes",
        )?;
        encoded_length = add_u32(
            encoded_length,
            add_u32(8, to_u32(substring.bytes.len(), path)?, path)?,
            path,
        )?;
        let physical_length = add_u32(4, encoded_length, path)?;
        if physical_length > l.max_loc_string_bytes {
            return Err(err(
                "M6-GFF-LIMIT-EXCEEDED",
                path,
                "LocString exceeds maxLocStringBytes",
            ));
        }
    }
    if add_u32(4, encoded_length, path)? > l.max_loc_string_bytes {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            "LocString exceeds maxLocStringBytes",
        ));
    }
    let mut ids = HashSet::new();
    ids.try_reserve(v.substrings.len())
        .map_err(|_| alloc(path))?;
    for s in &v.substrings {
        if s.string_id < 0 || !ids.insert(s.string_id) {
            return Err(err(
                "M6-GFF-VALUE-INVALID",
                path,
                "LocString string_id must be nonnegative and unique",
            ));
        }
    }
    debug_assert_eq!(loc_encoded_len(v)?, encoded_length);
    Ok(())
}
fn loc_encoded_len(v: &GffLocStringV1) -> Result<u32, GffErrorV1> {
    let mut n = 8u32;
    for s in &v.substrings {
        n = add_u32(
            n,
            add_u32(8, to_u32(s.bytes.len(), "locString.bytes")?, "locString")?,
            "locString",
        )?;
    }
    Ok(n)
}
fn struct_field_indices(bytes: &[u8], h: &Header, s: PStruct) -> Result<Vec<u32>, GffErrorV1> {
    if s.count == 0 {
        return Ok(Vec::new());
    }
    let mut v = Vec::new();
    v.try_reserve_exact(s.count as usize)
        .map_err(|_| alloc("input.fieldIndices"))?;
    if s.count == 1 {
        v.push(s.data);
        return Ok(v);
    }
    for j in 0..s.count {
        v.push(u32at(
            bytes,
            h.fio as usize + s.data as usize + j as usize * 4,
        )?);
    }
    Ok(v)
}
fn consume_exact(
    offset: u32,
    length: u32,
    cursor: &mut u32,
    limit: u32,
    path: String,
) -> Result<(), GffErrorV1> {
    if offset != *cursor {
        return Err(layout(path, "FieldData records are not canonical"));
    }
    let end = add_u32(offset, length, "fieldData")?;
    if end > limit {
        return Err(err(
            "M6-GFF-INDEX-OOB",
            "input.fieldData",
            "payload range out of bounds",
        ));
    }
    *cursor = end;
    Ok(())
}
fn u32at(bytes: &[u8], offset: usize) -> Result<u32, GffErrorV1> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| layout("input", "offset overflow"))?;
    let s = bytes.get(offset..end).ok_or_else(|| {
        err(
            "M6-GFF-INDEX-OOB",
            format!("input.byte[{offset}]"),
            "u32 read is out of bounds",
        )
    })?;
    Ok(u32::from_le_bytes(s.try_into().unwrap()))
}
fn slice8(bytes: &[u8], offset: usize, path: &str) -> Result<[u8; 8], GffErrorV1> {
    bytes
        .get(
            offset
                ..offset
                    .checked_add(8)
                    .ok_or_else(|| layout(path, "offset overflow"))?,
        )
        .ok_or_else(|| err("M6-GFF-INDEX-OOB", path, "8-byte value is out of bounds"))?
        .try_into()
        .map_err(|_| err("M6-GFF-INDEX-OOB", path, "8-byte value is out of bounds"))
}
fn append_data(out: &mut Vec<u8>, bytes: &[u8], limits: &GffLimitsV1) -> Result<u32, GffErrorV1> {
    let o = to_u32(out.len(), "fieldData")?;
    let next = out
        .len()
        .checked_add(bytes.len())
        .ok_or_else(|| layout("fieldData", "length overflow"))?;
    if next as u64 > limits.max_gff_bytes {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            "fieldData",
            "field data exceeds maxGffBytes",
        ));
    }
    reserve_bytes(out, bytes.len(), "fieldData")?;
    out.extend_from_slice(bytes);
    Ok(o)
}
fn reserve_bytes(out: &mut Vec<u8>, n: usize, path: &str) -> Result<(), GffErrorV1> {
    out.try_reserve(n).map_err(|_| alloc(path))
}
fn copy_bytes(v: &[u8], path: &str) -> Result<Vec<u8>, GffErrorV1> {
    #[cfg(test)]
    TRACKED_INPUT_MATERIALIZED_BYTES.fetch_add(v.len(), Ordering::SeqCst);
    let mut out = Vec::new();
    fail_tracked_allocation(path)?;
    out.try_reserve_exact(v.len()).map_err(|_| alloc(path))?;
    out.extend_from_slice(v);
    Ok(out)
}
fn copy_string(v: &str, path: &str) -> Result<String, GffErrorV1> {
    #[cfg(test)]
    TRACKED_INPUT_MATERIALIZED_BYTES.fetch_add(v.len(), Ordering::SeqCst);
    let mut out = String::new();
    fail_tracked_allocation(path)?;
    out.try_reserve_exact(v.len()).map_err(|_| alloc(path))?;
    out.push_str(v);
    Ok(out)
}
fn filled_vec<T: Clone>(value: T, n: usize, path: &str) -> Result<Vec<T>, GffErrorV1> {
    let mut out = Vec::new();
    fail_tracked_allocation(path)?;
    out.try_reserve_exact(n).map_err(|_| alloc(path))?;
    out.resize(n, value);
    Ok(out)
}
fn fail_tracked_allocation(_path: &str) -> Result<(), GffErrorV1> {
    #[cfg(test)]
    if FORCE_NEXT_TRACKED_ALLOCATION_FAILURE.swap(false, Ordering::SeqCst) {
        return Err(alloc(_path));
    }
    Ok(())
}
fn check_len(n: usize, max: u32, path: &str, name: &str) -> Result<(), GffErrorV1> {
    if n > max as usize {
        return Err(err(
            "M6-GFF-LIMIT-EXCEEDED",
            path,
            format!("length exceeds {name}"),
        ));
    }
    Ok(())
}
fn to_u32(v: usize, path: &str) -> Result<u32, GffErrorV1> {
    u32::try_from(v).map_err(|_| layout(path, "value does not fit u32"))
}
fn add_u32(a: u32, b: u32, path: &str) -> Result<u32, GffErrorV1> {
    a.checked_add(b)
        .ok_or_else(|| layout(path, "u32 addition overflow"))
}
fn mul_u32(a: u32, b: u32, path: &str) -> Result<u32, GffErrorV1> {
    a.checked_mul(b)
        .ok_or_else(|| layout(path, "u32 multiplication overflow"))
}
fn sha256_hex(bytes: &[u8]) -> Result<String, GffErrorV1> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::new();
    output
        .try_reserve_exact(64)
        .map_err(|_| alloc("report.outputSha256"))?;
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(output)
}
fn err(code: &str, path: impl Into<String>, message: impl Into<String>) -> GffErrorV1 {
    GffErrorV1::fatal(code, path, message)
}
fn layout(path: impl Into<String>, message: impl Into<String>) -> GffErrorV1 {
    err("M6-GFF-LAYOUT-INVALID", path, message)
}
fn alloc(path: impl Into<String>) -> GffErrorV1 {
    err("M6-GFF-ALLOCATION-FAILED", path, "allocation failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_document() -> GffDocumentV1 {
        GffDocumentV1 {
            schema_version: 1,
            file_type: GffFileTypeV1::Utc,
            root: GffStructV1 {
                struct_id: u32::MAX,
                fields: vec![GffFieldV1 {
                    label: "Value".to_owned(),
                    value: GffValueV1::Byte(1),
                }],
            },
        }
    }

    #[test]
    fn forced_writer_and_reader_allocation_seams_are_stable_fatal_errors() {
        let _lock = ALLOCATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        FORCE_NEXT_TRACKED_ALLOCATION_FAILURE.store(true, Ordering::SeqCst);
        let writer_error = write_gff_v32(&tiny_document(), &GffWriterOptionsV1::default())
            .expect_err("forced writer allocation must fail");
        assert_eq!(writer_error.code, "M6-GFF-ALLOCATION-FAILED");

        let artifact = write_gff_v32(&tiny_document(), &GffWriterOptionsV1::default()).unwrap();
        FORCE_NEXT_TRACKED_ALLOCATION_FAILURE.store(true, Ordering::SeqCst);
        let reader_error = read_gff_v32(&artifact.payload, &GffLimitsV1::default())
            .expect_err("forced reader allocation must fail");
        assert_eq!(reader_error.code, "M6-GFF-ALLOCATION-FAILED");
    }

    #[test]
    fn max_gff_rejection_precedes_any_tracked_input_sized_materialization() {
        let _lock = ALLOCATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let input = GffDocumentV1 {
            schema_version: 1,
            file_type: GffFileTypeV1::Utc,
            root: GffStructV1 {
                struct_id: u32::MAX,
                fields: vec![GffFieldV1 {
                    label: "LargeText".to_owned(),
                    value: GffValueV1::String(vec![7; 1_024]),
                }],
            },
        };
        let options = GffWriterOptionsV1 {
            schema_version: 1,
            limits: GffLimitsV1 {
                max_gff_bytes: 68,
                ..GffLimitsV1::default()
            },
        };
        FORCE_NEXT_TRACKED_ALLOCATION_FAILURE.store(true, Ordering::SeqCst);
        TRACKED_INPUT_MATERIALIZED_BYTES.store(0, Ordering::SeqCst);
        let error = write_gff_v32(&input, &options).unwrap_err();
        assert_eq!(error.code, "M6-GFF-LIMIT-EXCEEDED");
        assert_eq!(TRACKED_INPUT_MATERIALIZED_BYTES.load(Ordering::SeqCst), 0);
        assert!(
            FORCE_NEXT_TRACKED_ALLOCATION_FAILURE.swap(false, Ordering::SeqCst),
            "the pre-layout gate must not consume the materialization seam"
        );
    }

    #[test]
    fn oversized_physical_locstring_hits_limit_before_uniqueness_allocation() {
        let _lock = ALLOCATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let input = GffDocumentV1 {
            schema_version: 1,
            file_type: GffFileTypeV1::Utc,
            root: GffStructV1 {
                struct_id: u32::MAX,
                fields: vec![GffFieldV1 {
                    label: "Name".to_owned(),
                    value: GffValueV1::LocString(GffLocStringV1 {
                        string_ref: u32::MAX,
                        substrings: vec![GffLocSubstringV1 {
                            string_id: 0,
                            bytes: vec![7; 64],
                        }],
                    }),
                }],
            },
        };
        let artifact = write_gff_v32(&input, &GffWriterOptionsV1::default()).unwrap();
        let limits = GffLimitsV1 {
            max_loc_string_bytes: 20,
            ..GffLimitsV1::default()
        };
        TRACKED_INPUT_MATERIALIZED_BYTES.store(0, Ordering::SeqCst);
        FORCE_NEXT_LOC_UNIQUENESS_ALLOCATION_FAILURE.store(true, Ordering::SeqCst);
        let error = read_gff_v32(&artifact.payload, &limits).unwrap_err();
        assert_eq!(error.code, "M6-GFF-LIMIT-EXCEEDED");
        assert_eq!(TRACKED_INPUT_MATERIALIZED_BYTES.load(Ordering::SeqCst), 0);
        assert!(FORCE_NEXT_LOC_UNIQUENESS_ALLOCATION_FAILURE.swap(false, Ordering::SeqCst));
    }
}
