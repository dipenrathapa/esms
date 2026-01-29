use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use esms_backend::business::{calculate_stress_index, stress_level};
use esms_backend::models::SensorData;

fn bench_stress_index_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_index");

    // Benchmark with typical values
    let typical_data = SensorData {
        temperature: 25.0,
        humidity: 50.0,
        noise: 70.0,
        heart_rate: 75.0,
        motion: false,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    group.bench_function("typical_values", |b| {
        b.iter(|| calculate_stress_index(black_box(&typical_data)));
    });

    // Benchmark with minimum values
    let min_data = SensorData {
        temperature: 0.0,
        humidity: 0.0,
        noise: 0.0,
        heart_rate: 30.0,
        motion: false,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    group.bench_function("minimum_values", |b| {
        b.iter(|| calculate_stress_index(black_box(&min_data)));
    });

    // Benchmark with maximum values
    let max_data = SensorData {
        temperature: 60.0,
        humidity: 100.0,
        noise: 120.0,
        heart_rate: 200.0,
        motion: true,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    group.bench_function("maximum_values", |b| {
        b.iter(|| calculate_stress_index(black_box(&max_data)));
    });

    group.finish();
}

fn bench_stress_level_classification(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_level");

    group.bench_with_input(BenchmarkId::new("classify", "low"), &0.2, |b, &score| {
        b.iter(|| stress_level(black_box(score)));
    });

    group.bench_with_input(BenchmarkId::new("classify", "moderate"), &0.5, |b, &score| {
        b.iter(|| stress_level(black_box(score)));
    });

    group.bench_with_input(BenchmarkId::new("classify", "high"), &0.8, |b, &score| {
        b.iter(|| stress_level(black_box(score)));
    });

    group.finish();
}

fn bench_combined_workflow(c: &mut Criterion) {
    c.bench_function("full_stress_analysis", |b| {
        let data = SensorData {
            temperature: 30.0,
            humidity: 60.0,
            noise: 75.0,
            heart_rate: 90.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        b.iter(|| {
            let index = calculate_stress_index(black_box(&data));
            stress_level(black_box(index));
        });
    });
}

criterion_group!(
    benches,
    bench_stress_index_calculation,
    bench_stress_level_classification,
    bench_combined_workflow
);
criterion_main!(benches);
