use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, TimeDelta, Utc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    time::{Humanize, timespan_string, timespan_string_custom},
    tracker::DowntimeTracker,
};

#[derive(Debug, Clone)]
pub struct Report {
    items: Vec<ReportItem>,
    log_precision: Option<OutageLogPrecision>,
}

impl Report {
    pub fn from_path_bufs(paths: Vec<PathBuf>, log_precision: Option<OutageLogPrecision>) -> Self {
        let items = paths
            .into_iter()
            .map(|p| ReportItem::from_logfile(Logfile::from_path_buf(p)))
            .collect();

        Self {
            items,
            log_precision,
        }
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ReportItem> {
        self.items.iter()
    }

    pub fn iter_all_results(&self) -> impl Iterator<Item = &InternetCheckResult> {
        self.iter_items().flat_map(|item| &item.results)
    }

    pub fn iter_logfile_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.iter_items().map(|i| &i.logfile.path)
    }

    pub fn all_outages<'a>(&'a self) -> Vec<Outage<'a>> {
        let mut tracker = DowntimeTracker::new();

        self.iter_all_results()
            .filter_map(|result| {
                tracker.track(result, |start, end| {
                    Some(Outage::from_start_end(
                        start,
                        end,
                        self.log_precision.unwrap_or(OutageLogPrecision::Normal),
                    ))
                })
            })
            .collect()
    }

    pub fn log_precision(&self) -> OutageLogPrecision {
        self.log_precision.unwrap_or(OutageLogPrecision::Normal)
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
pub struct ReportItem {
    logfile: Logfile,
    results: Vec<InternetCheckResult>,
}

impl<'a> ReportItem {
    pub fn outages(&'a self, log_precision: OutageLogPrecision) -> Vec<Outage<'a>> {
        let mut tracker = DowntimeTracker::new();

        self.results
            .iter()
            .filter_map(|result| {
                tracker.track(result, |start, end| {
                    Some(Outage::from_start_end(start, end, log_precision))
                })
            })
            .collect()
    }

    pub fn logfile_name(&self) -> &str {
        &self.logfile.name
    }

    pub fn iter_results(&self) -> impl Iterator<Item = &InternetCheckResult> {
        self.results.iter()
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

pub struct Outage<'a> {
    start: &'a InternetCheckResult,
    end: &'a InternetCheckResult,
    duration: TimeDelta,
    log_precision: OutageLogPrecision,
}

impl<'a> Outage<'a> {
    pub fn duration(&'a self) -> &'a TimeDelta {
        &self.duration
    }
}

impl<'a> Outage<'a> {
    fn new(
        start: &'a InternetCheckResult,
        end: &'a InternetCheckResult,
        duration: TimeDelta,
        log_precision: OutageLogPrecision,
    ) -> Self {
        Self {
            start,
            end,
            duration,
            log_precision,
        }
    }

    fn from_start_end(
        start: &'a InternetCheckResult,
        end: &'a InternetCheckResult,
        log_precision: OutageLogPrecision,
    ) -> Self {
        let duration = end.timestamp - start.timestamp;
        Self::new(start, end, duration, log_precision)
    }
}

impl<'a> Display for Outage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timespan = match self.log_precision {
            OutageLogPrecision::Normal => timespan_string(self.start, self.end),
            OutageLogPrecision::Exact => {
                timespan_string_custom(self.start, self.end, None, Some("%H:%M:%S"))
            }
        };

        write!(f, "Outage at {} for {}", timespan, self.duration.humanize(),)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OutageLogPrecision {
    Normal,
    Exact,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CheckTarget {
    Google,
    Example,
    IP,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CheckError {
    Timeout,
    DnsFailure,
    ConnectionRefused,
    TlsError,
    HttpStatus(u16),
    Other(String),
    InvalidRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LatencySpeed {
    Slow,
    Ok,
}

impl LatencySpeed {
    pub fn new(results: &[&TargetResult], latency_threshold: Option<usize>) -> LatencySpeed {
        if results.is_empty() {
            return LatencySpeed::Ok;
        }

        let threshold = latency_threshold.unwrap_or(500);

        let sum: usize = results
            .iter()
            .map(|r| r.latency.get_duration().as_millis() as usize)
            .sum();

        if (sum / results.len()) > threshold {
            LatencySpeed::Slow
        } else {
            LatencySpeed::Ok
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Latency {
    duration: Duration,
    speed: LatencySpeed,
}

impl Latency {
    pub fn from_duration(duration: Duration, threshold: Option<u128>) -> Latency {
        let treshold_value = threshold.unwrap_or(500);

        let speed = if duration.as_millis() > treshold_value {
            LatencySpeed::Slow
        } else {
            LatencySpeed::Ok
        };

        Latency { duration, speed }
    }

    fn get_duration(&self) -> &Duration {
        &self.duration
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TargetResult {
    target: CheckTarget,
    success: bool,
    latency: Latency,
    status_code: Option<u16>,
    error: Option<CheckError>,
}

impl TargetResult {
    pub fn new(
        target: CheckTarget,
        success: bool,
        latency: Latency,
        status_code: Option<u16>,
        error: Option<CheckError>,
    ) -> TargetResult {
        TargetResult {
            target,
            success,
            latency,
            status_code,
            error,
        }
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub fn latency_duration(&self) -> &Duration {
        &self.latency.duration
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connectivity {
    Online,
    Offline,
}

impl From<bool> for Connectivity {
    fn from(value: bool) -> Self {
        if value {
            Connectivity::Online
        } else {
            Connectivity::Offline
        }
    }
}

impl Display for Connectivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Connectivity::Online => write!(f, "Online"),
            Connectivity::Offline => write!(f, "Offline"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InternetCheckResult {
    pub timestamp: DateTime<Utc>,
    connectivity: Connectivity,
    speed: LatencySpeed,
    results: Vec<TargetResult>,
    avg: Duration,
}

impl InternetCheckResult {
    pub fn new(
        connectivity: Connectivity,
        speed: LatencySpeed,
        results: Vec<TargetResult>,
        avg: Duration,
    ) -> InternetCheckResult {
        InternetCheckResult {
            timestamp: Utc::now(),
            connectivity,
            speed,
            results,
            avg,
        }
    }

    pub fn connectivity(&self) -> Connectivity {
        self.connectivity
    }

    pub fn get_time(&self) -> String {
        self.timestamp.format("%d.%m.%y - %H:%M").to_string()
    }
}
