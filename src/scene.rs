use crate::light::Light;
use crate::ray::Ray;
use crate::shapes::{Hit, Material, Plane, Shape, Sphere};
use glam::Vec3;

const RAY_EPSILON: f32 = 0.001;
const MAX_REFLECTION_DEPTH: u32 = 30;

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
        let reflectivity = Vec3::splat(0.15);
        let plane_material = Material::new(Vec3::splat(0.5), 0.4, 1000.0, Vec3::splat(0.05));
        let objects: Vec<Box<dyn Shape>> = vec![
            Box::new(Sphere::new(
                Vec3::new(-0.9, 0.0, 0.6),
                0.9,
                [220, 30, 30, 255],
                Material::new(Vec3::splat(0.5), 0.4, 10000.0, reflectivity),
            )),
            Box::new(Sphere::new(
                Vec3::new(1.0, -0.3, 0.7),
                0.6,
                [30, 200, 60, 255],
                Material::new(Vec3::splat(0.5), 0.4, 1000.0, Vec3::ONE),
            )),
            Box::new(Sphere::new(
                Vec3::new(-0.1, -0.55, -0.9),
                0.35,
                [40, 80, 220, 255],
                Material::new(Vec3::splat(0.5), 0.4, 100.0, reflectivity),
            )),
            Box::new(Sphere::new(
                Vec3::new(0.95, -0.65, -0.9),
                0.25,
                [230, 190, 30, 255],
                Material::new(Vec3::splat(0.5), 0.4, 10.0, reflectivity),
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
        Self::new(objects, lights, [20, 200, 255, 255])
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

    pub fn trace(&self, ray: &Ray) -> [u8; 4] {
        to_rgba8(self.trace_color(ray, MAX_REFLECTION_DEPTH))
    }

    fn trace_color(&self, ray: &Ray, depth: u32) -> Vec3 {
        let Some(hit) = self.closest_hit(ray) else {
            return background_to_vec3(self.background);
        };

        let local_color = self.shade(&hit, ray);
        let reflectivity = hit.material.reflectivity;

        if depth == 0 || reflectivity.length_squared() < 0.0001 {
            return local_color;
        }

        let reflected_direction = ray.direction().reflect(hit.n).normalize();
        let reflected_origin = hit.p + RAY_EPSILON * hit.n;
        let reflected_ray = Ray::new(reflected_origin, reflected_direction);
        let reflected_color = self.trace_color(&reflected_ray, depth - 1);

        // if reflectivity high, local color low
        local_color * (Vec3::ONE - reflectivity) + reflected_color * reflectivity
    }

    pub fn shade(&self, hit: &Hit, ray: &Ray) -> Vec3 {
        let view_direction = -ray.direction();
        let mut lighting = Vec3::ZERO;
        let surface_color = Vec3::new(
            hit.color[0] as f32,
            hit.color[1] as f32,
            hit.color[2] as f32,
        ) / 255.0;

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
            let diffuse = surface_color
                * light_power
                * hit.material.diffuse_coefficient
                * light.color_intensity;
            let half_vector = (view_direction + light_direction).normalize_or_zero();
            let specular_power = hit
                .n
                .dot(half_vector)
                .max(0.0)
                .powf(hit.material.phong_constant);
            let specular = hit.material.specularity * light.color_intensity * specular_power;
            lighting += diffuse + specular;
        }

        lighting + surface_color * 0.4
    }
}

fn background_to_vec3(background: [u8; 4]) -> Vec3 {
    Vec3::new(
        background[0] as f32,
        background[1] as f32,
        background[2] as f32,
    ) / 255.0
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
