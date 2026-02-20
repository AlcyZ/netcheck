use std::path::PathBuf;

use clap::Args;

use crate::{
    log::{DEFAULT_FILE_PREFIX, DEFAULT_LOG_MODE, DEFAULT_MAX_SIZE, LogMode},
    monitor::{DEFAULT_MONITOR_INTERVAL, DEFAULT_MONITOR_TIMEOUT},
};

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
