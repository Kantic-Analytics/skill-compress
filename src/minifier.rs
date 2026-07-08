use std::collections::{HashMap, HashSet};

pub fn minify_skill(input: &str) -> String {
    minify_skill_with_options(input, &MinifyOptions::default())
}

#[derive(Debug, Default, Clone)]
pub struct MinifyOptions {
    pub strip_changelog: bool,
    pub keep_latest: Option<usize>,
    pub strip_examples: bool,
    pub strip_meta_prose: bool,
    pub strip_nonessential: bool,
    pub runtime_only: bool,
}

impl MinifyOptions {
    fn effective_strip_changelog(&self) -> bool {
        self.strip_changelog || self.strip_nonessential || self.runtime_only
    }

    fn effective_strip_examples(&self) -> bool {
        self.strip_examples || self.strip_nonessential || self.runtime_only
    }

    fn effective_strip_meta_prose(&self) -> bool {
        self.strip_meta_prose || self.strip_nonessential || self.runtime_only
    }
}

pub fn minify_skill_with_options(input: &str, options: &MinifyOptions) -> String {
    let mut output = Vec::new();
    let mut in_fence = false;
    let mut previous_blank = false;

    for raw_line in input.lines() {
        let trimmed_start = raw_line.trim_start();
        let is_fence = trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~");

        if in_fence {
            output.push(raw_line.to_string());
            if is_fence {
                in_fence = false;
            }
            previous_blank = false;
            continue;
        }

        let line = raw_line.trim_end();

        if is_single_line_html_comment(line) {
            continue;
        }

        if line.trim().is_empty() {
            if !previous_blank {
                output.push(String::new());
            }
            previous_blank = true;
            continue;
        }

        output.push(minify_markdown_line(line));
        previous_blank = false;

        if is_fence {
            in_fence = true;
        }
    }

    output = remove_decorative_blank_lines(output);
    output = reference_duplicate_fenced_blocks(output);
    output = reference_duplicate_business_rules(output);
    output = strip_optional_sections(output, options);

    while output.last().is_some_and(|line| line.is_empty()) {
        output.pop();
    }

    let mut result = output.join("\n");
    result.push('\n');
    result
}

fn strip_optional_sections(lines: Vec<String>, options: &MinifyOptions) -> Vec<String> {
    let mut output = Vec::with_capacity(lines.len());
    let mut index = 0;
    let mut in_fence = false;

    while index < lines.len() {
        let line = &lines[index];
        if is_fence_line(line) {
            in_fence = !in_fence;
            output.push(line.clone());
            index += 1;
            continue;
        }

        if !in_fence && is_atx_heading(line) {
            let title = heading_title(line);
            let strip_kind = section_strip_kind(&title, options);

            if let Some(kind) = strip_kind {
                let level = heading_level(line);
                let section_end = find_section_end(&lines, index + 1, level);

                match kind {
                    StripKind::Changelog => {
                        if let Some(keep_latest) = options.keep_latest {
                            output.push(line.clone());
                            output.extend(keep_latest_list_items(
                                &lines[index + 1..section_end],
                                keep_latest,
                            ));
                        }
                    }
                    StripKind::Remove => {}
                }

                index = section_end;
                continue;
            }
        }

        output.push(line.clone());
        index += 1;
    }

    remove_decorative_blank_lines(output)
}

#[derive(Clone, Copy)]
enum StripKind {
    Changelog,
    Remove,
}

fn section_strip_kind(title: &str, options: &MinifyOptions) -> Option<StripKind> {
    let normalized = normalize_heading_title(title);

    if options.effective_strip_changelog() && is_changelog_title(&normalized) {
        return Some(StripKind::Changelog);
    }

    if options.effective_strip_examples() && is_examples_title(&normalized) {
        return Some(StripKind::Remove);
    }

    if options.effective_strip_meta_prose() && is_meta_prose_title(&normalized) {
        return Some(StripKind::Remove);
    }

    None
}

fn keep_latest_list_items(section_lines: &[String], keep_latest: usize) -> Vec<String> {
    if keep_latest == 0 {
        return Vec::new();
    }

    let mut kept = Vec::new();
    let mut count = 0;

    for line in section_lines {
        if line.trim_start().starts_with("- ") {
            if count >= keep_latest {
                continue;
            }
            kept.push(line.clone());
            count += 1;
        } else if count > 0 && line.starts_with("  ") {
            kept.push(line.clone());
        }
    }

    kept
}

fn find_section_end(lines: &[String], mut index: usize, level: usize) -> usize {
    let mut in_fence = false;

    while index < lines.len() {
        let line = &lines[index];
        if is_fence_line(line) {
            in_fence = !in_fence;
        } else if !in_fence && is_atx_heading(line) && heading_level(line) <= level {
            break;
        }
        index += 1;
    }

    index
}

fn heading_level(line: &str) -> usize {
    line.trim_start()
        .chars()
        .take_while(|value| *value == '#')
        .count()
}

fn heading_title(line: &str) -> String {
    let trimmed = line.trim_start();
    let level = heading_level(trimmed);
    trimmed[level..].trim().trim_matches('#').trim().to_string()
}

fn normalize_heading_title(title: &str) -> String {
    title
        .trim_matches('*')
        .to_lowercase()
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_changelog_title(title: &str) -> bool {
    matches!(
        title,
        "changelog" | "change log" | "release notes" | "historique" | "version history"
    )
}

fn is_examples_title(title: &str) -> bool {
    matches!(
        title,
        "examples"
            | "example"
            | "avoid"
            | "prefer"
            | "before after"
            | "before / after"
            | "banned"
            | "preferred"
            | "preferred colon technical consulting default"
    )
}

fn is_meta_prose_title(title: &str) -> bool {
    matches!(
        title,
        "reference"
            | "what it is"
            | "general principle"
            | "general philosophy"
            | "philosophy"
            | "reasoning"
            | "changelog"
    )
}

fn minify_markdown_line(line: &str) -> String {
    if is_long_horizontal_rule(line) {
        return "---".to_string();
    }

    if is_atx_heading(line) {
        return line.replace("**", "");
    }

    line.to_string()
}

fn reference_duplicate_business_rules(lines: Vec<String>) -> Vec<String> {
    // Count and rewrite only outside fenced code blocks. A rule that appears inside
    // a ```markdown example must stay verbatim (fence contents are never altered),
    // and its in-fence copies must not inflate the duplicate count of a prose rule.
    let mut counts: HashMap<String, usize> = HashMap::new();

    let mut in_fence = false;
    for line in &lines {
        if is_fence_line(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(rule) = business_rule_text(line) {
            *counts.entry(rule.to_string()).or_default() += 1;
        }
    }

    let mut ids: HashMap<String, String> = HashMap::new();
    let mut first_seen: HashSet<String> = HashSet::new();
    let mut next_id = 1;
    let mut in_fence = false;

    lines
        .into_iter()
        .map(|line| {
            if is_fence_line(&line) {
                in_fence = !in_fence;
                return line;
            }
            if in_fence {
                return line;
            }

            let Some(rule) = business_rule_text(&line).map(ToOwned::to_owned) else {
                return line;
            };

            if counts.get(&rule).copied().unwrap_or_default() < 2 {
                return line;
            }

            let id = ids.entry(rule.clone()).or_insert_with(|| {
                let id = format!("BR-{next_id:03}");
                next_id += 1;
                id
            });

            if first_seen.insert(rule) {
                add_rule_id(&line, id)
            } else {
                replace_rule_with_reference(&line, id)
            }
        })
        .collect()
}

fn business_rule_text(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rule = trimmed.strip_prefix("- ")?;

    if rule.chars().count() < 40 {
        return None;
    }

    let lower = rule.to_lowercase();
    let is_rule = lower.starts_with("always ")
        || lower.starts_with("never ")
        || lower.starts_with("do not ")
        || lower.starts_with("use ")
        || lower.starts_with("prefer ")
        || lower.starts_with("avoid ")
        || lower.contains(" must ")
        || lower.contains(" should ");

    is_rule.then_some(rule)
}

fn add_rule_id(line: &str, id: &str) -> String {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    format!(
        "{indent}- [{id}] {}",
        rest.trim_start().trim_start_matches("- ")
    )
}

fn replace_rule_with_reference(line: &str, id: &str) -> String {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, _) = line.split_at(indent_len);
    format!("{indent}- See {id}.")
}

fn remove_decorative_blank_lines(lines: Vec<String>) -> Vec<String> {
    let mut output = Vec::with_capacity(lines.len());
    // Fence-aware: blank lines inside a code fence are content, never decorative.
    // Without this guard a blank line next to a fenced line that *looks* like a
    // heading or rule (e.g. a Python `# comment` or a `---` divider in a snippet)
    // would be stripped, silently altering the code block.
    let mut in_fence = false;

    for (index, line) in lines.iter().enumerate() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            output.push(line.clone());
            continue;
        }

        if !in_fence && line.is_empty() {
            let previous = output.last().map(String::as_str).unwrap_or_default();
            let next = lines.get(index + 1).map(String::as_str).unwrap_or_default();

            if is_atx_heading(previous)
                || is_horizontal_rule(previous)
                || is_atx_heading(next)
                || is_horizontal_rule(next)
            {
                continue;
            }
        }

        output.push(line.clone());
    }

    output
}

fn reference_duplicate_fenced_blocks(lines: Vec<String>) -> Vec<String> {
    let mut output = Vec::with_capacity(lines.len());
    let mut seen_blocks: Vec<Vec<String>> = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if !is_fence_line(&lines[index]) {
            output.push(lines[index].clone());
            index += 1;
            continue;
        }

        let start = index;
        index += 1;

        while index < lines.len() && !is_fence_line(&lines[index]) {
            index += 1;
        }

        if index < lines.len() {
            index += 1;
        }

        let block = lines[start..index].to_vec();
        if seen_blocks.iter().any(|seen| seen == &block) {
            output.push("See earlier identical example block.".to_string());
        } else {
            seen_blocks.push(block.clone());
            output.extend(block);
        }
    }

    output
}

fn is_atx_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|value| *value == '#').count();
    (1..=6).contains(&hashes) && trimmed.chars().nth(hashes) == Some(' ')
}

fn is_fence_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn is_long_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() > 3 && trimmed.chars().all(|value| value == '-')
}

fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "---"
        || (trimmed.len() >= 3
            && (trimmed.chars().all(|value| value == '-')
                || trimmed.chars().all(|value| value == '*')
                || trimmed.chars().all(|value| value == '_')))
}

fn is_single_line_html_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<!--") && trimmed.ends_with("-->")
}

#[cfg(test)]
mod tests {
    use super::{minify_skill, minify_skill_with_options, MinifyOptions};

    #[test]
    fn minifier_is_idempotent() {
        let input = "---\nname: test\n---\n\n\nBody  \n\n\n";
        let once = minify_skill(input);
        let twice = minify_skill(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn preserves_fenced_code_contents() {
        let input = "Before  \n\n```rust\nfn main() {   \n}\n```\n\nAfter\n";
        let output = minify_skill(input);
        assert!(output.contains("fn main() {   \n"));
        assert!(output.contains("Before\n"));
    }

    #[test]
    fn preserves_blank_lines_inside_fences_next_to_comment_lines() {
        // A blank line inside a fence sits next to `# Read a PDF`, which looks like an
        // ATX heading. The decorative-blank pass must not strip it: fenced content is
        // never altered, and dropping it would register as a fidelity loss.
        let input = "Intro\n\n```python\nfrom pypdf import PdfReader\n\n# Read a PDF\nreader = PdfReader(\"x.pdf\")\n```\n";
        let output = minify_skill(input);
        assert!(
            output.contains("from pypdf import PdfReader\n\n# Read a PDF"),
            "blank line inside the fence was stripped:\n{output}"
        );
    }

    #[test]
    fn preserves_blank_line_before_divider_inside_fence() {
        // A `---` line inside a fence is a code divider, not a Markdown rule.
        let input = "Intro\n\n```text\nabove\n\n---\nbelow\n```\n";
        let output = minify_skill(input);
        assert!(
            output.contains("above\n\n---\nbelow"),
            "blank line inside the fence was stripped:\n{output}"
        );
    }

    #[test]
    fn minifies_decorative_markdown() {
        let input = "# **Title**\n\n------\n\n## **Next**\n";
        let output = minify_skill(input);
        assert_eq!(output, "# Title\n---\n## Next\n");
    }

    #[test]
    fn references_duplicate_fenced_blocks() {
        let input = "```text\nsame\n```\n\nOther\n\n```text\nsame\n```\n";
        let output = minify_skill(input);
        assert_eq!(
            output,
            "```text\nsame\n```\n\nOther\n\nSee earlier identical example block.\n"
        );
    }

    #[test]
    fn references_duplicate_business_rules() {
        let input = "- Always update project documentation before committing changes.\n\n- Something else.\n\n- Always update project documentation before committing changes.\n";
        let output = minify_skill(input);
        assert_eq!(
            output,
            "- [BR-001] Always update project documentation before committing changes.\n\n- Something else.\n\n- See BR-001.\n"
        );
    }

    #[test]
    fn business_rule_pass_never_edits_fence_content() {
        // A qualifying rule (>=40 chars, starts with "always") duplicated in prose is
        // referenced, but its copy inside a fence must stay byte-for-byte.
        let rule = "Always update project documentation before committing changes.";
        let input = format!("- {rule}\n\n- {rule}\n\n```text\n- {rule}\n```\n");
        let output = minify_skill(&input);
        // The prose duplicate is still referenced (the pass still does its job)...
        assert!(
            output.contains("BR-001"),
            "prose duplicate should be referenced, got:\n{output}"
        );
        // ...but the fenced block is untouched (no reference marker leaked inside it).
        let fence = output
            .split("```text\n")
            .nth(1)
            .and_then(|rest| rest.split("\n```").next())
            .expect("fenced block present");
        assert_eq!(fence, format!("- {rule}"), "fence content was altered");
    }

    #[test]
    fn in_fence_copies_do_not_inflate_duplicate_count() {
        // A qualifying rule appearing once in prose and twice inside a fence is NOT a
        // prose duplicate, so the prose occurrence must be left untouched.
        let rule = "Never send source content to an LLM unless explicitly enabled.";
        let input = format!("- {rule}\n\n```text\n- {rule}\n- {rule}\n```\n");
        let output = minify_skill(&input);
        assert!(
            !output.contains("BR-001"),
            "should not reference, got:\n{output}"
        );
        assert!(output.starts_with(&format!("- {rule}\n")));
    }

    #[test]
    fn strips_changelog_sections() {
        let input = "# Skill\n## Changelog\n- v2\n- v1\n## Rules\n- Keep this.\n";
        let output = minify_skill_with_options(
            input,
            &MinifyOptions {
                strip_changelog: true,
                ..MinifyOptions::default()
            },
        );
        assert_eq!(output, "# Skill\n## Rules\n- Keep this.\n");
    }

    #[test]
    fn keeps_latest_changelog_items() {
        let input = "# Skill\n## Changelog\n- v2\n- v1\n## Rules\n- Keep this.\n";
        let output = minify_skill_with_options(
            input,
            &MinifyOptions {
                strip_changelog: true,
                keep_latest: Some(1),
                ..MinifyOptions::default()
            },
        );
        assert_eq!(
            output,
            "# Skill\n## Changelog\n- v2\n## Rules\n- Keep this.\n"
        );
    }

    #[test]
    fn strips_example_sections() {
        let input =
            "# Skill\n## Rule\nText.\n### Examples\n```text\nsample\n```\n### Exceptions\nKeep.\n";
        let output = minify_skill_with_options(
            input,
            &MinifyOptions {
                strip_examples: true,
                ..MinifyOptions::default()
            },
        );
        assert_eq!(output, "# Skill\n## Rule\nText.\n### Exceptions\nKeep.\n");
    }

    #[test]
    fn runtime_only_strips_history_examples_and_meta_prose() {
        let input = "# Skill\n## Changelog\n- v2\n## Reference\nLink.\n## Rules\nKeep.\n### Examples\nExample.\n";
        let output = minify_skill_with_options(
            input,
            &MinifyOptions {
                runtime_only: true,
                ..MinifyOptions::default()
            },
        );
        assert_eq!(output, "# Skill\n## Rules\nKeep.\n");
    }
}
