use std::borrow::Borrow;

use chrono::TimeDelta;

use crate::{
    model::InternetCheckResult,
    report::{Report, ReportItem},
    time::{Humanize, timespan_string},
    tracker::DowntimeTracker,
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
        println!("Average duration: {}", avg.humanize());
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
                timespan_string(first, current),
                delta.humanize(),
            )
        })
        .collect::<Vec<String>>();

    println!("Duration Report for: {}", item.logfile.name);
    for message in messages {
        println!("{message}");
    }

    if let Some(avg) = DurationTracker::calculate_avg(deltas.iter().map(|(d, _, _)| d)) {
        println!("Average duration: {}", avg.humanize());
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

    fn calculate_avg<I>(deltas: I) -> Option<TimeDelta>
    where
        I: IntoIterator,
        I::Item: Borrow<TimeDelta>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = deltas.into_iter();
        let count = iter.len();

        if count == 0 {
            return None;
        }

        let total_nanos: i128 = iter
            .map(|d| d.borrow().num_nanoseconds().unwrap_or(0) as i128)
            .sum();

        let avg_nanos = total_nanos / (count as i128);

        Some(TimeDelta::nanoseconds(avg_nanos as i64))
    }
}
