# skill-compress

[![CI](https://github.com/Kantic-Analytics/skill-compress/actions/workflows/ci.yml/badge.svg)](https://github.com/Kantic-Analytics/skill-compress/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

`skill-compress` is a Rust CLI for analyzing and minifying `SKILL.md` files. It is designed for Agent Skills-style repositories where concise instructions, clear frontmatter, and progressive disclosure matter.

The tool is deterministic and local. There is no LLM rewrite mode; the only LLM feature is an optional, advisory verification judge.

## Features

- Validate common `SKILL.md` frontmatter fields such as `name` and `description`.
- Compute line, character, word, and estimated token counts.
- Detect long files, long descriptions, and sections that may belong in `references/`.
- Apply safe deterministic cleanup:
  - trim trailing whitespace outside fenced code blocks;
  - collapse repeated blank lines;
  - remove single-line HTML comments;
  - normalize decorative Markdown headings and horizontal rules;
  - reference duplicate fenced example blocks;
  - assign `BR-xxx` references to repeated exact-match business rules;
  - normalize the final newline.
- Produce human reports, JSON reports, and unified diffs.
- Verify that a compressed candidate preserved every must-preserve atom (`--verify`) — a deterministic fidelity gate that makes no LLM call.
- Optionally adjudicate the residue with an experimental, advisory LLM-as-judge (`--verify-llm`) via Anthropic, OpenAI, Mistral, Gemini, or local OpenAI-compatible endpoints.

## Installation

Prebuilt binaries are produced by GitHub Actions for Linux x86_64, macOS x86_64, macOS arm64, and Windows x86_64. Every `v*` tag publishes those archives as GitHub Release assets.

A user-global install makes `skill-compress` available from any terminal for the current OS user. System-wide installation for every user is optional and requires administrator privileges.

### Cargo

Install globally for the current user from a local checkout:

```bash
cargo install --path .
"$HOME/.cargo/bin/skill-compress" --help
```

Cargo installs the binary into `~/.cargo/bin` by default. If the command is not found, add Cargo's bin directory to your shell profile:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Linux and macOS

Download the matching `.tar.gz` archive from the [latest GitHub Release](https://github.com/Kantic-Analytics/skill-compress/releases/latest), then install it globally for the current user:

```bash
mkdir -p "$HOME/.local/bin"
tar -xzf skill-compress-linux-x86_64.tar.gz
install -m 755 skill-compress "$HOME/.local/bin/skill-compress"
"$HOME/.local/bin/skill-compress" --help
```

Use `skill-compress-macos-x86_64.tar.gz` on Intel Macs and `skill-compress-macos-aarch64.tar.gz` on Apple Silicon Macs.

If `~/.local/bin` is not already on your `PATH`, add it to your shell profile:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

To install for every user on the machine, use a system directory instead:

```bash
sudo install -m 755 skill-compress /usr/local/bin/skill-compress
```

### Windows

Download `skill-compress-windows-x86_64.zip` from the [latest GitHub Release](https://github.com/Kantic-Analytics/skill-compress/releases/latest), then install it globally for the current user from PowerShell:

```powershell
$InstallDir = "$env:LOCALAPPDATA\Programs\skill-compress\bin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Expand-Archive .\skill-compress-windows-x86_64.zip -DestinationPath $InstallDir -Force
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
  [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
}
$env:Path = "$env:Path;$InstallDir"
skill-compress --help
```

Open a new terminal after updating the user `PATH`. To install for every user on the machine, run PowerShell as Administrator and place the binary under `C:\Program Files\skill-compress\bin`, then add that directory to the machine `PATH`.

### Build Without Installing

To build a local binary without adding it to `PATH`:

```bash
cargo build --release
target/release/skill-compress --help
```

The CLI help includes the project banner:

```text
Analyze and minify SKILL.md files
Made with ❤️ by Kantic Analytics

Usage: skill-compress [OPTIONS] <PATH>
```

## Usage

Analyze a skill (the default — prints a human report, makes no changes):

```bash
cargo run -- examples/sample-skill.md
```

`cargo run -- --help` lists every flag grouped by feature: **Output**, **Compression**, **Verification**, and **LLM judge (experimental)**.

**Output** — choose what to do with the deterministic result:

```bash
cargo run -- --diff examples/sample-skill.md          # unified diff, no write
cargo run -- --check examples/sample-skill.md         # CI mode: nonzero if it would change
cargo run -- --report json examples/sample-skill.md   # machine-readable report
make sample                                            # copy to output/, then --write in place
```

**Compression** — optionally strip non-runtime sections (all off by default):

```bash
cargo run -- --runtime-only --diff examples/sample-skill.md
cargo run -- --strip-changelog --keep-latest 1 --diff examples/sample-skill.md
cargo run -- --strip-examples --strip-meta-prose --diff examples/sample-skill.md
cargo run -- --strip-nonessential --diff examples/sample-skill.md
```

**Verification** — check a compressed candidate against the original; see [Verifying Fidelity](#verifying-fidelity).

## LLM Providers (experimental judge only)

The tool does **not** rewrite skills with an LLM — that path proved unreliable (it silently dropped rules, merged distinct constraints, and broke structure). LLM providers are now used only by the experimental `--verify-llm` judge (see [Verifying Fidelity](#verifying-fidelity)), which adjudicates the atoms the deterministic gate reports as missing. No provider call is ever made unless `--verify-llm` is passed.

Supported providers: Anthropic, OpenAI, Mistral, Gemini, and local OpenAI-compatible endpoints (LM Studio, Ollama, llama.cpp).

Configuration (CLI flags fall back to these environment variables):

- `SKILL_COMPRESS_LLM_PROVIDER`
- `SKILL_COMPRESS_LLM_MODEL`
- `SKILL_COMPRESS_LLM_BASE_URL`
- `SKILL_COMPRESS_LLM_MAX_OUTPUT_TOKENS`
- `SKILL_COMPRESS_LLM_TIMEOUT_SECONDS`
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `MISTRAL_API_KEY`
- `GEMINI_API_KEY`
- `SKILL_COMPRESS_LLM_API_KEY`

## Development

Clone the repository, then run:

```bash
make verify
```

Useful commands:

```bash
make help
make fmt
make test
make verify
make sample
make sample-json
make sample-diff
make sample-runtime
make sample-runtime-diff
make sample-verify
make sample-verify-llm
make sample-all
```

The sample targets use `examples/sample-skill.md` and write generated files to `output/`:

- `output/sample-skill.min.md`
- `output/sample-skill.report.json`
- `output/sample-skill.diff`
- `output/sample-skill.runtime.md`
- `output/sample-skill.runtime.diff`

All sample targets are deterministic and make no network call, except `make sample-verify-llm`, which invokes the advisory OpenAI `--verify-llm` judge and requires `OPENAI_API_KEY`.

`output/` is ignored by Git except for `.gitkeep`; generated files should not be committed unless explicitly needed for release documentation.

## Evaluating Compression

Compare outputs against the same source:

```text
original      examples/sample-skill.md
deterministic output/sample-skill.min.md
runtime       output/sample-skill.runtime.md
```

Use separate baselines:

```text
deterministic_gain_vs_original = 1 - deterministic_tokens / original_tokens
runtime_gain_vs_original       = 1 - runtime_tokens / original_tokens
```

Re-run the analyzer on any generated output to confirm it is still a valid, idempotent skill:

```bash
cargo run -- output/sample-skill.min.md
cargo run -- --check output/sample-skill.min.md
```

## Verifying Fidelity

Any compressed candidate (deterministic or runtime-only) can be checked against the original with a deterministic gate that makes no LLM call. It extracts the original's *must-preserve atoms* — frontmatter keys, section headings, rule and acceptance bullets, and fenced code blocks — and reports every one the candidate does not contain, exiting nonzero if any is missing:

```bash
cargo run -- examples/sample-skill.md --verify output/sample-skill.min.md      # 388/388
cargo run -- examples/sample-skill.md --verify output/sample-skill.runtime.md   # lists the sections --runtime-only drops
cargo run -- examples/sample-skill.md --verify output/sample-skill.runtime.md --report json
# or via Make (defaults to auditing the deterministic min output):
make sample-verify
SAMPLE_VERIFY_CANDIDATE=output/sample-skill.runtime.md make sample-verify
```

The deterministic minifier is faithful by construction: `make sample-verify` reports `388/388` on `output/sample-skill.min.md`.

Matching is verbatim after light normalization (whitespace, dash style, and the deterministic minifier's own `[BR-001]`/`See BR-001.` markers are absorbed), so a *paraphrased* rule is reported as missing on purpose — reworded constraints are unverified drift, not proven equivalents.

An **experimental** LLM-as-judge layer can adjudicate the residue — deciding whether each deterministically-missing atom is paraphrased-equivalent, weakened, or truly lost. It sends the candidate to the configured provider and is advisory only: the deterministic result stays authoritative and drives the exit code.

```bash
ANTHROPIC_API_KEY=... cargo run -- examples/sample-skill.md \
  --verify output/sample-skill.runtime.md --verify-llm --provider anthropic
# or via Make with the OpenAI provider (override the model with SAMPLE_LLM_MODEL):
OPENAI_API_KEY=... make sample-verify-llm
OPENAI_API_KEY=... SAMPLE_VERIFY_CANDIDATE=output/sample-skill.runtime.md make sample-verify-llm
```

Project layout:

```text
src/
  lib.rs        CLI orchestration
  main.rs       process entrypoint
  minifier.rs   deterministic cleanup
  skill.rs      SKILL.md analysis and reports
  fidelity.rs   deterministic fidelity gate (--verify)
  llm.rs        provider-neutral LLM calls
examples/       public sample files
output/         local generated sample outputs
```

## Safety

The deterministic minifier is conservative and should be idempotent. Fenced code block contents are preserved, except when an entire repeated fenced block is replaced by a reference — every pass, including business-rule referencing, is fence-aware.

Business-rule referencing is intentionally exact-match only: the first repeated imperative list item receives an identifier such as `BR-001`, and later identical rules become `See BR-001.`. The tool never infers that two differently worded rules mean the same thing.

The aggressive deterministic flags are explicit because they can remove useful authoring context:

- `--strip-changelog`: removes changelog/history sections, or keeps the first N list items with `--keep-latest N`;
- `--strip-examples`: removes example-oriented sections such as `Examples`, `Avoid`, `Prefer`, and `Before/After`;
- `--strip-meta-prose`: removes explanatory sections such as `Reference`, `What it is`, and `Philosophy`;
- `--strip-nonessential`: enables changelog, examples, and meta-prose stripping;
- `--runtime-only`: keeps the skill focused on runtime execution and strips history/examples/meta-prose.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), keep changes focused, add tests for behavior changes, and run:

```bash
make verify
```

## Security

Do not open public issues for vulnerabilities or private document content. See [SECURITY.md](SECURITY.md).

The tool sends skill content to a provider only through the experimental `--verify-llm` judge, and only when that flag is explicit. Do not use it on files containing secrets until redaction is implemented and verified for your use case.

## License

This project is licensed under the [MIT License](LICENSE).
