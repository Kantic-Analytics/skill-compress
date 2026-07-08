# Epics and Stories

## Epic 1 - Project Foundation

### Story 1.1 - Create Rust CLI Skeleton

As a developer, I want a Rust CLI project with a basic command parser so that future features have a stable entrypoint.

Acceptance criteria:

- `cargo build` succeeds.
- `skill-compress --help` shows usage.
- CLI accepts a `SKILL.md` path.

### Story 1.2 - Add Config Loading

As a user, I want CLI flags and a config file so that the tool can be used consistently in projects and CI.

Acceptance criteria:

- loads `skill-compress.toml`;
- CLI flags override config;
- missing config is not an error.

## Epic 2 - Deterministic Analyzer

### Story 2.1 - Parse SKILL.md

As a user, I want the tool to parse YAML frontmatter and Markdown body so that diagnostics are grounded in the file structure.

Acceptance criteria:

- valid frontmatter parses;
- malformed YAML reports a structured error;
- missing frontmatter reports a diagnostic.

### Story 2.2 - Compute Metrics

As a user, I want size metrics so that I can understand whether a skill is too large.

Acceptance criteria:

- reports lines, chars, words, estimated tokens;
- reports description length;
- reports body length.

### Story 2.3 - Detect Common Issues

As a user, I want actionable diagnostics so that I know what to improve.

Acceptance criteria:

- warns on long descriptions;
- warns on large bodies;
- flags repeated blank lines;
- suggests progressive disclosure for long sections.

## Epic 3 - Deterministic Minifier

### Story 3.1 - Safe Deterministic Cleanup

As a user, I want safe deterministic cleanup so that files are smaller without semantic risk.

Acceptance criteria:

- removes trailing whitespace outside protected blocks;
- collapses excessive blank lines outside protected blocks;
- preserves fenced code blocks;
- normalizes decorative Markdown headings and long horizontal rules;
- normalizes final newline.

### Story 3.2 - Exact-Match References

As a user, I want repeated exact content to become references so that repeated rules and examples do not consume unnecessary context.

Acceptance criteria:

- repeated exact-match fenced example blocks are replaced after the first occurrence;
- repeated exact-match imperative list items receive `BR-xxx` identifiers;
- later exact business-rule duplicates become `See BR-xxx.`;
- paraphrases and near-duplicates are not collapsed in deterministic mode.

### Story 3.3 - Write and Check Modes

As a maintainer, I want write and check modes so that the tool works both locally and in CI.

Acceptance criteria:

- `--write` updates the file;
- `--check` exits non-zero when changes are needed;
- output explains what failed.

### Story 3.4 - Optional Runtime Compression Flags

As a user, I want explicit flags for nonessential content so that I can choose stronger compression when runtime usefulness matters more than authoring context.

Acceptance criteria:

- `--strip-changelog` removes changelog/history sections;
- `--keep-latest N` keeps the first N changelog list items;
- `--strip-examples` removes example-oriented sections;
- `--strip-meta-prose` removes explanatory meta sections;
- `--strip-nonessential` bundles changelog, examples, and meta-prose stripping;
- `--runtime-only` applies the runtime-focused bundle.

## Epic 4 - Reports

### Story 4.1 - Human Report

As a user, I want a readable report so that I can quickly act on recommendations.

Acceptance criteria:

- displays status;
- displays metrics;
- displays diagnostics grouped by severity.

### Story 4.2 - JSON Report

As an integrator, I want machine-readable output so that the CLI can feed CI and dashboards.

Acceptance criteria:

- `--report json` emits valid JSON;
- schema contains metrics and diagnostics;
- no unrelated text is printed in JSON mode.

## Epic 5 - LLM Service

### Story 5.1 - Provider Abstraction

As a developer, I want a provider-neutral LLM trait so that cloud and local providers can be added without changing optimizer logic.

Acceptance criteria:

- providers implement a shared interface;
- provider errors normalize into one error type;
- requests include system prompt, user prompt, temperature, and output limit.

### Story 5.2 - Anthropic Provider

As a user, I want to use Claude for skill optimization so that I can get high-quality semantic suggestions.

Acceptance criteria:

- reads `ANTHROPIC_API_KEY`;
- supports configured model;
- returns normalized text or JSON to the `--verify-llm` judge.

### Story 5.3 - OpenAI Provider

As a user, I want to use OpenAI ChatGPT models so that I can choose a common hosted provider.

Acceptance criteria:

- reads `OPENAI_API_KEY`;
- supports configured model;
- uses a compatible chat-style request in v1;
- reasoning models (`o1`/`o3`/`o4-mini`) send `max_completion_tokens` and omit `temperature`.

### Story 5.4 - Mistral Provider

As a user, I want to use Mistral so that I can choose an alternate hosted provider.

Acceptance criteria:

- reads `MISTRAL_API_KEY`;
- supports configured model;
- normalizes provider responses.

### Story 5.5 - Gemini Provider

As a user, I want to use Google Gemini so that I can choose Google-hosted models.

Acceptance criteria:

- reads `GEMINI_API_KEY`;
- maps Gemini content responses into the shared response type.

### Story 5.6 - Local Provider

As a user, I want to use LM Studio, Ollama, or llama.cpp so that skill optimization can run against local models.

Acceptance criteria:

- supports configurable OpenAI-compatible `base_url`;
- supports no API key or dummy API key;
- reports the endpoint used.

## Epic 6 - Fidelity Verification

There is no LLM rewrite mode; the LLM never produces a candidate. This epic covers checking that a compressed candidate preserved the original.

### Story 6.1 - Deterministic Verify

As a user, I want to check that a candidate dropped nothing so that compression is safe.

Acceptance criteria:

- `--verify CANDIDATE` extracts the original's must-preserve atoms and reports every one the candidate lacks;
- matching is verbatim after light, marker-aware normalization;
- exits nonzero when any atom is missing;
- makes no network call; `--report json` is supported.

### Story 6.2 - LLM-as-judge (experimental)

As a maintainer, I want an optional judge over the residue so that paraphrases and true losses are distinguished.

Acceptance criteria:

- `--verify-llm` runs only over the deterministically-missing atoms;
- classifies each `preserved` / `weakened` / `lost`;
- advisory only: never changes the deterministic exit code;
- degrades to a warning when the provider is unavailable.

### Story 6.3 - Compression Evaluation

As a maintainer, I want to compare deterministic and runtime outputs so that compression gains are not double-counted.

Acceptance criteria:

- reports deterministic gain versus original;
- reports runtime gain versus original;
- documents that separate-baseline gains are not additive.

## Epic 7 - Documentation and Distribution

### Story 7.1 - README

As a user, I want setup and usage docs so that I can install and run the tool quickly.

Acceptance criteria:

- explains deterministic modes and the `--verify` fidelity gate;
- documents provider config for the experimental `--verify-llm` judge;
- includes CI example.

### Story 7.2 - Makefile

As a developer, I want common commands so that build, test, lint, and install are simple.

Acceptance criteria:

- `make build`;
- `make test`;
- `make verify`;
- `make sample`;
- `make sample-json`;
- `make sample-diff`;
- `make sample-runtime`;
- `make sample-runtime-diff`;
- `make sample-verify`.
