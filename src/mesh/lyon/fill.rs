use std::ops::Add;

use crate::IndexType;

use super::super::PMesh;
use super::builder::PBuilder;
use lyon::tessellation::geometry_builder::{MaxIndex, Positions};
use lyon::tessellation::VertexId;
use lyon::{
    math::Point,
    tessellation::{BuffersBuilder, FillBuilder, FillOptions, FillTessellator, VertexBuffers},
};

/// This structure wraps a `lyon::tesselation::FillTessellator` and adds functionality to apply transformations to the path being built.
pub struct PFill<T>
where
    T: IndexType,
{
    // TODO: reuse the Tesselator!
    tessellator: FillTessellator,
    options: FillOptions,
    geometry: VertexBuffers<Point, T>,
}

impl<T> PFill<T>
where
    T: Add + IndexType + From<VertexId> + MaxIndex,
{
    /// Creates a new fill tessellator with the given tolerance.
    pub fn new(tol: f32) -> Self {
        // Will contain the result of the tessellation.
        let geometry: VertexBuffers<Point, T> = VertexBuffers::new();
        let tessellator = FillTessellator::new();
        let options = FillOptions::tolerance(tol);

        PFill::<T> {
            tessellator,
            options,
            geometry,
        }
    }

    /// Draws the path using the given closure.
    pub fn draw<F>(&mut self, draw_commands: F) -> &mut Self
    where
        F: FnOnce(&mut PBuilder<FillBuilder>),
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
    pub fn fill<F>(&mut self, tol: f32, draw_commands: F) -> &mut PMesh<T>
    where
        F: FnOnce(&mut PBuilder<FillBuilder>),
    {
        let mut tessellator = PFill::<T>::new(tol);
        tessellator.draw(draw_commands);
        self.extend(&tessellator.build());
        self
    }
}
