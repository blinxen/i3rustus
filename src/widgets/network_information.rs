use serde::Serialize;
use serde_json::Value;

use crate::config::GREEN;
use crate::config::RED;
use crate::widgets::Widget;
use crate::widgets::WidgetError;
use crate::LOGGER;
use std::process::Command;

#[derive(PartialEq, Eq)]
pub enum NetworkType {
    Ethernet,
    Wlan,
}

#[derive(Serialize)]
pub struct NetworkInformation<'a> {
    // Name of the widget
    name: &'a str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
    #[serde(skip_serializing)]
    network_type: NetworkType,
    #[serde(skip_serializing)]
    // Holds the error message if an error occured during widget update
    error: Option<String>,
}

impl<'a> NetworkInformation<'a> {
    pub fn new(network_type: NetworkType) -> Self {
        let name = if network_type == NetworkType::Wlan {
            "wireless"
        } else {
            "ethernet"
        };

        NetworkInformation {
            name,
            full_text: None,
            color: RED,
            network_type,
            error: None,
        }
    }

    fn get_ethernet_information(&self) -> Result<String, WidgetError> {
        Ok(String::from("E: down"))
    }

    fn get_wlan_information(&self) -> Result<String, WidgetError> {
        // TODO: Use netfilter to get information instead of relying on 'iw'
        // Information can also be found under /sys/class/net/<DEV>/uevent
        let wlan_devices_list = String::from_utf8(Command::new("iw").arg("dev").output()?.stdout)?;
        // Look for the index of the "Interface" substring
        let wlan_device_name_index = wlan_devices_list.find("Interface ").unwrap();
        // Example: wlp3s0
        let wlan_device_name = &wlan_devices_list[wlan_device_name_index..]
            .split_once('\n')
            .unwrap()
            .0
            .split_once(' ')
            .unwrap()
            .1;
        let wlan_info = String::from_utf8(
            Command::new("iw")
                .arg("dev")
                .arg(wlan_device_name)
                .arg("link")
                .output()?
                .stdout,
        )?;
        let wlan_ssid: String;
        if let Some(wlan_ssid_index) = wlan_info.find("SSID") {
            wlan_ssid = wlan_info[wlan_ssid_index..]
                .split_once('\n')
                .unwrap()
                .0
                .to_string();
        } else {
            wlan_ssid = String::from("W: down");
        }
        Ok(format!("W: ({})", wlan_ssid))
    }
}

impl<'a> Widget for NetworkInformation<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        // Depending on the network type, we call a different method
        let network_information = if self.network_type == NetworkType::Ethernet {
            self.get_ethernet_information()
        } else {
            self.get_wlan_information()
        };

        match network_information {
            Ok(network_information) => {
                self.color = if network_information.contains("down") {
                    RED
                } else {
                    GREEN
                };
                self.full_text = Some(network_information);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            LOGGER.error(&format!(
                "Error accured when trying to get network information.\n{}",
                error_msg
            ));
        }

        Ok(serde_json::to_value(self)?)
    }
}
