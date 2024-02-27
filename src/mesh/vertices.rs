use super::PMesh;
use crate::IndexType;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use std::ops::Index;

/// A list of vertices.
#[derive(Clone, Debug)]
pub struct PVertices {
    vertices: Vec<[f32; 3]>,
}

impl Index<usize> for PVertices {
    type Output = [f32; 3];

    #[inline(always)]
    fn index<'a>(&'a self, i: usize) -> &'a [f32; 3] {
        &self.vertices[i]
    }
}

impl PVertices {
    /// Creates a new PVertices with an empty list of vertices.
    pub fn new() -> Self {
        PVertices {
            vertices: Vec::new(),
        }
    }

    /// Returns a reference to the vector of vertices.
    #[inline(always)]
    pub fn get_vertices(&self) -> &Vec<[f32; 3]> {
        &self.vertices
    }

    /// Returns a reference to the vector of vertices.
    #[inline(always)]
    pub fn get_vertices_mut(&mut self) -> &mut Vec<[f32; 3]> {
        // TODO: remove this
        &mut self.vertices
    }

    /// Builds a new PVertices with the given vector of vertices consuming the vector.
    pub fn build(vertices: Vec<[f32; 3]>) -> PVertices {
        // TODO: Remove this!
        PVertices {
            vertices: vertices.clone(),
        }
    }

    /// Rotates the vertices around the y axis by the given angle.
    pub fn rotate_y(&mut self, angle: f32) -> &mut PVertices {
        let sin = angle.sin();
        let cos = angle.cos();
        for v in &mut self.vertices {
            let x = v[0];
            let z = v[2];
            v[0] = x * cos - z * sin;
            v[2] = x * sin + z * cos;
        }
        self
    }

    /// Translates the vertices by the given vector.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut PVertices {
        for v in &mut self.vertices {
            v[0] += x;
            v[1] += y;
            v[2] += z;
        }
        self
    }

    /// Scales the vertices by the given vector.
    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut PVertices {
        for v in &mut self.vertices {
            v[0] *= x;
            v[1] *= y;
            v[2] *= z;
        }
        self
    }

    /// Returns the number of vertices in the PVertices.
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the length of the arc defined by the vertices (assuming they form a path)
    pub fn arc_len(&self) -> f32 {
        let mut len = 0.0;
        for i in 0..self.len() - 1 {
            len += (self.vec(i) - self.vec(i + 1)).length();
        }
        len
    }

    /// Appends more vertices to the vector of vertices.
    pub fn extend(&mut self, other: &PVertices) {
        self.vertices.extend(other.vertices.clone());
    }

    /// Returns the vertex at the given index.
    pub fn vec(&self, i: usize) -> Vec3 {
        assert!(i < self.len());
        Vec3::new(
            self.vertices[i][0],
            self.vertices[i][1],
            self.vertices[i][2],
        )
    }

    /// Extrudes the vertices in the given direction.
    pub fn extrude<T>(&mut self, direction: Vec3) -> PMesh<T>
    where
        T: IndexType,
    {
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let offset = self.len() as u32;
        for v in &self.vertices {
            vertices.push([v[0], v[1], v[2]]);
            vertices.push([
                v[0] + direction[0],
                v[1] + direction[1],
                v[2] + direction[2],
            ]);
        }

        let max_index = offset * 2;
        for i in 0..offset {
            let i = (2 * i) as u32;
            indices.extend(vec![
                (i + 0) % max_index,
                (i + 1) % max_index,
                (i + 3) % max_index,
                (i + 0) % max_index,
                (i + 3) % max_index,
                (i + 2) % max_index,
            ]);
        }

        let mut uv: Vec<[f32; 2]> = Vec::new();
        {
            let arc_len = self.arc_len();
            let mut part_arc_len = 0.0;
            uv.push([0.0, 0.0]);
            uv.push([1.0, 0.0]);
            for i in 1..offset {
                part_arc_len += (self.vec((i - 1) as usize) - self.vec((i) as usize)).length();
                let prog = part_arc_len / arc_len;
                uv.push([0.0, prog]);
                uv.push([1.0, prog]);
                //println!("{} {}", part_arc_len, arc_len);
            }
        }

        /*let uv = vertices
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let prog = (i as f32) / (offset as f32);
            if i % 2 == 0 {
                [0.0, prog]
            } else {
                [1.0, prog]
            }
        })
        .collect();*/

        PMesh::build(vertices, indices, Some(uv))
    }

    /// Sorts the vertices in clockwise order around their average.
    pub fn sort_clockwise(&mut self) -> PVertices {
        let mut center = Vec3::new(0.0, 0.0, 0.0);
        for v in &self.vertices {
            center += Vec3::new(v[0], v[1], v[2]);
        }
        center /= self.len() as f32;

        let mut angles: Vec<(f32, usize)> = Vec::new();
        for (i, v) in self.vertices.iter().enumerate() {
            let mut v = Vec3::new(v[0], v[1], v[2]);
            v -= center;
            angles.push((v.y.atan2(v.x), i));
        }

        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        for (_, i) in angles {
            vertices.push(self.vertices[i]);
        }

        self.vertices = vertices;

        self.clone()
    }

    /// Converts the PVertices to a Bevy VertexAttributeValues.
    pub fn to_bevy(&self) -> VertexAttributeValues {
        VertexAttributeValues::Float32x3(self.vertices.clone())
    }

    /// Flips the y and z coordinates of the vertices.
    pub fn flip_yz(&mut self) -> &mut PVertices {
        for v in &mut self.vertices {
            let y = v[1];
            v[1] = v[2];
            v[2] = y;
        }
        self
    }
}
