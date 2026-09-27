use crate::ray::Ray;
use crate::sphere::{Hit, Shape};

pub fn closest_hit(
    // render order: loop through each ray, then each object
    objects: &[Box<dyn Shape>],
    ray: &Ray,
) -> Option<Hit> {
    let mut closest_t = f32::INFINITY;
    let mut closest_hit = None;

    for object in objects {
        if let Some(hit) = object.hit_with_ray(ray, 0.001, closest_t) {
            closest_t = hit.t;
            closest_hit = Some(hit);
        }
    }

    closest_hit
}
