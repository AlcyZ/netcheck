use anyhow::Result;
use clap::{Args, ValueEnum};
use inquire::MultiSelect;
use std::{
    collections::HashSet,
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::{model::OutageLogPrecision, project::Project, sort::sort_by_filename_date};

pub const DEFAULT_REPORT_MODE: ReportMode = ReportMode::Outages;

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    /// Defines reporting mode. Simple just prints a list of times with connectivity status.
    #[arg(short, long, value_enum, default_value_t = DEFAULT_REPORT_MODE)]
    pub mode: ReportMode,

    /// (Optional) Sets log directory.
    #[arg(short, long, value_enum)]
    pub dir: Option<PathBuf>,

    /// Flag: Enable to show exact time of outages.
    #[arg(long, default_value_t = false)]
    exact: bool,

    #[command(flatten)]
    file_args: ReportFileArgs,
}

impl ReportArgs {
    pub fn logfiles(&self, project: &Project) -> Result<Vec<PathBuf>> {
        let logdir = match self.dir.as_deref() {
            Some(p) => p,
            None => project.log_dir(),
        };

        self.file_args.logfiles(logdir)
    }

    pub fn log_precision(&self) -> Option<OutageLogPrecision> {
        if self.exact {
            Some(OutageLogPrecision::Exact)
        } else {
            None
        }
    }
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
struct ReportFileArgs {
    /// Uses positional argument to define files. Wildcards like "*" can be used.
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Picks all available logfiles from the log directory.
    #[arg(short, long)]
    all: bool,

    /// Only pick the last N logfiles.
    #[arg(short, long, value_name = "N")]
    last: Option<usize>,

    /// Enables interactive mode, where files can be selected via MultiSelect.
    #[arg(short, long)]
    interactive: bool,
}

impl<'a> ReportFileArgs {
    pub fn logfiles<P: AsRef<Path>>(&self, logdir: P) -> Result<Vec<PathBuf>> {
        Ok(match self.strategy() {
            ReportFileStrategy::All => Self::try_collect_from_logdir(logdir)?,
            ReportFileStrategy::Last(n) => Self::try_collect_n_from_logdir(logdir, n)?,
            ReportFileStrategy::Files(files) => Self::to_sorted(files),
            ReportFileStrategy::Default => Self::try_collect_n_from_logdir(logdir, 1)?,
            ReportFileStrategy::Interactive => Self::try_ask(logdir)?,
        })
    }

    fn strategy(&'a self) -> ReportFileStrategy<'a> {
        match self.last {
            Some(last) => ReportFileStrategy::Last(last),
            None if !self.files.is_empty() => ReportFileStrategy::Files(&self.files),
            None if self.all => ReportFileStrategy::All,
            None if self.interactive => ReportFileStrategy::Interactive,
            _ => ReportFileStrategy::Default,
        }
    }

    fn try_collect_from_logdir<P: AsRef<Path>>(logdir: P) -> Result<Vec<PathBuf>> {
        let mut logfiles = Self::try_iter_from_logdir(logdir)?.collect::<Vec<PathBuf>>();
        Self::sort_logfiles(&mut logfiles);

        Ok(logfiles)
    }

    fn try_collect_n_from_logdir<P: AsRef<Path>>(logdir: P, n: usize) -> Result<Vec<PathBuf>> {
        let mut logfiles = Self::try_collect_from_logdir(logdir)?;
        logfiles.truncate(n);

        Ok(logfiles)
    }

    fn try_iter_from_logdir<P: AsRef<Path>>(logdir: P) -> Result<impl Iterator<Item = PathBuf>> {
        Ok(read_dir(logdir.as_ref())?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                let is_jsonl = path.extension().and_then(|x| x.to_str()) == Some("jsonl");

                if is_jsonl { Some(path) } else { None }
            }))
    }

    fn try_ask<P: AsRef<Path>>(logdir: P) -> Result<Vec<PathBuf>> {
        let logfiles = ReportFileArgs::try_collect_from_logdir(logdir)?;

        let options = logfiles
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect::<Vec<&str>>();

        let indices: HashSet<usize> =
            MultiSelect::new("Bitte wähle ein oder mehrere Logdateien aus", options)
                .raw_prompt()?
                .into_iter()
                .map(|list| list.index)
                .collect();

        Ok(logfiles
            .into_iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, path)| path)
            .collect())
    }

    fn to_sorted(logfiles: &[PathBuf]) -> Vec<PathBuf> {
        let mut files = logfiles.to_vec();
        Self::sort_logfiles(&mut files);

        files
    }

    fn sort_logfiles(logfiles: &mut Vec<PathBuf>) {
        sort_by_filename_date(logfiles, |p| p.to_str().unwrap_or(""));
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum ReportMode {
    Simple,
    Outages,
    Cleanup,
}

enum ReportFileStrategy<'a> {
    Last(usize),
    Files(&'a [PathBuf]),
    All,
    Default,
    Interactive,
}
