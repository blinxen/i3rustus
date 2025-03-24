use serde::Serialize;
use serde_json::Value;

use crate::config::GREEN;
use crate::config::RED;
use crate::netlink::Netlink;
use crate::widgets::Widget;
use crate::widgets::WidgetError;

#[derive(PartialEq, Eq)]
pub enum NetworkType {
    Ethernet,
    Wlan,
}

#[derive(Serialize)]
pub struct NetworkInformation {
    // Name of the widget
    name: &'static str,
    // Text that will be shown in the status bar
    full_text: String,
    // Color of the text
    color: &'static str,
    // Device name
    device_name: String,
    #[serde(skip_serializing)]
    network_type: NetworkType,
    #[serde(skip_serializing)]
    // Holds the error message if an error occured during widget update
    error: Option<String>,
    #[serde(skip_serializing)]
    netlink: Result<Netlink, std::io::Error>,
    #[serde(skip_serializing)]
    default_full_text: String,
}

impl NetworkInformation {
    pub fn new(network_type: NetworkType, device_name: String) -> Self {
        let name = if network_type == NetworkType::Wlan {
            "wlan"
        } else {
            "ethernet"
        };

        let default_full_text = if network_type == NetworkType::Wlan {
            "W: down"
        } else {
            "E: down"
        };

        Self {
            name,
            full_text: default_full_text.to_string(),
            color: RED,
            device_name,
            network_type,
            error: None,
            netlink: Netlink::new(),
            default_full_text: default_full_text.to_string(),
        }
    }

    fn get_ethernet_information(&self) -> Result<String, WidgetError> {
        if let Ok(netlink) = self.netlink.as_ref() {
            let ip = netlink.interface_ip(&self.device_name)?;
            let bitrate = netlink.interface_bitrate(&self.device_name)?;
            if ip.is_empty() {
                Ok(self.default_full_text.to_string())
            } else {
                Ok(format!("E: S={} Mb/s => {}", bitrate, ip))
            }
        } else {
            Err(WidgetError::new(format!(
                "Netlink socket error: {}",
                &self.netlink.as_ref().unwrap_err()
            )))
        }
    }

    fn get_wlan_information(&self) -> Result<String, WidgetError> {
        if let Ok(netlink) = self.netlink.as_ref() {
            let bss = netlink.interface_bss_information(&self.device_name)?;
            let ip = netlink.interface_ip(&self.device_name)?;
            let bitrate = netlink.interface_bitrate(&self.device_name)?;
            if bss.ssid.is_empty() && ip.is_empty() {
                Ok(self.default_full_text.to_string())
            } else {
                Ok(format!(
                    "W: SSID={} F={} GHz S={} Mb/s => {}",
                    if bss.ssid.is_empty() {
                        String::from("????")
                    } else {
                        bss.ssid
                    },
                    bss.frequency,
                    bitrate,
                    if ip.is_empty() {
                        String::from("????")
                    } else {
                        ip
                    },
                ))
            }
        } else {
            Err(WidgetError::new(format!(
                "Netlink socket error: {}",
                &self.netlink.as_ref().unwrap_err()
            )))
        }
    }
}

impl Widget for NetworkInformation {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        self.error = None;
        // Depending on the network type, we call a different method
        let network_information = if self.network_type == NetworkType::Ethernet {
            self.get_ethernet_information()
        } else {
            self.get_wlan_information()
        };

        match network_information {
            Ok(network_information) => {
                self.error = None;
                self.color = if network_information[1..].eq(": down")
                    || network_information.contains("????")
                {
                    RED
                } else {
                    GREEN
                };
                self.full_text = network_information;
            }
            Err(error) => {
                self.error = Some(error.to_string());
                self.color = RED;
                self.full_text = self.default_full_text.to_string();
            }
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            log::error!(
                "Error occured when trying to get network information.\n{}",
                error_msg
            );
        }

        Ok(serde_json::to_value(self)?)
    }
}
