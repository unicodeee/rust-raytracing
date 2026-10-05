use crate::light::Light;
use crate::ray::Ray;
use crate::shapes::{Hit, Shape};
use glam::Vec3;

const RAY_EPSILON: f32 = 0.001;

pub fn closest_hit(objects: &[Box<dyn Shape>], ray: &Ray, lights: &[Light]) -> Option<Hit> {
    let mut closest_t = f32::INFINITY;
    let mut closest_hit = None;

    // First find the nearest object hit by the camera ray.
    for object in objects {
        if let Some(hit) = object.hit_with_ray(ray, RAY_EPSILON, closest_t) {
            closest_t = hit.t;
            closest_hit = Some(hit);
        }
    }

    // Only the nearest visible surface should be shaded.
    let mut hit = closest_hit?; // If Some(hit): unwrap it and continue.
                                    // If None: immediately return None from the function. because "?"
    let mut lighting = Vec3::ZERO; // L = ...
    let view_direction = -ray.direction();

    for light in lights {
        let to_light = light.origin - hit.p;
        let t_light = to_light.length();
        let light_direction = to_light / t_light;

        // Move the origin slightly off the surface to avoid self-intersection.
        let shadow_origin = hit.p + RAY_EPSILON * hit.n;
        let shadow_ray = Ray::new(shadow_origin, light_direction);
        let blocked = objects.iter().any(|object| {
            object
                .hit_with_ray(&shadow_ray, RAY_EPSILON, t_light - RAY_EPSILON)
                .is_some()
        });

        if blocked {
            continue;
        }

        let light_power = hit.n.dot(light_direction).clamp(0.0, 1.0);
        let kd = hit.material.diffuse_coefficient;
        let lambert_diffuse = light_power * kd * light.color_intensity;

        let half_vector = (view_direction + light_direction).normalize_or_zero();
        let specular_power = hit
            .n
            .dot(half_vector)
            .max(0.0)
            .powf(hit.material.phong_constant);
        let blinn_phong = hit.material.specularity * light.color_intensity * specular_power;

        lighting += lambert_diffuse + blinn_phong;
    }

    let ambient_color = Vec3::new(
        hit.color[0] as f32,
        hit.color[1] as f32,
        hit.color[2] as f32,
    ) / 255.0;
    lighting += ambient_color * 0.4;

    hit.color = to_rgba8(lighting);
    Some(hit)
}

fn to_rgba8(color: Vec3) -> [u8; 4] {
    let color = color.clamp(Vec3::ZERO, Vec3::ONE);

    [
        (color.x * 255.0).round() as u8,
        (color.y * 255.0).round() as u8,
        (color.z * 255.0).round() as u8,
        255,
    ]
}
