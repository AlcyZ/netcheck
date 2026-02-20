use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{
    app::{find::FindArgs, monitor::MonitorArgs, report::ReportArgs},
    find::run as find_run,
    monitor::run as monitor_run,
    project::Project,
    report::run as report_run,
};

pub(super) mod find;
pub(super) mod monitor;
pub(super) mod report;
pub(super) mod shared;

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
            Command::Monitor(args) => monitor_run(args, self.project)
                .await
                .context("The monitor command failed"),
            Command::Report(args) => report_run(args, self.project)
                .await
                .context("The report command failed"),
            Command::Find(args) => find_run(args),
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
    Find(FindArgs),
}
