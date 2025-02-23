use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use modav_core::repr::{
    col_sheet::ColumnSheet, utils::TypesStrategy, Config, HeaderStrategy, Sheet,
};
use std::time::Duration;

mod init {
    use super::*;
    pub use fields::*;
    pub use records::*;
    pub use types::*;

    const DURATION: u64 = 300;

    /// Lightweight CSV meausrement 1
    /// Record num: 6
    /// Field num: 6
    /// Type distribution: 5 String, 1 i32
    pub fn low1(c: &mut Criterion) {
        let builder = Config::new("./benches/samples/low1.csv")
            .labels(HeaderStrategy::ReadLabels)
            .types(TypesStrategy::Infer);

        let mut group = c.benchmark_group("Lightweight workload 1");

        group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
            bench.iter(|| Sheet::with_config(builder.clone()));
        });

        group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
            bench.iter(|| ColumnSheet::with_config(builder.clone()));
        });
    }

    /// Lightweight CSV meausrement 2
    /// Record num: 31
    /// Field num: 4
    /// Type distribution: 2 i32, 2 f32
    pub fn low2(c: &mut Criterion) {
        let builder = Config::new("./benches/samples/low2.csv")
            .labels(HeaderStrategy::ReadLabels)
            .types(TypesStrategy::Infer);

        let mut group = c.benchmark_group("Lightweight workload 2");

        group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
            bench.iter(|| Sheet::with_config(builder.clone()));
        });

        group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
            bench.iter(|| ColumnSheet::with_config(builder.clone()));
        });
    }

    /// Medium weight CSV meausrement 1
    /// Record num: 128
    /// Field num: 9
    /// Type distribution: 4 String, 5 i32
    pub fn mid1(c: &mut Criterion) {
        let builder = Config::new("./benches/samples/mid1.csv")
            .labels(HeaderStrategy::ReadLabels)
            .types(TypesStrategy::Infer);

        let mut group = c.benchmark_group("Medium workload 1");

        group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
            bench.iter(|| Sheet::with_config(builder.clone()));
        });

        group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
            bench.iter(|| ColumnSheet::with_config(builder.clone()));
        });
    }

    /// Medium weight CSV meausrement 2
    /// Record num: 200
    /// Field num: 3
    /// Type distribution: 1 i32, 2 f32
    pub fn mid2(c: &mut Criterion) {
        let builder = Config::new("./benches/samples/mid2.csv")
            .labels(HeaderStrategy::ReadLabels)
            .types(TypesStrategy::Infer);

        let mut group = c.benchmark_group("Medium workload 2");

        group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
            bench.iter(|| Sheet::with_config(builder.clone()));
        });

        group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
            bench.iter(|| ColumnSheet::with_config(builder.clone()));
        });
    }

    /// Heavy weight CSV meausrement 1
    /// Record num: 114635
    /// Field num: 22
    /// Type distribution: 7 i32, 15 String
    pub fn high1(c: &mut Criterion) {
        let builder = Config::new("./benches/samples/high1.csv")
            .labels(HeaderStrategy::ReadLabels)
            .types(TypesStrategy::Infer);

        let mut group = c.benchmark_group("Heavy workload 1");

        group.measurement_time(Duration::from_secs(DURATION));

        group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
            bench.iter(|| Sheet::with_config(builder.clone()));
        });

        group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
            bench.iter(|| ColumnSheet::with_config(builder.clone()));
        });
    }

    /// Heavy weight CSV meausrement 2
    /// Record num: 254750
    /// Field num: 13
    /// Type distribution: 6 i32, 7 String
    pub fn high2(c: &mut Criterion) {
        let builder = Config::new("./benches/samples/high2.csv")
            .labels(HeaderStrategy::ReadLabels)
            .types(TypesStrategy::Infer);

        let mut group = c.benchmark_group("Heavy workload 2");

        group.measurement_time(Duration::from_secs(DURATION));

        group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
            bench.iter(|| Sheet::with_config(builder.clone()));
        });

        group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
            bench.iter(|| ColumnSheet::with_config(builder.clone()));
        });
    }

    mod fields {
        use super::*;

        /// Low Fields test
        /// Record num: 10,000
        /// Field num: 5
        /// Type distribution: 2 String, 3 i32
        pub fn field_low(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/fields/field_test_low.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Fields Test Low Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }

        /// Mid Fields test
        /// Record num: 10,000
        /// Field num: 10
        /// Type distribution: 4 String, 6 i32
        pub fn field_mid(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/fields/field_test_mid.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Fields Test Mid Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }

        /// High Fields test
        /// Record num: 10,000
        /// Field num: 25
        /// Type distribution: 9 String, 16 i32
        pub fn field_high(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/fields/field_test_high.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Fields Test High Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }

        /// Stress Fields test
        /// Record num: 10,000
        /// Field num: 75
        /// Type distribution: 27 String, 48 i32
        pub fn field_stress(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/fields/field_test_stress.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Fields Test Stress Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }
    }

    mod records {
        use super::*;

        /// Low Records test
        /// Record num: 5,000
        /// Field num: 10
        /// Type distribution: 4 String, 6 i32
        pub fn record_low(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/records/record_test_low.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Record Test Low Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });
        }

        /// Mid Records test
        /// Record num: 50,000
        /// Field num: 10
        /// Type distribution: 4 String, 6 i32
        pub fn record_mid(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/records/record_test_mid.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Record Test Mid Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });
        }

        /// High Records test
        /// Record num: 150,000
        /// Field num: 10
        /// Type distribution: 4 String, 6 i32
        pub fn record_high(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/records/record_test_high.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Record Test High Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });
        }

        /// Stress Records test
        /// Record num: 500,000
        /// Field num: 10
        /// Type distribution: 4 String, 6 i32
        pub fn record_stress(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/records/record_test_stress.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Record Test Stress Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });
        }
    }

    mod types {
        use super::*;

        /// Int Types test
        /// Record num: 10,000
        /// Field num: 10
        /// Type distribution: 10 i32
        pub fn types_int(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/types/type_test_int.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Type Test Int Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }

        /// Text Types test
        /// Record num: 10,000
        /// Field num: 10
        /// Type distribution: 10 String
        pub fn types_text(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/types/type_test_text.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Type Test Text Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }

        /// Mixed Types test
        /// Record num: 10,000
        /// Field num: 10
        /// Type distribution: 5 String, i32
        pub fn types_mixed(c: &mut Criterion) {
            let builder = Config::new("./benches/samples/types/type_test_mixed.csv")
                .labels(HeaderStrategy::NoLabels)
                .types(TypesStrategy::Infer);

            let mut group = c.benchmark_group("Type Test Mixed Workload");

            group.measurement_time(Duration::from_secs(DURATION));

            group.bench_function(BenchmarkId::new("Row Sheet", ""), |bench| {
                bench.iter(|| Sheet::with_config(builder.clone()));
            });

            group.bench_function(BenchmarkId::new("Column Sheet", ""), |bench| {
                bench.iter(|| ColumnSheet::with_config(builder.clone()));
            });
        }
    }
}

criterion_group!(
    initialization,
    init::low1,
    init::low2,
    init::mid1,
    init::mid2,
    init::high1,
    init::high2
);

criterion_group!(
    init_fields,
    init::field_low,
    init::field_mid,
    init::field_high,
    init::field_stress,
);

criterion_group! {
    init_records,
    init::record_low,
    init::record_mid,
    init::record_high,
    init::record_stress,
}

criterion_group! {
    init_types,
    init::types_int,
    init::types_text,
    init::types_mixed
}

//criterion_main!(initialization, init_fields, init_records, init_types);
//criterion_main!(init_fields, init_records, init_types);
criterion_main!(init_records);
