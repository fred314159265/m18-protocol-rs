//! Protocol constants for M18 battery communication.
//!
//! This module defines all the constants used in the M18 UART protocol,
//! including command bytes, timing parameters, and serial port configuration.

/// Protocol command constants.

/// Synchronization byte used for baudrate detection
pub const SYNC_BYTE: u8 = 0xAA;

/// Calibration/interrupt command
pub const CAL_CMD: u8 = 0x55;

/// Configuration command (set charging parameters)
pub const CONF_CMD: u8 = 0x60;

/// Snapshot command (request battery state)
pub const SNAP_CMD: u8 = 0x61;

/// Keepalive command (periodic during charging)
pub const KEEPALIVE_CMD: u8 = 0x62;

/// Current limits for charging configuration.

/// Cutoff current in milliamps
pub const CUTOFF_CURRENT: u16 = 300;

/// Maximum current in milliamps
pub const MAX_CURRENT: u16 = 6000;

/// Accumulator (ACC) values for command sequencing.

/// Initial ACC value to use after reset
pub const INITIAL_ACC: u8 = 4;

/// Valid ACC values that rotate through command sequence
pub const ACC_VALUES: [u8; 3] = [0x04, 0x0C, 0x1C];

/// Serial port configuration.

/// Baud rate (4800 bps)
pub const BAUD_RATE: u32 = 4800;

/// Read timeout in milliseconds (increased for Windows compatibility)
pub const TIMEOUT_MS: u64 = 2000;

/// Stop bits configuration (2 stop bits required)
pub const STOP_BITS: serialport::StopBits = serialport::StopBits::Two;

/// Timing constants (all in milliseconds).

/// Duration to hold break condition during reset
pub const RESET_BREAK_DURATION: u64 = 300;

/// Settling time after clearing break before sending sync
pub const RESET_SETTLE_DURATION: u64 = 300;

/// Delay after successful sync response
pub const RESET_SYNC_DELAY: u64 = 10;

/// Interval between keepalive messages during simulation
pub const KEEPALIVE_INTERVAL: u64 = 500;

/// Delay between configuration commands
pub const CONFIGURE_DELAY: u64 = 600;