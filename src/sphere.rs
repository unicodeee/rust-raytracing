use crate::ray::Ray;
use glam::Vec3;

pub trait Shape {
    fn hit(&self, e: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit>;

    fn hit_with_ray(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        self.hit(ray.origin(), ray.direction(), t_min, t_max)
    }
}

pub struct Plane {
    origin: Vec3,
    up: Vec3, // up vector
    color: [u8;4],
}

impl Plane {

    pub fn new(origin: Vec3, up: Vec3) -> Self {
        let DEFAULT_COLOR: [u8;4] = [100, 0, 0, 255];
        Self {
            origin,
            up: up
                .try_normalize()
                .expect("plane normal must not be zero"),
            color: DEFAULT_COLOR,
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
            return None
        };
        let p = e + t * d;
        Some(Hit {
            t,
            p,
            n: self.up,
            color: self.color,
        })
    }

}




pub struct Sphere {
    pub origin: Vec3,
    pub radius: f32,
    pub color: [u8; 4],
}

pub struct Hit {
    pub t: f32,     // distance
    pub p: Vec3,    // hit at this point
    pub n: Vec3,    //
    pub color: [u8; 4],
}

impl Sphere {
    pub fn new(origin: Vec3, radius: f32, color: [u8; 4]) -> Self {
        Self {
            origin,
            radius,
            color,
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
        Some(Hit {
            t,
            p,
            n: (p - self.origin) / self.radius,
            color: self.color,
        })
    }
}
