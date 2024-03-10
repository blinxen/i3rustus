use crate::i3_status::CONFIG;
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
    device_name: &'static str,
}

impl Brightness {
    pub fn new() -> Self {
        Self {
            name: "brightness",
            full_text: None,
            color: YELLOW,
            device_name: CONFIG.brightness_device_name(),
        }
    }
}

impl Widget for Brightness {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        let actual_brightness = read_first_line_in_file(&format!(
            "{}/{}/actual_brightness",
            BACKLIGHT_PATH, self.device_name
        ))
        .unwrap()
        .parse::<f32>()
        .unwrap();
        let max_brightness = read_first_line_in_file(&format!(
            "{}/{}/max_brightness",
            BACKLIGHT_PATH, self.device_name
        ))
        .unwrap()
        .parse::<f32>()
        .unwrap();

        self.full_text = Some(
            String::from("☼: ")
                + &(actual_brightness / max_brightness * 100.0)
                    .round()
                    .to_string()
                + "%",
        );
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        Ok(serde_json::to_value(self)?)
    }
}
