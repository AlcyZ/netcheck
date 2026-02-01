use std::time::Duration;

use chrono::Local;
use reqwest::Client;

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{DynResult, check::check_connection, runner::run_loop};

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
        Duration::from_secs(10),
        do_it,
        // None::<fn() -> Ready<DynResult<()>>>,
        Some(async || {
            log("Fertsch!");

            Ok(())
        }),
    )
    .await?;

    Ok(())
}

async fn do_it(client: Client) -> DynResult<()> {
    println!("i do it now");

    // log("Looos gehts");
    let result = check_connection(client.clone(), None).await;

    // log(format!("{:#?}", result));

    // tracing::info!(result = serde_json::to_string(&result)?);
    tracing::info!(foo = serde_json::to_string(&result)?);

    // let test_asd = serde_json::to_string(&result)?;
    // println!("{test_asd}");

    if !result.is_internet_up() {
        tracing::warn!("Internet ist DOWN!");
    }

    log("done!");

    Ok(())
}

fn log<S: AsRef<str>>(message: S) {
    println!("{}", msg(message))
}

fn msg<S: AsRef<str>>(message: S) -> String {
    format!(
        "[{}]: {}",
        Local::now().format("%H:%M:%S").to_string(),
        message.as_ref()
    )
}
