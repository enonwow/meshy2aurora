//! Format-safe core for the standalone Meshy-to-Aurora pipeline.

pub mod erf;
pub mod glb;
pub mod mdl;
pub mod profile_a;
pub mod reference_proof;
pub mod tga;

pub use mdl::{
    BinaryMdlArtifactV1, InspectionReport, MdlFormatProfileV1, MdlMaterialTextureBindingV1,
    MdlWriteError, MdlWriterOptionsV1, ParserLimits, inspect_binary_mdl,
    inspect_binary_mdl_with_limits, write_binary_mdl,
};
pub use reference_proof::{
    CapabilityResult, CapabilityStatus, ExecutionMetadata, HashAlgorithm, InputFingerprint,
    InvariantResult, InvariantStatus, REFERENCE_PROOF_SCHEMA_VERSION, ReaderIdentity,
    ReferenceCapability, ReferenceIdentity, ReferenceManifest, ReferenceManifestEntry,
    ReferenceProofError, ReferenceProofPacket, ReferenceSource, build_reference_proof_packet,
};
