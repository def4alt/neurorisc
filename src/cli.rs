use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "NeuroRISC",
    version,
    about = "A neuro-based RISC-V processor",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run,
    Gui,
}
