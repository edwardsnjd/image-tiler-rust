use criterion::{criterion_group, criterion_main, Criterion};
use tiler::analysis::ColorInfo;

pub fn color_diff_benchmarks(c: &mut Criterion) {
    let c1 = ColorInfo::new(10, 20, 30);
    let c2 = ColorInfo::new(100, 200, 100);
    c.bench_function("abs_diff", |b| b.iter(|| c1.abs_diff(&c2)));
    c.bench_function("sqr_diff", |b| b.iter(|| c1.sqr_diff(&c2)));
}

criterion_group!(benches, color_diff_benchmarks);
criterion_main!(benches);
