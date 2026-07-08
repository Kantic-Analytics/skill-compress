# Project Brief

## Product

`skill-compress` is a command-line tool that helps authors reduce and improve `SKILL.md` files used by agent-compatible skill systems.

It provides three complementary workflows:

1. A deterministic local analyzer and minifier for safe formatting cleanup, Markdown normalization, and exact-match deduplication.
2. An optional LLM-assisted optimizer for semantic suggestions such as shortening instructions, improving descriptions, and proposing progressive-disclosure splits.
3. A sample workflow for comparing deterministic compression with LLM-assisted compression.

## Problem

Skill files tend to grow over time. Large `SKILL.md` bodies increase context cost, dilute attention, and make activation behavior harder to reason about.

Authors need a fast local tool that can:

- preserve required metadata;
- identify unnecessary verbosity;
- replace exact duplicate examples or business rules with references;
- keep the operational meaning intact;
- recommend when content should move to `references/`, `assets/`, or scripts;
- optionally ask an LLM for semantic compression while keeping edits auditable.

## Users

- Skill authors creating personal or team skills.
- Developers maintaining project-scoped agent skills.
- Teams reviewing skills before committing them to a repo.
- CI pipelines that enforce size and structure limits.

## Goals

- Provide a fast single-binary CLI.
- Make the safe path deterministic and usable offline.
- Support `--check` for CI.
- Support `--report json` for automation.
- Generate local sample outputs in `output/` from `examples/sample-skill.md`.
- Provide a Gemini sample target using `GEMINI_API_KEY`.
- Offer LLM suggestions through a provider-neutral service.
- Support cloud and local LLM providers without coupling the core minifier to one vendor.

## Non-Goals

- Do not create a general-purpose Markdown compressor.
- Do not silently rewrite meaning.
- Do not infer that two differently worded business rules are equivalent in deterministic mode.
- Do not require an LLM for normal use.
- Do not execute code contained in a skill.
- Do not validate every platform-specific extension of every agent client in v1.

## External Guidance

The design follows current Agent Skills guidance:

- skills use frontmatter metadata plus Markdown instructions;
- activation relies heavily on `name` and `description`;
- full skill bodies should stay concise because loaded content remains in context;
- large content should use progressive disclosure through referenced files.

Reference sources:

- https://docs.anthropic.com/en/docs/claude-code/skills
- https://agentskills.io/
- https://agentskills.io/skill-creation/best-practices
- https://agentskills.io/skill-creation/optimizing-descriptions
