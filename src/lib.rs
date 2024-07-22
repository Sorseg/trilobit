//! Quickhull implementation
//! ## Literature
//! - <http://media.steampowered.com/apps/valve/2014/DirkGregorius_ImplementingQuickHull.pdf>
//! - <https://github.com/akuukka/quickhull/blob/master/QuickHull.cpp#L392>
//! - <http://www.qhull.org/>

#[cfg(feature = "vis")]
pub mod vis;

use glam::Vec3;

pub type TriangleIndices = [usize; 3];

struct Face {
    indices: TriangleIndices,
    conflicts: Vec<usize>,
}
impl Face {
    fn new(indices: [usize; 3]) -> Self {
        Self {
            indices,
            conflicts: Vec::with_capacity(16),
        }
    }
}

struct Hull<'p> {
    points: &'p [Vec3],
    faces: Vec<Face>,
}

impl<'p> Hull<'p> {
    fn new(points: &'p [Vec3]) -> Self {
        if points.len() < 4 {
            panic!("TODO: error handling");
        }
        Self {
            points,
            // FIXME(opt): tune capacity
            faces: Vec::with_capacity(16),
        }
    }

    fn populate_initial_hull(&mut self) {
        // # find extremums
        // FIXME(opt): uses indices, see if using references instead affects performance
        let mut minmax_by_dim = [[0; 2]; 3];

        for i in 1..self.points.len() {
            for dim in 0..3 {
                let dim2 = (dim + 1) % 3;
                let dim3 = (dim + 2) % 3;
                // FIXME: is there a cleaner way to do this?
                let cmp = |&i1: &usize, &i2: &usize| {
                    let v1 = self.points[i1].to_array();
                    let v2 = self.points[i2].to_array();
                    v1[dim]
                        .total_cmp(&v2[dim])
                        // FIXME: Does this matter?
                        .then(v1[dim2].total_cmp(&v2[dim2]))
                        .then(v1[dim3].total_cmp(&v2[dim3]))
                };
                minmax_by_dim[dim][0] = std::cmp::min_by(minmax_by_dim[dim][0], i, cmp);
                minmax_by_dim[dim][1] = std::cmp::max_by(minmax_by_dim[dim][1], i, cmp);
            }
        }

        // # find furthest points (idx,idx, dist_sq) to form the first line
        let mut furthest_pair = (0, 0, 0.0);

        for p1 in 0..self.points.len() {
            for p2 in p1..self.points.len() {
                let dist = self.points[p1].distance_squared(self.points[p2]);
                if dist > furthest_pair.2 {
                    furthest_pair = (p1, p2, dist)
                }
            }
        }
        let furthest_pair = (furthest_pair.0, furthest_pair.1);

        // FIXME: check degenerate case of all points on laying on the same the line

        // # find furthest point from the line to form the initial face
        let furthest_from_the_line = (0..self.points.len())
            .filter(|&p| p != furthest_pair.0 && p != furthest_pair.1)
            // FIXME(opt): too many distance computations, hoist line points lookup
            .max_by(|&p1, &p2| {
                let p1 = self.points[p1];
                let p2 = self.points[p2];
                let line = (self.points[furthest_pair.0], self.points[furthest_pair.1]);
                point_to_line_distance_unscaled(p1, line)
                    .total_cmp(&point_to_line_distance_unscaled(p2, line))
            })
            .expect("Constructor should have checked that there is no less than 4 points in the initial set");

        let mut face = [furthest_pair.0, furthest_pair.1, furthest_from_the_line];

        // # find furthest point from the face

        let furthest_from_the_face = (0..self.points.len())
            .filter(|p| !face.contains(p))
            // FIXME(opt): too many distance computations, hoist line points lookup
            .max_by(|&p1, &p2| {
                let face = face.map(|p| self.points[p]);
                let p1 = self.points[p1];
                let p2 = self.points[p2];
                // since we took the furthest points in the beginning, we can use distance to a plane,
                // rather than to the triangle, which is simpler
                point_to_plane_distance(p1, face).total_cmp(&point_to_plane_distance(p2, face))
            })
            .unwrap();

        // # build initial tetrahedron

        if is_point_in_front_of_plane(
            self.points[furthest_from_the_face],
            face.map(|p| self.points[p]),
        ) {
            face.swap(0, 2);
        }
        self.faces.push(Face::new(face));
        self.faces
            .push(Face::new([face[0], furthest_from_the_face, face[1]]));
        self.faces
            .push(Face::new([face[1], furthest_from_the_face, face[2]]));
        self.faces
            .push(Face::new([face[2], furthest_from_the_face, face[0]]));
    }

    fn process_next_conflict(&mut self) -> bool {
        // find the furthest point from all conflict lists and the horizon
        // split the horizon into new faces
        // redistribute points into new faces
        // return false if no points left
        todo!()
    }

    fn fill_conflicts(&mut self) {
        for face in &mut self.faces {
            let face_plane = face.indices.map(|p| self.points[p]);
            for i in 0..self.points.len() {
                if is_point_in_front_of_plane(self.points[i], face_plane) {
                    face.conflicts.push(i);
                }
            }
        }
    }
}

// TODO: generalize for 2d
pub fn hull(points: &[Vec3]) -> Vec<TriangleIndices> {
    // FIXME: figure out optimal capacity here
    let mut hull = Hull::new(points);
    hull.populate_initial_hull();
    hull.fill_conflicts();
    while hull.process_next_conflict() {}
    // return faces
    todo!()
}

/// returns a value proportional to the distance
fn point_to_line_distance_unscaled(p: Vec3, line: (Vec3, Vec3)) -> f32 {
    ((line.1.y - line.0.y) * p.x - (line.1.x - line.0.x) * p.y + line.1.x * line.0.y
        - line.1.y * line.0.x)
        .abs()
}

fn point_to_plane_distance(p: Vec3, face: [Vec3; 3]) -> f32 {
    // https://math.stackexchange.com/questions/588871/minimum-distance-between-point-and-face
    let normal = (face[1] - face[0])
        .cross(face[1] - face[2])
        .normalize_or_zero();
    let t = normal.dot(face[0]) - normal.dot(p);
    let p0 = p + t * normal;
    (p - p0).length()
}

// FIXME: robustness
fn is_point_in_front_of_plane(p: Vec3, plane: [Vec3; 3]) -> bool {
    let normal = (plane[0] - plane[1]).cross(plane[0] - plane[2]);
    normal.dot(plane[0] - p) > 0.0
}
