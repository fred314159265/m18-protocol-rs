# M18 Protocol Library

A Rust library for reading Milwaukee M18 battery diagnostics over a serial port. Health reports, register reads, and charger simulation.

The matching board is [m18-usb-interface](https://github.com/fred314159265/m18-usb-interface): a USB-C adapter with isolation and the FTDI setup this library expects.

<img src="https://github.com/fred314159265/m18-usb-interface/raw/main/Images/pcb-3d.png" alt="3D render of the M18 USB interface PCB" width="640">

The protocol was reverse engineered in [m18-protocol](https://github.com/mnh-jansson/m18-protocol). This crate is a Rust port of that work.

## Features

- **Battery Diagnostics**: Read comprehensive data including cell voltages, temperatures, charge cycles, discharge history, and usage statistics.
- **Charger Simulation**: Mimic charger communication to maintain battery connection.
- **Structured Data**: Extract and parse data from 184 defined registers with proper typing.
- **Health Reports**: Generate comprehensive battery health summaries with JSON export.
- **Cross-Platform**: Should work on Windows, Linux, and macOS.

## Hardware

[m18-usb-interface](https://github.com/fred314159265/m18-usb-interface) is the adapter this library is written for. It isolates the pack from the PC, shifts J2 to the voltage the battery expects, and includes an FT_Prog template that inverts TX and RX.

A discrete USB-UART adapter works too, with the level shifting described in the [Python project's hardware section](https://github.com/mnh-jansson/m18-protocol#hardware).

The library opens the port itself at 4800 baud, 8 data bits, and 2 stop bits. Reset, and the idle and high states on J2, use the serial break and DTR lines, so the adapter has to let the host control those. The FT234X template on the interface board does.

## Installation

This crate is not published on crates.io. Add a git dependency:

```toml
[dependencies]
m18-protocol = { git = "https://github.com/fred314159265/m18-protocol-rs.git" }
```

## Quick start

```bash
cargo run --example health_report -- COM3
```

On Linux or macOS, pass the serial device instead, for example `/dev/ttyUSB0`.

```rust
fn main() -> m18_protocol::Result<()> {
    let mut m18 = m18_protocol::M18::new("COM3")?;
    let report = m18.health_report()?;
    println!("Pack voltage: {:.2} V", report.pack_voltage);
    Ok(())
}
```

## Examples

### Health Report

Generates a complete battery health report with interactive port selection:

```bash
# Interactive mode (will prompt for serial port)
cargo run --example health_report

# Or specify port directly
cargo run --example health_report -- COM3
# or on Linux/Mac
cargo run --example health_report -- /dev/ttyUSB0
```

### Basic Usage

Demonstrates core functionality including register reading, charger simulation, and message writing:

```bash
# Interactive mode
cargo run --example basic_usage

# With specific port
cargo run --example basic_usage -- /dev/ttyUSB0
```

## Disclaimer

**⚠️ IMPORTANT SAFETY NOTICE ⚠️**

This library is for **educational and research purposes only**. Use at your own risk.

- Improperly interfacing with battery packs can cause fires, explosions, or injury.
- This is NOT an official Milwaukee tool or endorsed by Milwaukee Tool.
- The author(s) assume NO responsibility for damage to batteries, equipment, or injury.

**By using this library, you acknowledge these risks.**

## Acknowledgments

This Rust implementation is based on the original reverse engineering and Python implementation [m18-protocol](https://github.com/mnh-jansson/m18-protocol). Special thanks to all contributors who helped document the protocol and "Tool Scientist" on YouTube for his [great video(s)](https://youtu.be/tHj0-Gzvbeo?si=bGzYBjuulpaU17mV) too.

The companion PCB is [m18-usb-interface](https://github.com/fred314159265/m18-usb-interface).
