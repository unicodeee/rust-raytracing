use glam::Vec3;
use crate::camera::Camera;
use crate::film::Film;
use crate::scene::closest_hit;
use crate::sphere::Shape;

pub fn render(
    objects: &[Box<dyn Shape>],
    camera: &Camera,
    width: u32,
    height: u32,
    background: [u8; 4],
) -> Film {
    let mut film = Film::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = 1.0 - (y as f32 + 0.5) / height as f32;
            let ray = camera.ray(s, t);

            let color = closest_hit(objects, &ray)
                .map(|hit| hit.color)
                .unwrap_or(background);


            //








            film.set_pixel(x, y, color);
        }
    }

    film
}
