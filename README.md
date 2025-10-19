# M18 Protocol Library

A Rust library for interfacing with Milwaukee M18 battery packs via serial communication. This library reverse-engineers the Redlink charging protocol to read battery diagnostics and simulate charger behavior.

## Features

- **Battery Diagnostics**: Read comprehensive data including cell voltages, temperatures, charge cycles, discharge history, and usage statistics
- **Charger Simulation**: Mimic charger communication to maintain battery connection
- **Structured Data**: Extract and parse data from 184 defined registers with proper typing
- **Health Reports**: Generate comprehensive battery health summaries with JSON export
- **Type Safety**: Strongly-typed API with full error handling
- **Cross-Platform**: Works on Windows, Linux, and macOS
- **Optional Form Submission**: Submit battery data for community research (requires `form-submission` feature)

## Hardware Requirements

You need a UART-to-USB adapter connected to the M18 battery. The library has been tested with CH340, FTDI, and CP2102 adapters.

**Physical Connections:**
```
UART-TX  →  M18-J2  (Battery powers this line to ~20V when active)
UART-RX  →  M18-J1  (Data line, 3.3V logic)
UART-GND →  M18-GND (Common ground)
```

**⚠️ Important:** Most UART adapters are **NOT** 20V tolerant on TX! You'll need isolation circuitry or a level shifter. See the hardware documentation in the Python reference implementation for circuit designs.

**Serial Configuration:**
- Baud rate: 4800 bps
- Stop bits: 2
- Data bits: 8
- Parity: None
- Timeout: 2000ms (increased for Windows compatibility)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
m18-protocol = "0.1.0"

# Or with form submission feature for community data collection
m18-protocol = { version = "0.1.0", features = ["form-submission"] }
```

## Quick Start

### Basic Health Report

The simplest way to get started is to generate a health report:

```rust
use m18_protocol::M18;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to battery (Windows: "COM3", Linux/Mac: "/dev/ttyUSB0")
    let mut m18 = M18::new("COM3")?;

    // Print comprehensive health report
    m18.print_health_report()?;

    Ok(())
}
```

This will output:
```
Reading battery. This will take 5-10sec

Type: 108 [12Ah HO (5s3p 21700)]
E-serial: 1234567 (does NOT match case serial)

Manufacture date: 2023-05-15
Days since 1st charge: 456
Days since last tool use: 3
Days since last charge: 1
Pack voltage: 20.12V
Cell Voltages (mV): [4024, 4025, 4023, 4024, 4025]
Cell Imbalance (mV): 2
Temperature (deg C): 23.50

CHARGING STATS:
Charge count [Redlink, dumb, (total)]: 145, 12, (157)
Total charge time: 78:23:15
...
```

### Export Health Data as JSON

```rust
use m18_protocol::M18;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut m18 = M18::new("/dev/ttyUSB0")?;

    // Get structured health report
    let report = m18.health_report()?;

    // Export to JSON
    let json = serde_json::to_string_pretty(&report)?;
    println!("{}", json);

    // Or access specific fields
    println!("Battery voltage: {:.2}V", report.pack_voltage);
    println!("Total cycles: {:.2}", report.usage_stats.total_discharge_cycles);
    println!("Times overheated: {}", report.usage_stats.times_overheated);

    Ok(())
}
```

### List Available Serial Ports

```rust
use m18_protocol::M18;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ports = M18::list_ports()?;

    println!("Available serial ports:");
    for port in ports {
        println!("  {} - {:?}", port.port_name, port.port_type);
    }

    Ok(())
}
```

### Read Specific Registers

```rust
use m18_protocol::{M18, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut m18 = M18::new("COM3")?;

    // Read specific registers by ID
    let register_ids = vec![
        2,   // Battery serial number
        4,   // Manufacture date
        12,  // Cell voltages
        29,  // Total discharge (amp-seconds)
    ];

    m18.print_registers(&register_ids, OutputFormat::Label, true)?;

    Ok(())
}
```

### Simulate Charger Behavior

```rust
use m18_protocol::M18;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut m18 = M18::new("/dev/ttyUSB0")?;

    // Simulate charging for 30 seconds
    // This keeps the battery active and responding to commands
    m18.simulate_for(Duration::from_secs(30))?;

    Ok(())
}
```

## Running the Examples

The library includes two comprehensive examples:

### Health Report Example

Generates a complete battery health report with interactive port selection:

```bash
# Interactive mode (will prompt for serial port)
cargo run --example health_report

# Or specify port directly
cargo run --example health_report -- COM3
# or on Linux/Mac
cargo run --example health_report -- /dev/ttyUSB0
```

### Basic Usage Example

Demonstrates core functionality including register reading, charger simulation, and message writing:

```bash
# Interactive mode
cargo run --example basic_usage

# With specific port
cargo run --example basic_usage -- /dev/ttyUSB0
```

## API Overview

### Main Struct: `M18`

**Creation & Setup:**
- `M18::new(port: &str) -> Result<M18>` - Create new interface on specified serial port
- `M18::list_ports() -> Result<Vec<SerialPortInfo>>` - List available serial ports
- `set_debug_print(tx: bool, rx: bool)` - Enable/disable protocol debug output

**Battery Communication:**
- `reset() -> Result<bool>` - Reset battery and establish communication
- `configure(state: u8) -> Result<Vec<u8>>` - Send configuration command
- `get_snapchat() -> Result<Vec<u8>>` - Get current battery snapshot
- `keepalive() -> Result<Vec<u8>>` - Send keepalive during charging
- `calibrate() -> Result<Vec<u8>>` - Send calibration command
- `simulate_for(duration: Duration) -> Result<()>` - Simulate charger for specified time

**Reading Diagnostics:**
- `health_report() -> Result<HealthReport>` - Generate structured health report
- `print_health_report() -> Result<()>` - Print formatted health report to stdout
- `read_registers(&[usize], force_refresh: bool) -> Result<Vec<(usize, RegisterValue)>>` - Read specific registers
- `read_all_registers(force_refresh: bool) -> Result<Vec<(usize, RegisterValue)>>` - Read all 184 registers
- `print_registers(&[usize], OutputFormat, force_refresh: bool) -> Result<()>` - Print register data

**Writing Data:**
- `write_message(message: &str) -> Result<()>` - Write up to 20-char ASCII message to battery

**Low-Level:**
- `send_custom_command(cmd: u8, addr_hi: u8, addr_lo: u8, len: u8) -> Result<Vec<u8>>` - Send custom command
- `idle()` - Set J2 pin to idle (low) state
- `high()` - Set J2 pin to high (~20V) state
- `high_for(duration: Duration)` - Pulse J2 high for specified time

### Key Types

**`HealthReport`** - Comprehensive battery diagnostics including:
- Battery type, serial number, manufacture date
- Cell voltages, pack voltage, cell imbalance
- Temperature
- Charging statistics (counts, times, low-voltage events)
- Usage statistics (discharge cycles, overheating, overcurrent events)
- Discharge current histogram

**`OutputFormat`** - Register output formatting:
- `Label` - Human-readable with labels
- `Raw` - Raw values for spreadsheet import
- `Array` - Rust array format
- `Form` - Google Forms submission format

**`RegisterValue`** - Parsed register value (enum):
- `UInt(u64)` - Unsigned integer
- `Float(f64)` - Floating point (temperature, etc.)
- `String(String)` - ASCII text
- `DateTime(DateTime<Utc>)` - Timestamp
- `Duration(String)` - HH:MM:SS format
- `CellVoltages([u16; 5])` - Five cell voltages in mV
- `SerialInfo { battery_type: u16, serial: u32 }` - Battery identification

### Error Handling

All operations return `Result<T, M18Error>` where `M18Error` includes:
- `SerialPort(serialport::Error)` - Serial communication errors
- `Io(std::io::Error)` - I/O errors
- `Timeout` - Communication timeout
- `EmptyResponse` - No data from battery
- `InvalidResponse` - Unexpected response format
- `MessageTooLong` - Message exceeds 20 characters
- `Parse(String)` - Data parsing error

## Supported Battery Types

The library recognizes the following M18 battery types by their internal type code:

| Type Code | Capacity | Description | Cell Configuration |
|-----------|----------|-------------|-------------------|
| 36 | 1.5Ah | CP (Compact) | 5s1p 18650 |
| 37 | 2Ah | CP (Compact) | 5s1p 18650 |
| 38 | 3Ah | XC (Extended Capacity) | 5s2p 18650 |
| 39 | 4Ah | XC (Extended Capacity) | 5s2p 18650 |
| 40 | 5Ah | XC (Extended Capacity) | 5s2p 18650 (≤Dec 2018) |
| 165 | 5Ah | XC (Extended Capacity) | 5s2p 18650 (Aug 2019 - Jun 2021) |
| 306 | 5Ah | XC (Extended Capacity) | 5s2p 18650 (Feb 2021 - Jul 2023) |
| 424 | 5Ah | XC (Extended Capacity) | 5s2p 18650 (≥Sep 2023) |
| 46 | 6Ah | XC (Extended Capacity) | 5s2p 18650 |
| 47 | 9Ah | HD (High Demand) | 5s3p 18650 |
| 104 | 3Ah | HO (High Output) | 5s1p 21700 |
| 150 | 5.5Ah | HO (High Output) | 5s2p 21700 (EU only) |
| 106 | 6Ah | HO (High Output) | 5s2p 21700 |
| 107 | 8Ah | HO (High Output) | 5s2p 21700 |
| 108 | 12Ah | HO (High Output) | 5s3p 21700 |
| 383 | 8Ah | Forge | 5s2p 21700 tabless |
| 384 | 12Ah | Forge | 5s3p 21700 tabless |

Note: The battery type code is read from register 2 (serial number register) and is different from the model number printed on the case.

## Register Map

The library defines 184 registers (IDs 0-183) covering:
- **0x0000-0x007B**: Basic info (cell type, serial, dates, notes)
- **0x4000-0x401F**: Real-time data (voltages, temperature)
- **0x6000-0x600C**: Forge-specific registers
- **0x9000-0x9150**: Historical data (usage stats, charge/discharge histograms)
- **0xA000-0xA005**: Additional data

See the source code in `src/data.rs` for complete register definitions.

## Troubleshooting

**"No serial ports found"**
- Ensure your USB-to-serial adapter is properly connected
- On Linux, you may need to add your user to the `dialout` group: `sudo usermod -a -G dialout $USER`
- On Windows, check Device Manager for the COM port number

**"Permission denied" (Linux)**
- Add user to dialout group: `sudo usermod -a -G dialout $USER` (logout required)
- Or temporarily: `sudo chmod 666 /dev/ttyUSB0`

**"Empty response" or "Timeout"**
- Check physical connections (TX, RX, GND)
- Verify battery has charge (needs power to communicate)
- Ensure isolation/level-shifting circuit is working
- Try increasing timeout in constants (TIMEOUT_MS in `src/constants.rs`)

**"Invalid response"**
- Check baud rate is 4800 with 2 stop bits
- Verify RX/TX aren't swapped
- Ensure proper bit-reversal in isolation circuit (if using optocouplers)

## License

This project is dual-licensed under:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

You may choose either license.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Disclaimer

**⚠️ IMPORTANT SAFETY NOTICE:**

This library is for **educational and research purposes only**. Use at your own risk.

- Improperly interfacing with battery packs can cause fires, explosions, or injury
- This is NOT an official Milwaukee tool or endorsed by Milwaukee Tool
- Modifying or reverse-engineering batteries may void warranties
- Ensure proper electrical isolation and current limiting in your hardware setup
- The authors assume NO responsibility for damage to batteries, equipment, or injury

**By using this library, you acknowledge these risks and agree to use it responsibly.**

## Acknowledgments

This Rust implementation is based on the original Python research by the M18 battery reverse-engineering community. Special thanks to all contributors who helped document the protocol.
