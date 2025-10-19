use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data types for register interpretation
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    UInt,
    Date,
    Ascii,
    SerialNumber,
    AdcTemperature,
    DecimalTemperature,
    CellVoltages,
    Duration,
}

impl DataType {
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

/// Memory region definition
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub address_high: u8,
    pub address_low: u8,
    pub length: u8,
}

/// Register definition
#[derive(Debug, Clone)]
pub struct RegisterDef {
    pub address: u16,
    pub length: u8,
    pub data_type: DataType,
    pub label: String,
}

/// Parsed register value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegisterValue {
    UInt(u64),
    Float(f64),
    String(String),
    DateTime(DateTime<Utc>),
    Duration(String),
    CellVoltages([u16; 5]),
    SerialInfo { battery_type: u16, serial: u32 },
}

/// Battery health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub battery_type: u16,
    pub battery_description: String,
    pub electronic_serial: u32,
    pub manufacture_date: DateTime<Utc>,
    pub days_since_first_charge: u16,
    pub days_since_last_tool_use: i64,
    pub days_since_last_charge: i64,
    pub pack_voltage: f64,
    pub cell_voltages: [u16; 5],
    pub cell_imbalance: u16,
    pub temperature: Option<f64>,
    pub charging_stats: ChargingStats,
    pub usage_stats: UsageStats,
    pub discharge_histogram: Vec<DischargeHistogramEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingStats {
    pub redlink_charge_count: u16,
    pub dumb_charge_count: u16,
    pub total_charge_count: u16,
    pub total_charge_time: String,
    pub time_idling_on_charger: String,
    pub low_voltage_charges: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_discharge_ah: f64,
    pub total_discharge_cycles: f64,
    pub times_discharged_to_empty: u16,
    pub times_overheated: u16,
    pub overcurrent_events: u16,
    pub low_voltage_events: u16,
    pub low_voltage_bounce: u16,
    pub total_time_on_tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DischargeHistogramEntry {
    pub current_range: String,
    pub duration: String,
    pub percentage: u8,
}

/// Battery type lookup information
#[derive(Debug, Clone)]
pub struct BatteryType {
    pub capacity_ah: u8,
    pub description: String,
}

/// Output format options
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Label,
    Raw,
    Array,
    Form,
}

/// Form submission data
#[cfg(feature = "form-submission")]
#[derive(Debug, Clone, Serialize)]
pub struct FormData {
    pub one_key_id: String,
    pub date: String,
    pub serial_number: String,
    pub sticker: String,
    pub battery_type: String,
    pub capacity: String,
    pub diagnostic_output: String,
}

/// Create battery type lookup map
pub fn create_battery_lookup() -> HashMap<u16, BatteryType> {
    let mut lookup = HashMap::new();
    
    lookup.insert(37, BatteryType {
        capacity_ah: 2,
        description: "2Ah CP (5s1p 18650)".to_string(),
    });
    
    lookup.insert(40, BatteryType {
        capacity_ah: 5,
        description: "5Ah XC (5s2p 18650)".to_string(),
    });
    
    lookup.insert(165, BatteryType {
        capacity_ah: 5,
        description: "5Ah XC (5s2p 18650)".to_string(),
    });
    
    lookup.insert(46, BatteryType {
        capacity_ah: 6,
        description: "6Ah XC (5s2p 18650)".to_string(),
    });
    
    lookup.insert(104, BatteryType {
        capacity_ah: 3,
        description: "3Ah HO (5s1p 21700)".to_string(),
    });
    
    lookup.insert(106, BatteryType {
        capacity_ah: 4,
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
    
    lookup.insert(384, BatteryType {
        capacity_ah: 12,
        description: "12Ah Forge (5s3p 21700 tabless)".to_string(),
    });
    
    lookup
}