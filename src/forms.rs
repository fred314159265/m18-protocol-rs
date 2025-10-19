#[cfg(feature = "form-submission")]
use crate::error::{M18Error, Result};
use crate::protocol::M18;
use crate::types::{FormData, OutputFormat};
use reqwest;
use std::collections::HashMap;
use std::io::{self, Write};

#[cfg(feature = "form-submission")]
impl M18 {
    /// Submit battery data to Google Forms
    pub async fn submit_form(&mut self) -> Result<()> {
        const FORM_URL: &str = "https://docs.google.com/forms/d/e/1FAIpQLScvTbSDYBzSQ8S4XoF-rfgwNj97C-Pn4Px3GIixJxf0C1YJJA/formResponse";
        
        println!("Getting data from battery...");
        
        // Get diagnostic data in form format
        let register_data = self.read_all_registers(true)?;
        let mut diagnostic_output = String::new();
        
        // Add timestamp
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        diagnostic_output.push_str(&timestamp);
        diagnostic_output.push('\n');
        
        // Add register values
        for (_, value) in register_data {
            diagnostic_output.push_str(&self.format_register_value(&value, OutputFormat::Form));
            diagnostic_output.push('\n');
        }
        
        // Prompt user for manual inputs
        println!("Please provide this information. All the values can be found on the label under the battery.");
        
        let one_key_id = prompt_input("Enter One-Key ID (example: H18FDCAD): ")?;
        let date = prompt_input("Enter Date (example: 190316): ")?;
        let serial_number = prompt_input("Enter Serial number (example: 0807426): ")?;
        let sticker = prompt_input("Enter Sticker (example: 4932 4512 45): ")?;
        let battery_type = prompt_input("Enter Type (example: M18B9): ")?;
        let capacity = prompt_input("Enter Capacity (example: 9.0Ah): ")?;
        
        let form_data = FormData {
            one_key_id,
            date,
            serial_number,
            sticker,
            battery_type,
            capacity,
            diagnostic_output,
        };
        
        // Create form data for submission
        let mut params = HashMap::new();
        params.insert("entry.905246449", form_data.one_key_id);
        params.insert("entry.453401884", form_data.date);
        params.insert("entry.2131879277", form_data.serial_number);
        params.insert("entry.337435885", form_data.sticker);
        params.insert("entry.1496274605", form_data.battery_type);
        params.insert("entry.324224550", form_data.capacity);
        params.insert("entry.716337020", form_data.diagnostic_output);
        
        // Submit the form
        let client = reqwest::Client::new();
        let response = client
            .post(FORM_URL)
            .form(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("Form submitted successfully!");
        } else {
            return Err(M18Error::Parse(format!(
                "Failed to submit form. Status code: {}", 
                response.status()
            )));
        }
        
        Ok(())
    }
}

/// Helper function to prompt for user input
fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(M18Error::Io)?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(M18Error::Io)?;
    
    Ok(input.trim().to_string())
}