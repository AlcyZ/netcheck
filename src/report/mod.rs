use anyhow::Result;

use crate::{
    app::report::{ReportArgs, ReportMode},
    model::Report,
    project::Project,
};

mod cleanup;
mod outages;
mod simple;

pub async fn run(args: ReportArgs, project: Project) -> Result<()> {
    let report = Report::from_path_bufs(args.logfiles(&project)?, args.log_precision());

    match args.mode {
        ReportMode::Simple => simple::handle(report),
        ReportMode::Outages => outages::handle(report),
        ReportMode::Cleanup => cleanup::handle(report),
    }

    Ok(())
}
