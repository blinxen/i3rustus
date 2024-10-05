use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::NEUTRAL;
use crate::i3_status::CONFIG;
use crate::widgets::Widget;
use crate::widgets::WidgetError;

const SECONDS_IN_A_MINUTE: i64 = 60;
const SECONDS_IN_AN_HOUR: i64 = 60 * 60;
const SECONDS_IN_A_DAY: i64 = 24 * 60 * 60;

#[derive(Serialize)]
pub struct Time {
    // Name of the widget
    name: &'static str,
    // Text that will be shown in the status bar
    full_text: String,
    // Color of the text
    color: &'static str,
    // Timezone offset in seconds
    tz_offset: i64,
}

impl Time {
    pub fn new() -> Self {
        Self {
            name: "time",
            full_text: String::new(),
            color: NEUTRAL,
            tz_offset: Self::timezone_offset().unwrap_or(0),
        }
    }

    fn timezone_offset() -> Option<i64> {
        let mut offset = 0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let tz_data = tzif::parse_tzif_file(Path::new(CONFIG.timezone())).ok()?;
        let (transition_times, local_time_types, transition_types) =
            if let Some(data) = tz_data.data_block2 {
                (
                    data.transition_times,
                    data.local_time_type_records,
                    data.transition_types,
                )
            } else {
                (
                    tz_data.data_block1.transition_times,
                    tz_data.data_block1.local_time_type_records,
                    tz_data.data_block1.transition_types,
                )
            };

        for (idx, time) in transition_times.iter().enumerate() {
            if now < time.0 {
                break;
            }
            offset = local_time_types[transition_types[idx]].utoff.0;
        }

        Some(offset)
    }

    fn is_leap_year(year: i64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn days_in_year(year: i64) -> i64 {
        if Self::is_leap_year(year) {
            366
        } else {
            365
        }
    }

    fn days_in_month(month: i64, year: i64) -> i64 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => panic!("Enreachable Error: Invalid month: {month}"),
        }
    }

    fn now(&self) -> String {
        let mut epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + self.tz_offset;
        // Calculate the amount of days that is in the EPOCH by dividing EPOCH with the amount of seconds in a day
        let mut total_days = epoch / SECONDS_IN_A_DAY;
        // Calculate the remaining seconds by getting the remainder of the previous calculation
        epoch %= SECONDS_IN_A_DAY;
        // Calculate hours by dividing the remaining seconds by the amount of seconds in a hour
        let hours = epoch / SECONDS_IN_AN_HOUR;
        // Calculate the remaining seconds by getting the remainder of the previous calculation
        epoch %= SECONDS_IN_AN_HOUR;
        // Calculate minutes by dividing the remaining seconds by the amount of seconds in a minute
        let minutes = epoch / SECONDS_IN_A_MINUTE;
        // Calculate the remaining seconds by getting the remainder of the previous calculation
        let seconds = epoch % SECONDS_IN_A_MINUTE;

        // Calculate the current year with respect to leap years
        let mut current_year = 1970;
        while total_days >= Time::days_in_year(current_year) {
            total_days -= Time::days_in_year(current_year);
            current_year += 1;
        }

        // Calculate the current month with respect to leap years
        let mut current_month = 1;
        while total_days >= Time::days_in_month(current_month, current_year) {
            total_days -= Time::days_in_month(current_month, current_year);
            current_month += 1;
        }

        let current_day = total_days + 1;

        format!(
            "{:02}.{:02}.{current_year} {:02}:{:02}:{:02}",
            current_day, current_month, hours, minutes, seconds
        )
    }
}

impl Widget for Time {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        self.full_text = self.now();
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        Ok(serde_json::to_value(self)?)
    }
}
