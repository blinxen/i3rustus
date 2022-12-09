use crate::widgets::Widget;
use crate::widgets::WidgetError;

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
        Ok(String::new())
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
