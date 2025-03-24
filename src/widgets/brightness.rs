use crate::widgets::{Widget, WidgetError};
use crate::{config::YELLOW, utils::file::read_first_line_in_file};
use serde::Serialize;
use serde_json::Value;

const BACKLIGHT_PATH: &str = "/sys/class/backlight";

#[derive(Serialize)]
pub struct Brightness {
    // Name of the widget
    name: &'static str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'static str,
    #[serde(skip_serializing)]
    // Device name of the digital display
    device_name: String,
}

impl Brightness {
    pub fn new(device_name: String) -> Self {
        Self {
            name: "brightness",
            full_text: None,
            color: YELLOW,
            device_name,
        }
    }
}

impl Widget for Brightness {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        let current_brightness = read_first_line_in_file(&format!(
            "{}/{}/actual_brightness",
            BACKLIGHT_PATH, self.device_name
        ));
        let max_brightness = read_first_line_in_file(&format!(
            "{}/{}/max_brightness",
            BACKLIGHT_PATH, self.device_name
        ));

        if let Ok(current_brightness) = current_brightness {
            if let Ok(max_brightness) = max_brightness {
                self.full_text = Some(
                    String::from("☼: ")
                        + &(current_brightness.parse::<f32>().unwrap()
                            / max_brightness.parse::<f32>().unwrap()
                            * 100.0)
                            .round()
                            .to_string()
                        + "%",
                );
            }
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        Ok(serde_json::to_value(self)?)
    }
}
