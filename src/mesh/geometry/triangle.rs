use crate::{IndexType, PMesh};
use bevy::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Triangle { a, b, c }
    }

    /// normal of the triangle (not normalized)
    pub fn normal(&self) -> Vec3 {
        (self.b - self.a).cross(self.c - self.a)
    }

    /// normalized normal of the triangle
    pub fn normal_normal(&self) -> Vec3 {
        self.normal().normalize()
    }

    /// the area of the triangle
    pub fn area(&self) -> f32 {
        0.5 * self.normal().length()
    }

    /// the centroid of the triangle
    pub fn centroid(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }

    /// Whether the triangle is degenerate (within tol)
    pub fn is_degenerate(&self, tol: f32) -> bool {
        self.normal().length() < tol
    }

    /// Whether the approximately coplanar triangles self and y have the same winding order (within tol)
    pub fn same_winding_direction(&self, y: Triangle, tol: f32) -> Option<bool> {
        let dot_product = self.normal_normal().dot(y.normal_normal());

        // Check if the dot product indicates the same winding direction, within the tolerance
        if (dot_product - 1.0).abs() < tol {
            Some(true) // Same winding direction
        } else if (dot_product + 1.0).abs() < tol {
            Some(false) // Opposite winding direction
        } else {
            // Triangles are not coplanar or too far from being coplanar
            None
        }
    }

    /// Check if point p is inside the triangle
    pub fn contains_point(&self, p: Vec3, tol: f32) -> bool {
        let v0 = self.c - self.a;
        let v1 = self.b - self.a;
        let v2 = p - self.a;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        (u >= -tol) && (v >= -tol) && (u + v <= 1.0 + tol)
    }

    /// Check if triangle x is fully within this triangle
    pub fn contains_triangle(&self, x: Triangle, tol: f32) -> bool {
        self.contains_point(x.a, tol)
            && self.contains_point(x.b, tol)
            && self.contains_point(x.c, tol)
    }

    /// Whether the triangles self and y are approximately coplanar (within tol)
    pub fn is_coplanar(&self, y: Triangle, tol: f32) -> bool {
        // Calculate the normal vector of the first triangle
        let n = self.normal();

        // Calculate D using the plane equation n . p + D = 0, where p is a point on the plane (vertex of the first triangle)
        let d_val = -n.dot(self.a);

        // Check if the vertices of the second triangle satisfy the plane equation
        let check_vertex_on_plane = |vertex: Vec3| -> bool { n.dot(vertex) + d_val <= tol };

        // If all vertices of the second triangle satisfy the plane equation within the tolerance, the triangles are coplanar
        check_vertex_on_plane(y.a) && check_vertex_on_plane(y.b) && check_vertex_on_plane(y.c)
    }

    /// Iterate over the vertices of the triangle
    pub fn iter(&self) -> impl Iterator<Item = Vec3> {
        vec![self.a, self.b, self.c].into_iter()
    }

    /// Iterate over the vertices of the triangle
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Vec3> {
        vec![&mut self.a, &mut self.b, &mut self.c].into_iter()
    }
}

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// Returns the Triangle at the given index.
    pub fn triangle_at(&self, i: usize) -> Triangle {
        Triangle {
            a: self.vec3_at(self.indices[i * 3 + 0].index()),
            b: self.vec3_at(self.indices[i * 3 + 1].index()),
            c: self.vec3_at(self.indices[i * 3 + 2].index()),
        }
    }

    /// Returns the Triangle at the given index.
    pub fn triangle_ex(&self, i: T, j: T, k: T) -> Triangle {
        Triangle {
            a: self.vec3_at(i.index()),
            b: self.vec3_at(j.index()),
            c: self.vec3_at(k.index()),
        }
    }

    /// Returns whether the triangle at the given index contains the given point.
    pub fn triangle_has_point(&self, i: usize, p: T, tol: f32) -> bool {
        self.triangle_at(i)
            .contains_point(self.vec3_at(p.index()), tol)
    }
}
