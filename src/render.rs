use crate::camera::Camera;
use crate::film::Film;
use crate::light::Light;
use crate::scene::closest_hit;
use crate::shapes::Shape;
use glam::Vec3;

pub fn render(
    objects: &[Box<dyn Shape>],
    camera: &Camera,
    width: u32,
    height: u32,
    background: [u8; 4],
) -> Film {
    let mut film = Film::new(width, height);

    let light1 = Light::new(Vec3::new(-4.0, 100.0, -4.0), Vec3::splat(1.0));

    let light2 = Light::new(Vec3::new(4.0, 4.0, -4.0), Vec3::splat(1.0));



    let neon_light = Light::new(
        Vec3::new(-4.0, 4.0, -4.0),
        Vec3::new(0.0, 1.0, 1.0), // cyan
    );

    let purple_light = Light::new(
        // Vec3::new(100.0, 4.0, -4.0),
        Vec3::new(4.0, 4.0, -4.0),
        Vec3::new(0.8, 0.1, 1.0), // purple
    );

    let yellow_light = Light::new(
        Vec3::new(-4.0, 4.0, -4.0),
        Vec3::new(1.0, 1.0, 0.0), // purple
    );


    let lights = vec![light1, light2];
    // let lights = vec![yellow_light, purple_light];

    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = 1.0 - (y as f32 + 0.5) / height as f32;
            let ray = camera.ray(s, t);

            let color = closest_hit(objects, &ray, &lights)
                .map(|hit| hit.color)
                .unwrap_or(background);

            //

            film.set_pixel(x, y, color);
        }
    }

    film
}
