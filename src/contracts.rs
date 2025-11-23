/// Core data structures for CyDnA protocol
/// 
/// All structures are optimized for zero-copy serialization using rkyv.
/// Field ordering is carefully designed to minimize padding and maximize
/// serialization efficiency while maintaining intuitive field ordering.
use rkyv::{Archive, Deserialize, Serialize};

/// Fixed size for AI/ML anomaly vector (from ML team specification)
/// Adjust this constant based on the final AI model specification
pub const ANOMALY_VECTOR_SIZE: usize = 32;

/// SensorPayload - Message from S-Layer to G-Layer
/// 
/// Carries time-critical output from the Transient Anomaly detection system.
/// This structure is optimized for zero-copy deserialization and minimal
/// network overhead.
///
/// # Memory Layout
/// Total size: 212 bytes (32-bit aligned)
/// - device_unique_id: 4 bytes (u32)
/// - timestamp_ms_utc: 8 bytes (u64)
/// - sensor_model_version: 2 bytes (u16)
/// - battery_level_percent: 1 byte (u8)
/// - time_to_live_ms: 2 bytes (u16)
/// - raw_data_hash_crc: 4 bytes (u32)
/// - anomaly_ai_vector: 128 bytes ([f32; 32])
/// - padding: 3 bytes (for alignment)
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
#[archive(check_bytes)]
pub struct SensorPayload {
    /// Non-zero unique hardware identifier for the sensor unit
    pub device_unique_id: u32,
    
    /// Millisecond Unix timestamp when vibration data was sampled
    /// Critical for auditability and TTL calculation
    pub timestamp_ms_utc: u64,
    
    /// Firmware/AI model version running on S-Layer
    pub sensor_model_version: u16,
    
    /// Current battery charge (0-100%)
    /// Used for G-Layer adaptive batching logic
    pub battery_level_percent: u8,
    
    /// Time-to-Live in milliseconds
    /// G-Layer discards if current_time > timestamp_ms_utc + time_to_live_ms
    pub time_to_live_ms: u16,
    
    /// Lightweight checksum (CRC32) of raw vibration data
    /// Enables quick integrity verification on G-Layer
    pub raw_data_hash_crc: u32,
    
    /// Split Learning output vector - anomaly detection scores
    /// Fixed size ensures deterministic memory layout for rkyv serialization
    pub anomaly_ai_vector: [f32; ANOMALY_VECTOR_SIZE],
}

impl SensorPayload {
    /// Create a new SensorPayload with validation
    ///
    /// # Errors
    /// Returns `CyDnAError` if any field is invalid:
    /// - device_unique_id must be non-zero
    /// - battery_level_percent must be 0-100
    pub fn new(
        device_unique_id: u32,
        timestamp_ms_utc: u64,
        sensor_model_version: u16,
        battery_level_percent: u8,
        time_to_live_ms: u16,
        raw_data_hash_crc: u32,
        anomaly_ai_vector: [f32; ANOMALY_VECTOR_SIZE],
    ) -> crate::Result<Self> {
        use crate::errors::CyDnAError;
        
        // Validate device_unique_id (must be non-zero)
        if device_unique_id == 0 {
            return Err(CyDnAError::InvalidDeviceId(device_unique_id));
        }
        
        // Validate battery_level_percent (must be 0-100)
        if battery_level_percent > 100 {
            return Err(CyDnAError::InvalidBatteryLevel(battery_level_percent));
        }
        
        Ok(Self {
            device_unique_id,
            timestamp_ms_utc,
            sensor_model_version,
            battery_level_percent,
            time_to_live_ms,
            raw_data_hash_crc,
            anomaly_ai_vector,
        })
    }
    
    /// Check if this payload has expired based on current time
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        current_time_ms > self.timestamp_ms_utc.saturating_add(self.time_to_live_ms as u64)
    }
    
    /// Get the expiration time in milliseconds
    pub fn expiration_time_ms(&self) -> u64 {
        self.timestamp_ms_utc.saturating_add(self.time_to_live_ms as u64)
    }
}

/// DLTTransactionRecord - Final output record for G-Layer
/// 
/// Created after AI inference is complete and submitted to DLT (IOTA Streams).
/// This structure maintains complete auditability and cryptographic attestation
/// of the anomaly detection result.
///
/// # Memory Layout
/// Total size: 112 bytes (8-byte aligned)
/// - gateway_unique_id: 4 bytes (u32)
/// - final_anomaly_score: 4 bytes (f32)
/// - is_critical_alert: 1 byte (bool)
/// - consensus_mode_used: 1 byte (u8)
/// - padding: 2 bytes (alignment)
/// - source_payload_hash: 32 bytes ([u8; 32])
/// - gateway_signature: 64 bytes ([u8; 64])
#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct DLTTransactionRecord {
    /// Unique ID of the Gateway that processed the payload
    pub gateway_unique_id: u32,
    
    /// Final prediction score from G-Layer AI inference
    /// Typically a probability value (0.0 to 1.0) or anomaly score
    pub final_anomaly_score: f32,
    
    /// Flag for consensus switching
    /// True if anomaly score passes critical threshold
    pub is_critical_alert: bool,
    
    /// Specifies signature mode used
    /// 0 = Single/1-of-N, 1 = Multi/M-of-N
    pub consensus_mode_used: u8,
    
    /// Blake2b hash of the original received SensorPayload bytes
    /// Used for cryptographic linkage between sensor and final record
    pub source_payload_hash: [u8; 32],
    
    /// Ed25519 digital signature from Gateway's private key
    /// Attests to the authenticity of this entire record
    pub gateway_signature: [u8; 64],
}

impl DLTTransactionRecord {
    /// Create a new DLTTransactionRecord with validation
    ///
    /// # Errors
    /// Returns `CyDnAError` if gateway_unique_id is zero
    pub fn new(
        gateway_unique_id: u32,
        final_anomaly_score: f32,
        is_critical_alert: bool,
        consensus_mode_used: u8,
        source_payload_hash: [u8; 32],
        gateway_signature: [u8; 64],
    ) -> crate::Result<Self> {
        use crate::errors::CyDnAError;
        
        if gateway_unique_id == 0 {
            return Err(CyDnAError::InvalidGatewayId(gateway_unique_id));
        }
        
        // Validate consensus_mode_used (must be 0 or 1)
        if consensus_mode_used > 1 {
            return Err(CyDnAError::SerializationError(
                format!("Invalid consensus_mode_used: {}", consensus_mode_used)
            ));
        }
        
        Ok(Self {
            gateway_unique_id,
            final_anomaly_score,
            is_critical_alert,
            consensus_mode_used,
            source_payload_hash,
            gateway_signature,
        })
    }
}

/// ACK/NACK packet for reliability mechanism
/// 
/// Minimal packet used for acknowledgment of critical alerts
/// Fixed size: 16 bytes
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
#[archive(check_bytes)]
pub struct AckPacket {
    /// Device ID that sent the critical alert
    pub device_unique_id: u32,
    
    /// Timestamp of the original payload
    pub original_timestamp_ms: u64,
    
    /// ACK type: 0 = ACK, 1 = NACK
    pub ack_type: u8,
    
    /// Padding for alignment
    pub _padding: [u8; 3],
}

impl AckPacket {
    /// Create an ACK packet
    pub fn ack(device_unique_id: u32, original_timestamp_ms: u64) -> Self {
        Self {
            device_unique_id,
            original_timestamp_ms,
            ack_type: 0,
            _padding: [0; 3],
        }
    }
    
    /// Create a NACK packet
    pub fn nack(device_unique_id: u32, original_timestamp_ms: u64) -> Self {
        Self {
            device_unique_id,
            original_timestamp_ms,
            ack_type: 1,
            _padding: [0; 3],
        }
    }
    
    /// Check if this is an ACK
    pub fn is_ack(&self) -> bool {
        self.ack_type == 0
    }
}

/// Extension methods for archived ACK packets
impl ArchivedAckPacket {
    /// Check if archived ACK packet is an ACK
    pub fn is_ack(&self) -> bool {
        self.ack_type == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sensor_payload_validation() {
        // Valid payload
        let result = SensorPayload::new(
            1,
            1000,
            1,
            50,
            1000,
            0x12345678,
            [0.0; ANOMALY_VECTOR_SIZE],
        );
        assert!(result.is_ok());
        
        // Invalid device ID
        let result = SensorPayload::new(
            0,
            1000,
            1,
            50,
            1000,
            0x12345678,
            [0.0; ANOMALY_VECTOR_SIZE],
        );
        assert!(result.is_err());
        
        // Invalid battery level
        let result = SensorPayload::new(
            1,
            1000,
            1,
            101,
            1000,
            0x12345678,
            [0.0; ANOMALY_VECTOR_SIZE],
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_payload_expiration() {
        let payload = SensorPayload::new(
            1,
            1000,
            1,
            50,
            100,
            0x12345678,
            [0.0; ANOMALY_VECTOR_SIZE],
        ).unwrap();
        
        assert!(!payload.is_expired(1050));
        assert!(payload.is_expired(1101));
    }
    
    #[test]
    fn test_dlt_transaction_validation() {
        let result = DLTTransactionRecord::new(
            1,
            0.95,
            true,
            0,
            [0u8; 32],
            [0u8; 64],
        );
        assert!(result.is_ok());
        
        let result = DLTTransactionRecord::new(
            0,
            0.95,
            true,
            0,
            [0u8; 32],
            [0u8; 64],
        );
        assert!(result.is_err());
    }
}
