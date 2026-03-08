use std::borrow::Borrow;

use chrono::{DateTime, Local, TimeDelta, Utc};

use crate::{
    model::{InternetCheckResult, OutageLogPrecision, Report, ReportItem},
    time::Humanize,
    tracker::DowntimeTracker,
};

pub fn handle(report: Report) {
    handle_report(report.clone());

    let mut tracker = DurationTracker::new();

    let deltas = report
        .iter_all_results()
        .filter_map(|r| tracker.track(r).and_then(|(d, _, _)| Some(d)))
        .collect::<Vec<TimeDelta>>();

    println!("Outages: {}", deltas.len());

    if let Some(avg) = DurationTracker::calculate_avg(&deltas) {
        println!("Average duration: {}", avg.humanize());
    }

    handle_still_outage(tracker);
}

fn handle_report(report: Report) {
    report
        .iter_items()
        .map(|i| (i, report.log_precision()))
        .for_each(handle_report_item);
}

fn handle_report_item(data: (&ReportItem, OutageLogPrecision)) {
    let (item, log_precision) = data;
    let outages = item.outages(log_precision);

    println!("Duration Report for: {}", item.logfile_name());
    outages.iter().for_each(|outage| println!("{outage}"));

    if let Some(avg) = DurationTracker::calculate_avg(outages.iter().map(|o| o.duration())) {
        println!("Average duration: {}", avg.humanize());
    }

    println!();
}

fn handle_still_outage(tracker: DurationTracker) {
    if let Some((time, elapsed)) = tracker.still_outage() {
        let message_lost = format!("Connection lost since: {time}");
        let message_elapsed = format!("Outage duration: {elapsed}");
        let message_len = message_lost.len().max(message_elapsed.len());

        let offset = 2;
        let width = message_len + offset + 2;
        let border = "#".repeat(width);

        let pb = || println!("{border}");
        let pe = || println!("#{:^width$}#", "", width = width - offset);
        let pl = |t: String| println!("#{:^width$}#", t, width = width - offset);

        println!();
        pb();
        pe();
        pl("WARNING".into());
        pl("Still no internet connection".into());
        pe();
        pl(message_lost);
        pl(message_elapsed);
        pe();
        pe();
        pb();
    }
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

    fn still_outage(&self) -> Option<(String, String)> {
        self.0.first_offline().map(|r| {
            (
                r.timestamp
                    .with_timezone(&Local)
                    .format("%Y-%m-%d: %H:%M")
                    .to_string(),
                DurationTracker::time_since(r.timestamp),
            )
        })
    }

    fn time_since(timestamp: DateTime<Utc>) -> String {
        let since = Utc::now() - timestamp;
        since.humanize()
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
