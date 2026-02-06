use std::{sync::Arc, time::Duration};

use reqwest::Client;

use crate::{
    DynResult,
    check::{Connectivity, check_connection},
    log::Logger,
    runner::run_loop,
};

pub async fn run() -> DynResult<()> {
    let logger = Arc::new(Logger::builder().build());

    run_loop(
        Arc::clone(&logger),
        Duration::from_secs(2),
        observe_connection,
        Some(async || {
            logger.log("i am done!")?;
            println!("DONE!");

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

    Ok(result.connectivity())
}
