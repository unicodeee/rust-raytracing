use glam::Vec3;
use rust_learn::camera::Camera;
use rust_learn::film::Film;
use rust_learn::render::render;
use rust_learn::scene::Scene;

fn render_view(
    scene: &Scene,
    filename: &str,
    eye: Vec3,
    target: Vec3,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::new(eye, target, Vec3::Y, width as f32 / height as f32, 45.0);
    let film: Film = render(scene, &camera, width, height);
    let path = std::env::current_dir()?.join(filename);
    film.save_png(&path)?;
    println!("Image created at: {}", path.display());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "vase".to_owned());
    match mode.as_str() {
        "vase" => render_vase_views()?,
        "pyramid" => render_view(
            &Scene::pyramid([187, 235, 70, 255]),
            "docs/renders/pyramid_hard.png",
            Vec3::new(3.2, 2.1, -5.5),
            Vec3::new(0.0, 0.0, 0.0),
            640,
            480,
        )?,
        "pyramid-smooth" => render_view(
            &Scene::smooth_pyramid([187, 235, 70, 255]),
            "docs/renders/pyramid_smooth.png",
            Vec3::new(3.2, 2.1, -5.5),
            Vec3::new(0.0, 0.0, 0.0),
            640,
            480,
        )?,
        "capuchino" => render_view(
            &Scene::capuchino_scene(),
            "docs/renders/capuchino_obj.png",
            Vec3::new(2.4, 1.8, -3.6),
            Vec3::ZERO,
            192,
            192,
        )?,
        _ => {
            return Err(format!(
                "unknown scene `{mode}`; use vase, pyramid, pyramid-smooth, or capuchino"
            )
            .into());
        }
    }
    Ok(())
}

fn render_vase_views() -> Result<(), Box<dyn std::error::Error>> {
    let scene = Scene::vase_scene();
    let target = Vec3::new(0.0, 0.0, 0.0);
    for (filename, eye) in [
        ("docs/renders/vase_front.png", Vec3::new(3.8, 2.2, -6.2)),
        ("docs/renders/vase_left.png", Vec3::new(-5.5, 1.8, -3.2)),
        ("docs/renders/vase_right.png", Vec3::new(5.5, 1.8, -2.4)),
    ] {
        render_view(&scene, filename, eye, target, 640, 480)?;
    }
    Ok(())
}
