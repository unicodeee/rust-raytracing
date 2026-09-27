use glam::{Mat4, Vec3};
use std::fmt;

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    pub fn origin(&self) -> Vec3 {
        self.origin
    }
    pub fn direction(&self) -> Vec3 {
        self.direction
    }
    pub fn scale(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
    pub fn transform(&self, matrix: Mat4) -> Self {
        Self {
            origin: matrix.transform_point3(self.origin),
            direction: matrix.transform_vector3(self.direction).normalize(),
        }
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Ray(origin: ({}, {}, {}), direction: ({}, {}, {}))",
            self.origin.x,
            self.origin.y,
            self.origin.z,
            self.direction.x,
            self.direction.y,
            self.direction.z
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_normalized_ray() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, 2.0));
        assert_eq!(ray.origin(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(ray.direction(), Vec3::Z);
    }

    #[test]
    fn evaluates_point_on_ray() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        assert_eq!(ray.scale(5.0), Vec3::new(0.0, 0.0, 5.0));
    }

    #[test]
    fn transforms_ray() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::Z);
        let transformed = ray.transform(Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)));
        assert_eq!(transformed.origin(), Vec3::new(11.0, 2.0, 3.0));
        assert_eq!(transformed.direction(), Vec3::Z);
    }

    #[test]
    fn displays_ray() {
        let output = format!("{}", Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::Z));
        assert!(output.contains("Ray"));
        assert!(output.contains("origin"));
        assert!(output.contains("direction"));
    }
}
