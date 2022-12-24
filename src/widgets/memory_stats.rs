use std::io::BufRead;
use std::io::BufReader;
use std::num::ParseFloatError;

use serde::Serialize;
use serde_json::Value;

use crate::config::NEUTRAL;
use crate::config::RED;
use crate::utils::file::read_file;
use crate::widgets::Widget;
use crate::widgets::WidgetError;
use crate::LOGGER;

const MEMORY_THRESHOLD: f32 = 50.0;

#[derive(Debug)]
struct MemoryInfromation {
    // Current memory that is being used by the OS
    used: f32,
    // Current memory that is not being actively used (without swapping)
    available: f32,
    // Total memory that can be used
    // Should be ~ used + available
    total_usable: f32,
    // The next fields are only needed for calculating "used" memory
    mem_free: f32,
    // Kernel buffers
    buffers: f32,
    // In memory cache for files from disk
    cache: f32,
    // Probably reclaimable cache or something like that
    reclaimable: f32,
}

#[derive(Serialize)]
pub struct MemoryUsage<'a> {
    // Name of the widget
    name: &'a str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
    #[serde(skip_serializing)]
    // Holds the error message if an error occured during widget update
    error: Option<String>,
}

impl<'a> MemoryUsage<'a> {
    pub fn new() -> Self {
        MemoryUsage {
            name: "memory",
            full_text: None,
            color: NEUTRAL,
            error: None,
        }
    }

    fn get_int_from_str(&self, str_to_parse: String) -> Result<f32, ParseFloatError> {
        // This will be a string with only numbers, so we can convert
        // it to a f32
        let mut resulting_string = String::new();

        // Could be improved by removing whitespace before iteration
        for character in str_to_parse.chars() {
            if character.is_ascii_digit() {
                resulting_string.push(character);
            }
        }

        resulting_string.parse::<f32>()
    }

    fn get_usage(&self) -> Result<MemoryInfromation, ParseFloatError> {
        let mut memory_information = MemoryInfromation {
            used: 0.0,
            available: 0.0,
            total_usable: 0.0,
            mem_free: 0.0,
            buffers: 0.0,
            cache: 0.0,
            reclaimable: 0.0,
        };

        if let Some(memory_file) = read_file("/proc/meminfo") {
            let reader = BufReader::new(memory_file);
            for line in reader.lines().flatten() {
                if line.starts_with("MemTotal") {
                    // Convert to kb to gb
                    memory_information.total_usable = self.get_int_from_str(line)?;
                } else if line.starts_with("MemAvailable") {
                    memory_information.available = self.get_int_from_str(line)?;
                } else if line.starts_with("Buffers") {
                    memory_information.buffers = self.get_int_from_str(line)?;
                } else if line.starts_with("MemFree") {
                    memory_information.mem_free = self.get_int_from_str(line)?;
                } else if line.starts_with("Cached") {
                    memory_information.cache = self.get_int_from_str(line)?;
                } else if line.starts_with("SReclaimable") {
                    memory_information.reclaimable = self.get_int_from_str(line)?;
                }
            }

            memory_information.used = memory_information.total_usable
                - memory_information.mem_free
                - memory_information.buffers
                - memory_information.cache
                - memory_information.reclaimable
        }

        Ok(memory_information)
    }
}

impl<'a> Widget for MemoryUsage<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        match self.get_usage() {
            Ok(usage) => {
                self.color = if (usage.used / usage.total_usable * 100.0) > MEMORY_THRESHOLD {
                    RED
                } else {
                    NEUTRAL
                };
                self.full_text = Some(format!(
                    "RAM (GiB): U={used:.1} A={available:.1} / {total_usable:.1}",
                    used = usage.used / 1024.0 / 1024.0,
                    available = usage.available / 1024.0 / 1024.0,
                    total_usable = usage.total_usable / 1024.0 / 1024.0
                ));
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            LOGGER.error(&format!(
                "Error occured when calculating memory usage.\n{}",
                error_msg
            ));
        }

        Ok(serde_json::to_value(self)?)
    }
}
