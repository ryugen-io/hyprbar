use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ks_lib::config::WindowConfig;

fn bench_calculate_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");

    group.bench_function("smart_scaling", |b| {
        let mut config = WindowConfig::default();
        config.height = 30;
        config.scale_font = true;
        config.pixel_font = true;

        b.iter(|| black_box(&config).calculate_dimensions())
    });

    group.finish();
}

criterion_group!(benches, bench_calculate_dimensions);
criterion_main!(benches);
