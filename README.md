# M18 Protocol Library

A Rust library for interfacing with Milwaukee M18 battery packs via serial communication. Made possible by the reverse engineering of the protocol by others in the [m18-protocol](https://github.com/mnh-jansson/m18-protocol) project.

## Features

- **Battery Diagnostics**: Read comprehensive data including cell voltages, temperatures, charge cycles, discharge history, and usage statistics.
- **Charger Simulation**: Mimic charger communication to maintain battery connection.
- **Structured Data**: Extract and parse data from 184 defined registers with proper typing.
- **Health Reports**: Generate comprehensive battery health summaries with JSON export.
- **Cross-Platform**: Should work on Windows, Linux, and macOS.

## Hardware Requirements

You will need a UART-to-USB adapter connected to an M18 battery via a voltage level shifter. See [the hardware documentation](https://github.com/mnh-jansson/m18-protocol/tree/master?tab=readme-ov-file#hardware) in the Python reference implementation for how to do this.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
m18-protocol = { git = "https://github.com/fred314159265/m18-protocol-rs.git" }
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
