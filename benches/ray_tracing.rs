use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use render3d::ray_tracing::{Light, RayTracing, Sphere, action::Action, vector::Vec3};

fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(10))
}

fn bench_targets(c: &mut Criterion) {
    c.bench_function("RayTracing::render", |b| {
        b.iter_custom(|iters| {
            let mut renderer = RayTracing::new(512, 512, 42);
            renderer.move_camera_to(Vec3::new(0., 0., 3.));
            renderer.rotate_camera_to(Vec3::new(1., 0., -0.5));
            renderer.put_light(Light::new(Vec3::new(0., 5., 2.), 1.));
            renderer.put_light(Light::new(Vec3::new(0., -5., 2.), -1.));
            for i in 0..3 {
                renderer.put_sphere(Sphere::new(Vec3::new(i as f32 * 3., 0., 2.), 1.));
            }

            let start_time = Instant::now();
            for _ in 0..iters {
                renderer.trigger_action(Action::RequestRender);
                renderer.render();
            }
            start_time.elapsed()
        })
    });
}

criterion_group!(
    name = benches;
    config = custom_criterion();
    targets = bench_targets
);
criterion_main!(benches);
