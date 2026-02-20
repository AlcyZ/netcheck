use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde_json::Value;

use crate::{
    app::report::{ReportArgs, ReportMode},
    check::InternetCheckResult,
    project::Project,
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

#[derive(Debug, Clone)]
struct ReportItem {
    logfile: Logfile,
    results: Vec<InternetCheckResult>,
}

impl ReportItem {
    fn from_logfile(logfile: Logfile) -> Self {
        let results = Report::collect_results_from_logfile(&logfile);

        ReportItem { logfile, results }
    }
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

    pub(super) fn iter_all_results(&self) -> impl Iterator<Item = &InternetCheckResult> {
        self.items.iter().flat_map(|item| &item.results)
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
