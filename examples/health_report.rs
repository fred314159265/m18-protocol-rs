//! Health Report Example
//!
//! This example demonstrates how to generate a comprehensive health report
//! for an M18 battery. It includes:
//! - Interactive serial port selection (or command-line argument)
//! - Complete health report with formatted output
//! - Structured health data with JSON export
//!
//! Usage:
//!   cargo run --example health_report              # Interactive mode
//!   cargo run --example health_report -- COM3      # Specify port
//!   cargo run --example health_report -- /dev/ttyUSB0
//!
//! Set RUST_LOG environment variable to control logging:
//!   RUST_LOG=debug cargo run --example health_report
//!   RUST_LOG=info cargo run --example health_report

use inquire::Select;
use log::{error, info};
use m18_protocol::{Result, M18};

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

    // Generate and display health report
    info!("=== M18 Battery Health Report ===");

    match m18.print_health_report() {
        Ok(()) => {
            info!("=== Health Report Complete ===");
        }
        Err(e) => {
            error!("Failed to generate health report: {}", e);
            error!("Check that:");
            error!("1. Battery is connected properly");
            error!("2. Correct serial port is specified");
            error!("3. UART connections are correct:");
            error!("   - UART-TX to M18-J2");
            error!("   - UART-RX to M18-J1");
            error!("   - UART-GND to M18-GND");
        }
    }

    // Alternatively, get the health report as a structured object
    info!("=== Structured Health Data ===");
    match m18.health_report() {
        Ok(report) => {
            info!("Battery Type: {}", report.battery_type);
            info!("Description: {}", report.battery_description);
            info!("Pack Voltage: {:.2}V", report.pack_voltage);
            info!("Cell Imbalance: {}mV", report.cell_imbalance);
            info!(
                "Total Discharge: {:.2}Ah",
                report.usage_stats.total_discharge_ah
            );

            // Export to JSON (requires serde feature)
            if let Ok(json) = serde_json::to_string_pretty(&report) {
                info!("JSON Export:");
                info!("{}", json);
            }
        }
        Err(e) => {
            error!("Failed to get structured health data: {}", e);
        }
    }

    Ok(())
}
