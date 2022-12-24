use serde_json::{json, Value};
use std::{thread, time};

use crate::{widgets::Widget, LOGGER};

pub(crate) const GREEN: &str = "#08FF00";
pub(crate) const RED: &str = "#FF0000";
pub(crate) const YELLOW: &str = "#FED037";
pub(crate) const NEUTRAL: &str = "#FFFFFF";

pub struct I3Config {
    widgets: Vec<Box<dyn Widget>>,
}

impl I3Config {
    pub fn new(widgets: Vec<Box<dyn Widget>>) -> Self {
        I3Config { widgets }
    }

    fn widgets_config(&mut self) -> Value {
        let mut config = json!([]);
        for widget in self.widgets.iter_mut() {
            widget.update();
            let widget_config = match widget.display_text() {
                Ok(conf) => conf,
                Err(error) => {
                    LOGGER.warning(&format!(
                        "Invalid config for {}: \n\t{}",
                        widget.name(),
                        error
                    ));
                    continue;
                }
            };

            config.as_array_mut().unwrap().push(widget_config);
        }

        config
    }

    pub fn init(&mut self) {
        // This is the output that is read by i3
        println!("{{\"version\":1}}");
        // Begin endless array
        println!("[");
        // Arrays have to be separated by comma in output
        println!("[]");
        loop {
            // The actual config for the status bar
            println!(",{}", self.widgets_config());
            // Wait 1 secs before printing update
            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
