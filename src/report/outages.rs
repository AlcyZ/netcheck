use crate::{
    check::InternetCheckResult,
    report::{Report, util::DowntimeTracker},
};

pub fn handle(report: Report) {
    let mut tracker = OutagesTracker::new();
    report
        .items
        .iter()
        .flat_map(|i| &i.results)
        .filter_map(|result| tracker.track(result))
        .for_each(|msg| println!("{msg}"));
}

struct OutagesTracker<'a>(DowntimeTracker<'a>);

impl<'a> OutagesTracker<'a> {
    fn new() -> Self {
        Self(DowntimeTracker::new())
    }

    fn track(&mut self, result: &'a InternetCheckResult) -> Option<String> {
        self.0.track(result, |first, current| {
            let duration = current.timestamp - first.timestamp;
            let msg = format!(
                "Internet ist vom {} bis zum {} ausgefallen.\n    {}",
                DowntimeTracker::format_datetime(&first.timestamp),
                DowntimeTracker::format_datetime(&result.timestamp),
                DowntimeTracker::human_duration_text(&duration)
            );
            Some(msg)
        })
    }
}
