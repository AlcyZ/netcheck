use std::{sync::Arc, time::Duration};

use reqwest::Client;

use crate::{DynResult, check::Connectivity, log::Logger};

pub async fn run_loop<Cb, FutCb, Shutdown, FutShutdown>(
    client: Client,
    logger: Arc<Logger>,
    duration: Duration,
    cb: Cb,
    shutdown: Option<Shutdown>,
) -> DynResult<()>
where
    Cb: Fn(Client, Arc<Logger>, Option<Connectivity>) -> FutCb,
    FutCb: Future<Output = DynResult<Connectivity>>,
    Shutdown: FnOnce() -> FutShutdown,
    FutShutdown: Future<Output = DynResult<()>>,
{
    println!("Press CTRL-C to abort...");

    let mut previous = None::<Connectivity>;
    previous = Some(cb(client.clone(), Arc::clone(&logger), previous).await?);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(duration) => {
                previous = Some(
                    cb(client.clone(), Arc::clone(&logger), previous).await?
                );
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
