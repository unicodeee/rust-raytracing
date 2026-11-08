//! A small indexed triangle-mesh shape.

use crate::shapes::{Hit, Material, Shape, Transform};
use glam::Vec3;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

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
    /// Weights for corners 0, 1, and 2. They always add up to one.
    pub barycentric: [f32; 3],
    pub normal: Vec3,
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
        const PARALLEL_EPSILON: f32 = 1e-7;

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

        let barycentric = [1.0 - beta - gamma, beta, gamma];
        let [na, nb, nc] = self.vertex_normals(mesh);
        let geometric_normal = ab.cross(ac).normalize_or_zero();
        if geometric_normal == Vec3::ZERO {
            return None;
        }
        let normal = (na * barycentric[0] + nb * barycentric[1] + nc * barycentric[2])
            .try_normalize()
            .unwrap_or(geometric_normal);

        Some(TriangleHit {
            t,
            barycentric,
            normal,
        })
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
    bounds_min: Vec3,
    bounds_max: Vec3,
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

        let bounds_min = vertices
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |bounds, &vertex| {
                bounds.min(vertex)
            });
        let bounds_max = vertices
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |bounds, &vertex| {
                bounds.max(vertex)
            });

        Self {
            vertices,
            normals,
            triangles,
            color,
            material: Rc::new(material),
            transform: Transform::IDENTITY,
            bounds_min,
            bounds_max,
        }
    }

    /// Iterates through triangles and fetches their three positions and normals.
    pub fn triangle_data(&self) -> impl Iterator<Item = ([Vec3; 3], [Vec3; 3])> + '_ {
        self.triangles
            .iter()
            .map(|&triangle| (triangle.positions(self), triangle.vertex_normals(self)))
    }

    /// Recomputes the bounding box after callers modify public vertex data.
    pub fn update_bounds(&mut self) {
        self.bounds_min = self
            .vertices
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |bounds, &vertex| {
                bounds.min(vertex)
            });
        self.bounds_max = self
            .vertices
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |bounds, &vertex| {
                bounds.max(vertex)
            });
    }

    fn bounds_hit(&self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> bool {
        let mut enter = t_min;
        let mut exit = t_max;
        for axis in 0..3 {
            if direction[axis].abs() < 1e-8 {
                if origin[axis] < self.bounds_min[axis] || origin[axis] > self.bounds_max[axis] {
                    return false;
                }
                continue;
            }

            let inverse_direction = direction[axis].recip();
            let mut near = (self.bounds_min[axis] - origin[axis]) * inverse_direction;
            let mut far = (self.bounds_max[axis] - origin[axis]) * inverse_direction;
            if near > far {
                std::mem::swap(&mut near, &mut far);
            }
            enter = enter.max(near);
            exit = exit.min(far);
            if exit < enter {
                return false;
            }
        }
        true
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

/// Reads triangle positions and vertex normals from Wavefront OBJ text.
///
/// Supported face corners are `v`, `v//vn`, and `v/vt/vn`; texture indices
/// are ignored. Faces must already be triangles. When normals are absent, the
/// loader computes smooth, area-weighted normals shared by position index.
pub fn load_obj(text: &str, color: [u8; 4], material: Material) -> Result<PolyMesh, ObjError> {
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

    let needs_generated_normals = faces.iter().flatten().any(|corner| corner.normal.is_none());
    let generated_normal_start = normals.len() as u32;
    if needs_generated_normals {
        normals.extend(compute_smooth_normals(&vertices, &faces));
    }

    let triangles = faces
        .into_iter()
        .map(|face| Triangle {
            vertices: face.map(|corner| corner.vertex),
            normals: face.map(|corner| {
                corner
                    .normal
                    .unwrap_or(generated_normal_start + corner.vertex)
            }),
        })
        .collect();

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

fn compute_smooth_normals(vertices: &[Vec3], faces: &[[ObjCorner; 3]]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; vertices.len()];
    for face in faces {
        let [i, j, k] = face.map(|corner| corner.vertex as usize);
        let face_normal = (vertices[j] - vertices[i]).cross(vertices[k] - vertices[i]);
        normals[i] += face_normal;
        normals[j] += face_normal;
        normals[k] += face_normal;
    }
    normals.into_iter().map(Vec3::normalize_or_zero).collect()
}

impl Shape for PolyMesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Tests every triangle linearly and returns the closest hit.
    fn hit(&self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        if !self.bounds_hit(origin, direction, t_min, t_max) {
            return None;
        }

        let mut closest_t = t_max;
        let mut closest = None;

        for &triangle in &self.triangles {
            if let Some(hit) = triangle.hit(self, origin, direction, t_min, closest_t) {
                closest_t = hit.t;
                closest = Some(Hit::new(
                    hit.t,
                    origin + hit.t * direction,
                    hit.normal,
                    self.color,
                    Rc::clone(&self.material),
                ));
            }
        }

        closest
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
        assert!((hit.barycentric.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(hit.normal.is_normalized());
    }

    #[test]
    fn mesh_hit_returns_closest_triangle_hit() {
        let mesh = triangle_mesh(vec![Vec3::Z], [0, 0, 0]);
        let hit = mesh
            .hit(Vec3::new(0.0, 0.0, -2.0), Vec3::Z, 0.001, 10.0)
            .unwrap();

        assert_eq!(hit.p, Vec3::ZERO);
        assert_eq!(hit.n, Vec3::Z);
    }

    #[test]
    fn mesh_bounds_reject_a_ray_that_misses_the_mesh() {
        let mesh = triangle_mesh(vec![Vec3::Z], [0, 0, 0]);

        assert!(!mesh.bounds_hit(Vec3::new(3.0, 0.0, -2.0), Vec3::Z, 0.001, 10.0));
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

        assert_eq!(
            (capuchino.vertices.len(), capuchino.triangles.len()),
            (33_120, 50_000)
        );
        assert_eq!(
            (vase.vertices.len(), vase.triangles.len()),
            (36_944, 73_884)
        );
        assert_eq!(vase.normals.len(), vase.vertices.len());
    }
}
