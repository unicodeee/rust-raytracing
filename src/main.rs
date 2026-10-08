use glam::Vec3;
use rust_learn::camera::Camera;
use rust_learn::film::Film;
use rust_learn::render::render;
use rust_learn::scene::Scene;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

fn render_view(
    scene: &Scene,
    filename: &str,
    eye: Vec3,
    target: Vec3,
) -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::new(eye, target, Vec3::Y, WIDTH as f32 / HEIGHT as f32, 45.0);
    let film: Film = render(scene, &camera, WIDTH, HEIGHT);
    let path = std::env::current_dir()?.join(filename);
    film.save_png(&path)?;
    println!("Image created at: {}", path.display());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CAMERA_HEIGHT: f32 = 1.6;
    let scene = Scene::init();
    let target = Vec3::new(0.0, -0.2, 0.0);
    render_view(
        &scene,
        "output_front.png",
        Vec3::new(0.0, CAMERA_HEIGHT, -5.0),
        target,
    )?;
    render_view(
        &scene,
        "output_right.png",
        Vec3::new(5.0, CAMERA_HEIGHT, 0.0),
        target,
    )?;
    render_view(
        &scene,
        "output_back.png",
        Vec3::new(0.0, CAMERA_HEIGHT, 5.0),
        target,
    )?;
    render_view(
        &scene,
        "output_left.png",
        Vec3::new(-5.0, CAMERA_HEIGHT, 0.0),
        target,
    )?;
    Ok(())
}
