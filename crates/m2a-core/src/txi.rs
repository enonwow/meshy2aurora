//! Deterministic Aurora TXI text writer and strict readback parser.

use std::fmt;

use serde::{Deserialize, Serialize};

pub const TXI_RESOURCE_TYPE_V1: u16 = 2022;
pub const TXI_TEXT_MAX_BYTES_V1: usize = 16 * 1024;
pub const TXI_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TxiBlendingV1 {
    Punchthrough,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TxiDocumentV1 {
    pub schema_version: u32,
    pub mipmap: Option<bool>,
    pub filter: Option<bool>,
    pub gamma: Option<f32>,
    pub is_bump_map: Option<bool>,
    pub clamp: Option<bool>,
    pub alpha_mean: Option<f32>,
    pub is_diffuse_bump_map: Option<bool>,
    pub is_specular_bump_map: Option<bool>,
    pub bump_map_scaling: Option<f32>,
    pub specular_color: Option<[f32; 3]>,
    pub blending: Option<TxiBlendingV1>,
}

impl TxiDocumentV1 {
    pub fn normal_map_v1() -> Self {
        Self {
            schema_version: TXI_SCHEMA_VERSION_V1,
            mipmap: Some(true),
            filter: Some(true),
            gamma: None,
            is_bump_map: Some(true),
            clamp: Some(false),
            alpha_mean: None,
            is_diffuse_bump_map: None,
            is_specular_bump_map: None,
            bump_map_scaling: Some(1.0),
            specular_color: None,
            blending: None,
        }
    }

    pub fn punchthrough_v1() -> Self {
        Self {
            schema_version: TXI_SCHEMA_VERSION_V1,
            mipmap: Some(true),
            filter: Some(true),
            gamma: None,
            is_bump_map: None,
            clamp: Some(false),
            alpha_mean: None,
            is_diffuse_bump_map: None,
            is_specular_bump_map: None,
            bump_map_scaling: None,
            specular_color: None,
            blending: Some(TxiBlendingV1::Punchthrough),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TxiTextErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for TxiTextErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(
                formatter,
                "{} at line {}: {}",
                self.code, line, self.message
            ),
            None => write!(formatter, "{}: {}", self.code, self.message),
        }
    }
}

impl std::error::Error for TxiTextErrorV1 {}

fn error(code: &str, line: Option<usize>, message: impl Into<String>) -> TxiTextErrorV1 {
    TxiTextErrorV1 {
        schema_version: TXI_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        line,
        message: message.into(),
    }
}

fn bool_token(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn format_float(value: f32) -> Result<String, TxiTextErrorV1> {
    if !value.is_finite() {
        return Err(error(
            "TXI-NUMBER-NON-FINITE",
            None,
            "TXI numeric values must be finite",
        ));
    }
    Ok(format!("{value:.6}"))
}

fn validate(document: &TxiDocumentV1) -> Result<(), TxiTextErrorV1> {
    if document.schema_version != TXI_SCHEMA_VERSION_V1 {
        return Err(error(
            "TXI-SCHEMA-VERSION-UNSUPPORTED",
            None,
            format!("expected schema version {TXI_SCHEMA_VERSION_V1}"),
        ));
    }
    for value in document
        .gamma
        .iter()
        .chain(document.alpha_mean.iter())
        .chain(document.bump_map_scaling.iter())
        .chain(
            document
                .specular_color
                .iter()
                .flat_map(|color| color.iter()),
        )
    {
        if !value.is_finite() {
            return Err(error(
                "TXI-NUMBER-NON-FINITE",
                None,
                "TXI numeric values must be finite",
            ));
        }
    }
    Ok(())
}

/// Writes the supported TXI subset using canonical ASCII, LF, and fixed float formatting.
pub fn write_txi_v1(document: &TxiDocumentV1) -> Result<Vec<u8>, TxiTextErrorV1> {
    validate(document)?;
    let mut output = String::new();
    macro_rules! boolean {
        ($field:ident, $token:literal) => {
            if let Some(value) = document.$field {
                output.push_str(concat!($token, " "));
                output.push_str(bool_token(value));
                output.push('\n');
            }
        };
    }
    boolean!(mipmap, "mipmap");
    boolean!(filter, "filter");
    if let Some(value) = document.gamma {
        output.push_str(&format!("gamma {}\n", format_float(value)?));
    }
    boolean!(is_bump_map, "isbumpmap");
    boolean!(clamp, "clamp");
    if let Some(value) = document.alpha_mean {
        output.push_str(&format!("alphamean {}\n", format_float(value)?));
    }
    boolean!(is_diffuse_bump_map, "isdiffusebumpmap");
    boolean!(is_specular_bump_map, "isspecularbumpmap");
    if let Some(value) = document.bump_map_scaling {
        output.push_str(&format!("bumpmapscaling {}\n", format_float(value)?));
    }
    if let Some(color) = document.specular_color {
        output.push_str(&format!(
            "specularcolor {} {} {}\n",
            format_float(color[0])?,
            format_float(color[1])?,
            format_float(color[2])?
        ));
    }
    if let Some(TxiBlendingV1::Punchthrough) = document.blending {
        output.push_str("blending punchthrough\n");
    }
    if output.is_empty() {
        return Err(error(
            "TXI-DOCUMENT-EMPTY",
            None,
            "TXI must contain at least one directive",
        ));
    }
    if output.len() > TXI_TEXT_MAX_BYTES_V1 {
        return Err(error(
            "TXI-TEXT-TOO-LARGE",
            None,
            "TXI payload is too large",
        ));
    }
    Ok(output.into_bytes())
}

fn parse_bool(value: &str, line: usize) -> Result<bool, TxiTextErrorV1> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(error(
            "TXI-BOOLEAN-INVALID",
            Some(line),
            "boolean value must be exactly 0 or 1",
        )),
    }
}

fn parse_float(value: &str, line: usize) -> Result<f32, TxiTextErrorV1> {
    value
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            error(
                "TXI-NUMBER-INVALID",
                Some(line),
                "numeric value must be finite",
            )
        })
}

/// Parses and canonicalizes the supported TXI subset; unsupported directives fail closed.
pub fn parse_txi_v1(bytes: &[u8]) -> Result<TxiDocumentV1, TxiTextErrorV1> {
    if bytes.len() > TXI_TEXT_MAX_BYTES_V1 {
        return Err(error(
            "TXI-TEXT-TOO-LARGE",
            None,
            "TXI payload is too large",
        ));
    }
    if bytes.is_empty()
        || bytes.last() != Some(&b'\n')
        || bytes.contains(&b'\r')
        || bytes.contains(&0)
        || !bytes.is_ascii()
    {
        return Err(error(
            "TXI-TEXT-ENCODING-INVALID",
            None,
            "TXI must be non-empty ASCII text with LF endings and a final LF",
        ));
    }
    let mut document = TxiDocumentV1 {
        schema_version: TXI_SCHEMA_VERSION_V1,
        mipmap: None,
        filter: None,
        gamma: None,
        is_bump_map: None,
        clamp: None,
        alpha_mean: None,
        is_diffuse_bump_map: None,
        is_specular_bump_map: None,
        bump_map_scaling: None,
        specular_color: None,
        blending: None,
    };
    let source = std::str::from_utf8(bytes).expect("ASCII is valid UTF-8");
    for (index, raw_line) in source.lines().enumerate() {
        let line = index + 1;
        if raw_line.is_empty()
            || raw_line != raw_line.trim()
            || raw_line.contains("  ")
            || raw_line.contains('\t')
        {
            return Err(error(
                "TXI-LINE-NON-CANONICAL",
                Some(line),
                "line spacing is not canonical",
            ));
        }
        let tokens = raw_line.split(' ').collect::<Vec<_>>();
        macro_rules! set_bool {
            ($field:ident) => {{
                if tokens.len() != 2 {
                    return Err(error(
                        "TXI-LINE-ARITY",
                        Some(line),
                        "directive requires one value",
                    ));
                }
                if document
                    .$field
                    .replace(parse_bool(tokens[1], line)?)
                    .is_some()
                {
                    return Err(error(
                        "TXI-DIRECTIVE-DUPLICATE",
                        Some(line),
                        "directive occurs more than once",
                    ));
                }
            }};
        }
        macro_rules! set_float {
            ($field:ident) => {{
                if tokens.len() != 2 {
                    return Err(error(
                        "TXI-LINE-ARITY",
                        Some(line),
                        "directive requires one value",
                    ));
                }
                if document
                    .$field
                    .replace(parse_float(tokens[1], line)?)
                    .is_some()
                {
                    return Err(error(
                        "TXI-DIRECTIVE-DUPLICATE",
                        Some(line),
                        "directive occurs more than once",
                    ));
                }
            }};
        }
        match tokens[0] {
            "mipmap" => set_bool!(mipmap),
            "filter" => set_bool!(filter),
            "gamma" => set_float!(gamma),
            "isbumpmap" => set_bool!(is_bump_map),
            "clamp" => set_bool!(clamp),
            "alphamean" => set_float!(alpha_mean),
            "isdiffusebumpmap" => set_bool!(is_diffuse_bump_map),
            "isspecularbumpmap" => set_bool!(is_specular_bump_map),
            "bumpmapscaling" => set_float!(bump_map_scaling),
            "specularcolor" => {
                if tokens.len() != 4 {
                    return Err(error(
                        "TXI-LINE-ARITY",
                        Some(line),
                        "specularcolor requires three values",
                    ));
                }
                if document
                    .specular_color
                    .replace([
                        parse_float(tokens[1], line)?,
                        parse_float(tokens[2], line)?,
                        parse_float(tokens[3], line)?,
                    ])
                    .is_some()
                {
                    return Err(error(
                        "TXI-DIRECTIVE-DUPLICATE",
                        Some(line),
                        "specularcolor occurs more than once",
                    ));
                }
            }
            "blending" => {
                if tokens.len() != 2 {
                    return Err(error(
                        "TXI-LINE-ARITY",
                        Some(line),
                        "blending requires one value",
                    ));
                }
                if tokens[1] != "punchthrough" {
                    return Err(error(
                        "TXI-BLENDING-UNSAFE",
                        Some(line),
                        "only punchthrough is supported by V1",
                    ));
                }
                if document
                    .blending
                    .replace(TxiBlendingV1::Punchthrough)
                    .is_some()
                {
                    return Err(error(
                        "TXI-DIRECTIVE-DUPLICATE",
                        Some(line),
                        "blending occurs more than once",
                    ));
                }
            }
            _ => {
                return Err(error(
                    "TXI-DIRECTIVE-UNSUPPORTED",
                    Some(line),
                    format!("unsupported directive {:?}", tokens[0]),
                ));
            }
        }
    }
    validate(&document)?;
    if write_txi_v1(&document)? != bytes {
        return Err(error(
            "TXI-TEXT-NON-CANONICAL",
            None,
            "directive order or numeric formatting is not canonical",
        ));
    }
    Ok(document)
}
