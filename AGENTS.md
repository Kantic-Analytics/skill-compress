# Repository Guidelines

## Project Structure & Module Organization

This repository is building `skill-compress`, a Rust CLI for analyzing and minifying `SKILL.md` files.

- `src/main.rs`: process entrypoint. Core implementation currently lives in `src/lib.rs`, `src/skill.rs`, `src/minifier.rs`, and `src/llm.rs`.
- `docs/bmad/`: product, architecture, LLM service, and quality specifications. Keep these aligned with behavior changes.
- `examples/`: public sample files, including `examples/sample-skill.md`.
- `output/`: local generated deterministic, diff, JSON, and LLM sample outputs.
- `skills/`: additional reference skills used for manual testing.
- `Cargo.toml`: crate metadata and dependencies.
- `Makefile`: development shortcuts, sample generation, and the Gemini sample LLM workflow.

## Build, Test, and Development Commands

- `cargo run`: run the current CLI locally.
- `cargo check --all-targets`: type-check without producing a final binary.
- `cargo build`: build the debug binary.
- `cargo build --release`: build the optimized CLI.
- `cargo test`: run the Rust test suite.
- `cargo fmt --all`: format Rust code.
- `cargo clippy --all-targets --no-deps -- -D warnings`: run lint checks.
- `make verify`: intended full local verification wrapper; keep it synchronized with the Rust workflow.
- `make sample-all`: regenerate deterministic sample outputs in `output/`.
- `make sample-runtime`: generate a stronger runtime-only sample output.
- `make sample-verify`: run the deterministic fidelity gate on a candidate vs the sample original (defaults to the min output; override with `SAMPLE_VERIFY_CANDIDATE=...`).

## Coding Style & Naming Conventions

Use Rust 2021 conventions and `rustfmt` defaults. Prefer small modules with focused responsibilities. Use `snake_case` for files, modules, functions, and variables; `PascalCase` for types; and `SCREAMING_SNAKE_CASE` for constants. Keep CLI behavior deterministic by default, especially around file rewrites.

## Testing Guidelines

Add unit tests near the module under test and CLI/integration tests under `tests/` when command behavior exists. Use fixture skills under `tests/fixtures/` once added. Cover frontmatter parsing, protected Markdown ranges, idempotent minification, exact-match references, JSON reports, and LLM provider error normalization. Real provider calls should not run in normal tests; use fake clients, dry-runs, or recorded fixtures.

## Commit & Pull Request Guidelines

There is no commit history yet, so use concise imperative commit subjects such as `Add deterministic skill minifier`. Include a body when behavior, security, or docs change. Pull requests should summarize the change, list verification commands, link related issues, and update `docs/bmad/` when requirements or architecture shift.

## Security & LLM Configuration

There is no LLM rewrite mode. The only path that sends content to a provider is the experimental `--verify-llm` judge, and only when that flag is explicit. Do not use it on files containing secrets until redaction is implemented, document required environment variables, and treat local endpoints such as LM Studio, Ollama, and llama.cpp as external services unless explicitly trusted.
