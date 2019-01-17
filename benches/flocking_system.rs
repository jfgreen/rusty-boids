#[macro_use]
extern crate criterion;
extern crate aproxiflock;

use aproxiflock::system::{FlockingConfig, FlockingSystem};
use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let config = FlockingConfig {
        width: 1000.,
        height: 800.,
        boid_count: 80000,
        max_speed: 2.5,
        max_force: 0.4,
        mouse_weight: 600.,
        sep_radius: 6.,
        ali_radius: 11.5,
        coh_radius: 11.5,
        sep_weight: 1.5,
        ali_weight: 1.0,
        coh_weight: 1.0,
    };

    let mut flock = FlockingSystem::new(config);
    flock.randomise();

    c.bench_function("flock update", move |b| {
        b.iter(|| flock.update());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
