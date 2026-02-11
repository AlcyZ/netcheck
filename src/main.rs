#![warn(clippy::all)]
#![warn(clippy::perf)]
#![warn(clippy::style)]

use tokio::runtime::Builder;

use crate::app::App;

mod app;
mod check;
mod project;
mod runner;
#[macro_use]
mod log;
mod monitor;
mod report;

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Builder::new_current_thread().enable_all().build()?;

    let app = App::new()?;
    rt.block_on(app.run())?;

    Ok(())
}
