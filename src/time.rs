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

pub fn human_duration_val(delta: &TimeDelta) -> (i64, &'static str) {
    let checks = [
        (delta.num_days(), "day", "days"),
        (delta.num_hours(), "hour", "hours"),
        (delta.num_minutes(), "minute", "minutes"),
        (delta.num_seconds(), "second", "seconds"),
    ];

    let (value, singular, plural) = checks
        .into_iter()
        .find(|(val, _, _)| *val >= 1)
        .unwrap_or(checks[3]);
    let unit = if value == 1 { singular } else { plural };

    (value, unit)
}

pub fn human_duration(delta: &TimeDelta) -> String {
    let (value, unit) = human_duration_val(delta);
    format!("{value} {unit}")
}
