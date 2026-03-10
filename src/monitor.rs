use std::{ops::Deref, sync::Arc, time::Duration};

use anyhow::Result;
use reqwest::Client;

use crate::{
    app::monitor::MonitorArgs,
    check::check_connection,
    log::Logger,
    model::{Connectivity, InternetCheckCycle},
    project::Project,
    runner::run_loop,
};

pub const DEFAULT_MONITOR_INTERVAL: u64 = 5;
pub const DEFAULT_MONITOR_TIMEOUT: u64 = 3;
pub const DEFAULT_MONITOR_EXCLUDE_STOPPED: bool = false;

pub async fn run(args: MonitorArgs, project: Project) -> Result<()> {
    let log_dir = match &args.logger.dir {
        Some(path) => path.deref(),
        None => project.log_dir(),
    };

    let logger = Logger::builder()
        .with_mode(args.logger.mode)
        .with_dir(log_dir)
        .with_file_prefix(args.logger.filename)
        .with_max_size(args.logger.size)
        .build()?;
    let logger = Arc::new(logger);

    let client = Client::builder()
        .timeout(Duration::from_secs(args.observer.timeout))
        .build()?;

    run_loop(
        client.clone(),
        Arc::clone(&logger),
        Duration::from_secs(args.observer.interval),
        observe_connection,
        Some(async || {
            if args.observer.exclude_stopped {
                log!(
                    logger,
                    "Graceful shutdown, finally connection check skipped"
                )?;
            } else {
                let result = check_connection(client, None, InternetCheckCycle::Stopped).await;
                log!(
                    logger,
                    "Graceful shutdown, perform final connection check",
                    result
                )?;
            }

            Ok(())
        }),
    )
    .await?;

    Ok(())
}

async fn observe_connection(
    client: Client,
    logger: Arc<Logger>,
    previous: Option<Connectivity>,
) -> Result<Connectivity> {
    let check_cycle = match previous {
        Some(_) => InternetCheckCycle::Started,
        None => InternetCheckCycle::Running,
    };
    let result = check_connection(client.clone(), None, check_cycle).await;

    match (previous, result.connectivity()) {
        (None, connectivity) => match connectivity {
            Connectivity::Online => log!(logger, "Started - Internet available", result)?,
            Connectivity::Offline => log!(logger, "Started - Internet unavailable", result)?,
        },
        (Some(_), Connectivity::Offline) => log!(logger, "Internet unavailable", result)?,
        (Some(Connectivity::Offline), Connectivity::Online) => {
            log!(logger, "Internet restored", result)?
        }
        _ => {}
    }

    logger.sync()?;

    Ok(result.connectivity())
}
