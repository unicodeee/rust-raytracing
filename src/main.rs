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
    // Randomly selected lime green: #BBEB46.
    let scene = Scene::pyramid([187, 235, 70, 255]);
    let target = Vec3::new(0.0, 0.0, 0.0);
    render_view(&scene, "pyramid.png", Vec3::new(3.2, 2.1, -5.5), target)?;
    Ok(())
}
