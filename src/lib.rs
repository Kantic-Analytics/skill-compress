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

    /// Preview what --write would do without modifying the file. Reports whether the
    /// file would change and by how much; takes precedence over --write so a stray
    /// --write never mutates when --dry-run is set.
    #[arg(long, help_heading = "Output")]
    pub dry_run: bool,

    /// Overwrite even when the target is a tracked file with uncommitted git changes.
    /// Without this, --write refuses to clobber unsaved work (exit code 5).
    #[arg(long, visible_alias = "allow-dirty", help_heading = "Output")]
    pub force: bool,

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

    if cli.dry_run {
        print_dry_run_result(&cli.path, &input, &minified, changed);
        return Ok(());
    }

    if cli.write {
        if changed {
            if !cli.force {
                if let Some(reason) = write_guard_reason(&cli.path) {
                    return Err(AppError::new(reason, 5));
                }
            }
            atomic_write(&cli.path, &minified)?;
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

/// Refuse to overwrite a tracked file that has uncommitted git changes, so `--write`
/// can never clobber unsaved work the user could not otherwise recover. Best-effort:
/// if `git` is unavailable, the path is not in a repo, or the file is untracked or
/// ignored, we return `None` (allowed) — git has nothing to restore in those cases,
/// and the write is the deterministic cleanup the user explicitly asked for.
fn write_guard_reason(path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "--"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None; // not a git work tree, or git unavailable: cannot check.
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let code = porcelain_block_code(&stdout)?;
    Some(format!(
        "{} has uncommitted git changes (status {code}); commit or stash them, or pass --force to overwrite",
        path.display()
    ))
}

/// Given `git status --porcelain -- <path>` output, return the two-letter status code
/// when a write should be blocked. Empty output (clean or ignored) and untracked
/// (`??`) are safe and yield `None`; any other status (modified, staged, renamed, …)
/// is uncommitted work and yields its code.
fn porcelain_block_code(stdout: &str) -> Option<String> {
    let line = stdout.lines().next()?;
    let code = line.get(..2).unwrap_or("");
    let trimmed = code.trim();
    if trimmed.is_empty() || code == "??" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Write `contents` to `path` atomically: write a sibling temp file on the same
/// filesystem, then rename it over the target. A crash or panic mid-write can then
/// never leave a half-written or truncated SKILL.md in place.
fn atomic_write(path: &Path, contents: &str) -> Result<(), AppError> {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "skill".to_string());
    let tmp_name = format!(".{file_name}.skill-compress-{}.tmp", std::process::id());
    let tmp_path = match path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        Some(parent) => parent.join(tmp_name),
        None => PathBuf::from(tmp_name),
    };

    fs::write(&tmp_path, contents)?;
    if let Err(error) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::from(error));
    }
    Ok(())
}

/// Report what `--write` would do, without touching the file.
fn print_dry_run_result(path: &Path, before: &str, after: &str, changed: bool) {
    if !changed {
        println!(
            "dry-run: {} already clean; --write would make no changes",
            path.display()
        );
        return;
    }
    println!(
        "dry-run: --write would update {} ({} -> {} lines, {} -> {} chars)",
        path.display(),
        before.lines().count(),
        after.lines().count(),
        before.chars().count(),
        after.chars().count(),
    );
    println!("  no file was modified; drop --dry-run to apply, or use --diff to see the changes");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_previews_without_writing_then_write_applies() {
        // A file the deterministic minifier would change (extra blank lines).
        let original = "---\nname: s\ndescription: d\n---\n\n\n# Title\n\n\nBody\n";
        let path = std::env::temp_dir().join("skill_compress_dry_run_test.md");
        std::fs::write(&path, original).unwrap();

        // --dry-run (even alongside --write) must leave the file byte-for-byte intact.
        let cli = Cli::parse_from([
            "skill-compress",
            path.to_str().unwrap(),
            "--write",
            "--dry-run",
        ]);
        run_with_cli(cli).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            original,
            "--dry-run must not modify the file"
        );

        // A real --write does change it.
        let cli = Cli::parse_from(["skill-compress", path.to_str().unwrap(), "--write"]);
        run_with_cli(cli).unwrap();
        assert_ne!(
            std::fs::read_to_string(&path).unwrap(),
            original,
            "--write should apply the minified output"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn porcelain_code_blocks_only_uncommitted_tracked_changes() {
        // Clean/ignored (no line) and untracked are safe to overwrite.
        assert_eq!(porcelain_block_code(""), None);
        assert_eq!(porcelain_block_code("?? brand-new.md\n"), None);
        // Any real status (unstaged, staged, added, renamed) blocks with its code.
        assert_eq!(porcelain_block_code(" M SKILL.md\n").as_deref(), Some("M"));
        assert_eq!(porcelain_block_code("M  SKILL.md\n").as_deref(), Some("M"));
        assert_eq!(porcelain_block_code("A  SKILL.md\n").as_deref(), Some("A"));
        assert_eq!(porcelain_block_code("R  a -> b\n").as_deref(), Some("R"));
    }

    #[test]
    fn atomic_write_replaces_contents_and_leaves_no_temp() {
        let dir = std::env::temp_dir();
        let path = dir.join("skill_compress_atomic_test.md");
        std::fs::write(&path, "old contents").unwrap();

        atomic_write(&path, "new contents\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new contents\n");

        // No leftover temp sibling from the atomic rename.
        let leftover = std::fs::read_dir(&dir).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("skill_compress_atomic_test.md.skill-compress-")
        });
        assert!(!leftover, "atomic_write left a temp file behind");

        let _ = std::fs::remove_file(&path);
    }
}
