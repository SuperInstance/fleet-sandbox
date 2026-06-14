# Govern Your Own Fleet

> You wouldn't fly a plane without instruments. Why run a codebase without a conservation law?

Every codebase is a fleet. Every file is an agent in that fleet — it makes decisions, holds state, and depends on others. Imports are coupling between agents. Custom logic is autonomy. And just like any fleet, the whole thing is governed by a conservation law.

**fleet-sandbox** is the instrument panel. It scans your codebase, computes conservation-law signals from code patterns, and tells you whether your fleet is balanced, violating the law, or degenerate. It gives you the numbers you need to govern.

---

## The Conservation Law of Codebases

Every file in your codebase can be measured along three axes:

### γ (gamma) — Alignment

**How much of your codebase is determined by external forces.** Frameworks, libraries, dependencies — every import line is a decision someone else made for you. γ measures the import ratio across your files. High γ means your code is mostly someone else's decisions. It means your fleet doesn't belong to you; it belongs to your dependencies.

### η (eta) — Freedom

**How much is your own custom logic.** Every line that isn't an import is a decision you made. η measures the non-import code ratio. High η means your code is yours — for better or worse. Too much freedom without structure is just as dangerous as too little.

### C (capacity) — Information Content

**How much your fleet can express.** Measured as `C = log₂(unique_symbol_count)` across the codebase. This is the information-theoretic capacity of your fleet — the space of things it can represent and do.

### The Law: γ + η ≈ C

When alignment and freedom are in balance, your fleet's total information content is conserved. The codebase can evolve, grow, and adapt without losing meaning. New features come in without old ones becoming incoherent.

When the law is **violated**:
- **γ ≫ η** — Over-coupled. Your fleet is a thin wrapper around someone else's fleet. A dependency update can cascade through everything. You've surrendered your agency.
- **η ≫ γ** — Under-structured. Your fleet reinvents every wheel. No external wisdom enters. Files balloon to thousands of lines. Knowledge doesn't flow between modules.

When the law is **degenerate**: γ + η is nowhere near C. This usually means generated code, minified bundles, or structural anomalies. The fleet isn't really a fleet — it's a pile.

---

## The Ternary Vote

Each file in your fleet casts exactly one vote. This vote tells you what to do with that file:

| Vote | Meaning | Signal |
|------|---------|--------|
| **-1** | **RETIRE** | Over-coupled (γ > 40%). This file is mostly imports — a thin wrapper or glue code. It should be refactored, merged into something else, or deleted entirely. |
| **0** | **MAINTAIN** | Balanced. This file contributes to the fleet's conservation. It has the right mix of external alignment and internal logic. Keep it as-is. |
| **+1** | **SPAWN** | Under-structured (γ < 5%, η > 92%). This file is doing too much alone, with almost no external dependencies. It needs to be broken into modules. It needs structure. It needs to spawn children. |

The fleet's vote distribution tells you what to do next:

- **Many -1 votes** → Your fleet is losing autonomy. Cut dependencies. Wrap framework code behind your own interfaces. Consider vendoring critical deps.
- **Many +1 votes** → Your fleet is under-structured. Add discipline. Extract shared patterns. Let external wisdom in. Break up monolithic files.
- **Mostly 0 votes** → Your fleet is healthy. Keep governing.

---

## Quick Start

### Install

```bash
# From source
git clone <your-repo>
cd fleet-sandbox
cargo install --path .

# Or build and run directly
cargo build --release
./target/release/fleet-sandbox scan ./my-project
```

### Scan a Codebase

```bash
fleet-sandbox scan ./my-project
fleet-sandbox scan ./my-project --verbose
```

Computes conservation metrics for every source file and prints a fleet summary. Shows γ, η, capacity, and the ternary vote for each file.

### Full Audit

```bash
fleet-sandbox audit ./my-project
```

Runs the full conservation audit. Includes everything from `scan`, plus actionable recommendations based on the fleet's status and vote distribution.

### Watch Mode

```bash
fleet-sandbox monitor ./my-project
```

Watches your codebase for file changes and recomputes conservation metrics in real time. Useful during active development — see how your refactoring affects the fleet's balance as you work.

### The Conservation Law Explained

```bash
fleet-sandbox explain
```

Prints a full explanation of the conservation law of codebases. The three signals, the law itself, and the ternary vote — in one readable reference.

---

## Supported Languages

fleet-sandbox understands source files in:

- 🦀 **Rust** (`.rs`)
- 🐍 **Python** (`.py`)
- 📜 **JavaScript** (`.js`, `.jsx`, `.mjs`, `.cjs`)
- 🔷 **TypeScript** (`.ts`, `.tsx`)
- 🐹 **Go** (`.go`)
- ☕ **Java** (`.java`)
- 🔧 **C** (`.c`, `.h`)
- ⚙️ **C++** (`.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`)

It automatically skips `node_modules`, `target`, `.git`, `vendor`, `__pycache__`, `venv`, `dist`, `build`, and other dependency/build directories.

---

## Example Output

```
$ fleet-sandbox scan ./my-rust-project

╔══════════════════════════════════════════════════════════╗
║          FLEET SCAN — Conservation Metrics              ║
╚══════════════════════════════════════════════════════════╝

  FILE BREAKDOWN
  ─────────────────────────────────────────────────────────────────────────────────────────
  File                                                Lang     γ       η    Capacity Vote
  ─────────────────────────────────────────────────────────────────────────────────────────
  src/main.rs                                          🦀     0.12    0.88     4.2b   ●  0
  src/api/mod.rs                                       🦀     0.35    0.65     3.8b   ●  0
  src/api/routes.rs                                    🦀     0.48    0.52     3.1b   ✗ -1
  src/db/models.rs                                     🦀     0.08    0.92     5.0b   ●  0
  src/utils.rs                                         🦀     0.02    0.98     2.5b   ○ +1
  ─────────────────────────────────────────────────────────────────────────────────────────

  FLEET SUMMARY
  ──────────────────────────────────────────────────────────
  Files scanned:           5
  γ (alignment):           0.2100
  η (freedom):             0.7900
  γ + η:                   1.0000
  C (capacity):            18.60 bits
  Status:                  BALANCED

  Votes: ✗ Retire          1
          ● Maintain       3
          ○ Spawn          1

  γ            [████████··································] 0.21
  η            [███████████████████████████████████·······] 0.79
```

---

## How It Works

### Import Detection

fleet-sandbox identifies import/dependency lines using language-specific patterns:

- **Rust**: `use`, `extern crate`, `mod`
- **Python**: `import`, `from ... import`
- **JS/TS**: `import`, `require()`, `export ... from`
- **Go**: `import`, `import()`
- **Java**: `import`, `package`
- **C/C++**: `#include`, `#import`, `using`

### Symbol Extraction

Unique symbols are extracted using language-aware parsing — function names, struct/class definitions, type aliases, traits, interfaces. These define your fleet's information capacity.

### Vote Computation

Each file's γ (import ratio) is compared against thresholds:
- γ > 0.40 → **-1** (RETIRE: too coupled)
- γ < 0.05 and η > 0.92 → **+1** (SPAWN: needs structure)
- Otherwise → **0** (MAINTAIN: balanced)

### Status Determination

The fleet's conservation status is computed from:
- Whether γ + η ≈ 1.0 (conservation holds)
- The ratio of RETIRE and SPAWN votes across all files
- >35% of files voting either way triggers VIOLATION
- Significant deviation from γ + η = 1.0 triggers DEGENERATE

---

## Why This Matters

Every architectural decision is secretly a conservation question. When you add a dependency, you're trading freedom for alignment. When you delete an abstraction, you're trading alignment for freedom. The question is never "should I do this?" — it's "does this conserve the fleet's capacity to evolve?"

fleet-sandbox gives you the instruments to answer that question with data instead of vibes.

Govern your own fleet. Measure. Decide. Conserve.

---

## License

MIT
