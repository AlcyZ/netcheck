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

    // logger.log(&result)?;
    match (previous, result.connectivity()) {
        (None, connectivity) => match connectivity {
            Connectivity::Online => {
                // tracing::info!(?result, "Started - Internet available")
            }
            Connectivity::Offline => {
                // tracing::warn!(?result, "Started - Internet unavailable")
            }
        },
        (Some(_), Connectivity::Offline) => {
            // tracing::warn!(?result, "Internet unavailable")
        }
        (Some(Connectivity::Offline), Connectivity::Online) => {
            // tracing::info!(?result, "Internet restored")
        }
        _ => {}
    }

    Ok(result.connectivity())
}
