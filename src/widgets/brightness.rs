use super::{Widget, WidgetError};
use crate::{config::NEUTRAL, utils::file::read_first_line_in_file};
use serde::Serialize;
use serde_json::Value;

const BACKLIGHT_PATH: &str = "/sys/class/backlight/amdgpu_bl1";

#[derive(Serialize)]
pub struct Brightness<'a> {
    // Name of the widget
    name: &'a str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
}

impl<'a> Brightness<'a> {
    pub fn new() -> Self {
        Self {
            name: "brightness",
            full_text: None,
            color: NEUTRAL,
        }
    }
}

impl<'a> Widget for Brightness<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        let actual_brightness =
            read_first_line_in_file(&format!("{}/actual_brightness", BACKLIGHT_PATH))
                .unwrap()
                .parse::<f32>()
                .unwrap();
        let max_brightness = read_first_line_in_file(&format!("{}/max_brightness", BACKLIGHT_PATH))
            .unwrap()
            .parse::<f32>()
            .unwrap();
        self.full_text = Some(
            (actual_brightness / max_brightness * 100.0)
                .round()
                .to_string(),
        );
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        Ok(serde_json::to_value(self)?)
    }
}
