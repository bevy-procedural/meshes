//! This module contains the bevy-specific backend for the procedural mesh generation.

use super::IndexType;
use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
mod indices;
mod vertices;
pub use indices::PIndices;
pub use vertices::PVertices;
mod backend_bevy;
mod iter;
mod normals;
mod shapes;

#[cfg(feature = "meshopt")]
pub mod meshopt;

#[cfg(feature = "lyon")]
pub mod lyon;

/// A mesh with vertices, indices, uv coordinates and normals.
#[derive(Clone, Debug)]
pub struct PMesh<T>
where
    T: IndexType,
{
    // TODO: allow a dynamic number of attributes
    vertices: PVertices,
    indices: PIndices<T>,
    uv: Option<Vec<[f32; 2]>>,
    normals: Option<Vec<[f32; 3]>>,

    /// ### From meshopt-rs:
    ///
    /// On most hardware, indexed triangle lists are the most efficient way to drive the GPU.
    /// However, in some cases triangle strips might prove beneficial:
    ///
    ///  - On some older GPUs, triangle strips may be a bit more efficient to render
    ///  - On extremely memory constrained systems, index buffers for triangle strips could save a bit of memory
    ///
    /// \[...\] Typically you should expect triangle strips to have ca. 50-60% of indices compared to triangle lists
    /// (ca. 1.5-1.8 indices per triangle) and have ca. 5% worse ACMR. Note that triangle strips require restart
    /// index support for rendering; using degenerate triangles to connect strips is not supported.
    topology: PrimitiveTopology,
}

impl<T> Default for PMesh<T>
where
    T: IndexType,
{
    /// Creates a new empty mesh.
    fn default() -> Self {
        PMesh::new()
    }
}

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Creates a new empty mesh.
    pub fn new() -> Self {
        PMesh {
            vertices: PVertices::new(),
            indices: PIndices::<T>::new(),
            uv: Some(Vec::new()),
            normals: None,
            topology: PrimitiveTopology::TriangleList,
        }
    }

    /// Returns the vertices of the mesh.
    pub fn get_vertices(&self) -> &PVertices {
        &self.vertices
    }

    /// Returns the vertices of the mesh to be modified in-place.
    pub fn get_vertices_mut(&mut self) -> &mut PVertices {
        &mut self.vertices
    }

    /// Removes all duplicate indices by duplicating the vertices, uvs, and normals and replacing the indices with a linear sequence.
    pub fn duplicate(&mut self) -> &mut PMesh<T> {
        self.vertices = PVertices::build(
            self.indices
                .iter_usize()
                .map(|i| self.vertices[i])
                .collect(),
        );
        if let Some(uv) = &self.uv {
            self.uv = Some(self.indices.iter_usize().map(|i| uv[i]).collect());
        }
        if let Some(normals) = &self.normals {
            self.normals = Some(self.indices.iter_usize().map(|i| normals[i]).collect());
        }
        self.indices
            .reset_to_interval(T::new(0), T::new(self.indices.len()));
        self
    }

    /// Returns the Vec3 at the given index.
    pub fn vec3_at(&self, i: usize) -> Vec3 {
        Vec3::from(self.vertices.vec(i))
    }

    /// Creates a PMesh from vertices, indices, uvs, normals, and the topology.
    pub fn build_ex(
        vertices: Vec<[f32; 3]>,
        indices: Vec<T>,
        uv: Option<Vec<[f32; 2]>>,
        normals: Option<Vec<[f32; 3]>>,
        topology: PrimitiveTopology,
    ) -> Self {
        if let Some(uv) = &uv {
            assert!(vertices.len() == uv.len());
        }
        if let Some(normals) = &normals {
            assert!(vertices.len() == normals.len());
        }
        PMesh {
            vertices: PVertices::build(vertices),
            indices: PIndices::build(indices),
            uv,
            normals,
            topology,
        }
    }

    /// Creates a TriangleList-PMesh from vertices, indices, and uvs and converts the indices to the given index type.
    pub fn build(vertices: Vec<[f32; 3]>, indices: Vec<u32>, uv: Option<Vec<[f32; 2]>>) -> Self {
        PMesh::build_ex(
            vertices,
            indices.iter().map(|i| T::new(*i as usize)).collect(),
            uv,
            None,
            PrimitiveTopology::TriangleList,
        )
    }

    /// Appends another mesh to this one.
    pub fn extend(&mut self, m: &PMesh<T>) -> &mut PMesh<T> {
        // convert topology if necessary
        let mut indices = if m.topology != self.topology {
            m.clone().set_topology(self.topology).indices.clone()
        } else {
            m.indices.clone()
        };

        let offset = self.vertices.len();
        self.vertices.extend(&m.vertices);
        self.indices
            .extend(indices.map(|i: T| i.add(T::new(offset))));
        if let Some(uv) = &mut self.uv {
            if let Some(uv2) = &m.uv {
                uv.extend(uv2);
            } else {
                // TODO: println!("WARN: Appending mesh without uv");
                self.uv = None;
            }
        }
        self
    }

    /// Rotates the mesh around the y-axis.
    pub fn rotate_y(&mut self, angle: f32) -> &mut PMesh<T> {
        self.vertices.rotate_y(angle);
        self
    }

    /// Translates the mesh.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut PMesh<T> {
        self.vertices.translate(x, y, z);
        self
    }

    /// Scales the mesh.
    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut PMesh<T> {
        self.vertices.scale(x, y, z);
        self
    }

    /// Scales the mesh uniformly.
    pub fn scale_uniform(&mut self, x: f32) -> &mut PMesh<T> {
        self.vertices.scale(x, x, x);
        self
    }

    /// Convert the mesh to a different topology by adjusting the indices.
    pub fn set_topology(&mut self, topology: PrimitiveTopology) -> &mut PMesh<T> {
        if topology != self.topology {
            if self.topology == PrimitiveTopology::TriangleList
                && topology == PrimitiveTopology::TriangleStrip
            {
                self.indices = self.indices.triangle_list_to_triangle_strip();
            } else if self.topology == PrimitiveTopology::TriangleStrip
                && topology == PrimitiveTopology::TriangleList
            {
                self.indices = self.indices.triangle_strip_to_triangle_list();
            } else {
                panic!("Topology change not implemented yet");
            }
            self.topology = topology;
        }
        self
    }

    /// Tries to introduce more shared indices and remove empty triangles.
    /// UV maps might be broken.
    pub fn optimize(&mut self) -> &mut PMesh<T> {
        let mut new_indices = Vec::new();
        let mut new_vertices = Vec::new();
        let mut new_uv = Vec::new();
        let eps = 0.0001;

        for i in self.indices.iter_usize() {
            let v = self.vertices[i];

            if let Some(index) = new_vertices
                .iter()
                .position(|v2| Vec3::from(v).distance(Vec3::from(*v2)) < eps)
            {
                new_indices.push(T::new(index));
            } else {
                new_indices.push(T::new(new_vertices.len()));
                new_vertices.push(v);
                if let Some(v) = &self.uv {
                    new_uv.push(v[i]);
                }
            }
        }

        /*println!(
            "Optimized {} vertices to {}",
            self.vertices.len(),
            new_vertices.len()
        );*/

        self.vertices = PVertices::build(new_vertices);
        self.indices = PIndices::build(new_indices);
        self.uv = match new_uv.len() > 0 {
            true => Some(new_uv),
            false => None,
        };

        self
    }

    /// Adds backfaces to the mesh.
    pub fn add_backfaces(&mut self) -> &mut PMesh<T> {
        self.indices.add_backfaces();
        self
    }

    /// Flips the y and z coordinates of the vertices.
    ///
    /// This is useful to place a mesh "lying on the ground" in a 3D scene.
    pub fn flip_yz(&mut self) -> &mut PMesh<T> {
        self.vertices.flip_yz();
        self
    }
}
