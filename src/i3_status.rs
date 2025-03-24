use crate::config::Config;
use crate::widget_executor::{UpdateWidgetValue, WidgetExecutor, WidgetValue};
use crate::widgets::battery_life::Battery;
use crate::widgets::brightness::Brightness;
use crate::widgets::cpu_stats::CpuUsage;
use crate::widgets::cpu_stats::CpuUsageType;
use crate::widgets::disk_stats::Disk;
use crate::widgets::memory_stats::MemoryUsage;
use crate::widgets::network_information::NetworkInformation;
use crate::widgets::network_information::NetworkType;
use crate::widgets::time::Time;

use actix::{Actor, Addr};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::{thread, time};

pub struct I3Status {
    widget_executors: HashMap<String, Addr<WidgetExecutor>>,
    config: Config,
}

impl I3Status {
    pub fn new() -> Self {
        let config = Config::init();
        Self {
            widget_executors: HashMap::new(),
            config,
        }
    }

    async fn widget_values(&self) -> Value {
        let mut values = json!([]);
        // Make sure widgets are printed in the correct order
        for widget_name in self.config.widget_order.iter() {
            if self.widget_executors.contains_key(widget_name) {
                match self.widget_executors[widget_name]
                    .send(WidgetValue {})
                    .await
                {
                    Ok(Ok(conf)) => values
                        .as_array_mut()
                        .expect("ERROR: Could not get a mutable Vec from serde JSON")
                        .push(conf),
                    Ok(Err(error)) => {
                        log::warn!("Invalid value for {}: \n\t{}", widget_name, error);
                        continue;
                    }
                    _ => {
                        log::error!(
                            "Unexpected error when trying to get the value of {}!",
                            widget_name
                        );
                        continue;
                    }
                };
            }
        }

        values
    }

    fn init_widgets(&mut self) {
        // Initialize widgets
        let executors = vec![
            WidgetExecutor::new(NetworkInformation::new(
                NetworkType::Wlan,
                self.config.wlan_device_name.clone(),
            )),
            WidgetExecutor::new(NetworkInformation::new(
                NetworkType::Ethernet,
                self.config.ethernet_device_name.clone(),
            )),
            WidgetExecutor::new(Battery::new(self.config.battery_device_name.clone())),
            WidgetExecutor::new(CpuUsage::new(CpuUsageType::CpuLoad)),
            WidgetExecutor::new(CpuUsage::new(CpuUsageType::Percentage)),
            WidgetExecutor::new(MemoryUsage::new()),
            WidgetExecutor::new(Disk::new(String::from("root"), String::from("/"))),
            WidgetExecutor::new(Time::new(self.config.timezone_file.clone())),
            WidgetExecutor::new(Brightness::new(self.config.brightness_device_name.clone())),
        ];

        for executor in executors {
            self.widget_executors
                .insert(executor.widget_name().to_owned(), executor.start());
        }
    }

    fn update_widgets(&self) {
        // Send update message to all executors
        // This will start a "update" job
        for executor in self.widget_executors.values() {
            executor.do_send(UpdateWidgetValue {});
        }
    }

    pub async fn init(&mut self) {
        self.init_widgets();
        // Make sure all widgets contain a valid value before starting the actual loop
        self.update_widgets();
        // This is the output that is read by i3
        println!("{{\"version\":1}}");
        // Begin endless array
        println!("[");
        // Arrays have to be separated by comma in output
        println!("[]");
        loop {
            // Print all values, these values will be seen in i3bar
            println!(",{}", self.widget_values().await);
            // Wait 1 secs before printing update
            thread::sleep(time::Duration::from_secs(1));
            self.update_widgets();
        }
    }
}
