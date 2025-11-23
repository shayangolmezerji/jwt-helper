/// Transmitter module - S-Layer communication logic
/// 
/// Handles serialization and transmission of SensorPayload over raw UDP.
/// Optimized for minimal latency with zero unnecessary allocations.

use std::net::UdpSocket;
use std::time::Instant;

use rkyv::to_bytes;

use crate::contracts::SensorPayload;
use crate::errors::{CyDnAError, Result};

/// Transmitter - Serializes and sends SensorPayload over UDP
/// 
/// This struct provides a stateless interface for sending time-critical
/// sensor data to the Gateway. All operations are optimized for latency.
pub struct Transmitter;

impl Transmitter {
    /// Serialize a SensorPayload to bytes using rkyv
    ///
    /// # Arguments
    /// * `payload` - The SensorPayload to serialize
    ///
    /// # Returns
    /// A Vec<u8> containing the serialized bytes
    ///
    /// # Performance
    /// This operation uses a stack-allocated serializer for the payload size,
    /// minimizing heap allocations and garbage collection pressure.
    pub fn serialize_payload(payload: &SensorPayload) -> Result<Vec<u8>> {
        // Serialize using rkyv's to_bytes
        // This is optimized for small, fixed-size payloads
        to_bytes::<_, 1024>(payload)
            .map(|aligned_vec| aligned_vec.to_vec())
            .map_err(|_| CyDnAError::SerializationError(
                "Failed to serialize SensorPayload".to_string()
            ))
    }
    
    /// Send a SensorPayload to a specific destination
    ///
    /// # Arguments
    /// * `socket` - Pre-existing UDP socket (must be bound)
    /// * `payload` - SensorPayload to transmit
    /// * `destination` - Destination address as string (e.g., "127.0.0.1:8080")
    ///
    /// # Returns
    /// Number of bytes transmitted, or CyDnAError on failure
    ///
    /// # Performance Notes
    /// - No socket creation overhead (caller provides socket)
    /// - Serialization happens once, inline
    /// - Send is non-blocking if socket is configured as such
    pub fn send(
        socket: &UdpSocket,
        payload: &SensorPayload,
        destination: &str,
    ) -> Result<usize> {
        // Serialize the payload to bytes
        let bytes = Self::serialize_payload(payload)?;
        
        // Validate payload size doesn't exceed UDP MTU-safe limit
        if bytes.len() > crate::MAX_PAYLOAD_SIZE {
            return Err(CyDnAError::BufferTooSmall {
                required: bytes.len(),
                available: crate::MAX_PAYLOAD_SIZE,
            });
        }
        
        // Send to destination
        socket.send_to(&bytes, destination)
            .map_err(|e| CyDnAError::IoError(e.to_string()))
    }
    
    /// Send a pre-serialized payload (for advanced use cases)
    ///
    /// Useful when you need to send the same serialized payload multiple times
    /// or have already serialized the data externally.
    pub fn send_raw(
        socket: &UdpSocket,
        bytes: &[u8],
        destination: &str,
    ) -> Result<usize> {
        if bytes.len() > crate::MAX_PAYLOAD_SIZE {
            return Err(CyDnAError::BufferTooSmall {
                required: bytes.len(),
                available: crate::MAX_PAYLOAD_SIZE,
            });
        }
        
        socket.send_to(bytes, destination)
            .map_err(|e| CyDnAError::IoError(e.to_string()))
    }
    
    /// Batch serialize multiple payloads (optimized for bulk operations)
    ///
    /// # Arguments
    /// * `payloads` - Slice of SensorPayloads to serialize
    ///
    /// # Returns
    /// Vec of serialized byte vectors
    pub fn serialize_batch(payloads: &[SensorPayload]) -> Result<Vec<Vec<u8>>> {
        payloads
            .iter()
            .map(Self::serialize_payload)
            .collect()
    }
}

/// TransmitterBuilder - Fluent interface for transmitter configuration
/// 
/// Allows configuration of transmitter behavior without creating
/// full state objects, keeping the transmitter itself stateless.
pub struct TransmitterBuilder {
    max_retries: u32,
    socket_timeout_ms: u64,
}

impl TransmitterBuilder {
    /// Create a new TransmitterBuilder with defaults
    pub fn new() -> Self {
        Self {
            max_retries: crate::MAX_RETRANSMIT_ATTEMPTS,
            socket_timeout_ms: crate::ACK_TIMEOUT_MS,
        }
    }
    
    /// Set maximum retransmission attempts
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
    
    /// Set socket timeout in milliseconds
    pub fn with_socket_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.socket_timeout_ms = timeout_ms;
        self
    }
    
    /// Get the configured max retries
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
    
    /// Get the configured socket timeout
    pub fn get_socket_timeout_ms(&self) -> u64 {
        self.socket_timeout_ms
    }
}

impl Default for TransmitterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for transmitter operations
#[derive(Debug, Clone)]
pub struct TransmitMetrics {
    /// Total bytes transmitted
    pub bytes_sent: u64,
    
    /// Serialization time in microseconds
    pub serialization_us: u64,
    
    /// Network transmission time in microseconds
    pub transmission_us: u64,
    
    /// Total operation time in microseconds
    pub total_us: u64,
}

/// Transmit with metrics collection
pub fn send_with_metrics(
    socket: &UdpSocket,
    payload: &SensorPayload,
    destination: &str,
) -> Result<TransmitMetrics> {
    let start = Instant::now();
    
    let serialization_start = Instant::now();
    let bytes = Transmitter::serialize_payload(payload)?;
    let serialization_us = serialization_start.elapsed().as_micros() as u64;
    
    if bytes.len() > crate::MAX_PAYLOAD_SIZE {
        return Err(CyDnAError::BufferTooSmall {
            required: bytes.len(),
            available: crate::MAX_PAYLOAD_SIZE,
        });
    }
    
    let transmission_start = Instant::now();
    let bytes_sent = socket.send_to(&bytes, destination)
        .map_err(|e| CyDnAError::IoError(e.to_string()))? as u64;
    let transmission_us = transmission_start.elapsed().as_micros() as u64;
    
    let total_us = start.elapsed().as_micros() as u64;
    
    Ok(TransmitMetrics {
        bytes_sent,
        serialization_us,
        transmission_us,
        total_us,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serialization() {
        let payload = SensorPayload::new(
            1,
            1000,
            1,
            50,
            1000,
            0x12345678,
            [0.1; crate::contracts::ANOMALY_VECTOR_SIZE],
        ).unwrap();
        
        let bytes = Transmitter::serialize_payload(&payload);
        assert!(bytes.is_ok());
        
        let bytes = bytes.unwrap();
        assert!(bytes.len() > 0);
        assert!(bytes.len() <= crate::MAX_PAYLOAD_SIZE);
    }
    
    #[test]
    fn test_batch_serialization() {
        let payloads = vec![
            SensorPayload::new(
                1, 1000, 1, 50, 1000, 0x12345678,
                [0.1; crate::contracts::ANOMALY_VECTOR_SIZE],
            ).unwrap(),
            SensorPayload::new(
                2, 2000, 1, 60, 1000, 0x87654321,
                [0.2; crate::contracts::ANOMALY_VECTOR_SIZE],
            ).unwrap(),
        ];
        
        let batch = Transmitter::serialize_batch(&payloads);
        assert!(batch.is_ok());
        assert_eq!(batch.unwrap().len(), 2);
    }
    
    #[test]
    fn test_transmitter_builder() {
        let builder = TransmitterBuilder::new()
            .with_max_retries(5)
            .with_socket_timeout_ms(200);
        
        assert_eq!(builder.get_max_retries(), 5);
        assert_eq!(builder.get_socket_timeout_ms(), 200);
    }
}
