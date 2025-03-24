use serde::Serialize;
use serde_json::Value;

use crate::widgets::{Widget, WidgetError};
use crate::{
    config::{GREEN, NEUTRAL, RED, YELLOW_WARNING},
    utils::file::{read_file, read_first_line_in_file},
};
use std::io::{BufRead, BufReader, Error};

const BATTERY_PATH: &str = "/sys/class/power_supply";
const BATTERY_LOWER_THRESHOLD: f32 = 20.0;
const BATTERY_UPPER_THRESHOLD: f32 = 80.0;

#[derive(Serialize)]
pub struct Battery {
    // Name of the widget
    name: &'static str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'static str,
    #[serde(skip_serializing)]
    // Holds the error message if an error occured during widget update
    error: Option<String>,
    #[serde(skip_serializing)]
    // Device name of the power supply
    device_name: String,
}

impl Battery {
    pub fn new(device_name: String) -> Self {
        Self {
            name: "battery",
            full_text: None,
            color: NEUTRAL,
            error: None,
            device_name,
        }
    }

    // Returns a emoji String that should represent the current state
    // Charging ⚡
    // Battery is being used 🔋
    // Battery full ☻
    // State unknown ?
    fn get_battery_state(&self) -> Result<String, Error> {
        match read_first_line_in_file(&format!("{}/{}/status", BATTERY_PATH, self.device_name))?
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

        if let Some(battery_file) =
            read_file(&format!("{}/{}/uevent", BATTERY_PATH, self.device_name))
        {
            let reader = BufReader::new(battery_file);
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

impl Widget for Battery {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        self.error = None;
        let battery_state = self.get_battery_state();
        let battery_life = self.get_battery_life();
        // Very ugly, but match would not make this more beautiful
        if let Ok(battery_state) = battery_state {
            if let Ok(battery_life) = battery_life {
                self.full_text = Some(format!("{} BAT {:.2}%", battery_state, battery_life));
                // See https://github.com/rust-lang/rust/issues/41620#issuecomment-314345874
                self.color = match battery_life {
                    x if x <= BATTERY_LOWER_THRESHOLD => RED,
                    x if x >= BATTERY_UPPER_THRESHOLD => YELLOW_WARNING,
                    _ => GREEN,
                };
            } else {
                self.error = Some(battery_life.err().unwrap().to_string());
            }
        } else {
            self.error = Some(battery_state.err().unwrap().to_string());
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            log::error!(
                "Error occured when trying to get battery life.\n{}",
                error_msg
            );
        }

        Ok(serde_json::to_value(self)?)
    }
}
