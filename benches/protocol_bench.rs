use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cynda_core::{SensorPayload, contracts::ANOMALY_VECTOR_SIZE};
use cynda_core::transmitter::Transmitter;

fn benchmark_serialization(c: &mut Criterion) {
    let payload = SensorPayload::new(
        42,
        1699470000000,
        1,
        75,
        5000,
        0xdeadbeef,
        [0.5; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    c.bench_function("serialize_single_payload", |b| {
        b.iter(|| {
            Transmitter::serialize_payload(black_box(&payload))
        });
    });
}

fn benchmark_batch_serialization(c: &mut Criterion) {
    let payloads = vec![
        SensorPayload::new(1, 1000, 1, 50, 1000, 0x11111111, [0.1; ANOMALY_VECTOR_SIZE]).unwrap(),
        SensorPayload::new(2, 2000, 1, 60, 1000, 0x22222222, [0.2; ANOMALY_VECTOR_SIZE]).unwrap(),
        SensorPayload::new(3, 3000, 1, 70, 1000, 0x33333333, [0.3; ANOMALY_VECTOR_SIZE]).unwrap(),
        SensorPayload::new(4, 4000, 1, 80, 1000, 0x44444444, [0.4; ANOMALY_VECTOR_SIZE]).unwrap(),
        SensorPayload::new(5, 5000, 1, 90, 1000, 0x55555555, [0.5; ANOMALY_VECTOR_SIZE]).unwrap(),
    ];
    
    c.bench_function("serialize_batch_5_payloads", |b| {
        b.iter(|| {
            Transmitter::serialize_batch(black_box(&payloads))
        });
    });
}

fn benchmark_ack_backoff(c: &mut Criterion) {
    use cynda_core::ack_manager::AckManager;
    
    c.bench_function("calculate_exponential_backoff", |b| {
        b.iter(|| {
            for attempt in 0..10 {
                AckManager::calculate_backoff_ms(attempt, black_box(100), black_box(10000));
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_serialization,
    benchmark_batch_serialization,
    benchmark_ack_backoff
);
criterion_main!(benches);
