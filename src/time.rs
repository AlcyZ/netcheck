use chrono::{DateTime, Local, TimeDelta, Utc};

use crate::model::InternetCheckResult;

pub fn timespan_string(start: &InternetCheckResult, end: &InternetCheckResult) -> String {
    timespan_string_custom(start, end, None, None)
}

pub fn timespan_string_custom(
    start: &InternetCheckResult,
    end: &InternetCheckResult,
    format_local_date: Option<&str>,
    format_time: Option<&str>,
) -> String {
    let format_local_date_str = format_local_date.unwrap_or("%Y-%m-%d");
    let format_time_str = format_time.unwrap_or("%H:%M");

    let to_local_date = |d: &DateTime<Utc>| {
        d.with_timezone(&Local)
            .format(format_local_date_str)
            .to_string()
    };
    let to_time = |d: &DateTime<Utc>| d.with_timezone(&Local).format(format_time_str).to_string();

    let date_first = to_local_date(&start.timestamp);
    let date_current = to_local_date(&end.timestamp);

    if date_first == date_current {
        format!(
            "{date_first}: {} - {}",
            to_time(&start.timestamp),
            to_time(&end.timestamp)
        )
    } else {
        format!(
            "{}: {} - {}: {}",
            date_first,
            to_time(&start.timestamp),
            date_current,
            to_time(&end.timestamp)
        )
    }
}

pub trait Humanize {
    fn humanize(&self) -> String;
}

impl Humanize for TimeDelta {
    fn humanize(&self) -> String {
        let total_seconds = self.num_seconds().abs();

        if total_seconds == 0 {
            return "0 seconds".to_string();
        }

        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let mut parts = Vec::new();

        if days > 0 {
            parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
        }
        if hours > 0 {
            parts.push(format!(
                "{} hour{}",
                hours,
                if hours == 1 { "" } else { "s" }
            ));
        }
        if minutes > 0 {
            parts.push(format!(
                "{} minute{}",
                minutes,
                if minutes == 1 { "" } else { "s" }
            ));
        }
        if seconds > 0 {
            parts.push(format!(
                "{} second{}",
                seconds,
                if seconds == 1 { "" } else { "s" }
            ));
        }

        match parts.len() {
            0 => "0 seconds".to_string(),
            1 => parts[0].clone(),
            _ => {
                let last = parts.pop().unwrap();
                format!("{} and {}", parts.join(", "), last)
            }
        }
    }
}
