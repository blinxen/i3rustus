use serde::Serialize;
use serde_json::Value;

use crate::{
    config::{RED, NEUTRAL},
    utils::file::{read_file, read_first_line_in_file},
    LOGGER,
};

use super::{Widget, WidgetError};
use std::io::{BufRead, BufReader, Error};

const BATTERY_STATUS_PATH: &str = "/sys/class/power_supply/BAT0/status";
const BATTERY_STATS_PATH: &str = "/sys/class/power_supply/BAT0/uevent";
const BATTERY_THRESHOLD: f32 = 20.0;

#[derive(Serialize)]
pub struct Battery<'a> {
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

impl<'a> Battery<'a> {
    pub fn new() -> Self {
        Battery {
            name: "battery",
            full_text: None,
            color: NEUTRAL,
            error: None,
        }
    }

    // Returns a emoji String that should represent the current state
    // Charging ⚡
    // Battery is being used 🔋
    // Battery full ☻
    // State unknown ?
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

    fn get_battery_life(&self) -> Result<f32, WidgetError> {
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
                        .split('=')
                        .last()
                        .unwrap()
                        .parse::<f32>()
                        .unwrap();
                } else if unpacked_line.starts_with("POWER_SUPPLY_ENERGY_NOW") {
                    power_now = unpacked_line
                        .split('=')
                        .last()
                        .unwrap()
                        .parse::<f32>()
                        .unwrap();
                }
            }
        }

        Ok(power_now / power_full * 100.0)
    }
}

impl<'a> Widget for Battery<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        let battery_state = self.get_battery_state();
        let battery_life = self.get_battery_life();
        // Very ugly, but match would not make this more beautiful
        if let Ok(battery_state) = battery_state {
            if let Ok(battery_life) = battery_life {
                self.full_text = Some(format!("{} BAT {:.2}%", battery_state, battery_life));
                if battery_life <= BATTERY_THRESHOLD {
                    self.color = RED;
                }
            } else {
                self.error = Some(battery_life.err().unwrap().to_string());
            }
        } else {
            self.error = Some(battery_state.err().unwrap().to_string());
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            LOGGER.error(&format!(
                "Error occured when trying to get battery life.\n{}",
                error_msg
            ));
        }

        Ok(serde_json::to_value(self)?)
    }
}
