use crate::light::Light;
use crate::ray::Ray;
use crate::shapes::{Hit, Material, Plane, Shape, Sphere};
use glam::Vec3;

const RAY_EPSILON: f32 = 0.001;

pub struct Scene {
    pub objects: Vec<Box<dyn Shape>>,
    pub lights: Vec<Light>,
    pub background: [u8; 4],
}

impl Scene {
    pub fn new(objects: Vec<Box<dyn Shape>>, lights: Vec<Light>, background: [u8; 4]) -> Self {
        Self {
            objects,
            lights,
            background,
        }
    }

    pub fn init() -> Self {
        let plane_material = Material::new(Vec3::splat(0.5), 0.4, 1000.0);
        let objects: Vec<Box<dyn Shape>> = vec![
            Box::new(Sphere::new(
                Vec3::new(-0.9, 0.0, 0.6),
                0.9,
                [220, 30, 30, 255],
                Material::new(Vec3::splat(0.5), 0.4, 10000.0),
            )),
            Box::new(Sphere::new(
                Vec3::new(1.0, -0.3, 0.7),
                0.6,
                [30, 200, 60, 255],
                Material::new(Vec3::splat(0.5), 0.4, 1000.0),
            )),
            Box::new(Sphere::new(
                Vec3::new(-0.1, -0.55, -0.9),
                0.35,
                [40, 80, 220, 255],
                Material::new(Vec3::splat(0.5), 0.4, 100.0),
            )),
            Box::new(Sphere::new(
                Vec3::new(0.95, -0.65, -0.9),
                0.25,
                [230, 190, 30, 255],
                Material::new(Vec3::splat(0.5), 0.4, 10.0),
            )),
            Box::new(Plane::new(
                Vec3::new(0.0, -0.9, 0.0),
                Vec3::Y,
                plane_material,
            )),
        ];
        let lights = vec![
            Light::new(Vec3::new(-4.0, 4.0, -4.0), Vec3::new(1.0, 1.0, 0.0)),
            Light::new(Vec3::new(4.0, 4.0, -4.0), Vec3::new(0.8, 0.1, 1.0)),
        ];
        Self::new(objects, lights, [20, 20, 30, 255])
    }

    pub fn closest_hit(&self, ray: &Ray) -> Option<Hit> {
        let mut closest_t = f32::INFINITY;
        let mut closest_hit = None;
        for object in &self.objects {
            if let Some(hit) = object.hit_with_ray(ray, RAY_EPSILON, closest_t) {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }
        closest_hit
    }

    pub fn shade(&self, hit: &Hit, ray: &Ray) -> [u8; 4] {
        let view_direction = -ray.direction();
        let mut lighting = Vec3::ZERO;
        for light in &self.lights {
            let to_light = light.origin - hit.p;
            let t_light = to_light.length();
            let light_direction = to_light / t_light;
            let shadow_ray = Ray::new(hit.p + RAY_EPSILON * hit.n, light_direction);
            let blocked = self.objects.iter().any(|object| {
                object
                    .hit_with_ray(&shadow_ray, RAY_EPSILON, t_light - RAY_EPSILON)
                    .is_some()
            });
            if blocked {
                continue;
            }

            let light_power = hit.n.dot(light_direction).clamp(0.0, 1.0);
            let diffuse = light_power * hit.material.diffuse_coefficient * light.color_intensity;
            let half_vector = (view_direction + light_direction).normalize_or_zero();
            let specular_power = hit
                .n
                .dot(half_vector)
                .max(0.0)
                .powf(hit.material.phong_constant);
            let specular = hit.material.specularity * light.color_intensity * specular_power;
            lighting += diffuse + specular;
        }

        let surface_color = Vec3::new(
            hit.color[0] as f32,
            hit.color[1] as f32,
            hit.color[2] as f32,
        ) / 255.0;
        to_rgba8(lighting + surface_color * 0.4)
    }
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
