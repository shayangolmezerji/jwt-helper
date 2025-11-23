# CyDnA Core Protocol

[![CI](https://github.com/shayangolmezerji/CyDnA/workflows/CI/badge.svg)](https://github.com/shayangolmezerji/CyDnA/actions)
[![License: CC-BY-4.0](https://img.shields.io/badge/License-CC--BY--4.0-blue.svg)](https://creativecommons.org/licenses/by/4.0/)

Ultra-low latency UDP protocol for sensor-to-gateway communication. Designed for time-critical systems.

**Creator**: Shayan Golmezerji | **License**: CC BY 4.0

## What This Is

A Rust library for real-time sensor data transmission. Zero-copy deserialization, Ed25519 signatures, exponential backoff ACK/NACK protocol.

## Features

- Zero-copy deserialization (rkyv)
- Ed25519 signatures + Blake2b hashing
- Custom ACK/NACK with exponential backoff
- 24 tests, all passing
- GitHub Actions CI/CD
- Minimal dependencies (tokio + rkyv only)

## Build

```bash
cargo build --release
```

## Test & Benchmark

```bash
cargo test --release
cargo bench
```

## Use

### Send Sensor Data

```rust
use cynda_core::{SensorPayload, transmitter::Transmitter};
use std::net::UdpSocket;

let socket = UdpSocket::bind("0.0.0.0:0")?;
let payload = SensorPayload::new(
    1, 1699470000000, 1, 85, 5000, 0xdeadbeef,
    [0.5; 32]
)?;
Transmitter::send(&socket, &payload, "10.0.0.1:8080")?;
```

### Receive & Validate

```rust
use cynda_core::receiver::Receiver;
use std::time::{SystemTime, UNIX_EPOCH};

let socket = UdpSocket::bind("0.0.0.0:8080")?;
let mut buf = vec![0u8; 1024];
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_millis() as u64;

let (payload, bytes, addr) = Receiver::receive_validated(&socket, &mut buf, now)?;
println!("Device: {}, Battery: {}%", 
    payload.device_unique_id, 
    payload.battery_level_percent);
```

### Critical Alert with Retry

```rust
use cynda_core::ack_manager::AckManager;

AckManager::send_critical_alert(
    &socket,
    &payload,
    "10.0.0.1:8080",
    3,      
    100     
)?;
```

## Data Structures

- **SensorPayload** (212 bytes): Device ID, timestamp, firmware, battery, 32×f32 anomaly vector, CRC32, TTL
- **DLTTransactionRecord** (112 bytes): Gateway ID, anomaly score, Ed25519 signature
- **AckPacket** (16 bytes): Device ID, timestamp, ACK/NACK flag

## Configuration

`src/lib.rs`:
```rust
CYNDA_VERSION = 1
MAX_PAYLOAD_SIZE = 1024
ACK_TIMEOUT_MS = 100
MAX_RETRANSMIT_ATTEMPTS = 3
BACKOFF_MULTIPLIER = 2
ANOMALY_VECTOR_SIZE = 32
```

## Error Types

```rust
pub enum CyDnAError {
    IoError(std::io::ErrorKind),
    SerializationError(String),
    DeserializationError(String),
    IntegrityCheckFailed { device_id: u16, timestamp: u64 },
    PayloadExpired { device_id: u16, timestamp: u64 },
    InvalidDeviceId(u16),
    InvalidBatteryLevel(u8),
    AckTimeout,
    MaxRetriesExceeded,
    InvalidPacketLength { received: usize },
    SignatureVerificationFailed,
    BufferTooSmall,
    InvalidGatewayId(u32),
}
```

## Dependencies

- tokio 1.40 (async runtime)
- rkyv 0.7 (zero-copy serialization)
- blake2 0.10 (hashing)
- ed25519-dalek 2.1 (signatures)
- crc32fast 1.3 (checksums)

## Benchmarks

Run with: `cargo bench`

Criterion benchmarks for:
- Single payload serialization
- Batch (5 payloads) serialization
- Exponential backoff calculation

## Deploy Checklist

- MTU ≥ 1024 bytes
- Socket timeouts 100-200ms
- Monitor dropped packets
- Secure Ed25519 keys
- Test failover scenarios
- Rate limiting configured
- Log aggregation enabled

## Contributing

```bash
cargo fmt
cargo clippy --all-targets
cargo test --release
cargo bench
```

All CI must pass.

## License

CC BY 4.0 - Provide attribution to Shayan Golmezerji

See LICENSE file.

## Support

- Issues: https://github.com/shayangolmezerji/CyDnA/issues

### Crypto Support

> 💰 **Support Me With Crypto!**
> 
> If you like this project and want to help me keep developing, consider sending some crypto support.
> Your contributions mean a lot!
> 
> **BTC:** bc1qv8tgypmq68jw4ela5ev7y8s4zmt8kk7ks96zvg  
> **ETH:** 0xc3D6BD5EA169006B30cfdAB5998A992A5a93191C  
> **TRX:** TFc3kzkXFU8Sk4PDC7iJz64YSXqfTHQMFd  
> **SOL:** C7o5E16gvk3ueKrNcUWvsPdXmE174s93fpZcP1mePFhs  
> **TON:** UQAZaNsOBnZL3O0FvXeYSGaTan1d3fe0RHZshpn_uDrDG3qj  

---