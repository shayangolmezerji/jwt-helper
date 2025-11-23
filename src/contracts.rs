use rkyv::{Archive, Deserialize, Serialize};

pub const ANOMALY_VECTOR_SIZE: usize = 32;

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
#[archive(check_bytes)]
pub struct SensorPayload {
    pub device_unique_id: u32,
    
    pub timestamp_ms_utc: u64,
    
    pub sensor_model_version: u16,
    
    pub battery_level_percent: u8,
    
    pub time_to_live_ms: u16,
    
    pub raw_data_hash_crc: u32,
    
    pub anomaly_ai_vector: [f32; ANOMALY_VECTOR_SIZE],
}

impl SensorPayload {
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
        
        if device_unique_id == 0 {
            return Err(CyDnAError::InvalidDeviceId(device_unique_id));
        }
        
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
    
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        current_time_ms > self.timestamp_ms_utc.saturating_add(self.time_to_live_ms as u64)
    }
    
    pub fn expiration_time_ms(&self) -> u64 {
        self.timestamp_ms_utc.saturating_add(self.time_to_live_ms as u64)
    }
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct DLTTransactionRecord {
    pub gateway_unique_id: u32,
    
    pub final_anomaly_score: f32,
    
    pub is_critical_alert: bool,
    
    pub consensus_mode_used: u8,
    
    pub source_payload_hash: [u8; 32],
    
    pub gateway_signature: [u8; 64],
}

impl DLTTransactionRecord {
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

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
#[archive(check_bytes)]
pub struct AckPacket {
    pub device_unique_id: u32,
    
    pub original_timestamp_ms: u64,
    
    pub ack_type: u8,
    
    pub _padding: [u8; 3],
}

impl AckPacket {
    pub fn ack(device_unique_id: u32, original_timestamp_ms: u64) -> Self {
        Self {
            device_unique_id,
            original_timestamp_ms,
            ack_type: 0,
            _padding: [0; 3],
        }
    }
    
    pub fn nack(device_unique_id: u32, original_timestamp_ms: u64) -> Self {
        Self {
            device_unique_id,
            original_timestamp_ms,
            ack_type: 1,
            _padding: [0; 3],
        }
    }
    
    pub fn is_ack(&self) -> bool {
        self.ack_type == 0
    }
}

impl ArchivedAckPacket {
    pub fn is_ack(&self) -> bool {
        self.ack_type == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sensor_payload_validation() {
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
