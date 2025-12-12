use crate::cli::{Cli, Commands};
use clap::Parser;

mod cli;
mod commands;
mod core;
mod neuro;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run => commands::run::run()?,
    }

    Ok(())
}
