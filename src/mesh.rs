use super::path::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use lyon::lyon_tessellation::VertexBuffers;
use lyon::math::{Angle, Point};

// TODO: make an open-source crate out of this

#[derive(Clone, Debug)]
pub struct MyMesh {
    pub vertices: MyPath,
    pub indices: Vec<u32>,
    pub uv: Option<Vec<[f32; 2]>>,
    pub normals: Option<Vec<[f32; 3]>>,
    topology: PrimitiveTopology,
}

impl Default for MyMesh {
    fn default() -> Self {
        MyMesh::new()
    }
}

impl MyMesh {
    pub fn new() -> Self {
        MyMesh {
            vertices: MyPath::new(),
            indices: Vec::new(),
            uv: Some(Vec::new()),
            normals: None,
            topology: PrimitiveTopology::TriangleStrip,
        }
    }

    // duplicate all shared vertices
    pub fn duplicate(&mut self) -> &mut MyMesh {
        self.vertices.vertices = self
            .indices
            .iter()
            .map(|i| self.vertices.vertices[*i as usize])
            .collect();
        if let Some(uv) = &self.uv {
            self.uv = Some(self.indices.iter().map(|i| uv[*i as usize]).collect());
        }
        if let Some(normals) = &self.normals {
            self.normals = Some(self.indices.iter().map(|i| normals[*i as usize]).collect());
        }
        self.indices = (0..self.indices.len() as u32).collect();
        self
    }

    pub fn vec3_at(&self, i: usize) -> Vec3 {
        Vec3::from(self.vertices.vertices[i])
    }

    pub fn iter_faces_list(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        assert!(self.topology == PrimitiveTopology::TriangleList);
        self.indices
            .chunks_exact(3)
            .map(|w| [w[0] as usize, w[1] as usize, w[2] as usize])
    }

    pub fn iter_faces_strip(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        assert!(self.topology == PrimitiveTopology::TriangleStrip);
        self.indices
            .windows(3)
            .map(|w| [w[0] as usize, w[1] as usize, w[2] as usize])
    }

    pub fn flat_normals(&mut self) -> &mut MyMesh {
        assert!(
            self.indices.len() == self.vertices.len(),
            "requires duplicated vertices"
        );
        assert!(
            self.topology == PrimitiveTopology::TriangleList,
            "requires triangle list topology"
        );

        self.normals = Some(
            self.iter_faces_list()
                .map(|[v1, v2, v3]| {
                    (self.vec3_at(v2) - self.vec3_at(v1))
                        .cross(self.vec3_at(v3) - self.vec3_at(v1))
                        .normalize()
                        .into()
                })
                .flat_map(|n| [n; 3])
                .collect(),
        );

        self
    }

    pub fn smooth_normals(&mut self, area_weighting: bool) -> &mut MyMesh {
        // based on https://stackoverflow.com/a/45496726/6144727

        // TODO: this looks a bit buggy on screen. Not sure whether it's a problem with the normals or the normal interpolation/color grading in the shader.

        let mut normals = vec![Vec3::ZERO; self.vertices.len()];
        for [f1, f2, f3] in self.iter_faces_list() {
            let v1 = self.vec3_at(f1);
            let v2 = self.vec3_at(f2);
            let v3 = self.vec3_at(f3);

            // calculate facet normal of the triangle using cross product;
            // both components are "normalized" against a common point chosen as the base
            let normal = (v2 - v1).cross(v3 - v1);

            let normal = if area_weighting {
                normal
            } else {
                normal.normalize()
            };

            // get the angle between the two other points for each point;
            // the starting point will be the 'base' and the two adjacent points will be normalized against it
            let angle1 = (v2 - v1).angle_between(v3 - v1);
            let angle2 = (v3 - v2).angle_between(v1 - v2);
            let angle3 = (v1 - v3).angle_between(v2 - v3);

            normals[f1] += normal * angle1;
            normals[f2] += normal * angle2;
            normals[f3] += normal * angle3;
        }

        self.normals = Some(
            normals
                .iter()
                .map(|n| {
                    let n = n.normalize();
                    [n.x, n.y, n.z]
                })
                .collect(),
        );

        self
    }

    pub fn import_geometry(
        geometry: &VertexBuffers<Point, u16>,
        flat: bool,
        normalize_uv: bool,
    ) -> MyMesh {
        let vertices: Vec<[f32; 3]> = geometry
            .vertices
            .iter()
            .map(|v| {
                if flat {
                    [v.x, 0.0, v.y]
                } else {
                    [-v.x, v.y, 0.0]
                }
            })
            .collect();
        let indices = geometry.indices.clone().iter().map(|i| *i as u32).collect();

        let mut uv_x_scale = 1.0;
        let mut uv_y_scale = 1.0;

        if normalize_uv {
            //println!("x_min: {}, x_max: {}, y_min: {}, y_max: {}", x_min, x_max, y_min, y_max);
            let (x_min, x_max, y_min, y_max) = get_bounding_rect(geometry);
            uv_x_scale = x_max - x_min;
            uv_y_scale = y_max - y_min;
        }
        let uv: Option<Vec<[f32; 2]>> = Some(
            geometry
                .vertices
                .iter()
                .map(|v| [v.x / uv_x_scale, v.y / uv_y_scale])
                .collect(),
        );
        MyMesh::build(vertices, indices, uv)
    }

    // TODO: not working
    pub fn sort_outline(&mut self) -> Vec<[f32; 3]> {
        // build a graph of adjacency
        let mut graph: Vec<Vec<usize>> = Vec::new();
        for _ in 0..self.vertices.len() {
            graph.push(Vec::new());
        }
        for i in 0..self.indices.len() / 2 {
            let i0 = self.indices[2 * i + 0] as usize;
            let i1 = self.indices[2 * i + 1] as usize;
            if !graph[i0].contains(&i1) {
                graph[i0].push(i1);
            }
            if !graph[i1].contains(&i0) {
                graph[i1].push(i0);
            }
        }
        /*for i in 0..self.indices.len() / 3 {
            let i0 = self.indices[3 * i + 0] as usize;
            let i1 = self.indices[3 * i + 1] as usize;
            let i2 = self.indices[3 * i + 2] as usize;
            graph[i0].push(i1);
            graph[i0].push(i2);
            graph[i1].push(i2);
            graph[i1].push(i0);
            graph[i2].push(i0);
            graph[i2].push(i1);
        }*/

        // sort neighbors by atan2
        for i in 0..self.vertices.len() {
            let mut v = Vec::new();
            for j in &graph[i] {
                let mut v0 = Vec3::from(self.vertices.vertices[*j]);
                v0 -= Vec3::from(self.vertices.vertices[i]);
                v.push((v0.y.atan2(v0.x), *j));
            }
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            graph[i] = v.iter().map(|(_, i)| *i).collect();
        }

        // sort vertices by always taking the clockwise next neighbor based on the previous one. If there are no holes, we can detect the outline
        let mut res = Vec::new();
        let mut last = 0;
        let mut current = graph[last][0];
        loop {
            res.push(self.vertices.vertices[current]);
            if graph[current].len() == 0 {
                break;
            }
            let j = graph[current].iter().position(|i| *i == last).unwrap();
            //println!("{} -> {}    {} {:?} ", last, current, j, graph[current]);
            last = current;
            let gl = graph[current].len();
            current = graph[current][(j + 1) % gl];
            /*for k in 1..gl {
                let l = (j + k) % gl;
                current = graph[current][l];
                assert!(current != last);
                if graph[current].len() > 0   {
                    break;
                }
            }*/
            // mark as visited
            graph[last].clear();
        }

        //println!("Sorted {} <- {} ", res.len(), self.vertices.len());

        res
    }

    pub fn sort_outline_cheap(&mut self) -> Vec<[f32; 3]> {
        let mut res = Vec::new();
        for i in 0..self.vertices.len() / 2 {
            res.push(self.vertices.vertices[2 * i]);
        }
        for i in (0..(self.vertices.len() - 0) / 2).rev() {
            res.push(self.vertices.vertices[2 * i + 1]);
        }

        res
    }

    pub fn rect(width: f32, height: f32) -> MyMesh {
        MyMesh::build(
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

    pub fn rect_c(width: f32, height: f32) -> MyMesh {
        let w = width * 0.5;
        let h = height * 0.5;
        MyMesh::build(
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

    fn simplify_vertices(_vertices: Vec<[f32; 3]>) -> Vec<[f32; 3]> {
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        vertices.reserve(_vertices.len());
        let eps = 0.00001;

        vertices.push(_vertices[0]);
        for i in 1.._vertices.len() {
            let last = Vec3::from(*vertices.last().unwrap());

            // ignore if equal to the last one
            if Vec3::from(_vertices[i]).distance(last) < eps {
                continue;
            }

            // ignore if it could be skipped (trivial hoop)
            if i < _vertices.len() - 1 && Vec3::from(_vertices[i + 1]).distance(last) < eps {
                continue;
            }

            vertices.push(_vertices[i]);
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

    pub fn fan(_vertices: Vec<[f32; 3]>, uv_options: Option<(f32, f32)>) -> MyMesh {
        // automatically reduce vertices if there are duplicates
        let vertices = MyMesh::simplify_vertices(_vertices);

        // the mesh is degenerate - drop it!
        if vertices.len() < 3 {
            return MyMesh::new();
        }

        let mut indices = Vec::new();
        for i in 1..vertices.len() - 1 {
            indices.push(0);
            indices.push(i as u32);
            indices.push(i as u32 + 1);
        }

        let uv = MyMesh::generate_uv_for_fan(&vertices, uv_options);
        MyMesh::build(vertices, indices, uv)
    }

    pub fn hexagon(radius: f32) -> MyMesh {
        let mut v = Vec::new();
        for i in 0..6 {
            let angle = ((6 - i) as f32 + 0.5) * std::f32::consts::PI / 3.0;
            v.push([radius * angle.cos(), 0.0, radius * angle.sin()]);
        }
        MyMesh::fan(v, Some((1.0, 0.0)))
    }

    pub fn triangle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> MyMesh {
        MyMesh::build(
            vec![a, b, c],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
        )
    }

    pub fn build_ex(
        vertices: Vec<[f32; 3]>,
        indices: Vec<u32>,
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
        MyMesh {
            vertices: MyPath::build(vertices),
            indices,
            uv,
            normals,
            topology,
        }
    }

    pub fn build(vertices: Vec<[f32; 3]>, indices: Vec<u32>, uv: Option<Vec<[f32; 2]>>) -> Self {
        MyMesh::build_ex(vertices, indices, uv, None, PrimitiveTopology::TriangleList)
    }

    pub fn append(&mut self, m: &MyMesh) -> &mut MyMesh {
        let offset = self.vertices.len() as u32;
        self.vertices.extend(m.vertices.clone());
        self.indices.extend(m.indices.iter().map(|i| i + offset));
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

    pub fn rotate_y(&mut self, angle: Angle) -> &mut MyMesh {
        self.vertices.rotate_y(angle);
        self
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut MyMesh {
        self.vertices.translate(x, y, z);
        self
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut MyMesh {
        self.vertices.scale(x, y, z);
        self
    }

    pub fn scale_all(&mut self, x: f32) -> &mut MyMesh {
        self.vertices.scale(x, x, x);
        self
    }

    // remove duplicate vertices. Messes with the uv mapping.
    pub fn optimize(&mut self) -> &mut MyMesh {
        let mut new_indices = Vec::new();
        let mut new_vertices = Vec::new();
        let mut new_uv = Vec::new();
        let eps = 0.0001;

        for i in &self.indices {
            let v = self.vertices.vertices[*i as usize];

            if let Some(index) = new_vertices
                .iter()
                .position(|v2| Vec3::from(v).distance(Vec3::from(*v2)) < eps)
            {
                new_indices.push(index as u32);
            } else {
                new_indices.push(new_vertices.len() as u32);
                new_vertices.push(v);
                if let Some(v) = &self.uv {
                    new_uv.push(v[*i as usize]);
                }
            }
        }

        /*println!(
            "Optimized {} vertices to {}",
            self.vertices.len(),
            new_vertices.len()
        );*/

        self.vertices = MyPath::build(new_vertices);
        self.indices = new_indices;
        self.uv = match new_uv.len() > 0 {
            true => Some(new_uv),
            false => None,
        };

        self
    }

    pub fn bevy_set(&self, mesh: &mut Mesh) {
        assert!(self.indices.iter().all(|i| *i < self.vertices.len() as u32));

        assert!(mesh.primitive_topology() == self.topology);

        mesh.remove_indices();
        mesh.insert_indices(Indices::U32(self.indices.clone()));
        mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(self.vertices.vertices.clone()),
        );
        if let Some(uv) = &self.uv {
            mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
            assert!(self.vertices.len() == uv.len());
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(uv.clone()),
            );
        }

        mesh.remove_attribute(Mesh::ATTRIBUTE_NORMAL);
        if let Some(normals) = &self.normals {
            assert!(self.vertices.len() == normals.len());
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(normals.clone()),
            );
        } else {
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
        }

        // This will sometimes panic when the mesh is weird. Not very stable at all!
        mesh.remove_attribute(Mesh::ATTRIBUTE_TANGENT);
        if mesh.generate_tangents().is_err() {
            // TODO
            println!("WARN: Failed to generate tangents");
        }
    }

    pub fn to_bevy(&self) -> Mesh {
        let mut mesh = Mesh::new(self.topology, RenderAssetUsages::RENDER_WORLD);
        self.bevy_set(&mut mesh);
        mesh
    }

    pub fn add_backfaces(&mut self) -> &mut MyMesh {
        let l = self.indices.len();
        self.indices.reserve(l);
        for i in (0..l).rev() {
            self.indices.push(self.indices[i]);
        }

        self
    }
}

pub fn get_bounding_rect(buf: &VertexBuffers<Point, u16>) -> (f32, f32, f32, f32) {
    /*let x_max = buf
        .vertices
        .iter()
        .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let x_min = buf
        .vertices
        .iter()
        .min_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let y_max = buf
        .vertices
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();
    let y_min = buf
        .vertices
        .iter()
        .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();
    (x_min.x, x_max.x, y_min.y, y_max.y)*/

    let mut x_min = std::f32::MAX;
    let mut x_max = std::f32::MIN;
    let mut y_min = std::f32::MAX;
    let mut y_max = std::f32::MIN;
    for v in &buf.vertices {
        if v.x < x_min {
            x_min = v.x;
        }
        if v.x > x_max {
            x_max = v.x;
        }
        if v.y < y_min {
            y_min = v.y;
        }
        if v.y > y_max {
            y_max = v.y;
        }
    }

    (x_min, x_max, y_min, y_max)
}
