use std::path::Path;
use walkdir::WalkDir;
use colored::Colorize;

/// A single source file's raw scanned data.
#[derive(Debug, Clone)]
pub struct FileScan {
    pub path: String,
    pub language: Language,
    pub total_lines: usize,
    pub import_lines: usize,
    pub code_lines: usize,
    pub unique_symbols: Vec<String>,
}

/// Detected language family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
    Unknown,
}

impl Language {
    pub fn icon(&self) -> &'static str {
        match self {
            Language::Rust => "🦀",
            Language::Python => "🐍",
            Language::JavaScript => "📜",
            Language::TypeScript => "🔷",
            Language::Go => "🐹",
            Language::Java => "☕",
            Language::C => "🔧",
            Language::Cpp => "⚙️",
            Language::Unknown => "📄",
        }
    }
}

fn detect_language(ext: &str) -> Language {
    match ext {
        "rs" => Language::Rust,
        "py" => Language::Python,
        "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
        "ts" | "tsx" => Language::TypeScript,
        "go" => Language::Go,
        "java" => Language::Java,
        "c" | "h" => Language::C,
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Language::Cpp,
        _ => Language::Unknown,
    }
}

/// Returns true if the file extension is one we can scan.
pub fn is_scannable(ext: &std::ffi::OsStr) -> bool {
    let ext_str = match ext.to_str() {
        Some(s) => s,
        None => return false,
    };
    matches!(
        detect_language(ext_str),
        Language::Rust
            | Language::Python
            | Language::JavaScript
            | Language::TypeScript
            | Language::Go
            | Language::Java
            | Language::C
            | Language::Cpp
    )
}

/// Directories to skip during traversal.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "vendor",
    "__pycache__",
    ".venv",
    "venv",
    "env",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "coverage",
    ".cache",
    ".idea",
    ".vscode",
    "Pods",
    ".gradle",
    ".m2",
];

fn should_skip(dir_name: &str) -> bool {
    SKIP_DIRS.contains(&dir_name)
}

/// Patterns that indicate an import / dependency line.
fn is_import_line(line: &str, lang: Language) -> bool {
    let trimmed = line.trim();
    match lang {
        Language::Rust => {
            trimmed.starts_with("use ")
                || trimmed.starts_with("extern crate")
                || trimmed.starts_with("mod ")
        }
        Language::Python => {
            trimmed.starts_with("import ")
                || trimmed.starts_with("from ")
                || trimmed.starts_with("include!")
        }
        Language::JavaScript | Language::TypeScript => {
            trimmed.starts_with("import ")
                || trimmed.starts_with("require(")
                || trimmed.starts_with("require ")
                || trimmed.starts_with("const ")
                    && trimmed.contains("require(")
                || trimmed.starts_with("export ... from")
                || (trimmed.starts_with("export ") && trimmed.contains(" from "))
        }
        Language::Go => {
            trimmed.starts_with("import ")
                || trimmed.starts_with("import(")
                || trimmed.starts_with("\t\"")
        }
        Language::Java => trimmed.starts_with("import ") || trimmed.starts_with("package "),
        Language::C | Language::Cpp => {
            trimmed.starts_with("#include")
                || trimmed.starts_with("#import")
                || trimmed.starts_with("using ")
                || trimmed.starts_with("using namespace")
        }
        Language::Unknown => false,
    }
}

/// Extract a symbol (function, struct, class, etc.) identifier from a line.
fn extract_symbols(line: &str, lang: Language) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
        return None;
    }

    match lang {
        Language::Rust => {
            for kw in &["fn ", "struct ", "enum ", "trait ", "impl ", "const ", "static ", "type "] {
                if let Some(pos) = trimmed.find(kw) {
                    let rest = &trimmed[pos + kw.len()..];
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }
        Language::Python => {
            for kw in &["def ", "class ", "async def "] {
                if let Some(pos) = trimmed.find(kw) {
                    let rest = &trimmed[pos + kw.len()..];
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }
        Language::JavaScript | Language::TypeScript => {
            for kw in &[
                "function ",
                "class ",
                "const ",
                "let ",
                "var ",
                "interface ",
                "type ",
                "enum ",
            ] {
                if trimmed.starts_with(kw) {
                    let rest = &trimmed[kw.len()..];
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                        .collect();
                    if !name.is_empty() && name != "from" && name != "require" {
                        return Some(name);
                    }
                }
            }
        }
        Language::Go => {
            for kw in &["func ", "type ", "struct ", "const ", "var "] {
                if let Some(pos) = trimmed.find(kw) {
                    let rest = &trimmed[pos + kw.len()..];
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }
        Language::Java => {
            // Look for method/class/interface declarations
            if trimmed.contains("class ") || trimmed.contains("interface ") {
                for kw in &["class ", "interface ", "enum "] {
                    if let Some(pos) = trimmed.find(kw) {
                        let rest = &trimmed[pos + kw.len()..];
                        let name: String = rest
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        if !name.is_empty() {
                            return Some(name);
                        }
                    }
                }
            }
        }
        Language::C | Language::Cpp => {
            for kw in &["void ", "int ", "char ", "double ", "float ", "struct ", "class ", "enum "] {
                if trimmed.starts_with(kw) || trimmed.contains(&format!(" {} ", kw)) {
                    let rest = if let Some(pos) = trimmed.rfind(kw) {
                        &trimmed[pos + kw.len()..]
                    } else {
                        continue;
                    };
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() && name != "main" {
                        return Some(name);
                    }
                }
            }
        }
        Language::Unknown => {}
    }
    None
}

/// Parse a single source file.
fn parse_file(path: &Path, lang: Language) -> Result<FileScan, std::io::Error> {
    let content = std::fs::read_to_string(path)?;

    let mut total_lines = 0usize;
    let mut import_lines = 0usize;
    let mut code_lines = 0usize;
    let mut symbols = Vec::new();
    let mut seen_symbols = std::collections::HashSet::new();

    for line in content.lines() {
        if line.trim().is_empty() || line.trim().starts_with("//") {
            continue;
        }
        total_lines += 1;

        if is_import_line(line, lang) {
            import_lines += 1;
        } else {
            code_lines += 1;
        }

        if let Some(sym) = extract_symbols(line, lang) {
            if seen_symbols.insert(sym.clone()) {
                symbols.push(sym);
            }
        }
    }

    // Ensure at least 1 to avoid div-by-zero downstream
    let total_lines = total_lines.max(1);

    Ok(FileScan {
        path: path
            .to_string_lossy()
            .to_string(),
        language: lang,
        total_lines,
        import_lines,
        code_lines,
        unique_symbols: symbols,
    })
}

/// Walk the codebase and scan every source file.
pub fn scan_codebase(root: &Path) -> Result<Vec<FileScan>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !should_skip(&name)
            } else {
                true
            }
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                // Log and continue — don't fail the whole scan for one bad file
                eprintln!("{} Skipping unreadable entry: {}", "⚠".yellow(), e);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let ext = match entry.path().extension().and_then(|e| e.to_str()) {
            Some(ext) => ext,
            None => continue,
        };

        let lang = detect_language(ext);
        if lang == Language::Unknown {
            continue;
        }

        match parse_file(entry.path(), lang) {
            Ok(scan) => {
                if scan.total_lines > 1 || !scan.unique_symbols.is_empty() {
                    files.push(scan);
                }
            }
            Err(e) => {
                eprintln!(
                    "{} Could not read {}: {}",
                    "⚠".yellow(),
                    entry.path().display(),
                    e
                );
            }
        }
    }

    // Sort by path for deterministic output
    files.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(files)
}
