use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{
    log::{DEFAULT_FILE_PREFIX, DEFAULT_LOG_MODE, DEFAULT_MAX_SIZE, LogMode},
    monitor,
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

    #[arg(short, long, default_value_t = DEFAULT_MAX_SIZE)]
    pub size: u64,

    #[arg(short, long, value_enum, default_value_t = DEFAULT_LOG_MODE)]
    pub mode: LogMode,
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
    #[arg(short, long, default_value_t = 10)]
    pub interval: u64,

    #[arg(short, long, default_value_t = 5)]
    pub timeout: u64,
}

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    /// Defines reporting mode. Simple just prints a list of times with connectivity status.
    #[arg(short, long, value_enum, default_value_t = DEFAULT_REPORT_MODE)]
    pub mode: ReportMode,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum ReportMode {
    Simple,
}
