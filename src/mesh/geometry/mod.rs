use bevy::math::Vec3;
pub mod line;
pub mod triangle;

pub fn are_points_coplanar(points: Vec<Vec3>, tol: f32) -> bool {
    if points.len() < 4 {
        return true; // Fewer than 4 points are always coplanar
    }

    // Base vectors from the first three points
    let base_a = points[1] - points[0];
    let base_b = points[2] - points[0];

    for i in 3..points.len() {
        let vec = points[i] - points[0];
        // Volume of parallelepiped is |base_a . (base_b x vec)|
        // If volume is not zero (within tolerance), points are not coplanar
        if base_a.dot(base_b.cross(vec)).abs() > tol {
            return false;
        }
    }

    true
}
