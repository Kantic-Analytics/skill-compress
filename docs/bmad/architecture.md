# Architecture

## Technology Choice

Use Rust for the CLI.

Reasons:

- fast startup;
- single static-ish binary distribution;
- strong typed error handling;
- good CLI ecosystem;
- suitable Markdown/YAML parsing libraries;
- no runtime dependency for users.

## Crate Layout

```text
skill-compress/
├── Cargo.toml
├── Makefile
├── README.md
├── examples/
│   └── sample-skill.md
├── output/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── minifier.rs
│   ├── skill.rs
│   └── llm.rs
├── docs/bmad/
└── skills/
```

## Core Pipeline

```text
read file
  -> split frontmatter/body
  -> parse YAML frontmatter
  -> scan Markdown protected ranges
  -> compute metrics
  -> run deterministic minifier
       -> whitespace/comment cleanup
       -> decorative Markdown normalization
       -> duplicate fenced block references
       -> exact-match business rule references
       -> optional section stripping flags
  -> run diagnostics
  -> optionally run LLM optimizer
  -> render report, diff, or write file
```

## Protected Content

The deterministic minifier must protect:

- YAML frontmatter content semantics;
- fenced code block content, except when an entire duplicate fenced block is replaced by a reference;
- indented code blocks;
- inline code spans;
- Markdown links and reference definitions;
- dynamic command injection syntax such as ``!`command` ``.

The v1 implementation should prefer conservative no-op behavior over risky transformation.

## Markdown Strategy

Use a scanner-based approach for v1 rather than full AST rewrite.

Rationale:

- Markdown AST round-tripping can change formatting unexpectedly;
- SKILL files often include special agent syntax that generic parsers may not understand;
- deterministic cleanup can be implemented safely with protected ranges and exact-match references.

Current deterministic transformations:

- trim trailing whitespace outside fenced blocks;
- collapse repeated blank lines;
- remove single-line HTML comments;
- normalize headings such as `# **Title**` to `# Title`;
- normalize long dash-only horizontal rules to `---`;
- remove decorative blank lines around headings and horizontal rules;
- replace repeated exact-match fenced example blocks with `See earlier identical example block.`;
- assign `BR-xxx` identifiers to repeated exact-match imperative list items and replace later duplicates with `See BR-xxx.`.
- optionally remove changelog/history, examples, and meta-prose sections through explicit flags.

The deterministic minifier does not collapse paraphrases, and there is no LLM rewrite mode that would. Judging semantic equivalence is left to the advisory `--verify-llm` layer, which never rewrites content.

Optional section stripping is heading-driven and conservative:

- section boundaries are determined by ATX heading level;
- fenced blocks are ignored while detecting headings;
- `--keep-latest N` keeps the first N list items from changelog sections;
- `--runtime-only` is an explicit bundle, not the default behavior.

Possible crates:

- `clap` for CLI parsing;
- `serde`, `serde_yaml`, `toml` for config and frontmatter;
- `similar` for diffs;
- `ureq` for synchronous optional LLM HTTP calls;
- `anyhow` or `thiserror` for errors;
- `assert_cmd`, `predicates`, `insta` for tests.

## Diagnostics

Diagnostics should be structured:

```rust
pub enum Severity {
    Error,
    Warning,
    Info,
}

pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}
```

Initial diagnostic codes:

- `frontmatter.missing`
- `frontmatter.invalid_yaml`
- `description.missing`
- `description.too_long`
- `body.too_long`
- `body.too_many_blank_lines`
- `section.possible_reference`
- `llm.provider_unavailable`

Implemented report fields currently include path, status, changed flag, before/after metrics, and diagnostics.

## Reports

Support:

- human text;
- JSON;
- unified diff.

JSON is the contract for CI and future integrations.

## Security

- Never execute skill content.
- Never send file content to an LLM unless `--verify-llm` is explicit.
- Do not run provider calls unless `--verify-llm` is explicit.
- Read provider API keys from environment variables such as `GEMINI_API_KEY`.
- In local-provider mode, still treat the endpoint as external unless explicitly configured as trusted.

## Performance Targets

- Analyze a 500-line skill in under 50 ms, excluding LLM calls.
- Deterministic minify and `--verify` should allocate modestly and avoid full filesystem scans.
- The `--verify-llm` judge has no strict latency target.
