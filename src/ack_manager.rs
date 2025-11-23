use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rkyv::to_bytes;

use crate::contracts::{AckPacket, SensorPayload};
use crate::errors::{CyDnAError, Result};

pub struct AckManager;

impl AckManager {
    fn serialize_ack(ack: &AckPacket) -> Result<Vec<u8>> {
        to_bytes::<_, 256>(ack)
            .map(|aligned_vec| aligned_vec.to_vec())
            .map_err(|_| CyDnAError::SerializationError(
                "Failed to serialize ACK packet".to_string()
            ))
    }
    
    pub fn send_ack(
        socket: &UdpSocket,
        device_unique_id: u32,
        original_timestamp_ms: u64,
        destination: &str,
    ) -> Result<usize> {
        let ack = AckPacket::ack(device_unique_id, original_timestamp_ms);
        let bytes = Self::serialize_ack(&ack)?;
        
        socket.send_to(&bytes, destination)
            .map_err(|e| CyDnAError::IoError(e.to_string()))
    }
    
    pub fn send_nack(
        socket: &UdpSocket,
        device_unique_id: u32,
        original_timestamp_ms: u64,
        destination: &str,
    ) -> Result<usize> {
        let nack = AckPacket::nack(device_unique_id, original_timestamp_ms);
        let bytes = Self::serialize_ack(&nack)?;
        
        socket.send_to(&bytes, destination)
            .map_err(|e| CyDnAError::IoError(e.to_string()))
    }
    
    pub fn wait_for_ack(
        socket: &UdpSocket,
        device_unique_id: u32,
        original_timestamp_ms: u64,
        buffer: &mut [u8],
    ) -> Result<bool> {
        match socket.recv_from(buffer) {
            Ok((bytes_received, _)) => {
                if bytes_received < 16 {
                    return Ok(false);
                }
                
                use rkyv::check_archived_root;
                let archived = check_archived_root::<AckPacket>(&buffer[..bytes_received])
                    .map_err(|_| CyDnAError::DeserializationError(
                        "Failed to parse ACK packet".to_string()
                    ))?;
                
                if archived.device_unique_id == device_unique_id 
                    && archived.original_timestamp_ms == original_timestamp_ms
                    && archived.is_ack() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock 
                   || e.kind() == std::io::ErrorKind::TimedOut => {
                Ok(false)
            }
            Err(e) => Err(CyDnAError::IoError(e.to_string())),
        }
    }
    
    pub fn calculate_backoff_ms(
        attempt: u32,
        base_ms: u64,
        max_delay_ms: u64,
    ) -> u64 {
        let multiplier = crate::BACKOFF_MULTIPLIER;
        let backoff = base_ms.saturating_mul(
            multiplier.saturating_pow(attempt)
        );
        backoff.min(max_delay_ms)
    }
    
    pub fn send_critical_alert(
        socket: &UdpSocket,
        payload: &SensorPayload,
        gateway_address: &str,
        max_retries: u32,
        base_timeout_ms: u64,
    ) -> Result<bool> {
        use crate::transmitter::Transmitter;
        
        let mut ack_buffer = vec![0u8; 256];
        
        for attempt in 0..max_retries {
            Transmitter::send(socket, payload, gateway_address)?;
            
            let timeout_ms = Self::calculate_backoff_ms(
                attempt,
                base_timeout_ms,
                base_timeout_ms * 10, // Max 10x base timeout
            );
            
            socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))
                .map_err(|e| CyDnAError::IoError(e.to_string()))?;
            
            if Self::wait_for_ack(
                socket,
                payload.device_unique_id,
                payload.timestamp_ms_utc,
                &mut ack_buffer,
            )? {
                return Ok(true);
            }
            
            if attempt == max_retries - 1 {
                return Err(CyDnAError::MaxRetriesExceeded);
            }
        }
        
        Err(CyDnAError::MaxRetriesExceeded)
    }
}

pub struct RetransmissionState {
    pub device_id: u32,
    
    pub payload_timestamp_ms: u64,
    
    pub attempt: u32,
    
    pub last_sent: Instant,
    
    pub next_retry: Instant,
}

impl RetransmissionState {
    pub fn new(device_id: u32, payload_timestamp_ms: u64) -> Self {
        let now = Instant::now();
        Self {
            device_id,
            payload_timestamp_ms,
            attempt: 0,
            last_sent: now,
            next_retry: now,
        }
    }
    
    pub fn is_ready_for_retry(&self) -> bool {
        Instant::now() >= self.next_retry
    }
    
    pub fn schedule_next_retry(&mut self, base_timeout_ms: u64) {
        let backoff_ms = AckManager::calculate_backoff_ms(
            self.attempt,
            base_timeout_ms,
            base_timeout_ms * 10,
        );
        
        self.next_retry = Instant::now() + Duration::from_millis(backoff_ms);
        self.attempt += 1;
        self.last_sent = Instant::now();
    }
    
    pub fn is_exhausted(&self) -> bool {
        self.attempt >= crate::MAX_RETRANSMIT_ATTEMPTS
    }
}

#[derive(Debug, Clone)]
pub struct AckContext {
    pub device_id: u32,
    
    pub timestamp_ms: u64,
    
    pub ack_received_timestamp: u64,
    
    pub rtt_ms: u64,
    
    pub is_ack: bool,
}

impl AckContext {
    pub fn new(
        device_id: u32,
        timestamp_ms: u64,
        is_ack: bool,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            device_id,
            timestamp_ms,
            ack_received_timestamp: now,
            rtt_ms: now.saturating_sub(timestamp_ms),
            is_ack,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exponential_backoff() {
        assert_eq!(AckManager::calculate_backoff_ms(0, 100, 5000), 100);
        assert_eq!(AckManager::calculate_backoff_ms(1, 100, 5000), 200);
        assert_eq!(AckManager::calculate_backoff_ms(2, 100, 5000), 400);
        assert_eq!(AckManager::calculate_backoff_ms(3, 100, 5000), 800);
        assert_eq!(AckManager::calculate_backoff_ms(10, 100, 5000), 5000); // Capped
    }
    
    #[test]
    fn test_retransmission_state() {
        let mut state = RetransmissionState::new(1, 1000);
        
        assert_eq!(state.attempt, 0);
        assert!(!state.is_exhausted());
        assert!(state.is_ready_for_retry());
        
        state.schedule_next_retry(100);
        assert_eq!(state.attempt, 1);
        assert!(!state.is_ready_for_retry()); // Just scheduled
    }
    
    #[test]
    fn test_ack_context() {
        let ctx = AckContext::new(1, 1000, true);
        assert_eq!(ctx.device_id, 1);
        assert_eq!(ctx.timestamp_ms, 1000);
        assert!(ctx.is_ack);
    }
}
