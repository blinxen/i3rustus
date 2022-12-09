use crate::widgets::Widget;
use crate::widgets::WidgetError;
use std::process::Command;

pub enum InternetType {
    Ethernet,
    Wlan,
}

pub struct InternetInformation {
    internet_type: InternetType,
}

impl InternetInformation {
    pub fn new(internet_type: InternetType) -> Self {
        InternetInformation {
            internet_type: internet_type,
        }
    }

    fn get_ethernet_information(&self) -> Result<String, WidgetError> {
        Ok(String::new())
    }

    fn get_wlan_information(&self) -> Result<String, WidgetError> {
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

impl Widget for InternetInformation {
    fn name(&self) -> &str {
        "internet"
    }

    fn display_text(&self) -> Result<String, WidgetError> {
        match self.internet_type {
            InternetType::Ethernet => self.get_ethernet_information(),
            InternetType::Wlan => self.get_wlan_information(),
        }
    }
}
