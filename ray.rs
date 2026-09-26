use glam::{Mat4, Vec3, Vec4};

use std::fmt;

// position: ray  = origin + scale * direction
// p(t) = o + t*d

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin: origin,
            direction: direction.normalize(),
        }
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn scale(&self, t: f32) -> Vec3 {
        // position: ray  = oririnal + scale * direction
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

fn main() {
    // let m = Mat4::from_cols(
    //     Vec4::new(1.0, 0.0, 0.0, 0.0), // Column 0 (X axis)
    //     Vec4::new(0.0, 1.0, 0.0, 0.0), // Column 1 (Y axis)
    //     Vec4::new(0.0, 0.0, 1.0, 0.0), // Column 2 (Z axis)
    //     Vec4::new(5.0, 10.0, 15.0, 1.0), // Column 3 (Translation / W axis)
    // );

    let o = Vec3::new(1.0, 1.0, 1.0);
    let d = Vec3::new(1.0, 2.0, 4.0);

    let ray = Ray::new(o, d);

    print!("\n{}", ray);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_ray() {
        let origin = Vec3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(0.0, 0.0, 2.0);

        let ray = Ray::new(origin, direction);

        assert_eq!(ray.origin(), origin);
        assert_eq!(ray.direction(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn evaluates_point_on_ray() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(ray.scale(5.0), Vec3::new(0.0, 0.0, 5.0));
    }

    #[test]
    fn transforms_ray() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, 1.0));

        let transform = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let transformed_ray = ray.transform(transform);

        assert_eq!(transformed_ray.origin(), Vec3::new(11.0, 2.0, 3.0));

        assert_eq!(transformed_ray.direction(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_ray_direction_is_normalized() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 5.0));

        assert_eq!(ray.direction(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn displays_ray() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, 1.0));

        let output = format!("{}", ray);

        assert!(output.contains("Ray"));
        assert!(output.contains("origin"));
        assert!(output.contains("direction"));
    }
}
