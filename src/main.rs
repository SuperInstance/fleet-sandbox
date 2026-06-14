use clap::{Parser, Subcommand};
use colored::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

mod conservation;
mod scanner;
mod report;

use conservation::{ConservationMetrics, FleetAudit};
use scanner::scan_codebase;
use report::{print_file_breakdown, print_fleet_summary, print_audit, print_explanation};

#[derive(Parser)]
#[command(
    name = "fleet-sandbox",
    about = "Govern Your Own Fleet — Conservation law analysis for codebases",
    long_about = "Every codebase is a fleet. Every file is an agent. \
                   This tool measures conservation-law signals so you can govern yours.",
    version,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a codebase and compute conservation metrics
    Scan {
        /// Path to the codebase root
        path: PathBuf,
        /// Show per-file breakdown even for large repos
        #[arg(short, long)]
        verbose: bool,
    },
    /// Full conservation audit with recommendations
    Audit {
        /// Path to the codebase root
        path: PathBuf,
    },
    /// Watch mode — recompute metrics on file change
    Monitor {
        /// Path to the codebase root
        path: PathBuf,
    },
    /// Print the conservation law explanation
    Explain,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Scan { path, verbose } => run_scan(&path, verbose),
        Commands::Audit { path } => run_audit(&path),
        Commands::Monitor { path } => run_monitor(&path),
        Commands::Explain => {
            print_explanation();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run_scan(path: &Path, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = validate_path(path)?;
    let audit = compute_audit(&path)?;

    println!("\n{}", "╔══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║          FLEET SCAN — Conservation Metrics              ║".cyan().bold());
    println!("{}\n", "╚══════════════════════════════════════════════════════════╝".cyan());

    if verbose || audit.files.len() <= 30 {
        print_file_breakdown(&audit);
    } else {
        let shown = 15;
        print_file_breakdown(&FleetAudit {
            files: audit.files.iter().take(shown).cloned().collect(),
            ..audit.clone()
        });
        println!(
            "\n  {} ... {} more files (use {} to see all)",
            "...".dimmed(),
            audit.files.len() - shown,
            "--verbose".yellow()
        );
    }

    println!();
    print_fleet_summary(&audit);

    Ok(())
}

fn run_audit(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let path = validate_path(path)?;
    let audit = compute_audit(&path)?;

    println!("\n{}", "╔══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║          FLEET AUDIT — Full Conservation Report         ║".cyan().bold());
    println!("{}\n", "╚══════════════════════════════════════════════════════════╝".cyan());

    print_file_breakdown(&audit);
    println!();
    print_fleet_summary(&audit);
    println!();
    print_audit(&audit);

    Ok(())
}

fn run_monitor(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let path = validate_path(path)?;

    use notify::{Watcher, RecursiveMode, Event};

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = notify::recommended_watcher(tx)
        .map_err(|e| format!("Failed to create watcher: {e}"))?;

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {e}"))?;

    println!(
        "{} Watching {} — press Ctrl+C to stop\n",
        "◉".green().bold(),
        path.display().to_string().cyan()
    );

    // Initial scan
    let audit = compute_audit(&path)?;
    print_fleet_summary(&audit);
    println!();

    let debounce = Duration::from_millis(500);
    let mut last_event_time = std::time::Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                let now = std::time::Instant::now();
                if now.duration_since(last_event_time) < debounce {
                    continue;
                }
                last_event_time = now;

                if !event.paths.is_empty() {
                    let changed = event.paths
                        .iter()
                        .filter_map(|p| p.strip_prefix(&path).ok())
                        .filter(|p| {
                            p.extension()
                                .is_some_and(|ext| scanner::is_scannable(ext))
                        })
                        .count();

                    if changed > 0 {
                        let current_time = chrono_like_now();
                        println!(
                            "\n{} [{}] Change detected — recomputing...",
                            "↻".yellow().bold(),
                            current_time
                        );

                        match compute_audit(&path) {
                            Ok(new_audit) => {
                                print_fleet_summary(&new_audit);
                                println!();
                            }
                            Err(e) => {
                                eprintln!("  {} Failed to recompute: {}", "✗".red(), e);
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("{} Watch error: {}", "⚠".yellow(), e);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Normal — keep waiting
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("{} Watcher disconnected", "✗".red());
                break;
            }
        }
    }

    Ok(())
}

fn validate_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let canonical = path.canonicalize().map_err(|_| {
        format!(
            "Path not found: {}",
            path.display().to_string().red()
        )
    })?;

    if !canonical.is_dir() {
        return Err(format!(
            "Not a directory: {}",
            canonical.display().to_string().red()
        ).into());
    }

    Ok(canonical)
}

fn compute_audit(path: &Path) -> Result<FleetAudit, Box<dyn std::error::Error>> {
    let files = scan_codebase(path)?;
    if files.is_empty() {
        return Err("No scannable source files found in this path.".into());
    }
    Ok(ConservationMetrics::compute_fleet_audit(files))
}

fn chrono_like_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600) % 24;
    format!("{hours:02}:{mins:02}:{secs:02}")
}
