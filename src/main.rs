#![warn(clippy::all)]
#![warn(clippy::perf)]
#![warn(clippy::style)]

use clap::Parser;
use tokio::runtime::Builder;

mod app;
mod check;
#[macro_use]
mod log;
mod netcheck;
mod runner;

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Builder::new_current_thread().enable_all().build()?;

    let args = app::Args::parse();
    rt.block_on(netcheck::run(args))?;

    Ok(())
}
