/// ACK Manager module - Lightweight reliability mechanism
/// 
/// Implements custom minimal ACK/NACK protocol over UDP for guaranteed delivery
/// of critical alerts. Uses exponential backoff for retransmissions.

use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rkyv::to_bytes;

use crate::contracts::{AckPacket, SensorPayload};
use crate::errors::{CyDnAError, Result};

/// ACK Manager - Handles acknowledgment and retransmission logic
/// 
/// Provides a stateless interface for reliable critical alert delivery.
/// Manages retransmission timing and backoff strategy.
pub struct AckManager;

impl AckManager {
    /// Serialize an ACK packet
    fn serialize_ack(ack: &AckPacket) -> Result<Vec<u8>> {
        to_bytes::<_, 256>(ack)
            .map(|aligned_vec| aligned_vec.to_vec())
            .map_err(|_| CyDnAError::SerializationError(
                "Failed to serialize ACK packet".to_string()
            ))
    }
    
    /// Send an ACK packet (G-Layer -> S-Layer)
    ///
    /// Called by the Gateway after successfully processing a critical alert.
    /// Notifies the sensor to stop retransmitting.
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
    
    /// Send a NACK packet (G-Layer -> S-Layer)
    ///
    /// Called when payload fails validation, requesting retransmission.
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
    
    /// Wait for ACK with timeout
    ///
    /// Blocks until ACK is received or timeout occurs.
    /// # Arguments
    /// * `socket` - UDP socket configured with timeout
    /// * `device_unique_id` - Expected sender device ID
    /// * `original_timestamp_ms` - Expected payload timestamp
    /// * `buffer` - Buffer for receiving ACK packet
    ///
    /// # Returns
    /// true if valid ACK received, false if timeout
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
                
                // Validate ACK structure
                use rkyv::check_archived_root;
                let archived = check_archived_root::<AckPacket>(&buffer[..bytes_received])
                    .map_err(|_| CyDnAError::DeserializationError(
                        "Failed to parse ACK packet".to_string()
                    ))?;
                
                // Verify this ACK matches our payload
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
    
    /// Calculate exponential backoff delay
    ///
    /// Implements truncated exponential backoff with jitter.
    /// Formula: delay = min(base_ms * (backoff_multiplier ^ attempt), max_delay_ms)
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
    
    /// Send critical alert with automatic retransmission
    ///
    /// Transmits a critical payload and waits for ACK. If no ACK is received
    /// within the timeout, automatically retransmits with exponential backoff.
    ///
    /// # Arguments
    /// * `socket` - UDP socket (should have timeout configured)
    /// * `payload` - Critical SensorPayload to send
    /// * `gateway_address` - Destination gateway address
    /// * `max_retries` - Maximum retransmission attempts
    /// * `base_timeout_ms` - Initial timeout in milliseconds
    ///
    /// # Returns
    /// true if ACK received, false if max retries exceeded
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
            // Send payload
            Transmitter::send(socket, payload, gateway_address)?;
            
            // Calculate backoff for this attempt
            let timeout_ms = Self::calculate_backoff_ms(
                attempt,
                base_timeout_ms,
                base_timeout_ms * 10, // Max 10x base timeout
            );
            
            // Set socket timeout
            socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))
                .map_err(|e| CyDnAError::IoError(e.to_string()))?;
            
            // Wait for ACK
            if Self::wait_for_ack(
                socket,
                payload.device_unique_id,
                payload.timestamp_ms_utc,
                &mut ack_buffer,
            )? {
                return Ok(true);
            }
            
            // ACK not received, will retry
            if attempt == max_retries - 1 {
                return Err(CyDnAError::MaxRetriesExceeded);
            }
        }
        
        Err(CyDnAError::MaxRetriesExceeded)
    }
}

/// Retransmission state tracker
/// 
/// Maintains state for a single payload's retransmission attempts.
pub struct RetransmissionState {
    /// Device ID of the sender
    pub device_id: u32,
    
    /// Original payload timestamp
    pub payload_timestamp_ms: u64,
    
    /// Current attempt number (0-based)
    pub attempt: u32,
    
    /// Time of last transmission
    pub last_sent: Instant,
    
    /// Next scheduled retry time
    pub next_retry: Instant,
}

impl RetransmissionState {
    /// Create a new retransmission state
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
    
    /// Check if ready for next retry
    pub fn is_ready_for_retry(&self) -> bool {
        Instant::now() >= self.next_retry
    }
    
    /// Schedule next retry with exponential backoff
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
    
    /// Check if max retries exceeded
    pub fn is_exhausted(&self) -> bool {
        self.attempt >= crate::MAX_RETRANSMIT_ATTEMPTS
    }
}

/// ACK context - metadata for ACK/NACK handling
#[derive(Debug, Clone)]
pub struct AckContext {
    /// Sender device ID
    pub device_id: u32,
    
    /// Original payload timestamp
    pub timestamp_ms: u64,
    
    /// Time ACK was received
    pub ack_received_timestamp: u64,
    
    /// Round-trip time in milliseconds
    pub rtt_ms: u64,
    
    /// Whether this was an ACK (true) or NACK (false)
    pub is_ack: bool,
}

impl AckContext {
    /// Create a new ACK context
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
