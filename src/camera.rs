use glam::Vec3;

use crate::ray::Ray;

pub struct Camera {
    eye: Vec3,
    right: Vec3,
    up: Vec3,
    back: Vec3,
    aspect: f32,
    field_of_view_degrees: f32,
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
        // w points backward, from the target toward the eye. -w
        // points from the eye toward the target.
        let w = (eye - target).normalize();
        let right = world_up.cross(w).normalize(); // u
        let up = w.cross(right).normalize();
        Self {
            eye,
            right,
            up,
            back: w,
            aspect,
            field_of_view_degrees,
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
        let view_scale = (self.field_of_view_degrees.to_radians() * 0.5).tan();
        let x = (2.0 * s - 1.0) * self.aspect * view_scale;
        let y = (2.0 * t - 1.0) * view_scale;
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
        close(camera.back(), Vec3::Z);
        close(camera.right(), Vec3::X);
        close(camera.up(), Vec3::Y);
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
