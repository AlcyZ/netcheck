#![warn(clippy::all)]
#![warn(clippy::perf)]
#![warn(clippy::style)]

use anyhow::Result;
use tokio::runtime::Builder;

use crate::app::App;

mod app;
mod check;
mod find;
mod project;
mod runner;
mod sort;
mod time;
mod tracker;
#[macro_use]
mod log;
mod monitor;
mod report;

fn main() {
    if let Err(err) = run() {
        if cfg!(debug_assertions) {
            eprintln!("{:?}", err);
        } else {
            eprintln!("Error: {}", err);

            for cause in err.chain().skip(1) {
                eprintln!("    - {}", cause);
            }
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let rt = Builder::new_current_thread().enable_all().build()?;

    let app = App::new()?;
    rt.block_on(app.run())?;

    Ok(())
}
