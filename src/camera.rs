use glam::Vec3;

use crate::ray::Ray;

pub struct Camera {
    eye: Vec3,
    right: Vec3,
    up: Vec3,
    back: Vec3,
    aspect: f32,
    half_height: f32,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        world_up: Vec3,
        aspect: f32,
        field_of_view_degrees: f32,
    ) -> Self {
        assert!(aspect > 0.0, "aspect ratio must be positive");
        assert!(field_of_view_degrees > 0.0 && field_of_view_degrees < 180.0);
        let back = (eye - target).normalize();
        let right = world_up.cross(back).normalize();
        let up = back.cross(right).normalize();
        let half_height = (field_of_view_degrees.to_radians() * 0.5).tan();
        Self {
            eye,
            right,
            up,
            back,
            aspect,
            half_height,
        }
    }

    pub fn eye(&self) -> Vec3 {
        self.eye
    }
    pub fn right(&self) -> Vec3 {
        self.right
    }
    pub fn up(&self) -> Vec3 {
        self.up
    }
    pub fn back(&self) -> Vec3 {
        self.back
    }

    pub fn ray(&self, s: f32, t: f32) -> Ray {
        let x = (2.0 * s - 1.0) * self.aspect * self.half_height;
        let y = (2.0 * t - 1.0) * self.half_height;
        Ray::new(self.eye, -self.back + x * self.right + y * self.up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn close(a: Vec3, b: Vec3) {
        assert!(a.distance(b) < 1e-5, "{a:?} != {b:?}");
    }

    #[test]
    fn center_ray_points_at_target() {
        let camera = Camera::new(Vec3::ZERO, -Vec3::Z, Vec3::Y, 1.0, 90.0);
        close(camera.ray(0.5, 0.5).direction(), -Vec3::Z);
    }

    #[test]
    fn corner_rays_match_the_view_plane_basis() {
        let camera = Camera::new(Vec3::ZERO, -Vec3::Z, Vec3::Y, 1.0, 90.0);
        close(
            camera.ray(0.0, 0.0).direction(),
            Vec3::new(-1.0, -1.0, -1.0).normalize(),
        );
        close(
            camera.ray(1.0, 1.0).direction(),
            Vec3::new(1.0, 1.0, -1.0).normalize(),
        );
    }
}
