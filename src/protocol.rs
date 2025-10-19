use crate::constants::*;
use crate::data::{create_data_id, DATA_MATRIX};
use crate::error::{M18Error, Result};
use crate::types::*;
use chrono::{DateTime, TimeZone, Utc};
use serialport::SerialPort;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::thread;
use std::time::{Duration, Instant};

/// Main M18 protocol interface
pub struct M18 {
    port: Box<dyn SerialPort>,
    acc: u8,
    print_tx: bool,
    print_rx: bool,
    register_defs: Vec<RegisterDef>,
    battery_lookup: HashMap<u16, BatteryType>,
}

impl M18 {
    /// Create a new M18 interface
    pub fn new(port_name: &str) -> Result<Self> {
        let port = serialport::new(port_name, BAUD_RATE)
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .stop_bits(STOP_BITS)
            .open()?;

        let mut m18 = M18 {
            port,
            acc: INITIAL_ACC,
            print_tx: false,
            print_rx: false,
            register_defs: create_data_id(),
            battery_lookup: create_battery_lookup(),
        };

        m18.idle();
        Ok(m18)
    }

    /// List available serial ports
    pub fn list_ports() -> Result<Vec<serialport::SerialPortInfo>> {
        Ok(serialport::available_ports()?)
    }

    /// Enable/disable debug printing for TX/RX
    pub fn set_debug_print(&mut self, tx: bool, rx: bool) {
        self.print_tx = tx;
        self.print_rx = rx;
    }

    /// Reset the connected battery
    pub fn reset(&mut self) -> Result<bool> {
        self.acc = INITIAL_ACC;
        
        // Toggle break condition and DTR for reset
        self.port.set_break()?;
        self.port.write_data_terminal_ready(true)?;
        thread::sleep(Duration::from_millis(RESET_BREAK_DURATION));
        
        self.port.clear_break()?;
        self.port.write_data_terminal_ready(false)?;
        thread::sleep(Duration::from_millis(RESET_SETTLE_DURATION));
        
        // Send sync byte
        self.send(&[SYNC_BYTE])?;
        
        match self.read_response(1) {
            Ok(response) if response.len() == 1 && response[0] == SYNC_BYTE => {
                thread::sleep(Duration::from_millis(RESET_SYNC_DELAY));
                Ok(true)
            }
            Ok(response) => {
                if self.print_rx {
                    println!("Unexpected response: {:02X?}", response);
                }
                Ok(false)
            }
            Err(_) => Ok(false),
        }
    }

    /// Update the ACC (accumulator) value for next command
    fn update_acc(&mut self) {
        let current_index = ACC_VALUES.iter().position(|&x| x == self.acc).unwrap_or(0);
        let next_index = (current_index + 1) % ACC_VALUES.len();
        self.acc = ACC_VALUES[next_index];
    }

    /// Reverse bits in a byte (for protocol bit ordering)
    fn reverse_bits(byte: u8) -> u8 {
        let mut result = 0u8;
        for i in 0..8 {
            if byte & (1 << i) != 0 {
                result |= 1 << (7 - i);
            }
        }
        result
    }

    /// Calculate checksum for payload
    fn checksum(payload: &[u8]) -> u16 {
        payload.iter().map(|&b| b as u16).sum()
    }

    /// Add checksum to command
    fn add_checksum(command: &[u8]) -> Vec<u8> {
        let mut result = command.to_vec();
        let checksum = Self::checksum(command);
        result.extend_from_slice(&checksum.to_be_bytes());
        result
    }

    /// Send raw bytes to the battery
    fn send(&mut self, command: &[u8]) -> Result<()> {
        self.port.clear(serialport::ClearBuffer::Input)?;
        
        if self.print_tx {
            let debug_print: String = command.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            println!("Sending:  {}", debug_print);
        }

        // Convert to MSB format (reverse bits)
        let msb_command: Vec<u8> = command.iter().map(|&b| Self::reverse_bits(b)).collect();
        self.port.write_all(&msb_command)?;
        Ok(())
    }

    /// Send command with checksum
    fn send_command(&mut self, command: &[u8]) -> Result<()> {
        let command_with_checksum = Self::add_checksum(command);
        self.send(&command_with_checksum)
    }

    /// Read response from battery
    fn read_response(&mut self, expected_size: usize) -> Result<Vec<u8>> {
        let mut msb_response = vec![0u8; 1];
        self.port.read_exact(&mut msb_response)?;
        
        if msb_response.is_empty() {
            return Err(M18Error::EmptyResponse);
        }

        // Check if we need to read more based on first byte
        let additional_bytes = if Self::reverse_bits(msb_response[0]) == 0x82 {
            1
        } else {
            expected_size - 1
        };

        if additional_bytes > 0 {
            let mut additional = vec![0u8; additional_bytes];
            self.port.read_exact(&mut additional)?;
            msb_response.extend(additional);
        }

        // Convert from MSB format (reverse bits)
        let lsb_response: Vec<u8> = msb_response.iter().map(|&b| Self::reverse_bits(b)).collect();
        
        if self.print_rx {
            let debug_print: String = lsb_response.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            println!("Received: {}", debug_print);
        }

        Ok(lsb_response)
    }

    /// Configure battery charging parameters
    pub fn configure(&mut self, state: u8) -> Result<Vec<u8>> {
        let command = [
            CONF_CMD,
            self.acc,
            8,
            (CUTOFF_CURRENT >> 8) as u8,
            (CUTOFF_CURRENT & 0xFF) as u8,
            (MAX_CURRENT >> 8) as u8,
            (MAX_CURRENT & 0xFF) as u8,
            (MAX_CURRENT >> 8) as u8,
            (MAX_CURRENT & 0xFF) as u8,
            state,
            13,
        ];
        self.send_command(&command)?;
        self.update_acc();
        self.read_response(5)
    }

    /// Get snapshot data from battery
    pub fn get_snapchat(&mut self) -> Result<Vec<u8>> {
        let command = [SNAP_CMD, self.acc, 0];
        self.send_command(&command)?;
        self.update_acc();
        self.read_response(8)
    }

    /// Send keepalive message
    pub fn keepalive(&mut self) -> Result<Vec<u8>> {
        let command = [KEEPALIVE_CMD, self.acc, 0];
        self.send_command(&command)?;
        self.read_response(9)
    }

    /// Send calibration command
    pub fn calibrate(&mut self) -> Result<Vec<u8>> {
        let command = [CAL_CMD, self.acc, 0];
        self.send_command(&command)?;
        self.update_acc();
        self.read_response(8)
    }

    /// Send custom command to battery
    pub fn send_custom_command(
        &mut self,
        command: u8,
        address_high: u8,
        address_low: u8,
        length: u8,
    ) -> Result<Vec<u8>> {
        let cmd = [command, 0x04, 0x03, address_high, address_low, length];
        self.send_command(&cmd)?;
        self.read_response((length + 5) as usize)
    }

    /// Simulate charger communication for specified duration
    pub fn simulate_for(&mut self, duration: Duration) -> Result<()> {
        println!("Simulating charger communication for {} seconds...", duration.as_secs());
        let start_time = Instant::now();

        self.reset()?;
        self.acc = INITIAL_ACC;  // Ensure ACC starts at initial value for configure sequence
        self.configure(2)?;
        self.get_snapchat()?;
        thread::sleep(Duration::from_millis(CONFIGURE_DELAY));
        self.keepalive()?;
        thread::sleep(Duration::from_millis(CONFIGURE_DELAY));  // Additional delay before second configure
        self.configure(1)?;
        self.get_snapchat()?;

        while start_time.elapsed() < duration {
            thread::sleep(Duration::from_millis(KEEPALIVE_INTERVAL));
            if let Err(e) = self.keepalive() {
                println!("Keepalive failed: {}", e);
                break;
            }
        }

        self.idle();
        println!("Duration: {:.2} seconds", start_time.elapsed().as_secs_f64());
        Ok(())
    }

    /// Set J2 pin to idle state (low)
    pub fn idle(&mut self) {
        let _ = self.port.set_break();
        let _ = self.port.write_data_terminal_ready(true);
    }

    /// Set J2 pin to high state
    pub fn high(&mut self) {
        let _ = self.port.clear_break();
        let _ = self.port.write_data_terminal_ready(false);
    }

    /// Set J2 pin high for specified duration, then return to idle
    pub fn high_for(&mut self, duration: Duration) {
        self.high();
        thread::sleep(duration);
        self.idle();
    }

    /// Calculate temperature from ADC reading
    fn calculate_temperature(&self, adc_value: u16) -> f64 {
        // Constants from original implementation
        const R1: f64 = 10e3; // 10k ohm
        const R2: f64 = 20e3; // 20k ohm
        const T1: f64 = 50.0;  // 50°C
        const T2: f64 = 35.0;  // 35°C
        const ADC1: f64 = 0x0180 as f64;
        const ADC2: f64 = 0x022E as f64;

        let m = (T2 - T1) / (R2 - R1);
        let b = T1 - m * R1;
        let resistance = R1 + (adc_value as f64 - ADC1) * (R2 - R1) / (ADC2 - ADC1);
        let temperature = m * resistance + b;
        
        (temperature * 100.0).round() / 100.0 // Round to 2 decimal places
    }

    /// Convert bytes to DateTime
    fn bytes_to_datetime(&self, bytes: &[u8]) -> Result<DateTime<Utc>> {
        if bytes.len() != 4 {
            return Err(M18Error::Parse("Invalid date bytes length".to_string()));
        }
        
        let epoch_time = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        
        Ok(Utc.timestamp_opt(epoch_time as i64, 0)
           .single()
           .ok_or_else(|| M18Error::Parse("Invalid timestamp".to_string()))?)
    }

    /// Format duration from seconds to HH:MM:SS
    fn format_duration(&self, seconds: u32) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }

    /// Write a message to battery memory
    pub fn write_message(&mut self, message: &str) -> Result<()> {
        if message.len() > 20 {
            return Err(M18Error::MessageTooLong { length: message.len() });
        }
        
        println!("Writing \"{}\" to memory", message);
        self.reset()?;
        
        let padded_message = format!("{:-<20}", message);
        for (i, byte) in padded_message.bytes().enumerate() {
            let command = [0x01, 0x05, 0x03, 0x00, (0x23 + i) as u8, byte];
            self.send_command(&command)?;
            let _response = self.read_response(2)?;
        }
        
        Ok(())
    }

    /// Read all memory regions and return raw data
    pub fn read_all_raw(&mut self) -> Result<Vec<(u16, Vec<u8>)>> {
        let mut results = Vec::new();
        self.reset()?;
        
        for region in DATA_MATRIX {
            let address = (region.address_high as u16) << 8 | region.address_low as u16;
            match self.send_custom_command(0x01, region.address_high, region.address_low, region.length) {
                Ok(response) if response.len() >= 4 && response[0] == 0x81 => {
                    let data = response[3..3 + region.length as usize].to_vec();
                    results.push((address, data));
                }
                Ok(response) => {
                    if self.print_rx {
                        let debug_print: String = response.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        println!("Invalid response from: 0x{:04X} Response: {}", address, debug_print);
                    }
                }
                Err(e) => {
                    if self.print_rx {
                        println!("Failed to read from 0x{:04X}: {}", address, e);
                    }
                }
            }
        }
        
        self.idle();
        Ok(results)
    }

    /// Parse raw data according to register definition
    fn parse_register_data(&self, register: &RegisterDef, data: &[u8]) -> Result<RegisterValue> {
        if data.len() != register.length as usize {
            return Err(M18Error::Parse(format!("Data length mismatch for register 0x{:04X}", register.address)));
        }

        match register.data_type {
            DataType::UInt => {
                let value = match data.len() {
                    1 => data[0] as u64,
                    2 => u16::from_be_bytes([data[0], data[1]]) as u64,
                    4 => u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as u64,
                    8 => u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]),
                    _ => return Err(M18Error::Parse("Invalid uint length".to_string())),
                };
                Ok(RegisterValue::UInt(value))
            }
            DataType::Date => {
                let dt = self.bytes_to_datetime(data)?;
                Ok(RegisterValue::DateTime(dt))
            }
            DataType::Duration => {
                let seconds = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                let formatted = self.format_duration(seconds);
                Ok(RegisterValue::Duration(formatted))
            }
            DataType::Ascii => {
                let s = String::from_utf8_lossy(data).to_string();
                Ok(RegisterValue::String(format!("\"{}\"", s)))
            }
            DataType::SerialNumber => {
                if data.len() != 5 {
                    return Err(M18Error::Parse("Invalid serial number length".to_string()));
                }
                let battery_type = u16::from_be_bytes([data[0], data[1]]);
                let serial = u32::from_be_bytes([0, data[2], data[3], data[4]]);
                Ok(RegisterValue::SerialInfo { battery_type, serial })
            }
            DataType::AdcTemperature => {
                let adc_value = u16::from_be_bytes([data[0], data[1]]);
                let temp = self.calculate_temperature(adc_value);
                Ok(RegisterValue::Float(temp))
            }
            DataType::DecimalTemperature => {
                let temp = data[0] as f64 + (data[1] as f64) / 256.0;
                Ok(RegisterValue::Float((temp * 100.0).round() / 100.0))
            }
            DataType::CellVoltages => {
                if data.len() != 10 {
                    return Err(M18Error::Parse("Invalid cell voltages length".to_string()));
                }
                let mut voltages = [0u16; 5];
                for i in 0..5 {
                    voltages[i] = u16::from_be_bytes([data[i * 2], data[i * 2 + 1]]);
                }
                Ok(RegisterValue::CellVoltages(voltages))
            }
        }
    }

    /// Read specific registers by ID
    pub fn read_registers(&mut self, register_ids: &[usize], force_refresh: bool) -> Result<Vec<(usize, RegisterValue)>> {
        let mut results = Vec::new();
        
        if force_refresh {
            // Read all regions to refresh data
            for region in DATA_MATRIX {
                let _ = self.send_custom_command(0x01, region.address_high, region.address_low, region.length);
            }
            self.idle();
            std::thread::sleep(Duration::from_millis(100));
        }

        self.reset()?;
        
        for &id in register_ids {
            if id >= self.register_defs.len() {
                continue;
            }
            
            let register = self.register_defs[id].clone();
            let address_high = ((register.address >> 8) & 0xFF) as u8;
            let address_low = (register.address & 0xFF) as u8;
            
            match self.send_custom_command(0x01, address_high, address_low, register.length) {
                Ok(response) if response.len() >= 4 && response[0] == 0x81 => {
                    let data = &response[3..3 + register.length as usize];
                    match self.parse_register_data(&register, data) {
                        Ok(value) => results.push((id, value)),
                        Err(e) => {
                            if self.print_rx {
                                println!("Failed to parse register {}: {}", id, e);
                            }
                        }
                    }
                }
                Ok(_) | Err(_) => {
                    // Skip invalid responses
                }
            }
        }
        
        self.idle();
        Ok(results)
    }

    /// Read all registers
    pub fn read_all_registers(&mut self, force_refresh: bool) -> Result<Vec<(usize, RegisterValue)>> {
        let ids: Vec<usize> = (0..self.register_defs.len()).collect();
        self.read_registers(&ids, force_refresh)
    }

    /// Print register data in various formats
    pub fn print_registers(&mut self, register_ids: &[usize], format: OutputFormat, force_refresh: bool) -> Result<()> {
        let ids = if register_ids.is_empty() {
            (0..self.register_defs.len()).collect()
        } else {
            register_ids.to_vec()
        };
        
        let results = self.read_registers(&ids, force_refresh)?;
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        match format {
            OutputFormat::Label => {
                println!("{}", timestamp);
                println!("ID  ADDR   LEN TYPE       LABEL                                   VALUE");
                for (id, value) in results {
                    let register = &self.register_defs[id];
                    let type_str = format!("{:?}", register.data_type);
                    let value_str = self.format_register_value(&value, format);
                    println!("{:3} 0x{:04X} {:2} {:>6}   {:<39} {:<}", 
                             id, register.address, register.length, type_str, register.label, value_str);
                }
            }
            OutputFormat::Raw => {
                println!("{}", timestamp);
                for (_, value) in results {
                    println!("{}", self.format_register_value(&value, format));
                }
            }
            OutputFormat::Array => {
                println!("Results as array: {:?}", results);
            }
            OutputFormat::Form => {
                println!("{}", timestamp);
                for (_, value) in results {
                    println!("{}", self.format_register_value(&value, format));
                }
            }
        }
        
        Ok(())
    }

    /// Format register value for display
    fn format_register_value(&self, value: &RegisterValue, format: OutputFormat) -> String {
        match (value, format) {
            (RegisterValue::UInt(v), _) => v.to_string(),
            (RegisterValue::Float(v), _) => format!("{:.2}", v),
            (RegisterValue::String(s), _) => s.clone(),
            (RegisterValue::DateTime(dt), _) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            (RegisterValue::Duration(d), _) => d.clone(),
            (RegisterValue::CellVoltages(voltages), OutputFormat::Label) => {
                format!("1: {:4}, 2: {:4}, 3: {:4}, 4: {:4}, 5: {:4}", 
                        voltages[0], voltages[1], voltages[2], voltages[3], voltages[4])
            }
            (RegisterValue::CellVoltages(voltages), OutputFormat::Raw) => {
                format!("{:4}\n{:4}\n{:4}\n{:4}\n{:4}", 
                        voltages[0], voltages[1], voltages[2], voltages[3], voltages[4])
            }
            (RegisterValue::CellVoltages(voltages), _) => {
                format!("{:?}", voltages)
            }
            (RegisterValue::SerialInfo { battery_type, serial }, OutputFormat::Raw) => {
                format!("{}\n{}", battery_type, serial)
            }
            (RegisterValue::SerialInfo { battery_type, serial }, _) => {
                format!("Type: {:3}, Serial: {}", battery_type, serial)
            }
        }
    }

    /// Generate a comprehensive health report
    pub fn health_report(&mut self) -> Result<HealthReport> {
        println!("Reading battery. This will take 5-10sec");
        
        // Define the register IDs needed for health report
        let reg_list = vec![
            4,  // Manufacture date
            28, // Days since first charge
            25, // Days since last tool use (corrected for current time)
            26, // Days since last charge (corrected for current time)
            12, // Voltages and imbalance
            13, // temp (non-forge)
            18, // temp (forge)
            29, // Total discharge (Ah)
            39, // Discharged to empty (count)
            40, // Overheat events
            41, // Overcurrent events
            42, // Low-voltage events
            43, // Low-voltage bounce
            33, 32, 31, // Redlink, dumb, total charge count
            35, // Total charge time
            36, // Time idling on charger
            38, // Low-voltage charges (any cell <2.5V)
            8,  // System date
            2,  // type & serial
        ];
        
        // Add discharge histogram registers (44-63 for 10-20A through 200A+)
        let mut full_reg_list = reg_list;
        full_reg_list.extend(44..=63);
        
        let results = self.read_registers(&full_reg_list, true)?;
        let values: HashMap<usize, RegisterValue> = results.into_iter().collect();
        
        // Extract battery info
        let (battery_type, electronic_serial) = if let Some(RegisterValue::SerialInfo { battery_type, serial }) = values.get(&2) {
            (*battery_type, *serial)
        } else {
            return Err(M18Error::Parse("Could not read battery serial info".to_string()));
        };
        
        let battery_info = self.battery_lookup.get(&battery_type)
            .cloned()
            .unwrap_or_else(|| BatteryType {
                capacity_ah: 0,
                description: "Unknown".to_string(),
            });
        
        // Extract dates
        let manufacture_date = if let Some(RegisterValue::DateTime(dt)) = values.get(&4) {
            *dt
        } else {
            return Err(M18Error::Parse("Could not read manufacture date".to_string()));
        };
        
        let system_date = if let Some(RegisterValue::DateTime(dt)) = values.get(&8) {
            *dt
        } else {
            Utc::now()
        };
        
        let last_tool_use = if let Some(RegisterValue::DateTime(dt)) = values.get(&25) {
            *dt
        } else {
            system_date
        };
        
        let last_charge = if let Some(RegisterValue::DateTime(dt)) = values.get(&26) {
            *dt
        } else {
            system_date
        };
        
        // Extract cell voltages
        let cell_voltages = if let Some(RegisterValue::CellVoltages(voltages)) = values.get(&12) {
            *voltages
        } else {
            return Err(M18Error::Parse("Could not read cell voltages".to_string()));
        };
        
        let pack_voltage = cell_voltages.iter().sum::<u16>() as f64 / 1000.0;
        let cell_imbalance = *cell_voltages.iter().max().unwrap() - *cell_voltages.iter().min().unwrap();
        
        // Extract temperature
        let temperature = values.get(&13)
            .or_else(|| values.get(&18))
            .and_then(|v| match v {
                RegisterValue::Float(temp) => Some(*temp),
                _ => None,
            });
        
        // Extract charging stats
        let get_uint = |id: usize| -> u16 {
            values.get(&id)
                .and_then(|v| match v {
                    RegisterValue::UInt(val) => Some(*val as u16),
                    _ => None,
                })
                .unwrap_or(0)
        };
        
        let get_duration = |id: usize| -> String {
            values.get(&id)
                .and_then(|v| match v {
                    RegisterValue::Duration(dur) => Some(dur.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "00:00:00".to_string())
        };
        
        let charging_stats = ChargingStats {
            redlink_charge_count: get_uint(33),
            dumb_charge_count: get_uint(32),
            total_charge_count: get_uint(31),
            total_charge_time: get_duration(35),
            time_idling_on_charger: get_duration(36),
            low_voltage_charges: get_uint(38),
        };
        
        // Extract usage stats
        let total_discharge_amp_sec = values.get(&29)
            .and_then(|v| match v {
                RegisterValue::UInt(val) => Some(*val),
                _ => None,
            })
            .unwrap_or(0) as f64;
        
        let total_discharge_ah = total_discharge_amp_sec / 3600.0;
        let total_discharge_cycles = if battery_info.capacity_ah > 0 {
            total_discharge_ah / (battery_info.capacity_ah as f64)
        } else {
            0.0
        };
        
        let usage_stats = UsageStats {
            total_discharge_ah,
            total_discharge_cycles,
            times_discharged_to_empty: get_uint(39),
            times_overheated: get_uint(40),
            overcurrent_events: get_uint(41),
            low_voltage_events: get_uint(42),
            low_voltage_bounce: get_uint(43),
            total_time_on_tool: "calculating...".to_string(), // Will be calculated below
        };
        
        // Build discharge histogram
        let mut discharge_histogram = Vec::new();
        let mut total_tool_time = 0u32;
        
        for i in 44..=63 {
            let time_seconds = get_uint(i) as u32;
            total_tool_time += time_seconds;
            
            let current_range = match i - 44 {
                0..=18 => format!("{}-{}A", (i - 44 + 1) * 10, (i - 44 + 2) * 10),
                19 => "> 200A".to_string(),
                _ => continue,
            };
            
            let duration = self.format_duration(time_seconds);
            let percentage = if total_tool_time > 0 {
                ((time_seconds as f64 / total_tool_time as f64) * 100.0).round() as u8
            } else {
                0
            };
            
            discharge_histogram.push(DischargeHistogramEntry {
                current_range,
                duration,
                percentage,
            });
        }
        
        // Update total time on tool in usage stats
        let mut usage_stats = usage_stats;
        usage_stats.total_time_on_tool = self.format_duration(total_tool_time);
        
        // Calculate percentage for histogram entries
        for entry in &mut discharge_histogram {
            let time_seconds: u32 = entry.duration.split(':')
                .map(|s| s.parse::<u32>().unwrap_or(0))
                .fold(0, |acc, x| acc * 60 + x);
            
            entry.percentage = if total_tool_time > 0 {
                ((time_seconds as f64 / total_tool_time as f64) * 100.0).round() as u8
            } else {
                0
            };
        }
        
        Ok(HealthReport {
            timestamp: Utc::now(),
            battery_type,
            battery_description: battery_info.description,
            electronic_serial,
            manufacture_date,
            days_since_first_charge: get_uint(28),
            days_since_last_tool_use: (system_date - last_tool_use).num_days(),
            days_since_last_charge: (system_date - last_charge).num_days(),
            pack_voltage,
            cell_voltages,
            cell_imbalance,
            temperature,
            charging_stats,
            usage_stats,
            discharge_histogram,
        })
    }

    /// Print a formatted health report
    pub fn print_health_report(&mut self) -> Result<()> {
        let report = self.health_report()?;
        
        println!("Type: {} [{}]", report.battery_type, report.battery_description);
        println!("E-serial: {} (does NOT match case serial)", report.electronic_serial);
        println!();
        println!("Manufacture date: {}", report.manufacture_date.format("%Y-%m-%d"));
        println!("Days since 1st charge: {}", report.days_since_first_charge);
        println!("Days since last tool use: {}", report.days_since_last_tool_use);
        println!("Days since last charge: {}", report.days_since_last_charge);
        println!("Pack voltage: {:.2}V", report.pack_voltage);
        println!("Cell Voltages (mV): {:?}", report.cell_voltages);
        println!("Cell Imbalance (mV): {}", report.cell_imbalance);
        
        if let Some(temp) = report.temperature {
            println!("Temperature (deg C): {:.2}", temp);
        }
        
        println!("\nCHARGING STATS:");
        println!("Charge count [Redlink, dumb, (total)]: {}, {}, ({})",
                 report.charging_stats.redlink_charge_count,
                 report.charging_stats.dumb_charge_count,
                 report.charging_stats.total_charge_count);
        println!("Total charge time: {}", report.charging_stats.total_charge_time);
        println!("Time idling on charger: {}", report.charging_stats.time_idling_on_charger);
        println!("Low-voltage charges (any cell <2.5V): {}", report.charging_stats.low_voltage_charges);
        
        println!("\nTOOL USE STATS:");
        println!("Total discharge (Ah): {:.2}", report.usage_stats.total_discharge_ah);
        println!("Total discharge cycles: {:.2}", report.usage_stats.total_discharge_cycles);
        println!("Times discharged to empty: {}", report.usage_stats.times_discharged_to_empty);
        println!("Times overheated: {}", report.usage_stats.times_overheated);
        println!("Overcurrent events: {}", report.usage_stats.overcurrent_events);
        println!("Low-voltage events: {}", report.usage_stats.low_voltage_events);
        println!("Low-voltage bounce/stutter: {}", report.usage_stats.low_voltage_bounce);
        println!("Total time on tool (>10A): {}", report.usage_stats.total_time_on_tool);
        
        println!("\nDISCHARGE HISTOGRAM:");
        for entry in &report.discharge_histogram {
            let bar = "X".repeat(entry.percentage as usize);
            println!("Time @ {:>8}: {} {:2}% {}", 
                     entry.current_range, entry.duration, entry.percentage, bar);
        }
        
        Ok(())
    }
}