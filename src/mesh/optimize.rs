use super::geometry::line::line_segment_intersection;
use crate::{IndexType, PMesh};
use bevy::math::Vec3;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
struct EdgeIntersection {
    pub edge1: usize,
    pub edge2: usize,
    pub intersection: Vec3,
}

impl EdgeIntersection {
    pub fn new(edge1: usize, edge2: usize, intersection: Vec3) -> Self {
        EdgeIntersection {
            edge1,
            edge2,
            intersection,
        }
    }

    pub fn swap(&self) -> Self {
        EdgeIntersection {
            edge1: self.edge2,
            edge2: self.edge1,
            intersection: self.intersection,
        }
    }
}

fn edge_ordering_flip(e1: usize, e2: usize) -> (usize, bool) {
    if e1 == 0 && e2 == 2 {
        (0, true)
    } else if e1 == 2 && e2 == 0 {
        (0, false)
    } else {
        (e1.max(e2), e1 > e2)
    }
}

impl<T> PMesh<T>
where
    T: IndexType,
{
    /// 2 intersections, 1 and 0 interior points
    ///```
    ///         third
    ///          /\
    ///         /  \
    ///        /    \
    ///       / inner\
    ///      /   /\   \
    ///     /   /  \   \
    ///left/   /    \   \
    ///   /---X------X---\ right
    /// new1 /        \ new2
    ///     /          \
    ///    /------------\
    /// outer1         outer2
    ///```
    /// 2 intersections, 2 and 0 interior points
    ///```
    ///              inner              
    ///               /\                
    ///              /  \               
    ///             /    \              
    ///  left new1 /      \new2  right
    ///     \-----X--------X------/  
    ///      \   /          \    /   
    ///       \ /------------\  /    
    ///        \outer1   outer2/         
    ///         \             /        
    ///          \           /         
    ///           \  third  /       
    ///```

    fn configuration_2p_shared(
        &mut self,
        in1: EdgeIntersection,
        in2: EdgeIntersection,
        triangle1: usize,
        triangle2: usize,
        tol: f32,
    ) {
        let edge1 = in1.edge1;
        let (edge2, flip) = edge_ordering_flip(in1.edge2, in2.edge2);

        // insert the intersection points
        self.vertices
            .get_vertices_mut()
            .push(in1.intersection.into());
        self.vertices
            .get_vertices_mut()
            .push(in2.intersection.into());

        // new vertices
        let mut new2 = T::new(self.vertices.len() - 2);
        let mut new1 = T::new(self.vertices.len() - 1);

        if flip {
            let tmp = new1;
            new1 = new2;
            new2 = tmp;
        }

        // outer triangle
        let (left, right, third) = self.indices.get_triangle(triangle1, edge1);

        // inner triangle
        let (inner, outer1, outer2) = self.indices.get_triangle(triangle2, edge2);

        // is the little triangle tip outside?
        if let Some(val) = self
            .triangle_ex(left, right, third)
            .same_winding_direction(self.triangle_ex(inner, right, third), tol)
        {
            if !val {
                // cut the triangle once. Some overlap is left for future iterations...

                /*
                self.indices.overwrite(triangle1, new2, new1, third);
                self.indices.push(new1, right, third);
                self.indices.push(left, new2, third);
                */

                self.indices.overwrite(triangle2, new1, new2, inner);

                if !self.triangle_has_point(triangle1, outer2, tol) {
                    self.indices.push(new1, outer2, new2);
                    self.indices.push(new1, outer1, outer2);
                } else if !self
                    .triangle_at(triangle1)
                    .contains_point(self.vec3_at(outer1.index()), tol)
                {
                    self.indices.push(new1, outer1, outer2);
                }

                return;
            }
        }

        self.indices.overwrite(triangle2, new2, new1, outer1);
        self.indices.push(outer1, outer2, new2);

        if self
            .triangle_at(triangle1)
            .contains_point(self.vec3_at(inner.index()), tol)
        {
            return;
        }

        self.indices.overwrite(triangle1, new2, right, inner);
        self.indices.push(inner, right, third);
        self.indices.push(inner, third, new1);
        self.indices.push(new1, third, left);
        self.indices.push(new1, new2, inner);
    }

    /// 2 intersections, 1 interior point each
    ///```
    /// left2          right2
    ///    \------------/
    ///     \   tip1   /
    ///      \   /\   /
    ///       \ /  \ /
    ///   new1 X    X new2
    ///       / \  / \
    ///      /   \/   \
    ///     /   tip2   \
    ///    /------------\
    /// left1         right1
    ///```
    /// 2 intersections, 2 and 1 interior points
    ///```                     
    /// left1_____________________right1      
    ///    \_ left2_____right2 _/         
    ///      \_  \        /  _/           
    ///        \_|        |_/             
    ///     new1 X_      _X new2           
    ///          \ \_  _/ /                 
    ///           |  °°  |                 
    ///           \ tip1 /                  
    ///            |    |                  
    ///            \    /                   
    ///             |  |                   
    ///              \/                    
    ///             tip2       
    ///```            
    fn configuration_2p_distinct(
        &mut self,
        in1: EdgeIntersection,
        in2: EdgeIntersection,
        triangle1: usize,
        triangle2: usize,
        tol: f32,
    ) {
        let (edge1, flip1) = edge_ordering_flip(in1.edge1, in2.edge1);
        let (edge2, flip2) = edge_ordering_flip(in1.edge2, in2.edge2);

        // insert the intersection points
        self.vertices
            .get_vertices_mut()
            .push(in1.intersection.into());
        self.vertices
            .get_vertices_mut()
            .push(in2.intersection.into());

        // new vertices
        let mut new1 = T::new(self.vertices.len() - 2);
        let mut new2 = T::new(self.vertices.len() - 1);

        if flip1 ^ flip2 {
            let tmp = new1;
            new1 = new2;
            new2 = tmp;
        }

        let (tip1, right1, left1) = self.indices.get_triangle(triangle1, edge1);
        let (tip2, left2, right2) = self.indices.get_triangle(triangle2, edge2);

        // detect flipped configuration! (windings are different)
        // i.e., there are two triangles of one mesh inside the other one instead of just one
        if let Some(val) = self
            .triangle_ex(new2, new1, tip2)
            .same_winding_direction(self.triangle_ex(new1, new2, tip1), tol)
        {
            if !val {
                if self.triangle_has_point(triangle1, tip2, tol) {
                    assert!(!self.triangle_has_point(triangle2, tip1, tol));
                    self.indices.overwrite(triangle1, tip1, new1, new2);
                    // TODO: foot is too large; continue it
                    self.indices.overwrite(triangle2, new2, new1, right2);
                    self.indices.push(left2, new2, right2);
                } else {
                    assert!(self.triangle_has_point(triangle2, tip1, tol));
                    self.indices.overwrite(triangle2, tip2, new2, new1);
                    // TODO: foot is too large; continue it
                    self.indices.overwrite(triangle1, new1, new2, right1);
                    self.indices.push(left1, new1, right1);
                }

                return;
            }
        }

        if !self.triangle_has_point(triangle1, tip2, tol) {
            // draw only when tip is outside
            self.indices.push(new2, new1, tip2);
        }

        if !self.triangle_has_point(triangle2, tip1, tol) {
            // draw only when tip is outside
            self.indices.push(new1, new2, tip1);
        }

        self.indices.overwrite(triangle2, new1, new2, left2);
        self.indices.push(new1, left2, right2);

        self.indices.overwrite(triangle1, new2, new1, right1);
        self.indices.push(new2, right1, left1);
    }

    /// 6 intersections, no interior points
    ///```
    ///                  t23
    ///                  /\             
    ///                 /  \               
    ///    t13    new5 /    \ new4  t12    
    ///      \--------+------+-------/
    ///        \    /         \     /    
    ///         \ /             \ /       
    ///          X new6          X new3   
    ///         / \             / \       
    ///       /     \         /     \    
    ///      /--------+------+--------\   
    ///    t21    new1 \    / new2    t22
    ///                 \  /               
    ///                  \/                
    ///                  t11             
    ///```
    fn configuration_6p(&mut self, _triangle1: usize, _triangle2: usize) {
        // TODO
    }

    /// 4 intersections, no interior points
    ///```
    ///          |\ tip1                            
    ///          | \                                
    ///          |  \                               
    ///   left2  |   \                              
    ///    |-----+----+--------------------tip2     
    ///    | new4|     \new3         -----/         
    ///    |     |      \       ----/               
    ///    |     |       X----/                    
    ///    |     |   ----/\new2                    
    ///    |   --+--/      \                       
    ///    |--/  |new1      \                      
    ///   right2 |           \                     
    ///          |            \                    
    ///          |             \                   
    ///          |              \                  
    ///          -----------------                 
    ///        left1             right1             
    ///```
    fn configuration_4p_00(
        &mut self,
        triangle1: usize,
        _triangle2: usize,
        intersections: Vec<EdgeIntersection>,
        _tol: f32,
    ) {
        // sort the intersections by the edge of the triangle with the interior point (and lexicographically also by the second triangle)
        let mut inter = intersections.clone();
        inter.sort_by(|a, b| (a.edge1, a.edge2).cmp(&(b.edge1, b.edge2)));
        assert!(inter.len() == 4);

        // find new4 - the point, where the two triangles edges increase by one and rotate it to the front
        for i in 0..4 {
            if ((inter[i].edge1 + 1) % 3) == inter[(i + 1) % 4].edge1
                && inter[i].edge2 == ((inter[(i + 1) % 4].edge2 + 1) % 3)
            {
                inter.rotate_left(i);
                break;
            }
        }

        let (left1, tip1, right1) = self.indices.get_triangle(triangle1, inter[0].edge1);

        let mut new = [T::new(0); 4];
        for i in 0..4 {
            new[i] = T::new(self.vertices.len());
            self.vertices
                .get_vertices_mut()
                .push(inter[i].intersection.into());
        }

        if inter[0].edge1 != inter[2].edge2 {
            self.indices.overwrite(triangle1, new[1], new[3], tip1);
            self.indices.push(new[0], new[2], right1);
            self.indices.push(new[0], right1, left1);
        } else {
            self.indices.overwrite(triangle1, new[2], new[0], tip1);
            self.indices.push(new[3], new[1], right1);
            self.indices.push(new[3], right1, left1);
        }
    }

    /// 4 intersections, 1 interior point
    ///```
    ///     left1|\                                 
    ///          | \                                
    ///          |  \                               
    ///   left2  |   \                              
    ///    |----\|    \ new3                        
    ///    |     X-----+-\                          
    ///    |     |new4  \ --------\                 
    ///    |     |       \         --------\        
    ///    |     |        tip1              -tip2   
    ///    |     |       /         --------/        
    ///    |     |new1  / --------/                 
    ///    |     X-----+-/                          
    ///    |----/|    / new2                        
    ///   right2 |   /                              
    ///          |  /                               
    ///          | /                                
    ///    right1|/                                 
    ///          -                                  
    ///```                                         
    fn configuration_4p_01(
        &mut self,
        triangle1: usize,
        _triangle2: usize,
        _interior: usize,
        intersections: Vec<EdgeIntersection>,
    ) {
        // sort the intersections by the edge of the triangle with the interior point (and lexicographically also by the second triangle)
        let mut inter = intersections.clone();
        inter.sort_by(|a, b| (a.edge1, a.edge2).cmp(&(b.edge1, b.edge2)));
        assert!(inter.len() == 4);
        assert!(
            inter.iter().map(|x| x.edge1).collect::<HashSet<_>>().len() == 3,
            "There should be an intersection on each edge"
        );
        assert!(
            inter.iter().map(|x| x.edge2).collect::<HashSet<_>>().len() == 2,
            "There should not be an intersection on each edge"
        );

        // find the edge where the interior point is and shift it to the front
        for i in 0..4 {
            if inter[i].edge1 != inter[(i + 1) % 4].edge1
                && inter[i].edge2 != inter[(i + 1) % 4].edge2
            {
                inter.rotate_left(i);
                break;
            }
        }

        // println!("inter: {:?}", inter.iter().map(|x| (x.edge1, x.edge2)).collect::<Vec<(usize,usize)>>());

        let (left1, _tip1, right1) = self.indices.get_triangle(triangle1, inter[0].edge1);
        let mut new = [T::new(0); 4];
        for i in 0..4 {
            new[i] = T::new(self.vertices.len());
            self.vertices
                .get_vertices_mut()
                .push(inter[i].intersection.into());
        }

        if inter[0].edge1 == inter[2].edge2 {
            self.indices.overwrite(triangle1, new[3], new[1], right1);
            self.indices.push(new[0], new[2], left1);
        } else {
            self.indices.overwrite(triangle1, new[2], new[1], right1);
            self.indices.push(new[0], new[3], left1);
        }
    }

    /// 4 intersections, 1 interior point each
    ///```                                                 
    ///                        -\ left1                  
    ///                        /  \                       
    ///                       /    \                      
    ///                      /      \                     
    ///                     /  inner2\                    
    ///                    /   /- -\  \                   
    ///                   /----     ---\new3              
    ///                /-X-new4         X--\              
    ///            /--- /                \  ---\          
    ///        /---    /inner1            \     ---\      
    ///    /---       --\                  \        ---\  
    ///  -----------------X-----------------+-------------
    ///   left2          new1 -----\         \new2  right2
    ///                             ----\     \           
    ///                                  ----\ \          
    ///                                       -right1     
    ///```                                           
    fn configuration_4p_11(
        &mut self,
        triangle1: usize,
        triangle2: usize,
        interior1: usize,
        interior2: usize,
        intersections: Vec<EdgeIntersection>,
    ) {
        // sort the intersections by the edge of the triangle with the interior point (and lexicographically also by the second triangle)
        let mut inter = intersections.clone();
        inter.sort_by(|a, b| (a.edge1, a.edge2).cmp(&(b.edge1, b.edge2)));
        assert!(inter.len() == 4);

        let mut d = None;

        // find new4 - the point, where the two triangles edges increase by one and rotate it to the front
        for i in 0..4 {
            if inter[i].edge1 == interior1 && inter[(i + 1) % 4].edge2 == interior2 {
                inter.rotate_left(i);
                d = Some(true);
                break;
            }
            if inter[(i + 1) % 4].edge1 == interior1 && inter[i].edge2 == interior2 {
                inter.rotate_left(i);
                d = Some(false);
                break;
            }
        }

        let (left1, _tip1, right1) = self.indices.get_triangle(triangle1, inter[0].edge1);
        let (left2, _tip2, right2) = self
            .indices
            .get_triangle(triangle2, inter[if d.unwrap() { 1 } else { 3 }].edge2);

        let mut new = [T::new(0); 4];
        for i in 0..4 {
            new[i] = T::new(self.vertices.len());
            self.vertices
                .get_vertices_mut()
                .push(inter[i].intersection.into());
        }

        self.indices.overwrite(triangle1, new[2], new[1], right1);
        self.indices.push(new[0], new[3], left1);

        self.indices.overwrite(triangle2, new[3], new[0], left2);
        self.indices.push(new[0], right2, left2);
    }

    fn process_edge_intersections(
        &mut self,
        intersections: Vec<EdgeIntersection>,
        triangle1: usize,
        triangle2: usize,
        tol: f32,
    ) -> bool {
        // never intersected
        if intersections.len() == 0 {
            return false;
        }

        // only touching
        if intersections.len() == 1 {
            return false;
        }

        if intersections.len() == 2 {
            let in1 = intersections[0];
            let in2 = intersections[1];

            // first two the same, other two different
            if in1.edge1 == in2.edge1 && in1.edge2 != in2.edge2 {
                self.configuration_2p_shared(in1, in2, triangle1, triangle2, tol);
                return true;
            } else if in1.edge2 == in2.edge2 && in1.edge1 != in2.edge1 {
                self.configuration_2p_shared(in2.swap(), in1.swap(), triangle2, triangle1, tol);
                return true;
            } else if in1.edge1 != in2.edge1 && in1.edge2 != in2.edge2 {
                self.configuration_2p_distinct(in1, in2, triangle1, triangle2, tol);
                return true;
            } else {
                panic!("impossible")
            }
        }

        if intersections.len() == 4 {
            let interior_points_t1 = self
                .triangle_at(triangle1)
                .iter()
                .position(|v| self.triangle_at(triangle2).contains_point(v, tol));
            let interior_points_t2 = self
                .triangle_at(triangle2)
                .iter()
                .position(|v| self.triangle_at(triangle1).contains_point(v, tol));

            if interior_points_t1.is_none() && interior_points_t2.is_none() {
                self.configuration_4p_00(triangle1, triangle2, intersections, tol);
                return true;
            } else if interior_points_t1.is_some() && interior_points_t2.is_none() {
                self.configuration_4p_01(
                    triangle1,
                    triangle2,
                    interior_points_t1.unwrap(),
                    intersections,
                );
                return true;
            } else if interior_points_t1.is_none() && interior_points_t2.is_some() {
                self.configuration_4p_01(
                    triangle2,
                    triangle1,
                    interior_points_t2.unwrap(),
                    intersections.iter().map(|x| x.swap()).collect(),
                );
                return true;
            } else if interior_points_t1.is_some() && interior_points_t2.is_some() {
                self.configuration_4p_11(
                    triangle1,
                    triangle2,
                    interior_points_t1.unwrap(),
                    interior_points_t2.unwrap(),
                    intersections,
                );
                return true;
            }

            return false;
        }

        if intersections.len() == 6 {
            self.configuration_6p(triangle1, triangle2);
            return true;
        }

        println!("Intersections: {}", intersections.len());

        return false;
    }

    /// Inserts vertices at all complanar intersections of edges and converts the created shapes into triangles.
    pub fn cut_complanar_edges(&mut self, in_max_changes: u32) -> &mut PMesh<T> {
        // TODO: this is very slow!

        let mut max_changes = in_max_changes; // TODO: for debugging only

        let tol = 0.0001;
        self.uv = None;
        self.normals = None;

        let mut triangle1 = 0;
        loop {
            if triangle1 >= self.indices.len() / 3 {
                break;
            }

            if self.triangle_at(triangle1).is_degenerate(tol) {
                triangle1 += 1;
                continue;
            }

            // println!("START {} {}", in_max_changes - max_changes, triangle1);

            let mut again = false;

            // compare with all following edges
            for triangle2 in (triangle1 + 1)..(self.indices.len() / 3) {
                if self.triangle_at(triangle2).is_degenerate(tol) {
                    continue;
                }

                if !self
                    .triangle_at(triangle1)
                    .is_coplanar(self.triangle_at(triangle2), tol * 2.0)
                {
                    continue;
                }

                if self
                    .triangle_at(triangle1)
                    .contains_triangle(self.triangle_at(triangle2), tol * 2.0)
                {
                    // remove triangle2
                    let is = self.indices.get_indices_mut();
                    is[triangle2 * 3 + 0] = T::new(0);
                    is[triangle2 * 3 + 1] = T::new(0);
                    is[triangle2 * 3 + 2] = T::new(0);
                }

                let mut intersections = Vec::new();
                for edge1 in 0..3 {
                    // consider the edge from pos to pos+1
                    let pos1a = triangle1 * 3 + edge1;
                    let pos1b = triangle1 * 3 + (edge1 + 1) % 3;
                    let i1a = self.indices[pos1a].index();
                    let i1b = self.indices[pos1b].index();
                    let v1a = self.vec3_at(i1a);
                    let v1b = self.vec3_at(i1b);
                    for edge2 in 0..3 {
                        // consider the edge from pos to pos+1
                        let pos2a = triangle2 * 3 + edge2;
                        let pos2b = triangle2 * 3 + (edge2 + 1) % 3;
                        let i2a = self.indices[pos2a].index();
                        let i2b = self.indices[pos2b].index();
                        let v2a = self.vec3_at(i2a);
                        let v2b = self.vec3_at(i2b);

                        // if two edges share a vertex, they can't intersect (except for collinear edges)
                        if i2a == i1a || i2a == i1b || i2b == i1a || i2b == i1b {
                            continue;
                        }

                        // check for intersection. Negative tol2 to avoid detecting points on edges.
                        // TODO: we still need a solution for edge collisions. If the intersection is on a vertex, we get the wrong number of intersections and can't procede!
                        if let Some(intersection) =
                            line_segment_intersection(v1a, v1b, v2a, v2b, tol, -tol)
                        {
                            intersections.push(EdgeIntersection {
                                edge1,
                                edge2,
                                intersection,
                            });
                        }
                    }
                }

                if self.process_edge_intersections(intersections, triangle1, triangle2, tol) {
                    again = true;

                    max_changes -= 1;
                    if max_changes == 0 {
                        return self;
                    }

                    break;
                }
            }

            if !again {
                triangle1 += 1;
            }
        }

        self.remove_coplanar_vertices();
        return self;
    }

    /// Removes all vertices that are in the middle of a flat surface and can be removed.
    pub fn remove_coplanar_vertices(&mut self) -> &mut PMesh<T> {
        /*let tol = 0.0001;

        for i in 0..(self.indices.len() / 3) {
            let triangle = self.triangle_at(i);
            if triangle.is_degenerate(tol) {
                continue;
            }

            let base_a = triangle.b - triangle.a;
            let base_b = triangle.c - triangle.a;

            for k in 0..3 {
                let mut coplanar = true;
                let central_point = i * 3 + k;
                let mut interior_angles = Vec::new();

                for j in 0..(self.indices.len() / 3) {
                    if i == j {
                        continue;
                    }

                    let triangle2 = self.triangle_at(j);
                    if triangle2.is_degenerate(tol) {
                        continue;
                    }

                    for l in 0..3 {
                        let other_point = j * 3 + l;

                        // skip triangles that don't share the central point
                        if self.indices.get_indices()[central_point]
                            != self.indices.get_indices()[other_point]
                        {
                            continue;
                        }

                        // TODO: sum the interior angles. Only if the sum is 360 and all triangles are complanar, the vertex can be removed

                        // Volume of parallelepiped is |base_a . (base_b x vec)|
                        // If volume is not zero (within tolerance), points are not coplanar
                        if base_a.dot(base_b.cross(triangle2.a)).abs() > tol
                            || base_a.dot(base_b.cross(triangle2.b)).abs() > tol
                            || base_a.dot(base_b.cross(triangle2.c)).abs() > tol
                        {
                            coplanar = false;
                            break;
                        }
                    }
                }
                if coplanar {
                    // TODO: remove
                }
            }
        }*/

        self
    }
}
