use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use reqwest::Client;

use crate::{check::Connectivity, log::Logger};

pub async fn run_loop<Cb, FutCb, Shutdown, FutShutdown>(
    client: Client,
    logger: Arc<Logger>,
    duration: Duration,
    cb: Cb,
    shutdown: Option<Shutdown>,
) -> Result<()>
where
    Cb: Fn(Client, Arc<Logger>, Option<Connectivity>) -> FutCb,
    FutCb: Future<Output = Result<Connectivity>>,
    Shutdown: FnOnce() -> FutShutdown,
    FutShutdown: Future<Output = Result<()>>,
{
    println!("Press CTRL-C to abort...");

    let mut previous = None::<Connectivity>;
    let start = Instant::now();

    previous = Some(cb(client.clone(), Arc::clone(&logger), previous).await?);
    let mut next_tick = duration.saturating_sub(start.elapsed());

    loop {
        tokio::select! {
            res = async {
                tokio::time::sleep(next_tick).await;

                let start = Instant::now();
                let result = cb(client.clone(), Arc::clone(&logger), previous).await;
                let next = duration.saturating_sub(start.elapsed());

                (result, next)
            } => {
                let (cb_result, new_tick) = res;
                previous = Some(cb_result?);
                next_tick = new_tick;
            }

            _ = tokio::signal::ctrl_c() => {
                println!("Gracefully shutdown...");

                if let Some(shutdown_cb) = shutdown {
                    shutdown_cb().await?;
                }

                break;
            },
        }
    }

    Ok(())
}
