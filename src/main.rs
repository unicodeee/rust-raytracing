use glam::Vec3;
use rust_learn::camera::Camera;
use rust_learn::film::Film;
use rust_learn::sphere::{Shape, Sphere};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;

    let mut film = Film::new(WIDTH, HEIGHT);
    let sphere_center = Vec3::new(0.0, 0.0, -1.0);
    let sphere = Sphere::new(sphere_center, 0.1, [255, 0, 0, 255]);

    let camera = Camera::new(
        Vec3::ZERO,
        sphere_center,
        Vec3::Y,
        WIDTH as f32 / HEIGHT as f32,
        45.0,
    );

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let s = (x as f32 + 0.5) / WIDTH as f32;
            let t = 1.0 - (y as f32 + 0.5) / HEIGHT as f32;
            let ray = camera.ray(s, t);

            let color = sphere
                .hit_with_ray(&ray, 0.001, f32::INFINITY)
                .map(|_| sphere.color)
                .unwrap_or([0, 0, 0, 255]);

            film.set_pixel(x, y, color);
        }
    }

    let path = std::env::current_dir()?.join("output.png");
    film.save_png(&path)?;
    println!("Image created at: {}", path.display());

    Ok(())
}
