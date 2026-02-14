use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{
    log::{DEFAULT_FILE_PREFIX, DEFAULT_LOG_MODE, DEFAULT_MAX_SIZE, LogMode},
    monitor::{self, DEFAULT_MONITOR_INTERVAL, DEFAULT_MONITOR_TIMEOUT},
    project::Project,
    report,
};

pub const DEFAULT_REPORT_MODE: ReportMode = ReportMode::Simple;

pub struct App {
    project: Project,
    cli: Cli,
}

impl App {
    pub fn new() -> Result<Self> {
        let cli = Cli::parse();
        let project = Project::new()?;

        Ok(App { cli, project })
    }

    pub async fn run(self) -> Result<()> {
        match self.cli.command {
            Command::Monitor(args) => monitor::run(args, self.project)
                .await
                .context("The monitor command failed"),
            Command::Report(args) => report::run(args, self.project)
                .await
                .context("The report command failed"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Network Monitor & Analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Monitor(MonitorArgs),
    Report(ReportArgs),
}

#[derive(Args, Debug)]
pub struct MonitorArgs {
    #[command(flatten)]
    pub logger: LoggerArgs,

    #[command(flatten)]
    pub observer: ObserverArgs,
}

#[derive(clap::Args, Debug)]
pub struct LoggerArgs {
    /// Sets the logfile name. The logger automatically appends the timestamp and an index to log
    /// file name.
    #[arg(short, long, default_value = DEFAULT_FILE_PREFIX)]
    pub filename: String,

    /// Sets the max size of the logfile. If this value is exceeded, a new logfile will be created.
    #[arg(short, long, default_value_t = DEFAULT_MAX_SIZE)]
    pub size: u64,

    /// Sets the log mode. 'Stdout' will only log in the terminal, 'File' will only log into files.
    #[arg(short, long, value_enum, default_value_t = DEFAULT_LOG_MODE)]
    pub mode: LogMode,

    /// (Optional) Sets log directory.
    #[arg(short, long, value_enum)]
    pub dir: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
pub struct LoggerLocationArgs {
    /// Sets the logfile name. The logger automatically appends the timestamp and an index to log
    /// file name.
    #[arg(short, long, default_value = DEFAULT_FILE_PREFIX)]
    pub filename: String,
}

#[derive(clap::Args, Debug)]
pub struct ObserverArgs {
    /// Sets the interval in which the connection checks will be performed.
    #[arg(short, long, default_value_t = DEFAULT_MONITOR_INTERVAL)]
    pub interval: u64,

    /// Sets the timeout for the requests that check the internet connection.
    #[arg(short, long, default_value_t = DEFAULT_MONITOR_TIMEOUT)]
    pub timeout: u64,
}

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    /// Defines reporting mode. Simple just prints a list of times with connectivity status.
    #[arg(short, long, value_enum, default_value_t = DEFAULT_REPORT_MODE)]
    pub mode: ReportMode,

    /// (Optional) Sets log directory.
    #[arg(short, long, value_enum)]
    pub dir: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum ReportMode {
    Simple,
    Outages,
    Cleanup,
}
