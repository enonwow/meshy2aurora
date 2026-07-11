//! Format-safe core for the standalone Meshy-to-Aurora pipeline.

pub mod erf;
pub mod mdl;
pub mod reference_proof;

pub use mdl::{
    InspectionReport, ParseError, ParserLimits, inspect_binary_mdl, inspect_binary_mdl_with_limits,
};
pub use reference_proof::{
    CapabilityResult, CapabilityStatus, ExecutionMetadata, HashAlgorithm, InputFingerprint,
    InvariantResult, InvariantStatus, REFERENCE_PROOF_SCHEMA_VERSION, ReaderIdentity,
    ReferenceCapability, ReferenceIdentity, ReferenceManifest, ReferenceManifestEntry,
    ReferenceProofError, ReferenceProofPacket, ReferenceSource, build_reference_proof_packet,
};
