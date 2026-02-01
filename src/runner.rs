use std::time::Duration;

use reqwest::Client;

use crate::{DynResult, check::Connectivity};

pub async fn run_loop<Cb, FutCb, Shutdown, FutShutdown>(
    duration: Duration,
    cb: Cb,
    shutdown: Option<Shutdown>,
) -> DynResult<()>
where
    Cb: Fn(Client, Option<Connectivity>) -> FutCb,
    FutCb: Future<Output = Connectivity>,
    Shutdown: FnOnce() -> FutShutdown,
    FutShutdown: Future<Output = DynResult<()>>,
{
    let mut previous = None::<Connectivity>;
    println!("Press CTRL-C to abort...");

    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    previous = Some(cb(client.clone(), previous).await);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(duration) => {
                previous = Some(
                    cb(client.clone(), previous).await
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
