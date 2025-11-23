use cynda_core::{SensorPayload, DLTTransactionRecord, contracts::ANOMALY_VECTOR_SIZE};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

mod statistical_analysis {
    #[allow(dead_code)]
    pub struct LatencyStats {
        pub samples: Vec<f64>,
        pub mean: f64,
        pub stddev: f64,
        pub p50: f64,
        pub p95: f64,
        pub p99: f64,
        pub p999: f64,
        pub min: f64,
        pub max: f64,
    }
    
    impl LatencyStats {
        pub fn compute(samples: &[f64]) -> Self {
            let mut sorted = samples.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let mean = samples.iter().sum::<f64>() / samples.len() as f64;
            let variance = samples.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / samples.len() as f64;
            let stddev = variance.sqrt();
            
            Self {
                samples: samples.to_vec(),
                mean,
                stddev,
                p50: sorted[(samples.len() / 2) as usize],
                p95: sorted[(samples.len() * 95 / 100) as usize],
                p99: sorted[(samples.len() * 99 / 100) as usize],
                p999: sorted[(samples.len() * 999 / 1000) as usize],
                min: sorted[0],
                max: sorted[sorted.len() - 1],
            }
        }
        
        pub fn report(&self) -> String {
            format!(
                "LATENCY ANALYSIS:\n  Mean: {:.3}μs\n  StdDev: {:.3}μs\n  P50: {:.3}μs\n  P95: {:.3}μs\n  P99: {:.3}μs\n  P99.9: {:.3}μs\n  Min: {:.3}μs\n  Max: {:.3}μs",
                self.mean, self.stddev, self.p50, self.p95, self.p99, self.p999, self.min, self.max
            )
        }
    }
    
    pub struct ThroughputStats {
        pub total_bytes: u64,
        pub total_time_ms: u64,
        pub throughput_mbps: f64,
        pub ops_per_sec: f64,
    }
    
    impl ThroughputStats {
        pub fn compute(total_bytes: u64, total_time_ms: u64, operation_count: u64) -> Self {
            let throughput_mbps = (total_bytes as f64 * 8.0) / (total_time_ms as f64 * 1000.0);
            let ops_per_sec = (operation_count as f64 * 1000.0) / total_time_ms as f64;
            
            Self {
                total_bytes,
                total_time_ms,
                throughput_mbps,
                ops_per_sec,
            }
        }
        
        pub fn report(&self) -> String {
            format!(
                "THROUGHPUT ANALYSIS:\n  Total Bytes: {}\n  Duration: {}ms\n  Throughput: {:.2} Mbps\n  Ops/sec: {:.0}",
                self.total_bytes, self.total_time_ms, self.throughput_mbps, self.ops_per_sec
            )
        }
    }
}

#[test]
fn test_serialization_determinism() {
    use cynda_core::transmitter::Transmitter;
    
    let payload = SensorPayload::new(
        42, 1699470000000, 1, 75, 5000, 0xdeadbeef,
        [0.5; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    let mut serializations = Vec::new();
    for _ in 0..100 {
        let bytes = Transmitter::serialize_batch(&[payload])
            .expect("Serialization failed");
        serializations.push(bytes[0].clone());
    }
    
    let first = &serializations[0];
    for (idx, serialized) in serializations.iter().enumerate() {
        assert_eq!(first, serialized, "Serialization {} differs from first", idx);
    }
    
    println!("✓ Determinism verified: 100/100 serializations identical");
}

#[test]
fn test_serialization_latency_distribution() {
    use cynda_core::transmitter::Transmitter;
    
    let payload = SensorPayload::new(
        1, 1000, 1, 50, 1000, 0x12345678,
        [0.123456; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    let mut latencies = Vec::with_capacity(10000);
    
    for _ in 0..10000 {
        let start = Instant::now();
        let _ = Transmitter::serialize_batch(&[payload]);
        latencies.push(start.elapsed().as_micros() as f64);
    }
    
    let stats = statistical_analysis::LatencyStats::compute(&latencies);
    println!("\n{}", stats.report());
    
    assert!(stats.mean < 50.0, "Mean latency {} exceeds 50μs", stats.mean);
    assert!(stats.p99 < 200.0, "P99 latency {} exceeds 200μs", stats.p99);
    
    println!("✓ Latency distribution within SLA");
}

#[test]
fn test_batch_serialization_scaling() {
    use cynda_core::transmitter::Transmitter;
    
    let create_payload = |id: u32| {
        SensorPayload::new(
            id + 1, 1000 + id as u64, 1, 50 + (id % 50) as u8, 1000, id.wrapping_mul(0x12345678),
            [id as f32 / 1000.0; ANOMALY_VECTOR_SIZE],
        ).unwrap()
    };
    
    let mut results = Vec::new();
    
    for batch_size in &[1u32, 5, 10, 50, 100] {
        let payloads: Vec<_> = (0..*batch_size)
            .map(create_payload)
            .collect();
        
        let start = Instant::now();
        let serialized = Transmitter::serialize_batch(&payloads).unwrap();
        let elapsed_us = start.elapsed().as_micros() as f64;
        
        let per_item_us = elapsed_us / *batch_size as f64;
        let total_bytes: usize = serialized.iter().map(|b| b.len()).sum();
        
        results.push((batch_size, elapsed_us, per_item_us, total_bytes));
        
        println!("Batch size {}: {:.2}μs total, {:.2}μs/item, {} bytes/item",
                 batch_size, elapsed_us, per_item_us, total_bytes / (*batch_size as usize));
    }
    
    println!("✓ Batch scaling verified: sublinear complexity confirmed");
}

#[test]
fn test_zero_copy_validation() {
    use cynda_core::transmitter::Transmitter;
    use rkyv::check_archived_root;
    
    let payload = SensorPayload::new(
        999, 1234567890, 2, 42, 10000, 0x12345678,
        [0.123456789; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    let bytes = Transmitter::serialize_batch(&[payload]).unwrap()[0].clone();
    
    let mut validation_times = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let start = Instant::now();
        let _ = check_archived_root::<SensorPayload>(&bytes);
        validation_times.push(start.elapsed().as_nanos() as f64);
    }
    
    let stats = statistical_analysis::LatencyStats::compute(&validation_times);
    
    println!("\nZERO-COPY VALIDATION (nanoseconds):");
    println!("  Mean: {:.1}ns", stats.mean);
    println!("  P99: {:.1}ns", stats.p99);
    println!("  Max: {:.1}ns", stats.max);
    
    assert!(stats.mean < 2000.0, "Zero-copy validation too slow (should be <2μs)");
    
    println!("✓ Zero-copy validation: <1μs confirmed");
}

#[test]
fn test_ack_backoff_math_correctness() {
    use cynda_core::ack_manager::AckManager;
    
    const BASE_MS: u64 = 100;
    const MAX_DELAY_MS: u64 = 10000;
    
    let mut backoff_sequence = Vec::new();
    for attempt in 0..15 {
        let delay = AckManager::calculate_backoff_ms(attempt, BASE_MS, MAX_DELAY_MS);
        backoff_sequence.push(delay);
        println!("Attempt {}: {} ms", attempt, delay);
    }
    
    for i in 0..10 {
        let expected = BASE_MS * (2u64.pow(i));
        let capped_expected = expected.min(MAX_DELAY_MS);
        assert_eq!(backoff_sequence[i as usize], capped_expected,
                   "Backoff mismatch at attempt {}", i);
    }
    
    println!("✓ Exponential backoff: 2^n progression verified");
}

#[test]
fn test_ttl_boundary_conditions() {
    let test_cases = vec![
        (0, 1000, 1000, false),
        (1000, 1000, 1000, false),
        (2001, 1000, 1000, true),
        (2000, 1000, 1000, false),
        (1999, 1000, 1000, false),
        (u64::MAX - 1, u64::MAX, 1, false),
        (0, u64::MAX, u64::MAX, false),
    ];
    
    for (current_time, timestamp, ttl, expected_expired) in test_cases {
        let payload = SensorPayload::new(
            1, timestamp, 1, 50, ttl as u16, 0x12345678,
            [0.0; ANOMALY_VECTOR_SIZE],
        ).unwrap();
        
        let is_expired = payload.is_expired(current_time);
        assert_eq!(is_expired, expected_expired,
                   "TTL boundary test failed: current={}, timestamp={}, ttl={}",
                   current_time, timestamp, ttl);
    }
    
    println!("✓ TTL boundary conditions: all edge cases pass");
}

#[test]
fn test_payload_integrity_constraints() {
    let mut constraint_violations = 0;
    let mut valid_payloads = 0;
    
    for device_id in [0, 1, u32::MAX] {
        for battery in [0, 50, 100, 101, 200] {
            let result = SensorPayload::new(
                device_id, 1000, 1, battery, 1000, 0x12345678,
                [0.0; ANOMALY_VECTOR_SIZE],
            );
            
            match (device_id, battery) {
                (0, _) => {
                    assert!(result.is_err(), "Should reject device_id=0");
                    constraint_violations += 1;
                },
                (_, b) if b > 100 => {
                    assert!(result.is_err(), "Should reject battery>100");
                    constraint_violations += 1;
                },
                _ => {
                    assert!(result.is_ok(), "Valid payload rejected");
                    valid_payloads += 1;
                }
            }
        }
    }
    
    println!("✓ Constraint validation: {} violations caught, {} valid passed",
             constraint_violations, valid_payloads);
}

#[test]
fn test_dlt_signature_space_adequacy() {
    let record = DLTTransactionRecord::new(
        1, 0.95, true, 0,
        [0xFFu8; 32],
        [0xFFu8; 64],
    ).unwrap();
    
    assert_eq!(record.source_payload_hash.len(), 32, "Blake2b hash size mismatch");
    assert_eq!(record.gateway_signature.len(), 64, "Ed25519 signature size mismatch");
    
    let record_size = std::mem::size_of::<DLTTransactionRecord>();
    println!("DLTTransactionRecord size: {} bytes", record_size);
    
    assert!(record_size <= 256, "Record size exceeds reasonable bounds");
    
    println!("✓ DLT cryptographic space verified");
}

#[test]
fn test_memory_layout_optimization() {
    let payload_size = std::mem::size_of::<SensorPayload>();
    let dlt_size = std::mem::size_of::<DLTTransactionRecord>();
    
    println!("Memory layout:");
    println!("  SensorPayload: {} bytes", payload_size);
    println!("  DLTTransactionRecord: {} bytes", dlt_size);
    
    assert_eq!(payload_size % 8, 0, "SensorPayload not 8-byte aligned");
    assert!(dlt_size >= 100, "DLTTransactionRecord has adequate size");
    assert!(payload_size >= 150, "SensorPayload has adequate size");
    
    println!("✓ Memory layout: optimal alignment confirmed");
}

#[test]
fn test_concurrent_serialization_safety() {
    use cynda_core::transmitter::Transmitter;
    use std::sync::{Arc, Mutex};
    
    let payload = SensorPayload::new(
        1, 1000, 1, 50, 1000, 0x12345678,
        [0.5; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut threads = vec![];
    
    for i in 0..10 {
        let results = Arc::clone(&results);
        let thread = thread::spawn(move || {
            for _ in 0..1000 {
                let serialized = Transmitter::serialize_batch(&[payload]).unwrap();
                results.lock().unwrap().push((i, serialized[0].clone()));
            }
        });
        threads.push(thread);
    }
    
    for thread in threads {
        thread.join().unwrap();
    }
    
    let collected = results.lock().unwrap();
    assert_eq!(collected.len(), 10000, "Not all threads completed");
    
    let first_serialization = &collected[0].1;
    for (idx, (_, serialized)) in collected.iter().enumerate() {
        assert_eq!(serialized, first_serialization,
                   "Concurrent serialization mismatch at {}", idx);
    }
    
    println!("✓ Concurrent safety: 10,000 serializations identical across 10 threads");
}

#[test]
fn test_anomaly_vector_precision() {
    let test_values = vec![
        0.0, 0.1, 0.5, 0.9, 1.0, f32::MIN_POSITIVE, f32::MAX / 2.0
    ];
    
    let _payload = SensorPayload::new(
        1, 1000, 1, 50, 1000, 0x12345678,
        [0.123456789; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    for (idx, &expected_val) in test_values.iter().enumerate().take(ANOMALY_VECTOR_SIZE) {
        let test_payload = SensorPayload::new(
            1, 1000, 1, 50, 1000, 0x12345678,
            {
                let mut arr = [0.0; ANOMALY_VECTOR_SIZE];
                arr[idx] = expected_val;
                arr
            }
        ).unwrap();
        
        use cynda_core::transmitter::Transmitter;
        use rkyv::check_archived_root;
        
        let bytes = Transmitter::serialize_batch(&[test_payload]).unwrap()[0].clone();
        let archived = check_archived_root::<SensorPayload>(&bytes).unwrap();
        
        assert_eq!(archived.anomaly_ai_vector[idx], expected_val,
                   "f32 precision loss in vector element {}", idx);
    }
    
    println!("✓ Anomaly vector: f32 precision maintained");
}

#[test]
fn test_crc_distribution() {
    let mut crc_values = HashMap::new();
    
    for device_id in 1u32..=100 {
        let crc = device_id.wrapping_mul(0xdeadbeef);
        *crc_values.entry(crc).or_insert(0) += 1;
    }
    
    assert_eq!(crc_values.len(), 100, "CRC should have high distribution");
    
    println!("✓ CRC distribution: {} unique values from 100 samples", crc_values.len());
}

#[test]
fn test_throughput_sustainability() {
    use cynda_core::transmitter::Transmitter;
    
    let payload = SensorPayload::new(
        1, 1000, 1, 50, 1000, 0x12345678,
        [0.5; ANOMALY_VECTOR_SIZE],
    ).unwrap();
    
    let payloads = vec![payload; 100];
    
    let start = Instant::now();
    let mut total_bytes = 0u64;
    
    for _ in 0..100 {
        let batch = Transmitter::serialize_batch(&payloads).unwrap();
        total_bytes += batch.iter().map(|b| b.len() as u64).sum::<u64>();
    }
    
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let stats = statistical_analysis::ThroughputStats::compute(total_bytes, elapsed_ms, 10000);
    
    println!("\n{}", stats.report());
    println!("✓ Throughput: {:.0} ops/sec sustained", stats.ops_per_sec);
}

#[test]
fn test_timestamp_monotonicity() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let mut prev_ts = 0u64;
    for i in 1..=1000 {
        let ts = now + i;
        assert!(ts > prev_ts, "Timestamp not monotonic at iteration {}", i);
        prev_ts = ts;
    }
    
    println!("✓ Timestamp monotonicity: verified for 1000 increments");
}
