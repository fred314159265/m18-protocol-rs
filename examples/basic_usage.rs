//! Basic Usage Example
//!
//! This example demonstrates the core functionality of the M18 protocol library:
//! - Listing and selecting serial ports
//! - Establishing battery connection
//! - Reading specific registers
//! - Simulating charger behavior
//! - Writing custom messages to battery memory
//! - Debug output for protocol analysis
//!
//! Usage:
//!   cargo run --example basic_usage                  # Interactive mode
//!   cargo run --example basic_usage -- COM3          # Specify port
//!   cargo run --example basic_usage -- /dev/ttyUSB0
//!
//! Set RUST_LOG environment variable to control logging:
//!   RUST_LOG=debug cargo run --example basic_usage
//!   RUST_LOG=info cargo run --example basic_usage

use inquire::Select;
use log::info;
use m18_protocol::{OutputFormat, Result, M18};
use std::time::Duration;

/// Interactive serial port selection using inquire
fn select_port() -> Result<String> {
    let ports = M18::list_ports()?;

    if ports.is_empty() {
        eprintln!("No serial ports found!");
        std::process::exit(1);
    }

    let port_names: Vec<String> = ports
        .iter()
        .map(|p| format!("{} - {:?}", p.port_name, p.port_type))
        .collect();

    let selection = Select::new("Select a serial port:", port_names)
        .prompt()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Selection cancelled: {}", e),
            )
        })?;

    // Extract just the port name (before " - ")
    let port_name = selection.split(" - ").next().unwrap().to_string();
    Ok(port_name)
}

fn main() -> Result<()> {
    // Initialize logger with default info level if RUST_LOG is not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Get port name from command line argument or interactive selection
    let port_name = std::env::args()
        .nth(1)
        .map(Ok)
        .unwrap_or_else(|| select_port())?;

    info!("Connecting to M18 battery on {}...", port_name);
    let mut m18 = M18::new(&port_name)?;

    // Enable debug printing to see protocol messages
    m18.set_debug_print(true, true);

    // Basic battery operations
    info!("=== Basic Battery Operations ===");

    // Reset and check connection
    if m18.reset()? {
        info!("✓ Battery connection established");
    } else {
        info!("✗ Failed to establish battery connection");
        return Ok(());
    }

    // Read a few key registers
    info!("=== Reading Key Registers ===");
    let key_registers = vec![2, 4, 12]; // Serial, Manufacture date, Cell voltages
    m18.print_registers(&key_registers, OutputFormat::Label, true)?;

    // Test charger simulation for 5 seconds
    info!("=== Charger Simulation Test ===");
    m18.simulate_for(Duration::from_secs(5))?;

    // // Write a test message to battery memory
    // info!("=== Writing Test Message ===");
    // m18.write_message("")?;

    // Read the message back
    info!("Reading message back...");
    m18.print_registers(&[7], OutputFormat::Label, true)?; // Register 7 is the note field

    info!("=== Basic Usage Complete ===");

    Ok(())
}
