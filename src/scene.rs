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

                let I = (light_origin - hit.p).normalize();

                let light_power = hit.n.dot(I).max(0.0).min(1.0);

                let k = Vec3::new(
                    hit.color[0] as f32,
                    hit.color[1] as f32,
                    hit.color[2] as f32,
                ) / 255.0; // clamp to [0, 1]
                let lambert = light_power * k * light_intensity;


                let ka = Vec3::new(
                    hit.color[0] as f32,
                    hit.color[1] as f32,
                    hit.color[2] as f32,
                ) / 255.0;
                let ia = 0.4; // constant
                let ambient = ka*ia;

                // add ks * I * max()
                let h = (hit.p.normalize() + I).normalize();
                let ks = 0.4;
                let p = 10.0; // Phong exponent
                let max_component = (hit.n.dot(h).max(0.0)).powf(p);
                let blinn_phong = ks * light_intensity * max_component;


                L += ambient + lambert + blinn_phong;
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