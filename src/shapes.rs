use crate::ray::Ray;
use glam::{Quat, Vec3};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
}

pub trait Shape {
    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit>;

    fn transform(&self) -> &Transform;

    fn hit_with_ray(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        self.hit(ray.origin(), ray.direction(), t_min, t_max)
    }
}

pub struct Material {
    pub diffuse_coefficient: Vec3,
    pub specularity: f32,
    pub phong_constant: f32,
    pub reflectivity: Vec3,
    pub k_t: f32,
    pub ior: f32,
}

impl Material {
    pub fn new(
        diffuse_coefficient: Vec3,
        specularity: f32,
        phong_constant: f32,
        reflectivity: Vec3,
        k_t: f32,
        ior: f32,
    ) -> Self {
        Self {
            diffuse_coefficient,
            specularity,
            phong_constant,
            reflectivity: reflectivity.clamp(Vec3::ZERO, Vec3::ONE),
            k_t,
            ior,
        }
    }
}

pub struct Plane {
    origin: Vec3,
    up: Vec3,
    color: [u8; 4],
    material: Rc<Material>,
    pub transform: Transform,
}

impl Plane {
    pub fn new(origin: Vec3, up: Vec3, material: Material) -> Self {
        Self::with_color(origin, up, [135, 206, 250, 255], material)
    }

    pub fn with_color(origin: Vec3, up: Vec3, color: [u8; 4], material: Material) -> Self {
        Self {
            origin,
            up: up.try_normalize().expect("plane normal must not be zero"),
            color,
            material: Rc::new(material),
            transform: Transform::IDENTITY,
        }
    }
}

impl Shape for Plane {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        let denominator = d.dot(self.up);

        if denominator.abs() < 1e-6 {
            return None;
        }

        let t: f32 = (self.origin - e).dot(self.up) / (d.dot(self.up));

        if t < t_min || t > t_max {
            return None;
        };
        let p = e + t * d;

        Some(Hit::new(
            t,
            p,
            self.up,
            self.up,
            self.color,
            Rc::clone(&self.material),
        ))
    }
}

pub struct Sphere {
    pub origin: Vec3,
    pub radius: f32,
    pub color: [u8; 4],
    pub material: Rc<Material>,
    pub transform: Transform,
}

pub struct Hit {
    pub t: f32,
    pub p: Vec3,
    pub n: Vec3,
    pub ng: Vec3,
    pub triangle: Option<usize>,
    pub beta: f32,
    pub gamma: f32,
    pub color: [u8; 4],
    pub material: Rc<Material>,
}

impl Hit {
    pub(crate) fn new(
        t: f32,
        p: Vec3,
        n: Vec3,
        ng: Vec3,
        color: [u8; 4],
        material: Rc<Material>,
    ) -> Self {
        Self {
            t,
            p,
            n: n.normalize(),
            ng: ng.normalize(),
            triangle: None,
            beta: 0.0,
            gamma: 0.0,
            color,
            material,
        }
    }

    pub(crate) fn with_triangle(mut self, triangle: usize, beta: f32, gamma: f32) -> Self {
        self.triangle = Some(triangle);
        self.beta = beta;
        self.gamma = gamma;
        self
    }
}

impl Sphere {
    pub fn new(origin: Vec3, radius: f32, color: [u8; 4], material: Material) -> Self {
        Self {
            origin,
            radius,
            color,
            material: Rc::new(material),
            transform: Transform::IDENTITY,
        }
    }
}

impl Shape for Sphere {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        let ec = e - self.origin;
        let a = d.dot(d);
        let b = 2.0 * d.dot(ec);
        let c = ec.dot(ec) - self.radius * self.radius;

        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }
        let sq = disc.sqrt();
        let mut t = (-b - sq) / (2.0 * a);

        if t < t_min || t > t_max {
            t = (-b + sq) / (2.0 * a);
            if t < t_min || t > t_max {
                return None;
            }
        }

        let p = e + t * d;
        let normal = (p - self.origin) / self.radius;
        Some(Hit::new(
            t,
            p,
            normal,
            normal,
            self.color,
            Rc::clone(&self.material),
        ))
    }
}
