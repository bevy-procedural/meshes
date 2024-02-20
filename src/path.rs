use super::mesh::MyMesh;
use bevy::prelude::*;
use lyon::math::Angle;

#[derive(Clone, Debug)]
pub struct MyPath {
    pub vertices: Vec<[f32; 3]>,
}

impl MyPath {
    pub fn new() -> Self {
        MyPath {
            vertices: Vec::new(),
        }
    }

    pub fn build(vertices: Vec<[f32; 3]>) -> MyPath {
        MyPath {
            vertices: vertices.clone(),
        }
    }

    pub fn append(&mut self, m: &MyPath) {
        self.vertices.extend(m.vertices.clone());
    }

    pub fn rotate_y(&mut self, angle: Angle) -> &mut MyPath {
        let sin = angle.get().sin();
        let cos = angle.get().cos();
        for v in &mut self.vertices {
            let x = v[0];
            let z = v[2];
            v[0] = x * cos - z * sin;
            v[2] = x * sin + z * cos;
        }
        self
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut MyPath {
        for v in &mut self.vertices {
            v[0] += x;
            v[1] += y;
            v[2] += z;
        }
        self
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut MyPath {
        for v in &mut self.vertices {
            v[0] *= x;
            v[1] *= y;
            v[2] *= z;
        }
        self
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn arc_len(&self) -> f32 {
        let mut len = 0.0;
        for i in 0..self.len() - 1 {
            len += (self.vec(i) - self.vec(i + 1)).length();
        }
        len
    }

    pub fn extend(&mut self, other: MyPath) {
        self.vertices.extend(other.vertices);
    }

    pub fn vec(&self, i: usize) -> Vec3 {
        assert!(i < self.len());
        Vec3::new(
            self.vertices[i][0],
            self.vertices[i][1],
            self.vertices[i][2],
        )
    }

    pub fn extrude(&mut self, direction: Vec3) -> MyMesh {
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

        MyMesh::build(vertices, indices, Some(uv))
    }

    pub fn sort_clockwise(&mut self) -> MyPath {
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

}
