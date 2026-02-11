use std::{
    fs::{File, read_dir},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;

use crate::{app::ReportArgs, check::InternetCheckResult, project::Project};

pub async fn run(args: ReportArgs, project: Project) -> Result<()> {
    match args.mode {
        crate::app::ReportMode::Simple => {
            let report = SimpleReport::from_args(args, project)?;
            report.simple_info();
        }
    }

    Ok(())
}

#[derive(Debug)]
struct SimpleReport {
    results: Vec<InternetCheckResult>,
}

impl SimpleReport {
    fn simple_info(&self) {
        for result in &self.results {
            let msg = format!("{}: {}", result.get_time(), result.connectivity());
            println!("{msg}");
        }
    }
}

impl SimpleReport {
    fn from_args(args: ReportArgs, project: Project) -> Result<Self> {
        let files =
            get_matching_files(project.log_dir(), &args.filename).context("Collecting logfiles")?;

        Ok(SimpleReport {
            results: SimpleReport::collect_results(&files),
        })
    }

    fn collect_results(files: &[PathBuf]) -> Vec<InternetCheckResult> {
        files
            .iter()
            .filter_map(|p| File::open(p).ok())
            .flat_map(|f| SimpleReport::collect_results_from_reader(BufReader::new(f)))
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

fn get_matching_files<D: AsRef<Path>, P: AsRef<str>>(dir: D, prefix: P) -> Result<Vec<PathBuf>> {
    if !dir.as_ref().is_dir() {
        let msg = format!(
            "No logfiles found at {} - Did you started the monitor first?",
            dir.as_ref()
                .to_str()
                .unwrap_or("<path contains invalid unicode>")
        );
        anyhow::bail!(msg);
    }

    let pattern = format!(
        r"^{}_\d{{4}}-\d{{2}}-\d{{2}}_\d+\.jsonl$",
        regex::escape(prefix.as_ref())
    );
    let re = Regex::new(&pattern)?;

    let mut files = read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| re.is_match(s))
                .unwrap_or(false)
        })
        .collect::<Vec<PathBuf>>();

    files.sort();

    Ok(files)
}
