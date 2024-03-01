use serde::Serialize;
use serde_json::Value;

use crate::config::GREEN;
use crate::config::RED;
use crate::netlink::Netlink;
use crate::widgets::Widget;
use crate::widgets::WidgetError;

static WIFI_DEVICE_NAME: &str = "wlp3s0";
static ETH_DEVICE_NAME: &str = "enp5s0";

static ETH_DEFAULT: &str = "E: down";
static WIFI_DEFAULT: &str = "W: down";

#[derive(PartialEq, Eq)]
pub enum NetworkType {
    Ethernet,
    Wlan,
}

#[derive(Serialize)]
pub struct NetworkInformation<'a> {
    // Name of the widget
    name: &'static str,
    // Text that will be shown in the status bar
    full_text: String,
    // Color of the text
    color: &'a str,
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

impl<'a> NetworkInformation<'a> {
    pub fn new(network_type: NetworkType) -> Self {
        let name = if network_type == NetworkType::Wlan {
            "wireless"
        } else {
            "ethernet"
        };

        let default_full_text = if network_type == NetworkType::Wlan {
            WIFI_DEFAULT
        } else {
            ETH_DEFAULT
        };

        Self {
            name,
            full_text: default_full_text.to_string(),
            color: RED,
            network_type,
            error: None,
            netlink: Netlink::new(),
            default_full_text: default_full_text.to_string(),
        }
    }

    fn get_ethernet_information(&self) -> Result<String, WidgetError> {
        if let Ok(netlink) = self.netlink.as_ref() {
            let ip = netlink.interface_ip(ETH_DEVICE_NAME)?;
            let bitrate = netlink.interface_bitrate(ETH_DEVICE_NAME)?;
            if ip.is_empty() {
                Ok(self.default_full_text.to_string())
            } else {
                Ok(format!("E: S={} Mb/s => {}", bitrate, ip))
            }
        } else {
            Err(WidgetError::new("Netlink socket error".to_string()))
        }
    }

    fn get_wlan_information(&self) -> Result<String, WidgetError> {
        if let Ok(netlink) = self.netlink.as_ref() {
            let bss = netlink.interface_bss_information(WIFI_DEVICE_NAME)?;
            let ip = netlink.interface_ip(WIFI_DEVICE_NAME)?;
            let bitrate = netlink.interface_bitrate(WIFI_DEVICE_NAME)?;
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
            Err(WidgetError::new("Netlink socket error".to_string()))
        }
    }
}

impl<'a> Widget for NetworkInformation<'a> {
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
