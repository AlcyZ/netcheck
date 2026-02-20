use anyhow::Result;

use crate::app::find::FindArgs;

pub fn run(args: FindArgs) -> Result<()> {
    println!("find! - {:#?}", args);
    Ok(())
}
