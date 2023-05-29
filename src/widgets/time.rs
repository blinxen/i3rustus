use chrono::Local;
use serde::Serialize;
use serde_json::Value;

use crate::config::NEUTRAL;
use crate::widgets::Widget;
use crate::widgets::WidgetError;

#[derive(Serialize)]
pub struct Time<'a> {
    // Name of the widget
    name: &'a str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
}

impl<'a> Time<'a> {
    pub fn new() -> Self {
        Self {
            name: "time",
            full_text: None,
            color: NEUTRAL,
        }
    }
}

impl<'a> Widget for Time<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        self.full_text = Some(Local::now().format("%d.%m.%Y %H:%M:%S").to_string());
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        Ok(serde_json::to_value(self)?)
    }
}
