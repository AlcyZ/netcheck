use tokio::runtime::Builder;

mod check;
mod netcheck;
mod runner;

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Builder::new_current_thread().enable_all().build()?;

    rt.block_on(netcheck::run())?;

    Ok(())
}
