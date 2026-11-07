//! A small indexed triangle-mesh shape.

use crate::shapes::{Hit, Material, Shape, Transform};
use glam::Vec3;
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

        Self {
            vertices,
            normals,
            triangles,
            color,
            material: Rc::new(material),
            transform: Transform::IDENTITY,
        }
    }

    /// Iterates through triangles and fetches their three positions and normals.
    pub fn triangle_data(&self) -> impl Iterator<Item = ([Vec3; 3], [Vec3; 3])> + '_ {
        self.triangles
            .iter()
            .map(|&triangle| (triangle.positions(self), triangle.vertex_normals(self)))
    }
}

impl Shape for PolyMesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Tests every triangle linearly and returns the closest hit.
    fn hit(&self, origin: Vec3, direction: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
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
}
