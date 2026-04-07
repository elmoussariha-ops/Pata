mod analyzer;
mod cli;
mod config;
mod diagnostics;
mod history;
mod model;
mod optimizer;
mod patcher;
mod planner;
mod rollback;
mod runner;
mod types;

use analyzer::analyze_project;
use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::AppConfig;
use diagnostics::run_diagnostics;
use history::{HistoryEntry, HistoryStore};
use optimizer::{run_optimization_cycle, spawn_optimizer};
use planner::build_plan;
use std::{
    io::{self, Write},
    path::PathBuf,
};
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;
    let cli = Cli::parse();
    let mut cfg = AppConfig::load_or_create()?;
    if let Some(root) = cli.root {
        cfg.workspace = root;
    }
    let root = cfg.workspace.clone();
    let history = HistoryStore::new();

    info!("Pata démarré sur {}", root.display());
    spawn_optimizer(&root, cfg.clone(), HistoryStore::new()).await;

    match cli.command.unwrap_or(Commands::Repl) {
        Commands::Analyze => {
            let analysis = analyze_project(&root)?;
            println!(
                "Analyse: {} fichiers Rust, {} lignes",
                analysis.rust_files, analysis.total_lines
            );
            history.append(&HistoryEntry::Analysis(analysis))?;
        }
        Commands::Diagnose => {
            let report = run_diagnostics(&root).await?;
            println!("Diagnostics: {}", report.findings.join("; "));
            history.append(&HistoryEntry::Diagnostics(report))?;
        }
        Commands::Plan => {
            let analysis = analyze_project(&root)?;
            let report = run_diagnostics(&root).await?;
            let plan = build_plan(&analysis, &report);
            println!("Plan: {}", plan.summary);
            history.append(&HistoryEntry::Plan(plan))?;
        }
        Commands::OptimizeOnce => {
            let result = run_optimization_cycle(&root, &cfg, &history).await?;
            println!("Optimization suggestion: {}", result.suggestion);
        }
        Commands::History { tail } => {
            for entry in history.tail(tail)? {
                println!("{}", serde_json::to_string_pretty(&entry)?);
            }
        }
        Commands::Repl => run_repl(root, cfg, history).await?,
    }

    Ok(())
}

async fn run_repl(root: PathBuf, cfg: AppConfig, history: HistoryStore) -> Result<()> {
    println!("Pata REPL — commandes: analyze | diagnose | plan | optimize | history | quit");
    let stdin = io::stdin();
    loop {
        print!("pata> ");
        io::stdout().flush()?;
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let cmd = line.trim();
        match cmd {
            "analyze" => {
                let analysis = analyze_project(&root)?;
                println!(
                    "{} fichiers, {} lignes",
                    analysis.rust_files, analysis.total_lines
                );
                history.append(&HistoryEntry::Analysis(analysis))?;
            }
            "diagnose" => {
                let report = run_diagnostics(&root).await?;
                for finding in &report.findings {
                    println!("- {finding}");
                }
                history.append(&HistoryEntry::Diagnostics(report))?;
            }
            "plan" => {
                let analysis = analyze_project(&root)?;
                let report = run_diagnostics(&root).await?;
                let plan = build_plan(&analysis, &report);
                println!("{}", serde_json::to_string_pretty(&plan)?);
                history.append(&HistoryEntry::Plan(plan))?;
            }
            "optimize" => {
                let record = run_optimization_cycle(&root, &cfg, &history).await?;
                println!("suggestion: {}", record.suggestion);
            }
            "history" => {
                for entry in history.tail(5)? {
                    println!("{}", serde_json::to_string_pretty(&entry)?);
                }
            }
            "quit" | "exit" => break,
            "" => continue,
            _ => println!("Commande inconnue."),
        }
    }
    Ok(())
}

fn init_logging() -> Result<()> {
    std::fs::create_dir_all(".pata/logs")?;
    let file_appender = tracing_appender::rolling::daily(".pata/logs", "pata.log");
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(file_appender)
        .init();
    Ok(())
}
