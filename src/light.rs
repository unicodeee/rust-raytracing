use glam::Vec3;

pub struct Light {
    pub origin: Vec3,
    pub color_intensity: Vec3, // I
}



impl Light {
    pub fn new(origin: Vec3, color_intensity: Vec3) -> Self {
        Self {
            origin,
            // color_intensity,
            color_intensity: color_intensity.clamp(Vec3::ZERO, Vec3::ONE ),
        }
        
    }
}


