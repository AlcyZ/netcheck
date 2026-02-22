use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::TimeDelta;
use serde_json::Value;

use crate::{
    app::report::{ReportArgs, ReportMode},
    model::InternetCheckResult,
    project::Project,
    time::{Humanize, timespan_string},
    tracker::DowntimeTracker,
};

mod cleanup;
mod outages;
mod simple;

pub async fn run(args: ReportArgs, project: Project) -> Result<()> {
    let report = Report::from_path_bufs(args.logfiles(&project)?);

    match args.mode {
        ReportMode::Simple => simple::handle(report),
        ReportMode::Outages => outages::handle(report),
        ReportMode::Cleanup => cleanup::handle(report),
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(super) struct Report {
    items: Vec<ReportItem>,
}

impl Report {
    pub(super) fn from_path_bufs(paths: Vec<PathBuf>) -> Self {
        let items = paths
            .into_iter()
            .map(|p| ReportItem::from_logfile(Logfile::from_path_buf(p)))
            .collect();

        Self { items }
    }

    pub(super) fn iter_items(&self) -> impl Iterator<Item = &ReportItem> {
        self.items.iter()
    }

    pub(super) fn iter_all_results(&self) -> impl Iterator<Item = &InternetCheckResult> {
        self.iter_items().flat_map(|item| &item.results)
    }

    pub(super) fn all_outages<'a>(&'a self) -> Vec<Outage<'a>> {
        let mut tracker = DowntimeTracker::new();

        self.iter_all_results()
            .filter_map(|result| {
                tracker.track(result, |start, end| {
                    Some(Outage::from_start_end(start, end))
                })
            })
            .collect()
    }
}

impl Report {
    fn collect_results_from_logfile(logfile: &Logfile) -> Vec<InternetCheckResult> {
        Self::collect_results_from_path(&logfile.path)
    }

    fn collect_results_from_path<P: AsRef<Path>>(path: P) -> Vec<InternetCheckResult> {
        match File::open(path.as_ref()) {
            Ok(file) => Self::collect_results_from_file(file),
            Err(_) => vec![],
        }
    }

    fn collect_results_from_file(file: File) -> Vec<InternetCheckResult> {
        let reader = BufReader::new(file);
        Self::collect_results_from_reader(reader)
    }

    fn collect_results_from_reader(reader: BufReader<File>) -> Vec<InternetCheckResult> {
        reader
            .lines()
            .filter_map(|l| l.ok())
            .filter_map(|line| {
                serde_json::from_str(&line)
                    .ok()
                    .and_then(|mut v: Value| v.get_mut("result").map(|r| r.take()))
                    .and_then(|f| serde_json::from_value::<InternetCheckResult>(f).ok())
            })
            .collect::<Vec<InternetCheckResult>>()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ReportItem {
    logfile: Logfile,
    results: Vec<InternetCheckResult>,
}

impl<'a> ReportItem {
    pub(super) fn outages(&'a self) -> Vec<Outage<'a>> {
        let mut tracker = DowntimeTracker::new();

        self.results
            .iter()
            .filter_map(|result| {
                tracker.track(result, |start, end| {
                    Some(Outage::from_start_end(start, end))
                })
            })
            .collect()
    }

    pub(super) fn logfile_name(&self) -> &str {
        &self.logfile.name
    }
}

impl ReportItem {
    fn from_logfile(logfile: Logfile) -> Self {
        let results = Report::collect_results_from_logfile(&logfile);

        ReportItem { logfile, results }
    }
}

#[derive(Debug, Clone)]
struct Logfile {
    name: String,
    path: PathBuf,
}

impl Logfile {
    fn new(name: String, path: PathBuf) -> Self {
        Logfile { name, path }
    }

    fn from_path_buf(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        Self::new(name, path)
    }
}

pub(super) struct Outage<'a> {
    start: &'a InternetCheckResult,
    end: &'a InternetCheckResult,
    duration: TimeDelta,
}

impl<'a> Outage<'a> {
    pub(super) fn duration(&'a self) -> &'a TimeDelta {
        &self.duration
    }
}

impl<'a> Outage<'a> {
    fn new(
        start: &'a InternetCheckResult,
        end: &'a InternetCheckResult,
        duration: TimeDelta,
    ) -> Self {
        Self {
            start,
            end,
            duration,
        }
    }

    fn from_start_end(start: &'a InternetCheckResult, end: &'a InternetCheckResult) -> Self {
        let duration = end.timestamp - start.timestamp;
        Self::new(start, end, duration)
    }
}

impl<'a> Display for Outage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Outage at {} for {}",
            timespan_string(self.start, self.end),
            self.duration.humanize(),
        )
    }
}
