//! A collection of primitive shapes

use super::{IndexType, PMesh};
use bevy::prelude::*;

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Creates a rectangle mesh.
    pub fn rect(width: f32, height: f32) -> PMesh<T> {
        PMesh::build(
            vec![
                [0.0, 0.0, 0.0],
                [width, 0.0, 0.0],
                [width, height, 0.0],
                [0.0, height, 0.0],
            ],
            vec![0, 1, 2, 0, 2, 3],
            Some(vec![
                [0.0, 0.0],
                [width, 0.0],
                [width, height],
                [0.0, height],
            ]),
        )
    }

    /// Creates a rectangle mesh centered at the origin.
    pub fn rect_c(width: f32, height: f32) -> PMesh<T> {
        let w = width * 0.5;
        let h = height * 0.5;
        PMesh::build(
            vec![[-w, -h, 0.0], [w, -h, 0.0], [w, h, 0.0], [-w, h, 0.0]],
            vec![0, 1, 2, 0, 2, 3],
            Some(vec![
                [0.0, 0.0],
                [width, 0.0],
                [width, height],
                [0.0, height],
            ]),
        )
    }

    /// Creates a triangle fan where the first vertex is the center.
    pub fn fan(vertices: Vec<[f32; 3]>, uv_options: Option<(f32, f32)>) -> PMesh<T> {
        // automatically reduce vertices if there are duplicates
        let vertices = simplify_vertices(vertices);

        // the mesh is degenerate - drop it!
        if vertices.len() < 3 {
            return PMesh::new();
        }

        let mut indices: Vec<u32> = Vec::new();
        for i in 1..vertices.len() - 1 {
            indices.push(0);
            indices.push(i as u32);
            indices.push(i as u32 + 1);
        }

        let uv = generate_uv_for_fan(&vertices, uv_options);
        PMesh::build(vertices, indices, uv)
    }

    /// Creates a hexagon mesh.
    pub fn polygon(radius: f32, sides: usize) -> PMesh<T> {
        let mut v = Vec::new();
        for i in 3..sides {
            let angle = ((sides - i) as f32 + 0.5) * std::f32::consts::PI / 3.0;
            v.push([radius * angle.cos(), radius * angle.sin(), 0.0]);
        }
        PMesh::fan(v, Some((1.0, 0.0)))
    }

    /// Creates a triangle mesh.
    pub fn triangle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> PMesh<T> {
        PMesh::build(
            vec![a, b, c],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
        )
    }
}

fn simplify_vertices(input_vertices: Vec<[f32; 3]>) -> Vec<[f32; 3]> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    vertices.reserve(input_vertices.len());
    let eps = 0.00001;

    vertices.push(input_vertices[0]);
    for i in 1..input_vertices.len() {
        let last = Vec3::from(*vertices.last().unwrap());

        // ignore if equal to the last one
        if Vec3::from(input_vertices[i]).distance(last) < eps {
            continue;
        }

        // ignore if it could be skipped (trivial hoop)
        if i < input_vertices.len() - 1 && Vec3::from(input_vertices[i + 1]).distance(last) < eps {
            continue;
        }

        vertices.push(input_vertices[i]);
    }

    // now do something similar to the end:
    while vertices.len() > 2 {
        let last = Vec3::from(*vertices.last().unwrap());

        // if the last one equals the first one, remove it
        if Vec3::from(vertices[0]).distance(last) < eps {
            vertices.pop();
            continue;
        }

        // if the last one equals the second one, remove the last two
        if Vec3::from(vertices[1]).distance(last) < eps {
            vertices.pop();
            vertices.pop();
            continue;
        }

        // ok
        break;
    }

    // not enough vertices left - the mesh is effectively empty!
    if vertices.len() <= 2 {
        vertices.clear();
    }

    vertices.shrink_to_fit();

    //println!("Simplified {} vertices to {}", _vertices.len(), vertices.len());
    //println!("vertices: {:?}", vertices);
    return vertices;
}

fn generate_uv_for_fan(
    vertices: &Vec<[f32; 3]>,
    uv_options: Option<(f32, f32)>,
) -> Option<Vec<[f32; 2]>> {
    if let Some((scale, angle)) = uv_options {
        // generate uv coordinates
        let mut uv = vec![[0.0, 0.0]]; // TODO: randomize center?
        let mut alpha = angle;
        let l = Vec3::from(vertices[1]).length();
        uv.push([alpha.cos() * scale * l, alpha.sin() * scale * l]);
        for i in 2..vertices.len() {
            let v0 = Vec3::from(vertices[i - 1]);
            let v1 = Vec3::from(vertices[i]);
            let a = v0.length();
            let b = v1.length();
            let c = (v1 - v0).length();
            // law of cosines
            let gamma = ((a * a + b * b - c * c) / (2.0 * a * b)).acos();
            alpha += gamma;
            uv.push([alpha.cos() * scale * b, alpha.sin() * scale * b]);
        }
        return Some(uv);
    }
    return None;
}
