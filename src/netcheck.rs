use std::time::Duration;

use reqwest::Client;

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{
    DynResult,
    check::{Connectivity, check_connection},
    runner::run_loop,
};

pub async fn run() -> DynResult<()> {
    let file_appender = tracing_appender::rolling::daily("./logs", "internet_check.log");

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,netcheck=info"));

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .json();

    let stdout_layer = fmt::layer().with_ansi(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    run_loop(
        Duration::from_secs(2),
        observe_connection,
        Some(async || {
            tracing::info!("Finished internet connection observation");

            Ok(())
        }),
    )
    .await?;

    Ok(())
}

async fn observe_connection(client: Client, previous: Option<Connectivity>) -> Connectivity {
    let result = check_connection(client.clone(), None).await;

    match (previous, result.connectivity()) {
        (None, connectivity) => match connectivity {
            Connectivity::Online => {
                tracing::info!(?result, "Started - Internet available")
            }
            Connectivity::Offline => {
                tracing::warn!(?result, "Started - Internet unavailable")
            }
        },
        (Some(_), Connectivity::Offline) => {
            tracing::warn!(?result, "Internet unavailable")
        }
        (Some(Connectivity::Offline), Connectivity::Online) => {
            tracing::info!(?result, "Internet restored")
        }
        _ => {}
    }

    result.connectivity()
}
