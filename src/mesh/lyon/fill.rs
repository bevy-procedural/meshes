use super::builder::PBuilder;
use super::super::PMesh;
use lyon::{
    lyon_tessellation::geometry_builder::simple_builder,
    math::Point,
    tessellation::{FillBuilder, FillOptions, FillTessellator, VertexBuffers},
};

// TODO: allow other index sizes!

/// This structure wraps a `lyon::tesselation::FillTessellator` and adds functionality to apply transformations to the path being built.
pub struct PFill {
    tessellator: FillTessellator,
    options: FillOptions,
    geometry: VertexBuffers<Point, u16>,
}

impl PFill {
    /// Creates a new fill tessellator with the given tolerance.
    pub fn new(tol: f32) -> Self {
        // Will contain the result of the tessellation.
        let geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let tessellator = FillTessellator::new();
        let options = FillOptions::tolerance(tol);

        PFill {
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
        let mut geometry_builder = simple_builder(&mut self.geometry);
        let builder = self
            .tessellator
            .builder(&self.options, &mut geometry_builder);

        let mut my_builder = PBuilder::new(builder);
        draw_commands(&mut my_builder);

        my_builder.build().unwrap();

        self
    }

    /// Builds a PMesh object, consuming the tessellator.
    pub fn build<T>(self, flat: bool) -> PMesh<u16> {
        PMesh::import_geometry(&self.geometry, flat, false)
    }
}
