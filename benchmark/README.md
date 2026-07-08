# Compression-accuracy benchmark

Measures the core trade-off of `skill-compress`: **fidelity** (how many of the
original's must-preserve atoms survive) against **size / token reduction**,
across every deterministic compression mode.

## Run it

```bash
make benchmark          # or: python3 benchmark/run_benchmark.py
open output/benchmark.html
```

The generator is **offline and reproducible** — it only shells out to
`cargo run` (deterministic minifier + `--verify` fidelity gate). No network
call, no LLM judge. `python3` (stdlib only) is the sole extra requirement.

## What it reports

For each `examples/*.md` input, and each mode:

| Mode | Flags | Intent |
|---|---|---|
| Deterministic min | *(none)* | Conservative cleanup, lossless by construction |
| Runtime-only | `--runtime-only` | Aggressive strip of changelog / examples / meta prose |

Per mode it captures:

- **Fidelity** — `preserved/total` atoms from `--verify --report json`.
- **Size after** — lines, chars, estimated tokens (from `--report json`).
- **Reductions** — char %, token %, line % vs the original.
- **Dropped atoms by kind** — heading / rule / code-block breakdown.

Fidelity below 100% for `--runtime-only` is expected: the dropped atoms are the
optional sections that mode removes on purpose, not regressions. The
`Deterministic min` mode must always report full fidelity
(`388/388` on the bundled sample).

Output is written to `output/benchmark.html` (git-ignored). The report uses the
Carbon IBM palette and is theme-aware (light/dark) and responsive.

## Adding inputs

Drop any `SKILL.md`-style file into `examples/`; it is picked up automatically
as an additional benchmark row on the next run.
