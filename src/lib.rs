//! # M18 Protocol Library
//!
//! A Rust library for interfacing with Milwaukee M18 battery packs via serial communication.
//! This library reverse-engineers the charging protocol to read battery diagnostics and 
//! simulate charger behavior.
//!
//! ## Features
//!
//! - Read comprehensive battery diagnostics (voltages, temperatures, charge cycles, etc.)
//! - Simulate charger communication to maintain battery connection
//! - Extract structured data from 184 defined registers
//! - Generate battery health reports
//! - Optional form submission for data collection
//!
//! ## Example
//!
//! ```no_run
//! use m18_protocol::M18;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut m18 = M18::new("/dev/ttyUSB0")?;
//!     let health = m18.health_report()?;
//!     println!("Battery voltage: {:.2}V", health.pack_voltage);
//!     Ok(())
//! }
//! ```

pub mod constants;
pub mod data;
pub mod error;
pub mod protocol;
pub mod types;

#[cfg(feature = "form-submission")]
pub mod forms;

pub use error::{M18Error, Result};
pub use protocol::M18;
pub use types::*;