use serde::Serialize;
use serde_json::Value;

use crate::{
    config::{NEUTRAL, RED},
    utils::file::read_first_line_in_file,
};

use crate::widgets::{Widget, WidgetError};

const CPU_USAGE_THRESHOLD: f32 = 30.0;

#[derive(PartialEq, Eq)]
pub enum CpuUsageType {
    // Show load
    CpuLoad,
    // Show percentage of the cpu usage
    Percentage,
}

#[derive(Serialize)]
pub struct CpuUsage<'a> {
    // Name of the widget
    name: &'a str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
    #[serde(skip_serializing)]
    pub usage_type: CpuUsageType,
    // Last idle time of CPU
    #[serde(skip_serializing)]
    last_idle_usage: f32,
    // Last total usage time of CPU (include idle time)
    #[serde(skip_serializing)]
    last_total_usage: f32,
    #[serde(skip_serializing)]
    // Holds the error message if an error occured during widget update
    error: Option<String>,
}

impl<'a> CpuUsage<'a> {
    pub fn new(usage_type: CpuUsageType) -> Self {
        let name = if usage_type == CpuUsageType::CpuLoad {
            "cpu_load"
        } else {
            "cpu_percentage"
        };

        CpuUsage {
            usage_type,
            last_idle_usage: 0.0,
            last_total_usage: 0.0,
            name,
            full_text: None,
            color: RED,
            error: None,
        }
    }

    fn get_cpu_load(&self) -> Result<String, WidgetError> {
        let load_avg = read_first_line_in_file("/proc/loadavg")?;
        // We only want the the load and not the
        let load = &load_avg.split_whitespace().collect::<Vec<&str>>()[0..3];
        Ok(format!("Load: {}", load.join(", ")))
    }

    fn get_cpu_usage(&mut self) -> Result<f32, WidgetError> {
        let mut total: f32 = 0.0;
        let mut idle: f32 = 0.0;

        let cpu_line = read_first_line_in_file("/proc/stat")?;
        let (_, cpu_stats) = cpu_line.split_once("  ").unwrap();

        for (i, number) in cpu_stats.trim().split(' ').enumerate() {
            let number_as_u32 = number.parse::<f32>().unwrap();

            if i == 3 {
                idle = number_as_u32;
            }
            total += number_as_u32;
        }

        let idle_delta = idle - self.last_idle_usage;
        let total_delta = total - self.last_total_usage;

        self.last_idle_usage = idle;
        self.last_total_usage = total;

        Ok(100.0 * (1.0 - idle_delta / total_delta))
    }
}

impl<'a> Widget for CpuUsage<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        self.error = None;
        if self.usage_type == CpuUsageType::CpuLoad {
            match self.get_cpu_load() {
                Ok(load) => {
                    self.full_text = Some(load);
                    self.color = NEUTRAL;
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        } else {
            match self.get_cpu_usage() {
                Ok(usage) => {
                    self.full_text = Some(format!("CPU:{:.0}%", usage));
                    self.color = if usage > CPU_USAGE_THRESHOLD {
                        RED
                    } else {
                        NEUTRAL
                    };
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        if let Some(error_msg) = &self.error {
            log::error!("Error occured when trying to get CPU stats.\n{}", error_msg);
        }

        Ok(serde_json::to_value(self)?)
    }
}
