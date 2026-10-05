use crate::ray::Ray;
use glam::Vec3;
use std::rc::Rc;

pub trait Shape {
    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit>;

    fn hit_with_ray(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        self.hit(ray.origin(), ray.direction(), t_min, t_max)
    }
}

pub struct Material {
    pub diffuse_coefficient: Vec3,
    pub specularity: f32,    // ks
    pub phong_constant: f32, // shininess
}

impl Material {
    pub fn new(diffuse_coefficient: Vec3, specularity: f32, phong_constant: f32) -> Self {
        Self {
            diffuse_coefficient,
            specularity,
            phong_constant,
        }
    }
}

pub struct Plane {
    origin: Vec3,
    up: Vec3, // up vector
    color: [u8; 4],
    material: Rc<Material>,
}

impl Plane {
    pub fn new(origin: Vec3, up: Vec3, material: Material) -> Self {
        let default_color: [u8; 4] = [135, 206, 250, 255];
        Self {
            origin,
            up: up.try_normalize().expect("plane normal must not be zero"),
            color: default_color,
            material: Rc::new(material),
        }
    }
}

impl Shape for Plane {
    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        let denominator = d.dot(self.up);

        // The ray is parallel, or nearly parallel, to the plane.
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
}

pub struct Hit {
    pub t: f32,  // distance
    pub p: Vec3, // hit at this point
    pub n: Vec3, //
    pub color: [u8; 4],
    pub material: Rc<Material>,
}

impl Hit {
    fn new(t: f32, p: Vec3, n: Vec3, color: [u8; 4], material: Rc<Material>) -> Self {
        Self {
            t,
            p,
            n: n.normalize(),
            color,
            // to do: impl for each shape
            material,
        }
    }
}

impl Sphere {
    pub fn new(origin: Vec3, radius: f32, color: [u8; 4], material: Material) -> Self {
        Self {
            origin,
            radius,
            color,
            material: Rc::new(material),
        }
    }
}

impl Shape for Sphere {
    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        let ec = e - self.origin;
        let a = d.dot(d); // A
        let b = 2.0 * d.dot(ec); // B
        let c = ec.dot(ec) - self.radius * self.radius; // C

        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        } // the ray misses
        let sq = disc.sqrt();
        let mut t = (-b - sq) / (2.0 * a); // near root first

        if t < t_min || t > t_max {
            t = (-b + sq) / (2.0 * a); // ...else the far root
            if t < t_min || t > t_max {
                return None;
            }
        }

        let p = e + t * d;
        Some(Hit::new(
            t,
            p,
            (p - self.origin) / self.radius,
            self.color,
            Rc::clone(&self.material),
        ))
    }
}
