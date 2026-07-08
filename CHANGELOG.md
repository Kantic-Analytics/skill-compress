# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog conventions where practical.

## Unreleased

### Added

- Rust CLI for analyzing and minifying `SKILL.md` files.
- Deterministic Markdown cleanup and exact-match reference generation.
- Optional compression flags for changelog, examples, meta-prose, and runtime-only output.
- Provider-neutral LLM routing (Anthropic, OpenAI, Mistral, Gemini, local OpenAI-compatible) used by the experimental `--verify-llm` judge; configurable via `--provider`/`--model`/`--base-url`/`--max-output-tokens`/`--timeout-seconds` and `SKILL_COMPRESS_LLM_*` / `<PROVIDER>_API_KEY` env vars.
- Sample Makefile workflows and BMAD specifications.
- Deterministic fidelity gate: `--verify <candidate>` reports every must-preserve atom (frontmatter keys, section headings, rule/acceptance bullets, code blocks) that a compressed candidate dropped, exiting nonzero on any loss (`--report json` supported). No LLM call; normalization absorbs the minifier's own `[BR-001]`/`See BR-001.` reference markers so deterministic output verifies cleanly.
- Experimental LLM-as-judge layer (`--verify-llm`): classifies each deterministically-missing atom as paraphrased-equivalent, weakened, or lost. Advisory only — the deterministic result stays authoritative and drives the exit code; degrades to a warning when the provider is unavailable.
- `make sample-verify` runs the fidelity gate on a candidate vs the sample original (defaults to the deterministic min, override with `SAMPLE_VERIFY_CANDIDATE=...`).

### Fixed

- LLM error reporting surfaces the provider's own message plus a status-class hint (instead of a bare status code), redacts API keys carried in request URLs, and retries transient failures (429/5xx, dropped connection / "Unexpected EOF") with backoff.
- OpenAI o-series reasoning models (`o1`/`o3`/`o4-mini`) now send `max_completion_tokens` and omit `temperature`, so they no longer fail with an HTTP 400.
- The deterministic business-rule referencing pass is now fence-aware: rules shown inside a ```` ```markdown ```` example are no longer rewritten to `See BR-xxx` or counted as duplicates, restoring the "fenced code block contents are never altered" invariant (the fidelity gate flagged this on the tool's own output).

- LLM responses returned as multiple blocks (Gemini `parts[]`, Anthropic `content[]`) are now concatenated instead of truncated to the first block.
- Truncated LLM responses (provider `stop_reason`/`finish_reason`/`finishReason` hitting the token ceiling) now fail with exit code 4 instead of returning a partial response.
- Request timeout raised to 300s to accommodate large inputs.

### Removed

- The LLM rewrite mode (`--llm`) and its per-provider `make sample-llm-*` targets. Across five providers it consistently dropped rules, merged distinct constraints, and broke structure, so the trustworthy path is now deterministic minification plus the `--verify` fidelity gate. Provider infrastructure is retained solely for the experimental `--verify-llm` judge.
