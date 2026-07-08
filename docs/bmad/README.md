# BMAD Specifications - skill-compress

This folder contains the product and engineering specifications for `skill-compress`, a lightweight CLI that analyzes and compresses `SKILL.md` files.

The documents are organized as a BMAD-style specification set:

- `project-brief.md`: product context, goals, users, constraints.
- `prd.md`: product requirements and acceptance criteria.
- `architecture.md`: technical architecture for the CLI.
- `llm-service.md`: provider-neutral LLM call service design.
- `epics-and-stories.md`: implementation epics and development stories.
- `quality-and-evals.md`: validation strategy, tests, and evaluation rules.

Design principles:

- deterministic compression is the default path;
- LLM-assisted rewriting is optional, explicit, reviewable, and never hidden;
- `SKILL.md` frontmatter and fenced code blocks are treated as protected content;
- exact-match repeated examples and business rules can be replaced with references;
- LLM compression gains must be evaluated against clear baselines and not double-counted;
- output must support CI usage through stable exit codes and machine-readable reports.

Current sample workflow:

- source: `examples/sample-skill.md`;
- deterministic output: `output/sample-skill.min.md`;
- runtime-only output: `output/sample-skill.runtime.md`;
- report: `output/sample-skill.report.json`;
- diff: `output/sample-skill.diff`;
- runtime-only diff: `output/sample-skill.runtime.diff`;
- fidelity check: `make sample-verify` verifies a candidate (default: the min output) against the sample original.
