//! A small indexed triangle-mesh shape.

use crate::shapes::{Hit, Material, Shape, Transform};
use glam::Vec3;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

const MAX_LEAF_TRIANGLES: usize = 8;
const MAX_BVH_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    fn from_vertices(vertices: &[Vec3]) -> Self {
        let min = vertices
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |bounds, &vertex| {
                bounds.min(vertex)
            });
        let max = vertices
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |bounds, &vertex| {
                bounds.max(vertex)
            });
        Self { min, max }
    }

    fn from_triangle(vertices: &[Vec3], triangle: Triangle) -> Self {
        let [a, b, c] = triangle.vertices.map(|index| vertices[index as usize]);
        Self {
            min: a.min(b).min(c),
            max: a.max(b).max(c),
        }
    }

    fn include(mut self, other: Self) -> Self {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self
    }

    fn extent(self) -> Vec3 {
        self.max - self.min
    }

    fn entry_distance(self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> Option<f32> {
        let mut enter = t_min;
        let mut exit = t_max;
        for axis in 0..3 {
            if direction[axis].abs() < 1e-8 {
                if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
                    return None;
                }
                continue;
            }

            let inverse_direction = direction[axis].recip();
            let mut near = (self.min[axis] - origin[axis]) * inverse_direction;
            let mut far = (self.max[axis] - origin[axis]) * inverse_direction;
            if near > far {
                std::mem::swap(&mut near, &mut far);
            }
            enter = enter.max(near);
            exit = exit.min(far);
            if exit < enter {
                return None;
            }
        }
        Some(enter)
    }

    fn hit(self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> bool {
        self.entry_distance(origin, direction, t_min, t_max)
            .is_some()
    }
}

enum BvhNode {
    Branch {
        bounds: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
    Leaf {
        bounds: Aabb,
        triangles: Box<[usize]>,
    },
}

impl BvhNode {
    fn build(vertices: &[Vec3], triangles: &[Triangle]) -> Self {
        let indices = (0..triangles.len()).collect();
        Self::build_recursive(vertices, triangles, indices, 0)
    }

    fn build_recursive(
        vertices: &[Vec3],
        triangles: &[Triangle],
        mut indices: Vec<usize>,
        depth: usize,
    ) -> Self {
        let bounds = indices.iter().fold(Aabb::empty(), |bounds, &index| {
            bounds.include(Aabb::from_triangle(vertices, triangles[index]))
        });

        if indices.len() <= MAX_LEAF_TRIANGLES || depth >= MAX_BVH_DEPTH {
            return Self::Leaf {
                bounds,
                triangles: indices.into_boxed_slice(),
            };
        }

        let centroid_bounds = indices.iter().fold(Aabb::empty(), |bounds, &index| {
            let centroid = triangles[index]
                .vertices
                .map(|vertex| vertices[vertex as usize])
                .into_iter()
                .sum::<Vec3>()
                / 3.0;
            bounds.include(Aabb {
                min: centroid,
                max: centroid,
            })
        });
        let extent = centroid_bounds.extent();
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };

        if extent[axis] <= f32::EPSILON {
            return Self::Leaf {
                bounds,
                triangles: indices.into_boxed_slice(),
            };
        }

        indices.sort_unstable_by(|&left, &right| {
            let left_centroid = triangles[left]
                .vertices
                .map(|vertex| vertices[vertex as usize][axis])
                .into_iter()
                .sum::<f32>()
                / 3.0;
            let right_centroid = triangles[right]
                .vertices
                .map(|vertex| vertices[vertex as usize][axis])
                .into_iter()
                .sum::<f32>()
                / 3.0;
            left_centroid.total_cmp(&right_centroid)
        });

        let right_indices = indices.split_off(indices.len() / 2);
        let left = Box::new(Self::build_recursive(
            vertices,
            triangles,
            indices,
            depth + 1,
        ));
        let right = Box::new(Self::build_recursive(
            vertices,
            triangles,
            right_indices,
            depth + 1,
        ));
        Self::Branch {
            bounds,
            left,
            right,
        }
    }

    fn bounds(&self) -> Aabb {
        match self {
            Self::Branch { bounds, .. } | Self::Leaf { bounds, .. } => *bounds,
        }
    }

    fn closest_hit(
        &self,
        mesh: &PolyMesh,
        origin: Vec3,
        direction: Vec3,
        t_min: f32,
        closest_t: &mut f32,
        closest: &mut Option<(usize, TriangleHit)>,
    ) {
        if self
            .bounds()
            .entry_distance(origin, direction, t_min, *closest_t)
            .is_none()
        {
            return;
        }

        match self {
            Self::Leaf { triangles, .. } => {
                for &triangle_index in triangles {
                    if let Some(hit) = mesh.triangles[triangle_index]
                        .hit(mesh, origin, direction, t_min, *closest_t)
                    {
                        *closest_t = hit.t;
                        *closest = Some((triangle_index, hit));
                    }
                }
            }
            Self::Branch { left, right, .. } => {
                let left_entry = left
                    .bounds()
                    .entry_distance(origin, direction, t_min, *closest_t);
                let right_entry = right
                    .bounds()
                    .entry_distance(origin, direction, t_min, *closest_t);

                match (left_entry, right_entry) {
                    (Some(left_t), Some(right_t)) if right_t < left_t => {
                        right.closest_hit(mesh, origin, direction, t_min, closest_t, closest);
                        left.closest_hit(mesh, origin, direction, t_min, closest_t, closest);
                    }
                    (Some(_), Some(_)) => {
                        left.closest_hit(mesh, origin, direction, t_min, closest_t, closest);
                        right.closest_hit(mesh, origin, direction, t_min, closest_t, closest);
                    }
                    (Some(_), None) => {
                        left.closest_hit(mesh, origin, direction, t_min, closest_t, closest);
                    }
                    (None, Some(_)) => {
                        right.closest_hit(mesh, origin, direction, t_min, closest_t, closest);
                    }
                    (None, None) => {}
                }
            }
        }
    }
}

/// Three position indices and three independently indexed vertex normals.
///
/// Keeping the two index arrays separate lets one position use different
/// normals on different faces, which is how OBJ represents hard edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle {
    pub vertices: [u32; 3],
    pub normals: [u32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleHit {
    pub t: f32,
    pub beta: f32,
    pub gamma: f32,
}

impl TriangleHit {
    /// Weights for corners 0, 1, and 2. They always add up to one.
    pub fn barycentric(self) -> [f32; 3] {
        [1.0 - self.beta - self.gamma, self.beta, self.gamma]
    }
}

impl Triangle {
    pub fn positions(self, mesh: &PolyMesh) -> [Vec3; 3] {
        self.vertices.map(|index| mesh.vertices[index as usize])
    }

    pub fn vertex_normals(self, mesh: &PolyMesh) -> [Vec3; 3] {
        self.normals.map(|index| mesh.normals[index as usize])
    }

    /// Intersects one triangle with the Moller-Trumbore method.
    pub fn hit(
        self,
        mesh: &PolyMesh,
        origin: Vec3,
        direction: Vec3,
        t_min: f32,
        t_max: f32,
    ) -> Option<TriangleHit> {
        const PARALLEL_EPSILON: f32 = 1e-8;

        let [a, b, c] = self.positions(mesh);
        let ab = b - a;
        let ac = c - a;
        let p = direction.cross(ac);
        let determinant = ab.dot(p);
        if determinant.abs() < PARALLEL_EPSILON {
            return None;
        }

        let inverse_determinant = determinant.recip();
        let from_a = origin - a;
        let beta = from_a.dot(p) * inverse_determinant;
        if !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let q = from_a.cross(ab);
        let gamma = direction.dot(q) * inverse_determinant;
        if gamma < 0.0 || beta + gamma > 1.0 {
            return None;
        }

        let t = ac.dot(q) * inverse_determinant;
        if t < t_min || t > t_max {
            return None;
        }

        Some(TriangleHit { t, beta, gamma })
    }
}

/// Indexed geometry plus render data for one polygon mesh.
pub struct PolyMesh {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub triangles: Vec<Triangle>,
    pub color: [u8; 4],
    pub material: Rc<Material>,
    pub transform: Transform,
    bounds: Aabb,
    bvh: BvhNode,
}

impl PolyMesh {
    pub fn new(
        vertices: Vec<Vec3>,
        normals: Vec<Vec3>,
        triangles: Vec<Triangle>,
        color: [u8; 4],
        material: Material,
    ) -> Self {
        assert!(
            triangles.iter().all(|triangle| triangle
                .vertices
                .iter()
                .all(|&index| (index as usize) < vertices.len())),
            "triangle position index is out of bounds"
        );
        assert!(
            triangles.iter().all(|triangle| triangle
                .normals
                .iter()
                .all(|&index| (index as usize) < normals.len())),
            "triangle normal index is out of bounds"
        );

        let bounds = Aabb::from_vertices(&vertices);
        let bvh = BvhNode::build(&vertices, &triangles);

        Self {
            vertices,
            normals,
            triangles,
            color,
            material: Rc::new(material),
            transform: Transform::IDENTITY,
            bounds,
            bvh,
        }
    }

    /// Iterates through triangles and fetches their three positions and normals.
    pub fn triangle_data(&self) -> impl Iterator<Item = ([Vec3; 3], [Vec3; 3])> + '_ {
        self.triangles
            .iter()
            .map(|&triangle| (triangle.positions(self), triangle.vertex_normals(self)))
    }

    pub fn face_normal(&self, triangle: usize) -> Vec3 {
        let [a, b, c] = self.triangles[triangle].positions(self);
        (b - a).cross(c - a).normalize_or_zero()
    }

    pub fn shading_normal(&self, triangle: usize, beta: f32, gamma: f32) -> Vec3 {
        let [na, nb, nc] = self.triangles[triangle].vertex_normals(self);
        let alpha = 1.0 - beta - gamma;
        (alpha * na + beta * nb + gamma * nc)
            .try_normalize()
            .unwrap_or_else(|| self.face_normal(triangle))
    }

    /// Recomputes the bounding box after callers modify public vertex data.
    pub fn update_bounds(&mut self) {
        self.bounds = Aabb::from_vertices(&self.vertices);
        self.bvh = BvhNode::build(&self.vertices, &self.triangles);
    }

    fn bounds_hit(&self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> bool {
        self.bounds.hit(origin, direction, t_min, t_max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjError {
    pub line: usize,
    pub message: String,
}

impl ObjError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ObjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OBJ line {}: {}", self.line, self.message)
    }
}

impl Error for ObjError {}

#[derive(Clone, Copy)]
struct ObjCorner {
    vertex: u32,
    normal: Option<u32>,
}

pub const DEFAULT_CREASE_DEGREES: f32 = 35.0;

/// Reads triangle positions and vertex normals from Wavefront OBJ text.
///
/// Supported face corners are `v`, `v//vn`, and `v/vt/vn`; texture indices
/// are ignored. Faces must already be triangles. When normals are absent, the
/// loader generates corner normals using a 35-degree crease threshold.
pub fn load_obj(text: &str, color: [u8; 4], material: Material) -> Result<PolyMesh, ObjError> {
    load_obj_with_crease_angle(text, color, material, DEFAULT_CREASE_DEGREES)
}

pub fn load_obj_with_crease_angle(
    text: &str,
    color: [u8; 4],
    material: Material,
    crease_degrees: f32,
) -> Result<PolyMesh, ObjError> {
    if !(0.0..=180.0).contains(&crease_degrees) {
        return Err(ObjError::new(
            0,
            "crease angle must be between 0 and 180 degrees",
        ));
    }

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces: Vec<[ObjCorner; 3]> = Vec::new();

    for (line_index, source_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let record = source_line.split('#').next().unwrap_or_default().trim();
        if record.is_empty() {
            continue;
        }

        let mut words = record.split_whitespace();
        match words.next() {
            Some("v") => vertices.push(parse_vec3(words, line_number, "vertex")?),
            Some("vn") => normals.push(
                parse_vec3(words, line_number, "normal")?
                    .try_normalize()
                    .ok_or_else(|| ObjError::new(line_number, "normal must not be zero"))?,
            ),
            Some("f") => {
                let corners: Vec<_> = words
                    .map(|word| parse_corner(word, line_number))
                    .collect::<Result<_, _>>()?;
                if corners.len() != 3 {
                    return Err(ObjError::new(
                        line_number,
                        format!(
                            "only triangle faces are supported; found {} corners",
                            corners.len()
                        ),
                    ));
                }
                faces.push([corners[0], corners[1], corners[2]]);
            }
            _ => {}
        }
    }

    validate_obj_indices(&faces, vertices.len(), normals.len())?;

    let triangles =
        build_triangles_with_crease_normals(&vertices, &mut normals, &faces, crease_degrees);

    Ok(PolyMesh::new(vertices, normals, triangles, color, material))
}

fn parse_vec3<'a>(
    mut words: impl Iterator<Item = &'a str>,
    line: usize,
    kind: &str,
) -> Result<Vec3, ObjError> {
    let mut values = [0.0; 3];
    for (axis, value) in values.iter_mut().enumerate() {
        let word = words.next().ok_or_else(|| {
            ObjError::new(line, format!("{kind} is missing coordinate {}", axis + 1))
        })?;
        *value = word
            .parse()
            .map_err(|_| ObjError::new(line, format!("invalid {kind} coordinate `{word}`")))?;
    }
    Ok(Vec3::from_array(values))
}

fn parse_corner(word: &str, line: usize) -> Result<ObjCorner, ObjError> {
    let fields: Vec<_> = word.split('/').collect();
    let vertex = parse_obj_index(fields[0], line, "vertex")?;
    let normal = fields
        .get(2)
        .filter(|field| !field.is_empty())
        .map(|field| parse_obj_index(field, line, "normal"))
        .transpose()?;
    Ok(ObjCorner { vertex, normal })
}

fn parse_obj_index(word: &str, line: usize, kind: &str) -> Result<u32, ObjError> {
    let one_based: u32 = word
        .parse()
        .map_err(|_| ObjError::new(line, format!("invalid positive {kind} index `{word}`")))?;
    one_based
        .checked_sub(1)
        .ok_or_else(|| ObjError::new(line, "OBJ indices are one-based; zero is invalid"))
}

fn validate_obj_indices(
    faces: &[[ObjCorner; 3]],
    vertex_count: usize,
    normal_count: usize,
) -> Result<(), ObjError> {
    for face in faces {
        for corner in face {
            if corner.vertex as usize >= vertex_count {
                return Err(ObjError::new(0, "face vertex index is out of bounds"));
            }
            if corner
                .normal
                .is_some_and(|normal| normal as usize >= normal_count)
            {
                return Err(ObjError::new(0, "face normal index is out of bounds"));
            }
        }
    }
    Ok(())
}

fn build_triangles_with_crease_normals(
    vertices: &[Vec3],
    normals: &mut Vec<Vec3>,
    faces: &[[ObjCorner; 3]],
    crease_degrees: f32,
) -> Vec<Triangle> {
    let cos_crease = crease_degrees.to_radians().cos();
    let face_normals: Vec<_> = faces
        .iter()
        .map(|face| {
            let [i, j, k] = face.map(|corner| corner.vertex as usize);
            (vertices[j] - vertices[i])
                .cross(vertices[k] - vertices[i])
                .normalize_or_zero()
        })
        .collect();

    let mut around = vec![Vec::new(); vertices.len()];
    for (face_index, face) in faces.iter().enumerate() {
        for corner in face {
            around[corner.vertex as usize].push(face_index);
        }
    }

    faces
        .iter()
        .enumerate()
        .map(|(face_index, face)| {
            let mut normal_indices = [0; 3];
            for (corner_index, corner) in face.iter().enumerate() {
                normal_indices[corner_index] = if let Some(normal) = corner.normal {
                    normal
                } else {
                    let mut normal = Vec3::ZERO;
                    for &adjacent in &around[corner.vertex as usize] {
                        if face_normals[adjacent].dot(face_normals[face_index]) >= cos_crease {
                            normal += face_normals[adjacent];
                        }
                    }
                    let normal = normal.try_normalize().unwrap_or(face_normals[face_index]);
                    let index = normals.len() as u32;
                    normals.push(normal);
                    index
                };
            }
            Triangle {
                vertices: face.map(|corner| corner.vertex),
                normals: normal_indices,
            }
        })
        .collect()
}

impl Shape for PolyMesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Traverses the mesh hierarchy and returns the closest triangle hit.
    fn hit(&self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        if !self.bounds_hit(origin, direction, t_min, t_max) {
            return None;
        }

        let mut closest_t = t_max;
        let mut closest: Option<(usize, TriangleHit)> = None;

        self.bvh
            .closest_hit(self, origin, direction, t_min, &mut closest_t, &mut closest);

        closest.map(|(triangle, hit)| {
            Hit::new(
                hit.t,
                origin + hit.t * direction,
                self.shading_normal(triangle, hit.beta, hit.gamma),
                self.face_normal(triangle),
                self.color,
                Rc::clone(&self.material),
            )
            .with_triangle(triangle, hit.beta, hit.gamma)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matte() -> Material {
        Material::new(Vec3::ONE, 0.0, 1.0, Vec3::ZERO, 0.0, 1.0)
    }

    fn triangle_mesh(normals: Vec<Vec3>, normal_indices: [u32; 3]) -> PolyMesh {
        PolyMesh::new(
            vec![
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            normals,
            vec![Triangle {
                vertices: [0, 1, 2],
                normals: normal_indices,
            }],
            [255, 0, 0, 255],
            matte(),
        )
    }

    fn collect_bvh_triangles(node: &BvhNode, indices: &mut Vec<usize>) {
        match node {
            BvhNode::Branch { left, right, .. } => {
                collect_bvh_triangles(left, indices);
                collect_bvh_triangles(right, indices);
            }
            BvhNode::Leaf { triangles, .. } => {
                assert!(triangles.len() <= MAX_LEAF_TRIANGLES);
                indices.extend(triangles.iter().copied());
            }
        }
    }

    #[test]
    fn iterates_positions_and_normals_for_each_triangle() {
        let mesh = triangle_mesh(vec![Vec3::Z, Vec3::Y], [0, 1, 0]);
        let data: Vec<_> = mesh.triangle_data().collect();

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].0, mesh.vertices.as_slice());
        assert_eq!(data[0].1, [Vec3::Z, Vec3::Y, Vec3::Z]);
    }

    #[test]
    fn triangle_hit_interpolates_vertex_normals() {
        let mesh = triangle_mesh(vec![Vec3::Z, Vec3::Y, Vec3::X], [0, 1, 2]);
        let hit = mesh.triangles[0]
            .hit(&mesh, Vec3::new(0.0, 0.0, -2.0), Vec3::Z, 0.001, 10.0)
            .unwrap();

        assert!((hit.t - 2.0).abs() < 1e-6);
        assert!((hit.barycentric().iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(mesh.shading_normal(0, hit.beta, hit.gamma).is_normalized());
    }

    #[test]
    fn mesh_hit_returns_closest_triangle_hit() {
        let mesh = triangle_mesh(vec![Vec3::Z], [0, 0, 0]);
        let hit = mesh
            .hit(Vec3::new(0.0, 0.0, -2.0), Vec3::Z, 0.001, 10.0)
            .unwrap();

        assert_eq!(hit.p, Vec3::ZERO);
        assert_eq!(hit.n, Vec3::Z);
        assert_eq!(hit.ng, Vec3::Z);
        assert_eq!(hit.triangle, Some(0));
        assert!((hit.beta - 0.25).abs() < 1e-6);
        assert!((hit.gamma - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mesh_bounds_reject_a_ray_that_misses_the_mesh() {
        let mesh = triangle_mesh(vec![Vec3::Z], [0, 0, 0]);

        assert!(!mesh.bounds_hit(Vec3::new(3.0, 0.0, -2.0), Vec3::Z, 0.001, 10.0));
    }

    #[test]
    fn bvh_partitions_triangles_and_finds_the_target_leaf() {
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        for index in 0..32 {
            let x = index as f32 * 2.0;
            let first_vertex = vertices.len() as u32;
            vertices.extend([
                Vec3::new(x - 0.4, -0.4, 0.0),
                Vec3::new(x + 0.4, -0.4, 0.0),
                Vec3::new(x, 0.4, 0.0),
            ]);
            triangles.push(Triangle {
                vertices: [first_vertex, first_vertex + 1, first_vertex + 2],
                normals: [0, 0, 0],
            });
        }
        let mesh = PolyMesh::new(vertices, vec![Vec3::Z], triangles, [255; 4], matte());

        let mut partitioned = Vec::new();
        collect_bvh_triangles(&mesh.bvh, &mut partitioned);
        partitioned.sort_unstable();
        assert_eq!(partitioned, (0..32).collect::<Vec<_>>());

        let hit = mesh
            .hit(Vec3::new(20.0, 0.0, -1.0), Vec3::Z, 0.001, 10.0)
            .unwrap();
        assert_eq!(hit.triangle, Some(10));
    }

    #[test]
    fn position_can_use_different_normals_on_different_faces() {
        let mesh = PolyMesh::new(
            vec![Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z],
            vec![Vec3::Z, Vec3::Y],
            vec![
                Triangle {
                    vertices: [0, 1, 2],
                    normals: [0, 0, 0],
                },
                Triangle {
                    vertices: [0, 3, 1],
                    normals: [1, 1, 1],
                },
            ],
            [255; 4],
            matte(),
        );

        assert_eq!(mesh.triangles[0].vertices[0], mesh.triangles[1].vertices[0]);
        assert_ne!(mesh.triangles[0].normals[0], mesh.triangles[1].normals[0]);
    }

    #[test]
    fn obj_loader_keeps_position_and_normal_indices_independent() {
        let obj = "\
v 0 0 0
v 1 0 0
v 0 1 0
v 0 0 1
vn 0 0 1
vn 0 1 0
f 1//1 2//1 3//1
f 1//2 4//2 2//2
";
        let mesh = load_obj(obj, [255; 4], matte()).unwrap();

        assert_eq!(mesh.triangles[0].vertices[0], mesh.triangles[1].vertices[0]);
        assert_ne!(mesh.triangles[0].normals[0], mesh.triangles[1].normals[0]);
    }

    #[test]
    fn obj_loader_computes_missing_normals() {
        let mesh = load_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3", [255; 4], matte()).unwrap();

        assert_eq!(mesh.normals, vec![Vec3::Z; 3]);
        assert_eq!(mesh.triangles[0].normals, [0, 1, 2]);
    }

    #[test]
    fn obj_loader_preserves_a_hard_edge_at_the_default_crease_angle() {
        let obj = "\
v 0 0 0
v 1 0 0
v 0 1 0
v 0 0 1
f 1 2 3
f 1 4 2
";
        let hard = load_obj(obj, [255; 4], matte()).unwrap();
        let smooth = load_obj_with_crease_angle(obj, [255; 4], matte(), 180.0).unwrap();

        let hard_first = hard.normals[hard.triangles[0].normals[0] as usize];
        let hard_second = hard.normals[hard.triangles[1].normals[0] as usize];
        assert_eq!(hard_first, Vec3::Z);
        assert_eq!(hard_second, Vec3::Y);

        let smooth_first = smooth.normals[smooth.triangles[0].normals[0] as usize];
        let smooth_second = smooth.normals[smooth.triangles[1].normals[0] as usize];
        assert!(smooth_first.abs_diff_eq(smooth_second, 1e-6));
        assert!(smooth_first.abs_diff_eq((Vec3::Y + Vec3::Z).normalize(), 1e-6));
    }

    #[test]
    fn obj_loader_rejects_non_triangle_faces() {
        let error = load_obj(
            "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4",
            [255; 4],
            matte(),
        )
        .err()
        .unwrap();

        assert!(error.message.contains("only triangle faces"));
    }

    #[test]
    fn bundled_obj_models_load() {
        let capuchino =
            load_obj(include_str!("../objects/capuchino.obj"), [255; 4], matte()).unwrap();
        let vase = load_obj(include_str!("../objects/vase.obj"), [255; 4], matte()).unwrap();
        let suzanne = load_obj(include_str!("../objects/suzanne.obj"), [255; 4], matte()).unwrap();

        assert_eq!(
            (capuchino.vertices.len(), capuchino.triangles.len()),
            (33_120, 50_000)
        );
        assert_eq!(
            (vase.vertices.len(), vase.triangles.len()),
            (36_944, 73_884)
        );
        assert_eq!(
            (suzanne.vertices.len(), suzanne.triangles.len()),
            (511, 968)
        );
        assert_eq!(vase.normals.len(), vase.triangles.len() * 3);
    }
}
