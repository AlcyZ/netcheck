use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct FindArgs {
    #[command(subcommand)]
    pub action: FindAction,

    /// (Optional) Sets log directory.
    #[arg(short, long, value_enum)]
    pub dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum FindAction {
    Longest,
    MostOutages,
}
