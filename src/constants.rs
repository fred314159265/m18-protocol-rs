//! Protocol constants for M18 battery communication.
//!
//! This module defines all the constants used in the M18 UART protocol,
//! including command bytes, timing parameters, and serial port configuration.

/// Synchronization byte used for baudrate detection
pub const SYNC_BYTE: u8 = 0xAA;

/// Cutoff current in milliamps
pub const CUTOFF_CURRENT: u16 = 300;

/// Maximum current in milliamps
pub const MAX_CURRENT: u16 = 6000;

/// Initial ACC value to use after reset
pub const INITIAL_ACC: u8 = 4;

/// Valid ACC values that rotate through command sequence
pub const ACC_VALUES: [u8; 3] = [0x04, 0x0C, 0x1C];

/// Baud rate (4800 bps)
pub const BAUD_RATE: u32 = 4800;

/// Read timeout in milliseconds
pub const TIMEOUT_MS: u64 = 2000;

/// Stop bits configuration (2 stop bits required)
pub const STOP_BITS: serialport::StopBits = serialport::StopBits::Two;

/// Duration to hold break condition during reset
pub const RESET_BREAK_DURATION_MS: u64 = 300;

/// Settling time after clearing break before sending sync
pub const RESET_SETTLE_DURATION_MS: u64 = 300;

/// Delay after successful sync response
pub const RESET_SYNC_DELAY_MS: u64 = 10;

/// Interval between keepalive messages during simulation
pub const KEEPALIVE_INTERVAL_MS: u64 = 500;

/// Delay between configuration commands
pub const CONFIGURE_DELAY_MS: u64 = 600;
