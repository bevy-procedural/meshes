//! This module contains the implementation of the lyon mesh import functions.

use super::{IndexType, PMesh};
use lyon::{lyon_tessellation::VertexBuffers, math::Point};
mod builder;
mod fill;
mod stroke;
pub use builder::{PBuilder, PathBuilder};
pub use fill::{FillBuilder, PFill};
pub use lyon::path::Winding;
pub use stroke::{PStroke, StrokeBuilder};

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Imports a mesh from a lyon VertexBuffers.
    pub fn import_geometry(geometry: &VertexBuffers<Point, T>, normalize_uv: bool) -> PMesh<T>
    where
        T: IndexType,
    {
        let vertices: Vec<[f32; 3]> = geometry.vertices.iter().map(|v| [v.x, v.y, 0.0]).collect();
        let indices = geometry.indices.clone().iter().cloned().collect();

        let mut uv_x_scale = 1.0;
        let mut uv_y_scale = 1.0;

        if normalize_uv {
            //println!("x_min: {}, x_max: {}, y_min: {}, y_max: {}", x_min, x_max, y_min, y_max);
            let (x_min, x_max, y_min, y_max) = get_bounding_rect(geometry);
            uv_x_scale = x_max - x_min;
            uv_y_scale = y_max - y_min;
        }
        let uv: Option<Vec<[f32; 2]>> = Some(
            geometry
                .vertices
                .iter()
                .map(|v| [v.x / uv_x_scale, v.y / uv_y_scale])
                .collect(),
        );
        PMesh::build_ex(vertices, indices, uv, None)
    }
}

fn get_bounding_rect<T>(buf: &VertexBuffers<Point, T>) -> (f32, f32, f32, f32) {
    let mut x_min = std::f32::MAX;
    let mut x_max = std::f32::MIN;
    let mut y_min = std::f32::MAX;
    let mut y_max = std::f32::MIN;
    for v in &buf.vertices {
        if v.x < x_min {
            x_min = v.x;
        }
        if v.x > x_max {
            x_max = v.x;
        }
        if v.y < y_min {
            y_min = v.y;
        }
        if v.y > y_max {
            y_max = v.y;
        }
    }

    (x_min, x_max, y_min, y_max)
}
