pub mod errors;
pub mod contracts;
pub mod transmitter;
pub mod receiver;
pub mod ack_manager;

pub use contracts::{SensorPayload, DLTTransactionRecord};
pub use errors::{CyDnAError, Result};

pub const CYNDA_VERSION: u16 = 1;

pub const MAX_PAYLOAD_SIZE: usize = 1024;

pub const ACK_TIMEOUT_MS: u64 = 100;

pub const MAX_RETRANSMIT_ATTEMPTS: u32 = 3;

pub const BACKOFF_MULTIPLIER: u64 = 2;
