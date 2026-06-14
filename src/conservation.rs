use crate::scanner::{FileScan, Language};

/// γ (gamma) — Alignment signal.
/// Fraction of the file that is determined by external dependencies (imports).
/// High γ = the file is mostly framework/dependency glue.
#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub path: String,
    pub language: Language,
    pub gamma: f64,
    pub eta: f64,
    pub capacity: f64,
    pub vote: i8,
    pub total_lines: usize,
    pub import_lines: usize,
    pub code_lines: usize,
    #[allow(dead_code)]
    pub symbol_count: usize,
}

/// -1 = over-coupled (retire candidate)
///  0 = balanced (maintain)
/// +1 = under-structured (spawn / break up)
pub fn ternary_vote(gamma: f64, eta: f64) -> i8 {
    // γ+η should sum to 1.0 for a "balanced" file
    // High γ (>0.4) = too much coupling → retire
    // Very low γ (<0.05) and high η with low capacity = under-structured → spawn
    if gamma > 0.40 {
        -1
    } else if gamma < 0.05 && eta > 0.92 {
        // Nearly zero imports but lots of custom code — needs structure
        1
    } else {
        0
    }
}

/// Conservation status for the whole fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConservationStatus {
    Balanced,
    Violation,
    Degenerate,
}

impl ConservationStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ConservationStatus::Balanced => "BALANCED",
            ConservationStatus::Violation => "VIOLATION",
            ConservationStatus::Degenerate => "DEGENERATE",
        }
    }
}

/// The complete fleet-level audit.
#[derive(Debug, Clone)]
pub struct FleetAudit {
    pub files: Vec<FileMetrics>,
    pub gamma: f64,
    pub eta: f64,
    pub capacity: f64,
    pub status: ConservationStatus,
    pub votes_retire: usize,
    pub votes_maintain: usize,
    pub votes_spawn: usize,
}

/// Core conservation computation.
pub struct ConservationMetrics;

impl ConservationMetrics {
    /// Compute metrics for a single file.
    pub fn compute_file(scan: &FileScan) -> FileMetrics {
        let total = scan.total_lines as f64;
        let imports = scan.import_lines as f64;
        let code = scan.code_lines as f64;

        let gamma = if total > 0.0 { imports / total } else { 0.0 };
        let eta = if total > 0.0 { code / total } else { 0.0 };

        let symbol_count = scan.unique_symbols.len();
        let capacity = if symbol_count > 1 {
            (symbol_count as f64).log2()
        } else {
            0.0
        };

        let vote = ternary_vote(gamma, eta);

        FileMetrics {
            path: scan.path.clone(),
            language: scan.language,
            gamma,
            eta,
            capacity,
            vote,
            total_lines: scan.total_lines,
            import_lines: scan.import_lines,
            code_lines: scan.code_lines,
            symbol_count,
        }
    }

    /// Compute the full fleet audit from scanned files.
    pub fn compute_fleet_audit(scans: Vec<FileScan>) -> FleetAudit {
        let files: Vec<FileMetrics> = scans.iter().map(Self::compute_file).collect();

        if files.is_empty() {
            return FleetAudit {
                files: vec![],
                gamma: 0.0,
                eta: 0.0,
                capacity: 0.0,
                status: ConservationStatus::Degenerate,
                votes_retire: 0,
                votes_maintain: 0,
                votes_spawn: 0,
            };
        }

        // Aggregate fleet-level metrics (weighted by lines)
        let total_lines: f64 = files.iter().map(|f| f.total_lines as f64).sum();
        let total_imports: f64 = files
            .iter()
            .map(|f| (f.import_lines as f64 / f.total_lines as f64) * f.total_lines as f64)
            .sum();
        let total_code: f64 = files
            .iter()
            .map(|f| (f.code_lines as f64 / f.total_lines as f64) * f.total_lines as f64)
            .sum();

        let gamma = if total_lines > 0.0 {
            total_imports / total_lines
        } else {
            0.0
        };
        let eta = if total_lines > 0.0 {
            total_code / total_lines
        } else {
            0.0
        };

        // Fleet capacity = sum of file capacities (total information content)
        let capacity: f64 = files.iter().map(|f| f.capacity).sum();

        // Count votes
        let votes_retire = files.iter().filter(|f| f.vote == -1).count();
        let votes_maintain = files.iter().filter(|f| f.vote == 0).count();
        let votes_spawn = files.iter().filter(|f| f.vote == 1).count();

        // Determine conservation status
        let status = Self::determine_status(gamma, eta, votes_retire, votes_spawn, files.len());

        FleetAudit {
            files,
            gamma,
            eta,
            capacity,
            status,
            votes_retire,
            votes_maintain,
            votes_spawn,
        }
    }

    fn determine_status(
        gamma: f64,
        eta: f64,
        votes_retire: usize,
        votes_spawn: usize,
        total_files: usize,
    ) -> ConservationStatus {
        let sum = gamma + eta;

        // Degenerate: almost no structure at all (near-zero capacity, everything in one bucket)
        if sum > 1.15 || sum < 0.5 {
            // The ratios don't even sum to ~1.0 — something is structurally degenerate
            // (massive files, generated code, or minified bundles)
            return ConservationStatus::Degenerate;
        }

        let retire_ratio = votes_retire as f64 / total_files as f64;
        let spawn_ratio = votes_spawn as f64 / total_files as f64;

        // Violation: too many files voting -1 (over-coupled) or +1 (under-structured)
        if retire_ratio > 0.35 || spawn_ratio > 0.35 {
            return ConservationStatus::Violation;
        }

        // Balanced: γ+η ≈ 1.0 and votes are mostly 0
        if (sum - 1.0).abs() < 0.2 && retire_ratio < 0.35 && spawn_ratio < 0.35 {
            return ConservationStatus::Balanced;
        }

        // Default to violation if we're in an ambiguous zone
        ConservationStatus::Violation
    }

    /// Cancellation measure δ(n): how well does γ+η ≈ C hold?
    /// Returns the deviation from conservation.
    pub fn cancellation_delta(gamma: f64, eta: f64, capacity: f64) -> f64 {
        let sum = gamma + eta;
        // Normalize capacity to [0,1] range for comparison
        let normalized_c = if capacity > 0.0 {
            capacity / (capacity + 1.0) // sigmoid-like normalization
        } else {
            0.0
        };
        (sum - normalized_c).abs()
    }
}
