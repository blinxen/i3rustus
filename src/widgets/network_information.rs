use crate::widgets::Widget;
use crate::widgets::WidgetError;
use std::process::Command;

pub enum NetworkType {
    Ethernet,
    Wlan,
}

pub struct NetworkInformation {
    network_type: NetworkType,
}

impl NetworkInformation {
    pub fn new(network_type: NetworkType) -> Self {
        NetworkInformation {
            network_type: network_type,
        }
    }

    fn get_ethernet_information(&self) -> Result<String, WidgetError> {
        Ok(String::new())
    }

    fn get_wlan_information(&self) -> Result<String, WidgetError> {
        // Information can also be found under /sys/class/net/<DEV>/uevent
        let wlan_devices_list = String::from_utf8(Command::new("iw").arg("dev").output()?.stdout)?;
        // Look for the index of the "Interface" substring
        let wlan_device_name_index = wlan_devices_list.find("Interface ").unwrap();
        // Example: wlp3s0
        let wlan_device_name = &wlan_devices_list[wlan_device_name_index..]
            .split_once("\n")
            .unwrap()
            .0
            .split_once(" ")
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
                .split_once("\n")
                .unwrap()
                .0
                .to_string();
        } else {
            wlan_ssid = String::from("down");
        }
        Ok(format!("W: ({})", wlan_ssid))
    }
}

impl Widget for NetworkInformation {
    fn name(&self) -> &str {
        "network"
    }

    fn display_text(&self) -> Result<String, WidgetError> {
        match self.network_type {
            NetworkType::Ethernet => self.get_ethernet_information(),
            NetworkType::Wlan => self.get_wlan_information(),
        }
    }
}
