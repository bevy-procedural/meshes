use super::builder::Builder;
use super::mesh::MyMesh;
use lyon::lyon_tessellation::geometry_builder::simple_builder;
use lyon::math::Point;
use lyon::tessellation::*;

pub struct MyFill {
    tessellator: FillTessellator,
    options: FillOptions,
    geometry: VertexBuffers<Point, u16>,
}

impl MyFill {
    pub fn new(tol: f32) -> Self {
        // Will contain the result of the tessellation.
        let geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let tessellator = FillTessellator::new();
        let options = FillOptions::tolerance(tol);

        MyFill {
            tessellator,
            options,
            geometry,
        }
    }

    pub fn draw<F>(&mut self, draw_commands: F) -> &mut Self
    where
        F: FnOnce(&mut Builder),
    {
        let mut geometry_builder = simple_builder(&mut self.geometry);
        let builder = self
            .tessellator
            .builder(&self.options, &mut geometry_builder);

        let mut my_builder = Builder::new(builder);
        draw_commands(&mut my_builder);

        my_builder.build().unwrap();

        self
    }

    // consumes self (can't be called twice)
    pub fn build(self, flat: bool) -> MyMesh {
        MyMesh::import_geometry(&self.geometry, flat, false)
    }
}
