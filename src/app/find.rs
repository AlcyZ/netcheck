use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct FindArgs {
    #[command(subcommand)]
    action: FindAction,
}

#[derive(Subcommand, Debug)]
enum FindAction {
    Last,
    Longest,
}
