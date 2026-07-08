# Product Requirements Document

## Summary

`skill-compress` analyzes, validates, and compresses `SKILL.md` files. It must be safe by default and useful in both human workflows and CI.

## Command Shape

Initial CLI:

```bash
skill-compress SKILL.md
skill-compress --write SKILL.md
skill-compress --check SKILL.md
skill-compress --report json SKILL.md
skill-compress --runtime-only --diff SKILL.md
skill-compress --strip-changelog --keep-latest 1 --diff SKILL.md
skill-compress SKILL.md --verify compressed.md
skill-compress SKILL.md --verify compressed.md --verify-llm --provider anthropic
make sample-verify
```

## Modes

### Analyze

Default mode. Reads a file and prints a human-readable report.

Requirements:

- parse frontmatter;
- compute line, character, word, and estimated token counts;
- report compression opportunities;
- report missing or suspicious metadata;
- exit with code `0` unless the file cannot be parsed.

### Minify

Produces a deterministic cleaned version. This mode can reduce formatting noise and exact duplicate content, but it must not infer equivalence between differently worded rules.

Requirements:

- preserve YAML frontmatter meaning;
- preserve fenced code blocks byte-for-byte except surrounding newline normalization;
- remove trailing whitespace outside protected blocks;
- collapse repeated blank lines outside protected blocks;
- remove safe HTML comments outside protected blocks;
- normalize decorative Markdown headings and long horizontal rules;
- replace repeated exact-match fenced example blocks with references;
- assign `BR-xxx` references to repeated exact-match imperative business rules;
- normalize final newline;
- never reorder Markdown sections in v1.

Deterministic references:

- duplicate fenced blocks: the first block remains, later identical blocks become `See earlier identical example block.`;
- duplicate business rules: the first exact repeated imperative list item becomes `- [BR-001] ...`, later identical rules become `- See BR-001.`;
- near-duplicates and paraphrases are always left unchanged; the tool never merges differently worded rules.

Optional compression flags:

- `--strip-changelog`: remove changelog, release notes, history, or version-history sections;
- `--keep-latest N`: keep only the first N changelog list items when `--strip-changelog` is active;
- `--strip-examples`: remove example-oriented sections such as `Examples`, `Avoid`, `Prefer`, and `Before/After`;
- `--strip-meta-prose`: remove explanatory meta sections such as `Reference`, `What it is`, and `Philosophy`;
- `--strip-nonessential`: bundle changelog, examples, and meta-prose stripping;
- `--runtime-only`: runtime-focused bundle that strips history, examples, and meta-prose.

### Check

CI mode.

Requirements:

- fail if deterministic minification would change the file;
- fail if configured size limits are exceeded;
- fail if required frontmatter fields are missing;
- support stable exit codes.

### Verify

Fidelity gate. Given the original (`PATH`) and a compressed candidate (`--verify CANDIDATE`), report every must-preserve atom the candidate dropped.

Requirements:

- extract the original's must-preserve atoms: frontmatter keys, section headings, rule/acceptance bullets, fenced code blocks;
- match verbatim after light normalization (whitespace, dash style, the minifier's own `[BR-xxx]`/`See BR-xxx` markers);
- report missing atoms grouped by kind; support `--report json`;
- exit nonzero when any must-preserve atom is missing;
- make no network call.

Experimental LLM-as-judge (`--verify-llm`):

- runs only over the deterministically-missing atoms;
- classifies each `preserved` / `weakened` / `lost`;
- advisory only — the deterministic result stays authoritative and drives the exit code;
- there is no LLM rewrite mode; providers are used only here.

Deterministic sample workflow:

- `sample` writes `output/sample-skill.min.md`;
- `sample-diff` writes `output/sample-skill.diff`;
- `sample-runtime` writes `output/sample-skill.runtime.md`;
- `sample-runtime-diff` writes `output/sample-skill.runtime.diff`;
- `sample-verify` runs the fidelity gate on a candidate (default: the min output).

## Configuration

Search order:

1. CLI flags.
2. `skill-compress.toml` in the current directory or ancestors.
3. Environment variables.
4. Built-in defaults.

Example:

```toml
[limits]
max_lines = 500
max_estimated_tokens = 5000
max_description_chars = 1024

[llm]
provider = "anthropic"
model = "claude-sonnet-4-5"
temperature = 0.1
max_output_tokens = 16384
```

## Frontmatter Requirements

The tool should validate common Agent Skills fields:

- `name`
- `description`
- `license`
- `compatibility`
- `metadata`
- `allowed-tools`

Validation levels:

- `error`: malformed YAML, missing body, unreadable file.
- `warning`: missing recommended fields, long description, excessive body length.
- `info`: possible compression or progressive-disclosure improvements.

## Output Requirements

Human report should include:

- file path;
- status;
- before and after metrics;
- warnings;
- suggested actions.

JSON report should include:

```json
{
  "path": "SKILL.md",
  "status": "ok",
  "metrics": {
    "lines": 120,
    "chars": 6400,
    "estimated_tokens": 1600
  },
  "diagnostics": [],
  "changes": []
}
```

## Exit Codes

- `0`: success, no blocking issue.
- `1`: check failed or diagnostics contain errors.
- `2`: invalid CLI usage.
- `3`: I/O error.
- `4`: LLM provider error.

## Acceptance Criteria

- A valid `SKILL.md` can be analyzed, minified, and verified without network access.
- `--write` applies deterministic cleanup, Markdown normalization, and exact-match references.
- `--check` is stable enough for CI.
- Fenced code block contents remain unchanged unless a whole repeated fenced block is replaced by a reference.
- `--verify` exits nonzero when a candidate drops any must-preserve atom.
- The experimental `--verify-llm` judge can target Anthropic, OpenAI, Mistral, Gemini, and local OpenAI-compatible endpoints; it is advisory and never changes the deterministic verdict.
