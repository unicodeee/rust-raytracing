use glam::Vec3;
use rust_learn::camera::Camera;
use rust_learn::film::Film;
use rust_learn::render::render;
use rust_learn::sphere::{Shape, Sphere, Plane};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const BACKGROUND: [u8; 4] = [20, 20, 30, 255];

fn make_spheres() -> Vec<Box<dyn Shape>> {
    vec![
        Box::new(Sphere::new(
            Vec3::new(0.0, 0.0, 0.0),
            0.55,
            [220, 30, 30, 255],
        )),
        Box::new(Sphere::new(
            Vec3::new(0.55, 0.05, 0.45),
            0.55,
            [30, 200, 60, 255],
        )),
        Box::new(Sphere::new(
            Vec3::new(-0.45, 0.15, 0.75),
            0.55,
            [40, 80, 220, 255],
        )),
        Box::new(Plane::new(
            Vec3::new(0.0, -0.6, 0.0),
            Vec3::Y,
        )),
    ]
}

fn render_view(filename: &str, eye: Vec3, target: Vec3) -> Result<(), Box<dyn std::error::Error>> {
    let spheres = make_spheres();

    let camera = Camera::new(eye, target, Vec3::Y, WIDTH as f32 / HEIGHT as f32, 45.0);

    let film: Film = render(&spheres, &camera, WIDTH, HEIGHT, BACKGROUND);

    let path = std::env::current_dir()?.join(filename);
    film.save_png(&path)?;
    println!("Image created at: {}", path.display());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_view(
        "output_front.png",
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 0.0, 0.35),
    )?;

    render_view(
        "output_left.png",
        Vec3::new(-3.5, 1.0, -3.0),
        Vec3::new(0.0, 0.0, 0.35),
    )?;

    render_view(
        "output_right.png",
        Vec3::new(3.5, 1.2, -3.0),
        Vec3::new(0.0, 0.0, 0.35),
    )?;

    Ok(())
}
