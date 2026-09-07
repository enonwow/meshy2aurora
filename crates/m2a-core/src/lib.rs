//! Format-safe core for the standalone Meshy-to-Aurora pipeline.

pub mod animated_donor;
pub mod animated_donor_candidate;
pub mod aurora_material;
pub mod aurora_material_bake;
pub mod aurora_material_package;
#[cfg(feature = "legacy-c-wolf-demo")]
pub mod c_wolf_rig;
pub mod creature_equipment;
pub mod creature_product;
pub mod creature_visibility_gate;
pub mod direct_creature_animation;
pub mod direct_creature_contract;
pub mod erf;
pub mod gff;
pub mod glb;
pub mod hak;
pub mod hierarchy_candidate;
pub mod hierarchy_experiment;
pub mod hook_horror_clone_diagnostic;
pub mod item;
pub mod item_icon;
pub mod item_material_mapping;
pub mod item_package;
pub mod item_part;
pub mod item_uti;
pub mod m7_corpus;
pub mod mdl;
pub mod model_components;
pub mod model_ir;
pub mod model_limits;
pub mod model_material_capabilities;
pub mod model_material_e2e;
pub mod model_material_separation;
pub mod model_material_uv_projection;
pub mod model_part;
pub mod model_pipeline;
pub mod model_segmentation;
pub mod model_source_quality;
pub mod model_texture_authoring;
pub mod mtr;
pub mod owned_fixture;
pub mod package;
pub mod placeable;
pub mod placeable_authoring;
pub mod placeable_collision;
pub mod placeable_texture;
pub mod plt;
pub mod profile_a;
pub mod proof_module;
pub mod reference_proof;
pub mod reference_supermodel;
pub mod reference_supermodel_admission;
pub mod reference_supermodel_authoring;
pub mod reference_supermodel_bind_pose;
pub mod reference_supermodel_generic;
pub mod reference_supermodel_motion;
pub mod reference_supermodel_product;
pub mod reference_supermodel_skinning;
pub mod reference_supermodel_structure;
pub mod reference_supermodel_surface_anatomy;
pub mod runtime_evidence;
pub mod skin_accessory;
pub mod supermodel_catalog;
pub mod tga;
pub mod tile;
pub mod tri_control_diagnostic;
pub mod tri_control_hak;
pub mod two_da;
pub mod txi;
pub mod walkmesh;

pub use mdl::{
    BinaryMdlArtifactV1, DirectCreatureEngineEnvelopeV1, DirectCreatureEngineEnvelopeVerdictV1,
    DirectCreatureEnvelopeDifferenceClassV1, DirectCreatureEnvelopeDifferentialV1,
    DirectCreatureEnvelopeFieldAssessmentV1, DirectCreatureEnvelopeFieldDifferenceV1,
    DirectCreatureEnvelopeFieldStateV1, DirectCreatureEnvelopeProjectionV1,
    DirectCreatureStructuralSummaryV1, InspectionReport, MdlFormatProfileV1,
    MdlMaterialTextureBindingV1, MdlRuntimeConformanceErrorV1, MdlStateProjectionProfileV1,
    MdlStateProjectionProvenanceV1, MdlWriteError, MdlWriterOptionsV1, ParserLimits,
    direct_creature_engine_envelope_digest_v1, direct_creature_envelope_differential_v1,
    direct_creature_envelope_projection_digest_v1, direct_creature_structural_summary_digest_v1,
    inspect_binary_mdl, inspect_binary_mdl_with_limits, inspect_direct_creature_engine_envelope_v1,
    inspect_direct_creature_envelope_projection_v1, summarize_direct_creature_structure_v1,
    verify_direct_creature_state_projection_v1,
    verify_direct_creature_state_projection_with_expected_provenance_v1, write_binary_mdl,
};
pub use model_ir::{
    AuroraMaterialSourceBindingV1, AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1,
    AuroraSegmentDeformationV1, AuroraVertexWeightsV1,
};
pub use model_limits::{AURORA_MODEL_TRIANGLE_BUDGET_V1, AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1};
pub use reference_proof::{
    CapabilityResult, CapabilityStatus, ExecutionMetadata, HashAlgorithm, InputFingerprint,
    InvariantResult, InvariantStatus, REFERENCE_PROOF_SCHEMA_VERSION, ReaderIdentity,
    ReferenceCapability, ReferenceIdentity, ReferenceManifest, ReferenceManifestEntry,
    ReferenceProofError, ReferenceProofPacket, ReferenceSource, build_reference_proof_packet,
};

pub mod reference_surface_guides;

pub mod reference_source_frame;
