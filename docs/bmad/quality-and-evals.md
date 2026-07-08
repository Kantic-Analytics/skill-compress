# Quality and Evaluation

## Quality Goals

The tool must be:

- deterministic by default;
- conservative with user content;
- useful in CI;
- explicit about any network or LLM usage;
- easy to test with fixture files.

## Test Strategy

### Unit Tests

Cover:

- frontmatter split;
- YAML parsing;
- metrics;
- protected range detection;
- whitespace minification;
- decorative Markdown normalization;
- duplicate fenced block referencing;
- exact-match business rule referencing with `BR-xxx`;
- optional section stripping flags;
- diagnostics;
- config precedence.

### Fixture Tests

Create fixture `SKILL.md` files for:

- valid minimal skill;
- missing frontmatter;
- malformed YAML;
- long description;
- large body;
- fenced code block with intentional whitespace;
- duplicate fenced code block;
- repeated exact-match imperative business rule;
- changelog section with `--strip-changelog` and `--keep-latest`;
- examples/meta-prose sections with `--runtime-only`;
- dynamic command syntax;
- HTML comments;
- nested lists and tables.

### CLI Tests

Cover:

- `--help`;
- default analyze mode;
- `--check`;
- `--write`;
- `--report json`;
- provider configuration validation without making network calls.
- `--verify <candidate>`: the deterministic fidelity gate (no network). It extracts the original's must-preserve atoms (frontmatter keys, section headings, rule/acceptance bullets, code blocks) and exits nonzero if any is missing. `make sample-verify` runs it on the deterministic min output and must report `388/388` — a regression there means a pass stopped being fence-aware or started dropping content.
- `--verify --verify-llm`: experimental LLM-as-judge over the residue; advisory only and excluded from default/CI runs (it makes a provider call).
- sample targets such as `make sample`, `make sample-json`, `make sample-diff`, and `make sample-verify`.

### LLM Tests

Do not call real providers in normal tests.

Use:

- fake provider implementation;
- recorded JSON fixtures where useful;
- judge-response parsing tests for `--verify-llm`;
- error normalization tests.

## LLM Evaluation Rules

LLM output must be rejected when:

- YAML frontmatter becomes invalid;
- required fields disappear;
- fenced code blocks are changed unexpectedly;
- commands or file paths are removed without explanation;
- safety constraints are weakened;
- optimizer output is not parseable;
- output risk level is `high` and `--force` is not set.

## Compression Metrics

Evaluate each compressor against the same source:

```text
original = examples/sample-skill.md
deterministic = output/sample-skill.min.md
runtime = output/sample-skill.runtime.md
llm = output/sample-skill.gemini.md
```

Use these formulas:

```text
deterministic_gain_vs_original = 1 - deterministic_tokens / original_tokens
runtime_gain_vs_original = 1 - runtime_tokens / original_tokens
llm_gain_vs_original = 1 - llm_tokens / original_tokens
llm_gain_vs_deterministic = 1 - llm_tokens / deterministic_tokens
```

`llm_gain_vs_original` and `llm_gain_vs_deterministic` must not be added together. The second value is the additional reduction from the deterministic output to the LLM output, using the deterministic output as its baseline.

Minimum evaluation report:

- before/after lines, chars, words, and estimated tokens;
- compression ratio for deterministic and LLM outputs;
- structural diagnostics from `skill-compress`;
- diff classification: safe, review, unsafe;
- count of rules or sections removed, rewritten, or referenced.

## Regression Invariants

The deterministic minifier must be idempotent:

```text
minify(minify(input)) == minify(input)
```

Exact-match business rule references must be idempotent:

```text
minify("- [BR-001] Rule\n- See BR-001.\n") == input
```

The parser must preserve protected blocks:

```text
protected_ranges(input) == protected_ranges_after_safe_cleanup(output)
```

The check mode must be stable:

```text
skill-compress --write SKILL.md
skill-compress --check SKILL.md
# exits 0
```

## Manual Acceptance Scenarios

### Scenario 1 - Local CI

Given a valid but messy `SKILL.md`, when the user runs:

```bash
skill-compress --write SKILL.md
skill-compress --check SKILL.md
```

Then the second command exits `0`.

### Scenario 2 - Deterministic Verify

Given an original and a compressed candidate, when the user runs:

```bash
skill-compress SKILL.md --verify candidate.md
```

Then the tool reports missing must-preserve atoms and exits nonzero if any is missing, with no network call.

### Scenario 3 - Sample Verify Target

When the user runs:

```bash
make sample-verify
```

Then the tool verifies `output/sample-skill.min.md` against the sample and reports `388/388`.

### Scenario 4 - Secret Handling

Planned hardening: given a `SKILL.md` containing a token-like value, when the user runs `--verify-llm`, then the provider prompt should contain a redacted placeholder instead of the original secret. Until this is implemented, do not run `--verify-llm` on files containing secrets.

## Definition of Done

For v1:

- deterministic analyzer works;
- deterministic minifier is idempotent;
- deterministic exact-match references are idempotent;
- CI check mode works;
- JSON report is stable;
- LLM provider abstraction exists (used only by the `--verify-llm` judge);
- Anthropic, OpenAI-compatible, Mistral, Gemini, and local provider designs are represented;
- the deterministic fidelity gate (`--verify`) and `make sample-verify` are documented;
- documentation explains the deterministic path and the experimental judge separately.
