use super::super::PMesh;
use super::PBuilder;
use crate::IndexType;
use lyon::math::Point;
use lyon::tessellation::geometry_builder::{MaxIndex, Positions};
use lyon::tessellation::*;
use std::ops::Add;

// TODO: allow other index sizes!

/// This structure wraps a `lyon::tesselation::StrokeTessellator` and adds functionality to apply transformations to the path being built.
pub struct PStroke<T>
where
    T: Add + IndexType + From<VertexId> + MaxIndex,
{
    tessellator: StrokeTessellator,
    options: StrokeOptions,
    geometry: VertexBuffers<Point, T>,
}

impl<T> PStroke<T>
where
    T: Add + IndexType + From<VertexId> + MaxIndex,
{
    /// Creates a new stroke tessellator with the given tolerance and width.
    pub fn new(width: f32, tol: f32) -> Self {
        // Will contain the result of the tessellation.
        let geometry: VertexBuffers<Point, T> = VertexBuffers::new();
        let tessellator = StrokeTessellator::new();
        let options = StrokeOptions::tolerance(tol).with_line_width(width);

        PStroke {
            tessellator,
            options,
            geometry,
        }
    }

    /// Draws the path using the given closure.
    pub fn draw<F>(&mut self, draw_commands: F) -> &mut Self
    where
        F: FnOnce(&mut PBuilder<StrokeBuilder>),
    {
        // TODO: Improve performance: Can I use the BuffersBuilder to directly write to the mesh?
        let geometry_builder = &mut BuffersBuilder::new(&mut self.geometry, Positions);
        let builder = self.tessellator.builder(&self.options, geometry_builder);
        let mut my_builder = PBuilder::new(builder);
        draw_commands(&mut my_builder);
        my_builder.build().unwrap();
        self
    }

    /// Builds a PMesh object, consuming the tessellator.
    pub fn build(self) -> PMesh<T> {
        PMesh::import_geometry(&self.geometry, false)
    }
}

impl<T> PMesh<T>
where
    T: Add + IndexType + From<VertexId> + MaxIndex,
{
    /// Fills the path built in the closure and appends it to the mesh.
    pub fn stroke<F>(&mut self, width: f32, tol: f32, draw_commands: F) -> &mut PMesh<T>
    where
        F: FnOnce(&mut PBuilder<StrokeBuilder>),
    {
        let mut tessellator = PStroke::<T>::new(width, tol);
        tessellator.draw(draw_commands);
        self.extend(&tessellator.build());
        self
    }
}
