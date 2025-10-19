/// Protocol command constants
pub const SYNC_BYTE: u8 = 0xAA;
pub const CAL_CMD: u8 = 0x55;
pub const CONF_CMD: u8 = 0x60;
pub const SNAP_CMD: u8 = 0x61;
pub const KEEPALIVE_CMD: u8 = 0x62;

/// Current limits
pub const CUTOFF_CURRENT: u16 = 300;
pub const MAX_CURRENT: u16 = 6000;

/// Initial ACC value
pub const INITIAL_ACC: u8 = 4;

/// ACC rotation values
pub const ACC_VALUES: [u8; 3] = [0x04, 0x0C, 0x1C];

/// Serial port configuration
pub const BAUD_RATE: u32 = 4800;
pub const TIMEOUT_MS: u64 = 2000;  // Increased from 800ms for Windows compatibility
pub const STOP_BITS: serialport::StopBits = serialport::StopBits::Two;

/// Timing constants (in milliseconds)
pub const RESET_BREAK_DURATION: u64 = 300;
pub const RESET_SETTLE_DURATION: u64 = 300;
pub const RESET_SYNC_DELAY: u64 = 10;
pub const KEEPALIVE_INTERVAL: u64 = 500;
pub const CONFIGURE_DELAY: u64 = 600;