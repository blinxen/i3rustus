use serde_json::Value;

use std::error::Error;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

pub mod battery_life;
pub mod brightness;
pub mod cpu_stats;
pub mod disk_stats;
pub mod memory_stats;
pub mod network_information;
pub mod time;

// All widgets HAVE to implement this trait
pub trait Widget {
    // Get name of the widget
    fn name(&self) -> &str;
    // Update widget values
    fn update(&mut self);
    // The text that will be shown on the status bar
    // This method returns the full_text and the color the text should have
    fn display_text(&self) -> Result<Value, WidgetError>;
}

// This should be used to signal that a widget is not working properly
// The idea is that display_text returns this Error and then we can so some generic
// Error handling for all widgets
#[derive(Debug)]
pub struct WidgetError {
    error_message: String,
}

impl WidgetError {
    pub fn new(msg: String) -> Self {
        WidgetError { error_message: msg }
    }
}

impl fmt::Display for WidgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl Error for WidgetError {}

impl From<io::Error> for WidgetError {
    fn from(item: io::Error) -> Self {
        WidgetError::new(item.to_string())
    }
}

impl From<FromUtf8Error> for WidgetError {
    fn from(item: FromUtf8Error) -> Self {
        WidgetError::new(item.to_string())
    }
}

impl From<serde_json::Error> for WidgetError {
    fn from(item: serde_json::Error) -> Self {
        WidgetError::new(item.to_string())
    }
}
