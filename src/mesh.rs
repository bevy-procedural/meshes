use super::{IndexType, PIndices, PVertices};
use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues, render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};
use lyon::{
    lyon_tessellation::VertexBuffers,
    math::{Angle, Point},
};

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

    /// Iterates the faces assuming a triangle list.
    pub fn iter_faces_list(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        assert!(self.topology == PrimitiveTopology::TriangleList);
        self.indices
            .chunks_exact(3)
            .map(|w| [w[0].index(), w[1].index(), w[2].index()])
    }

    /// Iterates the faces  assuming a triangle strip.
    pub fn iter_faces_strip(&self) -> impl Iterator<Item = [usize; 3]> + '_ {
        assert!(self.topology == PrimitiveTopology::TriangleStrip);
        self.indices
            .windows(3)
            .map(|w| [w[0].index(), w[1].index(), w[2].index()])
    }

    /// Calculates the normals of the mesh.
    pub fn flat_normals(&mut self) -> &mut PMesh<T> {
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

    /// Calculates the normals of the mesh with smoothing based on https://stackoverflow.com/a/45496726/6144727
    ///
    /// WARNING: This might be buggy
    pub fn smooth_normals(&mut self, area_weighting: bool) -> &mut PMesh<T> {
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

    /// Imports a mesh from a lyon VertexBuffers.
    pub fn import_geometry(
        geometry: &VertexBuffers<Point, T>,
        flat: bool,
        normalize_uv: bool,
    ) -> PMesh<T>
    where
        T: IndexType,
    {
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
        let indices = geometry.indices.clone().iter().cloned().collect();

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
        PMesh::build_ex(vertices, indices, uv, None, PrimitiveTopology::TriangleList)
    }

    /*
    /// Assume the shape is (roughly) a 2D-polygon without holes.
    /// Detects the outline and sorts the vertices clockwise.
    ///
    /// WARNING: This doesn't work yet.
    pub fn sort_outline(&mut self) -> Vec<[f32; 3]> {
        // TODO: not working

        // build a graph of adjacency
        let mut graph: Vec<Vec<usize>> = Vec::new();
        for _ in 0..self.vertices.len() {
            graph.push(Vec::new());
        }
        for i in 0..self.indices.len() / 2 {
            let i0 = self.indices[2 * i + 0].index();
            let i1 = self.indices[2 * i + 1].index();
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
    }*/

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
    pub fn hexagon(radius: f32) -> PMesh<T> {
        let mut v = Vec::new();
        for i in 0..6 {
            let angle = ((6 - i) as f32 + 0.5) * std::f32::consts::PI / 3.0;
            v.push([radius * angle.cos(), 0.0, radius * angle.sin()]);
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
    pub fn rotate_y(&mut self, angle: Angle) -> &mut PMesh<T> {
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

    /// Copies the mesh into an existing bevy mesh.
    pub fn bevy_set(&self, mesh: &mut Mesh) {
        assert!(self.indices.iter_usize().all(|i| i < self.vertices.len()));

        // adjust topology if necessary
        let indices = if mesh.primitive_topology() != self.topology {
            self.clone().set_topology(self.topology).indices.clone()
        } else {
            self.indices.clone()
        };

        mesh.remove_indices();
        let mut attributes_to_remove = Vec::new();
        for (attr, _) in mesh.attributes() {
            attributes_to_remove.push(attr);
        }
        for attr in attributes_to_remove {
            mesh.remove_attribute(attr);
        }

        mesh.insert_indices(indices.get_bevy());
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

        if mesh.contains_attribute(Mesh::ATTRIBUTE_NORMAL) {
            // This will sometimes panic when the mesh is weird. Not very stable at all!
            if mesh.generate_tangents().is_err() {
                // TODO
                println!("WARN: Failed to generate tangents");
            }
        }
    }

    /// Creates a bevy mesh from the mesh.
    pub fn to_bevy(&self, usage: RenderAssetUsages) -> Mesh {
        let mut mesh = Mesh::new(self.topology, usage);
        self.bevy_set(&mut mesh);
        mesh
    }

    /// Adds backfaces to the mesh.
    pub fn add_backfaces(&mut self) -> &mut PMesh<T> {
        self.indices.add_backfaces();
        self
    }
}

fn get_bounding_rect<T>(buf: &VertexBuffers<Point, T>) -> (f32, f32, f32, f32) {
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
