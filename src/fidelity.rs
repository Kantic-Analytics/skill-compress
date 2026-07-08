//! Deterministic fidelity gate.
//!
//! Compression should never silently drop a rule, an acceptance line, a section,
//! or a code block. This module extracts the "must-preserve atoms" of an original
//! `SKILL.md` and checks that a candidate (deterministic, runtime-only, or LLM
//! rewrite) still contains each of them. It uses no LLM: matching is exact after a
//! light, semantics-preserving normalization, so it is cheap, repeatable, and CI
//! friendly. Paraphrased content is reported as missing on purpose — a verbatim
//! preservation gate treats reworded rules as unverified drift, not as equivalent.

use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;

/// Category of a must-preserve atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AtomKind {
    FrontmatterKey,
    Heading,
    Rule,
    CodeBlock,
}

impl AtomKind {
    /// Plural label for grouped reporting.
    pub fn plural(self) -> &'static str {
        match self {
            AtomKind::FrontmatterKey => "frontmatter keys",
            AtomKind::Heading => "sections",
            AtomKind::Rule => "rules",
            AtomKind::CodeBlock => "code blocks",
        }
    }
}

/// A single unit of content that a faithful compression must keep.
#[derive(Clone, Debug, Serialize)]
pub struct Atom {
    pub kind: AtomKind,
    /// Original human-readable text (for reporting).
    pub text: String,
    /// Normalized key used for matching (never shown).
    #[serde(skip)]
    pub key: String,
}

/// Result of comparing a candidate against an original.
#[derive(Debug, Serialize)]
pub struct FidelityReport {
    pub total: usize,
    pub preserved: usize,
    pub missing: Vec<Atom>,
}

impl FidelityReport {
    pub fn to_json(&self) -> String {
        let missing: Vec<_> = self
            .missing
            .iter()
            .map(|atom| json!({"kind": atom.kind, "text": atom.text}))
            .collect();
        serde_json::to_string_pretty(&json!({
            "total": self.total,
            "preserved": self.preserved,
            "missing_count": self.missing.len(),
            "missing": missing,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}

/// Compare `candidate` against `original` and report every must-preserve atom of
/// the original that the candidate does not contain.
pub fn verify(original: &str, candidate: &str) -> FidelityReport {
    let original_atoms = dedupe(extract_atoms(original));
    let candidate_keys: HashSet<(AtomKind, String)> = extract_atoms(candidate)
        .into_iter()
        .map(|atom| (atom.kind, atom.key))
        .collect();

    let total = original_atoms.len();
    let missing: Vec<Atom> = original_atoms
        .into_iter()
        .filter(|atom| !candidate_keys.contains(&(atom.kind, atom.key.clone())))
        .collect();

    FidelityReport {
        total,
        preserved: total - missing.len(),
        missing,
    }
}

/// Keep one atom per (kind, key). The original intentionally repeats some rules;
/// a candidate is faithful if it contains each *distinct* atom at least once.
fn dedupe(atoms: Vec<Atom>) -> Vec<Atom> {
    let mut seen: HashSet<(AtomKind, String)> = HashSet::new();
    let mut out = Vec::new();
    for atom in atoms {
        if seen.insert((atom.kind, atom.key.clone())) {
            out.push(atom);
        }
    }
    out
}

/// Extract every must-preserve atom from a skill document. Fence-aware: content
/// inside code fences is captured as a `CodeBlock` atom and never mined for
/// headings or rules.
fn extract_atoms(doc: &str) -> Vec<Atom> {
    let mut atoms = Vec::new();
    atoms.extend(frontmatter_keys(doc));

    let mut in_fence = false;
    let mut fence_token = "";
    let mut block: Vec<&str> = Vec::new();

    // Skip the frontmatter region so its `---` and keys are not re-mined as body.
    for line in body_lines(doc) {
        let trimmed = line.trim();

        if in_fence {
            if is_fence(trimmed) && fence_prefix(trimmed) == fence_token {
                atoms.push(code_block_atom(&block));
                block.clear();
                in_fence = false;
            } else {
                block.push(line);
            }
            continue;
        }

        if is_fence(trimmed) {
            in_fence = true;
            fence_token = fence_prefix(trimmed);
            continue;
        }

        if let Some(text) = heading_text(trimmed) {
            atoms.push(Atom {
                kind: AtomKind::Heading,
                text: text.to_string(),
                key: normalize(text),
            });
        } else if let Some(text) = bullet_text(trimmed) {
            if is_reference_only(text) {
                continue; // "See BR-001." is a pointer, not content.
            }
            atoms.push(Atom {
                kind: AtomKind::Rule,
                text: text.to_string(),
                key: normalize(text),
            });
        }
    }

    // An unterminated fence still yields its collected content.
    if in_fence && !block.is_empty() {
        atoms.push(code_block_atom(&block));
    }

    atoms
}

/// Lines after the YAML frontmatter block (if any).
fn body_lines(doc: &str) -> impl Iterator<Item = &str> {
    let mut lines = doc.lines().peekable();
    if matches!(lines.peek(), Some(&first) if first.trim() == "---") {
        lines.next();
        for line in lines.by_ref() {
            if line.trim() == "---" || line.trim() == "..." {
                break;
            }
        }
    }
    lines
}

/// Top-level YAML frontmatter keys as `FrontmatterKey` atoms.
fn frontmatter_keys(doc: &str) -> Vec<Atom> {
    let mut lines = doc.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }
    let mut keys = Vec::new();
    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.trim() == "---" || trimmed.trim() == "..." {
            break;
        }
        // A top-level key has no leading whitespace and a `key:` shape.
        if trimmed.starts_with(char::is_whitespace) {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once(':') {
            if !key.is_empty()
                && key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                keys.push(Atom {
                    kind: AtomKind::FrontmatterKey,
                    text: key.to_string(),
                    key: key.to_string(),
                });
            }
        }
    }
    keys
}

fn code_block_atom(block: &[&str]) -> Atom {
    let normalized = block
        .iter()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    Atom {
        kind: AtomKind::CodeBlock,
        text: first_line_snippet(&normalized),
        key: normalized.trim().to_string(),
    }
}

fn is_fence(trimmed: &str) -> bool {
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// The fence marker without its language tag (``` or ~~~).
fn fence_prefix(trimmed: &str) -> &'static str {
    if trimmed.starts_with("~~~") {
        "~~~"
    } else {
        "```"
    }
}

fn heading_text(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix('#')?;
    let rest = rest.trim_start_matches('#');
    let text = rest.trim();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn bullet_text(trimmed: &str) -> Option<&str> {
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            let text = rest.trim();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

/// A bullet that only points at another rule, e.g. `See BR-001.`
fn is_reference_only(text: &str) -> bool {
    let core = text.trim().trim_end_matches('.');
    let rest = match core.strip_prefix("See ") {
        Some(rest) => rest,
        None => return false,
    };
    is_reference_marker(rest)
}

/// True for a bare `ABBR-123` reference identifier (uppercase letters, dash, digits).
fn is_reference_marker(token: &str) -> bool {
    let Some((abbr, num)) = token.split_once('-') else {
        return false;
    };
    !abbr.is_empty()
        && abbr.chars().all(|c| c.is_ascii_uppercase())
        && !num.is_empty()
        && num.chars().all(|c| c.is_ascii_digit())
}

/// Drop a leading reference marker the deterministic minifier prepends to the
/// first occurrence of a duplicated rule, e.g. `[BR-001] Always update…` → `Always update…`
fn strip_reference_prefix(text: &str) -> &str {
    let trimmed = text.trim_start();
    let Some(rest) = trimmed.strip_prefix('[') else {
        return text;
    };
    let Some(end) = rest.find(']') else {
        return text;
    };
    if is_reference_marker(&rest[..end]) {
        rest[end + 1..].trim_start()
    } else {
        text
    }
}

/// Drop a trailing reference marker (the fixture's illustrative form), e.g.
/// `... changes. (BR-001)` → `... changes.`
fn strip_reference_marker(text: &str) -> &str {
    let trimmed = text.trim_end();
    let Some(stripped) = trimmed.strip_suffix(')') else {
        return text;
    };
    match stripped.rfind(" (") {
        Some(idx) if is_reference_marker(&stripped[idx + 2..]) => stripped[..idx].trim_end(),
        _ => text,
    }
}

/// Semantics-preserving normalization: absorb cosmetic differences (dash style,
/// whitespace, trailing period, the minifier's `(BR-xxx)` marker) while keeping
/// wording and identifiers significant, so paraphrases still register as misses.
fn normalize(text: &str) -> String {
    let dash_normalized: String = text
        .chars()
        .map(|c| {
            if c == '\u{2013}' || c == '\u{2014}' {
                '-'
            } else {
                c
            }
        })
        .collect();
    let without_prefix = strip_reference_prefix(&dash_normalized);
    let without_ref = strip_reference_marker(without_prefix);
    without_ref
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches('.')
        .trim()
        .to_string()
}

fn first_line_snippet(text: &str) -> String {
    let line = text.lines().next().unwrap_or_default().trim();
    if line.chars().count() > 60 {
        let truncated: String = line.chars().take(60).collect();
        format!("{truncated}…")
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGINAL: &str = "---\nname: s\ndescription: d\n---\n# Title\n\n## Inputs\n- A source file.\n- A rule.\n\n## Rules\n- A rule.\n- Keep `GEMINI_API_KEY` unchanged.\n\n```bash\nmake build\n```\n";

    #[test]
    fn identical_document_is_fully_preserved() {
        let report = verify(ORIGINAL, ORIGINAL);
        assert_eq!(report.missing.len(), 0);
        assert_eq!(report.preserved, report.total);
        assert!(report.total > 0);
    }

    #[test]
    fn dropped_heading_and_rule_are_reported() {
        // Candidate drops the whole Inputs section and one rule.
        let candidate = "---\nname: s\ndescription: d\n---\n# Title\n\n## Rules\n- A rule.\n\n```bash\nmake build\n```\n";
        let report = verify(ORIGINAL, candidate);
        let missing: Vec<_> = report
            .missing
            .iter()
            .map(|a| (a.kind, a.text.as_str()))
            .collect();
        assert!(missing.contains(&(AtomKind::Heading, "Inputs")));
        assert!(missing.contains(&(AtomKind::Rule, "A source file.")));
        assert!(missing.contains(&(AtomKind::Rule, "Keep `GEMINI_API_KEY` unchanged.")));
    }

    #[test]
    fn near_miss_rules_are_distinct_atoms() {
        let original = "---\nname: s\n---\n- Preserve rules even when repetitive.\n- Preserve rules when in multiple sections.\n";
        // Candidate keeps only the first wording.
        let candidate = "---\nname: s\n---\n- Preserve rules even when repetitive.\n";
        let report = verify(original, candidate);
        assert_eq!(report.missing.len(), 1);
        assert_eq!(
            report.missing[0].text,
            "Preserve rules when in multiple sections."
        );
    }

    #[test]
    fn deterministic_reference_markers_do_not_count_as_loss() {
        // The deterministic minifier appends "(BR-001)" to the first occurrence and
        // replaces later copies with "See BR-001." Both must satisfy the original.
        let original = "---\nname: s\n---\n- Always update docs.\n- Always update docs.\n";
        let candidate = "---\nname: s\n---\n- Always update docs. (BR-001)\n- See BR-001.\n";
        let report = verify(original, candidate);
        assert_eq!(
            report.missing.len(),
            0,
            "reference markers must normalize to the original rule"
        );
    }

    #[test]
    fn deterministic_prefix_marker_matches_original() {
        // The real minifier prepends "[BR-001] " to the first occurrence.
        let original = "---\nname: s\n---\n- Always update docs.\n- Always update docs.\n";
        let candidate = "---\nname: s\n---\n- [BR-001] Always update docs.\n- See BR-001.\n";
        let report = verify(original, candidate);
        assert_eq!(report.missing.len(), 0);
    }

    #[test]
    fn dash_and_whitespace_differences_are_absorbed() {
        let original = "---\nname: s\n---\n## Scenario 01 - Intake\n- Do   the   thing.\n";
        let candidate = "---\nname: s\n---\n## Scenario 01 \u{2013} Intake\n- Do the thing\n";
        let report = verify(original, candidate);
        assert_eq!(report.missing.len(), 0);
    }

    #[test]
    fn paraphrase_registers_as_missing() {
        let original = "---\nname: s\n---\n- Preserve required runtime instructions even when they appear repetitive.\n";
        let candidate =
            "---\nname: s\n---\n- Preserve required runtime instructions even when repeated.\n";
        let report = verify(original, candidate);
        assert_eq!(report.missing.len(), 1);
    }

    #[test]
    fn changed_code_block_is_reported() {
        let original = "---\nname: s\n---\n```bash\nmake build\n```\n";
        let candidate = "---\nname: s\n---\n```bash\nmake release\n```\n";
        let report = verify(original, candidate);
        assert!(report.missing.iter().any(|a| a.kind == AtomKind::CodeBlock));
    }
}
