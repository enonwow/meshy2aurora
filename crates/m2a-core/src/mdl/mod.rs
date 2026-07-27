mod binary_reader;
mod errors;
mod parse_binary_mdl;
mod runtime_conformance;
mod semantic_readback;
mod skin_deformation;
mod types;
mod write_binary_mdl;
mod writer_types;

pub use errors::ParseError;
pub use parse_binary_mdl::{inspect_binary_mdl, inspect_binary_mdl_with_limits};
pub use runtime_conformance::{
    DirectCreatureAnimationProjectionV1, DirectCreatureEngineEnvelopeV1,
    DirectCreatureEngineEnvelopeVerdictV1, DirectCreatureEnvelopeDifferenceClassV1,
    DirectCreatureEnvelopeDifferentialV1, DirectCreatureEnvelopeFieldAssessmentV1,
    DirectCreatureEnvelopeFieldDifferenceV1, DirectCreatureEnvelopeFieldStateV1,
    DirectCreatureEnvelopeProjectionV1, DirectCreatureMeshProjectionV1,
    DirectCreatureModelProjectionV1, DirectCreatureNodeProjectionV1,
    DirectCreatureStateSkinProfileSummaryV1, DirectCreatureStructuralSummaryV1,
    EngineAnimationEnvelopeV1, EngineByteRangeV1, EngineControllerEnvelopeV1, EngineMeshEnvelopeV1,
    EngineNodeEnvelopeV1, EngineRawStreamEnvelopeV1, MdlRuntimeConformanceErrorV1,
    direct_creature_engine_envelope_digest_v1, direct_creature_envelope_differential_v1,
    direct_creature_envelope_projection_digest_v1, direct_creature_structural_summary_digest_v1,
    inspect_direct_creature_engine_envelope_v1, inspect_direct_creature_envelope_projection_v1,
    summarize_direct_creature_state_skin_profile_v1, summarize_direct_creature_structure_v1,
    verify_direct_creature_state_projection_v1,
    verify_direct_creature_state_projection_with_expected_provenance_v1,
};
pub use skin_deformation::{
    SkinDeformationNodeSampleV1, SkinDeformationSampleV1, SkinDeformationVertexSampleV1,
    evaluate_skin_deformation_v1,
};
pub(crate) use types::AnimationReport;
pub use types::{AabbEntryReport, AabbTreeReport, InspectionReport, NodeReport, ParserLimits};
pub use write_binary_mdl::{
    write_binary_mdl, write_binary_mdl_with_animations,
    write_binary_mdl_with_animations_and_supermodel, write_binary_mdl_with_supermodel,
    write_binary_tile_mdl_v1,
};
pub use writer_types::{
    BinaryMdlArtifactV1, M4SemanticProjectionV1, MdlAabbNodeLayoutV1, MdlAnimationClipLayoutV1,
    MdlAnimationClipV1, MdlAnimationEventV1, MdlAnimationInterpolationV1, MdlAnimationNodeLayoutV1,
    MdlAnimationSetV1, MdlAnimationTrackLayoutV1, MdlAnimationTrackPathV1, MdlAnimationTrackV1,
    MdlAnimationWriterReportV1, MdlFormatProfileV1, MdlLayoutReportV1, MdlMaterialTextureBindingV1,
    MdlMeshNodeLayoutV1, MdlRigNodeLayoutV1, MdlStateProjectionProfileV1,
    MdlStateProjectionProvenanceV1, MdlWriteError, MdlWriterDeviationV1, MdlWriterOptionsV1,
    MdlWriterReportV1, NWN_EE_MAX_MESH_INDEX_COUNT_V1, NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1,
};
