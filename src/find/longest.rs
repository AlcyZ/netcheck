use chrono::TimeDelta;

use crate::{
    check::InternetCheckResult,
    report::Report,
    time::{Humanize, timespan_string},
    tracker::DowntimeTracker,
};

pub fn run(report: Report) {
    let mut tracker = DurationTracker::new();

    if let Some((first, last, delta)) = report
        .iter_all_results()
        .filter_map(|result| tracker.track(result))
        .max_by_key(|(_, _, delta)| *delta)
    {
        let msg = format!(
            "Longest outage from {}, took {}",
            timespan_string(first, last),
            delta.humanize(),
        );
        println!("{msg}");
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
    ) -> Option<(&'a InternetCheckResult, &'a InternetCheckResult, TimeDelta)> {
        self.0.track(result, |first, current| {
            Some((first, current, current.timestamp - first.timestamp))
        })
    }
}
