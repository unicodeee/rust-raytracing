use glam::Vec3;
use crate::ray::Ray;
use crate::sphere::{Hit, Shape};
use crate::light::Light;

pub fn closest_hit(
    // render order: loop through each ray, then each object
    objects: &[Box<dyn Shape>],
    ray: &Ray,
    lights: &Vec<Light>,
) -> Option<Hit> {
    let mut closest_t = f32::INFINITY;
    let mut closest_hit = None;

    for object in objects {
        if let Some(mut hit) = object.hit_with_ray(ray, 0.001, closest_t) {
            closest_t = hit.t;

            // light:
            // TODO: move this light set up to light.rs


            // ambient

            let mut L = Vec3::ZERO;

            for light in lights {
                let light_intensity = light.color_intensity;
                let light_origin = light.origin;


                let light_power = hit.n.dot(light_origin - hit.p).max(0.0).min(1.0);


                let k = Vec3::new(
                    hit.color[0] as f32,
                    hit.color[1] as f32,
                    hit.color[2] as f32,
                ) / 255.0; // clamp to [0, 1]


                let ka = Vec3::new(
                    hit.color[0] as f32,
                    hit.color[1] as f32,
                    hit.color[2] as f32,
                ) / 255.0;

                let ia = 0.4; // constant

                L += ka*ia +  light_power * k * light_intensity

            }
            // L = L.clamp(Vec3::ZERO, Vec3::ONE);
            hit.color = to_rgba8(L);



            closest_hit = Some(hit);
        }
    }
        closest_hit
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