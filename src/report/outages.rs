use chrono::{DateTime, Local, TimeDelta, Utc};

use crate::{
    check::InternetCheckResult,
    report::{Report, ReportItem, util::DowntimeTracker},
};

pub fn handle(report: Report) {
    handle_report(report.clone());

    let mut tracker = DurationTracker::new();

    let deltas = report
        .items
        .iter()
        .flat_map(|i| &i.results)
        .filter_map(|r| tracker.track(r).and_then(|(d, _, _)| Some(d)))
        .collect::<Vec<TimeDelta>>();

    println!("Outages: {}", deltas.len());

    if let Some(avg) = DurationTracker::calculate_avg(&deltas) {
        let (value, unit) = DowntimeTracker::human_duration_val(&avg);
        println!("Average duration: {} {}", value, unit);
    }
}

fn handle_report(report: Report) {
    report.items.iter().for_each(handle_report_item);
}

fn handle_report_item(item: &ReportItem) {
    let mut tracker = DurationTracker::new();

    let deltas = item
        .results
        .iter()
        .flat_map(|r| tracker.track(r))
        .collect::<Vec<(TimeDelta, &InternetCheckResult, &InternetCheckResult)>>();

    let messages = deltas
        .iter()
        .map(|(delta, first, current)| {
            format!(
                "Internet outage: {} | Duration: {}",
                DurationTracker::timespan_string(first, current),
                DowntimeTracker::human_duration(&delta),
            )
        })
        .collect::<Vec<String>>();

    println!("Duration Report for: {}", item.logfile.name);
    for message in messages {
        println!("{message}");
    }
    println!();
}

struct DurationTracker<'a>(DowntimeTracker<'a>);

impl<'a> DurationTracker<'a> {
    fn new() -> Self {
        Self(DowntimeTracker::new())
    }

    fn track(
        &mut self,
        result: &'a InternetCheckResult,
    ) -> Option<(TimeDelta, &'a InternetCheckResult, &'a InternetCheckResult)> {
        self.0.track(result, |first, current| {
            let delta = current.timestamp - first.timestamp;
            if delta.num_seconds() > 0 {
                Some((delta, first, current))
            } else {
                None
            }
        })
    }

    fn calculate_avg(deltas: &[TimeDelta]) -> Option<TimeDelta> {
        if deltas.is_empty() {
            return None;
        }

        let total_nanos: i128 = deltas
            .iter()
            .map(|d| d.num_nanoseconds().unwrap_or(0) as i128)
            .sum();

        let avg_nanos = total_nanos / (deltas.len() as i128);

        Some(TimeDelta::nanoseconds(avg_nanos as i64))
    }

    fn timespan_string(first: &InternetCheckResult, current: &InternetCheckResult) -> String {
        let to_local_date =
            |d: &DateTime<Utc>| d.with_timezone(&Local).format("%Y-%m-%d").to_string();

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
}
