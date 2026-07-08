use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use serde::Serialize;

pub mod fidelity;
pub mod llm;
pub mod minifier;
pub mod skill;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub exit_code: i32,
}

impl AppError {
    pub fn new(message: impl Into<String>, exit_code: i32) -> Self {
        Self {
            message: message.into(),
            exit_code,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string(), 3)
    }
}

#[derive(Parser, Debug)]
#[command(name = "skill-compress")]
#[command(
    version,
    about = "Analyze and minify SKILL.md files\nMade with ❤️ by Kantic Analytics"
)]
pub struct Cli {
    /// Path to the SKILL.md file (the original / source of truth).
    pub path: PathBuf,

    // --- Output: what to do with the deterministic result (default: print report) ---
    /// Rewrite the file in place with the deterministic minified output.
    #[arg(long, help_heading = "Output")]
    pub write: bool,

    /// Fail when the deterministic minified output differs or diagnostics contain errors.
    #[arg(long, help_heading = "Output")]
    pub check: bool,

    /// Print a unified diff instead of the normal human report.
    #[arg(long, help_heading = "Output")]
    pub diff: bool,

    /// Emit output in the selected format.
    #[arg(long, value_enum, default_value_t = ReportFormat::Human, help_heading = "Output")]
    pub report: ReportFormat,

    // --- Compression: optional section stripping applied to the minified output ---
    /// Remove changelog/release-notes/history sections from deterministic output.
    #[arg(long, help_heading = "Compression")]
    pub strip_changelog: bool,

    /// Keep only the latest N changelog list items when stripping changelog sections.
    #[arg(long, help_heading = "Compression")]
    pub keep_latest: Option<usize>,

    /// Remove example-oriented sections such as Examples, Avoid, Prefer, and Before/After.
    #[arg(long, help_heading = "Compression")]
    pub strip_examples: bool,

    /// Remove explanatory meta-prose sections such as Reference, What it is, and Philosophy.
    #[arg(long, help_heading = "Compression")]
    pub strip_meta_prose: bool,

    /// Enable a conservative bundle of nonessential stripping rules.
    #[arg(long, help_heading = "Compression")]
    pub strip_nonessential: bool,

    /// Keep content aimed at runtime execution and strip history/examples/meta-prose.
    #[arg(long, help_heading = "Compression")]
    pub runtime_only: bool,

    // --- Verification: check a compressed candidate against PATH (the original) ---
    /// Verify that CANDIDATE preserves every must-preserve atom of PATH (rules,
    /// acceptance lines, section headings, frontmatter keys, code blocks). Exits
    /// nonzero if any is missing. No LLM call; matching is deterministic.
    #[arg(long, value_name = "CANDIDATE", help_heading = "Verification")]
    pub verify: Option<PathBuf>,

    /// Experimental: with --verify, ask an LLM to judge whether the deterministically
    /// missing atoms are paraphrased-equivalent, weakened, or truly lost. Sends the
    /// candidate to the configured provider; the deterministic verdict stays authoritative.
    #[arg(long, help_heading = "Verification")]
    pub verify_llm: bool,

    // --- LLM judge config: providers are used only by --verify-llm ---
    /// LLM provider for the experimental --verify-llm judge.
    #[arg(long, value_enum, help_heading = "LLM judge (experimental)")]
    pub provider: Option<llm::ProviderKind>,

    /// LLM model name for --verify-llm.
    #[arg(long, help_heading = "LLM judge (experimental)")]
    pub model: Option<String>,

    /// Override the provider base URL, useful for LM Studio, Ollama, and llama.cpp.
    #[arg(long, help_heading = "LLM judge (experimental)")]
    pub base_url: Option<String>,

    /// Maximum number of tokens the --verify-llm judge may generate.
    #[arg(long, help_heading = "LLM judge (experimental)")]
    pub max_output_tokens: Option<u64>,

    /// Per-request LLM HTTP timeout in seconds for --verify-llm.
    #[arg(long, help_heading = "LLM judge (experimental)")]
    pub timeout_seconds: Option<u64>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReportFormat {
    Human,
    Json,
}

pub fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    run_with_cli(cli)
}

pub fn run_with_cli(cli: Cli) -> Result<(), AppError> {
    let input = fs::read_to_string(&cli.path)?;

    if let Some(candidate_path) = cli.verify.clone() {
        return run_verify(&cli, &input, &candidate_path);
    }

    let analysis = skill::analyze(&cli.path, &input);
    let minify_options = minifier_options_from_cli(&cli);
    let minified = minifier::minify_skill_with_options(&input, &minify_options);
    let changed = minified != input;

    if cli.diff {
        print_diff(&input, &minified);
        return Ok(());
    }

    if cli.write {
        if changed {
            fs::write(&cli.path, minified)?;
        }
        print_write_result(&cli.path, changed);
        return Ok(());
    }

    let report = skill::Report::from_analysis(analysis, &minified, changed);

    match cli.report {
        ReportFormat::Human => print_human_report(&report),
        ReportFormat::Json => println!("{}", report.to_json()),
    }

    if cli.check && (changed || report.has_errors()) {
        return Err(AppError::new("check failed", 1));
    }

    Ok(())
}

fn run_verify(cli: &Cli, original: &str, candidate_path: &Path) -> Result<(), AppError> {
    let candidate = fs::read_to_string(candidate_path)?;
    let report = fidelity::verify(original, &candidate);

    match cli.report {
        ReportFormat::Human => print_fidelity_report(&report, &cli.path, candidate_path),
        ReportFormat::Json => println!("{}", report.to_json()),
    }

    // Experimental LLM-as-judge layer over the residue. Advisory only: it never
    // changes the deterministic exit code below.
    if cli.verify_llm && !report.missing.is_empty() {
        run_llm_judge(cli, &candidate, &report);
    }

    if !report.missing.is_empty() {
        return Err(AppError::new("fidelity check failed", 1));
    }
    Ok(())
}

/// Cap on how many missing atoms are sent to the judge in one call, to bound cost
/// on paraphrase-heavy candidates where nearly everything is "missing" verbatim.
const MAX_JUDGE_ITEMS: usize = 80;

fn run_llm_judge(cli: &Cli, candidate: &str, report: &fidelity::FidelityReport) {
    let config = match llm::LlmConfig::from_cli(
        cli.provider,
        cli.model.clone(),
        cli.base_url.clone(),
        cli.max_output_tokens,
        cli.timeout_seconds,
    ) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("warning: LLM judge skipped: {}", error.message);
            return;
        }
    };

    let judged = report.missing.len().min(MAX_JUDGE_ITEMS);
    let items: Vec<String> = report
        .missing
        .iter()
        .take(judged)
        .map(|atom| atom.text.clone())
        .collect();

    match llm::judge_missing(&config, candidate, &items) {
        Ok(verdicts) => {
            print_judge_report(&report.missing[..judged], &verdicts, report.missing.len())
        }
        Err(error) => eprintln!("warning: LLM judge failed: {}", error.message),
    }
}

fn print_judge_report(
    atoms: &[fidelity::Atom],
    verdicts: &[llm::JudgeVerdict],
    total_missing: usize,
) {
    use std::collections::HashMap;
    let by_index: HashMap<usize, &llm::JudgeVerdict> =
        verdicts.iter().map(|v| (v.index, v)).collect();

    let mut preserved = 0usize;
    let mut weakened = 0usize;
    let mut lost = 0usize;

    println!("\nLLM judge (experimental, advisory — deterministic result stays authoritative):");
    for (idx, atom) in atoms.iter().enumerate() {
        let (verdict, evidence) = match by_index.get(&idx) {
            Some(v) => (v.verdict.as_str(), v.evidence.as_str()),
            None => ("unknown", ""),
        };
        let mark = match verdict {
            "preserved" => {
                preserved += 1;
                "≈ preserved"
            }
            "weakened" => {
                weakened += 1;
                "⚠ weakened"
            }
            "lost" => {
                lost += 1;
                "✗ lost"
            }
            _ => "? unknown",
        };
        println!("  [{}] {}", mark, atom.text);
        if !evidence.is_empty() {
            println!("        ↳ {}", evidence);
        }
    }
    if atoms.len() < total_missing {
        println!(
            "  (judged {} of {} missing atoms; raise the cap for the rest)",
            atoms.len(),
            total_missing
        );
    }
    println!(
        "  judge summary: {} paraphrased-equivalent, {} weakened, {} lost",
        preserved, weakened, lost
    );
}

fn print_fidelity_report(
    report: &fidelity::FidelityReport,
    original_path: &Path,
    candidate_path: &Path,
) {
    println!(
        "fidelity {} -> {}",
        original_path.display(),
        candidate_path.display()
    );
    println!(
        "  {}/{} must-preserve atoms retained ({} missing)",
        report.preserved,
        report.total,
        report.missing.len()
    );

    if report.missing.is_empty() {
        println!("  ✅ no dropped rules, sections, or code blocks");
        return;
    }

    // Group missing atoms by kind for a scannable report.
    for kind in [
        fidelity::AtomKind::FrontmatterKey,
        fidelity::AtomKind::Heading,
        fidelity::AtomKind::Rule,
        fidelity::AtomKind::CodeBlock,
    ] {
        let group: Vec<_> = report
            .missing
            .iter()
            .filter(|atom| atom.kind == kind)
            .collect();
        if group.is_empty() {
            continue;
        }
        println!("  ❌ missing {} ({}):", kind.plural(), group.len());
        for atom in group {
            println!("     - {}", atom.text);
        }
    }
}

fn minifier_options_from_cli(cli: &Cli) -> minifier::MinifyOptions {
    minifier::MinifyOptions {
        strip_changelog: cli.strip_changelog,
        keep_latest: cli.keep_latest,
        strip_examples: cli.strip_examples,
        strip_meta_prose: cli.strip_meta_prose,
        strip_nonessential: cli.strip_nonessential,
        runtime_only: cli.runtime_only,
    }
}

fn print_write_result(path: &Path, changed: bool) {
    if changed {
        println!("updated {}", path.display());
    } else {
        println!("already clean {}", path.display());
    }
}

fn print_diff(before: &str, after: &str) {
    let diff = similar::TextDiff::from_lines(before, after);
    print!(
        "{}",
        diff.unified_diff()
            .header("before/SKILL.md", "after/SKILL.md")
    );
}

fn print_human_report(report: &skill::Report) {
    println!("skill-compress report");
    println!("path: {}", report.path);
    println!("status: {}", report.status);
    println!(
        "lines: {} -> {}",
        report.metrics.before.lines, report.metrics.after.lines
    );
    println!(
        "chars: {} -> {}",
        report.metrics.before.chars, report.metrics.after.chars
    );
    println!(
        "estimated tokens: {} -> {}",
        report.metrics.before.estimated_tokens, report.metrics.after.estimated_tokens
    );

    if report.diagnostics.is_empty() {
        println!("diagnostics: none");
        return;
    }

    println!("diagnostics:");
    for diagnostic in &report.diagnostics {
        let line = diagnostic
            .line
            .map(|value| format!(" line {}", value))
            .unwrap_or_default();
        println!(
            "- {:?} [{}]{}: {}",
            diagnostic.severity, diagnostic.code, line, diagnostic.message
        );
        if let Some(suggestion) = &diagnostic.suggestion {
            println!("  suggestion: {}", suggestion);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonError<'a> {
    pub error: &'a str,
}
