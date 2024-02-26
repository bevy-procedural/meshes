use super::super::{indices::PIndices, vertices::PVertices, IndexType, PMesh};
use memoffset::offset_of;
use meshopt::{typed_to_bytes, VertexDataAdapter};

pub fn get_adapter(vertices: &Vec<meshopt::Vertex>) -> VertexDataAdapter {
    let position_offset = offset_of!(meshopt::Vertex, p);
    let vertex_stride = std::mem::size_of::<meshopt::Vertex>();
    let vertex_data = typed_to_bytes(&vertices);

    VertexDataAdapter::new(vertex_data, vertex_stride, position_offset)
        .expect("failed to create vertex data reader")
}

pub struct MeshoptMesh {
    pub vertices: Vec<meshopt::Vertex>,
    pub indices: Vec<u32>,
}

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Converts the mesh into meshopt data.
    pub fn to_meshopt_data(&self) -> MeshoptMesh {
        let mut vertices = Vec::new();
        for i in 0..self.vertices.len() {
            vertices.push(meshopt::Vertex {
                p: [
                    self.vertices[i][0],
                    self.vertices[i][1],
                    self.vertices[i][2],
                ],
                n: [0.0, 0.0, 0.0],
                t: [0.0, 0.0],
            });
        }

        let indices = self
            .indices
            .get_indices()
            .iter()
            .map(|x| x.index() as u32)
            .collect();

        return MeshoptMesh { vertices, indices };
    }

    /// Imports the meshopt data into the mesh.
    pub fn import_meshopt_data(&mut self, mesh: &MeshoptMesh) {
        self.vertices = PVertices::build(
            mesh.vertices
                .iter()
                .map(|x| [x.p[0], x.p[1], x.p[2]])
                .collect(),
        );
        self.indices = PIndices::build(mesh.indices.iter().map(|x| T::new(*x as usize)).collect());
        self.normals = None;
        self.uv = None;
    }
}
