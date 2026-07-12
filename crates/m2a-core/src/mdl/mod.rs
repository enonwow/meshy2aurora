mod binary_reader;
mod errors;
mod parse_binary_mdl;
mod semantic_readback;
mod types;
mod write_binary_mdl;
mod writer_types;

pub use errors::ParseError;
pub use parse_binary_mdl::{inspect_binary_mdl, inspect_binary_mdl_with_limits};
pub use types::{InspectionReport, ParserLimits};
pub use write_binary_mdl::write_binary_mdl;
pub use writer_types::{
    BinaryMdlArtifactV1, M4SemanticProjectionV1, MdlFormatProfileV1, MdlLayoutReportV1,
    MdlMaterialTextureBindingV1, MdlMeshNodeLayoutV1, MdlRigNodeLayoutV1, MdlWriteError,
    MdlWriterDeviationV1, MdlWriterOptionsV1, MdlWriterReportV1,
};
