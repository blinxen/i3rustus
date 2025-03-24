use std::path::PathBuf;

pub(crate) const GREEN: &str = "#08FF00";
pub(crate) const RED: &str = "#FF0000";
pub(crate) const YELLOW_WARNING: &str = "#FED037";
pub(crate) const YELLOW: &str = "#E5DE00";
pub(crate) const NEUTRAL: &str = "#FFFFFF";

#[derive(Debug)]
pub struct Config {
    pub wlan_device_name: String,
    pub ethernet_device_name: String,
    pub battery_device_name: String,
    pub brightness_device_name: String,
    pub timezone_file: String,
    pub widget_order: Vec<String>,
}

impl Config {
    pub fn init() -> Self {
        let mut config = Config {
            wlan_device_name: String::new(),
            ethernet_device_name: String::new(),
            battery_device_name: String::new(),
            brightness_device_name: String::new(),
            timezone_file: String::new(),
            widget_order: Vec::new(),
        };

        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            if let Ok(config_file) = std::fs::read_to_string(home.join(".config/i3rustus/config")) {
                for line in config_file.lines() {
                    let mut split = line.split('=');
                    let left = split.next().map(str::trim);
                    let right = split.next().map(str::trim);
                    match left {
                        Some("wlan_device_name") => {
                            config.wlan_device_name = String::from(right.unwrap_or(""))
                        }
                        Some("ethernet_device_name") => {
                            config.ethernet_device_name = String::from(right.unwrap_or(""))
                        }
                        Some("battery_device_name") => {
                            config.battery_device_name = String::from(right.unwrap_or(""))
                        }
                        Some("brightness_device_name") => {
                            config.brightness_device_name = String::from(right.unwrap_or(""))
                        }
                        Some("timezone_file") => {
                            config.timezone_file = String::from(right.unwrap_or(""))
                        }
                        Some("order") => {
                            config.widget_order = right
                                .unwrap_or("")
                                .split(",")
                                .map(str::trim)
                                .map(String::from)
                                .collect()
                        }
                        _ => {}
                    }
                }
            }
        }

        config
    }
}
