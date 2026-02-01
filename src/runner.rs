use std::time::Duration;

use reqwest::Client;

use crate::DynResult;

pub async fn run_loop<Cb, FutCb, Shutdown, FutShutdown, Previous>(
    duration: Duration,
    cb: Cb,
    shutdown: Option<Shutdown>,
) -> DynResult<()>
where
    Cb: Fn(Client) -> FutCb,
    FutCb: Future<Output = DynResult<()>>,
    Shutdown: FnOnce() -> FutShutdown,
    FutShutdown: Future<Output = DynResult<()>>,
{
    let mut count = 0;
    println!("Press CTRL-C to abort...");

    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    cb(client.clone()).await?;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(duration) => cb(client.clone()).await?,
            _ = tokio::signal::ctrl_c() => {
                println!("Gracefully shutdown...");

                if let Some(shutdown_cb) = shutdown {
                    shutdown_cb().await?;
                }

                break;
            },
        }
        count = count + 1;
    }

    Ok(())
}
