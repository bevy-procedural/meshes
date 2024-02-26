#![allow(dead_code)]

//! ## Bevy Procedural: Meshes
//! This crate provides a set of procedural mesh generation tools for Bevy.

mod builder;
mod fill;
mod index_type;
mod indices;
mod mesh;
mod stroke;
mod vertices;
pub use builder::{PBuilder, Winding};
pub use fill::PFill;
pub use index_type::IndexType;
pub use indices::PIndices;
pub use mesh::PMesh;
pub use stroke::PStroke;
pub use vertices::PVertices;
