//! Tile-domain SET/WOK/MDL/package resolver.
//!
//! Render geometry remains `AuroraModelIrV1` and binary emission remains in
//! the shared `mdl` module. This module owns only tile resource bindings.

mod package;
mod set;

pub use crate::walkmesh::TileSurfaceV1;
pub use package::*;
pub use set::*;
