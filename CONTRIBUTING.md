# Contributing

Thanks for helping improve `skill-compress`.

Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before participating in project discussions.

## Development Setup

Install a recent stable Rust toolchain, then run:

```bash
cargo test
make verify
```

## Workflow

1. Open an issue or discussion for behavior changes.
2. Keep pull requests focused and small.
3. Update `README.md` and `docs/bmad/` when behavior changes.
4. Add tests for new minifier rules, CLI flags, or provider behavior.
5. Run `make verify` before submitting.

## Safety Rules

- Deterministic mode must stay conservative and idempotent.
- Never infer semantic equivalence between differently worded rules; matching is exact.
- Do not send file content to an LLM in tests.
- Do not commit API keys, generated local outputs, or private sample skills.

## Commit Style

Use short imperative subjects:

```text
Add runtime-only compression flag
Fix Gemini provider error handling
```

Include a body when the change affects behavior, security, compatibility, or documentation.
