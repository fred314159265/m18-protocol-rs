//! # M18 Protocol Library
//!
//! A Rust library for interfacing with Milwaukee M18 battery packs via serial communication.

pub mod constants;
pub mod data;
pub mod error;
pub mod protocol;
pub mod types;

pub use error::{M18Error, Result};
pub use protocol::M18;
pub use types::*;
