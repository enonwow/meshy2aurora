//! Product guardrails shared by every Aurora render-model route.
//!
//! These values are Meshy2Aurora policy, not claims about Aurora/NWN engine
//! limits. The binary MDL writer independently enforces its stricter format
//! boundary for each individual mesh stream.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::model_ir::AuroraModelIrV1;

/// One numeric source of truth for the accepted triangle budget of Creature,
/// Placeable and other render-model conversions.
///
/// Exactly 300,000 triangles are accepted. A total above this value is blocked
/// before conversion.
pub const AURORA_MODEL_TRIANGLE_BUDGET_V1: usize = 300_000;

/// Shared early warning threshold, derived from the one product budget.
pub const AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1: usize = AURORA_MODEL_TRIANGLE_BUDGET_V1 / 2;

/// Bounded raw/final envelope for the explicitly selected Meshy "100K"
/// Creature experiment.
///
/// Meshy treats 100,000 as a generation target and the canonical sources can
/// legitimately overshoot it. The experiment accepts that bounded 10% source
/// variance so the geometry sanitizer never has to misclassify valid
/// microtriangles merely to force the result below the requested target. This
/// is not a product budget and does not change `AURORA_MODEL_TRIANGLE_BUDGET_V1`.
pub const MESHY_CREATURE_P100K_EXPERIMENT_TRIANGLE_CEILING_V1: usize = 110_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelTriangleBudgetErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
    pub triangle_count: usize,
    pub triangle_budget: usize,
}

impl fmt::Display for ModelTriangleBudgetErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ModelTriangleBudgetErrorV1 {}

/// Applies the shared whole-model budget to an already materialized common IR.
///
/// GLB and Profile A admission use the same numeric constant earlier in the
/// pipeline. This guard closes direct package APIs that receive IR rather than
/// source GLB bytes.
pub fn validate_model_triangle_budget_v1(
    model: &AuroraModelIrV1,
) -> Result<usize, ModelTriangleBudgetErrorV1> {
    let mut triangle_count = 0usize;
    for (segment_index, segment) in model.segments.iter().enumerate() {
        if !segment.indices.len().is_multiple_of(3) {
            return Err(ModelTriangleBudgetErrorV1 {
                schema_version: 1,
                code: "M2A-MODEL-TRIANGLE-TOPOLOGY-INVALID".to_owned(),
                path: format!("model.segments[{segment_index}].indices"),
                message: "triangle-list index count must be divisible by three".to_owned(),
                triangle_count,
                triangle_budget: AURORA_MODEL_TRIANGLE_BUDGET_V1,
            });
        }
        triangle_count = triangle_count
            .checked_add(segment.indices.len() / 3)
            .ok_or_else(|| ModelTriangleBudgetErrorV1 {
                schema_version: 1,
                code: "M2A-MODEL-TRIANGLE-COUNT-OVERFLOW".to_owned(),
                path: "model.segments".to_owned(),
                message: "whole-model triangle count overflow".to_owned(),
                triangle_count,
                triangle_budget: AURORA_MODEL_TRIANGLE_BUDGET_V1,
            })?;
    }
    if triangle_count > AURORA_MODEL_TRIANGLE_BUDGET_V1 {
        return Err(ModelTriangleBudgetErrorV1 {
            schema_version: 1,
            code: "M2A-MODEL-TRIANGLE-BUDGET-EXCEEDED".to_owned(),
            path: "model.segments".to_owned(),
            message: format!(
                "model has {triangle_count} triangles; shared product budget is {AURORA_MODEL_TRIANGLE_BUDGET_V1}"
            ),
            triangle_count,
            triangle_budget: AURORA_MODEL_TRIANGLE_BUDGET_V1,
        });
    }
    Ok(triangle_count)
}
