use chrono::{DateTime, Local, TimeDelta, Utc};

use crate::check::{Connectivity, InternetCheckResult};

pub struct DowntimeTracker<'a> {
    first_offline: Option<&'a InternetCheckResult>,
}

impl<'a> DowntimeTracker<'a> {
    pub fn new() -> Self {
        Self {
            first_offline: None,
        }
    }

    pub fn track<T, F>(&mut self, result: &'a InternetCheckResult, cb: F) -> Option<T>
    where
        F: Fn(&'a InternetCheckResult, &'a InternetCheckResult) -> Option<T>,
    {
        match (self.first_offline, result.connectivity()) {
            (None, Connectivity::Offline) => {
                self.first_offline = Some(result);
                None
            }
            (Some(first), Connectivity::Online) => {
                self.first_offline = None;

                cb(first, result)
            }
            _ => None,
        }
    }

    pub fn format_datetime(datetime: &DateTime<Utc>) -> String {
        let local: DateTime<Local> = datetime.with_timezone(&Local);
        local.format("%d.%m.%Y %H:%M").to_string()
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
        let (value, unit) = Self::human_duration_val(delta);
        format!("{value} {unit}")
    }

    pub fn human_duration_text(delta: &TimeDelta) -> String {
        let (value, unit) = DowntimeTracker::human_duration_val(delta);
        format!("took {value} {unit}")
    }
}
