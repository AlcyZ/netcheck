use chrono::{DateTime, Local, TimeDelta, Utc};

use crate::check::InternetCheckResult;

pub fn timespan_string(first: &InternetCheckResult, current: &InternetCheckResult) -> String {
    let to_local_date = |d: &DateTime<Utc>| d.with_timezone(&Local).format("%Y-%m-%d").to_string();

    let to_time = |d: &DateTime<Utc>| d.with_timezone(&Local).format("%H:%M").to_string();

    let date_first = to_local_date(&first.timestamp);
    let date_current = to_local_date(&current.timestamp);

    if date_first == date_current {
        format!(
            "{date_first}: {} - {}",
            to_time(&first.timestamp),
            to_time(&current.timestamp)
        )
    } else {
        format!(
            "{}: {} - {}: {}",
            date_first,
            to_time(&first.timestamp),
            date_current,
            to_time(&current.timestamp)
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
