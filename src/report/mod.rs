use std::{
    collections::HashSet,
    fs::{File, read_dir},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;
use inquire::MultiSelect;
use serde_json::Value;

use crate::{
    app::{ReportArgs, ReportMode},
    check::InternetCheckResult,
    project::Project,
};

pub async fn run(args: ReportArgs, project: Project) -> Result<()> {
    let log_dir = match args.dir.as_deref() {
        Some(dir) => dir,
        None => project.log_dir(),
    };

    match args.mode {
        ReportMode::Simple => {
            let report = Report::try_from_prompt(log_dir)?;
            report.simple_info();
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Logfile {
    name: String,
    path: PathBuf,
}

impl Logfile {
    fn new(name: String, path: PathBuf) -> Self {
        Logfile { name, path }
    }

    fn try_collect<P: AsRef<Path>>(dir: P) -> Result<Vec<Logfile>> {
        let logfiles = read_dir(dir.as_ref())?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();

                if path
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|s| s != "jsonl")
                    .unwrap_or(true)
                {
                    None
                } else {
                    e.file_name()
                        .into_string()
                        .ok()
                        .map(|name| Logfile::new(name, path))
                }
            })
            .collect::<Vec<Logfile>>();

        Ok(logfiles)
    }
}

struct Prompter;

impl Prompter {
    fn ask_logfile(logfiles: Vec<Logfile>) -> Result<Vec<Logfile>> {
        #[derive(Debug)]
        struct LogItem<'a>(&'a Logfile);
        impl<'a> std::fmt::Display for LogItem<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0.name)
            }
        }
        let options: Vec<LogItem> = logfiles.iter().map(LogItem).collect();

        let indices: HashSet<usize> = MultiSelect::new("welche, sag:", options)
            .raw_prompt()?
            .into_iter()
            .map(|i| i.index)
            .collect();

        let selected = logfiles
            .into_iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, log)| log)
            .collect();

        Ok(selected)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
struct Report {
    items: Vec<ReportItem>,
}

impl Report {
    fn simple_info(&self) {
        for item in &self.items {
            println!("Logfile: {}", item.logfile.name);

            for result in &item.results {
                let msg = format!("{}: {}", result.get_time(), result.connectivity());
                println!("  {msg}");
            }
            println!();
        }
    }
}

impl Report {
    fn try_from_prompt<P: AsRef<Path>>(log_dir: P) -> Result<Self> {
        let logfiles = Logfile::try_collect(log_dir)?;

        let items = Prompter::ask_logfile(logfiles)?
            .into_iter()
            .map(|logfile| ReportItem::from_logfile(logfile))
            .collect();

        Ok(Report { items })
    }

    fn collect_results_from_logfile(logfile: &Logfile) -> Vec<InternetCheckResult> {
        if let Ok(file) = File::open(&logfile.path) {
            let reader = BufReader::new(file);

            Report::collect_results_from_reader(reader)
        } else {
            vec![]
        }
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
