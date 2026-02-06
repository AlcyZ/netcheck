use std::path::PathBuf;

use clap::Parser;

use crate::log::{
    DEFAULT_FILE_PREFIX, DEFAULT_LOG_DIR, DEFAULT_LOG_MODE, DEFAULT_MAX_SIZE, LogMode,
};

#[derive(Parser, Debug)]
#[command(version, about = "Network Monitor", long_about = None)]
pub struct Args {
    #[command(flatten)]
    pub logger: LoggerArgs,

    #[command(flatten)]
    pub observer: ObserverArgs,
}

#[derive(clap::Args, Debug)]
pub struct LoggerArgs {
    #[arg(short, long, default_value = DEFAULT_LOG_DIR)]
    pub dir: PathBuf,

    #[arg(short, long, default_value_t = DEFAULT_MAX_SIZE)]
    pub size: u64,

    #[arg(short, long, default_value = DEFAULT_FILE_PREFIX)]
    pub filename: String,

    #[arg(short, long, value_enum, default_value_t = DEFAULT_LOG_MODE)]
    pub mode: LogMode,
}

#[derive(clap::Args, Debug)]
pub struct ObserverArgs {
    #[arg(short, long, default_value_t = 10)]
    pub interval: u64,

    #[arg(short, long, default_value_t = 5)]
    pub timeout: u64,
}
