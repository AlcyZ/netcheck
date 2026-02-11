use std::{
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
    match args.mode {
        ReportMode::Simple => {
            let report = Report::try_from_prompt(project.log_dir())?;
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
    fn ask_logfile(logfiles: &[Logfile]) -> Result<Vec<&Logfile>> {
        struct LogItem<'a>(&'a Logfile);
        impl<'a> std::fmt::Display for LogItem<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0.name)
            }
        }
        let options: Vec<LogItem> = logfiles.iter().map(LogItem).collect();

        let selected = MultiSelect::new("welche, sag:", options).prompt()?;

        Ok(selected.into_iter().map(|item| item.0).collect())
    }
}

#[derive(Debug)]
struct Report {
    results: Vec<InternetCheckResult>,
}

impl Report {
    fn simple_info(&self) {
        for result in &self.results {
            let msg = format!("{}: {}", result.get_time(), result.connectivity());
            println!("{msg}");
        }
    }
}

impl Report {
    fn try_from_prompt<P: AsRef<Path>>(log_dir: P) -> Result<Self> {
        let logfiles = Logfile::try_collect(log_dir)?;

        let selected_logfiles = Prompter::ask_logfile(&logfiles)?
            .iter()
            .map(|f| f.path.as_path())
            .collect::<Vec<&Path>>();

        Ok(Report::from_files(&selected_logfiles))
    }

    fn from_files<P: AsRef<Path>>(files: &[P]) -> Self {
        Report {
            results: Report::collect_results(files),
        }
    }

    fn collect_results<P: AsRef<Path>>(files: &[P]) -> Vec<InternetCheckResult> {
        files
            .iter()
            .filter_map(|p| File::open(p).ok())
            .flat_map(|f| Report::collect_results_from_reader(BufReader::new(f)))
            .collect()
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
