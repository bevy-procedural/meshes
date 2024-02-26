use super::{IndexType, PMesh};
use bevy::{prelude::*, render::mesh::PrimitiveTopology};

impl<T> PMesh<T>
where
    T: IndexType,
{
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
}
