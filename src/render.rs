use crate::camera::Camera;
use crate::film::Film;
use crate::scene::Scene;

pub fn render(scene: &Scene, camera: &Camera, width: u32, height: u32) -> Film {
    let mut film = Film::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = 1.0 - (y as f32 + 0.5) / height as f32;
            let ray = camera.ray(s, t);
            let color = scene
                .closest_hit(&ray)
                .map(|hit| scene.shade(&hit, &ray))
                .unwrap_or(scene.background);
            film.set_pixel(x, y, color);
        }
    }
    film
}
