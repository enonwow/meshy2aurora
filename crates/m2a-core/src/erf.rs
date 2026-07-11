use std::collections::HashSet;
use std::fmt;

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

        let mut metadata_ranges = vec![0..HEADER_SIZE, key_range, resource_range];
        if let Some(localized_range) = localized_range {
            metadata_ranges.push(localized_range);
        }

        let mut seen_keys = HashSet::with_capacity(entry_count_usize);
        let mut seen_payload_ranges = Vec::with_capacity(entry_count_usize);
        let mut resources = Vec::with_capacity(entry_count_usize);

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

            let normalized_key = (resref.to_ascii_lowercase(), resource_type);
            if !seen_keys.insert(normalized_key) {
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
                && metadata_ranges
                    .iter()
                    .any(|metadata| ranges_overlap(&payload_range, metadata))
            {
                return Err(ErfError::new(
                    PAYLOAD_OOB,
                    resource_offset,
                    "resource payload overlaps archive metadata",
                ));
            }
            if payload_size != 0
                && seen_payload_ranges
                    .iter()
                    .any(|previous| ranges_overlap(&payload_range, previous))
            {
                return Err(ErfError::new(
                    PAYLOAD_OOB,
                    resource_offset,
                    "resource payload overlaps another resource payload",
                ));
            }
            if payload_size != 0 {
                seen_payload_ranges.push(payload_range);
            }

            resources.push(ErfResource {
                resref,
                resource_id,
                resource_type,
                offset: payload_offset,
                size: payload_size,
            });
        }

        Ok(Self {
            bytes,
            file_type,
            resources,
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
        let resource = self
            .resources
            .iter()
            .find(|resource| {
                resource.resource_type == resource_type
                    && resource.resref.eq_ignore_ascii_case(resref)
            })
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

    String::from_utf8(bytes[..end].to_vec())
        .map_err(|_| ErfError::new(RESREF_INVALID, offset, "resource resref is not valid ASCII"))
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
