use std::time::Duration;

use minifb::{Key, Window, WindowOptions};
use rand::SeedableRng;
use render3d::ray_tracing::{
    Light, RayTracing, Sphere, action::Action, fps::FpsCounter, vector::Vec3,
};

fn key_map(key: Key) -> Option<Action> {
    match key {
        Key::J => Some(Action::CameraRotationDown),
        Key::K => Some(Action::CameraRotationUp),
        Key::H => Some(Action::CameraRotationCCW),
        Key::L => Some(Action::CameraRotationCW),
        Key::W => Some(Action::CameraMoveForward),
        Key::S => Some(Action::CameraMoveBackward),
        Key::A => Some(Action::CameraMoveLeft),
        Key::D => Some(Action::CameraMoveRight),
        Key::Space => Some(Action::CameraMoveUp),
        Key::LeftCtrl => Some(Action::CameraMoveDown),
        _ => None,
    }
}

fn main() {
    const HEIGHT: usize = 300;
    const WIDTH: usize = 300;
    let title = "Render 3D - Ray Tracing";
    let mut window = Window::new(
        title,
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: true,
            title: true,
            resize: false,
            ..Default::default()
        },
    )
    .unwrap();
    window.set_target_fps(30); // 60 帧会消耗很多的 cpu, 需要使用 release profile 才有较好的帧率.

    let mut fps_counter = FpsCounter::new(Duration::from_secs(1));
    let mut renderer = RayTracing::new(HEIGHT, WIDTH, rand::rngs::SmallRng::seed_from_u64(42));
    for x in (0..10u8).step_by(3) {
        renderer.put_sphere(Sphere::new(Vec3::new(x.into(), 0., 1.), 1.));
    }
    renderer.put_light(Light::new(Vec3::new(5., 5., 3.), 1.));
    renderer.put_light(Light::new(Vec3::new(5., -5., 3.), 1.0));
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.set_title(&format!("{title}, fps: {:.2}", fps_counter.tick()));
        window
            .get_keys()
            .into_iter()
            .filter_map(key_map)
            .for_each(|a| renderer.press_key(a));
        let buffer = renderer.render();
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
