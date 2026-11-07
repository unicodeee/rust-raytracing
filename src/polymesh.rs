//! Indexed triangle-mesh data and connectivity helpers.
//!
//! A mesh keeps geometry (`vertices`) separate from connectivity
//! (`triangles`). Per-vertex attributes use the same indices as the vertex
//! table.

use glam::{Vec2, Vec3};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// An undirected edge, stored with its smaller vertex index first.
pub type Edge = (u32, u32);

/// Maps each undirected edge to the triangle indices that use it.
pub type EdgeMap = HashMap<Edge, Vec<usize>>;

/// An indexed triangle mesh.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// The vertex table: one position per vertex.
    pub vertices: Vec<Vec3>,
    /// The index table: three counter-clockwise vertex indices per face.
    pub triangles: Vec<[u32; 3]>,
    /// Optional per-vertex normals. When present, this has one entry per vertex.
    pub normals: Vec<Vec3>,
    /// Optional per-vertex texture coordinates. When present, this has one entry per vertex.
    pub uvs: Vec<Vec2>,
}

impl Mesh {
    /// Returns the three positions belonging to triangle `t`.
    ///
    /// # Panics
    ///
    /// Panics when `t` or one of the triangle's vertex indices is out of range.
    pub fn corners(&self, t: usize) -> [Vec3; 3] {
        let [i, j, k] = self.triangles[t];
        [
            self.vertices[i as usize],
            self.vertices[j as usize],
            self.vertices[k as usize],
        ]
    }

    /// Computes the unit face normal for triangle `t`.
    ///
    /// Counter-clockwise vertices produce a normal pointing out of the front
    /// of the triangle. A degenerate triangle has the zero vector as its
    /// normal instead of producing NaNs.
    pub fn face_normal(&self, t: usize) -> Vec3 {
        let [a, b, c] = self.corners(t);
        (b - a).cross(c - a).normalize_or_zero()
    }

    /// Replaces `normals` with area-weighted smooth normals for every vertex.
    ///
    /// Degenerate faces contribute nothing. Isolated vertices and vertices
    /// whose face normals cancel receive the zero vector.
    pub fn compute_vertex_normals(&mut self) {
        let mut normals = vec![Vec3::ZERO; self.vertices.len()];

        for (t, triangle) in self.triangles.iter().enumerate() {
            let [a, b, c] = self.corners(t);
            // The unnormalized cross product has magnitude twice the face area,
            // so larger faces naturally have more influence.
            let face_normal = (b - a).cross(c - a);
            for &vertex in triangle {
                normals[vertex as usize] += face_normal;
            }
        }

        self.normals = normals.into_iter().map(Vec3::normalize_or_zero).collect();
    }
}

/// Builds the six-vertex, four-triangle mesh used in the lecture example.
pub fn example_mesh() -> Mesh {
    Mesh {
        vertices: vec![
            Vec3::new(0.0, 0.0, 0.0), // v0
            Vec3::new(2.0, 0.0, 0.0), // v1
            Vec3::new(4.0, 0.0, 0.0), // v2
            Vec3::new(1.0, 1.8, 0.0), // v3
            Vec3::new(3.0, 1.8, 0.0), // v4
            Vec3::new(2.0, 3.6, 0.0), // v5
        ],
        triangles: vec![
            [0, 1, 3], // T0
            [1, 4, 3], // T1
            [1, 2, 4], // T2
            [3, 4, 5], // T3
        ],
        ..Mesh::default()
    }
}

/// Canonicalizes an edge so `(i, j)` and `(j, i)` have the same key.
pub fn edge_key(i: u32, j: u32) -> Edge {
    if i < j { (i, j) } else { (j, i) }
}

/// Builds an edge-to-triangles adjacency map in one pass over the index table.
///
/// An entry with one triangle is a boundary edge, two is an interior edge,
/// and three or more indicates a non-manifold edge.
pub fn edge_map(mesh: &Mesh) -> EdgeMap {
    let mut map = EdgeMap::new();
    for (triangle_index, &[i, j, k]) in mesh.triangles.iter().enumerate() {
        for (a, b) in [(i, j), (j, k), (k, i)] {
            map.entry(edge_key(a, b)).or_default().push(triangle_index);
        }
    }
    map
}

/// Builds a vertex-to-triangles adjacency table.
///
/// # Panics
///
/// Panics if a triangle contains a vertex index outside the vertex table.
pub fn vertex_map(mesh: &Mesh) -> Vec<Vec<usize>> {
    let mut map = vec![Vec::new(); mesh.vertices.len()];
    for (triangle_index, triangle) in mesh.triangles.iter().enumerate() {
        for &vertex in triangle {
            map[vertex as usize].push(triangle_index);
        }
    }
    map
}

/// Returns the triangles across the three edges of triangle `t`.
///
/// On a manifold mesh this returns at most three entries. A non-manifold edge
/// can contribute more than one neighbor.
pub fn neighbours(mesh: &Mesh, edges: &EdgeMap, t: usize) -> Vec<usize> {
    let [i, j, k] = mesh.triangles[t];
    let mut neighbours: Vec<_> = [(i, j), (j, k), (k, i)]
        .iter()
        .filter_map(|&(a, b)| edges.get(&edge_key(a, b)))
        .flatten()
        .copied()
        .filter(|&other| other != t)
        .collect();
    neighbours.sort_unstable();
    neighbours
}

/// An error encountered while parsing the position and face records of an OBJ file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjError {
    line: usize,
    message: String,
}

impl ObjError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }

    /// The one-based source line containing the error.
    pub fn line(&self) -> usize {
        self.line
    }

    /// A description of the invalid OBJ record.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ObjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OBJ line {}: {}", self.line, self.message)
    }
}

impl Error for ObjError {}

/// Loads positions and faces from Wavefront OBJ text.
///
/// OBJ indices are converted from one-based to zero-based indexing. Faces
/// with more than three vertices are triangulated as a fan. Texture, normal,
/// material, and grouping records are intentionally ignored; call
/// [`Mesh::compute_vertex_normals`] afterward when smooth normals are wanted.
///
/// This minimal loader accepts face corners such as `7`, `7/3`, and `7/3/2`,
/// but only uses the position index.
pub fn try_load_obj(text: &str) -> Result<Mesh, ObjError> {
    let mut mesh = Mesh::default();

    for (line_index, original_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        // OBJ comments can occupy a full line or follow a record.
        let record = original_line.split('#').next().unwrap_or_default().trim();
        if record.is_empty() {
            continue;
        }

        let mut words = record.split_whitespace();
        match words.next() {
            Some("v") => {
                let mut coordinates = [0.0; 3];
                for (axis, coordinate) in coordinates.iter_mut().enumerate() {
                    let value = words.next().ok_or_else(|| {
                        ObjError::new(
                            line_number,
                            format!("vertex is missing coordinate {}", axis + 1),
                        )
                    })?;
                    *coordinate = value.parse::<f32>().map_err(|_| {
                        ObjError::new(line_number, format!("invalid vertex coordinate `{value}`"))
                    })?;
                }
                mesh.vertices.push(Vec3::from_array(coordinates));
            }
            Some("f") => {
                let mut indices = Vec::new();
                for corner in words {
                    let position = corner.split('/').next().unwrap_or_default();
                    let one_based = position.parse::<u32>().map_err(|_| {
                        ObjError::new(
                            line_number,
                            format!("invalid positive vertex index `{position}`"),
                        )
                    })?;
                    if one_based == 0 {
                        return Err(ObjError::new(
                            line_number,
                            "OBJ vertex indices are one-based; index 0 is invalid",
                        ));
                    }
                    let zero_based = one_based - 1;
                    if zero_based as usize >= mesh.vertices.len() {
                        return Err(ObjError::new(
                            line_number,
                            format!(
                                "vertex index {one_based} is out of range for {} vertices",
                                mesh.vertices.len()
                            ),
                        ));
                    }
                    indices.push(zero_based);
                }

                if indices.len() < 3 {
                    return Err(ObjError::new(
                        line_number,
                        "a face must contain at least three vertices",
                    ));
                }

                for i in 1..indices.len() - 1 {
                    mesh.triangles
                        .push([indices[0], indices[i], indices[i + 1]]);
                }
            }
            _ => {}
        }
    }

    Ok(mesh)
}

/// Loads a minimal Wavefront OBJ mesh, panicking if the input is malformed.
///
/// Use [`try_load_obj`] when input errors should be handled by the caller.
pub fn load_obj(text: &str) -> Mesh {
    try_load_obj(text).expect("invalid Wavefront OBJ data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_mesh_has_expected_geometry_and_connectivity() {
        let mesh = example_mesh();

        assert_eq!(mesh.vertices.len(), 6);
        assert_eq!(mesh.triangles.len(), 4);
        assert_eq!(
            mesh.corners(1),
            [
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(3.0, 1.8, 0.0),
                Vec3::new(1.0, 1.8, 0.0),
            ]
        );
    }

    #[test]
    fn adjacency_maps_find_shared_edges_and_vertices() {
        let mesh = example_mesh();
        let edges = edge_map(&mesh);
        let around = vertex_map(&mesh);

        assert_eq!(edges.len(), 9);
        assert_eq!(edges[&edge_key(1, 3)], vec![0, 1]);
        assert_eq!(edges[&edge_key(1, 2)], vec![2]);
        assert_eq!(around[1], vec![0, 1, 2]);
        assert_eq!(neighbours(&mesh, &edges, 1), vec![0, 2, 3]);
    }

    #[test]
    fn normals_follow_counter_clockwise_winding() {
        let mut mesh = example_mesh();

        assert_eq!(mesh.face_normal(0), Vec3::Z);
        mesh.compute_vertex_normals();

        assert_eq!(mesh.normals, vec![Vec3::Z; mesh.vertices.len()]);
    }

    #[test]
    fn degenerate_geometry_does_not_produce_nan_normals() {
        let mut mesh = Mesh {
            vertices: vec![Vec3::ZERO, Vec3::X],
            triangles: vec![[0, 0, 1]],
            ..Mesh::default()
        };

        assert_eq!(mesh.face_normal(0), Vec3::ZERO);
        mesh.compute_vertex_normals();

        assert_eq!(mesh.normals, vec![Vec3::ZERO; 2]);
    }

    #[test]
    fn obj_loader_converts_indices_and_triangulates_polygons() {
        let obj = "\
# square with independently indexed attributes
v 0 0 0 # first vertex
v 1 0 0
v 1 1 0
v 0 1 0
vt 0 0
vn 0 0 1
f 1/1/1 2/1/1 3/1/1 4/1/1 # quad
";

        let mut mesh = try_load_obj(obj).unwrap();

        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.triangles, vec![[0, 1, 2], [0, 2, 3]]);
        assert!(mesh.uvs.is_empty());
        mesh.compute_vertex_normals();
        assert_eq!(mesh.normals, vec![Vec3::Z; 4]);
    }

    #[test]
    fn obj_loader_reports_invalid_one_based_indices() {
        let error = try_load_obj("v 0 0 0\nf 0 1 1").unwrap_err();

        assert_eq!(error.line(), 2);
        assert!(error.message().contains("one-based"));
    }
}
