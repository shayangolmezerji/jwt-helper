/// CyDnA Core - Ultra-low latency UDP messaging protocol
/// 
/// **Creator**: Shayan Golmezerji
/// **License**: Creative Commons Attribution 4.0 International (CC BY 4.0)
/// 
/// This library implements the core protocol for time-critical communication
/// between Sensor Layer (S-Layer) and Gateway Layer (G-Layer) hardware.
/// 
/// # Design Principles
/// - Zero-copy serialization using rkyv
/// - Minimal external dependencies (std + tokio only)
/// - Optimized for latency-critical operations
/// - Production-grade security and reliability

pub mod errors;
pub mod contracts;
pub mod transmitter;
pub mod receiver;
pub mod ack_manager;

pub use contracts::{SensorPayload, DLTTransactionRecord};
pub use errors::{CyDnAError, Result};

/// Protocol version
pub const CYNDA_VERSION: u16 = 1;

/// Maximum payload size (in bytes) - must fit in standard UDP MTU
pub const MAX_PAYLOAD_SIZE: usize = 1024;

/// ACK timeout duration in milliseconds
pub const ACK_TIMEOUT_MS: u64 = 100;

/// Maximum retransmission attempts for critical alerts
pub const MAX_RETRANSMIT_ATTEMPTS: u32 = 3;

/// Backoff multiplier for retransmission (exponential)
pub const BACKOFF_MULTIPLIER: u64 = 2;
