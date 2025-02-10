use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use modav_core::repr::{builders::SheetBuilder, col_sheet::ColumnSheet, utils::*, Sheet};
use std::time::Duration;

pub fn init_low1(c: &mut Criterion) {
    let builder = SheetBuilder::new("./benches/samples/low1.csv")
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Infer);

    let mut group = c.benchmark_group("Lightweight workload 1");

    group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
        bench.iter(|| Sheet::from_builder(builder.clone()));
    });

    group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
        bench.iter(|| ColumnSheet::from_builder(builder.clone()));
    });
}

pub fn init_low2(c: &mut Criterion) {
    let builder = SheetBuilder::new("./benches/samples/low2.csv")
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Infer);

    let mut group = c.benchmark_group("Lightweight workload 2");

    group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
        bench.iter(|| Sheet::from_builder(builder.clone()));
    });

    group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
        bench.iter(|| ColumnSheet::from_builder(builder.clone()));
    });
}

pub fn init_mid1(c: &mut Criterion) {
    let builder = SheetBuilder::new("./benches/samples/mid1.csv")
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Infer);

    let mut group = c.benchmark_group("Medium workload 1");

    group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
        bench.iter(|| Sheet::from_builder(builder.clone()));
    });

    group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
        bench.iter(|| ColumnSheet::from_builder(builder.clone()));
    });
}

pub fn init_mid2(c: &mut Criterion) {
    let builder = SheetBuilder::new("./benches/samples/mid2.csv")
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Infer);

    let mut group = c.benchmark_group("Medium workload 2");

    group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
        bench.iter(|| Sheet::from_builder(builder.clone()));
    });

    group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
        bench.iter(|| ColumnSheet::from_builder(builder.clone()));
    });
}

pub fn init_high1(c: &mut Criterion) {
    let builder = SheetBuilder::new("./benches/samples/high1.csv")
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Infer);

    let mut group = c.benchmark_group("Heavy workload 1");

    group.measurement_time(Duration::from_secs(50));

    group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
        bench.iter(|| Sheet::from_builder(builder.clone()));
    });

    group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
        bench.iter(|| ColumnSheet::from_builder(builder.clone()));
    });
}

pub fn init_high2(c: &mut Criterion) {
    let builder = SheetBuilder::new("./benches/samples/high2.csv")
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Infer);

    let mut group = c.benchmark_group("Heavy workload 2");

    group.measurement_time(Duration::from_secs(50));

    group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
        bench.iter(|| Sheet::from_builder(builder.clone()));
    });

    group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
        bench.iter(|| ColumnSheet::from_builder(builder.clone()));
    });
}

criterion_group!(
    initialization,
    init_low1,
    init_low2,
    init_mid1,
    init_mid2,
    init_high1,
    init_high2
);
criterion_main!(initialization);
