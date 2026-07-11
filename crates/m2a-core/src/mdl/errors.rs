use std::fmt;

use serde::Serialize;

pub const HEADER_INVALID: &str = "M2A-MDL-HEADER-INVALID";
pub const POINTER_OOB: &str = "M2A-MDL-POINTER-OOB";
pub const NODE_CYCLE: &str = "M2A-MDL-NODE-CYCLE";
pub const LIMIT_EXCEEDED: &str = "M2A-LIMIT-EXCEEDED";

/// Stable public error returned for input that cannot produce a valid report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub offset: usize,
    pub context: String,
}

impl ParseError {
    pub(crate) fn new(code: &str, offset: usize, context: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            code: code.to_owned(),
            severity: "error".to_owned(),
            offset,
            context: context.into(),
        }
    }

    pub(crate) fn header(offset: usize, context: impl Into<String>) -> Self {
        Self::new(HEADER_INVALID, offset, context)
    }

    pub(crate) fn pointer(offset: usize, context: impl Into<String>) -> Self {
        Self::new(POINTER_OOB, offset, context)
    }

    pub(crate) fn limit(offset: usize, context: impl Into<String>) -> Self {
        Self::new(LIMIT_EXCEEDED, offset, context)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.offset, self.context
        )
    }
}

impl std::error::Error for ParseError {}
