use std::cell::Cell;

use crate::utils::file::read_first_line_in_file;

use super::{Widget, WidgetError};

pub enum CpuUsageType {
    // Show load
    CpuLoad,
    // Show percentage of the cpu usage
    Percentage,
}

pub struct CpuUsage {
    pub usage_type: CpuUsageType,
    // We are using Cell here because we want to be able to change
    // the value of last_idle_usage and last_total_usage.
    // The problem here is that our trait "Widget", defines the function
    // "display_text" with having a borrowed self reference and not a
    // mutable self reference. Thus making it not possible to change the values
    // of the struct. We also don't want to change the traits function signature
    // just because of one widget.
    // Solution --> interior mutability --> Cell (https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
    // With this solution we can make a immutable value mutable again, but
    // we have to ensure that at runtime we don't break the borrowing rules.
    // Also we are using Cell instead of RefCell, because
    // Cell is used for types that implement the Copy trait and we are using ints.
    // RefCell would be an overkill here.
    //
    // Last idle time of CPU
    last_idle_usage: Cell<f32>,
    // Last total usage time of CPU (include idle time)
    last_total_usage: Cell<f32>,
}

impl CpuUsage {
    pub fn new(usage_type: CpuUsageType) -> CpuUsage {
        CpuUsage {
            usage_type: usage_type,
            last_idle_usage: Cell::new(0.0),
            last_total_usage: Cell::new(0.0),
        }
    }

    fn get_cpu_load(&self) -> Result<String, WidgetError> {
        let load_avg = read_first_line_in_file("/proc/loadavg")?;
        // We only want the the load and not the
        let load = &load_avg.split_whitespace().collect::<Vec<&str>>()[0..3];
        Ok(format!("Load: {}", load.join(" ")))
    }

    fn get_cpu_usage(&self) -> Result<String, WidgetError> {
        let mut total: f32 = 0.0;
        let mut idle: f32 = 0.0;

        let cpu_line = read_first_line_in_file("/proc/stat")?;
        let (_, cpu_stats) = cpu_line.split_once("  ").unwrap();

        for (i, number) in cpu_stats.trim().split(" ").enumerate() {
            let number_as_u32 = number.parse::<f32>().unwrap();

            if i == 3 {
                idle = number_as_u32;
            }
            total += number_as_u32;
        }

        let idle_delta = idle - self.last_idle_usage.replace(idle);
        let total_delta = total - self.last_total_usage.replace(total);

        Ok(format!(
            "CPU: {:.0}%",
            100.0 * (1.0 - idle_delta / total_delta)
        ))
    }
}

impl Widget for CpuUsage {
    fn name(&self) -> &str {
        "cpu"
    }

    fn display_text(&self) -> Result<String, WidgetError> {
        match self.usage_type {
            CpuUsageType::CpuLoad => self.get_cpu_load(),
            CpuUsageType::Percentage => self.get_cpu_usage(),
        }
    }
}
