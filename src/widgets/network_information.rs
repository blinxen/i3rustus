use dbus::arg::PropMap;
use dbus::arg::RefArg;
use dbus::blocking::Connection;
use dbus::Path;
use serde::Serialize;
use serde_json::Value;

use crate::config::GREEN;
use crate::config::RED;
use crate::utils::nm_dbus;
use crate::widgets::Widget;
use crate::widgets::WidgetError;
use crate::LOGGER;

use std::collections::HashMap;

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
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
    #[serde(skip_serializing)]
    network_type: NetworkType,
    #[serde(skip_serializing)]
    // Holds the error message if an error occured during widget update
    error: Option<String>,
    #[serde(skip_serializing)]
    dbus_connection: Result<Connection, dbus::Error>,
    #[serde(skip_serializing)]
    dbus_wifi_device_path: Result<Path<'static>, dbus::Error>,
}

impl<'a> NetworkInformation<'a> {
    pub fn new(network_type: NetworkType) -> Self {
        let name = if network_type == NetworkType::Wlan {
            "wireless"
        } else {
            "ethernet"
        };

        let dbus_connection = Connection::new_system();
        let dbus_wifi_device_path = if let Ok(dbus_connection) = &dbus_connection {
            nm_dbus::method_call(
                dbus_connection,
                &Path::new("/org/freedesktop/NetworkManager").unwrap(),
                "org.freedesktop.NetworkManager",
                "GetDeviceByIpIface",
                ("wlp3s0",),
            )
        } else {
            Err(dbus::Error::new_failed("Initial dBus connection failed!"))
        };

        NetworkInformation {
            name,
            full_text: None,
            color: RED,
            network_type,
            error: None,
            dbus_connection,
            dbus_wifi_device_path,
        }
    }

    fn get_ethernet_information(&self) -> Result<String, WidgetError> {
        Ok(ETH_DEFAULT.to_string())
    }

    fn get_wlan_information(&self) -> Result<String, WidgetError> {
        let connection: Path = nm_dbus::get_property(
            self.dbus_connection.as_ref()?,
            self.dbus_wifi_device_path.as_ref()?,
            "org.freedesktop.NetworkManager.Device",
            "ActiveConnection",
        )?;

        if connection.to_string() == "/" {
            return Ok(WIFI_DEFAULT.to_string());
        }

        let state: u32 = nm_dbus::get_property(
            self.dbus_connection.as_ref()?,
            &connection,
            "org.freedesktop.NetworkManager.Connection.Active",
            "State",
        )?;

        // https://people.freedesktop.org/~lkundrak/nm-docs/nm-dbus-types.html#NMActiveConnectionState
        if state != 1 && state != 2 {
            return Ok(WIFI_DEFAULT.to_string());
        }

        // bitrate is in kilobits/second
        let bitrate: u32 = nm_dbus::get_property(
            self.dbus_connection.as_ref()?,
            self.dbus_wifi_device_path.as_ref()?,
            "org.freedesktop.NetworkManager.Device.Wireless",
            "Bitrate",
        )?;

        let connection_object: Path = nm_dbus::get_property(
            self.dbus_connection.as_ref()?,
            &connection,
            "org.freedesktop.NetworkManager.Connection.Active",
            "Connection",
        )?;

        // FIXME: When disconnecting and then reconnecting to WIFI then this keeps
        // return "/" as the object path, which makes no sense
        let ip4_config: Path = nm_dbus::get_property(
            self.dbus_connection.as_ref()?,
            &connection,
            "org.freedesktop.NetworkManager.Connection.Active",
            "Ip4Config",
        )?;

        let connection_settings: HashMap<String, PropMap> = nm_dbus::method_call(
            self.dbus_connection.as_ref()?,
            &connection_object,
            "org.freedesktop.NetworkManager.Settings.Connection",
            "GetSettings",
            (),
        )?;

        let wifi_ip: Vec<PropMap> = nm_dbus::get_property(
            self.dbus_connection.as_ref()?,
            &ip4_config,
            "org.freedesktop.NetworkManager.IP4Config",
            "AddressData",
        )?;

        Ok(format!(
            "W: {} -> {} Mb/s => {}",
            // We are very confident that this many unwraps are fine
            connection_settings
                .get("connection")
                .unwrap()
                .get("id")
                .unwrap()
                .as_str()
                .unwrap(),
            bitrate / 1024,
            wifi_ip[0].get("address").unwrap().as_str().unwrap(),
        ))
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
                self.color = if network_information.contains("down") {
                    RED
                } else {
                    GREEN
                };
                self.full_text = Some(network_information);
            }
            Err(error) => {
                self.error = Some(error.to_string());
                self.color = RED;
                self.full_text = Some(String::from("ERROR"));
            }
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            LOGGER.error(&format!(
                "Error occured when trying to get network information.\n{}",
                error_msg
            ));
        }

        Ok(serde_json::to_value(self)?)
    }
}
