use super::super::PMesh;
use super::PBuilder;
use lyon::lyon_tessellation::geometry_builder::simple_builder;
use lyon::math::Point;
use lyon::path::builder::NoAttributes;
use lyon::tessellation::*;

// TODO: allow other index sizes!

/// This structure wraps a `lyon::tesselation::StrokeTessellator` and adds functionality to apply transformations to the path being built.
pub struct PStroke {
    tessellator: StrokeTessellator,
    options: StrokeOptions,
    geometry: VertexBuffers<Point, u16>,
}

impl PStroke {
    /// Creates a new stroke tessellator with the given tolerance and width.
    pub fn new(width: f32, tol: f32) -> Self {
        // Will contain the result of the tessellation.
        let geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
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
    pub fn build(self, flat: bool) -> PMesh<u16> {
        PMesh::import_geometry(&self.geometry, flat, false)
    }
}
