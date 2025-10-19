# M18 Protocol Library

A Rust library for interfacing with Milwaukee M18 battery packs via serial communication. This library reverse-engineers the charging protocol to read battery diagnostics and simulate charger behavior.

## Features

- **Battery Diagnostics**: Read comprehensive data including cell voltages, temperatures, charge cycles, discharge history, and usage statistics
- **Charger Simulation**: Mimic charger communication to maintain battery connection
- **Structured Data**: Extract data from 184 defined registers with proper typing
- **Health Reports**: Generate comprehensive battery health summaries
- **Optional Form Submission**: Submit battery data for research collection (requires `form-submission` feature)

## Hardware Requirements

To use this library, you need a UART interface connected to the M18 battery as follows:

- **UART-TX** → **M18-J2** (20V when charging)
- **UART-RX** → **M18-J1** (data line)
- **UART-GND** → **M18-GND** (ground)

**Serial Settings:**
- Baud rate: 4800
- Stop bits: 2
- Timeout: 800ms

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
m18-protocol = "0.1.0"

# For form submission functionality
m18-protocol = { version = "0.1.0", features = ["form-submission"] }
```

## Quick Start

```rust
use m18_protocol::{M18, OutputFormat, Result};

fn main() -> Result<()> {
    // Connect to battery (replace with your serial port)
    let mut m18 = M18::new("/dev/ttyUSB0")?;
    
    // Generate health report
    m18.print_health_report()?;
    
    // Read specific registers
    let registers = vec![2, 4, 12]; // Serial, manufacture date, cell voltages
    m18.print_registers(&registers, OutputFormat::Label, true)?;
    
    Ok(())
}
```

## Examples

Run the examples:

```bash
# Basic usage example
cargo run --example basic_usage

# Health report example  
cargo run --example health_report
```

## API Reference

### Core Methods

- `M18::new(port: &str)` - Create new M18 interface
- `M18::list_ports()` - List available serial ports
- `reset()` - Reset battery connection
- `health_report()` - Generate comprehensive health report
- `print_health_report()` - Print formatted health report
- `read_registers(ids: &[usize], force_refresh: bool)` - Read specific registers
- `print_registers(ids: &[usize], format: OutputFormat, force_refresh: bool)` - Print register data
- `simulate_for(duration: Duration)` - Simulate charger communication
- `write_message(message: &str)` - Write message to battery memory

## Supported Battery Types

The library recognizes these M18 battery types:

- **Type 37**: 2Ah CP (5s1p 18650)
- **Type 40/165**: 5Ah XC (5s2p 18650)
- **Type 46**: 6Ah XC (5s2p 18650)
- **Type 104**: 3Ah HO (5s1p 21700)
- **Type 106**: 6Ah HO (5s2p 21700)
- **Type 107**: 8Ah HO (5s2p 21700)
- **Type 108**: 12Ah HO (5s3p 21700)
- **Type 384**: 12Ah Forge (5s3p 21700 tabless)

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.

## Disclaimer

This library is for educational and research purposes. Use at your own risk. The authors are not responsible for any damage to batteries or equipment.
