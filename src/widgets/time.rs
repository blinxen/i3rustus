use chrono::Utc;
use chrono::DateTime;

use crate::widgets::Widget;
use crate::widgets::WidgetError;

pub struct Time { }

impl Widget for Time {

    fn name(&self) -> &str {
        "time"
    }

    fn display_text(&self) -> Result<String, WidgetError> {
        let current_time: DateTime<Utc> = Utc::now();
        Ok(format!("{}", current_time.format("%d.%m.%Y %H:%M:%S")))
    }

}
