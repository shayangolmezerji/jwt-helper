use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, CyDnAError>;

#[derive(Debug, Clone)]
pub enum CyDnAError {
    IoError(String),
    
    SerializationError(String),
    
    DeserializationError(String),
    
    IntegrityCheckFailed { expected: u32, actual: u32 },
    
    PayloadExpired { timestamp_ms: u64, ttl_ms: u16 },
    
    InvalidDeviceId(u32),
    
    InvalidBatteryLevel(u8),
    
    AckTimeout,
    
    MaxRetriesExceeded,
    
    InvalidPacketLength { expected: usize, received: usize },
    
    SignatureVerificationFailed,
    
    InvalidGatewayId(u32),
    
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

impl From<io::Error> for CyDnAError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}
