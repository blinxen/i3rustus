use actix::{Actor, Addr};
use serde_json::{json, Value};
use std::{thread, time};

use crate::{
    widget_executor::{UpdateWidgetValue, WidgetExecutor, WidgetValue},
    widgets::Widget,
};
use std::collections::HashMap;

pub(crate) const GREEN: &str = "#08FF00";
pub(crate) const RED: &str = "#FF0000";
pub(crate) const YELLOW: &str = "#FED037";
pub(crate) const NEUTRAL: &str = "#FFFFFF";

pub struct I3Config {
    widget_executors: HashMap<String, Addr<WidgetExecutor>>,
}

impl I3Config {
    pub fn new(widgets: Vec<Box<dyn Widget>>) -> Self {
        let mut executors = HashMap::new();

        for widget in widgets {
            // TODO: I don't like the to_string here
            executors.insert(
                widget.name().to_string(),
                WidgetExecutor::new(widget).start(),
            );
        }

        I3Config {
            widget_executors: executors,
        }
    }

    async fn widgets_config(&self) -> Value {
        let mut config = json!([]);
        for (widget_name, executor) in self.widget_executors.iter() {
            match executor.send(WidgetValue {}).await {
                Ok(Ok(conf)) => config
                    .as_array_mut()
                    .expect("ERROR: Could not get a mutable Vec from serde JSON")
                    .push(conf),
                Ok(Err(error)) => {
                    log::warn!("Invalid config for {}: \n\t{}", widget_name, error);
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

        config
    }

    fn update_widgets(&self) {
        // Send update message to all executors
        // This will start a "update" job
        for executor in self.widget_executors.values() {
            executor.do_send(UpdateWidgetValue {});
        }
    }

    pub async fn init(&mut self) {
        // Make sure all widgets contain a valid value before starting the actual loop
        self.update_widgets();
        // This is the output that is read by i3
        println!("{{\"version\":1}}");
        // Begin endless array
        println!("[");
        // Arrays have to be separated by comma in output
        println!("[]");
        loop {
            // The actual config for the status bar
            println!(",{}", self.widgets_config().await);
            // Wait 1 secs before printing update
            thread::sleep(time::Duration::from_secs(1));
            self.update_widgets();
        }
    }
}
