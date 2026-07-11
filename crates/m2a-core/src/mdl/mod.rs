mod binary_reader;
mod errors;
mod parse_binary_mdl;
mod types;

pub use errors::ParseError;
pub use parse_binary_mdl::{inspect_binary_mdl, inspect_binary_mdl_with_limits};
pub use types::{InspectionReport, ParserLimits};
