/// Custom error types for CyDnA protocol operations
/// 
/// Provides comprehensive error handling with minimal overhead,
/// designed for production reliability and debugging clarity.
use std::fmt;
use std::io;

/// Result type alias for CyDnA operations
pub type Result<T> = std::result::Result<T, CyDnAError>;

/// CyDnA protocol error enumeration
/// 
/// Covers all failure modes in the communication pipeline:
/// - Network I/O failures
/// - Serialization/deserialization issues
/// - Data integrity violations
/// - Protocol-level violations
#[derive(Debug, Clone)]
pub enum CyDnAError {
    /// I/O error (network send/receive failure)
    IoError(String),
    
    /// Serialization failed - payload too large or invalid structure
    SerializationError(String),
    
    /// Deserialization failed - corrupted packet or incompatible format
    DeserializationError(String),
    
    /// Integrity check failed - CRC mismatch or hash validation failure
    IntegrityCheckFailed { expected: u32, actual: u32 },
    
    /// Packet expired - TTL exceeded
    PayloadExpired { timestamp_ms: u64, ttl_ms: u16 },
    
    /// Invalid device ID - zero or out of valid range
    InvalidDeviceId(u32),
    
    /// Invalid battery level - out of 0-100 range
    InvalidBatteryLevel(u8),
    
    /// ACK timeout - no response from receiver
    AckTimeout,
    
    /// Maximum retransmission attempts exceeded
    MaxRetriesExceeded,
    
    /// Invalid packet length received
    InvalidPacketLength { expected: usize, received: usize },
    
    /// Cryptographic signature verification failed
    SignatureVerificationFailed,
    
    /// Invalid gateway ID
    InvalidGatewayId(u32),
    
    /// Buffer too small for operation
    BufferTooSmall { required: usize, available: usize },
}

impl fmt::Display for CyDnAError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            Self::IntegrityCheckFailed { expected, actual } => {
                write!(f, "Integrity check failed: expected CRC32 {:#x}, got {:#x}", expected, actual)
            }
            Self::PayloadExpired { timestamp_ms, ttl_ms } => {
                write!(f, "Payload expired: timestamp {} + TTL {} ms", timestamp_ms, ttl_ms)
            }
            Self::InvalidDeviceId(id) => write!(f, "Invalid device ID: {}", id),
            Self::InvalidBatteryLevel(level) => write!(f, "Invalid battery level: {}", level),
            Self::AckTimeout => write!(f, "ACK timeout - no response from receiver"),
            Self::MaxRetriesExceeded => write!(f, "Maximum retransmission attempts exceeded"),
            Self::InvalidPacketLength { expected, received } => {
                write!(f, "Invalid packet length: expected {}, received {}", expected, received)
            }
            Self::SignatureVerificationFailed => write!(f, "Cryptographic signature verification failed"),
            Self::InvalidGatewayId(id) => write!(f, "Invalid gateway ID: {}", id),
            Self::BufferTooSmall { required, available } => {
                write!(f, "Buffer too small: required {}, available {}", required, available)
            }
        }
    }
}

impl std::error::Error for CyDnAError {}

/// Convert from io::Error to CyDnAError
impl From<io::Error> for CyDnAError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}
