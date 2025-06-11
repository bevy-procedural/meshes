//! This module contains the bevy-specific backend for the procedural mesh generation.

use super::IndexType;
use bevy::prelude::*;
mod indices;
mod vertices;
pub use indices::PIndices;
pub use vertices::PVertices;
mod backend_bevy;
mod geometry;
mod iter;
mod normals;
mod operator;
mod shapes;
//mod optimize;

#[cfg(feature = "meshopt")]
pub mod meshopt;

#[cfg(feature = "lyon")]
pub mod lyon;

#[cfg(feature = "lyon")]
pub use lyon::Winding;

/// A mesh with vertices, indices of type T, uv coordinates and normals.
///
/// It will always use a triangle list topology, because on most hardware,
/// indexed triangle lists are more efficient than triangle strips (see meshopt-rs).
/// Lines and Points are not supported (use "stroke" and "circle" instead).
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

    /// Returns the indices of the mesh.
    pub fn get_indices(&self) -> &PIndices<T> {
        &self.indices
    }

    /// Returns the indices of the mesh to be modified in-place.
    pub fn get_indices_mut(&mut self) -> &mut PIndices<T> {
        &mut self.indices
    }

    /// Returns the UV coordinates of the mesh.
    pub fn get_uv(&self) -> Option<&Vec<[f32; 2]>> {
        self.uv.as_ref()
    }

    /// Returns the UV coordinates of the mesh to be modified in-place.
    pub fn get_uv_mut(&mut self) -> Option<&mut Vec<[f32; 2]>> {
        self.uv.as_mut()
    }

    /// Returns the normals of the mesh.
    pub fn get_normals(&self) -> Option<&Vec<[f32; 3]>> {
        self.normals.as_ref()
    }

    /// Returns the normals of the mesh to be modified in-place.
    pub fn get_normals_mut(&mut self) -> Option<&mut Vec<[f32; 3]>> {
        self.normals.as_mut()
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
        }
    }

    /// Creates a TriangleList-PMesh from vertices, indices, and uvs and converts the indices to the given index type.
    pub fn build(vertices: Vec<[f32; 3]>, indices: Vec<u32>, uv: Option<Vec<[f32; 2]>>) -> Self {
        PMesh::build_ex(
            vertices,
            indices.iter().map(|i| T::new(*i as usize)).collect(),
            uv,
            None,
        )
    }

    /// Appends another mesh to this one.
    pub fn extend(&mut self, m: &PMesh<T>) -> &mut PMesh<T> {
        let offset = self.vertices.len();
        self.vertices.extend(&m.vertices);
        self.indices
            .extend(m.clone().indices.map(|i: T| i.add(T::new(offset))));
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
