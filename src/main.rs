use glam::Vec3;
use rust_learn::camera::Camera;
use rust_learn::film::Film;
use rust_learn::render::render;
use rust_learn::shapes::{Material, Plane, Shape, Sphere};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const BACKGROUND: [u8; 4] = [20, 20, 30, 255];

fn make_spheres() -> Vec<Box<dyn Shape>> {
    vec![ // RED
        Box::new(Sphere::new(
            Vec3::new(-0.9, 0.0, 0.6),
            0.9,
            [220, 30, 30, 255],
            Material::new(Vec3::new(0.5, 0.5, 0.5), 0.4, 10000.0),
        )), // GREEN
        Box::new(Sphere::new(
            Vec3::new(1.0, -0.3, 0.7),
            0.6,
            [30, 200, 60, 255],
            Material::new(Vec3::new(0.5, 0.5, 0.5), 0.4, 1000.0),
        )), // BLUE
        Box::new(Sphere::new(
            Vec3::new(-0.1, -0.55, -0.9),
            0.35,
            [40, 80, 220, 255],
            Material::new(Vec3::new(0.5, 0.5, 0.5), 0.4, 100.0),
        )), // YELLOW
        Box::new(Sphere::new(
            Vec3::new(0.95, -0.65, -0.9),
            0.25,
            [230, 190, 30, 255],
            Material::new(Vec3::new(0.5, 0.5, 0.5), 0.4, 10.0),
        )),
        Box::new(Plane::new(
            Vec3::new(0.0, -0.9, 0.0),
            Vec3::Y,
            Material::new(Vec3::new(0.5, 0.5, 0.5), 0.4, 1000.0),)),
    ]
}

fn render_view(filename: &str, eye: Vec3, target: Vec3) -> Result<(), Box<dyn std::error::Error>> {
    let objects = make_spheres();

    let camera = Camera::new(eye, target, Vec3::Y, WIDTH as f32 / HEIGHT as f32, 45.0);

    let film: Film = render(&objects, &camera, WIDTH, HEIGHT, BACKGROUND);

    let path = std::env::current_dir()?.join(filename);
    film.save_png(&path)?;
    println!("Image created at: {}", path.display());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CAMERA_HEIGHT: f32 = 1.6;
    let target = Vec3::new(0.0, -0.2, 0.0);

    render_view(
        "output_front.png",
        Vec3::new(0.0, CAMERA_HEIGHT, -5.0),
        target,
    )?;

    render_view(
        "output_right.png",
        Vec3::new(5.0, CAMERA_HEIGHT, 0.0),
        target,
    )?;

    render_view(
        "output_back.png",
        Vec3::new(0.0, CAMERA_HEIGHT, 5.0),
        target,
    )?;

    render_view(
        "output_left.png",
        Vec3::new(-5.0, CAMERA_HEIGHT, 0.0),
        target,
    )?;

    Ok(())
}
