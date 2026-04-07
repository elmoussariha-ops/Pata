use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "pata", version, about = "Assistant agentique Rust local")]
pub struct Cli {
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Scan,
    Validate,
    Plan { objective: String },
    GeneratePatch { objective: String },
    OptimizeOnce,
    Tui,
}
