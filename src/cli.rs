use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "pata",
    version,
    about = "Agent local Rust pour macOS Apple Silicon"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Analyze,
    Diagnose,
    Plan,
    OptimizeOnce,
    History {
        #[arg(default_value_t = 10)]
        tail: usize,
    },
    Repl,
}
