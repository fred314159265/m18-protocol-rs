//! Type definitions for M18 battery protocol.
//!
//! This module contains all the data structures used for representing battery data,
//! including register definitions, health reports, and various data types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data types for register interpretation.
///
/// Each battery register stores data in a specific format. This enum defines
/// how to interpret the raw bytes from each register type.
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    /// Unsigned integer (1, 2, 4, or 8 bytes)
    UInt,
    /// Unix timestamp (4 bytes)
    Date,
    /// ASCII text string
    Ascii,
    /// Serial number format (2 bytes type + 3 bytes serial)
    SerialNumber,
    /// ADC temperature reading from thermistor (2 bytes)
    AdcTemperature,
    /// Decimal temperature format (1 byte + fractional byte)
    DecimalTemperature,
    /// Five cell voltages (10 bytes, 2 per cell)
    CellVoltages,
    /// Duration in HH:MM:SS format (4 bytes as seconds)
    Duration,
}

impl DataType {
    /// Parse a DataType from string representation.
    ///
    /// # Arguments
    /// * `s` - String identifier ("uint", "date", "ascii", etc.)
    ///
    /// # Returns
    /// The corresponding DataType variant, or an error if not recognized.
    pub fn from_str(s: &str) -> Result<Self, crate::M18Error> {
        match s {
            "uint" => Ok(DataType::UInt),
            "date" => Ok(DataType::Date),
            "ascii" => Ok(DataType::Ascii),
            "sn" => Ok(DataType::SerialNumber),
            "adc_t" => Ok(DataType::AdcTemperature),
            "dec_t" => Ok(DataType::DecimalTemperature),
            "cell_v" => Ok(DataType::CellVoltages),
            "hhmmss" => Ok(DataType::Duration),
            _ => Err(crate::M18Error::InvalidDataType(s.to_string())),
        }
    }
}

/// Memory region definition for bulk reads.
///
/// Represents a contiguous block of memory in the battery that can be read
/// in a single command.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// High byte of address
    pub address_high: u8,
    /// Low byte of address
    pub address_low: u8,
    /// Number of bytes to read
    pub length: u8,
}

/// Register definition with metadata.
///
/// Describes how to read and interpret a specific battery register.
#[derive(Debug, Clone)]
pub struct RegisterDef {
    /// 16-bit register address
    pub address: u16,
    /// Number of bytes in this register
    pub length: u8,
    /// How to interpret the raw bytes
    pub data_type: DataType,
    /// Human-readable description
    pub label: String,
}

/// Parsed register value.
///
/// Represents a battery register value after parsing from raw bytes.
/// Can be serialized to JSON for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegisterValue {
    /// Unsigned integer value
    UInt(u64),
    /// Floating-point value (temperature, voltage, etc.)
    Float(f64),
    /// String value (ASCII text, formatted durations)
    String(String),
    /// Timestamp value
    DateTime(DateTime<Utc>),
    /// Duration in HH:MM:SS format
    Duration(String),
    /// Array of five cell voltages in millivolts
    CellVoltages([u16; 5]),
    /// Battery serial number information
    SerialInfo {
        /// Battery type code (identifies model/capacity)
        battery_type: u16,
        /// Electronic serial number
        serial: u32
    },
}

/// Comprehensive battery health report.
///
/// Contains all diagnostic information about battery health, usage history,
/// and current state. Can be serialized to JSON for storage or analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// When this report was generated
    pub timestamp: DateTime<Utc>,
    /// Battery type code (identifies model)
    pub battery_type: u16,
    /// Human-readable battery description
    pub battery_description: String,
    /// Electronic serial number (not the same as case serial)
    pub electronic_serial: u32,
    /// When battery was manufactured
    pub manufacture_date: DateTime<Utc>,
    /// Days since battery was first charged
    pub days_since_first_charge: u16,
    /// Days since battery was last used in a tool
    pub days_since_last_tool_use: i64,
    /// Days since battery was last charged
    pub days_since_last_charge: i64,
    /// Total pack voltage in volts
    pub pack_voltage: f64,
    /// Individual cell voltages in millivolts
    pub cell_voltages: [u16; 5],
    /// Voltage difference between highest and lowest cell (mV)
    pub cell_imbalance: u16,
    /// Current temperature in Celsius (if available)
    pub temperature: Option<f64>,
    /// Charging-related statistics
    pub charging_stats: ChargingStats,
    /// Tool usage statistics
    pub usage_stats: UsageStats,
    /// Histogram of discharge current over battery lifetime
    pub discharge_histogram: Vec<DischargeHistogramEntry>,
}

/// Battery charging statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingStats {
    /// Number of charges using Redlink (UART) protocol
    pub redlink_charge_count: u16,
    /// Number of "dumb" charges (voltage-based only)
    pub dumb_charge_count: u16,
    /// Total number of charge cycles
    pub total_charge_count: u16,
    /// Total time spent charging (HH:MM:SS)
    pub total_charge_time: String,
    /// Time spent on charger after reaching full charge (HH:MM:SS)
    pub time_idling_on_charger: String,
    /// Number of times charged when any cell was below 2.5V
    pub low_voltage_charges: u16,
}

/// Battery usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total amp-hours discharged over battery lifetime
    pub total_discharge_ah: f64,
    /// Equivalent full discharge cycles (total_discharge_ah / capacity)
    pub total_discharge_cycles: f64,
    /// Number of times battery was completely drained
    pub times_discharged_to_empty: u16,
    /// Number of overheat events during tool use
    pub times_overheated: u16,
    /// Number of overcurrent protection events
    pub overcurrent_events: u16,
    /// Number of low-voltage protection events
    pub low_voltage_events: u16,
    /// Number of low-voltage "bounce" events (4 flashing LEDs)
    pub low_voltage_bounce: u16,
    /// Total time on tool drawing >10A (HH:MM:SS)
    pub total_time_on_tool: String,
}

/// Single entry in discharge current histogram.
///
/// The battery tracks how much time it spent discharging at different
/// current levels, creating a histogram of usage patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DischargeHistogramEntry {
    /// Current range (e.g., "10-20A", "> 200A")
    pub current_range: String,
    /// Time spent in this range (HH:MM:SS)
    pub duration: String,
    /// Percentage of total tool use time
    pub percentage: u8,
}

/// Battery type information.
///
/// Maps battery type codes to human-readable descriptions and capacities.
#[derive(Debug, Clone)]
pub struct BatteryType {
    /// Nominal capacity in amp-hours
    pub capacity_ah: u8,
    /// Full description including chemistry and form factor
    pub description: String,
}

/// Output format for printing register data.
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Human-readable labeled format
    Label,
    /// Raw values suitable for spreadsheet import
    Raw,
    /// Rust array format
    Array,
    /// Form submission format
    Form,
}

/// Form submission data for Google Forms integration.
///
/// Contains both manually-entered battery label information and
/// automatically-gathered diagnostic data.
#[cfg(feature = "form-submission")]
#[derive(Debug, Clone, Serialize)]
pub struct FormData {
    /// One-Key ID from battery label
    pub one_key_id: String,
    /// Manufacturing date from battery label
    pub date: String,
    /// Serial number from battery label (different from electronic serial)
    pub serial_number: String,
    /// Sticker number from battery label
    pub sticker: String,
    /// Battery model type (e.g., "M18B9")
    pub battery_type: String,
    /// Rated capacity (e.g., "9.0Ah")
    pub capacity: String,
    /// Raw diagnostic output from battery
    pub diagnostic_output: String,
}

/// Create battery type lookup map.
///
/// Returns a HashMap mapping battery type codes (from the serial number register)
/// to detailed battery information including capacity and description.
///
/// # Examples
/// ```
/// use m18_protocol::types::create_battery_lookup;
///
/// let lookup = create_battery_lookup();
/// if let Some(info) = lookup.get(&47) {
///     assert_eq!(info.capacity_ah, 9);
///     assert!(info.description.contains("9Ah HD"));
/// }
/// ```
pub fn create_battery_lookup() -> HashMap<u16, BatteryType> {
    let mut lookup = HashMap::new();

    lookup.insert(36, BatteryType {
        capacity_ah: 1,
        description: "1.5Ah CP (5s1p 18650)".to_string(),
    });

    lookup.insert(37, BatteryType {
        capacity_ah: 2,
        description: "2Ah CP (5s1p 18650)".to_string(),
    });

    lookup.insert(38, BatteryType {
        capacity_ah: 3,
        description: "3Ah XC (5s2p 18650)".to_string(),
    });

    lookup.insert(39, BatteryType {
        capacity_ah: 4,
        description: "4Ah XC (5s2p 18650)".to_string(),
    });

    lookup.insert(40, BatteryType {
        capacity_ah: 5,
        description: "5Ah XC (5s2p 18650) (<= Dec 2018)".to_string(),
    });

    lookup.insert(165, BatteryType {
        capacity_ah: 5,
        description: "5Ah XC (5s2p 18650) (Aug 2019 - Jun 2021)".to_string(),
    });

    lookup.insert(306, BatteryType {
        capacity_ah: 5,
        description: "5Ah XC (5s2p 18650) (Feb 2021 - Jul 2023)".to_string(),
    });

    lookup.insert(424, BatteryType {
        capacity_ah: 5,
        description: "5Ah XC (5s2p 18650) (>= Sep 2023)".to_string(),
    });

    lookup.insert(46, BatteryType {
        capacity_ah: 6,
        description: "6Ah XC (5s2p 18650)".to_string(),
    });

    lookup.insert(47, BatteryType {
        capacity_ah: 9,
        description: "9Ah HD (5s3p 18650)".to_string(),
    });

    lookup.insert(104, BatteryType {
        capacity_ah: 3,
        description: "3Ah HO (5s1p 21700)".to_string(),
    });

    lookup.insert(150, BatteryType {
        capacity_ah: 6,
        description: "5.5Ah HO (5s2p 21700) (EU only)".to_string(),
    });

    lookup.insert(106, BatteryType {
        capacity_ah: 6,
        description: "6Ah HO (5s2p 21700)".to_string(),
    });

    lookup.insert(107, BatteryType {
        capacity_ah: 8,
        description: "8Ah HO (5s2p 21700)".to_string(),
    });

    lookup.insert(108, BatteryType {
        capacity_ah: 12,
        description: "12Ah HO (5s3p 21700)".to_string(),
    });

    lookup.insert(383, BatteryType {
        capacity_ah: 8,
        description: "8Ah Forge (5s2p 21700 tabless)".to_string(),
    });

    lookup.insert(384, BatteryType {
        capacity_ah: 12,
        description: "12Ah Forge (5s3p 21700 tabless)".to_string(),
    });

    lookup
}