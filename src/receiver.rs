use std::net::UdpSocket;
use std::time::Instant;

use rkyv::check_archived_root;

use crate::contracts::SensorPayload;
use crate::errors::{CyDnAError, Result};

pub struct Receiver;

impl Receiver {
    pub fn receive<'a>(
        socket: &UdpSocket,
        buffer: &'a mut [u8],
    ) -> Result<(&'a crate::contracts::ArchivedSensorPayload, usize, std::net::SocketAddr)> {
        let (bytes_received, sender_addr) = socket.recv_from(buffer)
            .map_err(|e| CyDnAError::IoError(e.to_string()))?;
        
        if bytes_received < std::mem::size_of::<SensorPayload>() {
            return Err(CyDnAError::InvalidPacketLength {
                expected: std::mem::size_of::<SensorPayload>(),
                received: bytes_received,
            });
        }
        
        let archived = check_archived_root::<SensorPayload>(&buffer[..bytes_received])
            .map_err(|_| CyDnAError::DeserializationError(
                "Failed to validate archived payload structure".to_string()
            ))?;
        
        Ok((archived, bytes_received, sender_addr))
    }
    
    pub fn receive_with_ttl_check<'a>(
        socket: &UdpSocket,
        buffer: &'a mut [u8],
        current_time_ms: u64,
    ) -> Result<(&'a crate::contracts::ArchivedSensorPayload, usize, std::net::SocketAddr)> {
        let (archived, bytes_received, sender_addr) = Self::receive(socket, buffer)?;
        
        let timestamp_ms = archived.timestamp_ms_utc;
        let ttl_ms = archived.time_to_live_ms as u64;
        
        if current_time_ms > timestamp_ms.saturating_add(ttl_ms) {
            return Err(CyDnAError::PayloadExpired {
                timestamp_ms,
                ttl_ms: ttl_ms as u16,
            });
        }
        
        Ok((archived, bytes_received, sender_addr))
    }
    
    pub fn receive_validated<'a>(
        socket: &UdpSocket,
        buffer: &'a mut [u8],
        current_time_ms: u64,
    ) -> Result<(&'a crate::contracts::ArchivedSensorPayload, usize, std::net::SocketAddr)> {
        let (archived, bytes_received, sender_addr) = Self::receive_with_ttl_check(
            socket,
            buffer,
            current_time_ms,
        )?;
        
        let _crc = archived.raw_data_hash_crc;
        
        if archived.device_unique_id == 0 {
            return Err(CyDnAError::InvalidDeviceId(0));
        }
        
        if archived.battery_level_percent > 100 {
            return Err(CyDnAError::InvalidBatteryLevel(archived.battery_level_percent));
        }
        
        Ok((archived, bytes_received, sender_addr))
    }
    
    pub fn receive_batch(
        socket: &UdpSocket,
        count: usize,
        buffer_size: usize,
    ) -> Result<Vec<Vec<u8>>> {
        let mut batch = Vec::with_capacity(count);
        let mut recv_buffer = vec![0u8; buffer_size];
        
        for _ in 0..count {
            let (bytes_received, _) = socket.recv_from(&mut recv_buffer)
                .map_err(|e| CyDnAError::IoError(e.to_string()))?;
            
            batch.push(recv_buffer[..bytes_received].to_vec());
        }
        
        Ok(batch)
    }
}

pub struct ReceiverBuilder {
    buffer_size: usize,
    enable_crc_check: bool,
    enable_ttl_check: bool,
}

impl ReceiverBuilder {
    pub fn new() -> Self {
        Self {
            buffer_size: crate::MAX_PAYLOAD_SIZE,
            enable_crc_check: true,
            enable_ttl_check: true,
        }
    }
    
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    
    pub fn with_crc_check(mut self, enable: bool) -> Self {
        self.enable_crc_check = enable;
        self
    }
    
    pub fn with_ttl_check(mut self, enable: bool) -> Self {
        self.enable_ttl_check = enable;
        self
    }
    
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }
    
    pub fn is_crc_check_enabled(&self) -> bool {
        self.enable_crc_check
    }
    
    pub fn is_ttl_check_enabled(&self) -> bool {
        self.enable_ttl_check
    }
}

impl Default for ReceiverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ReceiveMetrics {
    pub bytes_received: u64,
    
    pub receive_us: u64,
    
    pub validation_us: u64,
    
    pub total_us: u64,
}

pub fn receive_with_metrics<'a>(
    socket: &UdpSocket,
    buffer: &'a mut [u8],
) -> Result<(&'a crate::contracts::ArchivedSensorPayload, ReceiveMetrics)> {
    let start = Instant::now();
    
    let receive_start = Instant::now();
    let (bytes_received, _sender_addr) = socket.recv_from(buffer)
        .map_err(|e| CyDnAError::IoError(e.to_string()))?;
    let receive_us = receive_start.elapsed().as_micros() as u64;
    
    if bytes_received < std::mem::size_of::<SensorPayload>() {
        return Err(CyDnAError::InvalidPacketLength {
            expected: std::mem::size_of::<SensorPayload>(),
            received: bytes_received,
        });
    }
    
    let validation_start = Instant::now();
    let archived = check_archived_root::<SensorPayload>(&buffer[..bytes_received])
        .map_err(|_| CyDnAError::DeserializationError(
            "Failed to validate archived payload structure".to_string()
        ))?;
    let validation_us = validation_start.elapsed().as_micros() as u64;
    
    let total_us = start.elapsed().as_micros() as u64;
    
    let metrics = ReceiveMetrics {
        bytes_received: bytes_received as u64,
        receive_us,
        validation_us,
        total_us,
    };
    
    Ok((archived, metrics))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_receiver_builder() {
        let builder = ReceiverBuilder::new()
            .with_buffer_size(2048)
            .with_crc_check(false);
        
        assert_eq!(builder.get_buffer_size(), 2048);
        assert!(!builder.is_crc_check_enabled());
        assert!(builder.is_ttl_check_enabled());
    }
}
