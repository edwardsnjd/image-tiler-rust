use criterion::{criterion_group, criterion_main, Criterion};
use image::RgbaImage;
use tiler::analysis::{analyse, AnalysisOptions};

pub fn analyse_benchmarks(c: &mut Criterion) {
    let img = RgbaImage::new(100, 100);
    let opts = AnalysisOptions::new(Some(1));
    c.bench_function("analyse_100x100", |b| b.iter(|| analyse(&img, &opts)));
}

criterion_group!(benches, analyse_benchmarks);
criterion_main!(benches);
