//! Shared walkmesh primitives and the tile-specific ASCII WOK envelope.
//!
//! Placeable PWK and tile WOK deliberately share face/adjacency validation,
//! but retain separate runtime envelopes and resource types.

mod aabb_tree;
mod ascii_common;
mod tile_navigation_ir;
mod tile_wok;

pub use aabb_tree::{AabbEntryV1, AabbTreeV1, build_aabb_tree_v1, validate_aabb_tree_v1};
pub use ascii_common::WalkmeshFaceV1;
pub(crate) use ascii_common::format_placeable_f32;
pub use tile_navigation_ir::{
    TILE_FOOTPRINT_HALF_EXTENT_V1, TILE_FOOTPRINT_SIZE_V1, TileNavigationIrV1, TileSurfaceV1,
    flat_tile_navigation_v1, validate_tile_navigation_v1,
};
pub use tile_wok::{
    TileWokArtifactV1, TileWokInspectionV1, TileWokReportV1, inspect_ascii_tile_wok_v1,
    write_ascii_tile_wok_v1,
};

use std::fmt;

use serde::{Deserialize, Serialize};

pub const TILE_WALKMESH_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileWalkmeshErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for TileWalkmeshErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TileWalkmeshErrorV1 {}

pub(crate) fn error(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> TileWalkmeshErrorV1 {
    TileWalkmeshErrorV1 {
        schema_version: TILE_WALKMESH_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}
