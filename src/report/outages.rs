use chrono::{DateTime, TimeDelta, Utc};

use crate::{
    check::{Connectivity, InternetCheckResult},
    report::Report,
};

pub fn handle(report: Report) {
    let mut tracker = DowntimeTracker::new();
    report
        .items
        .iter()
        .flat_map(|i| &i.results)
        .filter_map(|result| tracker.track(result))
        .for_each(|msg| println!("{msg}"));
}

struct DowntimeTracker<'a> {
    first_offline: Option<&'a InternetCheckResult>,
}

impl<'a> DowntimeTracker<'a> {
    fn new() -> Self {
        Self {
            first_offline: None,
        }
    }

    fn track(&mut self, result: &'a InternetCheckResult) -> Option<String> {
        match (self.first_offline, result.connectivity()) {
            (None, Connectivity::Offline) => {
                self.first_offline = Some(result);
                None
            }
            (Some(first), Connectivity::Online) => {
                let duration = result.timestamp - first.timestamp;
                let msg = format!(
                    "Internet ist vom {} bis zum {} ausgefallen.\n    {}",
                    DowntimeTracker::format_datetime(&first.timestamp),
                    DowntimeTracker::format_datetime(&result.timestamp),
                    DowntimeTracker::human_duration(duration)
                );
                self.first_offline = None;
                Some(msg)
            }
            _ => None,
        }
    }

    fn format_datetime(datetime: &DateTime<Utc>) -> String {
        datetime.format("%d.%m.%Y %H:%M").to_string()
    }

    fn human_duration(delta: TimeDelta) -> String {
        let checks = [
            (delta.num_days(), "Tag", "Tage"),
            (delta.num_hours(), "Stunde", "Stunden"),
            (delta.num_minutes(), "Minute", "Minuten"),
            (delta.num_seconds(), "Sekunde", "Sekunden"),
        ];

        let (value, singular, plural) = checks
            .into_iter()
            .find(|(val, _, _)| *val >= 1)
            .unwrap_or(checks[3]);

        let unit = if value == 1 { singular } else { plural };
        format!("Hat {value} {unit} gedauert")
    }
}
