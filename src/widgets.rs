use crate::config::{self, TextColor};
use std::error::Error;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

pub mod battery_life;
pub mod cpu_stats;
pub mod disk_stats;
pub mod memory_stats;
pub mod network_information;
pub mod time;

/// All widgets HAVE to implement this trait
pub trait Widget {
    // Name is a &str because we know at compile time how big the string will be
    // so we don't need to use String
    fn name(&self) -> &str;
    // The text that will be shown on the status bar
    // This method returns the full_text and the color the text should have
    fn display_text(&self) -> Result<(String, TextColor), WidgetError>;
    // JSON representation of the widget
    fn to_json(&self) -> String {
        // full_text is defined by i3 and is the display_text
        // Name is not defined by i3 and is only used to know which
        // config belongs to which widget
        let (full_text, text_color): (String, TextColor);

        match self.display_text() {
            Ok((text, color)) => (full_text, text_color) = (text, color),
            Err(error) => (full_text, text_color) = (error.error_message, TextColor::Critical),
        }

        format!(
            "{{ \"full_text\": \"{}\", \"name\": \"{}\", \"color\": \"{}\" }}",
            // unwrap should be safe to use here, because we check
            // wether text is OK or not first
            full_text,
            self.name(),
            match text_color {
                TextColor::Neutral => config::NEUTRAL,
                TextColor::Good => config::GREEN,
                TextColor::Warning => config::YELLOW,
                TextColor::Critical => config::RED,
            }
        )
    }
}

// This should be used to signal that a widget is not working properly
// The idea is that display_text return this Error and then we can so some generic
// Error handling for all widgets in to_json
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
        write!(f, "Woah, something bad happend!")
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
