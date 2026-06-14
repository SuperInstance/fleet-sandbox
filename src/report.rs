use crate::conservation::{ConservationStatus, FleetAudit};
use colored::*;

/// Print the conservation law explanation.
pub fn print_explanation() {
    let text = r#"
  ┌─────────────────────────────────────────────────────────────┐
  │            THE CONSERVATION LAW OF CODEBASES                │
  └─────────────────────────────────────────────────────────────┘

  Every codebase is a fleet. Every file is an agent in that fleet.
  The conservation law governs them all.

  ────────────────────────────────────────────────────────────────

  THE THREE SIGNALS

  γ (gamma) — ALIGNMENT
    How much of your codebase is *determined* by external forces —
    frameworks, libraries, dependencies. Measured by import ratio.
    High γ means your code is mostly someone else's decisions.

  η (eta) — FREEDOM
    How much is your own custom logic, your own decisions.
    Measured by non-import code ratio.
    High η means your code is mostly yours — for better or worse.

  C (capacity) — INFORMATION CONTENT
    Total unique symbols across the fleet: C = log₂(symbol_count).
    This is the fleet's information capacity — how much it can
    *express*.

  ────────────────────────────────────────────────────────────────

  THE CONSERVATION LAW

    γ + η ≈ C

  When alignment and freedom are in balance, your fleet's total
  information content is conserved. The codebase can evolve without
  losing meaning.

  When the law is VIOLATED:
    • γ ≫ η  →  Over-coupled. Your fleet is a wrapper around someone
                 else's fleet. Changes in dependencies cascade.
    • η ≫ γ  →  Under-structured. Your fleet reinvents wheels.
                 No external wisdom enters. Chaos grows.

  When the law is DEGENERATE:
    • γ + η is nowhere near C. The codebase is likely auto-generated,
      minified, or structurally anomalous. Audit needed.

  ────────────────────────────────────────────────────────────────

  THE TERNARY VOTE

  Each file casts one vote:

    -1  RETIRE    — Over-coupled (γ > 40%). This file is mostly imports.
                     It should be refactored, merged, or deleted.

     0  MAINTAIN  — Balanced. This file contributes to the fleet's
                     conservation. Keep it as-is.

    +1  SPAWN     — Under-structured (γ < 5%, η > 92%). This file is
                     doing too much alone. Break it into modules, add
                     abstractions, let structure in.

  The fleet's vote distribution tells you what to do next:

    • Many -1 votes → Your fleet is losing autonomy. Cut dependencies.
    • Many +1 votes → Your fleet is under-structured. Add discipline.
    • Mostly 0 votes → Your fleet is healthy. Keep governing.

  ────────────────────────────────────────────────────────────────

  Govern your own fleet. Measure. Decide. Conserve.
"#;
    println!("{}", text);
}

/// Print per-file breakdown table.
pub fn print_file_breakdown(audit: &FleetAudit) {
    println!(
        "  {}",
        "FILE BREAKDOWN".cyan().bold()
    );
    println!(
        "  {}",
        "─────────────────────────────────────────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  {:<50} {:^6} {:^7} {:^7} {:^8} {:^5}",
        "File", "Lang", "γ", "η", "Capacity", "Vote"
    );
    println!(
        "  {}",
        "─────────────────────────────────────────────────────────────────────────────────────────".dimmed()
    );

    for file in &audit.files {
        let display_path = shorten_path(&file.path, 48);
        let vote_str = match file.vote {
            -1 => "✗ -1".red(),
            0 => "●  0".green(),
            1 => "○ +1".yellow(),
            _ => "?".dimmed(),
        };

        let gamma_str = if file.gamma > 0.40 {
            format!("{:.2}", file.gamma).red()
        } else if file.gamma < 0.05 {
            format!("{:.2}", file.gamma).yellow()
        } else {
            format!("{:.2}", file.gamma).normal()
        };

        let eta_str = format!("{:.2}", file.eta);

        println!(
            "  {:<50} {:^6} {:^7} {:^7} {:^8} {:^5}",
            display_path,
            file.language.icon(),
            gamma_str,
            eta_str,
            format!("{:.1}b", file.capacity),
            vote_str,
        );
    }

    println!(
        "  {}",
        "─────────────────────────────────────────────────────────────────────────────────────────".dimmed()
    );
}

/// Print fleet-level summary.
pub fn print_fleet_summary(audit: &FleetAudit) {
    let status_colored = match audit.status {
        ConservationStatus::Balanced => audit.status.label().green().bold(),
        ConservationStatus::Violation => audit.status.label().yellow().bold(),
        ConservationStatus::Degenerate => audit.status.label().red().bold(),
    };

    println!(
        "  {}",
        "FLEET SUMMARY".cyan().bold()
    );
    println!(
        "  {}",
        "──────────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  {:<24} {}",
        "Files scanned:".dimmed(),
        audit.files.len()
    );
    println!(
        "  {:<24} {}",
        "γ (alignment):".dimmed(),
        format!("{:.4}", audit.gamma)
    );
    println!(
        "  {:<24} {}",
        "η (freedom):".dimmed(),
        format!("{:.4}", audit.eta)
    );
    println!(
        "  {:<24} {}",
        "γ + η:".dimmed(),
        format!("{:.4}", audit.gamma + audit.eta)
    );
    println!(
        "  {:<24} {}",
        "C (capacity):".dimmed(),
        format!("{:.2} bits", audit.capacity)
    );
    println!(
        "  {:<24} {}",
        "Status:".dimmed(),
        status_colored
    );

    println!();
    println!(
        "  {:<24} {}",
        "Votes: ✗ Retire".dimmed(),
        audit.votes_retire.to_string().red()
    );
    println!(
        "  {:<24} {}",
        "        ● Maintain".dimmed(),
        audit.votes_maintain.to_string().green()
    );
    println!(
        "  {:<24} {}",
        "        ○ Spawn".dimmed(),
        audit.votes_spawn.to_string().yellow()
    );

    // Conservation balance bar
    println!();
    print_balance_bar(audit.gamma, audit.eta);
}

/// Print full audit with recommendations.
pub fn print_audit(audit: &FleetAudit) {
    println!(
        "  {}",
        "AUDIT & RECOMMENDATIONS".cyan().bold()
    );
    println!(
        "  {}",
        "──────────────────────────────────────────────────────────".dimmed()
    );

    match audit.status {
        ConservationStatus::Balanced => {
            println!(
                "  {} Your fleet satisfies the conservation law.",
                "✓".green().bold()
            );
            println!("  γ + η ≈ C — alignment and freedom are in balance.\n");

            if audit.votes_retire > 0 || audit.votes_spawn > 0 {
                println!("  Minor actions to consider:");
                if audit.votes_retire > 0 {
                    println!(
                        "    • {} file(s) flagged for retirement review (high γ)",
                        audit.votes_retire
                    );
                }
                if audit.votes_spawn > 0 {
                    println!(
                        "    • {} file(s) flagged for structural decomposition (low γ)",
                        audit.votes_spawn
                    );
                }
            } else {
                println!("  No action needed. All files vote MAINTAIN.");
            }
        }
        ConservationStatus::Violation => {
            println!(
                "  {} Conservation law violation detected.",
                "⚠".yellow().bold()
            );
            println!("  γ + η deviates significantly from C.\n");

            let retire_pct =
                (audit.votes_retire as f64 / audit.files.len() as f64) * 100.0;
            let spawn_pct =
                (audit.votes_spawn as f64 / audit.files.len() as f64) * 100.0;

            if audit.gamma > 0.30 {
                println!(
                    "  {} High γ ({:.2}): Your fleet is over-coupled to external dependencies.",
                    "→".cyan(),
                    audit.gamma
                );
                println!("    Recommendations:");
                println!("      • Audit which dependencies can be replaced or removed");
                println!("      • Consider vendoring critical deps to reduce cascade risk");
                println!("      • Wrap framework code behind your own interfaces");
                println!();
            }

            if audit.eta > 0.90 {
                println!(
                    "  {} High η ({:.2}): Your fleet is under-structured.",
                    "→".cyan(),
                    audit.eta
                );
                println!("    Recommendations:");
                println!("      • Identify files with 0 imports doing heavy work");
                println!("      • Introduce abstractions — let external wisdom in");
                println!("      • Consider splitting large files into focused modules");
                println!();
            }

            if retire_pct > 30.0 {
                println!(
                    "  {} {:.0}% of files vote RETIRE — too much coupling.",
                    "→".cyan(),
                    retire_pct
                );
                println!("    Recommendations:");
                println!("      • Prioritize the highest-γ files for refactoring");
                println!("      • Look for files that are entirely import + thin wrappers");
                println!();
            }

            if spawn_pct > 30.0 {
                println!(
                    "  {} {:.0}% of files vote SPAWN — too little structure.",
                    "→".cyan(),
                    spawn_pct
                );
                println!("    Recommendations:");
                println!("      • Break up monolithic files with zero dependencies");
                println!("      • Extract shared patterns into reusable libraries");
                println!("      • Add type definitions, interfaces, or traits");
                println!();
            }
        }
        ConservationStatus::Degenerate => {
            println!(
                "  {} Degenerate state detected.",
                "✗".red().bold()
            );
            println!("  γ + η is far from expected range. The codebase may be:");
            println!("    • Auto-generated or minified");
            println!("    • Structurally anomalous (huge files, no symbols)");
            println!("    • Mixed with non-source artifacts");
            println!();
            println!("  Recommendations:");
            println!("    • Exclude build artifacts and generated code");
            println!("    • Review file sizes — files > 1000 lines skew metrics");
            println!("    • Check if .gitignore'd files are being scanned");
            println!("    • Consider running with --verbose to inspect per-file data");
        }
    }

    println!(
        "\n  {}",
        "──────────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  Conservation check: γ + η = {:.4}  |  Target: ~1.0000",
        audit.gamma + audit.eta
    );
    println!(
        "  Cancelation δ: {:.4}  |  Lower is better\n",
        crate::conservation::ConservationMetrics::cancellation_delta(
            audit.gamma,
            audit.eta,
            audit.capacity
        )
    );
}

/// Print a visual balance bar showing γ vs η.
fn print_balance_bar(gamma: f64, eta: f64) {
    let bar_width = 40;
    let gamma_width = ((gamma * bar_width as f64).round() as usize).min(bar_width);
    let eta_width = ((eta * bar_width as f64).round() as usize).min(bar_width);

    let gamma_bar: String = "█".repeat(gamma_width);
    let eta_bar: String = "█".repeat(eta_width);

    println!("  {:<12} [{}{}] {:.2}", "γ".dimmed(), gamma_bar.red(), "·".repeat(bar_width.saturating_sub(gamma_width)), gamma);
    println!("  {:<12} [{}{}] {:.2}", "η".dimmed(), eta_bar.green(), "·".repeat(bar_width.saturating_sub(eta_width)), eta);
}

/// Shorten a file path for display.
fn shorten_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    // Try to show the last meaningful segments
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 2 {
        return path.to_string();
    }

    let filename = parts.last().unwrap_or(&"");
    let parent = parts.get(parts.len() - 2).unwrap_or(&"");

    let short = format!(".../{}/{}", parent, filename);
    if short.len() <= max_len {
        short
    } else if filename.len() <= max_len {
        filename.to_string()
    } else {
        format!("...{}", &filename[filename.len().saturating_sub(max_len - 3)..])
    }
}
