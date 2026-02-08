use std::{sync::Arc, time::Duration};

use reqwest::Client;

use crate::{
    DynResult,
    app::MonitorArgs,
    check::{Connectivity, check_connection},
    log::Logger,
    runner::run_loop,
};

pub async fn run(args: MonitorArgs) -> DynResult<()> {
    let logger = Logger::builder()
        .with_mode(args.logger.mode)
        .with_dir(args.logger.location.dir)
        .with_file_prefix(args.logger.location.filename)
        .with_max_size(args.logger.size)
        .build();
    let logger = Arc::new(logger);

    let client = Client::builder()
        .timeout(Duration::from_secs(args.observer.timeout))
        .build()?;

    run_loop(
        client,
        Arc::clone(&logger),
        Duration::from_secs(args.observer.interval),
        observe_connection,
        Some(async || {
            log!(logger, "Graceful shutdown")?;

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
) -> DynResult<Connectivity> {
    let result = check_connection(client.clone(), None).await;

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
