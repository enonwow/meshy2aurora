#![allow(clippy::result_large_err)]

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const TWO_DA_SCHEMA_VERSION: u32 = 1;
pub const TWO_DA_MAX_INPUT_BYTES: u64 = 16_777_216;
pub const TWO_DA_MAX_COLUMNS: u32 = 4_096;
pub const TWO_DA_MAX_ROWS: u32 = 65_536;
pub const TWO_DA_MAX_TOKEN_BYTES: u32 = 1_048_576;
pub const TWO_DA_MAX_DIAGNOSTICS: u32 = 2_048;

pub const SCHEMA_INVALID: &str = "M5-2DA-SCHEMA-INVALID";
pub const LIMIT_EXCEEDED: &str = "M5-2DA-LIMIT-EXCEEDED";
pub const HEADER_INVALID: &str = "M5-2DA-HEADER-INVALID";
pub const ENCODING_UNSUPPORTED: &str = "M5-2DA-ENCODING-UNSUPPORTED";
pub const NEWLINE_INVALID: &str = "M5-2DA-NEWLINE-INVALID";
pub const TAB_FORBIDDEN: &str = "M5-2DA-TAB-FORBIDDEN";
pub const NUL_FORBIDDEN: &str = "M5-2DA-NUL-FORBIDDEN";
pub const DEFAULT_INVALID: &str = "M5-2DA-DEFAULT-INVALID";
pub const COLUMN_INVALID: &str = "M5-2DA-COLUMN-INVALID";
pub const COLUMN_AMBIGUOUS: &str = "M5-2DA-COLUMN-AMBIGUOUS";
pub const ROW_LABEL_INVALID: &str = "M5-2DA-ROW-LABEL-INVALID";
pub const ROW_ARITY_INVALID: &str = "M5-2DA-ROW-ARITY-INVALID";
pub const QUOTE_INVALID: &str = "M5-2DA-QUOTE-INVALID";
pub const ASSIGNMENT_DUPLICATE: &str = "M5-2DA-ASSIGNMENT-DUPLICATE";
pub const ASSIGNMENT_COLUMN_MISSING: &str = "M5-2DA-ASSIGNMENT-COLUMN-MISSING";
pub const VALUE_INVALID: &str = "M5-2DA-VALUE-INVALID";
pub const APPEND_U16_OVERFLOW: &str = "M5-2DA-APPEND-U16-OVERFLOW";
pub const LAYOUT_OVERFLOW: &str = "M5-2DA-LAYOUT-OVERFLOW";
pub const READBACK_FAILED: &str = "M5-2DA-READBACK-FAILED";
pub const SEMANTIC_DIFF: &str = "M5-2DA-SEMANTIC-DIFF";
pub const ROW_LABEL_MISMATCH: &str = "M5-2DA-ROW-LABEL-MISMATCH";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TwoDaNewlineV1 {
    CrLf,
    Lf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum TwoDaCellValueV1 {
    Null,
    Text { value: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TwoDaCellAssignmentV1 {
    pub column_name: String,
    pub value: TwoDaCellValueV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TwoDaAppendRequestV1 {
    pub schema_version: u32,
    pub cells: Vec<TwoDaCellAssignmentV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TwoDaLimitsV1 {
    pub max_input_bytes: u64,
    pub max_columns: u32,
    pub max_rows: u32,
    pub max_token_bytes: u32,
    pub max_diagnostics: u32,
}

impl Default for TwoDaLimitsV1 {
    fn default() -> Self {
        Self {
            max_input_bytes: TWO_DA_MAX_INPUT_BYTES,
            max_columns: TWO_DA_MAX_COLUMNS,
            max_rows: TWO_DA_MAX_ROWS,
            max_token_bytes: TWO_DA_MAX_TOKEN_BYTES,
            max_diagnostics: TWO_DA_MAX_DIAGNOSTICS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoDaDiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub line: Option<u32>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoDaInspectionV1 {
    pub schema_version: u32,
    pub format: String,
    pub version: String,
    pub source_sha256: String,
    pub byte_length: u64,
    pub newline: TwoDaNewlineV1,
    pub terminal_newline: bool,
    pub default_value: Option<TwoDaCellValueV1>,
    pub columns: Vec<String>,
    pub physical_row_count: u32,
    pub next_append_index: Option<u16>,
    pub row_label_mismatch_count: u32,
    pub diagnostics: Vec<TwoDaDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoDaChangedCellV1 {
    pub column_index: u32,
    pub column_name: String,
    pub value: TwoDaCellValueV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoDaAppendReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub output_sha256: String,
    pub source_byte_length: u64,
    pub output_byte_length: u64,
    pub source_prefix_preserved: bool,
    pub appended_row_index: u16,
    pub physical_rows_before: u32,
    pub physical_rows_after: u32,
    pub newline: TwoDaNewlineV1,
    pub inserted_separator_newline: bool,
    pub changed_cells: Vec<TwoDaChangedCellV1>,
    pub diagnostics: Vec<TwoDaDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwoDaAppendArtifactV1 {
    pub payload: Vec<u8>,
    pub report: TwoDaAppendReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoDaError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub byte_offset: u64,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub path: String,
    pub message: String,
}

impl fmt::Display for TwoDaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TwoDaError {}

impl TwoDaError {
    fn fatal(
        code: &str,
        path: &str,
        byte_offset: u64,
        line: Option<u32>,
        column: Option<u32>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: TWO_DA_SCHEMA_VERSION,
            code: code.to_owned(),
            severity: "FATAL".to_owned(),
            byte_offset,
            line,
            column,
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LineSpan<'a> {
    bytes: &'a [u8],
    byte_offset: u64,
    line: u32,
    column_offset: u32,
}

type ScanResult<'a> = (
    Option<TwoDaNewlineV1>,
    bool,
    Vec<LineSpan<'a>>,
    Option<TwoDaError>,
);

fn validate_limits(limits: &TwoDaLimitsV1) -> Result<(), TwoDaError> {
    validate_limit(
        limits.max_input_bytes,
        TWO_DA_MAX_INPUT_BYTES,
        "limits.maxInputBytes",
    )?;
    validate_limit(
        u64::from(limits.max_columns),
        u64::from(TWO_DA_MAX_COLUMNS),
        "limits.maxColumns",
    )?;
    validate_limit(
        u64::from(limits.max_rows),
        u64::from(TWO_DA_MAX_ROWS),
        "limits.maxRows",
    )?;
    validate_limit(
        u64::from(limits.max_token_bytes),
        u64::from(TWO_DA_MAX_TOKEN_BYTES),
        "limits.maxTokenBytes",
    )?;
    validate_limit(
        u64::from(limits.max_diagnostics),
        u64::from(TWO_DA_MAX_DIAGNOSTICS),
        "limits.maxDiagnostics",
    )?;
    Ok(())
}

fn validate_limit(value: u64, hard_cap: u64, path: &str) -> Result<(), TwoDaError> {
    if value == 0 || value > hard_cap {
        return Err(TwoDaError::fatal(
            SCHEMA_INVALID,
            path,
            0,
            None,
            None,
            format!("limit must be in 1..={hard_cap}, got {value}"),
        ));
    }
    Ok(())
}

fn scan_input<'a>(bytes: &'a [u8], limits: &TwoDaLimitsV1) -> Result<ScanResult<'a>, TwoDaError> {
    validate_limits(limits)?;
    scan_input_bounded(bytes, limits.max_input_bytes, limits.max_rows)
}

fn scan_input_bounded(
    bytes: &[u8],
    max_input_bytes: u64,
    max_rows: u32,
) -> Result<ScanResult<'_>, TwoDaError> {
    let byte_length = u64::try_from(bytes.len()).map_err(|_| {
        TwoDaError::fatal(
            LIMIT_EXCEEDED,
            "input",
            0,
            None,
            None,
            "input byte length cannot be represented as u64",
        )
    })?;
    if byte_length > max_input_bytes {
        return Err(TwoDaError::fatal(
            LIMIT_EXCEEDED,
            "input",
            max_input_bytes,
            None,
            None,
            format!("input byte length {byte_length} exceeds configured limit {max_input_bytes}"),
        ));
    }

    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(TwoDaError::fatal(
            ENCODING_UNSUPPORTED,
            "input",
            0,
            Some(1),
            Some(1),
            "UTF-8 BOM is not allowed",
        ));
    }

    let max_lines = max_rows
        .checked_add(3)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| parser_layout_error("line span limit cannot be represented as usize"))?;
    let mut newline = None;
    let mut lines = Vec::new();
    let mut line_limit_error = None;
    let mut line_start = 0usize;
    let mut line_number = 1u32;
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            0 => {
                return Err(scan_error(
                    NUL_FORBIDDEN,
                    index,
                    line_start,
                    line_number,
                    "NUL byte is not allowed",
                ));
            }
            b'\t' => {
                return Err(scan_error(
                    TAB_FORBIDDEN,
                    index,
                    line_start,
                    line_number,
                    "TAB byte is not allowed",
                ));
            }
            b'\r' => {
                if bytes.get(index + 1) != Some(&b'\n') {
                    return Err(scan_error(
                        NEWLINE_INVALID,
                        index,
                        line_start,
                        line_number,
                        "bare CR is not allowed",
                    ));
                }
                register_newline(
                    &mut newline,
                    TwoDaNewlineV1::CrLf,
                    index,
                    line_start,
                    line_number,
                )?;
                push_scanned_line(
                    &mut lines,
                    &mut line_limit_error,
                    max_lines,
                    max_rows,
                    LineSpan {
                        bytes: &bytes[line_start..index],
                        byte_offset: line_start as u64,
                        line: line_number,
                        column_offset: 0,
                    },
                )?;
                index += 2;
                line_start = index;
                line_number += 1;
            }
            b'\n' => {
                register_newline(
                    &mut newline,
                    TwoDaNewlineV1::Lf,
                    index,
                    line_start,
                    line_number,
                )?;
                push_scanned_line(
                    &mut lines,
                    &mut line_limit_error,
                    max_lines,
                    max_rows,
                    LineSpan {
                        bytes: &bytes[line_start..index],
                        byte_offset: line_start as u64,
                        line: line_number,
                        column_offset: 0,
                    },
                )?;
                index += 1;
                line_start = index;
                line_number += 1;
            }
            0x20..=0x7e => {
                index += 1;
            }
            _ => {
                return Err(scan_error(
                    ENCODING_UNSUPPORTED,
                    index,
                    line_start,
                    line_number,
                    "only printable ASCII and uniform line endings are allowed",
                ));
            }
        }
    }

    let terminal_newline = line_start == bytes.len() && newline.is_some();
    if line_start < bytes.len() {
        push_scanned_line(
            &mut lines,
            &mut line_limit_error,
            max_lines,
            max_rows,
            LineSpan {
                bytes: &bytes[line_start..],
                byte_offset: line_start as u64,
                line: line_number,
                column_offset: 0,
            },
        )?;
    }

    Ok((newline, terminal_newline, lines, line_limit_error))
}

fn push_scanned_line<'a>(
    lines: &mut Vec<LineSpan<'a>>,
    line_limit_error: &mut Option<TwoDaError>,
    max_lines: usize,
    max_rows: u32,
    line: LineSpan<'a>,
) -> Result<(), TwoDaError> {
    if line_limit_error.is_some() {
        return Ok(());
    }
    if lines.len() >= max_lines {
        *line_limit_error = Some(line_error(
            LIMIT_EXCEEDED,
            "limits.maxRows",
            line,
            0,
            format!("row count exceeds configured limit {max_rows}"),
        ));
        return Ok(());
    }
    try_reserve_bounded(lines, max_lines, "line span")?;
    lines.push(line);
    Ok(())
}

fn register_newline(
    detected: &mut Option<TwoDaNewlineV1>,
    current: TwoDaNewlineV1,
    byte_offset: usize,
    line_start: usize,
    line_number: u32,
) -> Result<(), TwoDaError> {
    if let Some(previous) = detected {
        if *previous != current {
            return Err(scan_error(
                NEWLINE_INVALID,
                byte_offset,
                line_start,
                line_number,
                "mixed LF and CRLF line endings are not allowed",
            ));
        }
    } else {
        *detected = Some(current);
    }
    Ok(())
}

fn scan_error(
    code: &str,
    byte_offset: usize,
    line_start: usize,
    line_number: u32,
    message: &str,
) -> TwoDaError {
    TwoDaError::fatal(
        code,
        "input",
        byte_offset as u64,
        Some(line_number),
        Some((byte_offset - line_start + 1) as u32),
        message,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TokenSpan<'a> {
    bytes: &'a [u8],
    quoted: bool,
    byte_offset: usize,
}

#[derive(Clone, Copy)]
struct TokenCountLimit<'a> {
    max_tokens: usize,
    code: &'a str,
    path: &'a str,
    message: &'a str,
}

fn parse_tokens<'a>(
    line: LineSpan<'a>,
    limits: &TwoDaLimitsV1,
    path: &str,
    count_limit: TokenCountLimit<'_>,
) -> Result<Vec<TokenSpan<'a>>, TwoDaError> {
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < line.bytes.len() {
        while line.bytes.get(index) == Some(&b' ') {
            index += 1;
        }
        if index == line.bytes.len() {
            break;
        }

        let token_start = index;
        let (content_start, content_end, quoted);
        if line.bytes[index] == b'"' {
            quoted = true;
            index += 1;
            content_start = index;
            while index < line.bytes.len() && line.bytes[index] != b'"' {
                index += 1;
            }
            if index == line.bytes.len() {
                return Err(line_error(
                    QUOTE_INVALID,
                    path,
                    line,
                    token_start,
                    "quoted token is missing its closing quote",
                ));
            }
            content_end = index;
            index += 1;
            if index < line.bytes.len() && line.bytes[index] != b' ' {
                return Err(line_error(
                    QUOTE_INVALID,
                    path,
                    line,
                    index,
                    "closing quote must be followed by a separator or end of line",
                ));
            }
        } else {
            quoted = false;
            content_start = index;
            while index < line.bytes.len() && line.bytes[index] != b' ' {
                if line.bytes[index] == b'"' {
                    return Err(line_error(
                        QUOTE_INVALID,
                        path,
                        line,
                        index,
                        "quote may only appear at the start of a quoted token",
                    ));
                }
                index += 1;
            }
            content_end = index;
        }

        let lexical_length = index - token_start;
        if lexical_length > limits.max_token_bytes as usize {
            return Err(line_error(
                LIMIT_EXCEEDED,
                "limits.maxTokenBytes",
                line,
                token_start,
                format!(
                    "token byte length {lexical_length} exceeds configured limit {}",
                    limits.max_token_bytes
                ),
            ));
        }
        if tokens.len() >= count_limit.max_tokens {
            return Err(line_error(
                count_limit.code,
                count_limit.path,
                line,
                token_start,
                count_limit.message,
            ));
        }
        try_reserve_bounded(&mut tokens, count_limit.max_tokens, "token span")?;
        tokens.push(TokenSpan {
            bytes: &line.bytes[content_start..content_end],
            quoted,
            byte_offset: token_start,
        });
    }

    Ok(tokens)
}

fn try_reserve_bounded<T>(
    values: &mut Vec<T>,
    max_len: usize,
    allocation_name: &str,
) -> Result<(), TwoDaError> {
    if values.len() < values.capacity() {
        return Ok(());
    }
    let remaining = max_len
        .checked_sub(values.len())
        .ok_or_else(|| parser_layout_error("bounded vector length exceeded its checked limit"))?;
    if remaining == 0 {
        return Err(parser_layout_error(format!(
            "{allocation_name} buffer reached its checked limit"
        )));
    }
    let additional = values.capacity().max(8).min(remaining);
    values
        .try_reserve_exact(additional)
        .map_err(|_| parser_layout_error(format!("unable to reserve {allocation_name} buffer")))
}

fn parser_layout_error(message: impl Into<String>) -> TwoDaError {
    TwoDaError::fatal(LAYOUT_OVERFLOW, "input", 0, None, None, message)
}

fn line_error(
    code: &str,
    path: &str,
    line: LineSpan<'_>,
    byte_offset: usize,
    message: impl Into<String>,
) -> TwoDaError {
    TwoDaError::fatal(
        code,
        path,
        line.byte_offset + byte_offset as u64,
        Some(line.line),
        Some(line.column_offset + (byte_offset + 1) as u32),
        message,
    )
}

fn parse_default(
    line: LineSpan<'_>,
    limits: &TwoDaLimitsV1,
) -> Result<Option<TwoDaCellValueV1>, TwoDaError> {
    if line.bytes.iter().all(|byte| *byte == b' ') {
        return Ok(None);
    }

    const PREFIX: &[u8] = b"DEFAULT:";
    if !line.bytes.starts_with(PREFIX)
        || line.bytes.len() == PREFIX.len()
        || line.bytes[PREFIX.len()] != b' '
    {
        return Err(line_error(
            DEFAULT_INVALID,
            "default",
            line,
            0,
            "line 2 must be blank or contain DEFAULT: followed by one token",
        ));
    }

    let remainder = LineSpan {
        bytes: &line.bytes[PREFIX.len()..],
        byte_offset: line.byte_offset + PREFIX.len() as u64,
        line: line.line,
        column_offset: line.column_offset + PREFIX.len() as u32,
    };
    let tokens = parse_tokens(
        remainder,
        limits,
        "default",
        TokenCountLimit {
            max_tokens: 1,
            code: DEFAULT_INVALID,
            path: "default",
            message: "DEFAULT: must contain exactly one token",
        },
    )?;
    if tokens.len() != 1 {
        return Err(line_error(
            DEFAULT_INVALID,
            "default",
            line,
            0,
            "DEFAULT: must contain exactly one token",
        ));
    }

    Ok(Some(token_to_cell(tokens[0])))
}

struct ParsedColumns {
    names: Vec<String>,
    byte_offsets: Vec<usize>,
}

fn parse_columns(line: LineSpan<'_>, limits: &TwoDaLimitsV1) -> Result<ParsedColumns, TwoDaError> {
    let tokens = parse_tokens(
        line,
        limits,
        "columns",
        TokenCountLimit {
            max_tokens: limits.max_columns as usize,
            code: LIMIT_EXCEEDED,
            path: "limits.maxColumns",
            message: "column count exceeds configured limit",
        },
    )?;
    if tokens.is_empty() {
        return Err(line_error(
            COLUMN_INVALID,
            "columns",
            line,
            0,
            "line 3 must contain at least one column name",
        ));
    }
    let column_count = tokens.len();
    let mut columns = Vec::new();
    let mut byte_offsets = Vec::new();
    for (index, token) in tokens.into_iter().enumerate() {
        if token.quoted
            || token.bytes.is_empty()
            || !token
                .bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            return Err(line_error(
                COLUMN_INVALID,
                &format!("columns[{index}]"),
                line,
                token.byte_offset,
                "column name must match [A-Za-z0-9_]+",
            ));
        }

        let column = ascii_string(token.bytes);
        try_reserve_bounded(&mut columns, column_count, "column")?;
        try_reserve_bounded(&mut byte_offsets, column_count, "column offset")?;
        columns.push(column);
        byte_offsets.push(token.byte_offset);
    }

    Ok(ParsedColumns {
        names: columns,
        byte_offsets,
    })
}

fn validate_column_uniqueness(
    columns: &ParsedColumns,
    line: LineSpan<'_>,
) -> Result<(), TwoDaError> {
    for index in 0..columns.names.len() {
        if columns.names[..index]
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&columns.names[index]))
        {
            return Err(line_error(
                COLUMN_AMBIGUOUS,
                &format!("columns[{index}]"),
                line,
                columns.byte_offsets[index],
                "column name collides after ASCII case-folding",
            ));
        }
    }
    Ok(())
}

fn token_to_cell(token: TokenSpan<'_>) -> TwoDaCellValueV1 {
    if !token.quoted && token.bytes == b"****" {
        TwoDaCellValueV1::Null
    } else {
        TwoDaCellValueV1::Text {
            value: ascii_string(token.bytes),
        }
    }
}

fn ascii_string(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn inspect_two_da_v2(
    bytes: &[u8],
    limits: &TwoDaLimitsV1,
) -> Result<TwoDaInspectionV1, TwoDaError> {
    let (newline, terminal_newline, lines, line_limit_error) = scan_input(bytes, limits)?;
    inspect_scanned(
        bytes,
        limits,
        newline,
        terminal_newline,
        lines.as_slice(),
        line_limit_error,
    )
}

fn inspect_scanned(
    bytes: &[u8],
    limits: &TwoDaLimitsV1,
    newline: Option<TwoDaNewlineV1>,
    terminal_newline: bool,
    lines: &[LineSpan<'_>],
    line_limit_error: Option<TwoDaError>,
) -> Result<TwoDaInspectionV1, TwoDaError> {
    let header = lines.first().copied().ok_or_else(|| {
        TwoDaError::fatal(
            HEADER_INVALID,
            "header",
            0,
            Some(1),
            Some(1),
            "line 1 must be exactly 2DA V2.0",
        )
    })?;
    if header.bytes != b"2DA V2.0" {
        return Err(line_error(
            HEADER_INVALID,
            "header",
            header,
            0,
            "line 1 must be exactly 2DA V2.0",
        ));
    }

    let default_line = lines.get(1).copied().ok_or_else(|| {
        TwoDaError::fatal(
            DEFAULT_INVALID,
            "default",
            bytes.len() as u64,
            Some(2),
            Some(1),
            "missing DEFAULT line",
        )
    })?;
    let default_value = parse_default(default_line, limits)?;

    let columns_line = lines.get(2).copied().ok_or_else(|| {
        TwoDaError::fatal(
            COLUMN_INVALID,
            "columns",
            bytes.len() as u64,
            Some(3),
            Some(1),
            "missing columns line",
        )
    })?;
    let parsed_columns = parse_columns(columns_line, limits)?;
    let expected_arity = parsed_columns.names.len() + 1;

    let mut physical_row_count = 0u32;
    let mut row_label_mismatch_count = 0u32;
    let mut diagnostics = Vec::new();
    for line in lines.iter().copied().skip(3) {
        if physical_row_count >= limits.max_rows {
            return Err(line_error(
                LIMIT_EXCEEDED,
                "limits.maxRows",
                line,
                0,
                format!("row count exceeds configured limit {}", limits.max_rows),
            ));
        }

        let row_path = format!("rows[{physical_row_count}]");
        let tokens = parse_tokens(
            line,
            limits,
            &row_path,
            TokenCountLimit {
                max_tokens: expected_arity,
                code: ROW_ARITY_INVALID,
                path: &row_path,
                message: "row has more tokens than its full width",
            },
        )?;
        if tokens.len() != expected_arity {
            return Err(line_error(
                ROW_ARITY_INVALID,
                &format!("rows[{physical_row_count}]"),
                line,
                0,
                format!(
                    "row has {} tokens but expected {expected_arity}",
                    tokens.len()
                ),
            ));
        }

        let label_token = tokens[0];
        let label = if !label_token.quoted
            && !label_token.bytes.is_empty()
            && label_token.bytes.iter().all(u8::is_ascii_digit)
        {
            std::str::from_utf8(label_token.bytes)
                .ok()
                .and_then(|value| value.parse::<u32>().ok())
        } else {
            None
        }
        .ok_or_else(|| {
            line_error(
                ROW_LABEL_INVALID,
                &format!("rows[{physical_row_count}].label"),
                line,
                label_token.byte_offset,
                "row label must be a non-negative decimal u32",
            )
        })?;

        if label != physical_row_count {
            row_label_mismatch_count += 1;
            if diagnostics.len() < limits.max_diagnostics as usize {
                diagnostics.push(TwoDaDiagnosticV1 {
                    schema_version: TWO_DA_SCHEMA_VERSION,
                    code: ROW_LABEL_MISMATCH.to_owned(),
                    severity: "WARNING".to_owned(),
                    path: format!("rows[{physical_row_count}].label"),
                    line: Some(line.line),
                    message: format!(
                        "printed row label {label} does not match physical row index {physical_row_count}"
                    ),
                });
            }
        }
        physical_row_count += 1;
    }
    if let Some(error) = line_limit_error {
        return Err(error);
    }
    validate_column_uniqueness(&parsed_columns, columns_line)?;
    let columns = parsed_columns.names;

    let newline = newline.ok_or_else(|| {
        TwoDaError::fatal(
            HEADER_INVALID,
            "header",
            0,
            Some(1),
            Some(1),
            "2DA input must contain LF or CRLF line endings",
        )
    })?;

    Ok(TwoDaInspectionV1 {
        schema_version: TWO_DA_SCHEMA_VERSION,
        format: "2DA".to_owned(),
        version: "V2.0".to_owned(),
        source_sha256: sha256_hex(bytes),
        byte_length: bytes.len() as u64,
        newline,
        terminal_newline,
        default_value,
        columns,
        physical_row_count,
        next_append_index: u16::try_from(physical_row_count).ok(),
        row_label_mismatch_count,
        diagnostics,
    })
}

pub fn append_two_da_row_v1(
    bytes: &[u8],
    request: &TwoDaAppendRequestV1,
    limits: &TwoDaLimitsV1,
) -> Result<TwoDaAppendArtifactV1, TwoDaError> {
    validate_limits(limits)?;
    if request.schema_version != TWO_DA_SCHEMA_VERSION {
        return Err(TwoDaError::fatal(
            SCHEMA_INVALID,
            "request.schemaVersion",
            0,
            None,
            None,
            format!(
                "schema version must be {TWO_DA_SCHEMA_VERSION}, got {}",
                request.schema_version
            ),
        ));
    }

    let source = inspect_two_da_v2(bytes, limits)?;
    let mut resolved = vec![None; source.columns.len()];
    for (request_index, assignment) in request.cells.iter().enumerate() {
        let column_index = source
            .columns
            .iter()
            .position(|column| column.eq_ignore_ascii_case(&assignment.column_name))
            .ok_or_else(|| {
                TwoDaError::fatal(
                    ASSIGNMENT_COLUMN_MISSING,
                    &format!("request.cells[{request_index}].columnName"),
                    0,
                    None,
                    None,
                    format!("column {:?} does not exist", assignment.column_name),
                )
            })?;
        if resolved[column_index].is_some() {
            return Err(TwoDaError::fatal(
                ASSIGNMENT_DUPLICATE,
                &format!("request.cells[{request_index}]"),
                0,
                None,
                None,
                format!(
                    "column {:?} is assigned more than once",
                    source.columns[column_index]
                ),
            ));
        }
        validate_generated_value(&assignment.value, limits, request_index)?;
        resolved[column_index] = Some(&assignment.value);
    }

    let appended_row_index = u16::try_from(source.physical_row_count).map_err(|_| {
        TwoDaError::fatal(
            APPEND_U16_OVERFLOW,
            "rows",
            source.byte_length,
            None,
            None,
            format!(
                "physical append index {} exceeds u16::MAX",
                source.physical_row_count
            ),
        )
    })?;
    let physical_rows_after = source
        .physical_row_count
        .checked_add(1)
        .ok_or_else(|| layout_error("physical row count overflow while planning append"))?;
    let newline_bytes = newline_bytes(source.newline);
    let inserted_separator_newline = !source.terminal_newline;
    let label = source.physical_row_count.to_string();

    let mut suffix_length = if inserted_separator_newline {
        newline_bytes.len() as u64
    } else {
        0
    };
    suffix_length = checked_layout_add(suffix_length, label.len() as u64)?;
    for value in &resolved {
        suffix_length = checked_layout_add(suffix_length, 1)?;
        suffix_length = checked_layout_add(suffix_length, encoded_cell_length(*value)?)?;
    }
    suffix_length = checked_layout_add(suffix_length, newline_bytes.len() as u64)?;
    let output_byte_length = checked_layout_add(source.byte_length, suffix_length)?;
    let output_length = usize::try_from(output_byte_length)
        .map_err(|_| layout_error("output byte length cannot be represented as usize"))?;

    let mut payload = Vec::new();
    payload
        .try_reserve_exact(output_length)
        .map_err(|_| layout_error("unable to reserve output buffer"))?;
    payload.extend_from_slice(bytes);
    if inserted_separator_newline {
        payload.extend_from_slice(newline_bytes);
    }
    payload.extend_from_slice(label.as_bytes());
    for value in &resolved {
        payload.push(b' ');
        emit_cell(&mut payload, *value);
    }
    payload.extend_from_slice(newline_bytes);
    if payload.len() != output_length {
        return Err(layout_error(
            "emitted output length differs from checked plan",
        ));
    }

    let output_inspection = verify_append_readback(bytes, &payload, &source, &resolved, limits)?;
    let changed_cells = resolved
        .iter()
        .enumerate()
        .filter_map(|(column_index, value)| {
            value.map(|value| TwoDaChangedCellV1 {
                column_index: column_index as u32,
                column_name: source.columns[column_index].clone(),
                value: value.clone(),
            })
        })
        .collect();

    Ok(TwoDaAppendArtifactV1 {
        payload,
        report: TwoDaAppendReportV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            source_sha256: source.source_sha256.clone(),
            output_sha256: output_inspection.source_sha256,
            source_byte_length: source.byte_length,
            output_byte_length,
            source_prefix_preserved: true,
            appended_row_index,
            physical_rows_before: source.physical_row_count,
            physical_rows_after,
            newline: source.newline,
            inserted_separator_newline,
            changed_cells,
            diagnostics: source.diagnostics,
        },
    })
}

fn validate_generated_value(
    value: &TwoDaCellValueV1,
    limits: &TwoDaLimitsV1,
    request_index: usize,
) -> Result<(), TwoDaError> {
    if let TwoDaCellValueV1::Text { value } = value
        && !value
            .as_bytes()
            .iter()
            .all(|byte| (0x20..=0x7e).contains(byte) && *byte != b'"')
    {
        return Err(TwoDaError::fatal(
            VALUE_INVALID,
            &format!("request.cells[{request_index}].value.value"),
            0,
            None,
            None,
            "TEXT must contain only printable ASCII and may not contain a quote",
        ));
    }

    let lexical_length = encoded_cell_length(Some(value))?;
    if lexical_length > u64::from(limits.max_token_bytes) {
        return Err(TwoDaError::fatal(
            LIMIT_EXCEEDED,
            "limits.maxTokenBytes",
            0,
            None,
            None,
            format!(
                "encoded value byte length {lexical_length} exceeds configured limit {}",
                limits.max_token_bytes
            ),
        ));
    }
    Ok(())
}

fn encoded_cell_length(value: Option<&TwoDaCellValueV1>) -> Result<u64, TwoDaError> {
    match value {
        None | Some(TwoDaCellValueV1::Null) => Ok(4),
        Some(TwoDaCellValueV1::Text { value }) => {
            let length = u64::try_from(value.len())
                .map_err(|_| layout_error("TEXT length cannot be represented as u64"))?;
            if value.is_empty() || value == "****" || value.as_bytes().contains(&b' ') {
                checked_layout_add(length, 2)
            } else {
                Ok(length)
            }
        }
    }
}

fn emit_cell(output: &mut Vec<u8>, value: Option<&TwoDaCellValueV1>) {
    match value {
        None | Some(TwoDaCellValueV1::Null) => output.extend_from_slice(b"****"),
        Some(TwoDaCellValueV1::Text { value }) => {
            let quoted = value.is_empty() || value == "****" || value.as_bytes().contains(&b' ');
            if quoted {
                output.push(b'"');
            }
            output.extend_from_slice(value.as_bytes());
            if quoted {
                output.push(b'"');
            }
        }
    }
}

fn newline_bytes(newline: TwoDaNewlineV1) -> &'static [u8] {
    match newline {
        TwoDaNewlineV1::CrLf => b"\r\n",
        TwoDaNewlineV1::Lf => b"\n",
    }
}

fn checked_layout_add(left: u64, right: u64) -> Result<u64, TwoDaError> {
    left.checked_add(right)
        .ok_or_else(|| layout_error("output length overflow"))
}

fn layout_error(message: impl Into<String>) -> TwoDaError {
    TwoDaError::fatal(LAYOUT_OVERFLOW, "output", 0, None, None, message)
}

fn verify_append_readback(
    source_bytes: &[u8],
    output: &[u8],
    source: &TwoDaInspectionV1,
    expected_cells: &[Option<&TwoDaCellValueV1>],
    limits: &TwoDaLimitsV1,
) -> Result<TwoDaInspectionV1, TwoDaError> {
    let output_length = u64::try_from(output.len())
        .map_err(|_| readback_error("output length cannot be represented as u64"))?;
    let rows_after = source
        .physical_row_count
        .checked_add(1)
        .ok_or_else(|| semantic_error("physical row count did not advance by one"))?;
    let mut readback_limits = *limits;
    readback_limits.max_rows = readback_limits.max_rows.max(rows_after);
    let (newline, terminal_newline, lines, line_limit_error) =
        scan_input_bounded(output, output_length, readback_limits.max_rows)
            .map_err(|error| readback_error(format!("output scan failed: {}", error.code)))?;
    let output_inspection = inspect_scanned(
        output,
        &readback_limits,
        newline,
        terminal_newline,
        lines.as_slice(),
        line_limit_error,
    )
    .map_err(|error| readback_error(format!("output parse failed: {}", error.code)))?;

    let expected_next_append_index = u16::try_from(rows_after).ok();
    if !output.starts_with(source_bytes)
        || output_inspection.format != source.format
        || output_inspection.version != source.version
        || output_inspection.newline != source.newline
        || output_inspection.default_value != source.default_value
        || output_inspection.columns != source.columns
        || output_inspection.physical_row_count != rows_after
        || output_inspection.next_append_index != expected_next_append_index
        || output_inspection.row_label_mismatch_count != source.row_label_mismatch_count
        || output_inspection.diagnostics != source.diagnostics
        || output_inspection.byte_length != output_length
        || !output_inspection.terminal_newline
    {
        return Err(semantic_error(
            "output differs from append-only source invariants",
        ));
    }

    let expected_label = source.physical_row_count.to_string();
    if !suffix_matches_canonical_plan(
        &output[source_bytes.len()..],
        source,
        expected_label.as_bytes(),
        expected_cells,
    ) {
        return Err(semantic_error(
            "emitted suffix bytes differ from the canonical append plan",
        ));
    }

    let appended_line = lines
        .last()
        .copied()
        .ok_or_else(|| readback_error("output has no appended row"))?;
    let expected_arity = expected_cells
        .len()
        .checked_add(1)
        .ok_or_else(|| semantic_error("appended row arity overflow"))?;
    let tokens = parse_tokens(
        appended_line,
        &readback_limits,
        "output.appendedRow",
        TokenCountLimit {
            max_tokens: expected_arity,
            code: ROW_ARITY_INVALID,
            path: "output.appendedRow",
            message: "appended row has more tokens than its full width",
        },
    )
    .map_err(|error| readback_error(format!("appended row parse failed: {}", error.code)))?;
    if tokens.len() != expected_cells.len() + 1
        || tokens[0].quoted
        || tokens[0].bytes != expected_label.as_bytes()
    {
        return Err(semantic_error("appended row label or arity differs"));
    }
    for (column_index, expected) in expected_cells.iter().enumerate() {
        let actual = token_to_cell(tokens[column_index + 1]);
        let expected = expected.map(Clone::clone).unwrap_or(TwoDaCellValueV1::Null);
        if actual != expected {
            return Err(semantic_error(format!(
                "appended cell {column_index} differs"
            )));
        }
    }

    Ok(output_inspection)
}

fn suffix_matches_canonical_plan(
    suffix: &[u8],
    source: &TwoDaInspectionV1,
    expected_label: &[u8],
    expected_cells: &[Option<&TwoDaCellValueV1>],
) -> bool {
    let mut cursor = 0usize;
    let newline = newline_bytes(source.newline);
    if !source.terminal_newline && !consume_exact(suffix, &mut cursor, newline) {
        return false;
    }
    if !consume_exact(suffix, &mut cursor, expected_label) {
        return false;
    }
    for value in expected_cells {
        if !consume_exact(suffix, &mut cursor, b" ") {
            return false;
        }
        match value {
            None | Some(TwoDaCellValueV1::Null) => {
                if !consume_exact(suffix, &mut cursor, b"****") {
                    return false;
                }
            }
            Some(TwoDaCellValueV1::Text { value }) => {
                let quoted =
                    value.is_empty() || value == "****" || value.as_bytes().contains(&b' ');
                if quoted && !consume_exact(suffix, &mut cursor, b"\"") {
                    return false;
                }
                if !consume_exact(suffix, &mut cursor, value.as_bytes()) {
                    return false;
                }
                if quoted && !consume_exact(suffix, &mut cursor, b"\"") {
                    return false;
                }
            }
        }
    }
    consume_exact(suffix, &mut cursor, newline) && cursor == suffix.len()
}

fn consume_exact(input: &[u8], cursor: &mut usize, expected: &[u8]) -> bool {
    let Some(end) = cursor.checked_add(expected.len()) else {
        return false;
    };
    if input.get(*cursor..end) != Some(expected) {
        return false;
    }
    *cursor = end;
    true
}

fn readback_error(message: impl Into<String>) -> TwoDaError {
    TwoDaError::fatal(READBACK_FAILED, "output", 0, None, None, message)
}

fn semantic_error(message: impl Into<String>) -> TwoDaError {
    TwoDaError::fatal(SEMANTIC_DIFF, "output", 0, None, None, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_accept_exact_hard_caps() {
        scan_input(b"ASCII", &TwoDaLimitsV1::default()).unwrap();
    }

    #[test]
    fn limits_reject_zero_and_above_hard_caps() {
        let cases = [
            (
                TwoDaLimitsV1 {
                    max_input_bytes: 0,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxInputBytes",
            ),
            (
                TwoDaLimitsV1 {
                    max_input_bytes: TWO_DA_MAX_INPUT_BYTES + 1,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxInputBytes",
            ),
            (
                TwoDaLimitsV1 {
                    max_columns: 0,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxColumns",
            ),
            (
                TwoDaLimitsV1 {
                    max_columns: TWO_DA_MAX_COLUMNS + 1,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxColumns",
            ),
            (
                TwoDaLimitsV1 {
                    max_rows: 0,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxRows",
            ),
            (
                TwoDaLimitsV1 {
                    max_rows: TWO_DA_MAX_ROWS + 1,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxRows",
            ),
            (
                TwoDaLimitsV1 {
                    max_token_bytes: 0,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxTokenBytes",
            ),
            (
                TwoDaLimitsV1 {
                    max_token_bytes: TWO_DA_MAX_TOKEN_BYTES + 1,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxTokenBytes",
            ),
            (
                TwoDaLimitsV1 {
                    max_diagnostics: 0,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxDiagnostics",
            ),
            (
                TwoDaLimitsV1 {
                    max_diagnostics: TWO_DA_MAX_DIAGNOSTICS + 1,
                    ..TwoDaLimitsV1::default()
                },
                "limits.maxDiagnostics",
            ),
        ];

        for (limits, path) in cases {
            let error = scan_input(&[0xff], &limits).unwrap_err();
            assert_eq!(error.code, SCHEMA_INVALID);
            assert_eq!(error.path, path);
        }
    }

    #[test]
    fn input_limit_precedes_encoding_errors() {
        let limits = TwoDaLimitsV1 {
            max_input_bytes: 1,
            ..TwoDaLimitsV1::default()
        };
        let error = scan_input(&[0xff, 0xff], &limits).unwrap_err();
        assert_eq!(error.code, LIMIT_EXCEEDED);
    }

    #[test]
    fn lf_scan_returns_borrowed_lines_and_terminal_state() {
        let (newline, terminal, lines, line_limit_error) =
            scan_input(b"2DA V2.0\n\nCOL\n", &TwoDaLimitsV1::default()).unwrap();
        assert!(line_limit_error.is_none());
        assert_eq!(newline, Some(TwoDaNewlineV1::Lf));
        assert!(terminal);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].bytes, b"2DA V2.0");
        assert_eq!(lines[1].bytes, b"");
        assert_eq!(lines[2].bytes, b"COL");
        assert_eq!(lines[2].byte_offset, 10);
        assert_eq!(lines[2].line, 3);
    }

    #[test]
    fn crlf_scan_preserves_final_unterminated_line() {
        let (newline, terminal, lines, line_limit_error) =
            scan_input(b"2DA V2.0\r\n\r\nCOL", &TwoDaLimitsV1::default()).unwrap();
        assert!(line_limit_error.is_none());
        assert_eq!(newline, Some(TwoDaNewlineV1::CrLf));
        assert!(!terminal);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[2].bytes, b"COL");
    }

    #[test]
    fn scan_accepts_printable_ascii_boundaries() {
        let (newline, terminal, lines, line_limit_error) = scan_input(
            b" !\"#$%&'()*+,-./09:;<=>?@AZ[\\]^_`az{|}~",
            &TwoDaLimitsV1::default(),
        )
        .unwrap();
        assert!(line_limit_error.is_none());
        assert_eq!(newline, None);
        assert!(!terminal);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn scan_rejects_bom_non_ascii_and_control_bytes() {
        let cases: &[&[u8]] = &[&[0xef, 0xbb, 0xbf], &[0x80], &[0x1f], &[0x7f]];
        for &bytes in cases {
            assert_eq!(
                scan_input(bytes, &TwoDaLimitsV1::default())
                    .unwrap_err()
                    .code,
                ENCODING_UNSUPPORTED
            );
        }
    }

    #[test]
    fn scan_rejects_nul_and_tab() {
        let cases: &[(&[u8], &str)] = &[(b"A\0", NUL_FORBIDDEN), (b"A\t", TAB_FORBIDDEN)];
        for &(bytes, code) in cases {
            assert_eq!(
                scan_input(bytes, &TwoDaLimitsV1::default())
                    .unwrap_err()
                    .code,
                code
            );
        }
    }

    #[test]
    fn scan_rejects_bare_cr_and_mixed_eol() {
        let cases: &[&[u8]] = &[b"A\rB", b"A\nB\r\n", b"A\r\nB\n"];
        for &bytes in cases {
            assert_eq!(
                scan_input(bytes, &TwoDaLimitsV1::default())
                    .unwrap_err()
                    .code,
                NEWLINE_INVALID
            );
        }
    }

    #[test]
    fn inspection_parses_default_columns_rows_and_hash() {
        let source = b"2DA V2.0\r\nDEFAULT: \"fallback text\"\r\n  Name VALUE_2  \r\n0 \"A B\" ****\r\n0 C \"****\"";
        let report = inspect_two_da_v2(source, &TwoDaLimitsV1::default()).unwrap();
        assert_eq!(report.format, "2DA");
        assert_eq!(report.version, "V2.0");
        assert_eq!(report.byte_length, source.len() as u64);
        assert_eq!(report.source_sha256.len(), 64);
        assert_eq!(report.newline, TwoDaNewlineV1::CrLf);
        assert!(!report.terminal_newline);
        assert_eq!(
            report.default_value,
            Some(TwoDaCellValueV1::Text {
                value: "fallback text".to_owned()
            })
        );
        assert_eq!(report.columns, ["Name", "VALUE_2"]);
        assert_eq!(report.physical_row_count, 2);
        assert_eq!(report.next_append_index, Some(2));
        assert_eq!(report.row_label_mismatch_count, 1);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].code, ROW_LABEL_MISMATCH);
        assert_eq!(report.diagnostics[0].line, Some(5));
    }

    #[test]
    fn inspection_keeps_null_and_quoted_stars_distinct() {
        let null =
            inspect_two_da_v2(b"2DA V2.0\nDEFAULT: ****\nA\n", &TwoDaLimitsV1::default()).unwrap();
        assert_eq!(null.default_value, Some(TwoDaCellValueV1::Null));

        let text = inspect_two_da_v2(
            b"2DA V2.0\nDEFAULT: \"****\"\nA\n",
            &TwoDaLimitsV1::default(),
        )
        .unwrap();
        assert_eq!(
            text.default_value,
            Some(TwoDaCellValueV1::Text {
                value: "****".to_owned()
            })
        );
    }

    #[test]
    fn inspection_rejects_header_default_column_and_quote_errors() {
        let cases: &[(&[u8], &str)] = &[
            (b"2DA V1.0\n\nA\n", HEADER_INVALID),
            (b"2DA V2.0\nDEFAULT:\nA\n", DEFAULT_INVALID),
            (b"2DA V2.0\nDEFAULT: one two\nA\n", DEFAULT_INVALID),
            (b"2DA V2.0\n\nA-B\n", COLUMN_INVALID),
            (b"2DA V2.0\n\nName NAME\n", COLUMN_AMBIGUOUS),
            (b"2DA V2.0\n\nA\n0 \"open\n", QUOTE_INVALID),
            (b"2DA V2.0\n\nA\n0 \"x\"tail\n", QUOTE_INVALID),
        ];
        for &(source, code) in cases {
            assert_eq!(
                inspect_two_da_v2(source, &TwoDaLimitsV1::default())
                    .unwrap_err()
                    .code,
                code
            );
        }
    }

    #[test]
    fn inspection_enforces_token_column_and_row_limits() {
        let token_limits = TwoDaLimitsV1 {
            max_token_bytes: 3,
            ..TwoDaLimitsV1::default()
        };
        assert_eq!(
            inspect_two_da_v2(b"2DA V2.0\n\nLONG\n", &token_limits)
                .unwrap_err()
                .path,
            "limits.maxTokenBytes"
        );

        let column_limits = TwoDaLimitsV1 {
            max_columns: 1,
            ..TwoDaLimitsV1::default()
        };
        assert_eq!(
            inspect_two_da_v2(b"2DA V2.0\n\nA B\n", &column_limits)
                .unwrap_err()
                .path,
            "limits.maxColumns"
        );

        let row_limits = TwoDaLimitsV1 {
            max_rows: 1,
            ..TwoDaLimitsV1::default()
        };
        assert_eq!(
            inspect_two_da_v2(b"2DA V2.0\n\nA\n0 x\n1 y\n", &row_limits)
                .unwrap_err()
                .path,
            "limits.maxRows"
        );
    }

    #[test]
    fn inspection_validates_row_arity_labels_and_diagnostic_cap() {
        let cases: &[(&[u8], &str)] = &[
            (b"2DA V2.0\n\nA B\n0 one\n", ROW_ARITY_INVALID),
            (b"2DA V2.0\n\nA\nzero one\n", ROW_LABEL_INVALID),
            (b"2DA V2.0\n\nA\n4294967296 one\n", ROW_LABEL_INVALID),
            (b"2DA V2.0\n\nA\n\n", ROW_ARITY_INVALID),
        ];
        for &(source, code) in cases {
            assert_eq!(
                inspect_two_da_v2(source, &TwoDaLimitsV1::default())
                    .unwrap_err()
                    .code,
                code
            );
        }

        let limits = TwoDaLimitsV1 {
            max_diagnostics: 1,
            ..TwoDaLimitsV1::default()
        };
        let report = inspect_two_da_v2(b"2DA V2.0\n\nA\n9 x\n9 y\n", &limits).unwrap();
        assert_eq!(report.row_label_mismatch_count, 2);
        assert_eq!(report.diagnostics.len(), 1);
    }

    #[test]
    fn append_checks_limits_then_request_schema_before_input() {
        let request = TwoDaAppendRequestV1 {
            schema_version: 2,
            cells: Vec::new(),
        };
        let invalid_limits = TwoDaLimitsV1 {
            max_rows: 0,
            ..TwoDaLimitsV1::default()
        };
        assert_eq!(
            append_two_da_row_v1(b"", &request, &invalid_limits)
                .unwrap_err()
                .path,
            "limits.maxRows"
        );

        let error = append_two_da_row_v1(b"", &request, &TwoDaLimitsV1::default()).unwrap_err();
        assert_eq!(error.code, SCHEMA_INVALID);
        assert_eq!(error.path, "request.schemaVersion");

        let request = TwoDaAppendRequestV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            cells: Vec::new(),
        };
        assert_eq!(
            append_two_da_row_v1(b"", &request, &TwoDaLimitsV1::default())
                .unwrap_err()
                .code,
            HEADER_INVALID
        );
    }

    #[test]
    fn append_accepts_exact_input_and_generated_token_limits() {
        let source = b"2DA V2.0\n\nA\n";
        let request = TwoDaAppendRequestV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "a".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "x".to_owned(),
                },
            }],
        };
        let limits = TwoDaLimitsV1 {
            max_input_bytes: source.len() as u64,
            max_token_bytes: 1,
            ..TwoDaLimitsV1::default()
        };
        let artifact = append_two_da_row_v1(source, &request, &limits).unwrap();
        assert_eq!(artifact.payload, b"2DA V2.0\n\nA\n0 x\n");
        assert!(artifact.report.output_byte_length > limits.max_input_bytes);

        let too_small_input = TwoDaLimitsV1 {
            max_input_bytes: source.len() as u64 - 1,
            ..limits
        };
        assert_eq!(
            append_two_da_row_v1(source, &request, &too_small_input)
                .unwrap_err()
                .code,
            LIMIT_EXCEEDED
        );

        let empty_text = TwoDaAppendRequestV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: String::new(),
                },
            }],
        };
        assert_eq!(
            append_two_da_row_v1(source, &empty_text, &limits)
                .unwrap_err()
                .path,
            "limits.maxTokenBytes"
        );
    }

    #[test]
    fn append_readback_classifies_parse_and_semantic_mutations() {
        let source_bytes = b"2DA V2.0\n\nA\n";
        let value = TwoDaCellValueV1::Text {
            value: "x".to_owned(),
        };
        let request = TwoDaAppendRequestV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: value.clone(),
            }],
        };
        let limits = TwoDaLimitsV1::default();
        let artifact = append_two_da_row_v1(source_bytes, &request, &limits).unwrap();
        let source = inspect_two_da_v2(source_bytes, &limits).unwrap();
        let expected = [Some(&value)];

        let mut semantic_mutation = artifact.payload.clone();
        semantic_mutation[source_bytes.len() + 2] = b'y';
        assert_eq!(
            verify_append_readback(
                source_bytes,
                &semantic_mutation,
                &source,
                &expected,
                &limits
            )
            .unwrap_err()
            .code,
            SEMANTIC_DIFF
        );

        let mut extra_separator = artifact.payload.clone();
        extra_separator.insert(source_bytes.len() + 1, b' ');
        assert_eq!(
            verify_append_readback(source_bytes, &extra_separator, &source, &expected, &limits)
                .unwrap_err()
                .code,
            SEMANTIC_DIFF
        );

        let mut redundant_quoting = artifact.payload.clone();
        redundant_quoting.splice(
            source_bytes.len() + 2..source_bytes.len() + 3,
            b"\"x\"".iter().copied(),
        );
        assert_eq!(
            verify_append_readback(
                source_bytes,
                &redundant_quoting,
                &source,
                &expected,
                &limits
            )
            .unwrap_err()
            .code,
            SEMANTIC_DIFF
        );

        let mut parse_mutation = artifact.payload.clone();
        parse_mutation[source_bytes.len() + 2] = 0;
        assert_eq!(
            verify_append_readback(source_bytes, &parse_mutation, &source, &expected, &limits)
                .unwrap_err()
                .code,
            READBACK_FAILED
        );

        let mut trailing_blank_row = artifact.payload;
        trailing_blank_row.push(b'\n');
        assert_eq!(
            verify_append_readback(
                source_bytes,
                &trailing_blank_row,
                &source,
                &expected,
                &limits
            )
            .unwrap_err()
            .code,
            READBACK_FAILED
        );
    }

    #[test]
    fn append_report_json_is_frozen_and_assignments_use_source_order() {
        let source = b"2DA V2.0\n\nA B\n";
        let request = TwoDaAppendRequestV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            cells: vec![TwoDaCellAssignmentV1 {
                column_name: "A".to_owned(),
                value: TwoDaCellValueV1::Text {
                    value: "x".to_owned(),
                },
            }],
        };
        let artifact = append_two_da_row_v1(source, &request, &TwoDaLimitsV1::default()).unwrap();
        assert_eq!(artifact.payload, b"2DA V2.0\n\nA B\n0 x ****\n");
        assert_eq!(
            serde_json::to_string(&artifact.report).unwrap(),
            r#"{"schemaVersion":1,"sourceSha256":"ff4dd1895efb5c60ca173178873d4e7be9b6dfbdb34c2631ffee5af713d0f0c1","outputSha256":"555e332c3f8771e604fce31cc28c57044c8e790965e55a6ddb8200c8f2d12ce8","sourceByteLength":14,"outputByteLength":23,"sourcePrefixPreserved":true,"appendedRowIndex":0,"physicalRowsBefore":0,"physicalRowsAfter":1,"newline":"LF","insertedSeparatorNewline":false,"changedCells":[{"columnIndex":0,"columnName":"A","value":{"kind":"TEXT","value":"x"}}],"diagnostics":[]}"#
        );
    }

    #[test]
    fn append_invalid_inputs_do_not_panic() {
        let complete = b"2DA V2.0\n\nA\n0 value\n";
        let request = TwoDaAppendRequestV1 {
            schema_version: TWO_DA_SCHEMA_VERSION,
            cells: Vec::new(),
        };
        for length in 0..complete.len() {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                append_two_da_row_v1(&complete[..length], &request, &TwoDaLimitsV1::default())
                    .map(|_| ())
                    .map_err(|error| error.code)
            }));
            assert!(result.is_ok(), "append panicked at prefix {length}");
        }
    }
}
