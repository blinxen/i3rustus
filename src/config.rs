pub(crate) const GREEN: &str = "#08FF00";
pub(crate) const RED: &str = "#FF0000";
pub(crate) const YELLOW: &str = "#FED037";
pub(crate) const NEUTRAL: &str = "#FFFFFF";

pub struct Config;

impl Config {
    pub fn new() -> Self {
        Config {}
    }

    pub fn widget_order(&self) -> Vec<String> {
        vec![
            "wireless".to_string(),
            "ethernet".to_string(),
            "battery".to_string(),
            "cpu_load".to_string(),
            "cpu_percentage".to_string(),
            "memory".to_string(),
            "disk".to_string(),
            "time".to_string(),
        ]
    }
}
