use super::PMesh;
use crate::IndexType;
use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues, render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Copies the mesh into an existing bevy mesh.
    pub fn bevy_set(&self, mesh: &mut Mesh) {
        assert!(self.indices.iter_usize().all(|i| i < self.vertices.len()));

        assert!(
            mesh.primitive_topology() == PrimitiveTopology::TriangleList,
            "Only triangle lists are supported"
        );

        mesh.remove_indices();
        let mut attributes_to_remove = Vec::new();
        for (attr, _) in mesh.attributes() {
            attributes_to_remove.push(attr);
        }
        for attr in attributes_to_remove {
            mesh.remove_attribute(attr);
        }

        mesh.insert_indices(self.indices.get_bevy());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices.to_bevy());
        if let Some(uv) = &self.uv {
            assert!(self.vertices.len() == uv.len());
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(uv.clone()),
            );
        }

        if let Some(normals) = &self.normals {
            assert!(self.vertices.len() == normals.len());
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(normals.clone()),
            );
        } else if mesh.primitive_topology() == PrimitiveTopology::TriangleList {
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
        }

        /*if mesh.contains_attribute(Mesh::ATTRIBUTE_NORMAL) {
            // This will sometimes panic when the mesh is weird. Not very stable at all!
            if mesh.generate_tangents().is_err() {
                // TODO
                println!("WARN: Failed to generate tangents");
            }
        }*/
    }

    /// Creates a bevy mesh from the mesh.
    pub fn to_bevy(&self, usage: RenderAssetUsages) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage);
        self.bevy_set(&mut mesh);
        mesh
    }
}
