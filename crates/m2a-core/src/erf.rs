use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

#[cfg(test)]
use std::cell::Cell;

use serde::Serialize;

pub const HEADER_SIZE: usize = 160;
pub const KEY_ENTRY_SIZE: usize = 24;
pub const RESOURCE_ENTRY_SIZE: usize = 8;
pub const DEFAULT_MAX_ENTRY_COUNT: usize = 262_144;

pub const SIGNATURE_UNSUPPORTED: &str = "M2A-ERF-SIGNATURE-UNSUPPORTED";
pub const VERSION_UNSUPPORTED: &str = "M2A-ERF-VERSION-UNSUPPORTED";
pub const HEADER_OOB: &str = "M2A-ERF-HEADER-OOB";
pub const KEY_TABLE_OOB: &str = "M2A-ERF-KEY-TABLE-OOB";
pub const RESOURCE_TABLE_OOB: &str = "M2A-ERF-RESOURCE-TABLE-OOB";
pub const PAYLOAD_OOB: &str = "M2A-ERF-PAYLOAD-OOB";
pub const DUPLICATE_KEY: &str = "M2A-ERF-DUPLICATE-KEY";
pub const RESOURCE_ID_INVALID: &str = "M2A-ERF-RESOURCE-ID-INVALID";
pub const RESREF_INVALID: &str = "M2A-ERF-RESREF-INVALID";
pub const RESOURCE_MISSING: &str = "M2A-ERF-RESOURCE-MISSING";
pub const LIMIT_EXCEEDED: &str = "M2A-ERF-LIMIT-EXCEEDED";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct NormalizedResourceKey {
    resref: [u8; 16],
    resource_type: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IndexedPayloadRange {
    range: Range<usize>,
    resource_index: usize,
    descriptor_offset: usize,
}

#[cfg(test)]
thread_local! {
    static FAIL_NEXT_PARSER_ALLOCATION: Cell<bool> = const { Cell::new(false) };
}

/// Project safety limits applied before parser-owned collections are allocated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErfLimits {
    pub max_entry_count: usize,
}

impl Default for ErfLimits {
    fn default() -> Self {
        Self {
            max_entry_count: DEFAULT_MAX_ENTRY_COUNT,
        }
    }
}

/// Supported ERF-family V1.0 container signatures.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErfFileType {
    Erf,
    Hak,
}

impl ErfFileType {
    pub const fn signature(self) -> [u8; 4] {
        match self {
            Self::Erf => *b"ERF ",
            Self::Hak => *b"HAK ",
        }
    }

    fn parse(signature: &[u8], offset: usize) -> Result<Self, ErfError> {
        match signature {
            b"ERF " => Ok(Self::Erf),
            b"HAK " => Ok(Self::Hak),
            _ => Err(ErfError::new(
                SIGNATURE_UNSUPPORTED,
                offset,
                "expected ERF or HAK signature",
            )),
        }
    }
}

/// Stable public diagnostic for a malformed or unsupported ERF-family archive.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErfError {
    pub schema_version: u32,
    pub code: String,
    pub offset: usize,
    pub context: String,
}

impl ErfError {
    fn new(code: &str, offset: usize, context: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            code: code.to_owned(),
            offset,
            context: context.into(),
        }
    }
}

impl fmt::Display for ErfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.offset, self.context
        )
    }
}

impl std::error::Error for ErfError {}

/// Metadata for one key-table entry, in original key-table order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErfResource {
    pub resref: String,
    pub resource_id: u32,
    pub resource_type: u16,
    pub offset: usize,
    pub size: usize,
}

/// A validated read-only view of an ERF/HAK V1.0 byte buffer.
#[derive(Debug)]
pub struct ErfArchive<'a> {
    bytes: &'a [u8],
    file_type: ErfFileType,
    resources: Vec<ErfResource>,
    lookup: HashMap<NormalizedResourceKey, usize>,
}

impl<'a> ErfArchive<'a> {
    /// Parses and validates all tables and resource payload ranges.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ErfError> {
        Self::parse_with_limits(bytes, ErfLimits::default())
    }

    /// Parses with caller-supplied project safety limits.
    pub fn parse_with_limits(bytes: &'a [u8], limits: ErfLimits) -> Result<Self, ErfError> {
        if bytes.len() < HEADER_SIZE {
            return Err(ErfError::new(
                HEADER_OOB,
                bytes.len(),
                format!("ERF V1.0 header requires {HEADER_SIZE} bytes"),
            ));
        }

        let file_type = ErfFileType::parse(&bytes[0..4], 0)?;
        if &bytes[4..8] != b"V1.0" {
            return Err(ErfError::new(
                VERSION_UNSUPPORTED,
                4,
                "only ERF-family V1.0 is supported",
            ));
        }

        let localized_string_size = read_u32(bytes, 12, HEADER_OOB, "localized string table size")?;
        let entry_count = read_u32(bytes, 16, HEADER_OOB, "entry count")?;
        let localized_string_offset =
            read_u32(bytes, 20, HEADER_OOB, "localized string table offset")?;
        let key_table_offset = read_u32(bytes, 24, HEADER_OOB, "key table offset")?;
        let resource_table_offset = read_u32(bytes, 28, HEADER_OOB, "resource table offset")?;

        let entry_count_usize = usize::try_from(entry_count).map_err(|_| {
            ErfError::new(KEY_TABLE_OOB, 16, "entry count does not fit this platform")
        })?;
        if entry_count_usize > limits.max_entry_count {
            return Err(ErfError::new(
                LIMIT_EXCEEDED,
                16,
                format!(
                    "entry count {entry_count} exceeds configured limit {}",
                    limits.max_entry_count
                ),
            ));
        }
        let key_table_offset = usize::try_from(key_table_offset).map_err(|_| {
            ErfError::new(
                KEY_TABLE_OOB,
                24,
                "key table offset does not fit this platform",
            )
        })?;
        let resource_table_offset = usize::try_from(resource_table_offset).map_err(|_| {
            ErfError::new(
                RESOURCE_TABLE_OOB,
                28,
                "resource table offset does not fit this platform",
            )
        })?;
        let localized_string_size = usize::try_from(localized_string_size).map_err(|_| {
            ErfError::new(
                HEADER_OOB,
                12,
                "localized string table size does not fit this platform",
            )
        })?;
        let localized_string_offset = usize::try_from(localized_string_offset).map_err(|_| {
            ErfError::new(
                HEADER_OOB,
                20,
                "localized string table offset does not fit this platform",
            )
        })?;

        let key_table = checked_table(
            bytes,
            key_table_offset,
            entry_count_usize,
            KEY_ENTRY_SIZE,
            KEY_TABLE_OOB,
            "key table",
        )?;
        let resource_table = checked_table(
            bytes,
            resource_table_offset,
            entry_count_usize,
            RESOURCE_ENTRY_SIZE,
            RESOURCE_TABLE_OOB,
            "resource table",
        )?;

        let key_range = key_table_offset..key_table_offset + key_table.len();
        let resource_range = resource_table_offset..resource_table_offset + resource_table.len();
        if ranges_overlap(&key_range, &resource_range) {
            return Err(ErfError::new(
                RESOURCE_TABLE_OOB,
                resource_table_offset,
                "resource table overlaps key table",
            ));
        }

        let localized_range = if localized_string_size == 0 {
            None
        } else {
            if localized_string_offset < HEADER_SIZE {
                return Err(ErfError::new(
                    HEADER_OOB,
                    localized_string_offset,
                    "localized string table starts inside the fixed header",
                ));
            }
            checked_slice(
                bytes,
                localized_string_offset,
                localized_string_size,
                HEADER_OOB,
                "localized string table",
            )?;
            Some(localized_string_offset..localized_string_offset + localized_string_size)
        };
        if localized_range
            .as_ref()
            .is_some_and(|localized| ranges_overlap(localized, &key_range))
        {
            return Err(ErfError::new(
                KEY_TABLE_OOB,
                key_table_offset,
                "key table overlaps localized string table",
            ));
        }
        if localized_range
            .as_ref()
            .is_some_and(|localized| ranges_overlap(localized, &resource_range))
        {
            return Err(ErfError::new(
                RESOURCE_TABLE_OOB,
                resource_table_offset,
                "resource table overlaps localized string table",
            ));
        }

        let metadata_ranges = [0..HEADER_SIZE, key_range, resource_range];

        let mut lookup = HashMap::new();
        try_reserve_lookup(&mut lookup, entry_count_usize)?;
        let mut seen_payload_ranges = Vec::new();
        try_reserve_vec(
            &mut seen_payload_ranges,
            entry_count_usize,
            16,
            "payload range table",
        )?;
        let mut resources = Vec::new();
        try_reserve_vec(
            &mut resources,
            entry_count_usize,
            16,
            "resource metadata table",
        )?;

        for index in 0..entry_count_usize {
            let key_relative_offset = index * KEY_ENTRY_SIZE;
            let key_offset = key_table_offset + key_relative_offset;
            let key = &key_table[key_relative_offset..key_relative_offset + KEY_ENTRY_SIZE];
            let resref = parse_resref(&key[..16], key_offset)?;
            let resource_id = u32::from_le_bytes(key[16..20].try_into().expect("fixed key field"));
            let resource_type =
                u16::from_le_bytes(key[20..22].try_into().expect("fixed key field"));

            if resource_id != index as u32 {
                return Err(ErfError::new(
                    RESOURCE_ID_INVALID,
                    key_offset + 16,
                    format!("ERF V1.0 resource id {resource_id} must equal key index {index}"),
                ));
            }

            let normalized_key = normalize_resource_key(&key[..16], resource_type);
            if lookup.insert(normalized_key, index).is_some() {
                return Err(ErfError::new(
                    DUPLICATE_KEY,
                    key_offset,
                    format!(
                        "resource key ({resref}, {resource_type}) is duplicated case-insensitively"
                    ),
                ));
            }

            let resource_relative_offset = index * RESOURCE_ENTRY_SIZE;
            let resource_offset = resource_table_offset + resource_relative_offset;
            let descriptor = &resource_table
                [resource_relative_offset..resource_relative_offset + RESOURCE_ENTRY_SIZE];
            let payload_offset =
                u32::from_le_bytes(descriptor[..4].try_into().expect("fixed resource field"));
            let payload_size =
                u32::from_le_bytes(descriptor[4..].try_into().expect("fixed resource field"));
            let payload_offset = usize::try_from(payload_offset).map_err(|_| {
                ErfError::new(
                    PAYLOAD_OOB,
                    resource_offset,
                    "payload offset does not fit this platform",
                )
            })?;
            let payload_size = usize::try_from(payload_size).map_err(|_| {
                ErfError::new(
                    PAYLOAD_OOB,
                    resource_offset + 4,
                    "payload size does not fit this platform",
                )
            })?;
            checked_slice(
                bytes,
                payload_offset,
                payload_size,
                PAYLOAD_OOB,
                "resource payload",
            )?;
            let payload_range = payload_offset..payload_offset + payload_size;
            if payload_size != 0
                && (metadata_ranges
                    .iter()
                    .any(|metadata| ranges_overlap(&payload_range, metadata))
                    || localized_range
                        .as_ref()
                        .is_some_and(|localized| ranges_overlap(&payload_range, localized)))
            {
                return Err(ErfError::new(
                    PAYLOAD_OOB,
                    resource_offset,
                    "resource payload overlaps archive metadata",
                ));
            }
            if payload_size != 0 {
                seen_payload_ranges.push(IndexedPayloadRange {
                    range: payload_range,
                    resource_index: index,
                    descriptor_offset: resource_offset,
                });
            }

            resources.push(ErfResource {
                resref,
                resource_id,
                resource_type,
                offset: payload_offset,
                size: payload_size,
            });
        }

        seen_payload_ranges.sort_unstable_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then_with(|| left.range.end.cmp(&right.range.end))
        });
        if let Some(pair) = seen_payload_ranges
            .windows(2)
            .find(|pair| ranges_overlap(&pair[0].range, &pair[1].range))
        {
            let offending = if pair[0].resource_index > pair[1].resource_index {
                &pair[0]
            } else {
                &pair[1]
            };
            return Err(ErfError::new(
                PAYLOAD_OOB,
                offending.descriptor_offset,
                "resource payload overlaps another resource payload",
            ));
        }

        Ok(Self {
            bytes,
            file_type,
            resources,
            lookup,
        })
    }

    pub const fn file_type(&self) -> ErfFileType {
        self.file_type
    }

    /// Returns metadata in the archive's key-table order.
    pub fn resources(&self) -> &[ErfResource] {
        &self.resources
    }

    /// Finds a resource using ASCII case-insensitive resref matching.
    ///
    /// The returned slice borrows the original input buffer; payload bytes are not copied.
    pub fn find(&self, resref: &str, resource_type: u16) -> Result<&'a [u8], ErfError> {
        validate_query_resref(resref)?;
        let normalized_key = normalize_resource_key(resref.as_bytes(), resource_type);
        let resource = self
            .lookup
            .get(&normalized_key)
            .and_then(|index| self.resources.get(*index))
            .ok_or_else(|| {
                ErfError::new(
                    RESOURCE_MISSING,
                    0,
                    format!("resource ({resref}, {resource_type}) was not found"),
                )
            })?;

        Ok(&self.bytes[resource.offset..resource.offset + resource.size])
    }
}

fn validate_query_resref(resref: &str) -> Result<(), ErfError> {
    if resref.is_empty() || resref.len() > 16 || !resref.bytes().all(is_query_resref_byte) {
        return Err(ErfError::new(
            RESREF_INVALID,
            0,
            "lookup resref must match [A-Za-z0-9_-]{1,16}",
        ));
    }
    Ok(())
}

fn parse_resref(bytes: &[u8], offset: usize) -> Result<String, ErfError> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if end == 0 {
        return Err(ErfError::new(
            RESREF_INVALID,
            offset,
            "resource resref must not be empty",
        ));
    }
    if bytes[end..].iter().any(|byte| *byte != 0) {
        return Err(ErfError::new(
            RESREF_INVALID,
            offset + end,
            "non-NUL byte follows resref NUL terminator",
        ));
    }
    if !bytes[..end].iter().copied().all(is_stored_resref_byte) {
        return Err(ErfError::new(
            RESREF_INVALID,
            offset,
            "stored resource resref must match [A-Za-z0-9_-]{1,16}",
        ));
    }

    let mut resref = String::new();
    try_reserve_string(&mut resref, end, offset, "resource resref")?;
    for byte in &bytes[..end] {
        resref.push(char::from(*byte));
    }
    Ok(resref)
}

fn normalize_resource_key(bytes: &[u8], resource_type: u16) -> NormalizedResourceKey {
    let mut resref = [0; 16];
    for (output, input) in resref.iter_mut().zip(bytes.iter().copied()) {
        *output = input.to_ascii_lowercase();
    }
    NormalizedResourceKey {
        resref,
        resource_type,
    }
}

fn try_reserve_lookup(
    lookup: &mut HashMap<NormalizedResourceKey, usize>,
    additional: usize,
) -> Result<(), ErfError> {
    if additional == 0 {
        return Ok(());
    }
    allocation_gate(16, "duplicate-key set")?;
    lookup
        .try_reserve(additional)
        .map_err(|_| allocation_error(16, "duplicate-key set"))
}

fn try_reserve_vec<T>(
    values: &mut Vec<T>,
    additional: usize,
    offset: usize,
    context: &str,
) -> Result<(), ErfError> {
    if additional == 0 {
        return Ok(());
    }
    allocation_gate(offset, context)?;
    values
        .try_reserve_exact(additional)
        .map_err(|_| allocation_error(offset, context))
}

fn try_reserve_string(
    value: &mut String,
    additional: usize,
    offset: usize,
    context: &str,
) -> Result<(), ErfError> {
    if additional == 0 {
        return Ok(());
    }
    allocation_gate(offset, context)?;
    value
        .try_reserve_exact(additional)
        .map_err(|_| allocation_error(offset, context))
}

fn allocation_gate(_offset: usize, _context: &str) -> Result<(), ErfError> {
    #[cfg(test)]
    if FAIL_NEXT_PARSER_ALLOCATION.with(|flag| flag.replace(false)) {
        return Err(allocation_error(_offset, _context));
    }

    Ok(())
}

fn allocation_error(offset: usize, context: &str) -> ErfError {
    ErfError::new(
        LIMIT_EXCEEDED,
        offset,
        format!("unable to allocate parser-owned {context}"),
    )
}

fn is_stored_resref_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

fn is_query_resref_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

fn checked_table<'a>(
    bytes: &'a [u8],
    offset: usize,
    count: usize,
    entry_size: usize,
    code: &str,
    context: &str,
) -> Result<&'a [u8], ErfError> {
    if offset < HEADER_SIZE {
        return Err(ErfError::new(
            code,
            offset,
            format!("{context} starts inside the fixed header"),
        ));
    }
    let size = count.checked_mul(entry_size).ok_or_else(|| {
        ErfError::new(
            code,
            offset,
            format!("{context} count multiplication overflow"),
        )
    })?;
    checked_slice(bytes, offset, size, code, context)
}

fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    code: &str,
    context: &str,
) -> Result<&'a [u8], ErfError> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| ErfError::new(code, offset, format!("{context} range overflow")))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| ErfError::new(code, offset, format!("{context} is outside input")))
}

fn read_u32(bytes: &[u8], offset: usize, code: &str, context: &str) -> Result<u32, ErfError> {
    let field = checked_slice(bytes, offset, 4, code, context)?;
    Ok(u32::from_le_bytes(
        field.try_into().expect("checked four-byte field"),
    ))
}

fn ranges_overlap(left: &std::ops::Range<usize>, right: &std::ops::Range<usize>) -> bool {
    !left.is_empty() && !right.is_empty() && left.start < right.end && right.start < left.end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_empty_resource_hak() -> Vec<u8> {
        let mut bytes = vec![0; HEADER_SIZE + KEY_ENTRY_SIZE + RESOURCE_ENTRY_SIZE];
        bytes[0..4].copy_from_slice(b"HAK ");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[16..20].copy_from_slice(&1_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
        bytes[28..32].copy_from_slice(&((HEADER_SIZE + KEY_ENTRY_SIZE) as u32).to_le_bytes());
        bytes[HEADER_SIZE] = b'x';
        bytes[HEADER_SIZE + 20..HEADER_SIZE + 22].copy_from_slice(&3_u16.to_le_bytes());
        let resource_offset = HEADER_SIZE + KEY_ENTRY_SIZE;
        let byte_length = bytes.len() as u32;
        bytes[resource_offset..resource_offset + 4].copy_from_slice(&byte_length.to_le_bytes());
        bytes
    }

    fn many_resource_hak(entry_count: usize, overlap_last: bool) -> Vec<u8> {
        let key_table_offset = HEADER_SIZE;
        let resource_table_offset = key_table_offset + entry_count * KEY_ENTRY_SIZE;
        let payload_offset = resource_table_offset + entry_count * RESOURCE_ENTRY_SIZE;
        let mut bytes = vec![0; payload_offset + entry_count];
        bytes[0..4].copy_from_slice(b"HAK ");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[16..20].copy_from_slice(&(entry_count as u32).to_le_bytes());
        bytes[24..28].copy_from_slice(&(key_table_offset as u32).to_le_bytes());
        bytes[28..32].copy_from_slice(&(resource_table_offset as u32).to_le_bytes());

        for index in 0..entry_count {
            let key_offset = key_table_offset + index * KEY_ENTRY_SIZE;
            let resref = format!("r{index:015x}");
            bytes[key_offset..key_offset + 16].copy_from_slice(resref.as_bytes());
            bytes[key_offset + 16..key_offset + 20].copy_from_slice(&(index as u32).to_le_bytes());
            bytes[key_offset + 20..key_offset + 22].copy_from_slice(&3_u16.to_le_bytes());

            let descriptor_offset = resource_table_offset + index * RESOURCE_ENTRY_SIZE;
            let stored_payload_index = if overlap_last && index + 1 == entry_count {
                index - 1
            } else {
                index
            };
            bytes[descriptor_offset..descriptor_offset + 4]
                .copy_from_slice(&((payload_offset + stored_payload_index) as u32).to_le_bytes());
            bytes[descriptor_offset + 4..descriptor_offset + 8]
                .copy_from_slice(&1_u32.to_le_bytes());
            bytes[payload_offset + index] = (index & 0xff) as u8;
        }
        bytes
    }

    #[test]
    fn deterministic_parser_allocation_failure_is_reported_without_panicking() {
        let bytes = one_empty_resource_hak();
        let result = std::panic::catch_unwind(|| {
            FAIL_NEXT_PARSER_ALLOCATION.with(|flag| flag.set(true));
            ErfArchive::parse(&bytes)
        });

        let error = result
            .expect("allocation failure seam panicked")
            .unwrap_err();
        assert_eq!(error.code, LIMIT_EXCEEDED);
        assert_eq!(error.offset, 16);
        assert!(error.context.contains("duplicate-key set"));
        assert!(ErfArchive::parse(&bytes).is_ok());
    }

    #[test]
    fn indexed_lookup_and_sorted_ranges_scale_to_large_synthetic_archives() {
        const ENTRY_COUNT: usize = 4_096;
        let bytes = many_resource_hak(ENTRY_COUNT, false);
        let archive = ErfArchive::parse(&bytes).unwrap();
        assert_eq!(archive.resources().len(), ENTRY_COUNT);
        assert_eq!(archive.lookup.len(), ENTRY_COUNT);
        assert_eq!(archive.find("R000000000000fff", 3).unwrap(), &[0xff]);

        let overlap = many_resource_hak(ENTRY_COUNT, true);
        let error = ErfArchive::parse(&overlap).unwrap_err();
        assert_eq!(error.code, PAYLOAD_OOB);
        assert_eq!(
            error.offset,
            HEADER_SIZE + ENTRY_COUNT * KEY_ENTRY_SIZE + (ENTRY_COUNT - 1) * RESOURCE_ENTRY_SIZE
        );
    }
}
