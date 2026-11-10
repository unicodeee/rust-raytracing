use glam::Vec3;
use rust_learn::camera::Camera;
use rust_learn::film::Film;
use rust_learn::render::render;
use rust_learn::scene::Scene;
use std::time::{Duration, Instant};

fn render_view(
    scene: &Scene,
    filename: &str,
    eye: Vec3,
    target: Vec3,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::new(eye, target, Vec3::Y, width as f32 / height as f32, 45.0);
    let started = Instant::now();
    let film: Film = render(scene, &camera, width, height);
    let render_time = started.elapsed();
    let path = std::env::current_dir()?.join(filename);
    film.save_png(&path)?;
    println!(
        "Image created at: {} (render time: {})",
        path.display(),
        format_duration(render_time)
    );
    Ok(())
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = duration.as_secs_f64() - (hours * 3_600 + minutes * 60) as f64;
    format!("{hours:02}h {minutes:02}m {seconds:06.3}s")
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
        "suzanne" => render_suzanne_views()?,
        _ => {
            return Err(format!(
                "unknown scene `{mode}`; use vase, pyramid, pyramid-smooth, capuchino, or suzanne"
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

fn render_suzanne_views() -> Result<(), Box<dyn std::error::Error>> {
    let scene = Scene::suzanne_scene();
    let target = Vec3::new(0.0, 0.65, 0.0);
    for (filename, eye) in [
        ("docs/renders/suzanne_obj.png", Vec3::new(0.3, 1.8, -6.4)),
        ("docs/renders/suzanne_left.png", Vec3::new(-4.5, 1.8, -5.0)),
        ("docs/renders/suzanne_right.png", Vec3::new(4.5, 1.8, -5.0)),
    ] {
        render_view(&scene, filename, eye, target, 1280, 960)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_is_formatted_as_hours_minutes_and_seconds() {
        assert_eq!(
            format_duration(Duration::from_millis(3_723_450)),
            "01h 02m 03.450s"
        );
    }
}
