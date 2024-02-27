use bevy::math::Vec3;

pub fn line_segment_intersection(
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
    tol: f32,
    tol2: f32,
) -> Option<Vec3> {
    let p = a;
    let q = c;
    let r = b - a;
    let s = d - c;
    let rxs = r.cross(s);
    let q_p = q - p;
    let qps = q_p.cross(s);

    // Check if the two lines are parallel
    if rxs.length_squared() < tol {
        // They are parallel. Now check if they are collinear.
        if qps.length_squared() < tol {
            // Collinear - handle this case if necessary
            // This implementation will not handle collinear segments
            return None;
        } else {
            // Parallel but not collinear, no intersection
            return None;
        }
    }

    let denominator = rxs.length_squared();
    let t = q_p.cross(s).dot(rxs) / denominator;
    let u = q_p.cross(r).dot(rxs) / denominator;

    // Check if the intersection point is on both line segments
    if t >= -tol2 && t <= 1.0 + tol2 && u >= -tol2 && u <= 1.0 + tol2 {
        // There is an intersection, and it is within the line segments
        let intersection = p + r * t;
        return Some(intersection);
    }

    // No intersection
    None
}
