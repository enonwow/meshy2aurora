//! Deterministic auditing and stabilization of detached skinned accessories.
//!
//! Meshy assets can contain rigid-looking crystals, plates or ornaments as
//! disconnected geometric components while assigning their vertices to
//! several animated limbs. The geometry is valid, but linear blend skinning
//! then stretches the detached part into a wing-like artifact. This stage
//! discovers components after a spatial position weld (so UV/normal seams do
//! not create false islands), measures their deformation on source clips and
//! can replace an approved accessory's mixed weights with one explicit bone.
//! Auto mode requires measured non-rigid deformation; mixed weights alone are
//! reported but never rewritten.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::{
    mdl::{
        MdlAnimationClipV1, MdlAnimationInterpolationV1, MdlAnimationSetV1,
        MdlAnimationTrackPathV1, MdlAnimationTrackV1,
    },
    model_ir::{
        AuroraModelIrV1, AuroraModelNodeV1, AuroraModelSegmentV1, AuroraSegmentDeformationV1,
        AuroraVertexWeightsV1,
    },
};

const MAX_METRIC_VERTICES_V2: usize = 128;
const MAX_SAMPLE_TIMES_PER_CLIP_V2: usize = 256;
const MIN_ACCESSORY_TRIANGLES_V1: usize = 2;
const MAX_ACCESSORY_TRIANGLE_SHARE_V1: f64 = 0.25;
const STABLE_WEIGHT_EPSILON_V1: f32 = 1.0e-5;
const RISKY_DOMINANT_WEIGHT_SHARE_BELOW_V1: f32 = 0.98;
const RISKY_PAIR_DISTANCE_RATIO_ABOVE_V1: f32 = 1.05;
const RISKY_PAIR_DISTANCE_ERROR_ABOVE_V1: f32 = 0.05;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkinAccessoryStabilizationModeV1 {
    #[default]
    Auto,
    KeepSourceWeights,
    SelectBone,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkinAccessoryComponentBoneOverrideV2 {
    pub segment_index: usize,
    pub component_index: usize,
    pub bone_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkinAccessoryStabilizationOptionsV2 {
    pub schema_version: u32,
    #[serde(default)]
    pub mode: SkinAccessoryStabilizationModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_bone_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_bone_overrides: Vec<SkinAccessoryComponentBoneOverrideV2>,
}

impl Default for SkinAccessoryStabilizationOptionsV2 {
    fn default() -> Self {
        Self {
            schema_version: 2,
            mode: SkinAccessoryStabilizationModeV1::Auto,
            selected_bone_name: None,
            component_bone_overrides: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkinAccessoryComponentActionV1 {
    PrimaryBody,
    StableAccessory,
    Stabilized,
    KeptSourceWeights,
    UnchangedDetachedComponent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkinAccessoryDeformationMetricsV2 {
    pub sampled_clip_count: u32,
    pub sampled_pose_count: u32,
    pub sampled_vertex_count: u32,
    pub vertex_sampling_mode: String,
    pub time_sampling_truncated_clip_count: u32,
    pub max_pair_distance_ratio: f32,
    pub max_pair_distance_error: f32,
    pub min_axis_alignment: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkinAccessoryComponentReportV2 {
    pub segment_index: usize,
    pub component_index: usize,
    pub triangle_count: usize,
    pub vertex_count: usize,
    pub is_primary_body: bool,
    pub centroid: [f32; 3],
    pub active_bone_count: u32,
    pub dominant_bone_name: Option<String>,
    pub dominant_bone_share: f32,
    pub risk_reasons: Vec<String>,
    pub action: SkinAccessoryComponentActionV1,
    pub selected_bone_id: Option<u32>,
    pub selected_bone_name: Option<String>,
    pub changed_vertex_count: usize,
    pub before: SkinAccessoryDeformationMetricsV2,
    pub after: SkinAccessoryDeformationMetricsV2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkinAccessoryStabilizationReportV2 {
    pub schema_version: u32,
    pub mode: SkinAccessoryStabilizationModeV1,
    pub audited_clip_count: u32,
    pub weld_tolerance: f32,
    pub component_count: usize,
    pub detached_component_count: usize,
    pub risky_component_count: usize,
    pub stabilized_component_count: usize,
    pub changed_vertex_count: usize,
    pub components: Vec<SkinAccessoryComponentReportV2>,
    pub warnings: Vec<String>,
}

#[deprecated(note = "use SkinAccessoryStabilizationOptionsV2; serialized schema is version 2")]
pub type SkinAccessoryStabilizationOptionsV1 = SkinAccessoryStabilizationOptionsV2;
#[deprecated(note = "use SkinAccessoryDeformationMetricsV2; report schema is version 2")]
pub type SkinAccessoryDeformationMetricsV1 = SkinAccessoryDeformationMetricsV2;
#[deprecated(note = "use SkinAccessoryComponentReportV2; report schema is version 2")]
pub type SkinAccessoryComponentReportV1 = SkinAccessoryComponentReportV2;
#[deprecated(note = "use SkinAccessoryStabilizationReportV2; serialized schema is version 2")]
pub type SkinAccessoryStabilizationReportV1 = SkinAccessoryStabilizationReportV2;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinAccessoryStabilizationErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for SkinAccessoryStabilizationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for SkinAccessoryStabilizationErrorV1 {}

fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> SkinAccessoryStabilizationErrorV1 {
    SkinAccessoryStabilizationErrorV1 {
        schema_version: 1,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

#[derive(Clone, Debug)]
struct ComponentV1 {
    component_index: usize,
    triangle_indices: Vec<usize>,
    vertex_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Mat4([f64; 16]);

impl Mat4 {
    fn from_f32(value: [f32; 16]) -> Self {
        Self(value.map(f64::from))
    }

    fn mul(self, right: Self) -> Self {
        let mut output = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                output[column * 4 + row] = (0..4)
                    .map(|index| self.0[index * 4 + row] * right.0[column * 4 + index])
                    .sum();
            }
        }
        Self(output)
    }

    fn transform_point(self, point: [f64; 3]) -> [f64; 3] {
        [
            self.0[0] * point[0] + self.0[4] * point[1] + self.0[8] * point[2] + self.0[12],
            self.0[1] * point[0] + self.0[5] * point[1] + self.0[9] * point[2] + self.0[13],
            self.0[2] * point[0] + self.0[6] * point[1] + self.0[10] * point[2] + self.0[14],
        ]
    }

    fn inverse_affine(self) -> Option<Self> {
        let a00 = self.0[0];
        let a01 = self.0[4];
        let a02 = self.0[8];
        let a10 = self.0[1];
        let a11 = self.0[5];
        let a12 = self.0[9];
        let a20 = self.0[2];
        let a21 = self.0[6];
        let a22 = self.0[10];
        let determinant = a00 * (a11 * a22 - a12 * a21) - a01 * (a10 * a22 - a12 * a20)
            + a02 * (a10 * a21 - a11 * a20);
        if !determinant.is_finite() || determinant.abs() <= f64::EPSILON {
            return None;
        }
        let inverse_determinant = 1.0 / determinant;
        let b00 = (a11 * a22 - a12 * a21) * inverse_determinant;
        let b01 = (a02 * a21 - a01 * a22) * inverse_determinant;
        let b02 = (a01 * a12 - a02 * a11) * inverse_determinant;
        let b10 = (a12 * a20 - a10 * a22) * inverse_determinant;
        let b11 = (a00 * a22 - a02 * a20) * inverse_determinant;
        let b12 = (a02 * a10 - a00 * a12) * inverse_determinant;
        let b20 = (a10 * a21 - a11 * a20) * inverse_determinant;
        let b21 = (a01 * a20 - a00 * a21) * inverse_determinant;
        let b22 = (a00 * a11 - a01 * a10) * inverse_determinant;
        let translation = [self.0[12], self.0[13], self.0[14]];
        let inverse_translation = [
            -(b00 * translation[0] + b01 * translation[1] + b02 * translation[2]),
            -(b10 * translation[0] + b11 * translation[1] + b12 * translation[2]),
            -(b20 * translation[0] + b21 * translation[1] + b22 * translation[2]),
        ];
        Some(Self([
            b00,
            b10,
            b20,
            0.0,
            b01,
            b11,
            b21,
            0.0,
            b02,
            b12,
            b22,
            0.0,
            inverse_translation[0],
            inverse_translation[1],
            inverse_translation[2],
            1.0,
        ]))
    }
}

#[derive(Clone, Copy)]
struct TrsV1 {
    translation: [f64; 3],
    rotation: [f64; 4],
    scale: [f64; 3],
}

/// Audits every Skin segment and optionally replaces only approved detached
/// accessory weights. Geometry, hierarchy, materials, UVs, tangents and
/// animation tracks are read-only.
#[deprecated(note = "use audit_and_stabilize_skin_accessories_v2; report schema is version 2")]
pub fn audit_and_stabilize_skin_accessories_v1(
    model: &mut AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    options: &SkinAccessoryStabilizationOptionsV2,
) -> Result<SkinAccessoryStabilizationReportV2, SkinAccessoryStabilizationErrorV1> {
    audit_and_stabilize_skin_accessories_v2(model, animations, options)
}

pub fn audit_and_stabilize_skin_accessories_v2(
    model: &mut AuroraModelIrV1,
    animations: &MdlAnimationSetV1,
    options: &SkinAccessoryStabilizationOptionsV2,
) -> Result<SkinAccessoryStabilizationReportV2, SkinAccessoryStabilizationErrorV1> {
    validate_options(options)?;
    let bind_worlds = node_worlds(&model.nodes, None, 0.0)?;
    let node_names = model
        .nodes
        .iter()
        .map(|node| (node.id, node.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let explicit_bone = selected_bone(options, &model.nodes)?;
    let component_bone_overrides = selected_component_bones(options, &model.nodes)?;
    let weld_tolerance = model_weld_tolerance(model)?;
    let mut reports = Vec::new();
    let mut detached_component_count = 0usize;
    let mut risky_component_count = 0usize;
    let mut stabilized_component_count = 0usize;
    let mut changed_vertex_count = 0usize;
    let mut warnings = Vec::new();

    for segment_index in 0..model.segments.len() {
        if model.segments[segment_index].deformation != AuroraSegmentDeformationV1::Skin {
            continue;
        }
        validate_skin_segment(&model.segments[segment_index], segment_index)?;
        let components = spatial_welded_components(&model.segments[segment_index], weld_tolerance)?;
        if components.is_empty() {
            continue;
        }
        let primary_index = components
            .iter()
            .enumerate()
            .max_by_key(|(_, component)| {
                (
                    component.triangle_indices.len(),
                    component.vertex_indices.len(),
                    usize::MAX - component.component_index,
                )
            })
            .map(|(index, _)| index)
            .expect("components are nonempty");
        let segment_triangle_count = model.segments[segment_index].indices.len() / 3;

        for (list_index, component) in components.iter().enumerate() {
            let is_primary_body = list_index == primary_index;
            if !is_primary_body {
                detached_component_count += 1;
            }
            let centroid = component_centroid(&model.segments[segment_index], component);
            let weight_summary =
                summarize_component_weights(&model.segments[segment_index], component, &node_names);
            let before = component_deformation_metrics(
                &model.segments[segment_index],
                component,
                &model.nodes,
                &bind_worlds,
                animations,
            )?;
            let triangle_share =
                component.triangle_indices.len() as f64 / segment_triangle_count.max(1) as f64;
            let small_detached_candidate = !is_primary_body
                && component.triangle_indices.len() >= MIN_ACCESSORY_TRIANGLES_V1
                && triangle_share <= MAX_ACCESSORY_TRIANGLE_SHARE_V1;
            let mut risk_reasons = Vec::new();
            if weight_summary.uniform_hard_bone.is_none()
                && weight_summary.dominant_share < RISKY_DOMINANT_WEIGHT_SHARE_BELOW_V1
            {
                risk_reasons.push("MIXED_COMPONENT_WEIGHTS".to_owned());
            }
            let observed_non_rigid_deformation = before.max_pair_distance_ratio
                > RISKY_PAIR_DISTANCE_RATIO_ABOVE_V1
                || before.max_pair_distance_error > RISKY_PAIR_DISTANCE_ERROR_ABOVE_V1;
            if observed_non_rigid_deformation {
                risk_reasons.push("NON_RIGID_CLIP_DEFORMATION".to_owned());
            }
            let risky = small_detached_candidate
                && weight_summary.uniform_hard_bone.is_none()
                && observed_non_rigid_deformation;
            if risky {
                risky_component_count += 1;
            } else if small_detached_candidate
                && weight_summary.uniform_hard_bone.is_none()
                && risk_reasons
                    .iter()
                    .any(|reason| reason == "MIXED_COMPONENT_WEIGHTS")
            {
                warnings.push(format!(
                    "segment {segment_index} component {} has mixed weights but no sampled non-rigid deformation; Auto kept source weights",
                    component.component_index
                ));
            }

            let mut action = if is_primary_body {
                SkinAccessoryComponentActionV1::PrimaryBody
            } else if weight_summary.uniform_hard_bone.is_some() {
                SkinAccessoryComponentActionV1::StableAccessory
            } else {
                SkinAccessoryComponentActionV1::UnchangedDetachedComponent
            };
            let mut selected_bone_id = None;
            let mut selected_bone_name = None;
            let mut changed = 0usize;

            if risky {
                match options.mode {
                    SkinAccessoryStabilizationModeV1::KeepSourceWeights => {
                        action = SkinAccessoryComponentActionV1::KeptSourceWeights;
                    }
                    SkinAccessoryStabilizationModeV1::Auto
                    | SkinAccessoryStabilizationModeV1::SelectBone => {
                        let bone = match options.mode {
                            SkinAccessoryStabilizationModeV1::SelectBone => {
                                component_bone_overrides
                                    .get(&(segment_index, component.component_index))
                                    .copied()
                                    .or(explicit_bone)
                            }
                            SkinAccessoryStabilizationModeV1::Auto => auto_torso_bone(
                                &model.nodes,
                                &bind_worlds,
                                model.segments[segment_index].parent_node_id,
                                centroid,
                            ),
                            SkinAccessoryStabilizationModeV1::KeepSourceWeights => None,
                        };
                        if let Some(bone_id) = bone {
                            selected_bone_id = Some(bone_id);
                            selected_bone_name = node_names.get(&bone_id).cloned();
                            changed = stabilize_component_weights(
                                &mut model.segments[segment_index],
                                component,
                                bone_id,
                            );
                            action = SkinAccessoryComponentActionV1::Stabilized;
                            stabilized_component_count += 1;
                            changed_vertex_count += changed;
                        } else {
                            warnings.push(format!(
                                "segment {segment_index} component {} is risky but no {} bone was available",
                                component.component_index,
                                if options.mode == SkinAccessoryStabilizationModeV1::SelectBone {
                                    "explicitly selected"
                                } else {
                                    "stable torso"
                                }
                            ));
                        }
                    }
                }
            }
            let after = if changed > 0 {
                component_deformation_metrics(
                    &model.segments[segment_index],
                    component,
                    &model.nodes,
                    &bind_worlds,
                    animations,
                )?
            } else {
                before.clone()
            };
            reports.push(SkinAccessoryComponentReportV2 {
                segment_index,
                component_index: component.component_index,
                triangle_count: component.triangle_indices.len(),
                vertex_count: component.vertex_indices.len(),
                is_primary_body,
                centroid,
                active_bone_count: weight_summary.active_bone_count,
                dominant_bone_name: weight_summary.dominant_bone_name,
                dominant_bone_share: weight_summary.dominant_share,
                risk_reasons,
                action,
                selected_bone_id,
                selected_bone_name,
                changed_vertex_count: changed,
                before,
                after,
            });
        }
    }

    Ok(SkinAccessoryStabilizationReportV2 {
        schema_version: 2,
        mode: options.mode,
        audited_clip_count: animations.clips.len() as u32,
        weld_tolerance,
        component_count: reports.len(),
        detached_component_count,
        risky_component_count,
        stabilized_component_count,
        changed_vertex_count,
        components: reports,
        warnings,
    })
}

fn validate_options(
    options: &SkinAccessoryStabilizationOptionsV2,
) -> Result<(), SkinAccessoryStabilizationErrorV1> {
    if options.schema_version != 2 {
        return Err(error(
            "M6-SKIN-ACCESSORY-OPTIONS-SCHEMA",
            "skinAccessoryStabilization.schemaVersion",
            format!(
                "expected skin-accessory options schema 2, got {}",
                options.schema_version
            ),
        ));
    }
    match options.mode {
        SkinAccessoryStabilizationModeV1::SelectBone
            if options
                .selected_bone_name
                .as_deref()
                .is_none_or(str::is_empty)
                && options.component_bone_overrides.is_empty() =>
        {
            Err(error(
                "M6-SKIN-ACCESSORY-BONE-MISSING",
                "skinAccessoryStabilization.selectedBoneName",
                "SELECT_BONE mode requires a global bone name or at least one component override",
            ))
        }
        SkinAccessoryStabilizationModeV1::Auto
        | SkinAccessoryStabilizationModeV1::KeepSourceWeights
            if options.selected_bone_name.is_some()
                || !options.component_bone_overrides.is_empty() =>
        {
            Err(error(
                "M6-SKIN-ACCESSORY-BONE-UNEXPECTED",
                "skinAccessoryStabilization.selectedBoneName",
                "selectedBoneName and componentBoneOverrides are valid only in SELECT_BONE mode",
            ))
        }
        _ => {
            let mut component_keys = BTreeSet::new();
            for (index, component_override) in options.component_bone_overrides.iter().enumerate() {
                if component_override.bone_name.trim().is_empty() {
                    return Err(error(
                        "M6-SKIN-ACCESSORY-BONE-MISSING",
                        format!(
                            "skinAccessoryStabilization.componentBoneOverrides[{index}].boneName"
                        ),
                        "component override bone name must be nonempty",
                    ));
                }
                if !component_keys.insert((
                    component_override.segment_index,
                    component_override.component_index,
                )) {
                    return Err(error(
                        "M6-SKIN-ACCESSORY-COMPONENT-OVERRIDE-DUPLICATE",
                        format!("skinAccessoryStabilization.componentBoneOverrides[{index}]"),
                        "each segment/component pair may be overridden only once",
                    ));
                }
            }
            Ok(())
        }
    }
}

fn selected_bone(
    options: &SkinAccessoryStabilizationOptionsV2,
    nodes: &[AuroraModelNodeV1],
) -> Result<Option<u32>, SkinAccessoryStabilizationErrorV1> {
    if options.mode != SkinAccessoryStabilizationModeV1::SelectBone {
        return Ok(None);
    }
    let Some(requested) = options.selected_bone_name.as_deref() else {
        return Ok(None);
    };
    let matches = nodes
        .iter()
        .filter(|node| node.name.eq_ignore_ascii_case(requested))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node] => Ok(Some(node.id)),
        [] => Err(error(
            "M6-SKIN-ACCESSORY-BONE-NOT-FOUND",
            "skinAccessoryStabilization.selectedBoneName",
            format!("selected bone {requested:?} is absent"),
        )),
        _ => Err(error(
            "M6-SKIN-ACCESSORY-BONE-AMBIGUOUS",
            "skinAccessoryStabilization.selectedBoneName",
            format!("selected bone {requested:?} is not unique after ASCII case-fold"),
        )),
    }
}

fn selected_component_bones(
    options: &SkinAccessoryStabilizationOptionsV2,
    nodes: &[AuroraModelNodeV1],
) -> Result<BTreeMap<(usize, usize), u32>, SkinAccessoryStabilizationErrorV1> {
    let mut selected = BTreeMap::new();
    for (index, component_override) in options.component_bone_overrides.iter().enumerate() {
        let matches = nodes
            .iter()
            .filter(|node| {
                node.name
                    .eq_ignore_ascii_case(&component_override.bone_name)
            })
            .collect::<Vec<_>>();
        let bone_id = match matches.as_slice() {
            [node] => node.id,
            [] => {
                return Err(error(
                    "M6-SKIN-ACCESSORY-BONE-NOT-FOUND",
                    format!("skinAccessoryStabilization.componentBoneOverrides[{index}].boneName"),
                    format!(
                        "selected component bone {:?} is absent",
                        component_override.bone_name
                    ),
                ));
            }
            _ => {
                return Err(error(
                    "M6-SKIN-ACCESSORY-BONE-AMBIGUOUS",
                    format!("skinAccessoryStabilization.componentBoneOverrides[{index}].boneName"),
                    format!(
                        "selected component bone {:?} is not unique after ASCII case-fold",
                        component_override.bone_name
                    ),
                ));
            }
        };
        selected.insert(
            (
                component_override.segment_index,
                component_override.component_index,
            ),
            bone_id,
        );
    }
    Ok(selected)
}

fn model_weld_tolerance(model: &AuroraModelIrV1) -> Result<f32, SkinAccessoryStabilizationErrorV1> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    let mut count = 0usize;
    for position in model
        .segments
        .iter()
        .filter(|segment| segment.deformation == AuroraSegmentDeformationV1::Skin)
        .flat_map(|segment| &segment.positions)
    {
        if position.iter().any(|value| !value.is_finite()) {
            return Err(error(
                "M6-SKIN-ACCESSORY-POSITION",
                "model.segments.positions",
                "skin positions must be finite",
            ));
        }
        count += 1;
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    if count == 0 {
        return Ok(1.0e-6);
    }
    let diagonal = ((f64::from(max[0]) - f64::from(min[0])).powi(2)
        + (f64::from(max[1]) - f64::from(min[1])).powi(2)
        + (f64::from(max[2]) - f64::from(min[2])).powi(2))
    .sqrt();
    Ok((diagonal * 1.0e-6).max(1.0e-6) as f32)
}

fn validate_skin_segment(
    segment: &AuroraModelSegmentV1,
    segment_index: usize,
) -> Result<(), SkinAccessoryStabilizationErrorV1> {
    let vertex_count = segment.positions.len();
    if segment.indices.is_empty()
        || !segment.indices.len().is_multiple_of(3)
        || segment.weights.len() != vertex_count
        || segment.normals.len() != vertex_count
        || segment.uv0.len() != vertex_count
        || segment
            .tangents
            .as_ref()
            .is_some_and(|values| values.len() != vertex_count)
    {
        return Err(error(
            "M6-SKIN-ACCESSORY-SEGMENT",
            format!("model.segments[{segment_index}]"),
            "Skin segment arrays must describe complete indexed triangles",
        ));
    }
    if segment
        .indices
        .iter()
        .any(|index| usize::try_from(*index).map_or(true, |index| index >= vertex_count))
    {
        return Err(error(
            "M6-SKIN-ACCESSORY-INDEX",
            format!("model.segments[{segment_index}].indices"),
            "Skin segment index escapes the vertex arrays",
        ));
    }
    Ok(())
}

#[derive(Clone)]
struct UnionFindV1 {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFindV1 {
    fn new(count: usize) -> Self {
        Self {
            parent: (0..count).collect(),
            rank: vec![0; count],
        }
    }

    fn find(&mut self, value: usize) -> usize {
        if self.parent[value] != value {
            self.parent[value] = self.find(self.parent[value]);
        }
        self.parent[value]
    }

    fn union(&mut self, left: usize, right: usize) {
        let left = self.find(left);
        let right = self.find(right);
        if left == right {
            return;
        }
        let (parent, child) = if self.rank[left] > self.rank[right] {
            (left, right)
        } else if self.rank[right] > self.rank[left] {
            (right, left)
        } else {
            self.rank[left] = self.rank[left].saturating_add(1);
            (left, right)
        };
        self.parent[child] = parent;
    }
}

fn spatial_welded_components(
    segment: &AuroraModelSegmentV1,
    tolerance: f32,
) -> Result<Vec<ComponentV1>, SkinAccessoryStabilizationErrorV1> {
    let mut union = UnionFindV1::new(segment.positions.len());
    let mut cells = BTreeMap::<[i64; 3], Vec<usize>>::new();
    let inverse_tolerance = 1.0 / f64::from(tolerance);
    let tolerance_sq = f64::from(tolerance).powi(2);
    for (vertex_index, position) in segment.positions.iter().enumerate() {
        let cell = std::array::from_fn(|axis| {
            (f64::from(position[axis]) * inverse_tolerance).floor() as i64
        });
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                for offset_z in -1..=1 {
                    let neighbor = [
                        cell[0].saturating_add(offset_x),
                        cell[1].saturating_add(offset_y),
                        cell[2].saturating_add(offset_z),
                    ];
                    if let Some(candidates) = cells.get(&neighbor) {
                        for &candidate in candidates {
                            let other = segment.positions[candidate];
                            let distance_sq = (0..3)
                                .map(|axis| {
                                    (f64::from(position[axis]) - f64::from(other[axis])).powi(2)
                                })
                                .sum::<f64>();
                            if distance_sq <= tolerance_sq {
                                union.union(vertex_index, candidate);
                            }
                        }
                    }
                }
            }
        }
        cells.entry(cell).or_default().push(vertex_index);
    }
    for triangle in segment.indices.chunks_exact(3) {
        let vertices = [
            usize::try_from(triangle[0]).map_err(|_| {
                error(
                    "M6-SKIN-ACCESSORY-INDEX",
                    "model.segments.indices",
                    "vertex index does not fit this platform",
                )
            })?,
            usize::try_from(triangle[1]).map_err(|_| {
                error(
                    "M6-SKIN-ACCESSORY-INDEX",
                    "model.segments.indices",
                    "vertex index does not fit this platform",
                )
            })?,
            usize::try_from(triangle[2]).map_err(|_| {
                error(
                    "M6-SKIN-ACCESSORY-INDEX",
                    "model.segments.indices",
                    "vertex index does not fit this platform",
                )
            })?,
        ];
        union.union(vertices[0], vertices[1]);
        union.union(vertices[1], vertices[2]);
    }
    let mut by_root = BTreeMap::<usize, (Vec<usize>, BTreeSet<usize>)>::new();
    for (triangle_index, triangle) in segment.indices.chunks_exact(3).enumerate() {
        let first = usize::try_from(triangle[0]).expect("validated index fits usize");
        let root = union.find(first);
        let entry = by_root.entry(root).or_default();
        entry.0.push(triangle_index);
        for source_index in triangle {
            entry
                .1
                .insert(usize::try_from(*source_index).expect("validated index fits usize"));
        }
    }
    let mut components = by_root
        .into_values()
        .map(|(triangle_indices, vertex_indices)| ComponentV1 {
            component_index: 0,
            triangle_indices,
            vertex_indices: vertex_indices.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    components.sort_by_key(|component| component.triangle_indices[0]);
    for (component_index, component) in components.iter_mut().enumerate() {
        component.component_index = component_index;
    }
    Ok(components)
}

fn component_centroid(segment: &AuroraModelSegmentV1, component: &ComponentV1) -> [f32; 3] {
    let mut sum = [0.0_f64; 3];
    for &vertex_index in &component.vertex_indices {
        for (axis, total) in sum.iter_mut().enumerate() {
            *total += f64::from(segment.positions[vertex_index][axis]);
        }
    }
    let divisor = component.vertex_indices.len().max(1) as f64;
    [
        (sum[0] / divisor) as f32,
        (sum[1] / divisor) as f32,
        (sum[2] / divisor) as f32,
    ]
}

struct WeightSummaryV1 {
    active_bone_count: u32,
    dominant_bone_name: Option<String>,
    dominant_share: f32,
    uniform_hard_bone: Option<u32>,
}

fn summarize_component_weights(
    segment: &AuroraModelSegmentV1,
    component: &ComponentV1,
    names: &BTreeMap<u32, String>,
) -> WeightSummaryV1 {
    let mut totals = BTreeMap::<u32, f64>::new();
    let mut uniform_hard_bone = None;
    let mut uniform = true;
    for (component_vertex, &vertex_index) in component.vertex_indices.iter().enumerate() {
        let row = &segment.weights[vertex_index];
        let active = row
            .bone_node_ids
            .iter()
            .copied()
            .zip(row.values)
            .take(usize::from(row.influence_count))
            .filter_map(|(bone, value)| bone.map(|bone| (bone, value)))
            .filter(|(_, value)| *value > STABLE_WEIGHT_EPSILON_V1)
            .collect::<Vec<_>>();
        for (bone, value) in &active {
            *totals.entry(*bone).or_default() += f64::from(*value);
        }
        let hard = match active.as_slice() {
            [(bone, value)] if (*value - 1.0).abs() <= STABLE_WEIGHT_EPSILON_V1 => Some(*bone),
            _ => None,
        };
        if component_vertex == 0 {
            uniform_hard_bone = hard;
        } else if hard != uniform_hard_bone {
            uniform = false;
        }
    }
    if !uniform {
        uniform_hard_bone = None;
    }
    let total = totals.values().sum::<f64>();
    let dominant = totals
        .iter()
        .max_by(|left, right| left.1.total_cmp(right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(bone, value)| (*bone, *value));
    WeightSummaryV1 {
        active_bone_count: u32::try_from(totals.len()).unwrap_or(u32::MAX),
        dominant_bone_name: dominant.and_then(|(bone, _)| names.get(&bone).cloned()),
        dominant_share: dominant
            .map(|(_, value)| (value / total.max(f64::EPSILON)) as f32)
            .unwrap_or(0.0),
        uniform_hard_bone,
    }
}

fn stabilize_component_weights(
    segment: &mut AuroraModelSegmentV1,
    component: &ComponentV1,
    bone_id: u32,
) -> usize {
    let replacement = AuroraVertexWeightsV1 {
        bone_node_ids: [Some(bone_id), None, None, None],
        values: [1.0, 0.0, 0.0, 0.0],
        influence_count: 1,
    };
    let mut changed = 0usize;
    for &vertex_index in &component.vertex_indices {
        if segment.weights[vertex_index] != replacement {
            segment.weights[vertex_index] = replacement.clone();
            changed += 1;
        }
    }
    changed
}

fn auto_torso_bone(
    nodes: &[AuroraModelNodeV1],
    bind_worlds: &BTreeMap<u32, Mat4>,
    segment_parent_id: u32,
    centroid: [f32; 3],
) -> Option<u32> {
    let parent_inverse = bind_worlds.get(&segment_parent_id)?.inverse_affine()?;
    let centroid = centroid.map(f64::from);
    nodes
        .iter()
        .filter(|node| is_torso_bone_name(&node.name))
        .filter_map(|node| {
            let bone_world = *bind_worlds.get(&node.id)?;
            let bone_in_parent =
                parent_inverse.transform_point(bone_world.transform_point([0.0, 0.0, 0.0]));
            let distance_sq = (0..3)
                .map(|axis| (bone_in_parent[axis] - centroid[axis]).powi(2))
                .sum::<f64>();
            Some((distance_sq, normalized_name(&node.name), node.id))
        })
        .min_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        })
        .map(|(_, _, node_id)| node_id)
}

fn normalized_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_torso_bone_name(name: &str) -> bool {
    let name = normalized_name(name);
    ["spine", "chest", "torso", "hips", "pelvis"]
        .iter()
        .any(|token| name.contains(token))
}

fn component_deformation_metrics(
    segment: &AuroraModelSegmentV1,
    component: &ComponentV1,
    nodes: &[AuroraModelNodeV1],
    bind_worlds: &BTreeMap<u32, Mat4>,
    animations: &MdlAnimationSetV1,
) -> Result<SkinAccessoryDeformationMetricsV2, SkinAccessoryStabilizationErrorV1> {
    let sampled_vertices = sampled_component_vertices(segment, component);
    let bind_points = sampled_vertices
        .indices
        .iter()
        .map(|&vertex_index| segment.positions[vertex_index].map(f64::from))
        .collect::<Vec<_>>();
    let (axis_left, axis_right, axis_length) = longest_pair(&bind_points);
    let parent_world = *bind_worlds.get(&segment.parent_node_id).ok_or_else(|| {
        error(
            "M6-SKIN-ACCESSORY-PARENT",
            "model.segments.parentNodeId",
            "Skin segment parent is absent from the bind hierarchy",
        )
    })?;
    let parent_inverse = parent_world.inverse_affine().ok_or_else(|| {
        error(
            "M6-SKIN-ACCESSORY-BIND",
            "model.nodes.bindLocalMatrix",
            "Skin segment parent bind world is singular",
        )
    })?;
    let mut max_ratio = 1.0_f64;
    let mut max_error = 0.0_f64;
    let mut min_axis_alignment = 1.0_f64;
    let mut sampled_pose_count = 0u32;
    let mut sampled_clip_count = 0u32;
    let mut time_sampling_truncated_clip_count = 0u32;
    for clip in &animations.clips {
        let times = clip_sample_times(clip);
        if times.values.is_empty() {
            continue;
        }
        if times.truncated {
            time_sampling_truncated_clip_count =
                time_sampling_truncated_clip_count.saturating_add(1);
        }
        sampled_clip_count = sampled_clip_count.saturating_add(1);
        for time in times.values {
            sampled_pose_count = sampled_pose_count.saturating_add(1);
            let animated_worlds = node_worlds(nodes, Some(clip), time)?;
            let deformed = sampled_vertices
                .indices
                .iter()
                .map(|&vertex_index| {
                    deform_position(
                        segment.positions[vertex_index],
                        &segment.weights[vertex_index],
                        bind_worlds,
                        &animated_worlds,
                        parent_world,
                        parent_inverse,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut pose_distance_scales = Vec::new();
            for left in 0..bind_points.len() {
                for right in left + 1..bind_points.len() {
                    let bind_distance = distance(bind_points[left], bind_points[right]);
                    if bind_distance <= f64::from(f32::EPSILON) {
                        continue;
                    }
                    let deformed_distance = distance(deformed[left], deformed[right]);
                    pose_distance_scales.push(deformed_distance / bind_distance);
                }
            }
            // Remove one uniform pose scale before measuring shape change.
            // Meshy commonly carries a constant armature-scale track; that
            // changes every distance equally and is not the wing-like
            // non-rigid accessory deformation this audit is designed to find.
            pose_distance_scales.sort_by(f64::total_cmp);
            let pose_scale = pose_distance_scales
                .get(pose_distance_scales.len() / 2)
                .copied()
                .unwrap_or(1.0)
                .max(f64::EPSILON);
            for distance_scale in pose_distance_scales {
                let normalized_scale = distance_scale / pose_scale;
                let ratio = normalized_scale.max(1.0 / normalized_scale.max(f64::EPSILON));
                max_ratio = max_ratio.max(ratio);
                max_error = max_error.max((normalized_scale - 1.0).abs());
            }
            if axis_length > f64::from(f32::EPSILON) {
                let bind_axis =
                    normalized(subtract(bind_points[axis_right], bind_points[axis_left]));
                let deformed_axis = normalized(subtract(deformed[axis_right], deformed[axis_left]));
                if let (Some(bind_axis), Some(deformed_axis)) = (bind_axis, deformed_axis) {
                    min_axis_alignment =
                        min_axis_alignment.min(dot(bind_axis, deformed_axis).abs());
                } else {
                    min_axis_alignment = 0.0;
                }
            }
        }
    }
    Ok(SkinAccessoryDeformationMetricsV2 {
        sampled_clip_count,
        sampled_pose_count,
        sampled_vertex_count: u32::try_from(sampled_vertices.indices.len()).unwrap_or(u32::MAX),
        vertex_sampling_mode: sampled_vertices.mode.to_owned(),
        time_sampling_truncated_clip_count,
        max_pair_distance_ratio: max_ratio as f32,
        max_pair_distance_error: max_error as f32,
        min_axis_alignment: min_axis_alignment as f32,
    })
}

struct SampledComponentVerticesV2 {
    indices: Vec<usize>,
    mode: &'static str,
}

fn sampled_component_vertices(
    segment: &AuroraModelSegmentV1,
    component: &ComponentV1,
) -> SampledComponentVerticesV2 {
    let vertices = &component.vertex_indices;
    if vertices.len() <= MAX_METRIC_VERTICES_V2 {
        return SampledComponentVerticesV2 {
            indices: vertices.to_vec(),
            mode: "EXACT_ALL_VERTICES",
        };
    }
    let mut selected = BTreeSet::new();
    selected.insert(vertices[0]);
    for axis in 0..3 {
        if let Some(&minimum) = vertices.iter().min_by(|left, right| {
            segment.positions[**left][axis]
                .total_cmp(&segment.positions[**right][axis])
                .then_with(|| left.cmp(right))
        }) {
            selected.insert(minimum);
        }
        if let Some(&maximum) = vertices.iter().max_by(|left, right| {
            segment.positions[**left][axis]
                .total_cmp(&segment.positions[**right][axis])
                .then_with(|| right.cmp(left))
        }) {
            selected.insert(maximum);
        }
    }

    let active_bones = vertices
        .iter()
        .flat_map(|&vertex_index| {
            let weights = &segment.weights[vertex_index];
            weights
                .bone_node_ids
                .iter()
                .copied()
                .take(usize::from(weights.influence_count))
                .flatten()
        })
        .collect::<BTreeSet<_>>();
    for bone in active_bones {
        if selected.len() >= MAX_METRIC_VERTICES_V2 {
            break;
        }
        if let Some(&maximum) = vertices.iter().max_by(|left, right| {
            vertex_bone_weight(&segment.weights[**left], bone)
                .total_cmp(&vertex_bone_weight(&segment.weights[**right], bone))
                .then_with(|| right.cmp(left))
        }) {
            selected.insert(maximum);
        }
    }

    let bounds = component_position_bounds(segment, vertices);
    while selected.len() < MAX_METRIC_VERTICES_V2 {
        let next = vertices
            .iter()
            .copied()
            .filter(|vertex| !selected.contains(vertex))
            .map(|candidate| {
                let nearest = selected
                    .iter()
                    .map(|chosen| {
                        combined_vertex_feature_distance(segment, candidate, *chosen, bounds)
                    })
                    .fold(f64::INFINITY, f64::min);
                (candidate, nearest)
            })
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            })
            .map(|(candidate, _)| candidate);
        let Some(next) = next else {
            break;
        };
        selected.insert(next);
    }
    SampledComponentVerticesV2 {
        indices: selected.into_iter().collect(),
        mode: "GEOMETRY_AND_WEIGHT_DIVERSE_V2",
    }
}

fn vertex_bone_weight(weights: &AuroraVertexWeightsV1, bone: u32) -> f32 {
    weights
        .bone_node_ids
        .iter()
        .copied()
        .zip(weights.values)
        .take(usize::from(weights.influence_count))
        .filter_map(|(candidate, value)| (candidate == Some(bone)).then_some(value))
        .sum()
}

fn component_position_bounds(
    segment: &AuroraModelSegmentV1,
    vertices: &[usize],
) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for &vertex in vertices {
        for axis in 0..3 {
            let value = f64::from(segment.positions[vertex][axis]);
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    (min, max)
}

fn combined_vertex_feature_distance(
    segment: &AuroraModelSegmentV1,
    left: usize,
    right: usize,
    bounds: ([f64; 3], [f64; 3]),
) -> f64 {
    let spatial = (0..3)
        .map(|axis| {
            let extent = (bounds.1[axis] - bounds.0[axis]).max(f64::EPSILON);
            ((f64::from(segment.positions[left][axis]) - f64::from(segment.positions[right][axis]))
                / extent)
                .powi(2)
        })
        .sum::<f64>();
    let left_weights = &segment.weights[left];
    let right_weights = &segment.weights[right];
    let bones = left_weights
        .bone_node_ids
        .iter()
        .chain(&right_weights.bone_node_ids)
        .copied()
        .flatten()
        .collect::<BTreeSet<_>>();
    let weight = bones
        .into_iter()
        .map(|bone| {
            f64::from(
                vertex_bone_weight(left_weights, bone) - vertex_bone_weight(right_weights, bone),
            )
            .powi(2)
        })
        .sum::<f64>();
    spatial + weight
}

fn longest_pair(points: &[[f64; 3]]) -> (usize, usize, f64) {
    let mut selected = (0usize, 0usize, 0.0_f64);
    for left in 0..points.len() {
        for right in left + 1..points.len() {
            let candidate = distance(points[left], points[right]);
            if candidate > selected.2 {
                selected = (left, right, candidate);
            }
        }
    }
    selected
}

struct ClipSampleTimesV2 {
    values: Vec<f32>,
    truncated: bool,
}

fn clip_sample_times(clip: &MdlAnimationClipV1) -> ClipSampleTimesV2 {
    let mut times = clip
        .tracks
        .iter()
        .flat_map(|track| track.times_seconds.iter().copied())
        .filter(|time| time.is_finite() && *time >= 0.0 && *time <= clip.length_seconds)
        .collect::<Vec<_>>();
    times.extend([0.0, clip.length_seconds]);
    times.sort_by(f32::total_cmp);
    times.dedup_by(|left, right| left.to_bits() == right.to_bits());
    if times.len() <= MAX_SAMPLE_TIMES_PER_CLIP_V2 {
        return ClipSampleTimesV2 {
            values: times,
            truncated: false,
        };
    }
    ClipSampleTimesV2 {
        values: (0..MAX_SAMPLE_TIMES_PER_CLIP_V2)
            .map(|sample| {
                let index = sample * (times.len() - 1) / (MAX_SAMPLE_TIMES_PER_CLIP_V2 - 1);
                times[index]
            })
            .collect(),
        truncated: true,
    }
}

fn deform_position(
    position: [f32; 3],
    weights: &AuroraVertexWeightsV1,
    bind_worlds: &BTreeMap<u32, Mat4>,
    animated_worlds: &BTreeMap<u32, Mat4>,
    parent_world: Mat4,
    parent_inverse: Mat4,
) -> Result<[f64; 3], SkinAccessoryStabilizationErrorV1> {
    let source = position.map(f64::from);
    let mut output = [0.0_f64; 3];
    let mut total = 0.0_f64;
    for lane in 0..usize::from(weights.influence_count) {
        let bone = weights.bone_node_ids[lane].ok_or_else(|| {
            error(
                "M6-SKIN-ACCESSORY-WEIGHT",
                "model.segments.weights.boneNodeIds",
                "active Skin weight lane has no bone id",
            )
        })?;
        let value = f64::from(weights.values[lane]);
        if !value.is_finite() || value < 0.0 {
            return Err(error(
                "M6-SKIN-ACCESSORY-WEIGHT",
                "model.segments.weights.values",
                "Skin weight must be finite and nonnegative",
            ));
        }
        let bind_world = *bind_worlds.get(&bone).ok_or_else(|| {
            error(
                "M6-SKIN-ACCESSORY-WEIGHT",
                "model.segments.weights.boneNodeIds",
                format!("Skin weight references absent bone {bone}"),
            )
        })?;
        let animated_world = *animated_worlds.get(&bone).ok_or_else(|| {
            error(
                "M6-SKIN-ACCESSORY-ANIMATION",
                "animations",
                format!("animation hierarchy has no bone {bone}"),
            )
        })?;
        let inverse_bind = bind_world.inverse_affine().ok_or_else(|| {
            error(
                "M6-SKIN-ACCESSORY-BIND",
                "model.nodes.bindLocalMatrix",
                format!("bone {bone} bind world is singular"),
            )
        })?;
        let transformed = parent_inverse
            .mul(animated_world)
            .mul(inverse_bind)
            .mul(parent_world)
            .transform_point(source);
        for axis in 0..3 {
            output[axis] += transformed[axis] * value;
        }
        total += value;
    }
    if !total.is_finite() || total <= f64::EPSILON {
        return Err(error(
            "M6-SKIN-ACCESSORY-WEIGHT",
            "model.segments.weights",
            "Skin vertex has no positive weight",
        ));
    }
    for value in &mut output {
        *value /= total;
    }
    Ok(output)
}

fn node_worlds(
    nodes: &[AuroraModelNodeV1],
    clip: Option<&MdlAnimationClipV1>,
    time: f32,
) -> Result<BTreeMap<u32, Mat4>, SkinAccessoryStabilizationErrorV1> {
    let mut locals = BTreeMap::new();
    for node in nodes {
        if locals
            .insert(node.id, sampled_local_matrix(node, clip, time)?)
            .is_some()
        {
            return Err(error(
                "M6-SKIN-ACCESSORY-NODE-DUPLICATE",
                "model.nodes",
                format!("duplicate model node id {}", node.id),
            ));
        }
    }
    let mut worlds = BTreeMap::<u32, Mat4>::new();
    while worlds.len() < nodes.len() {
        let mut progressed = false;
        for node in nodes {
            if worlds.contains_key(&node.id) {
                continue;
            }
            let world = match node.parent_id {
                None => locals[&node.id],
                Some(parent) => {
                    let Some(parent_world) = worlds.get(&parent).copied() else {
                        continue;
                    };
                    parent_world.mul(locals[&node.id])
                }
            };
            worlds.insert(node.id, world);
            progressed = true;
        }
        if !progressed {
            return Err(error(
                "M6-SKIN-ACCESSORY-HIERARCHY",
                "model.nodes",
                "model hierarchy contains a cycle or missing parent",
            ));
        }
    }
    Ok(worlds)
}

fn sampled_local_matrix(
    node: &AuroraModelNodeV1,
    clip: Option<&MdlAnimationClipV1>,
    time: f32,
) -> Result<Mat4, SkinAccessoryStabilizationErrorV1> {
    let mut trs = decompose(Mat4::from_f32(node.bind_local_matrix))?;
    if let Some(clip) = clip {
        let tracks = clip
            .tracks
            .iter()
            .filter(|track| track.target_node_id == node.id)
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        for track in tracks {
            let path_key = match track.path {
                MdlAnimationTrackPathV1::Translation => 0u8,
                MdlAnimationTrackPathV1::Rotation => 1,
                MdlAnimationTrackPathV1::Scale => 2,
                MdlAnimationTrackPathV1::Weights => 3,
            };
            if !seen.insert(path_key) {
                return Err(error(
                    "M6-SKIN-ACCESSORY-TRACK-DUPLICATE",
                    format!("animations.clips.{}.tracks", clip.name),
                    format!("node {} has duplicate {:?} tracks", node.id, track.path),
                ));
            }
            let row = sample_track(track, time)?;
            match track.path {
                MdlAnimationTrackPathV1::Translation if row.len() == 3 => {
                    trs.translation = [f64::from(row[0]), f64::from(row[1]), f64::from(row[2])];
                }
                MdlAnimationTrackPathV1::Rotation if row.len() == 4 => {
                    trs.rotation = normalize_quaternion([
                        f64::from(row[0]),
                        f64::from(row[1]),
                        f64::from(row[2]),
                        f64::from(row[3]),
                    ])?;
                }
                MdlAnimationTrackPathV1::Scale if row.len() == 3 => {
                    trs.scale = [f64::from(row[0]), f64::from(row[1]), f64::from(row[2])];
                }
                MdlAnimationTrackPathV1::Scale if row.len() == 1 => {
                    trs.scale = [f64::from(row[0]); 3];
                }
                MdlAnimationTrackPathV1::Weights => {}
                path => {
                    return Err(error(
                        "M6-SKIN-ACCESSORY-TRACK",
                        format!("animations.clips.{}.tracks", clip.name),
                        format!("track {path:?} has an invalid value width"),
                    ));
                }
            }
        }
    }
    compose(trs)
}

fn sample_track(
    track: &MdlAnimationTrackV1,
    time: f32,
) -> Result<Vec<f32>, SkinAccessoryStabilizationErrorV1> {
    if track.times_seconds.is_empty()
        || track.times_seconds.len() != track.values.len()
        || track
            .times_seconds
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(error(
            "M6-SKIN-ACCESSORY-TRACK",
            "animations.tracks",
            "animation track times and values must be nonempty, aligned and strictly increasing",
        ));
    }
    if time <= track.times_seconds[0] {
        return Ok(track.values[0].clone());
    }
    if time >= *track.times_seconds.last().expect("nonempty") {
        return Ok(track.values.last().expect("nonempty").clone());
    }
    let upper = track
        .times_seconds
        .partition_point(|candidate| *candidate < time);
    let lower = upper.saturating_sub(1);
    if track.interpolation == MdlAnimationInterpolationV1::Step {
        return Ok(track.values[lower].clone());
    }
    let span = track.times_seconds[upper] - track.times_seconds[lower];
    let phase = (time - track.times_seconds[lower]) / span;
    let from = &track.values[lower];
    let to = &track.values[upper];
    if from.len() != to.len() {
        return Err(error(
            "M6-SKIN-ACCESSORY-TRACK",
            "animations.tracks.values",
            "animation key widths differ",
        ));
    }
    if track.path == MdlAnimationTrackPathV1::Rotation && from.len() == 4 {
        let from_row: [f32; 4] = from.clone().try_into().expect("four values");
        let to_row: [f32; 4] = to.clone().try_into().expect("four values");
        let from = normalize_quaternion(from_row.map(f64::from))?;
        let mut to = normalize_quaternion(to_row.map(f64::from))?;
        if dot4(from, to) < 0.0 {
            to = to.map(|value| -value);
        }
        let mixed = std::array::from_fn(|index| {
            from[index] * (1.0 - f64::from(phase)) + to[index] * f64::from(phase)
        });
        return Ok(normalize_quaternion(mixed)?
            .map(|value| value as f32)
            .to_vec());
    }
    Ok(from
        .iter()
        .zip(to)
        .map(|(from, to)| from + (to - from) * phase)
        .collect())
}

fn decompose(matrix: Mat4) -> Result<TrsV1, SkinAccessoryStabilizationErrorV1> {
    let translation = [matrix.0[12], matrix.0[13], matrix.0[14]];
    let mut scale = [0.0; 3];
    for (column, value) in scale.iter_mut().enumerate() {
        *value = (0..3)
            .map(|row| matrix.0[column * 4 + row].powi(2))
            .sum::<f64>()
            .sqrt();
        if !value.is_finite() || *value <= f64::EPSILON {
            return Err(error(
                "M6-SKIN-ACCESSORY-BIND",
                "model.nodes.bindLocalMatrix",
                "bind matrix has a non-finite or zero scale axis",
            ));
        }
    }
    let rotation_matrix = [
        matrix.0[0] / scale[0],
        matrix.0[1] / scale[0],
        matrix.0[2] / scale[0],
        matrix.0[4] / scale[1],
        matrix.0[5] / scale[1],
        matrix.0[6] / scale[1],
        matrix.0[8] / scale[2],
        matrix.0[9] / scale[2],
        matrix.0[10] / scale[2],
    ];
    Ok(TrsV1 {
        translation,
        rotation: matrix3_quaternion(rotation_matrix)?,
        scale,
    })
}

fn compose(trs: TrsV1) -> Result<Mat4, SkinAccessoryStabilizationErrorV1> {
    let [x, y, z, w] = normalize_quaternion(trs.rotation)?;
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;
    let rotation = [
        1.0 - 2.0 * (yy + zz),
        2.0 * (xy + wz),
        2.0 * (xz - wy),
        2.0 * (xy - wz),
        1.0 - 2.0 * (xx + zz),
        2.0 * (yz + wx),
        2.0 * (xz + wy),
        2.0 * (yz - wx),
        1.0 - 2.0 * (xx + yy),
    ];
    Ok(Mat4([
        rotation[0] * trs.scale[0],
        rotation[1] * trs.scale[0],
        rotation[2] * trs.scale[0],
        0.0,
        rotation[3] * trs.scale[1],
        rotation[4] * trs.scale[1],
        rotation[5] * trs.scale[1],
        0.0,
        rotation[6] * trs.scale[2],
        rotation[7] * trs.scale[2],
        rotation[8] * trs.scale[2],
        0.0,
        trs.translation[0],
        trs.translation[1],
        trs.translation[2],
        1.0,
    ]))
}

fn matrix3_quaternion(matrix: [f64; 9]) -> Result<[f64; 4], SkinAccessoryStabilizationErrorV1> {
    let m00 = matrix[0];
    let m01 = matrix[3];
    let m02 = matrix[6];
    let m10 = matrix[1];
    let m11 = matrix[4];
    let m12 = matrix[7];
    let m20 = matrix[2];
    let m21 = matrix[5];
    let m22 = matrix[8];
    let trace = m00 + m11 + m22;
    let quaternion = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, s / 4.0]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [s / 4.0, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, s / 4.0, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, s / 4.0, (m10 - m01) / s]
    };
    normalize_quaternion(quaternion)
}

fn normalize_quaternion(
    quaternion: [f64; 4],
) -> Result<[f64; 4], SkinAccessoryStabilizationErrorV1> {
    let length = quaternion
        .iter()
        .map(|value| value.powi(2))
        .sum::<f64>()
        .sqrt();
    if !length.is_finite() || length <= f64::EPSILON {
        return Err(error(
            "M6-SKIN-ACCESSORY-QUATERNION",
            "model.nodes/animations.rotation",
            "rotation quaternion must be finite and nonzero",
        ));
    }
    Ok(quaternion.map(|value| value / length))
}

fn distance(left: [f64; 3], right: [f64; 3]) -> f64 {
    (0..3)
        .map(|axis| (left[axis] - right[axis]).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|axis| left[axis] - right[axis])
}

fn normalized(value: [f64; 3]) -> Option<[f64; 3]> {
    let length = value.iter().map(|item| item.powi(2)).sum::<f64>().sqrt();
    (length.is_finite() && length > f64::EPSILON).then(|| value.map(|item| item / length))
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn dot4(left: [f64; 4], right: [f64; 4]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{ComponentV1, sampled_component_vertices};
    use crate::model_ir::{
        AuroraModelSegmentV1, AuroraSegmentDeformationV1, AuroraVertexWeightsV1,
    };

    fn hard_weight(bone: u32) -> AuroraVertexWeightsV1 {
        AuroraVertexWeightsV1 {
            bone_node_ids: [Some(bone), None, None, None],
            values: [1.0, 0.0, 0.0, 0.0],
            influence_count: 1,
        }
    }

    #[test]
    fn large_component_sampling_is_geometry_and_weight_diverse_and_deterministic() {
        let positions = (0..200)
            .map(|index| [index as f32, (index % 7) as f32, (index % 11) as f32])
            .collect::<Vec<_>>();
        let mut weights = vec![hard_weight(1); positions.len()];
        weights[157] = hard_weight(2);
        let segment = AuroraModelSegmentV1 {
            segment_id: 1,
            material_slot: 0,
            deformation: AuroraSegmentDeformationV1::Skin,
            parent_node_id: 0,
            cast_shadow: true,
            normals: vec![[0.0, 1.0, 0.0]; positions.len()],
            uv0: vec![[0.0, 0.0]; positions.len()],
            tangents: None,
            indices: vec![0, 1, 2],
            face_surface_ids: Vec::new(),
            positions,
            weights,
        };
        let component = ComponentV1 {
            component_index: 0,
            triangle_indices: vec![0],
            vertex_indices: (0..200).collect(),
        };

        let first = sampled_component_vertices(&segment, &component);
        let second = sampled_component_vertices(&segment, &component);

        assert_eq!(first.mode, "GEOMETRY_AND_WEIGHT_DIVERSE_V2");
        assert_eq!(first.indices.len(), 128);
        assert!(first.indices.contains(&157));
        assert_eq!(first.indices, second.indices);
    }
}
