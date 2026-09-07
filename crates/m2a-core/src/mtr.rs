//! Deterministic Aurora MTR text writer and strict readback parser.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::aurora_material::{AuroraMaterialIrV1, AuroraMtrBlendingV1, AuroraRenderHintV1};

pub const MTR_RESOURCE_TYPE_V1: u16 = 2072;
pub const MTR_TEXT_MAX_BYTES_V1: usize = 16 * 1024;
pub const MTR_SCHEMA_VERSION_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MtrRenderHintV1 {
    Normal,
    NormalAndSpecMapped,
    NormalTangents,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MtrBlendingV1 {
    Punchthrough,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MtrTextureBindingV1 {
    pub slot: u8,
    pub resref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MtrDocumentV1 {
    pub schema_version: u32,
    pub textures: Vec<MtrTextureBindingV1>,
    pub render_hint: MtrRenderHintV1,
    pub transparency: bool,
    pub two_sided: bool,
    pub blending: Option<MtrBlendingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MtrMaterialResourceNamesV1 {
    pub diffuse: Option<String>,
    pub normal: Option<String>,
    pub specular: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MtrTextErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for MtrTextErrorV1 {
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

impl std::error::Error for MtrTextErrorV1 {}

fn error(code: &str, line: Option<usize>, message: impl Into<String>) -> MtrTextErrorV1 {
    MtrTextErrorV1 {
        schema_version: MTR_SCHEMA_VERSION_V1,
        code: code.to_owned(),
        line,
        message: message.into(),
    }
}

pub fn is_valid_aurora_resref_v1(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_document(document: &MtrDocumentV1) -> Result<(), MtrTextErrorV1> {
    if document.schema_version != MTR_SCHEMA_VERSION_V1 {
        return Err(error(
            "MTR-SCHEMA-VERSION-UNSUPPORTED",
            None,
            format!("expected schema version {MTR_SCHEMA_VERSION_V1}"),
        ));
    }
    let mut occupied = [false; 11];
    for binding in &document.textures {
        if binding.slot > 10 {
            return Err(error(
                "MTR-TEXTURE-SLOT-RANGE",
                None,
                format!("texture slot {} is outside 0..=10", binding.slot),
            ));
        }
        if occupied[usize::from(binding.slot)] {
            return Err(error(
                "MTR-TEXTURE-SLOT-DUPLICATE",
                None,
                format!("texture slot {} occurs more than once", binding.slot),
            ));
        }
        occupied[usize::from(binding.slot)] = true;
        if !is_valid_aurora_resref_v1(&binding.resref) {
            return Err(error(
                "MTR-TEXTURE-RESREF-INVALID",
                None,
                format!(
                    "texture{} has invalid Aurora resref {:?}",
                    binding.slot, binding.resref
                ),
            ));
        }
    }
    Ok(())
}

fn render_hint_token(value: MtrRenderHintV1) -> &'static str {
    match value {
        MtrRenderHintV1::Normal => "normal",
        MtrRenderHintV1::NormalAndSpecMapped => "normalandspecmapped",
        MtrRenderHintV1::NormalTangents => "normaltangents",
    }
}

/// Writes a canonical ASCII/LF MTR payload in Aurora parser order.
pub fn write_mtr_v1(document: &MtrDocumentV1) -> Result<Vec<u8>, MtrTextErrorV1> {
    validate_document(document)?;
    let mut textures = document.textures.clone();
    textures.sort_by_key(|binding| binding.slot);
    let mut output = String::new();
    for binding in textures {
        output.push_str(&format!("texture{} {}\n", binding.slot, binding.resref));
    }
    output.push_str(&format!(
        "renderhint {}\n",
        render_hint_token(document.render_hint)
    ));
    output.push_str(if document.transparency {
        "transparency 1\n"
    } else {
        "transparency 0\n"
    });
    output.push_str(if document.two_sided {
        "twosided 1\n"
    } else {
        "twosided 0\n"
    });
    if let Some(MtrBlendingV1::Punchthrough) = document.blending {
        output.push_str("blending punchthrough\n");
    }
    if output.len() > MTR_TEXT_MAX_BYTES_V1 {
        return Err(error(
            "MTR-TEXT-TOO-LARGE",
            None,
            format!("MTR payload exceeds {MTR_TEXT_MAX_BYTES_V1} bytes"),
        ));
    }
    Ok(output.into_bytes())
}

fn parse_bool(token: &str, line: usize) -> Result<bool, MtrTextErrorV1> {
    match token {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(error(
            "MTR-BOOLEAN-INVALID",
            Some(line),
            "boolean value must be exactly 0 or 1",
        )),
    }
}

/// Parses the supported V1 subset and rejects unknown, duplicate, or non-canonical syntax.
pub fn parse_mtr_v1(bytes: &[u8]) -> Result<MtrDocumentV1, MtrTextErrorV1> {
    if bytes.len() > MTR_TEXT_MAX_BYTES_V1 {
        return Err(error(
            "MTR-TEXT-TOO-LARGE",
            None,
            "MTR payload is too large",
        ));
    }
    if bytes.is_empty()
        || bytes.last() != Some(&b'\n')
        || bytes.contains(&b'\r')
        || bytes.contains(&0)
    {
        return Err(error(
            "MTR-TEXT-ENCODING-INVALID",
            None,
            "MTR must be non-empty ASCII text with LF endings and a final LF",
        ));
    }
    if !bytes.is_ascii() {
        return Err(error(
            "MTR-TEXT-NON-ASCII",
            None,
            "MTR must contain ASCII only",
        ));
    }
    let source = std::str::from_utf8(bytes).expect("ASCII is valid UTF-8");
    let mut textures = Vec::new();
    let mut render_hint = None;
    let mut transparency = None;
    let mut two_sided = None;
    let mut blending = None;
    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        if raw_line.is_empty()
            || raw_line != raw_line.trim()
            || raw_line.contains("  ")
            || raw_line.contains('\t')
        {
            return Err(error(
                "MTR-LINE-NON-CANONICAL",
                Some(line_number),
                "line spacing is not canonical",
            ));
        }
        let tokens = raw_line.split(' ').collect::<Vec<_>>();
        if tokens.len() != 2 {
            return Err(error(
                "MTR-LINE-ARITY",
                Some(line_number),
                "directive requires exactly one value",
            ));
        }
        match tokens[0] {
            key if key.strip_prefix("texture").is_some() => {
                let suffix = key.strip_prefix("texture").unwrap();
                let slot = suffix.parse::<u8>().map_err(|_| {
                    error(
                        "MTR-TEXTURE-SLOT-INVALID",
                        Some(line_number),
                        "texture directive must be texture0..texture10",
                    )
                })?;
                if slot > 10 {
                    return Err(error(
                        "MTR-TEXTURE-SLOT-RANGE",
                        Some(line_number),
                        "texture slot is outside 0..=10",
                    ));
                }
                textures.push(MtrTextureBindingV1 {
                    slot,
                    resref: tokens[1].to_owned(),
                });
            }
            "renderhint" => {
                if render_hint.is_some() {
                    return Err(error(
                        "MTR-DIRECTIVE-DUPLICATE",
                        Some(line_number),
                        "renderhint occurs more than once",
                    ));
                }
                render_hint = Some(match tokens[1] {
                    "normal" => MtrRenderHintV1::Normal,
                    "normalandspecmapped" => MtrRenderHintV1::NormalAndSpecMapped,
                    "normaltangents" => MtrRenderHintV1::NormalTangents,
                    _ => {
                        return Err(error(
                            "MTR-RENDERHINT-INVALID",
                            Some(line_number),
                            "unsupported renderhint",
                        ));
                    }
                });
            }
            "transparency" => {
                if transparency
                    .replace(parse_bool(tokens[1], line_number)?)
                    .is_some()
                {
                    return Err(error(
                        "MTR-DIRECTIVE-DUPLICATE",
                        Some(line_number),
                        "transparency occurs more than once",
                    ));
                }
            }
            "twosided" => {
                if two_sided
                    .replace(parse_bool(tokens[1], line_number)?)
                    .is_some()
                {
                    return Err(error(
                        "MTR-DIRECTIVE-DUPLICATE",
                        Some(line_number),
                        "twosided occurs more than once",
                    ));
                }
            }
            "blending" => {
                if blending.is_some() {
                    return Err(error(
                        "MTR-DIRECTIVE-DUPLICATE",
                        Some(line_number),
                        "blending occurs more than once",
                    ));
                }
                blending = Some(match tokens[1] {
                    "punchthrough" => MtrBlendingV1::Punchthrough,
                    _ => {
                        return Err(error(
                            "MTR-BLENDING-UNSAFE",
                            Some(line_number),
                            "only punchthrough is supported by V1",
                        ));
                    }
                });
            }
            _ => {
                return Err(error(
                    "MTR-DIRECTIVE-UNSUPPORTED",
                    Some(line_number),
                    format!("unsupported directive {:?}", tokens[0]),
                ));
            }
        }
    }
    let document = MtrDocumentV1 {
        schema_version: MTR_SCHEMA_VERSION_V1,
        textures,
        render_hint: render_hint
            .ok_or_else(|| error("MTR-RENDERHINT-MISSING", None, "renderhint is required"))?,
        transparency: transparency
            .ok_or_else(|| error("MTR-TRANSPARENCY-MISSING", None, "transparency is required"))?,
        two_sided: two_sided
            .ok_or_else(|| error("MTR-TWOSIDED-MISSING", None, "twosided is required"))?,
        blending,
    };
    validate_document(&document)?;
    if write_mtr_v1(&document)? != bytes {
        return Err(error(
            "MTR-TEXT-NON-CANONICAL",
            None,
            "directive order is not canonical",
        ));
    }
    Ok(document)
}

pub fn compile_mtr_document_v1(
    material: &AuroraMaterialIrV1,
    names: &MtrMaterialResourceNamesV1,
) -> Result<MtrDocumentV1, MtrTextErrorV1> {
    if !material.mtr_required {
        return Err(error(
            "MTR-MATERIAL-NOT-REQUIRED",
            None,
            "material profile does not require MTR output",
        ));
    }
    let state = material
        .mtr
        .as_ref()
        .ok_or_else(|| error("MTR-STATE-MISSING", None, "MTR state is missing"))?;
    let mut textures = Vec::new();
    if material.diffuse_texture.is_some() {
        textures.push(MtrTextureBindingV1 {
            slot: 0,
            resref: names.diffuse.clone().ok_or_else(|| {
                error(
                    "MTR-DIFFUSE-RESREF-MISSING",
                    None,
                    "texture0 resref is required",
                )
            })?,
        });
    }
    if material.normal_texture.is_some() {
        textures.push(MtrTextureBindingV1 {
            slot: 1,
            resref: names.normal.clone().ok_or_else(|| {
                error(
                    "MTR-NORMAL-RESREF-MISSING",
                    None,
                    "texture1 resref is required",
                )
            })?,
        });
    }
    if material.specular_texture_plan.is_some() {
        textures.push(MtrTextureBindingV1 {
            slot: 2,
            resref: names.specular.clone().ok_or_else(|| {
                error(
                    "MTR-SPECULAR-RESREF-MISSING",
                    None,
                    "texture2 resref is required",
                )
            })?,
        });
    }
    let document = MtrDocumentV1 {
        schema_version: MTR_SCHEMA_VERSION_V1,
        textures,
        render_hint: match state.render_hint {
            AuroraRenderHintV1::Normal => MtrRenderHintV1::Normal,
            AuroraRenderHintV1::NormalAndSpecMapped => MtrRenderHintV1::NormalAndSpecMapped,
        },
        transparency: state.transparency,
        two_sided: state.two_sided,
        blending: state.blending.map(|value| match value {
            AuroraMtrBlendingV1::Punchthrough => MtrBlendingV1::Punchthrough,
        }),
    };
    validate_document(&document)?;
    Ok(document)
}
