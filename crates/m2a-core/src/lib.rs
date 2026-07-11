//! Format-safe core for the standalone Meshy-to-Aurora pipeline.

pub mod mdl;

pub use mdl::{
    InspectionReport, ParseError, ParserLimits, inspect_binary_mdl, inspect_binary_mdl_with_limits,
};
