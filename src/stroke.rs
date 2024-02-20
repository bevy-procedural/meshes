use super::mesh::MyMesh;
use lyon::lyon_tessellation::geometry_builder::simple_builder;
use lyon::math::Point;
use lyon::path::builder::NoAttributes;
use lyon::tessellation::*;


pub struct MyStroke {
    tessellator: StrokeTessellator,
    options: StrokeOptions,
    geometry: VertexBuffers<Point, u16>,
}

impl MyStroke {
    pub fn new(width: f32, tol: f32) -> Self {
        // Will contain the result of the tessellation.
        let geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let tessellator = StrokeTessellator::new();
        let options = StrokeOptions::tolerance(tol).with_line_width(width);

        MyStroke {
            tessellator,
            options,
            geometry,
        }
    }

    pub fn draw<F>(&mut self, draw_commands: F) -> &mut Self
    where
        F: FnOnce(&mut NoAttributes<StrokeBuilder>),
    {
        let mut geometry_builder = simple_builder(&mut self.geometry);
        let mut builder = self
            .tessellator
            .builder(&self.options, &mut geometry_builder);

        draw_commands(&mut builder);

        builder.build().unwrap();

        self
    }

    // consumes self (can't be called twice)
    pub fn build(self, flat: bool) -> MyMesh {
        MyMesh::import_geometry(&self.geometry, flat, false)
    }
}
