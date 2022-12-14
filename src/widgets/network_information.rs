use crate::config::TextColor;
use crate::widgets::Widget;
use crate::widgets::WidgetError;
use std::process::Command;

#[derive(PartialEq, Eq)]
pub enum NetworkType {
    Ethernet,
    Wlan,
}

pub struct NetworkInformation {
    network_type: NetworkType,
}

impl NetworkInformation {
    pub fn new(network_type: NetworkType) -> Self {
        NetworkInformation { network_type }
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

impl Widget for NetworkInformation {
    fn name(&self) -> &str {
        "network"
    }

    fn display_text(&self) -> Result<(String, TextColor), WidgetError> {
        let network_fn = if self.network_type == NetworkType::Ethernet {
            NetworkInformation::get_ethernet_information
        } else {
            NetworkInformation::get_wlan_information
        };

        let network_information = network_fn(self)?;
        let color = if network_information.contains("down") {
            TextColor::Critical
        } else {
            TextColor::Good
        };
        Ok((network_information, color))
    }
}
