use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{
    DynResult,
    log::{DEFAULT_FILE_PREFIX, DEFAULT_LOG_DIR, DEFAULT_LOG_MODE, DEFAULT_MAX_SIZE, LogMode},
    monitor, report,
};

pub const DEFAULT_REPORT_MODE: ReportMode = ReportMode::Simple;

#[derive(Parser, Debug)]
#[command(version, about = "Network Monitor & Analyzer")]
pub struct App {
    #[command(subcommand)]
    command: Command,
}

impl App {
    pub async fn run(self) -> DynResult<()> {
        match self.command {
            Command::Monitor(args) => monitor::run(args).await,
            Command::Report(args) => report::run(args).await,
        }
    }
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
    #[command(flatten)]
    pub location: LoggerLocationArgs,

    #[arg(short, long, default_value_t = DEFAULT_MAX_SIZE)]
    pub size: u64,

    #[arg(short, long, value_enum, default_value_t = DEFAULT_LOG_MODE)]
    pub mode: LogMode,
}

#[derive(clap::Args, Debug)]
pub struct LoggerLocationArgs {
    #[arg(short, long, default_value = DEFAULT_LOG_DIR)]
    pub dir: PathBuf,

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
    #[command(flatten)]
    pub location: LoggerLocationArgs,

    /// Defines reporting mode. Simple just prints a list of times with connectivity status.
    #[arg(short, long, value_enum, default_value_t = DEFAULT_REPORT_MODE)]
    pub mode: ReportMode,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum ReportMode {
    Simple,
}
