use crate::light::Light;
use crate::polymesh::{PolyMesh, load_obj};
use crate::ray::Ray;
use crate::shapes::{Hit, Material, Plane, Shape, Sphere};
use glam::Vec3;

const RAY_EPSILON: f32 = 0.001;
const MAX_REFLECTION_DEPTH: u32 = 7;

fn load_table_mesh() -> PolyMesh {
    load_obj(
        include_str!("../objects/vase_scene_table.obj"),
        [126, 79, 48, 255],
        Material::new(Vec3::splat(0.65), 0.12, 24.0, Vec3::ZERO, 0.0, 1.0),
    )
    .expect("bundled table OBJ must be valid")
}

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
        let blue = [35, 105, 235, 255];
        let plane_material =
            Material::new(Vec3::splat(0.5), 0.25, 64.0, Vec3::splat(0.05), 0.0, 0.0);
        let objects: Vec<Box<dyn Shape>> = vec![
            Box::new(Sphere::new(
                Vec3::new(-2.55, -0.1, 0.5),
                0.7,
                blue,
                Material::new(Vec3::splat(0.5), 0.5, 8.0, Vec3::splat(0.05), 0.0, 0.0),
            )),
            Box::new(Sphere::new(
                Vec3::new(-0.85, -0.1, 0.5),
                0.7,
                blue,
                Material::new(Vec3::splat(0.5), 0.5, 32.0, Vec3::splat(0.05), 0.0, 0.0),
            )),
            Box::new(Sphere::new(
                Vec3::new(0.85, -0.1, 0.5),
                0.7,
                blue,
                Material::new(Vec3::splat(0.5), 0.5, 128.0, Vec3::splat(0.05), 0.0, 0.0),
            )),
            Box::new(Sphere::new(
                Vec3::new(2.55, -0.1, 0.5),
                0.7,
                blue,
                Material::new(Vec3::splat(0.5), 0.5, 512.0, Vec3::splat(0.05), 0.0, 0.0),
            )),
            Box::new(Plane::new(
                Vec3::new(0.0, -0.9, 0.0),
                Vec3::Y,
                plane_material,
            )),
        ];
        let lights = vec![
            Light::new(Vec3::new(-4.0, 5.0, -4.0), Vec3::splat(1.0)),
            Light::new(Vec3::new(4.0, 3.0, -2.0), Vec3::splat(0.35)),
        ];
        Self::new(objects, lights, [18, 24, 42, 255])
    }

    pub fn pyramid(color: [u8; 4]) -> Self {
        Self::pyramid_from_obj(include_str!("../objects/pyramid_hard.obj"), color)
    }

    pub fn smooth_pyramid(color: [u8; 4]) -> Self {
        Self::pyramid_from_obj(include_str!("../objects/pyramid_smooth.obj"), color)
    }

    fn pyramid_from_obj(obj: &str, color: [u8; 4]) -> Self {
        let mut pyramid = load_obj(
            obj,
            color,
            Material::new(Vec3::splat(0.85), 0.35, 48.0, Vec3::splat(0.08), 0.0, 1.0),
        )
        .expect("bundled pyramid OBJ must be valid");
        for vertex in &mut pyramid.vertices {
            *vertex = *vertex * 1.2 + Vec3::new(0.0, -0.9, 0.0);
        }
        pyramid.update_bounds();
        let floor = Plane::new(
            Vec3::new(0.0, -0.91, 0.0),
            Vec3::Y,
            Material::new(Vec3::splat(0.45), 0.1, 24.0, Vec3::splat(0.04), 0.0, 1.0),
        );

        Self::new(
            vec![Box::new(pyramid), Box::new(floor)],
            vec![
                Light::new(Vec3::new(-4.0, 5.0, -4.0), Vec3::splat(1.0)),
                Light::new(Vec3::new(4.0, 2.5, -2.0), Vec3::splat(0.3)),
            ],
            [18, 24, 42, 255],
        )
    }

    pub fn vase_scene() -> Self {
        let mut vase = load_obj(
            include_str!("../objects/vase.obj"),
            [64, 174, 166, 255],
            Material::new(Vec3::splat(0.72), 0.45, 96.0, Vec3::ZERO, 0.0, 1.0),
        )
        .expect("bundled vase OBJ must be valid");
        let minimum = vase
            .vertices
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |bounds, &vertex| {
                bounds.min(vertex)
            });
        let maximum = vase
            .vertices
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |bounds, &vertex| {
                bounds.max(vertex)
            });
        let center = (minimum + maximum) * 0.5;
        let scale = 2.25 / (maximum.z - minimum.z);
        for vertex in &mut vase.vertices {
            *vertex = Vec3::new(
                (vertex.x - center.x) * scale,
                (vertex.z - minimum.z) * scale - 0.25,
                -(vertex.y - center.y) * scale,
            );
        }
        for normal in &mut vase.normals {
            *normal = Vec3::new(normal.x, normal.z, -normal.y).normalize_or_zero();
        }
        vase.update_bounds();
        let table = load_table_mesh();
        let floor = Plane::with_color(
            Vec3::new(0.0, -2.0, 0.0),
            Vec3::Y,
            [192, 202, 216, 255],
            Material::new(Vec3::splat(0.55), 0.05, 12.0, Vec3::ZERO, 0.0, 1.0),
        );
        let wall = Plane::with_color(
            Vec3::new(0.0, 0.0, 2.8),
            -Vec3::Z,
            [192, 202, 216, 255],
            Material::new(Vec3::splat(0.55), 0.05, 12.0, Vec3::ZERO, 0.0, 1.0),
        );

        Self::new(
            vec![
                Box::new(vase),
                Box::new(table),
                Box::new(floor),
                Box::new(wall),
            ],
            vec![
                Light::new(Vec3::new(-3.5, 4.5, -4.0), Vec3::new(1.0, 0.82, 0.68)),
                Light::new(Vec3::new(4.0, 2.5, -2.0), Vec3::new(0.35, 0.5, 0.8)),
                Light::new(Vec3::new(0.0, 4.0, 2.0), Vec3::splat(0.3)),
            ],
            [24, 30, 42, 255],
        )
    }

    pub fn capuchino_scene() -> Self {
        let mut model = load_obj(
            include_str!("../objects/capuchino.obj"),
            [205, 112, 82, 255],
            Material::new(Vec3::splat(0.75), 0.25, 48.0, Vec3::ZERO, 0.0, 1.0),
        )
        .expect("bundled capuchino OBJ must be valid");
        for vertex in &mut model.vertices {
            *vertex *= 1.35;
        }
        model.update_bounds();

        Self::new(
            vec![Box::new(model)],
            vec![Light::new(Vec3::new(-3.0, 4.0, -4.0), Vec3::ONE)],
            [18, 24, 42, 255],
        )
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

        let local_color = self.shade_pixel(&hit, ray);
        let reflectivity = hit.material.reflectivity;
        let k_t = hit.material.k_t.clamp(0.0, 1.0);

        if depth == 0 || (reflectivity.length_squared() < RAY_EPSILON && k_t <= RAY_EPSILON) {
            return local_color;
        }

        let entering = ray.direction().dot(hit.ng) < 0.0;
        let shading_normal = if ray.direction().dot(hit.n) < 0.0 {
            hit.n
        } else {
            -hit.n
        };
        let geometric_normal = if entering { hit.ng } else { -hit.ng };

        let reflected_direction = ray.direction().reflect(shading_normal).normalize();
        let reflected_origin = hit.p + RAY_EPSILON * geometric_normal;
        let reflected_ray = Ray::new(reflected_origin, reflected_direction);
        let reflected_color = self.trace_color(&reflected_ray, depth - 1);

        let mut fresnel = 0.0;
        let mut refracted_color = Vec3::ZERO;

        if k_t > RAY_EPSILON {
            let ior = hit.material.ior;
            let eta = if entering { 1.0 / ior } else { ior };
            let direction = ray.direction().normalize();
            let refracted_direction = direction.refract(shading_normal, eta);

            if refracted_direction.length_squared() < RAY_EPSILON {
                // Total internal reflection.
                fresnel = 1.0;
            } else {
                fresnel = schlick(
                    fresnel_cos(direction, refracted_direction, shading_normal, entering),
                    ior,
                );

                let refracted_origin = hit.p - RAY_EPSILON * geometric_normal;
                let refracted_ray = Ray::new(refracted_origin, refracted_direction.normalize());

                refracted_color = self.trace_color(&refracted_ray, depth - 1);
            }
        }

        let reflection_weight = reflectivity + Vec3::splat(k_t * fresnel);

        let transmission_weight = Vec3::splat(k_t * (1.0 - fresnel));

        let local_weight = (Vec3::ONE - reflectivity - Vec3::splat(k_t)).max(Vec3::ZERO);

        local_color * local_weight
            + reflected_color * reflection_weight
            + refracted_color * transmission_weight
    }
    pub fn shade_pixel(&self, hit: &Hit, ray: &Ray) -> Vec3 {
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
            let light_power = hit.n.dot(light_direction).clamp(0.0, 1.0);
            if light_power == 0.0 {
                continue;
            }

            // The interpolated shading normal can face the light while the
            // actual flat triangle faces away. Trust the shading normal in
            // that disagreement region to avoid a faceted shadow terminator.
            let geometric_normal = if hit.ng.dot(hit.n) >= 0.0 {
                hit.ng
            } else {
                -hit.ng
            };
            let blocked = if geometric_normal.dot(light_direction) < 0.0 {
                false
            } else {
                let shadow_ray = Ray::new(hit.p + RAY_EPSILON * geometric_normal, light_direction);
                self.objects.iter().any(|object| {
                    object
                        .hit_with_ray(&shadow_ray, RAY_EPSILON, t_light - RAY_EPSILON)
                        .is_some()
                })
            };

            if blocked {
                continue;
            }

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

fn schlick(cos_theta: f32, n_t: f32) -> f32 {
    let r0 = ((n_t - 1.0) / (n_t + 1.0)).powi(2); // head-on reflectance: 0.04 for glass
    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}
// glass: schlick(cos 0°) = 0.04 (cos 60°) = 0.07 (cos 80°) = 0.41 (cos 90°) = 1.00
/// Which cosine to feed Schlick: going IN, the incident angle theta;
/// going OUT, the angle of the REFRACTED ray t (that is the air side).
fn fresnel_cos(d: Vec3, t: Vec3, n_f: Vec3, entering: bool) -> f32 {
    if entering { -d.dot(n_f) } else { -t.dot(n_f) }
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
