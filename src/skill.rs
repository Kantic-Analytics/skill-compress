use std::collections::BTreeSet;
use std::path::Path;

use serde::Serialize;

use crate::minifier::minify_skill;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub lines: usize,
    pub chars: usize,
    pub words: usize,
    pub estimated_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct MetricsDelta {
    pub before: Metrics,
    pub after: Metrics,
}

#[derive(Debug)]
pub struct Analysis {
    pub path: String,
    pub metrics: Metrics,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub path: String,
    pub status: String,
    pub changed: bool,
    pub metrics: MetricsDelta,
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn from_analysis(analysis: Analysis, minified: &str, changed: bool) -> Self {
        let status = if analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            "error"
        } else if changed {
            "changed"
        } else {
            "ok"
        };

        Self {
            path: analysis.path,
            status: status.to_string(),
            changed,
            metrics: MetricsDelta {
                before: analysis.metrics,
                after: collect_metrics(minified),
            },
            diagnostics: analysis.diagnostics,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("report serialization should not fail")
    }
}

pub fn analyze(path: &Path, input: &str) -> Analysis {
    let mut diagnostics = Vec::new();
    let split = split_frontmatter(input);

    match &split {
        FrontmatterSplit::Present { yaml } => {
            validate_frontmatter(yaml, &mut diagnostics);
        }
        FrontmatterSplit::Missing => diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "frontmatter.missing".to_string(),
            message: "Missing YAML frontmatter.".to_string(),
            line: Some(1),
            suggestion: Some("Add name and description fields between --- delimiters.".to_string()),
        }),
    }

    detect_markdown_issues(input, &mut diagnostics);

    let minified = minify_skill(input);
    if minified != input {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            code: "body.minifiable".to_string(),
            message: "Deterministic cleanup would change this file.".to_string(),
            line: None,
            suggestion: Some("Run with --diff or --write to inspect/apply cleanup.".to_string()),
        });
    }

    Analysis {
        path: path.display().to_string(),
        metrics: collect_metrics(input),
        diagnostics,
    }
}

pub fn looks_like_skill(input: &str) -> bool {
    matches!(split_frontmatter(input), FrontmatterSplit::Present { .. })
        && input.contains("description:")
}

pub fn collect_metrics(input: &str) -> Metrics {
    let chars = input.chars().count();
    Metrics {
        lines: input.lines().count(),
        chars,
        words: input.split_whitespace().count(),
        estimated_tokens: chars.div_ceil(4),
    }
}

enum FrontmatterSplit<'a> {
    Present { yaml: &'a str },
    Missing,
}

fn split_frontmatter(input: &str) -> FrontmatterSplit<'_> {
    if !input.starts_with("---\n") && !input.starts_with("---\r\n") {
        return FrontmatterSplit::Missing;
    }

    let mut offset = 0;
    let mut lines = input.split_inclusive('\n');
    let first = lines.next().unwrap_or_default();
    offset += first.len();
    let yaml_start = offset;

    for line in lines {
        let trimmed = line.trim();
        let current_offset = offset;
        offset += line.len();
        if trimmed == "---" || trimmed == "..." {
            return FrontmatterSplit::Present {
                yaml: &input[yaml_start..current_offset],
            };
        }
    }

    FrontmatterSplit::Missing
}

fn validate_frontmatter(yaml: &str, diagnostics: &mut Vec<Diagnostic>) {
    let parsed: serde_yaml::Value = match serde_yaml::from_str(yaml) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "frontmatter.invalid_yaml".to_string(),
                message: format!("Invalid YAML frontmatter: {}", error),
                line: error.location().map(|location| location.line() + 1),
                suggestion: Some("Fix YAML syntax before running --write.".to_string()),
            });
            return;
        }
    };

    let Some(mapping) = parsed.as_mapping() else {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "frontmatter.not_mapping".to_string(),
            message: "Frontmatter must be a YAML mapping.".to_string(),
            line: Some(1),
            suggestion: Some("Use key-value fields such as name and description.".to_string()),
        });
        return;
    };

    let keys: BTreeSet<String> = mapping
        .keys()
        .filter_map(|key| key.as_str().map(ToOwned::to_owned))
        .collect();

    if !keys.contains("name") {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "frontmatter.name_missing".to_string(),
            message: "Missing frontmatter field: name.".to_string(),
            line: Some(2),
            suggestion: Some("Add a short stable skill name.".to_string()),
        });
    }

    let description = mapping
        .get(serde_yaml::Value::String("description".to_string()))
        .and_then(|value| value.as_str());

    match description {
        Some(value) if value.chars().count() > 1024 => diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "description.too_long".to_string(),
            message: "Description is longer than 1024 characters.".to_string(),
            line: Some(3),
            suggestion: Some("Keep the trigger description concise and specific.".to_string()),
        }),
        Some(value) if !value.to_lowercase().contains("use") => diagnostics.push(Diagnostic {
            severity: Severity::Info,
            code: "description.trigger_hint".to_string(),
            message: "Description may not clearly describe when to use the skill.".to_string(),
            line: Some(3),
            suggestion: Some("Prefer a trigger phrase such as 'Use when ...'.".to_string()),
        }),
        Some(_) => {}
        None => diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "frontmatter.description_missing".to_string(),
            message: "Missing frontmatter field: description.".to_string(),
            line: Some(2),
            suggestion: Some("Add a concise use-case description.".to_string()),
        }),
    }
}

fn detect_markdown_issues(input: &str, diagnostics: &mut Vec<Diagnostic>) {
    let metrics = collect_metrics(input);

    if metrics.lines > 500 {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "body.too_many_lines".to_string(),
            message: format!(
                "File has {} lines; recommended limit is about 500.",
                metrics.lines
            ),
            line: None,
            suggestion: Some("Move detailed reference material into references/.".to_string()),
        });
    }

    if metrics.estimated_tokens > 5000 {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "body.too_many_tokens".to_string(),
            message: format!(
                "Estimated token count is {}; recommended limit is about 5000.",
                metrics.estimated_tokens
            ),
            line: None,
            suggestion: Some(
                "Shorten repeated prose or split optional details into separate files.".to_string(),
            ),
        });
    }

    let mut current_heading: Option<(String, usize, usize)> = None;

    for (index, line) in input.lines().enumerate() {
        if line.starts_with('#') {
            if let Some((heading, start, count)) = current_heading.take() {
                if count > 80 {
                    diagnostics.push(long_section_diagnostic(&heading, start, count));
                }
            }
            current_heading = Some((line.trim().to_string(), index + 1, 0));
        } else if let Some((_, _, count)) = current_heading.as_mut() {
            *count += 1;
        }
    }

    if let Some((heading, start, count)) = current_heading {
        if count > 80 {
            diagnostics.push(long_section_diagnostic(&heading, start, count));
        }
    }
}

fn long_section_diagnostic(heading: &str, start: usize, count: usize) -> Diagnostic {
    Diagnostic {
        severity: Severity::Info,
        code: "section.possible_reference".to_string(),
        message: format!("Section '{}' spans {} lines.", heading, count),
        line: Some(start),
        suggestion: Some("Consider moving deep reference content to references/.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze, collect_metrics, looks_like_skill};
    use std::path::Path;

    #[test]
    fn analyzes_valid_skill_frontmatter() {
        let input = "---\nname: test\ndescription: Use when testing.\n---\n\n# Body\n";
        let report = analyze(Path::new("SKILL.md"), input);
        assert!(report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "frontmatter.invalid_yaml"));
        assert!(looks_like_skill(input));
    }

    #[test]
    fn estimates_tokens_from_chars() {
        let metrics = collect_metrics("12345678");
        assert_eq!(metrics.estimated_tokens, 2);
    }
}
