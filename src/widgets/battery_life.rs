use crate::utils::file::{read_file, read_first_line_in_file};

use super::{Widget, WidgetError};
use std::io::{BufRead, BufReader, Error};

const BATTERY_STATUS_PATH: &str = "/sys/class/power_supply/BAT0/status";
const BATTERY_STATS_PATH: &str = "/sys/class/power_supply/BAT0/uevent";

pub struct Battery {}

impl Battery {
    pub fn new() -> Self {
        Battery {}
    }

    /// Returns a emoji String that should represent the current state
    /// Charging ⚡
    /// Battery is being used 🔋
    /// Battery full ☻
    /// State unknown ?
    fn get_battery_state(&self) -> Result<String, Error> {
        match read_first_line_in_file(BATTERY_STATUS_PATH)?
            .as_str()
            .trim()
        {
            "Unknown" => Ok(String::from("?")),
            "Charging" => Ok(String::from("⚡")),
            "Discharging" => Ok(String::from("🔋")),
            "Not charging" => Ok(String::from("🔋")),
            "Full" => Ok(String::from("☻")),
            &_ => panic!("Something horrible happened! Check battery state in /sys directory!"),
        }
    }

    fn get_battery_life(&self) -> Result<String, Error> {
        let mut power_full: f32 = 0.0;
        let mut power_now: f32 = 0.0;

        if let Some(memory_file) = read_file(BATTERY_STATS_PATH) {
            let reader = BufReader::new(memory_file);
            for line in reader.lines() {
                // TODO: Find a better solution for this
                // Apparently `?` consumes `line`,
                // which causes a move after being used once
                let unpacked_line = line?;

                // TODO: I don't like the unwraps here,
                // it should be impossble to panic here, because the file is managed
                // by the kernel and it should always look the same.
                if unpacked_line.starts_with("POWER_SUPPLY_ENERGY_FULL") {
                    power_full = unpacked_line
                        .split("=")
                        .last()
                        .unwrap()
                        .parse::<f32>()
                        .unwrap();
                } else if unpacked_line.starts_with("POWER_SUPPLY_ENERGY_NOW") {
                    power_now = unpacked_line
                        .split("=")
                        .last()
                        .unwrap()
                        .parse::<f32>()
                        .unwrap();
                }
            }
        }

        Ok(format!("{:.2}", (power_now / power_full) * 100.0))
    }
}

impl Widget for Battery {
    fn name(&self) -> &str {
        "battery"
    }

    fn display_text(&self) -> Result<String, WidgetError> {
        // Is it possible to beatify this?
        let state = match self.get_battery_state() {
            Ok(state) => Ok(state),
            Err(msg) => Err(WidgetError::new(msg.to_string())),
        };

        let life = match self.get_battery_life() {
            Ok(life) => Ok(life),
            Err(msg) => Err(WidgetError::new(msg.to_string())),
        };

        Ok(format!("{} BAT {}%", state?, life?))
    }
}
